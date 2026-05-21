use std::process::Command;

#[test]
fn sprint103_commands_are_listed_and_help_text_is_safe() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint103-paper-rotation-close",
        "paper-rotation-warning-closure",
        "lower-confidence-evidence-closure",
        "wonyotti-warning-closure",
        "larry-williams-warning-closure",
        "arthur-hayes-warning-closure",
        "paper-notrade-justification",
        "paper-rotation-readiness-gate-v2",
        "control-tower-paper-rotation-closure",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
    for (command, needle) in [
        ("sprint103-paper-rotation-close", "warning-closure-only"),
        (
            "lower-confidence-evidence-closure",
            "no silent confidence upgrade",
        ),
        ("wonyotti-warning-closure", "exact return claims"),
        ("larry-williams-warning-closure", "exact numeric rules"),
        ("arthur-hayes-warning-closure", "leverage risk"),
        (
            "paper-notrade-justification",
            "valid defensive paper outcome",
        ),
        (
            "paper-rotation-readiness-gate-v2",
            "live rotation remains forbidden",
        ),
        ("control-tower-paper-rotation-closure", "static/read-only"),
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
fn sprint103_commands_reject_remote_paths() {
    for command in [
        "sprint103-paper-rotation-close",
        "paper-rotation-warning-closure",
        "rotation-plan-warning-closure",
        "member-selection-warning-closure",
        "lower-confidence-evidence-closure",
        "wonyotti-warning-closure",
        "larry-williams-warning-closure",
        "arthur-hayes-warning-closure",
        "proposal-run-warning-closure",
        "entry-timing-warning-closure",
        "debate-session-warning-closure",
        "need-more-evidence-resolution-plan",
        "cross-group-conflict-closure",
        "chairman-synthesis-warning-closure",
        "style-weight-audit-warning-closure",
        "risk-governor-handoff-warning-closure-v2",
        "paper-trace-warning-closure",
        "paper-replay-warning-closure-v2",
        "expectation-trace-warning-closure",
        "notrade-riskdenied-trace-warning-closure",
        "regime-routing-warning-closure",
        "multi-expert-coverage-warning-closure",
        "paper-roster-usage-warning-closure",
        "watchlist-member-usage-policy",
        "saylor-treasury-watchlist-audit",
        "multi-scenario-paper-replay",
        "scenario-outcome-expectation-matrix",
        "committee-decision-stability",
        "paper-notrade-justification",
        "paper-need-more-evidence-justification",
        "risk-governor-notrade-reason-audit",
        "paper-rotation-readiness-gate-v2",
        "workspace-truth-closure-plan-v4",
        "workspace-acceptance-attempt-v19",
        "safety-coverage-preservation-v19",
        "control-tower-paper-rotation-closure",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success(), "{command} should fail");
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
