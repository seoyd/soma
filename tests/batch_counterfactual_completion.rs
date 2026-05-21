#[path = "support/sprint47_support.rs"]
mod sprint47_support;

use soma_zero::BatchCounterfactualCompletionRunner;

#[test]
fn batch_counterfactuals_build_for_multiple_rows() {
    let config = sprint47_support::example_batch_counterfactual("batch-counterfactuals");
    let first = BatchCounterfactualCompletionRunner::default()
        .run(&config)
        .expect("first report");
    let second = BatchCounterfactualCompletionRunner::default()
        .run(&config)
        .expect("second report");
    assert_eq!(first.eligible_rows, 2);
    assert_eq!(first.no_trade_built_count, 2);
    assert_eq!(first.risk_denied_built_count, 2);
    assert!(
        first
            .records
            .iter()
            .all(|record| record.no_trade_counterfactual_built
                && record.risk_denied_counterfactual_built)
    );
    assert_eq!(first.to_text(), second.to_text());
}
