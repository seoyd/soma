mod common;
#[path = "support/sprint63_support.rs"]
mod sprint63_support;
#[path = "support/sprint64_support.rs"]
mod sprint64_support;
#[path = "support/sprint69_support.rs"]
mod sprint69_support;

use std::fs;

use serde_json::Value;
use soma_zero::{
    ExtModelBPredictionClosureRunner, ExtModelBPredictionClosureStatus,
    ExternalArtifactRegistryRunner, ExternalModelCardValidationStatus,
    ExternalModelEvaluationReport, ExternalModelPromotionGateStatus,
    ExternalPredictionAblationStatus, ExternalPredictionCsvSchemaV2,
    ExternalPredictionEvaluationRunner, ExternalPredictionImportStatus,
    PreviousExternalComparisonRecommendation,
};

#[test]
fn external_prediction_import_schema_and_model_card_guards_hold() {
    let config = sprint63_support::import_config_from_example(
        "soma_external_prediction_import_v2_valid.toml",
        "external-family-suite-valid",
    );
    let bundle = ExternalPredictionEvaluationRunner::default()
        .run(&config)
        .expect("run valid import");
    assert_eq!(
        bundle.import_report.import_status,
        ExternalPredictionImportStatus::ExternalPredictionImportReady
    );

    let schema = ExternalPredictionCsvSchemaV2::default();
    assert!(
        !schema
            .validate_header(&["account_id".to_string(), "order_id".to_string()])
            .is_empty()
    );

    let missing = sprint63_support::import_config_from_example(
        "soma_external_prediction_import_v2_missing_model_card.toml",
        "external-family-suite-missing-card",
    );
    let bundle = ExternalPredictionEvaluationRunner::default()
        .run(&missing)
        .expect("run missing card");
    assert_eq!(
        bundle.model_card_validation_report.validation_status,
        ExternalModelCardValidationStatus::MissingModelCard
    );
    assert_eq!(
        bundle.external_model_promotion_gate_report.gate_status,
        ExternalModelPromotionGateStatus::BlockedByModelCard
    );
}

#[test]
fn external_prediction_ablation_and_comparison_stay_research_only() {
    let config = sprint63_support::import_config_from_example(
        "soma_external_prediction_ablation.toml",
        "external-family-suite-ablation",
    );
    let report = ExternalPredictionEvaluationRunner::default()
        .run_ablation(&config)
        .expect("run ablation");
    assert!(matches!(
        report.ablation_status,
        ExternalPredictionAblationStatus::Stable | ExternalPredictionAblationStatus::Sensitive
    ));

    let config = sprint64_support::registry_config_from_example(
        "soma_external_artifact_registry.toml",
        "external-family-suite-comparison",
    );
    let report = ExternalArtifactRegistryRunner::default()
        .run(&config)
        .expect("run registry")
        .previous_external_comparison_report;
    assert_eq!(
        report.recommendation,
        PreviousExternalComparisonRecommendation::DowngradeToDiagnostic
    );
}

#[test]
fn ext_model_b_closure_preserves_remote_rejection_duplicate_detection_and_expected_fixture() {
    let bundle = sprint69_support::run_sprint73_bundle(
        "soma_ext_model_b_prediction_close.toml",
        "external-family-suite-ext-model-b",
    );
    let expected: Value = sprint69_support::read_json(sprint69_support::example_path(
        "sprint73_data/expected_prediction_closure.json",
    ));
    let report = bundle.ext_model_b_prediction_closure_report;
    assert_eq!(expected["closure_id"], report.closure_id);
    assert_eq!(
        report.closure_status,
        ExtModelBPredictionClosureStatus::PredictionGapClosed
    );

    let mut remote = sprint69_support::sprint73_config_from_example(
        "soma_ext_model_b_prediction_close.toml",
        "external-family-suite-ext-model-b-remote",
    );
    remote.new_prediction_csv_paths = vec!["https://example.com/predictions.csv".to_string()];
    assert!(remote.validate().is_err());

    let mut duplicate = sprint69_support::sprint73_config_from_example(
        "soma_ext_model_b_prediction_close.toml",
        "external-family-suite-ext-model-b-duplicate",
    );
    let existing = duplicate.new_prediction_csv_paths[0].clone();
    duplicate.new_prediction_csv_paths.push(existing);
    let report = ExtModelBPredictionClosureRunner::default()
        .run_prediction_closure(&duplicate)
        .expect("run duplicate closure");
    assert_eq!(
        report.closure_status,
        ExtModelBPredictionClosureStatus::InvalidPredictions
    );
    assert_eq!(report.duplicate_count, 1);
}

#[test]
fn external_prediction_family_is_deterministic_under_same_fixture_inputs() {
    let config = sprint64_support::registry_config_from_example(
        "soma_external_artifact_registry.toml",
        "external-family-suite-deterministic",
    );
    let first = ExternalArtifactRegistryRunner::default()
        .run(&config)
        .expect("first");
    let second = ExternalArtifactRegistryRunner::default()
        .run(&config)
        .expect("second");
    let first_text =
        serde_json::to_string(&first.previous_external_comparison_report).expect("json");
    let second_text =
        serde_json::to_string(&second.previous_external_comparison_report).expect("json");
    assert_eq!(first_text, second_text);

    let eval_path = sprint64_support::absolutize("examples/sprint64_data/evaluation_a_v2.json");
    let eval: ExternalModelEvaluationReport =
        serde_json::from_str(&fs::read_to_string(eval_path).expect("read")).expect("parse");
    assert!(eval.calibration_metrics.brier_score.is_some());
}
