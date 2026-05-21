mod common;

use std::fs;
use std::process::Command;

use soma_zero::{
    OfficialProviderReadinessConfig, ProviderAuthCheckMode, ProviderCredentialProfile,
    ProviderKind, ProviderSecretValuePolicy,
};

fn unique(name: &str) -> String {
    format!("SOMA_TEST_{name}")
}

fn override_profile(
    provider_kind: ProviderKind,
    required_env_vars: &[String],
) -> ProviderCredentialProfile {
    ProviderCredentialProfile {
        provider_kind,
        required_env_vars: required_env_vars.to_vec(),
        optional_env_vars: Vec::new(),
        endpoint_template_env_vars: Vec::new(),
        secret_value_policy: vec![
            ProviderSecretValuePolicy::EnvVarNameOnly,
            ProviderSecretValuePolicy::NeverPersistSecret,
            ProviderSecretValuePolicy::NeverPrintSecret,
        ],
        auth_check_mode: ProviderAuthCheckMode::PresenceOnly,
        reason_codes: Vec::new(),
    }
}

#[test]
fn provider_help_mentions_research_only_and_new_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("provider-catalog"));
    assert!(stdout.contains("provider-readiness"));
    assert!(stdout.contains("provider-select"));
    assert!(stdout.contains("Research-only"));
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  account"));
}

#[test]
fn provider_readiness_rejects_remote_config_path() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "provider-readiness",
            "--config",
            "https://example.com/provider-readiness.toml",
        ])
        .output()
        .expect("run");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("provider-readiness config path must be local")
    );
}

#[test]
fn provider_select_runs_for_crypto_without_auth() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args(["provider-select", "--market", "crypto"])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("market=Crypto"));
    assert!(stdout.contains("status=Selected"));
}

#[test]
fn provider_readiness_never_prints_secret_values() {
    let key = unique("READINESS_ALPHA_KEY");
    unsafe { std::env::set_var(&key, "top-secret-value") };
    let config_path = common::output_dir("provider-readiness-cli").join("config.toml");
    fs::write(
        &config_path,
        OfficialProviderReadinessConfig {
            credential_profile_overrides: vec![override_profile(
                ProviderKind::AlphaVantage,
                std::slice::from_ref(&key),
            )],
            ..OfficialProviderReadinessConfig::default()
        }
        .to_toml_string()
        .expect("serialize"),
    )
    .expect("write");
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "provider-readiness",
            "--config",
            &config_path.display().to_string(),
        ])
        .output()
        .expect("run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("top-secret-value"));
    unsafe { std::env::remove_var(&key) };
}
