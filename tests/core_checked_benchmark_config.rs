use soma_zero::{CoreCheckedBenchmarkConfig, CoreReadinessStatus, ReasonCode};

#[test]
fn core_checked_benchmark_config_defaults_are_conservative() {
    let config = CoreCheckedBenchmarkConfig::default();

    assert_eq!(config.benchmark_id, "core-checked-benchmark");
    assert!(config.require_core_ready);
    assert!(config.run_baseline_eval);
    assert!(!config.run_python_training);
    assert!(config.strict_schema_validation);
    assert_eq!(config.min_ready_official_datasets, 1);
    assert_eq!(
        config.allowed_core_statuses,
        vec![
            CoreReadinessStatus::ReadyForMoreOfficialEvidence,
            CoreReadinessStatus::ReadyForExternalModelPrototype,
            CoreReadinessStatus::ReadyForSequenceDatasetBuild,
        ]
    );
}

#[test]
fn core_checked_benchmark_config_rejects_remote_paths() {
    let config = CoreCheckedBenchmarkConfig {
        existing_prediction_csv: Some("https://example.com/predictions.csv".to_string()),
        ..CoreCheckedBenchmarkConfig::default()
    };

    assert!(
        config
            .validate_local_paths()
            .contains(&ReasonCode::RemotePathRejected)
    );
}

#[test]
fn core_checked_benchmark_config_surface_stays_research_only() {
    let toml = CoreCheckedBenchmarkConfig::default()
        .to_toml_string()
        .expect("serialize config");

    assert!(!toml.contains("broker"));
    assert!(!toml.contains("account"));
    assert!(!toml.contains("llm"));
    assert!(!toml.contains("live"));
}
