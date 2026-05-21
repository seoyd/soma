use std::process::Command;

#[test]
fn sprint91_commands_are_listed_in_help_and_runtime_commands_are_absent() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint91-krx-evidence-recover",
        "krx-evidence-real-reduction-plan",
        "krx-evidence-assertion-migration",
        "krx-evidence-fixture-setup-reduction",
        "krx-evidence-auth-boundary-preservation",
        "krx-evidence-endpoint-template-preservation",
        "krx-evidence-source-boundary-preservation",
        "krx-evidence-market-data-only-preservation",
        "krx-evidence-compile-impact",
        "krx-evidence-no-run-rerun",
        "krx-evidence-full-gate-rerun",
        "seven-blocker-queue-progress-v7",
        "measured-target-delta-v7",
        "real-no-run-gate-attempt-v6",
        "real-full-workspace-gate-attempt-v9",
        "workspace-gate-recovery-v8",
        "remaining-blocker-queue-v7",
        "safety-coverage-preservation-v7",
        "control-tower-krx-evidence-recovery",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
    for forbidden in [
        "train-model",
        "live-inference",
        "mamba-runtime",
        "gated-deltanet-runtime",
        "live-order",
        "broker-account",
    ] {
        assert!(
            !stdout.contains(forbidden),
            "unexpected command {forbidden}"
        );
    }
}

#[test]
fn sprint91_subcommand_help_and_remote_path_rejection_stay_explicit() {
    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["krx-evidence-auth-boundary-preservation", "--help"])
        .output()
        .expect("help");
    let help_text = String::from_utf8_lossy(&help.stdout);
    assert!(help_text.contains("no secret values"));

    for command in [
        "sprint91-krx-evidence-recover",
        "krx-evidence-real-reduction-plan",
        "krx-evidence-assertion-migration",
        "krx-evidence-fixture-setup-reduction",
        "krx-evidence-auth-boundary-preservation",
        "krx-evidence-endpoint-template-preservation",
        "krx-evidence-source-boundary-preservation",
        "krx-evidence-market-data-only-preservation",
        "krx-evidence-compile-impact",
        "krx-evidence-no-run-rerun",
        "krx-evidence-full-gate-rerun",
        "seven-blocker-queue-progress-v7",
        "measured-target-delta-v7",
        "real-no-run-gate-attempt-v6",
        "real-full-workspace-gate-attempt-v9",
        "workspace-gate-recovery-v8",
        "remaining-blocker-queue-v7",
        "safety-coverage-preservation-v7",
        "control-tower-krx-evidence-recovery",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
