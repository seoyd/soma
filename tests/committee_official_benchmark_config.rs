use soma_zero::CommitteeOfficialBenchmarkConfig;

#[test]
fn official_benchmark_config_defaults_and_remote_rejection_work() {
    let cfg = CommitteeOfficialBenchmarkConfig::default();
    assert!(cfg.require_core_check);
    let remote = CommitteeOfficialBenchmarkConfig {
        scenario_pack_config_path: Some("https://example.com/pack.toml".to_string()),
        ..CommitteeOfficialBenchmarkConfig::default()
    };
    assert!(remote.validate().is_err());
}

#[test]
fn official_benchmark_config_toml_has_no_live_or_llm_fields() {
    let toml = CommitteeOfficialBenchmarkConfig::default()
        .to_toml_string()
        .expect("toml");
    assert!(!toml.contains("broker"));
    assert!(!toml.contains("account"));
    assert!(!toml.contains("llm"));
}
