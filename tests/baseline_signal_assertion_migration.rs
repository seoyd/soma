mod support;

use soma_zero::BaselineSignalAssertionMigrationStatus;
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_assertion_migration_keeps_entry_sentinels_explicit() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "baseline-signal-assertion-migration",
    );
    let report = bundle.baseline_signal_assertion_migration_report;
    assert_eq!(
        report.migration_status,
        BaselineSignalAssertionMigrationStatus::AssertionsMigratedWithWarnings
    );
    assert_eq!(report.assertions_found, 3);
    assert_eq!(report.assertions_migrated, 1);
    assert_eq!(report.assertions_remaining, 2);
}
