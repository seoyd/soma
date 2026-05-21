#[path = "support/sprint65_support.rs"]
mod sprint65_support;
#[path = "support/sprint66_support.rs"]
mod sprint66_support;

use soma_zero::{
    ExternalModelResearchOpsRunner, ExternalModelReviewItemKind, ExternalModelWatchStatus,
    ModelOpsOperatorQAStatus, ModelReviewClosureRunner, OwnerModelReviewAction,
    OwnerModelReviewActionKind, ReasonCode,
};

#[test]
fn operator_qa_contains_checklist_and_research_only_next_commands() {
    let config = sprint66_support::closure_config_from_example(
        "soma_model_ops_operator_qa.toml",
        "model-ops-qa-suite",
    );
    let report = ModelReviewClosureRunner::default()
        .run_operator_qa(&config)
        .expect("run operator qa");
    assert_eq!(
        report.qa_status,
        ModelOpsOperatorQAStatus::NeedsMorePredictions
    );
    for item in [
        "model card exists",
        "predictions cover known sequences",
        "calibration reviewed",
        "risk behavior reviewed",
        "NoTrade/RiskDenied reference preserved",
    ] {
        assert!(
            report
                .checklist_items
                .iter()
                .any(|existing| existing == item)
        );
    }
    assert!(
        report
            .next_commands
            .iter()
            .all(|command| command.contains("cargo run --quiet --bin soma_experiment"))
    );
}

#[test]
fn operator_qa_blocks_unsafe_actions_and_stays_diagnostic_only() {
    let mut config = sprint66_support::closure_config_from_example(
        "soma_model_ops_operator_qa.toml",
        "model-ops-qa-suite-blocked",
    );
    let actions = vec![OwnerModelReviewAction {
        action_id: "unsafe-live".to_string(),
        model_id: "ext-model-a".to_string(),
        model_version: "1.1.0".to_string(),
        action_kind: OwnerModelReviewActionKind::AddModelNote,
        note: Some("live order account override".to_string()),
        allowed: false,
        diagnostic_only: false,
        reason_codes: vec![ReasonCode::OwnerModelReviewWorkflowBuilt],
    }];
    config.owner_model_review_action_paths[0] = sprint66_support::write_support_json(
        "model-ops-qa-suite-blocked",
        "owner_actions.json",
        &actions,
    );
    let report = ModelReviewClosureRunner::default()
        .run_operator_qa(&config)
        .expect("run blocked operator qa");
    assert_eq!(report.qa_status, ModelOpsOperatorQAStatus::BlockedBySafety);
    assert!(!report.unsafe_actions_detected.is_empty());
}

#[test]
fn review_queue_contains_expected_item_kinds_and_forbidden_actions() {
    let config = sprint65_support::research_ops_config_from_example(
        "soma_external_model_research_ops.toml",
        "model-ops-qa-suite-review-queue",
    );
    let queue = ExternalModelResearchOpsRunner::default()
        .run_review_queue(&config)
        .expect("run review queue");
    assert!(queue.pending_items.iter().any(|item| {
        item.model_id == "ext-model-a"
            && item.model_version == "1.1.0"
            && item.item_kind == ExternalModelReviewItemKind::NewModelVersion
    }));
    assert!(queue.pending_items.iter().any(|item| {
        item.model_id == "ext-model-a"
            && item.item_kind == ExternalModelReviewItemKind::CalibrationDriftReview
    }));
    let sample = queue.pending_items.first().expect("pending item");
    for forbidden in ["PromoteToLive", "EnableRuntimeInference", "EnableTraining"] {
        assert!(sample.forbidden_actions.contains(&forbidden.to_string()));
    }
}

#[test]
fn watchlist_handles_active_removed_retired_and_diagnostic_entries() {
    let config = sprint65_support::research_ops_config_from_example(
        "soma_external_model_watchlist.toml",
        "model-ops-qa-suite-watchlist",
    );
    let watchlist = ExternalModelResearchOpsRunner::default()
        .run_watchlist(&config)
        .expect("run watchlist");
    assert!(watchlist.entries.iter().any(|entry| {
        entry.model_id == "ext-model-a" && entry.watch_status == ExternalModelWatchStatus::Active
    }));
    assert!(watchlist.entries.iter().any(|entry| {
        entry.model_id == "ext-model-b" && entry.watch_status == ExternalModelWatchStatus::Removed
    }));
    assert!(watchlist.entries.iter().any(|entry| {
        entry.model_id == "ext-model-a"
            && entry.model_version == "1.0.0"
            && entry.watch_status == ExternalModelWatchStatus::Retired
    }));

    let mut config = sprint65_support::research_ops_config_from_example(
        "soma_external_model_watchlist.toml",
        "model-ops-qa-suite-watchlist-diagnostic",
    );
    let actions = vec![OwnerModelReviewAction {
        action_id: "diagnostic-only".to_string(),
        model_id: "diag-model".to_string(),
        model_version: "0.9.0".to_string(),
        action_kind: OwnerModelReviewActionKind::MarkModelDiagnosticOnly,
        note: Some("keep model diagnostic only".to_string()),
        allowed: true,
        diagnostic_only: true,
        reason_codes: vec![ReasonCode::OwnerModelReviewWorkflowBuilt],
    }];
    config.owner_model_review_paths[0] = sprint65_support::write_support_json(
        "model-ops-qa-suite-watchlist-diagnostic",
        "owner_actions.json",
        &actions,
    );
    let watchlist = ExternalModelResearchOpsRunner::default()
        .run_watchlist(&config)
        .expect("run diagnostic watchlist");
    assert!(watchlist.entries.iter().any(|entry| {
        entry.model_id == "diag-model"
            && entry.watch_status == ExternalModelWatchStatus::DiagnosticOnly
    }));
}
