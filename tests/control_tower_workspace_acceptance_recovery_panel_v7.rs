mod support;

use support::sprint106_support::{read_fixture, run_sprint106};

#[test]
fn control_tower_panel_is_static_and_reports_open_workspace_truth() {
    let bundle = run_sprint106(
        "soma_control_tower_workspace_acceptance_recovery_v7.toml",
        "control_tower_workspace_acceptance_recovery_panel_v7",
    );
    let actual = serde_json::to_value(&bundle.control_tower_workspace_acceptance_recovery_panel_v7)
        .expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint106_data/control_tower_workspace_recovery_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .control_tower_workspace_acceptance_recovery_panel_v7
            .warnings
            .iter()
            .any(|warning| warning.contains("static/read-only"))
    );
}
