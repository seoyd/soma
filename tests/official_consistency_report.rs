use std::collections::BTreeMap;

use soma_zero::{
    AiSignalRecommendation, AiSignalStatus, AiSignalUsefulnessReport, BenchmarkStorageAudit,
    CalibrationSummary, ModelComparisonSummary, ModelUsefulnessGateResult,
    OfficialAiBenchmarkReport, OfficialConsistencyConfig, OfficialConsistencyReport,
    OfficialConsistencyStatus, OfficialDatasetCoverageReport, PerformanceSummary, ProviderKind,
    RiskGovernorSummary, StorageBudgetSummary, Timeframe,
};

fn config() -> OfficialConsistencyConfig {
    OfficialConsistencyConfig {
        consistency_id: "sprint22-consistency".to_string(),
        official_benchmark_report_paths: vec![],
        campaign_report_paths: vec![],
        require_real_official_data: true,
        min_crypto_datasets: 1,
        min_korean_equity_datasets: 1,
        min_us_equity_datasets: 1,
        min_total_outcomes: 20,
        min_per_venue_outcomes: 5,
        max_allowed_metric_variance: 0.05,
        max_allowed_drawdown_variance: 0.05,
        max_allowed_calibration_variance: 0.05,
        reason_codes: vec![],
    }
}

