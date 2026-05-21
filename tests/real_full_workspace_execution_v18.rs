mod support;

use soma_zero::{CommandObservation, build_real_full_workspace_execution_report_v18};
use support::sprint117_support::run_sprint117;

#[test]
fn real_full_workspace_execution_accepts_only_finished_pass() {
    let bundle = run_sprint117(
        "soma_real_full_workspace_execution_v18.toml",
        "real-full-workspace-execution-v18",
    );
    assert_eq!(
        bundle
            .real_full_workspace_execution_report_v18
            .execution_status,
        "RealFullWorkspaceDeferred"
    );
    let accepted = build_real_full_workspace_execution_report_v18(
        Some(&CommandObservation {
            attempted: true,
            finished: true,
            passed: Some(true),
            duration_ms: Some(1000),
            timeout_ms: Some(420000),
            exit_code: Some(0),
            timed_out: false,
            stdout: String::new(),
        }),
        Some(420000),
        Some((0, 0)),
    );
    assert!(accepted.full_workspace_accepted);
}
