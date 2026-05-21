mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    CandleAlignmentRecord, CandleAlignmentStatus, CommitteeTripleBarrierLabel,
    TripleBarrierReferenceBuilder, TripleBarrierReferenceConfig, TripleBarrierTieBreakPolicy,
    load_local_candle_series_map,
};

fn alignment(no_lookahead_safe: bool) -> CandleAlignmentRecord {
    CandleAlignmentRecord {
        scenario_row_id: "row-0".to_string(),
        symbol: "AAPL".to_string(),
        timestamp_ms: 1_700_000_000_000,
        horizon_bars: 2,
        candle_series_id: Some("AAPL".to_string()),
        matched_start_index: Some(0),
        matched_end_index: Some(0),
        future_window_start_index: Some(1),
        future_window_end_index: Some(2),
        status: CandleAlignmentStatus::MatchedExact,
        no_lookahead_safe,
        reason_codes: vec![],
    }
}

fn write_series(name: &str, candles: serde_json::Value) -> soma_zero::CandleSeries {
    let path = common::output_dir(name).join("candles.json");
    official_committee_support::write_json(
        &path,
        serde_json::json!({"symbol":"AAPL","timeframe":"OneDay","candles":candles}),
    );
    load_local_candle_series_map(&[path.display().to_string()])
        .expect("series")
        .remove("AAPL")
        .expect("AAPL")
}

#[test]
fn triple_barrier_builder_generates_take_profit_stop_loss_time_expired_and_is_deterministic() {
    let row = official_committee_support::scenario_row("tb", 0, "AAPL", 1_700_000_000_000);
    let take_series = write_series(
        "tb-take",
        serde_json::json!([
            {"timestamp_ms":1700000000000u64,"open":100.0,"high":100.5,"low":99.5,"close":100.0,"volume":1000.0,"spread_bps":4.0},
            {"timestamp_ms":1700000000001u64,"open":100.0,"high":103.0,"low":99.9,"close":102.5,"volume":1000.0,"spread_bps":4.0},
            {"timestamp_ms":1700000000002u64,"open":102.5,"high":103.0,"low":101.0,"close":102.0,"volume":1000.0,"spread_bps":4.0}
        ]),
    );
    let config = TripleBarrierReferenceConfig {
        horizon_bars: 2,
        ..TripleBarrierReferenceConfig::default()
    };
    let first = TripleBarrierReferenceBuilder::default()
        .build(&row, &alignment(true), &take_series, &config, false)
        .expect("build");
    let second = TripleBarrierReferenceBuilder::default()
        .build(&row, &alignment(true), &take_series, &config, false)
        .expect("build");
    assert_eq!(first, second);
    assert_eq!(
        first.reference.triple_barrier_label,
        CommitteeTripleBarrierLabel::TakeProfit
    );
    assert!(first.reference.net_return_pct.expect("return") < 0.03);

    let stop_series = write_series(
        "tb-stop",
        serde_json::json!([
            {"timestamp_ms":1700000000000u64,"open":100.0,"high":100.5,"low":99.5,"close":100.0,"volume":1000.0,"spread_bps":4.0},
            {"timestamp_ms":1700000000001u64,"open":100.0,"high":100.2,"low":98.0,"close":98.5,"volume":1000.0,"spread_bps":4.0},
            {"timestamp_ms":1700000000002u64,"open":98.5,"high":99.0,"low":98.0,"close":98.6,"volume":1000.0,"spread_bps":4.0}
        ]),
    );
    let stop = TripleBarrierReferenceBuilder::default()
        .build(&row, &alignment(true), &stop_series, &config, false)
        .expect("build");
    assert_eq!(
        stop.reference.triple_barrier_label,
        CommitteeTripleBarrierLabel::StopLoss
    );

    let time_series = write_series(
        "tb-time",
        serde_json::json!([
            {"timestamp_ms":1700000000000u64,"open":100.0,"high":100.5,"low":99.5,"close":100.0,"volume":1000.0,"spread_bps":4.0},
            {"timestamp_ms":1700000000001u64,"open":100.0,"high":100.8,"low":99.5,"close":100.4,"volume":1000.0,"spread_bps":4.0},
            {"timestamp_ms":1700000000002u64,"open":100.4,"high":100.9,"low":99.8,"close":100.5,"volume":1000.0,"spread_bps":4.0}
        ]),
    );
    let time = TripleBarrierReferenceBuilder::default()
        .build(&row, &alignment(true), &time_series, &config, false)
        .expect("build");
    assert_eq!(
        time.reference.triple_barrier_label,
        CommitteeTripleBarrierLabel::TimeExpired
    );
}

#[test]
fn triple_barrier_builder_uses_deterministic_tie_break_and_no_lookahead_flag() {
    let row = official_committee_support::scenario_row("tb-tie", 0, "AAPL", 1_700_000_000_000);
    let series = write_series(
        "tb-tie",
        serde_json::json!([
            {"timestamp_ms":1700000000000u64,"open":100.0,"high":100.5,"low":99.5,"close":100.0,"volume":1000.0,"spread_bps":4.0},
            {"timestamp_ms":1700000000001u64,"open":100.0,"high":103.0,"low":98.0,"close":101.0,"volume":1000.0,"spread_bps":4.0},
            {"timestamp_ms":1700000000002u64,"open":101.0,"high":101.5,"low":100.0,"close":101.0,"volume":1000.0,"spread_bps":4.0}
        ]),
    );
    let stop_first = TripleBarrierReferenceBuilder::default()
        .build(
            &row,
            &alignment(true),
            &series,
            &TripleBarrierReferenceConfig {
                horizon_bars: 2,
                tie_break_policy: TripleBarrierTieBreakPolicy::StopFirst,
                ..TripleBarrierReferenceConfig::default()
            },
            false,
        )
        .expect("build");
    assert_eq!(
        stop_first.reference.triple_barrier_label,
        CommitteeTripleBarrierLabel::StopLoss
    );

    let take_first = TripleBarrierReferenceBuilder::default()
        .build(
            &row,
            &alignment(true),
            &series,
            &TripleBarrierReferenceConfig {
                horizon_bars: 2,
                tie_break_policy: TripleBarrierTieBreakPolicy::TakeProfitFirst,
                ..TripleBarrierReferenceConfig::default()
            },
            false,
        )
        .expect("build");
    assert_eq!(
        take_first.reference.triple_barrier_label,
        CommitteeTripleBarrierLabel::TakeProfit
    );

    let unsafe_build = TripleBarrierReferenceBuilder::default()
        .build(
            &row,
            &alignment(false),
            &series,
            &TripleBarrierReferenceConfig {
                horizon_bars: 2,
                ..TripleBarrierReferenceConfig::default()
            },
            false,
        )
        .expect("build");
    assert!(!unsafe_build.reference.no_lookahead_safe);
}
