use crate::backtest::CostModel;
use crate::core::{Regime, SignalOutput};
use crate::feature::{FeatureName, FeatureVector};
use crate::regime::RegimeDecision;

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineSignalConfig {
    pub min_data_quality: f64,
    pub max_spread_bps: f64,
    pub base_horizon_bars: u32,
}

impl Default for BaselineSignalConfig {
    fn default() -> Self {
        Self {
            min_data_quality: 0.60,
            max_spread_bps: 12.0,
            base_horizon_bars: 8,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BaselineSignalModel {
    pub config: BaselineSignalConfig,
}

impl Default for BaselineSignalModel {
    fn default() -> Self {
        Self {
            config: BaselineSignalConfig::default(),
        }
    }
}

impl BaselineSignalModel {
    pub fn evaluate(
        &self,
        features: &FeatureVector,
        regime: &RegimeDecision,
        cost_model: &CostModel,
    ) -> SignalOutput {
        let data_quality = features.data_quality_score;
        let spread_bps = features
            .value(FeatureName::SpreadBps)
            .or_else(|| features.value(FeatureName::SpreadBpsFromCandle))
            .unwrap_or(10.0);
        let momentum = features.value(FeatureName::LogReturn3).unwrap_or(0.0);
        let close_over_ma20 = features.value(FeatureName::CloseOverMa20).unwrap_or(1.0);
        let volume_z = features.value(FeatureName::VolumeZ20).unwrap_or(0.0);
        let realized_vol = features.value(FeatureName::RealizedVol20).unwrap_or(0.0);
        let liquidity = features
            .value(FeatureName::LiquidityScoreHeuristic)
            .unwrap_or(0.3);

        let quality_penalty = (1.0 - data_quality).max(0.0);
        let spread_penalty = clamp01(spread_bps / self.config.max_spread_bps.max(1.0));
        let regime_conf = regime.regime_confidence;

        let mut expected_return = 0.002
            + 0.010 * momentum.max(0.0)
            + 0.006 * (close_over_ma20 - 1.0).max(0.0)
            + 0.004 * volume_z.max(0.0)
            + 0.004 * liquidity
            - 0.010 * spread_penalty
            - 0.012 * quality_penalty;
        let mut expected_drawdown =
            0.006 + 0.040 * realized_vol + 0.018 * spread_penalty + 0.020 * quality_penalty;
        let mut confidence =
            0.25 + 0.40 * regime_conf + 0.20 * data_quality + 0.08 * volume_z.max(0.0)
                - 0.18 * spread_penalty
                - 0.12 * realized_vol;
        let mut no_trade_probability =
            0.72 + 0.18 * quality_penalty + 0.16 * spread_penalty + 0.12 * realized_vol
                - 0.18 * regime_conf;

        match regime.regime {
            Regime::Unknown | Regime::Panic => {
                expected_return *= 0.25;
                expected_drawdown += 0.02;
                confidence *= 0.55;
                no_trade_probability += 0.20;
            }
            Regime::HighVolatility => {
                expected_return *= 0.7;
                expected_drawdown += 0.01;
                confidence *= 0.80;
                no_trade_probability += 0.12;
            }
            Regime::Range => {
                expected_return *= 0.75;
                confidence *= 0.85;
                no_trade_probability += 0.08;
            }
            Regime::TrendUp | Regime::RiskOn => {
                if momentum > 0.0 && volume_z > 0.2 && spread_bps <= self.config.max_spread_bps {
                    expected_return += 0.006;
                    confidence += 0.08;
                    no_trade_probability -= 0.18;
                }
            }
            Regime::TrendDown | Regime::RiskOff => {
                expected_return -= 0.004;
                expected_drawdown += 0.005;
                no_trade_probability += 0.08;
            }
        }

        let edge_after_cost = cost_model.expected_edge_after_cost(expected_return);
        if data_quality < self.config.min_data_quality || edge_after_cost <= 0.0 {
            no_trade_probability += 0.18;
            confidence *= 0.8;
        }

        SignalOutput {
            symbol: features.symbol.clone(),
            horizon_bars: match regime.regime {
                Regime::Panic | Regime::HighVolatility => 3,
                Regime::Range => 4,
                _ => self.config.base_horizon_bars,
            },
            p_win: clamp01(
                0.50 + expected_return * 6.0 + confidence * 0.12 - no_trade_probability * 0.10,
            ),
            p_stop: clamp01(
                0.22 + expected_drawdown * 4.0 + spread_penalty * 0.12 + quality_penalty * 0.10,
            ),
            expected_return: expected_return.clamp(-0.05, 0.05),
            expected_drawdown: expected_drawdown.clamp(0.002, 0.10),
            confidence: clamp01(confidence),
            no_trade_probability: clamp01(no_trade_probability),
            source: "baseline_rule_v0".to_string(),
        }
    }
}
