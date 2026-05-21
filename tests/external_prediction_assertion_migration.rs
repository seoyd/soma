mod support;

use soma_zero::{
    ExternalPredictionAssertionMigrationStatus, Sprint90ExternalPredictionRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn external_prediction_assertion_migration_counts_current_suite_assertions() {
    let config = sprint::sprint90_config_from_example(
        "soma_external_prediction_assertion_migration.toml",
        "external-assertion-migration",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_external_prediction_assertion_migration(&config)
        .expect("report");
    assert_eq!(report.assertions_found, 13);
    assert_eq!(report.assertions_migrated, 13);
    assert_eq!(report.assertions_remaining, 0);
    assert_eq!(
        report.migration_status,
        ExternalPredictionAssertionMigrationStatus::AssertionsMigrated
    );
}

#[test]
fn external_prediction_assertion_migration_is_deterministic() {
    let config = sprint::sprint90_config_from_example(
        "soma_external_prediction_assertion_migration.toml",
        "external-assertion-migration-deterministic",
    );
    let first = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_external_prediction_assertion_migration(&config)
        .expect("first");
    let second = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_external_prediction_assertion_migration(&config)
        .expect("second");
    assert_eq!(first, second);
}
