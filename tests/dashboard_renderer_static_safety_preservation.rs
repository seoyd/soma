mod support;

use soma_zero::{
    DashboardRendererStaticSafetyPreservationReport, DashboardRendererStaticSafetyStatus,
    Sprint94DashboardRendererRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn static_safety_matches_expected_fixture() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_static_safety_preservation(&sprint::sprint94_config_from_example(
            "soma_dashboard_renderer_static_safety_preservation.toml",
            "dashboard-renderer-static-safety",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<DashboardRendererStaticSafetyPreservationReport>(
        sprint::example_path("sprint94_data/dashboard_renderer_static_safety_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.static_safety_status,
        DashboardRendererStaticSafetyStatus::StaticSafetyPreserved
    );
}
