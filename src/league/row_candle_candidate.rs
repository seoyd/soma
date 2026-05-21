use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};
use super::match_key_normalization::{
    MatchKeyNormalizationAggregate, MatchKeyNormalizationReport, NormalizedMatchKey,
    reports_by_row_id,
};
use super::official_candle_coverage_pack::{
    OfficialCandleCoveragePack, OfficialCandleSeriesDescriptor, OfficialCandleSeriesSourceClass,
};
use super::timeframe_alignment::{
    TimeframeAlignmentInput, TimeframeAlignmentStatus, build_timeframe_alignment_report,
};
use super::timestamp_alignment_v2::{
    TimestampAlignmentV2Input, TimestampAlignmentV2Options, TimestampAlignmentV2Status,
    build_timestamp_alignment_v2_report,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RowCandleCandidateStatus {
    CandidateFound,
    MultipleCandidates,
    NoCandidate,
    SourceIneligible,
    SymbolMismatch,
    MarketMismatch,
    VenueMismatch,
    TimeframeMismatch,
    TimestampOutsideRange,
    MissingFutureWindow,
    MissingProvenance,
    MissingPreflight,
    DiagnosticOnly,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RowCandleCandidate {
    pub row_id: String,
    pub candle_series_id: String,
    pub candidate_score: i32,
    pub source_class: ComparableEvidenceSourceClass,
    pub symbol_match: bool,
    pub market_match: bool,
    pub venue_match: bool,
    pub timeframe_match: bool,
    pub timestamp_range_match: bool,
    pub future_window_available: bool,
    pub official_ready_possible: bool,
    pub benchmark_ready_possible: bool,
    pub diagnostic_only: bool,
    pub timeframe_alignment_status: TimeframeAlignmentStatus,
    pub timestamp_alignment_status: TimestampAlignmentV2Status,
    #[serde(default)]
    pub matched_candle_timestamp_ms: Option<u64>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RowCandleCandidateBucket {
    pub row_id: String,
    pub normalized_key: NormalizedMatchKey,
    pub status: RowCandleCandidateStatus,
    #[serde(default)]
    pub selected_candle_series_id: Option<String>,
    pub candidates: Vec<RowCandleCandidate>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RowCandleCandidateReportStatus {
    HealthyCandidates,
    NoCandidates,
    AmbiguousCandidates,
    SourceIneligible,
    TimeframeBlocked,
    TimestampBlocked,
    FutureWindowBlocked,
    ProvenancePreflightBlocked,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RowCandleCandidateReport {
    pub candidates_by_row: Vec<RowCandleCandidateBucket>,
    pub rows_with_candidates: usize,
    pub rows_without_candidates: usize,
    pub rows_with_multiple_candidates: usize,
    pub official_ready_candidate_count: usize,
    pub benchmark_ready_candidate_count: usize,
    pub diagnostic_candidate_count: usize,
    pub candidate_status: RowCandleCandidateReportStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowCandleCandidateOptions {
    pub allow_session_daily_alignment: bool,
    pub allow_timestamp_tolerance: bool,
    pub timestamp_tolerance_ms: u64,
    pub require_no_lookahead_safe: bool,
    pub require_exact_horizon_match: bool,
    pub require_official_source_for_official_ready: bool,
}

impl Default for RowCandleCandidateOptions {
    fn default() -> Self {
        Self {
            allow_session_daily_alignment: true,
            allow_timestamp_tolerance: true,
            timestamp_tolerance_ms: 60_000,
            require_no_lookahead_safe: true,
            require_exact_horizon_match: true,
            require_official_source_for_official_ready: true,
        }
    }
}

pub fn build_row_candle_candidate_report(
    rows: &[ComparableCommitteeEvidenceRow],
    pack: &OfficialCandleCoveragePack,
    normalization: &MatchKeyNormalizationAggregate,
    options: &RowCandleCandidateOptions,
) -> RowCandleCandidateReport {
    let reports_by_row = reports_by_row_id(normalization);
    let mut candidates_by_row = rows
        .iter()
        .map(|row| build_bucket(row, pack, &reports_by_row, options))
        .collect::<Vec<_>>();
    candidates_by_row.sort_by(|left, right| left.row_id.cmp(&right.row_id));
    let rows_with_candidates = candidates_by_row
        .iter()
        .filter(|bucket| !bucket.candidates.is_empty())
        .count();
    let rows_without_candidates = candidates_by_row.len().saturating_sub(rows_with_candidates);
    let rows_with_multiple_candidates = candidates_by_row
        .iter()
        .filter(|bucket| bucket.candidates.len() > 1)
        .count();
    let official_ready_candidate_count = candidates_by_row
        .iter()
        .filter(|bucket| {
            bucket
                .candidates
                .iter()
                .any(|candidate| candidate.official_ready_possible)
        })
        .count();
    let benchmark_ready_candidate_count = candidates_by_row
        .iter()
        .filter(|bucket| {
            bucket
                .candidates
                .iter()
                .any(|candidate| candidate.benchmark_ready_possible)
        })
        .count();
    let diagnostic_candidate_count = candidates_by_row
        .iter()
        .filter(|bucket| {
            bucket
                .candidates
                .iter()
                .any(|candidate| candidate.diagnostic_only)
        })
        .count();
    let candidate_status = determine_report_status(&candidates_by_row);
    RowCandleCandidateReport {
        candidates_by_row,
        rows_with_candidates,
        rows_without_candidates,
        rows_with_multiple_candidates,
        official_ready_candidate_count,
        benchmark_ready_candidate_count,
        diagnostic_candidate_count,
        candidate_status,
        reason_codes: stable_reason_codes(&[
            ReasonCode::OfficialCandleCoverageBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

pub fn buckets_by_row_id(
    report: &RowCandleCandidateReport,
) -> BTreeMap<String, RowCandleCandidateBucket> {
    report
        .candidates_by_row
        .iter()
        .cloned()
        .map(|bucket| (bucket.row_id.clone(), bucket))
        .collect()
}

impl RowCandleCandidateReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("rows_with_candidates={}", self.rows_with_candidates),
            format!("rows_without_candidates={}", self.rows_without_candidates),
            format!(
                "rows_with_multiple_candidates={}",
                self.rows_with_multiple_candidates
            ),
            format!(
                "official_ready_candidate_count={}",
                self.official_ready_candidate_count
            ),
            format!(
                "benchmark_ready_candidate_count={}",
                self.benchmark_ready_candidate_count
            ),
            format!(
                "diagnostic_candidate_count={}",
                self.diagnostic_candidate_count
            ),
            format!("candidate_status={:?}", self.candidate_status),
        ];
        for bucket in &self.candidates_by_row {
            lines.push(format!(
                "row_id={};status={:?};selected_candle_series_id={}",
                bucket.row_id,
                bucket.status,
                bucket.selected_candle_series_id.clone().unwrap_or_default()
            ));
            lines.extend(bucket.candidates.iter().map(|candidate| {
                format!(
                    "  candle_series_id={};candidate_score={};symbol_match={};market_match={};timeframe_match={};timestamp_status={:?};future_window_available={};official_ready_possible={};benchmark_ready_possible={};diagnostic_only={}",
                    candidate.candle_series_id,
                    candidate.candidate_score,
                    candidate.symbol_match,
                    candidate.market_match,
                    candidate.timeframe_match,
                    candidate.timestamp_alignment_status,
                    candidate.future_window_available,
                    candidate.official_ready_possible,
                    candidate.benchmark_ready_possible,
                    candidate.diagnostic_only,
                )
            }));
        }
        lines.join("\n")
    }
}

fn build_bucket(
    row: &ComparableCommitteeEvidenceRow,
    pack: &OfficialCandleCoveragePack,
    normalization_reports: &BTreeMap<String, MatchKeyNormalizationReport>,
    options: &RowCandleCandidateOptions,
) -> RowCandleCandidateBucket {
    let normalization = normalization_reports
        .get(&row.row_id)
        .cloned()
        .unwrap_or_else(|| panic!("missing normalization report for row {}", row.row_id));
    let mut candidates = pack
        .descriptors
        .iter()
        .map(|descriptor| build_candidate(row, descriptor, &normalization, options))
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .candidate_score
            .cmp(&left.candidate_score)
            .then(left.diagnostic_only.cmp(&right.diagnostic_only))
            .then(left.candle_series_id.cmp(&right.candle_series_id))
    });
    let meaningful = candidates
        .iter()
        .filter(|candidate| {
            candidate.symbol_match || candidate.market_match || candidate.timeframe_match
        })
        .cloned()
        .collect::<Vec<_>>();
    let selected = meaningful.first().cloned();
    let status = determine_bucket_status(row, &normalization, &candidates, selected.as_ref());
    RowCandleCandidateBucket {
        row_id: row.row_id.clone(),
        normalized_key: normalization.normalized_key,
        status,
        selected_candle_series_id: selected
            .as_ref()
            .map(|candidate| candidate.candle_series_id.clone()),
        candidates: meaningful,
        reason_codes: stable_reason_codes(&row.reason_codes),
    }
}

fn build_candidate(
    row: &ComparableCommitteeEvidenceRow,
    descriptor: &OfficialCandleSeriesDescriptor,
    normalization: &MatchKeyNormalizationReport,
    options: &RowCandleCandidateOptions,
) -> RowCandleCandidate {
    let timeframe_alignment = build_timeframe_alignment_report(
        &[TimeframeAlignmentInput {
            scenario_row_id: row.row_id.clone(),
            scenario_timeframe: normalization.normalized_key.normalized_timeframe.clone(),
            candle_series_id: descriptor.candle_series_id.clone(),
            candle_timeframe: descriptor.timeframe.clone(),
        }],
        false,
        true,
    );
    let timeframe_record = timeframe_alignment
        .records
        .first()
        .cloned()
        .expect("single timeframe record");
    let timestamp_alignment = build_timestamp_alignment_v2_report(
        &[TimestampAlignmentV2Input {
            scenario_row_id: row.row_id.clone(),
            candle_series_id: descriptor.candle_series_id.clone(),
            candle_path: descriptor.path.clone(),
            scenario_timestamp_ms: normalization.normalized_key.timestamp_ms,
            horizon_bars: row.horizon_bars,
        }],
        &TimestampAlignmentV2Options {
            allow_timestamp_tolerance: options.allow_timestamp_tolerance,
            timestamp_tolerance_ms: options.timestamp_tolerance_ms,
            allow_session_daily_match: options.allow_session_daily_alignment,
            require_no_lookahead_safe: options.require_no_lookahead_safe,
        },
    );
    let timestamp_record = timestamp_alignment
        .records
        .first()
        .cloned()
        .expect("single timestamp record");
    let symbol_match =
        descriptor.normalized_symbol == normalization.normalized_key.normalized_symbol;
    let market_match = descriptor.market == normalization.normalized_key.market;
    let venue_match = normalization
        .normalized_key
        .venue
        .as_ref()
        .is_none_or(|venue| descriptor.venue.as_deref() == Some(venue.as_str()));
    let timeframe_match = matches!(
        timeframe_record.status,
        TimeframeAlignmentStatus::ExactMatch | TimeframeAlignmentStatus::CompatibleAggregation
    );
    let timestamp_range_match = matches!(
        timestamp_record.status,
        TimestampAlignmentV2Status::ExactMatch
            | TimestampAlignmentV2Status::ToleranceMatch
            | TimestampAlignmentV2Status::SessionDailyMatch
    );
    let future_window_available = timestamp_record.status
        != TimestampAlignmentV2Status::InsufficientFutureWindow
        && timestamp_record.future_window_end_index.is_some();
    let source_boundary_ok = source_class_matches(row.source_class, descriptor.source_class);
    let official_ready_possible = symbol_match
        && market_match
        && venue_match
        && timeframe_match
        && timestamp_range_match
        && future_window_available
        && descriptor.provenance_available
        && descriptor.preflight_ready
        && source_boundary_ok
        && timestamp_record.no_lookahead_safe
        && !row.summary_derived
        && !row.diagnostic_only
        && descriptor.source_class == OfficialCandleSeriesSourceClass::OfficialNonCrypto
        && (!options.require_official_source_for_official_ready
            || row.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto)
        && (!options.require_exact_horizon_match || row.horizon_bars > 0);
    let benchmark_ready_possible = symbol_match
        && market_match
        && timeframe_match
        && timestamp_range_match
        && future_window_available
        && source_boundary_ok
        && timestamp_record.no_lookahead_safe
        && matches!(
            row.source_class,
            ComparableEvidenceSourceClass::OfficialNonCrypto
                | ComparableEvidenceSourceClass::OfficialCryptoOnly
        );
    let diagnostic_only = row.diagnostic_only
        || descriptor.diagnostic_only
        || !source_boundary_ok
        || matches!(
            timestamp_record.status,
            TimestampAlignmentV2Status::SessionDailyMatch
                | TimestampAlignmentV2Status::ToleranceMatch
        )
        || timeframe_record.status == TimeframeAlignmentStatus::CompatibleDownsampleDiagnosticOnly;

    let mut score = 0i32;
    score += if symbol_match { 50 } else { -50 };
    score += if market_match { 20 } else { -20 };
    score += if venue_match { 5 } else { -5 };
    score += match timeframe_record.status {
        TimeframeAlignmentStatus::ExactMatch => 15,
        TimeframeAlignmentStatus::CompatibleAggregation => 8,
        TimeframeAlignmentStatus::CompatibleDownsampleDiagnosticOnly => 2,
        _ => -10,
    };
    score += match timestamp_record.status {
        TimestampAlignmentV2Status::ExactMatch => 15,
        TimestampAlignmentV2Status::ToleranceMatch => 8,
        TimestampAlignmentV2Status::SessionDailyMatch => 5,
        TimestampAlignmentV2Status::InsufficientFutureWindow => -5,
        _ => -10,
    };
    score += i32::from(descriptor.provenance_available) * 3;
    score += i32::from(descriptor.preflight_ready) * 3;
    score += i32::from(source_boundary_ok) * 4;
    score += i32::from(future_window_available) * 4;

    let mut reason_codes = row.reason_codes.clone();
    reason_codes.extend(descriptor.reason_codes.clone());
    reason_codes.extend(timeframe_record.reason_codes);
    reason_codes.extend(timestamp_record.reason_codes.clone());
    if !symbol_match {
        reason_codes.push(ReasonCode::MissingOfficialCandles);
    }
    if !descriptor.provenance_available {
        reason_codes.push(ReasonCode::MissingOfficialProvenance);
    }
    if !descriptor.preflight_ready {
        reason_codes.push(ReasonCode::MissingOfficialPreflight);
    }
    if timestamp_record.status == TimestampAlignmentV2Status::RejectedNoLookahead {
        reason_codes.push(ReasonCode::RejectedNoLookaheadReference);
    }

    RowCandleCandidate {
        row_id: row.row_id.clone(),
        candle_series_id: descriptor.candle_series_id.clone(),
        candidate_score: score,
        source_class: row.source_class,
        symbol_match,
        market_match,
        venue_match,
        timeframe_match,
        timestamp_range_match,
        future_window_available,
        official_ready_possible,
        benchmark_ready_possible,
        diagnostic_only,
        timeframe_alignment_status: timeframe_record.status,
        timestamp_alignment_status: timestamp_record.status,
        matched_candle_timestamp_ms: timestamp_record.matched_candle_timestamp_ms,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn determine_bucket_status(
    row: &ComparableCommitteeEvidenceRow,
    normalization: &MatchKeyNormalizationReport,
    candidates: &[RowCandleCandidate],
    selected: Option<&RowCandleCandidate>,
) -> RowCandleCandidateStatus {
    if matches!(
        row.source_class,
        ComparableEvidenceSourceClass::ControlledDiagnostic
            | ComparableEvidenceSourceClass::YFinanceResearch
            | ComparableEvidenceSourceClass::FixtureArchitectureTest
            | ComparableEvidenceSourceClass::SyntheticTest
            | ComparableEvidenceSourceClass::Unknown
    ) {
        return RowCandleCandidateStatus::SourceIneligible;
    }
    if row.diagnostic_only {
        return RowCandleCandidateStatus::DiagnosticOnly;
    }
    if candidates.is_empty() {
        return RowCandleCandidateStatus::NoCandidate;
    }
    if candidates.iter().all(|candidate| !candidate.symbol_match) {
        return RowCandleCandidateStatus::SymbolMismatch;
    }
    if candidates.iter().all(|candidate| !candidate.market_match) {
        return RowCandleCandidateStatus::MarketMismatch;
    }
    if candidates.iter().all(|candidate| !candidate.venue_match) {
        return RowCandleCandidateStatus::VenueMismatch;
    }
    let Some(selected) = selected else {
        return RowCandleCandidateStatus::NoCandidate;
    };
    if candidates.len() > 1
        && candidates
            .get(1)
            .is_some_and(|next| next.candidate_score == selected.candidate_score)
    {
        return RowCandleCandidateStatus::MultipleCandidates;
    }
    if !selected.timeframe_match {
        return RowCandleCandidateStatus::TimeframeMismatch;
    }
    if !selected.timestamp_range_match
        && selected.timestamp_alignment_status
            != TimestampAlignmentV2Status::InsufficientFutureWindow
    {
        return RowCandleCandidateStatus::TimestampOutsideRange;
    }
    if !selected.future_window_available
        || selected.timestamp_alignment_status
            == TimestampAlignmentV2Status::InsufficientFutureWindow
    {
        return RowCandleCandidateStatus::MissingFutureWindow;
    }
    if selected
        .reason_codes
        .contains(&ReasonCode::MissingOfficialProvenance)
    {
        return RowCandleCandidateStatus::MissingProvenance;
    }
    if selected
        .reason_codes
        .contains(&ReasonCode::MissingOfficialPreflight)
    {
        return RowCandleCandidateStatus::MissingPreflight;
    }
    if normalization.normalized_key.source_class != row.source_class {
        return RowCandleCandidateStatus::SourceIneligible;
    }
    if selected.diagnostic_only {
        return RowCandleCandidateStatus::DiagnosticOnly;
    }
    RowCandleCandidateStatus::CandidateFound
}

fn determine_report_status(buckets: &[RowCandleCandidateBucket]) -> RowCandleCandidateReportStatus {
    if buckets.is_empty() {
        return RowCandleCandidateReportStatus::NoCandidates;
    }
    if buckets
        .iter()
        .all(|bucket| bucket.status == RowCandleCandidateStatus::SourceIneligible)
    {
        return RowCandleCandidateReportStatus::SourceIneligible;
    }
    if buckets.iter().any(|bucket| {
        matches!(
            bucket.status,
            RowCandleCandidateStatus::MissingPreflight
                | RowCandleCandidateStatus::MissingProvenance
        )
    }) {
        return RowCandleCandidateReportStatus::ProvenancePreflightBlocked;
    }
    if buckets
        .iter()
        .any(|bucket| bucket.status == RowCandleCandidateStatus::MissingFutureWindow)
    {
        return RowCandleCandidateReportStatus::FutureWindowBlocked;
    }
    if buckets
        .iter()
        .any(|bucket| bucket.status == RowCandleCandidateStatus::TimestampOutsideRange)
    {
        return RowCandleCandidateReportStatus::TimestampBlocked;
    }
    if buckets
        .iter()
        .any(|bucket| bucket.status == RowCandleCandidateStatus::TimeframeMismatch)
    {
        return RowCandleCandidateReportStatus::TimeframeBlocked;
    }
    if buckets
        .iter()
        .any(|bucket| bucket.status == RowCandleCandidateStatus::MultipleCandidates)
    {
        return RowCandleCandidateReportStatus::AmbiguousCandidates;
    }
    if buckets
        .iter()
        .all(|bucket| bucket.status == RowCandleCandidateStatus::CandidateFound)
    {
        return RowCandleCandidateReportStatus::HealthyCandidates;
    }
    RowCandleCandidateReportStatus::NoCandidates
}

fn source_class_matches(
    row_class: ComparableEvidenceSourceClass,
    descriptor_class: OfficialCandleSeriesSourceClass,
) -> bool {
    match row_class {
        ComparableEvidenceSourceClass::OfficialNonCrypto => {
            descriptor_class == OfficialCandleSeriesSourceClass::OfficialNonCrypto
        }
        ComparableEvidenceSourceClass::OfficialCryptoOnly => {
            descriptor_class == OfficialCandleSeriesSourceClass::OfficialCryptoOnly
        }
        ComparableEvidenceSourceClass::ControlledDiagnostic => {
            descriptor_class == OfficialCandleSeriesSourceClass::ControlledDiagnostic
        }
        ComparableEvidenceSourceClass::YFinanceResearch => {
            descriptor_class == OfficialCandleSeriesSourceClass::YFinanceResearch
        }
        ComparableEvidenceSourceClass::FixtureArchitectureTest => {
            descriptor_class == OfficialCandleSeriesSourceClass::FixtureArchitectureTest
        }
        ComparableEvidenceSourceClass::SyntheticTest => {
            descriptor_class == OfficialCandleSeriesSourceClass::SyntheticTest
        }
        ComparableEvidenceSourceClass::Unknown => {
            descriptor_class == OfficialCandleSeriesSourceClass::Unknown
        }
    }
}
