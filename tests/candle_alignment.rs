mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use std::collections::BTreeMap;

use soma_zero::{
    CandleAligner, CandleAlignmentOverallStatus, CandleAlignmentStatus,
    CommitteeReferencePackConfig, PersonaHorizon, load_local_candle_series_map,
};

fn write_series(
    name: &str,
    symbol: &str,
    timestamps: &[u64],
    high: f64,
    low: f64,
) -> std::path::PathBuf {
    let path = common::output_dir(name).join("candles.json");
    let candles = timestamps
        .iter()
        .enumerate()
        .map(|(index, timestamp)| {
            serde_json::json!({
                "timestamp_ms": timestamp,
                "open": 100.0 + index as f64,
                "high": high,
                "low": low,
                "close": 100.5 + index as f64,
                "volume": 1000.0,
                "spread_bps": 4.0
            })
        })
        .collect::<Vec<_>>();
    official_committee_support::write_json(
        &path,
        serde_json::json!({"symbol": symbol, "timeframe": "OneDay", "candles": candles}),
    )
}

#[test]
fn candle_alignment_handles_exact_tolerance_and_rejections_deterministically() {
    let exact_row = official_committee_support::scenario_row("align", 0, "AAPL", 1_700_000_000_000);
    let mut wrong_horizon_row =
        official_committee_support::scenario_row("align", 1, "AAPL", 1_700_000_000_000);
    wrong_horizon_row.target_horizon = PersonaHorizon::Intraday;
    let exact_timestamps = (0..30)
        .map(|offset| 1_700_000_000_000 + offset)
        .collect::<Vec<_>>();
    let exact_series = load_local_candle_series_map(&[write_series(
        "align-exact",
        "AAPL",
        &exact_timestamps,
        102.0,
        99.0,
    )
    .display()
    .to_string()])
    .expect("series");
    let config = CommitteeReferencePackConfig::default();
    let exact = CandleAligner::default().align_rows(&[exact_row.clone()], &exact_series, &config);
    assert_eq!(exact.records[0].status, CandleAlignmentStatus::MatchedExact);
    assert_eq!(
        exact.alignment_status,
        CandleAlignmentOverallStatus::HealthyAlignment
    );

    let mut tolerance_config = CommitteeReferencePackConfig::default();
    tolerance_config.timestamp_tolerance_ms = 10;
    let tolerance_timestamps = (0..30)
        .map(|offset| 1_700_000_000_005 + offset)
        .collect::<Vec<_>>();
    let tolerance_series = load_local_candle_series_map(&[write_series(
        "align-tolerance",
        "AAPL",
        &tolerance_timestamps,
        102.0,
        99.0,
    )
    .display()
    .to_string()])
    .expect("series");
    let tolerance = CandleAligner::default().align_rows(
        &[exact_row.clone()],
        &tolerance_series,
        &tolerance_config,
    );
    assert_eq!(
        tolerance.records[0].status,
        CandleAlignmentStatus::RejectedNoLookahead
    );

    let wrong_symbol =
        CandleAligner::default().align_rows(&[exact_row.clone()], &BTreeMap::new(), &config);
    assert_eq!(
        wrong_symbol.records[0].status,
        CandleAlignmentStatus::MissingCandleSeries
    );
    assert!(
        wrong_symbol.records[0]
            .reason_codes
            .iter()
            .any(|code| format!("{:?}", code) == "MissingRealLocalData")
    );

    let wrong_horizon =
        CandleAligner::default().align_rows(&[wrong_horizon_row], &exact_series, &config);
    assert_eq!(
        wrong_horizon.records[0].status,
        CandleAlignmentStatus::WrongHorizon
    );

    let repeat = CandleAligner::default().align_rows(&[exact_row], &exact_series, &config);
    assert_eq!(exact, repeat);
}

#[test]
fn candle_alignment_detects_missing_timestamp_gap_duplicate_and_short_windows() {
    let row = official_committee_support::scenario_row("align-edge", 0, "AAPL", 1_700_000_000_000);
    let config = CommitteeReferencePackConfig::default();

    let missing_timestamps = (0..30)
        .map(|offset| 1_700_000_000_100 + offset)
        .collect::<Vec<_>>();
    let missing_series = load_local_candle_series_map(&[write_series(
        "align-missing-ts",
        "AAPL",
        &missing_timestamps,
        102.0,
        99.0,
    )
    .display()
    .to_string()])
    .expect("series");
    let missing = CandleAligner::default().align_rows(&[row.clone()], &missing_series, &config);
    assert_eq!(
        missing.records[0].status,
        CandleAlignmentStatus::MissingTimestamp
    );

    let mut gap_timestamps = (0..30)
        .map(|offset| 1_700_000_000_000 + offset)
        .collect::<Vec<_>>();
    gap_timestamps[2] = 1_700_000_000_003;
    let gap_series = load_local_candle_series_map(&[write_series(
        "align-gap",
        "AAPL",
        &gap_timestamps,
        102.0,
        99.0,
    )
    .display()
    .to_string()])
    .expect("series");
    let gap = CandleAligner::default().align_rows(&[row.clone()], &gap_series, &config);
    assert_eq!(gap.records[0].status, CandleAlignmentStatus::GapDetected);

    let mut duplicate_timestamps = (0..30)
        .map(|offset| 1_700_000_000_000 + offset)
        .collect::<Vec<_>>();
    duplicate_timestamps[1] = 1_700_000_000_000;
    let duplicate_series = load_local_candle_series_map(&[write_series(
        "align-dup",
        "AAPL",
        &duplicate_timestamps,
        102.0,
        99.0,
    )
    .display()
    .to_string()])
    .expect("series");
    let duplicate = CandleAligner::default().align_rows(&[row.clone()], &duplicate_series, &config);
    assert_eq!(
        duplicate.records[0].status,
        CandleAlignmentStatus::DuplicateTimestamp
    );

    let short_series = load_local_candle_series_map(&[write_series(
        "align-short",
        "AAPL",
        &[1_700_000_000_000, 1_700_000_000_001],
        102.0,
        99.0,
    )
    .display()
    .to_string()])
    .expect("series");
    let short = CandleAligner::default().align_rows(&[row], &short_series, &config);
    assert_eq!(
        short.records[0].status,
        CandleAlignmentStatus::InsufficientFutureBars
    );
}
