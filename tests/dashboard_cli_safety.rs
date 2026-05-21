use std::process::Command;

#[test]
fn dashboard_cli_help_contains_research_and_read_only_warnings() {
    let provider = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["provider-simplify", "--help"])
        .output()
        .expect("help");
    let provider_text = String::from_utf8_lossy(&provider.stdout);
    assert!(provider_text.contains("Research-only"));

    let snapshot = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["dashboard-snapshot", "--help"])
        .output()
        .expect("help");
    let snapshot_text = String::from_utf8_lossy(&snapshot.stdout);
    assert!(snapshot_text.contains("Read-only"));

    let render = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["dashboard-render", "--help"])
        .output()
        .expect("help");
    let render_text = String::from_utf8_lossy(&render.stdout);
    assert!(render_text.contains("Read-only"));
}

#[test]
fn dashboard_cli_rejects_remote_paths_and_has_no_live_trade_commands() {
    for args in [
        vec![
            "provider-simplify",
            "--config",
            "https://example.com/provider.toml",
        ],
        vec![
            "dashboard-snapshot",
            "--config",
            "https://example.com/dashboard.toml",
        ],
        vec![
            "dashboard-render",
            "--config",
            "https://example.com/render.toml",
        ],
        vec![
            "dashboard-open",
            "--config",
            "https://example.com/open.toml",
        ],
        vec![
            "dashboard-serve",
            "--config",
            "https://example.com/serve.toml",
        ],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args(args)
            .output()
            .expect("run");
        assert!(!output.status.success());
    }
    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("provider-simplify"));
    assert!(text.contains("dashboard-snapshot"));
    assert!(text.contains("dashboard-render"));
    assert!(!text.contains("live-trade"));
    assert!(!text.contains("broker-order"));
    assert!(!text.contains("account-balance"));
    assert!(text.contains("dashboard-open"));
    assert!(text.contains("dashboard-serve"));
}