#[allow(clippy::too_many_arguments)]
fn benchmark_report(
    benchmark_id: &str,
    crypto_datasets: usize,
    korean_equity_datasets: usize,
    us_equity_datasets: usize,
    missing_auth_providers: Vec<&str>,
    official_dataset_count: usize,
    total_outcomes: usize,
    avg_net_return_pct: f64,
    avg_drawdown_pct: f64,
    avg_brier_score: f64,
    denial_rate: f64,
    stable: bool,
    usefulness_status: AiSignalStatus,
) -> OfficialAiBenchmarkReport {
    OfficialAiBenchmarkReport {
        benchmark_id: benchmark_id.to_string(),
        collection_report_path: None,
        coverage_report: OfficialDatasetCoverageReport {
            total_ready_entries: official_dataset_count,
            crypto_ready_entries: crypto_datasets,
            korean_equity_ready_entries: korean_equity_datasets,
            us_equity_ready_entries: us_equity_datasets,
            skipped_missing_auth_entries: missing_auth_providers.len(),
            skipped_budget_entries: 0,
            failed_preflight_entries: 0,
            provider_statuses: BTreeMap::new(),
            missing_auth_providers: missing_auth_providers
                .into_iter()
                .map(str::to_string)
                .collect(),
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
            status: usefulness_status,
            official_dataset_count,
            crypto_dataset_count: crypto_datasets,
            korean_equity_dataset_count: korean_equity_datasets,
            us_equity_dataset_count: us_equity_datasets,
            total_outcome_records: total_outcomes,
            baseline_summary: PerformanceSummary {
                dataset_count: official_dataset_count,
                total_trades: total_outcomes,
                avg_net_return_pct,
                avg_profit_factor: 1.2,
                avg_max_drawdown_pct: avg_drawdown_pct,
            },
            external_summary: Some(PerformanceSummary {
                dataset_count: official_dataset_count,
                total_trades: total_outcomes,
                avg_net_return_pct: avg_net_return_pct + 0.01,
                avg_profit_factor: 1.3,
                avg_max_drawdown_pct: avg_drawdown_pct,
            }),
            calibration_summary: CalibrationSummary {
                total_count: total_outcomes,
                avg_brier_score,
                avg_expected_calibration_error: avg_brier_score / 2.0,
                acceptable: true,
            },
            risk_governor_summary: RiskGovernorSummary {
                total_signals: total_outcomes,
                denied_by_risk: (denial_rate * total_outcomes as f64).round() as usize,
                denial_rate,
                approval_rate: 1.0 - denial_rate,
                emergency_stop_count: 0,
                cooldown_count: 0,
                defensive_value: 0.0,
                opportunity_cost: 0.0,
                stable,
            },
            model_comparison_summary: Some(ModelComparisonSummary {
                compared_datasets: official_dataset_count,
                external_better_count: official_dataset_count,
                avg_delta_net_return_pct: 0.01,
                avg_delta_max_drawdown_pct: -0.01,
                avg_delta_profit_factor: 0.10,
            }),
            storage_budget_summary: StorageBudgetSummary {
                collection_bytes: 1_024,
                dataset_export_bytes: 2_048,
                prediction_bytes: 512,
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
            prediction_bytes: 512,
            report_bytes: 256,
            raw_archive_bytes: 0,
            canonical_bytes: 1_024,
            budget_exceeded: false,
            largest_files: vec![],
            retention_actions: vec![],
            reason_codes: vec![],
        },
        dataset_reports: vec![soma_zero::OfficialAiDatasetReport {
            entry_id: format!("{benchmark_id}-dataset"),
            provider_kind: ProviderKind::Upbit,
            symbol: "BTC-USDT".to_string(),
            timeframe: Timeframe::OneMinute,
            dataset_export_dir: None,
            baseline_output_dir: None,
            external_output_dir: None,
            baseline_total_trades: total_outcomes,
            external_total_trades: Some(total_outcomes),
            schema_valid: Some(true),
            data_quality_score: 1.0,
            warnings: vec![],
            reason_codes: vec![],
        }],
        warnings: vec![],
        reason_codes: vec![],
    }
}

#[test]
fn upbit_only_reports_are_crypto_only() {
    let report = OfficialConsistencyReport::build(
        &config(),
        &[benchmark_report(
            "crypto-only",
            1,
            0,
            0,
            vec![],
            1,
            24,
            0.05,
            0.02,
            0.10,
            0.10,
            true,
            AiSignalStatus::ExternalModelEvaluated,
        )],
    );

    assert_eq!(
        report.consistency_status,
        OfficialConsistencyStatus::CryptoOnly
    );
}

#[test]
fn missing_krx_auth_blocks_korean_equity_claim() {
    let report = OfficialConsistencyReport::build(
        &config(),
        &[benchmark_report(
            "missing-krx",
            1,
            0,
            0,
            vec!["krx"],
            1,
            24,
            0.05,
            0.02,
            0.10,
            0.10,
            true,
            AiSignalStatus::ExternalModelEvaluated,
        )],
    );

    assert_eq!(
        report.consistency_status,
        OfficialConsistencyStatus::MissingAuth
    );
}

#[test]
fn missing_alphavantage_auth_blocks_us_equity_claim() {
    let report = OfficialConsistencyReport::build(
        &config(),
        &[benchmark_report(
            "missing-us",
            1,
            0,
            0,
            vec!["alphavantage"],
            1,
            24,
            0.05,
            0.02,
            0.10,
            0.10,
            true,
            AiSignalStatus::ExternalModelEvaluated,
        )],
    );

    assert_eq!(
        report.consistency_status,
        OfficialConsistencyStatus::MissingAuth
    );
}

#[test]
fn mock_like_non_official_entries_do_not_count() {
    let real = benchmark_report(
        "real",
        1,
        0,
        0,
        vec![],
        1,
        24,
        0.05,
        0.02,
        0.10,
        0.10,
        true,
        AiSignalStatus::ExternalModelEvaluated,
    );
    let mut mock = benchmark_report(
        "mock",
        0,
        0,
        0,
        vec![],
        0,
        0,
        0.0,
        0.0,
        0.0,
        0.0,
        true,
        AiSignalStatus::PipelineOnly,
    );
    mock.coverage_report.non_official_ready_entries = 1;

    let report = OfficialConsistencyReport::build(&config(), &[real, mock]);

    assert_eq!(
        report.consistency_status,
        OfficialConsistencyStatus::CryptoOnly
    );
    assert!(report.crypto_summary.contains("datasets=1"));
}

#[test]
fn insufficient_outcomes_are_flagged() {
    let report = OfficialConsistencyReport::build(
        &config(),
        &[benchmark_report(
            "insufficient-outcomes",
            1,
            1,
            1,
            vec![],
            3,
            9,
            0.05,
            0.02,
            0.10,
            0.10,
            true,
            AiSignalStatus::ExternalModelEvaluated,
        )],
    );

    assert_eq!(
        report.consistency_status,
        OfficialConsistencyStatus::InsufficientOutcomes
    );
}

#[test]
fn inconsistent_metrics_are_flagged() {
    let first = benchmark_report(
        "consistent-a",
        1,
        1,
        1,
        vec![],
        3,
        30,
        0.10,
        0.02,
        0.10,
        0.10,
        true,
        AiSignalStatus::ExternalModelEvaluated,
    );
    let second = benchmark_report(
        "consistent-b",
        1,
        1,
        1,
        vec![],
        3,
        30,
        -0.10,
        0.12,
        0.10,
        0.10,
        true,
        AiSignalStatus::ExternalModelEvaluated,
    );

    let report = OfficialConsistencyReport::build(&config(), &[first, second]);

    assert_eq!(
        report.consistency_status,
        OfficialConsistencyStatus::InconsistentMetrics
    );
}

#[test]
fn consistency_report_text_is_deterministic() {
    let reports = vec![benchmark_report(
        "deterministic",
        1,
        1,
        1,
        vec![],
        3,
        30,
        0.05,
        0.02,
        0.10,
        0.10,
        true,
        AiSignalStatus::ExternalModelEvaluated,
    )];

    let left = OfficialConsistencyReport::build(&config(), &reports);
    let right = OfficialConsistencyReport::build(&config(), &reports);

    assert_eq!(left.to_text(), right.to_text());
}
