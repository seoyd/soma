mod support;

use soma_zero::Sprint105VerificationPatchClosureRunner;
use support::sprint105_support::{run_sprint105, write_support_json};

#[test]
fn overclaim_guard_requires_finished_and_passed() {
    let bundle = run_sprint105("soma_overclaim_regression_guard.toml", "overclaim_guard");
    assert!(
        bundle
            .overclaim_regression_guard_report
            .full_acceptance_requires_finished_and_passed
    );
    assert!(
        !bundle
            .overclaim_regression_guard_report
            .focused_pass_is_full_acceptance
    );
    assert!(
        !bundle
            .overclaim_regression_guard_report
            .verification_pass_is_full_acceptance
    );
    assert!(
        !bundle
            .overclaim_regression_guard_report
            .no_run_pass_is_full_acceptance
    );
}

#[test]
fn overclaim_regression_is_detected_if_workspace_truth_lies() {
    let workspace_truth = write_support_json(
        "overclaim_regression_detected",
        "workspace_truth.json",
        &serde_json::json!({
            "truth_status": "WorkspaceTruthImported",
            "no_run_started": true,
            "no_run_finished": false,
            "full_started": true,
            "full_finished": false,
            "can_claim_full_acceptance": true
        }),
    );
    let mut config = soma_zero::Sprint105VerificationPatchClosureConfig::default();
    config.workspace_truth_paths = Some(vec![workspace_truth]);
    let bundle = Sprint105VerificationPatchClosureRunner::default()
        .run(&config)
        .expect("run");
    assert!(bundle.overclaim_regression_guard_report.regression_detected);
}

#[test]
fn no_run_pass_is_never_full_acceptance_even_when_full_workspace_passed() {
    let workspace_truth = write_support_json(
        "overclaim_no_run_pass_not_full",
        "workspace_truth.json",
        &serde_json::json!({
            "truth_status": "WorkspaceTruthImported",
            "no_run_started": true,
            "no_run_finished": true,
            "no_run_passed": true,
            "full_started": true,
            "full_finished": true,
            "full_passed": true,
            "can_claim_full_acceptance": true
        }),
    );
    let mut config = soma_zero::Sprint105VerificationPatchClosureConfig::default();
    config.workspace_truth_paths = Some(vec![workspace_truth]);
    let bundle = Sprint105VerificationPatchClosureRunner::default()
        .run(&config)
        .expect("run");
    assert!(
        !bundle
            .overclaim_regression_guard_report
            .no_run_pass_is_full_acceptance
    );
    assert!(bundle.final_verification_gate_v2.full_workspace_accepted);
}
