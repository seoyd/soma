mod common;
#[path = "support/sprint46_support.rs"]
mod sprint46_support;

use soma_zero::{
    FutureWindowExtensionJob, FutureWindowExtensionJobKind, FutureWindowExtensionJobStatus,
    FutureWindowGapKind, FutureWindowRequirementRunner, FutureWindowRequirementStatus,
    ProviderKind, ReasonCode,
};

#[test]
fn future_window_missing_bars_are_explicit() {
    let report = FutureWindowRequirementRunner::default()
        .run(&sprint46_support::future_window_short_config(
            "future-window-short-suite",
        ))
        .expect("future window report");
    assert_eq!(report.items.len(), 1);
    assert_eq!(
        report.items[0].gap_kind,
        FutureWindowGapKind::MissingFutureBars
    );
    assert_eq!(report.items[0].missing_future_bars, 1);
    assert_eq!(report.rows_missing_future_window, 1);
    assert!(matches!(
        report.requirement_status,
        FutureWindowRequirementStatus::NeedOfficialCandleExtension
            | FutureWindowRequirementStatus::NeedLongerFutureWindow
    ));
}

#[test]
fn future_window_extended_inputs_are_ready_and_deterministic() {
    let config = sprint46_support::future_window_extended_config("future-window-extended-suite");
    let first = FutureWindowRequirementRunner::default()
        .run(&config)
        .expect("first");
    let second = FutureWindowRequirementRunner::default()
        .run(&config)
        .expect("second");
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(
        first.items[0].gap_kind,
        FutureWindowGapKind::SufficientFutureBars
    );
    assert_eq!(first.items[0].missing_future_bars, 0);
    assert_eq!(first.rows_with_sufficient_future_window, 1);
}

#[test]
fn future_window_extension_jobs_keep_horizon_and_provider_flags_stable() {
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
    assert_eq!(job.horizon_bars, 3);
}
