use std::process::Command;

#[test]
fn sprint64_help_texts_include_safety_language() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");

    let registry_help = Command::new(bin)
        .args(["external-artifact-registry", "--help"])
        .output()
        .expect("run registry help");
    assert!(String::from_utf8_lossy(&registry_help.stdout).contains("no training"));

    let history_help = Command::new(bin)
        .args(["external-evaluation-history", "--help"])
        .output()
        .expect("run history help");
    assert!(String::from_utf8_lossy(&history_help.stdout).contains("Research-only"));

    let drift_help = Command::new(bin)
        .args(["calibration-drift", "--help"])
        .output()
        .expect("run drift help");
    assert!(String::from_utf8_lossy(&drift_help.stdout).contains("offline calibration"));

    let comparison_help = Command::new(bin)
        .args(["external-model-version-comparison", "--help"])
        .output()
        .expect("run comparison help");
    assert!(String::from_utf8_lossy(&comparison_help.stdout).contains("diagnostic-only"));

    let leaderboard_help = Command::new(bin)
        .args(["conservative-external-leaderboard", "--help"])
        .output()
        .expect("run leaderboard help");
    assert!(String::from_utf8_lossy(&leaderboard_help.stdout).contains("no deployment"));

    let audit_help = Command::new(bin)
        .args(["external-registry-audit", "--help"])
        .output()
        .expect("run audit help");
    assert!(String::from_utf8_lossy(&audit_help.stdout).contains("local-only"));
}

#[test]
fn forbidden_runtime_commands_do_not_exist() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    for command in [
        "train-mamba-runtime",
        "live-inference",
        "broker-account-order",
    ] {
        let output = Command::new(bin)
            .args([command, "--help"])
            .output()
            .expect("run forbidden help");
        assert!(!output.status.success());
    }
}
