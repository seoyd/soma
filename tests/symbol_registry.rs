use soma_zero::{
    AssetClass, MarketVenue, ReasonCode, SymbolRegistry, SymbolSpec, Timeframe, TimeframeSpec,
};

#[test]
fn symbol_registry_normalizes_generic_symbol() {
    let mut registry = SymbolRegistry::default();
    let spec = registry
        .register_symbol(SymbolSpec::new(
            "btc/usdt",
            MarketVenue::Generic,
            AssetClass::Crypto,
        ))
        .expect("valid symbol");
    assert_eq!(spec.normalized_symbol, "BTCUSDT");
    assert_eq!(
        registry
            .lookup_symbol("BTC-USDT")
            .expect("registered")
            .normalized_symbol,
        "BTCUSDT"
    );
}

#[test]
fn symbol_registry_preserves_venue_and_asset_class() {
    let mut registry = SymbolRegistry::default();
    let spec = registry
        .register_symbol(SymbolSpec::new(
            "005930",
            MarketVenue::KRX,
            AssetClass::Equity,
        ))
        .expect("valid symbol");
    assert_eq!(spec.venue, MarketVenue::KRX);
    assert_eq!(spec.asset_class, AssetClass::Equity);
}

#[test]
fn timeframe_spec_maps_one_minute_to_expected_step() {
    let spec = TimeframeSpec::from_timeframe(Timeframe::OneMinute);
    assert_eq!(spec.seconds, 60);
    assert_eq!(spec.expected_ms_step, 60_000);
}

#[test]
fn unsupported_timeframe_emits_reason_code() {
    let spec = TimeframeSpec::from_timeframe(Timeframe::Custom { seconds: 0 });
    assert_eq!(spec.reason_codes, vec![ReasonCode::UnsupportedTimeframe]);
}
