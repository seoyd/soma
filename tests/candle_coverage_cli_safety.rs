#[path = "support/candle_coverage_support.rs"]
mod candle_coverage_support;
mod common;

use std::fs;
use std::process::Command;

use soma_zero::{
    CandleCoverageClosureConfig, ComparableEvidenceBackfillConfig, OfficialCandleCoveragePackConfig,
};

#[test]
fn candle_coverage_cli_help_contains_research_only_warning_and_no_live_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("candle-pack"));
    assert!(stdout.contains("candle-coverage-match"));
    assert!(stdout.contains("comparable-backfill"));
    assert!(stdout.contains("candle-coverage-close"));
    assert!(stdout.contains("Research-only"));
    assert!(!stdout.contains("\n  live"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
    assert!(!stdout.contains("\n  order"));
    assert!(!stdout.contains("\n  llm"));
    assert!(!stdout.contains("mamba-runtime"));
}

#[test]
fn candle_coverage_cli_rejects_remote_configs() {
    for command in [
        "candle-pack",
        "candle-coverage-match",
        "comparable-backfill",
        "candle-coverage-close",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}

#[test]
fn candle_coverage_cli_commands_print_research_only_warnings() {
    let output_dir = common::output_dir("cli-sprint42");
    let pack_path = output_dir.join("pack.toml");
    fs::write(
        &pack_path,
        OfficialCandleCoveragePackConfig::default()
            .to_toml_string()
            .expect("toml"),
    )
    .expect("write pack");
    let backfill_path = output_dir.join("backfill.toml");
    fs::write(
        &backfill_path,
        ComparableEvidenceBackfillConfig::default()
            .to_toml_string()
            .expect("toml"),
    )
    .expect("write backfill");
    let closure_path = output_dir.join("closure.toml");
    fs::write(
        &closure_path,
        CandleCoverageClosureConfig::default()
            .to_toml_string()
            .expect("toml"),
    )
    .expect("write closure");

    let pack = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["candle-pack", "--config", &pack_path.display().to_string()])
        .output()
        .expect("pack");
    assert!(pack.status.success());
    assert!(String::from_utf8_lossy(&pack.stdout).contains("research_only_warning"));

    let backfill = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "comparable-backfill",
            "--config",
            &backfill_path.display().to_string(),
        ])
        .output()
        .expect("backfill");
    assert!(backfill.status.success());
    assert!(String::from_utf8_lossy(&backfill.stdout).contains("research_only_warning"));

    let close = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "candle-coverage-close",
            "--config",
            &closure_path.display().to_string(),
        ])
        .output()
        .expect("close");
    assert!(close.status.success());
    assert!(String::from_utf8_lossy(&close.stdout).contains("research_only_warning"));
}
