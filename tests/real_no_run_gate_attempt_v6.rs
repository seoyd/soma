mod support;

use soma_zero::{RealNoRunGateAttemptV6Status, Sprint91KrxEvidenceRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn real_no_run_gate_attempt_v6_threads_previous_status_by_default() {
    let config = sprint::sprint91_config_from_example(
        "soma_real_no_run_gate_attempt_v6.toml",
        "krx-real-no-run-default",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_real_no_run_gate_attempt_v6(&config)
        .expect("report");
    assert_eq!(
        report.no_run_status,
        RealNoRunGateAttemptV6Status::RealNoRunStillBlocked
    );
}

#[test]
fn real_no_run_gate_attempt_v6_maps_recovered_reruns() {
    let mut config = sprint::sprint91_config_from_example(
        "soma_real_no_run_gate_attempt_v6.toml",
        "krx-real-no-run-recovered",
    );
    config.run_real_no_run_after_reduction = true;
    let path = sprint::write_support_json(
        "krx-real-no-run-recovered",
        "krx_no_run_rerun_sample.json",
        &serde_json::json!({
            "started": true,
            "finished": true,
            "passed": true,
            "duration_ms": 10,
            "remaining_blocker_families": []
        }),
    );
    config
        .cargo_metadata_paths
        .retain(|value| !value.ends_with("krx_no_run_rerun_sample.json"));
    config.cargo_metadata_paths.push(path);
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_real_no_run_gate_attempt_v6(&config)
        .expect("report");
    assert_eq!(
        report.no_run_status,
        RealNoRunGateAttemptV6Status::RealNoRunPassed
    );
}
