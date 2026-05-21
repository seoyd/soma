mod support;

use soma_zero::{
    ResidualBinaryConsolidationPlanStatus, Sprint86ResidualGateRecoveryRunner,
    TestBinaryConsolidationSemanticRisk,
};
use support::sprint69_support as sprint;

#[test]
fn residual_consolidation_plan_records_delta_and_keep_separate_reason() {
    let config = sprint::sprint86_config_from_example(
        "soma_residual_consolidation_plan.toml",
        "residual-consolidation-plan-test",
    );
    let report = Sprint86ResidualGateRecoveryRunner::default()
        .run_residual_consolidation_plan(&config)
        .expect("plan");
    assert_eq!(
        report.plan_status,
        ResidualBinaryConsolidationPlanStatus::ResidualConsolidationPlanReady
    );
    assert_eq!(report.expected_binary_delta, 12);
    assert_eq!(
        report.semantic_risk,
        TestBinaryConsolidationSemanticRisk::Medium
    );
    assert!(
        report
            .planned_suites
            .contains_key("tests/model_ops_qa_suite.rs")
    );
}
