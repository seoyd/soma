use crate::core::{ReasonCode, Regime};
use crate::data::EvidenceSourceKind;
use crate::feature::FeatureName;

use super::persona_card_lite::{
    PersonaCardLite, PersonaHorizon, cycle_regime_guard_card, defensive_value_risk_card,
    trend_breakout_fast_card,
};
use super::persona_scorer::{PersonaScorer, PersonaScoringInput};
use super::persona_vote::{PersonaStance, PersonaVote};

#[derive(Clone, Debug, Default)]
pub struct TrendBreakoutFastScorer;

#[derive(Clone, Debug, Default)]
pub struct DefensiveValueRiskScorer;

#[derive(Clone, Debug, Default)]
pub struct CycleRegimeGuardScorer;

impl PersonaScorer for TrendBreakoutFastScorer {
    fn card(&self) -> PersonaCardLite {
        trend_breakout_fast_card()
    }

    fn score(&self, input: &PersonaScoringInput) -> PersonaVote {
        let card = self.card();
        let spread = input.spread_bps.unwrap_or(0.0);
        let breakout = feature_value(input, FeatureName::CloseOverMa20)
            .or_else(|| feature_value(input, FeatureName::CloseOverVwap20))
            .unwrap_or(0.5);
        let volume_z = feature_value(input, FeatureName::VolumeZ20).unwrap_or(0.0);
        let doctrine_violations = doctrine_checks(
            &card,
            input,
            spread > card.mutable_policy.max_spread_bps,
            input.data_quality_score < card.mutable_policy.min_data_quality,
        );
        let hard_violation = !doctrine_violations.is_empty();
        let stance = if hard_violation {
            PersonaStance::Veto
        } else if !source_allowed(&card, input.source_kind)
            || input.target_horizon > PersonaHorizon::Swing
        {
            PersonaStance::Abstain
        } else if matches!(
            input.regime,
            Regime::TrendUp | Regime::RiskOn | Regime::Range
        ) && input.expected_edge_after_cost > 0.0
            && input.signal_output.confidence >= card.mutable_policy.confidence_floor
            && volume_z >= 1.0
            && breakout >= 0.55
        {
            if input.signal_output.confidence >= 0.75 && volume_z >= 1.5 {
                PersonaStance::StrongApprove
            } else {
                PersonaStance::Approve
            }
        } else if input.signal_output.confidence < 0.45
            || spread > card.mutable_policy.max_spread_bps
        {
            PersonaStance::NoTrade
        } else if input.expected_edge_after_cost > 0.0 && input.signal_output.confidence >= 0.50 {
            PersonaStance::ReduceSize
        } else {
            PersonaStance::NoTrade
        };
        vote(
            &card,
            stance,
            input.source_kind,
            regime_fit(input.regime, &card),
            input.data_quality_score,
            risk_fit(input.expected_drawdown, 0.04),
            edge_fit(input.expected_edge_after_cost),
            doctrine_violations,
            vec![ReasonCode::PersonaVoteBuilt],
        )
    }
}

impl PersonaScorer for DefensiveValueRiskScorer {
    fn card(&self) -> PersonaCardLite {
        defensive_value_risk_card()
    }

    fn score(&self, input: &PersonaScoringInput) -> PersonaVote {
        let card = self.card();
        let spread = input.spread_bps.unwrap_or(0.0);
        let poor_quality = input.data_quality_score < card.mutable_policy.min_data_quality;
        let weak_risk_reward = input.expected_edge_after_cost <= input.expected_drawdown.max(0.001);
        let doctrine_violations =
            doctrine_checks(&card, input, false, poor_quality || weak_risk_reward);
        let hard_violation = !doctrine_violations.is_empty();
        let stance = if hard_violation || input.expected_edge_after_cost <= 0.0 {
            PersonaStance::Veto
        } else if !source_allowed(&card, input.source_kind) {
            PersonaStance::Abstain
        } else if poor_quality || spread > card.mutable_policy.max_spread_bps {
            PersonaStance::NoTrade
        } else if weak_risk_reward
            || input.signal_output.confidence < card.mutable_policy.confidence_floor
        {
            PersonaStance::ReduceSize
        } else if input.expected_drawdown < 0.03 && input.signal_output.confidence >= 0.60 {
            PersonaStance::Approve
        } else {
            PersonaStance::NoTrade
        };
        vote(
            &card,
            stance,
            input.source_kind,
            regime_fit(input.regime, &card),
            input.data_quality_score,
            risk_fit(input.expected_drawdown, 0.03),
            edge_fit(input.expected_edge_after_cost),
            doctrine_violations,
            vec![ReasonCode::PersonaVoteBuilt],
        )
    }
}

