mod support;

use soma_zero::{
    TestBinaryConsolidationPlanStatus, TestBinaryConsolidationReportStatus,
    TestBinaryConsolidationSemanticRisk,
};
use support::sprint69_support as sprint;

#[test]
fn sprint84_config_defaults_and_remote_guard_hold() {
    let config = sprint::sprint84_config_from_example(
        "soma_sprint84_test_cost_reduce.toml",
        "sprint84-config-defaults",
    );
    assert!(config.preserve_assertion_count);
    assert!(config.preserve_safety_tests);
    assert!(config.preserve_cli_safety_tests);
    assert!(config.preserve_determinism_tests);
    assert!(config.allow_file_renames);
    assert!(config.allow_test_grouping);

    let mut remote = config.clone();
    remote.sprint83_recovery_paths = vec!["https://example.com/recovery.json".to_string()];
    let error = remote.validate().expect_err("remote paths rejected");
    assert!(error.contains("must be local"));
}

#[test]
fn sprint84_consolidation_plan_and_report_reduce_targeted_binaries() {
    let bundle = sprint::run_sprint84_bundle(
        "soma_test_binary_consolidate.toml",
        "sprint84-consolidation-report",
    );
    assert_eq!(
        bundle.test_binary_consolidation_plan.plan_status,
        TestBinaryConsolidationPlanStatus::ConsolidationPlanReady
    );
    assert_eq!(
        bundle.test_binary_consolidation_plan.semantic_risk,
        TestBinaryConsolidationSemanticRisk::Low
    );
    assert_eq!(
        bundle
            .test_binary_consolidation_plan
            .expected_test_binary_delta,
        14
    );
    assert_eq!(
        bundle.test_binary_consolidation_report.report_status,
        TestBinaryConsolidationReportStatus::TestBinariesReduced
    );
    assert_eq!(bundle.test_binary_consolidation_report.files_before, 16);
    assert_eq!(bundle.test_binary_consolidation_report.files_after, 2);
    assert_eq!(
        bundle.test_binary_consolidation_report.preserved_assertions,
        70
    );
}

#[test]
fn sprint84_high_risk_targets_stay_separate() {
    let mut config = sprint::sprint84_config_from_example(
        "soma_test_binary_consolidate.toml",
        "sprint84-high-risk-plan",
    );
    config
        .target_test_files
        .push("tests/provider_auth_preflight.rs".to_string());
    let bundle = soma_zero::Sprint84TestCostReductionRunner::default()
        .run(&config)
        .expect("run sprint84 bundle");
    assert!(
        bundle
            .test_binary_consolidation_report
            .skipped_high_risk_files
            .contains(&"tests/provider_auth_preflight.rs".to_string())
    );
}

#[test]
fn sprint84_consolidation_bundle_is_deterministic() {
    let first = sprint::run_sprint84_bundle(
        "soma_test_binary_consolidate.toml",
        "sprint84-consolidation-a",
    );
    let second = sprint::run_sprint84_bundle(
        "soma_test_binary_consolidate.toml",
        "sprint84-consolidation-b",
    );
    assert_eq!(
        first.test_binary_consolidation_plan,
        second.test_binary_consolidation_plan
    );
    assert_eq!(
        first.test_binary_consolidation_report,
        second.test_binary_consolidation_report
    );
}
