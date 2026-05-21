use std::process::Command;

#[test]
fn sprint48_cli_help_contains_research_only_warning() {
    for command in [
        "barrier-profiles",
        "official-diversity-gap-map",
        "official-diversity-row-select",
        "outcome-diversity-audit",
        "balanced-outcome-coverage",
        "diversity-sufficiency-v2",
        "official-evidence-diversity-sweep",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("help output");
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Research-only"));
    }
}

#[test]
fn sprint48_cli_rejects_remote_paths_and_has_no_live_commands() {
    for command in [
        "barrier-profiles",
        "official-diversity-gap-map",
        "official-diversity-row-select",
        "outcome-diversity-audit",
        "balanced-outcome-coverage",
        "diversity-sufficiency-v2",
        "official-evidence-diversity-sweep",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("remote path rejection");
        assert!(String::from_utf8_lossy(&output.stderr).contains("must be local"));
    }

    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("global help");
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout).to_ascii_lowercase();
    assert!(!text.contains("\n  live"));
    assert!(!text.contains("\n  order"));
    assert!(!text.contains("\n  broker"));
    assert!(!text.contains("\n  account"));
    assert!(!text.contains("runtime-llm"));
    assert!(!text.contains("mamba-runtime"));
}
