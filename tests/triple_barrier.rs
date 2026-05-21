use soma_zero::{
    BarrierHit, Candle, CandleSeries, Side, Timeframe, TripleBarrierConfig, TripleBarrierOutcome,
    evaluate_triple_barrier,
};

fn candle(timestamp_ms: u64, open: f64, high: f64, low: f64, close: f64) -> Candle {
    Candle {
        timestamp_ms,
        open,
        high,
        low,
        close,
        volume: 1_000.0,
        trade_value: Some(close * 1_000.0),
        bid: None,
        ask: None,
        spread_bps: Some(2.0),
    }
}

fn series(candles: Vec<Candle>) -> CandleSeries {
    CandleSeries {
        symbol: "TEST".to_string(),
        timeframe: Timeframe::OneMinute,
        candles,
    }
}

#[test]
fn long_trade_hits_take_profit_first() {
    let result = evaluate_triple_barrier(
        &series(vec![
            candle(1, 100.0, 100.5, 99.5, 100.0),
            candle(2, 100.0, 102.5, 99.7, 102.0),
        ]),
        0,
        100.0,
        TripleBarrierConfig {
            take_profit_pct: 0.02,
            stop_loss_pct: 0.01,
            horizon_bars: 1,
            fee_bps: 1.0,
            slippage_bps: 1.0,
            side: Side::Long,
            use_high_low_intrabar: true,
        },
    );
    assert_eq!(result.outcome, TripleBarrierOutcome::Win);
    assert_eq!(result.first_hit, BarrierHit::TakeProfit);
}

#[test]
fn long_trade_hits_stop_loss_first() {
    let result = evaluate_triple_barrier(
        &series(vec![
            candle(1, 100.0, 100.4, 99.5, 100.0),
            candle(2, 100.0, 100.2, 98.8, 99.0),
        ]),
        0,
        100.0,
        TripleBarrierConfig {
            take_profit_pct: 0.02,
            stop_loss_pct: 0.01,
            horizon_bars: 1,
            fee_bps: 1.0,
            slippage_bps: 1.0,
            side: Side::Long,
            use_high_low_intrabar: true,
        },
    );
    assert_eq!(result.outcome, TripleBarrierOutcome::Loss);
    assert_eq!(result.first_hit, BarrierHit::StopLoss);
}

#[test]
fn short_trade_hits_take_profit_first() {
    let result = evaluate_triple_barrier(
        &series(vec![
            candle(1, 100.0, 100.3, 99.7, 100.0),
            candle(2, 100.0, 100.1, 97.6, 98.0),
        ]),
        0,
        100.0,
        TripleBarrierConfig {
            take_profit_pct: 0.02,
            stop_loss_pct: 0.01,
            horizon_bars: 1,
            fee_bps: 1.0,
            slippage_bps: 1.0,
            side: Side::Short,
            use_high_low_intrabar: true,
        },
    );
    assert_eq!(result.outcome, TripleBarrierOutcome::Win);
    assert_eq!(result.first_hit, BarrierHit::TakeProfit);
}

#[test]
fn short_trade_hits_stop_loss_first() {
    let result = evaluate_triple_barrier(
        &series(vec![
            candle(1, 100.0, 100.3, 99.7, 100.0),
            candle(2, 100.0, 101.2, 99.8, 100.9),
        ]),
        0,
        100.0,
        TripleBarrierConfig {
            take_profit_pct: 0.02,
            stop_loss_pct: 0.01,
            horizon_bars: 1,
            fee_bps: 1.0,
            slippage_bps: 1.0,
            side: Side::Short,
            use_high_low_intrabar: true,
        },
    );
    assert_eq!(result.outcome, TripleBarrierOutcome::Loss);
    assert_eq!(result.first_hit, BarrierHit::StopLoss);
}

#[test]
fn time_barrier_expiry_returns_neutral() {
    let result = evaluate_triple_barrier(
        &series(vec![
            candle(1, 100.0, 100.2, 99.8, 100.0),
            candle(2, 100.0, 100.9, 99.5, 100.4),
            candle(3, 100.4, 100.8, 99.6, 100.3),
        ]),
        0,
        100.0,
        TripleBarrierConfig {
            take_profit_pct: 0.03,
            stop_loss_pct: 0.03,
            horizon_bars: 2,
            fee_bps: 1.0,
            slippage_bps: 1.0,
            side: Side::Long,
            use_high_low_intrabar: true,
        },
    );
    assert_eq!(result.outcome, TripleBarrierOutcome::Neutral);
    assert_eq!(result.first_hit, BarrierHit::TimeExpired);
}

#[test]
fn horizon_exceeding_data_returns_no_data() {
    let result = evaluate_triple_barrier(
        &series(vec![
            candle(1, 100.0, 100.5, 99.5, 100.0),
            candle(2, 100.0, 100.7, 99.8, 100.1),
        ]),
        0,
        100.0,
        TripleBarrierConfig {
            take_profit_pct: 0.02,
            stop_loss_pct: 0.01,
            horizon_bars: 3,
            fee_bps: 1.0,
            slippage_bps: 1.0,
            side: Side::Long,
            use_high_low_intrabar: true,
        },
    );
    assert_eq!(result.outcome, TripleBarrierOutcome::NoData);
    assert_eq!(result.first_hit, BarrierHit::NoData);
}

#[test]
fn same_candle_dual_hit_uses_conservative_loss() {
    let result = evaluate_triple_barrier(
        &series(vec![
            candle(1, 100.0, 100.1, 99.9, 100.0),
            candle(2, 100.0, 102.2, 98.9, 100.5),
        ]),
        0,
        100.0,
        TripleBarrierConfig {
            take_profit_pct: 0.02,
            stop_loss_pct: 0.01,
            horizon_bars: 1,
            fee_bps: 1.0,
            slippage_bps: 1.0,
            side: Side::Long,
            use_high_low_intrabar: true,
        },
    );
    assert_eq!(result.outcome, TripleBarrierOutcome::Loss);
    assert_eq!(result.first_hit, BarrierHit::StopLoss);
}

#[test]
fn net_return_is_lower_than_gross_return_after_costs() {
    let result = evaluate_triple_barrier(
        &series(vec![
            candle(1, 100.0, 100.1, 99.9, 100.0),
            candle(2, 100.0, 102.2, 99.8, 102.0),
        ]),
        0,
        100.0,
        TripleBarrierConfig {
            take_profit_pct: 0.02,
            stop_loss_pct: 0.01,
            horizon_bars: 1,
            fee_bps: 5.0,
            slippage_bps: 5.0,
            side: Side::Long,
            use_high_low_intrabar: true,
        },
    );
    assert!(result.gross_return_pct > result.net_return_pct);
}
