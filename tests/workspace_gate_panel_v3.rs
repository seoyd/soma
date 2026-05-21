mod support;

use soma_zero::{
    CargoTestNoRunGateStatus, CompileOnlyWorkspaceCompileStatus,
    FullWorkspaceAcceptanceAttemptV4Status, ResidualWorkspaceBinaryAuditStatus,
    SafetyCoveragePreservationReportV2Status, Sprint86ResidualGateRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn workspace_gate_panel_v3_shows_residual_and_compile_only_statuses() {
    let config = sprint::sprint86_config_from_example(
        "soma_control_tower_workspace_gate_v3.toml",
        "control-tower-workspace-gate-v3-test",
    );
    let report = Sprint86ResidualGateRecoveryRunner::default()
        .run_control_tower_workspace_gate_v3(&config)
        .expect("panel");
    assert_eq!(
        report.workspace_gate_status,
        FullWorkspaceAcceptanceAttemptV4Status::FullWorkspaceStillBlocked
    );
    assert_eq!(
        report.residual_audit_status,
        ResidualWorkspaceBinaryAuditStatus::ResidualAuditReadyWithWarnings
    );
    assert_eq!(
        report.compile_only_status,
        CompileOnlyWorkspaceCompileStatus::CompileOnlyPassed
    );
    assert_eq!(
        report.no_run_gate_status,
        CargoTestNoRunGateStatus::NoRunGatePassed
    );
    assert_eq!(
        report.safety_coverage_status,
        SafetyCoveragePreservationReportV2Status::SafetyCoveragePreserved
    );
}
