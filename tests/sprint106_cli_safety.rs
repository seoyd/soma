use std::process::Command;

#[test]
fn sprint106_commands_are_listed_and_help_text_is_safe() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint106-workspace-acceptance-recover",
        "real-no-run-completion-v22",
        "real-full-workspace-attempt-v22",
        "workspace-compile-cost-profile-v3",
        "cargo-json-no-run-capture-v2",
        "test-binary-inventory-v3",
        "test-binary-explosion-attribution",
        "integration-target-cost-ranking",
        "long-running-rustc-snapshot-v2",
        "fixture-setup-cost-attribution-v2",
        "artifact-render-cost-attribution-v2",
        "cli-smoke-cost-attribution-v2",
        "high-cost-test-family-clusters",
        "safe-test-binary-consolidation-plan-v2",
        "shared-fixture-harness-expansion-plan-v2",
        "cli-smoke-tiering-plan-v2",
        "workspace-no-run-recovery-gate-v7",
        "workspace-full-acceptance-gate-v7",
        "focused-vs-full-bridge-v3",
        "acceptance-truth-gate-v7",
        "acceptance-recovery-patch-plan",
        "acceptance-recovery-verification",
        "safety-coverage-preservation-v22",
        "control-tower-workspace-acceptance-recovery-v7",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
    for (command, needle) in [
        (
            "sprint106-workspace-acceptance-recover",
            "workspace acceptance recovery",
        ),
        (
            "real-no-run-completion-v22",
            "no-run is not full acceptance",
        ),
        (
            "real-full-workspace-attempt-v22",
            "finished and passed required",
        ),
        ("acceptance-truth-gate-v7", "focused is not full"),
        (
            "safe-test-binary-consolidation-plan-v2",
            "no assertion deletion",
        ),
        ("safety-coverage-preservation-v22", "safety preserved"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("subcommand help");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(needle), "missing {needle} for {command}");
    }
}

#[test]
fn sprint106_commands_reject_remote_paths() {
    for command in [
        "sprint106-workspace-acceptance-recover",
        "real-no-run-completion-v22",
        "real-full-workspace-attempt-v22",
        "workspace-compile-cost-profile-v3",
        "cargo-json-no-run-capture-v2",
        "test-binary-inventory-v3",
        "test-binary-explosion-attribution",
        "integration-target-cost-ranking",
        "long-running-rustc-snapshot-v2",
        "fixture-setup-cost-attribution-v2",
        "artifact-render-cost-attribution-v2",
        "cli-smoke-cost-attribution-v2",
        "high-cost-test-family-clusters",
        "safe-test-binary-consolidation-plan-v2",
        "shared-fixture-harness-expansion-plan-v2",
        "cli-smoke-tiering-plan-v2",
        "workspace-no-run-recovery-gate-v7",
        "workspace-full-acceptance-gate-v7",
        "focused-vs-full-bridge-v3",
        "acceptance-truth-gate-v7",
        "acceptance-recovery-patch-plan",
        "acceptance-recovery-verification",
        "safety-coverage-preservation-v22",
        "control-tower-workspace-acceptance-recovery-v7",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success(), "{command} should fail");
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
