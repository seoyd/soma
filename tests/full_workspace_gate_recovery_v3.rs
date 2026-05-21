mod support;

use soma_zero::{FullWorkspaceGateRecoveryReportV3Status, Sprint85WorkspaceGateRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn full_workspace_gate_recovery_v3_reports_improved_but_blocked() {
    let config = sprint::sprint85_config_from_example(
        "soma_full_gate_recovery_v3.toml",
        "full-gate-recovery-v3-test",
    );
    let report = Sprint85WorkspaceGateRecoveryRunner::default()
        .run_full_gate_recovery_v3(&config)
        .expect("recovery");
    assert_eq!(
        report.recovery_status,
        FullWorkspaceGateRecoveryReportV3Status::GateImprovedButBlocked
    );
    assert_eq!(report.binary_count_delta, Some(9));
    assert_eq!(report.previous_gate_status, "FullWorkspaceStillBlocked");
}
