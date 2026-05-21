mod support;

use soma_zero::{
    CompileFamilyV2, RealFullWorkspaceGateAttemptV12Status, Sprint94DashboardRendererRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn real_full_workspace_gate_attempt_defaults_to_not_run() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_real_full_workspace_gate_attempt_v12(&sprint::sprint94_config_from_example(
            "soma_real_full_workspace_gate_attempt_v12.toml",
            "real-full-workspace-gate-attempt-v12",
        ))
        .expect("report");
    assert_eq!(
        report.full_status,
        RealFullWorkspaceGateAttemptV12Status::NotRun
    );
    assert_eq!(
        report.rerun_after_family,
        Some(CompileFamilyV2::DashboardRenderer)
    );
}
