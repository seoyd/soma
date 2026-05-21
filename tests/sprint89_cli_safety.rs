use std::process::Command;

#[test]
fn sprint89_commands_are_listed_in_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint89-candle-recover",
        "candle-real-reduction-plan",
        "candle-assertion-migration",
        "candle-fixture-setup-reduction",
        "candle-compile-impact",
        "candle-no-run-rerun",
        "candle-full-gate-rerun",
        "seven-blocker-queue-progress-v5",
        "measured-target-delta-v5",
        "real-no-run-gate-attempt-v4",
        "real-full-workspace-gate-attempt-v7",
        "workspace-gate-recovery-v6",
        "remaining-blocker-queue-v5",
        "safety-coverage-preservation-v5",
        "control-tower-candle-recovery",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
}

#[test]
fn sprint89_commands_reject_remote_paths() {
    for command in [
        "sprint89-candle-recover",
        "candle-real-reduction-plan",
        "candle-assertion-migration",
        "candle-fixture-setup-reduction",
        "candle-compile-impact",
        "candle-no-run-rerun",
        "candle-full-gate-rerun",
        "seven-blocker-queue-progress-v5",
        "measured-target-delta-v5",
        "real-no-run-gate-attempt-v4",
        "real-full-workspace-gate-attempt-v7",
        "workspace-gate-recovery-v6",
        "remaining-blocker-queue-v5",
        "safety-coverage-preservation-v5",
        "control-tower-candle-recovery",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
