mod common;
#[path = "support/sprint65_support.rs"]
mod sprint65_support;

use soma_zero::{
    ExternalModelArtifactKind, ExternalModelArtifactRegistry, ExternalModelResearchOpsRunner,
};

fn score_for<'a>(
    scores: &'a [soma_zero::ArtifactCompletenessScore],
    model_id: &str,
    model_version: &str,
) -> &'a soma_zero::ArtifactCompletenessScore {
    scores
        .iter()
        .find(|score| score.model_id == model_id && score.model_version == model_version)
        .expect("artifact completeness score")
}

#[test]
fn complete_artifacts_can_score_one() {
    let mut config = sprint65_support::research_ops_config_from_example(
        "soma_artifact_completeness.toml",
        "artifact-completeness-complete",
    );
    let mut registry: ExternalModelArtifactRegistry =
        sprint65_support::read_json(&config.external_artifact_registry_paths[0]);
    let mut sample = registry
        .entries
        .iter()
        .find(|entry| entry.artifact_kind == ExternalModelArtifactKind::EvaluationReport)
        .expect("sample entry")
        .clone();
    sample.entry_id = "vs-trinity:ext-model-a:1.1.0".to_string();
    sample.model_id = "ext-model-a".to_string();
    sample.model_version = "1.1.0".to_string();
    sample.artifact_kind = ExternalModelArtifactKind::VsTrinityReport;
    sample.artifact_path = "examples/sprint65_data/vs_trinity_a_v2.json".to_string();
    registry.entries.push(sample);
    config.external_artifact_registry_paths[0] = sprint65_support::write_support_json(
        "artifact-completeness-complete",
        "registry.json",
        &registry,
    );

    let scores = ExternalModelResearchOpsRunner::default()
        .run_artifact_completeness(&config)
        .expect("run completeness");
    let target = score_for(&scores, "ext-model-a", "1.1.0");
    assert!((target.completeness_ratio - 1.0).abs() < f64::EPSILON);
}

#[test]
fn missing_core_artifacts_lower_score() {
    let mut config = sprint65_support::research_ops_config_from_example(
        "soma_artifact_completeness.toml",
        "artifact-completeness-missing-core",
    );
    let mut registry: ExternalModelArtifactRegistry =
        sprint65_support::read_json(&config.external_artifact_registry_paths[0]);
    registry.entries.retain(|entry| {
        !(entry.model_id == "ext-model-a"
            && entry.model_version == "1.1.0"
            && matches!(
                entry.artifact_kind,
                ExternalModelArtifactKind::ModelCard
                    | ExternalModelArtifactKind::PredictionCsv
                    | ExternalModelArtifactKind::EvaluationReport
            ))
    });
    config.external_artifact_registry_paths[0] = sprint65_support::write_support_json(
        "artifact-completeness-missing-core",
        "registry.json",
        &registry,
    );
    let scores = ExternalModelResearchOpsRunner::default()
        .run_artifact_completeness(&config)
        .expect("run missing core completeness");
    let target = score_for(&scores, "ext-model-a", "1.1.0");
    assert!(!target.has_model_card);
    assert!(!target.has_prediction_csv);
    assert!(!target.has_evaluation_report);
    assert!(target.completeness_ratio < 1.0);
}

#[test]
fn missing_ablation_promotion_and_contract_lower_score() {
    let mut config = sprint65_support::research_ops_config_from_example(
        "soma_artifact_completeness.toml",
        "artifact-completeness-aux-missing",
    );
    let mut registry: ExternalModelArtifactRegistry =
        sprint65_support::read_json(&config.external_artifact_registry_paths[0]);
    registry.entries.retain(|entry| {
        !matches!(
            entry.artifact_kind,
            ExternalModelArtifactKind::AblationReport
                | ExternalModelArtifactKind::PromotionGateReport
                | ExternalModelArtifactKind::Mamba3FinContract
        )
    });
    config.external_artifact_registry_paths[0] = sprint65_support::write_support_json(
        "artifact-completeness-aux-missing",
        "registry.json",
        &registry,
    );
    let scores = ExternalModelResearchOpsRunner::default()
        .run_artifact_completeness(&config)
        .expect("run missing aux completeness");
    let target = score_for(&scores, "ext-model-a", "1.1.0");
    assert!(!target.has_ablation_report);
    assert!(!target.has_promotion_gate);
    assert!(!target.has_mamba_contract_if_applicable);
    assert!(target.completeness_ratio < 1.0);
}

#[test]
fn artifact_completeness_is_deterministic() {
    let first = sprint65_support::research_ops_config_from_example(
        "soma_artifact_completeness.toml",
        "artifact-completeness-determinism-first",
    );
    let second = sprint65_support::research_ops_config_from_example(
        "soma_artifact_completeness.toml",
        "artifact-completeness-determinism-second",
    );
    let first_scores = ExternalModelResearchOpsRunner::default()
        .run_artifact_completeness(&first)
        .expect("run first completeness");
    let second_scores = ExternalModelResearchOpsRunner::default()
        .run_artifact_completeness(&second)
        .expect("run second completeness");
    assert_eq!(first_scores, second_scores);
}
