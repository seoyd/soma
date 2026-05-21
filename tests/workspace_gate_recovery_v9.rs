mod support;

use std::fs;

use serde_json::json;
use soma_zero::{
    KrxEvidenceWarningClosureConfig, Sprint92KrxWarningClosureRunner, WorkspaceGateRecoveryV9Status,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_workspace_gate_recovery_v9.toml", name)
}

#[test]
fn workspace_recovery_matches_expected_fixture() {
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_workspace_gate_recovery_v9(&config("workspace-recovery-default"))
        .expect("report");
    let expected = harness::load_json_fixture(sprint::example_path(
        "sprint92_data/workspace_gate_recovery_v9_expected.json",
    ));
    assert_eq!(report, expected);
    assert_eq!(
        report.recovery_status,
        WorkspaceGateRecoveryV9Status::GateStillBlocked
    );
}

#[test]
fn workspace_recovery_can_improve_but_remain_blocked() {
    let mut config = config("workspace-recovery-improved");
    let mut summary = harness::load_json_fixture::<serde_json::Value>(sprint::example_path(
        "sprint92_data/sprint91_summary.json",
    ));
    summary["previous_no_run_status"] = json!("NotRun");
    let dir = harness::temp_output_dir_for_test("workspace-recovery-improved");
    let summary_path = dir.join("sprint91_summary.json");
    fs::write(
        &summary_path,
        serde_json::to_string_pretty(&summary).expect("json"),
    )
    .expect("write");
    let previous_bundle = config.sprint91_bundle_paths[0].clone();
    config.sprint91_bundle_paths = vec![summary_path.display().to_string(), previous_bundle];
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_workspace_gate_recovery_v9(&config)
        .expect("report");
    assert_eq!(
        report.recovery_status,
        WorkspaceGateRecoveryV9Status::GateImprovedButBlocked
    );
}
