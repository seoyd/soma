mod common;
#[path = "support/sprint63_support.rs"]
mod sprint63_support;

use soma_zero::{ExternalPredictionEvaluationRunner, ExternalVsTrinityComparisonStatus};

#[test]
fn external_vs_trinity_is_deterministic_and_comparable() {
    let config =
        sprint63_support::import_config_from_example("soma_external_vs_trinity.toml", "vs-trinity");
    let report = ExternalPredictionEvaluationRunner::default()
        .run_comparison(&config)
        .expect("run comparison");
    assert_eq!(
        report.comparison_status,
        ExternalVsTrinityComparisonStatus::Mixed
    );
    assert!(report.comparable_rows >= 3);
}

#[test]
fn external_vs_trinity_handles_missing_references() {
    let mut config =
        sprint63_support::import_config_from_example("soma_external_vs_trinity.toml", "vs-none");
    config.trinity_reference_paths.clear();
    let report = ExternalPredictionEvaluationRunner::default()
        .run_comparison(&config)
        .expect("run no comparison");
    assert_eq!(
        report.comparison_status,
        ExternalVsTrinityComparisonStatus::NotComparable
    );
}
