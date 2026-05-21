use std::process::Command;

#[test]
fn sprint96_help_surfaces_required_baseline_signal_warnings() {
    let recover = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["sprint96-baseline-signal-recover", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&recover.stdout).contains("BaselineSignal-only"));

    let no_trade = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["baseline-signal-notrade-default-preservation", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&no_trade.stdout).contains("NoTrade"));

    let feature = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["baseline-signal-feature-regime-preservation", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&feature.stdout).contains("feature/regime"));

    let no_run = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["baseline-signal-no-run-rerun", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&no_run.stdout).contains("compile-only"));

    let full = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["baseline-signal-full-gate-rerun", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&full.stdout).contains("quiet workspace"));

    let counterfactual = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["counterfactual-backfill-entry-gate", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&counterfactual.stdout).contains("entry/precheck only"));

    let panel = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["control-tower-baseline-signal-recovery", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&panel.stdout).contains("Read-only"));
}

#[test]
fn sprint96_root_help_and_remote_path_guards_are_present() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for required in [
        "sprint96-baseline-signal-recover",
        "baseline-signal-real-reduction-plan",
        "baseline-signal-notrade-default-preservation",
        "baseline-signal-feature-regime-preservation",
        "baseline-signal-no-run-rerun",
        "baseline-signal-full-gate-rerun",
        "counterfactual-backfill-entry-gate",
        "control-tower-baseline-signal-recovery",
    ] {
        assert!(stdout.contains(required), "missing command {required}");
    }

    for command in [
        "sprint96-baseline-signal-recover",
        "baseline-signal-real-reduction-plan",
        "baseline-signal-research-only-preservation",
        "baseline-signal-no-run-rerun",
        "baseline-signal-full-gate-rerun",
        "counterfactual-backfill-entry-gate",
        "control-tower-baseline-signal-recovery",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([command, "--config", "https://example.com/config.toml"])
            .output()
            .expect("run");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("config path must be local"));
    }
}
