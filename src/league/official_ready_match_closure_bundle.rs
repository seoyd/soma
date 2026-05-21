use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::official_candle_join_audit::OfficialCandleJoinAuditReport;
use super::official_ready_match_closure::OfficialReadyMatchClosureReport;
use super::row_candle_candidate::RowCandleCandidateReport;
use super::{
    CandleExpansionArtifactSize, CandleExpansionStorageReport, GapExpansionConsistencyReport,
    JoinRepairPlan, MatchKeyNormalizationAggregate, OfficialCandleLineageReport,
    build_candle_expansion_storage_report,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialReadyMatchClosureBundle {
    pub audit_report: OfficialCandleJoinAuditReport,
    pub normalization_aggregate: MatchKeyNormalizationAggregate,
    pub candidate_report: RowCandleCandidateReport,
    pub consistency_report: GapExpansionConsistencyReport,
    pub lineage_report: OfficialCandleLineageReport,
    pub repair_plan: JoinRepairPlan,
    pub closure_report: OfficialReadyMatchClosureReport,
    pub storage_report: CandleExpansionStorageReport,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialReadyMatchClosureBundle {
    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("match_key_normalization.txt"),
            self.normalization_aggregate.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("row_candle_candidates.txt"),
            self.candidate_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("gap_expansion_consistency.txt"),
            self.consistency_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_candle_lineage.txt"),
            self.lineage_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("join_repair_plan.txt"),
            self.repair_plan.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_ready_match_closure.txt"),
            self.closure_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("storage_report.txt"),
            self.storage_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_ready_match_summary.txt"),
            &self.final_summary,
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("official_ready_match_closure_bundle.json");
        fs::write(
            &json_path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn build_official_ready_match_storage_report(
    bundle: &OfficialReadyMatchClosureBundle,
    budget_bytes: usize,
) -> CandleExpansionStorageReport {
    let artifacts = vec![
        (
            "match_key_normalization.txt",
            bundle.normalization_aggregate.to_text(),
        ),
        (
            "row_candle_candidates.txt",
            bundle.candidate_report.to_text(),
        ),
        (
            "gap_expansion_consistency.txt",
            bundle.consistency_report.to_text(),
        ),
        (
            "official_candle_lineage.txt",
            bundle.lineage_report.to_text(),
        ),
        ("join_repair_plan.txt", bundle.repair_plan.to_text()),
        (
            "official_ready_match_closure.txt",
            bundle.closure_report.to_text(),
        ),
        (
            "official_ready_match_summary.txt",
            bundle.final_summary.clone(),
        ),
    ]
    .into_iter()
    .map(|(path, text)| CandleExpansionArtifactSize {
        path: path.to_string(),
        bytes: text.len(),
    })
    .collect::<Vec<_>>();
    build_candle_expansion_storage_report(budget_bytes, artifacts)
}

pub fn build_official_ready_match_final_summary(
    bundle: &OfficialReadyMatchClosureBundle,
) -> String {
    [
        format!("closure_id={}", bundle.closure_report.closure_id),
        format!("closure_status={:?}", bundle.closure_report.closure_status),
        format!(
            "final_recommendation={:?}",
            bundle.closure_report.final_recommendation
        ),
        format!(
            "after_official_ready_matches={}",
            bundle.closure_report.after_official_ready_matches
        ),
        format!(
            "after_backfilled_rows={}",
            bundle.closure_report.after_backfilled_rows
        ),
        format!(
            "current_bottleneck={:?}",
            bundle.closure_report.current_bottleneck
        ),
        "research_only_warning=official-ready match closure remains research-only, paper-only, and local-only".to_string(),
    ]
    .join("\n")
}

pub fn closure_bundle_with_storage(
    mut bundle: OfficialReadyMatchClosureBundle,
    budget_bytes: usize,
) -> OfficialReadyMatchClosureBundle {
    bundle.final_summary = build_official_ready_match_final_summary(&bundle);
    bundle.storage_report = build_official_ready_match_storage_report(&bundle, budget_bytes);
    bundle.reason_codes = stable_reason_codes(&[
        ReasonCode::OfficialCandleCoverageBuilt,
        ReasonCode::DeterministicPath,
    ]);
    bundle
}
