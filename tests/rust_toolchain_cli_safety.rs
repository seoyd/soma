use std::process::Command;

#[test]
fn sprint76_cli_help_and_local_only_guards_are_present() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    let expected = [
        ("rust-toolchain-modernize", "stable-only"),
        ("toolchain-version-report", "local-only"),
        ("cargo-workspace-audit", "no runtime feature changes"),
        ("test-tier-plan", "full workspace"),
        ("test-runtime-budget", "timing guidance"),
        ("slow-test-inventory", "no-test-deletion"),
        ("cli-smoke-tiering", "safety smoke"),
        ("developer-speed-runbook", "copyable commands only"),
        ("workspace-acceptance-v2", "full workspace"),
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
    assert!(root_stdout.contains("rust-toolchain-modernize"));
    assert!(root_stdout.contains("workspace-acceptance-v2"));
    assert!(!root_stdout.contains("mamba-runtime"));
    assert!(!root_stdout.contains("train-model"));

    for command in [
        "rust-toolchain-modernize",
        "toolchain-version-report",
        "cargo-workspace-audit",
        "test-tier-plan",
        "test-runtime-budget",
        "slow-test-inventory",
        "cli-smoke-tiering",
        "developer-speed-runbook",
        "workspace-acceptance-v2",
    ] {
        let remote = Command::new(bin)
            .args([command, "--config", "https://example.com/sprint76.toml"])
            .output()
            .expect("remote config");
        assert!(!remote.status.success());
        let stderr = String::from_utf8(remote.stderr).expect("stderr");
        assert!(stderr.contains("must be local"));
    }
}
