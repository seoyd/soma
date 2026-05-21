mod support;

use soma_zero::{
    CompileHeavyFamilyConsolidationPlanStatus, Sprint87CompileGateRecoveryRunner,
    TestBinaryConsolidationSemanticRisk,
};
use support::sprint69_support as sprint;

#[test]
fn compile_heavy_consolidation_plan_covers_all_broad_suites() {
    let config = sprint::sprint87_config_from_example(
        "soma_compile_heavy_consolidation_plan.toml",
        "compile-heavy-consolidation-plan",
    );
    let first = Sprint87CompileGateRecoveryRunner::default()
        .run_compile_heavy_consolidation_plan(&config)
        .expect("first");
    let second = Sprint87CompileGateRecoveryRunner::default()
        .run_compile_heavy_consolidation_plan(&config)
        .expect("second");
    for suite in [
        "tests/future_window_requirements_suite.rs",
        "tests/official_diversity_suite.rs",
        "tests/trinity_operational_loop_suite.rs",
        "tests/dataset_export_suite.rs",
        "tests/control_tower_v1_suite.rs",
        "tests/candle_expansion_ops_suite.rs",
        "tests/external_prediction_family_suite.rs",
        "tests/krx_evidence_suite.rs",
        "tests/dashboard_renderer_suite.rs",
        "tests/baseline_signal_suite.rs",
        "tests/counterfactual_backfill_suite.rs",
    ] {
        assert!(
            first.planned_suites.contains_key(suite),
            "missing planned suite {suite}"
        );
    }
    assert_eq!(
        first.plan_status,
        CompileHeavyFamilyConsolidationPlanStatus::CompileHeavyPlanReady
    );
    assert_eq!(
        first.semantic_risk,
        TestBinaryConsolidationSemanticRisk::Medium
    );
    assert_eq!(first.expected_test_target_delta, 5);
    assert_eq!(first, second);
}
