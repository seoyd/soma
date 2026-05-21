use soma_zero::OfficialCommitteeScenarioPackConfig;

#[test]
fn config_can_be_constructed_and_defaults_are_conservative() {
    let cfg = OfficialCommitteeScenarioPackConfig::default();
    assert!(cfg.require_provenance);
    assert!(cfg.require_preflight);
    assert!(!cfg.allow_yfinance_research);
    assert!(!cfg.allow_fixture);
}

#[test]
fn remote_paths_and_bounds_are_rejected() {
    let remote = OfficialCommitteeScenarioPackConfig {
        input_artifact_paths: vec!["https://example.com/rows.json".to_string()],
        ..OfficialCommitteeScenarioPackConfig::default()
    };
    let max_rows = OfficialCommitteeScenarioPackConfig {
        max_rows: 101,
        ..OfficialCommitteeScenarioPackConfig::default()
    };
    let max_symbols = OfficialCommitteeScenarioPackConfig {
        max_symbols: 51,
        ..OfficialCommitteeScenarioPackConfig::default()
    };
    let max_bytes = OfficialCommitteeScenarioPackConfig {
        max_bytes: 5_000_001,
        ..OfficialCommitteeScenarioPackConfig::default()
    };
    assert!(remote.validate().is_err());
    assert!(max_rows.validate().is_err());
    assert!(max_symbols.validate().is_err());
    assert!(max_bytes.validate().is_err());
}
