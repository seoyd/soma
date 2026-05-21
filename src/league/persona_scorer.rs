use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, Regime, RiskSnapshot, SignalOutput};
use crate::data::{EvidenceSourceKind, ProviderMarket};
use crate::feature::FeatureVector;

use super::persona_card_lite::{PersonaCardLite, PersonaHorizon};
use super::persona_vote::PersonaVote;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaScoringInput {
    pub symbol: String,
    pub timestamp_ms: u64,
    pub source_kind: EvidenceSourceKind,
    pub market: ProviderMarket,
    pub target_horizon: PersonaHorizon,
    #[serde(default)]
    pub feature_vector: Option<FeatureVector>,
    pub regime: Regime,
    pub signal_output: SignalOutput,
    pub data_quality_score: f64,
    #[serde(default)]
    pub spread_bps: Option<f64>,
    pub expected_edge_after_cost: f64,
    pub expected_drawdown: f64,
    #[serde(default)]
    pub risk_snapshot: Option<RiskSnapshot>,
    pub reason_codes: Vec<ReasonCode>,
}

pub trait PersonaScorer {
    fn card(&self) -> PersonaCardLite;
    fn score(&self, input: &PersonaScoringInput) -> PersonaVote;
}
