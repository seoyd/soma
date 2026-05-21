mod common;

use std::fs;
use std::process::Command;

use soma_zero::{CoreCheckConfig, RuntimeMode};

#[test]
fn core_check_help_contains_research_only_warning() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["core-check", "--help"])
        .output()
        .expect("run core-check --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Research-only core hardening report"));
}

#[test]
fn core_check_emits_deterministic_report() {
    let config = CoreCheckConfig {
        check_id: "core-check-deterministic".to_string(),
        output_root: common::output_dir("core-check-out").display().to_string(),
        runtime_mode: RuntimeMode::Research,
        ..CoreCheckConfig::default()
    };
    let config_dir = common::output_dir("core-check-config");
    let config_path = config_dir.join("core_check.toml");
    fs::write(
        &config_path,
        config
            .to_toml_string()
            .expect("serialize core-check config"),
    )
    .expect("write config");

    let first = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["core-check", "--config", &config_path.display().to_string()])
        .output()
        .expect("run core-check first");
    let second = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["core-check", "--config", &config_path.display().to_string()])
        .output()
        .expect("run core-check second");

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn cli_has_no_live_order_broker_or_account_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("run --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\n  live"));
    assert!(!stdout.contains("\n  order"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
}
