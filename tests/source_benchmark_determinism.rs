mod common;

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use soma_zero::{
    MarketVenue, OfficialCollectionEntryReport, OfficialCollectionEntryStatus,
    OfficialCollectionReport, ProviderKind, SourceAwareBenchmarkConfig, SourceAwareBenchmarkRunner,
    StorageBudgetReport, Timeframe, YahooResearchEvidenceReport,
};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

fn write_canonical(path: &Path) {
    fs::write(
        path,
        "timestamp_ms,open,high,low,close,volume\n1,100,100,100,100,10\n2,101,101,101,101,11\n3,102,102,102,102,12\n20,119,119,119,119,20\n21,120,120,120,120,21\n",
    )
    .expect("write csv");
}

fn write_fixture_reports() -> (PathBuf, PathBuf) {
    let dir = common::output_dir("source-benchmark-determinism");
    let official_csv = dir.join("official.csv");
    let yfinance_csv = dir.join("yfinance.csv");
    write_canonical(&official_csv);
    write_canonical(&yfinance_csv);
    let official_report = OfficialCollectionReport {
        plan_id: "determinism".to_string(),
        entry_reports: vec![OfficialCollectionEntryReport {
            entry_id: "official-aapl".to_string(),
            provider_kind: ProviderKind::AlphaVantage,
            symbol: "AAPL".to_string(),
            venue: Some(MarketVenue::NASDAQ),
            timeframe: Timeframe::OneDay,
            status: OfficialCollectionEntryStatus::Collected,
            canonical_csv_path: Some(official_csv.display().to_string()),
            manifest_path: None,
            provenance_path: None,
            preflight_status: Some("ReadyForRealEvidence".to_string()),
            row_count: 5,
            request_count: 1,
            bytes_written: 100,
            compressed: false,
            ready_for_evidence: true,
            reason_codes: vec![],
        }],
        storage_budget_report: StorageBudgetReport::default(),
        ready_entries_count: 1,
        skipped_entries_count: 0,
        failed_entries_count: 0,
        official_api_collected_count: 1,
        reason_codes: vec![],
    };
    let official_path = dir.join("official_collection_report.json");
    fs::write(
        &official_path,
        serde_json::to_string_pretty(&official_report).expect("json"),
    )
    .expect("write official");

    let prov = dir.join("yfinance.prov.json");
    fs::write(
        &prov,
        format!(
            "{{\"source_kind\":\"YFinanceResearch\",\"source_label\":\"determinism\",\"provider_label\":\"yfinance\",\"upstream_label\":\"Yahoo Finance\",\"local_path\":\"{}\",\"generated_by\":\"test\",\"user_supplied\":true,\"downloaded_by_soma\":false,\"remote_url_present\":false,\"official_provider\":false,\"affiliated_or_endorsed\":false,\"intended_use\":\"research-only unofficial supplemental benchmark data\",\"readiness_eligible\":false,\"benchmark_eligible\":true,\"reason_codes\":[\"YFinanceCanonicalized\"]}}",
            yfinance_csv.display()
        ),
    )
    .expect("write prov");
    let yahoo_report = YahooResearchEvidenceReport {
        research_id: "determinism".to_string(),
        yfinance_symbols: vec!["AAPL".to_string()],
        canonical_csv_paths: vec![yfinance_csv.display().to_string()],
        provenance_paths: vec![prov.display().to_string()],
        preflight_statuses: vec!["NotRealLocalEligible".to_string()],
        official_readiness_eligible_count: 0,
        benchmark_eligible_count: 1,
        total_rows: 5,
        total_storage_bytes: 0,
        generated_config_paths: vec![],
        warnings: vec![],
        reason_codes: vec![],
    };
    let yahoo_path = dir.join("yahoo_research_evidence_report.json");
    fs::write(
        &yahoo_path,
        serde_json::to_string_pretty(&yahoo_report).expect("json"),
    )
    .expect("write yahoo");
    (official_path, yahoo_path)
}

#[test]
fn source_benchmark_examples_parse() {
    for path in [
        example_path("soma_source_benchmark_yfinance_only.toml"),
        example_path("soma_source_benchmark_official_vs_yfinance.toml"),
        example_path("soma_source_compare_existing_reports.toml"),
    ] {
        let config = SourceAwareBenchmarkConfig::from_toml_path(&path).expect("parse");
        assert!(!config.benchmark_id.is_empty());
    }
}

#[test]
fn source_benchmark_report_is_deterministic() {
    let (official, yahoo) = write_fixture_reports();
    let config = SourceAwareBenchmarkConfig {
        benchmark_id: "deterministic-source-benchmark".to_string(),
        official_collection_report_paths: vec![official.display().to_string()],
        yahoo_research_report_paths: vec![yahoo.display().to_string()],
        min_outcome_records: 1,
        ..SourceAwareBenchmarkConfig::default()
    };
    let first = SourceAwareBenchmarkRunner::default()
        .run(&config)
        .expect("first");
    let second = SourceAwareBenchmarkRunner::default()
        .run(&config)
        .expect("second");
    assert_eq!(
        first.to_json_string().expect("json"),
        second.to_json_string().expect("json")
    );
}
