mod support;

use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn sprint89_bundle_is_deterministic_for_same_fixture_inputs() {
    let first = sprint::run_sprint89_bundle(
        "soma_sprint89_candle_recover.toml",
        "sprint89-determinism-first",
    );
    let second = sprint::run_sprint89_bundle(
        "soma_sprint89_candle_recover.toml",
        "sprint89-determinism-second",
    );
    assert_eq!(
        first.candle_expansion_real_reduction_report,
        second.candle_expansion_real_reduction_report
    );
    assert_eq!(
        first.seven_blocker_queue_progress_report_v5,
        second.seven_blocker_queue_progress_report_v5
    );
    assert_eq!(
        first.workspace_gate_recovery_v6,
        second.workspace_gate_recovery_v6
    );
    assert_eq!(
        first.control_tower_candle_recovery_panel,
        second.control_tower_candle_recovery_panel
    );
    harness::assert_deterministic_text(&first.final_summary, &second.final_summary);
}
