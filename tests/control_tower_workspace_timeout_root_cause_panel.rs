mod support;

use soma_zero::ControlTowerWorkspaceTimeoutRootCausePanel;
use support::sprint111_support::{read_fixture, run_sprint111};

#[test]
fn control_tower_workspace_timeout_root_cause_panel_matches_fixture() {
    let bundle = run_sprint111(
        "soma_control_tower_workspace_timeout_root_cause.toml",
        "control-tower-workspace-timeout-root-cause",
    );
    let expected: ControlTowerWorkspaceTimeoutRootCausePanel =
        read_fixture("sprint111_data/control_tower_timeout_root_cause_expected.json");
    assert_eq!(
        bundle.control_tower_workspace_timeout_root_cause_panel,
        expected
    );
    assert!(
        bundle
            .control_tower_workspace_timeout_root_cause_panel
            .static_read_only
    );
    assert!(
        bundle
            .control_tower_workspace_timeout_root_cause_panel
            .no_run_tests_button
    );
}
