use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::baseline_reference_backfill::BaselineReferenceBackfillPlan;
use super::complete_comparable_row_builder::CompleteComparableRowBundle;
use super::complete_row_closure::CompleteRowClosureReport;
use super::core_bottleneck_movement::CoreBottleneckMovementReport;
use super::counterfactual_backfill_plan::CounterfactualBackfillPlan;
use super::official_ready_row_inventory::OfficialReadyRowInventoryReport;
use super::outcome_reference_backfill::OutcomeReferenceBackfillPlan;
use super::scenario_materialization_v3::ScenarioMaterializationV3Report;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompleteRowClosureStorageReport {
    pub max_bytes: usize,
    pub estimated_output_bytes: usize,
    pub within_budget: bool,
    pub guidance: String,
    #[serde(default)]
    pub input_paths: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompleteRowClosureBundle {
    pub inventory_report: OfficialReadyRowInventoryReport,
    pub scenario_materialization_v3_report: ScenarioMaterializationV3Report,
    pub outcome_backfill_plan: OutcomeReferenceBackfillPlan,
    pub baseline_backfill_plan: BaselineReferenceBackfillPlan,
    pub counterfactual_backfill_plan: CounterfactualBackfillPlan,
    pub complete_comparable_row_bundle: CompleteComparableRowBundle,
    pub complete_row_closure_report: CompleteRowClosureReport,
    #[serde(default)]
    pub core_bottleneck_movement_report: Option<CoreBottleneckMovementReport>,
    pub storage_report: CompleteRowClosureStorageReport,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CompleteRowClosureStorageReport {
    pub fn to_text(&self) -> String {
        [
            format!("max_bytes={}", self.max_bytes),
            format!("estimated_output_bytes={}", self.estimated_output_bytes),
            format!("within_budget={}", self.within_budget),
            format!("guidance={}", self.guidance),
            format!("input_paths={}", self.input_paths.join("|")),
        ]
        .join("\n")
    }
}

impl CompleteRowClosureBundle {
    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        self.inventory_report.write_to_dir(output_dir)?;
        self.scenario_materialization_v3_report
            .write_to_dir(output_dir)?;
        self.outcome_backfill_plan.write_to_dir(output_dir)?;
        self.baseline_backfill_plan.write_to_dir(output_dir)?;
        self.counterfactual_backfill_plan.write_to_dir(output_dir)?;
        self.complete_comparable_row_bundle
            .write_to_dir(output_dir)?;
        fs::write(
            output_dir.join("complete_row_closure.txt"),
            self.complete_row_closure_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("storage_report.txt"),
            self.storage_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("core_bottleneck_movement.txt"),
            self.core_bottleneck_movement_report
                .as_ref()
                .map(CoreBottleneckMovementReport::to_text)
                .unwrap_or_else(|| "core_bottleneck_movement=none".to_string()),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("complete_row_closure_summary.txt"),
            &self.final_summary,
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("complete_row_closure_bundle.json");
        fs::write(
            &json_path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn build_complete_row_closure_storage_report(
    max_bytes: usize,
    input_paths: Vec<String>,
    bundle: &CompleteRowClosureBundle,
) -> CompleteRowClosureStorageReport {
    let estimated_output_bytes = bundle.inventory_report.to_text().len()
        + bundle.scenario_materialization_v3_report.to_text().len()
        + bundle.outcome_backfill_plan.to_text().len()
        + bundle.baseline_backfill_plan.to_text().len()
        + bundle.counterfactual_backfill_plan.to_text().len()
        + bundle.complete_comparable_row_bundle.to_text().len()
        + bundle.complete_row_closure_report.to_text().len()
        + bundle
            .core_bottleneck_movement_report
            .as_ref()
            .map(CoreBottleneckMovementReport::to_text)
            .unwrap_or_default()
            .len()
        + bundle.final_summary.len();
    let within_budget = estimated_output_bytes <= max_bytes;
    CompleteRowClosureStorageReport {
        max_bytes,
        estimated_output_bytes,
        within_budget,
        guidance: if within_budget {
            "storage budget respected; artifacts remain local, deterministic, and research-only"
                .to_string()
        } else {
            "storage budget exceeded; compact inputs before rerunning closure".to_string()
        },
        input_paths,
        reason_codes: stable_reason_codes(&[
            ReasonCode::StorageBudgetReportBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

pub fn build_complete_row_closure_final_summary(bundle: &CompleteRowClosureBundle) -> String {
    [
        format!("closure_id={}", bundle.complete_row_closure_report.closure_id),
        format!(
            "closure_status={:?}",
            bundle.complete_row_closure_report.closure_status
        ),
        format!(
            "final_recommendation={:?}",
            bundle.complete_row_closure_report.final_recommendation
        ),
        format!(
            "after_complete_rows={}",
            bundle.complete_row_closure_report.after_complete_rows
        ),
        format!(
            "after_official_complete_rows={}",
            bundle.complete_row_closure_report.after_official_complete_rows
        ),
        format!(
            "current_bottleneck={}",
            bundle
                .complete_row_closure_report
                .current_bottleneck
                .map(|value| format!("{value:?}"))
                .unwrap_or_default()
        ),
        "research_only_warning=complete row closure remains research-only, paper-only, and local-only".to_string(),
    ]
    .join("\n")
}
