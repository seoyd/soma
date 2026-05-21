use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};
use super::comparable_evidence_builder::ComparableEvidenceBuilder;
use super::official_candle_coverage_pack::{
    OfficialCandleSeriesDescriptor, load_pack_from_path_or_config,
};
use super::official_committee_pack::{
    OfficialCommitteeScenarioPack, OfficialCommitteeScenarioPackConfig,
};
use super::{CommitteeScenarioRow, OfficialCommitteeScenarioPackBuilder};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialReadyRowInventoryConfig {
    pub inventory_id: String,
    #[serde(default)]
    pub official_ready_match_closure_paths: Vec<String>,
    #[serde(default)]
    pub candle_join_audit_paths: Vec<String>,
    #[serde(default)]
    pub comparable_evidence_bundle_paths: Vec<String>,
    #[serde(default)]
    pub comparable_backfill_report_paths: Vec<String>,
    #[serde(default)]
    pub candle_coverage_match_report_paths: Vec<String>,
    #[serde(default)]
    pub official_candle_coverage_pack_paths: Vec<String>,
    #[serde(default)]
    pub scenario_pack_paths: Vec<String>,
    #[serde(default)]
    pub reference_pack_paths: Vec<String>,
    #[serde(default)]
    pub outcome_coverage_bundle_paths: Vec<String>,
    #[serde(default)]
    pub core_scorecard_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_official_ready_match: bool,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default)]
    pub allow_controlled_diagnostic: bool,
    #[serde(default = "default_true")]
    pub allow_crypto_only: bool,
    #[serde(default)]
    pub allow_yfinance_research: bool,
    #[serde(default)]
    pub allow_fixture: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialReadyRowCompletenessStatus {
    CompleteComparableRow,
    MissingScenarioRow,
    MissingOutcomeReference,
    MissingBaselineReference,
    MissingNoTradeCounterfactual,
    MissingRiskDeniedCounterfactual,
    MissingExternalReference,
    MissingCommitteeDecision,
    MissingChairDecision,
    MissingRiskDecision,
    MissingFeatureSummary,
    SummaryDerivedOnly,
    SourceIneligible,
    NoLookaheadViolation,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialReadyRowInventoryItem {
    pub row_id: String,
    #[serde(default)]
    pub scenario_row_id: Option<String>,
    #[serde(default)]
    pub comparable_row_id: Option<String>,
    #[serde(default)]
    pub candle_series_id: Option<String>,
    pub symbol: String,
    pub market: crate::data::ProviderMarket,
    #[serde(default)]
    pub venue: Option<String>,
    pub timeframe: String,
    pub horizon_bars: usize,
    pub timestamp_ms: u64,
    pub source_kind: String,
    pub source_class: ComparableEvidenceSourceClass,
    pub official_ready_match: bool,
    pub benchmark_ready_match: bool,
    pub row_level: bool,
    pub summary_derived: bool,
    pub no_lookahead_safe: bool,
    pub has_scenario_row: bool,
    pub has_committee_decision: bool,
    pub has_chair_decision: bool,
    pub has_risk_decision: bool,
    pub has_outcome_reference: bool,
    pub has_baseline_reference: bool,
    pub has_no_trade_counterfactual: bool,
    pub has_risk_denied_counterfactual: bool,
    pub has_external_reference: bool,
    pub completeness_statuses: Vec<OfficialReadyRowCompletenessStatus>,
    pub buildable_from_available_artifacts: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialReadyRowInventoryStatus {
    HealthyCompleteRows,
    NeedScenarioMaterialization,
    NeedOutcomeReferences,
    NeedBaselineReferences,
    NeedCounterfactuals,
    NeedCommitteeDecisions,
    DiagnosticOnly,
    #[default]
    InsufficientRows,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialReadyRowInventoryReport {
    pub inventory_id: String,
    pub items: Vec<OfficialReadyRowInventoryItem>,
    pub total_items: usize,
    pub official_ready_match_count: usize,
    pub benchmark_ready_match_count: usize,
    pub complete_comparable_row_count: usize,
    pub incomplete_row_count: usize,
    pub missing_outcome_count: usize,
    pub missing_baseline_count: usize,
    pub missing_no_trade_count: usize,
    pub missing_risk_denied_count: usize,
    pub missing_scenario_count: usize,
    pub summary_derived_only_count: usize,
    pub source_ineligible_count: usize,
    pub inventory_status: OfficialReadyRowInventoryStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialReadyRowInventoryRunner;

impl Default for OfficialReadyRowInventoryConfig {
    fn default() -> Self {
        Self {
            inventory_id: "official-ready-row-inventory".to_string(),
            official_ready_match_closure_paths: Vec::new(),
            candle_join_audit_paths: Vec::new(),
            comparable_evidence_bundle_paths: Vec::new(),
            comparable_backfill_report_paths: Vec::new(),
            candle_coverage_match_report_paths: Vec::new(),
            official_candle_coverage_pack_paths: Vec::new(),
            scenario_pack_paths: Vec::new(),
            reference_pack_paths: Vec::new(),
            outcome_coverage_bundle_paths: Vec::new(),
            core_scorecard_paths: Vec::new(),
            output_root: default_output_root(),
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_bytes: default_max_bytes(),
            require_official_ready_match: true,
            require_no_lookahead_safe: true,
            allow_controlled_diagnostic: false,
            allow_crypto_only: true,
            allow_yfinance_research: false,
            allow_fixture: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialReadyRowInventoryConfig {
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
        if self.inventory_id.trim().is_empty() {
            return Err("official ready row inventory id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("official ready row inventory paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err(
                "official ready row inventory max_rows must be between 1 and 500".to_string(),
            );
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err(
                "official ready row inventory max_symbols must be between 1 and 5".to_string(),
            );
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err(
                "official ready row inventory max_bytes must be between 1 and 5000000".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.inventory_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.official_ready_match_closure_paths
            .iter()
            .chain(self.candle_join_audit_paths.iter())
            .chain(self.comparable_evidence_bundle_paths.iter())
            .chain(self.comparable_backfill_report_paths.iter())
            .chain(self.candle_coverage_match_report_paths.iter())
            .chain(self.official_candle_coverage_pack_paths.iter())
            .chain(self.scenario_pack_paths.iter())
            .chain(self.reference_pack_paths.iter())
            .chain(self.outcome_coverage_bundle_paths.iter())
            .chain(self.core_scorecard_paths.iter())
            .cloned()
            .collect()
    }
}

impl OfficialReadyRowInventoryRunner {
    pub fn run(
        &self,
        config: &OfficialReadyRowInventoryConfig,
    ) -> Result<OfficialReadyRowInventoryReport, String> {
        config.validate()?;
        let bundles = load_comparable_bundles(config)?;
        let rows = bundles
            .iter()
            .flat_map(|bundle| bundle.rows.clone())
            .collect::<Vec<_>>();
        let scenario_rows = load_scenario_rows(config)?;
        let descriptors = load_candle_descriptors(config)?;
        self.run_from_rows(config, &rows, &scenario_rows, &descriptors)
    }

    pub fn run_from_rows(
        &self,
        config: &OfficialReadyRowInventoryConfig,
        rows: &[ComparableCommitteeEvidenceRow],
        scenario_rows: &BTreeMap<String, CommitteeScenarioRow>,
        descriptors: &BTreeMap<String, OfficialCandleSeriesDescriptor>,
    ) -> Result<OfficialReadyRowInventoryReport, String> {
        config.validate()?;
        if rows.len() > config.max_rows {
            return Err(format!(
                "official ready row inventory loaded {} rows which exceeds max_rows {}",
                rows.len(),
                config.max_rows
            ));
        }
        let unique_symbols = rows
            .iter()
            .map(|row| row.symbol.clone())
            .collect::<BTreeSet<_>>();
        if unique_symbols.len() > config.max_symbols {
            return Err(format!(
                "official ready row inventory loaded {} symbols which exceeds max_symbols {}",
                unique_symbols.len(),
                config.max_symbols
            ));
        }
        let storage_bytes = rows_storage_bytes(rows).max(input_storage_bytes(&config.all_paths()));
        if storage_bytes > config.max_bytes {
            return Err(format!(
                "official ready row inventory input size {} exceeds max_bytes {}",
                storage_bytes, config.max_bytes
            ));
        }
        let mut items = rows
            .iter()
            .filter(|row| {
                if config.require_official_ready_match {
                    row.candle_official_ready_match
                } else {
                    row.candle_official_ready_match || row.candle_benchmark_ready_match
                }
            })
            .map(|row| build_inventory_item(config, row, scenario_rows, descriptors))
            .collect::<Vec<_>>();
        items.sort_by(|left, right| {
            left.row_id
                .cmp(&right.row_id)
                .then(left.symbol.cmp(&right.symbol))
                .then(left.timestamp_ms.cmp(&right.timestamp_ms))
                .then(left.candle_series_id.cmp(&right.candle_series_id))
        });
        let total_items = items.len();
        let official_ready_match_count = items
            .iter()
            .filter(|item| item.official_ready_match)
            .count();
        let benchmark_ready_match_count = items
            .iter()
            .filter(|item| item.benchmark_ready_match)
            .count();
        let complete_comparable_row_count = items
            .iter()
            .filter(|item| {
                item.completeness_statuses
                    == vec![OfficialReadyRowCompletenessStatus::CompleteComparableRow]
            })
            .count();
        let incomplete_row_count = total_items.saturating_sub(complete_comparable_row_count);
        let missing_outcome_count = count_status(
            &items,
            OfficialReadyRowCompletenessStatus::MissingOutcomeReference,
        );
        let missing_baseline_count = count_status(
            &items,
            OfficialReadyRowCompletenessStatus::MissingBaselineReference,
        );
        let missing_no_trade_count = count_status(
            &items,
            OfficialReadyRowCompletenessStatus::MissingNoTradeCounterfactual,
        );
        let missing_risk_denied_count = count_status(
            &items,
            OfficialReadyRowCompletenessStatus::MissingRiskDeniedCounterfactual,
        );
        let missing_scenario_count = count_status(
            &items,
            OfficialReadyRowCompletenessStatus::MissingScenarioRow,
        );
        let summary_derived_only_count = count_status(
            &items,
            OfficialReadyRowCompletenessStatus::SummaryDerivedOnly,
        );
        let source_ineligible_count =
            count_status(&items, OfficialReadyRowCompletenessStatus::SourceIneligible);
        let inventory_status = determine_inventory_status(
            total_items,
            complete_comparable_row_count,
            missing_scenario_count,
            missing_outcome_count,
            missing_baseline_count,
            missing_no_trade_count,
            missing_risk_denied_count,
            source_ineligible_count,
            &items,
        );
        Ok(OfficialReadyRowInventoryReport {
            inventory_id: config.inventory_id.clone(),
            items,
            total_items,
            official_ready_match_count,
            benchmark_ready_match_count,
            complete_comparable_row_count,
            incomplete_row_count,
            missing_outcome_count,
            missing_baseline_count,
            missing_no_trade_count,
            missing_risk_denied_count,
            missing_scenario_count,
            summary_derived_only_count,
            source_ineligible_count,
            inventory_status,
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
}

impl OfficialReadyRowInventoryReport {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(
            &serde_json::to_string(self).unwrap_or_else(|_| self.inventory_id.clone()),
        )
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("inventory_id={}", self.inventory_id),
            format!("total_items={}", self.total_items),
            format!(
                "official_ready_match_count={}",
                self.official_ready_match_count
            ),
            format!(
                "benchmark_ready_match_count={}",
                self.benchmark_ready_match_count
            ),
            format!(
                "complete_comparable_row_count={}",
                self.complete_comparable_row_count
            ),
            format!("incomplete_row_count={}", self.incomplete_row_count),
            format!("missing_outcome_count={}", self.missing_outcome_count),
            format!("missing_baseline_count={}", self.missing_baseline_count),
            format!("missing_no_trade_count={}", self.missing_no_trade_count),
            format!(
                "missing_risk_denied_count={}",
                self.missing_risk_denied_count
            ),
            format!("missing_scenario_count={}", self.missing_scenario_count),
            format!(
                "summary_derived_only_count={}",
                self.summary_derived_only_count
            ),
            format!("source_ineligible_count={}", self.source_ineligible_count),
            format!("inventory_status={:?}", self.inventory_status),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.items.iter().map(|item| {
            format!(
                "row_id={};scenario_row_id={};candle_series_id={};source_class={:?};official_ready_match={};row_level={};summary_derived={};buildable={};statuses={}",
                item.row_id,
                item.scenario_row_id.clone().unwrap_or_default(),
                item.candle_series_id.clone().unwrap_or_default(),
                item.source_class,
                item.official_ready_match,
                item.row_level,
                item.summary_derived,
                item.buildable_from_available_artifacts,
                item.completeness_statuses
                    .iter()
                    .map(|status| format!("{status:?}"))
                    .collect::<Vec<_>>()
                    .join("|"),
            )
        }));
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
            output_dir.join("official_ready_row_inventory.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("official_ready_row_inventory.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn build_inventory_item(
    config: &OfficialReadyRowInventoryConfig,
    row: &ComparableCommitteeEvidenceRow,
    scenario_rows: &BTreeMap<String, CommitteeScenarioRow>,
    descriptors: &BTreeMap<String, OfficialCandleSeriesDescriptor>,
) -> OfficialReadyRowInventoryItem {
    let descriptor = row
        .matched_candle_series_id
        .as_ref()
        .and_then(|id| descriptors.get(id))
        .or_else(|| match_descriptor(row, descriptors));
    let scenario_row = row
        .scenario_row_id
        .as_ref()
        .and_then(|id| scenario_rows.get(id));
    let has_scenario_row = row.scenario_row_id.is_some() || row.row_level || scenario_row.is_some();
    let feature_summary_available = scenario_row
        .map(scenario_feature_available)
        .unwrap_or(row.row_level && !row.summary_derived);
    let has_committee_decision =
        row.committee_decision_id.is_some() || !row.committee_final_action.trim().is_empty();
    let has_chair_decision = row
        .chair_decision
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty());
    let has_risk_decision = row
        .risk_governor_decision
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty());
    let source_allowed = source_allowed(row.source_class, config);
    let external_missing = !row.external_reference_available && row.external_action.is_some();
    let all_required_references = row.outcome_reference_available
        && row.baseline_reference_available
        && row.no_trade_counterfactual_available
        && row.risk_denied_counterfactual_available;
    let mut statuses = Vec::new();
    if !source_allowed {
        statuses.push(OfficialReadyRowCompletenessStatus::SourceIneligible);
    }
    if row.diagnostic_only
        || matches!(
            row.source_class,
            ComparableEvidenceSourceClass::ControlledDiagnostic
                | ComparableEvidenceSourceClass::YFinanceResearch
                | ComparableEvidenceSourceClass::FixtureArchitectureTest
                | ComparableEvidenceSourceClass::SyntheticTest
        )
    {
        statuses.push(OfficialReadyRowCompletenessStatus::DiagnosticOnly);
    }
    if config.require_no_lookahead_safe && !row.no_lookahead_safe {
        statuses.push(OfficialReadyRowCompletenessStatus::NoLookaheadViolation);
    }
    if !has_scenario_row {
        statuses.push(OfficialReadyRowCompletenessStatus::MissingScenarioRow);
    }
    if !has_committee_decision {
        statuses.push(OfficialReadyRowCompletenessStatus::MissingCommitteeDecision);
    }
    if !has_chair_decision {
        statuses.push(OfficialReadyRowCompletenessStatus::MissingChairDecision);
    }
    if !has_risk_decision {
        statuses.push(OfficialReadyRowCompletenessStatus::MissingRiskDecision);
    }
    if !feature_summary_available {
        statuses.push(OfficialReadyRowCompletenessStatus::MissingFeatureSummary);
    }
    if row.summary_derived {
        statuses.push(OfficialReadyRowCompletenessStatus::SummaryDerivedOnly);
    }
    if !row.outcome_reference_available {
        statuses.push(OfficialReadyRowCompletenessStatus::MissingOutcomeReference);
    }
    if !row.baseline_reference_available {
        statuses.push(OfficialReadyRowCompletenessStatus::MissingBaselineReference);
    }
    if !row.no_trade_counterfactual_available {
        statuses.push(OfficialReadyRowCompletenessStatus::MissingNoTradeCounterfactual);
    }
    if !row.risk_denied_counterfactual_available {
        statuses.push(OfficialReadyRowCompletenessStatus::MissingRiskDeniedCounterfactual);
    }
    if external_missing {
        statuses.push(OfficialReadyRowCompletenessStatus::MissingExternalReference);
    }
    let complete = source_allowed
        && row.no_lookahead_safe
        && has_scenario_row
        && has_committee_decision
        && has_chair_decision
        && has_risk_decision
        && feature_summary_available
        && !row.summary_derived
        && all_required_references;
    if complete {
        statuses.clear();
        statuses.push(OfficialReadyRowCompletenessStatus::CompleteComparableRow);
    } else if statuses.is_empty() {
        statuses.push(OfficialReadyRowCompletenessStatus::DiagnosticOnly);
    }
    statuses.sort();
    statuses.dedup();
    let buildable_from_available_artifacts = row.candle_official_ready_match
        && row.no_lookahead_safe
        && source_allowed
        && has_committee_decision
        && has_chair_decision
        && has_risk_decision
        && (has_scenario_row || descriptor.is_some());
    OfficialReadyRowInventoryItem {
        row_id: row.row_id.clone(),
        scenario_row_id: row.scenario_row_id.clone(),
        comparable_row_id: Some(row.row_id.clone()),
        candle_series_id: row
            .matched_candle_series_id
            .clone()
            .or_else(|| descriptor.map(|entry| entry.candle_series_id.clone())),
        symbol: row.symbol.clone(),
        market: row.market,
        venue: descriptor.and_then(|entry| entry.venue.clone()),
        timeframe: row.timeframe.clone(),
        horizon_bars: row.horizon_bars,
        timestamp_ms: row.timestamp_ms,
        source_kind: row.source_kind.clone(),
        source_class: row.source_class,
        official_ready_match: row.candle_official_ready_match,
        benchmark_ready_match: row.candle_benchmark_ready_match,
        row_level: row.row_level || scenario_row.is_some_and(|entry| !summary_level(entry)),
        summary_derived: row.summary_derived || scenario_row.is_some_and(summary_level),
        no_lookahead_safe: row.no_lookahead_safe,
        has_scenario_row,
        has_committee_decision,
        has_chair_decision,
        has_risk_decision,
        has_outcome_reference: row.outcome_reference_available,
        has_baseline_reference: row.baseline_reference_available,
        has_no_trade_counterfactual: row.no_trade_counterfactual_available,
        has_risk_denied_counterfactual: row.risk_denied_counterfactual_available,
        has_external_reference: row.external_reference_available,
        completeness_statuses: statuses,
        buildable_from_available_artifacts,
        reason_codes: stable_reason_codes(&row.reason_codes),
    }
}

fn count_status(
    items: &[OfficialReadyRowInventoryItem],
    status: OfficialReadyRowCompletenessStatus,
) -> usize {
    items
        .iter()
        .filter(|item| item.completeness_statuses.contains(&status))
        .count()
}

fn determine_inventory_status(
    total_items: usize,
    complete_comparable_row_count: usize,
    missing_scenario_count: usize,
    missing_outcome_count: usize,
    missing_baseline_count: usize,
    missing_no_trade_count: usize,
    missing_risk_denied_count: usize,
    source_ineligible_count: usize,
    items: &[OfficialReadyRowInventoryItem],
) -> OfficialReadyRowInventoryStatus {
    if total_items == 0 {
        return OfficialReadyRowInventoryStatus::InsufficientRows;
    }
    if complete_comparable_row_count == total_items {
        return OfficialReadyRowInventoryStatus::HealthyCompleteRows;
    }
    if items.iter().all(|item| {
        item.completeness_statuses.iter().all(|status| {
            matches!(
                status,
                OfficialReadyRowCompletenessStatus::DiagnosticOnly
                    | OfficialReadyRowCompletenessStatus::SourceIneligible
            )
        })
    }) || source_ineligible_count == total_items
    {
        return OfficialReadyRowInventoryStatus::DiagnosticOnly;
    }
    if missing_scenario_count > 0 {
        return OfficialReadyRowInventoryStatus::NeedScenarioMaterialization;
    }
    if missing_outcome_count > 0 {
        return OfficialReadyRowInventoryStatus::NeedOutcomeReferences;
    }
    if missing_baseline_count > 0 {
        return OfficialReadyRowInventoryStatus::NeedBaselineReferences;
    }
    if missing_no_trade_count > 0 || missing_risk_denied_count > 0 {
        return OfficialReadyRowInventoryStatus::NeedCounterfactuals;
    }
    if items.iter().any(|item| {
        item.completeness_statuses
            .contains(&OfficialReadyRowCompletenessStatus::MissingCommitteeDecision)
            || item
                .completeness_statuses
                .contains(&OfficialReadyRowCompletenessStatus::MissingChairDecision)
            || item
                .completeness_statuses
                .contains(&OfficialReadyRowCompletenessStatus::MissingRiskDecision)
    }) {
        return OfficialReadyRowInventoryStatus::NeedCommitteeDecisions;
    }
    OfficialReadyRowInventoryStatus::InsufficientRows
}

fn load_comparable_bundles(
    config: &OfficialReadyRowInventoryConfig,
) -> Result<Vec<ComparableCommitteeEvidenceBundle>, String> {
    if config.comparable_evidence_bundle_paths.is_empty() {
        return Ok(Vec::new());
    }
    config
        .comparable_evidence_bundle_paths
        .iter()
        .map(|path| {
            if path.ends_with(".toml") {
                let comparable_config =
                    ComparableCommitteeEvidenceConfig::from_toml_path(Path::new(path))?;
                ComparableEvidenceBuilder::default().build(&comparable_config)
            } else {
                ComparableCommitteeEvidenceBundle::from_json_path(Path::new(path))
            }
        })
        .collect()
}

fn load_scenario_rows(
    config: &OfficialReadyRowInventoryConfig,
) -> Result<BTreeMap<String, CommitteeScenarioRow>, String> {
    let mut rows = BTreeMap::new();
    for path in &config.scenario_pack_paths {
        let pack = if path.ends_with(".toml") {
            OfficialCommitteeScenarioPackBuilder::default().build(
                &OfficialCommitteeScenarioPackConfig::from_toml_path(Path::new(path))?,
            )?
        } else {
            OfficialCommitteeScenarioPack::from_json_path(Path::new(path))?
        };
        for row in pack.rows {
            rows.insert(row.scenario_row_id.clone(), row);
        }
    }
    Ok(rows)
}

fn load_candle_descriptors(
    config: &OfficialReadyRowInventoryConfig,
) -> Result<BTreeMap<String, OfficialCandleSeriesDescriptor>, String> {
    let mut descriptors = BTreeMap::new();
    for path in &config.official_candle_coverage_pack_paths {
        let pack = load_pack_from_path_or_config(path)?;
        for descriptor in pack.descriptors {
            descriptors.insert(descriptor.candle_series_id.clone(), descriptor);
        }
    }
    Ok(descriptors)
}

fn match_descriptor<'a>(
    row: &ComparableCommitteeEvidenceRow,
    descriptors: &'a BTreeMap<String, OfficialCandleSeriesDescriptor>,
) -> Option<&'a OfficialCandleSeriesDescriptor> {
    descriptors.values().find(|descriptor| {
        descriptor
            .normalized_symbol
            .eq_ignore_ascii_case(&row.symbol.replace('-', ""))
            || descriptor.symbol.eq_ignore_ascii_case(&row.symbol)
                && descriptor.timeframe.eq_ignore_ascii_case(&row.timeframe)
    })
}

fn scenario_feature_available(row: &CommitteeScenarioRow) -> bool {
    row.feature_vector.is_some() || !row.signal_summary.trim().is_empty()
}

fn summary_level(row: &CommitteeScenarioRow) -> bool {
    row.materialization_level != super::CommitteeScenarioMaterializationLevel::RowLevel
}

fn source_allowed(
    source_class: ComparableEvidenceSourceClass,
    config: &OfficialReadyRowInventoryConfig,
) -> bool {
    match source_class {
        ComparableEvidenceSourceClass::OfficialNonCrypto => true,
        ComparableEvidenceSourceClass::OfficialCryptoOnly => config.allow_crypto_only,
        ComparableEvidenceSourceClass::ControlledDiagnostic => config.allow_controlled_diagnostic,
        ComparableEvidenceSourceClass::YFinanceResearch => config.allow_yfinance_research,
        ComparableEvidenceSourceClass::FixtureArchitectureTest
        | ComparableEvidenceSourceClass::SyntheticTest => config.allow_fixture,
        ComparableEvidenceSourceClass::Unknown => false,
    }
}

fn rows_storage_bytes(rows: &[ComparableCommitteeEvidenceRow]) -> usize {
    serde_json::to_vec(rows)
        .map(|bytes| bytes.len())
        .unwrap_or_default()
}

fn input_storage_bytes(paths: &[String]) -> usize {
    paths
        .iter()
        .filter_map(|path| fs::metadata(path).ok())
        .map(|metadata| metadata.len() as usize)
        .sum()
}

fn default_output_root() -> String {
    "target/soma_official_ready_row_inventory".to_string()
}

fn default_max_rows() -> usize {
    500
}

fn default_max_symbols() -> usize {
    5
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn default_true() -> bool {
    true
}
