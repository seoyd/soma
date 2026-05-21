mod support;

use soma_zero::ControlTowerWorkspaceTimeoutTrackExecutionPanel;
use support::sprint116_support::{read_fixture, run_sprint116};

#[test]
fn control_tower_workspace_timeout_track_execution_panel_is_read_only() {
    let bundle = run_sprint116(
        "soma_control_tower_workspace_timeout_track_execution.toml",
        "control-tower-workspace-timeout-track-execution",
    );
    let expected: ControlTowerWorkspaceTimeoutTrackExecutionPanel =
        read_fixture("sprint116_data/control_tower_timeout_track_expected.json");
    assert_eq!(
        bundle.control_tower_workspace_timeout_track_execution_panel,
        expected
    );
    let panel = bundle.control_tower_workspace_timeout_track_execution_panel;
    assert!(panel.static_read_only);
    assert!(panel.no_run_button);
    assert!(panel.no_action_button);
    assert!(panel.no_train_runtime_live_order_account_controls);
}
