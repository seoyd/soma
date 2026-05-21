mod support;

use soma_zero::{Sprint87CompileGateRecoveryRunner, WorkspaceCompileGateRecoveryReportStatus};
use support::sprint69_support as sprint;

#[test]
fn workspace_compile_gate_recovery_threads_previous_current_and_delta() {
    let config = sprint::sprint87_config_from_example(
        "soma_compile_gate_recovery.toml",
        "compile-gate-recovery",
    );
    let first = Sprint87CompileGateRecoveryRunner::default()
        .run_compile_gate_recovery(&config)
        .expect("first");
    let second = Sprint87CompileGateRecoveryRunner::default()
        .run_compile_gate_recovery(&config)
        .expect("second");
    assert_eq!(first.previous_compile_only_status, "CompileOnlyPassed");
    assert_eq!(
        first.previous_full_workspace_status,
        "FullWorkspaceStillBlocked"
    );
    assert_eq!(first.target_delta, Some(5));
    assert_eq!(
        first.recovery_status,
        WorkspaceCompileGateRecoveryReportStatus::GateImprovedButBlocked
    );
    assert_eq!(first, second);
}
