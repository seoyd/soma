mod support;

use soma_zero::{SafeConsolidationPatchV1Config, SafeConsolidationPatchV1Runner};
use support::sprint107_support::{run_sprint107, sprint107_config_from_example};

#[test]
fn config_defaults_are_safe_and_runner_builds_bundle() {
    let config = SafeConsolidationPatchV1Config::default();
    assert_eq!(config.max_targets_to_consolidate, 1);
    assert!(config.require_assertion_ledger);
    assert!(config.require_safety_sentinel_preservation);
    assert!(config.require_no_hidden_skips);
    assert!(config.require_no_assertion_deletion);
    let toml = config.to_toml_string().expect("toml");
    for forbidden in ["runtime_llm", "training", "broker", "order", "account"] {
        assert!(!toml.contains(forbidden), "unexpected field {forbidden}");
    }

    let bundle = run_sprint107(
        "soma_sprint107_safe_consolidation_patch.toml",
        "safe-consolidation-patch-v1",
    );
    assert_eq!(
        bundle
            .safe_consolidation_patch_selection_report
            .selected_status,
        "PatchCandidateSelected"
    );
    assert_eq!(
        bundle.safety_coverage_preservation_report_v23.safety_status,
        "SafetyCoveragePreserved"
    );
    assert_eq!(
        bundle
            .dual_agent_patch_verification_report_v1
            .verification_status,
        "DualAgentPatchVerifiedWithWarnings"
    );
}

#[test]
fn remote_paths_are_rejected() {
    let config = SafeConsolidationPatchV1Config {
        sprint106_bundle_paths: Some(vec!["https://example.com/report.json".to_string()]),
        ..SafeConsolidationPatchV1Config::default()
    };
    let err = config.validate().expect_err("remote path must fail");
    assert!(err.contains("must be local"));
}

#[test]
fn safety_coverage_missing_when_sentinel_preservation_is_disabled() {
    let mut config = sprint107_config_from_example(
        "soma_sprint107_safe_consolidation_patch.toml",
        "safe-consolidation-patch-v1-missing-sentinel",
    );
    config.require_safety_sentinel_preservation = false;

    let bundle = SafeConsolidationPatchV1Runner::default()
        .run(&config)
        .expect("run sprint107");

    assert_eq!(
        bundle
            .safety_sentinel_preservation_report_v1
            .sentinel_status,
        "SafetySentinelMissing"
    );
    assert_eq!(
        bundle
            .safety_coverage_preservation_report_v23
            .safety_sentinel_preservation_guard_present,
        false
    );
    assert_eq!(
        bundle.safety_coverage_preservation_report_v23.safety_status,
        "SafetyCoverageMissing"
    );
    assert_eq!(
        bundle
            .dual_agent_patch_verification_report_v1
            .verification_status,
        "DualAgentPatchBlocked"
    );
}
