use std::path::Path;

use serde::{Deserialize, Serialize};

use super::comparable_committee_evidence::ComparableCommitteeEvidenceBundle;
use super::comparable_evidence_quality::ComparableEvidenceQualityReport;
use super::counterfactual_depth_closure::CounterfactualDepthClosureReport;
use super::counterfactual_depth_plan::CounterfactualDepthPlan;
use super::scenario_materialization_closure::ScenarioMaterializationWeakClosureReport;
use crate::core::{ReasonCode, stable_reason_codes};
use crate::experiment::CoreScorecardRerunSummary;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CounterfactualDepthClosureBundle {
    pub closure_id: String,
    pub comparable_evidence_bundle: ComparableCommitteeEvidenceBundle,
    pub comparable_quality_report: ComparableEvidenceQualityReport,
    pub depth_plan: CounterfactualDepthPlan,
    pub closure_report: CounterfactualDepthClosureReport,
    pub materialization_weak_closure_report: ScenarioMaterializationWeakClosureReport,
    #[serde(default)]
    pub scorecard_rerun_summary: Option<CoreScorecardRerunSummary>,
    pub storage_summary: String,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CounterfactualDepthClosureBundle {
    pub fn new(
        closure_id: String,
        comparable_bundle: ComparableCommitteeEvidenceBundle,
        comparable_quality: ComparableEvidenceQualityReport,
        depth_plan: CounterfactualDepthPlan,
        closure_report: CounterfactualDepthClosureReport,
        scenario_materialization_report: ScenarioMaterializationWeakClosureReport,
        scorecard_rerun_summary: Option<CoreScorecardRerunSummary>,
    ) -> Self {
        let storage_summary = format!(
            "rows={};storage_bytes={};build_attempts={}",
            comparable_bundle.rows.len(),
            comparable_bundle.storage_bytes,
            closure_report.build_attempts.len()
        );
        let final_summary = format!(
            "closure_status={:?};final_recommendation={:?};improvement_detected={}",
            closure_report.closure_status,
            closure_report.final_recommendation,
            closure_report.improvement_detected
        );
        Self {
            closure_id,
            comparable_evidence_bundle: comparable_bundle,
            comparable_quality_report: comparable_quality,
            depth_plan,
            closure_report,
            materialization_weak_closure_report: scenario_materialization_report,
            scorecard_rerun_summary,
            storage_summary,
            final_summary,
            reason_codes: stable_reason_codes(&[
                ReasonCode::EvidenceClosureBuilt,
                ReasonCode::DeterministicPath,
            ]),
        }
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_str(input: &str) -> Result<Self, String> {
        serde_json::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_json_str(&text)
    }

    pub fn write_to_dir(&self, dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|err| err.to_string())?;
        std::fs::write(
            dir.join("counterfactual_depth_closure_bundle.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        std::fs::write(
            dir.join("comparable_evidence_bundle.txt"),
            self.comparable_evidence_bundle.to_text(),
        )
        .map_err(|err| err.to_string())?;
        std::fs::write(
            dir.join("comparable_evidence_quality.txt"),
            self.comparable_quality_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        std::fs::write(
            dir.join("counterfactual_depth_plan.txt"),
            self.depth_plan.to_text(),
        )
        .map_err(|err| err.to_string())?;
        std::fs::write(
            dir.join("counterfactual_depth_closure.txt"),
            self.closure_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        std::fs::write(
            dir.join("scenario_materialization_weak_closure.txt"),
            self.materialization_weak_closure_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        if let Some(summary) = &self.scorecard_rerun_summary {
            std::fs::write(
                dir.join("core_scorecard_rerun_summary.txt"),
                summary.to_text(),
            )
            .map_err(|err| err.to_string())?;
        }
        std::fs::write(
            dir.join("counterfactual_depth_summary.txt"),
            &self.final_summary,
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}
