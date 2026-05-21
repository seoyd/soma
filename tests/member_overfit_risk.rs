mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{MemberOverfitRiskReport, MemberOverfitRiskStatus};
use support::sprint99_support::run_sprint99;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint99_data")
        .join(name)
}

#[test]
fn member_overfit_risk_matches_expected_fixture() {
    let bundle = run_sprint99("soma_member_overfit_risk.toml", "member-overfit-risk");
    let expected: MemberOverfitRiskReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("overfit_risk_expected.json")).expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.member_overfit_risk_report, expected);
    assert_eq!(
        bundle.member_overfit_risk_report.overfit_status,
        MemberOverfitRiskStatus::OverfitRiskControlledWithWarnings
    );
    assert_eq!(
        bundle
            .member_overfit_risk_report
            .medium_overfit_risk_members,
        5
    );
}
