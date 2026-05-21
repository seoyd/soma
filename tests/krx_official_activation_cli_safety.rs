use std::process::Command;

#[test]
fn root_help_lists_krx_commands_and_research_only_text() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("run help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("krx-auth-readiness"));
    assert!(stdout.contains("krx-symbol-whitelist"));
    assert!(stdout.contains("krx-evidence-plan"));
    assert!(stdout.contains("krx-official-activate"));
    assert!(stdout.contains("Research-only"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
}

#[test]
fn krx_subcommand_help_mentions_research_only_scope() {
    for command in [
        "krx-auth-readiness",
        "krx-symbol-whitelist",
        "krx-evidence-plan",
        "krx-official-activate",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("run subcommand help");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Research-only") || stdout.contains("market-data-only"));
    }
}

#[test]
fn krx_commands_reject_remote_config_paths() {
    for command in [
        "krx-auth-readiness",
        "krx-symbol-whitelist",
        "krx-evidence-plan",
        "krx-official-activate",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run remote path check");
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("must be local"));
    }
}
