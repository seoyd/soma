use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::data::{ProviderKind, StorageBudgetReport};

use super::candle_expansion_operator_actions::CandleExpansionOperatorAction;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CandleAcquisitionJobKind {
    LocalOfficialCsvImport,
    ExistingCanonicalCsvReuse,
    KrxEodCollect,
    DataGoKrEodCollect,
    AlphaVantageCompactDailyCollect,
    AlpacaHistoricalBarsCollect,
    UpbitCryptoCandleCollect,
    DiagnosticControlledImport,
    SkippedMissingAuth,
    SkippedMissingApproval,
    SkippedMissingEndpointTemplate,
    SkippedSourceNotEligible,
    SkippedBudgetExceeded,
    #[default]
    SkippedUnsupportedProvider,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CandleAcquisitionJobStatus {
    Planned,
    ReadyToRun,
    RanSuccessfully,
    Skipped,
    Failed,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleAcquisitionJob {
    pub job_id: String,
    pub job_kind: CandleAcquisitionJobKind,
    #[serde(default)]
    pub provider_kind: Option<ProviderKind>,
    pub market: String,
    pub symbol: String,
    #[serde(default)]
    pub venue: Option<String>,
    pub timeframe: String,
    pub horizon_bars: usize,
    #[serde(default)]
    pub start_timestamp_ms: Option<u64>,
    #[serde(default)]
    pub end_timestamp_ms: Option<u64>,
    pub max_rows: usize,
    pub max_requests: usize,
    pub output_root: String,
    #[serde(default)]
    pub local_input_csv_path: Option<String>,
    #[serde(default)]
    pub local_input_provenance_path: Option<String>,
    #[serde(default)]
    pub local_input_preflight_path: Option<String>,
    #[serde(default)]
    pub local_input_manifest_path: Option<String>,
    #[serde(default)]
    pub expected_canonical_csv_path: Option<String>,
    #[serde(default)]
    pub expected_provenance_path: Option<String>,
    #[serde(default)]
    pub expected_preflight_path: Option<String>,
    pub status: CandleAcquisitionJobStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleAcquisitionPlan {
    pub plan_id: String,
    pub jobs: Vec<CandleAcquisitionJob>,
    pub runnable_jobs: usize,
    pub skipped_jobs: usize,
    pub import_jobs: usize,
    pub collection_jobs: usize,
    pub operator_actions: Vec<CandleExpansionOperatorAction>,
    pub storage_budget_summary: StorageBudgetReport,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CandleAcquisitionJob {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_else(|_| self.job_id.clone()))
    }

    pub fn is_runnable(&self) -> bool {
        matches!(
            self.status,
            CandleAcquisitionJobStatus::ReadyToRun | CandleAcquisitionJobStatus::DiagnosticOnly
        )
    }

    pub fn is_import_job(&self) -> bool {
        matches!(
            self.job_kind,
            CandleAcquisitionJobKind::LocalOfficialCsvImport
                | CandleAcquisitionJobKind::ExistingCanonicalCsvReuse
                | CandleAcquisitionJobKind::DiagnosticControlledImport
        )
    }

    pub fn is_collection_job(&self) -> bool {
        matches!(
            self.job_kind,
            CandleAcquisitionJobKind::KrxEodCollect
                | CandleAcquisitionJobKind::DataGoKrEodCollect
                | CandleAcquisitionJobKind::AlphaVantageCompactDailyCollect
                | CandleAcquisitionJobKind::AlpacaHistoricalBarsCollect
                | CandleAcquisitionJobKind::UpbitCryptoCandleCollect
        )
    }

    pub fn to_text(&self) -> String {
        format!(
            "job_id={};job_kind={:?};provider_kind={};market={};symbol={};timeframe={};horizon_bars={};status={:?};max_rows={};max_requests={};local_input_csv_path={};expected_canonical_csv_path={};expected_provenance_path={};expected_preflight_path={};fingerprint={}",
            self.job_id,
            self.job_kind,
            self.provider_kind
                .map(|provider| format!("{provider:?}"))
                .unwrap_or_default(),
            self.market,
            self.symbol,
            self.timeframe,
            self.horizon_bars,
            self.status,
            self.max_rows,
            self.max_requests,
            self.local_input_csv_path.clone().unwrap_or_default(),
            self.expected_canonical_csv_path.clone().unwrap_or_default(),
            self.expected_provenance_path.clone().unwrap_or_default(),
            self.expected_preflight_path.clone().unwrap_or_default(),
            self.fingerprint(),
        )
    }
}

impl CandleAcquisitionPlan {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_else(|_| self.plan_id.clone()))
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("plan_id={}", self.plan_id),
            format!("runnable_jobs={}", self.runnable_jobs),
            format!("skipped_jobs={}", self.skipped_jobs),
            format!("import_jobs={}", self.import_jobs),
            format!("collection_jobs={}", self.collection_jobs),
            format!("warnings={}", self.warnings.join(" | ")),
            format!("fingerprint={}", self.fingerprint()),
            "storage_budget_summary:".to_string(),
            self.storage_budget_summary.to_text(),
            "operator_actions:".to_string(),
        ];
        lines.extend(self.operator_actions.iter().map(|action| action.to_text()));
        lines.push("jobs:".to_string());
        lines.extend(self.jobs.iter().map(CandleAcquisitionJob::to_text));
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(output_dir.join("acquisition_plan.txt"), self.to_text())
            .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("candle_acquisition_plan.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }

    pub fn from_jobs(
        plan_id: String,
        mut jobs: Vec<CandleAcquisitionJob>,
        mut operator_actions: Vec<CandleExpansionOperatorAction>,
        storage_budget_summary: StorageBudgetReport,
        warnings: Vec<String>,
        reason_codes: Vec<ReasonCode>,
    ) -> Self {
        jobs.sort_by(|left, right| {
            left.job_id
                .cmp(&right.job_id)
                .then(left.job_kind.cmp(&right.job_kind))
                .then(left.symbol.cmp(&right.symbol))
                .then(left.timeframe.cmp(&right.timeframe))
        });
        operator_actions.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
        let runnable_jobs = jobs.iter().filter(|job| job.is_runnable()).count();
        let skipped_jobs = jobs
            .iter()
            .filter(|job| job.status == CandleAcquisitionJobStatus::Skipped)
            .count();
        let import_jobs = jobs.iter().filter(|job| job.is_import_job()).count();
        let collection_jobs = jobs.iter().filter(|job| job.is_collection_job()).count();
        Self {
            plan_id,
            jobs,
            runnable_jobs,
            skipped_jobs,
            import_jobs,
            collection_jobs,
            operator_actions,
            storage_budget_summary,
            warnings,
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }
}
