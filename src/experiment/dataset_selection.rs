use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{OfficialCollectionEntryStatus, OfficialCollectionReport, ProviderKind};
use crate::experiment::core_benchmark::is_non_official_provider;
use crate::experiment::{OfficialDatasetCoverageStatus, SelectedOfficialDatasets};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialDatasetSelectionPolicy {
    pub min_ready_official_datasets: usize,
    pub allow_crypto_only: bool,
    pub allow_missing_equity_auth: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OfficialBenchmarkDatasetSelector;

impl OfficialBenchmarkDatasetSelector {
    pub fn select_ready_entries(
        &self,
        collection_report: &OfficialCollectionReport,
        policy: &OfficialDatasetSelectionPolicy,
    ) -> SelectedOfficialDatasets {
        let mut selected_entries = Vec::new();
        let mut skipped_entries = Vec::new();
        let mut crypto_entries = Vec::new();
        let mut korean_equity_entries = Vec::new();
        let mut us_equity_entries = Vec::new();
        let mut missing_auth_entries = Vec::new();
        let mut failed_preflight_entries = Vec::new();

        for entry in &collection_report.entry_reports {
            match entry.status {
                OfficialCollectionEntryStatus::SkippedMissingAuth => {
                    missing_auth_entries.push(entry.entry_id.clone());
                    skipped_entries.push(entry.entry_id.clone());
                }
                OfficialCollectionEntryStatus::FailedPreflight => {
                    failed_preflight_entries.push(entry.entry_id.clone());
                    skipped_entries.push(entry.entry_id.clone());
                }
                _ if !entry.ready_for_evidence || is_non_official_provider(entry.provider_kind) => {
                    skipped_entries.push(entry.entry_id.clone());
                }
                _ => {
                    selected_entries.push(entry.entry_id.clone());
                    if is_crypto(entry.provider_kind) {
                        crypto_entries.push(entry.entry_id.clone());
                    } else if is_korean_equity(entry.provider_kind) {
                        korean_equity_entries.push(entry.entry_id.clone());
                    } else if is_us_equity(entry.provider_kind) {
                        us_equity_entries.push(entry.entry_id.clone());
                    }
                }
            }
        }

        selected_entries.sort();
        skipped_entries.sort();
        crypto_entries.sort();
        korean_equity_entries.sort();
        us_equity_entries.sort();
        missing_auth_entries.sort();
        failed_preflight_entries.sort();

        let coverage_status = if selected_entries.is_empty() {
            if !missing_auth_entries.is_empty() {
                OfficialDatasetCoverageStatus::MissingEquityAuth
            } else {
                OfficialDatasetCoverageStatus::MissingOfficialData
            }
        } else if selected_entries.len() < policy.min_ready_official_datasets {
            OfficialDatasetCoverageStatus::InsufficientReadyEntries
        } else if !missing_auth_entries.is_empty() && !policy.allow_missing_equity_auth {
            OfficialDatasetCoverageStatus::MissingEquityAuth
        } else if korean_equity_entries.is_empty() && us_equity_entries.is_empty() {
            OfficialDatasetCoverageStatus::CryptoOnly
        } else {
            OfficialDatasetCoverageStatus::MultiVenue
        };

        let mut reason_codes = vec![ReasonCode::OfficialDatasetCoverageBuilt];
        match coverage_status {
            OfficialDatasetCoverageStatus::CryptoOnly => {
                reason_codes.push(ReasonCode::BenchmarkCryptoOnlyEvidence)
            }
            OfficialDatasetCoverageStatus::MissingEquityAuth => {
                reason_codes.push(ReasonCode::MissingAuth)
            }
            OfficialDatasetCoverageStatus::MissingOfficialData => {
                reason_codes.push(ReasonCode::AiSignalMissingOfficialData)
            }
            OfficialDatasetCoverageStatus::InsufficientReadyEntries => {
                reason_codes.push(ReasonCode::AiSignalInsufficientOfficialData)
            }
            OfficialDatasetCoverageStatus::MultiVenue => {}
        }

        SelectedOfficialDatasets {
            selected_entries,
            skipped_entries,
            crypto_entries,
            korean_equity_entries,
            us_equity_entries,
            missing_auth_entries,
            failed_preflight_entries,
            coverage_status,
            reason_codes,
        }
    }
}

fn is_crypto(provider_kind: ProviderKind) -> bool {
    matches!(provider_kind, ProviderKind::Upbit)
}

fn is_korean_equity(provider_kind: ProviderKind) -> bool {
    matches!(
        provider_kind,
        ProviderKind::KrxOpenApi | ProviderKind::KoreaInvestmentMarketData
    )
}

fn is_us_equity(provider_kind: ProviderKind) -> bool {
    matches!(
        provider_kind,
        ProviderKind::AlphaVantage | ProviderKind::Alpaca
    )
}
