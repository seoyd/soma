use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};
use super::future_window_requirements::{
    FutureWindowRequirementConfig, FutureWindowRequirementRunner, load_descriptor_map_from_paths,
    load_rows_from_paths,
};
use super::future_window_scaleout::{
    FutureWindowScaleOutPlan, load_future_window_scaleout_plan_from_path_or_config,
};
use super::multi_row_official_evidence::{
    MultiRowOfficialEvidenceSet, load_multi_row_official_evidence_set_from_path_or_config,
};
use super::official_ready_row_inventory::{
    OfficialReadyRowInventoryConfig, OfficialReadyRowInventoryReport,
    OfficialReadyRowInventoryRunner,
};
use super::outcome_linkage_v3::{
    OutcomeLinkageV3Config, OutcomeLinkageV3Record, OutcomeLinkageV3Runner, OutcomeLinkageV3Status,
};
use super::triple_barrier_reference_builder::TripleBarrierTieBreakPolicy;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchOutcomeLinkageV3Config {
    pub batch_id: String,
    #[serde(default)]
    pub multi_row_set_config_path: Option<String>,
    #[serde(default)]
    pub future_window_scaleout_plan_path: Option<String>,
    #[serde(default)]
    pub official_ready_inventory_paths: Vec<String>,
    #[serde(default)]
    pub comparable_evidence_bundle_paths: Vec<String>,
    #[serde(default)]
    pub extended_candle_pack_paths: Vec<String>,
    #[serde(default)]
    pub include_row_ids: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_horizon_bars")]
    pub default_horizon_bars: usize,
    #[serde(default = "default_take_profit_pct")]
    pub take_profit_pct: f64,
    #[serde(default = "default_stop_loss_pct")]
    pub stop_loss_pct: f64,
    #[serde(default = "default_cost_bps")]
    pub cost_bps: f64,
    #[serde(default = "default_slippage_bps")]
    pub slippage_bps: f64,
    #[serde(default)]
    pub tie_break_policy: TripleBarrierTieBreakPolicy,
    #[serde(default = "default_true")]
    pub require_exact_symbol_match: bool,
    #[serde(default = "default_true")]
    pub require_exact_horizon_match: bool,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BatchOutcomeLinkageV3Status {
    BatchOutcomeLinksImproved,
    OfficialBatchOutcomeLinksImproved,
    StillNeedFutureWindows,
    StillNeedScenarioRows,
    StillTimestampMismatch,
    StillHorizonMismatch,
    SourceIneligible,
    DiagnosticOnly,
    #[default]
    NoImprovement,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchOutcomeLinkageV3Report {
    pub batch_id: String,
    pub records: Vec<OutcomeLinkageV3Record>,
    pub total_rows: usize,
    pub eligible_rows: usize,
    pub generated_outcome_count: usize,
    pub official_outcome_count: usize,
    pub skipped_missing_future_window_count: usize,
    pub skipped_missing_scenario_count: usize,
    pub skipped_timestamp_mismatch_count: usize,
    pub skipped_horizon_mismatch_count: usize,
    pub rejected_no_lookahead_count: usize,
    pub take_profit_count: usize,
    pub stop_loss_count: usize,
    pub time_expired_count: usize,
    pub linkage_status: BatchOutcomeLinkageV3Status,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BatchOutcomeLinkageV3Runner;

impl Default for BatchOutcomeLinkageV3Config {
    fn default() -> Self {
        Self {
            batch_id: "batch-outcome-linkage-v3".to_string(),
            multi_row_set_config_path: None,
            future_window_scaleout_plan_path: None,
            official_ready_inventory_paths: Vec::new(),
            comparable_evidence_bundle_paths: Vec::new(),
            extended_candle_pack_paths: Vec::new(),
            include_row_ids: Vec::new(),
            output_root: default_output_root(),
            default_horizon_bars: default_horizon_bars(),
            take_profit_pct: default_take_profit_pct(),
            stop_loss_pct: default_stop_loss_pct(),
            cost_bps: default_cost_bps(),
            slippage_bps: default_slippage_bps(),
            tie_break_policy: TripleBarrierTieBreakPolicy::StopFirst,
            require_exact_symbol_match: true,
            require_exact_horizon_match: true,
            require_no_lookahead_safe: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl BatchOutcomeLinkageV3Config {
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
        if self.batch_id.trim().is_empty() {
            return Err("batch outcome linkage id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("batch outcome linkage paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.batch_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.multi_row_set_config_path
            .iter()
            .chain(self.future_window_scaleout_plan_path.iter())
            .chain(self.official_ready_inventory_paths.iter())
            .chain(self.comparable_evidence_bundle_paths.iter())
            .chain(self.extended_candle_pack_paths.iter())
            .cloned()
            .collect()
    }
}

impl BatchOutcomeLinkageV3Runner {
    pub fn run(
        &self,
        config: &BatchOutcomeLinkageV3Config,
    ) -> Result<BatchOutcomeLinkageV3Report, String> {
        config.validate()?;
        let descriptors = load_descriptor_map_from_paths(&config.extended_candle_pack_paths)?;
        let set = config
            .multi_row_set_config_path
            .as_deref()
            .map(load_multi_row_official_evidence_set_from_path_or_config)
            .transpose()?;
        let rows = load_rows(config, set.as_ref())?;
        let inventory = load_inventory(config, &rows, &descriptors, set.as_ref())?;
        let allowed_rows = config
            .include_row_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let filter_row = |row_id: &str| allowed_rows.is_empty() || allowed_rows.contains(row_id);
        let filtered_items = inventory
            .items
            .iter()
            .filter(|item| filter_row(&item.row_id))
            .cloned()
            .collect::<Vec<_>>();
        let filtered_inventory = OfficialReadyRowInventoryReport {
            inventory_id: inventory.inventory_id.clone(),
            items: filtered_items.clone(),
            total_items: filtered_items.len(),
            official_ready_match_count: filtered_items
                .iter()
                .filter(|item| item.official_ready_match)
                .count(),
            benchmark_ready_match_count: filtered_items
                .iter()
                .filter(|item| item.benchmark_ready_match)
                .count(),
            complete_comparable_row_count: filtered_items
                .iter()
                .filter(|item| {
                    item.has_outcome_reference
                        && item.has_baseline_reference
                        && item.has_no_trade_counterfactual
                        && item.has_risk_denied_counterfactual
                })
                .count(),
            incomplete_row_count: filtered_items
                .iter()
                .filter(|item| {
                    !(item.has_outcome_reference
                        && item.has_baseline_reference
                        && item.has_no_trade_counterfactual
                        && item.has_risk_denied_counterfactual)
                })
                .count(),
            missing_outcome_count: filtered_items
                .iter()
                .filter(|item| !item.has_outcome_reference)
                .count(),
            missing_baseline_count: filtered_items
                .iter()
                .filter(|item| !item.has_baseline_reference)
                .count(),
            missing_no_trade_count: filtered_items
                .iter()
                .filter(|item| !item.has_no_trade_counterfactual)
                .count(),
            missing_risk_denied_count: filtered_items
                .iter()
                .filter(|item| !item.has_risk_denied_counterfactual)
                .count(),
            missing_scenario_count: filtered_items
                .iter()
                .filter(|item| !item.has_scenario_row)
                .count(),
            summary_derived_only_count: filtered_items
                .iter()
                .filter(|item| item.summary_derived)
                .count(),
            source_ineligible_count: filtered_items
                .iter()
                .filter(|item| {
                    item.source_class != ComparableEvidenceSourceClass::OfficialNonCrypto
                })
                .count(),
            inventory_status: inventory.inventory_status,
            reason_codes: inventory.reason_codes.clone(),
        };
        let filtered_rows = if rows.is_empty() {
            filtered_items
                .iter()
                .map(synthetic_row_from_item)
                .collect::<Vec<_>>()
        } else {
            rows.into_iter()
                .filter(|row| filter_row(&row.row_id))
                .collect::<Vec<_>>()
        };
        let row_map = filtered_rows
            .iter()
            .cloned()
            .map(|row| (row.row_id.clone(), row))
            .collect::<BTreeMap<_, _>>();
        let requirement_report =
            load_requirement_report(config, &filtered_inventory, &descriptors)?;
        let outcome_linkage_report = OutcomeLinkageV3Runner::default().run_from_inputs(
            &OutcomeLinkageV3Config {
                linkage_id: config.batch_id.clone(),
                output_root: config.output_root.clone(),
                extended_candle_pack_paths: config.extended_candle_pack_paths.clone(),
                default_horizon_bars: config.default_horizon_bars,
                take_profit_pct: config.take_profit_pct,
                stop_loss_pct: config.stop_loss_pct,
                cost_bps: config.cost_bps,
                slippage_bps: config.slippage_bps,
                tie_break_policy: config.tie_break_policy,
                require_exact_symbol_match: config.require_exact_symbol_match,
                require_exact_horizon_match: config.require_exact_horizon_match,
                require_no_lookahead_safe: config.require_no_lookahead_safe,
                reason_codes: config.reason_codes.clone(),
                ..OutcomeLinkageV3Config::default()
            },
            &filtered_inventory,
            &requirement_report,
            &descriptors,
            &row_map,
        )?;
        let eligible_rows = filtered_inventory
            .items
            .iter()
            .filter(|item| item.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto)
            .count();
        let take_profit_count = outcome_linkage_report
            .records
            .iter()
            .filter(|record| {
                record.outcome_reference.as_ref().is_some_and(|reference| {
                    matches!(
                        reference.triple_barrier_label,
                        super::committee_outcome_reference::CommitteeTripleBarrierLabel::TakeProfit
                    )
                })
            })
            .count();
        let stop_loss_count = outcome_linkage_report
            .records
            .iter()
            .filter(|record| {
                record.outcome_reference.as_ref().is_some_and(|reference| {
                    matches!(
                        reference.triple_barrier_label,
                        super::committee_outcome_reference::CommitteeTripleBarrierLabel::StopLoss
                    )
                })
            })
            .count();
        let time_expired_count = outcome_linkage_report
            .records
            .iter()
            .filter(|record| {
                record
                    .outcome_reference
                    .as_ref()
                    .is_some_and(|reference| {
                        matches!(
                            reference.triple_barrier_label,
                            super::committee_outcome_reference::CommitteeTripleBarrierLabel::TimeExpired
                        )
                    })
            })
            .count();
        Ok(BatchOutcomeLinkageV3Report {
            batch_id: config.batch_id.clone(),
            records: outcome_linkage_report.records.clone(),
            total_rows: filtered_inventory.items.len(),
            eligible_rows,
            generated_outcome_count: outcome_linkage_report.generated_outcome_count,
            official_outcome_count: outcome_linkage_report.official_outcome_count,
            skipped_missing_future_window_count: outcome_linkage_report.skipped_missing_future_bars,
            skipped_missing_scenario_count: outcome_linkage_report
                .records
                .iter()
                .filter(|record| {
                    matches!(
                        record.status,
                        super::outcome_linkage_v3::OutcomeLinkageV3RecordStatus::SkippedMissingScenario
                    )
                })
                .count(),
            skipped_timestamp_mismatch_count: outcome_linkage_report.skipped_timestamp_mismatch,
            skipped_horizon_mismatch_count: outcome_linkage_report.skipped_horizon_mismatch,
            rejected_no_lookahead_count: outcome_linkage_report.rejected_no_lookahead,
            take_profit_count,
            stop_loss_count,
            time_expired_count,
            linkage_status: map_batch_status(outcome_linkage_report.linkage_status),
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::CommitteeOutcomeReferenceBuilt,
                        ReasonCode::DeterministicPath,
                    ])
                    .collect::<Vec<_>>(),
            ),
        })
    }
}

impl BatchOutcomeLinkageV3Report {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_else(|_| self.batch_id.clone()))
    }

    pub fn to_text(&self) -> String {
        [
            format!("batch_id={}", self.batch_id),
            format!("total_rows={}", self.total_rows),
            format!("eligible_rows={}", self.eligible_rows),
            format!("generated_outcome_count={}", self.generated_outcome_count),
            format!("official_outcome_count={}", self.official_outcome_count),
            format!(
                "skipped_missing_future_window_count={}",
                self.skipped_missing_future_window_count
            ),
            format!(
                "skipped_missing_scenario_count={}",
                self.skipped_missing_scenario_count
            ),
            format!(
                "skipped_timestamp_mismatch_count={}",
                self.skipped_timestamp_mismatch_count
            ),
            format!(
                "skipped_horizon_mismatch_count={}",
                self.skipped_horizon_mismatch_count
            ),
            format!(
                "rejected_no_lookahead_count={}",
                self.rejected_no_lookahead_count
            ),
            format!("take_profit_count={}", self.take_profit_count),
            format!("stop_loss_count={}", self.stop_loss_count),
            format!("time_expired_count={}", self.time_expired_count),
            format!("linkage_status={:?}", self.linkage_status),
            format!("fingerprint={}", self.fingerprint()),
            self.records
                .iter()
                .map(|record| {
                    format!(
                        "row_id={};status={:?};label={};net_return_pct={}",
                        record.row_id,
                        record.status,
                        record
                            .outcome_reference
                            .as_ref()
                            .map(|reference| format!("{:?}", reference.triple_barrier_label))
                            .unwrap_or_default(),
                        record
                            .net_return_pct
                            .map(|value| value.to_string())
                            .unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
        ]
        .join("\n")
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
            output_dir.join("batch_outcome_linkage_v3.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("batch_outcome_linkage_v3_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_batch_outcome_linkage_v3_from_path_or_config(
    path: &str,
) -> Result<BatchOutcomeLinkageV3Report, String> {
    if path.ends_with(".json") {
        BatchOutcomeLinkageV3Report::from_json_path(Path::new(path))
    } else {
        BatchOutcomeLinkageV3Config::from_toml_path(Path::new(path))
            .and_then(|config| BatchOutcomeLinkageV3Runner::default().run(&config))
    }
}

fn load_inventory(
    config: &BatchOutcomeLinkageV3Config,
    rows: &[ComparableCommitteeEvidenceRow],
    descriptors: &BTreeMap<
        String,
        super::official_candle_coverage_pack::OfficialCandleSeriesDescriptor,
    >,
    set: Option<&MultiRowOfficialEvidenceSet>,
) -> Result<OfficialReadyRowInventoryReport, String> {
    if let Some(path) = config.official_ready_inventory_paths.first() {
        if path.ends_with(".json") {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            return serde_json::from_str(&text).map_err(|err| err.to_string());
        }
        let inventory_config = OfficialReadyRowInventoryConfig::from_toml_path(Path::new(path))?;
        return OfficialReadyRowInventoryRunner::default().run(&inventory_config);
    }
    if let Some(set) = set {
        return inventory_from_set(config, set);
    }
    OfficialReadyRowInventoryRunner::default().run_from_rows(
        &OfficialReadyRowInventoryConfig {
            inventory_id: format!("{}-inventory", config.batch_id),
            output_root: config.output_root.clone(),
            allow_crypto_only: true,
            allow_controlled_diagnostic: true,
            allow_yfinance_research: true,
            allow_fixture: true,
            ..OfficialReadyRowInventoryConfig::default()
        },
        rows,
        &BTreeMap::new(),
        descriptors,
    )
}

fn load_rows(
    config: &BatchOutcomeLinkageV3Config,
    set: Option<&MultiRowOfficialEvidenceSet>,
) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    if !config.comparable_evidence_bundle_paths.is_empty() {
        return load_rows_from_paths(&config.comparable_evidence_bundle_paths);
    }
    Ok(set
        .map(|set| {
            set.items
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
                    net_return_pct: item.net_return_pct,
                    cost_bps: default_cost_bps(),
                    slippage_bps: default_slippage_bps(),
                    committee_vs_baseline_delta: None,
                    committee_vs_notrade_delta: None,
                    risk_denied_value_proxy: item.risk_denied_value_proxy,
                    no_trade_value_proxy: item.no_trade_value_proxy,
                    outcome_reference_available: item.outcome_reference_available,
                    baseline_reference_available: item.baseline_reference_available,
                    no_trade_counterfactual_available: item.no_trade_counterfactual_available,
                    risk_denied_counterfactual_available: item.risk_denied_counterfactual_available,
                    external_reference_available: false,
                    row_level: item.row_level,
                    summary_derived: item.summary_derived,
                    no_lookahead_safe: item.no_lookahead_safe,
                    official_readiness_eligible: item.source_class
                        == ComparableEvidenceSourceClass::OfficialNonCrypto,
                    diagnostic_only: item.diagnostic_only,
                    candle_coverage_available: item.has_local_candle_window,
                    matched_candle_series_id: item.candle_series_id.clone(),
                    candle_match_status: Some("Matched".to_string()),
                    candle_official_ready_match: item.official_ready_match,
                    candle_benchmark_ready_match: item.benchmark_ready_match,
                    candle_diagnostic_only: item.diagnostic_only,
                    reason_codes: item.reason_codes.clone(),
                })
                .collect()
        })
        .unwrap_or_default())
}

fn inventory_from_set(
    config: &BatchOutcomeLinkageV3Config,
    set: &MultiRowOfficialEvidenceSet,
) -> Result<OfficialReadyRowInventoryReport, String> {
    let rows = load_rows(config, Some(set))?;
    OfficialReadyRowInventoryRunner::default().run_from_rows(
        &OfficialReadyRowInventoryConfig {
            inventory_id: format!("{}-inventory", config.batch_id),
            output_root: config.output_root.clone(),
            allow_crypto_only: true,
            allow_controlled_diagnostic: true,
            allow_yfinance_research: true,
            allow_fixture: true,
            ..OfficialReadyRowInventoryConfig::default()
        },
        &rows,
        &BTreeMap::new(),
        &load_descriptor_map_from_paths(&config.extended_candle_pack_paths)?,
    )
}

fn load_requirement_report(
    config: &BatchOutcomeLinkageV3Config,
    inventory: &OfficialReadyRowInventoryReport,
    descriptors: &BTreeMap<
        String,
        super::official_candle_coverage_pack::OfficialCandleSeriesDescriptor,
    >,
) -> Result<super::future_window_requirements::FutureWindowRequirementReport, String> {
    if let Some(path) = config.future_window_scaleout_plan_path.as_deref() {
        let plan: FutureWindowScaleOutPlan =
            load_future_window_scaleout_plan_from_path_or_config(path)?;
        return Ok(plan.requirement_report);
    }
    FutureWindowRequirementRunner::default().run_from_inventory(
        &FutureWindowRequirementConfig {
            requirement_id: format!("{}-requirements", config.batch_id),
            output_root: config.output_root.clone(),
            max_rows: inventory.items.len().max(1).min(500),
            max_symbols: inventory
                .items
                .iter()
                .map(|item| item.symbol.clone())
                .collect::<BTreeSet<_>>()
                .len()
                .max(1)
                .min(5),
            candle_coverage_pack_paths: config.extended_candle_pack_paths.clone(),
            ..FutureWindowRequirementConfig::default()
        },
        inventory,
        descriptors,
    )
}

fn map_batch_status(status: OutcomeLinkageV3Status) -> BatchOutcomeLinkageV3Status {
    match status {
        OutcomeLinkageV3Status::OutcomeLinksImproved => {
            BatchOutcomeLinkageV3Status::BatchOutcomeLinksImproved
        }
        OutcomeLinkageV3Status::OfficialOutcomeLinksImproved => {
            BatchOutcomeLinkageV3Status::OfficialBatchOutcomeLinksImproved
        }
        OutcomeLinkageV3Status::StillNeedFutureBars => {
            BatchOutcomeLinkageV3Status::StillNeedFutureWindows
        }
        OutcomeLinkageV3Status::StillNeedScenarioRows => {
            BatchOutcomeLinkageV3Status::StillNeedScenarioRows
        }
        OutcomeLinkageV3Status::StillTimestampMismatch => {
            BatchOutcomeLinkageV3Status::StillTimestampMismatch
        }
        OutcomeLinkageV3Status::StillHorizonMismatch => {
            BatchOutcomeLinkageV3Status::StillHorizonMismatch
        }
        OutcomeLinkageV3Status::SourceIneligible => BatchOutcomeLinkageV3Status::SourceIneligible,
        OutcomeLinkageV3Status::DiagnosticOnly => BatchOutcomeLinkageV3Status::DiagnosticOnly,
        OutcomeLinkageV3Status::NoImprovement => BatchOutcomeLinkageV3Status::NoImprovement,
    }
}

fn synthetic_row_from_item(
    item: &super::official_ready_row_inventory::OfficialReadyRowInventoryItem,
) -> ComparableCommitteeEvidenceRow {
    ComparableCommitteeEvidenceRow {
        row_id: item.row_id.clone(),
        symbol: item.symbol.clone(),
        market: item.market,
        timeframe: item.timeframe.clone(),
        horizon_bars: item.horizon_bars,
        timestamp_ms: item.timestamp_ms,
        source_kind: item.source_kind.clone(),
        source_class: item.source_class,
        scenario_row_id: item.scenario_row_id.clone(),
        committee_decision_id: Some(format!("committee-{}", item.row_id)),
        committee_final_action: "Approve".to_string(),
        chair_decision: Some("Approve".to_string()),
        risk_governor_decision: Some("Reject".to_string()),
        baseline_action: Some("Approve".to_string()),
        external_action: None,
        no_trade_baseline_action: "NoTrade".to_string(),
        outcome_label: None,
        net_return_pct: None,
        cost_bps: default_cost_bps(),
        slippage_bps: default_slippage_bps(),
        committee_vs_baseline_delta: None,
        committee_vs_notrade_delta: None,
        risk_denied_value_proxy: None,
        no_trade_value_proxy: None,
        outcome_reference_available: item.has_outcome_reference,
        baseline_reference_available: item.has_baseline_reference,
        no_trade_counterfactual_available: item.has_no_trade_counterfactual,
        risk_denied_counterfactual_available: item.has_risk_denied_counterfactual,
        external_reference_available: false,
        row_level: item.row_level,
        summary_derived: item.summary_derived,
        no_lookahead_safe: item.no_lookahead_safe,
        official_readiness_eligible: item.source_class
            == ComparableEvidenceSourceClass::OfficialNonCrypto,
        diagnostic_only: item.source_class != ComparableEvidenceSourceClass::OfficialNonCrypto,
        candle_coverage_available: item.candle_series_id.is_some(),
        matched_candle_series_id: item.candle_series_id.clone(),
        candle_match_status: Some("Matched".to_string()),
        candle_official_ready_match: item.official_ready_match,
        candle_benchmark_ready_match: item.benchmark_ready_match,
        candle_diagnostic_only: item.source_class
            != ComparableEvidenceSourceClass::OfficialNonCrypto,
        reason_codes: item.reason_codes.clone(),
    }
}

fn default_output_root() -> String {
    "target/soma_batch_outcome_linkage_v3".to_string()
}

fn default_horizon_bars() -> usize {
    3
}

fn default_take_profit_pct() -> f64 {
    0.02
}

fn default_stop_loss_pct() -> f64 {
    0.01
}

fn default_cost_bps() -> f64 {
    5.0
}

fn default_slippage_bps() -> f64 {
    2.0
}

fn default_true() -> bool {
    true
}
