mod support;

use soma_zero::{RealFullWorkspaceGateAttemptV7Status, Sprint89CandleRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn real_full_workspace_gate_attempt_v7_carries_forward_blocked_state_honestly() {
    let config = sprint::sprint89_config_from_example(
        "soma_real_full_workspace_gate_attempt_v7.toml",
        "real-full-v7",
    );
    let report = Sprint89CandleRecoveryRunner::default()
        .run_real_full_workspace_gate_attempt_v7(&config)
        .expect("report");
    assert_eq!(
        report.full_status,
        RealFullWorkspaceGateAttemptV7Status::FullWorkspaceStillBlocked
    );
    assert_eq!(report.blocked_families.len(), 6);
    assert!(!report.started);
}
