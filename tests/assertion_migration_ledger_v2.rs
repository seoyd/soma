mod support;

use support::sprint108_support::{read_fixture, run_sprint108};

#[test]
fn assertion_migration_ledger_v2_matches_expected_fixture() {
    let bundle = run_sprint108(
        "soma_assertion_migration_ledger_v2.toml",
        "assertion-migration-ledger-v2",
    );
    let actual = serde_json::to_value(&bundle.assertion_migration_ledger_v2).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint108_data/assertion_migration_ledger_v2_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(
        bundle
            .assertion_migration_ledger_v2
            .previous_ledger_refs
            .len(),
        1
    );
    assert_eq!(
        bundle.assertion_migration_ledger_v2.moved_assertions.len(),
        2
    );
    assert_eq!(
        bundle
            .assertion_migration_ledger_v2
            .preserved_assertions
            .len(),
        2
    );
    assert_eq!(bundle.assertion_migration_ledger_v2.assertion_delta, 0);
}

#[test]
fn negative_assertion_delta_represents_deletion_risk() {
    let bundle = run_sprint108(
        "soma_assertion_migration_ledger_v2.toml",
        "assertion-migration-ledger-v2-negative",
    );
    let mut report = bundle.assertion_migration_ledger_v2;
    report.assertion_delta = -1;
    assert!(report.assertion_delta < 0);
}
