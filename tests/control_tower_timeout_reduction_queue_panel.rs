mod support;

use soma_zero::ControlTowerTimeoutReductionQueuePanel;
use support::sprint118_support::{read_fixture, run_sprint118};

#[test]
fn control_tower_timeout_reduction_queue_panel_is_read_only() {
    let bundle = run_sprint118(
        "soma_control_tower_timeout_reduction_queue.toml",
        "control-tower-timeout-reduction-queue",
    );
    let expected: ControlTowerTimeoutReductionQueuePanel =
        read_fixture("sprint118_data/control_tower_timeout_reduction_queue_expected.json");
    assert_eq!(bundle.control_tower_timeout_reduction_queue_panel, expected);
    let panel = bundle.control_tower_timeout_reduction_queue_panel;
    assert!(panel.static_read_only);
    assert!(panel.no_run_button);
    assert!(panel.no_apply_button);
    assert!(panel.no_train_runtime_live_order_account_controls);
}
