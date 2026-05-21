use soma_zero::{
    DataFreshnessTier, ProviderDataSubject, ProviderKind, default_provider_freshness_profiles,
    provider_freshness_profile,
};

#[test]
fn krx_is_classified_as_eod_historical_with_approval_note() {
    let profile =
        provider_freshness_profile(ProviderDataSubject::Provider(ProviderKind::KrxOpenApi));
    assert_eq!(profile.default_freshness, DataFreshnessTier::Eod);
    assert!(
        profile
            .available_freshness_tiers
            .contains(&DataFreshnessTier::Historical)
    );
    assert!(profile.notes.iter().any(|note| note.contains("approval")));
}

#[test]
fn data_go_kr_is_eod_historical() {
    let profile = provider_freshness_profile(ProviderDataSubject::Provider(
        ProviderKind::DataGoKrFscStockPrice,
    ));
    assert_eq!(profile.default_freshness, DataFreshnessTier::Eod);
    assert!(
        profile
            .available_freshness_tiers
            .contains(&DataFreshnessTier::Historical)
    );
}

#[test]
fn alphavantage_default_is_eod_and_realtime_requires_entitlement() {
    let profile =
        provider_freshness_profile(ProviderDataSubject::Provider(ProviderKind::AlphaVantage));
    assert_eq!(profile.default_freshness, DataFreshnessTier::Eod);
    assert!(
        profile
            .available_freshness_tiers
            .contains(&DataFreshnessTier::Historical)
    );
    assert!(profile.requires_entitlement_for_realtime);
    assert!(profile.requires_entitlement_for_delayed);
}

#[test]
fn alpaca_includes_iex_and_sip_tiers() {
    let profile = provider_freshness_profile(ProviderDataSubject::Provider(ProviderKind::Alpaca));
    assert_eq!(profile.default_freshness, DataFreshnessTier::RealtimeIex);
    assert!(
        profile
            .available_freshness_tiers
            .contains(&DataFreshnessTier::RealtimeSip)
    );
}

#[test]
fn upbit_is_realtime_crypto_public() {
    let profile = provider_freshness_profile(ProviderDataSubject::Provider(ProviderKind::Upbit));
    assert_eq!(
        profile.default_freshness,
        DataFreshnessTier::RealtimeCryptoPublic
    );
}

#[test]
fn yfinance_is_research_only() {
    let profile = provider_freshness_profile(ProviderDataSubject::YFinanceResearch);
    assert_eq!(profile.default_freshness, DataFreshnessTier::ResearchOnly);
}

#[test]
fn freshness_profiles_are_deterministic() {
    let first = serde_json::to_string(&default_provider_freshness_profiles()).expect("json");
    let second = serde_json::to_string(&default_provider_freshness_profiles()).expect("json");
    assert_eq!(first, second);
}
