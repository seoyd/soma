use crate::core::{MarketSnapshot, ReasonCode, Regime, SignalOutput, Stance};

use super::{
    persona::{Persona, base_principles, build_vote},
    persona_card::{PersonaCard, cycle_risk_skeptic_card},
};

#[derive(Clone, Debug)]
pub struct CycleRiskSkeptic {
    pub base_voice: f64,
}

impl Default for CycleRiskSkeptic {
    fn default() -> Self {
        Self { base_voice: 0.70 }
    }
}

impl Persona for CycleRiskSkeptic {
    fn card(&self) -> PersonaCard {
        cycle_risk_skeptic_card(self.base_voice)
    }

    fn vote(&self, market: &MarketSnapshot, signal: &SignalOutput) -> crate::core::InvestorVote {
        let card = self.card();
        let overheated = matches!(
            market.regime,
            Regime::HighVolatility | Regime::Panic | Regime::Unknown
        ) || signal.no_trade_probability >= 0.55
            || market.volatility >= 0.04;

        if overheated {
            build_vote(
                "cycle_risk_skeptic",
                "risk",
                Stance::NoTrade,
                0.92,
                card.voice.current_voice_power,
                true,
                base_principles(
                    0.10,
                    0.95,
                    (market.trade_value / 1_000_000.0).clamp(0.0, 1.0),
                    0.95,
                    0.95,
                    (1.0 - market.spread_bps / 20.0).clamp(0.0, 1.0),
                ),
                -signal.expected_return.abs() * 0.25,
                0.85,
                vec![ReasonCode::OverheatedMarket, ReasonCode::CycleSkepticVeto],
            )
        } else if signal.expected_drawdown >= signal.expected_return.max(0.001) {
            build_vote(
                "cycle_risk_skeptic",
                "risk",
                Stance::NoTrade,
                0.68,
                card.voice.current_voice_power * 0.78,
                false,
                base_principles(0.20, 0.80, 0.60, 0.85, 0.78, 0.55),
                -signal.expected_drawdown * 0.10,
                0.62,
                vec![ReasonCode::SkepticWarning],
            )
        } else {
            build_vote(
                "cycle_risk_skeptic",
                "risk",
                Stance::NoTrade,
                0.38,
                card.voice.current_voice_power * 0.40,
                false,
                base_principles(0.35, 0.55, 0.60, 0.70, 0.45, 0.60),
                -signal.expected_drawdown * 0.03,
                0.28,
                vec![ReasonCode::NoTradePreferred],
            )
        }
    }
}
