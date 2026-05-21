use soma_zero::{
    AiSignalRecommendation, AiSignalStatus, BenchmarkStorageAudit, CalibrationSummary,
    CoreCheckedBenchmarkConfig, ExternalTabularBenchmarkStageBuilder, ModelUsefulnessGateResult,
    OfficialAiBenchmarkReport, OfficialAiDatasetReport, OfficialDatasetCoverageReport,
    PerformanceSummary, ProviderKind, ReasonCode, RiskGovernorSummary, StorageBudgetSummary,
    Timeframe,
};

fn benchmark_report(
    schema_valid: bool,
    baseline_trades: usize,
    external_trades: Option<usize>,
) -> OfficialAiBenchmarkReport {
    OfficialAiBenchmarkReport {
        benchmark_id: "external-stage".to_string(),
        collection_report_path: None,
        coverage_report: OfficialDatasetCoverageReport {
            total_ready_entries: 1,
            crypto_ready_entries: 1,
            korean_equity_ready_entries: 0,
            us_equity_ready_entries: 0,
            skipped_missing_auth_entries: 0,
            skipped_budget_entries: 0,
            failed_preflight_entries: 0,
            provider_statuses: Default::default(),
            missing_auth_providers: vec![],
            compactness_summary: "compact-only".to_string(),
            non_official_ready_entries: 0,
            reason_codes: vec![],
        },
        usefulness_gate_result: ModelUsefulnessGateResult {
            passed: true,
            failed_gates: vec![],
            warnings: vec![],
            reason_codes: vec![],
        },
        usefulness_report: soma_zero::AiSignalUsefulnessReport {
            status: AiSignalStatus::ExternalModelEvaluated,
            official_dataset_count: 1,
            crypto_dataset_count: 1,
            korean_equity_dataset_count: 0,
            us_equity_dataset_count: 0,
            total_outcome_records: baseline_trades,
            baseline_summary: PerformanceSummary::default(),
            external_summary: Some(PerformanceSummary::default()),
            calibration_summary: CalibrationSummary::default(),
            risk_governor_summary: RiskGovernorSummary::default(),
            model_comparison_summary: None,
            storage_budget_summary: StorageBudgetSummary::default(),
            blockers: vec![],
            warnings: vec![],
            recommendation: AiSignalRecommendation::NeedMoreExperiments,
            reason_codes: vec![],
        },
        storage_audit: BenchmarkStorageAudit {
            collection_bytes: 0,
            dataset_export_bytes: 0,
            prediction_bytes: 0,
            report_bytes: 0,
            raw_archive_bytes: 0,
            canonical_bytes: 0,
            budget_exceeded: false,
            largest_files: vec![],
            retention_actions: vec![],
            reason_codes: vec![],
        },
        dataset_reports: vec![OfficialAiDatasetReport {
            entry_id: "upbit-btc".to_string(),
            provider_kind: ProviderKind::Upbit,
            symbol: "BTC-USDT".to_string(),
            timeframe: Timeframe::OneMinute,
            dataset_export_dir: None,
            baseline_output_dir: None,
            external_output_dir: None,
            baseline_total_trades: baseline_trades,
            external_total_trades: external_trades,
            schema_valid: Some(schema_valid),
            data_quality_score: 1.0,
            warnings: vec![],
            reason_codes: vec![],
        }],
        warnings: vec![],
        reason_codes: vec![],
    }
}

#[test]
fn external_stage_baseline_only_works_without_python() {
    let stage = ExternalTabularBenchmarkStageBuilder::default().build(
        &CoreCheckedBenchmarkConfig::default(),
        Some(&benchmark_report(true, 40, None)),
    );

    assert!(!stage.training_requested);
    assert!(!stage.training_ran);
    assert!(!stage.prediction_validation_result.valid);
}

#[test]
fn external_stage_training_requested_without_python_is_reason_coded() {
    let stage = ExternalTabularBenchmarkStageBuilder::default().build(
        &CoreCheckedBenchmarkConfig {
            run_python_training: true,
            ..CoreCheckedBenchmarkConfig::default()
        },
        Some(&benchmark_report(true, 40, None)),
    );

    assert!(stage.reason_codes.contains(&ReasonCode::PythonUnavailable));
}

#[test]
fn external_stage_accepts_existing_prediction_csv_when_schema_and_rows_match() {
    let stage = ExternalTabularBenchmarkStageBuilder::default().build(
        &CoreCheckedBenchmarkConfig {
            existing_prediction_csv: Some("predictions.csv".to_string()),
            min_prediction_rows: 20,
            ..CoreCheckedBenchmarkConfig::default()
        },
        Some(&benchmark_report(true, 40, Some(40))),
    );

    assert_eq!(
        stage.training_backend_used.as_deref(),
        Some("existing-prediction-csv")
    );
    assert!(stage.prediction_validation_result.valid);
    assert!(stage.schema_valid);
    assert!(stage.row_alignment_valid);
}

#[test]
fn external_stage_rejects_schema_and_row_alignment_mismatch() {
    let stage = ExternalTabularBenchmarkStageBuilder::default().build(
        &CoreCheckedBenchmarkConfig {
            run_external_eval: true,
            existing_prediction_csv: Some("predictions.csv".to_string()),
            ..CoreCheckedBenchmarkConfig::default()
        },
        Some(&benchmark_report(false, 40, Some(20))),
    );

    assert!(!stage.prediction_validation_result.valid);
    assert!(stage.reason_codes.contains(&ReasonCode::SchemaMismatch));
    assert!(stage.reason_codes.contains(&ReasonCode::InvalidPrediction));
    assert!(
        stage
            .reason_codes
            .contains(&ReasonCode::MissingPredictionRows)
    );
}

#[test]
fn external_stage_is_deterministic_for_same_input() {
    let config = CoreCheckedBenchmarkConfig {
        existing_prediction_csv: Some("predictions.csv".to_string()),
        ..CoreCheckedBenchmarkConfig::default()
    };
    let report = benchmark_report(true, 40, Some(40));
    let builder = ExternalTabularBenchmarkStageBuilder::default();

    let first = builder.build(&config, Some(&report));
    let second = builder.build(&config, Some(&report));

    assert_eq!(first, second);
}
