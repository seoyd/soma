mod support;

use soma_zero::{Sprint89CandleRecoveryRunner, WorkspaceGateRecoveryV6Status};
use support::sprint69_support as sprint;

#[test]
fn workspace_gate_recovery_v6_threads_previous_and_current_statuses() {
    let config = sprint::sprint89_config_from_example(
        "soma_workspace_gate_recovery_v6.toml",
        "workspace-gate-recovery-v6",
    );
    let report = Sprint89CandleRecoveryRunner::default()
        .run_workspace_gate_recovery_v6(&config)
        .expect("report");
    assert_eq!(report.previous_no_run_status, "RealNoRunStillBlocked");
    assert_eq!(report.current_no_run_status, "RealNoRunStillBlocked");
    assert_eq!(report.previous_full_status, "FullWorkspaceStillBlocked");
    assert_eq!(report.current_full_status, "FullWorkspaceStillBlocked");
    assert_eq!(
        report.recovery_status,
        WorkspaceGateRecoveryV6Status::GateImprovedButBlocked
    );
    assert_eq!(report.remaining_blockers.len(), 6);
}
