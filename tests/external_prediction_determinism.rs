mod common;
#[path = "support/sprint63_support.rs"]
mod sprint63_support;

use soma_zero::ExternalPredictionEvaluationRunner;

#[test]
fn same_fixture_input_produces_same_bundle_summary() {
    let first = sprint63_support::import_config_from_example(
        "soma_external_prediction_import_v2_valid.toml",
        "determinism-first",
    );
    let second = sprint63_support::import_config_from_example(
        "soma_external_prediction_import_v2_valid.toml",
        "determinism-second",
    );
    let first_bundle = ExternalPredictionEvaluationRunner::default()
        .run(&first)
        .expect("run first bundle");
    let second_bundle = ExternalPredictionEvaluationRunner::default()
        .run(&second)
        .expect("run second bundle");
    assert_eq!(first_bundle.import_report, second_bundle.import_report);
    assert_eq!(
        first_bundle.external_model_evaluation_report,
        second_bundle.external_model_evaluation_report
    );
    assert_eq!(
        first_bundle.external_model_promotion_gate_report,
        second_bundle.external_model_promotion_gate_report
    );
    assert_eq!(first_bundle.final_summary, second_bundle.final_summary);
}
