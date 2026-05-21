use std::process::Command;

#[test]
fn owner_cli_help_contains_research_only_and_paper_only_warnings() {
    for command in [
        "owner-input-validate",
        "owner-review-queue",
        "owner-apply-input",
        "owner-impact-report",
        "owner-thesis-book",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("help output");
        let text = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
        assert!(text.contains("research-only"));
        assert!(text.contains("paper-only") || text.contains("paper only"));
        assert!(text.contains("local"));
    }
}

#[test]
fn owner_cli_rejects_remote_paths_and_owner_help_has_no_live_order_account_commands() {
    for command in [
        "owner-input-validate",
        "owner-review-queue",
        "owner-apply-input",
        "owner-impact-report",
        "owner-thesis-book",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/owner.toml"])
            .output()
            .expect("command output");
        assert!(!output.status.success());
    }
    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let text = String::from_utf8_lossy(&help.stdout).to_ascii_lowercase();
    assert!(text.contains("owner-input-validate"));
    assert!(!text.contains("live-trade"));
    assert!(!text.contains("broker-order"));
    assert!(!text.contains("account-balance"));
}
