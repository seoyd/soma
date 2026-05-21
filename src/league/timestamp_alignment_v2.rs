use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::official_candle_coverage_pack::load_candle_csv_timestamp_series;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TimestampAlignmentV2Status {
    ExactMatch,
    ToleranceMatch,
    SessionDailyMatch,
    MissingTimestamp,
    DuplicateTimestamp,
    GapBeforeTimestamp,
    GapAfterTimestamp,
    InsufficientFutureWindow,
    OutsideCandleRange,
    RejectedNoLookahead,
    BadDataQuality,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimestampAlignmentV2Record {
    pub scenario_row_id: String,
    pub candle_series_id: String,
    pub scenario_timestamp_ms: u64,
    #[serde(default)]
    pub matched_candle_timestamp_ms: Option<u64>,
    #[serde(default)]
    pub matched_index: Option<usize>,
    #[serde(default)]
    pub future_window_start_index: Option<usize>,
    #[serde(default)]
    pub future_window_end_index: Option<usize>,
    pub horizon_bars: usize,
    pub status: TimestampAlignmentV2Status,
    pub no_lookahead_safe: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TimestampAlignmentV2OverallStatus {
    HealthyTimestampAlignment,
    NeedBetterTimestampAlignment,
    NeedLongerCandleCoverage,
    DuplicateTimestampsDetected,
    GapHeavy,
    NoLookaheadBlocked,
    BadDataQuality,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimestampAlignmentV2Report {
    pub records: Vec<TimestampAlignmentV2Record>,
    pub exact_count: usize,
    pub tolerance_count: usize,
    pub session_daily_count: usize,
    pub missing_count: usize,
    pub duplicate_count: usize,
    pub gap_count: usize,
    pub insufficient_future_window_count: usize,
    pub no_lookahead_rejected_count: usize,
    pub alignment_status: TimestampAlignmentV2OverallStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampAlignmentV2Input {
    pub scenario_row_id: String,
    pub candle_series_id: String,
    pub candle_path: String,
    pub scenario_timestamp_ms: u64,
    pub horizon_bars: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimestampAlignmentV2Options {
    pub allow_timestamp_tolerance: bool,
    pub timestamp_tolerance_ms: u64,
    pub allow_session_daily_match: bool,
    pub require_no_lookahead_safe: bool,
}

impl Default for TimestampAlignmentV2Options {
    fn default() -> Self {
        Self {
            allow_timestamp_tolerance: true,
            timestamp_tolerance_ms: 60_000,
            allow_session_daily_match: true,
            require_no_lookahead_safe: true,
        }
    }
}

pub fn build_timestamp_alignment_v2_report(
    inputs: &[TimestampAlignmentV2Input],
    options: &TimestampAlignmentV2Options,
) -> TimestampAlignmentV2Report {
    let mut records = inputs
        .iter()
        .map(|input| build_record(input, options))
        .collect::<Vec<_>>();
    records.sort_by(|left, right| {
        left.scenario_row_id
            .cmp(&right.scenario_row_id)
            .then(left.candle_series_id.cmp(&right.candle_series_id))
    });
    let exact_count = records
        .iter()
        .filter(|record| record.status == TimestampAlignmentV2Status::ExactMatch)
        .count();
    let tolerance_count = records
        .iter()
        .filter(|record| record.status == TimestampAlignmentV2Status::ToleranceMatch)
        .count();
    let session_daily_count = records
        .iter()
        .filter(|record| record.status == TimestampAlignmentV2Status::SessionDailyMatch)
        .count();
    let missing_count = records
        .iter()
        .filter(|record| {
            matches!(
                record.status,
                TimestampAlignmentV2Status::MissingTimestamp
                    | TimestampAlignmentV2Status::OutsideCandleRange
            )
        })
        .count();
    let duplicate_count = records
        .iter()
        .filter(|record| record.status == TimestampAlignmentV2Status::DuplicateTimestamp)
        .count();
    let gap_count = records
        .iter()
        .filter(|record| {
            matches!(
                record.status,
                TimestampAlignmentV2Status::GapBeforeTimestamp
                    | TimestampAlignmentV2Status::GapAfterTimestamp
            )
        })
        .count();
    let insufficient_future_window_count = records
        .iter()
        .filter(|record| record.status == TimestampAlignmentV2Status::InsufficientFutureWindow)
        .count();
    let no_lookahead_rejected_count = records
        .iter()
        .filter(|record| record.status == TimestampAlignmentV2Status::RejectedNoLookahead)
        .count();
    let alignment_status = determine_alignment_status(&records);
    TimestampAlignmentV2Report {
        records,
        exact_count,
        tolerance_count,
        session_daily_count,
        missing_count,
        duplicate_count,
        gap_count,
        insufficient_future_window_count,
        no_lookahead_rejected_count,
        alignment_status,
        reason_codes: stable_reason_codes(&[ReasonCode::CandleAlignmentBuilt]),
    }
}

impl TimestampAlignmentV2Report {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("exact_count={}", self.exact_count),
            format!("tolerance_count={}", self.tolerance_count),
            format!("session_daily_count={}", self.session_daily_count),
            format!("missing_count={}", self.missing_count),
            format!("duplicate_count={}", self.duplicate_count),
            format!("gap_count={}", self.gap_count),
            format!(
                "insufficient_future_window_count={}",
                self.insufficient_future_window_count
            ),
            format!(
                "no_lookahead_rejected_count={}",
                self.no_lookahead_rejected_count
            ),
            format!("alignment_status={:?}", self.alignment_status),
        ];
        lines.extend(self.records.iter().map(|record| {
            format!(
                "scenario_row_id={};candle_series_id={};scenario_timestamp_ms={};matched_candle_timestamp_ms={};status={:?};no_lookahead_safe={}",
                record.scenario_row_id,
                record.candle_series_id,
                record.scenario_timestamp_ms,
                record.matched_candle_timestamp_ms
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                record.status,
                record.no_lookahead_safe,
            )
        }));
        lines.join("\n")
    }
}

fn build_record(
    input: &TimestampAlignmentV2Input,
    options: &TimestampAlignmentV2Options,
) -> TimestampAlignmentV2Record {
    let mut reason_codes = Vec::new();
    if input.scenario_timestamp_ms == 0 {
        reason_codes.push(ReasonCode::StaleTimestamp);
        return record(
            input,
            TimestampAlignmentV2Status::MissingTimestamp,
            false,
            None,
            None,
            None,
            None,
            reason_codes,
        );
    }
    let series = match load_candle_csv_timestamp_series(Path::new(&input.candle_path)) {
        Ok(series) => series,
        Err(_) => {
            reason_codes.push(ReasonCode::DataLoadFailed);
            return record(
                input,
                TimestampAlignmentV2Status::BadDataQuality,
                false,
                None,
                None,
                None,
                None,
                reason_codes,
            );
        }
    };
    let timestamps = series.timestamps;
    if timestamps.is_empty() {
        reason_codes.push(ReasonCode::DataUnusable);
        return record(
            input,
            TimestampAlignmentV2Status::BadDataQuality,
            false,
            None,
            None,
            None,
            None,
            reason_codes,
        );
    }
    let duplicate_count = timestamps
        .windows(2)
        .filter(|window| window[0] == window[1])
        .count();
    if duplicate_count > 0 {
        reason_codes.push(ReasonCode::DuplicateTimestampDetected);
        return record(
            input,
            TimestampAlignmentV2Status::DuplicateTimestamp,
            false,
            None,
            None,
            None,
            None,
            reason_codes,
        );
    }
    let min_step = timestamps
        .windows(2)
        .filter_map(|window| {
            let step = window[1].saturating_sub(window[0]);
            (step > 0).then_some(step)
        })
        .min();
    if timestamps.len() == 1 && input.horizon_bars > 0 {
        reason_codes.push(ReasonCode::InsufficientBars);
        return record(
            input,
            TimestampAlignmentV2Status::InsufficientFutureWindow,
            false,
            timestamps.first().copied(),
            Some(0),
            None,
            None,
            reason_codes,
        );
    }
    if input.scenario_timestamp_ms < timestamps[0]
        || input.scenario_timestamp_ms > *timestamps.last().unwrap_or(&timestamps[0])
    {
        reason_codes.push(ReasonCode::StaleTimestamp);
        return record(
            input,
            TimestampAlignmentV2Status::OutsideCandleRange,
            false,
            None,
            None,
            None,
            None,
            reason_codes,
        );
    }
    let exact_matches = timestamps
        .iter()
        .enumerate()
        .filter_map(|(index, timestamp)| {
            (*timestamp == input.scenario_timestamp_ms).then_some((index, *timestamp))
        })
        .collect::<Vec<_>>();
    if exact_matches.len() > 1 {
        reason_codes.push(ReasonCode::DuplicateTimestampDetected);
        return record(
            input,
            TimestampAlignmentV2Status::DuplicateTimestamp,
            false,
            None,
            None,
            None,
            None,
            reason_codes,
        );
    }
    if let Some((matched_index, matched_timestamp)) = exact_matches.first().copied() {
        return finalize_match(
            input,
            matched_timestamp,
            matched_index,
            TimestampAlignmentV2Status::ExactMatch,
            options,
            reason_codes,
            &timestamps,
        );
    }

    if options.allow_timestamp_tolerance {
        let mut candidates = timestamps
            .iter()
            .enumerate()
            .filter_map(|(index, timestamp)| {
                let distance = timestamp.max(&input.scenario_timestamp_ms)
                    - timestamp.min(&input.scenario_timestamp_ms);
                (distance <= options.timestamp_tolerance_ms)
                    .then_some((distance, *timestamp, index))
            })
            .collect::<Vec<_>>();
        candidates.sort();
        if let Some((best_distance, matched_timestamp, matched_index)) = candidates.first().copied()
        {
            if candidates
                .iter()
                .filter(|(distance, _, _)| *distance == best_distance)
                .count()
                > 1
            {
                reason_codes.push(ReasonCode::DuplicateTimestampDetected);
                return record(
                    input,
                    TimestampAlignmentV2Status::DuplicateTimestamp,
                    false,
                    None,
                    None,
                    None,
                    None,
                    reason_codes,
                );
            }
            return finalize_match(
                input,
                matched_timestamp,
                matched_index,
                TimestampAlignmentV2Status::ToleranceMatch,
                options,
                reason_codes,
                &timestamps,
            );
        }
    }

    if options.allow_session_daily_match {
        let scenario_day = input.scenario_timestamp_ms / 86_400_000;
        let daily_matches = timestamps
            .iter()
            .enumerate()
            .filter_map(|(index, timestamp)| {
                ((*timestamp / 86_400_000) == scenario_day).then_some((index, *timestamp))
            })
            .collect::<Vec<_>>();
        if daily_matches.len() == 1 {
            let (matched_index, matched_timestamp) = daily_matches[0];
            return finalize_match(
                input,
                matched_timestamp,
                matched_index,
                TimestampAlignmentV2Status::SessionDailyMatch,
                options,
                reason_codes,
                &timestamps,
            );
        }
    }

    if let Some(position) = timestamps
        .iter()
        .position(|timestamp| *timestamp > input.scenario_timestamp_ms)
    {
        if position > 0 {
            let previous = timestamps[position - 1];
            let next = timestamps[position];
            if let Some(step) = min_step {
                if input.scenario_timestamp_ms.saturating_sub(previous) > step {
                    reason_codes.push(ReasonCode::GapDetected);
                    return record(
                        input,
                        TimestampAlignmentV2Status::GapBeforeTimestamp,
                        false,
                        None,
                        None,
                        None,
                        None,
                        reason_codes,
                    );
                }
                if next.saturating_sub(input.scenario_timestamp_ms) > step {
                    reason_codes.push(ReasonCode::GapDetected);
                    return record(
                        input,
                        TimestampAlignmentV2Status::GapAfterTimestamp,
                        false,
                        None,
                        None,
                        None,
                        None,
                        reason_codes,
                    );
                }
            }
        }
    }

    reason_codes.push(ReasonCode::StaleTimestamp);
    record(
        input,
        TimestampAlignmentV2Status::MissingTimestamp,
        false,
        None,
        None,
        None,
        None,
        reason_codes,
    )
}

fn finalize_match(
    input: &TimestampAlignmentV2Input,
    matched_timestamp: u64,
    matched_index: usize,
    status: TimestampAlignmentV2Status,
    options: &TimestampAlignmentV2Options,
    mut reason_codes: Vec<ReasonCode>,
    timestamps: &[u64],
) -> TimestampAlignmentV2Record {
    let no_lookahead_safe = matched_timestamp <= input.scenario_timestamp_ms;
    if options.require_no_lookahead_safe && !no_lookahead_safe {
        reason_codes.push(ReasonCode::RejectedNoLookaheadReference);
        return record(
            input,
            TimestampAlignmentV2Status::RejectedNoLookahead,
            false,
            Some(matched_timestamp),
            Some(matched_index),
            None,
            None,
            reason_codes,
        );
    }
    let future_window_start_index = matched_index.saturating_add(1);
    let future_window_end_index = matched_index.saturating_add(input.horizon_bars);
    if input.horizon_bars == 0 || future_window_end_index >= timestamps.len() {
        reason_codes.push(ReasonCode::InsufficientBars);
        return record(
            input,
            TimestampAlignmentV2Status::InsufficientFutureWindow,
            no_lookahead_safe,
            Some(matched_timestamp),
            Some(matched_index),
            Some(future_window_start_index.min(timestamps.len().saturating_sub(1))),
            Some(future_window_end_index.min(timestamps.len().saturating_sub(1))),
            reason_codes,
        );
    }
    record(
        input,
        status,
        no_lookahead_safe,
        Some(matched_timestamp),
        Some(matched_index),
        Some(future_window_start_index),
        Some(future_window_end_index),
        reason_codes,
    )
}

#[allow(clippy::too_many_arguments)]
fn record(
    input: &TimestampAlignmentV2Input,
    status: TimestampAlignmentV2Status,
    no_lookahead_safe: bool,
    matched_candle_timestamp_ms: Option<u64>,
    matched_index: Option<usize>,
    future_window_start_index: Option<usize>,
    future_window_end_index: Option<usize>,
    reason_codes: Vec<ReasonCode>,
) -> TimestampAlignmentV2Record {
    TimestampAlignmentV2Record {
        scenario_row_id: input.scenario_row_id.clone(),
        candle_series_id: input.candle_series_id.clone(),
        scenario_timestamp_ms: input.scenario_timestamp_ms,
        matched_candle_timestamp_ms,
        matched_index,
        future_window_start_index,
        future_window_end_index,
        horizon_bars: input.horizon_bars,
        status,
        no_lookahead_safe,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn determine_alignment_status(
    records: &[TimestampAlignmentV2Record],
) -> TimestampAlignmentV2OverallStatus {
    if records
        .iter()
        .any(|record| record.status == TimestampAlignmentV2Status::BadDataQuality)
    {
        return TimestampAlignmentV2OverallStatus::BadDataQuality;
    }
    if records
        .iter()
        .any(|record| record.status == TimestampAlignmentV2Status::DuplicateTimestamp)
    {
        return TimestampAlignmentV2OverallStatus::DuplicateTimestampsDetected;
    }
    if records
        .iter()
        .any(|record| record.status == TimestampAlignmentV2Status::RejectedNoLookahead)
    {
        return TimestampAlignmentV2OverallStatus::NoLookaheadBlocked;
    }
    if records
        .iter()
        .any(|record| record.status == TimestampAlignmentV2Status::InsufficientFutureWindow)
    {
        return TimestampAlignmentV2OverallStatus::NeedLongerCandleCoverage;
    }
    if records.iter().any(|record| {
        matches!(
            record.status,
            TimestampAlignmentV2Status::GapBeforeTimestamp
                | TimestampAlignmentV2Status::GapAfterTimestamp
        )
    }) {
        return TimestampAlignmentV2OverallStatus::GapHeavy;
    }
    if records
        .iter()
        .all(|record| record.status == TimestampAlignmentV2Status::ExactMatch)
    {
        return TimestampAlignmentV2OverallStatus::HealthyTimestampAlignment;
    }
    TimestampAlignmentV2OverallStatus::NeedBetterTimestampAlignment
}

pub fn count_file_bytes(path: &str) -> usize {
    fs::metadata(Path::new(path))
        .map(|metadata| metadata.len() as usize)
        .unwrap_or_default()
}
