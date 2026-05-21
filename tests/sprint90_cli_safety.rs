use std::process::Command;

#[test]
fn sprint90_commands_are_listed_in_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint90-external-prediction-recover",
        "external-prediction-real-reduction-plan",
        "external-prediction-assertion-migration",
        "external-prediction-fixture-setup-reduction",
        "external-prediction-feature-variant-reduction",
        "external-prediction-compile-impact",
        "external-prediction-no-run-rerun",
        "external-prediction-full-gate-rerun",
        "external-prediction-schema-preservation",
        "external-prediction-model-card-preservation",
        "external-prediction-evaluation-preservation",
        "seven-blocker-queue-progress-v6",
        "measured-target-delta-v6",
        "real-no-run-gate-attempt-v5",
        "real-full-workspace-gate-attempt-v8",
        "workspace-gate-recovery-v7",
        "remaining-blocker-queue-v6",
        "safety-coverage-preservation-v6",
        "control-tower-external-prediction-recovery",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
}

#[test]
fn sprint90_commands_reject_remote_paths() {
    for command in [
        "sprint90-external-prediction-recover",
        "external-prediction-real-reduction-plan",
        "external-prediction-assertion-migration",
        "external-prediction-fixture-setup-reduction",
        "external-prediction-feature-variant-reduction",
        "external-prediction-compile-impact",
        "external-prediction-no-run-rerun",
        "external-prediction-full-gate-rerun",
        "external-prediction-schema-preservation",
        "external-prediction-model-card-preservation",
        "external-prediction-evaluation-preservation",
        "seven-blocker-queue-progress-v6",
        "measured-target-delta-v6",
        "real-no-run-gate-attempt-v5",
        "real-full-workspace-gate-attempt-v8",
        "workspace-gate-recovery-v7",
        "remaining-blocker-queue-v6",
        "safety-coverage-preservation-v6",
        "control-tower-external-prediction-recovery",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
