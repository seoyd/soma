mod support;

use soma_zero::{CandleExpansionAssertionMigrationStatus, Sprint89CandleRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn candle_assertion_migration_counts_current_suite_assertions() {
    let config = sprint::sprint89_config_from_example(
        "soma_candle_assertion_migration.toml",
        "candle-assertion-migration",
    );
    let report = Sprint89CandleRecoveryRunner::default()
        .run_candle_assertion_migration(&config)
        .expect("report");
    assert_eq!(report.assertions_found, 4);
    assert_eq!(report.assertions_migrated, 4);
    assert_eq!(report.assertions_remaining, 0);
    assert_eq!(
        report.migration_status,
        CandleExpansionAssertionMigrationStatus::AssertionsMigrated
    );
}

#[test]
fn candle_assertion_migration_is_deterministic() {
    let config = sprint::sprint89_config_from_example(
        "soma_candle_assertion_migration.toml",
        "candle-assertion-migration-deterministic",
    );
    let first = Sprint89CandleRecoveryRunner::default()
        .run_candle_assertion_migration(&config)
        .expect("first");
    let second = Sprint89CandleRecoveryRunner::default()
        .run_candle_assertion_migration(&config)
        .expect("second");
    assert_eq!(first, second);
}
