mod common;
#[path = "support/sprint67_support.rs"]
mod sprint67_support;

#[test]
fn control_tower_rollup_panel_shows_one_card_per_model_version_and_static_statuses() {
    let bundle = sprint67_support::run_rollup("soma_model_ops_rollup.toml", "rollup-panel");
    let panel = bundle
        .control_tower_model_ops_rollup_panel
        .expect("rollup panel");
    assert_eq!(panel.summary_cards.len(), 4);
    let flattened = panel.summary_cards.join(" ");
    assert!(flattened.contains("ext-model-b:1.0.0"));
    assert!(flattened.contains("prediction coverage regressed"));
    assert!(flattened.contains("history=PredictionHistoryReady"));
    assert!(flattened.contains("qa=NeedsMorePredictions"));
    assert!(flattened.contains("action=RequestMorePredictions"));
    assert_eq!(panel.mamba_family_status, "HoldMamba3RuntimeDeferred");
    let text = panel.to_text().to_lowercase();
    for forbidden in ["train button", "live button", "order/account"] {
        assert!(!text.contains(forbidden));
    }
}
