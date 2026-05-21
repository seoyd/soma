use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::{
    ProviderDataSubject, ProviderEntitlementStatusKind, ProviderKind, ProviderMarket,
    StorageBudgetReport,
};
use crate::experiment::{
    OfficialProviderReadinessConfig, OfficialProviderReadinessReport,
    OfficialProviderReadinessRunner, ProviderRealityConfig, ProviderRealityReport,
    ProviderRealityRunner,
};

use super::candle_acquisition_job::{
    CandleAcquisitionJob, CandleAcquisitionJobKind, CandleAcquisitionJobStatus,
    CandleAcquisitionPlan,
};
use super::candle_expansion_operator_actions::build_candle_expansion_operator_actions;
use super::official_candle_gap_map::{
    OfficialCandleCoverageGapMap, OfficialCandleGapKind, load_gap_map_from_path_or_config,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialCandleExpansionPlanConfig {
    pub plan_id: String,
    #[serde(default)]
    pub gap_config_path: Option<String>,
    #[serde(default)]
    pub gap_map_path: Option<String>,
    #[serde(default)]
    pub provider_readiness_report_paths: Vec<String>,
    #[serde(default)]
    pub provider_reality_report_paths: Vec<String>,
    #[serde(default)]
    pub official_acquisition_config_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub run_provider_readiness: bool,
    #[serde(default)]
    pub run_provider_reality: bool,
    #[serde(default = "default_true")]
    pub generate_collection_jobs: bool,
    #[serde(default = "default_true")]
    pub generate_import_jobs: bool,
    #[serde(default)]
    pub run_collection_jobs: bool,
    #[serde(default = "default_true")]
    pub run_import_jobs: bool,
    #[serde(default = "default_max_jobs")]
    pub max_jobs: usize,
    #[serde(default = "default_max_symbols_per_job")]
    pub max_symbols_per_job: usize,
    #[serde(default = "default_max_rows_per_job")]
    pub max_rows_per_job: usize,
    #[serde(default = "default_max_requests_per_job")]
    pub max_requests_per_job: usize,
    #[serde(default = "default_max_total_bytes")]
    pub max_total_bytes: usize,
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
    #[serde(default = "default_true")]
    pub allow_local_import: bool,
    #[serde(default)]
    pub run_reference_generation: bool,
    #[serde(default)]
    pub run_counterfactual_depth_close: bool,
    #[serde(default)]
    pub run_core_scorecard_rerun: bool,
    #[serde(default)]
    pub counterfactual_depth_closure_config_path: Option<String>,
    #[serde(default)]
    pub core_performance_config_path: Option<String>,
    #[serde(default)]
    pub previous_core_scorecard_path: Option<String>,
    #[serde(default)]
    pub reference_pack_config_paths: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default)]
struct ProviderState {
    auth_ready: bool,
    endpoint_ready: bool,
    approval_ready: bool,
}

