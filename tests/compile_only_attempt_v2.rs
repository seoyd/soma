mod support;

use std::fs;

use soma_zero::{CompileOnlyAttemptV2Status, Sprint87CompileGateRecoveryRunner};
use support::sprint69_support as sprint;

fn config_with_no_run_sample(
    name: &str,
    json: &str,
) -> soma_zero::WorkspaceCompileGraphAuditConfig {
    let mut config =
        sprint::sprint87_config_from_example("soma_compile_only_attempt_v2.toml", name);
    let path = sprint::sprint87_output_dir(name).join("no_run_attempt_sample.json");
    fs::write(&path, json).expect("write no-run sample");
    config.compile_only_attempt_paths = vec![path.display().to_string()];
    config
}

#[test]
fn compile_only_attempt_v2_reports_blocked_status() {
    let config = sprint::sprint87_config_from_example(
        "soma_compile_only_attempt_v2.toml",
        "compile-only-blocked",
    );
    let report = Sprint87CompileGateRecoveryRunner::default()
        .run_compile_only_attempt_v2(&config)
        .expect("compile only");
    assert_eq!(
        report.compile_status,
        CompileOnlyAttemptV2Status::CompileOnlyStillBlocked
    );
}

#[test]
fn compile_only_attempt_v2_reports_passed_status() {
    let config = config_with_no_run_sample(
        "compile-only-passed",
        r#"{"started":true,"finished":true,"passed":true,"duration_ms":100,"blocked_families":[],"blocked_test_targets":[]}"#,
    );
    let report = Sprint87CompileGateRecoveryRunner::default()
        .run_compile_only_attempt_v2(&config)
        .expect("compile only");
    assert_eq!(
        report.compile_status,
        CompileOnlyAttemptV2Status::CompileOnlyPassed
    );
    assert_eq!(report.passed, Some(true));
}

#[test]
fn compile_only_attempt_v2_reports_failed_status() {
    let config = config_with_no_run_sample(
        "compile-only-failed",
        r#"{"started":true,"finished":true,"passed":false,"duration_ms":100,"blocked_families":["FutureWindow"],"blocked_test_targets":["tests/future_window_requirements.rs"]}"#,
    );
    let report = Sprint87CompileGateRecoveryRunner::default()
        .run_compile_only_attempt_v2(&config)
        .expect("compile only");
    assert_eq!(
        report.compile_status,
        CompileOnlyAttemptV2Status::CompileOnlyFailed
    );
    assert_eq!(report.passed, Some(false));
}
