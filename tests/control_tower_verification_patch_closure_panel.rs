mod support;

use support::sprint105_support::run_sprint105;

#[test]
fn verification_patch_panel_is_read_only_and_shows_statuses() {
    let bundle = run_sprint105(
        "soma_control_tower_verification_patch_closure.toml",
        "control_tower_verification_patch_closure_panel",
    );
    let panel = &bundle.control_tower_verification_patch_closure_panel;
    assert!(panel.final_gate_status.contains("GateV2"));
    assert!(
        panel
            .warnings
            .iter()
            .any(|value| value.contains("read-only"))
    );
    assert!(
        panel
            .warnings
            .iter()
            .any(|value| value.contains("no verification execution button"))
    );
}
