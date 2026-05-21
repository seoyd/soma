use std::process::Command;

#[test]
fn committee_materialize_and_benchmark_help_warn_research_only() {
    let materialize = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["committee-materialize", "--help"])
        .output()
        .expect("materialize help");
    let benchmark = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["committee-benchmark", "--help"])
        .output()
        .expect("benchmark help");
    assert!(String::from_utf8_lossy(&materialize.stdout).contains("Research-only"));
    assert!(String::from_utf8_lossy(&benchmark.stdout).contains("Research-only"));
}

#[test]
fn benchmark_cli_rejects_remote_paths_and_has_no_live_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "committee-benchmark",
            "--config",
            "https://example.com/run.toml",
        ])
        .output()
        .expect("run benchmark");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("committee-benchmark config path must be local")
    );
    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["committee-benchmark", "--help"])
        .output()
        .expect("help");
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(!text.contains("live-trade"));
    assert!(!text.contains("broker-order"));
    assert!(!text.contains("runtime-llm"));
    assert!(!text.contains("mamba"));
}
