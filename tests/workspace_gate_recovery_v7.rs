mod support;

use soma_zero::{Sprint90ExternalPredictionRecoveryRunner, WorkspaceGateRecoveryV7Status};
use support::sprint69_support as sprint;

#[test]
fn workspace_gate_recovery_v7_threads_previous_and_current_statuses() {
    let config = sprint::sprint90_config_from_example(
        "soma_workspace_gate_recovery_v7.toml",
        "workspace-gate-recovery-v7",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_workspace_gate_recovery_v7(&config)
        .expect("report");
    assert_eq!(report.previous_no_run_status, "RealNoRunStillBlocked");
    assert_eq!(report.current_no_run_status, "RealNoRunStillBlocked");
    assert_eq!(report.previous_full_status, "FullWorkspaceStillBlocked");
    assert_eq!(report.current_full_status, "FullWorkspaceStillBlocked");
    assert_eq!(
        report.recovery_status,
        WorkspaceGateRecoveryV7Status::GateImprovedButBlocked
    );
    assert_eq!(report.remaining_blockers.len(), 5);
}
