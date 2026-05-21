mod common;
#[path = "support/sprint64_support.rs"]
mod sprint64_support;

use soma_zero::{
    ExternalArtifactRegistryRunner, ExternalModelArtifactRegistryConfig,
    ExternalModelArtifactRegistryStatus,
};

#[test]
fn registry_config_defaults_and_remote_paths_are_rejected() {
    let config = ExternalModelArtifactRegistryConfig::default();
    assert!(config.require_model_card);
    assert!(config.require_evaluation_report);
    assert!(config.require_same_dataset_fingerprint_for_comparison);
    assert!(config.require_same_feature_schema_hash_for_comparison);
    assert!(config.require_same_label_manifest_hash_for_comparison);

    let mut bad = config.clone();
    bad.registry_id = "remote".to_string();
    bad.sequence_export_manifest_paths = vec!["https://example.com/manifest.json".to_string()];
    assert!(bad.validate().is_err());
}

#[test]
fn registry_bundle_builds_and_limits_are_enforced() {
    let config = sprint64_support::registry_config_from_example(
        "soma_external_artifact_registry.toml",
        "registry-build",
    );
    let bundle = ExternalArtifactRegistryRunner::default()
        .run(&config)
        .expect("run registry bundle");
    assert_eq!(
        bundle.artifact_registry.registry_status,
        ExternalModelArtifactRegistryStatus::RegistryReadyWithWarnings
    );
    assert!(
        bundle
            .control_tower_external_leaderboard_panel_summary
            .is_some()
    );
    assert!(bundle.storage_report.within_budget);

    let mut max_models = config.clone();
    max_models.max_models = 1;
    assert!(
        ExternalArtifactRegistryRunner::default()
            .run(&max_models)
            .is_err()
    );

    let mut max_versions = config.clone();
    max_versions.max_versions_per_model = 1;
    assert!(
        ExternalArtifactRegistryRunner::default()
            .run(&max_versions)
            .is_err()
    );

    let mut max_artifacts = config.clone();
    max_artifacts.max_artifacts = 2;
    assert!(
        ExternalArtifactRegistryRunner::default()
            .run(&max_artifacts)
            .is_err()
    );

    let mut tiny_storage = config.clone();
    tiny_storage.max_bytes = 1;
    let tiny_bundle = ExternalArtifactRegistryRunner::default()
        .run(&tiny_storage)
        .expect("run tiny registry bundle");
    assert!(!tiny_bundle.storage_report.within_budget);
}
