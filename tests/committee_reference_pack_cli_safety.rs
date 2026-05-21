mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use std::process::Command;

#[test]
fn committee_reference_pack_help_contains_research_only_warning() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("committee-build-references"));
    assert!(stdout.contains("committee-align-candles"));
    assert!(stdout.contains("committee-sufficiency-close"));
    assert!(stdout.contains("Research-only"));
}

#[test]
fn committee_reference_pack_commands_reject_remote_config_paths_and_hide_live_commands() {
    for command in [
        "committee-build-references",
        "committee-align-candles",
        "committee-sufficiency-close",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([
                command,
                "--config",
                "https://example.com/reference-pack.toml",
            ])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }

    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(!stdout.contains("\n  live"));
    assert!(!stdout.contains("\n  order"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
    assert!(!stdout.contains("\n  llm"));
    assert!(!stdout.contains("mamba-runtime"));
}
