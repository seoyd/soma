mod support;

use soma_zero::{
    SafetyCoveragePreservationReportV11, SafetyCoveragePreservationReportV11Status,
    Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn safety_coverage_v11_matches_expected_fixture() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_safety_coverage_preservation_v11(&sprint::sprint95_config_from_example(
            "soma_safety_coverage_preservation_v11.toml",
            "safety-coverage-preservation-v11",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<SafetyCoveragePreservationReportV11>(
        sprint::example_path("sprint95_data/safety_coverage_v11_expected.json"),
    );
    expected.report_id = report.report_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.safety_status,
        SafetyCoveragePreservationReportV11Status::SafetyCoveragePreserved
    );
}
