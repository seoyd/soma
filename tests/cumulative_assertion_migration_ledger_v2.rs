mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn cumulative_assertion_migration_ledger_v2_rolls_forward_prior_ledgers() {
    let bundle = run_sprint110(
        "soma_cumulative_assertion_migration_ledger_v2.toml",
        "cumulative-assertion-migration-ledger-v2",
    );
    let report = bundle.cumulative_assertion_migration_ledger_report;
    assert_eq!(report.ledger_count, 4);
    assert_eq!(report.cumulative_moved_assertions, 8);
    assert_eq!(report.cumulative_preserved_assertions, 10);
    assert_eq!(report.cumulative_assertion_delta, 0);
    assert_eq!(report.retired_target_count, 4);
    assert_eq!(report.coverage_gap_count, 0);
    assert_eq!(report.cumulative_status, "CumulativeLedgerReady");
    assert_eq!(
        report.validation_reconciliation_refs,
        vec!["sprint109-validation-reconciliation".to_string()]
    );
}
