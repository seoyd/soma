use crate::backtest::CandleSeries;
use crate::core::ReasonCode;

use super::FeatureConfig;

#[derive(Clone, Debug, PartialEq)]
pub struct DataQualityResult {
    pub score: f64,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn assess_data_quality(
    series: &CandleSeries,
    index: usize,
    config: &FeatureConfig,
) -> DataQualityResult {
    let mut score: f64 = 1.0;
    let mut reason_codes = Vec::new();
    let Some(candle) = series.candle(index) else {
        return DataQualityResult {
            score: 0.0,
            reason_codes: vec![ReasonCode::InsufficientBars],
        };
    };

    if index + 1 < config.min_required_bars {
        score -= 0.35;
        reason_codes.push(ReasonCode::InsufficientBars);
    }
    if candle.open <= 0.0 || candle.high <= 0.0 || candle.low <= 0.0 || candle.close <= 0.0 {
        score -= 0.50;
        reason_codes.push(ReasonCode::NonPositivePrice);
    }
    if candle.volume <= 0.0 {
        score -= 0.20;
        reason_codes.push(ReasonCode::MissingVolume);
    }
    if candle.bid.is_none() || candle.ask.is_none() {
        score -= 0.10;
        reason_codes.push(ReasonCode::MissingBidAsk);
    }
    let spread_bps = candle
        .spread_bps
        .or_else(|| match (candle.bid, candle.ask) {
            (Some(bid), Some(ask)) if bid > 0.0 && ask > 0.0 => Some((ask - bid) / bid * 10_000.0),
            _ => None,
        })
        .unwrap_or(0.0);
    if spread_bps > 25.0 {
        score -= 0.20;
        reason_codes.push(ReasonCode::ExtremeSpreadDetected);
    }
    if index > 0 {
        if let Some(previous) = series.candle(index - 1) {
            if candle.timestamp_ms <= previous.timestamp_ms {
                score -= 0.20;
                reason_codes.push(ReasonCode::StaleTimestamp);
            }
        }
    }

    DataQualityResult {
        score: score.clamp(0.0, 1.0),
        reason_codes,
    }
}
