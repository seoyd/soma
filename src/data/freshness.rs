use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::ProviderKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProviderDataSubject {
    Provider(ProviderKind),
    YFinanceResearch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DataFreshnessTier {
    Eod,
    Historical,
    Delayed15m,
    RealtimeIex,
    RealtimeSip,
    RealtimeExchangeOfficial,
    RealtimeCryptoPublic,
    ResearchOnly,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderFreshnessProfile {
    pub provider_subject: ProviderDataSubject,
    pub default_freshness: DataFreshnessTier,
    pub available_freshness_tiers: Vec<DataFreshnessTier>,
    pub requires_entitlement_for_realtime: bool,
    pub requires_entitlement_for_delayed: bool,
    pub notes: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn default_provider_freshness_profiles() -> Vec<ProviderFreshnessProfile> {
    let mut profiles = vec![
        profile(
            ProviderDataSubject::Provider(ProviderKind::Upbit),
            DataFreshnessTier::RealtimeCryptoPublic,
            &[
                DataFreshnessTier::RealtimeCryptoPublic,
                DataFreshnessTier::Historical,
            ],
            false,
            false,
            &["public crypto candles suitable for bounded realtime/intraday research"],
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::KrxOpenApi),
            DataFreshnessTier::Eod,
            &[DataFreshnessTier::Eod, DataFreshnessTier::Historical],
            true,
            false,
            &["official Korean equity path stays approval-gated and EOD/historical by default"],
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::DataGoKrFscStockPrice),
            DataFreshnessTier::Eod,
            &[DataFreshnessTier::Eod, DataFreshnessTier::Historical],
            false,
            false,
            &["government fallback is EOD/historical only"],
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::KoreaInvestmentMarketData),
            DataFreshnessTier::RealtimeExchangeOfficial,
            &[
                DataFreshnessTier::Historical,
                DataFreshnessTier::RealtimeExchangeOfficial,
            ],
            true,
            false,
            &["credentialed market-data-only path; intraday/realtime capability remains bounded"],
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::AlphaVantage),
            DataFreshnessTier::Eod,
            &[
                DataFreshnessTier::Eod,
                DataFreshnessTier::Historical,
                DataFreshnessTier::Delayed15m,
            ],
            true,
            true,
            &[
                "compact/free default stays EOD/historical; delayed/realtime require premium entitlement",
            ],
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::Alpaca),
            DataFreshnessTier::RealtimeIex,
            &[
                DataFreshnessTier::Historical,
                DataFreshnessTier::RealtimeIex,
                DataFreshnessTier::RealtimeSip,
            ],
            true,
            false,
            &["basic plan is IEX-limited; paid plan is needed for SIP/fuller coverage"],
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::PolygonProfessional),
            DataFreshnessTier::RealtimeSip,
            &[
                DataFreshnessTier::Historical,
                DataFreshnessTier::RealtimeSip,
            ],
            true,
            false,
            &["professional paid candidate for broader realtime coverage"],
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::NasdaqDataLink),
            DataFreshnessTier::Historical,
            &[DataFreshnessTier::Historical],
            false,
            false,
            &["dataset-dependent professional historical source"],
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::Binance),
            DataFreshnessTier::Unknown,
            &[DataFreshnessTier::Unknown],
            false,
            false,
            &["deferred optional crypto fallback"],
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::Korbit),
            DataFreshnessTier::Unknown,
            &[DataFreshnessTier::Unknown],
            false,
            false,
            &["optional/deferred crypto fallback"],
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::KoscomProfessional),
            DataFreshnessTier::RealtimeExchangeOfficial,
            &[
                DataFreshnessTier::Historical,
                DataFreshnessTier::RealtimeExchangeOfficial,
            ],
            true,
            false,
            &["professional Korean market-data candidate"],
        ),
        profile(
            ProviderDataSubject::Provider(ProviderKind::MockFixture),
            DataFreshnessTier::Unknown,
            &[DataFreshnessTier::Unknown],
            false,
            false,
            &["test-only fixture path"],
        ),
        profile(
            ProviderDataSubject::YFinanceResearch,
            DataFreshnessTier::ResearchOnly,
            &[DataFreshnessTier::ResearchOnly],
            false,
            false,
            &["research-only supplemental path; never official readiness"],
        ),
    ];
    profiles.sort_by_key(|profile| profile_rank(profile.provider_subject));
    profiles
}

pub fn provider_freshness_profile(subject: ProviderDataSubject) -> ProviderFreshnessProfile {
    default_provider_freshness_profiles()
        .into_iter()
        .find(|profile| profile.provider_subject == subject)
        .unwrap_or_else(|| {
            profile(
                subject,
                DataFreshnessTier::Unknown,
                &[DataFreshnessTier::Unknown],
                false,
                false,
                &["unknown provider freshness"],
            )
        })
}

fn profile(
    provider_subject: ProviderDataSubject,
    default_freshness: DataFreshnessTier,
    available_freshness_tiers: &[DataFreshnessTier],
    requires_entitlement_for_realtime: bool,
    requires_entitlement_for_delayed: bool,
    notes: &[&str],
) -> ProviderFreshnessProfile {
    ProviderFreshnessProfile {
        provider_subject,
        default_freshness,
        available_freshness_tiers: available_freshness_tiers.to_vec(),
        requires_entitlement_for_realtime,
        requires_entitlement_for_delayed,
        notes: notes.iter().map(|value| value.to_string()).collect(),
        reason_codes: vec![ReasonCode::ProviderFreshnessBuilt],
    }
}

fn profile_rank(subject: ProviderDataSubject) -> usize {
    match subject {
        ProviderDataSubject::Provider(ProviderKind::Upbit) => 0,
        ProviderDataSubject::Provider(ProviderKind::Binance) => 1,
        ProviderDataSubject::Provider(ProviderKind::Korbit) => 2,
        ProviderDataSubject::Provider(ProviderKind::KrxOpenApi) => 3,
        ProviderDataSubject::Provider(ProviderKind::DataGoKrFscStockPrice) => 4,
        ProviderDataSubject::Provider(ProviderKind::KoreaInvestmentMarketData) => 5,
        ProviderDataSubject::Provider(ProviderKind::AlphaVantage) => 6,
        ProviderDataSubject::Provider(ProviderKind::Alpaca) => 7,
        ProviderDataSubject::Provider(ProviderKind::PolygonProfessional) => 8,
        ProviderDataSubject::Provider(ProviderKind::NasdaqDataLink) => 9,
        ProviderDataSubject::Provider(ProviderKind::KoscomProfessional) => 10,
        ProviderDataSubject::Provider(ProviderKind::MockFixture) => 11,
        ProviderDataSubject::Provider(ProviderKind::Unknown) => 12,
        ProviderDataSubject::YFinanceResearch => 13,
    }
}
