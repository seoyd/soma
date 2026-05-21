use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{
    DataFreshnessTier, ProviderDataSubject, ProviderEntitlementStatus,
    ProviderEntitlementStatusKind, provider_freshness_profile,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StrategyUseCase {
    EodSwing,
    DailyPortfolioResearch,
    IntradaySwing,
    RealtimeScalping,
    RealtimeExecutionSimulation,
    SourceComparison,
    ModelPrototypeResearch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyDataRequirement {
    pub use_case: StrategyUseCase,
    pub min_freshness_tier: DataFreshnessTier,
    pub require_official_source: bool,
    pub require_full_market_coverage: bool,
    pub allow_research_supplemental: bool,
    pub allow_delayed: bool,
    pub allow_eod: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyDataCompatibilityResult {
    pub use_case: StrategyUseCase,
    pub provider_subject: ProviderDataSubject,
    pub compatible: bool,
    pub limitations: Vec<String>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn default_strategy_requirement(use_case: StrategyUseCase) -> StrategyDataRequirement {
    match use_case {
        StrategyUseCase::EodSwing | StrategyUseCase::DailyPortfolioResearch => {
            StrategyDataRequirement {
                use_case,
                min_freshness_tier: DataFreshnessTier::Eod,
                require_official_source: false,
                require_full_market_coverage: false,
                allow_research_supplemental: false,
                allow_delayed: false,
                allow_eod: true,
                reason_codes: vec![ReasonCode::StrategyCompatibilityBuilt],
            }
        }
        StrategyUseCase::IntradaySwing => StrategyDataRequirement {
            use_case,
            min_freshness_tier: DataFreshnessTier::Delayed15m,
            require_official_source: false,
            require_full_market_coverage: false,
            allow_research_supplemental: false,
            allow_delayed: true,
            allow_eod: false,
            reason_codes: vec![ReasonCode::StrategyCompatibilityBuilt],
        },
        StrategyUseCase::RealtimeScalping | StrategyUseCase::RealtimeExecutionSimulation => {
            StrategyDataRequirement {
                use_case,
                min_freshness_tier: DataFreshnessTier::RealtimeIex,
                require_official_source: true,
                require_full_market_coverage: matches!(
                    use_case,
                    StrategyUseCase::RealtimeExecutionSimulation
                ),
                allow_research_supplemental: false,
                allow_delayed: false,
                allow_eod: false,
                reason_codes: vec![ReasonCode::StrategyCompatibilityBuilt],
            }
        }
        StrategyUseCase::SourceComparison | StrategyUseCase::ModelPrototypeResearch => {
            StrategyDataRequirement {
                use_case,
                min_freshness_tier: DataFreshnessTier::ResearchOnly,
                require_official_source: false,
                require_full_market_coverage: false,
                allow_research_supplemental: true,
                allow_delayed: true,
                allow_eod: true,
                reason_codes: vec![ReasonCode::StrategyCompatibilityBuilt],
            }
        }
    }
}

pub fn evaluate_strategy_data_compatibility(
    provider_subject: ProviderDataSubject,
    use_case: StrategyUseCase,
    entitlement: Option<&ProviderEntitlementStatus>,
) -> StrategyDataCompatibilityResult {
    let freshness = provider_freshness_profile(provider_subject);
    let mut limitations = Vec::new();
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let mut reason_codes = vec![ReasonCode::StrategyCompatibilityBuilt];

    if matches!(provider_subject, ProviderDataSubject::YFinanceResearch)
        && !matches!(
            use_case,
            StrategyUseCase::SourceComparison | StrategyUseCase::ModelPrototypeResearch
        )
    {
        blockers.push(
            "research-only data cannot satisfy official or execution-oriented validation"
                .to_string(),
        );
        reason_codes.push(ReasonCode::YFinanceResearchOnly);
    }

    match use_case {
        StrategyUseCase::EodSwing | StrategyUseCase::DailyPortfolioResearch => {
            if !freshness
                .available_freshness_tiers
                .iter()
                .any(|tier| matches!(tier, DataFreshnessTier::Eod | DataFreshnessTier::Historical))
            {
                blockers.push(
                    "provider does not expose EOD/historical data for this use-case".to_string(),
                );
            }
        }
        StrategyUseCase::IntradaySwing => {
            if freshness
                .available_freshness_tiers
                .iter()
                .all(|tier| matches!(tier, DataFreshnessTier::Eod | DataFreshnessTier::Historical))
            {
                blockers.push("EOD-only data cannot validate intraday swing behavior".to_string());
            }
        }
        StrategyUseCase::RealtimeScalping => {
            if freshness
                .available_freshness_tiers
                .iter()
                .all(|tier| !is_realtime(*tier))
            {
                blockers
                    .push("EOD or delayed-only data cannot validate realtime scalping".to_string());
            }
            if entitlement.is_some_and(|status| {
                status.status == ProviderEntitlementStatusKind::ReadyForRealtimeResearchIexOnly
            }) {
                limitations.push(
                    "IEX-only realtime is limited and does not imply full US market coverage"
                        .to_string(),
                );
                reason_codes.push(ReasonCode::AlpacaIexLimited);
            }
        }
        StrategyUseCase::RealtimeExecutionSimulation => {
            if freshness
                .available_freshness_tiers
                .iter()
                .any(|tier| matches!(tier, DataFreshnessTier::Delayed15m))
                && freshness
                    .available_freshness_tiers
                    .iter()
                    .all(|tier| !is_realtime(*tier))
            {
                blockers.push(
                    "Delayed15m data is incompatible with realtime execution simulation"
                        .to_string(),
                );
            } else if freshness
                .available_freshness_tiers
                .iter()
                .all(|tier| !is_realtime(*tier))
            {
                blockers.push(
                    "EOD-only data cannot validate realtime execution simulation".to_string(),
                );
            }
            if entitlement.is_some_and(|status| {
                status.status == ProviderEntitlementStatusKind::ReadyForRealtimeResearchIexOnly
            }) {
                blockers.push(
                    "IEX-only realtime cannot claim full-market execution coverage".to_string(),
                );
                reason_codes.push(ReasonCode::FullMarketCoverageUnavailable);
            }
        }
        StrategyUseCase::SourceComparison | StrategyUseCase::ModelPrototypeResearch => {
            if matches!(provider_subject, ProviderDataSubject::YFinanceResearch) {
                warnings.push("research-only supplemental data is acceptable for comparison and prototyping only".to_string());
            }
        }
    }

    if matches!(
        provider_subject,
        ProviderDataSubject::Provider(crate::data::ProviderKind::KrxOpenApi)
    ) && matches!(
        use_case,
        StrategyUseCase::IntradaySwing
            | StrategyUseCase::RealtimeScalping
            | StrategyUseCase::RealtimeExecutionSimulation
    ) {
        blockers.push(
            "KRX default EOD profile does not validate Korean intraday or scalping strategies"
                .to_string(),
        );
    }

    if matches!(
        provider_subject,
        ProviderDataSubject::Provider(crate::data::ProviderKind::AlphaVantage)
    ) && matches!(
        use_case,
        StrategyUseCase::RealtimeScalping | StrategyUseCase::RealtimeExecutionSimulation
    ) {
        blockers.push("AlphaVantage compact/free is not realtime".to_string());
        reason_codes.push(ReasonCode::AlphaVantageEodOnly);
    }

    StrategyDataCompatibilityResult {
        use_case,
        provider_subject,
        compatible: blockers.is_empty(),
        limitations,
        blockers,
        warnings,
        reason_codes,
    }
}

fn is_realtime(tier: DataFreshnessTier) -> bool {
    matches!(
        tier,
        DataFreshnessTier::RealtimeIex
            | DataFreshnessTier::RealtimeSip
            | DataFreshnessTier::RealtimeExchangeOfficial
            | DataFreshnessTier::RealtimeCryptoPublic
    )
}
