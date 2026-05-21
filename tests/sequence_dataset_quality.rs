mod common;
#[path = "support/sprint62_support.rs"]
mod sprint62_support;

use serde_json::json;
use soma_zero::{SequenceDatasetExportRunner, SequenceDatasetQualityStatus};

#[test]
fn quality_report_is_export_ready_with_warnings() {
    let config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_quality.toml",
        "quality-ready",
    );
    let report = SequenceDatasetExportRunner::default()
        .run_quality(&config)
        .expect("run quality");
    assert_eq!(
        report.quality_status,
        SequenceDatasetQualityStatus::ExportReadyWithWarnings
    );
}

#[test]
fn no_lookahead_violation_blocks_quality() {
    let mut config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_quality.toml",
        "quality-no-lookahead",
    );
    let path = sprint62_support::write_support_json(
        "quality-no-lookahead",
        "no_lookahead_bad.json",
        &json!({
            "checked_windows": 6,
            "failed_windows": 1,
            "violation_examples": ["future label leaked"]
        }),
    );
    config.no_lookahead_proof_paths = vec![path];
    let report = SequenceDatasetExportRunner::default()
        .run_quality(&config)
        .expect("run blocked quality");
    assert_eq!(
        report.quality_status,
        SequenceDatasetQualityStatus::NoLookaheadViolation
    );
}
