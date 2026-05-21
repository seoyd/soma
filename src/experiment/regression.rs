use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::campaign::CampaignAggregate;
use super::diff::{CampaignDiffReport, CampaignRegression};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegressionGuardConfig {
    pub max_drawdown_regression_pct: f64,
    pub max_calibration_regression: f64,
    pub min_data_quality_delta: f64,
    pub max_denial_rate_change: f64,
    pub max_no_trade_rate_change: f64,
    pub require_no_new_safety_regressions: bool,
}

impl Default for RegressionGuardConfig {
    fn default() -> Self {
        Self {
            max_drawdown_regression_pct: 0.02,
            max_calibration_regression: 0.02,
            min_data_quality_delta: -0.02,
            max_denial_rate_change: 0.15,
            max_no_trade_rate_change: 0.15,
            require_no_new_safety_regressions: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegressionGuardResult {
    pub passed: bool,
    pub regressions: Vec<CampaignRegression>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn evaluate_regression_guard(
    config: &RegressionGuardConfig,
    current: &CampaignAggregate,
    previous: Option<&CampaignAggregate>,
    diff: &CampaignDiffReport,
) -> RegressionGuardResult {
    let mut regressions = diff.regressions.clone();
    let mut warnings = Vec::new();
    let mut reason_codes = vec![ReasonCode::DeterministicPath];
    if !diff.comparable {
        warnings
            .push("previous campaign/report is unavailable for regression comparison".to_string());
        reason_codes.push(ReasonCode::CampaignDiffUnavailable);
        return RegressionGuardResult {
            passed: false,
            regressions,
            warnings,
            reason_codes,
        };
    }
    if current.total_outcome_records < 20
        || previous.is_some_and(|previous| previous.total_outcome_records < 20)
    {
        warnings.push("insufficient outcome records for a strong regression decision".to_string());
        reason_codes.push(ReasonCode::ComparisonNotConclusive);
        return RegressionGuardResult {
            passed: false,
            regressions,
            warnings,
            reason_codes,
        };
    }
    if diff.metric_deltas.delta_worst_drawdown_pct > config.max_drawdown_regression_pct
        && !regressions.contains(&CampaignRegression::DrawdownRegression)
    {
        regressions.push(CampaignRegression::DrawdownRegression);
    }
    if diff.metric_deltas.delta_avg_calibration_brier > config.max_calibration_regression
        && !regressions.contains(&CampaignRegression::CalibrationRegression)
    {
        regressions.push(CampaignRegression::CalibrationRegression);
    }
    if diff.metric_deltas.delta_data_quality_score < config.min_data_quality_delta
        && !regressions.contains(&CampaignRegression::DataQualityRegression)
    {
        regressions.push(CampaignRegression::DataQualityRegression);
    }
    if diff.metric_deltas.delta_denial_rate > config.max_denial_rate_change {
        warnings.push("denial rate changed materially".to_string());
    }
    if diff.metric_deltas.delta_no_trade_rate > config.max_no_trade_rate_change {
        warnings.push("no-trade rate changed materially".to_string());
    }
    let passed = if config.require_no_new_safety_regressions {
        regressions.is_empty()
    } else {
        !regressions.contains(&CampaignRegression::DrawdownRegression)
            && !regressions.contains(&CampaignRegression::CalibrationRegression)
            && !regressions.contains(&CampaignRegression::DataQualityRegression)
    };
    if !regressions.is_empty() {
        reason_codes.push(ReasonCode::RegressionDetected);
    }
    RegressionGuardResult {
        passed,
        regressions,
        warnings,
        reason_codes,
    }
}
