mod support;

use soma_zero::{FullWorkspaceGateRecoveryReportV4Status, Sprint86ResidualGateRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn full_gate_recovery_v4_threads_binary_delta_and_compile_only_truth() {
    let config = sprint::sprint86_config_from_example(
        "soma_full_gate_recovery_v4.toml",
        "full-gate-recovery-v4-test",
    );
    let report = Sprint86ResidualGateRecoveryRunner::default()
        .run_full_gate_recovery_v4(&config)
        .expect("recovery");
    assert_eq!(
        report.recovery_status,
        FullWorkspaceGateRecoveryReportV4Status::GateImprovedButBlocked
    );
    assert_eq!(report.binary_count_delta, Some(12));
    assert!(report.compile_only_status.contains("CompileOnlyPassed"));
}
