mod common;

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use soma_zero::{
    KRXBoundedCollectionSmokeConfig, KRXCollectionDryRunStatus, KRXSymbolWhitelistConfig,
    ReasonCode,
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
fn smoke_config_parses_and_rejects_remote_paths() {
    let config = KRXBoundedCollectionSmokeConfig::from_toml_path(&example_path(
        "soma_krx_collection_dry_run.toml",
    ))
    .expect("parse sprint50 dry-run example");
    assert!(!config.run_live_collection);
    assert_eq!(config.max_symbols, 5);

    let remote = KRXBoundedCollectionSmokeConfig {
        output_root: "https://example.com/out".to_string(),
        ..KRXBoundedCollectionSmokeConfig::default()
    };
    assert!(remote.validate().is_err());
    assert!(
        remote
            .validate_local_paths()
            .contains(&ReasonCode::LocalPathRejected)
    );
}

#[test]
fn dry_run_detects_missing_and_present_env_without_leaking_secrets() {
    let config = KRXBoundedCollectionSmokeConfig::from_toml_path(&example_path(
        "soma_krx_collection_dry_run.toml",
    ))
    .expect("parse config");
    let whitelist = KRXSymbolWhitelistConfig::from_toml_path(&example_path(
        "sprint50_data/krx_whitelist_compact.toml",
    ))
    .expect("parse whitelist")
    .build();

    with_krx_env(None, None, || {
        let report = config.build_dry_run_report(&whitelist);
        assert_eq!(
            report.dry_run_status,
            KRXCollectionDryRunStatus::MissingApiKeyAndEndpointTemplate
        );
        assert!(!report.safe_to_run_live_collection);
    });

    with_krx_env(
        Some("krx_test_secret_value"),
        Some("https://krx.example/api?token=masked123&symbol={symbol}"),
        || {
            let report = config.build_dry_run_report(&whitelist);
            let rendered = report.to_text();
            assert_eq!(
                report.dry_run_status,
                KRXCollectionDryRunStatus::ReadyToCollect
            );
            assert!(!rendered.contains("krx_test_secret_value"));
            assert!(!rendered.contains("masked123"));
            assert!(rendered.contains("configured(redacted"));
        },
    );
}

#[test]
fn dry_run_is_deterministic() {
    let config = KRXBoundedCollectionSmokeConfig::from_toml_path(&example_path(
        "soma_krx_collection_dry_run.toml",
    ))
    .expect("parse config");
    let whitelist = config.load_whitelist().expect("load whitelist");
    with_krx_env(None, None, || {
        let first = config.build_dry_run_report(&whitelist);
        let second = config.build_dry_run_report(&whitelist);
        assert_eq!(first, second);
    });
}