impl PersonaScorer for CycleRegimeGuardScorer {
    fn card(&self) -> PersonaCardLite {
        cycle_regime_guard_card()
    }

    fn score(&self, input: &PersonaScoringInput) -> PersonaVote {
        let card = self.card();
        let spread = input.spread_bps.unwrap_or(0.0);
        let volatility = input
            .risk_snapshot
            .as_ref()
            .map(|_| input.expected_drawdown)
            .unwrap_or(input.expected_drawdown);
        let doctrine_violations = doctrine_checks(
            &card,
            input,
            matches!(input.regime, Regime::Panic | Regime::Unknown)
                && input.data_quality_score < card.mutable_policy.min_data_quality,
            false,
        );
        let hard_violation = !doctrine_violations.is_empty();
        let stance = if hard_violation {
            PersonaStance::Veto
        } else if !source_allowed(&card, input.source_kind) {
            PersonaStance::Abstain
        } else if matches!(input.regime, Regime::Panic | Regime::Unknown)
            && input.data_quality_score < card.mutable_policy.min_data_quality
        {
            PersonaStance::Veto
        } else if volatility > card.mutable_policy.volatility_limit
            || spread > card.mutable_policy.max_spread_bps
        {
            PersonaStance::ReduceSize
        } else if matches!(input.regime, Regime::TrendUp | Regime::RiskOn)
            && input.expected_edge_after_cost > 0.0
        {
            PersonaStance::Approve
        } else {
            PersonaStance::NoTrade
        };
        vote(
            &card,
            stance,
            input.source_kind,
            regime_fit(input.regime, &card),
            input.data_quality_score,
            risk_fit(input.expected_drawdown, 0.035),
            edge_fit(input.expected_edge_after_cost),
            doctrine_violations,
            vec![ReasonCode::PersonaVoteBuilt],
        )
    }
}

pub fn active_trinity_scorers() -> Vec<Box<dyn PersonaScorer>> {
    vec![
        Box::new(TrendBreakoutFastScorer),
        Box::new(DefensiveValueRiskScorer),
        Box::new(CycleRegimeGuardScorer),
    ]
}

fn vote(
    card: &PersonaCardLite,
    stance: PersonaStance,
    source_kind: EvidenceSourceKind,
    regime_fit: f64,
    data_quality_fit: f64,
    risk_fit: f64,
    expected_edge_fit: f64,
    doctrine_violations: Vec<String>,
    mut reason_codes: Vec<ReasonCode>,
) -> PersonaVote {
    if !doctrine_violations.is_empty() {
        reason_codes.push(ReasonCode::DoctrineViolation);
    }
    PersonaVote {
        persona_id: card.persona_id.clone(),
        stance,
        conviction: (0.35 + regime_fit * 0.20 + data_quality_fit * 0.20 + expected_edge_fit * 0.25)
            .clamp(0.0, 1.0),
        voice_power: card
            .base_voice_power
            .min(card.mutable_policy.max_voice_power)
            .clamp(0.0, 1.0),
        horizon: card.horizon,
        source_kind,
        regime_fit,
        data_quality_fit: data_quality_fit.clamp(0.0, 1.0),
        risk_fit,
        expected_edge_fit,
        doctrine_violations,
        reason_codes,
    }
    .bounded()
}

fn regime_fit(regime: Regime, card: &PersonaCardLite) -> f64 {
    if card.regime_specialties.contains(&regime) {
        1.0
    } else {
        0.45
    }
}

fn edge_fit(expected_edge_after_cost: f64) -> f64 {
    (expected_edge_after_cost / 0.02).clamp(0.0, 1.0)
}

fn risk_fit(expected_drawdown: f64, limit: f64) -> f64 {
    (1.0 - expected_drawdown / limit.max(0.001)).clamp(0.0, 1.0)
}

fn source_allowed(card: &PersonaCardLite, source_kind: EvidenceSourceKind) -> bool {
    card.source_compatibility.contains(&source_kind)
}

fn doctrine_checks(
    card: &PersonaCardLite,
    input: &PersonaScoringInput,
    severe_regime_violation: bool,
    severe_quality_violation: bool,
) -> Vec<String> {
    let mut violations = Vec::new();
    if severe_regime_violation {
        violations.push(format!("{}:regime-hard-stop", card.persona_id));
    }
    if severe_quality_violation {
        violations.push(format!("{}:quality-hard-stop", card.persona_id));
    }
    if matches!(input.source_kind, EvidenceSourceKind::YFinanceResearch)
        && !source_allowed(card, input.source_kind)
    {
        violations.push(format!("{}:source-incompatible", card.persona_id));
    }
    violations
}

fn feature_value(input: &PersonaScoringInput, name: FeatureName) -> Option<f64> {
    input
        .feature_vector
        .as_ref()
        .and_then(|vector| vector.value(name))
}
