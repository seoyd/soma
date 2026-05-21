mod common;

use std::fs;

use soma_zero::{
    MarketVenue, OfficialAiBenchmarkConfig, OfficialAiBenchmarkRunner,
    OfficialCollectionEntryReport, OfficialCollectionEntryStatus, OfficialCollectionReport,
    ProviderKind, StorageBudgetReport, Timeframe,
};

fn write_collection_report(name: &str) -> std::path::PathBuf {
    let output_dir = common::output_dir(name);
    let report_path = output_dir.join("official_collection_report.json");
    let canonical_path = common::fixture_path("generic_ohlcv_valid.csv");
    let report = OfficialCollectionReport {
        plan_id: name.to_string(),
        entry_reports: vec![OfficialCollectionEntryReport {
            entry_id: "upbit-btc".to_string(),
            provider_kind: ProviderKind::Upbit,
            symbol: "BTC-USDT".to_string(),
            venue: Some(MarketVenue::Upbit),
            timeframe: Timeframe::OneMinute,
            status: OfficialCollectionEntryStatus::Collected,
            canonical_csv_path: Some(canonical_path.display().to_string()),
            manifest_path: None,
            provenance_path: None,
            preflight_status: Some("ReadyForRealEvidence".to_string()),
            row_count: 120,
            request_count: 1,
            bytes_written: 1024,
            compressed: false,
            ready_for_evidence: true,
            reason_codes: vec![soma_zero::ReasonCode::OfficialCollectionEntryCollected],
        }],
        storage_budget_report: StorageBudgetReport::default(),
        ready_entries_count: 1,
        skipped_entries_count: 0,
        failed_entries_count: 0,
        official_api_collected_count: 1,
        reason_codes: vec![soma_zero::ReasonCode::OfficialCollectionRan],
    };
    fs::write(
        &report_path,
        report
            .to_json_string()
            .expect("serialize collection report"),
    )
    .expect("write collection report");
    report_path
}

#[test]
fn benchmark_runner_is_deterministic_for_same_input() {
    let report_path = write_collection_report("ai-benchmark-determinism");
    let config = OfficialAiBenchmarkConfig {
        benchmark_id: "ai-benchmark-determinism".to_string(),
        official_collection_report_path: Some(report_path.display().to_string()),
        output_root: common::output_dir("ai-benchmark-determinism-output")
            .display()
            .to_string(),
        allow_upbit_only: true,
        min_outcome_records: 0,
        min_calibration_count: 1,
        max_allowed_drawdown_pct: 1.0,
        max_allowed_ece: 1.0,
        max_allowed_brier_score: 1.0,
        ..OfficialAiBenchmarkConfig::default()
    };
    let runner = OfficialAiBenchmarkRunner::default();
    let _ = fs::remove_dir_all(config.output_dir());
    let first = runner.run(&config).to_json_string().expect("first json");
    let _ = fs::remove_dir_all(config.output_dir());
    let second = runner.run(&config).to_json_string().expect("second json");
    assert_eq!(first, second);
}
