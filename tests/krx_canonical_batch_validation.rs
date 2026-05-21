mod common;

use std::fs;
use std::path::PathBuf;

use soma_zero::{KRXCanonicalBatchValidationReport, KRXCanonicalBatchValidationStatus};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

#[test]
fn valid_extended_krx_batch_passes() {
    let report = KRXCanonicalBatchValidationReport::build(
        "sprint50-batch",
        &vec![
            example_path("sprint50_data/krx_005930_extended_1d.csv")
                .display()
                .to_string(),
        ],
        true,
        true,
    );
    assert_eq!(
        report.validation_status,
        KRXCanonicalBatchValidationStatus::BatchValid
    );
    assert_eq!(report.valid_csv_count, 1);
}

#[test]
fn missing_provenance_blocks_official_readiness() {
    let output_dir = common::output_dir("krx-batch-missing-provenance");
    let copied = output_dir.join("krx_005930_missing_provenance.csv");
    fs::copy(
        example_path("sprint49_data/krx_005930_1d_compact.csv"),
        &copied,
    )
    .expect("copy csv without sidecars");
    let report = KRXCanonicalBatchValidationReport::build(
        "sprint50-batch-missing-provenance",
        &vec![copied.display().to_string()],
        true,
        true,
    );
    assert_eq!(
        report.validation_status,
        KRXCanonicalBatchValidationStatus::MissingProvenance
    );
    assert!(report.missing_provenance_count > 0);
}
