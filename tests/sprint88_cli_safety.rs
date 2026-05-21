use std::process::Command;

#[test]
fn sprint88_commands_are_listed_in_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint88-seven-blocker-recover",
        "seven-blocker-family-recovery",
        "per-family-compile-probe",
        "per-family-no-run-probe",
        "per-family-execution-probe",
        "candle-expansion-recovery",
        "external-prediction-recovery",
        "krx-evidence-recovery",
        "dashboard-renderer-recovery",
        "committee-cli-safety-isolation",
        "baseline-signal-recovery",
        "counterfactual-backfill-recovery",
        "dev-dependency-impact-probe",
        "feature-variant-impact-probe",
        "measured-target-delta-v4",
        "real-no-run-gate-attempt-v3",
        "real-full-workspace-gate-attempt-v6",
        "gate-rerun-after-each-family",
        "workspace-gate-recovery-v5",
        "remaining-blocker-queue-v4",
        "safety-coverage-preservation-v4",
        "control-tower-seven-blocker",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
}

#[test]
fn sprint88_commands_reject_remote_paths() {
    for command in [
        "sprint88-seven-blocker-recover",
        "seven-blocker-family-recovery",
        "per-family-compile-probe",
        "per-family-no-run-probe",
        "per-family-execution-probe",
        "candle-expansion-recovery",
        "external-prediction-recovery",
        "krx-evidence-recovery",
        "dashboard-renderer-recovery",
        "committee-cli-safety-isolation",
        "baseline-signal-recovery",
        "counterfactual-backfill-recovery",
        "dev-dependency-impact-probe",
        "feature-variant-impact-probe",
        "measured-target-delta-v4",
        "real-no-run-gate-attempt-v3",
        "real-full-workspace-gate-attempt-v6",
        "gate-rerun-after-each-family",
        "workspace-gate-recovery-v5",
        "remaining-blocker-queue-v4",
        "safety-coverage-preservation-v4",
        "control-tower-seven-blocker",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
