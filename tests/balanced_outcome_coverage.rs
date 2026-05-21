#[path = "support/sprint48_support.rs"]
mod support;

use soma_zero::{BalancedOutcomeCoverageRunner, BalancedOutcomeCoverageStatus};

#[test]
fn coverage_cells_are_grouped_by_market_symbol_timeframe_horizon_and_profile() {
    let config = support::balanced_coverage_config("coverage-grouping");
    let report = BalancedOutcomeCoverageRunner::default()
        .run(&config)
        .expect("coverage report");

    assert_eq!(report.cells.len(), 4);
    assert!(report.cells.iter().any(|cell| {
        cell.symbol == "AAPL"
            && cell.timeframe == "4h"
            && cell.horizon_bars == 5
            && cell.barrier_profile_id == "primary-preregistered"
    }));
}

#[test]
fn take_profit_stop_loss_and_time_expired_counts_are_computed() {
    let config = support::balanced_coverage_config("coverage-outcome-counts");
    let report = BalancedOutcomeCoverageRunner::default()
        .run(&config)
        .expect("coverage report");

    assert_eq!(report.total_take_profit, 2);
    assert_eq!(report.total_stop_loss, 1);
    assert_eq!(report.total_time_expired, 1);
}

#[test]
fn no_trade_and_risk_denied_counterfactual_counts_are_computed() {
    let config = support::balanced_coverage_config("coverage-counterfactual-counts");
    let report = BalancedOutcomeCoverageRunner::default()
        .run(&config)
        .expect("coverage report");

    assert_eq!(report.total_no_trade_counterfactuals, 4);
    assert_eq!(report.total_risk_denied_counterfactuals, 4);
}

#[test]
fn balanced_enough_status_requires_all_configured_minima() {
    let mut config = support::balanced_coverage_config("coverage-minima");
    let healthy = BalancedOutcomeCoverageRunner::default()
        .run(&config)
        .expect("healthy coverage report");
    assert_eq!(
        healthy.coverage_status,
        BalancedOutcomeCoverageStatus::BalancedEnoughForResearchBenchmark
    );

    config.min_no_trade_counterfactuals = 5;
    let insufficient = BalancedOutcomeCoverageRunner::default()
        .run(&config)
        .expect("insufficient coverage report");
    assert_eq!(
        insufficient.coverage_status,
        BalancedOutcomeCoverageStatus::NeedMoreCounterfactuals
    );
}

#[test]
fn coverage_report_is_deterministic() {
    let config = support::balanced_coverage_config("coverage-deterministic");

    let first = BalancedOutcomeCoverageRunner::default()
        .run(&config)
        .expect("first coverage report");
    let second = BalancedOutcomeCoverageRunner::default()
        .run(&config)
        .expect("second coverage report");

    assert_eq!(first, second);
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(first.fingerprint(), second.fingerprint());
}
