mod common;
#[path = "support/sprint64_support.rs"]
mod sprint64_support;

use std::fs;

use soma_zero::{
    ExternalArtifactRegistryRunner, ExternalModelEvaluationReport,
    ExternalModelVersionComparisonStatus,
};

#[test]
fn mixed_and_no_previous_statuses_work() {
    let config = sprint64_support::registry_config_from_example(
        "soma_external_model_version_comparison.toml",
        "version-comparison-default",
    );
    let report = ExternalArtifactRegistryRunner::default()
        .run_version_comparison(&config)
        .expect("run version comparison");
    assert_eq!(report.model_id, "ext-model-a");
    assert_eq!(
        report.comparison_status,
        ExternalModelVersionComparisonStatus::Mixed
    );

    let mut no_previous = sprint64_support::registry_config_from_example(
        "soma_external_model_version_comparison.toml",
        "version-comparison-none",
    );
    no_previous.external_prediction_import_report_paths = vec![
        no_previous
            .external_prediction_import_report_paths
            .last()
            .expect("b import")
            .clone(),
    ];
    no_previous.external_model_card_paths = vec![
        no_previous
            .external_model_card_paths
            .last()
            .expect("b card")
            .clone(),
    ];
    no_previous.external_model_evaluation_report_paths = vec![
        no_previous
            .external_model_evaluation_report_paths
            .last()
            .expect("b eval")
            .clone(),
    ];
    no_previous
        .external_prediction_ablation_report_paths
        .clear();
    no_previous.external_model_promotion_gate_paths.clear();
    let no_previous_report = ExternalArtifactRegistryRunner::default()
        .run_version_comparison(&no_previous)
        .expect("run no previous comparison");
    assert_eq!(
        no_previous_report.comparison_status,
        ExternalModelVersionComparisonStatus::NoComparablePrevious
    );
}

#[test]
fn regression_is_detected_when_risk_and_calibration_worsen() {
    let mut config = sprint64_support::registry_config_from_example(
        "soma_external_model_version_comparison.toml",
        "version-comparison-regressed",
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
    eval.calibration_metrics.ece = Some(0.41);
    eval.risk_metrics.risk_adjusted_score = Some(0.8);
    let regressed_eval = sprint64_support::write_support_json(
        "version-comparison-regressed",
        "evaluation_a_v2.json",
        &eval,
    );
    config.external_model_evaluation_report_paths[1] = regressed_eval;

    let report = ExternalArtifactRegistryRunner::default()
        .run_version_comparison(&config)
        .expect("run regressed comparison");
    assert_eq!(
        report.comparison_status,
        ExternalModelVersionComparisonStatus::Regressed
    );
}
