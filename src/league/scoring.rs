use crate::core::PersonaTier;

pub use super::evaluation::{
    HypotheticalTradeOutcome, SurvivalScoreComponents as SurvivalComponents,
};

pub fn silence_value(outcome: HypotheticalTradeOutcome) -> f64 {
    super::evaluation::silence_value_score(outcome)
}

pub fn survival_score(components: SurvivalComponents) -> f64 {
    super::evaluation::composite_survival_score(components)
}

pub fn update_voice_power(current: f64, normalized_survival_score: f64) -> f64 {
    super::voice::update_voice_power(current, normalized_survival_score, false)
}

pub fn violation_outcome(
    tier: PersonaTier,
    severe_doctrine_violation: bool,
    risk_bypass_attempt: bool,
) -> PersonaTier {
    super::tier::violation_outcome(tier, severe_doctrine_violation, risk_bypass_attempt)
}
