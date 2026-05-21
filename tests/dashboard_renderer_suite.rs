mod common;
mod support;

use std::fs;

use soma_zero::{
    ControlTowerV1Builder, ControlTowerV1Config, DashboardRenderConfig, DashboardRenderer,
    DashboardSnapshotBuilder, DashboardSourceConfig, DashboardSystemMode, DashboardV1Renderer,
    ReasonCode,
};
use support::shared_fixture_harness as harness;

fn dashboard_source_config(output_name: &str) -> DashboardSourceConfig {
    let mut config = DashboardSourceConfig::from_toml_path(&common::example_path(
        "soma_dashboard_source_kis_control_tower.toml",
    ))
    .expect("config");
    config.output_root = common::sprint52_output_dir(output_name)
        .display()
        .to_string();
    config
}

fn build_state(output_name: &str) -> soma_zero::DashboardState {
    DashboardSnapshotBuilder::default()
        .build(&dashboard_source_config(output_name))
        .expect("build")
}

#[test]
fn dashboard_renderer_outputs_static_local_artifacts_without_action_controls() {
    let mut config = DashboardRenderConfig::from_toml_path(&common::example_path(
        "soma_dashboard_render_static.toml",
    ))
    .expect("config");
    config.output_root = common::sprint52_output_dir("dashboard-renderer-suite")
        .display()
        .to_string();
    let report = DashboardRenderer::default()
        .render(&config)
        .expect("render");
    let html = fs::read_to_string(report.html_path.expect("html path")).expect("html");
    assert!(report.json_path.is_some());
    assert!(report.text_path.is_some());
    assert!(!html.contains("cdn"));
    assert!(!html.contains("<form"));
    assert!(!html.contains("POST"));
    assert!(!html.contains("<button"));
    assert!(!html.contains("account-balance"));
}

#[test]
fn dashboard_v1_renderer_writes_safe_static_outputs() {
    let mut config = ControlTowerV1Config::from_toml_path(&common::example_path(
        "soma_control_tower_v1_kis.toml",
    ))
    .expect("config");
    config.output_root = common::sprint54_output_dir("dashboard-v1-renderer-suite")
        .display()
        .to_string();
    let state = ControlTowerV1Builder::default()
        .build(
            &config,
            Some(&common::example_path("soma_control_tower_v1_kis.toml")),
        )
        .expect("state");
    let report = DashboardV1Renderer::default()
        .render(&state, &config)
        .expect("render");
    let html = fs::read_to_string(report.html_path.expect("html")).expect("html");
    assert!(!html.contains("<form"));
    assert!(!html.contains("<button"));
    assert!(!html.contains("ExecuteOrder"));
    assert!(!html.contains("account-balance"));
    assert!(report.owner_action_draft_dir.is_some());
}

#[test]
fn dashboard_renderer_is_deterministic() {
    let mut config = DashboardRenderConfig::from_toml_path(&common::example_path(
        "soma_dashboard_render_static.toml",
    ))
    .expect("config");
    config.output_root = common::sprint52_output_dir("dashboard-renderer-suite-deterministic")
        .display()
        .to_string();
    let first = DashboardRenderer::default().render(&config).expect("first");
    let second = DashboardRenderer::default()
        .render(&config)
        .expect("second");
    let html_first = fs::read_to_string(first.html_path.expect("html first")).expect("html");
    let html_second = fs::read_to_string(second.html_path.expect("html second")).expect("html");
    harness::assert_deterministic_text(&html_first, &html_second);
}

#[test]
fn dashboard_snapshot_writes_artifacts_and_enforces_limits() {
    let mut config = dashboard_source_config("dashboard-snapshot-suite");
    config.max_events = 2;
    config.max_candidates = 3;
    let state = DashboardSnapshotBuilder::default()
        .build_and_write(&config)
        .expect("build and write");
    let artifact_dir = config.artifact_dir();
    assert!(artifact_dir.join("dashboard_state.json").exists());
    assert!(artifact_dir.join("dashboard_state.txt").exists());
    assert_eq!(state.audit_timeline.events.len(), 2);
    assert_eq!(state.candidate_panel.candidates.len(), 3);
    let json = fs::read_to_string(artifact_dir.join("dashboard_state.json")).expect("json file");
    assert!(json.contains("dashboard_id"));
}

