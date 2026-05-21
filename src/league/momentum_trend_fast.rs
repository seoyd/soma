use crate::core::{MarketSnapshot, ReasonCode, Regime, SignalOutput, Stance};

use super::{
    persona::{Persona, base_principles, build_vote},
    persona_card::{PersonaCard, momentum_trend_fast_card},
};

#[derive(Clone, Debug)]
pub struct MomentumTrendFast {
    pub base_voice: f64,
}

impl Default for MomentumTrendFast {
    fn default() -> Self {
        Self { base_voice: 0.78 }
    }
}

impl Persona for MomentumTrendFast {
    fn card(&self) -> PersonaCard {
        momentum_trend_fast_card(self.base_voice)
    }

    fn vote(&self, market: &MarketSnapshot, signal: &SignalOutput) -> crate::core::InvestorVote {
        let card = self.card();
        let bullish = matches!(market.regime, Regime::TrendUp | Regime::RiskOn)
            && signal.expected_return > 0.002
            && signal.confidence
                > card
                    .mutable_policy
                    .confidence_entry_threshold
                    .unwrap_or(0.48)
            && signal.no_trade_probability < 0.62;
        let bearish = matches!(market.regime, Regime::TrendDown | Regime::RiskOff)
            && signal.confidence > 0.52
            && signal.no_trade_probability < 0.58;

        if bullish {
            let conviction =
                (0.45 + signal.confidence * 0.35 + signal.p_win * 0.20).clamp(0.0, 1.0);
            let voice = card.voice.current_voice_power * conviction;
            build_vote(
                "momentum_trend_fast",
                "trend",
                Stance::Buy,
                conviction,
                voice,
                false,
                base_principles(
                    signal.p_win - signal.p_stop,
                    0.95,
                    (market.trade_value / 1_000_000.0).clamp(0.0, 1.0),
                    (1.0 - signal.expected_drawdown / 0.08).clamp(0.0, 1.0),
                    (1.0 - market.volatility / 0.05).clamp(0.0, 1.0),
                    (1.0 - market.spread_bps / 15.0).clamp(0.0, 1.0),
                ),
                signal.expected_return * 0.15,
                (market.spread_bps / 20.0).clamp(0.0, 1.0) * 0.25,
                vec![ReasonCode::TrendAlignment],
            )
        } else if bearish {
            let conviction = (0.40 + signal.confidence * 0.30).clamp(0.0, 1.0);
            let voice = card.voice.current_voice_power * conviction;
            build_vote(
                "momentum_trend_fast",
                "trend",
                Stance::Sell,
                conviction,
                voice,
                false,
                base_principles(
                    signal.p_stop - signal.p_win,
                    0.85,
                    (market.trade_value / 1_000_000.0).clamp(0.0, 1.0),
                    (1.0 - signal.expected_drawdown / 0.08).clamp(0.0, 1.0),
                    (1.0 - market.volatility / 0.05).clamp(0.0, 1.0),
                    (1.0 - market.spread_bps / 15.0).clamp(0.0, 1.0),
                ),
                signal.expected_return.abs() * 0.05,
                (market.spread_bps / 20.0).clamp(0.0, 1.0) * 0.30,
                vec![ReasonCode::TrendAlignment],
            )
        } else {
            build_vote(
                "momentum_trend_fast",
                "trend",
                Stance::NoTrade,
                0.25,
                card.voice.current_voice_power * 0.25,
                false,
                base_principles(0.0, 0.20, 0.35, 0.55, 0.40, 0.40),
                0.0,
                0.20,
                vec![ReasonCode::RegimeMismatch, ReasonCode::NoTradePreferred],
            )
        }
    }
}
