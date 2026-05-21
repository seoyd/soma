use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, RiskDecisionKind};

use super::committee_decision::CommitteeDecisionRecord;
use super::committee_risk_bridge::{CommitteeOutcome, CommitteeRiskBridge};
use super::persona_scorer::PersonaScoringInput;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskBridgeDiagnosticStatus {
    RiskPassed,
    RiskDeniedExpected,
    RiskDeniedUnexpected,
    RiskOverBlockingSuspected,
    RiskUnderBlockingSuspected,
    EmergencyStop,
    Cooldown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskBridgeDiagnosticsReport {
    pub decision_id: String,
    pub committee_final_decision: String,
    pub risk_proposal_summary: String,
    pub risk_governor_decision: String,
    pub final_action: String,
    pub veto_applied: bool,
    pub denial_reason_codes: Vec<ReasonCode>,
    pub emergency_stop_triggered: bool,
    pub cooldown_triggered: bool,
    pub data_quality_block: bool,
    pub negative_edge_block: bool,
    pub invalid_prediction_block: bool,
    pub schema_mismatch_block: bool,
    pub diagnostic_status: RiskBridgeDiagnosticStatus,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_risk_bridge_diagnostics(
    bridge: &CommitteeRiskBridge,
    market: &crate::MarketSnapshot,
    input: &PersonaScoringInput,
    record: &CommitteeDecisionRecord,
    outcome: &CommitteeOutcome,
) -> RiskBridgeDiagnosticsReport {
    let proposal_summary = bridge
        .committee_decision_to_risk_proposal(market, input, record)
        .map(|proposal| {
            format!(
                "symbol={};qty={:.3};edge={:.6};confidence={:.6}",
                proposal.symbol,
                proposal.quantity_hint,
                proposal.expected_edge_after_cost,
                proposal.confidence
            )
        })
        .unwrap_or_else(|| "none".to_string());
    let denial_reason_codes = outcome.risk_decision.reason_codes.clone();
    let emergency_stop_triggered = outcome.risk_decision.kind == RiskDecisionKind::EmergencyStop;
    let cooldown_triggered = outcome.risk_decision.kind == RiskDecisionKind::Cooldown;
    let data_quality_block = denial_reason_codes.contains(&ReasonCode::DataQualityGateBreached)
        || denial_reason_codes.contains(&ReasonCode::DataQualityTooLow);
    let negative_edge_block = denial_reason_codes.contains(&ReasonCode::ExpectedEdgeNonPositive)
        || denial_reason_codes.contains(&ReasonCode::ExpectedEdgeBelowThreshold);
    let invalid_prediction_block = denial_reason_codes.contains(&ReasonCode::InvalidPrediction);
    let schema_mismatch_block = denial_reason_codes.contains(&ReasonCode::SchemaMismatch);
    let diagnostic_status = match outcome.risk_decision.kind {
        RiskDecisionKind::ApprovePaper => RiskBridgeDiagnosticStatus::RiskPassed,
        RiskDecisionKind::EmergencyStop => RiskBridgeDiagnosticStatus::EmergencyStop,
        RiskDecisionKind::Cooldown => RiskBridgeDiagnosticStatus::Cooldown,
        RiskDecisionKind::Deny
            if data_quality_block || negative_edge_block || schema_mismatch_block =>
        {
            RiskBridgeDiagnosticStatus::RiskDeniedExpected
        }
        RiskDecisionKind::Deny => RiskBridgeDiagnosticStatus::RiskDeniedUnexpected,
    };
    RiskBridgeDiagnosticsReport {
        decision_id: record.decision_id.clone(),
        committee_final_decision: format!("{:?}", record.final_decision),
        risk_governor_decision: format!("{:?}", outcome.risk_decision.kind),
        final_action: format!("{:?}", outcome.final_action),
        veto_applied: !matches!(outcome.risk_decision.kind, RiskDecisionKind::ApprovePaper),
        denial_reason_codes,
        emergency_stop_triggered,
        cooldown_triggered,
        data_quality_block,
        negative_edge_block,
        invalid_prediction_block,
        schema_mismatch_block,
        diagnostic_status,
        risk_proposal_summary: proposal_summary,
        reason_codes: vec![ReasonCode::RiskBridgeDiagnosticsBuilt],
    }
}

impl RiskBridgeDiagnosticsReport {
    pub fn to_text(&self) -> String {
        [
            format!("decision_id={}", self.decision_id),
            format!("committee_final_decision={}", self.committee_final_decision),
            format!("risk_governor_decision={}", self.risk_governor_decision),
            format!("final_action={}", self.final_action),
            format!("diagnostic_status={:?}", self.diagnostic_status),
            format!("risk_proposal_summary={}", self.risk_proposal_summary),
            format!(
                "denial_reason_codes={}",
                self.denial_reason_codes
                    .iter()
                    .map(|code| format!("{code:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }
}
