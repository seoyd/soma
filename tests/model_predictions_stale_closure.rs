#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{ModelPredictionsStaleClosureStatus, RealEvidencePredictionRefreshRunner};

#[test]
fn stale_warning_closes_when_predictions_refresh_validates() {
    let config = support::sprint75_config_from_example(
        "soma_model_predictions_stale_close.toml",
        "stale-close",
    );
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_stale_closure(&config)
        .expect("stale closure");
    assert_eq!(
        report.closure_status,
        ModelPredictionsStaleClosureStatus::StaleClosed
    );
    assert!(report.remaining_models.is_empty());
}

#[test]
fn stale_warning_is_explained_when_predictions_are_missing() {
    let mut config = support::sprint75_config_from_example(
        "soma_model_predictions_stale_close.toml",
        "stale-explained",
    );
    config.new_prediction_csv_paths.clear();
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_stale_closure(&config)
        .expect("stale closure");
    assert_eq!(
        report.closure_status,
        ModelPredictionsStaleClosureStatus::StaleExplained
    );
    assert_eq!(
        report.remaining_models,
        vec!["ext-model-b:1.0.0".to_string()]
    );
}
