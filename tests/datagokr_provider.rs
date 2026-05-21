mod common;

use soma_zero::{
    DataGoKrFscStockPriceImportConfig, DataGoKrProviderStatus,
    parse_datagokr_fsc_stock_price_fixture, run_datagokr_fsc_stock_price_import,
};

fn fixture_json() -> &'static str {
    r#"
[
  {"basDt":"20240102","mkp":"1000","hipr":"1100","lopr":"990","clpr":"1050","trqu":"12000"},
  {"basDt":"20240103","mkp":"1050","hipr":"1110","lopr":"1040","clpr":"1090","trqu":"15000"}
]
"#
}

fn unique(name: &str) -> String {
    format!("SOMA_TEST_{name}")
}

#[test]
fn datagokr_fixture_parser_maps_to_canonical_ohlcv() {
    let candles = parse_datagokr_fsc_stock_price_fixture(fixture_json()).expect("parse");
    assert_eq!(candles.len(), 2);
    assert_eq!(candles[0].open, 1000.0);
    assert_eq!(candles[1].close, 1090.0);
    assert_eq!(candles[1].volume, 15000.0);
}

#[test]
fn datagokr_missing_service_key_is_reason_coded() {
    let output_dir = common::output_dir("datagokr-missing-auth");
    let fixture_path = output_dir.join("fixture.json");
    std::fs::write(&fixture_path, fixture_json()).expect("write fixture");
    let key = unique("DATAGOKR_SERVICE_KEY");
    unsafe { std::env::remove_var(&key) };

    let report = run_datagokr_fsc_stock_price_import(&DataGoKrFscStockPriceImportConfig {
        import_id: "datagokr-missing-auth".to_string(),
        fixture_path: fixture_path.display().to_string(),
        output_root: output_dir.display().to_string(),
        symbol: "005930".to_string(),
        service_key_env_var: key,
        endpoint_profile: Some("approved-v0".to_string()),
        reason_codes: Vec::new(),
    })
    .expect("report");

    assert_eq!(report.status, DataGoKrProviderStatus::MissingAuth);
    assert!(
        report
            .reason_codes
            .contains(&soma_zero::ReasonCode::MissingAuth)
    );
}

#[test]
fn datagokr_missing_endpoint_profile_does_not_guess_silently() {
    let output_dir = common::output_dir("datagokr-missing-endpoint");
    let fixture_path = output_dir.join("fixture.json");
    std::fs::write(&fixture_path, fixture_json()).expect("write fixture");
    let key = unique("DATAGOKR_SERVICE_KEY_PRESENT");
    unsafe { std::env::set_var(&key, "present") };

    let report = run_datagokr_fsc_stock_price_import(&DataGoKrFscStockPriceImportConfig {
        import_id: "datagokr-missing-endpoint".to_string(),
        fixture_path: fixture_path.display().to_string(),
        output_root: output_dir.display().to_string(),
        symbol: "005930".to_string(),
        service_key_env_var: key.clone(),
        endpoint_profile: None,
        reason_codes: Vec::new(),
    })
    .expect("report");

    assert_eq!(
        report.status,
        DataGoKrProviderStatus::DeferredUntilEndpointProfile
    );
    assert!(
        report
            .reason_codes
            .contains(&soma_zero::ReasonCode::DataGoKrEndpointProfileMissing)
    );
    unsafe { std::env::remove_var(&key) };
}
