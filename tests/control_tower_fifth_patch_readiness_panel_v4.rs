mod support;

use support::sprint114_support::run_sprint114;

#[test]
fn control_tower_fifth_patch_readiness_panel_is_read_only() {
    let bundle = run_sprint114(
        "soma_control_tower_fifth_patch_readiness_v4.toml",
        "control-tower-fifth-patch-readiness-v4",
    );
    let panel = bundle.control_tower_fifth_patch_readiness_panel_v4;
    assert!(panel.static_read_only);
    assert!(panel.no_apply_patch_button);
    assert!(panel.no_run_button);
    assert!(panel.no_train_runtime_live_order_account_controls);
    assert_eq!(panel.fifth_gate_status, "FifthPatchStillBlocked");
}
