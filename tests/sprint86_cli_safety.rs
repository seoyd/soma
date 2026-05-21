use std::process::Command;

#[test]
fn sprint86_commands_are_listed_in_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint86-residual-gate-recover",
        "residual-binary-audit",
        "residual-family-classifier",
        "residual-consolidation-plan",
        "legacy-integration-migration",
        "compile-only-workspace-attempt",
        "cargo-test-no-run-gate",
        "full-workspace-attempt-v4",
        "full-gate-recovery-v4",
        "residual-blocker-drilldown-v2",
        "workspace-binary-delta-v2",
        "safety-coverage-preservation-v2",
        "control-tower-workspace-gate-v3",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
}

#[test]
fn sprint86_commands_reject_remote_paths() {
    for command in [
        "sprint86-residual-gate-recover",
        "residual-binary-audit",
        "residual-family-classifier",
        "residual-consolidation-plan",
        "legacy-integration-migration",
        "compile-only-workspace-attempt",
        "cargo-test-no-run-gate",
        "full-workspace-attempt-v4",
        "full-gate-recovery-v4",
        "residual-blocker-drilldown-v2",
        "workspace-binary-delta-v2",
        "safety-coverage-preservation-v2",
        "control-tower-workspace-gate-v3",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
