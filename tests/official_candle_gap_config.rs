mod common;

use soma_zero::OfficialCandleGapConfig;

#[test]
fn official_candle_gap_config_defaults_are_safe() {
    let config = OfficialCandleGapConfig::default();
    assert!(config.require_non_crypto_official);
    assert!(config.allow_crypto_only);
    assert!(config.allow_controlled_diagnostic);
    assert!(!config.allow_yfinance_research);
    assert!(!config.allow_fixture);
}

#[test]
fn official_candle_gap_config_rejects_remote_paths_and_bounds() {
    let mut config = OfficialCandleGapConfig {
        output_root: "https://example.com/out".to_string(),
        ..OfficialCandleGapConfig::default()
    };
    assert!(config.validate().unwrap_err().contains("local"));

    config.output_root = common::output_dir("gap-config").display().to_string();
    config.max_gaps = 101;
    assert!(config.validate().unwrap_err().contains("max_gaps"));
    config.max_gaps = 100;
    config.max_symbols = 6;
    assert!(config.validate().unwrap_err().contains("max_symbols"));
    config.max_symbols = 5;
    config.max_timeframes = 6;
    assert!(config.validate().unwrap_err().contains("max_timeframes"));
    config.max_timeframes = 5;
    config.max_bytes = 5_000_001;
    assert!(config.validate().unwrap_err().contains("max_bytes"));
}

#[test]
fn official_candle_gap_config_denies_unknown_fields() {
    let err = OfficialCandleGapConfig::from_toml_str(
        r#"
        gap_id = "gap"
        llm = true
        "#,
    )
    .unwrap_err();
    assert!(err.contains("unknown field"));
}
