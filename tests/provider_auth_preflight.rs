use std::path::PathBuf;

use soma_zero::{
    ProviderAuthEnvRequirement, ProviderAuthPreflightConfig, ProviderAuthPreflightRunner,
    ProviderAuthStatusKind, ProviderKind,
};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

fn unique(name: &str) -> String {
    format!("SOMA_TEST_{}", name)
}

fn clear_env(names: &[String]) {
    for name in names {
        unsafe { std::env::remove_var(name) };
    }
}

#[test]
fn upbit_reports_not_required() {
    let report = ProviderAuthPreflightRunner::default().run(&ProviderAuthPreflightConfig {
        providers_to_check: vec![ProviderKind::Upbit],
        ..ProviderAuthPreflightConfig::default()
    });

    assert_eq!(report.statuses.len(), 1);
    assert_eq!(
        report.statuses[0].status,
        ProviderAuthStatusKind::NotRequired
    );
    assert!(report.safe_to_collect);
}

#[test]
fn krx_missing_env_var_reports_missing_auth() {
    let key = unique("KRX_KEY_ONLY_MISSING");
    let endpoint = unique("KRX_ENDPOINT_PRESENT");
    clear_env(&[key.clone(), endpoint.clone()]);
    unsafe { std::env::set_var(&endpoint, "https://krx.example/{symbol}") };

    let report = ProviderAuthPreflightRunner::default().run(&ProviderAuthPreflightConfig {
        providers_to_check: vec![ProviderKind::KrxOpenApi],
        required_env_vars: vec![ProviderAuthEnvRequirement {
            provider_kind: ProviderKind::KrxOpenApi,
            api_key_env_var: Some(key.clone()),
            api_secret_env_var: None,
            endpoint_template_env_var: Some(endpoint.clone()),
        }],
        ..ProviderAuthPreflightConfig::default()
    });

    assert_eq!(
        report.statuses[0].status,
        ProviderAuthStatusKind::MissingAuth
    );
    assert!(report.to_text().contains(&key));
    clear_env(&[key, endpoint]);
}

#[test]
fn krx_missing_endpoint_template_reports_missing_endpoint() {
    let key = unique("KRX_KEY_PRESENT");
    let endpoint = unique("KRX_ENDPOINT_MISSING");
    clear_env(&[key.clone(), endpoint.clone()]);
    unsafe { std::env::set_var(&key, "present") };

    let report = ProviderAuthPreflightRunner::default().run(&ProviderAuthPreflightConfig {
        providers_to_check: vec![ProviderKind::KrxOpenApi],
        required_env_vars: vec![ProviderAuthEnvRequirement {
            provider_kind: ProviderKind::KrxOpenApi,
            api_key_env_var: Some(key.clone()),
            api_secret_env_var: None,
            endpoint_template_env_var: Some(endpoint.clone()),
        }],
        ..ProviderAuthPreflightConfig::default()
    });

    assert_eq!(
        report.statuses[0].status,
        ProviderAuthStatusKind::MissingEndpointTemplate
    );
    clear_env(&[key, endpoint]);
}

#[test]
fn alphavantage_missing_api_key_reports_missing_auth() {
    let key = unique("ALPHAVANTAGE_KEY_MISSING");
    clear_env(std::slice::from_ref(&key));

    let report = ProviderAuthPreflightRunner::default().run(&ProviderAuthPreflightConfig {
        providers_to_check: vec![ProviderKind::AlphaVantage],
        required_env_vars: vec![ProviderAuthEnvRequirement {
            provider_kind: ProviderKind::AlphaVantage,
            api_key_env_var: Some(key.clone()),
            api_secret_env_var: None,
            endpoint_template_env_var: None,
        }],
        ..ProviderAuthPreflightConfig::default()
    });

    assert_eq!(
        report.statuses[0].status,
        ProviderAuthStatusKind::MissingAuth
    );
}

#[test]
fn auth_report_never_includes_secret_values() {
    let key = unique("ALPACA_KEY_NAME");
    let secret = unique("ALPACA_SECRET_NAME");
    clear_env(&[key.clone(), secret.clone()]);
    unsafe { std::env::set_var(&key, "visible-key-value") };
    unsafe { std::env::set_var(&secret, "super-secret-value") };

    let report = ProviderAuthPreflightRunner::default().run(&ProviderAuthPreflightConfig {
        providers_to_check: vec![ProviderKind::Alpaca],
        required_env_vars: vec![ProviderAuthEnvRequirement {
            provider_kind: ProviderKind::Alpaca,
            api_key_env_var: Some(key.clone()),
            api_secret_env_var: Some(secret.clone()),
            endpoint_template_env_var: None,
        }],
        ..ProviderAuthPreflightConfig::default()
    });
    let text = report.to_text();

    assert!(text.contains(&key));
    assert!(text.contains(&secret));
    assert!(!text.contains("visible-key-value"));
    assert!(!text.contains("super-secret-value"));
    clear_env(&[key, secret]);
}

#[test]
fn missing_optional_auth_does_not_fail_report_when_allowed() {
    let report = ProviderAuthPreflightRunner::default().run(&ProviderAuthPreflightConfig {
        providers_to_check: vec![ProviderKind::Alpaca],
        allow_missing_optional_auth: true,
        fail_on_missing_required_auth: true,
        ..ProviderAuthPreflightConfig::default()
    });

    assert_eq!(report.statuses[0].status, ProviderAuthStatusKind::Deferred);
    assert!(report.safe_to_collect);
}

#[test]
fn auth_report_is_deterministic() {
    let config = ProviderAuthPreflightConfig::default();
    let runner = ProviderAuthPreflightRunner::default();

    let first = runner.run(&config).to_json_string().expect("first json");
    let second = runner.run(&config).to_json_string().expect("second json");

    assert_eq!(first, second);
}

#[test]
fn sprint25_provider_auth_example_parses() {
    let config = ProviderAuthPreflightConfig::from_toml_path(&example_path(
        "soma_provider_auth_preflight.toml",
    ))
    .expect("parse auth example");

    assert!(
        config
            .providers_to_check
            .contains(&ProviderKind::KrxOpenApi)
    );
    assert!(
        config
            .required_env_vars
            .iter()
            .any(|requirement| requirement.provider_kind == ProviderKind::AlphaVantage)
    );
}
