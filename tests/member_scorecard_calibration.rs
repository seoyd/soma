mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{MemberScorecardCalibrationReport, MemberScorecardCalibrationStatus};
use support::sprint99_support::run_sprint99;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint99_data")
        .join(name)
}

#[test]
fn member_scorecard_calibration_matches_expected_fixture() {
    let bundle = run_sprint99(
        "soma_member_scorecard_calibration.toml",
        "member-scorecard-calibration",
    );
    let expected: MemberScorecardCalibrationReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("scorecard_calibration_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.member_scorecard_calibration_report, expected);
    assert_eq!(
        bundle.member_scorecard_calibration_report.scorecard_status,
        MemberScorecardCalibrationStatus::ScorecardCalibrationReadyWithWarnings
    );
    assert_eq!(
        bundle.member_scorecard_calibration_report.scorecard_count,
        8
    );
}
