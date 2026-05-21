use std::process::Command;

#[test]
fn sprint79_cli_help_and_local_only_guards_are_present() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    let expected = [
        ("sequence-core-registry", "contract-only"),
        ("gated-deltanet-core-contract", "runtime deferred"),
        ("gated-deltanet-readiness", "no runtime"),
        ("sequence-core-comparison-plan", "offline comparison only"),
        ("sequence-core-external-contract", "prediction csv only"),
        ("training-storage-materialize", "no fake data"),
        ("training-storage-integrity", "manifest checks"),
        ("model-family-storage-contract", "no runtime/training"),
        ("control-tower-sequence-core", "read-only"),
    ];
    for (command, text) in expected {
        let help = Command::new(bin)
            .args([command, "--help"])
            .output()
            .expect("help");
        assert!(help.status.success());
        let stdout = String::from_utf8(help.stdout).expect("stdout");
        assert!(stdout.contains("--config"));
        assert!(stdout.to_lowercase().contains(&text.to_lowercase()));
    }

    let root_help = Command::new(bin).arg("--help").output().expect("root help");
    let root_stdout = String::from_utf8(root_help.stdout).expect("stdout");
    assert!(root_stdout.contains("sequence-core-registry"));
    assert!(root_stdout.contains("gated-deltanet-core-contract"));
    assert!(root_stdout.contains("training-storage-materialize"));
    assert!(!root_stdout.contains("train-model"));
    assert!(!root_stdout.contains("live-inference"));
    assert!(!root_stdout.contains("gated-deltanet-runtime"));

    for command in [
        "sequence-core-registry",
        "gated-deltanet-core-contract",
        "gated-deltanet-readiness",
        "sequence-core-comparison-plan",
        "sequence-core-external-contract",
        "training-storage-materialize",
        "training-storage-integrity",
        "model-family-storage-contract",
        "control-tower-sequence-core",
    ] {
        let remote = Command::new(bin)
            .args([command, "--config", "https://example.com/sprint79.toml"])
            .output()
            .expect("remote config");
        assert!(!remote.status.success());
        let stderr = String::from_utf8(remote.stderr).expect("stderr");
        assert!(stderr.contains("must be local"));
    }
}
