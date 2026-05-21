use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::future_window_requirements::{
    FutureWindowRequirementConfig, FutureWindowRequirementItem, FutureWindowRequirementReport,
    FutureWindowRequirementRunner, load_descriptor_map_from_paths,
};
use super::multi_row_official_evidence::{
    MultiRowOfficialEvidenceSet, load_multi_row_official_evidence_set_from_path_or_config,
};
use super::official_ready_row_inventory::{
    OfficialReadyRowInventoryReport, OfficialReadyRowInventoryRunner,
};
use super::{ComparableCommitteeEvidenceRow, OfficialReadyRowInventoryConfig};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FutureWindowScaleOutConfig {
    #[serde(alias = "plan_id")]
    pub scaleout_id: String,
    #[serde(default)]
    #[serde(alias = "multi_row_set_path")]
    pub multi_row_set_config_path: Option<String>,
    #[serde(default)]
    pub future_window_requirement_paths: Vec<String>,
    #[serde(default)]
    pub official_ready_inventory_paths: Vec<String>,
    #[serde(default)]
    pub comparable_evidence_bundle_paths: Vec<String>,
    #[serde(default)]
    pub official_candle_pack_paths: Vec<String>,
    #[serde(default)]
    pub canonical_csv_paths: Vec<String>,
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
    pub group_by_symbol_timeframe: bool,
    #[serde(default = "default_true")]
    pub prefer_local_extension: bool,
    #[serde(default = "default_true")]
    pub generate_provider_jobs: bool,
    #[serde(default = "default_true")]
    pub run_local_extension_jobs: bool,
    #[serde(default)]
    pub run_provider_collection_jobs: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FutureWindowScaleOutJobKind {
    LocalReuseOnly,
    LocalExtensionCandidate,
    ProviderCollectionPlanned,
    SkippedSufficient,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FutureWindowScaleOutGroup {
    pub group_id: String,
    pub symbol: String,
    pub market: crate::data::ProviderMarket,
    pub timeframe: String,
    pub row_ids: Vec<String>,
    pub row_count: usize,
    pub missing_future_bars: usize,
    pub local_reuse_possible: bool,
    pub provider_job_planned: bool,
    pub job_kind: FutureWindowScaleOutJobKind,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FutureWindowScaleOutJob {
    pub job_id: String,
    pub group_id: String,
    pub row_ids: Vec<String>,
    pub row_count: usize,
    pub expected_requests: usize,
    pub estimated_bytes: usize,
    pub runnable: bool,
    pub operator_action: String,
    pub job_kind: FutureWindowScaleOutJobKind,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FutureWindowScaleOutPlan {
    pub scaleout_id: String,
    pub grouped_requirements: Vec<FutureWindowScaleOutGroup>,
    pub jobs: Vec<FutureWindowScaleOutJob>,
    pub runnable_jobs: usize,
    pub skipped_jobs: usize,
    pub expected_rows_with_sufficient_windows: usize,
    pub expected_outcome_buildable_rows: usize,
    pub operator_actions: Vec<String>,
    pub storage_budget_summary: String,
    pub requirement_report: FutureWindowRequirementReport,
    #[serde(default, skip_serializing)]
    pub groups: Vec<FutureWindowScaleOutGroup>,
    #[serde(default, skip_serializing)]
    pub grouped_symbols: usize,
    #[serde(default, skip_serializing)]
    pub grouped_timeframes: usize,
    #[serde(default, skip_serializing)]
    pub local_reuse_groups: usize,
    #[serde(default, skip_serializing)]
    pub provider_job_groups: usize,
    #[serde(default, skip_serializing)]
    pub status_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FutureWindowScaleOutPlanner;

impl Default for FutureWindowScaleOutConfig {
    fn default() -> Self {
        Self {
            scaleout_id: "future-window-scaleout".to_string(),
            multi_row_set_config_path: None,
            future_window_requirement_paths: Vec::new(),
            official_ready_inventory_paths: Vec::new(),
            comparable_evidence_bundle_paths: Vec::new(),
            official_candle_pack_paths: Vec::new(),
            canonical_csv_paths: Vec::new(),
            output_root: default_output_root(),
            max_jobs: default_max_jobs(),
            max_symbols: default_max_symbols(),
            max_rows_per_job: default_max_rows_per_job(),
            max_requests_per_job: default_max_requests_per_job(),
            max_total_bytes: default_max_total_bytes(),
            group_by_symbol_timeframe: true,
            prefer_local_extension: true,
            generate_provider_jobs: true,
            run_local_extension_jobs: true,
            run_provider_collection_jobs: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl FutureWindowScaleOutConfig {
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
        if self.scaleout_id.trim().is_empty() {
            return Err("future-window scaleout id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("future-window scaleout paths must be local".to_string());
        }
        if self.max_jobs == 0 || self.max_jobs > default_max_jobs() {
            return Err("future-window scaleout max_jobs must be between 1 and 1000".to_string());
        }
        if self.max_rows_per_job == 0 || self.max_rows_per_job > default_max_rows_per_job() {
            return Err(
                "future-window scaleout max_rows_per_job must be between 1 and 500".to_string(),
            );
        }
        if self.max_requests_per_job == 0
            || self.max_requests_per_job > default_max_requests_per_job()
        {
            return Err(
                "future-window scaleout max_requests_per_job must be between 1 and 50".to_string(),
            );
        }
        if self.max_total_bytes == 0 || self.max_total_bytes > default_max_total_bytes() {
            return Err(
                "future-window scaleout max_total_bytes must be between 1 and 5000000".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.scaleout_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.multi_row_set_config_path
            .iter()
            .cloned()
            .chain(self.future_window_requirement_paths.iter().cloned())
            .chain(self.official_ready_inventory_paths.iter().cloned())
            .chain(self.comparable_evidence_bundle_paths.iter().cloned())
            .chain(self.official_candle_pack_paths.iter().cloned())
            .chain(self.canonical_csv_paths.iter().cloned())
            .collect()
    }
}

impl FutureWindowScaleOutPlanner {
    pub fn plan(
        &self,
        config: &FutureWindowScaleOutConfig,
    ) -> Result<FutureWindowScaleOutPlan, String> {
        config.validate()?;
        let requirement_report = load_requirement_report(config)?;
        let mut grouped = BTreeMap::<String, Vec<FutureWindowRequirementItem>>::new();
        for item in &requirement_report.items {
            let key = if config.group_by_symbol_timeframe {
                format!(
                    "{}::{:?}::{}",
                    item.symbol.to_ascii_lowercase(),
                    item.market,
                    item.timeframe.to_ascii_lowercase()
                )
            } else {
                item.row_id.clone()
            };
            grouped.entry(key).or_default().push(item.clone());
        }
        let mut grouped_requirements = grouped
            .into_iter()
            .map(|(_, mut items)| {
                items.sort_by(|left, right| {
                    left.row_id
                        .cmp(&right.row_id)
                        .then(left.timestamp_ms.cmp(&right.timestamp_ms))
                });
                let first = items
                    .first()
                    .cloned()
                    .unwrap_or_else(|| FutureWindowRequirementItem {
                        row_id: String::new(),
                        scenario_row_id: None,
                        comparable_row_id: None,
                        candle_series_id: None,
                        symbol: String::new(),
                        market: crate::data::ProviderMarket::USEquity,
                        venue: None,
                        timeframe: String::new(),
                        timestamp_ms: 0,
                        horizon_bars: 0,
                        current_available_future_bars: 0,
                        required_future_bars: 0,
                        missing_future_bars: 0,
                        required_start_timestamp_ms: 0,
                        required_end_timestamp_ms: 0,
                        can_extend_from_existing_csv: false,
                        can_extend_from_provider_collection: false,
                        gap_kind:
                            super::future_window_requirements::FutureWindowGapKind::DiagnosticOnly,
                        reason_codes: Vec::new(),
                    });
                let row_ids = items
                    .iter()
                    .map(|item| item.row_id.clone())
                    .collect::<Vec<_>>();
                let missing_future_bars = items.iter().map(|item| item.missing_future_bars).sum();
                let local_reuse_possible =
                    items.iter().all(|item| item.can_extend_from_existing_csv);
                let provider_job_planned = config.generate_provider_jobs
                    && items
                        .iter()
                        .any(|item| item.can_extend_from_provider_collection)
                    && !local_reuse_possible;
                let job_kind = if items.iter().all(|item| item.missing_future_bars == 0) {
                    FutureWindowScaleOutJobKind::SkippedSufficient
                } else if config.prefer_local_extension && local_reuse_possible {
                    if items.iter().any(|item| item.missing_future_bars > 0) {
                        FutureWindowScaleOutJobKind::LocalExtensionCandidate
                    } else {
                        FutureWindowScaleOutJobKind::LocalReuseOnly
                    }
                } else if provider_job_planned {
                    FutureWindowScaleOutJobKind::ProviderCollectionPlanned
                } else {
                    FutureWindowScaleOutJobKind::DiagnosticOnly
                };
                FutureWindowScaleOutGroup {
                    group_id: format!(
                        "{}-{}-{}",
                        first.symbol.to_ascii_lowercase(),
                        format!("{:?}", first.market).to_ascii_lowercase(),
                        first.timeframe.to_ascii_lowercase()
                    ),
                    symbol: first.symbol,
                    market: first.market,
                    timeframe: first.timeframe,
                    row_ids,
                    row_count: items.len(),
                    missing_future_bars,
                    local_reuse_possible,
                    provider_job_planned,
                    job_kind,
                    reason_codes: stable_reason_codes(&[
                        ReasonCode::DeterministicPath,
                        ReasonCode::LocalFileOnly,
                    ]),
                }
            })
            .collect::<Vec<_>>();
        grouped_requirements.sort_by(|left, right| {
            left.group_id
                .cmp(&right.group_id)
                .then(left.symbol.cmp(&right.symbol))
                .then(left.timeframe.cmp(&right.timeframe))
        });
        if grouped_requirements.len() > config.max_jobs {
            return Err(format!(
                "future-window scaleout grouped {} jobs which exceeds max_jobs {}",
                grouped_requirements.len(),
                config.max_jobs
            ));
        }
        let grouped_symbols = grouped_requirements
            .iter()
            .map(|group| group.symbol.clone())
            .collect::<BTreeSet<_>>()
            .len();
        if grouped_symbols > config.max_symbols {
            return Err(format!(
                "future-window scaleout grouped {} symbols which exceeds max_symbols {}",
                grouped_symbols, config.max_symbols
            ));
        }
        let grouped_timeframes = grouped_requirements
            .iter()
            .map(|group| group.timeframe.clone())
            .collect::<BTreeSet<_>>()
            .len();
        let local_reuse_groups = grouped_requirements
            .iter()
            .filter(|group| {
                matches!(
                    group.job_kind,
                    FutureWindowScaleOutJobKind::LocalReuseOnly
                        | FutureWindowScaleOutJobKind::LocalExtensionCandidate
                )
            })
            .count();
        let provider_job_groups = grouped_requirements
            .iter()
            .filter(|group| {
                group.job_kind == FutureWindowScaleOutJobKind::ProviderCollectionPlanned
            })
            .count();
        let mut jobs = grouped_requirements
            .iter()
            .map(|group| {
                let runnable = match group.job_kind {
                    FutureWindowScaleOutJobKind::LocalReuseOnly
                    | FutureWindowScaleOutJobKind::LocalExtensionCandidate => {
                        config.run_local_extension_jobs
                    }
                    FutureWindowScaleOutJobKind::ProviderCollectionPlanned => {
                        config.run_provider_collection_jobs
                    }
                    _ => false,
                };
                let expected_requests = group.row_count.min(config.max_requests_per_job);
                let estimated_bytes = group.row_count.saturating_mul(256);
                let operator_action = match group.job_kind {
                    FutureWindowScaleOutJobKind::SkippedSufficient => format!(
                        "no action: {} already has sufficient future windows",
                        group.group_id
                    ),
                    FutureWindowScaleOutJobKind::LocalReuseOnly
                    | FutureWindowScaleOutJobKind::LocalExtensionCandidate => format!(
                        "extend local candle coverage for {} ({})",
                        group.symbol, group.timeframe
                    ),
                    FutureWindowScaleOutJobKind::ProviderCollectionPlanned => format!(
                        "stage provider collection approval for {} ({})",
                        group.symbol, group.timeframe
                    ),
                    FutureWindowScaleOutJobKind::DiagnosticOnly => format!(
                        "diagnose future-window gap for {} ({})",
                        group.symbol, group.timeframe
                    ),
                };
                FutureWindowScaleOutJob {
                    job_id: format!("{}-job", group.group_id),
                    group_id: group.group_id.clone(),
                    row_ids: group.row_ids.clone(),
                    row_count: group.row_count.min(config.max_rows_per_job),
                    expected_requests,
                    estimated_bytes,
                    runnable,
                    operator_action,
                    job_kind: group.job_kind,
                    reason_codes: group.reason_codes.clone(),
                }
            })
            .collect::<Vec<_>>();
        jobs.sort_by(|left, right| left.job_id.cmp(&right.job_id));
        let runnable_jobs = jobs.iter().filter(|job| job.runnable).count();
        let skipped_jobs = jobs.len().saturating_sub(runnable_jobs);
        let expected_rows_with_sufficient_windows = requirement_report
            .rows_with_sufficient_future_window
            + jobs
                .iter()
                .filter(|job| {
                    job.runnable
                        && matches!(
                            job.job_kind,
                            FutureWindowScaleOutJobKind::LocalReuseOnly
                                | FutureWindowScaleOutJobKind::LocalExtensionCandidate
                                | FutureWindowScaleOutJobKind::ProviderCollectionPlanned
                        )
                })
                .map(|job| job.row_count)
                .sum::<usize>();
        let expected_outcome_buildable_rows = expected_rows_with_sufficient_windows.min(
            requirement_report
                .items
                .iter()
                .filter(|item| item.scenario_row_id.is_some())
                .count(),
        );
        let operator_actions = jobs
            .iter()
            .filter(|job| !matches!(job.job_kind, FutureWindowScaleOutJobKind::SkippedSufficient))
            .map(|job| job.operator_action.clone())
            .collect::<Vec<_>>();
        let estimated_total_bytes = jobs.iter().map(|job| job.estimated_bytes).sum::<usize>();
        let storage_budget_summary = format!(
            "estimated_total_bytes={};max_total_bytes={};within_budget={};research_only_warning=future-window scaleout remains local-first, bounded, and never implies live trading",
            estimated_total_bytes,
            config.max_total_bytes,
            estimated_total_bytes <= config.max_total_bytes
        );
        let status_summary = storage_budget_summary.clone();
        Ok(FutureWindowScaleOutPlan {
            scaleout_id: config.scaleout_id.clone(),
            grouped_requirements: grouped_requirements.clone(),
            jobs,
            runnable_jobs,
            skipped_jobs,
            expected_rows_with_sufficient_windows,
            expected_outcome_buildable_rows,
            operator_actions,
            storage_budget_summary,
            requirement_report,
            groups: grouped_requirements,
            grouped_symbols,
            grouped_timeframes,
            local_reuse_groups,
            provider_job_groups,
            status_summary,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::DeterministicPath,
                        ReasonCode::LocalFileOnly,
                        ReasonCode::ProviderRequestPlanned,
                    ])
                    .collect::<Vec<_>>(),
            ),
        })
    }
}

impl FutureWindowScaleOutPlan {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(
            &serde_json::to_string(self).unwrap_or_else(|_| self.scaleout_id.clone()),
        )
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("scaleout_id={}", self.scaleout_id),
            format!("grouped_requirements={}", self.grouped_requirements.len()),
            format!("jobs={}", self.jobs.len()),
            format!("runnable_jobs={}", self.runnable_jobs),
            format!("skipped_jobs={}", self.skipped_jobs),
            format!(
                "expected_rows_with_sufficient_windows={}",
                self.expected_rows_with_sufficient_windows
            ),
            format!(
                "expected_outcome_buildable_rows={}",
                self.expected_outcome_buildable_rows
            ),
            format!("operator_actions={}", self.operator_actions.join(" | ")),
            format!("storage_budget_summary={}", self.storage_budget_summary),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.grouped_requirements.iter().map(|group| {
            format!(
                "group_id={};symbol={};timeframe={};row_count={};missing_future_bars={};local_reuse_possible={};provider_job_planned={};job_kind={:?}",
                group.group_id,
                group.symbol,
                group.timeframe,
                group.row_count,
                group.missing_future_bars,
                group.local_reuse_possible,
                group.provider_job_planned,
                group.job_kind,
            )
        }));
        lines.extend(self.jobs.iter().map(|job| {
            format!(
                "job_id={};group_id={};row_count={};expected_requests={};estimated_bytes={};runnable={};job_kind={:?};operator_action={}",
                job.job_id,
                job.group_id,
                job.row_count,
                job.expected_requests,
                job.estimated_bytes,
                job.runnable,
                job.job_kind,
                job.operator_action
            )
        }));
        lines.push(self.requirement_report.to_text());
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
            output_dir.join("future_window_scaleout_plan.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("future_window_scaleout_plan.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_future_window_scaleout_plan_from_path_or_config(
    path: &str,
) -> Result<FutureWindowScaleOutPlan, String> {
    if path.ends_with(".json") {
        FutureWindowScaleOutPlan::from_json_path(Path::new(path))
    } else {
        FutureWindowScaleOutConfig::from_toml_path(Path::new(path))
            .and_then(|config| FutureWindowScaleOutPlanner::default().plan(&config))
    }
}

fn load_requirement_report(
    config: &FutureWindowScaleOutConfig,
) -> Result<FutureWindowRequirementReport, String> {
    if !config.future_window_requirement_paths.is_empty() {
        let mut merged = Vec::new();
        for path in &config.future_window_requirement_paths {
            let report = if path.ends_with(".json") {
                FutureWindowRequirementReport::from_json_path(Path::new(path))?
            } else {
                let requirement_config =
                    FutureWindowRequirementConfig::from_toml_path(Path::new(path))?;
                FutureWindowRequirementRunner::default().run(&requirement_config)?
            };
            merged.extend(report.items);
        }
        merged.sort_by(|left, right| left.row_id.cmp(&right.row_id));
        let rows_with_sufficient_future_window = merged
            .iter()
            .filter(|item| item.missing_future_bars == 0)
            .count();
        let rows_missing_future_window = merged
            .iter()
            .filter(|item| item.missing_future_bars > 0)
            .count();
        let rows_extendable_from_local_csv = merged
            .iter()
            .filter(|item| item.missing_future_bars > 0 && item.can_extend_from_existing_csv)
            .count();
        let rows_extendable_from_provider = merged
            .iter()
            .filter(|item| item.missing_future_bars > 0 && item.can_extend_from_provider_collection)
            .count();
        let rows_source_ineligible = merged
            .iter()
            .filter(|item| {
                matches!(
                    item.gap_kind,
                    super::future_window_requirements::FutureWindowGapKind::SourceIneligible
                )
            })
            .count();
        let no_lookahead_blocked_rows = merged
            .iter()
            .filter(|item| {
                matches!(
                    item.gap_kind,
                    super::future_window_requirements::FutureWindowGapKind::NoLookaheadViolation
                )
            })
            .count();
        return Ok(FutureWindowRequirementReport {
            requirement_id: format!("{}-requirements", config.scaleout_id),
            total_items: merged.len(),
            items: merged,
            rows_with_sufficient_future_window,
            rows_missing_future_window,
            rows_extendable_from_local_csv,
            rows_extendable_from_provider,
            rows_source_ineligible,
            no_lookahead_blocked_rows,
            requirement_status: if rows_missing_future_window > 0 {
                super::future_window_requirements::FutureWindowRequirementStatus::NeedLongerFutureWindow
            } else {
                super::future_window_requirements::FutureWindowRequirementStatus::HealthyFutureWindows
            },
            reason_codes: stable_reason_codes(&[
                ReasonCode::DeterministicPath,
                ReasonCode::LocalFileOnly,
            ]),
        });
    }

    if let Some(path) = config.multi_row_set_config_path.as_deref() {
        let set = load_multi_row_official_evidence_set_from_path_or_config(path)?;
        let inventory = inventory_from_set(config, &set)?;
        return FutureWindowRequirementRunner::default().run_from_inventory(
            &FutureWindowRequirementConfig {
                requirement_id: format!("{}-requirements", config.scaleout_id),
                official_ready_inventory_paths: config.official_ready_inventory_paths.clone(),
                comparable_evidence_bundle_paths: config.comparable_evidence_bundle_paths.clone(),
                candle_coverage_pack_paths: config
                    .official_candle_pack_paths
                    .iter()
                    .chain(config.canonical_csv_paths.iter())
                    .cloned()
                    .collect(),
                output_root: config.output_root.clone(),
                max_rows: config.max_jobs.min(500),
                max_symbols: config.max_symbols.min(5),
                ..FutureWindowRequirementConfig::default()
            },
            &inventory,
            &load_descriptor_map_from_paths(
                &config
                    .official_candle_pack_paths
                    .iter()
                    .chain(config.canonical_csv_paths.iter())
                    .cloned()
                    .collect::<Vec<_>>(),
            )?,
        );
    }

    let derived = FutureWindowRequirementConfig {
        requirement_id: format!("{}-requirements", config.scaleout_id),
        official_ready_inventory_paths: config.official_ready_inventory_paths.clone(),
        comparable_evidence_bundle_paths: config.comparable_evidence_bundle_paths.clone(),
        candle_coverage_pack_paths: config
            .official_candle_pack_paths
            .iter()
            .chain(config.canonical_csv_paths.iter())
            .cloned()
            .collect(),
        output_root: config.output_root.clone(),
        max_rows: config.max_jobs.min(500),
        max_symbols: config.max_symbols.min(5),
        ..FutureWindowRequirementConfig::default()
    };
    FutureWindowRequirementRunner::default().run(&derived)
}

fn inventory_from_set(
    config: &FutureWindowScaleOutConfig,
    set: &MultiRowOfficialEvidenceSet,
) -> Result<OfficialReadyRowInventoryReport, String> {
    let rows = set
        .items
        .iter()
        .map(|item| ComparableCommitteeEvidenceRow {
            row_id: item.row_id.clone(),
            symbol: item.symbol.clone(),
            market: item.market,
            timeframe: item.timeframe.clone(),
            horizon_bars: item.horizon_bars,
            timestamp_ms: item.timestamp_ms,
            source_kind: item.source_kind.clone(),
            source_class: item.source_class,
            scenario_row_id: item.scenario_row_id.clone(),
            committee_decision_id: None,
            committee_final_action: "Approve".to_string(),
            chair_decision: Some("Approve".to_string()),
            risk_governor_decision: Some("Reject".to_string()),
            baseline_action: Some("Approve".to_string()),
            external_action: None,
            no_trade_baseline_action: "NoTrade".to_string(),
            outcome_label: None,
            net_return_pct: None,
            cost_bps: 5.0,
            slippage_bps: 2.0,
            committee_vs_baseline_delta: None,
            committee_vs_notrade_delta: None,
            risk_denied_value_proxy: None,
            no_trade_value_proxy: None,
            outcome_reference_available: item.outcome_reference_available,
            baseline_reference_available: item.baseline_reference_available,
            no_trade_counterfactual_available: item.no_trade_counterfactual_available,
            risk_denied_counterfactual_available: item.risk_denied_counterfactual_available,
            external_reference_available: false,
            row_level: item.row_level,
            summary_derived: item.summary_derived,
            no_lookahead_safe: item.no_lookahead_safe,
            official_readiness_eligible: item.source_class
                == super::ComparableEvidenceSourceClass::OfficialNonCrypto,
            diagnostic_only: item.diagnostic_only,
            candle_coverage_available: item.has_local_candle_window,
            matched_candle_series_id: item.candle_series_id.clone(),
            candle_match_status: Some("Matched".to_string()),
            candle_official_ready_match: item.official_ready_match,
            candle_benchmark_ready_match: item.benchmark_ready_match,
            candle_diagnostic_only: item.diagnostic_only,
            reason_codes: item.reason_codes.clone(),
        })
        .collect::<Vec<_>>();
    OfficialReadyRowInventoryRunner::default().run_from_rows(
        &OfficialReadyRowInventoryConfig {
            inventory_id: format!("{}-inventory", config.scaleout_id),
            output_root: config.output_root.clone(),
            max_rows: config.max_jobs.min(500),
            max_symbols: config.max_symbols.min(5),
            allow_controlled_diagnostic: true,
            allow_crypto_only: true,
            allow_yfinance_research: true,
            allow_fixture: true,
            ..OfficialReadyRowInventoryConfig::default()
        },
        &rows,
        &BTreeMap::new(),
        &load_descriptor_map_from_paths(
            &config
                .official_candle_pack_paths
                .iter()
                .chain(config.canonical_csv_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )?,
    )
}

fn default_output_root() -> String {
    "target/soma_future_window_scaleout".to_string()
}

fn default_max_jobs() -> usize {
    1000
}

fn default_max_symbols() -> usize {
    10
}

fn default_max_rows_per_job() -> usize {
    500
}

fn default_max_requests_per_job() -> usize {
    50
}

fn default_max_total_bytes() -> usize {
    5_000_000
}

fn default_true() -> bool {
    true
}
