use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use soma_zero::{
    KRXAuthReadinessReport, KRXAuthReadinessStatus, KRXOfficialEvidenceActivationConfig, ReasonCode,
};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_krx_env(api_key: Option<&str>, endpoint: Option<&str>, run: impl FnOnce()) {
    let _guard = env_lock().lock().expect("env lock");
    let old_key = std::env::var_os("KRX_API_KEY");
    let old_endpoint = std::env::var_os("KRX_ENDPOINT_TEMPLATE");
    match api_key {
        Some(value) => unsafe { std::env::set_var("KRX_API_KEY", value) },
        None => unsafe { std::env::remove_var("KRX_API_KEY") },
    }
    match endpoint {
        Some(value) => unsafe { std::env::set_var("KRX_ENDPOINT_TEMPLATE", value) },
        None => unsafe { std::env::remove_var("KRX_ENDPOINT_TEMPLATE") },
    }
    run();
    match old_key {
        Some(value) => unsafe { std::env::set_var("KRX_API_KEY", value) },
        None => unsafe { std::env::remove_var("KRX_API_KEY") },
    }
    match old_endpoint {
        Some(value) => unsafe { std::env::set_var("KRX_ENDPOINT_TEMPLATE", value) },
        None => unsafe { std::env::remove_var("KRX_ENDPOINT_TEMPLATE") },
    }
}

#[test]
fn sprint49_auth_example_parses() {
    let config = KRXOfficialEvidenceActivationConfig::from_toml_path(&example_path(
        "soma_krx_auth_readiness.toml",
    ))
    .expect("parse auth readiness example");
    assert_eq!(config.activation_id, "krx_auth_readiness");
    assert!(config.require_krx_api_key);
    assert!(config.require_krx_endpoint_template);
}

#[test]
fn remote_paths_are_rejected() {
    let config = KRXOfficialEvidenceActivationConfig {
        output_root: "https://example.com/out".to_string(),
        ..KRXOfficialEvidenceActivationConfig::default()
    };
    assert!(
        config
            .validate_local_paths()
            .contains(&ReasonCode::RemotePathRejected)
    );
    assert!(config.validate().is_err());
}

#[test]
fn auth_readiness_detects_presence_without_printing_secret_values() {
    with_krx_env(
        Some("krx_test_redaction_value"),
        Some("https://krx.example/api?token=masked123&symbol={symbol}"),
        || {
            let report = KRXAuthReadinessReport::from_config(
                &KRXOfficialEvidenceActivationConfig::default(),
            );
            let rendered = report.to_text();
            assert!(report.api_key_present);
            assert!(report.endpoint_template_present);
            assert_eq!(report.readiness_status, KRXAuthReadinessStatus::Ready);
            assert!(
                report
                    .endpoint_template_preview_redacted
                    .as_deref()
                    .unwrap_or_default()
                    .contains("configured(redacted")
            );
            assert!(!rendered.contains("krx_test_redaction_value"));
            assert!(!rendered.contains("masked123"));
            assert!(rendered.contains("KRX_API_KEY"));
            assert!(rendered.contains("KRX_ENDPOINT_TEMPLATE"));
        },
    );
}

#[test]
fn auth_readiness_classifies_missing_states() {
    with_krx_env(None, None, || {
        let report =
            KRXAuthReadinessReport::from_config(&KRXOfficialEvidenceActivationConfig::default());
        assert_eq!(
            report.readiness_status,
            KRXAuthReadinessStatus::MissingApiKeyAndEndpointTemplate
        );
        assert!(!report.safe_to_collect_market_data);
    });
    with_krx_env(Some("present"), None, || {
        let report =
            KRXAuthReadinessReport::from_config(&KRXOfficialEvidenceActivationConfig::default());
        assert_eq!(
            report.readiness_status,
            KRXAuthReadinessStatus::MissingEndpointTemplate
        );
    });
    with_krx_env(None, Some("https://krx.example/{symbol}"), || {
        let report =
            KRXAuthReadinessReport::from_config(&KRXOfficialEvidenceActivationConfig::default());
        assert_eq!(
            report.readiness_status,
            KRXAuthReadinessStatus::MissingApiKey
        );
    });
}

#[test]
fn auth_report_is_deterministic() {
    with_krx_env(None, None, || {
        let config = KRXOfficialEvidenceActivationConfig::default();
        let first = KRXAuthReadinessReport::from_config(&config);
        let second = KRXAuthReadinessReport::from_config(&config);
        assert_eq!(first, second);
    });
}
