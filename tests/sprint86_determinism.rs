mod support;

use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn sprint86_bundle_is_deterministic_for_same_fixture_inputs() {
    let first = sprint::run_sprint86_bundle(
        "soma_sprint86_residual_gate_recover.toml",
        "sprint86-determinism-first",
    );
    let second = sprint::run_sprint86_bundle(
        "soma_sprint86_residual_gate_recover.toml",
        "sprint86-determinism-second",
    );
    assert_eq!(
        first.residual_workspace_binary_audit_report,
        second.residual_workspace_binary_audit_report
    );
    assert_eq!(
        first.residual_integration_family_classifier,
        second.residual_integration_family_classifier
    );
    assert_eq!(
        first.control_tower_workspace_gate_panel_v3,
        second.control_tower_workspace_gate_panel_v3
    );
    harness::assert_deterministic_text(&first.final_summary, &second.final_summary);
}
