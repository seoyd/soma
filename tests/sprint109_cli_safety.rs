use std::process::Command;

#[test]
fn sprint109_commands_are_listed_and_help_text_is_safe() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint109-safe-consolidation-patch-v3",
        "sprint108-verification-carry-forward",
        "previous-patch-ledger-carry-forward",
        "cumulative-assertion-migration-ledger",
        "third-safe-consolidation-patch-selection",
        "assertion-migration-ledger-v3",
        "equivalent-coverage-proof-v2",
        "retired-target-safety-audit-v3",
        "safety-sentinel-preservation-v3",
        "shared-fixture-harness-expansion-v3",
        "shared-render-helper-expansion-v3",
        "cli-smoke-tiering-application-v3",
        "test-binary-delta-v6",
        "extended-no-run-observation-v2",
        "timeout-cleanup-verification-v2",
        "workspace-no-run-recovery-gate-v10",
        "workspace-full-acceptance-gate-v10",
        "acceptance-truth-gate-v10",
        "control-tower-safe-consolidation-patch-v3",
        "control-tower-workspace-acceptance-recovery-v10",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
    for forbidden in [
        "sprint109-training",
        "sprint109-live-inference",
        "sprint109-mamba-runtime",
        "sprint109-gated-runtime",
        "sprint109-broker",
        "sprint109-order",
        "sprint109-account",
    ] {
        assert!(
            !stdout.contains(forbidden),
            "unexpected command {forbidden}"
        );
    }
    for (command, needle) in [
        (
            "sprint109-safe-consolidation-patch-v3",
            "third safe consolidation patch",
        ),
        (
            "sprint108-verification-carry-forward",
            "5.5 verification is not acceptance",
        ),
        ("assertion-migration-ledger-v3", "no assertion deletion"),
        (
            "equivalent-coverage-proof-v2",
            "coverage required before retirement",
        ),
        ("timeout-cleanup-verification-v2", "timeout is not pass"),
        ("acceptance-truth-gate-v10", "focused is not full"),
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
fn sprint109_commands_reject_remote_paths() {
    for command in [
        "sprint109-safe-consolidation-patch-v3",
        "sprint108-verification-carry-forward",
        "previous-patch-ledger-carry-forward",
        "cumulative-assertion-migration-ledger",
        "third-safe-consolidation-patch-selection",
        "assertion-migration-ledger-v3",
        "equivalent-coverage-proof-v2",
        "retired-target-safety-audit-v3",
        "safety-sentinel-preservation-v3",
        "shared-fixture-harness-expansion-v3",
        "shared-render-helper-expansion-v3",
        "cli-smoke-tiering-application-v3",
        "test-binary-delta-v6",
        "extended-no-run-observation-v2",
        "timeout-cleanup-verification-v2",
        "workspace-no-run-recovery-gate-v10",
        "workspace-full-acceptance-gate-v10",
        "acceptance-truth-gate-v10",
        "control-tower-safe-consolidation-patch-v3",
        "control-tower-workspace-acceptance-recovery-v10",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success(), "{command} should fail");
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
