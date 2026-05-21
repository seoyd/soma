use std::process::Command;

#[test]
fn sprint100_commands_are_listed_and_help_text_is_safe() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint100-committee-closure",
        "proposal-warning-closure",
        "proposal-evidence-completeness",
        "proposal-risk-field-completeness",
        "entry-timing-condition-completeness",
        "debate-evidence-closure",
        "debate-evidence-gap-plan",
        "debate-dissent-coverage",
        "debate-participation-balance",
        "chairman-unsafe-rule-closure",
        "chairman-rulebook-repair-plan",
        "chairman-rulebook-v2-draft",
        "chairman-rulebook-approval-gate",
        "rule-audit-trail-completeness",
        "rulebook-diff-risk-closure",
        "scorecard-warning-closure",
        "scorecard-evidence-depth",
        "promotion-demotion-stability",
        "overfit-warning-closure",
        "roster-balance-warning-closure",
        "paper-replay-warning-closure",
        "paper-need-more-evidence-closure",
        "risk-handoff-warning-closure",
        "risk-final-veto-trace",
        "committee-paper-readiness-gate",
        "committee-paper-loop-dry-run-plan",
        "workspace-truth-closure-plan-v2",
        "workspace-acceptance-attempt-v17",
        "safety-coverage-preservation-v16",
        "control-tower-ai-committee-closure",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
    for (command, needle) in [
        ("sprint100-committee-closure", "paper-only"),
        ("proposal-warning-closure", "not order"),
        ("chairman-rulebook-repair-plan", "no central AI core"),
        (
            "committee-paper-readiness-gate",
            "does not imply broker execution",
        ),
        (
            "workspace-truth-closure-plan-v2",
            "full workspace acceptance",
        ),
        (
            "control-tower-ai-committee-closure",
            "no train/runtime/live/order/account/browser controls",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("subcommand help");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(needle), "missing {needle} for {command}");
    }
    assert!(!stdout.contains("sprint100-training"));
    assert!(!stdout.contains("sprint100-live-trading"));
}

#[test]
fn sprint100_commands_reject_remote_paths() {
    for command in [
        "sprint100-committee-closure",
        "proposal-warning-closure",
        "proposal-evidence-completeness",
        "proposal-risk-field-completeness",
        "entry-timing-condition-completeness",
        "debate-evidence-closure",
        "debate-evidence-gap-plan",
        "debate-dissent-coverage",
        "debate-participation-balance",
        "chairman-unsafe-rule-closure",
        "chairman-rulebook-repair-plan",
        "chairman-rulebook-v2-draft",
        "chairman-rulebook-approval-gate",
        "rule-audit-trail-completeness",
        "rulebook-diff-risk-closure",
        "scorecard-warning-closure",
        "scorecard-evidence-depth",
        "promotion-demotion-stability",
        "overfit-warning-closure",
        "roster-balance-warning-closure",
        "paper-replay-warning-closure",
        "paper-need-more-evidence-closure",
        "risk-handoff-warning-closure",
        "risk-final-veto-trace",
        "committee-paper-readiness-gate",
        "committee-paper-loop-dry-run-plan",
        "workspace-truth-closure-plan-v2",
        "workspace-acceptance-attempt-v17",
        "safety-coverage-preservation-v16",
        "control-tower-ai-committee-closure",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success(), "{command} should fail");
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
