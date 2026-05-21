mod common;

use std::fs;
use std::process::Command;

use soma_zero::{CommitteeSmokeTestConfig, ReasonCode};

#[test]
fn committee_help_contains_research_only_warning() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("committee-smoke"));
    assert!(stdout.contains("persona-cards"));
    assert!(stdout.contains("Research-only"));
}

#[test]
fn committee_commands_reject_remote_config_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "committee-smoke",
            "--config",
            "https://example.com/committee.toml",
        ])
        .output()
        .expect("run");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
}

#[test]
fn committee_cli_has_no_live_or_llm_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\n  live"));
    assert!(!stdout.contains("\n  order"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
    assert!(!stdout.contains("\n  llm"));
    assert!(!stdout.contains("mamba-runtime"));
}

#[test]
fn committee_smoke_cli_runs_with_local_fixture_config() {
    let config_path = common::output_dir("committee-smoke-cli").join("committee_smoke.toml");
    fs::write(
        &config_path,
        CommitteeSmokeTestConfig {
            test_id: "committee-smoke-cli".to_string(),
            require_core_check: false,
            reason_codes: vec![ReasonCode::CommitteeSmokeTestBuilt],
            ..CommitteeSmokeTestConfig::default()
        }
        .to_toml_string()
        .expect("toml"),
    )
    .expect("write");
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "committee-smoke",
            "--config",
            &config_path.display().to_string(),
        ])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("final_status="));
    assert!(stdout.contains("source_summary="));
}
