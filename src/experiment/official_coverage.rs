use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{
    MarketVenue, OfficialCollectionEntryStatus, OfficialCollectionReport, ProviderKind,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialDatasetCoverageReport {
    pub total_ready_entries: usize,
    pub crypto_ready_entries: usize,
    pub korean_equity_ready_entries: usize,
    pub us_equity_ready_entries: usize,
    pub skipped_missing_auth_entries: usize,
    pub skipped_budget_entries: usize,
    pub failed_preflight_entries: usize,
    pub provider_statuses: BTreeMap<String, String>,
    pub missing_auth_providers: Vec<String>,
    pub compactness_summary: String,
    pub non_official_ready_entries: usize,
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialDatasetCoverageReport {
    pub fn from_collection_report(report: &OfficialCollectionReport) -> Self {
        let mut total_ready_entries = 0usize;
        let mut crypto_ready_entries = 0usize;
        let mut korean_equity_ready_entries = 0usize;
        let mut us_equity_ready_entries = 0usize;
        let mut skipped_missing_auth_entries = 0usize;
        let mut skipped_budget_entries = 0usize;
        let mut failed_preflight_entries = 0usize;
        let mut non_official_ready_entries = 0usize;
        let mut provider_counts = BTreeMap::<String, ProviderCoverageState>::new();
        let mut missing_auth_providers = Vec::new();
        let mut compact_paths = 0usize;
        let mut canonical_paths = 0usize;
        let mut reason_codes = vec![ReasonCode::OfficialDatasetCoverageBuilt];

        for entry in &report.entry_reports {
            let key = provider_name(entry.provider_kind).to_string();
            let state = provider_counts.entry(key.clone()).or_default();
            if entry
                .canonical_csv_path
                .as_deref()
                .is_some_and(|path| path.contains("_compact"))
            {
                compact_paths += 1;
            }
            if entry.canonical_csv_path.is_some() {
                canonical_paths += 1;
            }
            match entry.status {
                OfficialCollectionEntryStatus::SkippedMissingAuth => {
                    skipped_missing_auth_entries += 1;
                    state.skipped_missing_auth += 1;
                    if !missing_auth_providers.contains(&key) {
                        missing_auth_providers.push(key.clone());
                    }
                }
                OfficialCollectionEntryStatus::SkippedBudgetExceeded => {
                    skipped_budget_entries += 1;
                    state.skipped_budget += 1;
                }
                OfficialCollectionEntryStatus::FailedPreflight => {
                    failed_preflight_entries += 1;
                    state.failed_preflight += 1;
                }
                OfficialCollectionEntryStatus::FailedProvider => {
                    state.failed_provider += 1;
                }
                OfficialCollectionEntryStatus::DiagnosticOnly => {
                    if entry
                        .preflight_status
                        .as_deref()
                        .is_some_and(|value| value != "ReadyForRealEvidence")
                    {
                        failed_preflight_entries += 1;
                        state.failed_preflight += 1;
                    } else {
                        state.diagnostic_only += 1;
                    }
                }
                OfficialCollectionEntryStatus::Collected => {
                    state.collected += 1;
                }
            }

            if entry.ready_for_evidence {
                if entry.provider_kind == ProviderKind::MockFixture {
                    non_official_ready_entries += 1;
                } else {
                    total_ready_entries += 1;
                    if is_crypto(entry.provider_kind, entry.venue) {
                        crypto_ready_entries += 1;
                    }
                    if is_korean_equity(entry.provider_kind, entry.venue) {
                        korean_equity_ready_entries += 1;
                    }
                    if is_us_equity(entry.provider_kind, entry.venue) {
                        us_equity_ready_entries += 1;
                    }
                }
            }
        }

        let provider_statuses = provider_counts
            .into_iter()
            .map(|(provider, state)| (provider, state.to_status_line()))
            .collect::<BTreeMap<_, _>>();
        let compactness_summary = if canonical_paths == 0 {
            "no-canonical-output".to_string()
        } else if compact_paths == canonical_paths {
            "compact-only".to_string()
        } else {
            "mixed-output-size".to_string()
        };
        if total_ready_entries > 0
            && crypto_ready_entries == total_ready_entries
            && korean_equity_ready_entries == 0
            && us_equity_ready_entries == 0
        {
            reason_codes.push(ReasonCode::BenchmarkCryptoOnlyEvidence);
        }
        if korean_equity_ready_entries == 0
            && missing_auth_providers.iter().any(|value| value == "krx")
        {
            reason_codes.push(ReasonCode::BenchmarkKoreanEquityMissing);
        }
        if us_equity_ready_entries == 0
            && missing_auth_providers
                .iter()
                .any(|value| value == "alphavantage" || value == "alpaca")
        {
            reason_codes.push(ReasonCode::BenchmarkUsEquityMissing);
        }

        Self {
            total_ready_entries,
            crypto_ready_entries,
            korean_equity_ready_entries,
            us_equity_ready_entries,
            skipped_missing_auth_entries,
            skipped_budget_entries,
            failed_preflight_entries,
            provider_statuses,
            missing_auth_providers,
            compactness_summary,
            non_official_ready_entries,
            reason_codes: dedupe_reasons(reason_codes),
        }
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("total_ready_entries={}", self.total_ready_entries),
            format!("crypto_ready_entries={}", self.crypto_ready_entries),
            format!(
                "korean_equity_ready_entries={}",
                self.korean_equity_ready_entries
            ),
            format!("us_equity_ready_entries={}", self.us_equity_ready_entries),
            format!(
                "skipped_missing_auth_entries={}",
                self.skipped_missing_auth_entries
            ),
            format!("skipped_budget_entries={}", self.skipped_budget_entries),
            format!("failed_preflight_entries={}", self.failed_preflight_entries),
            format!(
                "non_official_ready_entries={}",
                self.non_official_ready_entries
            ),
            format!("compactness_summary={}", self.compactness_summary),
            format!(
                "missing_auth_providers={}",
                self.missing_auth_providers.join("|")
            ),
        ];
        for (provider, status) in &self.provider_statuses {
            lines.push(format!("provider_status={provider}:{status}"));
        }
        lines.push(format!(
            "reason_codes={}",
            self.reason_codes
                .iter()
                .map(|reason| format!("{reason:?}"))
                .collect::<Vec<_>>()
                .join("|")
        ));
        lines.join("\n")
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
struct ProviderCoverageState {
    collected: usize,
    skipped_missing_auth: usize,
    skipped_budget: usize,
    failed_preflight: usize,
    failed_provider: usize,
    diagnostic_only: usize,
}

impl ProviderCoverageState {
    fn to_status_line(&self) -> String {
        format!(
            "collected={};missing_auth={};budget={};failed_preflight={};failed_provider={};diagnostic={}",
            self.collected,
            self.skipped_missing_auth,
            self.skipped_budget,
            self.failed_preflight,
            self.failed_provider,
            self.diagnostic_only
        )
    }
}

fn provider_name(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Upbit => "upbit",
        ProviderKind::KrxOpenApi => "krx",
        ProviderKind::DataGoKrFscStockPrice => "data-go-kr-fsc-stock-price",
        ProviderKind::AlphaVantage => "alphavantage",
        ProviderKind::Alpaca => "alpaca",
        ProviderKind::Binance => "binance",
        ProviderKind::Korbit => "korbit",
        ProviderKind::KoreaInvestmentMarketData => "korea-investment",
        ProviderKind::PolygonProfessional => "polygon",
        ProviderKind::NasdaqDataLink => "nasdaq-data-link",
        ProviderKind::KoscomProfessional => "koscom",
        ProviderKind::MockFixture => "mock-fixture",
        ProviderKind::Unknown => "unknown",
    }
}

fn is_crypto(provider: ProviderKind, venue: Option<MarketVenue>) -> bool {
    matches!(
        provider,
        ProviderKind::Upbit | ProviderKind::Binance | ProviderKind::Korbit
    ) || matches!(venue, Some(MarketVenue::Upbit))
}

fn is_korean_equity(provider: ProviderKind, venue: Option<MarketVenue>) -> bool {
    matches!(
        provider,
        ProviderKind::KrxOpenApi
            | ProviderKind::DataGoKrFscStockPrice
            | ProviderKind::KoreaInvestmentMarketData
            | ProviderKind::KoscomProfessional
    ) || matches!(venue, Some(MarketVenue::KOSPI | MarketVenue::KOSDAQ))
}

fn is_us_equity(provider: ProviderKind, venue: Option<MarketVenue>) -> bool {
    matches!(
        provider,
        ProviderKind::AlphaVantage
            | ProviderKind::Alpaca
            | ProviderKind::PolygonProfessional
            | ProviderKind::NasdaqDataLink
    ) || matches!(
        venue,
        Some(MarketVenue::NASDAQ | MarketVenue::NYSE | MarketVenue::AMEX)
    )
}

fn dedupe_reasons(values: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}
