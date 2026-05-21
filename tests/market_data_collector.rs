mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::json;
use soma_zero::{
    AdjustedPricePolicy, AssetClass, AuthConfig, CandleFetchRequest, CollectionSizePolicy,
    CollectorRunner, FixtureHttpClient, MarketVenue, PreflightFinalStatus, ProviderKind,
    ReasonCode, RequestedOutputSize, Timeframe,
};

fn provider_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("provider")
        .join(name)
}

#[test]
fn upbit_profile_is_market_data_only() {
    let provider = ProviderKind::Upbit.profile();
    assert!(!provider.supports_trading);
    assert!(!provider.supports_account);
    assert!(provider.public_candles_only);
    assert_eq!(provider.max_candles_per_request, 200);
}

#[test]
fn equity_provider_profiles_are_market_data_only() {
    for kind in [ProviderKind::KrxOpenApi, ProviderKind::AlphaVantage] {
        let provider = kind.profile();
        assert!(!provider.supports_trading);
        assert!(!provider.supports_account);
        assert!(
            provider
                .capabilities
                .contains(&soma_zero::ProviderCapability::TradingNotSupported)
        );
    }
}

#[test]
fn collector_splits_requests_and_writes_outputs() {
    let start = 1_711_929_600_000u64;
    let step = 60_000u64;
    let counts = [200usize, 200usize, 5usize];
    let mut end_cursor = start + (405 - 1) as u64 * step;
    let responses = counts
        .into_iter()
        .map(|count| {
            let window_start = end_cursor - (count as u64 - 1) * step;
            let body = build_upbit_response("KRW-BTC", window_start, count, step);
            let response = json!({
                "match_substring": format!("to={}", to_param(end_cursor)),
                "body": body,
            });
            end_cursor -= count as u64 * step;
            response
        })
        .collect::<Vec<_>>();
    let fixture_path = write_fixture("collector-split", responses);
    let client = FixtureHttpClient::from_path(&fixture_path).expect("fixture client");
    let output = common::output_dir("collector-split");
    let result = CollectorRunner::default()
        .run_with_client(
            &CandleFetchRequest {
                request_id: "collector-split".to_string(),
                provider_kind: ProviderKind::Upbit,
                symbol: "KRW-BTC".to_string(),
                market_venue: Some(MarketVenue::Upbit),
                asset_class: AssetClass::Crypto,
                timeframe: Timeframe::OneMinute,
                start_timestamp_ms: Some(start),
                end_timestamp_ms: Some(start + (405 - 1) as u64 * step),
                output_root: output.display().to_string(),
                limit_per_request: None,
                include_raw_archive: true,
                fill_missing_policy: soma_zero::FillMissingPolicy::LeaveGaps,
                fixture_path: Some(fixture_path.display().to_string()),
                adjusted_price_policy: AdjustedPricePolicy::Raw,
                collection_size_policy: CollectionSizePolicy::default(),
                auth_config: None,
                endpoint_template: None,
                requested_output_size: None,
                allow_full_history_override: false,
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            &client,
        )
        .expect("collect candles");

    assert_eq!(result.request_count, 3);
    assert_eq!(result.row_count, 405);
    assert_eq!(result.provider_id, "upbit");
    assert_eq!(
        result.preflight_status,
        PreflightFinalStatus::ReadyForRealEvidence
    );
    assert!(Path::new(&result.canonical_csv_path).exists());
    assert!(Path::new(&result.manifest_path).exists());
    assert!(Path::new(&result.provenance_path).exists());
    assert_eq!(result.raw_request_paths.len(), 3);

    let provenance = fs::read_to_string(&result.provenance_path).expect("read provenance");
    assert!(provenance.contains("source_kind=OfficialApiCollected"));
    assert!(
        fs::read_to_string(Path::new(&result.output_dir).join("preflight_report.json"))
            .expect("read preflight")
            .contains("\"final_status\": \"ReadyForRealEvidence\"")
    );
    assert!(
        Path::new(&result.output_dir)
            .join("generated_real_evidence_closure.toml")
            .exists()
    );
}

#[test]
fn collector_retries_transient_fixture_failure() {
    let start = 1_711_929_600_000u64;
    let step = 60_000u64;
    let fixture_path = write_fixture(
        "collector-retry",
        vec![json!({
            "match_substring": "market=KRW-BTC",
            "fail_times": 1,
            "body": build_upbit_response("KRW-BTC", start, 60, step),
        })],
    );
    let client = FixtureHttpClient::from_path(&fixture_path).expect("fixture client");
    let output = common::output_dir("collector-retry");
    let result = CollectorRunner::default()
        .run_with_client(
            &CandleFetchRequest {
                request_id: "collector-retry".to_string(),
                provider_kind: ProviderKind::Upbit,
                symbol: "KRW-BTC".to_string(),
                market_venue: Some(MarketVenue::Upbit),
                asset_class: AssetClass::Crypto,
                timeframe: Timeframe::OneMinute,
                start_timestamp_ms: Some(start),
                end_timestamp_ms: Some(start + (60 - 1) as u64 * step),
                output_root: output.display().to_string(),
                limit_per_request: None,
                include_raw_archive: false,
                fill_missing_policy: soma_zero::FillMissingPolicy::LeaveGaps,
                fixture_path: Some(fixture_path.display().to_string()),
                adjusted_price_policy: AdjustedPricePolicy::Raw,
                collection_size_policy: CollectionSizePolicy::default(),
                auth_config: None,
                endpoint_template: None,
                requested_output_size: None,
                allow_full_history_override: false,
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            &client,
        )
        .expect("collect candles");

    assert!(
        result
            .reason_codes
            .contains(&ReasonCode::ProviderRequestRetried)
    );
}

#[test]
fn alphavantage_requires_api_key_without_fixture() {
    let output_root = common::output_dir("collector-alpha-auth");
    let error = CollectorRunner::default()
        .run(&CandleFetchRequest {
            request_id: "collector-alpha-auth".to_string(),
            provider_kind: ProviderKind::AlphaVantage,
            symbol: "AAPL".to_string(),
            market_venue: Some(MarketVenue::NASDAQ),
            asset_class: AssetClass::Equity,
            timeframe: Timeframe::OneDay,
            start_timestamp_ms: None,
            end_timestamp_ms: None,
            output_root: output_root.display().to_string(),
            limit_per_request: None,
            include_raw_archive: true,
            fill_missing_policy: soma_zero::FillMissingPolicy::LeaveGaps,
            fixture_path: None,
            adjusted_price_policy: AdjustedPricePolicy::Raw,
            collection_size_policy: CollectionSizePolicy::default(),
            auth_config: None,
            endpoint_template: None,
            requested_output_size: Some(RequestedOutputSize::Compact),
            allow_full_history_override: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        })
        .expect_err("missing api key");
    assert!(error.contains("MissingApiKey"));
}

#[test]
fn auth_config_renders_env_var_names_not_secret_values() {
    let auth = AuthConfig {
        provider_kind: ProviderKind::AlphaVantage,
        api_key_env_var: Some("ALPHAVANTAGE_API_KEY".to_string()),
        api_secret_env_var: Some("ALPACA_SECRET".to_string()),
        auth_header_name: Some("X-API-KEY".to_string()),
        query_param_name: Some("apikey".to_string()),
        allow_missing_for_mock: true,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    let rendered = auth.to_deterministic_string();
    assert!(rendered.contains("ALPHAVANTAGE_API_KEY"));
    assert!(!rendered.contains("super-secret-value"));
}

#[test]
fn krx_fixture_maps_to_canonical_and_writes_budget_report() {
    let fixture_path = provider_fixture_path("krx_daily_stock_response.json");
    let output_root = common::output_dir("collector-krx");
    let result = CollectorRunner::default()
        .run(&CandleFetchRequest {
            request_id: "collector-krx".to_string(),
            provider_kind: ProviderKind::KrxOpenApi,
            symbol: "005930".to_string(),
            market_venue: Some(MarketVenue::KOSPI),
            asset_class: AssetClass::Equity,
            timeframe: Timeframe::OneDay,
            start_timestamp_ms: Some(1_712_534_400_000),
            end_timestamp_ms: Some(1_712_707_200_000),
            output_root: output_root.display().to_string(),
            limit_per_request: None,
            include_raw_archive: true,
            fill_missing_policy: soma_zero::FillMissingPolicy::LeaveGaps,
            fixture_path: Some(fixture_path.display().to_string()),
            adjusted_price_policy: AdjustedPricePolicy::Raw,
            collection_size_policy: CollectionSizePolicy::default(),
            auth_config: Some(AuthConfig {
                provider_kind: ProviderKind::KrxOpenApi,
                api_key_env_var: Some("KRX_API_KEY".to_string()),
                api_secret_env_var: None,
                auth_header_name: Some("Authorization".to_string()),
                query_param_name: None,
                allow_missing_for_mock: true,
                reason_codes: vec![ReasonCode::DeterministicPath],
            }),
            endpoint_template: None,
            requested_output_size: Some(RequestedOutputSize::Compact),
            allow_full_history_override: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        })
        .expect("collect krx fixture");

    assert_eq!(result.provider_id, "krx");
    assert!(Path::new(&result.canonical_csv_path).exists());
    assert!(
        fs::read_to_string(&result.budget_report_path)
            .expect("read budget")
            .contains("collection_size_policy_summary=")
    );
    assert!(
        fs::read_to_string(&result.manifest_path)
            .expect("read manifest")
            .contains("provider_symbol=005930")
    );
}

#[test]
fn alphavantage_compact_fixture_enforces_row_cap() {
    let fixture_path = write_static_fixture(
        "alphavantage-compact-generated",
        build_alphavantage_daily_response(120),
    );
    let output_root = common::output_dir("collector-alphavantage");
    let mut size_policy = CollectionSizePolicy::default();
    size_policy.max_rows_per_symbol = 100;
    let result = CollectorRunner::default()
        .run(&CandleFetchRequest {
            request_id: "collector-alphavantage".to_string(),
            provider_kind: ProviderKind::AlphaVantage,
            symbol: "AAPL".to_string(),
            market_venue: Some(MarketVenue::NASDAQ),
            asset_class: AssetClass::Equity,
            timeframe: Timeframe::OneDay,
            start_timestamp_ms: None,
            end_timestamp_ms: None,
            output_root: output_root.display().to_string(),
            limit_per_request: None,
            include_raw_archive: true,
            fill_missing_policy: soma_zero::FillMissingPolicy::LeaveGaps,
            fixture_path: Some(fixture_path.display().to_string()),
            adjusted_price_policy: AdjustedPricePolicy::Adjusted,
            collection_size_policy: size_policy,
            auth_config: Some(AuthConfig {
                provider_kind: ProviderKind::AlphaVantage,
                api_key_env_var: Some("ALPHAVANTAGE_API_KEY".to_string()),
                api_secret_env_var: None,
                auth_header_name: None,
                query_param_name: Some("apikey".to_string()),
                allow_missing_for_mock: true,
                reason_codes: vec![ReasonCode::DeterministicPath],
            }),
            endpoint_template: None,
            requested_output_size: Some(RequestedOutputSize::Compact),
            allow_full_history_override: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        })
        .expect("collect alphavantage fixture");

    assert_eq!(result.row_count, 100);
    assert!(result.truncated);
    assert!(result.row_limit_applied);
    assert_eq!(result.preflight_status, PreflightFinalStatus::NeedsMoreRows);
    assert!(
        fs::read_to_string(&result.budget_report_path)
            .expect("read budget")
            .contains("row_limit_applied=true")
    );
    assert!(
        fs::read_to_string(&result.manifest_path)
            .expect("read manifest")
            .contains("adjusted_price_policy_summary=Adjusted")
    );
}

#[test]
fn cli_collect_candles_supports_mock_fixture_without_network() {
    let output_root = common::output_dir("collector-cli");
    let fixture_path = provider_fixture_path("alphavantage_daily_compact_response.json");
    let out_arg = output_root.display().to_string();
    let fixture_arg = fixture_path.display().to_string();
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .args([
            "collect-candles",
            "--provider",
            "mock-fixture",
            "--symbol",
            "AAPL",
            "--venue",
            "NASDAQ",
            "--timeframe",
            "1d",
            "--out",
            &out_arg,
            "--fixture",
            &fixture_arg,
            "--max-rows",
            "100",
        ])
        .output()
        .expect("run collect-candles");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("provider=mock-fixture"));
    assert!(stdout.contains("preflight_status=NotRealLocalEligible"));
}

fn write_fixture(name: &str, responses: Vec<serde_json::Value>) -> PathBuf {
    let dir = common::output_dir(&format!("{name}-fixture"));
    let path = dir.join(format!("{name}.json"));
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({ "responses": responses })).expect("fixture json"),
    )
    .expect("write fixture");
    path
}

