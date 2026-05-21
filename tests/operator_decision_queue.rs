#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{OperatorDecisionOption, OperatorDecisionQueueItemStatus};

#[test]
fn operator_decision_queue_covers_expected_statuses_and_forbidden_options() {
    let bundle = support::run_briefing(
        "soma_operator_decision_queue.toml",
        "operator-decision-queue",
    );

    let statuses = bundle
        .operator_decision_queue
        .items
        .iter()
        .map(|item| item.status)
        .collect::<Vec<_>>();
    assert!(statuses.contains(&OperatorDecisionQueueItemStatus::Pending));
    assert!(statuses.contains(&OperatorDecisionQueueItemStatus::ReadyToReview));
    assert!(statuses.contains(&OperatorDecisionQueueItemStatus::Blocked));
    assert!(statuses.contains(&OperatorDecisionQueueItemStatus::Closed));

    for item in &bundle.operator_decision_queue.items {
        assert!(
            item.forbidden_options
                .contains(&OperatorDecisionOption::PromoteLive)
        );
        assert!(
            item.forbidden_options
                .contains(&OperatorDecisionOption::EnableRuntime)
        );
        assert!(
            item.forbidden_options
                .contains(&OperatorDecisionOption::TrainModel)
        );
        assert!(
            item.forbidden_options
                .contains(&OperatorDecisionOption::ExecuteTrade)
        );
    }

    let leaderboard = bundle
        .operator_decision_queue
        .items
        .iter()
        .find(|item| item.target_id == "ext-model-a:1.2.0")
        .expect("leaderboard queue item");
    assert_eq!(
        leaderboard.default_safe_option,
        OperatorDecisionOption::DeferReview
    );
}
