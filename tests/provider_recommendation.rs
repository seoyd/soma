use soma_zero::{
    BudgetPreference, ProviderDataSubject, ProviderEntitlementStatus,
    ProviderEntitlementStatusKind, ProviderKind, ProviderMarket, ProviderRecommendationRequest,
    StrategyUseCase, recommend_provider,
};

fn status(
    subject: ProviderDataSubject,
    kind: ProviderEntitlementStatusKind,
) -> ProviderEntitlementStatus {
    ProviderEntitlementStatus {
        provider_subject: subject,
        freshness_available: Vec::new(),
        cost_tier: soma_zero::ProviderCostTier::Unknown,
        auth_ready: true,
        approval_ready: true,
        endpoint_template_ready: true,
        realtime_entitlement_ready: true,
        delayed_entitlement_ready: true,
        official_readiness_eligible: true,
        research_only: false,
        status: kind,
        reason_codes: Vec::new(),
    }
}

#[test]
fn korean_eod_recommends_krx_if_approved() {
    let recommendation = recommend_provider(
        &ProviderRecommendationRequest {
            market: ProviderMarket::KoreanEquity,
            desired_use_case: StrategyUseCase::EodSwing,
            budget_preference: BudgetPreference::FreeOnly,
            need_realtime: false,
            need_official_readiness: true,
            max_data_size_preference: None,
            reason_codes: Vec::new(),
        },
        &[status(
            ProviderDataSubject::Provider(ProviderKind::KrxOpenApi),
            ProviderEntitlementStatusKind::ReadyForEodResearch,
        )],
    );
    assert_eq!(
        recommendation.primary_provider,
        Some(ProviderDataSubject::Provider(ProviderKind::KrxOpenApi))
    );
}

#[test]
fn korean_eod_falls_back_to_data_go_kr_when_krx_missing() {
    let recommendation = recommend_provider(
        &ProviderRecommendationRequest {
            market: ProviderMarket::KoreanEquity,
            desired_use_case: StrategyUseCase::EodSwing,
            budget_preference: BudgetPreference::FreeOnly,
            need_realtime: false,
            need_official_readiness: true,
            max_data_size_preference: None,
            reason_codes: Vec::new(),
        },
        &[status(
            ProviderDataSubject::Provider(ProviderKind::KrxOpenApi),
            ProviderEntitlementStatusKind::MissingApproval,
        )],
    );
    assert_eq!(
        recommendation.primary_provider,
        Some(ProviderDataSubject::Provider(
            ProviderKind::DataGoKrFscStockPrice
        ))
    );
}

#[test]
fn us_eod_recommends_alphavantage_compact() {
    let recommendation = recommend_provider(
        &ProviderRecommendationRequest {
            market: ProviderMarket::USEquity,
            desired_use_case: StrategyUseCase::EodSwing,
            budget_preference: BudgetPreference::FreeOnly,
            need_realtime: false,
            need_official_readiness: true,
            max_data_size_preference: None,
            reason_codes: Vec::new(),
        },
        &[status(
            ProviderDataSubject::Provider(ProviderKind::AlphaVantage),
            ProviderEntitlementStatusKind::ReadyForEodResearch,
        )],
    );
    assert_eq!(
        recommendation.primary_provider,
        Some(ProviderDataSubject::Provider(ProviderKind::AlphaVantage))
    );
}

#[test]
fn us_realtime_free_only_recommends_alpaca_iex_with_warning() {
    let recommendation = recommend_provider(
        &ProviderRecommendationRequest {
            market: ProviderMarket::USEquity,
            desired_use_case: StrategyUseCase::RealtimeScalping,
            budget_preference: BudgetPreference::FreeOnly,
            need_realtime: true,
            need_official_readiness: false,
            max_data_size_preference: None,
            reason_codes: Vec::new(),
        },
        &[status(
            ProviderDataSubject::Provider(ProviderKind::Alpaca),
            ProviderEntitlementStatusKind::ReadyForRealtimeResearchIexOnly,
        )],
    );
    assert_eq!(
        recommendation.primary_provider,
        Some(ProviderDataSubject::Provider(ProviderKind::Alpaca))
    );
    assert!(
        recommendation
            .required_operator_actions
            .iter()
            .any(|item| item.contains("IEX"))
    );
}

#[test]
fn us_full_market_realtime_recommends_paid_provider_path() {
    let recommendation = recommend_provider(
        &ProviderRecommendationRequest {
            market: ProviderMarket::USEquity,
            desired_use_case: StrategyUseCase::RealtimeExecutionSimulation,
            budget_preference: BudgetPreference::PaidAllowed,
            need_realtime: true,
            need_official_readiness: false,
            max_data_size_preference: None,
            reason_codes: Vec::new(),
        },
        &[status(
            ProviderDataSubject::Provider(ProviderKind::Alpaca),
            ProviderEntitlementStatusKind::ReadyForRealtimeResearchIexOnly,
        )],
    );
    assert_eq!(
        recommendation.status,
        soma_zero::ProviderRecommendationStatus::NeedPaidProvider
    );
}

#[test]
fn yfinance_is_never_selected_for_official_readiness() {
    let recommendation = recommend_provider(
        &ProviderRecommendationRequest {
            market: ProviderMarket::USEquity,
            desired_use_case: StrategyUseCase::EodSwing,
            budget_preference: BudgetPreference::FreeOnly,
            need_realtime: false,
            need_official_readiness: true,
            max_data_size_preference: None,
            reason_codes: Vec::new(),
        },
        &[],
    );
    assert_ne!(
        recommendation.primary_provider,
        Some(ProviderDataSubject::YFinanceResearch)
    );
    assert!(
        recommendation
            .research_fallbacks
            .iter()
            .any(|item| item == "yfinance")
    );
}

#[test]
fn crypto_intraday_recommends_upbit() {
    let recommendation = recommend_provider(
        &ProviderRecommendationRequest {
            market: ProviderMarket::Crypto,
            desired_use_case: StrategyUseCase::IntradaySwing,
            budget_preference: BudgetPreference::FreeOnly,
            need_realtime: true,
            need_official_readiness: false,
            max_data_size_preference: None,
            reason_codes: Vec::new(),
        },
        &[],
    );
    assert_eq!(
        recommendation.primary_provider,
        Some(ProviderDataSubject::Provider(ProviderKind::Upbit))
    );
}

#[test]
fn provider_recommendation_is_deterministic() {
    let request = ProviderRecommendationRequest {
        market: ProviderMarket::USEquity,
        desired_use_case: StrategyUseCase::EodSwing,
        budget_preference: BudgetPreference::FreeOnly,
        need_realtime: false,
        need_official_readiness: true,
        max_data_size_preference: None,
        reason_codes: Vec::new(),
    };
    let first = recommend_provider(&request, &[]);
    let second = recommend_provider(&request, &[]);
    assert_eq!(first, second);
}
