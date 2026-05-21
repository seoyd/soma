use std::process::Command;

#[test]
fn sprint87_commands_are_listed_in_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint87-compile-gate-recover",
        "workspace-compile-graph-audit",
        "test-target-fanout",
        "dev-dependency-fanout",
        "feature-unification-audit",
        "compile-family-classifier-v2",
        "compile-heavy-consolidation-plan",
        "compile-only-attempt-v2",
        "no-run-acceptance-gate-v2",
        "full-workspace-attempt-v5",
        "compile-gate-recovery",
        "compile-blocker-drilldown-v3",
        "test-target-delta-v3",
        "safety-coverage-preservation-v3",
        "control-tower-compile-gate-v4",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
}

#[test]
fn sprint87_commands_reject_remote_paths() {
    for command in [
        "sprint87-compile-gate-recover",
        "workspace-compile-graph-audit",
        "test-target-fanout",
        "dev-dependency-fanout",
        "feature-unification-audit",
        "compile-family-classifier-v2",
        "compile-heavy-consolidation-plan",
        "compile-only-attempt-v2",
        "no-run-acceptance-gate-v2",
        "full-workspace-attempt-v5",
        "compile-gate-recovery",
        "compile-blocker-drilldown-v3",
        "test-target-delta-v3",
        "safety-coverage-preservation-v3",
        "control-tower-compile-gate-v4",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
