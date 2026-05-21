mod support;

use std::fs;

use serde_json::json;
use soma_zero::{
    KrxEvidenceWarningClosureConfig, RealFullWorkspaceGateAttemptV10Status,
    Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_real_full_workspace_gate_attempt_v10.toml", name)
}

#[test]
fn real_full_attempt_defaults_to_still_blocked() {
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_real_full_workspace_gate_attempt_v10(&config("real-full-default"))
        .expect("report");
    assert_eq!(
        report.full_status,
        RealFullWorkspaceGateAttemptV10Status::FullWorkspaceStillBlocked
    );
    assert!(report.started);
    assert!(!report.finished);
}

#[test]
fn real_full_attempt_accepts_only_finished_passing_runs() {
    let mut config = config("real-full-pass");
    let mut summary = harness::load_json_fixture::<serde_json::Value>(sprint::example_path(
        "sprint92_data/sprint91_summary.json",
    ));
    summary["previous_full_status"] = json!("FullWorkspaceAccepted");
    summary["full_timeout_ms"] = json!(52);
    let dir = harness::temp_output_dir_for_test("real-full-pass");
    let summary_path = dir.join("sprint91_summary.json");
    fs::write(
        &summary_path,
        serde_json::to_string_pretty(&summary).expect("json"),
    )
    .expect("write");
    config.sprint91_bundle_paths = vec![summary_path.display().to_string()];
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_real_full_workspace_gate_attempt_v10(&config)
        .expect("report");
    assert_eq!(
        report.full_status,
        RealFullWorkspaceGateAttemptV10Status::FullWorkspaceAccepted
    );
    assert_eq!(report.passed, Some(true));
    assert!(report.finished);
}
