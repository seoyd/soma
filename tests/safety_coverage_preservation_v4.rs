mod support;

use soma_zero::{SafetyCoveragePreservationReportV4Status, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn safety_coverage_preservation_v4_keeps_all_guards_and_committee_isolation() {
    let config = sprint::sprint88_config_from_example(
        "soma_sprint88_seven_blocker_recover.toml",
        "safety-v4",
    );
    let first = Sprint88SevenBlockerRecoveryRunner::default()
        .run_safety_coverage_preservation_v4(&config)
        .expect("first");
    let second = Sprint88SevenBlockerRecoveryRunner::default()
        .run_safety_coverage_preservation_v4(&config)
        .expect("second");
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
    assert!(first.committee_cli_safety_isolated);
    assert_eq!(
        first.safety_status,
        SafetyCoveragePreservationReportV4Status::SafetyCoveragePreserved
    );
    assert_eq!(first, second);
}
