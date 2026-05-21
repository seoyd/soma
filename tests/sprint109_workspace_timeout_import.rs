mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn sprint109_workspace_timeout_import_preserves_timeout_truth() {
    let bundle = run_sprint110(
        "soma_sprint109_workspace_timeout_import.toml",
        "sprint109-workspace-timeout-import",
    );
    let report = bundle.sprint109_workspace_timeout_import_report;
    assert_eq!(report.no_run_timeout_seconds, Some(180));
    assert_eq!(report.no_run_exit_code, Some(124));
    assert_eq!(report.full_timeout_seconds, Some(180));
    assert_eq!(report.full_exit_code, Some(124));
    assert!(report.no_remaining_cargo_rustc_processes);
    assert!(!report.imported_as_pass);
    assert_eq!(report.timeout_status, "WorkspaceTimeoutImported");
}
