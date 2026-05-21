mod common;

use std::fs;

use soma_zero::{
    AssetClass, AuthConfig, CollectionOutputSize, MarketVenue, OfficialCollectionEntry,
    OfficialCollectionPlan, OfficialEvidenceAcquisitionPlan,
    OfficialEvidenceAcquisitionRecommendation, OfficialEvidenceAcquisitionRunner,
    OfficialEvidenceExpansionConfig, OfficialEvidenceExpansionStatus, ProviderAuthEnvRequirement,
    ProviderAuthPreflightConfig, ProviderKind, RawArchivePolicy, ReasonCode, RetentionPolicy,
    Timeframe,
};

fn fixture_collection_plan(name: &str) -> std::path::PathBuf {
    let output_root = common::output_dir(&format!("{name}-collection"));
    let plan_path = output_root.join("plan.toml");
    let plan = OfficialCollectionPlan {
        plan_id: format!("{name}-source"),
        output_root: output_root.display().to_string(),
        max_total_bytes: 16 * 1024 * 1024,
        max_total_rows: 1500,
        max_total_requests: 10,
        default_collection_size_policy: soma_zero::CollectionSizePolicy {
            max_symbols_per_run: 3,
            max_rows_per_symbol: 500,
            max_total_rows_per_run: 1500,
            max_raw_bytes_per_run: 5 * 1024 * 1024,
            max_canonical_bytes_per_run: 2 * 1024 * 1024,
            max_requests_per_run: 10,
            max_days_per_run: 365,
            default_outputsize: CollectionOutputSize::Compact,
            raw_archive_policy: RawArchivePolicy::CompactJson,
            retention_policy: RetentionPolicy::DeleteRawAfterCanonicalAndManifest,
            allow_full_history: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        default_compression_policy: soma_zero::CompressionPolicy::default(),
        default_retention_policy: RetentionPolicy::DeleteRawAfterCanonicalAndManifest,
        storage_budget: soma_zero::StorageBudget::default(),
        entries: vec![
            OfficialCollectionEntry {
                entry_id: "upbit-btc".to_string(),
                provider_kind: ProviderKind::Upbit,
                symbol: "KRW-BTC".to_string(),
                normalized_symbol: None,
                venue: Some(MarketVenue::Upbit),
                asset_class: AssetClass::Crypto,
                timeframe: Timeframe::OneMinute,
                start: None,
                end: None,
                max_rows: Some(50),
                max_requests: Some(1),
                outputsize: Some(CollectionOutputSize::Compact),
                auth_config_ref: None,
                endpoint_template: None,
                fixture_path: Some(
                    common::fixture_path("generic_ohlcv_valid.csv")
                        .display()
                        .to_string(),
                ),
                enabled: true,
                tags: vec!["crypto".to_string()],
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            OfficialCollectionEntry {
                entry_id: "krx-005930".to_string(),
                provider_kind: ProviderKind::KrxOpenApi,
                symbol: "005930".to_string(),
                normalized_symbol: None,
                venue: Some(MarketVenue::KOSPI),
                asset_class: AssetClass::Equity,
                timeframe: Timeframe::OneDay,
                start: None,
                end: None,
                max_rows: Some(50),
                max_requests: Some(1),
                outputsize: Some(CollectionOutputSize::Compact),
                auth_config_ref: Some(AuthConfig {
                    provider_kind: ProviderKind::KrxOpenApi,
                    api_key_env_var: Some("IGNORED".to_string()),
                    api_secret_env_var: None,
                    auth_header_name: Some("Authorization".to_string()),
                    query_param_name: None,
                    allow_missing_for_mock: false,
                    reason_codes: vec![ReasonCode::DeterministicPath],
                }),
                endpoint_template: Some("https://krx.example/{symbol}".to_string()),
                fixture_path: Some(
                    common::fixture_path("generic_ohlcv_valid.csv")
                        .display()
                        .to_string(),
                ),
                enabled: true,
                tags: vec!["krx".to_string()],
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            OfficialCollectionEntry {
                entry_id: "us-aapl".to_string(),
                provider_kind: ProviderKind::AlphaVantage,
                symbol: "AAPL".to_string(),
                normalized_symbol: None,
                venue: Some(MarketVenue::NASDAQ),
                asset_class: AssetClass::Equity,
                timeframe: Timeframe::OneDay,
                start: None,
                end: None,
                max_rows: Some(50),
                max_requests: Some(1),
                outputsize: Some(CollectionOutputSize::Compact),
                auth_config_ref: Some(AuthConfig {
                    provider_kind: ProviderKind::AlphaVantage,
                    api_key_env_var: Some("IGNORED".to_string()),
                    api_secret_env_var: None,
                    auth_header_name: None,
                    query_param_name: Some("apikey".to_string()),
                    allow_missing_for_mock: false,
                    reason_codes: vec![ReasonCode::DeterministicPath],
                }),
                endpoint_template: None,
                fixture_path: Some(
                    common::fixture_path("generic_ohlcv_valid.csv")
                        .display()
                        .to_string(),
                ),
                enabled: true,
                tags: vec!["us".to_string()],
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
        ],
        continue_on_missing_auth: true,
        continue_on_provider_failure: true,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    fs::write(&plan_path, toml::to_string_pretty(&plan).expect("toml")).expect("write plan");
    plan_path
}

fn krx_key(prefix: &str) -> String {
    format!("SOMA_TEST_ACQUIRE_{prefix}_KRX_KEY")
}

fn krx_endpoint(prefix: &str) -> String {
    format!("SOMA_TEST_ACQUIRE_{prefix}_KRX_ENDPOINT")
}

fn alpha_key(prefix: &str) -> String {
    format!("SOMA_TEST_ACQUIRE_{prefix}_ALPHA_KEY")
}

fn auth_config(prefix: &str) -> ProviderAuthPreflightConfig {
    ProviderAuthPreflightConfig {
        check_id: "official-acquire-test".to_string(),
        providers_to_check: vec![
            ProviderKind::Upbit,
            ProviderKind::KrxOpenApi,
            ProviderKind::AlphaVantage,
        ],
        required_env_vars: vec![
            ProviderAuthEnvRequirement {
                provider_kind: ProviderKind::KrxOpenApi,
                api_key_env_var: Some(krx_key(prefix)),
                api_secret_env_var: None,
                endpoint_template_env_var: Some(krx_endpoint(prefix)),
            },
            ProviderAuthEnvRequirement {
                provider_kind: ProviderKind::AlphaVantage,
                api_key_env_var: Some(alpha_key(prefix)),
                api_secret_env_var: None,
                endpoint_template_env_var: None,
            },
        ],
        ..ProviderAuthPreflightConfig::default()
    }
}

fn clear_env(prefix: &str) {
    unsafe {
        std::env::remove_var(krx_key(prefix));
        std::env::remove_var(krx_endpoint(prefix));
        std::env::remove_var(alpha_key(prefix));
    }
}

#[test]
fn auth_preflight_runs_first_and_crypto_only_is_supported() {
    let prefix = "CRYPTO";
    clear_env(prefix);
    let report = OfficialEvidenceAcquisitionRunner::default()
        .run(&OfficialEvidenceAcquisitionPlan {
            plan_id: "official-acquire-crypto".to_string(),
            auth_preflight_config: auth_config(prefix),
            official_collection_plan_path: Some(
                fixture_collection_plan("official-acquire-crypto")
                    .display()
                    .to_string(),
            ),
            expansion_config: None,
            output_root: common::output_dir("official-acquire-crypto-out")
                .display()
                .to_string(),
            ..OfficialEvidenceAcquisitionPlan::default()
        })
        .expect("run");

    assert_eq!(
        report.auth_preflight_report.check_id,
        "official-acquire-test"
    );
    assert_eq!(
        report.final_status,
        OfficialEvidenceExpansionStatus::CryptoOnly
    );
    assert_eq!(
        report.final_recommendation,
        OfficialEvidenceAcquisitionRecommendation::RunCryptoOnlyEvidence
    );
}

#[test]
fn missing_auth_entries_are_skipped_and_only_ready_providers_collect() {
    let prefix = "SKIP";
    clear_env(prefix);
    let report = OfficialEvidenceAcquisitionRunner::default()
        .run(&OfficialEvidenceAcquisitionPlan {
            plan_id: "official-acquire-skip-missing".to_string(),
            auth_preflight_config: auth_config(prefix),
            official_collection_plan_path: Some(
                fixture_collection_plan("official-acquire-skip-missing")
                    .display()
                    .to_string(),
            ),
            expansion_config: None,
            output_root: common::output_dir("official-acquire-skip-missing-out")
                .display()
                .to_string(),
            ..OfficialEvidenceAcquisitionPlan::default()
        })
        .expect("run");

    let collection_report = report.collection_report.expect("collection report");
    assert!(
        collection_report
            .entry_reports
            .iter()
            .all(|entry| entry.provider_kind == ProviderKind::Upbit)
    );
}

#[test]
fn no_ready_provider_returns_missing_auth_when_upbit_is_disabled() {
    let prefix = "NOREADY";
    clear_env(prefix);
    let report = OfficialEvidenceAcquisitionRunner::default()
        .run(&OfficialEvidenceAcquisitionPlan {
            plan_id: "official-acquire-no-ready".to_string(),
            auth_preflight_config: auth_config(prefix),
            official_collection_plan_path: Some(
                fixture_collection_plan("official-acquire-no-ready")
                    .display()
                    .to_string(),
            ),
            run_upbit_if_public_available: false,
            expansion_config: None,
            output_root: common::output_dir("official-acquire-no-ready-out")
                .display()
                .to_string(),
            ..OfficialEvidenceAcquisitionPlan::default()
        })
        .expect("run");

    assert_eq!(
        report.final_status,
        OfficialEvidenceExpansionStatus::MissingAuth
    );
}

#[test]
fn equity_auth_ready_enables_multi_provider_generated_plan() {
    let prefix = "MULTI";
    clear_env(prefix);
    unsafe {
        std::env::set_var(krx_key(prefix), "present");
        std::env::set_var(krx_endpoint(prefix), "template");
        std::env::set_var(alpha_key(prefix), "present");
    }
    let report = OfficialEvidenceAcquisitionRunner::default()
        .run(&OfficialEvidenceAcquisitionPlan {
            plan_id: "official-acquire-multi".to_string(),
            auth_preflight_config: auth_config(prefix),
            official_collection_plan_path: Some(
                fixture_collection_plan("official-acquire-multi")
                    .display()
                    .to_string(),
            ),
            expansion_config: None,
            output_root: common::output_dir("official-acquire-multi-out")
                .display()
                .to_string(),
            ..OfficialEvidenceAcquisitionPlan::default()
        })
        .expect("run");

    let generated = report
        .generated_collection_plan
        .expect("generated collection plan");
    assert!(
        generated
            .entries
            .iter()
            .any(|entry| entry.provider_kind == ProviderKind::KrxOpenApi)
    );
    assert!(
        generated
            .entries
            .iter()
            .any(|entry| entry.provider_kind == ProviderKind::AlphaVantage)
    );
    unsafe {
        std::env::remove_var(krx_key(prefix));
        std::env::remove_var(krx_endpoint(prefix));
        std::env::remove_var(alpha_key(prefix));
    }
}

#[test]
fn expansion_report_is_generated_only_when_configured() {
    let prefix = "EXPAND";
    clear_env(prefix);
    let with_expansion = OfficialEvidenceAcquisitionRunner::default()
        .run(&OfficialEvidenceAcquisitionPlan {
            plan_id: "official-acquire-with-expansion".to_string(),
            auth_preflight_config: auth_config(prefix),
            official_collection_plan_path: Some(
                fixture_collection_plan("official-acquire-with-expansion")
                    .display()
                    .to_string(),
            ),
            expansion_config: Some(OfficialEvidenceExpansionConfig {
                run_core_benchmark: false,
                ..OfficialEvidenceExpansionConfig::default()
            }),
            output_root: common::output_dir("official-acquire-with-expansion-out")
                .display()
                .to_string(),
            ..OfficialEvidenceAcquisitionPlan::default()
        })
        .expect("run with expansion");
    let without_expansion = OfficialEvidenceAcquisitionRunner::default()
        .run(&OfficialEvidenceAcquisitionPlan {
            plan_id: "official-acquire-no-expansion".to_string(),
            auth_preflight_config: auth_config(prefix),
            official_collection_plan_path: Some(
                fixture_collection_plan("official-acquire-no-expansion")
                    .display()
                    .to_string(),
            ),
            expansion_config: None,
            output_root: common::output_dir("official-acquire-no-expansion-out")
                .display()
                .to_string(),
            ..OfficialEvidenceAcquisitionPlan::default()
        })
        .expect("run without expansion");

    assert!(with_expansion.expansion_report.is_some());
    assert!(without_expansion.expansion_report.is_none());
}

#[test]
fn official_acquisition_report_is_deterministic() {
    let prefix = "DETERMINISTIC";
    clear_env(prefix);
    let plan = OfficialEvidenceAcquisitionPlan {
        plan_id: "official-acquire-deterministic".to_string(),
        auth_preflight_config: auth_config(prefix),
        official_collection_plan_path: Some(
            fixture_collection_plan("official-acquire-deterministic")
                .display()
                .to_string(),
        ),
        expansion_config: None,
        output_root: common::output_dir("official-acquire-deterministic-out")
            .display()
            .to_string(),
        ..OfficialEvidenceAcquisitionPlan::default()
    };

    let first = OfficialEvidenceAcquisitionRunner::default()
        .run(&plan)
        .expect("first")
        .to_json_string()
        .expect("json");
    let second = OfficialEvidenceAcquisitionRunner::default()
        .run(&plan)
        .expect("second")
        .to_json_string()
        .expect("json");

    assert_eq!(first, second);
}
