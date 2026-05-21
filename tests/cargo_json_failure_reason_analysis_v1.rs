mod support;

use soma_zero::{
    CargoJsonFailureReasonAnalysisReportV1, CargoJsonReasonLineClassificationReportV1,
};
use support::sprint118_support::{read_fixture, run_sprint118};

#[test]
fn cargo_json_failure_reason_analysis_matches_expected() {
    let bundle = run_sprint118(
        "soma_cargo_json_failure_reason_analysis_v1.toml",
        "cargo-json-failure-reason-analysis-v1",
    );
    let expected: CargoJsonFailureReasonAnalysisReportV1 =
        read_fixture("sprint118_data/cargo_json_failure_reason_analysis_expected.json");
    assert_eq!(
        bundle.cargo_json_failure_reason_analysis_report_v1,
        expected
    );
    let expected_lines: CargoJsonReasonLineClassificationReportV1 =
        read_fixture("sprint118_data/cargo_json_reason_line_classification_expected.json");
    assert_eq!(
        bundle.cargo_json_reason_line_classification_report_v1,
        expected_lines
    );
}
