use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::complete_row_closure_bundle::CompleteRowClosureStorageReport;
use super::complete_row_closure_v2::CompleteRowClosureV2Report;
use super::counterfactual_completion_v2::CounterfactualCompletionV2Report;
use super::future_window_requirements::FutureWindowRequirementReport;
use super::official_future_window_extension::FutureWindowExtensionPlan;
use super::outcome_linkage_v3::OutcomeLinkageV3Report;
use crate::experiment::CorePerformanceRerunAfterOutcomeLinkage;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompleteRowClosureV2Bundle {
    pub future_window_requirement_report: FutureWindowRequirementReport,
    #[serde(default)]
    pub future_window_extension_plan: Option<FutureWindowExtensionPlan>,
    pub outcome_linkage_v3_report: OutcomeLinkageV3Report,
    pub counterfactual_completion_v2_report: CounterfactualCompletionV2Report,
    pub complete_row_closure_v2_report: CompleteRowClosureV2Report,
    #[serde(default)]
    pub core_performance_rerun_summary: Option<CorePerformanceRerunAfterOutcomeLinkage>,
    pub storage_report: CompleteRowClosureStorageReport,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CompleteRowClosureV2Bundle {
    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        self.future_window_requirement_report
            .write_to_dir(output_dir)?;
        if let Some(plan) = self.future_window_extension_plan.as_ref() {
            plan.write_to_dir(output_dir)?;
        }
        self.outcome_linkage_v3_report.write_to_dir(output_dir)?;
        self.counterfactual_completion_v2_report
            .write_to_dir(output_dir)?;
        fs::write(
            output_dir.join("complete_row_closure_v2.txt"),
            self.complete_row_closure_v2_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("core_performance_rerun_after_outcome_linkage.txt"),
            self.core_performance_rerun_summary
                .as_ref()
                .map(CorePerformanceRerunAfterOutcomeLinkage::to_text)
                .unwrap_or_else(|| "ran=false".to_string()),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("storage_report.txt"),
            self.storage_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("complete_row_closure_v2_summary.txt"),
            &self.final_summary,
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("complete_row_closure_v2_bundle.json");
        fs::write(
            &json_path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn build_complete_row_closure_v2_summary(bundle: &CompleteRowClosureV2Bundle) -> String {
    [
        format!(
            "closure_id={}",
            bundle.complete_row_closure_v2_report.closure_id
        ),
        format!(
            "closure_status={:?}",
            bundle.complete_row_closure_v2_report.closure_status
        ),
        format!(
            "final_recommendation={:?}",
            bundle.complete_row_closure_v2_report.final_recommendation
        ),
        format!(
            "after_complete_rows={}",
            bundle.complete_row_closure_v2_report.after_complete_rows
        ),
        format!(
            "after_official_complete_rows={}",
            bundle.complete_row_closure_v2_report.after_official_complete_rows
        ),
        format!(
            "current_bottleneck={}",
            bundle
                .complete_row_closure_v2_report
                .current_bottleneck
                .map(|value| format!("{value:?}"))
                .unwrap_or_default()
        ),
        "research_only_warning=complete row closure v2 remains research-only, paper-only, and local-only".to_string(),
    ]
    .join("\n")
}

pub fn build_complete_row_closure_v2_storage_report(
    max_bytes: usize,
    input_paths: Vec<String>,
    bundle: &CompleteRowClosureV2Bundle,
) -> CompleteRowClosureStorageReport {
    let estimated_output_bytes = bundle.future_window_requirement_report.to_text().len()
        + bundle
            .future_window_extension_plan
            .as_ref()
            .map(FutureWindowExtensionPlan::to_text)
            .unwrap_or_default()
            .len()
        + bundle.outcome_linkage_v3_report.to_text().len()
        + bundle.counterfactual_completion_v2_report.to_text().len()
        + bundle.complete_row_closure_v2_report.to_text().len()
        + bundle
            .core_performance_rerun_summary
            .as_ref()
            .map(CorePerformanceRerunAfterOutcomeLinkage::to_text)
            .unwrap_or_default()
            .len()
        + bundle.final_summary.len();
    CompleteRowClosureStorageReport {
        max_bytes,
        estimated_output_bytes,
        within_budget: estimated_output_bytes <= max_bytes,
        guidance: if estimated_output_bytes <= max_bytes {
            "storage budget respected; sprint 46 artifacts remain local, deterministic, and research-only".to_string()
        } else {
            "storage budget exceeded; compact local inputs before rerunning sprint 46 closure"
                .to_string()
        },
        input_paths,
        reason_codes: stable_reason_codes(&[
            ReasonCode::StorageBudgetReportBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}
