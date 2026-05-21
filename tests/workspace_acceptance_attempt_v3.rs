mod support;

use soma_zero::{Sprint85WorkspaceGateRecoveryRunner, WorkspaceWideAcceptanceAttemptV3Status};
use support::sprint69_support as sprint;

#[test]
fn workspace_acceptance_attempt_v3_keeps_blocked_state_honest() {
    let config = sprint::sprint85_config_from_example(
        "soma_workspace_acceptance_attempt_v3.toml",
        "workspace-acceptance-attempt-v3-test",
    );
    let report = Sprint85WorkspaceGateRecoveryRunner::default()
        .run_workspace_acceptance_attempt_v3(&config)
        .expect("attempt");
    assert_eq!(
        report.attempt_status,
        WorkspaceWideAcceptanceAttemptV3Status::FullWorkspaceStillBlocked
    );
    assert!(report.fmt_passed);
    assert!(report.check_passed);
    assert!(report.domain_suites_passed);
    assert!(report.full_workspace_started);
    assert!(!report.full_workspace_finished);
}
