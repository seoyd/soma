mod support;

use soma_zero::WorkspaceGateRecoveryV14Status;
use support::sprint69_support as sprint;

#[test]
fn workspace_gate_recovery_v14_stays_improved_but_blocked() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "workspace-gate-recovery-v14",
    )
    .workspace_gate_recovery_v14;
    assert_eq!(
        report.recovery_status,
        WorkspaceGateRecoveryV14Status::GateImprovedButBlocked
    );
}
