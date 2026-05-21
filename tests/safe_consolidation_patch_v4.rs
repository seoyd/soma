mod support;

use soma_zero::{SafeConsolidationPatchV4Config, SafeConsolidationPatchV4Runner};
use support::sprint110_support::{run_sprint110, sprint110_config_from_example};

#[test]
fn config_defaults_are_safe_and_runner_builds_bundle() {
    let config = SafeConsolidationPatchV4Config::default();
    assert_eq!(config.max_targets_to_consolidate, 1);
    assert!(config.require_sprint109_validation_reconciliation);
    assert!(config.require_cumulative_ledger);
    assert!(config.require_assertion_ledger);
    assert!(config.require_equivalent_coverage_proof);
    assert!(config.require_safety_sentinel_preservation);
    assert!(config.require_no_hidden_skips);
    assert!(config.require_no_assertion_deletion);
    let toml = config.to_toml_string().expect("toml");
    for forbidden in ["runtime_llm", "training", "broker", "order", "account"] {
        assert!(!toml.contains(forbidden), "unexpected field {forbidden}");
    }

    let bundle = run_sprint110(
        "soma_sprint110_safe_consolidation_patch_v4.toml",
        "safe-consolidation-patch-v4",
    );
    assert_eq!(
        bundle
            .fourth_safe_consolidation_patch_selection_report
            .selected_status,
        "FourthPatchCandidateSelected"
    );
    assert_eq!(
        bundle
            .sprint109_external_validation_reconciliation_report
            .reconciliation_status,
        "Sprint109ValidationReconciledWithWarnings"
    );
    assert_eq!(
        bundle.safety_coverage_preservation_report_v26.safety_status,
        "SafetyCoveragePreserved"
    );
}

#[test]
fn remote_paths_are_rejected() {
    let config = SafeConsolidationPatchV4Config {
        sprint109_bundle_paths: Some(vec!["https://example.com/report.json".to_string()]),
        ..SafeConsolidationPatchV4Config::default()
    };
    let err = config.validate().expect_err("remote path must fail");
    assert!(err.contains("must be local"));
}

#[test]
fn safety_coverage_missing_when_sentinel_preservation_is_disabled() {
    let mut config = sprint110_config_from_example(
        "soma_sprint110_safe_consolidation_patch_v4.toml",
        "safe-consolidation-patch-v4-missing-sentinel",
    );
    config.require_safety_sentinel_preservation = false;

    let bundle = SafeConsolidationPatchV4Runner::default()
        .run(&config)
        .expect("run sprint110");

    assert_eq!(
        bundle
            .safety_sentinel_preservation_report_v4
            .sentinel_status,
        "SafetySentinelMissing"
    );
    assert!(
        !bundle
            .safety_coverage_preservation_report_v26
            .safety_sentinel_preservation_guard_present
    );
    assert_eq!(
        bundle.safety_coverage_preservation_report_v26.safety_status,
        "SafetyCoverageMissing"
    );
}

#[test]
fn equivalent_coverage_is_required_before_retiring_target() {
    let mut config = sprint110_config_from_example(
        "soma_sprint110_safe_consolidation_patch_v4.toml",
        "safe-consolidation-patch-v4-missing-equivalent-coverage",
    );
    config.require_equivalent_coverage_proof = false;

    let bundle = SafeConsolidationPatchV4Runner::default()
        .run(&config)
        .expect("run sprint110");

    assert_eq!(
        bundle.equivalent_coverage_proof_report_v3.proof_status,
        "EquivalentCoverageMissing"
    );
    assert!(
        bundle
            .retired_narrow_target_manifest_v4
            .retired_targets
            .is_empty()
    );
    assert_eq!(
        bundle.retired_narrow_target_manifest_v4.retired_status,
        "NarrowTargetRetirementBlocked"
    );
}

#[test]
fn sprint109_validation_reconciliation_is_required_for_safety_coverage() {
    let mut config = sprint110_config_from_example(
        "soma_sprint110_safe_consolidation_patch_v4.toml",
        "safe-consolidation-patch-v4-missing-validation-reconciliation",
    );
    config.require_sprint109_validation_reconciliation = false;

    let bundle = SafeConsolidationPatchV4Runner::default()
        .run(&config)
        .expect("run sprint110");

    assert!(
        !bundle
            .safety_coverage_preservation_report_v26
            .sprint109_validation_reconciliation_guard_present
    );
    assert_eq!(
        bundle.safety_coverage_preservation_report_v26.safety_status,
        "SafetyCoverageMissing"
    );
}
