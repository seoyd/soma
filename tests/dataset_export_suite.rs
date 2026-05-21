mod common;
#[path = "support/sprint62_support.rs"]
mod sprint62_support;

use soma_zero::{
    SequenceDatasetExportRunner, SequenceDatasetQualityStatus, SequenceLabelKind,
    SequenceSplitPolicy,
};

#[test]
fn dataset_export_config_rejects_remote_paths() {
    let mut config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_export_small.toml",
        "dataset-export-remote-suite",
    );
    config.kis_canonical_csv_paths = vec!["https://example.com/rows.json".to_string()];
    let err = config.validate().expect_err("remote paths should fail");
    assert!(err.contains("local"));
}

#[test]
fn dataset_export_builds_expected_bundle_with_chronological_split() {
    let config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_export_small.toml",
        "dataset-export-suite",
    );
    let bundle = SequenceDatasetExportRunner::default()
        .run(&config)
        .expect("run export");
    assert_eq!(bundle.sequence_dataset_export_artifact.row_count, 6);
    assert_eq!(
        bundle.sequence_dataset_export_artifact.rows[0].label_kind,
        SequenceLabelKind::TakeProfit
    );
    assert!(
        bundle.sequence_dataset_export_artifact.rows[0].label_timestamp_ms
            > bundle.sequence_dataset_export_artifact.rows[0].window_end_timestamp_ms
    );
    assert_eq!(
        bundle.quality_report.quality_status,
        SequenceDatasetQualityStatus::ExportReadyWithWarnings
    );
    assert_eq!(
        bundle.split_manifest.as_ref().expect("split").policy,
        SequenceSplitPolicy::ChronologicalHoldout
    );
}

#[test]
fn dataset_export_is_deterministic() {
    let config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_export_small.toml",
        "dataset-export-deterministic-suite",
    );
    let runner = SequenceDatasetExportRunner::default();
    let left = runner.run(&config).expect("first export");
    let right = runner.run(&config).expect("second export");
    assert_eq!(
        left.sequence_export_manifest.fingerprint,
        right.sequence_export_manifest.fingerprint
    );
    assert_eq!(
        left.sequence_dataset_export_artifact.rows,
        right.sequence_dataset_export_artifact.rows
    );
    assert_eq!(left.split_manifest, right.split_manifest);
}
