use soma_zero::experiment::aggregate::build_aggregate_benchmark;
use soma_zero::{
    DataQualitySeverity, ExperimentRunKey, ExperimentRunStatus, ExperimentRunSummary, ReasonCode,
};

fn summary(
    dataset_id: &str,
    variant_id: &str,
    status: ExperimentRunStatus,
    total_decisions: usize,
    denied_trades: usize,
    no_trades: usize,
    net_return_pct: f64,
    max_drawdown_pct: f64,
    profit_factor: Option<f64>,
    data_quality_score: f64,
) -> ExperimentRunSummary {
    ExperimentRunSummary {
        run_key: ExperimentRunKey {
            dataset_id: dataset_id.to_string(),
            variant_id: variant_id.to_string(),
            experiment_id: format!("{dataset_id}-{variant_id}"),
        },
        status,
        manifest_summary: String::new(),
        data_quality_score,
        data_quality_severity: DataQualitySeverity::Good,
        total_decisions,
        executed_trades: total_decisions.saturating_sub(no_trades),
        denied_trades,
        no_trades,
        net_return_pct,
        max_drawdown_pct,
        profit_factor,
        calibration_brier: None,
        risk_defensive_value: None,
        external_better: None,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

#[test]
fn aggregate_benchmark_counts_and_renders_deterministically() {
    let benchmark = build_aggregate_benchmark(&[
        summary(
            "a",
            "baseline",
            ExperimentRunStatus::Passed,
            10,
            2,
            1,
            0.10,
            0.05,
            Some(1.8),
            0.95,
        ),
        summary(
            "b",
            "compare",
            ExperimentRunStatus::Failed,
            8,
            4,
            2,
            -0.20,
            0.25,
            Some(0.7),
            0.80,
        ),
        summary(
            "c",
            "baseline",
            ExperimentRunStatus::Skipped,
            0,
            0,
            0,
            0.0,
            0.0,
            None,
            0.0,
        ),
    ]);

    assert_eq!(benchmark.total_runs, 3);
    assert_eq!(benchmark.passed_runs, 1);
    assert_eq!(benchmark.failed_runs, 1);
    assert_eq!(benchmark.skipped_runs, 1);
    assert_eq!(benchmark.baseline_runs, 2);
    assert_eq!(benchmark.external_runs, 1);
    assert_eq!(benchmark.worst_max_drawdown_pct, 0.25);
    assert_eq!(benchmark.worst_net_return_pct, -0.20);
    assert!(
        benchmark
            .to_markdown_table_string()
            .contains("| total_runs | 3 |")
    );
    assert_eq!(
        benchmark.to_markdown_table_string(),
        benchmark.to_markdown_table_string()
    );
}
