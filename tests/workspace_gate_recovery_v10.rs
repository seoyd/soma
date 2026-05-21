mod support;

use soma_zero::{
    RealWorkspaceTimeoutAttributionConfig, Sprint93TimeoutAttributionRunner,
    WorkspaceGateRecoveryV10Status,
};
use support::sprint69_support as sprint;

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_workspace_gate_recovery_v10.toml", name)
}

#[test]
fn workspace_gate_recovery_improves_without_claiming_full_acceptance() {
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_workspace_gate_recovery_v10(&config("workspace-gate-recovery-v10"))
        .expect("report");
    assert_eq!(
        report.recovery_status,
        WorkspaceGateRecoveryV10Status::GateImprovedButBlocked
    );
    assert_eq!(report.current_full_status, "FullWorkspaceStillBlocked");
}
