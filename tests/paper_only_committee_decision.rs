mod support;

use soma_zero::{PaperOnlyCommitteeDecisionKind, PaperOnlyCommitteeDecisionRecord};

#[test]
fn paper_only_decision_variants_never_enable_execution() {
    for decision in [
        PaperOnlyCommitteeDecisionKind::WatchCandidate,
        PaperOnlyCommitteeDecisionKind::PaperApproved,
        PaperOnlyCommitteeDecisionKind::PaperRejected,
        PaperOnlyCommitteeDecisionKind::NoTrade,
        PaperOnlyCommitteeDecisionKind::RiskDenied,
        PaperOnlyCommitteeDecisionKind::NeedMoreEvidence,
    ] {
        let record = PaperOnlyCommitteeDecisionRecord::new("decision", "session", decision, None);
        assert!(record.paper_only);
        assert!(!record.broker_execution_allowed);
        assert!(!record.live_execution_allowed);
        assert_eq!(record.final_decision, decision);
    }
}
