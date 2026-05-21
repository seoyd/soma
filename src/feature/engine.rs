use std::collections::BTreeMap;
use std::f64::consts::TAU;

use crate::backtest::{Candle, CandleSeries};
use crate::core::ReasonCode;

use super::quality::assess_data_quality;
use super::rolling::{
    atr, clamp_finite, log_return, realized_volatility, rolling_mean, rolling_std, rolling_zscore,
    safe_div,
};
use super::{FeatureConfig, FeatureFrame, FeatureName, FeatureValue, FeatureVector};

#[derive(Clone, Debug, PartialEq)]
pub struct FeatureEngine {
    pub config: FeatureConfig,
}

impl Default for FeatureEngine {
    fn default() -> Self {
        Self {
            config: FeatureConfig::default(),
        }
    }
}

impl FeatureEngine {
    pub fn feature_names(&self) -> Vec<FeatureName> {
        let mut names = vec![
            FeatureName::Close,
            FeatureName::LogReturn1,
            FeatureName::LogReturn3,
            FeatureName::LogReturn5,
            FeatureName::LogReturn10,
            FeatureName::LogReturn20,
            FeatureName::ClosePositionInRange,
            FeatureName::HighLowRangePct,
            FeatureName::CandleBodyPct,
            FeatureName::UpperWickPct,
            FeatureName::LowerWickPct,
            FeatureName::Ma5,
            FeatureName::Ma20,
            FeatureName::Ma5OverMa20,
            FeatureName::CloseOverMa20,
            FeatureName::SlopeMa5,
            FeatureName::SlopeMa20,
            FeatureName::Atr14,
            FeatureName::RealizedVol10,
            FeatureName::RealizedVol20,
            FeatureName::BollingerWidth20,
            FeatureName::RangeVolatility,
            FeatureName::Vwap20,
            FeatureName::CloseOverVwap20,
        ];
        if self.config.include_volume_features {
            names.extend([
                FeatureName::Volume,
                FeatureName::VolumeZ20,
                FeatureName::TradeValue,
                FeatureName::TradeValueZ20,
                FeatureName::VolumeRatio5_20,
            ]);
        }
        if self.config.include_spread_features {
            names.extend([
                FeatureName::SpreadBps,
                FeatureName::SpreadBpsFromCandle,
                FeatureName::LiquidityScoreHeuristic,
            ]);
        }
        if self.config.include_data_quality_feature {
            names.push(FeatureName::DataQualityScore);
        }
        if self.config.include_time_features {
            names.extend([
                FeatureName::MinuteOfDaySin,
                FeatureName::MinuteOfDayCos,
                FeatureName::DayOfWeekSin,
                FeatureName::DayOfWeekCos,
            ]);
        }
        names
    }

    pub fn build_at(&self, series: &CandleSeries, index: usize) -> FeatureVector {
        let feature_names = self.feature_names();
        let Some(candle) = series.candle(index) else {
            return FeatureVector {
                symbol: series.symbol.clone(),
                timestamp_ms: 0,
                timeframe: series.timeframe,
                feature_names,
                values: vec![FeatureValue::Missing; self.feature_names().len()],
                data_quality_score: 0.0,
                reason_codes: vec![ReasonCode::InsufficientBars],
            };
        };

        let quality = assess_data_quality(series, index, &self.config);
        let window = series.lookback_window(index, self.config.min_required_bars.max(20) - 1);
        let lookback = window.unwrap_or(&series.candles[..=index]);
        let closes: Vec<f64> = lookback.iter().map(|row| row.close).collect();
        let volumes: Vec<f64> = lookback.iter().map(|row| row.volume).collect();
        let trade_values: Vec<f64> = lookback
            .iter()
            .map(|row| {
                row.trade_value
                    .unwrap_or(row.close.max(0.0) * row.volume.max(0.0))
            })
            .collect();
        let log_returns: Vec<f64> = closes
            .windows(2)
            .filter_map(|pair| log_return(pair[0], pair[1]))
            .collect();
        let range_window: Vec<f64> = lookback
            .iter()
            .map(|row| safe_div(row.high - row.low, row.close.max(1e-9)))
            .collect();

        let mut values = Vec::with_capacity(feature_names.len());
        for name in &feature_names {
            values.push(self.compute_feature(
                *name,
                candle,
                lookback,
                &closes,
                &volumes,
                &trade_values,
                &log_returns,
                &range_window,
                quality.score,
            ));
        }

        let mut reason_codes = quality.reason_codes;
        if values
            .iter()
            .any(|value| matches!(value, FeatureValue::Missing))
        {
            reason_codes.push(ReasonCode::FeatureUnavailable);
        }
        if values
            .iter()
            .any(|value| matches!(value, FeatureValue::Value(number) if !number.is_finite()))
        {
            reason_codes.push(ReasonCode::NonFiniteFeature);
        }

        FeatureVector {
            symbol: series.symbol.clone(),
            timestamp_ms: candle.timestamp_ms,
            timeframe: series.timeframe,
            feature_names,
            values,
            data_quality_score: quality.score,
            reason_codes,
        }
    }

