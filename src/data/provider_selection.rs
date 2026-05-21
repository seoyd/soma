use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::ProviderKind;
use super::credential_profiles::{ProviderCredentialStatus, ProviderCredentialStatusKind};
use super::provider_catalog::{
    MarketDataProviderCatalog, ProviderImplementedStatus, ProviderMarket, ProviderSourceClass,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderSelectionResultStatus {
    Selected,
    MissingAuth,
    Deferred,
    NoEligibleProvider,
    ResearchOnlyFallback,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderSelectionPolicy {
    pub market: ProviderMarket,
    pub preferred_provider: ProviderKind,
    pub fallback_providers: Vec<ProviderKind>,
    pub allow_research_supplemental: bool,
    pub allow_professional_paid: bool,
    pub require_official_for_readiness: bool,
    pub max_providers_per_market: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderSelectionResult {
    pub market: ProviderMarket,
    pub selected_provider: Option<ProviderKind>,
    pub fallback_selected: Option<ProviderKind>,
    pub missing_auth_providers: Vec<ProviderKind>,
    pub deferred_providers: Vec<ProviderKind>,
    pub documented_only_providers: Vec<ProviderKind>,
    pub status: ProviderSelectionResultStatus,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn default_provider_selection_policies() -> Vec<ProviderSelectionPolicy> {
    vec![
        ProviderSelectionPolicy {
            market: ProviderMarket::Crypto,
            preferred_provider: ProviderKind::Upbit,
            fallback_providers: vec![ProviderKind::Binance, ProviderKind::Korbit],
            allow_research_supplemental: false,
            allow_professional_paid: false,
            require_official_for_readiness: true,
            max_providers_per_market: 3,
            reason_codes: vec![ReasonCode::ProviderSelectionBuilt],
        },
        ProviderSelectionPolicy {
            market: ProviderMarket::KoreanEquity,
            preferred_provider: ProviderKind::KoreaInvestmentMarketData,
            fallback_providers: vec![
                ProviderKind::KrxOpenApi,
                ProviderKind::DataGoKrFscStockPrice,
                ProviderKind::KoscomProfessional,
            ],
            allow_research_supplemental: false,
            allow_professional_paid: true,
            require_official_for_readiness: true,
            max_providers_per_market: 4,
            reason_codes: vec![ReasonCode::ProviderSelectionBuilt],
        },
        ProviderSelectionPolicy {
            market: ProviderMarket::USEquity,
            preferred_provider: ProviderKind::KoreaInvestmentMarketData,
            fallback_providers: vec![
                ProviderKind::AlphaVantage,
                ProviderKind::Alpaca,
                ProviderKind::PolygonProfessional,
                ProviderKind::NasdaqDataLink,
            ],
            allow_research_supplemental: true,
            allow_professional_paid: true,
            require_official_for_readiness: true,
            max_providers_per_market: 4,
            reason_codes: vec![ReasonCode::ProviderSelectionBuilt],
        },
    ]
}

pub fn select_provider(
    catalog: &MarketDataProviderCatalog,
    statuses: &[ProviderCredentialStatus],
    policy: &ProviderSelectionPolicy,
) -> ProviderSelectionResult {
    let mut selected_provider = None;
    let mut fallback_selected = None;
    let mut missing_auth_providers = Vec::new();
    let mut deferred_providers = Vec::new();
    let mut documented_only_providers = Vec::new();
    let mut reason_codes = vec![ReasonCode::ProviderSelectionBuilt];

    let ordered_candidates = std::iter::once(policy.preferred_provider)
        .chain(policy.fallback_providers.iter().copied())
        .take(policy.max_providers_per_market.max(1))
        .collect::<Vec<_>>();

    for provider_kind in ordered_candidates {
        let Some(entry) = catalog.entry(provider_kind) else {
            deferred_providers.push(provider_kind);
            continue;
        };
        if !supports_market(entry.market, provider_kind, policy.market) {
            continue;
        }
        if is_professional(entry.source_class) && !policy.allow_professional_paid {
            deferred_providers.push(provider_kind);
            continue;
        }
        if matches!(
            entry.implemented_status,
            ProviderImplementedStatus::Deferred | ProviderImplementedStatus::Foundation
        ) {
            deferred_providers.push(provider_kind);
            continue;
        }
        if entry.implemented_status == ProviderImplementedStatus::DocumentedOnly {
            documented_only_providers.push(provider_kind);
            continue;
        }
        let Some(status) = statuses
            .iter()
            .find(|status| status.provider_kind == provider_kind)
        else {
            missing_auth_providers.push(provider_kind);
            continue;
        };
        match status.status {
            ProviderCredentialStatusKind::Ready | ProviderCredentialStatusKind::NotRequired => {
                if selected_provider.is_none() {
                    selected_provider = Some(provider_kind);
                } else if fallback_selected.is_none() {
                    fallback_selected = Some(provider_kind);
                }
                break;
            }
            ProviderCredentialStatusKind::MissingAuth
            | ProviderCredentialStatusKind::MissingEndpointTemplate => {
                missing_auth_providers.push(provider_kind);
            }
            ProviderCredentialStatusKind::Deferred => deferred_providers.push(provider_kind),
        }
    }

    let status = if selected_provider.is_some() {
        ProviderSelectionResultStatus::Selected
    } else if policy.market == ProviderMarket::USEquity
        && policy.allow_research_supplemental
        && !missing_auth_providers.is_empty()
    {
        reason_codes.push(ReasonCode::ProviderSelectionResearchFallback);
        ProviderSelectionResultStatus::ResearchOnlyFallback
    } else if !missing_auth_providers.is_empty() {
        reason_codes.push(ReasonCode::ProviderSelectionMissingAuth);
        ProviderSelectionResultStatus::MissingAuth
    } else if !deferred_providers.is_empty() || !documented_only_providers.is_empty() {
        ProviderSelectionResultStatus::Deferred
    } else {
        ProviderSelectionResultStatus::NoEligibleProvider
    };

    ProviderSelectionResult {
        market: policy.market,
        selected_provider,
        fallback_selected,
        missing_auth_providers,
        deferred_providers,
        documented_only_providers,
        status,
        reason_codes,
    }
}

fn is_professional(source_class: ProviderSourceClass) -> bool {
    matches!(source_class, ProviderSourceClass::ProfessionalMarketDataApi)
}

fn supports_market(
    entry_market: ProviderMarket,
    provider_kind: ProviderKind,
    requested_market: ProviderMarket,
) -> bool {
    entry_market == requested_market
        || (provider_kind == ProviderKind::KoreaInvestmentMarketData
            && matches!(
                requested_market,
                ProviderMarket::KoreanEquity | ProviderMarket::USEquity
            ))
}
