#[path = "support/sprint47_support.rs"]
mod sprint47_support;

use soma_zero::BatchOutcomeLinkageV3Runner;

#[test]
fn batch_outcomes_generate_for_multiple_rows() {
    let config = sprint47_support::example_batch_outcome("batch-outcome-linkage");
    let first = BatchOutcomeLinkageV3Runner::default()
        .run(&config)
        .expect("first report");
    let second = BatchOutcomeLinkageV3Runner::default()
        .run(&config)
        .expect("second report");
    assert_eq!(first.eligible_rows, 2);
    assert_eq!(first.generated_outcome_count, 2);
    assert_eq!(first.official_outcome_count, 2);
    assert!(
        first
            .records
            .iter()
            .all(|record| record.cost_bps > 0.0 && record.slippage_bps > 0.0)
    );
    assert_eq!(first.to_text(), second.to_text());
}
