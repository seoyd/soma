use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::backtest::Timeframe;
use crate::core::ReasonCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeatureName {
    Close,
    LogReturn1,
    LogReturn3,
    LogReturn5,
    LogReturn10,
    LogReturn20,
    ClosePositionInRange,
    HighLowRangePct,
    CandleBodyPct,
    UpperWickPct,
    LowerWickPct,
    Ma5,
    Ma20,
    Ma5OverMa20,
    CloseOverMa20,
    SlopeMa5,
    SlopeMa20,
    Volume,
    VolumeZ20,
    TradeValue,
    TradeValueZ20,
    VolumeRatio5_20,
    Atr14,
    RealizedVol10,
    RealizedVol20,
    BollingerWidth20,
    RangeVolatility,
    Vwap20,
    CloseOverVwap20,
    SpreadBps,
    SpreadBpsFromCandle,
    LiquidityScoreHeuristic,
    DataQualityScore,
    MinuteOfDaySin,
    MinuteOfDayCos,
    DayOfWeekSin,
    DayOfWeekCos,
}

impl FeatureName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Close => "close",
            Self::LogReturn1 => "log_return_1",
            Self::LogReturn3 => "log_return_3",
            Self::LogReturn5 => "log_return_5",
            Self::LogReturn10 => "log_return_10",
            Self::LogReturn20 => "log_return_20",
            Self::ClosePositionInRange => "close_position_in_range",
            Self::HighLowRangePct => "high_low_range_pct",
            Self::CandleBodyPct => "candle_body_pct",
            Self::UpperWickPct => "upper_wick_pct",
            Self::LowerWickPct => "lower_wick_pct",
            Self::Ma5 => "ma_5",
            Self::Ma20 => "ma_20",
            Self::Ma5OverMa20 => "ma_5_over_ma_20",
            Self::CloseOverMa20 => "close_over_ma_20",
            Self::SlopeMa5 => "slope_ma_5",
            Self::SlopeMa20 => "slope_ma_20",
            Self::Volume => "volume",
            Self::VolumeZ20 => "volume_z_20",
            Self::TradeValue => "trade_value",
            Self::TradeValueZ20 => "trade_value_z_20",
            Self::VolumeRatio5_20 => "volume_ratio_5_20",
            Self::Atr14 => "atr_14",
            Self::RealizedVol10 => "realized_vol_10",
            Self::RealizedVol20 => "realized_vol_20",
            Self::BollingerWidth20 => "bollinger_width_20",
            Self::RangeVolatility => "range_volatility",
            Self::Vwap20 => "vwap_20",
            Self::CloseOverVwap20 => "close_over_vwap_20",
            Self::SpreadBps => "spread_bps",
            Self::SpreadBpsFromCandle => "spread_bps_from_candle",
            Self::LiquidityScoreHeuristic => "liquidity_score_heuristic",
            Self::DataQualityScore => "data_quality_score",
            Self::MinuteOfDaySin => "minute_of_day_sin",
            Self::MinuteOfDayCos => "minute_of_day_cos",
            Self::DayOfWeekSin => "day_of_week_sin",
            Self::DayOfWeekCos => "day_of_week_cos",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum FeatureValue {
    Value(f64),
    Missing,
}

impl FeatureValue {
    pub fn as_f64(self) -> Option<f64> {
        match self {
            Self::Value(value) => Some(value),
            Self::Missing => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureVector {
    pub symbol: String,
    pub timestamp_ms: u64,
    pub timeframe: Timeframe,
    pub feature_names: Vec<FeatureName>,
    pub values: Vec<FeatureValue>,
    pub data_quality_score: f64,
    pub reason_codes: Vec<ReasonCode>,
}

impl FeatureVector {
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn value(&self, name: FeatureName) -> Option<f64> {
        self.feature_names
            .iter()
            .position(|candidate| *candidate == name)
            .and_then(|index| self.values.get(index).copied())
            .and_then(FeatureValue::as_f64)
    }

    pub fn has_non_finite_values(&self) -> bool {
        self.values.iter().any(|value| match value {
            FeatureValue::Value(number) => !number.is_finite(),
            FeatureValue::Missing => false,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureFrame {
    pub symbol: String,
    pub timeframe: Timeframe,
    pub rows: Vec<FeatureVector>,
    pub feature_names: Vec<FeatureName>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FeatureConfig {
    pub return_windows: Vec<usize>,
    pub volatility_windows: Vec<usize>,
    pub volume_windows: Vec<usize>,
    pub atr_window: usize,
    pub vwap_window: usize,
    pub zscore_window: usize,
    pub min_required_bars: usize,
    pub include_volume_features: bool,
    pub include_spread_features: bool,
    pub include_data_quality_feature: bool,
    pub include_time_features: bool,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            return_windows: vec![1, 3, 5, 10, 20],
            volatility_windows: vec![10, 20],
            volume_windows: vec![5, 20],
            atr_window: 14,
            vwap_window: 20,
            zscore_window: 20,
            min_required_bars: 20,
            include_volume_features: true,
            include_spread_features: true,
            include_data_quality_feature: true,
            include_time_features: true,
        }
    }
}
