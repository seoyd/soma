use std::process::Command;

#[test]
fn sprint98_commands_are_listed_and_help_text_is_safe() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "committee-owned-core-architecture",
        "investor-style-registry",
        "ai-committee-member-specs",
        "committee-member-core-contracts",
        "committee-member-learning-policy",
        "committee-member-proposals",
        "entry-timing-proposals",
        "committee-debate-trigger",
        "committee-debate-session",
        "chairman-governance-policy",
        "chairman-rule-proposal",
        "chairman-rulebook-version",
        "rule-adaptation-audit",
        "promotion-demotion-policy",
        "member-scorecards",
        "member-promotion-demotion",
        "committee-roster-lifecycle",
        "paper-only-committee-decision",
        "control-tower-ai-committee",
        "sprint98-committee-owned-core",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }

    let checks = [
        (
            "committee-owned-core-architecture",
            "central core is deprecated",
        ),
        ("investor-style-registry", "no impersonation"),
        ("ai-committee-member-specs", "owns its own core"),
        ("committee-member-proposals", "entry timing proposals"),
        ("committee-debate-session", "paper-only debate"),
        ("chairman-governance-policy", "cannot bypass Risk Governor"),
        ("promotion-demotion-policy", "multi-axis"),
        ("paper-only-committee-decision", "no broker execution"),
        ("control-tower-ai-committee", "static status only"),
    ];
    for (command, needle) in checks {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("subcommand help");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(needle), "missing {needle} for {command}");
    }
}

#[test]
fn sprint98_commands_reject_remote_paths() {
    for command in [
        "committee-owned-core-architecture",
        "investor-style-registry",
        "ai-committee-member-specs",
        "committee-member-core-contracts",
        "committee-member-learning-policy",
        "committee-member-proposals",
        "entry-timing-proposals",
        "committee-debate-trigger",
        "committee-debate-session",
        "chairman-governance-policy",
        "chairman-rule-proposal",
        "chairman-rulebook-version",
        "rule-adaptation-audit",
        "promotion-demotion-policy",
        "member-scorecards",
        "member-promotion-demotion",
        "committee-roster-lifecycle",
        "paper-only-committee-decision",
        "control-tower-ai-committee",
        "sprint98-committee-owned-core",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success(), "{command} should fail");
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
