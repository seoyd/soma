mod support;

use soma_zero::{
    CommitteeConsensusState, CommitteeDebateSession, CommitteeDebateStance, CommitteeDebateTrigger,
    CommitteeDebateTriggerReason, CommitteeDebateTriggerStatus, CommitteeMemberDebateTurn,
    CommitteeMemberDebateTurnStatus,
};
use support::sprint98_support::run_sprint98;

#[test]
fn debate_session_records_members_turns_and_consensus() {
    let bundle = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "committee-debate-session",
    );
    assert!(
        !bundle
            .committee_debate_session
            .participating_members
            .is_empty()
    );
    assert_eq!(
        bundle.committee_debate_session.member_turns,
        bundle.committee_member_debate_turns
    );
    let stances = bundle
        .committee_member_debate_turns
        .iter()
        .map(|turn| turn.stance)
        .collect::<Vec<_>>();
    assert!(stances.contains(&CommitteeDebateStance::SupportEntry));
    assert!(stances.contains(&CommitteeDebateStance::OpposeEntry));
    assert!(stances.contains(&CommitteeDebateStance::WaitForConfirmation));
    assert!(stances.contains(&CommitteeDebateStance::RequestMoreEvidence));
    assert_eq!(
        bundle.committee_debate_session.consensus_state,
        CommitteeConsensusState::NeedMoreEvidence
    );
    let trigger = CommitteeDebateTrigger {
        trigger_id: "risk-trigger".to_string(),
        triggering_member_id: "risk-defender".to_string(),
        triggering_proposal_id: "risk-proposal".to_string(),
        trigger_reason: CommitteeDebateTriggerReason::RiskDenyProposed,
        debate_required: true,
        trigger_status: CommitteeDebateTriggerStatus::DebateTriggered,
        reason_codes: vec![],
    };
    let risk_session = CommitteeDebateSession::new(
        "risk-session",
        trigger,
        vec![CommitteeMemberDebateTurn {
            turn_id: "turn".to_string(),
            session_id: "risk-session".to_string(),
            member_id: "risk-defender".to_string(),
            stance: CommitteeDebateStance::DemandRiskDeny,
            argument_summary: "deny".to_string(),
            evidence_refs: vec!["risk".to_string()],
            counterarguments: vec![],
            confidence: 0.9,
            turn_status: CommitteeMemberDebateTurnStatus::TurnReady,
            reason_codes: vec![],
        }],
    );
    assert_eq!(
        risk_session.consensus_state,
        CommitteeConsensusState::RiskDenied
    );
    assert!(bundle.paper_only_committee_decision_record.paper_only);
}
