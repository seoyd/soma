mod support;

use soma_zero::{RealFullWorkspaceGateAttemptV6Status, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn real_full_workspace_gate_attempt_v6_requires_finished_passing_run() {
    let config = sprint::sprint88_config_from_example(
        "soma_real_full_workspace_gate_attempt_v6.toml",
        "real-full",
    );
    let report = Sprint88SevenBlockerRecoveryRunner::default()
        .run_real_full_workspace_gate_attempt_v6(&config)
        .expect("report");
    assert!(report.started);
    assert!(!report.finished);
    assert_eq!(
        report.full_status,
        RealFullWorkspaceGateAttemptV6Status::FullWorkspaceStillBlocked
    );
    assert_eq!(report.blocked_families.len(), 7);
}
