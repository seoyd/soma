mod support;

use std::fs;

use soma_zero::{
    KrxEvidenceWarningClosureConfig, SafetyCoveragePreservationReportV8Status,
    Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_safety_coverage_preservation_v8.toml", name)
}

#[test]
fn safety_coverage_matches_expected_fixture_and_is_deterministic() {
    let runner = Sprint92KrxWarningClosureRunner::default();
    let config = config("safety-v8-default");
    let first = runner
        .run_safety_coverage_preservation_v8(&config)
        .expect("first");
    let second = runner
        .run_safety_coverage_preservation_v8(&config)
        .expect("second");
    let expected = harness::load_json_fixture(sprint::example_path(
        "sprint92_data/safety_coverage_v8_expected.json",
    ));
    assert_eq!(first, expected);
    assert_eq!(first, second);
    assert_eq!(
        first.safety_status,
        SafetyCoveragePreservationReportV8Status::SafetyCoveragePreserved
    );
    assert!(first.raw_archive_redaction_guard_present);
}

#[test]
fn safety_coverage_detects_missing_raw_archive_guard() {
    let mut config = config("safety-v8-missing");
    let dir = harness::temp_output_dir_for_test("safety-v8-missing");
    let raw = dir.join("krx_raw_archive_secret_safety.rs");
    fs::write(&raw, "#[test]\nfn bad() { assert!(true); }\n").expect("write raw");
    config.krx_secret_safety_paths = vec![raw.display().to_string()];
    config.krx_raw_archive_paths = vec![raw.display().to_string()];
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_safety_coverage_preservation_v8(&config)
        .expect("report");
    assert_eq!(
        report.safety_status,
        SafetyCoveragePreservationReportV8Status::SafetyCoverageMissing
    );
    assert!(!report.raw_archive_redaction_guard_present);
}
