use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::committee_counterfactual_audit::CommitteeCounterfactualAuditReport;
use super::committee_evidence_sufficiency::{
    CommitteeEvidenceSufficiencyGateResult, CommitteeEvidenceSufficiencyStatus,
};
use super::committee_outcome_coverage::{
    CommitteeOutcomeCoverageReport, CommitteeOutcomeCoverageStatus,
};
use super::committee_performance_matrix::{
    CommitteePerformanceEvidenceMatrix, CommitteePerformanceStatus,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeOutcomeCoverageBundleStatus {
    OutcomeCoverageHealthy,
    NeedMoreOfficialRows,
    NeedMoreOutcomeLinks,
    NeedMoreCounterfactuals,
    NeedMoreBaselineReferences,
    PerformanceEvidenceInsufficient,
    ResearchOnly,
    FixtureOnly,
    CryptoOnly,
    NoLookaheadBlocked,
    NeedMoreEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeOutcomeCoverageRecommendation {
    MoreOfficialCommitteeEvidence,
    ImproveOutcomeLinkingFirst,
    ImproveCounterfactualDepthFirst,
    ImproveBaselineReferenceDepth,
    ImproveScenarioMaterializationFirst,
    ImproveRiskGovernorFirst,
    CommitteeV1BenchmarkReady,
    SixPersonaDesignReviewOnly,
    KeepTrinity,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeOutcomeCoverageBundle {
    pub coverage_report: CommitteeOutcomeCoverageReport,
    #[serde(default)]
    pub counterfactual_audit_report: Option<CommitteeCounterfactualAuditReport>,
    pub performance_matrix: CommitteePerformanceEvidenceMatrix,
    pub sufficiency_gate_result: CommitteeEvidenceSufficiencyGateResult,
    pub storage_summary: String,
    pub final_status: CommitteeOutcomeCoverageBundleStatus,
    pub final_recommendation: CommitteeOutcomeCoverageRecommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CommitteeOutcomeCoverageBundle {
    pub fn new(
        coverage_report: CommitteeOutcomeCoverageReport,
        counterfactual_audit_report: Option<CommitteeCounterfactualAuditReport>,
        performance_matrix: CommitteePerformanceEvidenceMatrix,
        sufficiency_gate_result: CommitteeEvidenceSufficiencyGateResult,
        storage_summary: String,
    ) -> Self {
        let (final_status, final_recommendation) = determine_final_outcome(
            &coverage_report,
            counterfactual_audit_report.as_ref(),
            &performance_matrix,
            &sufficiency_gate_result,
        );
        Self {
            coverage_report,
            counterfactual_audit_report,
            performance_matrix,
            sufficiency_gate_result,
            storage_summary,
            final_status,
            final_recommendation,
            reason_codes: vec![ReasonCode::CommitteeOutcomeCoverageBundleBuilt],
        }
    }

    pub fn to_text(&self) -> String {
        [
            self.coverage_report.to_text(),
            self.counterfactual_audit_report
                .as_ref()
                .map(|report| report.to_text())
                .unwrap_or_else(|| "counterfactual_audit_report=none".to_string()),
            self.performance_matrix.to_text(),
            self.sufficiency_gate_result.to_text(),
            format!("storage_summary={}", self.storage_summary),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("outcome_coverage_report.txt"),
            self.coverage_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("counterfactual_audit_report.txt"),
            self.counterfactual_audit_report
                .as_ref()
                .map(|report| report.to_text())
                .unwrap_or_else(|| "counterfactual_audit_report=none".to_string()),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("performance_evidence_matrix.txt"),
            self.performance_matrix.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("sufficiency_gate.txt"),
            self.sufficiency_gate_result.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("committee_outcome_coverage_summary.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("committee_outcome_coverage_bundle.json");
        fs::write(
            &json_path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn determine_final_outcome(
    coverage_report: &CommitteeOutcomeCoverageReport,
    counterfactual_audit_report: Option<&CommitteeCounterfactualAuditReport>,
    performance_matrix: &CommitteePerformanceEvidenceMatrix,
    sufficiency_gate_result: &CommitteeEvidenceSufficiencyGateResult,
) -> (
    CommitteeOutcomeCoverageBundleStatus,
    CommitteeOutcomeCoverageRecommendation,
) {
    if sufficiency_gate_result.sufficiency_status
        == CommitteeEvidenceSufficiencyStatus::NoLookaheadViolation
    {
        return (
            CommitteeOutcomeCoverageBundleStatus::NoLookaheadBlocked,
            CommitteeOutcomeCoverageRecommendation::NeedMoreEvidence,
        );
    }
    match coverage_report.coverage_status {
        CommitteeOutcomeCoverageStatus::ResearchOnlyCoverage => {
            return (
                CommitteeOutcomeCoverageBundleStatus::ResearchOnly,
                CommitteeOutcomeCoverageRecommendation::KeepTrinity,
            );
        }
        CommitteeOutcomeCoverageStatus::FixtureOnlyCoverage => {
            return (
                CommitteeOutcomeCoverageBundleStatus::FixtureOnly,
                CommitteeOutcomeCoverageRecommendation::KeepTrinity,
            );
        }
        CommitteeOutcomeCoverageStatus::CryptoOnlyCoverage => {
            return (
                CommitteeOutcomeCoverageBundleStatus::CryptoOnly,
                CommitteeOutcomeCoverageRecommendation::KeepTrinity,
            );
        }
        CommitteeOutcomeCoverageStatus::NeedMoreOfficialRows => {
            return (
                CommitteeOutcomeCoverageBundleStatus::NeedMoreOfficialRows,
                CommitteeOutcomeCoverageRecommendation::MoreOfficialCommitteeEvidence,
            );
        }
        CommitteeOutcomeCoverageStatus::NeedMoreOutcomeLinks => {
            return (
                CommitteeOutcomeCoverageBundleStatus::NeedMoreOutcomeLinks,
                CommitteeOutcomeCoverageRecommendation::ImproveOutcomeLinkingFirst,
            );
        }
        CommitteeOutcomeCoverageStatus::NeedMoreBaselineReferences => {
            return (
                CommitteeOutcomeCoverageBundleStatus::NeedMoreBaselineReferences,
                CommitteeOutcomeCoverageRecommendation::ImproveBaselineReferenceDepth,
            );
        }
        CommitteeOutcomeCoverageStatus::NeedMoreNoTradeCounterfactuals
        | CommitteeOutcomeCoverageStatus::NeedMoreRiskDeniedCounterfactuals => {
            return (
                CommitteeOutcomeCoverageBundleStatus::NeedMoreCounterfactuals,
                CommitteeOutcomeCoverageRecommendation::ImproveCounterfactualDepthFirst,
            );
        }
        CommitteeOutcomeCoverageStatus::HealthyCoverage
        | CommitteeOutcomeCoverageStatus::InsufficientCoverage => {}
    }
    if !sufficiency_gate_result.passed {
        return match sufficiency_gate_result.sufficiency_status {
            CommitteeEvidenceSufficiencyStatus::InsufficientOfficialRows => (
                CommitteeOutcomeCoverageBundleStatus::NeedMoreOfficialRows,
                CommitteeOutcomeCoverageRecommendation::MoreOfficialCommitteeEvidence,
            ),
            CommitteeEvidenceSufficiencyStatus::InsufficientOutcomeLinks => (
                CommitteeOutcomeCoverageBundleStatus::NeedMoreOutcomeLinks,
                CommitteeOutcomeCoverageRecommendation::ImproveOutcomeLinkingFirst,
            ),
            CommitteeEvidenceSufficiencyStatus::InsufficientBaselineReferences => (
                CommitteeOutcomeCoverageBundleStatus::NeedMoreBaselineReferences,
                CommitteeOutcomeCoverageRecommendation::ImproveBaselineReferenceDepth,
            ),
            CommitteeEvidenceSufficiencyStatus::InsufficientCounterfactuals => (
                CommitteeOutcomeCoverageBundleStatus::NeedMoreCounterfactuals,
                CommitteeOutcomeCoverageRecommendation::ImproveCounterfactualDepthFirst,
            ),
            _ => (
                CommitteeOutcomeCoverageBundleStatus::NeedMoreEvidence,
                CommitteeOutcomeCoverageRecommendation::NeedMoreEvidence,
            ),
        };
    }
    if matches!(
        performance_matrix.performance_status,
        CommitteePerformanceStatus::EvidenceInsufficient
    ) {
        return (
            CommitteeOutcomeCoverageBundleStatus::PerformanceEvidenceInsufficient,
            CommitteeOutcomeCoverageRecommendation::ImproveScenarioMaterializationFirst,
        );
    }
    if let Some(audit_report) = counterfactual_audit_report {
        if audit_report.no_trade_count == 0 || audit_report.risk_denied_count == 0 {
            return (
                CommitteeOutcomeCoverageBundleStatus::NeedMoreCounterfactuals,
                CommitteeOutcomeCoverageRecommendation::ImproveCounterfactualDepthFirst,
            );
        }
    }
    match performance_matrix.performance_status {
        CommitteePerformanceStatus::EvidencePositive => (
            CommitteeOutcomeCoverageBundleStatus::OutcomeCoverageHealthy,
            CommitteeOutcomeCoverageRecommendation::CommitteeV1BenchmarkReady,
        ),
        CommitteePerformanceStatus::EvidenceMixed => (
            CommitteeOutcomeCoverageBundleStatus::OutcomeCoverageHealthy,
            CommitteeOutcomeCoverageRecommendation::SixPersonaDesignReviewOnly,
        ),
        CommitteePerformanceStatus::EvidenceNegative => (
            CommitteeOutcomeCoverageBundleStatus::NeedMoreEvidence,
            CommitteeOutcomeCoverageRecommendation::ImproveRiskGovernorFirst,
        ),
        CommitteePerformanceStatus::ResearchOnly => (
            CommitteeOutcomeCoverageBundleStatus::ResearchOnly,
            CommitteeOutcomeCoverageRecommendation::KeepTrinity,
        ),
        CommitteePerformanceStatus::FixtureOnly => (
            CommitteeOutcomeCoverageBundleStatus::FixtureOnly,
            CommitteeOutcomeCoverageRecommendation::KeepTrinity,
        ),
        CommitteePerformanceStatus::CryptoOnly => (
            CommitteeOutcomeCoverageBundleStatus::CryptoOnly,
            CommitteeOutcomeCoverageRecommendation::KeepTrinity,
        ),
        CommitteePerformanceStatus::EvidenceInsufficient => (
            CommitteeOutcomeCoverageBundleStatus::PerformanceEvidenceInsufficient,
            CommitteeOutcomeCoverageRecommendation::NeedMoreEvidence,
        ),
    }
}
