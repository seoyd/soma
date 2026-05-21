mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use std::process::Command;

use soma_zero::OfficialEvidenceReplicationConfig;

#[test]
fn official_replication_help_lists_sprint39_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("official-replication"));
    assert!(stdout.contains("official-artifact-inventory"));
    assert!(stdout.contains("official-row-inject"));
    assert!(stdout.contains("Research-only"));
}

#[test]
fn official_replication_commands_reject_remote_configs() {
    for command in [
        "official-replication",
        "official-artifact-inventory",
        "official-row-inject",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/official.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}

#[test]
fn official_inventory_and_row_inject_commands_print_research_only_warnings() {
    let config = OfficialEvidenceReplicationConfig {
        replication_id: "official-cli".to_string(),
        output_root: common::output_dir("official-cli-root")
            .display()
            .to_string(),
        require_preflight: false,
        require_provenance: false,
        require_local_candles: false,
        ..OfficialEvidenceReplicationConfig::default()
    };
    let config_path = official_committee_support::write_replication_config("official-cli", &config);

    let inventory = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "official-artifact-inventory",
            "--config",
            &config_path.display().to_string(),
        ])
        .output()
        .expect("inventory");
    assert!(inventory.status.success());
    assert!(String::from_utf8_lossy(&inventory.stdout).contains("research_only_warning"));

    let row_inject = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "official-row-inject",
            "--config",
            &config_path.display().to_string(),
        ])
        .output()
        .expect("row inject");
    assert!(row_inject.status.success());
    assert!(String::from_utf8_lossy(&row_inject.stdout).contains("research_only_warning"));
}
