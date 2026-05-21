use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::campaign::CampaignAggregate;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CampaignRegression {
    DataQualityRegression,
    DrawdownRegression,
    CalibrationRegression,
    RiskGovernorRegression,
    SignalRegression,
    CoverageRegression,
    PersonaRedundancyRegression,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CampaignImprovement {
    MoreUsableData,
    BetterCalibration,
    BetterRiskDefense,
    BetterNetReturn,
    LowerDrawdown,
    BetterRegimeCoverage,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CampaignDiffMetricDeltas {
    pub delta_passed_runs: isize,
    pub delta_usable_dataset_count: isize,
    pub delta_outcome_records: isize,
    pub delta_avg_net_return_pct: f64,
    pub delta_worst_drawdown_pct: f64,
    pub delta_avg_calibration_brier: f64,
    pub delta_data_quality_score: f64,
    pub delta_risk_defensive_value: f64,
    pub delta_denial_rate: f64,
    pub delta_no_trade_rate: f64,
    pub delta_persona_redundancy_warnings: isize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CampaignDiffReport {
    pub current_campaign_id: String,
    pub previous_campaign_id: Option<String>,
    pub comparable: bool,
    pub metric_deltas: CampaignDiffMetricDeltas,
    pub regressions: Vec<CampaignRegression>,
    pub improvements: Vec<CampaignImprovement>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_campaign_diff_report(
    current: &CampaignAggregate,
    previous: Option<&CampaignAggregate>,
    previous_campaign_id: Option<&str>,
) -> CampaignDiffReport {
    let Some(previous) = previous else {
        return CampaignDiffReport {
            current_campaign_id: current.campaign_id.clone(),
            previous_campaign_id: previous_campaign_id.map(str::to_string),
            comparable: false,
            metric_deltas: CampaignDiffMetricDeltas::default(),
            regressions: Vec::new(),
            improvements: Vec::new(),
            reason_codes: vec![
                ReasonCode::DeterministicPath,
                ReasonCode::CampaignDiffUnavailable,
            ],
        };
    };
    let current_denial_rate = safe_ratio(
        current.total_denials as f64,
        current.total_outcome_records as f64,
    );
    let previous_denial_rate = safe_ratio(
        previous.total_denials as f64,
        previous.total_outcome_records as f64,
    );
    let current_no_trade_rate = safe_ratio(
        current.total_no_trades as f64,
        current.total_outcome_records as f64,
    );
    let previous_no_trade_rate = safe_ratio(
        previous.total_no_trades as f64,
        previous.total_outcome_records as f64,
    );
    let metric_deltas = CampaignDiffMetricDeltas {
        delta_passed_runs: current.passed_runs as isize - previous.passed_runs as isize,
        delta_usable_dataset_count: current.usable_dataset_count as isize
            - previous.usable_dataset_count as isize,
        delta_outcome_records: current.total_outcome_records as isize
            - previous.total_outcome_records as isize,
        delta_avg_net_return_pct: current.average_net_return_pct - previous.average_net_return_pct,
        delta_worst_drawdown_pct: current.worst_max_drawdown_pct - previous.worst_max_drawdown_pct,
        delta_avg_calibration_brier: current.average_calibration_brier.unwrap_or(0.0)
            - previous.average_calibration_brier.unwrap_or(0.0),
        delta_data_quality_score: current.average_data_quality_score
            - previous.average_data_quality_score,
        delta_risk_defensive_value: current.risk_defensive_value_total
            - previous.risk_defensive_value_total,
        delta_denial_rate: current_denial_rate - previous_denial_rate,
        delta_no_trade_rate: current_no_trade_rate - previous_no_trade_rate,
        delta_persona_redundancy_warnings: current.persona_redundancy_warning_count as isize
            - previous.persona_redundancy_warning_count as isize,
    };
    let mut regressions = Vec::new();
    if metric_deltas.delta_data_quality_score < 0.0 {
        regressions.push(CampaignRegression::DataQualityRegression);
    }
    if metric_deltas.delta_worst_drawdown_pct > 0.0 {
        regressions.push(CampaignRegression::DrawdownRegression);
    }
    if metric_deltas.delta_avg_calibration_brier > 0.0 {
        regressions.push(CampaignRegression::CalibrationRegression);
    }
    if metric_deltas.delta_denial_rate > 0.10 || metric_deltas.delta_no_trade_rate > 0.10 {
        regressions.push(CampaignRegression::RiskGovernorRegression);
    }
    if metric_deltas.delta_avg_net_return_pct < 0.0 {
        regressions.push(CampaignRegression::SignalRegression);
    }
    if metric_deltas.delta_usable_dataset_count < 0
        || metric_deltas.delta_outcome_records < 0
        || current.regime_coverage_count < previous.regime_coverage_count
    {
        regressions.push(CampaignRegression::CoverageRegression);
    }
    if metric_deltas.delta_persona_redundancy_warnings > 0 {
        regressions.push(CampaignRegression::PersonaRedundancyRegression);
    }
    let mut improvements = Vec::new();
    if metric_deltas.delta_usable_dataset_count > 0
        && !regressions.contains(&CampaignRegression::DataQualityRegression)
    {
        improvements.push(CampaignImprovement::MoreUsableData);
    }
    if metric_deltas.delta_avg_calibration_brier < 0.0 {
        improvements.push(CampaignImprovement::BetterCalibration);
    }
    if metric_deltas.delta_risk_defensive_value > 0.0 {
        improvements.push(CampaignImprovement::BetterRiskDefense);
    }
    if metric_deltas.delta_worst_drawdown_pct < 0.0 {
        improvements.push(CampaignImprovement::LowerDrawdown);
    }
    if current.regime_coverage_count > previous.regime_coverage_count {
        improvements.push(CampaignImprovement::BetterRegimeCoverage);
    }
    if metric_deltas.delta_avg_net_return_pct > 0.0
        && !regressions.contains(&CampaignRegression::DrawdownRegression)
        && !regressions.contains(&CampaignRegression::CalibrationRegression)
        && !regressions.contains(&CampaignRegression::DataQualityRegression)
    {
        improvements.push(CampaignImprovement::BetterNetReturn);
    }
    let mut reason_codes = vec![ReasonCode::DeterministicPath];
    if !regressions.is_empty() {
        reason_codes.push(ReasonCode::RegressionDetected);
    }
    CampaignDiffReport {
        current_campaign_id: current.campaign_id.clone(),
        previous_campaign_id: Some(
            previous_campaign_id
                .unwrap_or(&previous.campaign_id)
                .to_string(),
        ),
        comparable: true,
        metric_deltas,
        regressions,
        improvements,
        reason_codes,
    }
}

fn safe_ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator <= 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}
