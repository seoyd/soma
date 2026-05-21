mod support;

use support::sprint107_support::{read_fixture, run_sprint107};

#[test]
fn control_tower_safe_patch_panel_is_read_only() {
    let bundle = run_sprint107(
        "soma_control_tower_safe_consolidation_patch_v1.toml",
        "control-tower-safe-consolidation-patch-v1",
    );
    let actual = serde_json::to_value(&bundle.control_tower_safe_consolidation_patch_panel_v1)
        .expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint107_data/control_tower_safe_consolidation_patch_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .control_tower_safe_consolidation_patch_panel_v1
            .warnings
            .iter()
            .any(|warning| warning.contains("No run-tests button"))
    );
}
