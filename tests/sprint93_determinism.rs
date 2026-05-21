mod support;

use soma_zero::Sprint93TimeoutAttributionRunner;
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn sprint93_bundle_is_deterministic_for_same_fixture() {
    let left = sprint::run_sprint93_bundle(
        "soma_sprint93_timeout_attribution.toml",
        "sprint93-determinism-left",
    );
    let right = sprint::run_sprint93_bundle(
        "soma_sprint93_timeout_attribution.toml",
        "sprint93-determinism-right",
    );
    assert_eq!(
        left.dashboard_renderer_entry_release_gate,
        right.dashboard_renderer_entry_release_gate
    );
    assert_eq!(
        left.control_tower_timeout_attribution_panel,
        right.control_tower_timeout_attribution_panel
    );
    harness::assert_deterministic_text(&left.final_summary, &right.final_summary);
}

#[test]
fn sprint93_runner_is_deterministic_for_same_config() {
    let config = sprint::sprint93_config_from_example(
        "soma_sprint93_timeout_attribution.toml",
        "sprint93-determinism-runner",
    );
    let first = Sprint93TimeoutAttributionRunner::default()
        .run(&config)
        .expect("first");
    let second = Sprint93TimeoutAttributionRunner::default()
        .run(&config)
        .expect("second");
    assert_eq!(
        first.remaining_blocker_queue_v9,
        second.remaining_blocker_queue_v9
    );
    assert_eq!(
        first.cargo_target_progress_timeline,
        second.cargo_target_progress_timeline
    );
}
