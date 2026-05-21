mod support;

use soma_zero::{RealNoRunGateAttemptV5Status, Sprint90ExternalPredictionRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn real_no_run_gate_attempt_v5_keeps_previous_blocked_state_when_not_run() {
    let config = sprint::sprint90_config_from_example(
        "soma_real_no_run_gate_attempt_v5.toml",
        "real-no-run-gate-attempt-v5",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_real_no_run_gate_attempt_v5(&config)
        .expect("report");
    assert_eq!(
        report.no_run_status,
        RealNoRunGateAttemptV5Status::RealNoRunStillBlocked
    );
    assert!(!report.started);
    assert!(!report.finished);
    assert_eq!(report.blocked_families.len(), 5);
}
