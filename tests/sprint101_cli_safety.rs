use std::process::Command;

#[test]
fn sprint101_commands_are_listed_and_help_text_is_safe() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint101-investor-archetype-ingest",
        "investor-archetype-ingestion",
        "investor-source-confidence",
        "investor-safety-normalization",
        "investor-feature-vector-cards",
        "investor-do-not-learn-guards",
        "investor-impersonation-risk",
        "investor-unverified-claim-filter",
        "investor-private-life-myth-filter",
        "eighteen-investor-registry",
        "style-group-taxonomy",
        "style-conflict-matrix",
        "regime-routing-policy",
        "multi-expert-committee-topology",
        "member-confidence-weight-policy",
        "member-feature-scope-mapping",
        "member-learning-data-cards",
        "archetype-to-member-mapping",
        "eighteen-roster-plan",
        "eighteen-activation-gate",
        "paper-roster-expansion-gate",
        "chairman-style-governance-v2",
        "promotion-demotion-policy-v2",
        "control-tower-investor-archetype",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
    for (command, needle) in [
        ("sprint101-investor-archetype-ingest", "no impersonation"),
        ("investor-feature-vector-cards", "not trained models"),
        (
            "eighteen-investor-registry",
            "does not imply 18 live AI agents",
        ),
        (
            "chairman-style-governance-v2",
            "cannot bypass Risk Governor",
        ),
        (
            "control-tower-investor-archetype",
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
    assert!(!stdout.contains("investor-live-trading"));
}

#[test]
fn sprint101_commands_reject_remote_paths() {
    for command in [
        "sprint101-investor-archetype-ingest",
        "investor-archetype-ingestion",
        "investor-source-confidence",
        "investor-safety-normalization",
        "investor-feature-vector-cards",
        "investor-do-not-learn-guards",
        "investor-impersonation-risk",
        "investor-unverified-claim-filter",
        "investor-private-life-myth-filter",
        "eighteen-investor-registry",
        "style-group-taxonomy",
        "style-conflict-matrix",
        "regime-routing-policy",
        "multi-expert-committee-topology",
        "member-confidence-weight-policy",
        "member-feature-scope-mapping",
        "member-learning-data-cards",
        "archetype-to-member-mapping",
        "eighteen-roster-plan",
        "eighteen-activation-gate",
        "paper-roster-expansion-gate",
        "chairman-style-governance-v2",
        "promotion-demotion-policy-v2",
        "control-tower-investor-archetype",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success(), "{command} should fail");
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
