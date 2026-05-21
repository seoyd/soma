mod common;
#[path = "support/sprint63_support.rs"]
mod sprint63_support;
#[path = "support/sprint64_support.rs"]
mod sprint64_support;

use std::fs;

use soma_zero::{
    ExternalArtifactRegistryRunner, ExternalModelCardValidationStatus,
    ExternalModelEvaluationReport, ExternalModelEvaluationStatus, ExternalModelPromotionGateStatus,
    ExternalPredictionCsvSchemaV2, ExternalPredictionEvaluationRunner,
    ExternalPredictionImportStatus, PreviousExternalComparisonRecommendation,
};

#[test]
fn import_defaults_valid_import_and_limits_work() {
    let config = soma_zero::ExternalPredictionImportV2Config::default();
    assert!(config.require_model_card);
    assert!(config.require_sequence_id_match);
    assert!(config.require_model_version);
    assert!(config.require_no_duplicate_sequence_predictions);
    assert!(config.require_probability_sanity);

    let mut bad = config.clone();
    bad.import_id = "remote".to_string();
    bad.sequence_export_manifest_path = "https://example.com/manifest.json".to_string();
    bad.prediction_csv_paths = vec!["https://example.com/preds.csv".to_string()];
    assert!(bad.validate().is_err());

    let config = sprint63_support::import_config_from_example(
        "soma_external_prediction_import_v2_valid.toml",
        "external-suite-valid",
    );
    let bundle = ExternalPredictionEvaluationRunner::default()
        .run(&config)
        .expect("run valid import");
    assert_eq!(
        bundle.import_report.import_status,
        ExternalPredictionImportStatus::ExternalPredictionImportReady
    );
    assert_ne!(bundle.storage_report.storage_bytes, 0);
}

