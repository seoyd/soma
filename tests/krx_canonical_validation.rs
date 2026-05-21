mod common;

use std::fs;
use std::path::PathBuf;

use soma_zero::{KRXCanonicalValidationReport, KRXCanonicalValidationStatus};

fn sprint49_data(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint49_data")
        .join(name)
        .display()
        .to_string()
}

#[test]
fn valid_compact_fixture_is_officially_ready() {
    let report = KRXCanonicalValidationReport::validate(
        &sprint49_data("krx_005930_1d_compact.csv"),
        Some("005930".to_string()),
        Some("005930".to_string()),
        Some(&sprint49_data("krx_005930_1d_compact_provenance.json")),
        Some(&sprint49_data("krx_005930_1d_compact_preflight.json")),
        true,
        true,
    );
    assert_eq!(
        report.validation_status,
        KRXCanonicalValidationStatus::Valid
    );
    assert!(report.official_readiness_eligible);
    assert!(report.row_count > 0);
}

#[test]
fn missing_file_is_reported() {
    let report = KRXCanonicalValidationReport::validate(
        "examples/sprint49_data/does_not_exist.csv",
        None,
        None,
        None,
        None,
        true,
        true,
    );
    assert_eq!(
        report.validation_status,
        KRXCanonicalValidationStatus::MissingFile
    );
}

#[test]
fn missing_required_columns_and_duplicates_are_detected() {
    let out = common::output_dir("krx-canonical-validation");
    let missing_columns = out.join("missing_columns.csv");
    fs::write(
        &missing_columns,
        "timestamp_ms,open,high,low,close\n1,1,1,1,1\n",
    )
    .expect("write csv");
    let report = KRXCanonicalValidationReport::validate(
        &missing_columns.display().to_string(),
        None,
        None,
        None,
        None,
        false,
        false,
    );
    assert_eq!(
        report.validation_status,
        KRXCanonicalValidationStatus::MissingRequiredColumns
    );

    let duplicate_csv = out.join("duplicate.csv");
    fs::write(
        &duplicate_csv,
        concat!(
            "timestamp_ms,open,high,low,close,volume,trade_value,bid,ask,spread_bps\n",
            "1,1,2,1,2,3,4,1,2,1\n",
            "1,1,2,1,2,3,4,1,2,1\n"
        ),
    )
    .expect("write duplicate csv");
    let duplicate_report = KRXCanonicalValidationReport::validate(
        &duplicate_csv.display().to_string(),
        None,
        None,
        None,
        None,
        false,
        false,
    );
    assert_eq!(
        duplicate_report.validation_status,
        KRXCanonicalValidationStatus::DuplicateTimestamp
    );
}

#[test]
fn missing_provenance_or_preflight_blocks_official_readiness() {
    let no_provenance = KRXCanonicalValidationReport::validate(
        &sprint49_data("krx_000660_1d_compact.csv"),
        Some("000660".to_string()),
        Some("000660".to_string()),
        None,
        Some(&sprint49_data("krx_000660_1d_compact_preflight.json")),
        true,
        true,
    );
    assert_eq!(
        no_provenance.validation_status,
        KRXCanonicalValidationStatus::ProvenanceMissing
    );

    let no_preflight = KRXCanonicalValidationReport::validate(
        &sprint49_data("krx_000660_1d_compact.csv"),
        Some("000660".to_string()),
        Some("000660".to_string()),
        Some(&sprint49_data("krx_000660_1d_compact_provenance.json")),
        None,
        true,
        true,
    );
    assert_eq!(
        no_preflight.validation_status,
        KRXCanonicalValidationStatus::PreflightMissing
    );
}

#[test]
fn validation_is_deterministic() {
    let first = KRXCanonicalValidationReport::validate(
        &sprint49_data("krx_005930_1d_compact.csv"),
        Some("005930".to_string()),
        Some("005930".to_string()),
        Some(&sprint49_data("krx_005930_1d_compact_provenance.json")),
        Some(&sprint49_data("krx_005930_1d_compact_preflight.json")),
        true,
        true,
    );
    let second = KRXCanonicalValidationReport::validate(
        &sprint49_data("krx_005930_1d_compact.csv"),
        Some("005930".to_string()),
        Some("005930".to_string()),
        Some(&sprint49_data("krx_005930_1d_compact_provenance.json")),
        Some(&sprint49_data("krx_005930_1d_compact_preflight.json")),
        true,
        true,
    );
    assert_eq!(first, second);
}
