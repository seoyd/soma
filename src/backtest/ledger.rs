use serde::{Deserialize, Serialize};

use crate::core::{
    ChairOutput, InvestorVote, ReasonCode, Regime, RiskDecision, SignalOutput, TradeProposal,
};
use crate::league::Horizon;

use super::{AttributionRecord, ShadowOutcomeRecord, TripleBarrierResult};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub id: String,
    pub timestamp_ms: u64,
    pub symbol: String,
    pub signal_output: SignalOutput,
    pub investor_votes: Vec<InvestorVote>,
    pub chair_output: ChairOutput,
    pub risk_decision: RiskDecision,
    pub trade_proposal: Option<TradeProposal>,
    pub selected_for_execution: bool,
    pub paper_order_id: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
    pub audit_event_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoTradeEvaluation {
    pub hypothetical_result: Option<TripleBarrierResult>,
    pub avoided_loss_score: f64,
    pub missed_gain_penalty: f64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeRecord {
    pub decision_id: String,
    pub symbol: String,
    pub timestamp_ms: u64,
    pub regime: Regime,
    pub horizon: Horizon,
    pub signal_confidence: f64,
    pub executed: bool,
    pub denied_by_risk: bool,
    pub no_trade: bool,
    pub triple_barrier_result: Option<TripleBarrierResult>,
    pub hypothetical_result: Option<TripleBarrierResult>,
    pub realized_net_return_pct: f64,
    pub avoided_loss_score: f64,
    pub missed_gain_penalty: f64,
    pub attribution_records: Vec<AttributionRecord>,
    pub shadow_outcomes: Vec<ShadowOutcomeRecord>,
    pub reason_codes: Vec<ReasonCode>,
}