#[test]
fn dashboard_config_defaults_and_remote_paths_are_rejected() {
    let config = DashboardSourceConfig::default();
    assert!(config.redact_secrets);
    let mut remote = config.clone();
    remote.provider_simplification_report_paths =
        vec!["https://example.com/report.json".to_string()];
    assert!(
        remote
            .validate_local_paths()
            .contains(&ReasonCode::RemotePathRejected)
    );
    let json = serde_json::to_string(&config).expect("json");
    assert!(!json.contains("broker"));
    assert!(!json.contains("order"));
    assert!(!json.contains("account"));
}

#[test]
fn missing_reports_are_reason_coded_not_fatal() {
    let config = DashboardSourceConfig {
        dashboard_id: "missing-dashboard".to_string(),
        provider_simplification_report_paths: vec![
            common::sprint52_output_dir("missing")
                .join("missing.json")
                .display()
                .to_string(),
        ],
        output_root: common::sprint52_output_dir("missing-dashboard")
            .display()
            .to_string(),
        ..DashboardSourceConfig::default()
    };
    let state = DashboardSnapshotBuilder::default()
        .build(&config)
        .expect("build state");
    assert!(state.reason_codes.contains(&ReasonCode::MissingFile));
    assert!(
        state
            .reason_codes
            .contains(&ReasonCode::DashboardReportMissing)
    );
}

#[test]
fn dashboard_state_defaults_to_research_and_has_no_generated_wall_clock() {
    let config = DashboardSourceConfig {
        dashboard_id: "research-dashboard".to_string(),
        provider_simplification_report_paths: vec![
            common::sprint52_data_path("provider_simplification_sample.json")
                .display()
                .to_string(),
        ],
        output_root: common::sprint52_output_dir("research-dashboard")
            .display()
            .to_string(),
        ..DashboardSourceConfig::default()
    };
    let first = DashboardSnapshotBuilder::default()
        .build(&config)
        .expect("build state");
    let second = DashboardSnapshotBuilder::default()
        .build(&config)
        .expect("build state");
    assert_eq!(first.system_mode, DashboardSystemMode::Research);
    assert!(
        first
            .audit_timeline
            .events
            .iter()
            .all(|event| event.timestamp_ms.is_none())
    );
    assert_eq!(first.fingerprint, second.fingerprint);
}

#[test]
fn dashboard_panels_render_expected_values() {
    let state = build_state("dashboard-panels-suite");
    assert_eq!(
        state
            .provider_panel
            .active_primary_provider_by_market
            .get("KoreanEquity")
            .map(String::as_str),
        Some("KIS")
    );
    assert!(state.provider_panel.krx_status.reference_enabled);
    assert_eq!(
        state.provider_panel.yfinance_status.provider_label,
        "yfinance"
    );
    assert!(state.provider_panel.kis_status.auth_ready);
    let provider_json = serde_json::to_string(&state.provider_panel).expect("json");
    assert!(!provider_json.contains("top-secret"));
    assert_eq!(state.evidence_panel.official_rows, 42);
    assert_eq!(state.evidence_panel.outcome_links, 12);
    assert_eq!(state.committee_panel.active_personas, 3);
    assert!(
        state
            .committee_panel
            .member_views
            .iter()
            .all(|member| member.conviction.unwrap_or_default() <= 1.0)
    );
    assert!(
        state
            .committee_panel
            .member_views
            .iter()
            .all(|member| member.voice_power.unwrap_or_default() <= 1.0)
    );
    assert!(
        !state
            .committee_panel
            .recommendation
            .contains("language model")
    );
    assert_eq!(state.chair_panel.final_decision, "RequireConfirm");
    assert!(state.chair_panel.human_confirm_required);
    assert!(state.risk_panel.default_deny_active);
    assert_eq!(state.risk_panel.last_risk_decision.as_deref(), Some("Deny"));
}
