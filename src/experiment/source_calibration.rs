use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::source_benchmark::SourceBenchmarkSummary;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceCalibrationComparison {
    #[serde(default)]
    pub official_brier: Option<f64>,
    #[serde(default)]
    pub yfinance_brier: Option<f64>,
    #[serde(default)]
    pub official_ece: Option<f64>,
    #[serde(default)]
    pub yfinance_ece: Option<f64>,
    #[serde(default)]
    pub brier_delta: Option<f64>,
    #[serde(default)]
    pub ece_delta: Option<f64>,
    pub calibration_consistent: bool,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_source_calibration_comparison(
    official: Option<&SourceBenchmarkSummary>,
    yfinance: Option<&SourceBenchmarkSummary>,
    max_allowed_calibration_delta: f64,
) -> SourceCalibrationComparison {
    let official_brier = official.and_then(|summary| summary.avg_brier_score);
    let yfinance_brier = yfinance.and_then(|summary| summary.avg_brier_score);
    let official_ece = official.and_then(|summary| summary.avg_expected_calibration_error);
    let yfinance_ece = yfinance.and_then(|summary| summary.avg_expected_calibration_error);
    let brier_delta = official_brier
        .zip(yfinance_brier)
        .map(|(left, right)| (left - right).abs());
    let ece_delta = official_ece
        .zip(yfinance_ece)
        .map(|(left, right)| (left - right).abs());
    let mut warnings = Vec::new();
    if official.is_none() {
        warnings.push("official calibration metrics are missing".to_string());
    }
    if yfinance.is_none() {
        warnings.push("yfinance calibration metrics are missing".to_string());
    }
    let calibration_consistent = brier_delta
        .is_none_or(|delta| delta <= max_allowed_calibration_delta)
        && ece_delta.is_none_or(|delta| delta <= max_allowed_calibration_delta);
    if !calibration_consistent {
        warnings.push("calibration delta exceeds configured threshold".to_string());
    }
    SourceCalibrationComparison {
        official_brier,
        yfinance_brier,
        official_ece,
        yfinance_ece,
        brier_delta,
        ece_delta,
        calibration_consistent,
        warnings,
        reason_codes: vec![ReasonCode::SourceCalibrationCompared],
    }
}
