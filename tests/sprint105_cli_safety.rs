use std::process::Command;

#[test]
fn sprint105_commands_are_listed_and_help_text_is_safe() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint105-verification-patch-close",
        "verification-finding-closure",
        "review-patch-effect",
        "overclaim-regression-guard",
        "workspace-attempt-truth-hardening",
        "safety-boolean-coverage-audit",
        "paper-rejected-transition-audit",
        "risk-required-transition-audit",
        "missing-artifact-finding-policy",
        "final-verification-gate-v2",
        "dual-agent-review-loop-v2",
        "paper-lifecycle-warning-closure",
        "paper-candidate-transition-coverage",
        "paper-candidate-gate-completeness",
        "paper-candidate-evidence-depth-closure",
        "paper-candidate-trace-closure",
        "paper-candidate-stability-closure",
        "risk-governor-batch-veto-warning-closure",
        "risk-governor-no-bypass-audit-v2",
        "lower-confidence-carry-forward-closure",
        "paper-lifecycle-readiness-gate-v2",
        "paper-candidate-batch-replay-v2",
        "workspace-acceptance-truth-recovery-plan-v6",
        "workspace-compile-cost-diagnosis-v2",
        "focused-vs-full-gate-bridge-v2",
        "safety-coverage-preservation-v21",
        "control-tower-verification-patch-closure",
        "control-tower-paper-lifecycle-closure",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
    for (command, needle) in [
        (
            "sprint105-verification-patch-close",
            "verification patch closure",
        ),
        ("overclaim-regression-guard", "finished && passed only"),
        ("paper-lifecycle-warning-closure", "not an order"),
        ("risk-required-transition-audit", "Risk Governor required"),
        (
            "lower-confidence-carry-forward-closure",
            "no silent confidence upgrade",
        ),
        (
            "workspace-acceptance-truth-recovery-plan-v6",
            "full workspace separate",
        ),
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
fn sprint105_commands_reject_remote_paths() {
    for command in [
        "sprint105-verification-patch-close",
        "verification-finding-closure",
        "review-patch-effect",
        "overclaim-regression-guard",
        "workspace-attempt-truth-hardening",
        "safety-boolean-coverage-audit",
        "paper-rejected-transition-audit",
        "risk-required-transition-audit",
        "missing-artifact-finding-policy",
        "final-verification-gate-v2",
        "dual-agent-review-loop-v2",
        "paper-lifecycle-warning-closure",
        "paper-candidate-transition-coverage",
        "paper-candidate-gate-completeness",
        "paper-candidate-evidence-depth-closure",
        "paper-candidate-trace-closure",
        "paper-candidate-stability-closure",
        "risk-governor-batch-veto-warning-closure",
        "risk-governor-no-bypass-audit-v2",
        "lower-confidence-carry-forward-closure",
        "paper-lifecycle-readiness-gate-v2",
        "paper-candidate-batch-replay-v2",
        "workspace-acceptance-truth-recovery-plan-v6",
        "workspace-compile-cost-diagnosis-v2",
        "focused-vs-full-gate-bridge-v2",
        "safety-coverage-preservation-v21",
        "control-tower-verification-patch-closure",
        "control-tower-paper-lifecycle-closure",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success(), "{command} should fail");
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
