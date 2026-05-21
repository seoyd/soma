use std::process::Command;

#[test]
fn sprint85_commands_are_listed_in_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint85-workspace-gate-recover",
        "workspace-test-surface-audit",
        "remaining-test-binary-inventory",
        "domain-suite-plan",
        "shared-fixture-adoption",
        "workspace-smoke-policy-v2",
        "workspace-acceptance-attempt-v3",
        "full-gate-recovery-v3",
        "workspace-blocker-drilldown",
        "control-tower-workspace-gate-v2",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
}

#[test]
fn sprint85_commands_reject_remote_paths() {
    for command in [
        "sprint85-workspace-gate-recover",
        "workspace-test-surface-audit",
        "remaining-test-binary-inventory",
        "domain-suite-plan",
        "shared-fixture-adoption",
        "workspace-smoke-policy-v2",
        "workspace-acceptance-attempt-v3",
        "full-gate-recovery-v3",
        "workspace-blocker-drilldown",
        "control-tower-workspace-gate-v2",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
