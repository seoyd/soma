use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::committee_evidence_sufficiency::CommitteeEvidenceSufficiencyStatus;
use super::official_evidence_replication::OfficialEvidenceReplicationConfig;
use super::official_reference_replication::OfficialReferenceReplicationArtifacts;
use super::official_row_injection::{
    OfficialEvidenceBoundary, OfficialRowInjectionResult, classify_row_boundary,
};
use super::sufficiency_closure::SufficiencyClosureReport;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialSufficiencyReplicationStatus {
    OfficialSufficiencyPassed,
    ControlledSufficiencyOnly,
    CryptoOnlySufficiency,
    MissingOfficialRows,
    MissingNonCryptoOfficialRows,
    MissingOfficialReferences,
    MissingOutcomeLinks,
    MissingCounterfactuals,
    MissingBaselineReferences,
    TooMuchSummaryDerived,
    TooMuchDiagnosticOnly,
    NeedMoreEvidence,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialSufficiencyReplicationReport {
    #[serde(default)]
    pub previous_controlled_status: Option<CommitteeEvidenceSufficiencyStatus>,
    pub current_official_status: OfficialSufficiencyReplicationStatus,
    pub official_row_count: usize,
    pub non_crypto_official_row_count: usize,
    pub official_reference_count: usize,
    pub outcome_link_count: usize,
    pub baseline_reference_count: usize,
    pub no_trade_counterfactual_count: usize,
    pub risk_denied_counterfactual_count: usize,
    pub summary_derived_ratio: f64,
    pub diagnostic_only_ratio: f64,
    pub controlled_only_ratio: f64,
    pub crypto_only_ratio: f64,
    pub passed_for_controlled: bool,
    pub passed_for_official: bool,
    pub remaining_gaps: Vec<String>,
    pub final_status: OfficialSufficiencyReplicationStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialSufficiencyReplicationBuilder;

impl OfficialSufficiencyReplicationBuilder {
    pub fn build(
        &self,
        config: &OfficialEvidenceReplicationConfig,
        row_injection: &OfficialRowInjectionResult,
        reference_artifacts: Option<&OfficialReferenceReplicationArtifacts>,
    ) -> OfficialSufficiencyReplicationReport {
        let previous_controlled_status =
            load_previous_controlled_status(&config.previous_sufficiency_closure_paths);
        let official_row_count = row_injection.official_row_count;
        let non_crypto_official_row_count = row_injection.non_crypto_official_row_count;
        let controlled_row_count = row_injection
            .injected_rows
            .iter()
            .filter(|row| classify_row_boundary(row) == OfficialEvidenceBoundary::Controlled)
            .count();
        let crypto_row_count = row_injection.crypto_only_row_count;
        let summary_derived_count = row_injection
            .injected_rows
            .iter()
            .filter(|row| row.materialization_level != super::committee_scenario_loader::CommitteeScenarioMaterializationLevel::RowLevel)
            .count();
        let total_rows = row_injection.injected_rows.len().max(1);
        let summary_derived_ratio = summary_derived_count as f64 / total_rows as f64;

        let reference_report = reference_artifacts.map(|artifacts| &artifacts.report);
        let closure_report =
            reference_artifacts.and_then(|artifacts| artifacts.closure_report.as_ref());
        let official_reference_count = reference_report
            .map(|report| report.official_ready_reference_count)
            .unwrap_or(0);
        let outcome_link_count = reference_report
            .map(|report| report.outcome_reference_count)
            .unwrap_or(0);
        let baseline_reference_count = reference_report
            .map(|report| report.baseline_reference_count)
            .unwrap_or(0);
        let no_trade_counterfactual_count = reference_report
            .map(|report| report.no_trade_counterfactual_count)
            .unwrap_or(0);
        let risk_denied_counterfactual_count = reference_report
            .map(|report| report.risk_denied_counterfactual_count)
            .unwrap_or(0);
        let diagnostic_only_ratio = reference_report
            .map(|report| {
                let total = report
                    .generated_reference_pack
                    .as_ref()
                    .map(|pack| pack.generated_references.len())
                    .unwrap_or(0)
                    .max(1);
                report.diagnostic_only_reference_count as f64 / total as f64
            })
            .unwrap_or(0.0);
        let controlled_only_ratio = controlled_row_count as f64 / total_rows as f64;
        let crypto_only_ratio = crypto_row_count as f64 / total_rows as f64;
        let passed_for_controlled = closure_report
            .map(|report| {
                matches!(
                    report.current_status,
                    CommitteeEvidenceSufficiencyStatus::SufficientForCommitteeBenchmark
                        | CommitteeEvidenceSufficiencyStatus::SufficientForCryptoOnlyBenchmark
                        | CommitteeEvidenceSufficiencyStatus::SufficientForDiagnosticsOnly
                )
            })
            .unwrap_or_else(|| {
                outcome_link_count > 0
                    && baseline_reference_count > 0
                    && no_trade_counterfactual_count > 0
                    && risk_denied_counterfactual_count > 0
                    && !row_injection.injected_rows.is_empty()
            });
        let mut remaining_gaps = Vec::new();
        let current_official_status = if official_row_count == 0 {
            remaining_gaps.push("missing_official_rows".to_string());
            OfficialSufficiencyReplicationStatus::MissingOfficialRows
        } else if config.require_non_crypto_official && non_crypto_official_row_count == 0 {
            remaining_gaps.push("missing_non_crypto_official_rows".to_string());
            if crypto_only_ratio > 0.0 && passed_for_controlled {
                OfficialSufficiencyReplicationStatus::CryptoOnlySufficiency
            } else {
                OfficialSufficiencyReplicationStatus::MissingNonCryptoOfficialRows
            }
        } else if outcome_link_count == 0 {
            remaining_gaps.push("missing_outcome_links".to_string());
            OfficialSufficiencyReplicationStatus::MissingOutcomeLinks
        } else if baseline_reference_count == 0 {
            remaining_gaps.push("missing_baseline_references".to_string());
            OfficialSufficiencyReplicationStatus::MissingBaselineReferences
        } else if no_trade_counterfactual_count == 0 || risk_denied_counterfactual_count == 0 {
            remaining_gaps.push("missing_counterfactuals".to_string());
            OfficialSufficiencyReplicationStatus::MissingCounterfactuals
        } else if official_reference_count == 0 && config.require_non_crypto_official {
            remaining_gaps.push("missing_official_references".to_string());
            if passed_for_controlled {
                OfficialSufficiencyReplicationStatus::ControlledSufficiencyOnly
            } else {
                OfficialSufficiencyReplicationStatus::MissingOfficialReferences
            }
        } else if summary_derived_ratio > 0.40 {
            remaining_gaps.push("too_much_summary_derived".to_string());
            OfficialSufficiencyReplicationStatus::TooMuchSummaryDerived
        } else if diagnostic_only_ratio > 0.50 {
            remaining_gaps.push("too_much_diagnostic_only".to_string());
            OfficialSufficiencyReplicationStatus::TooMuchDiagnosticOnly
        } else if !config.require_non_crypto_official && passed_for_controlled {
            OfficialSufficiencyReplicationStatus::OfficialSufficiencyPassed
        } else if passed_for_controlled && official_reference_count > 0 {
            OfficialSufficiencyReplicationStatus::OfficialSufficiencyPassed
        } else if passed_for_controlled && crypto_only_ratio > 0.0 {
            OfficialSufficiencyReplicationStatus::CryptoOnlySufficiency
        } else if passed_for_controlled {
            OfficialSufficiencyReplicationStatus::ControlledSufficiencyOnly
        } else {
            remaining_gaps.push("need_more_evidence".to_string());
            OfficialSufficiencyReplicationStatus::NeedMoreEvidence
        };
        let passed_for_official = current_official_status
            == OfficialSufficiencyReplicationStatus::OfficialSufficiencyPassed;
        OfficialSufficiencyReplicationReport {
            previous_controlled_status,
            current_official_status,
            official_row_count,
            non_crypto_official_row_count,
            official_reference_count,
            outcome_link_count,
            baseline_reference_count,
            no_trade_counterfactual_count,
            risk_denied_counterfactual_count,
            summary_derived_ratio,
            diagnostic_only_ratio,
            controlled_only_ratio,
            crypto_only_ratio,
            passed_for_controlled,
            passed_for_official,
            remaining_gaps,
            final_status: current_official_status,
            reason_codes: stable_reason_codes(&[
                ReasonCode::OfficialSufficiencyReplicationBuilt,
                ReasonCode::SufficiencyClosureBuilt,
            ]),
        }
    }
}

impl OfficialSufficiencyReplicationReport {
    pub fn to_text(&self) -> String {
        [
            format!(
                "previous_controlled_status={}",
                self.previous_controlled_status
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default()
            ),
            format!("current_official_status={:?}", self.current_official_status),
            format!("official_row_count={}", self.official_row_count),
            format!(
                "non_crypto_official_row_count={}",
                self.non_crypto_official_row_count
            ),
            format!("official_reference_count={}", self.official_reference_count),
            format!("outcome_link_count={}", self.outcome_link_count),
            format!("baseline_reference_count={}", self.baseline_reference_count),
            format!(
                "no_trade_counterfactual_count={}",
                self.no_trade_counterfactual_count
            ),
            format!(
                "risk_denied_counterfactual_count={}",
                self.risk_denied_counterfactual_count
            ),
            format!("summary_derived_ratio={:.6}", self.summary_derived_ratio),
            format!("diagnostic_only_ratio={:.6}", self.diagnostic_only_ratio),
            format!("controlled_only_ratio={:.6}", self.controlled_only_ratio),
            format!("crypto_only_ratio={:.6}", self.crypto_only_ratio),
            format!("passed_for_controlled={}", self.passed_for_controlled),
            format!("passed_for_official={}", self.passed_for_official),
            format!("remaining_gaps={}", self.remaining_gaps.join("|")),
            format!("final_status={:?}", self.final_status),
        ]
        .join("\n")
    }
}

fn load_previous_controlled_status(paths: &[String]) -> Option<CommitteeEvidenceSufficiencyStatus> {
    for path in paths {
        if path.ends_with(".json") {
            if let Ok(report) = SufficiencyClosureReport::from_json_path(Path::new(path)) {
                return Some(report.current_status);
            }
        }
        if let Ok(text) = fs::read_to_string(path) {
            if let Some(line) = text.lines().find(|line| {
                line.starts_with("current_status=") || line.starts_with("sufficiency_status=")
            }) {
                let value = line.split('=').nth(1).unwrap_or_default().trim();
                return parse_previous_status(value);
            }
        }
    }
    None
}

fn parse_previous_status(value: &str) -> Option<CommitteeEvidenceSufficiencyStatus> {
    match value {
        "SufficientForCommitteeBenchmark" => {
            Some(CommitteeEvidenceSufficiencyStatus::SufficientForCommitteeBenchmark)
        }
        "SufficientForCryptoOnlyBenchmark" => {
            Some(CommitteeEvidenceSufficiencyStatus::SufficientForCryptoOnlyBenchmark)
        }
        "SufficientForDiagnosticsOnly" => {
            Some(CommitteeEvidenceSufficiencyStatus::SufficientForDiagnosticsOnly)
        }
        "InsufficientOfficialRows" => {
            Some(CommitteeEvidenceSufficiencyStatus::InsufficientOfficialRows)
        }
        "InsufficientOutcomeLinks" => {
            Some(CommitteeEvidenceSufficiencyStatus::InsufficientOutcomeLinks)
        }
        "InsufficientCounterfactuals" => {
            Some(CommitteeEvidenceSufficiencyStatus::InsufficientCounterfactuals)
        }
        "InsufficientBaselineReferences" => {
            Some(CommitteeEvidenceSufficiencyStatus::InsufficientBaselineReferences)
        }
        "TooMuchSummaryDerived" => Some(CommitteeEvidenceSufficiencyStatus::TooMuchSummaryDerived),
        "TooMuchResearchOnly" => Some(CommitteeEvidenceSufficiencyStatus::TooMuchResearchOnly),
        "TooMuchFixture" => Some(CommitteeEvidenceSufficiencyStatus::TooMuchFixture),
        "NoLookaheadViolation" => Some(CommitteeEvidenceSufficiencyStatus::NoLookaheadViolation),
        "NeedMoreEvidence" => Some(CommitteeEvidenceSufficiencyStatus::NeedMoreEvidence),
        _ => None,
    }
}
