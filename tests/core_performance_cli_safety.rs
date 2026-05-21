mod common;

use std::fs;
use std::process::Command;

use soma_zero::{
    CorePerformanceRegressionConfig, CorePerformanceRegressionSummary,
    CorePerformanceScorecardConfig,
};

#[test]
fn core_performance_cli_help_mentions_research_only_and_no_live_commands() {
    for command in ["core-performance", "core-bottleneck", "core-regression"] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("help");
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains("Research-only"));
        assert!(!text.contains("live-trade"));
        assert!(!text.contains("\n  broker"));
        assert!(!text.contains("\n  account"));
        assert!(!text.contains("\n  live"));
        assert!(!text.contains("runtime-llm"));
    }
}

#[test]
fn core_performance_cli_rejects_remote_paths() {
    for (command, expected) in [
        (
            "core-performance",
            "core-performance config path must be local",
        ),
        (
            "core-bottleneck",
            "core-bottleneck config path must be local",
        ),
        (
            "core-regression",
            "core-regression config path must be local",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("remote");
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected));
    }
}

#[test]
fn core_performance_cli_runs_with_local_configs() {
    let performance_config = CorePerformanceScorecardConfig {
        scorecard_id: "core-performance-cli".to_string(),
        output_root: common::output_dir("core-performance-cli-root")
            .display()
            .to_string(),
        ..CorePerformanceScorecardConfig::default()
    };
    let performance_config_path =
        common::output_dir("core-performance-cli-config").join("core_performance.toml");
    fs::write(
        &performance_config_path,
        performance_config.to_toml_string().expect("scorecard toml"),
    )
    .expect("write scorecard toml");

    let performance = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "core-performance",
            "--config",
            &performance_config_path.display().to_string(),
        ])
        .output()
        .expect("core-performance");
    assert!(performance.status.success());
    assert!(String::from_utf8_lossy(&performance.stdout).contains("research_only_warning"));

    let bottleneck = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "core-bottleneck",
            "--config",
            &performance_config_path.display().to_string(),
        ])
        .output()
        .expect("core-bottleneck");
    assert!(bottleneck.status.success());
    assert!(String::from_utf8_lossy(&bottleneck.stdout).contains("primary_bottleneck="));

    let current_summary = CorePerformanceRegressionSummary {
        scorecard_id: "core-performance-cli".to_string(),
        official_row_count: 0,
        outcome_linked_rows: 0,
        counterfactual_rows: 0,
        brier_score: None,
        ece: None,
        denial_rate: 0.0,
        avoided_loss_total: 0.0,
        actionability_ratio: None,
        report_bytes: 32,
        fingerprint: "cli-fingerprint".to_string(),
    };
    let summary_path = common::output_dir("core-regression-cli-summary").join("summary.json");
    current_summary
        .to_json_path(&summary_path)
        .expect("write summary");
    let regression_config = CorePerformanceRegressionConfig {
        current_scorecard_path: Some(summary_path.display().to_string()),
        ..CorePerformanceRegressionConfig::default()
    };
    let regression_config_path =
        common::output_dir("core-regression-cli-config").join("core_regression.toml");
    fs::write(
        &regression_config_path,
        regression_config.to_toml_string().expect("regression toml"),
    )
    .expect("write regression toml");

    let regression = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "core-regression",
            "--config",
            &regression_config_path.display().to_string(),
        ])
        .output()
        .expect("core-regression");
    assert!(regression.status.success());
    assert!(String::from_utf8_lossy(&regression.stdout).contains("comparable=false"));
}
