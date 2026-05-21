mod common;
#[path = "support/sprint64_support.rs"]
mod sprint64_support;

use std::fs;

use soma_zero::{ExternalArtifactRegistryRunner, ExternalModelCardV2, ModelVersionLineageStatus};

#[test]
fn valid_lineage_is_reported_for_latest_model_version() {
    let config = sprint64_support::registry_config_from_example(
        "soma_external_artifact_registry.toml",
        "lineage-valid",
    );
    let report = ExternalArtifactRegistryRunner::default()
        .run(&config)
        .expect("run registry")
        .model_card_lineage_report;
    let latest = report
        .records
        .iter()
        .find(|record| record.model_id == "ext-model-a" && record.model_version == "1.1.0")
        .expect("find ext-model-a latest lineage");
    assert_eq!(
        latest.lineage_status,
        ModelVersionLineageStatus::LineageValid
    );
    assert!(report.valid_lineage_count >= 1);
}

#[test]
fn schema_change_is_detected() {
    let mut config = sprint64_support::registry_config_from_example(
        "soma_external_artifact_registry.toml",
        "lineage-schema-change",
    );
    let card_path = sprint64_support::absolutize("examples/sprint64_data/model_card_a_v2.json");
    let mut card: ExternalModelCardV2 =
        serde_json::from_str(&fs::read_to_string(card_path).expect("read model card"))
            .expect("parse model card");
    card.feature_schema_hash = "schema-change".to_string();
    let changed_path = sprint64_support::write_support_json(
        "lineage-schema-change",
        "model_card_a_v2.json",
        &card,
    );
    config.external_model_card_paths[1] = changed_path;

    let report = ExternalArtifactRegistryRunner::default()
        .run(&config)
        .expect("run schema change registry")
        .model_card_lineage_report;
    let latest = report
        .records
        .iter()
        .find(|record| record.model_id == "ext-model-a" && record.model_version == "1.1.0")
        .expect("find changed lineage");
    assert_eq!(
        latest.lineage_status,
        ModelVersionLineageStatus::SchemaChanged
    );
}
