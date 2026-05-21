use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::backtest::{
    CostModel, NoTradeScoreConfig, Timeframe, TripleBarrierConfig, TripleBarrierResult,
    evaluate_no_trade_counterfactual, evaluate_triple_barrier,
};
use crate::core::{ReasonCode, Side, stable_reason_codes};
use crate::{Candle, CandleSeries};

use super::committee_outcome_linker::OutcomeLinkedCommitteeScenarioRow;
use super::committee_outcome_reference::CommitteeTripleBarrierLabel;
use super::committee_scenario_loader::{CommitteeScenarioRow, CommitteeScenarioSourceKind};
use super::persona_card_lite::PersonaHorizon;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommitteeCounterfactualType {
    #[default]
    NoTrade,
    RiskDenied,
    BaselineAction,
    ExternalAction,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CounterfactualBuildStatus {
    Built,
    #[default]
    UnavailableNoCandleData,
    UnavailableNoTimestampMatch,
    UnavailableWrongHorizon,
    EstimatedDiagnosticOnly,
    RejectedNoLookahead,
    RejectedBadDataQuality,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeCounterfactualBuildConfig {
    #[serde(default = "default_horizon_bars")]
    pub default_horizon_bars: usize,
    #[serde(default = "default_take_profit_pct")]
    pub default_take_profit_pct: f64,
    #[serde(default = "default_stop_loss_pct")]
    pub default_stop_loss_pct: f64,
    #[serde(default)]
    pub cost_model: Option<CostModel>,
    #[serde(default)]
    pub allow_estimated_when_missing_candles: bool,
    #[serde(default = "default_true")]
    pub build_no_trade_counterfactuals: bool,
    #[serde(default = "default_true")]
    pub build_risk_denied_counterfactuals: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeCounterfactualRecord {
    pub counterfactual_id: String,
    pub scenario_row_id: String,
    pub counterfactual_type: CommitteeCounterfactualType,
    pub build_status: CounterfactualBuildStatus,
    #[serde(default)]
    pub triple_barrier_label: Option<CommitteeTripleBarrierLabel>,
    #[serde(default)]
    pub net_return_pct: Option<f64>,
    #[serde(default)]
    pub avoided_loss_value: Option<f64>,
    #[serde(default)]
    pub missed_gain_value: Option<f64>,
    #[serde(default)]
    pub max_favorable_excursion_pct: Option<f64>,
    #[serde(default)]
    pub max_adverse_excursion_pct: Option<f64>,
    pub cost_bps: f64,
    pub slippage_bps: f64,
    pub no_lookahead_safe: bool,
    pub diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitteeCounterfactualBuilder;

impl Default for CommitteeCounterfactualBuildConfig {
    fn default() -> Self {
        Self {
            default_horizon_bars: default_horizon_bars(),
            default_take_profit_pct: default_take_profit_pct(),
            default_stop_loss_pct: default_stop_loss_pct(),
            cost_model: None,
            allow_estimated_when_missing_candles: false,
            build_no_trade_counterfactuals: true,
            build_risk_denied_counterfactuals: true,
            reason_codes: vec![ReasonCode::CommitteeCounterfactualBuilderBuilt],
        }
    }
}

impl CommitteeCounterfactualBuilder {
    pub fn build_records(
        &self,
        row: &OutcomeLinkedCommitteeScenarioRow,
        candle_series: Option<&CandleSeries>,
        config: &CommitteeCounterfactualBuildConfig,
    ) -> Vec<CommitteeCounterfactualRecord> {
        let mut records = Vec::new();
        if config.build_no_trade_counterfactuals {
            records.push(self.build_record(
                row,
                candle_series,
                config,
                CommitteeCounterfactualType::NoTrade,
            ));
        }
        if config.build_risk_denied_counterfactuals {
            records.push(self.build_record(
                row,
                candle_series,
                config,
                CommitteeCounterfactualType::RiskDenied,
            ));
        }
        records.sort_by(|left, right| left.counterfactual_id.cmp(&right.counterfactual_id));
        records
    }

    fn build_record(
        &self,
        row: &OutcomeLinkedCommitteeScenarioRow,
        candle_series: Option<&CandleSeries>,
        config: &CommitteeCounterfactualBuildConfig,
        counterfactual_type: CommitteeCounterfactualType,
    ) -> CommitteeCounterfactualRecord {
        let base_cost_model = config
            .cost_model
            .unwrap_or_else(|| cost_model_for_row(row, candle_series));
        let horizon_bars = row
            .outcome_reference
            .as_ref()
            .map(|reference| reference.horizon_bars)
            .unwrap_or_else(|| {
                horizon_bars_for_row(&row.scenario_row, config.default_horizon_bars)
            });
        let counterfactual_id = format!(
            "{}-{:?}",
            row.scenario_row.scenario_row_id, counterfactual_type
        );

        if row
            .outcome_reference
            .as_ref()
            .is_some_and(|reference| !reference.no_lookahead_safe)
        {
            return unavailable_record(
                counterfactual_id,
                row,
                counterfactual_type,
                CounterfactualBuildStatus::RejectedNoLookahead,
                base_cost_model,
                false,
                vec![
                    ReasonCode::CommitteeCounterfactualBuilderBuilt,
                    ReasonCode::RejectedNoLookaheadReference,
                ],
            );
        }

        let Some(series) = candle_series else {
            return unavailable_record(
                counterfactual_id,
                row,
                counterfactual_type,
                CounterfactualBuildStatus::UnavailableNoCandleData,
                base_cost_model,
                false,
                vec![
                    ReasonCode::CommitteeCounterfactualBuilderBuilt,
                    ReasonCode::CommitteeCounterfactualUnavailable,
                    ReasonCode::MissingRealLocalData,
                ],
            );
        };

        let resolution = match resolve_entry_index(
            series,
            row.scenario_row.timestamp_ms,
            config.allow_estimated_when_missing_candles,
        ) {
            Some(resolution) => resolution,
            None => {
                return unavailable_record(
                    counterfactual_id,
                    row,
                    counterfactual_type,
                    CounterfactualBuildStatus::UnavailableNoTimestampMatch,
                    base_cost_model,
                    false,
                    vec![
                        ReasonCode::CommitteeCounterfactualBuilderBuilt,
                        ReasonCode::CommitteeCounterfactualUnavailable,
                        ReasonCode::StaleTimestamp,
                    ],
                );
            }
        };

        if resolution.entry_index + horizon_bars >= series.len() || horizon_bars == 0 {
            return unavailable_record(
                counterfactual_id,
                row,
                counterfactual_type,
                CounterfactualBuildStatus::UnavailableWrongHorizon,
                base_cost_model,
                false,
                vec![
                    ReasonCode::CommitteeCounterfactualBuilderBuilt,
                    ReasonCode::CommitteeCounterfactualUnavailable,
                    ReasonCode::InsufficientBars,
                ],
            );
        }

        if !candle_window_is_usable(series, resolution.entry_index, horizon_bars) {
            return unavailable_record(
                counterfactual_id,
                row,
                counterfactual_type,
                CounterfactualBuildStatus::RejectedBadDataQuality,
                base_cost_model,
                false,
                vec![
                    ReasonCode::CommitteeCounterfactualBuilderBuilt,
                    ReasonCode::CommitteeCounterfactualRejected,
                    ReasonCode::DataQualityTooLow,
                ],
            );
        }

        let entry_price = series
            .candle(resolution.entry_index)
            .map(|candle| candle.close)
            .unwrap_or_default();
        let result = evaluate_triple_barrier(
            series,
            resolution.entry_index,
            entry_price,
            TripleBarrierConfig {
                take_profit_pct: config.default_take_profit_pct,
                stop_loss_pct: config.default_stop_loss_pct,
                horizon_bars,
                fee_bps: base_cost_model.fee_bps,
                slippage_bps: base_cost_model.slippage_bps,
                side: Side::Long,
                use_high_low_intrabar: true,
            },
        );
        let evaluation = evaluate_no_trade_counterfactual(
            Some(&result),
            counterfactual_type == CommitteeCounterfactualType::RiskDenied,
            NoTradeScoreConfig::default(),
        );
        let diagnostic_only = resolution.estimated;
        let build_status = if diagnostic_only {
            CounterfactualBuildStatus::EstimatedDiagnosticOnly
        } else {
            CounterfactualBuildStatus::Built
        };
        let mut reason_codes = config.reason_codes.clone();
        reason_codes.extend([
            ReasonCode::CommitteeCounterfactualBuilderBuilt,
            ReasonCode::CommitteeCounterfactualBuilt,
            ReasonCode::CounterfactualEvaluated,
        ]);
        if diagnostic_only {
            reason_codes.push(ReasonCode::CommitteeCounterfactualEstimatedDiagnostic);
        }
        reason_codes.extend(result.reason_codes.clone());
        reason_codes.extend(evaluation.reason_codes);
        CommitteeCounterfactualRecord {
            counterfactual_id,
            scenario_row_id: row.scenario_row.scenario_row_id.clone(),
            counterfactual_type,
            build_status,
            triple_barrier_label: Some(label_from_result(&result, counterfactual_type)),
            net_return_pct: Some(result.net_return_pct),
            avoided_loss_value: (evaluation.avoided_loss_score > 0.0)
                .then_some(evaluation.avoided_loss_score),
            missed_gain_value: (evaluation.missed_gain_penalty < 0.0)
                .then_some(evaluation.missed_gain_penalty.abs()),
            max_favorable_excursion_pct: Some(result.max_favorable_excursion_pct),
            max_adverse_excursion_pct: Some(result.max_adverse_excursion_pct),
            cost_bps: base_cost_model.fee_bps,
            slippage_bps: base_cost_model.slippage_bps,
            no_lookahead_safe: !diagnostic_only,
            diagnostic_only,
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }
}

impl CommitteeCounterfactualRecord {
    pub fn built(&self) -> bool {
        matches!(
            self.build_status,
            CounterfactualBuildStatus::Built | CounterfactualBuildStatus::EstimatedDiagnosticOnly
        )
    }

    pub fn unavailable(&self) -> bool {
        matches!(
            self.build_status,
            CounterfactualBuildStatus::UnavailableNoCandleData
                | CounterfactualBuildStatus::UnavailableNoTimestampMatch
                | CounterfactualBuildStatus::UnavailableWrongHorizon
        )
    }

    pub fn estimated(&self) -> bool {
        self.build_status == CounterfactualBuildStatus::EstimatedDiagnosticOnly
    }

    pub fn to_text_line(&self) -> String {
        format!(
            "counterfactual_id={};scenario_row_id={};type={:?};status={:?};net_return_pct={};avoided_loss_value={};missed_gain_value={};diagnostic_only={};no_lookahead_safe={}",
            self.counterfactual_id,
            self.scenario_row_id,
            self.counterfactual_type,
            self.build_status,
            self.net_return_pct
                .map(crate::core::deterministic_float_format)
                .unwrap_or_default(),
            self.avoided_loss_value
                .map(crate::core::deterministic_float_format)
                .unwrap_or_default(),
            self.missed_gain_value
                .map(crate::core::deterministic_float_format)
                .unwrap_or_default(),
            self.diagnostic_only,
            self.no_lookahead_safe
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EntryIndexResolution {
    entry_index: usize,
    estimated: bool,
}

fn resolve_entry_index(
    series: &CandleSeries,
    timestamp_ms: u64,
    allow_estimated: bool,
) -> Option<EntryIndexResolution> {
    if let Some(entry_index) = series
        .candles
        .iter()
        .position(|candle| candle.timestamp_ms == timestamp_ms)
    {
        return Some(EntryIndexResolution {
            entry_index,
            estimated: false,
        });
    }
    if !allow_estimated || series.candles.is_empty() {
        return None;
    }
    series
        .candles
        .iter()
        .enumerate()
        .min_by_key(|(_, candle)| timestamp_distance(candle.timestamp_ms, timestamp_ms))
        .map(|(entry_index, _)| EntryIndexResolution {
            entry_index,
            estimated: true,
        })
}

fn candle_window_is_usable(series: &CandleSeries, entry_index: usize, horizon_bars: usize) -> bool {
    series.candles[entry_index..=entry_index + horizon_bars]
        .iter()
        .all(candle_is_usable)
}

fn candle_is_usable(candle: &Candle) -> bool {
    candle.open.is_finite()
        && candle.high.is_finite()
        && candle.low.is_finite()
        && candle.close.is_finite()
        && candle.open > 0.0
        && candle.high > 0.0
        && candle.low > 0.0
        && candle.close > 0.0
        && candle.high >= candle.low
        && candle.high >= candle.open.min(candle.close)
        && candle.low <= candle.open.max(candle.close)
}

fn unavailable_record(
    counterfactual_id: String,
    row: &OutcomeLinkedCommitteeScenarioRow,
    counterfactual_type: CommitteeCounterfactualType,
    build_status: CounterfactualBuildStatus,
    cost_model: CostModel,
    no_lookahead_safe: bool,
    mut reason_codes: Vec<ReasonCode>,
) -> CommitteeCounterfactualRecord {
    reason_codes.extend(row.reason_codes.clone());
    CommitteeCounterfactualRecord {
        counterfactual_id,
        scenario_row_id: row.scenario_row.scenario_row_id.clone(),
        counterfactual_type,
        build_status,
        triple_barrier_label: None,
        net_return_pct: None,
        avoided_loss_value: None,
        missed_gain_value: None,
        max_favorable_excursion_pct: None,
        max_adverse_excursion_pct: None,
        cost_bps: cost_model.fee_bps,
        slippage_bps: cost_model.slippage_bps,
        no_lookahead_safe,
        diagnostic_only: build_status == CounterfactualBuildStatus::EstimatedDiagnosticOnly,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn label_from_result(
    result: &TripleBarrierResult,
    counterfactual_type: CommitteeCounterfactualType,
) -> CommitteeTripleBarrierLabel {
    match counterfactual_type {
        CommitteeCounterfactualType::NoTrade => CommitteeTripleBarrierLabel::NoTradeCounterfactual,
        CommitteeCounterfactualType::RiskDenied => {
            CommitteeTripleBarrierLabel::RiskDeniedCounterfactual
        }
        CommitteeCounterfactualType::BaselineAction
        | CommitteeCounterfactualType::ExternalAction => match result.first_hit {
            crate::backtest::BarrierHit::TakeProfit => CommitteeTripleBarrierLabel::TakeProfit,
            crate::backtest::BarrierHit::StopLoss => CommitteeTripleBarrierLabel::StopLoss,
            crate::backtest::BarrierHit::TimeExpired => CommitteeTripleBarrierLabel::TimeExpired,
            crate::backtest::BarrierHit::NoData => CommitteeTripleBarrierLabel::Unknown,
        },
    }
}

fn timestamp_distance(left: u64, right: u64) -> u64 {
    left.max(right) - left.min(right)
}

pub fn horizon_bars_for_row(row: &CommitteeScenarioRow, default_horizon_bars: usize) -> usize {
    row.outcome_reference
        .as_ref()
        .map(|_| default_horizon_bars)
        .unwrap_or_else(|| match row.target_horizon {
            PersonaHorizon::Intraday => 6,
            PersonaHorizon::Swing => 24,
            PersonaHorizon::MultiDay => 48,
            PersonaHorizon::LongTerm => 96,
        })
}

fn cost_model_for_row(
    row: &OutcomeLinkedCommitteeScenarioRow,
    candle_series: Option<&CandleSeries>,
) -> CostModel {
    let spread_bps = candle_series
        .and_then(|series| series.candles.first())
        .and_then(|candle| candle.spread_bps)
        .or(row.scenario_row.spread_bps);
    CostModel {
        fee_bps: row
            .outcome_reference
            .as_ref()
            .map(|reference| reference.cost_bps)
            .unwrap_or(5.0),
        slippage_bps: row
            .outcome_reference
            .as_ref()
            .map(|reference| reference.slippage_bps)
            .unwrap_or(2.0),
        spread_bps,
        min_cost_bps: None,
    }
}

pub fn load_local_candle_series_map(
    paths: &[String],
) -> Result<BTreeMap<String, CandleSeries>, String> {
    let mut series_by_symbol = BTreeMap::new();
    for path in paths {
        let path_ref = Path::new(path);
        let text = fs::read_to_string(path_ref).map_err(|err| err.to_string())?;
        let mut loaded = parse_candle_series_text(&text)?;
        loaded.sort_by(|left, right| left.symbol.cmp(&right.symbol));
        for series in loaded {
            series_by_symbol.insert(normalize_symbol(&series.symbol), series);
        }
    }
    Ok(series_by_symbol)
}

fn parse_candle_series_text(input: &str) -> Result<Vec<CandleSeries>, String> {
    let parsed: Value = serde_json::from_str(input).map_err(|err| err.to_string())?;
    if let Ok(series) = serde_json::from_value::<CandleSeries>(parsed.clone()) {
        return Ok(vec![series]);
    }
    if let Some(items) = parsed.as_array() {
        return items
            .iter()
            .cloned()
            .map(parse_candle_series_value)
            .collect::<Result<Vec<_>, _>>();
    }
    for key in ["series", "candle_series", "items"] {
        if let Some(items) = parsed.get(key).and_then(Value::as_array) {
            return items
                .iter()
                .cloned()
                .map(parse_candle_series_value)
                .collect::<Result<Vec<_>, _>>();
        }
    }
    Ok(vec![parse_candle_series_value(parsed)?])
}

fn parse_candle_series_value(value: Value) -> Result<CandleSeries, String> {
    if let Ok(series) = serde_json::from_value::<CandleSeries>(value.clone()) {
        return Ok(series);
    }
    let symbol = value
        .get("symbol")
        .and_then(Value::as_str)
        .ok_or_else(|| "candle series missing symbol".to_string())?
        .to_string();
    let timeframe = parse_timeframe(
        value
            .get("timeframe")
            .and_then(Value::as_str)
            .unwrap_or("OneDay"),
    );
    let candles = value
        .get("candles")
        .and_then(Value::as_array)
        .ok_or_else(|| "candle series missing candles".to_string())?
        .iter()
        .map(parse_candle)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CandleSeries {
        symbol,
        timeframe,
        candles,
    })
}

fn parse_candle(value: &Value) -> Result<Candle, String> {
    Ok(Candle {
        timestamp_ms: value
            .get("timestamp_ms")
            .and_then(Value::as_u64)
            .ok_or_else(|| "candle missing timestamp_ms".to_string())?,
        open: value
            .get("open")
            .and_then(Value::as_f64)
            .ok_or_else(|| "candle missing open".to_string())?,
        high: value
            .get("high")
            .and_then(Value::as_f64)
            .ok_or_else(|| "candle missing high".to_string())?,
        low: value
            .get("low")
            .and_then(Value::as_f64)
            .ok_or_else(|| "candle missing low".to_string())?,
        close: value
            .get("close")
            .and_then(Value::as_f64)
            .ok_or_else(|| "candle missing close".to_string())?,
        volume: value.get("volume").and_then(Value::as_f64).unwrap_or(0.0),
        trade_value: value.get("trade_value").and_then(Value::as_f64),
        bid: value.get("bid").and_then(Value::as_f64),
        ask: value.get("ask").and_then(Value::as_f64),
        spread_bps: value.get("spread_bps").and_then(Value::as_f64),
    })
}

fn parse_timeframe(raw: &str) -> Timeframe {
    match raw {
        "OneMinute" | "1m" => Timeframe::OneMinute,
        "FiveMinute" | "5m" => Timeframe::FiveMinute,
        "FifteenMinute" | "15m" => Timeframe::FifteenMinute,
        "OneHour" | "1h" => Timeframe::OneHour,
        "OneDay" | "1d" => Timeframe::OneDay,
        _ => Timeframe::OneDay,
    }
}

pub fn normalize_symbol(symbol: &str) -> String {
    symbol.trim().to_ascii_uppercase()
}

pub fn fixture_source_kind(row: &CommitteeScenarioRow) -> bool {
    matches!(
        row.source_kind,
        CommitteeScenarioSourceKind::Fixture | CommitteeScenarioSourceKind::SyntheticTest
    )
}

fn default_horizon_bars() -> usize {
    24
}

fn default_take_profit_pct() -> f64 {
    0.02
}

fn default_stop_loss_pct() -> f64 {
    0.01
}

fn default_true() -> bool {
    true
}
