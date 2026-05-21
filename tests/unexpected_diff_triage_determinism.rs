#[path = "support/sprint69_support.rs"]
mod support;

#[test]
fn unexpected_diff_triage_is_deterministic_for_same_fixture_input() {
    let left = support::run_triage(
        "soma_unexpected_diff_triage.toml",
        "triage-determinism-left",
    );
    let right = support::run_triage(
        "soma_unexpected_diff_triage.toml",
        "triage-determinism-right",
    );

    assert_eq!(
        left.unexpected_diff_triage_report,
        right.unexpected_diff_triage_report
    );
    assert_eq!(
        left.snapshot_diff_classification_report,
        right.snapshot_diff_classification_report
    );
    assert_eq!(
        left.contract_alignment_audit_v2,
        right.contract_alignment_audit_v2
    );
    assert_eq!(
        left.owner_review_closure_v2_report,
        right.owner_review_closure_v2_report
    );
    assert_eq!(
        left.trace_completeness_warning_reduction_report,
        right.trace_completeness_warning_reduction_report
    );
    assert_eq!(
        left.downgrade_evidence_closure_plan,
        right.downgrade_evidence_closure_plan
    );
    assert_eq!(left.diff_root_cause_report, right.diff_root_cause_report);
    assert_eq!(
        left.model_version_review_disposition_report,
        right.model_version_review_disposition_report
    );
    assert_eq!(
        left.control_tower_diff_triage_panel,
        right.control_tower_diff_triage_panel
    );
}
