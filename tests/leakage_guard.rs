use soma_zero::{
    Candle, CandleSeries, FeatureEngine, LeakageGuard, ReasonCode, Side, Timeframe,
    TripleBarrierConfig, WalkForwardFold,
};

fn series() -> CandleSeries {
    CandleSeries {
        symbol: "LEAK".to_string(),
        timeframe: Timeframe::FiveMinute,
        candles: (0..40)
            .map(|i| {
                let base = 100.0 + i as f64 * 0.2;
                Candle {
                    timestamp_ms: i as u64 * 300_000,
                    open: base,
                    high: base + 0.8,
                    low: base - 0.4,
                    close: base + 0.3,
                    volume: 1_000.0 + i as f64 * 20.0,
                    trade_value: Some((base + 0.3) * 1_000.0),
                    bid: Some(base + 0.28),
                    ask: Some(base + 0.32),
                    spread_bps: Some(2.0),
                }
            })
            .collect(),
    }
}

#[test]
fn feature_at_index_is_stable_when_future_candle_changes() {
    let base = series();
    let mut mutated = series();
    mutated.candles[25].close *= 1.5;
    mutated.candles[25].high *= 1.6;
    let engine = FeatureEngine::default();

    assert!(LeakageGuard::feature_stable_at(
        &engine, &base, &mutated, 15
    ));
}

#[test]
fn label_can_change_only_in_label_stage_when_future_changes() {
    let base = series();
    let mut mutated = series();
    mutated.candles[12].high = 130.0;
    mutated.candles[12].close = 129.0;

    assert!(LeakageGuard::label_changes_only_in_label_stage(
        &base,
        &mutated,
        8,
        base.candles[8].close,
        TripleBarrierConfig {
            take_profit_pct: 0.02,
            stop_loss_pct: 0.01,
            horizon_bars: 5,
            fee_bps: 2.0,
            slippage_bps: 2.0,
            side: Side::Long,
            use_high_low_intrabar: true,
        },
    ));
}

#[test]
fn overlapping_fold_is_flagged_as_leakage() {
    let report = LeakageGuard::analyze_fold(
        &WalkForwardFold {
            fold_id: 0,
            train_start_index: 0,
            train_end_index: 20,
            validation_start_index: None,
            validation_end_index: None,
            test_start_index: 18,
            test_end_index: 30,
            embargo_start_index: None,
            embargo_end_index: None,
            reason_codes: vec![],
        },
        4,
    );

    assert!(report.has_leakage);
    assert!(report.warnings.contains(&ReasonCode::FoldOverlapDetected));
}

#[test]
fn unsafe_boundary_rows_are_counted() {
    let report = LeakageGuard::analyze_fold(
        &WalkForwardFold {
            fold_id: 0,
            train_start_index: 0,
            train_end_index: 10,
            validation_start_index: Some(11),
            validation_end_index: Some(14),
            test_start_index: 18,
            test_end_index: 24,
            embargo_start_index: Some(15),
            embargo_end_index: Some(17),
            reason_codes: vec![],
        },
        3,
    );

    assert!(report.unsafe_rows_count > 0);
    assert!(report.warnings.contains(&ReasonCode::UnsafeLabelBoundary));
}
