mod common;
#[path = "support/sprint66_support.rs"]
mod sprint66_support;

use soma_zero::{ModelReviewClosureRunner, model::ControlTowerModelOpsRefreshStatus};

#[test]
fn control_tower_refresh_summarizes_closure_history_qa_and_regression() {
    let config = sprint66_support::closure_config_from_example(
        "soma_control_tower_model_ops_refresh.toml",
        "control-tower-model-ops-refresh",
    );
    let report = ModelReviewClosureRunner::default()
        .run_control_tower_refresh(&config)
        .expect("run control tower refresh");
    assert_eq!(
        report.refresh_status,
        ControlTowerModelOpsRefreshStatus::ModelOpsRefreshedWithWarnings
    );
    assert_eq!(report.review_closure_status, "NeedsMorePredictions");
    assert_eq!(
        report.prediction_history_status,
        "PredictionHistoryPackReady"
    );
    assert_eq!(report.operator_qa_status, "NeedsMorePredictions");
    assert_eq!(
        report.regression_guard_status.as_deref(),
        Some("RegressionDetected")
    );
    let flattened = serde_json::to_string(&report)
        .expect("encode refresh report")
        .to_lowercase();
    for forbidden in ["live", "train", "broker", "order", "account"] {
        assert!(!flattened.contains(&format!("{forbidden} control")));
    }
}

#[test]
fn control_tower_refresh_report_is_deterministic() {
    let config = sprint66_support::closure_config_from_example(
        "soma_control_tower_model_ops_refresh.toml",
        "control-tower-model-ops-refresh-deterministic",
    );
    let runner = ModelReviewClosureRunner::default();
    let first = runner
        .run_control_tower_refresh(&config)
        .expect("first refresh");
    let second = runner
        .run_control_tower_refresh(&config)
        .expect("second refresh");
    assert_eq!(first, second);
}
