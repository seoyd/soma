use serde::{Deserialize, Serialize};

use crate::backtest::Candle;
use crate::core::{ReasonCode, Regime};
use crate::feature::{FeatureName, FeatureVector};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegimeClassifierConfig {
    pub min_data_quality: f64,
    pub high_vol_threshold: f64,
    pub panic_return_threshold: f64,
    pub panic_volume_z_threshold: f64,
    pub risk_on_return_threshold: f64,
    pub risk_off_return_threshold: f64,
    pub range_return_abs_threshold: f64,
}

impl Default for RegimeClassifierConfig {
    fn default() -> Self {
        Self {
            min_data_quality: 0.55,
            high_vol_threshold: 0.08,
            panic_return_threshold: -0.03,
            panic_volume_z_threshold: 1.0,
            risk_on_return_threshold: 0.01,
            risk_off_return_threshold: -0.01,
            range_return_abs_threshold: 0.01,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RegimeDecision {
    pub regime: Regime,
    pub regime_confidence: f64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RegimeClassifier {
    pub config: RegimeClassifierConfig,
}

impl Default for RegimeClassifier {
    fn default() -> Self {
        Self {
            config: RegimeClassifierConfig::default(),
        }
    }
}

impl RegimeClassifier {
    pub fn classify(&self, features: &FeatureVector, candles: &[Candle]) -> RegimeDecision {
        let mut reason_codes = Vec::new();
        let quality = features.data_quality_score;
        if candles.len() < 5 || quality < self.config.min_data_quality {
            reason_codes.push(ReasonCode::UnknownRegimeGateBreached);
            if quality < self.config.min_data_quality {
                reason_codes.push(ReasonCode::FeatureDataQualityLow);
            }
            return RegimeDecision {
                regime: Regime::Unknown,
                regime_confidence: 0.2,
                reason_codes,
            };
        }

        let close_over_ma20 = features.value(FeatureName::CloseOverMa20).unwrap_or(1.0);
        let ma5_over_ma20 = features.value(FeatureName::Ma5OverMa20).unwrap_or(1.0);
        let log_return_3 = features.value(FeatureName::LogReturn3).unwrap_or(0.0);
        let log_return_5 = features.value(FeatureName::LogReturn5).unwrap_or(0.0);
        let realized_vol_20 = features.value(FeatureName::RealizedVol20).unwrap_or(0.0);
        let atr_14 = features.value(FeatureName::Atr14).unwrap_or(0.0);
        let volume_z_20 = features.value(FeatureName::VolumeZ20).unwrap_or(0.0);
        let close_position = features
            .value(FeatureName::ClosePositionInRange)
            .unwrap_or(0.5);

        let panic = log_return_3 <= self.config.panic_return_threshold
            && (realized_vol_20 >= self.config.high_vol_threshold
                || atr_14 >= candles.last().map(|c| c.close * 0.02).unwrap_or(0.0))
            && (volume_z_20 >= self.config.panic_volume_z_threshold
                || log_return_5 <= self.config.panic_return_threshold * 1.5);
        if panic {
            reason_codes.push(ReasonCode::PanicRegimeDetected);
            return RegimeDecision {
                regime: Regime::Panic,
                regime_confidence: 0.9,
                reason_codes,
            };
        }

        let high_vol = realized_vol_20 >= self.config.high_vol_threshold
            || atr_14 >= candles.last().map(|c| c.close * 0.02).unwrap_or(0.0);
        if high_vol {
            reason_codes.push(ReasonCode::HighVolatilityDetected);
            return RegimeDecision {
                regime: Regime::HighVolatility,
                regime_confidence: 0.8,
                reason_codes,
            };
        }

        let risk_off = log_return_5 <= self.config.risk_off_return_threshold
            && (close_over_ma20 < 0.997 || realized_vol_20 > self.config.high_vol_threshold * 0.75);
        if risk_off {
            reason_codes.push(ReasonCode::RiskOffDetected);
            return RegimeDecision {
                regime: Regime::RiskOff,
                regime_confidence: 0.72,
                reason_codes,
            };
        }

        let risk_on = log_return_5 >= self.config.risk_on_return_threshold
            && volume_z_20 >= 0.2
            && realized_vol_20 < self.config.high_vol_threshold;
        if risk_on {
            reason_codes.push(ReasonCode::RiskOnDetected);
            return RegimeDecision {
                regime: Regime::RiskOn,
                regime_confidence: 0.72,
                reason_codes,
            };
        }

        let trend_down = close_over_ma20 < 0.998 && ma5_over_ma20 < 0.998 && log_return_5 < 0.0;
        if trend_down {
            reason_codes.push(ReasonCode::TrendDownDetected);
            return RegimeDecision {
                regime: Regime::TrendDown,
                regime_confidence: 0.70,
                reason_codes,
            };
        }

        let trend_up = close_over_ma20 > 1.002 && ma5_over_ma20 > 1.001 && log_return_5 > 0.0;
        if trend_up {
            reason_codes.push(ReasonCode::TrendUpDetected);
            return RegimeDecision {
                regime: Regime::TrendUp,
                regime_confidence: 0.70,
                reason_codes,
            };
        }

        let range = log_return_5.abs() <= self.config.range_return_abs_threshold
            && realized_vol_20 < self.config.high_vol_threshold
            && (0.25..=0.75).contains(&close_position);
        if range {
            reason_codes.push(ReasonCode::RangeRegimeDetected);
            return RegimeDecision {
                regime: Regime::Range,
                regime_confidence: 0.60,
                reason_codes,
            };
        }

        reason_codes.push(ReasonCode::UnknownRegimeGateBreached);
        RegimeDecision {
            regime: Regime::Unknown,
            regime_confidence: 0.30,
            reason_codes,
        }
    }
}
