use crate::core::{ReasonCode, Stance};

use super::persona_card::PersonaCard;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoctrineViolation {
    AveragingDown,
    IntradayEntrySignal,
    UnknownAsset,
    MarginOfSafetyMissing,
    PoorRiskReward,
    EuphoriaChasing,
    CooldownIgnored,
    RiskBypassAttempt,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DoctrineObservation {
    pub stance: Stance,
    pub is_adding_to_loser: bool,
    pub is_intraday_signal: bool,
    pub asset_is_scorable: bool,
    pub fundamentals_available: bool,
    pub margin_of_safety_present: bool,
    pub risk_reward: f64,
    pub euphoria_score: f64,
    pub cooldown_active: bool,
    pub risk_bypass_attempt: bool,
}

impl Default for DoctrineObservation {
    fn default() -> Self {
        Self {
            stance: Stance::NoTrade,
            is_adding_to_loser: false,
            is_intraday_signal: false,
            asset_is_scorable: true,
            fundamentals_available: false,
            margin_of_safety_present: true,
            risk_reward: 2.0,
            euphoria_score: 0.0,
            cooldown_active: false,
            risk_bypass_attempt: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DoctrineCheck {
    pub violations: Vec<DoctrineViolation>,
    pub severe: bool,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn check_doctrine(card: &PersonaCard, observation: &DoctrineObservation) -> DoctrineCheck {
    let doctrine = &card.immutable_doctrine;
    let mut violations = Vec::new();
    let mut reason_codes = Vec::new();

    if doctrine.never_average_down && observation.is_adding_to_loser {
        violations.push(DoctrineViolation::AveragingDown);
        reason_codes.push(ReasonCode::AveragingDownRejected);
    }

    if doctrine.do_not_speak_intraday_as_entry_signal
        && observation.is_intraday_signal
        && matches!(observation.stance, Stance::Buy | Stance::Sell)
    {
        violations.push(DoctrineViolation::IntradayEntrySignal);
        reason_codes.push(ReasonCode::IntradayEntryForbidden);
    }

    if doctrine.reject_unknown_or_unscorable_asset && !observation.asset_is_scorable {
        violations.push(DoctrineViolation::UnknownAsset);
        reason_codes.push(ReasonCode::UnknownAssetRejected);
    }

    if doctrine.margin_of_safety_required_when_fundamentals_available
        && observation.fundamentals_available
        && !observation.margin_of_safety_present
    {
        violations.push(DoctrineViolation::MarginOfSafetyMissing);
        reason_codes.push(ReasonCode::MarginOfSafetyMissing);
    }

    if doctrine.reject_poor_risk_reward && observation.risk_reward < 1.5 {
        violations.push(DoctrineViolation::PoorRiskReward);
        reason_codes.push(ReasonCode::PoorRiskRewardRejected);
    }

    if doctrine.reject_euphoria_chasing && observation.euphoria_score > 0.7 {
        violations.push(DoctrineViolation::EuphoriaChasing);
        reason_codes.push(ReasonCode::EuphoriaRejected);
    }

    if doctrine.respect_cooldown
        && observation.cooldown_active
        && !matches!(observation.stance, Stance::NoTrade | Stance::Abstain)
    {
        violations.push(DoctrineViolation::CooldownIgnored);
        reason_codes.push(ReasonCode::CooldownRequired);
    }

    if observation.risk_bypass_attempt {
        violations.push(DoctrineViolation::RiskBypassAttempt);
        reason_codes.push(ReasonCode::RiskBypassAttempt);
    }

    let severe = observation.risk_bypass_attempt || violations.len() >= 2;
    if !violations.is_empty() {
        reason_codes.push(ReasonCode::DoctrineViolation);
    }
    if severe {
        reason_codes.push(ReasonCode::Quarantined);
    }

    DoctrineCheck {
        violations,
        severe,
        reason_codes,
    }
}

pub fn doctrine_consistency_score(violation_count: usize, severe: bool) -> f64 {
    if severe {
        0.0
    } else {
        (1.0 - violation_count as f64 * 0.35).clamp(0.0, 1.0)
    }
}

pub fn doctrine_violation_penalty(violation_count: usize, severe: bool) -> f64 {
    let base = (violation_count as f64 * 0.18).clamp(0.0, 0.72);
    if severe {
        (base + 0.28).clamp(0.0, 1.0)
    } else {
        base
    }
}
