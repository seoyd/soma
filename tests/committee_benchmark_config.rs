use soma_zero::CommitteeBenchmarkConfig;

#[test]
fn benchmark_config_can_be_constructed_and_rejects_remote_paths() {
    let cfg = CommitteeBenchmarkConfig::default();
    assert!(cfg.require_core_check);
    let remote = CommitteeBenchmarkConfig {
        materialization_config_path: Some("https://example.com/materialize.toml".to_string()),
        ..CommitteeBenchmarkConfig::default()
    };
    let max = CommitteeBenchmarkConfig {
        max_decisions: 51,
        ..CommitteeBenchmarkConfig::default()
    };
    assert!(remote.validate().is_err());
    assert!(max.validate().is_err());
}

#[test]
fn benchmark_config_toml_has_no_live_or_llm_fields() {
    let toml = CommitteeBenchmarkConfig::default()
        .to_toml_string()
        .expect("toml");
    assert!(!toml.contains("broker"));
    assert!(!toml.contains("account"));
    assert!(!toml.contains("llm"));
}
