#[path = "support/sprint69_support.rs"]
mod support;

use std::fs;

use soma_zero::{
    PredictionRequirementKind, RealEvidencePredictionRefreshRunner,
    RealEvidencePredictionRequirementStatus,
};

#[test]
fn prediction_requirements_match_expected_counts() {
    let config =
        support::sprint75_config_from_example("soma_real_prediction_requirements.toml", "reqs");
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_prediction_requirements(&config)
        .expect("requirements");
    assert_eq!(
        report.requirement_status,
        RealEvidencePredictionRequirementStatus::PredictionsRequired
    );
    assert_eq!(report.required_count, 1);
    assert_eq!(report.recommended_count, 3);
    assert!(
        report
            .items
            .iter()
            .any(|item| item.requirement_kind == PredictionRequirementKind::PredictionStale)
    );
}

#[test]
fn no_new_predictions_required_is_reported_when_state_is_clean() {
    let mut config = support::sprint75_config_from_example(
        "soma_real_prediction_requirements.toml",
        "reqs-no-action",
    );
    let dir = support::sprint75_output_dir("reqs-no-action-inputs");
    let followup = dir.join("followup.json");
    let modelops = dir.join("modelops.json");
    fs::write(
        &followup,
        r#"{"real_evidence_ready":true,"model_predictions_stale_present":false,"remaining_warnings":["DirectWatchMonitoringOnly","RuntimeMambaDeferred","LiveTradingForbidden","BrokerForbidden"]}"#,
    )
    .expect("write followup");
    fs::write(
        &modelops,
        r#"{"evidence_rows_added":0,"sequence_windows_added":0,"existing_prediction_coverage_affected":false,"models_requiring_new_predictions":[],"models_requiring_reevaluation":[]}"#,
    )
    .expect("write modelops");
    config.real_evidence_followup_paths = vec![followup.display().to_string()];
    config.control_tower_real_evidence_refresh_paths = vec![followup.display().to_string()];
    config.real_modelops_impact_paths = vec![modelops.display().to_string()];
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_prediction_requirements(&config)
        .expect("requirements");
    assert_eq!(
        report.requirement_status,
        RealEvidencePredictionRequirementStatus::NoPredictionsRequired
    );
    assert_eq!(report.no_action_count, 1);
    assert_eq!(report.optional_count, 1);
    assert_eq!(
        report.items[0].requirement_kind,
        PredictionRequirementKind::NoNewPredictionsRequired
    );
}
