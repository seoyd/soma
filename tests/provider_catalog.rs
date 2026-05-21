use soma_zero::{
    ProviderKind, ProviderMarket, ProviderSourceClass, build_default_provider_catalog,
};

fn entry(provider_kind: ProviderKind) -> soma_zero::ProviderCatalogEntry {
    build_default_provider_catalog()
        .entry(provider_kind)
        .cloned()
        .expect("catalog entry")
}

#[test]
fn korean_equity_catalog_contains_krx_as_official_exchange_api() {
    let entry = entry(ProviderKind::KrxOpenApi);
    assert_eq!(entry.market, ProviderMarket::KoreanEquity);
    assert_eq!(entry.source_class, ProviderSourceClass::OfficialExchangeApi);
    assert!(entry.official_readiness_eligible);
}

#[test]
fn korean_equity_catalog_contains_data_go_kr_as_government_api() {
    let entry = entry(ProviderKind::DataGoKrFscStockPrice);
    assert_eq!(entry.market, ProviderMarket::KoreanEquity);
    assert_eq!(
        entry.source_class,
        ProviderSourceClass::PublicGovernmentDataApi
    );
}

#[test]
fn korean_equity_catalog_contains_kis_as_broker_market_data_only() {
    let entry = entry(ProviderKind::KoreaInvestmentMarketData);
    assert_eq!(entry.market, ProviderMarket::KoreanEquity);
    assert_eq!(entry.source_class, ProviderSourceClass::BrokerMarketDataApi);
    assert!(
        entry
            .notes
            .iter()
            .any(|note| note.contains("market-data-only"))
    );
}

#[test]
fn us_equity_catalog_contains_alphavantage_alpaca_and_professional_cards() {
    let catalog = build_default_provider_catalog();
    assert_eq!(
        catalog.entry(ProviderKind::AlphaVantage).unwrap().market,
        ProviderMarket::USEquity
    );
    assert_eq!(
        catalog.entry(ProviderKind::Alpaca).unwrap().market,
        ProviderMarket::USEquity
    );
    assert_eq!(
        catalog
            .entry(ProviderKind::PolygonProfessional)
            .unwrap()
            .source_class,
        ProviderSourceClass::ProfessionalMarketDataApi
    );
    assert_eq!(
        catalog
            .entry(ProviderKind::NasdaqDataLink)
            .unwrap()
            .source_class,
        ProviderSourceClass::ProfessionalMarketDataApi
    );
}

#[test]
fn yfinance_is_not_present_as_official_provider() {
    let catalog = build_default_provider_catalog();
    assert!(
        !catalog
            .providers
            .iter()
            .any(|entry| entry.provider_name.contains("yfinance"))
    );
}

#[test]
fn provider_catalog_ordering_is_deterministic() {
    let first = build_default_provider_catalog().to_text();
    let second = build_default_provider_catalog().to_text();
    assert_eq!(first, second);
}
