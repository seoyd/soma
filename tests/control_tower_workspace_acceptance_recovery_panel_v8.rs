mod support;

use support::sprint107_support::run_sprint107;

#[test]
fn control_tower_workspace_panel_v8_shows_no_run_full_and_truth() {
    let bundle = run_sprint107(
        "soma_control_tower_workspace_acceptance_recovery_v8.toml",
        "control-tower-workspace-acceptance-recovery-v8",
    );
    let panel = bundle.control_tower_workspace_acceptance_recovery_panel_v8;
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
}
