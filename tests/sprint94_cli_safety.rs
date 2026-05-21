use std::process::Command;

#[test]
fn sprint94_help_contains_required_safety_language() {
    let recover = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["sprint94-dashboard-renderer-recover", "--help"])
        .output()
        .expect("help");
    let recover_text = String::from_utf8_lossy(&recover.stdout);
    assert!(recover_text.contains("DashboardRenderer-only"));

    let plan = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["dashboard-renderer-real-reduction-plan", "--help"])
        .output()
        .expect("help");
    let plan_text = String::from_utf8_lossy(&plan.stdout);
    assert!(plan_text.contains("preserve assertions"));

    let assertion = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["dashboard-renderer-assertion-migration", "--help"])
        .output()
        .expect("help");
    let assertion_text = String::from_utf8_lossy(&assertion.stdout);
    assert!(assertion_text.contains("no assertion deletion"));

    let static_safety = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["dashboard-renderer-static-safety-preservation", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&static_safety.stdout).contains("static/read-only"));

    let secret = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["dashboard-renderer-secret-redaction-preservation", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&secret.stdout).contains("no secrets"));

    let browser = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["dashboard-renderer-no-browser-execution", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&browser.stdout).contains("no browser execution"));

    let action = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["dashboard-renderer-no-action-control", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&action.stdout).contains("no order/account/trade controls"));

    let no_run = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["dashboard-renderer-no-run-rerun", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&no_run.stdout).contains("no-run only"));

    let full = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["dashboard-renderer-full-gate-rerun", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&full.stdout).contains("full workspace only"));

    let panel = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["control-tower-dashboard-renderer-recovery", "--help"])
        .output()
        .expect("help");
    assert!(String::from_utf8_lossy(&panel.stdout).contains("Read-only"));
}

#[test]
fn sprint94_cli_rejects_remote_paths_and_no_new_runtime_commands_exist() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "sprint94-dashboard-renderer-recover",
            "--config",
            "https://example.com/sprint94.toml",
        ])
        .output()
        .expect("run");
    assert!(!output.status.success());

    let help = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("sprint94-dashboard-renderer-recover"));
    assert!(text.contains("control-tower-dashboard-renderer-recovery"));
    assert!(!text.contains("sprint94-live-inference"));
    assert!(!text.contains("sprint94-train-model"));
    assert!(!text.contains("sprint94-mamba-runtime"));
    assert!(!text.contains("sprint94-gated-deltanet"));
    assert!(!text.contains("sprint94-broker-order"));
    assert!(!text.contains("sprint94-account-balance"));
}
