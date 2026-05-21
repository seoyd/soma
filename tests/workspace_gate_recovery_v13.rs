mod support;

use soma_zero::WorkspaceGateRecoveryStatusV13;
use support::sprint69_support as sprint;

#[test]
fn sprint96_workspace_gate_recovery_stays_honest() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "workspace-gate-recovery-v13",
    );
    let report = bundle.workspace_gate_recovery_v13;
    assert_eq!(
        report.recovery_status,
        WorkspaceGateRecoveryStatusV13::GateImprovedButBlocked
    );
    assert_eq!(report.previous_no_run_status, report.current_no_run_status);
    assert_eq!(report.previous_full_status, report.current_full_status);
}
