mod support;

use support::sprint105_support::run_sprint105;

#[test]
fn review_patch_effect_detects_expected_review_fixes() {
    let bundle = run_sprint105("soma_review_patch_effect.toml", "review_patch_effect");
    let report = &bundle.review_patch_effect_report;
    assert!(report.overclaim_patch_detected);
    assert!(report.workspace_attempt_patch_detected);
    assert!(report.safety_boolean_patch_detected);
    assert!(report.missing_artifact_policy_patch_detected);
    assert!(report.paper_rejected_transition_patch_detected);
    assert!(report.risk_transition_patch_detected);
}
