mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{InvestorStyleBlindspotReport, InvestorStyleBlindspotStatus};
use support::sprint99_support::run_sprint99;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint99_data")
        .join(name)
}

#[test]
fn investor_style_blindspot_matches_expected_fixture() {
    let bundle = run_sprint99(
        "soma_investor_style_blindspot.toml",
        "investor-style-blindspot",
    );
    let expected: InvestorStyleBlindspotReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("blindspot_expected.json")).expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.investor_style_blindspot_report, expected);
    assert_eq!(
        bundle.investor_style_blindspot_report.blindspot_status,
        InvestorStyleBlindspotStatus::BlindspotsDocumented
    );
    assert!(
        bundle
            .investor_style_blindspot_report
            .missing_counterbalance_styles
            .is_empty()
    );
}
