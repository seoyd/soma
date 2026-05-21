mod common;
mod support;

use soma_zero::ExperimentRunner;
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn same_valid_fixture_and_same_config_produce_same_bundle_summary() {
    let config = common::baseline_config("experiment-determinism-suite", "generic_ohlcv_valid.csv");
    let first = ExperimentRunner::default().run(&config);
    let second = ExperimentRunner::default().run(&config);
    assert_eq!(
        first.to_deterministic_summary(),
        second.to_deterministic_summary()
    );
}

#[test]
fn no_wall_clock_timestamp_appears_unless_passed_in_config() {
    let config = common::baseline_config(
        "experiment-determinism-suite-no-wallclock",
        "generic_ohlcv_valid.csv",
    );
    let bundle = ExperimentRunner::default().run(&config);
    assert_eq!(
        bundle.experiment_manifest.input_data_manifest.created_at_ms,
        None
    );
}

#[test]
fn sprint84_sprint85_and_sprint86_bundles_are_deterministic() {
    let sprint84_left = sprint::run_sprint84_bundle(
        "soma_sprint84_test_cost_reduce.toml",
        "experiment-determinism-sprint84-left",
    );
    let sprint84_right = sprint::run_sprint84_bundle(
        "soma_sprint84_test_cost_reduce.toml",
        "experiment-determinism-sprint84-right",
    );
    assert_eq!(
        sprint84_left.control_tower_test_cost_panel,
        sprint84_right.control_tower_test_cost_panel
    );

    let sprint85_left = sprint::run_sprint85_bundle(
        "soma_sprint85_workspace_gate_recovery.toml",
        "experiment-determinism-sprint85-left",
    );
    let sprint85_right = sprint::run_sprint85_bundle(
        "soma_sprint85_workspace_gate_recovery.toml",
        "experiment-determinism-sprint85-right",
    );
    assert_eq!(
        sprint85_left.control_tower_workspace_gate_panel_v2,
        sprint85_right.control_tower_workspace_gate_panel_v2
    );

    let sprint86_left = sprint::run_sprint86_bundle(
        "soma_sprint86_residual_gate_recover.toml",
        "experiment-determinism-sprint86-left",
    );
    let sprint86_right = sprint::run_sprint86_bundle(
        "soma_sprint86_residual_gate_recover.toml",
        "experiment-determinism-sprint86-right",
    );
    assert_eq!(
        sprint86_left.residual_workspace_binary_audit_report,
        sprint86_right.residual_workspace_binary_audit_report
    );
    assert_eq!(
        sprint86_left.control_tower_workspace_gate_panel_v3,
        sprint86_right.control_tower_workspace_gate_panel_v3
    );
    harness::assert_deterministic_text(&sprint86_left.final_summary, &sprint86_right.final_summary);
}
