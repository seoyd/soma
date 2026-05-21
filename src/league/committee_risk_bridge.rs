use serde::{Deserialize, Serialize};

use crate::core::{ChairOutput, RiskDecision, RiskDecisionKind, RiskSnapshot, Side, TradeProposal};
use crate::risk::RiskGovernor;
use crate::{MarketSnapshot, ReasonCode};

use super::committee_decision::{CommitteeDecision, CommitteeDecisionRecord};
use super::persona_scorer::PersonaScoringInput;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeFinalAction {
    PaperApprove,
    PaperReduceSize,
    HumanConfirmRequired,
    FinalNoTrade,
    FinalDenied,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeOutcome {
    pub committee_record: CommitteeDecisionRecord,
    pub risk_decision: RiskDecision,
    pub final_action: CommitteeFinalAction,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default)]
pub struct CommitteeRiskBridge {
    pub governor: RiskGovernor,
}

impl CommitteeRiskBridge {
    pub fn committee_decision_to_risk_proposal(
        &self,
        market: &MarketSnapshot,
        input: &PersonaScoringInput,
        record: &CommitteeDecisionRecord,
    ) -> Option<TradeProposal> {
        match record.final_decision {
            CommitteeDecision::ApproveCandidate | CommitteeDecision::ReduceSizeCandidate => {
                Some(TradeProposal {
                    symbol: input.symbol.clone(),
                    side: Side::Long,
                    quantity_hint: if matches!(
                        record.final_decision,
                        CommitteeDecision::ReduceSizeCandidate
                    ) {
                        0.15
                    } else {
                        0.30
                    },
                    entry_price_hint: market.price,
                    stop_loss: Some(market.price * (1.0 - input.expected_drawdown.max(0.01))),
                    take_profit: Some(
                        market.price * (1.0 + input.expected_edge_after_cost.abs().max(0.01) * 2.0),
                    ),
                    max_slippage_bps: market.spread_bps.max(4.0),
                    expected_edge_after_cost: input.expected_edge_after_cost,
                    confidence: record.weighted_score.abs().clamp(0.0, 1.0),
                    source_chair_output: ChairOutput {
                        selected_speakers: record.selected_speakers.clone(),
                        lead_speaker: record
                            .selected_speakers
                            .first()
                            .cloned()
                            .unwrap_or_default(),
                        forced_contrarian: record
                            .chair_reason_codes
                            .contains(&ReasonCode::ContrarianIncluded),
                        council_score: record.weighted_score,
                        disagreement_score: record.disagreement_score,
                        groupthink_risk: record.groupthink_risk,
                        size_multiplier: if matches!(
                            record.final_decision,
                            CommitteeDecision::ReduceSizeCandidate
                        ) {
                            0.5
                        } else {
                            1.0
                        },
                        decision: match record.final_decision {
                            CommitteeDecision::ApproveCandidate => {
                                crate::core::ChairDecisionKind::ApproveCandidate
                            }
                            CommitteeDecision::ReduceSizeCandidate => {
                                crate::core::ChairDecisionKind::ReduceSizeCandidate
                            }
                            CommitteeDecision::RequireHumanConfirm => {
                                crate::core::ChairDecisionKind::RequireConfirm
                            }
                            CommitteeDecision::NoTrade | CommitteeDecision::Vetoed => {
                                crate::core::ChairDecisionKind::NoTrade
                            }
                        },
                        reason_codes: record.chair_reason_codes.clone(),
                    },
                })
            }
            _ => None,
        }
    }

    pub fn risk_result_to_committee_outcome(
        &self,
        committee_record: CommitteeDecisionRecord,
        risk_decision: RiskDecision,
    ) -> CommitteeOutcome {
        let final_action = match committee_record.final_decision {
            CommitteeDecision::RequireHumanConfirm => CommitteeFinalAction::HumanConfirmRequired,
            CommitteeDecision::NoTrade => CommitteeFinalAction::FinalNoTrade,
            CommitteeDecision::Vetoed => CommitteeFinalAction::FinalDenied,
            CommitteeDecision::ApproveCandidate | CommitteeDecision::ReduceSizeCandidate => {
                match risk_decision.kind {
                    RiskDecisionKind::ApprovePaper
                        if matches!(
                            committee_record.final_decision,
                            CommitteeDecision::ReduceSizeCandidate
                        ) =>
                    {
                        CommitteeFinalAction::PaperReduceSize
                    }
                    RiskDecisionKind::ApprovePaper => CommitteeFinalAction::PaperApprove,
                    RiskDecisionKind::Deny
                    | RiskDecisionKind::Cooldown
                    | RiskDecisionKind::EmergencyStop => CommitteeFinalAction::FinalDenied,
                }
            }
        };
        CommitteeOutcome {
            committee_record,
            risk_decision,
            final_action,
            reason_codes: vec![ReasonCode::CommitteeRiskBridgeBuilt],
        }
    }

    pub fn evaluate(
        &self,
        market: &MarketSnapshot,
        risk_snapshot: &RiskSnapshot,
        input: &PersonaScoringInput,
        record: CommitteeDecisionRecord,
    ) -> CommitteeOutcome {
        let proposal = self.committee_decision_to_risk_proposal(market, input, &record);
        let risk_decision =
            self.governor
                .evaluate(market, risk_snapshot, proposal.as_ref(), input.timestamp_ms);
        self.risk_result_to_committee_outcome(record, risk_decision)
    }
}
