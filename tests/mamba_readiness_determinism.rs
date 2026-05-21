mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use soma_zero::{
    AiSignalRecommendation, AiSignalStatus, AiSignalUsefulnessReport, BenchmarkStorageAudit,
    CalibrationSummary, MambaReadinessConfig, MambaReadinessRunner, ModelComparisonSummary,
    ModelUsefulnessGateResult, OfficialAiBenchmarkReport, OfficialConsistencyConfig,
    OfficialDatasetCoverageReport, PerformanceSummary, ProviderKind, RiskGovernorSummary,
    StorageBudgetSummary, Timeframe,
};

fn write_dataset_export_dir(name: &str) -> PathBuf {
    let output_dir = common::output_dir(name);
    fs::write(
        output_dir.join("dataset.csv"),
        [
            "row_id,symbol,timestamp_ms,timeframe,fold_id,split_kind,regime,data_quality_score,close,volume,label_outcome,label_net_return_pct,label_gross_return_pct,label_bars_held,label_first_hit,reason_codes",
            "row-0,BTC-USDT,0,OneMinute,0,Train,Range,1.0,100.0,1000.0,Win,0.01,0.012,2,TimeExpired,",
            "row-1,BTC-USDT,60000,OneMinute,0,Train,Range,1.0,101.0,1010.0,Win,0.01,0.012,2,TimeExpired,",
            "row-2,BTC-USDT,120000,OneMinute,0,Train,Range,1.0,102.0,1020.0,Win,0.01,0.012,2,TimeExpired,",
            "row-3,BTC-USDT,180000,OneMinute,0,Train,Range,1.0,103.0,1030.0,Win,0.01,0.012,2,TimeExpired,",
            "row-4,BTC-USDT,240000,OneMinute,0,Train,Range,1.0,104.0,1040.0,Win,0.01,0.012,2,TimeExpired,",
        ]
        .join("\n"),
    )
    .expect("write dataset csv");
    output_dir
}

