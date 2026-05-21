mod support;

use soma_zero::{CargoTestNoRunGateStatus, Sprint86ResidualGateRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn cargo_test_no_run_gate_records_passed_compile_only_status() {
    let config = sprint::sprint86_config_from_example(
        "soma_cargo_test_no_run_gate.toml",
        "cargo-test-no-run-gate-test",
    );
    let report = Sprint86ResidualGateRecoveryRunner::default()
        .run_cargo_test_no_run_gate(&config)
        .expect("no run gate");
    assert_eq!(
        report.gate_status,
        CargoTestNoRunGateStatus::NoRunGatePassed
    );
}
