mod support;

use soma_zero::{
    DashboardRendererNoBrowserExecutionReport, DashboardRendererNoBrowserExecutionStatus,
    Sprint94DashboardRendererRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn no_browser_execution_matches_expected_fixture() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_no_browser_execution(&sprint::sprint94_config_from_example(
            "soma_dashboard_renderer_no_browser_execution.toml",
            "dashboard-renderer-no-browser",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<DashboardRendererNoBrowserExecutionReport>(
        sprint::example_path("sprint94_data/dashboard_renderer_no_browser_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.browser_status,
        DashboardRendererNoBrowserExecutionStatus::NoBrowserExecutionPreserved
    );
}
