mod common;
#[path = "support/sprint67_support.rs"]
mod sprint67_support;

use soma_zero::{ModelOpsRollupRunner, OperatorQARollupStatus, OwnerModelReviewAction};

#[test]
fn operator_qa_rollup_collapses_duplicates_and_keeps_request_more_predictions_once() {
    let config =
        sprint67_support::rollup_config_from_example("soma_operator_qa_rollup.toml", "qa-rollup");
    let report = ModelOpsRollupRunner::default()
        .run_operator_qa_rollup(&config)
        .expect("run operator qa rollup");
    assert_eq!(
        report
            .items
            .iter()
            .filter(|item| item.model_id == "ext-model-b" && item.model_version == "1.0.0")
            .count(),
        1
    );
    let item = report
        .items
        .iter()
        .find(|item| item.model_id == "ext-model-b" && item.model_version == "1.0.0")
        .expect("ext-model-b qa rollup");
    assert_eq!(item.status, OperatorQARollupStatus::NeedsMorePredictions);
}

#[test]
fn operator_qa_rollup_preserves_blocked_actions_and_next_commands_are_research_only() {
    let mut config = sprint67_support::rollup_config_from_example(
        "soma_operator_qa_rollup.toml",
        "qa-rollup-blocked",
    );
    let mut actions: Vec<OwnerModelReviewAction> =
        sprint67_support::read_json(&config.owner_model_review_paths[0]);
    actions[0].allowed = false;
    actions[0].note = Some("runtime live blocked".to_string());
    config.owner_model_review_paths[0] =
        sprint67_support::write_support_json("qa-rollup-blocked", "owner_actions.json", &actions);
    let report = ModelOpsRollupRunner::default()
        .run_operator_qa_rollup(&config)
        .expect("run blocked operator qa rollup");
    let blocked = report
        .items
        .iter()
        .find(|item| !item.blocked_actions.is_empty())
        .expect("blocked qa item");
    assert!(!blocked.blocked_actions.is_empty());
    assert!(
        blocked
            .next_command
            .as_ref()
            .is_some_and(|command| command.contains("cargo run --quiet --bin soma_experiment"))
    );
}
