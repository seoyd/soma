use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::batch_counterfactual_completion::BatchCounterfactualCompletionReport;
use super::batch_outcome_linkage_v3::BatchOutcomeLinkageV3Report;
use super::future_window_scaleout::FutureWindowScaleOutPlan;
use super::multi_row_official_evidence::MultiRowOfficialEvidenceSet;
use super::official_evidence_scaleout::{
    OfficialEvidenceScaleOutReport, OfficialEvidenceScaleOutStorageReport,
};
use super::official_evidence_sufficiency_v2::OfficialEvidenceSufficiencyV2Report;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceScaleOutBundle {
    pub multi_row_set: MultiRowOfficialEvidenceSet,
    #[serde(default)]
    pub future_window_scaleout_plan: Option<FutureWindowScaleOutPlan>,
    #[serde(default)]
    pub batch_outcome_linkage_report: Option<BatchOutcomeLinkageV3Report>,
    #[serde(default)]
    pub batch_counterfactual_completion_report: Option<BatchCounterfactualCompletionReport>,
    pub sufficiency_v2_report: OfficialEvidenceSufficiencyV2Report,
    pub scaleout_report: OfficialEvidenceScaleOutReport,
    pub storage_report: OfficialEvidenceScaleOutStorageReport,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialEvidenceScaleOutBundle {
    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        self.multi_row_set.write_to_dir(output_dir)?;
        if let Some(plan) = self.future_window_scaleout_plan.as_ref() {
            plan.write_to_dir(output_dir)?;
        }
        if let Some(report) = self.batch_outcome_linkage_report.as_ref() {
            report.write_to_dir(output_dir)?;
        }
        if let Some(report) = self.batch_counterfactual_completion_report.as_ref() {
            report.write_to_dir(output_dir)?;
        }
        self.sufficiency_v2_report.write_to_dir(output_dir)?;
        fs::write(
            output_dir.join("official_evidence_scaleout_report.txt"),
            self.scaleout_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("storage_report.txt"),
            self.storage_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_evidence_scaleout_summary.txt"),
            &self.final_summary,
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("official_evidence_scaleout_bundle.json");
        fs::write(
            &json_path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn build_official_evidence_scaleout_summary(bundle: &OfficialEvidenceScaleOutBundle) -> String {
    [
        format!("scaleout_id={}", bundle.scaleout_report.scaleout_id),
        format!("status={:?}", bundle.scaleout_report.final_status),
        format!(
            "final_recommendation={:?}",
            bundle.scaleout_report.final_recommendation
        ),
        format!(
            "before_official_complete_rows={}",
            bundle.scaleout_report.before_counts.official_complete_rows
        ),
        format!(
            "after_official_complete_rows={}",
            bundle.scaleout_report.after_counts.official_complete_rows
        ),
        format!(
            "after_status={:?}",
            bundle.sufficiency_v2_report.sufficiency_status
        ),
        "research_only_warning=official evidence scaleout remains research-only, paper-only, local-only, and never implies live readiness".to_string(),
    ]
    .join("\n")
}

pub fn build_official_evidence_scaleout_reason_codes() -> Vec<ReasonCode> {
    stable_reason_codes(&[
        ReasonCode::OfficialEvidenceCounted,
        ReasonCode::DeterministicPath,
        ReasonCode::LocalFileOnly,
    ])
}
