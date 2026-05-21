mod support;

use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn moved_assertions_and_binary_delta_are_preserved_in_bundle() {
    let bundle = sprint::run_sprint86_bundle(
        "soma_sprint86_residual_gate_recover.toml",
        "workspace-legacy-regression-suite",
    );
    assert_eq!(
        bundle.workspace_binary_delta_report_v2.binary_count_before,
        Some(20)
    );
    assert_eq!(
        bundle.workspace_binary_delta_report_v2.binary_count_after,
        Some(8)
    );
    assert!(
        bundle
            .legacy_integration_migration_report
            .migrated_files
            .iter()
            .any(|file| file.ends_with("official_expansion_status_mapping.rs"))
    );
    assert_eq!(
        bundle.safety_coverage_preservation_report_v2.safety_status,
        soma_zero::SafetyCoveragePreservationReportV2Status::SafetyCoveragePreserved
    );
}

#[test]
fn legacy_regression_bundle_is_deterministic_and_has_no_semantic_drift_sample() {
    let first = sprint::run_sprint86_bundle(
        "soma_sprint86_residual_gate_recover.toml",
        "workspace-legacy-regression-suite-first",
    );
    let second = sprint::run_sprint86_bundle(
        "soma_sprint86_residual_gate_recover.toml",
        "workspace-legacy-regression-suite-second",
    );
    assert_eq!(
        first.legacy_integration_migration_report,
        second.legacy_integration_migration_report
    );
    assert_eq!(
        first.residual_binary_consolidation_plan,
        second.residual_binary_consolidation_plan
    );
    harness::assert_deterministic_text(&first.final_summary, &second.final_summary);
}
