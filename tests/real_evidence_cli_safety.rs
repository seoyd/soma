use std::process::Command;

#[test]
fn real_evidence_help_is_local_only_and_has_no_live_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("cli help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("real-evidence"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  live"));
    assert!(!stdout.contains("\n  execute"));
}

#[test]
fn remote_real_evidence_config_path_is_rejected() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["real-evidence", "--config", "https://example.com/real.toml"])
        .output()
        .expect("run cli");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("real-evidence config path must be local"));
}