fn config(name: &str) -> MambaReadinessConfig {
    let dataset_dir = write_dataset_export_dir(&format!("{name}-dataset"));
    let benchmark_dir = common::output_dir(&format!("{name}-benchmark"));
    let benchmark_path = benchmark_dir.join("official_ai_benchmark_report.json");
    let report = OfficialAiBenchmarkReport {
        benchmark_id: name.to_string(),
        collection_report_path: None,
        coverage_report: OfficialDatasetCoverageReport {
            total_ready_entries: 1,
            crypto_ready_entries: 1,
            korean_equity_ready_entries: 0,
            us_equity_ready_entries: 0,
            skipped_missing_auth_entries: 0,
            skipped_budget_entries: 0,
            failed_preflight_entries: 0,
            provider_statuses: BTreeMap::new(),
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
        usefulness_report: AiSignalUsefulnessReport {
            status: AiSignalStatus::ExternalModelEvaluated,
            official_dataset_count: 1,
            crypto_dataset_count: 1,
            korean_equity_dataset_count: 0,
            us_equity_dataset_count: 0,
            total_outcome_records: 24,
            baseline_summary: PerformanceSummary {
                dataset_count: 1,
                total_trades: 24,
                avg_net_return_pct: 0.05,
                avg_profit_factor: 1.2,
                avg_max_drawdown_pct: 0.02,
            },
            external_summary: Some(PerformanceSummary {
                dataset_count: 1,
                total_trades: 24,
                avg_net_return_pct: 0.06,
                avg_profit_factor: 1.3,
                avg_max_drawdown_pct: 0.02,
            }),
            calibration_summary: CalibrationSummary {
                total_count: 24,
                avg_brier_score: 0.10,
                avg_expected_calibration_error: 0.05,
                acceptable: true,
            },
            risk_governor_summary: RiskGovernorSummary {
                total_signals: 24,
                denied_by_risk: 2,
                denial_rate: 0.08,
                approval_rate: 0.92,
                emergency_stop_count: 0,
                cooldown_count: 0,
                defensive_value: 0.0,
                opportunity_cost: 0.0,
                stable: true,
            },
            model_comparison_summary: Some(ModelComparisonSummary {
                compared_datasets: 1,
                external_better_count: 1,
                avg_delta_net_return_pct: 0.01,
                avg_delta_max_drawdown_pct: -0.01,
                avg_delta_profit_factor: 0.10,
            }),
            storage_budget_summary: StorageBudgetSummary {
                collection_bytes: 1_024,
                dataset_export_bytes: 2_048,
                prediction_bytes: 0,
                report_bytes: 256,
                budget_exceeded: false,
            },
            blockers: vec![],
            warnings: vec![],
            recommendation: AiSignalRecommendation::ExternalModelCandidate,
            reason_codes: vec![],
        },
        storage_audit: BenchmarkStorageAudit {
            collection_bytes: 1_024,
            dataset_export_bytes: 2_048,
            prediction_bytes: 0,
            report_bytes: 256,
            raw_archive_bytes: 0,
            canonical_bytes: 1_024,
            budget_exceeded: false,
            largest_files: vec![],
            retention_actions: vec![],
            reason_codes: vec![],
        },
        dataset_reports: vec![soma_zero::OfficialAiDatasetReport {
            entry_id: format!("{name}-dataset"),
            provider_kind: ProviderKind::Upbit,
            symbol: "BTC-USDT".to_string(),
            timeframe: Timeframe::OneMinute,
            dataset_export_dir: Some(dataset_dir.display().to_string()),
            baseline_output_dir: None,
            external_output_dir: None,
            baseline_total_trades: 24,
            external_total_trades: Some(24),
            schema_valid: Some(true),
            data_quality_score: 1.0,
            warnings: vec![],
            reason_codes: vec![],
        }],
        warnings: vec![],
        reason_codes: vec![],
    };
    fs::write(
        &benchmark_path,
        report.to_json_string().expect("serialize benchmark report"),
    )
    .expect("write benchmark report");

    MambaReadinessConfig {
        readiness_id: name.to_string(),
        official_consistency: OfficialConsistencyConfig {
            consistency_id: format!("{name}-consistency"),
            official_benchmark_report_paths: vec![benchmark_path.display().to_string()],
            campaign_report_paths: vec![],
            require_real_official_data: true,
            min_crypto_datasets: 1,
            min_korean_equity_datasets: 0,
            min_us_equity_datasets: 0,
            min_total_outcomes: 20,
            min_per_venue_outcomes: 5,
            max_allowed_metric_variance: 0.10,
            max_allowed_drawdown_variance: 0.10,
            max_allowed_calibration_variance: 0.10,
            reason_codes: vec![],
        },
        sequence_dataset_config: soma_zero::SequenceDatasetConfig {
            window_size: 3,
            stride: 1,
            horizon_bars: 2,
            max_windows: 64,
            max_bytes: 1_048_576,
            ..soma_zero::SequenceDatasetConfig::default()
        },
        escalation_gate_config: soma_zero::ModelEscalationGateConfig {
            allow_mamba3_prototype_without_equity_data: true,
            require_min_outcomes: 20,
            require_calibration_threshold: 0.10,
            ..soma_zero::ModelEscalationGateConfig::default()
        },
        output_root: common::output_dir(&format!("{name}-output"))
            .display()
            .to_string(),
        reason_codes: vec![],
    }
}

#[test]
fn mamba_readiness_runner_is_deterministic() {
    let config = config("mamba-readiness-deterministic");
    let runner = MambaReadinessRunner::default();

    let left = runner.run(&config).expect("first readiness run");
    let right = runner.run(&config).expect("second readiness run");

    assert_eq!(left.to_text(), right.to_text());
    assert_eq!(
        left.to_json_string().expect("left json"),
        right.to_json_string().expect("right json")
    );
}
