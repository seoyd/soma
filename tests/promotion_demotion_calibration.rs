mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{PromotionDemotionCalibrationReport, PromotionDemotionCalibrationStatus};
use support::sprint99_support::run_sprint99;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint99_data")
        .join(name)
}

#[test]
fn promotion_demotion_calibration_matches_expected_fixture() {
    let bundle = run_sprint99(
        "soma_promotion_demotion_calibration.toml",
        "promotion-demotion-calibration",
    );
    let expected: PromotionDemotionCalibrationReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("promotion_calibration_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.promotion_demotion_calibration_report, expected);
    assert_eq!(
        bundle
            .promotion_demotion_calibration_report
            .calibration_status,
        PromotionDemotionCalibrationStatus::PromotionCalibrationReady
    );
    assert!(
        bundle
            .promotion_demotion_calibration_report
            .raw_return_only_ranking_blocked
    );
}
