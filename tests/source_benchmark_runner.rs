mod common;

use std::fs;
use std::path::{Path, PathBuf};

use soma_zero::{
    CoreCheckConfig, MarketVenue, OfficialCollectionEntryReport, OfficialCollectionEntryStatus,
    OfficialCollectionReport, ProviderKind, SourceAwareBenchmarkConfig, SourceAwareBenchmarkRunner,
    StorageBudgetReport, Timeframe, YahooResearchEvidenceReport,
};

fn write_canonical_csv(path: &Path, rows: &[(u64, f64)]) {
    let mut text = "timestamp_ms,open,high,low,close,volume\n".to_string();
    for (timestamp, close) in rows {
        text.push_str(&format!(
            "{timestamp},{close},{close},{close},{close},100\n"
        ));
    }
    fs::write(path, text).expect("write csv");
}

fn write_official_collection_report(
    name: &str,
    symbol: &str,
    path: &Path,
    row_count: usize,
) -> PathBuf {
    let report = OfficialCollectionReport {
        plan_id: name.to_string(),
        entry_reports: vec![OfficialCollectionEntryReport {
            entry_id: format!("{name}-{symbol}"),
            provider_kind: ProviderKind::AlphaVantage,
            symbol: symbol.to_string(),
            venue: Some(MarketVenue::NASDAQ),
            timeframe: Timeframe::OneDay,
            status: OfficialCollectionEntryStatus::Collected,
            canonical_csv_path: Some(path.display().to_string()),
            manifest_path: None,
            provenance_path: None,
            preflight_status: Some("ReadyForRealEvidence".to_string()),
            row_count,
            request_count: 1,
            bytes_written: 256,
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
    let path =
        common::output_dir(&format!("{name}-official")).join("official_collection_report.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&report).expect("serialize official"),
    )
    .expect("write official");
    path
}

fn write_yahoo_report(name: &str, symbol: &str, csv_path: &Path) -> PathBuf {
    let provenance_path = common::output_dir(&format!("{name}-prov")).join("prov.json");
    fs::write(
        &provenance_path,
        format!(
            "{{\"source_kind\":\"YFinanceResearch\",\"source_label\":\"{name}\",\"provider_label\":\"yfinance\",\"upstream_label\":\"Yahoo Finance\",\"local_path\":\"{}\",\"generated_by\":\"test\",\"user_supplied\":true,\"downloaded_by_soma\":false,\"remote_url_present\":false,\"official_provider\":false,\"affiliated_or_endorsed\":false,\"intended_use\":\"research-only unofficial supplemental benchmark data\",\"readiness_eligible\":false,\"benchmark_eligible\":true,\"reason_codes\":[\"YFinanceCanonicalized\"]}}",
            csv_path.display()
        ),
    )
    .expect("write provenance");
    let report = YahooResearchEvidenceReport {
        research_id: name.to_string(),
        yfinance_symbols: vec![symbol.to_string()],
        canonical_csv_paths: vec![csv_path.display().to_string()],
        provenance_paths: vec![provenance_path.display().to_string()],
        preflight_statuses: vec!["NotRealLocalEligible".to_string()],
        official_readiness_eligible_count: 0,
        benchmark_eligible_count: 1,
        total_rows: 0,
        total_storage_bytes: 0,
        generated_config_paths: vec![],
        warnings: vec![],
        reason_codes: vec![],
    };
    let path =
        common::output_dir(&format!("{name}-yahoo")).join("yahoo_research_evidence_report.json");
    fs::write(
        &path,
        serde_json::to_string_pretty(&report).expect("serialize yahoo"),
    )
    .expect("write yahoo");
    path
}

#[test]
fn core_check_failure_blocks_source_benchmark() {
    let config = SourceAwareBenchmarkConfig {
        benchmark_id: "core-blocked".to_string(),
        core_check_config: Some(CoreCheckConfig {
            official_evidence_ready: true,
            sequence_dataset_ready: true,
            external_model_bridge_ready: false,
            ..CoreCheckConfig::default()
        }),
        ..SourceAwareBenchmarkConfig::default()
    };
    let report = SourceAwareBenchmarkRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        report.final_status,
        soma_zero::SourceAwareBenchmarkStatus::CoreBlocked
    );
}

#[test]
fn yfinance_only_is_research_only() {
    let dir = common::output_dir("source-runner-yfinance-only");
    let csv = dir.join("aapl_1d.csv");
    write_canonical_csv(
        &csv,
        &[
            (1, 100.0),
            (2, 101.0),
            (3, 102.0),
            (4, 103.0),
            (5, 104.0),
            (6, 105.0),
            (7, 106.0),
            (8, 107.0),
            (9, 108.0),
            (10, 109.0),
            (11, 110.0),
            (12, 111.0),
            (13, 112.0),
            (14, 113.0),
            (15, 114.0),
            (16, 115.0),
            (17, 116.0),
            (18, 117.0),
            (19, 118.0),
            (20, 119.0),
            (21, 120.0),
        ],
    );
    let yahoo = write_yahoo_report("yfinance-only", "AAPL", &csv);
    let config = SourceAwareBenchmarkConfig {
        benchmark_id: "yfinance-only".to_string(),
        yahoo_research_report_paths: vec![yahoo.display().to_string()],
        ..SourceAwareBenchmarkConfig::default()
    };
    let report = SourceAwareBenchmarkRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        report.final_status,
        soma_zero::SourceAwareBenchmarkStatus::YFinanceResearchOnly
    );
}

#[test]
fn official_only_is_official_only_benchmark() {
    let dir = common::output_dir("source-runner-official-only");
    let csv = dir.join("official.csv");
    write_canonical_csv(&csv, &[(1, 100.0), (2, 101.0), (3, 102.0)]);
    let official = write_official_collection_report("official-only", "AAPL", &csv, 3);
    let config = SourceAwareBenchmarkConfig {
        benchmark_id: "official-only".to_string(),
        official_collection_report_paths: vec![official.display().to_string()],
        ..SourceAwareBenchmarkConfig::default()
    };
    let report = SourceAwareBenchmarkRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        report.final_status,
        soma_zero::SourceAwareBenchmarkStatus::OfficialOnlyBenchmark
    );
}

