mod support;

use support::sprint69_support as sprint;

#[test]
fn sprint94_bundle_is_deterministic() {
    let first = sprint::run_sprint94_bundle(
        "soma_sprint94_dashboard_renderer_recover.toml",
        "sprint94-determinism-a",
    );
    let second = sprint::run_sprint94_bundle(
        "soma_sprint94_dashboard_renderer_recover.toml",
        "sprint94-determinism-b",
    );
    assert_eq!(
        first.dashboard_renderer_real_reduction_plan,
        second.dashboard_renderer_real_reduction_plan
    );
    assert_eq!(
        first.dashboard_renderer_real_reduction_report,
        second.dashboard_renderer_real_reduction_report
    );
    assert_eq!(
        first.dashboard_renderer_assertion_migration_report,
        second.dashboard_renderer_assertion_migration_report
    );
    assert_eq!(
        first.control_tower_dashboard_renderer_recovery_panel,
        second.control_tower_dashboard_renderer_recovery_panel
    );
    assert_eq!(first.final_summary, second.final_summary);
}
