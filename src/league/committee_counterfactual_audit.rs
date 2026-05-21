use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::backtest::CostModel;
use crate::core::{ReasonCode, stable_reason_codes};

use super::committee_counterfactual_builder::{
    CommitteeCounterfactualBuildConfig, CommitteeCounterfactualBuilder,
    CommitteeCounterfactualRecord, CommitteeCounterfactualType, CounterfactualBuildStatus,
    load_local_candle_series_map, normalize_symbol,
};
use super::committee_outcome_linker::OutcomeLinkedCommitteeScenarioPack;
use super::committee_outcome_linker::{CommitteeOutcomeLinker, CommitteeOutcomeLinkerConfig};
use super::official_committee_pack::{
    OfficialCommitteeScenarioPack, OfficialCommitteeScenarioPackBuilder,
    OfficialCommitteeScenarioPackConfig,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeCounterfactualAuditConfig {
    pub audit_id: String,
    #[serde(default)]
    pub scenario_pack_paths: Vec<String>,
    #[serde(default)]
    pub candle_series_paths: Vec<String>,
    #[serde(default)]
    pub outcome_reference_paths: Vec<String>,
    #[serde(default)]
    pub cost_model: Option<CostModel>,
    #[serde(default = "default_horizon_bars")]
    pub default_horizon_bars: usize,
    #[serde(default = "default_take_profit_pct")]
    pub default_take_profit_pct: f64,
    #[serde(default = "default_stop_loss_pct")]
    pub default_stop_loss_pct: f64,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    pub output_root: String,
    #[serde(default = "default_true")]
    pub build_no_trade_counterfactuals: bool,
    #[serde(default = "default_true")]
    pub build_risk_denied_counterfactuals: bool,
    #[serde(default)]
    pub allow_estimated_when_missing_candles: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeCounterfactualAuditStatus {
    HealthyCounterfactuals,
    NeedMoreCandleData,
    NeedBetterTimestampAlignment,
    NeedMoreNoTradeCounterfactuals,
    NeedMoreRiskDeniedCounterfactuals,
    DiagnosticOnly,
    InsufficientCounterfactuals,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeCounterfactualAuditReport {
    pub audit_id: String,
    pub records: Vec<CommitteeCounterfactualRecord>,
    pub built_count: usize,
    pub unavailable_count: usize,
    pub estimated_count: usize,
    pub no_trade_count: usize,
    pub risk_denied_count: usize,
    pub no_lookahead_rejected_count: usize,
    pub avoided_loss_total: f64,
    pub missed_gain_total: f64,
    pub audit_status: CommitteeCounterfactualAuditStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitteeCounterfactualAuditRunner;

impl Default for CommitteeCounterfactualAuditConfig {
    fn default() -> Self {
        Self {
            audit_id: "committee_counterfactual_audit".to_string(),
            scenario_pack_paths: Vec::new(),
            candle_series_paths: Vec::new(),
            outcome_reference_paths: Vec::new(),
            cost_model: None,
            default_horizon_bars: default_horizon_bars(),
            default_take_profit_pct: default_take_profit_pct(),
            default_stop_loss_pct: default_stop_loss_pct(),
            max_rows: default_max_rows(),
            output_root: "target/soma_committee_counterfactual_audit".to_string(),
            build_no_trade_counterfactuals: true,
            build_risk_denied_counterfactuals: true,
            allow_estimated_when_missing_candles: false,
            reason_codes: vec![ReasonCode::CommitteeCounterfactualAuditBuilt],
        }
    }
}

impl CommitteeCounterfactualAuditConfig {
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
        let paths = self
            .scenario_pack_paths
            .iter()
            .chain(self.candle_series_paths.iter())
            .chain(self.outcome_reference_paths.iter())
            .chain(std::iter::once(&self.output_root));
        if paths.clone().any(|path| path.contains("://")) {
            return Err("committee counterfactual audit paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err(
                "committee counterfactual audit max_rows must be between 1 and 100".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.audit_id)
    }

    pub fn to_build_config(&self) -> CommitteeCounterfactualBuildConfig {
        CommitteeCounterfactualBuildConfig {
            default_horizon_bars: self.default_horizon_bars,
            default_take_profit_pct: self.default_take_profit_pct,
            default_stop_loss_pct: self.default_stop_loss_pct,
            cost_model: self.cost_model,
            allow_estimated_when_missing_candles: self.allow_estimated_when_missing_candles,
            build_no_trade_counterfactuals: self.build_no_trade_counterfactuals,
            build_risk_denied_counterfactuals: self.build_risk_denied_counterfactuals,
            reason_codes: vec![ReasonCode::CommitteeCounterfactualBuilderBuilt],
        }
    }
}

impl CommitteeCounterfactualAuditRunner {
    pub fn run(
        &self,
        config: &CommitteeCounterfactualAuditConfig,
    ) -> Result<CommitteeCounterfactualAuditReport, String> {
        config.validate()?;
        let linked_packs = load_linked_packs(config)?;
        let candle_series = load_local_candle_series_map(&config.candle_series_paths)?;
        let builder = CommitteeCounterfactualBuilder::default();
        let build_config = config.to_build_config();
        let mut records = Vec::new();
        for linked_pack in &linked_packs {
            for row in linked_pack.linked_rows.iter().take(config.max_rows) {
                let symbol = normalize_symbol(&row.scenario_row.symbol);
                records.extend(builder.build_records(
                    row,
                    candle_series.get(&symbol),
                    &build_config,
                ));
            }
        }
        records.sort_by(|left, right| left.counterfactual_id.cmp(&right.counterfactual_id));
        Ok(build_committee_counterfactual_audit_report(
            &config.audit_id,
            records,
            &config.reason_codes,
        ))
    }
}

pub fn build_committee_counterfactual_audit_report(
    audit_id: &str,
    records: Vec<CommitteeCounterfactualRecord>,
    reason_codes: &[ReasonCode],
) -> CommitteeCounterfactualAuditReport {
    let built_count = records.iter().filter(|record| record.built()).count();
    let unavailable_count = records.iter().filter(|record| record.unavailable()).count();
    let estimated_count = records.iter().filter(|record| record.estimated()).count();
    let no_trade_count = records
        .iter()
        .filter(|record| {
            record.counterfactual_type == CommitteeCounterfactualType::NoTrade && record.built()
        })
        .count();
    let risk_denied_count = records
        .iter()
        .filter(|record| {
            record.counterfactual_type == CommitteeCounterfactualType::RiskDenied && record.built()
        })
        .count();
    let no_lookahead_rejected_count = records
        .iter()
        .filter(|record| record.build_status == CounterfactualBuildStatus::RejectedNoLookahead)
        .count();
    let avoided_loss_total = records
        .iter()
        .filter_map(|record| record.avoided_loss_value)
        .sum::<f64>();
    let missed_gain_total = records
        .iter()
        .filter_map(|record| record.missed_gain_value)
        .sum::<f64>();
    let audit_status =
        if built_count > 0 && estimated_count == 0 && no_lookahead_rejected_count == 0 {
            CommitteeCounterfactualAuditStatus::HealthyCounterfactuals
        } else if built_count == 0
            && records.iter().any(|record| {
                record.build_status == CounterfactualBuildStatus::UnavailableNoCandleData
            })
        {
            CommitteeCounterfactualAuditStatus::NeedMoreCandleData
        } else if built_count == 0
            && records.iter().any(|record| {
                record.build_status == CounterfactualBuildStatus::UnavailableNoTimestampMatch
            })
        {
            CommitteeCounterfactualAuditStatus::NeedBetterTimestampAlignment
        } else if no_trade_count == 0 {
            CommitteeCounterfactualAuditStatus::NeedMoreNoTradeCounterfactuals
        } else if risk_denied_count == 0 {
            CommitteeCounterfactualAuditStatus::NeedMoreRiskDeniedCounterfactuals
        } else if built_count > 0 && built_count == estimated_count {
            CommitteeCounterfactualAuditStatus::DiagnosticOnly
        } else {
            CommitteeCounterfactualAuditStatus::InsufficientCounterfactuals
        };
    CommitteeCounterfactualAuditReport {
        audit_id: audit_id.to_string(),
        records,
        built_count,
        unavailable_count,
        estimated_count,
        no_trade_count,
        risk_denied_count,
        no_lookahead_rejected_count,
        avoided_loss_total,
        missed_gain_total,
        audit_status,
        reason_codes: stable_reason_codes(
            &reason_codes
                .iter()
                .cloned()
                .chain([ReasonCode::CommitteeCounterfactualAuditBuilt])
                .collect::<Vec<_>>(),
        ),
    }
}

impl CommitteeCounterfactualAuditReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("audit_id={}", self.audit_id),
            format!("audit_status={:?}", self.audit_status),
            format!("built_count={}", self.built_count),
            format!("unavailable_count={}", self.unavailable_count),
            format!("estimated_count={}", self.estimated_count),
            format!("no_trade_count={}", self.no_trade_count),
            format!("risk_denied_count={}", self.risk_denied_count),
            format!(
                "no_lookahead_rejected_count={}",
                self.no_lookahead_rejected_count
            ),
            format!(
                "avoided_loss_total={}",
                crate::core::deterministic_float_format(self.avoided_loss_total)
            ),
            format!(
                "missed_gain_total={}",
                crate::core::deterministic_float_format(self.missed_gain_total)
            ),
        ];
        for record in &self.records {
            lines.push(record.to_text_line());
        }
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let txt_path = output_dir.join("counterfactual_audit_report.txt");
        fs::write(&txt_path, self.to_text()).map_err(|err| err.to_string())?;
        let json_path = output_dir.join("counterfactual_audit_report.json");
        fs::write(
            &json_path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn load_linked_packs(
    config: &CommitteeCounterfactualAuditConfig,
) -> Result<Vec<OutcomeLinkedCommitteeScenarioPack>, String> {
    let mut linked_packs = Vec::new();
    let linker = CommitteeOutcomeLinker::default();
    for path in &config.scenario_pack_paths {
        let pack = if path.ends_with(".toml") {
            let pack_config = OfficialCommitteeScenarioPackConfig::from_toml_path(Path::new(path))?;
            OfficialCommitteeScenarioPackBuilder::default().build(&pack_config)?
        } else {
            OfficialCommitteeScenarioPack::from_json_path(Path::new(path))?
        };
        let linked_pack = if config.outcome_reference_paths.is_empty() {
            OutcomeLinkedCommitteeScenarioPack {
                pack: pack.clone(),
                linked_rows: pack
                    .rows
                    .iter()
                    .map(
                        |row| super::committee_outcome_linker::OutcomeLinkedCommitteeScenarioRow {
                            scenario_row: row.clone(),
                            outcome_reference: None,
                            baseline_reference: None,
                            external_reference: None,
                            reason_codes: vec![ReasonCode::CommitteeOutcomeLinkerBuilt],
                        },
                    )
                    .collect(),
                unmatched_rows: Vec::new(),
                link_summary: super::committee_outcome_linker::CommitteeOutcomeLinkSummary {
                    linker_id: format!("{}-outcome-only", config.audit_id),
                    matched_rows: 0,
                    unmatched_rows: 0,
                    timestamp_tolerance_ms: 0,
                    strict_timestamp_match: true,
                    no_lookahead_violations: 0,
                    warnings: vec!["no outcome references provided".to_string()],
                    reason_codes: vec![ReasonCode::CommitteeOutcomeLinkerBuilt],
                },
                outcome_linked_count: 0,
                baseline_linked_count: 0,
                external_linked_count: 0,
                no_trade_counterfactual_count: 0,
                risk_denial_counterfactual_count: 0,
                no_lookahead_violations: 0,
                reason_codes: vec![ReasonCode::CommitteeOutcomeLinkerBuilt],
            }
        } else {
            linker.link(
                &pack,
                &CommitteeOutcomeLinkerConfig {
                    linker_id: format!("{}-outcome-only", config.audit_id),
                    scenario_pack_path: None,
                    outcome_artifact_paths: config.outcome_reference_paths.clone(),
                    baseline_artifact_paths: Vec::new(),
                    external_prediction_paths: Vec::new(),
                    output_root: config.output_root.clone(),
                    strict_timestamp_match: true,
                    max_timestamp_tolerance_ms: 0,
                    require_same_symbol: true,
                    require_same_horizon: true,
                    reason_codes: vec![ReasonCode::CommitteeOutcomeLinkerBuilt],
                },
            )?
        };
        linked_packs.push(linked_pack);
    }
    linked_packs.sort_by(|left, right| left.pack.pack_id.cmp(&right.pack.pack_id));
    Ok(linked_packs)
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

fn default_max_rows() -> usize {
    100
}

fn default_true() -> bool {
    true
}
