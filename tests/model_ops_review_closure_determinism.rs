mod common;
#[path = "support/sprint66_support.rs"]
mod sprint66_support;

use soma_zero::ModelReviewClosureRunner;

#[test]
fn model_review_closure_bundle_is_deterministic() {
    let config = sprint66_support::closure_config_from_example(
        "soma_model_review_close.toml",
        "model-review-closure-deterministic",
    );
    let runner = ModelReviewClosureRunner::default();
    let first = runner.run(&config).expect("first closure bundle");
    let second = runner.run(&config).expect("second closure bundle");
    assert_eq!(first, second);
}

#[test]
fn prediction_history_pack_report_is_deterministic() {
    let config = sprint66_support::prediction_history_config_from_example(
        "soma_prediction_history_pack.toml",
        "prediction-history-deterministic",
    );
    let runner = ModelReviewClosureRunner::default();
    let first = runner
        .run_prediction_history_pack(&config)
        .expect("first history pack");
    let second = runner
        .run_prediction_history_pack(&config)
        .expect("second history pack");
    assert_eq!(first, second);
}
