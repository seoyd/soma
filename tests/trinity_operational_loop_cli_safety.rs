use std::process::Command;

fn help_output(subcommand: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([subcommand, "--help"])
        .output()
        .expect("help output");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn sprint56_cli_help_contains_safety_warnings() {
    let candidate = help_output("candidate-generate");
    assert!(candidate.to_ascii_lowercase().contains("research-only"));

    let cycle = help_output("committee-cycle");
    assert!(cycle.to_ascii_lowercase().contains("paper-only"));

    let loop_help = help_output("trinity-operational-loop");
    assert!(
        loop_help.to_ascii_lowercase().contains("no-live")
            || loop_help.to_ascii_lowercase().contains("no live")
    );

    let paper = help_output("paper-lifecycle-report");
    assert!(
        paper.to_ascii_lowercase().contains("simulated-only")
            || paper.to_ascii_lowercase().contains("simulated only")
    );

    let audit = help_output("operational-audit-timeline");
    assert!(
        audit.to_ascii_lowercase().contains("audit-only")
            || audit.to_ascii_lowercase().contains("audit only")
    );
}
