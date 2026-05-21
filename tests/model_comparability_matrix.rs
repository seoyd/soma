mod common;
#[path = "support/sprint65_support.rs"]
mod sprint65_support;

use soma_zero::{
    ConservativeExternalLeaderboard, ExternalModelArtifactKind, ExternalModelArtifactRegistry,
    ExternalModelResearchOpsRunner, ModelComparabilityDimension, ModelComparabilityMatrixStatus,
};

fn ext_model_a_only_config(output_name: &str) -> soma_zero::ExternalModelResearchOpsConfig {
    let mut config = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        output_name,
    );
    let mut registry: ExternalModelArtifactRegistry =
        sprint65_support::read_json(&config.external_artifact_registry_paths[0]);
    registry
        .entries
        .retain(|entry| entry.model_id == "ext-model-a" || entry.model_id == "registry");
    config.external_artifact_registry_paths[0] =
        sprint65_support::write_support_json(output_name, "registry.json", &registry);
    let mut leaderboard: ConservativeExternalLeaderboard =
        sprint65_support::read_json(&config.conservative_leaderboard_paths[0]);
    leaderboard
        .entries
        .retain(|entry| entry.model_id == "ext-model-a");
    leaderboard.eligible_entries = leaderboard.entries.len();
    leaderboard.blocked_entries = 0;
    config.conservative_leaderboard_paths[0] =
        sprint65_support::write_support_json(output_name, "leaderboard.json", &leaderboard);
    config.owner_model_review_paths[0] = sprint65_support::write_support_json(
        output_name,
        "owner_actions.json",
        &Vec::<serde_json::Value>::new(),
    );
    config
}

fn find_cell<'a>(
    matrix: &'a soma_zero::ModelComparabilityMatrix,
    model_id: &str,
    model_version: &str,
    dimension: ModelComparabilityDimension,
) -> &'a soma_zero::ModelComparabilityCell {
    matrix
        .cells
        .iter()
        .find(|cell| {
            cell.model_id == model_id
                && cell.model_version == model_version
                && cell.dimension == dimension
        })
        .expect("comparability cell")
}

#[test]
fn fully_comparable_models_pass() {
    let config = ext_model_a_only_config("comparability-fully-comparable");
    let matrix = ExternalModelResearchOpsRunner::default()
        .run_comparability_matrix(&config)
        .expect("run fully comparable matrix");
    assert_eq!(
        matrix.matrix_status,
        ModelComparabilityMatrixStatus::FullyComparable
    );
}

#[test]
fn dataset_and_feature_mismatches_fail_dimensions() {
    let mut config = ext_model_a_only_config("comparability-dataset-feature-mismatch");
    let mut registry: ExternalModelArtifactRegistry =
        sprint65_support::read_json(&config.external_artifact_registry_paths[0]);
    for entry in &mut registry.entries {
        if entry.model_id == "ext-model-a" && entry.model_version == "1.1.0" {
            entry.dataset_fingerprint = Some("dataset-mismatch".to_string());
            entry.feature_schema_hash = Some("schema-mismatch".to_string());
        }
    }
    config.external_artifact_registry_paths[0] = sprint65_support::write_support_json(
        "comparability-dataset-feature-mismatch",
        "registry.json",
        &registry,
    );
    let matrix = ExternalModelResearchOpsRunner::default()
        .run_comparability_matrix(&config)
        .expect("run mismatched matrix");
    assert!(
        !find_cell(
            &matrix,
            "ext-model-a",
            "1.1.0",
            ModelComparabilityDimension::DatasetFingerprint
        )
        .comparable
    );
    assert!(
        !find_cell(
            &matrix,
            "ext-model-a",
            "1.1.0",
            ModelComparabilityDimension::FeatureSchemaHash
        )
        .comparable
    );
}

#[test]
fn label_split_prediction_and_metric_mismatches_fail_dimensions() {
    let mut config = ext_model_a_only_config("comparability-label-split-prediction");
    let mut registry: ExternalModelArtifactRegistry =
        sprint65_support::read_json(&config.external_artifact_registry_paths[0]);
    for entry in &mut registry.entries {
        if entry.model_id == "ext-model-a" && entry.model_version == "1.1.0" {
            entry.label_manifest_hash = Some("label-mismatch".to_string());
            entry.split_policy = Some("Random".to_string());
            if entry.artifact_kind == ExternalModelArtifactKind::PredictionCsv {
                entry.prediction_schema_version = Some("v1".to_string());
            }
        }
    }
    config.external_artifact_registry_paths[0] = sprint65_support::write_support_json(
        "comparability-label-split-prediction",
        "registry.json",
        &registry,
    );
    let mut leaderboard: ConservativeExternalLeaderboard =
        sprint65_support::read_json(&config.conservative_leaderboard_paths[0]);
    if let Some(entry) = leaderboard
        .entries
        .iter_mut()
        .find(|entry| entry.model_id == "ext-model-a" && entry.model_version == "1.1.0")
    {
        entry.brier_score = None;
    }
    config.conservative_leaderboard_paths[0] = sprint65_support::write_support_json(
        "comparability-label-split-prediction",
        "leaderboard.json",
        &leaderboard,
    );
    let matrix = ExternalModelResearchOpsRunner::default()
        .run_comparability_matrix(&config)
        .expect("run label mismatch matrix");
    for dimension in [
        ModelComparabilityDimension::LabelManifestHash,
        ModelComparabilityDimension::SplitPolicy,
        ModelComparabilityDimension::PredictionSchemaVersion,
        ModelComparabilityDimension::EvaluationMetricAvailability,
    ] {
        assert!(
            !find_cell(&matrix, "ext-model-a", "1.1.0", dimension).comparable,
            "expected mismatch for {dimension:?}"
        );
    }
}

#[test]
fn comparability_matrix_is_deterministic() {
    let first = ext_model_a_only_config("comparability-determinism-first");
    let second = ext_model_a_only_config("comparability-determinism-second");
    let first_matrix = ExternalModelResearchOpsRunner::default()
        .run_comparability_matrix(&first)
        .expect("run first matrix");
    let second_matrix = ExternalModelResearchOpsRunner::default()
        .run_comparability_matrix(&second)
        .expect("run second matrix");
    assert_eq!(first_matrix, second_matrix);
}
