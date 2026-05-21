mod support;

use soma_zero::WorkspaceAcceptanceTruthClosureStatus;
use support::sprint99_support::run_sprint99;

#[test]
fn workspace_acceptance_truth_closure_keeps_full_workspace_separate() {
    let bundle = run_sprint99(
        "soma_workspace_acceptance_truth_closure_plan.toml",
        "workspace-acceptance-truth-closure-plan",
    );
    let plan = bundle.workspace_acceptance_truth_closure_plan;
    let attempt = bundle.workspace_acceptance_attempt_v16;
    assert_eq!(
        plan.closure_status,
        WorkspaceAcceptanceTruthClosureStatus::WorkspaceTruthStillOpen
    );
    assert!(!plan.can_claim_full_acceptance);
    assert!(
        plan.recommended_actions
            .contains(&"KeepFocusedTestsSeparate".to_string())
    );
    assert!(!attempt.can_claim_full_acceptance);
    assert!(!attempt.full_finished);
}
