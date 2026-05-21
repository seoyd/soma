use std::process::Command;

#[test]
fn sprint102_commands_are_listed_and_help_text_is_safe() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint102-paper-rotation",
        "paper-rotation-scenario-pack",
        "lower-confidence-evidence-hardening",
        "paper-member-proposal-run",
        "group-debate-session",
        "risk-governor-paper-handoff",
        "paper-decision-trace-v2",
        "control-tower-paper-rotation",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
    for (command, needle) in [
        ("sprint102-paper-rotation", "paper-only"),
        ("lower-confidence-evidence-hardening", "Wonyotti"),
        ("paper-member-proposal-run", "not an order"),
        ("group-debate-session", "paper-only debate"),
        ("risk-governor-paper-handoff", "final veto"),
        ("paper-decision-trace-v2", "no broker/live execution"),
        ("control-tower-paper-rotation", "read-only"),
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
fn sprint102_commands_reject_remote_paths() {
    for command in [
        "sprint102-paper-rotation",
        "paper-rotation-scenario-pack",
        "paper-rotation-market-context",
        "archetype-group-rotation-plan",
        "archetype-member-selection",
        "lower-confidence-evidence-hardening",
        "weak-source-candidate-review",
        "wonyotti-evidence-hardening",
        "larry-williams-evidence-hardening",
        "arthur-hayes-evidence-hardening",
        "paper-member-proposal-run",
        "paper-entry-timing-run",
        "group-debate-trigger",
        "group-debate-session",
        "cross-group-debate-conflict",
        "chairman-synthesis-dry-run",
        "chairman-style-weight-audit",
        "risk-governor-paper-handoff",
        "paper-decision-trace-v2",
        "paper-decision-replay-v2",
        "proposal-expectation-trace",
        "notrade-riskdenied-committee-trace",
        "regime-routed-dry-run",
        "multi-expert-rotation-coverage",
        "paper-roster-expansion-usage",
        "eighteen-activation-safety",
        "workspace-truth-closure-plan-v3",
        "workspace-acceptance-attempt-v18",
        "safety-coverage-preservation-v18",
        "control-tower-paper-rotation",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success(), "{command} should fail");
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
