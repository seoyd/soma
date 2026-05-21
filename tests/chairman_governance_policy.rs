mod support;

use soma_zero::{ChairmanRuleProposalStatus, ChairmanRulebookVersionStatus};
use support::sprint98_support::run_sprint98;

#[test]
fn chairman_governance_stays_audited_and_cannot_bypass_risk() {
    let first = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "chairman-governance-policy",
    );
    let second = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "chairman-governance-policy-second",
    );
    let policy = &first.chairman_ai_governance_policy;
    assert!(!policy.can_bypass_risk_governor);
    assert!(!policy.can_promote_member_unilaterally);
    assert!(!policy.can_demote_member_unilaterally);
    assert!(!first.chairman_rule_proposals.is_empty());
    assert!(
        first
            .chairman_rule_proposals
            .iter()
            .all(|proposal| proposal.required_audit)
    );
    assert!(
        first
            .chairman_rule_proposals
            .iter()
            .any(|proposal| proposal.proposal_status == ChairmanRuleProposalStatus::NeedsAudit)
    );
    assert_eq!(
        first.chairman_rulebook_version.rulebook_status,
        ChairmanRulebookVersionStatus::RulebookReadyWithWarnings
    );
    assert_eq!(
        first.chairman_ai_governance_policy,
        second.chairman_ai_governance_policy
    );
    assert_eq!(
        first.chairman_rulebook_version,
        second.chairman_rulebook_version
    );
}
