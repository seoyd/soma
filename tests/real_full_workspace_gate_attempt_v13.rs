mod support;

use soma_zero::{RealFullWorkspaceGateAttemptV13Status, Sprint95CommitteeCliSafetyRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn real_full_workspace_gate_attempt_v13_is_not_run_by_default() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_real_full_workspace_gate_attempt_v13(&sprint::sprint95_config_from_example(
            "soma_real_full_workspace_gate_attempt_v13.toml",
            "real-full-workspace-gate-attempt-v13",
        ))
        .expect("report");
    assert_eq!(
        report.full_status,
        RealFullWorkspaceGateAttemptV13Status::NotRun
    );
}
