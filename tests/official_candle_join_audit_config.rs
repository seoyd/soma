mod common;

use soma_zero::OfficialCandleJoinAuditConfig;

#[test]
fn join_audit_config_defaults_are_safe_and_local_only() {
    let config = OfficialCandleJoinAuditConfig::default();
    assert!(config.allow_explicit_symbol_alias);
    assert!(config.allow_explicit_timeframe_alias);
    assert!(config.allow_explicit_timestamp_policy_map);
    assert!(config.allow_session_daily_alignment);
    assert!(config.allow_timestamp_tolerance);
    assert!(config.require_exact_horizon_match);
    assert!(config.require_no_lookahead_safe);
    assert!(config.require_official_source_for_official_ready);
}

#[test]
fn join_audit_config_rejects_remote_paths_and_out_of_bounds_limits() {
    let mut config = OfficialCandleJoinAuditConfig {
        output_root: "https://example.com/out".to_string(),
        ..OfficialCandleJoinAuditConfig::default()
    };
    assert!(config.validate().unwrap_err().contains("local"));
    config.output_root = common::output_dir("sprint44-audit-config")
        .display()
        .to_string();
    config.max_rows = 501;
    assert!(config.validate().unwrap_err().contains("max_rows"));
    config.max_rows = 500;
    config.max_symbols = 6;
    assert!(config.validate().unwrap_err().contains("max_symbols"));
    config.max_symbols = 5;
    config.max_bytes = 5_000_001;
    assert!(config.validate().unwrap_err().contains("max_bytes"));
}

#[test]
fn join_audit_config_denies_unknown_runtime_or_live_fields() {
    for invalid in [
        "audit_id = 'x'\nlive = true\n",
        "audit_id = 'x'\nbroker = 'no'\n",
        "audit_id = 'x'\naccount = 'no'\n",
        "audit_id = 'x'\nllm = true\n",
        "audit_id = 'x'\nmamba = true\n",
        "audit_id = 'x'\npersona_expansion = true\n",
    ] {
        assert!(OfficialCandleJoinAuditConfig::from_toml_str(invalid).is_err());
    }
}
