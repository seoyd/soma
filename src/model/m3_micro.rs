//! Sprint 103: three independent Soma-specific M3-Micro research agents.
//!
//! This module is deliberately isolated from committee, prospective, and
//! execution paths. It is not an official Mamba-3 implementation.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    ops::Range,
    path::{Path, PathBuf},
    time::Instant,
};

use serde::{Deserialize, Serialize};

use crate::{
    backtest::CandleSeries,
    core::stable_hash_string,
    eval::leakage::row_is_unsafe,
    feature::{log_return, rolling_mean, rolling_std, rolling_zscore, safe_div},
};

use super::momentum_future_prediction_v4::persist_artifact;

const PARAMETER_LIMIT: usize = 500_000;
const STATE_LIMIT: f32 = 256.0;
const PARAMETER_ABS_LIMIT: f32 = 8.0;
const PROBABILITY_EPSILON: f32 = 1e-6;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum M3MicroError {
    InvalidConfiguration,
    InvalidShape,
    InvalidChronology,
    InsufficientHistory,
    UnavailableSourceEvidence,
    NonCausalFormula,
    NonFiniteInput,
    NonFiniteParameter,
    NonFiniteGradient,
    NonFiniteOutput,
    StateExplosion,
    WrongSchema,
    WrongAgent,
    WrongCheckpoint,
    CorruptArtifact,
    ValidationFitForbidden,
    HoldoutAccessForbidden,
    AutomaticMutationForbidden,
    AutomaticPromotionForbidden,
    IneligiblePromotion,
    Io,
}

impl std::fmt::Display for M3MicroError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidConfiguration => "invalid M3-Micro configuration",
            Self::InvalidShape => "M3-Micro tensor shape rejected",
            Self::InvalidChronology => "M3-Micro chronology rejected",
            Self::InsufficientHistory => "insufficient causal history",
            Self::UnavailableSourceEvidence => "UnavailableSourceEvidence",
            Self::NonCausalFormula => "non-causal Formula rejected",
            Self::NonFiniteInput => "NaN/Inf input rejected",
            Self::NonFiniteParameter => "NaN/Inf or unbounded parameter rejected",
            Self::NonFiniteGradient => "NaN/Inf gradient rejected",
            Self::NonFiniteOutput => "NaN/Inf output rejected",
            Self::StateExplosion => "recurrent state explosion rejected",
            Self::WrongSchema => "input schema identity rejected",
            Self::WrongAgent => "agent identity rejected",
            Self::WrongCheckpoint => "checkpoint identity rejected",
            Self::CorruptArtifact => "corrupt artifact rejected",
            Self::ValidationFitForbidden => "validation fit forbidden",
            Self::HoldoutAccessForbidden => "sealed holdout access forbidden",
            Self::AutomaticMutationForbidden => "automatic Formula mutation forbidden",
            Self::AutomaticPromotionForbidden => "automatic promotion forbidden",
            Self::IneligiblePromotion => "challenger is not promotion eligible",
            Self::Io => "local artifact I/O failed",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for M3MicroError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AgentId {
    TrendContinuation,
    VolatilityRegime,
    ReversalDistortion,
}

impl AgentId {
    pub const ORDERED: [Self; 3] = [
        Self::TrendContinuation,
        Self::VolatilityRegime,
        Self::ReversalDistortion,
    ];

