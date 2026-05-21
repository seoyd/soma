use std::process::Command;

#[test]
fn cli_help_exposes_campaign_and_compare_as_research_only_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("cli help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Research-only"));
    assert!(stdout.contains("campaign"));
    assert!(stdout.contains("compare"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  live"));
    assert!(!stdout.contains("\n  api"));
}
