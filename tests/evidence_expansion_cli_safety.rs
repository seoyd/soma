mod common;

use std::fs;
use std::process::Command;

use soma_zero::{
    OfficialEvidenceExpansionConfig, ProviderAuthPreflightConfig, VenueCoverageExpansionPlan,
};

#[test]
fn cli_help_exposes_expansion_commands_as_research_only() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("run --help");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("provider-auth-check"));
    assert!(stdout.contains("evidence-expand"));
    assert!(stdout.contains("official-coverage"));
    assert!(stdout.contains("Research-only"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
}

#[test]
fn provider_auth_check_rejects_remote_config_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "provider-auth-check",
            "--config",
            "https://example.com/auth.toml",
        ])
        .output()
        .expect("run provider-auth-check");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("provider-auth-check config path must be local")
    );
}

#[test]
fn evidence_expand_rejects_remote_config_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "evidence-expand",
            "--config",
            "https://example.com/expand.toml",
        ])
        .output()
        .expect("run evidence-expand");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("evidence-expand config path must be local")
    );
}

#[test]
fn provider_auth_check_runs_with_local_config() {
    let config_path = common::output_dir("provider-auth-check-cli").join("auth.toml");
    fs::write(
        &config_path,
        ProviderAuthPreflightConfig::default()
            .to_toml_string()
            .expect("serialize config"),
    )
    .expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "provider-auth-check",
            "--config",
            &config_path.display().to_string(),
        ])
        .output()
        .expect("run provider-auth-check");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("check_id="));
}

#[test]
fn official_coverage_runs_with_local_config() {
    let config_path = common::output_dir("official-coverage-cli").join("coverage.toml");
    fs::write(
        &config_path,
        VenueCoverageExpansionPlan::default()
            .to_toml_string()
            .expect("serialize plan"),
    )
    .expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "official-coverage",
            "--config",
            &config_path.display().to_string(),
        ])
        .output()
        .expect("run official-coverage");

    assert!(output.status.success());
}

#[test]
fn evidence_expand_runs_with_local_config() {
    let config_path = common::output_dir("evidence-expand-cli").join("expand.toml");
    fs::write(
        &config_path,
        OfficialEvidenceExpansionConfig {
            run_auth_preflight: false,
            run_core_benchmark: false,
            ..OfficialEvidenceExpansionConfig::default()
        }
        .to_toml_string()
        .expect("serialize config"),
    )
    .expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "evidence-expand",
            "--config",
            &config_path.display().to_string(),
        ])
        .output()
        .expect("run evidence-expand");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("expansion_id="));
}
