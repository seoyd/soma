use std::process::Command;

#[test]
fn sprint78_cli_help_and_local_only_guards_are_present() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    let expected = [
        ("core-completion-v2", "research-only"),
        ("mamba3fin-core-contract", "contract-only"),
        ("mamba3-runtime-readiness", "runtime deferred"),
        ("committee-completion-gate", "no expansion"),
        ("committee-materialization-plan-v2", "no persona expansion"),
        ("training-data-storage-decision", "no training"),
        ("training-data-registry-spec", "storage spec only"),
        ("training-data-layout-plan", "runtime behavior"),
        ("training-data-lineage-spec", "lineage contract"),
        ("mamba3-implementation-roadmap", "staged/deferred"),
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
    assert!(root_stdout.contains("core-completion-v2"));
    assert!(root_stdout.contains("mamba3fin-core-contract"));
    assert!(root_stdout.contains("training-data-storage-decision"));
    assert!(!root_stdout.contains("train-model"));
    assert!(!root_stdout.contains("live-inference"));
    assert!(!root_stdout.contains("order-account"));

    for command in [
        "core-completion-v2",
        "mamba3fin-core-contract",
        "mamba3-runtime-readiness",
        "committee-completion-gate",
        "committee-materialization-plan-v2",
        "training-data-storage-decision",
        "training-data-registry-spec",
        "training-data-layout-plan",
        "training-data-lineage-spec",
        "mamba3-implementation-roadmap",
    ] {
        let remote = Command::new(bin)
            .args([command, "--config", "https://example.com/sprint78.toml"])
            .output()
            .expect("remote config");
        assert!(!remote.status.success());
        let stderr = String::from_utf8(remote.stderr).expect("stderr");
        assert!(stderr.contains("must be local"));
    }
}
