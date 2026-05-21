use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RiskGovernorValueInputs {
    pub total_decisions: usize,
    pub approved_count: usize,
    pub reduced_count: usize,
    pub no_trade_count: usize,
    pub denied_count: usize,
    pub emergency_stop_count: usize,
    pub cooldown_count: usize,
    pub avoided_loss_total: f64,
    pub missed_gain_total: f64,
    pub risk_denied_counterfactual_count: usize,
    pub hard_veto_count: usize,
    pub soft_threshold_denial_count: usize,
    #[serde(default)]
    pub underblocking_suspected: bool,
    #[serde(default)]
    pub evidence_weak: bool,
    #[serde(default)]
    pub diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskGovernorValueStatus {
    RiskDefensiveValuePositive,
    RiskOverBlockingSuspected,
    RiskUnderBlockingSuspected,
    RiskBalanced,
    RiskDominantBecauseEvidenceWeak,
    InsufficientRiskCounterfactuals,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskGovernorValueReport {
    pub total_decisions: usize,
    pub approved_count: usize,
    pub reduced_count: usize,
    pub no_trade_count: usize,
    pub denied_count: usize,
    pub emergency_stop_count: usize,
    pub cooldown_count: usize,
    pub denial_rate: f64,
    pub approval_rate: f64,
    pub avoided_loss_total: f64,
    pub missed_gain_total: f64,
    pub risk_denied_counterfactual_count: usize,
    pub overblocking_suspected: bool,
    pub underblocking_suspected: bool,
    pub hard_veto_count: usize,
    pub soft_threshold_denial_count: usize,
    pub status: RiskGovernorValueStatus,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_risk_governor_value_report(
    inputs: &RiskGovernorValueInputs,
) -> RiskGovernorValueReport {
    let total = inputs.total_decisions.max(1) as f64;
    let denial_rate = inputs.denied_count as f64 / total;
    let approval_rate = (inputs.approved_count + inputs.reduced_count) as f64 / total;
    let overblocking_suspected = inputs.risk_denied_counterfactual_count > 0
        && inputs.soft_threshold_denial_count >= 2
        && inputs.missed_gain_total > inputs.avoided_loss_total;
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();

    let status = if inputs.diagnostic_only {
        warnings.push("risk-governor evidence remains diagnostic-only".to_string());
        RiskGovernorValueStatus::DiagnosticOnly
    } else if inputs.risk_denied_counterfactual_count == 0 {
        blockers.push("risk-denied counterfactuals are unavailable".to_string());
        if inputs.evidence_weak {
            RiskGovernorValueStatus::RiskDominantBecauseEvidenceWeak
        } else {
            RiskGovernorValueStatus::InsufficientRiskCounterfactuals
        }
    } else if overblocking_suspected {
        warnings.push(
            "soft-threshold denials appear to block more gain than loss avoidance".to_string(),
        );
        RiskGovernorValueStatus::RiskOverBlockingSuspected
    } else if inputs.underblocking_suspected {
        warnings.push("risk denials may be too sparse for observed losses".to_string());
        RiskGovernorValueStatus::RiskUnderBlockingSuspected
    } else if inputs.evidence_weak
        && inputs.denied_count >= inputs.approved_count + inputs.reduced_count
    {
        warnings.push("evidence remains weak, so heavy denial is not treated as a bug".to_string());
        RiskGovernorValueStatus::RiskDominantBecauseEvidenceWeak
    } else if inputs.avoided_loss_total > inputs.missed_gain_total {
        RiskGovernorValueStatus::RiskDefensiveValuePositive
    } else {
        RiskGovernorValueStatus::RiskBalanced
    };

    let mut reason_codes = inputs.reason_codes.clone();
    reason_codes.push(ReasonCode::RiskGovernorValueReportBuilt);
    if overblocking_suspected || inputs.underblocking_suspected {
        reason_codes.push(ReasonCode::RiskDeniedCounterfactual);
    }

    RiskGovernorValueReport {
        total_decisions: inputs.total_decisions,
        approved_count: inputs.approved_count,
        reduced_count: inputs.reduced_count,
        no_trade_count: inputs.no_trade_count,
        denied_count: inputs.denied_count,
        emergency_stop_count: inputs.emergency_stop_count,
        cooldown_count: inputs.cooldown_count,
        denial_rate,
        approval_rate,
        avoided_loss_total: inputs.avoided_loss_total,
        missed_gain_total: inputs.missed_gain_total,
        risk_denied_counterfactual_count: inputs.risk_denied_counterfactual_count,
        overblocking_suspected,
        underblocking_suspected: inputs.underblocking_suspected,
        hard_veto_count: inputs.hard_veto_count,
        soft_threshold_denial_count: inputs.soft_threshold_denial_count,
        status,
        blockers,
        warnings,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

impl RiskGovernorValueReport {
    pub fn to_text(&self) -> String {
        [
            format!("total_decisions={}", self.total_decisions),
            format!("approved_count={}", self.approved_count),
            format!("reduced_count={}", self.reduced_count),
            format!("no_trade_count={}", self.no_trade_count),
            format!("denied_count={}", self.denied_count),
            format!("emergency_stop_count={}", self.emergency_stop_count),
            format!("cooldown_count={}", self.cooldown_count),
            format!("denial_rate={:.6}", self.denial_rate),
            format!("approval_rate={:.6}", self.approval_rate),
            format!("avoided_loss_total={:.6}", self.avoided_loss_total),
            format!("missed_gain_total={:.6}", self.missed_gain_total),
            format!(
                "risk_denied_counterfactual_count={}",
                self.risk_denied_counterfactual_count
            ),
            format!("overblocking_suspected={}", self.overblocking_suspected),
            format!("underblocking_suspected={}", self.underblocking_suspected),
            format!("hard_veto_count={}", self.hard_veto_count),
            format!(
                "soft_threshold_denial_count={}",
                self.soft_threshold_denial_count
            ),
            format!("status={:?}", self.status),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}
