mod support;

use soma_zero::ControlTowerWorkspaceDiagnosticPilotPanel;
use support::sprint112_support::{read_fixture, run_sprint112};

#[test]
fn control_tower_workspace_diagnostic_pilot_panel_matches_fixture_and_is_read_only() {
    let bundle = run_sprint112(
        "soma_control_tower_workspace_diagnostic_pilot.toml",
        "control-tower-diagnostic",
    );
    let expected: ControlTowerWorkspaceDiagnosticPilotPanel =
        read_fixture("sprint112_data/control_tower_diagnostic_pilot_expected.json");
    assert_eq!(
        bundle.control_tower_workspace_diagnostic_pilot_panel,
        expected
    );
    assert!(
        bundle
            .control_tower_workspace_diagnostic_pilot_panel
            .static_read_only
    );
    assert!(
        bundle
            .control_tower_workspace_diagnostic_pilot_panel
            .no_run_button
    );
    assert!(
        bundle
            .control_tower_workspace_diagnostic_pilot_panel
            .no_train_runtime_live_order_account_controls
    );
}
