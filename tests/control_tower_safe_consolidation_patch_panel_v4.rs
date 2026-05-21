mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn control_tower_safe_patch_panel_shows_reconciliation_and_read_only_warnings() {
    let bundle = run_sprint110(
        "soma_control_tower_safe_consolidation_patch_v4.toml",
        "control-tower-safe-consolidation-patch-v4",
    );
    let panel = bundle.control_tower_safe_consolidation_patch_panel_v4;
    assert_eq!(
        panel.verification_reconciliation_status,
        "Sprint109ValidationReconciledWithWarnings"
    );
    assert_eq!(panel.patch_selection_status, "FourthPatchCandidateSelected");
    assert!(
        panel
            .warnings
            .iter()
            .any(|w| w.contains("No run-tests button"))
    );
}
