mod support;

use soma_zero::{
    DashboardRendererDeterminismPreservationReport, DashboardRendererDeterminismStatus,
    Sprint94DashboardRendererRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn determinism_matches_expected_fixture() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_determinism_preservation(&sprint::sprint94_config_from_example(
            "soma_dashboard_renderer_determinism_preservation.toml",
            "dashboard-renderer-determinism",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<DashboardRendererDeterminismPreservationReport>(
        sprint::example_path("sprint94_data/dashboard_renderer_determinism_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.determinism_status,
        DashboardRendererDeterminismStatus::DeterminismPreserved
    );
}
