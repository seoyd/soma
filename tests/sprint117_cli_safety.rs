use std::process::Command;

#[test]
fn sprint117_cli_help_has_required_warnings_and_no_forbidden_commands() {
    let binary = env!("CARGO_BIN_EXE_soma_experiment");
    let help = Command::new(binary)
        .arg("--help")
        .output()
        .expect("run top-level help");
    assert!(help.status.success());
    let help_text = String::from_utf8_lossy(&help.stdout);
    for required in [
        "sprint117-deferred-real-observation",
        "sprint116-baseline-truth-import",
        "acceptance-truth-gate-v18",
        "control-tower-acceptance-truth-v18",
    ] {
        assert!(help_text.contains(required));
    }
    for forbidden in [
        "
  train-model",
        "
  live-inference",
        "
  mamba-runtime",
        "
  gated-runtime",
        "
  broker",
        "
  order",
        "
  account",
    ] {
        assert!(
            !help_text.contains(forbidden),
            "unexpected command: {forbidden}"
        );
    }
    let sprint_help = Command::new(binary)
        .args(["sprint117-deferred-real-observation", "--help"])
        .output()
        .expect("run sprint117 help");
    assert!(
        String::from_utf8_lossy(&sprint_help.stdout).contains("deferred-real-observation-only")
    );
    let no_run_help = Command::new(binary)
        .args(["real-no-run-execution-v18", "--help"])
        .output()
        .expect("run no-run help");
    assert!(String::from_utf8_lossy(&no_run_help.stdout).contains("no-run-is-not-full"));
    let cargo_json_help = Command::new(binary)
        .args(["real-cargo-json-execution-v18", "--help"])
        .output()
        .expect("run cargo json help");
    assert!(
        String::from_utf8_lossy(&cargo_json_help.stdout).contains("cargo-json-is-not-acceptance")
    );
    let truth_help = Command::new(binary)
        .args(["acceptance-truth-gate-v18", "--help"])
        .output()
        .expect("run truth help");
    assert!(
        String::from_utf8_lossy(&truth_help.stdout)
            .contains("only full finished and passed workspace tests can claim full acceptance")
    );
    let remote = Command::new(binary)
        .args([
            "sprint117-deferred-real-observation",
            "--config",
            "https://example.invalid/config.toml",
        ])
        .output()
        .expect("run remote config rejection");
    assert!(!remote.status.success());
}
