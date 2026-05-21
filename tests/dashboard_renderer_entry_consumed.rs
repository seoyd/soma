mod support;

use soma_zero::{DashboardRendererEntryConsumedStatus, Sprint94DashboardRendererRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn dashboard_entry_is_consumed_for_dashboard_renderer() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_entry_consumed(&sprint::sprint94_config_from_example(
            "soma_dashboard_renderer_entry_consumed.toml",
            "dashboard-renderer-entry-consumed",
        ))
        .expect("report");
    assert!(report.reduction_started);
    assert!(report.entry_consumed);
    assert_eq!(
        report.consumed_status,
        DashboardRendererEntryConsumedStatus::EntryConsumedForDashboardRenderer
    );
}
