mod support;

use soma_zero::{RealFullWorkspaceGateAttemptV8Status, Sprint90ExternalPredictionRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn real_full_workspace_gate_attempt_v8_keeps_previous_blocked_state_when_not_run() {
    let config = sprint::sprint90_config_from_example(
        "soma_real_full_workspace_gate_attempt_v8.toml",
        "real-full-workspace-gate-attempt-v8",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_real_full_workspace_gate_attempt_v8(&config)
        .expect("report");
    assert_eq!(
        report.full_status,
        RealFullWorkspaceGateAttemptV8Status::FullWorkspaceStillBlocked
    );
    assert!(!report.started);
    assert!(!report.finished);
    assert_eq!(report.command, "cargo test --workspace --quiet");
}
