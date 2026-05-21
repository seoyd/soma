mod support;

use soma_zero::{DashboardRendererRecoveryStatus, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn dashboard_renderer_recovery_stays_static_and_read_only() {
    let config = sprint::sprint88_config_from_example(
        "soma_dashboard_renderer_recovery.toml",
        "dashboard-recovery",
    );
    let report = Sprint88SevenBlockerRecoveryRunner::default()
        .run_dashboard_renderer_recovery(&config)
        .expect("report");
    assert!(report.static_html_covered);
    assert!(report.json_txt_state_covered);
    assert!(report.no_post_actions_covered);
    assert!(report.no_secret_leakage_covered);
    assert!(report.no_browser_execution_covered);
    assert!(report.deterministic_render_covered);
    assert_eq!(
        report.recovery_status,
        DashboardRendererRecoveryStatus::DashboardRendererReduced
    );
}
