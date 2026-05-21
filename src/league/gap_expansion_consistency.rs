use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::official_candle_expansion_runner::{
    OfficialCandleExpansionFinalStatus, OfficialCandleExpansionReport,
};
use super::official_candle_gap_map::{OfficialCandleCoverageGapMap, OfficialCandleGapStatus};
use super::row_candle_candidate::RowCandleCandidateReport;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GapExpansionConsistencyStatus {
    Consistent,
    GapMapSaysNoGapsButClosureHasRemainingGaps,
    ExpansionAddedSeriesButNoMatches,
    AcquisitionJobDidNotTargetGap,
    AddedSeriesDoesNotMatchGapKey,
    GapStillRequiresOperatorAction,
    DiagnosticOnlyImprovement,
    #[default]
    IncompleteArtifacts,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GapExpansionConsistencyReport {
    #[serde(default)]
    pub gap_map_status: Option<OfficialCandleGapStatus>,
    #[serde(default)]
    pub expansion_status: Option<OfficialCandleExpansionFinalStatus>,
    #[serde(default)]
    pub closure_status: Option<String>,
    pub gap_cells: usize,
    pub acquisition_jobs: usize,
    pub added_candle_series: usize,
    pub remaining_gaps: usize,
    pub consistency_status: GapExpansionConsistencyStatus,
    pub inconsistencies: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_gap_expansion_consistency_report(
    gap_maps: &[OfficialCandleCoverageGapMap],
    expansion_reports: &[OfficialCandleExpansionReport],
    candidate_report: &RowCandleCandidateReport,
) -> GapExpansionConsistencyReport {
    let primary_gap_map = gap_maps.first();
    let primary_expansion = expansion_reports.first();
    let gap_cells = gap_maps.iter().map(|map| map.cells.len()).sum();
    let acquisition_jobs = expansion_reports
        .iter()
        .map(|report| report.executed_jobs.len())
        .sum();
    let added_candle_series = expansion_reports
        .iter()
        .map(|report| report.added_official_candle_series)
        .sum();
    let remaining_gaps = primary_gap_map
        .map(|map| map.total_gaps)
        .unwrap_or_default();
    let mut inconsistencies = Vec::new();

    if primary_gap_map.is_none() || primary_expansion.is_none() {
        inconsistencies
            .push("gap map or expansion report missing for full consistency trace".to_string());
    }
    if primary_gap_map.is_some_and(|map| map.gap_status == OfficialCandleGapStatus::NoGapsDetected)
        && primary_expansion.is_some_and(|report| {
            matches!(
                report.final_status,
                OfficialCandleExpansionFinalStatus::StillMissingOfficialCandles
                    | OfficialCandleExpansionFinalStatus::StillNeedBetterTimestampAlignment
                    | OfficialCandleExpansionFinalStatus::StillNeedBetterTimeframeAlignment
                    | OfficialCandleExpansionFinalStatus::StillNeedLongerFutureWindows
                    | OfficialCandleExpansionFinalStatus::StillScenarioMaterializationWeak
            ) || report.after_counts.gap_count > 0
        })
    {
        inconsistencies.push(
            "gap map reports NoGapsDetected while expansion still reports remaining candle gaps"
                .to_string(),
        );
    }
    if primary_expansion.is_some_and(|report| {
        report.added_official_candle_series > 0 && report.added_official_ready_matches == 0
    }) {
        inconsistencies.push(
            "official candle expansion added series but did not increase official-ready matches"
                .to_string(),
        );
    }
    if let (Some(map), Some(report)) = (primary_gap_map, primary_expansion) {
        for cell in &map.cells {
            let targeted = report.executed_jobs.iter().any(|job| {
                job.symbol == cell.symbol
                    && job.timeframe == cell.timeframe
                    && job.horizon_bars == cell.horizon_bars
            });
            if !targeted {
                inconsistencies.push(format!(
                    "acquisition jobs did not target gap cell {} {} {}",
                    cell.symbol, cell.timeframe, cell.horizon_bars
                ));
            }
            if report.added_official_candle_series > 0
                && !report
                    .executed_jobs
                    .iter()
                    .any(|job| job.symbol == cell.symbol && job.timeframe == cell.timeframe)
            {
                inconsistencies.push(format!(
                    "added series do not match gap key for {} {}",
                    cell.symbol, cell.timeframe
                ));
            }
            if cell.requires_operator_action {
                inconsistencies.push(format!(
                    "gap cell {} {} still requires operator action",
                    cell.symbol, cell.timeframe
                ));
            }
        }
        if report.after_counts.official_ready_matches == 0
            && candidate_report.official_ready_candidate_count == 0
            && report.added_official_candle_series > 0
        {
            inconsistencies.push(
                "added official series remain unmatched because row-level joins are still blocked"
                    .to_string(),
            );
        }
    }
    let consistency_status = if primary_gap_map.is_none() || primary_expansion.is_none() {
        GapExpansionConsistencyStatus::IncompleteArtifacts
    } else if inconsistencies
        .iter()
        .any(|item| item.contains("NoGapsDetected"))
    {
        GapExpansionConsistencyStatus::GapMapSaysNoGapsButClosureHasRemainingGaps
    } else if inconsistencies
        .iter()
        .any(|item| item.contains("did not increase official-ready matches"))
    {
        GapExpansionConsistencyStatus::ExpansionAddedSeriesButNoMatches
    } else if inconsistencies
        .iter()
        .any(|item| item.contains("did not target gap cell"))
    {
        GapExpansionConsistencyStatus::AcquisitionJobDidNotTargetGap
    } else if inconsistencies
        .iter()
        .any(|item| item.contains("do not match gap key"))
    {
        GapExpansionConsistencyStatus::AddedSeriesDoesNotMatchGapKey
    } else if inconsistencies
        .iter()
        .any(|item| item.contains("requires operator action"))
    {
        GapExpansionConsistencyStatus::GapStillRequiresOperatorAction
    } else if primary_expansion.is_some_and(|report| {
        matches!(
            report.final_status,
            OfficialCandleExpansionFinalStatus::DiagnosticCandleCoverageOnly
        )
    }) {
        GapExpansionConsistencyStatus::DiagnosticOnlyImprovement
    } else {
        GapExpansionConsistencyStatus::Consistent
    };

    GapExpansionConsistencyReport {
        gap_map_status: primary_gap_map.map(|map| map.gap_status),
        expansion_status: primary_expansion.map(|report| report.final_status),
        closure_status: primary_expansion.map(|report| format!("{:?}", report.final_status)),
        gap_cells,
        acquisition_jobs,
        added_candle_series,
        remaining_gaps,
        consistency_status,
        inconsistencies,
        reason_codes: stable_reason_codes(&[
            ReasonCode::OfficialCandleCoverageBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

impl GapExpansionConsistencyReport {
    pub fn to_text(&self) -> String {
        [
            format!("gap_map_status={:?}", self.gap_map_status),
            format!("expansion_status={:?}", self.expansion_status),
            format!(
                "closure_status={}",
                self.closure_status.clone().unwrap_or_default()
            ),
            format!("gap_cells={}", self.gap_cells),
            format!("acquisition_jobs={}", self.acquisition_jobs),
            format!("added_candle_series={}", self.added_candle_series),
            format!("remaining_gaps={}", self.remaining_gaps),
            format!("consistency_status={:?}", self.consistency_status),
            format!("inconsistencies={}", self.inconsistencies.join(" | ")),
        ]
        .join("\n")
    }
}
