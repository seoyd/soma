use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::ProviderKind;

use super::kis_auth_readiness::{KISAuthReadinessReport, KISAuthReadinessStatus};
use super::kis_endpoint_policy::{KISEndpointCategory, KISEndpointPolicy, KISEndpointPolicyReport};
use super::kis_market_data_activation::KISMarketDataActivationConfig;
use super::kis_operator_actions::{KISOperatorAction, build_kis_operator_actions};
use super::kis_symbol_whitelist::{
    KISDataFreshness, KISMarket, KISSymbolEntry, KISSymbolWhitelist,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISCollectionJobKind {
    FixtureReplayDomestic,
    FixtureReplayOverseas,
    LocalCanonicalCsvImport,
    ExistingCollectedCsvReuse,
    KISDomesticEodCollectDryRun,
    KISDomesticMinuteCollectDryRun,
    KISDomesticRealtimeQuoteDryRun,
    KISOverseasEodCollectDryRun,
    KISOverseasRealtimeQuoteDryRun,
    KISDomesticEodCollectLive,
    KISDomesticMinuteCollectLive,
    KISOverseasEodCollectLive,
    KISOverseasQuoteCollectLive,
    SkippedMissingAppKey,
    SkippedMissingAppSecret,
    SkippedMissingBaseUrl,
    SkippedMissingWebSocketApprovalKey,
    SkippedEndpointDenied,
    SkippedBudgetExceeded,
    SkippedInvalidSymbol,
    SkippedAllSymbolDenied,
    SkippedFullHistoryDenied,
    SkippedLiveCollectionDisabled,
    SkippedUnsupportedSchema,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISCollectionJobStatus {
    Planned,
    ReadyToRun,
    RanSuccessfully,
    Skipped,
    Failed,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISCollectionJob {
    pub job_id: String,
    pub job_kind: KISCollectionJobKind,
    pub provider_kind: ProviderKind,
    pub market: KISMarket,
    #[serde(default)]
    pub venue: Option<String>,
    pub provider_symbol: String,
    pub normalized_symbol: String,
    #[serde(default)]
    pub exchange_code: Option<String>,
    pub timeframe: String,
    pub freshness: KISDataFreshness,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    pub max_rows: usize,
    pub max_requests: usize,
    pub max_days: usize,
    pub endpoint_category: KISEndpointCategory,
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
    pub status: KISCollectionJobStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISStorageBudgetSummary {
    pub estimated_raw_archive_bytes: usize,
    pub estimated_canonical_csv_bytes: usize,
    pub estimated_total_bytes: usize,
    pub budget_ok: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KISCollectionBatchPlan {
    pub batch_id: String,
    pub jobs: Vec<KISCollectionJob>,
    pub runnable_jobs: Vec<String>,
    pub skipped_jobs: Vec<String>,
    pub dry_run_jobs: Vec<String>,
    pub live_collection_jobs: Vec<String>,
    pub fixture_replay_jobs: Vec<String>,
    pub local_import_jobs: Vec<String>,
    pub domestic_jobs: Vec<String>,
    pub overseas_jobs: Vec<String>,
    pub operator_actions: Vec<KISOperatorAction>,
    pub storage_budget_summary: KISStorageBudgetSummary,
    pub reason_codes: Vec<ReasonCode>,
}

impl KISCollectionBatchPlan {
    pub fn build(
        config: &KISMarketDataActivationConfig,
        auth: &KISAuthReadinessReport,
        endpoint_policy: &KISEndpointPolicy,
        endpoint_report: &KISEndpointPolicyReport,
        whitelist: &KISSymbolWhitelist,
    ) -> Self {
        let storage_budget_summary = build_storage_budget_summary(config, whitelist);
        let scope_exceeded = whitelist.domestic_count > config.max_domestic_symbols
            || whitelist.overseas_count > config.max_overseas_symbols
            || whitelist.enabled_entries.len()
                > config.max_domestic_symbols + config.max_overseas_symbols;
        let budget_exceeded = !storage_budget_summary.budget_ok;
        let mut jobs = Vec::new();
        for entry in whitelist.entries.iter().filter(|entry| entry.enabled) {
            let endpoint_category = endpoint_category_for_entry(entry);
            let fixture_path = infer_fixture_response_path(config, entry);
            let canonical_path = match_symbol_path(&config.local_kis_canonical_csv_paths, entry);
            if !entry.is_valid() {
                jobs.push(base_job(
                    config,
                    entry,
                    KISCollectionJobKind::SkippedInvalidSymbol,
                    KISCollectionJobStatus::Skipped,
                    endpoint_category,
                    None,
                    canonical_path,
                    vec![ReasonCode::InvalidSymbol],
                ));
                continue;
            }
            if scope_exceeded {
                jobs.push(base_job(
                    config,
                    entry,
                    KISCollectionJobKind::SkippedAllSymbolDenied,
                    KISCollectionJobStatus::Skipped,
                    endpoint_category,
                    fixture_path.clone(),
                    canonical_path,
                    vec![ReasonCode::DeniedByDefault, ReasonCode::BudgetExceeded],
                ));
                continue;
            }
            if budget_exceeded {
                jobs.push(base_job(
                    config,
                    entry,
                    KISCollectionJobKind::SkippedBudgetExceeded,
                    KISCollectionJobStatus::Skipped,
                    endpoint_category,
                    fixture_path.clone(),
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
                        if entry.market == KISMarket::KoreanEquity {
                            KISCollectionJobKind::FixtureReplayDomestic
                        } else {
                            KISCollectionJobKind::FixtureReplayOverseas
                        },
                        KISCollectionJobStatus::ReadyToRun,
                        endpoint_category,
                        Some(path),
                        canonical_path.clone(),
                        vec![ReasonCode::MockFixtureLoaded, ReasonCode::LocalFileOnly],
                    ));
                }
            }
            if config.run_local_import {
                if let Some(path) = canonical_path.clone() {
                    let kind = if path.contains("collected") || path.contains("imported") {
                        KISCollectionJobKind::ExistingCollectedCsvReuse
                    } else {
                        KISCollectionJobKind::LocalCanonicalCsvImport
                    };
                    jobs.push(base_job(
                        config,
                        entry,
                        kind,
                        KISCollectionJobStatus::ReadyToRun,
                        endpoint_category,
                        fixture_path.clone(),
                        Some(path),
                        vec![
                            ReasonCode::KISLocalImportPreferred,
                            ReasonCode::LocalFileOnly,
                        ],
                    ));
                }
            }
            if config.run_collection_dry_run {
                jobs.push(base_job(
                    config,
                    entry,
                    dry_run_kind(entry),
                    KISCollectionJobStatus::DiagnosticOnly,
                    endpoint_category,
                    fixture_path.clone(),
                    canonical_path.clone(),
                    vec![ReasonCode::ProviderRequestPlanned],
                ));
            }
            let (live_kind, live_status, live_reasons) = live_job_decision(
                config,
                auth,
                endpoint_policy,
                entry,
                endpoint_category,
                endpoint_report,
            );
            jobs.push(base_job(
                config,
                entry,
                live_kind,
                live_status,
                endpoint_category,
                fixture_path,
                canonical_path,
                live_reasons,
            ));
        }
        jobs.sort_by(|left, right| left.job_id.cmp(&right.job_id));
        let runnable_jobs = collect_ids(&jobs, |job| {
            matches!(
                job.status,
                KISCollectionJobStatus::ReadyToRun | KISCollectionJobStatus::Planned
            )
        });
        let skipped_jobs = collect_ids(&jobs, |job| {
            matches!(
                job.status,
                KISCollectionJobStatus::Skipped | KISCollectionJobStatus::DiagnosticOnly
            )
        });
        let dry_run_jobs = collect_ids(&jobs, |job| is_dry_run_kind(job.job_kind));
        let live_collection_jobs = collect_ids(&jobs, |job| {
            is_live_kind(job.job_kind) && job.status == KISCollectionJobStatus::ReadyToRun
        });
        let fixture_replay_jobs = collect_ids(&jobs, |job| {
            matches!(
                job.job_kind,
                KISCollectionJobKind::FixtureReplayDomestic
                    | KISCollectionJobKind::FixtureReplayOverseas
            )
        });
        let local_import_jobs = collect_ids(&jobs, |job| {
            matches!(
                job.job_kind,
                KISCollectionJobKind::LocalCanonicalCsvImport
                    | KISCollectionJobKind::ExistingCollectedCsvReuse
            )
        });
        let domestic_jobs = collect_ids(&jobs, |job| job.market == KISMarket::KoreanEquity);
        let overseas_jobs = collect_ids(&jobs, |job| job.market != KISMarket::KoreanEquity);
        let operator_actions = build_kis_operator_actions(
            config,
            auth,
            endpoint_report,
            whitelist,
            &[],
            budget_exceeded,
        );
        let reason_codes = stable_reason_codes(
            &[
                vec![ReasonCode::KISCollectionBatchPlanBuilt],
                storage_budget_summary.reason_codes.clone(),
                jobs.iter()
                    .flat_map(|job| job.reason_codes.clone())
                    .collect(),
            ]
            .concat(),
        );
        Self {
            batch_id: config.activation_id.clone(),
            jobs,
            runnable_jobs,
            skipped_jobs,
            dry_run_jobs,
            live_collection_jobs,
            fixture_replay_jobs,
            local_import_jobs,
            domestic_jobs,
            overseas_jobs,
            operator_actions,
            storage_budget_summary,
            reason_codes,
        }
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            "research_only_warning=kis collection plan is research-only, bounded, and market-data-only".to_string(),
            "secret_safety_warning=auth env-var names only; no secret values are rendered".to_string(),
            format!("batch_id={}", self.batch_id),
            format!("runnable_jobs={}", self.runnable_jobs.join("|")),
            format!("skipped_jobs={}", self.skipped_jobs.join("|")),
            format!("dry_run_jobs={}", self.dry_run_jobs.join("|")),
            format!("live_collection_jobs={}", self.live_collection_jobs.join("|")),
            format!("fixture_replay_jobs={}", self.fixture_replay_jobs.join("|")),
            format!("local_import_jobs={}", self.local_import_jobs.join("|")),
            format!("domestic_jobs={}", self.domestic_jobs.join("|")),
            format!("overseas_jobs={}", self.overseas_jobs.join("|")),
            format!("estimated_raw_archive_bytes={}", self.storage_budget_summary.estimated_raw_archive_bytes),
            format!("estimated_canonical_csv_bytes={}", self.storage_budget_summary.estimated_canonical_csv_bytes),
            format!("estimated_total_bytes={}", self.storage_budget_summary.estimated_total_bytes),
            format!("budget_ok={}", self.storage_budget_summary.budget_ok),
            format!(
                "reason_codes={}",
                self.reason_codes.iter().map(|reason| format!("{reason:?}")).collect::<Vec<_>>().join("|")
            ),
        ];
        lines.extend(self.jobs.iter().map(|job| {
            format!(
                "job_id={};job_kind={:?};market={:?};provider_symbol={};normalized_symbol={};exchange_code={};timeframe={};freshness={:?};endpoint_category={:?};status={:?};expected_raw_archive_path={};expected_canonical_csv_path={};reason_codes={}",
                job.job_id,
                job.job_kind,
                job.market,
                job.provider_symbol,
                job.normalized_symbol,
                job.exchange_code.clone().unwrap_or_default(),
                job.timeframe,
                job.freshness,
                job.endpoint_category,
                job.status,
                job.expected_raw_archive_path.clone().unwrap_or_default(),
                job.expected_canonical_csv_path.clone().unwrap_or_default(),
                job.reason_codes.iter().map(|reason| format!("{reason:?}")).collect::<Vec<_>>().join("|")
            )
        }));
        lines.join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        std::fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("kis_collection_batch_plan.txt");
        std::fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        std::fs::write(
            output_dir.join("kis_collection_batch_plan.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

fn build_storage_budget_summary(
    config: &KISMarketDataActivationConfig,
    whitelist: &KISSymbolWhitelist,
) -> KISStorageBudgetSummary {
    let estimated_raw_archive_bytes =
        whitelist.enabled_entries.len() * config.max_rows_per_symbol * 96;
    let estimated_canonical_csv_bytes =
        whitelist.enabled_entries.len() * config.max_rows_per_symbol * 72;
    let estimated_total_bytes = estimated_raw_archive_bytes + estimated_canonical_csv_bytes;
    let budget_ok = estimated_raw_archive_bytes <= config.max_raw_bytes
        && estimated_canonical_csv_bytes <= config.max_canonical_bytes
        && estimated_total_bytes <= config.max_total_bytes;
    let mut reason_codes = vec![ReasonCode::StorageBudgetReportBuilt];
    if !budget_ok {
        reason_codes.push(ReasonCode::BudgetExceeded);
    }
    KISStorageBudgetSummary {
        estimated_raw_archive_bytes,
        estimated_canonical_csv_bytes,
        estimated_total_bytes,
        budget_ok,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn base_job(
    config: &KISMarketDataActivationConfig,
    entry: &KISSymbolEntry,
    job_kind: KISCollectionJobKind,
    status: KISCollectionJobStatus,
    endpoint_category: KISEndpointCategory,
    expected_raw_archive_path: Option<String>,
    expected_canonical_csv_path: Option<String>,
    reason_codes: Vec<ReasonCode>,
) -> KISCollectionJob {
    let stem = format!("{:?}-{}", entry.market, entry.normalized_symbol).to_ascii_lowercase();
    let output_dir = config.output_dir();
    let canonical_path = expected_canonical_csv_path.clone().or_else(|| {
        Some(
            output_dir
                .join(format!("{stem}_canonical.csv"))
                .display()
                .to_string(),
        )
    });
    KISCollectionJob {
        job_id: format!("kis-{}-{}", stem, job_slug(job_kind)),
        job_kind,
        provider_kind: ProviderKind::KoreaInvestmentMarketData,
        market: entry.market,
        venue: entry.venue.clone(),
        provider_symbol: entry.provider_symbol.clone(),
        normalized_symbol: entry.normalized_symbol.clone(),
        exchange_code: entry.exchange_code.clone(),
        timeframe: entry.timeframe.clone(),
        freshness: entry.data_freshness,
        start_date: None,
        end_date: None,
        max_rows: entry
            .max_rows
            .unwrap_or(config.max_rows_per_symbol)
            .min(config.max_rows_per_symbol),
        max_requests: config.max_requests,
        max_days: config.max_days,
        endpoint_category,
        expected_raw_archive_path,
        expected_canonical_csv_path: canonical_path.clone(),
        expected_provenance_path: canonical_path
            .as_ref()
            .map(|path| infer_sidecar_path(path, "_provenance.json")),
        expected_preflight_path: canonical_path
            .as_ref()
            .map(|path| infer_sidecar_path(path, "_preflight.json")),
        expected_manifest_path: canonical_path
            .as_ref()
            .map(|path| infer_sidecar_path(path, "_manifest.json")),
        status,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn live_job_decision(
    config: &KISMarketDataActivationConfig,
    auth: &KISAuthReadinessReport,
    endpoint_policy: &KISEndpointPolicy,
    entry: &KISSymbolEntry,
    endpoint_category: KISEndpointCategory,
    endpoint_report: &KISEndpointPolicyReport,
) -> (
    KISCollectionJobKind,
    KISCollectionJobStatus,
    Vec<ReasonCode>,
) {
    if endpoint_report.unsafe_endpoint_detected && endpoint_category.is_broker_surface() {
        return (
            KISCollectionJobKind::SkippedEndpointDenied,
            KISCollectionJobStatus::Skipped,
            vec![ReasonCode::KISEndpointDenied, ReasonCode::DeniedByDefault],
        );
    }
    if !endpoint_policy.is_allowed(endpoint_category) {
        return (
            KISCollectionJobKind::SkippedEndpointDenied,
            KISCollectionJobStatus::Skipped,
            vec![ReasonCode::KISEndpointDenied, ReasonCode::DeniedByDefault],
        );
    }
    if !config.run_live_market_data_collection {
        return (
            KISCollectionJobKind::SkippedLiveCollectionDisabled,
            KISCollectionJobStatus::Skipped,
            vec![ReasonCode::KISCollectionDisabledByDefault],
        );
    }
    if config.max_days > 365 {
        return (
            KISCollectionJobKind::SkippedFullHistoryDenied,
            KISCollectionJobStatus::Skipped,
            vec![ReasonCode::FullHistoryDenied],
        );
    }
    match auth.readiness_status {
        KISAuthReadinessStatus::MissingAppKey | KISAuthReadinessStatus::MissingAppKeyAndSecret => {
            return (
                KISCollectionJobKind::SkippedMissingAppKey,
                KISCollectionJobStatus::Skipped,
                vec![ReasonCode::MissingApiKey],
            );
        }
        KISAuthReadinessStatus::MissingAppSecret => {
            return (
                KISCollectionJobKind::SkippedMissingAppSecret,
                KISCollectionJobStatus::Skipped,
                vec![ReasonCode::MissingAuth],
            );
        }
        KISAuthReadinessStatus::MissingBaseUrl => {
            return (
                KISCollectionJobKind::SkippedMissingBaseUrl,
                KISCollectionJobStatus::Skipped,
                vec![ReasonCode::MissingEndpointTemplate],
            );
        }
        KISAuthReadinessStatus::MissingWebSocketApprovalKey
            if endpoint_category.requires_websocket_approval() =>
        {
            return (
                KISCollectionJobKind::SkippedMissingWebSocketApprovalKey,
                KISCollectionJobStatus::Skipped,
                vec![ReasonCode::MissingApproval],
            );
        }
        _ => {}
    }
    (
        live_kind(entry),
        KISCollectionJobStatus::ReadyToRun,
        vec![
            ReasonCode::KISProviderConfigured,
            ReasonCode::ProviderRequestPlanned,
        ],
    )
}

fn job_slug(kind: KISCollectionJobKind) -> &'static str {
    match kind {
        KISCollectionJobKind::FixtureReplayDomestic => "fixture-domestic",
        KISCollectionJobKind::FixtureReplayOverseas => "fixture-overseas",
        KISCollectionJobKind::LocalCanonicalCsvImport => "local-import",
        KISCollectionJobKind::ExistingCollectedCsvReuse => "reuse",
        KISCollectionJobKind::KISDomesticEodCollectDryRun => "dry-run-domestic-eod",
        KISCollectionJobKind::KISDomesticMinuteCollectDryRun => "dry-run-domestic-minute",
        KISCollectionJobKind::KISDomesticRealtimeQuoteDryRun => "dry-run-domestic-rt",
        KISCollectionJobKind::KISOverseasEodCollectDryRun => "dry-run-overseas-eod",
        KISCollectionJobKind::KISOverseasRealtimeQuoteDryRun => "dry-run-overseas-rt",
        KISCollectionJobKind::KISDomesticEodCollectLive => "live-domestic-eod",
        KISCollectionJobKind::KISDomesticMinuteCollectLive => "live-domestic-minute",
        KISCollectionJobKind::KISOverseasEodCollectLive => "live-overseas-eod",
        KISCollectionJobKind::KISOverseasQuoteCollectLive => "live-overseas-quote",
        KISCollectionJobKind::SkippedMissingAppKey => "skip-missing-app-key",
        KISCollectionJobKind::SkippedMissingAppSecret => "skip-missing-app-secret",
        KISCollectionJobKind::SkippedMissingBaseUrl => "skip-missing-base-url",
        KISCollectionJobKind::SkippedMissingWebSocketApprovalKey => "skip-missing-approval",
        KISCollectionJobKind::SkippedEndpointDenied => "skip-endpoint-denied",
        KISCollectionJobKind::SkippedBudgetExceeded => "skip-budget",
        KISCollectionJobKind::SkippedInvalidSymbol => "skip-invalid-symbol",
        KISCollectionJobKind::SkippedAllSymbolDenied => "skip-scope",
        KISCollectionJobKind::SkippedFullHistoryDenied => "skip-full-history",
        KISCollectionJobKind::SkippedLiveCollectionDisabled => "skip-live-disabled",
        KISCollectionJobKind::SkippedUnsupportedSchema => "skip-schema",
        KISCollectionJobKind::DiagnosticOnly => "diagnostic",
    }
}

fn dry_run_kind(entry: &KISSymbolEntry) -> KISCollectionJobKind {
    match (entry.market, entry.timeframe.as_str(), entry.data_freshness) {
        (KISMarket::KoreanEquity, "1d", _) => KISCollectionJobKind::KISDomesticEodCollectDryRun,
        (KISMarket::KoreanEquity, _, KISDataFreshness::Realtime) => {
            KISCollectionJobKind::KISDomesticRealtimeQuoteDryRun
        }
        (KISMarket::KoreanEquity, _, _) => KISCollectionJobKind::KISDomesticMinuteCollectDryRun,
        (_, _, KISDataFreshness::Realtime) => KISCollectionJobKind::KISOverseasRealtimeQuoteDryRun,
        _ => KISCollectionJobKind::KISOverseasEodCollectDryRun,
    }
}

fn live_kind(entry: &KISSymbolEntry) -> KISCollectionJobKind {
    match (entry.market, entry.timeframe.as_str()) {
        (KISMarket::KoreanEquity, "1d") => KISCollectionJobKind::KISDomesticEodCollectLive,
        (KISMarket::KoreanEquity, _) => KISCollectionJobKind::KISDomesticMinuteCollectLive,
        (_, "1d") => KISCollectionJobKind::KISOverseasEodCollectLive,
        _ => KISCollectionJobKind::KISOverseasQuoteCollectLive,
    }
}

fn is_dry_run_kind(kind: KISCollectionJobKind) -> bool {
    matches!(
        kind,
        KISCollectionJobKind::KISDomesticEodCollectDryRun
            | KISCollectionJobKind::KISDomesticMinuteCollectDryRun
            | KISCollectionJobKind::KISDomesticRealtimeQuoteDryRun
            | KISCollectionJobKind::KISOverseasEodCollectDryRun
            | KISCollectionJobKind::KISOverseasRealtimeQuoteDryRun
    )
}

fn is_live_kind(kind: KISCollectionJobKind) -> bool {
    matches!(
        kind,
        KISCollectionJobKind::KISDomesticEodCollectLive
            | KISCollectionJobKind::KISDomesticMinuteCollectLive
            | KISCollectionJobKind::KISOverseasEodCollectLive
            | KISCollectionJobKind::KISOverseasQuoteCollectLive
    )
}

fn collect_ids<F>(jobs: &[KISCollectionJob], predicate: F) -> Vec<String>
where
    F: Fn(&KISCollectionJob) -> bool,
{
    jobs.iter()
        .filter(|job| predicate(job))
        .map(|job| job.job_id.clone())
        .collect()
}

fn endpoint_category_for_entry(entry: &KISSymbolEntry) -> KISEndpointCategory {
    match (entry.market, entry.timeframe.as_str(), entry.data_freshness) {
        (KISMarket::KoreanEquity, _, KISDataFreshness::Realtime) => {
            KISEndpointCategory::DomesticStockRealtimeQuote
        }
        (KISMarket::KoreanEquity, "1d", _) => KISEndpointCategory::DomesticStockPeriodPrice,
        (KISMarket::KoreanEquity, _, _) => KISEndpointCategory::DomesticStockMinutePrice,
        (_, _, KISDataFreshness::Realtime) => KISEndpointCategory::OverseasStockRealtimeQuote,
        _ => KISEndpointCategory::OverseasStockPeriodPrice,
    }
}

fn infer_fixture_response_path(
    config: &KISMarketDataActivationConfig,
    entry: &KISSymbolEntry,
) -> Option<String> {
    let base_dirs = [
        config.domestic_symbol_whitelist_path.as_deref(),
        config.overseas_symbol_whitelist_path.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter_map(|path| Path::new(path).parent().map(|parent| parent.to_path_buf()))
    .collect::<Vec<_>>();
    let mut candidates = Vec::new();
    for base_dir in base_dirs {
        if entry.market == KISMarket::KoreanEquity {
            candidates.push(base_dir.join(format!(
                "kis_domestic_{}_fixture.json",
                entry.normalized_symbol
            )));
        } else {
            let exchange = entry
                .exchange_code
                .clone()
                .unwrap_or_else(|| "na".to_string())
                .to_ascii_lowercase();
            candidates.push(base_dir.join(format!(
                "kis_overseas_{}_{}_fixture.json",
                exchange,
                entry.normalized_symbol.to_ascii_lowercase()
            )));
        }
    }
    candidates
        .into_iter()
        .find(|path| path.exists())
        .map(|path| path.display().to_string())
}

fn match_symbol_path(paths: &[String], entry: &KISSymbolEntry) -> Option<String> {
    paths
        .iter()
        .find(|path| path_matches_entry(path, entry))
        .cloned()
}

fn path_matches_entry(path: &str, entry: &KISSymbolEntry) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.contains(&entry.normalized_symbol.to_ascii_lowercase())
        || lower.contains(&entry.provider_symbol.to_ascii_lowercase())
}

fn infer_sidecar_path(path: &str, suffix: &str) -> String {
    let path = Path::new(path);
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    path.with_file_name(format!("{stem}{suffix}"))
        .display()
        .to_string()
}
