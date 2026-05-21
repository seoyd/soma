mod support;

use support::sprint112_support::run_sprint112;

#[test]
fn control_tower_fifth_patch_reevaluation_panel_is_read_only_and_preserves_no_apply() {
    let bundle = run_sprint112(
        "soma_control_tower_fifth_patch_reevaluation.toml",
        "control-tower-fifth-patch",
    );
    let panel = bundle.control_tower_fifth_patch_reevaluation_panel;
    assert!(panel.static_read_only);
    assert!(panel.no_apply_patch_button);
    assert!(panel.no_train_runtime_live_order_account_controls);
    assert_eq!(
        panel.no_apply_guarantee_status,
        "FifthPatchNoApplyGuaranteed"
    );
}
