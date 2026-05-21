use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::committee_replay::CommitteeReplayReport;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeAttributionStatus {
    Balanced,
    PersonaDominated,
    ChairDominated,
    RiskDominated,
    SourceLimited,
    InsufficientSamples,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaContribution {
    pub persona_id: String,
    pub vote_count: usize,
    pub selected_count: usize,
    pub avg_voice_power: f64,
    pub avg_conviction: f64,
    pub stance_distribution: BTreeMap<String, usize>,
    pub decision_influence_proxy: f64,
    pub doctrine_violation_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeAttributionReport {
    pub persona_contributions: Vec<PersonaContribution>,
    pub chair_contribution_summary: BTreeMap<String, usize>,
    pub risk_governor_contribution_summary: BTreeMap<String, usize>,
    pub source_contribution_summary: BTreeMap<String, usize>,
    pub high_influence_personas: Vec<String>,
    pub low_influence_personas: Vec<String>,
    pub overdominance_warnings: Vec<String>,
    pub underparticipation_warnings: Vec<String>,
    pub attribution_status: CommitteeAttributionStatus,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_committee_attribution_report(
    replay_report: &CommitteeReplayReport,
) -> CommitteeAttributionReport {
    let mut by_persona = BTreeMap::<String, Vec<_>>::new();
    let mut selected_counts = BTreeMap::<String, usize>::new();
    let mut chair_contribution_summary = BTreeMap::new();
    let mut risk_governor_contribution_summary = BTreeMap::new();
    let mut source_contribution_summary = BTreeMap::new();
    for record in &replay_report.records {
        *chair_contribution_summary
            .entry(format!("{:?}", record.chair_decision_record.final_decision))
            .or_insert(0) += 1;
        *risk_governor_contribution_summary
            .entry(format!(
                "{:?}",
                record.risk_bridge_outcome.risk_decision.kind
            ))
            .or_insert(0) += 1;
        *source_contribution_summary
            .entry(format!("{:?}", record.scenario_row.source_kind))
            .or_insert(0) += 1;
        for selected in &record.chair_decision_record.selected_speakers {
            *selected_counts.entry(selected.clone()).or_insert(0) += 1;
        }
        for vote in &record.persona_votes {
            by_persona
                .entry(vote.persona_id.clone())
                .or_default()
                .push(vote);
        }
    }
    let total_influence = by_persona
        .values()
        .flat_map(|votes| votes.iter())
        .map(|vote| vote.voice_power * vote.conviction)
        .sum::<f64>()
        .max(1e-9);
    let mut persona_contributions = by_persona
        .into_iter()
        .map(|(persona_id, votes)| {
            let vote_count = votes.len();
            let mut stance_distribution = BTreeMap::new();
            let mut voice_sum = 0.0;
            let mut conviction_sum = 0.0;
            let mut doctrine_violation_count = 0usize;
            let mut influence_sum = 0.0;
            for vote in votes {
                *stance_distribution
                    .entry(format!("{:?}", vote.stance))
                    .or_insert(0) += 1;
                voice_sum += vote.voice_power;
                conviction_sum += vote.conviction;
                doctrine_violation_count += vote.doctrine_violations.len();
                influence_sum += vote.voice_power * vote.conviction;
            }
            PersonaContribution {
                selected_count: selected_counts.get(&persona_id).copied().unwrap_or(0),
                avg_voice_power: voice_sum / vote_count.max(1) as f64,
                avg_conviction: conviction_sum / vote_count.max(1) as f64,
                decision_influence_proxy: influence_sum / total_influence,
                persona_id,
                vote_count,
                stance_distribution,
                doctrine_violation_count,
                reason_codes: vec![ReasonCode::CommitteeAttributionBuilt],
            }
        })
        .collect::<Vec<_>>();
    persona_contributions.sort_by(|left, right| left.persona_id.cmp(&right.persona_id));
    let high_influence_personas = persona_contributions
        .iter()
        .filter(|row| row.decision_influence_proxy >= 0.40)
        .map(|row| row.persona_id.clone())
        .collect::<Vec<_>>();
    let low_influence_personas = persona_contributions
        .iter()
        .filter(|row| row.selected_count == 0)
        .map(|row| row.persona_id.clone())
        .collect::<Vec<_>>();
    let mut overdominance_warnings = Vec::new();
    if let Some(top) = persona_contributions.iter().max_by(|left, right| {
        left.decision_influence_proxy
            .total_cmp(&right.decision_influence_proxy)
    }) {
        if top.decision_influence_proxy >= 0.60 {
            overdominance_warnings.push(format!("{} dominates decision influence", top.persona_id));
        }
    }
    let underparticipation_warnings = low_influence_personas
        .iter()
        .map(|persona| format!("{persona} was never selected"))
        .collect::<Vec<_>>();
    let attribution_status = if replay_report.record_count < 3 {
        CommitteeAttributionStatus::InsufficientSamples
    } else if !overdominance_warnings.is_empty() {
        CommitteeAttributionStatus::PersonaDominated
    } else if risk_governor_contribution_summary
        .get("Deny")
        .copied()
        .unwrap_or(0)
        * 2
        >= replay_report.record_count
    {
        CommitteeAttributionStatus::RiskDominated
    } else if chair_contribution_summary.len() == 1 {
        CommitteeAttributionStatus::ChairDominated
    } else if source_contribution_summary.len() <= 1 {
        CommitteeAttributionStatus::SourceLimited
    } else {
        CommitteeAttributionStatus::Balanced
    };
    CommitteeAttributionReport {
        persona_contributions,
        chair_contribution_summary,
        risk_governor_contribution_summary,
        source_contribution_summary,
        high_influence_personas,
        low_influence_personas,
        overdominance_warnings,
        underparticipation_warnings,
        attribution_status,
        reason_codes: vec![ReasonCode::CommitteeAttributionBuilt],
    }
}

impl CommitteeAttributionReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![format!("attribution_status={:?}", self.attribution_status)];
        for persona in &self.persona_contributions {
            lines.push(format!(
                "persona={};vote_count={};selected_count={};avg_voice_power={:.6};avg_conviction={:.6};influence={:.6}",
                persona.persona_id,
                persona.vote_count,
                persona.selected_count,
                persona.avg_voice_power,
                persona.avg_conviction,
                persona.decision_influence_proxy
            ));
        }
        lines.push(format!(
            "overdominance_warnings={}",
            self.overdominance_warnings.join("|")
        ));
        lines.push(format!(
            "underparticipation_warnings={}",
            self.underparticipation_warnings.join("|")
        ));
        lines.join("\n")
    }
}
