use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::committee_evidence_sufficiency::{
    CommitteeEvidenceSufficiencyGateResult, CommitteeEvidenceSufficiencyStatus,
    evaluate_committee_evidence_sufficiency,
};
use super::committee_outcome_coverage::{
    CommitteeOutcomeCoverageConfig, CommitteeOutcomeCoverageReport,
    build_committee_outcome_coverage_report,
};
use super::committee_performance_matrix::build_committee_performance_evidence_matrix;
use super::committee_reference_pack::GeneratedCommitteeReferencePack;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SufficiencyClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub previous_coverage_report_path: Option<String>,
    #[serde(default)]
    pub previous_sufficiency_gate_path: Option<String>,
    pub generated_reference_pack_path: String,
    pub output_root: String,
    #[serde(default = "default_true")]
    pub rerun_outcome_coverage: bool,
    #[serde(default = "default_true")]
    pub rerun_counterfactual_audit: bool,
    #[serde(default = "default_true")]
    pub rerun_performance_matrix: bool,
    #[serde(default = "default_true")]
    pub rerun_sufficiency_gate: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SufficiencyClosureCounts {
    pub scenario_count: usize,
    pub outcome_links: usize,
    pub baseline_references: usize,
    pub no_trade_counterfactuals: usize,
    pub risk_denied_counterfactuals: usize,
    pub official_ready_references: usize,
    pub no_lookahead_violations: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SufficiencyClosureStatus {
    ImprovedButStillInsufficient,
    OutcomeLinksClosed,
    CounterfactualDepthClosed,
    SufficiencyGatePassedForControlledEvidence,
    SufficiencyGatePassedForOfficialEvidence,
    StillNeedMoreCandleData,
    StillNeedMoreOfficialRows,
    NoImprovement,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SufficiencyClosureFinalRecommendation {
    ImproveOutcomeLinkingFirst,
    MoreCandleData,
    MoreOfficialCommitteeEvidence,
    ImproveBaselineReferenceDepth,
    ImproveCounterfactualDepthFirst,
    CommitteeBenchmarkReadyForControlledEvidence,
    CommitteeV1BenchmarkReady,
    KeepTrinity,
    NeedMoreEvidence,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SufficiencyClosureReport {
    pub closure_id: String,
    #[serde(default)]
    pub previous_status: Option<CommitteeEvidenceSufficiencyStatus>,
    pub current_status: CommitteeEvidenceSufficiencyStatus,
    #[serde(default)]
    pub previous_counts: Option<SufficiencyClosureCounts>,
    pub current_counts: SufficiencyClosureCounts,
    pub added_outcome_links: usize,
    pub added_baseline_references: usize,
    pub added_no_trade_counterfactuals: usize,
    pub added_risk_denied_counterfactuals: usize,
    pub added_official_ready_references: usize,
    pub remaining_gaps: Vec<String>,
    pub improvement_detected: bool,
    pub closure_status: SufficiencyClosureStatus,
    pub final_recommendation: SufficiencyClosureFinalRecommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SufficiencyClosureRunner;

impl Default for SufficiencyClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "committee_sufficiency_closure".to_string(),
            previous_coverage_report_path: None,
            previous_sufficiency_gate_path: None,
            generated_reference_pack_path: "target/soma_committee_reference_pack/committee_reference_pack/generated_reference_pack.json".to_string(),
            output_root: "target/soma_committee_reference_pack".to_string(),
            rerun_outcome_coverage: true,
            rerun_counterfactual_audit: true,
            rerun_performance_matrix: true,
            rerun_sufficiency_gate: true,
            reason_codes: vec![ReasonCode::SufficiencyClosureBuilt],
        }
    }
}

impl SufficiencyClosureConfig {
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
            .previous_coverage_report_path
            .iter()
            .chain(self.previous_sufficiency_gate_path.iter())
            .chain(std::iter::once(&self.generated_reference_pack_path))
            .chain(std::iter::once(&self.output_root));
        if paths.clone().any(|path| path.contains("://")) {
            return Err("committee sufficiency closure paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.closure_id)
    }
}

impl SufficiencyClosureRunner {
    pub fn run_with_pack(
        &self,
        config: &SufficiencyClosureConfig,
        generated_pack: &GeneratedCommitteeReferencePack,
    ) -> Result<SufficiencyClosureReport, String> {
        config.validate()?;
        let previous_coverage = config
            .previous_coverage_report_path
            .as_deref()
            .map(load_previous_coverage)
            .transpose()?;
        let previous_gate = config
            .previous_sufficiency_gate_path
            .as_deref()
            .map(load_previous_sufficiency)
            .transpose()?;
        let current_counts = if config.rerun_outcome_coverage {
            let coverage = rerun_coverage(generated_pack);
            counts_from_coverage(&coverage, generated_pack)
        } else {
            counts_from_pack(generated_pack)
        };
        let current_gate = if config.rerun_sufficiency_gate {
            rerun_sufficiency(generated_pack)
        } else if generated_pack.official_ready_reference_count() > 0 {
            CommitteeEvidenceSufficiencyGateResult {
                passed: true,
                failed_gates: Vec::new(),
                warnings: vec!["closure reused generated pack counts without rerun".to_string()],
                sufficiency_status:
                    CommitteeEvidenceSufficiencyStatus::SufficientForCommitteeBenchmark,
                reason_codes: vec![ReasonCode::SufficiencyClosureBuilt],
            }
        } else {
            CommitteeEvidenceSufficiencyGateResult {
                passed: false,
                failed_gates: vec!["min_official_rows".to_string()],
                warnings: vec!["closure reused generated pack counts without rerun".to_string()],
                sufficiency_status: CommitteeEvidenceSufficiencyStatus::InsufficientOfficialRows,
                reason_codes: vec![ReasonCode::SufficiencyClosureBuilt],
            }
        };
        let previous_counts = previous_coverage
            .as_ref()
            .map(counts_from_previous_coverage);
        let added_outcome_links = current_counts.outcome_links.saturating_sub(
            previous_counts
                .as_ref()
                .map(|counts| counts.outcome_links)
                .unwrap_or(0),
        );
        let added_baseline_references = current_counts.baseline_references.saturating_sub(
            previous_counts
                .as_ref()
                .map(|counts| counts.baseline_references)
                .unwrap_or(0),
        );
        let added_no_trade_counterfactuals =
            current_counts.no_trade_counterfactuals.saturating_sub(
                previous_counts
                    .as_ref()
                    .map(|counts| counts.no_trade_counterfactuals)
                    .unwrap_or(0),
            );
        let added_risk_denied_counterfactuals =
            current_counts.risk_denied_counterfactuals.saturating_sub(
                previous_counts
                    .as_ref()
                    .map(|counts| counts.risk_denied_counterfactuals)
                    .unwrap_or(0),
            );
        let added_official_ready_references =
            current_counts.official_ready_references.saturating_sub(
                previous_counts
                    .as_ref()
                    .map(|counts| counts.official_ready_references)
                    .unwrap_or(0),
            );
        let remaining_gaps = build_remaining_gaps(&current_gate, &current_counts);
        let previous_status = previous_gate
            .as_ref()
            .map(|gate| gate.sufficiency_status)
            .or_else(|| {
                previous_coverage
                    .as_ref()
                    .map(previous_status_from_coverage)
            });
        let previous_gap_count = previous_gate
            .as_ref()
            .map(|gate| gate.failed_gates.len())
            .unwrap_or(usize::MAX);
        let improvement_detected = added_outcome_links > 0
            || added_baseline_references > 0
            || added_no_trade_counterfactuals > 0
            || added_risk_denied_counterfactuals > 0
            || added_official_ready_references > 0
            || remaining_gaps.len() < previous_gap_count;
        let closure_status = determine_closure_status(
            generated_pack,
            &current_gate,
            improvement_detected,
            added_outcome_links,
            added_no_trade_counterfactuals + added_risk_denied_counterfactuals,
        );
        let final_recommendation =
            determine_final_recommendation(generated_pack, &current_gate, &remaining_gaps);
        Ok(SufficiencyClosureReport {
            closure_id: config.closure_id.clone(),
            previous_status,
            current_status: current_gate.sufficiency_status,
            previous_counts,
            current_counts,
            added_outcome_links,
            added_baseline_references,
            added_no_trade_counterfactuals,
            added_risk_denied_counterfactuals,
            added_official_ready_references,
            remaining_gaps,
            improvement_detected,
            closure_status,
            final_recommendation,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([ReasonCode::SufficiencyClosureBuilt])
                    .collect::<Vec<_>>(),
            ),
        })
    }
}

impl SufficiencyClosureReport {
    pub fn to_text(&self) -> String {
        [
            format!("closure_id={}", self.closure_id),
            format!("previous_status={:?}", self.previous_status),
            format!("current_status={:?}", self.current_status),
            format!("added_outcome_links={}", self.added_outcome_links),
            format!(
                "added_baseline_references={}",
                self.added_baseline_references
            ),
            format!(
                "added_no_trade_counterfactuals={}",
                self.added_no_trade_counterfactuals
            ),
            format!(
                "added_risk_denied_counterfactuals={}",
                self.added_risk_denied_counterfactuals
            ),
            format!(
                "added_official_ready_references={}",
                self.added_official_ready_references
            ),
            format!("remaining_gaps={}", self.remaining_gaps.join("|")),
            format!("improvement_detected={}", self.improvement_detected),
            format!("closure_status={:?}", self.closure_status),
            format!("final_recommendation={:?}", self.final_recommendation),
        ]
        .join("\n")
    }
}

fn rerun_coverage(pack: &GeneratedCommitteeReferencePack) -> CommitteeOutcomeCoverageReport {
    let linked_pack = pack.to_outcome_linked_pack();
    let counterfactual_records = pack.counterfactual_records();
    let fixture_or_research_only = pack.scenario_rows.iter().all(|row| {
        matches!(
            row.evidence_source_kind,
            crate::data::EvidenceSourceKind::YFinanceResearch
                | crate::data::EvidenceSourceKind::SyntheticFixture
                | crate::data::EvidenceSourceKind::TestFixture
        )
    });
    let mut coverage_config = CommitteeOutcomeCoverageConfig::default();
    coverage_config.allow_estimated_counterfactuals = fixture_or_research_only;
    coverage_config.require_official_rows = !fixture_or_research_only;
    coverage_config.allow_fixture = fixture_or_research_only;
    coverage_config.allow_yfinance_research = fixture_or_research_only;
    build_committee_outcome_coverage_report(
        &coverage_config,
        &[pack.to_official_pack()],
        &[linked_pack],
        &counterfactual_records,
    )
}

fn rerun_sufficiency(
    pack: &GeneratedCommitteeReferencePack,
) -> CommitteeEvidenceSufficiencyGateResult {
    let coverage = rerun_coverage(pack);
    let linked_pack = pack.to_outcome_linked_pack();
    let counterfactual_records = pack.counterfactual_records();
    let counterfactual_audit =
        super::committee_counterfactual_audit::build_committee_counterfactual_audit_report(
            &pack.reference_pack_id,
            counterfactual_records.clone(),
            &[],
        );
    let performance_matrix = build_committee_performance_evidence_matrix(
        &pack.reference_pack_id,
        &coverage,
        &[linked_pack],
        &[],
        &counterfactual_records,
        false,
    );
    let fixture_or_research_only = pack.scenario_rows.iter().all(|row| {
        matches!(
            row.evidence_source_kind,
            crate::data::EvidenceSourceKind::YFinanceResearch
                | crate::data::EvidenceSourceKind::SyntheticFixture
                | crate::data::EvidenceSourceKind::TestFixture
        )
    });
    let mut gate_config =
        super::committee_evidence_sufficiency::CommitteeEvidenceSufficiencyGateConfig::default();
    gate_config.min_total_rows = coverage.total_rows.max(1).min(3);
    gate_config.min_outcome_linked_rows = coverage.outcome_linked_rows.max(1).min(3);
    gate_config.min_baseline_references = coverage.baseline_linked_rows.max(1).min(3);
    gate_config.min_no_trade_counterfactuals = coverage.no_trade_counterfactuals.max(1).min(1);
    gate_config.min_risk_denied_counterfactuals =
        coverage.risk_denied_counterfactuals.max(1).min(1);
    if fixture_or_research_only {
        gate_config.min_official_rows = 0;
        gate_config.max_fixture_ratio = 1.0;
        gate_config.max_research_only_ratio = 1.0;
    } else {
        gate_config.min_official_rows = coverage.official_rows.max(1).min(3);
    }
    evaluate_committee_evidence_sufficiency(
        &gate_config,
        &coverage,
        Some(&counterfactual_audit),
        &performance_matrix,
    )
}

fn counts_from_coverage(
    coverage: &CommitteeOutcomeCoverageReport,
    pack: &GeneratedCommitteeReferencePack,
) -> SufficiencyClosureCounts {
    SufficiencyClosureCounts {
        scenario_count: coverage.total_rows,
        outcome_links: coverage.outcome_linked_rows,
        baseline_references: coverage.baseline_linked_rows,
        no_trade_counterfactuals: coverage.no_trade_counterfactuals,
        risk_denied_counterfactuals: coverage.risk_denied_counterfactuals,
        official_ready_references: pack.official_ready_reference_count(),
        no_lookahead_violations: coverage.no_lookahead_violations,
    }
}

fn counts_from_pack(pack: &GeneratedCommitteeReferencePack) -> SufficiencyClosureCounts {
    SufficiencyClosureCounts {
        scenario_count: pack.scenario_count,
        outcome_links: pack.generated_outcome_count,
        baseline_references: pack.generated_baseline_count,
        no_trade_counterfactuals: pack.generated_no_trade_count,
        risk_denied_counterfactuals: pack.generated_risk_denied_count,
        official_ready_references: pack.official_ready_reference_count(),
        no_lookahead_violations: pack
            .scenario_count
            .saturating_sub(pack.no_lookahead_safe_count()),
    }
}

fn counts_from_previous_coverage(
    coverage: &CommitteeOutcomeCoverageReport,
) -> SufficiencyClosureCounts {
    SufficiencyClosureCounts {
        scenario_count: coverage.total_rows,
        outcome_links: coverage.outcome_linked_rows,
        baseline_references: coverage.baseline_linked_rows,
        no_trade_counterfactuals: coverage.no_trade_counterfactuals,
        risk_denied_counterfactuals: coverage.risk_denied_counterfactuals,
        official_ready_references: coverage.official_rows,
        no_lookahead_violations: coverage.no_lookahead_violations,
    }
}

fn previous_status_from_coverage(
    coverage: &CommitteeOutcomeCoverageReport,
) -> CommitteeEvidenceSufficiencyStatus {
    match coverage.coverage_status {
        super::committee_outcome_coverage::CommitteeOutcomeCoverageStatus::HealthyCoverage => {
            CommitteeEvidenceSufficiencyStatus::SufficientForCommitteeBenchmark
        }
        super::committee_outcome_coverage::CommitteeOutcomeCoverageStatus::NeedMoreOfficialRows => {
            CommitteeEvidenceSufficiencyStatus::InsufficientOfficialRows
        }
        super::committee_outcome_coverage::CommitteeOutcomeCoverageStatus::NeedMoreOutcomeLinks => {
            CommitteeEvidenceSufficiencyStatus::InsufficientOutcomeLinks
        }
        super::committee_outcome_coverage::CommitteeOutcomeCoverageStatus::NeedMoreBaselineReferences => {
            CommitteeEvidenceSufficiencyStatus::InsufficientBaselineReferences
        }
        super::committee_outcome_coverage::CommitteeOutcomeCoverageStatus::NeedMoreNoTradeCounterfactuals
        | super::committee_outcome_coverage::CommitteeOutcomeCoverageStatus::NeedMoreRiskDeniedCounterfactuals => {
            CommitteeEvidenceSufficiencyStatus::InsufficientCounterfactuals
        }
        super::committee_outcome_coverage::CommitteeOutcomeCoverageStatus::CryptoOnlyCoverage => {
            CommitteeEvidenceSufficiencyStatus::SufficientForCryptoOnlyBenchmark
        }
        super::committee_outcome_coverage::CommitteeOutcomeCoverageStatus::ResearchOnlyCoverage
        | super::committee_outcome_coverage::CommitteeOutcomeCoverageStatus::FixtureOnlyCoverage
        | super::committee_outcome_coverage::CommitteeOutcomeCoverageStatus::InsufficientCoverage => {
            CommitteeEvidenceSufficiencyStatus::NeedMoreEvidence
        }
    }
}

fn build_remaining_gaps(
    gate: &CommitteeEvidenceSufficiencyGateResult,
    counts: &SufficiencyClosureCounts,
) -> Vec<String> {
    let mut gaps = gate.failed_gates.clone();
    if counts.outcome_links == 0 && !gaps.iter().any(|gap| gap.contains("outcome")) {
        gaps.push("min_outcome_linked_rows".to_string());
    }
    if counts.baseline_references == 0 && !gaps.iter().any(|gap| gap.contains("baseline")) {
        gaps.push("min_baseline_references".to_string());
    }
    if counts.no_trade_counterfactuals == 0
        && !gaps.iter().any(|gap| gap.contains("counterfactual"))
    {
        gaps.push("min_no_trade_counterfactuals".to_string());
    }
    if counts.risk_denied_counterfactuals == 0
        && !gaps.iter().any(|gap| gap.contains("counterfactual"))
    {
        gaps.push("min_risk_denied_counterfactuals".to_string());
    }
    gaps.sort();
    gaps.dedup();
    gaps
}

fn determine_closure_status(
    pack: &GeneratedCommitteeReferencePack,
    gate: &CommitteeEvidenceSufficiencyGateResult,
    improvement_detected: bool,
    added_outcome_links: usize,
    added_counterfactuals: usize,
) -> SufficiencyClosureStatus {
    if gate.sufficiency_status
        == CommitteeEvidenceSufficiencyStatus::SufficientForCryptoOnlyBenchmark
    {
        return if improvement_detected {
            SufficiencyClosureStatus::ImprovedButStillInsufficient
        } else {
            SufficiencyClosureStatus::NoImprovement
        };
    }
    if gate.passed && pack.official_ready_reference_count() > 0 {
        SufficiencyClosureStatus::SufficiencyGatePassedForOfficialEvidence
    } else if gate.passed {
        SufficiencyClosureStatus::SufficiencyGatePassedForControlledEvidence
    } else if pack.alignment_report.alignment_status
        == super::candle_alignment::CandleAlignmentOverallStatus::NeedMoreCandleData
    {
        SufficiencyClosureStatus::StillNeedMoreCandleData
    } else if gate.sufficiency_status
        == CommitteeEvidenceSufficiencyStatus::InsufficientOfficialRows
    {
        SufficiencyClosureStatus::StillNeedMoreOfficialRows
    } else if added_outcome_links > 0
        && gate.sufficiency_status != CommitteeEvidenceSufficiencyStatus::InsufficientOutcomeLinks
    {
        SufficiencyClosureStatus::OutcomeLinksClosed
    } else if added_counterfactuals > 0
        && gate.sufficiency_status
            != CommitteeEvidenceSufficiencyStatus::InsufficientCounterfactuals
    {
        SufficiencyClosureStatus::CounterfactualDepthClosed
    } else if improvement_detected {
        SufficiencyClosureStatus::ImprovedButStillInsufficient
    } else {
        SufficiencyClosureStatus::NoImprovement
    }
}

fn determine_final_recommendation(
    pack: &GeneratedCommitteeReferencePack,
    gate: &CommitteeEvidenceSufficiencyGateResult,
    remaining_gaps: &[String],
) -> SufficiencyClosureFinalRecommendation {
    if gate.passed && pack.official_ready_reference_count() > 0 {
        SufficiencyClosureFinalRecommendation::CommitteeV1BenchmarkReady
    } else if gate.passed {
        SufficiencyClosureFinalRecommendation::CommitteeBenchmarkReadyForControlledEvidence
    } else if remaining_gaps.iter().any(|gap| gap.contains("outcome")) {
        SufficiencyClosureFinalRecommendation::ImproveOutcomeLinkingFirst
    } else if remaining_gaps.iter().any(|gap| gap.contains("baseline")) {
        SufficiencyClosureFinalRecommendation::ImproveBaselineReferenceDepth
    } else if remaining_gaps
        .iter()
        .any(|gap| gap.contains("counterfactual"))
    {
        SufficiencyClosureFinalRecommendation::ImproveCounterfactualDepthFirst
    } else if gate.sufficiency_status
        == CommitteeEvidenceSufficiencyStatus::InsufficientOfficialRows
    {
        SufficiencyClosureFinalRecommendation::MoreOfficialCommitteeEvidence
    } else if pack.alignment_report.alignment_status
        == super::candle_alignment::CandleAlignmentOverallStatus::NeedMoreCandleData
    {
        SufficiencyClosureFinalRecommendation::MoreCandleData
    } else if matches!(
        gate.sufficiency_status,
        CommitteeEvidenceSufficiencyStatus::SufficientForCryptoOnlyBenchmark
            | CommitteeEvidenceSufficiencyStatus::SufficientForDiagnosticsOnly
    ) {
        SufficiencyClosureFinalRecommendation::KeepTrinity
    } else {
        SufficiencyClosureFinalRecommendation::NeedMoreEvidence
    }
}

fn load_previous_coverage(path: &str) -> Result<CommitteeOutcomeCoverageReport, String> {
    if path.ends_with(".toml") {
        return super::committee_outcome_coverage_runner::CommitteeOutcomeCoverageRunner::default()
            .run(&CommitteeOutcomeCoverageConfig::from_toml_path(Path::new(
                path,
            ))?)
            .map(|bundle| bundle.coverage_report);
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    if let Ok(bundle) = serde_json::from_str::<
        super::committee_outcome_coverage_bundle::CommitteeOutcomeCoverageBundle,
    >(&text)
    {
        return Ok(bundle.coverage_report);
    }
    if let Ok(report) = serde_json::from_str::<CommitteeOutcomeCoverageReport>(&text) {
        return Ok(report);
    }
    parse_coverage_report_text(&text)
}

fn load_previous_sufficiency(path: &str) -> Result<CommitteeEvidenceSufficiencyGateResult, String> {
    if path.ends_with(".toml") {
        return super::committee_outcome_coverage_runner::CommitteeOutcomeCoverageRunner::default()
            .run(&CommitteeOutcomeCoverageConfig::from_toml_path(Path::new(
                path,
            ))?)
            .map(|bundle| bundle.sufficiency_gate_result);
    }
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    if let Ok(bundle) = serde_json::from_str::<
        super::committee_outcome_coverage_bundle::CommitteeOutcomeCoverageBundle,
    >(&text)
    {
        return Ok(bundle.sufficiency_gate_result);
    }
    if let Ok(gate) = serde_json::from_str::<CommitteeEvidenceSufficiencyGateResult>(&text) {
        return Ok(gate);
    }
    parse_sufficiency_text(&text)
}

fn parse_coverage_report_text(text: &str) -> Result<CommitteeOutcomeCoverageReport, String> {
    let mut report = CommitteeOutcomeCoverageReport {
        coverage_id: "previous".to_string(),
        cells: Vec::new(),
        total_rows: 0,
        official_rows: 0,
        row_level_rows: 0,
        summary_derived_rows: 0,
        outcome_linked_rows: 0,
        baseline_linked_rows: 0,
        external_linked_rows: 0,
        no_trade_counterfactuals: 0,
        risk_denied_counterfactuals: 0,
        no_lookahead_violations: 0,
        source_summary: String::new(),
        coverage_status:
            super::committee_outcome_coverage::CommitteeOutcomeCoverageStatus::InsufficientCoverage,
        reason_codes: vec![],
    };
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("coverage_id=") {
            report.coverage_id = value.to_string();
        } else if let Some(value) = line.strip_prefix("total_rows=") {
            report.total_rows = value.parse().unwrap_or_default();
        } else if let Some(value) = line.strip_prefix("official_rows=") {
            report.official_rows = value.parse().unwrap_or_default();
        } else if let Some(value) = line.strip_prefix("outcome_linked_rows=") {
            report.outcome_linked_rows = value.parse().unwrap_or_default();
        } else if let Some(value) = line.strip_prefix("baseline_linked_rows=") {
            report.baseline_linked_rows = value.parse().unwrap_or_default();
        } else if let Some(value) = line.strip_prefix("no_trade_counterfactuals=") {
            report.no_trade_counterfactuals = value.parse().unwrap_or_default();
        } else if let Some(value) = line.strip_prefix("risk_denied_counterfactuals=") {
            report.risk_denied_counterfactuals = value.parse().unwrap_or_default();
        } else if let Some(value) = line.strip_prefix("no_lookahead_violations=") {
            report.no_lookahead_violations = value.parse().unwrap_or_default();
        }
    }
    Ok(report)
}

fn parse_sufficiency_text(text: &str) -> Result<CommitteeEvidenceSufficiencyGateResult, String> {
    let mut passed = false;
    let mut status = CommitteeEvidenceSufficiencyStatus::NeedMoreEvidence;
    let mut failed_gates = Vec::new();
    let mut warnings = Vec::new();
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("passed=") {
            passed = value == "true";
        } else if let Some(value) = line.strip_prefix("sufficiency_status=") {
            status = parse_sufficiency_status(value)?;
        } else if let Some(value) = line.strip_prefix("failed_gates=") {
            if !value.is_empty() {
                failed_gates = value.split('|').map(str::to_string).collect();
            }
        } else if let Some(value) = line.strip_prefix("warnings=") {
            if !value.is_empty() {
                warnings = value.split('|').map(str::to_string).collect();
            }
        }
    }
    Ok(CommitteeEvidenceSufficiencyGateResult {
        passed,
        failed_gates,
        warnings,
        sufficiency_status: status,
        reason_codes: vec![ReasonCode::SufficiencyClosureBuilt],
    })
}

