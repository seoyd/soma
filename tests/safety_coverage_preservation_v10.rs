mod support;

use soma_zero::{
    SafetyCoveragePreservationReportV10, SafetyCoveragePreservationReportV10Status,
    Sprint94DashboardRendererRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn safety_coverage_matches_expected_fixture() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_safety_coverage_preservation_v10(&sprint::sprint94_config_from_example(
            "soma_safety_coverage_preservation_v10.toml",
            "safety-coverage-preservation-v10",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<SafetyCoveragePreservationReportV10>(
        sprint::example_path("sprint94_data/safety_coverage_v10_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.safety_status,
        SafetyCoveragePreservationReportV10Status::SafetyCoveragePreserved
    );
}
