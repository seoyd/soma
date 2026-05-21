use std::process::Command;

#[test]
fn help_mentions_sprint58_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("kis-auth-close"));
    assert!(stdout.contains("kis-market-data-dry-run"));
    assert!(stdout.contains("kis-collection-plan-v2"));
    assert!(stdout.contains("kis-market-data-smoke"));
    assert!(stdout.contains("control-tower-auto-refresh"));
    assert!(stdout.contains("operational-runbook-v2"));
}

#[test]
fn kis_auth_close_never_prints_secret_values() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "kis-auth-close",
            "--config",
            "examples/soma_kis_auth_close.toml",
        ])
        .env("KIS_APP_KEY", "top-secret-app-key")
        .env("KIS_APP_SECRET", "top-secret-app-secret")
        .env("KIS_BASE_URL", "https://private.example.internal")
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("top-secret-app-key"));
    assert!(!stdout.contains("top-secret-app-secret"));
    assert!(!stdout.contains("https://private.example.internal"));
    assert!(stdout.contains("app_key_env_var_name=KIS_APP_KEY"));
}

#[test]
fn kis_market_data_smoke_rejects_remote_config_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "kis-market-data-smoke",
            "--config",
            "https://example.com/smoke.toml",
        ])
        .output()
        .expect("run");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("kis-market-data-smoke config path must be local")
    );
}
