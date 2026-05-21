mod common;

use std::fs;

use soma_zero::{
    ControlTowerV1Builder, ControlTowerV1Config, DashboardV1Renderer,
    generate_owner_action_draft_bundle,
};

#[test]
fn control_tower_v1_render_and_drafts_are_deterministic() {
    let mut config = ControlTowerV1Config::from_toml_path(&common::example_path(
        "soma_control_tower_v1_kis.toml",
    ))
    .expect("config");
    config.output_root = common::sprint54_output_dir("control-tower-determinism")
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

    let drafts_first = generate_owner_action_draft_bundle(&state_first, &config).expect("drafts");
    let drafts_second = generate_owner_action_draft_bundle(&state_second, &config).expect("drafts");
    assert_eq!(drafts_first.drafts, drafts_second.drafts);
}