fn write_static_fixture(name: &str, contents: String) -> PathBuf {
    let dir = common::output_dir(&format!("{name}-fixture"));
    let path = dir.join(format!("{name}.json"));
    fs::write(&path, contents).expect("write static fixture");
    path
}

fn build_upbit_response(
    symbol: &str,
    start_timestamp_ms: u64,
    count: usize,
    step_ms: u64,
) -> String {
    let rows = (0..count)
        .map(|offset| {
            let timestamp_ms = start_timestamp_ms + offset as u64 * step_ms;
            let price = 10_000.0 + offset as f64;
            json!({
                "market": symbol,
                "timestamp": timestamp_ms,
                "opening_price": price,
                "high_price": price + 1.0,
                "low_price": price - 1.0,
                "trade_price": price + 0.5,
                "candle_acc_trade_price": (price + 0.5) * 10.0,
                "candle_acc_trade_volume": 10.0
            })
        })
        .rev()
        .collect::<Vec<_>>();
    serde_json::to_string(&rows).expect("upbit response json")
}

fn build_alphavantage_daily_response(count: usize) -> String {
    let mut series = serde_json::Map::new();
    for offset in 0..count {
        let day = offset + 1;
        let month = 1 + (day - 1) / 28;
        let day_in_month = 1 + (day - 1) % 28;
        let date = format!("2024-{month:02}-{day_in_month:02}");
        let price = 150.0 + offset as f64;
        series.insert(
            date,
            json!({
                "1. open": format!("{price:.4}"),
                "2. high": format!("{:.4}", price + 1.0),
                "3. low": format!("{:.4}", price - 1.0),
                "4. close": format!("{:.4}", price + 0.5),
                "5. volume": format!("{}", 1_000_000 + offset),
            }),
        );
    }
    serde_json::to_string(&json!({
        "Meta Data": {
            "1. Information": "Daily Prices (open, high, low, close) and Volumes",
            "2. Symbol": "AAPL",
        },
        "Time Series (Daily)": series
    }))
    .expect("alphavantage response json")
}

fn to_param(timestamp_ms: u64) -> String {
    let seconds = timestamp_ms / 1_000;
    let days = (seconds / 86_400) as i64;
    let secs_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3_600;
    let minute = (secs_of_day % 3_600) / 60;
    let second = secs_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + i64::from(month <= 2);
    (year as i32, month as u32, day as u32)
}
