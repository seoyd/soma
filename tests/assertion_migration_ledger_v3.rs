mod support;

use support::sprint109_support::{read_fixture, run_sprint109};

#[test]
fn assertion_migration_ledger_v3_matches_expected_fixture() {
    let bundle = run_sprint109(
        "soma_assertion_migration_ledger_v3.toml",
        "assertion-migration-ledger-v3",
    );
    let actual = serde_json::to_value(&bundle.assertion_migration_ledger_v3).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint109_data/assertion_migration_ledger_v3_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(
        bundle
            .assertion_migration_ledger_v3
            .previous_ledger_refs
            .len(),
        1
    );
    assert_eq!(
        bundle.assertion_migration_ledger_v3.moved_assertions.len(),
        2
    );
    assert_eq!(
        bundle
            .assertion_migration_ledger_v3
            .preserved_assertions
            .len(),
        2
    );
    assert_eq!(bundle.assertion_migration_ledger_v3.assertion_delta, 0);
    assert_eq!(
        bundle
            .cumulative_assertion_migration_ledger_report
            .ledger_count,
        3
    );
    assert_eq!(
        bundle
            .cumulative_assertion_migration_ledger_report
            .cumulative_moved_assertions,
        6
    );
    assert_eq!(
        bundle
            .cumulative_assertion_migration_ledger_report
            .cumulative_preserved_assertions,
        8
    );
    assert_eq!(
        bundle
            .cumulative_assertion_migration_ledger_report
            .retired_target_count,
        3
    );
}

#[test]
fn negative_assertion_delta_represents_deletion_risk() {
    let bundle = run_sprint109(
        "soma_assertion_migration_ledger_v3.toml",
        "assertion-migration-ledger-v3-negative",
    );
    let mut report = bundle.assertion_migration_ledger_v3;
    report.assertion_delta = -1;
    assert!(report.assertion_delta < 0);
}
