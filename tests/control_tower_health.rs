mod common;

use soma_zero::{
    ControlTowerHealthStatus, ControlTowerRefreshPlanner, ControlTowerV1Builder,
    ControlTowerV1Config, summarize_control_tower_health,
};

#[test]
fn control_tower_health_reports_need_evidence_depth_and_owner_review() {
    let mut config = ControlTowerV1Config::from_toml_path(&common::example_path(
        "soma_control_tower_v1_kis.toml",
    ))
    .expect("config");
    config.output_root = common::sprint54_output_dir("health-summary")
        .display()
        .to_string();
    let state = ControlTowerV1Builder::default()
        .build(
            &config,
            Some(&common::example_path("soma_control_tower_v1_kis.toml")),
        )
        .expect("state");
    assert_eq!(
        state.health_summary.health_status,
        ControlTowerHealthStatus::NeedEvidenceDepth
    );

    let mut improved = state.clone();
    improved.kis_monitor_panel.outcome_links = 20;
    improved.kis_monitor_panel.complete_rows = 20;
    improved.evidence_panel.sufficiency_status = "Healthy".to_string();
    let summary = summarize_control_tower_health(
        &improved.kis_monitor_panel,
        &improved.evidence_panel,
        &improved.owner_panel,
        &improved.risk_panel,
        &improved.candidate_panel,
        &improved.next_action_panel,
        &ControlTowerRefreshPlanner::default(),
        &[],
        &[],
        true,
    );
    assert_eq!(
        summary.health_status,
        ControlTowerHealthStatus::NeedOwnerReview
    );
}

#[test]
fn control_tower_health_detects_secret_failure_deterministically() {
    let mut config = ControlTowerV1Config::from_toml_path(&common::example_path(
        "soma_control_tower_v1_kis.toml",
    ))
    .expect("config");
    config.output_root = common::sprint54_output_dir("health-secret")
        .display()
        .to_string();
    let state = ControlTowerV1Builder::default()
        .build(
            &config,
            Some(&common::example_path("soma_control_tower_v1_kis.toml")),
        )
        .expect("state");
    let summary = summarize_control_tower_health(
        &state.kis_monitor_panel,
        &state.evidence_panel,
        &state.owner_panel,
        &state.risk_panel,
        &state.candidate_panel,
        &state.next_action_panel,
        &ControlTowerRefreshPlanner::default(),
        &["KIS_APP_KEY=secret".to_string()],
        &[],
        false,
    );
    assert_eq!(
        summary.health_status,
        ControlTowerHealthStatus::SecretRedactionFailed
    );
    let again = summarize_control_tower_health(
        &state.kis_monitor_panel,
        &state.evidence_panel,
        &state.owner_panel,
        &state.risk_panel,
        &state.candidate_panel,
        &state.next_action_panel,
        &ControlTowerRefreshPlanner::default(),
        &["KIS_APP_KEY=secret".to_string()],
        &[],
        false,
    );
    assert_eq!(summary, again);
}
