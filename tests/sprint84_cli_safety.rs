mod support;

#[test]
fn sprint84_cli_help_and_local_only_guards_are_present() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    let expected = [
        (
            "sprint84-test-cost-reduce",
            "no safety deletion, no runtime/training/live behavior",
        ),
        (
            "test-binary-consolidate",
            "preserves assertions and keeps high-risk files separate",
        ),
        (
            "shared-fixture-harness-report",
            "deterministic helper migration status",
        ),
        (
            "representative-smoke-harness",
            "safety smoke remains retained",
        ),
        ("exhaustive-smoke-manifest", "full/release documentation"),
        (
            "safety-smoke-manifest",
            "required help/local-only/forbidden-command checks",
        ),
        ("workspace-final-gate-v2", "no fake pass"),
        (
            "control-tower-test-cost",
            "read-only control tower test cost panel",
        ),
    ];
    for (command, text) in expected {
        let help = std::process::Command::new(bin)
            .args([command, "--help"])
            .output()
            .expect("help");
        assert!(help.status.success());
        let stdout = String::from_utf8(help.stdout).expect("stdout");
        assert!(stdout.contains("--config"));
        assert!(stdout.to_lowercase().contains(&text.to_lowercase()));
    }

    let root_help = std::process::Command::new(bin)
        .arg("--help")
        .output()
        .expect("root help");
    let root_stdout = String::from_utf8(root_help.stdout).expect("stdout");
    assert!(root_stdout.contains("sprint84-test-cost-reduce"));
    assert!(root_stdout.contains("control-tower-test-cost"));
    assert!(!root_stdout.contains("train-model"));
    assert!(!root_stdout.contains("live-inference"));

    for command in [
        "sprint84-test-cost-reduce",
        "test-binary-consolidate",
        "shared-fixture-harness-report",
        "representative-smoke-harness",
        "exhaustive-smoke-manifest",
        "safety-smoke-manifest",
        "cli-smoke-execution-policy",
        "test-runtime-before-after",
        "workspace-final-gate-v2",
        "control-tower-test-cost",
    ] {
        let remote = std::process::Command::new(bin)
            .args([command, "--config", "https://example.com/sprint84.toml"])
            .output()
            .expect("remote config");
        assert!(!remote.status.success());
        let stderr = String::from_utf8(remote.stderr).expect("stderr");
        assert!(stderr.contains("must be local"));
    }
}
