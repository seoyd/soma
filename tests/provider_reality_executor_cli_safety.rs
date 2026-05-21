mod common;

use std::fs;
use std::process::Command;

use soma_zero::{EvidenceLaneKind, ExecutableEvidencePlanConfig, ExplicitEvidenceLaneConfig};

#[test]
fn evidence_executor_help_mentions_research_only_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("evidence-plan"));
    assert!(stdout.contains("evidence-execute"));
    assert!(stdout.contains("readiness-matrix"));
    assert!(stdout.contains("Research-only"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
}

#[test]
fn evidence_commands_reject_remote_config_paths() {
    for command in ["evidence-plan", "evidence-execute", "readiness-matrix"] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}

#[test]
fn evidence_execute_runs_without_network_and_never_prints_secrets() {
    let config_path = common::output_dir("executor-cli").join("config.toml");
    fs::write(
        &config_path,
        ExecutableEvidencePlanConfig {
            explicit_lanes: vec![ExplicitEvidenceLaneConfig {
                lane_kind: EvidenceLaneKind::CryptoIntradayEvidence,
                provider: "upbit".to_string(),
                symbols: vec!["BTC-KRW".to_string()],
                enabled: true,
                output_subdir: None,
                max_rows: None,
                max_requests: None,
                allow_full_history: false,
                allow_all_symbols: false,
                reason_codes: vec![],
            }],
            output_root: common::output_dir("executor-cli-out").display().to_string(),
            allow_yfinance_research: false,
            ..ExecutableEvidencePlanConfig::default()
        }
        .to_toml_string()
        .expect("toml"),
    )
    .expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "evidence-execute",
            "--config",
            &config_path.display().to_string(),
        ])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("final_status=CryptoOnlyRan"));
    assert!(!stdout.contains("sk-live-"));
    assert!(!stdout.contains("secret"));
}
