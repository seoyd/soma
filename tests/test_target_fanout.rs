mod support;

use soma_zero::{Sprint87CompileGateRecoveryRunner, TestTargetFanoutStatus};
use support::sprint69_support as sprint;

#[test]
fn test_target_fanout_counts_targets_and_records_keep_separate_entries() {
    let config =
        sprint::sprint87_config_from_example("soma_test_target_fanout.toml", "test-target-fanout");
    let report = Sprint87CompileGateRecoveryRunner::default()
        .run_test_target_fanout(&config)
        .expect("fanout");
    assert_eq!(report.total_integration_test_targets, Some(132));
    assert!(
        report
            .broad_family_targets
            .contains(&"future_window_requirements".to_string())
    );
    assert!(
        report
            .grouped_targets_existing
            .contains(&"tests/workspace_safety_guard_suite.rs".to_string())
    );
    assert!(
        report
            .keep_separate_targets
            .contains(&"committee_cli_safety".to_string())
    );
    assert_eq!(
        report.fanout_status,
        TestTargetFanoutStatus::FanoutReportReadyWithWarnings
    );
}
