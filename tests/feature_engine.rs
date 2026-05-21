use soma_zero::{Candle, CandleSeries, FeatureEngine, FeatureName, FeatureValue, Timeframe};

fn candle(
    timestamp_ms: u64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    bid: Option<f64>,
    ask: Option<f64>,
) -> Candle {
    Candle {
        timestamp_ms,
        open,
        high,
        low,
        close,
        volume,
        trade_value: Some(close * volume.max(0.0)),
        bid,
        ask,
        spread_bps: None,
    }
}

fn healthy_series() -> CandleSeries {
    let mut candles = Vec::new();
    for i in 0..30 {
        let base = 100.0 + i as f64 * 0.4;
        let volume = if i == 29 {
            8_000.0
        } else {
            1_000.0 + i as f64 * 20.0
        };
        candles.push(candle(
            60_000 * (i as u64 + 1),
            base,
            base + 0.6,
            base - 0.4,
            base + 0.2,
            volume,
            Some(base + 0.18),
            Some(base + 0.22),
        ));
    }
    CandleSeries {
        symbol: "FEAT".to_string(),
        timeframe: Timeframe::OneMinute,
        candles,
    }
}

#[test]
fn feature_vector_has_stable_feature_order() {
    let engine = FeatureEngine::default();
    let names = engine.feature_names();
    assert_eq!(names[0], FeatureName::Close);
    assert_eq!(names[1], FeatureName::LogReturn1);
    assert!(names.contains(&FeatureName::DataQualityScore));
    assert_eq!(names.last(), Some(&FeatureName::DayOfWeekCos));
}

#[test]
fn feature_engine_uses_only_current_and_past_candles() {
    let engine = FeatureEngine::default();
    let original = healthy_series();
    let mut mutated = healthy_series();
    mutated.candles[25].high = 1_000.0;
    mutated.candles[25].low = 1.0;
    mutated.candles[25].close = 700.0;
    let a = engine.build_at(&original, 20);
    let b = engine.build_at(&mutated, 20);
    assert_eq!(a, b);
}

#[test]
fn feature_engine_returns_missing_and_low_quality_when_insufficient_bars() {
    let engine = FeatureEngine::default();
    let short = CandleSeries {
        symbol: "SHORT".to_string(),
        timeframe: Timeframe::OneMinute,
        candles: healthy_series().candles[..5].to_vec(),
    };
    let features = engine.build_at(&short, 4);
    assert!(features.data_quality_score < 1.0);
    assert!(
        features
            .values
            .iter()
            .any(|value| matches!(value, FeatureValue::Missing))
    );
}

#[test]
fn no_nan_or_inf_appears_in_generated_features() {
    let engine = FeatureEngine::default();
    let features = engine.build_at(&healthy_series(), 29);
    assert!(!features.has_non_finite_values());
}

#[test]
fn volume_z_20_detects_volume_spike() {
    let engine = FeatureEngine::default();
    let features = engine.build_at(&healthy_series(), 29);
    assert!(features.value(FeatureName::VolumeZ20).unwrap_or(0.0) > 1.0);
}

#[test]
fn close_position_in_range_is_bounded() {
    let engine = FeatureEngine::default();
    let features = engine.build_at(&healthy_series(), 29);
    let value = features
        .value(FeatureName::ClosePositionInRange)
        .expect("position");
    assert!((0.0..=1.0).contains(&value));
}

#[test]
fn spread_bps_is_computed_from_bid_ask_when_available() {
    let engine = FeatureEngine::default();
    let features = engine.build_at(&healthy_series(), 29);
    assert!(features.value(FeatureName::SpreadBps).unwrap_or(0.0) > 0.0);
}

#[test]
fn data_quality_score_drops_for_bad_data() {
    let engine = FeatureEngine::default();
    let mut bad = healthy_series();
    bad.candles[29].close = -1.0;
    bad.candles[29].volume = 0.0;
    let features = engine.build_at(&bad, 29);
    assert!(features.data_quality_score < 0.7);
}
