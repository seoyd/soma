use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::backtest::{
    BarrierHit, CandleSeries, Timeframe, TripleBarrierConfig, TripleBarrierOutcome,
    evaluate_triple_barrier,
};
use crate::core::{ReasonCode, Regime, stable_hash};
use crate::feature::{FeatureEngine, FeatureName, FeatureValue};
use crate::regime::RegimeClassifier;

use super::leakage::row_is_unsafe;
use super::walk_forward::{WalkForwardFold, WalkForwardSplit};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatasetSplitKind {
    Train,
    Validation,
    Test,
    Embargo,
    Unsafe,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatasetOutputFormat {
    Csv,
    JsonLines,
    InMemoryOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatasetExportConfig {
    pub include_labels: bool,
    pub include_metadata: bool,
    pub include_reason_codes: bool,
    pub output_format: DatasetOutputFormat,
}

impl Default for DatasetExportConfig {
    fn default() -> Self {
        Self {
            include_labels: true,
            include_metadata: true,
            include_reason_codes: true,
            output_format: DatasetOutputFormat::InMemoryOnly,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatasetRow {
    pub row_id: String,
    pub symbol: String,
    pub timestamp_ms: u64,
    pub timeframe: Timeframe,
    pub fold_id: Option<usize>,
    pub split_kind: DatasetSplitKind,
    pub regime: Regime,
    pub data_quality_score: f64,
    pub feature_values: Vec<FeatureValue>,
    pub label_outcome: Option<TripleBarrierOutcome>,
    pub label_net_return_pct: Option<f64>,
    pub label_gross_return_pct: Option<f64>,
    pub label_bars_held: Option<usize>,
    pub label_first_hit: Option<BarrierHit>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DatasetFrame {
    pub feature_names: Vec<FeatureName>,
    pub rows: Vec<DatasetRow>,
    pub metadata: BTreeMap<String, String>,
}

impl DatasetFrame {
    pub fn to_csv_string(&self, export_config: &DatasetExportConfig) -> String {
        let mut header = vec![
            "row_id".to_string(),
            "symbol".to_string(),
            "timestamp_ms".to_string(),
            "timeframe".to_string(),
            "fold_id".to_string(),
            "split_kind".to_string(),
            "regime".to_string(),
            "data_quality_score".to_string(),
        ];
        header.extend(
            self.feature_names
                .iter()
                .map(|feature| feature.as_str().to_string()),
        );
        if export_config.include_labels {
            header.extend([
                "label_outcome".to_string(),
                "label_net_return_pct".to_string(),
                "label_gross_return_pct".to_string(),
                "label_bars_held".to_string(),
                "label_first_hit".to_string(),
            ]);
        }
        if export_config.include_reason_codes {
            header.push("reason_codes".to_string());
        }

        let mut lines = vec![header.join(",")];
        for row in &self.rows {
            let mut fields = vec![
                row.row_id.clone(),
                row.symbol.clone(),
                row.timestamp_ms.to_string(),
                format!("{:?}", row.timeframe),
                row.fold_id
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                format!("{:?}", row.split_kind),
                format!("{:?}", row.regime),
                format_float(row.data_quality_score),
            ];
            fields.extend(row.feature_values.iter().map(format_feature_value));
            if export_config.include_labels {
                fields.extend([
                    row.label_outcome
                        .map(|value| format!("{value:?}"))
                        .unwrap_or_default(),
                    row.label_net_return_pct
                        .map(format_float)
                        .unwrap_or_default(),
                    row.label_gross_return_pct
                        .map(format_float)
                        .unwrap_or_default(),
                    row.label_bars_held
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    row.label_first_hit
                        .map(|value| format!("{value:?}"))
                        .unwrap_or_default(),
                ]);
            }
            if export_config.include_reason_codes {
                fields.push(
                    row.reason_codes
                        .iter()
                        .map(|reason| format!("{reason:?}"))
                        .collect::<Vec<_>>()
                        .join("|"),
                );
            }
            lines.push(fields.join(","));
        }
        lines.join("\n")
    }

    pub fn to_jsonl_string(&self) -> Option<String> {
        None
    }
}

pub fn build_dataset_frame(
    series: &CandleSeries,
    split: &WalkForwardSplit,
    feature_engine: &FeatureEngine,
    regime_classifier: &RegimeClassifier,
    barrier_config: TripleBarrierConfig,
    export_config: &DatasetExportConfig,
) -> DatasetFrame {
    let feature_names = feature_engine.feature_names();
    let mut rows = Vec::new();

    for fold in &split.folds {
        for index in indices_for_fold(fold) {
            let Some((split_kind, split_end_index, mut reason_codes)) =
                split_assignment(fold, index, barrier_config.horizon_bars)
            else {
                continue;
            };
            let feature_vector = feature_engine.build_at(series, index);
            let lookback_bars = feature_engine
                .config
                .min_required_bars
                .max(20)
                .saturating_sub(1);
            let lookback = series
                .lookback_window(index, lookback_bars)
                .unwrap_or(&series.candles[..=index]);
            let regime = regime_classifier.classify(&feature_vector, lookback).regime;
            reason_codes.extend(feature_vector.reason_codes.iter().cloned());

            let label_result = if export_config.include_labels
                && matches!(
                    split_kind,
                    DatasetSplitKind::Train | DatasetSplitKind::Validation | DatasetSplitKind::Test
                )
                && !row_is_unsafe(index, split_end_index, barrier_config.horizon_bars)
            {
                Some(evaluate_triple_barrier(
                    series,
                    index,
                    series.candles[index].close,
                    barrier_config,
                ))
            } else {
                None
            };

            rows.push(DatasetRow {
                row_id: dataset_row_id(
                    &series.symbol,
                    series.candles[index].timestamp_ms,
                    series.timeframe,
                    fold.fold_id,
                    split_kind,
                ),
                symbol: series.symbol.clone(),
                timestamp_ms: series.candles[index].timestamp_ms,
                timeframe: series.timeframe,
                fold_id: Some(fold.fold_id),
                split_kind,
                regime,
                data_quality_score: feature_vector.data_quality_score,
                feature_values: feature_vector.values,
                label_outcome: label_result.as_ref().map(|result| result.outcome),
                label_net_return_pct: label_result.as_ref().map(|result| result.net_return_pct),
                label_gross_return_pct: label_result.as_ref().map(|result| result.gross_return_pct),
                label_bars_held: label_result.as_ref().map(|result| result.bars_held),
                label_first_hit: label_result.as_ref().map(|result| result.first_hit),
                reason_codes,
            });
        }
    }

    DatasetFrame {
        feature_names,
        rows,
        metadata: BTreeMap::from([
            ("engine".to_string(), "walk_forward_dataset_v0".to_string()),
            (
                "format".to_string(),
                format!("{:?}", export_config.output_format),
            ),
            ("jsonl".to_string(), "deferred".to_string()),
        ]),
    }
}

pub fn dataset_row_id(
    symbol: &str,
    timestamp_ms: u64,
    timeframe: Timeframe,
    fold_id: usize,
    split_kind: DatasetSplitKind,
) -> String {
    let material = format!("{symbol}:{timestamp_ms}:{timeframe:?}:{fold_id}:{split_kind:?}");
    format!("row-{:#016x}", stable_hash(&material))
}

fn indices_for_fold(fold: &WalkForwardFold) -> Vec<usize> {
    let mut indices = (fold.train_start_index..=fold.train_end_index).collect::<Vec<_>>();
    if let (Some(start), Some(end)) = (fold.validation_start_index, fold.validation_end_index) {
        indices.extend(start..=end);
    }
    if let (Some(start), Some(end)) = (fold.embargo_start_index, fold.embargo_end_index) {
        indices.extend(start..=end);
    }
    indices.extend(fold.test_start_index..=fold.test_end_index);
    indices.sort_unstable();
    indices
}

fn split_assignment(
    fold: &WalkForwardFold,
    index: usize,
    horizon_bars: usize,
) -> Option<(DatasetSplitKind, usize, Vec<ReasonCode>)> {
    if (fold.train_start_index..=fold.train_end_index).contains(&index) {
        if row_is_unsafe(index, fold.train_end_index, horizon_bars) {
            return Some((
                DatasetSplitKind::Unsafe,
                fold.train_end_index,
                vec![ReasonCode::UnsafeLabelBoundary],
            ));
        }
        return Some((
            DatasetSplitKind::Train,
            fold.train_end_index,
            vec![ReasonCode::WalkForwardFoldGenerated],
        ));
    }
    if let (Some(start), Some(end)) = (fold.validation_start_index, fold.validation_end_index) {
        if (start..=end).contains(&index) {
            if row_is_unsafe(index, end, horizon_bars) {
                return Some((
                    DatasetSplitKind::Unsafe,
                    end,
                    vec![ReasonCode::UnsafeLabelBoundary],
                ));
            }
            return Some((
                DatasetSplitKind::Validation,
                end,
                vec![ReasonCode::WalkForwardFoldGenerated],
            ));
        }
    }
    if let (Some(start), Some(end)) = (fold.embargo_start_index, fold.embargo_end_index) {
        if (start..=end).contains(&index) {
            return Some((
                DatasetSplitKind::Embargo,
                end,
                vec![ReasonCode::EmbargoApplied],
            ));
        }
    }
    if (fold.test_start_index..=fold.test_end_index).contains(&index) {
        if row_is_unsafe(index, fold.test_end_index, horizon_bars) {
            return Some((
                DatasetSplitKind::Unsafe,
                fold.test_end_index,
                vec![ReasonCode::UnsafeLabelBoundary],
            ));
        }
        return Some((
            DatasetSplitKind::Test,
            fold.test_end_index,
            vec![ReasonCode::WalkForwardFoldGenerated],
        ));
    }
    None
}

fn format_feature_value(value: &FeatureValue) -> String {
    match value {
        FeatureValue::Value(number) => format_float(*number),
        FeatureValue::Missing => "MISSING".to_string(),
    }
}

fn format_float(value: f64) -> String {
    format!("{value:.8}")
}
