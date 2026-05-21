mod support;

use soma_zero::{CompileOnlyWorkspaceCompileStatus, Sprint86ResidualGateRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn compile_only_workspace_attempt_reports_passed_compile_state() {
    let config = sprint::sprint86_config_from_example(
        "soma_compile_only_workspace_attempt.toml",
        "compile-only-workspace-attempt-test",
    );
    let report = Sprint86ResidualGateRecoveryRunner::default()
        .run_compile_only_workspace_attempt(&config)
        .expect("compile only");
    assert_eq!(
        report.compile_status,
        CompileOnlyWorkspaceCompileStatus::CompileOnlyPassed
    );
    assert!(report.started);
    assert!(report.finished);
    assert_eq!(report.passed, Some(true));
}
