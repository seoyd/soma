#[path = "support/candle_expansion_support.rs"]
mod candle_expansion_support;
mod common;

use std::process::Command;

use soma_zero::OfficialCandleExpansionPlanConfig;

#[test]
fn candle_expansion_cli_help_contains_research_only_warning_and_no_live_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("candle-gap-map"));
    assert!(stdout.contains("candle-expansion-plan"));
    assert!(stdout.contains("candle-expand"));
    assert!(stdout.contains("Research-only"));
    assert!(!stdout.contains("\n  live"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
    assert!(!stdout.contains("\n  order"));
    assert!(!stdout.contains("\n  llm"));
    assert!(!stdout.contains("mamba-runtime"));
}

#[test]
fn candle_expansion_cli_rejects_remote_paths() {
    for command in [
        "candle-gap-map",
        "candle-expansion-plan",
        "candle-expansion-actions",
        "candle-expand",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/remote.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("must be local"));
    }
}

#[test]
fn candle_expansion_cli_commands_print_research_only_warning() {
    let gap_map = candle_expansion_support::manual_gap_map_path(
        "cli-expansion",
        soma_zero::ProviderMarket::USEquity,
        "AAPL",
        "1d",
        soma_zero::ComparableEvidenceSourceClass::OfficialNonCrypto,
        Vec::new(),
    );
    let plan_path =
        candle_expansion_support::plan_config_path(&OfficialCandleExpansionPlanConfig {
            plan_id: "cli-expansion".to_string(),
            gap_map_path: Some(gap_map.display().to_string()),
            output_root: common::output_dir("cli-expansion-out")
                .display()
                .to_string(),
            ..OfficialCandleExpansionPlanConfig::default()
        });

    let plan = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "candle-expansion-plan",
            "--config",
            &plan_path.display().to_string(),
        ])
        .output()
        .expect("plan");
    assert!(plan.status.success());
    assert!(String::from_utf8_lossy(&plan.stdout).contains("research_only_warning"));

    let actions = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "candle-expansion-actions",
            "--config",
            &plan_path.display().to_string(),
        ])
        .output()
        .expect("actions");
    assert!(actions.status.success());
    assert!(String::from_utf8_lossy(&actions.stdout).contains("research_only_warning"));

    let expand = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "candle-expand",
            "--config",
            &plan_path.display().to_string(),
        ])
        .output()
        .expect("expand");
    assert!(expand.status.success());
    assert!(String::from_utf8_lossy(&expand.stdout).contains("research_only_warning"));
}
