mod common;

use soma_zero::{ExperimentRunner, experiment::aggregate::build_data_quality_aggregate};

#[test]
fn data_quality_aggregate_counts_and_picks_worst_dataset_deterministically() {
    let valid = ExperimentRunner::default().run(&common::baseline_config(
        "dq-valid",
        "generic_ohlcv_valid.csv",
    ));
    let bad = ExperimentRunner::default().run(&common::baseline_config(
        "dq-bad",
        "generic_ohlcv_bad_ohlc.csv",
    ));

    let aggregate = build_data_quality_aggregate(&[
        ("valid_fixture".to_string(), &valid),
        ("bad_fixture".to_string(), &bad),
    ]);

    assert_eq!(aggregate.dataset_count, 2);
    assert_eq!(aggregate.good_count, 1);
    assert!(aggregate.bad_count + aggregate.unusable_count >= 1);
    assert_eq!(aggregate.worst_dataset_id.as_deref(), Some("bad_fixture"));
    assert!(!aggregate.common_reason_codes.is_empty());
}
