use soma_zero::{
    BaselineSignalModel, Candle, CandleSeries, CostModel, FeatureEngine, Regime, RegimeDecision,
    Timeframe,
};

fn make_series(closes: &[f64], volumes: &[f64], spread_bps: f64) -> CandleSeries {
    let candles = closes
        .iter()
        .enumerate()
        .map(|(i, close)| Candle {
            timestamp_ms: 60_000 * (i as u64 + 1),
            open: *close - 0.2,
            high: *close + 0.5,
            low: *close - 0.5,
            close: *close,
            volume: volumes[i],
            trade_value: Some(*close * volumes[i]),
            bid: Some(*close * (1.0 - spread_bps / 20_000.0)),
            ask: Some(*close * (1.0 + spread_bps / 20_000.0)),
            spread_bps: Some(spread_bps),
        })
        .collect();
    CandleSeries {
        symbol: "SIG".to_string(),
        timeframe: Timeframe::OneMinute,
        candles,
    }
}

fn features_for(series: &CandleSeries) -> soma_zero::FeatureVector {
    FeatureEngine::default().build_at(series, series.candles.len() - 1)
}

fn cost() -> CostModel {
    CostModel {
        fee_bps: 2.0,
        slippage_bps: 2.0,
        spread_bps: Some(2.0),
        min_cost_bps: None,
    }
}

#[test]
fn baseline_signal_keeps_high_no_trade_for_unknown_or_panic_regimes() {
    let series = make_series(&vec![100.0; 25], &vec![1_000.0; 25], 2.0);
    let model = BaselineSignalModel::default();
    let unknown = model.evaluate(
        &features_for(&series),
        &RegimeDecision {
            regime: Regime::Unknown,
            regime_confidence: 0.2,
            reason_codes: vec![],
        },
        &cost(),
    );
    let panic = model.evaluate(
        &features_for(&series),
        &RegimeDecision {
            regime: Regime::Panic,
            regime_confidence: 0.9,
            reason_codes: vec![],
        },
        &cost(),
    );
    assert!(unknown.no_trade_probability >= 0.7);
    assert!(panic.no_trade_probability >= 0.75);
}

#[test]
fn baseline_signal_prefers_bullish_confirmation_over_neutral_case() {
    let bullish_closes: Vec<f64> = (0..30).map(|i| 100.0 + i as f64 * 0.4).collect();
    let bullish_volumes: Vec<f64> = (0..29).map(|_| 1_000.0).chain([8_000.0]).collect();
    let neutral_closes: Vec<f64> = vec![100.0; 30];
    let neutral_volumes: Vec<f64> = vec![1_000.0; 30];
    let model = BaselineSignalModel::default();
    let bullish = model.evaluate(
        &features_for(&make_series(&bullish_closes, &bullish_volumes, 2.0)),
        &RegimeDecision {
            regime: Regime::TrendUp,
            regime_confidence: 0.8,
            reason_codes: vec![],
        },
        &cost(),
    );
    let neutral = model.evaluate(
        &features_for(&make_series(&neutral_closes, &neutral_volumes, 2.0)),
        &RegimeDecision {
            regime: Regime::Range,
            regime_confidence: 0.6,
            reason_codes: vec![],
        },
        &cost(),
    );
    assert!(bullish.p_win > neutral.p_win);
}

#[test]
fn baseline_signal_is_deterministic_and_penalizes_poor_quality() {
    let mut bad_series = make_series(&vec![100.0; 30], &vec![1_000.0; 30], 30.0);
    bad_series.candles[29].close = -1.0;
    let model = BaselineSignalModel::default();
    let bad = model.evaluate(
        &features_for(&bad_series),
        &RegimeDecision {
            regime: Regime::TrendUp,
            regime_confidence: 0.7,
            reason_codes: vec![],
        },
        &cost(),
    );
    let good = model.evaluate(
        &features_for(&make_series(&vec![100.0; 30], &vec![1_000.0; 30], 2.0)),
        &RegimeDecision {
            regime: Regime::TrendUp,
            regime_confidence: 0.7,
            reason_codes: vec![],
        },
        &cost(),
    );
    assert!(
        bad.no_trade_probability >= good.no_trade_probability || bad.confidence <= good.confidence
    );
    assert_eq!(
        model.evaluate(
            &features_for(&make_series(&vec![100.0; 30], &vec![1_000.0; 30], 2.0)),
            &RegimeDecision {
                regime: Regime::Range,
                regime_confidence: 0.5,
                reason_codes: vec![]
            },
            &cost()
        ),
        model.evaluate(
            &features_for(&make_series(&vec![100.0; 30], &vec![1_000.0; 30], 2.0)),
            &RegimeDecision {
                regime: Regime::Range,
                regime_confidence: 0.5,
                reason_codes: vec![]
            },
            &cost()
        )
    );
}
