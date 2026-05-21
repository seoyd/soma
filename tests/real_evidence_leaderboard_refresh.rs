#[path = "support/sprint69_support.rs"]
mod support;

use std::fs;

use soma_zero::{RealEvidenceLeaderboardRefreshStatus, RealEvidencePredictionRefreshRunner};

#[test]
fn leaderboard_refresh_detects_affected_model() {
    let config = support::sprint75_config_from_example(
        "soma_real_leaderboard_refresh.toml",
        "leaderboard-refresh",
    );
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_leaderboard_refresh(&config)
        .expect("leaderboard refresh");
    assert_eq!(
        report.leaderboard_status,
        RealEvidenceLeaderboardRefreshStatus::LeaderboardRefreshed
    );
    assert_eq!(
        report.affected_models,
        vec!["ext-model-b:1.0.0".to_string()]
    );
}

#[test]
fn leaderboard_can_report_no_change() {
    let mut config = support::sprint75_config_from_example(
        "soma_real_leaderboard_refresh.toml",
        "leaderboard-no-change",
    );
    let dir = support::sprint75_output_dir("leaderboard-no-change-inputs");
    let modelops = dir.join("modelops.json");
    let followup = dir.join("followup.json");
    fs::write(
        &modelops,
        r#"{"evidence_rows_added":0,"sequence_windows_added":0,"existing_prediction_coverage_affected":false,"models_requiring_new_predictions":[],"models_requiring_reevaluation":[]}"#,
    )
    .expect("write modelops");
    fs::write(
        &followup,
        r#"{"real_evidence_ready":true,"model_predictions_stale_present":false,"remaining_warnings":["DirectWatchMonitoringOnly","RuntimeMambaDeferred","LiveTradingForbidden","BrokerForbidden"]}"#,
    )
    .expect("write followup");
    config.real_modelops_impact_paths = vec![modelops.display().to_string()];
    config.real_evidence_followup_paths = vec![followup.display().to_string()];
    config.control_tower_real_evidence_refresh_paths = vec![followup.display().to_string()];
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_leaderboard_refresh(&config)
        .expect("leaderboard refresh");
    assert_eq!(
        report.leaderboard_status,
        RealEvidenceLeaderboardRefreshStatus::NoLeaderboardChange
    );
}
