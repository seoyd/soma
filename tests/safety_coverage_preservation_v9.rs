mod support;

use soma_zero::{
    RealWorkspaceTimeoutAttributionConfig, SafetyCoveragePreservationReportV9,
    SafetyCoveragePreservationReportV9Status, Sprint93TimeoutAttributionRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_safety_coverage_preservation_v9.toml", name)
}

#[test]
fn safety_coverage_v9_matches_expected_fixture() {
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_safety_coverage_preservation_v9(&config("safety-coverage-preservation-v9"))
        .expect("report");
    let mut expected = harness::load_json_fixture::<SafetyCoveragePreservationReportV9>(
        sprint::example_path("sprint93_data/safety_coverage_v9_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.safety_status,
        SafetyCoveragePreservationReportV9Status::SafetyCoveragePreserved
    );
}
