mod support;

use soma_zero::WorkspaceAcceptanceFinalGateV2Status;
use support::sprint69_support as sprint;

#[test]
fn sprint84_workspace_final_gate_reports_blocked_honestly() {
    let bundle =
        sprint::run_sprint84_bundle("soma_workspace_final_gate_v2.toml", "sprint84-final-gate");
    let gate = bundle.workspace_acceptance_final_gate_v2;
    assert!(gate.fmt_passed);
    assert!(gate.check_passed);
    assert!(gate.focused_suite_passed);
    assert!(gate.representative_smoke_passed);
    assert!(gate.safety_smoke_passed);
    assert!(gate.full_workspace_started);
    assert!(!gate.full_workspace_finished);
    assert_eq!(
        gate.gate_status,
        WorkspaceAcceptanceFinalGateV2Status::FullWorkspaceStillBlocked
    );
    assert!(gate.safety_coverage_preserved);
}

#[test]
fn sprint84_focused_only_gate_never_becomes_full_acceptance() {
    let mut config = sprint::sprint84_config_from_example(
        "soma_workspace_final_gate_v2.toml",
        "sprint84-focused-only-gate",
    );
    let dir =
        support::shared_fixture_harness::temp_output_dir_for_test("sprint84-focused-only-json");
    let gate_json = dir.join("workspace_final_gate_expected.json");
    std::fs::write(
        &gate_json,
        serde_json::to_string_pretty(&serde_json::json!({
            "fmt_passed": true,
            "check_passed": true,
            "focused_suite_passed": true,
            "representative_smoke_passed": true,
            "safety_smoke_passed": true,
            "full_workspace_started": false,
            "full_workspace_finished": false,
            "full_workspace_passed": null,
            "full_workspace_duration_ms": null
        }))
        .unwrap(),
    )
    .unwrap();
    config
        .sprint83_recovery_paths
        .retain(|path| !path.ends_with("workspace_final_gate_expected.json"));
    config
        .sprint83_recovery_paths
        .push(gate_json.display().to_string());
    let gate = soma_zero::Sprint84TestCostReductionRunner::default()
        .run_workspace_final_gate_v2(&config)
        .expect("focused only gate");
    assert_eq!(
        gate.gate_status,
        WorkspaceAcceptanceFinalGateV2Status::FocusedOnly
    );
}
