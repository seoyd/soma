use soma_zero::{ProviderCostTier, ProviderDataSubject, ProviderKind, provider_cost_profile};

#[test]
fn krx_requires_approval_and_has_commercial_warning() {
    let profile = provider_cost_profile(ProviderDataSubject::Provider(ProviderKind::KrxOpenApi));
    assert_eq!(profile.cost_tier, ProviderCostTier::RequiresApproval);
    assert!(profile.approval_required);
    assert!(profile.commercial_use_warning);
}

#[test]
fn alphavantage_free_and_premium_are_both_represented() {
    let profile = provider_cost_profile(ProviderDataSubject::Provider(ProviderKind::AlphaVantage));
    assert_eq!(profile.cost_tier, ProviderCostTier::FreeWithLimits);
    assert!(
        profile
            .subscription_required_for
            .iter()
            .any(|item| item.contains("premium"))
    );
}

#[test]
fn alpaca_free_iex_and_paid_sip_are_represented() {
    let profile = provider_cost_profile(ProviderDataSubject::Provider(ProviderKind::Alpaca));
    assert_eq!(profile.cost_tier, ProviderCostTier::FreeWithLimits);
    assert!(
        profile
            .subscription_required_for
            .iter()
            .any(|item| item.contains("SIP"))
    );
}

#[test]
fn data_go_kr_service_key_profile_is_represented() {
    let profile = provider_cost_profile(ProviderDataSubject::Provider(
        ProviderKind::DataGoKrFscStockPrice,
    ));
    assert_eq!(profile.cost_tier, ProviderCostTier::FreeWithLimits);
    assert!(profile.free_limits_summary.is_some());
}

#[test]
fn yfinance_is_not_official_provider_cost_profile() {
    let profile = provider_cost_profile(ProviderDataSubject::YFinanceResearch);
    assert_eq!(profile.cost_tier, ProviderCostTier::FreeWithLimits);
    let json = serde_json::to_string(&profile).expect("json");
    assert!(!json.contains("secret-value"));
}
