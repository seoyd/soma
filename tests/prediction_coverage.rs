mod common;
#[path = "support/sprint63_support.rs"]
mod sprint63_support;

use soma_zero::{ExternalPredictionEvaluationRunner, PredictionCoverageStatus};

#[test]
fn prediction_coverage_full_and_partial_work() {
    let full = sprint63_support::import_config_from_example(
        "soma_external_prediction_import_v2_valid.toml",
        "coverage-full",
    );
    let bundle = ExternalPredictionEvaluationRunner::default()
        .run(&full)
        .expect("run full coverage bundle");
    assert_eq!(
        bundle.prediction_coverage_report.coverage_status,
        PredictionCoverageStatus::FullCoverage
    );

    let partial = sprint63_support::import_config_from_example(
        "soma_external_prediction_import_v2_valid.toml",
        "coverage-partial",
    );
    let mut partial = partial;
    partial.prediction_csv_paths = vec![
        sprint63_support::example_path("sprint63_data/external_predictions_missing_rows.csv")
            .display()
            .to_string(),
    ];
    let bundle = ExternalPredictionEvaluationRunner::default()
        .run(&partial)
        .expect("run partial coverage bundle");
    assert_eq!(
        bundle.prediction_coverage_report.coverage_status,
        PredictionCoverageStatus::PartialCoverage
    );
    assert_eq!(bundle.prediction_coverage_report.missing_sequence_count, 2);
}
