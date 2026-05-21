mod support;

use support::sprint109_support::{read_fixture, run_sprint109};

#[test]
fn control_tower_safe_patch_panel_matches_expected_fixture() {
    let bundle = run_sprint109(
        "soma_control_tower_safe_consolidation_patch_v3.toml",
        "control-tower-safe-consolidation-patch-v3",
    );
    let actual = serde_json::to_value(&bundle.control_tower_safe_consolidation_patch_panel_v3)
        .expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint109_data/control_tower_safe_consolidation_patch_v3_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .control_tower_safe_consolidation_patch_panel_v3
            .warnings
            .iter()
            .any(|warning| warning.contains("No run-tests button"))
    );
}
