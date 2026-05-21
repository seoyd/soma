mod support;

use soma_zero::{
    DashboardRendererNoActionControlReport, DashboardRendererNoActionControlStatus,
    Sprint94DashboardRendererRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn no_action_control_matches_expected_fixture() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_no_action_control(&sprint::sprint94_config_from_example(
            "soma_dashboard_renderer_no_action_control.toml",
            "dashboard-renderer-no-action",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<DashboardRendererNoActionControlReport>(
        sprint::example_path("sprint94_data/dashboard_renderer_no_action_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.action_control_status,
        DashboardRendererNoActionControlStatus::NoActionControlsPreserved
    );
}
