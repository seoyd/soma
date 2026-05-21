mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn sprint109_focused_suite_import_is_explicit_and_not_full_acceptance() {
    let bundle = run_sprint110(
        "soma_sprint109_focused_suite_import.toml",
        "sprint109-focused-suite-import",
    );
    let report = bundle.sprint109_focused_suite_result_import_report;
    assert_eq!(report.test_targets, 14);
    assert_eq!(report.test_count, 23);
    assert!(report.passed);
    assert_eq!(report.failed_count, 0);
    assert!(!report.imported_as_full_acceptance);
    assert_eq!(report.import_status, "FocusedSuiteResultImported");
}
