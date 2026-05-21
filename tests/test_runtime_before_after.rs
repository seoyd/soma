mod support;

use soma_zero::TestRuntimeImprovementStatus;
use support::sprint69_support as sprint;

#[test]
fn sprint84_runtime_before_after_is_sample_backed_and_honest() {
    let bundle = sprint::run_sprint84_bundle(
        "soma_test_runtime_before_after.toml",
        "sprint84-runtime-before-after",
    );
    let report = bundle.test_runtime_before_after_report;
    assert_eq!(report.test_binary_count_before, Some(16));
    assert_eq!(report.test_binary_count_after, Some(2));
    assert!(!report.measured);
    assert!(report.sample_backed);
    assert_eq!(
        report.improvement_status,
        TestRuntimeImprovementStatus::SampleBackedOnly
    );
}
