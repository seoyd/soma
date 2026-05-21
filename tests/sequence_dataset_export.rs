mod common;
#[path = "support/sprint62_support.rs"]
mod sprint62_support;

use soma_zero::{
    SequenceDatasetExportRunner, SequenceDatasetQualityStatus, SequenceLabelKind,
    SequenceSplitPolicy,
};

#[test]
fn sequence_dataset_export_config_rejects_remote_paths() {
    let mut config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_export_small.toml",
        "export-remote",
    );
    config.kis_canonical_csv_paths = vec!["https://example.com/rows.json".to_string()];
    let err = config.validate().expect_err("remote paths should fail");
    assert!(err.contains("local"));
}

#[test]
fn sequence_dataset_export_builds_expected_bundle() {
    let config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_export_small.toml",
        "export-small",
    );
    let bundle = SequenceDatasetExportRunner::default()
        .run(&config)
        .expect("run sequence export");
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
    assert!(
        bundle
            .sequence_export_manifest
            .dataset_csv_path
            .ends_with("dataset.csv")
    );
}
