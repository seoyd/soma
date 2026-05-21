mod common;

use soma_zero::{
    ControlTowerRefreshPlanner, ControlTowerV1Builder, ControlTowerV1Config, NextActionKind,
    build_next_action_panel,
};

#[test]
fn next_action_panel_prefers_kis_and_owner_review_work() {
    let mut config = ControlTowerV1Config::from_toml_path(&common::example_path(
        "soma_control_tower_v1_kis.toml",
    ))
    .expect("config");
    config.output_root = common::sprint54_output_dir("next-actions")
        .display()
        .to_string();
    let state = ControlTowerV1Builder::default()
        .build(
            &config,
            Some(&common::example_path("soma_control_tower_v1_kis.toml")),
        )
        .expect("state");
    let panel = state.next_action_panel.clone();
    assert!(matches!(
        panel.primary_next_action.action_kind,
        NextActionKind::RunKISCandleSufficiency
            | NextActionKind::RunKISOutcomeLinkClose
            | NextActionKind::RunOwnerReviewQueue
    ));
    assert!(
        panel
            .recommended_actions
            .iter()
            .any(|item| matches!(item.action_kind, NextActionKind::RunOwnerReviewQueue))
    );
    assert!(
        panel
            .recommended_actions
            .iter()
            .any(|item| matches!(item.action_kind, NextActionKind::ReviewRiskBlockedCandidate))
    );
    assert!(panel.recommended_actions.iter().all(|item| {
        !item
            .command_suggestion
            .clone()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains("order")
    }));
}

#[test]
fn next_action_panel_selects_kis_market_data_activate_when_auth_missing() {
    let mut config = ControlTowerV1Config::from_toml_path(&common::example_path(
        "soma_control_tower_v1_kis.toml",
    ))
    .expect("config");
    config.output_root = common::sprint54_output_dir("next-actions-auth-missing")
        .display()
        .to_string();
    let mut state = ControlTowerV1Builder::default()
        .build(
            &config,
            Some(&common::example_path("soma_control_tower_v1_kis.toml")),
        )
        .expect("state");
    state.kis_monitor_panel.auth_ready = false;
    state.kis_monitor_panel.base_url_ready = false;
    let panel = build_next_action_panel(
        Some("examples/soma_control_tower_v1_kis.toml"),
        &config.output_root,
        &state.kis_monitor_panel,
        &state.owner_panel,
        &state.risk_panel,
        &state.candidate_panel,
        &ControlTowerRefreshPlanner::default(),
    );
    assert!(
        panel
            .blocked_actions
            .iter()
            .any(|item| matches!(item.action_kind, NextActionKind::RunKISMarketDataActivate))
    );
}
