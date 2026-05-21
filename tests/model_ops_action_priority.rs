mod common;
#[path = "support/sprint67_support.rs"]
mod sprint67_support;

use soma_zero::{ModelOpsDecisionKind, ModelOpsRollupRunner};

#[test]
fn action_priority_sorts_required_actions_first_and_keeps_primary_prediction_request() {
    let config = sprint67_support::rollup_config_from_example(
        "soma_model_action_priority.toml",
        "action-priority",
    );
    let report = ModelOpsRollupRunner::default()
        .run_action_priority(&config)
        .expect("run action priority");
    assert!(!report.required_actions.is_empty());
    assert_eq!(
        report.primary_next_action.as_ref().map(|item| item.action),
        Some(ModelOpsDecisionKind::RequestMorePredictions)
    );
    assert!(report.required_actions[0].safe_to_run);
    assert!(
        !report.required_actions[0]
            .command_suggestion
            .as_ref()
            .unwrap()
            .contains("order")
    );
}

#[test]
fn action_priority_can_elevate_retire_for_critical_regression() {
    let mut config = sprint67_support::rollup_config_from_example(
        "soma_model_action_priority.toml",
        "action-priority-retire",
    );
    let mut closure: soma_zero::ModelReviewClosureReport =
        sprint67_support::read_json(&config.model_review_closure_paths[0]);
    for action in &mut closure.actions {
        if action.model_id == "ext-model-b" {
            action.action = soma_zero::ModelReviewClosureDecision::RetireModelVersion;
        }
    }
    config.model_review_closure_paths[0] =
        sprint67_support::write_support_json("action-priority-retire", "closure.json", &closure);
    let report = ModelOpsRollupRunner::default()
        .run_action_priority(&config)
        .expect("run retire priority");
    assert!(
        report
            .required_actions
            .iter()
            .any(|item| item.model_id == "ext-model-b"
                && item.action == ModelOpsDecisionKind::RetireModelVersion)
    );
}
