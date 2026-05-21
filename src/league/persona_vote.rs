use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::EvidenceSourceKind;

use super::persona_card_lite::PersonaHorizon;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PersonaStance {
    StrongApprove,
    Approve,
    ReduceSize,
    NoTrade,
    Veto,
    Abstain,
}

impl PersonaStance {
    pub fn score(self) -> f64 {
        match self {
            Self::StrongApprove => 1.0,
            Self::Approve => 0.65,
            Self::ReduceSize => 0.20,
            Self::NoTrade => -0.35,
            Self::Veto => -1.0,
            Self::Abstain => 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaVote {
    pub persona_id: String,
    pub stance: PersonaStance,
    pub conviction: f64,
    pub voice_power: f64,
    pub horizon: PersonaHorizon,
    pub source_kind: EvidenceSourceKind,
    pub regime_fit: f64,
    pub data_quality_fit: f64,
    pub risk_fit: f64,
    pub expected_edge_fit: f64,
    pub doctrine_violations: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl PersonaVote {
    pub fn bounded(mut self) -> Self {
        self.conviction = self.conviction.clamp(0.0, 1.0);
        self.voice_power = self.voice_power.clamp(0.0, 1.0);
        self.regime_fit = self.regime_fit.clamp(0.0, 1.0);
        self.data_quality_fit = self.data_quality_fit.clamp(0.0, 1.0);
        self.risk_fit = self.risk_fit.clamp(0.0, 1.0);
        self.expected_edge_fit = self.expected_edge_fit.clamp(0.0, 1.0);
        self
    }
}
