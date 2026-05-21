mod support;

use soma_zero::{SafetyCoveragePreservationReportV3Status, Sprint87CompileGateRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn safety_coverage_preservation_v3_keeps_all_required_guards() {
    let config = sprint::sprint87_config_from_example(
        "soma_safety_coverage_preservation_v3.toml",
        "safety-coverage-v3",
    );
    let first = Sprint87CompileGateRecoveryRunner::default()
        .run_safety_coverage_preservation_v3(&config)
        .expect("first");
    let second = Sprint87CompileGateRecoveryRunner::default()
        .run_safety_coverage_preservation_v3(&config)
        .expect("second");
    assert_eq!(
        first.safety_status,
        SafetyCoveragePreservationReportV3Status::SafetyCoveragePreserved
    );
    assert!(first.live_trading_guard_present);
    assert!(first.broker_guard_present);
    assert!(first.order_guard_present);
    assert!(first.account_guard_present);
    assert!(first.runtime_llm_guard_present);
    assert!(first.mamba_runtime_guard_present);
    assert!(first.gated_runtime_guard_present);
    assert!(first.model_training_guard_present);
    assert!(first.rust_neural_training_guard_present);
    assert!(first.python_training_dependency_guard_present);
    assert!(first.secret_guard_present);
    assert!(first.no_lookahead_guard_present);
    assert!(first.source_boundary_guard_present);
    assert!(first.browser_execution_guard_present);
    assert!(first.ui_order_control_guard_present);
    assert_eq!(first, second);
}