#[test]
fn schema_model_card_and_probability_rejections_hold() {
    let schema = ExternalPredictionCsvSchemaV2::default();
    let errors = schema.validate_header(&[
        "sequence_id".to_string(),
        "model_id".to_string(),
        "model_version".to_string(),
        "prediction_timestamp_ms".to_string(),
    ]);
    assert!(errors.is_empty());
    assert!(
        !schema
            .validate_header(&["account_id".to_string(), "order_id".to_string()])
            .is_empty()
    );

    let config = sprint63_support::import_config_from_example(
        "soma_external_prediction_import_v2_bad_schema.toml",
        "external-suite-bad-schema",
    );
    let bundle = ExternalPredictionEvaluationRunner::default()
        .run(&config)
        .expect("run bad schema bundle");
    assert!(!bundle.import_report.header_errors.is_empty());
    assert!(bundle.import_report.invalid_prediction_rows > 0);

    let invalid_prob_csv = sprint63_support::write_support_file(
        "external-suite-invalid-prob",
        "invalid_prob.csv",
        "sequence_id,model_id,model_version,prediction_timestamp_ms,p_take_profit,p_stop_loss,p_time_expired,p_win,expected_return_pct,expected_drawdown_pct,confidence,predicted_label,rank_score,reason_code\n70169ad73045cb5d,mamba3fin-lite-ext,0.1.0-diagnostic,1714611600000,0.90,0.10,0.10,1.20,0.010,0.005,1.20,TakeProfit,0.50,\n",
    );
    let mut config = sprint63_support::import_config_from_example(
        "soma_external_prediction_import_v2_valid.toml",
        "external-suite-invalid-prob-run",
    );
    config.prediction_csv_paths = vec![invalid_prob_csv];
    let bundle = ExternalPredictionEvaluationRunner::default()
        .run(&config)
        .expect("run invalid prob bundle");
    assert!(bundle.import_report.invalid_prediction_rows > 0);

    let missing = sprint63_support::import_config_from_example(
        "soma_external_prediction_import_v2_missing_model_card.toml",
        "external-suite-missing-card",
    );
    let bundle = ExternalPredictionEvaluationRunner::default()
        .run(&missing)
        .expect("run missing card bundle");
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
fn sequence_duplicates_evaluation_and_promotion_gate_stay_conservative() {
    let csv = sprint63_support::write_support_file(
        "external-suite-unknown-duplicate",
        "unknown_duplicate.csv",
        "sequence_id,model_id,model_version,prediction_timestamp_ms,p_take_profit,p_stop_loss,p_time_expired,p_win,expected_return_pct,expected_drawdown_pct,confidence,predicted_label,rank_score,reason_code\nunknown-seq,mamba3fin-lite-ext,0.1.0-diagnostic,1714611600000,0.60,0.20,0.20,0.60,0.010,0.005,0.60,TakeProfit,0.80,\nunknown-seq,mamba3fin-lite-ext,0.1.0-diagnostic,1714611600001,0.55,0.25,0.20,0.55,0.009,0.005,0.60,TakeProfit,0.79,\n",
    );
    let mut config = sprint63_support::import_config_from_example(
        "soma_external_prediction_import_v2_valid.toml",
        "external-suite-unknown-duplicate-run",
    );
    config.prediction_csv_paths = vec![csv];
    let bundle = ExternalPredictionEvaluationRunner::default()
        .run(&config)
        .expect("run unknown sequence bundle");
    assert!(bundle.prediction_coverage_report.extra_sequence_count > 0);
    assert!(bundle.prediction_coverage_report.duplicate_prediction_count > 0);

    let config = sprint63_support::import_config_from_example(
        "soma_external_model_evaluate.toml",
        "external-suite-evaluation-valid",
    );
    let report = ExternalPredictionEvaluationRunner::default()
        .run_evaluation(&config)
        .expect("run evaluation");
    assert_eq!(
        report.evaluation_status,
        ExternalModelEvaluationStatus::EvaluationReady
    );

    let valid = sprint63_support::import_config_from_example(
        "soma_external_model_promotion_gate.toml",
        "external-suite-promotion-valid",
    );
    let report = ExternalPredictionEvaluationRunner::default()
        .run_promotion_gate(&valid)
        .expect("run promotion gate");
    assert_eq!(
        report.gate_status,
        ExternalModelPromotionGateStatus::ResearchCandidate
    );
}

#[test]
fn previous_external_comparison_and_secret_style_columns_stay_offline_only() {
    let config = sprint64_support::registry_config_from_example(
        "soma_external_artifact_registry.toml",
        "external-suite-previous-comparison",
    );
    let report = ExternalArtifactRegistryRunner::default()
        .run(&config)
        .expect("run registry bundle")
        .previous_external_comparison_report;
    assert_eq!(
        report.recommendation,
        PreviousExternalComparisonRecommendation::DowngradeToDiagnostic
    );

    let mut config = sprint64_support::registry_config_from_example(
        "soma_external_artifact_registry.toml",
        "external-suite-previous-comparison-drift",
    );
    config.external_prediction_import_report_paths.truncate(2);
    config.external_model_card_paths.truncate(2);
    config.external_model_evaluation_report_paths.truncate(2);
    config.external_prediction_ablation_report_paths.truncate(2);
    config.external_model_promotion_gate_paths.truncate(2);
    let eval_path = sprint64_support::absolutize("examples/sprint64_data/evaluation_a_v2.json");
    let mut eval: ExternalModelEvaluationReport =
        serde_json::from_str(&fs::read_to_string(eval_path).expect("read evaluation"))
            .expect("parse evaluation");
    eval.calibration_metrics.brier_score = Some(0.18);
    eval.calibration_metrics.ece = Some(0.45);
    let severe_eval = sprint64_support::write_support_json(
        "external-suite-previous-comparison-drift",
        "evaluation_a_v2.json",
        &eval,
    );
    config.external_model_evaluation_report_paths[1] = severe_eval;
    let report = ExternalArtifactRegistryRunner::default()
        .run(&config)
        .expect("run severe drift bundle")
        .previous_external_comparison_report;
    assert_eq!(
        report.recommendation,
        PreviousExternalComparisonRecommendation::BlockedByDrift
    );
}
