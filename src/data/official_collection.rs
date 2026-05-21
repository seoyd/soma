use crate::backtest::Timeframe;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::{
    AdjustedPricePolicy, AssetClass, AuthConfig, CandleFetchRequest, CollectionOutputSize,
    CollectionSizePolicy, CollectorRunner, MarketVenue, ProviderKind, RawArchivePolicy,
    RequestedOutputSize, RetentionPolicy,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionMode {
    None,
    GzipRawOnly,
    GzipCanonicalOnly,
    GzipRawAndCanonical,
    ZipBundle,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompressionPolicy {
    pub mode: CompressionMode,
    #[serde(default)]
    pub compression_level: Option<u32>,
    #[serde(default = "default_true")]
    pub deterministic_header: bool,
    #[serde(default = "default_true")]
    pub keep_uncompressed_for_preflight: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for CompressionPolicy {
    fn default() -> Self {
        Self {
            mode: CompressionMode::None,
            compression_level: None,
            deterministic_header: true,
            keep_uncompressed_for_preflight: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StorageBudget {
    pub max_total_bytes: usize,
    pub max_raw_bytes: usize,
    pub max_canonical_bytes: usize,
    pub max_manifest_bytes: usize,
    pub max_file_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for StorageBudget {
    fn default() -> Self {
        Self {
            max_total_bytes: 10 * 1024 * 1024,
            max_raw_bytes: 5 * 1024 * 1024,
            max_canonical_bytes: 3 * 1024 * 1024,
            max_manifest_bytes: 512 * 1024,
            max_file_count: 64,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StorageBudgetReport {
    pub total_bytes: usize,
    pub raw_bytes: usize,
    pub canonical_bytes: usize,
    pub manifest_bytes: usize,
    pub compressed_bytes: usize,
    pub uncompressed_bytes_estimate: usize,
    pub file_count: usize,
    pub budget_exceeded: bool,
    pub compression_applied: bool,
    pub retention_actions: Vec<String>,
    pub skipped_files: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for StorageBudgetReport {
    fn default() -> Self {
        Self {
            total_bytes: 0,
            raw_bytes: 0,
            canonical_bytes: 0,
            manifest_bytes: 0,
            compressed_bytes: 0,
            uncompressed_bytes_estimate: 0,
            file_count: 0,
            budget_exceeded: false,
            compression_applied: false,
            retention_actions: Vec::new(),
            skipped_files: Vec::new(),
            reason_codes: vec![ReasonCode::StorageBudgetReportBuilt],
        }
    }
}

impl StorageBudgetReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            format!("total_bytes={}", self.total_bytes),
            format!("raw_bytes={}", self.raw_bytes),
            format!("canonical_bytes={}", self.canonical_bytes),
            format!("manifest_bytes={}", self.manifest_bytes),
            format!("compressed_bytes={}", self.compressed_bytes),
            format!(
                "uncompressed_bytes_estimate={}",
                self.uncompressed_bytes_estimate
            ),
            format!("file_count={}", self.file_count),
            format!("budget_exceeded={}", self.budget_exceeded),
            format!("compression_applied={}", self.compression_applied),
            format!("retention_actions={}", self.retention_actions.join(" | ")),
            format!("skipped_files={}", self.skipped_files.join(" | ")),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCollectionPlan {
    pub plan_id: String,
    pub output_root: String,
    pub max_total_bytes: usize,
    pub max_total_rows: usize,
    pub max_total_requests: usize,
    #[serde(default)]
    pub default_collection_size_policy: CollectionSizePolicy,
    #[serde(default)]
    pub default_compression_policy: CompressionPolicy,
    #[serde(default)]
    pub default_retention_policy: RetentionPolicy,
    #[serde(default)]
    pub storage_budget: StorageBudget,
    pub entries: Vec<OfficialCollectionEntry>,
    #[serde(default = "default_true")]
    pub continue_on_missing_auth: bool,
    #[serde(default = "default_true")]
    pub continue_on_provider_failure: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialCollectionPlan {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reasons = Vec::new();
        if self.output_root.contains("://") {
            reasons.push(ReasonCode::LocalPathRejected);
        }
        for entry in &self.entries {
            if entry
                .fixture_path
                .as_deref()
                .is_some_and(|value| value.contains("://"))
            {
                reasons.push(ReasonCode::LocalPathRejected);
            }
        }
        dedupe_reasons(reasons)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCollectionEntry {
    pub entry_id: String,
    pub provider_kind: ProviderKind,
    pub symbol: String,
    #[serde(default)]
    pub normalized_symbol: Option<String>,
    #[serde(default)]
    pub venue: Option<MarketVenue>,
    pub asset_class: AssetClass,
    pub timeframe: Timeframe,
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub end: Option<String>,
    #[serde(default)]
    pub max_rows: Option<usize>,
    #[serde(default)]
    pub max_requests: Option<usize>,
    #[serde(default)]
    pub outputsize: Option<CollectionOutputSize>,
    #[serde(default)]
    pub auth_config_ref: Option<AuthConfig>,
    #[serde(default)]
    pub endpoint_template: Option<String>,
    #[serde(default)]
    pub fixture_path: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfficialCollectionEntryStatus {
    Collected,
    SkippedMissingAuth,
    SkippedBudgetExceeded,
    FailedProvider,
    FailedPreflight,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCollectionEntryReport {
    pub entry_id: String,
    pub provider_kind: ProviderKind,
    pub symbol: String,
    pub venue: Option<MarketVenue>,
    pub timeframe: Timeframe,
    pub status: OfficialCollectionEntryStatus,
    #[serde(default)]
    pub canonical_csv_path: Option<String>,
    #[serde(default)]
    pub manifest_path: Option<String>,
    #[serde(default)]
    pub provenance_path: Option<String>,
    #[serde(default)]
    pub preflight_status: Option<String>,
    pub row_count: usize,
    pub request_count: usize,
    pub bytes_written: usize,
    pub compressed: bool,
    pub ready_for_evidence: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCollectionReport {
    pub plan_id: String,
    pub entry_reports: Vec<OfficialCollectionEntryReport>,
    pub storage_budget_report: StorageBudgetReport,
    pub ready_entries_count: usize,
    pub skipped_entries_count: usize,
    pub failed_entries_count: usize,
    pub official_api_collected_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialCollectionReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let entry_lines = self
            .entry_reports
            .iter()
            .map(|entry| {
                format!(
                    "{} {:?} {:?} rows={} ready={} bytes={}",
                    entry.entry_id,
                    entry.provider_kind,
                    entry.status,
                    entry.row_count,
                    entry.ready_for_evidence,
                    entry.bytes_written
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        [
            format!("plan_id={}", self.plan_id),
            format!("ready_entries_count={}", self.ready_entries_count),
            format!("skipped_entries_count={}", self.skipped_entries_count),
            format!("failed_entries_count={}", self.failed_entries_count),
            format!(
                "official_api_collected_count={}",
                self.official_api_collected_count
            ),
            "entries:".to_string(),
            entry_lines,
            "storage_budget_report:".to_string(),
            self.storage_budget_report.to_text(),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let json_path = output_dir.join("official_collection_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_collection_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OfficialCollectionRunner {
    pub collector: CollectorRunner,
}

impl OfficialCollectionRunner {
    pub fn run_plan(&self, plan: &OfficialCollectionPlan) -> OfficialCollectionReport {
        if !plan.validate_local_paths().is_empty() {
            return OfficialCollectionReport {
                plan_id: plan.plan_id.clone(),
                entry_reports: Vec::new(),
                storage_budget_report: StorageBudgetReport {
                    budget_exceeded: true,
                    reason_codes: vec![
                        ReasonCode::LocalPathRejected,
                        ReasonCode::StorageBudgetReportBuilt,
                    ],
                    ..StorageBudgetReport::default()
                },
                ready_entries_count: 0,
                skipped_entries_count: 0,
                failed_entries_count: 0,
                official_api_collected_count: 0,
                reason_codes: vec![ReasonCode::LocalPathRejected],
            };
        }

        let mut entry_reports = Vec::new();
        let mut ready_entries_count = 0usize;
        let mut skipped_entries_count = 0usize;
        let mut failed_entries_count = 0usize;
        let mut official_api_collected_count = 0usize;
        let mut total_rows = 0usize;
        let mut total_requests = 0usize;
        let mut storage_budget_report = StorageBudgetReport::default();
        let base_output_dir = Path::new(&plan.output_root).join(&plan.plan_id);
        let _ = fs::create_dir_all(&base_output_dir);

        for entry in plan.entries.iter().filter(|entry| entry.enabled) {
            let entry_budget_rows = entry
                .max_rows
                .unwrap_or(plan.default_collection_size_policy.max_rows_per_symbol);
            let entry_budget_requests = entry
                .max_requests
                .unwrap_or(plan.default_collection_size_policy.max_requests_per_run);
            if total_rows.saturating_add(entry_budget_rows) > plan.max_total_rows
                || total_requests.saturating_add(entry_budget_requests) > plan.max_total_requests
            {
                skipped_entries_count += 1;
                entry_reports.push(OfficialCollectionEntryReport {
                    entry_id: entry.entry_id.clone(),
                    provider_kind: entry.provider_kind,
                    symbol: entry.symbol.clone(),
                    venue: entry.venue,
                    timeframe: entry.timeframe,
                    status: OfficialCollectionEntryStatus::SkippedBudgetExceeded,
                    canonical_csv_path: None,
                    manifest_path: None,
                    provenance_path: None,
                    preflight_status: None,
                    row_count: 0,
                    request_count: 0,
                    bytes_written: 0,
                    compressed: false,
                    ready_for_evidence: false,
                    reason_codes: vec![
                        ReasonCode::CollectionBudgetExceeded,
                        ReasonCode::OfficialCollectionEntrySkippedBudgetExceeded,
                    ],
                });
                continue;
            }

            let auth_missing = entry_requires_auth(entry.provider_kind)
                && entry.fixture_path.is_none()
                && entry
                    .auth_config_ref
                    .as_ref()
                    .and_then(|config| config.api_key_env_var.as_deref())
                    .and_then(|env_var| std::env::var(env_var).ok())
                    .filter(|value| !value.trim().is_empty())
                    .is_none();
            if auth_missing && plan.continue_on_missing_auth {
                skipped_entries_count += 1;
                entry_reports.push(OfficialCollectionEntryReport {
                    entry_id: entry.entry_id.clone(),
                    provider_kind: entry.provider_kind,
                    symbol: entry.symbol.clone(),
                    venue: entry.venue,
                    timeframe: entry.timeframe,
                    status: OfficialCollectionEntryStatus::SkippedMissingAuth,
                    canonical_csv_path: None,
                    manifest_path: None,
                    provenance_path: None,
                    preflight_status: None,
                    row_count: 0,
                    request_count: 0,
                    bytes_written: 0,
                    compressed: false,
                    ready_for_evidence: false,
                    reason_codes: vec![
                        ReasonCode::MissingApiKey,
                        ReasonCode::OfficialCollectionEntrySkippedMissingAuth,
                    ],
                });
                continue;
            }

            let mut policy = plan.default_collection_size_policy.clone();
            policy.max_rows_per_symbol = entry.max_rows.unwrap_or(policy.max_rows_per_symbol);
            policy.max_requests_per_run = entry.max_requests.unwrap_or(policy.max_requests_per_run);
            policy.retention_policy = plan.default_retention_policy;
            let request = CandleFetchRequest {
                request_id: format!("{}-{}", plan.plan_id, entry.entry_id),
                provider_kind: entry.provider_kind,
                symbol: entry.symbol.clone(),
                market_venue: entry.venue,
                asset_class: entry.asset_class,
                timeframe: entry.timeframe,
                start_timestamp_ms: entry
                    .start
                    .as_deref()
                    .map(parse_optional_timestamp)
                    .transpose()
                    .ok()
                    .flatten(),
                end_timestamp_ms: entry
                    .end
                    .as_deref()
                    .map(parse_optional_timestamp)
                    .transpose()
                    .ok()
                    .flatten(),
                output_root: base_output_dir.display().to_string(),
                limit_per_request: entry.max_requests,
                include_raw_archive: policy.raw_archive_policy != RawArchivePolicy::None,
                fill_missing_policy: super::FillMissingPolicy::LeaveGaps,
                fixture_path: entry.fixture_path.clone(),
                adjusted_price_policy: AdjustedPricePolicy::Raw,
                collection_size_policy: policy,
                auth_config: entry.auth_config_ref.clone(),
                endpoint_template: entry.endpoint_template.clone(),
                requested_output_size: map_outputsize(entry.outputsize),
                allow_full_history_override: false,
                reason_codes: vec![ReasonCode::OfficialCollectionPlanBuilt],
            };

            match self.collector.run(&request) {
                Ok(result) => {
                    let (raw_bytes, canonical_bytes, manifest_bytes, file_count) =
                        measure_entry_output(Path::new(&result.output_dir));
                    let mut bytes_written = raw_bytes + canonical_bytes + manifest_bytes;
                    let compressed = false;
                    if plan.default_compression_policy.mode != CompressionMode::None {
                        storage_budget_report.retention_actions.push(
                            "compression deferred; kept bounded uncompressed outputs".to_string(),
                        );
                        storage_budget_report
                            .reason_codes
                            .push(ReasonCode::CompressionDeferred);
                    }
                    if matches!(
                        plan.default_retention_policy,
                        RetentionPolicy::DeleteRawAfterCanonicalAndManifest
                    ) {
                        let raw_dir = Path::new(&result.output_dir).join("raw");
                        if raw_dir.exists() {
                            let _ = remove_files_in_dir(&raw_dir);
                            storage_budget_report
                                .retention_actions
                                .push(format!("deleted raw archive for entry {}", entry.entry_id));
                            storage_budget_report
                                .reason_codes
                                .push(ReasonCode::RetentionActionApplied);
                            let (
                                raw_bytes_after,
                                canonical_bytes_after,
                                manifest_bytes_after,
                                file_count_after,
                            ) = measure_entry_output(Path::new(&result.output_dir));
                            bytes_written =
                                raw_bytes_after + canonical_bytes_after + manifest_bytes_after;
                            storage_budget_report.raw_bytes = storage_budget_report
                                .raw_bytes
                                .saturating_add(raw_bytes_after);
                            storage_budget_report.canonical_bytes = storage_budget_report
                                .canonical_bytes
                                .saturating_add(canonical_bytes_after);
                            storage_budget_report.manifest_bytes = storage_budget_report
                                .manifest_bytes
                                .saturating_add(manifest_bytes_after);
                            storage_budget_report.file_count = storage_budget_report
                                .file_count
                                .saturating_add(file_count_after);
                        }
                    } else {
                        storage_budget_report.raw_bytes =
                            storage_budget_report.raw_bytes.saturating_add(raw_bytes);
                        storage_budget_report.canonical_bytes = storage_budget_report
                            .canonical_bytes
                            .saturating_add(canonical_bytes);
                        storage_budget_report.manifest_bytes = storage_budget_report
                            .manifest_bytes
                            .saturating_add(manifest_bytes);
                        storage_budget_report.file_count =
                            storage_budget_report.file_count.saturating_add(file_count);
                    }

                    total_rows = total_rows.saturating_add(result.row_count);
                    total_requests = total_requests.saturating_add(result.request_count);
                    storage_budget_report.total_bytes = storage_budget_report
                        .raw_bytes
                        .saturating_add(storage_budget_report.canonical_bytes)
                        .saturating_add(storage_budget_report.manifest_bytes);
                    storage_budget_report.uncompressed_bytes_estimate =
                        storage_budget_report.total_bytes;
                    storage_budget_report.compression_applied = compressed;
                    if storage_budget_report.total_bytes > plan.max_total_bytes
                        || storage_budget_report.total_bytes > plan.storage_budget.max_total_bytes
                        || storage_budget_report.raw_bytes > plan.storage_budget.max_raw_bytes
                        || storage_budget_report.canonical_bytes
                            > plan.storage_budget.max_canonical_bytes
                        || storage_budget_report.manifest_bytes
                            > plan.storage_budget.max_manifest_bytes
                        || storage_budget_report.file_count > plan.storage_budget.max_file_count
                    {
                        storage_budget_report.budget_exceeded = true;
                        storage_budget_report
                            .reason_codes
                            .push(ReasonCode::CollectionBudgetExceeded);
                    }

                    let status = if result.ready_for_real_evidence {
                        ready_entries_count += 1;
                        official_api_collected_count += 1;
                        OfficialCollectionEntryStatus::Collected
                    } else if result.preflight_status
                        == crate::data::PreflightFinalStatus::ReadyForRealEvidence
                    {
                        ready_entries_count += 1;
                        official_api_collected_count += 1;
                        OfficialCollectionEntryStatus::Collected
                    } else {
                        OfficialCollectionEntryStatus::DiagnosticOnly
                    };
                    entry_reports.push(OfficialCollectionEntryReport {
                        entry_id: entry.entry_id.clone(),
                        provider_kind: entry.provider_kind,
                        symbol: entry.symbol.clone(),
                        venue: entry.venue,
                        timeframe: entry.timeframe,
                        status,
                        canonical_csv_path: Some(result.canonical_csv_path.clone()),
                        manifest_path: Some(result.manifest_path.clone()),
                        provenance_path: Some(result.provenance_path.clone()),
                        preflight_status: Some(format!("{:?}", result.preflight_status)),
                        row_count: result.row_count,
                        request_count: result.request_count,
                        bytes_written,
                        compressed,
                        ready_for_evidence: result.ready_for_real_evidence,
                        reason_codes: if result.ready_for_real_evidence {
                            vec![ReasonCode::OfficialCollectionEntryCollected]
                        } else {
                            vec![ReasonCode::OfficialCollectionEntryDiagnosticOnly]
                        },
                    });
                    if storage_budget_report.budget_exceeded {
                        skipped_entries_count += plan
                            .entries
                            .iter()
                            .filter(|candidate| {
                                candidate.enabled && candidate.entry_id != entry.entry_id
                            })
                            .count();
                        break;
                    }
                }
                Err(err) => {
                    if err.contains("MissingApiKey") && plan.continue_on_missing_auth {
                        skipped_entries_count += 1;
                        entry_reports.push(OfficialCollectionEntryReport {
                            entry_id: entry.entry_id.clone(),
                            provider_kind: entry.provider_kind,
                            symbol: entry.symbol.clone(),
                            venue: entry.venue,
                            timeframe: entry.timeframe,
                            status: OfficialCollectionEntryStatus::SkippedMissingAuth,
                            canonical_csv_path: None,
                            manifest_path: None,
                            provenance_path: None,
                            preflight_status: None,
                            row_count: 0,
                            request_count: 0,
                            bytes_written: 0,
                            compressed: false,
                            ready_for_evidence: false,
                            reason_codes: vec![
                                ReasonCode::MissingApiKey,
                                ReasonCode::OfficialCollectionEntrySkippedMissingAuth,
                            ],
                        });
                    } else {
                        failed_entries_count += 1;
                        entry_reports.push(OfficialCollectionEntryReport {
                            entry_id: entry.entry_id.clone(),
                            provider_kind: entry.provider_kind,
                            symbol: entry.symbol.clone(),
                            venue: entry.venue,
                            timeframe: entry.timeframe,
                            status: OfficialCollectionEntryStatus::FailedProvider,
                            canonical_csv_path: None,
                            manifest_path: None,
                            provenance_path: None,
                            preflight_status: None,
                            row_count: 0,
                            request_count: 0,
                            bytes_written: 0,
                            compressed: false,
                            ready_for_evidence: false,
                            reason_codes: vec![
                                ReasonCode::OfficialCollectionEntryFailedProvider,
                                ReasonCode::ProviderRequestFailed,
                            ],
                        });
                        if !plan.continue_on_provider_failure {
                            break;
                        }
                    }
                }
            }
        }

        storage_budget_report.reason_codes =
            dedupe_reasons(storage_budget_report.reason_codes.clone());
        let report = OfficialCollectionReport {
            plan_id: plan.plan_id.clone(),
            entry_reports,
            storage_budget_report,
            ready_entries_count,
            skipped_entries_count,
            failed_entries_count,
            official_api_collected_count,
            reason_codes: vec![ReasonCode::OfficialCollectionRan],
        };
        let _ = report.write_to_dir(&base_output_dir);
        report
    }
}

fn measure_entry_output(path: &Path) -> (usize, usize, usize, usize) {
    let mut raw_bytes = 0usize;
    let mut canonical_bytes = 0usize;
    let mut manifest_bytes = 0usize;
    let mut file_count = 0usize;
    if !path.exists() {
        return (0, 0, 0, 0);
    }
    visit_files(path, &mut |file_path, bytes| {
        file_count += 1;
        let file_string = file_path.to_string_lossy();
        if file_string.contains("/raw/") {
            raw_bytes = raw_bytes.saturating_add(bytes);
        } else if file_string.contains("/canonical/") {
            canonical_bytes = canonical_bytes.saturating_add(bytes);
        } else {
            manifest_bytes = manifest_bytes.saturating_add(bytes);
        }
    });
    (raw_bytes, canonical_bytes, manifest_bytes, file_count)
}

fn visit_files(path: &Path, visitor: &mut dyn FnMut(&Path, usize)) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                visit_files(&entry_path, visitor);
            } else if let Ok(metadata) = entry.metadata() {
                visitor(&entry_path, metadata.len() as usize);
            }
        }
    }
}

fn remove_files_in_dir(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(path).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        if entry.file_type().map_err(|err| err.to_string())?.is_file() {
            fs::remove_file(entry.path()).map_err(|err| err.to_string())?;
        }
    }
    Ok(())
}

fn map_outputsize(value: Option<CollectionOutputSize>) -> Option<RequestedOutputSize> {
    match value {
        Some(CollectionOutputSize::Compact) => Some(RequestedOutputSize::Compact),
        Some(CollectionOutputSize::FullAllowedOnlyWithExplicitFlag) => {
            Some(RequestedOutputSize::Full)
        }
        _ => None,
    }
}

fn entry_requires_auth(provider_kind: ProviderKind) -> bool {
    matches!(
        provider_kind,
        ProviderKind::KrxOpenApi | ProviderKind::AlphaVantage | ProviderKind::Alpaca
    )
}

fn parse_optional_timestamp(value: &str) -> Result<u64, String> {
    if value.len() == 8 && value.chars().all(|ch| ch.is_ascii_digit()) {
        let year = value[0..4].parse::<i32>().map_err(|err| err.to_string())?;
        let month = value[4..6].parse::<u32>().map_err(|err| err.to_string())?;
        let day = value[6..8].parse::<u32>().map_err(|err| err.to_string())?;
        let days = days_from_civil(year, month, day);
        let millis = days
            .checked_mul(86_400_000)
            .ok_or_else(|| "timestamp overflow".to_string())?;
        return u64::try_from(millis).map_err(|_| "timestamp overflow".to_string());
    }
    value.parse::<u64>().map_err(|err| err.to_string())
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let mp = month as i32 + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    i64::from(era * 146_097 + doe - 719_468)
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

fn default_true() -> bool {
    true
}
