use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::data::ProviderMarket;

use super::barrier_profile_registry::{
    BarrierProfileRegistry, load_barrier_profile_registry_from_path_or_config,
};
use super::batch_counterfactual_completion::{
    BatchCounterfactualCompletionReport, load_batch_counterfactual_completion_from_path_or_config,
};
use super::batch_outcome_linkage_v3::{
    BatchOutcomeLinkageV3Report, load_batch_outcome_linkage_v3_from_path_or_config,
};
use super::committee_outcome_reference::CommitteeTripleBarrierLabel;
use super::multi_row_official_evidence::{
    MultiRowOfficialEvidenceSet, load_multi_row_official_evidence_set_from_path_or_config,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BalancedOutcomeCoverageConfig {
    pub coverage_id: String,
    #[serde(default)]
    pub multi_row_set_paths: Vec<String>,
    #[serde(default)]
    pub batch_outcome_linkage_paths: Vec<String>,
    #[serde(default)]
    pub batch_counterfactual_completion_paths: Vec<String>,
    #[serde(default)]
    pub barrier_profile_registry_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_min_official_complete_rows")]
    pub min_official_complete_rows: usize,
    #[serde(default = "default_min_symbols")]
    pub min_symbols: usize,
    #[serde(default = "default_min_timeframes")]
    pub min_timeframes: usize,
    #[serde(default = "default_min_horizons")]
    pub min_horizons: usize,
    #[serde(default = "default_min_take_profit")]
    pub min_take_profit: usize,
    #[serde(default = "default_min_stop_loss")]
    pub min_stop_loss: usize,
    #[serde(default = "default_min_time_expired")]
    pub min_time_expired: usize,
    #[serde(default = "default_min_no_trade_counterfactuals")]
    pub min_no_trade_counterfactuals: usize,
    #[serde(default = "default_min_risk_denied_counterfactuals")]
    pub min_risk_denied_counterfactuals: usize,
    #[serde(default = "default_min_outcome_entropy")]
    pub min_outcome_entropy: f64,
    #[serde(default = "default_true")]
    pub require_preregistered_profile: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BalancedOutcomeCoverageCell {
    pub market: ProviderMarket,
    pub symbol: String,
    pub timeframe: String,
    pub horizon_bars: usize,
    pub barrier_profile_id: String,
    pub take_profit_count: usize,
    pub stop_loss_count: usize,
    pub time_expired_count: usize,
    pub no_trade_counterfactual_count: usize,
    pub risk_denied_counterfactual_count: usize,
    pub official_complete_rows: usize,
    pub no_lookahead_safe_rows: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BalancedOutcomeCoverageStatus {
    BalancedEnoughForResearchBenchmark,
    PlumbingOnly,
    NeedMoreRows,
    NeedMoreSymbols,
    NeedMoreOutcomeLabels,
    NeedMoreCounterfactuals,
    DiagnosticOnly,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BalancedOutcomeCoverageReport {
    pub coverage_id: String,
    pub cells: Vec<BalancedOutcomeCoverageCell>,
    pub total_official_complete_rows: usize,
    pub total_take_profit: usize,
    pub total_stop_loss: usize,
    pub total_time_expired: usize,
    pub total_no_trade_counterfactuals: usize,
    pub total_risk_denied_counterfactuals: usize,
    pub symbol_diversity: usize,
    pub timeframe_diversity: usize,
    pub horizon_diversity: usize,
    pub outcome_entropy: f64,
    pub coverage_status: BalancedOutcomeCoverageStatus,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BalancedOutcomeCoverageRunner;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CoverageKey {
    market: ProviderMarket,
    symbol: String,
    timeframe: String,
    horizon_bars: usize,
    barrier_profile_id: String,
}

#[derive(Clone, Debug, Default)]
struct CoverageAccumulator {
    take_profit_count: usize,
    stop_loss_count: usize,
    time_expired_count: usize,
    no_trade_counterfactual_count: usize,
    risk_denied_counterfactual_count: usize,
    official_complete_rows: usize,
    no_lookahead_safe_rows: usize,
}

impl Default for BalancedOutcomeCoverageConfig {
    fn default() -> Self {
        Self {
            coverage_id: "balanced-outcome-coverage".to_string(),
            multi_row_set_paths: Vec::new(),
            batch_outcome_linkage_paths: Vec::new(),
            batch_counterfactual_completion_paths: Vec::new(),
            barrier_profile_registry_path: None,
            output_root: default_output_root(),
            min_official_complete_rows: default_min_official_complete_rows(),
            min_symbols: default_min_symbols(),
            min_timeframes: default_min_timeframes(),
            min_horizons: default_min_horizons(),
            min_take_profit: default_min_take_profit(),
            min_stop_loss: default_min_stop_loss(),
            min_time_expired: default_min_time_expired(),
            min_no_trade_counterfactuals: default_min_no_trade_counterfactuals(),
            min_risk_denied_counterfactuals: default_min_risk_denied_counterfactuals(),
            min_outcome_entropy: default_min_outcome_entropy(),
            require_preregistered_profile: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl BalancedOutcomeCoverageConfig {
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
        if self.coverage_id.trim().is_empty() {
            return Err("balanced outcome coverage id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("balanced outcome coverage paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.coverage_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.multi_row_set_paths
            .iter()
            .chain(self.batch_outcome_linkage_paths.iter())
            .chain(self.batch_counterfactual_completion_paths.iter())
            .chain(self.barrier_profile_registry_path.iter())
            .cloned()
            .collect()
    }
}

impl BalancedOutcomeCoverageRunner {
    pub fn run(
        &self,
        config: &BalancedOutcomeCoverageConfig,
    ) -> Result<BalancedOutcomeCoverageReport, String> {
        config.validate()?;
        let set = config
            .multi_row_set_paths
            .first()
            .map(|path| load_multi_row_official_evidence_set_from_path_or_config(path))
            .transpose()?;
        let outcome_report = config
            .batch_outcome_linkage_paths
            .first()
            .map(|path| load_batch_outcome_linkage_v3_from_path_or_config(path))
            .transpose()?;
        let counterfactual_report = config
            .batch_counterfactual_completion_paths
            .first()
            .map(|path| load_batch_counterfactual_completion_from_path_or_config(path))
            .transpose()?;
        let registry = config
            .barrier_profile_registry_path
            .as_deref()
            .map(load_barrier_profile_registry_from_path_or_config)
            .transpose()?;
        Ok(self.run_from_inputs(
            config,
            set.as_ref(),
            outcome_report.as_ref(),
            counterfactual_report.as_ref(),
            registry.as_ref(),
        ))
    }

    pub fn run_from_inputs(
        &self,
        config: &BalancedOutcomeCoverageConfig,
        set: Option<&MultiRowOfficialEvidenceSet>,
        outcome_report: Option<&BatchOutcomeLinkageV3Report>,
        counterfactual_report: Option<&BatchCounterfactualCompletionReport>,
        registry: Option<&BarrierProfileRegistry>,
    ) -> BalancedOutcomeCoverageReport {
        let default_profile_id = registry
            .and_then(|registry| registry.official_profile(None))
            .map(|profile| profile.profile_id.clone())
            .unwrap_or_else(|| "unregistered".to_string());
        let mut cells = BTreeMap::<CoverageKey, CoverageAccumulator>::new();
        let mut row_keys = BTreeMap::<String, CoverageKey>::new();

        if let Some(set) = set {
            for item in set.items.iter().filter(|item| item.official_complete) {
                let barrier_profile_id = if config.require_preregistered_profile {
                    default_profile_id.clone()
                } else {
                    "unregistered".to_string()
                };
                let key = CoverageKey {
                    market: item.market,
                    symbol: item.symbol.clone(),
                    timeframe: item.timeframe.clone(),
                    horizon_bars: item.horizon_bars,
                    barrier_profile_id,
                };
                let entry = cells.entry(key.clone()).or_default();
                entry.official_complete_rows += 1;
                if item.no_lookahead_safe {
                    entry.no_lookahead_safe_rows += 1;
                }
                row_keys.insert(item.row_id.clone(), key);
            }
        }
        if let Some(report) = outcome_report {
            for record in &report.records {
                if let Some(reference) = record.outcome_reference.as_ref() {
                    if let Some(key) = row_keys.get(&record.row_id) {
                        let entry = cells.entry(key.clone()).or_default();
                        match reference.triple_barrier_label {
                            CommitteeTripleBarrierLabel::TakeProfit => entry.take_profit_count += 1,
                            CommitteeTripleBarrierLabel::StopLoss => entry.stop_loss_count += 1,
                            CommitteeTripleBarrierLabel::TimeExpired => {
                                entry.time_expired_count += 1
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        if let Some(report) = counterfactual_report {
            for record in &report.records {
                if let Some(key) = row_keys.get(&record.row_id) {
                    let entry = cells.entry(key.clone()).or_default();
                    if record.no_trade_counterfactual_built {
                        entry.no_trade_counterfactual_count += 1;
                    }
                    if record.risk_denied_counterfactual_built {
                        entry.risk_denied_counterfactual_count += 1;
                    }
                }
            }
        }

        let mut rendered_cells = cells
            .into_iter()
            .map(|(key, acc)| BalancedOutcomeCoverageCell {
                market: key.market,
                symbol: key.symbol,
                timeframe: key.timeframe,
                horizon_bars: key.horizon_bars,
                barrier_profile_id: key.barrier_profile_id,
                take_profit_count: acc.take_profit_count,
                stop_loss_count: acc.stop_loss_count,
                time_expired_count: acc.time_expired_count,
                no_trade_counterfactual_count: acc.no_trade_counterfactual_count,
                risk_denied_counterfactual_count: acc.risk_denied_counterfactual_count,
                official_complete_rows: acc.official_complete_rows,
                no_lookahead_safe_rows: acc.no_lookahead_safe_rows,
                reason_codes: stable_reason_codes(&[
                    ReasonCode::DeterministicPath,
                    ReasonCode::OfficialEvidenceCounted,
                ]),
            })
            .collect::<Vec<_>>();
        rendered_cells.sort_by(|left, right| {
            left.market
                .cmp(&right.market)
                .then(left.symbol.cmp(&right.symbol))
                .then(left.timeframe.cmp(&right.timeframe))
                .then(left.horizon_bars.cmp(&right.horizon_bars))
                .then(left.barrier_profile_id.cmp(&right.barrier_profile_id))
        });

        let total_official_complete_rows = rendered_cells
            .iter()
            .map(|cell| cell.official_complete_rows)
            .sum();
        let total_take_profit = rendered_cells
            .iter()
            .map(|cell| cell.take_profit_count)
            .sum();
        let total_stop_loss = rendered_cells.iter().map(|cell| cell.stop_loss_count).sum();
        let total_time_expired = rendered_cells
            .iter()
            .map(|cell| cell.time_expired_count)
            .sum();
        let total_no_trade_counterfactuals = rendered_cells
            .iter()
            .map(|cell| cell.no_trade_counterfactual_count)
            .sum();
        let total_risk_denied_counterfactuals = rendered_cells
            .iter()
            .map(|cell| cell.risk_denied_counterfactual_count)
            .sum();
        let symbol_diversity = rendered_cells
            .iter()
            .map(|cell| cell.symbol.clone())
            .collect::<BTreeSet<_>>()
            .len();
        let timeframe_diversity = rendered_cells
            .iter()
            .map(|cell| cell.timeframe.clone())
            .collect::<BTreeSet<_>>()
            .len();
        let horizon_diversity = rendered_cells
            .iter()
            .map(|cell| cell.horizon_bars)
            .collect::<BTreeSet<_>>()
            .len();
        let outcome_entropy = entropy(&[total_take_profit, total_stop_loss, total_time_expired]);
        let diagnostic_only = total_official_complete_rows == 0;
        let coverage_status = determine_status(
            config,
            diagnostic_only,
            total_official_complete_rows,
            symbol_diversity,
            timeframe_diversity,
            horizon_diversity,
            total_take_profit,
            total_stop_loss,
            total_time_expired,
            total_no_trade_counterfactuals,
            total_risk_denied_counterfactuals,
            outcome_entropy,
            registry,
        );
        let warnings = vec![
            "balanced outcome coverage is research-only; balanced labels do not imply profitable research results"
                .to_string(),
        ];

        BalancedOutcomeCoverageReport {
            coverage_id: config.coverage_id.clone(),
            cells: rendered_cells,
            total_official_complete_rows,
            total_take_profit,
            total_stop_loss,
            total_time_expired,
            total_no_trade_counterfactuals,
            total_risk_denied_counterfactuals,
            symbol_diversity,
            timeframe_diversity,
            horizon_diversity,
            outcome_entropy,
            coverage_status,
            warnings,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::DeterministicPath,
                        ReasonCode::OfficialEvidenceCounted,
                    ])
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

impl BalancedOutcomeCoverageReport {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(
            &serde_json::to_string(self).unwrap_or_else(|_| self.coverage_id.clone()),
        )
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("coverage_id={}", self.coverage_id),
            format!(
                "total_official_complete_rows={}",
                self.total_official_complete_rows
            ),
            format!("total_take_profit={}", self.total_take_profit),
            format!("total_stop_loss={}", self.total_stop_loss),
            format!("total_time_expired={}", self.total_time_expired),
            format!(
                "total_no_trade_counterfactuals={}",
                self.total_no_trade_counterfactuals
            ),
            format!(
                "total_risk_denied_counterfactuals={}",
                self.total_risk_denied_counterfactuals
            ),
            format!("symbol_diversity={}", self.symbol_diversity),
            format!("timeframe_diversity={}", self.timeframe_diversity),
            format!("horizon_diversity={}", self.horizon_diversity),
            format!("outcome_entropy={}", self.outcome_entropy),
            format!("coverage_status={:?}", self.coverage_status),
            format!("warnings={}", self.warnings.join(" | ")),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.cells.iter().map(|cell| {
            format!(
                "market={:?};symbol={};timeframe={};horizon_bars={};barrier_profile_id={};take_profit_count={};stop_loss_count={};time_expired_count={};no_trade_counterfactual_count={};risk_denied_counterfactual_count={};official_complete_rows={};no_lookahead_safe_rows={}",
                cell.market,
                cell.symbol,
                cell.timeframe,
                cell.horizon_bars,
                cell.barrier_profile_id,
                cell.take_profit_count,
                cell.stop_loss_count,
                cell.time_expired_count,
                cell.no_trade_counterfactual_count,
                cell.risk_denied_counterfactual_count,
                cell.official_complete_rows,
                cell.no_lookahead_safe_rows,
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
            output_dir.join("balanced_outcome_coverage.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("balanced_outcome_coverage.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_balanced_outcome_coverage_from_path_or_config(
    path: &str,
) -> Result<BalancedOutcomeCoverageReport, String> {
    if path.ends_with(".json") {
        BalancedOutcomeCoverageReport::from_json_path(Path::new(path))
    } else {
        BalancedOutcomeCoverageConfig::from_toml_path(Path::new(path))
            .and_then(|config| BalancedOutcomeCoverageRunner::default().run(&config))
    }
}

fn determine_status(
    config: &BalancedOutcomeCoverageConfig,
    diagnostic_only: bool,
    total_official_complete_rows: usize,
    symbol_diversity: usize,
    timeframe_diversity: usize,
    horizon_diversity: usize,
    total_take_profit: usize,
    total_stop_loss: usize,
    total_time_expired: usize,
    total_no_trade_counterfactuals: usize,
    total_risk_denied_counterfactuals: usize,
    outcome_entropy: f64,
    registry: Option<&BarrierProfileRegistry>,
) -> BalancedOutcomeCoverageStatus {
    if diagnostic_only
        || (config.require_preregistered_profile
            && registry
                .is_some_and(|registry| registry.official_sufficiency_eligible_profiles.is_empty()))
    {
        return BalancedOutcomeCoverageStatus::DiagnosticOnly;
    }
    if total_official_complete_rows < config.min_official_complete_rows {
        return BalancedOutcomeCoverageStatus::NeedMoreRows;
    }
    if symbol_diversity < config.min_symbols
        || timeframe_diversity < config.min_timeframes
        || horizon_diversity < config.min_horizons
    {
        return BalancedOutcomeCoverageStatus::NeedMoreSymbols;
    }
    if total_take_profit < config.min_take_profit
        || total_stop_loss < config.min_stop_loss
        || total_time_expired < config.min_time_expired
        || outcome_entropy < config.min_outcome_entropy
    {
        return BalancedOutcomeCoverageStatus::NeedMoreOutcomeLabels;
    }
    if total_no_trade_counterfactuals < config.min_no_trade_counterfactuals
        || total_risk_denied_counterfactuals < config.min_risk_denied_counterfactuals
    {
        return BalancedOutcomeCoverageStatus::NeedMoreCounterfactuals;
    }
    if total_official_complete_rows > 0 {
        return BalancedOutcomeCoverageStatus::BalancedEnoughForResearchBenchmark;
    }
    BalancedOutcomeCoverageStatus::PlumbingOnly
}

fn entropy(counts: &[usize]) -> f64 {
    let total = counts.iter().sum::<usize>() as f64;
    if total == 0.0 {
        return 0.0;
    }
    counts
        .iter()
        .filter(|count| **count > 0)
        .map(|count| {
            let p = *count as f64 / total;
            -(p * p.log2())
        })
        .sum::<f64>()
}

fn default_output_root() -> String {
    "target/soma_balanced_outcome_coverage".to_string()
}

fn default_true() -> bool {
    true
}

fn default_min_official_complete_rows() -> usize {
    3
}

fn default_min_symbols() -> usize {
    2
}

fn default_min_timeframes() -> usize {
    1
}

fn default_min_horizons() -> usize {
    1
}

fn default_min_take_profit() -> usize {
    1
}

fn default_min_stop_loss() -> usize {
    1
}

fn default_min_time_expired() -> usize {
    1
}

fn default_min_no_trade_counterfactuals() -> usize {
    2
}

fn default_min_risk_denied_counterfactuals() -> usize {
    2
}

fn default_min_outcome_entropy() -> f64 {
    1.0
}
