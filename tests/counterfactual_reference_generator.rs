mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    CandleAlignmentRecord, CandleAlignmentStatus, CommitteeCounterfactualType,
    CounterfactualBuildStatus, CounterfactualReferenceGenerator, CounterfactualReferencePolicy,
    TripleBarrierReferenceConfig, load_local_candle_series_map,
};

fn alignment() -> CandleAlignmentRecord {
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
        no_lookahead_safe: true,
        reason_codes: vec![],
    }
}

#[test]
fn counterfactual_generator_builds_no_trade_and_risk_denied_records() {
    let row =
        official_committee_support::scenario_row("counterfactual", 0, "AAPL", 1_700_000_000_000);
    let series_path = official_committee_support::write_json(
        &common::output_dir("counterfactual-generator").join("candles.json"),
        serde_json::json!({
            "symbol":"AAPL",
            "timeframe":"OneDay",
            "candles":[
                {"timestamp_ms":1700000000000u64,"open":100.0,"high":100.5,"low":99.5,"close":100.0,"volume":1000.0,"spread_bps":4.0},
                {"timestamp_ms":1700000000001u64,"open":100.0,"high":99.9,"low":98.0,"close":98.5,"volume":1000.0,"spread_bps":4.0},
                {"timestamp_ms":1700000000002u64,"open":98.5,"high":99.0,"low":98.0,"close":98.6,"volume":1000.0,"spread_bps":4.0}
            ]
        }),
    );
    let series =
        load_local_candle_series_map(&[series_path.display().to_string()]).expect("series");
    let no_trade = CounterfactualReferenceGenerator::default().generate_no_trade(
        &row,
        &alignment(),
        series.get("AAPL").expect("AAPL"),
        &TripleBarrierReferenceConfig {
            horizon_bars: 2,
            ..TripleBarrierReferenceConfig::default()
        },
        &CounterfactualReferencePolicy::default(),
        false,
    );
    assert_eq!(
        no_trade.counterfactual_type,
        CommitteeCounterfactualType::NoTrade
    );
    assert_eq!(no_trade.build_status, CounterfactualBuildStatus::Built);
    assert!(no_trade.avoided_loss_value.expect("avoided") > 0.0);

    let risk_denied = CounterfactualReferenceGenerator::default().generate_risk_denied(
        &row,
        &alignment(),
        series.get("AAPL").expect("AAPL"),
        &TripleBarrierReferenceConfig {
            horizon_bars: 2,
            ..TripleBarrierReferenceConfig::default()
        },
        &CounterfactualReferencePolicy::default(),
        true,
    );
    assert_eq!(
        risk_denied.counterfactual_type,
        CommitteeCounterfactualType::RiskDenied
    );
    assert!(risk_denied.diagnostic_only);
    assert!(
        risk_denied
            .reason_codes
            .iter()
            .any(|code| format!("{:?}", code) == "RiskDenied")
    );
}
