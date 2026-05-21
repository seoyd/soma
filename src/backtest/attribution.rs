use serde::{Deserialize, Serialize};

use super::triple_barrier::TripleBarrierResult;
use crate::core::{ReasonCode, Stance};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CounterfactualRole {
    SupportedFinalDecision,
    OpposedFinalDecision,
    ForcedContrarian,
    ShadowOnly,
    RiskVetoAligned,
    RiskVetoOpposed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AttributionRecord {
    pub persona_id: String,
    pub selected_for_decision: bool,
    pub stance: Stance,
    pub conviction: f64,
    pub voice_power: f64,
    pub contribution_score: f64,
    pub counterfactual_role: CounterfactualRole,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShadowOutcomeRecord {
    pub persona_id: String,
    pub hypothetical_stance: Stance,
    pub hypothetical_result: Option<TripleBarrierResult>,
    pub would_have_supported_trade: bool,
    pub would_have_blocked_trade: bool,
    pub evaluation_pending: bool,
}