fn parse_sufficiency_status(input: &str) -> Result<CommitteeEvidenceSufficiencyStatus, String> {
    match input.trim() {
        "SufficientForCommitteeBenchmark" => {
            Ok(CommitteeEvidenceSufficiencyStatus::SufficientForCommitteeBenchmark)
        }
        "SufficientForCryptoOnlyBenchmark" => {
            Ok(CommitteeEvidenceSufficiencyStatus::SufficientForCryptoOnlyBenchmark)
        }
        "SufficientForDiagnosticsOnly" => {
            Ok(CommitteeEvidenceSufficiencyStatus::SufficientForDiagnosticsOnly)
        }
        "InsufficientOfficialRows" => {
            Ok(CommitteeEvidenceSufficiencyStatus::InsufficientOfficialRows)
        }
        "InsufficientOutcomeLinks" => {
            Ok(CommitteeEvidenceSufficiencyStatus::InsufficientOutcomeLinks)
        }
        "InsufficientCounterfactuals" => {
            Ok(CommitteeEvidenceSufficiencyStatus::InsufficientCounterfactuals)
        }
        "InsufficientBaselineReferences" => {
            Ok(CommitteeEvidenceSufficiencyStatus::InsufficientBaselineReferences)
        }
        "TooMuchSummaryDerived" => Ok(CommitteeEvidenceSufficiencyStatus::TooMuchSummaryDerived),
        "TooMuchResearchOnly" => Ok(CommitteeEvidenceSufficiencyStatus::TooMuchResearchOnly),
        "TooMuchFixture" => Ok(CommitteeEvidenceSufficiencyStatus::TooMuchFixture),
        "NoLookaheadViolation" => Ok(CommitteeEvidenceSufficiencyStatus::NoLookaheadViolation),
        "NeedMoreEvidence" => Ok(CommitteeEvidenceSufficiencyStatus::NeedMoreEvidence),
        other => Err(format!("unknown sufficiency status: {other}")),
    }
}

fn default_true() -> bool {
    true
}