impl Default for OfficialCandleExpansionPlanConfig {
    fn default() -> Self {
        Self {
            plan_id: "official-candle-expansion-plan".to_string(),
            gap_config_path: None,
            gap_map_path: None,
            provider_readiness_report_paths: Vec::new(),
            provider_reality_report_paths: Vec::new(),
            official_acquisition_config_paths: Vec::new(),
            output_root: default_output_root(),
            run_provider_readiness: false,
            run_provider_reality: false,
            generate_collection_jobs: true,
            generate_import_jobs: true,
            run_collection_jobs: false,
            run_import_jobs: true,
            max_jobs: default_max_jobs(),
            max_symbols_per_job: default_max_symbols_per_job(),
            max_rows_per_job: default_max_rows_per_job(),
            max_requests_per_job: default_max_requests_per_job(),
            max_total_bytes: default_max_total_bytes(),
            allow_krx: true,
            allow_data_go_kr: true,
            allow_alpha_vantage: true,
            allow_alpaca: true,
            allow_upbit_crypto: true,
            allow_local_import: true,
            run_reference_generation: false,
            run_counterfactual_depth_close: false,
            run_core_scorecard_rerun: false,
            counterfactual_depth_closure_config_path: None,
            core_performance_config_path: None,
            previous_core_scorecard_path: None,
            reference_pack_config_paths: Vec::new(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialCandleExpansionPlanConfig {
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
        if self.plan_id.trim().is_empty() {
            return Err("official candle expansion plan id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("official candle expansion plan paths must be local".to_string());
        }
        if self.max_jobs == 0 || self.max_jobs > default_max_jobs() {
            return Err("official candle expansion max_jobs must be between 1 and 10".to_string());
        }
        if self.max_symbols_per_job == 0 || self.max_symbols_per_job > default_max_symbols_per_job()
        {
            return Err(
                "official candle expansion max_symbols_per_job must be between 1 and 5".to_string(),
            );
        }
        if self.max_rows_per_job == 0 || self.max_rows_per_job > default_max_rows_per_job() {
            return Err(
                "official candle expansion max_rows_per_job must be between 1 and 500".to_string(),
            );
        }
        if self.max_requests_per_job == 0
            || self.max_requests_per_job > default_max_requests_per_job()
        {
            return Err(
                "official candle expansion max_requests_per_job must be between 1 and 10"
                    .to_string(),
            );
        }
        if self.max_total_bytes == 0 || self.max_total_bytes > default_max_total_bytes() {
            return Err(
                "official candle expansion max_total_bytes must be between 1 and 5000000"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.plan_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.gap_config_path
            .iter()
            .cloned()
            .chain(self.gap_map_path.iter().cloned())
            .chain(self.provider_readiness_report_paths.iter().cloned())
            .chain(self.provider_reality_report_paths.iter().cloned())
            .chain(self.official_acquisition_config_paths.iter().cloned())
            .chain(
                self.counterfactual_depth_closure_config_path
                    .iter()
                    .cloned(),
            )
            .chain(self.core_performance_config_path.iter().cloned())
            .chain(self.previous_core_scorecard_path.iter().cloned())
            .chain(self.reference_pack_config_paths.iter().cloned())
            .collect()
    }
}

pub fn build_official_candle_acquisition_plan(
    config: &OfficialCandleExpansionPlanConfig,
) -> Result<CandleAcquisitionPlan, String> {
    config.validate()?;
    let gap_map = load_gap_map(config)?;
    let readiness_reports = load_provider_readiness_reports(config, &gap_map)?;
    let reality_reports = load_provider_reality_reports(config)?;
    let mut warnings = Vec::new();
    let mut jobs = Vec::new();
    let mut estimated_bytes = 0usize;

    for (index, cell) in gap_map.cells.iter().enumerate() {
        if index >= config.max_jobs {
            warnings.push(format!(
                "truncated_jobs={};max_jobs={}",
                gap_map.cells.len().saturating_sub(config.max_jobs),
                config.max_jobs
            ));
            break;
        }
        let mut job = build_job_for_cell(config, cell, &readiness_reports, &reality_reports);
        let job_estimate = estimate_job_bytes(&job);
        if estimated_bytes.saturating_add(job_estimate) > config.max_total_bytes {
            job.job_kind = CandleAcquisitionJobKind::SkippedBudgetExceeded;
            job.status = CandleAcquisitionJobStatus::Skipped;
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
            estimated_bytes = estimated_bytes.saturating_add(job_estimate);
        }
        jobs.push(job);
    }

    let storage_budget_summary = build_storage_summary(
        config,
        estimated_bytes,
        jobs.len(),
        jobs.iter()
            .any(|job| job.job_kind == CandleAcquisitionJobKind::SkippedBudgetExceeded),
    );
    let operator_actions = build_candle_expansion_operator_actions(&gap_map, &jobs);
    Ok(CandleAcquisitionPlan::from_jobs(
        config.plan_id.clone(),
        jobs,
        operator_actions,
        storage_budget_summary,
        warnings,
        config
            .reason_codes
            .iter()
            .cloned()
            .chain([
                ReasonCode::OfficialEvidenceAcquisitionRan,
                ReasonCode::DeterministicPath,
            ])
            .collect(),
    ))
}

pub fn load_gap_map(
    config: &OfficialCandleExpansionPlanConfig,
) -> Result<OfficialCandleCoverageGapMap, String> {
    if let Some(path) = config.gap_map_path.as_deref() {
        return load_gap_map_from_path_or_config(path);
    }
    if let Some(path) = config.gap_config_path.as_deref() {
        return load_gap_map_from_path_or_config(path);
    }
    Err("official candle expansion plan requires gap_config_path or gap_map_path".to_string())
}

fn build_job_for_cell(
    config: &OfficialCandleExpansionPlanConfig,
    cell: &super::official_candle_gap_map::OfficialCandleGapCell,
    readiness_reports: &[OfficialProviderReadinessReport],
    reality_reports: &[ProviderRealityReport],
) -> CandleAcquisitionJob {
    match cell.source_class {
        super::ComparableEvidenceSourceClass::OfficialNonCrypto => {
            if config.generate_import_jobs && config.allow_local_import {
                if let Some(job) = build_local_job(config, cell) {
                    return job;
                }
            }
            if !config.generate_collection_jobs {
                return skipped_job(
                    config,
                    cell,
                    CandleAcquisitionJobKind::SkippedUnsupportedProvider,
                    None,
                    vec![ReasonCode::DeniedByDefault],
                );
            }
            build_official_collection_job(config, cell, readiness_reports, reality_reports)
        }
        super::ComparableEvidenceSourceClass::OfficialCryptoOnly => {
            if config.generate_import_jobs
                && config.allow_local_import
                && cell.buildable_from_existing_local_csv
            {
                if let Some(job) = build_local_job(config, cell) {
                    return job;
                }
            }
            if !config.generate_collection_jobs {
                return skipped_job(
                    config,
                    cell,
                    CandleAcquisitionJobKind::SkippedUnsupportedProvider,
                    Some(ProviderKind::Upbit),
                    vec![ReasonCode::DeniedByDefault, ReasonCode::CryptoOnlyEvidence],
                );
            }
            if !config.allow_upbit_crypto {
                return skipped_job(
                    config,
                    cell,
                    CandleAcquisitionJobKind::SkippedUnsupportedProvider,
                    Some(ProviderKind::Upbit),
                    vec![ReasonCode::CryptoOnlyEvidence],
                );
            }
            build_job(
                config,
                cell,
                CandleAcquisitionJobKind::UpbitCryptoCandleCollect,
                Some(ProviderKind::Upbit),
                CandleAcquisitionJobStatus::DiagnosticOnly,
                None,
                None,
                None,
                vec![ReasonCode::CryptoOnlyEvidence],
            )
        }
        super::ComparableEvidenceSourceClass::ControlledDiagnostic => {
            if config.generate_import_jobs
                && config.allow_local_import
                && cell.buildable_from_existing_local_csv
            {
                if let Some(job) = build_local_job(config, cell) {
                    return job;
                }
            }
            build_job(
                config,
                cell,
                CandleAcquisitionJobKind::DiagnosticControlledImport,
                None,
                CandleAcquisitionJobStatus::DiagnosticOnly,
                None,
                None,
                None,
                vec![ReasonCode::ControlledOnlyEvidence],
            )
        }
        super::ComparableEvidenceSourceClass::YFinanceResearch
        | super::ComparableEvidenceSourceClass::FixtureArchitectureTest
        | super::ComparableEvidenceSourceClass::SyntheticTest
        | super::ComparableEvidenceSourceClass::Unknown => skipped_job(
            config,
            cell,
            CandleAcquisitionJobKind::SkippedSourceNotEligible,
            None,
            vec![ReasonCode::ReadinessEvidenceExcluded],
        ),
    }
}

fn build_local_job(
    config: &OfficialCandleExpansionPlanConfig,
    cell: &super::official_candle_gap_map::OfficialCandleGapCell,
) -> Option<CandleAcquisitionJob> {
    let csv_path = first_existing(cell, &[".csv"]);
    let provenance = sidecar_for(cell, &csv_path, "_provenance.json");
    let preflight = sidecar_for(cell, &csv_path, "_preflight.json");
    let manifest = sidecar_for(cell, &csv_path, "_manifest.json");

    if csv_path.is_none() {
        if cell.requires_operator_action {
            return Some(build_job(
                config,
                cell,
                CandleAcquisitionJobKind::LocalOfficialCsvImport,
                None,
                CandleAcquisitionJobStatus::Skipped,
                None,
                None,
                None,
                vec![ReasonCode::MissingOfficialData],
            ));
        }
        return None;
    }

    let status = if cell
        .gap_kinds
        .contains(&OfficialCandleGapKind::ControlledOnlySource)
        || cell
            .gap_kinds
            .contains(&OfficialCandleGapKind::ResearchOnlySource)
        || cell
            .gap_kinds
            .contains(&OfficialCandleGapKind::FixtureOnlySource)
        || cell
            .gap_kinds
            .contains(&OfficialCandleGapKind::CryptoOnlySource)
        || matches!(
            cell.source_class,
            super::ComparableEvidenceSourceClass::ControlledDiagnostic
                | super::ComparableEvidenceSourceClass::OfficialCryptoOnly
                | super::ComparableEvidenceSourceClass::YFinanceResearch
                | super::ComparableEvidenceSourceClass::FixtureArchitectureTest
                | super::ComparableEvidenceSourceClass::SyntheticTest
                | super::ComparableEvidenceSourceClass::Unknown
        )
        || provenance.is_none()
        || preflight.is_none()
    {
        CandleAcquisitionJobStatus::DiagnosticOnly
    } else {
        CandleAcquisitionJobStatus::ReadyToRun
    };

    Some(build_job(
        config,
        cell,
        CandleAcquisitionJobKind::ExistingCanonicalCsvReuse,
        None,
        status,
        csv_path,
        provenance,
        preflight.or_else(|| manifest.clone()),
        vec![ReasonCode::DeterministicPath],
    ))
}

fn build_official_collection_job(
    config: &OfficialCandleExpansionPlanConfig,
    cell: &super::official_candle_gap_map::OfficialCandleGapCell,
    readiness_reports: &[OfficialProviderReadinessReport],
    reality_reports: &[ProviderRealityReport],
) -> CandleAcquisitionJob {
    match cell.market {
        ProviderMarket::KoreanEquity => {
            if config.allow_krx {
                let provider_state =
                    provider_state(ProviderKind::KrxOpenApi, readiness_reports, reality_reports);
                return if !provider_state.approval_ready {
                    skipped_job(
                        config,
                        cell,
                        CandleAcquisitionJobKind::SkippedMissingApproval,
                        Some(ProviderKind::KrxOpenApi),
                        vec![ReasonCode::MissingApproval, ReasonCode::KrxApprovalPending],
                    )
                } else if !provider_state.auth_ready {
                    skipped_job(
                        config,
                        cell,
                        CandleAcquisitionJobKind::SkippedMissingAuth,
                        Some(ProviderKind::KrxOpenApi),
                        vec![ReasonCode::MissingAuth],
                    )
                } else if !provider_state.endpoint_ready {
                    skipped_job(
                        config,
                        cell,
                        CandleAcquisitionJobKind::SkippedMissingEndpointTemplate,
                        Some(ProviderKind::KrxOpenApi),
                        vec![ReasonCode::MissingEndpointTemplate],
                    )
                } else {
                    build_job(
                        config,
                        cell,
                        CandleAcquisitionJobKind::KrxEodCollect,
                        Some(ProviderKind::KrxOpenApi),
                        CandleAcquisitionJobStatus::ReadyToRun,
                        None,
                        None,
                        None,
                        vec![ReasonCode::DeterministicPath],
                    )
                };
            }
            if config.allow_data_go_kr {
                let provider_state = provider_state(
                    ProviderKind::DataGoKrFscStockPrice,
                    readiness_reports,
                    reality_reports,
                );
                return if !provider_state.auth_ready {
                    skipped_job(
                        config,
                        cell,
                        CandleAcquisitionJobKind::SkippedMissingAuth,
                        Some(ProviderKind::DataGoKrFscStockPrice),
                        vec![ReasonCode::MissingAuth],
                    )
                } else {
                    build_job(
                        config,
                        cell,
                        CandleAcquisitionJobKind::DataGoKrEodCollect,
                        Some(ProviderKind::DataGoKrFscStockPrice),
                        CandleAcquisitionJobStatus::ReadyToRun,
                        None,
                        None,
                        None,
                        vec![ReasonCode::DeterministicPath],
                    )
                };
            }
            skipped_job(
                config,
                cell,
                CandleAcquisitionJobKind::SkippedUnsupportedProvider,
                None,
                vec![ReasonCode::MissingOfficialData],
            )
        }
        ProviderMarket::USEquity => {
            if config.allow_alpha_vantage {
                let provider_state = provider_state(
                    ProviderKind::AlphaVantage,
                    readiness_reports,
                    reality_reports,
                );
                return if !provider_state.auth_ready {
                    skipped_job(
                        config,
                        cell,
                        CandleAcquisitionJobKind::SkippedMissingAuth,
                        Some(ProviderKind::AlphaVantage),
                        vec![ReasonCode::MissingAuth],
                    )
                } else {
                    build_job(
                        config,
                        cell,
                        CandleAcquisitionJobKind::AlphaVantageCompactDailyCollect,
                        Some(ProviderKind::AlphaVantage),
                        CandleAcquisitionJobStatus::ReadyToRun,
                        None,
                        None,
                        None,
                        vec![ReasonCode::AlphaVantageEodOnly],
                    )
                };
            }
            if config.allow_alpaca {
                let provider_state =
                    provider_state(ProviderKind::Alpaca, readiness_reports, reality_reports);
                return if !provider_state.auth_ready {
                    skipped_job(
                        config,
                        cell,
                        CandleAcquisitionJobKind::SkippedMissingAuth,
                        Some(ProviderKind::Alpaca),
                        vec![ReasonCode::MissingAuth],
                    )
                } else {
                    build_job(
                        config,
                        cell,
                        CandleAcquisitionJobKind::AlpacaHistoricalBarsCollect,
                        Some(ProviderKind::Alpaca),
                        CandleAcquisitionJobStatus::ReadyToRun,
                        None,
                        None,
                        None,
                        vec![ReasonCode::AlpacaIexLimited],
                    )
                };
            }
            skipped_job(
                config,
                cell,
                CandleAcquisitionJobKind::SkippedUnsupportedProvider,
                None,
                vec![ReasonCode::MissingOfficialData],
            )
        }
        ProviderMarket::Crypto => build_job(
            config,
            cell,
            CandleAcquisitionJobKind::UpbitCryptoCandleCollect,
            Some(ProviderKind::Upbit),
            CandleAcquisitionJobStatus::DiagnosticOnly,
            None,
            None,
            None,
            vec![ReasonCode::CryptoOnlyEvidence],
        ),
        _ => skipped_job(
            config,
            cell,
            CandleAcquisitionJobKind::SkippedUnsupportedProvider,
            None,
            vec![ReasonCode::MissingOfficialData],
        ),
    }
}

fn build_job(
    config: &OfficialCandleExpansionPlanConfig,
    cell: &super::official_candle_gap_map::OfficialCandleGapCell,
    job_kind: CandleAcquisitionJobKind,
    provider_kind: Option<ProviderKind>,
    status: CandleAcquisitionJobStatus,
    local_input_csv_path: Option<String>,
    local_input_provenance_path: Option<String>,
    local_input_preflight_path: Option<String>,
    reason_codes: Vec<ReasonCode>,
) -> CandleAcquisitionJob {
    let job_output_dir = config.output_dir().join("jobs").join(format!(
        "{}-{}-{}",
        normalize_symbol(&cell.symbol),
        cell.timeframe,
        cell.horizon_bars
    ));
    let output_stem = local_input_csv_path
        .as_deref()
        .and_then(|path| {
            Path::new(path)
                .file_stem()
                .and_then(|value| value.to_str())
                .map(|value| value.to_string())
        })
        .unwrap_or_else(|| {
            format!(
                "{}_{}",
                normalize_symbol(&cell.symbol).to_ascii_lowercase(),
                cell.timeframe
            )
        });
    CandleAcquisitionJob {
        job_id: format!(
            "{}-{}-{}-{}",
            config.plan_id,
            normalize_symbol(&cell.symbol),
            cell.timeframe,
            cell.horizon_bars
        ),
        job_kind,
        provider_kind,
        market: format!("{:?}", cell.market),
        symbol: cell.symbol.clone(),
        venue: cell.venue.clone(),
        timeframe: cell.timeframe.clone(),
        horizon_bars: cell.horizon_bars,
        start_timestamp_ms: cell.required_start_timestamp_ms,
        end_timestamp_ms: cell.required_end_timestamp_ms,
        max_rows: config.max_rows_per_job.min(cell.required_min_rows.max(1)),
        max_requests: config.max_requests_per_job,
        output_root: job_output_dir.display().to_string(),
        local_input_csv_path: local_input_csv_path.clone(),
        local_input_provenance_path: local_input_provenance_path.clone(),
        local_input_preflight_path: local_input_preflight_path.clone(),
        local_input_manifest_path: local_input_csv_path
            .as_deref()
            .and_then(|path| discover_sidecar(path, "_manifest.json")),
        expected_canonical_csv_path: Some(
            job_output_dir
                .join(format!("{output_stem}.csv"))
                .display()
                .to_string(),
        ),
        expected_provenance_path: Some(
            job_output_dir
                .join(format!("{output_stem}_provenance.json"))
                .display()
                .to_string(),
        ),
        expected_preflight_path: Some(
            job_output_dir
                .join(format!("{output_stem}_preflight.json"))
                .display()
                .to_string(),
        ),
        status,
        reason_codes: stable_reason_codes(
            &reason_codes
                .into_iter()
                .chain([ReasonCode::DeterministicPath])
                .collect::<Vec<_>>(),
        ),
    }
}

fn skipped_job(
    config: &OfficialCandleExpansionPlanConfig,
    cell: &super::official_candle_gap_map::OfficialCandleGapCell,
    job_kind: CandleAcquisitionJobKind,
    provider_kind: Option<ProviderKind>,
    reason_codes: Vec<ReasonCode>,
) -> CandleAcquisitionJob {
    build_job(
        config,
        cell,
        job_kind,
        provider_kind,
        CandleAcquisitionJobStatus::Skipped,
        None,
        None,
        None,
        reason_codes,
    )
}

fn provider_state(
    provider_kind: ProviderKind,
    readiness_reports: &[OfficialProviderReadinessReport],
    reality_reports: &[ProviderRealityReport],
) -> ProviderState {
    let env_auth_ready = provider_auth_ready(provider_kind);
    let env_endpoint_ready = provider_endpoint_ready(provider_kind);
    let env_approval_ready = provider_approval_ready(provider_kind);
    let readiness_missing_auth = readiness_reports.iter().any(|report| {
        report
            .selection_results
            .iter()
            .any(|result| result.missing_auth_providers.contains(&provider_kind))
    });
    let reality_status = reality_reports
        .iter()
        .flat_map(|report| report.entitlement_statuses.iter())
        .find(|status| status.provider_subject == ProviderDataSubject::Provider(provider_kind));
    ProviderState {
        auth_ready: env_auth_ready
            && !readiness_missing_auth
            && reality_status
                .map(|status| status.status != ProviderEntitlementStatusKind::MissingAuth)
                .unwrap_or(true),
        endpoint_ready: env_endpoint_ready
            && reality_status
                .map(|status| {
                    status.status != ProviderEntitlementStatusKind::MissingEndpointTemplate
                })
                .unwrap_or(true),
        approval_ready: match provider_kind {
            ProviderKind::KrxOpenApi => {
                env_approval_ready
                    && reality_status
                        .map(|status| {
                            status.status != ProviderEntitlementStatusKind::MissingApproval
                        })
                        .unwrap_or(true)
            }
            _ => reality_status
                .map(|status| status.status != ProviderEntitlementStatusKind::MissingApproval)
                .unwrap_or(true),
        },
    }
}

fn load_provider_readiness_reports(
    config: &OfficialCandleExpansionPlanConfig,
    gap_map: &OfficialCandleCoverageGapMap,
) -> Result<Vec<OfficialProviderReadinessReport>, String> {
    let mut reports = config
        .provider_readiness_report_paths
        .iter()
        .map(|path| {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            serde_json::from_str::<OfficialProviderReadinessReport>(&text)
                .map_err(|err| err.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if config.run_provider_readiness {
        let mut readiness = OfficialProviderReadinessConfig::default();
        readiness.report_id = format!("{}-provider-readiness", config.plan_id);
        readiness.output_dir = config
            .output_dir()
            .join("provider_readiness")
            .display()
            .to_string();
        readiness.markets = gap_map
            .cells
            .iter()
            .map(|cell| cell.market)
            .collect::<Vec<_>>();
        readiness.markets.sort();
        readiness.markets.dedup();
        if readiness.markets.is_empty() {
            readiness.markets = vec![
                ProviderMarket::KoreanEquity,
                ProviderMarket::USEquity,
                ProviderMarket::Crypto,
            ];
        }
        let report = OfficialProviderReadinessRunner::default().run(&readiness);
        let _ = report.write_to_dir(Path::new(&readiness.output_dir));
        reports.push(report);
    }
    Ok(reports)
}

fn load_provider_reality_reports(
    config: &OfficialCandleExpansionPlanConfig,
) -> Result<Vec<ProviderRealityReport>, String> {
    let mut reports = config
        .provider_reality_report_paths
        .iter()
        .map(|path| ProviderRealityReport::from_json_path(Path::new(path)))
        .collect::<Result<Vec<_>, _>>()?;
    if config.run_provider_reality {
        let mut reality = ProviderRealityConfig::default();
        reality.report_id = format!("{}-provider-reality", config.plan_id);
        reality.output_dir = config
            .output_dir()
            .join("provider_reality")
            .display()
            .to_string();
        let report = ProviderRealityRunner::default().run(&reality)?;
        let _ = report.write_to_dir(Path::new(&reality.output_dir));
        reports.push(report);
    }
    Ok(reports)
}

fn build_storage_summary(
    config: &OfficialCandleExpansionPlanConfig,
    estimated_bytes: usize,
    file_count: usize,
    budget_exceeded: bool,
) -> StorageBudgetReport {
    StorageBudgetReport {
        total_bytes: estimated_bytes,
        raw_bytes: 0,
        canonical_bytes: estimated_bytes,
        manifest_bytes: file_count.saturating_mul(512),
        compressed_bytes: 0,
        uncompressed_bytes_estimate: estimated_bytes,
        file_count,
        budget_exceeded: budget_exceeded || estimated_bytes > config.max_total_bytes,
        compression_applied: false,
        retention_actions: Vec::new(),
        skipped_files: Vec::new(),
        reason_codes: stable_reason_codes(&[
            ReasonCode::StorageBudgetReportBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

fn estimate_job_bytes(job: &CandleAcquisitionJob) -> usize {
    job.max_rows.saturating_mul(64).saturating_add(2048)
}

fn first_existing(
    cell: &super::official_candle_gap_map::OfficialCandleGapCell,
    suffixes: &[&str],
) -> Option<String> {
    cell.related_artifact_paths.iter().find_map(|path| {
        let matches_suffix = suffixes.iter().any(|suffix| path.ends_with(suffix));
        if matches_suffix && Path::new(path).exists() {
            Some(path.clone())
        } else {
            None
        }
    })
}

fn sidecar_for(
    cell: &super::official_candle_gap_map::OfficialCandleGapCell,
    csv_path: &Option<String>,
    suffix: &str,
) -> Option<String> {
    cell.related_artifact_paths
        .iter()
        .find(|path| path.ends_with(suffix) && Path::new(path).exists())
        .cloned()
        .or_else(|| {
            csv_path
                .as_deref()
                .and_then(|path| discover_sidecar(path, suffix))
        })
}

fn discover_sidecar(csv_path: &str, suffix: &str) -> Option<String> {
    let path = Path::new(csv_path);
    let stem = path.file_stem()?.to_string_lossy();
    let sidecar = path.parent()?.join(format!("{stem}{suffix}"));
    sidecar.exists().then(|| sidecar.display().to_string())
}

fn provider_auth_ready(provider_kind: ProviderKind) -> bool {
    match provider_kind {
        ProviderKind::KrxOpenApi => env_present("KRX_API_KEY"),
        ProviderKind::DataGoKrFscStockPrice => {
            env_present("DATA_GO_KR_SERVICE_KEY") || env_present("DATAGOKR_SERVICE_KEY")
        }
        ProviderKind::AlphaVantage => env_present("ALPHAVANTAGE_API_KEY"),
        ProviderKind::Alpaca => env_present("ALPACA_API_KEY") && env_present("ALPACA_SECRET_KEY"),
        ProviderKind::Upbit => true,
        _ => false,
    }
}

fn provider_endpoint_ready(provider_kind: ProviderKind) -> bool {
    match provider_kind {
        ProviderKind::KrxOpenApi => env_present("KRX_ENDPOINT_TEMPLATE"),
        _ => true,
    }
}

fn provider_approval_ready(provider_kind: ProviderKind) -> bool {
    match provider_kind {
        ProviderKind::KrxOpenApi => truthy_env("KRX_APPROVAL_READY") || truthy_env("KRX_APPROVED"),
        _ => true,
    }
}

fn env_present(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn truthy_env(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "ready" | "approved"
            )
        })
        .unwrap_or(false)
}

fn normalize_symbol(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_uppercase()
}

fn default_output_root() -> String {
    "target/soma_official_candle_expansion".to_string()
}

fn default_max_jobs() -> usize {
    10
}

fn default_max_symbols_per_job() -> usize {
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
