use std::process::Command;

#[test]
fn sprint75_cli_help_and_local_only_guards_are_present() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    let expected = [
        ("real-prediction-requirements", "research-only"),
        ("real-prediction-refresh-plan", "training"),
        ("real-prediction-import", "schema"),
        ("real-external-reevaluate", "offline"),
        ("real-leaderboard-refresh", "deployment"),
        ("real-modelops-refresh", "live inference"),
        ("model-predictions-stale-close", "warning"),
        ("control-tower-warning-close-v2", "read-only"),
        ("direct-watch-post-evidence-gate", "monitoring-only"),
        ("real-modelops-runbook", "copyable"),
    ];
    for (command, text) in expected {
        let help = Command::new(bin)
            .args([command, "--help"])
            .output()
            .expect("help");
        assert!(help.status.success(), "{command} --help failed");
        let stdout = String::from_utf8(help.stdout).expect("stdout");
        assert!(stdout.contains("--config"));
        assert!(stdout.to_lowercase().contains(&text.to_lowercase()));
    }
    let root_help = Command::new(bin).arg("--help").output().expect("root help");
    assert!(root_help.status.success());
    let root_stdout = String::from_utf8(root_help.stdout).expect("stdout");
    assert!(root_stdout.contains("real-prediction-requirements"));
    assert!(root_stdout.contains("real-modelops-runbook"));
    assert!(!root_stdout.contains("train-model"));
    assert!(!root_stdout.contains("live-inference"));
    assert!(!root_stdout.contains("mamba-runtime"));
    assert!(!root_stdout.contains("broker-order"));

    let remote = Command::new(bin)
        .args([
            "real-prediction-requirements",
            "--config",
            "https://example.com/sprint75.toml",
        ])
        .output()
        .expect("remote config");
    assert!(!remote.status.success());
    let stderr = String::from_utf8(remote.stderr).expect("stderr");
    assert!(stderr.contains("must be local"));
}
