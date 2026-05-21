mod support;

use soma_zero::{SafeConsolidationPatchV3Config, SafeConsolidationPatchV3Runner};
use support::sprint109_support::{run_sprint109, sprint109_config_from_example};

#[test]
fn config_defaults_are_safe_and_runner_builds_bundle() {
    let config = SafeConsolidationPatchV3Config::default();
    assert_eq!(config.max_targets_to_consolidate, 1);
    assert!(config.require_verification_reconciliation);
    assert!(config.require_assertion_ledger);
    assert!(config.require_equivalent_coverage_proof);
    assert!(config.require_safety_sentinel_preservation);
    assert!(config.require_no_hidden_skips);
    assert!(config.require_no_assertion_deletion);
    let toml = config.to_toml_string().expect("toml");
    for forbidden in ["runtime_llm", "training", "broker", "order", "account"] {
        assert!(!toml.contains(forbidden), "unexpected field {forbidden}");
    }

    let bundle = run_sprint109(
        "soma_sprint109_safe_consolidation_patch_v3.toml",
        "safe-consolidation-patch-v3",
    );
    assert_eq!(
        bundle
            .third_safe_consolidation_patch_selection_report
            .selected_status,
        "ThirdPatchCandidateSelected"
    );
    assert_eq!(
        bundle.safety_coverage_preservation_report_v25.safety_status,
        "SafetyCoveragePreserved"
    );
    assert_eq!(
        bundle
            .dual_agent_patch_verification_report_v3
            .verification_status,
        "DualAgentPatchVerifiedWithWarnings"
    );
}

#[test]
fn remote_paths_are_rejected() {
    let config = SafeConsolidationPatchV3Config {
        sprint108_bundle_paths: Some(vec!["https://example.com/report.json".to_string()]),
        ..SafeConsolidationPatchV3Config::default()
    };
    let err = config.validate().expect_err("remote path must fail");
    assert!(err.contains("must be local"));
}

#[test]
fn safety_coverage_missing_when_sentinel_preservation_is_disabled() {
    let mut config = sprint109_config_from_example(
        "soma_sprint109_safe_consolidation_patch_v3.toml",
        "safe-consolidation-patch-v3-missing-sentinel",
    );
    config.require_safety_sentinel_preservation = false;

    let bundle = SafeConsolidationPatchV3Runner::default()
        .run(&config)
        .expect("run sprint109");

    assert_eq!(
        bundle
            .safety_sentinel_preservation_report_v3
            .sentinel_status,
        "SafetySentinelMissing"
    );
    assert_eq!(
        bundle
            .safety_coverage_preservation_report_v25
            .safety_sentinel_preservation_guard_present,
        false
    );
    assert_eq!(
        bundle.safety_coverage_preservation_report_v25.safety_status,
        "SafetyCoverageMissing"
    );
    assert_eq!(
        bundle
            .dual_agent_patch_verification_report_v3
            .verification_status,
        "DualAgentPatchBlocked"
    );
}

#[test]
fn assertion_preservation_is_required_for_verification_and_safety_v24() {
    let mut config = sprint109_config_from_example(
        "soma_sprint109_safe_consolidation_patch_v3.toml",
        "safe-consolidation-patch-v3-missing-assertion-preservation",
    );
    config.require_no_assertion_deletion = false;

    let bundle = SafeConsolidationPatchV3Runner::default()
        .run(&config)
        .expect("run sprint109");

    assert_eq!(
        bundle
            .assertion_preservation_verification_report_v3
            .preservation_status,
        "AssertionDeletionDetected"
    );
    assert_eq!(
        bundle
            .acceptance_recovery_verification_report_v3
            .verification_status,
        "AcceptanceRecoveryVerificationFailed"
    );
    assert_eq!(
        bundle.safety_coverage_preservation_report_v25.safety_status,
        "SafetyCoverageMissing"
    );
    assert_eq!(
        bundle
            .dual_agent_patch_verification_report_v3
            .verification_status,
        "DualAgentPatchBlocked"
    );
}

#[test]
fn equivalent_coverage_is_required_before_retiring_third_target() {
    let mut config = sprint109_config_from_example(
        "soma_sprint109_safe_consolidation_patch_v3.toml",
        "safe-consolidation-patch-v3-missing-equivalent-coverage",
    );
    config.require_equivalent_coverage_proof = false;

    let bundle = SafeConsolidationPatchV3Runner::default()
        .run(&config)
        .expect("run sprint109");

    assert_eq!(
        bundle.equivalent_coverage_proof_report_v2.proof_status,
        "EquivalentCoverageMissing"
    );
    assert!(
        bundle
            .retired_narrow_target_manifest_v2
            .retired_targets
            .is_empty()
    );
    assert_eq!(
        bundle.retired_narrow_target_manifest_v2.retired_status,
        "NarrowTargetRetirementBlocked"
    );
    assert_eq!(
        bundle.safety_coverage_preservation_report_v25.safety_status,
        "SafetyCoverageMissing"
    );
}
