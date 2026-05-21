mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn sprint109_validation_reconciliation_imports_external_truth_without_claiming_full_acceptance() {
    let bundle = run_sprint110(
        "soma_sprint109_validation_reconcile.toml",
        "sprint109-validation-reconcile",
    );
    let report = bundle.sprint109_external_validation_reconciliation_report;
    assert!(report.focused_suite_imported);
    assert!(report.focused_suite_passed);
    assert_eq!(report.focused_test_target_count, Some(14));
    assert_eq!(report.focused_test_count, Some(23));
    assert!(report.cli_smoke_imported);
    assert!(report.cargo_build_imported);
    assert!(report.workspace_no_run_timeout_imported);
    assert!(report.workspace_full_timeout_imported);
    assert!(report.timeout_cleanup_imported);
    assert!(report.no_remaining_cargo_rustc_processes);
    assert_eq!(
        report.reconciliation_status,
        "Sprint109ValidationReconciledWithWarnings"
    );
    assert!(!bundle.acceptance_truth_gate_v11.can_claim_full_acceptance);
}
