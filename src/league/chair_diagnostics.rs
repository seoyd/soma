use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::committee_decision::{CommitteeDecision, CommitteeDecisionRecord, CommitteeInput};
use super::persona_card_lite::{PersonaGroup, PersonaHorizon, persona_card_lite_by_id};
use super::persona_vote::PersonaStance;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SpeakerFilterReason {
    Inactive,
    SourceIncompatible,
    HorizonIncompatible,
    LowRegimeFit,
    LowDataQualityFit,
    LowVoicePower,
    DoctrineViolation,
    NotSelectedByTopK,
    Selected,
    ForcedContrarian,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpeakerSelectionTrace {
    pub persona_id: String,
    pub initial_candidate: bool,
    pub selected: bool,
    pub filter_reasons: Vec<SpeakerFilterReason>,
    pub base_voice_power: f64,
    pub adjusted_voice_power: f64,
    pub regime_weight: f64,
    pub cluster_penalty: f64,
    pub contrarian_bonus: f64,
    pub final_voice_power: f64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairDiagnosticStatus {
    Healthy,
    TooFewSpeakers,
    GroupthinkRisk,
    ExcessiveDisagreement,
    OverFiltered,
    VetoDominated,
    ResearchOnlySource,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairDiagnosticsReport {
    pub decision_id: String,
    pub speaker_traces: Vec<SpeakerSelectionTrace>,
    pub selected_speakers: Vec<String>,
    pub filtered_speakers: Vec<String>,
    pub cluster_counts: BTreeMap<String, usize>,
    pub cluster_penalty_applied: bool,
    pub contrarian_included: bool,
    pub groupthink_risk: f64,
    pub disagreement_score: f64,
    pub uncertainty: f64,
    pub weighted_score: f64,
    pub final_decision: CommitteeDecision,
    pub diagnostic_status: ChairDiagnosticStatus,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_chair_diagnostics(
    input: &CommitteeInput,
    record: &CommitteeDecisionRecord,
) -> ChairDiagnosticsReport {
    let mut traces = Vec::new();
    let mut cluster_counts = BTreeMap::<String, usize>::new();
    let mut selected_counts = BTreeMap::<String, usize>::new();
    for speaker in &record.selected_speakers {
        *selected_counts.entry(speaker.clone()).or_insert(0) += 1;
    }
    for vote in &record.all_votes {
        let card = persona_card_lite_by_id(&vote.persona_id);
        let source_compatible = vote.source_kind == input.source_kind;
        let horizon_compatible = horizon_compatible(vote.horizon, input.target_horizon);
        let active = card.as_ref().is_some_and(|card| card.active);
        let selected = selected_counts.get(&vote.persona_id).copied().unwrap_or(0) > 0;
        let cluster_label = card
            .as_ref()
            .map(|card| match card.group {
                PersonaGroup::Fast => "Momentum",
                PersonaGroup::Slow => "Defensive",
                PersonaGroup::Risk => "Regime",
                PersonaGroup::Crypto => "Crypto",
                PersonaGroup::ResearchOnly => "ResearchOnly",
            })
            .unwrap_or("ResearchOnly")
            .to_string();
        if selected {
            *cluster_counts.entry(cluster_label.clone()).or_insert(0) += 1;
        }
        let cluster_position = cluster_counts.get(&cluster_label).copied().unwrap_or(1);
        let cluster_penalty = match cluster_position {
            0 | 1 => 1.0,
            2 => 0.6,
            _ => 0.3,
        };
        let regime_weight = 0.5 + vote.regime_fit * 0.5;
        let adjusted_voice_power = vote.voice_power * vote.conviction;
        let contrarian_bonus = if record
            .chair_reason_codes
            .contains(&ReasonCode::ContrarianIncluded)
            && matches!(
                vote.stance,
                PersonaStance::ReduceSize
                    | PersonaStance::NoTrade
                    | PersonaStance::Veto
                    | PersonaStance::Abstain
            ) {
            0.10
        } else {
            0.0
        };
        let final_voice_power =
            adjusted_voice_power * regime_weight * cluster_penalty + contrarian_bonus;
        let mut filter_reasons = Vec::new();
        let initial_candidate = active && source_compatible && horizon_compatible;
        if !active {
            filter_reasons.push(SpeakerFilterReason::Inactive);
        }
        if !source_compatible {
            filter_reasons.push(SpeakerFilterReason::SourceIncompatible);
        }
        if !horizon_compatible {
            filter_reasons.push(SpeakerFilterReason::HorizonIncompatible);
        }
        if vote.regime_fit < 0.5 {
            filter_reasons.push(SpeakerFilterReason::LowRegimeFit);
        }
        if vote.data_quality_fit < 0.5 {
            filter_reasons.push(SpeakerFilterReason::LowDataQualityFit);
        }
        if vote.voice_power < 0.1 {
            filter_reasons.push(SpeakerFilterReason::LowVoicePower);
        }
        if !vote.doctrine_violations.is_empty() {
            filter_reasons.push(SpeakerFilterReason::DoctrineViolation);
        }
        if selected {
            filter_reasons.push(SpeakerFilterReason::Selected);
        } else if initial_candidate {
            filter_reasons.push(SpeakerFilterReason::NotSelectedByTopK);
        }
        if contrarian_bonus > 0.0 {
            filter_reasons.push(SpeakerFilterReason::ForcedContrarian);
        }
        traces.push(SpeakerSelectionTrace {
            persona_id: vote.persona_id.clone(),
            initial_candidate,
            selected,
            filter_reasons,
            base_voice_power: vote.voice_power,
            adjusted_voice_power,
            regime_weight,
            cluster_penalty,
            contrarian_bonus,
            final_voice_power,
            reason_codes: vec![ReasonCode::ChairDiagnosticsBuilt],
        });
    }
    traces.sort_by(|left, right| left.persona_id.cmp(&right.persona_id));
    let filtered_speakers = traces
        .iter()
        .filter(|trace| !trace.selected)
        .map(|trace| trace.persona_id.clone())
        .collect::<Vec<_>>();
    let diagnostic_status =
        if input.source_kind == crate::data::EvidenceSourceKind::YFinanceResearch {
            ChairDiagnosticStatus::ResearchOnlySource
        } else if record.final_decision == CommitteeDecision::Vetoed {
            ChairDiagnosticStatus::VetoDominated
        } else if record.selected_speakers.len() < 2 {
            ChairDiagnosticStatus::TooFewSpeakers
        } else if record.groupthink_risk >= 0.65 {
            ChairDiagnosticStatus::GroupthinkRisk
        } else if record.disagreement_score >= 0.55 {
            ChairDiagnosticStatus::ExcessiveDisagreement
        } else if filtered_speakers.len() > record.selected_speakers.len() {
            ChairDiagnosticStatus::OverFiltered
        } else {
            ChairDiagnosticStatus::Healthy
        };
    ChairDiagnosticsReport {
        decision_id: record.decision_id.clone(),
        selected_speakers: record.selected_speakers.clone(),
        filtered_speakers,
        cluster_counts,
        cluster_penalty_applied: record
            .chair_reason_codes
            .contains(&ReasonCode::ClusterPenaltyApplied),
        contrarian_included: record
            .chair_reason_codes
            .contains(&ReasonCode::ContrarianIncluded),
        groupthink_risk: record.groupthink_risk,
        disagreement_score: record.disagreement_score,
        uncertainty: record.uncertainty,
        weighted_score: record.weighted_score,
        final_decision: record.final_decision,
        diagnostic_status,
        speaker_traces: traces,
        reason_codes: vec![ReasonCode::ChairDiagnosticsBuilt],
    }
}

impl ChairDiagnosticsReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("decision_id={}", self.decision_id),
            format!("diagnostic_status={:?}", self.diagnostic_status),
            format!("weighted_score={:.6}", self.weighted_score),
            format!("disagreement_score={:.6}", self.disagreement_score),
            format!("groupthink_risk={:.6}", self.groupthink_risk),
            format!("contrarian_included={}", self.contrarian_included),
        ];
        for trace in &self.speaker_traces {
            lines.push(format!(
                "speaker={};selected={};final_voice_power={:.6};reasons={:?}",
                trace.persona_id, trace.selected, trace.final_voice_power, trace.filter_reasons
            ));
        }
        lines.join("\n")
    }
}

fn horizon_compatible(persona: PersonaHorizon, target: PersonaHorizon) -> bool {
    match (persona, target) {
        (PersonaHorizon::Intraday, PersonaHorizon::Intraday | PersonaHorizon::Swing) => true,
        (
            PersonaHorizon::Swing,
            PersonaHorizon::Swing | PersonaHorizon::MultiDay | PersonaHorizon::LongTerm,
        ) => true,
        (
            PersonaHorizon::MultiDay,
            PersonaHorizon::Swing | PersonaHorizon::MultiDay | PersonaHorizon::LongTerm,
        ) => true,
        (PersonaHorizon::LongTerm, PersonaHorizon::LongTerm) => true,
        _ => false,
    }
}
