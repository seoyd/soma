use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::ProviderKind;
use super::freshness::ProviderDataSubject;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProviderCostTier {
    Free,
    FreeWithLimits,
    Paid,
    PaidProfessional,
    RequiresApproval,
    RequiresSubscription,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCostProfile {
    pub provider_subject: ProviderDataSubject,
    pub cost_tier: ProviderCostTier,
    pub free_limits_summary: Option<String>,
    pub subscription_required_for: Vec<String>,
    pub approval_required: bool,
    pub commercial_use_warning: bool,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn default_provider_cost_profiles() -> Vec<ProviderCostProfile> {
    let mut profiles = vec![
        profile(
            ProviderDataSubject::Provider(ProviderKind::Upbit),
            ProviderCostTier::Free,
            Some("public market-data endpoints; bounded requests still required"),
            &[],
            false,
            false,
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::KrxOpenApi),
            ProviderCostTier::RequiresApproval,
            None,
            &["operator approval and endpoint access".to_string()],
            true,
            true,
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::DataGoKrFscStockPrice),
            ProviderCostTier::FreeWithLimits,
            Some("public service key and government data limits apply"),
            &[],
            false,
            false,
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::KoreaInvestmentMarketData),
            ProviderCostTier::RequiresSubscription,
            None,
            &["developer access or broker-linked market-data access".to_string()],
            false,
            true,
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::AlphaVantage),
            ProviderCostTier::FreeWithLimits,
            Some("compact/free plan is bounded and EOD-oriented"),
            &["premium delayed/realtime entitlement".to_string()],
            false,
            false,
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::Alpaca),
            ProviderCostTier::FreeWithLimits,
            Some("basic plan is IEX-limited"),
            &["paid SIP/all-exchange coverage".to_string()],
            false,
            false,
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::PolygonProfessional),
            ProviderCostTier::PaidProfessional,
            None,
            &["professional realtime aggregates and broader coverage".to_string()],
            false,
            true,
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::NasdaqDataLink),
            ProviderCostTier::PaidProfessional,
            None,
            &["professional datasets vary by subscription".to_string()],
            false,
            true,
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::KoscomProfessional),
            ProviderCostTier::PaidProfessional,
            None,
            &["commercial provider onboarding".to_string()],
            false,
            true,
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::MockFixture),
            ProviderCostTier::Unknown,
            None,
            &[],
            false,
            false,
        ),
        profile(
            ProviderDataSubject::YFinanceResearch,
            ProviderCostTier::FreeWithLimits,
            Some("unofficial supplemental research path"),
            &[],
            false,
            false,
        ),
    ];
    profiles.sort_by_key(|profile| match profile.provider_subject {
        ProviderDataSubject::Provider(kind) => kind as usize,
        ProviderDataSubject::YFinanceResearch => usize::MAX,
    });
    profiles
}

pub fn provider_cost_profile(subject: ProviderDataSubject) -> ProviderCostProfile {
    default_provider_cost_profiles()
        .into_iter()
        .find(|profile| profile.provider_subject == subject)
        .unwrap_or_else(|| profile(subject, ProviderCostTier::Unknown, None, &[], false, false))
}

fn profile(
    provider_subject: ProviderDataSubject,
    cost_tier: ProviderCostTier,
    free_limits_summary: Option<&str>,
    subscription_required_for: &[String],
    approval_required: bool,
    commercial_use_warning: bool,
) -> ProviderCostProfile {
    ProviderCostProfile {
        provider_subject,
        cost_tier,
        free_limits_summary: free_limits_summary.map(|value| value.to_string()),
        subscription_required_for: subscription_required_for.to_vec(),
        approval_required,
        commercial_use_warning,
        reason_codes: vec![ReasonCode::ProviderCostProfileBuilt],
    }
}
