use std::collections::BTreeMap;
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
use super::committee_outcome_reference::CommitteeTripleBarrierLabel;
use super::multi_row_official_evidence::{
    MultiRowOfficialEvidenceSet, load_multi_row_official_evidence_set_from_path_or_config,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeDiversityAuditConfig {
    pub audit_id: String,
    #[serde(default)]
    pub batch_outcome_linkage_paths: Vec<String>,
    #[serde(default)]
    pub batch_counterfactual_completion_paths: Vec<String>,
    #[serde(default)]
    pub multi_row_set_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_min_total_outcomes")]
    pub min_total_outcomes: usize,
    #[serde(default = "default_max_single_outcome_label_ratio")]
    pub max_single_outcome_label_ratio: f64,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OutcomeDiversityStatus {
    HealthyOutcomeDiversity,
    SingleOutcomeDominated,
    MissingStopLoss,
    MissingTimeExpired,
    MissingTakeProfit,
    TooFewOutcomes,
    DiagnosticOnly,
    #[default]
    InsufficientDiversity,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeDiversityAuditReport {
    pub audit_id: String,
    pub total_outcomes: usize,
    pub official_outcomes: usize,
    pub take_profit_count: usize,
    pub stop_loss_count: usize,
    pub time_expired_count: usize,
    pub no_trade_counterfactual_count: usize,
    pub risk_denied_counterfactual_count: usize,
    pub single_outcome_label_ratio: f64,
    pub outcome_entropy: f64,
    pub outcome_diversity_status: OutcomeDiversityStatus,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OutcomeDiversityAuditRunner;

impl Default for OutcomeDiversityAuditConfig {
    fn default() -> Self {
        Self {
            audit_id: "outcome-diversity-audit".to_string(),
            batch_outcome_linkage_paths: Vec::new(),
            batch_counterfactual_completion_paths: Vec::new(),
            multi_row_set_paths: Vec::new(),
            output_root: default_output_root(),
            min_total_outcomes: default_min_total_outcomes(),
            max_single_outcome_label_ratio: default_max_single_outcome_label_ratio(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OutcomeDiversityAuditConfig {
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
        if self.audit_id.trim().is_empty() {
            return Err("outcome diversity audit id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("outcome diversity audit paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.audit_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.batch_outcome_linkage_paths
            .iter()
            .chain(self.batch_counterfactual_completion_paths.iter())
            .chain(self.multi_row_set_paths.iter())
            .cloned()
            .collect()
    }
}

impl OutcomeDiversityAuditRunner {
    pub fn run(
        &self,
        config: &OutcomeDiversityAuditConfig,
    ) -> Result<OutcomeDiversityAuditReport, String> {
        config.validate()?;
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
        let set = config
            .multi_row_set_paths
            .first()
            .map(|path| load_multi_row_official_evidence_set_from_path_or_config(path))
            .transpose()?;
        Ok(self.run_from_inputs(
            config,
            outcome_report.as_ref(),
            counterfactual_report.as_ref(),
            set.as_ref(),
        ))
    }

    pub fn run_from_inputs(
        &self,
        config: &OutcomeDiversityAuditConfig,
        outcome_report: Option<&BatchOutcomeLinkageV3Report>,
        counterfactual_report: Option<&BatchCounterfactualCompletionReport>,
        set: Option<&MultiRowOfficialEvidenceSet>,
    ) -> OutcomeDiversityAuditReport {
        let mut label_counts = BTreeMap::<CommitteeTripleBarrierLabel, usize>::new();
        let mut total_outcomes = 0usize;
        let mut official_outcomes = 0usize;
        let mut diagnostic_only = false;
        if let Some(report) = outcome_report {
            for record in &report.records {
                if let Some(reference) = record.outcome_reference.as_ref() {
                    if matches!(
                        reference.triple_barrier_label,
                        CommitteeTripleBarrierLabel::TakeProfit
                            | CommitteeTripleBarrierLabel::StopLoss
                            | CommitteeTripleBarrierLabel::TimeExpired
                    ) {
                        total_outcomes += 1;
                        *label_counts
                            .entry(reference.triple_barrier_label)
                            .or_insert(0usize) += 1;
                        if reference.no_lookahead_safe {
                            official_outcomes += 1;
                        }
                    }
                } else if matches!(
                    record.status,
                    super::outcome_linkage_v3::OutcomeLinkageV3RecordStatus::DiagnosticOnly
                ) {
                    diagnostic_only = true;
                }
            }
        }
        if let Some(set) = set {
            diagnostic_only |= set.official_complete_rows == 0
                && (set.controlled_rows > 0
                    || set.crypto_only_rows > 0
                    || set.yfinance_rows > 0
                    || set.fixture_rows > 0);
        }
        let take_profit_count = *label_counts
            .get(&CommitteeTripleBarrierLabel::TakeProfit)
            .unwrap_or(&0);
        let stop_loss_count = *label_counts
            .get(&CommitteeTripleBarrierLabel::StopLoss)
            .unwrap_or(&0);
        let time_expired_count = *label_counts
            .get(&CommitteeTripleBarrierLabel::TimeExpired)
            .unwrap_or(&0);
        let single_outcome_label_ratio = if total_outcomes == 0 {
            0.0
        } else {
            label_counts.values().copied().max().unwrap_or_default() as f64 / total_outcomes as f64
        };
        let outcome_entropy = entropy(&[take_profit_count, stop_loss_count, time_expired_count]);
        let no_trade_counterfactual_count = counterfactual_report
            .map(|report| report.no_trade_built_count)
            .unwrap_or_default();
        let risk_denied_counterfactual_count = counterfactual_report
            .map(|report| report.risk_denied_built_count)
            .unwrap_or_default();
        let outcome_diversity_status = determine_status(
            config,
            diagnostic_only,
            total_outcomes,
            take_profit_count,
            stop_loss_count,
            time_expired_count,
            single_outcome_label_ratio,
        );
        let blockers = build_blockers(
            config,
            total_outcomes,
            take_profit_count,
            stop_loss_count,
            time_expired_count,
            single_outcome_label_ratio,
            diagnostic_only,
        );
        let warnings = vec![
            "outcome diversity remains research-only; mixed labels never imply profitability"
                .to_string(),
        ];
        OutcomeDiversityAuditReport {
            audit_id: config.audit_id.clone(),
            total_outcomes,
            official_outcomes,
            take_profit_count,
            stop_loss_count,
            time_expired_count,
            no_trade_counterfactual_count,
            risk_denied_counterfactual_count,
            single_outcome_label_ratio,
            outcome_entropy,
            outcome_diversity_status,
            blockers,
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

impl OutcomeDiversityAuditReport {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_else(|_| self.audit_id.clone()))
    }

    pub fn to_text(&self) -> String {
        [
            format!("audit_id={}", self.audit_id),
            format!("total_outcomes={}", self.total_outcomes),
            format!("official_outcomes={}", self.official_outcomes),
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
            format!(
                "single_outcome_label_ratio={}",
                self.single_outcome_label_ratio
            ),
            format!("outcome_entropy={}", self.outcome_entropy),
            format!(
                "outcome_diversity_status={:?}",
                self.outcome_diversity_status
            ),
            format!("blockers={}", self.blockers.join(" | ")),
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
            output_dir.join("outcome_diversity_audit.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("outcome_diversity_audit.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_outcome_diversity_audit_from_path_or_config(
    path: &str,
) -> Result<OutcomeDiversityAuditReport, String> {
    if path.ends_with(".json") {
        OutcomeDiversityAuditReport::from_json_path(Path::new(path))
    } else {
        OutcomeDiversityAuditConfig::from_toml_path(Path::new(path))
            .and_then(|config| OutcomeDiversityAuditRunner::default().run(&config))
    }
}

fn determine_status(
    config: &OutcomeDiversityAuditConfig,
    diagnostic_only: bool,
    total_outcomes: usize,
    take_profit_count: usize,
    stop_loss_count: usize,
    time_expired_count: usize,
    single_outcome_label_ratio: f64,
) -> OutcomeDiversityStatus {
    if diagnostic_only && total_outcomes == 0 {
        return OutcomeDiversityStatus::DiagnosticOnly;
    }
    if total_outcomes < config.min_total_outcomes {
        return OutcomeDiversityStatus::TooFewOutcomes;
    }
    if single_outcome_label_ratio > config.max_single_outcome_label_ratio {
        return OutcomeDiversityStatus::SingleOutcomeDominated;
    }
    if take_profit_count == 0 {
        return OutcomeDiversityStatus::MissingTakeProfit;
    }
    if stop_loss_count == 0 {
        return OutcomeDiversityStatus::MissingStopLoss;
    }
    if time_expired_count == 0 {
        return OutcomeDiversityStatus::MissingTimeExpired;
    }
    OutcomeDiversityStatus::HealthyOutcomeDiversity
}

fn build_blockers(
    config: &OutcomeDiversityAuditConfig,
    total_outcomes: usize,
    take_profit_count: usize,
    stop_loss_count: usize,
    time_expired_count: usize,
    single_outcome_label_ratio: f64,
    diagnostic_only: bool,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if diagnostic_only {
        blockers.push(
            "diagnostic-only or ineligible evidence cannot satisfy official outcome diversity"
                .to_string(),
        );
    }
    if total_outcomes < config.min_total_outcomes {
        blockers.push(format!(
            "total_outcomes {} < min_total_outcomes {}",
            total_outcomes, config.min_total_outcomes
        ));
    }
    if take_profit_count == 0 {
        blockers.push("take_profit_count 0 < required 1".to_string());
    }
    if stop_loss_count == 0 {
        blockers.push("stop_loss_count 0 < required 1".to_string());
    }
    if time_expired_count == 0 {
        blockers.push("time_expired_count 0 < required 1".to_string());
    }
    if single_outcome_label_ratio > config.max_single_outcome_label_ratio {
        blockers.push(format!(
            "single_outcome_label_ratio {} > max_single_outcome_label_ratio {}",
            single_outcome_label_ratio, config.max_single_outcome_label_ratio
        ));
    }
    blockers
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
    "target/soma_outcome_diversity_audit".to_string()
}

fn default_min_total_outcomes() -> usize {
    2
}

fn default_max_single_outcome_label_ratio() -> f64 {
    0.8
}
