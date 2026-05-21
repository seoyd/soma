mod support;

use soma_zero::{RealFullWorkspaceGateAttemptV9Status, Sprint91KrxEvidenceRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn real_full_workspace_gate_attempt_v9_threads_previous_status_by_default() {
    let config = sprint::sprint91_config_from_example(
        "soma_real_full_workspace_gate_attempt_v9.toml",
        "krx-real-full-default",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_real_full_workspace_gate_attempt_v9(&config)
        .expect("report");
    assert_eq!(
        report.full_status,
        RealFullWorkspaceGateAttemptV9Status::FullWorkspaceStillBlocked
    );
}

#[test]
fn real_full_workspace_gate_attempt_v9_maps_accepted_reruns() {
    let mut config = sprint::sprint91_config_from_example(
        "soma_real_full_workspace_gate_attempt_v9.toml",
        "krx-real-full-accepted",
    );
    config.run_real_full_after_reduction = true;
    let path = sprint::write_support_json(
        "krx-real-full-accepted",
        "krx_full_gate_rerun_sample.json",
        &serde_json::json!({
            "started": true,
            "finished": true,
            "passed": true,
            "duration_ms": 25,
            "remaining_blocker_families": []
        }),
    );
    config
        .cargo_metadata_paths
        .retain(|value| !value.ends_with("krx_full_gate_rerun_sample.json"));
    config.cargo_metadata_paths.push(path);
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_real_full_workspace_gate_attempt_v9(&config)
        .expect("report");
    assert_eq!(
        report.full_status,
        RealFullWorkspaceGateAttemptV9Status::FullWorkspaceAccepted
    );
}
