#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{
    DirectWatchPostEvidenceGateStatus, ModelPredictionsStaleClosureStatus,
    RealEvidencePredictionRefreshRunner,
};

#[test]
fn control_tower_panel_shows_refresh_and_remaining_warnings() {
    let config =
        support::sprint75_config_from_example("soma_real_modelops_refresh.toml", "panel-refresh");
    let panel = RealEvidencePredictionRefreshRunner::default()
        .run_control_tower_model_ops_panel(&config)
        .expect("panel");
    assert_eq!(panel.affected_models, vec!["ext-model-b:1.0.0".to_string()]);
    assert_eq!(
        panel.stale_closure_status,
        ModelPredictionsStaleClosureStatus::StaleClosed
    );
    assert_eq!(
        panel.direct_watch_post_evidence_status,
        DirectWatchPostEvidenceGateStatus::DirectWatchReadyWithWarnings
    );
    assert!(
        panel
            .remaining_warnings
            .contains(&"RuntimeMambaDeferred".to_string())
    );
    assert_eq!(panel.mamba_deferred_status, "RuntimeMambaDeferred");
    let text = serde_json::to_string(&panel).expect("serialize");
    assert!(!text.contains("train button"));
    assert!(!text.contains("live button"));
    assert!(!text.contains("order/account"));
}
