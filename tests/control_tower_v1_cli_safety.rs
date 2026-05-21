use std::process::Command;

#[test]
fn sprint54_cli_help_has_safe_local_only_warnings() {
    for command in [
        "control-tower-v1",
        "dashboard-action-drafts",
        "dashboard-open",
        "dashboard-serve",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("help output");
        let text = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
        assert!(text.contains("local") || text.contains("localhost"));
        assert!(
            text.contains("read-only")
                || text.contains("paper-only")
                || text.contains("local-file")
                || text.contains("never auto-applies")
                || text.contains("get-only")
        );
    }
}

#[test]
fn sprint54_cli_rejects_remote_paths_and_has_no_unsafe_global_commands() {
    for command in [
        "control-tower-v1",
        "dashboard-action-drafts",
        "dashboard-open",
        "dashboard-serve",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/x.toml"])
            .output()
            .expect("output");
        assert!(!output.status.success());
    }
    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let text = String::from_utf8_lossy(&help.stdout).to_ascii_lowercase();
    assert!(text.contains("control-tower-v1"));
    assert!(text.contains("dashboard-action-drafts"));
    assert!(text.contains("dashboard-open"));
    assert!(text.contains("dashboard-serve"));
    for forbidden in [
        "live-trade",
        "broker-order",
        "account-balance",
        "runtime-llm",
    ] {
        assert!(!text.contains(forbidden));
    }
}
