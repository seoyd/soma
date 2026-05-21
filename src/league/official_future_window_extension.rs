use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::data::{ProviderKind, ProviderMarket, StorageBudgetReport};

use super::future_window_extension_job::{
    FutureWindowExtensionJob, FutureWindowExtensionJobKind, FutureWindowExtensionJobStatus,
};
use super::future_window_requirements::{
    FutureWindowGapKind, FutureWindowRequirementConfig, FutureWindowRequirementItem,
    descriptor_from_csv_path, load_descriptor_map_from_paths,
    load_future_window_requirement_from_path_or_config,
};
use super::official_candle_coverage_pack::{OfficialCandleSeriesDescriptor, normalize_symbol};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialFutureWindowExtensionConfig {
    pub extension_id: String,
    #[serde(default)]
    pub future_window_requirement_path: Option<String>,
    #[serde(default)]
    pub official_canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub provenance_paths: Vec<String>,
    #[serde(default)]
    pub preflight_report_paths: Vec<String>,
    #[serde(default)]
    pub provider_readiness_report_paths: Vec<String>,
    #[serde(default)]
    pub provider_reality_report_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_jobs")]
    pub max_jobs: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_rows_per_job")]
    pub max_rows_per_job: usize,
    #[serde(default = "default_max_requests_per_job")]
    pub max_requests_per_job: usize,
    #[serde(default = "default_max_total_bytes")]
    pub max_total_bytes: usize,
    #[serde(default = "default_true")]
    pub prefer_local_extension: bool,
    #[serde(default = "default_true")]
    pub generate_provider_jobs: bool,
    #[serde(default = "default_true")]
    pub run_local_extension_jobs: bool,
    #[serde(default)]
    pub run_provider_collection_jobs: bool,
    #[serde(default = "default_true")]
    pub allow_krx: bool,
    #[serde(default = "default_true")]
    pub allow_data_go_kr: bool,
    #[serde(default = "default_true")]
    pub allow_alpha_vantage: bool,
    #[serde(default = "default_true")]
    pub allow_alpaca: bool,
    #[serde(default = "default_true")]
    pub allow_upbit_crypto: bool,
    #[serde(default)]
    pub allow_controlled_diagnostic: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FutureWindowExtensionPlan {
    pub extension_id: String,
    pub jobs: Vec<FutureWindowExtensionJob>,
    pub runnable_jobs: usize,
    pub skipped_jobs: usize,
    pub operator_actions: Vec<String>,
    pub expected_added_future_windows: usize,
    pub expected_added_outcome_buildable_rows: usize,
    pub storage_budget_summary: StorageBudgetReport,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for OfficialFutureWindowExtensionConfig {
    fn default() -> Self {
        Self {
            extension_id: "official-future-window-extension".to_string(),
            future_window_requirement_path: None,
            official_canonical_csv_paths: Vec::new(),
            provenance_paths: Vec::new(),
            preflight_report_paths: Vec::new(),
            provider_readiness_report_paths: Vec::new(),
            provider_reality_report_paths: Vec::new(),
            output_root: default_output_root(),
            max_jobs: default_max_jobs(),
            max_symbols: default_max_symbols(),
            max_rows_per_job: default_max_rows_per_job(),
            max_requests_per_job: default_max_requests_per_job(),
            max_total_bytes: default_max_total_bytes(),
            prefer_local_extension: true,
            generate_provider_jobs: true,
            run_local_extension_jobs: true,
            run_provider_collection_jobs: false,
            allow_krx: true,
            allow_data_go_kr: true,
            allow_alpha_vantage: true,
            allow_alpaca: true,
            allow_upbit_crypto: true,
            allow_controlled_diagnostic: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialFutureWindowExtensionConfig {
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

    pub fn validate(&self) -> Result<(), String> {
        if self.extension_id.trim().is_empty() {
            return Err("future window extension id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("future window extension paths must be local".to_string());
        }
        if self.max_jobs == 0 || self.max_jobs > default_max_jobs() {
            return Err("future window extension max_jobs must be between 1 and 10".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err("future window extension max_symbols must be between 1 and 5".to_string());
        }
        if self.max_rows_per_job == 0 || self.max_rows_per_job > default_max_rows_per_job() {
            return Err(
                "future window extension max_rows_per_job must be between 1 and 500".to_string(),
            );
        }
        if self.max_requests_per_job == 0
            || self.max_requests_per_job > default_max_requests_per_job()
        {
            return Err(
                "future window extension max_requests_per_job must be between 1 and 10".to_string(),
            );
        }
        if self.max_total_bytes == 0 || self.max_total_bytes > default_max_total_bytes() {
            return Err(
                "future window extension max_total_bytes must be between 1 and 5000000".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.extension_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.future_window_requirement_path
            .iter()
            .cloned()
            .chain(self.official_canonical_csv_paths.iter().cloned())
            .chain(self.provenance_paths.iter().cloned())
            .chain(self.preflight_report_paths.iter().cloned())
            .chain(self.provider_readiness_report_paths.iter().cloned())
            .chain(self.provider_reality_report_paths.iter().cloned())
            .collect()
    }
}

pub fn build_official_future_window_extension_plan(
    config: &OfficialFutureWindowExtensionConfig,
) -> Result<FutureWindowExtensionPlan, String> {
    config.validate()?;
    let requirement_report = if let Some(path) = config.future_window_requirement_path.as_deref() {
        load_future_window_requirement_from_path_or_config(path)?
    } else {
        let derived = FutureWindowRequirementConfig {
            requirement_id: format!("{}-derived-requirements", config.extension_id),
            candle_coverage_pack_paths: config.official_canonical_csv_paths.clone(),
            output_root: config.output_root.clone(),
            ..FutureWindowRequirementConfig::default()
        };
        load_future_window_requirement_from_path_or_config(
            &derived.to_toml_string().map_err(|err| err.to_string())?,
        )?
    };
    let local_descriptors = load_local_descriptors(config)?;
    let mut jobs = Vec::new();
    let mut operator_actions = BTreeSet::new();
    let mut total_bytes = 0usize;
    let mut symbols = BTreeSet::new();

    for (index, item) in requirement_report.items.iter().enumerate() {
        if index >= config.max_jobs {
            break;
        }
        symbols.insert(normalize_symbol(&item.symbol));
        if symbols.len() > config.max_symbols {
            break;
        }
        if item.gap_kind == FutureWindowGapKind::SufficientFutureBars {
            continue;
        }
        let mut job = build_job(config, item, &local_descriptors, &mut operator_actions);
        let job_bytes = job
            .expected_output_csv
            .as_ref()
            .and_then(|path| fs::metadata(path).ok())
            .map(|metadata| metadata.len() as usize)
            .unwrap_or(0);
        if total_bytes.saturating_add(job_bytes) > config.max_total_bytes {
            job.job_kind = FutureWindowExtensionJobKind::SkippedBudgetExceeded;
            job.status = FutureWindowExtensionJobStatus::Skipped;
            job.reason_codes = stable_reason_codes(
                &job.reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::BudgetExceeded,
                        ReasonCode::CollectionBudgetExceeded,
                    ])
                    .collect::<Vec<_>>(),
            );
        } else {
            total_bytes = total_bytes.saturating_add(job_bytes);
        }
        collect_operator_actions_for_job(&job, &mut operator_actions);
        jobs.push(job);
    }

    jobs.sort_by(|left, right| {
        left.job_id
            .cmp(&right.job_id)
            .then(left.job_kind.cmp(&right.job_kind))
            .then(left.symbol.cmp(&right.symbol))
            .then(left.timeframe.cmp(&right.timeframe))
    });
    let runnable_jobs = jobs.iter().filter(|job| job.is_runnable()).count();
    let skipped_jobs = jobs
        .iter()
        .filter(|job| job.status == FutureWindowExtensionJobStatus::Skipped)
        .count();
    let expected_added_future_windows = jobs
        .iter()
        .filter(|job| {
            !matches!(
                job.job_kind,
                FutureWindowExtensionJobKind::SkippedMissingAuth
                    | FutureWindowExtensionJobKind::SkippedMissingApproval
                    | FutureWindowExtensionJobKind::SkippedMissingEndpointTemplate
                    | FutureWindowExtensionJobKind::SkippedMissingProvenance
                    | FutureWindowExtensionJobKind::SkippedMissingPreflight
                    | FutureWindowExtensionJobKind::SkippedSourceIneligible
                    | FutureWindowExtensionJobKind::SkippedBudgetExceeded
                    | FutureWindowExtensionJobKind::SkippedUnsupportedProvider
            )
        })
        .count();
    let expected_added_outcome_buildable_rows = jobs
        .iter()
        .filter(|job| {
            matches!(
                job.job_kind,
                FutureWindowExtensionJobKind::LocalCsvWindowReuse
                    | FutureWindowExtensionJobKind::LocalCsvWindowExtension
                    | FutureWindowExtensionJobKind::OfficialCanonicalCsvImport
            )
        })
        .count();
    let storage_budget_summary = StorageBudgetReport {
        total_bytes,
        canonical_bytes: total_bytes,
        file_count: jobs
            .iter()
            .filter(|job| job.expected_output_csv.is_some())
            .count(),
        budget_exceeded: total_bytes > config.max_total_bytes,
        reason_codes: stable_reason_codes(&[
            ReasonCode::StorageBudgetReportBuilt,
            ReasonCode::DeterministicPath,
        ]),
        ..StorageBudgetReport::default()
    };

    Ok(FutureWindowExtensionPlan {
        extension_id: config.extension_id.clone(),
        jobs,
        runnable_jobs,
        skipped_jobs,
        operator_actions: operator_actions.into_iter().collect(),
        expected_added_future_windows,
        expected_added_outcome_buildable_rows,
        storage_budget_summary,
        reason_codes: stable_reason_codes(
            &config
                .reason_codes
                .iter()
                .cloned()
                .chain([ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly])
                .collect::<Vec<_>>(),
        ),
    })
}

impl FutureWindowExtensionPlan {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(
            &serde_json::to_string(self).unwrap_or_else(|_| self.extension_id.clone()),
        )
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("extension_id={}", self.extension_id),
            format!("runnable_jobs={}", self.runnable_jobs),
            format!("skipped_jobs={}", self.skipped_jobs),
            format!(
                "expected_added_future_windows={}",
                self.expected_added_future_windows
            ),
            format!(
                "expected_added_outcome_buildable_rows={}",
                self.expected_added_outcome_buildable_rows
            ),
            format!("operator_actions={}", self.operator_actions.join(" | ")),
            format!("fingerprint={}", self.fingerprint()),
            "storage_budget_summary:".to_string(),
            self.storage_budget_summary.to_text(),
            "jobs:".to_string(),
        ];
        lines.extend(self.jobs.iter().map(FutureWindowExtensionJob::to_text));
        lines.join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("future_window_extension_plan.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("future_window_extension_plan.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn load_local_descriptors(
    config: &OfficialFutureWindowExtensionConfig,
) -> Result<BTreeMap<String, OfficialCandleSeriesDescriptor>, String> {
    let mut descriptors = load_descriptor_map_from_paths(&config.official_canonical_csv_paths)?;
    for path in &config.official_canonical_csv_paths {
        if path.ends_with(".csv") {
            let descriptor = descriptor_from_csv_path(path)?;
            descriptors.insert(descriptor.candle_series_id.clone(), descriptor);
        }
    }
    Ok(descriptors)
}

fn build_job(
    config: &OfficialFutureWindowExtensionConfig,
    item: &FutureWindowRequirementItem,
    local_descriptors: &BTreeMap<String, OfficialCandleSeriesDescriptor>,
    operator_actions: &mut BTreeSet<String>,
) -> FutureWindowExtensionJob {
    let market = format!("{:?}", item.market);
    let descriptor = best_local_descriptor(item, local_descriptors);
    let mut reason_codes = item.reason_codes.clone();

    if matches!(item.gap_kind, FutureWindowGapKind::SourceIneligible) {
        reason_codes.push(ReasonCode::ReadinessEvidenceExcluded);
        return FutureWindowExtensionJob {
            job_id: format!("{}-skip-source", item.row_id),
            job_kind: FutureWindowExtensionJobKind::SkippedSourceIneligible,
            provider_kind: None,
            market,
            venue: item.venue.clone(),
            symbol: item.symbol.clone(),
            timeframe: item.timeframe.clone(),
            horizon_bars: item.horizon_bars,
            required_start_timestamp_ms: item.required_start_timestamp_ms,
            required_end_timestamp_ms: item.required_end_timestamp_ms,
            max_rows: config.max_rows_per_job,
            max_requests: config.max_requests_per_job,
            expected_output_csv: None,
            expected_provenance: None,
            expected_preflight: None,
            status: FutureWindowExtensionJobStatus::Skipped,
            reason_codes: stable_reason_codes(&reason_codes),
        };
    }

    if config.prefer_local_extension {
        if let Some(descriptor) = descriptor {
            let sidecars = sidecars_for_csv(Path::new(&descriptor.path));
            let output_csv = Some(descriptor.path.clone());
            let output_provenance = sidecars.provenance.clone();
            let output_preflight = sidecars.preflight.clone();
            if sidecars.provenance.is_none() {
                operator_actions.insert("ProvideOfficialProvenance".to_string());
                reason_codes.push(ReasonCode::MissingOfficialProvenance);
                return FutureWindowExtensionJob {
                    job_id: format!("{}-local-sidecar", item.row_id),
                    job_kind: FutureWindowExtensionJobKind::SkippedMissingProvenance,
                    provider_kind: None,
                    market,
                    venue: item.venue.clone(),
                    symbol: item.symbol.clone(),
                    timeframe: item.timeframe.clone(),
                    horizon_bars: item.horizon_bars,
                    required_start_timestamp_ms: item.required_start_timestamp_ms,
                    required_end_timestamp_ms: item.required_end_timestamp_ms,
                    max_rows: config.max_rows_per_job,
                    max_requests: config.max_requests_per_job,
                    expected_output_csv: output_csv,
                    expected_provenance: output_provenance,
                    expected_preflight: output_preflight,
                    status: FutureWindowExtensionJobStatus::Skipped,
                    reason_codes: stable_reason_codes(&reason_codes),
                };
            }
            if sidecars.preflight.is_none() {
                operator_actions.insert("RunDataPreflight".to_string());
                reason_codes.push(ReasonCode::MissingOfficialPreflight);
                return FutureWindowExtensionJob {
                    job_id: format!("{}-local-sidecar", item.row_id),
                    job_kind: FutureWindowExtensionJobKind::SkippedMissingPreflight,
                    provider_kind: None,
                    market,
                    venue: item.venue.clone(),
                    symbol: item.symbol.clone(),
                    timeframe: item.timeframe.clone(),
                    horizon_bars: item.horizon_bars,
                    required_start_timestamp_ms: item.required_start_timestamp_ms,
                    required_end_timestamp_ms: item.required_end_timestamp_ms,
                    max_rows: config.max_rows_per_job,
                    max_requests: config.max_requests_per_job,
                    expected_output_csv: output_csv,
                    expected_provenance: output_provenance,
                    expected_preflight: output_preflight,
                    status: FutureWindowExtensionJobStatus::Skipped,
                    reason_codes: stable_reason_codes(&reason_codes),
                };
            }

            let job_kind = if descriptor.timestamp_end_ms >= item.required_end_timestamp_ms {
                if item.candle_series_id.as_deref() == Some(descriptor.candle_series_id.as_str()) {
                    FutureWindowExtensionJobKind::LocalCsvWindowReuse
                } else {
                    FutureWindowExtensionJobKind::OfficialCanonicalCsvImport
                }
            } else {
                FutureWindowExtensionJobKind::LocalCsvWindowExtension
            };
            return FutureWindowExtensionJob {
                job_id: format!("{}-local", item.row_id),
                job_kind,
                provider_kind: descriptor.provider_kind,
                market,
                venue: item.venue.clone(),
                symbol: item.symbol.clone(),
                timeframe: item.timeframe.clone(),
                horizon_bars: item.horizon_bars,
                required_start_timestamp_ms: item.required_start_timestamp_ms,
                required_end_timestamp_ms: item.required_end_timestamp_ms,
                max_rows: config.max_rows_per_job,
                max_requests: config.max_requests_per_job,
                expected_output_csv: Some(descriptor.path.clone()),
                expected_provenance: sidecars.provenance,
                expected_preflight: sidecars.preflight,
                status: if item.gap_kind == FutureWindowGapKind::DiagnosticOnly {
                    FutureWindowExtensionJobStatus::DiagnosticOnly
                } else if config.run_local_extension_jobs {
                    FutureWindowExtensionJobStatus::ReadyToRun
                } else {
                    FutureWindowExtensionJobStatus::Planned
                },
                reason_codes: stable_reason_codes(&reason_codes),
            };
        }
    }

    build_provider_job(config, item, market, reason_codes)
}

fn build_provider_job(
    config: &OfficialFutureWindowExtensionConfig,
    item: &FutureWindowRequirementItem,
    market: String,
    mut reason_codes: Vec<ReasonCode>,
) -> FutureWindowExtensionJob {
    let provider = match item.market {
        ProviderMarket::KoreanEquity if config.allow_krx => Some(ProviderKind::KrxOpenApi),
        ProviderMarket::KoreanEquity if config.allow_data_go_kr => {
            Some(ProviderKind::DataGoKrFscStockPrice)
        }
        ProviderMarket::USEquity if config.allow_alpha_vantage => Some(ProviderKind::AlphaVantage),
        ProviderMarket::USEquity if config.allow_alpaca => Some(ProviderKind::Alpaca),
        ProviderMarket::Crypto if config.allow_upbit_crypto => Some(ProviderKind::Upbit),
        _ => None,
    };

    let (job_kind, provider_kind) = match provider {
        Some(ProviderKind::KrxOpenApi) => (
            FutureWindowExtensionJobKind::KrxEodFutureWindowCollect,
            Some(ProviderKind::KrxOpenApi),
        ),
        Some(ProviderKind::DataGoKrFscStockPrice) => (
            FutureWindowExtensionJobKind::DataGoKrEodFutureWindowCollect,
            Some(ProviderKind::DataGoKrFscStockPrice),
        ),
        Some(ProviderKind::AlphaVantage) => (
            FutureWindowExtensionJobKind::AlphaVantageCompactFutureWindowCollect,
            Some(ProviderKind::AlphaVantage),
        ),
        Some(ProviderKind::Alpaca) => (
            FutureWindowExtensionJobKind::AlpacaHistoricalFutureWindowCollect,
            Some(ProviderKind::Alpaca),
        ),
        Some(ProviderKind::Upbit) => (
            FutureWindowExtensionJobKind::UpbitCryptoFutureWindowCollect,
            Some(ProviderKind::Upbit),
        ),
        _ => (
            FutureWindowExtensionJobKind::SkippedUnsupportedProvider,
            None,
        ),
    };

    let mut job_kind = job_kind;
    let mut status = if config.run_provider_collection_jobs {
        FutureWindowExtensionJobStatus::ReadyToRun
    } else {
        FutureWindowExtensionJobStatus::Planned
    };

    if !config.generate_provider_jobs {
        job_kind = FutureWindowExtensionJobKind::SkippedUnsupportedProvider;
        status = FutureWindowExtensionJobStatus::Skipped;
    }

    if provider_kind == Some(ProviderKind::KrxOpenApi)
        && std::env::var("KRX_APPROVED")
            .unwrap_or_default()
            .to_ascii_lowercase()
            != "true"
    {
        reason_codes.push(ReasonCode::MissingApproval);
        job_kind = FutureWindowExtensionJobKind::SkippedMissingApproval;
        status = FutureWindowExtensionJobStatus::Skipped;
    } else if provider_kind == Some(ProviderKind::KrxOpenApi)
        && std::env::var("KRX_ENDPOINT_TEMPLATE")
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        reason_codes.push(ReasonCode::MissingEndpointTemplate);
        job_kind = FutureWindowExtensionJobKind::SkippedMissingEndpointTemplate;
        status = FutureWindowExtensionJobStatus::Skipped;
    } else if missing_auth(provider_kind) {
        reason_codes.push(ReasonCode::MissingAuth);
        job_kind = FutureWindowExtensionJobKind::SkippedMissingAuth;
        status = FutureWindowExtensionJobStatus::Skipped;
    }

    FutureWindowExtensionJob {
        job_id: format!("{}-provider", item.row_id),
        job_kind,
        provider_kind,
        market,
        venue: item.venue.clone(),
        symbol: item.symbol.clone(),
        timeframe: item.timeframe.clone(),
        horizon_bars: item.horizon_bars,
        required_start_timestamp_ms: item.required_start_timestamp_ms,
        required_end_timestamp_ms: item.required_end_timestamp_ms,
        max_rows: config.max_rows_per_job,
        max_requests: config.max_requests_per_job,
        expected_output_csv: Some(
            PathBuf::from(&config.output_root)
                .join(format!(
                    "{}_{}_extended.csv",
                    item.symbol.to_ascii_lowercase(),
                    item.timeframe
                ))
                .display()
                .to_string(),
        ),
        expected_provenance: Some(
            PathBuf::from(&config.output_root)
                .join(format!(
                    "{}_{}_extended_provenance.json",
                    item.symbol.to_ascii_lowercase(),
                    item.timeframe
                ))
                .display()
                .to_string(),
        ),
        expected_preflight: Some(
            PathBuf::from(&config.output_root)
                .join(format!(
                    "{}_{}_extended_preflight.json",
                    item.symbol.to_ascii_lowercase(),
                    item.timeframe
                ))
                .display()
                .to_string(),
        ),
        status,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn best_local_descriptor(
    item: &FutureWindowRequirementItem,
    descriptors: &BTreeMap<String, OfficialCandleSeriesDescriptor>,
) -> Option<OfficialCandleSeriesDescriptor> {
    descriptors
        .values()
        .filter(|descriptor| {
            normalize_symbol(&descriptor.symbol) == normalize_symbol(&item.symbol)
                && descriptor.market == item.market
                && descriptor.timeframe == item.timeframe
        })
        .cloned()
        .max_by(|left, right| {
            left.timestamp_end_ms
                .cmp(&right.timestamp_end_ms)
                .then(left.row_count.cmp(&right.row_count))
                .then(left.candle_series_id.cmp(&right.candle_series_id))
        })
}

fn collect_operator_actions_for_job(
    job: &FutureWindowExtensionJob,
    operator_actions: &mut BTreeSet<String>,
) {
    match job.job_kind {
        FutureWindowExtensionJobKind::SkippedMissingApproval => {
            operator_actions.insert("WaitForKrxApproval".to_string());
        }
        FutureWindowExtensionJobKind::SkippedMissingEndpointTemplate => {
            operator_actions.insert("SetKrxEndpointTemplate".to_string());
        }
        FutureWindowExtensionJobKind::SkippedMissingAuth => match job.provider_kind {
            Some(ProviderKind::AlphaVantage) => {
                operator_actions.insert("SetAlphaVantageApiKey".to_string());
            }
            Some(ProviderKind::Alpaca) => {
                operator_actions.insert("SetAlpacaApiKeys".to_string());
            }
            Some(ProviderKind::KrxOpenApi) => {
                operator_actions.insert("SetKrxApiKey".to_string());
            }
            Some(ProviderKind::DataGoKrFscStockPrice) => {
                operator_actions.insert("SetDataGoKrServiceKey".to_string());
            }
            Some(ProviderKind::Upbit) => {
                operator_actions.insert("SetUpbitAccessKey".to_string());
            }
            _ => {}
        },
        FutureWindowExtensionJobKind::SkippedMissingProvenance => {
            operator_actions.insert("ProvideOfficialProvenance".to_string());
        }
        FutureWindowExtensionJobKind::SkippedMissingPreflight => {
            operator_actions.insert("RunDataPreflight".to_string());
        }
        _ => {}
    }
}

fn missing_auth(provider_kind: Option<ProviderKind>) -> bool {
    match provider_kind {
        Some(ProviderKind::AlphaVantage) => std::env::var("ALPHAVANTAGE_API_KEY")
            .unwrap_or_default()
            .trim()
            .is_empty(),
        Some(ProviderKind::Alpaca) => {
            std::env::var("ALPACA_API_KEY")
                .unwrap_or_default()
                .trim()
                .is_empty()
                || std::env::var("ALPACA_SECRET_KEY")
                    .unwrap_or_default()
                    .trim()
                    .is_empty()
        }
        Some(ProviderKind::KrxOpenApi) => std::env::var("KRX_API_KEY")
            .unwrap_or_default()
            .trim()
            .is_empty(),
        Some(ProviderKind::DataGoKrFscStockPrice) => std::env::var("DATA_GO_KR_SERVICE_KEY")
            .unwrap_or_default()
            .trim()
            .is_empty(),
        Some(ProviderKind::Upbit) => std::env::var("UPBIT_ACCESS_KEY")
            .unwrap_or_default()
            .trim()
            .is_empty(),
        _ => false,
    }
}

#[derive(Clone, Debug, Default)]
struct Sidecars {
    provenance: Option<String>,
    preflight: Option<String>,
}

fn sidecars_for_csv(path: &Path) -> Sidecars {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    let provenance = path
        .with_file_name(format!("{stem}_provenance.json"))
        .exists()
        .then(|| {
            path.with_file_name(format!("{stem}_provenance.json"))
                .display()
                .to_string()
        });
    let preflight = path
        .with_file_name(format!("{stem}_preflight.json"))
        .exists()
        .then(|| {
            path.with_file_name(format!("{stem}_preflight.json"))
                .display()
                .to_string()
        });
    Sidecars {
        provenance,
        preflight,
    }
}

fn default_output_root() -> String {
    "target/soma_future_window_extension".to_string()
}

fn default_max_jobs() -> usize {
    10
}

fn default_max_symbols() -> usize {
    5
}

fn default_max_rows_per_job() -> usize {
    500
}

fn default_max_requests_per_job() -> usize {
    10
}

fn default_max_total_bytes() -> usize {
    5_000_000
}

fn default_true() -> bool {
    true
}
