use std::process::Command;

#[test]
fn unexpected_diff_triage_cli_emits_research_only_warning() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--bin",
            "soma_experiment",
            "--",
            "unexpected-diff-triage",
            "--config",
            "examples/soma_unexpected_diff_triage.toml",
        ])
        .output()
        .expect("run unexpected-diff-triage");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("research_only_warning="));
    assert!(stdout.contains("\"triage_status\": \"UnexpectedDiffExplained\""));
    assert!(!stdout.contains("live trading readiness"));
}
