#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde::Serialize;
use soma_zero::{
    ControlTowerAutoRefreshConfig, EnvironmentIsolationConfig, KISAuthClosureConfig,
    KISCollectionPlanV2Config, KISMarketDataDryRunConfig, KISMarketDataEvidenceSmokeConfig,
    OperationalRunbookV2Config, SecretRedactionAuditConfig,
};

pub fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

pub fn sprint58_data_path(name: &str) -> PathBuf {
    example_path("sprint58_data").join(name)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("sprint58-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint58 test dir");
    path
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub fn with_kis_env(
    app_key: Option<&str>,
    app_secret: Option<&str>,
    base_url: Option<&str>,
    ws_approval_key: Option<&str>,
    run: impl FnOnce(),
) {
    let _guard = env_lock().lock().expect("env lock");
    let old_app_key = std::env::var_os("KIS_APP_KEY");
    let old_app_secret = std::env::var_os("KIS_APP_SECRET");
    let old_base_url = std::env::var_os("KIS_BASE_URL");
    let old_ws = std::env::var_os("KIS_WS_APPROVAL_KEY");
    match app_key {
        Some(value) => unsafe { std::env::set_var("KIS_APP_KEY", value) },
        None => unsafe { std::env::remove_var("KIS_APP_KEY") },
    }
    match app_secret {
        Some(value) => unsafe { std::env::set_var("KIS_APP_SECRET", value) },
        None => unsafe { std::env::remove_var("KIS_APP_SECRET") },
    }
    match base_url {
        Some(value) => unsafe { std::env::set_var("KIS_BASE_URL", value) },
        None => unsafe { std::env::remove_var("KIS_BASE_URL") },
    }
    match ws_approval_key {
        Some(value) => unsafe { std::env::set_var("KIS_WS_APPROVAL_KEY", value) },
        None => unsafe { std::env::remove_var("KIS_WS_APPROVAL_KEY") },
    }
    run();
    match old_app_key {
        Some(value) => unsafe { std::env::set_var("KIS_APP_KEY", value) },
        None => unsafe { std::env::remove_var("KIS_APP_KEY") },
    }
    match old_app_secret {
        Some(value) => unsafe { std::env::set_var("KIS_APP_SECRET", value) },
        None => unsafe { std::env::remove_var("KIS_APP_SECRET") },
    }
    match old_base_url {
        Some(value) => unsafe { std::env::set_var("KIS_BASE_URL", value) },
        None => unsafe { std::env::remove_var("KIS_BASE_URL") },
    }
    match old_ws {
        Some(value) => unsafe { std::env::set_var("KIS_WS_APPROVAL_KEY", value) },
        None => unsafe { std::env::remove_var("KIS_WS_APPROVAL_KEY") },
    }
}

pub fn auth_config(output_root: &Path) -> KISAuthClosureConfig {
    KISAuthClosureConfig {
        closure_id: "test-kis-auth-close".to_string(),
        kis_activation_config_paths: vec![
            example_path("soma_kis_market_data_activate_fixture_replay.toml")
                .display()
                .to_string(),
        ],
        output_root: output_root.display().to_string(),
        ..KISAuthClosureConfig::default()
    }
}

pub fn dry_run_config(output_root: &Path) -> KISMarketDataDryRunConfig {
    KISMarketDataDryRunConfig {
        dry_run_id: "test-kis-dry-run".to_string(),
        endpoint_policy_path: Some(
            example_path("sprint51_data/kis_endpoint_policy.toml")
                .display()
                .to_string(),
        ),
        domestic_symbol_whitelist_path: Some(
            example_path("soma_kis_symbol_whitelist_domestic.toml")
                .display()
                .to_string(),
        ),
        overseas_symbol_whitelist_path: Some(
            example_path("soma_kis_symbol_whitelist_overseas.toml")
                .display()
                .to_string(),
        ),
        output_root: output_root.display().to_string(),
        max_symbols: 4,
        max_requests: 4,
        max_rows_per_symbol: 120,
        max_days: 90,
        max_bytes: 500_000,
        ..KISMarketDataDryRunConfig::default()
    }
}

pub fn collection_plan_config(output_root: &Path) -> KISCollectionPlanV2Config {
    KISCollectionPlanV2Config {
        plan_id: "test-kis-collection-plan-v2".to_string(),
        fixture_response_paths: vec![
            example_path("sprint51_data/kis_domestic_005930_fixture.json")
                .display()
                .to_string(),
            example_path("sprint51_data/kis_overseas_nasd_aapl_fixture.json")
                .display()
                .to_string(),
        ],
        local_canonical_csv_paths: vec![
            example_path("sprint51_data/kis_kr_005930_1d_eod.csv")
                .display()
                .to_string(),
            example_path("sprint51_data/kis_us_nasd_AAPL_1d_delayed.csv")
                .display()
                .to_string(),
        ],
        domestic_symbol_whitelist_path: Some(
            example_path("soma_kis_symbol_whitelist_domestic.toml")
                .display()
                .to_string(),
        ),
        overseas_symbol_whitelist_path: Some(
            example_path("soma_kis_symbol_whitelist_overseas.toml")
                .display()
                .to_string(),
        ),
        endpoint_policy_path: Some(
            example_path("sprint51_data/kis_endpoint_policy.toml")
                .display()
                .to_string(),
        ),
        output_root: output_root.display().to_string(),
        max_jobs: 12,
        max_symbols: 4,
        max_rows_per_symbol: 120,
        max_requests: 4,
        max_days: 90,
        max_bytes: 500_000,
        run_fixture_replay: true,
        run_local_import: true,
        run_operator_live_collection: false,
        ..KISCollectionPlanV2Config::default()
    }
}

pub fn env_isolation_config(output_root: &Path) -> EnvironmentIsolationConfig {
    EnvironmentIsolationConfig {
        report_id: "test-kis-env-isolation".to_string(),
        output_root: output_root.display().to_string(),
        ..EnvironmentIsolationConfig::default()
    }
}

pub fn secret_audit_config(
    output_root: &Path,
    artifact_paths: Vec<String>,
) -> SecretRedactionAuditConfig {
    SecretRedactionAuditConfig {
        audit_id: "test-secret-redaction-audit".to_string(),
        artifact_paths,
        output_root: output_root.display().to_string(),
        ..SecretRedactionAuditConfig::default()
    }
}

pub fn auto_refresh_config(output_root: &Path) -> ControlTowerAutoRefreshConfig {
    ControlTowerAutoRefreshConfig {
        refresh_id: "test-control-tower-auto-refresh".to_string(),
        control_tower_refresh_config_path: Some(
            example_path("soma_control_tower_refresh_after_kis_depth.toml")
                .display()
                .to_string(),
        ),
        source_smoke_report_paths: vec![
            sprint58_data_path("kis_market_data_smoke_sample.json")
                .display()
                .to_string(),
        ],
        output_root: output_root.display().to_string(),
        ..ControlTowerAutoRefreshConfig::default()
    }
}

pub fn runbook_v2_config(output_root: &Path) -> OperationalRunbookV2Config {
    OperationalRunbookV2Config {
        runbook_id: "test-operational-runbook-v2".to_string(),
        smoke_report_paths: vec![
            sprint58_data_path("kis_market_data_smoke_sample.json")
                .display()
                .to_string(),
        ],
        control_tower_auto_refresh_report_paths: vec![
            sprint58_data_path("control_tower_auto_refresh_sample.json")
                .display()
                .to_string(),
        ],
        output_root: output_root.display().to_string(),
        ..OperationalRunbookV2Config::default()
    }
}

pub fn write_toml<T: Serialize>(path: &Path, value: &T) -> PathBuf {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create config parent");
    }
    fs::write(path, toml::to_string_pretty(value).expect("serialize toml")).expect("write toml");
    path.to_path_buf()
}

