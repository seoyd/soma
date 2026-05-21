use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::baseline_reference_generator::BaselineReferenceSource;
use super::committee_counterfactual_builder::CommitteeCounterfactualRecord;
use super::committee_outcome_linker::{
    CommitteeOutcomeLinkSummary, OutcomeLinkedCommitteeScenarioPack,
    OutcomeLinkedCommitteeScenarioRow,
};
use super::committee_outcome_reference::{
    CommitteeBaselineReference, CommitteeExternalReference, CommitteeOutcomeReference,
};
use super::committee_scenario_loader::{CommitteeScenarioRow, CommitteeScenarioSet};
use super::official_committee_pack::{OfficialCommitteeScenarioPack, classify_row};
use super::triple_barrier_reference_builder::TripleBarrierReferenceSource;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeReferencePackConfig {
    pub reference_pack_id: String,
    #[serde(default)]
    pub scenario_pack_paths: Vec<String>,
    #[serde(default)]
    pub scenario_set_paths: Vec<String>,
    #[serde(default)]
    pub candle_series_paths: Vec<String>,
    #[serde(default)]
    pub baseline_reference_paths: Vec<String>,
    #[serde(default)]
    pub external_prediction_paths: Vec<String>,
    pub output_root: String,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_horizon_bars")]
    pub default_horizon_bars: usize,
    #[serde(default = "default_take_profit_pct")]
    pub default_take_profit_pct: f64,
    #[serde(default = "default_stop_loss_pct")]
    pub default_stop_loss_pct: f64,
    #[serde(default = "default_cost_bps")]
    pub default_cost_bps: f64,
    #[serde(default = "default_slippage_bps")]
    pub default_slippage_bps: f64,
    #[serde(default)]
    pub timestamp_tolerance_ms: u64,
    #[serde(default = "default_true")]
    pub require_exact_symbol_match: bool,
    #[serde(default = "default_true")]
    pub require_exact_horizon_match: bool,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default = "default_true")]
    pub build_triple_barrier_outcomes: bool,
    #[serde(default = "default_true")]
    pub build_no_trade_counterfactuals: bool,
    #[serde(default = "default_true")]
    pub build_risk_denied_counterfactuals: bool,
    #[serde(default = "default_true")]
    pub build_baseline_references: bool,
    #[serde(default)]
    pub allow_estimated_references: bool,
    #[serde(default)]
    pub allow_controlled_fixture_references: bool,
    #[serde(default)]
    pub allow_yfinance_research: bool,
    #[serde(default)]
    pub allow_fixture: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GeneratedReferenceKind {
    TripleBarrierOutcome,
    NoTradeCounterfactual,
    RiskDeniedCounterfactual,
    BaselineAction,
    ExternalPredictionAction,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GeneratedReferenceStatus {
    Generated,
    SkippedNoCandleMatch,
    SkippedNoOutcomeWindow,
    SkippedNoBaselineData,
    SkippedInvalidPrediction,
    SkippedSourceNotAllowed,
    DiagnosticOnlyEstimated,
    RejectedNoLookahead,
    RejectedBadDataQuality,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GeneratedReferenceSource {
    LocalCandleSeries,
    ExistingArtifact,
    DeterministicBaselinePolicy,
    EstimatedDiagnosticOnly,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeneratedCommitteeReference {
    pub reference_id: String,
    pub scenario_row_id: String,
    pub reference_kind: GeneratedReferenceKind,
    pub status: GeneratedReferenceStatus,
    #[serde(default)]
    pub outcome_reference: Option<CommitteeOutcomeReference>,
    #[serde(default)]
    pub baseline_reference: Option<CommitteeBaselineReference>,
    #[serde(default)]
    pub external_reference: Option<CommitteeExternalReference>,
    #[serde(default)]
    pub no_trade_counterfactual: Option<CommitteeCounterfactualRecord>,
    #[serde(default)]
    pub risk_denied_counterfactual: Option<CommitteeCounterfactualRecord>,
    pub generated_from: GeneratedReferenceSource,
    pub official_readiness_eligible: bool,
    pub diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeneratedCommitteeReferencePack {
    pub reference_pack_id: String,
    #[serde(default)]
    pub scenario_rows: Vec<CommitteeScenarioRow>,
    pub generated_references: Vec<GeneratedCommitteeReference>,
    pub alignment_report: super::candle_alignment::CandleAlignmentReport,
    pub scenario_count: usize,
    pub generated_outcome_count: usize,
    pub generated_baseline_count: usize,
    pub generated_no_trade_count: usize,
    pub generated_risk_denied_count: usize,
    pub generated_external_count: usize,
    pub diagnostic_only_count: usize,
    pub rejected_count: usize,
    pub source_summary: String,
    pub storage_bytes: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for CommitteeReferencePackConfig {
    fn default() -> Self {
        Self {
            reference_pack_id: "committee_reference_pack".to_string(),
            scenario_pack_paths: Vec::new(),
            scenario_set_paths: Vec::new(),
            candle_series_paths: Vec::new(),
            baseline_reference_paths: Vec::new(),
            external_prediction_paths: Vec::new(),
            output_root: "target/soma_committee_reference_pack".to_string(),
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_bytes: default_max_bytes(),
            default_horizon_bars: default_horizon_bars(),
            default_take_profit_pct: default_take_profit_pct(),
            default_stop_loss_pct: default_stop_loss_pct(),
            default_cost_bps: default_cost_bps(),
            default_slippage_bps: default_slippage_bps(),
            timestamp_tolerance_ms: 0,
            require_exact_symbol_match: true,
            require_exact_horizon_match: true,
            require_no_lookahead_safe: true,
            build_triple_barrier_outcomes: true,
            build_no_trade_counterfactuals: true,
            build_risk_denied_counterfactuals: true,
            build_baseline_references: true,
            allow_estimated_references: false,
            allow_controlled_fixture_references: false,
            allow_yfinance_research: false,
            allow_fixture: false,
            reason_codes: vec![ReasonCode::CommitteeReferencePackBuilt],
        }
    }
}

impl CommitteeReferencePackConfig {
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
            .chain(self.scenario_set_paths.iter())
            .chain(self.candle_series_paths.iter())
            .chain(self.baseline_reference_paths.iter())
            .chain(self.external_prediction_paths.iter())
            .chain(std::iter::once(&self.output_root));
        if paths.clone().any(|path| path.contains("://")) {
            return Err("committee reference pack paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err("committee reference pack max_rows must be between 1 and 100".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err("committee reference pack max_symbols must be between 1 and 5".to_string());
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err(
                "committee reference pack max_bytes must be between 1 and 5000000".to_string(),
            );
        }
        if self.default_horizon_bars == 0 {
            return Err(
                "committee reference pack default_horizon_bars must be positive".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.reference_pack_id)
    }
}

impl GeneratedCommitteeReference {
    pub fn built(&self) -> bool {
        matches!(
            self.status,
            GeneratedReferenceStatus::Generated | GeneratedReferenceStatus::DiagnosticOnlyEstimated
        )
    }

    pub fn rejected(&self) -> bool {
        matches!(
            self.status,
            GeneratedReferenceStatus::RejectedNoLookahead
                | GeneratedReferenceStatus::RejectedBadDataQuality
        )
    }

    pub fn to_text_line(&self) -> String {
        format!(
            "reference_id={};scenario_row_id={};kind={:?};status={:?};generated_from={:?};official_readiness_eligible={};diagnostic_only={}",
            self.reference_id,
            self.scenario_row_id,
            self.reference_kind,
            self.status,
            self.generated_from,
            self.official_readiness_eligible,
            self.diagnostic_only,
        )
    }
}

impl GeneratedCommitteeReferencePack {
    pub fn new(
        reference_pack_id: impl Into<String>,
        scenario_rows: Vec<CommitteeScenarioRow>,
        mut generated_references: Vec<GeneratedCommitteeReference>,
        alignment_report: super::candle_alignment::CandleAlignmentReport,
        reason_codes: Vec<ReasonCode>,
    ) -> Self {
        generated_references.sort_by(|left, right| {
            left.scenario_row_id
                .cmp(&right.scenario_row_id)
                .then(left.reference_kind.cmp(&right.reference_kind))
                .then(left.reference_id.cmp(&right.reference_id))
        });
        let generated_outcome_count = generated_references
            .iter()
            .filter(|reference| {
                reference.reference_kind == GeneratedReferenceKind::TripleBarrierOutcome
                    && reference.built()
            })
            .count();
        let generated_baseline_count = generated_references
            .iter()
            .filter(|reference| {
                reference.reference_kind == GeneratedReferenceKind::BaselineAction
                    && reference.built()
            })
            .count();
        let generated_no_trade_count = generated_references
            .iter()
            .filter(|reference| {
                reference.reference_kind == GeneratedReferenceKind::NoTradeCounterfactual
                    && reference.built()
            })
            .count();
        let generated_risk_denied_count = generated_references
            .iter()
            .filter(|reference| {
                reference.reference_kind == GeneratedReferenceKind::RiskDeniedCounterfactual
                    && reference.built()
            })
            .count();
        let generated_external_count = generated_references
            .iter()
            .filter(|reference| {
                reference.reference_kind == GeneratedReferenceKind::ExternalPredictionAction
                    && reference.built()
            })
            .count();
        let diagnostic_only_count = generated_references
            .iter()
            .filter(|reference| reference.diagnostic_only)
            .count();
        let rejected_count = generated_references
            .iter()
            .filter(|reference| reference.rejected())
            .count();
        let storage_bytes = serde_json::to_vec(&generated_references)
            .map(|bytes| bytes.len())
            .unwrap_or_default()
            + alignment_report.to_text().len();
        let source_summary = generated_references
            .iter()
            .fold(BTreeMap::<String, usize>::new(), |mut acc, reference| {
                *acc.entry(format!("{:?}", reference.generated_from))
                    .or_insert(0) += 1;
                acc
            })
            .into_iter()
            .map(|(source, count)| format!("{source}={count}"))
            .collect::<Vec<_>>()
            .join("|");
        Self {
            reference_pack_id: reference_pack_id.into(),
            scenario_count: scenario_rows.len(),
            scenario_rows,
            generated_references,
            alignment_report,
            generated_outcome_count,
            generated_baseline_count,
            generated_no_trade_count,
            generated_risk_denied_count,
            generated_external_count,
            diagnostic_only_count,
            rejected_count,
            source_summary,
            storage_bytes,
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("reference_pack_id={}", self.reference_pack_id),
            format!("scenario_count={}", self.scenario_count),
            format!("generated_outcome_count={}", self.generated_outcome_count),
            format!("generated_baseline_count={}", self.generated_baseline_count),
            format!("generated_no_trade_count={}", self.generated_no_trade_count),
            format!(
                "generated_risk_denied_count={}",
                self.generated_risk_denied_count
            ),
            format!("generated_external_count={}", self.generated_external_count),
            format!("diagnostic_only_count={}", self.diagnostic_only_count),
            format!("rejected_count={}", self.rejected_count),
            format!("source_summary={}", self.source_summary),
            format!("storage_bytes={}", self.storage_bytes),
        ];
        lines.push(self.alignment_report.to_text());
        lines.extend(
            self.generated_references
                .iter()
                .map(GeneratedCommitteeReference::to_text_line),
        );
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        self.alignment_report.write_to_dir(output_dir)?;
        fs::write(
            output_dir.join("generated_reference_pack.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("generated_reference_pack.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }

    pub fn official_ready_reference_count(&self) -> usize {
        self.generated_references
            .iter()
            .filter(|reference| reference.official_readiness_eligible && reference.built())
            .count()
    }

    pub fn research_only_reference_count(&self) -> usize {
        self.generated_references
            .iter()
            .filter(|reference| {
                self.row_for_reference(reference).is_some_and(|row| {
                    matches!(
                        row.evidence_source_kind,
                        crate::data::EvidenceSourceKind::YFinanceResearch
                    )
                }) && reference.built()
            })
            .count()
    }

    pub fn fixture_reference_count(&self) -> usize {
        self.generated_references
            .iter()
            .filter(|reference| {
                self.row_for_reference(reference).is_some_and(|row| {
                    matches!(
                        row.source_kind,
                        super::committee_scenario_loader::CommitteeScenarioSourceKind::Fixture
                            | super::committee_scenario_loader::CommitteeScenarioSourceKind::SyntheticTest
                    )
                }) && reference.built()
            })
            .count()
    }

    pub fn no_lookahead_safe_count(&self) -> usize {
        self.generated_references
            .iter()
            .filter(|reference| reference.built())
            .filter(|reference| {
                reference
                    .outcome_reference
                    .as_ref()
                    .map(|item| item.no_lookahead_safe)
                    .or_else(|| {
                        reference
                            .no_trade_counterfactual
                            .as_ref()
                            .map(|item| item.no_lookahead_safe)
                    })
                    .or_else(|| {
                        reference
                            .risk_denied_counterfactual
                            .as_ref()
                            .map(|item| item.no_lookahead_safe)
                    })
                    .unwrap_or(true)
            })
            .count()
    }

    pub fn row_for_reference(
        &self,
        reference: &GeneratedCommitteeReference,
    ) -> Option<&CommitteeScenarioRow> {
        self.scenario_rows
            .iter()
            .find(|row| row.scenario_row_id == reference.scenario_row_id)
    }

    pub fn to_committee_scenario_set(&self) -> CommitteeScenarioSet {
        CommitteeScenarioSet {
            scenario_id: self.reference_pack_id.clone(),
            rows: self.scenario_rows.clone(),
            source_summary: self
                .scenario_rows
                .iter()
                .fold(BTreeMap::<String, usize>::new(), |mut acc, row| {
                    *acc.entry(format!("{:?}", row.evidence_source_kind)).or_insert(0) += 1;
                    acc
                })
                .into_iter()
                .map(|(kind, count)| format!("{kind}={count}"))
                .collect::<Vec<_>>()
                .join("|"),
            row_count: self.scenario_rows.len(),
            official_row_count: self
                .scenario_rows
                .iter()
                .filter(|row| row.evidence_source_kind.readiness_eligible())
                .count(),
            research_only_row_count: self
                .scenario_rows
                .iter()
                .filter(|row| row.evidence_source_kind == crate::data::EvidenceSourceKind::YFinanceResearch)
                .count(),
            fixture_row_count: self
                .scenario_rows
                .iter()
                .filter(|row| {
                    matches!(
                        row.source_kind,
                        super::committee_scenario_loader::CommitteeScenarioSourceKind::Fixture
                            | super::committee_scenario_loader::CommitteeScenarioSourceKind::SyntheticTest
                    )
                })
                .count(),
            skipped_row_count: 0,
            reason_codes: self.reason_codes.clone(),
        }
    }

    pub fn to_official_pack(&self) -> OfficialCommitteeScenarioPack {
        let mut source_counts = BTreeMap::<String, usize>::new();
        for row in &self.scenario_rows {
            *source_counts
                .entry(format!("{:?}", classify_row(row)))
                .or_insert(0) += 1;
        }
        OfficialCommitteeScenarioPack {
            pack_id: self.reference_pack_id.clone(),
            rows: self.scenario_rows.clone(),
            source_summary: source_counts
                .into_iter()
                .map(|(kind, count)| format!("{kind}={count}"))
                .collect::<Vec<_>>()
                .join("|"),
            official_row_count: self
                .scenario_rows
                .iter()
                .filter(|row| row.evidence_source_kind.readiness_eligible())
                .count(),
            crypto_only_row_count: self
                .scenario_rows
                .iter()
                .filter(|row| row.market == crate::data::ProviderMarket::Crypto)
                .count(),
            yfinance_row_count: self
                .scenario_rows
                .iter()
                .filter(|row| row.evidence_source_kind == crate::data::EvidenceSourceKind::YFinanceResearch)
                .count(),
            fixture_row_count: self
                .scenario_rows
                .iter()
                .filter(|row| {
                    matches!(
                        row.source_kind,
                        super::committee_scenario_loader::CommitteeScenarioSourceKind::Fixture
                            | super::committee_scenario_loader::CommitteeScenarioSourceKind::SyntheticTest
                    )
                })
                .count(),
            row_level_count: self
                .scenario_rows
                .iter()
                .filter(|row| {
                    row.materialization_level
                        == super::committee_scenario_loader::CommitteeScenarioMaterializationLevel::RowLevel
                })
                .count(),
            summary_derived_count: self
                .scenario_rows
                .iter()
                .filter(|row| {
                    row.materialization_level
                        != super::committee_scenario_loader::CommitteeScenarioMaterializationLevel::RowLevel
                })
                .count(),
            outcome_linked_count: self.generated_outcome_count,
            baseline_reference_count: self.generated_baseline_count,
            external_reference_count: self.generated_external_count,
            no_trade_counterfactual_count: self.generated_no_trade_count,
            risk_denial_counterfactual_count: self.generated_risk_denied_count,
            storage_bytes: self.storage_bytes,
            reason_codes: self.reason_codes.clone(),
        }
    }

    pub fn to_outcome_linked_pack(&self) -> OutcomeLinkedCommitteeScenarioPack {
        let by_row = self.generated_references.iter().fold(
            BTreeMap::<String, Vec<&GeneratedCommitteeReference>>::new(),
            |mut acc, reference| {
                acc.entry(reference.scenario_row_id.clone())
                    .or_default()
                    .push(reference);
                acc
            },
        );
        let mut linked_rows = Vec::new();
        let mut unmatched_rows = Vec::new();
        let mut outcome_linked_count = 0usize;
        let mut baseline_linked_count = 0usize;
        let mut external_linked_count = 0usize;
        let mut no_trade_counterfactual_count = 0usize;
        let mut risk_denial_counterfactual_count = 0usize;
        let mut no_lookahead_violations = 0usize;
        for row in &self.scenario_rows {
            let references = by_row
                .get(&row.scenario_row_id)
                .cloned()
                .unwrap_or_default();
            let outcome_reference = references
                .iter()
                .filter(|reference| {
                    reference.reference_kind == GeneratedReferenceKind::TripleBarrierOutcome
                })
                .find_map(|reference| reference.outcome_reference.clone());
            let baseline_reference = references
                .iter()
                .filter(|reference| {
                    reference.reference_kind == GeneratedReferenceKind::BaselineAction
                })
                .find_map(|reference| reference.baseline_reference.clone());
            let external_reference = references
                .iter()
                .filter(|reference| {
                    reference.reference_kind == GeneratedReferenceKind::ExternalPredictionAction
                })
                .find_map(|reference| reference.external_reference.clone());
            if outcome_reference.is_some()
                || baseline_reference.is_some()
                || external_reference.is_some()
            {
                if outcome_reference.is_some() {
                    outcome_linked_count += 1;
                }
                if baseline_reference.is_some() {
                    baseline_linked_count += 1;
                }
                if external_reference.is_some() {
                    external_linked_count += 1;
                }
                let built_counterfactuals = references
                    .iter()
                    .filter(|reference| reference.built())
                    .collect::<Vec<_>>();
                if built_counterfactuals.iter().any(|reference| {
                    reference.reference_kind == GeneratedReferenceKind::NoTradeCounterfactual
                }) {
                    no_trade_counterfactual_count += 1;
                }
                if built_counterfactuals.iter().any(|reference| {
                    reference.reference_kind == GeneratedReferenceKind::RiskDeniedCounterfactual
                }) {
                    risk_denial_counterfactual_count += 1;
                }
                if built_counterfactuals.iter().any(|reference| {
                    reference
                        .outcome_reference
                        .as_ref()
                        .map(|item| !item.no_lookahead_safe)
                        .or_else(|| {
                            reference
                                .no_trade_counterfactual
                                .as_ref()
                                .map(|item| !item.no_lookahead_safe)
                        })
                        .or_else(|| {
                            reference
                                .risk_denied_counterfactual
                                .as_ref()
                                .map(|item| !item.no_lookahead_safe)
                        })
                        .unwrap_or(false)
                }) {
                    no_lookahead_violations += 1;
                }
                linked_rows.push(OutcomeLinkedCommitteeScenarioRow {
                    scenario_row: row.clone(),
                    outcome_reference,
                    baseline_reference,
                    external_reference,
                    reason_codes: stable_reason_codes(
                        &references
                            .iter()
                            .flat_map(|reference| reference.reason_codes.clone())
                            .collect::<Vec<_>>(),
                    ),
                });
            } else {
                unmatched_rows.push(row.clone());
            }
        }
        OutcomeLinkedCommitteeScenarioPack {
            pack: self.to_official_pack(),
            linked_rows,
            unmatched_rows,
            link_summary: CommitteeOutcomeLinkSummary {
                linker_id: format!("{}-generated", self.reference_pack_id),
                matched_rows: outcome_linked_count
                    .max(baseline_linked_count)
                    .max(external_linked_count),
                unmatched_rows: self.scenario_rows.len().saturating_sub(
                    outcome_linked_count
                        .max(baseline_linked_count)
                        .max(external_linked_count),
                ),
                timestamp_tolerance_ms: 0,
                strict_timestamp_match: true,
                no_lookahead_violations,
                warnings: Vec::new(),
                reason_codes: vec![ReasonCode::CommitteeReferencePackBuilt],
            },
            outcome_linked_count,
            baseline_linked_count,
            external_linked_count,
            no_trade_counterfactual_count,
            risk_denial_counterfactual_count,
            no_lookahead_violations,
            reason_codes: self.reason_codes.clone(),
        }
    }

    pub fn counterfactual_records(&self) -> Vec<CommitteeCounterfactualRecord> {
        let mut records = self
            .generated_references
            .iter()
            .flat_map(|reference| {
                let mut items = Vec::new();
                if let Some(record) = &reference.no_trade_counterfactual {
                    items.push(record.clone());
                }
                if let Some(record) = &reference.risk_denied_counterfactual {
                    items.push(record.clone());
                }
                items
            })
            .collect::<Vec<_>>();
        records.sort_by(|left, right| {
            left.scenario_row_id
                .cmp(&right.scenario_row_id)
                .then(left.counterfactual_type.cmp(&right.counterfactual_type))
                .then(left.counterfactual_id.cmp(&right.counterfactual_id))
        });
        records
    }

    pub fn source_kind_summary(&self) -> String {
        self.generated_references
            .iter()
            .filter_map(|reference| {
                self.row_for_reference(reference)
                    .map(|row| row.evidence_source_kind)
            })
            .fold(BTreeMap::<String, usize>::new(), |mut acc, kind| {
                *acc.entry(format!("{:?}", kind)).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .map(|(kind, count)| format!("{kind}={count}"))
            .collect::<Vec<_>>()
            .join("|")
    }

    pub fn generated_from_summary(&self) -> String {
        self.generated_references
            .iter()
            .fold(BTreeMap::<String, usize>::new(), |mut acc, reference| {
                *acc.entry(format!("{:?}", reference.generated_from))
                    .or_insert(0) += 1;
                acc
            })
            .into_iter()
            .map(|(kind, count)| format!("{kind}={count}"))
            .collect::<Vec<_>>()
            .join("|")
    }

    pub fn estimated_reference_count(&self) -> usize {
        self.generated_references
            .iter()
            .filter(|reference| {
                reference.generated_from == GeneratedReferenceSource::EstimatedDiagnosticOnly
            })
            .count()
    }
}

impl From<TripleBarrierReferenceSource> for GeneratedReferenceSource {
    fn from(value: TripleBarrierReferenceSource) -> Self {
        match value {
            TripleBarrierReferenceSource::LocalCandleSeries => Self::LocalCandleSeries,
            TripleBarrierReferenceSource::EstimatedDiagnosticOnly => Self::EstimatedDiagnosticOnly,
        }
    }
}

impl From<BaselineReferenceSource> for GeneratedReferenceSource {
    fn from(value: BaselineReferenceSource) -> Self {
        match value {
            BaselineReferenceSource::ExistingArtifact => Self::ExistingArtifact,
            BaselineReferenceSource::DeterministicNoTrade
            | BaselineReferenceSource::DeterministicBaselineSignalApprox => {
                Self::DeterministicBaselinePolicy
            }
            BaselineReferenceSource::Unknown => Self::Unknown,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_max_rows() -> usize {
    100
}

fn default_max_symbols() -> usize {
    5
}

fn default_max_bytes() -> usize {
    5_000_000
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
