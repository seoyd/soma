use soma_zero::{
    ProviderDataSubject, ProviderEntitlementStatus, ProviderEntitlementStatusKind, ProviderKind,
    StrategyUseCase, evaluate_strategy_data_compatibility,
};

fn limited_alpaca_status() -> ProviderEntitlementStatus {
    ProviderEntitlementStatus {
        provider_subject: ProviderDataSubject::Provider(ProviderKind::Alpaca),
        freshness_available: vec![soma_zero::DataFreshnessTier::RealtimeIex],
        cost_tier: soma_zero::ProviderCostTier::FreeWithLimits,
        auth_ready: true,
        approval_ready: true,
        endpoint_template_ready: true,
        realtime_entitlement_ready: true,
        delayed_entitlement_ready: true,
        official_readiness_eligible: true,
        research_only: false,
        status: ProviderEntitlementStatusKind::ReadyForRealtimeResearchIexOnly,
        reason_codes: Vec::new(),
    }
}

#[test]
fn alphavantage_compact_is_compatible_with_eod_swing() {
    let result = evaluate_strategy_data_compatibility(
        ProviderDataSubject::Provider(ProviderKind::AlphaVantage),
        StrategyUseCase::EodSwing,
        None,
    );
    assert!(result.compatible);
}

#[test]
fn alphavantage_compact_is_incompatible_with_realtime_scalping() {
    let result = evaluate_strategy_data_compatibility(
        ProviderDataSubject::Provider(ProviderKind::AlphaVantage),
        StrategyUseCase::RealtimeScalping,
        None,
    );
    assert!(!result.compatible);
}

#[test]
fn krx_eod_is_compatible_with_korean_eod_research() {
    let result = evaluate_strategy_data_compatibility(
        ProviderDataSubject::Provider(ProviderKind::KrxOpenApi),
        StrategyUseCase::EodSwing,
        None,
    );
    assert!(result.compatible);
}

#[test]
fn krx_eod_is_incompatible_with_intraday_scalping() {
    let result = evaluate_strategy_data_compatibility(
        ProviderDataSubject::Provider(ProviderKind::KrxOpenApi),
        StrategyUseCase::RealtimeScalping,
        None,
    );
    assert!(!result.compatible);
}

#[test]
fn alpaca_iex_is_compatible_with_limited_realtime_research() {
    let result = evaluate_strategy_data_compatibility(
        ProviderDataSubject::Provider(ProviderKind::Alpaca),
        StrategyUseCase::RealtimeScalping,
        Some(&limited_alpaca_status()),
    );
    assert!(result.compatible);
    assert!(
        result
            .limitations
            .iter()
            .any(|item| item.contains("IEX-only"))
    );
}

#[test]
fn alpaca_iex_is_incompatible_with_full_market_execution_claims() {
    let result = evaluate_strategy_data_compatibility(
        ProviderDataSubject::Provider(ProviderKind::Alpaca),
        StrategyUseCase::RealtimeExecutionSimulation,
        Some(&limited_alpaca_status()),
    );
    assert!(!result.compatible);
}

#[test]
fn yfinance_is_only_compatible_with_comparison_and_prototype_work() {
    let comparison = evaluate_strategy_data_compatibility(
        ProviderDataSubject::YFinanceResearch,
        StrategyUseCase::SourceComparison,
        None,
    );
    let eod = evaluate_strategy_data_compatibility(
        ProviderDataSubject::YFinanceResearch,
        StrategyUseCase::EodSwing,
        None,
    );
    assert!(comparison.compatible);
    assert!(!eod.compatible);
}

#[test]
fn compatibility_is_deterministic() {
    let first = evaluate_strategy_data_compatibility(
        ProviderDataSubject::Provider(ProviderKind::AlphaVantage),
        StrategyUseCase::EodSwing,
        None,
    );
    let second = evaluate_strategy_data_compatibility(
        ProviderDataSubject::Provider(ProviderKind::AlphaVantage),
        StrategyUseCase::EodSwing,
        None,
    );
    assert_eq!(first, second);
}
