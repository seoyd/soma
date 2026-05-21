use soma_zero::{
    FutureWindowExtensionJob, FutureWindowExtensionJobKind, FutureWindowExtensionJobStatus,
    ProviderKind, ReasonCode,
};

#[test]
fn extension_job_fingerprint_and_flags_are_stable() {
    let job = FutureWindowExtensionJob {
        job_id: "job-a".to_string(),
        job_kind: FutureWindowExtensionJobKind::AlphaVantageCompactFutureWindowCollect,
        provider_kind: Some(ProviderKind::AlphaVantage),
        market: "USEquity".to_string(),
        venue: Some("NASDAQ".to_string()),
        symbol: "AAPL".to_string(),
        timeframe: "1d".to_string(),
        horizon_bars: 3,
        required_start_timestamp_ms: 1,
        required_end_timestamp_ms: 2,
        max_rows: 10,
        max_requests: 2,
        expected_output_csv: Some("examples/sprint46_data/aapl_1d_extended.csv".to_string()),
        expected_provenance: None,
        expected_preflight: None,
        status: FutureWindowExtensionJobStatus::ReadyToRun,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    assert_eq!(job.fingerprint(), job.fingerprint());
    assert!(job.is_provider_job());
    assert!(job.is_runnable());
}
