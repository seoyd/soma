use std::process::Command;

#[test]
fn sprint80_cli_help_and_local_only_guards_are_present() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    let expected = [
        ("sequence-core-prototype-compare", "prototype-only"),
        (
            "sequence-core-prototype-registry",
            "prototype artifact registry only",
        ),
        ("mamba3fin-prototype-import", "prediction csv only"),
        ("gated-deltanet-prototype-import", "prediction csv only"),
        ("sequence-core-prototype-evaluate", "offline-only"),
        ("committee-evidence-expand-v2", "trinity-only"),
        ("committee-vs-sequence-core", "diagnostic-only"),
        (
            "training-artifact-populate",
            "no fake data artifact population",
        ),
        ("training-populated-integrity", "secret safety"),
        (
            "control-tower-sequence-prototype",
            "read-only sequence prototype panel",
        ),
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
    assert!(root_stdout.contains("sequence-core-prototype-compare"));
    assert!(root_stdout.contains("committee-evidence-expand-v2"));
    assert!(root_stdout.contains("training-artifact-populate"));
    assert!(!root_stdout.contains("train-model"));
    assert!(!root_stdout.contains("live-inference"));
    assert!(!root_stdout.contains("gated-deltanet-runtime"));

    for command in [
        "sequence-core-prototype-compare",
        "sequence-core-prototype-registry",
        "mamba3fin-prototype-import",
        "gated-deltanet-prototype-import",
        "sequence-core-prototype-evaluate",
        "committee-evidence-expand-v2",
        "committee-vs-sequence-core",
        "training-artifact-populate",
        "training-populated-integrity",
        "control-tower-sequence-prototype",
    ] {
        let remote = Command::new(bin)
            .args([command, "--config", "https://example.com/sprint80.toml"])
            .output()
            .expect("remote config");
        assert!(!remote.status.success());
        let stderr = String::from_utf8(remote.stderr).expect("stderr");
        assert!(stderr.contains("must be local"));
    }
}
