mod common;

use soma_zero::{
    DashboardRenderConfig, DashboardRenderer, DashboardSnapshotBuilder, DashboardSourceConfig,
    HumanConfirmForbiddenAction, HumanConfirmSafeAction,
};

#[test]
fn human_confirm_panel_remains_view_only() {
    let mut config = DashboardSourceConfig::from_toml_path(&common::example_path(
        "soma_dashboard_source_kis_control_tower.toml",
    ))
    .expect("config");
    config.output_root = common::sprint52_output_dir("dashboard-human-confirm")
        .display()
        .to_string();
    let state = DashboardSnapshotBuilder::default()
        .build(&config)
        .expect("build");
    let pending = &state.human_confirm_panel.pending_items[0];
    assert!(
        pending
            .safe_actions
            .contains(&HumanConfirmSafeAction::ViewOnly)
    );
    assert!(
        pending
            .forbidden_actions
            .contains(&HumanConfirmForbiddenAction::ExecuteOrder)
    );

    let mut render = DashboardRenderConfig::from_toml_path(&common::example_path(
        "soma_dashboard_render_static.toml",
    ))
    .expect("render config");
    render.output_root = common::sprint52_output_dir("dashboard-human-confirm-render")
        .display()
        .to_string();
    let report = DashboardRenderer::default()
        .render(&render)
        .expect("render");
    let html = std::fs::read_to_string(report.html_path.expect("html path")).expect("html");
    assert!(!html.contains("<button"));
    assert!(!html.contains("action="));
}
