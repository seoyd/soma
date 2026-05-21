mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{SafetyCoveragePreservationReportV15, SafetyCoveragePreservationReportV15Status};
use support::sprint99_support::run_sprint99;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint99_data")
        .join(name)
}

#[test]
fn safety_coverage_preservation_v15_matches_expected_fixture() {
    let bundle = run_sprint99(
        "soma_safety_coverage_preservation_v15.toml",
        "safety-coverage-preservation-v15",
    );
    let expected: SafetyCoveragePreservationReportV15 = serde_json::from_str(
        &fs::read_to_string(fixture_path("safety_coverage_v15_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.safety_coverage_preservation_report_v15, expected);
    assert_eq!(
        bundle.safety_coverage_preservation_report_v15.safety_status,
        SafetyCoveragePreservationReportV15Status::SafetyCoveragePreservedWithWarnings
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v15
            .live_trading_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v15
            .runtime_llm_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v15
            .browser_execution_guard_present
    );
}
