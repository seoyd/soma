use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::ProviderKind;
use super::cost_profile::{ProviderCostTier, provider_cost_profile};
use super::credential_profiles::{
    ProviderCredentialProfile, ProviderCredentialStatusKind, default_provider_credential_profiles,
    evaluate_provider_credential_profiles,
};
use super::freshness::{DataFreshnessTier, ProviderDataSubject, provider_freshness_profile};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderEntitlementUseCase {
    EodResearch,
    SwingResearch,
    IntradayResearch,
    RealtimeResearch,
    FullMarketCoverageResearch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderEntitlementStatusKind {
    ReadyForEodResearch,
    ReadyForIntradayResearch,
    ReadyForRealtimeResearch,
    ReadyForRealtimeResearchIexOnly,
    ReadyForCryptoResearch,
    MissingAuth,
    MissingApproval,
    MissingEndpointTemplate,
    MissingPremiumEntitlement,
    ResearchOnlyFallback,
    NotSuitableForUseCase,
    Deferred,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderEntitlementPreflightConfig {
    pub check_id: String,
    pub providers_to_check: Vec<ProviderDataSubject>,
    pub required_use_case: ProviderEntitlementUseCase,
    #[serde(default)]
    pub credential_profile_overrides: Vec<ProviderCredentialProfile>,
    #[serde(default)]
    pub approval_ready_providers: Vec<ProviderDataSubject>,
    #[serde(default)]
    pub realtime_entitled_providers: Vec<ProviderDataSubject>,
    #[serde(default)]
    pub delayed_entitled_providers: Vec<ProviderDataSubject>,
    #[serde(default)]
    pub full_market_coverage_providers: Vec<ProviderDataSubject>,
    #[serde(default = "default_true")]
    pub allow_missing_premium: bool,
    #[serde(default = "default_true")]
    pub allow_research_fallback: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderEntitlementStatus {
    pub provider_subject: ProviderDataSubject,
    pub freshness_available: Vec<DataFreshnessTier>,
    pub cost_tier: ProviderCostTier,
    pub auth_ready: bool,
    pub approval_ready: bool,
    pub endpoint_template_ready: bool,
    pub realtime_entitlement_ready: bool,
    pub delayed_entitlement_ready: bool,
    pub official_readiness_eligible: bool,
    pub research_only: bool,
    pub status: ProviderEntitlementStatusKind,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProviderEntitlementPreflightRunner;

impl Default for ProviderEntitlementPreflightConfig {
    fn default() -> Self {
        Self {
            check_id: "provider_entitlement_preflight".to_string(),
            providers_to_check: vec![
                ProviderDataSubject::Provider(ProviderKind::KrxOpenApi),
                ProviderDataSubject::Provider(ProviderKind::AlphaVantage),
                ProviderDataSubject::Provider(ProviderKind::Alpaca),
                ProviderDataSubject::Provider(ProviderKind::Upbit),
                ProviderDataSubject::YFinanceResearch,
            ],
            required_use_case: ProviderEntitlementUseCase::EodResearch,
            credential_profile_overrides: Vec::new(),
            approval_ready_providers: Vec::new(),
            realtime_entitled_providers: Vec::new(),
            delayed_entitled_providers: Vec::new(),
            full_market_coverage_providers: Vec::new(),
            allow_missing_premium: true,
            allow_research_fallback: true,
            reason_codes: vec![ReasonCode::ProviderEntitlementPreflightBuilt],
        }
    }
}

impl ProviderEntitlementPreflightConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl ProviderEntitlementPreflightRunner {
    pub fn run(
        &self,
        config: &ProviderEntitlementPreflightConfig,
    ) -> Vec<ProviderEntitlementStatus> {
        let credential_statuses = evaluate_provider_credential_profiles(
            &apply_credential_overrides(&config.credential_profile_overrides),
        );
        let mut statuses = config
            .providers_to_check
            .iter()
            .copied()
            .map(|subject| evaluate_subject(config, &credential_statuses, subject))
            .collect::<Vec<_>>();
        statuses.sort_by_key(|status| subject_rank(status.provider_subject));
        statuses
    }
}

fn apply_credential_overrides(
    overrides: &[ProviderCredentialProfile],
) -> Vec<ProviderCredentialProfile> {
    let mut profiles = default_provider_credential_profiles();
    for override_profile in overrides {
        if let Some(existing) = profiles
            .iter_mut()
            .find(|profile| profile.provider_kind == override_profile.provider_kind)
        {
            *existing = override_profile.clone();
        } else {
            profiles.push(override_profile.clone());
        }
    }
    profiles
}

fn evaluate_subject(
    config: &ProviderEntitlementPreflightConfig,
    credential_statuses: &[super::credential_profiles::ProviderCredentialStatus],
    subject: ProviderDataSubject,
) -> ProviderEntitlementStatus {
    let freshness = provider_freshness_profile(subject);
    let cost = provider_cost_profile(subject);
    let credential_status = subject.provider_kind().and_then(|kind| {
        credential_statuses
            .iter()
            .find(|status| status.provider_kind == kind)
    });
    let auth_ready = matches!(
        credential_status.map(|status| status.status),
        Some(ProviderCredentialStatusKind::Ready | ProviderCredentialStatusKind::NotRequired)
    ) || matches!(subject, ProviderDataSubject::Provider(ProviderKind::Upbit));
    let endpoint_template_ready = !matches!(
        credential_status.map(|status| status.status),
        Some(ProviderCredentialStatusKind::MissingEndpointTemplate)
    );
    let approval_ready = config.approval_ready_providers.contains(&subject)
        || !matches!(
            subject,
            ProviderDataSubject::Provider(ProviderKind::KrxOpenApi)
        );
    let realtime_entitlement_ready = config.realtime_entitled_providers.contains(&subject)
        || matches!(
            subject,
            ProviderDataSubject::Provider(ProviderKind::Upbit)
                | ProviderDataSubject::Provider(ProviderKind::KoreaInvestmentMarketData)
        );
    let delayed_entitlement_ready =
        config.delayed_entitled_providers.contains(&subject) || realtime_entitlement_ready;
    let full_market_coverage_ready = config.full_market_coverage_providers.contains(&subject)
        || matches!(
            subject,
            ProviderDataSubject::Provider(ProviderKind::PolygonProfessional)
                | ProviderDataSubject::Provider(ProviderKind::NasdaqDataLink)
        );
    let official_readiness_eligible = !matches!(
        subject,
        ProviderDataSubject::YFinanceResearch
            | ProviderDataSubject::Provider(ProviderKind::MockFixture)
    );
    let research_only = matches!(subject, ProviderDataSubject::YFinanceResearch);

    let mut reason_codes = vec![ReasonCode::ProviderEntitlementPreflightBuilt];
    let status = if matches!(subject, ProviderDataSubject::YFinanceResearch) {
        reason_codes.push(ReasonCode::YFinanceResearchOnly);
        ProviderEntitlementStatusKind::ResearchOnlyFallback
    } else if matches!(
        subject,
        ProviderDataSubject::Provider(ProviderKind::MockFixture)
    ) {
        ProviderEntitlementStatusKind::Deferred
    } else if !endpoint_template_ready {
        reason_codes.push(ReasonCode::MissingEndpointTemplate);
        ProviderEntitlementStatusKind::MissingEndpointTemplate
    } else if !auth_ready
        && !matches!(
            subject,
            ProviderDataSubject::Provider(ProviderKind::Upbit)
                | ProviderDataSubject::Provider(ProviderKind::Binance)
                | ProviderDataSubject::Provider(ProviderKind::Korbit)
        )
    {
        reason_codes.push(ReasonCode::MissingAuth);
        ProviderEntitlementStatusKind::MissingAuth
    } else {
        match config.required_use_case {
            ProviderEntitlementUseCase::EodResearch | ProviderEntitlementUseCase::SwingResearch => {
                if matches!(
                    subject,
                    ProviderDataSubject::Provider(ProviderKind::KrxOpenApi)
                ) && !approval_ready
                {
                    reason_codes.push(ReasonCode::KrxApprovalPending);
                    ProviderEntitlementStatusKind::MissingApproval
                } else if freshness.available_freshness_tiers.iter().any(|tier| {
                    matches!(tier, DataFreshnessTier::Eod | DataFreshnessTier::Historical)
                }) {
                    if matches!(
                        subject,
                        ProviderDataSubject::Provider(ProviderKind::AlphaVantage)
                    ) {
                        reason_codes.push(ReasonCode::AlphaVantageEodOnly);
                    }
                    ProviderEntitlementStatusKind::ReadyForEodResearch
                } else {
                    ProviderEntitlementStatusKind::NotSuitableForUseCase
                }
            }
            ProviderEntitlementUseCase::IntradayResearch => {
                if matches!(
                    subject,
                    ProviderDataSubject::Provider(ProviderKind::KrxOpenApi)
                ) && !approval_ready
                {
                    reason_codes.push(ReasonCode::KrxApprovalPending);
                    ProviderEntitlementStatusKind::MissingApproval
                } else if freshness.available_freshness_tiers.iter().any(|tier| {
                    matches!(
                        tier,
                        DataFreshnessTier::RealtimeIex
                            | DataFreshnessTier::RealtimeSip
                            | DataFreshnessTier::RealtimeExchangeOfficial
                            | DataFreshnessTier::RealtimeCryptoPublic
                    )
                }) {
                    if matches!(subject, ProviderDataSubject::Provider(ProviderKind::Alpaca))
                        && !full_market_coverage_ready
                    {
                        reason_codes.push(ReasonCode::AlpacaIexLimited);
                    }
                    ProviderEntitlementStatusKind::ReadyForIntradayResearch
                } else {
                    ProviderEntitlementStatusKind::NotSuitableForUseCase
                }
            }
            ProviderEntitlementUseCase::RealtimeResearch => {
                if matches!(
                    subject,
                    ProviderDataSubject::Provider(ProviderKind::AlphaVantage)
                ) && !realtime_entitlement_ready
                {
                    reason_codes.push(ReasonCode::AlphaVantagePremiumRequired);
                    ProviderEntitlementStatusKind::MissingPremiumEntitlement
                } else if matches!(subject, ProviderDataSubject::Provider(ProviderKind::Alpaca))
                    && !full_market_coverage_ready
                {
                    reason_codes.push(ReasonCode::AlpacaIexLimited);
                    ProviderEntitlementStatusKind::ReadyForRealtimeResearchIexOnly
                } else if realtime_entitlement_ready
                    || matches!(subject, ProviderDataSubject::Provider(ProviderKind::Upbit))
                {
                    ProviderEntitlementStatusKind::ReadyForRealtimeResearch
                } else if config.allow_missing_premium {
                    reason_codes.push(ReasonCode::MissingPremiumEntitlement);
                    ProviderEntitlementStatusKind::MissingPremiumEntitlement
                } else {
                    ProviderEntitlementStatusKind::NotSuitableForUseCase
                }
            }
            ProviderEntitlementUseCase::FullMarketCoverageResearch => {
                if matches!(
                    subject,
                    ProviderDataSubject::Provider(ProviderKind::AlphaVantage)
                ) {
                    reason_codes.push(ReasonCode::FullMarketCoverageUnavailable);
                    ProviderEntitlementStatusKind::MissingPremiumEntitlement
                } else if matches!(subject, ProviderDataSubject::Provider(ProviderKind::Alpaca))
                    && !full_market_coverage_ready
                {
                    reason_codes.push(ReasonCode::FullMarketCoverageUnavailable);
                    ProviderEntitlementStatusKind::MissingPremiumEntitlement
                } else if full_market_coverage_ready {
                    ProviderEntitlementStatusKind::ReadyForRealtimeResearch
                } else {
                    ProviderEntitlementStatusKind::NotSuitableForUseCase
                }
            }
        }
    };

    ProviderEntitlementStatus {
        provider_subject: subject,
        freshness_available: freshness.available_freshness_tiers,
        cost_tier: cost.cost_tier,
        auth_ready,
        approval_ready,
        endpoint_template_ready,
        realtime_entitlement_ready,
        delayed_entitlement_ready,
        official_readiness_eligible,
        research_only,
        status,
        reason_codes,
    }
}

fn default_true() -> bool {
    true
}

fn subject_rank(subject: ProviderDataSubject) -> usize {
    match subject {
        ProviderDataSubject::Provider(kind) => kind as usize,
        ProviderDataSubject::YFinanceResearch => usize::MAX,
    }
}

impl ProviderDataSubject {
    pub fn provider_kind(self) -> Option<ProviderKind> {
        match self {
            ProviderDataSubject::Provider(kind) => Some(kind),
            ProviderDataSubject::YFinanceResearch => None,
        }
    }
}
