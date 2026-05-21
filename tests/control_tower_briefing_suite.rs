#[path = "support/sprint69_support.rs"]
mod support;

use std::fs;

use soma_zero::{ControlTowerBriefingFinalRefreshStatus, ControlTowerBriefingRefreshStatusV2};

#[test]
fn control_tower_briefing_panel_is_static_and_links_fragments() {
    let bundle = support::run_briefing(
        "soma_control_tower_briefing.toml",
        "control-tower-briefing-suite",
    );
    let panel = bundle.control_tower_briefing_panel;
    assert!(!panel.section_summaries.is_empty());
    assert!(!panel.owner_attention_summary.is_empty());
    assert!(!panel.model_attention_summary.is_empty());
    assert!(!panel.deferred_summary.is_empty());
    assert!(!panel.next_command_blocks.is_empty());
    assert!(!panel.fragment_links.is_empty());

    let html = fs::read_to_string(
        support::briefing_output_path("control-tower-briefing-suite")
            .join("sprint71-control-tower-briefing")
            .join("control_tower_briefing.html"),
    )
    .expect("briefing html");
    assert!(!html.contains("<button"));
    assert!(!html.contains("<form"));
    assert!(!html.contains("ExecuteTrade"));
    assert!(!html.contains("PromoteLive"));
}

#[test]
fn static_briefing_renderer_writes_safe_files_without_secrets_or_scripts() {
    let _bundle = support::run_briefing(
        "soma_operator_briefing.toml",
        "control-tower-static-briefing-suite",
    );
    let root = support::briefing_output_path("control-tower-static-briefing-suite")
        .join("sprint71-operator-briefing");
    for relative in [
        "control_tower_briefing.html",
        "briefing_state.json",
        "briefing_summary.txt",
        "fragments/owner_review_briefing.html",
        "fragments/next_actions.html",
        "fragments/model_attention.html",
        "fragments/deferred_items.html",
    ] {
        assert!(
            root.join(relative).exists(),
            "missing rendered file: {relative}"
        );
    }
    let html = fs::read_to_string(root.join("control_tower_briefing.html")).expect("html");
    for forbidden in [
        "<script",
        "<form",
        "action=",
        "http://",
        "https://",
        "KIS_APP_KEY",
        "KIS_APP_SECRET",
    ] {
        assert!(!html.contains(forbidden));
    }
}

#[test]
fn briefing_refresh_states_stay_conservative_and_read_only() {
    let bundle = support::run_sprint73_bundle(
        "soma_control_tower_final_refresh.toml",
        "control-tower-briefing-final-refresh-suite",
    );
    let report = bundle.control_tower_briefing_final_refresh;
    assert_eq!(report.briefing_state_before, "StillNeedsEvidence");
    assert_eq!(report.briefing_state_after, "BriefingReadyWithWarnings");
    assert_eq!(
        report.refresh_status,
        ControlTowerBriefingFinalRefreshStatus::BriefingReadyWithWarnings
    );

    let html = fs::read_to_string(
        support::sprint73_output_path("control-tower-briefing-final-refresh-suite")
            .join("sprint73-ext-model-b-prediction-gap-closure")
            .join("control_tower_briefing.html"),
    )
    .expect("read html");
    assert!(!html.contains("<form"));
    assert!(!html.contains("button"));
    assert!(!html.contains("order"));
    assert!(!html.contains("account"));

    let bundle = support::run_offline_attachment(
        "soma_offline_evidence_attach.toml",
        "control-tower-briefing-refresh-v2-suite",
    );
    let refresh = bundle.control_tower_briefing_refresh_v2;
    assert_eq!(
        refresh.refresh_status,
        ControlTowerBriefingRefreshStatusV2::StillNeedsEvidence
    );
    assert_eq!(refresh.briefing_state_after, "StillNeedsEvidence");
    assert_eq!(refresh.direct_watch_score, "86~90 / NeedsEvidence");
}

#[test]
fn operator_briefing_outputs_are_deterministic() {
    let first = support::run_briefing(
        "soma_operator_briefing.toml",
        "control-tower-briefing-determinism-a",
    );
    let second = support::run_briefing(
        "soma_operator_briefing.toml",
        "control-tower-briefing-determinism-b",
    );
    assert_eq!(
        first.operator_briefing_report,
        second.operator_briefing_report
    );
    assert_eq!(first.owner_action_checklist, second.owner_action_checklist);
    assert_eq!(
        first.operator_decision_queue,
        second.operator_decision_queue
    );
    assert_eq!(
        first.leaderboard_warning_closure_report,
        second.leaderboard_warning_closure_report
    );
    let first_html = fs::read_to_string(
        support::briefing_output_path("control-tower-briefing-determinism-a")
            .join("sprint71-operator-briefing")
            .join("control_tower_briefing.html"),
    )
    .expect("first html");
    let second_html = fs::read_to_string(
        support::briefing_output_path("control-tower-briefing-determinism-b")
            .join("sprint71-operator-briefing")
            .join("control_tower_briefing.html"),
    )
    .expect("second html");
    assert_eq!(first_html, second_html);
}
