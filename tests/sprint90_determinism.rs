mod support;

use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn sprint90_bundle_is_deterministic_for_same_fixture_inputs() {
    let first = sprint::run_sprint90_bundle(
        "soma_sprint90_external_prediction_recover.toml",
        "sprint90-determinism-first",
    );
    let second = sprint::run_sprint90_bundle(
        "soma_sprint90_external_prediction_recover.toml",
        "sprint90-determinism-second",
    );
    assert_eq!(
        first.external_prediction_real_reduction_report,
        second.external_prediction_real_reduction_report
    );
    assert_eq!(
        first.seven_blocker_queue_progress_report_v6,
        second.seven_blocker_queue_progress_report_v6
    );
    assert_eq!(
        first.workspace_gate_recovery_v7,
        second.workspace_gate_recovery_v7
    );
    assert_eq!(
        first.control_tower_external_prediction_recovery_panel,
        second.control_tower_external_prediction_recovery_panel
    );
    harness::assert_deterministic_text(&first.final_summary, &second.final_summary);
}
