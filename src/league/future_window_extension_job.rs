use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string};
use crate::data::ProviderKind;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FutureWindowExtensionJobKind {
    LocalCsvWindowReuse,
    LocalCsvWindowExtension,
    OfficialCanonicalCsvImport,
    KrxEodFutureWindowCollect,
    DataGoKrEodFutureWindowCollect,
    AlphaVantageCompactFutureWindowCollect,
    AlpacaHistoricalFutureWindowCollect,
    UpbitCryptoFutureWindowCollect,
    SkippedMissingAuth,
    SkippedMissingApproval,
    SkippedMissingEndpointTemplate,
    SkippedMissingProvenance,
    SkippedMissingPreflight,
    SkippedSourceIneligible,
    SkippedBudgetExceeded,
    #[default]
    SkippedUnsupportedProvider,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FutureWindowExtensionJobStatus {
    Planned,
    ReadyToRun,
    RanSuccessfully,
    Skipped,
    Failed,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FutureWindowExtensionJob {
    pub job_id: String,
    pub job_kind: FutureWindowExtensionJobKind,
    #[serde(default)]
    pub provider_kind: Option<ProviderKind>,
    pub market: String,
    #[serde(default)]
    pub venue: Option<String>,
    pub symbol: String,
    pub timeframe: String,
    pub horizon_bars: usize,
    pub required_start_timestamp_ms: u64,
    pub required_end_timestamp_ms: u64,
    pub max_rows: usize,
    pub max_requests: usize,
    #[serde(default)]
    pub expected_output_csv: Option<String>,
    #[serde(default)]
    pub expected_provenance: Option<String>,
    #[serde(default)]
    pub expected_preflight: Option<String>,
    pub status: FutureWindowExtensionJobStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl FutureWindowExtensionJob {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_else(|_| self.job_id.clone()))
    }

    pub fn is_runnable(&self) -> bool {
        matches!(
            self.status,
            FutureWindowExtensionJobStatus::ReadyToRun
                | FutureWindowExtensionJobStatus::DiagnosticOnly
        )
    }

    pub fn is_provider_job(&self) -> bool {
        matches!(
            self.job_kind,
            FutureWindowExtensionJobKind::KrxEodFutureWindowCollect
                | FutureWindowExtensionJobKind::DataGoKrEodFutureWindowCollect
                | FutureWindowExtensionJobKind::AlphaVantageCompactFutureWindowCollect
                | FutureWindowExtensionJobKind::AlpacaHistoricalFutureWindowCollect
                | FutureWindowExtensionJobKind::UpbitCryptoFutureWindowCollect
        )
    }

    pub fn to_text(&self) -> String {
        format!(
            "job_id={};job_kind={:?};provider_kind={};market={};symbol={};timeframe={};horizon_bars={};required_start_timestamp_ms={};required_end_timestamp_ms={};status={:?};max_rows={};max_requests={};expected_output_csv={};expected_provenance={};expected_preflight={};fingerprint={}",
            self.job_id,
            self.job_kind,
            self.provider_kind
                .map(|value| format!("{value:?}"))
                .unwrap_or_default(),
            self.market,
            self.symbol,
            self.timeframe,
            self.horizon_bars,
            self.required_start_timestamp_ms,
            self.required_end_timestamp_ms,
            self.status,
            self.max_rows,
            self.max_requests,
            self.expected_output_csv.clone().unwrap_or_default(),
            self.expected_provenance.clone().unwrap_or_default(),
            self.expected_preflight.clone().unwrap_or_default(),
            self.fingerprint(),
        )
    }
}
