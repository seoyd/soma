use soma_zero::experiment::campaign::CampaignAggregate;
use soma_zero::experiment::diff::{
    CampaignImprovement, CampaignRegression, build_campaign_diff_report,
};
use soma_zero::{ReasonCode, diff_report_to_text};

fn aggregate(
    campaign_id: &str,
    usable: usize,
    outcomes: usize,
    avg_net: f64,
    worst_drawdown: f64,
    calibration: Option<f64>,
    quality: f64,
    defensive: f64,
    denials: usize,
    no_trades: usize,
    redundancy: usize,
    regime_coverage: usize,
) -> CampaignAggregate {
    CampaignAggregate {
        campaign_id: campaign_id.to_string(),
        matrix_count: 1,
        total_runs: 3,
        passed_runs: 2,
        failed_runs: 0,
        skipped_runs: 0,
        usable_dataset_count: usable,
        total_dataset_count: usable,
        total_outcome_records: outcomes,
        total_executed_trades: outcomes / 2,
        total_no_trades: no_trades,
        total_denials: denials,
        average_data_quality_score: quality,
        worst_data_quality_score: quality,
        average_net_return_pct: avg_net,
        median_net_return_pct: avg_net,
        worst_net_return_pct: avg_net,
        average_max_drawdown_pct: worst_drawdown,
        worst_max_drawdown_pct: worst_drawdown,
        average_profit_factor: Some(1.2),
        average_calibration_brier: calibration,
        regime_coverage_count: regime_coverage,
        unknown_regime_rate: 0.0,
        panic_regime_rate: 0.0,
        risk_defensive_value_total: defensive,
        risk_opportunity_cost_total: 0.0,
        persona_redundancy_warning_count: redundancy,
        external_model_validated_count: 0,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

#[test]
fn missing_previous_report_is_not_comparable() {
    let current = aggregate(
        "current",
        2,
        40,
        0.02,
        0.05,
        Some(0.10),
        0.90,
        1.0,
        4,
        2,
        0,
        2,
    );
    let diff = build_campaign_diff_report(&current, None, None);
    assert!(!diff.comparable);
}

#[test]
fn net_return_improvement_alone_does_not_count_when_drawdown_worsens() {
    let current = aggregate(
        "current",
        2,
        40,
        0.04,
        0.20,
        Some(0.10),
        0.90,
        1.0,
        4,
        2,
        0,
        2,
    );
    let previous = aggregate("prev", 2, 40, 0.02, 0.05, Some(0.10), 0.90, 1.0, 4, 2, 0, 2);
    let diff = build_campaign_diff_report(&current, Some(&previous), Some("prev"));
    assert!(
        diff.regressions
            .contains(&CampaignRegression::DrawdownRegression)
    );
    assert!(
        !diff
            .improvements
            .contains(&CampaignImprovement::BetterNetReturn)
    );
}

#[test]
fn diff_detects_quality_calibration_and_coverage_changes_deterministically() {
    let current = aggregate(
        "current",
        3,
        60,
        0.03,
        0.04,
        Some(0.20),
        0.90,
        2.0,
        8,
        6,
        1,
        3,
    );
    let previous = aggregate("prev", 2, 40, 0.02, 0.04, Some(0.10), 0.90, 1.0, 4, 2, 0, 2);
    let diff = build_campaign_diff_report(&current, Some(&previous), Some("prev"));
    assert!(
        diff.regressions
            .contains(&CampaignRegression::CalibrationRegression)
    );
    assert!(
        diff.improvements
            .contains(&CampaignImprovement::MoreUsableData)
    );
    assert!(
        diff.improvements
            .contains(&CampaignImprovement::BetterRegimeCoverage)
    );
    assert_eq!(diff_report_to_text(&diff), diff_report_to_text(&diff));
}
