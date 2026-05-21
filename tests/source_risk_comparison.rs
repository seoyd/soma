use soma_zero::{SourceBenchmarkSummary, build_source_risk_interaction_comparison};

fn summary(denial_rate: f64, drawdown: f64) -> SourceBenchmarkSummary {
    SourceBenchmarkSummary {
        source_label: "test".to_string(),
        dataset_count: 1,
        total_outcome_records: 100,
        avg_net_return_pct: Some(1.0),
        avg_max_drawdown_pct: Some(drawdown),
        avg_brier_score: None,
        avg_expected_calibration_error: None,
        denial_rate: Some(denial_rate),
        defensive_value: Some(0.1),
        opportunity_cost: Some(0.2),
        useful_candidate: false,
        status_label: Some("Baseline".to_string()),
        warnings: vec![],
        reason_codes: vec![],
    }
}

#[test]
fn equal_denial_rates_are_consistent() {
    let comparison = build_source_risk_interaction_comparison(
        Some(&summary(0.5, 0.1)),
        Some(&summary(0.5, 0.1)),
        0.1,
    );
    assert!(comparison.risk_behavior_consistent);
}

#[test]
fn large_denial_rate_delta_warns() {
    let comparison = build_source_risk_interaction_comparison(
        Some(&summary(0.6, 0.1)),
        Some(&summary(0.2, 0.1)),
        0.1,
    );
    assert!(
        comparison
            .warnings
            .iter()
            .any(|warning| warning.contains("denial-rate"))
    );
}

#[test]
fn lower_denial_and_worse_drawdown_warns() {
    let comparison = build_source_risk_interaction_comparison(
        Some(&summary(0.5, 0.1)),
        Some(&summary(0.2, 0.4)),
        0.5,
    );
    assert!(
        comparison
            .warnings
            .iter()
            .any(|warning| warning.contains("worse drawdown"))
    );
}
