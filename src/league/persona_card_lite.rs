use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, Regime};
use crate::data::EvidenceSourceKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PersonaGroup {
    Fast,
    Slow,
    Risk,
    Crypto,
    ResearchOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PersonaHorizon {
    Intraday,
    Swing,
    MultiDay,
    LongTerm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PersonaRole {
    SignalTrigger,
    UniverseFilter,
    RiskSkeptic,
    RegimeGuard,
    ExecutionGuard,
    ResearchOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctrineRule {
    pub rule_id: String,
    pub description: String,
    pub hard: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaMutablePolicy {
    pub entry_threshold: f64,
    pub no_trade_threshold: f64,
    pub reduce_size_threshold: f64,
    pub max_voice_power: f64,
    pub min_data_quality: f64,
    pub max_spread_bps: f64,
    pub volatility_limit: f64,
    pub confidence_floor: f64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaCardLite {
    pub persona_id: String,
    pub archetype_label: String,
    pub group: PersonaGroup,
    pub horizon: PersonaHorizon,
    pub role: PersonaRole,
    pub immutable_doctrine: Vec<DoctrineRule>,
    pub mutable_policy: PersonaMutablePolicy,
    pub base_voice_power: f64,
    pub regime_specialties: Vec<Regime>,
    pub source_compatibility: Vec<EvidenceSourceKind>,
    pub active: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl PersonaMutablePolicy {
    pub fn is_bounded(&self) -> bool {
        (0.0..=1.0).contains(&self.entry_threshold)
            && (0.0..=1.0).contains(&self.no_trade_threshold)
            && (0.0..=1.0).contains(&self.reduce_size_threshold)
            && (0.0..=1.0).contains(&self.max_voice_power)
            && (0.0..=1.0).contains(&self.min_data_quality)
            && self.max_spread_bps >= 0.0
            && self.volatility_limit >= 0.0
            && (0.0..=1.0).contains(&self.confidence_floor)
    }
}

pub fn all_persona_cards_lite() -> Vec<PersonaCardLite> {
    let mut cards = vec![
        trend_breakout_fast_card(),
        defensive_value_risk_card(),
        cycle_regime_guard_card(),
    ];
    cards.extend(
        [
            (
                "quality_value_long",
                PersonaGroup::Slow,
                PersonaRole::UniverseFilter,
            ),
            (
                "box_breakout_fast",
                PersonaGroup::Fast,
                PersonaRole::SignalTrigger,
            ),
            (
                "systematic_turtle_fast",
                PersonaGroup::Fast,
                PersonaRole::SignalTrigger,
            ),
            (
                "btc_cycle_crypto",
                PersonaGroup::Crypto,
                PersonaRole::RegimeGuard,
            ),
            (
                "onchain_crypto",
                PersonaGroup::Crypto,
                PersonaRole::ResearchOnly,
            ),
            (
                "event_crypto",
                PersonaGroup::Crypto,
                PersonaRole::ResearchOnly,
            ),
        ]
        .into_iter()
        .map(|(persona_id, group, role)| PersonaCardLite {
            persona_id: persona_id.to_string(),
            archetype_label: "future archetype placeholder; not active in Sprint 32".to_string(),
            group,
            horizon: PersonaHorizon::Swing,
            role,
            immutable_doctrine: vec![],
            mutable_policy: default_policy(),
            base_voice_power: 0.0,
            regime_specialties: vec![],
            source_compatibility: vec![EvidenceSourceKind::OfficialApiCollected],
            active: false,
            reason_codes: vec![ReasonCode::PersonaCardLiteBuilt],
        }),
    );
    cards
}

pub fn active_persona_cards_lite() -> Vec<PersonaCardLite> {
    all_persona_cards_lite()
        .into_iter()
        .filter(|card| card.active)
        .collect()
}

pub fn persona_card_lite_by_id(persona_id: &str) -> Option<PersonaCardLite> {
    all_persona_cards_lite()
        .into_iter()
        .find(|card| card.persona_id == persona_id)
}

pub fn trend_breakout_fast_card() -> PersonaCardLite {
    PersonaCardLite {
        persona_id: "trend_breakout_fast".to_string(),
        archetype_label:
            "Livermore/Darvas-like archetype label; not a literal investor reproduction".to_string(),
        group: PersonaGroup::Fast,
        horizon: PersonaHorizon::Intraday,
        role: PersonaRole::SignalTrigger,
        immutable_doctrine: vec![
            doctrine("never_average_down", "Never average down.", true),
            doctrine("cut_loss_quickly", "Cut losses quickly.", true),
            doctrine(
                "only_trade_with_confirmed_momentum",
                "Only speak on confirmed momentum or breakout.",
                true,
            ),
            doctrine("no_trade_in_bad_liquidity", "Reject poor liquidity.", true),
        ],
        mutable_policy: PersonaMutablePolicy {
            entry_threshold: 0.58,
            no_trade_threshold: 0.48,
            reduce_size_threshold: 0.68,
            max_voice_power: 0.82,
            min_data_quality: 0.72,
            max_spread_bps: 18.0,
            volatility_limit: 0.06,
            confidence_floor: 0.55,
            reason_codes: vec![ReasonCode::PersonaCardLiteBuilt],
        },
        base_voice_power: 0.78,
        regime_specialties: vec![Regime::TrendUp, Regime::RiskOn, Regime::Range],
        source_compatibility: vec![
            EvidenceSourceKind::OfficialApiCollected,
            EvidenceSourceKind::RealLocal,
            EvidenceSourceKind::YFinanceResearch,
            EvidenceSourceKind::TestFixture,
            EvidenceSourceKind::SyntheticFixture,
        ],
        active: true,
        reason_codes: vec![ReasonCode::PersonaCardLiteBuilt],
    }
}

pub fn defensive_value_risk_card() -> PersonaCardLite {
    PersonaCardLite {
        persona_id: "defensive_value_risk".to_string(),
        archetype_label: "Graham/Marks-like archetype label; not a literal investor reproduction"
            .to_string(),
        group: PersonaGroup::Slow,
        horizon: PersonaHorizon::Swing,
        role: PersonaRole::RiskSkeptic,
        immutable_doctrine: vec![
            doctrine(
                "margin_of_safety_required",
                "Margin of safety required.",
                true,
            ),
            doctrine(
                "avoid_overheated_crowd",
                "Avoid overheated crowd behavior.",
                true,
            ),
            doctrine("risk_control_first", "Risk control first.", true),
            doctrine("no_trade_is_valid", "NoTrade is a valid output.", false),
        ],
        mutable_policy: PersonaMutablePolicy {
            entry_threshold: 0.66,
            no_trade_threshold: 0.54,
            reduce_size_threshold: 0.62,
            max_voice_power: 0.62,
            min_data_quality: 0.80,
            max_spread_bps: 14.0,
            volatility_limit: 0.05,
            confidence_floor: 0.52,
            reason_codes: vec![ReasonCode::PersonaCardLiteBuilt],
        },
        base_voice_power: 0.58,
        regime_specialties: vec![Regime::Range, Regime::RiskOff, Regime::HighVolatility],
        source_compatibility: vec![
            EvidenceSourceKind::OfficialApiCollected,
            EvidenceSourceKind::RealLocal,
            EvidenceSourceKind::YFinanceResearch,
            EvidenceSourceKind::TestFixture,
            EvidenceSourceKind::SyntheticFixture,
        ],
        active: true,
        reason_codes: vec![ReasonCode::PersonaCardLiteBuilt],
    }
}

pub fn cycle_regime_guard_card() -> PersonaCardLite {
    PersonaCardLite {
        persona_id: "cycle_regime_guard".to_string(),
        archetype_label: "Dalio/PTJ-like archetype label; not a literal investor reproduction"
            .to_string(),
        group: PersonaGroup::Risk,
        horizon: PersonaHorizon::MultiDay,
        role: PersonaRole::RegimeGuard,
        immutable_doctrine: vec![
            doctrine("know_the_regime", "Know the regime before speaking.", true),
            doctrine("risk_before_return", "Risk comes before return.", true),
            doctrine(
                "reduce_size_under_uncertainty",
                "Reduce size under uncertainty.",
                false,
            ),
            doctrine(
                "do_not_fight_extreme_volatility",
                "Do not fight extreme volatility.",
                true,
            ),
        ],
        mutable_policy: PersonaMutablePolicy {
            entry_threshold: 0.60,
            no_trade_threshold: 0.50,
            reduce_size_threshold: 0.58,
            max_voice_power: 0.74,
            min_data_quality: 0.76,
            max_spread_bps: 16.0,
            volatility_limit: 0.05,
            confidence_floor: 0.50,
            reason_codes: vec![ReasonCode::PersonaCardLiteBuilt],
        },
        base_voice_power: 0.70,
        regime_specialties: vec![
            Regime::TrendUp,
            Regime::RiskOn,
            Regime::HighVolatility,
            Regime::Panic,
            Regime::Unknown,
        ],
        source_compatibility: vec![
            EvidenceSourceKind::OfficialApiCollected,
            EvidenceSourceKind::RealLocal,
            EvidenceSourceKind::YFinanceResearch,
            EvidenceSourceKind::TestFixture,
            EvidenceSourceKind::SyntheticFixture,
        ],
        active: true,
        reason_codes: vec![ReasonCode::PersonaCardLiteBuilt],
    }
}

fn doctrine(rule_id: &str, description: &str, hard: bool) -> DoctrineRule {
    DoctrineRule {
        rule_id: rule_id.to_string(),
        description: description.to_string(),
        hard,
        reason_codes: vec![ReasonCode::PersonaCardLiteBuilt],
    }
}

fn default_policy() -> PersonaMutablePolicy {
    PersonaMutablePolicy {
        entry_threshold: 0.60,
        no_trade_threshold: 0.50,
        reduce_size_threshold: 0.55,
        max_voice_power: 0.0,
        min_data_quality: 0.80,
        max_spread_bps: 15.0,
        volatility_limit: 0.05,
        confidence_floor: 0.50,
        reason_codes: vec![ReasonCode::PersonaCardLiteBuilt],
    }
}
