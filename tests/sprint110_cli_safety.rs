use std::process::Command;

#[test]
fn sprint110_commands_are_listed_and_help_text_is_safe() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let commands = [
        "sprint110-safe-consolidation-patch-v4",
        "sprint109-validation-reconcile",
        "sprint109-focused-suite-import",
        "sprint109-cli-smoke-import",
        "sprint109-cargo-build-import",
        "sprint109-workspace-timeout-import",
        "fourth-safe-consolidation-patch-selection",
        "assertion-migration-ledger-v4",
        "cumulative-assertion-migration-ledger-v2",
        "equivalent-coverage-proof-v3",
        "retired-target-safety-audit-v4",
        "safety-sentinel-preservation-v4",
        "shared-fixture-harness-expansion-v4",
        "shared-render-helper-expansion-v4",
        "cli-smoke-tiering-application-v4",
        "test-binary-delta-v7",
        "cumulative-binary-delta-v2",
        "extended-no-run-observation-v3",
        "workspace-cargo-json-progress-v4",
        "timeout-cleanup-verification-v3",
        "workspace-no-run-recovery-gate-v11",
        "workspace-full-acceptance-gate-v11",
        "acceptance-truth-gate-v11",
        "control-tower-safe-consolidation-patch-v4",
        "control-tower-workspace-acceptance-recovery-v11",
    ];
    for command in commands {
        assert!(stdout.contains(command), "missing command {command}");
    }
    let sprint110_help_lines = stdout
        .lines()
        .filter(|line| commands.iter().any(|command| line.contains(command)))
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in [
        "training",
        "live-inference",
        "mamba-runtime",
        "gated-runtime",
        "broker",
        "order",
        "account",
    ] {
        assert!(!sprint110_help_lines.contains(forbidden));
    }
}
