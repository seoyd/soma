use std::process::Command;

#[test]
fn sprint108_commands_are_listed_and_help_text_is_safe() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint108-safe-consolidation-patch-v2",
        "sprint107-verification-reconcile",
        "independent-verification-closure-v1",
        "verification-patch-carry-forward",
        "second-safe-consolidation-patch-selection",
        "assertion-migration-ledger-v2",
        "equivalent-coverage-proof-v1",
        "retired-target-safety-audit-v2",
        "safety-sentinel-preservation-v2",
        "shared-fixture-harness-expansion-v2",
        "shared-render-helper-expansion-v2",
        "cli-smoke-tiering-application-v2",
        "test-binary-delta-v5",
        "extended-no-run-observation-v1",
        "timeout-cleanup-verification-v1",
        "workspace-no-run-recovery-gate-v9",
        "workspace-full-acceptance-gate-v9",
        "acceptance-truth-gate-v9",
        "control-tower-safe-consolidation-patch-v2",
        "control-tower-workspace-acceptance-recovery-v9",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
    for forbidden in [
        "sprint108-training",
        "sprint108-live-inference",
        "sprint108-mamba-runtime",
        "sprint108-gated-runtime",
        "sprint108-broker",
        "sprint108-order",
        "sprint108-account",
    ] {
        assert!(
            !stdout.contains(forbidden),
            "unexpected command {forbidden}"
        );
    }
    for (command, needle) in [
        (
            "sprint108-safe-consolidation-patch-v2",
            "second safe consolidation patch",
        ),
        (
            "sprint107-verification-reconcile",
            "5.5 verification is not acceptance",
        ),
        ("assertion-migration-ledger-v2", "no assertion deletion"),
        (
            "equivalent-coverage-proof-v1",
            "coverage required before retirement",
        ),
        ("timeout-cleanup-verification-v1", "timeout is not pass"),
        ("acceptance-truth-gate-v9", "focused is not full"),
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
fn sprint108_commands_reject_remote_paths() {
    for command in [
        "sprint108-safe-consolidation-patch-v2",
        "sprint107-verification-reconcile",
        "independent-verification-closure-v1",
        "verification-patch-carry-forward",
        "second-safe-consolidation-patch-selection",
        "assertion-migration-ledger-v2",
        "equivalent-coverage-proof-v1",
        "retired-target-safety-audit-v2",
        "safety-sentinel-preservation-v2",
        "shared-fixture-harness-expansion-v2",
        "shared-render-helper-expansion-v2",
        "cli-smoke-tiering-application-v2",
        "test-binary-delta-v5",
        "extended-no-run-observation-v1",
        "timeout-cleanup-verification-v1",
        "workspace-no-run-recovery-gate-v9",
        "workspace-full-acceptance-gate-v9",
        "acceptance-truth-gate-v9",
        "control-tower-safe-consolidation-patch-v2",
        "control-tower-workspace-acceptance-recovery-v9",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success(), "{command} should fail");
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
