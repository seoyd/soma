use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};
use super::comparable_evidence_builder::ComparableEvidenceBuilder;
use super::complete_row_closure_bundle::CompleteRowClosureBundle;
use super::outcome_linkage_v3::{
    OutcomeLinkageV3Report, load_outcome_linkage_v3_from_path_or_config,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterfactualCompletionV2Config {
    pub completion_id: String,
    #[serde(default)]
    pub outcome_linkage_v3_path: Option<String>,
    #[serde(default)]
    pub complete_row_closure_path: Option<String>,
    #[serde(default)]
    pub risk_decision_paths: Vec<String>,
    #[serde(default)]
    pub committee_decision_paths: Vec<String>,
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
pub enum CounterfactualCompletionV2RecordStatus {
    Completed,
    Partial,
    SkippedMissingOutcome,
    SkippedMissingRiskDecision,
    SkippedMissingCommitteeDecision,
    SkippedSourceIneligible,
    RejectedNoLookahead,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CounterfactualCompletionRecord {
    pub row_id: String,
    pub no_trade_counterfactual_built: bool,
    pub risk_denied_counterfactual_built: bool,
    #[serde(default)]
    pub avoided_loss_value: Option<f64>,
    #[serde(default)]
    pub missed_gain_value: Option<f64>,
    pub diagnostic_only: bool,
    pub status: CounterfactualCompletionV2RecordStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CounterfactualCompletionV2Status {
    CounterfactualsImproved,
    OfficialCounterfactualsImproved,
    StillNeedOutcomeReferences,
    StillNeedRiskDecisions,
    StillNeedCommitteeDecisions,
    SourceIneligible,
    DiagnosticOnly,
    #[default]
    NoImprovement,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CounterfactualCompletionV2Report {
    pub completion_id: String,
    pub records: Vec<CounterfactualCompletionRecord>,
    pub completed_count: usize,
    pub no_trade_built_count: usize,
    pub risk_denied_built_count: usize,
    pub skipped_missing_outcome_count: usize,
    pub skipped_missing_risk_decision_count: usize,
    pub official_counterfactual_count: usize,
    pub diagnostic_counterfactual_count: usize,
    pub completion_status: CounterfactualCompletionV2Status,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CounterfactualCompletionV2Runner;

impl Default for CounterfactualCompletionV2Config {
    fn default() -> Self {
        Self {
            completion_id: "counterfactual-completion-v2".to_string(),
            outcome_linkage_v3_path: None,
            complete_row_closure_path: None,
            risk_decision_paths: Vec::new(),
            committee_decision_paths: Vec::new(),
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

impl CounterfactualCompletionV2Config {
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
        if self.completion_id.trim().is_empty() {
            return Err("counterfactual completion v2 id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("counterfactual completion v2 paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.completion_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.outcome_linkage_v3_path
            .iter()
            .cloned()
            .chain(self.complete_row_closure_path.iter().cloned())
            .chain(self.risk_decision_paths.iter().cloned())
            .chain(self.committee_decision_paths.iter().cloned())
            .collect()
    }
}

impl CounterfactualCompletionV2Runner {
    pub fn run(
        &self,
        config: &CounterfactualCompletionV2Config,
    ) -> Result<CounterfactualCompletionV2Report, String> {
        config.validate()?;
        let outcome_report = load_outcomes(config)?;
        let rows = load_rows(config)?;
        self.run_from_inputs(config, &outcome_report, &rows)
    }

    pub fn run_from_inputs(
        &self,
        config: &CounterfactualCompletionV2Config,
        outcome_report: &OutcomeLinkageV3Report,
        rows: &[ComparableCommitteeEvidenceRow],
    ) -> Result<CounterfactualCompletionV2Report, String> {
        config.validate()?;
        let outcome_map = outcome_report
            .records
            .iter()
            .map(|record| (record.row_id.clone(), record))
            .collect::<BTreeMap<_, _>>();
        let mut records = rows
            .iter()
            .map(|row| build_record(config, row, outcome_map.get(&row.row_id)))
            .collect::<Vec<_>>();
        records.sort_by(|left, right| left.row_id.cmp(&right.row_id));
        let completed_count = records
            .iter()
            .filter(|record| {
                matches!(
                    record.status,
                    CounterfactualCompletionV2RecordStatus::Completed
                        | CounterfactualCompletionV2RecordStatus::DiagnosticOnly
                )
            })
            .count();
        let no_trade_built_count = records
            .iter()
            .filter(|record| record.no_trade_counterfactual_built)
            .count();
        let risk_denied_built_count = records
            .iter()
            .filter(|record| record.risk_denied_counterfactual_built)
            .count();
        let skipped_missing_outcome_count = records
            .iter()
            .filter(|record| {
                record.status == CounterfactualCompletionV2RecordStatus::SkippedMissingOutcome
            })
            .count();
        let skipped_missing_risk_decision_count = records
            .iter()
            .filter(|record| {
                record.status == CounterfactualCompletionV2RecordStatus::SkippedMissingRiskDecision
            })
            .count();
        let official_counterfactual_count = records
            .iter()
            .filter(|record| {
                (record.no_trade_counterfactual_built || record.risk_denied_counterfactual_built)
                    && !record.diagnostic_only
            })
            .count();
        let diagnostic_counterfactual_count = records
            .iter()
            .filter(|record| {
                (record.no_trade_counterfactual_built || record.risk_denied_counterfactual_built)
                    && record.diagnostic_only
            })
            .count();
        let completion_status = determine_status(&records);
        Ok(CounterfactualCompletionV2Report {
            completion_id: config.completion_id.clone(),
            records,
            completed_count,
            no_trade_built_count,
            risk_denied_built_count,
            skipped_missing_outcome_count,
            skipped_missing_risk_decision_count,
            official_counterfactual_count,
            diagnostic_counterfactual_count,
            completion_status,
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

impl CounterfactualCompletionV2Report {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(
            &serde_json::to_string(self).unwrap_or_else(|_| self.completion_id.clone()),
        )
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("completion_id={}", self.completion_id),
            format!("completed_count={}", self.completed_count),
            format!("no_trade_built_count={}", self.no_trade_built_count),
            format!("risk_denied_built_count={}", self.risk_denied_built_count),
            format!(
                "skipped_missing_outcome_count={}",
                self.skipped_missing_outcome_count
            ),
            format!(
                "skipped_missing_risk_decision_count={}",
                self.skipped_missing_risk_decision_count
            ),
            format!(
                "official_counterfactual_count={}",
                self.official_counterfactual_count
            ),
            format!(
                "diagnostic_counterfactual_count={}",
                self.diagnostic_counterfactual_count
            ),
            format!("completion_status={:?}", self.completion_status),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.records.iter().map(|record| {
            format!(
                "row_id={};status={:?};no_trade_counterfactual_built={};risk_denied_counterfactual_built={};avoided_loss_value={};missed_gain_value={};diagnostic_only={}",
                record.row_id,
                record.status,
                record.no_trade_counterfactual_built,
                record.risk_denied_counterfactual_built,
                record.avoided_loss_value.map(|value| value.to_string()).unwrap_or_default(),
                record.missed_gain_value.map(|value| value.to_string()).unwrap_or_default(),
                record.diagnostic_only,
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
            output_dir.join("counterfactual_completion_v2.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("counterfactual_completion_v2_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_counterfactual_completion_v2_from_path_or_config(
    path: &str,
) -> Result<CounterfactualCompletionV2Report, String> {
    if path.ends_with(".json") {
        CounterfactualCompletionV2Report::from_json_path(Path::new(path))
    } else {
        CounterfactualCompletionV2Config::from_toml_path(Path::new(path))
            .and_then(|config| CounterfactualCompletionV2Runner::default().run(&config))
    }
}

fn build_record(
    config: &CounterfactualCompletionV2Config,
    row: &ComparableCommitteeEvidenceRow,
    outcome_record: Option<&&super::outcome_linkage_v3::OutcomeLinkageV3Record>,
) -> CounterfactualCompletionRecord {
    let mut reason_codes = row.reason_codes.clone();
    let diagnostic_only = matches!(
        row.source_class,
        ComparableEvidenceSourceClass::ControlledDiagnostic
            | ComparableEvidenceSourceClass::OfficialCryptoOnly
            | ComparableEvidenceSourceClass::YFinanceResearch
            | ComparableEvidenceSourceClass::FixtureArchitectureTest
            | ComparableEvidenceSourceClass::SyntheticTest
    ) || row.diagnostic_only;

    if config.require_no_lookahead_safe && !row.no_lookahead_safe {
        reason_codes.push(ReasonCode::RejectedNoLookaheadReference);
        return CounterfactualCompletionRecord {
            row_id: row.row_id.clone(),
            no_trade_counterfactual_built: false,
            risk_denied_counterfactual_built: false,
            avoided_loss_value: None,
            missed_gain_value: None,
            diagnostic_only,
            status: CounterfactualCompletionV2RecordStatus::RejectedNoLookahead,
            reason_codes: stable_reason_codes(&reason_codes),
        };
    }
    if matches!(
        row.source_class,
        ComparableEvidenceSourceClass::YFinanceResearch
            | ComparableEvidenceSourceClass::FixtureArchitectureTest
            | ComparableEvidenceSourceClass::SyntheticTest
    ) {
        reason_codes.push(ReasonCode::ReadinessEvidenceExcluded);
        return CounterfactualCompletionRecord {
            row_id: row.row_id.clone(),
            no_trade_counterfactual_built: false,
            risk_denied_counterfactual_built: false,
            avoided_loss_value: None,
            missed_gain_value: None,
            diagnostic_only,
            status: CounterfactualCompletionV2RecordStatus::SkippedSourceIneligible,
            reason_codes: stable_reason_codes(&reason_codes),
        };
    }
    let outcome_return = outcome_record
        .and_then(|record| record.outcome_reference.as_ref())
        .and_then(|reference| reference.net_return_pct)
        .or(row.net_return_pct);
    if config.require_outcome_reference && outcome_return.is_none() {
        reason_codes.push(ReasonCode::CommitteeCounterfactualUnavailable);
        return CounterfactualCompletionRecord {
            row_id: row.row_id.clone(),
            no_trade_counterfactual_built: false,
            risk_denied_counterfactual_built: false,
            avoided_loss_value: None,
            missed_gain_value: None,
            diagnostic_only,
            status: CounterfactualCompletionV2RecordStatus::SkippedMissingOutcome,
            reason_codes: stable_reason_codes(&reason_codes),
        };
    }
    let outcome_return = outcome_return.unwrap_or_default();
    let (avoided_loss_value, missed_gain_value) = avoided_loss_and_missed_gain(
        outcome_return,
        config.credit_avoided_loss_factor,
        config.penalize_missed_gain_factor,
        config.max_missed_gain_penalty,
    );
    if avoided_loss_value.is_some() {
        reason_codes.push(ReasonCode::AvoidedLossRecorded);
    }
    if missed_gain_value.is_some() {
        reason_codes.push(ReasonCode::MissedGainRecorded);
    }
    let no_trade_counterfactual_built =
        config.build_no_trade && !row.no_trade_counterfactual_available;
    if no_trade_counterfactual_built {
        reason_codes.push(ReasonCode::NoTradeCounterfactual);
    }

    let risk_denied_applicable = row
        .risk_governor_decision
        .as_deref()
        .map(is_risk_denied_decision)
        .unwrap_or(false);
    let risk_denied_counterfactual_built = config.build_risk_denied
        && !row.risk_denied_counterfactual_available
        && risk_denied_applicable
        && row.committee_final_action.trim().is_empty().not();

    let status = if config.build_risk_denied
        && row
            .risk_governor_decision
            .as_ref()
            .is_none_or(|value| value.trim().is_empty())
        && !row.risk_denied_counterfactual_available
    {
        reason_codes.push(ReasonCode::RiskDeniedCounterfactual);
        CounterfactualCompletionV2RecordStatus::SkippedMissingRiskDecision
    } else if config.build_risk_denied
        && row.committee_final_action.trim().is_empty()
        && !row.risk_denied_counterfactual_available
    {
        reason_codes.push(ReasonCode::CounterfactualEvaluated);
        CounterfactualCompletionV2RecordStatus::SkippedMissingCommitteeDecision
    } else if diagnostic_only && (no_trade_counterfactual_built || risk_denied_counterfactual_built)
    {
        CounterfactualCompletionV2RecordStatus::DiagnosticOnly
    } else if no_trade_counterfactual_built
        && (!config.build_risk_denied
            || risk_denied_counterfactual_built
            || row.risk_denied_counterfactual_available
            || !risk_denied_applicable)
    {
        CounterfactualCompletionV2RecordStatus::Completed
    } else if risk_denied_counterfactual_built {
        reason_codes.push(ReasonCode::RiskDeniedCounterfactual);
        CounterfactualCompletionV2RecordStatus::Completed
    } else if no_trade_counterfactual_built {
        CounterfactualCompletionV2RecordStatus::Partial
    } else {
        CounterfactualCompletionV2RecordStatus::Partial
    };

    CounterfactualCompletionRecord {
        row_id: row.row_id.clone(),
        no_trade_counterfactual_built,
        risk_denied_counterfactual_built,
        avoided_loss_value,
        missed_gain_value,
        diagnostic_only,
        status,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn avoided_loss_and_missed_gain(
    outcome_return: f64,
    credit_avoided_loss_factor: f64,
    penalize_missed_gain_factor: f64,
    max_missed_gain_penalty: f64,
) -> (Option<f64>, Option<f64>) {
    if outcome_return < 0.0 {
        (
            Some((-outcome_return) * credit_avoided_loss_factor.max(0.0)),
            None,
        )
    } else if outcome_return > 0.0 {
        (
            None,
            Some(
                (outcome_return * penalize_missed_gain_factor.max(0.0))
                    .min(max_missed_gain_penalty.max(0.0)),
            ),
        )
    } else {
        (None, None)
    }
}

fn determine_status(
    records: &[CounterfactualCompletionRecord],
) -> CounterfactualCompletionV2Status {
    if records.is_empty() {
        return CounterfactualCompletionV2Status::NoImprovement;
    }
    if records.iter().any(|record| {
        record.status == CounterfactualCompletionV2RecordStatus::Completed
            && !record.diagnostic_only
    }) {
        return CounterfactualCompletionV2Status::OfficialCounterfactualsImproved;
    }
    if records.iter().any(|record| {
        matches!(
            record.status,
            CounterfactualCompletionV2RecordStatus::Completed
                | CounterfactualCompletionV2RecordStatus::DiagnosticOnly
        )
    }) {
        return CounterfactualCompletionV2Status::CounterfactualsImproved;
    }
    if records.iter().any(|record| {
        record.status == CounterfactualCompletionV2RecordStatus::SkippedMissingOutcome
    }) {
        return CounterfactualCompletionV2Status::StillNeedOutcomeReferences;
    }
    if records.iter().any(|record| {
        record.status == CounterfactualCompletionV2RecordStatus::SkippedMissingRiskDecision
    }) {
        return CounterfactualCompletionV2Status::StillNeedRiskDecisions;
    }
    if records.iter().any(|record| {
        record.status == CounterfactualCompletionV2RecordStatus::SkippedMissingCommitteeDecision
    }) {
        return CounterfactualCompletionV2Status::StillNeedCommitteeDecisions;
    }
    if records.iter().all(|record| {
        record.status == CounterfactualCompletionV2RecordStatus::SkippedSourceIneligible
    }) {
        return CounterfactualCompletionV2Status::SourceIneligible;
    }
    if records.iter().all(|record| record.diagnostic_only) {
        return CounterfactualCompletionV2Status::DiagnosticOnly;
    }
    CounterfactualCompletionV2Status::NoImprovement
}

fn is_risk_denied_decision(value: &str) -> bool {
    let lowered = value.trim().to_ascii_lowercase();
    lowered.contains("deny") || lowered.contains("block") || lowered.contains("reject")
}

fn load_outcomes(
    config: &CounterfactualCompletionV2Config,
) -> Result<OutcomeLinkageV3Report, String> {
    if let Some(path) = config.outcome_linkage_v3_path.as_deref() {
        load_outcome_linkage_v3_from_path_or_config(path)
    } else {
        Err("counterfactual completion v2 requires outcome_linkage_v3_path".to_string())
    }
}

fn load_rows(
    config: &CounterfactualCompletionV2Config,
) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    let Some(path) = config.complete_row_closure_path.as_deref() else {
        return Ok(Vec::new());
    };
    if path.ends_with(".json") {
        if let Ok(bundle) = CompleteRowClosureBundle::from_json_path(Path::new(path)) {
            return Ok(bundle.complete_comparable_row_bundle.rows);
        }
        if let Ok(bundle) = ComparableCommitteeEvidenceBundle::from_json_path(Path::new(path)) {
            return Ok(bundle.rows);
        }
    }
    let comparable_config = ComparableCommitteeEvidenceConfig::from_toml_path(Path::new(path))?;
    Ok(ComparableEvidenceBuilder::default()
        .build(&comparable_config)?
        .rows)
}

trait BoolExt {
    fn not(self) -> bool;
}

impl BoolExt for bool {
    fn not(self) -> bool {
        !self
    }
}

fn default_output_root() -> String {
    "target/soma_counterfactual_completion_v2".to_string()
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
