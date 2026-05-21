mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{MemberStyleDriftReport, MemberStyleDriftStatus};
use support::sprint99_support::run_sprint99;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint99_data")
        .join(name)
}

#[test]
fn member_style_drift_matches_expected_fixture() {
    let bundle = run_sprint99("soma_member_style_drift.toml", "member-style-drift");
    let expected: MemberStyleDriftReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("style_drift_expected.json")).expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.member_style_drift_report, expected);
    assert_eq!(
        bundle.member_style_drift_report.style_drift_status,
        MemberStyleDriftStatus::StyleDriftControlled
    );
    assert!(bundle.member_style_drift_report.drift_examples.is_empty());
}
