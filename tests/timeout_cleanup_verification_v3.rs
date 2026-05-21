mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn timeout_cleanup_verification_v3_defaults_to_not_applicable_without_real_timeout() {
    let bundle = run_sprint110(
        "soma_timeout_cleanup_verification_v3.toml",
        "timeout-cleanup-verification-v3",
    );
    let report = bundle.timeout_cleanup_verification_report_v3;
    assert!(!report.timeout_occurred);
    assert_eq!(report.cleanup_status, "NotApplicable");
}
