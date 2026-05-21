mod common;
#[path = "support/sprint60_support.rs"]
mod sprint60_support;

use soma_zero::{CounterfactualCoverageStatus, EvidenceHardeningRunner};

#[test]
fn counterfactual_depth_and_totals_are_preserved() {
    let config = sprint60_support::config_from_example(
        "soma_counterfactual_coverage.toml",
        "counterfactual-coverage",
    );
    let report = EvidenceHardeningRunner::default()
        .run(&config)
        .expect("run counterfactual coverage")
        .counterfactual_coverage_report;
    assert_eq!(
        report.coverage_status,
        CounterfactualCoverageStatus::Healthy
    );
    assert_eq!(report.no_trade_depth, 2);
    assert_eq!(report.risk_denied_depth, 1);
    assert_eq!(report.avoided_loss_total, 42.5);
    assert_eq!(report.missed_gain_total, 13.0);
}
