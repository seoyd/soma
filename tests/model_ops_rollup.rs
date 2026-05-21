mod common;
#[path = "support/sprint67_support.rs"]
mod sprint67_support;

use soma_zero::{ModelOpsRollupConfig, ModelOpsRollupRunner};

#[test]
fn rollup_config_defaults_are_local_only_and_forbid_runtime_fields() {
    let config = ModelOpsRollupConfig::default();
    assert!(config.deduplicate_by_model_version);
    assert!(config.require_model_id);
    assert!(config.require_model_version);
    let encoded = toml::to_string(&config).expect("serialize rollup config");
    for forbidden in ["live", "runtime", "training", "broker", "order", "account"] {
        assert!(
            !encoded.contains(&format!("{forbidden}_")),
            "unexpected forbidden config field: {forbidden}"
        );
    }

    let mut bad = config.clone();
    bad.model_review_closure_paths = vec!["https://example.com/closure.json".to_string()];
    assert!(bad.validate().is_err());
}

#[test]
fn rollup_runner_deduplicates_raw_entries_by_model_version() {
    let bundle = sprint67_support::run_rollup("soma_model_ops_rollup.toml", "rollup-dedup");
    assert_eq!(bundle.model_version_summary_cards.len(), 4);
    assert_eq!(
        bundle
            .model_version_summary_cards
            .iter()
            .filter(|card| card.display_name == "ext-model-b:1.0.0")
            .count(),
        1
    );
}

#[test]
fn rollup_limits_are_enforced() {
    let mut config =
        sprint67_support::rollup_config_from_example("soma_model_ops_rollup.toml", "rollup-limits");
    config.max_models = 1;
    assert!(ModelOpsRollupRunner::default().run(&config).is_err());

    let mut config = sprint67_support::rollup_config_from_example(
        "soma_model_ops_rollup.toml",
        "rollup-version-limit",
    );
    config.max_versions = 2;
    assert!(ModelOpsRollupRunner::default().run(&config).is_err());

    let mut config = sprint67_support::rollup_config_from_example(
        "soma_model_ops_rollup.toml",
        "rollup-artifact-limit",
    );
    config.max_artifacts = 1;
    assert!(ModelOpsRollupRunner::default().run(&config).is_err());

    let mut config =
        sprint67_support::rollup_config_from_example("soma_model_ops_rollup.toml", "rollup-bytes");
    config.max_bytes = 64;
    assert!(ModelOpsRollupRunner::default().run(&config).is_err());
}
