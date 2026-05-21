use std::collections::BTreeMap;

use crate::backtest::OutcomeRecord;
use crate::core::{PersonaTier, ReasonCode, Regime};

use super::{
    doctrine::{doctrine_consistency_score, doctrine_violation_penalty},
    persona_card::PersonaCard,
    persona_card::{EvaluationProfile, Horizon},
    tier::{TierAction, demote_one_tier, promote_one_tier, tier_from_voice_power, tier_rank},
    voice::update_voice_power,
};

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HypotheticalTradeOutcome {
    StopFirst(f64),
    TakeProfitFirst(f64),
    Neutral,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurvivalScoreComponents {
    pub drawdown_control: f64,
    pub risk_efficiency: f64,
    pub net_expectancy_after_cost: f64,
    pub calibration: f64,
    pub regime_fit: f64,
    pub silence_value: f64,
    pub doctrine_consistency: f64,
    pub overconfidence_penalty: f64,
    pub overtrade_penalty: f64,
    pub correlation_penalty: f64,
    pub doctrine_violation_penalty: f64,
}

pub fn calibration_score(confidence: f64, realized_win_rate: f64) -> f64 {
    clamp01(1.0 - (confidence - realized_win_rate).abs())
}

pub fn net_expectancy_after_cost_score(expectancy: f64) -> f64 {
    clamp01((expectancy + 0.02) / 0.08)
}

pub fn risk_efficiency_score(expectancy: f64, expected_drawdown: f64) -> f64 {
    if expected_drawdown <= 0.0 {
        1.0
    } else {
        clamp01(expectancy.max(0.0) / expected_drawdown.max(1e-9))
    }
}

pub fn drawdown_control_score(drawdown_pct: f64) -> f64 {
    clamp01(1.0 - drawdown_pct.max(0.0) / 0.25)
}

pub fn regime_fit_score(profile: &EvaluationProfile, regime: Regime, horizon: Horizon) -> f64 {
    let horizon_fit = if profile.horizon == horizon {
        1.0
    } else {
        0.65
    };
    let regime_fit = if profile.favored_regimes.contains(&regime) {
        1.0
    } else if profile.tolerated_regimes.contains(&regime) {
        0.75
    } else {
        0.55
    };
    clamp01((horizon_fit + regime_fit) * 0.5)
}

pub fn silence_value_score(outcome: HypotheticalTradeOutcome) -> f64 {
    match outcome {
        HypotheticalTradeOutcome::StopFirst(avoided_loss) => 0.7 * avoided_loss.max(0.0),
        HypotheticalTradeOutcome::TakeProfitFirst(missed_gain) => -0.2 * missed_gain.max(0.0),
        HypotheticalTradeOutcome::Neutral => 0.0,
    }
}

pub fn overconfidence_penalty(confidence: f64, realized_win_rate: f64) -> f64 {
    clamp01((confidence - realized_win_rate).max(0.0))
}

pub fn overtrade_penalty(trade_count: u32, max_trade_frequency: u32) -> f64 {
    if max_trade_frequency == 0 {
        0.0
    } else {
        clamp01(
            (trade_count.saturating_sub(max_trade_frequency) as f64) / max_trade_frequency as f64,
        )
    }
}

pub fn correlation_penalty(correlation: f64) -> f64 {
    clamp01((correlation - 0.60).max(0.0) / 0.40)
}

pub fn composite_survival_score(components: SurvivalScoreComponents) -> f64 {
    clamp01(
        0.22 * clamp01(components.drawdown_control)
            + 0.18 * clamp01(components.risk_efficiency)
            + 0.17 * clamp01(components.net_expectancy_after_cost)
            + 0.15 * clamp01(components.calibration)
            + 0.12 * clamp01(components.regime_fit)
            + 0.10 * clamp01(components.silence_value)
            + 0.06 * clamp01(components.doctrine_consistency)
            - clamp01(components.overconfidence_penalty)
            - clamp01(components.overtrade_penalty)
            - clamp01(components.correlation_penalty)
            - clamp01(components.doctrine_violation_penalty),
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct PersonaEvaluationInput {
    pub persona_id: String,
    pub tier: PersonaTier,
    pub current_voice_power: f64,
    pub sample_count: u32,
    pub survival_score_components: SurvivalScoreComponents,
    pub high_confidence_miss_count: u32,
    pub consecutive_bad_periods: u32,
    pub doctrine_violation_count: u32,
    pub severe_event: bool,
    pub regime: Regime,
    pub horizon: Horizon,
    pub evaluation_profile: EvaluationProfile,
    pub risk_bypass_attempt: bool,
    pub current_s_tier_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PersonaEvaluationOutput {
    pub survival_score: f64,
    pub next_voice_power: f64,
    pub current_tier: PersonaTier,
    pub next_tier: PersonaTier,
    pub action: TierAction,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn evaluate_persona(input: &PersonaEvaluationInput) -> PersonaEvaluationOutput {
    let mut reason_codes = Vec::new();
    let survival_score = composite_survival_score(input.survival_score_components);
    let next_voice_power = update_voice_power(
        input.current_voice_power,
        survival_score,
        input.severe_event,
    );
    let domain_fit = regime_fit_score(&input.evaluation_profile, input.regime, input.horizon);

    if input.risk_bypass_attempt || (input.severe_event && input.doctrine_violation_count > 0) {
        reason_codes.push(ReasonCode::RiskBypassAttempt);
        reason_codes.push(ReasonCode::Quarantined);
        return PersonaEvaluationOutput {
            survival_score,
            next_voice_power,
            current_tier: input.tier,
            next_tier: PersonaTier::XQuarantined,
            action: TierAction::Quarantine,
            reason_codes,
        };
    }

    let should_demote = input.high_confidence_miss_count >= 3
        || input.consecutive_bad_periods >= 2
        || survival_score < 0.25
        || (domain_fit < 0.60 && survival_score < 0.20);

    let promotion_ready = input.sample_count >= input.evaluation_profile.promotion_min_samples
        && survival_score >= 0.60
        && input.doctrine_violation_count == 0
        && input.consecutive_bad_periods == 0;

    let desired_tier = tier_from_voice_power(
        next_voice_power,
        input.evaluation_profile.max_s_tier,
        input.current_s_tier_count,
    );

    let (next_tier, action) = if should_demote {
        reason_codes.push(ReasonCode::DemotionApplied);
        if input.severe_event {
            reason_codes.push(ReasonCode::SevereDemotion);
        }
        (demote_one_tier(input.tier), TierAction::Demote)
    } else if promotion_ready && tier_rank(desired_tier) > tier_rank(input.tier) {
        reason_codes.push(ReasonCode::PromotionGranted);
        (promote_one_tier(input.tier), TierAction::Promote)
    } else {
        if tier_rank(desired_tier) > tier_rank(input.tier)
            && input.sample_count < input.evaluation_profile.promotion_min_samples
        {
            reason_codes.push(ReasonCode::PromotionInsufficientSamples);
        }
        (input.tier, TierAction::Keep)
    };

    PersonaEvaluationOutput {
        survival_score,
        next_voice_power,
        current_tier: input.tier,
        next_tier,
        action,
        reason_codes,
    }
}

pub fn build_persona_evaluation_inputs(
    outcome_records: &[OutcomeRecord],
    persona_profiles: &[PersonaCard],
) -> Vec<PersonaEvaluationInput> {
    let mut persona_outcomes: BTreeMap<String, Vec<&OutcomeRecord>> = BTreeMap::new();
    for outcome in outcome_records {
        for attribution in &outcome.attribution_records {
            persona_outcomes
                .entry(attribution.persona_id.clone())
                .or_default()
                .push(outcome);
        }
    }

    let current_s_tier_count = persona_profiles
        .iter()
        .filter(|profile| profile.tier == PersonaTier::S)
        .count();

    persona_profiles
        .iter()
        .map(|profile| {
            let records = persona_outcomes
                .get(&profile.persona_id)
                .cloned()
                .unwrap_or_default();
            let sample_count = records.len() as u32;
            let average_net_return = average(records.iter().map(|record| {
                if record.executed {
                    record.realized_net_return_pct
                } else {
                    record.avoided_loss_score + record.missed_gain_penalty
                }
            }));
            let average_drawdown = average(records.iter().map(|record| {
                record
                    .triple_barrier_result
                    .as_ref()
                    .or(record.hypothetical_result.as_ref())
                    .map(|result| result.max_adverse_excursion_pct)
                    .unwrap_or(0.0)
            }));
            let average_confidence = average(records.iter().map(|record| record.signal_confidence));
            let silence_raw = average(
                records
                    .iter()
                    .map(|record| record.avoided_loss_score + record.missed_gain_penalty),
            );
            let high_confidence_miss_count = records
                .iter()
                .filter(|record| {
                    record.signal_confidence >= 0.75
                        && if record.executed {
                            record.realized_net_return_pct < 0.0
                        } else {
                            record.avoided_loss_score + record.missed_gain_penalty < 0.0
                        }
                })
                .count() as u32;
            let doctrine_violation_count = records
                .iter()
                .filter(|record| record.reason_codes.contains(&ReasonCode::DoctrineViolation))
                .count() as u32;
            let severe_event = records.iter().any(|record| {
                record.reason_codes.contains(&ReasonCode::Quarantined)
                    || record.reason_codes.contains(&ReasonCode::RiskBypassAttempt)
            });
            let risk_bypass_attempt = records
                .iter()
                .any(|record| record.reason_codes.contains(&ReasonCode::RiskBypassAttempt));
            let realized_win_rate = if sample_count == 0 {
                0.5
            } else {
                records
                    .iter()
                    .filter(|record| {
                        if record.executed {
                            record.realized_net_return_pct > 0.0
                        } else {
                            record.avoided_loss_score > 0.0
                        }
                    })
                    .count() as f64
                    / sample_count as f64
            };
            let consecutive_bad_periods = trailing_negative_count(&records).min(3) as u32;
            let dominant_regime = dominant_regime(&records).unwrap_or(Regime::Unknown);

            let components = SurvivalScoreComponents {
                drawdown_control: drawdown_control_score(average_drawdown),
                risk_efficiency: risk_efficiency_score(
                    average_net_return,
                    average_drawdown.max(1e-9),
                ),
                net_expectancy_after_cost: net_expectancy_after_cost_score(average_net_return),
                calibration: if sample_count < 4 {
                    0.5
                } else {
                    calibration_score(average_confidence, realized_win_rate)
                },
                regime_fit: regime_fit_score(
                    &profile.evaluation,
                    dominant_regime,
                    profile.evaluation.horizon,
                ),
                silence_value: clamp01((silence_raw + 0.05) / 0.10),
                doctrine_consistency: doctrine_consistency_score(
                    doctrine_violation_count as usize,
                    severe_event,
                ),
                overconfidence_penalty: overconfidence_penalty(
                    average_confidence,
                    realized_win_rate,
                ),
                overtrade_penalty: overtrade_penalty(
                    sample_count,
                    profile
                        .mutable_policy
                        .max_trade_frequency
                        .unwrap_or(sample_count.max(1)),
                ),
                correlation_penalty: 0.0,
                doctrine_violation_penalty: doctrine_violation_penalty(
                    doctrine_violation_count as usize,
                    severe_event,
                ),
            };

            PersonaEvaluationInput {
                persona_id: profile.persona_id.clone(),
                tier: profile.tier,
                current_voice_power: profile.voice.current_voice_power,
                sample_count,
                survival_score_components: components,
                high_confidence_miss_count,
                consecutive_bad_periods,
                doctrine_violation_count,
                severe_event,
                regime: dominant_regime,
                horizon: profile.evaluation.horizon,
                evaluation_profile: profile.evaluation.clone(),
                risk_bypass_attempt,
                current_s_tier_count,
            }
        })
        .collect()
}

fn average(values: impl Iterator<Item = f64>) -> f64 {
    let mut total = 0.0;
    let mut count = 0usize;
    for value in values {
        total += value;
        count += 1;
    }
    if count == 0 {
        0.0
    } else {
        total / count as f64
    }
}

fn trailing_negative_count(records: &[&OutcomeRecord]) -> usize {
    let mut ordered = records.to_vec();
    ordered.sort_by_key(|record| record.timestamp_ms);
    let mut count = 0usize;
    for record in ordered.iter().rev() {
        let value = if record.executed {
            record.realized_net_return_pct
        } else {
            record.avoided_loss_score + record.missed_gain_penalty
        };
        if value < 0.0 {
            count += 1;
        } else {
            break;
        }
    }
    count
}

fn dominant_regime(records: &[&OutcomeRecord]) -> Option<Regime> {
    let mut counts: Vec<(Regime, usize)> = Vec::new();
    for record in records {
        if let Some((_, count)) = counts
            .iter_mut()
            .find(|(regime, _)| *regime == record.regime)
        {
            *count += 1;
        } else {
            counts.push((record.regime, 1));
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(regime, _)| regime)
}