    pub fn build_frame(&self, series: &CandleSeries) -> FeatureFrame {
        let feature_names = self.feature_names();
        let rows = (0..series.len())
            .map(|index| self.build_at(series, index))
            .collect();
        FeatureFrame {
            symbol: series.symbol.clone(),
            timeframe: series.timeframe,
            rows,
            feature_names,
            metadata: BTreeMap::from([
                ("engine".to_string(), "feature_engine_v0".to_string()),
                ("stable_order".to_string(), "true".to_string()),
            ]),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn compute_feature(
        &self,
        name: FeatureName,
        candle: &Candle,
        lookback: &[Candle],
        closes: &[f64],
        volumes: &[f64],
        trade_values: &[f64],
        log_returns: &[f64],
        range_window: &[f64],
        data_quality_score: f64,
    ) -> FeatureValue {
        let value = match name {
            FeatureName::Close => Some(candle.close),
            FeatureName::LogReturn1 => relative_log_return(closes, 1),
            FeatureName::LogReturn3 => relative_log_return(closes, 3),
            FeatureName::LogReturn5 => relative_log_return(closes, 5),
            FeatureName::LogReturn10 => relative_log_return(closes, 10),
            FeatureName::LogReturn20 => relative_log_return(closes, 20),
            FeatureName::ClosePositionInRange => Some(clamp_finite(
                safe_div(candle.close - candle.low, candle.high - candle.low),
                0.0,
                1.0,
                0.5,
            )),
            FeatureName::HighLowRangePct => Some(clamp_finite(
                safe_div(candle.high - candle.low, candle.close.max(1e-9)),
                0.0,
                5.0,
                0.0,
            )),
            FeatureName::CandleBodyPct => Some(clamp_finite(
                safe_div(
                    (candle.close - candle.open).abs(),
                    (candle.high - candle.low).max(1e-9),
                ),
                0.0,
                1.0,
                0.0,
            )),
            FeatureName::UpperWickPct => Some(clamp_finite(
                safe_div(
                    candle.high - candle.open.max(candle.close),
                    (candle.high - candle.low).max(1e-9),
                ),
                0.0,
                1.0,
                0.0,
            )),
            FeatureName::LowerWickPct => Some(clamp_finite(
                safe_div(
                    candle.open.min(candle.close) - candle.low,
                    (candle.high - candle.low).max(1e-9),
                ),
                0.0,
                1.0,
                0.0,
            )),
            FeatureName::Ma5 => rolling_mean(closes, 5),
            FeatureName::Ma20 => rolling_mean(closes, 20),
            FeatureName::Ma5OverMa20 => ratio_of(rolling_mean(closes, 5), rolling_mean(closes, 20)),
            FeatureName::CloseOverMa20 => ratio_of(Some(candle.close), rolling_mean(closes, 20)),
            FeatureName::SlopeMa5 => slope_of(closes, 5),
            FeatureName::SlopeMa20 => slope_of(closes, 20),
            FeatureName::Volume => Some(candle.volume),
            FeatureName::VolumeZ20 => rolling_zscore(volumes, self.config.zscore_window),
            FeatureName::TradeValue => Some(*trade_values.last().unwrap_or(&0.0)),
            FeatureName::TradeValueZ20 => rolling_zscore(trade_values, self.config.zscore_window),
            FeatureName::VolumeRatio5_20 => {
                ratio_of(rolling_mean(volumes, 5), rolling_mean(volumes, 20))
            }
            FeatureName::Atr14 => atr(lookback, self.config.atr_window),
            FeatureName::RealizedVol10 => realized_volatility(log_returns, 10),
            FeatureName::RealizedVol20 => realized_volatility(log_returns, 20),
            FeatureName::BollingerWidth20 => rolling_mean(closes, 20)
                .zip(rolling_std(closes, 20))
                .map(|(ma20, std20)| safe_div(4.0 * std20, ma20.max(1e-9))),
            FeatureName::RangeVolatility => rolling_mean(range_window, 5),
            FeatureName::Vwap20 => lookback
                .len()
                .checked_sub(self.config.vwap_window)
                .and_then(|start| lookback.get(start..))
                .map(|tail| {
                    let numerator: f64 =
                        tail.iter().map(|row| row.close * row.volume.max(0.0)).sum();
                    let denominator: f64 = tail.iter().map(|row| row.volume.max(0.0)).sum();
                    safe_div(numerator, denominator.max(1e-9))
                }),
            FeatureName::CloseOverVwap20 => ratio_of(
                Some(candle.close),
                self.compute_feature(
                    FeatureName::Vwap20,
                    candle,
                    lookback,
                    closes,
                    volumes,
                    trade_values,
                    log_returns,
                    range_window,
                    data_quality_score,
                )
                .as_f64(),
            ),
            FeatureName::SpreadBps => spread_from_bid_ask(candle),
            FeatureName::SpreadBpsFromCandle => Some(candle.spread_bps.unwrap_or_else(|| {
                safe_div(candle.high - candle.low, candle.close.max(1e-9)) * 10_000.0 * 0.08
            })),
            FeatureName::LiquidityScoreHeuristic => Some(clamp_finite(
                0.6 * safe_div(*trade_values.last().unwrap_or(&0.0), 1_000_000.0)
                    + 0.4 * (1.0 - safe_div(candle.spread_bps.unwrap_or(2.0), 25.0)),
                0.0,
                1.0,
                0.0,
            )),
            FeatureName::DataQualityScore => Some(data_quality_score),
            FeatureName::MinuteOfDaySin => Some(time_angle(candle.timestamp_ms, 1_440).0),
            FeatureName::MinuteOfDayCos => Some(time_angle(candle.timestamp_ms, 1_440).1),
            FeatureName::DayOfWeekSin => Some(day_of_week_angle(candle.timestamp_ms).0),
            FeatureName::DayOfWeekCos => Some(day_of_week_angle(candle.timestamp_ms).1),
        };

        value
            .filter(|number| number.is_finite())
            .map(FeatureValue::Value)
            .unwrap_or(FeatureValue::Missing)
    }
}

fn relative_log_return(closes: &[f64], window: usize) -> Option<f64> {
    let previous = *closes.get(closes.len().checked_sub(window + 1)?)?;
    let current = *closes.last()?;
    log_return(previous, current)
}

fn ratio_of(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    Some(safe_div(left?, right?))
}

fn slope_of(values: &[f64], window: usize) -> Option<f64> {
    let slice = values.get(values.len().checked_sub(window)?..)?;
    let first = *slice.first()?;
    let last = *slice.last()?;
    Some(safe_div(last - first, first.max(1e-9)))
}

fn spread_from_bid_ask(candle: &Candle) -> Option<f64> {
    Some(safe_div(candle.ask? - candle.bid?, candle.bid?.max(1e-9)) * 10_000.0)
}

fn time_angle(timestamp_ms: u64, period_minutes: u64) -> (f64, f64) {
    let total_minutes = timestamp_ms / 60_000;
    let minute_in_period = (total_minutes % period_minutes) as f64;
    let angle = TAU * minute_in_period / period_minutes as f64;
    (angle.sin(), angle.cos())
}

fn day_of_week_angle(timestamp_ms: u64) -> (f64, f64) {
    let day_index = (((timestamp_ms / 86_400_000) + 4) % 7) as f64;
    let angle = TAU * day_index / 7.0;
    (angle.sin(), angle.cos())
}
