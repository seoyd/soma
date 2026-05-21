mod support;

use serde_json::Value;
use support::sprint114_support::{read_fixture, run_sprint114};

#[test]
fn control_tower_mixed_family_panel_is_read_only() {
    let bundle = run_sprint114(
        "soma_control_tower_mixed_family_isolation.toml",
        "control-tower-mixed-family-isolation",
    );
    let expected: Value =
        read_fixture("sprint114_data/control_tower_mixed_family_isolation_expected.json");
    let panel = bundle.control_tower_mixed_family_isolation_panel;
    assert_eq!(panel.panel_id, expected["panel_id"].as_str().unwrap());
    assert!(panel.static_read_only);
    assert!(panel.no_run_button);
    assert!(panel.no_apply_patch_button);
    assert!(panel.no_train_runtime_live_order_account_controls);
    assert!(
        panel
            .still_mixed_families
            .contains(&"LinkTimeCost".to_string())
    );
}
