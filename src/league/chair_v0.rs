use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::chair::correlation::cluster_multiplier;
use crate::core::ReasonCode;

use super::committee_decision::{
    ChairCommitteeConfig, CommitteeDecision, CommitteeDecisionRecord, CommitteeInput,
    PersonaCluster,
};
use super::persona_card_lite::{PersonaGroup, PersonaHorizon, persona_card_lite_by_id};
use super::persona_vote::{PersonaStance, PersonaVote};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChairV0 {
    pub config: ChairCommitteeConfig,
}

impl ChairV0 {
    pub fn evaluate(&self, input: &CommitteeInput) -> CommitteeDecisionRecord {
        let mut reason_codes = vec![ReasonCode::ChairV0Built];
        let mut compatible = input
            .persona_votes
            .iter()
            .filter(|vote| vote.source_kind == input.source_kind)
            .filter(|vote| horizon_compatible(vote.horizon, input.target_horizon))
            .filter(|vote| {
                persona_card_lite_by_id(&vote.persona_id).is_some_and(|card| card.active)
            })
            .cloned()
            .collect::<Vec<_>>();
        compatible.sort_by(|left, right| {
            let lhs = left.voice_power * left.conviction * (0.5 + left.regime_fit * 0.5);
            let rhs = right.voice_power * right.conviction * (0.5 + right.regime_fit * 0.5);
            rhs.total_cmp(&lhs)
        });
        let max_speakers = self.config.max_speakers.max(self.config.min_speakers);
        let mut selected = compatible
            .iter()
            .take(max_speakers)
            .cloned()
            .collect::<Vec<_>>();
        if selected.len() < self.config.min_speakers {
            reason_codes.push(ReasonCode::NoActiveVoices);
        }
        let aligned = one_sided(&selected);
        if self.config.require_contrarian && aligned {
            if let Some(extra) = compatible.iter().skip(selected.len()).find(|vote| {
                matches!(
                    vote.stance,
                    PersonaStance::ReduceSize
                        | PersonaStance::NoTrade
                        | PersonaStance::Veto
                        | PersonaStance::Abstain
                )
            }) {
                selected.push(extra.clone());
                reason_codes.push(ReasonCode::ContrarianIncluded);
            }
        }

        let mut cluster_counts = BTreeMap::<PersonaCluster, usize>::new();
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        let mut hard_veto = false;
        for vote in &selected {
            let cluster = cluster_for_vote(vote);
            let entry = cluster_counts.entry(cluster).or_insert(0);
            *entry += 1;
            let cluster_mult = if self.config.cluster_penalty_enabled {
                cluster_multiplier(*entry)
            } else {
                1.0
            };
            if self.config.cluster_penalty_enabled && *entry > 1 {
                reason_codes.push(ReasonCode::ClusterPenaltyApplied);
            }
            let weight =
                vote.voice_power * vote.conviction * (0.5 + vote.regime_fit * 0.5) * cluster_mult;
            weighted_sum += vote.stance.score() * weight;
            total_weight += weight;
            hard_veto |= vote.stance == PersonaStance::Veto || !vote.doctrine_violations.is_empty();
        }
        let weighted_score = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        };
        let disagreement_score = disagreement_score(&selected, weighted_score);
        let groupthink_risk = groupthink_risk(&selected, &cluster_counts);
        if groupthink_risk >= self.config.groupthink_warning_threshold {
            reason_codes.push(ReasonCode::GroupthinkRiskElevated);
        }
        if disagreement_score >= 0.35 {
            reason_codes.push(ReasonCode::DisagreementElevated);
        }
        let uncertainty =
            ((1.0 - weighted_score.abs()).max(0.0) + disagreement_score + groupthink_risk) / 3.0;
        let final_decision = if selected.is_empty() {
            CommitteeDecision::NoTrade
        } else if hard_veto && self.config.veto_absolute {
            CommitteeDecision::Vetoed
        } else if disagreement_score >= self.config.uncertainty_reduce_threshold
            || groupthink_risk >= self.config.groupthink_warning_threshold
        {
            CommitteeDecision::ReduceSizeCandidate
        } else if weighted_score <= self.config.no_trade_threshold {
            CommitteeDecision::NoTrade
        } else if weighted_score >= self.config.approve_threshold {
            CommitteeDecision::ApproveCandidate
        } else {
            CommitteeDecision::RequireHumanConfirm
        };
        CommitteeDecisionRecord {
            decision_id: format!(
                "committee-{}-{}",
                input.scoring_input.symbol, input.scoring_input.timestamp_ms
            ),
            symbol: input.scoring_input.symbol.clone(),
            timestamp_ms: input.scoring_input.timestamp_ms,
            selected_speakers: selected
                .iter()
                .map(|vote| vote.persona_id.clone())
                .collect(),
            all_votes: input.persona_votes.clone(),
            weighted_score,
            disagreement_score,
            groupthink_risk,
            uncertainty,
            final_decision,
            chair_reason_codes: reason_codes.clone(),
            source_kind: input.source_kind,
            regime: input.regime,
            core_fingerprint: None,
            reason_codes,
        }
    }
}

fn cluster_for_vote(vote: &PersonaVote) -> PersonaCluster {
    match persona_card_lite_by_id(&vote.persona_id).map(|card| card.group) {
        Some(PersonaGroup::Fast) => PersonaCluster::Momentum,
        Some(PersonaGroup::Slow) => PersonaCluster::Defensive,
        Some(PersonaGroup::Risk) => PersonaCluster::Regime,
        Some(PersonaGroup::Crypto) => PersonaCluster::Crypto,
        Some(PersonaGroup::ResearchOnly) | None => PersonaCluster::ResearchOnly,
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

fn one_sided(votes: &[PersonaVote]) -> bool {
    let mut positive = false;
    let mut negative = false;
    for vote in votes {
        match vote.stance {
            PersonaStance::StrongApprove | PersonaStance::Approve => positive = true,
            PersonaStance::ReduceSize | PersonaStance::NoTrade | PersonaStance::Veto => {
                negative = true
            }
            PersonaStance::Abstain => {}
        }
    }
    !(positive && negative)
}

fn disagreement_score(votes: &[PersonaVote], weighted_score: f64) -> f64 {
    if votes.is_empty() {
        return 0.0;
    }
    let total = votes
        .iter()
        .map(|vote| (vote.stance.score() - weighted_score).abs())
        .sum::<f64>();
    (total / votes.len() as f64).clamp(0.0, 1.0)
}

fn groupthink_risk(votes: &[PersonaVote], clusters: &BTreeMap<PersonaCluster, usize>) -> f64 {
    if votes.is_empty() {
        return 0.0;
    }
    let aligned = if one_sided(votes) { 0.75 } else { 0.25 };
    let max_cluster_share =
        clusters.values().copied().max().unwrap_or(1) as f64 / votes.len() as f64;
    (aligned + max_cluster_share) / 2.0
}
