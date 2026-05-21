use soma_zero::{SourceBenchmarkSummary, build_source_calibration_comparison};

fn summary(brier: f64, ece: f64) -> SourceBenchmarkSummary {
    SourceBenchmarkSummary {
        source_label: "test".to_string(),
        dataset_count: 1,
        total_outcome_records: 100,
        avg_net_return_pct: None,
        avg_max_drawdown_pct: None,
        avg_brier_score: Some(brier),
        avg_expected_calibration_error: Some(ece),
        denial_rate: None,
        defensive_value: None,
        opportunity_cost: None,
        useful_candidate: false,
        status_label: Some("Baseline".to_string()),
        warnings: vec![],
        reason_codes: vec![],
    }
}

#[test]
fn equal_calibration_metrics_are_consistent() {
    let comparison = build_source_calibration_comparison(
        Some(&summary(0.10, 0.05)),
        Some(&summary(0.10, 0.05)),
        0.05,
    );
    assert!(comparison.calibration_consistent);
}

#[test]
fn high_ece_delta_is_inconsistent() {
    let comparison = build_source_calibration_comparison(
        Some(&summary(0.10, 0.05)),
        Some(&summary(0.10, 0.20)),
        0.05,
    );
    assert!(!comparison.calibration_consistent);
}

#[test]
fn yfinance_only_marks_official_missing() {
    let comparison = build_source_calibration_comparison(None, Some(&summary(0.10, 0.05)), 0.05);
    assert!(
        comparison
            .warnings
            .iter()
            .any(|warning| warning.contains("official"))
    );
}
