use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::{ProviderKind, ProviderMarket};

use super::OperatorActionPriority;
use super::krx_collection_smoke::{
    KRXBoundedCollectionSmokeConfig, KRXCollectionDryRunReport, KRXCollectionDryRunStatus,
};
use super::krx_evidence_job::KRXStorageBudgetSummary;
use super::krx_operator_actions::{KRXOperatorAction, KRXOperatorActionKind};
use super::krx_symbol_whitelist::{KRXSymbolEntry, KRXSymbolWhitelist};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXCollectionBatchJobKind {
    FixtureReplay,
    LocalCanonicalCsvImport,
    ExistingCollectedCsvReuse,
    KrxEodCollectDryRun,
    KrxEodCollectLive,
    SkippedMissingApiKey,
    SkippedMissingEndpointTemplate,
    SkippedBudgetExceeded,
    SkippedInvalidSymbol,
    SkippedAllSymbolDenied,
    SkippedFullHistoryDenied,
    SkippedLiveCollectionDisabled,
    SkippedUnsupportedSchema,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXCollectionBatchJobStatus {
    Planned,
    ReadyToRun,
    RanSuccessfully,
    Skipped,
    Failed,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KRXCollectionBatchJob {
    pub job_id: String,
    pub job_kind: KRXCollectionBatchJobKind,
    pub provider_kind: ProviderKind,
    pub market: ProviderMarket,
    pub venue: String,
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
    #[serde(default)]
    pub expected_raw_archive_path: Option<String>,
    #[serde(default)]
    pub expected_canonical_csv_path: Option<String>,
    #[serde(default)]
    pub expected_provenance_path: Option<String>,
    #[serde(default)]
    pub expected_preflight_path: Option<String>,
    #[serde(default)]
    pub expected_manifest_path: Option<String>,
    pub status: KRXCollectionBatchJobStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KRXCollectionBatchPlan {
    pub batch_id: String,
    pub jobs: Vec<KRXCollectionBatchJob>,
    pub runnable_jobs: Vec<String>,
    pub skipped_jobs: Vec<String>,
    pub dry_run_jobs: Vec<String>,
    pub live_collection_jobs: Vec<String>,
    pub fixture_replay_jobs: Vec<String>,
    pub local_import_jobs: Vec<String>,
    pub operator_actions: Vec<KRXOperatorAction>,
    pub storage_budget_summary: KRXStorageBudgetSummary,
    pub reason_codes: Vec<ReasonCode>,
}

impl KRXCollectionBatchPlan {
    pub fn build(
        config: &KRXBoundedCollectionSmokeConfig,
        dry_run: &KRXCollectionDryRunReport,
        whitelist: &KRXSymbolWhitelist,
    ) -> Self {
        let storage_budget_summary = build_storage_budget_summary(config, whitelist);
        let mut jobs = Vec::new();
        for entry in whitelist.entries.iter().filter(|entry| entry.enabled) {
            let fixture_path = match_symbol_path(
                &config.local_fixture_response_paths,
                &entry.provider_symbol,
                &entry.normalized_symbol,
            );
            let canonical_path = match_symbol_path(
                &config.local_canonical_csv_paths,
                &entry.provider_symbol,
                &entry.normalized_symbol,
            );
            if !entry.is_valid() {
                jobs.push(base_job(
                    config,
                    entry,
                    KRXCollectionBatchJobKind::SkippedInvalidSymbol,
                    KRXCollectionBatchJobStatus::Skipped,
                    None,
                    canonical_path,
                    vec![ReasonCode::InvalidSymbol],
                ));
                continue;
            }
            if whitelist.enabled_entries.len() > config.max_symbols {
                jobs.push(base_job(
                    config,
                    entry,
                    KRXCollectionBatchJobKind::SkippedAllSymbolDenied,
                    KRXCollectionBatchJobStatus::Skipped,
                    None,
                    canonical_path,
                    vec![ReasonCode::DeniedByDefault],
                ));
                continue;
            }
            if !storage_budget_summary.budget_ok {
                jobs.push(base_job(
                    config,
                    entry,
                    KRXCollectionBatchJobKind::SkippedBudgetExceeded,
                    KRXCollectionBatchJobStatus::Skipped,
                    fixture_path,
                    canonical_path,
                    vec![ReasonCode::BudgetExceeded],
                ));
                continue;
            }
            if config.run_fixture_replay {
                if let Some(path) = fixture_path.clone() {
                    jobs.push(base_job(
                        config,
                        entry,
                        KRXCollectionBatchJobKind::FixtureReplay,
                        KRXCollectionBatchJobStatus::ReadyToRun,
                        Some(path),
                        canonical_path.clone(),
                        vec![ReasonCode::MockFixtureLoaded, ReasonCode::LocalFileOnly],
                    ));
                }
            }
            if config.run_local_import {
                if let Some(path) = canonical_path.clone() {
                    let kind = if path.contains("collected") || path.contains("imported") {
                        KRXCollectionBatchJobKind::ExistingCollectedCsvReuse
                    } else {
                        KRXCollectionBatchJobKind::LocalCanonicalCsvImport
                    };
                    jobs.push(base_job(
                        config,
                        entry,
                        kind,
                        KRXCollectionBatchJobStatus::ReadyToRun,
                        fixture_path.clone(),
                        Some(path),
                        vec![
                            ReasonCode::KRXLocalImportPreferred,
                            ReasonCode::LocalFileOnly,
                        ],
                    ));
                }
            }
            if config.run_dry_run {
                let status = if matches!(
                    dry_run.dry_run_status,
                    KRXCollectionDryRunStatus::BudgetExceeded
                ) {
                    KRXCollectionBatchJobStatus::Skipped
                } else {
                    KRXCollectionBatchJobStatus::DiagnosticOnly
                };
                let mut reasons = vec![ReasonCode::ProviderRequestPlanned];
                if status == KRXCollectionBatchJobStatus::Skipped {
                    reasons.push(ReasonCode::BudgetExceeded);
                }
                jobs.push(base_job(
                    config,
                    entry,
                    KRXCollectionBatchJobKind::KrxEodCollectDryRun,
                    status,
                    fixture_path.clone(),
                    canonical_path.clone(),
                    reasons,
                ));
            }
            let (live_kind, live_status, live_reasons) = if !config.run_live_collection {
                (
                    KRXCollectionBatchJobKind::SkippedLiveCollectionDisabled,
                    KRXCollectionBatchJobStatus::Skipped,
                    vec![ReasonCode::KRXCollectionDisabledByDefault],
                )
            } else if config.max_days > 365 {
                (
                    KRXCollectionBatchJobKind::SkippedFullHistoryDenied,
                    KRXCollectionBatchJobStatus::Skipped,
                    vec![ReasonCode::FullHistoryDenied],
                )
            } else if matches!(
                dry_run.dry_run_status,
                KRXCollectionDryRunStatus::MissingApiKey
            ) || matches!(
                dry_run.dry_run_status,
                KRXCollectionDryRunStatus::MissingApiKeyAndEndpointTemplate
            ) {
                (
                    KRXCollectionBatchJobKind::SkippedMissingApiKey,
                    KRXCollectionBatchJobStatus::Skipped,
                    vec![ReasonCode::MissingApiKey],
                )
            } else if matches!(
                dry_run.dry_run_status,
                KRXCollectionDryRunStatus::MissingEndpointTemplate
            ) {
                (
                    KRXCollectionBatchJobKind::SkippedMissingEndpointTemplate,
                    KRXCollectionBatchJobStatus::Skipped,
                    vec![ReasonCode::MissingEndpointTemplate],
                )
            } else if matches!(
                dry_run.dry_run_status,
                KRXCollectionDryRunStatus::ScopeTooBroad
            ) {
                (
                    KRXCollectionBatchJobKind::SkippedAllSymbolDenied,
                    KRXCollectionBatchJobStatus::Skipped,
                    vec![ReasonCode::DeniedByDefault],
                )
            } else if matches!(
                dry_run.dry_run_status,
                KRXCollectionDryRunStatus::BudgetExceeded
            ) {
                (
                    KRXCollectionBatchJobKind::SkippedBudgetExceeded,
                    KRXCollectionBatchJobStatus::Skipped,
                    vec![ReasonCode::BudgetExceeded],
                )
            } else {
                (
                    KRXCollectionBatchJobKind::KrxEodCollectLive,
                    KRXCollectionBatchJobStatus::ReadyToRun,
                    vec![
                        ReasonCode::KRXProviderConfigured,
                        ReasonCode::ProviderRequestPlanned,
                    ],
                )
            };
            jobs.push(base_job(
                config,
                entry,
                live_kind,
                live_status,
                fixture_path,
                canonical_path,
                live_reasons,
            ));
        }
        jobs.sort_by(|left, right| left.job_id.cmp(&right.job_id));
        let runnable_jobs = jobs
            .iter()
            .filter(|job| {
                matches!(
                    job.status,
                    KRXCollectionBatchJobStatus::ReadyToRun | KRXCollectionBatchJobStatus::Planned
                )
            })
            .map(|job| job.job_id.clone())
            .collect::<Vec<_>>();
        let skipped_jobs = jobs
            .iter()
            .filter(|job| {
                matches!(
                    job.status,
                    KRXCollectionBatchJobStatus::Skipped
                        | KRXCollectionBatchJobStatus::DiagnosticOnly
                )
            })
            .map(|job| job.job_id.clone())
            .collect::<Vec<_>>();
        let dry_run_jobs = collect_ids(&jobs, |job| {
            matches!(job.job_kind, KRXCollectionBatchJobKind::KrxEodCollectDryRun)
        });
        let live_collection_jobs = collect_ids(&jobs, |job| {
            matches!(job.job_kind, KRXCollectionBatchJobKind::KrxEodCollectLive)
        });
        let fixture_replay_jobs = collect_ids(&jobs, |job| {
            matches!(job.job_kind, KRXCollectionBatchJobKind::FixtureReplay)
        });
        let local_import_jobs = collect_ids(&jobs, |job| {
            matches!(
                job.job_kind,
                KRXCollectionBatchJobKind::LocalCanonicalCsvImport
                    | KRXCollectionBatchJobKind::ExistingCollectedCsvReuse
            )
        });
        let operator_actions =
            build_operator_actions(config, dry_run, whitelist, &storage_budget_summary);
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
            batch_id: format!("{}-collection-batch", config.smoke_id),
            jobs,
            runnable_jobs,
            skipped_jobs,
            dry_run_jobs,
            live_collection_jobs,
            fixture_replay_jobs,
            local_import_jobs,
            operator_actions,
            storage_budget_summary,
            reason_codes,
        }
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("batch_id={}", self.batch_id),
            format!("runnable_jobs={}", self.runnable_jobs.join("|")),
            format!("skipped_jobs={}", self.skipped_jobs.join("|")),
            format!("dry_run_jobs={}", self.dry_run_jobs.join("|")),
            format!(
                "live_collection_jobs={}",
                self.live_collection_jobs.join("|")
            ),
            format!("fixture_replay_jobs={}", self.fixture_replay_jobs.join("|")),
            format!("local_import_jobs={}", self.local_import_jobs.join("|")),
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
        lines.extend(self.jobs.iter().map(KRXCollectionBatchJob::to_text));
        lines
            .into_iter()
            .chain(
                self.operator_actions
                    .iter()
                    .map(|action| format!("operator_action={}", action.to_text())),
            )
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("krx_collection_batch_plan.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_collection_batch_plan.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

impl KRXCollectionBatchJob {
    pub fn to_text(&self) -> String {
        format!(
            "job_id={};job_kind={:?};status={:?};provider_symbol={};normalized_symbol={};timeframe={};max_rows={};max_requests={};max_days={};expected_raw_archive_path={};expected_canonical_csv_path={};expected_provenance_path={};expected_preflight_path={};expected_manifest_path={};reason_codes={}",
            self.job_id,
            self.job_kind,
            self.status,
            self.provider_symbol,
            self.normalized_symbol,
            self.timeframe,
            self.max_rows,
            self.max_requests,
            self.max_days,
            self.expected_raw_archive_path.clone().unwrap_or_default(),
            self.expected_canonical_csv_path.clone().unwrap_or_default(),
            self.expected_provenance_path.clone().unwrap_or_default(),
            self.expected_preflight_path.clone().unwrap_or_default(),
            self.expected_manifest_path.clone().unwrap_or_default(),
            self.reason_codes
                .iter()
                .map(|reason| format!("{reason:?}"))
                .collect::<Vec<_>>()
                .join("|")
        )
    }
}

fn base_job(
    config: &KRXBoundedCollectionSmokeConfig,
    entry: &KRXSymbolEntry,
    job_kind: KRXCollectionBatchJobKind,
    status: KRXCollectionBatchJobStatus,
    expected_raw_archive_path: Option<String>,
    expected_canonical_csv_path: Option<String>,
    reason_codes: Vec<ReasonCode>,
) -> KRXCollectionBatchJob {
    let artifact_root = config.output_dir().join("artifacts");
    let symbol_slug = entry.normalized_symbol.to_ascii_lowercase();
    KRXCollectionBatchJob {
        job_id: format!(
            "{}-{}",
            symbol_slug,
            match job_kind {
                KRXCollectionBatchJobKind::FixtureReplay => "fixture-replay",
                KRXCollectionBatchJobKind::LocalCanonicalCsvImport => "local-import",
                KRXCollectionBatchJobKind::ExistingCollectedCsvReuse => "reuse",
                KRXCollectionBatchJobKind::KrxEodCollectDryRun => "dry-run",
                KRXCollectionBatchJobKind::KrxEodCollectLive => "live-collect",
                KRXCollectionBatchJobKind::SkippedMissingApiKey => "skip-missing-api-key",
                KRXCollectionBatchJobKind::SkippedMissingEndpointTemplate =>
                    "skip-missing-endpoint",
                KRXCollectionBatchJobKind::SkippedBudgetExceeded => "skip-budget",
                KRXCollectionBatchJobKind::SkippedInvalidSymbol => "skip-invalid-symbol",
                KRXCollectionBatchJobKind::SkippedAllSymbolDenied => "skip-all-symbol",
                KRXCollectionBatchJobKind::SkippedFullHistoryDenied => "skip-full-history",
                KRXCollectionBatchJobKind::SkippedLiveCollectionDisabled => "skip-live-disabled",
                KRXCollectionBatchJobKind::SkippedUnsupportedSchema => "skip-schema",
                KRXCollectionBatchJobKind::DiagnosticOnly => "diagnostic-only",
            }
        ),
        job_kind,
        provider_kind: ProviderKind::KrxOpenApi,
        market: ProviderMarket::KoreanEquity,
        venue: "KRX".to_string(),
        provider_symbol: entry.provider_symbol.clone(),
        normalized_symbol: entry.normalized_symbol.clone(),
        timeframe: "1d".to_string(),
        start_date: None,
        end_date: None,
        max_rows: entry.max_rows.unwrap_or(config.max_rows_per_symbol),
        max_requests: config.max_requests,
        max_days: config.max_days,
        expected_raw_archive_path: expected_raw_archive_path.or_else(|| {
            Some(
                artifact_root
                    .join("raw")
                    .join(format!("{}_response.json", symbol_slug))
                    .display()
                    .to_string(),
            )
        }),
        expected_canonical_csv_path: expected_canonical_csv_path.or_else(|| {
            Some(
                artifact_root
                    .join("canonical")
                    .join(format!("{}_1d.csv", symbol_slug))
                    .display()
                    .to_string(),
            )
        }),
        expected_provenance_path: Some(
            artifact_root
                .join("provenance")
                .join(format!("{}_1d_provenance.json", symbol_slug))
                .display()
                .to_string(),
        ),
        expected_preflight_path: Some(
            artifact_root
                .join("preflight")
                .join(format!("{}_1d_preflight.json", symbol_slug))
                .display()
                .to_string(),
        ),
        expected_manifest_path: Some(
            artifact_root
                .join("manifest")
                .join(format!("{}_1d_manifest.json", symbol_slug))
                .display()
                .to_string(),
        ),
        status,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn build_storage_budget_summary(
    config: &KRXBoundedCollectionSmokeConfig,
    whitelist: &KRXSymbolWhitelist,
) -> KRXStorageBudgetSummary {
    let estimated_bytes = whitelist
        .entries
        .iter()
        .filter(|entry| entry.enabled)
        .map(|entry| entry.max_rows.unwrap_or(config.max_rows_per_symbol) * 168)
        .sum::<usize>();
    let budget_ok = estimated_bytes <= config.max_total_bytes
        && estimated_bytes <= config.max_raw_bytes + config.max_canonical_bytes;
    let mut reason_codes = vec![ReasonCode::CollectionBudgetReportBuilt];
    if budget_ok {
        reason_codes.push(ReasonCode::StorageBudgetReportBuilt);
    } else {
        reason_codes.push(ReasonCode::BudgetExceeded);
    }
    KRXStorageBudgetSummary {
        estimated_bytes,
        max_bytes: config.max_total_bytes,
        budget_ok,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn match_symbol_path(
    paths: &[String],
    provider_symbol: &str,
    normalized_symbol: &str,
) -> Option<String> {
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

fn collect_ids(
    jobs: &[KRXCollectionBatchJob],
    predicate: impl Fn(&KRXCollectionBatchJob) -> bool,
) -> Vec<String> {
    jobs.iter()
        .filter(|job| predicate(job))
        .map(|job| job.job_id.clone())
        .collect()
}

fn build_operator_actions(
    config: &KRXBoundedCollectionSmokeConfig,
    dry_run: &KRXCollectionDryRunReport,
    whitelist: &KRXSymbolWhitelist,
    storage_budget_summary: &KRXStorageBudgetSummary,
) -> Vec<KRXOperatorAction> {
    let mut actions = Vec::new();
    if matches!(
        dry_run.dry_run_status,
        KRXCollectionDryRunStatus::MissingApiKey
            | KRXCollectionDryRunStatus::MissingApiKeyAndEndpointTemplate
    ) {
        actions.push(action(
            "set-krx-api-key",
            KRXOperatorActionKind::SetKRXApiKey,
            OperatorActionPriority::Required,
            vec![super::krx_auth_readiness::KRX_API_KEY_ENV_VAR.to_string()],
            "Set KRX_API_KEY in the local environment and rerun the dry run.",
            Some(
                "cargo run --quiet --bin soma_experiment -- krx-collection-dry-run --config examples/soma_krx_collection_dry_run.toml".to_string(),
            ),
            vec![ReasonCode::MissingApiKey, ReasonCode::KRXOperatorActionPlanBuilt],
        ));
    }
    if matches!(
        dry_run.dry_run_status,
        KRXCollectionDryRunStatus::MissingEndpointTemplate
            | KRXCollectionDryRunStatus::MissingApiKeyAndEndpointTemplate
    ) {
        actions.push(action(
            "set-krx-endpoint-template",
            KRXOperatorActionKind::SetKRXEndpointTemplate,
            OperatorActionPriority::Required,
            vec![super::krx_auth_readiness::KRX_ENDPOINT_TEMPLATE_ENV_VAR.to_string()],
            "Set KRX_ENDPOINT_TEMPLATE locally; reports will keep the preview redacted.",
            Some(
                "cargo run --quiet --bin soma_experiment -- krx-collection-dry-run --config examples/soma_krx_collection_dry_run.toml".to_string(),
            ),
            vec![ReasonCode::MissingEndpointTemplate, ReasonCode::KRXOperatorActionPlanBuilt],
        ));
    }
    if whitelist.enabled_entries.len() > config.max_symbols {
        actions.push(action(
            "reduce-krx-scope",
            KRXOperatorActionKind::ReduceScope,
            OperatorActionPriority::Required,
            Vec::new(),
            "Reduce the KRX symbol whitelist to the compact bounded set before collection.",
            Some(
                "cargo run --quiet --bin soma_experiment -- krx-collection-plan --config examples/soma_krx_collection_plan_missing_auth.toml".to_string(),
            ),
            vec![ReasonCode::DeniedByDefault, ReasonCode::KRXOperatorActionPlanBuilt],
        ));
    }
    if !storage_budget_summary.budget_ok {
        actions.push(action(
            "reduce-krx-storage-budget",
            KRXOperatorActionKind::ReduceScope,
            OperatorActionPriority::Required,
            Vec::new(),
            "Reduce symbol count, row count, or raw retention so the bounded storage budget is respected.",
            None,
            vec![ReasonCode::BudgetExceeded, ReasonCode::KRXOperatorActionPlanBuilt],
        ));
    }
    if !config.run_live_collection {
        actions.push(action(
            "live-collection-remains-disabled",
            KRXOperatorActionKind::RunKRXOfficialAcquire,
            OperatorActionPriority::Recommended,
            Vec::new(),
            "Live KRX collection is disabled by default; keep using fixture replay or local import unless an operator explicitly enables bounded live collection.",
            Some(
                "cargo run --quiet --bin soma_experiment -- krx-collection-close --config examples/soma_krx_collection_close_operator_live_template.toml".to_string(),
            ),
            vec![ReasonCode::KRXCollectionDisabledByDefault, ReasonCode::KRXOperatorActionPlanBuilt],
        ));
    }
    actions.sort_by(|left, right| left.action_id.cmp(&right.action_id));
    actions
}

fn action(
    action_id: &str,
    action_kind: KRXOperatorActionKind,
    priority: OperatorActionPriority,
    env_var_names: Vec<String>,
    description: &str,
    command_suggestion: Option<String>,
    reason_codes: Vec<ReasonCode>,
) -> KRXOperatorAction {
    KRXOperatorAction {
        action_id: action_id.to_string(),
        action_kind,
        priority,
        env_var_names,
        description: description.to_string(),
        command_suggestion,
        expected_output_artifact: None,
        safe_to_run: true,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}
