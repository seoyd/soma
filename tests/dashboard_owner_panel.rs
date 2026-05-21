mod common;

use std::fs;

use soma_zero::{
    DashboardRenderConfig, DashboardRenderer, DashboardSnapshotBuilder, DashboardSourceConfig,
};

#[test]
fn dashboard_owner_panel_candidate_human_confirm_and_audit_are_enriched() {
    let mut config = DashboardSourceConfig::from_toml_path(&common::example_path(
        "soma_dashboard_source_with_owner_panel.toml",
    ))
    .expect("config");
    config.output_root = common::sprint53_output_dir("dashboard-owner-panel")
        .display()
        .to_string();
    let state = DashboardSnapshotBuilder::default()
        .build(&config)
        .expect("state");
    assert!(!state.owner_panel.pending_review_items.is_empty());
    assert!(!state.owner_panel.blocked_owner_inputs.is_empty());
    assert!(!state.owner_panel.paper_confirmed_items.is_empty());
    assert!(!state.owner_panel.active_thesis_notes.is_empty());
    assert!(
        state
            .candidate_panel
            .candidates
            .iter()
            .any(|candidate| !candidate.owner_feedback_history.is_empty())
    );
    assert!(
        state
            .human_confirm_panel
            .pending_items
            .iter()
            .any(|item| !item.allowed_owner_actions.is_empty()
                && !item.paper_confirm_explanation.is_empty())
    );
    assert!(state.audit_timeline.events.iter().any(|event| {
        matches!(
            event.kind,
            soma_zero::DashboardEventKind::OwnerInputSubmitted
                | soma_zero::DashboardEventKind::OwnerInputApplied
                | soma_zero::DashboardEventKind::OwnerInputBlocked
                | soma_zero::DashboardEventKind::HumanConfirmTransition
        )
    }));

    let mut render = DashboardRenderConfig::from_toml_path(&common::example_path(
        "soma_dashboard_render_static.toml",
    ))
    .expect("render config");
    render.source_config_path = Some(
        common::example_path("soma_dashboard_source_with_owner_panel.toml")
            .display()
            .to_string(),
    );
    render.dashboard_state_path = None;
    render.output_root = common::sprint53_output_dir("dashboard-owner-render")
        .display()
        .to_string();
    let report = DashboardRenderer::default()
        .render(&render)
        .expect("render");
    let html = fs::read_to_string(report.html_path.expect("html path")).expect("html");
    assert!(html.contains("Owner"));
    assert!(!html.contains("<button"));
    assert!(!html.contains("top-secret"));

    let second = DashboardSnapshotBuilder::default()
        .build(&config)
        .expect("state");
    assert_eq!(state.fingerprint, second.fingerprint);
}
