mod common;

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    AiSignalStatus, MarketVenue, OfficialAiBenchmarkConfig, OfficialAiBenchmarkRunner,
    OfficialCollectionEntryReport, OfficialCollectionEntryStatus, OfficialCollectionReport,
    ProviderKind, StorageBudgetReport, Timeframe,
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

fn benchmark_config(name: &str, report_path: PathBuf) -> OfficialAiBenchmarkConfig {
    OfficialAiBenchmarkConfig {
        benchmark_id: name.to_string(),
        official_collection_plan_path: None,
        official_collection_report_path: Some(report_path.display().to_string()),
        run_collection: false,
        run_dataset_export: true,
        run_python_training: false,
        run_external_prediction_eval: false,
        run_baseline_eval: true,
        run_ablation_eval: false,
        output_root: common::output_dir(&format!("{name}-benchmark"))
            .display()
            .to_string(),
        python_executable: None,
        train_script_path: None,
        existing_prediction_csv: None,
        strict_schema_validation: true,
        min_official_ready_datasets: 1,
        min_outcome_records: 0,
        min_calibration_count: 1,
        min_comparable_models: 1,
        max_allowed_drawdown_pct: 1.0,
        max_allowed_ece: 1.0,
        max_allowed_brier_score: 1.0,
        min_profit_factor: None,
        min_net_return_pct: None,
        allow_upbit_only: true,
        allow_equity_missing_auth: true,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

#[test]
fn benchmark_runner_works_in_baseline_only_mode_without_python() {
    let report_path = write_collection_report("official-ai-benchmark-baseline");
    let report = OfficialAiBenchmarkRunner::default().run(&benchmark_config(
        "official-ai-benchmark-baseline",
        report_path,
    ));

    assert_eq!(
        report.usefulness_report.status,
        AiSignalStatus::BaselineEvaluated
    );
    assert_eq!(report.dataset_reports.len(), 1);
    assert!(report.usefulness_report.official_dataset_count >= 1);
}

#[test]
fn benchmark_runner_rejects_invalid_external_predictions_conservatively() {
    let report_path = write_collection_report("official-ai-benchmark-invalid");
    let output_dir = common::output_dir("official-ai-benchmark-invalid-predictions");
    let prediction_path = output_dir.join("bad_predictions.csv");
    fs::write(
        &prediction_path,
        "row_id,symbol,timestamp_ms,timeframe,fold_id,split_kind,model_id,p_win,p_stop,expected_return,expected_drawdown,confidence,no_trade_probability,horizon_bars,reason_codes\nbad,BTCUSDT,1,OneMinute,0,Test,m,9.0,0.1,0.1,0.1,0.8,0.1,8,\n",
    )
    .expect("write invalid predictions");
    let mut config = benchmark_config("official-ai-benchmark-invalid", report_path);
    config.run_external_prediction_eval = true;
    config.existing_prediction_csv = Some(prediction_path.display().to_string());

    let report = OfficialAiBenchmarkRunner::default().run(&config);
    assert_eq!(
        report.usefulness_report.status,
        AiSignalStatus::BaselineEvaluated
    );
    assert_eq!(report.dataset_reports[0].schema_valid, Some(false));
}

#[test]
fn benchmark_runner_evaluates_valid_external_predictions() {
    let report_path = write_collection_report("official-ai-benchmark-external");
    let output_dir = common::output_dir("official-ai-benchmark-external-predictions");
    let prediction_path = output_dir.join("predictions.csv");
    fs::write(
        &prediction_path,
        common::perfect_prediction_csv("benchmark-external", "generic_ohlcv_valid.csv"),
    )
    .expect("write valid predictions");
    let mut config = benchmark_config("official-ai-benchmark-external", report_path);
    config.run_external_prediction_eval = true;
    config.existing_prediction_csv = Some(prediction_path.display().to_string());

    let report = OfficialAiBenchmarkRunner::default().run(&config);
    assert!(report.usefulness_report.external_summary.is_some());
    assert_eq!(report.dataset_reports[0].schema_valid, Some(true));
}
