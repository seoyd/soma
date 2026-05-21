use std::process::Command;

#[test]
fn sprint47_cli_help_contains_research_only_warning() {
    for command in [
        "multi-row-official-set",
        "future-window-scaleout-plan",
        "batch-outcome-linkage-v3",
        "batch-counterfactual-complete",
        "official-evidence-sufficiency-v2",
        "official-evidence-scaleout",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("help output");
        assert!(String::from_utf8_lossy(&output.stdout).contains("Research-only"));
    }
}

#[test]
fn sprint47_cli_rejects_remote_paths_and_has_no_live_commands() {
    for command in [
        "multi-row-official-set",
        "future-window-scaleout-plan",
        "batch-outcome-linkage-v3",
        "batch-counterfactual-complete",
        "official-evidence-sufficiency-v2",
        "official-evidence-scaleout",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("remote path rejection");
        assert!(String::from_utf8_lossy(&output.stderr).contains("must be local"));
    }

    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["official-evidence-scaleout", "--help"])
        .output()
        .expect("scaleout help");
    let text = String::from_utf8_lossy(&help.stdout).to_ascii_lowercase();
    assert!(!text.contains("live-trade"));
    assert!(!text.contains("broker-order"));
    assert!(!text.contains("runtime-llm"));
    assert!(!text.contains("mamba"));
}
