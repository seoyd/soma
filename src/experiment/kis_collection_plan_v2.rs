use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::kis_endpoint_policy::{KISEndpointCategory, KISEndpointPolicy, KISEndpointPolicyStatus};
use super::kis_market_data_dry_run::{KISMarketDataDryRunReport, KISMarketDataDryRunStatus};
use super::kis_symbol_whitelist::{
    KISDataFreshness, KISMarket, KISSymbolEntry, KISSymbolWhitelist, KISSymbolWhitelistConfig,
};

fn default_output_root() -> String {
    "target/sprint58/kis_collection_plan_v2".to_string()
}

fn default_max_jobs() -> usize {
    20
}

fn default_max_symbols() -> usize {
    5
}

fn default_max_rows_per_symbol() -> usize {
    300
}

fn default_max_requests() -> usize {
    10
}

fn default_max_days() -> usize {
    365
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn default_true() -> bool {
    true
}

fn push_plan_job(
    jobs: &mut Vec<KISCollectionPlanV2Job>,
    active_jobs: &mut usize,
    config: &KISCollectionPlanV2Config,
    entry: &KISSymbolEntry,
    job_kind: KISCollectionPlanV2JobKind,
    runnable: bool,
    command_suggestion: Option<String>,
    expected_artifact: Option<String>,
    reason_codes: Vec<ReasonCode>,
) {
    *active_jobs += 1;
    jobs.push(KISCollectionPlanV2Job {
        job_id: format!(
            "{}-{}-{:?}-{:02}",
            config.plan_id,
            entry.normalized_symbol.to_ascii_lowercase(),
            job_kind,
            *active_jobs
        ),
        job_kind,
        normalized_symbol: entry.normalized_symbol.clone(),
        market: entry.market,
        timeframe: entry.timeframe.clone(),
        runnable,
        command_suggestion,
        expected_artifact,
        reason_codes: stable_reason_codes(&reason_codes),
    });
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KISCollectionPlanV2Config {
    pub plan_id: String,
    #[serde(default)]
    pub dry_run_report_paths: Vec<String>,
    #[serde(default)]
    pub fixture_response_paths: Vec<String>,
    #[serde(default)]
    pub local_canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub domestic_symbol_whitelist_path: Option<String>,
    #[serde(default)]
    pub overseas_symbol_whitelist_path: Option<String>,
    #[serde(default)]
    pub endpoint_policy_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_jobs")]
    pub max_jobs: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_rows_per_symbol")]
    pub max_rows_per_symbol: usize,
    #[serde(default = "default_max_requests")]
    pub max_requests: usize,
    #[serde(default = "default_max_days")]
    pub max_days: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub run_fixture_replay: bool,
    #[serde(default = "default_true")]
    pub run_local_import: bool,
    #[serde(default)]
    pub run_operator_live_collection: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISCollectionPlanV2Status {
    Ready,
    MissingAuth,
    MissingBaseUrl,
    EndpointPolicyBlocked,
    LiveCollectionDisabled,
    BudgetExceeded,
    NoSymbols,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISCollectionPlanV2JobKind {
    FixtureReplayDomestic,
    FixtureReplayOverseas,
    LocalCanonicalCsvImport,
    ExistingCollectedCsvReuse,
    DomesticEodDryRun,
    OverseasEodDryRun,
    DomesticEodOperatorLive,
    OverseasEodOperatorLive,
    RealtimeQuoteDryRun,
    RealtimeQuoteOperatorLive,
    SkippedMissingAuth,
    SkippedMissingBaseUrl,
    SkippedEndpointPolicyBlocked,
    SkippedLiveCollectionDisabled,
    SkippedBudgetExceeded,
    SkippedInvalidSymbol,
    SkippedAllSymbolDenied,
    SkippedFullHistoryDenied,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISCollectionPlanV2Job {
    pub job_id: String,
    pub job_kind: KISCollectionPlanV2JobKind,
    pub normalized_symbol: String,
    pub market: KISMarket,
    pub timeframe: String,
    pub runnable: bool,
    #[serde(default)]
    pub command_suggestion: Option<String>,
    #[serde(default)]
    pub expected_artifact: Option<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISCollectionPlanV2StorageSummary {
    pub estimated_total_bytes: usize,
    pub budget_ok: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISCollectionPlanV2 {
    pub plan_id: String,
    #[serde(default)]
    pub jobs: Vec<KISCollectionPlanV2Job>,
    #[serde(default)]
    pub runnable_jobs: Vec<String>,
    #[serde(default)]
    pub skipped_jobs: Vec<String>,
    #[serde(default)]
    pub fixture_jobs: Vec<String>,
    #[serde(default)]
    pub local_import_jobs: Vec<String>,
    #[serde(default)]
    pub dry_run_jobs: Vec<String>,
    #[serde(default)]
    pub operator_live_jobs: Vec<String>,
    #[serde(default)]
    pub domestic_jobs: Vec<String>,
    #[serde(default)]
    pub overseas_jobs: Vec<String>,
    #[serde(default)]
    pub operator_actions: Vec<String>,
    pub storage_budget_summary: KISCollectionPlanV2StorageSummary,
    pub plan_status: KISCollectionPlanV2Status,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KISCollectionPlanV2Runner;

impl Default for KISCollectionPlanV2Config {
    fn default() -> Self {
        Self {
            plan_id: "sprint58-kis-collection-plan-v2".to_string(),
            dry_run_report_paths: Vec::new(),
            fixture_response_paths: Vec::new(),
            local_canonical_csv_paths: Vec::new(),
            domestic_symbol_whitelist_path: None,
            overseas_symbol_whitelist_path: None,
            endpoint_policy_path: None,
            output_root: default_output_root(),
            max_jobs: default_max_jobs(),
            max_symbols: default_max_symbols(),
            max_rows_per_symbol: default_max_rows_per_symbol(),
            max_requests: default_max_requests(),
            max_days: default_max_days(),
            max_bytes: default_max_bytes(),
            run_fixture_replay: true,
            run_local_import: true,
            run_operator_live_collection: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl KISCollectionPlanV2Config {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.plan_id.trim().is_empty() {
            return Err("kis collection plan v2 id must not be empty".to_string());
        }
        if self
            .dry_run_report_paths
            .iter()
            .chain(self.fixture_response_paths.iter())
            .chain(self.local_canonical_csv_paths.iter())
            .chain(self.domestic_symbol_whitelist_path.iter())
            .chain(self.overseas_symbol_whitelist_path.iter())
            .chain(self.endpoint_policy_path.iter())
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("kis collection plan v2 paths must be local".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.plan_id)
    }
}

impl KISCollectionPlanV2 {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            "market_data_only_warning=kis collection plan v2 is local-first and market-data-only"
                .to_string(),
            format!("plan_id={}", self.plan_id),
            format!("plan_status={:?}", self.plan_status),
            format!("runnable_jobs={}", self.runnable_jobs.join("|")),
            format!("skipped_jobs={}", self.skipped_jobs.join("|")),
            format!("fixture_jobs={}", self.fixture_jobs.join("|")),
            format!("local_import_jobs={}", self.local_import_jobs.join("|")),
            format!("dry_run_jobs={}", self.dry_run_jobs.join("|")),
            format!("operator_live_jobs={}", self.operator_live_jobs.join("|")),
            format!("domestic_jobs={}", self.domestic_jobs.join("|")),
            format!("overseas_jobs={}", self.overseas_jobs.join("|")),
            format!("operator_actions={}", self.operator_actions.join("|")),
            format!(
                "estimated_total_bytes={}",
                self.storage_budget_summary.estimated_total_bytes
            ),
            format!("budget_ok={}", self.storage_budget_summary.budget_ok),
        ];
        for job in &self.jobs {
            lines.push(format!(
                "job={};kind={:?};market={:?};runnable={};command={};expected={}",
                job.job_id,
                job.job_kind,
                job.market,
                job.runnable,
                job.command_suggestion.clone().unwrap_or_default(),
                job.expected_artifact.clone().unwrap_or_default()
            ));
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

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("kis_collection_plan_v2.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_collection_plan_v2.json"),
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

impl KISCollectionPlanV2Runner {
    pub fn run(&self, config: &KISCollectionPlanV2Config) -> Result<KISCollectionPlanV2, String> {
        config.validate()?;
        let dry_run_report = load_latest_report(&config.dry_run_report_paths)?;
        self.run_with_dry_run(config, dry_run_report.as_ref())
    }

    pub fn run_with_dry_run(
        &self,
        config: &KISCollectionPlanV2Config,
        dry_run_report: Option<&KISMarketDataDryRunReport>,
    ) -> Result<KISCollectionPlanV2, String> {
        config.validate()?;
        let endpoint_policy = if let Some(path) = &config.endpoint_policy_path {
            KISEndpointPolicy::from_toml_path(Path::new(path))?
        } else {
            KISEndpointPolicy::default()
        };
        let endpoint_policy_ok = matches!(
            endpoint_policy
                .report_for_categories(&[
                    KISEndpointCategory::DomesticStockPeriodPrice,
                    KISEndpointCategory::OverseasStockPeriodPrice,
                    KISEndpointCategory::DomesticStockRealtimeQuote,
                    KISEndpointCategory::OverseasStockRealtimeQuote,
                ])
                .policy_status,
            KISEndpointPolicyStatus::MarketDataOnly | KISEndpointPolicyStatus::DiagnosticOnly
        );
        let mut entries = merged_entries(
            load_whitelist(config.domestic_symbol_whitelist_path.as_deref())?,
            load_whitelist(config.overseas_symbol_whitelist_path.as_deref())?,
        );
        let total_symbols = entries.iter().filter(|entry| entry.enabled).count();
        let estimated_total_bytes = total_symbols
            .min(config.max_symbols)
            .saturating_mul(config.max_rows_per_symbol)
            .saturating_mul(96);
        let budget_ok = estimated_total_bytes <= config.max_bytes;
        let auth_status = dry_run_report.map(|report| report.auth_status);
        let plan_status = if dry_run_report.is_none() {
            KISCollectionPlanV2Status::DiagnosticOnly
        } else if matches!(
            dry_run_report.map(|report| report.dry_run_status),
            Some(KISMarketDataDryRunStatus::MissingAuth)
        ) {
            KISCollectionPlanV2Status::MissingAuth
        } else if matches!(
            dry_run_report.map(|report| report.dry_run_status),
            Some(KISMarketDataDryRunStatus::MissingBaseUrl)
        ) {
            KISCollectionPlanV2Status::MissingBaseUrl
        } else if !endpoint_policy_ok
            || matches!(
                dry_run_report.map(|report| report.dry_run_status),
                Some(KISMarketDataDryRunStatus::EndpointPolicyBlocked)
            )
        {
            KISCollectionPlanV2Status::EndpointPolicyBlocked
        } else if !budget_ok
            || matches!(
                dry_run_report.map(|report| report.dry_run_status),
                Some(KISMarketDataDryRunStatus::BudgetExceeded)
            )
        {
            KISCollectionPlanV2Status::BudgetExceeded
        } else if total_symbols == 0 {
            KISCollectionPlanV2Status::NoSymbols
        } else if !config.run_operator_live_collection {
            KISCollectionPlanV2Status::LiveCollectionDisabled
        } else {
            KISCollectionPlanV2Status::Ready
        };

        entries.sort_by(|left, right| {
            left.market
                .cmp(&right.market)
                .then(left.normalized_symbol.cmp(&right.normalized_symbol))
        });
        let mut jobs = Vec::new();
        let mut active_jobs = 0usize;
        for entry in entries.into_iter().filter(|entry| entry.enabled) {
            let endpoint_category = endpoint_category_for_entry(&entry);
            let fixture_path = matching_path(&config.fixture_response_paths, &entry);
            let csv_path = matching_path(&config.local_canonical_csv_paths, &entry);
            let all_symbol_denied = entry.provider_symbol.eq_ignore_ascii_case("ALL");
            let invalid_symbol = entry.normalized_symbol.trim().is_empty();
            let full_history_denied =
                entry.max_rows.unwrap_or(config.max_rows_per_symbol) > config.max_rows_per_symbol;
            let auth_missing = matches!(
                auth_status,
                Some(super::kis_auth_closure::KISAuthClosureStatus::MissingAppKey)
                    | Some(super::kis_auth_closure::KISAuthClosureStatus::MissingAppSecret)
                    | Some(super::kis_auth_closure::KISAuthClosureStatus::MissingAppKeyAndSecret)
            );
            let base_url_missing = matches!(
                auth_status,
                Some(super::kis_auth_closure::KISAuthClosureStatus::MissingBaseUrl)
            );

            if invalid_symbol {
                push_plan_job(
                    &mut jobs,
                    &mut active_jobs,
                    config,
                    &entry,
                    KISCollectionPlanV2JobKind::SkippedInvalidSymbol,
                    false,
                    None,
                    None,
                    vec![ReasonCode::InvalidSymbol],
                );
                continue;
            }
            if all_symbol_denied {
                push_plan_job(
                    &mut jobs,
                    &mut active_jobs,
                    config,
                    &entry,
                    KISCollectionPlanV2JobKind::SkippedAllSymbolDenied,
                    false,
                    None,
                    None,
                    vec![ReasonCode::DeniedByDefault],
                );
                continue;
            }
            if full_history_denied {
                push_plan_job(
                    &mut jobs,
                    &mut active_jobs,
                    config,
                    &entry,
                    KISCollectionPlanV2JobKind::SkippedFullHistoryDenied,
                    false,
                    None,
                    None,
                    vec![ReasonCode::FullHistoryDenied],
                );
                continue;
            }
            if !budget_ok || jobs.len() >= config.max_jobs {
                push_plan_job(
                    &mut jobs,
                    &mut active_jobs,
                    config,
                    &entry,
                    KISCollectionPlanV2JobKind::SkippedBudgetExceeded,
                    false,
                    None,
                    None,
                    vec![ReasonCode::BudgetExceeded],
                );
                continue;
            }
            if config.run_fixture_replay {
                if let Some(path) = fixture_path.clone() {
                    push_plan_job(
                        &mut jobs,
                        &mut active_jobs,
                        config,
                        &entry,
                        if entry.market == KISMarket::KoreanEquity {
                            KISCollectionPlanV2JobKind::FixtureReplayDomestic
                        } else {
                            KISCollectionPlanV2JobKind::FixtureReplayOverseas
                        },
                        true,
                        Some(format!("fixture-replay {}", path)),
                        Some(path),
                        vec![ReasonCode::MockFixtureLoaded, ReasonCode::LocalFileOnly],
                    );
                }
            }
            if config.run_local_import {
                if let Some(path) = csv_path.clone() {
                    push_plan_job(
                        &mut jobs,
                        &mut active_jobs,
                        config,
                        &entry,
                        if path.contains("collected") || path.contains("imported") {
                            KISCollectionPlanV2JobKind::ExistingCollectedCsvReuse
                        } else {
                            KISCollectionPlanV2JobKind::LocalCanonicalCsvImport
                        },
                        true,
                        Some(format!("local-import {}", path)),
                        Some(path),
                        vec![
                            ReasonCode::KISLocalImportPreferred,
                            ReasonCode::LocalFileOnly,
                        ],
                    );
                }
            }
            push_plan_job(
                &mut jobs,
                &mut active_jobs,
                config,
                &entry,
                dry_run_kind(endpoint_category),
                matches!(
                    dry_run_report.map(|report| report.dry_run_status),
                    Some(KISMarketDataDryRunStatus::Ready)
                ),
                Some(format!("dry-run {}", entry.normalized_symbol)),
                None,
                vec![ReasonCode::ProviderRequestPlanned],
            );
            let live_job_kind = if !endpoint_policy_ok {
                KISCollectionPlanV2JobKind::SkippedEndpointPolicyBlocked
            } else if auth_missing {
                KISCollectionPlanV2JobKind::SkippedMissingAuth
            } else if base_url_missing {
                KISCollectionPlanV2JobKind::SkippedMissingBaseUrl
            } else if !config.run_operator_live_collection {
                KISCollectionPlanV2JobKind::SkippedLiveCollectionDisabled
            } else if dry_run_report
                .is_some_and(|report| !report.safe_to_run_operator_live_collection)
            {
                KISCollectionPlanV2JobKind::SkippedMissingAuth
            } else {
                live_kind(endpoint_category)
            };
            let live_runnable = matches!(
                live_job_kind,
                KISCollectionPlanV2JobKind::DomesticEodOperatorLive
                    | KISCollectionPlanV2JobKind::OverseasEodOperatorLive
                    | KISCollectionPlanV2JobKind::RealtimeQuoteOperatorLive
            );
            push_plan_job(
                &mut jobs,
                &mut active_jobs,
                config,
                &entry,
                live_job_kind,
                live_runnable,
                if live_runnable {
                    Some(format!("operator-live {}", entry.normalized_symbol))
                } else {
                    None
                },
                None,
                vec![ReasonCode::KISCollectionDisabledByDefault],
            );
        }

        jobs.sort_by(|left, right| left.job_id.cmp(&right.job_id));
        let id_list = |filter: fn(&KISCollectionPlanV2Job) -> bool| {
            jobs.iter()
                .filter(|job| filter(job))
                .map(|job| job.job_id.clone())
                .collect::<Vec<_>>()
        };
        let fixture_jobs = id_list(|job| {
            matches!(
                job.job_kind,
                KISCollectionPlanV2JobKind::FixtureReplayDomestic
                    | KISCollectionPlanV2JobKind::FixtureReplayOverseas
            )
        });
        let local_import_jobs = id_list(|job| {
            matches!(
                job.job_kind,
                KISCollectionPlanV2JobKind::LocalCanonicalCsvImport
                    | KISCollectionPlanV2JobKind::ExistingCollectedCsvReuse
            )
        });
        let dry_run_jobs = id_list(|job| {
            matches!(
                job.job_kind,
                KISCollectionPlanV2JobKind::DomesticEodDryRun
                    | KISCollectionPlanV2JobKind::OverseasEodDryRun
                    | KISCollectionPlanV2JobKind::RealtimeQuoteDryRun
            )
        });
        let operator_live_jobs = id_list(|job| {
            matches!(
                job.job_kind,
                KISCollectionPlanV2JobKind::DomesticEodOperatorLive
                    | KISCollectionPlanV2JobKind::OverseasEodOperatorLive
                    | KISCollectionPlanV2JobKind::RealtimeQuoteOperatorLive
            )
        });
        let domestic_jobs = id_list(|job| job.market == KISMarket::KoreanEquity);
        let overseas_jobs = id_list(|job| job.market != KISMarket::KoreanEquity);
        let runnable_jobs = id_list(|job| job.runnable);
        let skipped_jobs = id_list(|job| !job.runnable);
        let mut reason_codes = config.reason_codes.clone();
        reason_codes.push(ReasonCode::KISCollectionPlanV2Built);
        if !budget_ok {
            reason_codes.push(ReasonCode::BudgetExceeded);
        }
        let plan = KISCollectionPlanV2 {
            plan_id: config.plan_id.clone(),
            jobs,
            runnable_jobs,
            skipped_jobs,
            fixture_jobs,
            local_import_jobs,
            dry_run_jobs,
            operator_live_jobs,
            domestic_jobs,
            overseas_jobs,
            operator_actions: if config.run_operator_live_collection {
                Vec::new()
            } else {
                vec!["operator live collection remains disabled by default".to_string()]
            },
            storage_budget_summary: KISCollectionPlanV2StorageSummary {
                estimated_total_bytes,
                budget_ok,
                reason_codes: stable_reason_codes(&if budget_ok {
                    vec![ReasonCode::CollectionBudgetReportBuilt]
                } else {
                    vec![
                        ReasonCode::CollectionBudgetReportBuilt,
                        ReasonCode::BudgetExceeded,
                    ]
                }),
            },
            plan_status,
            reason_codes: stable_reason_codes(&reason_codes),
        };
        plan.write_to_dir(&config.artifact_dir())?;
        Ok(plan)
    }
}

fn load_latest_report(paths: &[String]) -> Result<Option<KISMarketDataDryRunReport>, String> {
    let mut latest = None;
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        latest = Some(serde_json::from_str(&text).map_err(|err| err.to_string())?);
    }
    Ok(latest)
}

fn load_whitelist(path: Option<&str>) -> Result<Option<KISSymbolWhitelist>, String> {
    path.map(|path| {
        let config = KISSymbolWhitelistConfig::from_toml_path(Path::new(path))?;
        config.validate()?;
        Ok(config.build())
    })
    .transpose()
}

fn merged_entries(
    domestic: Option<KISSymbolWhitelist>,
    overseas: Option<KISSymbolWhitelist>,
) -> Vec<KISSymbolEntry> {
    domestic
        .into_iter()
        .chain(overseas)
        .flat_map(|whitelist| whitelist.entries)
        .collect()
}

fn matching_path(paths: &[String], entry: &KISSymbolEntry) -> Option<String> {
    let normalized = entry.normalized_symbol.to_ascii_lowercase();
    paths
        .iter()
        .find(|path| {
            let lower = path.to_ascii_lowercase();
            lower.contains(&normalized)
                || lower.contains(&entry.provider_symbol.to_ascii_lowercase())
        })
        .cloned()
}

fn endpoint_category_for_entry(entry: &KISSymbolEntry) -> KISEndpointCategory {
    if entry.data_freshness == KISDataFreshness::Realtime {
        if entry.market == KISMarket::KoreanEquity {
            KISEndpointCategory::DomesticStockRealtimeQuote
        } else {
            KISEndpointCategory::OverseasStockRealtimeQuote
        }
    } else if entry.market == KISMarket::KoreanEquity {
        KISEndpointCategory::DomesticStockPeriodPrice
    } else {
        KISEndpointCategory::OverseasStockPeriodPrice
    }
}

fn dry_run_kind(category: KISEndpointCategory) -> KISCollectionPlanV2JobKind {
    match category {
        KISEndpointCategory::DomesticStockRealtimeQuote
        | KISEndpointCategory::OverseasStockRealtimeQuote => {
            KISCollectionPlanV2JobKind::RealtimeQuoteDryRun
        }
        KISEndpointCategory::DomesticStockPeriodPrice => {
            KISCollectionPlanV2JobKind::DomesticEodDryRun
        }
        _ => KISCollectionPlanV2JobKind::OverseasEodDryRun,
    }
}

fn live_kind(category: KISEndpointCategory) -> KISCollectionPlanV2JobKind {
    match category {
        KISEndpointCategory::DomesticStockRealtimeQuote
        | KISEndpointCategory::OverseasStockRealtimeQuote => {
            KISCollectionPlanV2JobKind::RealtimeQuoteOperatorLive
        }
        KISEndpointCategory::DomesticStockPeriodPrice => {
            KISCollectionPlanV2JobKind::DomesticEodOperatorLive
        }
        _ => KISCollectionPlanV2JobKind::OverseasEodOperatorLive,
    }
}
