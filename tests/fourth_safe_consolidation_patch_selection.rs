mod support;

use soma_zero::SafeConsolidationPatchV4Runner;
use support::sprint110_support::{run_sprint110, sprint110_config_from_example};

#[test]
fn fourth_safe_patch_selection_prefers_shared_toml_builder_target() {
    let bundle = run_sprint110(
        "soma_fourth_safe_consolidation_patch_selection.toml",
        "fourth-safe-consolidation-patch-selection",
    );
    let report = bundle.fourth_safe_consolidation_patch_selection_report;
    assert_eq!(
        report.selected_target_group,
        "tests/shared_toml_builder_application_v1.rs"
    );
    assert_eq!(report.selected_status, "FourthPatchCandidateSelected");
    assert_eq!(report.risk_class, "Low");
    for excluded in [
        "tests/shared_fixture_harness_expansion_plan_v2.rs",
        "tests/shared_output_dir_helper_application_v1.rs",
        "tests/shared_render_helper_application_v1.rs",
        "tests/committee_cli_safety.rs",
        "tests/workspace_cli_safety.rs",
        "tests/paper_lifecycle_safety.rs",
    ] {
        assert!(
            !report.candidate_targets.contains(&excluded.to_string()),
            "{excluded} must stay outside fourth-patch candidates"
        );
    }
}

#[test]
fn fourth_safe_patch_selection_rejects_when_disabled() {
    let mut config = sprint110_config_from_example(
        "soma_fourth_safe_consolidation_patch_selection.toml",
        "fourth-safe-consolidation-patch-selection-none",
    );
    config.apply_one_safe_consolidation = false;
    let bundle = SafeConsolidationPatchV4Runner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        bundle
            .fourth_safe_consolidation_patch_selection_report
            .selected_status,
        "NoSafeCandidate"
    );
}
