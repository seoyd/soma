mod support;

use soma_zero::ControlTowerDeferredObservationExecutionPanel;
use support::sprint117_support::{read_fixture, run_sprint117};

#[test]
fn control_tower_deferred_observation_execution_panel_is_read_only() {
    let bundle = run_sprint117(
        "soma_control_tower_deferred_observation_execution.toml",
        "control-tower-deferred-observation-execution",
    );
    let expected: ControlTowerDeferredObservationExecutionPanel =
        read_fixture("sprint117_data/control_tower_deferred_observation_expected.json");
    assert_eq!(
        bundle.control_tower_deferred_observation_execution_panel,
        expected
    );
    let panel = bundle.control_tower_deferred_observation_execution_panel;
    assert!(panel.static_read_only);
    assert!(panel.no_run_button);
    assert!(panel.no_action_button);
    assert!(panel.no_train_runtime_live_order_account_controls);
}
