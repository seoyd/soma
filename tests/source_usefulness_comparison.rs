use soma_zero::{SourceBenchmarkSummary, build_source_model_usefulness_comparison};

fn summary(useful: bool, status: &str) -> SourceBenchmarkSummary {
    SourceBenchmarkSummary {
        source_label: "test".to_string(),
        dataset_count: 1,
        total_outcome_records: 100,
        avg_net_return_pct: Some(1.0),
        avg_max_drawdown_pct: Some(0.1),
        avg_brier_score: Some(0.1),
        avg_expected_calibration_error: Some(0.05),
        denial_rate: Some(0.4),
        defensive_value: Some(0.1),
        opportunity_cost: Some(0.2),
        useful_candidate: useful,
        status_label: Some(status.to_string()),
        warnings: vec![],
        reason_codes: vec![],
    }
}

#[test]
fn yfinance_useful_candidate_does_not_imply_official_useful_candidate() {
    let comparison = build_source_model_usefulness_comparison(
        Some(&summary(false, "Baseline")),
        Some(&summary(true, "UsefulCandidate")),
        true,
        true,
        true,
    );
    assert!(!comparison.can_generalize_from_yfinance_to_official);
}

#[test]
fn official_missing_keeps_yfinance_research_only() {
    let comparison = build_source_model_usefulness_comparison(
        None,
        Some(&summary(true, "UsefulCandidate")),
        true,
        true,
        true,
    );
    assert!(
        comparison
            .warnings
            .iter()
            .any(|warning| warning.contains("research-only"))
    );
}

#[test]
fn low_mismatch_and_agreement_can_generalize() {
    let comparison = build_source_model_usefulness_comparison(
        Some(&summary(true, "UsefulCandidate")),
        Some(&summary(true, "UsefulCandidate")),
        true,
        true,
        true,
    );
    assert!(comparison.can_generalize_from_yfinance_to_official);
}

#[test]
fn high_mismatch_blocks_generalization() {
    let comparison = build_source_model_usefulness_comparison(
        Some(&summary(true, "UsefulCandidate")),
        Some(&summary(true, "UsefulCandidate")),
        false,
        true,
        true,
    );
    assert!(!comparison.can_generalize_from_yfinance_to_official);
}
