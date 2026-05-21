mod support;

use soma_zero::CounterfactualBackfillAssertionMigrationStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_assertion_migration_stays_explicit() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-assertion-migration",
    )
    .counterfactual_backfill_assertion_migration_report;
    assert!(matches!(
        report.migration_status,
        CounterfactualBackfillAssertionMigrationStatus::AssertionsMigratedWithWarnings
            | CounterfactualBackfillAssertionMigrationStatus::AssertionsMigrated
    ));
    assert!(!report.high_risk_assertions_kept_separate.is_empty());
}
