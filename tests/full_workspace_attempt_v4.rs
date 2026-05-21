mod support;

use soma_zero::{FullWorkspaceAcceptanceAttemptV4Status, Sprint86ResidualGateRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn full_workspace_attempt_v4_stays_blocked_when_full_run_does_not_finish() {
    let config = sprint::sprint86_config_from_example(
        "soma_full_workspace_attempt_v4.toml",
        "full-workspace-attempt-v4-test",
    );
    let report = Sprint86ResidualGateRecoveryRunner::default()
        .run_full_workspace_attempt_v4(&config)
        .expect("full attempt");
    assert_eq!(
        report.attempt_status,
        FullWorkspaceAcceptanceAttemptV4Status::FullWorkspaceStillBlocked
    );
    assert!(report.compile_only_passed == Some(true));
    assert!(report.full_workspace_started);
    assert!(!report.full_workspace_finished);
}
