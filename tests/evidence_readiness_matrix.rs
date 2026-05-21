use soma_zero::{
    EvidenceLaneKind, ExecutableEvidencePlanConfig, ExplicitEvidenceLaneConfig,
    ProviderRealityEvidenceExecutor, ReadinessCellStatus,
};

#[test]
fn readiness_matrix_marks_yfinance_as_not_official() {
    let report = ProviderRealityEvidenceExecutor::default()
        .run(&ExecutableEvidencePlanConfig {
            output_root: "target/sprint10-tests/matrix-yfinance".to_string(),
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
        .expect("report");

    let cell = &report.readiness_matrix.cells[0];
    assert_eq!(cell.status, ReadinessCellStatus::ResearchOnly);
    assert!(!cell.official_readiness_eligible);
}

#[test]
fn readiness_matrix_evaluated_cell_includes_outcome_count() {
    let report = ProviderRealityEvidenceExecutor::default()
        .run(&ExecutableEvidencePlanConfig {
            output_root: "target/sprint10-tests/matrix-upbit".to_string(),
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
        .expect("report");

    let cell = &report.readiness_matrix.cells[0];
    assert_eq!(cell.status, ReadinessCellStatus::Evaluated);
    assert!(cell.outcome_count > 0);
}
