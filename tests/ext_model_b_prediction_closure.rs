#[path = "support/sprint69_support.rs"]
mod support;

use serde_json::Value;
use soma_zero::ExtModelBPredictionClosureStatus;

#[test]
fn ext_model_b_prediction_closure_matches_expected_fixture() {
    let bundle = support::run_sprint73_bundle(
        "soma_ext_model_b_prediction_close.toml",
        "ext-model-b-prediction-closure",
    );
    let expected: Value = support::read_json(support::example_path(
        "sprint73_data/expected_prediction_closure.json",
    ));
    let report = bundle.ext_model_b_prediction_closure_report;

    assert_eq!(expected["closure_id"], report.closure_id);
    assert_eq!(
        expected["prediction_files_before"].as_u64().unwrap(),
        report.prediction_files_before as u64
    );
    assert_eq!(
        expected["prediction_files_after"].as_u64().unwrap(),
        report.prediction_files_after as u64
    );
    assert_eq!(
        expected["coverage_before"].as_f64().unwrap(),
        report.coverage_before
    );
    assert_eq!(
        expected["coverage_after"].as_f64().unwrap(),
        report.coverage_after
    );
    assert_eq!(
        expected["missing_sequence_count_after"].as_u64().unwrap(),
        report.missing_sequence_count_after as u64
    );
    assert_eq!(
        report.closure_status,
        ExtModelBPredictionClosureStatus::PredictionGapClosed
    );
}

#[test]
fn ext_model_b_prediction_closure_rejects_remote_paths_and_detects_invalid_duplicates() {
    let mut remote = support::sprint73_config_from_example(
        "soma_ext_model_b_prediction_close.toml",
        "ext-model-b-prediction-closure-remote",
    );
    remote.new_prediction_csv_paths = vec!["https://example.com/predictions.csv".to_string()];
    assert!(remote.validate().is_err());

    let mut duplicate = support::sprint73_config_from_example(
        "soma_ext_model_b_prediction_close.toml",
        "ext-model-b-prediction-closure-duplicate",
    );
    let existing = duplicate.new_prediction_csv_paths[0].clone();
    duplicate.new_prediction_csv_paths.push(existing);
    let report = soma_zero::ExtModelBPredictionClosureRunner::default()
        .run_prediction_closure(&duplicate)
        .expect("run duplicate closure");
    assert_eq!(
        report.closure_status,
        ExtModelBPredictionClosureStatus::InvalidPredictions
    );
    assert_eq!(report.duplicate_count, 1);
}

#[test]
fn ext_model_b_prediction_closure_blocks_when_model_card_is_missing() {
    let mut config = support::sprint73_config_from_example(
        "soma_ext_model_b_prediction_close.toml",
        "ext-model-b-prediction-closure-missing-card",
    );
    config.model_card_paths.clear();
    let report = soma_zero::ExtModelBPredictionClosureRunner::default()
        .run_prediction_closure(&config)
        .expect("run missing card closure");
    assert_eq!(
        report.closure_status,
        ExtModelBPredictionClosureStatus::MissingModelCard
    );
}
