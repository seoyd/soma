mod support;

use soma_zero::{Sprint91KrxEvidenceRecoveryRunner, WorkspaceGateRecoveryV8Status};
use support::sprint69_support as sprint;

#[test]
fn workspace_gate_recovery_v8_compares_previous_and_current_states() {
    let config = sprint::sprint91_config_from_example(
        "soma_workspace_gate_recovery_v8.toml",
        "krx-workspace-recovery-default",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_workspace_gate_recovery_v8(&config)
        .expect("report");
    assert_eq!(report.previous_no_run_status, "RealNoRunStillBlocked");
    assert_eq!(report.previous_full_status, "FullWorkspaceStillBlocked");
    assert_eq!(
        report.recovery_status,
        WorkspaceGateRecoveryV8Status::GateStillBlocked
    );
}

#[test]
fn workspace_gate_recovery_v8_can_mark_gate_improved_but_blocked() {
    let mut config = sprint::sprint91_config_from_example(
        "soma_workspace_gate_recovery_v8.toml",
        "krx-workspace-recovery-improved",
    );
    let assertion_path = sprint::write_support_json(
        "krx-workspace-recovery-improved",
        "krx_assertion_migration_expected.json",
        &serde_json::json!({
            "donor_files": ["tests/krx_collection_dry_run.rs"],
            "target_suite": "tests/krx_evidence_suite.rs",
            "high_risk_assertions_kept_separate": []
        }),
    );
    config
        .cargo_metadata_paths
        .retain(|value| !value.ends_with("krx_assertion_migration_expected.json"));
    config.cargo_metadata_paths.push(assertion_path);
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_workspace_gate_recovery_v8(&config)
        .expect("report");
    assert_eq!(
        report.recovery_status,
        WorkspaceGateRecoveryV8Status::GateImprovedButBlocked
    );
}
