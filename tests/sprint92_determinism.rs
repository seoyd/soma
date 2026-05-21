mod support;

use soma_zero::Sprint92KrxWarningClosureRunner;
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn sprint92_bundle_is_deterministic_for_same_fixture() {
    let left = sprint::run_sprint92_bundle(
        "soma_sprint92_krx_warning_close.toml",
        "sprint92-determinism-left",
    );
    let right = sprint::run_sprint92_bundle(
        "soma_sprint92_krx_warning_close.toml",
        "sprint92-determinism-right",
    );
    assert_eq!(
        left.krx_evidence_warning_closure_report,
        right.krx_evidence_warning_closure_report
    );
    assert_eq!(
        left.dashboard_renderer_entry_gate,
        right.dashboard_renderer_entry_gate
    );
    harness::assert_deterministic_text(&left.final_summary, &right.final_summary);
}

#[test]
fn sprint92_runner_is_deterministic_for_same_config() {
    let config = sprint::sprint92_config_from_example(
        "soma_sprint92_krx_warning_close.toml",
        "sprint92-determinism-runner",
    );
    let first = Sprint92KrxWarningClosureRunner::default()
        .run(&config)
        .expect("first");
    let second = Sprint92KrxWarningClosureRunner::default()
        .run(&config)
        .expect("second");
    assert_eq!(
        first.remaining_blocker_queue_v8,
        second.remaining_blocker_queue_v8
    );
    assert_eq!(
        first.control_tower_krx_warning_closure_panel,
        second.control_tower_krx_warning_closure_panel
    );
}
