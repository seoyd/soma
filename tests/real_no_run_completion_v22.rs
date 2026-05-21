mod support;

use support::sprint106_support::run_sprint106;

#[test]
fn no_run_attempt_defaults_to_not_run_and_never_overclaims() {
    let bundle = run_sprint106(
        "soma_real_no_run_completion_v22.toml",
        "real_no_run_completion_v22",
    );
    let report = bundle.real_no_run_completion_attempt_v22;
    assert_eq!(report.no_run_status, "NotRun");
    assert!(!report.finished);
    assert_ne!(report.passed, Some(true));
}
