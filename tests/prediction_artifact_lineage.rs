mod common;
#[path = "support/sprint64_support.rs"]
mod sprint64_support;

use std::fs;

use soma_zero::{
    ExternalArtifactRegistryRunner, ExternalModelCardV2, ExternalPredictionImportV2Report,
    PredictionArtifactLineageStatus,
};

#[test]
fn valid_prediction_lineage_passes() {
    let config = sprint64_support::registry_config_from_example(
        "soma_external_artifact_registry.toml",
        "prediction-lineage-valid",
    );
    let report = ExternalArtifactRegistryRunner::default()
        .run(&config)
        .expect("run prediction lineage bundle")
        .prediction_artifact_lineage_report;
    assert_eq!(report.model_id, "ext-model-a");
    assert_eq!(
        report.lineage_status,
        PredictionArtifactLineageStatus::PredictionLineageValid
    );
    assert_eq!(report.coverage_ratio, Some(1.0));
}

#[test]
fn weak_coverage_and_schema_mismatch_are_detected() {
    let mut weak_config = sprint64_support::registry_config_from_example(
        "soma_external_artifact_registry.toml",
        "prediction-lineage-weak",
    );
    let import_path = sprint64_support::absolutize("examples/sprint64_data/import_a_v1.json");
    let mut import: ExternalPredictionImportV2Report =
        serde_json::from_str(&fs::read_to_string(import_path).expect("read import report"))
            .expect("parse import report");
    import.total_prediction_rows = 6;
    import.valid_prediction_rows = 3;
    let weak_import = sprint64_support::write_support_json(
        "prediction-lineage-weak",
        "import_a_v1.json",
        &import,
    );
    weak_config.external_prediction_import_report_paths[0] = weak_import;
    let weak_report = ExternalArtifactRegistryRunner::default()
        .run(&weak_config)
        .expect("run weak coverage registry")
        .prediction_artifact_lineage_report;
    assert_eq!(
        weak_report.lineage_status,
        PredictionArtifactLineageStatus::CoverageWeak
    );

    let mut mismatch_config = sprint64_support::registry_config_from_example(
        "soma_external_artifact_registry.toml",
        "prediction-lineage-mismatch",
    );
    let card_path = sprint64_support::absolutize("examples/sprint64_data/model_card_a_v1.json");
    let mut card: ExternalModelCardV2 =
        serde_json::from_str(&fs::read_to_string(card_path).expect("read model card"))
            .expect("parse model card");
    card.feature_schema_hash = "mismatch-feature".to_string();
    let mismatch_card = sprint64_support::write_support_json(
        "prediction-lineage-mismatch",
        "model_card_a_v1.json",
        &card,
    );
    mismatch_config.external_model_card_paths[0] = mismatch_card;
    let mismatch_report = ExternalArtifactRegistryRunner::default()
        .run(&mismatch_config)
        .expect("run mismatch registry")
        .prediction_artifact_lineage_report;
    assert_eq!(
        mismatch_report.lineage_status,
        PredictionArtifactLineageStatus::FeatureSchemaMismatch
    );
}
