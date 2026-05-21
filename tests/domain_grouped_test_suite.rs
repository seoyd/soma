mod support;

use soma_zero::{
    DomainGroupedTestSuitePlanStatus, Sprint85WorkspaceGateRecoveryRunner,
    TestBinaryConsolidationSemanticRisk,
};
use support::sprint69_support as sprint;

#[test]
fn domain_grouped_test_suite_plan_preserves_keep_separate_candidate() {
    let config = sprint::sprint85_config_from_example(
        "soma_domain_suite_plan.toml",
        "domain-suite-plan-test",
    );
    let report = Sprint85WorkspaceGateRecoveryRunner::default()
        .run_domain_suite_plan(&config)
        .expect("plan");
    assert_eq!(
        report.plan_status,
        DomainGroupedTestSuitePlanStatus::DomainSuitePlanReady
    );
    assert_eq!(report.expected_binary_delta, 9);
    assert_eq!(
        report.semantic_risk,
        TestBinaryConsolidationSemanticRisk::Medium
    );
    assert!(
        report
            .grouped_suites
            .contains_key("tests/workspace_safety_guard_suite.rs")
    );
}
