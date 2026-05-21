use std::process::Command;

#[test]
fn sprint99_commands_are_listed_and_help_text_is_safe() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint99-committee-quality-harden",
        "committee-member-proposal-quality",
        "entry-timing-proposal-quality",
        "committee-debate-quality",
        "debate-evidence-sufficiency",
        "chairman-rulebook-quality",
        "chairman-rule-risk-audit-v2",
        "rulebook-version-diff",
        "promotion-demotion-calibration",
        "member-scorecard-calibration",
        "member-overfit-risk",
        "member-style-drift",
        "investor-style-blindspot",
        "committee-roster-balance",
        "paper-only-decision-replay",
        "paper-decision-trace-completeness",
        "risk-governor-debate-handoff",
        "committee-architecture-regression-guard",
        "workspace-acceptance-truth-closure-plan",
        "workspace-acceptance-attempt-v16",
        "safety-coverage-preservation-v15",
        "control-tower-ai-committee-quality",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
    for (command, needle) in [
        ("sprint99-committee-quality-harden", "paper-only"),
        ("committee-member-proposal-quality", "not order"),
        ("chairman-rulebook-quality", "no live rule mutation"),
        ("promotion-demotion-calibration", "not capital allocation"),
        (
            "workspace-acceptance-truth-closure-plan",
            "full workspace acceptance",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("subcommand help");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(needle), "missing {needle} for {command}");
    }
    assert!(!stdout.contains("sprint99-training"));
    assert!(!stdout.contains("sprint99-live-inference"));
    assert!(!stdout.contains("sprint99-mamba-runtime"));
    assert!(!stdout.contains("sprint99-gated-runtime"));
}

#[test]
fn sprint99_commands_reject_remote_paths() {
    for command in [
        "sprint99-committee-quality-harden",
        "committee-member-proposal-quality",
        "entry-timing-proposal-quality",
        "committee-debate-quality",
        "debate-evidence-sufficiency",
        "chairman-rulebook-quality",
        "chairman-rule-risk-audit-v2",
        "rulebook-version-diff",
        "promotion-demotion-calibration",
        "member-scorecard-calibration",
        "member-overfit-risk",
        "member-style-drift",
        "investor-style-blindspot",
        "committee-roster-balance",
        "paper-only-decision-replay",
        "paper-decision-trace-completeness",
        "risk-governor-debate-handoff",
        "committee-architecture-regression-guard",
        "workspace-acceptance-truth-closure-plan",
        "workspace-acceptance-attempt-v16",
        "safety-coverage-preservation-v15",
        "control-tower-ai-committee-quality",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success(), "{command} should fail");
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
