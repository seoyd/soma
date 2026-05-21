mod support;

use std::fs;

use soma_zero::{FullWorkspaceAcceptanceAttemptV5Status, Sprint87CompileGateRecoveryRunner};
use support::sprint69_support as sprint;

fn config_with_samples(
    name: &str,
    no_run_json: &str,
    full_json: &str,
) -> soma_zero::WorkspaceCompileGraphAuditConfig {
    let mut config =
        sprint::sprint87_config_from_example("soma_full_workspace_attempt_v5.toml", name);
    let dir = sprint::sprint87_output_dir(name);
    let no_run_path = dir.join("no_run_attempt_sample.json");
    let full_path = dir.join("full_workspace_attempt_v5_expected.json");
    fs::write(&no_run_path, no_run_json).expect("write no-run");
    fs::write(&full_path, full_json).expect("write full");
    config.compile_only_attempt_paths = vec![no_run_path.display().to_string()];
    config.full_workspace_attempt_paths = vec![
        full_path.display().to_string(),
        sprint::example_path("sprint87_data/safety_coverage_v3_expected.json")
            .display()
            .to_string(),
    ];
    config
}

#[test]
fn full_workspace_attempt_v5_keeps_compile_only_blocked_distinct() {
    let config = sprint::sprint87_config_from_example(
        "soma_full_workspace_attempt_v5.toml",
        "full-workspace-attempt-blocked",
    );
    let report = Sprint87CompileGateRecoveryRunner::default()
        .run_full_workspace_attempt_v5(&config)
        .expect("attempt");
    assert_eq!(
        report.attempt_status,
        FullWorkspaceAcceptanceAttemptV5Status::CompileOnlyBlocked
    );
}

#[test]
fn full_workspace_attempt_v5_reports_still_blocked_after_compile_only_passes() {
    let config = config_with_samples(
        "full-workspace-attempt-still-blocked",
        r#"{"started":true,"finished":true,"passed":true,"duration_ms":100,"blocked_families":[],"blocked_test_targets":[]}"#,
        r#"{"fmt_passed":true,"check_passed":true,"broad_family_suites_passed":true,"safety_guard_suite_passed":true,"workspace_cli_safety_passed":true,"workspace_determinism_passed":true,"representative_smoke_passed":true,"safety_smoke_passed":true,"full_workspace_started":true,"full_workspace_finished":false,"full_workspace_blockers":["DashboardRenderer"]}"#,
    );
    let report = Sprint87CompileGateRecoveryRunner::default()
        .run_full_workspace_attempt_v5(&config)
        .expect("attempt");
    assert_eq!(
        report.attempt_status,
        FullWorkspaceAcceptanceAttemptV5Status::FullWorkspaceStillBlocked
    );
}

#[test]
fn full_workspace_attempt_v5_accepts_only_finished_passing_full_run() {
    let config = config_with_samples(
        "full-workspace-attempt-accepted",
        r#"{"started":true,"finished":true,"passed":true,"duration_ms":100,"blocked_families":[],"blocked_test_targets":[]}"#,
        r#"{"fmt_passed":true,"check_passed":true,"broad_family_suites_passed":true,"safety_guard_suite_passed":true,"workspace_cli_safety_passed":true,"workspace_determinism_passed":true,"representative_smoke_passed":true,"safety_smoke_passed":true,"full_workspace_started":true,"full_workspace_finished":true,"full_workspace_passed":true,"full_workspace_duration_ms":200,"full_workspace_blockers":[]}"#,
    );
    let report = Sprint87CompileGateRecoveryRunner::default()
        .run_full_workspace_attempt_v5(&config)
        .expect("attempt");
    assert_eq!(
        report.attempt_status,
        FullWorkspaceAcceptanceAttemptV5Status::FullWorkspaceAccepted
    );
}
