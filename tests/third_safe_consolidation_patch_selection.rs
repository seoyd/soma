mod support;

use soma_zero::SafeConsolidationPatchV3Runner;
use support::sprint109_support::{read_fixture, run_sprint109, sprint109_config_from_example};

#[test]
fn third_safe_patch_selection_matches_expected_fixture() {
    let bundle = run_sprint109(
        "soma_third_safe_consolidation_patch_selection.toml",
        "third-safe-consolidation-patch-selection",
    );
    let actual = serde_json::to_value(&bundle.third_safe_consolidation_patch_selection_report)
        .expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint109_data/third_patch_selection_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(
        bundle
            .third_safe_consolidation_patch_selection_report
            .selected_target_group,
        "render-helper-diagnostics"
    );
    assert!(
        !bundle
            .third_safe_consolidation_patch_selection_report
            .candidate_targets
            .contains(&"tests/shared_fixture_harness_expansion_plan_v2.rs".to_string())
    );
    assert!(
        !bundle
            .third_safe_consolidation_patch_selection_report
            .candidate_targets
            .contains(&"tests/shared_output_dir_helper_application_v1.rs".to_string())
    );
}

#[test]
fn third_safe_patch_selection_rejects_no_safe_candidate() {
    let mut config = sprint109_config_from_example(
        "soma_third_safe_consolidation_patch_selection.toml",
        "third-safe-consolidation-patch-selection-none",
    );
    config.apply_one_safe_consolidation = false;
    let bundle = SafeConsolidationPatchV3Runner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        bundle
            .third_safe_consolidation_patch_selection_report
            .selected_status,
        "NoSafeCandidate"
    );
}
