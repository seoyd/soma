mod common;

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    CoreCheckConfig, CoreCheckedBenchmarkConfig, CoreCheckedBenchmarkRunner,
    CoreCheckedBenchmarkStatus, MarketVenue, OfficialCollectionEntryReport,
    OfficialCollectionEntryStatus, OfficialCollectionReport, ProviderKind, ReasonCode,
    StorageBudgetReport, Timeframe,
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

fn config(name: &str, report_path: Option<PathBuf>) -> CoreCheckedBenchmarkConfig {
    CoreCheckedBenchmarkConfig {
        benchmark_id: name.to_string(),
        official_collection_report_path: report_path.map(|path| path.display().to_string()),
        output_root: common::output_dir(&format!("{name}-output"))
            .display()
            .to_string(),
        allow_crypto_only: true,
        min_outcome_records: 0,
        min_prediction_rows: 1,
        min_calibration_count: 1,
        max_allowed_brier_score: 1.0,
        max_allowed_ece: 1.0,
        max_allowed_drawdown_worsening_pct: 1.0,
        ..CoreCheckedBenchmarkConfig::default()
    }
}

#[test]
fn core_checked_benchmark_runner_reports_missing_official_data_when_unconfigured() {
    let report = CoreCheckedBenchmarkRunner::default()
        .run(&config("core-benchmark-no-data", None))
        .expect("run benchmark");

    assert_eq!(
        report.final_status,
        CoreCheckedBenchmarkStatus::MissingOfficialData
    );
}

#[test]
fn core_checked_benchmark_runner_supports_baseline_only_evaluation() {
    let report = CoreCheckedBenchmarkRunner::default()
        .run(&config(
            "core-benchmark-baseline",
            Some(write_collection_report("core-benchmark-baseline")),
        ))
        .expect("run baseline benchmark");

    assert_eq!(
        report.final_status,
        CoreCheckedBenchmarkStatus::BaselineOnlyEvaluated
    );
    assert!(report.baseline_report.is_some());
    assert!(report.external_report.is_none());
}

#[test]
fn core_checked_benchmark_runner_blocks_when_core_gate_fails() {
    let report = CoreCheckedBenchmarkRunner::default()
        .run(&CoreCheckedBenchmarkConfig {
            core_check_config: Some(CoreCheckConfig::default()),
            allowed_core_statuses: vec![],
            benchmark_id: "core-benchmark-blocked".to_string(),
            output_root: common::output_dir("core-benchmark-blocked-output")
                .display()
                .to_string(),
            ..config("core-benchmark-blocked", None)
        })
        .expect("run blocked benchmark");

    assert_eq!(report.final_status, CoreCheckedBenchmarkStatus::CoreBlocked);
}

#[test]
fn core_checked_benchmark_runner_evaluates_existing_predictions() {
    let report_path = write_collection_report("core-benchmark-external");
    let prediction_path =
        common::output_dir("core-benchmark-external-predictions").join("predictions.csv");
    fs::write(
        &prediction_path,
        common::perfect_prediction_csv(
            "core-benchmark-external-prediction-seed",
            "generic_ohlcv_valid.csv",
        ),
    )
    .expect("write predictions");

    let report = CoreCheckedBenchmarkRunner::default()
        .run(&CoreCheckedBenchmarkConfig {
            run_external_eval: true,
            existing_prediction_csv: Some(prediction_path.display().to_string()),
            ..config("core-benchmark-external", Some(report_path))
        })
        .expect("run external benchmark");

    assert!(report.external_report.is_some());
    assert!(matches!(
        report.final_status,
        CoreCheckedBenchmarkStatus::ExternalModelEvaluated
            | CoreCheckedBenchmarkStatus::ExternalTabularCandidate
            | CoreCheckedBenchmarkStatus::PoorCalibration
            | CoreCheckedBenchmarkStatus::WorseThanBaseline
            | CoreCheckedBenchmarkStatus::PoorRiskBehavior
            | CoreCheckedBenchmarkStatus::NeedMoreExperiments
    ));
}
