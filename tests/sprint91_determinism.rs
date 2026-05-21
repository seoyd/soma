mod support;

use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn sprint91_bundle_is_deterministic_for_same_fixture_inputs() {
    let first = sprint::run_sprint91_bundle(
        "soma_sprint91_krx_evidence_recover.toml",
        "sprint91-determinism-first",
    );
    let second = sprint::run_sprint91_bundle(
        "soma_sprint91_krx_evidence_recover.toml",
        "sprint91-determinism-second",
    );
    assert_eq!(
        first.krx_evidence_real_reduction_report,
        second.krx_evidence_real_reduction_report
    );
    assert_eq!(
        first.seven_blocker_queue_progress_report_v7,
        second.seven_blocker_queue_progress_report_v7
    );
    assert_eq!(
        first.workspace_gate_recovery_v8,
        second.workspace_gate_recovery_v8
    );
    assert_eq!(
        first.control_tower_krx_evidence_recovery_panel,
        second.control_tower_krx_evidence_recovery_panel
    );
    harness::assert_deterministic_text(&first.final_summary, &second.final_summary);
}
