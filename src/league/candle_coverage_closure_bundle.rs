use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::candle_coverage_closure::CandleCoverageClosureReport;
use super::candle_coverage_match::CandleCoverageMatchReport;
use super::candle_coverage_storage::CandleCoverageStorageReport;
use super::comparable_evidence_backfill::ComparableEvidenceBackfillReport;
use super::official_candle_coverage_pack::OfficialCandleCoveragePack;
use super::timeframe_alignment::TimeframeAlignmentReport;
use super::timestamp_alignment_v2::TimestampAlignmentV2Report;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleCoverageClosureBundle {
    pub candle_pack: OfficialCandleCoveragePack,
    pub timeframe_alignment_report: TimeframeAlignmentReport,
    pub timestamp_alignment_report: TimestampAlignmentV2Report,
    pub match_report: CandleCoverageMatchReport,
    #[serde(default)]
    pub backfill_report: Option<ComparableEvidenceBackfillReport>,
    pub closure_report: CandleCoverageClosureReport,
    pub storage_report: CandleCoverageStorageReport,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CandleCoverageClosureBundle {
    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        self.candle_pack.write_to_dir(output_dir)?;
        fs::write(
            output_dir.join("timeframe_alignment.txt"),
            self.timeframe_alignment_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("timestamp_alignment_v2.txt"),
            self.timestamp_alignment_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("candle_coverage_match.txt"),
            self.match_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("comparable_backfill.txt"),
            self.backfill_report
                .as_ref()
                .map(ComparableEvidenceBackfillReport::to_text)
                .unwrap_or_else(|| "comparable_backfill=none".to_string()),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("candle_coverage_closure.txt"),
            self.closure_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("storage_report.txt"),
            self.storage_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("candle_coverage_summary.txt"),
            &self.final_summary,
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("candle_coverage_closure_bundle.json");
        fs::write(
            &json_path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }

    pub fn from_parts(
        candle_pack: OfficialCandleCoveragePack,
        timeframe_alignment_report: TimeframeAlignmentReport,
        timestamp_alignment_report: TimestampAlignmentV2Report,
        match_report: CandleCoverageMatchReport,
        backfill_report: Option<ComparableEvidenceBackfillReport>,
        closure_report: CandleCoverageClosureReport,
        storage_report: CandleCoverageStorageReport,
    ) -> Self {
        let final_summary = format!(
            "final_status={:?};final_recommendation={:?};improvement_detected={}",
            closure_report.final_status,
            closure_report.final_recommendation,
            closure_report.improvement_detected,
        );
        Self {
            candle_pack,
            timeframe_alignment_report,
            timestamp_alignment_report,
            match_report,
            backfill_report,
            closure_report,
            storage_report,
            final_summary,
            reason_codes: stable_reason_codes(&[ReasonCode::OfficialCandleCoverageBuilt]),
        }
    }
}
