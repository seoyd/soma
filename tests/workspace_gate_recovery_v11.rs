mod support;

use soma_zero::{Sprint94DashboardRendererRecoveryRunner, WorkspaceGateRecoveryV11Status};
use support::sprint69_support as sprint;

#[test]
fn workspace_gate_recovery_stays_improved_but_blocked() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_workspace_gate_recovery_v11(&sprint::sprint94_config_from_example(
            "soma_workspace_gate_recovery_v11.toml",
            "workspace-gate-recovery-v11",
        ))
        .expect("report");
    assert_eq!(
        report.recovery_status,
        WorkspaceGateRecoveryV11Status::GateImprovedButBlocked
    );
    assert_eq!(report.current_full_status, "FullWorkspaceStillBlocked");
}
