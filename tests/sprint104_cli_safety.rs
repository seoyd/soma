use std::process::Command;

#[test]
fn sprint104_commands_are_listed_and_help_text_is_safe() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint104-dual-agent-paper-lifecycle",
        "dual-agent-workflow-policy",
        "implementation-agent-role",
        "verification-agent-role",
        "prompt-compliance-verification",
        "safety-invariant-verification",
        "architecture-regression-verification",
        "test-coverage-verification",
        "final-verification-gate",
        "paper-batch-replay",
        "paper-candidate-lifecycle",
        "paper-candidate-promotion-gate",
        "paper-candidate-notrade-gate",
        "paper-candidate-riskdenied-gate",
        "risk-governor-batch-veto",
        "lower-confidence-carry-forward",
        "control-tower-dual-agent",
        "control-tower-paper-candidate-lifecycle",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
    for (command, needle) in [
        (
            "sprint104-dual-agent-paper-lifecycle",
            "dual-agent workflow",
        ),
        (
            "verification-agent-role",
            "verification is not full workspace acceptance",
        ),
        (
            "paper-candidate-lifecycle",
            "paper candidate is not an order",
        ),
        (
            "lower-confidence-carry-forward",
            "no silent confidence upgrade",
        ),
        ("control-tower-dual-agent", "static/read-only"),
        (
            "control-tower-paper-candidate-lifecycle",
            "no promote-to-live",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("subcommand help");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(needle), "missing {needle} for {command}");
    }
    for forbidden in [
        "training-command",
        "live-inference-command",
        "mamba-runtime-command",
        "gated-runtime-command",
        "broker-account-command",
    ] {
        assert!(
            !stdout.contains(forbidden),
            "unexpected command marker {forbidden}"
        );
    }
}

#[test]
fn sprint104_commands_reject_remote_paths() {
    for command in [
        "sprint104-dual-agent-paper-lifecycle",
        "dual-agent-workflow-policy",
        "implementation-agent-role",
        "verification-agent-role",
        "prompt-compliance-verification",
        "safety-invariant-verification",
        "architecture-regression-verification",
        "test-coverage-verification",
        "final-verification-gate",
        "paper-batch-replay",
        "paper-candidate-lifecycle",
        "paper-candidate-promotion-gate",
        "paper-candidate-notrade-gate",
        "paper-candidate-riskdenied-gate",
        "risk-governor-batch-veto",
        "lower-confidence-carry-forward",
        "control-tower-dual-agent",
        "control-tower-paper-candidate-lifecycle",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success(), "{command} should fail");
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
