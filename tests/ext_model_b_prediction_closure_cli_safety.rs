use std::process::Command;

#[test]
fn sprint73_cli_help_and_local_only_guards_are_present() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    for command in [
        "ext-model-b-prediction-close",
        "prediction-coverage-finalize",
        "evidence-gap-final-close",
        "direct-watch-final-gate",
        "control-tower-final-refresh",
        "sprint73-workspace-acceptance",
    ] {
        let help = Command::new(bin)
            .args([command, "--help"])
            .output()
            .expect("sprint73 help");
        assert!(help.status.success(), "{command} --help failed");
        let stdout = String::from_utf8(help.stdout).expect("stdout utf8");
        assert!(stdout.contains("--config"));
    }

    let root_help = Command::new(bin).arg("--help").output().expect("root help");
    assert!(root_help.status.success());
    let root_stdout = String::from_utf8(root_help.stdout).expect("stdout utf8");
    assert!(root_stdout.contains("ext-model-b-prediction-close"));
    assert!(root_stdout.contains("prediction-coverage-finalize"));
    assert!(root_stdout.contains("sprint73-workspace-acceptance"));
    assert!(!root_stdout.contains("train-model"));
    assert!(!root_stdout.contains("live-inference"));
    assert!(!root_stdout.contains("broker-order"));

    let remote = Command::new(bin)
        .args([
            "ext-model-b-prediction-close",
            "--config",
            "https://example.com/sprint73.toml",
        ])
        .output()
        .expect("remote config");
    assert!(!remote.status.success());
    let stderr = String::from_utf8(remote.stderr).expect("stderr utf8");
    assert!(stderr.contains("must be local") || stderr.contains("config path must be local"));
}
