use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::{EvidenceSourceKind, ProviderKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProviderMarket {
    Crypto,
    KoreanEquity,
    USEquity,
    GlobalEquity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderSourceClass {
    OfficialExchangeApi,
    PublicGovernmentDataApi,
    BrokerMarketDataApi,
    ProfessionalMarketDataApi,
    ResearchSupplementalData,
    TestFixture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderSupportedOutput {
    DailyBars,
    IntradayBars,
    Quotes,
    IndexData,
    ReferenceData,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderImplementedStatus {
    Implemented,
    Foundation,
    Stub,
    Deferred,
    DocumentedOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCatalogEntry {
    pub provider_kind: ProviderKind,
    pub provider_name: String,
    pub market: ProviderMarket,
    pub source_class: ProviderSourceClass,
    pub evidence_source_kind: EvidenceSourceKind,
    pub auth_requirement: String,
    pub supported_timeframes: Vec<String>,
    pub supported_outputs: Vec<ProviderSupportedOutput>,
    pub implemented_status: ProviderImplementedStatus,
    pub official_readiness_eligible: bool,
    pub benchmark_eligible: bool,
    pub notes: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketDataProviderCatalog {
    pub providers: Vec<ProviderCatalogEntry>,
    pub default_priority_by_market: BTreeMap<ProviderMarket, Vec<ProviderKind>>,
    pub reason_codes: Vec<ReasonCode>,
}

impl MarketDataProviderCatalog {
    pub fn default_catalog() -> Self {
        build_default_provider_catalog()
    }

    pub fn entries_for_market(&self, market: ProviderMarket) -> Vec<ProviderCatalogEntry> {
        self.providers
            .iter()
            .filter(|entry| entry.market == market)
            .cloned()
            .collect()
    }

    pub fn entry(&self, provider_kind: ProviderKind) -> Option<&ProviderCatalogEntry> {
        self.providers
            .iter()
            .find(|entry| entry.provider_kind == provider_kind)
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("provider_count={}", self.providers.len()),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|code| format!("{code:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ];
        for (market, providers) in &self.default_priority_by_market {
            lines.push(format!(
                "priority={market:?}:{}",
                providers
                    .iter()
                    .map(|provider| provider_slug(*provider))
                    .collect::<Vec<_>>()
                    .join("|")
            ));
        }
        for entry in &self.providers {
            lines.push(format!(
                "provider={};market={:?};source_class={:?};evidence={:?};status={:?};official={};benchmark={}",
                entry.provider_name,
                entry.market,
                entry.source_class,
                entry.evidence_source_kind,
                entry.implemented_status,
                entry.official_readiness_eligible,
                entry.benchmark_eligible,
            ));
        }
        lines.join("\n")
    }
}

pub fn build_default_provider_catalog() -> MarketDataProviderCatalog {
    let providers = vec![
        entry(
            ProviderKind::Upbit,
            "upbit",
            ProviderMarket::Crypto,
            ProviderSourceClass::OfficialExchangeApi,
            EvidenceSourceKind::OfficialApiCollected,
            "none",
            &["1m", "1d"],
            &[
                ProviderSupportedOutput::DailyBars,
                ProviderSupportedOutput::IntradayBars,
            ],
            ProviderImplementedStatus::Implemented,
            true,
            true,
            &["public crypto market-data path without auth"],
            &[ReasonCode::ProviderCatalogBuilt],
        ),
        entry(
            ProviderKind::Binance,
            "binance",
            ProviderMarket::Crypto,
            ProviderSourceClass::OfficialExchangeApi,
            EvidenceSourceKind::OfficialApiCollected,
            "none",
            &["1m", "1d"],
            &[
                ProviderSupportedOutput::DailyBars,
                ProviderSupportedOutput::IntradayBars,
            ],
            ProviderImplementedStatus::Deferred,
            true,
            false,
            &["deferred optional crypto fallback"],
            &[ReasonCode::ProviderCatalogBuilt],
        ),
        entry(
            ProviderKind::Korbit,
            "korbit",
            ProviderMarket::Crypto,
            ProviderSourceClass::OfficialExchangeApi,
            EvidenceSourceKind::OfficialApiCollected,
            "none",
            &["1d"],
            &[ProviderSupportedOutput::DailyBars],
            ProviderImplementedStatus::Deferred,
            true,
            false,
            &["optional/deferred crypto provider card"],
            &[ReasonCode::ProviderCatalogBuilt],
        ),
        entry(
            ProviderKind::KrxOpenApi,
            "krx-open-api",
            ProviderMarket::KoreanEquity,
            ProviderSourceClass::OfficialExchangeApi,
            EvidenceSourceKind::OfficialApiCollected,
            "api-key+endpoint-template",
            &["1d"],
            &[
                ProviderSupportedOutput::DailyBars,
                ProviderSupportedOutput::Quotes,
                ProviderSupportedOutput::ReferenceData,
            ],
            ProviderImplementedStatus::Implemented,
            true,
            true,
            &["Korean equity reference and fallback provider retained alongside KIS primary"],
            &[
                ReasonCode::ProviderCatalogBuilt,
                ReasonCode::KRXRetainedAsReference,
            ],
        ),
        entry(
            ProviderKind::DataGoKrFscStockPrice,
            "data-go-kr-fsc-stock-price",
            ProviderMarket::KoreanEquity,
            ProviderSourceClass::PublicGovernmentDataApi,
            EvidenceSourceKind::OfficialApiCollected,
            "service-key",
            &["1d"],
            &[
                ProviderSupportedOutput::DailyBars,
                ProviderSupportedOutput::ReferenceData,
            ],
            ProviderImplementedStatus::Stub,
            true,
            true,
            &["government data fallback with fixture parser v0"],
            &[ReasonCode::ProviderCatalogBuilt],
        ),
        entry(
            ProviderKind::KoreaInvestmentMarketData,
            "kis-market-data-only",
            ProviderMarket::KoreanEquity,
            ProviderSourceClass::BrokerMarketDataApi,
            EvidenceSourceKind::OfficialApiCollected,
            "app-key+app-secret",
            &["1d"],
            &[
                ProviderSupportedOutput::DailyBars,
                ProviderSupportedOutput::Quotes,
            ],
            ProviderImplementedStatus::Implemented,
            true,
            true,
            &[
                "market-data-only primary operational path for Korean equity and eligible US equity collection",
                "order/account/balance/position endpoints remain forbidden",
            ],
            &[
                ReasonCode::ProviderCatalogBuilt,
                ReasonCode::ProviderPriorityUpdated,
                ReasonCode::KRXRetainedAsReference,
            ],
        ),
        entry(
            ProviderKind::KoscomProfessional,
            "koscom-professional",
            ProviderMarket::KoreanEquity,
            ProviderSourceClass::ProfessionalMarketDataApi,
            EvidenceSourceKind::OfficialApiCollected,
            "api-key",
            &["1d"],
            &[
                ProviderSupportedOutput::DailyBars,
                ProviderSupportedOutput::Quotes,
                ProviderSupportedOutput::ReferenceData,
            ],
            ProviderImplementedStatus::DocumentedOnly,
            true,
            false,
            &["optional professional Korean equity provider card only"],
            &[
                ReasonCode::ProviderCatalogBuilt,
                ReasonCode::ProfessionalProviderCardOnly,
            ],
        ),
        entry(
            ProviderKind::AlphaVantage,
            "alphavantage",
            ProviderMarket::USEquity,
            ProviderSourceClass::OfficialExchangeApi,
            EvidenceSourceKind::OfficialApiCollected,
            "api-key",
            &["1m", "1d"],
            &[
                ProviderSupportedOutput::DailyBars,
                ProviderSupportedOutput::IntradayBars,
            ],
            ProviderImplementedStatus::Implemented,
            true,
            true,
            &["US equity fallback when KIS credentials are unavailable"],
            &[ReasonCode::ProviderCatalogBuilt],
        ),
        entry(
            ProviderKind::Alpaca,
            "alpaca-market-data",
            ProviderMarket::USEquity,
            ProviderSourceClass::BrokerMarketDataApi,
            EvidenceSourceKind::OfficialApiCollected,
            "api-key-id+api-secret-key",
            &["1d"],
            &[
                ProviderSupportedOutput::DailyBars,
                ProviderSupportedOutput::IntradayBars,
            ],
            ProviderImplementedStatus::Stub,
            true,
            true,
            &["historical bars fixture parser v0; no trading/account endpoints"],
            &[ReasonCode::ProviderCatalogBuilt],
        ),
        entry(
            ProviderKind::PolygonProfessional,
            "polygon-professional",
            ProviderMarket::USEquity,
            ProviderSourceClass::ProfessionalMarketDataApi,
            EvidenceSourceKind::OfficialApiCollected,
            "api-key",
            &["1m", "1d"],
            &[
                ProviderSupportedOutput::DailyBars,
                ProviderSupportedOutput::IntradayBars,
            ],
            ProviderImplementedStatus::DocumentedOnly,
            true,
            false,
            &["professional paid provider card only"],
            &[
                ReasonCode::ProviderCatalogBuilt,
                ReasonCode::ProfessionalProviderCardOnly,
            ],
        ),
        entry(
            ProviderKind::NasdaqDataLink,
            "nasdaq-data-link",
            ProviderMarket::USEquity,
            ProviderSourceClass::ProfessionalMarketDataApi,
            EvidenceSourceKind::OfficialApiCollected,
            "api-key",
            &["1d"],
            &[
                ProviderSupportedOutput::DailyBars,
                ProviderSupportedOutput::ReferenceData,
            ],
            ProviderImplementedStatus::DocumentedOnly,
            true,
            false,
            &["professional paid provider card only"],
            &[
                ReasonCode::ProviderCatalogBuilt,
                ReasonCode::ProfessionalProviderCardOnly,
            ],
        ),
        entry(
            ProviderKind::MockFixture,
            "mock-fixture",
            ProviderMarket::GlobalEquity,
            ProviderSourceClass::TestFixture,
            EvidenceSourceKind::TestFixture,
            "none",
            &["1m", "1d"],
            &[
                ProviderSupportedOutput::DailyBars,
                ProviderSupportedOutput::IntradayBars,
            ],
            ProviderImplementedStatus::Implemented,
            false,
            false,
            &["test-only deterministic fixture path"],
            &[ReasonCode::ProviderCatalogBuilt],
        ),
    ];
    let mut default_priority_by_market = BTreeMap::new();
    default_priority_by_market.insert(
        ProviderMarket::Crypto,
        vec![
            ProviderKind::Upbit,
            ProviderKind::Binance,
            ProviderKind::Korbit,
        ],
    );
    default_priority_by_market.insert(
        ProviderMarket::KoreanEquity,
        vec![
            ProviderKind::KoreaInvestmentMarketData,
            ProviderKind::KrxOpenApi,
            ProviderKind::DataGoKrFscStockPrice,
            ProviderKind::KoscomProfessional,
        ],
    );
    default_priority_by_market.insert(
        ProviderMarket::USEquity,
        vec![
            ProviderKind::KoreaInvestmentMarketData,
            ProviderKind::AlphaVantage,
            ProviderKind::Alpaca,
            ProviderKind::PolygonProfessional,
            ProviderKind::NasdaqDataLink,
        ],
    );
    default_priority_by_market.insert(
        ProviderMarket::GlobalEquity,
        vec![ProviderKind::MockFixture],
    );

    MarketDataProviderCatalog {
        providers,
        default_priority_by_market,
        reason_codes: vec![
            ReasonCode::ProviderCatalogBuilt,
            ReasonCode::DeterministicPath,
            ReasonCode::ProviderPriorityUpdated,
            ReasonCode::KRXRetainedAsReference,
        ],
    }
}

fn entry(
    provider_kind: ProviderKind,
    provider_name: &str,
    market: ProviderMarket,
    source_class: ProviderSourceClass,
    evidence_source_kind: EvidenceSourceKind,
    auth_requirement: &str,
    supported_timeframes: &[&str],
    supported_outputs: &[ProviderSupportedOutput],
    implemented_status: ProviderImplementedStatus,
    official_readiness_eligible: bool,
    benchmark_eligible: bool,
    notes: &[&str],
    reason_codes: &[ReasonCode],
) -> ProviderCatalogEntry {
    ProviderCatalogEntry {
        provider_kind,
        provider_name: provider_name.to_string(),
        market,
        source_class,
        evidence_source_kind,
        auth_requirement: auth_requirement.to_string(),
        supported_timeframes: supported_timeframes
            .iter()
            .map(|value| value.to_string())
            .collect(),
        supported_outputs: supported_outputs.to_vec(),
        implemented_status,
        official_readiness_eligible,
        benchmark_eligible,
        notes: notes.iter().map(|value| value.to_string()).collect(),
        reason_codes: reason_codes.to_vec(),
    }
}

fn provider_slug(provider_kind: ProviderKind) -> &'static str {
    match provider_kind {
        ProviderKind::Upbit => "upbit",
        ProviderKind::Binance => "binance",
        ProviderKind::Korbit => "korbit",
        ProviderKind::KrxOpenApi => "krx",
        ProviderKind::DataGoKrFscStockPrice => "data-go-kr-fsc-stock-price",
        ProviderKind::AlphaVantage => "alphavantage",
        ProviderKind::Alpaca => "alpaca",
        ProviderKind::KoreaInvestmentMarketData => "kis-market-data-only",
        ProviderKind::PolygonProfessional => "polygon",
        ProviderKind::NasdaqDataLink => "nasdaq-data-link",
        ProviderKind::KoscomProfessional => "koscom",
        ProviderKind::MockFixture => "mock-fixture",
        ProviderKind::Unknown => "unknown",
    }
}
