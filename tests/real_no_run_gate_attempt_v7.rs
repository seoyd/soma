mod support;

use std::fs;

use serde_json::json;
use soma_zero::{
    KrxEvidenceWarningClosureConfig, RealNoRunGateAttemptV7Status, Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_real_no_run_gate_attempt_v7.toml", name)
}

#[test]
fn real_no_run_attempt_defaults_to_still_blocked() {
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_real_no_run_gate_attempt_v7(&config("real-no-run-default"))
        .expect("report");
    assert_eq!(
        report.no_run_status,
        RealNoRunGateAttemptV7Status::RealNoRunStillBlocked
    );
    assert!(report.started);
    assert!(!report.finished);
}

#[test]
fn real_no_run_attempt_can_pass_and_is_deterministic() {
    let mut config = config("real-no-run-pass");
    let mut summary = harness::load_json_fixture::<serde_json::Value>(sprint::example_path(
        "sprint92_data/sprint91_summary.json",
    ));
    summary["previous_no_run_status"] = json!("RealNoRunPassed");
    summary["no_run_timeout_ms"] = json!(42);
    let dir = harness::temp_output_dir_for_test("real-no-run-pass");
    let summary_path = dir.join("sprint91_summary.json");
    fs::write(
        &summary_path,
        serde_json::to_string_pretty(&summary).expect("json"),
    )
    .expect("write");
    config.sprint91_bundle_paths = vec![summary_path.display().to_string()];
    let first = Sprint92KrxWarningClosureRunner::default()
        .run_real_no_run_gate_attempt_v7(&config)
        .expect("first");
    let second = Sprint92KrxWarningClosureRunner::default()
        .run_real_no_run_gate_attempt_v7(&config)
        .expect("second");
    assert_eq!(first, second);
    assert_eq!(
        first.no_run_status,
        RealNoRunGateAttemptV7Status::RealNoRunPassed
    );
    assert_eq!(first.passed, Some(true));
}
