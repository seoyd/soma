use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::experiment::CoreScorecardRerunSummary;

use super::balanced_outcome_coverage::BalancedOutcomeCoverageReport;
use super::barrier_profile_registry::BarrierProfileRegistry;
use super::batch_counterfactual_completion::BatchCounterfactualCompletionReport;
use super::batch_outcome_linkage_v3::BatchOutcomeLinkageV3Report;
use super::diversity_aware_sufficiency_v2::DiversityAwareSufficiencyV2Report;
use super::official_diversity_row_selector::OfficialDiversityRowSelectorReport;
use super::official_evidence_diversity_gap::OfficialEvidenceDiversityGapMap;
use super::official_evidence_diversity_sweep::{
    OfficialEvidenceDiversityStorageReport, OfficialEvidenceDiversitySweepReport,
};
use super::outcome_diversity_audit::OutcomeDiversityAuditReport;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceDiversitySweepBundle {
    #[serde(default)]
    pub barrier_profile_registry: Option<BarrierProfileRegistry>,
    pub diversity_gap_map: OfficialEvidenceDiversityGapMap,
    #[serde(default)]
    pub row_selector_report: Option<OfficialDiversityRowSelectorReport>,
    #[serde(default)]
    pub batch_outcome_linkage_report: Option<BatchOutcomeLinkageV3Report>,
    #[serde(default)]
    pub batch_counterfactual_completion_report: Option<BatchCounterfactualCompletionReport>,
    pub outcome_diversity_audit_report: OutcomeDiversityAuditReport,
    pub balanced_outcome_coverage_report: BalancedOutcomeCoverageReport,
    pub diversity_aware_sufficiency_v2_report: DiversityAwareSufficiencyV2Report,
    #[serde(default)]
    pub committee_benchmark_summary: Option<String>,
    #[serde(default)]
    pub outcome_coverage_summary: Option<String>,
    #[serde(default)]
    pub counterfactual_depth_summary: Option<String>,
    #[serde(default)]
    pub core_performance_summary: Option<CoreScorecardRerunSummary>,
    pub diversity_sweep_report: OfficialEvidenceDiversitySweepReport,
    pub storage_report: OfficialEvidenceDiversityStorageReport,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialEvidenceDiversitySweepBundle {
    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        if let Some(registry) = self.barrier_profile_registry.as_ref() {
            registry.write_to_dir(output_dir)?;
        }
        self.diversity_gap_map.write_to_dir(output_dir)?;
        if let Some(report) = self.row_selector_report.as_ref() {
            report.write_to_dir(output_dir)?;
        }
        if let Some(report) = self.batch_outcome_linkage_report.as_ref() {
            report.write_to_dir(output_dir)?;
        }
        if let Some(report) = self.batch_counterfactual_completion_report.as_ref() {
            report.write_to_dir(output_dir)?;
        }
        self.outcome_diversity_audit_report
            .write_to_dir(output_dir)?;
        self.balanced_outcome_coverage_report
            .write_to_dir(output_dir)?;
        self.diversity_aware_sufficiency_v2_report
            .write_to_dir(output_dir)?;
        if let Some(summary) = self.committee_benchmark_summary.as_ref() {
            fs::write(output_dir.join("committee_benchmark_summary.txt"), summary)
                .map_err(|err| err.to_string())?;
        }
        if let Some(summary) = self.outcome_coverage_summary.as_ref() {
            fs::write(output_dir.join("outcome_coverage_summary.txt"), summary)
                .map_err(|err| err.to_string())?;
        }
        if let Some(summary) = self.counterfactual_depth_summary.as_ref() {
            fs::write(output_dir.join("counterfactual_depth_summary.txt"), summary)
                .map_err(|err| err.to_string())?;
        }
        if let Some(summary) = self.core_performance_summary.as_ref() {
            fs::write(
                output_dir.join("core_performance_summary.txt"),
                summary.to_text(),
            )
            .map_err(|err| err.to_string())?;
        }
        fs::write(
            output_dir.join("official_evidence_diversity_sweep_report.txt"),
            self.diversity_sweep_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("storage_report.txt"),
            self.storage_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_evidence_diversity_summary.txt"),
            &self.final_summary,
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("official_evidence_diversity_sweep_bundle.json");
        fs::write(
            &json_path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn build_official_evidence_diversity_summary(
    bundle: &OfficialEvidenceDiversitySweepBundle,
) -> String {
    [
        format!("run_id={}", bundle.diversity_sweep_report.run_id),
        format!("final_status={:?}", bundle.diversity_sweep_report.final_status),
        format!(
            "final_recommendation={:?}",
            bundle.diversity_sweep_report.final_recommendation
        ),
        format!(
            "current_official_complete_rows={}",
            bundle.diversity_sweep_report.current_official_complete_rows
        ),
        format!(
            "current_outcome_diversity_status={:?}",
            bundle.diversity_sweep_report.current_outcome_diversity_status
        ),
        format!(
            "current_sufficiency_status={:?}",
            bundle.diversity_sweep_report.current_sufficiency_status
        ),
        format!("storage_budget_exceeded={}", bundle.storage_report.budget_exceeded),
        "research_only_warning=official evidence diversity sweeps remain research-only, paper-only, local-only, and never imply live readiness or profitability"
            .to_string(),
    ]
    .join("\n")
}

pub fn build_official_evidence_diversity_reason_codes() -> Vec<ReasonCode> {
    stable_reason_codes(&[
        ReasonCode::DeterministicPath,
        ReasonCode::OfficialEvidenceCounted,
        ReasonCode::LocalFileOnly,
    ])
}
