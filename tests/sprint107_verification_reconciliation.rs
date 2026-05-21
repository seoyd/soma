mod support;

use support::sprint108_support::{read_fixture, run_sprint108};

#[test]
fn sprint107_verification_reconciliation_matches_expected_fixture() {
    let bundle = run_sprint108(
        "soma_sprint107_verification_reconcile.toml",
        "sprint107-verification-reconcile",
    );
    let actual =
        serde_json::to_value(&bundle.sprint107_verification_reconciliation_report).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint108_data/sprint107_verification_reconciliation_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .sprint107_verification_reconciliation_report
            .child_process_cleanup_fix_confirmed
    );
    assert!(
        bundle
            .sprint107_verification_reconciliation_report
            .full_acceptance_requires_sentinel_fix_confirmed
    );
    assert!(
        bundle
            .sprint107_verification_reconciliation_report
            .focused_full_bridge_fix_confirmed
    );
    assert!(
        bundle
            .sprint107_verification_reconciliation_report
            .safety_coverage_all_guard_fix_confirmed
    );
    assert!(
        bundle
            .sprint107_verification_reconciliation_report
            .regression_test_added
    );
}
