use serde::{Deserialize, Serialize};

use crate::core::{PersonaTier, Regime};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Horizon {
    Intraday,
    Swing,
    Position,
}

impl Horizon {
    pub fn accepts_bars(self, bars: u32) -> bool {
        match self {
            Self::Intraday => bars <= 12,
            Self::Swing => (13..=72).contains(&bars),
            Self::Position => bars >= 73,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ImmutableDoctrine {
    pub never_average_down: bool,
    pub cut_losses_quickly: bool,
    pub pyramid_only_on_strength: bool,
    pub speak_only_on_trend_or_breakout: bool,
    pub rest_after_consecutive_losses: bool,
    pub no_leverage: bool,
    pub do_not_speak_intraday_as_entry_signal: bool,
    pub reject_unknown_or_unscorable_asset: bool,
    pub margin_of_safety_required_when_fundamentals_available: bool,
    pub risk_first: bool,
    pub reject_poor_risk_reward: bool,
    pub reject_euphoria_chasing: bool,
    pub respect_cooldown: bool,
    pub no_trade_is_valid: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct MutablePolicy {
    pub breakout_lookback: Option<u32>,
    pub volume_z_threshold: Option<f64>,
    pub stop_loss_atr_mult: Option<f64>,
    pub take_profit_rr: Option<f64>,
    pub confidence_entry_threshold: Option<f64>,
    pub max_trade_frequency: Option<u32>,
    pub max_exposure_hint: Option<f64>,
    pub unknown_asset_penalty: Option<f64>,
    pub quality_threshold_placeholder: Option<f64>,
    pub defensive_bias: Option<f64>,
    pub overheat_threshold: Option<f64>,
    pub min_risk_reward: Option<f64>,
    pub volatility_penalty: Option<f64>,
    pub groupthink_penalty: Option<f64>,
    pub veto_sensitivity: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub base_voice_power: f64,
    pub current_voice_power: f64,
    pub ema_alpha: f64,
    pub severe_event_multiplier: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvaluationProfile {
    pub horizon: Horizon,
    pub favored_regimes: Vec<Regime>,
    pub tolerated_regimes: Vec<Regime>,
    pub promotion_min_samples: u32,
    pub max_s_tier: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaCard {
    pub persona_id: String,
    pub archetype: String,
    pub tier: PersonaTier,
    pub immutable_doctrine: ImmutableDoctrine,
    pub mutable_policy: MutablePolicy,
    pub voice: VoiceConfig,
    pub evaluation: EvaluationProfile,
}

pub fn horizon_from_bars(horizon_bars: u32) -> Horizon {
    match horizon_bars {
        0..=12 => Horizon::Intraday,
        13..=72 => Horizon::Swing,
        _ => Horizon::Position,
    }
}

pub fn momentum_trend_fast_card(base_voice_power: f64) -> PersonaCard {
    PersonaCard {
        persona_id: "momentum_trend_fast".to_string(),
        archetype: "Livermore-like trend/momentum delegate".to_string(),
        tier: PersonaTier::B,
        immutable_doctrine: ImmutableDoctrine {
            never_average_down: true,
            cut_losses_quickly: true,
            pyramid_only_on_strength: true,
            speak_only_on_trend_or_breakout: true,
            rest_after_consecutive_losses: true,
            ..ImmutableDoctrine::default()
        },
        mutable_policy: MutablePolicy {
            breakout_lookback: Some(20),
            volume_z_threshold: Some(1.25),
            stop_loss_atr_mult: Some(1.6),
            take_profit_rr: Some(2.0),
            confidence_entry_threshold: Some(0.52),
            max_trade_frequency: Some(4),
            ..MutablePolicy::default()
        },
        voice: VoiceConfig {
            base_voice_power,
            current_voice_power: base_voice_power,
            ema_alpha: 0.08,
            severe_event_multiplier: 0.5,
        },
        evaluation: EvaluationProfile {
            horizon: Horizon::Intraday,
            favored_regimes: vec![Regime::TrendUp, Regime::RiskOn],
            tolerated_regimes: vec![Regime::TrendDown, Regime::Range],
            promotion_min_samples: 16,
            max_s_tier: 1,
        },
    }
}

pub fn value_quality_filter_card(base_voice_power: f64) -> PersonaCard {
    PersonaCard {
        persona_id: "value_quality_filter".to_string(),
        archetype: "Graham/Buffett-like defensive filter".to_string(),
        tier: PersonaTier::C,
        immutable_doctrine: ImmutableDoctrine {
            no_leverage: true,
            do_not_speak_intraday_as_entry_signal: true,
            reject_unknown_or_unscorable_asset: true,
            margin_of_safety_required_when_fundamentals_available: true,
            ..ImmutableDoctrine::default()
        },
        mutable_policy: MutablePolicy {
            max_exposure_hint: Some(0.35),
            unknown_asset_penalty: Some(0.40),
            quality_threshold_placeholder: Some(0.60),
            defensive_bias: Some(0.70),
            ..MutablePolicy::default()
        },
        voice: VoiceConfig {
            base_voice_power,
            current_voice_power: base_voice_power,
            ema_alpha: 0.08,
            severe_event_multiplier: 0.5,
        },
        evaluation: EvaluationProfile {
            horizon: Horizon::Position,
            favored_regimes: vec![Regime::Range, Regime::RiskOff],
            tolerated_regimes: vec![Regime::TrendUp],
            promotion_min_samples: 18,
            max_s_tier: 1,
        },
    }
}

pub fn cycle_risk_skeptic_card(base_voice_power: f64) -> PersonaCard {
    PersonaCard {
        persona_id: "cycle_risk_skeptic".to_string(),
        archetype: "Howard Marks / PTJ-like risk and cycle skeptic".to_string(),
        tier: PersonaTier::A,
        immutable_doctrine: ImmutableDoctrine {
            risk_first: true,
            reject_poor_risk_reward: true,
            reject_euphoria_chasing: true,
            respect_cooldown: true,
            no_trade_is_valid: true,
            ..ImmutableDoctrine::default()
        },
        mutable_policy: MutablePolicy {
            overheat_threshold: Some(0.75),
            min_risk_reward: Some(1.8),
            volatility_penalty: Some(0.55),
            groupthink_penalty: Some(0.35),
            veto_sensitivity: Some(0.65),
            ..MutablePolicy::default()
        },
        voice: VoiceConfig {
            base_voice_power,
            current_voice_power: base_voice_power,
            ema_alpha: 0.08,
            severe_event_multiplier: 0.5,
        },
        evaluation: EvaluationProfile {
            horizon: Horizon::Swing,
            favored_regimes: vec![
                Regime::HighVolatility,
                Regime::Panic,
                Regime::RiskOff,
                Regime::Unknown,
            ],
            tolerated_regimes: vec![Regime::Range],
            promotion_min_samples: 14,
            max_s_tier: 1,
        },
    }
}

pub fn active_persona_cards() -> Vec<PersonaCard> {
    vec![
        momentum_trend_fast_card(0.78),
        value_quality_filter_card(0.48),
        cycle_risk_skeptic_card(0.70),
    ]
}

pub fn persona_card_by_id(persona_id: &str) -> Option<PersonaCard> {
    match persona_id {
        "momentum_trend_fast" => Some(momentum_trend_fast_card(0.78)),
        "value_quality_filter" => Some(value_quality_filter_card(0.48)),
        "cycle_risk_skeptic" => Some(cycle_risk_skeptic_card(0.70)),
        _ => None,
    }
}
