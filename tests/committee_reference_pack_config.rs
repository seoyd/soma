mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::CommitteeReferencePackConfig;

#[test]
fn reference_pack_config_has_safe_defaults_and_round_trips() {
    let config = CommitteeReferencePackConfig::default();
    assert_eq!(config.max_rows, 100);
    assert_eq!(config.max_symbols, 5);
    assert!(config.require_exact_symbol_match);
    assert!(config.require_exact_horizon_match);
    assert!(config.require_no_lookahead_safe);
    assert!(!config.allow_yfinance_research);
    assert!(!config.allow_fixture);
    assert!(!config.allow_estimated_references);

    let parsed =
        CommitteeReferencePackConfig::from_toml_str(&config.to_toml_string().expect("toml"))
            .expect("parse toml");
    assert_eq!(parsed, config);
}

#[test]
fn reference_pack_config_rejects_remote_paths_and_bounds() {
    let mut config = CommitteeReferencePackConfig::default();
    config.scenario_pack_paths = vec!["https://example.com/pack.toml".to_string()];
    assert!(config.validate().unwrap_err().contains("local"));

    let mut rows = CommitteeReferencePackConfig::default();
    rows.max_rows = 101;
    assert!(rows.validate().unwrap_err().contains("max_rows"));

    let mut symbols = CommitteeReferencePackConfig::default();
    symbols.max_symbols = 6;
    assert!(symbols.validate().unwrap_err().contains("max_symbols"));

    let mut bytes = CommitteeReferencePackConfig::default();
    bytes.max_bytes = 5_000_001;
    assert!(bytes.validate().unwrap_err().contains("max_bytes"));
}
