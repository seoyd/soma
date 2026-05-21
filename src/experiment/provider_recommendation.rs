use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{
    ProviderDataSubject, ProviderEntitlementStatus, ProviderEntitlementStatusKind, ProviderKind,
    ProviderMarket,
};

use super::strategy_compatibility::StrategyUseCase;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetPreference {
    FreeOnly,
    FreeOrLowCost,
    PaidAllowed,
    ProfessionalAllowed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderRecommendationStatus {
    Ready,
    MissingAuth,
    MissingApproval,
    NeedPaidProvider,
    ResearchOnlyAvailable,
    NoSuitableProvider,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRecommendationRequest {
    pub market: ProviderMarket,
    pub desired_use_case: StrategyUseCase,
    pub budget_preference: BudgetPreference,
    pub need_realtime: bool,
    pub need_official_readiness: bool,
    pub max_data_size_preference: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRecommendation {
    pub primary_provider: Option<ProviderDataSubject>,
    pub fallback_providers: Vec<ProviderDataSubject>,
    pub research_fallbacks: Vec<String>,
    pub rejected_providers: Vec<String>,
    pub required_operator_actions: Vec<String>,
    pub status: ProviderRecommendationStatus,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn recommend_provider(
    request: &ProviderRecommendationRequest,
    entitlement_statuses: &[ProviderEntitlementStatus],
) -> ProviderRecommendation {
    let mut recommendation = match request.market {
        ProviderMarket::KoreanEquity => recommend_korean(request, entitlement_statuses),
        ProviderMarket::USEquity => recommend_us(request, entitlement_statuses),
        ProviderMarket::Crypto => recommend_crypto(request),
        ProviderMarket::GlobalEquity => ProviderRecommendation {
            primary_provider: None,
            fallback_providers: Vec::new(),
            research_fallbacks: Vec::new(),
            rejected_providers: vec!["global-equity is not a concrete provider lane".to_string()],
            required_operator_actions: Vec::new(),
            status: ProviderRecommendationStatus::NoSuitableProvider,
            reason_codes: vec![ReasonCode::ProviderRecommendationBuilt],
        },
    };
    recommendation
        .reason_codes
        .push(ReasonCode::ProviderRecommendationBuilt);
    recommendation
}

fn recommend_korean(
    request: &ProviderRecommendationRequest,
    entitlement_statuses: &[ProviderEntitlementStatus],
) -> ProviderRecommendation {
    let krx = find_status(
        entitlement_statuses,
        ProviderDataSubject::Provider(ProviderKind::KrxOpenApi),
    );
    let datagokr = ProviderDataSubject::Provider(ProviderKind::DataGoKrFscStockPrice);
    let kis = ProviderDataSubject::Provider(ProviderKind::KoreaInvestmentMarketData);

    if request.desired_use_case == StrategyUseCase::EodSwing {
        if krx.is_some_and(|status| {
            status.status == ProviderEntitlementStatusKind::ReadyForEodResearch
        }) {
            return ProviderRecommendation {
                primary_provider: Some(ProviderDataSubject::Provider(ProviderKind::KrxOpenApi)),
                fallback_providers: vec![datagokr, kis],
                research_fallbacks: Vec::new(),
                rejected_providers: Vec::new(),
                required_operator_actions: Vec::new(),
                status: ProviderRecommendationStatus::Ready,
                reason_codes: vec![ReasonCode::ProviderRecommendationBuilt],
            };
        }
        return ProviderRecommendation {
            primary_provider: Some(datagokr),
            fallback_providers: vec![kis],
            research_fallbacks: Vec::new(),
            rejected_providers: vec!["krx approval pending or auth missing".to_string()],
            required_operator_actions: vec![
                "wait for KRX approval or use data.go.kr as bounded fallback".to_string(),
            ],
            status: if krx.is_some_and(|status| {
                status.status == ProviderEntitlementStatusKind::MissingApproval
            }) {
                ProviderRecommendationStatus::MissingApproval
            } else {
                ProviderRecommendationStatus::MissingAuth
            },
            reason_codes: vec![ReasonCode::ProviderRecommendationBuilt],
        };
    }

    ProviderRecommendation {
        primary_provider: Some(kis),
        fallback_providers: vec![datagokr],
        research_fallbacks: Vec::new(),
        rejected_providers: vec!["KRX default EOD profile is not intraday scalping data".to_string()],
        required_operator_actions: vec!["keep Korean intraday/realtime work deferred until explicit entitled market-data profile exists".to_string()],
        status: ProviderRecommendationStatus::NeedPaidProvider,
        reason_codes: vec![ReasonCode::ProviderRecommendationBuilt],
    }
}

fn recommend_us(
    request: &ProviderRecommendationRequest,
    entitlement_statuses: &[ProviderEntitlementStatus],
) -> ProviderRecommendation {
    let alpha = find_status(
        entitlement_statuses,
        ProviderDataSubject::Provider(ProviderKind::AlphaVantage),
    );
    let alpaca = find_status(
        entitlement_statuses,
        ProviderDataSubject::Provider(ProviderKind::Alpaca),
    );

    match request.desired_use_case {
        StrategyUseCase::EodSwing | StrategyUseCase::DailyPortfolioResearch => {
            ProviderRecommendation {
                primary_provider: Some(ProviderDataSubject::Provider(ProviderKind::AlphaVantage)),
                fallback_providers: vec![ProviderDataSubject::Provider(ProviderKind::Alpaca)],
                research_fallbacks: vec!["yfinance".to_string()],
                rejected_providers: Vec::new(),
                required_operator_actions: vec![
                    "use AlphaVantage compact/daily only; do not treat it as realtime".to_string(),
                ],
                status: if alpha.is_some_and(|status| {
                    status.status == ProviderEntitlementStatusKind::ReadyForEodResearch
                }) {
                    ProviderRecommendationStatus::Ready
                } else {
                    ProviderRecommendationStatus::MissingAuth
                },
                reason_codes: vec![ReasonCode::AlphaVantageEodOnly],
            }
        }
        StrategyUseCase::RealtimeScalping => {
            if request.budget_preference == BudgetPreference::FreeOnly {
                ProviderRecommendation {
                    primary_provider: Some(ProviderDataSubject::Provider(ProviderKind::Alpaca)),
                    fallback_providers: vec![ProviderDataSubject::Provider(ProviderKind::PolygonProfessional)],
                    research_fallbacks: Vec::new(),
                    rejected_providers: vec!["AlphaVantage compact is not realtime".to_string()],
                    required_operator_actions: vec!["use Alpaca Basic only for IEX-limited realtime research; paid SIP/Polygon is needed for fuller coverage".to_string()],
                    status: if alpaca.is_some_and(|status| {
                        matches!(
                            status.status,
                            ProviderEntitlementStatusKind::ReadyForRealtimeResearch
                                | ProviderEntitlementStatusKind::ReadyForRealtimeResearchIexOnly
                        )
                    }) {
                        ProviderRecommendationStatus::Ready
                    } else {
                        ProviderRecommendationStatus::MissingAuth
                    },
                    reason_codes: vec![ReasonCode::AlpacaIexLimited],
                }
            } else {
                ProviderRecommendation {
                    primary_provider: Some(ProviderDataSubject::Provider(ProviderKind::Alpaca)),
                    fallback_providers: vec![ProviderDataSubject::Provider(
                        ProviderKind::PolygonProfessional,
                    )],
                    research_fallbacks: Vec::new(),
                    rejected_providers: vec!["yfinance is research-only".to_string()],
                    required_operator_actions: vec![
                        "use Alpaca paid SIP or Polygon for broader realtime coverage".to_string(),
                    ],
                    status: ProviderRecommendationStatus::NeedPaidProvider,
                    reason_codes: vec![ReasonCode::FullMarketCoverageUnavailable],
                }
            }
        }
        StrategyUseCase::RealtimeExecutionSimulation | StrategyUseCase::IntradaySwing => {
            ProviderRecommendation {
                primary_provider: Some(ProviderDataSubject::Provider(ProviderKind::Alpaca)),
                fallback_providers: vec![ProviderDataSubject::Provider(
                    ProviderKind::PolygonProfessional,
                )],
                research_fallbacks: Vec::new(),
                rejected_providers: vec!["AlphaVantage compact/free is not realtime".to_string()],
                required_operator_actions: vec![
                    "upgrade to Alpaca paid SIP or Polygon for full-market realtime work"
                        .to_string(),
                ],
                status: ProviderRecommendationStatus::NeedPaidProvider,
                reason_codes: vec![ReasonCode::FullMarketCoverageUnavailable],
            }
        }
        StrategyUseCase::SourceComparison | StrategyUseCase::ModelPrototypeResearch => {
            ProviderRecommendation {
                primary_provider: Some(ProviderDataSubject::Provider(ProviderKind::AlphaVantage)),
                fallback_providers: vec![ProviderDataSubject::Provider(ProviderKind::Alpaca)],
                research_fallbacks: vec!["yfinance".to_string()],
                rejected_providers: Vec::new(),
                required_operator_actions: vec![
                    "keep yfinance in research-only comparison mode".to_string(),
                ],
                status: ProviderRecommendationStatus::Ready,
                reason_codes: vec![ReasonCode::YFinanceResearchOnly],
            }
        }
    }
}

fn recommend_crypto(request: &ProviderRecommendationRequest) -> ProviderRecommendation {
    ProviderRecommendation {
        primary_provider: Some(ProviderDataSubject::Provider(ProviderKind::Upbit)),
        fallback_providers: vec![
            ProviderDataSubject::Provider(ProviderKind::Binance),
            ProviderDataSubject::Provider(ProviderKind::Korbit),
        ],
        research_fallbacks: Vec::new(),
        rejected_providers: Vec::new(),
        required_operator_actions: vec![
            "keep crypto collection bounded and public-endpoint-only".to_string(),
        ],
        status: if matches!(
            request.desired_use_case,
            StrategyUseCase::RealtimeScalping
                | StrategyUseCase::IntradaySwing
                | StrategyUseCase::RealtimeExecutionSimulation
        ) {
            ProviderRecommendationStatus::Ready
        } else {
            ProviderRecommendationStatus::Ready
        },
        reason_codes: vec![ReasonCode::ProviderRecommendationBuilt],
    }
}

fn find_status<'a>(
    entitlement_statuses: &'a [ProviderEntitlementStatus],
    subject: ProviderDataSubject,
) -> Option<&'a ProviderEntitlementStatus> {
    entitlement_statuses
        .iter()
        .find(|status| status.provider_subject == subject)
}
