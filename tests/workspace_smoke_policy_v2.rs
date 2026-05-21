mod support;

use soma_zero::{Sprint85WorkspaceGateRecoveryRunner, WorkspaceWideSmokePolicyV2Status};
use support::sprint69_support as sprint;

#[test]
fn workspace_smoke_policy_v2_covers_workspace_gate_and_safety() {
    let config = sprint::sprint85_config_from_example(
        "soma_workspace_smoke_policy_v2.toml",
        "workspace-smoke-policy-v2-test",
    );
    let report = Sprint85WorkspaceGateRecoveryRunner::default()
        .run_workspace_smoke_policy_v2(&config)
        .expect("smoke");
    assert_eq!(
        report.policy_status,
        WorkspaceWideSmokePolicyV2Status::WorkspaceSmokePolicyReady
    );
    assert!(
        report
            .command_family_coverage
            .get("workspace-gate")
            .expect("workspace-gate")
            .contains(&"workspace-acceptance-attempt-v3".to_string())
    );
}
