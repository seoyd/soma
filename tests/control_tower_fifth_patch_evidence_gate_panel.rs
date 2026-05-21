mod support;

use support::sprint113_support::run_sprint113;

#[test]
fn control_tower_fifth_patch_evidence_gate_panel_shows_no_apply_guarantee() {
    let bundle = run_sprint113(
        "soma_control_tower_fifth_patch_evidence_gate.toml",
        "control-tower-fifth-patch-evidence-gate",
    );
    assert!(
        bundle
            .control_tower_fifth_patch_evidence_gate_panel
            .static_read_only
    );
    assert!(
        bundle
            .control_tower_fifth_patch_evidence_gate_panel
            .no_apply_patch_button
    );
    assert!(
        bundle
            .control_tower_fifth_patch_evidence_gate_panel
            .no_train_runtime_live_order_account_controls
    );
    assert_eq!(
        bundle
            .control_tower_fifth_patch_evidence_gate_panel
            .no_apply_guarantee_status,
        "FifthPatchNoApplyGuaranteed"
    );
    assert_eq!(
        bundle
            .control_tower_fifth_patch_evidence_gate_panel
            .fifth_gate_status,
        "FifthPatchStillBlocked"
    );
}
