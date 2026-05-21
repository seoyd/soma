mod support;

use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn sprint85_bundle_is_deterministic_for_same_fixture_inputs() {
    let first = sprint::run_sprint85_bundle(
        "soma_sprint85_workspace_gate_recovery.toml",
        "sprint85-determinism-first",
    );
    let second = sprint::run_sprint85_bundle(
        "soma_sprint85_workspace_gate_recovery.toml",
        "sprint85-determinism-second",
    );
    assert_eq!(
        first.workspace_wide_test_surface_audit_report,
        second.workspace_wide_test_surface_audit_report
    );
    assert_eq!(
        first.domain_grouped_test_suite_report,
        second.domain_grouped_test_suite_report
    );
    assert_eq!(
        first.control_tower_workspace_gate_panel_v2,
        second.control_tower_workspace_gate_panel_v2
    );
    harness::assert_deterministic_text(&first.final_summary, &second.final_summary);
}
