use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::{ProviderKind, ProviderMarket};

use super::krx_auth_readiness::{KRXAuthReadinessReport, KRXAuthReadinessStatus};
use super::krx_official_activation::KRXOfficialEvidenceActivationConfig;
use super::krx_operator_actions::{KRXOperatorAction, build_krx_operator_actions};
use super::krx_symbol_whitelist::KRXSymbolWhitelist;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXEvidenceJobKind {
    LocalCanonicalCsvImport,
    ExistingCollectedCsvReuse,
    KrxEodCollect,
    SkippedMissingApiKey,
    SkippedMissingEndpointTemplate,
    SkippedMissingProvenance,
    SkippedMissingPreflight,
    SkippedBudgetExceeded,
    SkippedInvalidSymbol,
    SkippedAllSymbolDenied,
    SkippedFullHistoryDenied,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXEvidenceJobStatus {
    Planned,
    ReadyToRun,
    RanSuccessfully,
    Skipped,
    Failed,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KRXEvidenceJob {
    pub job_id: String,
    pub job_kind: KRXEvidenceJobKind,
    pub provider_kind: ProviderKind,
    pub market: ProviderMarket,
    pub provider_symbol: String,
    pub normalized_symbol: String,
    pub timeframe: String,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    pub max_rows: usize,
    pub max_requests: usize,
    pub max_days: usize,
    pub output_root: String,
    #[serde(default)]
    pub expected_canonical_csv: Option<String>,
    #[serde(default)]
    pub expected_provenance: Option<String>,
    #[serde(default)]
    pub expected_preflight: Option<String>,
    #[serde(default)]
    pub expected_manifest: Option<String>,
    pub status: KRXEvidenceJobStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KRXStorageBudgetSummary {
    pub estimated_bytes: usize,
    pub max_bytes: usize,
    pub budget_ok: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KRXEvidenceJobPlan {
    pub plan_id: String,
    pub jobs: Vec<KRXEvidenceJob>,
    pub runnable_jobs: Vec<String>,
    pub skipped_jobs: Vec<String>,
    pub local_import_jobs: Vec<String>,
    pub collection_jobs: Vec<String>,
    pub operator_actions: Vec<KRXOperatorAction>,
    pub storage_budget_summary: KRXStorageBudgetSummary,
    pub reason_codes: Vec<ReasonCode>,
}

impl KRXEvidenceJobPlan {
    pub fn build(
        config: &KRXOfficialEvidenceActivationConfig,
        auth: &KRXAuthReadinessReport,
        whitelist: &KRXSymbolWhitelist,
    ) -> Self {
        let storage_budget_summary = build_storage_budget_summary(config, whitelist);
        let mut jobs = Vec::new();
        let scope_denied = whitelist.enabled_entries.len() > config.max_symbols;
        for entry in whitelist.entries.iter().filter(|entry| entry.enabled) {
            let invalid_symbol = !entry.is_valid();
            let canonical = match_path(
                &config.local_krx_canonical_csv_paths,
                &entry.provider_symbol,
                &entry.normalized_symbol,
            );
            let provenance = match_path(
                &config.local_krx_provenance_paths,
                &entry.provider_symbol,
                &entry.normalized_symbol,
            );
            let preflight = match_path(
                &config.local_krx_preflight_paths,
                &entry.provider_symbol,
                &entry.normalized_symbol,
            );
            let output_root = config.output_dir().join("jobs").display().to_string();
            if invalid_symbol {
                jobs.push(base_job(
                    config,
                    entry,
                    KRXEvidenceJobKind::SkippedInvalidSymbol,
                    KRXEvidenceJobStatus::Skipped,
                    output_root,
                    canonical,
                    provenance,
                    preflight,
                    vec![ReasonCode::InvalidSymbol],
                ));
                continue;
            }
            if scope_denied {
                jobs.push(base_job(
                    config,
                    entry,
                    KRXEvidenceJobKind::SkippedAllSymbolDenied,
                    KRXEvidenceJobStatus::Skipped,
                    output_root,
                    canonical,
                    provenance,
                    preflight,
                    vec![ReasonCode::DeniedByDefault],
                ));
                continue;
            }
            if !storage_budget_summary.budget_ok {
                jobs.push(base_job(
                    config,
                    entry,
                    KRXEvidenceJobKind::SkippedBudgetExceeded,
                    KRXEvidenceJobStatus::Skipped,
                    output_root,
                    canonical,
                    provenance,
                    preflight,
                    vec![ReasonCode::BudgetExceeded],
                ));
                continue;
            }
            if let Some(canonical_path) = canonical.clone() {
                let job_kind = if canonical_path.contains("official_collection")
                    || canonical_path.contains("collected")
                {
                    KRXEvidenceJobKind::ExistingCollectedCsvReuse
                } else {
                    KRXEvidenceJobKind::LocalCanonicalCsvImport
                };
                let status = if config.run_local_import {
                    KRXEvidenceJobStatus::ReadyToRun
                } else {
                    KRXEvidenceJobStatus::Planned
                };
                jobs.push(base_job(
                    config,
                    entry,
                    job_kind,
                    status,
                    output_root,
                    Some(canonical_path),
                    provenance,
                    preflight,
                    vec![
                        ReasonCode::KRXEvidenceJobPlanBuilt,
                        ReasonCode::LocalFileOnly,
                    ],
                ));
                continue;
            }
            let (job_kind, status, reasons) = if !config.run_krx_collection {
                (
                    KRXEvidenceJobKind::DiagnosticOnly,
                    KRXEvidenceJobStatus::DiagnosticOnly,
                    vec![ReasonCode::KRXCollectionDisabledByDefault],
                )
            } else if config.max_days > 365 {
                (
                    KRXEvidenceJobKind::SkippedFullHistoryDenied,
                    KRXEvidenceJobStatus::Skipped,
                    vec![ReasonCode::FullHistoryDenied],
                )
            } else if matches!(
                auth.readiness_status,
                KRXAuthReadinessStatus::MissingApiKey
                    | KRXAuthReadinessStatus::MissingApiKeyAndEndpointTemplate
            ) {
                (
                    KRXEvidenceJobKind::SkippedMissingApiKey,
                    KRXEvidenceJobStatus::Skipped,
                    vec![ReasonCode::MissingApiKey],
                )
            } else if matches!(
                auth.readiness_status,
                KRXAuthReadinessStatus::MissingEndpointTemplate
            ) {
                (
                    KRXEvidenceJobKind::SkippedMissingEndpointTemplate,
                    KRXEvidenceJobStatus::Skipped,
                    vec![ReasonCode::MissingEndpointTemplate],
                )
            } else {
                (
                    KRXEvidenceJobKind::KrxEodCollect,
                    KRXEvidenceJobStatus::Planned,
                    vec![
                        ReasonCode::KRXProviderConfigured,
                        ReasonCode::ProviderRequestPlanned,
                    ],
                )
            };
            jobs.push(base_job(
                config,
                entry,
                job_kind,
                status,
                output_root,
                None,
                provenance,
                preflight,
                reasons,
            ));
        }
        jobs.sort_by(|left, right| left.job_id.cmp(&right.job_id));
        let runnable_jobs = jobs
            .iter()
            .filter(|job| {
                matches!(
                    job.status,
                    KRXEvidenceJobStatus::ReadyToRun | KRXEvidenceJobStatus::Planned
                )
            })
            .map(|job| job.job_id.clone())
            .collect::<Vec<_>>();
        let skipped_jobs = jobs
            .iter()
            .filter(|job| {
                matches!(
                    job.status,
                    KRXEvidenceJobStatus::Skipped | KRXEvidenceJobStatus::DiagnosticOnly
                )
            })
            .map(|job| job.job_id.clone())
            .collect::<Vec<_>>();
        let local_import_jobs = jobs
            .iter()
            .filter(|job| {
                matches!(
                    job.job_kind,
                    KRXEvidenceJobKind::LocalCanonicalCsvImport
                        | KRXEvidenceJobKind::ExistingCollectedCsvReuse
                )
            })
            .map(|job| job.job_id.clone())
            .collect::<Vec<_>>();
        let collection_jobs = jobs
            .iter()
            .filter(|job| matches!(job.job_kind, KRXEvidenceJobKind::KrxEodCollect))
            .map(|job| job.job_id.clone())
            .collect::<Vec<_>>();
        let operator_actions = build_krx_operator_actions(
            config,
            auth,
            whitelist,
            &[],
            !storage_budget_summary.budget_ok,
        );
        let reason_codes = stable_reason_codes(
            &[
                vec![ReasonCode::KRXEvidenceJobPlanBuilt],
                storage_budget_summary.reason_codes.clone(),
                jobs.iter()
                    .flat_map(|job| job.reason_codes.clone())
                    .collect::<Vec<_>>(),
            ]
            .concat(),
        );
        Self {
            plan_id: format!("{}-job-plan", config.activation_id),
            jobs,
            runnable_jobs,
            skipped_jobs,
            local_import_jobs,
            collection_jobs,
            operator_actions,
            storage_budget_summary,
            reason_codes,
        }
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("plan_id={}", self.plan_id),
            format!("runnable_jobs={}", self.runnable_jobs.join("|")),
            format!("skipped_jobs={}", self.skipped_jobs.join("|")),
            format!("local_import_jobs={}", self.local_import_jobs.join("|")),
            format!("collection_jobs={}", self.collection_jobs.join("|")),
            format!(
                "storage_budget_summary=estimated_bytes:{};max_bytes:{};budget_ok:{}",
                self.storage_budget_summary.estimated_bytes,
                self.storage_budget_summary.max_bytes,
                self.storage_budget_summary.budget_ok
            ),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ];
        lines.extend(self.jobs.iter().map(|job| {
            format!(
                "job_id={};job_kind={:?};status={:?};provider_symbol={};normalized_symbol={};max_rows={};max_requests={};max_days={};expected_canonical_csv={};expected_provenance={};expected_preflight={};expected_manifest={};reason_codes={}",
                job.job_id,
                job.job_kind,
                job.status,
                job.provider_symbol,
                job.normalized_symbol,
                job.max_rows,
                job.max_requests,
                job.max_days,
                job.expected_canonical_csv.clone().unwrap_or_default(),
                job.expected_provenance.clone().unwrap_or_default(),
                job.expected_preflight.clone().unwrap_or_default(),
                job.expected_manifest.clone().unwrap_or_default(),
                job.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            )
        }));
        lines.join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("krx_evidence_job_plan.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_evidence_job_plan.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

fn base_job(
    config: &KRXOfficialEvidenceActivationConfig,
    entry: &super::krx_symbol_whitelist::KRXSymbolEntry,
    job_kind: KRXEvidenceJobKind,
    status: KRXEvidenceJobStatus,
    output_root: String,
    expected_canonical_csv: Option<String>,
    expected_provenance: Option<String>,
    expected_preflight: Option<String>,
    reason_codes: Vec<ReasonCode>,
) -> KRXEvidenceJob {
    KRXEvidenceJob {
        job_id: format!(
            "{}-{}",
            entry.normalized_symbol.to_ascii_lowercase(),
            match job_kind {
                KRXEvidenceJobKind::LocalCanonicalCsvImport => "local-import",
                KRXEvidenceJobKind::ExistingCollectedCsvReuse => "reuse",
                KRXEvidenceJobKind::KrxEodCollect => "collect",
                KRXEvidenceJobKind::SkippedMissingApiKey => "skip-missing-api-key",
                KRXEvidenceJobKind::SkippedMissingEndpointTemplate => "skip-missing-endpoint",
                KRXEvidenceJobKind::SkippedMissingProvenance => "skip-missing-provenance",
                KRXEvidenceJobKind::SkippedMissingPreflight => "skip-missing-preflight",
                KRXEvidenceJobKind::SkippedBudgetExceeded => "skip-budget",
                KRXEvidenceJobKind::SkippedInvalidSymbol => "skip-invalid-symbol",
                KRXEvidenceJobKind::SkippedAllSymbolDenied => "skip-all-symbol",
                KRXEvidenceJobKind::SkippedFullHistoryDenied => "skip-full-history",
                KRXEvidenceJobKind::DiagnosticOnly => "diagnostic-only",
            }
        ),
        job_kind,
        provider_kind: ProviderKind::KrxOpenApi,
        market: ProviderMarket::KoreanEquity,
        provider_symbol: entry.provider_symbol.clone(),
        normalized_symbol: entry.normalized_symbol.clone(),
        timeframe: "1d".to_string(),
        start_date: None,
        end_date: None,
        max_rows: entry.max_rows.unwrap_or(config.max_rows_per_symbol),
        max_requests: config.max_requests,
        max_days: config.max_days,
        output_root,
        expected_canonical_csv,
        expected_provenance,
        expected_preflight,
        expected_manifest: Some(
            config
                .output_dir()
                .join("manifests")
                .join(format!(
                    "{}_manifest.txt",
                    entry.normalized_symbol.to_ascii_lowercase()
                ))
                .display()
                .to_string(),
        ),
        status,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn build_storage_budget_summary(
    config: &KRXOfficialEvidenceActivationConfig,
    whitelist: &KRXSymbolWhitelist,
) -> KRXStorageBudgetSummary {
    let estimated_bytes = whitelist
        .entries
        .iter()
        .filter(|entry| entry.enabled)
        .map(|entry| entry.max_rows.unwrap_or(config.max_rows_per_symbol) * 96)
        .sum::<usize>();
    let budget_ok = estimated_bytes <= config.max_bytes && config.max_days <= 365;
    let mut reason_codes = vec![ReasonCode::CollectionBudgetReportBuilt];
    if !budget_ok {
        reason_codes.push(ReasonCode::BudgetExceeded);
    }
    KRXStorageBudgetSummary {
        estimated_bytes,
        max_bytes: config.max_bytes,
        budget_ok,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn match_path(paths: &[String], provider_symbol: &str, normalized_symbol: &str) -> Option<String> {
    let provider_symbol = provider_symbol.to_ascii_lowercase();
    let normalized_symbol = normalized_symbol.to_ascii_lowercase();
    paths.iter().find_map(|path| {
        let lower = path.to_ascii_lowercase();
        if lower.contains(&provider_symbol) || lower.contains(&normalized_symbol) {
            Some(path.clone())
        } else {
            None
        }
    })
}
