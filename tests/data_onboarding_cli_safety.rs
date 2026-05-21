use std::process::Command;

#[test]
fn cli_help_exposes_local_only_onboarding_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("run --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("data-preflight"));
    assert!(stdout.contains("onboard-data"));
}

#[test]
fn data_preflight_rejects_remote_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "data-preflight",
            "--input",
            "https://example.com/data.csv",
            "--out",
            "target/cli-preflight",
            "--symbol",
            "BTC-USDT",
            "--timeframe",
            "1m",
        ])
        .output()
        .expect("run data-preflight");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("data-preflight paths must be local"));
}

#[test]
fn onboard_data_rejects_remote_config_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "onboard-data",
            "--config",
            "https://example.com/onboarding.toml",
        ])
        .output()
        .expect("run onboard-data");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("onboard-data config path must be local"));
}
