mod common;
#[path = "support/sprint66_support.rs"]
mod sprint66_support;

use soma_zero::{
    ModelOpsReviewClosureConfig, ModelReviewClosureDecision, ModelReviewClosureRunner,
    ModelReviewClosureStatus, OwnerModelReviewAction, OwnerModelReviewActionKind, ReasonCode,
};

#[test]
fn review_closure_config_defaults_and_remote_paths_are_rejected() {
    let config = ModelOpsReviewClosureConfig::default();
    assert!(config.require_owner_reason_for_retire);
    assert!(config.require_owner_reason_for_downgrade);
    let encoded = toml::to_string(&config).expect("serialize closure config");
    for forbidden in ["live", "runtime", "training", "broker", "order", "account"] {
        assert!(
            !encoded.contains(&format!("{forbidden}_")),
            "unexpected forbidden config field: {forbidden}"
        );
    }

    let mut bad = config.clone();
    bad.external_model_research_ops_paths = vec!["https://example.com/ops.json".to_string()];
    assert!(bad.validate().is_err());
}

#[test]
fn review_closure_applies_keep_retire_and_request_more_predictions() {
    let config = sprint66_support::closure_config_from_example(
        "soma_model_review_close.toml",
        "model-review-closure-base",
    );
    let bundle = ModelReviewClosureRunner::default()
        .run(&config)
        .expect("run model review closure");
    assert_eq!(
        bundle.model_review_closure_report.closure_status,
        ModelReviewClosureStatus::NeedsMorePredictions
    );
    assert!(
        bundle
            .model_review_closure_report
            .actions
            .iter()
            .any(|action| {
                action.model_id == "ext-model-a"
                    && action.model_version == "1.1.0"
                    && action.action == ModelReviewClosureDecision::KeepResearchCandidate
                    && action.applied
            })
    );
    assert!(
        bundle
            .model_review_closure_report
            .actions
            .iter()
            .any(|action| {
                action.model_id == "ext-model-a"
                    && action.model_version == "1.0.0"
                    && action.action == ModelReviewClosureDecision::RetireModelVersion
                    && action.applied
            })
    );
    assert!(
        bundle
            .model_review_closure_report
            .actions
            .iter()
            .any(|action| {
                action.model_id == "ext-model-b"
                    && action.action == ModelReviewClosureDecision::RequestMorePredictions
                    && action.applied
            })
    );
}

#[test]
fn review_closure_requires_reason_for_retire_and_downgrade_and_blocks_unsafe_notes() {
    let mut config = sprint66_support::closure_config_from_example(
        "soma_model_review_close.toml",
        "model-review-closure-reason-required",
    );
    let actions = vec![
        OwnerModelReviewAction {
            action_id: "retire-no-reason".to_string(),
            model_id: "ext-model-a".to_string(),
            model_version: "1.0.0".to_string(),
            action_kind: OwnerModelReviewActionKind::RetireModelVersion,
            note: None,
            allowed: true,
            diagnostic_only: true,
            reason_codes: vec![ReasonCode::OwnerModelReviewWorkflowBuilt],
        },
        OwnerModelReviewAction {
            action_id: "downgrade-live".to_string(),
            model_id: "ext-model-a".to_string(),
            model_version: "1.1.0".to_string(),
            action_kind: OwnerModelReviewActionKind::MarkModelDiagnosticOnly,
            note: Some("live runtime downgrade".to_string()),
            allowed: true,
            diagnostic_only: true,
            reason_codes: vec![ReasonCode::OwnerModelReviewWorkflowBuilt],
        },
    ];
    config.owner_model_review_action_paths[0] = sprint66_support::write_support_json(
        "model-review-closure-reason-required",
        "owner_actions.json",
        &actions,
    );

    let report = ModelReviewClosureRunner::default()
        .run(&config)
        .expect("run reason-required closure")
        .model_review_closure_report;
    assert!(report.actions.iter().any(|action| {
        action.model_id == "ext-model-a"
            && action.model_version == "1.0.0"
            && action.action == ModelReviewClosureDecision::RetireModelVersion
            && !action.allowed
    }));
    assert!(report.actions.iter().any(|action| {
        action.model_id == "ext-model-a"
            && action.model_version == "1.1.0"
            && action.action == ModelReviewClosureDecision::DowngradeToDiagnostic
            && !action.allowed
    }));
}

#[test]
fn review_closure_can_defer_review() {
    let mut config = sprint66_support::closure_config_from_example(
        "soma_model_review_close.toml",
        "model-review-closure-defer",
    );
    let actions = vec![OwnerModelReviewAction {
        action_id: "defer-ext-model-b".to_string(),
        model_id: "ext-model-b".to_string(),
        model_version: "1.0.0".to_string(),
        action_kind: OwnerModelReviewActionKind::DeferReview,
        note: Some("defer until additional review context is available".to_string()),
        allowed: true,
        diagnostic_only: false,
        reason_codes: vec![ReasonCode::OwnerModelReviewWorkflowBuilt],
    }];
    config.owner_model_review_action_paths[0] = sprint66_support::write_support_json(
        "model-review-closure-defer",
        "owner_actions.json",
        &actions,
    );
    let report = ModelReviewClosureRunner::default()
        .run(&config)
        .expect("run defer closure")
        .model_review_closure_report;
    assert!(report.actions.iter().any(|action| {
        action.model_id == "ext-model-b"
            && action.action == ModelReviewClosureDecision::DeferReview
            && !action.applied
    }));
    assert!(report.deferred_count > 0);
}
