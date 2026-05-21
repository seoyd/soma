mod support;

use support::sprint109_support::{read_fixture, run_sprint109};

#[test]
fn sprint107_verification_reconciliation_matches_expected_fixture() {
    let bundle = run_sprint109(
        "soma_sprint108_verification_carry_forward.toml",
        "sprint108-verification-carry-forward",
    );
    let actual =
        serde_json::to_value(&bundle.sprint108_verification_carry_forward_report).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint109_data/sprint108_verification_carry_forward_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .sprint108_verification_carry_forward_report
            .child_process_cleanup_fix_confirmed
    );
    assert!(
        bundle
            .sprint108_verification_carry_forward_report
            .full_acceptance_requires_sentinel_fix_confirmed
    );
    assert!(
        bundle
            .sprint108_verification_carry_forward_report
            .focused_full_bridge_fix_confirmed
    );
    assert!(
        bundle
            .sprint108_verification_carry_forward_report
            .safety_coverage_all_guard_fix_confirmed
    );
    assert!(
        bundle
            .sprint108_verification_carry_forward_report
            .regression_test_added
    );
}
