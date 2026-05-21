use soma_zero::OfficialEvidenceReplicationConfig;

#[test]
fn config_defaults_are_conservative_and_constructible() {
    let config = OfficialEvidenceReplicationConfig::default();
    assert!(config.require_non_crypto_official);
    assert!(config.require_provenance);
    assert!(config.require_preflight);
    assert!(config.require_local_candles);
    assert!(config.require_outcome_links);
    assert!(!config.allow_yfinance_research);
    assert!(!config.allow_fixture);
    assert!(!config.allow_controlled_fixture);
    assert_eq!(config.max_rows, 500);
    assert_eq!(config.max_symbols, 5);
}

#[test]
fn config_rejects_remote_paths_and_scope_overrides() {
    let remote = OfficialEvidenceReplicationConfig {
        official_canonical_csv_paths: vec!["https://example.com/aapl.csv".to_string()],
        ..OfficialEvidenceReplicationConfig::default()
    };
    let too_many_rows = OfficialEvidenceReplicationConfig {
        max_rows: 501,
        ..OfficialEvidenceReplicationConfig::default()
    };
    let too_many_symbols = OfficialEvidenceReplicationConfig {
        max_symbols: 6,
        ..OfficialEvidenceReplicationConfig::default()
    };
    let too_many_bytes = OfficialEvidenceReplicationConfig {
        max_bytes: 5_000_001,
        ..OfficialEvidenceReplicationConfig::default()
    };
    assert!(remote.validate().is_err());
    assert!(too_many_rows.validate().is_err());
    assert!(too_many_symbols.validate().is_err());
    assert!(too_many_bytes.validate().is_err());
}

#[test]
fn config_denies_unknown_live_llm_broker_fields() {
    let result = OfficialEvidenceReplicationConfig::from_toml_str(
        r#"
replication_id = "bad"
output_root = "target/soma_official_replication"
broker = "forbidden"
"#,
    );
    assert!(result.is_err());

    let llm = OfficialEvidenceReplicationConfig::from_toml_str(
        r#"
replication_id = "bad"
output_root = "target/soma_official_replication"
runtime_llm = true
"#,
    );
    assert!(llm.is_err());
}
