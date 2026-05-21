mod common;
#[path = "support/sprint62_support.rs"]
mod sprint62_support;

use serde_json::json;
use soma_zero::{SequenceDatasetDriftStatus, SequenceDatasetExportRunner};

#[test]
fn no_drift_is_detected() {
    let config = sprint62_support::drift_config_from_example(
        "soma_sequence_dataset_drift.toml",
        "drift-none",
    );
    let report = SequenceDatasetExportRunner::default()
        .run_drift_guard(&config)
        .expect("run drift");
    assert_eq!(report.drift_status, SequenceDatasetDriftStatus::NoDrift);
}

#[test]
fn feature_schema_drift_is_detected() {
    let mut config = sprint62_support::drift_config_from_example(
        "soma_sequence_dataset_drift.toml",
        "drift-schema",
    );
    let current = sprint62_support::write_support_json(
        "drift-schema",
        "current_manifest.json",
        &json!({
            "feature_schema_manifest_path": "feature_schema_v2.json",
            "label_manifest_path": "label_manifest.json",
            "source_artifacts": ["a.json"],
            "row_count": 6,
            "label_distribution": {"TakeProfit": 2}
        }),
    );
    config.current_manifest_path = current;
    let report = SequenceDatasetExportRunner::default()
        .run_drift_guard(&config)
        .expect("run drift detect");
    assert_eq!(
        report.drift_status,
        SequenceDatasetDriftStatus::UnexpectedFeatureSchemaDrift
    );
}
