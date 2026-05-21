use std::process::Command;

#[test]
fn workspace_help_contains_research_only_warning_and_workspace_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Research-only"));
    for command in [
        "future-window-requirements",
        "future-window-extension-plan",
        "outcome-linkage-v3",
        "counterfactual-complete-v2",
        "complete-row-close-v2",
        "sprint85-workspace-gate-recover",
        "workspace-test-surface-audit",
        "remaining-test-binary-inventory",
        "domain-suite-plan",
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
fn workspace_commands_reject_remote_paths_and_hide_live_commands() {
    for command in [
        "future-window-requirements",
        "future-window-extension-plan",
        "outcome-linkage-v3",
        "counterfactual-complete-v2",
        "complete-row-close-v2",
        "workspace-test-surface-audit",
        "remaining-test-binary-inventory",
        "domain-suite-plan",
        "workspace-acceptance-attempt-v3",
        "control-tower-workspace-gate-v2",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&help.stdout);
    for forbidden in [
        "\n  live",
        "\n  order",
        "\n  broker",
        "\n  account",
        "mamba-runtime",
    ] {
        assert!(!stdout.contains(forbidden));
    }
}

#[test]
fn workspace_help_output_is_deterministic() {
    let first = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("first");
    let second = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("second");
    assert_eq!(first.stdout, second.stdout);
}
