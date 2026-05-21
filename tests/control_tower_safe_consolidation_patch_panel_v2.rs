mod support;

use support::sprint108_support::{read_fixture, run_sprint108};

#[test]
fn control_tower_safe_patch_panel_matches_expected_fixture() {
    let bundle = run_sprint108(
        "soma_control_tower_safe_consolidation_patch_v2.toml",
        "control-tower-safe-consolidation-patch-v2",
    );
    let actual = serde_json::to_value(&bundle.control_tower_safe_consolidation_patch_panel_v2)
        .expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint108_data/control_tower_safe_consolidation_patch_v2_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .control_tower_safe_consolidation_patch_panel_v2
            .warnings
            .iter()
            .any(|warning| warning.contains("No run-tests button"))
    );
}
