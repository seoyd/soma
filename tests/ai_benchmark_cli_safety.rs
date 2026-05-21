mod common;

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use soma_zero::{
    MarketVenue, OfficialAiBenchmarkConfig, OfficialCollectionEntryReport,
    OfficialCollectionEntryStatus, OfficialCollectionReport, ProviderKind, StorageBudgetReport,
    Timeframe,
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
fn cli_help_exposes_ai_benchmark_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("run --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ai-benchmark"));
    assert!(stdout.contains("collect-train-evaluate"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
    assert!(!stdout.contains("\n  live"));
}

#[test]
fn ai_benchmark_rejects_remote_config_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "ai-benchmark",
            "--config",
            "https://example.com/benchmark.toml",
        ])
        .output()
        .expect("run ai-benchmark");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("ai-benchmark config path must be local")
    );
}

#[test]
fn ai_benchmark_cli_runs_with_local_config() {
    let report_path = write_collection_report("ai-benchmark-cli");
    let config = OfficialAiBenchmarkConfig {
        benchmark_id: "ai-benchmark-cli".to_string(),
        official_collection_report_path: Some(report_path.display().to_string()),
        output_root: common::output_dir("ai-benchmark-cli-output")
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
    let config_dir = common::output_dir("ai-benchmark-cli-config");
    let config_path = config_dir.join("benchmark.toml");
    fs::write(
        &config_path,
        config
            .to_toml_string()
            .expect("serialize ai benchmark config"),
    )
    .expect("write benchmark config");

    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "ai-benchmark",
            "--config",
            &config_path.display().to_string(),
        ])
        .output()
        .expect("run ai-benchmark");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("benchmark_id=ai-benchmark-cli"));
}
