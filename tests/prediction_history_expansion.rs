#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{PredictionHistoryExpansionPlanStatus, PredictionHistoryExpansionStatus};

#[test]
fn prediction_history_expansion_adds_conservative_history_without_clearing_gap() {
    let (plan, report) =
        support::run_prediction_history_expand("soma_prediction_history_expand.toml");

    assert_eq!(plan.target_models, vec!["ext-model-b:1.0.0".to_string()]);
    assert_eq!(report.model_id, "ext-model-b");
    assert_eq!(report.model_version, "1.0.0");
    assert!(report.prediction_files_before < report.prediction_files_after);
    assert_eq!(
        plan.plan_status,
        PredictionHistoryExpansionPlanStatus::PredictionHistoryExpansionReady
    );
    assert_eq!(
        report.expansion_status,
        PredictionHistoryExpansionStatus::PredictionHistoryExpanded
    );
}

#[test]
fn prediction_history_expansion_requires_local_sequence_context() {
    let mut config =
        support::prediction_history_config_from_example("soma_prediction_history_expand.toml");
    config.sequence_export_manifest_paths.clear();

    let (plan, report) = soma_zero::PredictionHistoryExpansionRunner::default()
        .run(&config)
        .expect("run without manifest");
    assert_eq!(
        plan.plan_status,
        PredictionHistoryExpansionPlanStatus::NeedSequenceManifest
    );
    assert_eq!(
        report.expansion_status,
        PredictionHistoryExpansionStatus::PredictionHistoryExpanded
    );
}
