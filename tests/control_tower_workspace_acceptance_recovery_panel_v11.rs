mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn control_tower_workspace_recovery_panel_shows_no_run_full_and_truth() {
    let bundle = run_sprint110(
        "soma_control_tower_workspace_acceptance_recovery_v11.toml",
        "control-tower-workspace-acceptance-recovery-v11",
    );
    let panel = bundle.control_tower_workspace_acceptance_recovery_panel_v11;
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
            .any(|w| w.contains("No run-tests button"))
    );
}
