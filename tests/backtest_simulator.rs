use soma_zero::{
    BacktestSimulator, Candle, CandleSeries, ChairConfig, ReasonCode, RiskGovernor, Timeframe,
};

fn candle(timestamp_ms: u64, open: f64, high: f64, low: f64, close: f64) -> Candle {
    Candle {
        timestamp_ms,
        open,
        high,
        low,
        close,
        volume: 5_000.0,
        trade_value: Some(close * 5_000.0),
        bid: None,
        ask: None,
        spread_bps: Some(1.5),
    }
}

fn trending_series() -> CandleSeries {
    CandleSeries {
        symbol: "SIM".to_string(),
        timeframe: Timeframe::FiveMinute,
        candles: (0..32)
            .map(|i| {
                let base = 100.0 + i as f64 * 0.35;
                candle(i as u64 + 1, base, base + 0.6, base - 0.3, base + 0.25)
            })
            .collect(),
    }
}

#[test]
fn synthetic_series_produces_deterministic_result() {
    let mut simulator = BacktestSimulator::default();
    simulator.config.max_steps = Some(6);

    let result_a = simulator.run(&trending_series());
    let result_b = simulator.run(&trending_series());

    assert_eq!(result_a, result_b);
}

#[test]
fn same_input_produces_same_backtest_result() {
    let mut simulator = BacktestSimulator::default();
    simulator.config.max_steps = Some(5);

    let one = simulator.run(&trending_series());
    let two = simulator.run(&trending_series());
    assert_eq!(one, two);
}

#[test]
fn simulator_uses_paper_only_order_ids() {
    let mut simulator = BacktestSimulator::default();
    simulator.config.max_steps = Some(6);

    let result = simulator.run(&trending_series());
    assert!(
        result
            .reason_codes
            .contains(&ReasonCode::PaperExecutionOnly)
    );
    assert!(
        result
            .decision_records
            .iter()
            .filter_map(|record| record.paper_order_id.as_ref())
            .all(|order_id| order_id.starts_with("paper-"))
    );
}

#[test]
fn simulator_respects_full_auto_mode() {
    let mut manual = BacktestSimulator::default();
    manual.config.max_steps = Some(8);
    manual.chair.config = ChairConfig {
        strong_threshold: 1.0,
        weak_threshold: 0.01,
        ..ChairConfig::default()
    };

    let mut auto = manual.clone();
    auto.config.full_auto = true;

    let manual_result = manual.run(&trending_series());
    let auto_result = auto.run(&trending_series());

    assert!(manual_result.decision_records.iter().all(|record| {
        !record
            .reason_codes
            .contains(&ReasonCode::RequireConfirmBlockedInAuto)
    }));
    assert!(auto_result.executed_trades <= manual_result.executed_trades);
}

#[test]
fn simulator_records_no_executions_when_risk_governor_denies() {
    let mut simulator = BacktestSimulator::default();
    simulator.config.max_steps = Some(6);
    simulator.governor = RiskGovernor {
        config: soma_zero::GovernorConfig {
            min_expected_edge: 0.05,
            ..soma_zero::GovernorConfig::default()
        },
    };

    let result = simulator.run(&trending_series());
    assert_eq!(result.executed_trades, 0);
    assert!(
        result
            .outcome_records
            .iter()
            .filter(|record| record.denied_by_risk || record.no_trade)
            .all(|record| !record.executed)
    );
}

#[test]
fn outcome_record_count_matches_total_decisions() {
    let mut simulator = BacktestSimulator::default();
    simulator.config.max_steps = Some(5);

    let result = simulator.run(&trending_series());
    assert_eq!(result.outcome_records.len(), result.total_decisions);
    assert_eq!(result.decision_records.len(), result.total_decisions);
}
