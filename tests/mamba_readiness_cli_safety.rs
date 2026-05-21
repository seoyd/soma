use std::process::Command;

#[test]
fn sprint55_help_texts_and_remote_path_guards_are_safe() {
    let checks = [
        ("core-completion-audit", "research-only"),
        ("sequence-readiness", "research-only"),
        ("mamba-readiness-v2", "runtime remains deferred"),
        ("model-escalation-decision", "no live trading"),
        ("mamba-prototype-plan", "external research-only"),
    ];
    for (command, needle) in checks {
        let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--help"])
            .output()
            .expect("help");
        assert!(help.status.success());
        let text = String::from_utf8_lossy(&help.stdout).to_ascii_lowercase();
        assert!(text.contains(needle));

        let remote = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/x.toml"])
            .output()
            .expect("remote");
        assert!(!remote.status.success());
    }
}

#[test]
fn sprint55_cli_has_no_live_order_broker_account_or_runtime_llm_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
    assert!(text.contains("core-completion-audit"));
    assert!(text.contains("sequence-readiness"));
    assert!(text.contains("mamba-readiness-v2"));
    assert!(text.contains("model-escalation-decision"));
    for forbidden in [
        "\n  live",
        "\n  order",
        "\n  broker",
        "\n  account",
        "runtime-llm",
        "rust-native-mamba",
    ] {
        assert!(!text.contains(forbidden));
    }
}
