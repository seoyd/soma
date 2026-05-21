use soma_zero::{Candle, atr, log_return, rolling_mean, rolling_std, safe_div};

fn candle(open: f64, high: f64, low: f64, close: f64) -> Candle {
    Candle {
        timestamp_ms: 0,
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

#[test]
fn rolling_mean_is_correct() {
    let values = [1.0, 2.0, 3.0, 4.0];
    assert_eq!(rolling_mean(&values, 2), Some(3.5));
}

#[test]
fn rolling_std_is_correct_and_stable() {
    let values = [1.0, 2.0, 3.0, 4.0];
    let std = rolling_std(&values, 4).expect("std");
    assert!((std - 1.118_033_988_75).abs() < 1e-9);
}

#[test]
fn safe_div_does_not_produce_nan_or_inf() {
    assert_eq!(safe_div(1.0, 0.0), 0.0);
    assert_eq!(safe_div(f64::INFINITY, 1.0), 0.0);
}

#[test]
fn log_return_handles_non_positive_price_safely() {
    assert_eq!(log_return(0.0, 10.0), None);
    assert_eq!(log_return(10.0, -1.0), None);
}

#[test]
fn atr_produces_expected_result_on_small_series() {
    let candles = vec![
        candle(10.0, 11.0, 9.5, 10.5),
        candle(10.5, 11.5, 10.0, 11.0),
        candle(11.0, 11.8, 10.6, 11.2),
    ];
    let value = atr(&candles, 3).expect("atr");
    assert!((value - 1.4).abs() < 1e-9);
}
