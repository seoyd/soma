use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::ReasonCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Long,
    Short,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stance {
    Buy,
    Sell,
    NoTrade,
    Abstain,
}

impl Stance {
    pub fn direction(self) -> f64 {
        match self {
            Self::Buy => 1.0,
            Self::Sell => -1.0,
            Self::NoTrade | Self::Abstain => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairDecisionKind {
    NoTrade,
    ApproveCandidate,
    ReduceSizeCandidate,
    RequireConfirm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskDecisionKind {
    Deny,
    ApprovePaper,
    Cooldown,
    EmergencyStop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Regime {
    Unknown,
    TrendUp,
    TrendDown,
    Range,
    HighVolatility,
    Panic,
    RiskOn,
    RiskOff,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonaTier {
    S,
    A,
    B,
    C,
    D,
    XQuarantined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    SignalEvaluated,
    LeagueEvaluated,
    ChairEvaluated,
    RiskEvaluated,
    PaperOrderCreated,
    SimulationCompleted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperOrderStatus {
    Accepted,
    Filled,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub symbol: String,
    pub timestamp_ms: u64,
    pub price: f64,
    pub bid: f64,
    pub ask: f64,
    pub spread_bps: f64,
    pub volume: f64,
    pub trade_value: f64,
    pub volatility: f64,
    pub regime: Regime,
    pub data_quality_score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureVector {
    pub trend_strength: f64,
    pub breakout_score: f64,
    pub liquidity_score: f64,
    pub spread_penalty: f64,
    pub volatility_score: f64,
    pub data_quality_score: f64,
    pub regime_bias: f64,
    pub overheat_score: f64,
    pub no_trade_bias: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignalOutput {
    pub symbol: String,
    pub horizon_bars: u32,
    pub p_win: f64,
    pub p_stop: f64,
    pub expected_return: f64,
    pub expected_drawdown: f64,
    pub confidence: f64,
    pub no_trade_probability: f64,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SixPrinciples {
    pub signal_edge: f64,
    pub regime_fit: f64,
    pub liquidity_fit: f64,
    pub loss_protection: f64,
    pub event_risk: f64,
    pub execution_quality: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestorVote {
    pub persona_id: String,
    pub cluster_id: String,
    pub stance: Stance,
    pub conviction: f64,
    pub voice_power: f64,
    pub veto: bool,
    pub six_principles: SixPrinciples,
    pub expected_return_adjustment: f64,
    pub risk_penalty: f64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairInput {
    pub market: MarketSnapshot,
    pub signal: SignalOutput,
    pub votes: Vec<InvestorVote>,
    pub full_auto: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairOutput {
    pub selected_speakers: Vec<String>,
    pub lead_speaker: String,
    pub forced_contrarian: bool,
    pub council_score: f64,
    pub disagreement_score: f64,
    pub groupthink_risk: f64,
    pub size_multiplier: f64,
    pub decision: ChairDecisionKind,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TradeProposal {
    pub symbol: String,
    pub side: Side,
    pub quantity_hint: f64,
    pub entry_price_hint: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub max_slippage_bps: f64,
    pub expected_edge_after_cost: f64,
    pub confidence: f64,
    pub source_chair_output: ChairOutput,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskSnapshot {
    pub daily_pnl_pct: f64,
    pub consecutive_losses: u32,
    pub current_positions_count: u32,
    pub total_exposure_pct: f64,
    pub symbol_exposure_pct: f64,
    pub api_health_score: f64,
    pub data_quality_score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrderPlan {
    pub symbol: String,
    pub side: Side,
    pub quantity: f64,
    pub entry_price: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub max_slippage_bps: f64,
    pub paper_only: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskDecision {
    pub kind: RiskDecisionKind,
    pub approved_order_plan: Option<OrderPlan>,
    pub reason_codes: Vec<ReasonCode>,
    pub audit_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperOrder {
    pub order_id: String,
    pub symbol: String,
    pub side: Side,
    pub quantity: f64,
    pub entry_price: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub paper_only: bool,
    pub status: PaperOrderStatus,
    pub timestamp_ms: u64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp_ms: u64,
    pub event_type: AuditEventType,
    pub input_hash: u64,
    pub decision_summary: String,
    pub reason_codes: Vec<ReasonCode>,
    pub numeric_snapshot: BTreeMap<String, f64>,
}
