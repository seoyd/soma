mod support;

use support::sprint109_support::run_sprint109;

#[test]
fn control_tower_workspace_recovery_panel_shows_no_run_full_and_truth() {
    let bundle = run_sprint109(
        "soma_control_tower_workspace_acceptance_recovery_v10.toml",
        "control-tower-workspace-acceptance-recovery-v10",
    );
    let panel = bundle.control_tower_workspace_acceptance_recovery_panel_v10;
    assert_eq!(panel.current_no_run_status, "NotRun");
    assert_eq!(panel.current_full_status, "NotRun");
    assert_eq!(
        panel.acceptance_truth_status,
        "AcceptanceTruthReadyWithWarnings"
    );
    assert!(
        panel
            .warnings
            .iter()
            .any(|warning| warning.contains("No run-tests button"))
    );
    assert!(
        panel
            .warnings
            .iter()
            .any(|warning| warning.contains("train/runtime/live/order/account/browser controls"))
    );
}
