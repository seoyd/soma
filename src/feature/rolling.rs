use crate::backtest::Candle;

pub fn safe_div(numerator: f64, denominator: f64) -> f64 {
    if !numerator.is_finite() || !denominator.is_finite() || denominator.abs() < 1e-12 {
        0.0
    } else {
        numerator / denominator
    }
}

pub fn clamp_finite(value: f64, min: f64, max: f64, default: f64) -> f64 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        default.clamp(min, max)
    }
}

pub fn rolling_sum(values: &[f64], window: usize) -> Option<f64> {
    let slice = values.get(values.len().checked_sub(window)?..)?;
    let sum: f64 = slice
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .sum();
    Some(sum)
}

pub fn rolling_mean(values: &[f64], window: usize) -> Option<f64> {
    let slice = values.get(values.len().checked_sub(window)?..)?;
    if slice.iter().any(|value| !value.is_finite()) {
        return None;
    }
    Some(slice.iter().sum::<f64>() / window as f64)
}

pub fn rolling_std(values: &[f64], window: usize) -> Option<f64> {
    let slice = values.get(values.len().checked_sub(window)?..)?;
    let mean = rolling_mean(values, window)?;
    let variance = slice
        .iter()
        .map(|value| {
            let diff = *value - mean;
            diff * diff
        })
        .sum::<f64>()
        / window as f64;
    Some(variance.sqrt())
}

pub fn rolling_min(values: &[f64], window: usize) -> Option<f64> {
    let slice = values.get(values.len().checked_sub(window)?..)?;
    slice.iter().copied().reduce(f64::min)
}

pub fn rolling_max(values: &[f64], window: usize) -> Option<f64> {
    let slice = values.get(values.len().checked_sub(window)?..)?;
    slice.iter().copied().reduce(f64::max)
}

pub fn rolling_zscore(values: &[f64], window: usize) -> Option<f64> {
    let mean = rolling_mean(values, window)?;
    let std = rolling_std(values, window)?;
    let current = *values.last()?;
    if std <= 1e-12 {
        Some(0.0)
    } else {
        Some((current - mean) / std)
    }
}

pub fn pct_change(previous: f64, current: f64) -> Option<f64> {
    if previous <= 0.0 || current <= 0.0 || !previous.is_finite() || !current.is_finite() {
        None
    } else {
        Some((current / previous) - 1.0)
    }
}

pub fn log_return(previous: f64, current: f64) -> Option<f64> {
    if previous <= 0.0 || current <= 0.0 || !previous.is_finite() || !current.is_finite() {
        None
    } else {
        Some((current / previous).ln())
    }
}

pub fn true_range(previous_close: Option<f64>, candle: &Candle) -> f64 {
    let base = candle.high - candle.low;
    if let Some(previous_close) = previous_close {
        base.max((candle.high - previous_close).abs())
            .max((candle.low - previous_close).abs())
    } else {
        base
    }
}

pub fn atr(candles: &[Candle], window: usize) -> Option<f64> {
    let slice = candles.get(candles.len().checked_sub(window)?..)?;
    let mut ranges = Vec::with_capacity(slice.len());
    for (offset, candle) in slice.iter().enumerate() {
        let previous_close = if offset == 0 {
            candles
                .len()
                .checked_sub(window + 1)
                .and_then(|index| candles.get(index))
                .map(|previous| previous.close)
        } else {
            slice.get(offset - 1).map(|previous| previous.close)
        };
        ranges.push(true_range(previous_close, candle));
    }
    rolling_mean(&ranges, ranges.len())
}

pub fn realized_volatility(values: &[f64], window: usize) -> Option<f64> {
    let std = rolling_std(values, window)?;
    Some(std * (window as f64).sqrt())
}
