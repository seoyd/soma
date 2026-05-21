mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::RiskGovernorHandoffWarningClosureReport;
use support::sprint100_support::run_sprint100;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint100_data")
        .join(name)
}

#[test]
fn risk_handoff_warning_closure_matches_expected_fixture() {
    let bundle = run_sprint100(
        "soma_risk_handoff_warning_closure.toml",
        "risk-handoff-warning-closure",
    );
    let expected: RiskGovernorHandoffWarningClosureReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("risk_handoff_warning_closure_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(
        bundle.risk_governor_handoff_warning_closure_report,
        expected
    );
}
