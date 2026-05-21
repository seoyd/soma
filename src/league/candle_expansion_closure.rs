use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::experiment::CoreBottleneckKind;

use super::comparable_evidence_backfill::ComparableEvidenceBackfillReport;
use super::official_candle_expansion_runner::{
    CandleExpansionCounts, OfficialCandleExpansionFinalStatus,
    OfficialCandleExpansionRecommendation, OfficialCandleExpansionReport,
};
use super::official_candle_gap_map::{OfficialCandleCoverageGapMap, OfficialCandleGapStatus};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CandleExpansionClosureStatus {
    GapClosed,
    GapImproved,
    #[default]
    GapUnchanged,
    GapMovedToOutcomeLinking,
    GapMovedToCounterfactualDepth,
    GapMovedToOfficialEvidence,
    BlockedByAuth,
    BlockedByData,
    BlockedByAlignment,
    DiagnosticOnlyImprovement,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleExpansionClosureReport {
    pub expansion_id: String,
    #[serde(default)]
    pub previous_gap_status: Option<OfficialCandleGapStatus>,
    pub current_gap_status: OfficialCandleGapStatus,
    #[serde(default)]
    pub previous_bottleneck: Option<CoreBottleneckKind>,
    #[serde(default)]
    pub current_bottleneck: Option<CoreBottleneckKind>,
    pub added_series: usize,
    pub added_official_series: usize,
    pub added_non_crypto_official_series: usize,
    pub added_matches: usize,
    pub added_official_ready_matches: usize,
    pub added_backfilled_rows: usize,
    pub added_complete_rows: usize,
    pub added_references: usize,
    pub added_counterfactuals: usize,
    pub remaining_gaps: usize,
    pub improvement_detected: bool,
    pub closure_status: CandleExpansionClosureStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CandleExpansionClosureReport {
    pub fn to_text(&self) -> String {
        [
            format!("expansion_id={}", self.expansion_id),
            format!("previous_gap_status={:?}", self.previous_gap_status),
            format!("current_gap_status={:?}", self.current_gap_status),
            format!("previous_bottleneck={:?}", self.previous_bottleneck),
            format!("current_bottleneck={:?}", self.current_bottleneck),
            format!("added_series={}", self.added_series),
            format!("added_official_series={}", self.added_official_series),
            format!(
                "added_non_crypto_official_series={}",
                self.added_non_crypto_official_series
            ),
            format!("added_matches={}", self.added_matches),
            format!(
                "added_official_ready_matches={}",
                self.added_official_ready_matches
            ),
            format!("added_backfilled_rows={}", self.added_backfilled_rows),
            format!("added_complete_rows={}", self.added_complete_rows),
            format!("added_references={}", self.added_references),
            format!("added_counterfactuals={}", self.added_counterfactuals),
            format!("remaining_gaps={}", self.remaining_gaps),
            format!("improvement_detected={}", self.improvement_detected),
            format!("closure_status={:?}", self.closure_status),
        ]
        .join("\n")
    }
}

pub fn build_candle_expansion_closure_report(
    report: &OfficialCandleExpansionReport,
    before_gap_map: &OfficialCandleCoverageGapMap,
    after_gap_map: &OfficialCandleCoverageGapMap,
    before_counts: Option<&CandleExpansionCounts>,
    after_counts: &CandleExpansionCounts,
    backfill_report: Option<&ComparableEvidenceBackfillReport>,
) -> CandleExpansionClosureReport {
    let previous_bottleneck = report.previous_primary_bottleneck;
    let current_bottleneck = report.current_primary_bottleneck;
    let before_counts = before_counts.cloned().unwrap_or_default();
    let added_series = after_counts
        .total_series
        .saturating_sub(before_counts.total_series);
    let added_official_series = after_counts
        .official_series
        .saturating_sub(before_counts.official_series);
    let added_non_crypto_official_series = after_counts
        .non_crypto_official_series
        .saturating_sub(before_counts.non_crypto_official_series);
    let added_matches = after_counts.matches.saturating_sub(before_counts.matches);
    let added_official_ready_matches = after_counts
        .official_ready_matches
        .saturating_sub(before_counts.official_ready_matches);
    let added_backfilled_rows = report.added_backfilled_rows;
    let added_complete_rows = after_counts
        .complete_comparable_rows
        .saturating_sub(before_counts.complete_comparable_rows);
    let added_references = report.added_outcome_references;
    let added_counterfactuals = report
        .added_no_trade_counterfactuals
        .saturating_add(report.added_risk_denied_counterfactuals);
    let remaining_gaps = after_gap_map.total_gaps;
    let improvement_detected = report.bottleneck_changed
        || added_series > 0
        || added_matches > 0
        || added_backfilled_rows > 0
        || previous_bottleneck != current_bottleneck
        || before_gap_map.total_gaps > after_gap_map.total_gaps;
    let closure_status =
        determine_closure_status(report, after_gap_map, improvement_detected, backfill_report);
    CandleExpansionClosureReport {
        expansion_id: report.expansion_id.clone(),
        previous_gap_status: Some(before_gap_map.gap_status),
        current_gap_status: after_gap_map.gap_status,
        previous_bottleneck,
        current_bottleneck,
        added_series,
        added_official_series,
        added_non_crypto_official_series,
        added_matches,
        added_official_ready_matches,
        added_backfilled_rows,
        added_complete_rows,
        added_references,
        added_counterfactuals,
        remaining_gaps,
        improvement_detected,
        closure_status,
        reason_codes: stable_reason_codes(&[
            ReasonCode::OfficialCandleCoverageBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

fn determine_closure_status(
    report: &OfficialCandleExpansionReport,
    after_gap_map: &OfficialCandleCoverageGapMap,
    improvement_detected: bool,
    backfill_report: Option<&ComparableEvidenceBackfillReport>,
) -> CandleExpansionClosureStatus {
    match report.final_status {
        OfficialCandleExpansionFinalStatus::MissingAuth => {
            CandleExpansionClosureStatus::BlockedByAuth
        }
        OfficialCandleExpansionFinalStatus::MissingApproval
        | OfficialCandleExpansionFinalStatus::MissingOfficialCsv
        | OfficialCandleExpansionFinalStatus::MissingOfficialProvenance
        | OfficialCandleExpansionFinalStatus::MissingOfficialPreflight
        | OfficialCandleExpansionFinalStatus::StillMissingOfficialCandles => {
            CandleExpansionClosureStatus::BlockedByData
        }
        OfficialCandleExpansionFinalStatus::StillNeedBetterTimestampAlignment
        | OfficialCandleExpansionFinalStatus::StillNeedBetterTimeframeAlignment => {
            CandleExpansionClosureStatus::BlockedByAlignment
        }
        OfficialCandleExpansionFinalStatus::DiagnosticCandleCoverageOnly => {
            CandleExpansionClosureStatus::DiagnosticOnlyImprovement
        }
        _ if matches!(
            after_gap_map.gap_status,
            OfficialCandleGapStatus::NoGapsDetected
        ) =>
        {
            CandleExpansionClosureStatus::GapClosed
        }
        _ if matches!(
            report.final_recommendation,
            OfficialCandleExpansionRecommendation::ImproveOutcomeLinkingFirst
        ) =>
        {
            CandleExpansionClosureStatus::GapMovedToOutcomeLinking
        }
        _ if matches!(
            report.final_recommendation,
            OfficialCandleExpansionRecommendation::ImproveCounterfactualDepthFirst
        ) =>
        {
            CandleExpansionClosureStatus::GapMovedToCounterfactualDepth
        }
        _ if matches!(
            report.final_recommendation,
            OfficialCandleExpansionRecommendation::RerunCorePerformance
        ) && backfill_report.is_some_and(|entry| entry.rows_still_missing_candles == 0) =>
        {
            CandleExpansionClosureStatus::GapMovedToOfficialEvidence
        }
        _ if improvement_detected => CandleExpansionClosureStatus::GapImproved,
        _ => CandleExpansionClosureStatus::GapUnchanged,
    }
}
