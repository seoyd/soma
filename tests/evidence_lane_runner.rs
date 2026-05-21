use soma_zero::{
    EvidenceLaneKind, EvidenceLaneRunner, EvidenceLaneStatus, EvidencePlanBuilder,
    ExecutableEvidencePlanConfig, ExplicitEvidenceLaneConfig,
};

#[test]
fn skipped_missing_auth_lane_does_not_attempt_collection() {
    let plan = EvidencePlanBuilder::default()
        .from_explicit_lanes(&ExecutableEvidencePlanConfig {
            output_root: "target/sprint10-tests/runner-missing-auth".to_string(),
            allow_yfinance_research: false,
            explicit_lanes: vec![ExplicitEvidenceLaneConfig {
                lane_kind: EvidenceLaneKind::USEquityEodEvidence,
                provider: "alphavantage".to_string(),
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

    let report = EvidenceLaneRunner::default()
        .run_lane(&plan.lanes[0], &ExecutableEvidencePlanConfig::default());
    assert_eq!(report.lane_status, EvidenceLaneStatus::SkippedMissingAuth);
    assert!(report.collection_report.is_none());
}

#[test]
fn preflight_failure_blocks_benchmark() {
    let mut plan = EvidencePlanBuilder::default()
        .from_explicit_lanes(&ExecutableEvidencePlanConfig {
            output_root: "target/sprint10-tests/runner-preflight".to_string(),
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
    plan.lanes[0].simulate_preflight_failure = true;

    let report = EvidenceLaneRunner::default()
        .run_lane(&plan.lanes[0], &ExecutableEvidencePlanConfig::default());
    assert_eq!(report.lane_status, EvidenceLaneStatus::FailedPreflight);
    assert!(report.benchmark_report.is_none());
}

#[test]
fn diagnostic_lane_emits_diagnostic_only_report() {
    let plan = EvidencePlanBuilder::default()
        .from_explicit_lanes(&ExecutableEvidencePlanConfig {
            output_root: "target/sprint10-tests/runner-diagnostic".to_string(),
            allow_yfinance_research: false,
            explicit_lanes: vec![ExplicitEvidenceLaneConfig {
                lane_kind: EvidenceLaneKind::DiagnosticsOnly,
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

    let report = EvidenceLaneRunner::default()
        .run_lane(&plan.lanes[0], &ExecutableEvidencePlanConfig::default());
    assert_eq!(report.lane_status, EvidenceLaneStatus::DiagnosticOnly);
    assert!(report.benchmark_report.is_none());
}
