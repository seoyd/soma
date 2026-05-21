mod common;

use std::fs;
use std::path::Path;

use soma_zero::{
    ControlTowerV1Builder, ControlTowerV1Config, DashboardSystemMode, DashboardV1Renderer,
    ReasonCode, TrinityCommitteeOperationalLoopConfig, TrinityOperationalLoopRunner,
    generate_owner_action_draft_bundle,
};

#[test]
fn control_tower_v1_config_defaults_and_remote_rejection_are_safe() {
    let config = ControlTowerV1Config::default();
    assert!(config.redact_secrets);
    assert!(!config.enable_dashboard_open);
    assert!(!config.enable_dashboard_serve);
    assert!(config.generate_owner_action_drafts);
    assert!(config.validate().is_ok());

    let remote = r#"
control_tower_id = "remote"
output_root = "https://example.com/out"
"#;
    let config = ControlTowerV1Config::from_toml_str(remote).expect("parse");
    assert_eq!(
        config.validate_local_paths(),
        vec![
            ReasonCode::LocalPathRejected,
            ReasonCode::RemotePathRejected
        ]
    );
}

#[test]
fn control_tower_v1_loads_operational_loop_panels_and_stays_paper_only() {
    let loop_config = TrinityCommitteeOperationalLoopConfig::from_toml_path(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/soma_trinity_operational_loop_kis.toml"
    )))
    .expect("config");
    let bundle = TrinityOperationalLoopRunner::default()
        .run(&loop_config)
        .expect("run loop");
    let report_path = loop_config
        .artifact_dir()
        .join("trinity_operational_loop_report.json");
    let config = ControlTowerV1Config {
        control_tower_id: "sprint87-control-tower".to_string(),
        operational_loop_report_paths: vec![report_path.display().to_string()],
        output_root: "target/test-sprint87-control-tower".to_string(),
        render_html: true,
        render_json: true,
        render_text: true,
        ..ControlTowerV1Config::default()
    };
    let state = ControlTowerV1Builder::default()
        .build(&config, None)
        .expect("state");
    assert!(matches!(
        state.system_mode,
        DashboardSystemMode::Paper | DashboardSystemMode::Research
    ));
    assert_eq!(
        state.operational_loop_panel.generated_candidates,
        bundle.report.generated_candidate_count
    );
    assert_eq!(state.trinity_status_panel.active_count, 3);
    assert!(!state.candidate_lifecycle_panel.candidate_views.is_empty());
}

#[test]
fn control_tower_v1_render_and_drafts_are_deterministic_and_safe() {
    let mut config = ControlTowerV1Config::from_toml_path(&common::example_path(
        "soma_control_tower_v1_kis.toml",
    ))
    .expect("config");
    config.output_root = common::sprint54_output_dir("control-tower-v1-suite")
        .display()
        .to_string();
    let state_first = ControlTowerV1Builder::default()
        .build(
            &config,
            Some(&common::example_path("soma_control_tower_v1_kis.toml")),
        )
        .expect("state");
    let state_second = ControlTowerV1Builder::default()
        .build(
            &config,
            Some(&common::example_path("soma_control_tower_v1_kis.toml")),
        )
        .expect("state");
    assert_eq!(
        state_first.to_json_string().expect("json"),
        state_second.to_json_string().expect("json")
    );

    let report_first = DashboardV1Renderer::default()
        .render(&state_first, &config)
        .expect("render");
    let report_second = DashboardV1Renderer::default()
        .render(&state_second, &config)
        .expect("render");
    let html_first = fs::read_to_string(report_first.html_path.expect("html 1")).expect("html 1");
    let html_second = fs::read_to_string(report_second.html_path.expect("html 2")).expect("html 2");
    assert_eq!(html_first, html_second);
    assert!(!html_first.contains("<form"));
    assert!(!html_first.contains("<button"));
    assert!(!html_first.contains("ExecuteOrder"));
    assert!(!html_first.contains("account-balance"));

    let drafts_first = generate_owner_action_draft_bundle(&state_first, &config).expect("drafts");
    let drafts_second = generate_owner_action_draft_bundle(&state_second, &config).expect("drafts");
    assert_eq!(drafts_first.drafts, drafts_second.drafts);
}
