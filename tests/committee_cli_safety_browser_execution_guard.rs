mod support;

use soma_zero::{
    CommitteeCliSafetyBrowserExecutionGuardReport, CommitteeCliSafetyBrowserExecutionGuardStatus,
    Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn browser_execution_guard_matches_expected_fixture() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_browser_execution_guard(&sprint::sprint95_config_from_example(
            "soma_committee_cli_safety_browser_execution_guard.toml",
            "committee-cli-safety-browser-guard",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<CommitteeCliSafetyBrowserExecutionGuardReport>(
        sprint::example_path("sprint95_data/committee_cli_safety_browser_guard_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.browser_guard_status,
        CommitteeCliSafetyBrowserExecutionGuardStatus::BrowserExecutionGuardPreserved
    );
}
