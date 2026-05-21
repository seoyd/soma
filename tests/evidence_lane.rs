use soma_zero::{
    EvidenceLaneKind, EvidencePlanBuilder, ExecutableEvidencePlanConfig,
    ExplicitEvidenceLaneConfig, ProviderKind,
};

#[test]
fn explicit_upbit_lane_becomes_crypto_intraday_runnable() {
    let plan = EvidencePlanBuilder::default()
        .from_explicit_lanes(&ExecutableEvidencePlanConfig {
            plan_id: "upbit".to_string(),
            output_root: "target/sprint10-tests/evidence-lane-upbit".to_string(),
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
        })
        .expect("plan");

    assert_eq!(plan.runnable_lanes.len(), 1);
    let lane = &plan.runnable_lanes[0];
    assert_eq!(lane.lane_kind, EvidenceLaneKind::CryptoIntradayEvidence);
    assert_eq!(lane.provider_kind, Some(ProviderKind::Upbit));
    assert!(lane.strategy_compatibility.compatible);
}

#[test]
fn yfinance_lane_stays_research_only() {
    let plan = EvidencePlanBuilder::default()
        .from_explicit_lanes(&ExecutableEvidencePlanConfig {
            plan_id: "yfinance".to_string(),
            output_root: "target/sprint10-tests/evidence-lane-yfinance".to_string(),
            explicit_lanes: vec![ExplicitEvidenceLaneConfig {
                lane_kind: EvidenceLaneKind::YFinanceResearchFallback,
                provider: "yfinance".to_string(),
                symbols: vec!["AAPL".to_string()],
                enabled: true,
                output_subdir: None,
                max_rows: None,
                max_requests: None,
                allow_full_history: false,
                allow_all_symbols: false,
                reason_codes: vec![],
            }],
            ..ExecutableEvidencePlanConfig::default()
        })
        .expect("plan");

    let lane = &plan.runnable_lanes[0];
    assert_eq!(lane.lane_kind, EvidenceLaneKind::YFinanceResearchFallback);
    assert!(!lane.official_readiness_eligible());
}
