mod common;
#[path = "support/sprint62_support.rs"]
mod sprint62_support;

use serde_json::json;
use soma_zero::SequenceDatasetExportRunner;

#[test]
fn feature_schema_order_hash_is_stable() {
    let config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_export_small.toml",
        "schema-stable",
    );
    let left = SequenceDatasetExportRunner::default()
        .run(&config)
        .expect("first export");
    let right = SequenceDatasetExportRunner::default()
        .run(&config)
        .expect("second export");
    assert_eq!(
        left.feature_schema_manifest.feature_order_hash,
        right.feature_schema_manifest.feature_order_hash
    );
}

#[test]
fn missing_required_feature_blocks_export() {
    let mut config = sprint62_support::export_config_from_example(
        "soma_sequence_dataset_export_small.toml",
        "schema-missing",
    );
    let path = sprint62_support::write_support_json(
        "schema-missing",
        "feature_schema_missing.json",
        &json!({
            "feature_names": ["open", "high"],
            "required_features": ["open", "high", "close"],
            "optional_features": [],
            "missing_features": ["close"],
            "feature_normalization_policy": "explicit-zscore",
            "version": "v1",
            "frozen": true
        }),
    );
    config.feature_schema_lock_paths = vec![path];
    let err = SequenceDatasetExportRunner::default()
        .run(&config)
        .expect_err("missing required feature should fail");
    assert!(err.contains("missing required"));
}
