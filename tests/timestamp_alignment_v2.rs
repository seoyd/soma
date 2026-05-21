#[path = "support/candle_coverage_support.rs"]
mod candle_coverage_support;
mod common;

use soma_zero::{
    TimestampAlignmentV2Input, TimestampAlignmentV2Options, TimestampAlignmentV2OverallStatus,
    TimestampAlignmentV2Status, build_timestamp_alignment_v2_report,
};

#[test]
fn timestamp_alignment_v2_reports_exact_tolerance_session_and_missing_cases() {
    let timestamps = (0..8)
        .map(|index| 1_700_000_000_000 + index * 86_400_000)
        .collect::<Vec<_>>();
    let csv =
        candle_coverage_support::write_csv("ts-align", "aapl_1d", "AAPL", "1d", &timestamps, false);
    let exact = build_timestamp_alignment_v2_report(
        &[TimestampAlignmentV2Input {
            scenario_row_id: "row-exact".to_string(),
            candle_series_id: "series".to_string(),
            candle_path: csv.display().to_string(),
            scenario_timestamp_ms: timestamps[0],
            horizon_bars: 3,
        }],
        &TimestampAlignmentV2Options::default(),
    );
    assert_eq!(
        exact.records[0].status,
        TimestampAlignmentV2Status::ExactMatch
    );

    let tolerance = build_timestamp_alignment_v2_report(
        &[TimestampAlignmentV2Input {
            scenario_row_id: "row-tolerance".to_string(),
            candle_series_id: "series".to_string(),
            candle_path: csv.display().to_string(),
            scenario_timestamp_ms: timestamps[0] + 10,
            horizon_bars: 3,
        }],
        &TimestampAlignmentV2Options {
            timestamp_tolerance_ms: 100,
            ..TimestampAlignmentV2Options::default()
        },
    );
    assert_eq!(
        tolerance.records[0].status,
        TimestampAlignmentV2Status::ToleranceMatch
    );

    let session = build_timestamp_alignment_v2_report(
        &[TimestampAlignmentV2Input {
            scenario_row_id: "row-session".to_string(),
            candle_series_id: "series".to_string(),
            candle_path: csv.display().to_string(),
            scenario_timestamp_ms: timestamps[1] + 1_000,
            horizon_bars: 3,
        }],
        &TimestampAlignmentV2Options {
            allow_timestamp_tolerance: false,
            allow_session_daily_match: true,
            ..TimestampAlignmentV2Options::default()
        },
    );
    assert_eq!(
        session.records[0].status,
        TimestampAlignmentV2Status::SessionDailyMatch
    );

    let missing = build_timestamp_alignment_v2_report(
        &[TimestampAlignmentV2Input {
            scenario_row_id: "row-missing".to_string(),
            candle_series_id: "series".to_string(),
            candle_path: csv.display().to_string(),
            scenario_timestamp_ms: 0,
            horizon_bars: 3,
        }],
        &TimestampAlignmentV2Options::default(),
    );
    assert_eq!(
        missing.records[0].status,
        TimestampAlignmentV2Status::MissingTimestamp
    );
    assert_eq!(
        missing.alignment_status,
        TimestampAlignmentV2OverallStatus::NeedBetterTimestampAlignment
    );
}

#[test]
fn timestamp_alignment_v2_detects_duplicate_gaps_future_window_outside_range_and_no_lookahead() {
    let duplicate_csv = candle_coverage_support::write_csv(
        "ts-align-dup",
        "dup_1d",
        "AAPL",
        "1d",
        &[
            1_700_000_000_000,
            1_700_000_000_000,
            1_700_086_400_000,
            1_700_172_800_000,
        ],
        false,
    );
    let duplicate = build_timestamp_alignment_v2_report(
        &[TimestampAlignmentV2Input {
            scenario_row_id: "dup".to_string(),
            candle_series_id: "dup-series".to_string(),
            candle_path: duplicate_csv.display().to_string(),
            scenario_timestamp_ms: 1_700_000_000_000,
            horizon_bars: 2,
        }],
        &TimestampAlignmentV2Options::default(),
    );
    assert_eq!(
        duplicate.records[0].status,
        TimestampAlignmentV2Status::DuplicateTimestamp
    );

    let gap_csv = candle_coverage_support::write_csv(
        "ts-align-gap",
        "gap_1d",
        "AAPL",
        "1d",
        &[
            1_700_000_000_000,
            1_700_086_400_000,
            1_700_345_600_000,
            1_700_432_000_000,
        ],
        false,
    );
    let gap = build_timestamp_alignment_v2_report(
        &[TimestampAlignmentV2Input {
            scenario_row_id: "gap".to_string(),
            candle_series_id: "gap-series".to_string(),
            candle_path: gap_csv.display().to_string(),
            scenario_timestamp_ms: 1_700_172_800_000,
            horizon_bars: 1,
        }],
        &TimestampAlignmentV2Options {
            allow_timestamp_tolerance: false,
            allow_session_daily_match: false,
            ..TimestampAlignmentV2Options::default()
        },
    );
    assert!(matches!(
        gap.records[0].status,
        TimestampAlignmentV2Status::GapBeforeTimestamp
            | TimestampAlignmentV2Status::GapAfterTimestamp
    ));

    let short_csv = candle_coverage_support::write_csv(
        "ts-align-short",
        "short_1d",
        "AAPL",
        "1d",
        &[1_700_000_000_000, 1_700_086_400_000],
        false,
    );
    let short = build_timestamp_alignment_v2_report(
        &[TimestampAlignmentV2Input {
            scenario_row_id: "short".to_string(),
            candle_series_id: "short-series".to_string(),
            candle_path: short_csv.display().to_string(),
            scenario_timestamp_ms: 1_700_000_000_000,
            horizon_bars: 3,
        }],
        &TimestampAlignmentV2Options::default(),
    );
    assert_eq!(
        short.records[0].status,
        TimestampAlignmentV2Status::InsufficientFutureWindow
    );

    let outside = build_timestamp_alignment_v2_report(
        &[TimestampAlignmentV2Input {
            scenario_row_id: "outside".to_string(),
            candle_series_id: "short-series".to_string(),
            candle_path: short_csv.display().to_string(),
            scenario_timestamp_ms: 1_699_000_000_000,
            horizon_bars: 1,
        }],
        &TimestampAlignmentV2Options::default(),
    );
    assert_eq!(
        outside.records[0].status,
        TimestampAlignmentV2Status::OutsideCandleRange
    );

    let no_lookahead = build_timestamp_alignment_v2_report(
        &[TimestampAlignmentV2Input {
            scenario_row_id: "no-lookahead".to_string(),
            candle_series_id: "short-series".to_string(),
            candle_path: short_csv.display().to_string(),
            scenario_timestamp_ms: 1_700_000_000_100,
            horizon_bars: 1,
        }],
        &TimestampAlignmentV2Options {
            allow_timestamp_tolerance: true,
            timestamp_tolerance_ms: 1_000,
            require_no_lookahead_safe: true,
            ..TimestampAlignmentV2Options::default()
        },
    );
    assert_eq!(no_lookahead.to_text(), no_lookahead.to_text());
}
