use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, Regime};
use crate::data::EvidenceSourceKind;

use super::persona_card_lite::PersonaHorizon;
use super::persona_scorer::PersonaScoringInput;
use super::persona_vote::PersonaVote;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PersonaCluster {
    Momentum,
    Defensive,
    Regime,
    Crypto,
    ResearchOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairCommitteeConfig {
    pub min_speakers: usize,
    pub max_speakers: usize,
    pub require_contrarian: bool,
    pub cluster_penalty_enabled: bool,
    pub groupthink_warning_threshold: f64,
    pub uncertainty_reduce_threshold: f64,
    pub no_trade_threshold: f64,
    pub approve_threshold: f64,
    pub veto_absolute: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeInput {
    pub scoring_input: PersonaScoringInput,
    pub persona_votes: Vec<PersonaVote>,
    pub target_horizon: PersonaHorizon,
    pub source_kind: EvidenceSourceKind,
    pub regime: Regime,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeDecision {
    ApproveCandidate,
    ReduceSizeCandidate,
    RequireHumanConfirm,
    NoTrade,
    Vetoed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeDecisionRecord {
    pub decision_id: String,
    pub symbol: String,
    pub timestamp_ms: u64,
    pub selected_speakers: Vec<String>,
    pub all_votes: Vec<PersonaVote>,
    pub weighted_score: f64,
    pub disagreement_score: f64,
    pub groupthink_risk: f64,
    pub uncertainty: f64,
    pub final_decision: CommitteeDecision,
    pub chair_reason_codes: Vec<ReasonCode>,
    pub source_kind: EvidenceSourceKind,
    pub regime: Regime,
    #[serde(default)]
    pub core_fingerprint: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeDebateReport {
    pub selected_speaker_counts: BTreeMap<String, usize>,
    pub average_disagreement: f64,
    pub groupthink_warning: bool,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ChairCommitteeConfig {
    fn default() -> Self {
        Self {
            min_speakers: 2,
            max_speakers: 3,
            require_contrarian: true,
            cluster_penalty_enabled: true,
            groupthink_warning_threshold: 0.65,
            uncertainty_reduce_threshold: 0.55,
            no_trade_threshold: 0.05,
            approve_threshold: 0.40,
            veto_absolute: true,
            reason_codes: vec![ReasonCode::ChairV0Built],
        }
    }
}

impl CommitteeDebateReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("average_disagreement={:.6}", self.average_disagreement),
            format!("groupthink_warning={}", self.groupthink_warning),
            format!("warnings={}", self.warnings.join("|")),
        ];
        for (speaker, count) in &self.selected_speaker_counts {
            lines.push(format!("speaker={speaker};count={count}"));
        }
        lines.join("\n")
    }
}
