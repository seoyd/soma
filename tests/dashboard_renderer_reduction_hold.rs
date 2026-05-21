mod support;

use soma_zero::{
    DashboardRendererReductionHoldStatus, RealWorkspaceTimeoutAttributionConfig,
    Sprint93TimeoutAttributionRunner,
};
use support::sprint69_support as sprint;

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_dashboard_renderer_reduction_hold.toml", name)
}

#[test]
fn dashboard_renderer_reduction_stays_held_even_when_entry_is_released() {
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_dashboard_renderer_reduction_hold(&config("dashboard-renderer-reduction-hold"))
        .expect("report");
    assert_eq!(
        report.hold_status,
        DashboardRendererReductionHoldStatus::ReductionReadyButNotStarted
    );
    assert!(!report.dashboard_renderer_reduction_started);
}
