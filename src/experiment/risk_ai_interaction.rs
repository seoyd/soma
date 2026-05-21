use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::eval::WalkForwardReport;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskAiInteractionReport {
    pub model_id: String,
    pub total_signals: usize,
    pub approved_candidates: usize,
    pub denied_by_risk: usize,
    pub no_trade_by_signal: usize,
    pub no_trade_by_risk: usize,
    pub emergency_stop_count: usize,
    pub cooldown_count: usize,
    pub avoided_loss_count: usize,
    pub missed_gain_count: usize,
    pub defensive_value: f64,
    pub opportunity_cost: f64,
    pub denial_rate: f64,
    pub approval_rate: f64,
    pub reason_code_counts: Vec<(String, usize)>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl RiskAiInteractionReport {
    pub fn from_walk_forward_report(
        model_id: impl Into<String>,
        report: &WalkForwardReport,
    ) -> Self {
        let aggregate = &report.aggregate_metrics;
        let total_signals = aggregate.decision_metrics.total_decisions;
        let denied_by_risk = aggregate.decision_metrics.denied_by_risk;
        let approved_candidates = aggregate.decision_metrics.approve_candidate_count;
        let denial_rate = safe_ratio(denied_by_risk, total_signals);
        let approval_rate = safe_ratio(approved_candidates, total_signals);
        let mut warnings = Vec::new();
        if denial_rate > 0.90 && aggregate.risk_metrics.defensive_value <= 0.0 {
            warnings.push(
                "risk governor denies most signals without measurable defensive value".to_string(),
            );
        }
        if denial_rate < 0.05 && aggregate.trade_metrics.max_drawdown_pct > 0.20 {
            warnings.push(
                "risk governor denial rate is very low despite elevated drawdown".to_string(),
            );
        }

        Self {
            model_id: model_id.into(),
            total_signals,
            approved_candidates,
            denied_by_risk,
            no_trade_by_signal: aggregate.decision_metrics.no_trade,
            no_trade_by_risk: aggregate.decision_metrics.denied_by_risk,
            emergency_stop_count: aggregate.risk_metrics.emergency_stop_count,
            cooldown_count: aggregate.risk_metrics.cooldown_count,
            avoided_loss_count: aggregate.risk_metrics.avoided_loss_count,
            missed_gain_count: aggregate.risk_metrics.missed_gain_count,
            defensive_value: aggregate.risk_metrics.defensive_value,
            opportunity_cost: aggregate.risk_metrics.opportunity_cost,
            denial_rate,
            approval_rate,
            reason_code_counts: aggregate
                .decision_metrics
                .reason_code_counts
                .iter()
                .map(|(key, value)| (key.clone(), *value))
                .collect(),
            warnings,
            reason_codes: vec![ReasonCode::RiskAiInteractionBuilt],
        }
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("model_id={}", self.model_id),
            format!("total_signals={}", self.total_signals),
            format!("approved_candidates={}", self.approved_candidates),
            format!("denied_by_risk={}", self.denied_by_risk),
            format!("no_trade_by_signal={}", self.no_trade_by_signal),
            format!("no_trade_by_risk={}", self.no_trade_by_risk),
            format!("emergency_stop_count={}", self.emergency_stop_count),
            format!("cooldown_count={}", self.cooldown_count),
            format!("avoided_loss_count={}", self.avoided_loss_count),
            format!("missed_gain_count={}", self.missed_gain_count),
            format!("defensive_value={:.8}", self.defensive_value),
            format!("opportunity_cost={:.8}", self.opportunity_cost),
            format!("denial_rate={:.8}", self.denial_rate),
            format!("approval_rate={:.8}", self.approval_rate),
            format!("warnings={}", self.warnings.join(" | ")),
        ];
        for (reason, count) in &self.reason_code_counts {
            lines.push(format!("reason_code_count={reason}:{count}"));
        }
        lines.join("\n")
    }
}

fn safe_ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}
