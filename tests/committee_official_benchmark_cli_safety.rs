use std::process::Command;

#[test]
fn official_cli_help_contains_research_only_warning() {
    for command in [
        "committee-pack-official",
        "committee-link-outcomes",
        "committee-official-benchmark",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("help");
        assert!(String::from_utf8_lossy(&output.stdout).contains("Research-only"));
    }
}

#[test]
fn official_cli_rejects_remote_paths_and_has_no_live_commands() {
    let pack = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "committee-pack-official",
            "--config",
            "https://example.com/pack.toml",
        ])
        .output()
        .expect("pack");
    let link = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "committee-link-outcomes",
            "--config",
            "https://example.com/link.toml",
        ])
        .output()
        .expect("link");
    let benchmark = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "committee-official-benchmark",
            "--config",
            "https://example.com/benchmark.toml",
        ])
        .output()
        .expect("benchmark");
    assert!(String::from_utf8_lossy(&pack.stderr).contains("must be local"));
    assert!(String::from_utf8_lossy(&link.stderr).contains("must be local"));
    assert!(String::from_utf8_lossy(&benchmark.stderr).contains("must be local"));

    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["committee-official-benchmark", "--help"])
        .output()
        .expect("help");
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(!text.contains("live-trade"));
    assert!(!text.contains("broker-order"));
    assert!(!text.contains("runtime-llm"));
    assert!(!text.contains("mamba"));
}
