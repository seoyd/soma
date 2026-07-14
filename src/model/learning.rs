//! Deterministic, CPU-only learning utilities for the shadow Mamba momentum experiment.
//!
//! The Tiny Mamba core is an experimental frozen encoder here. Only a logistic
//! prediction head is trainable, and nothing in this module participates in the
//! active committee or execution path.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::stable_hash_string;

use super::backend::BackendCapabilityProbe;
use super::tiny_tensor::from_vec_1d;
use super::{
    BackendError, BackendFallbackPolicy, BackendOperationSet, BackendPreference, BackendSelection,
    BackendSelectionRequest, CpuMamba3Backend, Mamba3BackendKind, Mamba3ConformanceStatusV0,
    Mamba3ExecutionBackend, Mamba3SisoErrorV0, ModelPrecision, TinyMamba3SisoV0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelMathematicalStatus {
    ExperimentalInternalReference,
    OfficialOracleExecutionBlocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelAgentDeploymentStatus {
    ShadowOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequencePooling {
    LastOutput,
    MeanOutput,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LearningError {
    InvalidConfig,
    InsufficientHistory,
    InvalidCandle,
    NonFinite,
    Shape,
    EmptyTraining,
    EmptyValidation,
    InvalidLabel,
    Backend(BackendError),
    Mamba(Mamba3SisoErrorV0),
    DuplicateVersion,
    FrozenEncoderMutated,
}

impl From<BackendError> for LearningError {
    fn from(value: BackendError) -> Self {
        Self::Backend(value)
    }
}

impl From<Mamba3SisoErrorV0> for LearningError {
    fn from(value: Mamba3SisoErrorV0) -> Self {
        Self::Mamba(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumCandleV0 {
    pub timestamp: i64,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub volume: f32,
}

impl MomentumCandleV0 {
    fn validate(&self) -> Result<(), LearningError> {
        if [self.open, self.high, self.low, self.close, self.volume]
            .iter()
            .any(|value| !value.is_finite())
            || self.open <= 0.0
            || self.high <= 0.0
            || self.low <= 0.0
            || self.close <= 0.0
            || self.volume < 0.0
            || self.high < self.low
        {
            return Err(LearningError::InvalidCandle);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumFeatureConfigV0 {
    pub momentum_lookback: usize,
    pub trend_lookback: usize,
    pub volatility_lookback: usize,
    pub volume_lookback: usize,
    pub drawdown_lookback: usize,
    pub epsilon: f32,
}

impl Default for MomentumFeatureConfigV0 {
    fn default() -> Self {
        Self {
            momentum_lookback: 5,
            trend_lookback: 8,
            volatility_lookback: 8,
            volume_lookback: 8,
            drawdown_lookback: 8,
            epsilon: 1e-6,
        }
    }
}

impl MomentumFeatureConfigV0 {
    pub fn validate(&self) -> Result<(), LearningError> {
        if self.momentum_lookback == 0
            || self.trend_lookback == 0
            || self.volatility_lookback == 0
            || self.volume_lookback == 0
            || self.drawdown_lookback == 0
            || !self.epsilon.is_finite()
            || self.epsilon <= 0.0
        {
            return Err(LearningError::InvalidConfig);
        }
        Ok(())
    }

    pub fn feature_names(&self) -> Vec<String> {
        vec![
            "log_return_1".to_string(),
            format!("momentum_{}", self.momentum_lookback),
            format!("price_to_ma_{}", self.trend_lookback),
            format!("rolling_volatility_{}", self.volatility_lookback),
            format!("drawdown_{}", self.drawdown_lookback),
            format!("volume_zscore_{}", self.volume_lookback),
        ]
    }

    pub fn feature_count(&self) -> usize {
        self.feature_names().len()
    }

    pub fn minimum_history(&self) -> Result<usize, LearningError> {
        self.validate()?;
        Ok(self
            .momentum_lookback
            .max(self.trend_lookback)
            .max(self.volatility_lookback)
            .max(self.volume_lookback)
            .max(self.drawdown_lookback)
            + 1)
    }

    pub fn digest(&self) -> String {
        stable_hash_string(&format!(
            "{}:{}:{}:{}:{}:{:.8}",
            self.momentum_lookback,
            self.trend_lookback,
            self.volatility_lookback,
            self.volume_lookback,
            self.drawdown_lookback,
            self.epsilon
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumFeatureRowV0 {
    pub source_index: usize,
    pub values: Vec<f32>,
}

pub fn build_momentum_features_v0(
    candles: &[MomentumCandleV0],
    config: &MomentumFeatureConfigV0,
) -> Result<Vec<MomentumFeatureRowV0>, LearningError> {
    config.validate()?;
    for candle in candles {
        candle.validate()?;
    }
    let minimum_history = config.minimum_history()?;
    if candles.len() < minimum_history {
        return Err(LearningError::InsufficientHistory);
    }
    let mut rows = Vec::with_capacity(candles.len() - minimum_history + 1);
    for index in minimum_history - 1..candles.len() {
        let close = candles[index].close;
        let log_return = (close / candles[index - 1].close).ln();
        let momentum = close / candles[index - config.momentum_lookback].close - 1.0;
        let trend_start = index + 1 - config.trend_lookback;
        let trend_mean = mean(candles[trend_start..=index].iter().map(|row| row.close))?;
        let price_to_ma = close / trend_mean - 1.0;
        let volatility_start = index + 1 - config.volatility_lookback;
        let returns = (volatility_start..=index)
            .map(|row| (candles[row].close / candles[row - 1].close).ln())
            .collect::<Vec<_>>();
        let volatility = standard_deviation(&returns, config.epsilon)?;
        let drawdown_start = index + 1 - config.drawdown_lookback;
        let rolling_high = candles[drawdown_start..=index]
            .iter()
            .map(|row| row.close)
            .fold(f32::NEG_INFINITY, f32::max);
        let drawdown = close / rolling_high - 1.0;
        let volume_start = index + 1 - config.volume_lookback;
        let volumes = candles[volume_start..=index]
            .iter()
            .map(|row| row.volume)
            .collect::<Vec<_>>();
        let volume_mean = mean(volumes.iter().copied())?;
        let volume_scale = standard_deviation(&volumes, config.epsilon)?;
        let volume_zscore = if volume_scale <= config.epsilon {
            0.0
        } else {
            (candles[index].volume - volume_mean) / volume_scale
        };
        let values = vec![
            log_return,
            momentum,
            price_to_ma,
            volatility,
            drawdown,
            volume_zscore,
        ];
        if values.iter().any(|value| !value.is_finite()) {
            return Err(LearningError::NonFinite);
        }
        rows.push(MomentumFeatureRowV0 {
            source_index: index,
            values,
        });
    }
    Ok(rows)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureNormalizerV0 {
    pub means: Vec<f32>,
    pub scales: Vec<f32>,
    pub fitted_on_start: usize,
    pub fitted_on_end: usize,
    pub constant_feature_indices: Vec<usize>,
}

impl FeatureNormalizerV0 {
    pub fn fit(rows: &[MomentumFeatureRowV0]) -> Result<Self, LearningError> {
        let width = row_width(rows)?;
        let means = (0..width)
            .map(|column| mean(rows.iter().map(|row| row.values[column])))
            .collect::<Result<Vec<_>, _>>()?;
        let mut scales = Vec::with_capacity(width);
        let mut constant_feature_indices = Vec::new();
        for column in 0..width {
            let values = rows
                .iter()
                .map(|row| row.values[column])
                .collect::<Vec<_>>();
            let scale = standard_deviation(&values, 1e-6)?;
            if scale <= 1e-6 {
                scales.push(1.0);
                constant_feature_indices.push(column);
            } else {
                scales.push(scale);
            }
        }
        Ok(Self {
            means,
            scales,
            fitted_on_start: rows
                .first()
                .ok_or(LearningError::EmptyTraining)?
                .source_index,
            fitted_on_end: rows
                .last()
                .ok_or(LearningError::EmptyTraining)?
                .source_index,
            constant_feature_indices,
        })
    }

    pub fn transform(
        &self,
        rows: &[MomentumFeatureRowV0],
    ) -> Result<Vec<MomentumFeatureRowV0>, LearningError> {
        if self.means.len() != self.scales.len()
            || self.means.is_empty()
            || self
                .means
                .iter()
                .chain(&self.scales)
                .any(|value| !value.is_finite())
            || self.scales.iter().any(|value| *value <= 0.0)
        {
            return Err(LearningError::InvalidConfig);
        }
        rows.iter()
            .map(|row| {
                if row.values.len() != self.means.len()
                    || row.values.iter().any(|value| !value.is_finite())
                {
                    return Err(LearningError::Shape);
                }
                let values = row
                    .values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| (value - self.means[index]) / self.scales[index])
                    .collect::<Vec<_>>();
                if values.iter().any(|value| !value.is_finite()) {
                    return Err(LearningError::NonFinite);
                }
                Ok(MomentumFeatureRowV0 {
                    source_index: row.source_index,
                    values,
                })
            })
            .collect()
    }

    pub fn fit_transform(
        rows: &[MomentumFeatureRowV0],
    ) -> Result<(Self, Vec<MomentumFeatureRowV0>), LearningError> {
        let normalizer = Self::fit(rows)?;
        let transformed = normalizer.transform(rows)?;
        Ok((normalizer, transformed))
    }

    pub fn digest(&self) -> String {
        stable_hash_string(&format!(
            "{:?}:{:?}:{}:{}:{:?}",
            self.means,
            self.scales,
            self.fitted_on_start,
            self.fitted_on_end,
            self.constant_feature_indices
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumSequenceConfigV0 {
    pub sequence_length: usize,
    pub prediction_horizon: usize,
    pub label_dead_zone: f32,
    pub stride: usize,
    pub include_neutral_labels: bool,
}

impl Default for MomentumSequenceConfigV0 {
    fn default() -> Self {
        Self {
            sequence_length: 8,
            prediction_horizon: 1,
            label_dead_zone: 0.001,
            stride: 1,
            include_neutral_labels: false,
        }
    }
}

impl MomentumSequenceConfigV0 {
    pub fn validate(&self) -> Result<(), LearningError> {
        if self.sequence_length == 0
            || self.prediction_horizon == 0
            || self.stride == 0
            || !self.label_dead_zone.is_finite()
            || self.label_dead_zone < 0.0
        {
            return Err(LearningError::InvalidConfig);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SequenceExampleV0 {
    pub sequence_start: usize,
    pub sequence_end: usize,
    pub label_index: usize,
    pub input: Vec<Vec<f32>>,
    pub label: f32,
    pub snapshot_ids: Vec<String>,
}

pub fn build_momentum_sequence_examples_v0(
    candles: &[MomentumCandleV0],
    features: &[MomentumFeatureRowV0],
    config: &MomentumSequenceConfigV0,
    snapshot_ids: &[String],
) -> Result<Vec<SequenceExampleV0>, LearningError> {
    config.validate()?;
    if snapshot_ids.is_empty() || features.len() < config.sequence_length {
        return Err(LearningError::InsufficientHistory);
    }
    row_width(features)?;
    for candle in candles {
        candle.validate()?;
    }
    let mut examples = Vec::new();
    for end_position in (config.sequence_length - 1..features.len()).step_by(config.stride) {
        let window = &features[end_position + 1 - config.sequence_length..=end_position];
        if window
            .windows(2)
            .any(|pair| pair[1].source_index != pair[0].source_index + 1)
        {
            return Err(LearningError::Shape);
        }
        let sequence_end = window
            .last()
            .ok_or(LearningError::InsufficientHistory)?
            .source_index;
        let label_index = sequence_end
            .checked_add(config.prediction_horizon)
            .ok_or(LearningError::InvalidConfig)?;
        if label_index >= candles.len() {
            continue;
        }
        let future_return = candles[label_index].close / candles[sequence_end].close - 1.0;
        let label = if future_return > config.label_dead_zone {
            1.0
        } else if future_return < -config.label_dead_zone {
            0.0
        } else if config.include_neutral_labels {
            0.5
        } else {
            continue;
        };
        examples.push(SequenceExampleV0 {
            sequence_start: window[0].source_index,
            sequence_end,
            label_index,
            input: window.iter().map(|row| row.values.clone()).collect(),
            label,
            snapshot_ids: sorted_unique(snapshot_ids),
        });
    }
    if examples.is_empty() {
        return Err(LearningError::InsufficientHistory);
    }
    Ok(examples)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChronologicalSplitConfigV0 {
    pub train_fraction: f32,
    pub validation_fraction: f32,
    pub purge_gap: usize,
}

impl Default for ChronologicalSplitConfigV0 {
    fn default() -> Self {
        Self {
            train_fraction: 0.6,
            validation_fraction: 0.2,
            purge_gap: 9,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChronologicalSplitsV0 {
    pub train: Vec<SequenceExampleV0>,
    pub validation: Vec<SequenceExampleV0>,
    pub test: Vec<SequenceExampleV0>,
}

pub fn chronological_split_v0(
    examples: &[SequenceExampleV0],
    split: &ChronologicalSplitConfigV0,
    sequence: &MomentumSequenceConfigV0,
) -> Result<ChronologicalSplitsV0, LearningError> {
    sequence.validate()?;
    if !split.train_fraction.is_finite()
        || !split.validation_fraction.is_finite()
        || split.train_fraction <= 0.0
        || split.validation_fraction <= 0.0
        || split.train_fraction + split.validation_fraction >= 1.0
        || split.purge_gap < sequence.sequence_length + sequence.prediction_horizon
    {
        return Err(LearningError::InvalidConfig);
    }
    if examples.len() < 3 {
        return Err(LearningError::InsufficientHistory);
    }
    if examples
        .windows(2)
        .any(|pair| pair[1].sequence_end <= pair[0].sequence_end)
    {
        return Err(LearningError::Shape);
    }
    let train_boundary = (examples.len() as f32 * split.train_fraction).floor() as usize;
    let validation_boundary =
        train_boundary + (examples.len() as f32 * split.validation_fraction).floor() as usize;
    let train_end = train_boundary.saturating_sub(split.purge_gap);
    let validation_start = train_boundary.saturating_add(split.purge_gap);
    let validation_end = validation_boundary.saturating_sub(split.purge_gap);
    let test_start = validation_boundary.saturating_add(split.purge_gap);
    if train_end == 0 || validation_start >= validation_end || test_start >= examples.len() {
        return Err(LearningError::InsufficientHistory);
    }
    let result = ChronologicalSplitsV0 {
        train: examples[..train_end].to_vec(),
        validation: examples[validation_start..validation_end].to_vec(),
        test: examples[test_start..].to_vec(),
    };
    if result.train.last().unwrap().label_index >= result.validation.first().unwrap().sequence_start
        || result.validation.last().unwrap().label_index
            >= result.test.first().unwrap().sequence_start
    {
        return Err(LearningError::InvalidConfig);
    }
    Ok(result)
}

#[derive(Clone, Debug)]
pub struct AgentModelRuntimeV0 {
    pub backend_selection: BackendSelection,
    pub mathematical_status: ModelMathematicalStatus,
    cpu_backend: CpuMamba3Backend,
}

impl AgentModelRuntimeV0 {
    pub fn select(
        probe: &impl BackendCapabilityProbe,
        preference: BackendPreference,
        fallback_policy: BackendFallbackPolicy,
    ) -> Result<Self, LearningError> {
        let backend_selection = super::select_mamba3_backend(
            probe,
            BackendSelectionRequest {
                preference,
                fallback_policy,
                required_operations: BackendOperationSet::FULL_INFERENCE,
                required_precision: ModelPrecision::F32,
            },
        )?;
        if backend_selection.selected != Mamba3BackendKind::CpuReference {
            return Err(LearningError::Backend(
                BackendError::StrictBackendUnavailable,
            ));
        }
        Ok(Self {
            backend_selection,
            mathematical_status: ModelMathematicalStatus::ExperimentalInternalReference,
            cpu_backend: CpuMamba3Backend::default(),
        })
    }

    pub fn encode(
        &self,
        model: &TinyMamba3SisoV0,
        input: &[Vec<f32>],
        pooling: SequencePooling,
    ) -> Result<Vec<f32>, LearningError> {
        if self.backend_selection.selected != Mamba3BackendKind::CpuReference {
            return Err(LearningError::Backend(
                BackendError::StrictBackendUnavailable,
            ));
        }
        let tensors = input
            .iter()
            .map(|row| {
                if row.len() != model.config.input_dim {
                    return Err(LearningError::Shape);
                }
                from_vec_1d(row.clone()).map_err(|_| LearningError::NonFinite)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if tensors.is_empty() {
            return Err(LearningError::InsufficientHistory);
        }
        let output = self.cpu_backend.forward(model, &tensors)?;
        let representation = match pooling {
            SequencePooling::LastOutput => output
                .output
                .last()
                .ok_or(LearningError::InsufficientHistory)?
                .values
                .clone(),
            SequencePooling::MeanOutput => {
                let width = output
                    .output
                    .first()
                    .ok_or(LearningError::InsufficientHistory)?
                    .dim;
                let mut mean = vec![0.0; width];
                for row in &output.output {
                    for (index, value) in row.values.iter().enumerate() {
                        mean[index] += value;
                    }
                }
                for value in &mut mean {
                    *value /= output.output.len() as f32;
                }
                mean
            }
        };
        if representation.iter().any(|value| !value.is_finite()) {
            return Err(LearningError::NonFinite);
        }
        Ok(representation)
    }
}

#[derive(Clone, Debug)]
pub struct FrozenMamba3EncoderV0 {
    pub model: TinyMamba3SisoV0,
    pub runtime: AgentModelRuntimeV0,
    pub pooling: SequencePooling,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EncodedSequenceV0 {
    pub representation: Vec<f32>,
    pub backend: Mamba3BackendKind,
    pub mathematical_status: ModelMathematicalStatus,
}

impl FrozenMamba3EncoderV0 {
    pub fn parameter_digest(&self) -> String {
        stable_hash_string(&serde_json::to_string(&self.model).unwrap_or_default())
    }

    pub fn encode_sequence(&self, input: &[Vec<f32>]) -> Result<EncodedSequenceV0, LearningError> {
        let representation = self.runtime.encode(&self.model, input, self.pooling)?;
        Ok(EncodedSequenceV0 {
            representation,
            backend: self.runtime.backend_selection.selected,
            mathematical_status: self.runtime.mathematical_status,
        })
    }

    pub fn encode_batch(
        &self,
        examples: &[SequenceExampleV0],
    ) -> Result<Vec<EncodedTrainingExampleV0>, LearningError> {
        examples
            .iter()
            .map(|example| {
                validate_label(example.label)?;
                let encoded = self.encode_sequence(&example.input)?;
                Ok(EncodedTrainingExampleV0 {
                    representation: encoded.representation,
                    label: example.label,
                    snapshot_ids: example.snapshot_ids.clone(),
                })
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EncodedTrainingExampleV0 {
    pub representation: Vec<f32>,
    pub label: f32,
    pub snapshot_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogisticPredictionHeadV0 {
    pub weights: Vec<f32>,
    pub bias: f32,
}

impl LogisticPredictionHeadV0 {
    pub fn seeded(dimension: usize, seed: u64) -> Result<Self, LearningError> {
        if dimension == 0 {
            return Err(LearningError::Shape);
        }
        let weights = (0..dimension)
            .map(|index| seeded_value(seed, index, 41))
            .collect::<Vec<_>>();
        let result = Self {
            weights,
            bias: seeded_value(seed, dimension, 43),
        };
        result.validate()?;
        Ok(result)
    }

    pub fn validate(&self) -> Result<(), LearningError> {
        if self.weights.is_empty()
            || !self.bias.is_finite()
            || self.weights.iter().any(|value| !value.is_finite())
        {
            return Err(LearningError::NonFinite);
        }
        Ok(())
    }

    pub fn parameter_digest(&self) -> String {
        stable_hash_string(&format!("{:?}:{:.8}", self.weights, self.bias))
    }

    pub fn probability(&self, representation: &[f32]) -> Result<f32, LearningError> {
        self.validate()?;
        if representation.len() != self.weights.len()
            || representation.iter().any(|value| !value.is_finite())
        {
            return Err(LearningError::Shape);
        }
        let logit = self.bias
            + self
                .weights
                .iter()
                .zip(representation)
                .map(|(weight, value)| weight * value)
                .sum::<f32>();
        if !logit.is_finite() {
            return Err(LearningError::NonFinite);
        }
        let probability = stable_sigmoid(logit);
        if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
            return Err(LearningError::NonFinite);
        }
        Ok(probability)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeadGradientsV0 {
    pub weight_gradients: Vec<f32>,
    pub bias_gradient: f32,
}

pub fn brier_loss_and_gradients_v0(
    head: &LogisticPredictionHeadV0,
    examples: &[EncodedTrainingExampleV0],
) -> Result<(f32, HeadGradientsV0), LearningError> {
    if examples.is_empty() {
        return Err(LearningError::EmptyTraining);
    }
    head.validate()?;
    let mut loss = 0.0;
    let mut gradients = HeadGradientsV0 {
        weight_gradients: vec![0.0; head.weights.len()],
        bias_gradient: 0.0,
    };
    for example in examples {
        validate_label(example.label)?;
        let probability = head.probability(&example.representation)?;
        let residual = probability - example.label;
        loss += residual * residual;
        let derivative = 2.0 * residual * probability * (1.0 - probability);
        for (gradient, value) in gradients
            .weight_gradients
            .iter_mut()
            .zip(&example.representation)
        {
            *gradient += derivative * value;
        }
        gradients.bias_gradient += derivative;
    }
    let count = examples.len() as f32;
    loss /= count;
    for gradient in &mut gradients.weight_gradients {
        *gradient /= count;
    }
    gradients.bias_gradient /= count;
    if !loss.is_finite()
        || !gradients.bias_gradient.is_finite()
        || gradients
            .weight_gradients
            .iter()
            .any(|value| !value.is_finite())
    {
        return Err(LearningError::NonFinite);
    }
    Ok((loss, gradients))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SgdConfigV0 {
    pub learning_rate: f32,
    pub weight_decay: f32,
    pub gradient_clip_norm: Option<f32>,
}

impl Default for SgdConfigV0 {
    fn default() -> Self {
        Self {
            learning_rate: 0.05,
            weight_decay: 0.0,
            gradient_clip_norm: None,
        }
    }
}

impl SgdConfigV0 {
    pub fn validate(&self) -> Result<(), LearningError> {
        if !self.learning_rate.is_finite()
            || self.learning_rate <= 0.0
            || !self.weight_decay.is_finite()
            || self.weight_decay < 0.0
            || self
                .gradient_clip_norm
                .is_some_and(|value| !value.is_finite() || value <= 0.0)
        {
            return Err(LearningError::InvalidConfig);
        }
        Ok(())
    }

    pub fn digest(&self) -> String {
        stable_hash_string(&format!(
            "{:.8}:{:.8}:{:?}",
            self.learning_rate, self.weight_decay, self.gradient_clip_norm
        ))
    }
}

pub fn apply_sgd_v0(
    head: &mut LogisticPredictionHeadV0,
    gradients: &HeadGradientsV0,
    config: &SgdConfigV0,
) -> Result<(), LearningError> {
    config.validate()?;
    head.validate()?;
    if gradients.weight_gradients.len() != head.weights.len()
        || !gradients.bias_gradient.is_finite()
        || gradients
            .weight_gradients
            .iter()
            .any(|value| !value.is_finite())
    {
        return Err(LearningError::Shape);
    }
    let mut updates = gradients
        .weight_gradients
        .iter()
        .zip(&head.weights)
        .map(|(gradient, weight)| gradient + config.weight_decay * weight)
        .collect::<Vec<_>>();
    let mut bias_update = gradients.bias_gradient;
    let norm =
        (updates.iter().map(|value| value * value).sum::<f32>() + bias_update * bias_update).sqrt();
    if !norm.is_finite() {
        return Err(LearningError::NonFinite);
    }
    if let Some(limit) = config.gradient_clip_norm.filter(|limit| norm > *limit) {
        let scale = limit / norm;
        for update in &mut updates {
            *update *= scale;
        }
        bias_update *= scale;
    }
    for (weight, update) in head.weights.iter_mut().zip(updates) {
        *weight -= config.learning_rate * update;
    }
    head.bias -= config.learning_rate * bias_update;
    head.validate()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeadTrainingConfigV0 {
    pub epochs: usize,
    pub batch_size: usize,
    pub optimizer: SgdConfigV0,
    pub seed: u64,
    pub early_stopping_patience: Option<usize>,
}

impl Default for HeadTrainingConfigV0 {
    fn default() -> Self {
        Self {
            epochs: 30,
            batch_size: 8,
            optimizer: SgdConfigV0::default(),
            seed: 7,
            early_stopping_patience: Some(8),
        }
    }
}

impl HeadTrainingConfigV0 {
    pub fn validate(&self) -> Result<(), LearningError> {
        if self.epochs == 0 || self.batch_size == 0 {
            return Err(LearningError::InvalidConfig);
        }
        self.optimizer.validate()
    }
    pub fn digest(&self) -> String {
        stable_hash_string(&format!(
            "{}:{}:{}:{}:{:?}",
            self.epochs,
            self.batch_size,
            self.optimizer.digest(),
            self.seed,
            self.early_stopping_patience
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrainingEpochMetricsV0 {
    pub epoch: usize,
    pub train_brier: f32,
    pub validation_brier: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeadTrainingResultV0 {
    pub initial_head: LogisticPredictionHeadV0,
    pub final_head: LogisticPredictionHeadV0,
    pub best_head: LogisticPredictionHeadV0,
    pub epoch_metrics: Vec<TrainingEpochMetricsV0>,
    pub stopped_epoch: usize,
    pub encoder_digest_before: String,
    pub encoder_digest_after: String,
    pub backend: Mamba3BackendKind,
}

pub fn train_frozen_mamba_head_v0(
    encoder: &FrozenMamba3EncoderV0,
    initial_head: LogisticPredictionHeadV0,
    train: &[SequenceExampleV0],
    validation: &[SequenceExampleV0],
    config: &HeadTrainingConfigV0,
) -> Result<HeadTrainingResultV0, LearningError> {
    config.validate()?;
    if train.is_empty() {
        return Err(LearningError::EmptyTraining);
    }
    if validation.is_empty() {
        return Err(LearningError::EmptyValidation);
    }
    let encoder_digest_before = encoder.parameter_digest();
    let train = encoder.encode_batch(train)?;
    let validation = encoder.encode_batch(validation)?;
    let mut head = initial_head.clone();
    head.validate()?;
    let mut best_head = head.clone();
    let mut best_validation = f32::INFINITY;
    let mut no_improvement = 0usize;
    let mut epoch_metrics = Vec::new();
    let mut stopped_epoch = config.epochs;
    for epoch in 1..=config.epochs {
        for batch in train.chunks(config.batch_size) {
            let (_, gradients) = brier_loss_and_gradients_v0(&head, batch)?;
            apply_sgd_v0(&mut head, &gradients, &config.optimizer)?;
        }
        let train_brier = brier_loss_and_gradients_v0(&head, &train)?.0;
        let validation_brier = brier_loss_and_gradients_v0(&head, &validation)?.0;
        epoch_metrics.push(TrainingEpochMetricsV0 {
            epoch,
            train_brier,
            validation_brier,
        });
        if validation_brier < best_validation {
            best_validation = validation_brier;
            best_head = head.clone();
            no_improvement = 0;
        } else {
            no_improvement += 1;
            if config
                .early_stopping_patience
                .is_some_and(|patience| no_improvement >= patience)
            {
                stopped_epoch = epoch;
                break;
            }
        }
    }
    let final_head = best_head.clone();
    let encoder_digest_after = encoder.parameter_digest();
    if encoder_digest_before != encoder_digest_after {
        return Err(LearningError::FrozenEncoderMutated);
    }
    Ok(HeadTrainingResultV0 {
        initial_head,
        final_head,
        best_head,
        epoch_metrics,
        stopped_epoch,
        encoder_digest_before,
        encoder_digest_after,
        backend: encoder.runtime.backend_selection.selected,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CalibrationBucketV0 {
    pub lower_bound: f32,
    pub upper_bound: f32,
    pub sample_count: usize,
    pub mean_probability: f32,
    pub positive_label_rate: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvaluationMetricsV0 {
    pub brier_score: f32,
    pub sample_count: usize,
    pub accuracy: f32,
    pub positive_label_rate: f32,
    pub mean_predicted_probability: f32,
    pub high_confidence_error_count: usize,
    pub abstention_count: usize,
    pub calibration_buckets: Vec<CalibrationBucketV0>,
}

pub fn evaluate_head_v0(
    head: &LogisticPredictionHeadV0,
    examples: &[EncodedTrainingExampleV0],
) -> Result<EvaluationMetricsV0, LearningError> {
    if examples.is_empty() {
        return Err(LearningError::InsufficientHistory);
    }
    let probabilities = examples
        .iter()
        .map(|example| head.probability(&example.representation))
        .collect::<Result<Vec<_>, _>>()?;
    evaluate_probabilities_v0(
        &probabilities,
        &examples
            .iter()
            .map(|example| example.label)
            .collect::<Vec<_>>(),
    )
}

pub fn evaluate_probabilities_v0(
    probabilities: &[f32],
    labels: &[f32],
) -> Result<EvaluationMetricsV0, LearningError> {
    if probabilities.is_empty() || probabilities.len() != labels.len() {
        return Err(LearningError::Shape);
    }
    let mut brier = 0.0;
    let mut correct = 0usize;
    let mut positives = 0.0;
    let mut probability_sum = 0.0;
    let mut high_confidence_error_count = 0usize;
    for (probability, label) in probabilities.iter().zip(labels) {
        validate_label(*label)?;
        if !probability.is_finite() || !(0.0..=1.0).contains(probability) {
            return Err(LearningError::NonFinite);
        }
        brier += (probability - label).powi(2);
        correct += usize::from((*probability >= 0.5) == (*label >= 0.5));
        positives += label;
        probability_sum += probability;
        high_confidence_error_count += usize::from(
            (*probability >= 0.8 && *label < 0.5) || (*probability <= 0.2 && *label > 0.5),
        );
    }
    let count = probabilities.len() as f32;
    let calibration_buckets = (0..5)
        .map(|bucket| {
            let lower_bound = bucket as f32 / 5.0;
            let upper_bound = (bucket + 1) as f32 / 5.0;
            let grouped = probabilities
                .iter()
                .zip(labels)
                .filter(|(probability, _)| {
                    **probability >= lower_bound && (**probability < upper_bound || bucket == 4)
                })
                .collect::<Vec<_>>();
            let sample_count = grouped.len();
            let mean_probability = if sample_count == 0 {
                0.0
            } else {
                grouped
                    .iter()
                    .map(|(probability, _)| **probability)
                    .sum::<f32>()
                    / sample_count as f32
            };
            let positive_label_rate = if sample_count == 0 {
                0.0
            } else {
                grouped.iter().map(|(_, label)| **label).sum::<f32>() / sample_count as f32
            };
            CalibrationBucketV0 {
                lower_bound,
                upper_bound,
                sample_count,
                mean_probability,
                positive_label_rate,
            }
        })
        .collect();
    Ok(EvaluationMetricsV0 {
        brier_score: brier / count,
        sample_count: probabilities.len(),
        accuracy: correct as f32 / count,
        positive_label_rate: positives / count,
        mean_predicted_probability: probability_sum / count,
        high_confidence_error_count,
        abstention_count: 0,
        calibration_buckets,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstantProbabilityBaselineV0 {
    pub probability: f32,
}

impl ConstantProbabilityBaselineV0 {
    pub fn fit(train: &[SequenceExampleV0]) -> Result<Self, LearningError> {
        if train.is_empty() {
            return Err(LearningError::EmptyTraining);
        }
        let probability = train
            .iter()
            .map(|example| {
                validate_label(example.label)?;
                Ok(example.label)
            })
            .collect::<Result<Vec<_>, LearningError>>()?
            .iter()
            .sum::<f32>()
            / train.len() as f32;
        Ok(Self { probability })
    }
    pub fn evaluate(
        &self,
        examples: &[SequenceExampleV0],
    ) -> Result<EvaluationMetricsV0, LearningError> {
        evaluate_probabilities_v0(
            &vec![self.probability; examples.len()],
            &examples
                .iter()
                .map(|example| example.label)
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinearMomentumBaselineV0 {
    pub head: LogisticPredictionHeadV0,
}

impl LinearMomentumBaselineV0 {
    pub fn train(
        train: &[SequenceExampleV0],
        validation: &[SequenceExampleV0],
        config: &HeadTrainingConfigV0,
    ) -> Result<Self, LearningError> {
        let train = raw_last_examples(train)?;
        let validation = raw_last_examples(validation)?;
        if train.is_empty() {
            return Err(LearningError::EmptyTraining);
        }
        if validation.is_empty() {
            return Err(LearningError::EmptyValidation);
        }
        config.validate()?;
        let mut head =
            LogisticPredictionHeadV0::seeded(train[0].representation.len(), config.seed)?;
        let mut best_head = head.clone();
        let mut best_validation = f32::INFINITY;
        let mut no_improvement = 0usize;
        for _ in 0..config.epochs {
            for batch in train.chunks(config.batch_size) {
                let (_, gradients) = brier_loss_and_gradients_v0(&head, batch)?;
                apply_sgd_v0(&mut head, &gradients, &config.optimizer)?;
            }
            let validation_brier = brier_loss_and_gradients_v0(&head, &validation)?.0;
            if validation_brier < best_validation {
                best_validation = validation_brier;
                best_head = head.clone();
                no_improvement = 0;
            } else {
                no_improvement += 1;
                if config
                    .early_stopping_patience
                    .is_some_and(|patience| no_improvement >= patience)
                {
                    break;
                }
            }
        }
        Ok(Self { head: best_head })
    }
    pub fn evaluate(
        &self,
        examples: &[SequenceExampleV0],
    ) -> Result<EvaluationMetricsV0, LearningError> {
        evaluate_head_v0(&self.head, &raw_last_examples(examples)?)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BaselineComparisonV0 {
    pub constant_probability: EvaluationMetricsV0,
    pub linear_momentum: EvaluationMetricsV0,
    pub frozen_mamba: EvaluationMetricsV0,
    pub current_deterministic_momentum_policy: Option<EvaluationMetricsV0>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MambaRepresentationValueStatusV0 {
    Helped,
    Failed,
    Mixed,
    InsufficientEvidence,
}

pub fn mamba_representation_value_status_v0(
    frozen_mamba: &EvaluationMetricsV0,
    constant: &EvaluationMetricsV0,
    linear: &EvaluationMetricsV0,
    minimum_samples: usize,
) -> MambaRepresentationValueStatusV0 {
    if frozen_mamba.sample_count < minimum_samples
        || constant.sample_count < minimum_samples
        || linear.sample_count < minimum_samples
    {
        return MambaRepresentationValueStatusV0::InsufficientEvidence;
    }
    if frozen_mamba.brier_score < constant.brier_score
        && frozen_mamba.brier_score < linear.brier_score
        && frozen_mamba.high_confidence_error_count <= linear.high_confidence_error_count
    {
        MambaRepresentationValueStatusV0::Helped
    } else if frozen_mamba.brier_score > linear.brier_score {
        MambaRepresentationValueStatusV0::Failed
    } else {
        MambaRepresentationValueStatusV0::Mixed
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexRangeV0 {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SandboxModelMetricsV0 {
    pub train: EvaluationMetricsV0,
    pub validation: EvaluationMetricsV0,
    pub test: EvaluationMetricsV0,
    pub mamba_value_status: MambaRepresentationValueStatusV0,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SandboxModelVersionV0 {
    pub model_version_id: String,
    pub parent_version_id: Option<String>,
    pub agent_id: String,
    pub architecture: String,
    pub deployment_status: ModelAgentDeploymentStatus,
    pub mathematical_status: ModelMathematicalStatus,
    pub official_conformance_status: Mamba3ConformanceStatusV0,
    pub feature_config_digest: String,
    pub normalizer_digest: String,
    pub encoder_parameter_digest: String,
    pub head_parameter_digest: String,
    pub training_config_digest: String,
    pub data_snapshot_ids: Vec<String>,
    pub train_range: IndexRangeV0,
    pub validation_range: IndexRangeV0,
    pub test_range: IndexRangeV0,
    pub backend: Mamba3BackendKind,
    pub metrics: SandboxModelMetricsV0,
    #[serde(default)]
    pub campaign_id: Option<String>,
    #[serde(default)]
    pub window_id: Option<String>,
    #[serde(default)]
    pub training_path: Option<String>,
    #[serde(default)]
    pub initial_head_parameter_digest: Option<String>,
    #[serde(default)]
    pub backend_fallback_reason: Option<String>,
    #[serde(default)]
    pub drift_status: Option<String>,
    #[serde(default)]
    pub creation_reason_codes: Vec<String>,
}

impl SandboxModelVersionV0 {
    pub fn new(
        parent_version_id: Option<String>,
        agent_id: impl Into<String>,
        feature_config_digest: String,
        normalizer_digest: String,
        encoder_parameter_digest: String,
        head_parameter_digest: String,
        training_config_digest: String,
        data_snapshot_ids: &[String],
        train_range: IndexRangeV0,
        validation_range: IndexRangeV0,
        test_range: IndexRangeV0,
        backend: Mamba3BackendKind,
        metrics: SandboxModelMetricsV0,
    ) -> Self {
        let agent_id = agent_id.into();
        let data_snapshot_ids = sorted_unique(data_snapshot_ids);
        let material = format!(
            "{:?}:{agent_id}:{}:{}:{}:{}:{}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}",
            parent_version_id,
            feature_config_digest,
            normalizer_digest,
            encoder_parameter_digest,
            head_parameter_digest,
            training_config_digest,
            data_snapshot_ids,
            train_range,
            validation_range,
            test_range,
            backend,
            metrics
        );
        Self {
            model_version_id: format!("frozen-mamba-reservoir-{}", stable_hash_string(&material)),
            parent_version_id,
            agent_id,
            architecture: "FrozenMambaReservoirHeadV0".to_string(),
            deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
            mathematical_status: ModelMathematicalStatus::ExperimentalInternalReference,
            official_conformance_status: Mamba3ConformanceStatusV0::OfficialOracleExecutionBlocked,
            feature_config_digest,
            normalizer_digest,
            encoder_parameter_digest,
            head_parameter_digest,
            training_config_digest,
            data_snapshot_ids,
            train_range,
            validation_range,
            test_range,
            backend,
            metrics,
            campaign_id: None,
            window_id: None,
            training_path: None,
            initial_head_parameter_digest: None,
            backend_fallback_reason: None,
            drift_status: None,
            creation_reason_codes: vec![],
        }
    }

    pub fn with_campaign_metadata(
        mut self,
        campaign_id: impl Into<String>,
        window_id: impl Into<String>,
        training_path: impl Into<String>,
        initial_head_parameter_digest: String,
        backend_fallback_reason: Option<String>,
        drift_status: impl Into<String>,
        creation_reason_codes: Vec<String>,
    ) -> Self {
        self.campaign_id = Some(campaign_id.into());
        self.window_id = Some(window_id.into());
        self.training_path = Some(training_path.into());
        self.initial_head_parameter_digest = Some(initial_head_parameter_digest);
        self.backend_fallback_reason = backend_fallback_reason;
        self.drift_status = Some(drift_status.into());
        self.creation_reason_codes = creation_reason_codes;
        self.model_version_id = format!(
            "frozen-mamba-campaign-{}",
            stable_hash_string(&format!(
                "{}:{:?}:{:?}:{:?}:{:?}:{:?}",
                self.model_version_id,
                self.campaign_id,
                self.window_id,
                self.training_path,
                self.parent_version_id,
                self.initial_head_parameter_digest
            ))
        );
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct SandboxModelVersionJournalV0 {
    versions: BTreeMap<String, SandboxModelVersionV0>,
}

impl SandboxModelVersionJournalV0 {
    pub fn insert(&mut self, version: SandboxModelVersionV0) -> Result<(), LearningError> {
        if self.versions.contains_key(&version.model_version_id) {
            return Err(LearningError::DuplicateVersion);
        }
        self.versions
            .insert(version.model_version_id.clone(), version);
        Ok(())
    }
    pub fn get(&self, id: &str) -> Option<&SandboxModelVersionV0> {
        self.versions.get(id)
    }
}

fn raw_last_examples(
    examples: &[SequenceExampleV0],
) -> Result<Vec<EncodedTrainingExampleV0>, LearningError> {
    examples
        .iter()
        .map(|example| {
            let representation = example.input.last().ok_or(LearningError::Shape)?.clone();
            validate_label(example.label)?;
            Ok(EncodedTrainingExampleV0 {
                representation,
                label: example.label,
                snapshot_ids: example.snapshot_ids.clone(),
            })
        })
        .collect()
}

fn stable_sigmoid(value: f32) -> f32 {
    if value >= 0.0 {
        1.0 / (1.0 + (-value).exp())
    } else {
        let exp = value.exp();
        exp / (1.0 + exp)
    }
}

fn mean(values: impl Iterator<Item = f32>) -> Result<f32, LearningError> {
    let values = values.collect::<Vec<_>>();
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err(LearningError::NonFinite);
    }
    let value = values.iter().sum::<f32>() / values.len() as f32;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(LearningError::NonFinite)
    }
}

fn standard_deviation(values: &[f32], epsilon: f32) -> Result<f32, LearningError> {
    let average = mean(values.iter().copied())?;
    let variance = values
        .iter()
        .map(|value| (value - average).powi(2))
        .sum::<f32>()
        / values.len() as f32;
    let result = variance.max(0.0).sqrt();
    if !result.is_finite() || !epsilon.is_finite() {
        Err(LearningError::NonFinite)
    } else {
        Ok(result)
    }
}

fn row_width(rows: &[MomentumFeatureRowV0]) -> Result<usize, LearningError> {
    let width = rows
        .first()
        .ok_or(LearningError::EmptyTraining)?
        .values
        .len();
    if width == 0
        || rows.iter().any(|row| {
            row.values.len() != width || row.values.iter().any(|value| !value.is_finite())
        })
    {
        Err(LearningError::Shape)
    } else {
        Ok(width)
    }
}

fn validate_label(value: f32) -> Result<(), LearningError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(LearningError::InvalidLabel)
    }
}

fn sorted_unique(values: &[String]) -> Vec<String> {
    let mut values = values.to_vec();
    values.sort();
    values.dedup();
    values
}

fn seeded_value(seed: u64, index: usize, salt: u64) -> f32 {
    let mixed = seed
        .wrapping_mul(1_103_515_245)
        .wrapping_add((index as u64 + 1) * 12_345)
        .wrapping_add(salt * 97);
    ((mixed % 97) as f32 - 48.0) / 960.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BackendCapabilities, BackendReadiness, BackendReasonCode, Mamba3SisoConfigV0,
        Mamba3SisoPrecisionV0, Mamba3SisoRopeFractionV0, StaticBackendCapabilityProbe,
    };

    fn candles() -> Vec<MomentumCandleV0> {
        (0..160)
            .map(|index| {
                let close = 100.0 + index as f32 * 0.5 + if index % 7 == 0 { -0.3 } else { 0.0 };
                MomentumCandleV0 {
                    timestamp: index as i64,
                    open: close - 0.2,
                    high: close + 0.4,
                    low: close - 0.5,
                    close,
                    volume: 1_000.0 + (index % 9) as f32 * 30.0,
                }
            })
            .collect()
    }

    fn examples() -> Vec<SequenceExampleV0> {
        let config = MomentumFeatureConfigV0::default();
        let features = build_momentum_features_v0(&candles(), &config).unwrap();
        let normalizer = FeatureNormalizerV0::fit(&features[..40]).unwrap();
        build_momentum_sequence_examples_v0(
            &candles(),
            &normalizer.transform(&features).unwrap(),
            &MomentumSequenceConfigV0::default(),
            &["snapshot-a".to_string()],
        )
        .unwrap()
    }

    #[test]
    fn features_are_ordered_and_train_only_normalization_is_stable() {
        let config = MomentumFeatureConfigV0::default();
        let mut later = candles();
        let features = build_momentum_features_v0(&later, &config).unwrap();
        let first = FeatureNormalizerV0::fit(&features[..20]).unwrap();
        later[60].close *= 2.0;
        let changed = build_momentum_features_v0(&later, &config).unwrap();
        assert_eq!(first, FeatureNormalizerV0::fit(&changed[..20]).unwrap());
        assert_eq!(config.feature_names().len(), 6);
        assert_eq!(
            features[0].source_index,
            config.minimum_history().unwrap() - 1
        );
    }

    #[test]
    fn sequences_and_purged_splits_are_chronological() {
        let sequence = MomentumSequenceConfigV0::default();
        let split = ChronologicalSplitConfigV0 {
            purge_gap: sequence.sequence_length + sequence.prediction_horizon,
            ..ChronologicalSplitConfigV0::default()
        };
        let splits = chronological_split_v0(&examples(), &split, &sequence).unwrap();
        assert!(
            splits.train.last().unwrap().label_index
                < splits.validation.first().unwrap().sequence_start
        );
        assert!(
            splits.validation.last().unwrap().label_index
                < splits.test.first().unwrap().sequence_start
        );
    }

    #[test]
    fn brier_gradient_matches_central_difference() {
        let head = LogisticPredictionHeadV0 {
            weights: vec![0.2, -0.1],
            bias: 0.05,
        };
        let samples = vec![EncodedTrainingExampleV0 {
            representation: vec![0.4, -0.3],
            label: 1.0,
            snapshot_ids: vec![],
        }];
        let (_, gradients) = brier_loss_and_gradients_v0(&head, &samples).unwrap();
        let step = 1e-3;
        let mut plus = head.clone();
        plus.weights[0] += step;
        let mut minus = head.clone();
        minus.weights[0] -= step;
        let numerical = (brier_loss_and_gradients_v0(&plus, &samples).unwrap().0
            - brier_loss_and_gradients_v0(&minus, &samples).unwrap().0)
            / (2.0 * step);
        assert!((gradients.weight_gradients[0] - numerical).abs() < 1e-3);
    }

    #[test]
    fn sgd_and_metrics_are_deterministic() {
        let samples = vec![
            EncodedTrainingExampleV0 {
                representation: vec![1.0],
                label: 1.0,
                snapshot_ids: vec![],
            },
            EncodedTrainingExampleV0 {
                representation: vec![-1.0],
                label: 0.0,
                snapshot_ids: vec![],
            },
        ];
        let mut head = LogisticPredictionHeadV0::seeded(1, 3).unwrap();
        let before = brier_loss_and_gradients_v0(&head, &samples).unwrap().0;
        for _ in 0..30 {
            let (_, gradients) = brier_loss_and_gradients_v0(&head, &samples).unwrap();
            apply_sgd_v0(&mut head, &gradients, &SgdConfigV0::default()).unwrap();
        }
        assert!(brier_loss_and_gradients_v0(&head, &samples).unwrap().0 < before);
        assert_eq!(
            evaluate_head_v0(&head, &samples).unwrap(),
            evaluate_head_v0(&head, &samples).unwrap()
        );
    }

    #[test]
    fn value_status_and_version_are_computed() {
        let metrics = EvaluationMetricsV0 {
            brier_score: 0.2,
            sample_count: 20,
            accuracy: 0.6,
            positive_label_rate: 0.5,
            mean_predicted_probability: 0.5,
            high_confidence_error_count: 1,
            abstention_count: 0,
            calibration_buckets: vec![],
        };
        let worse = EvaluationMetricsV0 {
            brier_score: 0.3,
            high_confidence_error_count: 2,
            ..metrics.clone()
        };
        assert_eq!(
            mamba_representation_value_status_v0(&metrics, &worse, &worse, 10),
            MambaRepresentationValueStatusV0::Helped
        );
        let version = SandboxModelVersionV0::new(
            None,
            "shadow",
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
            &["s2".to_string(), "s1".to_string()],
            IndexRangeV0 { start: 0, end: 1 },
            IndexRangeV0 { start: 2, end: 3 },
            IndexRangeV0 { start: 4, end: 5 },
            Mamba3BackendKind::CpuReference,
            SandboxModelMetricsV0 {
                train: metrics.clone(),
                validation: metrics.clone(),
                test: metrics,
                mamba_value_status: MambaRepresentationValueStatusV0::Helped,
            },
        );
        assert_eq!(
            version.deployment_status,
            ModelAgentDeploymentStatus::ShadowOnly
        );
        let mut journal = SandboxModelVersionJournalV0::default();
        journal.insert(version.clone()).unwrap();
        assert_eq!(
            journal.insert(version),
            Err(LearningError::DuplicateVersion)
        );
    }

    #[test]
    fn runtime_requires_full_ready_cpu_selection() {
        let cpu = BackendCapabilities {
            kind: Mamba3BackendKind::CpuReference,
            readiness: BackendReadiness::FullInferenceReady,
            supported_operations: BackendOperationSet::FULL_INFERENCE,
            supported_precisions: vec![ModelPrecision::F32],
            device_count: 1,
            selected_device: None,
            reason_codes: vec![BackendReasonCode::CpuReferenceAvailable],
        };
        let partial = BackendCapabilities {
            kind: Mamba3BackendKind::Metal,
            readiness: BackendReadiness::PartialOperations,
            supported_operations: BackendOperationSet::EMPTY,
            supported_precisions: vec![],
            device_count: 1,
            selected_device: None,
            reason_codes: vec![BackendReasonCode::PartialOperationCoverage],
        };
        let probe = StaticBackendCapabilityProbe {
            cpu: cpu.clone(),
            metal: partial.clone(),
            cuda: partial,
        };
        assert_eq!(
            AgentModelRuntimeV0::select(
                &probe,
                BackendPreference::Auto,
                BackendFallbackPolicy::AllowCpuFallback
            )
            .unwrap()
            .backend_selection
            .selected,
            Mamba3BackendKind::CpuReference
        );
        assert!(
            AgentModelRuntimeV0::select(
                &probe,
                BackendPreference::Metal,
                BackendFallbackPolicy::Strict
            )
            .is_err()
        );
    }

    #[test]
    fn frozen_encoder_is_deterministic() {
        let config = Mamba3SisoConfigV0 {
            input_dim: 6,
            state_dim: 8,
            head_dim: 2,
            expansion: 1,
            rope_fraction: Mamba3SisoRopeFractionV0::Half,
            norm_epsilon: 1e-5,
            a_floor: 1e-4,
            mimo_rank: 1,
            precision: Mamba3SisoPrecisionV0::F32,
            short_convolution_enabled: false,
        };
        let model = TinyMamba3SisoV0::new(
            config.clone(),
            super::super::mamba3_siso_params_from_seed_v0(&config, 19).unwrap(),
        )
        .unwrap();
        let probe = super::super::SystemBackendCapabilityProbe;
        let encoder = FrozenMamba3EncoderV0 {
            model,
            runtime: AgentModelRuntimeV0::select(
                &probe,
                BackendPreference::Auto,
                BackendFallbackPolicy::AllowCpuFallback,
            )
            .unwrap(),
            pooling: SequencePooling::MeanOutput,
        };
        let first = encoder.encode_sequence(&examples()[0].input).unwrap();
        assert_eq!(
            first,
            encoder.encode_sequence(&examples()[0].input).unwrap()
        );
    }

    #[test]
    fn frozen_head_training_updates_only_the_head_and_restores_best_checkpoint() {
        let config = Mamba3SisoConfigV0 {
            input_dim: 6,
            state_dim: 8,
            head_dim: 2,
            expansion: 1,
            rope_fraction: Mamba3SisoRopeFractionV0::Half,
            norm_epsilon: 1e-5,
            a_floor: 1e-4,
            mimo_rank: 1,
            precision: Mamba3SisoPrecisionV0::F32,
            short_convolution_enabled: false,
        };
        let model = TinyMamba3SisoV0::new(
            config.clone(),
            super::super::mamba3_siso_params_from_seed_v0(&config, 23).unwrap(),
        )
        .unwrap();
        let encoder = FrozenMamba3EncoderV0 {
            model,
            runtime: AgentModelRuntimeV0::select(
                &super::super::SystemBackendCapabilityProbe,
                BackendPreference::Auto,
                BackendFallbackPolicy::AllowCpuFallback,
            )
            .unwrap(),
            pooling: SequencePooling::LastOutput,
        };
        let sequence = MomentumSequenceConfigV0::default();
        let splits = chronological_split_v0(
            &examples(),
            &ChronologicalSplitConfigV0 {
                purge_gap: sequence.sequence_length + sequence.prediction_horizon,
                ..ChronologicalSplitConfigV0::default()
            },
            &sequence,
        )
        .unwrap();
        let initial = LogisticPredictionHeadV0::seeded(6, 29).unwrap();
        let result = train_frozen_mamba_head_v0(
            &encoder,
            initial.clone(),
            &splits.train,
            &splits.validation,
            &HeadTrainingConfigV0 {
                epochs: 4,
                batch_size: 4,
                optimizer: SgdConfigV0::default(),
                seed: 29,
                early_stopping_patience: None,
            },
        )
        .unwrap();
        assert_eq!(result.encoder_digest_before, result.encoder_digest_after);
        assert_eq!(result.final_head, result.best_head);
        assert_ne!(result.final_head, initial);
        assert_eq!(result.backend, Mamba3BackendKind::CpuReference);
    }
}
