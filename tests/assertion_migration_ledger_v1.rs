mod support;

use support::sprint107_support::{read_fixture, run_sprint107};

#[test]
fn assertion_migration_ledger_tracks_moves_without_deletion() {
    let bundle = run_sprint107(
        "soma_assertion_migration_ledger_v1.toml",
        "assertion-migration-ledger-v1",
    );
    let actual = serde_json::to_value(&bundle.assertion_migration_ledger_v1).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint107_data/assertion_migration_ledger_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(bundle.assertion_migration_ledger_v1.assertion_delta, 0);
    assert_eq!(
        bundle.assertion_migration_ledger_v1.source_targets,
        vec!["tests/shared_fixture_harness_expansion_plan_v2.rs".to_string()]
    );
}
