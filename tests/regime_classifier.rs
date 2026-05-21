use soma_zero::{Candle, CandleSeries, FeatureEngine, Regime, RegimeClassifier, Timeframe};

fn make_series(closes: &[f64], volumes: &[f64]) -> CandleSeries {
    let candles = closes
        .iter()
        .enumerate()
        .map(|(i, close)| Candle {
            timestamp_ms: 60_000 * (i as u64 + 1),
            open: *close - 0.2,
            high: *close + 0.6,
            low: *close - 0.6,
            close: *close,
            volume: volumes[i],
            trade_value: Some(*close * volumes[i]),
            bid: Some(*close - 0.01),
            ask: Some(*close + 0.01),
            spread_bps: None,
        })
        .collect();
    CandleSeries {
        symbol: "REG".to_string(),
        timeframe: Timeframe::OneMinute,
        candles,
    }
}

fn classify(series: &CandleSeries) -> Regime {
    let engine = FeatureEngine::default();
    let classifier = RegimeClassifier::default();
    let index = series.candles.len() - 1;
    let features = engine.build_at(series, index);
    classifier
        .classify(&features, series.lookback_window(index, 20).unwrap())
        .regime
}

#[test]
fn trend_up_series_classifies_as_trend_up_or_risk_on() {
    let closes: Vec<f64> = (0..30).map(|i| 100.0 + i as f64 * 0.5).collect();
    let volumes: Vec<f64> = (0..30).map(|i| 1_000.0 + i as f64 * 25.0).collect();
    let regime = classify(&make_series(&closes, &volumes));
    assert!(matches!(regime, Regime::TrendUp | Regime::RiskOn));
}

#[test]
fn trend_down_series_classifies_as_trend_down_or_risk_off() {
    let closes: Vec<f64> = (0..30).map(|i| 110.0 - i as f64 * 0.5).collect();
    let volumes: Vec<f64> = (0..30).map(|i| 1_000.0 + i as f64 * 20.0).collect();
    let regime = classify(&make_series(&closes, &volumes));
    assert!(matches!(regime, Regime::TrendDown | Regime::RiskOff));
}

#[test]
fn range_series_classifies_as_range() {
    let closes: Vec<f64> = (0..30)
        .map(|i| if i % 2 == 0 { 100.2 } else { 99.8 })
        .collect();
    let volumes: Vec<f64> = vec![1_200.0; 30];
    let regime = classify(&make_series(&closes, &volumes));
    assert_eq!(regime, Regime::Range);
}

#[test]
fn panic_series_classifies_as_panic() {
    let mut closes: Vec<f64> = (0..25).map(|i| 105.0 + i as f64 * 0.1).collect();
    closes.extend([103.0, 100.0, 96.0, 92.0, 88.0]);
    let mut volumes: Vec<f64> = vec![1_000.0; 25];
    volumes.extend([2_000.0, 2_500.0, 3_000.0, 4_000.0, 5_000.0]);
    let regime = classify(&make_series(&closes, &volumes));
    assert_eq!(regime, Regime::Panic);
}

#[test]
fn high_volatility_series_classifies_as_high_volatility() {
    let closes: Vec<f64> = (0..30)
        .map(|i| if i % 2 == 0 { 101.8 } else { 98.2 })
        .collect();
    let volumes: Vec<f64> = vec![1_500.0; 30];
    let regime = classify(&make_series(&closes, &volumes));
    assert_eq!(regime, Regime::HighVolatility);
}

#[test]
fn insufficient_data_classifies_as_unknown() {
    let closes: Vec<f64> = vec![100.0, 100.1, 100.2];
    let volumes: Vec<f64> = vec![1_000.0; 3];
    let regime = classify(&make_series(&closes, &volumes));
    assert_eq!(regime, Regime::Unknown);
}

#[test]
fn precedence_is_deterministic_when_multiple_conditions_apply() {
    let mut closes: Vec<f64> = (0..25).map(|i| 110.0 - i as f64 * 0.1).collect();
    closes.extend([104.0, 100.0, 95.0, 90.0, 85.0]);
    let mut volumes: Vec<f64> = vec![1_000.0; 25];
    volumes.extend([1_500.0, 2_000.0, 3_000.0, 4_000.0, 5_000.0]);
    let regime = classify(&make_series(&closes, &volumes));
    assert_eq!(regime, Regime::Panic);
}
