mod support;

use soma_zero::AICommitteeMemberStatus;
use support::sprint98_support::run_sprint98;

#[test]
fn member_specs_record_style_core_permissions_and_statuses() {
    let first = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "ai-committee-member-specs",
    );
    let second = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "ai-committee-member-specs-second",
    );
    for spec in &first.ai_committee_member_specs {
        assert!(
            !spec.owned_core_refs.is_empty(),
            "{} missing core refs",
            spec.member_id
        );
        assert!(
            !spec.proposal_permissions.is_empty(),
            "{} missing proposal permissions",
            spec.member_id
        );
        assert!(
            !spec.debate_permissions.is_empty(),
            "{} missing debate permissions",
            spec.member_id
        );
        assert!(
            spec.promotion_eligible
                || matches!(spec.member_status, AICommitteeMemberStatus::RetiredMember)
        );
        assert!(spec.demotion_eligible);
    }
    assert!(
        first
            .ai_committee_member_specs
            .iter()
            .any(|spec| matches!(spec.member_status, AICommitteeMemberStatus::WatchOnlyMember))
    );
    assert_eq!(
        first.ai_committee_member_specs,
        second.ai_committee_member_specs
    );
}
