use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::candle_acquisition_job::CandleAcquisitionPlan;
use super::candle_expansion_closure::CandleExpansionClosureReport;
use super::candle_expansion_operator_actions::CandleExpansionOperatorAction;
use super::comparable_evidence_backfill::ComparableEvidenceBackfillReport;
use super::official_candle_coverage_pack::OfficialCandleCoveragePack;
use super::official_candle_expansion_runner::OfficialCandleExpansionReport;
use super::official_candle_gap_map::OfficialCandleCoverageGapMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleExpansionArtifactSize {
    pub path: String,
    pub bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleExpansionStorageReport {
    pub total_bytes: usize,
    pub budget_bytes: usize,
    pub budget_exceeded: bool,
    pub artifact_count: usize,
    pub artifacts: Vec<CandleExpansionArtifactSize>,
    pub largest_artifacts: Vec<CandleExpansionArtifactSize>,
    pub deleted_artifacts: Vec<String>,
    pub compaction_recommendations: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCandleExpansionBundle {
    pub expansion_report: OfficialCandleExpansionReport,
    pub gap_map: OfficialCandleCoverageGapMap,
    pub acquisition_plan: CandleAcquisitionPlan,
    pub operator_actions: Vec<CandleExpansionOperatorAction>,
    #[serde(default)]
    pub new_candle_pack: Option<OfficialCandleCoveragePack>,
    #[serde(default)]
    pub backfill_report: Option<ComparableEvidenceBackfillReport>,
    pub closure_report: CandleExpansionClosureReport,
    pub storage_report: CandleExpansionStorageReport,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CandleExpansionStorageReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("total_bytes={}", self.total_bytes),
            format!("budget_bytes={}", self.budget_bytes),
            format!("budget_exceeded={}", self.budget_exceeded),
            format!("artifact_count={}", self.artifact_count),
            format!("deleted_artifacts={}", self.deleted_artifacts.join("|")),
            format!(
                "compaction_recommendations={}",
                self.compaction_recommendations.join(" | ")
            ),
        ];
        lines.push("artifacts:".to_string());
        lines.extend(
            self.artifacts
                .iter()
                .map(|artifact| format!("path={};bytes={}", artifact.path, artifact.bytes)),
        );
        lines.push("largest_artifacts:".to_string());
        lines.extend(
            self.largest_artifacts
                .iter()
                .map(|artifact| format!("path={};bytes={}", artifact.path, artifact.bytes)),
        );
        lines.join("\n")
    }
}

impl OfficialCandleExpansionBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            self.final_summary.clone(),
            self.expansion_report.to_text(),
            self.gap_map.to_text(),
            self.acquisition_plan.to_text(),
            self.closure_report.to_text(),
            self.storage_report.to_text(),
        ]
        .join("\n\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("candle_gap_map.txt"),
            self.gap_map.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("acquisition_plan.txt"),
            self.acquisition_plan.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("operator_actions.txt"),
            self.operator_actions
                .iter()
                .map(CandleExpansionOperatorAction::to_text)
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("candle_expansion_report.txt"),
            self.expansion_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("candle_expansion_closure.txt"),
            self.closure_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("storage_report.txt"),
            self.storage_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_candle_expansion_summary.txt"),
            &self.final_summary,
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("official_candle_expansion_bundle.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn build_candle_expansion_storage_report(
    budget_bytes: usize,
    artifacts: Vec<CandleExpansionArtifactSize>,
) -> CandleExpansionStorageReport {
    let total_bytes = artifacts
        .iter()
        .map(|artifact| artifact.bytes)
        .sum::<usize>();
    let mut artifacts = artifacts;
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    let mut largest_artifacts = artifacts.clone();
    largest_artifacts.sort_by(|left, right| {
        right
            .bytes
            .cmp(&left.bytes)
            .then(left.path.cmp(&right.path))
    });
    let budget_exceeded = total_bytes > budget_bytes;
    CandleExpansionStorageReport {
        total_bytes,
        budget_bytes,
        budget_exceeded,
        artifact_count: artifacts.len(),
        artifacts,
        largest_artifacts,
        deleted_artifacts: Vec::new(),
        compaction_recommendations: if budget_exceeded {
            vec!["reduce scope or compact local outputs; no silent deletion performed".to_string()]
        } else {
            Vec::new()
        },
        reason_codes: stable_reason_codes(
            &(vec![
                ReasonCode::StorageBudgetReportBuilt,
                ReasonCode::DeterministicPath,
            ]
            .into_iter()
            .chain(budget_exceeded.then_some(ReasonCode::BudgetExceeded))
            .collect::<Vec<_>>()),
        ),
    }
}

pub fn build_expansion_final_summary(
    report: &OfficialCandleExpansionReport,
    closure: &CandleExpansionClosureReport,
    storage: &CandleExpansionStorageReport,
) -> String {
    [
        format!("expansion_id={}", report.expansion_id),
        format!("final_status={:?}", report.final_status),
        format!("final_recommendation={:?}", report.final_recommendation),
        format!("closure_status={:?}", closure.closure_status),
        format!("remaining_gaps={}", closure.remaining_gaps),
        format!("budget_exceeded={}", storage.budget_exceeded),
        "research_only_warning=official candle expansion remains research-only, paper-only, and local-first".to_string(),
    ]
    .join("\n")
}
