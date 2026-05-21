mod common;

use soma_zero::{
    ExperimentRunner, GovernorConfig, ReasonCode,
    experiment::aggregate::{build_risk_governor_aggregate, summarize_run},
};

#[test]
fn risk_governor_aggregate_counts_denials_and_is_stable() {
    let mut config = common::baseline_config("risk-aggregate", "generic_ohlcv_valid.csv");
    config.risk_config = GovernorConfig {
        min_confidence: 1.1,
        min_expected_edge: 10.0,
        max_spread_bps: 0.0,
        ..GovernorConfig::default()
    };
    let bundle = ExperimentRunner::default().run(&config);
    let summary = summarize_run("valid_fixture", "baseline_strict", &bundle);

    let aggregate = build_risk_governor_aggregate(
        &[summary.clone()],
        &[("valid_fixture".to_string(), &bundle)],
    );
    let aggregate_again =
        build_risk_governor_aggregate(&[summary], &[("valid_fixture".to_string(), &bundle)]);

    assert!(aggregate.total_denials > 0 || !aggregate.most_common_denial_reasons.is_empty());
    assert!(aggregate.defensive_value_total >= 0.0);
    assert!(aggregate.opportunity_cost_total >= 0.0);
    assert!(aggregate.deny_rate_by_dataset.contains_key("valid_fixture"));
    assert_eq!(aggregate, aggregate_again);
    assert_eq!(aggregate.reason_codes, vec![ReasonCode::DeterministicPath]);
}
