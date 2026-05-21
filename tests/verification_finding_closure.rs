mod support;

use serde_json::to_value;
use soma_zero::Sprint105VerificationPatchClosureConfig;
use support::sprint105_support::{read_fixture, run_sprint105};

#[test]
fn verification_findings_close_and_known_warnings_stay_explicit() {
    let config = Sprint105VerificationPatchClosureConfig::default();
    assert!(config.require_review_patch_closure);
    assert!(config.require_overclaim_guard);
    assert!(config.require_safety_boolean_audit);
    assert!(config.require_lifecycle_warning_closure);
    assert!(config.require_risk_transition_audit);
    assert!(config.require_lower_confidence_carry_forward);
    assert!(config.require_workspace_truth_recovery_plan);
    assert!(config.preserve_dual_agent_separation);
    assert!(config.preserve_runtime_deferred);
    assert!(config.preserve_safety_guards);
    assert!(config.validate().is_ok());

    let bundle = run_sprint105(
        "soma_sprint105_verification_patch_close.toml",
        "verification_finding_closure",
    );
    let actual = to_value(&bundle.verification_finding_closure_report).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint105_data/verification_finding_closure_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .verification_finding_closure_report
            .closure_status
            .contains("Closed")
    );
}

#[test]
fn sprint105_rejects_remote_paths() {
    let mut config = Sprint105VerificationPatchClosureConfig::default();
    config.output_root = "https://example.com/out".to_string();
    assert!(config.validate().is_err());
}
