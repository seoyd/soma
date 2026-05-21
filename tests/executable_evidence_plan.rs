use soma_zero::{
    EvidenceLaneKind, EvidencePlanBuilder, ExecutableEvidencePlanConfig, ExplicitEvidenceLaneConfig,
};

#[test]
fn executable_plan_config_rejects_remote_paths_and_all_symbol_scope() {
    let remote = ExecutableEvidencePlanConfig {
        output_root: "https://example.com/out".to_string(),
        ..ExecutableEvidencePlanConfig::default()
    };
    assert!(remote.validate().is_err());

    let all_symbol = ExecutableEvidencePlanConfig {
        explicit_lanes: vec![ExplicitEvidenceLaneConfig {
            lane_kind: EvidenceLaneKind::CryptoIntradayEvidence,
            provider: "upbit".to_string(),
            symbols: vec!["*".to_string()],
            enabled: true,
            output_subdir: None,
            max_rows: None,
            max_requests: None,
            allow_full_history: false,
            allow_all_symbols: false,
            reason_codes: vec![],
        }],
        ..ExecutableEvidencePlanConfig::default()
    };
    assert!(all_symbol.validate().is_err());
}

#[test]
fn explicit_plan_build_is_deterministic() {
    let config = ExecutableEvidencePlanConfig {
        plan_id: "deterministic-explicit".to_string(),
        output_root: "target/sprint10-tests/executable-deterministic".to_string(),
        allow_yfinance_research: false,
        explicit_lanes: vec![ExplicitEvidenceLaneConfig {
            lane_kind: EvidenceLaneKind::CryptoIntradayEvidence,
            provider: "upbit".to_string(),
            symbols: vec!["BTC-KRW".to_string()],
            enabled: true,
            output_subdir: None,
            max_rows: None,
            max_requests: None,
            allow_full_history: false,
            allow_all_symbols: false,
            reason_codes: vec![],
        }],
        ..ExecutableEvidencePlanConfig::default()
    };
    let first = EvidencePlanBuilder::default()
        .from_explicit_lanes(&config)
        .expect("plan")
        .to_json_string()
        .expect("json");
    let second = EvidencePlanBuilder::default()
        .from_explicit_lanes(&config)
        .expect("plan")
        .to_json_string()
        .expect("json");
    assert_eq!(first, second);
}
