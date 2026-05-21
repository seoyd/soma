mod support;

use support::sprint111_support::run_sprint111;

#[test]
fn control_tower_fifth_patch_decision_panel_stays_read_only() {
    let bundle = run_sprint111(
        "soma_control_tower_fifth_patch_decision.toml",
        "control-tower-fifth-patch-decision",
    );
    assert!(
        bundle
            .control_tower_fifth_patch_decision_panel
            .static_read_only
    );
    assert!(
        bundle
            .control_tower_fifth_patch_decision_panel
            .no_apply_patch_button
    );
    assert!(
        bundle
            .control_tower_fifth_patch_decision_panel
            .no_run_tests_button
    );
    assert_eq!(
        bundle
            .control_tower_fifth_patch_decision_panel
            .decision_gate_status,
        bundle.fifth_patch_decision_gate.gate_status
    );
}
