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
fn sprint57_cli_help_contains_safety_warnings() {
    let depth = help_output("kis-evidence-depth-run");
    assert!(depth.to_ascii_lowercase().contains("research-only"));

    let refresh = help_output("control-tower-refresh");
    assert!(refresh.to_ascii_lowercase().contains("read-only"));

    let runbook = help_output("operational-runbook");
    assert!(runbook.to_ascii_lowercase().contains("paper-only"));
}
