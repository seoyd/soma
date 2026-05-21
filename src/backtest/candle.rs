use serde::{Deserialize, Serialize};

use crate::core::{MarketSnapshot, Regime};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Timeframe {
    OneMinute,
    FiveMinute,
    FifteenMinute,
    OneHour,
    OneDay,
    Custom { seconds: u32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Candle {
    pub timestamp_ms: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_value: Option<f64>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub spread_bps: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleSeries {
    pub symbol: String,
    pub timeframe: Timeframe,
    pub candles: Vec<Candle>,
}

impl CandleSeries {
    pub fn len(&self) -> usize {
        self.candles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candles.is_empty()
    }

    pub fn candle(&self, index: usize) -> Option<&Candle> {
        self.candles.get(index)
    }

    pub fn lookback_window(&self, current_index: usize, lookback_bars: usize) -> Option<&[Candle]> {
        if current_index >= self.candles.len() {
            return None;
        }
        let start = current_index.saturating_sub(lookback_bars);
        Some(&self.candles[start..=current_index])
    }

    pub fn replay_cursor(&self, current_index: usize) -> Option<MarketReplayCursor<'_>> {
        MarketReplayCursor::new(self, current_index)
    }

    pub fn market_snapshot_at(&self, index: usize) -> Option<MarketSnapshot> {
        let candle = self.candle(index)?;
        let prev_close = if index > 0 {
            self.candles.get(index - 1).map(|prev| prev.close)
        } else {
            None
        };
        Some(MarketSnapshot {
            symbol: self.symbol.clone(),
            timestamp_ms: candle.timestamp_ms,
            price: candle.close,
            bid: candle
                .bid
                .unwrap_or(candle.close * (1.0 - candle.spread_bps.unwrap_or(2.0) / 20_000.0)),
            ask: candle
                .ask
                .unwrap_or(candle.close * (1.0 + candle.spread_bps.unwrap_or(2.0) / 20_000.0)),
            spread_bps: candle
                .spread_bps
                .unwrap_or_else(|| infer_spread_bps(candle)),
            volume: candle.volume,
            trade_value: candle
                .trade_value
                .unwrap_or_else(|| candle.close.max(0.0) * candle.volume.max(0.0)),
            volatility: infer_volatility(candle),
            regime: infer_regime(prev_close, candle),
            data_quality_score: infer_data_quality(candle),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarketReplayCursor<'a> {
    series: &'a CandleSeries,
    current_index: usize,
}

impl<'a> MarketReplayCursor<'a> {
    pub fn new(series: &'a CandleSeries, current_index: usize) -> Option<Self> {
        if current_index < series.len() {
            Some(Self {
                series,
                current_index,
            })
        } else {
            None
        }
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn current_candle(&self) -> &'a Candle {
        &self.series.candles[self.current_index]
    }

    pub fn lookback_window(&self, lookback_bars: usize) -> &'a [Candle] {
        let start = self.current_index.saturating_sub(lookback_bars);
        &self.series.candles[start..=self.current_index]
    }

    pub fn market_snapshot(&self) -> MarketSnapshot {
        self.series
            .market_snapshot_at(self.current_index)
            .expect("cursor index validated")
    }

    pub fn advance(&self) -> Option<Self> {
        Self::new(self.series, self.current_index + 1)
    }
}

fn infer_spread_bps(candle: &Candle) -> f64 {
    ((candle.high - candle.low).abs() / candle.close.max(1e-9) * 10_000.0 * 0.08).clamp(1.0, 20.0)
}

fn infer_volatility(candle: &Candle) -> f64 {
    ((candle.high - candle.low).abs() / candle.open.max(1e-9)).clamp(0.0, 0.25)
}

fn infer_data_quality(candle: &Candle) -> f64 {
    let monotonic = candle.high >= candle.low
        && candle.high >= candle.open.min(candle.close)
        && candle.low <= candle.open.max(candle.close);
    let positive = candle.open > 0.0 && candle.high > 0.0 && candle.low > 0.0 && candle.close > 0.0;
    if monotonic && positive {
        0.98
    } else if positive {
        0.75
    } else {
        0.40
    }
}

fn infer_regime(prev_close: Option<f64>, candle: &Candle) -> Regime {
    let intrabar_range = (candle.high - candle.low).abs() / candle.open.max(1e-9);
    if intrabar_range >= 0.08 {
        return Regime::Panic;
    }
    if intrabar_range >= 0.04 {
        return Regime::HighVolatility;
    }
    let Some(prev_close) = prev_close else {
        return Regime::Unknown;
    };
    let change = (candle.close - prev_close) / prev_close.max(1e-9);
    if change >= 0.004 {
        Regime::TrendUp
    } else if change <= -0.004 {
        Regime::TrendDown
    } else {
        Regime::Range
    }
}
