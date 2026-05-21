mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn assertion_migration_ledger_v4_rolls_shared_toml_assertions_into_fixture_harness() {
    let bundle = run_sprint110(
        "soma_assertion_migration_ledger_v4.toml",
        "assertion-migration-ledger-v4",
    );
    let report = bundle.assertion_migration_ledger_v4;
    assert_eq!(
        report.previous_ledger_refs,
        vec!["assertion-migration-ledger-v3".to_string()]
    );
    assert_eq!(report.moved_assertions.len(), 2);
    assert_eq!(report.preserved_assertions.len(), 2);
    assert_eq!(report.assertion_delta, 0);
    assert_eq!(
        report.source_targets,
        vec!["tests/shared_toml_builder_application_v1.rs".to_string()]
    );
    assert_eq!(
        report.destination_targets,
        vec!["tests/shared_fixture_harness_application_v1.rs".to_string()]
    );
    assert_eq!(report.ledger_status, "AssertionMigrationLedgerReady");
}
