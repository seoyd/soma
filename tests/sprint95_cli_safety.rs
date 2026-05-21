use std::process::Command;

#[test]
fn sprint95_help_surfaces_required_committee_cli_safety_warnings() {
    let recover = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["sprint95-committee-cli-safety-recover", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&recover.stdout).contains("CommitteeCliSafety-only"));

    let isolation = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["committee-cli-safety-isolation-decision", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&isolation.stdout).contains("isolation decision"));

    let remote = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["committee-cli-safety-remote-path-preservation", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&remote.stdout).contains("remote paths rejected"));

    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["committee-cli-safety-help-text-preservation", "--help"])
        .output()
        .expect("help");
    let help_text = String::from_utf8_lossy(&help.stdout);
    assert!(help_text.contains("research-only"));
    assert!(help_text.contains("paper-only"));
    assert!(help_text.contains("local-only"));
    assert!(help_text.contains("no-runtime"));
    assert!(help_text.contains("no-training"));

    let forbidden = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "committee-cli-safety-forbidden-command-preservation",
            "--help",
        ])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&forbidden.stdout).contains("forbidden commands absent"));

    let runtime = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "committee-cli-safety-runtime-deferred-preservation",
            "--help",
        ])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&runtime.stdout).contains("runtime remains deferred"));

    let baseline = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["baseline-signal-entry-gate", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&baseline.stdout).contains("entry only"));

    let panel = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["control-tower-committee-cli-safety-recovery", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&panel.stdout).contains("Read-only"));
}

// no CLI training command exists
// no CLI live inference command exists
// no CLI Mamba runtime command exists
// no CLI Gated DeltaNet runtime command exists
// no CLI live/order/broker/account command exists
// no runtime LLM path exists
// no Tauri/Svelte command
// no persona expansion command
// no browser execution controls
// no POST/action
// no order/account UI controls
// no balance command
// no holdings command
// no buying power command
// no orderable quantity command
// no correction/cancel command
#[test]
fn sprint95_root_help_keeps_forbidden_command_surface_absent() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for required in [
        "sprint95-committee-cli-safety-recover",
        "committee-cli-safety-reduction-plan",
        "committee-cli-safety-isolation-decision",
        "baseline-signal-entry-gate",
        "control-tower-committee-cli-safety-recovery",
    ] {
        assert!(stdout.contains(required), "missing command {required}");
    }
    for forbidden in [
        "sprint95-train-model",
        "sprint95-live-inference",
        "sprint95-mamba-runtime",
        "sprint95-gated-deltanet",
        "sprint95-broker-order",
        "sprint95-account-balance",
        "sprint95-persona-expansion",
        "tauri-svelte",
    ] {
        assert!(
            !stdout.contains(forbidden),
            "unexpected command {forbidden}"
        );
    }
}

#[test]
fn sprint95_commands_reject_remote_paths() {
    for command in [
        "sprint95-committee-cli-safety-recover",
        "committee-cli-safety-reduction-plan",
        "committee-cli-safety-isolation-decision",
        "committee-cli-safety-remote-path-preservation",
        "committee-cli-safety-help-text-preservation",
        "committee-cli-safety-forbidden-command-preservation",
        "committee-cli-safety-runtime-deferred-preservation",
        "committee-cli-safety-persona-expansion-guard",
        "committee-cli-safety-order-account-guard",
        "committee-cli-safety-browser-execution-guard",
        "baseline-signal-entry-gate",
        "control-tower-committee-cli-safety-recovery",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}

#[test]
fn sprint95_help_output_is_deterministic() {
    let first = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("first");
    let second = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("second");
    assert_eq!(first.stdout, second.stdout);
}
