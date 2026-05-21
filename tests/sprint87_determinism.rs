mod support;

use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn sprint87_bundle_is_deterministic_for_same_fixture_inputs() {
    let first = sprint::run_sprint87_bundle(
        "soma_sprint87_compile_gate_recover.toml",
        "sprint87-determinism-first",
    );
    let second = sprint::run_sprint87_bundle(
        "soma_sprint87_compile_gate_recover.toml",
        "sprint87-determinism-second",
    );
    assert_eq!(
        first.workspace_compile_graph_audit_report,
        second.workspace_compile_graph_audit_report
    );
    assert_eq!(
        first.remaining_compile_family_classifier_v2,
        second.remaining_compile_family_classifier_v2
    );
    assert_eq!(
        first.control_tower_compile_gate_panel_v4,
        second.control_tower_compile_gate_panel_v4
    );
    harness::assert_deterministic_text(&first.final_summary, &second.final_summary);
}
