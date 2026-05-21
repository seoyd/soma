use soma_zero::{Candle, CandleSeries, ReasonCode, Timeframe, WalkForwardConfig, WalkForwardSplit};

fn series(len: usize) -> CandleSeries {
    CandleSeries {
        symbol: "WF".to_string(),
        timeframe: Timeframe::FiveMinute,
        candles: (0..len)
            .map(|i| {
                let base = 100.0 + i as f64 * 0.3;
                Candle {
                    timestamp_ms: i as u64 * 300_000,
                    open: base,
                    high: base + 0.7,
                    low: base - 0.4,
                    close: base + 0.2,
                    volume: 1_000.0 + i as f64 * 10.0,
                    trade_value: Some((base + 0.2) * 1_000.0),
                    bid: Some(base + 0.18),
                    ask: Some(base + 0.22),
                    spread_bps: Some(2.0),
                }
            })
            .collect(),
    }
}

#[test]
fn fold_generation_is_deterministic_and_time_ordered() {
    let config = WalkForwardConfig {
        train_window_bars: 30,
        validation_window_bars: Some(5),
        test_window_bars: 12,
        step_bars: 10,
        embargo_bars: 3,
        min_train_bars: 20,
        max_folds: Some(3),
        allow_partial_last_fold: false,
    };
    let left = WalkForwardSplit::generate(&series(120), config);
    let right = WalkForwardSplit::generate(&series(120), config);

    assert_eq!(left, right);
    assert_eq!(left.folds.len(), 3);
    for fold in &left.folds {
        assert!(fold.train_end_index < fold.test_start_index);
        assert_eq!(
            fold.embargo_end_index.expect("embargo") + 1,
            fold.test_start_index
        );
        assert!(
            fold.reason_codes
                .contains(&ReasonCode::WalkForwardFoldGenerated)
        );
    }
}

#[test]
fn insufficient_data_produces_zero_folds_with_reason_code() {
    let split = WalkForwardSplit::generate(
        &series(20),
        WalkForwardConfig {
            train_window_bars: 20,
            validation_window_bars: Some(5),
            test_window_bars: 10,
            step_bars: 5,
            embargo_bars: 3,
            min_train_bars: 20,
            max_folds: None,
            allow_partial_last_fold: false,
        },
    );

    assert!(split.folds.is_empty());
    assert!(
        split
            .reason_codes
            .contains(&ReasonCode::WalkForwardInsufficientData)
    );
}

#[test]
fn max_folds_is_respected() {
    let split = WalkForwardSplit::generate(
        &series(200),
        WalkForwardConfig {
            max_folds: Some(2),
            ..WalkForwardConfig::default()
        },
    );

    assert_eq!(split.folds.len(), 2);
}
