mod support;

use soma_zero::{CompileFamilyV2, Sprint94DashboardRendererRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn dashboard_renderer_recovery_panel_stays_read_only() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_control_tower_dashboard_renderer_recovery(&sprint::sprint94_config_from_example(
            "soma_control_tower_dashboard_renderer_recovery.toml",
            "dashboard-renderer-recovery-panel",
        ))
        .expect("report");
    assert_eq!(report.primary_next_family, "CommitteeCliSafety");
    assert_eq!(report.runtime_deferred_status, "RuntimeDeferred");
    assert!(
        report
            .next_actions
            .iter()
            .any(|item| item.contains("CommitteeCliSafety"))
    );
    assert!(!report.warnings.is_empty());
    assert_eq!(
        format!("{:?}", CompileFamilyV2::DashboardRenderer),
        "DashboardRenderer"
    );
}
