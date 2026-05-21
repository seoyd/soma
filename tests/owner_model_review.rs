mod common;
#[path = "support/sprint65_support.rs"]
mod sprint65_support;

use soma_zero::{ExternalModelResearchOpsRunner, OwnerModelReviewActionKind};

#[test]
fn safe_owner_review_actions_are_accepted() {
    let config = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "owner-review-actions",
    );
    let bundle = ExternalModelResearchOpsRunner::default()
        .run(&config)
        .expect("run owner review workflow");
    let impact = bundle
        .owner_model_review_impact_report
        .expect("owner impact report");
    assert!(
        impact
            .actions
            .iter()
            .any(|action| action.action_kind == OwnerModelReviewActionKind::WatchModel)
    );
    assert!(
        impact
            .actions
            .iter()
            .any(|action| action.action_kind == OwnerModelReviewActionKind::UnwatchModel)
    );
    assert!(
        impact
            .actions
            .iter()
            .any(|action| action.action_kind == OwnerModelReviewActionKind::AddModelNote)
    );
    assert!(impact.actions.iter().any(|action| {
        action.action_kind == OwnerModelReviewActionKind::RequestMorePredictions
    }));
    assert!(impact.actions.iter().any(|action| {
        action.action_kind == OwnerModelReviewActionKind::RequestCalibrationReview
    }));
    assert!(
        impact
            .actions
            .iter()
            .any(|action| { action.action_kind == OwnerModelReviewActionKind::RequestRiskReview })
    );
    assert!(impact.actions.iter().any(|action| {
        action.action_kind == OwnerModelReviewActionKind::MarkModelDiagnosticOnly
    }));
    assert!(
        impact
            .actions
            .iter()
            .any(|action| { action.action_kind == OwnerModelReviewActionKind::RetireModelVersion })
    );
    assert!(impact.accepted_count >= 6);
}

#[test]
fn owner_review_cannot_enable_live_runtime_or_override_gates() {
    let config = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "owner-review-safety",
    );
    let bundle = ExternalModelResearchOpsRunner::default()
        .run(&config)
        .expect("run owner review safety");
    let impact = bundle
        .owner_model_review_impact_report
        .expect("owner impact report");
    assert!(impact.blocked_count >= 1);
    let encoded = serde_json::to_string(&impact).expect("serialize impact report");
    for forbidden in [
        "PromoteToLive",
        "EnableRuntimeInference",
        "OverridePromotionGate",
    ] {
        assert!(!encoded.contains(forbidden));
    }
}

#[test]
fn owner_review_impact_report_is_deterministic() {
    let first = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "owner-review-determinism-first",
    );
    let second = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "owner-review-determinism-second",
    );
    let first_report = ExternalModelResearchOpsRunner::default()
        .run(&first)
        .expect("run first owner review")
        .owner_model_review_impact_report
        .expect("first owner impact");
    let second_report = ExternalModelResearchOpsRunner::default()
        .run(&second)
        .expect("run second owner review")
        .owner_model_review_impact_report
        .expect("second owner impact");
    assert_eq!(first_report, second_report);
}