#[test]
fn high_mismatch_is_detected() {
    let dir = common::output_dir("source-runner-high-mismatch");
    let official_csv = dir.join("official.csv");
    let yfinance_csv = dir.join("yfinance.csv");
    write_canonical_csv(&official_csv, &[(1, 100.0), (2, 101.0), (3, 102.0)]);
    write_canonical_csv(&yfinance_csv, &[(1, 150.0), (2, 151.0), (3, 152.0)]);
    let official = write_official_collection_report("high-mismatch", "AAPL", &official_csv, 3);
    let yahoo = write_yahoo_report("high-mismatch", "AAPL", &yfinance_csv);
    let config = SourceAwareBenchmarkConfig {
        benchmark_id: "high-mismatch".to_string(),
        official_collection_report_paths: vec![official.display().to_string()],
        yahoo_research_report_paths: vec![yahoo.display().to_string()],
        min_outcome_records: 1,
        ..SourceAwareBenchmarkConfig::default()
    };
    let report = SourceAwareBenchmarkRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        report.final_status,
        soma_zero::SourceAwareBenchmarkStatus::SourceMismatchHigh
    );
}

#[test]
fn low_mismatch_allows_source_comparison() {
    let dir = common::output_dir("source-runner-low-mismatch");
    let official_csv = dir.join("official.csv");
    let yfinance_csv = dir.join("yfinance.csv");
    write_canonical_csv(
        &official_csv,
        &[
            (1, 100.0),
            (2, 101.0),
            (3, 102.0),
            (4, 103.0),
            (5, 104.0),
            (6, 105.0),
            (7, 106.0),
            (8, 107.0),
            (9, 108.0),
            (10, 109.0),
            (11, 110.0),
            (12, 111.0),
            (13, 112.0),
            (14, 113.0),
            (15, 114.0),
            (16, 115.0),
            (17, 116.0),
            (18, 117.0),
            (19, 118.0),
            (20, 119.0),
            (21, 120.0),
        ],
    );
    write_canonical_csv(
        &yfinance_csv,
        &[
            (1, 100.0),
            (2, 101.0),
            (3, 102.0),
            (4, 103.0),
            (5, 104.0),
            (6, 105.0),
            (7, 106.0),
            (8, 107.0),
            (9, 108.0),
            (10, 109.0),
            (11, 110.0),
            (12, 111.0),
            (13, 112.0),
            (14, 113.0),
            (15, 114.0),
            (16, 115.0),
            (17, 116.0),
            (18, 117.0),
            (19, 118.0),
            (20, 119.0),
            (21, 120.0),
        ],
    );
    let official = write_official_collection_report("low-mismatch", "AAPL", &official_csv, 21);
    let yahoo = write_yahoo_report("low-mismatch", "AAPL", &yfinance_csv);
    let config = SourceAwareBenchmarkConfig {
        benchmark_id: "low-mismatch".to_string(),
        official_collection_report_paths: vec![official.display().to_string()],
        yahoo_research_report_paths: vec![yahoo.display().to_string()],
        min_outcome_records: 20,
        ..SourceAwareBenchmarkConfig::default()
    };
    let report = SourceAwareBenchmarkRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        report.final_status,
        soma_zero::SourceAwareBenchmarkStatus::SourceComparisonAvailable
    );
}

#[test]
fn insufficient_outcomes_is_detected() {
    let dir = common::output_dir("source-runner-insufficient");
    let official_csv = dir.join("official.csv");
    let yfinance_csv = dir.join("yfinance.csv");
    write_canonical_csv(&official_csv, &[(1, 100.0), (2, 101.0), (3, 102.0)]);
    write_canonical_csv(&yfinance_csv, &[(1, 100.0), (2, 101.0), (3, 102.0)]);
    let official = write_official_collection_report("insufficient", "AAPL", &official_csv, 3);
    let yahoo = write_yahoo_report("insufficient", "AAPL", &yfinance_csv);
    let config = SourceAwareBenchmarkConfig {
        benchmark_id: "insufficient".to_string(),
        official_collection_report_paths: vec![official.display().to_string()],
        yahoo_research_report_paths: vec![yahoo.display().to_string()],
        min_outcome_records: 20,
        ..SourceAwareBenchmarkConfig::default()
    };
    let report = SourceAwareBenchmarkRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        report.final_status,
        soma_zero::SourceAwareBenchmarkStatus::InsufficientOutcomes
    );
}
