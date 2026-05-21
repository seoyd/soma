use std::process::Command;

#[test]
fn sprint107_commands_are_listed_and_help_text_is_safe() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint107-safe-consolidation-patch",
        "safe-consolidation-patch-selection",
        "consolidation-candidate-risk-review",
        "assertion-migration-ledger-v1",
        "assertion-preservation-verification-v1",
        "safety-sentinel-preservation-v1",
        "shared-fixture-harness-application-v1",
        "shared-toml-builder-application-v1",
        "shared-output-dir-helper-application-v1",
        "shared-render-helper-application-v1",
        "artifact-render-cache-application-v1",
        "cli-smoke-tiering-application-v1",
        "consolidated-test-target-manifest-v1",
        "retired-narrow-target-manifest-v1",
        "test-binary-delta-v4",
        "post-patch-workspace-no-run-v23",
        "post-patch-workspace-full-v23",
        "workspace-no-run-recovery-gate-v8",
        "workspace-full-acceptance-gate-v8",
        "acceptance-truth-gate-v8",
        "control-tower-safe-consolidation-patch-v1",
        "control-tower-workspace-acceptance-recovery-v8",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
    for (command, needle) in [
        (
            "sprint107-safe-consolidation-patch",
            "first safe consolidation patch",
        ),
        ("assertion-migration-ledger-v1", "no assertion deletion"),
        ("safety-sentinel-preservation-v1", "sentinels preserved"),
        (
            "post-patch-workspace-no-run-v23",
            "no-run is not full acceptance",
        ),
        (
            "post-patch-workspace-full-v23",
            "finished and passed required",
        ),
        ("acceptance-truth-gate-v8", "focused is not full"),
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
fn sprint107_commands_reject_remote_paths() {
    for command in [
        "sprint107-safe-consolidation-patch",
        "safe-consolidation-patch-selection",
        "consolidation-candidate-risk-review",
        "assertion-migration-ledger-v1",
        "assertion-preservation-verification-v1",
        "safety-sentinel-preservation-v1",
        "shared-fixture-harness-application-v1",
        "shared-toml-builder-application-v1",
        "shared-output-dir-helper-application-v1",
        "shared-render-helper-application-v1",
        "artifact-render-cache-application-v1",
        "cli-smoke-tiering-application-v1",
        "consolidated-test-target-manifest-v1",
        "retired-narrow-target-manifest-v1",
        "test-binary-delta-v4",
        "post-patch-workspace-no-run-v23",
        "post-patch-workspace-full-v23",
        "workspace-no-run-recovery-gate-v8",
        "workspace-full-acceptance-gate-v8",
        "acceptance-truth-gate-v8",
        "control-tower-safe-consolidation-patch-v1",
        "control-tower-workspace-acceptance-recovery-v8",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success(), "{command} should fail");
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
