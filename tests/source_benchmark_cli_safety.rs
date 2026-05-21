mod common;

use std::fs;
use std::process::Command;

use soma_zero::SourceAwareBenchmarkConfig;

#[test]
fn source_benchmark_help_mentions_research_only() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("source-benchmark"));
    assert!(stdout.contains("Research-only"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
}

#[test]
fn source_benchmark_rejects_remote_config_path() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "source-benchmark",
            "--config",
            "https://example.com/source.toml",
        ])
        .output()
        .expect("run");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("source-benchmark config path must be local")
    );
}

#[test]
fn source_benchmark_runs_with_local_config() {
    let config_path = common::output_dir("source-benchmark-cli").join("config.toml");
    fs::write(
        &config_path,
        SourceAwareBenchmarkConfig {
            benchmark_id: "cli-source-benchmark".to_string(),
            allow_yfinance_only_research: true,
            ..SourceAwareBenchmarkConfig::default()
        }
        .to_toml_string()
        .expect("serialize"),
    )
    .expect("write");
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "source-benchmark",
            "--config",
            &config_path.display().to_string(),
        ])
        .output()
        .expect("run");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("final_status="));
}
