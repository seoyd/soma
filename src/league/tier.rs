use crate::core::PersonaTier;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TierAction {
    Keep,
    Promote,
    Demote,
    Quarantine,
}

pub fn tier_rank(tier: PersonaTier) -> usize {
    match tier {
        PersonaTier::S => 5,
        PersonaTier::A => 4,
        PersonaTier::B => 3,
        PersonaTier::C => 2,
        PersonaTier::D => 1,
        PersonaTier::XQuarantined => 0,
    }
}

pub fn promote_one_tier(tier: PersonaTier) -> PersonaTier {
    match tier {
        PersonaTier::S => PersonaTier::S,
        PersonaTier::A => PersonaTier::S,
        PersonaTier::B => PersonaTier::A,
        PersonaTier::C => PersonaTier::B,
        PersonaTier::D => PersonaTier::C,
        PersonaTier::XQuarantined => PersonaTier::XQuarantined,
    }
}

pub fn demote_one_tier(tier: PersonaTier) -> PersonaTier {
    match tier {
        PersonaTier::S => PersonaTier::A,
        PersonaTier::A => PersonaTier::B,
        PersonaTier::B => PersonaTier::C,
        PersonaTier::C => PersonaTier::D,
        PersonaTier::D | PersonaTier::XQuarantined => PersonaTier::XQuarantined,
    }
}

pub fn tier_from_voice_power(
    voice_power: f64,
    max_s_tier: usize,
    current_s_tier_count: usize,
) -> PersonaTier {
    if voice_power >= 0.80 && current_s_tier_count < max_s_tier {
        PersonaTier::S
    } else if voice_power >= 0.60 {
        PersonaTier::A
    } else if voice_power >= 0.40 {
        PersonaTier::B
    } else if voice_power >= 0.20 {
        PersonaTier::C
    } else {
        PersonaTier::D
    }
}

pub fn violation_outcome(
    tier: PersonaTier,
    severe_doctrine_violation: bool,
    risk_bypass_attempt: bool,
) -> PersonaTier {
    if severe_doctrine_violation || risk_bypass_attempt {
        PersonaTier::XQuarantined
    } else {
        demote_one_tier(tier)
    }
}
