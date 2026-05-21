use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash};
use crate::data::StorageBudget;
use crate::eval::{DatasetFrame, DatasetSplitKind, LeakageGuard};
use crate::feature::FeatureEngine;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SequenceDatasetConfig {
    pub window_size: usize,
    pub stride: usize,
    pub horizon_bars: usize,
    pub feature_schema_hash: u64,
    pub label_config_summary: String,
    #[serde(default = "default_true")]
    pub include_metadata: bool,
    pub max_windows: usize,
    pub max_bytes: usize,
    #[serde(default)]
    pub storage_budget: StorageBudget,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for SequenceDatasetConfig {
    fn default() -> Self {
        Self {
            window_size: 64,
            stride: 1,
            horizon_bars: 8,
            feature_schema_hash: 0,
            label_config_summary: "triple_barrier".to_string(),
            include_metadata: true,
            max_windows: 1024,
            max_bytes: 4 * 1024 * 1024,
            storage_budget: StorageBudget::default(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SequenceRowRef {
    pub symbol: String,
    pub start_index: usize,
    pub end_index: usize,
    pub label_index: usize,
    pub timestamp_start_ms: u64,
    pub timestamp_end_ms: u64,
    pub feature_schema_hash: u64,
    pub label_outcome: Option<String>,
    pub label_net_return_pct: Option<f64>,
    pub split_kind: DatasetSplitKind,
    #[serde(default)]
    pub fold_id: Option<usize>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SequenceDatasetSpec {
    pub config: SequenceDatasetConfig,
    pub estimated_windows: usize,
    pub estimated_bytes: usize,
    pub no_lookahead_safe: bool,
    pub storage_budget_ok: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl SequenceDatasetConfig {
    pub fn validate(&self) -> Vec<ReasonCode> {
        let mut reasons = Vec::new();
        if self.window_size == 0 || self.stride == 0 || self.horizon_bars == 0 {
            reasons.push(ReasonCode::SequenceDatasetInvalid);
        }
        reasons
    }
}

impl SequenceDatasetSpec {
    pub fn from_dataset_frame(frame: &DatasetFrame, config: &SequenceDatasetConfig) -> Self {
        let reason_codes = config.validate();
        if !reason_codes.is_empty() {
            return Self {
                config: config.clone(),
                estimated_windows: 0,
                estimated_bytes: 0,
                no_lookahead_safe: false,
                storage_budget_ok: false,
                reason_codes,
            };
        }
        let refs = build_sequence_row_refs(frame, config);
        let estimated_windows = refs.len();
        let estimated_bytes = estimate_bytes(
            estimated_windows,
            frame.feature_names.len(),
            config.window_size,
            config.include_metadata,
        );
        let storage_budget_ok = estimated_bytes <= config.max_bytes
            && estimated_bytes <= config.storage_budget.max_total_bytes;
        let mut reason_codes = vec![ReasonCode::SequenceDatasetSpecBuilt];
        if !storage_budget_ok {
            reason_codes.push(ReasonCode::SequenceStorageBudgetExceeded);
        }
        Self {
            config: SequenceDatasetConfig {
                feature_schema_hash: if config.feature_schema_hash == 0 {
                    stable_hash(
                        &frame
                            .feature_names
                            .iter()
                            .map(|name| name.as_str().to_string())
                            .collect::<Vec<_>>()
                            .join("|"),
                    )
                } else {
                    config.feature_schema_hash
                },
                ..config.clone()
            },
            estimated_windows,
            estimated_bytes,
            no_lookahead_safe: refs.iter().all(|row| row.start_index <= row.end_index),
            storage_budget_ok,
            reason_codes,
        }
    }

    pub fn from_dataset_csv_path(
        path: &Path,
        config: &SequenceDatasetConfig,
    ) -> Result<Self, String> {
        let reason_codes = config.validate();
        if !reason_codes.is_empty() {
            return Ok(Self {
                config: config.clone(),
                estimated_windows: 0,
                estimated_bytes: 0,
                no_lookahead_safe: false,
                storage_budget_ok: false,
                reason_codes,
            });
        }
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let mut lines = text.lines();
        let header = lines
            .next()
            .ok_or_else(|| "dataset csv missing header".to_string())?;
        let feature_count = header
            .split(',')
            .filter(|name| is_feature_column(name.trim()))
            .count();
        let row_count = lines.count();
        let estimated_windows = estimate_window_count(row_count, config);
        let estimated_bytes = estimate_bytes(
            estimated_windows,
            feature_count,
            config.window_size,
            config.include_metadata,
        );
        let storage_budget_ok = estimated_bytes <= config.max_bytes
            && estimated_bytes <= config.storage_budget.max_total_bytes;
        let mut reason_codes = vec![ReasonCode::SequenceDatasetSpecBuilt];
        if !storage_budget_ok {
            reason_codes.push(ReasonCode::SequenceStorageBudgetExceeded);
        }
        Ok(Self {
            config: SequenceDatasetConfig {
                feature_schema_hash: if config.feature_schema_hash == 0 {
                    stable_hash(header)
                } else {
                    config.feature_schema_hash
                },
                ..config.clone()
            },
            estimated_windows,
            estimated_bytes,
            no_lookahead_safe: true,
            storage_budget_ok,
            reason_codes,
        })
    }
}

pub fn build_sequence_row_refs(
    frame: &DatasetFrame,
    config: &SequenceDatasetConfig,
) -> Vec<SequenceRowRef> {
    if !config.validate().is_empty() {
        return Vec::new();
    }
    let feature_schema_hash = if config.feature_schema_hash == 0 {
        stable_hash(
            &frame
                .feature_names
                .iter()
                .map(|name| name.as_str().to_string())
                .collect::<Vec<_>>()
                .join("|"),
        )
    } else {
        config.feature_schema_hash
    };
    frame
        .rows
        .iter()
        .enumerate()
        .filter(|(index, _)| *index + 1 >= config.window_size)
        .step_by(config.stride)
        .take(config.max_windows)
        .map(|(index, row)| SequenceRowRef {
            symbol: row.symbol.clone(),
            start_index: index + 1 - config.window_size,
            end_index: index,
            label_index: index,
            timestamp_start_ms: frame.rows[index + 1 - config.window_size].timestamp_ms,
            timestamp_end_ms: row.timestamp_ms,
            feature_schema_hash,
            label_outcome: row.label_outcome.map(|value| format!("{value:?}")),
            label_net_return_pct: row.label_net_return_pct,
            split_kind: row.split_kind,
            fold_id: row.fold_id,
            reason_codes: vec![ReasonCode::SequenceWindowEstimated],
        })
        .collect()
}

pub fn prior_window_features_unchanged(
    engine: &FeatureEngine,
    before: &crate::backtest::CandleSeries,
    after: &crate::backtest::CandleSeries,
    end_index: usize,
    window_size: usize,
) -> bool {
    if window_size == 0 || end_index + 1 < window_size {
        return false;
    }
    let start_index = end_index + 1 - window_size;
    (start_index..=end_index)
        .all(|index| LeakageGuard::feature_stable_at(engine, before, after, index))
}

fn estimate_window_count(total_rows: usize, config: &SequenceDatasetConfig) -> usize {
    if total_rows < config.window_size {
        return 0;
    }
    let raw = 1 + (total_rows - config.window_size) / config.stride;
    raw.min(config.max_windows)
}

fn estimate_bytes(
    estimated_windows: usize,
    feature_count: usize,
    window_size: usize,
    include_metadata: bool,
) -> usize {
    let feature_bytes = estimated_windows
        .saturating_mul(feature_count)
        .saturating_mul(window_size)
        .saturating_mul(8);
    let metadata_bytes = if include_metadata {
        estimated_windows.saturating_mul(64)
    } else {
        0
    };
    feature_bytes.saturating_add(metadata_bytes)
}

fn default_true() -> bool {
    true
}

fn is_feature_column(name: &str) -> bool {
    !matches!(
        name,
        "row_id"
            | "symbol"
            | "timestamp_ms"
            | "timeframe"
            | "fold_id"
            | "split_kind"
            | "regime"
            | "data_quality_score"
            | "label_outcome"
            | "label_net_return_pct"
            | "label_gross_return_pct"
            | "label_bars_held"
            | "label_first_hit"
            | "reason_codes"
    )
}
