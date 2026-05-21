mod support;

use std::fs;

use soma_zero::{NoRunAcceptanceGateV2Status, Sprint87CompileGateRecoveryRunner};
use support::sprint69_support as sprint;

fn config_with_no_run_sample(
    name: &str,
    json: &str,
) -> soma_zero::WorkspaceCompileGraphAuditConfig {
    let mut config =
        sprint::sprint87_config_from_example("soma_no_run_acceptance_gate_v2.toml", name);
    let path = sprint::sprint87_output_dir(name).join("no_run_attempt_sample.json");
    fs::write(&path, json).expect("write no-run sample");
    config.compile_only_attempt_paths = vec![path.display().to_string()];
    config
}

#[test]
fn no_run_acceptance_gate_v2_reports_still_blocked() {
    let config = sprint::sprint87_config_from_example(
        "soma_no_run_acceptance_gate_v2.toml",
        "no-run-gate-blocked",
    );
    let report = Sprint87CompileGateRecoveryRunner::default()
        .run_no_run_acceptance_gate_v2(&config)
        .expect("no-run");
    assert_eq!(
        report.no_run_gate_status,
        NoRunAcceptanceGateV2Status::NoRunGateStillBlocked
    );
}

#[test]
fn no_run_acceptance_gate_v2_reports_passed_when_compile_only_finishes() {
    let config = config_with_no_run_sample(
        "no-run-gate-passed",
        r#"{"started":true,"finished":true,"passed":true,"duration_ms":100,"blocked_families":[],"blocked_test_targets":[]}"#,
    );
    let report = Sprint87CompileGateRecoveryRunner::default()
        .run_no_run_acceptance_gate_v2(&config)
        .expect("no-run");
    assert_eq!(
        report.no_run_gate_status,
        NoRunAcceptanceGateV2Status::NoRunGatePassed
    );
}
