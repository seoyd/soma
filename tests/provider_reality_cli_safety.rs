mod common;

use std::fs;
use std::process::Command;

use soma_zero::{ProviderRealityConfig, StrategyDataCheckRequest, StrategyUseCase};

#[test]
fn provider_reality_help_mentions_research_only_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("provider-reality"));
    assert!(stdout.contains("strategy-data-check"));
    assert!(stdout.contains("provider-recommend"));
    assert!(stdout.contains("Research-only"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
}

#[test]
fn provider_reality_rejects_remote_config_path() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "provider-reality",
            "--config",
            "https://example.com/provider-reality.toml",
        ])
        .output()
        .expect("run");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("provider-reality config path must be local")
    );
}

#[test]
fn strategy_data_check_runs_without_network() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "strategy-data-check",
            "--provider",
            "yfinance",
            "--use-case",
            "source-comparison",
        ])
        .output()
        .expect("run");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("compatible=true"));
}

#[test]
fn provider_recommend_runs_without_network() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "provider-recommend",
            "--market",
            "us-equity",
            "--use-case",
            "realtime-scalping",
            "--budget",
            "free-only",
        ])
        .output()
        .expect("run");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("status="));
}

#[test]
fn provider_reality_never_prints_secret_values() {
    let config_path = common::output_dir("provider-reality-cli").join("config.toml");
    fs::write(
        &config_path,
        ProviderRealityConfig {
            strategy_checks: vec![StrategyDataCheckRequest {
                provider: "alphavantage".to_string(),
                use_case: StrategyUseCase::EodSwing,
            }],
            ..ProviderRealityConfig::default()
        }
        .to_toml_string()
        .expect("serialize"),
    )
    .expect("write");
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "provider-reality",
            "--config",
            &config_path.display().to_string(),
        ])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("super-secret-value"));
    assert!(!stdout.contains("sk-live-"));
}
