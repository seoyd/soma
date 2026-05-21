mod support;

use soma_zero::SafeConsolidationPatchV2Runner;
use support::sprint108_support::{read_fixture, run_sprint108, sprint108_config_from_example};

#[test]
fn second_safe_patch_selection_matches_expected_fixture() {
    let bundle = run_sprint108(
        "soma_second_safe_consolidation_patch_selection.toml",
        "second-safe-consolidation-patch-selection",
    );
    let actual = serde_json::to_value(&bundle.second_safe_consolidation_patch_selection_report)
        .expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint108_data/second_patch_selection_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(
        bundle
            .second_safe_consolidation_patch_selection_report
            .selected_target_group,
        "output-dir-helper-diagnostics"
    );
    assert!(
        !bundle
            .second_safe_consolidation_patch_selection_report
            .candidate_targets
            .contains(&"tests/shared_fixture_harness_expansion_plan_v2.rs".to_string())
    );
}

#[test]
fn second_safe_patch_selection_rejects_no_safe_candidate() {
    let mut config = sprint108_config_from_example(
        "soma_second_safe_consolidation_patch_selection.toml",
        "second-safe-consolidation-patch-selection-none",
    );
    config.apply_one_safe_consolidation = false;
    let bundle = SafeConsolidationPatchV2Runner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        bundle
            .second_safe_consolidation_patch_selection_report
            .selected_status,
        "NoSafeCandidate"
    );
}
