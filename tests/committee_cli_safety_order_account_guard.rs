mod support;

use soma_zero::{
    CommitteeCliSafetyOrderAccountGuardReport, CommitteeCliSafetyOrderAccountStatus,
    Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn order_account_guard_matches_expected_fixture() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_order_account_guard(&sprint::sprint95_config_from_example(
            "soma_committee_cli_safety_order_account_guard.toml",
            "committee-cli-safety-order-account",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<CommitteeCliSafetyOrderAccountGuardReport>(
        sprint::example_path("sprint95_data/committee_cli_safety_order_account_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.order_account_status,
        CommitteeCliSafetyOrderAccountStatus::OrderAccountGuardPreserved
    );
}
