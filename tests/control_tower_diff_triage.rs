#[path = "support/sprint69_support.rs"]
mod support;

use std::fs;

use soma_zero::ControlTowerDiffTriagePanelStatus;

#[test]
fn control_tower_diff_triage_panel_and_fragments_are_static_and_safe() {
    let bundle = support::run_triage(
        "soma_unexpected_diff_triage.toml",
        "control-tower-diff-triage",
    );

    assert_eq!(
        bundle.control_tower_diff_triage_panel.panel_status,
        ControlTowerDiffTriagePanelStatus::NeedMoreEvidence
    );
    assert!(
        bundle
            .control_tower_diff_triage_panel
            .unexpected_diff_summary
            .contains("UnexpectedDiffExplained;triaged=2;explained=2;unknown=0")
    );
    assert!(
        bundle
            .control_tower_diff_triage_panel
            .trace_warning_summary
            .contains("TraceWarningsReduced;original=4;reduced=3;remaining=1;ratio=0.7500")
    );

    let fragments_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("sprint70-tests")
        .join("control-tower-diff-triage")
        .join("sprint70-unexpected-diff-triage")
        .join("fragments");
    for file_name in [
        "unexpected_diff_triage.html",
        "contract_alignment_audit.html",
        "owner_review_closure.html",
        "trace_warning_reduction.html",
        "downgrade_evidence_closure.html",
    ] {
        let text = fs::read_to_string(fragments_dir.join(file_name)).expect("fragment text");
        assert!(!text.contains("<script"));
        assert!(!text.contains("<form"));
        assert!(!text.contains("KIS_APP_KEY"));
        assert!(!text.contains("KIS_APP_SECRET"));
        assert!(!text.contains("account balance"));
        assert!(!text.contains("holdings"));
    }
}
