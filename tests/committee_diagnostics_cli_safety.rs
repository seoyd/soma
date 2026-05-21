mod common;

use std::fs;
use std::process::Command;

use soma_zero::{
    CommitteeDiagnosticsConfig, CommitteeScenarioLoadConfig, CommitteeScenarioSourceKind,
    ReasonCode,
};

#[test]
fn diagnostics_help_contains_research_only_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("committee-load-scenarios"));
    assert!(stdout.contains("committee-replay"));
    assert!(stdout.contains("committee-diagnostics"));
    assert!(stdout.contains("Research-only"));
}

#[test]
fn diagnostics_commands_reject_remote_paths_and_no_live_commands_exist() {
    for command in [
        "committee-load-scenarios",
        "committee-replay",
        "committee-diagnostics",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
    }
    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(!stdout.contains("\n  live"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
    assert!(!stdout.contains("\n  llm"));
    assert!(!stdout.contains("mamba-runtime"));
}

#[test]
fn diagnostics_cli_runs_on_local_fixture_config() {
    let load_path = common::output_dir("committee-diagnostics-cli").join("load.toml");
    fs::write(
        &load_path,
        CommitteeScenarioLoadConfig {
            scenario_id: "committee-cli-scenarios".to_string(),
            source_kind: CommitteeScenarioSourceKind::Fixture,
            output_root: common::output_dir("committee-diagnostics-cli-out")
                .display()
                .to_string(),
            reason_codes: vec![ReasonCode::CommitteeScenarioLoaderBuilt],
            ..CommitteeScenarioLoadConfig::default()
        }
        .to_toml_string()
        .expect("toml"),
    )
    .expect("write");
    let diag_path = common::output_dir("committee-diagnostics-cli2").join("diagnostics.toml");
    fs::write(
        &diag_path,
        CommitteeDiagnosticsConfig {
            diagnostic_id: "committee-cli-diagnostics".to_string(),
            scenario_load_config_path: Some(load_path.display().to_string()),
            output_root: common::output_dir("committee-diagnostics-cli-final")
                .display()
                .to_string(),
            reason_codes: vec![ReasonCode::CommitteeDiagnosticsBuilt],
            ..CommitteeDiagnosticsConfig::default()
        }
        .to_toml_string()
        .expect("toml"),
    )
    .expect("write");
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "committee-diagnostics",
            "--config",
            &diag_path.display().to_string(),
        ])
        .output()
        .expect("run");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("final_status="));
}
