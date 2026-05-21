mod support;

use soma_zero::{LegacyIntegrationMigrationReportStatus, Sprint86ResidualGateRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn legacy_integration_migration_lists_moved_and_kept_separate_files() {
    let config = sprint::sprint86_config_from_example(
        "soma_legacy_integration_migration.toml",
        "legacy-integration-migration-test",
    );
    let report = Sprint86ResidualGateRecoveryRunner::default()
        .run_legacy_integration_migration(&config)
        .expect("migration");
    assert_eq!(
        report.migration_status,
        LegacyIntegrationMigrationReportStatus::MigrationReadyWithWarnings
    );
    assert!(
        report
            .migrated_files
            .iter()
            .any(|file| file.ends_with("external_model_watchlist.rs"))
    );
    assert!(
        report
            .kept_separate_files
            .iter()
            .any(|file| file.ends_with("external_model_research_ops_cli_safety.rs"))
    );
}
