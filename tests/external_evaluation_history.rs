mod common;
#[path = "support/sprint64_support.rs"]
mod sprint64_support;

use soma_zero::{ExternalArtifactRegistryRunner, ExternalEvaluationHistoryStatus};

#[test]
fn history_builds_with_latest_and_previous_versions() {
    let config = sprint64_support::registry_config_from_example(
        "soma_external_evaluation_history.toml",
        "history-default",
    );
    let report = ExternalArtifactRegistryRunner::default()
        .run_history(&config)
        .expect("run evaluation history");
    assert_eq!(
        report.history_status,
        ExternalEvaluationHistoryStatus::NeedMoreVersions
    );
    assert_eq!(
        report
            .latest_model_versions
            .get("ext-model-a")
            .map(String::as_str),
        Some("1.1.0")
    );
    assert_eq!(
        report
            .previous_model_versions
            .get("ext-model-a")
            .map(String::as_str),
        Some("1.0.0")
    );
}

#[test]
fn history_ready_when_only_models_with_multiple_versions_are_kept() {
    let mut config = sprint64_support::registry_config_from_example(
        "soma_external_evaluation_history.toml",
        "history-ready",
    );
    config.external_prediction_import_report_paths.truncate(2);
    config.external_model_card_paths.truncate(2);
    config.external_model_evaluation_report_paths.truncate(2);
    config.external_prediction_ablation_report_paths.truncate(2);
    config.external_model_promotion_gate_paths.truncate(2);

    let report = ExternalArtifactRegistryRunner::default()
        .run_history(&config)
        .expect("run ready history");
    assert_eq!(
        report.history_status,
        ExternalEvaluationHistoryStatus::HistoryReady
    );
    assert_eq!(report.model_histories.len(), 1);
}
