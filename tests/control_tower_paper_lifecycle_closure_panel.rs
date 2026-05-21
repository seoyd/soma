mod support;

use support::sprint105_support::run_sprint105;

#[test]
fn paper_lifecycle_panel_is_read_only_and_shows_statuses() {
    let bundle = run_sprint105(
        "soma_control_tower_paper_lifecycle_closure.toml",
        "control_tower_paper_lifecycle_closure_panel",
    );
    let panel = &bundle.control_tower_paper_lifecycle_closure_panel;
    assert!(panel.readiness_gate_status.contains("PaperLifecycle"));
    assert!(
        panel
            .warnings
            .iter()
            .any(|value| value.contains("read-only"))
    );
    assert!(
        panel
            .warnings
            .iter()
            .any(|value| value.contains("no promote-to-live button"))
    );
}
