mod common;
#[path = "support/sprint66_support.rs"]
mod sprint66_support;

use soma_zero::{ModelReviewClosureRunner, PredictionHistoryPackConfig};

#[test]
fn prediction_history_pack_registers_multiple_versions_and_reaches_ready_status() {
    let config = sprint66_support::prediction_history_config_from_example(
        "soma_prediction_history_pack.toml",
        "prediction-history-ready",
    );
    let report = ModelReviewClosureRunner::default()
        .run_prediction_history_pack(&config)
        .expect("run prediction history pack");
    assert_eq!(report.model_count, 2);
    assert_eq!(report.version_count, 4);
    assert_eq!(report.comparable_version_count, 4);
    assert_eq!(
        report.history_status,
        soma_zero::PredictionHistoryPackStatus::PredictionHistoryPackReady
    );
}

#[test]
fn prediction_history_pack_detects_contract_mismatch_and_missing_inputs() {
    let mut config = sprint66_support::prediction_history_config_from_example(
        "soma_prediction_history_pack.toml",
        "prediction-history-mismatch",
    );
    let mut card: soma_zero::ExternalModelCardV2 =
        sprint66_support::read_json(&config.model_card_paths[2]);
    card.feature_schema_hash = "schema-mismatch".to_string();
    config.model_card_paths[2] = sprint66_support::write_support_json(
        "prediction-history-mismatch",
        "model_card_a_v3.json",
        &card,
    );
    let report = ModelReviewClosureRunner::default()
        .run_prediction_history_pack(&config)
        .expect("run mismatched history pack");
    assert_eq!(
        report.history_status,
        soma_zero::PredictionHistoryPackStatus::ContractMismatch
    );

    let mut missing = sprint66_support::prediction_history_config_from_example(
        "soma_prediction_history_pack.toml",
        "prediction-history-missing",
    );
    let mut shifted_card: soma_zero::ExternalModelCardV2 =
        sprint66_support::read_json(&missing.model_card_paths[2]);
    shifted_card.model_version = "9.9.9".to_string();
    missing.model_card_paths[2] = sprint66_support::write_support_json(
        "prediction-history-missing",
        "model_card_a_shifted.json",
        &shifted_card,
    );
    let report = ModelReviewClosureRunner::default()
        .run_prediction_history_pack(&missing)
        .expect("run missing history pack");
    assert!(report.missing_card_count > 0);
    assert!(report.missing_prediction_count > 0);
}

#[test]
fn prediction_history_pack_rejects_remote_paths() {
    let mut config = PredictionHistoryPackConfig::default();
    config.sequence_export_manifest_paths = vec!["https://example.com/manifest.json".to_string()];
    assert!(config.validate().is_err());
}
