use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::batch_counterfactual_completion::{
    BatchCounterfactualCompletionReport, load_batch_counterfactual_completion_from_path_or_config,
};
use super::batch_outcome_linkage_v3::{
    BatchOutcomeLinkageV3Report, load_batch_outcome_linkage_v3_from_path_or_config,
};
use super::multi_row_official_evidence::{
    MultiRowOfficialEvidenceSet, load_multi_row_official_evidence_set_from_path_or_config,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialEvidenceSufficiencyV2Config {
    pub sufficiency_id: String,
    pub multi_row_set_path: String,
    #[serde(default)]
    pub batch_outcome_linkage_path: Option<String>,
    #[serde(default)]
    pub batch_counterfactual_completion_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_min_total_rows", alias = "min_rows_for_plumbing")]
    pub min_total_rows: usize,
    #[serde(
        default = "default_min_official_complete_rows",
        alias = "min_rows_for_research"
    )]
    pub min_official_complete_rows: usize,
    #[serde(default = "default_min_symbols", alias = "min_symbols_for_plumbing")]
    pub min_symbols: usize,
    #[serde(
        default = "default_min_timeframes",
        alias = "min_timeframes_for_signal_quality"
    )]
    pub min_timeframes: usize,
    #[serde(default = "default_min_horizons")]
    pub min_horizons: usize,
    #[serde(default = "default_min_take_profit_outcomes")]
    pub min_take_profit_outcomes: usize,
    #[serde(default = "default_min_stop_loss_outcomes")]
    pub min_stop_loss_outcomes: usize,
    #[serde(default = "default_min_time_expired_outcomes")]
    pub min_time_expired_outcomes: usize,
    #[serde(
        default = "default_min_no_trade_counterfactuals",
        alias = "min_counterfactuals_for_research"
    )]
    pub min_no_trade_counterfactuals: usize,
    #[serde(
        default = "default_min_risk_denied_counterfactuals",
        alias = "min_counterfactuals_for_signal_quality"
    )]
    pub min_risk_denied_counterfactuals: usize,
    #[serde(default = "default_min_baseline_references")]
    pub min_baseline_references: usize,
    #[serde(default = "default_min_no_lookahead_safe_ratio")]
    pub min_no_lookahead_safe_ratio: f64,
    #[serde(default = "default_max_single_symbol_concentration_ratio")]
    pub max_single_symbol_concentration_ratio: f64,
    #[serde(default = "default_max_single_outcome_label_ratio")]
    pub max_single_outcome_label_ratio: f64,
    #[serde(default = "default_true", alias = "require_non_crypto_official_rows")]
    pub require_non_crypto_official: bool,
    #[serde(default = "default_true")]
    pub require_outcome_diversity: bool,
    #[serde(default = "default_true")]
    pub require_counterfactual_diversity: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceSufficiencyV2Counts {
    pub total_rows: usize,
    pub official_complete_rows: usize,
    pub symbols: usize,
    pub timeframes: usize,
    pub horizons: usize,
    pub take_profit_count: usize,
    pub stop_loss_count: usize,
    pub time_expired_count: usize,
    pub no_trade_counterfactual_count: usize,
    pub risk_denied_counterfactual_count: usize,
    pub baseline_reference_count: usize,
    pub no_lookahead_safe_ratio: f64,
    pub single_symbol_concentration_ratio: f64,
    pub single_outcome_label_ratio: f64,
    pub non_crypto_official_rows: usize,
    pub crypto_only_rows: usize,
    pub controlled_rows: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialEvidenceSufficiencyV2Status {
    InsufficientRows,
    InsufficientOfficialCompleteRows,
    InsufficientSymbolDiversity,
    InsufficientOutcomeDiversity,
    InsufficientCounterfactualDepth,
    SingleSymbolDominated,
    SingleOutcomeDominated,
    PlumbingValidated,
    CommitteeBenchmarkResearchReady,
    TentativeSignalQualityReviewReady,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialEvidenceSufficiencyV2Recommendation {
    RunCommitteeOfficialBenchmark,
    MoreOfficialRows,
    MoreOfficialSymbols,
    MoreOutcomeDiversity,
    MoreCounterfactualDepth,
    ImproveSignalModelFirst,
    KeepTrinity,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceSufficiencyV2Report {
    pub sufficiency_id: String,
    pub counts: OfficialEvidenceSufficiencyV2Counts,
    pub total_rows: usize,
    pub official_complete_rows: usize,
    pub symbols: usize,
    pub timeframes: usize,
    pub horizons: usize,
    pub take_profit_count: usize,
    pub stop_loss_count: usize,
    pub time_expired_count: usize,
    pub no_trade_counterfactual_count: usize,
    pub risk_denied_counterfactual_count: usize,
    pub baseline_reference_count: usize,
    pub no_lookahead_safe_ratio: f64,
    pub single_symbol_concentration_ratio: f64,
    pub single_outcome_label_ratio: f64,
    pub passed_plumbing_validation: bool,
    pub passed_committee_benchmark_research: bool,
    pub passed_tentative_signal_quality_review: bool,
    pub failed_gates: Vec<String>,
    pub still_insufficient_for_usefulness_claims: bool,
    pub sufficiency_status: OfficialEvidenceSufficiencyV2Status,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialEvidenceSufficiencyV2Runner;

impl Default for OfficialEvidenceSufficiencyV2Config {
    fn default() -> Self {
        Self {
            sufficiency_id: "official-evidence-sufficiency-v2".to_string(),
            multi_row_set_path: String::new(),
            batch_outcome_linkage_path: None,
            batch_counterfactual_completion_path: None,
            output_root: default_output_root(),
            min_total_rows: default_min_total_rows(),
            min_official_complete_rows: default_min_official_complete_rows(),
            min_symbols: default_min_symbols(),
            min_timeframes: default_min_timeframes(),
            min_horizons: default_min_horizons(),
            min_take_profit_outcomes: default_min_take_profit_outcomes(),
            min_stop_loss_outcomes: default_min_stop_loss_outcomes(),
            min_time_expired_outcomes: default_min_time_expired_outcomes(),
            min_no_trade_counterfactuals: default_min_no_trade_counterfactuals(),
            min_risk_denied_counterfactuals: default_min_risk_denied_counterfactuals(),
            min_baseline_references: default_min_baseline_references(),
            min_no_lookahead_safe_ratio: default_min_no_lookahead_safe_ratio(),
            max_single_symbol_concentration_ratio: default_max_single_symbol_concentration_ratio(),
            max_single_outcome_label_ratio: default_max_single_outcome_label_ratio(),
            require_non_crypto_official: true,
            require_outcome_diversity: true,
            require_counterfactual_diversity: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialEvidenceSufficiencyV2Config {
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
        if self.sufficiency_id.trim().is_empty() {
            return Err("official evidence sufficiency id must not be empty".to_string());
        }
        if self.multi_row_set_path.trim().is_empty() {
            return Err("official evidence sufficiency requires multi_row_set_path".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("official evidence sufficiency paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.sufficiency_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.batch_outcome_linkage_path
            .iter()
            .cloned()
            .chain(self.batch_counterfactual_completion_path.iter().cloned())
            .chain(std::iter::once(self.multi_row_set_path.clone()))
            .collect()
    }
}

impl OfficialEvidenceSufficiencyV2Runner {
    pub fn run(
        &self,
        config: &OfficialEvidenceSufficiencyV2Config,
    ) -> Result<OfficialEvidenceSufficiencyV2Report, String> {
        config.validate()?;
        let set =
            load_multi_row_official_evidence_set_from_path_or_config(&config.multi_row_set_path)?;
        let outcome_report = config
            .batch_outcome_linkage_path
            .as_deref()
            .map(load_batch_outcome_linkage_v3_from_path_or_config)
            .transpose()?;
        let counterfactual_report = config
            .batch_counterfactual_completion_path
            .as_deref()
            .map(load_batch_counterfactual_completion_from_path_or_config)
            .transpose()?;
        Ok(self.run_from_inputs(
            config,
            &set,
            outcome_report.as_ref(),
            counterfactual_report.as_ref(),
        ))
    }

    pub fn run_from_inputs(
        &self,
        config: &OfficialEvidenceSufficiencyV2Config,
        set: &MultiRowOfficialEvidenceSet,
        outcome_report: Option<&BatchOutcomeLinkageV3Report>,
        counterfactual_report: Option<&BatchCounterfactualCompletionReport>,
    ) -> OfficialEvidenceSufficiencyV2Report {
        let counts = compute_counts(set, outcome_report, counterfactual_report);
        let failed_gates = collect_failed_gates(config, &counts);
        let passed_plumbing_validation = plumbing_passed(config, &counts);
        let passed_committee_benchmark_research =
            passed_plumbing_validation && research_passed(config, &counts);
        let passed_tentative_signal_quality_review =
            passed_committee_benchmark_research && signal_quality_passed(config, &counts);
        let sufficiency_status = determine_status(
            config,
            &counts,
            passed_plumbing_validation,
            passed_committee_benchmark_research,
            passed_tentative_signal_quality_review,
        );
        let warnings = build_warnings(&counts, passed_tentative_signal_quality_review);
        OfficialEvidenceSufficiencyV2Report {
            sufficiency_id: config.sufficiency_id.clone(),
            total_rows: counts.total_rows,
            official_complete_rows: counts.official_complete_rows,
            symbols: counts.symbols,
            timeframes: counts.timeframes,
            horizons: counts.horizons,
            take_profit_count: counts.take_profit_count,
            stop_loss_count: counts.stop_loss_count,
            time_expired_count: counts.time_expired_count,
            no_trade_counterfactual_count: counts.no_trade_counterfactual_count,
            risk_denied_counterfactual_count: counts.risk_denied_counterfactual_count,
            baseline_reference_count: counts.baseline_reference_count,
            no_lookahead_safe_ratio: counts.no_lookahead_safe_ratio,
            single_symbol_concentration_ratio: counts.single_symbol_concentration_ratio,
            single_outcome_label_ratio: counts.single_outcome_label_ratio,
            counts,
            passed_plumbing_validation,
            passed_committee_benchmark_research,
            passed_tentative_signal_quality_review,
            failed_gates,
            still_insufficient_for_usefulness_claims: true,
            sufficiency_status,
            warnings,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::OfficialEvidenceCounted,
                        ReasonCode::DeterministicPath,
                    ])
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

impl OfficialEvidenceSufficiencyV2Report {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(
            &serde_json::to_string(self).unwrap_or_else(|_| self.sufficiency_id.clone()),
        )
    }

    pub fn to_text(&self) -> String {
        [
            format!("sufficiency_id={}", self.sufficiency_id),
            format!("total_rows={}", self.total_rows),
            format!("official_complete_rows={}", self.official_complete_rows),
            format!("symbols={}", self.symbols),
            format!("timeframes={}", self.timeframes),
            format!("horizons={}", self.horizons),
            format!("take_profit_count={}", self.take_profit_count),
            format!("stop_loss_count={}", self.stop_loss_count),
            format!("time_expired_count={}", self.time_expired_count),
            format!(
                "no_trade_counterfactual_count={}",
                self.no_trade_counterfactual_count
            ),
            format!(
                "risk_denied_counterfactual_count={}",
                self.risk_denied_counterfactual_count
            ),
            format!("baseline_reference_count={}", self.baseline_reference_count),
            format!("no_lookahead_safe_ratio={}", self.no_lookahead_safe_ratio),
            format!(
                "single_symbol_concentration_ratio={}",
                self.single_symbol_concentration_ratio
            ),
            format!(
                "single_outcome_label_ratio={}",
                self.single_outcome_label_ratio
            ),
            format!(
                "passed_plumbing_validation={}",
                self.passed_plumbing_validation
            ),
            format!(
                "passed_committee_benchmark_research={}",
                self.passed_committee_benchmark_research
            ),
            format!(
                "passed_tentative_signal_quality_review={}",
                self.passed_tentative_signal_quality_review
            ),
            format!("failed_gates={}", self.failed_gates.join(" | ")),
            format!(
                "still_insufficient_for_usefulness_claims={}",
                self.still_insufficient_for_usefulness_claims
            ),
            format!("sufficiency_status={:?}", self.sufficiency_status),
            format!("warnings={}", self.warnings.join(" | ")),
            format!("fingerprint={}", self.fingerprint()),
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
            output_dir.join("official_evidence_sufficiency_v2.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("official_evidence_sufficiency_v2_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_official_evidence_sufficiency_v2_from_path_or_config(
    path: &str,
) -> Result<OfficialEvidenceSufficiencyV2Report, String> {
    if path.ends_with(".json") {
        OfficialEvidenceSufficiencyV2Report::from_json_path(Path::new(path))
    } else {
        OfficialEvidenceSufficiencyV2Config::from_toml_path(Path::new(path))
            .and_then(|config| OfficialEvidenceSufficiencyV2Runner::default().run(&config))
    }
}

pub fn compute_counts(
    set: &MultiRowOfficialEvidenceSet,
    outcome_report: Option<&BatchOutcomeLinkageV3Report>,
    counterfactual_report: Option<&BatchCounterfactualCompletionReport>,
) -> OfficialEvidenceSufficiencyV2Counts {
    let symbols = set
        .items
        .iter()
        .map(|item| item.symbol.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let timeframes = set
        .items
        .iter()
        .map(|item| item.timeframe.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let horizons = set
        .items
        .iter()
        .map(|item| item.horizon_bars)
        .collect::<BTreeSet<_>>()
        .len();
    let mut symbol_counts = BTreeMap::<String, usize>::new();
    for item in &set.items {
        *symbol_counts.entry(item.symbol.clone()).or_default() += 1;
    }
    let max_symbol_rows = symbol_counts.values().copied().max().unwrap_or_default();
    let single_symbol_concentration_ratio = if set.total_rows == 0 {
        0.0
    } else {
        max_symbol_rows as f64 / set.total_rows as f64
    };
    let (take_profit_count, stop_loss_count, time_expired_count, single_outcome_label_ratio) =
        if let Some(report) = outcome_report {
            let mut label_counts = BTreeMap::<String, usize>::new();
            for record in &report.records {
                if let Some(reference) = record.outcome_reference.as_ref() {
                    *label_counts
                        .entry(format!("{:?}", reference.triple_barrier_label))
                        .or_default() += 1;
                }
            }
            let total = label_counts.values().sum::<usize>();
            (
                report.take_profit_count,
                report.stop_loss_count,
                report.time_expired_count,
                if total == 0 {
                    0.0
                } else {
                    label_counts.values().copied().max().unwrap_or_default() as f64 / total as f64
                },
            )
        } else {
            (0, 0, 0, 0.0)
        };
    let no_lookahead_safe_ratio = if set.total_rows == 0 {
        0.0
    } else {
        set.no_lookahead_safe_count as f64 / set.total_rows as f64
    };
    OfficialEvidenceSufficiencyV2Counts {
        total_rows: set.total_rows,
        official_complete_rows: set.official_complete_rows,
        symbols,
        timeframes,
        horizons,
        take_profit_count,
        stop_loss_count,
        time_expired_count,
        no_trade_counterfactual_count: counterfactual_report
            .map(|report| report.no_trade_built_count)
            .unwrap_or(set.no_trade_counterfactual_count),
        risk_denied_counterfactual_count: counterfactual_report
            .map(|report| report.risk_denied_built_count)
            .unwrap_or(set.risk_denied_counterfactual_count),
        baseline_reference_count: set.baseline_reference_count,
        no_lookahead_safe_ratio,
        single_symbol_concentration_ratio,
        single_outcome_label_ratio,
        non_crypto_official_rows: set.non_crypto_official_rows,
        crypto_only_rows: set.crypto_only_rows,
        controlled_rows: set.controlled_rows,
    }
}

fn plumbing_passed(
    config: &OfficialEvidenceSufficiencyV2Config,
    counts: &OfficialEvidenceSufficiencyV2Counts,
) -> bool {
    counts.total_rows >= config.min_total_rows
        && counts.official_complete_rows >= config.min_official_complete_rows
        && counts.symbols >= config.min_symbols
        && counts.timeframes >= config.min_timeframes
        && counts.horizons >= config.min_horizons
        && counts.baseline_reference_count >= config.min_baseline_references
        && counts.no_lookahead_safe_ratio >= config.min_no_lookahead_safe_ratio
        && (!config.require_non_crypto_official
            || counts.non_crypto_official_rows
                == counts
                    .total_rows
                    .saturating_sub(counts.crypto_only_rows + counts.controlled_rows))
}

fn research_passed(
    config: &OfficialEvidenceSufficiencyV2Config,
    counts: &OfficialEvidenceSufficiencyV2Counts,
) -> bool {
    (!config.require_outcome_diversity
        || (counts.take_profit_count >= config.min_take_profit_outcomes
            && counts.stop_loss_count >= config.min_stop_loss_outcomes
            && counts.time_expired_count >= config.min_time_expired_outcomes))
        && (!config.require_counterfactual_diversity
            || (counts.no_trade_counterfactual_count >= config.min_no_trade_counterfactuals
                && counts.risk_denied_counterfactual_count
                    >= config.min_risk_denied_counterfactuals))
}

fn signal_quality_passed(
    config: &OfficialEvidenceSufficiencyV2Config,
    counts: &OfficialEvidenceSufficiencyV2Counts,
) -> bool {
    counts.single_symbol_concentration_ratio <= config.max_single_symbol_concentration_ratio
        && counts.single_outcome_label_ratio <= config.max_single_outcome_label_ratio
}

fn collect_failed_gates(
    config: &OfficialEvidenceSufficiencyV2Config,
    counts: &OfficialEvidenceSufficiencyV2Counts,
) -> Vec<String> {
    let mut failed = Vec::new();
    if counts.total_rows < config.min_total_rows {
        failed.push(format!(
            "total_rows {} < min_total_rows {}",
            counts.total_rows, config.min_total_rows
        ));
    }
    if counts.official_complete_rows < config.min_official_complete_rows {
        failed.push(format!(
            "official_complete_rows {} < min_official_complete_rows {}",
            counts.official_complete_rows, config.min_official_complete_rows
        ));
    }
    if counts.symbols < config.min_symbols {
        failed.push(format!(
            "symbols {} < min_symbols {}",
            counts.symbols, config.min_symbols
        ));
    }
    if counts.timeframes < config.min_timeframes {
        failed.push(format!(
            "timeframes {} < min_timeframes {}",
            counts.timeframes, config.min_timeframes
        ));
    }
    if counts.horizons < config.min_horizons {
        failed.push(format!(
            "horizons {} < min_horizons {}",
            counts.horizons, config.min_horizons
        ));
    }
    if counts.baseline_reference_count < config.min_baseline_references {
        failed.push(format!(
            "baseline_reference_count {} < min_baseline_references {}",
            counts.baseline_reference_count, config.min_baseline_references
        ));
    }
    if counts.no_lookahead_safe_ratio < config.min_no_lookahead_safe_ratio {
        failed.push(format!(
            "no_lookahead_safe_ratio {} < min_no_lookahead_safe_ratio {}",
            counts.no_lookahead_safe_ratio, config.min_no_lookahead_safe_ratio
        ));
    }
    if config.require_outcome_diversity {
        if counts.take_profit_count < config.min_take_profit_outcomes {
            failed.push(format!(
                "take_profit_count {} < min_take_profit_outcomes {}",
                counts.take_profit_count, config.min_take_profit_outcomes
            ));
        }
        if counts.stop_loss_count < config.min_stop_loss_outcomes {
            failed.push(format!(
                "stop_loss_count {} < min_stop_loss_outcomes {}",
                counts.stop_loss_count, config.min_stop_loss_outcomes
            ));
        }
        if counts.time_expired_count < config.min_time_expired_outcomes {
            failed.push(format!(
                "time_expired_count {} < min_time_expired_outcomes {}",
                counts.time_expired_count, config.min_time_expired_outcomes
            ));
        }
    }
    if config.require_counterfactual_diversity {
        if counts.no_trade_counterfactual_count < config.min_no_trade_counterfactuals {
            failed.push(format!(
                "no_trade_counterfactual_count {} < min_no_trade_counterfactuals {}",
                counts.no_trade_counterfactual_count, config.min_no_trade_counterfactuals
            ));
        }
        if counts.risk_denied_counterfactual_count < config.min_risk_denied_counterfactuals {
            failed.push(format!(
                "risk_denied_counterfactual_count {} < min_risk_denied_counterfactuals {}",
                counts.risk_denied_counterfactual_count, config.min_risk_denied_counterfactuals
            ));
        }
    }
    if counts.single_symbol_concentration_ratio > config.max_single_symbol_concentration_ratio {
        failed.push(format!(
            "single_symbol_concentration_ratio {} > max_single_symbol_concentration_ratio {}",
            counts.single_symbol_concentration_ratio, config.max_single_symbol_concentration_ratio
        ));
    }
    if counts.single_outcome_label_ratio > config.max_single_outcome_label_ratio {
        failed.push(format!(
            "single_outcome_label_ratio {} > max_single_outcome_label_ratio {}",
            counts.single_outcome_label_ratio, config.max_single_outcome_label_ratio
        ));
    }
    failed
}

fn determine_status(
    config: &OfficialEvidenceSufficiencyV2Config,
    counts: &OfficialEvidenceSufficiencyV2Counts,
    plumbing: bool,
    research: bool,
    signal_quality: bool,
) -> OfficialEvidenceSufficiencyV2Status {
    if signal_quality {
        return OfficialEvidenceSufficiencyV2Status::TentativeSignalQualityReviewReady;
    }
    if research {
        return OfficialEvidenceSufficiencyV2Status::CommitteeBenchmarkResearchReady;
    }
    if plumbing {
        return OfficialEvidenceSufficiencyV2Status::PlumbingValidated;
    }
    if counts.total_rows < config.min_total_rows {
        return OfficialEvidenceSufficiencyV2Status::InsufficientRows;
    }
    if counts.official_complete_rows < config.min_official_complete_rows {
        return OfficialEvidenceSufficiencyV2Status::InsufficientOfficialCompleteRows;
    }
    if counts.symbols < config.min_symbols
        || counts.timeframes < config.min_timeframes
        || counts.horizons < config.min_horizons
    {
        return OfficialEvidenceSufficiencyV2Status::InsufficientSymbolDiversity;
    }
    if config.require_outcome_diversity
        && (counts.take_profit_count < config.min_take_profit_outcomes
            || counts.stop_loss_count < config.min_stop_loss_outcomes
            || counts.time_expired_count < config.min_time_expired_outcomes)
    {
        return OfficialEvidenceSufficiencyV2Status::InsufficientOutcomeDiversity;
    }
    if config.require_counterfactual_diversity
        && (counts.no_trade_counterfactual_count < config.min_no_trade_counterfactuals
            || counts.risk_denied_counterfactual_count < config.min_risk_denied_counterfactuals)
    {
        return OfficialEvidenceSufficiencyV2Status::InsufficientCounterfactualDepth;
    }
    if counts.single_symbol_concentration_ratio > config.max_single_symbol_concentration_ratio {
        return OfficialEvidenceSufficiencyV2Status::SingleSymbolDominated;
    }
    if counts.single_outcome_label_ratio > config.max_single_outcome_label_ratio {
        return OfficialEvidenceSufficiencyV2Status::SingleOutcomeDominated;
    }
    OfficialEvidenceSufficiencyV2Status::NeedMoreEvidence
}

fn build_warnings(
    counts: &OfficialEvidenceSufficiencyV2Counts,
    signal_quality: bool,
) -> Vec<String> {
    let mut warnings = vec![
        "passing sufficiency gates does not imply profitability, live readiness, or real-money use"
            .to_string(),
    ];
    if counts.official_complete_rows <= 1 {
        warnings.push(
            "one official complete row remains far too small for usefulness claims".to_string(),
        );
    }
    if counts.crypto_only_rows > 0 || counts.controlled_rows > 0 {
        warnings.push(
            "crypto-only or controlled rows do not satisfy official evidence sufficiency gates"
                .to_string(),
        );
    }
    if !signal_quality {
        warnings
            .push("signal-quality review remains conservative and sample-size bound".to_string());
    }
    warnings
}

fn default_output_root() -> String {
    "target/soma_official_evidence_sufficiency_v2".to_string()
}

fn default_min_total_rows() -> usize {
    20
}

fn default_min_official_complete_rows() -> usize {
    20
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

fn default_min_take_profit_outcomes() -> usize {
    1
}

fn default_min_stop_loss_outcomes() -> usize {
    1
}

fn default_min_time_expired_outcomes() -> usize {
    1
}

fn default_min_no_trade_counterfactuals() -> usize {
    10
}

fn default_min_risk_denied_counterfactuals() -> usize {
    10
}

fn default_min_baseline_references() -> usize {
    20
}

fn default_min_no_lookahead_safe_ratio() -> f64 {
    1.0
}

fn default_max_single_symbol_concentration_ratio() -> f64 {
    0.75
}

fn default_max_single_outcome_label_ratio() -> f64 {
    0.80
}

fn default_true() -> bool {
    true
}
