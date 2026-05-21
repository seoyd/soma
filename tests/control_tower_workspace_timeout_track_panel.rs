mod support;

use support::sprint115_support::run_sprint115;

#[test]
fn control_tower_workspace_timeout_track_panel_is_read_only() {
    let bundle = run_sprint115(
        "soma_control_tower_workspace_timeout_track.toml",
        "control-tower-workspace-timeout-track",
    );
    let panel = bundle.control_tower_workspace_timeout_track_panel;
    assert!(panel.static_read_only);
    assert!(panel.no_run_button);
    assert!(panel.no_train_runtime_live_order_account_controls);
    assert_eq!(
        panel.timeout_track_status,
        "WorkspaceTimeoutDiagnosticTrackActive"
    );
}
