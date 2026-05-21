use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::persona_vote::{PersonaStance, PersonaVote};
use super::trinity_personas::active_trinity_scorers;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonaOperationalStatus {
    #[default]
    Idle,
    WaitingData,
    Analyzing,
    Voting,
    Abstained,
    Vetoed,
    Done,
    Error,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PersonaOperationalView {
    pub persona_id: String,
    pub status: PersonaOperationalStatus,
    #[serde(default)]
    pub current_candidate_id: Option<String>,
    #[serde(default)]
    pub current_symbol: Option<String>,
    #[serde(default)]
    pub last_stance: Option<String>,
    #[serde(default)]
    pub last_conviction: Option<f64>,
    #[serde(default)]
    pub last_voice_power: Option<f64>,
    #[serde(default)]
    pub last_reason_codes: Vec<ReasonCode>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TrinityOperationalStatusReport {
    #[serde(default)]
    pub persona_views: Vec<PersonaOperationalView>,
    pub active_count: usize,
    pub idle_count: usize,
    pub analyzing_count: usize,
    pub voting_count: usize,
    pub blocked_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
    pub fingerprint: String,
}

impl TrinityOperationalStatusReport {
    pub fn stabilize(&mut self) {
        self.persona_views
            .sort_by(|left, right| left.persona_id.cmp(&right.persona_id));
        for view in &mut self.persona_views {
            view.last_reason_codes = stable_reason_codes(&view.last_reason_codes);
            view.reason_codes = stable_reason_codes(&view.reason_codes);
        }
        self.active_count = self.persona_views.len();
        self.idle_count = self
            .persona_views
            .iter()
            .filter(|view| matches!(view.status, PersonaOperationalStatus::Idle))
            .count();
        self.analyzing_count = self
            .persona_views
            .iter()
            .filter(|view| matches!(view.status, PersonaOperationalStatus::Analyzing))
            .count();
        self.voting_count = self
            .persona_views
            .iter()
            .filter(|view| matches!(view.status, PersonaOperationalStatus::Voting))
            .count();
        self.blocked_count = self
            .persona_views
            .iter()
            .filter(|view| {
                matches!(
                    view.status,
                    PersonaOperationalStatus::Abstained
                        | PersonaOperationalStatus::Vetoed
                        | PersonaOperationalStatus::Error
                )
            })
            .count();
        self.reason_codes = stable_reason_codes(&self.reason_codes);
        self.fingerprint = stable_hash_string(&serde_json::to_string(self).unwrap_or_default());
    }

    pub fn to_text(&self) -> String {
        [
            "research_only_warning=trinity operational status is monitor-only deterministic state"
                .to_string(),
            format!("active_count={}", self.active_count),
            format!("idle_count={}", self.idle_count),
            format!("analyzing_count={}", self.analyzing_count),
            format!("voting_count={}", self.voting_count),
            format!("blocked_count={}", self.blocked_count),
            format!("fingerprint={}", self.fingerprint),
        ]
        .join("\n")
    }
}

pub fn idle_trinity_operational_status_report() -> TrinityOperationalStatusReport {
    let mut report = TrinityOperationalStatusReport {
        persona_views: active_trinity_scorers()
            .into_iter()
            .map(|scorer| {
                let card = scorer.card();
                PersonaOperationalView {
                    persona_id: card.persona_id,
                    status: PersonaOperationalStatus::Idle,
                    current_candidate_id: None,
                    current_symbol: None,
                    last_stance: None,
                    last_conviction: None,
                    last_voice_power: None,
                    last_reason_codes: vec![ReasonCode::DeterministicPath],
                    reason_codes: vec![ReasonCode::DeterministicPath],
                }
            })
            .collect(),
        active_count: 0,
        idle_count: 0,
        analyzing_count: 0,
        voting_count: 0,
        blocked_count: 0,
        reason_codes: vec![ReasonCode::DeterministicPath],
        fingerprint: String::new(),
    };
    report.stabilize();
    report
}

pub fn build_status_report_from_votes(
    candidate_id: &str,
    symbol: &str,
    votes: &[PersonaVote],
) -> TrinityOperationalStatusReport {
    let mut views = active_trinity_scorers()
        .into_iter()
        .map(|scorer| {
            let card = scorer.card();
            if let Some(vote) = votes.iter().find(|vote| vote.persona_id == card.persona_id) {
                let status = match vote.stance {
                    PersonaStance::Abstain => PersonaOperationalStatus::Abstained,
                    PersonaStance::Veto => PersonaOperationalStatus::Vetoed,
                    _ => PersonaOperationalStatus::Done,
                };
                PersonaOperationalView {
                    persona_id: vote.persona_id.clone(),
                    status,
                    current_candidate_id: Some(candidate_id.to_string()),
                    current_symbol: Some(symbol.to_string()),
                    last_stance: Some(format!("{:?}", vote.stance)),
                    last_conviction: Some(vote.conviction),
                    last_voice_power: Some(vote.voice_power),
                    last_reason_codes: vote.reason_codes.clone(),
                    reason_codes: vec![ReasonCode::DeterministicPath],
                }
            } else {
                PersonaOperationalView {
                    persona_id: card.persona_id,
                    status: PersonaOperationalStatus::Idle,
                    current_candidate_id: None,
                    current_symbol: None,
                    last_stance: None,
                    last_conviction: None,
                    last_voice_power: None,
                    last_reason_codes: vec![ReasonCode::DeterministicPath],
                    reason_codes: vec![ReasonCode::DeterministicPath],
                }
            }
        })
        .collect::<Vec<_>>();
    views.sort_by(|left, right| left.persona_id.cmp(&right.persona_id));
    let mut report = TrinityOperationalStatusReport {
        persona_views: views,
        active_count: 0,
        idle_count: 0,
        analyzing_count: 0,
        voting_count: 0,
        blocked_count: 0,
        reason_codes: vec![ReasonCode::DeterministicPath],
        fingerprint: String::new(),
    };
    report.stabilize();
    report
}
