use soma_zero::OfficialAiBenchmarkConfig;
use std::path::PathBuf;

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

#[test]
fn benchmark_config_defaults_are_conservative() {
    let config = OfficialAiBenchmarkConfig::default();
    assert!(!config.run_python_training);
    assert!(config.run_baseline_eval);
    assert!(config.run_dataset_export);
    assert!(config.strict_schema_validation);
}

#[test]
fn benchmark_config_rejects_remote_paths() {
    let config = OfficialAiBenchmarkConfig {
        official_collection_report_path: Some("https://example.com/report.json".to_string()),
        ..OfficialAiBenchmarkConfig::default()
    };
    assert!(
        config
            .validate_local_paths()
            .contains(&soma_zero::ReasonCode::LocalPathRejected)
    );
}

#[test]
fn benchmark_config_serialization_has_no_broker_or_account_fields() {
    let serialized = OfficialAiBenchmarkConfig::default()
        .to_toml_string()
        .expect("serialize benchmark config");
    assert!(!serialized.contains("broker"));
    assert!(!serialized.contains("account"));
    assert!(!serialized.contains("order"));
    assert!(!serialized.contains("llm"));
}

#[test]
fn sprint21_benchmark_examples_parse() {
    for path in [
        example_path("soma_ai_benchmark_upbit_only.toml"),
        example_path("soma_ai_benchmark_official_compact.toml"),
        example_path("soma_ai_benchmark_existing_predictions.toml"),
    ] {
        let config =
            OfficialAiBenchmarkConfig::from_toml_path(&path).expect("parse benchmark example");
        assert!(!config.benchmark_id.is_empty());
    }
}
