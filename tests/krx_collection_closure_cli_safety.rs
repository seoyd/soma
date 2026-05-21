use std::process::Command;

#[test]
fn sprint50_cli_help_contains_safety_warnings() {
    let commands = [
        "krx-collection-dry-run",
        "krx-collection-plan",
        "krx-bounded-collect",
        "krx-candle-sufficiency",
        "krx-outcome-link-close",
        "krx-collection-close",
    ];
    for command in commands {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("run help");
        assert!(output.status.success(), "help failed for {command}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.to_ascii_lowercase().contains("research-only"),
            "missing research-only in {command}"
        );
        assert!(
            stdout.to_ascii_lowercase().contains("market-data-only")
                || stdout.to_ascii_lowercase().contains("market-data"),
            "missing market-data warning in {command}"
        );
        assert!(
            stdout.to_ascii_lowercase().contains("secret"),
            "missing secret-safety wording in {command}"
        );
    }
}
