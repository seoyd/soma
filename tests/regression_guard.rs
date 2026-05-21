use soma_zero::ReasonCode;
use soma_zero::experiment::campaign::CampaignAggregate;
use soma_zero::experiment::diff::build_campaign_diff_report;
use soma_zero::experiment::regression::{RegressionGuardConfig, evaluate_regression_guard};

fn aggregate(
    campaign_id: &str,
    outcomes: usize,
    avg_net: f64,
    drawdown: f64,
    calibration: f64,
    quality: f64,
    denials: usize,
    no_trades: usize,
) -> CampaignAggregate {
    CampaignAggregate {
        campaign_id: campaign_id.to_string(),
        matrix_count: 1,
        total_runs: 3,
        passed_runs: 2,
        failed_runs: 0,
        skipped_runs: 0,
        usable_dataset_count: 2,
        total_dataset_count: 2,
        total_outcome_records: outcomes,
        total_executed_trades: outcomes / 2,
        total_no_trades: no_trades,
        total_denials: denials,
        average_data_quality_score: quality,
        worst_data_quality_score: quality,
        average_net_return_pct: avg_net,
        median_net_return_pct: avg_net,
        worst_net_return_pct: avg_net,
        average_max_drawdown_pct: drawdown,
        worst_max_drawdown_pct: drawdown,
        average_profit_factor: Some(1.2),
        average_calibration_brier: Some(calibration),
        regime_coverage_count: 2,
        unknown_regime_rate: 0.0,
        panic_regime_rate: 0.0,
        risk_defensive_value_total: 1.0,
        risk_opportunity_cost_total: 0.0,
        persona_redundancy_warning_count: 0,
        external_model_validated_count: 0,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

#[test]
fn regression_guard_passes_without_regressions() {
    let previous = aggregate("prev", 40, 0.02, 0.05, 0.20, 0.90, 4, 2);
    let current = aggregate("current", 40, 0.03, 0.04, 0.18, 0.91, 4, 2);
    let diff = build_campaign_diff_report(&current, Some(&previous), Some("prev"));
    let guard = evaluate_regression_guard(
        &RegressionGuardConfig::default(),
        &current,
        Some(&previous),
        &diff,
    );
    assert!(guard.passed);
}

#[test]
fn regression_guard_fails_on_drawdown_and_calibration_regressions() {
    let previous = aggregate("prev", 40, 0.02, 0.05, 0.10, 0.90, 4, 2);
    let current = aggregate("current", 40, 0.03, 0.20, 0.20, 0.90, 4, 2);
    let diff = build_campaign_diff_report(&current, Some(&previous), Some("prev"));
    let guard = evaluate_regression_guard(
        &RegressionGuardConfig::default(),
        &current,
        Some(&previous),
        &diff,
    );
    assert!(!guard.passed);
    assert!(!guard.regressions.is_empty());
}

#[test]
fn regression_guard_warns_on_large_denial_rate_change_and_is_conservative_on_low_samples() {
    let previous = aggregate("prev", 10, 0.02, 0.05, 0.10, 0.90, 1, 1);
    let current = aggregate("current", 10, 0.02, 0.05, 0.10, 0.90, 8, 1);
    let diff = build_campaign_diff_report(&current, Some(&previous), Some("prev"));
    let guard = evaluate_regression_guard(
        &RegressionGuardConfig::default(),
        &current,
        Some(&previous),
        &diff,
    );
    assert!(!guard.passed);
    assert!(!guard.warnings.is_empty());
}
