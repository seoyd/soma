use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};
use super::official_candle_coverage_pack::{
    OfficialCandleCoveragePack, OfficialCandleSeriesDescriptor, OfficialCandleSeriesSourceClass,
    normalize_symbol,
};
use super::timeframe_alignment::{
    TimeframeAlignmentInput, TimeframeAlignmentReport, TimeframeAlignmentStatus,
    build_timeframe_alignment_report,
};
use super::timestamp_alignment_v2::{
    TimestampAlignmentV2Input, TimestampAlignmentV2Options, TimestampAlignmentV2Report,
    TimestampAlignmentV2Status, build_timestamp_alignment_v2_report,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CandleCoverageMatchStatus {
    Matched,
    MatchedDiagnosticOnly,
    NoMatchingSeries,
    TimeframeMismatch,
    TimestampMismatch,
    InsufficientFutureWindow,
    SourceNotEligible,
    PreflightMissing,
    ProvenanceMissing,
    NoLookaheadRejected,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleCoverageMatch {
    pub scenario_row_id: String,
    #[serde(default)]
    pub comparable_row_id: Option<String>,
    #[serde(default)]
    pub candle_series_id: Option<String>,
    pub source_class: ComparableEvidenceSourceClass,
    pub timeframe_alignment_status: TimeframeAlignmentStatus,
    pub timestamp_alignment_status: TimestampAlignmentV2Status,
    pub match_status: CandleCoverageMatchStatus,
    pub official_ready_match: bool,
    pub benchmark_ready_match: bool,
    pub diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CandleCoverageStatus {
    HealthyCandleCoverage,
    NeedMoreOfficialCandles,
    NeedBetterTimeframeAlignment,
    NeedBetterTimestampAlignment,
    NeedLongerFutureWindows,
    SourceIneligible,
    DiagnosticOnly,
    #[default]
    InsufficientCandleCoverage,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleCoverageMatchReport {
    pub matches: Vec<CandleCoverageMatch>,
    pub matched_count: usize,
    pub official_ready_match_count: usize,
    pub benchmark_ready_match_count: usize,
    pub diagnostic_only_match_count: usize,
    pub no_match_count: usize,
    pub missing_provenance_count: usize,
    pub missing_preflight_count: usize,
    pub timeframe_mismatch_count: usize,
    pub timestamp_mismatch_count: usize,
    pub insufficient_future_window_count: usize,
    pub coverage_status: CandleCoverageStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleCoverageMatchComputation {
    pub timeframe_alignment_report: TimeframeAlignmentReport,
    pub timestamp_alignment_report: TimestampAlignmentV2Report,
    pub match_report: CandleCoverageMatchReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandleCoverageMatchOptions {
    pub allow_timeframe_aggregation: bool,
    pub allow_downsample_diagnostic: bool,
    pub allow_timestamp_tolerance: bool,
    pub timestamp_tolerance_ms: u64,
    pub allow_session_daily_match: bool,
    pub require_no_lookahead_safe: bool,
}

impl Default for CandleCoverageMatchOptions {
    fn default() -> Self {
        Self {
            allow_timeframe_aggregation: false,
            allow_downsample_diagnostic: true,
            allow_timestamp_tolerance: true,
            timestamp_tolerance_ms: 60_000,
            allow_session_daily_match: true,
            require_no_lookahead_safe: true,
        }
    }
}

pub fn build_candle_coverage_match_computation(
    rows: &[ComparableCommitteeEvidenceRow],
    pack: &OfficialCandleCoveragePack,
    options: &CandleCoverageMatchOptions,
) -> CandleCoverageMatchComputation {
    let candidate_pairs = rows
        .iter()
        .map(|row| (row, select_candidate(row, pack)))
        .collect::<Vec<_>>();

    let timeframe_inputs = candidate_pairs
        .iter()
        .filter_map(|(row, candidate)| {
            candidate
                .as_ref()
                .map(|descriptor| TimeframeAlignmentInput {
                    scenario_row_id: row.row_id.clone(),
                    scenario_timeframe: row.timeframe.clone(),
                    candle_series_id: descriptor.candle_series_id.clone(),
                    candle_timeframe: descriptor.timeframe.clone(),
                })
        })
        .collect::<Vec<_>>();
    let timeframe_alignment_report = build_timeframe_alignment_report(
        &timeframe_inputs,
        options.allow_timeframe_aggregation,
        options.allow_downsample_diagnostic,
    );

    let timestamp_inputs = candidate_pairs
        .iter()
        .filter_map(|(row, candidate)| {
            candidate
                .as_ref()
                .map(|descriptor| TimestampAlignmentV2Input {
                    scenario_row_id: row.row_id.clone(),
                    candle_series_id: descriptor.candle_series_id.clone(),
                    candle_path: descriptor.path.clone(),
                    scenario_timestamp_ms: row.timestamp_ms,
                    horizon_bars: row.horizon_bars,
                })
        })
        .collect::<Vec<_>>();
    let timestamp_alignment_report = build_timestamp_alignment_v2_report(
        &timestamp_inputs,
        &TimestampAlignmentV2Options {
            allow_timestamp_tolerance: options.allow_timestamp_tolerance,
            timestamp_tolerance_ms: options.timestamp_tolerance_ms,
            allow_session_daily_match: options.allow_session_daily_match,
            require_no_lookahead_safe: options.require_no_lookahead_safe,
        },
    );

    let mut matches = rows
        .iter()
        .map(|row| {
            build_match(
                row,
                pack,
                &timeframe_alignment_report,
                &timestamp_alignment_report,
            )
        })
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| {
        left.scenario_row_id
            .cmp(&right.scenario_row_id)
            .then(left.candle_series_id.cmp(&right.candle_series_id))
    });
    let matched_count = matches
        .iter()
        .filter(|entry| {
            matches!(
                entry.match_status,
                CandleCoverageMatchStatus::Matched
                    | CandleCoverageMatchStatus::MatchedDiagnosticOnly
            )
        })
        .count();
    let official_ready_match_count = matches
        .iter()
        .filter(|entry| entry.official_ready_match)
        .count();
    let benchmark_ready_match_count = matches
        .iter()
        .filter(|entry| entry.benchmark_ready_match)
        .count();
    let diagnostic_only_match_count = matches.iter().filter(|entry| entry.diagnostic_only).count();
    let no_match_count = matches
        .iter()
        .filter(|entry| entry.match_status == CandleCoverageMatchStatus::NoMatchingSeries)
        .count();
    let missing_provenance_count = matches
        .iter()
        .filter(|entry| entry.match_status == CandleCoverageMatchStatus::ProvenanceMissing)
        .count();
    let missing_preflight_count = matches
        .iter()
        .filter(|entry| entry.match_status == CandleCoverageMatchStatus::PreflightMissing)
        .count();
    let timeframe_mismatch_count = matches
        .iter()
        .filter(|entry| entry.match_status == CandleCoverageMatchStatus::TimeframeMismatch)
        .count();
    let timestamp_mismatch_count = matches
        .iter()
        .filter(|entry| entry.match_status == CandleCoverageMatchStatus::TimestampMismatch)
        .count();
    let insufficient_future_window_count = matches
        .iter()
        .filter(|entry| entry.match_status == CandleCoverageMatchStatus::InsufficientFutureWindow)
        .count();
    let coverage_status = determine_coverage_status(
        &matches,
        official_ready_match_count,
        benchmark_ready_match_count,
    );
    CandleCoverageMatchComputation {
        timeframe_alignment_report,
        timestamp_alignment_report,
        match_report: CandleCoverageMatchReport {
            matches,
            matched_count,
            official_ready_match_count,
            benchmark_ready_match_count,
            diagnostic_only_match_count,
            no_match_count,
            missing_provenance_count,
            missing_preflight_count,
            timeframe_mismatch_count,
            timestamp_mismatch_count,
            insufficient_future_window_count,
            coverage_status,
            reason_codes: stable_reason_codes(&[ReasonCode::OfficialCandleCoverageBuilt]),
        },
    }
}

impl CandleCoverageMatchReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("matched_count={}", self.matched_count),
            format!(
                "official_ready_match_count={}",
                self.official_ready_match_count
            ),
            format!(
                "benchmark_ready_match_count={}",
                self.benchmark_ready_match_count
            ),
            format!(
                "diagnostic_only_match_count={}",
                self.diagnostic_only_match_count
            ),
            format!("no_match_count={}", self.no_match_count),
            format!("missing_provenance_count={}", self.missing_provenance_count),
            format!("missing_preflight_count={}", self.missing_preflight_count),
            format!("timeframe_mismatch_count={}", self.timeframe_mismatch_count),
            format!("timestamp_mismatch_count={}", self.timestamp_mismatch_count),
            format!(
                "insufficient_future_window_count={}",
                self.insufficient_future_window_count
            ),
            format!("coverage_status={:?}", self.coverage_status),
        ];
        lines.extend(self.matches.iter().map(|entry| {
            format!(
                "scenario_row_id={};comparable_row_id={};candle_series_id={};source_class={:?};timeframe_alignment_status={:?};timestamp_alignment_status={:?};match_status={:?};official_ready_match={};benchmark_ready_match={};diagnostic_only={}",
                entry.scenario_row_id,
                entry.comparable_row_id.clone().unwrap_or_default(),
                entry.candle_series_id.clone().unwrap_or_default(),
                entry.source_class,
                entry.timeframe_alignment_status,
                entry.timestamp_alignment_status,
                entry.match_status,
                entry.official_ready_match,
                entry.benchmark_ready_match,
                entry.diagnostic_only,
            )
        }));
        lines.join("\n")
    }
}

fn build_match(
    row: &ComparableCommitteeEvidenceRow,
    pack: &OfficialCandleCoveragePack,
    timeframe_report: &TimeframeAlignmentReport,
    timestamp_report: &TimestampAlignmentV2Report,
) -> CandleCoverageMatch {
    let mut reason_codes = row.reason_codes.clone();
    let Some(descriptor) = select_candidate(row, pack) else {
        reason_codes.push(ReasonCode::MissingOfficialCandles);
        return CandleCoverageMatch {
            scenario_row_id: row
                .scenario_row_id
                .clone()
                .unwrap_or_else(|| row.row_id.clone()),
            comparable_row_id: Some(row.row_id.clone()),
            candle_series_id: None,
            source_class: row.source_class,
            timeframe_alignment_status: TimeframeAlignmentStatus::Unknown,
            timestamp_alignment_status: TimestampAlignmentV2Status::Unknown,
            match_status: CandleCoverageMatchStatus::NoMatchingSeries,
            official_ready_match: false,
            benchmark_ready_match: false,
            diagnostic_only: true,
            reason_codes: stable_reason_codes(&reason_codes),
        };
    };
    let timeframe_status = timeframe_report
        .records
        .iter()
        .find(|record| {
            record.scenario_row_id == row.row_id
                && record.candle_series_id == descriptor.candle_series_id
        })
        .map(|record| record.status)
        .unwrap_or(TimeframeAlignmentStatus::Unknown);
    let timestamp_record = timestamp_report.records.iter().find(|record| {
        record.scenario_row_id == row.row_id
            && record.candle_series_id == descriptor.candle_series_id
    });
    let timestamp_status = timestamp_record
        .map(|record| record.status)
        .unwrap_or(TimestampAlignmentV2Status::Unknown);

    let match_status = if !descriptor.provenance_available {
        reason_codes.push(ReasonCode::MissingOfficialProvenance);
        CandleCoverageMatchStatus::ProvenanceMissing
    } else if !descriptor.preflight_ready {
        reason_codes.push(ReasonCode::MissingOfficialPreflight);
        CandleCoverageMatchStatus::PreflightMissing
    } else if matches!(
        timeframe_status,
        TimeframeAlignmentStatus::IncompatibleUpsample
            | TimeframeAlignmentStatus::IncompatibleMixedGranularity
            | TimeframeAlignmentStatus::MissingScenarioTimeframe
            | TimeframeAlignmentStatus::MissingCandleTimeframe
            | TimeframeAlignmentStatus::Unknown
    ) {
        CandleCoverageMatchStatus::TimeframeMismatch
    } else if timestamp_status == TimestampAlignmentV2Status::RejectedNoLookahead {
        reason_codes.push(ReasonCode::RejectedNoLookaheadReference);
        CandleCoverageMatchStatus::NoLookaheadRejected
    } else if timestamp_status == TimestampAlignmentV2Status::InsufficientFutureWindow {
        CandleCoverageMatchStatus::InsufficientFutureWindow
    } else if matches!(
        timestamp_status,
        TimestampAlignmentV2Status::MissingTimestamp
            | TimestampAlignmentV2Status::OutsideCandleRange
            | TimestampAlignmentV2Status::GapBeforeTimestamp
            | TimestampAlignmentV2Status::GapAfterTimestamp
            | TimestampAlignmentV2Status::DuplicateTimestamp
            | TimestampAlignmentV2Status::BadDataQuality
            | TimestampAlignmentV2Status::Unknown
    ) {
        CandleCoverageMatchStatus::TimestampMismatch
    } else if descriptor_ineligible_for_row(row, descriptor) {
        CandleCoverageMatchStatus::SourceNotEligible
    } else if row.diagnostic_only
        || descriptor.diagnostic_only
        || descriptor.source_class != OfficialCandleSeriesSourceClass::OfficialNonCrypto
        || timeframe_status == TimeframeAlignmentStatus::CompatibleDownsampleDiagnosticOnly
        || timestamp_status == TimestampAlignmentV2Status::SessionDailyMatch
    {
        CandleCoverageMatchStatus::MatchedDiagnosticOnly
    } else {
        CandleCoverageMatchStatus::Matched
    };

    let official_ready_match = match_status == CandleCoverageMatchStatus::Matched
        && descriptor.official_readiness_eligible
        && row.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto
        && row.official_readiness_eligible
        && !row.summary_derived;
    let benchmark_ready_match = matches!(
        match_status,
        CandleCoverageMatchStatus::Matched | CandleCoverageMatchStatus::MatchedDiagnosticOnly
    ) && descriptor.benchmark_eligible
        && matches!(
            row.source_class,
            ComparableEvidenceSourceClass::OfficialNonCrypto
                | ComparableEvidenceSourceClass::OfficialCryptoOnly
        )
        && !matches!(
            match_status,
            CandleCoverageMatchStatus::MatchedDiagnosticOnly
        )
        && timestamp_record.is_some_and(|record| record.no_lookahead_safe);
    let diagnostic_only = matches!(
        match_status,
        CandleCoverageMatchStatus::MatchedDiagnosticOnly
            | CandleCoverageMatchStatus::NoMatchingSeries
            | CandleCoverageMatchStatus::SourceNotEligible
            | CandleCoverageMatchStatus::ProvenanceMissing
            | CandleCoverageMatchStatus::PreflightMissing
            | CandleCoverageMatchStatus::TimeframeMismatch
            | CandleCoverageMatchStatus::TimestampMismatch
            | CandleCoverageMatchStatus::InsufficientFutureWindow
            | CandleCoverageMatchStatus::NoLookaheadRejected
    ) || row.diagnostic_only;

    CandleCoverageMatch {
        scenario_row_id: row
            .scenario_row_id
            .clone()
            .unwrap_or_else(|| row.row_id.clone()),
        comparable_row_id: Some(row.row_id.clone()),
        candle_series_id: Some(descriptor.candle_series_id.clone()),
        source_class: row.source_class,
        timeframe_alignment_status: timeframe_status,
        timestamp_alignment_status: timestamp_status,
        match_status,
        official_ready_match,
        benchmark_ready_match,
        diagnostic_only,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn select_candidate<'a>(
    row: &ComparableCommitteeEvidenceRow,
    pack: &'a OfficialCandleCoveragePack,
) -> Option<&'a OfficialCandleSeriesDescriptor> {
    let normalized_symbol = normalize_symbol(&row.symbol);
    pack.descriptors
        .iter()
        .filter(|descriptor| descriptor.normalized_symbol == normalized_symbol)
        .min_by_key(|descriptor| candidate_key(row, descriptor))
}

fn candidate_key(
    row: &ComparableCommitteeEvidenceRow,
    descriptor: &OfficialCandleSeriesDescriptor,
) -> (usize, usize, String) {
    let class_rank = match descriptor.source_class {
        OfficialCandleSeriesSourceClass::OfficialNonCrypto => 0,
        OfficialCandleSeriesSourceClass::OfficialCryptoOnly => 1,
        OfficialCandleSeriesSourceClass::ControlledDiagnostic => 2,
        OfficialCandleSeriesSourceClass::YFinanceResearch => 3,
        OfficialCandleSeriesSourceClass::FixtureArchitectureTest => 4,
        OfficialCandleSeriesSourceClass::SyntheticTest => 5,
        OfficialCandleSeriesSourceClass::Unknown => 6,
    };
    let boundary_penalty = usize::from(!row_source_accepts_descriptor(
        row.source_class,
        descriptor.source_class,
    ));
    (class_rank, boundary_penalty, descriptor.path.clone())
}

fn descriptor_ineligible_for_row(
    row: &ComparableCommitteeEvidenceRow,
    descriptor: &OfficialCandleSeriesDescriptor,
) -> bool {
    !row_source_accepts_descriptor(row.source_class, descriptor.source_class)
}

fn row_source_accepts_descriptor(
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

fn determine_coverage_status(
    matches: &[CandleCoverageMatch],
    official_ready_match_count: usize,
    benchmark_ready_match_count: usize,
) -> CandleCoverageStatus {
    if official_ready_match_count > 0 {
        return CandleCoverageStatus::HealthyCandleCoverage;
    }
    if matches.iter().any(|entry| {
        entry.match_status == CandleCoverageMatchStatus::TimeframeMismatch
            || matches!(
                entry.timeframe_alignment_status,
                TimeframeAlignmentStatus::CompatibleAggregation
                    | TimeframeAlignmentStatus::CompatibleDownsampleDiagnosticOnly
                    | TimeframeAlignmentStatus::IncompatibleUpsample
                    | TimeframeAlignmentStatus::IncompatibleMixedGranularity
                    | TimeframeAlignmentStatus::MissingScenarioTimeframe
                    | TimeframeAlignmentStatus::MissingCandleTimeframe
                    | TimeframeAlignmentStatus::Unknown
            )
    }) {
        return CandleCoverageStatus::NeedBetterTimeframeAlignment;
    }
    if matches
        .iter()
        .any(|entry| entry.match_status == CandleCoverageMatchStatus::NoLookaheadRejected)
        || matches
            .iter()
            .any(|entry| entry.match_status == CandleCoverageMatchStatus::TimestampMismatch)
    {
        return CandleCoverageStatus::NeedBetterTimestampAlignment;
    }
    if matches
        .iter()
        .any(|entry| entry.match_status == CandleCoverageMatchStatus::InsufficientFutureWindow)
    {
        return CandleCoverageStatus::NeedLongerFutureWindows;
    }
    if matches
        .iter()
        .all(|entry| entry.match_status == CandleCoverageMatchStatus::SourceNotEligible)
        && !matches.is_empty()
    {
        return CandleCoverageStatus::SourceIneligible;
    }
    if benchmark_ready_match_count == 0
        && matches
            .iter()
            .any(|entry| entry.match_status == CandleCoverageMatchStatus::MatchedDiagnosticOnly)
    {
        return CandleCoverageStatus::DiagnosticOnly;
    }
    if matches
        .iter()
        .any(|entry| entry.match_status == CandleCoverageMatchStatus::NoMatchingSeries)
    {
        return CandleCoverageStatus::NeedMoreOfficialCandles;
    }
    CandleCoverageStatus::InsufficientCandleCoverage
}
