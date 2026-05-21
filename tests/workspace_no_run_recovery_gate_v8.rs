mod support;

use support::sprint107_support::run_sprint107;

#[test]
fn workspace_no_run_gate_keeps_not_run_separate() {
    let bundle = run_sprint107(
        "soma_workspace_no_run_recovery_gate_v8.toml",
        "workspace-no-run-recovery-gate-v8",
    );
    assert_eq!(
        bundle.post_patch_workspace_no_run_attempt_v23.no_run_status,
        "NotRun"
    );
    assert_eq!(
        bundle.workspace_no_run_recovery_gate_v8.gate_status,
        "NoRunNotRun"
    );
    assert!(!bundle.workspace_no_run_recovery_gate_v8.no_run_recovered);
}
