mod support;

use soma_zero::{Sprint95CommitteeCliSafetyRecoveryRunner, WorkspaceGateRecoveryV12Status};
use support::sprint69_support as sprint;

#[test]
fn workspace_gate_recovery_stays_improved_but_blocked() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_workspace_gate_recovery_v12(&sprint::sprint95_config_from_example(
            "soma_workspace_gate_recovery_v12.toml",
            "workspace-gate-recovery-v12",
        ))
        .expect("report");
    assert_eq!(
        report.recovery_status,
        WorkspaceGateRecoveryV12Status::GateImprovedButBlocked
    );
    assert_eq!(report.current_full_status, "FullWorkspaceStillBlocked");
}
