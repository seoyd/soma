use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::comparable_committee_evidence::ComparableCommitteeEvidenceRow;
use super::official_candle_expansion_runner::OfficialCandleExpansionReport;
use super::official_candle_gap_map::OfficialCandleCoverageGapMap;
use super::row_candle_candidate::{
    RowCandleCandidateBucket, RowCandleCandidateReport, RowCandleCandidateStatus, buckets_by_row_id,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialCandleLineageStage {
    ComparableRow,
    ScenarioRow,
    GapCell,
    AcquisitionJob,
    CandleSeriesDescriptor,
    MatchKeyNormalization,
    TimeframeAlignment,
    TimestampAlignment,
    CandleCoverageMatch,
    ComparableBackfill,
    ReferenceGeneration,
    CounterfactualGeneration,
    CoreScorecardRerun,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCandleLineageNode {
    pub stage: OfficialCandleLineageStage,
    #[serde(default)]
    pub artifact_path: Option<String>,
    #[serde(default)]
    pub object_id: Option<String>,
    pub status: String,
    pub summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialCandleLineageTerminalStatus {
    OfficialReadyMatchClosed,
    BackfillClosed,
    ReferenceClosed,
    CounterfactualClosed,
    BlockedNoCandidate,
    BlockedSymbolMismatch,
    BlockedTimeframeMismatch,
    BlockedTimestampMismatch,
    BlockedMissingFutureWindow,
    BlockedMissingProvenance,
    BlockedMissingPreflight,
    BlockedSourceIneligible,
    BlockedNoLookahead,
    DiagnosticOnly,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCandleLineageTrace {
    pub trace_id: String,
    #[serde(default)]
    pub row_id: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub timeframe: Option<String>,
    #[serde(default)]
    pub horizon_bars: Option<usize>,
    pub nodes: Vec<OfficialCandleLineageNode>,
    pub terminal_status: OfficialCandleLineageTerminalStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCandleLineageReport {
    pub traces: Vec<OfficialCandleLineageTrace>,
    pub closed_match_count: usize,
    pub blocked_count: usize,
    pub diagnostic_count: usize,
    pub most_common_terminal_statuses: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_official_candle_lineage_report(
    rows: &[ComparableCommitteeEvidenceRow],
    candidate_report: &RowCandleCandidateReport,
    gap_maps: &[OfficialCandleCoverageGapMap],
    expansion_reports: &[OfficialCandleExpansionReport],
    reference_pack_paths: &[String],
    counterfactual_depth_paths: &[String],
    core_scorecard_paths: &[String],
) -> OfficialCandleLineageReport {
    let buckets = buckets_by_row_id(candidate_report);
    let mut traces = rows
        .iter()
        .filter(|row| !row.candle_official_ready_match)
        .map(|row| {
            build_trace(
                row,
                buckets.get(&row.row_id),
                gap_maps,
                expansion_reports,
                reference_pack_paths,
                counterfactual_depth_paths,
                core_scorecard_paths,
            )
        })
        .collect::<Vec<_>>();
    traces.sort_by(|left, right| left.trace_id.cmp(&right.trace_id));
    let closed_match_count = traces
        .iter()
        .filter(|trace| {
            matches!(
                trace.terminal_status,
                OfficialCandleLineageTerminalStatus::OfficialReadyMatchClosed
                    | OfficialCandleLineageTerminalStatus::BackfillClosed
                    | OfficialCandleLineageTerminalStatus::ReferenceClosed
                    | OfficialCandleLineageTerminalStatus::CounterfactualClosed
            )
        })
        .count();
    let blocked_count = traces.len().saturating_sub(closed_match_count);
    let diagnostic_count = traces
        .iter()
        .filter(|trace| {
            trace.terminal_status == OfficialCandleLineageTerminalStatus::DiagnosticOnly
        })
        .count();
    let mut counts = BTreeMap::<String, usize>::new();
    for trace in &traces {
        *counts
            .entry(format!("{:?}", trace.terminal_status))
            .or_default() += 1;
    }
    let mut most_common_terminal_statuses = counts.into_iter().collect::<Vec<_>>();
    most_common_terminal_statuses
        .sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    OfficialCandleLineageReport {
        traces,
        closed_match_count,
        blocked_count,
        diagnostic_count,
        most_common_terminal_statuses: most_common_terminal_statuses
            .into_iter()
            .map(|(status, count)| format!("{status}:{count}"))
            .collect(),
        reason_codes: stable_reason_codes(&[
            ReasonCode::OfficialCandleCoverageBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

impl OfficialCandleLineageReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("closed_match_count={}", self.closed_match_count),
            format!("blocked_count={}", self.blocked_count),
            format!("diagnostic_count={}", self.diagnostic_count),
            format!(
                "most_common_terminal_statuses={}",
                self.most_common_terminal_statuses.join(" | ")
            ),
        ];
        for trace in &self.traces {
            lines.push(format!(
                "trace_id={};terminal_status={:?};row_id={};symbol={};timeframe={}",
                trace.trace_id,
                trace.terminal_status,
                trace.row_id.clone().unwrap_or_default(),
                trace.symbol.clone().unwrap_or_default(),
                trace.timeframe.clone().unwrap_or_default()
            ));
            lines.extend(trace.nodes.iter().map(|node| {
                format!(
                    "  stage={:?};status={};object_id={};artifact_path={};summary={}",
                    node.stage,
                    node.status,
                    node.object_id.clone().unwrap_or_default(),
                    node.artifact_path.clone().unwrap_or_default(),
                    node.summary,
                )
            }));
        }
        lines.join("\n")
    }
}

fn build_trace(
    row: &ComparableCommitteeEvidenceRow,
    bucket: Option<&RowCandleCandidateBucket>,
    gap_maps: &[OfficialCandleCoverageGapMap],
    expansion_reports: &[OfficialCandleExpansionReport],
    reference_pack_paths: &[String],
    counterfactual_depth_paths: &[String],
    core_scorecard_paths: &[String],
) -> OfficialCandleLineageTrace {
    let matching_gap = gap_maps.iter().flat_map(|map| &map.cells).find(|cell| {
        cell.symbol == row.symbol
            && cell.timeframe == row.timeframe
            && cell.horizon_bars == row.horizon_bars
    });
    let matching_job = expansion_reports
        .iter()
        .flat_map(|report| &report.executed_jobs)
        .find(|job| job.symbol == row.symbol && job.timeframe == row.timeframe);
    let selected = bucket.and_then(|bucket| bucket.candidates.first());
    let terminal_status = determine_terminal_status(row, bucket, selected);
    let nodes = vec![
        OfficialCandleLineageNode {
            stage: OfficialCandleLineageStage::ComparableRow,
            artifact_path: None,
            object_id: Some(row.row_id.clone()),
            status: "loaded".to_string(),
            summary: format!(
                "source_class={:?};summary_derived={}",
                row.source_class, row.summary_derived
            ),
            reason_codes: stable_reason_codes(&row.reason_codes),
        },
        OfficialCandleLineageNode {
            stage: OfficialCandleLineageStage::ScenarioRow,
            artifact_path: None,
            object_id: row.scenario_row_id.clone(),
            status: if row.scenario_row_id.is_some() {
                "linked"
            } else {
                "missing"
            }
            .to_string(),
            summary: row
                .scenario_row_id
                .clone()
                .unwrap_or_else(|| "scenario row unavailable".to_string()),
            reason_codes: stable_reason_codes(&row.reason_codes),
        },
        OfficialCandleLineageNode {
            stage: OfficialCandleLineageStage::GapCell,
            artifact_path: matching_gap
                .and_then(|cell| cell.related_artifact_paths.first().cloned()),
            object_id: matching_gap
                .map(|cell| format!("{}:{}:{}", cell.symbol, cell.timeframe, cell.horizon_bars)),
            status: if matching_gap.is_some() {
                "linked"
            } else {
                "missing"
            }
            .to_string(),
            summary: matching_gap
                .map(|cell| format!("gap_kinds={:?}", cell.gap_kinds))
                .unwrap_or_else(|| "no gap cell for row key".to_string()),
            reason_codes: matching_gap
                .map(|cell| stable_reason_codes(&cell.reason_codes))
                .unwrap_or_default(),
        },
        OfficialCandleLineageNode {
            stage: OfficialCandleLineageStage::AcquisitionJob,
            artifact_path: matching_job.and_then(|job| job.expected_canonical_csv_path.clone()),
            object_id: matching_job.map(|job| job.job_id.clone()),
            status: if matching_job.is_some() {
                "linked"
            } else {
                "missing"
            }
            .to_string(),
            summary: matching_job
                .map(|job| format!("job_kind={:?};status={:?}", job.job_kind, job.status))
                .unwrap_or_else(|| "no acquisition job for row key".to_string()),
            reason_codes: matching_job
                .map(|job| stable_reason_codes(&job.reason_codes))
                .unwrap_or_default(),
        },
        OfficialCandleLineageNode {
            stage: OfficialCandleLineageStage::CandleSeriesDescriptor,
            artifact_path: selected.map(|candidate| candidate.candle_series_id.clone()),
            object_id: selected.map(|candidate| candidate.candle_series_id.clone()),
            status: if selected.is_some() {
                "candidate-selected"
            } else {
                "missing"
            }
            .to_string(),
            summary: selected
                .map(|candidate| {
                    format!(
                        "candidate_score={};official_ready_possible={}",
                        candidate.candidate_score, candidate.official_ready_possible
                    )
                })
                .unwrap_or_else(|| "no candidate candle series".to_string()),
            reason_codes: selected
                .map(|candidate| stable_reason_codes(&candidate.reason_codes))
                .unwrap_or_default(),
        },
        OfficialCandleLineageNode {
            stage: OfficialCandleLineageStage::MatchKeyNormalization,
            artifact_path: None,
            object_id: Some(row.row_id.clone()),
            status: "normalized".to_string(),
            summary: bucket
                .map(|bucket| {
                    format!(
                        "normalized_symbol={};normalized_timeframe={}",
                        bucket.normalized_key.normalized_symbol,
                        bucket.normalized_key.normalized_timeframe
                    )
                })
                .unwrap_or_else(|| "normalization unavailable".to_string()),
            reason_codes: stable_reason_codes(&row.reason_codes),
        },
        OfficialCandleLineageNode {
            stage: OfficialCandleLineageStage::TimeframeAlignment,
            artifact_path: None,
            object_id: selected.map(|candidate| candidate.candle_series_id.clone()),
            status: selected
                .map(|candidate| format!("{:?}", candidate.timeframe_alignment_status))
                .unwrap_or_else(|| "missing".to_string()),
            summary: selected
                .map(|candidate| format!("timeframe_match={}", candidate.timeframe_match))
                .unwrap_or_else(|| "timeframe alignment unavailable".to_string()),
            reason_codes: selected
                .map(|candidate| stable_reason_codes(&candidate.reason_codes))
                .unwrap_or_default(),
        },
        OfficialCandleLineageNode {
            stage: OfficialCandleLineageStage::TimestampAlignment,
            artifact_path: None,
            object_id: selected.map(|candidate| candidate.candle_series_id.clone()),
            status: selected
                .map(|candidate| format!("{:?}", candidate.timestamp_alignment_status))
                .unwrap_or_else(|| "missing".to_string()),
            summary: selected
                .map(|candidate| {
                    format!(
                        "timestamp_range_match={};matched_timestamp_ms={}",
                        candidate.timestamp_range_match,
                        candidate
                            .matched_candle_timestamp_ms
                            .map(|value| value.to_string())
                            .unwrap_or_default()
                    )
                })
                .unwrap_or_else(|| "timestamp alignment unavailable".to_string()),
            reason_codes: selected
                .map(|candidate| stable_reason_codes(&candidate.reason_codes))
                .unwrap_or_default(),
        },
        OfficialCandleLineageNode {
            stage: OfficialCandleLineageStage::CandleCoverageMatch,
            artifact_path: None,
            object_id: Some(row.row_id.clone()),
            status: format!(
                "{:?}",
                bucket.map(|bucket| bucket.status).unwrap_or_default()
            ),
            summary: bucket
                .map(|bucket| {
                    format!(
                        "selected_candle_series_id={}",
                        bucket.selected_candle_series_id.clone().unwrap_or_default()
                    )
                })
                .unwrap_or_else(|| "row candidate report unavailable".to_string()),
            reason_codes: bucket
                .map(|bucket| stable_reason_codes(&bucket.reason_codes))
                .unwrap_or_default(),
        },
        OfficialCandleLineageNode {
            stage: OfficialCandleLineageStage::ComparableBackfill,
            artifact_path: None,
            object_id: Some(row.row_id.clone()),
            status: if row.candle_coverage_available {
                "closed"
            } else {
                "pending"
            }
            .to_string(),
            summary: format!(
                "candle_coverage_available={}",
                row.candle_coverage_available
            ),
            reason_codes: stable_reason_codes(&row.reason_codes),
        },
        OfficialCandleLineageNode {
            stage: OfficialCandleLineageStage::ReferenceGeneration,
            artifact_path: reference_pack_paths.first().cloned(),
            object_id: Some(row.row_id.clone()),
            status: if reference_pack_paths.is_empty() {
                "not-configured"
            } else {
                "configured"
            }
            .to_string(),
            summary: format!("reference_paths={}", reference_pack_paths.join("|")),
            reason_codes: stable_reason_codes(&row.reason_codes),
        },
        OfficialCandleLineageNode {
            stage: OfficialCandleLineageStage::CounterfactualGeneration,
            artifact_path: counterfactual_depth_paths.first().cloned(),
            object_id: Some(row.row_id.clone()),
            status: if counterfactual_depth_paths.is_empty() {
                "not-configured"
            } else {
                "configured"
            }
            .to_string(),
            summary: format!(
                "counterfactual_paths={}",
                counterfactual_depth_paths.join("|")
            ),
            reason_codes: stable_reason_codes(&row.reason_codes),
        },
        OfficialCandleLineageNode {
            stage: OfficialCandleLineageStage::CoreScorecardRerun,
            artifact_path: core_scorecard_paths.first().cloned(),
            object_id: Some(row.row_id.clone()),
            status: if core_scorecard_paths.is_empty() {
                "not-configured"
            } else {
                "configured"
            }
            .to_string(),
            summary: format!("core_scorecard_paths={}", core_scorecard_paths.join("|")),
            reason_codes: stable_reason_codes(&row.reason_codes),
        },
    ];
    OfficialCandleLineageTrace {
        trace_id: format!("lineage-{}", row.row_id),
        row_id: Some(row.row_id.clone()),
        symbol: Some(row.symbol.clone()),
        timeframe: Some(row.timeframe.clone()),
        horizon_bars: Some(row.horizon_bars),
        nodes,
        terminal_status,
        reason_codes: stable_reason_codes(&row.reason_codes),
    }
}

fn determine_terminal_status(
    row: &ComparableCommitteeEvidenceRow,
    bucket: Option<&RowCandleCandidateBucket>,
    selected: Option<&super::row_candle_candidate::RowCandleCandidate>,
) -> OfficialCandleLineageTerminalStatus {
    if row.diagnostic_only {
        return OfficialCandleLineageTerminalStatus::DiagnosticOnly;
    }
    match bucket.map(|bucket| bucket.status).unwrap_or_default() {
        RowCandleCandidateStatus::NoCandidate => {
            OfficialCandleLineageTerminalStatus::BlockedNoCandidate
        }
        RowCandleCandidateStatus::SymbolMismatch => {
            OfficialCandleLineageTerminalStatus::BlockedSymbolMismatch
        }
        RowCandleCandidateStatus::TimeframeMismatch => {
            OfficialCandleLineageTerminalStatus::BlockedTimeframeMismatch
        }
        RowCandleCandidateStatus::TimestampOutsideRange => {
            OfficialCandleLineageTerminalStatus::BlockedTimestampMismatch
        }
        RowCandleCandidateStatus::MissingFutureWindow => {
            OfficialCandleLineageTerminalStatus::BlockedMissingFutureWindow
        }
        RowCandleCandidateStatus::MissingProvenance => {
            OfficialCandleLineageTerminalStatus::BlockedMissingProvenance
        }
        RowCandleCandidateStatus::MissingPreflight => {
            OfficialCandleLineageTerminalStatus::BlockedMissingPreflight
        }
        RowCandleCandidateStatus::SourceIneligible => {
            OfficialCandleLineageTerminalStatus::BlockedSourceIneligible
        }
        RowCandleCandidateStatus::DiagnosticOnly => {
            OfficialCandleLineageTerminalStatus::DiagnosticOnly
        }
        _ => {
            if selected.is_some_and(|candidate| {
                candidate
                    .reason_codes
                    .contains(&ReasonCode::RejectedNoLookaheadReference)
            }) {
                OfficialCandleLineageTerminalStatus::BlockedNoLookahead
            } else if row.candle_coverage_available {
                OfficialCandleLineageTerminalStatus::BackfillClosed
            } else if selected.is_some_and(|candidate| candidate.official_ready_possible) {
                OfficialCandleLineageTerminalStatus::OfficialReadyMatchClosed
            } else {
                OfficialCandleLineageTerminalStatus::ReferenceClosed
            }
        }
    }
}
