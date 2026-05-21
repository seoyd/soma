use std::process::Command;

#[test]
fn complete_row_closure_help_contains_research_only_warning() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("official-ready-row-inventory"));
    assert!(stdout.contains("scenario-materialize-v3"));
    assert!(stdout.contains("complete-row-close"));
    assert!(stdout.contains("Research-only"));
}

#[test]
fn complete_row_closure_commands_reject_remote_paths_and_hide_live_commands() {
    for command in [
        "official-ready-row-inventory",
        "scenario-materialize-v3",
        "complete-row-close",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(!stdout.contains("\n  live"));
    assert!(!stdout.contains("\n  order"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
    assert!(!stdout.contains("mamba-runtime"));
}
