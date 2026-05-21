use crate::core::{FeatureVector, MarketSnapshot, Regime, SignalOutput};
use crate::signal::derive_features;

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

#[derive(Clone, Debug)]
pub struct MockSignalEngine {
    pub base_horizon_bars: u32,
}

impl Default for MockSignalEngine {
    fn default() -> Self {
        Self {
            base_horizon_bars: 8,
        }
    }
}

impl MockSignalEngine {
    pub fn evaluate(&self, market: &MarketSnapshot) -> SignalOutput {
        let features = derive_features(market);
        self.evaluate_with_features(market, &features)
    }

    pub fn evaluate_with_features(
        &self,
        market: &MarketSnapshot,
        features: &FeatureVector,
    ) -> SignalOutput {
        let edge_bias = 0.025 * features.regime_bias
            + 0.018 * features.trend_strength
            + 0.010 * features.breakout_score
            - 0.020 * features.volatility_score
            - 0.012 * features.spread_penalty;
        let expected_return = (edge_bias + 0.006 * features.liquidity_score).clamp(-0.05, 0.05);
        let expected_drawdown = (0.006
            + 0.040 * features.volatility_score
            + 0.020 * features.spread_penalty
            + 0.015 * features.overheat_score)
            .clamp(0.002, 0.08);
        let confidence = clamp01(
            0.35 + 0.35 * features.data_quality_score
                + 0.20 * features.liquidity_score
                + 0.15 * features.breakout_score
                - 0.15 * features.volatility_score
                - 0.10 * features.spread_penalty,
        );
        let p_win = clamp01(
            0.50 + expected_return * 8.0 + confidence * 0.15 - features.no_trade_bias * 0.10,
        );
        let p_stop = clamp01(
            0.25 + expected_drawdown * 5.0 + features.overheat_score * 0.20 - expected_return * 2.0,
        );
        let no_trade_probability = clamp01(
            0.65 + features.no_trade_bias * 0.30
                + if expected_return <= 0.0 { 0.15 } else { -0.20 }
                - confidence * 0.35,
        );
        let horizon_bars = match market.regime {
            Regime::TrendUp | Regime::TrendDown => self.base_horizon_bars.max(6),
            Regime::Range => 4,
            Regime::HighVolatility | Regime::Panic => 3,
            Regime::RiskOn | Regime::RiskOff => self.base_horizon_bars,
            Regime::Unknown => 2,
        };

        SignalOutput {
            symbol: market.symbol.clone(),
            horizon_bars,
            p_win,
            p_stop,
            expected_return,
            expected_drawdown,
            confidence,
            no_trade_probability,
            source: "mock_signal_v0".to_string(),
        }
    }
}
