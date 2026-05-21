use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::{
    ChairDecisionKind, ChairInput, ChairOutput, InvestorVote, MarketSnapshot, ReasonCode, Side,
    SignalOutput, Stance, TradeProposal,
};
use crate::league::persona_card_by_id;

use super::correlation::cluster_multiplier;

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChairConfig {
    pub strong_threshold: f64,
    pub weak_threshold: f64,
    pub allow_forced_contrarian: bool,
    pub cluster_penalty_enabled: bool,
    pub defensive_bonus_weight: f64,
    pub risk_penalty_weight: f64,
    pub groupthink_penalty_weight: f64,
    pub disagreement_penalty_weight: f64,
    pub cluster_groupthink_penalty: f64,
}

impl Default for ChairConfig {
    fn default() -> Self {
        Self {
            strong_threshold: 0.35,
            weak_threshold: 0.18,
            allow_forced_contrarian: true,
            cluster_penalty_enabled: true,
            defensive_bonus_weight: 0.35,
            risk_penalty_weight: 0.50,
            groupthink_penalty_weight: 0.20,
            disagreement_penalty_weight: 0.15,
            cluster_groupthink_penalty: 0.10,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ChairEngine {
    pub config: ChairConfig,
}

impl ChairEngine {
    pub fn evaluate(&self, input: &ChairInput) -> ChairOutput {
        let mut horizon_filtered = false;
        let mut active: Vec<InvestorVote> = input
            .votes
            .iter()
            .filter(|vote| vote.voice_power > 0.0)
            .filter(|vote| {
                let keep = persona_card_by_id(&vote.persona_id)
                    .map(|card| {
                        card.evaluation
                            .horizon
                            .accepts_bars(input.signal.horizon_bars)
                    })
                    .unwrap_or(true);
                if !keep {
                    horizon_filtered = true;
                }
                keep
            })
            .cloned()
            .collect();

        if active.is_empty() {
            let mut reason_codes = vec![ReasonCode::NoActiveVoices, ReasonCode::CandidateRejected];
            if horizon_filtered {
                reason_codes.push(ReasonCode::HorizonFiltered);
            }
            return ChairOutput {
                selected_speakers: Vec::new(),
                lead_speaker: String::new(),
                forced_contrarian: false,
                council_score: 0.0,
                disagreement_score: 0.0,
                groupthink_risk: 0.0,
                size_multiplier: 0.0,
                decision: ChairDecisionKind::NoTrade,
                reason_codes,
            };
        }

        active.sort_by(|left, right| {
            let lhs = left.voice_power * left.conviction;
            let rhs = right.voice_power * right.conviction;
            rhs.total_cmp(&lhs)
        });

        let mut selected: Vec<InvestorVote> = active.iter().take(2).cloned().collect();
        let mut forced_contrarian = false;
        if self.config.allow_forced_contrarian && self.is_one_sided(&selected) {
            let selected_direction = selected
                .iter()
                .find(|vote| matches!(vote.stance, Stance::Buy | Stance::Sell))
                .map(|vote| vote.stance.direction())
                .unwrap_or(0.0);
            let already_contrarian = selected.iter().any(|vote| {
                vote.stance == Stance::NoTrade
                    || (matches!(vote.stance, Stance::Buy | Stance::Sell)
                        && vote.stance.direction() != selected_direction)
            });
            if already_contrarian {
                forced_contrarian = true;
            } else if let Some(contrarian) = active.iter().skip(selected.len()).find(|vote| {
                vote.stance == Stance::NoTrade || vote.stance.direction() != selected_direction
            }) {
                selected.push(contrarian.clone());
                forced_contrarian = true;
            }
        }

        let mut cluster_counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut weighted_direction = 0.0;
        let mut defensive_bonus = 0.0;
        let mut aggregate_risk_penalty = 0.0;
        let mut hard_veto = false;
        let mut had_cluster_penalty = false;

        for vote in &selected {
            let count = cluster_counts.entry(vote.cluster_id.clone()).or_insert(0);
            *count += 1;
            let multiplier = if self.config.cluster_penalty_enabled {
                cluster_multiplier(*count)
            } else {
                1.0
            };
            if self.config.cluster_penalty_enabled && *count > 1 {
                had_cluster_penalty = true;
            }
            let strength = vote.voice_power * vote.conviction * multiplier;
            if vote.stance == Stance::NoTrade {
                defensive_bonus += strength;
            } else if vote.stance != Stance::Abstain {
                weighted_direction += vote.stance.direction() * strength;
            }
            aggregate_risk_penalty += vote.risk_penalty * multiplier;
            if vote.veto {
                hard_veto = true;
            }
        }

        let directional_strength: f64 = selected
            .iter()
            .filter(|vote| matches!(vote.stance, Stance::Buy | Stance::Sell))
            .map(|vote| vote.voice_power * vote.conviction)
            .sum();
        let disagreement_score = if directional_strength > 0.0 {
            clamp01(1.0 - weighted_direction.abs() / directional_strength.max(1e-9))
        } else {
            0.0
        };
        let mut groupthink_risk = if self.is_one_sided(&selected) {
            0.8
        } else {
            0.25
        };
        if forced_contrarian {
            groupthink_risk *= 0.5;
        }
        if had_cluster_penalty {
            groupthink_risk =
                clamp01(groupthink_risk + self.config.cluster_groupthink_penalty.max(0.0));
        }

        let council_score = weighted_direction
            - defensive_bonus * self.config.defensive_bonus_weight.max(0.0)
            - aggregate_risk_penalty * self.config.risk_penalty_weight.max(0.0)
            - groupthink_risk * self.config.groupthink_penalty_weight.max(0.0)
            - disagreement_score * self.config.disagreement_penalty_weight.max(0.0);
        let uncertainty = clamp01(
            (input.signal.no_trade_probability + disagreement_score + groupthink_risk) / 3.0,
        );

        let mut reason_codes = Vec::new();
        if horizon_filtered {
            reason_codes.push(ReasonCode::HorizonFiltered);
        }
        if forced_contrarian {
            reason_codes.push(ReasonCode::ContrarianIncluded);
        }
        if had_cluster_penalty {
            reason_codes.push(ReasonCode::ClusterPenaltyApplied);
        }
        if groupthink_risk > 0.5 {
            reason_codes.push(ReasonCode::GroupthinkRiskElevated);
        }
        if disagreement_score > 0.4 {
            reason_codes.push(ReasonCode::DisagreementElevated);
        }

        let mut decision = if hard_veto {
            if council_score > self.config.strong_threshold {
                ChairDecisionKind::ReduceSizeCandidate
            } else {
                ChairDecisionKind::NoTrade
            }
        } else if weighted_direction < 0.0 {
            reason_codes.push(ReasonCode::ShortSellingDisabled);
            ChairDecisionKind::NoTrade
        } else if council_score > self.config.strong_threshold && uncertainty < 0.45 {
            ChairDecisionKind::ApproveCandidate
        } else if council_score > self.config.weak_threshold {
            ChairDecisionKind::RequireConfirm
        } else {
            ChairDecisionKind::NoTrade
        };

        if hard_veto {
            reason_codes.push(ReasonCode::CycleSkepticVeto);
        }
        if matches!(decision, ChairDecisionKind::RequireConfirm) && input.full_auto {
            decision = ChairDecisionKind::NoTrade;
            reason_codes.push(ReasonCode::RequireConfirmBlockedInAuto);
        }

        match decision {
            ChairDecisionKind::ApproveCandidate => reason_codes.push(ReasonCode::CandidateApproved),
            ChairDecisionKind::ReduceSizeCandidate => {
                reason_codes.push(ReasonCode::CandidateReduced)
            }
            ChairDecisionKind::NoTrade | ChairDecisionKind::RequireConfirm => {
                reason_codes.push(ReasonCode::CandidateRejected);
                if council_score <= self.config.weak_threshold {
                    reason_codes.push(ReasonCode::WeakCouncilScore);
                }
            }
        }

        let size_multiplier = match decision {
            ChairDecisionKind::ApproveCandidate => clamp01(council_score + 0.25),
            ChairDecisionKind::ReduceSizeCandidate => clamp01((council_score + 0.15).min(0.5)),
            ChairDecisionKind::NoTrade | ChairDecisionKind::RequireConfirm => 0.0,
        };

        ChairOutput {
            selected_speakers: selected
                .iter()
                .map(|vote| vote.persona_id.clone())
                .collect(),
            lead_speaker: selected
                .first()
                .map(|vote| vote.persona_id.clone())
                .unwrap_or_default(),
            forced_contrarian,
            council_score,
            disagreement_score,
            groupthink_risk,
            size_multiplier,
            decision,
            reason_codes,
        }
    }

    pub fn build_trade_proposal(
        &self,
        market: &MarketSnapshot,
        signal: &SignalOutput,
        chair_output: &ChairOutput,
    ) -> Option<TradeProposal> {
        if !matches!(
            chair_output.decision,
            ChairDecisionKind::ApproveCandidate | ChairDecisionKind::ReduceSizeCandidate
        ) {
            return None;
        }

        if chair_output.council_score <= 0.0 {
            return None;
        }

        let stop_distance = signal.expected_drawdown.clamp(0.002, 0.05);
        let mut take_profit_distance = signal.expected_return.max(stop_distance * 1.6).min(0.10);
        if take_profit_distance <= stop_distance {
            take_profit_distance = stop_distance * 1.5;
        }
        let quantity_hint = match chair_output.decision {
            ChairDecisionKind::ReduceSizeCandidate => chair_output.size_multiplier.min(0.5),
            _ => chair_output.size_multiplier,
        }
        .max(0.05);
        let expected_edge_after_cost =
            signal.expected_return - (market.spread_bps + 2.0) / 10_000.0;

        Some(TradeProposal {
            symbol: market.symbol.clone(),
            side: Side::Long,
            quantity_hint,
            entry_price_hint: market.price,
            stop_loss: Some(market.price * (1.0 - stop_distance)),
            take_profit: Some(market.price * (1.0 + take_profit_distance)),
            max_slippage_bps: market.spread_bps.max(1.0) * 1.5,
            expected_edge_after_cost,
            confidence: clamp01(signal.confidence * (1.0 - chair_output.groupthink_risk * 0.2)),
            source_chair_output: chair_output.clone(),
        })
    }

    fn is_one_sided(&self, votes: &[InvestorVote]) -> bool {
        let mut directional = votes
            .iter()
            .filter(|vote| matches!(vote.stance, Stance::Buy | Stance::Sell))
            .map(|vote| vote.stance.direction());
        if let Some(first) = directional.next() {
            directional.all(|direction| direction == first)
        } else {
            false
        }
    }
}
