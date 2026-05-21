use std::process::Command;

#[test]
fn sprint81_cli_help_and_local_only_guards_are_present() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    let expected = [
        ("prototype-interpretation", "interpretation-only"),
        ("prototype-confidence", "diagnostic-only"),
        ("prototype-winner-gate", "no runtime selection"),
        ("prototype-disagreement", "offline-only"),
        ("committee-reference-audit-v2", "trinity-only"),
        ("sequence-core-decision-gate", "runtime deferred"),
        (
            "control-tower-prototype-interpretation",
            "read-only prototype interpretation panel",
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
    assert!(root_stdout.contains("prototype-interpretation"));
    assert!(root_stdout.contains("sequence-core-decision-gate"));
    assert!(root_stdout.contains("control-tower-prototype-interpretation"));
    assert!(!root_stdout.contains("train-model"));
    assert!(!root_stdout.contains("live-inference"));
    assert!(!root_stdout.contains("gated-deltanet-runtime"));

    for command in [
        "prototype-interpretation",
        "prototype-confidence",
        "prototype-winner-gate",
        "prototype-disagreement",
        "prototype-failure-modes",
        "prototype-calibration-risk",
        "no-trade-risk-denied-interpretation",
        "committee-reference-audit-v2",
        "committee-reference-depth-plan",
        "committee-sequence-disagreement",
        "sequence-core-decision-gate",
        "training-lineage-completeness",
        "control-tower-prototype-interpretation",
    ] {
        let remote = Command::new(bin)
            .args([command, "--config", "https://example.com/sprint81.toml"])
            .output()
            .expect("remote config");
        assert!(!remote.status.success());
        let stderr = String::from_utf8(remote.stderr).expect("stderr");
        assert!(stderr.contains("must be local"));
    }
}
