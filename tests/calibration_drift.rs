mod common;
#[path = "support/sprint64_support.rs"]
mod sprint64_support;

use std::fs;

use soma_zero::{
    CalibrationDriftStatus, ExternalArtifactRegistryRunner, ExternalModelEvaluationReport,
};

#[test]
fn stable_and_insufficient_history_are_detected() {
    let config = sprint64_support::registry_config_from_example(
        "soma_calibration_drift.toml",
        "drift-default",
    );
    let report = ExternalArtifactRegistryRunner::default()
        .run_calibration_drift(&config)
        .expect("run calibration drift");
    assert_eq!(report.drift_status, CalibrationDriftStatus::Stable);
    assert_eq!(report.stable_count, 1);
    assert_eq!(report.insufficient_history_count, 1);
}

#[test]
fn severe_drift_is_detected() {
    let mut config = sprint64_support::registry_config_from_example(
        "soma_calibration_drift.toml",
        "drift-severe",
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
    let severe_eval =
        sprint64_support::write_support_json("drift-severe", "evaluation_a_v2.json", &eval);
    config.external_model_evaluation_report_paths[1] = severe_eval;

    let report = ExternalArtifactRegistryRunner::default()
        .run_calibration_drift(&config)
        .expect("run severe drift");
    assert_eq!(report.drift_status, CalibrationDriftStatus::SevereDrift);
    assert_eq!(report.severe_drift_count, 1);
}
