use std::process::Command;

#[test]
fn sprint77_cli_help_and_local_only_guards_are_present() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    let expected = [
        ("repeated-workspace-timing", "no fake timing"),
        ("test-binary-cost", "diagnostic"),
        ("fixture-setup-cost", "no semantic changes"),
        ("artifact-render-cost", "deterministic"),
        ("cli-smoke-cost-reduce", "safety smoke retained"),
        ("fixture-dedup-plan", "no test deletion"),
        ("fixture-cache-plan", "no secret cache"),
        ("artifact-render-cache-plan", "no hidden failures"),
        ("test-support-refactor-plan", "manual review"),
        ("dev-loop-savings-estimate", "estimate-only"),
        ("workspace-acceptance-v3", "full workspace"),
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
    assert!(root_stdout.contains("repeated-workspace-timing"));
    assert!(root_stdout.contains("workspace-acceptance-v3"));
    assert!(!root_stdout.contains("mamba-runtime"));
    assert!(!root_stdout.contains("train-model"));

    for command in [
        "repeated-workspace-timing",
        "test-binary-cost",
        "fixture-setup-cost",
        "artifact-render-cost",
        "cli-smoke-cost-reduce",
        "fixture-dedup-plan",
        "fixture-cache-plan",
        "artifact-render-cache-plan",
        "test-support-refactor-plan",
        "dev-loop-savings-estimate",
        "workspace-acceptance-v3",
    ] {
        let remote = Command::new(bin)
            .args([command, "--config", "https://example.com/sprint77.toml"])
            .output()
            .expect("remote config");
        assert!(!remote.status.success());
        let stderr = String::from_utf8(remote.stderr).expect("stderr");
        assert!(stderr.contains("must be local"));
    }
}
