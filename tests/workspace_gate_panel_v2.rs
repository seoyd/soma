mod support;

use soma_zero::{
    DomainGroupedTestSuiteReportStatus, SharedFixtureHarnessAdoptionStatus,
    Sprint85WorkspaceGateRecoveryRunner, WorkspaceWideAcceptanceAttemptV3Status,
    WorkspaceWideSmokePolicyV2Status,
};
use support::sprint69_support as sprint;

#[test]
fn workspace_gate_panel_v2_summarizes_gate_and_safety_state() {
    let config = sprint::sprint85_config_from_example(
        "soma_control_tower_workspace_gate_v2.toml",
        "control-tower-workspace-gate-v2-test",
    );
    let report = Sprint85WorkspaceGateRecoveryRunner::default()
        .run_control_tower_workspace_gate_v2(&config)
        .expect("panel");
    assert_eq!(
        report.workspace_gate_status,
        WorkspaceWideAcceptanceAttemptV3Status::FullWorkspaceStillBlocked
    );
    assert_eq!(
        report.binary_consolidation_status,
        DomainGroupedTestSuiteReportStatus::DomainSuitesReadyWithWarnings
    );
    assert_eq!(
        report.shared_fixture_harness_status,
        SharedFixtureHarnessAdoptionStatus::HarnessAdoptionReadyWithWarnings
    );
    assert_eq!(
        report.smoke_policy_status,
        WorkspaceWideSmokePolicyV2Status::WorkspaceSmokePolicyReady
    );
}
