mod support;

use serde_json::Value;
use soma_zero::{
    CommandObservationSnapshot, build_real_cargo_json_progress_observation_v1_from_stdout,
    build_real_nextest_probe_execution_report_v1, build_real_sccache_probe_execution_report_v1,
};
use support::sprint113_support::{read_fixture, run_sprint113};

#[test]
fn cargo_json_progress_parses_actual_json_and_counts_errors() {
    let stdout = r#"{"reason":"compiler-message","message":{"message":"warn"}}
{"reason":"compiler-artifact","target":{"src_path":"tests/workspace_timeout_root_cause.rs"},"filenames":["target/debug/deps/workspace_timeout_root_cause"]}
not-json
{"reason":"compiler-artifact","target":{"src_path":"tests/control_tower_workspace_timeout_root_cause_panel.rs"},"filenames":["target/debug/deps/control_tower_workspace_timeout_root_cause_panel"]}"#;
    let report =
        build_real_cargo_json_progress_observation_v1_from_stdout(true, true, false, stdout);
    assert_eq!(report.parsed_json_message_count, 3);
    assert_eq!(report.parse_error_count, 1);
    assert_eq!(report.artifact_count, 2);
    assert_eq!(report.compiler_message_count, 1);
    assert!(
        report
            .last_seen_targets
            .contains(&"tests/workspace_timeout_root_cause.rs".to_string())
    );

    let bundle = run_sprint113(
        "soma_real_cargo_json_progress_observation_v1.toml",
        "real-cargo-json-progress-observation-v1",
    );
    let expected: Value = read_fixture("sprint113_data/real_cargo_json_progress_expected.json");
    assert_eq!(
        bundle
            .real_cargo_json_progress_observation_v1
            .observation_status,
        expected["observation_status"].as_str().unwrap()
    );
    assert_eq!(
        bundle
            .real_cargo_json_progress_observation_v1
            .parsed_json_message_count,
        0
    );
    assert_eq!(
        bundle.acceptance_truth_gate_v14.cargo_json_truth_status,
        "Insufficient"
    );
}

#[test]
fn nextest_and_sccache_probe_reports_use_actual_snapshot_result() {
    let not_run = CommandObservationSnapshot::default();
    let nextest_not_run = build_real_nextest_probe_execution_report_v1(&not_run);
    assert!(!nextest_not_run.attempted);
    assert!(!nextest_not_run.nextest_available);
    assert_eq!(nextest_not_run.exit_code, None);
    assert_eq!(nextest_not_run.probe_status, "NextestProbeNotRun");

    let sccache_success =
        build_real_sccache_probe_execution_report_v1(&CommandObservationSnapshot {
            attempted: true,
            finished: true,
            exit_code: Some(0),
            stdout: "sccache 0.8.1\n".to_string(),
            ..CommandObservationSnapshot::default()
        });
    assert!(sccache_success.sccache_available);
    assert_eq!(sccache_success.version.as_deref(), Some("sccache 0.8.1"));
    assert_eq!(sccache_success.probe_status, "SccacheProbeSucceeded");
}
