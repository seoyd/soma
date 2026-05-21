mod common;
#[path = "support/sprint65_support.rs"]
mod sprint65_support;

use soma_zero::{
    ExternalModelArtifactKind, ExternalModelArtifactRegistry, ExternalModelResearchOpsConfig,
    ExternalModelResearchOpsRunner, ExternalModelResearchOpsStatus,
};

#[test]
fn config_defaults_and_remote_paths_are_rejected() {
    let config = ExternalModelResearchOpsConfig::default();
    assert!(config.enable_owner_model_review);
    assert!(config.enable_watchlist);
    assert!(config.enable_retirement_policy);
    assert!(config.enable_control_tower_model_ops_panel);

    let encoded = toml::to_string(&config).expect("serialize config");
    for forbidden in ["runtime", "training", "broker", "order", "account"] {
        assert!(
            !encoded.contains(forbidden),
            "unexpected forbidden field marker: {forbidden}"
        );
    }

    let mut bad = config.clone();
    bad.external_artifact_registry_paths = vec!["https://example.com/registry.json".to_string()];
    assert!(bad.validate().is_err());
}

#[test]
fn runner_builds_bundle_and_limits_are_enforced() {
    let config = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "research-ops-build",
    );
    let bundle = ExternalModelResearchOpsRunner::default()
        .run(&config)
        .expect("run research ops");
    assert_eq!(
        bundle.external_model_research_ops_report.final_status,
        ExternalModelResearchOpsStatus::NeedMorePredictionHistory
    );
    assert!(bundle.control_tower_model_ops_panel_summary.is_some());
    assert!(bundle.storage_report.within_budget);

    let mut max_models = config.clone();
    max_models.max_models = 1;
    assert!(
        ExternalModelResearchOpsRunner::default()
            .run(&max_models)
            .is_err()
    );

    let mut max_versions = config.clone();
    max_versions.max_versions = 1;
    assert!(
        ExternalModelResearchOpsRunner::default()
            .run(&max_versions)
            .is_err()
    );

    let mut max_review_items = config.clone();
    max_review_items.max_review_items = 1;
    assert!(
        ExternalModelResearchOpsRunner::default()
            .run(&max_review_items)
            .is_err()
    );

    let mut tiny_storage = config.clone();
    tiny_storage.max_bytes = 1;
    let tiny_bundle = ExternalModelResearchOpsRunner::default()
        .run(&tiny_storage)
        .expect("run tiny storage research ops");
    assert!(!tiny_bundle.storage_report.within_budget);
}

#[test]
fn runner_surfaces_need_better_coverage_when_critical_artifacts_are_missing() {
    let mut config = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "research-ops-need-better-coverage",
    );
    let mut registry: ExternalModelArtifactRegistry =
        sprint65_support::read_json(&config.external_artifact_registry_paths[0]);
    registry.entries.retain(|entry| {
        !(entry.model_id == "ext-model-a"
            && entry.model_version == "1.1.0"
            && entry.artifact_kind == ExternalModelArtifactKind::EvaluationReport)
    });
    config.external_artifact_registry_paths[0] = sprint65_support::write_support_json(
        "research-ops-need-better-coverage",
        "external_registry_summary.json",
        &registry,
    );

    let bundle = ExternalModelResearchOpsRunner::default()
        .run(&config)
        .expect("run coverage constrained research ops");
    assert_eq!(
        bundle.external_model_research_ops_report.final_status,
        ExternalModelResearchOpsStatus::NeedBetterCoverage
    );
}
