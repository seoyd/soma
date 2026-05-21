mod support;

use serde_json::Value;
use support::sprint113_support::{read_fixture, run_sprint113};

#[test]
fn control_tower_real_workspace_observation_panel_is_read_only() {
    let bundle = run_sprint113(
        "soma_control_tower_real_workspace_observation.toml",
        "control-tower-real-workspace-observation",
    );
    let expected: Value =
        read_fixture("sprint113_data/control_tower_real_workspace_observation_expected.json");
    assert_eq!(
        bundle
            .control_tower_real_workspace_observation_panel
            .panel_id,
        expected["panel_id"].as_str().unwrap()
    );
    assert!(
        bundle
            .control_tower_real_workspace_observation_panel
            .static_read_only
    );
    assert!(
        bundle
            .control_tower_real_workspace_observation_panel
            .no_run_button
    );
    assert!(
        bundle
            .control_tower_real_workspace_observation_panel
            .no_apply_patch_button
    );
    assert!(
        bundle
            .control_tower_real_workspace_observation_panel
            .no_train_runtime_live_order_account_controls
    );
    assert_eq!(
        bundle
            .control_tower_real_workspace_observation_panel
            .acceptance_truth_status,
        "AcceptanceTruthReadyWithWarnings"
    );
}
