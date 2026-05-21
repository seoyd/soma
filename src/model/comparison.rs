use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::eval::{TradeMetrics, WalkForwardReport};

use super::calibration::CalibrationReport;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelComparisonReport {
    pub baseline_model_id: String,
    pub external_model_id: String,
    pub fold_id: Option<usize>,
    pub baseline_metrics: TradeMetrics,
    pub external_metrics: TradeMetrics,
    pub delta_net_return_pct: f64,
    pub delta_profit_factor: f64,
    pub delta_max_drawdown_pct: f64,
    pub delta_no_trade_value: f64,
    pub delta_risk_denied_avoided_loss: f64,
    pub external_better: bool,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn compare_walk_forward_reports(
    baseline_model_id: &str,
    external_model_id: &str,
    baseline_report: &WalkForwardReport,
    external_report: &WalkForwardReport,
    baseline_calibration: Option<&CalibrationReport>,
    external_calibration: Option<&CalibrationReport>,
    fold_id: Option<usize>,
) -> ModelComparisonReport {
    let (baseline_metrics, baseline_no_trade_value, baseline_risk_value) =
        select_metrics(baseline_report, fold_id);
    let (external_metrics, external_no_trade_value, external_risk_value) =
        select_metrics(external_report, fold_id);

    let delta_net_return_pct = external_metrics.net_return_pct - baseline_metrics.net_return_pct;
    let delta_profit_factor = external_metrics.profit_factor.unwrap_or(0.0)
        - baseline_metrics.profit_factor.unwrap_or(0.0);
    let delta_max_drawdown_pct =
        external_metrics.max_drawdown_pct - baseline_metrics.max_drawdown_pct;
    let delta_no_trade_value = external_no_trade_value - baseline_no_trade_value;
    let delta_risk_denied_avoided_loss = external_risk_value - baseline_risk_value;

    let calibration_ok = match (baseline_calibration, external_calibration) {
        (Some(baseline), Some(external)) => external.brier_score <= baseline.brier_score + 0.02,
        _ => true,
    };
    let enough_samples = external_metrics.total_trades >= baseline_metrics.total_trades.min(3);
    let external_better = delta_net_return_pct > 0.0
        && delta_max_drawdown_pct <= 0.02
        && calibration_ok
        && enough_samples;

    let mut reason_codes = Vec::new();
    if external_better {
        reason_codes.push(ReasonCode::ExternalModelBetter);
    } else {
        reason_codes.push(ReasonCode::ComparisonNotConclusive);
    }

    ModelComparisonReport {
        baseline_model_id: baseline_model_id.to_string(),
        external_model_id: external_model_id.to_string(),
        fold_id,
        baseline_metrics,
        external_metrics,
        delta_net_return_pct,
        delta_profit_factor,
        delta_max_drawdown_pct,
        delta_no_trade_value,
        delta_risk_denied_avoided_loss,
        external_better,
        reason_codes,
    }
}

fn select_metrics(report: &WalkForwardReport, fold_id: Option<usize>) -> (TradeMetrics, f64, f64) {
    if let Some(fold_id) = fold_id {
        if let Some(fold) = report.folds.iter().find(|fold| fold.fold_id == fold_id) {
            return (
                fold.test_trade_metrics.clone(),
                fold.test_no_trade_metrics.net_silence_value,
                fold.test_risk_metrics.defensive_value,
            );
        }
    }
    (
        report.aggregate_metrics.trade_metrics.clone(),
        report.aggregate_metrics.no_trade_metrics.net_silence_value,
        report.aggregate_metrics.risk_metrics.defensive_value,
    )
}
