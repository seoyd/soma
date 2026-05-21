mod support;

use soma_zero::{
    DashboardRendererFullGateRerunStatus, DashboardRendererNoRunGateRerunStatus,
    Sprint94DashboardRendererRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn no_run_and_full_reruns_stay_not_run_by_default() {
    let config = sprint::sprint94_config_from_example(
        "soma_dashboard_renderer_no_run_rerun.toml",
        "dashboard-renderer-gate-rerun",
    );
    let no_run = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_no_run_rerun(&config)
        .expect("no-run");
    let full = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_full_gate_rerun(&config)
        .expect("full");
    assert_eq!(no_run.status, DashboardRendererNoRunGateRerunStatus::NotRun);
    assert_eq!(full.status, DashboardRendererFullGateRerunStatus::NotRun);
    assert!(no_run.report_id.contains("no-run"));
    assert!(full.report_id.contains("full-rerun"));
}
