use soma_zero::core::{PersonaTier, ReasonCode, Regime};
use soma_zero::league::{
    Horizon, HypotheticalTradeOutcome, PersonaEvaluationInput, TierAction, active_persona_cards,
    composite_survival_score, evaluate_persona, silence_value, silence_value_score,
    tier_from_voice_power, update_voice_power,
};
use soma_zero::league::{SurvivalScoreComponents, regime_fit_score};

fn profile() -> soma_zero::league::EvaluationProfile {
    active_persona_cards()[0].evaluation.clone()
}

fn components() -> SurvivalScoreComponents {
    SurvivalScoreComponents {
        drawdown_control: 0.8,
        risk_efficiency: 0.7,
        net_expectancy_after_cost: 0.65,
        calibration: 0.66,
        regime_fit: 0.75,
        silence_value: 0.2,
        doctrine_consistency: 0.9,
        overconfidence_penalty: 0.05,
        overtrade_penalty: 0.02,
        correlation_penalty: 0.03,
        doctrine_violation_penalty: 0.0,
    }
}

fn evaluation_input() -> PersonaEvaluationInput {
    PersonaEvaluationInput {
        persona_id: "momentum_trend_fast".to_string(),
        tier: PersonaTier::B,
        current_voice_power: 0.45,
        sample_count: 20,
        survival_score_components: components(),
        high_confidence_miss_count: 0,
        consecutive_bad_periods: 0,
        doctrine_violation_count: 0,
        severe_event: false,
        regime: Regime::TrendUp,
        horizon: Horizon::Intraday,
        evaluation_profile: profile(),
        risk_bypass_attempt: false,
        current_s_tier_count: 0,
    }
}

#[test]
fn survival_score_is_clamped_to_unit_interval() {
    let mut input = components();
    input.drawdown_control = 5.0;
    input.risk_efficiency = 5.0;
    input.net_expectancy_after_cost = 5.0;
    input.calibration = 5.0;
    input.regime_fit = 5.0;
    input.silence_value = 5.0;
    input.doctrine_consistency = 5.0;
    let score = composite_survival_score(input);
    assert!((0.0..=1.0).contains(&score));
}

#[test]
fn doctrine_violation_sharply_lowers_survival_score() {
    let clean = composite_survival_score(components());
    let mut violated = components();
    violated.doctrine_violation_penalty = 0.70;
    let penalized = composite_survival_score(violated);
    assert!(penalized < clean);
}

#[test]
fn severe_doctrine_violation_sends_persona_to_quarantine() {
    let mut input = evaluation_input();
    input.doctrine_violation_count = 1;
    input.severe_event = true;

    let output = evaluate_persona(&input);
    assert_eq!(output.next_tier, PersonaTier::XQuarantined);
    assert_eq!(output.action, TierAction::Quarantine);
}

#[test]
fn voice_power_ema_is_stable_and_bounded() {
    let next = update_voice_power(0.95, 1.0);
    assert!((0.0..=1.0).contains(&next));
    assert!(next > 0.95);
}

#[test]
fn promotion_requires_minimum_sample_count() {
    let mut input = evaluation_input();
    input.sample_count = 4;
    input.current_voice_power = 0.75;
    let output = evaluate_persona(&input);
    assert_eq!(output.action, TierAction::Keep);
    assert!(
        output
            .reason_codes
            .contains(&ReasonCode::PromotionInsufficientSamples)
    );
}

#[test]
fn demotion_can_happen_faster_than_promotion() {
    let mut input = evaluation_input();
    input.sample_count = 4;
    input.high_confidence_miss_count = 3;
    let output = evaluate_persona(&input);
    assert_eq!(output.action, TierAction::Demote);
}

#[test]
fn repeated_high_confidence_misses_demote() {
    let mut input = evaluation_input();
    input.high_confidence_miss_count = 4;
    let output = evaluate_persona(&input);
    assert_eq!(output.action, TierAction::Demote);
    assert!(output.reason_codes.contains(&ReasonCode::DemotionApplied));
}

#[test]
fn no_trade_gets_positive_silence_value_when_stop_hits() {
    let value = silence_value_score(HypotheticalTradeOutcome::StopFirst(0.10));
    assert!(value > 0.0);
    assert_eq!(
        value,
        silence_value(HypotheticalTradeOutcome::StopFirst(0.10))
    );
}

#[test]
fn no_trade_gets_small_penalty_when_take_profit_hits() {
    let value = silence_value_score(HypotheticalTradeOutcome::TakeProfitFirst(0.10));
    assert!(value < 0.0);
    assert!(value > -0.05);
}

#[test]
fn regime_and_horizon_scoring_is_style_adjusted() {
    let profile = profile();
    let in_domain = regime_fit_score(&profile, Regime::TrendUp, Horizon::Intraday);
    let out_of_domain = regime_fit_score(&profile, Regime::Range, Horizon::Position);
    assert!(in_domain > out_of_domain);
    assert!(out_of_domain >= 0.5);
}

#[test]
fn same_input_produces_same_output() {
    let input = evaluation_input();
    let a = evaluate_persona(&input);
    let b = evaluate_persona(&input);
    assert_eq!(a, b);
}

#[test]
fn s_tier_is_capped() {
    let tier = tier_from_voice_power(0.95, 1, 1);
    assert_eq!(tier, PersonaTier::A);
}
