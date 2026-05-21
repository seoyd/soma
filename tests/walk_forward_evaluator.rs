use soma_zero::{
    Candle, CandleSeries, GovernorConfig, ReasonCode, Regime, Timeframe, WalkForwardConfig,
    WalkForwardEvaluator,
};

fn trend_series(len: usize) -> CandleSeries {
    CandleSeries {
        symbol: "EVAL".to_string(),
        timeframe: Timeframe::FiveMinute,
        candles: (0..len)
            .map(|i| {
                let base = 100.0 + i as f64 * 0.35;
                Candle {
                    timestamp_ms: i as u64 * 300_000,
                    open: base,
                    high: base + 0.8,
                    low: base - 0.3,
                    close: base + 0.4,
                    volume: 1_200.0 + i as f64 * 18.0,
                    trade_value: Some((base + 0.4) * 1_200.0),
                    bid: Some(base + 0.38),
                    ask: Some(base + 0.42),
                    spread_bps: Some(2.0),
                }
            })
            .collect(),
    }
}

fn low_quality_series(len: usize) -> CandleSeries {
    let mut series = trend_series(len);
    for candle in &mut series.candles[20..35] {
        candle.volume = 0.0;
        candle.bid = None;
        candle.ask = None;
        candle.spread_bps = Some(40.0);
    }
    series
}

fn panic_series(len: usize) -> CandleSeries {
    let mut series = trend_series(len);
    for (offset, candle) in series.candles[45..55].iter_mut().enumerate() {
        let price = 118.0 - offset as f64 * 5.0;
        candle.open = price;
        candle.high = price + 0.8;
        candle.low = price - 9.0;
        candle.close = price - 8.0;
        candle.volume = 5_000.0 + offset as f64 * 1_000.0;
    }
    series
}

#[test]
fn evaluator_produces_fold_reports_and_is_deterministic() {
    let evaluator = WalkForwardEvaluator::default();
    let config = WalkForwardConfig {
        train_window_bars: 40,
        validation_window_bars: Some(8),
        test_window_bars: 20,
        step_bars: 18,
        embargo_bars: 4,
        min_train_bars: 20,
        max_folds: Some(2),
        allow_partial_last_fold: false,
    };

    let left = evaluator.evaluate(&trend_series(120), config);
    let right = evaluator.evaluate(&trend_series(120), config);

    assert_eq!(left, right);
    assert_eq!(left.folds.len(), 2);
    assert!(
        left.reason_codes
            .contains(&ReasonCode::FeatureSchemaValidated)
    );
    assert!(
        left.reason_codes
            .contains(&ReasonCode::WalkForwardEvaluated)
    );
}

#[test]
fn evaluator_preserves_risk_veto_and_counts_no_trade_on_low_quality_data() {
    let mut evaluator = WalkForwardEvaluator::default();
    evaluator.governor.config = GovernorConfig {
        min_data_quality: 0.9,
        ..GovernorConfig::default()
    };
    let report = evaluator.evaluate(
        &low_quality_series(120),
        WalkForwardConfig {
            max_folds: Some(1),
            ..WalkForwardConfig::default()
        },
    );

    assert_eq!(report.folds.len(), 1);
    assert!(
        report.folds[0].test_decision_metrics.denied_by_risk > 0
            || report.folds[0].test_decision_metrics.no_trade > 0
    );
}

#[test]
fn evaluator_regime_split_can_surface_panic() {
    let report = WalkForwardEvaluator::default().evaluate(
        &panic_series(120),
        WalkForwardConfig {
            max_folds: Some(1),
            ..WalkForwardConfig::default()
        },
    );

    assert!(
        report.folds[0]
            .regime_metrics
            .iter()
            .any(|metrics| metrics.regime == Regime::Panic)
    );
}
