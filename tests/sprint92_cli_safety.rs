use std::process::Command;

#[test]
fn sprint92_help_exposes_only_research_local_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "sprint92-krx-warning-close",
        "krx-warning-closure",
        "krx-secret-safety-isolation",
        "krx-raw-archive-redaction-coverage",
        "krx-genuine-reduction-gate",
        "dashboard-renderer-entry-gate",
        "control-tower-krx-warning-closure",
    ] {
        assert!(stdout.contains(command), "missing command {command}");
    }
    for forbidden in [
        "\n  live",
        "\n  order",
        "\n  broker",
        "\n  account",
        "mamba-runtime",
    ] {
        assert!(!stdout.contains(forbidden));
    }
}

#[test]
fn sprint92_commands_reject_remote_paths() {
    for command in [
        "sprint92-krx-warning-close",
        "krx-warning-closure",
        "krx-secret-safety-isolation",
        "krx-raw-archive-redaction-coverage",
        "krx-genuine-reduction-gate",
        "dashboard-renderer-entry-gate",
        "control-tower-krx-warning-closure",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
