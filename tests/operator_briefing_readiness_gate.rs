#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{OperatorBriefingReadinessGateStatus, OperatorBriefingReadinessRecommendation};

#[test]
fn operator_briefing_readiness_gate_stays_blocked_until_last_gap_is_closed() {
    let bundle = support::run_offline_attachment(
        "soma_offline_evidence_attach.toml",
        "operator-briefing-readiness-gate",
    );
    let gate = bundle.operator_briefing_readiness_gate;

    assert_eq!(
        gate.gate_status,
        OperatorBriefingReadinessGateStatus::BlockedByEvidenceGap
    );
    assert_eq!(
        gate.final_recommendation,
        OperatorBriefingReadinessRecommendation::AttachMoreEvidence
    );
    assert!(gate.static_only);
    assert!(gate.paper_only);
    assert!(gate.forbidden_controls_absent);
}
