use std::process::Command;

#[test]
fn committee_v1_help_contains_research_only_warning() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["committee-v1", "--help"])
        .output()
        .expect("run help");
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Research-only"));
}

#[test]
fn no_live_or_broker_cli_commands_exist() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("run help");
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(!text.contains("live-trade"));
    assert!(!text.contains("broker-order"));
    assert!(!text.contains("account-balance"));
    assert!(!text.contains("runtime-llm"));
}

#[test]
fn committee_v1_remote_paths_are_rejected_by_cli() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["committee-v1", "--config", "https://example.com/run.toml"])
        .output()
        .expect("run command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("committee-v1 config path must be local"));
}
