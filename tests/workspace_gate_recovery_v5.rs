mod support;

use soma_zero::{Sprint88SevenBlockerRecoveryRunner, WorkspaceGateRecoveryV5Status};
use support::sprint69_support as sprint;

#[test]
fn workspace_gate_recovery_v5_threads_previous_and_current_gate_states() {
    let config = sprint::sprint88_config_from_example(
        "soma_workspace_gate_recovery_v5.toml",
        "workspace-gate-recovery",
    );
    let first = Sprint88SevenBlockerRecoveryRunner::default()
        .run_workspace_gate_recovery_v5(&config)
        .expect("first");
    let second = Sprint88SevenBlockerRecoveryRunner::default()
        .run_workspace_gate_recovery_v5(&config)
        .expect("second");
    assert_eq!(first.previous_no_run_status, "NoRunGateStillBlocked");
    assert_eq!(first.current_no_run_status, "RealNoRunStillBlocked");
    assert_eq!(first.previous_full_status, "CompileOnlyBlocked");
    assert_eq!(first.current_full_status, "FullWorkspaceStillBlocked");
    assert_eq!(first.remaining_blockers.len(), 7);
    assert_eq!(
        first.recovery_status,
        WorkspaceGateRecoveryV5Status::GateStillBlocked
    );
    assert_eq!(first, second);
}
