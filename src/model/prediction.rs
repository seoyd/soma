use serde::{Deserialize, Serialize};

use crate::backtest::Timeframe;
use crate::core::ReasonCode;
use crate::eval::dataset::DatasetSplitKind;

use super::meta::ModelArtifactMeta;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionInputFormat {
    Csv,
    InMemoryOnly,
    JsonLines,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PredictionImportConfig {
    pub require_feature_schema_match: bool,
    pub require_row_alignment: bool,
    pub min_confidence: Option<f64>,
    pub max_missing_rows: usize,
    pub input_format: PredictionInputFormat,
}

impl Default for PredictionImportConfig {
    fn default() -> Self {
        Self {
            require_feature_schema_match: true,
            require_row_alignment: true,
            min_confidence: None,
            max_missing_rows: 0,
            input_format: PredictionInputFormat::InMemoryOnly,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PredictionValidationResult {
    pub valid: bool,
    pub row_count: usize,
    pub missing_row_count: usize,
    pub extra_row_count: usize,
    pub schema_match: bool,
    pub feature_schema_hash_match: bool,
    pub invalid_probability_count: usize,
    pub nan_or_inf_count: usize,
    pub timestamp_mismatch_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for PredictionValidationResult {
    fn default() -> Self {
        Self {
            valid: false,
            row_count: 0,
            missing_row_count: 0,
            extra_row_count: 0,
            schema_match: false,
            feature_schema_hash_match: false,
            invalid_probability_count: 0,
            nan_or_inf_count: 0,
            timestamp_mismatch_count: 0,
            reason_codes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PredictionRow {
    pub row_id: String,
    pub symbol: String,
    pub timestamp_ms: u64,
    pub timeframe: Timeframe,
    pub fold_id: Option<usize>,
    pub split_kind: Option<DatasetSplitKind>,
    pub model_id: String,
    pub p_win: f64,
    pub p_stop: f64,
    pub expected_return: f64,
    pub expected_drawdown: f64,
    pub confidence: f64,
    pub no_trade_probability: f64,
    pub horizon_bars: u32,
    pub reason_codes: Vec<ReasonCode>,
}

impl PredictionRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        row_id: impl Into<String>,
        symbol: impl Into<String>,
        timestamp_ms: u64,
        timeframe: Timeframe,
        fold_id: Option<usize>,
        split_kind: Option<DatasetSplitKind>,
        model_id: impl Into<String>,
        p_win: f64,
        p_stop: f64,
        expected_return: f64,
        expected_drawdown: f64,
        confidence: f64,
        no_trade_probability: f64,
        horizon_bars: u32,
    ) -> Result<Self, Vec<ReasonCode>> {
        let row = Self {
            row_id: row_id.into(),
            symbol: symbol.into(),
            timestamp_ms,
            timeframe,
            fold_id,
            split_kind,
            model_id: model_id.into(),
            p_win,
            p_stop,
            expected_return,
            expected_drawdown,
            confidence,
            no_trade_probability,
            horizon_bars,
            reason_codes: Vec::new(),
        };
        let validation = row.validation_errors();
        if validation.is_empty() {
            Ok(row)
        } else {
            Err(validation)
        }
    }

    pub fn validation_errors(&self) -> Vec<ReasonCode> {
        let mut errors = Vec::new();
        if !in_unit_interval(self.p_win)
            || !in_unit_interval(self.p_stop)
            || !in_unit_interval(self.confidence)
            || !in_unit_interval(self.no_trade_probability)
        {
            errors.push(ReasonCode::InvalidProbability);
        }
        if !self.expected_return.is_finite() || !self.expected_drawdown.is_finite() {
            errors.push(ReasonCode::InvalidPrediction);
        }
        errors
    }

    pub fn is_valid(&self) -> bool {
        self.validation_errors().is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PredictionFrame {
    pub model_meta: ModelArtifactMeta,
    pub rows: Vec<PredictionRow>,
    pub schema_validation: PredictionValidationResult,
    pub reason_codes: Vec<ReasonCode>,
}

impl PredictionFrame {
    pub fn for_fold(&self, fold_id: usize) -> Self {
        let rows = self
            .rows
            .iter()
            .filter(|row| row.fold_id == Some(fold_id) || row.fold_id.is_none())
            .cloned()
            .collect::<Vec<_>>();
        Self {
            model_meta: self.model_meta.clone(),
            rows,
            schema_validation: self.schema_validation.clone(),
            reason_codes: self.reason_codes.clone(),
        }
    }

    pub fn find_by_row_id(&self, row_id: &str) -> Option<&PredictionRow> {
        self.rows.iter().find(|row| row.row_id == row_id)
    }

    pub fn find_by_key(
        &self,
        symbol: &str,
        timestamp_ms: u64,
        timeframe: Timeframe,
        fold_id: Option<usize>,
    ) -> Option<&PredictionRow> {
        self.rows.iter().find(|row| {
            row.symbol == symbol
                && row.timestamp_ms == timestamp_ms
                && row.timeframe == timeframe
                && (row.fold_id == fold_id || row.fold_id.is_none())
        })
    }
}

fn in_unit_interval(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}
