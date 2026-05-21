use crate::core::{MarketSnapshot, ReasonCode, Regime, SignalOutput, Stance};

use super::{
    persona::{Persona, base_principles, build_vote, is_speculative_symbol},
    persona_card::{PersonaCard, value_quality_filter_card},
};

#[derive(Clone, Debug)]
pub struct ValueQualityFilter {
    pub base_voice: f64,
}

impl Default for ValueQualityFilter {
    fn default() -> Self {
        Self { base_voice: 0.48 }
    }
}

impl Persona for ValueQualityFilter {
    fn card(&self) -> PersonaCard {
        value_quality_filter_card(self.base_voice)
    }

    fn vote(&self, market: &MarketSnapshot, signal: &SignalOutput) -> crate::core::InvestorVote {
        let card = self.card();
        if signal.horizon_bars <= 12 {
            return build_vote(
                "value_quality_filter",
                "quality",
                Stance::NoTrade,
                0.20,
                card.voice.current_voice_power * 0.25,
                false,
                base_principles(0.0, 0.25, 0.40, 0.70, 0.45, 0.45),
                0.0,
                0.30,
                vec![
                    ReasonCode::IntradayEntryForbidden,
                    ReasonCode::QualityFilterAbstain,
                    ReasonCode::NoTradePreferred,
                ],
            );
        }

        if is_speculative_symbol(&market.symbol)
            || matches!(market.regime, Regime::Unknown | Regime::Panic)
        {
            return build_vote(
                "value_quality_filter",
                "quality",
                Stance::Abstain,
                0.0,
                0.0,
                false,
                base_principles(0.0, 0.10, 0.25, 0.60, 0.20, 0.30),
                0.0,
                0.35,
                vec![ReasonCode::QualityFilterAbstain],
            );
        }

        if signal.expected_return > 0.003
            && signal.confidence > 0.55
            && market.spread_bps <= 10.0
            && market.data_quality_score >= 0.80
        {
            let conviction = (0.35 + signal.confidence * 0.35).clamp(0.0, 1.0);
            let voice = card.voice.current_voice_power * conviction;
            build_vote(
                "value_quality_filter",
                "quality",
                Stance::Buy,
                conviction,
                voice,
                false,
                base_principles(
                    (signal.expected_return / 0.03).clamp(0.0, 1.0),
                    if matches!(
                        market.regime,
                        Regime::TrendUp | Regime::RiskOn | Regime::Range
                    ) {
                        0.75
                    } else {
                        0.40
                    },
                    (market.trade_value / 800_000.0).clamp(0.0, 1.0),
                    (1.0 - signal.expected_drawdown / 0.08).clamp(0.0, 1.0),
                    (1.0 - market.volatility / 0.06).clamp(0.0, 1.0),
                    (1.0 - market.spread_bps / 15.0).clamp(0.0, 1.0),
                ),
                signal.expected_return * 0.10,
                0.18,
                vec![ReasonCode::QualityFilterPass],
            )
        } else {
            build_vote(
                "value_quality_filter",
                "quality",
                Stance::NoTrade,
                0.45,
                card.voice.current_voice_power * 0.45,
                false,
                base_principles(0.20, 0.45, 0.50, 0.65, 0.45, 0.50),
                -signal.expected_drawdown * 0.05,
                0.32,
                vec![ReasonCode::NoTradePreferred],
            )
        }
    }
}
