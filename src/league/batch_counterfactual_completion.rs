use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::OfficialReadyRowInventoryConfig;
use super::batch_outcome_linkage_v3::{
    BatchOutcomeLinkageV3Report, load_batch_outcome_linkage_v3_from_path_or_config,
};
use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};
use super::counterfactual_completion_v2::{
    CounterfactualCompletionRecord, CounterfactualCompletionV2Config,
    CounterfactualCompletionV2Runner,
};
use super::future_window_requirements::load_rows_from_paths;
use super::multi_row_official_evidence::{
    MultiRowOfficialEvidenceSet, load_multi_row_official_evidence_set_from_path_or_config,
};
use super::official_ready_row_inventory::{
    OfficialReadyRowInventoryReport, OfficialReadyRowInventoryRunner,
};
use super::outcome_linkage_v3::OutcomeLinkageV3Report;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchCounterfactualCompletionConfig {
    pub batch_id: String,
    #[serde(default)]
    pub multi_row_set_config_path: Option<String>,
    #[serde(default)]
    pub outcome_linkage_batch_path: Option<String>,
    #[serde(default)]
    pub batch_outcome_linkage_path: Option<String>,
    #[serde(default)]
    pub outcome_linkage_v3_path: Option<String>,
    #[serde(default)]
    pub official_ready_inventory_paths: Vec<String>,
    #[serde(default)]
    pub comparable_evidence_bundle_paths: Vec<String>,
    #[serde(default)]
    pub committee_decision_paths: Vec<String>,
    #[serde(default)]
    pub risk_decision_paths: Vec<String>,
    #[serde(default)]
    pub include_row_ids: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub build_no_trade: bool,
    #[serde(default = "default_true")]
    pub build_risk_denied: bool,
    #[serde(default = "default_credit_avoided_loss_factor")]
    pub credit_avoided_loss_factor: f64,
    #[serde(default = "default_penalize_missed_gain_factor")]
    pub penalize_missed_gain_factor: f64,
    #[serde(default = "default_max_missed_gain_penalty")]
    pub max_missed_gain_penalty: f64,
    #[serde(default = "default_true")]
    pub require_outcome_reference: bool,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BatchCounterfactualCompletionStatus {
    BatchCounterfactualsImproved,
    OfficialBatchCounterfactualsImproved,
    StillNeedOutcomeReferences,
    StillNeedRiskDecisions,
    StillNeedCommitteeDecisions,
    SourceIneligible,
    DiagnosticOnly,
    #[default]
    NoImprovement,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchCounterfactualCompletionReport {
    pub batch_id: String,
    pub records: Vec<CounterfactualCompletionRecord>,
    pub total_rows: usize,
    pub eligible_rows: usize,
    pub completed_count: usize,
    pub no_trade_built_count: usize,
    pub risk_denied_built_count: usize,
    pub official_counterfactual_count: usize,
    pub diagnostic_counterfactual_count: usize,
    pub skipped_missing_outcome_count: usize,
    pub skipped_missing_risk_decision_count: usize,
    pub skipped_missing_committee_decision_count: usize,
    pub avoided_loss_total: f64,
    pub missed_gain_total: f64,
    pub completion_status: BatchCounterfactualCompletionStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BatchCounterfactualCompletionRunner;

impl Default for BatchCounterfactualCompletionConfig {
    fn default() -> Self {
        Self {
            batch_id: "batch-counterfactual-completion".to_string(),
            multi_row_set_config_path: None,
            outcome_linkage_batch_path: None,
            batch_outcome_linkage_path: None,
            outcome_linkage_v3_path: None,
            official_ready_inventory_paths: Vec::new(),
            comparable_evidence_bundle_paths: Vec::new(),
            committee_decision_paths: Vec::new(),
            risk_decision_paths: Vec::new(),
            include_row_ids: Vec::new(),
            output_root: default_output_root(),
            build_no_trade: true,
            build_risk_denied: true,
            credit_avoided_loss_factor: default_credit_avoided_loss_factor(),
            penalize_missed_gain_factor: default_penalize_missed_gain_factor(),
            max_missed_gain_penalty: default_max_missed_gain_penalty(),
            require_outcome_reference: true,
            require_no_lookahead_safe: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl BatchCounterfactualCompletionConfig {
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
            return Err("batch counterfactual completion id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("batch counterfactual completion paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.batch_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.multi_row_set_config_path
            .iter()
            .chain(self.outcome_linkage_batch_path.iter())
            .chain(self.batch_outcome_linkage_path.iter())
            .chain(self.outcome_linkage_v3_path.iter())
            .chain(self.official_ready_inventory_paths.iter())
            .chain(self.comparable_evidence_bundle_paths.iter())
            .chain(self.committee_decision_paths.iter())
            .chain(self.risk_decision_paths.iter())
            .cloned()
            .collect()
    }
}

impl BatchCounterfactualCompletionRunner {
    pub fn run(
        &self,
        config: &BatchCounterfactualCompletionConfig,
    ) -> Result<BatchCounterfactualCompletionReport, String> {
        config.validate()?;
        let outcome_report = load_outcome_report(config)?;
        let set = config
            .multi_row_set_config_path
            .as_deref()
            .map(load_multi_row_official_evidence_set_from_path_or_config)
            .transpose()?;
        let _inventory = load_inventory(config, set.as_ref())?;
        let allowed_rows = config
            .include_row_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let filter_row = |row_id: &str| allowed_rows.is_empty() || allowed_rows.contains(row_id);
        let mut rows = load_rows(config, set.as_ref())?
            .into_iter()
            .filter(|row| filter_row(&row.row_id))
            .collect::<Vec<_>>();
        apply_decisions(
            &mut rows,
            &config.committee_decision_paths,
            &config.risk_decision_paths,
        )?;
        rows.sort_by(|left, right| left.row_id.cmp(&right.row_id));
        let counterfactual_report = CounterfactualCompletionV2Runner::default().run_from_inputs(
            &CounterfactualCompletionV2Config {
                completion_id: config.batch_id.clone(),
                output_root: config.output_root.clone(),
                build_no_trade: config.build_no_trade,
                build_risk_denied: config.build_risk_denied,
                credit_avoided_loss_factor: config.credit_avoided_loss_factor,
                penalize_missed_gain_factor: config.penalize_missed_gain_factor,
                max_missed_gain_penalty: config.max_missed_gain_penalty,
                require_outcome_reference: config.require_outcome_reference,
                require_no_lookahead_safe: config.require_no_lookahead_safe,
                reason_codes: config.reason_codes.clone(),
                ..CounterfactualCompletionV2Config::default()
            },
            &outcome_report,
            &rows,
        )?;
        let eligible_rows = rows
            .iter()
            .filter(|row| row.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto)
            .count();
        let avoided_loss_total = counterfactual_report
            .records
            .iter()
            .filter_map(|record| record.avoided_loss_value)
            .sum::<f64>();
        let missed_gain_total = counterfactual_report
            .records
            .iter()
            .filter_map(|record| record.missed_gain_value)
            .sum::<f64>();
        Ok(BatchCounterfactualCompletionReport {
            batch_id: config.batch_id.clone(),
            records: counterfactual_report.records.clone(),
            total_rows: rows.len(),
            eligible_rows,
            completed_count: counterfactual_report.completed_count,
            no_trade_built_count: counterfactual_report.no_trade_built_count,
            risk_denied_built_count: counterfactual_report.risk_denied_built_count,
            official_counterfactual_count: counterfactual_report.official_counterfactual_count,
            diagnostic_counterfactual_count: counterfactual_report.diagnostic_counterfactual_count,
            skipped_missing_outcome_count: counterfactual_report.skipped_missing_outcome_count,
            skipped_missing_risk_decision_count: counterfactual_report
                .skipped_missing_risk_decision_count,
            skipped_missing_committee_decision_count: counterfactual_report
                .records
                .iter()
                .filter(|record| {
                    matches!(
                        record.status,
                        super::counterfactual_completion_v2::CounterfactualCompletionV2RecordStatus::SkippedMissingCommitteeDecision
                    )
                })
                .count(),
            avoided_loss_total,
            missed_gain_total,
            completion_status: map_batch_status(counterfactual_report.completion_status),
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::CounterfactualEvaluated,
                        ReasonCode::DeterministicPath,
                    ])
                    .collect::<Vec<_>>(),
            ),
        })
    }
}

impl BatchCounterfactualCompletionReport {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_else(|_| self.batch_id.clone()))
    }

    pub fn to_text(&self) -> String {
        [
            format!("batch_id={}", self.batch_id),
            format!("total_rows={}", self.total_rows),
            format!("eligible_rows={}", self.eligible_rows),
            format!("completed_count={}", self.completed_count),
            format!("no_trade_built_count={}", self.no_trade_built_count),
            format!("risk_denied_built_count={}", self.risk_denied_built_count),
            format!(
                "official_counterfactual_count={}",
                self.official_counterfactual_count
            ),
            format!(
                "diagnostic_counterfactual_count={}",
                self.diagnostic_counterfactual_count
            ),
            format!(
                "skipped_missing_outcome_count={}",
                self.skipped_missing_outcome_count
            ),
            format!(
                "skipped_missing_risk_decision_count={}",
                self.skipped_missing_risk_decision_count
            ),
            format!(
                "skipped_missing_committee_decision_count={}",
                self.skipped_missing_committee_decision_count
            ),
            format!("avoided_loss_total={}", self.avoided_loss_total),
            format!("missed_gain_total={}", self.missed_gain_total),
            format!("completion_status={:?}", self.completion_status),
            format!("fingerprint={}", self.fingerprint()),
            self.records
                .iter()
                .map(|record| {
                    format!(
                        "row_id={};status={:?};no_trade_counterfactual_built={};risk_denied_counterfactual_built={};avoided_loss_value={};missed_gain_value={}",
                        record.row_id,
                        record.status,
                        record.no_trade_counterfactual_built,
                        record.risk_denied_counterfactual_built,
                        record
                            .avoided_loss_value
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        record
                            .missed_gain_value
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
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
            output_dir.join("batch_counterfactual_completion.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("batch_counterfactual_completion_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_batch_counterfactual_completion_from_path_or_config(
    path: &str,
) -> Result<BatchCounterfactualCompletionReport, String> {
    if path.ends_with(".json") {
        BatchCounterfactualCompletionReport::from_json_path(Path::new(path))
    } else {
        BatchCounterfactualCompletionConfig::from_toml_path(Path::new(path))
            .and_then(|config| BatchCounterfactualCompletionRunner::default().run(&config))
    }
}

fn load_outcome_report(
    config: &BatchCounterfactualCompletionConfig,
) -> Result<OutcomeLinkageV3Report, String> {
    if let Some(path) = config
        .outcome_linkage_batch_path
        .as_deref()
        .or(config.batch_outcome_linkage_path.as_deref())
    {
        let report: BatchOutcomeLinkageV3Report =
            load_batch_outcome_linkage_v3_from_path_or_config(path)?;
        let diagnostic_outcome_count = report
            .records
            .iter()
            .filter(|record| {
                matches!(
                    record.status,
                    super::outcome_linkage_v3::OutcomeLinkageV3RecordStatus::DiagnosticOnly
                )
            })
            .count();
        let records = report.records.clone();
        return Ok(OutcomeLinkageV3Report {
            linkage_id: report.batch_id.clone(),
            records,
            generated_outcome_count: report.generated_outcome_count,
            skipped_missing_future_bars: report.skipped_missing_future_window_count,
            skipped_timestamp_mismatch: report.skipped_timestamp_mismatch_count,
            skipped_horizon_mismatch: report.skipped_horizon_mismatch_count,
            rejected_no_lookahead: report.rejected_no_lookahead_count,
            official_outcome_count: report.official_outcome_count,
            diagnostic_outcome_count,
            linkage_status: match report.linkage_status {
                super::batch_outcome_linkage_v3::BatchOutcomeLinkageV3Status::BatchOutcomeLinksImproved => {
                    super::outcome_linkage_v3::OutcomeLinkageV3Status::OutcomeLinksImproved
                }
                super::batch_outcome_linkage_v3::BatchOutcomeLinkageV3Status::OfficialBatchOutcomeLinksImproved => {
                    super::outcome_linkage_v3::OutcomeLinkageV3Status::OfficialOutcomeLinksImproved
                }
                super::batch_outcome_linkage_v3::BatchOutcomeLinkageV3Status::StillNeedFutureWindows => {
                    super::outcome_linkage_v3::OutcomeLinkageV3Status::StillNeedFutureBars
                }
                super::batch_outcome_linkage_v3::BatchOutcomeLinkageV3Status::StillNeedScenarioRows => {
                    super::outcome_linkage_v3::OutcomeLinkageV3Status::StillNeedScenarioRows
                }
                super::batch_outcome_linkage_v3::BatchOutcomeLinkageV3Status::StillTimestampMismatch => {
                    super::outcome_linkage_v3::OutcomeLinkageV3Status::StillTimestampMismatch
                }
                super::batch_outcome_linkage_v3::BatchOutcomeLinkageV3Status::StillHorizonMismatch => {
                    super::outcome_linkage_v3::OutcomeLinkageV3Status::StillHorizonMismatch
                }
                super::batch_outcome_linkage_v3::BatchOutcomeLinkageV3Status::SourceIneligible => {
                    super::outcome_linkage_v3::OutcomeLinkageV3Status::SourceIneligible
                }
                super::batch_outcome_linkage_v3::BatchOutcomeLinkageV3Status::DiagnosticOnly => {
                    super::outcome_linkage_v3::OutcomeLinkageV3Status::DiagnosticOnly
                }
                super::batch_outcome_linkage_v3::BatchOutcomeLinkageV3Status::NoImprovement => {
                    super::outcome_linkage_v3::OutcomeLinkageV3Status::NoImprovement
                }
            },
            reason_codes: report.reason_codes,
        });
    }
    if let Some(path) = config.outcome_linkage_v3_path.as_deref() {
        if path.ends_with(".json") {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            return serde_json::from_str(&text).map_err(|err| err.to_string());
        }
        let config =
            super::outcome_linkage_v3::OutcomeLinkageV3Config::from_toml_path(Path::new(path))?;
        return super::outcome_linkage_v3::OutcomeLinkageV3Runner::default().run(&config);
    }
    Err(
        "batch counterfactual completion requires a batch outcome linkage or outcome linkage path"
            .to_string(),
    )
}

fn load_inventory(
    config: &BatchCounterfactualCompletionConfig,
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
    let rows = load_rows(config, set)?;
    OfficialReadyRowInventoryRunner::default().run_from_rows(
        &OfficialReadyRowInventoryConfig {
            inventory_id: format!("{}-inventory", config.batch_id),
            output_root: config.output_root.clone(),
            allow_controlled_diagnostic: true,
            allow_crypto_only: true,
            allow_yfinance_research: true,
            allow_fixture: true,
            ..OfficialReadyRowInventoryConfig::default()
        },
        &rows,
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
}

fn load_rows(
    config: &BatchCounterfactualCompletionConfig,
    set: Option<&MultiRowOfficialEvidenceSet>,
) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    if !config.comparable_evidence_bundle_paths.is_empty() {
        return load_rows_from_paths(&config.comparable_evidence_bundle_paths);
    }
    Ok(set
        .map(|set| set.items.iter().map(synthetic_row_from_set_item).collect())
        .unwrap_or_default())
}

fn synthetic_row_from_set_item(
    item: &super::multi_row_official_evidence::MultiRowOfficialEvidenceItem,
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
        chair_decision: item
            .committee_decision_available
            .then(|| "Approve".to_string()),
        risk_governor_decision: item.risk_decision_available.then(|| "Reject".to_string()),
        baseline_action: item
            .baseline_reference_available
            .then(|| "Approve".to_string()),
        external_action: None,
        no_trade_baseline_action: "NoTrade".to_string(),
        outcome_label: None,
        net_return_pct: item.net_return_pct,
        cost_bps: 5.0,
        slippage_bps: 2.0,
        committee_vs_baseline_delta: None,
        committee_vs_notrade_delta: None,
        risk_denied_value_proxy: item.risk_denied_value_proxy,
        no_trade_value_proxy: item.no_trade_value_proxy,
        outcome_reference_available: item.outcome_reference_available,
        baseline_reference_available: item.baseline_reference_available,
        no_trade_counterfactual_available: false,
        risk_denied_counterfactual_available: false,
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
    }
}

fn apply_decisions(
    rows: &mut [ComparableCommitteeEvidenceRow],
    committee_paths: &[String],
    risk_paths: &[String],
) -> Result<(), String> {
    let committee_map = load_decision_map(
        committee_paths,
        "committee_decisions",
        "committee_final_action",
    )?;
    let chair_map = load_decision_map(committee_paths, "committee_decisions", "chair_decision")?;
    let risk_map = load_decision_map(risk_paths, "risk_decisions", "risk_governor_decision")?;
    for row in rows {
        if let Some(action) = committee_map.get(&row.row_id) {
            row.committee_final_action = action.clone();
        }
        if let Some(chair) = chair_map.get(&row.row_id) {
            row.chair_decision = Some(chair.clone());
        }
        if let Some(risk) = risk_map.get(&row.row_id) {
            row.risk_governor_decision = Some(risk.clone());
        }
    }
    Ok(())
}

fn load_decision_map(
    paths: &[String],
    root_key: &str,
    field_key: &str,
) -> Result<BTreeMap<String, String>, String> {
    let mut map = BTreeMap::new();
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let value: Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;
        let entries = value
            .get(root_key)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for entry in entries {
            if let (Some(row_id), Some(field)) = (
                entry.get("row_id").and_then(Value::as_str),
                entry.get(field_key).and_then(Value::as_str),
            ) {
                map.insert(row_id.to_string(), field.to_string());
            }
        }
    }
    Ok(map)
}

fn default_output_root() -> String {
    "target/soma_batch_counterfactual_completion".to_string()
}

fn map_batch_status(
    status: super::counterfactual_completion_v2::CounterfactualCompletionV2Status,
) -> BatchCounterfactualCompletionStatus {
    match status {
        super::counterfactual_completion_v2::CounterfactualCompletionV2Status::CounterfactualsImproved => {
            BatchCounterfactualCompletionStatus::BatchCounterfactualsImproved
        }
        super::counterfactual_completion_v2::CounterfactualCompletionV2Status::OfficialCounterfactualsImproved => {
            BatchCounterfactualCompletionStatus::OfficialBatchCounterfactualsImproved
        }
        super::counterfactual_completion_v2::CounterfactualCompletionV2Status::StillNeedOutcomeReferences => {
            BatchCounterfactualCompletionStatus::StillNeedOutcomeReferences
        }
        super::counterfactual_completion_v2::CounterfactualCompletionV2Status::StillNeedRiskDecisions => {
            BatchCounterfactualCompletionStatus::StillNeedRiskDecisions
        }
        super::counterfactual_completion_v2::CounterfactualCompletionV2Status::StillNeedCommitteeDecisions => {
            BatchCounterfactualCompletionStatus::StillNeedCommitteeDecisions
        }
        super::counterfactual_completion_v2::CounterfactualCompletionV2Status::SourceIneligible => {
            BatchCounterfactualCompletionStatus::SourceIneligible
        }
        super::counterfactual_completion_v2::CounterfactualCompletionV2Status::DiagnosticOnly => {
            BatchCounterfactualCompletionStatus::DiagnosticOnly
        }
        super::counterfactual_completion_v2::CounterfactualCompletionV2Status::NoImprovement => {
            BatchCounterfactualCompletionStatus::NoImprovement
        }
    }
}

fn default_credit_avoided_loss_factor() -> f64 {
    1.0
}

fn default_penalize_missed_gain_factor() -> f64 {
    1.0
}

fn default_max_missed_gain_penalty() -> f64 {
    1.0
}

fn default_true() -> bool {
    true
}
