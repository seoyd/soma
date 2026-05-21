mod common;

use std::process::Command;

#[test]
fn official_ready_match_cli_help_contains_research_only_commands_and_no_live_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("candle-join-audit"));
    assert!(stdout.contains("candle-join-repair-plan"));
    assert!(stdout.contains("official-ready-match-close"));
    assert!(stdout.contains("candle-lineage"));
    assert!(stdout.contains("Research-only"));
    assert!(!stdout.contains("\n  live"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
    assert!(!stdout.contains("\n  order"));
    assert!(!stdout.contains("mamba-runtime"));
}

#[test]
fn official_ready_match_cli_rejects_remote_paths() {
    for command in [
        "candle-join-audit",
        "candle-join-repair-plan",
        "official-ready-match-close",
        "candle-lineage",
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
fn official_ready_match_cli_commands_print_research_only_warning() {
    let audit = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "candle-join-audit",
            "--config",
            "examples/soma_candle_join_audit_symbol_mismatch.toml",
        ])
        .output()
        .expect("audit");
    assert!(audit.status.success());
    assert!(String::from_utf8_lossy(&audit.stdout).contains("research_only_warning"));

    let repair = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "candle-join-repair-plan",
            "--config",
            "examples/soma_candle_join_audit_symbol_mismatch.toml",
        ])
        .output()
        .expect("repair");
    assert!(repair.status.success());
    assert!(String::from_utf8_lossy(&repair.stdout).contains("research_only_warning"));

    let close = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "official-ready-match-close",
            "--config",
            "examples/soma_official_ready_match_close_official_replication.toml",
        ])
        .output()
        .expect("close");
    assert!(close.status.success());
    assert!(String::from_utf8_lossy(&close.stdout).contains("research_only_warning"));
}
