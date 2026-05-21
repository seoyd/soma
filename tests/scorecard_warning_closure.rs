mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::ScorecardCalibrationWarningClosureReport;
use support::sprint100_support::run_sprint100;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint100_data")
        .join(name)
}

#[test]
fn scorecard_warning_closure_matches_expected_fixture() {
    let bundle = run_sprint100(
        "soma_scorecard_warning_closure.toml",
        "scorecard-warning-closure",
    );
    let expected: ScorecardCalibrationWarningClosureReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("scorecard_warning_closure_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(
        bundle.scorecard_calibration_warning_closure_report,
        expected
    );
}
