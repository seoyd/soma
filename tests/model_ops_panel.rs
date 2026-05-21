mod common;
#[path = "support/sprint65_support.rs"]
mod sprint65_support;

use soma_zero::ExternalModelResearchOpsRunner;

#[test]
fn panel_shows_core_model_ops_sections_and_mamba_runtime_deferred() {
    let config = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "model-ops-panel",
    );
    let bundle = ExternalModelResearchOpsRunner::default()
        .run(&config)
        .expect("run model ops panel");
    let panel = bundle
        .control_tower_model_ops_panel_summary
        .expect("panel summary");
    let text = panel.to_text();
    for expected in [
        "review_queue_summary=",
        "watchlist=",
        "comparability_status=",
        "model_risk_profiles=",
        "leaderboard_changes=",
        "RuntimeDeferred",
    ] {
        assert!(text.contains(expected), "missing panel content: {expected}");
    }
}

#[test]
fn panel_has_no_train_live_or_order_account_controls() {
    let config = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "model-ops-panel-safety",
    );
    let bundle = ExternalModelResearchOpsRunner::default()
        .run(&config)
        .expect("run model ops panel safety");
    let panel = bundle
        .control_tower_model_ops_panel_summary
        .expect("panel summary");
    let text = panel.to_text();
    for forbidden in ["train button", "live button", "order", "account"] {
        assert!(
            !text.to_ascii_lowercase().contains(forbidden),
            "unexpected panel control: {forbidden}"
        );
    }
}

#[test]
fn model_ops_panel_is_deterministic() {
    let first = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "model-ops-panel-determinism-first",
    );
    let second = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "model-ops-panel-determinism-second",
    );
    let first_panel = ExternalModelResearchOpsRunner::default()
        .run(&first)
        .expect("run first panel")
        .control_tower_model_ops_panel_summary
        .expect("first panel summary");
    let second_panel = ExternalModelResearchOpsRunner::default()
        .run(&second)
        .expect("run second panel")
        .control_tower_model_ops_panel_summary
        .expect("second panel summary");
    assert_eq!(first_panel, second_panel);
}
