mod common;

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    CoreCheckedBenchmarkConfig, CoreCheckedBenchmarkRunner, MarketVenue,
    OfficialCollectionEntryReport, OfficialCollectionEntryStatus, OfficialCollectionReport,
    ProviderKind, ReasonCode, StorageBudgetReport, Timeframe,
};

fn write_collection_report(name: &str) -> PathBuf {
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
            reason_codes: vec![ReasonCode::OfficialCollectionEntryCollected],
        }],
        storage_budget_report: StorageBudgetReport::default(),
        ready_entries_count: 1,
        skipped_entries_count: 0,
        failed_entries_count: 0,
        official_api_collected_count: 1,
        reason_codes: vec![ReasonCode::OfficialCollectionRan],
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
fn core_checked_benchmark_runner_is_deterministic_for_same_input() {
    let report_path = write_collection_report("core-benchmark-determinism");
    let config = CoreCheckedBenchmarkConfig {
        benchmark_id: "core-benchmark-determinism".to_string(),
        official_collection_report_path: Some(report_path.display().to_string()),
        output_root: common::output_dir("core-benchmark-determinism-output")
            .display()
            .to_string(),
        allow_crypto_only: true,
        min_outcome_records: 0,
        min_calibration_count: 1,
        max_allowed_brier_score: 1.0,
        max_allowed_ece: 1.0,
        max_allowed_drawdown_worsening_pct: 1.0,
        ..CoreCheckedBenchmarkConfig::default()
    };
    let runner = CoreCheckedBenchmarkRunner::default();

    let _ = fs::remove_dir_all(config.output_dir());
    let first = runner.run(&config).expect("first report");
    let _ = fs::remove_dir_all(config.output_dir());
    let second = runner.run(&config).expect("second report");

    assert_eq!(
        first.to_json_string().expect("first json"),
        second.to_json_string().expect("second json")
    );
}
