mod support;

use soma_zero::CommitteeProposalAction;
use support::sprint98_support::run_sprint98;

#[test]
fn member_proposals_cover_core_paper_only_actions() {
    let bundle = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "committee-member-proposals",
    );
    let actions = bundle
        .ai_committee_member_proposals
        .iter()
        .map(|proposal| proposal.proposed_action)
        .collect::<Vec<_>>();
    for action in [
        CommitteeProposalAction::EnterLong,
        CommitteeProposalAction::Wait,
        CommitteeProposalAction::NoTrade,
        CommitteeProposalAction::RiskDeny,
        CommitteeProposalAction::RequestMoreEvidence,
    ] {
        assert!(actions.contains(&action), "missing {action:?}");
    }
    assert!(
        bundle
            .ai_committee_member_proposals
            .iter()
            .any(|proposal| proposal.proposed_entry_timing.is_some())
    );
    assert!(
        bundle
            .ai_committee_member_proposals
            .iter()
            .all(|proposal| (0.0..=1.0).contains(&proposal.confidence))
    );
    assert!(
        bundle
            .ai_committee_member_proposals
            .iter()
            .all(|proposal| !proposal.evidence_refs.is_empty())
    );
}
