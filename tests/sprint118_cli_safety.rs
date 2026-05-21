use std::process::Command;

#[test]
fn sprint118_cli_help_has_required_warnings_and_no_forbidden_commands() {
    let binary = env!("CARGO_BIN_EXE_soma_experiment");
    let help = Command::new(binary)
        .arg("--help")
        .output()
        .expect("run help");
    assert!(help.status.success());
    let help_text = String::from_utf8_lossy(&help.stdout);
    for required in [
        "sprint118-timeout-reduction-queue",
        "sprint117-baseline-truth-import",
        "acceptance-truth-gate-v19",
        "control-tower-acceptance-truth-v19",
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
        .args(["sprint118-timeout-reduction-queue", "--help"])
        .output()
        .expect("run sprint help");
    assert!(String::from_utf8_lossy(&sprint_help.stdout).contains("timeout-reduction-only"));
    let cargo_json_help = Command::new(binary)
        .args(["cargo-json-failure-reason-analysis-v1", "--help"])
        .output()
        .expect("run cargo json help");
    assert!(
        String::from_utf8_lossy(&cargo_json_help.stdout).contains("cargo JSON is supporting-only")
    );
    let full_help = Command::new(binary)
        .args(["truthful-full-workspace-attempt-v19", "--help"])
        .output()
        .expect("run full help");
    assert!(
        String::from_utf8_lossy(&full_help.stdout)
            .contains("only a finished and passed full run may claim full acceptance")
    );
    let no_run_help = Command::new(binary)
        .args(["workspace-no-run-recovery-gate-v19", "--help"])
        .output()
        .expect("run no-run help");
    assert!(String::from_utf8_lossy(&no_run_help.stdout).contains("no-run-is-not-full"));
    let remote = Command::new(binary)
        .args([
            "sprint118-timeout-reduction-queue",
            "--config",
            "https://example.invalid/config.toml",
        ])
        .output()
        .expect("run remote config rejection");
    assert!(!remote.status.success());
    let stderr = String::from_utf8_lossy(&remote.stderr);
    assert!(
        stderr.contains("config path must be local")
            || stderr.contains("must use local-only paths")
    );
}
