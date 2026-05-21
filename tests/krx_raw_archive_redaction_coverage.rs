mod support;

use std::fs;

use soma_zero::{
    KrxEvidenceWarningClosureConfig, KrxRawArchiveRedactionCoverageStatus,
    Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_krx_raw_archive_redaction_coverage.toml", name)
}

#[test]
fn raw_archive_coverage_matches_expected_fixture_and_is_deterministic() {
    let runner = Sprint92KrxWarningClosureRunner::default();
    let config = config("krx-raw-archive-default");
    let first = runner
        .run_krx_raw_archive_redaction_coverage(&config)
        .expect("first");
    let second = runner
        .run_krx_raw_archive_redaction_coverage(&config)
        .expect("second");
    let expected = harness::load_json_fixture(sprint::example_path(
        "sprint92_data/krx_raw_archive_redaction_expected.json",
    ));
    assert_eq!(first, expected);
    assert_eq!(first, second);
    assert_eq!(
        first.coverage_status,
        KrxRawArchiveRedactionCoverageStatus::RedactionCoverageReadyWithIsolatedSentinel
    );
}

#[test]
fn raw_archive_coverage_detects_regressions_and_missing_assertions() {
    let dir = harness::temp_output_dir_for_test("krx-raw-archive-regression");
    let raw = dir.join("krx_raw_archive_secret_safety.rs");
    fs::write(
        &raw,
        "#[test]\nfn archive_redaction_assertions() { assert!(true); }\n",
    )
    .expect("write raw");
    let mut regression = config("krx-raw-archive-regression");
    regression.krx_secret_safety_paths = vec![raw.display().to_string()];
    regression.krx_raw_archive_paths = vec![raw.display().to_string()];
    let regression_report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_raw_archive_redaction_coverage(&regression)
        .expect("regression");
    assert_eq!(
        regression_report.coverage_status,
        KrxRawArchiveRedactionCoverageStatus::RedactionRegression
    );

    fs::write(&raw, "#[test]\nfn different_name() { assert!(true); }\n")
        .expect("write raw missing");
    let missing_report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_raw_archive_redaction_coverage(&regression)
        .expect("missing");
    assert_eq!(
        missing_report.coverage_status,
        KrxRawArchiveRedactionCoverageStatus::RedactionCoverageIncomplete
    );
}
