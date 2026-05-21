use crate::core::{InvestorVote, MarketSnapshot, ReasonCode, SignalOutput, SixPrinciples, Stance};

use super::{
    doctrine::{DoctrineCheck, DoctrineObservation, check_doctrine},
    persona_card::PersonaCard,
};

pub trait Persona {
    fn card(&self) -> PersonaCard;
    fn vote(&self, market: &MarketSnapshot, signal: &SignalOutput) -> InvestorVote;

    fn doctrine_check(&self, observation: &DoctrineObservation) -> DoctrineCheck {
        check_doctrine(&self.card(), observation)
    }
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

pub fn base_principles(
    signal_edge: f64,
    regime_fit: f64,
    liquidity_fit: f64,
    loss_protection: f64,
    event_risk: f64,
    execution_quality: f64,
) -> SixPrinciples {
    SixPrinciples {
        signal_edge: clamp01(signal_edge),
        regime_fit: clamp01(regime_fit),
        liquidity_fit: clamp01(liquidity_fit),
        loss_protection: clamp01(loss_protection),
        event_risk: clamp01(event_risk),
        execution_quality: clamp01(execution_quality),
    }
}

pub fn build_vote(
    persona_id: &str,
    cluster_id: &str,
    stance: Stance,
    conviction: f64,
    voice_power: f64,
    veto: bool,
    six_principles: SixPrinciples,
    expected_return_adjustment: f64,
    risk_penalty: f64,
    reason_codes: Vec<ReasonCode>,
) -> InvestorVote {
    InvestorVote {
        persona_id: persona_id.to_string(),
        cluster_id: cluster_id.to_string(),
        stance,
        conviction: clamp01(conviction),
        voice_power: clamp01(voice_power),
        veto,
        six_principles,
        expected_return_adjustment,
        risk_penalty: clamp01(risk_penalty),
        reason_codes,
    }
}

pub fn is_speculative_symbol(symbol: &str) -> bool {
    let upper = symbol.to_ascii_uppercase();
    ["BTC", "ETH", "SOL", "DOGE", "PEPE", "USDT", "PERP"]
        .iter()
        .any(|token| upper.contains(token))
}
