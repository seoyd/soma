use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::source_benchmark::SourceBenchmarkSummary;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceModelUsefulnessComparison {
    #[serde(default)]
    pub official_status: Option<String>,
    #[serde(default)]
    pub yfinance_status: Option<String>,
    pub official_useful_candidate: bool,
    pub yfinance_useful_candidate: bool,
    pub can_generalize_from_yfinance_to_official: bool,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_source_model_usefulness_comparison(
    official: Option<&SourceBenchmarkSummary>,
    yfinance: Option<&SourceBenchmarkSummary>,
    low_mismatch: bool,
    calibration_consistent: bool,
    risk_consistent: bool,
) -> SourceModelUsefulnessComparison {
    let official_useful_candidate = official.is_some_and(|summary| summary.useful_candidate);
    let yfinance_useful_candidate = yfinance.is_some_and(|summary| summary.useful_candidate);
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    if yfinance_useful_candidate && !official_useful_candidate {
        warnings.push(
            "yfinance useful-candidate result does not imply official useful-candidate status"
                .to_string(),
        );
    }
    if !low_mismatch {
        blockers.push("source mismatch is too high for cross-source generalization".to_string());
    }
    if !calibration_consistent {
        blockers.push("calibration comparison is inconsistent across sources".to_string());
    }
    if !risk_consistent {
        blockers.push("risk behavior comparison is inconsistent across sources".to_string());
    }
    if official.is_none() && yfinance.is_some() {
        warnings.push(
            "official benchmark summary is missing; yfinance remains research-only".to_string(),
        );
    }
    let can_generalize_from_yfinance_to_official =
        official_useful_candidate && yfinance_useful_candidate && blockers.is_empty();
    SourceModelUsefulnessComparison {
        official_status: official.and_then(|summary| summary.status_label.clone()),
        yfinance_status: yfinance.and_then(|summary| summary.status_label.clone()),
        official_useful_candidate,
        yfinance_useful_candidate,
        can_generalize_from_yfinance_to_official,
        blockers,
        warnings,
        reason_codes: vec![ReasonCode::SourceUsefulnessCompared],
    }
}