    fn seed_offset(self) -> u64 {
        match self {
            Self::TrendContinuation => 0x1031,
            Self::VolatilityRegime => 0x1032,
            Self::ReversalDistortion => 0x1033,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FormulaId {
    LogReturn1,
    LogReturn5,
    LogReturn20,
    VolatilityAdjustedMomentum,
    NormalizedCloseSlope20,
    ReturnSignAgreement5,
    VolumeConfirmedMovement5,
    TrendPersistence20,
    BreakoutDistance20,
    RealizedVolatility5,
    RealizedVolatility10,
    RealizedVolatility20,
    HighLowRangeEstimator,
    VolatilityOfVolatility10,
    RangeExpansion5,
    VolumeShock20,
    CrossTimeframeVolatilityRatio,
    RegimeDuration20,
    ReturnZScore20,
    PriceDeviation20,
    WickRejectionStructure,
    FailedBreakout20,
    VolumeExhaustion20,
    ShortHorizonReversal,
    RangeNormalizedDisplacement,
    LiquidityDistortion,
    OrderFlowImbalance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FormulaSource {
    Ohlcv,
    Quotes,
    OrderFlow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormulaNormalizationPolicy {
    AgentTrainingZScore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FiniteFallbackPolicy {
    ZeroOnZeroDenominator,
    RejectNonFinite,
    UnavailableSourceEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputeCostClass {
    Constant,
    SmallWindow,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormulaSpec {
    pub formula_id: FormulaId,
    pub version: u32,
    pub required_sources: Vec<FormulaSource>,
    pub required_history: usize,
    pub output_dimension: usize,
    pub normalization_policy: FormulaNormalizationPolicy,
    pub finite_fallback_policy: FiniteFallbackPolicy,
    pub causal_only: bool,
    pub compute_cost_class: ComputeCostClass,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormulaRegistry {
    specs: BTreeMap<FormulaId, FormulaSpec>,
    pub registry_digest: String,
}

impl FormulaRegistry {
    pub fn sprint103() -> Self {
        let mut specs = BTreeMap::new();
        let mut insert = |formula_id, required_history, required_sources, fallback| {
            specs.insert(
                formula_id,
                FormulaSpec {
                    formula_id,
                    version: 1,
                    required_sources,
                    required_history,
                    output_dimension: 1,
                    normalization_policy: FormulaNormalizationPolicy::AgentTrainingZScore,
                    finite_fallback_policy: fallback,
                    causal_only: true,
                    compute_cost_class: if required_history <= 2 {
                        ComputeCostClass::Constant
                    } else {
                        ComputeCostClass::SmallWindow
                    },
                },
            );
        };
        let ohlcv = || vec![FormulaSource::Ohlcv];
        insert(
            FormulaId::LogReturn1,
            2,
            ohlcv(),
            FiniteFallbackPolicy::RejectNonFinite,
        );
        insert(
            FormulaId::LogReturn5,
            6,
            ohlcv(),
            FiniteFallbackPolicy::RejectNonFinite,
        );
        insert(
            FormulaId::LogReturn20,
            21,
            ohlcv(),
            FiniteFallbackPolicy::RejectNonFinite,
        );
        insert(
            FormulaId::VolatilityAdjustedMomentum,
            21,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::NormalizedCloseSlope20,
            20,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::ReturnSignAgreement5,
            6,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::VolumeConfirmedMovement5,
            20,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::TrendPersistence20,
            21,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::BreakoutDistance20,
            21,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::RealizedVolatility5,
            6,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::RealizedVolatility10,
            11,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::RealizedVolatility20,
            21,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::HighLowRangeEstimator,
            1,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::VolatilityOfVolatility10,
            21,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::RangeExpansion5,
            6,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::VolumeShock20,
            20,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::CrossTimeframeVolatilityRatio,
            21,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::RegimeDuration20,
            21,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::ReturnZScore20,
            21,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::PriceDeviation20,
            20,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::WickRejectionStructure,
            1,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::FailedBreakout20,
            21,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::VolumeExhaustion20,
            21,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::ShortHorizonReversal,
            6,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::RangeNormalizedDisplacement,
            1,
            ohlcv(),
            FiniteFallbackPolicy::ZeroOnZeroDenominator,
        );
        insert(
            FormulaId::LiquidityDistortion,
            1,
            vec![FormulaSource::Ohlcv, FormulaSource::Quotes],
            FiniteFallbackPolicy::UnavailableSourceEvidence,
        );
        insert(
            FormulaId::OrderFlowImbalance,
            1,
            vec![FormulaSource::OrderFlow],
            FiniteFallbackPolicy::UnavailableSourceEvidence,
        );
        let registry_digest = stable_hash_string(&format!("{specs:?}"));
        Self {
            specs,
            registry_digest,
        }
    }

    pub fn get(&self, formula_id: FormulaId) -> Option<&FormulaSpec> {
        self.specs.get(&formula_id)
    }

    pub fn specs(&self) -> impl Iterator<Item = &FormulaSpec> {
        self.specs.values()
    }

    pub fn schema_digest(&self, formula_ids: &[FormulaId]) -> Result<String, M3MicroError> {
        if formula_ids.is_empty() {
            return Err(M3MicroError::InvalidConfiguration);
        }
        let specs = formula_ids
            .iter()
            .map(|id| self.get(*id).ok_or(M3MicroError::InvalidConfiguration))
            .collect::<Result<Vec<_>, _>>()?;
        if specs.iter().any(|spec| !spec.causal_only) {
            return Err(M3MicroError::NonCausalFormula);
        }
        Ok(stable_hash_string(&format!("{specs:?}")))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentFormulaGenome {
    pub agent_id: AgentId,
    pub generation: u32,
    pub active_formula_ids: Vec<FormulaId>,
    pub input_schema_digest: String,
    pub candidate_parent_digest: Option<String>,
    pub rejected_formula_ids: Vec<FormulaId>,
    pub genome_digest: String,
}

impl AgentFormulaGenome {
    pub fn initial(agent_id: AgentId, registry: &FormulaRegistry) -> Result<Self, M3MicroError> {
        let (active_formula_ids, rejected_formula_ids) = match agent_id {
            AgentId::TrendContinuation => (
                vec![
                    FormulaId::LogReturn1,
                    FormulaId::LogReturn5,
                    FormulaId::LogReturn20,
                    FormulaId::VolatilityAdjustedMomentum,
                    FormulaId::NormalizedCloseSlope20,
                    FormulaId::ReturnSignAgreement5,
                    FormulaId::VolumeConfirmedMovement5,
                    FormulaId::BreakoutDistance20,
                ],
                vec![],
            ),
            AgentId::VolatilityRegime => (
                vec![
                    FormulaId::RealizedVolatility5,
                    FormulaId::RealizedVolatility20,
                    FormulaId::HighLowRangeEstimator,
                    FormulaId::VolatilityOfVolatility10,
                    FormulaId::RangeExpansion5,
                    FormulaId::VolumeShock20,
                    FormulaId::CrossTimeframeVolatilityRatio,
                    FormulaId::RegimeDuration20,
                ],
                vec![],
            ),
            AgentId::ReversalDistortion => (
                vec![
                    FormulaId::ReturnZScore20,
                    FormulaId::PriceDeviation20,
                    FormulaId::WickRejectionStructure,
                    FormulaId::FailedBreakout20,
                    FormulaId::VolumeExhaustion20,
                    FormulaId::ShortHorizonReversal,
                    FormulaId::RangeNormalizedDisplacement,
                    FormulaId::LogReturn1,
                ],
                vec![
                    FormulaId::LiquidityDistortion,
                    FormulaId::OrderFlowImbalance,
                ],
            ),
        };
        Self::build(
            agent_id,
            0,
            active_formula_ids,
            rejected_formula_ids,
            None,
            registry,
        )
    }

    fn build(
        agent_id: AgentId,
        generation: u32,
        active_formula_ids: Vec<FormulaId>,
        rejected_formula_ids: Vec<FormulaId>,
        candidate_parent_digest: Option<String>,
        registry: &FormulaRegistry,
    ) -> Result<Self, M3MicroError> {
        if !(8..=16).contains(&active_formula_ids.len())
            || active_formula_ids.iter().collect::<BTreeSet<_>>().len() != active_formula_ids.len()
        {
            return Err(M3MicroError::InvalidConfiguration);
        }
        let input_schema_digest = registry.schema_digest(&active_formula_ids)?;
        let mut genome = Self {
            agent_id,
            generation,
            active_formula_ids,
            input_schema_digest,
            candidate_parent_digest,
            rejected_formula_ids,
            genome_digest: String::new(),
        };
        genome.genome_digest = genome.computed_digest();
        Ok(genome)
    }

    fn computed_digest(&self) -> String {
        stable_hash_string(&format!(
            "{:?}:{}:{:?}:{}:{:?}:{:?}",
            self.agent_id,
            self.generation,
            self.active_formula_ids,
            self.input_schema_digest,
            self.candidate_parent_digest,
            self.rejected_formula_ids
        ))
    }

    pub fn validate(&self, registry: &FormulaRegistry) -> Result<(), M3MicroError> {
        if self.input_schema_digest != registry.schema_digest(&self.active_formula_ids)?
            || self.genome_digest != self.computed_digest()
        {
            return Err(M3MicroError::WrongSchema);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct FormulaCacheKey {
    evidence_prefix_digest: String,
    formula_id: FormulaId,
    formula_version: u32,
    index: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FormulaResultCache {
    values: BTreeMap<FormulaCacheKey, Vec<f32>>,
}

impl FormulaResultCache {
    pub fn get_or_compute(
        &mut self,
        registry: &FormulaRegistry,
        series: &CandleSeries,
        formula_id: FormulaId,
        index: usize,
    ) -> Result<Vec<f32>, M3MicroError> {
        let spec = registry
            .get(formula_id)
            .ok_or(M3MicroError::InvalidConfiguration)?;
        let key = FormulaCacheKey {
            evidence_prefix_digest: evidence_prefix_digest(series, index)?,
            formula_id,
            formula_version: spec.version,
            index,
        };
        if let Some(value) = self.values.get(&key) {
            return Ok(value.clone());
        }
        let value = compute_formula(spec, series, index)?;
        self.values.insert(key, value.clone());
        Ok(value)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn byte_size(&self) -> usize {
        self.values
            .iter()
            .map(|(key, value)| {
                std::mem::size_of_val(key)
                    + key.evidence_prefix_digest.len()
                    + value.len() * std::mem::size_of::<f32>()
            })
            .sum()
    }

    pub fn digest(&self) -> String {
        stable_hash_string(&format!("{:?}", self.values))
    }
}

fn evidence_prefix_digest(series: &CandleSeries, index: usize) -> Result<String, M3MicroError> {
    let prefix = series
        .candles
        .get(..=index)
        .ok_or(M3MicroError::InvalidShape)?;
    Ok(stable_hash_string(&format!(
        "{}:{:?}:{prefix:?}",
        series.symbol, series.timeframe
    )))
}

fn validate_ohlcv(series: &CandleSeries, index: usize) -> Result<(), M3MicroError> {
    let candles = series
        .candles
        .get(..=index)
        .ok_or(M3MicroError::InvalidShape)?;
    if candles.iter().any(|candle| {
        [
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            candle.volume,
        ]
        .iter()
        .any(|value| !value.is_finite())
            || candle.open <= 0.0
            || candle.high <= 0.0
            || candle.low <= 0.0
            || candle.close <= 0.0
            || candle.volume < 0.0
            || candle.high < candle.low
    }) {
        return Err(M3MicroError::NonFiniteInput);
    }
    Ok(())
}

fn compute_formula(
    spec: &FormulaSpec,
    series: &CandleSeries,
    index: usize,
) -> Result<Vec<f32>, M3MicroError> {
    if !spec.causal_only {
        return Err(M3MicroError::NonCausalFormula);
    }
    if index + 1 < spec.required_history {
        return Err(M3MicroError::InsufficientHistory);
    }
    if spec.required_sources.contains(&FormulaSource::OrderFlow) {
        return Err(M3MicroError::UnavailableSourceEvidence);
    }
    validate_ohlcv(series, index)?;
    let candle = &series.candles[index];
    if spec.required_sources.contains(&FormulaSource::Quotes)
        && (candle.bid.is_none() || candle.ask.is_none())
    {
        return Err(M3MicroError::UnavailableSourceEvidence);
    }
    let closes = series.candles[..=index]
        .iter()
        .map(|value| value.close)
        .collect::<Vec<_>>();
    let volumes = series.candles[..=index]
        .iter()
        .map(|value| value.volume)
        .collect::<Vec<_>>();
    let returns = closes
        .windows(2)
        .map(|pair| log_return(pair[0], pair[1]).ok_or(M3MicroError::NonFiniteInput))
        .collect::<Result<Vec<_>, _>>()?;
    let ranges = series.candles[..=index]
        .iter()
        .map(|value| safe_div(value.high - value.low, value.close))
        .collect::<Vec<_>>();
    let return_n = |window: usize| -> Result<f64, M3MicroError> {
        let previous = *closes
            .get(closes.len().saturating_sub(window + 1))
            .ok_or(M3MicroError::InsufficientHistory)?;
        log_return(previous, *closes.last().unwrap()).ok_or(M3MicroError::NonFiniteInput)
    };
    let realized = |window: usize| -> Result<f64, M3MicroError> {
        let std = rolling_std(&returns, window).ok_or(M3MicroError::InsufficientHistory)?;
        Ok(std * (window as f64).sqrt())
    };
    let volume_z = || rolling_zscore(&volumes, 20).ok_or(M3MicroError::InsufficientHistory);
    let value = match spec.formula_id {
        FormulaId::LogReturn1 => return_n(1)?,
        FormulaId::LogReturn5 => return_n(5)?,
        FormulaId::LogReturn20 => return_n(20)?,
        FormulaId::VolatilityAdjustedMomentum => safe_div(return_n(20)?, realized(20)?),
        FormulaId::NormalizedCloseSlope20 => {
            let start = closes.len() - 20;
            safe_div(closes[index] - closes[start], closes[start]) / 19.0
        }
        FormulaId::ReturnSignAgreement5 => {
            let tail = &returns[returns.len() - 5..];
            tail.iter().map(|value| value.signum()).sum::<f64>() / 5.0
        }
        FormulaId::VolumeConfirmedMovement5 => return_n(5)? * volume_z()?.tanh(),
        FormulaId::TrendPersistence20 => {
            let tail = &returns[returns.len() - 20..];
            let sign = tail.last().unwrap().signum();
            tail.iter()
                .rev()
                .take_while(|value| value.signum() == sign)
                .count() as f64
                * sign
                / 20.0
        }
        FormulaId::BreakoutDistance20 => {
            let prior = &closes[closes.len() - 21..closes.len() - 1];
            let high = prior.iter().copied().reduce(f64::max).unwrap();
            let low = prior.iter().copied().reduce(f64::min).unwrap();
            if candle.close > high {
                safe_div(candle.close - high, high - low)
            } else if candle.close < low {
                -safe_div(low - candle.close, high - low)
            } else {
                0.0
            }
        }
        FormulaId::RealizedVolatility5 => realized(5)?,
        FormulaId::RealizedVolatility10 => realized(10)?,
        FormulaId::RealizedVolatility20 => realized(20)?,
        FormulaId::HighLowRangeEstimator => {
            safe_div((candle.high / candle.low).ln().abs(), 4.0_f64.ln()).sqrt()
        }
        FormulaId::VolatilityOfVolatility10 => {
            let local = returns[returns.len() - 20..]
                .windows(5)
                .map(|window| {
                    let mean = window.iter().sum::<f64>() / window.len() as f64;
                    (window
                        .iter()
                        .map(|value| (value - mean).powi(2))
                        .sum::<f64>()
                        / window.len() as f64)
                        .sqrt()
                })
                .collect::<Vec<_>>();
            rolling_std(&local, 10).unwrap_or(0.0)
        }
        FormulaId::RangeExpansion5 => {
            let previous_mean = ranges[ranges.len() - 6..ranges.len() - 1]
                .iter()
                .sum::<f64>()
                / 5.0;
            safe_div(ranges[index], previous_mean) - 1.0
        }
        FormulaId::VolumeShock20 => volume_z()?,
        FormulaId::CrossTimeframeVolatilityRatio => safe_div(realized(5)?, realized(20)?),
        FormulaId::RegimeDuration20 => {
            let tail = &ranges[ranges.len() - 20..];
            let mean = tail.iter().sum::<f64>() / tail.len() as f64;
            let high_regime = ranges[index] >= mean;
            tail.iter()
                .rev()
                .take_while(|value| (**value >= mean) == high_regime)
                .count() as f64
                / 20.0
                * if high_regime { 1.0 } else { -1.0 }
        }
        FormulaId::ReturnZScore20 => rolling_zscore(&returns, 20).unwrap_or(0.0),
        FormulaId::PriceDeviation20 => {
            let mean = rolling_mean(&closes, 20).ok_or(M3MicroError::InsufficientHistory)?;
            safe_div(candle.close - mean, mean)
        }
        FormulaId::WickRejectionStructure => {
            let range = (candle.high - candle.low).max(f64::EPSILON);
            let upper = candle.high - candle.open.max(candle.close);
            let lower = candle.open.min(candle.close) - candle.low;
            safe_div(lower - upper, range)
        }
        FormulaId::FailedBreakout20 => {
            let prior = &series.candles[series.candles.len().min(index) - 20..index];
            let high = prior
                .iter()
                .map(|value| value.high)
                .reduce(f64::max)
                .unwrap();
            let low = prior
                .iter()
                .map(|value| value.low)
                .reduce(f64::min)
                .unwrap();
            if candle.high > high && candle.close <= high {
                -safe_div(candle.high - high, high - low)
            } else if candle.low < low && candle.close >= low {
                safe_div(low - candle.low, high - low)
            } else {
                0.0
            }
        }
        FormulaId::VolumeExhaustion20 => {
            -return_n(1)?.signum() * (-volume_z()?).max(0.0) * return_n(1)?.abs()
        }
        FormulaId::ShortHorizonReversal => -return_n(1)? * return_n(5)?.signum(),
        FormulaId::RangeNormalizedDisplacement => {
            let midpoint = (candle.high + candle.low) * 0.5;
            safe_div(candle.close - midpoint, candle.high - candle.low)
        }
        FormulaId::LiquidityDistortion => {
            let bid = candle.bid.ok_or(M3MicroError::UnavailableSourceEvidence)?;
            let ask = candle.ask.ok_or(M3MicroError::UnavailableSourceEvidence)?;
            safe_div(candle.close - (bid + ask) * 0.5, ask - bid)
        }
        FormulaId::OrderFlowImbalance => {
            return Err(M3MicroError::UnavailableSourceEvidence);
        }
    };
    if !value.is_finite() {
        return Err(M3MicroError::NonFiniteOutput);
    }
    Ok(vec![value as f32])
}

pub fn build_causal_formula_row(
    registry: &FormulaRegistry,
    cache: &mut FormulaResultCache,
    genome: &AgentFormulaGenome,
    series: &CandleSeries,
    index: usize,
) -> Result<Vec<f32>, M3MicroError> {
    genome.validate(registry)?;
    let mut row = Vec::with_capacity(genome.active_formula_ids.len());
    for formula_id in &genome.active_formula_ids {
        row.extend(cache.get_or_compute(registry, series, *formula_id, index)?);
    }
    if row.len() != genome.active_formula_ids.len() || row.iter().any(|value| !value.is_finite()) {
        return Err(M3MicroError::InvalidShape);
    }
    Ok(row)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroConfig {
    pub input_dim: usize,
    pub d_model: usize,
    pub d_state: usize,
    pub block_count: usize,
    pub expansion: usize,
    pub output_dim: usize,
    pub decay_min: f32,
    pub decay_max: f32,
}

impl M3MicroConfig {
    pub fn for_agent(agent_id: AgentId, input_dim: usize) -> Self {
        Self {
            input_dim,
            d_model: 64,
            d_state: 8,
            block_count: 2,
            expansion: 2,
            output_dim: match agent_id {
                AgentId::TrendContinuation | AgentId::VolatilityRegime => 5,
                AgentId::ReversalDistortion => 6,
            },
            decay_min: 0.01,
            decay_max: 0.98,
        }
    }

    pub fn validate(&self) -> Result<(), M3MicroError> {
        if !(8..=16).contains(&self.input_dim)
            || self.d_model != 64
            || !matches!(self.d_state, 8 | 16)
            || self.block_count != 2
            || self.expansion == 0
            || !(1..=6).contains(&self.output_dim)
            || !self.decay_min.is_finite()
            || !self.decay_max.is_finite()
            || !(0.0..self.decay_max).contains(&self.decay_min)
            || self.decay_max >= 1.0
        {
            return Err(M3MicroError::InvalidConfiguration);
        }
        let count = M3MicroLayout::new(self)?.parameter_count;
        if count > PARAMETER_LIMIT {
            return Err(M3MicroError::InvalidConfiguration);
        }
        Ok(())
    }

    pub fn inner_dim(&self) -> usize {
        self.d_model * self.expansion
    }
}

#[derive(Clone, Debug)]
struct BlockLayout {
    w_in: Range<usize>,
    b_in: Range<usize>,
    w_decay: Range<usize>,
    b_decay: Range<usize>,
    w_prev_gate: Range<usize>,
    b_prev_gate: Range<usize>,
    w_curr_gate: Range<usize>,
    b_curr_gate: Range<usize>,
    decay_state_bias: Range<usize>,
    prev_scale: Range<usize>,
    curr_scale: Range<usize>,
    readout_scale: Range<usize>,
    skip: Range<usize>,
    w_out: Range<usize>,
    b_out: Range<usize>,
}

#[derive(Clone, Debug)]
struct M3MicroLayout {
    w_embed: Range<usize>,
    b_embed: Range<usize>,
    blocks: Vec<BlockLayout>,
    w_head: Range<usize>,
    b_head: Range<usize>,
    parameter_count: usize,
}

#[derive(Default)]
struct LayoutBuilder {
    next: usize,
}

impl LayoutBuilder {
    fn take(&mut self, count: usize) -> Range<usize> {
        let start = self.next;
        self.next = self.next.saturating_add(count);
        start..self.next
    }
}

impl M3MicroLayout {
    fn new(config: &M3MicroConfig) -> Result<Self, M3MicroError> {
        let inner = config.inner_dim();
        let state_len = inner
            .checked_mul(config.d_state)
            .ok_or(M3MicroError::InvalidConfiguration)?;
        let mut builder = LayoutBuilder::default();
        let w_embed = builder.take(config.d_model * config.input_dim);
        let b_embed = builder.take(config.d_model);
        let mut blocks = Vec::with_capacity(config.block_count);
        for _ in 0..config.block_count {
            blocks.push(BlockLayout {
                w_in: builder.take(inner * config.d_model),
                b_in: builder.take(inner),
                w_decay: builder.take(inner * inner),
                b_decay: builder.take(inner),
                w_prev_gate: builder.take(inner * inner),
                b_prev_gate: builder.take(inner),
                w_curr_gate: builder.take(inner * inner),
                b_curr_gate: builder.take(inner),
                decay_state_bias: builder.take(state_len),
                prev_scale: builder.take(state_len),
                curr_scale: builder.take(state_len),
                readout_scale: builder.take(state_len),
                skip: builder.take(inner),
                w_out: builder.take(config.d_model * inner),
                b_out: builder.take(config.d_model),
            });
        }
        let w_head = builder.take(config.output_dim * config.d_model);
        let b_head = builder.take(config.output_dim);
        if builder.next > PARAMETER_LIMIT {
            return Err(M3MicroError::InvalidConfiguration);
        }
        Ok(Self {
            w_embed,
            b_embed,
            blocks,
            w_head,
            b_head,
            parameter_count: builder.next,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroParameters {
    values: Vec<f32>,
}

impl M3MicroParameters {
    fn seeded(config: &M3MicroConfig, seed: u64) -> Result<Self, M3MicroError> {
        let layout = M3MicroLayout::new(config)?;
        let mut rng = DeterministicRng::new(seed);
        let mut values = (0..layout.parameter_count)
            .map(|_| rng.symmetric(0.02))
            .collect::<Vec<_>>();
        for block in &layout.blocks {
            values[block.b_decay.clone()].fill(1.25);
            values[block.decay_state_bias.clone()].fill(0.0);
            values[block.prev_scale.clone()].fill(0.05);
            values[block.curr_scale.clone()].fill(0.05);
            values[block.readout_scale.clone()].fill(0.05);
            values[block.skip.clone()].fill(0.0);
        }
        let parameters = Self { values };
        parameters.validate(config)?;
        Ok(parameters)
    }

    fn validate(&self, config: &M3MicroConfig) -> Result<(), M3MicroError> {
        let expected = M3MicroLayout::new(config)?.parameter_count;
        if self.values.len() != expected {
            return Err(M3MicroError::InvalidShape);
        }
        if self
            .values
            .iter()
            .any(|value| !value.is_finite() || value.abs() > PARAMETER_ABS_LIMIT)
        {
            return Err(M3MicroError::NonFiniteParameter);
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn storage_identity(&self) -> usize {
        self.values.as_ptr() as usize
    }

    pub fn digest(&self) -> String {
        stable_hash_string(&format!("{:?}", self.values))
    }
}

struct DeterministicRng(u64);

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x9e3779b97f4a7c15)
    }

    fn next(&mut self) -> u32 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        (value >> 16) as u32
    }

    fn symmetric(&mut self, scale: f32) -> f32 {
        (self.next() as f32 / u32::MAX as f32 * 2.0 - 1.0) * scale
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroBlockState {
    values: Vec<f32>,
    previous_u: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroState {
    pub blocks: Vec<M3MicroBlockState>,
    pub step_index: usize,
}

impl M3MicroState {
    pub fn zero(config: &M3MicroConfig) -> Result<Self, M3MicroError> {
        config.validate()?;
        let inner = config.inner_dim();
        Ok(Self {
            blocks: (0..config.block_count)
                .map(|_| M3MicroBlockState {
                    values: vec![0.0; inner * config.d_state],
                    previous_u: vec![0.0; inner],
                })
                .collect(),
            step_index: 0,
        })
    }

    pub fn validate(&self, config: &M3MicroConfig) -> Result<(), M3MicroError> {
        let inner = config.inner_dim();
        if self.blocks.len() != config.block_count
            || self.blocks.iter().any(|block| {
                block.values.len() != inner * config.d_state || block.previous_u.len() != inner
            })
        {
            return Err(M3MicroError::InvalidShape);
        }
        if self
            .blocks
            .iter()
            .flat_map(|block| block.values.iter().chain(&block.previous_u))
            .any(|value| !value.is_finite())
        {
            return Err(M3MicroError::NonFiniteOutput);
        }
        if self
            .blocks
            .iter()
            .flat_map(|block| &block.values)
            .any(|value| value.abs() > STATE_LIMIT)
        {
            return Err(M3MicroError::StateExplosion);
        }
        Ok(())
    }

    pub fn digest(&self) -> String {
        stable_hash_string(&format!("{self:?}"))
    }

    pub fn byte_size(&self) -> usize {
        self.blocks
            .iter()
            .map(|block| (block.values.len() + block.previous_u.len()) * std::mem::size_of::<f32>())
            .sum::<usize>()
            + std::mem::size_of::<usize>()
    }

    pub fn storage_identities(&self) -> Vec<usize> {
        self.blocks
            .iter()
            .flat_map(|block| {
                [
                    block.values.as_ptr() as usize,
                    block.previous_u.as_ptr() as usize,
                ]
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroModel {
    pub config: M3MicroConfig,
    pub parameters: M3MicroParameters,
    pub model_identity: String,
}

#[derive(Clone, Debug)]
struct BlockTape {
    h_input: Vec<f32>,
    u: Vec<f32>,
    decay: Vec<f32>,
    prev_gate: Vec<f32>,
    curr_gate: Vec<f32>,
    previous_state: Vec<f32>,
    previous_u: Vec<f32>,
    next_state: Vec<f32>,
    z: Vec<f32>,
    h_output: Vec<f32>,
}

#[derive(Clone, Debug)]
struct StepTape {
    input: Vec<f32>,
    embedded: Vec<f32>,
    blocks: Vec<BlockTape>,
}

#[derive(Clone, Debug)]
struct ForwardTape {
    steps: Vec<StepTape>,
    raw_output: Vec<f32>,
}

impl M3MicroModel {
    pub fn seeded(config: M3MicroConfig, seed: u64) -> Result<Self, M3MicroError> {
        config.validate()?;
        let parameters = M3MicroParameters::seeded(&config, seed)?;
        let mut model = Self {
            config,
            parameters,
            model_identity: String::new(),
        };
        model.refresh_identity();
        Ok(model)
    }

    pub fn validate(&self) -> Result<(), M3MicroError> {
        self.config.validate()?;
        self.parameters.validate(&self.config)?;
        if self.model_identity != self.computed_identity() {
            return Err(M3MicroError::CorruptArtifact);
        }
        Ok(())
    }

    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }

    pub fn parameter_digest(&self) -> String {
        self.parameters.digest()
    }

    fn computed_identity(&self) -> String {
        stable_hash_string(&format!("{:?}:{}", self.config, self.parameters.digest()))
    }

    fn refresh_identity(&mut self) {
        self.model_identity = self.computed_identity();
    }

    pub fn forward(
        &self,
        sequence: &[Vec<f32>],
        state: &mut M3MicroState,
    ) -> Result<Vec<f32>, M3MicroError> {
        Ok(self.forward_internal(sequence, state, false)?.raw_output)
    }

    fn forward_for_training(&self, sequence: &[Vec<f32>]) -> Result<ForwardTape, M3MicroError> {
        let mut state = M3MicroState::zero(&self.config)?;
        self.forward_internal(sequence, &mut state, true)
    }

    fn forward_internal(
        &self,
        sequence: &[Vec<f32>],
        state: &mut M3MicroState,
        record_tape: bool,
    ) -> Result<ForwardTape, M3MicroError> {
        self.validate()?;
        state.validate(&self.config)?;
        if sequence.is_empty()
            || sequence.iter().any(|row| {
                row.len() != self.config.input_dim || row.iter().any(|value| !value.is_finite())
            })
        {
            return Err(M3MicroError::NonFiniteInput);
        }
        let layout = M3MicroLayout::new(&self.config)?;
        let params = &self.parameters.values;
        let inner = self.config.inner_dim();
        let mut tapes = Vec::with_capacity(sequence.len());
        let mut final_hidden = vec![0.0; self.config.d_model];
        for input in sequence {
            let embedded = affine_tanh(
                params,
                &layout.w_embed,
                &layout.b_embed,
                self.config.d_model,
                self.config.input_dim,
                input,
            )?;
            let mut hidden = embedded.clone();
            let mut block_tapes = Vec::with_capacity(self.config.block_count);
            for (block_index, block_layout) in layout.blocks.iter().enumerate() {
                let block_state = &mut state.blocks[block_index];
                let previous_state = block_state.values.clone();
                let previous_u = block_state.previous_u.clone();
                let u = affine_tanh(
                    params,
                    &block_layout.w_in,
                    &block_layout.b_in,
                    inner,
                    self.config.d_model,
                    &hidden,
                )?;
                let decay_channel = affine(
                    params,
                    &block_layout.w_decay,
                    &block_layout.b_decay,
                    inner,
                    inner,
                    &u,
                )?;
                let prev_gate_pre = affine(
                    params,
                    &block_layout.w_prev_gate,
                    &block_layout.b_prev_gate,
                    inner,
                    inner,
                    &u,
                )?;
                let curr_gate_pre = affine(
                    params,
                    &block_layout.w_curr_gate,
                    &block_layout.b_curr_gate,
                    inner,
                    inner,
                    &u,
                )?;
                let prev_gate = prev_gate_pre
                    .iter()
                    .map(|value| sigmoid(*value))
                    .collect::<Vec<_>>();
                let curr_gate = curr_gate_pre
                    .iter()
                    .map(|value| sigmoid(*value))
                    .collect::<Vec<_>>();
                let mut decay = vec![0.0; inner * self.config.d_state];
                let mut next_state = vec![0.0; decay.len()];
                let mut readout = vec![0.0; inner];
                for channel in 0..inner {
                    for state_index in 0..self.config.d_state {
                        let index = channel * self.config.d_state + state_index;
                        let state_bias = params[block_layout.decay_state_bias.start + index];
                        let bounded = sigmoid(decay_channel[channel] + state_bias);
                        decay[index] = self.config.decay_min
                            + (self.config.decay_max - self.config.decay_min) * bounded;
                        let prev_scale = params[block_layout.prev_scale.start + index].tanh();
                        let curr_scale = params[block_layout.curr_scale.start + index].tanh();
                        let next = decay[index] * previous_state[index]
                            + prev_gate[channel] * previous_u[channel] * prev_scale
                            + curr_gate[channel] * u[channel] * curr_scale;
                        if !next.is_finite() {
                            return Err(M3MicroError::NonFiniteOutput);
                        }
                        if next.abs() > STATE_LIMIT {
                            return Err(M3MicroError::StateExplosion);
                        }
                        next_state[index] = next;
                        readout[channel] += next
                            * params[block_layout.readout_scale.start + index].tanh()
                            / self.config.d_state as f32;
                    }
                }
                let z = (0..inner)
                    .map(|channel| {
                        (readout[channel]
                            + sigmoid(params[block_layout.skip.start + channel]) * u[channel])
                            .tanh()
                    })
                    .collect::<Vec<_>>();
                let h_output = affine_tanh(
                    params,
                    &block_layout.w_out,
                    &block_layout.b_out,
                    self.config.d_model,
                    inner,
                    &z,
                )?;
                block_state.values.clone_from(&next_state);
                block_state.previous_u.clone_from(&u);
                if record_tape {
                    block_tapes.push(BlockTape {
                        h_input: hidden,
                        u,
                        decay,
                        prev_gate,
                        curr_gate,
                        previous_state,
                        previous_u,
                        next_state,
                        z,
                        h_output: h_output.clone(),
                    });
                }
                hidden = h_output;
            }
            final_hidden = hidden;
            if record_tape {
                tapes.push(StepTape {
                    input: input.clone(),
                    embedded,
                    blocks: block_tapes,
                });
            }
            state.step_index = state
                .step_index
                .checked_add(1)
                .ok_or(M3MicroError::StateExplosion)?;
            state.validate(&self.config)?;
        }
        let raw_output = affine(
            params,
            &layout.w_head,
            &layout.b_head,
            self.config.output_dim,
            self.config.d_model,
            &final_hidden,
        )?;
        if raw_output.iter().any(|value| !value.is_finite()) {
            return Err(M3MicroError::NonFiniteOutput);
        }
        Ok(ForwardTape {
            steps: tapes,
            raw_output,
        })
    }
}

fn affine(
    params: &[f32],
    weights: &Range<usize>,
    bias: &Range<usize>,
    rows: usize,
    cols: usize,
    input: &[f32],
) -> Result<Vec<f32>, M3MicroError> {
    if input.len() != cols
        || weights.len() != rows * cols
        || bias.len() != rows
        || input.iter().any(|value| !value.is_finite())
    {
        return Err(M3MicroError::InvalidShape);
    }
    let mut output = vec![0.0; rows];
    for row in 0..rows {
        let mut value = params[bias.start + row];
        for col in 0..cols {
            value += params[weights.start + row * cols + col] * input[col];
        }
        if !value.is_finite() {
            return Err(M3MicroError::NonFiniteOutput);
        }
        output[row] = value;
    }
    Ok(output)
}

fn affine_tanh(
    params: &[f32],
    weights: &Range<usize>,
    bias: &Range<usize>,
    rows: usize,
    cols: usize,
    input: &[f32],
) -> Result<Vec<f32>, M3MicroError> {
    Ok(affine(params, weights, bias, rows, cols, input)?
        .into_iter()
        .map(f32::tanh)
        .collect())
}

fn sigmoid(value: f32) -> f32 {
    if value >= 0.0 {
        1.0 / (1.0 + (-value).exp())
    } else {
        let exponential = value.exp();
        exponential / (1.0 + exponential)
    }
}

fn softplus(value: f32) -> f32 {
    if value > 20.0 {
        value
    } else if value < -20.0 {
        value.exp()
    } else {
        (1.0 + value.exp()).ln()
    }
}

fn softmax(values: &[f32]) -> Result<Vec<f32>, M3MicroError> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err(M3MicroError::NonFiniteOutput);
    }
    let maximum = values.iter().copied().reduce(f32::max).unwrap();
    let exponentials = values
        .iter()
        .map(|value| (value - maximum).exp())
        .collect::<Vec<_>>();
    let total = exponentials.iter().sum::<f32>();
    if !total.is_finite() || total <= 0.0 {
        return Err(M3MicroError::NonFiniteOutput);
    }
    Ok(exponentials
        .into_iter()
        .map(|value| value / total)
        .collect())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroTarget {
    pub agent_id: AgentId,
    pub direction_distribution: Option<[f32; 3]>,
    pub future_return: Option<f32>,
    pub continuation: Option<f32>,
    pub future_variance: Option<f32>,
    pub volatility_regime: Option<[f32; 3]>,
    pub risk_abstention: Option<f32>,
    pub reversal: Option<f32>,
    pub failed_breakout: Option<f32>,
}

impl M3MicroTarget {
    pub fn validate(&self) -> Result<(), M3MicroError> {
        let probability = |value: Option<f32>| {
            value.is_some_and(|value| value.is_finite() && (0.0..=1.0).contains(&value))
        };
        let distribution = |value: Option<[f32; 3]>| {
            value.is_some_and(|value| {
                value
                    .iter()
                    .all(|item| item.is_finite() && (0.0..=1.0).contains(item))
                    && (value.iter().sum::<f32>() - 1.0).abs() <= 1e-4
            })
        };
        let valid = match self.agent_id {
            AgentId::TrendContinuation => {
                distribution(self.direction_distribution)
                    && self.future_return.is_some_and(f32::is_finite)
                    && probability(self.continuation)
                    && self.future_variance.is_none()
                    && self.volatility_regime.is_none()
                    && self.risk_abstention.is_none()
                    && self.reversal.is_none()
                    && self.failed_breakout.is_none()
            }
            AgentId::VolatilityRegime => {
                self.future_variance
                    .is_some_and(|value| value.is_finite() && value >= 0.0)
                    && distribution(self.volatility_regime)
                    && probability(self.risk_abstention)
                    && self.direction_distribution.is_none()
                    && self.future_return.is_none()
                    && self.continuation.is_none()
                    && self.reversal.is_none()
                    && self.failed_breakout.is_none()
            }
            AgentId::ReversalDistortion => {
                probability(self.reversal)
                    && probability(self.failed_breakout)
                    && self.future_return.is_some_and(f32::is_finite)
                    && distribution(self.direction_distribution)
                    && self.continuation.is_none()
                    && self.future_variance.is_none()
                    && self.volatility_regime.is_none()
                    && self.risk_abstention.is_none()
            }
        };
        if valid {
            Ok(())
        } else {
            Err(M3MicroError::InvalidConfiguration)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroPrediction {
    pub agent_id: AgentId,
    pub direction_distribution: Option<[f32; 3]>,
    pub expected_return: Option<f32>,
    pub continuation_probability: Option<f32>,
    pub predicted_variance: Option<f32>,
    pub volatility_regime: Option<[f32; 3]>,
    pub risk_abstention_probability: Option<f32>,
    pub reversal_probability: Option<f32>,
    pub failed_breakout_probability: Option<f32>,
    pub confidence: f32,
    pub abstain: bool,
}

impl M3MicroPrediction {
    fn from_raw(agent_id: AgentId, raw: &[f32]) -> Result<Self, M3MicroError> {
        if raw.iter().any(|value| !value.is_finite()) {
            return Err(M3MicroError::NonFiniteOutput);
        }
        let mut prediction = Self {
            agent_id,
            direction_distribution: None,
            expected_return: None,
            continuation_probability: None,
            predicted_variance: None,
            volatility_regime: None,
            risk_abstention_probability: None,
            reversal_probability: None,
            failed_breakout_probability: None,
            confidence: 0.0,
            abstain: false,
        };
        match agent_id {
            AgentId::TrendContinuation if raw.len() == 5 => {
                let distribution = softmax(&raw[..3])?;
                prediction.direction_distribution =
                    Some([distribution[0], distribution[1], distribution[2]]);
                prediction.continuation_probability = Some(sigmoid(raw[3]));
                prediction.expected_return = Some(raw[4].tanh());
                prediction.confidence = distribution.iter().copied().reduce(f32::max).unwrap();
                prediction.abstain = distribution[1] >= distribution[0].max(distribution[2]);
            }
            AgentId::VolatilityRegime if raw.len() == 5 => {
                let distribution = softmax(&raw[1..4])?;
                prediction.predicted_variance = Some(softplus(raw[0]) + PROBABILITY_EPSILON);
                prediction.volatility_regime =
                    Some([distribution[0], distribution[1], distribution[2]]);
                let risk = sigmoid(raw[4]);
                prediction.risk_abstention_probability = Some(risk);
                prediction.confidence = distribution.iter().copied().reduce(f32::max).unwrap();
                prediction.abstain = risk >= 0.5;
            }
            AgentId::ReversalDistortion if raw.len() == 6 => {
                let distribution = softmax(&raw[3..6])?;
                prediction.reversal_probability = Some(sigmoid(raw[0]));
                prediction.failed_breakout_probability = Some(sigmoid(raw[1]));
                prediction.expected_return = Some(raw[2].tanh());
                prediction.direction_distribution =
                    Some([distribution[0], distribution[1], distribution[2]]);
                prediction.confidence = prediction
                    .reversal_probability
                    .unwrap()
                    .max(distribution.iter().copied().reduce(f32::max).unwrap());
                prediction.abstain = distribution[1] >= distribution[0].max(distribution[2])
                    || prediction.reversal_probability.unwrap() < 0.5;
            }
            _ => return Err(M3MicroError::InvalidShape),
        }
        if !prediction.confidence.is_finite() || !(0.0..=1.0).contains(&prediction.confidence) {
            return Err(M3MicroError::NonFiniteOutput);
        }
        Ok(prediction)
    }
}

fn bce_loss_and_gradient(logit: f32, target: f32, calibration: f32) -> (f32, f32) {
    let probability = sigmoid(logit).clamp(PROBABILITY_EPSILON, 1.0 - PROBABILITY_EPSILON);
    let bce = -(target * probability.ln() + (1.0 - target) * (1.0 - probability).ln());
    let residual = probability - target;
    (
        bce + calibration * residual * residual,
        residual + calibration * 2.0 * residual * probability * (1.0 - probability),
    )
}

fn distribution_loss_and_gradient(
    logits: &[f32],
    target: &[f32; 3],
    calibration: f32,
) -> Result<(f32, Vec<f32>), M3MicroError> {
    let probabilities = softmax(logits)?;
    let cross_entropy = probabilities
        .iter()
        .zip(target)
        .map(|(probability, target)| -target * probability.clamp(PROBABILITY_EPSILON, 1.0).ln())
        .sum::<f32>();
    let mut gradient = probabilities
        .iter()
        .zip(target)
        .map(|(probability, target)| probability - target)
        .collect::<Vec<_>>();
    let probability_gradient = probabilities
        .iter()
        .zip(target)
        .map(|(probability, target)| calibration * 2.0 * (probability - target) / 3.0)
        .collect::<Vec<_>>();
    let projection = probability_gradient
        .iter()
        .zip(&probabilities)
        .map(|(gradient, probability)| gradient * probability)
        .sum::<f32>();
    for index in 0..3 {
        gradient[index] += probabilities[index] * (probability_gradient[index] - projection);
    }
    let brier = probabilities
        .iter()
        .zip(target)
        .map(|(probability, target)| (probability - target).powi(2))
        .sum::<f32>()
        / 3.0;
    Ok((cross_entropy + calibration * brier, gradient))
}

fn loss_and_output_gradient(
    agent_id: AgentId,
    raw: &[f32],
    target: &M3MicroTarget,
) -> Result<(f32, Vec<f32>), M3MicroError> {
    target.validate()?;
    if target.agent_id != agent_id {
        return Err(M3MicroError::WrongAgent);
    }
    let calibration = 0.05;
    let mut gradient = vec![0.0; raw.len()];
    let mut loss = 0.0;
    match agent_id {
        AgentId::TrendContinuation if raw.len() == 5 => {
            let (distribution_loss, distribution_gradient) = distribution_loss_and_gradient(
                &raw[..3],
                &target.direction_distribution.unwrap(),
                calibration,
            )?;
            loss += distribution_loss;
            gradient[..3].copy_from_slice(&distribution_gradient);
            let (continuation_loss, continuation_gradient) =
                bce_loss_and_gradient(raw[3], target.continuation.unwrap(), calibration);
            loss += continuation_loss;
            gradient[3] = continuation_gradient;
            let prediction = raw[4].tanh();
            let residual = prediction - target.future_return.unwrap();
            loss += residual * residual;
            gradient[4] = 2.0 * residual * (1.0 - prediction * prediction);
        }
        AgentId::VolatilityRegime if raw.len() == 5 => {
            let prediction = softplus(raw[0]) + PROBABILITY_EPSILON;
            let observed = target.future_variance.unwrap() + PROBABILITY_EPSILON;
            loss += prediction.ln() + observed / prediction;
            gradient[0] =
                (1.0 / prediction - observed / (prediction * prediction)) * sigmoid(raw[0]);
            let (regime_loss, regime_gradient) = distribution_loss_and_gradient(
                &raw[1..4],
                &target.volatility_regime.unwrap(),
                calibration,
            )?;
            loss += regime_loss;
            gradient[1..4].copy_from_slice(&regime_gradient);
            let (risk_loss, risk_gradient) =
                bce_loss_and_gradient(raw[4], target.risk_abstention.unwrap(), calibration);
            loss += risk_loss;
            gradient[4] = risk_gradient;
        }
        AgentId::ReversalDistortion if raw.len() == 6 => {
            let (reversal_loss, reversal_gradient) =
                bce_loss_and_gradient(raw[0], target.reversal.unwrap(), calibration);
            let (breakout_loss, breakout_gradient) =
                bce_loss_and_gradient(raw[1], target.failed_breakout.unwrap(), calibration);
            let prediction = raw[2].tanh();
            let residual = prediction - target.future_return.unwrap();
            let (direction_loss, direction_gradient) = distribution_loss_and_gradient(
                &raw[3..6],
                &target.direction_distribution.unwrap(),
                calibration,
            )?;
            loss += reversal_loss + breakout_loss + residual * residual + direction_loss;
            gradient[0] = reversal_gradient;
            gradient[1] = breakout_gradient;
            gradient[2] = 2.0 * residual * (1.0 - prediction * prediction);
            gradient[3..6].copy_from_slice(&direction_gradient);
        }
        _ => return Err(M3MicroError::InvalidShape),
    }
    if !loss.is_finite() || gradient.iter().any(|value| !value.is_finite()) {
        return Err(M3MicroError::NonFiniteGradient);
    }
    Ok((loss, gradient))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroOptimizerConfig {
    pub learning_rate: f32,
    pub beta1: f32,
    pub beta2: f32,
    pub epsilon: f32,
    pub weight_decay: f32,
    pub gradient_clip_norm: f32,
}

impl Default for M3MicroOptimizerConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.001,
            beta1: 0.9,
            beta2: 0.999,
            epsilon: 1e-8,
            weight_decay: 1e-5,
            gradient_clip_norm: 1.0,
        }
    }
}

impl M3MicroOptimizerConfig {
    fn validate(&self) -> Result<(), M3MicroError> {
        if !self.learning_rate.is_finite()
            || self.learning_rate <= 0.0
            || !self.beta1.is_finite()
            || !(0.0..1.0).contains(&self.beta1)
            || !self.beta2.is_finite()
            || !(0.0..1.0).contains(&self.beta2)
            || !self.epsilon.is_finite()
            || self.epsilon <= 0.0
            || !self.weight_decay.is_finite()
            || self.weight_decay < 0.0
            || !self.gradient_clip_norm.is_finite()
            || self.gradient_clip_norm <= 0.0
        {
            return Err(M3MicroError::InvalidConfiguration);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroOptimizerState {
    pub config: M3MicroOptimizerConfig,
    pub step: u64,
    first_moment: Vec<f32>,
    second_moment: Vec<f32>,
}

impl M3MicroOptimizerState {
    pub fn new(
        parameter_count: usize,
        config: M3MicroOptimizerConfig,
    ) -> Result<Self, M3MicroError> {
        config.validate()?;
        if parameter_count == 0 || parameter_count > PARAMETER_LIMIT {
            return Err(M3MicroError::InvalidConfiguration);
        }
        Ok(Self {
            config,
            step: 0,
            first_moment: vec![0.0; parameter_count],
            second_moment: vec![0.0; parameter_count],
        })
    }

    fn validate(&self, parameter_count: usize) -> Result<(), M3MicroError> {
        self.config.validate()?;
        if self.first_moment.len() != parameter_count || self.second_moment.len() != parameter_count
        {
            return Err(M3MicroError::InvalidShape);
        }
        if self
            .first_moment
            .iter()
            .chain(&self.second_moment)
            .any(|value| !value.is_finite())
        {
            return Err(M3MicroError::NonFiniteParameter);
        }
        Ok(())
    }

    pub fn digest(&self) -> String {
        stable_hash_string(&format!(
            "{:?}:{}:{:?}:{:?}",
            self.config, self.step, self.first_moment, self.second_moment
        ))
    }

    pub fn storage_identities(&self) -> [usize; 2] {
        [
            self.first_moment.as_ptr() as usize,
            self.second_moment.as_ptr() as usize,
        ]
    }

    fn apply(&mut self, model: &mut M3MicroModel, gradients: &[f32]) -> Result<(), M3MicroError> {
        self.validate(model.parameter_count())?;
        if gradients.len() != model.parameter_count() {
            return Err(M3MicroError::InvalidShape);
        }
        if gradients.iter().any(|value| !value.is_finite()) {
            return Err(M3MicroError::NonFiniteGradient);
        }
        let norm = gradients
            .iter()
            .map(|value| value * value)
            .sum::<f32>()
            .sqrt();
        if !norm.is_finite() {
            return Err(M3MicroError::NonFiniteGradient);
        }
        let scale = if norm > self.config.gradient_clip_norm {
            self.config.gradient_clip_norm / norm
        } else {
            1.0
        };
        self.step = self
            .step
            .checked_add(1)
            .ok_or(M3MicroError::NonFiniteGradient)?;
        let correction1 = 1.0 - self.config.beta1.powf(self.step as f32);
        let correction2 = 1.0 - self.config.beta2.powf(self.step as f32);
        for index in 0..gradients.len() {
            let gradient = gradients[index] * scale
                + self.config.weight_decay * model.parameters.values[index];
            self.first_moment[index] =
                self.config.beta1 * self.first_moment[index] + (1.0 - self.config.beta1) * gradient;
            self.second_moment[index] = self.config.beta2 * self.second_moment[index]
                + (1.0 - self.config.beta2) * gradient * gradient;
            let first = self.first_moment[index] / correction1;
            let second = self.second_moment[index] / correction2;
            let update = self.config.learning_rate * first / (second.sqrt() + self.config.epsilon);
            if !update.is_finite() {
                return Err(M3MicroError::NonFiniteGradient);
            }
            model.parameters.values[index] = (model.parameters.values[index] - update)
                .clamp(-PARAMETER_ABS_LIMIT, PARAMETER_ABS_LIMIT);
        }
        model.parameters.validate(&model.config)?;
        self.validate(model.parameter_count())?;
        model.refresh_identity();
        Ok(())
    }
}

fn add_affine_backward(
    params: &[f32],
    weights: &Range<usize>,
    bias: &Range<usize>,
    rows: usize,
    cols: usize,
    input: &[f32],
    output_gradient: &[f32],
    parameter_gradient: &mut [f32],
) -> Result<Vec<f32>, M3MicroError> {
    if input.len() != cols
        || output_gradient.len() != rows
        || weights.len() != rows * cols
        || bias.len() != rows
        || parameter_gradient.len() != params.len()
    {
        return Err(M3MicroError::InvalidShape);
    }
    let mut input_gradient = vec![0.0; cols];
    for row in 0..rows {
        parameter_gradient[bias.start + row] += output_gradient[row];
        for col in 0..cols {
            let index = weights.start + row * cols + col;
            parameter_gradient[index] += output_gradient[row] * input[col];
            input_gradient[col] += params[index] * output_gradient[row];
        }
    }
    Ok(input_gradient)
}

fn add_assign(destination: &mut [f32], source: &[f32]) -> Result<(), M3MicroError> {
    if destination.len() != source.len() {
        return Err(M3MicroError::InvalidShape);
    }
    for (destination, source) in destination.iter_mut().zip(source) {
        *destination += source;
    }
    Ok(())
}

fn model_loss_and_gradients(
    model: &M3MicroModel,
    agent_id: AgentId,
    sequence: &[Vec<f32>],
    target: &M3MicroTarget,
) -> Result<(f32, Vec<f32>), M3MicroError> {
    let tape = model.forward_for_training(sequence)?;
    let (loss, output_gradient) = loss_and_output_gradient(agent_id, &tape.raw_output, target)?;
    let config = &model.config;
    let layout = M3MicroLayout::new(config)?;
    let params = &model.parameters.values;
    let mut gradients = vec![0.0; params.len()];
    let final_hidden = tape
        .steps
        .last()
        .and_then(|step| step.blocks.last())
        .map(|block| &block.h_output)
        .ok_or(M3MicroError::InvalidShape)?;
    let head_hidden_gradient = add_affine_backward(
        params,
        &layout.w_head,
        &layout.b_head,
        config.output_dim,
        config.d_model,
        final_hidden,
        &output_gradient,
        &mut gradients,
    )?;
    let inner = config.inner_dim();
    let state_len = inner * config.d_state;
    let mut future_state_gradients = vec![vec![0.0; state_len]; config.block_count];
    let mut future_previous_u_gradients = vec![vec![0.0; inner]; config.block_count];
    let last_step = tape.steps.len() - 1;
    for (step_index, step) in tape.steps.iter().enumerate().rev() {
        let mut hidden_gradient = if step_index == last_step {
            head_hidden_gradient.clone()
        } else {
            vec![0.0; config.d_model]
        };
        for block_index in (0..config.block_count).rev() {
            let block = &step.blocks[block_index];
            let block_layout = &layout.blocks[block_index];
            let output_pre_gradient = hidden_gradient
                .iter()
                .zip(&block.h_output)
                .map(|(gradient, output)| gradient * (1.0 - output * output))
                .collect::<Vec<_>>();
            let z_gradient = add_affine_backward(
                params,
                &block_layout.w_out,
                &block_layout.b_out,
                config.d_model,
                inner,
                &block.z,
                &output_pre_gradient,
                &mut gradients,
            )?;
            let readout_gradient = z_gradient
                .iter()
                .zip(&block.z)
                .map(|(gradient, z)| gradient * (1.0 - z * z))
                .collect::<Vec<_>>();
            let mut u_gradient = vec![0.0; inner];
            let mut state_gradient = future_state_gradients[block_index].clone();
            for channel in 0..inner {
                let skip = sigmoid(params[block_layout.skip.start + channel]);
                u_gradient[channel] += readout_gradient[channel] * skip;
                gradients[block_layout.skip.start + channel] +=
                    readout_gradient[channel] * block.u[channel] * skip * (1.0 - skip);
                for state_index in 0..config.d_state {
                    let index = channel * config.d_state + state_index;
                    let raw = params[block_layout.readout_scale.start + index];
                    let scale = raw.tanh();
                    state_gradient[index] +=
                        readout_gradient[channel] * scale / config.d_state as f32;
                    gradients[block_layout.readout_scale.start + index] +=
                        readout_gradient[channel] * block.next_state[index] / config.d_state as f32
                            * (1.0 - scale * scale);
                }
            }
            let mut previous_state_gradient = vec![0.0; state_len];
            let mut previous_u_gradient = vec![0.0; inner];
            let mut decay_channel_gradient = vec![0.0; inner];
            let mut prev_gate_gradient = vec![0.0; inner];
            let mut curr_gate_gradient = vec![0.0; inner];
            let decay_range = config.decay_max - config.decay_min;
            for channel in 0..inner {
                for state_index in 0..config.d_state {
                    let index = channel * config.d_state + state_index;
                    let state_derivative = state_gradient[index];
                    previous_state_gradient[index] = state_derivative * block.decay[index];
                    let bounded_decay = (block.decay[index] - config.decay_min) / decay_range;
                    let decay_pre_gradient = state_derivative
                        * block.previous_state[index]
                        * decay_range
                        * bounded_decay
                        * (1.0 - bounded_decay);
                    decay_channel_gradient[channel] += decay_pre_gradient;
                    gradients[block_layout.decay_state_bias.start + index] += decay_pre_gradient;
                    let prev_raw = params[block_layout.prev_scale.start + index];
                    let prev_scale = prev_raw.tanh();
                    prev_gate_gradient[channel] +=
                        state_derivative * block.previous_u[channel] * prev_scale;
                    previous_u_gradient[channel] +=
                        state_derivative * block.prev_gate[channel] * prev_scale;
                    gradients[block_layout.prev_scale.start + index] += state_derivative
                        * block.prev_gate[channel]
                        * block.previous_u[channel]
                        * (1.0 - prev_scale * prev_scale);
                    let curr_raw = params[block_layout.curr_scale.start + index];
                    let curr_scale = curr_raw.tanh();
                    curr_gate_gradient[channel] += state_derivative * block.u[channel] * curr_scale;
                    u_gradient[channel] += state_derivative * block.curr_gate[channel] * curr_scale;
                    gradients[block_layout.curr_scale.start + index] += state_derivative
                        * block.curr_gate[channel]
                        * block.u[channel]
                        * (1.0 - curr_scale * curr_scale);
                }
            }
            let decay_pre_gradient = decay_channel_gradient;
            let prev_gate_pre_gradient = prev_gate_gradient
                .iter()
                .zip(&block.prev_gate)
                .map(|(gradient, gate)| gradient * gate * (1.0 - gate))
                .collect::<Vec<_>>();
            let curr_gate_pre_gradient = curr_gate_gradient
                .iter()
                .zip(&block.curr_gate)
                .map(|(gradient, gate)| gradient * gate * (1.0 - gate))
                .collect::<Vec<_>>();
            let decay_u_gradient = add_affine_backward(
                params,
                &block_layout.w_decay,
                &block_layout.b_decay,
                inner,
                inner,
                &block.u,
                &decay_pre_gradient,
                &mut gradients,
            )?;
            let prev_u_gate_gradient = add_affine_backward(
                params,
                &block_layout.w_prev_gate,
                &block_layout.b_prev_gate,
                inner,
                inner,
                &block.u,
                &prev_gate_pre_gradient,
                &mut gradients,
            )?;
            let curr_u_gate_gradient = add_affine_backward(
                params,
                &block_layout.w_curr_gate,
                &block_layout.b_curr_gate,
                inner,
                inner,
                &block.u,
                &curr_gate_pre_gradient,
                &mut gradients,
            )?;
            add_assign(&mut u_gradient, &decay_u_gradient)?;
            add_assign(&mut u_gradient, &prev_u_gate_gradient)?;
            add_assign(&mut u_gradient, &curr_u_gate_gradient)?;
            add_assign(&mut u_gradient, &future_previous_u_gradients[block_index])?;
            let u_pre_gradient = u_gradient
                .iter()
                .zip(&block.u)
                .map(|(gradient, u)| gradient * (1.0 - u * u))
                .collect::<Vec<_>>();
            hidden_gradient = add_affine_backward(
                params,
                &block_layout.w_in,
                &block_layout.b_in,
                inner,
                config.d_model,
                &block.h_input,
                &u_pre_gradient,
                &mut gradients,
            )?;
            future_state_gradients[block_index] = previous_state_gradient;
            future_previous_u_gradients[block_index] = previous_u_gradient;
        }
        let embed_pre_gradient = hidden_gradient
            .iter()
            .zip(&step.embedded)
            .map(|(gradient, embedded)| gradient * (1.0 - embedded * embedded))
            .collect::<Vec<_>>();
        let _ = add_affine_backward(
            params,
            &layout.w_embed,
            &layout.b_embed,
            config.d_model,
            config.input_dim,
            &step.input,
            &embed_pre_gradient,
            &mut gradients,
        )?;
    }
    if gradients.iter().any(|value| !value.is_finite()) {
        return Err(M3MicroError::NonFiniteGradient);
    }
    Ok((loss, gradients))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetPolicy {
    CostEvidenceDirectionContinuation,
    FutureVarianceAndRegime,
    ReversalDistortionAndDirection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LossPolicy {
    DirectionRpsReturnAndCalibration,
    QlikeRegimeAndCalibration,
    ReversalReturnRpsAndCalibration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidencePolicy {
    DirectionMaximumProbability,
    RegimeMaximumProbability,
    ReversalOrDirectionMaximum,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbstentionPolicy {
    NeutralDominant,
    RiskProbabilityAtLeastHalf,
    NeutralOrLowReversalConfidence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndependentAgentSpec {
    pub agent_id: AgentId,
    pub architecture: String,
    pub target_policy: TargetPolicy,
    pub loss_policy: LossPolicy,
    pub confidence_policy: ConfidencePolicy,
    pub abstention_policy: AbstentionPolicy,
    pub prediction_head_count: usize,
    pub active_research_agent: bool,
    pub prospective_candidate: bool,
    pub live_authority: bool,
    pub trading_authority: bool,
    pub spec_digest: String,
}

impl IndependentAgentSpec {
    fn for_agent(agent_id: AgentId) -> Self {
        let (target_policy, loss_policy, confidence_policy, abstention_policy) = match agent_id {
            AgentId::TrendContinuation => (
                TargetPolicy::CostEvidenceDirectionContinuation,
                LossPolicy::DirectionRpsReturnAndCalibration,
                ConfidencePolicy::DirectionMaximumProbability,
                AbstentionPolicy::NeutralDominant,
            ),
            AgentId::VolatilityRegime => (
                TargetPolicy::FutureVarianceAndRegime,
                LossPolicy::QlikeRegimeAndCalibration,
                ConfidencePolicy::RegimeMaximumProbability,
                AbstentionPolicy::RiskProbabilityAtLeastHalf,
            ),
            AgentId::ReversalDistortion => (
                TargetPolicy::ReversalDistortionAndDirection,
                LossPolicy::ReversalReturnRpsAndCalibration,
                ConfidencePolicy::ReversalOrDirectionMaximum,
                AbstentionPolicy::NeutralOrLowReversalConfidence,
            ),
        };
        let mut spec = Self {
            agent_id,
            architecture: "soma-m3-micro-v1-not-official-mamba3".to_string(),
            target_policy,
            loss_policy,
            confidence_policy,
            abstention_policy,
            prediction_head_count: 3,
            active_research_agent: true,
            prospective_candidate: false,
            live_authority: false,
            trading_authority: false,
            spec_digest: String::new(),
        };
        spec.spec_digest = stable_hash_string(&format!(
            "{:?}:{}:{:?}:{:?}:{:?}:{:?}:{}:{}:{}:{}",
            spec.agent_id,
            spec.architecture,
            spec.target_policy,
            spec.loss_policy,
            spec.confidence_policy,
            spec.abstention_policy,
            spec.prediction_head_count,
            spec.active_research_agent,
            spec.live_authority,
            spec.trading_authority
        ));
        spec
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentNormalizer {
    pub schema_digest: String,
    pub means: Vec<f32>,
    pub scales: Vec<f32>,
    pub fitted_on_start: Option<usize>,
    pub fitted_on_end: Option<usize>,
    pub normalizer_digest: String,
}

impl AgentNormalizer {
    pub fn unfitted(schema_digest: String, width: usize) -> Result<Self, M3MicroError> {
        if schema_digest.is_empty() || width == 0 {
            return Err(M3MicroError::InvalidConfiguration);
        }
        let mut value = Self {
            schema_digest,
            means: vec![0.0; width],
            scales: vec![1.0; width],
            fitted_on_start: None,
            fitted_on_end: None,
            normalizer_digest: String::new(),
        };
        value.refresh_digest();
        Ok(value)
    }

    pub fn fit_development(
        schema_digest: &str,
        rows: &[(usize, Vec<f32>)],
    ) -> Result<Self, M3MicroError> {
        if schema_digest.is_empty() || rows.is_empty() {
            return Err(M3MicroError::InvalidConfiguration);
        }
        let width = rows[0].1.len();
        if width == 0
            || rows
                .iter()
                .any(|(_, row)| row.len() != width || row.iter().any(|value| !value.is_finite()))
            || rows.windows(2).any(|pair| pair[1].0 < pair[0].0)
        {
            return Err(M3MicroError::InvalidShape);
        }
        let mut means = vec![0.0; width];
        for (_, row) in rows {
            for (mean, value) in means.iter_mut().zip(row) {
                *mean += *value;
            }
        }
        for mean in &mut means {
            *mean /= rows.len() as f32;
        }
        let mut scales = vec![0.0; width];
        for (_, row) in rows {
            for index in 0..width {
                scales[index] += (row[index] - means[index]).powi(2);
            }
        }
        for scale in &mut scales {
            *scale = (*scale / rows.len() as f32).sqrt();
            if *scale <= 1e-6 {
                *scale = 1.0;
            }
        }
        let mut value = Self {
            schema_digest: schema_digest.to_string(),
            means,
            scales,
            fitted_on_start: Some(rows.first().unwrap().0),
            fitted_on_end: Some(rows.last().unwrap().0),
            normalizer_digest: String::new(),
        };
        value.refresh_digest();
        value.validate(width)?;
        Ok(value)
    }

    fn refresh_digest(&mut self) {
        self.normalizer_digest = stable_hash_string(&format!(
            "{}:{:?}:{:?}:{:?}:{:?}",
            self.schema_digest, self.means, self.scales, self.fitted_on_start, self.fitted_on_end
        ));
    }

    fn validate(&self, width: usize) -> Result<(), M3MicroError> {
        if self.schema_digest.is_empty()
            || self.means.len() != width
            || self.scales.len() != width
            || self.means.iter().any(|value| !value.is_finite())
            || self
                .scales
                .iter()
                .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err(M3MicroError::InvalidShape);
        }
        let mut copy = self.clone();
        copy.normalizer_digest.clear();
        let expected = stable_hash_string(&format!(
            "{}:{:?}:{:?}:{:?}:{:?}",
            copy.schema_digest, copy.means, copy.scales, copy.fitted_on_start, copy.fitted_on_end
        ));
        if self.normalizer_digest != expected {
            return Err(M3MicroError::CorruptArtifact);
        }
        Ok(())
    }

    pub fn transform(&self, schema_digest: &str, row: &[f32]) -> Result<Vec<f32>, M3MicroError> {
        if schema_digest != self.schema_digest {
            return Err(M3MicroError::WrongSchema);
        }
        self.validate(row.len())?;
        if row.iter().any(|value| !value.is_finite()) {
            return Err(M3MicroError::NonFiniteInput);
        }
        let transformed = row
            .iter()
            .enumerate()
            .map(|(index, value)| (value - self.means[index]) / self.scales[index])
            .collect::<Vec<_>>();
        if transformed.iter().any(|value| !value.is_finite()) {
            return Err(M3MicroError::NonFiniteOutput);
        }
        Ok(transformed)
    }

    pub fn storage_identities(&self) -> [usize; 2] {
        [self.means.as_ptr() as usize, self.scales.as_ptr() as usize]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrainingHistoryEntry {
    pub optimizer_step: u64,
    pub loss: f32,
    pub parameter_digest_before: String,
    pub parameter_digest_after: String,
    pub development_only: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvaluationHistoryEntry {
    pub partition: HistoricalPartition,
    pub source_index: usize,
    pub loss: f32,
    pub prediction_digest: String,
    pub stage_evidence: ValidationStageEvidence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionHistoryEntry {
    pub prior_model_identity: String,
    pub challenger_model_identity: String,
    pub evidence_digest: String,
    pub manual: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IndependentM3MicroAgent {
    pub spec: IndependentAgentSpec,
    pub model: M3MicroModel,
    pub recurrent_state: M3MicroState,
    pub optimizer_state: M3MicroOptimizerState,
    pub normalizer: AgentNormalizer,
    pub formula_genome: AgentFormulaGenome,
    pub training_history: Vec<TrainingHistoryEntry>,
    pub evaluation_history: Vec<EvaluationHistoryEntry>,
    pub checkpoint_identity: String,
    pub artifact_identity: String,
    pub promotion_history: Vec<PromotionHistoryEntry>,
}

impl IndependentM3MicroAgent {
    fn new(agent_id: AgentId, registry: &FormulaRegistry, seed: u64) -> Result<Self, M3MicroError> {
        let formula_genome = AgentFormulaGenome::initial(agent_id, registry)?;
        Self::from_genome(agent_id, formula_genome, seed)
    }

    fn from_genome(
        agent_id: AgentId,
        formula_genome: AgentFormulaGenome,
        seed: u64,
    ) -> Result<Self, M3MicroError> {
        if formula_genome.agent_id != agent_id {
            return Err(M3MicroError::WrongAgent);
        }
        let config = M3MicroConfig::for_agent(agent_id, formula_genome.active_formula_ids.len());
        let model = M3MicroModel::seeded(config.clone(), seed)?;
        let recurrent_state = M3MicroState::zero(&config)?;
        let optimizer_state =
            M3MicroOptimizerState::new(model.parameter_count(), M3MicroOptimizerConfig::default())?;
        let normalizer = AgentNormalizer::unfitted(
            formula_genome.input_schema_digest.clone(),
            config.input_dim,
        )?;
        let checkpoint_identity = stable_hash_string(&format!(
            "unmaterialized-checkpoint:{agent_id:?}:{}",
            model.model_identity
        ));
        let artifact_identity = stable_hash_string(&format!(
            "sprint103-agent-artifact:{agent_id:?}:{}:{}",
            model.model_identity, formula_genome.genome_digest
        ));
        let agent = Self {
            spec: IndependentAgentSpec::for_agent(agent_id),
            model,
            recurrent_state,
            optimizer_state,
            normalizer,
            formula_genome,
            training_history: Vec::new(),
            evaluation_history: Vec::new(),
            checkpoint_identity,
            artifact_identity,
            promotion_history: Vec::new(),
        };
        agent.validate()?;
        Ok(agent)
    }

    pub fn agent_id(&self) -> AgentId {
        self.spec.agent_id
    }

    pub fn validate(&self) -> Result<(), M3MicroError> {
        self.model.validate()?;
        self.recurrent_state.validate(&self.model.config)?;
        self.optimizer_state
            .validate(self.model.parameter_count())?;
        self.normalizer.validate(self.model.config.input_dim)?;
        self.formula_genome
            .validate(&FormulaRegistry::sprint103())?;
        if self.formula_genome.agent_id != self.agent_id()
            || self.formula_genome.input_schema_digest != self.normalizer.schema_digest
            || self.model.config.input_dim != self.formula_genome.active_formula_ids.len()
            || self.model.parameter_count() > PARAMETER_LIMIT
            || self.spec.prediction_head_count > 3
            || !self.spec.active_research_agent
            || self.spec.prospective_candidate
            || self.spec.live_authority
            || self.spec.trading_authority
            || self
                .evaluation_history
                .iter()
                .any(|entry| entry.stage_evidence.validate_complete().is_err())
        {
            return Err(M3MicroError::InvalidConfiguration);
        }
        Ok(())
    }

    pub fn parameter_count(&self) -> usize {
        self.model.parameter_count()
    }

    pub fn parameter_digest(&self) -> String {
        self.model.parameter_digest()
    }

    pub fn state_digest(&self) -> String {
        self.recurrent_state.digest()
    }

    pub fn optimizer_digest(&self) -> String {
        self.optimizer_state.digest()
    }

    pub fn fit_normalizer(
        &mut self,
        partition: HistoricalPartition,
        rows: &[(usize, Vec<f32>)],
    ) -> Result<(), M3MicroError> {
        if partition != HistoricalPartition::Development {
            return Err(M3MicroError::ValidationFitForbidden);
        }
        self.normalizer =
            AgentNormalizer::fit_development(&self.formula_genome.input_schema_digest, rows)?;
        Ok(())
    }

    pub fn normalize_sequence(
        &self,
        schema_digest: &str,
        sequence: &[Vec<f32>],
    ) -> Result<Vec<Vec<f32>>, M3MicroError> {
        if sequence.is_empty() {
            return Err(M3MicroError::InvalidShape);
        }
        sequence
            .iter()
            .map(|row| self.normalizer.transform(schema_digest, row))
            .collect()
    }

    pub fn train_development_step(
        &mut self,
        partition: HistoricalPartition,
        normalized_sequence: &[Vec<f32>],
        target: &M3MicroTarget,
    ) -> Result<f32, M3MicroError> {
        if partition != HistoricalPartition::Development {
            return Err(M3MicroError::ValidationFitForbidden);
        }
        let before = self.parameter_digest();
        let (loss, gradients) =
            model_loss_and_gradients(&self.model, self.agent_id(), normalized_sequence, target)?;
        self.optimizer_state.apply(&mut self.model, &gradients)?;
        let after = self.parameter_digest();
        self.training_history.push(TrainingHistoryEntry {
            optimizer_step: self.optimizer_state.step,
            loss,
            parameter_digest_before: before,
            parameter_digest_after: after,
            development_only: true,
        });
        self.validate()?;
        Ok(loss)
    }

    pub fn infer(
        &mut self,
        schema_digest: &str,
        raw_sequence: &[Vec<f32>],
    ) -> Result<M3MicroPrediction, M3MicroError> {
        let normalized = self.normalize_sequence(schema_digest, raw_sequence)?;
        let raw = self.model.forward(&normalized, &mut self.recurrent_state)?;
        M3MicroPrediction::from_raw(self.agent_id(), &raw)
    }

    fn predict_stateless_raw(
        &self,
        schema_digest: &str,
        raw_sequence: &[Vec<f32>],
    ) -> Result<(Vec<f32>, M3MicroPrediction), M3MicroError> {
        let normalized = self.normalize_sequence(schema_digest, raw_sequence)?;
        let mut state = M3MicroState::zero(&self.model.config)?;
        let raw = self.model.forward(&normalized, &mut state)?;
        let prediction = M3MicroPrediction::from_raw(self.agent_id(), &raw)?;
        Ok((raw, prediction))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroRoster {
    pub roster_version: String,
    agents: BTreeMap<AgentId, IndependentM3MicroAgent>,
    pub roster_digest: String,
}

impl M3MicroRoster {
    pub fn new(seed: u64) -> Result<Self, M3MicroError> {
        let registry = FormulaRegistry::sprint103();
        let agents = AgentId::ORDERED
            .into_iter()
            .map(|agent_id| {
                IndependentM3MicroAgent::new(agent_id, &registry, seed ^ agent_id.seed_offset())
                    .map(|agent| (agent_id, agent))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let mut roster = Self {
            roster_version: "sprint103-m3-micro-three-independent-agents-v1".into(),
            agents,
            roster_digest: String::new(),
        };
        roster.refresh_digest();
        roster.validate()?;
        Ok(roster)
    }

    fn refresh_digest(&mut self) {
        self.roster_digest = stable_hash_string(&format!(
            "{}:{:?}",
            self.roster_version,
            self.agents
                .iter()
                .map(|(id, agent)| (
                    id,
                    &agent.spec.spec_digest,
                    &agent.model.model_identity,
                    &agent.formula_genome.genome_digest,
                ))
                .collect::<Vec<_>>()
        ));
    }

    pub fn validate(&self) -> Result<(), M3MicroError> {
        if self.agents.len() != 3
            || self.agents.keys().copied().collect::<Vec<_>>() != AgentId::ORDERED
            || self.agents.values().any(|agent| agent.validate().is_err())
        {
            return Err(M3MicroError::InvalidConfiguration);
        }
        let parameter_storage = self
            .agents
            .values()
            .map(|agent| agent.model.parameters.storage_identity())
            .collect::<BTreeSet<_>>();
        let optimizer_storage = self
            .agents
            .values()
            .flat_map(|agent| agent.optimizer_state.storage_identities())
            .collect::<BTreeSet<_>>();
        let normalizer_storage = self
            .agents
            .values()
            .flat_map(|agent| agent.normalizer.storage_identities())
            .collect::<BTreeSet<_>>();
        let state_storage = self
            .agents
            .values()
            .flat_map(|agent| agent.recurrent_state.storage_identities())
            .collect::<BTreeSet<_>>();
        if parameter_storage.len() != 3
            || optimizer_storage.len() != 6
            || normalizer_storage.len() != 6
            || state_storage.len() != 12
        {
            return Err(M3MicroError::InvalidConfiguration);
        }
        let mut copy = self.clone();
        copy.roster_digest.clear();
        copy.refresh_digest();
        if self.roster_digest != copy.roster_digest {
            return Err(M3MicroError::CorruptArtifact);
        }
        Ok(())
    }

    pub fn agent(&self, agent_id: AgentId) -> &IndependentM3MicroAgent {
        self.agents
            .get(&agent_id)
            .expect("fixed three-agent roster")
    }

    pub fn agents(&self) -> impl Iterator<Item = &IndependentM3MicroAgent> {
        self.agents.values()
    }

    fn mutate_agent<T>(
        &mut self,
        agent_id: AgentId,
        mutation: impl FnOnce(&mut IndependentM3MicroAgent) -> Result<T, M3MicroError>,
    ) -> Result<T, M3MicroError> {
        self.validate()?;
        let mut candidate = self
            .agents
            .get(&agent_id)
            .ok_or(M3MicroError::WrongAgent)?
            .clone();
        let result = mutation(&mut candidate)?;
        candidate.validate()?;

        let prior_digest = self.roster_digest.clone();
        let prior = self
            .agents
            .insert(agent_id, candidate)
            .ok_or(M3MicroError::WrongAgent)?;
        self.refresh_digest();
        if let Err(error) = self.validate() {
            self.agents.insert(agent_id, prior);
            self.roster_digest = prior_digest;
            return Err(error);
        }
        Ok(result)
    }

    pub fn fit_agent_normalizer(
        &mut self,
        agent_id: AgentId,
        partition: HistoricalPartition,
        rows: &[(usize, Vec<f32>)],
    ) -> Result<(), M3MicroError> {
        self.mutate_agent(agent_id, |agent| agent.fit_normalizer(partition, rows))
    }

    pub fn train_agent_development_step(
        &mut self,
        agent_id: AgentId,
        partition: HistoricalPartition,
        normalized_sequence: &[Vec<f32>],
        target: &M3MicroTarget,
    ) -> Result<f32, M3MicroError> {
        self.mutate_agent(agent_id, |agent| {
            agent.train_development_step(partition, normalized_sequence, target)
        })
    }

    pub fn infer_agent(
        &mut self,
        agent_id: AgentId,
        schema_digest: &str,
        raw_sequence: &[Vec<f32>],
    ) -> Result<M3MicroPrediction, M3MicroError> {
        self.mutate_agent(agent_id, |agent| agent.infer(schema_digest, raw_sequence))
    }

    pub fn predict_validation(
        &self,
        input: &M3MicroValidationInput,
        boundary: &HistoricalPartitionBoundary,
    ) -> Result<(Vec<f32>, M3MicroPrediction), M3MicroError> {
        self.validate()?;
        let agent = self
            .agents
            .get(&input.input.agent_id)
            .ok_or(M3MicroError::WrongAgent)?;
        input.input.validate(agent, boundary)?;
        agent.predict_stateless_raw(&input.input.input_schema_digest, &input.input.raw_sequence)
    }

    fn record_agent_evaluation(
        &mut self,
        agent_id: AgentId,
        entry: EvaluationHistoryEntry,
    ) -> Result<(), M3MicroError> {
        if entry.partition != HistoricalPartition::Validation
            || !entry.loss.is_finite()
            || entry.prediction_digest.is_empty()
        {
            return Err(M3MicroError::InvalidConfiguration);
        }
        entry.stage_evidence.validate_complete()?;
        self.mutate_agent(agent_id, |agent| {
            agent.evaluation_history.push(entry);
            Ok(())
        })
    }

    pub fn materialize_agent_checkpoint(
        &mut self,
        agent_id: AgentId,
        store: &M3MicroCheckpointStore,
    ) -> Result<SavedM3MicroCheckpoint, M3MicroError> {
        self.mutate_agent(agent_id, |agent| store.save(agent))
    }

    pub fn promote_agent_challenger(
        &mut self,
        agent_id: AgentId,
        challenger: FormulaChallenger,
        evidence: &ManualPromotionEvidence,
    ) -> Result<(), M3MicroError> {
        if challenger.challenger.agent_id() != agent_id {
            return Err(M3MicroError::WrongAgent);
        }
        self.mutate_agent(agent_id, |champion| {
            promote_formula_challenger(champion, challenger, evidence)
        })
    }

    pub fn total_parameter_count(&self) -> usize {
        self.agents
            .values()
            .map(|agent| agent.parameter_count())
            .sum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegacyLogisticModel {
    C1AnchorLogistic,
    C2CompactFeatureLogistic,
    C3StrongL2CompactLogistic,
    C4CalibratedCompactLogistic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegacyModelStatus {
    LegacyHistoricalBenchmarkOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyLogisticDisposition {
    pub model: LegacyLogisticModel,
    pub status: LegacyModelStatus,
    pub active_agent: bool,
    pub promotion_eligible: bool,
    pub prospective_candidate: bool,
    pub historical_artifacts_preserved: bool,
}

pub fn legacy_logistic_dispositions() -> [LegacyLogisticDisposition; 4] {
    [
        LegacyLogisticModel::C1AnchorLogistic,
        LegacyLogisticModel::C2CompactFeatureLogistic,
        LegacyLogisticModel::C3StrongL2CompactLogistic,
        LegacyLogisticModel::C4CalibratedCompactLogistic,
    ]
    .map(|model| LegacyLogisticDisposition {
        model,
        status: LegacyModelStatus::LegacyHistoricalBenchmarkOnly,
        active_agent: false,
        promotion_eligible: false,
        prospective_candidate: false,
        historical_artifacts_preserved: true,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NonLearningBaseline {
    TrainingPrevalenceConstantC0,
    AgentSpecificMathematicalBaseline,
    LegacyHistoricalLogisticResult,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormulaMutationAuthority {
    ManualRegistered,
    AutomaticForbidden,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualFormulaMutationPlan {
    pub agent_id: AgentId,
    pub parent_genome_digest: String,
    pub remove_formula_id: FormulaId,
    pub add_formula_id: FormulaId,
    pub authority: FormulaMutationAuthority,
    pub plan_digest: String,
}

impl ManualFormulaMutationPlan {
    pub fn new(
        champion: &IndependentM3MicroAgent,
        remove_formula_id: FormulaId,
        add_formula_id: FormulaId,
    ) -> Self {
        let authority = FormulaMutationAuthority::ManualRegistered;
        let plan_digest = stable_hash_string(&format!(
            "{:?}:{}:{remove_formula_id:?}:{add_formula_id:?}:{authority:?}",
            champion.agent_id(),
            champion.formula_genome.genome_digest
        ));
        Self {
            agent_id: champion.agent_id(),
            parent_genome_digest: champion.formula_genome.genome_digest.clone(),
            remove_formula_id,
            add_formula_id,
            authority,
            plan_digest,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormulaChallenger {
    pub plan: ManualFormulaMutationPlan,
    pub champion_model_identity: String,
    pub challenger: IndependentM3MicroAgent,
    pub challenger_identity: String,
    pub promotion_eligible: bool,
}

pub fn create_formula_challenger(
    champion: &IndependentM3MicroAgent,
    plan: ManualFormulaMutationPlan,
    registry: &FormulaRegistry,
    fresh_seed: u64,
) -> Result<FormulaChallenger, M3MicroError> {
    if plan.authority == FormulaMutationAuthority::AutomaticForbidden {
        return Err(M3MicroError::AutomaticMutationForbidden);
    }
    if plan.agent_id != champion.agent_id()
        || plan.parent_genome_digest != champion.formula_genome.genome_digest
        || !champion
            .formula_genome
            .active_formula_ids
            .contains(&plan.remove_formula_id)
        || champion
            .formula_genome
            .active_formula_ids
            .contains(&plan.add_formula_id)
        || registry.get(plan.add_formula_id).is_none()
    {
        return Err(M3MicroError::InvalidConfiguration);
    }
    let mut active_formula_ids = champion.formula_genome.active_formula_ids.clone();
    let index = active_formula_ids
        .iter()
        .position(|formula_id| *formula_id == plan.remove_formula_id)
        .ok_or(M3MicroError::InvalidConfiguration)?;
    active_formula_ids[index] = plan.add_formula_id;
    let genome = AgentFormulaGenome::build(
        champion.agent_id(),
        champion.formula_genome.generation + 1,
        active_formula_ids,
        champion.formula_genome.rejected_formula_ids.clone(),
        Some(champion.formula_genome.genome_digest.clone()),
        registry,
    )?;
    let challenger = IndependentM3MicroAgent::from_genome(champion.agent_id(), genome, fresh_seed)?;
    if challenger.formula_genome.input_schema_digest == champion.formula_genome.input_schema_digest
        || challenger.model.model_identity == champion.model.model_identity
        || challenger.normalizer.normalizer_digest == champion.normalizer.normalizer_digest
    {
        return Err(M3MicroError::WrongSchema);
    }
    let challenger_identity = stable_hash_string(&format!(
        "{}:{}:{}",
        plan.plan_digest, challenger.formula_genome.genome_digest, challenger.model.model_identity
    ));
    Ok(FormulaChallenger {
        plan,
        champion_model_identity: champion.model.model_identity.clone(),
        challenger,
        challenger_identity,
        promotion_eligible: false,
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualPromotionEvidence {
    pub challenger_identity: String,
    pub development_complete: bool,
    pub validation_complete: bool,
    pub sealed_holdout_reads: usize,
    pub automatic: bool,
    pub evidence_digest: String,
}

pub fn promote_formula_challenger(
    champion: &mut IndependentM3MicroAgent,
    challenger: FormulaChallenger,
    evidence: &ManualPromotionEvidence,
) -> Result<(), M3MicroError> {
    if evidence.automatic {
        return Err(M3MicroError::AutomaticPromotionForbidden);
    }
    if !challenger.promotion_eligible
        || !evidence.development_complete
        || !evidence.validation_complete
        || evidence.sealed_holdout_reads != 0
        || evidence.challenger_identity != challenger.challenger_identity
        || champion.model.model_identity != challenger.champion_model_identity
        || champion.agent_id() != challenger.challenger.agent_id()
        || evidence.evidence_digest.is_empty()
    {
        return Err(M3MicroError::IneligiblePromotion);
    }
    let prior_model_identity = champion.model.model_identity.clone();
    let challenger_model_identity = challenger.challenger.model.model_identity.clone();
    let mut promoted = challenger.challenger;
    promoted.promotion_history = champion.promotion_history.clone();
    promoted.promotion_history.push(PromotionHistoryEntry {
        prior_model_identity,
        challenger_model_identity,
        evidence_digest: evidence.evidence_digest.clone(),
        manual: true,
    });
    *champion = promoted;
    champion.validate()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroCheckpointPayload {
    pub agent_id: AgentId,
    pub spec: IndependentAgentSpec,
    pub model: M3MicroModel,
    pub recurrent_state: M3MicroState,
    pub optimizer_state: M3MicroOptimizerState,
    pub normalizer: AgentNormalizer,
    pub formula_genome: AgentFormulaGenome,
    pub training_history: Vec<TrainingHistoryEntry>,
    pub evaluation_history: Vec<EvaluationHistoryEntry>,
    pub artifact_identity: String,
    pub promotion_history: Vec<PromotionHistoryEntry>,
}

impl M3MicroCheckpointPayload {
    fn from_agent(agent: &IndependentM3MicroAgent) -> Result<Self, M3MicroError> {
        agent.validate()?;
        Ok(Self {
            agent_id: agent.agent_id(),
            spec: agent.spec.clone(),
            model: agent.model.clone(),
            recurrent_state: agent.recurrent_state.clone(),
            optimizer_state: agent.optimizer_state.clone(),
            normalizer: agent.normalizer.clone(),
            formula_genome: agent.formula_genome.clone(),
            training_history: agent.training_history.clone(),
            evaluation_history: agent.evaluation_history.clone(),
            artifact_identity: agent.artifact_identity.clone(),
            promotion_history: agent.promotion_history.clone(),
        })
    }

    fn to_agent(
        &self,
        checkpoint_identity: String,
    ) -> Result<IndependentM3MicroAgent, M3MicroError> {
        let agent = IndependentM3MicroAgent {
            spec: self.spec.clone(),
            model: self.model.clone(),
            recurrent_state: self.recurrent_state.clone(),
            optimizer_state: self.optimizer_state.clone(),
            normalizer: self.normalizer.clone(),
            formula_genome: self.formula_genome.clone(),
            training_history: self.training_history.clone(),
            evaluation_history: self.evaluation_history.clone(),
            checkpoint_identity,
            artifact_identity: self.artifact_identity.clone(),
            promotion_history: self.promotion_history.clone(),
        };
        if self.agent_id != agent.agent_id() {
            return Err(M3MicroError::WrongCheckpoint);
        }
        agent.validate()?;
        Ok(agent)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroCheckpoint {
    pub format_version: u32,
    pub agent_id: AgentId,
    pub payload: M3MicroCheckpointPayload,
    pub checkpoint_digest: String,
}

impl M3MicroCheckpoint {
    pub fn from_agent(agent: &IndependentM3MicroAgent) -> Result<Self, M3MicroError> {
        let payload = M3MicroCheckpointPayload::from_agent(agent)?;
        let mut checkpoint = Self {
            format_version: 2,
            agent_id: agent.agent_id(),
            payload,
            checkpoint_digest: String::new(),
        };
        checkpoint.checkpoint_digest = checkpoint.computed_digest()?;
        Ok(checkpoint)
    }

    fn computed_digest(&self) -> Result<String, M3MicroError> {
        let bytes = serde_json::to_vec(&(self.format_version, self.agent_id, &self.payload))
            .map_err(|_| M3MicroError::CorruptArtifact)?;
        Ok(stable_hash_string(&format!("{bytes:?}")))
    }

    pub fn validate(&self) -> Result<(), M3MicroError> {
        if self.format_version != 2
            || self.agent_id != self.payload.agent_id
            || self.checkpoint_digest != self.computed_digest()?
        {
            return Err(M3MicroError::WrongCheckpoint);
        }
        self.payload
            .to_agent(self.checkpoint_digest.clone())
            .map(|_| ())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, M3MicroError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(|_| M3MicroError::CorruptArtifact)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, M3MicroError> {
        let checkpoint: Self =
            serde_json::from_slice(bytes).map_err(|_| M3MicroError::CorruptArtifact)?;
        checkpoint.validate()?;
        Ok(checkpoint)
    }

    pub fn restore(self) -> Result<IndependentM3MicroAgent, M3MicroError> {
        self.validate()?;
        self.payload.to_agent(self.checkpoint_digest)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedM3MicroCheckpoint {
    pub agent_id: AgentId,
    pub checkpoint_digest: String,
    pub path: PathBuf,
    pub byte_len: usize,
}

#[derive(Clone, Debug)]
pub struct M3MicroCheckpointStore {
    root: PathBuf,
}

impl M3MicroCheckpointStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, M3MicroError> {
        let root = root.into();
        if root.as_os_str().is_empty() {
            return Err(M3MicroError::InvalidConfiguration);
        }
        Ok(Self { root })
    }

    fn path(&self, agent_id: AgentId, digest: &str) -> Result<PathBuf, M3MicroError> {
        if digest.is_empty() || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(M3MicroError::WrongCheckpoint);
        }
        Ok(self
            .root
            .join(format!("{agent_id:?}"))
            .join(format!("{digest}.json")))
    }

    pub fn save(
        &self,
        agent: &mut IndependentM3MicroAgent,
    ) -> Result<SavedM3MicroCheckpoint, M3MicroError> {
        let checkpoint = M3MicroCheckpoint::from_agent(agent)?;
        let bytes = checkpoint.to_bytes()?;
        let digest = checkpoint.checkpoint_digest.clone();
        let path = self.path(agent.agent_id(), &digest)?;
        persist_artifact(&path, &bytes, &digest, |bytes| {
            M3MicroCheckpoint::from_bytes(bytes)
                .map(|value| value.checkpoint_digest)
                .map_err(|error| error.to_string())
        })
        .map_err(|_| M3MicroError::Io)?;
        let reopened_bytes = fs::read(&path).map_err(|_| M3MicroError::Io)?;
        let reopened = M3MicroCheckpoint::from_bytes(&reopened_bytes)?;
        if reopened.agent_id != agent.agent_id()
            || reopened.checkpoint_digest != digest
            || reopened_bytes != bytes
        {
            return Err(M3MicroError::WrongCheckpoint);
        }
        agent.checkpoint_identity = digest.clone();
        agent.validate()?;
        Ok(SavedM3MicroCheckpoint {
            agent_id: agent.agent_id(),
            checkpoint_digest: digest,
            path,
            byte_len: bytes.len(),
        })
    }

    pub fn load(
        &self,
        agent_id: AgentId,
        digest: &str,
    ) -> Result<IndependentM3MicroAgent, M3MicroError> {
        let bytes = fs::read(self.path(agent_id, digest)?).map_err(|_| M3MicroError::Io)?;
        let checkpoint = M3MicroCheckpoint::from_bytes(&bytes)?;
        if checkpoint.agent_id != agent_id || checkpoint.checkpoint_digest != digest {
            return Err(M3MicroError::WrongCheckpoint);
        }
        checkpoint.restore()
    }

    pub fn delete(&self, agent_id: AgentId, digest: &str) -> Result<(), M3MicroError> {
        let path = self.path(agent_id, digest)?;
        fs::remove_file(path).map_err(|_| M3MicroError::Io)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HistoricalPartition {
    Development,
    Validation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalPartitionBoundary {
    pub partition: HistoricalPartition,
    pub start_index: usize,
    pub end_index: usize,
    pub boundary_version: String,
    pub boundary_digest: String,
}

impl HistoricalPartitionBoundary {
    pub fn new(
        partition: HistoricalPartition,
        start_index: usize,
        end_index: usize,
        boundary_version: impl Into<String>,
    ) -> Result<Self, M3MicroError> {
        let mut boundary = Self {
            partition,
            start_index,
            end_index,
            boundary_version: boundary_version.into(),
            boundary_digest: String::new(),
        };
        boundary.boundary_digest = boundary.computed_digest()?;
        boundary.validate()?;
        Ok(boundary)
    }

    fn computed_digest(&self) -> Result<String, M3MicroError> {
        let bytes = serde_json::to_vec(&(
            self.partition,
            self.start_index,
            self.end_index,
            &self.boundary_version,
        ))
        .map_err(|_| M3MicroError::CorruptArtifact)?;
        Ok(stable_hash_string(&format!("{bytes:?}")))
    }

    #[cfg(test)]
    fn refresh_digest(&mut self) -> Result<(), M3MicroError> {
        self.boundary_digest = self.computed_digest()?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), M3MicroError> {
        if self.start_index > self.end_index || self.boundary_version.is_empty() {
            return Err(M3MicroError::InvalidChronology);
        }
        if self.boundary_digest != self.computed_digest()? {
            return Err(M3MicroError::CorruptArtifact);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct M3MicroTargetReference {
    pub agent_id: AgentId,
    pub partition: HistoricalPartition,
    pub target_index: usize,
    pub target_policy: TargetPolicy,
    pub event_digest: String,
    pub target_commitment: Option<String>,
    pub reference_digest: String,
}

impl M3MicroTargetReference {
    pub fn new(
        agent_id: AgentId,
        partition: HistoricalPartition,
        target_index: usize,
        target_policy: TargetPolicy,
        event_digest: impl Into<String>,
        target_commitment: Option<String>,
    ) -> Result<Self, M3MicroError> {
        let mut reference = Self {
            agent_id,
            partition,
            target_index,
            target_policy,
            event_digest: event_digest.into(),
            target_commitment,
            reference_digest: String::new(),
        };
        reference.reference_digest = reference.computed_digest()?;
        reference.validate_integrity()?;
        Ok(reference)
    }

    pub fn commitment_for(target: &M3MicroTarget) -> Result<String, M3MicroError> {
        let bytes = serde_json::to_vec(target).map_err(|_| M3MicroError::CorruptArtifact)?;
        Ok(stable_hash_string(&format!("{bytes:?}")))
    }

    fn computed_digest(&self) -> Result<String, M3MicroError> {
        let bytes = serde_json::to_vec(&(
            self.agent_id,
            self.partition,
            self.target_index,
            self.target_policy,
            &self.event_digest,
            &self.target_commitment,
        ))
        .map_err(|_| M3MicroError::CorruptArtifact)?;
        Ok(stable_hash_string(&format!("{bytes:?}")))
    }

    fn validate_integrity(&self) -> Result<(), M3MicroError> {
        if self.event_digest.is_empty()
            || self
                .target_commitment
                .as_ref()
                .is_some_and(|value| value.is_empty())
            || self.reference_digest != self.computed_digest()?
        {
            return Err(M3MicroError::CorruptArtifact);
        }
        Ok(())
    }

    fn validate_for(
        &self,
        agent: &IndependentM3MicroAgent,
        partition: HistoricalPartition,
        target_index: usize,
    ) -> Result<(), M3MicroError> {
        self.validate_integrity()?;
        if self.agent_id != agent.agent_id()
            || self.partition != partition
            || self.target_index != target_index
            || self.target_policy != agent.spec.target_policy
        {
            return Err(M3MicroError::WrongAgent);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroHistoricalInput {
    pub agent_id: AgentId,
    pub partition: HistoricalPartition,
    pub sequence_start: usize,
    pub sequence_end: usize,
    pub target_index: usize,
    pub horizon_bars: usize,
    pub boundary: HistoricalPartitionBoundary,
    pub input_schema_digest: String,
    pub raw_sequence: Vec<Vec<f32>>,
    pub target_reference: M3MicroTargetReference,
    pub input_digest: String,
}

impl M3MicroHistoricalInput {
    pub fn new(
        agent_id: AgentId,
        partition: HistoricalPartition,
        sequence_start: usize,
        sequence_end: usize,
        target_index: usize,
        horizon_bars: usize,
        boundary: HistoricalPartitionBoundary,
        input_schema_digest: impl Into<String>,
        raw_sequence: Vec<Vec<f32>>,
        target_reference: M3MicroTargetReference,
    ) -> Result<Self, M3MicroError> {
        let mut input = Self {
            agent_id,
            partition,
            sequence_start,
            sequence_end,
            target_index,
            horizon_bars,
            boundary,
            input_schema_digest: input_schema_digest.into(),
            raw_sequence,
            target_reference,
            input_digest: String::new(),
        };
        input.refresh_digest()?;
        Ok(input)
    }

    fn computed_digest(&self) -> Result<String, M3MicroError> {
        let bytes = serde_json::to_vec(&(
            2u32,
            self.agent_id,
            self.partition,
            self.sequence_start,
            self.sequence_end,
            self.target_index,
            self.horizon_bars,
            &self.boundary,
            &self.input_schema_digest,
            &self.raw_sequence,
            &self.target_reference,
        ))
        .map_err(|_| M3MicroError::CorruptArtifact)?;
        Ok(stable_hash_string(&format!("{bytes:?}")))
    }

    fn refresh_digest(&mut self) -> Result<(), M3MicroError> {
        self.input_digest = self.computed_digest()?;
        Ok(())
    }

    fn validate(
        &self,
        agent: &IndependentM3MicroAgent,
        canonical_boundary: &HistoricalPartitionBoundary,
    ) -> Result<(), M3MicroError> {
        canonical_boundary.validate()?;
        self.boundary.validate()?;
        if self.boundary != *canonical_boundary
            || self.partition != canonical_boundary.partition
            || self.agent_id != agent.agent_id()
        {
            return Err(M3MicroError::InvalidChronology);
        }
        self.target_reference
            .validate_for(agent, self.partition, self.target_index)?;
        if self.input_schema_digest != agent.formula_genome.input_schema_digest {
            return Err(M3MicroError::WrongSchema);
        }
        if self.horizon_bars == 0
            || self.sequence_start < canonical_boundary.start_index
            || self.sequence_start > self.sequence_end
            || self.sequence_end >= self.target_index
        {
            return Err(M3MicroError::InvalidChronology);
        }
        let expected_target = self
            .sequence_end
            .checked_add(self.horizon_bars)
            .ok_or(M3MicroError::InvalidChronology)?;
        if self.target_index != expected_target
            || self.target_index > canonical_boundary.end_index
            || row_is_unsafe(
                self.sequence_end,
                canonical_boundary.end_index,
                self.horizon_bars,
            )
        {
            return Err(M3MicroError::InvalidChronology);
        }
        let expected_length = self
            .sequence_end
            .checked_sub(self.sequence_start)
            .and_then(|length| length.checked_add(1))
            .ok_or(M3MicroError::InvalidChronology)?;
        if self.raw_sequence.len() != expected_length
            || self.raw_sequence.iter().any(|row| {
                row.len() != agent.model.config.input_dim
                    || row.iter().any(|value| !value.is_finite())
            })
        {
            return Err(M3MicroError::InvalidShape);
        }
        if self.input_digest != self.computed_digest()? {
            return Err(M3MicroError::CorruptArtifact);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroDevelopmentExample {
    pub input: M3MicroHistoricalInput,
    pub target: M3MicroTarget,
}

impl M3MicroDevelopmentExample {
    fn validate(
        &self,
        agent: &IndependentM3MicroAgent,
        boundary: &HistoricalPartitionBoundary,
    ) -> Result<(), M3MicroError> {
        if self.input.partition != HistoricalPartition::Development {
            return Err(M3MicroError::InvalidChronology);
        }
        self.input.validate(agent, boundary)?;
        self.target.validate()?;
        if self.target.agent_id != self.input.agent_id {
            return Err(M3MicroError::WrongAgent);
        }
        if let Some(commitment) = &self.input.target_reference.target_commitment {
            if commitment != &M3MicroTargetReference::commitment_for(&self.target)? {
                return Err(M3MicroError::CorruptArtifact);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct M3MicroValidationInput {
    pub input: M3MicroHistoricalInput,
}

impl M3MicroValidationInput {
    fn validate(
        &self,
        agent: &IndependentM3MicroAgent,
        boundary: &HistoricalPartitionBoundary,
    ) -> Result<(), M3MicroError> {
        if self.input.partition != HistoricalPartition::Validation {
            return Err(M3MicroError::InvalidChronology);
        }
        self.input.validate(agent, boundary)
    }
}

#[derive(Clone, PartialEq)]
pub struct M3MicroValidationTargetSource {
    targets: BTreeMap<String, M3MicroTarget>,
    access_count: usize,
}

impl std::fmt::Debug for M3MicroValidationTargetSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("M3MicroValidationTargetSource")
            .field("registered_target_count", &self.targets.len())
            .field("access_count", &self.access_count)
            .finish()
    }
}

impl M3MicroValidationTargetSource {
    pub fn new(
        registrations: Vec<(M3MicroTargetReference, M3MicroTarget)>,
    ) -> Result<Self, M3MicroError> {
        let mut targets = BTreeMap::new();
        for (reference, target) in registrations {
            reference.validate_integrity()?;
            if targets.insert(reference.reference_digest, target).is_some() {
                return Err(M3MicroError::InvalidConfiguration);
            }
        }
        Ok(Self {
            targets,
            access_count: 0,
        })
    }

    pub fn access_count(&self) -> usize {
        self.access_count
    }

    fn registered_reference_digests(&self) -> BTreeSet<String> {
        self.targets.keys().cloned().collect()
    }

    fn reveal(
        &mut self,
        verified_prediction: &VerifiedPersistedPrediction,
        target_reference: &M3MicroTargetReference,
    ) -> Result<M3MicroTarget, M3MicroError> {
        target_reference.validate_integrity()?;
        verified_prediction.validate_target_reference(target_reference)?;
        self.access_count = self
            .access_count
            .checked_add(1)
            .ok_or(M3MicroError::InvalidConfiguration)?;
        let target = self
            .targets
            .get(&target_reference.reference_digest)
            .ok_or(M3MicroError::CorruptArtifact)?
            .clone();
        target.validate()?;
        if target.agent_id != target_reference.agent_id {
            return Err(M3MicroError::WrongAgent);
        }
        if let Some(commitment) = &target_reference.target_commitment {
            if commitment != &M3MicroTargetReference::commitment_for(&target)? {
                return Err(M3MicroError::CorruptArtifact);
            }
        }
        Ok(target)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct M3MicroHistoricalDataset {
    pub boundaries: BTreeMap<HistoricalPartition, HistoricalPartitionBoundary>,
    pub development: BTreeMap<AgentId, Vec<M3MicroDevelopmentExample>>,
    pub validation_inputs: BTreeMap<AgentId, Vec<M3MicroValidationInput>>,
    pub validation_targets: M3MicroValidationTargetSource,
}

impl M3MicroHistoricalDataset {
    pub fn target_access_count(&self) -> usize {
        self.validation_targets.access_count()
    }

    fn validate(&self, roster: &M3MicroRoster) -> Result<(), M3MicroError> {
        let expected_partitions = BTreeSet::from([
            HistoricalPartition::Development,
            HistoricalPartition::Validation,
        ]);
        if self.boundaries.keys().copied().collect::<BTreeSet<_>>() != expected_partitions
            || self.development.keys().copied().collect::<Vec<_>>() != AgentId::ORDERED
            || self.validation_inputs.keys().copied().collect::<Vec<_>>() != AgentId::ORDERED
        {
            return Err(M3MicroError::InvalidConfiguration);
        }
        let development_boundary = self
            .boundaries
            .get(&HistoricalPartition::Development)
            .ok_or(M3MicroError::InvalidConfiguration)?;
        let validation_boundary = self
            .boundaries
            .get(&HistoricalPartition::Validation)
            .ok_or(M3MicroError::InvalidConfiguration)?;
        development_boundary.validate()?;
        validation_boundary.validate()?;
        if development_boundary.end_index >= validation_boundary.start_index {
            return Err(M3MicroError::InvalidChronology);
        }

        let mut expected_references = BTreeSet::new();
        let mut validation_count = 0usize;
        for agent_id in AgentId::ORDERED {
            let agent = roster.agent(agent_id);
            let development = self
                .development
                .get(&agent_id)
                .ok_or(M3MicroError::InvalidConfiguration)?;
            let validation = self
                .validation_inputs
                .get(&agent_id)
                .ok_or(M3MicroError::InvalidConfiguration)?;
            if development.is_empty()
                || validation.is_empty()
                || development
                    .iter()
                    .any(|example| example.validate(agent, development_boundary).is_err())
                || validation
                    .iter()
                    .any(|input| input.validate(agent, validation_boundary).is_err())
                || development.windows(2).any(|pair| {
                    pair[1].input.sequence_end <= pair[0].input.sequence_end
                        || pair[1].input.target_index <= pair[0].input.target_index
                })
                || validation.windows(2).any(|pair| {
                    pair[1].input.sequence_end <= pair[0].input.sequence_end
                        || pair[1].input.target_index <= pair[0].input.target_index
                })
            {
                return Err(M3MicroError::InvalidChronology);
            }
            for input in validation {
                validation_count = validation_count
                    .checked_add(1)
                    .ok_or(M3MicroError::InvalidConfiguration)?;
                if !expected_references
                    .insert(input.input.target_reference.reference_digest.clone())
                {
                    return Err(M3MicroError::InvalidConfiguration);
                }
            }
        }
        if expected_references.len() != validation_count
            || expected_references != self.validation_targets.registered_reference_digests()
        {
            return Err(M3MicroError::InvalidConfiguration);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStage {
    InputValidated,
    PredictionComputed,
    PredictionPersisted,
    PredictionReopenedAndVerified,
    TargetRevealed,
    Evaluated,
}

impl ValidationStage {
    const ORDERED: [Self; 6] = [
        Self::InputValidated,
        Self::PredictionComputed,
        Self::PredictionPersisted,
        Self::PredictionReopenedAndVerified,
        Self::TargetRevealed,
        Self::Evaluated,
    ];
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationStageEvidence {
    pub stages: Vec<ValidationStage>,
}

impl ValidationStageEvidence {
    fn advance(&mut self, stage: ValidationStage) -> Result<(), M3MicroError> {
        if ValidationStage::ORDERED.get(self.stages.len()).copied() != Some(stage) {
            return Err(M3MicroError::CorruptArtifact);
        }
        self.stages.push(stage);
        Ok(())
    }

    fn validate_complete(&self) -> Result<(), M3MicroError> {
        if self.stages == ValidationStage::ORDERED {
            Ok(())
        } else {
            Err(M3MicroError::CorruptArtifact)
        }
    }

    fn prediction_before_reveal(&self) -> bool {
        self.validate_complete().is_ok()
            && self
                .stages
                .iter()
                .position(|stage| *stage == ValidationStage::PredictionReopenedAndVerified)
                < self
                    .stages
                    .iter()
                    .position(|stage| *stage == ValidationStage::TargetRevealed)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint103SafetyCounters {
    pub network_requests: usize,
    pub live_market_requests: usize,
    pub sealed_holdout_reads: usize,
    pub live_predictions: usize,
    pub live_outcome_reads: usize,
    pub paper_trades: usize,
    pub live_trades: usize,
    pub orders: usize,
    pub account_accesses: usize,
    pub winner_selections: usize,
    pub chair_executions: usize,
    pub chair_learning_steps: usize,
    pub committee_votes: usize,
    pub reward_applications: usize,
    pub penalty_applications: usize,
    pub automatic_promotions: usize,
    pub automatic_formula_mutations: usize,
    pub prospective_state_writes: usize,
}

impl Sprint103SafetyCounters {
    pub fn validate_zero(&self) -> Result<(), M3MicroError> {
        let values = [
            self.network_requests,
            self.live_market_requests,
            self.sealed_holdout_reads,
            self.live_predictions,
            self.live_outcome_reads,
            self.paper_trades,
            self.live_trades,
            self.orders,
            self.account_accesses,
            self.winner_selections,
            self.chair_executions,
            self.chair_learning_steps,
            self.committee_votes,
            self.reward_applications,
            self.penalty_applications,
            self.automatic_promotions,
            self.automatic_formula_mutations,
            self.prospective_state_writes,
        ];
        if values.iter().all(|value| *value == 0) {
            Ok(())
        } else {
            Err(M3MicroError::HoldoutAccessForbidden)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedValidationPrediction {
    version: u32,
    agent_id: AgentId,
    partition: HistoricalPartition,
    sequence_start: usize,
    sequence_end: usize,
    target_index: usize,
    horizon_bars: usize,
    partition_start_index: usize,
    partition_end_index: usize,
    partition_boundary_digest: String,
    model_identity: String,
    schema_digest: String,
    genome_digest: String,
    input_digest: String,
    event_digest: String,
    target_reference_digest: String,
    prediction: M3MicroPrediction,
    prediction_digest: String,
}

impl PersistedValidationPrediction {
    fn new(
        agent: &IndependentM3MicroAgent,
        input: &M3MicroValidationInput,
        prediction: M3MicroPrediction,
    ) -> Result<Self, M3MicroError> {
        let historical_input = &input.input;
        let mut value = Self {
            version: 2,
            agent_id: agent.agent_id(),
            partition: historical_input.partition,
            sequence_start: historical_input.sequence_start,
            sequence_end: historical_input.sequence_end,
            target_index: historical_input.target_index,
            horizon_bars: historical_input.horizon_bars,
            partition_start_index: historical_input.boundary.start_index,
            partition_end_index: historical_input.boundary.end_index,
            partition_boundary_digest: historical_input.boundary.boundary_digest.clone(),
            model_identity: agent.model.model_identity.clone(),
            schema_digest: agent.formula_genome.input_schema_digest.clone(),
            genome_digest: agent.formula_genome.genome_digest.clone(),
            input_digest: historical_input.input_digest.clone(),
            event_digest: historical_input.target_reference.event_digest.clone(),
            target_reference_digest: historical_input.target_reference.reference_digest.clone(),
            prediction,
            prediction_digest: String::new(),
        };
        value.prediction_digest = value.computed_digest()?;
        value.validate()?;
        Ok(value)
    }

    fn computed_digest(&self) -> Result<String, M3MicroError> {
        let mut canonical = self.clone();
        canonical.prediction_digest.clear();
        let bytes = serde_json::to_vec(&canonical).map_err(|_| M3MicroError::CorruptArtifact)?;
        Ok(stable_hash_string(&format!("{bytes:?}")))
    }

    fn validate(&self) -> Result<(), M3MicroError> {
        let expected_target = self
            .sequence_end
            .checked_add(self.horizon_bars)
            .ok_or(M3MicroError::InvalidChronology)?;
        if self.version != 2
            || self.partition != HistoricalPartition::Validation
            || self.sequence_start > self.sequence_end
            || self.horizon_bars == 0
            || self.target_index != expected_target
            || self.target_index > self.partition_end_index
            || self.model_identity.is_empty()
            || self.schema_digest.is_empty()
            || self.genome_digest.is_empty()
            || self.input_digest.is_empty()
            || self.event_digest.is_empty()
            || self.target_reference_digest.is_empty()
            || self.partition_boundary_digest.is_empty()
            || self.prediction.agent_id != self.agent_id
            || self.prediction_digest != self.computed_digest()?
        {
            return Err(M3MicroError::CorruptArtifact);
        }
        Ok(())
    }

    fn validate_bindings(
        &self,
        agent: &IndependentM3MicroAgent,
        input: &M3MicroValidationInput,
        boundary: &HistoricalPartitionBoundary,
    ) -> Result<(), M3MicroError> {
        self.validate()?;
        input.validate(agent, boundary)?;
        let historical_input = &input.input;
        if self.agent_id != agent.agent_id()
            || self.partition != historical_input.partition
            || self.sequence_start != historical_input.sequence_start
            || self.sequence_end != historical_input.sequence_end
            || self.target_index != historical_input.target_index
            || self.horizon_bars != historical_input.horizon_bars
            || self.partition_start_index != boundary.start_index
            || self.partition_end_index != boundary.end_index
            || self.partition_boundary_digest != boundary.boundary_digest
            || self.model_identity != agent.model.model_identity
            || self.schema_digest != agent.formula_genome.input_schema_digest
            || self.genome_digest != agent.formula_genome.genome_digest
            || self.input_digest != historical_input.input_digest
            || self.event_digest != historical_input.target_reference.event_digest
            || self.target_reference_digest != historical_input.target_reference.reference_digest
        {
            return Err(M3MicroError::CorruptArtifact);
        }
        Ok(())
    }

    fn persist(&self, artifact_root: &Path) -> Result<PathBuf, M3MicroError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| M3MicroError::CorruptArtifact)?;
        let path = artifact_root
            .join(format!("{:?}", self.agent_id))
            .join(format!("{}.json", self.prediction_digest));
        persist_artifact(&path, &bytes, &self.prediction_digest, |bytes| {
            let value: PersistedValidationPrediction = serde_json::from_slice(bytes)
                .map_err(|_| "validation prediction decode failed".to_string())?;
            value.validate().map_err(|error| error.to_string())?;
            Ok(value.prediction_digest)
        })
        .map_err(|_| M3MicroError::Io)?;
        Ok(path)
    }
}

#[derive(Clone, Debug)]
struct VerifiedPersistedPrediction {
    artifact: PersistedValidationPrediction,
    path: PathBuf,
}

impl VerifiedPersistedPrediction {
    fn reopen(
        path: PathBuf,
        expected_digest: &str,
        agent: &IndependentM3MicroAgent,
        input: &M3MicroValidationInput,
        boundary: &HistoricalPartitionBoundary,
    ) -> Result<Self, M3MicroError> {
        let bytes = fs::read(&path).map_err(|_| M3MicroError::Io)?;
        let artifact: PersistedValidationPrediction =
            serde_json::from_slice(&bytes).map_err(|_| M3MicroError::CorruptArtifact)?;
        artifact.validate_bindings(agent, input, boundary)?;
        if artifact.prediction_digest != expected_digest
            || path.file_stem().and_then(|value| value.to_str()) != Some(expected_digest)
        {
            return Err(M3MicroError::CorruptArtifact);
        }
        Ok(Self { artifact, path })
    }

    fn validate_target_reference(
        &self,
        target_reference: &M3MicroTargetReference,
    ) -> Result<(), M3MicroError> {
        if self.artifact.target_reference_digest != target_reference.reference_digest
            || self.artifact.event_digest != target_reference.event_digest
            || self.artifact.agent_id != target_reference.agent_id
            || self.artifact.partition != target_reference.partition
            || self.artifact.target_index != target_reference.target_index
            || self.path.as_os_str().is_empty()
        {
            return Err(M3MicroError::CorruptArtifact);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoricalIntegrationStatus {
    CompletedDevelopmentValidationOnly,
    ImplementationCompleteEvidencePending,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentBaselineComparison {
    pub m3_micro_mean_loss: f32,
    pub training_prevalence_constant_mean_loss: f32,
    pub agent_specific_mathematical_mean_loss: f32,
    pub legacy_historical_logistic_mean_loss: Option<f32>,
    pub legacy_result_executed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentHistoricalResult {
    pub agent_id: AgentId,
    pub development_example_count: usize,
    pub validation_example_count: usize,
    pub mean_development_loss: f32,
    pub mean_validation_loss: f32,
    pub parameter_count: usize,
    pub prediction_artifact_count: usize,
    pub normalizer_training_only: bool,
    pub prediction_before_reveal: bool,
    pub baseline_comparison: AgentBaselineComparison,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint103HistoricalReport {
    pub status: HistoricalIntegrationStatus,
    pub agent_results: Vec<AgentHistoricalResult>,
    pub safety_counters: Sprint103SafetyCounters,
    pub sealed_holdout_partition_present: bool,
    pub report_digest: String,
}

pub struct Sprint103HistoricalRunner;

fn probability_logit(probability: f32) -> f32 {
    let probability = probability.clamp(PROBABILITY_EPSILON, 1.0 - PROBABILITY_EPSILON);
    (probability / (1.0 - probability)).ln()
}

fn inverse_softplus(value: f32) -> f32 {
    let value = value.max(PROBABILITY_EPSILON);
    if value > 20.0 {
        value
    } else {
        value.exp_m1().max(PROBABILITY_EPSILON).ln()
    }
}

fn inverse_tanh(value: f32) -> f32 {
    let value = value.clamp(-0.999, 0.999);
    0.5 * ((1.0 + value) / (1.0 - value)).ln()
}

fn constant_baseline_raw(
    agent_id: AgentId,
    development: &[&M3MicroDevelopmentExample],
) -> Result<Vec<f32>, M3MicroError> {
    if development.is_empty() {
        return Err(M3MicroError::InvalidShape);
    }
    let count = development.len() as f32;
    let mut mean_distribution = [0.0; 3];
    let mut mean_scalar1 = 0.0;
    let mut mean_scalar2 = 0.0;
    for example in development {
        match agent_id {
            AgentId::TrendContinuation => {
                for (mean, target) in mean_distribution
                    .iter_mut()
                    .zip(example.target.direction_distribution.unwrap())
                {
                    *mean += target / count;
                }
                mean_scalar1 += example.target.continuation.unwrap() / count;
                mean_scalar2 += example.target.future_return.unwrap() / count;
            }
            AgentId::VolatilityRegime => {
                for (mean, target) in mean_distribution
                    .iter_mut()
                    .zip(example.target.volatility_regime.unwrap())
                {
                    *mean += target / count;
                }
                mean_scalar1 += example.target.future_variance.unwrap() / count;
                mean_scalar2 += example.target.risk_abstention.unwrap() / count;
            }
            AgentId::ReversalDistortion => {
                for (mean, target) in mean_distribution
                    .iter_mut()
                    .zip(example.target.direction_distribution.unwrap())
                {
                    *mean += target / count;
                }
                mean_scalar1 += example.target.reversal.unwrap() / count;
                mean_scalar2 += example.target.failed_breakout.unwrap() / count;
            }
        }
    }
    let distribution_logits = mean_distribution.map(|value| value.max(PROBABILITY_EPSILON).ln());
    let raw = match agent_id {
        AgentId::TrendContinuation => vec![
            distribution_logits[0],
            distribution_logits[1],
            distribution_logits[2],
            probability_logit(mean_scalar1),
            inverse_tanh(mean_scalar2),
        ],
        AgentId::VolatilityRegime => vec![
            inverse_softplus(mean_scalar1),
            distribution_logits[0],
            distribution_logits[1],
            distribution_logits[2],
            probability_logit(mean_scalar2),
        ],
        AgentId::ReversalDistortion => {
            let mean_return = development
                .iter()
                .map(|example| example.target.future_return.unwrap())
                .sum::<f32>()
                / count;
            vec![
                probability_logit(mean_scalar1),
                probability_logit(mean_scalar2),
                inverse_tanh(mean_return),
                distribution_logits[0],
                distribution_logits[1],
                distribution_logits[2],
            ]
        }
    };
    if raw.iter().any(|value| !value.is_finite()) {
        return Err(M3MicroError::NonFiniteOutput);
    }
    Ok(raw)
}

fn agent_mathematical_baseline_raw(
    agent_id: AgentId,
    normalized_sequence: &[Vec<f32>],
) -> Result<Vec<f32>, M3MicroError> {
    let last = normalized_sequence
        .last()
        .ok_or(M3MicroError::InvalidShape)?;
    if last.len() < 2 || last.iter().any(|value| !value.is_finite()) {
        return Err(M3MicroError::InvalidShape);
    }
    let first = last[0].clamp(-8.0, 8.0);
    let second = last[1].clamp(-8.0, 8.0);
    Ok(match agent_id {
        AgentId::TrendContinuation => vec![-first, -first.abs(), first, first.abs(), first],
        AgentId::VolatilityRegime => {
            let volatility = first.abs();
            vec![
                inverse_softplus(volatility + PROBABILITY_EPSILON),
                -volatility,
                1.0 - volatility,
                volatility,
                volatility + second.abs(),
            ]
        }
        AgentId::ReversalDistortion => vec![
            first.abs() + second,
            second.abs(),
            -first,
            first,
            -first.abs(),
            -first,
        ],
    })
}

impl Sprint103HistoricalRunner {
    pub fn pending_report() -> Sprint103HistoricalReport {
        let mut report = Sprint103HistoricalReport {
            status: HistoricalIntegrationStatus::ImplementationCompleteEvidencePending,
            agent_results: Vec::new(),
            safety_counters: Sprint103SafetyCounters::default(),
            sealed_holdout_partition_present: false,
            report_digest: String::new(),
        };
        report.report_digest = stable_hash_string(&format!(
            "{:?}:{:?}:{}",
            report.status, report.safety_counters, report.sealed_holdout_partition_present
        ));
        report
    }

    pub fn run(
        roster: &mut M3MicroRoster,
        dataset: &mut M3MicroHistoricalDataset,
        artifact_root: &Path,
    ) -> Result<Sprint103HistoricalReport, M3MicroError> {
        let counters = Sprint103SafetyCounters::default();
        counters.validate_zero()?;
        roster.validate()?;
        dataset.validate(roster)?;
        let validation_boundary = dataset
            .boundaries
            .get(&HistoricalPartition::Validation)
            .ok_or(M3MicroError::InvalidConfiguration)?
            .clone();
        let mut agent_results = Vec::with_capacity(3);
        for agent_id in AgentId::ORDERED {
            let development = dataset
                .development
                .get(&agent_id)
                .ok_or(M3MicroError::InvalidConfiguration)?
                .iter()
                .collect::<Vec<_>>();
            let validation = dataset
                .validation_inputs
                .get(&agent_id)
                .ok_or(M3MicroError::InvalidConfiguration)?
                .clone();
            let mut unique_rows = BTreeMap::<usize, Vec<f32>>::new();
            for example in &development {
                for (offset, row) in example.input.raw_sequence.iter().enumerate() {
                    let index = example
                        .input
                        .sequence_start
                        .checked_add(offset)
                        .ok_or(M3MicroError::InvalidChronology)?;
                    if unique_rows
                        .insert(index, row.clone())
                        .is_some_and(|prior| prior != *row)
                    {
                        return Err(M3MicroError::InvalidShape);
                    }
                }
            }
            let rows = unique_rows.into_iter().collect::<Vec<_>>();
            roster.fit_agent_normalizer(agent_id, HistoricalPartition::Development, &rows)?;
            let mut development_losses = Vec::with_capacity(development.len());
            for example in &development {
                let normalized = roster.agent(agent_id).normalize_sequence(
                    &example.input.input_schema_digest,
                    &example.input.raw_sequence,
                )?;
                development_losses.push(roster.train_agent_development_step(
                    agent_id,
                    HistoricalPartition::Development,
                    &normalized,
                    &example.target,
                )?);
            }
            let constant_raw = constant_baseline_raw(agent_id, &development)?;
            let mut validation_losses = Vec::with_capacity(validation.len());
            let mut constant_baseline_losses = Vec::with_capacity(validation.len());
            let mut mathematical_baseline_losses = Vec::with_capacity(validation.len());
            let mut artifact_count = 0usize;
            let history_start = roster.agent(agent_id).evaluation_history.len();
            for input in &validation {
                let mut stage_evidence = ValidationStageEvidence::default();
                input.validate(roster.agent(agent_id), &validation_boundary)?;
                stage_evidence.advance(ValidationStage::InputValidated)?;
                let (raw, prediction) = roster.predict_validation(input, &validation_boundary)?;
                stage_evidence.advance(ValidationStage::PredictionComputed)?;
                let persisted =
                    PersistedValidationPrediction::new(roster.agent(agent_id), input, prediction)?;
                let path = persisted.persist(artifact_root)?;
                stage_evidence.advance(ValidationStage::PredictionPersisted)?;
                let verified = VerifiedPersistedPrediction::reopen(
                    path,
                    &persisted.prediction_digest,
                    roster.agent(agent_id),
                    input,
                    &validation_boundary,
                )?;
                stage_evidence.advance(ValidationStage::PredictionReopenedAndVerified)?;
                artifact_count += 1;
                let target = dataset
                    .validation_targets
                    .reveal(&verified, &input.input.target_reference)?;
                stage_evidence.advance(ValidationStage::TargetRevealed)?;
                let (loss, _) = loss_and_output_gradient(agent_id, &raw, &target)?;
                validation_losses.push(loss);
                let (constant_loss, _) =
                    loss_and_output_gradient(agent_id, &constant_raw, &target)?;
                constant_baseline_losses.push(constant_loss);
                let normalized = roster.agent(agent_id).normalize_sequence(
                    &input.input.input_schema_digest,
                    &input.input.raw_sequence,
                )?;
                let mathematical_raw = agent_mathematical_baseline_raw(agent_id, &normalized)?;
                let (mathematical_loss, _) =
                    loss_and_output_gradient(agent_id, &mathematical_raw, &target)?;
                mathematical_baseline_losses.push(mathematical_loss);
                stage_evidence.advance(ValidationStage::Evaluated)?;
                roster.record_agent_evaluation(
                    agent_id,
                    EvaluationHistoryEntry {
                        partition: HistoricalPartition::Validation,
                        source_index: input.input.sequence_end,
                        loss,
                        prediction_digest: persisted.prediction_digest,
                        stage_evidence,
                    },
                )?;
            }
            let mean = |values: &[f32]| values.iter().sum::<f32>() / values.len().max(1) as f32;
            let agent = roster.agent(agent_id);
            let development_end = development
                .last()
                .map(|example| example.input.sequence_end)
                .ok_or(M3MicroError::InvalidChronology)?;
            agent_results.push(AgentHistoricalResult {
                agent_id,
                development_example_count: development.len(),
                validation_example_count: validation.len(),
                mean_development_loss: mean(&development_losses),
                mean_validation_loss: mean(&validation_losses),
                parameter_count: agent.parameter_count(),
                prediction_artifact_count: artifact_count,
                normalizer_training_only: agent.normalizer.fitted_on_end <= Some(development_end),
                prediction_before_reveal: agent
                    .evaluation_history
                    .get(history_start..)
                    .is_some_and(|entries| {
                        !entries.is_empty()
                            && entries
                                .iter()
                                .all(|entry| entry.stage_evidence.prediction_before_reveal())
                    }),
                baseline_comparison: AgentBaselineComparison {
                    m3_micro_mean_loss: mean(&validation_losses),
                    training_prevalence_constant_mean_loss: mean(&constant_baseline_losses),
                    agent_specific_mathematical_mean_loss: mean(&mathematical_baseline_losses),
                    legacy_historical_logistic_mean_loss: None,
                    legacy_result_executed: false,
                },
            });
        }
        counters.validate_zero()?;
        roster.validate()?;
        let mut report = Sprint103HistoricalReport {
            status: HistoricalIntegrationStatus::CompletedDevelopmentValidationOnly,
            agent_results,
            safety_counters: counters,
            sealed_holdout_partition_present: false,
            report_digest: String::new(),
        };
        report.report_digest = stable_hash_string(&format!(
            "{:?}:{:?}:{:?}:{}",
            report.status,
            report.agent_results,
            report.safety_counters,
            report.sealed_holdout_partition_present
        ));
        Ok(report)
    }
}

pub fn brier_score(probabilities: &[f32], targets: &[f32]) -> Result<f32, M3MicroError> {
    if probabilities.is_empty()
        || probabilities.len() != targets.len()
        || probabilities
            .iter()
            .chain(targets)
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err(M3MicroError::InvalidShape);
    }
    Ok(probabilities
        .iter()
        .zip(targets)
        .map(|(probability, target)| (probability - target).powi(2))
        .sum::<f32>()
        / probabilities.len() as f32)
}

pub fn ranked_probability_score(
    probabilities: &[[f32; 3]],
    targets: &[[f32; 3]],
) -> Result<f32, M3MicroError> {
    if probabilities.is_empty() || probabilities.len() != targets.len() {
        return Err(M3MicroError::InvalidShape);
    }
    let mut score = 0.0;
    for (probability, target) in probabilities.iter().zip(targets) {
        if probability
            .iter()
            .chain(target)
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            || (probability.iter().sum::<f32>() - 1.0).abs() > 1e-4
            || (target.iter().sum::<f32>() - 1.0).abs() > 1e-4
        {
            return Err(M3MicroError::InvalidShape);
        }
        let predicted_first = probability[0];
        let target_first = target[0];
        let predicted_second = probability[0] + probability[1];
        let target_second = target[0] + target[1];
        score += ((predicted_first - target_first).powi(2)
            + (predicted_second - target_second).powi(2))
            / 2.0;
    }
    Ok(score / probabilities.len() as f32)
}

pub fn qlike(predicted_variance: &[f32], observed_variance: &[f32]) -> Result<f32, M3MicroError> {
    if predicted_variance.is_empty()
        || predicted_variance.len() != observed_variance.len()
        || predicted_variance
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        || observed_variance
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(M3MicroError::InvalidShape);
    }
    Ok(predicted_variance
        .iter()
        .zip(observed_variance)
        .map(|(predicted, observed)| predicted.ln() + observed / predicted)
        .sum::<f32>()
        / predicted_variance.len() as f32)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum M3MicroBackendKind {
    DefaultCpu,
    MetalFeaturePortableReference,
}

impl M3MicroBackendKind {
    pub fn available(self) -> bool {
        match self {
            Self::DefaultCpu => true,
            Self::MetalFeaturePortableReference => {
                cfg!(all(target_os = "macos", feature = "backend-metal"))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentResourceMeasurement {
    pub agent_id: AgentId,
    pub parameter_count: usize,
    pub checkpoint_size_bytes: usize,
    pub recurrent_state_size_bytes: usize,
    pub single_agent_inference_nanoseconds: u128,
    pub estimated_one_agent_training_peak_bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterResourceMeasurement {
    pub environment: String,
    pub input_shape: [usize; 3],
    pub agents: Vec<AgentResourceMeasurement>,
    pub total_parameter_count: usize,
    pub three_agent_sequential_inference_nanoseconds: u128,
    pub three_agent_batched_execution_supported: bool,
    pub three_agent_batched_execution_nanoseconds: Option<u128>,
    pub estimated_three_agent_inference_peak_bytes: usize,
    pub formula_cache_size_bytes: usize,
}

pub fn measure_roster_resources(
    roster: &M3MicroRoster,
    raw_sequences: &BTreeMap<AgentId, Vec<Vec<f32>>>,
    formula_cache: &FormulaResultCache,
) -> Result<RosterResourceMeasurement, M3MicroError> {
    roster.validate()?;
    let mut agents = Vec::with_capacity(3);
    let mut sequential_inference_nanoseconds = 0u128;
    for agent_id in AgentId::ORDERED {
        let agent = roster.agent(agent_id);
        let sequence = raw_sequences
            .get(&agent_id)
            .ok_or(M3MicroError::InvalidShape)?;
        let mut inference_agent = agent.clone();
        let start = Instant::now();
        let _ = inference_agent.infer(&agent.formula_genome.input_schema_digest, sequence)?;
        let elapsed = start.elapsed().as_nanos();
        sequential_inference_nanoseconds += elapsed;
        let checkpoint_size_bytes = M3MicroCheckpoint::from_agent(agent)?.to_bytes()?.len();
        let parameter_bytes = agent.parameter_count() * std::mem::size_of::<f32>();
        let optimizer_bytes = parameter_bytes * 2;
        let gradient_bytes = parameter_bytes;
        let state_bytes = agent.recurrent_state.byte_size();
        agents.push(AgentResourceMeasurement {
            agent_id,
            parameter_count: agent.parameter_count(),
            checkpoint_size_bytes,
            recurrent_state_size_bytes: state_bytes,
            single_agent_inference_nanoseconds: elapsed,
            estimated_one_agent_training_peak_bytes: parameter_bytes
                + optimizer_bytes
                + gradient_bytes
                + state_bytes,
        });
    }
    let sequence_length = raw_sequences
        .values()
        .next()
        .map(Vec::len)
        .unwrap_or_default();
    Ok(RosterResourceMeasurement {
        environment: format!(
            "{}-{}-rust-f32",
            std::env::consts::OS,
            std::env::consts::ARCH
        ),
        input_shape: [3, sequence_length, 8],
        total_parameter_count: roster.total_parameter_count(),
        three_agent_sequential_inference_nanoseconds: sequential_inference_nanoseconds,
        three_agent_batched_execution_supported: false,
        three_agent_batched_execution_nanoseconds: None,
        estimated_three_agent_inference_peak_bytes: agents
            .iter()
            .map(|agent| {
                agent.parameter_count * std::mem::size_of::<f32>()
                    + agent.recurrent_state_size_bytes
            })
            .sum(),
        formula_cache_size_bytes: formula_cache.byte_size(),
        agents,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::{Candle, Timeframe};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_roster() -> M3MicroRoster {
        M3MicroRoster::new(103).unwrap()
    }

    fn row(width: usize, first: f32, second: f32) -> Vec<f32> {
        let mut value = vec![0.0; width];
        value[0] = first;
        value[1] = second;
        value
    }

    fn trend_target() -> M3MicroTarget {
        M3MicroTarget {
            agent_id: AgentId::TrendContinuation,
            direction_distribution: Some([0.0, 0.0, 1.0]),
            future_return: Some(0.5),
            continuation: Some(1.0),
            future_variance: None,
            volatility_regime: None,
            risk_abstention: None,
            reversal: None,
            failed_breakout: None,
        }
    }

    fn volatility_target() -> M3MicroTarget {
        M3MicroTarget {
            agent_id: AgentId::VolatilityRegime,
            direction_distribution: None,
            future_return: None,
            continuation: None,
            future_variance: Some(0.8),
            volatility_regime: Some([0.0, 0.0, 1.0]),
            risk_abstention: Some(1.0),
            reversal: None,
            failed_breakout: None,
        }
    }

    fn reversal_target() -> M3MicroTarget {
        M3MicroTarget {
            agent_id: AgentId::ReversalDistortion,
            direction_distribution: Some([1.0, 0.0, 0.0]),
            future_return: Some(-0.5),
            continuation: None,
            future_variance: None,
            volatility_regime: None,
            risk_abstention: None,
            reversal: Some(1.0),
            failed_breakout: Some(1.0),
        }
    }

    fn target(agent_id: AgentId) -> M3MicroTarget {
        match agent_id {
            AgentId::TrendContinuation => trend_target(),
            AgentId::VolatilityRegime => volatility_target(),
            AgentId::ReversalDistortion => reversal_target(),
        }
    }

    fn historical_input(
        roster: &M3MicroRoster,
        agent_id: AgentId,
        partition: HistoricalPartition,
        sequence_start: usize,
        sequence_end: usize,
        target_index: usize,
        horizon_bars: usize,
        boundary: &HistoricalPartitionBoundary,
        raw_sequence: Vec<Vec<f32>>,
        expected_target: &M3MicroTarget,
    ) -> M3MicroHistoricalInput {
        let agent = roster.agent(agent_id);
        let reference = M3MicroTargetReference::new(
            agent_id,
            partition,
            target_index,
            agent.spec.target_policy,
            format!("{agent_id:?}:{partition:?}:{target_index}"),
            Some(M3MicroTargetReference::commitment_for(expected_target).unwrap()),
        )
        .unwrap();
        M3MicroHistoricalInput::new(
            agent_id,
            partition,
            sequence_start,
            sequence_end,
            target_index,
            horizon_bars,
            boundary.clone(),
            agent.formula_genome.input_schema_digest.clone(),
            raw_sequence,
            reference,
        )
        .unwrap()
    }

    fn historical_dataset(roster: &M3MicroRoster) -> M3MicroHistoricalDataset {
        let development_boundary = HistoricalPartitionBoundary::new(
            HistoricalPartition::Development,
            0,
            2,
            "fixture-development-v2",
        )
        .unwrap();
        let validation_boundary = HistoricalPartitionBoundary::new(
            HistoricalPartition::Validation,
            4,
            6,
            "fixture-validation-v2",
        )
        .unwrap();
        let mut development = BTreeMap::new();
        let mut validation_inputs = BTreeMap::new();
        let mut target_registrations = Vec::new();
        for agent_id in AgentId::ORDERED {
            let width = roster.agent(agent_id).model.config.input_dim;
            let expected_target = target(agent_id);
            let development_input = historical_input(
                roster,
                agent_id,
                HistoricalPartition::Development,
                0,
                1,
                2,
                1,
                &development_boundary,
                vec![row(width, 0.1, 0.2), row(width, 0.2, 0.3)],
                &expected_target,
            );
            let validation_input = historical_input(
                roster,
                agent_id,
                HistoricalPartition::Validation,
                4,
                5,
                6,
                1,
                &validation_boundary,
                vec![row(width, 0.3, 0.4), row(width, 0.4, 0.5)],
                &expected_target,
            );
            target_registrations.push((
                validation_input.target_reference.clone(),
                expected_target.clone(),
            ));
            development.insert(
                agent_id,
                vec![M3MicroDevelopmentExample {
                    input: development_input,
                    target: expected_target,
                }],
            );
            validation_inputs.insert(
                agent_id,
                vec![M3MicroValidationInput {
                    input: validation_input,
                }],
            );
        }
        M3MicroHistoricalDataset {
            boundaries: BTreeMap::from([
                (HistoricalPartition::Development, development_boundary),
                (HistoricalPartition::Validation, validation_boundary),
            ]),
            development,
            validation_inputs,
            validation_targets: M3MicroValidationTargetSource::new(target_registrations).unwrap(),
        }
    }

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "soma-sprint103-{label}-{}-{nonce}",
            std::process::id()
        ))
    }

    fn candle_series(length: usize, constant: bool) -> CandleSeries {
        let candles = (0..length)
            .map(|index| {
                let drift = if constant {
                    0.0
                } else {
                    index as f64 * 0.15 + (index as f64 * 0.3).sin()
                };
                let close = 100.0 + drift;
                Candle {
                    timestamp_ms: index as u64 * 60_000,
                    open: close - 0.1,
                    high: close + 0.7,
                    low: close - 0.8,
                    close,
                    volume: if constant {
                        10.0
                    } else {
                        10.0 + (index % 7) as f64
                    },
                    trade_value: None,
                    bid: Some(close - 0.02),
                    ask: Some(close + 0.02),
                    spread_bps: Some(4.0),
                }
            })
            .collect();
        CandleSeries {
            symbol: "SYNTH".into(),
            timeframe: Timeframe::OneMinute,
            candles,
        }
    }

    #[test]
    fn sprint103_roster_budget_and_no_shared_trainable_storage() {
        let roster = test_roster();
        roster.validate().unwrap();
        assert_eq!(roster.agents().count(), 3);
        assert_eq!(
            roster.agent(AgentId::TrendContinuation).parameter_count(),
            141_573
        );
        assert_eq!(
            roster.agent(AgentId::VolatilityRegime).parameter_count(),
            141_573
        );
        assert_eq!(
            roster.agent(AgentId::ReversalDistortion).parameter_count(),
            141_638
        );
        assert_eq!(roster.total_parameter_count(), 424_784);
        assert!(
            roster
                .agents()
                .all(|agent| agent.parameter_count() < PARAMETER_LIMIT)
        );
        let parameter_storage = roster
            .agents()
            .map(|agent| agent.model.parameters.storage_identity())
            .collect::<BTreeSet<_>>();
        assert_eq!(parameter_storage.len(), 3);
    }

    #[test]
    fn sprint103_zero_constant_impulse_and_long_sequence_are_finite() {
        let agent = test_roster().agent(AgentId::TrendContinuation).clone();
        let width = agent.model.config.input_dim;
        let zero = vec![vec![0.0; width]; 16];
        let mut first_state = M3MicroState::zero(&agent.model.config).unwrap();
        let first = agent.model.forward(&zero, &mut first_state).unwrap();
        assert!(first.iter().all(|value| value.is_finite()));
        first_state.validate(&agent.model.config).unwrap();

        let constant = vec![vec![0.25; width]; 256];
        let mut constant_state = M3MicroState::zero(&agent.model.config).unwrap();
        let output = agent.model.forward(&constant, &mut constant_state).unwrap();
        assert!(output.iter().all(|value| value.is_finite()));
        assert!(
            constant_state
                .blocks
                .iter()
                .flat_map(|block| &block.values)
                .all(|value| value.abs() < STATE_LIMIT)
        );

        let impulse = [vec![1.0; width], vec![0.0; width], vec![0.0; width]];
        let mut impulse_state = M3MicroState::zero(&agent.model.config).unwrap();
        agent.model.forward(&impulse, &mut impulse_state).unwrap();
        assert!(
            impulse_state
                .blocks
                .iter()
                .flat_map(|block| &block.values)
                .any(|value| value.abs() > 1e-8)
        );
    }

    #[test]
    fn sprint103_two_point_memory_and_selective_forgetting() {
        let agent = test_roster().agent(AgentId::TrendContinuation).clone();
        let width = agent.model.config.input_dim;
        let current = row(width, 0.0, 1.0);
        let prior_signal = row(width, 1.0, 0.0);
        let mut with_prior = M3MicroState::zero(&agent.model.config).unwrap();
        let output_with_prior = agent
            .model
            .forward(&[prior_signal, current.clone()], &mut with_prior)
            .unwrap();
        let mut without_prior = M3MicroState::zero(&agent.model.config).unwrap();
        let output_without_prior = agent
            .model
            .forward(&[vec![0.0; width], current], &mut without_prior)
            .unwrap();
        assert_ne!(output_with_prior, output_without_prior);
        assert_ne!(with_prior.digest(), without_prior.digest());

        let layout = M3MicroLayout::new(&agent.model.config).unwrap();
        let mut retaining = agent.model.clone();
        let mut forgetting = agent.model.clone();
        for block in &layout.blocks {
            retaining.parameters.values[block.b_decay.clone()].fill(8.0);
            forgetting.parameters.values[block.b_decay.clone()].fill(-8.0);
        }
        retaining.refresh_identity();
        forgetting.refresh_identity();
        let mut retain_state = M3MicroState::zero(&agent.model.config).unwrap();
        let mut forget_state = M3MicroState::zero(&agent.model.config).unwrap();
        let mut impulse_then_zero = vec![row(width, 1.0, -1.0)];
        impulse_then_zero.extend(vec![vec![0.0; width]; 8]);
        retaining
            .forward(&impulse_then_zero, &mut retain_state)
            .unwrap();
        forgetting
            .forward(&impulse_then_zero, &mut forget_state)
            .unwrap();
        let norm = |state: &M3MicroState| {
            state
                .blocks
                .iter()
                .flat_map(|block| &block.values)
                .map(|value| value.abs())
                .sum::<f32>()
        };
        assert!(norm(&retain_state) > norm(&forget_state));
    }

    #[test]
    fn sprint103_determinism_serialization_and_reconstruction() {
        let roster = test_roster();
        let agent = roster.agent(AgentId::TrendContinuation);
        let sequence = vec![
            row(agent.model.config.input_dim, 0.5, -0.2),
            row(agent.model.config.input_dim, 0.1, 0.3),
        ];
        let mut left_state = M3MicroState::zero(&agent.model.config).unwrap();
        let mut right_state = M3MicroState::zero(&agent.model.config).unwrap();
        let left = agent.model.forward(&sequence, &mut left_state).unwrap();
        let right = agent.model.forward(&sequence, &mut right_state).unwrap();
        assert_eq!(left, right);
        assert_eq!(left_state, right_state);

        let bytes = serde_json::to_vec(&agent.model).unwrap();
        let restored: M3MicroModel = serde_json::from_slice(&bytes).unwrap();
        restored.validate().unwrap();
        let mut restored_state = M3MicroState::zero(&restored.config).unwrap();
        assert_eq!(
            left,
            restored.forward(&sequence, &mut restored_state).unwrap()
        );

        let checkpoint = M3MicroCheckpoint::from_agent(agent).unwrap();
        let decoded = M3MicroCheckpoint::from_bytes(&checkpoint.to_bytes().unwrap()).unwrap();
        let reconstructed = decoded.restore().unwrap();
        assert_eq!(agent.parameter_digest(), reconstructed.parameter_digest());
        assert_eq!(agent.optimizer_digest(), reconstructed.optimizer_digest());
        assert_eq!(agent.state_digest(), reconstructed.state_digest());
    }

    #[test]
    fn sprint103_wrong_shape_schema_nan_inf_gradient_and_state_explosion_reject() {
        let mut agent = test_roster().agent(AgentId::TrendContinuation).clone();
        let mut state = M3MicroState::zero(&agent.model.config).unwrap();
        assert_eq!(
            agent.model.forward(&[vec![0.0; 7]], &mut state),
            Err(M3MicroError::NonFiniteInput)
        );
        let mut nan_row = vec![0.0; agent.model.config.input_dim];
        nan_row[0] = f32::NAN;
        assert_eq!(
            agent.model.forward(&[nan_row], &mut state),
            Err(M3MicroError::NonFiniteInput)
        );
        let mut inf_row = vec![0.0; agent.model.config.input_dim];
        inf_row[0] = f32::INFINITY;
        assert_eq!(
            agent.model.forward(&[inf_row], &mut state),
            Err(M3MicroError::NonFiniteInput)
        );
        assert_eq!(
            agent.normalizer.transform("wrong-schema", &vec![0.0; 8]),
            Err(M3MicroError::WrongSchema)
        );
        let mut bad_gradient = vec![0.0; agent.parameter_count()];
        bad_gradient[0] = f32::NAN;
        assert_eq!(
            agent.optimizer_state.apply(&mut agent.model, &bad_gradient),
            Err(M3MicroError::NonFiniteGradient)
        );
        let mut exploded = M3MicroState::zero(&agent.model.config).unwrap();
        exploded.blocks[0].values[0] = STATE_LIMIT + 1.0;
        assert_eq!(
            agent.model.forward(&[vec![0.0; 8]], &mut exploded),
            Err(M3MicroError::StateExplosion)
        );
    }

    #[test]
    fn sprint103_parameter_state_and_optimizer_isolation() {
        let mut roster = test_roster();
        let roster_digest_before = roster.roster_digest.clone();
        let before_parameters =
            AgentId::ORDERED.map(|agent_id| roster.agent(agent_id).parameter_digest());
        let before_optimizers =
            AgentId::ORDERED.map(|agent_id| roster.agent(agent_id).optimizer_digest());
        let before_states = AgentId::ORDERED.map(|agent_id| roster.agent(agent_id).state_digest());
        let width = roster
            .agent(AgentId::TrendContinuation)
            .model
            .config
            .input_dim;
        roster
            .train_agent_development_step(
                AgentId::TrendContinuation,
                HistoricalPartition::Development,
                &[row(width, 1.0, 0.0), row(width, 0.0, 0.5)],
                &trend_target(),
            )
            .unwrap();
        assert_ne!(
            before_parameters[0],
            roster.agent(AgentId::TrendContinuation).parameter_digest()
        );
        assert_eq!(
            before_parameters[1],
            roster.agent(AgentId::VolatilityRegime).parameter_digest()
        );
        assert_eq!(
            before_parameters[2],
            roster.agent(AgentId::ReversalDistortion).parameter_digest()
        );
        assert_ne!(
            before_optimizers[0],
            roster.agent(AgentId::TrendContinuation).optimizer_digest()
        );
        assert_eq!(
            before_optimizers[1],
            roster.agent(AgentId::VolatilityRegime).optimizer_digest()
        );
        assert_eq!(
            before_optimizers[2],
            roster.agent(AgentId::ReversalDistortion).optimizer_digest()
        );
        assert_ne!(roster_digest_before, roster.roster_digest);
        roster.validate().unwrap();

        let schema = roster
            .agent(AgentId::TrendContinuation)
            .formula_genome
            .input_schema_digest
            .clone();
        roster
            .infer_agent(
                AgentId::TrendContinuation,
                &schema,
                &[row(width, 0.2, -0.1)],
            )
            .unwrap();
        assert_ne!(
            before_states[0],
            roster.agent(AgentId::TrendContinuation).state_digest()
        );
        assert_eq!(
            before_states[1],
            roster.agent(AgentId::VolatilityRegime).state_digest()
        );
        assert_eq!(
            before_states[2],
            roster.agent(AgentId::ReversalDistortion).state_digest()
        );
        roster.validate().unwrap();
    }

    #[test]
    fn sprint103_checkpoint_delete_corruption_restore_isolation() {
        let root = temp_root("checkpoint");
        let store = M3MicroCheckpointStore::new(&root).unwrap();
        let mut roster = test_roster();
        let mut digests = BTreeMap::new();
        for agent_id in AgentId::ORDERED {
            let saved = roster
                .materialize_agent_checkpoint(agent_id, &store)
                .unwrap();
            digests.insert(agent_id, saved.checkpoint_digest);
        }
        let agent2_before = roster.agent(AgentId::VolatilityRegime).parameter_digest();
        let agent3_before = roster.agent(AgentId::ReversalDistortion).parameter_digest();
        store
            .delete(
                AgentId::TrendContinuation,
                &digests[&AgentId::TrendContinuation],
            )
            .unwrap();
        assert_eq!(
            store
                .load(
                    AgentId::VolatilityRegime,
                    &digests[&AgentId::VolatilityRegime]
                )
                .unwrap()
                .parameter_digest(),
            agent2_before
        );
        assert_eq!(
            store
                .load(
                    AgentId::ReversalDistortion,
                    &digests[&AgentId::ReversalDistortion]
                )
                .unwrap()
                .parameter_digest(),
            agent3_before
        );

        let checkpoint =
            M3MicroCheckpoint::from_agent(roster.agent(AgentId::TrendContinuation)).unwrap();
        let mut corrupt = checkpoint.to_bytes().unwrap();
        corrupt[0] ^= 0xff;
        assert_eq!(
            M3MicroCheckpoint::from_bytes(&corrupt),
            Err(M3MicroError::CorruptArtifact)
        );
        let restored = checkpoint.restore().unwrap();
        assert_eq!(
            restored.parameter_digest(),
            roster.agent(AgentId::TrendContinuation).parameter_digest()
        );
        assert_eq!(
            roster.agent(AgentId::VolatilityRegime).parameter_digest(),
            agent2_before
        );
        assert_eq!(
            roster.agent(AgentId::ReversalDistortion).parameter_digest(),
            agent3_before
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sprint103_checkpoint_canonical_payload_is_idempotent_and_bound() {
        let root = temp_root("checkpoint-idempotence");
        let store = M3MicroCheckpointStore::new(&root).unwrap();
        let mut agent = test_roster().agent(AgentId::TrendContinuation).clone();
        let canonical_before = M3MicroCheckpoint::from_agent(&agent).unwrap();
        let first = store.save(&mut agent).unwrap();
        let first_bytes = fs::read(&first.path).unwrap();
        let second = store.save(&mut agent).unwrap();
        let second_bytes = fs::read(&second.path).unwrap();
        assert_eq!(first.checkpoint_digest, second.checkpoint_digest);
        assert_eq!(first.path, second.path);
        assert_eq!(first_bytes, second_bytes);

        let mut identity_only = agent.clone();
        identity_only.checkpoint_identity = "derived-identity-is-not-canonical".into();
        assert_eq!(
            canonical_before.checkpoint_digest,
            M3MicroCheckpoint::from_agent(&identity_only)
                .unwrap()
                .checkpoint_digest
        );

        let mut loaded = store
            .load(AgentId::TrendContinuation, &first.checkpoint_digest)
            .unwrap();
        let loaded_save = store.save(&mut loaded).unwrap();
        assert_eq!(first.checkpoint_digest, loaded_save.checkpoint_digest);
        assert_eq!(first_bytes, fs::read(&loaded_save.path).unwrap());

        let width = agent.model.config.input_dim;
        agent
            .train_development_step(
                HistoricalPartition::Development,
                &[row(width, 0.8, -0.2), row(width, 0.2, 0.4)],
                &trend_target(),
            )
            .unwrap();
        assert_ne!(
            first.checkpoint_digest,
            M3MicroCheckpoint::from_agent(&agent)
                .unwrap()
                .checkpoint_digest
        );

        let mut wrong_version = canonical_before.clone();
        wrong_version.format_version = 1;
        assert!(wrong_version.validate().is_err());
        let mut wrong_binding = canonical_before.clone();
        wrong_binding.agent_id = AgentId::VolatilityRegime;
        assert!(wrong_binding.validate().is_err());
        let mut wrong_digest = canonical_before.clone();
        wrong_digest.checkpoint_digest = "0".repeat(16);
        assert!(wrong_digest.validate().is_err());
        let mut wrong_payload = canonical_before;
        wrong_payload.payload.agent_id = AgentId::VolatilityRegime;
        assert!(wrong_payload.validate().is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sprint103_formula_registry_cache_causality_and_missing_source() {
        let registry = FormulaRegistry::sprint103();
        assert!(registry.specs().all(|spec| spec.causal_only));
        let series = candle_series(64, false);
        let truncated = CandleSeries {
            symbol: series.symbol.clone(),
            timeframe: series.timeframe,
            candles: series.candles[..41].to_vec(),
        };
        let spec = registry.get(FormulaId::LogReturn20).unwrap();
        let full = compute_formula(spec, &series, 40).unwrap();
        let prefix = compute_formula(spec, &truncated, 40).unwrap();
        assert_eq!(full, prefix);
        assert_eq!(
            compute_formula(spec, &series, 5),
            Err(M3MicroError::InsufficientHistory)
        );
        assert_eq!(
            compute_formula(
                registry.get(FormulaId::OrderFlowImbalance).unwrap(),
                &series,
                40
            ),
            Err(M3MicroError::UnavailableSourceEvidence)
        );
        let constant = candle_series(64, true);
        assert_eq!(
            compute_formula(
                registry.get(FormulaId::VolumeShock20).unwrap(),
                &constant,
                40
            )
            .unwrap(),
            vec![0.0]
        );

        let genome = AgentFormulaGenome::initial(AgentId::TrendContinuation, &registry).unwrap();
        let mut cache = FormulaResultCache::default();
        let first = build_causal_formula_row(&registry, &mut cache, &genome, &series, 40).unwrap();
        let digest = cache.digest();
        let second = build_causal_formula_row(&registry, &mut cache, &genome, &series, 40).unwrap();
        assert_eq!(first, second);
        assert_eq!(digest, cache.digest());
        assert!(first.iter().all(|value| value.is_finite()));

        let rows = (40..45)
            .map(|index| {
                (
                    index,
                    build_causal_formula_row(&registry, &mut cache, &genome, &series, index)
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();
        let normalizer =
            AgentNormalizer::fit_development(&genome.input_schema_digest, &rows).unwrap();
        let transformed = normalizer
            .transform(&genome.input_schema_digest, &rows[0].1)
            .unwrap();
        assert!(transformed.iter().all(|value| value.is_finite()));
        assert_eq!(
            normalizer.transform("different", &rows[0].1),
            Err(M3MicroError::WrongSchema)
        );
    }

    #[test]
    fn sprint103_formula_challenger_isolation_and_legacy_boundary() {
        let roster = test_roster();
        let registry = FormulaRegistry::sprint103();
        let champion = roster.agent(AgentId::TrendContinuation);
        let agent2_model = roster
            .agent(AgentId::VolatilityRegime)
            .model
            .model_identity
            .clone();
        let agent3_schema = roster
            .agent(AgentId::ReversalDistortion)
            .formula_genome
            .input_schema_digest
            .clone();
        let plan = ManualFormulaMutationPlan::new(
            champion,
            FormulaId::LogReturn1,
            FormulaId::TrendPersistence20,
        );
        let challenger = create_formula_challenger(champion, plan, &registry, 9_999).unwrap();
        assert_ne!(
            champion.formula_genome.input_schema_digest,
            challenger.challenger.formula_genome.input_schema_digest
        );
        assert_ne!(
            champion.model.model_identity,
            challenger.challenger.model.model_identity
        );
        assert_eq!(
            agent2_model,
            roster.agent(AgentId::VolatilityRegime).model.model_identity
        );
        assert_eq!(
            agent3_schema,
            roster
                .agent(AgentId::ReversalDistortion)
                .formula_genome
                .input_schema_digest
        );
        assert!(legacy_logistic_dispositions().iter().all(|item| {
            item.status == LegacyModelStatus::LegacyHistoricalBenchmarkOnly
                && !item.active_agent
                && !item.promotion_eligible
                && !item.prospective_candidate
                && item.historical_artifacts_preserved
        }));
    }

    #[test]
    fn sprint103_roster_promotion_and_failed_mutation_are_atomic() {
        let mut roster = test_roster();
        let registry = FormulaRegistry::sprint103();
        let selected = AgentId::TrendContinuation;
        let other_identities = [
            roster
                .agent(AgentId::VolatilityRegime)
                .model
                .model_identity
                .clone(),
            roster
                .agent(AgentId::ReversalDistortion)
                .model
                .model_identity
                .clone(),
        ];
        let plan = ManualFormulaMutationPlan::new(
            roster.agent(selected),
            FormulaId::LogReturn1,
            FormulaId::TrendPersistence20,
        );
        let mut challenger =
            create_formula_challenger(roster.agent(selected), plan, &registry, 11_003).unwrap();
        challenger.promotion_eligible = true;
        let evidence = ManualPromotionEvidence {
            challenger_identity: challenger.challenger_identity.clone(),
            development_complete: true,
            validation_complete: true,
            sealed_holdout_reads: 0,
            automatic: false,
            evidence_digest: stable_hash_string("manual-fixture-promotion-evidence"),
        };
        let roster_digest_before = roster.roster_digest.clone();
        let selected_genome_before = roster.agent(selected).formula_genome.genome_digest.clone();
        roster
            .promote_agent_challenger(selected, challenger, &evidence)
            .unwrap();
        roster.validate().unwrap();
        assert_ne!(roster_digest_before, roster.roster_digest);
        assert_ne!(
            selected_genome_before,
            roster.agent(selected).formula_genome.genome_digest
        );
        assert_eq!(
            other_identities[0],
            roster.agent(AgentId::VolatilityRegime).model.model_identity
        );
        assert_eq!(
            other_identities[1],
            roster
                .agent(AgentId::ReversalDistortion)
                .model
                .model_identity
        );

        let roster_before_failure = roster.clone();
        assert_eq!(
            roster.mutate_agent(selected, |agent| {
                agent.model.parameters.values[0] += 1.0;
                Err::<(), _>(M3MicroError::InvalidShape)
            }),
            Err(M3MicroError::InvalidShape)
        );
        assert_eq!(roster, roster_before_failure);
        roster.validate().unwrap();
    }

    #[test]
    fn sprint103_synthetic_learning_decreases_role_losses() {
        let mut roster = test_roster();
        for agent_id in AgentId::ORDERED {
            roster
                .mutate_agent(agent_id, |agent| {
                    agent.optimizer_state.config.learning_rate = 0.003;
                    Ok(())
                })
                .unwrap();
            let width = roster.agent(agent_id).model.config.input_dim;
            let sequence = match agent_id {
                AgentId::TrendContinuation => vec![
                    row(width, 1.0, 0.0),
                    row(width, 0.5, 0.1),
                    row(width, 0.25, 0.2),
                    row(width, 0.0, 0.3),
                ],
                AgentId::VolatilityRegime => vec![
                    row(width, 0.05, 0.0),
                    row(width, 0.2, 0.4),
                    row(width, 0.8, 1.0),
                    row(width, 1.0, 1.0),
                ],
                AgentId::ReversalDistortion => vec![
                    row(width, 1.0, 0.8),
                    row(width, 0.5, 0.4),
                    row(width, -0.5, -0.4),
                    row(width, -1.0, -0.8),
                ],
            };
            let expected = target(agent_id);
            let initial = model_loss_and_gradients(
                &roster.agent(agent_id).model,
                agent_id,
                &sequence,
                &expected,
            )
            .unwrap()
            .0;
            for _ in 0..5 {
                roster
                    .train_agent_development_step(
                        agent_id,
                        HistoricalPartition::Development,
                        &sequence,
                        &expected,
                    )
                    .unwrap();
            }
            let final_loss = model_loss_and_gradients(
                &roster.agent(agent_id).model,
                agent_id,
                &sequence,
                &expected,
            )
            .unwrap()
            .0;
            assert!(
                final_loss < initial,
                "{agent_id:?}: initial={initial}, final={final_loss}"
            );
        }
    }

    #[test]
    fn sprint103_synthetic_delayed_signal_recall_loss_decreases() {
        let mut roster = test_roster();
        roster
            .mutate_agent(AgentId::TrendContinuation, |agent| {
                agent.optimizer_state.config.learning_rate = 0.003;
                Ok(())
            })
            .unwrap();
        let width = roster
            .agent(AgentId::TrendContinuation)
            .model
            .config
            .input_dim;
        let sequence = vec![
            row(width, 1.0, -0.5),
            vec![0.0; width],
            vec![0.0; width],
            vec![0.0; width],
            vec![0.0; width],
        ];
        let expected = trend_target();
        let initial = model_loss_and_gradients(
            &roster.agent(AgentId::TrendContinuation).model,
            AgentId::TrendContinuation,
            &sequence,
            &expected,
        )
        .unwrap()
        .0;
        for _ in 0..4 {
            roster
                .train_agent_development_step(
                    AgentId::TrendContinuation,
                    HistoricalPartition::Development,
                    &sequence,
                    &expected,
                )
                .unwrap();
        }
        let final_loss = model_loss_and_gradients(
            &roster.agent(AgentId::TrendContinuation).model,
            AgentId::TrendContinuation,
            &sequence,
            &expected,
        )
        .unwrap()
        .0;
        assert!(final_loss < initial);
    }

    #[test]
    fn sprint103_historical_boundary_persists_before_reveal_and_never_fits_validation() {
        let root = temp_root("historical");
        let mut roster = test_roster();
        let mut dataset = historical_dataset(&roster);
        assert_eq!(dataset.target_access_count(), 0);
        let report = Sprint103HistoricalRunner::run(&mut roster, &mut dataset, &root).unwrap();
        assert_eq!(
            report.status,
            HistoricalIntegrationStatus::CompletedDevelopmentValidationOnly
        );
        assert!(!report.sealed_holdout_partition_present);
        report.safety_counters.validate_zero().unwrap();
        assert!(report.agent_results.iter().all(|result| {
            result.normalizer_training_only
                && result.prediction_before_reveal
                && result.prediction_artifact_count == 1
                && result.baseline_comparison.m3_micro_mean_loss.is_finite()
                && result
                    .baseline_comparison
                    .training_prevalence_constant_mean_loss
                    .is_finite()
                && result
                    .baseline_comparison
                    .agent_specific_mathematical_mean_loss
                    .is_finite()
                && !result.baseline_comparison.legacy_result_executed
                && result
                    .baseline_comparison
                    .legacy_historical_logistic_mean_loss
                    .is_none()
        }));
        assert_eq!(dataset.target_access_count(), 3);
        roster.validate().unwrap();
        assert_eq!(
            roster.fit_agent_normalizer(
                AgentId::TrendContinuation,
                HistoricalPartition::Validation,
                &[]
            ),
            Err(M3MicroError::ValidationFitForbidden)
        );
        assert_eq!(
            roster.train_agent_development_step(
                AgentId::TrendContinuation,
                HistoricalPartition::Validation,
                &[vec![0.0; 8]],
                &trend_target()
            ),
            Err(M3MicroError::ValidationFitForbidden)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sprint103_validation_capability_reveals_only_after_verified_reopen() {
        let root = temp_root("validation-capability");
        let roster = test_roster();
        let mut dataset = historical_dataset(&roster);
        let boundary = dataset
            .boundaries
            .get(&HistoricalPartition::Validation)
            .unwrap()
            .clone();
        let input = dataset.validation_inputs[&AgentId::TrendContinuation][0].clone();
        let agent_before = roster.agent(AgentId::TrendContinuation).clone();
        assert_eq!(dataset.target_access_count(), 0);

        let (raw, prediction) = roster.predict_validation(&input, &boundary).unwrap();
        assert!(raw.iter().all(|value| value.is_finite()));
        let agent_after_prediction = roster.agent(AgentId::TrendContinuation);
        assert_eq!(
            agent_before.parameter_digest(),
            agent_after_prediction.parameter_digest()
        );
        assert_eq!(
            agent_before.optimizer_digest(),
            agent_after_prediction.optimizer_digest()
        );
        assert_eq!(
            agent_before.state_digest(),
            agent_after_prediction.state_digest()
        );
        assert_eq!(
            agent_before.normalizer.normalizer_digest,
            agent_after_prediction.normalizer.normalizer_digest
        );
        assert_eq!(
            agent_before.training_history,
            agent_after_prediction.training_history
        );

        let persisted =
            PersistedValidationPrediction::new(agent_after_prediction, &input, prediction).unwrap();
        let artifact_value = serde_json::to_value(&persisted).unwrap();
        let artifact_fields = artifact_value.as_object().unwrap();
        assert!(!artifact_fields.contains_key("target"));
        assert!(!artifact_fields.contains_key("target_value"));
        assert!(!artifact_fields.contains_key("target_commitment"));
        assert!(!artifact_fields.contains_key("loss"));
        assert!(!artifact_fields.contains_key("correctness"));

        let path = persisted.persist(&root).unwrap();
        assert_eq!(dataset.target_access_count(), 0);
        let verified = VerifiedPersistedPrediction::reopen(
            path,
            &persisted.prediction_digest,
            roster.agent(AgentId::TrendContinuation),
            &input,
            &boundary,
        )
        .unwrap();
        assert_eq!(dataset.target_access_count(), 0);
        let revealed = dataset
            .validation_targets
            .reveal(&verified, &input.input.target_reference)
            .unwrap();
        assert_eq!(dataset.target_access_count(), 1);
        assert_eq!(revealed, trend_target());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sprint103_persist_and_reopen_failures_do_not_reveal_or_evaluate() {
        let persistence_root = temp_root("persistence-failure");
        fs::write(&persistence_root, b"not-a-directory").unwrap();
        let mut persistence_roster = test_roster();
        let mut persistence_dataset = historical_dataset(&persistence_roster);
        assert!(
            Sprint103HistoricalRunner::run(
                &mut persistence_roster,
                &mut persistence_dataset,
                &persistence_root
            )
            .is_err()
        );
        assert_eq!(persistence_dataset.target_access_count(), 0);
        assert!(
            persistence_roster
                .agents()
                .all(|agent| agent.evaluation_history.is_empty())
        );
        fs::remove_file(persistence_root).unwrap();

        let corruption_root = temp_root("reopen-corruption");
        let corruption_roster = test_roster();
        let corruption_dataset = historical_dataset(&corruption_roster);
        let boundary = corruption_dataset
            .boundaries
            .get(&HistoricalPartition::Validation)
            .unwrap()
            .clone();
        let input = corruption_dataset.validation_inputs[&AgentId::TrendContinuation][0].clone();
        let (_, prediction) = corruption_roster
            .predict_validation(&input, &boundary)
            .unwrap();
        let persisted = PersistedValidationPrediction::new(
            corruption_roster.agent(AgentId::TrendContinuation),
            &input,
            prediction,
        )
        .unwrap();
        let path = persisted.persist(&corruption_root).unwrap();
        fs::write(&path, b"corrupt-after-persist").unwrap();
        assert!(
            VerifiedPersistedPrediction::reopen(
                path,
                &persisted.prediction_digest,
                corruption_roster.agent(AgentId::TrendContinuation),
                &input,
                &boundary,
            )
            .is_err()
        );
        assert_eq!(corruption_dataset.target_access_count(), 0);
        assert!(
            corruption_roster
                .agents()
                .all(|agent| agent.evaluation_history.is_empty())
        );
        fs::remove_dir_all(corruption_root).unwrap();
    }

    #[test]
    fn sprint103_independent_partition_boundary_rejects_all_chronology_spoofs() {
        let roster = test_roster();
        let dataset = historical_dataset(&roster);
        dataset.validate(&roster).unwrap();
        let agent = roster.agent(AgentId::TrendContinuation);
        let boundary = dataset
            .boundaries
            .get(&HistoricalPartition::Validation)
            .unwrap();
        let exact = dataset.validation_inputs[&AgentId::TrendContinuation][0]
            .input
            .clone();
        exact.validate(agent, boundary).unwrap();
        assert_eq!(exact.target_index, boundary.end_index);

        let crossing_target = trend_target();
        let crossing = historical_input(
            &roster,
            AgentId::TrendContinuation,
            HistoricalPartition::Validation,
            4,
            5,
            7,
            2,
            boundary,
            vec![row(agent.model.config.input_dim, 0.1, 0.2); 2],
            &crossing_target,
        );
        assert_eq!(
            crossing.validate(agent, boundary),
            Err(M3MicroError::InvalidChronology)
        );

        let mut inconsistent = exact.clone();
        inconsistent.horizon_bars = 2;
        inconsistent.refresh_digest().unwrap();
        assert_eq!(
            inconsistent.validate(agent, boundary),
            Err(M3MicroError::InvalidChronology)
        );
        let mut zero_horizon = exact.clone();
        zero_horizon.horizon_bars = 0;
        zero_horizon.refresh_digest().unwrap();
        assert_eq!(
            zero_horizon.validate(agent, boundary),
            Err(M3MicroError::InvalidChronology)
        );

        let overflow_boundary = HistoricalPartitionBoundary::new(
            HistoricalPartition::Validation,
            usize::MAX - 1,
            usize::MAX,
            "overflow-boundary-v2",
        )
        .unwrap();
        let overflow = historical_input(
            &roster,
            AgentId::TrendContinuation,
            HistoricalPartition::Validation,
            usize::MAX - 1,
            usize::MAX - 1,
            usize::MAX,
            2,
            &overflow_boundary,
            vec![row(agent.model.config.input_dim, 0.1, 0.2)],
            &crossing_target,
        );
        assert_eq!(
            overflow.validate(agent, &overflow_boundary),
            Err(M3MicroError::InvalidChronology)
        );

        let mut spoofed = exact;
        spoofed.boundary.end_index += 1;
        spoofed.boundary.refresh_digest().unwrap();
        spoofed.refresh_digest().unwrap();
        assert_eq!(
            spoofed.validate(agent, boundary),
            Err(M3MicroError::InvalidChronology)
        );

        let mut overlap = historical_dataset(&roster);
        let development_boundary = overlap
            .boundaries
            .get_mut(&HistoricalPartition::Development)
            .unwrap();
        development_boundary.end_index = boundary.start_index;
        development_boundary.refresh_digest().unwrap();
        assert_eq!(
            overlap.validate(&roster),
            Err(M3MicroError::InvalidChronology)
        );
    }

    #[test]
    fn sprint103_metrics_safety_and_resource_contracts() {
        assert_eq!(brier_score(&[0.0, 1.0], &[0.0, 1.0]).unwrap(), 0.0);
        assert_eq!(
            ranked_probability_score(&[[1.0, 0.0, 0.0]], &[[1.0, 0.0, 0.0]]).unwrap(),
            0.0
        );
        assert!(qlike(&[0.5], &[0.5]).unwrap().is_finite());
        Sprint103SafetyCounters::default().validate_zero().unwrap();
        assert!(M3MicroBackendKind::DefaultCpu.available());

        let roster = test_roster();
        let sequences = AgentId::ORDERED
            .into_iter()
            .map(|agent_id| {
                (
                    agent_id,
                    vec![vec![0.0; roster.agent(agent_id).model.config.input_dim]; 4],
                )
            })
            .collect::<BTreeMap<_, _>>();
        let registry = FormulaRegistry::sprint103();
        let series = candle_series(64, false);
        let mut formula_cache = FormulaResultCache::default();
        for agent_id in AgentId::ORDERED {
            build_causal_formula_row(
                &registry,
                &mut formula_cache,
                &roster.agent(agent_id).formula_genome,
                &series,
                40,
            )
            .unwrap();
        }
        let measurement = measure_roster_resources(&roster, &sequences, &formula_cache).unwrap();
        println!(
            "SPRINT103_MEASUREMENT={}",
            serde_json::to_string(&measurement).unwrap()
        );
        assert_eq!(measurement.total_parameter_count, 424_784);
        assert_eq!(measurement.agents.len(), 3);
        assert!(!measurement.three_agent_batched_execution_supported);
        assert!(measurement.formula_cache_size_bytes > 0);
        assert!(
            measurement.agents.iter().all(
                |agent| agent.checkpoint_size_bytes > 0 && agent.recurrent_state_size_bytes > 0
            )
        );
    }

    #[cfg(all(target_os = "macos", feature = "backend-metal"))]
    #[test]
    fn sprint103_metal_feature_has_same_architecture_and_finite_reference_output() {
        assert!(M3MicroBackendKind::MetalFeaturePortableReference.available());
        let roster = test_roster();
        for agent in roster.agents() {
            assert_eq!(
                agent.parameter_count(),
                M3MicroLayout::new(&agent.model.config)
                    .unwrap()
                    .parameter_count
            );
            let sequence = vec![vec![0.0; agent.model.config.input_dim]; 2];
            let mut default_state = M3MicroState::zero(&agent.model.config).unwrap();
            let mut metal_feature_state = M3MicroState::zero(&agent.model.config).unwrap();
            let default = agent.model.forward(&sequence, &mut default_state).unwrap();
            let metal_feature = agent
                .model
                .forward(&sequence, &mut metal_feature_state)
                .unwrap();
            assert_eq!(default, metal_feature);
            assert_eq!(default_state, metal_feature_state);
            assert!(default.iter().all(|value| value.is_finite()));
        }
    }
}
