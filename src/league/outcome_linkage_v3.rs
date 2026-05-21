use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::committee_outcome_reference::{
    CommitteeOutcomeReference, CommitteeTripleBarrierLabel, parse_evidence_source_kind,
};
use super::complete_row_closure_bundle::CompleteRowClosureBundle;
use super::future_window_requirements::{
    FutureBar, FutureWindowGapKind, FutureWindowRequirementConfig, FutureWindowRequirementReport,
    FutureWindowRequirementRunner, load_descriptor_map_from_paths, load_future_bars_from_csv,
    load_future_window_inputs, load_future_window_requirement_from_path_or_config,
};
use super::official_ready_row_inventory::OfficialReadyRowInventoryReport;
use super::triple_barrier_reference_builder::TripleBarrierTieBreakPolicy;
use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeLinkageV3Config {
    pub linkage_id: String,
    #[serde(default)]
    pub official_ready_inventory_path: Option<String>,
    #[serde(default)]
    pub scenario_materialization_v3_path: Option<String>,
    #[serde(default)]
    pub future_window_requirement_path: Option<String>,
    #[serde(default)]
    pub extended_candle_pack_paths: Vec<String>,
    #[serde(default)]
    pub complete_row_closure_paths: Vec<String>,
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
pub enum OutcomeLinkageV3RecordStatus {
    OutcomeGenerated,
    SkippedMissingFutureBars,
    SkippedMissingScenario,
    SkippedTimestampMismatch,
    SkippedHorizonMismatch,
    SkippedSourceIneligible,
    RejectedNoLookahead,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeLinkageV3Record {
    pub row_id: String,
    #[serde(default)]
    pub scenario_row_id: Option<String>,
    #[serde(default)]
    pub candle_series_id: Option<String>,
    pub status: OutcomeLinkageV3RecordStatus,
    #[serde(default)]
    pub outcome_reference: Option<CommitteeOutcomeReference>,
    #[serde(default)]
    pub net_return_pct: Option<f64>,
    #[serde(default)]
    pub mfe_pct: Option<f64>,
    #[serde(default)]
    pub mae_pct: Option<f64>,
    pub cost_bps: f64,
    pub slippage_bps: f64,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OutcomeLinkageV3Status {
    OutcomeLinksImproved,
    OfficialOutcomeLinksImproved,
    StillNeedFutureBars,
    StillNeedScenarioRows,
    StillTimestampMismatch,
    StillHorizonMismatch,
    SourceIneligible,
    DiagnosticOnly,
    #[default]
    NoImprovement,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeLinkageV3Report {
    pub linkage_id: String,
    pub records: Vec<OutcomeLinkageV3Record>,
    pub generated_outcome_count: usize,
    pub skipped_missing_future_bars: usize,
    pub skipped_timestamp_mismatch: usize,
    pub skipped_horizon_mismatch: usize,
    pub rejected_no_lookahead: usize,
    pub official_outcome_count: usize,
    pub diagnostic_outcome_count: usize,
    pub linkage_status: OutcomeLinkageV3Status,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OutcomeLinkageV3Runner;

impl Default for OutcomeLinkageV3Config {
    fn default() -> Self {
        Self {
            linkage_id: "outcome-linkage-v3".to_string(),
            official_ready_inventory_path: None,
            scenario_materialization_v3_path: None,
            future_window_requirement_path: None,
            extended_candle_pack_paths: Vec::new(),
            complete_row_closure_paths: Vec::new(),
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

impl OutcomeLinkageV3Config {
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
        if self.linkage_id.trim().is_empty() {
            return Err("outcome linkage v3 id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("outcome linkage v3 paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.linkage_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.official_ready_inventory_path
            .iter()
            .cloned()
            .chain(self.scenario_materialization_v3_path.iter().cloned())
            .chain(self.future_window_requirement_path.iter().cloned())
            .chain(self.extended_candle_pack_paths.iter().cloned())
            .chain(self.complete_row_closure_paths.iter().cloned())
            .collect()
    }
}

impl OutcomeLinkageV3Runner {
    pub fn run(&self, config: &OutcomeLinkageV3Config) -> Result<OutcomeLinkageV3Report, String> {
        config.validate()?;
        let inventory = load_inventory(config)?;
        let requirements = load_requirements(config)?;
        let row_map = load_row_map(config)?;
        let descriptors = load_descriptor_map_from_paths(&config.extended_candle_pack_paths)?;
        self.run_from_inputs(config, &inventory, &requirements, &descriptors, &row_map)
    }

    pub fn run_from_inputs(
        &self,
        config: &OutcomeLinkageV3Config,
        inventory: &OfficialReadyRowInventoryReport,
        requirements: &FutureWindowRequirementReport,
        descriptors: &BTreeMap<
            String,
            super::official_candle_coverage_pack::OfficialCandleSeriesDescriptor,
        >,
        row_map: &BTreeMap<
            String,
            super::comparable_committee_evidence::ComparableCommitteeEvidenceRow,
        >,
    ) -> Result<OutcomeLinkageV3Report, String> {
        config.validate()?;
        let requirement_map = requirements
            .items
            .iter()
            .map(|item| (item.row_id.clone(), item.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut records = inventory
            .items
            .iter()
            .map(|item| {
                build_record(
                    config,
                    item,
                    requirement_map.get(&item.row_id),
                    descriptors,
                    row_map,
                )
            })
            .collect::<Vec<_>>();
        records.sort_by(|left, right| left.row_id.cmp(&right.row_id));
        let generated_outcome_count = records
            .iter()
            .filter(|record| {
                matches!(
                    record.status,
                    OutcomeLinkageV3RecordStatus::OutcomeGenerated
                        | OutcomeLinkageV3RecordStatus::DiagnosticOnly
                ) && record.outcome_reference.is_some()
            })
            .count();
        let skipped_missing_future_bars = records
            .iter()
            .filter(|record| {
                record.status == OutcomeLinkageV3RecordStatus::SkippedMissingFutureBars
            })
            .count();
        let skipped_timestamp_mismatch = records
            .iter()
            .filter(|record| {
                record.status == OutcomeLinkageV3RecordStatus::SkippedTimestampMismatch
            })
            .count();
        let skipped_horizon_mismatch = records
            .iter()
            .filter(|record| record.status == OutcomeLinkageV3RecordStatus::SkippedHorizonMismatch)
            .count();
        let rejected_no_lookahead = records
            .iter()
            .filter(|record| record.status == OutcomeLinkageV3RecordStatus::RejectedNoLookahead)
            .count();
        let official_outcome_count = records
            .iter()
            .filter(|record| record.status == OutcomeLinkageV3RecordStatus::OutcomeGenerated)
            .count();
        let diagnostic_outcome_count = records
            .iter()
            .filter(|record| record.status == OutcomeLinkageV3RecordStatus::DiagnosticOnly)
            .count();
        let linkage_status = determine_status(&records);
        Ok(OutcomeLinkageV3Report {
            linkage_id: config.linkage_id.clone(),
            records,
            generated_outcome_count,
            skipped_missing_future_bars,
            skipped_timestamp_mismatch,
            skipped_horizon_mismatch,
            rejected_no_lookahead,
            official_outcome_count,
            diagnostic_outcome_count,
            linkage_status,
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

impl OutcomeLinkageV3Report {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_else(|_| self.linkage_id.clone()))
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("linkage_id={}", self.linkage_id),
            format!("generated_outcome_count={}", self.generated_outcome_count),
            format!(
                "skipped_missing_future_bars={}",
                self.skipped_missing_future_bars
            ),
            format!(
                "skipped_timestamp_mismatch={}",
                self.skipped_timestamp_mismatch
            ),
            format!("skipped_horizon_mismatch={}", self.skipped_horizon_mismatch),
            format!("rejected_no_lookahead={}", self.rejected_no_lookahead),
            format!("official_outcome_count={}", self.official_outcome_count),
            format!("diagnostic_outcome_count={}", self.diagnostic_outcome_count),
            format!("linkage_status={:?}", self.linkage_status),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.records.iter().map(|record| {
            format!(
                "row_id={};status={:?};label={};net_return_pct={};mfe_pct={};mae_pct={};candle_series_id={}",
                record.row_id,
                record.status,
                record
                    .outcome_reference
                    .as_ref()
                    .map(|reference| format!("{:?}", reference.triple_barrier_label))
                    .unwrap_or_default(),
                record.net_return_pct.map(|value| value.to_string()).unwrap_or_default(),
                record.mfe_pct.map(|value| value.to_string()).unwrap_or_default(),
                record.mae_pct.map(|value| value.to_string()).unwrap_or_default(),
                record.candle_series_id.clone().unwrap_or_default(),
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
        fs::write(output_dir.join("outcome_linkage_v3.txt"), self.to_text())
            .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("outcome_linkage_v3_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_outcome_linkage_v3_from_path_or_config(
    path: &str,
) -> Result<OutcomeLinkageV3Report, String> {
    if path.ends_with(".json") {
        OutcomeLinkageV3Report::from_json_path(Path::new(path))
    } else {
        OutcomeLinkageV3Config::from_toml_path(Path::new(path))
            .and_then(|config| OutcomeLinkageV3Runner::default().run(&config))
    }
}

fn build_record(
    config: &OutcomeLinkageV3Config,
    item: &super::official_ready_row_inventory::OfficialReadyRowInventoryItem,
    requirement: Option<&super::future_window_requirements::FutureWindowRequirementItem>,
    descriptors: &BTreeMap<
        String,
        super::official_candle_coverage_pack::OfficialCandleSeriesDescriptor,
    >,
    row_map: &BTreeMap<
        String,
        super::comparable_committee_evidence::ComparableCommitteeEvidenceRow,
    >,
) -> OutcomeLinkageV3Record {
    let mut reason_codes = item.reason_codes.clone();
    let row = row_map.get(&item.row_id);
    let horizon_bars = row
        .map(|row| row.horizon_bars)
        .filter(|value| *value > 0)
        .unwrap_or(item.horizon_bars.max(config.default_horizon_bars));
    let status_and_reference = if config.require_no_lookahead_safe && !item.no_lookahead_safe {
        reason_codes.push(ReasonCode::RejectedNoLookaheadReference);
        (OutcomeLinkageV3RecordStatus::RejectedNoLookahead, None)
    } else if item.scenario_row_id.is_none() && row.and_then(|row| row.scenario_row_id.as_ref()).is_none() {
        reason_codes.push(ReasonCode::FeatureUnavailable);
        (OutcomeLinkageV3RecordStatus::SkippedMissingScenario, None)
    } else if matches!(
        item.source_class,
        super::comparable_committee_evidence::ComparableEvidenceSourceClass::YFinanceResearch
            | super::comparable_committee_evidence::ComparableEvidenceSourceClass::FixtureArchitectureTest
            | super::comparable_committee_evidence::ComparableEvidenceSourceClass::SyntheticTest
    ) {
        reason_codes.push(ReasonCode::ReadinessEvidenceExcluded);
        (OutcomeLinkageV3RecordStatus::SkippedSourceIneligible, None)
    } else if requirement.is_some_and(|entry| {
        matches!(
            entry.gap_kind,
            FutureWindowGapKind::MissingFutureBars | FutureWindowGapKind::MissingCandleWindow
        )
    }) {
        reason_codes.push(ReasonCode::InsufficientBars);
        (OutcomeLinkageV3RecordStatus::SkippedMissingFutureBars, None)
    } else if requirement.is_some_and(|entry| {
        matches!(entry.gap_kind, FutureWindowGapKind::TimestampOutsideRange | FutureWindowGapKind::MissingScenarioTimestamp)
    }) {
        reason_codes.push(ReasonCode::UnsupportedTimestampFormat);
        (OutcomeLinkageV3RecordStatus::SkippedTimestampMismatch, None)
    } else if config.require_exact_horizon_match
        && requirement.is_some_and(|entry| entry.horizon_bars != horizon_bars)
    {
        reason_codes.push(ReasonCode::HorizonFiltered);
        (OutcomeLinkageV3RecordStatus::SkippedHorizonMismatch, None)
    } else {
        match generate_outcome_reference(config, item, requirement, descriptors, horizon_bars) {
            Ok(Some(reference)) => {
                let status = if matches!(
                    item.source_class,
                    super::comparable_committee_evidence::ComparableEvidenceSourceClass::OfficialNonCrypto
                ) {
                    OutcomeLinkageV3RecordStatus::OutcomeGenerated
                } else {
                    OutcomeLinkageV3RecordStatus::DiagnosticOnly
                };
                (status, Some(reference))
            }
            Ok(None) => (OutcomeLinkageV3RecordStatus::SkippedMissingFutureBars, None),
            Err(reason) => {
                let status = if reason.contains("timestamp") {
                    OutcomeLinkageV3RecordStatus::SkippedTimestampMismatch
                } else if reason.contains("horizon") {
                    OutcomeLinkageV3RecordStatus::SkippedHorizonMismatch
                } else {
                    OutcomeLinkageV3RecordStatus::SkippedMissingFutureBars
                };
                (status, None)
            }
        }
    };
    let status = status_and_reference.0;
    let outcome_reference = status_and_reference.1;
    if let Some(reference) = outcome_reference.as_ref() {
        reason_codes.extend(reference.reason_codes.iter().cloned());
    }
    OutcomeLinkageV3Record {
        row_id: item.row_id.clone(),
        scenario_row_id: item
            .scenario_row_id
            .clone()
            .or_else(|| row.and_then(|row| row.scenario_row_id.clone())),
        candle_series_id: requirement
            .and_then(|entry| entry.candle_series_id.clone())
            .or_else(|| item.candle_series_id.clone()),
        status,
        net_return_pct: outcome_reference
            .as_ref()
            .and_then(|reference| reference.net_return_pct),
        mfe_pct: outcome_reference
            .as_ref()
            .and_then(|reference| reference.max_favorable_excursion_pct),
        mae_pct: outcome_reference
            .as_ref()
            .and_then(|reference| reference.max_adverse_excursion_pct),
        outcome_reference,
        cost_bps: config.cost_bps,
        slippage_bps: config.slippage_bps,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn generate_outcome_reference(
    config: &OutcomeLinkageV3Config,
    item: &super::official_ready_row_inventory::OfficialReadyRowInventoryItem,
    requirement: Option<&super::future_window_requirements::FutureWindowRequirementItem>,
    descriptors: &BTreeMap<
        String,
        super::official_candle_coverage_pack::OfficialCandleSeriesDescriptor,
    >,
    horizon_bars: usize,
) -> Result<Option<CommitteeOutcomeReference>, String> {
    let candle_series_id = requirement
        .and_then(|entry| entry.candle_series_id.as_ref())
        .or(item.candle_series_id.as_ref())
        .cloned();
    let descriptor = candle_series_id
        .as_ref()
        .and_then(|id| descriptors.get(id))
        .cloned()
        .or_else(|| {
            descriptors
                .values()
                .find(|descriptor| {
                    descriptor.symbol == item.symbol && descriptor.timeframe == item.timeframe
                })
                .cloned()
        })
        .ok_or_else(|| "missing future window descriptor".to_string())?;
    if config.require_exact_symbol_match && descriptor.symbol != item.symbol {
        return Err("symbol mismatch".to_string());
    }
    let bars = load_future_bars_from_csv(Path::new(&descriptor.path))?;
    let Some(entry_index) = bars
        .iter()
        .position(|bar| bar.timestamp_ms == item.timestamp_ms)
    else {
        return Err("timestamp mismatch".to_string());
    };
    let future_end_index = entry_index.saturating_add(horizon_bars);
    if future_end_index >= bars.len() {
        return Ok(None);
    }
    let entry_bar = &bars[entry_index];
    let take_price = entry_bar.close * (1.0 + config.take_profit_pct.max(0.0));
    let stop_price = entry_bar.close * (1.0 - config.stop_loss_pct.max(0.0));
    let mut label = CommitteeTripleBarrierLabel::TimeExpired;
    let mut exit_price = bars[future_end_index].close;
    let mut mfe = 0.0f64;
    let mut mae = 0.0f64;
    let mut reasons = vec![
        ReasonCode::CommitteeOutcomeReferenceBuilt,
        ReasonCode::DeterministicPath,
    ];
    for current_index in (entry_index + 1)..=future_end_index {
        let bar = &bars[current_index];
        mfe = mfe.max((bar.high / entry_bar.close.max(1e-9)) - 1.0);
        mae = mae.min((bar.low / entry_bar.close.max(1e-9)) - 1.0);
        let take_hit = bar.high >= take_price;
        let stop_hit = bar.low <= stop_price;
        if take_hit || stop_hit {
            label = choose_label(
                bar,
                entry_bar.close,
                take_price,
                stop_price,
                config.tie_break_policy,
                take_hit,
                stop_hit,
            );
            exit_price = match label {
                CommitteeTripleBarrierLabel::TakeProfit => take_price,
                CommitteeTripleBarrierLabel::StopLoss => stop_price,
                _ => bar.close,
            };
            match label {
                CommitteeTripleBarrierLabel::TakeProfit => reasons.push(ReasonCode::TakeProfitHit),
                CommitteeTripleBarrierLabel::StopLoss => reasons.push(ReasonCode::StopLossHit),
                _ => {}
            }
            if take_hit && stop_hit && label == CommitteeTripleBarrierLabel::StopLoss {
                reasons.push(ReasonCode::ConservativeSameCandleLoss);
            }
            break;
        }
    }
    if label == CommitteeTripleBarrierLabel::TimeExpired {
        reasons.push(ReasonCode::TimeBarrierExpired);
    }
    let gross_return_pct = (exit_price / entry_bar.close.max(1e-9)) - 1.0;
    let net_return_pct = gross_return_pct - (config.cost_bps + config.slippage_bps) / 10_000.0;
    reasons.push(ReasonCode::CostApplied);
    Ok(Some(CommitteeOutcomeReference {
        outcome_id: format!("{}-outcome-v3", item.row_id),
        decision_id: item.scenario_row_id.clone(),
        symbol: item.symbol.clone(),
        timestamp_ms: item.timestamp_ms,
        horizon_bars,
        triple_barrier_label: label,
        net_return_pct: Some(net_return_pct),
        max_favorable_excursion_pct: Some(mfe.max(0.0)),
        max_adverse_excursion_pct: Some(mae.min(0.0)),
        cost_bps: config.cost_bps,
        slippage_bps: config.slippage_bps,
        source_kind: parse_evidence_source_kind(Some(&item.source_kind)),
        no_lookahead_safe: item.no_lookahead_safe,
        reason_codes: stable_reason_codes(&reasons),
    }))
}

fn choose_label(
    bar: &FutureBar,
    entry_price: f64,
    take_price: f64,
    stop_price: f64,
    tie_break_policy: TripleBarrierTieBreakPolicy,
    take_hit: bool,
    stop_hit: bool,
) -> CommitteeTripleBarrierLabel {
    match (take_hit, stop_hit) {
        (true, false) => CommitteeTripleBarrierLabel::TakeProfit,
        (false, true) => CommitteeTripleBarrierLabel::StopLoss,
        (true, true) => match tie_break_policy {
            TripleBarrierTieBreakPolicy::StopFirst => CommitteeTripleBarrierLabel::StopLoss,
            TripleBarrierTieBreakPolicy::TakeProfitFirst => CommitteeTripleBarrierLabel::TakeProfit,
            TripleBarrierTieBreakPolicy::TimeOrder => {
                let take_distance = (take_price - bar.open).abs();
                let stop_distance = (bar.open - stop_price).abs();
                if take_distance < stop_distance {
                    CommitteeTripleBarrierLabel::TakeProfit
                } else if stop_distance < take_distance {
                    CommitteeTripleBarrierLabel::StopLoss
                } else if bar.close >= entry_price {
                    CommitteeTripleBarrierLabel::TakeProfit
                } else {
                    CommitteeTripleBarrierLabel::StopLoss
                }
            }
        },
        (false, false) => CommitteeTripleBarrierLabel::TimeExpired,
    }
}

fn determine_status(records: &[OutcomeLinkageV3Record]) -> OutcomeLinkageV3Status {
    if records.is_empty() {
        return OutcomeLinkageV3Status::NoImprovement;
    }
    if records
        .iter()
        .any(|record| record.status == OutcomeLinkageV3RecordStatus::OutcomeGenerated)
    {
        return OutcomeLinkageV3Status::OfficialOutcomeLinksImproved;
    }
    if records
        .iter()
        .any(|record| record.status == OutcomeLinkageV3RecordStatus::DiagnosticOnly)
    {
        return OutcomeLinkageV3Status::OutcomeLinksImproved;
    }
    if records
        .iter()
        .any(|record| record.status == OutcomeLinkageV3RecordStatus::SkippedMissingFutureBars)
    {
        return OutcomeLinkageV3Status::StillNeedFutureBars;
    }
    if records
        .iter()
        .any(|record| record.status == OutcomeLinkageV3RecordStatus::SkippedMissingScenario)
    {
        return OutcomeLinkageV3Status::StillNeedScenarioRows;
    }
    if records
        .iter()
        .any(|record| record.status == OutcomeLinkageV3RecordStatus::SkippedTimestampMismatch)
    {
        return OutcomeLinkageV3Status::StillTimestampMismatch;
    }
    if records
        .iter()
        .any(|record| record.status == OutcomeLinkageV3RecordStatus::SkippedHorizonMismatch)
    {
        return OutcomeLinkageV3Status::StillHorizonMismatch;
    }
    if records
        .iter()
        .all(|record| record.status == OutcomeLinkageV3RecordStatus::SkippedSourceIneligible)
    {
        return OutcomeLinkageV3Status::SourceIneligible;
    }
    if records
        .iter()
        .all(|record| record.status == OutcomeLinkageV3RecordStatus::DiagnosticOnly)
    {
        return OutcomeLinkageV3Status::DiagnosticOnly;
    }
    OutcomeLinkageV3Status::NoImprovement
}

fn load_inventory(
    config: &OutcomeLinkageV3Config,
) -> Result<OfficialReadyRowInventoryReport, String> {
    if let Some(path) = config.official_ready_inventory_path.as_deref() {
        if path.ends_with(".json") {
            OfficialReadyRowInventoryReport::from_json_path(Path::new(path))
        } else {
            let loaded = FutureWindowRequirementConfig {
                official_ready_inventory_paths: vec![path.to_string()],
                output_root: config.output_root.clone(),
                ..FutureWindowRequirementConfig::default()
            };
            Ok(load_future_window_inputs(&loaded)?.inventory)
        }
    } else if let Some(path) = config.complete_row_closure_paths.first() {
        let bundle = CompleteRowClosureBundle::from_json_path(Path::new(path))?;
        Ok(bundle.inventory_report)
    } else {
        Err("outcome linkage v3 requires official_ready_inventory_path or complete_row_closure_paths".to_string())
    }
}

fn load_requirements(
    config: &OutcomeLinkageV3Config,
) -> Result<FutureWindowRequirementReport, String> {
    if let Some(path) = config.future_window_requirement_path.as_deref() {
        load_future_window_requirement_from_path_or_config(path)
    } else {
        let derived = FutureWindowRequirementConfig {
            requirement_id: format!("{}-requirements", config.linkage_id),
            official_ready_inventory_paths: config
                .official_ready_inventory_path
                .iter()
                .cloned()
                .collect(),
            complete_row_closure_paths: config.complete_row_closure_paths.clone(),
            candle_coverage_pack_paths: config.extended_candle_pack_paths.clone(),
            output_root: config.output_root.clone(),
            default_horizon_bars: config.default_horizon_bars,
            reason_codes: config.reason_codes.clone(),
            ..FutureWindowRequirementConfig::default()
        };
        FutureWindowRequirementRunner::default().run(&derived)
    }
}

fn load_row_map(
    config: &OutcomeLinkageV3Config,
) -> Result<
    BTreeMap<String, super::comparable_committee_evidence::ComparableCommitteeEvidenceRow>,
    String,
> {
    if let Some(path) = config.complete_row_closure_paths.first() {
        let bundle = CompleteRowClosureBundle::from_json_path(Path::new(path))?;
        Ok(bundle
            .complete_comparable_row_bundle
            .rows
            .into_iter()
            .map(|row| (row.row_id.clone(), row))
            .collect())
    } else {
        Ok(BTreeMap::new())
    }
}

fn default_output_root() -> String {
    "target/soma_outcome_linkage_v3".to_string()
}

fn default_horizon_bars() -> usize {
    24
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
