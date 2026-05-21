use soma_zero::{
    BacktestSimulator, Candle, CandleSeries, GovernorConfig, ReasonCode, RiskGovernor, Timeframe,
};

fn candle(timestamp_ms: u64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Candle {
    Candle {
        timestamp_ms,
        open,
        high,
        low,
        close,
        volume,
        trade_value: Some(close * volume.max(0.0)),
        bid: Some(close - 0.01),
        ask: Some(close + 0.01),
        spread_bps: Some(2.0),
    }
}

fn series() -> CandleSeries {
    CandleSeries {
        symbol: "INT".to_string(),
        timeframe: Timeframe::FiveMinute,
        candles: (0..35)
            .map(|i| {
                let base = 100.0 + i as f64 * 0.3;
                candle(
                    300_000 * (i as u64 + 1),
                    base,
                    base + 0.5,
                    base - 0.4,
                    base + 0.2,
                    1_000.0 + i as f64 * 20.0,
                )
            })
            .collect(),
    }
}

#[test]
fn backtest_simulator_runs_with_feature_and_regime_path() {
    let mut simulator = BacktestSimulator::default();
    simulator.config.max_steps = Some(8);
    let result = simulator.run(&series());
    assert!(result.total_decisions > 0);
    assert_eq!(result.outcome_records.len(), result.total_decisions);
}

#[test]
fn simulator_produces_outcome_records() {
    let mut simulator = BacktestSimulator::default();
    simulator.config.max_steps = Some(6);
    let result = simulator.run(&series());
    assert!(!result.outcome_records.is_empty());
}

#[test]
fn simulator_does_not_leak_future_candles_into_signal_path() {
    let base = series();
    let mut future_mutated = series();
    future_mutated.candles[20].close = 500.0;
    future_mutated.candles[20].high = 600.0;
    future_mutated.candles[20].low = 10.0;

    let mut simulator = BacktestSimulator::default();
    simulator.config.max_steps = Some(5);

    let a = simulator.run(&base);
    let b = simulator.run(&future_mutated);
    assert_eq!(a.decision_records, b.decision_records);
    assert_eq!(a.outcome_records, b.outcome_records);
}

#[test]
fn poor_data_quality_causes_no_trade_or_risk_denial() {
    let mut bad = series();
    bad.candles[25].close = -1.0;
    bad.candles[25].volume = 0.0;
    let mut simulator = BacktestSimulator::default();
    simulator.config.max_steps = Some(10);
    let result = simulator.run(&bad);
    assert!(result.no_trades + result.denied_trades > 0);
}

#[test]
fn same_input_produces_same_backtest_result() {
    let mut simulator = BacktestSimulator::default();
    simulator.config.max_steps = Some(7);
    let a = simulator.run(&series());
    let b = simulator.run(&series());
    assert_eq!(a, b);
}

#[test]
fn risk_governor_denies_when_feature_data_quality_below_threshold() {
    let mut bad = series();
    bad.candles[24].close = -1.0;
    bad.candles[24].volume = 0.0;
    let mut simulator = BacktestSimulator::default();
    simulator.config.max_steps = Some(10);
    simulator.governor = RiskGovernor {
        config: GovernorConfig {
            min_data_quality: 0.95,
            ..GovernorConfig::default()
        },
    };
    let result = simulator.run(&bad);
    assert!(result.denied_trades > 0 || result.no_trades > 0);
}

#[test]
fn risk_governor_still_overrides_chair_and_signal() {
    let mut simulator = BacktestSimulator::default();
    simulator.config.max_steps = Some(8);
    simulator.governor = RiskGovernor {
        config: GovernorConfig {
            min_expected_edge: 0.04,
            ..GovernorConfig::default()
        },
    };
    let result = simulator.run(&series());
    assert!(result.denied_trades > 0 || result.executed_trades == 0);
}

#[test]
fn no_real_broker_path_exists() {
    let result = BacktestSimulator::default().run(&series());
    assert!(
        result
            .reason_codes
            .contains(&ReasonCode::PaperExecutionOnly)
    );
}
