mod support;

use soma_zero::ChairmanRuleProposalRiskAuditV2Status;
use support::sprint99_support::run_sprint99;

#[test]
fn chairman_rule_proposal_risk_audit_v2_is_safe_for_paper() {
    let bundle = run_sprint99(
        "soma_chairman_rule_risk_audit_v2.toml",
        "chairman-rule-risk-audit-v2",
    );
    let report = bundle.chairman_rule_proposal_risk_audit_v2;
    assert_eq!(
        report.audit_status,
        ChairmanRuleProposalRiskAuditV2Status::RuleProposalSafeForPaper
    );
    assert!(!report.bypass_risk_governor_detected);
    assert!(!report.live_application_detected);
    assert!(!report.unaudited_change_detected);
    assert!(report.overfit_risk < 0.5);
}