pub fn smoke_config(output_root: &Path) -> KISMarketDataEvidenceSmokeConfig {
    let config_dir = output_root.join("configs");
    fs::create_dir_all(&config_dir).expect("create config dir");
    let auth = auth_config(output_root);
    let auth_path = write_toml(&config_dir.join("kis_auth_close.toml"), &auth);
    let mut dry = dry_run_config(output_root);
    dry.kis_auth_closure_config_path = Some(auth_path.display().to_string());
    let dry_path = write_toml(&config_dir.join("kis_market_data_dry_run.toml"), &dry);
    let plan = collection_plan_config(output_root);
    let plan_path = write_toml(&config_dir.join("kis_collection_plan_v2.toml"), &plan);
    KISMarketDataEvidenceSmokeConfig {
        smoke_id: "test-kis-market-data-smoke".to_string(),
        auth_closure_config_path: Some(auth_path.display().to_string()),
        dry_run_config_path: Some(dry_path.display().to_string()),
        collection_plan_v2_config_path: Some(plan_path.display().to_string()),
        endpoint_policy_path: Some(
            example_path("sprint51_data/kis_endpoint_policy.toml")
                .display()
                .to_string(),
        ),
        barrier_profile_registry_path: Some(
            example_path("soma_barrier_profiles_primary.toml")
                .display()
                .to_string(),
        ),
        output_root: output_root.display().to_string(),
        max_bytes: 5_000_000,
        ..KISMarketDataEvidenceSmokeConfig::default()
    }
}
