use soma_zero::OfficialCandleCoveragePackConfig;

#[test]
fn official_candle_pack_config_defaults_are_conservative() {
    let config = OfficialCandleCoveragePackConfig::default();
    assert!(config.require_official_source);
    assert!(config.require_provenance);
    assert!(config.require_preflight);
    assert!(!config.require_manifest);
    assert!(config.allow_crypto_only);
    assert!(!config.allow_controlled_fixture);
    assert!(!config.allow_yfinance_research);
    assert!(!config.allow_fixture);
    assert!(!config.allow_synthetic_test);
    assert!(!config.allow_timeframe_aggregation);
    assert!(config.allow_timestamp_tolerance);
    assert!(config.require_no_lookahead_safe);
}

#[test]
fn official_candle_pack_config_rejects_remote_paths_and_out_of_bounds_limits() {
    let remote = OfficialCandleCoveragePackConfig {
        canonical_csv_paths: vec!["https://example.com/aapl.csv".to_string()],
        ..OfficialCandleCoveragePackConfig::default()
    };
    assert!(remote.validate().is_err());

    assert!(
        OfficialCandleCoveragePackConfig {
            max_rows: 1001,
            ..OfficialCandleCoveragePackConfig::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        OfficialCandleCoveragePackConfig {
            max_symbols: 6,
            ..OfficialCandleCoveragePackConfig::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        OfficialCandleCoveragePackConfig {
            max_timeframes: 6,
            ..OfficialCandleCoveragePackConfig::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        OfficialCandleCoveragePackConfig {
            max_bytes: 5_000_001,
            ..OfficialCandleCoveragePackConfig::default()
        }
        .validate()
        .is_err()
    );
}

#[test]
fn official_candle_pack_config_has_no_broker_order_account_or_llm_fields() {
    for invalid in [
        "pack_id = 'x'\nbroker = 'no'\n",
        "pack_id = 'x'\norder = 'no'\n",
        "pack_id = 'x'\naccount = 'no'\n",
        "pack_id = 'x'\nllm = 'no'\n",
        "pack_id = 'x'\nmamba = 'no'\n",
        "pack_id = 'x'\npersona_expansion = true\n",
    ] {
        assert!(OfficialCandleCoveragePackConfig::from_toml_str(invalid).is_err());
    }
}
