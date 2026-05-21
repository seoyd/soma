use serde::{Deserialize, Serialize};

use crate::backtest::{Candle, CandleSeries, Timeframe};
use crate::core::ReasonCode;

use super::{TimeframeSpec, detect_temporal_issues};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResampleMethod {
    OhlcvAggregate,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResampleConfig {
    pub source_timeframe: Timeframe,
    pub target_timeframe: Timeframe,
    pub method: ResampleMethod,
    pub require_contiguous_source: bool,
    pub drop_partial: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ResampleConfig {
    fn default() -> Self {
        Self {
            source_timeframe: Timeframe::OneMinute,
            target_timeframe: Timeframe::FiveMinute,
            method: ResampleMethod::OhlcvAggregate,
            require_contiguous_source: true,
            drop_partial: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResampleResult {
    pub series: CandleSeries,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Resampler;

impl Resampler {
    pub fn resample(
        &self,
        series: &CandleSeries,
        config: &ResampleConfig,
    ) -> Result<ResampleResult, Vec<ReasonCode>> {
        let source = TimeframeSpec::from_timeframe(config.source_timeframe);
        let target = TimeframeSpec::from_timeframe(config.target_timeframe);
        if !source.is_supported() || !target.is_supported() || target.seconds <= source.seconds {
            return Err(vec![ReasonCode::UnsupportedTimeframe]);
        }
        let ratio = target.seconds / source.seconds;
        if ratio <= 1 || target.seconds % source.seconds != 0 {
            return Err(vec![ReasonCode::UnsupportedTimeframe]);
        }

        let (_, _, gap_count, _) =
            detect_temporal_issues(&series.candles, Some(source.expected_ms_step));
        if config.require_contiguous_source && gap_count > 0 {
            return Err(vec![ReasonCode::NonContiguousSource]);
        }

        let mut aggregated = Vec::new();
        let mut reason_codes = config.reason_codes.clone();
        let ratio = ratio as usize;
        for chunk in series.candles.chunks(ratio) {
            if chunk.len() < ratio {
                if config.drop_partial {
                    reason_codes.push(ReasonCode::PartialWindowDropped);
                    break;
                }
                return Err(vec![ReasonCode::PartialWindowDropped]);
            }
            aggregated.push(aggregate_chunk(chunk));
        }
        reason_codes.push(ReasonCode::ResamplingApplied);
        Ok(ResampleResult {
            series: CandleSeries {
                symbol: series.symbol.clone(),
                timeframe: config.target_timeframe,
                candles: aggregated,
            },
            reason_codes,
        })
    }
}

fn aggregate_chunk(chunk: &[Candle]) -> Candle {
    let first = chunk.first().expect("chunk");
    let last = chunk.last().expect("chunk");
    let trade_value = chunk
        .iter()
        .try_fold(0.0, |acc, candle| Some(acc + candle.trade_value?));
    let bid = chunk.first().and_then(|candle| candle.bid);
    let ask = chunk.last().and_then(|candle| candle.ask);
    let spread_bps = match (bid, ask) {
        (Some(bid), Some(ask)) if bid > 0.0 => Some((ask - bid) / bid * 10_000.0),
        _ => chunk.last().and_then(|candle| candle.spread_bps),
    };
    Candle {
        timestamp_ms: first.timestamp_ms,
        open: first.open,
        high: chunk
            .iter()
            .map(|candle| candle.high)
            .fold(f64::NEG_INFINITY, f64::max),
        low: chunk
            .iter()
            .map(|candle| candle.low)
            .fold(f64::INFINITY, f64::min),
        close: last.close,
        volume: chunk.iter().map(|candle| candle.volume).sum(),
        trade_value,
        bid,
        ask,
        spread_bps,
    }
}
