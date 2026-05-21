mod support;

use soma_zero::{
    DashboardRendererAssertionMigrationReport, DashboardRendererAssertionMigrationStatus,
    Sprint94DashboardRendererRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn assertion_migration_matches_expected_fixture() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_assertion_migration(&sprint::sprint94_config_from_example(
            "soma_dashboard_renderer_assertion_migration.toml",
            "dashboard-renderer-assertion-migration",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<DashboardRendererAssertionMigrationReport>(
        sprint::example_path("sprint94_data/dashboard_renderer_assertion_migration_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.migration_status,
        DashboardRendererAssertionMigrationStatus::AssertionsMigratedWithWarnings
    );
    assert_eq!(report.assertions_found, 8);
    assert_eq!(report.assertions_migrated, 6);
    assert_eq!(report.assertions_remaining, 2);
    assert_eq!(report.high_risk_assertions_kept_separate.len(), 2);
}
