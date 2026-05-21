mod common;

use soma_zero::{
    AlpacaHistoricalBarsImportConfig, AlpacaProviderStatus, parse_alpaca_historical_bars_fixture,
    run_alpaca_historical_bars_import,
};

fn fixture_json() -> &'static str {
    r#"
{"bars":[
  {"t":"2024-01-02T00:00:00Z","o":100.0,"h":110.0,"l":99.0,"c":105.0,"v":1000.0},
  {"t":"2024-01-03T00:00:00Z","o":105.0,"h":111.0,"l":104.0,"c":109.0,"v":1200.0}
]}
"#
}

fn unique(name: &str) -> String {
    format!("SOMA_TEST_{name}")
}

#[test]
fn alpaca_fixture_parser_maps_to_canonical_ohlcv() {
    let candles = parse_alpaca_historical_bars_fixture(fixture_json()).expect("parse");
    assert_eq!(candles.len(), 2);
    assert_eq!(candles[0].open, 100.0);
    assert_eq!(candles[1].close, 109.0);
}

#[test]
fn alpaca_missing_auth_is_reason_coded() {
    let output_dir = common::output_dir("alpaca-missing-auth");
    let fixture_path = output_dir.join("fixture.json");
    std::fs::write(&fixture_path, fixture_json()).expect("write fixture");
    let key = unique("ALPACA_KEY_ID");
    let secret = unique("ALPACA_SECRET_KEY");
    unsafe {
        std::env::remove_var(&key);
        std::env::remove_var(&secret);
    }

    let report = run_alpaca_historical_bars_import(&AlpacaHistoricalBarsImportConfig {
        import_id: "alpaca-missing-auth".to_string(),
        fixture_path: fixture_path.display().to_string(),
        output_root: output_dir.display().to_string(),
        symbol: "AAPL".to_string(),
        api_key_env_var: key,
        api_secret_env_var: secret,
        max_rows: 10,
        reason_codes: Vec::new(),
    })
    .expect("report");

    assert_eq!(report.status, AlpacaProviderStatus::MissingAuth);
    assert!(
        report
            .reason_codes
            .contains(&soma_zero::ReasonCode::MissingAuth)
    );
}

#[test]
fn alpaca_limit_enforcement_truncates_rows() {
    let output_dir = common::output_dir("alpaca-limit");
    let fixture_path = output_dir.join("fixture.json");
    std::fs::write(&fixture_path, fixture_json()).expect("write fixture");
    let key = unique("ALPACA_KEY_ID_PRESENT");
    let secret = unique("ALPACA_SECRET_KEY_PRESENT");
    unsafe {
        std::env::set_var(&key, "present");
        std::env::set_var(&secret, "present");
    }

    let report = run_alpaca_historical_bars_import(&AlpacaHistoricalBarsImportConfig {
        import_id: "alpaca-limit".to_string(),
        fixture_path: fixture_path.display().to_string(),
        output_root: output_dir.display().to_string(),
        symbol: "AAPL".to_string(),
        api_key_env_var: key.clone(),
        api_secret_env_var: secret.clone(),
        max_rows: 1,
        reason_codes: Vec::new(),
    })
    .expect("report");

    assert_eq!(report.status, AlpacaProviderStatus::Ready);
    assert_eq!(report.row_count, 1);
    unsafe {
        std::env::remove_var(&key);
        std::env::remove_var(&secret);
    }
}
