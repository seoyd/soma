mod support;

use soma_zero::SafeConsolidationPatchV2Runner;
use support::sprint108_support::{read_fixture, run_sprint108, sprint108_config_from_example};

#[test]
fn timeout_cleanup_verification_matches_expected_fixture() {
    let bundle = run_sprint108(
        "soma_timeout_cleanup_verification_v1.toml",
        "timeout-cleanup-verification-v1",
    );
    let actual =
        serde_json::to_value(&bundle.timeout_cleanup_verification_report_v1).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint108_data/timeout_cleanup_verification_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(
        bundle.timeout_cleanup_verification_report_v1.cleanup_status,
        "NotApplicable"
    );
}

#[test]
fn timeout_cleanup_verification_cleans_up_after_timeout() {
    let mut config = sprint108_config_from_example(
        "soma_timeout_cleanup_verification_v1.toml",
        "timeout-cleanup-verification-v1-timeout",
    );
    config.run_real_no_run_after_patch = true;
    config.no_run_timeout_ms = Some(1);
    let bundle = SafeConsolidationPatchV2Runner::default()
        .run(&config)
        .expect("run");
    let report = bundle.timeout_cleanup_verification_report_v1;
    assert!(report.timeout_occurred);
    assert!(report.remaining_cargo_processes == 0);
    assert!(report.remaining_rustc_processes == 0);
    assert_eq!(report.cleanup_status, "TimeoutCleanupVerified");
}
