mod common;

use std::fs;
use std::process::Command;

use soma_zero::{CommitteeCounterfactualAuditConfig, CommitteeOutcomeCoverageConfig};

#[test]
fn sprint37_cli_help_mentions_research_only_and_has_no_live_commands() {
    for command in [
        "committee-outcome-coverage",
        "committee-counterfactual-audit",
        "committee-performance-matrix",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("help");
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains("Research-only"));
        assert!(!text.contains("live-trade"));
        assert!(!text.contains("broker"));
        assert!(!text.contains("account"));
        assert!(!text.contains("runtime-llm"));
        assert!(!text.contains("mamba"));
    }
}

#[test]
fn sprint37_cli_rejects_remote_paths_and_accepts_local_configs() {
    for (command, flag) in [
        (
            "committee-outcome-coverage",
            "committee-outcome-coverage config path must be local",
        ),
        (
            "committee-counterfactual-audit",
            "committee-counterfactual-audit config path must be local",
        ),
        (
            "committee-performance-matrix",
            "committee-performance-matrix config path must be local",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("remote");
        assert!(String::from_utf8_lossy(&output.stderr).contains(flag));
    }

    let coverage_path = common::output_dir("coverage-cli-local").join("coverage.toml");
    fs::write(
        &coverage_path,
        CommitteeOutcomeCoverageConfig::default()
            .to_toml_string()
            .expect("toml"),
    )
    .expect("write coverage");
    let audit_path = common::output_dir("audit-cli-local").join("audit.toml");
    fs::write(
        &audit_path,
        CommitteeCounterfactualAuditConfig::default()
            .to_toml_string()
            .expect("toml"),
    )
    .expect("write audit");

    assert!(
        Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([
                "committee-counterfactual-audit",
                "--config",
                &audit_path.display().to_string()
            ])
            .output()
            .expect("audit")
            .status
            .success()
    );
    assert!(
        Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([
                "committee-performance-matrix",
                "--config",
                &coverage_path.display().to_string()
            ])
            .output()
            .expect("matrix")
            .status
            .success()
    );
}
