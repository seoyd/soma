mod support;

use soma_zero::{Sprint87CompileGateRecoveryRunner, TestTargetDeltaReportV3Status};
use support::sprint69_support as sprint;

#[test]
fn test_target_delta_v3_reports_sample_backed_reduction() {
    let config =
        sprint::sprint87_config_from_example("soma_test_target_delta_v3.toml", "test-target-delta");
    let report = Sprint87CompileGateRecoveryRunner::default()
        .run_test_target_delta_v3(&config)
        .expect("delta");
    assert_eq!(report.target_count_before, Some(17));
    assert_eq!(report.target_count_after, Some(12));
    assert_eq!(report.moved_test_count, 16);
    assert_eq!(report.grouped_suite_count, 11);
    assert_eq!(report.kept_separate_count, 1);
    assert!(report.sample_backed);
    assert!(!report.measured);
    assert_eq!(
        report.delta_status,
        TestTargetDeltaReportV3Status::SampleBackedOnly
    );
}
