#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::PredictionCoverageFinalizationStatus;

#[test]
fn prediction_coverage_finalization_reaches_full_fixture_coverage() {
    let bundle = support::run_sprint73_bundle(
        "soma_prediction_coverage_finalize.toml",
        "prediction-coverage-finalization",
    );
    let report = bundle.prediction_coverage_finalization_report;

    assert_eq!(report.total_sequences, 2);
    assert_eq!(report.predicted_sequences, 2);
    assert_eq!(report.missing_sequences, 0);
    assert_eq!(report.coverage_ratio, 1.0);
    assert!(report.coverage_passed);
    assert_eq!(
        report.coverage_status,
        PredictionCoverageFinalizationStatus::CoverageFinalized
    );
}

#[test]
fn prediction_coverage_finalization_stays_insufficient_when_threshold_is_higher() {
    let mut config = support::sprint73_config_from_example(
        "soma_prediction_coverage_finalize.toml",
        "prediction-coverage-finalization-insufficient",
    );
    config.min_coverage_ratio = 1.1;
    let report = soma_zero::ExtModelBPredictionClosureRunner::default()
        .run_prediction_coverage_finalization(&config)
        .expect("run coverage finalization");
    assert!(!report.coverage_passed);
    assert_eq!(
        report.coverage_status,
        PredictionCoverageFinalizationStatus::CoverageStillInsufficient
    );
}
