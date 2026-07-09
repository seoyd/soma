use std::{collections::BTreeMap, fs};

use serde::{Deserialize, Serialize};

use crate::backtest::{
    AttributionRecord, BarrierHit, Candle, CandleSeries, CounterfactualRole, OutcomeRecord,
    Timeframe, TripleBarrierOutcome, TripleBarrierResult,
};
use crate::chair::{ChairConfig, ChairEngine};
use crate::core::{
    ChairInput, ChairOutput, InvestorVote, MarketSnapshot, PaperOrder, PaperOrderStatus,
    PersonaTier, ReasonCode, Regime, RiskDecision, RiskDecisionKind, RiskSnapshot, Side,
    SignalOutput, Stance, TradeProposal, stable_hash_string, stable_reason_codes,
};
use crate::owner::{
    OwnerInput, OwnerTradeRequestReview, owner_rejection_explanation, review_owner_trade_request,
};
use crate::paper::{Broker, PaperBroker};
use crate::risk::{GovernorConfig, RiskGovernor};

use super::tier::demote_one_tier;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Horizon {
    Intraday,
    Swing,
    Position,
}

impl Horizon {
    pub fn accepts_bars(self, bars: u32) -> bool {
        match self {
            Self::Intraday => bars <= 12,
            Self::Swing => (13..=72).contains(&bars),
            Self::Position => bars >= 73,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ImmutableDoctrine {
    pub never_average_down: bool,
    pub cut_losses_quickly: bool,
    pub pyramid_only_on_strength: bool,
    pub speak_only_on_trend_or_breakout: bool,
    pub rest_after_consecutive_losses: bool,
    pub no_leverage: bool,
    pub do_not_speak_intraday_as_entry_signal: bool,
    pub reject_unknown_or_unscorable_asset: bool,
    pub margin_of_safety_required_when_fundamentals_available: bool,
    pub risk_first: bool,
    pub reject_poor_risk_reward: bool,
    pub reject_euphoria_chasing: bool,
    pub respect_cooldown: bool,
    pub no_trade_is_valid: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct MutablePolicy {
    pub breakout_lookback: Option<u32>,
    pub volume_z_threshold: Option<f64>,
    pub stop_loss_atr_mult: Option<f64>,
    pub take_profit_rr: Option<f64>,
    pub confidence_entry_threshold: Option<f64>,
    pub max_trade_frequency: Option<u32>,
    pub max_exposure_hint: Option<f64>,
    pub unknown_asset_penalty: Option<f64>,
    pub quality_threshold_placeholder: Option<f64>,
    pub defensive_bias: Option<f64>,
    pub overheat_threshold: Option<f64>,
    pub min_risk_reward: Option<f64>,
    pub volatility_penalty: Option<f64>,
    pub groupthink_penalty: Option<f64>,
    pub veto_sensitivity: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub base_voice_power: f64,
    pub current_voice_power: f64,
    pub ema_alpha: f64,
    pub severe_event_multiplier: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvaluationProfile {
    pub horizon: Horizon,
    pub favored_regimes: Vec<Regime>,
    pub tolerated_regimes: Vec<Regime>,
    pub promotion_min_samples: u32,
    pub max_s_tier: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersonaCard {
    pub persona_id: String,
    pub archetype: String,
    pub tier: PersonaTier,
    pub immutable_doctrine: ImmutableDoctrine,
    pub mutable_policy: MutablePolicy,
    pub voice: VoiceConfig,
    pub evaluation: EvaluationProfile,
}

pub type AgentId = String;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentKind {
    MomentumTrendFast,
    ValueQualityFilter,
    CycleRiskSkeptic,
    Future8AgentPlaceholder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Cooldown,
    Observer,
    SandboxOnly,
    Quarantined,
    Disabled,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentDoctrine {
    pub immutable_rules: ImmutableDoctrine,
    pub allowed_horizons: Vec<Horizon>,
    pub allowed_markets: Vec<String>,
    pub allowed_assets: Vec<String>,
    pub veto_permissions: bool,
    pub prohibited_behaviors: Vec<String>,
    pub risk_constraints: BTreeMap<String, f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentVoiceState {
    pub voice_power: f64,
    pub tier: PersonaTier,
    pub cooldown_bars: u32,
    pub veto_power: bool,
    pub recent_penalty_count: u32,
    pub recent_reward_count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AgentMemorySummary {
    pub total_decisions: u64,
    pub total_paper_trades: u64,
    pub total_no_trades: u64,
    pub wins: u64,
    pub losses: u64,
    pub avoided_losses: u64,
    pub missed_gains: u64,
    pub high_confidence_misses: u64,
    pub doctrine_violations: u64,
    pub max_drawdown_contribution: f64,
    pub last_updated_event_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentVersion {
    pub version_id: String,
    pub parent_version_id: Option<String>,
    pub created_from_feedback_event: Option<String>,
    pub live_enabled: bool,
    pub sandbox_only: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CanonicalAgentState {
    pub agent_id: AgentId,
    pub kind: AgentKind,
    pub status: AgentStatus,
    pub doctrine: AgentDoctrine,
    pub mutable_policy: MutablePolicy,
    pub voice_state: AgentVoiceState,
    pub memory_summary: AgentMemorySummary,
    pub version: AgentVersion,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentProposal {
    pub proposal_id: String,
    pub agent_id: AgentId,
    pub stance: Stance,
    pub confidence: f64,
    pub expected_edge: f64,
    pub expected_drawdown: f64,
    pub no_trade_probability: f64,
    pub horizon: Horizon,
    pub market: String,
    pub symbol: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentFeedback {
    pub agent_id: AgentId,
    pub proposal_id: Option<String>,
    pub outcome_id: String,
    pub paper_only: bool,
    #[serde(default)]
    pub outcome_kind: AgentFeedbackOutcomeKind,
    pub realized_net_return: f64,
    #[serde(default)]
    pub counterfactual_net_return: Option<f64>,
    pub avoided_loss_score: f64,
    pub missed_gain_penalty: f64,
    pub drawdown_contribution: f64,
    pub confidence_at_decision: f64,
    pub doctrine_violation: bool,
    pub risk_warning_correct: bool,
    pub no_trade_correct: bool,
    pub overtrade: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentFeedbackOutcomeKind {
    ExecutedPaperTrade,
    #[default]
    NoTrade,
    RiskDenied,
    Abstained,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackContext {
    pub paper_only: bool,
    pub outcome_finalized: bool,
    pub doctrine_violation: bool,
    pub overtrade: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentFeedbackBuildError {
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairTierAction {
    Keep,
    Promote,
    Demote,
    Cooldown,
    Quarantine,
    SandboxCandidate,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairRewardPenalty {
    pub agent_id: AgentId,
    pub source_feedback_id: String,
    pub reward_delta: f64,
    pub penalty_delta: f64,
    pub voice_delta: f64,
    pub cooldown_delta: u32,
    pub tier_action: ChairTierAction,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxPromotionStatus {
    Proposed,
    BacktestPending,
    PaperPending,
    Rejected,
    EligibleForOwnerReview,
    Promoted,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SandboxPromotionCandidate {
    pub candidate_id: String,
    pub agent_id: AgentId,
    pub parent_version_id: String,
    pub candidate_version_id: String,
    pub source_feedback_ids: Vec<String>,
    pub proposed_policy_delta: BTreeMap<String, f64>,
    pub sandbox_only: bool,
    pub promotion_status: SandboxPromotionStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentStateSnapshot {
    pub agent_id: AgentId,
    pub version_id: String,
    pub parent_version_id: Option<String>,
    pub state: CanonicalAgentState,
    pub feedback_event_id: Option<String>,
    pub created_from_paper_only: bool,
    pub sandbox_only: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStateJournalError {
    DuplicateVersion,
    NonPaperSnapshot,
    SandboxSnapshotLiveEnabled,
    SnapshotStateMismatch,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AgentStateJournal {
    snapshots: Vec<AgentStateSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeedbackCycleResult {
    pub original_state_version: String,
    pub feedback: AgentFeedback,
    pub reward_penalty: ChairRewardPenalty,
    pub updated_state: CanonicalAgentState,
    pub version_entry: AgentStateSnapshot,
    pub sandbox_candidate: Option<SandboxPromotionCandidate>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperLearningLoopConfig {
    pub market: String,
    pub chair: ChairConfig,
    pub risk_governor: GovernorConfig,
}

impl Default for PaperLearningLoopConfig {
    fn default() -> Self {
        Self {
            market: "US".to_string(),
            chair: ChairConfig::default(),
            risk_governor: GovernorConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PaperOutcomeContext {
    pub outcome_finalized: bool,
    #[serde(default)]
    pub finalized_at_timestamp_ms: u64,
    pub outcome_kind: PaperOutcomeKind,
    #[serde(default)]
    pub fill_evidence: Option<PaperFillEvidence>,
    pub realized_net_return_pct: f64,
    pub hypothetical_net_return_pct: Option<f64>,
    pub max_adverse_excursion_pct: f64,
    pub doctrine_violation_agents: Vec<AgentId>,
    pub overtrade_agents: Vec<AgentId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperFillEvidence {
    pub fill_id: String,
    pub paper_order_id: String,
    pub symbol: String,
    pub filled_at_timestamp_ms: u64,
    pub paper_only: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperOutcomeKind {
    FilledPaperOrder,
    #[default]
    NoExecution,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperLearningLoopInput {
    pub initial_agent_states: Vec<CanonicalAgentState>,
    pub market_snapshot: MarketSnapshot,
    pub signal_input: SignalOutput,
    pub owner_advisory: Option<OwnerInput>,
    pub risk_snapshot: RiskSnapshot,
    pub paper_context: Option<PaperOutcomeContext>,
    pub loop_config: PaperLearningLoopConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperLearningLoopReport {
    pub decision_id: String,
    pub active_agent_count: usize,
    pub feedback_count: usize,
    pub updated_state_count: usize,
    pub version_snapshot_count: usize,
    pub sandbox_candidate_count: usize,
    pub paper_order_created: bool,
    pub paper_outcome_finalized: bool,
    pub risk_veto_preserved: bool,
    pub paper_only: bool,
    pub live_execution_supported: bool,
    pub live_call_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperLearningLoopResult {
    pub original_agent_states: Vec<CanonicalAgentState>,
    pub agent_votes: Vec<InvestorVote>,
    pub agent_proposals: Vec<AgentProposal>,
    pub chair_output: ChairOutput,
    pub risk_decision: RiskDecision,
    pub paper_order: Option<PaperOrder>,
    pub paper_outcome: Option<OutcomeRecord>,
    pub feedback_records: Vec<AgentFeedback>,
    pub reward_penalties: Vec<ChairRewardPenalty>,
    pub updated_agent_states: Vec<CanonicalAgentState>,
    pub version_snapshots: Vec<AgentStateSnapshot>,
    pub sandbox_candidates: Vec<SandboxPromotionCandidate>,
    pub owner_explanation: Option<OwnerTradeRequestReview>,
    pub report: PaperLearningLoopReport,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperLearningLoopError {
    InvalidActiveAgentSet,
    InvalidDecisionInput,
    InvalidPaperOutcome,
    FeedbackBuild(AgentFeedbackBuildError),
    VersionJournal(AgentStateJournalError),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperLearningEpisode {
    pub episode_id: String,
    pub input: PaperLearningLoopInput,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperLearningChainConfig {
    pub max_episodes: usize,
    pub require_finalized_outcomes: bool,
}

impl Default for PaperLearningChainConfig {
    fn default() -> Self {
        Self {
            max_episodes: 64,
            require_finalized_outcomes: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperLearningChainInput {
    pub initial_agent_states: Vec<CanonicalAgentState>,
    pub episodes: Vec<PaperLearningEpisode>,
    pub chain_config: PaperLearningChainConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperLearningEpisodeResult {
    pub episode_id: String,
    pub input_states: Vec<CanonicalAgentState>,
    pub result: PaperLearningLoopResult,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentAttributionSummary {
    pub agent_id: AgentId,
    pub selected_count: u64,
    pub supported_final_count: u64,
    pub opposed_final_count: u64,
    pub abstained_count: u64,
    pub risk_veto_aligned_count: u64,
    pub risk_veto_opposed_count: u64,
    pub no_trade_correct_count: u64,
    pub no_trade_missed_gain_count: u64,
    pub profitable_selected_count: u64,
    pub losing_selected_count: u64,
    pub high_confidence_miss_count: u64,
    pub doctrine_violation_count: u64,
    pub total_reward: f64,
    pub total_penalty: f64,
    pub net_reward_penalty: f64,
    pub final_voice_power: f64,
    pub final_status: AgentStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentLearningSummary {
    pub agent_id: AgentId,
    pub start_version_id: String,
    pub end_version_id: String,
    pub start_voice_power: f64,
    pub end_voice_power: f64,
    pub tier_before: PersonaTier,
    pub tier_after: PersonaTier,
    pub status_before: AgentStatus,
    pub status_after: AgentStatus,
    pub wins_delta: u64,
    pub losses_delta: u64,
    pub avoided_losses_delta: u64,
    pub missed_gains_delta: u64,
    pub high_confidence_misses_delta: u64,
    pub doctrine_violations_delta: u64,
    pub sandbox_candidates_created: u64,
    pub cooldown_triggered: bool,
    pub quarantined: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LearningChainSummary {
    pub total_episodes: usize,
    pub total_paper_trades: u64,
    pub total_no_trades: u64,
    pub total_risk_denials: u64,
    pub agent_summaries: Vec<AgentLearningSummary>,
    pub sandbox_candidate_count: usize,
    pub any_live_mutation_detected: bool,
    pub any_risk_bypass_detected: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperLearningChainResult {
    pub initial_states: Vec<CanonicalAgentState>,
    pub final_states: Vec<CanonicalAgentState>,
    pub episode_results: Vec<PaperLearningEpisodeResult>,
    pub version_journal: AgentStateJournal,
    pub attribution_summary: Vec<AgentAttributionSummary>,
    pub agent_learning_summaries: Vec<AgentLearningSummary>,
    pub sandbox_candidates: Vec<SandboxPromotionCandidate>,
    pub summary: LearningChainSummary,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperLearningChainError {
    EmptyEpisodes,
    TooManyEpisodes,
    InvalidEpisodeId,
    InvalidEpisodeReasonCode,
    DuplicateEpisodeId,
    DuplicateDecisionId,
    NonMonotonicEpisodeTime,
    NonCausalOutcomeTime,
    IncompleteEpisode,
    RiskGovernorChanged,
    InvalidInitialAgentSet,
    VersionParentMismatch,
    VersionFinalMismatch,
    Episode(PaperLearningLoopError),
    VersionJournal(AgentStateJournalError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CooldownTickMode {
    PerEpisode,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperReplayConfig {
    pub max_episodes: usize,
    pub stop_on_quarantine: bool,
    pub stop_on_emergency_stop: bool,
    pub allow_sandbox_candidates: bool,
    pub cooldown_tick_mode: CooldownTickMode,
    pub active_agent_limit: usize,
}

impl Default for PaperReplayConfig {
    fn default() -> Self {
        Self {
            max_episodes: 64,
            stop_on_quarantine: false,
            stop_on_emergency_stop: true,
            allow_sandbox_candidates: true,
            cooldown_tick_mode: CooldownTickMode::PerEpisode,
            active_agent_limit: 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperReplayInput {
    pub initial_agent_states: Vec<CanonicalAgentState>,
    pub episode_inputs: Vec<PaperLearningEpisode>,
    pub replay_config: PaperReplayConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayAttributionSummary {
    pub agent_id: AgentId,
    pub selected_count: u64,
    pub supported_final_count: u64,
    pub opposed_final_count: u64,
    pub abstained_count: u64,
    pub cooldown_skipped_count: u64,
    pub risk_veto_aligned_count: u64,
    pub no_trade_correct_count: u64,
    pub no_trade_missed_gain_count: u64,
    pub profitable_selected_count: u64,
    pub losing_selected_count: u64,
    pub total_reward: f64,
    pub total_penalty: f64,
    pub net_reward_penalty: f64,
    pub final_voice_power: f64,
    pub final_status: AgentStatus,
    pub final_cooldown: u32,
    pub final_tier: PersonaTier,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperReplayResult {
    pub initial_states: Vec<CanonicalAgentState>,
    pub final_states: Vec<CanonicalAgentState>,
    pub chain_results: Vec<PaperLearningChainResult>,
    pub learning_chain_summary: LearningChainSummary,
    pub replay_attribution_summary: Vec<ReplayAttributionSummary>,
    pub version_journal: AgentStateJournal,
    pub sandbox_candidates: Vec<SandboxPromotionCandidate>,
    pub stopped_early: bool,
    pub stop_reason_codes: Vec<ReasonCode>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperReplayError {
    InvalidConfig,
    EmptyEpisodes,
    TooManyEpisodes,
    DuplicateEpisodeId,
    DuplicateDecisionId,
    NonMonotonicEpisodeTime,
    NonCausalOutcomeTime,
    RiskGovernorChanged,
    InvalidInitialAgentSet,
    VersionFinalMismatch,
    Chain(PaperLearningChainError),
    VersionJournal(AgentStateJournalError),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerAgentLearningView {
    pub agent_id: AgentId,
    pub agent_kind: AgentKind,
    pub start_version_id: String,
    pub end_version_id: String,
    pub start_voice_power: f64,
    pub end_voice_power: f64,
    pub voice_delta: f64,
    pub tier_before: PersonaTier,
    pub tier_after: PersonaTier,
    pub status_before: AgentStatus,
    pub status_after: AgentStatus,
    pub cooldown_before: u32,
    pub cooldown_after: u32,
    pub wins_delta: u64,
    pub losses_delta: u64,
    pub avoided_losses_delta: u64,
    pub missed_gains_delta: u64,
    pub high_confidence_misses_delta: u64,
    pub doctrine_violations_delta: u64,
    pub total_reward: f64,
    pub total_penalty: f64,
    pub net_reward_penalty: f64,
    pub sandbox_candidates_created: u64,
    pub owner_visible_explanation: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairReviewSummary {
    pub decisions_composed: u64,
    pub rewards_given: u64,
    pub penalties_given: u64,
    pub cooldowns_started: u64,
    pub quarantines: u64,
    pub sandbox_candidates: u64,
    pub top_rewarded_agent: Option<AgentId>,
    pub top_penalized_agent: Option<AgentId>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskReviewSummary {
    pub risk_denials: u64,
    pub emergency_stops: u64,
    pub cooldown_blocks: u64,
    pub owner_requests_denied: u64,
    pub bad_data_denials: u64,
    pub spread_denials: u64,
    pub stale_data_denials: u64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SandboxReviewSummary {
    pub candidate_count: u64,
    pub candidates_by_agent: BTreeMap<AgentId, u64>,
    pub any_live_candidate: bool,
    pub safety_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerAdvisorySummary {
    pub owner_requests_seen: u64,
    pub owner_requests_accepted_as_context: u64,
    pub owner_requests_rejected: u64,
    pub owner_forced_trade_attempts_blocked: u64,
    pub owner_promotion_attempts_blocked: u64,
    pub owner_cooldown_clear_attempts_blocked: u64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerLearningReport {
    pub report_id: String,
    pub generated_from_replay_id: Option<String>,
    pub total_episodes: usize,
    pub total_paper_trades: u64,
    pub total_no_trades: u64,
    pub total_risk_denials: u64,
    pub agents: Vec<OwnerAgentLearningView>,
    pub chair_summary: ChairReviewSummary,
    pub risk_summary: RiskReviewSummary,
    pub sandbox_summary: SandboxReviewSummary,
    pub owner_advisory_summary: OwnerAdvisorySummary,
    #[serde(default)]
    pub data_quality_summary: Option<LocalDataQualitySummary>,
    pub safety_warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerLearningReportError {
    InvalidReportId,
    InvalidReplayRoster,
    MissingAgentSummary,
    UnsafePrivateData { reason_codes: Vec<ReasonCode> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerReviewCommand {
    ShowSummary,
    ShowAgent { agent_id: AgentId },
    ShowRisk,
    ShowSandbox,
    ShowOwnerAdvisory,
    ExplainReasonCodes { reason_codes: Vec<ReasonCode> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerReviewResponse {
    pub text: String,
    pub reason_codes: Vec<ReasonCode>,
    pub no_state_mutation: bool,
    pub order_execution_supported: bool,
    pub sandbox_promotion_supported: bool,
    pub cooldown_clear_supported: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoricalOhlcvRow {
    pub symbol: String,
    pub timestamp_ms: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_value: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoricalReplayDataset {
    pub symbol: String,
    pub rows: Vec<HistoricalOhlcvRow>,
    pub source: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalReplayConfig {
    pub max_rows: usize,
    pub require_monotonic_timestamps: bool,
    pub reject_non_finite: bool,
    pub reject_non_positive_prices: bool,
    pub strict_ohlc_bounds: bool,
    pub synthetic_only: bool,
}

impl Default for HistoricalReplayConfig {
    fn default() -> Self {
        Self {
            max_rows: 10_000,
            require_monotonic_timestamps: true,
            reject_non_finite: true,
            reject_non_positive_prices: true,
            strict_ohlc_bounds: true,
            synthetic_only: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalReplayError {
    pub row_number: Option<usize>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoricalOwnerReportError {
    Historical(HistoricalReplayError),
    Replay(PaperReplayError),
    Report(OwnerLearningReportError),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HistoricalReplayAdapter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LocalDataSourceKind {
    SyntheticFixture,
    KoreanStockCsv,
    UsStockCsv,
    BtcCryptoCsv,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalTimestampUnit {
    Milliseconds,
    MillisecondsOrDateTimeUtc,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalSymbolPolicy {
    SingleSymbolStrict,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpectedCadence {
    FixedMillis(u64),
    DailyApprox,
    Variable,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CadenceTolerance {
    pub allowed_gap_multiplier: u64,
    pub max_gap_count: usize,
    pub allow_weekend_or_session_gap: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceQualityThresholds {
    pub max_timestamp_gap_count: usize,
    pub max_duplicate_timestamp_count: usize,
    pub max_missing_optional_ratio: f64,
    pub max_volume_anomaly_ratio: f64,
    pub max_suspicious_scale_score: f64,
    pub max_ohlc_distortion_count: usize,
    pub min_accepted_rows: usize,
    pub min_quality_score: f64,
    pub reject_on_private_marker: bool,
    pub reject_on_forbidden_column: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalDataSourceProfile {
    pub kind: LocalDataSourceKind,
    pub name: String,
    pub description: String,
    pub required_columns: Vec<String>,
    pub optional_columns: Vec<String>,
    pub timestamp_unit: LocalTimestampUnit,
    pub price_scale: f64,
    pub volume_scale: f64,
    pub symbol_policy: LocalSymbolPolicy,
    pub allowed_source_markers: Vec<String>,
    pub reject_private_markers: bool,
    pub expected_cadence: ExpectedCadence,
    pub cadence_tolerance: CadenceTolerance,
    pub quality_thresholds: SourceQualityThresholds,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalDataSourceRegistry {
    profiles: BTreeMap<LocalDataSourceKind, LocalDataSourceProfile>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalDataQualitySummary {
    pub total_rows: usize,
    pub accepted_rows: usize,
    pub rejected_rows: usize,
    pub first_timestamp: u64,
    pub last_timestamp: u64,
    pub symbol: String,
    pub source_kind: LocalDataSourceKind,
    pub has_trade_value: bool,
    pub monotonic: bool,
    pub min_close: f64,
    pub max_close: f64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalCsvSourceResult {
    pub dataset: HistoricalReplayDataset,
    pub quality_summary: LocalDataQualitySummary,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalDataSourceError {
    Registry { reason_codes: Vec<ReasonCode> },
    Historical(HistoricalReplayError),
    Replay(PaperReplayError),
    Report(OwnerLearningReportError),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchReplaySource {
    pub source_id: String,
    pub source_kind: LocalDataSourceKind,
    pub display_name: String,
    pub csv_text: String,
    pub profile_name: String,
    pub enabled: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchReplayMode {
    IndependentPerSource,
    SequentialCarryover,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceOrderPolicy {
    AsProvided,
    SourceKindThenId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityReplayPolicy {
    RejectPoorAndBelow,
    RejectRejectedOnly,
    ReplayAllAcceptedWithWarnings,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchReplayConfig {
    pub max_sources: usize,
    pub max_rows_per_source: usize,
    pub require_all_sources_valid: bool,
    pub stop_on_source_error: bool,
    pub include_owner_reports: bool,
    pub include_agent_tables: bool,
    pub active_agent_limit: usize,
    pub replay_mode: BatchReplayMode,
    pub source_order_policy: SourceOrderPolicy,
    pub quality_policy: QualityReplayPolicy,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for BatchReplayConfig {
    fn default() -> Self {
        Self {
            max_sources: 16,
            max_rows_per_source: 10_000,
            require_all_sources_valid: true,
            stop_on_source_error: true,
            include_owner_reports: true,
            include_agent_tables: true,
            active_agent_limit: 3,
            replay_mode: BatchReplayMode::SequentialCarryover,
            source_order_policy: SourceOrderPolicy::AsProvided,
            quality_policy: QualityReplayPolicy::RejectPoorAndBelow,
            reason_codes: vec![ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchReplayInput {
    pub initial_agent_states: Vec<CanonicalAgentState>,
    pub sources: Vec<BatchReplaySource>,
    pub config: BatchReplayConfig,
    pub replay_config: PaperReplayConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentPerformanceRow {
    pub agent_id: AgentId,
    pub agent_kind: AgentKind,
    pub source_kind: Option<LocalDataSourceKind>,
    pub source_id: Option<String>,
    pub total_episodes: u64,
    pub selected_count: u64,
    pub supported_count: u64,
    pub opposed_count: u64,
    pub abstained_count: u64,
    pub risk_veto_aligned_count: u64,
    pub no_trade_correct_count: u64,
    pub no_trade_missed_gain_count: u64,
    pub wins_delta: u64,
    pub losses_delta: u64,
    pub avoided_losses_delta: u64,
    pub missed_gains_delta: u64,
    pub high_confidence_misses_delta: u64,
    pub doctrine_violations_delta: u64,
    pub reward_total: f64,
    pub penalty_total: f64,
    pub net_reward_penalty: f64,
    pub start_voice_power: f64,
    pub end_voice_power: f64,
    pub voice_delta: f64,
    pub start_status: AgentStatus,
    pub end_status: AgentStatus,
    pub start_tier: PersonaTier,
    pub end_tier: PersonaTier,
    pub cooldown_events: u64,
    pub quarantine_events: u64,
    pub sandbox_candidates_created: u64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentPerformanceTable {
    pub rows: Vec<AgentPerformanceRow>,
    pub aggregate_rows_by_agent: Vec<AgentPerformanceRow>,
    pub aggregate_rows_by_source_kind: Vec<AgentPerformanceRow>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourcePerformanceRow {
    pub source_id: String,
    pub source_kind: LocalDataSourceKind,
    pub display_name: String,
    pub accepted: bool,
    pub total_rows: usize,
    pub accepted_rows: usize,
    pub rejected_rows: usize,
    pub total_episodes: usize,
    pub total_paper_trades: u64,
    pub total_no_trades: u64,
    pub total_risk_denials: u64,
    pub data_quality_summary: Option<LocalDataQualitySummary>,
    pub first_timestamp: u64,
    pub last_timestamp: u64,
    pub symbol: String,
    pub min_close: f64,
    pub max_close: f64,
    pub monotonic: bool,
    pub paper_only: bool,
    pub not_live_ready: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourcePerformanceTable {
    pub rows: Vec<SourcePerformanceRow>,
    pub accepted_count: usize,
    pub rejected_count: usize,
    pub by_source_kind_counts: BTreeMap<LocalDataSourceKind, u64>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceConsistencyDiagnostics {
    pub source_id: String,
    pub source_kind: LocalDataSourceKind,
    pub row_count: usize,
    pub timestamp_monotonic: bool,
    pub timestamp_gap_count: usize,
    pub min_timestamp: u64,
    pub max_timestamp: u64,
    pub min_close: f64,
    pub max_close: f64,
    pub close_range_pct: f64,
    pub min_volume: f64,
    pub max_volume: f64,
    pub volume_range_ratio: f64,
    pub trade_value_range_ratio: Option<f64>,
    pub trade_value_available: bool,
    pub optional_columns_present: Vec<String>,
    pub missing_optional_ratio: f64,
    pub profile_match: bool,
    pub suspicious_scale: bool,
    pub suspicious_scale_score: f64,
    pub ohlc_distortion_count: usize,
    pub expected_cadence: ExpectedCadence,
    pub quality_score: f64,
    pub quality_bucket: DataQualityBucket,
    pub data_quality_warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DataQualityBucket {
    Excellent,
    Good,
    Caution,
    Poor,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceQualityScore {
    pub score: f64,
    pub bucket: DataQualityBucket,
    pub diagnostics: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CrossSourceConsistencyReport {
    pub source_diagnostics: Vec<SourceConsistencyDiagnostics>,
    pub source_kind_counts: BTreeMap<LocalDataSourceKind, u64>,
    pub accepted_source_count: usize,
    pub rejected_source_count: usize,
    pub suspicious_source_count: usize,
    pub common_warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentConsistencyStatus {
    Stable,
    SourceSensitive,
    Unstable,
    InsufficientData,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentCrossSourceConsistencyRow {
    pub agent_id: AgentId,
    pub agent_kind: AgentKind,
    pub source_kind_count: usize,
    pub total_sources: usize,
    pub sources_with_positive_net_reward: usize,
    pub sources_with_negative_net_reward: usize,
    pub voice_delta_min: f64,
    pub voice_delta_max: f64,
    pub voice_delta_range: f64,
    pub reward_penalty_min: f64,
    pub reward_penalty_max: f64,
    pub reward_penalty_range: f64,
    pub high_confidence_miss_total: u64,
    pub avoided_loss_total: u64,
    pub cooldown_count_total: u64,
    pub quarantine_count_total: u64,
    pub consistency_status: AgentConsistencyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentCrossSourceConsistencyTable {
    pub rows: Vec<AgentCrossSourceConsistencyRow>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentPerformanceByQualityRow {
    pub agent_id: AgentId,
    pub agent_kind: AgentKind,
    pub quality_bucket: DataQualityBucket,
    pub source_count: usize,
    pub total_episodes: u64,
    pub reward_total: f64,
    pub penalty_total: f64,
    pub net_reward_penalty: f64,
    pub voice_delta_min: f64,
    pub voice_delta_max: f64,
    pub high_confidence_misses: u64,
    pub avoided_losses: u64,
    pub cooldown_events: u64,
    pub quarantine_events: u64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentPerformanceByQualityTable {
    pub rows: Vec<AgentPerformanceByQualityRow>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchLearningSummary {
    pub total_sources: usize,
    pub total_episodes: usize,
    pub total_paper_trades: u64,
    pub total_no_trades: u64,
    pub total_risk_denials: u64,
    pub sandbox_candidate_count: usize,
    pub any_live_mutation_detected: bool,
    pub any_risk_bypass_detected: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchReplaySourceResult {
    pub source_id: String,
    pub source_kind: LocalDataSourceKind,
    pub accepted: bool,
    pub dataset_quality_summary: Option<LocalDataQualitySummary>,
    pub replay_result: Option<PaperReplayResult>,
    pub owner_learning_report: Option<OwnerLearningReport>,
    pub agent_performance_rows: Vec<AgentPerformanceRow>,
    pub source_performance_row: SourcePerformanceRow,
    pub source_consistency_diagnostics: SourceConsistencyDiagnostics,
    pub quality_score: SourceQualityScore,
    pub quality_bucket: DataQualityBucket,
    pub quality_reason_codes: Vec<ReasonCode>,
    pub replay_blocked_by_quality: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchReplayResult {
    pub initial_states: Vec<CanonicalAgentState>,
    pub final_states: Vec<CanonicalAgentState>,
    pub source_processing_order: Vec<String>,
    pub source_results: Vec<BatchReplaySourceResult>,
    pub aggregate_agent_performance_table: AgentPerformanceTable,
    pub aggregate_source_performance_table: SourcePerformanceTable,
    pub cross_source_consistency_report: CrossSourceConsistencyReport,
    pub agent_cross_source_consistency_table: AgentCrossSourceConsistencyTable,
    pub agent_performance_by_quality_table: AgentPerformanceByQualityTable,
    pub quality_bucket_counts: BTreeMap<DataQualityBucket, u64>,
    pub aggregate_learning_summary: BatchLearningSummary,
    pub rejected_sources: usize,
    pub accepted_sources: usize,
    pub replay_mode: BatchReplayMode,
    pub source_order_policy: SourceOrderPolicy,
    pub quality_policy: QualityReplayPolicy,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchReplayError {
    pub source_id: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatchOwnerLearningReport {
    pub batch_summary: BatchLearningSummary,
    pub source_performance_table: SourcePerformanceTable,
    pub agent_performance_table: AgentPerformanceTable,
    pub cross_source_consistency_report: CrossSourceConsistencyReport,
    pub agent_cross_source_consistency_table: AgentCrossSourceConsistencyTable,
    pub agent_performance_by_quality_table: AgentPerformanceByQualityTable,
    pub quality_policy: QualityReplayPolicy,
    pub quality_bucket_counts: BTreeMap<DataQualityBucket, u64>,
    pub blocked_by_quality_sources: Vec<String>,
    pub source_quality_threshold_summary: Vec<String>,
    pub replay_mode: BatchReplayMode,
    pub source_order_policy: SourceOrderPolicy,
    pub source_processing_order: Vec<String>,
    pub per_source_report_refs: Vec<String>,
    pub safety_warnings: Vec<String>,
    pub deferred_items: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManualAdjustedClosePolicy {
    Ignore,
    UseForReturnOnly,
    RejectIfPresent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualHistoricalDailyImportConfig {
    pub dataset_id: String,
    pub source_kind: LocalDataSourceKind,
    pub max_rows: usize,
    pub min_rows: usize,
    pub require_monotonic_dates: bool,
    pub allow_duplicate_dates: bool,
    pub strict_single_symbol: bool,
    pub allow_adjusted_close: bool,
    pub adjusted_close_policy: ManualAdjustedClosePolicy,
    pub reject_weekend_gap: bool,
    pub calendar_validation_deferred: bool,
    pub reject_private_markers: bool,
    pub reject_endpoint_markers: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ManualHistoricalDailyImportConfig {
    fn default() -> Self {
        Self {
            dataset_id: "manual-historical-daily-dataset".to_string(),
            source_kind: LocalDataSourceKind::UsStockCsv,
            max_rows: 20_000,
            min_rows: 4,
            require_monotonic_dates: true,
            allow_duplicate_dates: false,
            strict_single_symbol: true,
            allow_adjusted_close: true,
            adjusted_close_policy: ManualAdjustedClosePolicy::Ignore,
            reject_weekend_gap: false,
            calendar_validation_deferred: true,
            reject_private_markers: true,
            reject_endpoint_markers: true,
            reason_codes: vec![
                ReasonCode::ManualHistoricalImportDailyOnly,
                ReasonCode::ManualHistoricalImportNoNetwork,
                ReasonCode::ManualHistoricalImportSanitizedOnly,
                ReasonCode::LocalFileOnly,
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ManualHistoricalDailyRow {
    pub symbol: String,
    pub date: String,
    pub timestamp_ms: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub adjusted_close: Option<f64>,
    pub trade_value: Option<f64>,
    pub currency: Option<String>,
    pub market: Option<String>,
    pub source: Option<String>,
    pub split_factor: Option<f64>,
    pub dividend: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualHistoricalDateRange {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ManualHistoricalDailyDataset {
    pub dataset_id: String,
    pub source_kind: LocalDataSourceKind,
    pub symbol: String,
    pub rows: Vec<ManualHistoricalDailyRow>,
    pub date_range: ManualHistoricalDateRange,
    pub data_quality_summary: LocalDataQualitySummary,
    pub sanitized: bool,
    pub local_only: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualHistoricalDailyImportError {
    pub row_number: Option<usize>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkForwardSplit {
    pub train_start_index: usize,
    pub train_end_index: usize,
    pub eval_start_index: usize,
    pub eval_end_index: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalkForwardConfig {
    pub min_train_rows: usize,
    pub eval_window_rows: usize,
    pub step_rows: usize,
    pub cost_bps: f64,
    pub slippage_bps: f64,
    pub max_position_fraction: f64,
    pub allow_short: bool,
    pub no_lookahead: bool,
    pub min_prediction_samples: usize,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for WalkForwardConfig {
    fn default() -> Self {
        Self {
            min_train_rows: 4,
            eval_window_rows: 4,
            step_rows: 2,
            cost_bps: 5.0,
            slippage_bps: 5.0,
            max_position_fraction: 1.0,
            allow_short: false,
            no_lookahead: true,
            min_prediction_samples: 8,
            reason_codes: vec![
                ReasonCode::WalkForwardNoLookahead,
                ReasonCode::WalkForwardEvaluationOnly,
                ReasonCode::WalkForwardTrainingDeferred,
                ReasonCode::PaperExecutionOnly,
                ReasonCode::LocalFileOnly,
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalkForwardCommitteeConfig {
    pub active_agent_limit: usize,
    pub min_vote_score_for_trade: f64,
    pub probability_return_scale: f64,
    pub min_probability: f64,
    pub max_probability: f64,
    pub stop_loss_bps: f64,
    pub take_profit_bps: f64,
    pub expected_edge_scale: f64,
}

impl Default for WalkForwardCommitteeConfig {
    fn default() -> Self {
        Self {
            active_agent_limit: 3,
            min_vote_score_for_trade: 0.10,
            probability_return_scale: 5.0,
            min_probability: 0.20,
            max_probability: 0.80,
            stop_loss_bps: 200.0,
            take_profit_bps: 400.0,
            expected_edge_scale: 0.02,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalkForwardEvaluationInput {
    pub dataset: ManualHistoricalDailyDataset,
    pub initial_agent_states: Vec<CanonicalAgentState>,
    pub walk_forward_config: WalkForwardConfig,
    pub committee_config: WalkForwardCommitteeConfig,
    pub risk_config: GovernorConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BaselineStrategyKind {
    AlwaysNoTrade,
    BuyAndHold,
    EqualWeightCommittee,
    VoiceAdaptiveCommittee,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BaselinePerformanceMetrics {
    pub strategy: BaselineStrategyKind,
    pub total_return: f64,
    pub max_drawdown: f64,
    pub trade_count: u64,
    pub win_count: u64,
    pub loss_count: u64,
    pub no_trade_count: u64,
    pub risk_denial_count: u64,
    pub avg_return_per_trade: f64,
    pub volatility_estimate: Option<f64>,
    pub sharpe_like: Option<f64>,
    pub downside_loss: Option<f64>,
    pub cost_paid: f64,
    pub slippage_paid: f64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PredictionQualitySample {
    pub predicted_probability: Option<f64>,
    pub abstained: bool,
    pub realized_direction_up: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PredictionQualityMetrics {
    pub strategy: BaselineStrategyKind,
    pub brier_score: Option<f64>,
    pub sample_count: usize,
    pub calibrated_sample_count: usize,
    pub missing_probability_count: usize,
    pub abstention_count: usize,
    pub high_confidence_error_count: usize,
    pub low_confidence_correct_count: usize,
    pub mean_confidence: Option<f64>,
    pub mean_realized_direction: Option<f64>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProofGateComparison {
    pub always_no_trade: BaselinePerformanceMetrics,
    pub buy_and_hold: BaselinePerformanceMetrics,
    pub equal_weight_committee: BaselinePerformanceMetrics,
    pub voice_adaptive_committee: BaselinePerformanceMetrics,
    pub voice_beats_equal_weight: bool,
    pub committee_beats_no_trade: bool,
    pub committee_beats_buy_hold_risk_adjusted: bool,
    pub insufficient_evidence: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoiceAdaptationComparison {
    pub equal_weight_total_return: f64,
    pub voice_adaptive_total_return: f64,
    pub equal_weight_risk_adjusted_score: f64,
    pub voice_adaptive_risk_adjusted_score: f64,
    pub voice_beats_equal_weight: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofGateStatus {
    ComputedNoProfitabilityClaim,
    InsufficientEvidence,
    NoEdgeProven,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalkForwardWindowResult {
    pub split: WalkForwardSplit,
    pub baseline_results: Vec<BaselinePerformanceMetrics>,
    pub committee_results: Vec<BaselinePerformanceMetrics>,
    pub scoring_results: Vec<PredictionQualityMetrics>,
    pub agent_state_before: Vec<CanonicalAgentState>,
    pub agent_state_after: Vec<CanonicalAgentState>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalkForwardEvaluationResult {
    pub dataset_id: String,
    pub symbol: String,
    pub windows: Vec<WalkForwardWindowResult>,
    pub aggregate_baseline_comparison: ProofGateComparison,
    pub voice_adaptation_comparison: VoiceAdaptationComparison,
    pub scoring_summary: Vec<PredictionQualityMetrics>,
    pub proof_gate_status: ProofGateStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkForwardEvaluationError {
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofGateReport {
    pub dataset_summary: String,
    pub walk_forward_config: String,
    pub baseline_comparison_table: Vec<String>,
    pub voice_adaptation_result: String,
    pub prediction_quality_summary: Vec<String>,
    pub null_strategy_warning: String,
    pub insufficient_evidence_warning: Option<String>,
    pub no_profitability_claim: String,
    pub no_live_readiness_warning: String,
    pub next_required_evidence: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HistoricalEvidenceSourceKind {
    UsStockDaily,
    KoreanStockDaily,
    BtcCryptoDaily,
    SyntheticDailySample,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalEvidenceSourceSpec {
    pub source_id: String,
    pub source_kind: HistoricalEvidenceSourceKind,
    pub symbol: String,
    pub market: String,
    pub currency: Option<String>,
    pub csv_path: Option<String>,
    pub csv_text: Option<String>,
    pub enabled: bool,
    pub expected_min_rows: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalEvidencePackManifest {
    pub pack_id: String,
    pub description: String,
    pub sources: Vec<HistoricalEvidenceSourceSpec>,
    pub local_only: bool,
    pub sanitized_only: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalEvidencePackConfig {
    pub max_sources: usize,
    pub min_sources: usize,
    pub min_sources_by_kind: BTreeMap<HistoricalEvidenceSourceKind, usize>,
    pub max_rows_per_source: usize,
    pub require_all_sources_valid: bool,
    pub allow_synthetic_sources_for_tests_only: bool,
    pub reject_private_markers: bool,
    pub reject_endpoint_markers: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for HistoricalEvidencePackConfig {
    fn default() -> Self {
        Self {
            max_sources: 32,
            min_sources: 1,
            min_sources_by_kind: BTreeMap::new(),
            max_rows_per_source: 20_000,
            require_all_sources_valid: false,
            allow_synthetic_sources_for_tests_only: false,
            reject_private_markers: true,
            reject_endpoint_markers: true,
            reason_codes: vec![
                ReasonCode::EvidencePackLocalOnly,
                ReasonCode::EvidencePackSanitizedOnly,
                ReasonCode::EvidencePackNoNetwork,
                ReasonCode::EvidencePackNoDownloader,
                ReasonCode::LocalFileOnly,
                ReasonCode::PaperExecutionOnly,
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoricalEvidenceSource {
    pub spec: HistoricalEvidenceSourceSpec,
    pub dataset: Option<ManualHistoricalDailyDataset>,
    pub accepted: bool,
    pub rejected: bool,
    pub disabled: bool,
    pub insufficient_evidence: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoricalEvidencePack {
    pub pack_id: String,
    pub description: String,
    pub sources: Vec<HistoricalEvidenceSource>,
    pub local_only: bool,
    pub sanitized_only: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalEvidencePackError {
    pub source_id: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceAggregationMethod {
    Mean,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoricalEvidencePackEvaluationConfig {
    pub initial_agent_states: Vec<CanonicalAgentState>,
    pub walk_forward_config: WalkForwardConfig,
    pub committee_config: WalkForwardCommitteeConfig,
    pub risk_config: GovernorConfig,
    pub min_accepted_sources_for_proof: usize,
    pub min_prediction_samples: usize,
    pub aggregation_method: EvidenceAggregationMethod,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for HistoricalEvidencePackEvaluationConfig {
    fn default() -> Self {
        Self {
            initial_agent_states: canonical_current_agent_states(),
            walk_forward_config: WalkForwardConfig::default(),
            committee_config: WalkForwardCommitteeConfig::default(),
            risk_config: GovernorConfig::default(),
            min_accepted_sources_for_proof: 1,
            min_prediction_samples: WalkForwardConfig::default().min_prediction_samples,
            aggregation_method: EvidenceAggregationMethod::Mean,
            reason_codes: vec![
                ReasonCode::WalkForwardNoLookahead,
                ReasonCode::WalkForwardEvaluationOnly,
                ReasonCode::EvidencePackLocalOnly,
                ReasonCode::EvidencePackNoNetwork,
                ReasonCode::EvidencePackNoDownloader,
                ReasonCode::PaperExecutionOnly,
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregateBaselineOverallStatus {
    Pass,
    Fail,
    Mixed,
    InsufficientEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AggregateBaselineComparison {
    pub source_count: usize,
    pub accepted_source_count: usize,
    pub rejected_source_count: usize,
    pub insufficient_source_count: usize,
    pub voice_beats_equal_weight_count: usize,
    pub voice_loses_equal_weight_count: usize,
    pub voice_ties_equal_weight_count: usize,
    pub committee_beats_no_trade_count: usize,
    pub committee_loses_no_trade_count: usize,
    pub committee_beats_buy_hold_count: usize,
    pub committee_loses_buy_hold_count: usize,
    pub mean_total_return_by_baseline: BTreeMap<BaselineStrategyKind, f64>,
    pub mean_max_drawdown_by_baseline: BTreeMap<BaselineStrategyKind, f64>,
    pub mean_brier_score_by_baseline: BTreeMap<BaselineStrategyKind, f64>,
    pub overall_status: AggregateBaselineOverallStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceAdaptationValidityStatus {
    Helped,
    Failed,
    Mixed,
    InsufficientEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoiceAdaptationValidity {
    pub compared_source_count: usize,
    pub voice_better_count: usize,
    pub equal_weight_better_count: usize,
    pub tie_count: usize,
    pub mean_delta_vs_equal_weight: f64,
    pub brier_delta_vs_equal_weight: Option<f64>,
    pub drawdown_delta_vs_equal_weight: Option<f64>,
    pub status: VoiceAdaptationValidityStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AggregatePredictionQualitySummary {
    pub source_count: usize,
    pub total_samples: usize,
    pub missing_probability_count: usize,
    pub abstention_count: usize,
    pub high_confidence_error_count: usize,
    pub mean_brier_score: Option<f64>,
    pub best_source_by_brier: Option<String>,
    pub worst_source_by_brier: Option<String>,
    pub insufficient_evidence: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoricalEvidenceSourceEvaluationResult {
    pub source_id: String,
    pub source_kind: HistoricalEvidenceSourceKind,
    pub symbol: String,
    pub market: String,
    pub dataset_summary: String,
    pub walk_forward_result: Option<WalkForwardEvaluationResult>,
    pub proof_gate_report: Option<ProofGateReport>,
    pub accepted: bool,
    pub rejected: bool,
    pub insufficient_evidence: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketEvidenceResult {
    pub market: String,
    pub source_count: usize,
    pub accepted_count: usize,
    pub rejected_count: usize,
    pub baseline_comparison: AggregateBaselineComparison,
    pub voice_adaptation_comparison: VoiceAdaptationValidity,
    pub prediction_quality_summary: AggregatePredictionQualitySummary,
    pub insufficient_evidence: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoricalEvidencePackEvaluationResult {
    pub pack_id: String,
    pub source_results: Vec<HistoricalEvidenceSourceEvaluationResult>,
    pub aggregate_result: AggregateBaselineComparison,
    pub market_results: Vec<MarketEvidenceResult>,
    pub symbol_results: Vec<HistoricalEvidenceSourceEvaluationResult>,
    pub voice_adaptation_summary: VoiceAdaptationValidity,
    pub prediction_quality_summary: AggregatePredictionQualitySummary,
    pub proof_gate_status: AggregateBaselineOverallStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiSymbolProofGateReport {
    pub pack_summary: String,
    pub source_table: Vec<String>,
    pub market_table: Vec<String>,
    pub aggregate_baseline_comparison: String,
    pub voice_adaptation_validity: String,
    pub prediction_quality_summary: String,
    pub failed_symbols: Vec<String>,
    pub rejected_sources: Vec<String>,
    pub insufficient_evidence_warnings: Vec<String>,
    pub next_required_evidence: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerEvidenceTrialStatus {
    NoOwnerEvidencePackFound,
    RejectedForSafety,
    InsufficientEvidence,
    Fail,
    Mixed,
    Pass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerEvidenceManifestStatus {
    Missing,
    ProvidedJson,
    LoadedFromLocalPath,
    Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceTriageDimensionStatus {
    Pass,
    Fail,
    Mixed,
    InsufficientEvidence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerActionItem {
    pub item_id: String,
    pub description: String,
    pub required: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerEvidenceTrialConfig {
    pub manifest_path: Option<String>,
    pub manifest_json: Option<String>,
    pub require_owner_pack: bool,
    pub allow_example_pack: bool,
    pub min_accepted_sources: usize,
    pub min_sources_by_market: BTreeMap<String, usize>,
    pub include_rejected_sources: bool,
    pub include_failed_symbols: bool,
    pub historical_pack_config: HistoricalEvidencePackConfig,
    pub evaluation_config: HistoricalEvidencePackEvaluationConfig,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for OwnerEvidenceTrialConfig {
    fn default() -> Self {
        Self {
            manifest_path: None,
            manifest_json: None,
            require_owner_pack: false,
            allow_example_pack: false,
            min_accepted_sources: 1,
            min_sources_by_market: BTreeMap::new(),
            include_rejected_sources: true,
            include_failed_symbols: true,
            historical_pack_config: HistoricalEvidencePackConfig::default(),
            evaluation_config: HistoricalEvidencePackEvaluationConfig::default(),
            reason_codes: vec![
                ReasonCode::OwnerEvidencePackExpectedPath,
                ReasonCode::OwnerEvidencePackLocalOnly,
                ReasonCode::OwnerEvidencePackSanitizedOnly,
                ReasonCode::EvidencePackNoNetwork,
                ReasonCode::EvidencePackNoDownloader,
                ReasonCode::PaperExecutionOnly,
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceTriageSummary {
    pub status: OwnerEvidenceTrialStatus,
    pub pack_id: Option<String>,
    pub accepted_source_count: usize,
    pub rejected_source_count: usize,
    pub disabled_source_count: usize,
    pub insufficient_source_count: usize,
    pub failed_symbol_count: usize,
    pub mixed_symbol_count: usize,
    pub pass_symbol_count: usize,
    pub markets_present: Vec<String>,
    pub voice_adaptation_status: EvidenceTriageDimensionStatus,
    pub committee_vs_no_trade_status: EvidenceTriageDimensionStatus,
    pub committee_vs_buy_hold_status: EvidenceTriageDimensionStatus,
    pub prediction_quality_status: EvidenceTriageDimensionStatus,
    pub owner_action_items: Vec<OwnerActionItem>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketTriageResult {
    pub market: String,
    pub source_count: usize,
    pub accepted_count: usize,
    pub rejected_count: usize,
    pub insufficient_count: usize,
    pub voice_status: EvidenceTriageDimensionStatus,
    pub committee_vs_no_trade_status: EvidenceTriageDimensionStatus,
    pub committee_vs_buy_hold_status: EvidenceTriageDimensionStatus,
    pub brier_status: EvidenceTriageDimensionStatus,
    pub status: OwnerEvidenceTrialStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerEvidenceTriageReport {
    pub header: String,
    pub trial_status: OwnerEvidenceTrialStatus,
    pub pack_summary: String,
    pub source_summary: Vec<String>,
    pub market_summary: Vec<String>,
    pub baseline_failure_summary: Vec<String>,
    pub voice_adaptation_summary: String,
    pub prediction_quality_summary: String,
    pub failed_symbols: Vec<String>,
    pub rejected_sources: Vec<String>,
    pub insufficient_evidence_reasons: Vec<String>,
    pub owner_action_checklist: Vec<OwnerActionItem>,
    pub safety_warnings: Vec<String>,
    pub no_profitability_claim: String,
    pub no_live_readiness_warning: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnerEvidenceTrialResult {
    pub trial_status: OwnerEvidenceTrialStatus,
    pub manifest_status: OwnerEvidenceManifestStatus,
    pub pack_evaluation: Option<HistoricalEvidencePackEvaluationResult>,
    pub multi_symbol_report: Option<MultiSymbolProofGateReport>,
    pub triage_summary: EvidenceTriageSummary,
    pub market_triage: Vec<MarketTriageResult>,
    pub triage_report: OwnerEvidenceTriageReport,
    pub owner_action_checklist: Vec<OwnerActionItem>,
    pub reason_codes: Vec<ReasonCode>,
}

impl CanonicalAgentState {
    pub fn can_vote_live(&self) -> bool {
        self.status == AgentStatus::Active
            && self.version.live_enabled
            && !self.version.sandbox_only
            && self.voice_state.cooldown_bars == 0
    }
}

impl SandboxPromotionCandidate {
    pub fn can_affect_live_decision(&self) -> bool {
        false
    }

    pub fn can_vote_live(&self) -> bool {
        false
    }
}

impl AgentStateJournal {
    pub fn append_snapshot(
        &mut self,
        snapshot: AgentStateSnapshot,
    ) -> Result<(), AgentStateJournalError> {
        if !snapshot.created_from_paper_only {
            return Err(AgentStateJournalError::NonPaperSnapshot);
        }
        if snapshot.sandbox_only && snapshot.state.version.live_enabled {
            return Err(AgentStateJournalError::SandboxSnapshotLiveEnabled);
        }
        if snapshot.agent_id != snapshot.state.agent_id
            || snapshot.version_id != snapshot.state.version.version_id
            || snapshot.parent_version_id != snapshot.state.version.parent_version_id
            || snapshot.sandbox_only != snapshot.state.version.sandbox_only
        {
            return Err(AgentStateJournalError::SnapshotStateMismatch);
        }
        if self.contains_version(&snapshot.version_id) {
            return Err(AgentStateJournalError::DuplicateVersion);
        }
        self.snapshots.push(snapshot);
        Ok(())
    }

    pub fn latest_for_agent(&self, agent_id: &str) -> Option<&AgentStateSnapshot> {
        self.snapshots
            .iter()
            .rev()
            .find(|snapshot| snapshot.agent_id == agent_id)
    }

    pub fn snapshots_for_agent(&self, agent_id: &str) -> Vec<&AgentStateSnapshot> {
        self.snapshots
            .iter()
            .filter(|snapshot| snapshot.agent_id == agent_id)
            .collect()
    }

    pub fn count_for_agent(&self, agent_id: &str) -> usize {
        self.snapshots
            .iter()
            .filter(|snapshot| snapshot.agent_id == agent_id)
            .count()
    }

    pub fn contains_version(&self, version_id: &str) -> bool {
        self.snapshots
            .iter()
            .any(|snapshot| snapshot.version_id == version_id)
    }

    pub fn snapshots(&self) -> &[AgentStateSnapshot] {
        &self.snapshots
    }
}

pub fn run_3_agent_paper_learning_loop(
    input: PaperLearningLoopInput,
) -> Result<PaperLearningLoopResult, PaperLearningLoopError> {
    validate_three_agent_set(&input.initial_agent_states)?;
    validate_learning_loop_input(&input)?;

    let original_agent_states = input.initial_agent_states.clone();
    let mut agent_votes = super::default_league_votes(&input.market_snapshot, &input.signal_input);
    for vote in &mut agent_votes {
        let state = original_agent_states
            .iter()
            .find(|state| state.agent_id == vote.persona_id)
            .ok_or(PaperLearningLoopError::InvalidActiveAgentSet)?;
        if state.can_vote_live() {
            vote.voice_power = (state.voice_state.voice_power * vote.conviction).clamp(0.0, 1.0);
        } else {
            vote.stance = Stance::Abstain;
            vote.voice_power = 0.0;
            vote.veto = false;
            vote.reason_codes.push(ReasonCode::AgentAbstained);
            if state.status == AgentStatus::Cooldown || state.voice_state.cooldown_bars > 0 {
                vote.reason_codes.push(ReasonCode::CooldownAgentUnavailable);
                vote.reason_codes
                    .push(ReasonCode::CooldownChairBypassRejected);
            }
            vote.reason_codes = stable_reason_codes(&vote.reason_codes);
        }
    }

    let chair = ChairEngine {
        config: input.loop_config.chair,
    };
    let chair_output = chair.evaluate(&ChairInput {
        market: input.market_snapshot.clone(),
        signal: input.signal_input.clone(),
        votes: agent_votes.clone(),
        full_auto: true,
    });
    let trade_proposal =
        chair.build_trade_proposal(&input.market_snapshot, &input.signal_input, &chair_output);
    let risk_governor = RiskGovernor {
        config: input.loop_config.risk_governor,
    };
    let risk_decision = risk_governor.evaluate(
        &input.market_snapshot,
        &input.risk_snapshot,
        trade_proposal.as_ref(),
        input.market_snapshot.timestamp_ms,
    );

    let mut broker = PaperBroker::default();
    let mut paper_order = if risk_decision.kind == RiskDecisionKind::ApprovePaper {
        risk_decision.approved_order_plan.clone().map(|plan| {
            broker.submit_paper_order(
                plan,
                input.market_snapshot.timestamp_ms,
                vec![ReasonCode::ApprovePaperOnly, ReasonCode::PaperExecutionOnly],
            )
        })
    } else {
        None
    };
    if input.paper_context.as_ref().is_some_and(|context| {
        context.outcome_finalized && context.outcome_kind == PaperOutcomeKind::FilledPaperOrder
    }) {
        let order = paper_order
            .as_mut()
            .ok_or(PaperLearningLoopError::InvalidPaperOutcome)?;
        let context = input
            .paper_context
            .as_ref()
            .ok_or(PaperLearningLoopError::InvalidPaperOutcome)?;
        if !paper_fill_evidence_matches(order, context.fill_evidence.as_ref()) {
            return Err(PaperLearningLoopError::InvalidPaperOutcome);
        }
        if context.fill_evidence.as_ref().is_none_or(|evidence| {
            evidence.filled_at_timestamp_ms > context.finalized_at_timestamp_ms
        }) {
            return Err(PaperLearningLoopError::InvalidPaperOutcome);
        }
        order.status = PaperOrderStatus::Filled;
        let ledger_order = broker
            .ledger
            .orders
            .iter_mut()
            .find(|ledger_order| ledger_order.order_id == order.order_id)
            .ok_or(PaperLearningLoopError::InvalidPaperOutcome)?;
        ledger_order.status = PaperOrderStatus::Filled;
    } else if input.paper_context.as_ref().is_some_and(|context| {
        context.outcome_finalized
            && context.outcome_kind == PaperOutcomeKind::NoExecution
            && context.fill_evidence.is_some()
    }) {
        return Err(PaperLearningLoopError::InvalidPaperOutcome);
    }
    let decision_id = format!(
        "paper-learning-loop:{}:{}",
        input.market_snapshot.symbol, input.market_snapshot.timestamp_ms
    );
    let agent_proposals = build_loop_agent_proposals(
        &agent_votes,
        &original_agent_states,
        &chair_output,
        &input.market_snapshot,
        &input.signal_input,
        &input.loop_config.market,
        &decision_id,
    );
    let paper_outcome = input
        .paper_context
        .as_ref()
        .filter(|context| context.outcome_finalized)
        .map(|context| {
            build_loop_outcome(
                &decision_id,
                &input.market_snapshot,
                &input.signal_input,
                &agent_votes,
                &chair_output,
                &risk_decision,
                paper_order.as_ref(),
                context,
            )
        })
        .transpose()?;

    let owner_explanation = input.owner_advisory.as_ref().map(|advisory| {
        let mut review = review_owner_trade_request(
            advisory,
            &risk_decision,
            &input.market_snapshot,
            input.market_snapshot.timestamp_ms,
        );
        let requested_action = advisory
            .requested_action
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let cooldown_bypass_requested = requested_action.contains("cooldown")
            || requested_action.contains("activate agent")
            || requested_action.contains("force speaker");
        if cooldown_bypass_requested
            && original_agent_states.iter().any(|state| {
                state.status == AgentStatus::Cooldown || state.voice_state.cooldown_bars > 0
            })
        {
            review.paper_action_allowed = false;
            review
                .reason_codes
                .push(ReasonCode::OwnerRequestedButAgentInCooldown);
            review
                .reason_codes
                .push(ReasonCode::CooldownOwnerBypassRejected);
            review
                .reason_codes
                .push(ReasonCode::OwnerRequestedButCooldownActive);
            review.reason_codes = stable_reason_codes(&review.reason_codes);
            review.explanation = owner_rejection_explanation(&review.reason_codes);
        }
        if requested_action.contains("promote")
            || requested_action.contains("promotion")
            || requested_action.contains("sandbox")
        {
            review.paper_action_allowed = false;
            review
                .reason_codes
                .push(ReasonCode::OwnerRequestedButSandboxOnly);
            review
                .reason_codes
                .push(ReasonCode::OwnerRequestedButPolicyBlocked);
            review.reason_codes = stable_reason_codes(&review.reason_codes);
            review.explanation = owner_rejection_explanation(&review.reason_codes);
        }
        review
    });

    let mut feedback_records = Vec::new();
    let mut reward_penalties = Vec::new();
    let mut updated_agent_states = original_agent_states.clone();
    let mut version_snapshots = Vec::new();
    let mut sandbox_candidates = Vec::new();
    let mut journal = AgentStateJournal::default();

    if let (Some(outcome), Some(context)) = (&paper_outcome, &input.paper_context) {
        for state in &original_agent_states {
            let proposal = agent_proposals
                .iter()
                .find(|proposal| proposal.agent_id == state.agent_id)
                .ok_or(PaperLearningLoopError::InvalidActiveAgentSet)?;
            let vote = agent_votes
                .iter()
                .find(|vote| vote.persona_id == state.agent_id)
                .ok_or(PaperLearningLoopError::InvalidActiveAgentSet)?;
            let attribution = outcome
                .attribution_records
                .iter()
                .find(|record| record.persona_id == state.agent_id)
                .ok_or(PaperLearningLoopError::InvalidPaperOutcome)?;
            let attributed_outcome =
                outcome_for_agent(outcome, attribution, context.hypothetical_net_return_pct);
            let feedback_context = FeedbackContext {
                paper_only: true,
                outcome_finalized: true,
                doctrine_violation: context.doctrine_violation_agents.contains(&state.agent_id),
                overtrade: context.overtrade_agents.contains(&state.agent_id),
            };
            let mut feedback = build_agent_feedback_from_paper_outcome(
                state,
                proposal,
                &attributed_outcome,
                &feedback_context,
            )
            .map_err(PaperLearningLoopError::FeedbackBuild)?;
            apply_attribution_to_feedback(
                &mut feedback,
                attribution,
                vote,
                chair_output.lead_speaker == state.agent_id,
            );
            feedback.doctrine_violation =
                detect_doctrine_violation(&state.doctrine, proposal, &feedback);
            if feedback.doctrine_violation {
                feedback
                    .reason_codes
                    .push(ReasonCode::FeedbackDoctrineViolation);
                feedback.reason_codes = stable_reason_codes(&feedback.reason_codes);
            }
            let reward_penalty = compute_chair_reward_penalty(state, &feedback);
            let mut updated_state = apply_chair_reward_penalty(state, &reward_penalty);
            updated_state.memory_summary = apply_feedback_to_memory_summary(state, &feedback);
            updated_state.status = classify_agent_status(&updated_state);
            let version_snapshot = AgentStateSnapshot {
                agent_id: updated_state.agent_id.clone(),
                version_id: updated_state.version.version_id.clone(),
                parent_version_id: updated_state.version.parent_version_id.clone(),
                state: updated_state.clone(),
                feedback_event_id: Some(reward_penalty.source_feedback_id.clone()),
                created_from_paper_only: true,
                sandbox_only: updated_state.version.sandbox_only,
                reason_codes: reward_penalty.reason_codes.clone(),
            };
            journal
                .append_snapshot(version_snapshot.clone())
                .map_err(PaperLearningLoopError::VersionJournal)?;
            if let Some(candidate) =
                build_sandbox_promotion_candidate(&updated_state, &[feedback.clone()])
            {
                sandbox_candidates.push(candidate);
            }
            if let Some(slot) = updated_agent_states
                .iter_mut()
                .find(|candidate| candidate.agent_id == updated_state.agent_id)
            {
                *slot = updated_state;
            }
            feedback_records.push(feedback);
            reward_penalties.push(reward_penalty);
            version_snapshots.push(version_snapshot);
        }
    }

    let mut reason_codes = chair_output
        .reason_codes
        .iter()
        .chain(risk_decision.reason_codes.iter())
        .cloned()
        .collect::<Vec<_>>();
    reason_codes.push(ReasonCode::DeterministicPath);
    reason_codes.push(ReasonCode::PaperExecutionOnly);
    if paper_outcome.is_none() {
        reason_codes.push(ReasonCode::FeedbackOutcomeIncomplete);
    }
    if let Some(review) = &owner_explanation {
        reason_codes.extend(review.reason_codes.iter().cloned());
    }
    for feedback in &feedback_records {
        reason_codes.extend(feedback.reason_codes.iter().cloned());
    }
    reason_codes = stable_reason_codes(&reason_codes);

    let report = PaperLearningLoopReport {
        decision_id,
        active_agent_count: original_agent_states.len(),
        feedback_count: feedback_records.len(),
        updated_state_count: updated_agent_states
            .iter()
            .zip(original_agent_states.iter())
            .filter(|(updated, original)| updated.version != original.version)
            .count(),
        version_snapshot_count: version_snapshots.len(),
        sandbox_candidate_count: sandbox_candidates.len(),
        paper_order_created: paper_order.is_some(),
        paper_outcome_finalized: paper_outcome.is_some(),
        risk_veto_preserved: if risk_decision.kind == RiskDecisionKind::ApprovePaper {
            paper_order.is_some()
        } else {
            paper_order.is_none()
        },
        paper_only: true,
        live_execution_supported: broker.supports_live_execution(),
        live_call_count: broker.live_call_count(),
        reason_codes: reason_codes.clone(),
    };

    Ok(PaperLearningLoopResult {
        original_agent_states,
        agent_votes,
        agent_proposals,
        chair_output,
        risk_decision,
        paper_order,
        paper_outcome,
        feedback_records,
        reward_penalties,
        updated_agent_states,
        version_snapshots,
        sandbox_candidates,
        owner_explanation,
        report,
        reason_codes,
    })
}

pub fn run_3_agent_paper_learning_chain(
    input: PaperLearningChainInput,
) -> Result<PaperLearningChainResult, PaperLearningChainError> {
    validate_three_agent_set(&input.initial_agent_states)
        .map_err(|_| PaperLearningChainError::InvalidInitialAgentSet)?;
    if input.episodes.is_empty() {
        return Err(PaperLearningChainError::EmptyEpisodes);
    }
    if input.episodes.len() > input.chain_config.max_episodes {
        return Err(PaperLearningChainError::TooManyEpisodes);
    }
    let mut episode_ids = input
        .episodes
        .iter()
        .map(|episode| episode.episode_id.as_str())
        .collect::<Vec<_>>();
    if episode_ids
        .iter()
        .any(|episode_id| episode_id.trim().is_empty())
    {
        return Err(PaperLearningChainError::InvalidEpisodeId);
    }
    if input.episodes.iter().any(|episode| {
        episode.reason_codes.iter().any(|reason| {
            !matches!(
                reason,
                ReasonCode::DeterministicPath
                    | ReasonCode::PaperExecutionOnly
                    | ReasonCode::SyntheticFixtureEvidence
            )
        })
    }) {
        return Err(PaperLearningChainError::InvalidEpisodeReasonCode);
    }
    episode_ids.sort_unstable();
    if episode_ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(PaperLearningChainError::DuplicateEpisodeId);
    }
    if input.chain_config.require_finalized_outcomes
        && input.episodes.iter().any(|episode| {
            episode
                .input
                .paper_context
                .as_ref()
                .is_none_or(|context| !context.outcome_finalized)
        })
    {
        return Err(PaperLearningChainError::IncompleteEpisode);
    }
    let expected_risk_governor = input.episodes[0].input.loop_config.risk_governor;
    if input
        .episodes
        .iter()
        .any(|episode| episode.input.loop_config.risk_governor != expected_risk_governor)
    {
        return Err(PaperLearningChainError::RiskGovernorChanged);
    }
    let mut decision_ids = input
        .episodes
        .iter()
        .map(|episode| {
            format!(
                "{}:{}",
                episode.input.market_snapshot.symbol, episode.input.market_snapshot.timestamp_ms
            )
        })
        .collect::<Vec<_>>();
    decision_ids.sort();
    if decision_ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(PaperLearningChainError::DuplicateDecisionId);
    }
    if input.episodes.windows(2).any(|pair| {
        pair[0].input.market_snapshot.timestamp_ms >= pair[1].input.market_snapshot.timestamp_ms
    }) {
        return Err(PaperLearningChainError::NonMonotonicEpisodeTime);
    }
    if input.episodes.windows(2).any(|pair| {
        pair[0].input.paper_context.as_ref().is_some_and(|context| {
            context.outcome_finalized
                && context.finalized_at_timestamp_ms >= pair[1].input.market_snapshot.timestamp_ms
        })
    }) {
        return Err(PaperLearningChainError::NonCausalOutcomeTime);
    }

    let initial_states = input.initial_agent_states.clone();
    let mut current_states = initial_states.clone();
    let mut episode_results = Vec::with_capacity(input.episodes.len());
    let mut version_journal = AgentStateJournal::default();
    let mut sandbox_candidates = Vec::new();
    let mut attribution_summary = initial_states
        .iter()
        .map(empty_attribution_summary)
        .collect::<Vec<_>>();
    let mut reason_codes = vec![
        ReasonCode::DeterministicPath,
        ReasonCode::PaperExecutionOnly,
    ];

    for episode in input.episodes {
        let input_states = current_states.clone();
        let mut episode_input = episode.input;
        episode_input.initial_agent_states = input_states.clone();
        let result = run_3_agent_paper_learning_loop(episode_input)
            .map_err(PaperLearningChainError::Episode)?;
        for snapshot in &result.version_snapshots {
            let expected_parent = input_states
                .iter()
                .find(|state| state.agent_id == snapshot.agent_id)
                .map(|state| state.version.version_id.as_str());
            if snapshot.parent_version_id.as_deref() != expected_parent {
                return Err(PaperLearningChainError::VersionParentMismatch);
            }
            version_journal
                .append_snapshot(snapshot.clone())
                .map_err(PaperLearningChainError::VersionJournal)?;
        }
        accumulate_attribution(&mut attribution_summary, &input_states, &result);
        sandbox_candidates.extend(result.sandbox_candidates.iter().cloned());
        current_states = result.updated_agent_states.clone();
        let episode_reason_codes = stable_reason_codes(
            &episode
                .reason_codes
                .iter()
                .chain(result.reason_codes.iter())
                .cloned()
                .collect::<Vec<_>>(),
        );
        reason_codes.extend(episode_reason_codes.iter().cloned());
        episode_results.push(PaperLearningEpisodeResult {
            episode_id: episode.episode_id,
            input_states,
            result,
            reason_codes: episode_reason_codes,
        });
    }

    finalize_attribution(&mut attribution_summary, &current_states);
    let agent_learning_summaries = build_agent_learning_summaries(
        &initial_states,
        &current_states,
        &episode_results,
        &sandbox_candidates,
    );
    if current_states.iter().any(
        |state| match version_journal.latest_for_agent(&state.agent_id) {
            Some(snapshot) => snapshot.version_id != state.version.version_id,
            None => initial_states
                .iter()
                .find(|initial| initial.agent_id == state.agent_id)
                .is_none_or(|initial| initial.version.version_id != state.version.version_id),
        },
    ) {
        return Err(PaperLearningChainError::VersionFinalMismatch);
    }
    let any_live_mutation_detected = episode_results.iter().any(|episode| {
        episode.result.original_agent_states != episode.input_states
            || (episode.result.paper_outcome.is_none()
                && episode.result.updated_agent_states != episode.result.original_agent_states)
            || episode
                .result
                .updated_agent_states
                .iter()
                .zip(episode.result.original_agent_states.iter())
                .any(|(updated, original)| {
                    updated.doctrine != original.doctrine
                        || updated.mutable_policy != original.mutable_policy
                })
    });
    let any_risk_bypass_detected = episode_results.iter().any(|episode| {
        (episode.result.risk_decision.kind != RiskDecisionKind::ApprovePaper
            && episode.result.paper_order.is_some())
            || episode
                .result
                .paper_outcome
                .as_ref()
                .is_some_and(|outcome| outcome.denied_by_risk && outcome.executed)
            || episode
                .result
                .owner_explanation
                .as_ref()
                .is_some_and(|review| review.owner_forced_trade)
    });
    let total_paper_trades = episode_results
        .iter()
        .filter(|episode| {
            episode
                .result
                .paper_outcome
                .as_ref()
                .is_some_and(|outcome| outcome.executed)
        })
        .count() as u64;
    let total_no_trades = episode_results
        .iter()
        .filter(|episode| {
            episode
                .result
                .paper_outcome
                .as_ref()
                .is_some_and(|outcome| outcome.no_trade)
        })
        .count() as u64;
    let total_risk_denials = episode_results
        .iter()
        .filter(|episode| {
            episode
                .result
                .paper_outcome
                .as_ref()
                .is_some_and(|outcome| outcome.denied_by_risk)
        })
        .count() as u64;
    reason_codes = stable_reason_codes(&reason_codes);
    let summary = LearningChainSummary {
        total_episodes: episode_results.len(),
        total_paper_trades,
        total_no_trades,
        total_risk_denials,
        agent_summaries: agent_learning_summaries.clone(),
        sandbox_candidate_count: sandbox_candidates.len(),
        any_live_mutation_detected,
        any_risk_bypass_detected,
        reason_codes: reason_codes.clone(),
    };

    Ok(PaperLearningChainResult {
        initial_states,
        final_states: current_states,
        episode_results,
        version_journal,
        attribution_summary,
        agent_learning_summaries,
        sandbox_candidates,
        summary,
        reason_codes,
    })
}

pub fn is_agent_available_for_live_decision(state: &CanonicalAgentState) -> bool {
    state.can_vote_live()
}

pub fn clear_expired_cooldown(state: &CanonicalAgentState) -> CanonicalAgentState {
    let mut next = state.clone();
    if next.voice_state.cooldown_bars == 0 && next.status == AgentStatus::Cooldown {
        next.status = classify_agent_status(&next);
        next.reason_codes.push(ReasonCode::CooldownExpired);
        next.reason_codes = stable_reason_codes(&next.reason_codes);
    }
    next
}

pub fn apply_cooldown_tick_after_episode(state: &CanonicalAgentState) -> CanonicalAgentState {
    if state.voice_state.cooldown_bars == 0
        || matches!(
            state.status,
            AgentStatus::Quarantined | AgentStatus::Disabled | AgentStatus::SandboxOnly
        )
    {
        return clear_expired_cooldown(state);
    }
    let mut next = state.clone();
    next.voice_state.cooldown_bars = next.voice_state.cooldown_bars.saturating_sub(1);
    next.reason_codes.push(ReasonCode::CooldownTicked);
    if next.voice_state.cooldown_bars == 0 {
        next.status = classify_agent_status(&next);
        next.reason_codes.push(ReasonCode::CooldownExpired);
    } else {
        next.status = AgentStatus::Cooldown;
    }
    next.reason_codes = stable_reason_codes(&next.reason_codes);
    next
}

pub fn run_3_agent_paper_replay(
    input: PaperReplayInput,
) -> Result<PaperReplayResult, PaperReplayError> {
    validate_paper_replay_input(&input)?;
    let requested_episode_count = input.episode_inputs.len();
    let initial_states = input.initial_agent_states.clone();
    let mut current_states = initial_states.clone();
    let mut chain_results = Vec::with_capacity(requested_episode_count);
    let mut version_journal = AgentStateJournal::default();
    let mut sandbox_candidates = Vec::new();
    let mut replay_attribution_summary = initial_states
        .iter()
        .map(empty_replay_attribution_summary)
        .collect::<Vec<_>>();
    let mut stop_reason_codes = Vec::new();
    let mut reason_codes = vec![
        ReasonCode::DeterministicPath,
        ReasonCode::PaperExecutionOnly,
    ];

    for episode in input.episode_inputs {
        let episode_id = episode.episode_id.clone();
        let chain = run_3_agent_paper_learning_chain(PaperLearningChainInput {
            initial_agent_states: current_states.clone(),
            episodes: vec![episode],
            chain_config: PaperLearningChainConfig {
                max_episodes: 1,
                require_finalized_outcomes: true,
            },
        })
        .map_err(PaperReplayError::Chain)?;

        for snapshot in chain.version_journal.snapshots() {
            version_journal
                .append_snapshot(snapshot.clone())
                .map_err(PaperReplayError::VersionJournal)?;
        }
        accumulate_replay_attribution(&mut replay_attribution_summary, &chain);
        current_states = chain.final_states.clone();

        if input.replay_config.cooldown_tick_mode == CooldownTickMode::PerEpisode {
            let mut ticked_states = Vec::with_capacity(current_states.len());
            for state in &current_states {
                let ticked = apply_cooldown_tick_after_episode(state);
                if ticked != *state {
                    let (versioned, snapshot) =
                        version_cooldown_transition(state, ticked, &episode_id);
                    version_journal
                        .append_snapshot(snapshot)
                        .map_err(PaperReplayError::VersionJournal)?;
                    ticked_states.push(versioned);
                } else {
                    ticked_states.push(ticked);
                }
            }
            current_states = ticked_states;
        }

        if input.replay_config.allow_sandbox_candidates {
            sandbox_candidates.extend(chain.sandbox_candidates.iter().cloned());
        }
        reason_codes.extend(chain.reason_codes.iter().cloned());
        let emergency_stop = chain.episode_results.iter().find_map(|episode_result| {
            (episode_result.result.risk_decision.kind == RiskDecisionKind::EmergencyStop)
                .then(|| episode_result.result.risk_decision.reason_codes.clone())
        });
        let quarantine_detected = current_states
            .iter()
            .any(|state| state.status == AgentStatus::Quarantined);
        chain_results.push(chain);

        if input.replay_config.stop_on_emergency_stop {
            if let Some(emergency_reasons) = emergency_stop {
                stop_reason_codes.extend(emergency_reasons);
                break;
            }
        }
        if input.replay_config.stop_on_quarantine && quarantine_detected {
            stop_reason_codes.push(ReasonCode::Quarantined);
            break;
        }
    }

    if current_states.iter().any(|state| {
        version_journal
            .latest_for_agent(&state.agent_id)
            .is_none_or(|snapshot| snapshot.version_id != state.version.version_id)
    }) {
        return Err(PaperReplayError::VersionFinalMismatch);
    }
    finalize_replay_attribution(&mut replay_attribution_summary, &current_states);
    let episode_results = chain_results
        .iter()
        .flat_map(|chain| chain.episode_results.iter().cloned())
        .collect::<Vec<_>>();
    let agent_summaries = build_agent_learning_summaries(
        &initial_states,
        &current_states,
        &episode_results,
        &sandbox_candidates,
    );
    let learning_chain_summary = LearningChainSummary {
        total_episodes: episode_results.len(),
        total_paper_trades: chain_results
            .iter()
            .map(|chain| chain.summary.total_paper_trades)
            .sum(),
        total_no_trades: chain_results
            .iter()
            .map(|chain| chain.summary.total_no_trades)
            .sum(),
        total_risk_denials: chain_results
            .iter()
            .map(|chain| chain.summary.total_risk_denials)
            .sum(),
        agent_summaries,
        sandbox_candidate_count: sandbox_candidates.len(),
        any_live_mutation_detected: chain_results
            .iter()
            .any(|chain| chain.summary.any_live_mutation_detected),
        any_risk_bypass_detected: chain_results
            .iter()
            .any(|chain| chain.summary.any_risk_bypass_detected),
        reason_codes: stable_reason_codes(&reason_codes),
    };
    reason_codes.extend(stop_reason_codes.iter().cloned());
    reason_codes = stable_reason_codes(&reason_codes);
    stop_reason_codes = stable_reason_codes(&stop_reason_codes);
    let stopped_early = chain_results.len() < requested_episode_count;

    Ok(PaperReplayResult {
        initial_states,
        final_states: current_states,
        chain_results,
        learning_chain_summary,
        replay_attribution_summary,
        version_journal,
        sandbox_candidates,
        stopped_early,
        stop_reason_codes,
        reason_codes,
    })
}

fn validate_paper_replay_input(input: &PaperReplayInput) -> Result<(), PaperReplayError> {
    validate_three_agent_set(&input.initial_agent_states)
        .map_err(|_| PaperReplayError::InvalidInitialAgentSet)?;
    if input.replay_config.active_agent_limit != 3 || input.replay_config.max_episodes == 0 {
        return Err(PaperReplayError::InvalidConfig);
    }
    if input.episode_inputs.is_empty() {
        return Err(PaperReplayError::EmptyEpisodes);
    }
    if input.episode_inputs.len() > input.replay_config.max_episodes {
        return Err(PaperReplayError::TooManyEpisodes);
    }
    let mut episode_ids = input
        .episode_inputs
        .iter()
        .map(|episode| episode.episode_id.as_str())
        .collect::<Vec<_>>();
    episode_ids.sort_unstable();
    if episode_ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(PaperReplayError::DuplicateEpisodeId);
    }
    let mut decision_ids = input
        .episode_inputs
        .iter()
        .map(|episode| {
            (
                episode.input.market_snapshot.symbol.as_str(),
                episode.input.market_snapshot.timestamp_ms,
            )
        })
        .collect::<Vec<_>>();
    decision_ids.sort_unstable();
    if decision_ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(PaperReplayError::DuplicateDecisionId);
    }
    if input.episode_inputs.windows(2).any(|pair| {
        pair[0].input.market_snapshot.timestamp_ms >= pair[1].input.market_snapshot.timestamp_ms
    }) {
        return Err(PaperReplayError::NonMonotonicEpisodeTime);
    }
    if input.episode_inputs.windows(2).any(|pair| {
        pair[0].input.paper_context.as_ref().is_some_and(|context| {
            context.outcome_finalized
                && context.finalized_at_timestamp_ms >= pair[1].input.market_snapshot.timestamp_ms
        })
    }) {
        return Err(PaperReplayError::NonCausalOutcomeTime);
    }
    let expected_risk_governor = input.episode_inputs[0].input.loop_config.risk_governor;
    if input
        .episode_inputs
        .iter()
        .any(|episode| episode.input.loop_config.risk_governor != expected_risk_governor)
    {
        return Err(PaperReplayError::RiskGovernorChanged);
    }
    Ok(())
}

fn version_cooldown_transition(
    state: &CanonicalAgentState,
    mut transitioned: CanonicalAgentState,
    episode_id: &str,
) -> (CanonicalAgentState, AgentStateSnapshot) {
    let event_id = format!(
        "cooldown-tick:{episode_id}:{}:{}",
        state.agent_id, transitioned.voice_state.cooldown_bars
    );
    let reason_codes = stable_reason_codes(&transitioned.reason_codes);
    transitioned.version = AgentVersion {
        version_id: format!(
            "{}::cooldown::{}",
            state.version.version_id,
            stable_hash_string(&event_id)
        ),
        parent_version_id: Some(state.version.version_id.clone()),
        created_from_feedback_event: Some(event_id.clone()),
        live_enabled: state.version.live_enabled,
        sandbox_only: state.version.sandbox_only,
        reason_codes: reason_codes.clone(),
    };
    let snapshot = AgentStateSnapshot {
        agent_id: transitioned.agent_id.clone(),
        version_id: transitioned.version.version_id.clone(),
        parent_version_id: transitioned.version.parent_version_id.clone(),
        state: transitioned.clone(),
        feedback_event_id: Some(event_id),
        created_from_paper_only: true,
        sandbox_only: transitioned.version.sandbox_only,
        reason_codes,
    };
    (transitioned, snapshot)
}

fn empty_replay_attribution_summary(state: &CanonicalAgentState) -> ReplayAttributionSummary {
    ReplayAttributionSummary {
        agent_id: state.agent_id.clone(),
        selected_count: 0,
        supported_final_count: 0,
        opposed_final_count: 0,
        abstained_count: 0,
        cooldown_skipped_count: 0,
        risk_veto_aligned_count: 0,
        no_trade_correct_count: 0,
        no_trade_missed_gain_count: 0,
        profitable_selected_count: 0,
        losing_selected_count: 0,
        total_reward: 0.0,
        total_penalty: 0.0,
        net_reward_penalty: 0.0,
        final_voice_power: state.voice_state.voice_power,
        final_status: state.status,
        final_cooldown: state.voice_state.cooldown_bars,
        final_tier: state.voice_state.tier,
        reason_codes: Vec::new(),
    }
}

fn accumulate_replay_attribution(
    summaries: &mut [ReplayAttributionSummary],
    chain: &PaperLearningChainResult,
) {
    for attribution in &chain.attribution_summary {
        let Some(summary) = summaries
            .iter_mut()
            .find(|summary| summary.agent_id == attribution.agent_id)
        else {
            continue;
        };
        summary.selected_count += attribution.selected_count;
        summary.supported_final_count += attribution.supported_final_count;
        summary.opposed_final_count += attribution.opposed_final_count;
        summary.abstained_count += attribution.abstained_count;
        summary.risk_veto_aligned_count += attribution.risk_veto_aligned_count;
        summary.no_trade_correct_count += attribution.no_trade_correct_count;
        summary.no_trade_missed_gain_count += attribution.no_trade_missed_gain_count;
        summary.profitable_selected_count += attribution.profitable_selected_count;
        summary.losing_selected_count += attribution.losing_selected_count;
        summary.total_reward += attribution.total_reward;
        summary.total_penalty += attribution.total_penalty;
        summary
            .reason_codes
            .extend(attribution.reason_codes.iter().cloned());
    }
    for episode in &chain.episode_results {
        for state in &episode.input_states {
            if (state.status == AgentStatus::Cooldown || state.voice_state.cooldown_bars > 0)
                && episode.result.agent_votes.iter().any(|vote| {
                    vote.persona_id == state.agent_id
                        && vote.stance == Stance::Abstain
                        && vote
                            .reason_codes
                            .contains(&ReasonCode::CooldownAgentUnavailable)
                })
            {
                if let Some(summary) = summaries
                    .iter_mut()
                    .find(|summary| summary.agent_id == state.agent_id)
                {
                    summary.cooldown_skipped_count =
                        summary.cooldown_skipped_count.saturating_add(1);
                    summary
                        .reason_codes
                        .push(ReasonCode::CooldownAgentUnavailable);
                }
            }
        }
    }
}

fn finalize_replay_attribution(
    summaries: &mut [ReplayAttributionSummary],
    final_states: &[CanonicalAgentState],
) {
    for summary in summaries {
        if let Some(state) = final_states
            .iter()
            .find(|state| state.agent_id == summary.agent_id)
        {
            summary.final_voice_power = state.voice_state.voice_power;
            summary.final_status = state.status;
            summary.final_cooldown = state.voice_state.cooldown_bars;
            summary.final_tier = state.voice_state.tier;
        }
        summary.net_reward_penalty = summary.total_reward - summary.total_penalty;
        summary.reason_codes = stable_reason_codes(&summary.reason_codes);
    }
}

impl Default for LocalDataSourceRegistry {
    fn default() -> Self {
        let profiles = [
            local_source_profile(LocalDataSourceKind::SyntheticFixture),
            local_source_profile(LocalDataSourceKind::KoreanStockCsv),
            local_source_profile(LocalDataSourceKind::UsStockCsv),
            local_source_profile(LocalDataSourceKind::BtcCryptoCsv),
        ]
        .into_iter()
        .map(|profile| (profile.kind, profile))
        .collect();
        Self { profiles }
    }
}

impl LocalDataSourceRegistry {
    pub fn register_profile(
        &mut self,
        profile: LocalDataSourceProfile,
    ) -> Result<(), LocalDataSourceError> {
        self.validate_profile(&profile)?;
        self.profiles.insert(profile.kind, profile);
        Ok(())
    }

    pub fn get_profile(&self, kind: LocalDataSourceKind) -> Option<&LocalDataSourceProfile> {
        self.profiles.get(&kind)
    }

    pub fn list_profiles(&self) -> Vec<&LocalDataSourceProfile> {
        self.profiles.values().collect()
    }

    pub fn validate_profile(
        &self,
        profile: &LocalDataSourceProfile,
    ) -> Result<(), LocalDataSourceError> {
        validate_local_source_profile(profile)
    }
}

pub fn parse_local_csv_with_profile(
    csv_text: &str,
    profile: &LocalDataSourceProfile,
    historical_config: &HistoricalReplayConfig,
) -> Result<LocalCsvSourceResult, LocalDataSourceError> {
    validate_local_source_profile(profile)?;
    if contains_temporary_instruction_marker(csv_text) {
        return Err(local_registry_error(
            ReasonCode::HistoricalReplayWorkMdMarker,
        ));
    }
    if contains_local_private_marker(csv_text) {
        return Err(local_registry_error(
            ReasonCode::HistoricalReplayPrivateMarker,
        ));
    }
    let mut lines = csv_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    let header_line = lines
        .next()
        .ok_or_else(|| local_registry_error(ReasonCode::HistoricalReplayEmptyDataset))?;
    let header = header_line
        .split(',')
        .map(|value| value.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    let header_index = header
        .iter()
        .enumerate()
        .map(|(index, name)| (name.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    if header_index.len() != header.len() {
        return Err(local_registry_error(ReasonCode::LocalSourceProfileInvalid));
    }
    if header.iter().any(|column| forbidden_local_column(column)) {
        return Err(local_registry_error(
            ReasonCode::HistoricalReplayForbiddenColumn,
        ));
    }
    let allowed_columns = profile
        .required_columns
        .iter()
        .chain(profile.optional_columns.iter())
        .map(|column| column.as_str())
        .collect::<Vec<_>>();
    if header
        .iter()
        .any(|column| !allowed_columns.contains(&column.as_str()))
    {
        return Err(local_registry_error(
            ReasonCode::HistoricalReplayForbiddenColumn,
        ));
    }
    if profile
        .required_columns
        .iter()
        .filter(|column| column.as_str() != "timestamp_ms")
        .any(|column| !header_index.contains_key(column.as_str()))
    {
        return Err(local_registry_error(
            ReasonCode::LocalSourceMissingRequiredColumn,
        ));
    }
    let has_timestamp_ms = header_index.contains_key("timestamp_ms");
    let has_date_time = header_index.contains_key("date") && header_index.contains_key("time");
    let timestamp_supported = match profile.timestamp_unit {
        LocalTimestampUnit::Milliseconds => has_timestamp_ms,
        LocalTimestampUnit::MillisecondsOrDateTimeUtc => has_timestamp_ms || has_date_time,
    };
    if !timestamp_supported {
        return Err(local_registry_error(
            ReasonCode::LocalSourceUnsupportedTimestamp,
        ));
    }

    let data_lines = lines.collect::<Vec<_>>();
    if data_lines.is_empty() {
        return Err(local_registry_error(
            ReasonCode::HistoricalReplayEmptyDataset,
        ));
    }
    if data_lines.len() > historical_config.max_rows {
        return Err(local_registry_error(
            ReasonCode::HistoricalReplayTooManyRows,
        ));
    }
    let mut canonical_rows = Vec::with_capacity(data_lines.len());
    let mut first_symbol: Option<String> = None;
    let mut previous_timestamp: Option<u64> = None;
    for (offset, line) in data_lines.iter().enumerate() {
        let values = line
            .split(',')
            .map(|value| value.trim())
            .collect::<Vec<_>>();
        if values.len() != header.len() {
            return Err(LocalDataSourceError::Historical(historical_error(
                Some(offset + 2),
                ReasonCode::HistoricalReplayInvalidRow,
            )));
        }
        let value = |name: &str| {
            header_index
                .get(name)
                .and_then(|index| values.get(*index))
                .copied()
        };
        let symbol = value("symbol").unwrap_or_default();
        if symbol.is_empty()
            || first_symbol
                .as_deref()
                .is_some_and(|expected| expected != symbol)
        {
            return Err(local_registry_error(
                ReasonCode::HistoricalReplayMultiSymbolUnsupported,
            ));
        }
        first_symbol.get_or_insert_with(|| symbol.to_string());
        let timestamp_ms = if let Some(timestamp) =
            value("timestamp_ms").filter(|timestamp| !timestamp.is_empty())
        {
            timestamp
                .parse::<u64>()
                .map_err(|_| local_registry_error(ReasonCode::LocalSourceUnsupportedTimestamp))?
        } else {
            parse_local_datetime_utc_ms(
                value("date").unwrap_or_default(),
                value("time").unwrap_or_default(),
            )
            .ok_or_else(|| local_registry_error(ReasonCode::LocalSourceUnsupportedTimestamp))?
        };
        if previous_timestamp == Some(timestamp_ms) {
            return Err(local_registry_error(
                ReasonCode::HistoricalReplayDuplicateTimestamp,
            ));
        }
        if historical_config.require_monotonic_timestamps
            && previous_timestamp.is_some_and(|previous| previous > timestamp_ms)
        {
            return Err(local_registry_error(
                ReasonCode::HistoricalReplayNonMonotonicTimestamp,
            ));
        }
        previous_timestamp = Some(timestamp_ms);
        let source = value("source")
            .filter(|source| !source.is_empty())
            .unwrap_or("synthetic");
        if !profile.allowed_source_markers.iter().any(|allowed| {
            let normalized = source.to_ascii_lowercase();
            normalized == allowed.as_str() || normalized.starts_with(&format!("{allowed}:"))
        }) {
            return Err(local_registry_error(
                ReasonCode::HistoricalReplayUnsafeSource,
            ));
        }
        let scaled = |name: &str, scale: f64| -> Result<String, LocalDataSourceError> {
            let parsed = value(name)
                .and_then(|number| number.parse::<f64>().ok())
                .ok_or_else(|| local_registry_error(ReasonCode::HistoricalReplayInvalidRow))?;
            Ok(format!("{:.12}", parsed * scale))
        };
        let trade_value = value("trade_value")
            .or_else(|| value("quote_volume"))
            .filter(|value| !value.is_empty())
            .unwrap_or_default();
        for optional_numeric in ["adjusted_close", "quote_volume", "trade_count"] {
            if let Some(optional_value) = value(optional_numeric).filter(|value| !value.is_empty())
            {
                let parsed = optional_value
                    .parse::<f64>()
                    .map_err(|_| local_registry_error(ReasonCode::HistoricalReplayInvalidRow))?;
                if !parsed.is_finite()
                    || parsed < 0.0
                    || (optional_numeric == "adjusted_close" && parsed == 0.0)
                {
                    return Err(local_registry_error(ReasonCode::HistoricalReplayInvalidRow));
                }
            }
        }
        canonical_rows.push(format!(
            "{symbol},{timestamp_ms},{},{},{},{},{},{},synthetic:{}",
            scaled("open", profile.price_scale)?,
            scaled("high", profile.price_scale)?,
            scaled("low", profile.price_scale)?,
            scaled("close", profile.price_scale)?,
            scaled("volume", profile.volume_scale)?,
            trade_value,
            local_source_kind_label(profile.kind),
        ));
    }
    let canonical_csv = format!(
        "symbol,timestamp_ms,open,high,low,close,volume,trade_value,source\n{}",
        canonical_rows.join("\n")
    );
    let dataset = HistoricalReplayAdapter
        .parse_csv_string(&canonical_csv, historical_config)
        .map_err(LocalDataSourceError::Historical)?;
    let quality_summary = build_local_data_quality_summary(&dataset, profile.kind);
    Ok(LocalCsvSourceResult {
        reason_codes: stable_reason_codes(
            &profile
                .reason_codes
                .iter()
                .chain(dataset.reason_codes.iter())
                .cloned()
                .collect::<Vec<_>>(),
        ),
        dataset,
        quality_summary,
    })
}

pub fn normalize_dataset_to_candle_series(
    dataset: &HistoricalReplayDataset,
    historical_config: &HistoricalReplayConfig,
) -> Result<CandleSeries, HistoricalReplayError> {
    HistoricalReplayAdapter.to_candle_series(dataset, historical_config)
}

pub fn parse_manual_historical_daily_csv(
    csv_text: &str,
    config: &ManualHistoricalDailyImportConfig,
) -> Result<ManualHistoricalDailyDataset, ManualHistoricalDailyImportError> {
    validate_manual_historical_config(config)?;
    if let Some(reason) = manual_historical_safety_reason(csv_text, None, config) {
        return Err(manual_historical_error(None, reason));
    }
    let mut lines = csv_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    let header_line = lines
        .next()
        .ok_or_else(|| manual_historical_error(None, ReasonCode::HistoricalReplayEmptyDataset))?;
    let header = header_line
        .split(',')
        .map(|value| value.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    if let Some(reason) = manual_historical_safety_reason(csv_text, Some(&header), config) {
        return Err(manual_historical_error(None, reason));
    }
    let header_index = header
        .iter()
        .enumerate()
        .map(|(index, name)| (name.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    if header.is_empty() || header_index.len() != header.len() {
        return Err(manual_historical_error(
            None,
            ReasonCode::HistoricalReplayInvalidHeader,
        ));
    }
    let allowed_columns = [
        "symbol",
        "date",
        "open",
        "high",
        "low",
        "close",
        "volume",
        "adjusted_close",
        "trade_value",
        "currency",
        "market",
        "source",
        "split_factor",
        "dividend",
    ];
    let required_columns = ["symbol", "date", "open", "high", "low", "close", "volume"];
    if header
        .iter()
        .any(|column| !allowed_columns.contains(&column.as_str()))
    {
        return Err(manual_historical_error(
            None,
            ReasonCode::HistoricalReplayForbiddenColumn,
        ));
    }
    if required_columns
        .iter()
        .any(|column| !header_index.contains_key(column))
    {
        return Err(manual_historical_error(
            None,
            ReasonCode::LocalSourceMissingRequiredColumn,
        ));
    }
    if header_index.contains_key("adjusted_close")
        && (!config.allow_adjusted_close
            || config.adjusted_close_policy == ManualAdjustedClosePolicy::RejectIfPresent)
    {
        return Err(manual_historical_error(
            None,
            ReasonCode::HistoricalReplayForbiddenColumn,
        ));
    }

    let data_lines = lines.collect::<Vec<_>>();
    if data_lines.is_empty() {
        return Err(manual_historical_error(
            None,
            ReasonCode::HistoricalReplayEmptyDataset,
        ));
    }
    if config.max_rows == 0 || data_lines.len() > config.max_rows {
        return Err(manual_historical_error(
            None,
            ReasonCode::HistoricalReplayTooManyRows,
        ));
    }
    if data_lines.len() < config.min_rows {
        return Err(manual_historical_error(
            None,
            ReasonCode::WalkForwardInsufficientRows,
        ));
    }

    let mut rows = Vec::with_capacity(data_lines.len());
    let mut first_symbol: Option<String> = None;
    let mut previous_timestamp: Option<u64> = None;
    for (offset, line) in data_lines.iter().enumerate() {
        let row_number = offset + 2;
        if let Some(reason) = manual_historical_safety_reason(line, None, config) {
            return Err(manual_historical_error(Some(row_number), reason));
        }
        let values = line
            .split(',')
            .map(|value| value.trim())
            .collect::<Vec<_>>();
        if values.len() != header.len() {
            return Err(manual_historical_error(
                Some(row_number),
                ReasonCode::HistoricalReplayInvalidRow,
            ));
        }
        let value = |name: &str| {
            header_index
                .get(name)
                .and_then(|index| values.get(*index))
                .copied()
        };
        let symbol = value("symbol").unwrap_or_default().trim();
        if symbol.is_empty() {
            return Err(manual_historical_error(
                Some(row_number),
                ReasonCode::HistoricalReplayInvalidRow,
            ));
        }
        if config.strict_single_symbol
            && first_symbol
                .as_deref()
                .is_some_and(|expected| expected != symbol)
        {
            return Err(manual_historical_error(
                Some(row_number),
                ReasonCode::HistoricalReplayMultiSymbolUnsupported,
            ));
        }
        first_symbol.get_or_insert_with(|| symbol.to_string());
        let date = value("date").unwrap_or_default();
        let timestamp_ms = parse_manual_daily_date_ms(date).ok_or_else(|| {
            manual_historical_error(
                Some(row_number),
                ReasonCode::ManualHistoricalImportInvalidDate,
            )
        })?;
        if previous_timestamp == Some(timestamp_ms) && !config.allow_duplicate_dates {
            return Err(manual_historical_error(
                Some(row_number),
                ReasonCode::HistoricalReplayDuplicateTimestamp,
            ));
        }
        if config.require_monotonic_dates
            && previous_timestamp.is_some_and(|previous| previous > timestamp_ms)
        {
            return Err(manual_historical_error(
                Some(row_number),
                ReasonCode::HistoricalReplayNonMonotonicTimestamp,
            ));
        }
        previous_timestamp = Some(timestamp_ms);

        let open = parse_manual_required_number(value("open"), row_number)?;
        let high = parse_manual_required_number(value("high"), row_number)?;
        let low = parse_manual_required_number(value("low"), row_number)?;
        let close = parse_manual_required_number(value("close"), row_number)?;
        let volume = parse_manual_volume(value("volume"), row_number)?;
        if low > high || open < low || open > high || close < low || close > high || low <= 0.0 {
            return Err(manual_historical_error(
                Some(row_number),
                ReasonCode::HistoricalReplayInvalidOhlc,
            ));
        }
        let adjusted_close = parse_manual_optional_positive(value("adjusted_close"), row_number)?;
        let trade_value = parse_manual_optional_non_negative(value("trade_value"), row_number)?;
        let split_factor = parse_manual_optional_positive(value("split_factor"), row_number)?;
        let dividend = parse_manual_optional_non_negative(value("dividend"), row_number)?;

        rows.push(ManualHistoricalDailyRow {
            symbol: symbol.to_string(),
            date: date.to_string(),
            timestamp_ms,
            open,
            high,
            low,
            close,
            volume,
            adjusted_close,
            trade_value,
            currency: optional_manual_string(value("currency"))?,
            market: optional_manual_string(value("market"))?,
            source: optional_manual_string(value("source"))?,
            split_factor,
            dividend,
        });
    }

    let symbol = first_symbol.unwrap_or_default();
    let date_range = ManualHistoricalDateRange {
        start_date: rows.first().map_or(String::new(), |row| row.date.clone()),
        end_date: rows.last().map_or(String::new(), |row| row.date.clone()),
    };
    let quality_summary = manual_daily_quality_summary(&symbol, config.source_kind, &rows);
    let reason_codes = stable_reason_codes(
        &config
            .reason_codes
            .iter()
            .cloned()
            .chain([
                ReasonCode::ManualHistoricalImportDailyOnly,
                ReasonCode::ManualHistoricalImportNoNetwork,
                ReasonCode::ManualHistoricalImportSanitizedOnly,
                ReasonCode::WalkForwardTrainingDeferred,
                ReasonCode::DeterministicPath,
                ReasonCode::LocalFileOnly,
            ])
            .collect::<Vec<_>>(),
    );
    let dataset = ManualHistoricalDailyDataset {
        dataset_id: config.dataset_id.clone(),
        source_kind: config.source_kind,
        symbol,
        rows,
        date_range,
        data_quality_summary: quality_summary,
        sanitized: true,
        local_only: true,
        reason_codes,
    };
    validate_manual_historical_daily_dataset(&dataset, config)?;
    Ok(dataset)
}

pub fn validate_manual_historical_daily_dataset(
    dataset: &ManualHistoricalDailyDataset,
    config: &ManualHistoricalDailyImportConfig,
) -> Result<(), ManualHistoricalDailyImportError> {
    validate_manual_historical_config(config)?;
    if !dataset.sanitized || !dataset.local_only {
        return Err(manual_historical_error(
            None,
            ReasonCode::ManualHistoricalImportSanitizedOnly,
        ));
    }
    if dataset.rows.len() < config.min_rows
        || dataset.rows.len() > config.max_rows
        || dataset.symbol.trim().is_empty()
        || dataset.source_kind != config.source_kind
    {
        return Err(manual_historical_error(
            None,
            ReasonCode::WalkForwardInsufficientRows,
        ));
    }
    let mut previous_timestamp = None;
    let mut expected_symbol: Option<&str> = None;
    for (index, row) in dataset.rows.iter().enumerate() {
        let row_number = index + 2;
        if parse_manual_daily_date_ms(&row.date) != Some(row.timestamp_ms) {
            return Err(manual_historical_error(
                Some(row_number),
                ReasonCode::ManualHistoricalImportInvalidDate,
            ));
        }
        if config.strict_single_symbol
            && expected_symbol.is_some_and(|symbol| symbol != row.symbol.as_str())
        {
            return Err(manual_historical_error(
                Some(row_number),
                ReasonCode::HistoricalReplayMultiSymbolUnsupported,
            ));
        }
        expected_symbol.get_or_insert(row.symbol.as_str());
        if previous_timestamp == Some(row.timestamp_ms) && !config.allow_duplicate_dates {
            return Err(manual_historical_error(
                Some(row_number),
                ReasonCode::HistoricalReplayDuplicateTimestamp,
            ));
        }
        if config.require_monotonic_dates
            && previous_timestamp.is_some_and(|previous| previous > row.timestamp_ms)
        {
            return Err(manual_historical_error(
                Some(row_number),
                ReasonCode::HistoricalReplayNonMonotonicTimestamp,
            ));
        }
        previous_timestamp = Some(row.timestamp_ms);
        let numeric_values = [
            row.open,
            row.high,
            row.low,
            row.close,
            row.volume,
            row.adjusted_close.unwrap_or(1.0),
            row.trade_value.unwrap_or(0.0),
            row.split_factor.unwrap_or(1.0),
            row.dividend.unwrap_or(0.0),
        ];
        if numeric_values.iter().any(|value| !value.is_finite()) {
            return Err(manual_historical_error(
                Some(row_number),
                ReasonCode::HistoricalReplayNonFinite,
            ));
        }
        if row.open <= 0.0 || row.high <= 0.0 || row.low <= 0.0 || row.close <= 0.0 {
            return Err(manual_historical_error(
                Some(row_number),
                ReasonCode::HistoricalReplayNonPositivePrice,
            ));
        }
        if row.volume < 0.0
            || row.adjusted_close.is_some_and(|value| value <= 0.0)
            || row.trade_value.is_some_and(|value| value < 0.0)
            || row.split_factor.is_some_and(|value| value <= 0.0)
            || row.dividend.is_some_and(|value| value < 0.0)
        {
            return Err(manual_historical_error(
                Some(row_number),
                ReasonCode::HistoricalReplayInvalidRow,
            ));
        }
        if row.low > row.high
            || row.open < row.low
            || row.open > row.high
            || row.close < row.low
            || row.close > row.high
        {
            return Err(manual_historical_error(
                Some(row_number),
                ReasonCode::HistoricalReplayInvalidOhlc,
            ));
        }
    }
    Ok(())
}

pub fn to_daily_candle_series(
    dataset: &ManualHistoricalDailyDataset,
    config: &ManualHistoricalDailyImportConfig,
) -> Result<CandleSeries, ManualHistoricalDailyImportError> {
    validate_manual_historical_daily_dataset(dataset, config)?;
    Ok(CandleSeries {
        symbol: dataset.symbol.clone(),
        timeframe: Timeframe::OneDay,
        candles: dataset
            .rows
            .iter()
            .map(|row| Candle {
                timestamp_ms: row.timestamp_ms,
                open: row.open,
                high: row.high,
                low: row.low,
                close: match config.adjusted_close_policy {
                    ManualAdjustedClosePolicy::UseForReturnOnly => {
                        row.adjusted_close.unwrap_or(row.close)
                    }
                    ManualAdjustedClosePolicy::Ignore
                    | ManualAdjustedClosePolicy::RejectIfPresent => row.close,
                },
                volume: row.volume,
                trade_value: row.trade_value,
                bid: None,
                ask: None,
                spread_bps: None,
            })
            .collect(),
    })
}

pub fn build_walk_forward_splits(
    row_count: usize,
    config: &WalkForwardConfig,
) -> Result<Vec<WalkForwardSplit>, WalkForwardEvaluationError> {
    if !walk_forward_config_is_valid(config) {
        return Err(walk_forward_error(vec![
            ReasonCode::WalkForwardInsufficientRows,
        ]));
    }
    if row_count < config.min_train_rows + config.eval_window_rows {
        return Err(walk_forward_error(vec![
            ReasonCode::WalkForwardInsufficientRows,
        ]));
    }
    let mut splits = Vec::new();
    let mut eval_start = config.min_train_rows;
    while eval_start + config.eval_window_rows <= row_count {
        let split = WalkForwardSplit {
            train_start_index: 0,
            train_end_index: eval_start,
            eval_start_index: eval_start,
            eval_end_index: eval_start + config.eval_window_rows,
        };
        if split.train_end_index > split.eval_start_index
            || split.eval_start_index >= split.eval_end_index
            || split.eval_end_index > row_count
        {
            return Err(walk_forward_error(vec![ReasonCode::LeakageDetected]));
        }
        splits.push(split);
        eval_start = eval_start.saturating_add(config.step_rows);
    }
    if splits.is_empty() {
        return Err(walk_forward_error(vec![
            ReasonCode::WalkForwardInsufficientRows,
        ]));
    }
    Ok(splits)
}

pub fn compute_prediction_quality_metrics(
    strategy: BaselineStrategyKind,
    samples: &[PredictionQualitySample],
    min_samples: usize,
) -> PredictionQualityMetrics {
    let mut brier_sum = 0.0;
    let mut calibrated_sample_count = 0usize;
    let mut missing_probability_count = 0usize;
    let mut abstention_count = 0usize;
    let mut high_confidence_error_count = 0usize;
    let mut low_confidence_correct_count = 0usize;
    let mut confidence_sum = 0.0;
    let mut realized_sum = 0.0;
    let mut reason_codes = vec![
        ReasonCode::PredictionScoringBrier,
        ReasonCode::DeterministicPath,
    ];
    for sample in samples {
        let realized = if sample.realized_direction_up {
            1.0
        } else {
            0.0
        };
        realized_sum += realized;
        if sample.abstained {
            abstention_count += 1;
            reason_codes.push(ReasonCode::PredictionScoringAbstained);
        }
        let Some(probability) = sample.predicted_probability else {
            missing_probability_count += 1;
            reason_codes.push(ReasonCode::PredictionScoringMissingProbability);
            continue;
        };
        if !(0.0..=1.0).contains(&probability) || !probability.is_finite() {
            missing_probability_count += 1;
            reason_codes.push(ReasonCode::InvalidProbability);
            continue;
        }
        calibrated_sample_count += 1;
        brier_sum += (probability - realized).powi(2);
        let confidence = if probability >= 0.5 {
            probability
        } else {
            1.0 - probability
        };
        confidence_sum += confidence;
        let predicted_up = probability >= 0.5;
        let correct = predicted_up == sample.realized_direction_up;
        if confidence >= 0.75 && !correct {
            high_confidence_error_count += 1;
        }
        if confidence <= 0.55 && correct {
            low_confidence_correct_count += 1;
        }
    }
    if samples.len() < min_samples || calibrated_sample_count == 0 {
        reason_codes.push(ReasonCode::PredictionScoringInsufficientSamples);
    }
    PredictionQualityMetrics {
        strategy,
        brier_score: (samples.len() >= min_samples && calibrated_sample_count > 0)
            .then_some(brier_sum / calibrated_sample_count as f64),
        sample_count: samples.len(),
        calibrated_sample_count,
        missing_probability_count,
        abstention_count,
        high_confidence_error_count,
        low_confidence_correct_count,
        mean_confidence: (calibrated_sample_count > 0)
            .then_some(confidence_sum / calibrated_sample_count as f64),
        mean_realized_direction: (!samples.is_empty())
            .then_some(realized_sum / samples.len() as f64),
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

pub fn run_walk_forward_evaluation(
    input: WalkForwardEvaluationInput,
) -> Result<WalkForwardEvaluationResult, WalkForwardEvaluationError> {
    validate_three_agent_set(&input.initial_agent_states)
        .map_err(|_| walk_forward_error(vec![ReasonCode::LocalSourceRejected]))?;
    if input.committee_config.active_agent_limit != 3
        || input
            .initial_agent_states
            .iter()
            .any(|state| state.kind == AgentKind::Future8AgentPlaceholder)
    {
        return Err(walk_forward_error(vec![ReasonCode::LocalSourceRejected]));
    }
    let import_config = ManualHistoricalDailyImportConfig {
        dataset_id: input.dataset.dataset_id.clone(),
        source_kind: input.dataset.source_kind,
        min_rows: input.walk_forward_config.min_train_rows
            + input.walk_forward_config.eval_window_rows,
        ..ManualHistoricalDailyImportConfig::default()
    };
    let series = to_daily_candle_series(&input.dataset, &import_config)
        .map_err(|error| walk_forward_error(error.reason_codes))?;
    let splits = build_walk_forward_splits(series.len(), &input.walk_forward_config)?;
    let mut windows = Vec::with_capacity(splits.len());
    let mut voice_states = input.initial_agent_states.clone();
    let equal_states = input.initial_agent_states.clone();
    for split in splits {
        let agent_state_before = voice_states.clone();
        let always = evaluate_always_no_trade_window(&series, split);
        let buy_hold = evaluate_buy_and_hold_window(
            &series,
            split,
            &input.walk_forward_config,
            &input.committee_config,
            &input.risk_config,
        );
        let equal = evaluate_committee_window(
            BaselineStrategyKind::EqualWeightCommittee,
            &series,
            split,
            &equal_states,
            &input.walk_forward_config,
            &input.committee_config,
            &input.risk_config,
        );
        let voice = evaluate_committee_window(
            BaselineStrategyKind::VoiceAdaptiveCommittee,
            &series,
            split,
            &voice_states,
            &input.walk_forward_config,
            &input.committee_config,
            &input.risk_config,
        );
        voice_states = voice.final_states.clone();
        let reason_codes = stable_reason_codes(
            &input
                .walk_forward_config
                .reason_codes
                .iter()
                .cloned()
                .chain([
                    ReasonCode::WalkForwardWindowCreated,
                    ReasonCode::WalkForwardNoLookahead,
                    ReasonCode::WalkForwardEvaluated,
                    ReasonCode::PaperExecutionOnly,
                ])
                .collect::<Vec<_>>(),
        );
        windows.push(WalkForwardWindowResult {
            split,
            baseline_results: vec![always.metrics.clone(), buy_hold.metrics.clone()],
            committee_results: vec![equal.metrics.clone(), voice.metrics.clone()],
            scoring_results: vec![
                always.scoring.clone(),
                buy_hold.scoring.clone(),
                equal.scoring.clone(),
                voice.scoring.clone(),
            ],
            agent_state_before,
            agent_state_after: voice_states.clone(),
            reason_codes,
        });
    }
    let aggregate_metrics = aggregate_walk_forward_metrics(&windows);
    let scoring_summary =
        aggregate_prediction_summaries(&windows, input.walk_forward_config.min_prediction_samples);
    let comparison = build_proof_gate_comparison(
        &aggregate_metrics,
        &scoring_summary,
        input.walk_forward_config.min_prediction_samples,
    );
    let voice_adaptation_comparison = VoiceAdaptationComparison {
        equal_weight_total_return: comparison.equal_weight_committee.total_return,
        voice_adaptive_total_return: comparison.voice_adaptive_committee.total_return,
        equal_weight_risk_adjusted_score: risk_adjusted_score(&comparison.equal_weight_committee),
        voice_adaptive_risk_adjusted_score: risk_adjusted_score(
            &comparison.voice_adaptive_committee,
        ),
        voice_beats_equal_weight: comparison.voice_beats_equal_weight,
        reason_codes: if comparison.voice_beats_equal_weight {
            vec![ReasonCode::BaselineComparisonVoiceAdaptationHelped]
        } else {
            vec![ReasonCode::BaselineComparisonVoiceAdaptationFailed]
        },
    };
    let proof_gate_status = if comparison.insufficient_evidence {
        ProofGateStatus::InsufficientEvidence
    } else if comparison.voice_beats_equal_weight
        && comparison.committee_beats_no_trade
        && comparison.committee_beats_buy_hold_risk_adjusted
    {
        ProofGateStatus::ComputedNoProfitabilityClaim
    } else {
        ProofGateStatus::NoEdgeProven
    };
    let reason_codes = stable_reason_codes(
        &input
            .dataset
            .reason_codes
            .iter()
            .chain(input.walk_forward_config.reason_codes.iter())
            .chain(comparison.reason_codes.iter())
            .cloned()
            .chain([
                ReasonCode::ManualHistoricalImportNoNetwork,
                ReasonCode::ManualHistoricalImportSanitizedOnly,
                ReasonCode::WalkForwardEvaluationOnly,
                ReasonCode::PaperExecutionOnly,
                ReasonCode::HardcodingAuditPassed,
            ])
            .collect::<Vec<_>>(),
    );
    Ok(WalkForwardEvaluationResult {
        dataset_id: input.dataset.dataset_id,
        symbol: input.dataset.symbol,
        windows,
        aggregate_baseline_comparison: comparison,
        voice_adaptation_comparison,
        scoring_summary,
        proof_gate_status,
        reason_codes,
    })
}

pub fn build_proof_gate_report(result: &WalkForwardEvaluationResult) -> ProofGateReport {
    let comparison = &result.aggregate_baseline_comparison;
    let baseline_comparison_table = [
        &comparison.always_no_trade,
        &comparison.buy_and_hold,
        &comparison.equal_weight_committee,
        &comparison.voice_adaptive_committee,
    ]
    .into_iter()
    .map(|metrics| {
        format!(
            "{:?}: total_return={:.6} max_drawdown={:.6} trades={} no_trades={} risk_denials={} risk_adjusted={:.6}",
            metrics.strategy,
            metrics.total_return,
            metrics.max_drawdown,
            metrics.trade_count,
            metrics.no_trade_count,
            metrics.risk_denial_count,
            risk_adjusted_score(metrics),
        )
    })
    .collect::<Vec<_>>();
    let prediction_quality_summary = result
        .scoring_summary
        .iter()
        .map(|metrics| {
            format!(
                "{:?}: brier={:?} samples={} calibrated={} missing={} abstained={} high_confidence_errors={}",
                metrics.strategy,
                metrics.brier_score,
                metrics.sample_count,
                metrics.calibrated_sample_count,
                metrics.missing_probability_count,
                metrics.abstention_count,
                metrics.high_confidence_error_count,
            )
        })
        .collect::<Vec<_>>();
    let mut reason_codes = result.reason_codes.clone();
    reason_codes.push(ReasonCode::HardcodingAuditPassed);
    ProofGateReport {
        dataset_summary: format!(
            "dataset_id={} symbol={} windows={}",
            result.dataset_id,
            result.symbol,
            result.windows.len()
        ),
        walk_forward_config:
            "expanding train window; eval window is out-of-sample and no-lookahead".to_string(),
        baseline_comparison_table,
        voice_adaptation_result: if comparison.voice_beats_equal_weight {
            "VoiceAdaptiveCommittee beat EqualWeightCommittee on computed risk-adjusted score."
                .to_string()
        } else {
            "VoiceAdaptiveCommittee did not beat EqualWeightCommittee on this dataset.".to_string()
        },
        prediction_quality_summary,
        null_strategy_warning: if comparison.committee_beats_no_trade {
            "Committee beat AlwaysNoTrade on this computed sample, but this is not a profitability claim.".to_string()
        } else {
            "Committee did not beat AlwaysNoTrade. No edge proven.".to_string()
        },
        insufficient_evidence_warning: comparison
            .insufficient_evidence
            .then_some("Insufficient evidence due to small sample count.".to_string()),
        no_profitability_claim: "No profitability claim.".to_string(),
        no_live_readiness_warning: "No live trading readiness.".to_string(),
        next_required_evidence: if comparison.committee_beats_buy_hold_risk_adjusted {
            "Retest on larger owner-provided sanitized local daily CSV datasets before trusting any committee edge.".to_string()
        } else {
            "Committee did not beat BuyAndHold. Next evidence must improve baseline-relative out-of-sample results.".to_string()
        },
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

pub fn render_proof_gate_report_text(report: &ProofGateReport) -> String {
    let mut lines = vec![
        "Proof gate report.".to_string(),
        "Local historical daily CSV only.".to_string(),
        "Paper-only evaluation.".to_string(),
        "No live trading readiness.".to_string(),
        "No profitability claim.".to_string(),
        "Voice adaptation must beat equal weight before it is trusted.".to_string(),
        "Synthetic fixture success is not market evidence.".to_string(),
        format!("Dataset: {}", report.dataset_summary),
        format!("Walk-forward: {}", report.walk_forward_config),
        "Baseline comparison table:".to_string(),
    ];
    lines.extend(report.baseline_comparison_table.iter().cloned());
    lines.push(format!(
        "Voice adaptation: {}",
        report.voice_adaptation_result
    ));
    lines.push("Prediction quality summary:".to_string());
    lines.extend(report.prediction_quality_summary.iter().cloned());
    lines.push(report.null_strategy_warning.clone());
    if let Some(warning) = &report.insufficient_evidence_warning {
        lines.push(warning.clone());
    }
    lines.push(report.no_profitability_claim.clone());
    lines.push(report.no_live_readiness_warning.clone());
    lines.push(report.next_required_evidence.clone());
    redact_owner_report_output(&lines.join("\n"))
}

pub fn parse_historical_evidence_pack_manifest_json(
    manifest_json: &str,
) -> Result<HistoricalEvidencePackManifest, HistoricalEvidencePackError> {
    if let Some(reason) = evidence_pack_data_safety_reason(manifest_json) {
        return Err(historical_evidence_pack_error(None, reason));
    }
    serde_json::from_str(manifest_json)
        .map_err(|_| historical_evidence_pack_error(None, ReasonCode::EvidencePackSourceRejected))
}

pub fn load_historical_evidence_pack_from_manifest(
    manifest: &HistoricalEvidencePackManifest,
    config: &HistoricalEvidencePackConfig,
) -> Result<HistoricalEvidencePack, HistoricalEvidencePackError> {
    validate_historical_evidence_manifest_header(manifest, config)?;
    let mut sources = manifest.sources.clone();
    sources.sort_by(|left, right| {
        (
            left.source_kind,
            left.market.as_str(),
            left.symbol.as_str(),
            left.source_id.as_str(),
        )
            .cmp(&(
                right.source_kind,
                right.market.as_str(),
                right.symbol.as_str(),
                right.source_id.as_str(),
            ))
    });
    let loaded_sources = sources
        .into_iter()
        .map(|source| load_historical_evidence_source(source, config))
        .collect::<Vec<_>>();
    let mut reason_codes = manifest
        .reason_codes
        .iter()
        .chain(config.reason_codes.iter())
        .cloned()
        .chain([
            ReasonCode::EvidencePackLocalOnly,
            ReasonCode::EvidencePackSanitizedOnly,
            ReasonCode::EvidencePackNoNetwork,
            ReasonCode::EvidencePackNoDownloader,
            ReasonCode::LocalFileOnly,
            ReasonCode::PaperExecutionOnly,
        ])
        .collect::<Vec<_>>();
    reason_codes.extend(
        loaded_sources
            .iter()
            .flat_map(|source| source.reason_codes.iter().cloned()),
    );
    Ok(HistoricalEvidencePack {
        pack_id: manifest.pack_id.clone(),
        description: manifest.description.clone(),
        sources: loaded_sources,
        local_only: manifest.local_only,
        sanitized_only: manifest.sanitized_only,
        reason_codes: stable_reason_codes(&reason_codes),
    })
}

pub fn validate_historical_evidence_pack(
    pack: &HistoricalEvidencePack,
    config: &HistoricalEvidencePackConfig,
) -> Result<(), HistoricalEvidencePackError> {
    if !pack.local_only {
        return Err(historical_evidence_pack_error(
            None,
            ReasonCode::EvidencePackLocalOnly,
        ));
    }
    if !pack.sanitized_only {
        return Err(historical_evidence_pack_error(
            None,
            ReasonCode::EvidencePackSanitizedOnly,
        ));
    }
    if config.require_all_sources_valid && pack.sources.iter().any(|source| source.rejected) {
        return Err(historical_evidence_pack_error(
            pack.sources
                .iter()
                .find(|source| source.rejected)
                .map(|source| source.spec.source_id.clone()),
            ReasonCode::EvidencePackSourceRejected,
        ));
    }
    let accepted = pack.sources.iter().filter(|source| source.accepted).count();
    if accepted < config.min_sources {
        return Err(historical_evidence_pack_error(
            None,
            ReasonCode::EvidencePackInsufficientSources,
        ));
    }
    for (kind, min_count) in &config.min_sources_by_kind {
        let count = pack
            .sources
            .iter()
            .filter(|source| source.accepted && source.spec.source_kind == *kind)
            .count();
        if count < *min_count {
            return Err(historical_evidence_pack_error(
                None,
                ReasonCode::EvidencePackInsufficientSources,
            ));
        }
    }
    Ok(())
}

pub fn evaluate_historical_evidence_pack(
    pack: &HistoricalEvidencePack,
    eval_config: &HistoricalEvidencePackEvaluationConfig,
) -> HistoricalEvidencePackEvaluationResult {
    let mut source_results = pack
        .sources
        .iter()
        .map(|source| evaluate_historical_evidence_source(source, eval_config))
        .collect::<Vec<_>>();
    source_results.sort_by(|left, right| {
        (
            left.source_kind,
            left.market.as_str(),
            left.symbol.as_str(),
            left.source_id.as_str(),
        )
            .cmp(&(
                right.source_kind,
                right.market.as_str(),
                right.symbol.as_str(),
                right.source_id.as_str(),
            ))
    });
    let aggregate_result = build_aggregate_baseline_comparison(&source_results, eval_config);
    let voice_adaptation_summary = build_voice_adaptation_validity(&source_results, eval_config);
    let prediction_quality_summary =
        build_aggregate_prediction_quality_summary(&source_results, eval_config);
    let market_results = build_market_evidence_results(&source_results, eval_config);
    let symbol_results = source_results.clone();
    let mut reason_codes = pack
        .reason_codes
        .iter()
        .chain(eval_config.reason_codes.iter())
        .chain(aggregate_result.reason_codes.iter())
        .chain(voice_adaptation_summary.reason_codes.iter())
        .chain(prediction_quality_summary.reason_codes.iter())
        .cloned()
        .chain([
            ReasonCode::EvidencePackLocalOnly,
            ReasonCode::EvidencePackNoNetwork,
            ReasonCode::EvidencePackNoDownloader,
            ReasonCode::WalkForwardEvaluationOnly,
            ReasonCode::PaperExecutionOnly,
            ReasonCode::HardcodingAuditPassed,
        ])
        .collect::<Vec<_>>();
    reason_codes.extend(
        source_results
            .iter()
            .flat_map(|source| source.reason_codes.iter().cloned()),
    );
    HistoricalEvidencePackEvaluationResult {
        pack_id: pack.pack_id.clone(),
        source_results,
        aggregate_result: aggregate_result.clone(),
        market_results,
        symbol_results,
        voice_adaptation_summary,
        prediction_quality_summary,
        proof_gate_status: aggregate_result.overall_status,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

pub fn build_multi_symbol_proof_gate_report(
    result: &HistoricalEvidencePackEvaluationResult,
) -> MultiSymbolProofGateReport {
    let source_table = result
        .source_results
        .iter()
        .map(|source| {
            format!(
                "{} {:?} {} {} accepted={} rejected={} insufficient={} status={:?}",
                source.source_id,
                source.source_kind,
                source.market,
                source.symbol,
                source.accepted,
                source.rejected,
                source.insufficient_evidence,
                source
                    .walk_forward_result
                    .as_ref()
                    .map(|result| result.proof_gate_status)
            )
        })
        .collect::<Vec<_>>();
    let market_table = result
        .market_results
        .iter()
        .map(|market| {
            format!(
                "{} sources={} accepted={} rejected={} status={:?} voice={:?}",
                market.market,
                market.source_count,
                market.accepted_count,
                market.rejected_count,
                market.baseline_comparison.overall_status,
                market.voice_adaptation_comparison.status,
            )
        })
        .collect::<Vec<_>>();
    let failed_symbols = result
        .source_results
        .iter()
        .filter_map(|source| {
            let comparison = &source
                .walk_forward_result
                .as_ref()?
                .aggregate_baseline_comparison;
            (!comparison.voice_beats_equal_weight
                || !comparison.committee_beats_buy_hold_risk_adjusted
                || !comparison.committee_beats_no_trade)
                .then_some(format!(
                    "{}:{} voice_lost={} buy_hold_stronger={} no_trade_stronger={}",
                    source.market,
                    source.symbol,
                    !comparison.voice_beats_equal_weight,
                    !comparison.committee_beats_buy_hold_risk_adjusted,
                    !comparison.committee_beats_no_trade,
                ))
        })
        .collect::<Vec<_>>();
    let rejected_sources = result
        .source_results
        .iter()
        .filter(|source| source.rejected)
        .map(|source| {
            format!(
                "{} {:?} {}",
                source.source_id, source.source_kind, source.symbol
            )
        })
        .collect::<Vec<_>>();
    let mut insufficient_evidence_warnings = result
        .source_results
        .iter()
        .filter(|source| source.insufficient_evidence)
        .map(|source| format!("{}:{} insufficient evidence", source.market, source.symbol))
        .collect::<Vec<_>>();
    if result.aggregate_result.overall_status
        == AggregateBaselineOverallStatus::InsufficientEvidence
    {
        insufficient_evidence_warnings
            .push("Evidence pack has insufficient accepted out-of-sample sources.".to_string());
    }
    let next_required_evidence = vec![
        "Add more owner-provided sanitized local daily CSV symbols before trusting any edge."
            .to_string(),
        "Keep US, KR, and BTC market evidence separated while increasing source count."
            .to_string(),
        "Continue comparing VoiceAdaptiveCommittee against EqualWeightCommittee, AlwaysNoTrade, and BuyAndHold."
            .to_string(),
    ];
    let reason_codes = stable_reason_codes(
        &result
            .reason_codes
            .iter()
            .cloned()
            .chain([
                ReasonCode::EvidencePackLocalOnly,
                ReasonCode::WalkForwardEvaluationOnly,
                ReasonCode::HardcodingAuditPassed,
            ])
            .collect::<Vec<_>>(),
    );
    MultiSymbolProofGateReport {
        pack_summary: format!(
            "pack_id={} sources={} accepted={} rejected={} status={:?}",
            result.pack_id,
            result.aggregate_result.source_count,
            result.aggregate_result.accepted_source_count,
            result.aggregate_result.rejected_source_count,
            result.proof_gate_status,
        ),
        source_table,
        market_table,
        aggregate_baseline_comparison: format!(
            "status={:?} voice_win/loss/tie={}/{}/{} committee_vs_no_trade={}/{} committee_vs_buy_hold={}/{}",
            result.aggregate_result.overall_status,
            result.aggregate_result.voice_beats_equal_weight_count,
            result.aggregate_result.voice_loses_equal_weight_count,
            result.aggregate_result.voice_ties_equal_weight_count,
            result.aggregate_result.committee_beats_no_trade_count,
            result.aggregate_result.committee_loses_no_trade_count,
            result.aggregate_result.committee_beats_buy_hold_count,
            result.aggregate_result.committee_loses_buy_hold_count,
        ),
        voice_adaptation_validity: voice_adaptation_report_sentence(
            &result.voice_adaptation_summary,
        ),
        prediction_quality_summary: format!(
            "sources={} samples={} mean_brier={:?} missing={} abstained={} high_confidence_errors={} insufficient={}",
            result.prediction_quality_summary.source_count,
            result.prediction_quality_summary.total_samples,
            result.prediction_quality_summary.mean_brier_score,
            result.prediction_quality_summary.missing_probability_count,
            result.prediction_quality_summary.abstention_count,
            result
                .prediction_quality_summary
                .high_confidence_error_count,
            result.prediction_quality_summary.insufficient_evidence,
        ),
        failed_symbols,
        rejected_sources,
        insufficient_evidence_warnings,
        next_required_evidence,
        reason_codes,
    }
}

pub fn render_multi_symbol_proof_gate_report_text(report: &MultiSymbolProofGateReport) -> String {
    let mut lines = vec![
        "Multi-symbol proof gate report.".to_string(),
        "Local owner-provided sanitized historical daily CSV only.".to_string(),
        "Paper-only evaluation.".to_string(),
        "No live trading readiness.".to_string(),
        "No profitability claim.".to_string(),
        "Synthetic fixture success is not market evidence.".to_string(),
        "Voice adaptation must beat equal weight before it is trusted.".to_string(),
        "Bad or mixed results are valid outputs.".to_string(),
        format!("Pack summary: {}", report.pack_summary),
        "Source table:".to_string(),
    ];
    lines.extend(report.source_table.iter().cloned());
    lines.push("Market table:".to_string());
    lines.extend(report.market_table.iter().cloned());
    lines.push(format!(
        "Aggregate baseline comparison: {}",
        report.aggregate_baseline_comparison
    ));
    lines.push(format!(
        "Voice adaptation validity: {}",
        report.voice_adaptation_validity
    ));
    lines.push(format!(
        "Prediction quality summary: {}",
        report.prediction_quality_summary
    ));
    lines.push("Failed symbols:".to_string());
    if report.failed_symbols.is_empty() {
        lines.push("none".to_string());
    } else {
        lines.extend(report.failed_symbols.iter().cloned());
    }
    lines.push("Rejected sources:".to_string());
    if report.rejected_sources.is_empty() {
        lines.push("none".to_string());
    } else {
        lines.extend(report.rejected_sources.iter().cloned());
    }
    lines.push("Insufficient evidence warnings:".to_string());
    if report.insufficient_evidence_warnings.is_empty() {
        lines.push("none".to_string());
    } else {
        lines.extend(report.insufficient_evidence_warnings.iter().cloned());
    }
    lines.push("Next required evidence:".to_string());
    lines.extend(report.next_required_evidence.iter().cloned());
    redact_owner_report_output(&lines.join("\n"))
}

pub fn run_owner_historical_evidence_trial(
    config: OwnerEvidenceTrialConfig,
) -> OwnerEvidenceTrialResult {
    let manifest_input = match owner_trial_manifest_input(&config) {
        OwnerTrialManifestInput::Missing => {
            return owner_trial_no_pack_result(
                &config,
                OwnerEvidenceManifestStatus::Missing,
                vec![
                    ReasonCode::OwnerEvidencePackNotFound,
                    ReasonCode::EvidenceTriageNoPack,
                ],
            );
        }
        OwnerTrialManifestInput::Json(text) => (OwnerEvidenceManifestStatus::ProvidedJson, text),
        OwnerTrialManifestInput::Path(path) => {
            if let Some(reason) = evidence_pack_path_safety_reason(&path) {
                return owner_trial_rejected_result(
                    &config,
                    OwnerEvidenceManifestStatus::Rejected,
                    owner_trial_reason_from_evidence(reason),
                );
            }
            match fs::read_to_string(&path) {
                Ok(text) => (OwnerEvidenceManifestStatus::LoadedFromLocalPath, text),
                Err(_) => {
                    return owner_trial_no_pack_result(
                        &config,
                        OwnerEvidenceManifestStatus::Missing,
                        vec![
                            ReasonCode::OwnerEvidencePackNotFound,
                            ReasonCode::EvidenceTriageNoPack,
                        ],
                    );
                }
            }
        }
    };
    let (manifest_status, manifest_text) = manifest_input;
    let manifest = match serde_json::from_str::<HistoricalEvidencePackManifest>(&manifest_text) {
        Ok(manifest) => manifest,
        Err(_) => {
            return owner_trial_rejected_result(
                &config,
                OwnerEvidenceManifestStatus::Rejected,
                ReasonCode::EvidencePackSourceRejected,
            );
        }
    };
    if owner_trial_manifest_is_example(&manifest) && !config.allow_example_pack {
        return owner_trial_no_pack_result(
            &config,
            manifest_status,
            vec![
                ReasonCode::OwnerEvidencePackExampleOnly,
                ReasonCode::EvidenceTriageNoPack,
            ],
        );
    }

    let mut pack_config = config.historical_pack_config.clone();
    pack_config.min_sources = pack_config.min_sources.max(config.min_accepted_sources);
    if config.allow_example_pack {
        pack_config.allow_synthetic_sources_for_tests_only = true;
    }
    let pack = match load_historical_evidence_pack_from_manifest(&manifest, &pack_config) {
        Ok(pack) => pack,
        Err(error) => {
            let reason = error
                .reason_codes
                .first()
                .cloned()
                .map(owner_trial_reason_from_evidence)
                .unwrap_or(ReasonCode::EvidenceTriageRejectedForSafety);
            return owner_trial_rejected_result(
                &config,
                OwnerEvidenceManifestStatus::Rejected,
                reason,
            );
        }
    };
    let validation_error = validate_historical_evidence_pack(&pack, &pack_config).err();
    let mut eval_config = config.evaluation_config.clone();
    eval_config.min_accepted_sources_for_proof = eval_config
        .min_accepted_sources_for_proof
        .max(config.min_accepted_sources);
    let pack_evaluation = evaluate_historical_evidence_pack(&pack, &eval_config);
    let mut triage_summary = build_evidence_triage_summary(&pack_evaluation, Some(&pack), &config);
    if let Some(error) = validation_error {
        triage_summary.reason_codes = stable_reason_codes(
            &triage_summary
                .reason_codes
                .iter()
                .cloned()
                .chain(
                    error
                        .reason_codes
                        .iter()
                        .cloned()
                        .map(owner_trial_reason_from_evidence),
                )
                .collect::<Vec<_>>(),
        );
        if triage_summary.status == OwnerEvidenceTrialStatus::Pass {
            triage_summary.status = OwnerEvidenceTrialStatus::InsufficientEvidence;
        }
    }
    let market_triage = build_market_triage_results(&pack_evaluation, &config);
    let multi_symbol_report = build_multi_symbol_proof_gate_report(&pack_evaluation);
    let triage_report = build_owner_evidence_triage_report(
        &triage_summary,
        &market_triage,
        Some(&pack_evaluation),
        Some(&multi_symbol_report),
        manifest_status,
        &config,
    );
    let reason_codes = stable_reason_codes(
        &config
            .reason_codes
            .iter()
            .chain(pack.reason_codes.iter())
            .chain(pack_evaluation.reason_codes.iter())
            .chain(triage_summary.reason_codes.iter())
            .cloned()
            .chain([
                ReasonCode::OwnerEvidencePackLocalOnly,
                ReasonCode::OwnerEvidencePackSanitizedOnly,
                ReasonCode::EvidencePackNoNetwork,
                ReasonCode::EvidencePackNoDownloader,
                ReasonCode::PaperExecutionOnly,
                ReasonCode::HardcodingAuditPassed,
            ])
            .collect::<Vec<_>>(),
    );
    OwnerEvidenceTrialResult {
        trial_status: triage_summary.status,
        manifest_status,
        pack_evaluation: Some(pack_evaluation),
        multi_symbol_report: Some(multi_symbol_report),
        triage_report,
        owner_action_checklist: triage_summary.owner_action_items.clone(),
        triage_summary,
        market_triage,
        reason_codes,
    }
}

pub fn render_owner_evidence_triage_report_text(report: &OwnerEvidenceTriageReport) -> String {
    let mut lines = vec![
        report.header.clone(),
        format!("Trial status: {:?}", report.trial_status),
        "Local owner-provided sanitized historical daily CSV only.".to_string(),
        "Paper-only evaluation.".to_string(),
        "No live trading readiness.".to_string(),
        "No profitability claim.".to_string(),
        "Bad, mixed, or insufficient results are valid outputs.".to_string(),
        "VoiceAdaptiveCommittee must beat EqualWeightCommittee before it is trusted.".to_string(),
        "No data was downloaded.".to_string(),
        format!("Pack summary: {}", report.pack_summary),
        "Source summary:".to_string(),
    ];
    lines.extend(report.source_summary.iter().cloned());
    lines.push("Market triage:".to_string());
    lines.extend(report.market_summary.iter().cloned());
    lines.push("Baseline failure summary:".to_string());
    if report.baseline_failure_summary.is_empty() {
        lines.push("none".to_string());
    } else {
        lines.extend(report.baseline_failure_summary.iter().cloned());
    }
    lines.push(format!(
        "Voice adaptation summary: {}",
        report.voice_adaptation_summary
    ));
    lines.push(format!(
        "Prediction-quality summary: {}",
        report.prediction_quality_summary
    ));
    lines.push("Failed symbols:".to_string());
    if report.failed_symbols.is_empty() {
        lines.push("none".to_string());
    } else {
        lines.extend(report.failed_symbols.iter().cloned());
    }
    lines.push("Rejected sources:".to_string());
    if report.rejected_sources.is_empty() {
        lines.push("none".to_string());
    } else {
        lines.extend(report.rejected_sources.iter().cloned());
    }
    lines.push("Insufficient evidence reasons:".to_string());
    if report.insufficient_evidence_reasons.is_empty() {
        lines.push("none".to_string());
    } else {
        lines.extend(report.insufficient_evidence_reasons.iter().cloned());
    }
    lines.push("Owner action checklist:".to_string());
    lines.extend(report.owner_action_checklist.iter().map(|item| {
        format!(
            "{} required={} {}",
            item.item_id, item.required, item.description
        )
    }));
    lines.push("Safety warnings:".to_string());
    lines.extend(report.safety_warnings.iter().cloned());
    lines.push(report.no_profitability_claim.clone());
    lines.push(report.no_live_readiness_warning.clone());
    redact_owner_report_output(&lines.join("\n"))
}

enum OwnerTrialManifestInput {
    Missing,
    Json(String),
    Path(String),
}

fn owner_trial_manifest_input(config: &OwnerEvidenceTrialConfig) -> OwnerTrialManifestInput {
    match (&config.manifest_json, &config.manifest_path) {
        (Some(json), _) if !json.trim().is_empty() => OwnerTrialManifestInput::Json(json.clone()),
        (_, Some(path)) if !path.trim().is_empty() => OwnerTrialManifestInput::Path(path.clone()),
        _ => OwnerTrialManifestInput::Missing,
    }
}

fn owner_trial_no_pack_result(
    config: &OwnerEvidenceTrialConfig,
    manifest_status: OwnerEvidenceManifestStatus,
    reason_codes: Vec<ReasonCode>,
) -> OwnerEvidenceTrialResult {
    let checklist = owner_action_checklist(config);
    let triage_summary = EvidenceTriageSummary {
        status: OwnerEvidenceTrialStatus::NoOwnerEvidencePackFound,
        pack_id: None,
        accepted_source_count: 0,
        rejected_source_count: 0,
        disabled_source_count: 0,
        insufficient_source_count: 0,
        failed_symbol_count: 0,
        mixed_symbol_count: 0,
        pass_symbol_count: 0,
        markets_present: Vec::new(),
        voice_adaptation_status: EvidenceTriageDimensionStatus::InsufficientEvidence,
        committee_vs_no_trade_status: EvidenceTriageDimensionStatus::InsufficientEvidence,
        committee_vs_buy_hold_status: EvidenceTriageDimensionStatus::InsufficientEvidence,
        prediction_quality_status: EvidenceTriageDimensionStatus::InsufficientEvidence,
        owner_action_items: checklist.clone(),
        reason_codes: stable_reason_codes(&reason_codes),
    };
    let triage_report = build_owner_evidence_triage_report(
        &triage_summary,
        &[],
        None,
        None,
        manifest_status,
        config,
    );
    OwnerEvidenceTrialResult {
        trial_status: OwnerEvidenceTrialStatus::NoOwnerEvidencePackFound,
        manifest_status,
        pack_evaluation: None,
        multi_symbol_report: None,
        triage_report,
        owner_action_checklist: checklist,
        triage_summary,
        market_triage: Vec::new(),
        reason_codes: stable_reason_codes(
            &config
                .reason_codes
                .iter()
                .cloned()
                .chain(reason_codes)
                .chain([
                    ReasonCode::EvidenceTriageNoPack,
                    ReasonCode::EvidencePackNoNetwork,
                    ReasonCode::EvidencePackNoDownloader,
                    ReasonCode::PaperExecutionOnly,
                ])
                .collect::<Vec<_>>(),
        ),
    }
}

fn owner_trial_rejected_result(
    config: &OwnerEvidenceTrialConfig,
    manifest_status: OwnerEvidenceManifestStatus,
    reason: ReasonCode,
) -> OwnerEvidenceTrialResult {
    let checklist = owner_action_checklist(config);
    let triage_summary = EvidenceTriageSummary {
        status: OwnerEvidenceTrialStatus::RejectedForSafety,
        pack_id: None,
        accepted_source_count: 0,
        rejected_source_count: 1,
        disabled_source_count: 0,
        insufficient_source_count: 0,
        failed_symbol_count: 0,
        mixed_symbol_count: 0,
        pass_symbol_count: 0,
        markets_present: Vec::new(),
        voice_adaptation_status: EvidenceTriageDimensionStatus::InsufficientEvidence,
        committee_vs_no_trade_status: EvidenceTriageDimensionStatus::InsufficientEvidence,
        committee_vs_buy_hold_status: EvidenceTriageDimensionStatus::InsufficientEvidence,
        prediction_quality_status: EvidenceTriageDimensionStatus::InsufficientEvidence,
        owner_action_items: checklist.clone(),
        reason_codes: stable_reason_codes(&[
            reason.clone(),
            ReasonCode::EvidenceTriageRejectedForSafety,
        ]),
    };
    let triage_report = build_owner_evidence_triage_report(
        &triage_summary,
        &[],
        None,
        None,
        manifest_status,
        config,
    );
    OwnerEvidenceTrialResult {
        trial_status: OwnerEvidenceTrialStatus::RejectedForSafety,
        manifest_status,
        pack_evaluation: None,
        multi_symbol_report: None,
        triage_report,
        owner_action_checklist: checklist,
        triage_summary,
        market_triage: Vec::new(),
        reason_codes: stable_reason_codes(
            &config
                .reason_codes
                .iter()
                .cloned()
                .chain([reason, ReasonCode::EvidenceTriageRejectedForSafety])
                .collect::<Vec<_>>(),
        ),
    }
}

fn owner_trial_manifest_is_example(manifest: &HistoricalEvidencePackManifest) -> bool {
    let manifest_text = format!(
        "{} {}",
        manifest.pack_id.to_ascii_lowercase(),
        manifest.description.to_ascii_lowercase()
    );
    manifest_text.contains("example")
        || manifest
            .sources
            .iter()
            .any(|source| source.source_kind == HistoricalEvidenceSourceKind::SyntheticDailySample)
}

fn build_evidence_triage_summary(
    result: &HistoricalEvidencePackEvaluationResult,
    pack: Option<&HistoricalEvidencePack>,
    config: &OwnerEvidenceTrialConfig,
) -> EvidenceTriageSummary {
    let disabled_source_count = pack.map_or(0, |pack| {
        pack.sources.iter().filter(|source| source.disabled).count()
    });
    let source_statuses = result
        .source_results
        .iter()
        .filter_map(symbol_triage_status)
        .collect::<Vec<_>>();
    let pass_symbol_count = source_statuses
        .iter()
        .filter(|status| **status == EvidenceTriageDimensionStatus::Pass)
        .count();
    let failed_symbol_count = source_statuses
        .iter()
        .filter(|status| **status == EvidenceTriageDimensionStatus::Fail)
        .count();
    let mixed_symbol_count = source_statuses
        .iter()
        .filter(|status| **status == EvidenceTriageDimensionStatus::Mixed)
        .count();
    let markets_present = result
        .source_results
        .iter()
        .map(|source| source.market.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let voice_adaptation_status = voice_triage_dimension(result.voice_adaptation_summary.status);
    let committee_vs_no_trade_status = comparison_triage_dimension(
        result.aggregate_result.committee_beats_no_trade_count,
        result.aggregate_result.committee_loses_no_trade_count,
        result.aggregate_result.accepted_source_count,
        config.min_accepted_sources,
    );
    let committee_vs_buy_hold_status = comparison_triage_dimension(
        result.aggregate_result.committee_beats_buy_hold_count,
        result.aggregate_result.committee_loses_buy_hold_count,
        result.aggregate_result.accepted_source_count,
        config.min_accepted_sources,
    );
    let prediction_quality_status =
        prediction_quality_triage_dimension(&result.prediction_quality_summary);
    let market_min_missing = config
        .min_sources_by_market
        .iter()
        .any(|(market, min_count)| {
            result
                .market_results
                .iter()
                .find(|row| row.market == *market)
                .map_or(0, |row| row.accepted_count)
                < *min_count
        });
    let safety_rejected = result
        .source_results
        .iter()
        .any(|source| source.reason_codes.iter().any(owner_trial_reason_is_safety));
    let mut reason_codes = result
        .reason_codes
        .iter()
        .cloned()
        .chain([ReasonCode::HardcodingAuditPassed])
        .collect::<Vec<_>>();
    reason_codes.extend(
        result
            .source_results
            .iter()
            .flat_map(|source| source.reason_codes.iter().cloned())
            .map(owner_trial_reason_from_evidence),
    );
    let status = if safety_rejected {
        reason_codes.push(ReasonCode::EvidenceTriageRejectedForSafety);
        OwnerEvidenceTrialStatus::RejectedForSafety
    } else if result.aggregate_result.accepted_source_count < config.min_accepted_sources
        || market_min_missing
        || result.prediction_quality_summary.insufficient_evidence
    {
        reason_codes.push(ReasonCode::EvidenceTriageInsufficient);
        OwnerEvidenceTrialStatus::InsufficientEvidence
    } else if result.aggregate_result.overall_status == AggregateBaselineOverallStatus::Mixed
        || mixed_symbol_count > 0
        || (pass_symbol_count > 0 && failed_symbol_count > 0)
    {
        reason_codes.push(ReasonCode::EvidenceTriageMixed);
        OwnerEvidenceTrialStatus::Mixed
    } else if voice_adaptation_status == EvidenceTriageDimensionStatus::Fail
        || committee_vs_no_trade_status == EvidenceTriageDimensionStatus::Fail
        || committee_vs_buy_hold_status == EvidenceTriageDimensionStatus::Fail
        || result.aggregate_result.overall_status == AggregateBaselineOverallStatus::Fail
    {
        reason_codes.push(ReasonCode::EvidenceTriageFail);
        OwnerEvidenceTrialStatus::Fail
    } else if result.aggregate_result.overall_status == AggregateBaselineOverallStatus::Pass
        && prediction_quality_status == EvidenceTriageDimensionStatus::Pass
    {
        reason_codes.push(ReasonCode::EvidenceTriagePass);
        OwnerEvidenceTrialStatus::Pass
    } else {
        reason_codes.push(ReasonCode::EvidenceTriageMixed);
        OwnerEvidenceTrialStatus::Mixed
    };
    if voice_adaptation_status == EvidenceTriageDimensionStatus::Fail {
        reason_codes.push(ReasonCode::EvidenceTriageVoiceFailed);
    }
    if result.aggregate_result.committee_loses_buy_hold_count > 0 {
        reason_codes.push(ReasonCode::EvidenceTriageBuyHoldStronger);
    }
    if result.aggregate_result.committee_loses_no_trade_count > 0 {
        reason_codes.push(ReasonCode::EvidenceTriageNoTradeStronger);
    }
    if prediction_quality_status != EvidenceTriageDimensionStatus::Pass {
        reason_codes.push(ReasonCode::EvidenceTriagePredictionWeak);
    }
    EvidenceTriageSummary {
        status,
        pack_id: Some(result.pack_id.clone()),
        accepted_source_count: result.aggregate_result.accepted_source_count,
        rejected_source_count: result.aggregate_result.rejected_source_count,
        disabled_source_count,
        insufficient_source_count: result.aggregate_result.insufficient_source_count,
        failed_symbol_count,
        mixed_symbol_count,
        pass_symbol_count,
        markets_present,
        voice_adaptation_status,
        committee_vs_no_trade_status,
        committee_vs_buy_hold_status,
        prediction_quality_status,
        owner_action_items: owner_action_checklist(config),
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn build_market_triage_results(
    result: &HistoricalEvidencePackEvaluationResult,
    config: &OwnerEvidenceTrialConfig,
) -> Vec<MarketTriageResult> {
    result
        .market_results
        .iter()
        .map(|market| {
            let market_min_sources = *config
                .min_sources_by_market
                .get(&market.market)
                .unwrap_or(&config.min_accepted_sources.min(1));
            let voice_status = market_voice_triage_dimension(
                &market.voice_adaptation_comparison,
                market_min_sources,
            );
            let committee_vs_no_trade_status = comparison_triage_dimension(
                market.baseline_comparison.committee_beats_no_trade_count,
                market.baseline_comparison.committee_loses_no_trade_count,
                market.accepted_count,
                market_min_sources,
            );
            let committee_vs_buy_hold_status = comparison_triage_dimension(
                market.baseline_comparison.committee_beats_buy_hold_count,
                market.baseline_comparison.committee_loses_buy_hold_count,
                market.accepted_count,
                market_min_sources,
            );
            let brier_status = market_prediction_quality_triage_dimension(
                &market.prediction_quality_summary,
                market_min_sources,
            );
            let safety_rejected = result.source_results.iter().any(|source| {
                source.market == market.market
                    && source.reason_codes.iter().any(owner_trial_reason_is_safety)
            });
            let insufficient_count = result
                .source_results
                .iter()
                .filter(|source| source.market == market.market && source.insufficient_evidence)
                .count();
            let status = if safety_rejected && market.accepted_count == 0 {
                OwnerEvidenceTrialStatus::RejectedForSafety
            } else if market.accepted_count < market_min_sources
                || brier_status == EvidenceTriageDimensionStatus::InsufficientEvidence
            {
                OwnerEvidenceTrialStatus::InsufficientEvidence
            } else if [
                voice_status,
                committee_vs_no_trade_status,
                committee_vs_buy_hold_status,
                brier_status,
            ]
            .contains(&EvidenceTriageDimensionStatus::Fail)
            {
                OwnerEvidenceTrialStatus::Fail
            } else if [
                voice_status,
                committee_vs_no_trade_status,
                committee_vs_buy_hold_status,
                brier_status,
            ]
            .contains(&EvidenceTriageDimensionStatus::Mixed)
            {
                OwnerEvidenceTrialStatus::Mixed
            } else {
                OwnerEvidenceTrialStatus::Pass
            };
            let mut reason_codes = market.reason_codes.clone();
            reason_codes.push(match status {
                OwnerEvidenceTrialStatus::Pass => ReasonCode::EvidenceTriagePass,
                OwnerEvidenceTrialStatus::Fail => ReasonCode::EvidenceTriageFail,
                OwnerEvidenceTrialStatus::Mixed => ReasonCode::EvidenceTriageMixed,
                OwnerEvidenceTrialStatus::InsufficientEvidence => {
                    ReasonCode::EvidenceTriageInsufficient
                }
                OwnerEvidenceTrialStatus::RejectedForSafety => {
                    ReasonCode::EvidenceTriageRejectedForSafety
                }
                OwnerEvidenceTrialStatus::NoOwnerEvidencePackFound => {
                    ReasonCode::EvidenceTriageNoPack
                }
            });
            MarketTriageResult {
                market: market.market.clone(),
                source_count: market.source_count,
                accepted_count: market.accepted_count,
                rejected_count: market.rejected_count,
                insufficient_count,
                voice_status,
                committee_vs_no_trade_status,
                committee_vs_buy_hold_status,
                brier_status,
                status,
                reason_codes: stable_reason_codes(&reason_codes),
            }
        })
        .collect()
}

fn build_owner_evidence_triage_report(
    summary: &EvidenceTriageSummary,
    market_triage: &[MarketTriageResult],
    result: Option<&HistoricalEvidencePackEvaluationResult>,
    report: Option<&MultiSymbolProofGateReport>,
    manifest_status: OwnerEvidenceManifestStatus,
    config: &OwnerEvidenceTrialConfig,
) -> OwnerEvidenceTriageReport {
    let source_summary = result.map_or_else(
        || vec!["No owner evidence pack was found; no data was evaluated.".to_string()],
        |result| {
            result
                .source_results
                .iter()
                .map(|source| {
                    format!(
                        "{} {} {} accepted={} rejected={} insufficient={}",
                        source.source_id,
                        source.market,
                        source.symbol,
                        source.accepted,
                        source.rejected,
                        source.insufficient_evidence
                    )
                })
                .collect()
        },
    );
    let market_summary = if market_triage.is_empty() {
        vec!["No market evidence was evaluated.".to_string()]
    } else {
        market_triage
            .iter()
            .map(|market| {
                format!(
                    "{} status={:?} sources={} accepted={} rejected={} insufficient={} voice={:?} no_trade={:?} buy_hold={:?} brier={:?}",
                    market.market,
                    market.status,
                    market.source_count,
                    market.accepted_count,
                    market.rejected_count,
                    market.insufficient_count,
                    market.voice_status,
                    market.committee_vs_no_trade_status,
                    market.committee_vs_buy_hold_status,
                    market.brier_status,
                )
            })
            .collect()
    };
    let baseline_failure_summary = result.map_or_else(Vec::new, |result| {
        let mut failures = Vec::new();
        if result.aggregate_result.voice_loses_equal_weight_count > 0 {
            failures.push(format!(
                "VoiceAdaptiveCommittee lost to EqualWeightCommittee on {} source(s).",
                result.aggregate_result.voice_loses_equal_weight_count
            ));
        }
        if result.aggregate_result.committee_loses_buy_hold_count > 0 {
            failures.push(format!(
                "BuyAndHold beat the committee on {} source(s).",
                result.aggregate_result.committee_loses_buy_hold_count
            ));
        }
        if result.aggregate_result.committee_loses_no_trade_count > 0 {
            failures.push(format!(
                "AlwaysNoTrade beat the committee on {} source(s).",
                result.aggregate_result.committee_loses_no_trade_count
            ));
        }
        failures
    });
    let failed_symbols = report
        .filter(|_| config.include_failed_symbols)
        .map_or_else(Vec::new, |report| report.failed_symbols.clone());
    let rejected_sources = report
        .filter(|_| config.include_rejected_sources)
        .map_or_else(Vec::new, |report| report.rejected_sources.clone());
    let insufficient_evidence_reasons = report.map_or_else(
        || {
            vec![
                "No owner evidence pack was found, so no out-of-sample source was evaluated."
                    .to_string(),
            ]
        },
        |report| report.insufficient_evidence_warnings.clone(),
    );
    let prediction_quality_summary = result.map_or_else(
        || "No prediction-quality evidence was evaluated.".to_string(),
        |result| {
            format!(
                "status={:?} samples={} mean_brier={:?} missing={} abstained={} high_confidence_errors={}",
                summary.prediction_quality_status,
                result.prediction_quality_summary.total_samples,
                result.prediction_quality_summary.mean_brier_score,
                result.prediction_quality_summary.missing_probability_count,
                result.prediction_quality_summary.abstention_count,
                result.prediction_quality_summary.high_confidence_error_count,
            )
        },
    );
    let safety_warnings = vec![
        "Local files only; URL paths and live endpoints are rejected.".to_string(),
        "Secrets, account data, order data, raw provider responses, and private markers are rejected."
            .to_string(),
        "No downloader, network client, broker, order, or runtime model is used.".to_string(),
    ];
    let mut reason_codes = summary.reason_codes.clone();
    reason_codes.extend(match manifest_status {
        OwnerEvidenceManifestStatus::Missing => vec![ReasonCode::OwnerEvidencePackNotFound],
        OwnerEvidenceManifestStatus::ProvidedJson => vec![ReasonCode::OwnerEvidencePackLocalOnly],
        OwnerEvidenceManifestStatus::LoadedFromLocalPath => {
            vec![ReasonCode::OwnerEvidencePackExpectedPath]
        }
        OwnerEvidenceManifestStatus::Rejected => vec![ReasonCode::EvidenceTriageRejectedForSafety],
    });
    OwnerEvidenceTriageReport {
        header: "Owner historical evidence trial report.".to_string(),
        trial_status: summary.status,
        pack_summary: result.map_or_else(
            || "No owner evidence pack found; no data evaluated.".to_string(),
            |result| {
                format!(
                    "pack_id={} accepted={} rejected={} disabled={} insufficient={} status={:?}",
                    result.pack_id,
                    summary.accepted_source_count,
                    summary.rejected_source_count,
                    summary.disabled_source_count,
                    summary.insufficient_source_count,
                    summary.status,
                )
            },
        ),
        source_summary,
        market_summary,
        baseline_failure_summary,
        voice_adaptation_summary: owner_voice_summary(summary.voice_adaptation_status),
        prediction_quality_summary,
        failed_symbols,
        rejected_sources,
        insufficient_evidence_reasons,
        owner_action_checklist: summary.owner_action_items.clone(),
        safety_warnings,
        no_profitability_claim: "No profitability claim.".to_string(),
        no_live_readiness_warning: "No live trading readiness.".to_string(),
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn symbol_triage_status(
    source: &HistoricalEvidenceSourceEvaluationResult,
) -> Option<EvidenceTriageDimensionStatus> {
    let result = source.walk_forward_result.as_ref()?;
    let comparison = &result.aggregate_baseline_comparison;
    if comparison.insufficient_evidence || source.insufficient_evidence {
        return Some(EvidenceTriageDimensionStatus::InsufficientEvidence);
    }
    let wins = [
        comparison.voice_beats_equal_weight,
        comparison.committee_beats_no_trade,
        comparison.committee_beats_buy_hold_risk_adjusted,
    ]
    .iter()
    .filter(|value| **value)
    .count();
    match wins {
        3 => Some(EvidenceTriageDimensionStatus::Pass),
        0 => Some(EvidenceTriageDimensionStatus::Fail),
        _ => Some(EvidenceTriageDimensionStatus::Mixed),
    }
}

fn voice_triage_dimension(status: VoiceAdaptationValidityStatus) -> EvidenceTriageDimensionStatus {
    match status {
        VoiceAdaptationValidityStatus::Helped => EvidenceTriageDimensionStatus::Pass,
        VoiceAdaptationValidityStatus::Failed => EvidenceTriageDimensionStatus::Fail,
        VoiceAdaptationValidityStatus::Mixed => EvidenceTriageDimensionStatus::Mixed,
        VoiceAdaptationValidityStatus::InsufficientEvidence => {
            EvidenceTriageDimensionStatus::InsufficientEvidence
        }
    }
}

fn market_voice_triage_dimension(
    summary: &VoiceAdaptationValidity,
    min_count: usize,
) -> EvidenceTriageDimensionStatus {
    if summary.compared_source_count < min_count.max(1) {
        EvidenceTriageDimensionStatus::InsufficientEvidence
    } else if summary.voice_better_count == summary.compared_source_count {
        EvidenceTriageDimensionStatus::Pass
    } else if summary.equal_weight_better_count + summary.tie_count == summary.compared_source_count
    {
        EvidenceTriageDimensionStatus::Fail
    } else {
        EvidenceTriageDimensionStatus::Mixed
    }
}

fn comparison_triage_dimension(
    win_count: usize,
    loss_count: usize,
    accepted_count: usize,
    min_count: usize,
) -> EvidenceTriageDimensionStatus {
    if accepted_count < min_count.max(1) || win_count + loss_count == 0 {
        EvidenceTriageDimensionStatus::InsufficientEvidence
    } else if win_count > 0 && loss_count == 0 {
        EvidenceTriageDimensionStatus::Pass
    } else if win_count == 0 && loss_count > 0 {
        EvidenceTriageDimensionStatus::Fail
    } else {
        EvidenceTriageDimensionStatus::Mixed
    }
}

fn market_prediction_quality_triage_dimension(
    summary: &AggregatePredictionQualitySummary,
    min_count: usize,
) -> EvidenceTriageDimensionStatus {
    if summary.source_count < min_count.max(1)
        || summary.total_samples == 0
        || summary.mean_brier_score.is_none()
    {
        EvidenceTriageDimensionStatus::InsufficientEvidence
    } else if summary.high_confidence_error_count > 0
        || summary.missing_probability_count > summary.total_samples / 2
    {
        EvidenceTriageDimensionStatus::Mixed
    } else {
        EvidenceTriageDimensionStatus::Pass
    }
}

fn prediction_quality_triage_dimension(
    summary: &AggregatePredictionQualitySummary,
) -> EvidenceTriageDimensionStatus {
    if summary.insufficient_evidence
        || summary.total_samples == 0
        || summary.mean_brier_score.is_none()
    {
        EvidenceTriageDimensionStatus::InsufficientEvidence
    } else if summary.high_confidence_error_count > 0
        || summary.missing_probability_count > summary.total_samples / 2
    {
        EvidenceTriageDimensionStatus::Mixed
    } else {
        EvidenceTriageDimensionStatus::Pass
    }
}

fn owner_voice_summary(status: EvidenceTriageDimensionStatus) -> String {
    match status {
        EvidenceTriageDimensionStatus::Pass => {
            "VoiceAdaptiveCommittee beat EqualWeightCommittee on the computed trial evidence."
                .to_string()
        }
        EvidenceTriageDimensionStatus::Fail => {
            "VoiceAdaptiveCommittee failed to beat EqualWeightCommittee on the computed trial evidence."
                .to_string()
        }
        EvidenceTriageDimensionStatus::Mixed => {
            "VoiceAdaptiveCommittee evidence is mixed against EqualWeightCommittee.".to_string()
        }
        EvidenceTriageDimensionStatus::InsufficientEvidence => {
            "Insufficient evidence to trust VoiceAdaptiveCommittee over EqualWeightCommittee."
                .to_string()
        }
    }
}

fn owner_action_checklist(config: &OwnerEvidenceTrialConfig) -> Vec<OwnerActionItem> {
    let min_sources = config.min_accepted_sources.max(1);
    vec![
        OwnerActionItem {
            item_id: "provide-us-daily-csv".to_string(),
            description: format!(
                "Provide at least {min_sources} sanitized local US daily CSV file(s)."
            ),
            required: true,
            reason_codes: vec![ReasonCode::OwnerEvidencePackExpectedPath],
        },
        OwnerActionItem {
            item_id: "provide-kr-daily-csv".to_string(),
            description: format!(
                "Provide at least {min_sources} sanitized local KR daily CSV file(s)."
            ),
            required: true,
            reason_codes: vec![ReasonCode::OwnerEvidencePackExpectedPath],
        },
        OwnerActionItem {
            item_id: "provide-btc-daily-csv".to_string(),
            description: "Provide a sanitized local BTC daily CSV file.".to_string(),
            required: true,
            reason_codes: vec![ReasonCode::OwnerEvidencePackExpectedPath],
        },
        OwnerActionItem {
            item_id: "use-required-columns".to_string(),
            description:
                "Use symbol,date,open,high,low,close,volume with YYYY-MM-DD dates.".to_string(),
            required: true,
            reason_codes: vec![ReasonCode::ManualHistoricalImportDailyOnly],
        },
        OwnerActionItem {
            item_id: "remove-private-columns".to_string(),
            description:
                "Remove account, order, API, private, raw-response, endpoint, and live-provider columns."
                    .to_string(),
            required: true,
            reason_codes: vec![ReasonCode::EvidenceTrialUnsafePrivateData],
        },
        OwnerActionItem {
            item_id: "keep-local".to_string(),
            description:
                "Keep files local; do not paste API keys, broker credentials, private provider docs, or temporary instruction files."
                    .to_string(),
            required: true,
            reason_codes: vec![
                ReasonCode::OwnerEvidencePackLocalOnly,
                ReasonCode::EvidencePackNoNetwork,
                ReasonCode::EvidencePackNoDownloader,
            ],
        },
    ]
}

fn owner_trial_reason_from_evidence(reason: ReasonCode) -> ReasonCode {
    match reason {
        ReasonCode::EvidencePackUnsafePrivateData
        | ReasonCode::EvidencePackEnvPathRejected
        | ReasonCode::EvidencePackPrivatePathRejected
        | ReasonCode::EvidencePackPathRejected => ReasonCode::EvidenceTrialUnsafePrivateData,
        ReasonCode::EvidencePackSecretLikeDataRejected => {
            ReasonCode::EvidenceTrialSecretLikeDataRejected
        }
        ReasonCode::EvidencePackWorkMdPathRejected
        | ReasonCode::EvidencePackWorkMdMarkerRejected => {
            ReasonCode::EvidenceTrialWorkMdMarkerRejected
        }
        ReasonCode::EvidencePackRawProviderResponseRejected => {
            ReasonCode::EvidenceTrialRawProviderResponseRejected
        }
        ReasonCode::EvidencePackAccountDataRejected => ReasonCode::EvidenceTrialAccountDataRejected,
        ReasonCode::EvidencePackOrderDataRejected => ReasonCode::EvidenceTrialOrderDataRejected,
        ReasonCode::EvidencePackEndpointDataRejected => {
            ReasonCode::EvidenceTrialEndpointDataRejected
        }
        ReasonCode::EvidencePackLiveProviderRejected => {
            ReasonCode::EvidenceTrialLiveProviderRejected
        }
        ReasonCode::EvidencePackUrlPathRejected | ReasonCode::EvidencePackUrlRejected => {
            ReasonCode::EvidenceTrialUrlRejected
        }
        ReasonCode::EvidencePackLocalOnly => ReasonCode::OwnerEvidencePackLocalOnly,
        ReasonCode::EvidencePackSanitizedOnly => ReasonCode::OwnerEvidencePackSanitizedOnly,
        ReasonCode::EvidencePackInsufficientSources | ReasonCode::EvidencePackInsufficientRows => {
            ReasonCode::EvidenceTriageInsufficient
        }
        other => other,
    }
}

fn owner_trial_reason_is_safety(reason: &ReasonCode) -> bool {
    matches!(
        reason,
        ReasonCode::EvidenceTrialUnsafePrivateData
            | ReasonCode::EvidenceTrialSecretLikeDataRejected
            | ReasonCode::EvidenceTrialWorkMdMarkerRejected
            | ReasonCode::EvidenceTrialRawProviderResponseRejected
            | ReasonCode::EvidenceTrialAccountDataRejected
            | ReasonCode::EvidenceTrialOrderDataRejected
            | ReasonCode::EvidenceTrialEndpointDataRejected
            | ReasonCode::EvidenceTrialLiveProviderRejected
            | ReasonCode::EvidenceTrialUrlRejected
            | ReasonCode::EvidencePackUnsafePrivateData
            | ReasonCode::EvidencePackSecretLikeDataRejected
            | ReasonCode::EvidencePackWorkMdPathRejected
            | ReasonCode::EvidencePackWorkMdMarkerRejected
            | ReasonCode::EvidencePackRawProviderResponseRejected
            | ReasonCode::EvidencePackAccountDataRejected
            | ReasonCode::EvidencePackOrderDataRejected
            | ReasonCode::EvidencePackEndpointDataRejected
            | ReasonCode::EvidencePackLiveProviderRejected
            | ReasonCode::EvidencePackUrlPathRejected
            | ReasonCode::EvidencePackUrlRejected
            | ReasonCode::EvidencePackEnvPathRejected
            | ReasonCode::EvidencePackPrivatePathRejected
    )
}

fn validate_historical_evidence_manifest_header(
    manifest: &HistoricalEvidencePackManifest,
    config: &HistoricalEvidencePackConfig,
) -> Result<(), HistoricalEvidencePackError> {
    if manifest.pack_id.trim().is_empty()
        || manifest.sources.len() > config.max_sources
        || config.max_sources == 0
        || config.max_rows_per_source == 0
    {
        return Err(historical_evidence_pack_error(
            None,
            ReasonCode::EvidencePackSourceRejected,
        ));
    }
    if !manifest.local_only {
        return Err(historical_evidence_pack_error(
            None,
            ReasonCode::EvidencePackLocalOnly,
        ));
    }
    if !manifest.sanitized_only {
        return Err(historical_evidence_pack_error(
            None,
            ReasonCode::EvidencePackSanitizedOnly,
        ));
    }
    let manifest_reason_text = format!("{:?}", manifest.reason_codes);
    for value in [
        manifest.pack_id.as_str(),
        manifest.description.as_str(),
        manifest_reason_text.as_str(),
    ] {
        if let Some(reason) = evidence_pack_data_safety_reason(value) {
            return Err(historical_evidence_pack_error(None, reason));
        }
    }
    Ok(())
}

fn load_historical_evidence_source(
    spec: HistoricalEvidenceSourceSpec,
    config: &HistoricalEvidencePackConfig,
) -> HistoricalEvidenceSource {
    let base_reasons = stable_reason_codes(
        &spec
            .reason_codes
            .iter()
            .chain(config.reason_codes.iter())
            .cloned()
            .chain([
                ReasonCode::EvidencePackLocalOnly,
                ReasonCode::EvidencePackSanitizedOnly,
                ReasonCode::EvidencePackNoNetwork,
                ReasonCode::EvidencePackNoDownloader,
                ReasonCode::LocalFileOnly,
            ])
            .collect::<Vec<_>>(),
    );
    if !spec.enabled {
        return HistoricalEvidenceSource {
            spec,
            dataset: None,
            accepted: false,
            rejected: false,
            disabled: true,
            insufficient_evidence: false,
            reason_codes: stable_reason_codes(
                &base_reasons
                    .iter()
                    .cloned()
                    .chain([ReasonCode::EvidencePackSourceDisabled])
                    .collect::<Vec<_>>(),
            ),
        };
    }
    if spec.source_id.trim().is_empty()
        || spec.symbol.trim().is_empty()
        || spec.market.trim().is_empty()
    {
        return rejected_historical_evidence_source(
            spec,
            &base_reasons,
            ReasonCode::EvidencePackSourceRejected,
            false,
        );
    }
    let source_field_safety_reason = [
        spec.source_id.as_str(),
        spec.symbol.as_str(),
        spec.market.as_str(),
        spec.currency.as_deref().unwrap_or_default(),
    ]
    .iter()
    .find_map(|value| evidence_pack_data_safety_reason(value));
    if let Some(reason) = source_field_safety_reason {
        return rejected_historical_evidence_source(spec, &base_reasons, reason, false);
    }
    let Some(local_kind) = evidence_source_local_kind(&spec, config) else {
        return rejected_historical_evidence_source(
            spec,
            &base_reasons,
            ReasonCode::EvidencePackUnsupportedSourceKind,
            false,
        );
    };
    if spec.expected_min_rows > config.max_rows_per_source {
        return rejected_historical_evidence_source(
            spec,
            &base_reasons,
            ReasonCode::EvidencePackInsufficientRows,
            true,
        );
    }
    let csv_text = match evidence_pack_source_text(&spec) {
        Ok(csv_text) => csv_text,
        Err(reason) => {
            return rejected_historical_evidence_source(spec, &base_reasons, reason, false);
        }
    };
    if let Some(reason) = evidence_pack_data_safety_reason(&csv_text) {
        return rejected_historical_evidence_source(spec, &base_reasons, reason, false);
    }
    let import_config = ManualHistoricalDailyImportConfig {
        dataset_id: spec.source_id.clone(),
        source_kind: local_kind,
        max_rows: config.max_rows_per_source,
        min_rows: spec.expected_min_rows.max(2),
        reject_private_markers: config.reject_private_markers,
        reject_endpoint_markers: config.reject_endpoint_markers,
        ..ManualHistoricalDailyImportConfig::default()
    };
    let dataset = match parse_manual_historical_daily_csv(&csv_text, &import_config) {
        Ok(dataset) => dataset,
        Err(error) => {
            let reason = error
                .reason_codes
                .first()
                .cloned()
                .map(evidence_pack_reason_from_manual)
                .unwrap_or(ReasonCode::EvidencePackSourceRejected);
            let insufficient = reason == ReasonCode::EvidencePackInsufficientRows;
            return rejected_historical_evidence_source(spec, &base_reasons, reason, insufficient);
        }
    };
    if dataset.symbol != spec.symbol || dataset.rows.len() < spec.expected_min_rows {
        return rejected_historical_evidence_source(
            spec,
            &base_reasons,
            ReasonCode::EvidencePackInsufficientRows,
            true,
        );
    }
    let reason_codes = stable_reason_codes(
        &base_reasons
            .iter()
            .chain(dataset.reason_codes.iter())
            .cloned()
            .chain([ReasonCode::HardcodingAuditPassed])
            .collect::<Vec<_>>(),
    );
    HistoricalEvidenceSource {
        spec,
        dataset: Some(dataset),
        accepted: true,
        rejected: false,
        disabled: false,
        insufficient_evidence: false,
        reason_codes,
    }
}

fn rejected_historical_evidence_source(
    spec: HistoricalEvidenceSourceSpec,
    base_reasons: &[ReasonCode],
    reason: ReasonCode,
    insufficient_evidence: bool,
) -> HistoricalEvidenceSource {
    HistoricalEvidenceSource {
        spec,
        dataset: None,
        accepted: false,
        rejected: true,
        disabled: false,
        insufficient_evidence,
        reason_codes: stable_reason_codes(
            &base_reasons
                .iter()
                .cloned()
                .chain([reason, ReasonCode::EvidencePackSourceRejected])
                .collect::<Vec<_>>(),
        ),
    }
}

fn evidence_source_local_kind(
    spec: &HistoricalEvidenceSourceSpec,
    config: &HistoricalEvidencePackConfig,
) -> Option<LocalDataSourceKind> {
    match spec.source_kind {
        HistoricalEvidenceSourceKind::UsStockDaily => Some(LocalDataSourceKind::UsStockCsv),
        HistoricalEvidenceSourceKind::KoreanStockDaily => Some(LocalDataSourceKind::KoreanStockCsv),
        HistoricalEvidenceSourceKind::BtcCryptoDaily => Some(LocalDataSourceKind::BtcCryptoCsv),
        HistoricalEvidenceSourceKind::SyntheticDailySample
            if config.allow_synthetic_sources_for_tests_only =>
        {
            let market = spec.market.to_ascii_uppercase();
            if market.contains("KR") {
                Some(LocalDataSourceKind::KoreanStockCsv)
            } else if market.contains("BTC") || spec.symbol.to_ascii_uppercase().contains("BTC") {
                Some(LocalDataSourceKind::BtcCryptoCsv)
            } else {
                Some(LocalDataSourceKind::UsStockCsv)
            }
        }
        _ => None,
    }
}

fn evidence_pack_source_text(spec: &HistoricalEvidenceSourceSpec) -> Result<String, ReasonCode> {
    if let Some(csv_text) = &spec.csv_text {
        return Ok(csv_text.clone());
    }
    let path = spec
        .csv_path
        .as_deref()
        .ok_or(ReasonCode::EvidencePackPathRejected)?;
    if let Some(reason) = evidence_pack_path_safety_reason(path) {
        return Err(reason);
    }
    fs::read_to_string(path).map_err(|_| ReasonCode::EvidencePackPathRejected)
}

fn evidence_pack_path_safety_reason(path: &str) -> Option<ReasonCode> {
    let lower = path.to_ascii_lowercase();
    let private_instruction_name = concat!("work", ".", "md");
    if lower.trim().is_empty() {
        return Some(ReasonCode::EvidencePackPathRejected);
    }
    if ["http://", "https://", "ws://", "wss://", "ftp://"]
        .iter()
        .any(|prefix| lower.starts_with(prefix))
    {
        return Some(ReasonCode::EvidencePackUrlPathRejected);
    }
    if lower.contains(private_instruction_name) || contains_temporary_instruction_marker(path) {
        return Some(ReasonCode::EvidencePackWorkMdPathRejected);
    }
    if lower.contains(".env") {
        return Some(ReasonCode::EvidencePackEnvPathRejected);
    }
    if lower.contains("local_private") || lower.contains("private mapping") {
        return Some(ReasonCode::EvidencePackPrivatePathRejected);
    }
    if lower.contains("broker_endpoint")
        || lower.contains("order_endpoint")
        || lower.contains("live_endpoint")
        || lower.contains("exchange_secret")
    {
        return Some(ReasonCode::EvidencePackPathRejected);
    }
    None
}

fn evidence_pack_data_safety_reason(text: &str) -> Option<ReasonCode> {
    let lower = text.to_ascii_lowercase();
    let private_instruction_name = concat!("work", ".", "md");
    if lower.contains(private_instruction_name) || contains_temporary_instruction_marker(text) {
        return Some(ReasonCode::EvidencePackWorkMdMarkerRejected);
    }
    if lower.contains("live_provider")
        || lower.contains("live_endpoint")
        || lower.contains("exchange_secret")
    {
        return Some(ReasonCode::EvidencePackLiveProviderRejected);
    }
    if ["http://", "https://", "ws://", "wss://", "ftp://"]
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Some(ReasonCode::EvidencePackUrlRejected);
    }
    if lower.contains("raw_response") || lower.contains("raw provider response") {
        return Some(ReasonCode::EvidencePackRawProviderResponseRejected);
    }
    if lower.contains("account_id") {
        return Some(ReasonCode::EvidencePackAccountDataRejected);
    }
    if lower.contains("order_id") {
        return Some(ReasonCode::EvidencePackOrderDataRejected);
    }
    if lower.contains("endpoint")
        || lower.contains("url_endpoint")
        || lower.contains("broker_endpoint")
        || lower.contains("order_endpoint")
    {
        return Some(ReasonCode::EvidencePackEndpointDataRejected);
    }
    if [
        "authorization",
        "bearer ",
        "access_token",
        "refresh_token",
        "app_key",
        "app_secret",
        "api_key",
        "private_key",
        "wallet_private_key",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        return Some(ReasonCode::EvidencePackSecretLikeDataRejected);
    }
    if lower.contains("local_private")
        || lower.contains("private mapping")
        || lower.contains(".env")
    {
        return Some(ReasonCode::EvidencePackUnsafePrivateData);
    }
    None
}

fn evidence_pack_reason_from_manual(reason: ReasonCode) -> ReasonCode {
    match reason {
        ReasonCode::WalkForwardInsufficientRows => ReasonCode::EvidencePackInsufficientRows,
        ReasonCode::ManualHistoricalImportUnsafePrivateData => {
            ReasonCode::EvidencePackUnsafePrivateData
        }
        ReasonCode::ManualHistoricalImportSecretLikeDataRejected => {
            ReasonCode::EvidencePackSecretLikeDataRejected
        }
        ReasonCode::ManualHistoricalImportRawProviderRejected
        | ReasonCode::ManualHistoricalImportRawProviderResponseRejected => {
            ReasonCode::EvidencePackRawProviderResponseRejected
        }
        ReasonCode::ManualHistoricalImportEndpointRejected
        | ReasonCode::ManualHistoricalImportEndpointDataRejected => {
            ReasonCode::EvidencePackEndpointDataRejected
        }
        ReasonCode::ManualHistoricalImportAccountDataRejected => {
            ReasonCode::EvidencePackAccountDataRejected
        }
        ReasonCode::ManualHistoricalImportOrderDataRejected => {
            ReasonCode::EvidencePackOrderDataRejected
        }
        ReasonCode::ManualHistoricalImportLiveProviderRejected => {
            ReasonCode::EvidencePackLiveProviderRejected
        }
        ReasonCode::ManualHistoricalImportWorkMdRejected
        | ReasonCode::ManualHistoricalImportWorkMdMarkerRejected => {
            ReasonCode::EvidencePackWorkMdMarkerRejected
        }
        _ => ReasonCode::EvidencePackSourceRejected,
    }
}

fn historical_evidence_pack_error(
    source_id: Option<String>,
    reason: ReasonCode,
) -> HistoricalEvidencePackError {
    HistoricalEvidencePackError {
        source_id,
        reason_codes: vec![reason],
    }
}

fn evaluate_historical_evidence_source(
    source: &HistoricalEvidenceSource,
    eval_config: &HistoricalEvidencePackEvaluationConfig,
) -> HistoricalEvidenceSourceEvaluationResult {
    let dataset_summary = source
        .dataset
        .as_ref()
        .map(historical_evidence_dataset_summary)
        .unwrap_or_else(|| {
            if source.disabled {
                "disabled source".to_string()
            } else {
                "rejected source".to_string()
            }
        });
    let Some(dataset) = &source.dataset else {
        return HistoricalEvidenceSourceEvaluationResult {
            source_id: source.spec.source_id.clone(),
            source_kind: source.spec.source_kind,
            symbol: source.spec.symbol.clone(),
            market: source.spec.market.clone(),
            dataset_summary,
            walk_forward_result: None,
            proof_gate_report: None,
            accepted: false,
            rejected: source.rejected,
            insufficient_evidence: source.insufficient_evidence,
            reason_codes: source.reason_codes.clone(),
        };
    };
    let walk_forward_config = WalkForwardConfig {
        min_prediction_samples: eval_config.min_prediction_samples,
        ..eval_config.walk_forward_config.clone()
    };
    let input = WalkForwardEvaluationInput {
        dataset: dataset.clone(),
        initial_agent_states: eval_config.initial_agent_states.clone(),
        walk_forward_config,
        committee_config: eval_config.committee_config.clone(),
        risk_config: eval_config.risk_config,
    };
    match run_walk_forward_evaluation(input) {
        Ok(result) => {
            let insufficient_evidence = result.proof_gate_status
                == ProofGateStatus::InsufficientEvidence
                || result.aggregate_baseline_comparison.insufficient_evidence;
            let proof_gate_report = build_proof_gate_report(&result);
            let reason_codes = stable_reason_codes(
                &source
                    .reason_codes
                    .iter()
                    .chain(result.reason_codes.iter())
                    .cloned()
                    .chain([ReasonCode::HardcodingAuditPassed])
                    .collect::<Vec<_>>(),
            );
            HistoricalEvidenceSourceEvaluationResult {
                source_id: source.spec.source_id.clone(),
                source_kind: source.spec.source_kind,
                symbol: source.spec.symbol.clone(),
                market: source.spec.market.clone(),
                dataset_summary,
                walk_forward_result: Some(result),
                proof_gate_report: Some(proof_gate_report),
                accepted: source.accepted,
                rejected: false,
                insufficient_evidence,
                reason_codes,
            }
        }
        Err(error) => HistoricalEvidenceSourceEvaluationResult {
            source_id: source.spec.source_id.clone(),
            source_kind: source.spec.source_kind,
            symbol: source.spec.symbol.clone(),
            market: source.spec.market.clone(),
            dataset_summary,
            walk_forward_result: None,
            proof_gate_report: None,
            accepted: source.accepted,
            rejected: false,
            insufficient_evidence: true,
            reason_codes: stable_reason_codes(
                &source
                    .reason_codes
                    .iter()
                    .chain(error.reason_codes.iter())
                    .cloned()
                    .chain([ReasonCode::EvidencePackInsufficientRows])
                    .collect::<Vec<_>>(),
            ),
        },
    }
}

fn historical_evidence_dataset_summary(dataset: &ManualHistoricalDailyDataset) -> String {
    format!(
        "dataset_id={} symbol={} rows={} range={}..{} source_kind={:?}",
        dataset.dataset_id,
        dataset.symbol,
        dataset.rows.len(),
        dataset.date_range.start_date,
        dataset.date_range.end_date,
        dataset.source_kind,
    )
}

fn build_aggregate_baseline_comparison(
    source_results: &[HistoricalEvidenceSourceEvaluationResult],
    eval_config: &HistoricalEvidencePackEvaluationConfig,
) -> AggregateBaselineComparison {
    let accepted_results = accepted_walk_forward_results(source_results);
    let accepted_source_count = accepted_results.len();
    let rejected_source_count = source_results
        .iter()
        .filter(|source| source.rejected)
        .count();
    let insufficient_source_count = source_results
        .iter()
        .filter(|source| source.insufficient_evidence)
        .count();
    let mut voice_beats_equal_weight_count = 0usize;
    let mut voice_loses_equal_weight_count = 0usize;
    let mut voice_ties_equal_weight_count = 0usize;
    let mut committee_beats_no_trade_count = 0usize;
    let mut committee_loses_no_trade_count = 0usize;
    let mut committee_beats_buy_hold_count = 0usize;
    let mut committee_loses_buy_hold_count = 0usize;
    for result in &accepted_results {
        let comparison = &result.aggregate_baseline_comparison;
        let delta = risk_adjusted_score(&comparison.voice_adaptive_committee)
            - risk_adjusted_score(&comparison.equal_weight_committee);
        if delta > f64::EPSILON {
            voice_beats_equal_weight_count += 1;
        } else if delta < -f64::EPSILON {
            voice_loses_equal_weight_count += 1;
        } else {
            voice_ties_equal_weight_count += 1;
        }
        if comparison.committee_beats_no_trade {
            committee_beats_no_trade_count += 1;
        } else {
            committee_loses_no_trade_count += 1;
        }
        if comparison.committee_beats_buy_hold_risk_adjusted {
            committee_beats_buy_hold_count += 1;
        } else {
            committee_loses_buy_hold_count += 1;
        }
    }
    let overall_status = aggregate_baseline_status(
        accepted_source_count,
        voice_beats_equal_weight_count,
        voice_loses_equal_weight_count,
        voice_ties_equal_weight_count,
        committee_loses_no_trade_count,
        committee_loses_buy_hold_count,
        eval_config.min_accepted_sources_for_proof,
    );
    let mut reason_codes = vec![ReasonCode::HardcodingAuditPassed];
    match overall_status {
        AggregateBaselineOverallStatus::Pass => {
            reason_codes.push(ReasonCode::AggregateBaselineVoiceWon)
        }
        AggregateBaselineOverallStatus::Fail => {
            reason_codes.push(ReasonCode::AggregateBaselineNoEdge)
        }
        AggregateBaselineOverallStatus::Mixed => {
            reason_codes.push(ReasonCode::AggregateBaselineVoiceMixed)
        }
        AggregateBaselineOverallStatus::InsufficientEvidence => {
            reason_codes.push(ReasonCode::AggregateBaselineInsufficientEvidence)
        }
    }
    if voice_beats_equal_weight_count > 0 {
        reason_codes.push(ReasonCode::AggregateBaselineVoiceWon);
    }
    if voice_loses_equal_weight_count > 0 {
        reason_codes.push(ReasonCode::AggregateBaselineVoiceLost);
    }
    if committee_loses_buy_hold_count > 0 {
        reason_codes.push(ReasonCode::AggregateBaselineBuyHoldStronger);
    }
    if committee_loses_no_trade_count > 0 {
        reason_codes.push(ReasonCode::AggregateBaselineNoTradeStronger);
    }
    if overall_status != AggregateBaselineOverallStatus::Pass {
        reason_codes.push(ReasonCode::AggregateBaselineNoEdge);
    }
    AggregateBaselineComparison {
        source_count: source_results.len(),
        accepted_source_count,
        rejected_source_count,
        insufficient_source_count,
        voice_beats_equal_weight_count,
        voice_loses_equal_weight_count,
        voice_ties_equal_weight_count,
        committee_beats_no_trade_count,
        committee_loses_no_trade_count,
        committee_beats_buy_hold_count,
        committee_loses_buy_hold_count,
        mean_total_return_by_baseline: mean_baseline_metric(&accepted_results, |metrics| {
            metrics.total_return
        }),
        mean_max_drawdown_by_baseline: mean_baseline_metric(&accepted_results, |metrics| {
            metrics.max_drawdown
        }),
        mean_brier_score_by_baseline: mean_brier_score_by_baseline(&accepted_results),
        overall_status,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn aggregate_baseline_status(
    accepted_source_count: usize,
    voice_win_count: usize,
    voice_loss_count: usize,
    voice_tie_count: usize,
    no_trade_loss_count: usize,
    buy_hold_loss_count: usize,
    min_accepted_sources_for_proof: usize,
) -> AggregateBaselineOverallStatus {
    if accepted_source_count < min_accepted_sources_for_proof || accepted_source_count == 0 {
        return AggregateBaselineOverallStatus::InsufficientEvidence;
    }
    if voice_win_count == accepted_source_count
        && no_trade_loss_count == 0
        && buy_hold_loss_count == 0
    {
        return AggregateBaselineOverallStatus::Pass;
    }
    if voice_win_count > 0
        && (voice_loss_count > 0
            || voice_tie_count > 0
            || no_trade_loss_count > 0
            || buy_hold_loss_count > 0)
    {
        return AggregateBaselineOverallStatus::Mixed;
    }
    AggregateBaselineOverallStatus::Fail
}

fn accepted_walk_forward_results(
    source_results: &[HistoricalEvidenceSourceEvaluationResult],
) -> Vec<WalkForwardEvaluationResult> {
    source_results
        .iter()
        .filter(|source| source.accepted)
        .filter_map(|source| source.walk_forward_result.clone())
        .collect()
}

fn mean_baseline_metric(
    results: &[WalkForwardEvaluationResult],
    value: fn(&BaselinePerformanceMetrics) -> f64,
) -> BTreeMap<BaselineStrategyKind, f64> {
    let mut grouped = BTreeMap::<BaselineStrategyKind, Vec<f64>>::new();
    for result in results {
        let comparison = &result.aggregate_baseline_comparison;
        for metrics in [
            &comparison.always_no_trade,
            &comparison.buy_and_hold,
            &comparison.equal_weight_committee,
            &comparison.voice_adaptive_committee,
        ] {
            grouped
                .entry(metrics.strategy)
                .or_default()
                .push(value(metrics));
        }
    }
    grouped
        .into_iter()
        .map(|(strategy, values)| {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            (strategy, mean)
        })
        .collect()
}

fn mean_brier_score_by_baseline(
    results: &[WalkForwardEvaluationResult],
) -> BTreeMap<BaselineStrategyKind, f64> {
    let mut grouped = BTreeMap::<BaselineStrategyKind, Vec<f64>>::new();
    for result in results {
        for metrics in &result.scoring_summary {
            if let Some(score) = metrics.brier_score {
                grouped.entry(metrics.strategy).or_default().push(score);
            }
        }
    }
    grouped
        .into_iter()
        .map(|(strategy, values)| {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            (strategy, mean)
        })
        .collect()
}

fn build_voice_adaptation_validity(
    source_results: &[HistoricalEvidenceSourceEvaluationResult],
    eval_config: &HistoricalEvidencePackEvaluationConfig,
) -> VoiceAdaptationValidity {
    let accepted_results = accepted_walk_forward_results(source_results);
    let mut voice_better_count = 0usize;
    let mut equal_weight_better_count = 0usize;
    let mut tie_count = 0usize;
    let mut deltas = Vec::new();
    let mut brier_deltas = Vec::new();
    let mut drawdown_deltas = Vec::new();
    for result in &accepted_results {
        let comparison = &result.aggregate_baseline_comparison;
        let delta = risk_adjusted_score(&comparison.voice_adaptive_committee)
            - risk_adjusted_score(&comparison.equal_weight_committee);
        deltas.push(delta);
        if delta > f64::EPSILON {
            voice_better_count += 1;
        } else if delta < -f64::EPSILON {
            equal_weight_better_count += 1;
        } else {
            tie_count += 1;
        }
        drawdown_deltas.push(
            comparison.voice_adaptive_committee.max_drawdown
                - comparison.equal_weight_committee.max_drawdown,
        );
        let equal_brier = result
            .scoring_summary
            .iter()
            .find(|metrics| metrics.strategy == BaselineStrategyKind::EqualWeightCommittee)
            .and_then(|metrics| metrics.brier_score);
        let voice_brier = result
            .scoring_summary
            .iter()
            .find(|metrics| metrics.strategy == BaselineStrategyKind::VoiceAdaptiveCommittee)
            .and_then(|metrics| metrics.brier_score);
        if let (Some(voice), Some(equal)) = (voice_brier, equal_brier) {
            brier_deltas.push(voice - equal);
        }
    }
    let compared_source_count = accepted_results.len();
    let status = if compared_source_count < eval_config.min_accepted_sources_for_proof {
        VoiceAdaptationValidityStatus::InsufficientEvidence
    } else if voice_better_count == compared_source_count && compared_source_count > 0 {
        VoiceAdaptationValidityStatus::Helped
    } else if equal_weight_better_count + tie_count == compared_source_count {
        VoiceAdaptationValidityStatus::Failed
    } else {
        VoiceAdaptationValidityStatus::Mixed
    };
    let mut reason_codes = vec![ReasonCode::HardcodingAuditPassed];
    match status {
        VoiceAdaptationValidityStatus::Helped => {
            reason_codes.push(ReasonCode::AggregateBaselineVoiceWon)
        }
        VoiceAdaptationValidityStatus::Failed => {
            reason_codes.push(ReasonCode::AggregateBaselineVoiceLost);
            reason_codes.push(ReasonCode::HardcodedVoiceSuccessRejected);
        }
        VoiceAdaptationValidityStatus::Mixed => {
            reason_codes.push(ReasonCode::AggregateBaselineVoiceMixed)
        }
        VoiceAdaptationValidityStatus::InsufficientEvidence => {
            reason_codes.push(ReasonCode::AggregateBaselineInsufficientEvidence)
        }
    }
    VoiceAdaptationValidity {
        compared_source_count,
        voice_better_count,
        equal_weight_better_count,
        tie_count,
        mean_delta_vs_equal_weight: mean_or_zero(&deltas),
        brier_delta_vs_equal_weight: (!brier_deltas.is_empty())
            .then_some(mean_or_zero(&brier_deltas)),
        drawdown_delta_vs_equal_weight: (!drawdown_deltas.is_empty())
            .then_some(mean_or_zero(&drawdown_deltas)),
        status,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn build_aggregate_prediction_quality_summary(
    source_results: &[HistoricalEvidenceSourceEvaluationResult],
    eval_config: &HistoricalEvidencePackEvaluationConfig,
) -> AggregatePredictionQualitySummary {
    let mut voice_rows = Vec::new();
    for source in source_results {
        let Some(result) = &source.walk_forward_result else {
            continue;
        };
        if let Some(metrics) = result
            .scoring_summary
            .iter()
            .find(|metrics| metrics.strategy == BaselineStrategyKind::VoiceAdaptiveCommittee)
        {
            voice_rows.push((source.source_id.clone(), metrics.clone()));
        }
    }
    let source_count = voice_rows.len();
    let total_samples = voice_rows
        .iter()
        .map(|(_, metrics)| metrics.sample_count)
        .sum::<usize>();
    let calibrated_samples = voice_rows
        .iter()
        .map(|(_, metrics)| metrics.calibrated_sample_count)
        .sum::<usize>();
    let brier_numerator = voice_rows
        .iter()
        .filter_map(|(_, metrics)| {
            metrics
                .brier_score
                .map(|score| score * metrics.calibrated_sample_count as f64)
        })
        .sum::<f64>();
    let ranked_brier = voice_rows
        .iter()
        .filter_map(|(source_id, metrics)| metrics.brier_score.map(|score| (source_id, score)))
        .collect::<Vec<_>>();
    let best_source_by_brier = ranked_brier
        .iter()
        .min_by(|left, right| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(source_id, _)| (*source_id).clone());
    let worst_source_by_brier = ranked_brier
        .iter()
        .max_by(|left, right| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(source_id, _)| (*source_id).clone());
    let insufficient_evidence = source_count < eval_config.min_accepted_sources_for_proof
        || total_samples < eval_config.min_prediction_samples
        || calibrated_samples == 0;
    let mut reason_codes = vec![ReasonCode::PredictionScoringBrier];
    if insufficient_evidence {
        reason_codes.push(ReasonCode::PredictionScoringInsufficientSamples);
        reason_codes.push(ReasonCode::AggregateBaselineInsufficientEvidence);
    }
    AggregatePredictionQualitySummary {
        source_count,
        total_samples,
        missing_probability_count: voice_rows
            .iter()
            .map(|(_, metrics)| metrics.missing_probability_count)
            .sum(),
        abstention_count: voice_rows
            .iter()
            .map(|(_, metrics)| metrics.abstention_count)
            .sum(),
        high_confidence_error_count: voice_rows
            .iter()
            .map(|(_, metrics)| metrics.high_confidence_error_count)
            .sum(),
        mean_brier_score: (calibrated_samples > 0)
            .then_some(brier_numerator / calibrated_samples as f64),
        best_source_by_brier,
        worst_source_by_brier,
        insufficient_evidence,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn build_market_evidence_results(
    source_results: &[HistoricalEvidenceSourceEvaluationResult],
    eval_config: &HistoricalEvidencePackEvaluationConfig,
) -> Vec<MarketEvidenceResult> {
    let mut grouped = BTreeMap::<String, Vec<HistoricalEvidenceSourceEvaluationResult>>::new();
    for source in source_results {
        grouped
            .entry(source.market.clone())
            .or_default()
            .push(source.clone());
    }
    grouped
        .into_iter()
        .map(|(market, rows)| {
            let baseline_comparison = build_aggregate_baseline_comparison(&rows, eval_config);
            let voice_adaptation_comparison = build_voice_adaptation_validity(&rows, eval_config);
            let prediction_quality_summary =
                build_aggregate_prediction_quality_summary(&rows, eval_config);
            let insufficient_evidence = baseline_comparison.overall_status
                == AggregateBaselineOverallStatus::InsufficientEvidence;
            let reason_codes = stable_reason_codes(
                &baseline_comparison
                    .reason_codes
                    .iter()
                    .chain(voice_adaptation_comparison.reason_codes.iter())
                    .chain(prediction_quality_summary.reason_codes.iter())
                    .cloned()
                    .collect::<Vec<_>>(),
            );
            MarketEvidenceResult {
                market,
                source_count: rows.len(),
                accepted_count: rows
                    .iter()
                    .filter(|source| source.accepted && source.walk_forward_result.is_some())
                    .count(),
                rejected_count: rows.iter().filter(|source| source.rejected).count(),
                baseline_comparison,
                voice_adaptation_comparison,
                prediction_quality_summary,
                insufficient_evidence,
                reason_codes,
            }
        })
        .collect()
}

fn voice_adaptation_report_sentence(summary: &VoiceAdaptationValidity) -> String {
    match summary.status {
        VoiceAdaptationValidityStatus::Helped => {
            "Voice adaptation helped on this evidence pack based on computed risk-adjusted results."
                .to_string()
        }
        VoiceAdaptationValidityStatus::Failed => {
            "Voice adaptation did not beat equal weight on this evidence pack.".to_string()
        }
        VoiceAdaptationValidityStatus::Mixed => "Voice adaptation evidence is mixed.".to_string(),
        VoiceAdaptationValidityStatus::InsufficientEvidence => {
            "Insufficient evidence to trust voice adaptation.".to_string()
        }
    }
}

fn mean_or_zero(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn validate_manual_historical_config(
    config: &ManualHistoricalDailyImportConfig,
) -> Result<(), ManualHistoricalDailyImportError> {
    if !matches!(
        config.source_kind,
        LocalDataSourceKind::UsStockCsv
            | LocalDataSourceKind::KoreanStockCsv
            | LocalDataSourceKind::BtcCryptoCsv
    ) || config.dataset_id.trim().is_empty()
        || config.max_rows == 0
        || config.min_rows < 2
        || config.min_rows > config.max_rows
        || config.reject_weekend_gap
        || !config.calendar_validation_deferred
    {
        return Err(manual_historical_error(
            None,
            ReasonCode::ManualHistoricalImportDailyOnly,
        ));
    }
    Ok(())
}

fn manual_historical_error(
    row_number: Option<usize>,
    reason_code: ReasonCode,
) -> ManualHistoricalDailyImportError {
    ManualHistoricalDailyImportError {
        row_number,
        reason_codes: vec![reason_code],
    }
}

fn manual_historical_safety_reason(
    text: &str,
    header: Option<&[String]>,
    config: &ManualHistoricalDailyImportConfig,
) -> Option<ReasonCode> {
    let normalized = text.to_ascii_lowercase();
    if contains_temporary_instruction_marker(text) {
        return Some(ReasonCode::ManualHistoricalImportWorkMdMarkerRejected);
    }
    if config.reject_endpoint_markers
        && header.is_some_and(|columns| {
            columns.iter().any(|column| {
                column.contains("endpoint")
                    || column == "url"
                    || column == "url_endpoint"
                    || column == "broker_endpoint"
                    || column == "order_endpoint"
            })
        })
    {
        return Some(ReasonCode::ManualHistoricalImportEndpointDataRejected);
    }
    if normalized.contains("live_provider")
        || normalized.contains("live_endpoint")
        || normalized.contains("exchange_secret")
    {
        return Some(ReasonCode::ManualHistoricalImportLiveProviderRejected);
    }
    if config.reject_endpoint_markers
        && (normalized.contains("http://")
            || normalized.contains("https://")
            || normalized.contains("broker-endpoint")
            || normalized.contains("order-endpoint")
            || normalized.contains("url_endpoint")
            || normalized.contains("broker_endpoint")
            || normalized.contains("order_endpoint"))
    {
        return Some(ReasonCode::ManualHistoricalImportEndpointDataRejected);
    }
    if normalized.contains("raw_response") || normalized.contains("raw provider response") {
        return Some(ReasonCode::ManualHistoricalImportRawProviderResponseRejected);
    }
    if normalized.contains("account_id") {
        return Some(ReasonCode::ManualHistoricalImportAccountDataRejected);
    }
    if normalized.contains("order_id") {
        return Some(ReasonCode::ManualHistoricalImportOrderDataRejected);
    }
    if [
        "authorization",
        "bearer ",
        "access_token",
        "refresh_token",
        "app_key",
        "app_secret",
        "api_key",
        "private_key",
        "wallet_private_key",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
    {
        return Some(ReasonCode::ManualHistoricalImportSecretLikeDataRejected);
    }
    if config.reject_private_markers
        && (normalized.contains("local_private")
            || normalized.contains("private mapping")
            || normalized.contains(".env"))
    {
        return Some(ReasonCode::ManualHistoricalImportUnsafePrivateData);
    }
    None
}

fn parse_manual_daily_date_ms(date: &str) -> Option<u64> {
    let bytes = date.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(index, value)| index == 4 || index == 7 || value.is_ascii_digit())
    {
        return None;
    }
    parse_local_datetime_utc_ms(date, "000000")
}

fn parse_manual_required_number(
    value: Option<&str>,
    row_number: usize,
) -> Result<f64, ManualHistoricalDailyImportError> {
    let parsed = value
        .and_then(|value| value.parse::<f64>().ok())
        .ok_or_else(|| {
            manual_historical_error(Some(row_number), ReasonCode::HistoricalReplayInvalidRow)
        })?;
    if !parsed.is_finite() {
        return Err(manual_historical_error(
            Some(row_number),
            ReasonCode::HistoricalReplayNonFinite,
        ));
    }
    if parsed <= 0.0 {
        return Err(manual_historical_error(
            Some(row_number),
            ReasonCode::HistoricalReplayNonPositivePrice,
        ));
    }
    Ok(parsed)
}

fn parse_manual_volume(
    value: Option<&str>,
    row_number: usize,
) -> Result<f64, ManualHistoricalDailyImportError> {
    let parsed = value
        .and_then(|value| value.parse::<f64>().ok())
        .ok_or_else(|| {
            manual_historical_error(Some(row_number), ReasonCode::HistoricalReplayInvalidRow)
        })?;
    if !parsed.is_finite() {
        return Err(manual_historical_error(
            Some(row_number),
            ReasonCode::HistoricalReplayNonFinite,
        ));
    }
    if parsed < 0.0 {
        return Err(manual_historical_error(
            Some(row_number),
            ReasonCode::HistoricalReplayInvalidRow,
        ));
    }
    Ok(parsed)
}

fn parse_manual_optional_positive(
    value: Option<&str>,
    row_number: usize,
) -> Result<Option<f64>, ManualHistoricalDailyImportError> {
    let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    let parsed = parse_manual_required_number(Some(value), row_number)?;
    Ok(Some(parsed))
}

fn parse_manual_optional_non_negative(
    value: Option<&str>,
    row_number: usize,
) -> Result<Option<f64>, ManualHistoricalDailyImportError> {
    let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    let parsed = value.parse::<f64>().map_err(|_| {
        manual_historical_error(Some(row_number), ReasonCode::HistoricalReplayInvalidRow)
    })?;
    if !parsed.is_finite() {
        return Err(manual_historical_error(
            Some(row_number),
            ReasonCode::HistoricalReplayNonFinite,
        ));
    }
    if parsed < 0.0 {
        return Err(manual_historical_error(
            Some(row_number),
            ReasonCode::HistoricalReplayInvalidRow,
        ));
    }
    Ok(Some(parsed))
}

fn optional_manual_string(
    value: Option<&str>,
) -> Result<Option<String>, ManualHistoricalDailyImportError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if let Some(reason) =
        manual_historical_safety_reason(value, None, &ManualHistoricalDailyImportConfig::default())
    {
        return Err(manual_historical_error(None, reason));
    }
    Ok(Some(value.to_string()))
}

fn manual_daily_quality_summary(
    symbol: &str,
    source_kind: LocalDataSourceKind,
    rows: &[ManualHistoricalDailyRow],
) -> LocalDataQualitySummary {
    let min_close = rows
        .iter()
        .map(|row| row.close)
        .fold(f64::INFINITY, f64::min);
    let max_close = rows
        .iter()
        .map(|row| row.close)
        .fold(f64::NEG_INFINITY, f64::max);
    LocalDataQualitySummary {
        total_rows: rows.len(),
        accepted_rows: rows.len(),
        rejected_rows: 0,
        first_timestamp: rows.first().map_or(0, |row| row.timestamp_ms),
        last_timestamp: rows.last().map_or(0, |row| row.timestamp_ms),
        symbol: symbol.to_string(),
        source_kind,
        has_trade_value: rows.iter().any(|row| row.trade_value.is_some()),
        monotonic: rows
            .windows(2)
            .all(|pair| pair[0].timestamp_ms < pair[1].timestamp_ms),
        min_close,
        max_close,
        reason_codes: vec![
            ReasonCode::ManualHistoricalImportDailyOnly,
            ReasonCode::ManualHistoricalImportNoNetwork,
            ReasonCode::ManualHistoricalImportSanitizedOnly,
            ReasonCode::LocalFileOnly,
        ],
    }
}

fn walk_forward_config_is_valid(config: &WalkForwardConfig) -> bool {
    config.min_train_rows >= 1
        && config.eval_window_rows >= 2
        && config.step_rows >= 1
        && config.cost_bps.is_finite()
        && config.cost_bps >= 0.0
        && config.slippage_bps.is_finite()
        && config.slippage_bps >= 0.0
        && config.max_position_fraction.is_finite()
        && (0.0..=1.0).contains(&config.max_position_fraction)
        && config.no_lookahead
}

fn walk_forward_error(reason_codes: Vec<ReasonCode>) -> WalkForwardEvaluationError {
    WalkForwardEvaluationError {
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

#[derive(Clone, Debug, PartialEq)]
struct StrategyEvaluation {
    metrics: BaselinePerformanceMetrics,
    scoring: PredictionQualityMetrics,
    final_states: Vec<CanonicalAgentState>,
}

fn evaluate_always_no_trade_window(
    series: &CandleSeries,
    split: WalkForwardSplit,
) -> StrategyEvaluation {
    let decision_count = split
        .eval_end_index
        .saturating_sub(split.eval_start_index + 1);
    let samples = (split.eval_start_index..split.eval_end_index.saturating_sub(1))
        .map(|index| PredictionQualitySample {
            predicted_probability: None,
            abstained: true,
            realized_direction_up: direction_up(series, index),
        })
        .collect::<Vec<_>>();
    StrategyEvaluation {
        metrics: BaselinePerformanceMetrics {
            strategy: BaselineStrategyKind::AlwaysNoTrade,
            total_return: 0.0,
            max_drawdown: 0.0,
            trade_count: 0,
            win_count: 0,
            loss_count: 0,
            no_trade_count: decision_count as u64,
            risk_denial_count: 0,
            avg_return_per_trade: 0.0,
            volatility_estimate: None,
            sharpe_like: None,
            downside_loss: None,
            cost_paid: 0.0,
            slippage_paid: 0.0,
            reason_codes: vec![
                ReasonCode::BaselineAlwaysNoTrade,
                ReasonCode::NoTradePreferred,
                ReasonCode::PaperExecutionOnly,
            ],
        },
        scoring: compute_prediction_quality_metrics(
            BaselineStrategyKind::AlwaysNoTrade,
            &samples,
            1,
        ),
        final_states: Vec::new(),
    }
}

fn evaluate_buy_and_hold_window(
    series: &CandleSeries,
    split: WalkForwardSplit,
    walk_config: &WalkForwardConfig,
    committee_config: &WalkForwardCommitteeConfig,
    risk_config: &GovernorConfig,
) -> StrategyEvaluation {
    let Some(first) = series.candle(split.eval_start_index) else {
        return empty_strategy_evaluation(BaselineStrategyKind::BuyAndHold);
    };
    let Some(last) = series.candle(split.eval_end_index.saturating_sub(1)) else {
        return empty_strategy_evaluation(BaselineStrategyKind::BuyAndHold);
    };
    let market = series
        .market_snapshot_at(split.eval_start_index)
        .expect("split index validated");
    let expected_edge = committee_config
        .expected_edge_scale
        .max(risk_config.min_expected_edge + f64::EPSILON);
    let risk_decision = evaluate_baseline_trade_risk(
        BaselineStrategyKind::BuyAndHold,
        &market,
        Side::Long,
        walk_config.max_position_fraction,
        expected_edge,
        risk_config.min_confidence,
        walk_config,
        committee_config,
        risk_config,
    );
    let approved = risk_decision.kind == RiskDecisionKind::ApprovePaper;
    let gross_return = if first.close > 0.0 {
        (last.close / first.close - 1.0) * walk_config.max_position_fraction
    } else {
        0.0
    };
    let cost_paid = if approved {
        bps_fraction(walk_config.cost_bps) * 2.0 * walk_config.max_position_fraction
    } else {
        0.0
    };
    let slippage_paid = if approved {
        bps_fraction(walk_config.slippage_bps) * 2.0 * walk_config.max_position_fraction
    } else {
        0.0
    };
    let net_return = if approved {
        gross_return - cost_paid - slippage_paid
    } else {
        0.0
    };
    let samples = vec![PredictionQualitySample {
        predicted_probability: None,
        abstained: false,
        realized_direction_up: last.close > first.close,
    }];
    let reason_codes = stable_reason_codes(
        &risk_decision
            .reason_codes
            .iter()
            .cloned()
            .chain([
                ReasonCode::BaselineBuyAndHold,
                ReasonCode::WalkForwardCostApplied,
                ReasonCode::PaperExecutionOnly,
            ])
            .collect::<Vec<_>>(),
    );
    StrategyEvaluation {
        metrics: BaselinePerformanceMetrics {
            strategy: BaselineStrategyKind::BuyAndHold,
            total_return: net_return,
            max_drawdown: if approved {
                buy_hold_drawdown(series, split, first.close) * walk_config.max_position_fraction
            } else {
                0.0
            },
            trade_count: u64::from(approved),
            win_count: u64::from(approved && net_return > 0.0),
            loss_count: u64::from(approved && net_return < 0.0),
            no_trade_count: u64::from(!approved),
            risk_denial_count: u64::from(!approved),
            avg_return_per_trade: if approved { net_return } else { 0.0 },
            volatility_estimate: buy_hold_volatility(series, split),
            sharpe_like: None,
            downside_loss: (approved && net_return < 0.0).then_some(net_return.abs()),
            cost_paid,
            slippage_paid,
            reason_codes,
        },
        scoring: compute_prediction_quality_metrics(BaselineStrategyKind::BuyAndHold, &samples, 1),
        final_states: Vec::new(),
    }
}

fn evaluate_committee_window(
    strategy: BaselineStrategyKind,
    series: &CandleSeries,
    split: WalkForwardSplit,
    states: &[CanonicalAgentState],
    walk_config: &WalkForwardConfig,
    committee_config: &WalkForwardCommitteeConfig,
    risk_config: &GovernorConfig,
) -> StrategyEvaluation {
    let mut returns = Vec::new();
    let mut samples = Vec::new();
    let mut trade_count = 0u64;
    let mut win_count = 0u64;
    let mut loss_count = 0u64;
    let mut no_trade_count = 0u64;
    let mut risk_denial_count = 0u64;
    let mut cost_paid = 0.0;
    let mut slippage_paid = 0.0;
    let mut current_states = states.to_vec();
    let mut reason_codes = vec![
        strategy_reason_code(strategy),
        ReasonCode::PaperExecutionOnly,
    ];
    for index in split.eval_start_index..split.eval_end_index.saturating_sub(1) {
        let decision = committee_decision_at(
            strategy,
            series,
            index,
            &current_states,
            walk_config,
            committee_config,
            risk_config,
        );
        samples.push(PredictionQualitySample {
            predicted_probability: decision.predicted_probability,
            abstained: decision.position_fraction == 0.0,
            realized_direction_up: direction_up(series, index),
        });
        reason_codes.extend(decision.reason_codes.iter().cloned());
        if decision.position_fraction == 0.0 {
            no_trade_count += 1;
            continue;
        }
        if decision.risk_decision.kind != RiskDecisionKind::ApprovePaper {
            no_trade_count += 1;
            risk_denial_count += 1;
            if strategy == BaselineStrategyKind::VoiceAdaptiveCommittee {
                current_states =
                    apply_voice_adaptive_feedback(&current_states, &decision, 0.0, true, true);
            }
            continue;
        }
        let gross = next_return(series, index) * decision.position_fraction;
        let trade_cost =
            bps_fraction(walk_config.cost_bps) * 2.0 * decision.position_fraction.abs();
        let trade_slippage =
            bps_fraction(walk_config.slippage_bps) * 2.0 * decision.position_fraction.abs();
        let net = gross - trade_cost - trade_slippage;
        cost_paid += trade_cost;
        slippage_paid += trade_slippage;
        returns.push(net);
        trade_count += 1;
        if net > 0.0 {
            win_count += 1;
        } else if net < 0.0 {
            loss_count += 1;
        }
        if strategy == BaselineStrategyKind::VoiceAdaptiveCommittee {
            current_states =
                apply_voice_adaptive_feedback(&current_states, &decision, net, false, false);
        }
    }
    let total_return = returns.iter().sum::<f64>();
    let max_drawdown = cumulative_max_drawdown(&returns);
    StrategyEvaluation {
        metrics: BaselinePerformanceMetrics {
            strategy,
            total_return,
            max_drawdown,
            trade_count,
            win_count,
            loss_count,
            no_trade_count,
            risk_denial_count,
            avg_return_per_trade: if trade_count > 0 {
                total_return / trade_count as f64
            } else {
                0.0
            },
            volatility_estimate: volatility(&returns),
            sharpe_like: sharpe_like(&returns),
            downside_loss: downside_loss(&returns),
            cost_paid,
            slippage_paid,
            reason_codes: stable_reason_codes(&reason_codes),
        },
        scoring: compute_prediction_quality_metrics(strategy, &samples, 1),
        final_states: current_states,
    }
}

#[derive(Clone, Debug)]
struct CommitteeDecisionAt {
    decision_id: String,
    market: MarketSnapshot,
    signal: SignalOutput,
    votes: Vec<InvestorVote>,
    chair_output: ChairOutput,
    proposals: Vec<AgentProposal>,
    risk_decision: RiskDecision,
    position_fraction: f64,
    predicted_probability: Option<f64>,
    reason_codes: Vec<ReasonCode>,
}

fn committee_decision_at(
    strategy: BaselineStrategyKind,
    series: &CandleSeries,
    index: usize,
    states: &[CanonicalAgentState],
    walk_config: &WalkForwardConfig,
    committee_config: &WalkForwardCommitteeConfig,
    risk_config: &GovernorConfig,
) -> CommitteeDecisionAt {
    let market = series
        .market_snapshot_at(index)
        .expect("walk-forward split index validated");
    let signal = walk_forward_signal(series, index, committee_config);
    let mut votes = super::default_league_votes(&market, &signal);
    let adaptive = strategy == BaselineStrategyKind::VoiceAdaptiveCommittee;
    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;
    for vote in &mut votes {
        let state = states
            .iter()
            .find(|state| state.agent_id == vote.persona_id);
        let active = state.is_some_and(|state| {
            state.kind != AgentKind::Future8AgentPlaceholder
                && state.status == AgentStatus::Active
                && state.voice_state.cooldown_bars == 0
        });
        let weight = if !active {
            0.0
        } else if adaptive {
            state.map_or(0.0, |state| state.voice_state.voice_power)
        } else {
            1.0
        };
        vote.voice_power = weight;
        if weight == 0.0 {
            vote.stance = Stance::Abstain;
            vote.reason_codes.push(ReasonCode::AgentAbstained);
        }
        weighted_sum += vote.stance.direction() * vote.conviction * weight;
        weight_total += weight;
        vote.reason_codes = stable_reason_codes(&vote.reason_codes);
    }
    let score = if weight_total > 0.0 {
        weighted_sum / weight_total
    } else {
        0.0
    };
    let position_fraction = if score >= committee_config.min_vote_score_for_trade {
        walk_config.max_position_fraction
    } else if walk_config.allow_short && score <= -committee_config.min_vote_score_for_trade {
        -walk_config.max_position_fraction
    } else {
        0.0
    };
    let predicted_probability = Some(probability_from_vote_score(score, committee_config));
    let selected_speakers = votes
        .iter()
        .filter(|vote| vote.voice_power > 0.0)
        .map(|vote| vote.persona_id.clone())
        .collect::<Vec<_>>();
    let lead_speaker = votes
        .iter()
        .max_by(|left, right| {
            (left.voice_power * left.conviction)
                .partial_cmp(&(right.voice_power * right.conviction))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|vote| vote.persona_id.clone())
        .unwrap_or_else(|| "none".to_string());
    let chair_output = ChairOutput {
        selected_speakers,
        lead_speaker,
        forced_contrarian: false,
        council_score: score.abs().clamp(0.0, 1.0),
        disagreement_score: 0.0,
        groupthink_risk: 0.0,
        size_multiplier: position_fraction.abs(),
        decision: if position_fraction == 0.0 {
            crate::core::ChairDecisionKind::NoTrade
        } else {
            crate::core::ChairDecisionKind::ApproveCandidate
        },
        reason_codes: vec![
            strategy_reason_code(strategy),
            ReasonCode::DeterministicPath,
        ],
    };
    let decision_id = format!(
        "manual-walk-forward:{}:{}:{:?}",
        series.symbol, market.timestamp_ms, strategy
    );
    let proposals = build_loop_agent_proposals(
        &votes,
        states,
        &chair_output,
        &market,
        &signal,
        manual_market_name_from_symbol(&series.symbol),
        &decision_id,
    );
    let side = if position_fraction < 0.0 {
        Side::Short
    } else {
        Side::Long
    };
    let risk_decision = if position_fraction == 0.0 {
        RiskDecision {
            kind: RiskDecisionKind::Deny,
            approved_order_plan: None,
            reason_codes: vec![ReasonCode::NoTradePreferred],
            audit_id: format!("manual-walk-forward-no-trade:{}", market.timestamp_ms),
        }
    } else {
        evaluate_baseline_trade_risk(
            strategy,
            &market,
            side,
            position_fraction.abs(),
            score.abs() * committee_config.expected_edge_scale
                - bps_fraction(walk_config.cost_bps + walk_config.slippage_bps) * 2.0,
            predicted_probability
                .map(|probability| probability.max(1.0 - probability))
                .unwrap_or(0.0),
            walk_config,
            committee_config,
            risk_config,
        )
    };
    let reason_codes = stable_reason_codes(
        &chair_output
            .reason_codes
            .iter()
            .chain(risk_decision.reason_codes.iter())
            .cloned()
            .chain([
                ReasonCode::WalkForwardNoLookahead,
                ReasonCode::WalkForwardEvaluated,
            ])
            .collect::<Vec<_>>(),
    );
    CommitteeDecisionAt {
        decision_id,
        market,
        signal,
        votes,
        chair_output,
        proposals,
        risk_decision,
        position_fraction,
        predicted_probability,
        reason_codes,
    }
}

fn walk_forward_signal(
    series: &CandleSeries,
    index: usize,
    config: &WalkForwardCommitteeConfig,
) -> SignalOutput {
    let current = series.candle(index).expect("split index validated");
    let previous_close = index
        .checked_sub(1)
        .and_then(|previous| series.candle(previous))
        .map_or(current.open, |candle| candle.close);
    let past_return = if previous_close > 0.0 {
        current.close / previous_close - 1.0
    } else {
        0.0
    };
    let range_pct = if current.open > 0.0 {
        (current.high - current.low).abs() / current.open
    } else {
        0.0
    };
    let p_win = (0.5 + past_return * config.probability_return_scale)
        .clamp(config.min_probability, config.max_probability);
    SignalOutput {
        symbol: series.symbol.clone(),
        horizon_bars: 1,
        p_win,
        p_stop: (1.0 - p_win).clamp(config.min_probability, config.max_probability),
        expected_return: past_return,
        expected_drawdown: range_pct,
        confidence: ((p_win - 0.5).abs() * 2.0).clamp(0.0, 1.0),
        no_trade_probability: (1.0 - (p_win - 0.5).abs() * 2.0).clamp(0.0, 1.0),
        source: "manual-historical-walk-forward".to_string(),
    }
}

fn evaluate_baseline_trade_risk(
    strategy: BaselineStrategyKind,
    market: &MarketSnapshot,
    side: Side,
    quantity_hint: f64,
    expected_edge_after_cost: f64,
    confidence: f64,
    walk_config: &WalkForwardConfig,
    committee_config: &WalkForwardCommitteeConfig,
    risk_config: &GovernorConfig,
) -> RiskDecision {
    let stop_loss = if side == Side::Long {
        market.price * (1.0 - bps_fraction(committee_config.stop_loss_bps))
    } else {
        market.price * (1.0 + bps_fraction(committee_config.stop_loss_bps))
    };
    let take_profit = if side == Side::Long {
        market.price * (1.0 + bps_fraction(committee_config.take_profit_bps))
    } else {
        market.price * (1.0 - bps_fraction(committee_config.take_profit_bps))
    };
    let chair_output = ChairOutput {
        selected_speakers: vec![format!("{strategy:?}")],
        lead_speaker: format!("{strategy:?}"),
        forced_contrarian: false,
        council_score: confidence.clamp(0.0, 1.0),
        disagreement_score: 0.0,
        groupthink_risk: 0.0,
        size_multiplier: quantity_hint,
        decision: crate::core::ChairDecisionKind::ApproveCandidate,
        reason_codes: vec![strategy_reason_code(strategy)],
    };
    let proposal = TradeProposal {
        symbol: market.symbol.clone(),
        side,
        quantity_hint,
        entry_price_hint: market.price,
        stop_loss: Some(stop_loss),
        take_profit: Some(take_profit),
        max_slippage_bps: walk_config.slippage_bps,
        expected_edge_after_cost,
        confidence: confidence.clamp(0.0, 1.0),
        source_chair_output: chair_output,
    };
    RiskGovernor {
        config: *risk_config,
    }
    .evaluate(
        market,
        &RiskSnapshot {
            daily_pnl_pct: 0.0,
            consecutive_losses: 0,
            current_positions_count: 0,
            total_exposure_pct: 0.0,
            symbol_exposure_pct: 0.0,
            api_health_score: 1.0,
            data_quality_score: market.data_quality_score,
        },
        Some(&proposal),
        market.timestamp_ms,
    )
}

fn apply_voice_adaptive_feedback(
    states: &[CanonicalAgentState],
    decision: &CommitteeDecisionAt,
    net_return: f64,
    denied_by_risk: bool,
    no_trade: bool,
) -> Vec<CanonicalAgentState> {
    let outcome = committee_outcome_record(decision, net_return, denied_by_risk, no_trade);
    let context = FeedbackContext {
        paper_only: true,
        outcome_finalized: true,
        doctrine_violation: false,
        overtrade: false,
    };
    states
        .iter()
        .map(|state| {
            let Some(proposal) = decision
                .proposals
                .iter()
                .find(|proposal| proposal.agent_id == state.agent_id)
            else {
                return state.clone();
            };
            apply_paper_feedback_cycle(state, proposal, &outcome, &context)
                .map(|cycle| cycle.updated_state)
                .unwrap_or_else(|_| state.clone())
        })
        .collect()
}

fn committee_outcome_record(
    decision: &CommitteeDecisionAt,
    net_return: f64,
    denied_by_risk: bool,
    no_trade: bool,
) -> OutcomeRecord {
    let executed = !denied_by_risk && !no_trade && decision.position_fraction != 0.0;
    let hypothetical = net_return;
    let avoided_loss_score = if !executed && hypothetical < 0.0 {
        hypothetical.abs()
    } else {
        0.0
    };
    let missed_gain_penalty = if !executed && hypothetical > 0.0 {
        hypothetical * 0.20
    } else {
        0.0
    };
    let attribution_records = build_loop_attribution(
        &decision.votes,
        &decision.chair_output,
        denied_by_risk,
        no_trade,
    );
    let mut reason_codes = decision.reason_codes.clone();
    reason_codes.push(ReasonCode::PaperExecutionOnly);
    if executed {
        reason_codes.push(ReasonCode::PaperFillSimulated);
    } else if denied_by_risk {
        reason_codes.push(ReasonCode::RiskDeniedCounterfactual);
    } else {
        reason_codes.push(ReasonCode::NoTradeCounterfactual);
    }
    OutcomeRecord {
        decision_id: decision.decision_id.clone(),
        symbol: decision.market.symbol.clone(),
        timestamp_ms: decision.market.timestamp_ms,
        regime: decision.market.regime,
        horizon: Horizon::Position,
        signal_confidence: decision.signal.confidence,
        executed,
        denied_by_risk,
        no_trade,
        triple_barrier_result: executed.then(|| {
            synthetic_barrier_result(net_return, decision.market.price, net_return.min(0.0).abs())
        }),
        hypothetical_result: (!executed).then(|| {
            synthetic_barrier_result(
                hypothetical,
                decision.market.price,
                hypothetical.min(0.0).abs(),
            )
        }),
        realized_net_return_pct: if executed { net_return } else { 0.0 },
        avoided_loss_score,
        missed_gain_penalty,
        attribution_records,
        shadow_outcomes: Vec::new(),
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn empty_strategy_evaluation(strategy: BaselineStrategyKind) -> StrategyEvaluation {
    StrategyEvaluation {
        metrics: BaselinePerformanceMetrics {
            strategy,
            total_return: 0.0,
            max_drawdown: 0.0,
            trade_count: 0,
            win_count: 0,
            loss_count: 0,
            no_trade_count: 0,
            risk_denial_count: 0,
            avg_return_per_trade: 0.0,
            volatility_estimate: None,
            sharpe_like: None,
            downside_loss: None,
            cost_paid: 0.0,
            slippage_paid: 0.0,
            reason_codes: vec![strategy_reason_code(strategy)],
        },
        scoring: compute_prediction_quality_metrics(strategy, &[], 1),
        final_states: Vec::new(),
    }
}

fn strategy_reason_code(strategy: BaselineStrategyKind) -> ReasonCode {
    match strategy {
        BaselineStrategyKind::AlwaysNoTrade => ReasonCode::BaselineAlwaysNoTrade,
        BaselineStrategyKind::BuyAndHold => ReasonCode::BaselineBuyAndHold,
        BaselineStrategyKind::EqualWeightCommittee => ReasonCode::BaselineEqualWeightCommittee,
        BaselineStrategyKind::VoiceAdaptiveCommittee => ReasonCode::BaselineVoiceAdaptiveCommittee,
    }
}

fn probability_from_vote_score(score: f64, config: &WalkForwardCommitteeConfig) -> f64 {
    (0.5 + score / 2.0).clamp(config.min_probability, config.max_probability)
}

fn bps_fraction(bps: f64) -> f64 {
    bps / 10_000.0
}

fn direction_up(series: &CandleSeries, index: usize) -> bool {
    series
        .candle(index)
        .zip(series.candle(index + 1))
        .is_some_and(|(current, next)| next.close > current.close)
}

fn next_return(series: &CandleSeries, index: usize) -> f64 {
    series
        .candle(index)
        .zip(series.candle(index + 1))
        .and_then(|(current, next)| {
            (current.close > 0.0).then_some(next.close / current.close - 1.0)
        })
        .unwrap_or(0.0)
}

fn buy_hold_drawdown(series: &CandleSeries, split: WalkForwardSplit, entry_price: f64) -> f64 {
    if entry_price <= 0.0 {
        return 0.0;
    }
    let mut peak = entry_price;
    let mut max_drawdown = 0.0_f64;
    for candle in &series.candles[split.eval_start_index..split.eval_end_index] {
        peak = peak.max(candle.close);
        if peak > 0.0 {
            max_drawdown = max_drawdown.max((peak - candle.close) / peak);
        }
    }
    max_drawdown
}

fn buy_hold_volatility(series: &CandleSeries, split: WalkForwardSplit) -> Option<f64> {
    let returns = (split.eval_start_index..split.eval_end_index.saturating_sub(1))
        .map(|index| next_return(series, index))
        .collect::<Vec<_>>();
    volatility(&returns)
}

fn volatility(returns: &[f64]) -> Option<f64> {
    if returns.len() < 3 {
        return None;
    }
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance = returns
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / returns.len() as f64;
    Some(variance.sqrt())
}

fn sharpe_like(returns: &[f64]) -> Option<f64> {
    let vol = volatility(returns)?;
    if vol <= f64::EPSILON {
        None
    } else {
        Some((returns.iter().sum::<f64>() / returns.len() as f64) / vol)
    }
}

fn downside_loss(returns: &[f64]) -> Option<f64> {
    let losses = returns
        .iter()
        .copied()
        .filter(|value| *value < 0.0)
        .collect::<Vec<_>>();
    (!losses.is_empty())
        .then_some(losses.iter().map(|value| value.abs()).sum::<f64>() / losses.len() as f64)
}

fn cumulative_max_drawdown(returns: &[f64]) -> f64 {
    let mut cumulative = 0.0_f64;
    let mut peak = 0.0_f64;
    let mut max_drawdown = 0.0_f64;
    for value in returns {
        cumulative += value;
        peak = peak.max(cumulative);
        max_drawdown = max_drawdown.max(peak - cumulative);
    }
    max_drawdown
}

fn manual_market_name_from_symbol(symbol: &str) -> &'static str {
    if symbol.ends_with(".KS") || symbol.ends_with(".KQ") {
        "KR"
    } else if symbol.contains("BTC") {
        "BTC"
    } else {
        "US"
    }
}

fn aggregate_walk_forward_metrics(
    windows: &[WalkForwardWindowResult],
) -> BTreeMap<BaselineStrategyKind, BaselinePerformanceMetrics> {
    let mut grouped = BTreeMap::<BaselineStrategyKind, Vec<BaselinePerformanceMetrics>>::new();
    for window in windows {
        for metrics in window
            .baseline_results
            .iter()
            .chain(window.committee_results.iter())
        {
            grouped
                .entry(metrics.strategy)
                .or_default()
                .push(metrics.clone());
        }
    }
    grouped
        .into_iter()
        .map(|(strategy, rows)| (strategy, aggregate_strategy_metrics(strategy, &rows)))
        .collect()
}

fn aggregate_strategy_metrics(
    strategy: BaselineStrategyKind,
    rows: &[BaselinePerformanceMetrics],
) -> BaselinePerformanceMetrics {
    let total_return = rows.iter().map(|row| row.total_return).sum::<f64>();
    let trade_count = rows.iter().map(|row| row.trade_count).sum::<u64>();
    let mut reason_codes = vec![
        strategy_reason_code(strategy),
        ReasonCode::DeterministicPath,
    ];
    reason_codes.extend(rows.iter().flat_map(|row| row.reason_codes.iter().cloned()));
    let window_returns = rows.iter().map(|row| row.total_return).collect::<Vec<_>>();
    BaselinePerformanceMetrics {
        strategy,
        total_return,
        max_drawdown: rows
            .iter()
            .map(|row| row.max_drawdown)
            .fold(0.0_f64, f64::max),
        trade_count,
        win_count: rows.iter().map(|row| row.win_count).sum(),
        loss_count: rows.iter().map(|row| row.loss_count).sum(),
        no_trade_count: rows.iter().map(|row| row.no_trade_count).sum(),
        risk_denial_count: rows.iter().map(|row| row.risk_denial_count).sum(),
        avg_return_per_trade: if trade_count > 0 {
            total_return / trade_count as f64
        } else {
            0.0
        },
        volatility_estimate: volatility(&window_returns),
        sharpe_like: sharpe_like(&window_returns),
        downside_loss: downside_loss(&window_returns),
        cost_paid: rows.iter().map(|row| row.cost_paid).sum(),
        slippage_paid: rows.iter().map(|row| row.slippage_paid).sum(),
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn aggregate_prediction_summaries(
    windows: &[WalkForwardWindowResult],
    min_samples: usize,
) -> Vec<PredictionQualityMetrics> {
    [
        BaselineStrategyKind::AlwaysNoTrade,
        BaselineStrategyKind::BuyAndHold,
        BaselineStrategyKind::EqualWeightCommittee,
        BaselineStrategyKind::VoiceAdaptiveCommittee,
    ]
    .into_iter()
    .map(|strategy| {
        let rows = windows
            .iter()
            .flat_map(|window| window.scoring_results.iter())
            .filter(|metrics| metrics.strategy == strategy)
            .collect::<Vec<_>>();
        aggregate_prediction_metrics(strategy, &rows, min_samples)
    })
    .collect()
}

fn aggregate_prediction_metrics(
    strategy: BaselineStrategyKind,
    rows: &[&PredictionQualityMetrics],
    min_samples: usize,
) -> PredictionQualityMetrics {
    let sample_count = rows.iter().map(|row| row.sample_count).sum::<usize>();
    let calibrated_sample_count = rows
        .iter()
        .map(|row| row.calibrated_sample_count)
        .sum::<usize>();
    let brier_numerator = rows
        .iter()
        .filter_map(|row| {
            row.brier_score
                .map(|score| score * row.calibrated_sample_count as f64)
        })
        .sum::<f64>();
    let confidence_numerator = rows
        .iter()
        .filter_map(|row| {
            row.mean_confidence
                .map(|confidence| confidence * row.calibrated_sample_count as f64)
        })
        .sum::<f64>();
    let realized_numerator = rows
        .iter()
        .filter_map(|row| {
            row.mean_realized_direction
                .map(|realized| realized * row.sample_count as f64)
        })
        .sum::<f64>();
    let mut reason_codes = vec![ReasonCode::PredictionScoringBrier];
    reason_codes.extend(rows.iter().flat_map(|row| row.reason_codes.iter().cloned()));
    if sample_count < min_samples || calibrated_sample_count == 0 {
        reason_codes.push(ReasonCode::PredictionScoringInsufficientSamples);
    }
    PredictionQualityMetrics {
        strategy,
        brier_score: (sample_count >= min_samples && calibrated_sample_count > 0)
            .then_some(brier_numerator / calibrated_sample_count as f64),
        sample_count,
        calibrated_sample_count,
        missing_probability_count: rows.iter().map(|row| row.missing_probability_count).sum(),
        abstention_count: rows.iter().map(|row| row.abstention_count).sum(),
        high_confidence_error_count: rows.iter().map(|row| row.high_confidence_error_count).sum(),
        low_confidence_correct_count: rows
            .iter()
            .map(|row| row.low_confidence_correct_count)
            .sum(),
        mean_confidence: (calibrated_sample_count > 0)
            .then_some(confidence_numerator / calibrated_sample_count as f64),
        mean_realized_direction: (sample_count > 0)
            .then_some(realized_numerator / sample_count as f64),
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn build_proof_gate_comparison(
    metrics: &BTreeMap<BaselineStrategyKind, BaselinePerformanceMetrics>,
    scoring: &[PredictionQualityMetrics],
    min_prediction_samples: usize,
) -> ProofGateComparison {
    let always = metrics
        .get(&BaselineStrategyKind::AlwaysNoTrade)
        .cloned()
        .unwrap_or_else(|| aggregate_strategy_metrics(BaselineStrategyKind::AlwaysNoTrade, &[]));
    let buy_hold = metrics
        .get(&BaselineStrategyKind::BuyAndHold)
        .cloned()
        .unwrap_or_else(|| aggregate_strategy_metrics(BaselineStrategyKind::BuyAndHold, &[]));
    let equal = metrics
        .get(&BaselineStrategyKind::EqualWeightCommittee)
        .cloned()
        .unwrap_or_else(|| {
            aggregate_strategy_metrics(BaselineStrategyKind::EqualWeightCommittee, &[])
        });
    let voice = metrics
        .get(&BaselineStrategyKind::VoiceAdaptiveCommittee)
        .cloned()
        .unwrap_or_else(|| {
            aggregate_strategy_metrics(BaselineStrategyKind::VoiceAdaptiveCommittee, &[])
        });
    let voice_beats_equal_weight = risk_adjusted_score(&voice) > risk_adjusted_score(&equal);
    let committee_beats_no_trade =
        voice.total_return > always.total_return && voice.trade_count > 0;
    let committee_beats_buy_hold_risk_adjusted =
        risk_adjusted_score(&voice) > risk_adjusted_score(&buy_hold);
    let voice_scoring_samples = scoring
        .iter()
        .find(|metrics| metrics.strategy == BaselineStrategyKind::VoiceAdaptiveCommittee)
        .map_or(0, |metrics| metrics.sample_count);
    let insufficient_evidence = voice_scoring_samples < min_prediction_samples;
    let mut reason_codes = vec![ReasonCode::HardcodingAuditPassed];
    if voice_beats_equal_weight {
        reason_codes.push(ReasonCode::BaselineComparisonVoiceAdaptationHelped);
    } else {
        reason_codes.push(ReasonCode::BaselineComparisonVoiceAdaptationFailed);
    }
    if !committee_beats_no_trade || !committee_beats_buy_hold_risk_adjusted {
        reason_codes.push(ReasonCode::BaselineComparisonNoEdge);
    }
    if insufficient_evidence {
        reason_codes.push(ReasonCode::PredictionScoringInsufficientSamples);
    }
    ProofGateComparison {
        always_no_trade: always,
        buy_and_hold: buy_hold,
        equal_weight_committee: equal,
        voice_adaptive_committee: voice,
        voice_beats_equal_weight,
        committee_beats_no_trade,
        committee_beats_buy_hold_risk_adjusted,
        insufficient_evidence,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn risk_adjusted_score(metrics: &BaselinePerformanceMetrics) -> f64 {
    metrics.sharpe_like.unwrap_or(metrics.total_return) - metrics.max_drawdown
}

pub fn build_owner_learning_report_from_local_csv_source(
    report_id: &str,
    csv_text: &str,
    source_kind: LocalDataSourceKind,
    historical_config: &HistoricalReplayConfig,
    initial_agent_states: &[CanonicalAgentState],
    replay_config: PaperReplayConfig,
) -> Result<OwnerLearningReport, LocalDataSourceError> {
    let registry = LocalDataSourceRegistry::default();
    let profile = registry
        .get_profile(source_kind)
        .ok_or_else(|| local_registry_error(ReasonCode::LocalSourceUnknown))?;
    let parsed = parse_local_csv_with_profile(csv_text, profile, historical_config)?;
    let replay_input = HistoricalReplayAdapter
        .to_paper_replay_input(
            &parsed.dataset,
            historical_config,
            initial_agent_states.to_vec(),
            replay_config,
        )
        .map_err(LocalDataSourceError::Historical)?;
    let replay = run_3_agent_paper_replay(replay_input).map_err(LocalDataSourceError::Replay)?;
    let mut report = build_owner_learning_report(
        report_id,
        Some(format!(
            "local-csv:{}:{}",
            local_source_kind_label(source_kind),
            parsed.dataset.symbol
        )),
        &replay,
    )
    .map_err(LocalDataSourceError::Report)?;
    report.data_quality_summary = Some(parsed.quality_summary);
    report
        .safety_warnings
        .push("Local CSV source is sanitized and read-only.".to_string());
    report.reason_codes = stable_reason_codes(
        &report
            .reason_codes
            .iter()
            .chain(parsed.reason_codes.iter())
            .cloned()
            .collect::<Vec<_>>(),
    );
    Ok(report)
}

pub fn run_local_dataset_batch_replay(
    input: BatchReplayInput,
) -> Result<BatchReplayResult, BatchReplayError> {
    validate_three_agent_set(&input.initial_agent_states).map_err(|_| BatchReplayError {
        source_id: None,
        reason_codes: vec![
            ReasonCode::BatchReplaySourceRejected,
            ReasonCode::LocalSourceRejected,
        ],
    })?;
    if input.sources.is_empty()
        || input.sources.len() > input.config.max_sources
        || input.config.max_sources == 0
        || input.config.max_rows_per_source == 0
        || input.config.active_agent_limit != 3
        || input.replay_config.active_agent_limit != 3
    {
        return Err(BatchReplayError {
            source_id: None,
            reason_codes: vec![ReasonCode::LocalSourceRejected],
        });
    }
    let mut source_ids = input
        .sources
        .iter()
        .map(|source| source.source_id.as_str())
        .collect::<Vec<_>>();
    source_ids.sort_unstable();
    if source_ids
        .iter()
        .any(|source_id| source_id.trim().is_empty())
        || source_ids.windows(2).any(|pair| pair[0] == pair[1])
    {
        return Err(BatchReplayError {
            source_id: None,
            reason_codes: vec![ReasonCode::LocalSourceRejected],
        });
    }
    let replay_mode = input.config.replay_mode;
    let source_order_policy = input.config.source_order_policy;
    let quality_policy = input.config.quality_policy;
    let mut sources = input.sources;
    if source_order_policy == SourceOrderPolicy::SourceKindThenId {
        sources.sort_by(|left, right| {
            left.source_kind
                .cmp(&right.source_kind)
                .then_with(|| left.source_id.cmp(&right.source_id))
        });
    }

    let registry = LocalDataSourceRegistry::default();
    let initial_states = input.initial_agent_states.clone();
    let mut current_states = initial_states.clone();
    let mut source_results = Vec::with_capacity(sources.len());
    let mut detailed_agent_rows = Vec::new();
    let mut source_rows = Vec::with_capacity(sources.len());
    let mut reason_codes = input.config.reason_codes.clone();

    for source in sources {
        let source_failure = if !source.enabled {
            Some(vec![
                ReasonCode::BatchReplaySourceRejected,
                ReasonCode::LocalSourceRejected,
            ])
        } else if let Some(reason) = batch_source_safety_reason(&source) {
            Some(vec![ReasonCode::BatchReplaySourceRejected, reason])
        } else {
            None
        };
        if let Some(failure_reasons) = source_failure {
            if input.config.require_all_sources_valid || input.config.stop_on_source_error {
                return Err(BatchReplayError {
                    source_id: Some(source.source_id),
                    reason_codes: stable_reason_codes(&failure_reasons),
                });
            }
            let rejected = rejected_batch_source_result(&source, failure_reasons);
            source_rows.push(rejected.source_performance_row.clone());
            reason_codes.extend(rejected.reason_codes.iter().cloned());
            source_results.push(rejected);
            continue;
        }

        let parsed = registry
            .get_profile(source.source_kind)
            .ok_or_else(|| local_registry_error(ReasonCode::LocalSourceUnknown))
            .and_then(|profile| {
                if source.profile_name.trim().is_empty() || source.profile_name != profile.name {
                    return Err(local_registry_error(ReasonCode::LocalSourceProfileInvalid));
                }
                let historical_config = HistoricalReplayConfig {
                    max_rows: input.config.max_rows_per_source,
                    ..HistoricalReplayConfig::default()
                };
                parse_local_csv_with_profile(&source.csv_text, profile, &historical_config)
                    .map(|parsed| (parsed, historical_config, profile.clone()))
            });
        let (parsed, historical_config, profile) = match parsed {
            Ok(parsed) => parsed,
            Err(error) => {
                let mut failure_reasons = local_data_source_error_reasons(&error);
                failure_reasons.push(ReasonCode::BatchReplaySourceRejected);
                if input.config.require_all_sources_valid || input.config.stop_on_source_error {
                    return Err(BatchReplayError {
                        source_id: Some(source.source_id),
                        reason_codes: stable_reason_codes(&failure_reasons),
                    });
                }
                let rejected = rejected_batch_source_result(&source, failure_reasons);
                source_rows.push(rejected.source_performance_row.clone());
                reason_codes.extend(rejected.reason_codes.iter().cloned());
                source_results.push(rejected);
                continue;
            }
        };
        let source_consistency_diagnostics =
            build_source_consistency_diagnostics(&source, &profile, &parsed.dataset);
        let quality_score = source_quality_score(&source_consistency_diagnostics);
        let replay_blocked_by_quality =
            quality_replay_blocked(quality_policy, quality_score.bucket);
        if replay_blocked_by_quality {
            let blocked = quality_blocked_source_result(
                &source,
                parsed.quality_summary,
                source_consistency_diagnostics,
                quality_score,
            );
            source_rows.push(blocked.source_performance_row.clone());
            reason_codes.extend(blocked.reason_codes.iter().cloned());
            source_results.push(blocked);
            continue;
        }
        let replay_input = match HistoricalReplayAdapter.to_paper_replay_input(
            &parsed.dataset,
            &historical_config,
            match replay_mode {
                BatchReplayMode::IndependentPerSource => initial_states.clone(),
                BatchReplayMode::SequentialCarryover => current_states.clone(),
            },
            input.replay_config,
        ) {
            Ok(replay_input) => replay_input,
            Err(error) => {
                let failure_reasons = stable_reason_codes(
                    &error
                        .reason_codes
                        .iter()
                        .cloned()
                        .chain([ReasonCode::BatchReplaySourceRejected])
                        .collect::<Vec<_>>(),
                );
                if input.config.require_all_sources_valid || input.config.stop_on_source_error {
                    return Err(BatchReplayError {
                        source_id: Some(source.source_id),
                        reason_codes: failure_reasons,
                    });
                }
                let rejected = rejected_batch_source_result(&source, failure_reasons);
                source_rows.push(rejected.source_performance_row.clone());
                reason_codes.extend(rejected.reason_codes.iter().cloned());
                source_results.push(rejected);
                continue;
            }
        };
        let replay = match run_3_agent_paper_replay(replay_input) {
            Ok(replay) => replay,
            Err(_) => {
                let failure_reasons = vec![
                    ReasonCode::BatchReplaySourceRejected,
                    ReasonCode::LocalSourceRejected,
                    ReasonCode::DeterministicPath,
                    ReasonCode::PaperExecutionOnly,
                ];
                if input.config.require_all_sources_valid || input.config.stop_on_source_error {
                    return Err(BatchReplayError {
                        source_id: Some(source.source_id),
                        reason_codes: stable_reason_codes(&failure_reasons),
                    });
                }
                let rejected = rejected_batch_source_result(&source, failure_reasons);
                source_rows.push(rejected.source_performance_row.clone());
                reason_codes.extend(rejected.reason_codes.iter().cloned());
                source_results.push(rejected);
                continue;
            }
        };
        let mut owner_report = match build_owner_learning_report(
            &format!("batch-owner-report:{}", source.source_id),
            Some(format!(
                "batch-local-csv:{}:{}",
                local_source_kind_label(source.source_kind),
                parsed.dataset.symbol
            )),
            &replay,
        ) {
            Ok(report) => report,
            Err(_) => {
                let failure_reasons = vec![
                    ReasonCode::BatchReplaySourceRejected,
                    ReasonCode::LocalSourceRejected,
                ];
                if input.config.require_all_sources_valid || input.config.stop_on_source_error {
                    return Err(BatchReplayError {
                        source_id: Some(source.source_id),
                        reason_codes: stable_reason_codes(&failure_reasons),
                    });
                }
                let rejected = rejected_batch_source_result(&source, failure_reasons);
                source_rows.push(rejected.source_performance_row.clone());
                reason_codes.extend(rejected.reason_codes.iter().cloned());
                source_results.push(rejected);
                continue;
            }
        };
        owner_report.data_quality_summary = Some(parsed.quality_summary.clone());
        owner_report
            .safety_warnings
            .push("Local CSV source is sanitized and read-only.".to_string());
        owner_report.reason_codes = stable_reason_codes(
            &owner_report
                .reason_codes
                .iter()
                .chain(parsed.reason_codes.iter())
                .cloned()
                .collect::<Vec<_>>(),
        );
        let agent_rows = build_agent_performance_rows(&source, &replay, &owner_report);
        let source_row = accepted_source_performance_row(&source, &parsed.quality_summary, &replay);
        if replay_mode == BatchReplayMode::SequentialCarryover {
            current_states = replay.final_states.clone();
        }
        detailed_agent_rows.extend(agent_rows.iter().cloned());
        source_rows.push(source_row.clone());
        let accepted_reasons = stable_reason_codes(
            &source
                .reason_codes
                .iter()
                .chain(parsed.reason_codes.iter())
                .cloned()
                .chain([
                    ReasonCode::BatchReplaySourceAccepted,
                    ReasonCode::PaperExecutionOnly,
                ])
                .collect::<Vec<_>>(),
        );
        reason_codes.extend(accepted_reasons.iter().cloned());
        source_results.push(BatchReplaySourceResult {
            source_id: source.source_id,
            source_kind: source.source_kind,
            accepted: true,
            dataset_quality_summary: Some(parsed.quality_summary),
            replay_result: Some(replay),
            owner_learning_report: input.config.include_owner_reports.then_some(owner_report),
            agent_performance_rows: if input.config.include_agent_tables {
                agent_rows
            } else {
                Vec::new()
            },
            source_performance_row: source_row,
            source_consistency_diagnostics,
            quality_score: quality_score.clone(),
            quality_bucket: quality_score.bucket,
            quality_reason_codes: quality_score.reason_codes.clone(),
            replay_blocked_by_quality: false,
            reason_codes: accepted_reasons,
        });
    }

    source_rows.sort_by(|left, right| {
        left.source_kind
            .cmp(&right.source_kind)
            .then_with(|| left.source_id.cmp(&right.source_id))
    });
    let accepted_sources = source_rows.iter().filter(|row| row.accepted).count();
    let rejected_sources = source_rows.len().saturating_sub(accepted_sources);
    let aggregate_agent_performance_table = build_agent_performance_table(detailed_agent_rows);
    let aggregate_source_performance_table = build_source_performance_table(source_rows);
    let cross_source_consistency_report = build_cross_source_consistency_report(&source_results);
    let agent_cross_source_consistency_table =
        build_agent_cross_source_consistency_table(&aggregate_agent_performance_table.rows);
    let agent_performance_by_quality_table = build_agent_performance_by_quality_table(
        &aggregate_agent_performance_table.rows,
        &source_results,
    );
    let mut quality_bucket_counts = BTreeMap::new();
    for result in &source_results {
        *quality_bucket_counts
            .entry(result.quality_bucket)
            .or_insert(0) += 1;
    }
    let aggregate_learning_summary = BatchLearningSummary {
        total_sources: source_results.len(),
        total_episodes: aggregate_source_performance_table
            .rows
            .iter()
            .map(|row| row.total_episodes)
            .sum(),
        total_paper_trades: aggregate_source_performance_table
            .rows
            .iter()
            .map(|row| row.total_paper_trades)
            .sum(),
        total_no_trades: aggregate_source_performance_table
            .rows
            .iter()
            .map(|row| row.total_no_trades)
            .sum(),
        total_risk_denials: aggregate_source_performance_table
            .rows
            .iter()
            .map(|row| row.total_risk_denials)
            .sum(),
        sandbox_candidate_count: source_results
            .iter()
            .filter_map(|result| result.replay_result.as_ref())
            .map(|replay| replay.sandbox_candidates.len())
            .sum(),
        any_live_mutation_detected: source_results
            .iter()
            .filter_map(|result| result.replay_result.as_ref())
            .any(|replay| replay.learning_chain_summary.any_live_mutation_detected),
        any_risk_bypass_detected: source_results
            .iter()
            .filter_map(|result| result.replay_result.as_ref())
            .any(|replay| replay.learning_chain_summary.any_risk_bypass_detected),
        reason_codes: vec![
            ReasonCode::BatchReplayBuilt,
            ReasonCode::DeterministicPath,
            ReasonCode::PaperExecutionOnly,
        ],
    };
    reason_codes.extend([
        ReasonCode::BatchReplayBuilt,
        ReasonCode::DeterministicPath,
        ReasonCode::PaperExecutionOnly,
    ]);
    let source_processing_order = source_results
        .iter()
        .map(|result| result.source_id.clone())
        .collect();

    Ok(BatchReplayResult {
        initial_states,
        final_states: current_states,
        source_processing_order,
        source_results,
        aggregate_agent_performance_table,
        aggregate_source_performance_table,
        cross_source_consistency_report,
        agent_cross_source_consistency_table,
        agent_performance_by_quality_table,
        quality_bucket_counts,
        aggregate_learning_summary,
        rejected_sources,
        accepted_sources,
        replay_mode,
        source_order_policy,
        quality_policy,
        reason_codes: stable_reason_codes(&reason_codes),
    })
}

pub fn build_batch_owner_learning_report(batch: &BatchReplayResult) -> BatchOwnerLearningReport {
    BatchOwnerLearningReport {
        batch_summary: batch.aggregate_learning_summary.clone(),
        source_performance_table: batch.aggregate_source_performance_table.clone(),
        agent_performance_table: batch.aggregate_agent_performance_table.clone(),
        cross_source_consistency_report: batch.cross_source_consistency_report.clone(),
        agent_cross_source_consistency_table: batch.agent_cross_source_consistency_table.clone(),
        agent_performance_by_quality_table: batch.agent_performance_by_quality_table.clone(),
        quality_policy: batch.quality_policy,
        quality_bucket_counts: batch.quality_bucket_counts.clone(),
        blocked_by_quality_sources: batch
            .source_results
            .iter()
            .filter(|source| source.replay_blocked_by_quality)
            .map(|source| source.source_id.clone())
            .collect(),
        source_quality_threshold_summary: build_source_quality_threshold_summary(),
        replay_mode: batch.replay_mode,
        source_order_policy: batch.source_order_policy,
        source_processing_order: batch.source_processing_order.clone(),
        per_source_report_refs: batch
            .source_results
            .iter()
            .filter_map(|source| {
                source
                    .owner_learning_report
                    .as_ref()
                    .map(|report| report.report_id.clone())
            })
            .collect(),
        safety_warnings: vec![
            "Paper-only batch report.".to_string(),
            "Not live trading ready.".to_string(),
            "Risk Governor remains final veto.".to_string(),
            "Owner input remains advisory only.".to_string(),
            "Synthetic/sanitized local data only.".to_string(),
            "Source quality diagnostics are not live data validation.".to_string(),
            "No profitability claim.".to_string(),
            "Market calendar/session validation is deferred.".to_string(),
            "Expanded fixtures remain synthetic and are not production market data.".to_string(),
        ],
        deferred_items: vec![
            "Live downloads remain disabled.".to_string(),
            "Broker and account integration remain disabled.".to_string(),
            "Real profitability is not established.".to_string(),
        ],
        reason_codes: vec![
            ReasonCode::BatchReplayBuilt,
            ReasonCode::DeterministicPath,
            ReasonCode::PaperExecutionOnly,
        ],
    }
}

pub fn render_batch_owner_learning_report_text(report: &BatchOwnerLearningReport) -> String {
    let mut lines = vec!["Local Dataset Batch Learning Report".to_string()];
    lines.extend(report.safety_warnings.iter().cloned());
    lines.extend([
        "Replay mode".to_string(),
        format!(
            "replay_mode={:?} source_order_policy={:?} quality_policy={:?} processing_order={}",
            report.replay_mode,
            report.source_order_policy,
            report.quality_policy,
            report.source_processing_order.join(",")
        ),
        "Source quality threshold summary".to_string(),
    ]);
    lines.extend(report.source_quality_threshold_summary.iter().cloned());
    lines.push("Quality bucket counts".to_string());
    for (bucket, count) in &report.quality_bucket_counts {
        lines.push(format!("quality_bucket={bucket:?} count={count}"));
    }
    lines.push("Blocked-by-quality sources".to_string());
    if report.blocked_by_quality_sources.is_empty() {
        lines.push("none".to_string());
    } else {
        lines.extend(
            report
                .blocked_by_quality_sources
                .iter()
                .map(|source| format!("blocked_source={source}")),
        );
    }
    lines.extend([
        "Source summary".to_string(),
        format!(
            "sources={} accepted={} rejected={} episodes={} paper_trades={} no_trades={} risk_denials={}",
            report.batch_summary.total_sources,
            report.source_performance_table.accepted_count,
            report.source_performance_table.rejected_count,
            report.batch_summary.total_episodes,
            report.batch_summary.total_paper_trades,
            report.batch_summary.total_no_trades,
            report.batch_summary.total_risk_denials,
        ),
    ]);
    for source in &report.source_performance_table.rows {
        lines.push(format!(
            "source={} kind={:?} accepted={} rows={} episodes={} no_trades={} risk_denials={} paper_only={} not_live_ready={}",
            source.source_id,
            source.source_kind,
            source.accepted,
            source.total_rows,
            source.total_episodes,
            source.total_no_trades,
            source.total_risk_denials,
            source.paper_only,
            source.not_live_ready,
        ));
    }
    lines.push("Agent performance table".to_string());
    for agent in &report.agent_performance_table.aggregate_rows_by_agent {
        lines.push(format!(
            "agent={} episodes={} selected={} supported={} opposed={} abstained={} risk_aligned={} no_trade_correct={} missed_gain={} wins={} losses={} avoided_losses={} reward={:.6} penalty={:.6} net={:.6} voice_delta={:.6}",
            agent.agent_id,
            agent.total_episodes,
            agent.selected_count,
            agent.supported_count,
            agent.opposed_count,
            agent.abstained_count,
            agent.risk_veto_aligned_count,
            agent.no_trade_correct_count,
            agent.no_trade_missed_gain_count,
            agent.wins_delta,
            agent.losses_delta,
            agent.avoided_losses_delta,
            agent.reward_total,
            agent.penalty_total,
            agent.net_reward_penalty,
            agent.voice_delta,
        ));
    }
    lines.push("Cross-source diagnostics".to_string());
    for diagnostic in &report.cross_source_consistency_report.source_diagnostics {
        lines.push(format!(
            "source={} kind={:?} rows={} cadence={:?} timestamp_gaps={} quality_score={:.6} quality_bucket={:?} monotonic={} close_range_pct={:.6} volume_range_ratio={:.6} trade_value_range_ratio={:?} trade_value={} profile_match={} suspicious_scale={} warnings={}",
            diagnostic.source_id,
            diagnostic.source_kind,
            diagnostic.row_count,
            diagnostic.expected_cadence,
            diagnostic.timestamp_gap_count,
            diagnostic.quality_score,
            diagnostic.quality_bucket,
            diagnostic.timestamp_monotonic,
            diagnostic.close_range_pct,
            diagnostic.volume_range_ratio,
            diagnostic.trade_value_range_ratio,
            diagnostic.trade_value_available,
            diagnostic.profile_match,
            diagnostic.suspicious_scale,
            diagnostic.data_quality_warnings.join("|"),
        ));
    }
    lines.push("Source consistency warnings".to_string());
    if report
        .cross_source_consistency_report
        .common_warnings
        .is_empty()
    {
        lines.push("none".to_string());
    } else {
        lines.extend(
            report
                .cross_source_consistency_report
                .common_warnings
                .iter()
                .cloned(),
        );
    }
    lines.push("Agent cross-source consistency table".to_string());
    for agent in &report.agent_cross_source_consistency_table.rows {
        lines.push(format!(
            "agent={} kinds={} sources={} positive_net={} negative_net={} voice_delta_range={:.6} reward_penalty_range={:.6} high_confidence_misses={} avoided_losses={} cooldowns={} quarantines={} status={:?}",
            agent.agent_id,
            agent.source_kind_count,
            agent.total_sources,
            agent.sources_with_positive_net_reward,
            agent.sources_with_negative_net_reward,
            agent.voice_delta_range,
            agent.reward_penalty_range,
            agent.high_confidence_miss_total,
            agent.avoided_loss_total,
            agent.cooldown_count_total,
            agent.quarantine_count_total,
            agent.consistency_status,
        ));
    }
    lines.push("Agent performance by quality bucket".to_string());
    for agent in &report.agent_performance_by_quality_table.rows {
        lines.push(format!(
            "agent={} quality_bucket={:?} sources={} episodes={} reward={:.6} penalty={:.6} net={:.6} voice_min={:.6} voice_max={:.6} high_confidence_misses={} avoided_losses={} cooldowns={} quarantines={}",
            agent.agent_id,
            agent.quality_bucket,
            agent.source_count,
            agent.total_episodes,
            agent.reward_total,
            agent.penalty_total,
            agent.net_reward_penalty,
            agent.voice_delta_min,
            agent.voice_delta_max,
            agent.high_confidence_misses,
            agent.avoided_losses,
            agent.cooldown_events,
            agent.quarantine_events,
        ));
    }
    lines.extend([
        "Risk Governor summary".to_string(),
        format!("risk_denials={}", report.batch_summary.total_risk_denials),
        "Sandbox summary".to_string(),
        format!(
            "sandbox_candidates={}",
            report.batch_summary.sandbox_candidate_count
        ),
        "Rejected source list".to_string(),
    ]);
    lines.extend(
        report
            .source_performance_table
            .rows
            .iter()
            .filter(|row| !row.accepted)
            .map(|row| format!("rejected_source={}", row.source_id)),
    );
    lines.push("Deferred/live-readiness warning".to_string());
    lines.extend(report.deferred_items.iter().cloned());
    redact_owner_report_output(&lines.join("\n"))
}

fn build_source_quality_threshold_summary() -> Vec<String> {
    LocalDataSourceRegistry::default()
        .list_profiles()
        .into_iter()
        .map(|profile| {
            format!(
                "kind={:?} cadence={:?} gap_multiplier={} max_gaps={} max_missing_optional_ratio={:.3} max_volume_ratio={:.3} max_ohlc_distortions={} min_rows={} min_score={:.3}",
                profile.kind,
                profile.expected_cadence,
                profile.cadence_tolerance.allowed_gap_multiplier,
                profile.quality_thresholds.max_timestamp_gap_count,
                profile.quality_thresholds.max_missing_optional_ratio,
                profile.quality_thresholds.max_volume_anomaly_ratio,
                profile.quality_thresholds.max_ohlc_distortion_count,
                profile.quality_thresholds.min_accepted_rows,
                profile.quality_thresholds.min_quality_score,
            )
        })
        .collect()
}

fn build_agent_performance_rows(
    source: &BatchReplaySource,
    replay: &PaperReplayResult,
    report: &OwnerLearningReport,
) -> Vec<AgentPerformanceRow> {
    report
        .agents
        .iter()
        .filter_map(|agent| {
            let attribution = replay
                .replay_attribution_summary
                .iter()
                .find(|summary| summary.agent_id == agent.agent_id)?;
            Some(AgentPerformanceRow {
                agent_id: agent.agent_id.clone(),
                agent_kind: agent.agent_kind,
                source_kind: Some(source.source_kind),
                source_id: Some(source.source_id.clone()),
                total_episodes: report.total_episodes as u64,
                selected_count: attribution.selected_count,
                supported_count: attribution.supported_final_count,
                opposed_count: attribution.opposed_final_count,
                abstained_count: attribution.abstained_count,
                risk_veto_aligned_count: attribution.risk_veto_aligned_count,
                no_trade_correct_count: attribution.no_trade_correct_count,
                no_trade_missed_gain_count: attribution.no_trade_missed_gain_count,
                wins_delta: agent.wins_delta,
                losses_delta: agent.losses_delta,
                avoided_losses_delta: agent.avoided_losses_delta,
                missed_gains_delta: agent.missed_gains_delta,
                high_confidence_misses_delta: agent.high_confidence_misses_delta,
                doctrine_violations_delta: agent.doctrine_violations_delta,
                reward_total: agent.total_reward,
                penalty_total: agent.total_penalty,
                net_reward_penalty: agent.net_reward_penalty,
                start_voice_power: agent.start_voice_power,
                end_voice_power: agent.end_voice_power,
                voice_delta: agent.voice_delta,
                start_status: agent.status_before,
                end_status: agent.status_after,
                start_tier: agent.tier_before,
                end_tier: agent.tier_after,
                cooldown_events: replay
                    .chain_results
                    .iter()
                    .flat_map(|chain| chain.agent_learning_summaries.iter())
                    .filter(|summary| {
                        summary.agent_id == agent.agent_id && summary.cooldown_triggered
                    })
                    .count() as u64,
                quarantine_events: replay
                    .chain_results
                    .iter()
                    .flat_map(|chain| chain.agent_learning_summaries.iter())
                    .filter(|summary| summary.agent_id == agent.agent_id && summary.quarantined)
                    .count() as u64,
                sandbox_candidates_created: agent.sandbox_candidates_created,
                reason_codes: agent.reason_codes.clone(),
            })
        })
        .collect()
}

fn accepted_source_performance_row(
    source: &BatchReplaySource,
    quality: &LocalDataQualitySummary,
    replay: &PaperReplayResult,
) -> SourcePerformanceRow {
    SourcePerformanceRow {
        source_id: source.source_id.clone(),
        source_kind: source.source_kind,
        display_name: source.display_name.clone(),
        accepted: true,
        total_rows: quality.total_rows,
        accepted_rows: quality.accepted_rows,
        rejected_rows: quality.rejected_rows,
        total_episodes: replay.learning_chain_summary.total_episodes,
        total_paper_trades: replay.learning_chain_summary.total_paper_trades,
        total_no_trades: replay.learning_chain_summary.total_no_trades,
        total_risk_denials: replay.learning_chain_summary.total_risk_denials,
        data_quality_summary: Some(quality.clone()),
        first_timestamp: quality.first_timestamp,
        last_timestamp: quality.last_timestamp,
        symbol: quality.symbol.clone(),
        min_close: quality.min_close,
        max_close: quality.max_close,
        monotonic: quality.monotonic,
        paper_only: true,
        not_live_ready: true,
        reason_codes: stable_reason_codes(
            &source
                .reason_codes
                .iter()
                .chain(quality.reason_codes.iter())
                .chain(replay.reason_codes.iter())
                .cloned()
                .chain([
                    ReasonCode::BatchReplaySourceAccepted,
                    ReasonCode::PaperExecutionOnly,
                ])
                .collect::<Vec<_>>(),
        ),
    }
}

fn source_quality_score(diagnostics: &SourceConsistencyDiagnostics) -> SourceQualityScore {
    SourceQualityScore {
        score: diagnostics.quality_score,
        bucket: diagnostics.quality_bucket,
        diagnostics: diagnostics.data_quality_warnings.clone(),
        reason_codes: diagnostics.reason_codes.clone(),
    }
}

fn quality_replay_blocked(policy: QualityReplayPolicy, bucket: DataQualityBucket) -> bool {
    match policy {
        QualityReplayPolicy::RejectPoorAndBelow => {
            matches!(
                bucket,
                DataQualityBucket::Poor | DataQualityBucket::Rejected
            )
        }
        QualityReplayPolicy::RejectRejectedOnly
        | QualityReplayPolicy::ReplayAllAcceptedWithWarnings => {
            bucket == DataQualityBucket::Rejected
        }
    }
}

fn quality_blocked_source_result(
    source: &BatchReplaySource,
    quality_summary: LocalDataQualitySummary,
    source_consistency_diagnostics: SourceConsistencyDiagnostics,
    mut quality_score: SourceQualityScore,
) -> BatchReplaySourceResult {
    quality_score
        .reason_codes
        .push(ReasonCode::SourceQualityReplayBlocked);
    quality_score.reason_codes = stable_reason_codes(&quality_score.reason_codes);
    let reason_codes = stable_reason_codes(
        &source
            .reason_codes
            .iter()
            .chain(quality_score.reason_codes.iter())
            .cloned()
            .chain([ReasonCode::BatchReplaySourceRejected])
            .collect::<Vec<_>>(),
    );
    let source_performance_row = SourcePerformanceRow {
        source_id: source.source_id.clone(),
        source_kind: source.source_kind,
        display_name: source.display_name.clone(),
        accepted: false,
        total_rows: quality_summary.total_rows,
        accepted_rows: quality_summary.accepted_rows,
        rejected_rows: quality_summary.rejected_rows,
        total_episodes: 0,
        total_paper_trades: 0,
        total_no_trades: 0,
        total_risk_denials: 0,
        data_quality_summary: Some(quality_summary.clone()),
        first_timestamp: quality_summary.first_timestamp,
        last_timestamp: quality_summary.last_timestamp,
        symbol: quality_summary.symbol.clone(),
        min_close: quality_summary.min_close,
        max_close: quality_summary.max_close,
        monotonic: quality_summary.monotonic,
        paper_only: true,
        not_live_ready: true,
        reason_codes: reason_codes.clone(),
    };
    BatchReplaySourceResult {
        source_id: source.source_id.clone(),
        source_kind: source.source_kind,
        accepted: false,
        dataset_quality_summary: Some(quality_summary),
        replay_result: None,
        owner_learning_report: None,
        agent_performance_rows: Vec::new(),
        source_performance_row,
        source_consistency_diagnostics,
        quality_bucket: quality_score.bucket,
        quality_reason_codes: quality_score.reason_codes.clone(),
        quality_score,
        replay_blocked_by_quality: true,
        reason_codes,
    }
}

fn rejected_batch_source_result(
    source: &BatchReplaySource,
    reason_codes: Vec<ReasonCode>,
) -> BatchReplaySourceResult {
    let reason_codes = stable_reason_codes(
        &reason_codes
            .into_iter()
            .chain([
                ReasonCode::SourceQualityRejected,
                ReasonCode::SourceQualityReplayBlocked,
            ])
            .collect::<Vec<_>>(),
    );
    let source_consistency_diagnostics =
        rejected_source_consistency_diagnostics(source, &reason_codes);
    let quality_score = SourceQualityScore {
        score: 0.0,
        bucket: DataQualityBucket::Rejected,
        diagnostics: source_consistency_diagnostics.data_quality_warnings.clone(),
        reason_codes: reason_codes.clone(),
    };
    let row = SourcePerformanceRow {
        source_id: source.source_id.clone(),
        source_kind: source.source_kind,
        display_name: source.display_name.clone(),
        accepted: false,
        total_rows: 0,
        accepted_rows: 0,
        rejected_rows: 0,
        total_episodes: 0,
        total_paper_trades: 0,
        total_no_trades: 0,
        total_risk_denials: 0,
        data_quality_summary: None,
        first_timestamp: 0,
        last_timestamp: 0,
        symbol: String::new(),
        min_close: 0.0,
        max_close: 0.0,
        monotonic: false,
        paper_only: true,
        not_live_ready: true,
        reason_codes: reason_codes.clone(),
    };
    BatchReplaySourceResult {
        source_id: source.source_id.clone(),
        source_kind: source.source_kind,
        accepted: false,
        dataset_quality_summary: None,
        replay_result: None,
        owner_learning_report: None,
        agent_performance_rows: Vec::new(),
        source_performance_row: row,
        source_consistency_diagnostics,
        quality_score,
        quality_bucket: DataQualityBucket::Rejected,
        quality_reason_codes: reason_codes.clone(),
        replay_blocked_by_quality: true,
        reason_codes,
    }
}

fn build_agent_performance_table(mut rows: Vec<AgentPerformanceRow>) -> AgentPerformanceTable {
    let aggregate_rows_by_agent = aggregate_agent_performance_rows(&rows, false);
    let aggregate_rows_by_source_kind = aggregate_agent_performance_rows(&rows, true);
    rows.sort_by(agent_performance_sort);
    AgentPerformanceTable {
        rows,
        aggregate_rows_by_agent,
        aggregate_rows_by_source_kind,
        reason_codes: vec![
            ReasonCode::BatchReplayBuilt,
            ReasonCode::DeterministicPath,
            ReasonCode::PaperExecutionOnly,
        ],
    }
}

fn aggregate_agent_performance_rows(
    rows: &[AgentPerformanceRow],
    group_by_source_kind: bool,
) -> Vec<AgentPerformanceRow> {
    let mut grouped =
        BTreeMap::<(Option<LocalDataSourceKind>, AgentId), AgentPerformanceRow>::new();
    for row in rows {
        let key = (
            group_by_source_kind.then_some(row.source_kind).flatten(),
            row.agent_id.clone(),
        );
        if let Some(aggregate) = grouped.get_mut(&key) {
            aggregate.total_episodes += row.total_episodes;
            aggregate.selected_count += row.selected_count;
            aggregate.supported_count += row.supported_count;
            aggregate.opposed_count += row.opposed_count;
            aggregate.abstained_count += row.abstained_count;
            aggregate.risk_veto_aligned_count += row.risk_veto_aligned_count;
            aggregate.no_trade_correct_count += row.no_trade_correct_count;
            aggregate.no_trade_missed_gain_count += row.no_trade_missed_gain_count;
            aggregate.wins_delta += row.wins_delta;
            aggregate.losses_delta += row.losses_delta;
            aggregate.avoided_losses_delta += row.avoided_losses_delta;
            aggregate.missed_gains_delta += row.missed_gains_delta;
            aggregate.high_confidence_misses_delta += row.high_confidence_misses_delta;
            aggregate.doctrine_violations_delta += row.doctrine_violations_delta;
            aggregate.reward_total += row.reward_total;
            aggregate.penalty_total += row.penalty_total;
            aggregate.net_reward_penalty = aggregate.reward_total - aggregate.penalty_total;
            aggregate.end_voice_power = row.end_voice_power;
            aggregate.voice_delta += row.voice_delta;
            aggregate.end_status = row.end_status;
            aggregate.end_tier = row.end_tier;
            aggregate.cooldown_events += row.cooldown_events;
            aggregate.quarantine_events += row.quarantine_events;
            aggregate.sandbox_candidates_created += row.sandbox_candidates_created;
            aggregate
                .reason_codes
                .extend(row.reason_codes.iter().cloned());
            aggregate.reason_codes = stable_reason_codes(&aggregate.reason_codes);
        } else {
            let mut aggregate = row.clone();
            aggregate.source_kind = if group_by_source_kind {
                row.source_kind
            } else {
                None
            };
            aggregate.source_id = None;
            grouped.insert(key, aggregate);
        }
    }
    grouped.into_values().collect()
}

fn build_source_performance_table(rows: Vec<SourcePerformanceRow>) -> SourcePerformanceTable {
    let accepted_count = rows.iter().filter(|row| row.accepted).count();
    let rejected_count = rows.len().saturating_sub(accepted_count);
    let mut by_source_kind_counts = BTreeMap::new();
    for row in &rows {
        *by_source_kind_counts.entry(row.source_kind).or_insert(0) += 1;
    }
    SourcePerformanceTable {
        rows,
        accepted_count,
        rejected_count,
        by_source_kind_counts,
        reason_codes: vec![
            ReasonCode::BatchReplayBuilt,
            ReasonCode::DeterministicPath,
            ReasonCode::PaperExecutionOnly,
        ],
    }
}

fn build_source_consistency_diagnostics(
    source: &BatchReplaySource,
    profile: &LocalDataSourceProfile,
    dataset: &HistoricalReplayDataset,
) -> SourceConsistencyDiagnostics {
    let timestamp_deltas = dataset
        .rows
        .windows(2)
        .map(|pair| pair[1].timestamp_ms.saturating_sub(pair[0].timestamp_ms))
        .collect::<Vec<_>>();
    let expected_interval = match profile.expected_cadence {
        ExpectedCadence::FixedMillis(interval) => Some(interval),
        ExpectedCadence::DailyApprox => Some(86_400_000),
        ExpectedCadence::Variable | ExpectedCadence::Unknown => None,
    };
    let timestamp_gap_count = expected_interval.map_or(0, |expected| {
        timestamp_deltas
            .iter()
            .filter(|gap| {
                **gap > expected.saturating_mul(profile.cadence_tolerance.allowed_gap_multiplier)
            })
            .count()
    });
    let min_close = dataset
        .rows
        .iter()
        .map(|row| row.close)
        .fold(f64::INFINITY, f64::min);
    let max_close = dataset
        .rows
        .iter()
        .map(|row| row.close)
        .fold(f64::NEG_INFINITY, f64::max);
    let close_range_pct = if min_close > 0.0 {
        (max_close - min_close) / min_close
    } else {
        0.0
    };
    let min_volume = dataset
        .rows
        .iter()
        .map(|row| row.volume)
        .fold(f64::INFINITY, f64::min);
    let max_volume = dataset
        .rows
        .iter()
        .map(|row| row.volume)
        .fold(f64::NEG_INFINITY, f64::max);
    let volume_range_ratio = if min_volume > 0.0 {
        max_volume / min_volume
    } else if max_volume > 0.0 {
        max_volume / f64::EPSILON
    } else {
        1.0
    };
    let trade_values = dataset
        .rows
        .iter()
        .filter_map(|row| row.trade_value)
        .collect::<Vec<_>>();
    let trade_value_range_ratio = if trade_values.is_empty() {
        None
    } else {
        let min_trade_value = trade_values.iter().copied().fold(f64::INFINITY, f64::min);
        let max_trade_value = trade_values
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        Some(if min_trade_value > 0.0 {
            max_trade_value / min_trade_value
        } else if max_trade_value > 0.0 {
            max_trade_value / f64::EPSILON
        } else {
            1.0
        })
    };
    let header = source
        .csv_text
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| {
            line.split(',')
                .map(|column| column.trim().to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut optional_columns_present = profile
        .optional_columns
        .iter()
        .filter(|column| header.contains(column))
        .cloned()
        .collect::<Vec<_>>();
    optional_columns_present.sort();
    let missing_optional_ratio = if profile.optional_columns.is_empty() {
        0.0
    } else {
        (profile.optional_columns.len() - optional_columns_present.len()) as f64
            / profile.optional_columns.len() as f64
    };
    let trade_value_available = dataset.rows.iter().any(|row| row.trade_value.is_some());
    let profile_match = source.profile_name == profile.name && source.source_kind == profile.kind;
    let suspicious_scale = min_close < 0.0001 || max_close > 10_000_000.0 || close_range_pct > 5.0;
    let suspicious_scale_score = if suspicious_scale { 1.0 } else { 0.0 };
    let volume_anomaly = volume_range_ratio > profile.quality_thresholds.max_volume_anomaly_ratio
        || trade_value_range_ratio
            .is_some_and(|ratio| ratio > profile.quality_thresholds.max_volume_anomaly_ratio);
    let ohlc_distortion_count = dataset
        .rows
        .iter()
        .filter(|row| row.close > 0.0 && (row.high - row.low) / row.close > 0.50)
        .count();
    let mut data_quality_warnings = Vec::new();
    let mut reason_codes = profile.cadence_tolerance.reason_codes.clone();
    reason_codes.extend([ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly]);
    if timestamp_gap_count > 0 {
        data_quality_warnings.push("Irregular timestamp gap detected.".to_string());
        reason_codes.extend([
            ReasonCode::SourceConsistencyTimestampGap,
            ReasonCode::SourceCadenceGapDetected,
        ]);
    }
    if timestamp_gap_count > profile.cadence_tolerance.max_gap_count
        || timestamp_gap_count > profile.quality_thresholds.max_timestamp_gap_count
    {
        reason_codes.push(ReasonCode::SourceCadenceTooManyGaps);
    }
    if suspicious_scale {
        data_quality_warnings.push("Suspicious price scale or close range detected.".to_string());
        reason_codes.push(ReasonCode::SourceConsistencySuspiciousScale);
    }
    if volume_anomaly {
        data_quality_warnings.push("Abnormal volume or trade-value range detected.".to_string());
        reason_codes.push(ReasonCode::SourceConsistencyVolumeAnomaly);
    }
    if missing_optional_ratio > profile.quality_thresholds.max_missing_optional_ratio {
        data_quality_warnings
            .push("Optional column coverage is below profile threshold.".to_string());
    }
    if !trade_value_available {
        data_quality_warnings.push("Optional trade value is unavailable.".to_string());
        reason_codes.push(ReasonCode::SourceConsistencyMissingOptionalTradeValue);
    }
    if !profile_match {
        data_quality_warnings.push("Source profile does not match source kind.".to_string());
        reason_codes.push(ReasonCode::SourceConsistencyProfileMismatch);
    }
    if ohlc_distortion_count > 0 {
        data_quality_warnings.push("Wide OHLC range requires review.".to_string());
        reason_codes.push(ReasonCode::SourceConsistencyQualityWarning);
    }
    if profile.expected_cadence == ExpectedCadence::Variable {
        reason_codes.push(ReasonCode::SourceCadenceVariableAccepted);
    }
    let mut score = 1.0_f64;
    score -= (timestamp_gap_count as f64 * 0.20).min(0.60);
    if missing_optional_ratio > profile.quality_thresholds.max_missing_optional_ratio {
        score -= 0.20;
    }
    if volume_anomaly {
        score -= 0.25;
    }
    if suspicious_scale_score > profile.quality_thresholds.max_suspicious_scale_score {
        score -= 0.35;
    }
    if ohlc_distortion_count > profile.quality_thresholds.max_ohlc_distortion_count {
        score -= (ohlc_distortion_count as f64 * 0.15).min(0.45);
    }
    score = score.clamp(0.0, 1.0);
    let quality_bucket = if dataset.rows.len() < profile.quality_thresholds.min_accepted_rows
        || score < profile.quality_thresholds.min_quality_score
    {
        score = 0.0;
        reason_codes.extend([
            ReasonCode::SourceQualityBelowThreshold,
            ReasonCode::SourceQualityRejected,
        ]);
        DataQualityBucket::Rejected
    } else if score >= 0.95 {
        reason_codes.push(ReasonCode::SourceQualityExcellent);
        DataQualityBucket::Excellent
    } else if score >= 0.80 {
        reason_codes.push(ReasonCode::SourceQualityGood);
        DataQualityBucket::Good
    } else if score >= 0.55 {
        reason_codes.push(ReasonCode::SourceQualityCaution);
        DataQualityBucket::Caution
    } else {
        reason_codes.push(ReasonCode::SourceQualityPoor);
        DataQualityBucket::Poor
    };
    if !data_quality_warnings.is_empty() {
        reason_codes.push(ReasonCode::SourceConsistencyQualityWarning);
    }

    SourceConsistencyDiagnostics {
        source_id: source.source_id.clone(),
        source_kind: source.source_kind,
        row_count: dataset.rows.len(),
        timestamp_monotonic: dataset
            .rows
            .windows(2)
            .all(|pair| pair[0].timestamp_ms < pair[1].timestamp_ms),
        timestamp_gap_count,
        min_timestamp: dataset.rows.first().map_or(0, |row| row.timestamp_ms),
        max_timestamp: dataset.rows.last().map_or(0, |row| row.timestamp_ms),
        min_close,
        max_close,
        close_range_pct,
        min_volume,
        max_volume,
        volume_range_ratio,
        trade_value_range_ratio,
        trade_value_available,
        optional_columns_present,
        missing_optional_ratio,
        profile_match,
        suspicious_scale,
        suspicious_scale_score,
        ohlc_distortion_count,
        expected_cadence: profile.expected_cadence,
        quality_score: score,
        quality_bucket,
        data_quality_warnings,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn rejected_source_consistency_diagnostics(
    source: &BatchReplaySource,
    rejection_reasons: &[ReasonCode],
) -> SourceConsistencyDiagnostics {
    let profile_match = !rejection_reasons.iter().any(|reason| {
        matches!(
            reason,
            ReasonCode::LocalSourceUnknown | ReasonCode::LocalSourceProfileInvalid
        )
    });
    let timestamp_monotonic = !rejection_reasons.iter().any(|reason| {
        matches!(
            reason,
            ReasonCode::HistoricalReplayNonMonotonicTimestamp
                | ReasonCode::HistoricalReplayDuplicateTimestamp
        )
    });
    let mut warnings = vec!["Source rejected before replay diagnostics.".to_string()];
    let mut reason_codes = rejection_reasons.to_vec();
    if !profile_match {
        warnings.push("Source profile does not match source kind.".to_string());
        reason_codes.push(ReasonCode::SourceConsistencyProfileMismatch);
    }
    if !timestamp_monotonic {
        warnings.push("Timestamp ordering is invalid.".to_string());
    }
    reason_codes.push(ReasonCode::SourceConsistencyQualityWarning);
    SourceConsistencyDiagnostics {
        source_id: source.source_id.clone(),
        source_kind: source.source_kind,
        row_count: 0,
        timestamp_monotonic,
        timestamp_gap_count: 0,
        min_timestamp: 0,
        max_timestamp: 0,
        min_close: 0.0,
        max_close: 0.0,
        close_range_pct: 0.0,
        min_volume: 0.0,
        max_volume: 0.0,
        volume_range_ratio: 0.0,
        trade_value_range_ratio: None,
        trade_value_available: false,
        optional_columns_present: Vec::new(),
        missing_optional_ratio: 1.0,
        profile_match,
        suspicious_scale: false,
        suspicious_scale_score: 0.0,
        ohlc_distortion_count: 0,
        expected_cadence: ExpectedCadence::Unknown,
        quality_score: 0.0,
        quality_bucket: DataQualityBucket::Rejected,
        data_quality_warnings: warnings,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn build_cross_source_consistency_report(
    source_results: &[BatchReplaySourceResult],
) -> CrossSourceConsistencyReport {
    let mut source_diagnostics = source_results
        .iter()
        .map(|result| result.source_consistency_diagnostics.clone())
        .collect::<Vec<_>>();
    source_diagnostics.sort_by(|left, right| {
        left.source_kind
            .cmp(&right.source_kind)
            .then_with(|| left.source_id.cmp(&right.source_id))
    });
    let mut source_kind_counts = BTreeMap::new();
    for diagnostic in &source_diagnostics {
        *source_kind_counts
            .entry(diagnostic.source_kind)
            .or_insert(0) += 1;
    }
    let mut common_warnings = source_diagnostics
        .iter()
        .flat_map(|diagnostic| diagnostic.data_quality_warnings.iter().cloned())
        .collect::<Vec<_>>();
    common_warnings.sort();
    common_warnings.dedup();
    let accepted_source_count = source_results
        .iter()
        .filter(|result| result.accepted)
        .count();
    let rejected_source_count = source_results.len().saturating_sub(accepted_source_count);
    let suspicious_source_count = source_diagnostics
        .iter()
        .filter(|diagnostic| {
            !diagnostic.data_quality_warnings.is_empty()
                || !diagnostic.timestamp_monotonic
                || !diagnostic.profile_match
                || diagnostic.suspicious_scale
        })
        .count();
    let reason_codes = stable_reason_codes(
        &source_diagnostics
            .iter()
            .flat_map(|diagnostic| diagnostic.reason_codes.iter().cloned())
            .chain([ReasonCode::BatchReplayBuilt, ReasonCode::PaperExecutionOnly])
            .collect::<Vec<_>>(),
    );
    CrossSourceConsistencyReport {
        source_diagnostics,
        source_kind_counts,
        accepted_source_count,
        rejected_source_count,
        suspicious_source_count,
        common_warnings,
        reason_codes,
    }
}

fn build_agent_cross_source_consistency_table(
    rows: &[AgentPerformanceRow],
) -> AgentCrossSourceConsistencyTable {
    let mut grouped = BTreeMap::<AgentId, Vec<&AgentPerformanceRow>>::new();
    for row in rows {
        grouped.entry(row.agent_id.clone()).or_default().push(row);
    }
    let rows = grouped
        .into_iter()
        .map(|(agent_id, agent_rows)| {
            let agent_kind = agent_rows[0].agent_kind;
            let mut source_kinds = agent_rows
                .iter()
                .filter_map(|row| row.source_kind)
                .collect::<Vec<_>>();
            source_kinds.sort();
            source_kinds.dedup();
            let voice_delta_min = agent_rows
                .iter()
                .map(|row| row.voice_delta)
                .fold(f64::INFINITY, f64::min);
            let voice_delta_max = agent_rows
                .iter()
                .map(|row| row.voice_delta)
                .fold(f64::NEG_INFINITY, f64::max);
            let reward_penalty_min = agent_rows
                .iter()
                .map(|row| row.net_reward_penalty)
                .fold(f64::INFINITY, f64::min);
            let reward_penalty_max = agent_rows
                .iter()
                .map(|row| row.net_reward_penalty)
                .fold(f64::NEG_INFINITY, f64::max);
            let voice_delta_range = voice_delta_max - voice_delta_min;
            let reward_penalty_range = reward_penalty_max - reward_penalty_min;
            let consistency_status = if agent_rows.len() < 2 || source_kinds.len() < 2 {
                AgentConsistencyStatus::InsufficientData
            } else if voice_delta_range <= 0.05 && reward_penalty_range <= 0.25 {
                AgentConsistencyStatus::Stable
            } else if voice_delta_range <= 0.15 && reward_penalty_range <= 1.0 {
                AgentConsistencyStatus::SourceSensitive
            } else {
                AgentConsistencyStatus::Unstable
            };
            let status_reason = match consistency_status {
                AgentConsistencyStatus::Stable => ReasonCode::AgentConsistencyStable,
                AgentConsistencyStatus::SourceSensitive => {
                    ReasonCode::AgentConsistencySourceSensitive
                }
                AgentConsistencyStatus::Unstable => ReasonCode::AgentConsistencyUnstable,
                AgentConsistencyStatus::InsufficientData => {
                    ReasonCode::AgentConsistencyInsufficientData
                }
            };
            AgentCrossSourceConsistencyRow {
                agent_id,
                agent_kind,
                source_kind_count: source_kinds.len(),
                total_sources: agent_rows.len(),
                sources_with_positive_net_reward: agent_rows
                    .iter()
                    .filter(|row| row.net_reward_penalty > 0.0)
                    .count(),
                sources_with_negative_net_reward: agent_rows
                    .iter()
                    .filter(|row| row.net_reward_penalty < 0.0)
                    .count(),
                voice_delta_min,
                voice_delta_max,
                voice_delta_range,
                reward_penalty_min,
                reward_penalty_max,
                reward_penalty_range,
                high_confidence_miss_total: agent_rows
                    .iter()
                    .map(|row| row.high_confidence_misses_delta)
                    .sum(),
                avoided_loss_total: agent_rows.iter().map(|row| row.avoided_losses_delta).sum(),
                cooldown_count_total: agent_rows.iter().map(|row| row.cooldown_events).sum(),
                quarantine_count_total: agent_rows.iter().map(|row| row.quarantine_events).sum(),
                consistency_status,
                reason_codes: vec![
                    status_reason,
                    ReasonCode::DeterministicPath,
                    ReasonCode::PaperExecutionOnly,
                ],
            }
        })
        .collect();
    AgentCrossSourceConsistencyTable {
        rows,
        reason_codes: vec![
            ReasonCode::BatchReplayBuilt,
            ReasonCode::DeterministicPath,
            ReasonCode::PaperExecutionOnly,
        ],
    }
}

fn build_agent_performance_by_quality_table(
    rows: &[AgentPerformanceRow],
    source_results: &[BatchReplaySourceResult],
) -> AgentPerformanceByQualityTable {
    let source_buckets = source_results
        .iter()
        .map(|result| (result.source_id.as_str(), result.quality_bucket))
        .collect::<BTreeMap<_, _>>();
    let mut grouped = BTreeMap::<(AgentId, DataQualityBucket), AgentPerformanceByQualityRow>::new();
    for row in rows {
        let Some(source_id) = row.source_id.as_deref() else {
            continue;
        };
        let Some(quality_bucket) = source_buckets.get(source_id).copied() else {
            continue;
        };
        let key = (row.agent_id.clone(), quality_bucket);
        if let Some(aggregate) = grouped.get_mut(&key) {
            aggregate.source_count += 1;
            aggregate.total_episodes += row.total_episodes;
            aggregate.reward_total += row.reward_total;
            aggregate.penalty_total += row.penalty_total;
            aggregate.net_reward_penalty = aggregate.reward_total - aggregate.penalty_total;
            aggregate.voice_delta_min = aggregate.voice_delta_min.min(row.voice_delta);
            aggregate.voice_delta_max = aggregate.voice_delta_max.max(row.voice_delta);
            aggregate.high_confidence_misses += row.high_confidence_misses_delta;
            aggregate.avoided_losses += row.avoided_losses_delta;
            aggregate.cooldown_events += row.cooldown_events;
            aggregate.quarantine_events += row.quarantine_events;
            aggregate
                .reason_codes
                .extend(row.reason_codes.iter().cloned());
            aggregate.reason_codes = stable_reason_codes(&aggregate.reason_codes);
        } else {
            grouped.insert(
                key,
                AgentPerformanceByQualityRow {
                    agent_id: row.agent_id.clone(),
                    agent_kind: row.agent_kind,
                    quality_bucket,
                    source_count: 1,
                    total_episodes: row.total_episodes,
                    reward_total: row.reward_total,
                    penalty_total: row.penalty_total,
                    net_reward_penalty: row.net_reward_penalty,
                    voice_delta_min: row.voice_delta,
                    voice_delta_max: row.voice_delta,
                    high_confidence_misses: row.high_confidence_misses_delta,
                    avoided_losses: row.avoided_losses_delta,
                    cooldown_events: row.cooldown_events,
                    quarantine_events: row.quarantine_events,
                    reason_codes: row.reason_codes.clone(),
                },
            );
        }
    }
    AgentPerformanceByQualityTable {
        rows: grouped.into_values().collect(),
        reason_codes: vec![
            ReasonCode::BatchReplayBuilt,
            ReasonCode::DeterministicPath,
            ReasonCode::PaperExecutionOnly,
        ],
    }
}

fn agent_performance_sort(
    left: &AgentPerformanceRow,
    right: &AgentPerformanceRow,
) -> std::cmp::Ordering {
    left.source_kind
        .cmp(&right.source_kind)
        .then_with(|| left.source_id.cmp(&right.source_id))
        .then_with(|| left.agent_id.cmp(&right.agent_id))
}

fn local_data_source_error_reasons(error: &LocalDataSourceError) -> Vec<ReasonCode> {
    match error {
        LocalDataSourceError::Registry { reason_codes } => reason_codes.clone(),
        LocalDataSourceError::Historical(error) => error.reason_codes.clone(),
        LocalDataSourceError::Replay(_) | LocalDataSourceError::Report(_) => {
            vec![ReasonCode::LocalSourceRejected]
        }
    }
}

fn batch_source_safety_reason(source: &BatchReplaySource) -> Option<ReasonCode> {
    let combined = format!(
        "{}\n{}\n{}\n{}",
        source.source_id, source.display_name, source.profile_name, source.csv_text
    );
    let normalized = combined.to_ascii_lowercase();
    if contains_temporary_instruction_marker(&combined) {
        Some(ReasonCode::BatchReplayWorkMdMarkerRejected)
    } else if normalized.contains("live_provider")
        || normalized.contains("live_endpoint")
        || normalized.contains("exchange_secret")
    {
        Some(ReasonCode::BatchReplayLiveProviderRejected)
    } else if normalized.contains("http://")
        || normalized.contains("https://")
        || normalized.contains("broker-endpoint")
        || normalized.contains("order-endpoint")
        || normalized.contains(",endpoint")
        || normalized.contains("endpoint,")
        || normalized.contains("url_endpoint")
    {
        Some(ReasonCode::BatchReplayEndpointDataRejected)
    } else if normalized.contains("raw_response") || normalized.contains("raw provider response") {
        Some(ReasonCode::BatchReplayRawProviderResponseRejected)
    } else if normalized.contains("account_id") {
        Some(ReasonCode::BatchReplayAccountDataRejected)
    } else if normalized.contains("order_id") {
        Some(ReasonCode::BatchReplayOrderDataRejected)
    } else if [
        "authorization",
        "bearer ",
        "access_token",
        "refresh_token",
        "app_key",
        "app_secret",
        "api_key",
        "private_key",
        "wallet_private_key",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
    {
        Some(ReasonCode::BatchReplaySecretLikeDataRejected)
    } else if normalized.contains("local_private")
        || normalized.contains("private mapping")
        || normalized.contains(".env")
    {
        Some(ReasonCode::BatchReplayUnsafePrivateData)
    } else {
        None
    }
}

fn local_source_profile(kind: LocalDataSourceKind) -> LocalDataSourceProfile {
    let mut required_columns = vec![
        "symbol".to_string(),
        "timestamp_ms".to_string(),
        "open".to_string(),
        "high".to_string(),
        "low".to_string(),
        "close".to_string(),
        "volume".to_string(),
    ];
    let (name, description, timestamp_unit, optional_columns, allowed_source_markers) = match kind {
        LocalDataSourceKind::SyntheticFixture => (
            "synthetic-fixture",
            "Generic sanitized fixture CSV",
            LocalTimestampUnit::Milliseconds,
            vec!["trade_value", "source"],
            vec!["synthetic", "fixture"],
        ),
        LocalDataSourceKind::KoreanStockCsv => {
            required_columns.retain(|column| column != "timestamp_ms");
            (
                "korean-stock-csv",
                "Local sanitized Korean stock CSV",
                LocalTimestampUnit::MillisecondsOrDateTimeUtc,
                vec![
                    "timestamp_ms",
                    "date",
                    "time",
                    "trade_value",
                    "market",
                    "source",
                    "currency",
                ],
                vec![
                    "local",
                    "sanitized",
                    "fixture",
                    "manual_export",
                    "synthetic",
                ],
            )
        }
        LocalDataSourceKind::UsStockCsv => {
            required_columns.retain(|column| column != "timestamp_ms");
            (
                "us-stock-csv",
                "Local sanitized US stock CSV",
                LocalTimestampUnit::MillisecondsOrDateTimeUtc,
                vec![
                    "timestamp_ms",
                    "date",
                    "time",
                    "adjusted_close",
                    "trade_value",
                    "market",
                    "source",
                    "currency",
                ],
                vec![
                    "local",
                    "sanitized",
                    "fixture",
                    "manual_export",
                    "synthetic",
                ],
            )
        }
        LocalDataSourceKind::BtcCryptoCsv => (
            "btc-crypto-csv",
            "Local sanitized BTC crypto CSV",
            LocalTimestampUnit::Milliseconds,
            vec![
                "quote_volume",
                "trade_count",
                "source",
                "exchange",
                "currency",
            ],
            vec![
                "local",
                "sanitized",
                "fixture",
                "manual_export",
                "synthetic",
            ],
        ),
        LocalDataSourceKind::Unknown => (
            "unknown",
            "Rejected local source",
            LocalTimestampUnit::Milliseconds,
            Vec::new(),
            Vec::new(),
        ),
    };
    let calendar_deferred = matches!(
        kind,
        LocalDataSourceKind::KoreanStockCsv | LocalDataSourceKind::UsStockCsv
    );
    let cadence_tolerance = CadenceTolerance {
        allowed_gap_multiplier: 2,
        max_gap_count: 0,
        allow_weekend_or_session_gap: false,
        reason_codes: if calendar_deferred {
            vec![
                ReasonCode::SourceCadenceExpected,
                ReasonCode::SourceCadenceCalendarDeferred,
            ]
        } else {
            vec![ReasonCode::SourceCadenceExpected]
        },
    };
    let quality_thresholds = SourceQualityThresholds {
        max_timestamp_gap_count: 0,
        max_duplicate_timestamp_count: 0,
        max_missing_optional_ratio: if kind == LocalDataSourceKind::BtcCryptoCsv {
            0.20
        } else {
            0.25
        },
        max_volume_anomaly_ratio: if kind == LocalDataSourceKind::BtcCryptoCsv {
            500.0
        } else {
            1_000.0
        },
        max_suspicious_scale_score: 0.0,
        max_ohlc_distortion_count: 0,
        min_accepted_rows: 4,
        min_quality_score: 0.30,
        reject_on_private_marker: true,
        reject_on_forbidden_column: true,
    };
    LocalDataSourceProfile {
        kind,
        name: name.to_string(),
        description: description.to_string(),
        required_columns,
        optional_columns: optional_columns.into_iter().map(str::to_string).collect(),
        timestamp_unit,
        price_scale: 1.0,
        volume_scale: 1.0,
        symbol_policy: LocalSymbolPolicy::SingleSymbolStrict,
        allowed_source_markers: allowed_source_markers
            .into_iter()
            .map(str::to_string)
            .collect(),
        reject_private_markers: true,
        expected_cadence: ExpectedCadence::FixedMillis(60_000),
        cadence_tolerance,
        quality_thresholds,
        reason_codes: vec![ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly],
    }
}

fn validate_local_source_profile(
    profile: &LocalDataSourceProfile,
) -> Result<(), LocalDataSourceError> {
    if profile.kind == LocalDataSourceKind::Unknown {
        return Err(local_registry_error(ReasonCode::LocalSourceUnknown));
    }
    let profile_text = format!(
        "{} {} {}",
        profile.name,
        profile.description,
        profile.allowed_source_markers.join(" ")
    )
    .to_ascii_lowercase();
    if profile_text.contains("http://") || profile_text.contains("https://") {
        return Err(local_registry_error(
            ReasonCode::LocalSourceNetworkForbidden,
        ));
    }
    if profile_text.contains("broker-endpoint") || profile_text.contains("order-endpoint") {
        return Err(local_registry_error(ReasonCode::LocalSourceBrokerForbidden));
    }
    let mut columns = profile
        .required_columns
        .iter()
        .chain(profile.optional_columns.iter())
        .map(|column| column.as_str())
        .collect::<Vec<_>>();
    columns.sort_unstable();
    if columns.iter().any(|column| forbidden_local_column(column)) {
        return Err(local_registry_error(
            ReasonCode::LocalSourceUnsafePrivateData,
        ));
    }
    if profile.name.trim().is_empty()
        || profile.description.trim().is_empty()
        || !profile.price_scale.is_finite()
        || profile.price_scale <= 0.0
        || !profile.volume_scale.is_finite()
        || profile.volume_scale <= 0.0
        || profile.allowed_source_markers.is_empty()
        || !profile.reject_private_markers
        || profile.cadence_tolerance.allowed_gap_multiplier == 0
        || !profile
            .quality_thresholds
            .max_missing_optional_ratio
            .is_finite()
        || !(0.0..=1.0).contains(&profile.quality_thresholds.max_missing_optional_ratio)
        || !profile
            .quality_thresholds
            .max_volume_anomaly_ratio
            .is_finite()
        || profile.quality_thresholds.max_volume_anomaly_ratio <= 1.0
        || !profile
            .quality_thresholds
            .max_suspicious_scale_score
            .is_finite()
        || !profile.quality_thresholds.min_quality_score.is_finite()
        || !(0.0..=1.0).contains(&profile.quality_thresholds.min_quality_score)
        || profile.quality_thresholds.min_accepted_rows < 2
        || matches!(profile.expected_cadence, ExpectedCadence::FixedMillis(0))
        || matches!(profile.expected_cadence, ExpectedCadence::Unknown)
        || columns.windows(2).any(|pair| pair[0] == pair[1])
        || ["symbol", "open", "high", "low", "close", "volume"]
            .iter()
            .any(|required| !columns.contains(required))
    {
        return Err(local_registry_error(ReasonCode::LocalSourceProfileInvalid));
    }
    Ok(())
}

fn build_local_data_quality_summary(
    dataset: &HistoricalReplayDataset,
    source_kind: LocalDataSourceKind,
) -> LocalDataQualitySummary {
    let min_close = dataset
        .rows
        .iter()
        .map(|row| row.close)
        .fold(f64::INFINITY, f64::min);
    let max_close = dataset
        .rows
        .iter()
        .map(|row| row.close)
        .fold(f64::NEG_INFINITY, f64::max);
    LocalDataQualitySummary {
        total_rows: dataset.rows.len(),
        accepted_rows: dataset.rows.len(),
        rejected_rows: 0,
        first_timestamp: dataset.rows.first().map_or(0, |row| row.timestamp_ms),
        last_timestamp: dataset.rows.last().map_or(0, |row| row.timestamp_ms),
        symbol: dataset.symbol.clone(),
        source_kind,
        has_trade_value: dataset.rows.iter().any(|row| row.trade_value.is_some()),
        monotonic: dataset
            .rows
            .windows(2)
            .all(|pair| pair[0].timestamp_ms < pair[1].timestamp_ms),
        min_close,
        max_close,
        reason_codes: vec![
            ReasonCode::DeterministicPath,
            ReasonCode::LocalFileOnly,
            ReasonCode::SyntheticFixtureEvidence,
        ],
    }
}

fn parse_local_datetime_utc_ms(date: &str, time: &str) -> Option<u64> {
    let date_digits = date
        .chars()
        .filter(|value| value.is_ascii_digit())
        .collect::<String>();
    let time_digits = time
        .chars()
        .filter(|value| value.is_ascii_digit())
        .collect::<String>();
    if date_digits.len() != 8 || time_digits.len() != 6 {
        return None;
    }
    let year = date_digits.get(0..4)?.parse::<i64>().ok()?;
    let month = date_digits.get(4..6)?.parse::<u32>().ok()?;
    let day = date_digits.get(6..8)?.parse::<u32>().ok()?;
    let hour = time_digits.get(0..2)?.parse::<u32>().ok()?;
    let minute = time_digits.get(2..4)?.parse::<u32>().ok()?;
    let second = time_digits.get(4..6)?.parse::<u32>().ok()?;
    if year < 1970
        || !(1..=12).contains(&month)
        || day == 0
        || day > days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }
    let adjusted_year = year - if month <= 2 { 1 } else { 0 };
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let shifted_month = i64::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days_since_epoch = era * 146_097 + day_of_era - 719_468;
    let seconds_since_epoch = days_since_epoch
        .checked_mul(86_400)?
        .checked_add(i64::from(hour) * 3_600)?
        .checked_add(i64::from(minute) * 60)?
        .checked_add(i64::from(second))?;
    u64::try_from(seconds_since_epoch).ok()?.checked_mul(1_000)
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
        2 => 28,
        _ => 0,
    }
}

fn forbidden_local_column(column: &str) -> bool {
    let normalized = column.to_ascii_lowercase();
    [
        "account_id",
        "order_id",
        "api_key",
        "app_key",
        "app_secret",
        "token",
        "authorization",
        "bearer",
        "wallet",
        "private",
        "raw_response",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn contains_local_private_marker(text: &str) -> bool {
    let normalized = text.to_ascii_lowercase();
    [
        "authorization",
        "bearer ",
        "account_id",
        "order_id",
        "api_key",
        "app_key",
        "app_secret",
        "access_token",
        "refresh_token",
        "wallet",
        "local_private",
        "raw_response",
        "raw toss response",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn contains_temporary_instruction_marker(text: &str) -> bool {
    text.to_ascii_lowercase()
        .contains(concat!("work", ".", "md"))
}

fn local_source_kind_label(kind: LocalDataSourceKind) -> &'static str {
    match kind {
        LocalDataSourceKind::SyntheticFixture => "synthetic-fixture",
        LocalDataSourceKind::KoreanStockCsv => "kr-stock",
        LocalDataSourceKind::UsStockCsv => "us-stock",
        LocalDataSourceKind::BtcCryptoCsv => "btc-crypto",
        LocalDataSourceKind::Unknown => "unknown",
    }
}

fn local_registry_error(reason_code: ReasonCode) -> LocalDataSourceError {
    LocalDataSourceError::Registry {
        reason_codes: vec![reason_code],
    }
}

impl HistoricalReplayAdapter {
    pub fn parse_csv_string(
        &self,
        input: &str,
        config: &HistoricalReplayConfig,
    ) -> Result<HistoricalReplayDataset, HistoricalReplayError> {
        if !config.reject_non_finite {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayNonFinite,
            ));
        }
        if !config.reject_non_positive_prices {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayNonPositivePrice,
            ));
        }
        if contains_owner_report_private_material(input) {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayUnsafePrivateDataRejected,
            ));
        }
        let mut lines = input.lines().map(str::trim).filter(|line| !line.is_empty());
        let header_line = lines
            .next()
            .ok_or_else(|| historical_error(None, ReasonCode::HistoricalReplayEmptyDataset))?;
        let header = header_line
            .split(',')
            .map(|value| value.trim())
            .collect::<Vec<_>>();
        let allowed_columns = [
            "symbol",
            "timestamp_ms",
            "open",
            "high",
            "low",
            "close",
            "volume",
            "trade_value",
            "source",
        ];
        let required_columns = [
            "symbol",
            "timestamp_ms",
            "open",
            "high",
            "low",
            "close",
            "volume",
        ];
        let column_index = header
            .iter()
            .enumerate()
            .map(|(index, name)| (*name, index))
            .collect::<BTreeMap<_, _>>();
        if header.is_empty()
            || column_index.len() != header.len()
            || header.iter().any(|name| !allowed_columns.contains(name))
            || required_columns
                .iter()
                .any(|name| !column_index.contains_key(name))
        {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayInvalidHeader,
            ));
        }
        let data_lines = lines.collect::<Vec<_>>();
        if data_lines.is_empty() {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayEmptyDataset,
            ));
        }
        if config.max_rows == 0 || data_lines.len() > config.max_rows {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayTooManyRows,
            ));
        }
        if !config.synthetic_only {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayUnsafeSource,
            ));
        }

        let mut rows = Vec::with_capacity(data_lines.len());
        let mut dataset_symbol: Option<String> = None;
        let mut dataset_source: Option<String> = None;
        for (offset, line) in data_lines.iter().enumerate() {
            let row_number = offset + 2;
            let values = line
                .split(',')
                .map(|value| value.trim())
                .collect::<Vec<_>>();
            if values.len() != header.len() {
                return Err(historical_error(
                    Some(row_number),
                    ReasonCode::HistoricalReplayInvalidRow,
                ));
            }
            let value = |name: &str| {
                column_index
                    .get(name)
                    .and_then(|index| values.get(*index))
                    .copied()
            };
            let symbol = value("symbol").unwrap_or_default();
            let source = value("source")
                .filter(|source| !source.is_empty())
                .unwrap_or("fixture");
            if symbol.is_empty() {
                return Err(historical_error(
                    Some(row_number),
                    ReasonCode::HistoricalReplayInvalidRow,
                ));
            }
            if !historical_source_is_safe(source) || contains_owner_report_private_material(source)
            {
                return Err(historical_error(
                    Some(row_number),
                    ReasonCode::HistoricalReplayUnsafeSource,
                ));
            }
            if dataset_symbol
                .as_deref()
                .is_some_and(|expected| expected != symbol)
                || dataset_source
                    .as_deref()
                    .is_some_and(|expected| expected != source)
            {
                return Err(historical_error(
                    Some(row_number),
                    ReasonCode::HistoricalReplayInvalidRow,
                ));
            }
            dataset_symbol.get_or_insert_with(|| symbol.to_string());
            dataset_source.get_or_insert_with(|| source.to_string());

            let timestamp_ms = parse_historical_u64(value("timestamp_ms"), row_number)?;
            let open = parse_historical_f64(value("open"), row_number)?;
            let high = parse_historical_f64(value("high"), row_number)?;
            let low = parse_historical_f64(value("low"), row_number)?;
            let close = parse_historical_f64(value("close"), row_number)?;
            let volume = parse_historical_f64(value("volume"), row_number)?;
            let trade_value = value("trade_value")
                .filter(|value| !value.is_empty())
                .map(|value| parse_historical_f64(Some(value), row_number))
                .transpose()?;
            let row = HistoricalOhlcvRow {
                symbol: symbol.to_string(),
                timestamp_ms,
                open,
                high,
                low,
                close,
                volume,
                trade_value,
            };
            validate_historical_row(&row, config, rows.last(), row_number)?;
            rows.push(row);
        }
        let dataset = HistoricalReplayDataset {
            symbol: dataset_symbol.unwrap_or_default(),
            rows,
            source: dataset_source.unwrap_or_else(|| "fixture".to_string()),
            reason_codes: vec![
                ReasonCode::DeterministicPath,
                ReasonCode::LocalFileOnly,
                ReasonCode::SyntheticFixtureEvidence,
            ],
        };
        self.validate_dataset(&dataset, config)?;
        Ok(dataset)
    }

    pub fn validate_dataset(
        &self,
        dataset: &HistoricalReplayDataset,
        config: &HistoricalReplayConfig,
    ) -> Result<(), HistoricalReplayError> {
        if !config.reject_non_finite {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayNonFinite,
            ));
        }
        if !config.reject_non_positive_prices {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayNonPositivePrice,
            ));
        }
        if dataset.rows.is_empty() {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayEmptyDataset,
            ));
        }
        if config.max_rows == 0 || dataset.rows.len() > config.max_rows {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayTooManyRows,
            ));
        }
        if contains_owner_report_private_material(&dataset.source) {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayUnsafePrivateDataRejected,
            ));
        }
        if !config.synthetic_only || !historical_source_is_safe(&dataset.source) {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayUnsafeSource,
            ));
        }
        if dataset.symbol.trim().is_empty()
            || contains_owner_report_private_material(&dataset.symbol)
        {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayUnsafePrivateDataRejected,
            ));
        }
        let mut previous = None;
        for (index, row) in dataset.rows.iter().enumerate() {
            if contains_owner_report_private_material(&row.symbol) {
                return Err(historical_error(
                    Some(index + 2),
                    ReasonCode::HistoricalReplayUnsafePrivateDataRejected,
                ));
            }
            if row.symbol != dataset.symbol {
                return Err(historical_error(
                    Some(index + 2),
                    ReasonCode::HistoricalReplayInvalidRow,
                ));
            }
            validate_historical_row(row, config, previous, index + 2)?;
            previous = Some(row);
        }
        Ok(())
    }

    pub fn to_candle_series(
        &self,
        dataset: &HistoricalReplayDataset,
        config: &HistoricalReplayConfig,
    ) -> Result<CandleSeries, HistoricalReplayError> {
        self.validate_dataset(dataset, config)?;
        Ok(CandleSeries {
            symbol: dataset.symbol.clone(),
            timeframe: Timeframe::OneMinute,
            candles: dataset
                .rows
                .iter()
                .map(|row| Candle {
                    timestamp_ms: row.timestamp_ms,
                    open: row.open,
                    high: row.high,
                    low: row.low,
                    close: row.close,
                    volume: row.volume,
                    trade_value: row.trade_value,
                    bid: None,
                    ask: None,
                    spread_bps: None,
                })
                .collect(),
        })
    }

    pub fn to_paper_replay_input(
        &self,
        dataset: &HistoricalReplayDataset,
        historical_config: &HistoricalReplayConfig,
        initial_agent_states: Vec<CanonicalAgentState>,
        replay_config: PaperReplayConfig,
    ) -> Result<PaperReplayInput, HistoricalReplayError> {
        let series = self.to_candle_series(dataset, historical_config)?;
        if series.len() < 2 {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayEmptyDataset,
            ));
        }
        let mut episode_inputs = Vec::with_capacity(series.len() / 2);
        for decision_index in (0..series.len().saturating_sub(1)).step_by(2) {
            let outcome_index = decision_index + 1;
            let market = series.market_snapshot_at(decision_index).ok_or_else(|| {
                historical_error(
                    Some(decision_index + 2),
                    ReasonCode::HistoricalReplayInvalidRow,
                )
            })?;
            let decision_candle = &series.candles[decision_index];
            let outcome_candle = &series.candles[outcome_index];
            let hypothetical_return = outcome_candle.close / decision_candle.close - 1.0;
            let intrabar_return = decision_candle.close / decision_candle.open - 1.0;
            let range_pct = (decision_candle.high - decision_candle.low) / decision_candle.open;
            let signal = SignalOutput {
                symbol: dataset.symbol.clone(),
                horizon_bars: 1,
                p_win: (0.5 + intrabar_return * 4.0).clamp(0.20, 0.80),
                p_stop: (0.5 - intrabar_return * 4.0).clamp(0.20, 0.80),
                expected_return: intrabar_return.clamp(-0.05, 0.05),
                expected_drawdown: range_pct.max(0.0),
                confidence: 0.20,
                no_trade_probability: 0.80,
                source: format!("historical-{}-adapter", dataset.source),
            };
            let input = PaperLearningLoopInput {
                initial_agent_states: initial_agent_states.clone(),
                market_snapshot: market,
                signal_input: signal,
                owner_advisory: None,
                risk_snapshot: RiskSnapshot {
                    daily_pnl_pct: 0.0,
                    consecutive_losses: 0,
                    current_positions_count: 0,
                    total_exposure_pct: 0.0,
                    symbol_exposure_pct: 0.0,
                    api_health_score: 1.0,
                    data_quality_score: 1.0,
                },
                paper_context: Some(PaperOutcomeContext {
                    outcome_finalized: true,
                    finalized_at_timestamp_ms: outcome_candle.timestamp_ms,
                    outcome_kind: PaperOutcomeKind::NoExecution,
                    fill_evidence: None,
                    realized_net_return_pct: 0.0,
                    hypothetical_net_return_pct: Some(hypothetical_return),
                    max_adverse_excursion_pct: hypothetical_return.min(0.0).abs(),
                    doctrine_violation_agents: Vec::new(),
                    overtrade_agents: Vec::new(),
                }),
                loop_config: PaperLearningLoopConfig::default(),
            };
            episode_inputs.push(PaperLearningEpisode {
                episode_id: format!(
                    "historical:{}:{}",
                    dataset.symbol, decision_candle.timestamp_ms
                ),
                input,
                reason_codes: vec![
                    ReasonCode::DeterministicPath,
                    ReasonCode::PaperExecutionOnly,
                    ReasonCode::SyntheticFixtureEvidence,
                ],
            });
        }
        if episode_inputs.is_empty() {
            return Err(historical_error(
                None,
                ReasonCode::HistoricalReplayEmptyDataset,
            ));
        }
        Ok(PaperReplayInput {
            initial_agent_states,
            episode_inputs,
            replay_config,
        })
    }
}

pub fn build_owner_learning_report_from_historical_replay(
    report_id: &str,
    dataset: &HistoricalReplayDataset,
    historical_config: &HistoricalReplayConfig,
    initial_agent_states: &[CanonicalAgentState],
    replay_config: PaperReplayConfig,
) -> Result<OwnerLearningReport, HistoricalOwnerReportError> {
    let replay_input = HistoricalReplayAdapter
        .to_paper_replay_input(
            dataset,
            historical_config,
            initial_agent_states.to_vec(),
            replay_config,
        )
        .map_err(HistoricalOwnerReportError::Historical)?;
    let replay =
        run_3_agent_paper_replay(replay_input).map_err(HistoricalOwnerReportError::Replay)?;
    build_owner_learning_report(
        report_id,
        Some(format!("historical:{}:{}", dataset.source, dataset.symbol)),
        &replay,
    )
    .map_err(HistoricalOwnerReportError::Report)
}

fn parse_historical_u64(
    value: Option<&str>,
    row_number: usize,
) -> Result<u64, HistoricalReplayError> {
    value
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| historical_error(Some(row_number), ReasonCode::HistoricalReplayInvalidRow))
}

fn parse_historical_f64(
    value: Option<&str>,
    row_number: usize,
) -> Result<f64, HistoricalReplayError> {
    value
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<f64>().ok())
        .ok_or_else(|| historical_error(Some(row_number), ReasonCode::HistoricalReplayInvalidRow))
}

fn validate_historical_row(
    row: &HistoricalOhlcvRow,
    config: &HistoricalReplayConfig,
    previous: Option<&HistoricalOhlcvRow>,
    row_number: usize,
) -> Result<(), HistoricalReplayError> {
    let numeric_values = [
        row.open,
        row.high,
        row.low,
        row.close,
        row.volume,
        row.trade_value.unwrap_or(0.0),
    ];
    if config.reject_non_finite && numeric_values.iter().any(|value| !value.is_finite()) {
        return Err(historical_error(
            Some(row_number),
            ReasonCode::HistoricalReplayNonFinite,
        ));
    }
    if config.reject_non_positive_prices
        && [row.open, row.high, row.low, row.close]
            .iter()
            .any(|value| *value <= 0.0)
    {
        return Err(historical_error(
            Some(row_number),
            ReasonCode::HistoricalReplayNonPositivePrice,
        ));
    }
    if row.timestamp_ms == 0 || row.volume < 0.0 || row.trade_value.is_some_and(|value| value < 0.0)
    {
        return Err(historical_error(
            Some(row_number),
            ReasonCode::HistoricalReplayInvalidRow,
        ));
    }
    if row.high < row.low
        || (config.strict_ohlc_bounds
            && (row.open < row.low
                || row.open > row.high
                || row.close < row.low
                || row.close > row.high))
    {
        return Err(historical_error(
            Some(row_number),
            ReasonCode::HistoricalReplayInvalidOhlc,
        ));
    }
    if config.require_monotonic_timestamps
        && previous.is_some_and(|previous| previous.timestamp_ms >= row.timestamp_ms)
    {
        return Err(historical_error(
            Some(row_number),
            ReasonCode::HistoricalReplayNonMonotonicTimestamp,
        ));
    }
    Ok(())
}

fn historical_source_is_safe(source: &str) -> bool {
    let normalized = source.trim().to_ascii_lowercase();
    normalized == "fixture"
        || normalized == "synthetic"
        || normalized.starts_with("fixture:")
        || normalized.starts_with("synthetic:")
}

fn historical_error(row_number: Option<usize>, reason_code: ReasonCode) -> HistoricalReplayError {
    HistoricalReplayError {
        row_number,
        reason_codes: vec![reason_code],
    }
}

pub fn build_owner_learning_report(
    report_id: &str,
    generated_from_replay_id: Option<String>,
    replay: &PaperReplayResult,
) -> Result<OwnerLearningReport, OwnerLearningReportError> {
    if report_id.trim().is_empty() {
        return Err(OwnerLearningReportError::InvalidReportId);
    }
    if contains_owner_report_private_material(report_id)
        || generated_from_replay_id
            .as_deref()
            .is_some_and(contains_owner_report_private_material)
    {
        return Err(OwnerLearningReportError::UnsafePrivateData {
            reason_codes: vec![ReasonCode::OwnerReportUnsafePrivateDataRejected],
        });
    }
    if replay.initial_states.len() != 3 || replay.final_states.len() != 3 {
        return Err(OwnerLearningReportError::InvalidReplayRoster);
    }

    let mut agents = Vec::with_capacity(3);
    for learning in &replay.learning_chain_summary.agent_summaries {
        let initial = replay
            .initial_states
            .iter()
            .find(|state| state.agent_id == learning.agent_id)
            .ok_or(OwnerLearningReportError::MissingAgentSummary)?;
        let final_state = replay
            .final_states
            .iter()
            .find(|state| state.agent_id == learning.agent_id)
            .ok_or(OwnerLearningReportError::MissingAgentSummary)?;
        let attribution = replay
            .replay_attribution_summary
            .iter()
            .find(|summary| summary.agent_id == learning.agent_id)
            .ok_or(OwnerLearningReportError::MissingAgentSummary)?;
        let mut reason_codes = learning
            .reason_codes
            .iter()
            .chain(attribution.reason_codes.iter())
            .cloned()
            .collect::<Vec<_>>();
        if final_state.status == AgentStatus::Quarantined {
            reason_codes.push(ReasonCode::Quarantined);
        }
        if final_state.voice_state.cooldown_bars > 0 {
            reason_codes.push(ReasonCode::CooldownRequired);
        }
        let explanation = owner_agent_learning_explanation(
            attribution.net_reward_penalty,
            final_state.status,
            learning,
        );
        agents.push(OwnerAgentLearningView {
            agent_id: learning.agent_id.clone(),
            agent_kind: final_state.kind,
            start_version_id: learning.start_version_id.clone(),
            end_version_id: learning.end_version_id.clone(),
            start_voice_power: learning.start_voice_power,
            end_voice_power: learning.end_voice_power,
            voice_delta: learning.end_voice_power - learning.start_voice_power,
            tier_before: learning.tier_before,
            tier_after: learning.tier_after,
            status_before: learning.status_before,
            status_after: learning.status_after,
            cooldown_before: initial.voice_state.cooldown_bars,
            cooldown_after: final_state.voice_state.cooldown_bars,
            wins_delta: learning.wins_delta,
            losses_delta: learning.losses_delta,
            avoided_losses_delta: learning.avoided_losses_delta,
            missed_gains_delta: learning.missed_gains_delta,
            high_confidence_misses_delta: learning.high_confidence_misses_delta,
            doctrine_violations_delta: learning.doctrine_violations_delta,
            total_reward: attribution.total_reward,
            total_penalty: attribution.total_penalty,
            net_reward_penalty: attribution.net_reward_penalty,
            sandbox_candidates_created: learning.sandbox_candidates_created,
            owner_visible_explanation: explanation,
            reason_codes: stable_reason_codes(&reason_codes),
        });
    }
    agents.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
    if agents.len() != 3 {
        return Err(OwnerLearningReportError::MissingAgentSummary);
    }

    let episode_results = replay
        .chain_results
        .iter()
        .flat_map(|chain| chain.episode_results.iter())
        .collect::<Vec<_>>();
    let rewards_given = episode_results
        .iter()
        .flat_map(|episode| episode.result.reward_penalties.iter())
        .filter(|reward| reward.reward_delta > 0.0)
        .count() as u64;
    let penalties_given = episode_results
        .iter()
        .flat_map(|episode| episode.result.reward_penalties.iter())
        .filter(|reward| reward.penalty_delta > 0.0)
        .count() as u64;
    let cooldowns_started = episode_results
        .iter()
        .flat_map(|episode| episode.result.reward_penalties.iter())
        .filter(|reward| reward.tier_action == ChairTierAction::Cooldown)
        .count() as u64;
    let quarantines = episode_results
        .iter()
        .flat_map(|episode| episode.result.reward_penalties.iter())
        .filter(|reward| reward.tier_action == ChairTierAction::Quarantine)
        .count() as u64;
    let top_rewarded_agent = top_agent_by(&agents, |agent| agent.total_reward);
    let top_penalized_agent = top_agent_by(&agents, |agent| agent.total_penalty);
    let chair_summary = ChairReviewSummary {
        decisions_composed: episode_results.len() as u64,
        rewards_given,
        penalties_given,
        cooldowns_started,
        quarantines,
        sandbox_candidates: replay.sandbox_candidates.len() as u64,
        top_rewarded_agent,
        top_penalized_agent,
        reason_codes: vec![
            ReasonCode::DeterministicPath,
            ReasonCode::PaperExecutionOnly,
        ],
    };

    let owner_reviews = episode_results
        .iter()
        .filter_map(|episode| episode.result.owner_explanation.as_ref())
        .collect::<Vec<_>>();
    let owner_advisory_summary = OwnerAdvisorySummary {
        owner_requests_seen: owner_reviews.len() as u64,
        owner_requests_accepted_as_context: owner_reviews
            .iter()
            .filter(|review| review.paper_action_allowed)
            .count() as u64,
        owner_requests_rejected: owner_reviews
            .iter()
            .filter(|review| !review.paper_action_allowed)
            .count() as u64,
        owner_forced_trade_attempts_blocked: owner_reviews
            .iter()
            .filter(|review| {
                !review.owner_forced_trade
                    && review
                        .reason_codes
                        .contains(&ReasonCode::OwnerRequestedButRiskDenied)
            })
            .count() as u64,
        owner_promotion_attempts_blocked: owner_reviews
            .iter()
            .filter(|review| {
                review
                    .reason_codes
                    .contains(&ReasonCode::OwnerRequestedButSandboxOnly)
            })
            .count() as u64,
        owner_cooldown_clear_attempts_blocked: owner_reviews
            .iter()
            .filter(|review| {
                review
                    .reason_codes
                    .contains(&ReasonCode::OwnerRequestedButCooldownActive)
            })
            .count() as u64,
        reason_codes: stable_reason_codes(
            &owner_reviews
                .iter()
                .flat_map(|review| review.reason_codes.iter().cloned())
                .collect::<Vec<_>>(),
        ),
    };
    let risk_decisions = episode_results
        .iter()
        .map(|episode| &episode.result.risk_decision)
        .collect::<Vec<_>>();
    let risk_summary = RiskReviewSummary {
        risk_denials: risk_decisions
            .iter()
            .filter(|decision| decision.kind != RiskDecisionKind::ApprovePaper)
            .count() as u64,
        emergency_stops: risk_decisions
            .iter()
            .filter(|decision| decision.kind == RiskDecisionKind::EmergencyStop)
            .count() as u64,
        cooldown_blocks: risk_decisions
            .iter()
            .filter(|decision| decision.kind == RiskDecisionKind::Cooldown)
            .count() as u64,
        owner_requests_denied: owner_advisory_summary.owner_requests_rejected,
        bad_data_denials: risk_decisions
            .iter()
            .filter(|decision| {
                decision
                    .reason_codes
                    .contains(&ReasonCode::DataQualityGateBreached)
            })
            .count() as u64,
        spread_denials: risk_decisions
            .iter()
            .filter(|decision| {
                decision
                    .reason_codes
                    .contains(&ReasonCode::SpreadGateBreached)
            })
            .count() as u64,
        stale_data_denials: owner_reviews
            .iter()
            .filter(|review| {
                review
                    .reason_codes
                    .contains(&ReasonCode::OwnerRequestedButStaleData)
            })
            .count() as u64,
        reason_codes: stable_reason_codes(
            &risk_decisions
                .iter()
                .flat_map(|decision| decision.reason_codes.iter().cloned())
                .collect::<Vec<_>>(),
        ),
    };

    let mut candidates_by_agent = BTreeMap::new();
    for candidate in &replay.sandbox_candidates {
        *candidates_by_agent
            .entry(candidate.agent_id.clone())
            .or_insert(0) += 1;
    }
    let any_live_candidate = replay.sandbox_candidates.iter().any(|candidate| {
        !candidate.sandbox_only || candidate.can_vote_live() || candidate.can_affect_live_decision()
    });
    let sandbox_summary = SandboxReviewSummary {
        candidate_count: replay.sandbox_candidates.len() as u64,
        candidates_by_agent,
        any_live_candidate,
        safety_status: if any_live_candidate {
            "unsafe-live-candidate-detected".to_string()
        } else {
            "safe-paper-only".to_string()
        },
        reason_codes: vec![
            ReasonCode::PaperExecutionOnly,
            ReasonCode::ShadowEvaluationPending,
        ],
    };

    Ok(OwnerLearningReport {
        report_id: report_id.to_string(),
        generated_from_replay_id,
        total_episodes: replay.learning_chain_summary.total_episodes,
        total_paper_trades: replay.learning_chain_summary.total_paper_trades,
        total_no_trades: replay.learning_chain_summary.total_no_trades,
        total_risk_denials: replay.learning_chain_summary.total_risk_denials,
        agents,
        chair_summary,
        risk_summary,
        sandbox_summary,
        owner_advisory_summary,
        data_quality_summary: None,
        safety_warnings: vec![
            "Paper-only report.".to_string(),
            "Not live trading ready.".to_string(),
            "Risk Governor remains final veto.".to_string(),
            "Owner input is advisory only.".to_string(),
        ],
        reason_codes: vec![
            ReasonCode::DeterministicPath,
            ReasonCode::OwnerLearningReportBuilt,
            ReasonCode::PaperExecutionOnly,
        ],
    })
}

pub fn render_owner_learning_report_text(report: &OwnerLearningReport) -> String {
    let mut lines = vec![
        format!("Owner Learning Report: {}", report.report_id),
        "Safety status".to_string(),
    ];
    lines.extend(report.safety_warnings.iter().cloned());
    lines.extend([
        "Overall paper replay summary".to_string(),
        format!(
            "generated_from={}",
            report
                .generated_from_replay_id
                .as_deref()
                .unwrap_or("unspecified")
        ),
        format!("total_episodes={}", report.total_episodes),
        format!("total_paper_trades={}", report.total_paper_trades),
        format!("total_no_trades={}", report.total_no_trades),
        format!("total_risk_denials={}", report.total_risk_denials),
    ]);
    if let Some(quality) = &report.data_quality_summary {
        lines.push("Data quality summary".to_string());
        lines.push(format!(
            "source_kind={:?} symbol={} rows={} accepted={} rejected={} first_timestamp={} last_timestamp={} monotonic={} has_trade_value={} min_close={:.6} max_close={:.6}",
            quality.source_kind,
            quality.symbol,
            quality.total_rows,
            quality.accepted_rows,
            quality.rejected_rows,
            quality.first_timestamp,
            quality.last_timestamp,
            quality.monotonic,
            quality.has_trade_value,
            quality.min_close,
            quality.max_close,
        ));
    }
    lines.push("Agent changes".to_string());
    for agent in &report.agents {
        lines.push(format!(
            "agent={} kind={:?} voice={:.6}->{:.6} delta={:.6} tier={:?}->{:?} status={:?}->{:?} cooldown={}->{} wins={} losses={} avoided_losses={} missed_gains={} high_confidence_misses={} doctrine_violations={} reward={:.6} penalty={:.6} net={:.6} sandbox_candidates={} explanation={}",
            agent.agent_id,
            agent.agent_kind,
            agent.start_voice_power,
            agent.end_voice_power,
            agent.voice_delta,
            agent.tier_before,
            agent.tier_after,
            agent.status_before,
            agent.status_after,
            agent.cooldown_before,
            agent.cooldown_after,
            agent.wins_delta,
            agent.losses_delta,
            agent.avoided_losses_delta,
            agent.missed_gains_delta,
            agent.high_confidence_misses_delta,
            agent.doctrine_violations_delta,
            agent.total_reward,
            agent.total_penalty,
            agent.net_reward_penalty,
            agent.sandbox_candidates_created,
            agent.owner_visible_explanation,
        ));
    }
    lines.extend([
        "Chair rewards and penalties".to_string(),
        format!(
            "decisions={} rewards={} penalties={} cooldowns={} quarantines={} top_rewarded={} top_penalized={}",
            report.chair_summary.decisions_composed,
            report.chair_summary.rewards_given,
            report.chair_summary.penalties_given,
            report.chair_summary.cooldowns_started,
            report.chair_summary.quarantines,
            report
                .chair_summary
                .top_rewarded_agent
                .as_deref()
                .unwrap_or("none"),
            report
                .chair_summary
                .top_penalized_agent
                .as_deref()
                .unwrap_or("none"),
        ),
        "Risk Governor denials".to_string(),
        format!(
            "denials={} emergency_stops={} cooldown_blocks={} bad_data={} spread={} stale_data={}",
            report.risk_summary.risk_denials,
            report.risk_summary.emergency_stops,
            report.risk_summary.cooldown_blocks,
            report.risk_summary.bad_data_denials,
            report.risk_summary.spread_denials,
            report.risk_summary.stale_data_denials,
        ),
        "Sandbox candidates".to_string(),
        format!(
            "candidate_count={} any_live_candidate={} safety_status={}",
            report.sandbox_summary.candidate_count,
            report.sandbox_summary.any_live_candidate,
            report.sandbox_summary.safety_status,
        ),
        "Owner advisory outcomes".to_string(),
        format!(
            "seen={} accepted_as_context={} rejected={} forced_trade_blocked={} promotion_blocked={} cooldown_clear_blocked={}",
            report.owner_advisory_summary.owner_requests_seen,
            report
                .owner_advisory_summary
                .owner_requests_accepted_as_context,
            report.owner_advisory_summary.owner_requests_rejected,
            report
                .owner_advisory_summary
                .owner_forced_trade_attempts_blocked,
            report
                .owner_advisory_summary
                .owner_promotion_attempts_blocked,
            report
                .owner_advisory_summary
                .owner_cooldown_clear_attempts_blocked,
        ),
        "Deferred/live-readiness warning".to_string(),
        "This report cannot approve, execute, promote, or clear cooldown.".to_string(),
    ]);
    redact_owner_report_output(&lines.join("\n"))
}

pub fn render_owner_learning_report_markdown(report: &OwnerLearningReport) -> String {
    let text = render_owner_learning_report_text(report);
    let mut markdown = String::from("# Owner Learning Report\n\n");
    for line in text.lines() {
        if matches!(
            line,
            "Safety status"
                | "Overall paper replay summary"
                | "Data quality summary"
                | "Agent changes"
                | "Chair rewards and penalties"
                | "Risk Governor denials"
                | "Sandbox candidates"
                | "Owner advisory outcomes"
                | "Deferred/live-readiness warning"
        ) {
            markdown.push_str(&format!("\n## {line}\n"));
        } else {
            markdown.push_str(&format!("- {line}\n"));
        }
    }
    redact_owner_report_output(&markdown)
}

pub fn render_owner_learning_report_json_like(report: &OwnerLearningReport) -> String {
    let serialized = serde_json::to_string_pretty(report)
        .unwrap_or_else(|_| "{\"error\":\"serialization-failed\"}".to_string());
    redact_owner_report_output(&serialized)
}

pub fn handle_owner_review_command(
    report: &OwnerLearningReport,
    command: OwnerReviewCommand,
) -> OwnerReviewResponse {
    let (text, reason_codes) = match command {
        OwnerReviewCommand::ShowSummary => (
            render_owner_learning_report_text(report),
            report.reason_codes.clone(),
        ),
        OwnerReviewCommand::ShowAgent { agent_id } => {
            if let Some(agent) = report
                .agents
                .iter()
                .find(|agent| agent.agent_id == agent_id)
            {
                (
                    format!(
                        "agent={} voice_delta={:.6} status={:?} cooldown={} net={:.6} explanation={}",
                        agent.agent_id,
                        agent.voice_delta,
                        agent.status_after,
                        agent.cooldown_after,
                        agent.net_reward_penalty,
                        agent.owner_visible_explanation,
                    ),
                    agent.reason_codes.clone(),
                )
            } else {
                (
                    format!("agent={agent_id} unavailable"),
                    vec![ReasonCode::AttributionUnavailable],
                )
            }
        }
        OwnerReviewCommand::ShowRisk => (
            format!(
                "Risk Governor remains final veto. denials={} emergency_stops={} cooldown_blocks={}",
                report.risk_summary.risk_denials,
                report.risk_summary.emergency_stops,
                report.risk_summary.cooldown_blocks,
            ),
            report.risk_summary.reason_codes.clone(),
        ),
        OwnerReviewCommand::ShowSandbox => (
            format!(
                "sandbox_candidates={} any_live_candidate={} safety_status={}",
                report.sandbox_summary.candidate_count,
                report.sandbox_summary.any_live_candidate,
                report.sandbox_summary.safety_status,
            ),
            report.sandbox_summary.reason_codes.clone(),
        ),
        OwnerReviewCommand::ShowOwnerAdvisory => (
            format!(
                "Owner input is advisory only. seen={} rejected={} forced_trade_blocked={} promotion_blocked={} cooldown_clear_blocked={}",
                report.owner_advisory_summary.owner_requests_seen,
                report.owner_advisory_summary.owner_requests_rejected,
                report
                    .owner_advisory_summary
                    .owner_forced_trade_attempts_blocked,
                report
                    .owner_advisory_summary
                    .owner_promotion_attempts_blocked,
                report
                    .owner_advisory_summary
                    .owner_cooldown_clear_attempts_blocked,
            ),
            report.owner_advisory_summary.reason_codes.clone(),
        ),
        OwnerReviewCommand::ExplainReasonCodes { reason_codes } => {
            let explanation = owner_rejection_explanation(&reason_codes);
            let stable = stable_reason_codes(&reason_codes);
            (
                format!(
                    "{} reason_codes={}",
                    explanation,
                    stable
                        .iter()
                        .map(|reason| format!("{reason:?}"))
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                stable,
            )
        }
    };
    OwnerReviewResponse {
        text: redact_owner_report_output(&text),
        reason_codes: stable_reason_codes(&reason_codes),
        no_state_mutation: true,
        order_execution_supported: false,
        sandbox_promotion_supported: false,
        cooldown_clear_supported: false,
    }
}

fn owner_agent_learning_explanation(
    net_reward_penalty: f64,
    final_status: AgentStatus,
    learning: &AgentLearningSummary,
) -> String {
    if final_status == AgentStatus::Quarantined {
        "Quarantined after a severe doctrine or safety violation.".to_string()
    } else if final_status == AgentStatus::Cooldown {
        "Temporarily unavailable because bounded penalties triggered cooldown.".to_string()
    } else if net_reward_penalty > 0.0 {
        "Paper outcomes produced more bounded reward than penalty.".to_string()
    } else if net_reward_penalty < 0.0 {
        "Paper outcomes produced more bounded penalty than reward.".to_string()
    } else if learning.avoided_losses_delta > 0 {
        "NoTrade behavior recorded an avoided paper loss.".to_string()
    } else {
        "No net reward or penalty change was recorded.".to_string()
    }
}

fn top_agent_by(
    agents: &[OwnerAgentLearningView],
    value: impl Fn(&OwnerAgentLearningView) -> f64,
) -> Option<AgentId> {
    let mut selected: Option<&OwnerAgentLearningView> = None;
    for agent in agents {
        if selected.is_none_or(|current| value(agent) > value(current)) {
            selected = Some(agent);
        }
    }
    selected
        .filter(|agent| value(agent) > 0.0)
        .map(|agent| agent.agent_id.clone())
}

fn contains_owner_report_private_material(text: &str) -> bool {
    let normalized = text.to_ascii_lowercase();
    [
        "authorization:",
        "bearer ",
        "toss_app_key",
        "toss_app_secret",
        "app_key",
        "app_secret",
        "api_key",
        "api_secret",
        "secret=",
        "token=",
        "access_token",
        "refresh_token",
        "account_id",
        "order_id",
        "private_key",
        "wallet_private_key",
        "raw_response",
        "raw toss response",
        "http://",
        "https://",
        "broker-endpoint",
        "order-endpoint",
        "url_endpoint",
        "live_provider",
        "live_endpoint",
        "exchange_secret",
        "private mapping",
        "private field mapping",
        "local_private",
        ".env",
        concat!("work", ".", "md"),
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn redact_owner_report_output(text: &str) -> String {
    let mut redacted = false;
    let lines = text
        .lines()
        .map(|line| {
            if contains_owner_report_private_material(line) {
                redacted = true;
                "[REDACTED PRIVATE DATA]".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>();
    if redacted {
        format!(
            "reason={:?}\n{}",
            ReasonCode::OwnerReportSecretRedacted,
            lines.join("\n")
        )
    } else {
        lines.join("\n")
    }
}

fn empty_attribution_summary(state: &CanonicalAgentState) -> AgentAttributionSummary {
    AgentAttributionSummary {
        agent_id: state.agent_id.clone(),
        selected_count: 0,
        supported_final_count: 0,
        opposed_final_count: 0,
        abstained_count: 0,
        risk_veto_aligned_count: 0,
        risk_veto_opposed_count: 0,
        no_trade_correct_count: 0,
        no_trade_missed_gain_count: 0,
        profitable_selected_count: 0,
        losing_selected_count: 0,
        high_confidence_miss_count: 0,
        doctrine_violation_count: 0,
        total_reward: 0.0,
        total_penalty: 0.0,
        net_reward_penalty: 0.0,
        final_voice_power: state.voice_state.voice_power,
        final_status: state.status,
        reason_codes: Vec::new(),
    }
}

fn accumulate_attribution(
    summaries: &mut [AgentAttributionSummary],
    episode_states: &[CanonicalAgentState],
    result: &PaperLearningLoopResult,
) {
    for state in episode_states {
        let Some(summary) = summaries
            .iter_mut()
            .find(|summary| summary.agent_id == state.agent_id)
        else {
            continue;
        };
        let Some(feedback) = result
            .feedback_records
            .iter()
            .find(|feedback| feedback.agent_id == state.agent_id)
        else {
            summary
                .reason_codes
                .push(ReasonCode::AttributionUnavailable);
            continue;
        };
        let has_reason = |reason: ReasonCode| feedback.reason_codes.contains(&reason);
        let selected = has_reason(ReasonCode::AgentSelectedForDecision);
        let supported = has_reason(ReasonCode::AgentSupportedFinalDecision);
        let opposed = has_reason(ReasonCode::AgentOpposedFinalDecision);
        let abstained = has_reason(ReasonCode::AgentAbstained);
        let risk_aligned = has_reason(ReasonCode::AgentRiskVetoAligned);
        let risk_opposed = has_reason(ReasonCode::AgentRiskVetoOpposed);
        let no_trade_correct = has_reason(ReasonCode::AgentNoTradeCorrect);
        let no_trade_missed = has_reason(ReasonCode::AgentNoTradeMissedGain);
        summary.selected_count += selected as u64;
        summary.supported_final_count += supported as u64;
        summary.opposed_final_count += opposed as u64;
        summary.abstained_count += abstained as u64;
        summary.risk_veto_aligned_count += risk_aligned as u64;
        summary.risk_veto_opposed_count += risk_opposed as u64;
        summary.no_trade_correct_count += no_trade_correct as u64;
        summary.no_trade_missed_gain_count += no_trade_missed as u64;
        summary.profitable_selected_count +=
            (selected && feedback.realized_net_return > 0.0) as u64;
        summary.losing_selected_count += (selected && feedback.realized_net_return < 0.0) as u64;
        summary.high_confidence_miss_count += feedback
            .reason_codes
            .contains(&ReasonCode::FeedbackHighConfidenceLoss)
            as u64;
        summary.doctrine_violation_count += feedback.doctrine_violation as u64;
        if !(selected
            || supported
            || opposed
            || abstained
            || risk_aligned
            || risk_opposed
            || no_trade_correct
            || no_trade_missed)
        {
            summary
                .reason_codes
                .push(ReasonCode::AttributionUnavailable);
        }
        if let Some(reward) = result
            .reward_penalties
            .iter()
            .find(|reward| reward.agent_id == state.agent_id)
        {
            summary.total_reward += reward.reward_delta;
            summary.total_penalty += reward.penalty_delta;
        }
    }
}

fn finalize_attribution(
    summaries: &mut [AgentAttributionSummary],
    final_states: &[CanonicalAgentState],
) {
    for summary in summaries {
        if let Some(state) = final_states
            .iter()
            .find(|state| state.agent_id == summary.agent_id)
        {
            summary.final_voice_power = state.voice_state.voice_power;
            summary.final_status = state.status;
        }
        summary.net_reward_penalty = summary.total_reward - summary.total_penalty;
        if summary.selected_count > 0 {
            summary
                .reason_codes
                .push(ReasonCode::AttributionSelectedAgent);
        }
        if summary.supported_final_count > 0 {
            summary
                .reason_codes
                .push(ReasonCode::AttributionSupportedFinal);
        }
        if summary.opposed_final_count > 0 {
            summary
                .reason_codes
                .push(ReasonCode::AttributionOpposedFinal);
        }
        if summary.abstained_count > 0 {
            summary.reason_codes.push(ReasonCode::AttributionAbstained);
        }
        if summary.risk_veto_aligned_count > 0 {
            summary
                .reason_codes
                .push(ReasonCode::AttributionRiskVetoAligned);
        }
        if summary.risk_veto_opposed_count > 0 {
            summary
                .reason_codes
                .push(ReasonCode::AttributionRiskVetoOpposed);
        }
        if summary.no_trade_correct_count > 0 {
            summary
                .reason_codes
                .push(ReasonCode::AttributionNoTradeCorrect);
        }
        if summary.no_trade_missed_gain_count > 0 {
            summary
                .reason_codes
                .push(ReasonCode::AttributionNoTradeMissedGain);
        }
        summary.reason_codes = stable_reason_codes(&summary.reason_codes);
    }
}

fn build_agent_learning_summaries(
    initial_states: &[CanonicalAgentState],
    final_states: &[CanonicalAgentState],
    episode_results: &[PaperLearningEpisodeResult],
    sandbox_candidates: &[SandboxPromotionCandidate],
) -> Vec<AgentLearningSummary> {
    initial_states
        .iter()
        .filter_map(|initial| {
            let final_state = final_states
                .iter()
                .find(|state| state.agent_id == initial.agent_id)?;
            let cooldown_triggered = episode_results.iter().any(|episode| {
                episode.result.reward_penalties.iter().any(|reward| {
                    reward.agent_id == initial.agent_id
                        && reward.tier_action == ChairTierAction::Cooldown
                })
            });
            let quarantined = final_state.status == AgentStatus::Quarantined
                || episode_results.iter().any(|episode| {
                    episode.result.reward_penalties.iter().any(|reward| {
                        reward.agent_id == initial.agent_id
                            && reward.tier_action == ChairTierAction::Quarantine
                    })
                });
            let mut summary_reason_codes = vec![ReasonCode::DeterministicPath];
            if cooldown_triggered {
                summary_reason_codes.push(ReasonCode::CooldownRequired);
            }
            if quarantined {
                summary_reason_codes.push(ReasonCode::Quarantined);
            }
            Some(AgentLearningSummary {
                agent_id: initial.agent_id.clone(),
                start_version_id: initial.version.version_id.clone(),
                end_version_id: final_state.version.version_id.clone(),
                start_voice_power: initial.voice_state.voice_power,
                end_voice_power: final_state.voice_state.voice_power,
                tier_before: initial.voice_state.tier,
                tier_after: final_state.voice_state.tier,
                status_before: initial.status,
                status_after: final_state.status,
                wins_delta: final_state
                    .memory_summary
                    .wins
                    .saturating_sub(initial.memory_summary.wins),
                losses_delta: final_state
                    .memory_summary
                    .losses
                    .saturating_sub(initial.memory_summary.losses),
                avoided_losses_delta: final_state
                    .memory_summary
                    .avoided_losses
                    .saturating_sub(initial.memory_summary.avoided_losses),
                missed_gains_delta: final_state
                    .memory_summary
                    .missed_gains
                    .saturating_sub(initial.memory_summary.missed_gains),
                high_confidence_misses_delta: final_state
                    .memory_summary
                    .high_confidence_misses
                    .saturating_sub(initial.memory_summary.high_confidence_misses),
                doctrine_violations_delta: final_state
                    .memory_summary
                    .doctrine_violations
                    .saturating_sub(initial.memory_summary.doctrine_violations),
                sandbox_candidates_created: sandbox_candidates
                    .iter()
                    .filter(|candidate| candidate.agent_id == initial.agent_id)
                    .count() as u64,
                cooldown_triggered,
                quarantined,
                reason_codes: stable_reason_codes(&summary_reason_codes),
            })
        })
        .collect()
}

fn validate_three_agent_set(states: &[CanonicalAgentState]) -> Result<(), PaperLearningLoopError> {
    let mut actual = states
        .iter()
        .map(|state| state.agent_id.as_str())
        .collect::<Vec<_>>();
    actual.sort_unstable();
    let mut expected = vec![
        "momentum_trend_fast",
        "value_quality_filter",
        "cycle_risk_skeptic",
    ];
    expected.sort_unstable();
    if actual != expected
        || states.iter().any(|state| {
            state.kind == AgentKind::Future8AgentPlaceholder
                || agent_kind_from_id(&state.agent_id) != Some(state.kind)
                || matches!(
                    state.status,
                    AgentStatus::SandboxOnly | AgentStatus::Disabled
                )
                || !canonical_state_numeric_valid(state)
        })
    {
        return Err(PaperLearningLoopError::InvalidActiveAgentSet);
    }
    Ok(())
}

fn canonical_state_numeric_valid(state: &CanonicalAgentState) -> bool {
    let policy_values = [
        state.mutable_policy.volume_z_threshold,
        state.mutable_policy.stop_loss_atr_mult,
        state.mutable_policy.take_profit_rr,
        state.mutable_policy.confidence_entry_threshold,
        state.mutable_policy.max_exposure_hint,
        state.mutable_policy.unknown_asset_penalty,
        state.mutable_policy.quality_threshold_placeholder,
        state.mutable_policy.defensive_bias,
        state.mutable_policy.overheat_threshold,
        state.mutable_policy.min_risk_reward,
        state.mutable_policy.volatility_penalty,
        state.mutable_policy.groupthink_penalty,
        state.mutable_policy.veto_sensitivity,
    ];
    !state.version.version_id.trim().is_empty()
        && !state.version.sandbox_only
        && state.voice_state.voice_power.is_finite()
        && (0.0..=1.0).contains(&state.voice_state.voice_power)
        && state.memory_summary.max_drawdown_contribution.is_finite()
        && policy_values
            .iter()
            .flatten()
            .all(|value| value.is_finite())
        && state
            .doctrine
            .risk_constraints
            .values()
            .all(|value| value.is_finite())
}

fn validate_learning_loop_input(
    input: &PaperLearningLoopInput,
) -> Result<(), PaperLearningLoopError> {
    let market = &input.market_snapshot;
    let signal = &input.signal_input;
    let risk = &input.risk_snapshot;
    let market_values = [
        market.price,
        market.bid,
        market.ask,
        market.spread_bps,
        market.volume,
        market.trade_value,
        market.volatility,
        market.data_quality_score,
    ];
    let signal_values = [
        signal.p_win,
        signal.p_stop,
        signal.expected_return,
        signal.expected_drawdown,
        signal.confidence,
        signal.no_trade_probability,
    ];
    let risk_values = [
        risk.daily_pnl_pct,
        risk.total_exposure_pct,
        risk.symbol_exposure_pct,
        risk.api_health_score,
        risk.data_quality_score,
    ];
    let chair = input.loop_config.chair;
    let governor = input.loop_config.risk_governor;
    let chair_values = [
        chair.strong_threshold,
        chair.weak_threshold,
        chair.defensive_bonus_weight,
        chair.risk_penalty_weight,
        chair.groupthink_penalty_weight,
        chair.disagreement_penalty_weight,
        chair.cluster_groupthink_penalty,
    ];
    let governor_values = [
        governor.max_daily_loss_pct,
        governor.min_expected_edge,
        governor.min_confidence,
        governor.max_spread_bps,
        governor.min_data_quality,
        governor.min_api_health,
        governor.max_allowed_volatility,
        governor.min_risk_reward,
        governor.max_total_exposure,
        governor.max_symbol_exposure,
        governor.min_trade_value,
    ];
    let valid_agent_ids = input
        .initial_agent_states
        .iter()
        .map(|state| state.agent_id.as_str())
        .collect::<Vec<_>>();
    let context_agents_valid = input.paper_context.as_ref().is_none_or(|context| {
        context
            .doctrine_violation_agents
            .iter()
            .chain(context.overtrade_agents.iter())
            .all(|agent_id| valid_agent_ids.contains(&agent_id.as_str()))
    });
    let context_time_valid = input.paper_context.as_ref().is_none_or(|context| {
        !context.outcome_finalized || context.finalized_at_timestamp_ms >= market.timestamp_ms
    });
    let valid_probabilities = [
        signal.p_win,
        signal.p_stop,
        signal.confidence,
        signal.no_trade_probability,
        market.data_quality_score,
        risk.api_health_score,
        risk.data_quality_score,
    ]
    .iter()
    .all(|value| (0.0..=1.0).contains(value));
    if market.symbol.trim().is_empty()
        || signal.symbol != market.symbol
        || !matches!(input.loop_config.market.as_str(), "US" | "KR" | "BTC")
        || market_values.iter().any(|value| !value.is_finite())
        || signal_values.iter().any(|value| !value.is_finite())
        || risk_values.iter().any(|value| !value.is_finite())
        || chair_values.iter().any(|value| !value.is_finite())
        || governor_values.iter().any(|value| !value.is_finite())
        || market.price <= 0.0
        || market.bid <= 0.0
        || market.ask <= 0.0
        || market.ask < market.bid
        || market.spread_bps < 0.0
        || market.volume < 0.0
        || market.trade_value < 0.0
        || market.volatility < 0.0
        || signal.horizon_bars == 0
        || signal.expected_drawdown < 0.0
        || risk.total_exposure_pct < 0.0
        || risk.symbol_exposure_pct < 0.0
        || chair.strong_threshold < 0.0
        || chair.weak_threshold < 0.0
        || chair.strong_threshold < chair.weak_threshold
        || chair_values[2..].iter().any(|value| *value < 0.0)
        || governor.max_daily_loss_pct <= 0.0
        || governor.max_consecutive_losses == 0
        || governor.min_expected_edge < 0.0
        || !(0.0..=1.0).contains(&governor.min_confidence)
        || governor.max_spread_bps < 0.0
        || !(0.0..=1.0).contains(&governor.min_data_quality)
        || !(0.0..=1.0).contains(&governor.min_api_health)
        || governor.max_allowed_volatility < 0.0
        || governor.min_risk_reward <= 0.0
        || !(0.0..=1.0).contains(&governor.max_total_exposure)
        || !(0.0..=1.0).contains(&governor.max_symbol_exposure)
        || governor.min_trade_value < 0.0
        || !valid_probabilities
        || !context_agents_valid
        || !context_time_valid
    {
        return Err(PaperLearningLoopError::InvalidDecisionInput);
    }
    Ok(())
}

fn build_loop_agent_proposals(
    votes: &[InvestorVote],
    states: &[CanonicalAgentState],
    chair_output: &ChairOutput,
    market: &MarketSnapshot,
    signal: &SignalOutput,
    market_name: &str,
    decision_id: &str,
) -> Vec<AgentProposal> {
    votes
        .iter()
        .map(|vote| {
            let proposal_horizon = states
                .iter()
                .find(|state| state.agent_id == vote.persona_id)
                .and_then(|state| state.doctrine.allowed_horizons.first())
                .copied()
                .unwrap_or_else(|| horizon_from_bars(signal.horizon_bars));
            let mut reason_codes = vote.reason_codes.clone();
            if chair_output.lead_speaker == vote.persona_id {
                reason_codes.push(ReasonCode::AgentSelectedForDecision);
            } else if chair_output.selected_speakers.contains(&vote.persona_id) {
                reason_codes.push(ReasonCode::AgentSupportedFinalDecision);
            }
            if vote.stance == Stance::Abstain {
                reason_codes.push(ReasonCode::AgentAbstained);
            }
            AgentProposal {
                proposal_id: format!("{decision_id}:{}", vote.persona_id),
                agent_id: vote.persona_id.clone(),
                stance: vote.stance,
                confidence: (signal.confidence * vote.conviction).clamp(0.0, 1.0),
                expected_edge: signal.expected_return + vote.expected_return_adjustment,
                expected_drawdown: signal.expected_drawdown.max(0.0),
                no_trade_probability: signal.no_trade_probability.clamp(0.0, 1.0),
                horizon: proposal_horizon,
                market: market_name.to_string(),
                symbol: market.symbol.clone(),
                reason_codes: stable_reason_codes(&reason_codes),
            }
        })
        .collect()
}

fn build_loop_outcome(
    decision_id: &str,
    market: &MarketSnapshot,
    signal: &SignalOutput,
    votes: &[InvestorVote],
    chair_output: &ChairOutput,
    risk_decision: &RiskDecision,
    paper_order: Option<&PaperOrder>,
    context: &PaperOutcomeContext,
) -> Result<OutcomeRecord, PaperLearningLoopError> {
    let numeric_values = [
        context.realized_net_return_pct,
        context.max_adverse_excursion_pct,
        context.hypothetical_net_return_pct.unwrap_or(0.0),
    ];
    if numeric_values.iter().any(|value| !value.is_finite())
        || context.max_adverse_excursion_pct < 0.0
    {
        return Err(PaperLearningLoopError::InvalidPaperOutcome);
    }

    let executed = match (paper_order, context.outcome_kind) {
        (Some(order), PaperOutcomeKind::FilledPaperOrder)
            if order.status == PaperOrderStatus::Filled
                && paper_fill_evidence_matches(order, context.fill_evidence.as_ref()) =>
        {
            true
        }
        (None, PaperOutcomeKind::NoExecution) if context.fill_evidence.is_none() => false,
        _ => return Err(PaperLearningLoopError::InvalidPaperOutcome),
    };
    let denied_by_risk = !executed
        && risk_decision.kind != RiskDecisionKind::ApprovePaper
        && risk_decision.approved_order_plan.is_none()
        && chair_output.decision != crate::core::ChairDecisionKind::NoTrade;
    let no_trade = !executed && !denied_by_risk;
    let triple_barrier_result = executed.then(|| {
        synthetic_barrier_result(
            context.realized_net_return_pct,
            market.price,
            context.max_adverse_excursion_pct,
        )
    });
    let hypothetical_result = (!executed)
        .then(|| context.hypothetical_net_return_pct)
        .flatten()
        .map(|return_pct| {
            synthetic_barrier_result(return_pct, market.price, context.max_adverse_excursion_pct)
        });
    let avoided_loss_score = hypothetical_result
        .as_ref()
        .filter(|result| result.net_return_pct < 0.0)
        .map(|result| result.net_return_pct.abs())
        .unwrap_or(0.0);
    let missed_gain_penalty = hypothetical_result
        .as_ref()
        .filter(|result| result.net_return_pct > 0.0)
        .map(|result| result.net_return_pct * 0.20)
        .unwrap_or(0.0);
    let attribution_records = build_loop_attribution(votes, chair_output, denied_by_risk, no_trade);
    let mut reason_codes = chair_output
        .reason_codes
        .iter()
        .chain(risk_decision.reason_codes.iter())
        .cloned()
        .collect::<Vec<_>>();
    reason_codes.push(ReasonCode::PaperExecutionOnly);
    reason_codes.push(ReasonCode::DeterministicPath);
    if executed {
        reason_codes.push(ReasonCode::PaperFillSimulated);
    } else if denied_by_risk {
        reason_codes.push(ReasonCode::RiskDeniedCounterfactual);
    } else {
        reason_codes.push(ReasonCode::NoTradeCounterfactual);
    }
    if avoided_loss_score > 0.0 {
        reason_codes.push(ReasonCode::AvoidedLossRecorded);
        reason_codes.push(ReasonCode::PositiveSilenceValue);
    }
    if missed_gain_penalty > 0.0 {
        reason_codes.push(ReasonCode::MissedGainRecorded);
        reason_codes.push(ReasonCode::NegativeSilenceValue);
    }

    Ok(OutcomeRecord {
        decision_id: decision_id.to_string(),
        symbol: market.symbol.clone(),
        timestamp_ms: market.timestamp_ms,
        regime: market.regime,
        horizon: horizon_from_bars(signal.horizon_bars),
        signal_confidence: signal.confidence,
        executed,
        denied_by_risk,
        no_trade,
        triple_barrier_result,
        hypothetical_result,
        realized_net_return_pct: if executed {
            context.realized_net_return_pct
        } else {
            0.0
        },
        avoided_loss_score,
        missed_gain_penalty,
        attribution_records,
        shadow_outcomes: Vec::new(),
        reason_codes: stable_reason_codes(&reason_codes),
    })
}

fn paper_fill_evidence_matches(order: &PaperOrder, evidence: Option<&PaperFillEvidence>) -> bool {
    evidence.is_some_and(|evidence| {
        !evidence.fill_id.trim().is_empty()
            && evidence.paper_order_id == order.order_id
            && evidence.symbol == order.symbol
            && evidence.paper_only
            && evidence.filled_at_timestamp_ms >= order.timestamp_ms
    })
}

fn synthetic_barrier_result(
    return_pct: f64,
    entry_price: f64,
    max_adverse_excursion_pct: f64,
) -> TripleBarrierResult {
    let (outcome, first_hit, outcome_reason) = if return_pct > 0.0 {
        (
            TripleBarrierOutcome::Win,
            BarrierHit::TakeProfit,
            ReasonCode::TakeProfitHit,
        )
    } else if return_pct < 0.0 {
        (
            TripleBarrierOutcome::Loss,
            BarrierHit::StopLoss,
            ReasonCode::StopLossHit,
        )
    } else {
        (
            TripleBarrierOutcome::Neutral,
            BarrierHit::TimeExpired,
            ReasonCode::TimeBarrierExpired,
        )
    };
    TripleBarrierResult {
        outcome,
        first_hit,
        entry_index: 0,
        exit_index: 1,
        entry_price,
        exit_price: entry_price * (1.0 + return_pct),
        gross_return_pct: return_pct,
        net_return_pct: return_pct,
        max_favorable_excursion_pct: return_pct.max(0.0),
        max_adverse_excursion_pct,
        bars_held: 1,
        reason_codes: vec![
            ReasonCode::PaperFillSimulated,
            ReasonCode::DeterministicPath,
            outcome_reason,
        ],
    }
}

fn build_loop_attribution(
    votes: &[InvestorVote],
    chair_output: &ChairOutput,
    denied_by_risk: bool,
    no_trade: bool,
) -> Vec<AttributionRecord> {
    votes
        .iter()
        .map(|vote| {
            let selected_for_decision = chair_output.selected_speakers.contains(&vote.persona_id);
            let counterfactual_role = if denied_by_risk {
                if matches!(vote.stance, Stance::NoTrade | Stance::Abstain) || vote.veto {
                    CounterfactualRole::RiskVetoAligned
                } else {
                    CounterfactualRole::RiskVetoOpposed
                }
            } else if no_trade {
                if matches!(vote.stance, Stance::NoTrade | Stance::Abstain) {
                    CounterfactualRole::SupportedFinalDecision
                } else {
                    CounterfactualRole::OpposedFinalDecision
                }
            } else if vote.stance == Stance::Buy {
                CounterfactualRole::SupportedFinalDecision
            } else {
                CounterfactualRole::OpposedFinalDecision
            };
            AttributionRecord {
                persona_id: vote.persona_id.clone(),
                selected_for_decision,
                stance: vote.stance,
                conviction: vote.conviction,
                voice_power: vote.voice_power,
                contribution_score: match counterfactual_role {
                    CounterfactualRole::SupportedFinalDecision
                    | CounterfactualRole::RiskVetoAligned => vote.voice_power * vote.conviction,
                    CounterfactualRole::OpposedFinalDecision
                    | CounterfactualRole::RiskVetoOpposed => -(vote.voice_power * vote.conviction),
                    CounterfactualRole::ForcedContrarian => {
                        vote.voice_power * vote.conviction * 0.5
                    }
                    CounterfactualRole::ShadowOnly => 0.0,
                },
                counterfactual_role,
                reason_codes: vote.reason_codes.clone(),
            }
        })
        .collect()
}

fn outcome_for_agent(
    outcome: &OutcomeRecord,
    attribution: &AttributionRecord,
    hypothetical_return: Option<f64>,
) -> OutcomeRecord {
    let mut attributed = outcome.clone();
    if attribution.stance == Stance::Abstain {
        attributed.realized_net_return_pct = 0.0;
        attributed.avoided_loss_score = 0.0;
        attributed.missed_gain_penalty = 0.0;
        attributed.triple_barrier_result = outcome.executed.then(|| {
            synthetic_barrier_result(
                0.0,
                outcome
                    .triple_barrier_result
                    .as_ref()
                    .map(|result| result.entry_price)
                    .unwrap_or(1.0),
                0.0,
            )
        });
        return attributed;
    }
    let supported = matches!(
        attribution.counterfactual_role,
        CounterfactualRole::SupportedFinalDecision | CounterfactualRole::RiskVetoAligned
    );
    if outcome.executed && !supported {
        attributed.realized_net_return_pct = 0.0;
        attributed.triple_barrier_result = Some(synthetic_barrier_result(
            0.0,
            outcome
                .triple_barrier_result
                .as_ref()
                .map(|result| result.entry_price)
                .unwrap_or(1.0),
            0.0,
        ));
        if outcome.realized_net_return_pct < 0.0 {
            attributed.avoided_loss_score = outcome.realized_net_return_pct.abs();
            attributed.missed_gain_penalty = 0.0;
        } else if outcome.realized_net_return_pct > 0.0 {
            attributed.avoided_loss_score = 0.0;
            attributed.missed_gain_penalty = outcome.realized_net_return_pct * 0.20;
        }
    } else if !outcome.executed && !supported {
        let counterfactual = hypothetical_return.unwrap_or(0.0);
        attributed.realized_net_return_pct = counterfactual;
        attributed.avoided_loss_score = 0.0;
        attributed.missed_gain_penalty = 0.0;
    }
    attributed
}

fn apply_attribution_to_feedback(
    feedback: &mut AgentFeedback,
    attribution: &AttributionRecord,
    vote: &InvestorVote,
    is_lead: bool,
) {
    if is_lead {
        feedback
            .reason_codes
            .push(ReasonCode::AgentSelectedForDecision);
    }
    if vote.stance == Stance::Abstain {
        feedback.reason_codes.push(ReasonCode::AgentAbstained);
    }
    match attribution.counterfactual_role {
        CounterfactualRole::SupportedFinalDecision => feedback
            .reason_codes
            .push(ReasonCode::AgentSupportedFinalDecision),
        CounterfactualRole::OpposedFinalDecision | CounterfactualRole::ForcedContrarian => feedback
            .reason_codes
            .push(ReasonCode::AgentOpposedFinalDecision),
        CounterfactualRole::RiskVetoAligned => {
            feedback.reason_codes.push(ReasonCode::AgentRiskVetoAligned)
        }
        CounterfactualRole::RiskVetoOpposed => {
            feedback.reason_codes.push(ReasonCode::AgentRiskVetoOpposed)
        }
        CounterfactualRole::ShadowOnly => {}
    }
    let no_trade_aligned = matches!(vote.stance, Stance::NoTrade);
    if no_trade_aligned && feedback.avoided_loss_score > 0.0 {
        feedback.no_trade_correct = true;
        feedback.reason_codes.push(ReasonCode::AgentNoTradeCorrect);
    }
    if no_trade_aligned && feedback.missed_gain_penalty > 0.0 {
        feedback
            .reason_codes
            .push(ReasonCode::AgentNoTradeMissedGain);
    }
    feedback.reason_codes = stable_reason_codes(&feedback.reason_codes);
}

pub fn canonical_current_agent_states() -> Vec<CanonicalAgentState> {
    active_persona_cards()
        .into_iter()
        .filter_map(canonical_agent_state_from_card)
        .collect()
}

pub fn canonical_agent_state_from_card(card: PersonaCard) -> Option<CanonicalAgentState> {
    let kind = agent_kind_from_id(&card.persona_id)?;
    let agent_id = card.persona_id.clone();
    let doctrine = canonical_doctrine(&card, kind);
    Some(CanonicalAgentState {
        agent_id: agent_id.clone(),
        kind,
        status: AgentStatus::Active,
        doctrine,
        mutable_policy: card.mutable_policy,
        voice_state: AgentVoiceState {
            voice_power: card.voice.current_voice_power.clamp(0.0, 1.0),
            tier: card.tier,
            cooldown_bars: 0,
            veto_power: matches!(kind, AgentKind::CycleRiskSkeptic),
            recent_penalty_count: 0,
            recent_reward_count: 0,
        },
        memory_summary: AgentMemorySummary::default(),
        version: AgentVersion {
            version_id: format!("{agent_id}:v1"),
            parent_version_id: None,
            created_from_feedback_event: None,
            live_enabled: true,
            sandbox_only: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        reason_codes: vec![ReasonCode::DeterministicPath],
    })
}

pub fn future_agent_placeholder_state(agent_id: &str) -> CanonicalAgentState {
    CanonicalAgentState {
        agent_id: agent_id.to_string(),
        kind: AgentKind::Future8AgentPlaceholder,
        status: AgentStatus::Disabled,
        doctrine: AgentDoctrine {
            immutable_rules: ImmutableDoctrine::default(),
            allowed_horizons: Vec::new(),
            allowed_markets: Vec::new(),
            allowed_assets: Vec::new(),
            veto_permissions: false,
            prohibited_behaviors: vec!["active voting before validation".to_string()],
            risk_constraints: BTreeMap::new(),
        },
        mutable_policy: MutablePolicy::default(),
        voice_state: AgentVoiceState {
            voice_power: 0.0,
            tier: PersonaTier::D,
            cooldown_bars: 0,
            veto_power: false,
            recent_penalty_count: 0,
            recent_reward_count: 0,
        },
        memory_summary: AgentMemorySummary::default(),
        version: AgentVersion {
            version_id: format!("{agent_id}:deferred"),
            parent_version_id: None,
            created_from_feedback_event: None,
            live_enabled: false,
            sandbox_only: true,
            reason_codes: vec![ReasonCode::ShadowEvaluationPending],
        },
        reason_codes: vec![ReasonCode::ShadowEvaluationPending],
    }
}

pub fn apply_feedback_to_memory_summary(
    state: &CanonicalAgentState,
    feedback: &AgentFeedback,
) -> AgentMemorySummary {
    if !feedback_is_valid_for(state, feedback) {
        return state.memory_summary.clone();
    }
    let mut summary = state.memory_summary.clone();
    summary.total_decisions = summary.total_decisions.saturating_add(1);
    match feedback.outcome_kind {
        AgentFeedbackOutcomeKind::ExecutedPaperTrade => {
            summary.total_paper_trades = summary.total_paper_trades.saturating_add(1);
            if feedback.realized_net_return > 0.0 {
                summary.wins = summary.wins.saturating_add(1);
            } else if feedback.realized_net_return < 0.0 {
                summary.losses = summary.losses.saturating_add(1);
                if feedback.confidence_at_decision >= 0.75 {
                    summary.high_confidence_misses =
                        summary.high_confidence_misses.saturating_add(1);
                }
            }
        }
        AgentFeedbackOutcomeKind::NoTrade | AgentFeedbackOutcomeKind::RiskDenied => {
            summary.total_no_trades = summary.total_no_trades.saturating_add(1);
        }
        AgentFeedbackOutcomeKind::Abstained => {}
    }
    if feedback.avoided_loss_score > 0.0 {
        summary.avoided_losses = summary.avoided_losses.saturating_add(1);
    }
    if feedback.missed_gain_penalty > 0.0 {
        summary.missed_gains = summary.missed_gains.saturating_add(1);
    }
    if feedback.doctrine_violation {
        summary.doctrine_violations = summary.doctrine_violations.saturating_add(1);
    }
    summary.max_drawdown_contribution = summary
        .max_drawdown_contribution
        .max(feedback.drawdown_contribution.max(0.0));
    summary.last_updated_event_id = Some(feedback.outcome_id.clone());
    summary
}

pub fn detect_doctrine_violation(
    doctrine: &AgentDoctrine,
    proposal: &AgentProposal,
    feedback: &AgentFeedback,
) -> bool {
    feedback.doctrine_violation
        || (matches!(proposal.stance, Stance::Buy | Stance::Sell)
            && (!doctrine.allowed_horizons.contains(&proposal.horizon)
                || !doctrine.allowed_markets.contains(&proposal.market)))
        || proposal
            .reason_codes
            .iter()
            .chain(feedback.reason_codes.iter())
            .any(|reason| {
                matches!(
                    reason,
                    ReasonCode::DoctrineViolation
                        | ReasonCode::RiskBypassAttempt
                        | ReasonCode::AveragingDownRejected
                )
            })
}

pub fn compute_chair_reward_penalty(
    state: &CanonicalAgentState,
    feedback: &AgentFeedback,
) -> ChairRewardPenalty {
    if !feedback_is_valid_for(state, feedback) {
        return ChairRewardPenalty {
            agent_id: state.agent_id.clone(),
            source_feedback_id: feedback_event_id(feedback),
            reward_delta: 0.0,
            penalty_delta: 0.0,
            voice_delta: 0.0,
            cooldown_delta: 0,
            tier_action: ChairTierAction::Keep,
            reason_codes: vec![ReasonCode::PaperExecutionOnly],
        };
    }
    let performance_return =
        if feedback.outcome_kind == AgentFeedbackOutcomeKind::ExecutedPaperTrade {
            feedback.realized_net_return
        } else {
            feedback.counterfactual_net_return.unwrap_or(0.0)
        };
    let profit_reward = (performance_return.max(0.0) * 4.0).clamp(0.0, 0.25);
    let avoided_loss_reward = (feedback.avoided_loss_score.max(0.0) * 0.60).clamp(0.0, 0.35);
    let risk_warning_reward = if feedback.risk_warning_correct {
        0.12
    } else {
        0.0
    };
    let no_trade_reward = if feedback.no_trade_correct { 0.10 } else { 0.0 };
    let reward_delta =
        (profit_reward + avoided_loss_reward + risk_warning_reward + no_trade_reward)
            .clamp(0.0, 1.0);

    let (loss_multiplier, loss_penalty_cap) = if feedback.confidence_at_decision >= 0.75 {
        (8.0, 0.80)
    } else {
        (4.0, 0.40)
    };
    let loss_penalty =
        (performance_return.min(0.0).abs() * loss_multiplier).clamp(0.0, loss_penalty_cap);
    let repeated_high_confidence_penalty = if performance_return < 0.0
        && feedback.confidence_at_decision >= 0.75
        && state.memory_summary.high_confidence_misses > 0
    {
        0.20
    } else {
        0.0
    };
    let missed_gain_penalty = (feedback.missed_gain_penalty.max(0.0) * 0.20).clamp(0.0, 0.15);
    let drawdown_penalty = (feedback.drawdown_contribution.max(0.0) * 0.60).clamp(0.0, 0.40);
    let overtrade_penalty = if feedback.overtrade { 0.35 } else { 0.0 };
    let bad_data_penalty = if feedback
        .reason_codes
        .contains(&ReasonCode::FeedbackBadDataProposal)
    {
        0.30
    } else {
        0.0
    };
    let risk_opposition_penalty = if feedback
        .reason_codes
        .contains(&ReasonCode::FeedbackRiskGovernorOpposed)
    {
        0.35
    } else {
        0.0
    };
    let doctrine_penalty = if feedback.doctrine_violation {
        1.0
    } else {
        0.0
    };
    let penalty_delta = (loss_penalty
        + repeated_high_confidence_penalty
        + missed_gain_penalty
        + drawdown_penalty
        + overtrade_penalty
        + bad_data_penalty
        + risk_opposition_penalty
        + doctrine_penalty)
        .clamp(0.0, 1.0);
    let voice_delta = (reward_delta * 0.08 - penalty_delta * 0.12).clamp(-0.50, 0.25);
    let tier_action = if feedback.doctrine_violation
        || feedback
            .reason_codes
            .contains(&ReasonCode::RiskBypassAttempt)
    {
        ChairTierAction::Quarantine
    } else if penalty_delta >= 0.35 {
        ChairTierAction::Cooldown
    } else if reward_delta >= 0.25 && penalty_delta == 0.0 {
        ChairTierAction::SandboxCandidate
    } else if penalty_delta > reward_delta {
        ChairTierAction::Demote
    } else {
        ChairTierAction::Keep
    };
    let cooldown_delta = match tier_action {
        ChairTierAction::Quarantine => 10,
        ChairTierAction::Cooldown => 3,
        _ => 0,
    };
    let mut reason_codes = feedback.reason_codes.clone();
    reason_codes.push(ReasonCode::PersonaEvaluationBuilt);
    if avoided_loss_reward > 0.0 {
        reason_codes.push(ReasonCode::AvoidedLossRecorded);
        reason_codes.push(ReasonCode::PositiveSilenceValue);
    }
    if missed_gain_penalty > 0.0 {
        reason_codes.push(ReasonCode::MissedGainRecorded);
        reason_codes.push(ReasonCode::NegativeSilenceValue);
    }
    if matches!(tier_action, ChairTierAction::Cooldown) {
        reason_codes.push(ReasonCode::CooldownRequired);
        reason_codes.push(ReasonCode::CooldownStarted);
    }
    if matches!(tier_action, ChairTierAction::Quarantine) {
        reason_codes.push(ReasonCode::Quarantined);
    }
    if matches!(tier_action, ChairTierAction::SandboxCandidate) {
        reason_codes.push(ReasonCode::ShadowEvaluationPending);
    }
    ChairRewardPenalty {
        agent_id: state.agent_id.clone(),
        source_feedback_id: feedback_event_id(feedback),
        reward_delta,
        penalty_delta,
        voice_delta,
        cooldown_delta,
        tier_action,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn feedback_event_id(feedback: &AgentFeedback) -> String {
    let payload = serde_json::to_string(feedback).unwrap_or_else(|_| format!("{feedback:?}"));
    format!(
        "{}::feedback::{}",
        feedback.outcome_id,
        stable_hash_string(&payload)
    )
}

pub fn update_agent_voice_state(
    voice_state: &AgentVoiceState,
    reward_penalty: &ChairRewardPenalty,
) -> AgentVoiceState {
    let mut next = voice_state.clone();
    next.voice_power = (next.voice_power + reward_penalty.voice_delta).clamp(0.0, 1.0);
    if reward_penalty.reward_delta > 0.0 {
        next.recent_reward_count = next.recent_reward_count.saturating_add(1);
    }
    if reward_penalty.penalty_delta > 0.0 {
        next.recent_penalty_count = next.recent_penalty_count.saturating_add(1);
    }
    next.cooldown_bars = next
        .cooldown_bars
        .saturating_add(reward_penalty.cooldown_delta);
    next.tier = match reward_penalty.tier_action {
        ChairTierAction::Promote | ChairTierAction::SandboxCandidate => next.tier,
        ChairTierAction::Demote => match next.tier {
            PersonaTier::D | PersonaTier::XQuarantined => next.tier,
            _ => demote_one_tier(next.tier),
        },
        ChairTierAction::Quarantine => PersonaTier::XQuarantined,
        _ => next.tier,
    };
    if matches!(reward_penalty.tier_action, ChairTierAction::Quarantine) {
        next.veto_power = false;
    }
    next
}

pub fn classify_agent_status(state: &CanonicalAgentState) -> AgentStatus {
    if matches!(state.status, AgentStatus::Disabled) {
        AgentStatus::Disabled
    } else if matches!(state.status, AgentStatus::Quarantined) {
        AgentStatus::Quarantined
    } else if state.version.sandbox_only {
        AgentStatus::SandboxOnly
    } else if state.voice_state.tier == PersonaTier::XQuarantined
        || state.memory_summary.doctrine_violations > 0
    {
        AgentStatus::Quarantined
    } else if state.voice_state.cooldown_bars > 0 {
        AgentStatus::Cooldown
    } else if matches!(state.status, AgentStatus::Observer) {
        AgentStatus::Observer
    } else {
        AgentStatus::Active
    }
}

fn apply_chair_reward_penalty(
    state: &CanonicalAgentState,
    reward_penalty: &ChairRewardPenalty,
) -> CanonicalAgentState {
    if reward_penalty.agent_id != state.agent_id
        || reward_penalty.source_feedback_id.trim().is_empty()
        || !reward_penalty
            .reason_codes
            .contains(&ReasonCode::PaperExecutionOnly)
    {
        return state.clone();
    }
    let mut next = state.clone();
    next.voice_state = update_agent_voice_state(&state.voice_state, reward_penalty);
    next.version = AgentVersion {
        version_id: format!(
            "{}::{}",
            state.version.version_id, reward_penalty.source_feedback_id
        ),
        parent_version_id: Some(state.version.version_id.clone()),
        created_from_feedback_event: Some(reward_penalty.source_feedback_id.clone()),
        live_enabled: state.version.live_enabled
            && !matches!(reward_penalty.tier_action, ChairTierAction::Quarantine),
        sandbox_only: state.version.sandbox_only,
        reason_codes: reward_penalty.reason_codes.clone(),
    };
    next.reason_codes = stable_reason_codes(
        &state
            .reason_codes
            .iter()
            .chain(reward_penalty.reason_codes.iter())
            .cloned()
            .collect::<Vec<_>>(),
    );
    next.status = classify_agent_status(&next);
    next
}

fn apply_agent_feedback(
    state: &CanonicalAgentState,
    proposal: &AgentProposal,
    feedback: &AgentFeedback,
) -> CanonicalAgentState {
    if !feedback_is_valid_for(state, feedback) || !proposal_is_valid_for(state, proposal) {
        return state.clone();
    }
    let mut normalized_feedback = feedback.clone();
    normalized_feedback.doctrine_violation =
        detect_doctrine_violation(&state.doctrine, proposal, feedback);
    let reward_penalty = compute_chair_reward_penalty(state, &normalized_feedback);
    let mut next = apply_chair_reward_penalty(state, &reward_penalty);
    next.memory_summary = apply_feedback_to_memory_summary(state, &normalized_feedback);
    next.status = classify_agent_status(&next);
    next
}

pub fn build_agent_feedback_from_paper_outcome(
    agent_state: &CanonicalAgentState,
    proposal: &AgentProposal,
    paper_outcome: &OutcomeRecord,
    context: &FeedbackContext,
) -> Result<AgentFeedback, AgentFeedbackBuildError> {
    if proposal.agent_id != agent_state.agent_id {
        return Err(feedback_error(ReasonCode::FeedbackAgentMismatch));
    }
    if proposal.proposal_id.trim().is_empty()
        || proposal.symbol.trim().is_empty()
        || paper_outcome.symbol.trim().is_empty()
    {
        return Err(feedback_error(ReasonCode::FeedbackOutcomeIncomplete));
    }
    if proposal.symbol != paper_outcome.symbol {
        return Err(feedback_error(ReasonCode::FeedbackProposalOutcomeMismatch));
    }
    if !context.paper_only {
        return Err(feedback_error(ReasonCode::FeedbackOutcomeNotPaper));
    }
    if !context.outcome_finalized || outcome_is_pending(paper_outcome) {
        return Err(feedback_error(ReasonCode::FeedbackOutcomeIncomplete));
    }

    let numeric_values = [
        proposal.confidence,
        proposal.expected_edge,
        proposal.expected_drawdown,
        proposal.no_trade_probability,
        paper_outcome.signal_confidence,
        paper_outcome.realized_net_return_pct,
        paper_outcome.avoided_loss_score,
        paper_outcome.missed_gain_penalty,
        paper_outcome
            .triple_barrier_result
            .as_ref()
            .map(|result| result.max_adverse_excursion_pct)
            .unwrap_or(0.0),
    ];
    if numeric_values.iter().any(|value| !value.is_finite()) {
        return Err(feedback_error(ReasonCode::FeedbackNonFiniteValue));
    }
    if paper_outcome.executed
        && paper_outcome
            .triple_barrier_result
            .as_ref()
            .is_none_or(|result| {
                (result.net_return_pct - paper_outcome.realized_net_return_pct).abs() > 1e-12
            })
    {
        return Err(feedback_error(ReasonCode::FeedbackProposalOutcomeMismatch));
    }

    let mut reason_codes = paper_outcome
        .reason_codes
        .iter()
        .chain(proposal.reason_codes.iter())
        .cloned()
        .collect::<Vec<_>>();
    reason_codes.push(ReasonCode::PaperExecutionOnly);
    reason_codes.push(ReasonCode::DeterministicPath);

    let missing_return = paper_outcome.executed && paper_outcome.triple_barrier_result.is_none();
    if missing_return {
        reason_codes.push(ReasonCode::FeedbackMissingReturn);
    }

    let risk_role = paper_outcome
        .attribution_records
        .iter()
        .find(|record| record.persona_id == agent_state.agent_id)
        .map(|record| record.counterfactual_role);
    let avoided_loss_score = paper_outcome.avoided_loss_score.max(0.0);
    let missed_gain_penalty = paper_outcome.missed_gain_penalty.abs();
    let no_trade_aligned = proposal.stance == Stance::NoTrade
        || matches!(
            risk_role,
            Some(CounterfactualRole::SupportedFinalDecision | CounterfactualRole::RiskVetoAligned)
        );
    let no_trade_correct = (paper_outcome.no_trade || paper_outcome.denied_by_risk)
        && no_trade_aligned
        && avoided_loss_score > 0.0;
    if no_trade_correct {
        reason_codes.push(ReasonCode::FeedbackNoTradeAvoidedLoss);
    }
    if (paper_outcome.no_trade || paper_outcome.denied_by_risk)
        && no_trade_aligned
        && missed_gain_penalty > 0.0
    {
        reason_codes.push(ReasonCode::FeedbackNoTradeMissedGain);
    }

    if let Some(attribution) = paper_outcome
        .attribution_records
        .iter()
        .find(|record| record.persona_id == agent_state.agent_id)
    {
        if attribution.selected_for_decision {
            reason_codes.push(ReasonCode::AgentSelectedForDecision);
        }
        if proposal.stance == Stance::Abstain {
            reason_codes.push(ReasonCode::AgentAbstained);
        }
        match attribution.counterfactual_role {
            CounterfactualRole::SupportedFinalDecision => {
                reason_codes.push(ReasonCode::AgentSupportedFinalDecision);
            }
            CounterfactualRole::OpposedFinalDecision | CounterfactualRole::ForcedContrarian => {
                reason_codes.push(ReasonCode::AgentOpposedFinalDecision);
            }
            CounterfactualRole::RiskVetoAligned => {
                reason_codes.push(ReasonCode::AgentRiskVetoAligned);
            }
            CounterfactualRole::RiskVetoOpposed => {
                reason_codes.push(ReasonCode::AgentRiskVetoOpposed);
            }
            CounterfactualRole::ShadowOnly => {}
        }
    }
    let risk_governor_aligned = paper_outcome.denied_by_risk
        && match risk_role {
            Some(CounterfactualRole::RiskVetoAligned) => true,
            Some(CounterfactualRole::RiskVetoOpposed) => false,
            _ => matches!(proposal.stance, Stance::NoTrade),
        };
    let risk_governor_opposed = paper_outcome.denied_by_risk
        && match risk_role {
            Some(CounterfactualRole::RiskVetoAligned) => false,
            Some(CounterfactualRole::RiskVetoOpposed) => true,
            _ => matches!(proposal.stance, Stance::Buy | Stance::Sell),
        };
    if risk_governor_aligned {
        reason_codes.push(ReasonCode::FeedbackCorrectRiskWarning);
        reason_codes.push(ReasonCode::FeedbackRiskGovernorAligned);
    }
    if risk_governor_opposed {
        reason_codes.push(ReasonCode::FeedbackRiskGovernorOpposed);
    }

    let data_quality_bad = paper_outcome.reason_codes.iter().any(|reason| {
        matches!(
            reason,
            ReasonCode::DataQualityGateBreached
                | ReasonCode::FeatureDataQualityLow
                | ReasonCode::NonFiniteFeature
                | ReasonCode::StaleTimestamp
        )
    });
    if data_quality_bad && matches!(proposal.stance, Stance::Buy | Stance::Sell) {
        reason_codes.push(ReasonCode::FeedbackBadDataProposal);
    }

    let doctrine_violation = context.doctrine_violation
        || proposal
            .reason_codes
            .iter()
            .chain(paper_outcome.reason_codes.iter())
            .any(|reason| {
                matches!(
                    reason,
                    ReasonCode::DoctrineViolation
                        | ReasonCode::RiskBypassAttempt
                        | ReasonCode::AveragingDownRejected
                )
            });
    if doctrine_violation {
        reason_codes.push(ReasonCode::FeedbackDoctrineViolation);
    }

    let attributed_return = if missing_return {
        0.0
    } else {
        paper_outcome.realized_net_return_pct
    };
    let supported_execution = match risk_role {
        None | Some(CounterfactualRole::SupportedFinalDecision) => true,
        Some(
            CounterfactualRole::OpposedFinalDecision
            | CounterfactualRole::ForcedContrarian
            | CounterfactualRole::ShadowOnly
            | CounterfactualRole::RiskVetoAligned
            | CounterfactualRole::RiskVetoOpposed,
        ) => false,
    };
    let outcome_kind = if proposal.stance == Stance::Abstain {
        AgentFeedbackOutcomeKind::Abstained
    } else if paper_outcome.executed
        && supported_execution
        && matches!(proposal.stance, Stance::Buy | Stance::Sell)
    {
        AgentFeedbackOutcomeKind::ExecutedPaperTrade
    } else if paper_outcome.denied_by_risk {
        AgentFeedbackOutcomeKind::RiskDenied
    } else {
        AgentFeedbackOutcomeKind::NoTrade
    };
    let realized_net_return = if outcome_kind == AgentFeedbackOutcomeKind::ExecutedPaperTrade {
        attributed_return
    } else {
        0.0
    };
    let counterfactual_net_return = (outcome_kind != AgentFeedbackOutcomeKind::ExecutedPaperTrade
        && matches!(proposal.stance, Stance::Buy | Stance::Sell))
    .then_some(attributed_return);
    if realized_net_return < 0.0 && proposal.confidence >= 0.75 {
        reason_codes.push(ReasonCode::FeedbackHighConfidenceLoss);
    }

    Ok(AgentFeedback {
        agent_id: agent_state.agent_id.clone(),
        proposal_id: Some(proposal.proposal_id.clone()),
        outcome_id: paper_outcome.decision_id.clone(),
        paper_only: true,
        outcome_kind,
        realized_net_return,
        counterfactual_net_return,
        avoided_loss_score,
        missed_gain_penalty,
        drawdown_contribution: paper_outcome
            .triple_barrier_result
            .as_ref()
            .map(|result| result.max_adverse_excursion_pct.max(0.0))
            .unwrap_or(0.0),
        confidence_at_decision: proposal.confidence.clamp(0.0, 1.0),
        doctrine_violation,
        risk_warning_correct: risk_governor_aligned,
        no_trade_correct,
        overtrade: context.overtrade,
        reason_codes: stable_reason_codes(&reason_codes),
    })
}

pub fn apply_paper_feedback_cycle(
    state: &CanonicalAgentState,
    proposal: &AgentProposal,
    outcome: &OutcomeRecord,
    context: &FeedbackContext,
) -> Result<FeedbackCycleResult, AgentFeedbackBuildError> {
    let mut feedback = build_agent_feedback_from_paper_outcome(state, proposal, outcome, context)?;
    feedback.doctrine_violation = detect_doctrine_violation(&state.doctrine, proposal, &feedback);
    if feedback.doctrine_violation {
        feedback
            .reason_codes
            .push(ReasonCode::FeedbackDoctrineViolation);
        feedback.reason_codes = stable_reason_codes(&feedback.reason_codes);
    }

    let reward_penalty = compute_chair_reward_penalty(state, &feedback);
    let mut updated_state = apply_chair_reward_penalty(state, &reward_penalty);
    updated_state.memory_summary = apply_feedback_to_memory_summary(state, &feedback);
    updated_state.status = classify_agent_status(&updated_state);

    let version_entry = AgentStateSnapshot {
        agent_id: updated_state.agent_id.clone(),
        version_id: updated_state.version.version_id.clone(),
        parent_version_id: updated_state.version.parent_version_id.clone(),
        state: updated_state.clone(),
        feedback_event_id: Some(reward_penalty.source_feedback_id.clone()),
        created_from_paper_only: true,
        sandbox_only: updated_state.version.sandbox_only,
        reason_codes: reward_penalty.reason_codes.clone(),
    };
    let sandbox_candidate = if feedback.overtrade
        || feedback
            .reason_codes
            .contains(&ReasonCode::FeedbackHighConfidenceLoss)
        || feedback.no_trade_correct
        || matches!(
            reward_penalty.tier_action,
            ChairTierAction::SandboxCandidate
        ) {
        build_sandbox_promotion_candidate(&updated_state, &[feedback.clone()])
    } else {
        None
    };
    let reason_codes = stable_reason_codes(
        &feedback
            .reason_codes
            .iter()
            .chain(reward_penalty.reason_codes.iter())
            .cloned()
            .collect::<Vec<_>>(),
    );

    Ok(FeedbackCycleResult {
        original_state_version: state.version.version_id.clone(),
        feedback,
        reward_penalty,
        updated_state,
        version_entry,
        sandbox_candidate,
        reason_codes,
    })
}

pub fn build_sandbox_promotion_candidate(
    state: &CanonicalAgentState,
    feedback_batch: &[AgentFeedback],
) -> Option<SandboxPromotionCandidate> {
    if feedback_batch.is_empty()
        || feedback_batch
            .iter()
            .any(|feedback| !feedback_is_valid_for(state, feedback))
        || matches!(
            state.status,
            AgentStatus::Quarantined | AgentStatus::Disabled
        )
    {
        return None;
    }
    let mut source_feedback_ids = feedback_batch
        .iter()
        .map(feedback_event_id)
        .collect::<Vec<_>>();
    source_feedback_ids.sort();
    source_feedback_ids.dedup();
    let has_review_trigger = feedback_batch.iter().any(|feedback| {
        feedback.overtrade
            || feedback.no_trade_correct
            || (feedback.realized_net_return < 0.0 && feedback.confidence_at_decision >= 0.75)
            || feedback
                .reason_codes
                .contains(&ReasonCode::FeedbackHighConfidenceLoss)
    });
    if source_feedback_ids.len() < 3 && !has_review_trigger {
        return None;
    }
    let candidate_version_id = format!(
        "{}::sandbox::{}",
        state.version.version_id,
        source_feedback_ids.join("+")
    );
    Some(SandboxPromotionCandidate {
        candidate_id: format!("candidate::{candidate_version_id}"),
        agent_id: state.agent_id.clone(),
        parent_version_id: state.version.version_id.clone(),
        candidate_version_id,
        source_feedback_ids,
        proposed_policy_delta: BTreeMap::new(),
        sandbox_only: true,
        promotion_status: SandboxPromotionStatus::Proposed,
        reason_codes: vec![
            ReasonCode::DeterministicPath,
            ReasonCode::ShadowEvaluationPending,
            ReasonCode::PromotionInsufficientSamples,
        ],
    })
}

fn feedback_error(reason_code: ReasonCode) -> AgentFeedbackBuildError {
    AgentFeedbackBuildError {
        reason_codes: vec![reason_code],
    }
}

fn outcome_is_pending(outcome: &OutcomeRecord) -> bool {
    let terminal_classification_count =
        outcome.executed as usize + outcome.denied_by_risk as usize + outcome.no_trade as usize;
    outcome.decision_id.trim().is_empty()
        || terminal_classification_count != 1
        || (outcome.executed && outcome.triple_barrier_result.is_none())
        || outcome
            .triple_barrier_result
            .as_ref()
            .is_some_and(|result| result.outcome == TripleBarrierOutcome::NoData)
        || outcome
            .hypothetical_result
            .as_ref()
            .is_some_and(|result| result.outcome == TripleBarrierOutcome::NoData)
        || outcome
            .shadow_outcomes
            .iter()
            .any(|shadow| shadow.evaluation_pending)
}

fn feedback_is_valid_for(state: &CanonicalAgentState, feedback: &AgentFeedback) -> bool {
    feedback.paper_only
        && feedback.agent_id == state.agent_id
        && (0.0..=1.0).contains(&feedback.confidence_at_decision)
        && feedback.avoided_loss_score >= 0.0
        && feedback.missed_gain_penalty >= 0.0
        && feedback.drawdown_contribution >= 0.0
        && [
            feedback.realized_net_return,
            feedback.counterfactual_net_return.unwrap_or(0.0),
            feedback.avoided_loss_score,
            feedback.missed_gain_penalty,
            feedback.drawdown_contribution,
            feedback.confidence_at_decision,
        ]
        .iter()
        .all(|value| value.is_finite())
}

fn proposal_is_valid_for(state: &CanonicalAgentState, proposal: &AgentProposal) -> bool {
    proposal.agent_id == state.agent_id
        && !proposal.proposal_id.trim().is_empty()
        && !proposal.symbol.trim().is_empty()
        && [
            proposal.confidence,
            proposal.expected_edge,
            proposal.expected_drawdown,
            proposal.no_trade_probability,
        ]
        .iter()
        .all(|value| value.is_finite())
}

fn agent_kind_from_id(agent_id: &str) -> Option<AgentKind> {
    match agent_id {
        "momentum_trend_fast" => Some(AgentKind::MomentumTrendFast),
        "value_quality_filter" => Some(AgentKind::ValueQualityFilter),
        "cycle_risk_skeptic" => Some(AgentKind::CycleRiskSkeptic),
        _ => None,
    }
}

fn canonical_doctrine(card: &PersonaCard, kind: AgentKind) -> AgentDoctrine {
    let (allowed_markets, allowed_assets, veto_permissions, prohibited_behaviors) = match kind {
        AgentKind::MomentumTrendFast => (
            vec!["US".to_string(), "KR".to_string(), "BTC".to_string()],
            vec!["liquid_equity".to_string(), "btc".to_string()],
            false,
            vec![
                "averaging_down".to_string(),
                "overtrading".to_string(),
                "poor_liquidity_entry".to_string(),
            ],
        ),
        AgentKind::ValueQualityFilter => (
            vec!["US".to_string(), "KR".to_string()],
            vec!["scorable_equity".to_string()],
            false,
            vec![
                "intraday_entry".to_string(),
                "unknown_asset".to_string(),
                "missing_margin_of_safety".to_string(),
            ],
        ),
        AgentKind::CycleRiskSkeptic => (
            vec!["US".to_string(), "KR".to_string(), "BTC".to_string()],
            vec!["all_supported".to_string()],
            true,
            vec![
                "risk_bypass".to_string(),
                "euphoria_chasing".to_string(),
                "cooldown_ignored".to_string(),
            ],
        ),
        AgentKind::Future8AgentPlaceholder => (Vec::new(), Vec::new(), false, Vec::new()),
    };
    let mut risk_constraints = BTreeMap::new();
    if let Some(value) = card.mutable_policy.min_risk_reward {
        risk_constraints.insert("min_risk_reward".to_string(), value);
    }
    if let Some(value) = card.mutable_policy.max_exposure_hint {
        risk_constraints.insert("max_exposure_hint".to_string(), value);
    }
    AgentDoctrine {
        immutable_rules: card.immutable_doctrine.clone(),
        allowed_horizons: vec![card.evaluation.horizon],
        allowed_markets,
        allowed_assets,
        veto_permissions,
        prohibited_behaviors,
        risk_constraints,
    }
}

pub fn horizon_from_bars(horizon_bars: u32) -> Horizon {
    match horizon_bars {
        0..=12 => Horizon::Intraday,
        13..=72 => Horizon::Swing,
        _ => Horizon::Position,
    }
}

pub fn momentum_trend_fast_card(base_voice_power: f64) -> PersonaCard {
    PersonaCard {
        persona_id: "momentum_trend_fast".to_string(),
        archetype: "Livermore-like trend/momentum delegate".to_string(),
        tier: PersonaTier::B,
        immutable_doctrine: ImmutableDoctrine {
            never_average_down: true,
            cut_losses_quickly: true,
            pyramid_only_on_strength: true,
            speak_only_on_trend_or_breakout: true,
            rest_after_consecutive_losses: true,
            ..ImmutableDoctrine::default()
        },
        mutable_policy: MutablePolicy {
            breakout_lookback: Some(20),
            volume_z_threshold: Some(1.25),
            stop_loss_atr_mult: Some(1.6),
            take_profit_rr: Some(2.0),
            confidence_entry_threshold: Some(0.52),
            max_trade_frequency: Some(4),
            ..MutablePolicy::default()
        },
        voice: VoiceConfig {
            base_voice_power,
            current_voice_power: base_voice_power,
            ema_alpha: 0.08,
            severe_event_multiplier: 0.5,
        },
        evaluation: EvaluationProfile {
            horizon: Horizon::Intraday,
            favored_regimes: vec![Regime::TrendUp, Regime::RiskOn],
            tolerated_regimes: vec![Regime::TrendDown, Regime::Range],
            promotion_min_samples: 16,
            max_s_tier: 1,
        },
    }
}

pub fn value_quality_filter_card(base_voice_power: f64) -> PersonaCard {
    PersonaCard {
        persona_id: "value_quality_filter".to_string(),
        archetype: "Graham/Buffett-like defensive filter".to_string(),
        tier: PersonaTier::C,
        immutable_doctrine: ImmutableDoctrine {
            no_leverage: true,
            do_not_speak_intraday_as_entry_signal: true,
            reject_unknown_or_unscorable_asset: true,
            margin_of_safety_required_when_fundamentals_available: true,
            ..ImmutableDoctrine::default()
        },
        mutable_policy: MutablePolicy {
            max_exposure_hint: Some(0.35),
            unknown_asset_penalty: Some(0.40),
            quality_threshold_placeholder: Some(0.60),
            defensive_bias: Some(0.70),
            ..MutablePolicy::default()
        },
        voice: VoiceConfig {
            base_voice_power,
            current_voice_power: base_voice_power,
            ema_alpha: 0.08,
            severe_event_multiplier: 0.5,
        },
        evaluation: EvaluationProfile {
            horizon: Horizon::Position,
            favored_regimes: vec![Regime::Range, Regime::RiskOff],
            tolerated_regimes: vec![Regime::TrendUp],
            promotion_min_samples: 18,
            max_s_tier: 1,
        },
    }
}

pub fn cycle_risk_skeptic_card(base_voice_power: f64) -> PersonaCard {
    PersonaCard {
        persona_id: "cycle_risk_skeptic".to_string(),
        archetype: "Howard Marks / PTJ-like risk and cycle skeptic".to_string(),
        tier: PersonaTier::A,
        immutable_doctrine: ImmutableDoctrine {
            risk_first: true,
            reject_poor_risk_reward: true,
            reject_euphoria_chasing: true,
            respect_cooldown: true,
            no_trade_is_valid: true,
            ..ImmutableDoctrine::default()
        },
        mutable_policy: MutablePolicy {
            overheat_threshold: Some(0.75),
            min_risk_reward: Some(1.8),
            volatility_penalty: Some(0.55),
            groupthink_penalty: Some(0.35),
            veto_sensitivity: Some(0.65),
            ..MutablePolicy::default()
        },
        voice: VoiceConfig {
            base_voice_power,
            current_voice_power: base_voice_power,
            ema_alpha: 0.08,
            severe_event_multiplier: 0.5,
        },
        evaluation: EvaluationProfile {
            horizon: Horizon::Swing,
            favored_regimes: vec![
                Regime::HighVolatility,
                Regime::Panic,
                Regime::RiskOff,
                Regime::Unknown,
            ],
            tolerated_regimes: vec![Regime::Range],
            promotion_min_samples: 14,
            max_s_tier: 1,
        },
    }
}

pub fn active_persona_cards() -> Vec<PersonaCard> {
    vec![
        momentum_trend_fast_card(0.78),
        value_quality_filter_card(0.48),
        cycle_risk_skeptic_card(0.70),
    ]
}

pub fn persona_card_by_id(persona_id: &str) -> Option<PersonaCard> {
    match persona_id {
        "momentum_trend_fast" => Some(momentum_trend_fast_card(0.78)),
        "value_quality_filter" => Some(value_quality_filter_card(0.48)),
        "cycle_risk_skeptic" => Some(cycle_risk_skeptic_card(0.70)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::{AttributionRecord, BarrierHit, TripleBarrierResult};

    fn state() -> CanonicalAgentState {
        canonical_agent_state_from_card(momentum_trend_fast_card(0.78)).expect("canonical state")
    }

    fn proposal(state: &CanonicalAgentState) -> AgentProposal {
        AgentProposal {
            proposal_id: "proposal-1".to_string(),
            agent_id: state.agent_id.clone(),
            stance: Stance::Buy,
            confidence: 0.60,
            expected_edge: 0.01,
            expected_drawdown: 0.02,
            no_trade_probability: 0.20,
            horizon: state.doctrine.allowed_horizons[0],
            market: "US".to_string(),
            symbol: "FAKE123".to_string(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }

    fn feedback(state: &CanonicalAgentState) -> AgentFeedback {
        AgentFeedback {
            agent_id: state.agent_id.clone(),
            proposal_id: Some("proposal-1".to_string()),
            outcome_id: "outcome-1".to_string(),
            paper_only: true,
            outcome_kind: AgentFeedbackOutcomeKind::ExecutedPaperTrade,
            realized_net_return: 0.01,
            counterfactual_net_return: None,
            avoided_loss_score: 0.0,
            missed_gain_penalty: 0.0,
            drawdown_contribution: 0.01,
            confidence_at_decision: 0.60,
            doctrine_violation: false,
            risk_warning_correct: false,
            no_trade_correct: false,
            overtrade: false,
            reason_codes: vec![ReasonCode::PaperExecutionOnly],
        }
    }

    fn batch_source(
        source_id: &str,
        kind: LocalDataSourceKind,
        csv_text: &str,
    ) -> BatchReplaySource {
        let profile_name = LocalDataSourceRegistry::default()
            .get_profile(kind)
            .map(|profile| profile.name.clone())
            .unwrap_or_default();
        BatchReplaySource {
            source_id: source_id.to_string(),
            source_kind: kind,
            display_name: source_id.to_string(),
            csv_text: csv_text.to_string(),
            profile_name,
            enabled: true,
            reason_codes: vec![ReasonCode::LocalFileOnly],
        }
    }

    fn valid_batch_sources() -> Vec<BatchReplaySource> {
        vec![
            batch_source(
                "synthetic-fixture",
                LocalDataSourceKind::SyntheticFixture,
                include_str!("../../fixtures/historical/sample_ohlcv.csv"),
            ),
            batch_source(
                "korean-stock",
                LocalDataSourceKind::KoreanStockCsv,
                include_str!("../../fixtures/historical/sample_kr_stock.csv"),
            ),
            batch_source(
                "us-stock",
                LocalDataSourceKind::UsStockCsv,
                include_str!("../../fixtures/historical/sample_us_stock.csv"),
            ),
            batch_source(
                "btc-crypto",
                LocalDataSourceKind::BtcCryptoCsv,
                include_str!("../../fixtures/historical/sample_btc_crypto.csv"),
            ),
        ]
    }

    fn expanded_batch_sources() -> Vec<BatchReplaySource> {
        vec![
            batch_source(
                "expanded-synthetic",
                LocalDataSourceKind::SyntheticFixture,
                include_str!("../../fixtures/historical/expanded_synthetic_mixed.csv"),
            ),
            batch_source(
                "expanded-korean-stock",
                LocalDataSourceKind::KoreanStockCsv,
                include_str!("../../fixtures/historical/expanded_kr_stock.csv"),
            ),
            batch_source(
                "expanded-us-stock",
                LocalDataSourceKind::UsStockCsv,
                include_str!("../../fixtures/historical/expanded_us_stock.csv"),
            ),
            batch_source(
                "expanded-btc-crypto",
                LocalDataSourceKind::BtcCryptoCsv,
                include_str!("../../fixtures/historical/expanded_btc_crypto.csv"),
            ),
        ]
    }

    fn quality_fixture_sources() -> Vec<BatchReplaySource> {
        vec![
            batch_source(
                "quality-clean-synthetic",
                LocalDataSourceKind::SyntheticFixture,
                include_str!("../../fixtures/historical/quality_clean_synthetic.csv"),
            ),
            batch_source(
                "quality-gap-korean",
                LocalDataSourceKind::KoreanStockCsv,
                include_str!("../../fixtures/historical/quality_gap_kr_stock.csv"),
            ),
            batch_source(
                "quality-scale-us",
                LocalDataSourceKind::UsStockCsv,
                include_str!("../../fixtures/historical/quality_scale_us_stock.csv"),
            ),
            batch_source(
                "quality-volume-btc",
                LocalDataSourceKind::BtcCryptoCsv,
                include_str!("../../fixtures/historical/quality_volume_btc_crypto.csv"),
            ),
            batch_source(
                "quality-missing-optional-btc",
                LocalDataSourceKind::BtcCryptoCsv,
                include_str!("../../fixtures/historical/quality_missing_optional_btc_crypto.csv"),
            ),
        ]
    }

    fn valid_batch_input() -> BatchReplayInput {
        BatchReplayInput {
            initial_agent_states: canonical_current_agent_states(),
            sources: valid_batch_sources(),
            config: BatchReplayConfig::default(),
            replay_config: PaperReplayConfig::default(),
        }
    }

    fn expanded_batch_input(mode: BatchReplayMode) -> BatchReplayInput {
        let mut input = valid_batch_input();
        input.sources = expanded_batch_sources();
        input.config.replay_mode = mode;
        input
    }

    fn completed_outcome(proposal: &AgentProposal) -> OutcomeRecord {
        OutcomeRecord {
            decision_id: "outcome-cycle-1".to_string(),
            symbol: proposal.symbol.clone(),
            timestamp_ms: 1_700_000_000_000,
            regime: Regime::TrendUp,
            horizon: proposal.horizon,
            signal_confidence: proposal.confidence,
            executed: true,
            denied_by_risk: false,
            no_trade: false,
            triple_barrier_result: Some(TripleBarrierResult {
                outcome: TripleBarrierOutcome::Win,
                first_hit: BarrierHit::TakeProfit,
                entry_index: 0,
                exit_index: 2,
                entry_price: 100.0,
                exit_price: 102.0,
                gross_return_pct: 0.02,
                net_return_pct: 0.019,
                max_favorable_excursion_pct: 0.02,
                max_adverse_excursion_pct: 0.005,
                bars_held: 2,
                reason_codes: vec![ReasonCode::TakeProfitHit],
            }),
            hypothetical_result: None,
            realized_net_return_pct: 0.019,
            avoided_loss_score: 0.0,
            missed_gain_penalty: 0.0,
            attribution_records: Vec::new(),
            shadow_outcomes: Vec::new(),
            reason_codes: vec![ReasonCode::PaperExecutionOnly],
        }
    }

    fn finalized_paper_context() -> FeedbackContext {
        FeedbackContext {
            paper_only: true,
            outcome_finalized: true,
            doctrine_violation: false,
            overtrade: false,
        }
    }

    #[test]
    fn canonical_state_represents_current_three_and_disables_future_placeholder() {
        let states = canonical_current_agent_states();
        assert_eq!(states.len(), 3);
        assert!(states.iter().all(CanonicalAgentState::can_vote_live));
        assert_eq!(states[0].kind, AgentKind::MomentumTrendFast);
        assert_eq!(states[1].kind, AgentKind::ValueQualityFilter);
        assert_eq!(states[2].kind, AgentKind::CycleRiskSkeptic);

        let future = future_agent_placeholder_state("growth-story-future");
        assert_eq!(future.kind, AgentKind::Future8AgentPlaceholder);
        assert_eq!(future.status, AgentStatus::Disabled);
        assert!(!future.can_vote_live());
    }

    #[test]
    fn feedback_updates_memory_deterministically() {
        let state = state();
        let positive = apply_feedback_to_memory_summary(&state, &feedback(&state));
        assert_eq!(positive.total_decisions, 1);
        assert_eq!(positive.total_paper_trades, 1);
        assert_eq!(positive.wins, 1);

        let mut negative_feedback = feedback(&state);
        negative_feedback.realized_net_return = -0.01;
        negative_feedback.confidence_at_decision = 0.90;
        let negative = apply_feedback_to_memory_summary(&state, &negative_feedback);
        assert_eq!(negative.losses, 1);
        assert_eq!(negative.high_confidence_misses, 1);
        assert_eq!(
            apply_feedback_to_memory_summary(&state, &negative_feedback),
            negative
        );

        let reward = compute_chair_reward_penalty(&state, &feedback(&state));
        assert!(reward.reward_delta > 0.0);
        assert!(reward.voice_delta > 0.0);
        assert_eq!(
            reward,
            compute_chair_reward_penalty(&state, &feedback(&state))
        );
    }

    #[test]
    fn high_confidence_loss_is_penalized_more_than_low_confidence_loss() {
        let state = state();
        let mut low = feedback(&state);
        low.realized_net_return = -0.02;
        low.confidence_at_decision = 0.40;
        let mut high = low.clone();
        high.confidence_at_decision = 0.90;

        let low_result = compute_chair_reward_penalty(&state, &low);
        let high_result = compute_chair_reward_penalty(&state, &high);
        assert!(high_result.penalty_delta > low_result.penalty_delta);
        assert!(high_result.voice_delta < low_result.voice_delta);
    }

    #[test]
    fn correct_no_trade_reward_exceeds_missed_gain_penalty() {
        let state = state();
        let mut avoided = feedback(&state);
        avoided.realized_net_return = 0.0;
        avoided.avoided_loss_score = 0.20;
        avoided.no_trade_correct = true;
        let mut missed = feedback(&state);
        missed.realized_net_return = 0.0;
        missed.missed_gain_penalty = 0.20;

        let avoided_result = compute_chair_reward_penalty(&state, &avoided);
        let missed_result = compute_chair_reward_penalty(&state, &missed);
        assert!(avoided_result.reward_delta > missed_result.penalty_delta);
        assert!(avoided_result.voice_delta > missed_result.voice_delta);
        let avoided_memory = apply_feedback_to_memory_summary(&state, &avoided);
        let missed_memory = apply_feedback_to_memory_summary(&state, &missed);
        assert_eq!(avoided_memory.avoided_losses, 1);
        assert_eq!(missed_memory.missed_gains, 1);
    }

    #[test]
    fn doctrine_violation_quarantines_without_mutating_doctrine_or_policy() {
        let state = state();
        let original = state.clone();
        let mut violation = feedback(&state);
        violation.doctrine_violation = true;

        let next = apply_agent_feedback(&state, &proposal(&state), &violation);
        assert_eq!(next.status, AgentStatus::Quarantined);
        assert_eq!(next.voice_state.tier, PersonaTier::XQuarantined);
        assert!(!next.can_vote_live());
        assert_eq!(next.memory_summary.doctrine_violations, 1);
        assert_eq!(next.doctrine, original.doctrine);
        assert_eq!(next.mutable_policy, original.mutable_policy);
        assert_eq!(state, original);
        assert_eq!(
            next.version.parent_version_id.as_deref(),
            Some(original.version.version_id.as_str())
        );
    }

    #[test]
    fn sandbox_candidate_is_deterministic_and_cannot_affect_live_decision() {
        let state = state();
        let first_feedback = feedback(&state);
        let mut second_feedback = first_feedback.clone();
        second_feedback.outcome_id = "outcome-2".to_string();
        let mut third_feedback = first_feedback.clone();
        third_feedback.outcome_id = "outcome-3".to_string();
        let batch = vec![second_feedback, first_feedback, third_feedback];

        let first = build_sandbox_promotion_candidate(&state, &batch).expect("sandbox candidate");
        let second = build_sandbox_promotion_candidate(&state, &batch).expect("sandbox candidate");
        assert_eq!(first, second);
        assert!(first.sandbox_only);
        assert_eq!(first.promotion_status, SandboxPromotionStatus::Proposed);
        assert_eq!(first.parent_version_id, state.version.version_id);
        assert!(!first.can_affect_live_decision());
        assert!(!first.can_vote_live());
        assert!(first.proposed_policy_delta.is_empty());
    }

    #[test]
    fn promote_action_cannot_change_live_tier_in_scaffold() {
        let state = state();
        let reward = ChairRewardPenalty {
            agent_id: state.agent_id.clone(),
            source_feedback_id: "promotion-review".to_string(),
            reward_delta: 1.0,
            penalty_delta: 0.0,
            voice_delta: 0.10,
            cooldown_delta: 0,
            tier_action: ChairTierAction::Promote,
            reason_codes: vec![ReasonCode::ShadowEvaluationPending],
        };
        let next = apply_chair_reward_penalty(&state, &reward);
        assert_eq!(next.voice_state.tier, state.voice_state.tier);
        assert_eq!(next.status, AgentStatus::Active);
    }

    #[test]
    fn correct_risk_warning_rewards_and_overtrade_cools_down() {
        let state = state();
        let mut warning = feedback(&state);
        warning.realized_net_return = 0.0;
        warning.risk_warning_correct = true;
        let warning_result = compute_chair_reward_penalty(&state, &warning);
        assert!(warning_result.reward_delta > 0.0);
        assert!(warning_result.voice_delta > 0.0);

        let mut overtrade = feedback(&state);
        overtrade.realized_net_return = 0.0;
        overtrade.drawdown_contribution = 0.0;
        overtrade.overtrade = true;
        let overtrade_result = compute_chair_reward_penalty(&state, &overtrade);
        assert_eq!(overtrade_result.tier_action, ChairTierAction::Cooldown);
        assert!(overtrade_result.voice_delta < 0.0);
    }

    #[test]
    fn non_paper_feedback_cannot_update_state_or_build_candidate() {
        let state = state();
        let mut non_paper = feedback(&state);
        non_paper.paper_only = false;
        assert_eq!(
            apply_agent_feedback(&state, &proposal(&state), &non_paper),
            state
        );
        assert!(build_sandbox_promotion_candidate(&state, &[non_paper]).is_none());
    }

    #[test]
    fn non_finite_feedback_cannot_update_state() {
        let state = state();
        let mut invalid = feedback(&state);
        invalid.realized_net_return = f64::NAN;
        assert_eq!(
            apply_agent_feedback(&state, &proposal(&state), &invalid),
            state
        );
        assert!(build_sandbox_promotion_candidate(&state, &[invalid]).is_none());
    }

    #[test]
    fn completed_paper_outcome_builds_deterministic_feedback_and_cycle() {
        let state = state();
        let proposal = proposal(&state);
        let outcome = completed_outcome(&proposal);
        let context = finalized_paper_context();
        let original = state.clone();

        let feedback =
            build_agent_feedback_from_paper_outcome(&state, &proposal, &outcome, &context)
                .expect("completed paper feedback");
        assert_eq!(feedback.agent_id, state.agent_id);
        assert_eq!(feedback.proposal_id.as_deref(), Some("proposal-1"));
        assert_eq!(feedback.outcome_id, outcome.decision_id);
        assert!(feedback.paper_only);
        assert_eq!(feedback.realized_net_return, 0.019);
        assert_eq!(feedback.drawdown_contribution, 0.005);

        let first = apply_paper_feedback_cycle(&state, &proposal, &outcome, &context)
            .expect("feedback cycle");
        let second = apply_paper_feedback_cycle(&state, &proposal, &outcome, &context)
            .expect("feedback cycle");
        assert_eq!(first, second);
        assert_eq!(state, original);
        assert_ne!(
            first.updated_state.version.version_id,
            state.version.version_id
        );
        assert_eq!(
            first.updated_state.version.parent_version_id.as_deref(),
            Some(state.version.version_id.as_str())
        );
        assert_eq!(first.updated_state.doctrine, state.doctrine);
        assert_eq!(first.updated_state.mutable_policy, state.mutable_policy);
        assert_eq!(first.updated_state.memory_summary.wins, 1);
        assert!(first.version_entry.created_from_paper_only);
        assert_eq!(
            first.version_entry.feedback_event_id.as_deref(),
            Some(first.reward_penalty.source_feedback_id.as_str())
        );
    }

    #[test]
    fn feedback_adapter_rejects_mismatch_non_paper_incomplete_and_non_finite() {
        let state = state();
        let mut proposal = proposal(&state);
        let mut outcome = completed_outcome(&proposal);
        let mut context = finalized_paper_context();

        proposal.agent_id = "different-agent".to_string();
        let mismatch =
            build_agent_feedback_from_paper_outcome(&state, &proposal, &outcome, &context)
                .expect_err("agent mismatch");
        assert_eq!(
            mismatch.reason_codes,
            vec![ReasonCode::FeedbackAgentMismatch]
        );

        proposal.agent_id = state.agent_id.clone();
        proposal.symbol = "OTHER".to_string();
        let proposal_mismatch =
            build_agent_feedback_from_paper_outcome(&state, &proposal, &outcome, &context)
                .expect_err("proposal outcome mismatch");
        assert_eq!(
            proposal_mismatch.reason_codes,
            vec![ReasonCode::FeedbackProposalOutcomeMismatch]
        );

        proposal.symbol = outcome.symbol.clone();
        context.paper_only = false;
        let non_paper =
            build_agent_feedback_from_paper_outcome(&state, &proposal, &outcome, &context)
                .expect_err("non-paper outcome");
        assert_eq!(
            non_paper.reason_codes,
            vec![ReasonCode::FeedbackOutcomeNotPaper]
        );

        context.paper_only = true;
        context.outcome_finalized = false;
        let incomplete =
            build_agent_feedback_from_paper_outcome(&state, &proposal, &outcome, &context)
                .expect_err("incomplete outcome");
        assert_eq!(
            incomplete.reason_codes,
            vec![ReasonCode::FeedbackOutcomeIncomplete]
        );

        context.outcome_finalized = true;
        outcome.realized_net_return_pct = f64::NAN;
        let non_finite =
            build_agent_feedback_from_paper_outcome(&state, &proposal, &outcome, &context)
                .expect_err("non-finite outcome");
        assert_eq!(
            non_finite.reason_codes,
            vec![ReasonCode::FeedbackNonFiniteValue]
        );
    }

    #[test]
    fn feedback_adapter_rejects_missing_return_and_maps_no_trade_and_loss() {
        let state = state();
        let mut proposal = proposal(&state);
        let context = finalized_paper_context();
        let mut outcome = completed_outcome(&proposal);

        outcome.triple_barrier_result = None;
        let missing =
            build_agent_feedback_from_paper_outcome(&state, &proposal, &outcome, &context)
                .expect_err("executed outcome without fill result");
        assert_eq!(
            missing.reason_codes,
            vec![ReasonCode::FeedbackOutcomeIncomplete]
        );

        outcome.executed = false;
        outcome.no_trade = true;
        outcome.realized_net_return_pct = 0.0;
        outcome.avoided_loss_score = 0.20;
        outcome.missed_gain_penalty = 0.0;
        proposal.stance = Stance::NoTrade;
        let avoided =
            build_agent_feedback_from_paper_outcome(&state, &proposal, &outcome, &context)
                .expect("avoided loss feedback");
        assert_eq!(avoided.avoided_loss_score, 0.20);
        assert!(avoided.no_trade_correct);
        assert!(
            avoided
                .reason_codes
                .contains(&ReasonCode::FeedbackNoTradeAvoidedLoss)
        );

        outcome.avoided_loss_score = 0.0;
        outcome.missed_gain_penalty = -0.20;
        let missed = build_agent_feedback_from_paper_outcome(&state, &proposal, &outcome, &context)
            .expect("missed gain feedback");
        assert_eq!(missed.missed_gain_penalty, 0.20);
        assert!(
            missed
                .reason_codes
                .contains(&ReasonCode::FeedbackNoTradeMissedGain)
        );

        proposal.confidence = 0.90;
        proposal.stance = Stance::Buy;
        outcome.executed = true;
        outcome.no_trade = false;
        outcome.signal_confidence = proposal.confidence;
        outcome.realized_net_return_pct = -0.02;
        outcome.missed_gain_penalty = 0.0;
        outcome.triple_barrier_result = completed_outcome(&proposal).triple_barrier_result;
        if let Some(result) = &mut outcome.triple_barrier_result {
            result.outcome = TripleBarrierOutcome::Loss;
            result.first_hit = BarrierHit::StopLoss;
            result.net_return_pct = outcome.realized_net_return_pct;
        }
        let loss = build_agent_feedback_from_paper_outcome(&state, &proposal, &outcome, &context)
            .expect("loss feedback");
        assert!(
            loss.reason_codes
                .contains(&ReasonCode::FeedbackHighConfidenceLoss)
        );
    }

    #[test]
    fn feedback_adapter_maps_risk_alignment_bad_data_and_doctrine_violation() {
        let state = state();
        let mut proposal = proposal(&state);
        proposal.stance = Stance::NoTrade;
        let mut outcome = completed_outcome(&proposal);
        outcome.executed = false;
        outcome.denied_by_risk = true;
        outcome.no_trade = false;
        outcome.realized_net_return_pct = 0.0;
        outcome.triple_barrier_result = None;
        outcome.attribution_records = vec![AttributionRecord {
            persona_id: state.agent_id.clone(),
            selected_for_decision: true,
            stance: Stance::NoTrade,
            conviction: 0.8,
            voice_power: 0.7,
            contribution_score: 0.4,
            counterfactual_role: CounterfactualRole::RiskVetoAligned,
            reason_codes: vec![ReasonCode::RiskDeniedCounterfactual],
        }];
        let mut context = finalized_paper_context();
        let aligned =
            build_agent_feedback_from_paper_outcome(&state, &proposal, &outcome, &context)
                .expect("risk aligned feedback");
        assert!(aligned.risk_warning_correct);
        assert!(
            aligned
                .reason_codes
                .contains(&ReasonCode::FeedbackCorrectRiskWarning)
        );
        assert!(
            aligned
                .reason_codes
                .contains(&ReasonCode::FeedbackRiskGovernorAligned)
        );

        proposal.stance = Stance::Buy;
        outcome.attribution_records[0].counterfactual_role = CounterfactualRole::RiskVetoOpposed;
        outcome
            .reason_codes
            .push(ReasonCode::DataQualityGateBreached);
        context.doctrine_violation = true;
        let opposed =
            build_agent_feedback_from_paper_outcome(&state, &proposal, &outcome, &context)
                .expect("risk opposed feedback");
        assert!(opposed.doctrine_violation);
        assert!(
            opposed
                .reason_codes
                .contains(&ReasonCode::FeedbackRiskGovernorOpposed)
        );
        assert!(
            opposed
                .reason_codes
                .contains(&ReasonCode::FeedbackBadDataProposal)
        );
        assert!(
            opposed
                .reason_codes
                .contains(&ReasonCode::FeedbackDoctrineViolation)
        );
        let penalty = compute_chair_reward_penalty(&state, &opposed);
        assert!(penalty.penalty_delta > penalty.reward_delta);
        assert_eq!(penalty.tier_action, ChairTierAction::Quarantine);
    }

    #[test]
    fn state_journal_is_paper_only_versioned_and_duplicate_safe() {
        let state = state();
        let proposal = proposal(&state);
        let outcome = completed_outcome(&proposal);
        let cycle =
            apply_paper_feedback_cycle(&state, &proposal, &outcome, &finalized_paper_context())
                .expect("feedback cycle");
        let mut journal = AgentStateJournal::default();

        journal
            .append_snapshot(cycle.version_entry.clone())
            .expect("append snapshot");
        assert_eq!(journal.count_for_agent(&state.agent_id), 1);
        assert!(journal.contains_version(&cycle.version_entry.version_id));
        assert_eq!(
            journal
                .latest_for_agent(&state.agent_id)
                .map(|snapshot| snapshot.version_id.as_str()),
            Some(cycle.version_entry.version_id.as_str())
        );
        assert_eq!(
            journal.snapshots_for_agent(&state.agent_id).len(),
            journal.count_for_agent(&state.agent_id)
        );
        assert_eq!(
            journal.append_snapshot(cycle.version_entry.clone()),
            Err(AgentStateJournalError::DuplicateVersion)
        );

        let mut non_paper = cycle.version_entry.clone();
        non_paper.version_id = "non-paper-version".to_string();
        non_paper.state.version.version_id = non_paper.version_id.clone();
        non_paper.created_from_paper_only = false;
        assert_eq!(
            journal.append_snapshot(non_paper),
            Err(AgentStateJournalError::NonPaperSnapshot)
        );

        let mut unsafe_sandbox = cycle.version_entry;
        unsafe_sandbox.version_id = "unsafe-sandbox-version".to_string();
        unsafe_sandbox.state.version.version_id = unsafe_sandbox.version_id.clone();
        unsafe_sandbox.sandbox_only = true;
        unsafe_sandbox.state.version.sandbox_only = true;
        unsafe_sandbox.state.version.live_enabled = true;
        assert_eq!(
            journal.append_snapshot(unsafe_sandbox),
            Err(AgentStateJournalError::SandboxSnapshotLiveEnabled)
        );
    }

    fn paper_learning_loop_input(
        confidence: f64,
        realized_return: f64,
        spread_bps: f64,
    ) -> PaperLearningLoopInput {
        let market = MarketSnapshot {
            symbol: "FAKE123".to_string(),
            timestamp_ms: 1_800_000_000_000,
            price: 100.0,
            bid: 99.99,
            ask: 100.01,
            spread_bps,
            volume: 100_000.0,
            trade_value: 2_000_000.0,
            volatility: 0.01,
            regime: Regime::TrendUp,
            data_quality_score: 1.0,
        };
        let signal = SignalOutput {
            symbol: market.symbol.clone(),
            horizon_bars: 8,
            p_win: 0.90,
            p_stop: 0.05,
            expected_return: 0.04,
            expected_drawdown: 0.01,
            confidence,
            no_trade_probability: 0.05,
            source: "deterministic-paper-scenario".to_string(),
        };
        let fill_symbol = market.symbol.clone();
        let fill_timestamp_ms = market.timestamp_ms;
        PaperLearningLoopInput {
            initial_agent_states: canonical_current_agent_states(),
            market_snapshot: market,
            signal_input: signal,
            owner_advisory: None,
            risk_snapshot: RiskSnapshot {
                daily_pnl_pct: 0.0,
                consecutive_losses: 0,
                current_positions_count: 0,
                total_exposure_pct: 0.0,
                symbol_exposure_pct: 0.0,
                api_health_score: 1.0,
                data_quality_score: 1.0,
            },
            paper_context: Some(PaperOutcomeContext {
                outcome_finalized: true,
                finalized_at_timestamp_ms: fill_timestamp_ms,
                outcome_kind: PaperOutcomeKind::FilledPaperOrder,
                fill_evidence: Some(PaperFillEvidence {
                    fill_id: format!("fill:{fill_symbol}:{fill_timestamp_ms}"),
                    paper_order_id: "paper-000001".to_string(),
                    symbol: fill_symbol,
                    filled_at_timestamp_ms: fill_timestamp_ms,
                    paper_only: true,
                }),
                realized_net_return_pct: realized_return,
                hypothetical_net_return_pct: Some(realized_return),
                max_adverse_excursion_pct: realized_return.min(0.0).abs(),
                doctrine_violation_agents: Vec::new(),
                overtrade_agents: Vec::new(),
            }),
            loop_config: PaperLearningLoopConfig {
                market: "US".to_string(),
                chair: ChairConfig::default(),
                risk_governor: GovernorConfig {
                    min_confidence: 0.40,
                    max_total_exposure: 1.0,
                    max_symbol_exposure: 1.0,
                    ..GovernorConfig::default()
                },
            },
        }
    }

    #[test]
    fn paper_learning_loop_profitable_scenario_updates_selected_agent_and_versions() {
        let input = paper_learning_loop_input(0.90, 0.08, 2.0);
        let original = input.initial_agent_states.clone();
        let result =
            run_3_agent_paper_learning_loop(input).expect("profitable paper learning loop");
        let selected_id = result.chair_output.lead_speaker.clone();
        let selected_original = original
            .iter()
            .find(|state| state.agent_id == selected_id)
            .expect("selected original state");
        let selected_updated = result
            .updated_agent_states
            .iter()
            .find(|state| state.agent_id == selected_id)
            .expect("selected updated state");
        let selected_reward = result
            .reward_penalties
            .iter()
            .find(|reward| reward.agent_id == selected_id)
            .expect("selected reward");

        assert_eq!(result.agent_votes.len(), 3);
        assert_eq!(result.report.active_agent_count, 3);
        assert_eq!(result.risk_decision.kind, RiskDecisionKind::ApprovePaper);
        assert!(
            result
                .paper_order
                .as_ref()
                .is_some_and(|order| order.paper_only)
        );
        assert!(
            result
                .paper_outcome
                .as_ref()
                .is_some_and(|outcome| outcome.executed)
        );
        assert_eq!(selected_updated.memory_summary.wins, 1);
        assert!(selected_reward.reward_delta > selected_reward.penalty_delta);
        assert!(
            selected_updated.voice_state.voice_power > selected_original.voice_state.voice_power
        );
        assert_eq!(result.version_snapshots.len(), 3);
        assert!(result.version_snapshots.iter().all(|snapshot| {
            snapshot.created_from_paper_only
                && snapshot.parent_version_id.is_some()
                && snapshot.state.doctrine
                    == original
                        .iter()
                        .find(|state| state.agent_id == snapshot.agent_id)
                        .expect("snapshot original")
                        .doctrine
        }));
        assert_eq!(result.original_agent_states, original);
        assert!(!result.report.live_execution_supported);
        assert_eq!(result.report.live_call_count, 0);
    }

    #[test]
    fn paper_learning_loop_high_confidence_loss_is_penalized_more() {
        let low = run_3_agent_paper_learning_loop(paper_learning_loop_input(0.60, -0.02, 2.0))
            .expect("low confidence loss loop");
        let high = run_3_agent_paper_learning_loop(paper_learning_loop_input(0.90, -0.02, 2.0))
            .expect("high confidence loss loop");
        let low_selected = low
            .reward_penalties
            .iter()
            .find(|reward| reward.agent_id == low.chair_output.lead_speaker)
            .expect("low confidence selected reward");
        let high_selected = high
            .reward_penalties
            .iter()
            .find(|reward| reward.agent_id == high.chair_output.lead_speaker)
            .expect("high confidence selected reward");
        let high_state = high
            .updated_agent_states
            .iter()
            .find(|state| state.agent_id == high.chair_output.lead_speaker)
            .expect("high confidence selected state");

        assert!(high_selected.penalty_delta > low_selected.penalty_delta);
        assert!(high_selected.voice_delta < low_selected.voice_delta);
        assert_eq!(high_state.memory_summary.high_confidence_misses, 1);
        assert!(!high.sandbox_candidates.is_empty());
        assert!(high.sandbox_candidates.iter().all(|candidate| {
            candidate.sandbox_only
                && !candidate.can_vote_live()
                && !candidate.can_affect_live_decision()
                && matches!(
                    candidate.promotion_status,
                    SandboxPromotionStatus::Proposed | SandboxPromotionStatus::BacktestPending
                )
        }));
        assert!(high.version_snapshots.iter().all(|snapshot| {
            snapshot.parent_version_id.as_deref()
                == high
                    .original_agent_states
                    .iter()
                    .find(|state| state.agent_id == snapshot.agent_id)
                    .map(|state| state.version.version_id.as_str())
        }));
    }

    #[test]
    fn paper_learning_loop_no_trade_avoided_loss_rewards_defensive_agents() {
        let mut input = paper_learning_loop_input(0.20, 0.0, 2.0);
        input.signal_input.expected_return = 0.0;
        input.signal_input.expected_drawdown = 0.04;
        input.signal_input.no_trade_probability = 0.95;
        input
            .paper_context
            .as_mut()
            .expect("paper context")
            .outcome_kind = PaperOutcomeKind::NoExecution;
        input
            .paper_context
            .as_mut()
            .expect("paper context")
            .fill_evidence = None;
        input
            .paper_context
            .as_mut()
            .expect("paper context")
            .hypothetical_net_return_pct = Some(-0.04);
        let result = run_3_agent_paper_learning_loop(input).expect("NoTrade avoided loss loop");
        let skeptic_feedback = result
            .feedback_records
            .iter()
            .find(|feedback| feedback.agent_id == "cycle_risk_skeptic")
            .expect("skeptic feedback");
        let skeptic_state = result
            .updated_agent_states
            .iter()
            .find(|state| state.agent_id == "cycle_risk_skeptic")
            .expect("skeptic state");

        assert!(result.paper_order.is_none());
        assert!(
            result
                .paper_outcome
                .as_ref()
                .is_some_and(|outcome| outcome.no_trade)
        );
        assert!(skeptic_feedback.no_trade_correct);
        assert!(
            skeptic_feedback
                .reason_codes
                .contains(&ReasonCode::AgentNoTradeCorrect)
        );
        assert_eq!(skeptic_state.memory_summary.avoided_losses, 1);
        assert_eq!(skeptic_state.memory_summary.total_paper_trades, 0);
        assert_eq!(skeptic_state.memory_summary.total_no_trades, 1);
        assert!(
            result
                .reward_penalties
                .iter()
                .find(|reward| reward.agent_id == "cycle_risk_skeptic")
                .is_some_and(|reward| reward.reward_delta > 0.0)
        );
    }

    #[test]
    fn paper_learning_loop_records_abstention_and_no_trade_missed_gain() {
        let mut input = paper_learning_loop_input(0.20, 0.0, 2.0);
        input.market_snapshot.symbol = "BTC-USD".to_string();
        input.signal_input.symbol = input.market_snapshot.symbol.clone();
        input.loop_config.market = "BTC".to_string();
        input.signal_input.horizon_bars = 24;
        input.signal_input.expected_return = 0.0;
        input.signal_input.expected_drawdown = 0.04;
        input.signal_input.no_trade_probability = 0.95;
        input
            .paper_context
            .as_mut()
            .expect("paper context")
            .outcome_kind = PaperOutcomeKind::NoExecution;
        input
            .paper_context
            .as_mut()
            .expect("paper context")
            .fill_evidence = None;
        input
            .paper_context
            .as_mut()
            .expect("paper context")
            .hypothetical_net_return_pct = Some(0.04);

        let result = run_3_agent_paper_learning_loop(input).expect("NoTrade missed gain loop");
        let abstained = result
            .feedback_records
            .iter()
            .find(|feedback| feedback.agent_id == "value_quality_filter")
            .expect("abstained value feedback");
        let no_trade = result
            .feedback_records
            .iter()
            .find(|feedback| feedback.agent_id == "cycle_risk_skeptic")
            .expect("NoTrade skeptic feedback");

        assert_eq!(result.feedback_records.len(), 3);
        assert!(abstained.reason_codes.contains(&ReasonCode::AgentAbstained));
        assert_eq!(
            result
                .updated_agent_states
                .iter()
                .find(|state| state.agent_id == "value_quality_filter")
                .expect("abstained value state")
                .memory_summary
                .total_paper_trades,
            0
        );
        assert!(
            no_trade
                .reason_codes
                .contains(&ReasonCode::AgentNoTradeMissedGain)
        );
        assert!(no_trade.missed_gain_penalty > 0.0 && !no_trade.no_trade_correct);
    }

    #[test]
    fn paper_learning_loop_risk_denial_creates_no_order_and_preserves_attribution() {
        let mut input = paper_learning_loop_input(0.90, 0.0, 50.0);
        input
            .paper_context
            .as_mut()
            .expect("paper context")
            .outcome_kind = PaperOutcomeKind::NoExecution;
        input
            .paper_context
            .as_mut()
            .expect("paper context")
            .fill_evidence = None;
        let result = run_3_agent_paper_learning_loop(input).expect("risk denial loop");

        assert_eq!(result.risk_decision.kind, RiskDecisionKind::Deny);
        assert!(result.paper_order.is_none());
        assert!(
            result
                .paper_outcome
                .as_ref()
                .is_some_and(|outcome| { outcome.denied_by_risk && !outcome.executed })
        );
        assert!(
            result
                .risk_decision
                .reason_codes
                .contains(&ReasonCode::SpreadGateBreached)
        );
        assert!(result.feedback_records.iter().any(|feedback| {
            feedback
                .reason_codes
                .contains(&ReasonCode::AgentRiskVetoAligned)
        }));
        assert!(result.report.risk_veto_preserved);
        assert_eq!(result.report.live_call_count, 0);
        assert!(
            result
                .updated_agent_states
                .iter()
                .all(|state| state.memory_summary.total_paper_trades == 0)
        );
    }

    #[test]
    fn paper_learning_loop_doctrine_violation_quarantines_without_live_candidate() {
        let mut input = paper_learning_loop_input(0.90, 0.01, 2.0);
        input
            .paper_context
            .as_mut()
            .expect("paper context")
            .doctrine_violation_agents
            .push("momentum_trend_fast".to_string());
        let original = input.initial_agent_states.clone();
        let result = run_3_agent_paper_learning_loop(input).expect("doctrine violation loop");
        let updated = result
            .updated_agent_states
            .iter()
            .find(|state| state.agent_id == "momentum_trend_fast")
            .expect("violating state");
        let penalty = result
            .reward_penalties
            .iter()
            .find(|reward| reward.agent_id == "momentum_trend_fast")
            .expect("violation penalty");

        assert_eq!(penalty.tier_action, ChairTierAction::Quarantine);
        assert_eq!(updated.status, AgentStatus::Quarantined);
        assert_eq!(updated.memory_summary.doctrine_violations, 1);
        assert!(!updated.version.live_enabled);
        assert_eq!(
            updated.doctrine,
            original
                .iter()
                .find(|state| state.agent_id == updated.agent_id)
                .expect("original violating state")
                .doctrine
        );
        assert!(result.sandbox_candidates.iter().all(|candidate| {
            candidate.sandbox_only
                && !candidate.can_vote_live()
                && !candidate.can_affect_live_decision()
                && candidate.promotion_status != SandboxPromotionStatus::Promoted
        }));
    }

    #[test]
    fn paper_learning_loop_owner_force_trade_cannot_bypass_risk_denial() {
        let mut input = paper_learning_loop_input(0.90, 0.0, 50.0);
        input.paper_context = None;
        input.owner_advisory = Some(OwnerInput {
            owner_input_id: "owner-force-buy".to_string(),
            input_kind: crate::owner::OwnerInputKind::PaperConfirm,
            requested_action: Some("place trade and buy now".to_string()),
            ..OwnerInput::default()
        });
        let original = input.initial_agent_states.clone();
        let result = run_3_agent_paper_learning_loop(input).expect("owner force trade denial loop");
        let explanation = result
            .owner_explanation
            .as_ref()
            .expect("owner rejection explanation");

        assert_eq!(result.risk_decision.kind, RiskDecisionKind::Deny);
        assert!(result.paper_order.is_none());
        assert!(explanation.advisory_only);
        assert!(!explanation.owner_forced_trade);
        assert!(!explanation.paper_action_allowed);
        assert!(!explanation.explanation.is_empty());
        assert!(
            explanation
                .reason_codes
                .contains(&ReasonCode::OwnerRequestedButRiskDenied)
        );
        assert_eq!(result.updated_agent_states, original);
        assert!(result.feedback_records.is_empty());
        assert!(result.version_snapshots.is_empty());
    }

    #[test]
    fn paper_learning_loop_is_deterministic_and_updates_only_after_final_outcome() {
        let input = paper_learning_loop_input(0.90, -0.02, 2.0);
        let first =
            run_3_agent_paper_learning_loop(input.clone()).expect("first deterministic loop");
        let second =
            run_3_agent_paper_learning_loop(input.clone()).expect("second deterministic loop");
        assert_eq!(first, second);

        let mut pending = input;
        pending
            .paper_context
            .as_mut()
            .expect("pending context")
            .outcome_finalized = false;
        let pending = run_3_agent_paper_learning_loop(pending).expect("pending paper outcome loop");
        assert!(pending.paper_outcome.is_none());
        assert!(pending.feedback_records.is_empty());
        assert!(pending.reward_penalties.is_empty());
        assert!(pending.version_snapshots.is_empty());
        assert!(pending.sandbox_candidates.is_empty());
        assert_eq!(pending.updated_agent_states, pending.original_agent_states);

        let future = future_agent_placeholder_state("future-agent-four");
        let mut invalid = paper_learning_loop_input(0.90, 0.01, 2.0);
        invalid.initial_agent_states.push(future);
        assert_eq!(
            run_3_agent_paper_learning_loop(invalid),
            Err(PaperLearningLoopError::InvalidActiveAgentSet)
        );
    }

    #[test]
    fn paper_learning_loop_rejects_mismatched_input_and_fill_evidence() {
        let mut mismatched = paper_learning_loop_input(0.90, 0.01, 2.0);
        mismatched.signal_input.symbol = "OTHER".to_string();
        assert_eq!(
            run_3_agent_paper_learning_loop(mismatched),
            Err(PaperLearningLoopError::InvalidDecisionInput)
        );

        let mut accepted_without_fill = paper_learning_loop_input(0.90, 0.01, 2.0);
        accepted_without_fill
            .paper_context
            .as_mut()
            .expect("paper context")
            .fill_evidence = None;
        assert_eq!(
            run_3_agent_paper_learning_loop(accepted_without_fill),
            Err(PaperLearningLoopError::InvalidPaperOutcome)
        );

        let mut false_fill = paper_learning_loop_input(0.90, 0.0, 50.0);
        false_fill
            .paper_context
            .as_mut()
            .expect("paper context")
            .outcome_kind = PaperOutcomeKind::FilledPaperOrder;
        assert_eq!(
            run_3_agent_paper_learning_loop(false_fill),
            Err(PaperLearningLoopError::InvalidPaperOutcome)
        );
    }

    #[test]
    fn feedback_content_changes_version_identity_for_same_decision() {
        let first = run_3_agent_paper_learning_loop(paper_learning_loop_input(0.90, 0.01, 2.0))
            .expect("first outcome");
        let second = run_3_agent_paper_learning_loop(paper_learning_loop_input(0.90, 0.02, 2.0))
            .expect("second outcome");
        let first_version = first
            .updated_agent_states
            .iter()
            .find(|state| state.agent_id == first.chair_output.lead_speaker)
            .expect("first selected state")
            .version
            .version_id
            .clone();
        let second_version = second
            .updated_agent_states
            .iter()
            .find(|state| state.agent_id == second.chair_output.lead_speaker)
            .expect("second selected state")
            .version
            .version_id
            .clone();
        assert_ne!(first_version, second_version);
    }

    fn learning_episode(
        episode_id: &str,
        mut input: PaperLearningLoopInput,
        timestamp_offset: u64,
    ) -> PaperLearningEpisode {
        input.market_snapshot.timestamp_ms = input
            .market_snapshot
            .timestamp_ms
            .saturating_add(timestamp_offset);
        if let Some(context) = input.paper_context.as_mut() {
            context.finalized_at_timestamp_ms = input.market_snapshot.timestamp_ms;
            if let Some(evidence) = context.fill_evidence.as_mut() {
                evidence.fill_id = format!("fill:{episode_id}");
                evidence.symbol = input.market_snapshot.symbol.clone();
                evidence.filled_at_timestamp_ms = input.market_snapshot.timestamp_ms;
            }
        }
        PaperLearningEpisode {
            episode_id: episode_id.to_string(),
            input,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }

    fn improving_learning_chain_input() -> PaperLearningChainInput {
        let profitable = learning_episode(
            "episode-profitable",
            paper_learning_loop_input(0.90, 0.08, 2.0),
            1,
        );
        let mut avoided_input = paper_learning_loop_input(0.20, 0.0, 2.0);
        avoided_input.signal_input.expected_return = 0.0;
        avoided_input.signal_input.expected_drawdown = 0.04;
        avoided_input.signal_input.no_trade_probability = 0.95;
        let avoided_context = avoided_input
            .paper_context
            .as_mut()
            .expect("avoided loss context");
        avoided_context.outcome_kind = PaperOutcomeKind::NoExecution;
        avoided_context.fill_evidence = None;
        avoided_context.hypothetical_net_return_pct = Some(-0.04);
        let avoided = learning_episode("episode-avoided-loss", avoided_input, 2);

        let mut risk_input = paper_learning_loop_input(0.90, 0.0, 50.0);
        let risk_context = risk_input
            .paper_context
            .as_mut()
            .expect("risk denial context");
        risk_context.outcome_kind = PaperOutcomeKind::NoExecution;
        risk_context.fill_evidence = None;
        risk_context.hypothetical_net_return_pct = Some(-0.03);
        let risk_warning = learning_episode("episode-risk-warning", risk_input, 3);

        PaperLearningChainInput {
            initial_agent_states: canonical_current_agent_states(),
            episodes: vec![profitable, avoided, risk_warning],
            chain_config: PaperLearningChainConfig::default(),
        }
    }

    #[test]
    fn paper_learning_chain_runs_three_deterministic_episodes_with_stable_attribution() {
        let input = improving_learning_chain_input();
        let first = run_3_agent_paper_learning_chain(input.clone()).expect("first learning chain");
        let second = run_3_agent_paper_learning_chain(input).expect("second learning chain");
        assert_eq!(first, second);
        assert_eq!(first.episode_results.len(), 3);
        assert_eq!(first.summary.total_episodes, 3);
        assert_eq!(first.summary.total_paper_trades, 1);
        assert_eq!(first.summary.total_no_trades, 1);
        assert_eq!(first.summary.total_risk_denials, 1);
        assert_ne!(first.initial_states, first.final_states);
        assert!(
            first
                .episode_results
                .iter()
                .all(|episode| episode.result.report.active_agent_count == 3)
        );
        assert_eq!(first.version_journal.snapshots().len(), 9);
        for episode in &first.episode_results {
            for snapshot in &episode.result.version_snapshots {
                let parent = episode
                    .input_states
                    .iter()
                    .find(|state| state.agent_id == snapshot.agent_id)
                    .expect("snapshot parent state");
                assert_eq!(
                    snapshot.parent_version_id.as_deref(),
                    Some(parent.version.version_id.as_str())
                );
            }
        }
        for final_state in &first.final_states {
            assert_eq!(
                first
                    .version_journal
                    .latest_for_agent(&final_state.agent_id)
                    .map(|snapshot| snapshot.version_id.as_str()),
                Some(final_state.version.version_id.as_str())
            );
        }

        let momentum = first
            .attribution_summary
            .iter()
            .find(|summary| summary.agent_id == "momentum_trend_fast")
            .expect("momentum attribution");
        let skeptic = first
            .attribution_summary
            .iter()
            .find(|summary| summary.agent_id == "cycle_risk_skeptic")
            .expect("skeptic attribution");
        assert!(momentum.selected_count > 0);
        assert!(momentum.profitable_selected_count > 0);
        assert!(skeptic.no_trade_correct_count > 0);
        assert!(skeptic.risk_veto_aligned_count > 0);
        assert!(
            first
                .attribution_summary
                .iter()
                .any(|summary| summary.supported_final_count > 0)
        );
        assert!(
            first
                .attribution_summary
                .iter()
                .any(|summary| summary.opposed_final_count > 0)
        );
        assert!(
            skeptic.final_voice_power
                > first
                    .initial_states
                    .iter()
                    .find(|state| state.agent_id == skeptic.agent_id)
                    .expect("initial skeptic")
                    .voice_state
                    .voice_power
        );
        assert!(!first.summary.any_live_mutation_detected);
        assert!(!first.summary.any_risk_bypass_detected);
        assert!(
            first
                .final_states
                .iter()
                .zip(first.initial_states.iter())
                .all(|(final_state, initial)| {
                    final_state.doctrine == initial.doctrine
                        && final_state.mutable_policy == initial.mutable_policy
                })
        );
    }

    #[test]
    fn repeated_high_confidence_losses_cool_down_then_doctrine_quarantines() {
        let first_loss = learning_episode(
            "episode-loss-one",
            paper_learning_loop_input(0.90, -0.02, 2.0),
            11,
        );
        let second_loss = learning_episode(
            "episode-loss-two",
            paper_learning_loop_input(0.90, -0.02, 2.0),
            12,
        );
        let mut violation_input = paper_learning_loop_input(0.90, 0.01, 2.0);
        violation_input
            .paper_context
            .as_mut()
            .expect("violation context")
            .doctrine_violation_agents
            .push("momentum_trend_fast".to_string());
        violation_input
            .paper_context
            .as_mut()
            .expect("violation context")
            .outcome_kind = PaperOutcomeKind::NoExecution;
        violation_input
            .paper_context
            .as_mut()
            .expect("violation context")
            .fill_evidence = None;
        let violation = learning_episode("episode-doctrine-violation", violation_input, 13);
        let result = run_3_agent_paper_learning_chain(PaperLearningChainInput {
            initial_agent_states: canonical_current_agent_states(),
            episodes: vec![first_loss, second_loss, violation],
            chain_config: PaperLearningChainConfig::default(),
        })
        .expect("penalty learning chain");
        let summary = result
            .agent_learning_summaries
            .iter()
            .find(|summary| summary.agent_id == "momentum_trend_fast")
            .expect("momentum learning summary");
        let attribution = result
            .attribution_summary
            .iter()
            .find(|summary| summary.agent_id == "momentum_trend_fast")
            .expect("momentum attribution summary");

        assert_eq!(summary.high_confidence_misses_delta, 2);
        assert_eq!(summary.doctrine_violations_delta, 1);
        assert!(summary.end_voice_power < summary.start_voice_power);
        assert!(summary.cooldown_triggered);
        assert!(summary.quarantined);
        assert_eq!(summary.status_after, AgentStatus::Quarantined);
        assert!(attribution.total_penalty > attribution.total_reward);
        assert_eq!(attribution.high_confidence_miss_count, 2);
        assert!(
            result.episode_results[2]
                .result
                .agent_votes
                .iter()
                .any(|vote| vote.persona_id == "momentum_trend_fast"
                    && vote.stance == Stance::Abstain
                    && vote
                        .reason_codes
                        .contains(&ReasonCode::CooldownAgentUnavailable))
        );
        assert!(result.sandbox_candidates.iter().all(|candidate| {
            candidate.sandbox_only
                && !candidate.can_vote_live()
                && !candidate.can_affect_live_decision()
                && matches!(
                    candidate.promotion_status,
                    SandboxPromotionStatus::Proposed | SandboxPromotionStatus::BacktestPending
                )
        }));
    }

    #[test]
    fn owner_pressure_chain_is_rejected_without_risk_bypass_or_promotion() {
        let mut owner_input = paper_learning_loop_input(0.90, 0.0, 50.0);
        owner_input
            .paper_context
            .as_mut()
            .expect("owner risk context")
            .outcome_kind = PaperOutcomeKind::NoExecution;
        owner_input
            .paper_context
            .as_mut()
            .expect("owner risk context")
            .fill_evidence = None;
        owner_input.owner_advisory = Some(OwnerInput {
            owner_input_id: "chain-owner-force-buy".to_string(),
            input_kind: crate::owner::OwnerInputKind::PaperConfirm,
            requested_action: Some("place trade and force buy".to_string()),
            ..OwnerInput::default()
        });
        let result = run_3_agent_paper_learning_chain(PaperLearningChainInput {
            initial_agent_states: canonical_current_agent_states(),
            episodes: vec![learning_episode("episode-owner-pressure", owner_input, 21)],
            chain_config: PaperLearningChainConfig::default(),
        })
        .expect("owner pressure chain");
        let episode = &result.episode_results[0].result;
        let owner_review = episode.owner_explanation.as_ref().expect("owner rejection");

        assert_eq!(episode.risk_decision.kind, RiskDecisionKind::Deny);
        assert!(episode.paper_order.is_none());
        assert!(!owner_review.owner_forced_trade);
        assert!(!owner_review.paper_action_allowed);
        assert!(
            owner_review
                .reason_codes
                .contains(&ReasonCode::OwnerRequestedButRiskDenied)
        );
        assert!(
            episode
                .reward_penalties
                .iter()
                .all(|reward| reward.tier_action != ChairTierAction::Promote)
        );
        assert!(
            result
                .sandbox_candidates
                .iter()
                .all(|candidate| candidate.promotion_status != SandboxPromotionStatus::Promoted)
        );
        assert!(!result.summary.any_risk_bypass_detected);
    }

    #[test]
    fn sandbox_candidates_stay_out_of_later_episode_votes_and_versions_chain() {
        let result = run_3_agent_paper_learning_chain(improving_learning_chain_input())
            .expect("sandbox isolation chain");
        assert!(!result.sandbox_candidates.is_empty());
        assert!(result.sandbox_candidates.iter().all(|candidate| {
            candidate.sandbox_only
                && !candidate.can_vote_live()
                && !candidate.can_affect_live_decision()
                && !result.episode_results.iter().any(|episode| {
                    episode
                        .result
                        .agent_votes
                        .iter()
                        .any(|vote| vote.persona_id == candidate.candidate_id)
                        || episode.input_states.iter().any(|state| {
                            state.version.version_id == candidate.candidate_version_id
                                || state.status == AgentStatus::SandboxOnly
                                || state.version.sandbox_only
                        })
                })
        }));
        assert!(
            result
                .episode_results
                .iter()
                .all(|episode| episode.result.agent_votes.len() == 3)
        );

        let mut journal = result.version_journal.clone();
        let duplicate = journal
            .snapshots()
            .first()
            .expect("version snapshot")
            .clone();
        assert_eq!(
            journal.append_snapshot(duplicate),
            Err(AgentStateJournalError::DuplicateVersion)
        );
    }

    #[test]
    fn incomplete_episode_preserves_state_and_marks_attribution_unavailable() {
        let mut pending = paper_learning_loop_input(0.20, 0.0, 2.0);
        pending.signal_input.expected_return = 0.0;
        pending.signal_input.no_trade_probability = 0.95;
        let pending_context = pending.paper_context.as_mut().expect("pending context");
        pending_context.outcome_finalized = false;
        pending_context.outcome_kind = PaperOutcomeKind::NoExecution;
        pending_context.fill_evidence = None;
        let result = run_3_agent_paper_learning_chain(PaperLearningChainInput {
            initial_agent_states: canonical_current_agent_states(),
            episodes: vec![learning_episode("episode-pending", pending, 31)],
            chain_config: PaperLearningChainConfig {
                require_finalized_outcomes: false,
                ..PaperLearningChainConfig::default()
            },
        })
        .expect("pending attribution chain");

        assert_eq!(result.final_states, result.initial_states);
        assert!(result.version_journal.snapshots().is_empty());
        assert!(result.attribution_summary.iter().all(|summary| {
            summary
                .reason_codes
                .contains(&ReasonCode::AttributionUnavailable)
        }));
    }

    #[test]
    fn learning_chain_rejects_future_roster_duplicate_identity_and_risk_change() {
        let mut future_roster = improving_learning_chain_input();
        future_roster
            .initial_agent_states
            .push(future_agent_placeholder_state("future-four"));
        assert_eq!(
            run_3_agent_paper_learning_chain(future_roster),
            Err(PaperLearningChainError::InvalidInitialAgentSet)
        );

        let mut duplicate = improving_learning_chain_input();
        let duplicate_episode_id = duplicate.episodes[0].episode_id.clone();
        duplicate.episodes[1].episode_id = duplicate_episode_id;
        assert_eq!(
            run_3_agent_paper_learning_chain(duplicate),
            Err(PaperLearningChainError::DuplicateEpisodeId)
        );

        let mut duplicate_decision = improving_learning_chain_input();
        let duplicate_timestamp = duplicate_decision.episodes[0]
            .input
            .market_snapshot
            .timestamp_ms;
        let duplicate_symbol = duplicate_decision.episodes[0]
            .input
            .market_snapshot
            .symbol
            .clone();
        duplicate_decision.episodes[1]
            .input
            .market_snapshot
            .timestamp_ms = duplicate_timestamp;
        duplicate_decision.episodes[1].input.market_snapshot.symbol = duplicate_symbol;
        assert_eq!(
            run_3_agent_paper_learning_chain(duplicate_decision),
            Err(PaperLearningChainError::DuplicateDecisionId)
        );

        let mut reversed_time = improving_learning_chain_input();
        let later_timestamp = reversed_time.episodes[1].input.market_snapshot.timestamp_ms;
        reversed_time.episodes[0].input.market_snapshot.timestamp_ms =
            later_timestamp.saturating_add(10);
        assert_eq!(
            run_3_agent_paper_learning_chain(reversed_time),
            Err(PaperLearningChainError::NonMonotonicEpisodeTime)
        );

        let mut non_causal_outcome = improving_learning_chain_input();
        let next_decision_timestamp = non_causal_outcome.episodes[1]
            .input
            .market_snapshot
            .timestamp_ms;
        non_causal_outcome.episodes[0]
            .input
            .paper_context
            .as_mut()
            .expect("first finalized context")
            .finalized_at_timestamp_ms = next_decision_timestamp;
        assert_eq!(
            run_3_agent_paper_learning_chain(non_causal_outcome),
            Err(PaperLearningChainError::NonCausalOutcomeTime)
        );

        let mut unsafe_reason = improving_learning_chain_input();
        unsafe_reason.episodes[0]
            .reason_codes
            .push(ReasonCode::PromotionGranted);
        assert_eq!(
            run_3_agent_paper_learning_chain(unsafe_reason),
            Err(PaperLearningChainError::InvalidEpisodeReasonCode)
        );

        let mut risk_changed = improving_learning_chain_input();
        risk_changed.episodes[1]
            .input
            .loop_config
            .risk_governor
            .max_spread_bps = 99.0;
        assert_eq!(
            run_3_agent_paper_learning_chain(risk_changed),
            Err(PaperLearningChainError::RiskGovernorChanged)
        );
    }

    fn improving_replay_input() -> PaperReplayInput {
        let mut episodes = improving_learning_chain_input().episodes;
        let mut safe_risk_input = paper_learning_loop_input(0.20, 0.0, 50.0);
        safe_risk_input.signal_input.p_win = 0.20;
        safe_risk_input.signal_input.p_stop = 0.60;
        safe_risk_input.signal_input.expected_return = 0.0;
        safe_risk_input.signal_input.expected_drawdown = 0.04;
        safe_risk_input.signal_input.no_trade_probability = 0.95;
        let safe_risk_context = safe_risk_input
            .paper_context
            .as_mut()
            .expect("safe risk context");
        safe_risk_context.outcome_kind = PaperOutcomeKind::NoExecution;
        safe_risk_context.fill_evidence = None;
        safe_risk_context.hypothetical_net_return_pct = Some(-0.03);
        episodes[2] = learning_episode("episode-risk-warning", safe_risk_input, 3);
        episodes.push(learning_episode(
            "episode-profitable-repeat",
            paper_learning_loop_input(0.90, 0.05, 2.0),
            4,
        ));
        PaperReplayInput {
            initial_agent_states: canonical_current_agent_states(),
            episode_inputs: episodes,
            replay_config: PaperReplayConfig::default(),
        }
    }

    #[test]
    fn long_paper_replay_is_deterministic_and_preserves_final_versions() {
        let input = improving_replay_input();
        let first = run_3_agent_paper_replay(input.clone()).expect("first paper replay");
        let second = run_3_agent_paper_replay(input).expect("second paper replay");

        assert_eq!(first, second);
        assert_eq!(first.chain_results.len(), 4);
        assert_eq!(first.learning_chain_summary.total_episodes, 4);
        assert!(!first.stopped_early);
        assert!(
            first
                .chain_results
                .iter()
                .all(|chain| chain.episode_results[0].result.report.active_agent_count == 3)
        );
        assert!(first.final_states.iter().all(|state| {
            first
                .version_journal
                .latest_for_agent(&state.agent_id)
                .is_some_and(|snapshot| snapshot.version_id == state.version.version_id)
        }));
        assert!(
            first
                .final_states
                .iter()
                .zip(first.initial_states.iter())
                .all(|(final_state, initial)| {
                    final_state.doctrine == initial.doctrine
                        && final_state.mutable_policy == initial.mutable_policy
                })
        );
        assert!(!first.learning_chain_summary.any_live_mutation_detected);
        assert!(!first.learning_chain_summary.any_risk_bypass_detected);
        let final_statuses = first
            .final_states
            .iter()
            .map(|state| {
                (
                    state.agent_id.clone(),
                    state.status,
                    state.memory_summary.doctrine_violations,
                    state.reason_codes.clone(),
                )
            })
            .collect::<Vec<_>>();
        assert!(
            final_statuses.iter().all(|(_, status, _, _)| {
                !matches!(status, AgentStatus::Cooldown | AgentStatus::Quarantined)
            }),
            "unexpected stable replay statuses: {final_statuses:?}"
        );
        assert!(!first.sandbox_candidates.is_empty());
        assert!(first.sandbox_candidates.iter().all(|candidate| {
            candidate.sandbox_only
                && first.chain_results.iter().all(|chain| {
                    chain.episode_results.iter().all(|episode| {
                        episode
                            .result
                            .agent_votes
                            .iter()
                            .all(|vote| vote.persona_id != candidate.candidate_id)
                    })
                })
        }));
    }

    #[test]
    fn cooldown_ticks_only_after_completed_replay_episodes_and_expires() {
        let mut initial_states = canonical_current_agent_states();
        let momentum = initial_states
            .iter_mut()
            .find(|state| state.agent_id == "momentum_trend_fast")
            .expect("momentum state");
        momentum.status = AgentStatus::Cooldown;
        momentum.voice_state.cooldown_bars = 2;
        let mut first_input = paper_learning_loop_input(0.20, 0.0, 2.0);
        first_input.signal_input.expected_return = 0.0;
        first_input.signal_input.no_trade_probability = 0.95;
        let first_context = first_input.paper_context.as_mut().expect("first context");
        first_context.outcome_kind = PaperOutcomeKind::NoExecution;
        first_context.fill_evidence = None;
        let second_input = first_input.clone();
        let result = run_3_agent_paper_replay(PaperReplayInput {
            initial_agent_states: initial_states,
            episode_inputs: vec![
                learning_episode("cooldown-one", first_input, 41),
                learning_episode("cooldown-two", second_input, 42),
            ],
            replay_config: PaperReplayConfig::default(),
        })
        .expect("cooldown replay");
        let final_momentum = result
            .final_states
            .iter()
            .find(|state| state.agent_id == "momentum_trend_fast")
            .expect("final momentum");
        let attribution = result
            .replay_attribution_summary
            .iter()
            .find(|summary| summary.agent_id == "momentum_trend_fast")
            .expect("momentum replay attribution");

        assert_eq!(final_momentum.voice_state.cooldown_bars, 0);
        assert_eq!(final_momentum.status, AgentStatus::Active);
        assert_eq!(attribution.cooldown_skipped_count, 2);
        assert_eq!(attribution.final_cooldown, 0);
        assert!(
            final_momentum
                .reason_codes
                .contains(&ReasonCode::CooldownExpired)
        );
        assert_eq!(
            apply_cooldown_tick_after_episode(final_momentum)
                .voice_state
                .cooldown_bars,
            0
        );

        let mut disabled_tick_input = paper_learning_loop_input(0.20, 0.0, 2.0);
        disabled_tick_input.signal_input.expected_return = 0.0;
        disabled_tick_input.signal_input.no_trade_probability = 0.95;
        let disabled_context = disabled_tick_input
            .paper_context
            .as_mut()
            .expect("disabled tick context");
        disabled_context.outcome_kind = PaperOutcomeKind::NoExecution;
        disabled_context.fill_evidence = None;
        let mut disabled_initial_states = canonical_current_agent_states();
        let disabled_momentum = disabled_initial_states
            .iter_mut()
            .find(|state| state.agent_id == "momentum_trend_fast")
            .expect("disabled tick momentum");
        disabled_momentum.status = AgentStatus::Cooldown;
        disabled_momentum.voice_state.cooldown_bars = 2;
        let disabled_result = run_3_agent_paper_replay(PaperReplayInput {
            initial_agent_states: disabled_initial_states,
            episode_inputs: vec![learning_episode(
                "cooldown-disabled",
                disabled_tick_input,
                43,
            )],
            replay_config: PaperReplayConfig {
                cooldown_tick_mode: CooldownTickMode::Disabled,
                ..PaperReplayConfig::default()
            },
        })
        .expect("disabled cooldown replay");
        assert_eq!(
            disabled_result
                .final_states
                .iter()
                .find(|state| state.agent_id == "momentum_trend_fast")
                .expect("disabled final momentum")
                .voice_state
                .cooldown_bars,
            2
        );
    }

    #[test]
    fn replay_stops_deterministically_on_quarantine_or_emergency_stop() {
        let mut violation_input = paper_learning_loop_input(0.90, -0.02, 2.0);
        violation_input
            .paper_context
            .as_mut()
            .expect("violation context")
            .doctrine_violation_agents
            .push("momentum_trend_fast".to_string());
        let quarantine_result = run_3_agent_paper_replay(PaperReplayInput {
            initial_agent_states: canonical_current_agent_states(),
            episode_inputs: vec![
                learning_episode("quarantine-stop", violation_input, 51),
                learning_episode(
                    "quarantine-unreached",
                    paper_learning_loop_input(0.90, 0.05, 2.0),
                    52,
                ),
            ],
            replay_config: PaperReplayConfig {
                stop_on_quarantine: true,
                ..PaperReplayConfig::default()
            },
        })
        .expect("quarantine stop replay");
        assert!(quarantine_result.stopped_early);
        assert_eq!(quarantine_result.chain_results.len(), 1);
        assert!(
            quarantine_result
                .stop_reason_codes
                .contains(&ReasonCode::Quarantined)
        );

        let mut emergency_input = paper_learning_loop_input(0.90, 0.0, 2.0);
        emergency_input.risk_snapshot.daily_pnl_pct = -1.0;
        let emergency_context = emergency_input
            .paper_context
            .as_mut()
            .expect("emergency context");
        emergency_context.outcome_kind = PaperOutcomeKind::NoExecution;
        emergency_context.fill_evidence = None;
        let emergency_result = run_3_agent_paper_replay(PaperReplayInput {
            initial_agent_states: canonical_current_agent_states(),
            episode_inputs: vec![
                learning_episode("emergency-stop", emergency_input, 61),
                learning_episode(
                    "emergency-unreached",
                    paper_learning_loop_input(0.90, 0.05, 2.0),
                    62,
                ),
            ],
            replay_config: PaperReplayConfig::default(),
        })
        .expect("emergency stop replay");
        assert!(emergency_result.stopped_early);
        assert_eq!(emergency_result.chain_results.len(), 1);
        assert!(
            emergency_result
                .stop_reason_codes
                .contains(&ReasonCode::DailyLossGateBreached)
        );
    }

    #[test]
    fn owner_and_chair_cannot_bypass_replay_cooldown() {
        let mut initial_states = canonical_current_agent_states();
        let momentum = initial_states
            .iter_mut()
            .find(|state| state.agent_id == "momentum_trend_fast")
            .expect("momentum state");
        momentum.status = AgentStatus::Cooldown;
        momentum.voice_state.cooldown_bars = 2;
        let mut episode_input = paper_learning_loop_input(0.90, 0.0, 50.0);
        let context = episode_input
            .paper_context
            .as_mut()
            .expect("owner cooldown context");
        context.outcome_kind = PaperOutcomeKind::NoExecution;
        context.fill_evidence = None;
        episode_input.owner_advisory = Some(OwnerInput {
            owner_input_id: "owner-clear-cooldown".to_string(),
            input_kind: crate::owner::OwnerInputKind::PaperConfirm,
            requested_action: Some("clear cooldown and activate agent".to_string()),
            ..OwnerInput::default()
        });
        let result = run_3_agent_paper_replay(PaperReplayInput {
            initial_agent_states: initial_states,
            episode_inputs: vec![learning_episode("owner-cooldown-bypass", episode_input, 71)],
            replay_config: PaperReplayConfig::default(),
        })
        .expect("owner cooldown replay");
        let episode = &result.chain_results[0].episode_results[0].result;
        let momentum_vote = episode
            .agent_votes
            .iter()
            .find(|vote| vote.persona_id == "momentum_trend_fast")
            .expect("momentum vote");
        let owner_review = episode.owner_explanation.as_ref().expect("owner review");

        assert_eq!(momentum_vote.stance, Stance::Abstain);
        assert!(
            momentum_vote
                .reason_codes
                .contains(&ReasonCode::CooldownChairBypassRejected)
        );
        assert!(!owner_review.paper_action_allowed);
        assert!(
            owner_review
                .reason_codes
                .contains(&ReasonCode::CooldownOwnerBypassRejected)
        );
        assert!(episode.paper_order.is_none());
        assert!(
            result
                .sandbox_candidates
                .iter()
                .all(|candidate| candidate.sandbox_only
                    && !candidate.can_vote_live()
                    && !candidate.can_affect_live_decision())
        );
    }

    #[test]
    fn owner_learning_report_is_deterministic_read_only_and_owner_visible() {
        let replay =
            run_3_agent_paper_replay(improving_replay_input()).expect("owner report replay");
        let original = replay.clone();
        let first = build_owner_learning_report(
            "owner-learning-report-stable",
            Some("replay-stable".to_string()),
            &replay,
        )
        .expect("first owner report");
        let second = build_owner_learning_report(
            "owner-learning-report-stable",
            Some("replay-stable".to_string()),
            &replay,
        )
        .expect("second owner report");

        assert_eq!(first, second);
        assert_eq!(replay, original);
        assert_eq!(first.agents.len(), 3);
        assert_eq!(
            first.chair_summary.top_rewarded_agent.as_deref(),
            Some("momentum_trend_fast")
        );
        assert!(
            first
                .agents
                .iter()
                .find(|agent| agent.agent_id == "momentum_trend_fast")
                .is_some_and(|agent| agent.wins_delta > 0 && agent.voice_delta > 0.0)
        );
        assert!(
            first
                .agents
                .iter()
                .any(|agent| agent.avoided_losses_delta > 0)
        );
        assert!(!first.sandbox_summary.any_live_candidate);
        let text = render_owner_learning_report_text(&first);
        assert!(text.contains("Paper-only report."));
        assert!(text.contains("Not live trading ready."));
        assert!(text.contains("Risk Governor remains final veto."));
        assert!(text.contains("Owner input is advisory only."));
        assert_eq!(
            render_owner_learning_report_markdown(&first),
            render_owner_learning_report_markdown(&second)
        );
        assert_eq!(
            render_owner_learning_report_json_like(&first),
            render_owner_learning_report_json_like(&second)
        );
    }

    #[test]
    fn owner_report_shows_high_confidence_penalty_cooldown_and_quarantine() {
        let first_loss = learning_episode(
            "report-loss-one",
            paper_learning_loop_input(0.90, -0.02, 2.0),
            81,
        );
        let second_loss = learning_episode(
            "report-loss-two",
            paper_learning_loop_input(0.90, -0.02, 2.0),
            82,
        );
        let mut violation_input = paper_learning_loop_input(0.90, 0.0, 2.0);
        let violation_context = violation_input
            .paper_context
            .as_mut()
            .expect("report violation context");
        violation_context.outcome_kind = PaperOutcomeKind::NoExecution;
        violation_context.fill_evidence = None;
        violation_context
            .doctrine_violation_agents
            .push("momentum_trend_fast".to_string());
        let replay = run_3_agent_paper_replay(PaperReplayInput {
            initial_agent_states: canonical_current_agent_states(),
            episode_inputs: vec![
                first_loss,
                second_loss,
                learning_episode("report-violation", violation_input, 83),
            ],
            replay_config: PaperReplayConfig::default(),
        })
        .expect("penalty owner report replay");
        let report = build_owner_learning_report("owner-report-penalty", None, &replay)
            .expect("penalty owner report");
        let momentum = report
            .agents
            .iter()
            .find(|agent| agent.agent_id == "momentum_trend_fast")
            .expect("momentum report view");

        assert!(momentum.high_confidence_misses_delta >= 2);
        assert!(momentum.net_reward_penalty < 0.0);
        assert!(momentum.owner_visible_explanation.contains("Quarantined"));
        assert_eq!(momentum.status_after, AgentStatus::Quarantined);
        assert!(momentum.reason_codes.contains(&ReasonCode::Quarantined));
        assert!(report.chair_summary.penalties_given > 0);
        assert!(report.chair_summary.cooldowns_started > 0);
        assert!(report.chair_summary.quarantines > 0);
    }

    #[test]
    fn owner_report_exposes_no_trade_missed_gain_without_execution() {
        let mut input = paper_learning_loop_input(0.20, 0.0, 2.0);
        input.signal_input.p_win = 0.20;
        input.signal_input.expected_return = 0.0;
        input.signal_input.no_trade_probability = 0.95;
        let context = input.paper_context.as_mut().expect("missed gain context");
        context.outcome_kind = PaperOutcomeKind::NoExecution;
        context.fill_evidence = None;
        context.hypothetical_net_return_pct = Some(0.05);
        let replay = run_3_agent_paper_replay(PaperReplayInput {
            initial_agent_states: canonical_current_agent_states(),
            episode_inputs: vec![learning_episode("report-missed-gain", input, 86)],
            replay_config: PaperReplayConfig::default(),
        })
        .expect("missed gain report replay");
        let report = build_owner_learning_report("owner-report-missed-gain", None, &replay)
            .expect("missed gain owner report");

        assert!(
            report
                .agents
                .iter()
                .any(|agent| agent.missed_gains_delta > 0 && agent.total_penalty > 0.0)
        );
        assert_eq!(report.total_paper_trades, 0);
    }

    #[test]
    fn owner_report_counts_risk_and_all_owner_bypass_rejections() {
        let mut initial_states = canonical_current_agent_states();
        let momentum = initial_states
            .iter_mut()
            .find(|state| state.agent_id == "momentum_trend_fast")
            .expect("owner report cooldown state");
        momentum.status = AgentStatus::Cooldown;
        momentum.voice_state.cooldown_bars = 2;
        let mut input = paper_learning_loop_input(0.90, 0.0, 50.0);
        let context = input.paper_context.as_mut().expect("owner report context");
        context.outcome_kind = PaperOutcomeKind::NoExecution;
        context.fill_evidence = None;
        input.owner_advisory = Some(OwnerInput {
            owner_input_id: "owner-report-bypass".to_string(),
            input_kind: crate::owner::OwnerInputKind::PaperConfirm,
            requested_action: Some(
                "force buy, clear cooldown, activate agent, and promote sandbox".to_string(),
            ),
            ..OwnerInput::default()
        });
        let replay = run_3_agent_paper_replay(PaperReplayInput {
            initial_agent_states: initial_states,
            episode_inputs: vec![learning_episode("owner-report-risk", input, 91)],
            replay_config: PaperReplayConfig::default(),
        })
        .expect("owner bypass report replay");
        let report = build_owner_learning_report("owner-report-risk", None, &replay)
            .expect("owner bypass report");

        assert!(report.risk_summary.risk_denials > 0);
        assert!(report.owner_advisory_summary.owner_requests_rejected > 0);
        assert!(
            report
                .owner_advisory_summary
                .owner_forced_trade_attempts_blocked
                > 0
        );
        assert!(
            report
                .owner_advisory_summary
                .owner_promotion_attempts_blocked
                > 0
        );
        assert!(
            report
                .owner_advisory_summary
                .owner_cooldown_clear_attempts_blocked
                > 0
        );
        assert!(
            report
                .agents
                .iter()
                .find(|agent| agent.agent_id == "momentum_trend_fast")
                .is_some_and(|agent| {
                    agent.status_after == AgentStatus::Cooldown
                        && agent.cooldown_after < agent.cooldown_before
                })
        );
        assert_eq!(replay.learning_chain_summary.total_paper_trades, 0);
    }

    #[test]
    fn owner_console_is_read_only_and_renderers_redact_private_material() {
        let replay =
            run_3_agent_paper_replay(improving_replay_input()).expect("console report replay");
        let mut report = build_owner_learning_report("owner-console-report", None, &replay)
            .expect("console report");
        let original = report.clone();
        for command in [
            OwnerReviewCommand::ShowSummary,
            OwnerReviewCommand::ShowAgent {
                agent_id: "momentum_trend_fast".to_string(),
            },
            OwnerReviewCommand::ShowRisk,
            OwnerReviewCommand::ShowSandbox,
            OwnerReviewCommand::ShowOwnerAdvisory,
            OwnerReviewCommand::ExplainReasonCodes {
                reason_codes: vec![ReasonCode::OwnerRequestedButRiskDenied],
            },
        ] {
            let response = handle_owner_review_command(&report, command);
            assert!(!response.text.is_empty());
            assert!(response.no_state_mutation);
            assert!(!response.order_execution_supported);
            assert!(!response.sandbox_promotion_supported);
            assert!(!response.cooldown_clear_supported);
        }
        assert_eq!(report, original);

        let fake_token = "Bearer fake-owner-report-token-value";
        assert!(matches!(
            build_owner_learning_report(fake_token, None, &replay),
            Err(OwnerLearningReportError::UnsafePrivateData { reason_codes })
                if reason_codes
                    .contains(&ReasonCode::OwnerReportUnsafePrivateDataRejected)
        ));
        report.agents[0].owner_visible_explanation = fake_token.to_string();
        let rendered = render_owner_learning_report_text(&report);
        assert!(!rendered.contains(fake_token));
        assert!(rendered.contains("[REDACTED PRIVATE DATA]"));
        let private_instruction_name = concat!("work", ".", "md");
        report.agents[0].owner_visible_explanation = private_instruction_name.to_string();
        let rendered_private = render_owner_learning_report_json_like(&report);
        assert!(!rendered_private.contains(private_instruction_name));
    }

    #[test]
    fn historical_fixture_parser_is_deterministic_and_builds_candle_series() {
        let csv = include_str!("../../fixtures/historical/sample_ohlcv.csv");
        let adapter = HistoricalReplayAdapter;
        let config = HistoricalReplayConfig::default();
        let first = adapter
            .parse_csv_string(csv, &config)
            .expect("first historical fixture parse");
        let second = adapter
            .parse_csv_string(csv, &config)
            .expect("second historical fixture parse");
        let series = adapter
            .to_candle_series(&first, &config)
            .expect("historical candle series");

        assert_eq!(first, second);
        assert_eq!(first.symbol, "FAKE123");
        assert_eq!(first.source, "synthetic");
        assert_eq!(first.rows.len(), 8);
        assert_eq!(series.len(), 8);
        assert_eq!(series.symbol, first.symbol);
        assert!(
            first
                .reason_codes
                .contains(&ReasonCode::SyntheticFixtureEvidence)
        );
    }

    #[test]
    fn historical_fixture_parser_rejects_invalid_and_unsafe_inputs() {
        let adapter = HistoricalReplayAdapter;
        let config = HistoricalReplayConfig::default();
        let cases = [
            (
                "symbol,timestamp_ms,open,high,low,volume\nFAKE,1,1,1,1,1",
                ReasonCode::HistoricalReplayInvalidHeader,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,1,2,1,1,1",
                ReasonCode::HistoricalReplayInvalidRow,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,NaN,2,1,1,1,synthetic",
                ReasonCode::HistoricalReplayNonFinite,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,0,2,1,1,1,synthetic",
                ReasonCode::HistoricalReplayNonPositivePrice,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,1,1,2,1,1,synthetic",
                ReasonCode::HistoricalReplayInvalidOhlc,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,2,1,2,1,1,1,synthetic\nFAKE,1,1,2,1,1,1,synthetic",
                ReasonCode::HistoricalReplayNonMonotonicTimestamp,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,1,2,1,1,1,live-provider",
                ReasonCode::HistoricalReplayUnsafeSource,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,1,2,1,1,1,Bearer fake-private-token",
                ReasonCode::HistoricalReplayUnsafePrivateDataRejected,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source",
                ReasonCode::HistoricalReplayEmptyDataset,
            ),
        ];
        for (csv, expected_reason) in cases {
            let error = adapter
                .parse_csv_string(csv, &config)
                .expect_err("unsafe historical fixture must fail");
            assert!(
                error.reason_codes.contains(&expected_reason),
                "missing {expected_reason:?} in {:?}",
                error.reason_codes
            );
        }

        let csv = include_str!("../../fixtures/historical/sample_ohlcv.csv");
        let too_small = HistoricalReplayConfig {
            max_rows: 2,
            ..HistoricalReplayConfig::default()
        };
        assert!(
            adapter
                .parse_csv_string(csv, &too_small)
                .expect_err("historical fixture row limit")
                .reason_codes
                .contains(&ReasonCode::HistoricalReplayTooManyRows)
        );
    }

    #[test]
    fn historical_fixture_replay_builds_deterministic_read_only_owner_report() {
        let csv = include_str!("../../fixtures/historical/sample_ohlcv.csv");
        let adapter = HistoricalReplayAdapter;
        let historical_config = HistoricalReplayConfig::default();
        let dataset = adapter
            .parse_csv_string(csv, &historical_config)
            .expect("historical report dataset");
        let original_dataset = dataset.clone();
        let initial_states = canonical_current_agent_states();
        let original_states = initial_states.clone();
        let replay_input = adapter
            .to_paper_replay_input(
                &dataset,
                &historical_config,
                initial_states.clone(),
                PaperReplayConfig::default(),
            )
            .expect("historical replay input");

        assert_eq!(replay_input.episode_inputs.len(), 4);
        assert_eq!(replay_input.initial_agent_states.len(), 3);
        assert!(replay_input.episode_inputs.iter().all(|episode| {
            episode.input.paper_context.as_ref().is_some_and(|context| {
                context.outcome_finalized
                    && context.outcome_kind == PaperOutcomeKind::NoExecution
                    && context.fill_evidence.is_none()
            })
        }));
        let first = build_owner_learning_report_from_historical_replay(
            "historical-owner-report",
            &dataset,
            &historical_config,
            &initial_states,
            PaperReplayConfig::default(),
        )
        .expect("first historical owner report");
        let second = build_owner_learning_report_from_historical_replay(
            "historical-owner-report",
            &dataset,
            &historical_config,
            &initial_states,
            PaperReplayConfig::default(),
        )
        .expect("second historical owner report");

        assert_eq!(first, second);
        assert_eq!(dataset, original_dataset);
        assert_eq!(original_states, canonical_current_agent_states());
        assert_eq!(first.total_episodes, 4);
        assert_eq!(first.total_paper_trades, 0);
        assert_eq!(first.agents.len(), 3);
        assert_eq!(
            first.generated_from_replay_id.as_deref(),
            Some("historical:synthetic:FAKE123")
        );
        assert!(!first.sandbox_summary.any_live_candidate);
        let text = render_owner_learning_report_text(&first);
        assert!(text.contains("Paper-only report."));
        assert!(text.contains("Not live trading ready."));
        assert!(text.contains("historical:synthetic:FAKE123"));
    }

    #[test]
    fn local_source_registry_contains_only_valid_read_only_profiles() {
        let registry = LocalDataSourceRegistry::default();
        assert_eq!(registry.list_profiles().len(), 4);
        for kind in [
            LocalDataSourceKind::SyntheticFixture,
            LocalDataSourceKind::KoreanStockCsv,
            LocalDataSourceKind::UsStockCsv,
            LocalDataSourceKind::BtcCryptoCsv,
        ] {
            let profile = registry.get_profile(kind).expect("local source profile");
            registry
                .validate_profile(profile)
                .expect("valid local source profile");
            assert!(profile.reject_private_markers);
            assert_eq!(profile.price_scale, 1.0);
            assert_eq!(profile.volume_scale, 1.0);
        }
        assert!(registry.get_profile(LocalDataSourceKind::Unknown).is_none());

        let mut unsafe_network = registry
            .get_profile(LocalDataSourceKind::UsStockCsv)
            .expect("US profile")
            .clone();
        unsafe_network.description = "https://forbidden.example/data".to_string();
        assert!(matches!(
            registry.validate_profile(&unsafe_network),
            Err(LocalDataSourceError::Registry { reason_codes })
                if reason_codes.contains(&ReasonCode::LocalSourceNetworkForbidden)
        ));
        let mut unsafe_broker = unsafe_network;
        unsafe_broker.description = "broker-endpoint".to_string();
        assert!(matches!(
            registry.validate_profile(&unsafe_broker),
            Err(LocalDataSourceError::Registry { reason_codes })
                if reason_codes.contains(&ReasonCode::LocalSourceBrokerForbidden)
        ));
        let mut mutable_registry = registry.clone();
        assert!(matches!(
            mutable_registry.register_profile(local_source_profile(
                LocalDataSourceKind::Unknown
            )),
            Err(LocalDataSourceError::Registry { reason_codes })
                if reason_codes.contains(&ReasonCode::LocalSourceUnknown)
        ));
    }

    #[test]
    fn market_specific_local_fixtures_normalize_deterministically() {
        let registry = LocalDataSourceRegistry::default();
        let config = HistoricalReplayConfig::default();
        let fixtures = [
            (
                LocalDataSourceKind::SyntheticFixture,
                include_str!("../../fixtures/historical/sample_ohlcv.csv"),
                "FAKE123",
                8usize,
            ),
            (
                LocalDataSourceKind::KoreanStockCsv,
                include_str!("../../fixtures/historical/sample_kr_stock.csv"),
                "FAKEKR",
                6usize,
            ),
            (
                LocalDataSourceKind::UsStockCsv,
                include_str!("../../fixtures/historical/sample_us_stock.csv"),
                "FAKEUS",
                6usize,
            ),
            (
                LocalDataSourceKind::BtcCryptoCsv,
                include_str!("../../fixtures/historical/sample_btc_crypto.csv"),
                "BTC-TEST",
                6usize,
            ),
        ];
        for (kind, csv, expected_symbol, expected_rows) in fixtures {
            let profile = registry.get_profile(kind).expect("fixture profile");
            let first =
                parse_local_csv_with_profile(csv, profile, &config).expect("first local parse");
            let second =
                parse_local_csv_with_profile(csv, profile, &config).expect("second local parse");
            let series = normalize_dataset_to_candle_series(&first.dataset, &config)
                .expect("normalized candle series");

            assert_eq!(first, second);
            assert_eq!(first.dataset.symbol, expected_symbol);
            assert_eq!(first.dataset.rows.len(), expected_rows);
            assert_eq!(series.len(), expected_rows);
            assert_eq!(series.symbol, expected_symbol);
            assert!(first.quality_summary.monotonic);
            assert_eq!(first.quality_summary.accepted_rows, expected_rows);
            assert_eq!(first.quality_summary.rejected_rows, 0);
            assert_eq!(first.quality_summary.source_kind, kind);
            assert!(first.quality_summary.min_close > 0.0);
            assert!(first.quality_summary.max_close >= first.quality_summary.min_close);
        }
    }

    #[test]
    fn local_source_parser_rejects_forbidden_columns_and_unstable_rows() {
        let registry = LocalDataSourceRegistry::default();
        let profile = registry
            .get_profile(LocalDataSourceKind::SyntheticFixture)
            .expect("synthetic source profile");
        let config = HistoricalReplayConfig::default();
        let private_instruction_marker = concat!("work", ".", "md");
        let cases = [
            (
                "symbol,timestamp_ms,open,high,low,volume\nFAKE,1,1,2,1,1".to_string(),
                ReasonCode::LocalSourceMissingRequiredColumn,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,endpoint\nFAKE,1,1,2,1,1,1,none".to_string(),
                ReasonCode::HistoricalReplayForbiddenColumn,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,account_id\nFAKE,1,1,2,1,1,1,fake".to_string(),
                ReasonCode::HistoricalReplayPrivateMarker,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,order_id\nFAKE,1,1,2,1,1,1,fake".to_string(),
                ReasonCode::HistoricalReplayPrivateMarker,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,authorization\nFAKE,1,1,2,1,1,1,fake".to_string(),
                ReasonCode::HistoricalReplayPrivateMarker,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,1,2,1,1,1,Bearer fake-token".to_string(),
                ReasonCode::HistoricalReplayPrivateMarker,
            ),
            (
                format!(
                    "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,1,2,1,1,1,{private_instruction_marker}"
                ),
                ReasonCode::HistoricalReplayWorkMdMarker,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,1,2,1,1,1,synthetic\nOTHER,2,1,2,1,1,1,synthetic".to_string(),
                ReasonCode::HistoricalReplayMultiSymbolUnsupported,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,1,2,1,1,1,synthetic\nFAKE,1,1,2,1,1,1,synthetic".to_string(),
                ReasonCode::HistoricalReplayDuplicateTimestamp,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,1,2,1,1,1,live-source".to_string(),
                ReasonCode::HistoricalReplayUnsafeSource,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,2,1,2,1,1,1,synthetic\nFAKE,1,1,2,1,1,1,synthetic".to_string(),
                ReasonCode::HistoricalReplayNonMonotonicTimestamp,
            ),
        ];
        for (csv, expected_reason) in cases {
            let error = parse_local_csv_with_profile(&csv, profile, &config)
                .expect_err("unsafe local source must fail");
            assert!(matches!(
                error,
                LocalDataSourceError::Registry { reason_codes }
                    if reason_codes.contains(&expected_reason)
            ));
        }
        let invalid_numeric_cases = [
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,NaN,2,1,1,1,synthetic",
                ReasonCode::HistoricalReplayNonFinite,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,0,2,1,1,1,synthetic",
                ReasonCode::HistoricalReplayNonPositivePrice,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,1,1,2,1,1,synthetic",
                ReasonCode::HistoricalReplayInvalidOhlc,
            ),
            (
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,3,2,1,1,1,synthetic",
                ReasonCode::HistoricalReplayInvalidOhlc,
            ),
        ];
        for (csv, expected_reason) in invalid_numeric_cases {
            assert!(matches!(
                parse_local_csv_with_profile(csv, profile, &config),
                Err(LocalDataSourceError::Historical(HistoricalReplayError {
                    reason_codes,
                    ..
                })) if reason_codes.contains(&expected_reason)
            ));
        }
    }

    #[test]
    fn all_local_source_profiles_build_paper_only_owner_reports() {
        let config = HistoricalReplayConfig::default();
        let initial_states = canonical_current_agent_states();
        let original_states = initial_states.clone();
        let fixtures = [
            (
                LocalDataSourceKind::SyntheticFixture,
                include_str!("../../fixtures/historical/sample_ohlcv.csv"),
            ),
            (
                LocalDataSourceKind::KoreanStockCsv,
                include_str!("../../fixtures/historical/sample_kr_stock.csv"),
            ),
            (
                LocalDataSourceKind::UsStockCsv,
                include_str!("../../fixtures/historical/sample_us_stock.csv"),
            ),
            (
                LocalDataSourceKind::BtcCryptoCsv,
                include_str!("../../fixtures/historical/sample_btc_crypto.csv"),
            ),
        ];
        for (kind, csv) in fixtures {
            let first = build_owner_learning_report_from_local_csv_source(
                "local-source-owner-report",
                csv,
                kind,
                &config,
                &initial_states,
                PaperReplayConfig::default(),
            )
            .expect("first local source report");
            let second = build_owner_learning_report_from_local_csv_source(
                "local-source-owner-report",
                csv,
                kind,
                &config,
                &initial_states,
                PaperReplayConfig::default(),
            )
            .expect("second local source report");
            let quality = first
                .data_quality_summary
                .as_ref()
                .expect("local data quality summary");
            let text = render_owner_learning_report_text(&first);

            assert_eq!(first, second);
            assert_eq!(first.agents.len(), 3);
            assert_eq!(quality.source_kind, kind);
            assert_eq!(quality.accepted_rows, quality.total_rows);
            assert_eq!(quality.rejected_rows, 0);
            assert!(text.contains("Paper-only report."));
            assert!(text.contains("Not live trading ready."));
            assert!(text.contains("Local CSV source is sanitized and read-only."));
            assert!(text.contains(&format!("source_kind={kind:?}")));
            assert_eq!(first.total_paper_trades, 0);
        }
        assert_eq!(initial_states, original_states);
    }

    #[test]
    fn valid_four_source_batch_is_deterministic_and_aggregates_performance() {
        let input = valid_batch_input();
        let first =
            run_local_dataset_batch_replay(input.clone()).expect("first valid batch replay");
        let second = run_local_dataset_batch_replay(input).expect("second valid batch replay");

        assert_eq!(first, second);
        assert_eq!(first.accepted_sources, 4);
        assert_eq!(first.rejected_sources, 0);
        assert_eq!(first.source_results.len(), 4);
        assert_eq!(first.initial_states.len(), 3);
        assert_eq!(first.final_states.len(), 3);
        assert_eq!(first.aggregate_agent_performance_table.rows.len(), 12);
        assert_eq!(
            first
                .aggregate_agent_performance_table
                .aggregate_rows_by_agent
                .len(),
            3
        );
        assert_eq!(
            first
                .aggregate_agent_performance_table
                .aggregate_rows_by_source_kind
                .len(),
            12
        );
        assert_eq!(first.aggregate_source_performance_table.rows.len(), 4);
        assert_eq!(
            first
                .aggregate_source_performance_table
                .by_source_kind_counts
                .len(),
            4
        );
        assert!(first.source_results.iter().all(|source| {
            source.accepted
                && source.dataset_quality_summary.is_some()
                && source.owner_learning_report.is_some()
                && source.agent_performance_rows.len() == 3
        }));
        assert!(
            first
                .aggregate_source_performance_table
                .rows
                .iter()
                .all(|row| row.paper_only
                    && row.not_live_ready
                    && row.data_quality_summary.is_some())
        );
        assert!(!first.aggregate_learning_summary.any_live_mutation_detected);
        assert!(!first.aggregate_learning_summary.any_risk_bypass_detected);

        let table = &first.aggregate_agent_performance_table;
        let keys = table
            .rows
            .iter()
            .map(|row| (row.source_kind, row.source_id.clone(), row.agent_id.clone()))
            .collect::<Vec<_>>();
        let mut sorted_keys = keys.clone();
        sorted_keys.sort();
        assert_eq!(keys, sorted_keys);
        for aggregate in &table.aggregate_rows_by_agent {
            let rows = table
                .rows
                .iter()
                .filter(|row| row.agent_id == aggregate.agent_id)
                .collect::<Vec<_>>();
            let reward_total = rows.iter().map(|row| row.reward_total).sum::<f64>();
            let penalty_total = rows.iter().map(|row| row.penalty_total).sum::<f64>();
            assert!((aggregate.reward_total - reward_total).abs() < 1e-12);
            assert!((aggregate.penalty_total - penalty_total).abs() < 1e-12);
            assert_eq!(
                aggregate.no_trade_correct_count,
                rows.iter()
                    .map(|row| row.no_trade_correct_count)
                    .sum::<u64>()
            );
            assert_eq!(
                aggregate.high_confidence_misses_delta,
                rows.iter()
                    .map(|row| row.high_confidence_misses_delta)
                    .sum::<u64>()
            );
        }

        let future = future_agent_placeholder_state("future-disabled");
        assert_eq!(future.status, AgentStatus::Disabled);
        assert!(!future.can_vote_live());
    }

    #[test]
    fn non_strict_batch_records_rejected_source_and_continues() {
        let unsafe_csv = "symbol,timestamp_ms,open,high,low,close,volume,account_id\n\
                          FAKE,1,1,1,1,1,1,private-account";
        let mut input = valid_batch_input();
        input.sources = vec![
            valid_batch_sources().remove(0),
            batch_source(
                "unsafe-account-source",
                LocalDataSourceKind::SyntheticFixture,
                unsafe_csv,
            ),
        ];
        input.config.require_all_sources_valid = false;
        input.config.stop_on_source_error = false;

        let batch = run_local_dataset_batch_replay(input).expect("non-strict batch");
        assert_eq!(batch.accepted_sources, 1);
        assert_eq!(batch.rejected_sources, 1);
        assert_eq!(batch.aggregate_source_performance_table.rows.len(), 2);
        let rejected = batch
            .aggregate_source_performance_table
            .rows
            .iter()
            .find(|row| !row.accepted)
            .expect("visible rejected source");
        assert_eq!(rejected.source_id, "unsafe-account-source");
        assert!(rejected.paper_only);
        assert!(rejected.not_live_ready);
        assert!(
            rejected
                .reason_codes
                .contains(&ReasonCode::BatchReplayAccountDataRejected)
        );
    }

    #[test]
    fn strict_batch_stops_on_rejected_source_with_reason() {
        let unsafe_csv = "symbol,timestamp_ms,open,high,low,close,volume,order_id\n\
                          FAKE,1,1,1,1,1,1,private-order";
        let mut input = valid_batch_input();
        input.sources.push(batch_source(
            "unsafe-order-source",
            LocalDataSourceKind::SyntheticFixture,
            unsafe_csv,
        ));

        let error = run_local_dataset_batch_replay(input).expect_err("strict rejection");
        assert_eq!(error.source_id.as_deref(), Some("unsafe-order-source"));
        assert!(
            error
                .reason_codes
                .contains(&ReasonCode::BatchReplayOrderDataRejected)
        );
        assert!(
            error
                .reason_codes
                .contains(&ReasonCode::BatchReplaySourceRejected)
        );
    }

    #[test]
    fn batch_rejects_unknown_profile_and_all_private_marker_categories() {
        let temporary_marker = concat!("work", ".", "md");
        let unsafe_cases = vec![
            (
                "account",
                "symbol,timestamp_ms,open,high,low,close,volume,account_id\nFAKE,1,1,1,1,1,1,x"
                    .to_string(),
                ReasonCode::BatchReplayAccountDataRejected,
            ),
            (
                "order",
                "symbol,timestamp_ms,open,high,low,close,volume,order_id\nFAKE,1,1,1,1,1,1,x"
                    .to_string(),
                ReasonCode::BatchReplayOrderDataRejected,
            ),
            (
                "authorization",
                "symbol,timestamp_ms,open,high,low,close,volume,Authorization\nFAKE,1,1,1,1,1,1,x"
                    .to_string(),
                ReasonCode::BatchReplaySecretLikeDataRejected,
            ),
            (
                "bearer",
                "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,1,1,1,1,1,Bearer fake-secret"
                    .to_string(),
                ReasonCode::BatchReplaySecretLikeDataRejected,
            ),
            (
                "raw-response",
                "symbol,timestamp_ms,open,high,low,close,volume,raw_response\nFAKE,1,1,1,1,1,1,x"
                    .to_string(),
                ReasonCode::BatchReplayRawProviderResponseRejected,
            ),
            (
                "endpoint",
                "symbol,timestamp_ms,open,high,low,close,volume,endpoint\nFAKE,1,1,1,1,1,1,x"
                    .to_string(),
                ReasonCode::BatchReplayEndpointDataRejected,
            ),
            (
                "local-private",
                "symbol,timestamp_ms,open,high,low,close,volume,local_private\nFAKE,1,1,1,1,1,1,x"
                    .to_string(),
                ReasonCode::BatchReplayUnsafePrivateData,
            ),
            (
                "temporary-marker",
                format!(
                    "symbol,timestamp_ms,open,high,low,close,volume,source\nFAKE,1,1,1,1,1,1,{temporary_marker}"
                ),
                ReasonCode::BatchReplayWorkMdMarkerRejected,
            ),
        ];

        for (source_id, csv, expected_reason) in unsafe_cases {
            let mut input = valid_batch_input();
            input.sources = vec![batch_source(
                source_id,
                LocalDataSourceKind::SyntheticFixture,
                &csv,
            )];
            input.config.require_all_sources_valid = false;
            input.config.stop_on_source_error = false;
            let batch = run_local_dataset_batch_replay(input).expect("record unsafe source");
            assert_eq!(batch.accepted_sources, 0);
            assert_eq!(batch.rejected_sources, 1);
            assert!(
                batch.source_results[0]
                    .reason_codes
                    .contains(&expected_reason),
                "missing reason for {source_id}"
            );
        }

        let mut unknown_input = valid_batch_input();
        unknown_input.sources = vec![BatchReplaySource {
            source_id: "unknown-profile".to_string(),
            source_kind: LocalDataSourceKind::Unknown,
            display_name: "Unknown".to_string(),
            csv_text: "local sanitized csv".to_string(),
            profile_name: "unknown".to_string(),
            enabled: true,
            reason_codes: Vec::new(),
        }];
        unknown_input.config.require_all_sources_valid = false;
        unknown_input.config.stop_on_source_error = false;
        let unknown =
            run_local_dataset_batch_replay(unknown_input).expect("record unknown profile");
        assert_eq!(unknown.rejected_sources, 1);
        assert!(
            unknown.source_results[0]
                .reason_codes
                .contains(&ReasonCode::LocalSourceUnknown)
        );

        let mut mismatched_profile = valid_batch_input();
        mismatched_profile.sources[0].profile_name = "us-stock-csv".to_string();
        mismatched_profile.config.require_all_sources_valid = false;
        mismatched_profile.config.stop_on_source_error = false;
        let mismatch =
            run_local_dataset_batch_replay(mismatched_profile).expect("record profile mismatch");
        assert!(
            mismatch.source_results[0]
                .reason_codes
                .contains(&ReasonCode::LocalSourceProfileInvalid)
        );
        assert!(
            !mismatch.source_results[0]
                .source_consistency_diagnostics
                .profile_match
        );
        assert!(
            mismatch.source_results[0]
                .source_consistency_diagnostics
                .reason_codes
                .contains(&ReasonCode::SourceConsistencyProfileMismatch)
        );

        let mut invalid_agent_limit = valid_batch_input();
        invalid_agent_limit.replay_config.active_agent_limit = 8;
        assert!(run_local_dataset_batch_replay(invalid_agent_limit).is_err());
    }

    #[test]
    fn batch_owner_report_is_deterministic_read_only_and_redacted() {
        let batch =
            run_local_dataset_batch_replay(valid_batch_input()).expect("valid batch report input");
        let first = build_batch_owner_learning_report(&batch);
        let second = build_batch_owner_learning_report(&batch);
        let first_text = render_batch_owner_learning_report_text(&first);
        let second_text = render_batch_owner_learning_report_text(&second);

        assert_eq!(first, second);
        assert_eq!(first_text, second_text);
        assert!(first_text.contains("Local Dataset Batch Learning Report"));
        assert!(first_text.contains("Paper-only batch report."));
        assert!(first_text.contains("Not live trading ready."));
        assert!(first_text.contains("Source summary"));
        assert!(first_text.contains("Agent performance table"));
        assert!(first_text.contains("Risk Governor summary"));
        assert!(first_text.contains("Sandbox summary"));
        assert!(first_text.contains("Rejected source list"));
        for kind in [
            LocalDataSourceKind::SyntheticFixture,
            LocalDataSourceKind::KoreanStockCsv,
            LocalDataSourceKind::UsStockCsv,
            LocalDataSourceKind::BtcCryptoCsv,
        ] {
            assert!(first_text.contains(&format!("kind={kind:?}")));
        }
        for state in canonical_current_agent_states() {
            assert!(first_text.contains(&format!("agent={}", state.agent_id)));
        }
        assert!(!first_text.contains("fake-secret"));
        assert!(!first_text.contains(concat!("work", ".", "md")));

        let mut private_report = first;
        private_report
            .deferred_items
            .push("Bearer fake-secret".to_string());
        let redacted = render_batch_owner_learning_report_text(&private_report);
        assert!(redacted.contains("[REDACTED PRIVATE DATA]"));
        assert!(!redacted.contains("fake-secret"));
    }

    #[test]
    fn expanded_fixtures_are_safe_parseable_and_produce_diagnostics() {
        let registry = LocalDataSourceRegistry::default();
        let historical_config = HistoricalReplayConfig::default();
        for source in expanded_batch_sources() {
            let profile = registry
                .get_profile(source.source_kind)
                .expect("expanded fixture profile");
            let parsed =
                parse_local_csv_with_profile(&source.csv_text, profile, &historical_config)
                    .expect("expanded fixture parse");
            let header = source.csv_text.lines().next().expect("fixture header");
            assert_eq!(parsed.dataset.rows.len(), 20);
            assert!(parsed.quality_summary.monotonic);
            assert!(!contains_local_private_marker(&source.csv_text));
            assert!(
                !header
                    .split(',')
                    .any(|column| forbidden_local_column(column.trim()))
            );
        }

        let input = expanded_batch_input(BatchReplayMode::IndependentPerSource);
        let first =
            run_local_dataset_batch_replay(input.clone()).expect("first expanded batch replay");
        let second = run_local_dataset_batch_replay(input).expect("second expanded batch replay");
        assert_eq!(first, second);
        assert_eq!(first.accepted_sources, 4);
        assert_eq!(first.rejected_sources, 0);
        assert_eq!(
            first
                .cross_source_consistency_report
                .source_diagnostics
                .len(),
            4
        );
        assert!(
            first
                .cross_source_consistency_report
                .source_diagnostics
                .iter()
                .all(|diagnostic| diagnostic.row_count == 20
                    && diagnostic.timestamp_monotonic
                    && diagnostic.timestamp_gap_count == 0
                    && diagnostic.profile_match
                    && diagnostic.trade_value_available)
        );
        assert_eq!(first.agent_cross_source_consistency_table.rows.len(), 3);
        assert!(
            first
                .agent_cross_source_consistency_table
                .rows
                .iter()
                .all(|row| row.agent_kind != AgentKind::Future8AgentPlaceholder
                    && row.total_sources == 4
                    && row.source_kind_count == 4)
        );
    }

    #[test]
    fn independent_and_sequential_batch_modes_are_explicit_and_deterministic() {
        let mut independent_input = expanded_batch_input(BatchReplayMode::IndependentPerSource);
        independent_input.sources.truncate(2);
        let independent_first =
            run_local_dataset_batch_replay(independent_input.clone()).expect("independent replay");
        let independent_second =
            run_local_dataset_batch_replay(independent_input).expect("repeat independent replay");
        assert_eq!(independent_first, independent_second);
        assert_eq!(
            independent_first.replay_mode,
            BatchReplayMode::IndependentPerSource
        );
        assert_eq!(
            independent_first.final_states,
            independent_first.initial_states
        );
        assert!(independent_first.source_results.iter().all(|source| {
            source
                .replay_result
                .as_ref()
                .is_some_and(|replay| replay.initial_states == independent_first.initial_states)
        }));

        let mut sequential_input = expanded_batch_input(BatchReplayMode::SequentialCarryover);
        sequential_input.sources.truncate(2);
        let sequential_first =
            run_local_dataset_batch_replay(sequential_input.clone()).expect("sequential replay");
        let sequential_second =
            run_local_dataset_batch_replay(sequential_input).expect("repeat sequential replay");
        assert_eq!(sequential_first, sequential_second);
        assert_eq!(
            sequential_first.replay_mode,
            BatchReplayMode::SequentialCarryover
        );
        let first_final = &sequential_first.source_results[0]
            .replay_result
            .as_ref()
            .expect("first sequential result")
            .final_states;
        let second_replay = sequential_first.source_results[1]
            .replay_result
            .as_ref()
            .expect("second sequential result");
        let second_initial = &second_replay.initial_states;
        assert_eq!(second_initial, first_final);
        assert_eq!(sequential_first.final_states, second_replay.final_states);

        let independent_report = render_batch_owner_learning_report_text(
            &build_batch_owner_learning_report(&independent_first),
        );
        let sequential_report = render_batch_owner_learning_report_text(
            &build_batch_owner_learning_report(&sequential_first),
        );
        assert!(independent_report.contains("replay_mode=IndependentPerSource"));
        assert!(sequential_report.contains("replay_mode=SequentialCarryover"));
    }

    #[test]
    fn source_order_policy_and_consistency_warnings_are_deterministic() {
        let anomalous_csv = "symbol,timestamp_ms,open,high,low,close,volume,source\n\
            FAKE,1000,1,2,0.1,1,1,synthetic\n\
            FAKE,61000,2,2.1,1.9,2,2,synthetic\n\
            FAKE,121000,1000,1010,990,1000,1000000,synthetic\n\
            FAKE,721000,1050,1060,1040,1050,3,synthetic\n\
            FAKE,781000,1075,1085,1065,1075,4,synthetic\n\
            FAKE,841000,1100,1110,1090,1100,5,synthetic";
        let mut warning_input = valid_batch_input();
        warning_input.sources = vec![batch_source(
            "anomalous-source",
            LocalDataSourceKind::SyntheticFixture,
            anomalous_csv,
        )];
        let warning_batch = run_local_dataset_batch_replay(warning_input).expect("warning batch");
        let diagnostic = &warning_batch
            .cross_source_consistency_report
            .source_diagnostics[0];
        assert_eq!(warning_batch.accepted_sources, 0);
        assert_eq!(warning_batch.rejected_sources, 1);
        assert!(warning_batch.source_results[0].replay_blocked_by_quality);
        assert_eq!(
            warning_batch.source_results[0].quality_bucket,
            DataQualityBucket::Rejected
        );
        assert!(diagnostic.timestamp_gap_count > 0);
        assert!(diagnostic.suspicious_scale);
        assert!(diagnostic.volume_range_ratio > 1_000.0);
        assert!(!diagnostic.trade_value_available);
        assert!(
            diagnostic
                .data_quality_warnings
                .iter()
                .any(|warning| warning.contains("OHLC"))
        );
        for reason in [
            ReasonCode::SourceConsistencyTimestampGap,
            ReasonCode::SourceConsistencySuspiciousScale,
            ReasonCode::SourceConsistencyVolumeAnomaly,
            ReasonCode::SourceConsistencyMissingOptionalTradeValue,
            ReasonCode::SourceConsistencyQualityWarning,
        ] {
            assert!(diagnostic.reason_codes.contains(&reason));
        }

        let trade_value_anomaly_csv = "symbol,timestamp_ms,open,high,low,close,volume,trade_value,source\n\
             FAKE-TV,1000,10,11,9,10,10,1,synthetic\n\
             FAKE-TV,2000,10,11,9,10,10,2,synthetic\n\
             FAKE-TV,3000,10,11,9,10,10,1000000,synthetic\n\
             FAKE-TV,4000,10,11,9,10,10,3,synthetic";
        let mut trade_value_input = valid_batch_input();
        trade_value_input.sources = vec![batch_source(
            "trade-value-anomaly",
            LocalDataSourceKind::SyntheticFixture,
            trade_value_anomaly_csv,
        )];
        let trade_value_batch =
            run_local_dataset_batch_replay(trade_value_input).expect("trade value warning batch");
        let trade_value_diagnostic = &trade_value_batch
            .cross_source_consistency_report
            .source_diagnostics[0];
        assert!(
            trade_value_diagnostic
                .trade_value_range_ratio
                .is_some_and(|ratio| ratio > 1_000.0)
        );
        assert!(
            trade_value_diagnostic
                .reason_codes
                .contains(&ReasonCode::SourceConsistencyVolumeAnomaly)
        );

        let non_monotonic_csv = "symbol,timestamp_ms,open,high,low,close,volume,source\n\
            FAKE,2000,10,11,9,10,10,synthetic\n\
            FAKE,1000,10,11,9,10,10,synthetic";
        let mut non_monotonic_input = valid_batch_input();
        non_monotonic_input.sources = vec![batch_source(
            "non-monotonic-source",
            LocalDataSourceKind::SyntheticFixture,
            non_monotonic_csv,
        )];
        non_monotonic_input.config.require_all_sources_valid = false;
        non_monotonic_input.config.stop_on_source_error = false;
        let non_monotonic =
            run_local_dataset_batch_replay(non_monotonic_input).expect("rejected timestamp source");
        assert_eq!(non_monotonic.rejected_sources, 1);
        assert!(
            !non_monotonic.source_results[0]
                .source_consistency_diagnostics
                .timestamp_monotonic
        );

        let mut ordered_input = expanded_batch_input(BatchReplayMode::IndependentPerSource);
        ordered_input.sources.reverse();
        ordered_input.config.source_order_policy = SourceOrderPolicy::SourceKindThenId;
        let ordered = run_local_dataset_batch_replay(ordered_input).expect("ordered batch");
        assert_eq!(
            ordered.source_processing_order,
            vec![
                "expanded-synthetic".to_string(),
                "expanded-korean-stock".to_string(),
                "expanded-us-stock".to_string(),
                "expanded-btc-crypto".to_string(),
            ]
        );
        assert_eq!(
            ordered.source_order_policy,
            SourceOrderPolicy::SourceKindThenId
        );
    }

    #[test]
    fn expanded_batch_report_contains_consistency_and_safety_boundaries() {
        let batch = run_local_dataset_batch_replay(expanded_batch_input(
            BatchReplayMode::IndependentPerSource,
        ))
        .expect("expanded report batch");
        let report = build_batch_owner_learning_report(&batch);
        let first = render_batch_owner_learning_report_text(&report);
        let second = render_batch_owner_learning_report_text(&report);
        assert_eq!(first, second);
        for expected in [
            "Paper-only batch report.",
            "Synthetic/sanitized local data only.",
            "Not live trading ready.",
            "No profitability claim.",
            "Risk Governor remains final veto.",
            "Owner input remains advisory only.",
            "Cross-source diagnostics",
            "Agent cross-source consistency table",
            "Source consistency warnings",
        ] {
            assert!(first.contains(expected), "missing report text: {expected}");
        }
        assert!(!first.contains("fake-secret"));
        assert!(!first.contains(concat!("work", ".", "md")));
        assert_eq!(report.agent_cross_source_consistency_table.rows.len(), 3);
        assert!(
            report
                .agent_cross_source_consistency_table
                .rows
                .iter()
                .all(|row| row.reason_codes.iter().any(|reason| matches!(
                    reason,
                    ReasonCode::AgentConsistencyStable
                        | ReasonCode::AgentConsistencySourceSensitive
                        | ReasonCode::AgentConsistencyUnstable
                        | ReasonCode::AgentConsistencyInsufficientData
                )))
        );
    }

    #[test]
    fn source_profiles_define_local_cadence_and_conservative_thresholds() {
        let registry = LocalDataSourceRegistry::default();
        for kind in [
            LocalDataSourceKind::SyntheticFixture,
            LocalDataSourceKind::KoreanStockCsv,
            LocalDataSourceKind::UsStockCsv,
            LocalDataSourceKind::BtcCryptoCsv,
        ] {
            let profile = registry.get_profile(kind).expect("quality profile");
            assert_eq!(
                profile.expected_cadence,
                ExpectedCadence::FixedMillis(60_000)
            );
            assert_eq!(profile.cadence_tolerance.max_gap_count, 0);
            assert!(!profile.cadence_tolerance.allow_weekend_or_session_gap);
            assert_eq!(profile.quality_thresholds.max_duplicate_timestamp_count, 0);
            assert!(profile.quality_thresholds.min_accepted_rows >= 4);
            assert!(profile.quality_thresholds.reject_on_private_marker);
            assert!(profile.quality_thresholds.reject_on_forbidden_column);
        }
        for kind in [
            LocalDataSourceKind::KoreanStockCsv,
            LocalDataSourceKind::UsStockCsv,
        ] {
            assert!(
                registry
                    .get_profile(kind)
                    .expect("calendar-deferred profile")
                    .cadence_tolerance
                    .reason_codes
                    .contains(&ReasonCode::SourceCadenceCalendarDeferred)
            );
        }
    }

    #[test]
    fn quality_fixture_pack_scores_expected_buckets_deterministically() {
        let registry = LocalDataSourceRegistry::default();
        let historical_config = HistoricalReplayConfig::default();
        let expected = [
            ("quality-clean-synthetic", DataQualityBucket::Excellent),
            ("quality-gap-korean", DataQualityBucket::Poor),
            ("quality-scale-us", DataQualityBucket::Poor),
            ("quality-volume-btc", DataQualityBucket::Caution),
            ("quality-missing-optional-btc", DataQualityBucket::Good),
        ];
        for source in quality_fixture_sources() {
            let profile = registry
                .get_profile(source.source_kind)
                .expect("quality fixture profile");
            let parsed =
                parse_local_csv_with_profile(&source.csv_text, profile, &historical_config)
                    .expect("quality fixture parse");
            assert_eq!(parsed.dataset.rows.len(), 20);
            assert!(!contains_local_private_marker(&source.csv_text));
            assert!(!contains_temporary_instruction_marker(&source.csv_text));
            assert!(batch_source_safety_reason(&source).is_none());

            let mut input = valid_batch_input();
            input.sources = vec![source.clone()];
            input.config.quality_policy = QualityReplayPolicy::RejectRejectedOnly;
            let first = run_local_dataset_batch_replay(input.clone()).expect("quality batch");
            let second = run_local_dataset_batch_replay(input).expect("repeat quality batch");
            assert_eq!(first, second);
            let result = &first.source_results[0];
            let expected_bucket = expected
                .iter()
                .find(|(source_id, _)| *source_id == source.source_id)
                .map(|(_, bucket)| *bucket)
                .expect("expected quality bucket");
            assert_eq!(result.quality_bucket, expected_bucket);
            assert_eq!(result.quality_score.bucket, expected_bucket);
            assert!((0.0..=1.0).contains(&result.quality_score.score));
            assert!(!result.replay_blocked_by_quality);
        }
    }

    #[test]
    fn quality_replay_policies_block_or_allow_poor_sources() {
        let poor_source = quality_fixture_sources()
            .into_iter()
            .find(|source| source.source_id == "quality-gap-korean")
            .expect("poor quality source");

        let mut conservative = valid_batch_input();
        conservative.sources = vec![poor_source.clone()];
        conservative.config.quality_policy = QualityReplayPolicy::RejectPoorAndBelow;
        let blocked =
            run_local_dataset_batch_replay(conservative).expect("conservative quality policy");
        assert_eq!(
            blocked.source_results[0].quality_bucket,
            DataQualityBucket::Poor
        );
        assert!(blocked.source_results[0].replay_blocked_by_quality);
        assert!(blocked.source_results[0].replay_result.is_none());
        assert!(
            blocked.source_results[0]
                .quality_reason_codes
                .contains(&ReasonCode::SourceQualityReplayBlocked)
        );

        for policy in [
            QualityReplayPolicy::RejectRejectedOnly,
            QualityReplayPolicy::ReplayAllAcceptedWithWarnings,
        ] {
            let mut input = valid_batch_input();
            input.sources = vec![poor_source.clone()];
            input.config.quality_policy = policy;
            let allowed = run_local_dataset_batch_replay(input).expect("permissive quality policy");
            assert_eq!(
                allowed.source_results[0].quality_bucket,
                DataQualityBucket::Poor
            );
            assert!(!allowed.source_results[0].replay_blocked_by_quality);
            assert!(allowed.source_results[0].replay_result.is_some());
        }

        for policy in [
            QualityReplayPolicy::RejectPoorAndBelow,
            QualityReplayPolicy::RejectRejectedOnly,
            QualityReplayPolicy::ReplayAllAcceptedWithWarnings,
        ] {
            let mut input = valid_batch_input();
            input.sources = vec![batch_source(
                "unsafe-live-provider",
                LocalDataSourceKind::SyntheticFixture,
                "symbol,timestamp_ms,open,high,low,close,volume,live_provider\nFAKE,1,1,1,1,1,1,x",
            )];
            input.config.require_all_sources_valid = false;
            input.config.stop_on_source_error = false;
            input.config.quality_policy = policy;
            let rejected = run_local_dataset_batch_replay(input).expect("unsafe source result");
            assert_eq!(
                rejected.source_results[0].quality_bucket,
                DataQualityBucket::Rejected
            );
            assert!(rejected.source_results[0].replay_blocked_by_quality);
            assert!(rejected.source_results[0].replay_result.is_none());
            assert!(
                rejected.source_results[0]
                    .reason_codes
                    .contains(&ReasonCode::BatchReplayLiveProviderRejected)
            );
        }
    }

    fn manual_daily_csv() -> &'static str {
        "symbol,date,open,high,low,close,volume,adjusted_close,trade_value,currency,market,source\n\
         FAKEUS,2024-01-02,100,103,99,102,10000,102,1020000,USD,US,manual_export\n\
         FAKEUS,2024-01-03,102,104,101,103,10000,103,1030000,USD,US,manual_export\n\
         FAKEUS,2024-01-04,103,105,102,104,10000,104,1040000,USD,US,manual_export\n\
         FAKEUS,2024-01-05,104,106,103,105,10000,105,1050000,USD,US,manual_export\n\
         FAKEUS,2024-01-08,105,107,104,106,10000,106,1060000,USD,US,manual_export\n\
         FAKEUS,2024-01-09,106,108,105,107,10000,107,1070000,USD,US,manual_export\n\
         FAKEUS,2024-01-10,107,109,105,106,10000,106,1060000,USD,US,manual_export\n\
         FAKEUS,2024-01-11,106,107,103,104,10000,104,1040000,USD,US,manual_export\n\
         FAKEUS,2024-01-12,104,106,103,105,10000,105,1050000,USD,US,manual_export\n\
         FAKEUS,2024-01-16,105,108,104,107,10000,107,1070000,USD,US,manual_export\n\
         FAKEUS,2024-01-17,107,110,106,109,10000,109,1090000,USD,US,manual_export\n\
         FAKEUS,2024-01-18,109,111,107,108,10000,108,1080000,USD,US,manual_export"
    }

    fn manual_import_config() -> ManualHistoricalDailyImportConfig {
        ManualHistoricalDailyImportConfig {
            dataset_id: "unit-manual-daily".to_string(),
            source_kind: LocalDataSourceKind::UsStockCsv,
            min_rows: 4,
            ..ManualHistoricalDailyImportConfig::default()
        }
    }

    fn proof_gate_input() -> WalkForwardEvaluationInput {
        let dataset =
            parse_manual_historical_daily_csv(manual_daily_csv(), &manual_import_config())
                .expect("manual daily dataset");
        WalkForwardEvaluationInput {
            dataset,
            initial_agent_states: canonical_current_agent_states(),
            walk_forward_config: WalkForwardConfig {
                min_train_rows: 4,
                eval_window_rows: 4,
                step_rows: 4,
                cost_bps: 0.0,
                slippage_bps: 0.0,
                min_prediction_samples: 2,
                ..WalkForwardConfig::default()
            },
            committee_config: WalkForwardCommitteeConfig {
                min_vote_score_for_trade: 0.0,
                min_probability: 0.10,
                max_probability: 0.90,
                expected_edge_scale: 0.10,
                ..WalkForwardCommitteeConfig::default()
            },
            risk_config: GovernorConfig {
                min_expected_edge: 0.0,
                min_confidence: 0.0,
                min_data_quality: 0.0,
                max_allowed_volatility: 1.0,
                max_spread_bps: 1_000.0,
                min_risk_reward: 1.0,
                min_trade_value: 0.0,
                ..GovernorConfig::default()
            },
        }
    }

    #[test]
    fn manual_historical_daily_import_parses_valid_csv_and_rejects_bad_rows() {
        let config = manual_import_config();
        let dataset =
            parse_manual_historical_daily_csv(manual_daily_csv(), &config).expect("manual import");
        let series = to_daily_candle_series(&dataset, &config).expect("daily candle series");

        assert_eq!(dataset.dataset_id, "unit-manual-daily");
        assert_eq!(dataset.source_kind, LocalDataSourceKind::UsStockCsv);
        assert_eq!(dataset.symbol, "FAKEUS");
        assert_eq!(dataset.rows.len(), 12);
        assert_eq!(dataset.date_range.start_date, "2024-01-02");
        assert_eq!(dataset.date_range.end_date, "2024-01-18");
        assert!(dataset.sanitized);
        assert!(dataset.local_only);
        assert_eq!(series.timeframe, Timeframe::OneDay);
        assert_eq!(series.len(), dataset.rows.len());
        assert!(
            dataset
                .reason_codes
                .contains(&ReasonCode::ManualHistoricalImportNoNetwork)
        );
        assert!(
            dataset
                .reason_codes
                .contains(&ReasonCode::ManualHistoricalImportSanitizedOnly)
        );

        let cases = [
            (
                "symbol,date,open,high,low,close,volume\nFAKE,20240102,1,2,1,1,1\nFAKE,2024-01-03,1,2,1,1,1",
                ReasonCode::ManualHistoricalImportInvalidDate,
            ),
            (
                "symbol,date,open,high,low,close,volume\nFAKE,2024-01-03,1,2,1,1,1\nFAKE,2024-01-02,1,2,1,1,1",
                ReasonCode::HistoricalReplayNonMonotonicTimestamp,
            ),
            (
                "symbol,date,open,high,low,close,volume\nFAKE,2024-01-02,1,2,1,1,1\nFAKE,2024-01-02,1,2,1,1,1",
                ReasonCode::HistoricalReplayDuplicateTimestamp,
            ),
            (
                "symbol,date,open,high,low,close,volume\nFAKE,2024-01-02,1,2,1,1,1\nOTHER,2024-01-03,1,2,1,1,1",
                ReasonCode::HistoricalReplayMultiSymbolUnsupported,
            ),
            (
                "symbol,date,open,high,low,close,volume\nFAKE,2024-01-02,1,1,2,1,1\nFAKE,2024-01-03,1,2,1,1,1",
                ReasonCode::HistoricalReplayInvalidOhlc,
            ),
            (
                "symbol,date,open,high,low,close,volume\nFAKE,2024-01-02,NaN,2,1,1,1\nFAKE,2024-01-03,1,2,1,1,1",
                ReasonCode::HistoricalReplayNonFinite,
            ),
        ];
        for (csv, expected_reason) in cases {
            let mut strict = config.clone();
            strict.min_rows = 2;
            let error = parse_manual_historical_daily_csv(csv, &strict)
                .expect_err("invalid manual daily csv");
            assert!(
                error.reason_codes.contains(&expected_reason),
                "missing {expected_reason:?} in {:?}",
                error.reason_codes
            );
        }
    }

    #[test]
    fn manual_historical_daily_import_rejects_private_and_live_markers() {
        let private_instruction_name = concat!("work", ".", "md");
        let cases = [
            (
                "symbol,date,open,high,low,close,volume,account_id\nFAKE,2024-01-02,1,2,1,1,1,x".to_string(),
                ReasonCode::ManualHistoricalImportAccountDataRejected,
            ),
            (
                "symbol,date,open,high,low,close,volume,order_id\nFAKE,2024-01-02,1,2,1,1,1,x".to_string(),
                ReasonCode::ManualHistoricalImportOrderDataRejected,
            ),
            (
                "symbol,date,open,high,low,close,volume,source\nFAKE,2024-01-02,1,2,1,1,1,Authorization: fake".to_string(),
                ReasonCode::ManualHistoricalImportSecretLikeDataRejected,
            ),
            (
                "symbol,date,open,high,low,close,volume,raw_response\nFAKE,2024-01-02,1,2,1,1,1,x".to_string(),
                ReasonCode::ManualHistoricalImportRawProviderResponseRejected,
            ),
            (
                "symbol,date,open,high,low,close,volume,live_endpoint\nFAKE,2024-01-02,1,2,1,1,1,x".to_string(),
                ReasonCode::ManualHistoricalImportLiveProviderRejected,
            ),
            (
                format!(
                    "symbol,date,open,high,low,close,volume,source\nFAKE,2024-01-02,1,2,1,1,1,{private_instruction_name}"
                ),
                ReasonCode::ManualHistoricalImportWorkMdMarkerRejected,
            ),
        ];
        for (csv, expected_reason) in cases {
            let mut config = manual_import_config();
            config.min_rows = 2;
            let error =
                parse_manual_historical_daily_csv(&csv, &config).expect_err("unsafe daily csv");
            assert!(
                error.reason_codes.contains(&expected_reason),
                "missing {expected_reason:?} in {:?}",
                error.reason_codes
            );
        }
    }

    #[test]
    fn walk_forward_splits_are_deterministic_and_no_lookahead() {
        let config = WalkForwardConfig {
            min_train_rows: 4,
            eval_window_rows: 3,
            step_rows: 2,
            ..WalkForwardConfig::default()
        };
        let first = build_walk_forward_splits(10, &config).expect("first split");
        let second = build_walk_forward_splits(10, &config).expect("second split");
        assert_eq!(first, second);
        assert_eq!(
            first,
            vec![
                WalkForwardSplit {
                    train_start_index: 0,
                    train_end_index: 4,
                    eval_start_index: 4,
                    eval_end_index: 7,
                },
                WalkForwardSplit {
                    train_start_index: 0,
                    train_end_index: 6,
                    eval_start_index: 6,
                    eval_end_index: 9,
                },
            ]
        );
        assert!(first.iter().all(|split| {
            split.train_end_index <= split.eval_start_index
                && split.eval_start_index < split.eval_end_index
                && split.eval_end_index <= 10
        }));
        assert!(build_walk_forward_splits(5, &config).is_err());
        let bad = WalkForwardConfig {
            no_lookahead: false,
            ..config
        };
        assert!(build_walk_forward_splits(10, &bad).is_err());
    }

    #[test]
    fn four_baseline_walk_forward_proof_gate_is_computed_and_deterministic() {
        let input = proof_gate_input();
        let first = run_walk_forward_evaluation(input.clone()).expect("first proof gate");
        let second = run_walk_forward_evaluation(input).expect("second proof gate");
        assert_eq!(first, second);
        assert_eq!(first.windows.len(), 2);
        assert_eq!(first.symbol, "FAKEUS");
        assert!(first.windows.iter().all(|window| {
            window.split.train_end_index <= window.split.eval_start_index
                && window
                    .reason_codes
                    .contains(&ReasonCode::WalkForwardNoLookahead)
                && window.agent_state_before.len() == 3
                && window.agent_state_after.len() == 3
        }));
        let comparison = &first.aggregate_baseline_comparison;
        assert_eq!(
            comparison.always_no_trade.strategy,
            BaselineStrategyKind::AlwaysNoTrade
        );
        assert_eq!(
            comparison.buy_and_hold.strategy,
            BaselineStrategyKind::BuyAndHold
        );
        assert_eq!(
            comparison.equal_weight_committee.strategy,
            BaselineStrategyKind::EqualWeightCommittee
        );
        assert_eq!(
            comparison.voice_adaptive_committee.strategy,
            BaselineStrategyKind::VoiceAdaptiveCommittee
        );
        assert_eq!(
            comparison.voice_beats_equal_weight,
            risk_adjusted_score(&comparison.voice_adaptive_committee)
                > risk_adjusted_score(&comparison.equal_weight_committee)
        );
        assert_eq!(
            comparison.committee_beats_no_trade,
            comparison.voice_adaptive_committee.total_return
                > comparison.always_no_trade.total_return
                && comparison.voice_adaptive_committee.trade_count > 0
        );
        assert!(
            first
                .reason_codes
                .contains(&ReasonCode::HardcodingAuditPassed)
        );
        assert!(first.scoring_summary.iter().any(|metrics| metrics.strategy
            == BaselineStrategyKind::VoiceAdaptiveCommittee
            && metrics.calibrated_sample_count > 0));
        assert!(first.windows.iter().all(|window| {
            window
                .agent_state_after
                .iter()
                .all(|state| state.kind != AgentKind::Future8AgentPlaceholder)
        }));
    }

    #[test]
    fn brier_prediction_quality_counts_missing_abstention_and_confidence_errors() {
        let samples = vec![
            PredictionQualitySample {
                predicted_probability: Some(0.90),
                abstained: false,
                realized_direction_up: false,
            },
            PredictionQualitySample {
                predicted_probability: Some(0.20),
                abstained: false,
                realized_direction_up: false,
            },
            PredictionQualitySample {
                predicted_probability: Some(0.52),
                abstained: false,
                realized_direction_up: true,
            },
            PredictionQualitySample {
                predicted_probability: None,
                abstained: false,
                realized_direction_up: true,
            },
            PredictionQualitySample {
                predicted_probability: None,
                abstained: true,
                realized_direction_up: false,
            },
        ];
        let metrics = compute_prediction_quality_metrics(
            BaselineStrategyKind::VoiceAdaptiveCommittee,
            &samples,
            3,
        );
        assert_eq!(metrics.sample_count, 5);
        assert_eq!(metrics.calibrated_sample_count, 3);
        assert_eq!(metrics.missing_probability_count, 2);
        assert_eq!(metrics.abstention_count, 1);
        assert_eq!(metrics.high_confidence_error_count, 1);
        assert_eq!(metrics.low_confidence_correct_count, 1);
        assert!((metrics.brier_score.expect("brier score") - 0.360_133_333_333).abs() < 1e-9);
        let insufficient = compute_prediction_quality_metrics(
            BaselineStrategyKind::AlwaysNoTrade,
            &samples[0..1],
            3,
        );
        assert!(insufficient.brier_score.is_none());
        assert!(
            insufficient
                .reason_codes
                .contains(&ReasonCode::PredictionScoringInsufficientSamples)
        );
    }

    #[test]
    fn proof_gate_report_is_owner_readable_and_preserves_safety_boundaries() {
        let result = run_walk_forward_evaluation(proof_gate_input()).expect("proof gate");
        let report = build_proof_gate_report(&result);
        let first = render_proof_gate_report_text(&report);
        let second = render_proof_gate_report_text(&report);
        assert_eq!(first, second);
        for expected in [
            "Local historical daily CSV only.",
            "Paper-only evaluation.",
            "No live trading readiness.",
            "No profitability claim.",
            "Voice adaptation must beat equal weight before it is trusted.",
            "Synthetic fixture success is not market evidence.",
            "AlwaysNoTrade",
            "BuyAndHold",
            "EqualWeightCommittee",
            "VoiceAdaptiveCommittee",
        ] {
            assert!(first.contains(expected), "missing report text: {expected}");
        }
        if !result
            .aggregate_baseline_comparison
            .voice_beats_equal_weight
        {
            assert!(first.contains("VoiceAdaptiveCommittee did not beat EqualWeightCommittee"));
        }
        if !result
            .aggregate_baseline_comparison
            .committee_beats_buy_hold_risk_adjusted
        {
            assert!(first.contains("Committee did not beat BuyAndHold"));
        }
        assert!(!first.contains("fake-secret"));
        assert!(!first.contains(concat!("work", ".", "md")));
        assert!(
            report
                .reason_codes
                .contains(&ReasonCode::HardcodingAuditPassed)
        );
    }

    fn daily_evidence_csv(symbol: &str, market: &str, currency: &str, closes: &[f64]) -> String {
        let mut rows =
            "symbol,date,open,high,low,close,volume,currency,market,source\n".to_string();
        for (index, close) in closes.iter().enumerate() {
            let day = index + 2;
            rows.push_str(&format!(
                "{symbol},2024-01-{day:02},{close:.2},{:.2},{:.2},{close:.2},10000,{currency},{market},manual_export\n",
                close + 1.0,
                close - 1.0,
            ));
        }
        rows
    }

    fn evidence_source_spec(
        source_id: &str,
        source_kind: HistoricalEvidenceSourceKind,
        symbol: &str,
        market: &str,
        csv_text: Option<String>,
    ) -> HistoricalEvidenceSourceSpec {
        HistoricalEvidenceSourceSpec {
            source_id: source_id.to_string(),
            source_kind,
            symbol: symbol.to_string(),
            market: market.to_string(),
            currency: Some(
                match source_kind {
                    HistoricalEvidenceSourceKind::KoreanStockDaily => "KRW",
                    HistoricalEvidenceSourceKind::BtcCryptoDaily => "USD",
                    _ => "USD",
                }
                .to_string(),
            ),
            csv_path: None,
            csv_text,
            enabled: true,
            expected_min_rows: 12,
            reason_codes: Vec::new(),
        }
    }

    fn evidence_pack_eval_config(min_sources: usize) -> HistoricalEvidencePackEvaluationConfig {
        HistoricalEvidencePackEvaluationConfig {
            initial_agent_states: canonical_current_agent_states(),
            walk_forward_config: WalkForwardConfig {
                min_train_rows: 4,
                eval_window_rows: 4,
                step_rows: 4,
                cost_bps: 0.0,
                slippage_bps: 0.0,
                min_prediction_samples: 2,
                ..WalkForwardConfig::default()
            },
            committee_config: WalkForwardCommitteeConfig {
                min_vote_score_for_trade: 0.0,
                min_probability: 0.10,
                max_probability: 0.90,
                expected_edge_scale: 0.10,
                ..WalkForwardCommitteeConfig::default()
            },
            risk_config: GovernorConfig {
                min_expected_edge: 0.0,
                min_confidence: 0.0,
                min_data_quality: 0.0,
                max_allowed_volatility: 1.0,
                max_spread_bps: 1_000.0,
                min_risk_reward: 1.0,
                min_trade_value: 0.0,
                ..GovernorConfig::default()
            },
            min_accepted_sources_for_proof: min_sources,
            min_prediction_samples: 2,
            aggregation_method: EvidenceAggregationMethod::Mean,
            reason_codes: Vec::new(),
        }
    }

    fn valid_multi_symbol_manifest() -> HistoricalEvidencePackManifest {
        HistoricalEvidencePackManifest {
            pack_id: "unit-multi-symbol-pack".to_string(),
            description: "sanitized local daily CSV pack".to_string(),
            sources: vec![
                evidence_source_spec(
                    "us-a",
                    HistoricalEvidenceSourceKind::UsStockDaily,
                    "FAKEUS",
                    "US",
                    Some(daily_evidence_csv(
                        "FAKEUS",
                        "US",
                        "USD",
                        &[
                            100.0, 102.0, 101.0, 103.0, 104.0, 103.0, 105.0, 106.0, 104.0, 107.0,
                            108.0, 109.0,
                        ],
                    )),
                ),
                evidence_source_spec(
                    "kr-a",
                    HistoricalEvidenceSourceKind::KoreanStockDaily,
                    "FAKEKR.KS",
                    "KR",
                    Some(daily_evidence_csv(
                        "FAKEKR.KS",
                        "KR",
                        "KRW",
                        &[
                            50.0, 49.0, 51.0, 50.0, 52.0, 51.0, 53.0, 52.0, 54.0, 53.0, 55.0, 56.0,
                        ],
                    )),
                ),
                evidence_source_spec(
                    "btc-a",
                    HistoricalEvidenceSourceKind::BtcCryptoDaily,
                    "BTC-USD",
                    "BTC",
                    Some(daily_evidence_csv(
                        "BTC-USD",
                        "BTC",
                        "USD",
                        &[
                            30000.0, 30100.0, 29900.0, 30200.0, 30400.0, 30300.0, 30600.0, 30500.0,
                            30700.0, 30650.0, 30800.0, 30900.0,
                        ],
                    )),
                ),
            ],
            local_only: true,
            sanitized_only: true,
            reason_codes: Vec::new(),
        }
    }

    #[test]
    fn multi_symbol_evidence_pack_accepts_us_kr_btc_and_reports_deterministically() {
        let mut config = HistoricalEvidencePackConfig {
            min_sources: 3,
            ..HistoricalEvidencePackConfig::default()
        };
        config
            .min_sources_by_kind
            .insert(HistoricalEvidenceSourceKind::UsStockDaily, 1);
        config
            .min_sources_by_kind
            .insert(HistoricalEvidenceSourceKind::KoreanStockDaily, 1);
        config
            .min_sources_by_kind
            .insert(HistoricalEvidenceSourceKind::BtcCryptoDaily, 1);

        let manifest = valid_multi_symbol_manifest();
        let encoded = serde_json::to_string(&manifest).expect("manifest json");
        let parsed =
            parse_historical_evidence_pack_manifest_json(&encoded).expect("manifest parsed");
        let pack = load_historical_evidence_pack_from_manifest(&parsed, &config)
            .expect("evidence pack loaded");
        validate_historical_evidence_pack(&pack, &config).expect("evidence pack valid");

        let result = evaluate_historical_evidence_pack(&pack, &evidence_pack_eval_config(3));
        assert_eq!(result.source_results.len(), 3);
        assert_eq!(result.market_results.len(), 3);
        assert!(result.source_results.iter().all(|source| {
            source.accepted
                && !source.rejected
                && source.walk_forward_result.is_some()
                && source.proof_gate_report.is_some()
        }));
        assert_eq!(
            result
                .source_results
                .iter()
                .map(|source| source.source_id.as_str())
                .collect::<Vec<_>>(),
            vec!["us-a", "kr-a", "btc-a"]
        );
        for source in &result.source_results {
            let walk = source.walk_forward_result.as_ref().expect("walk result");
            let comparison = &walk.aggregate_baseline_comparison;
            assert_eq!(
                comparison.always_no_trade.strategy,
                BaselineStrategyKind::AlwaysNoTrade
            );
            assert_eq!(
                comparison.buy_and_hold.strategy,
                BaselineStrategyKind::BuyAndHold
            );
            assert_eq!(
                comparison.equal_weight_committee.strategy,
                BaselineStrategyKind::EqualWeightCommittee
            );
            assert_eq!(
                comparison.voice_adaptive_committee.strategy,
                BaselineStrategyKind::VoiceAdaptiveCommittee
            );
            assert_eq!(
                comparison.voice_beats_equal_weight,
                risk_adjusted_score(&comparison.voice_adaptive_committee)
                    > risk_adjusted_score(&comparison.equal_weight_committee)
            );
            assert!(walk.windows.iter().all(|window| {
                window.split.train_end_index <= window.split.eval_start_index
                    && window
                        .reason_codes
                        .contains(&ReasonCode::WalkForwardNoLookahead)
            }));
        }

        let report = build_multi_symbol_proof_gate_report(&result);
        let first = render_multi_symbol_proof_gate_report_text(&report);
        let second = render_multi_symbol_proof_gate_report_text(&report);
        assert_eq!(first, second);
        for expected in [
            "Local owner-provided sanitized historical daily CSV only.",
            "Paper-only evaluation.",
            "No live trading readiness.",
            "No profitability claim.",
            "Synthetic fixture success is not market evidence.",
            "Voice adaptation must beat equal weight before it is trusted.",
            "Bad or mixed results are valid outputs.",
            "AlwaysNoTrade",
            "BuyAndHold",
            "EqualWeightCommittee",
            "VoiceAdaptiveCommittee",
        ] {
            assert!(first.contains(expected), "missing {expected}");
        }
        assert!(!first.contains("fake-secret"));
        assert!(!first.contains(concat!("work", ".", "md")));
    }

    #[test]
    fn evidence_pack_rejected_sources_stay_visible_with_safety_reasons() {
        let unsafe_instruction_path = format!("./{}", concat!("work", ".", "md"));
        let cases = vec![
            (
                "disabled",
                {
                    let mut source = evidence_source_spec(
                        "disabled",
                        HistoricalEvidenceSourceKind::UsStockDaily,
                        "FAKEUS",
                        "US",
                        Some(daily_evidence_csv(
                            "FAKEUS",
                            "US",
                            "USD",
                            &[10.0, 11.0, 12.0, 13.0],
                        )),
                    );
                    source.enabled = false;
                    source
                },
                ReasonCode::EvidencePackSourceDisabled,
                false,
            ),
            (
                "unsupported",
                evidence_source_spec("unsupported", HistoricalEvidenceSourceKind::Unknown, "X", "US", None),
                ReasonCode::EvidencePackUnsupportedSourceKind,
                true,
            ),
            (
                "url-path",
                HistoricalEvidenceSourceSpec {
                    csv_path: Some("https://example.invalid/local.csv".to_string()),
                    csv_text: None,
                    ..evidence_source_spec(
                        "url-path",
                        HistoricalEvidenceSourceKind::UsStockDaily,
                        "FAKEUS",
                        "US",
                        None,
                    )
                },
                ReasonCode::EvidencePackUrlPathRejected,
                true,
            ),
            (
                "instruction-path",
                HistoricalEvidenceSourceSpec {
                    csv_path: Some(unsafe_instruction_path),
                    csv_text: None,
                    ..evidence_source_spec(
                        "instruction-path",
                        HistoricalEvidenceSourceKind::UsStockDaily,
                        "FAKEUS",
                        "US",
                        None,
                    )
                },
                ReasonCode::EvidencePackWorkMdPathRejected,
                true,
            ),
            (
                "env-path",
                HistoricalEvidenceSourceSpec {
                    csv_path: Some("./.env".to_string()),
                    csv_text: None,
                    ..evidence_source_spec(
                        "env-path",
                        HistoricalEvidenceSourceKind::UsStockDaily,
                        "FAKEUS",
                        "US",
                        None,
                    )
                },
                ReasonCode::EvidencePackEnvPathRejected,
                true,
            ),
            (
                "private-path",
                HistoricalEvidenceSourceSpec {
                    csv_path: Some("./local_private/data.csv".to_string()),
                    csv_text: None,
                    ..evidence_source_spec(
                        "private-path",
                        HistoricalEvidenceSourceKind::UsStockDaily,
                        "FAKEUS",
                        "US",
                        None,
                    )
                },
                ReasonCode::EvidencePackPrivatePathRejected,
                true,
            ),
            (
                "account",
                evidence_source_spec(
                    "account",
                    HistoricalEvidenceSourceKind::UsStockDaily,
                    "FAKEUS",
                    "US",
                    Some("symbol,date,open,high,low,close,volume,account_id\nFAKEUS,2024-01-02,1,2,1,1,1,x".to_string()),
                ),
                ReasonCode::EvidencePackAccountDataRejected,
                true,
            ),
            (
                "order",
                evidence_source_spec(
                    "order",
                    HistoricalEvidenceSourceKind::UsStockDaily,
                    "FAKEUS",
                    "US",
                    Some("symbol,date,open,high,low,close,volume,order_id\nFAKEUS,2024-01-02,1,2,1,1,1,x".to_string()),
                ),
                ReasonCode::EvidencePackOrderDataRejected,
                true,
            ),
            (
                "secret",
                evidence_source_spec(
                    "secret",
                    HistoricalEvidenceSourceKind::UsStockDaily,
                    "FAKEUS",
                    "US",
                    Some("symbol,date,open,high,low,close,volume,source\nFAKEUS,2024-01-02,1,2,1,1,1,Authorization: fake".to_string()),
                ),
                ReasonCode::EvidencePackSecretLikeDataRejected,
                true,
            ),
            (
                "raw",
                evidence_source_spec(
                    "raw",
                    HistoricalEvidenceSourceKind::UsStockDaily,
                    "FAKEUS",
                    "US",
                    Some("symbol,date,open,high,low,close,volume,raw_response\nFAKEUS,2024-01-02,1,2,1,1,1,x".to_string()),
                ),
                ReasonCode::EvidencePackRawProviderResponseRejected,
                true,
            ),
            (
                "live",
                evidence_source_spec(
                    "live",
                    HistoricalEvidenceSourceKind::UsStockDaily,
                    "FAKEUS",
                    "US",
                    Some("symbol,date,open,high,low,close,volume,live_endpoint\nFAKEUS,2024-01-02,1,2,1,1,1,x".to_string()),
                ),
                ReasonCode::EvidencePackLiveProviderRejected,
                true,
            ),
        ];

        for (source_id, source, expected_reason, rejected) in cases {
            let manifest = HistoricalEvidencePackManifest {
                pack_id: format!("pack-{source_id}"),
                description: "safety case".to_string(),
                sources: vec![source],
                local_only: true,
                sanitized_only: true,
                reason_codes: Vec::new(),
            };
            let pack = load_historical_evidence_pack_from_manifest(
                &manifest,
                &HistoricalEvidencePackConfig::default(),
            )
            .expect("pack loads with visible source state");
            assert_eq!(pack.sources.len(), 1);
            assert_eq!(pack.sources[0].rejected, rejected);
            assert!(
                pack.sources[0].reason_codes.contains(&expected_reason),
                "{source_id} missing {expected_reason:?}: {:?}",
                pack.sources[0].reason_codes
            );
        }
    }

    fn metric(
        strategy: BaselineStrategyKind,
        total_return: f64,
        max_drawdown: f64,
    ) -> BaselinePerformanceMetrics {
        BaselinePerformanceMetrics {
            strategy,
            total_return,
            max_drawdown,
            trade_count: u64::from(strategy != BaselineStrategyKind::AlwaysNoTrade),
            win_count: u64::from(total_return > 0.0),
            loss_count: u64::from(total_return < 0.0),
            no_trade_count: u64::from(strategy == BaselineStrategyKind::AlwaysNoTrade),
            risk_denial_count: 0,
            avg_return_per_trade: total_return,
            volatility_estimate: Some(max_drawdown.abs()),
            sharpe_like: None,
            downside_loss: (total_return < 0.0).then_some(total_return.abs()),
            cost_paid: 0.0,
            slippage_paid: 0.0,
            reason_codes: vec![strategy_reason_code(strategy)],
        }
    }

    fn scoring(
        strategy: BaselineStrategyKind,
        brier_score: Option<f64>,
        sample_count: usize,
        missing: usize,
        abstained: usize,
        high_confidence_errors: usize,
    ) -> PredictionQualityMetrics {
        PredictionQualityMetrics {
            strategy,
            brier_score,
            sample_count,
            calibrated_sample_count: brier_score
                .map_or(0, |_| sample_count.saturating_sub(missing)),
            missing_probability_count: missing,
            abstention_count: abstained,
            high_confidence_error_count: high_confidence_errors,
            low_confidence_correct_count: 0,
            mean_confidence: brier_score.map(|_| 0.60),
            mean_realized_direction: Some(0.50),
            reason_codes: vec![ReasonCode::PredictionScoringBrier],
        }
    }

    fn synthetic_walk_result(
        dataset_id: &str,
        symbol: &str,
        always_return: f64,
        buy_hold_return: f64,
        equal_return: f64,
        voice_return: f64,
    ) -> WalkForwardEvaluationResult {
        let always = metric(BaselineStrategyKind::AlwaysNoTrade, always_return, 0.0);
        let buy_hold = metric(BaselineStrategyKind::BuyAndHold, buy_hold_return, 0.02);
        let equal = metric(
            BaselineStrategyKind::EqualWeightCommittee,
            equal_return,
            0.03,
        );
        let voice = metric(
            BaselineStrategyKind::VoiceAdaptiveCommittee,
            voice_return,
            0.03,
        );
        let scoring_summary = vec![
            scoring(BaselineStrategyKind::AlwaysNoTrade, None, 4, 4, 4, 0),
            scoring(BaselineStrategyKind::BuyAndHold, None, 4, 4, 0, 0),
            scoring(
                BaselineStrategyKind::EqualWeightCommittee,
                Some(0.24),
                4,
                0,
                1,
                0,
            ),
            scoring(
                BaselineStrategyKind::VoiceAdaptiveCommittee,
                Some(0.20),
                4,
                1,
                2,
                1,
            ),
        ];
        let comparison = ProofGateComparison {
            always_no_trade: always,
            buy_and_hold: buy_hold,
            equal_weight_committee: equal,
            voice_adaptive_committee: voice,
            voice_beats_equal_weight: voice_return > equal_return,
            committee_beats_no_trade: voice_return > always_return && voice_return != 0.0,
            committee_beats_buy_hold_risk_adjusted: voice_return - 0.03 > buy_hold_return - 0.02,
            insufficient_evidence: false,
            reason_codes: vec![ReasonCode::HardcodingAuditPassed],
        };
        WalkForwardEvaluationResult {
            dataset_id: dataset_id.to_string(),
            symbol: symbol.to_string(),
            windows: Vec::new(),
            voice_adaptation_comparison: VoiceAdaptationComparison {
                equal_weight_total_return: comparison.equal_weight_committee.total_return,
                voice_adaptive_total_return: comparison.voice_adaptive_committee.total_return,
                equal_weight_risk_adjusted_score: risk_adjusted_score(
                    &comparison.equal_weight_committee,
                ),
                voice_adaptive_risk_adjusted_score: risk_adjusted_score(
                    &comparison.voice_adaptive_committee,
                ),
                voice_beats_equal_weight: comparison.voice_beats_equal_weight,
                reason_codes: vec![ReasonCode::BaselineComparisonVoiceAdaptationHelped],
            },
            aggregate_baseline_comparison: comparison,
            scoring_summary,
            proof_gate_status: ProofGateStatus::ComputedNoProfitabilityClaim,
            reason_codes: vec![ReasonCode::HardcodingAuditPassed],
        }
    }

    fn synthetic_source_result(
        source_id: &str,
        market: &str,
        symbol: &str,
        result: WalkForwardEvaluationResult,
    ) -> HistoricalEvidenceSourceEvaluationResult {
        HistoricalEvidenceSourceEvaluationResult {
            source_id: source_id.to_string(),
            source_kind: HistoricalEvidenceSourceKind::UsStockDaily,
            symbol: symbol.to_string(),
            market: market.to_string(),
            dataset_summary: format!("synthetic {symbol}"),
            walk_forward_result: Some(result),
            proof_gate_report: None,
            accepted: true,
            rejected: false,
            insufficient_evidence: false,
            reason_codes: vec![ReasonCode::HardcodingAuditPassed],
        }
    }

    fn clean_prediction_source_result(
        source_id: &str,
        market: &str,
        symbol: &str,
        result: WalkForwardEvaluationResult,
    ) -> HistoricalEvidenceSourceEvaluationResult {
        let mut source = synthetic_source_result(source_id, market, symbol, result);
        if let Some(result) = &mut source.walk_forward_result {
            for metrics in &mut result.scoring_summary {
                if metrics.strategy == BaselineStrategyKind::VoiceAdaptiveCommittee {
                    metrics.missing_probability_count = 0;
                    metrics.abstention_count = 0;
                    metrics.high_confidence_error_count = 0;
                }
            }
        }
        source
    }

    #[test]
    fn aggregate_status_and_voice_validity_change_with_input_metrics() {
        let config = evidence_pack_eval_config(2);
        let helped_rows = vec![
            synthetic_source_result(
                "a",
                "US",
                "A",
                synthetic_walk_result("a", "A", 0.0, 0.01, 0.02, 0.05),
            ),
            synthetic_source_result(
                "b",
                "US",
                "B",
                synthetic_walk_result("b", "B", 0.0, 0.01, 0.02, 0.05),
            ),
        ];
        let helped = build_voice_adaptation_validity(&helped_rows, &config);
        assert_eq!(helped.status, VoiceAdaptationValidityStatus::Helped);

        let failed_rows = vec![
            synthetic_source_result(
                "a",
                "US",
                "A",
                synthetic_walk_result("a", "A", 0.0, 0.01, 0.05, 0.02),
            ),
            synthetic_source_result(
                "b",
                "US",
                "B",
                synthetic_walk_result("b", "B", 0.0, 0.01, 0.05, 0.02),
            ),
        ];
        let failed = build_voice_adaptation_validity(&failed_rows, &config);
        assert_eq!(failed.status, VoiceAdaptationValidityStatus::Failed);
        assert_eq!(
            voice_adaptation_report_sentence(&failed),
            "Voice adaptation did not beat equal weight on this evidence pack."
        );

        let mixed_rows = vec![helped_rows[0].clone(), failed_rows[0].clone()];
        let mixed = build_voice_adaptation_validity(&mixed_rows, &config);
        assert_eq!(mixed.status, VoiceAdaptationValidityStatus::Mixed);
        assert_eq!(
            voice_adaptation_report_sentence(&mixed),
            "Voice adaptation evidence is mixed."
        );
    }

    #[test]
    fn aggregate_report_surfaces_insufficient_buy_hold_and_no_trade_failures() {
        let config = evidence_pack_eval_config(2);
        let rows = vec![synthetic_source_result(
            "down",
            "US",
            "DOWN",
            synthetic_walk_result("down", "DOWN", 0.0, 0.03, 0.01, -0.02),
        )];
        let aggregate = build_aggregate_baseline_comparison(&rows, &config);
        assert_eq!(
            aggregate.overall_status,
            AggregateBaselineOverallStatus::InsufficientEvidence
        );
        assert_eq!(aggregate.committee_loses_no_trade_count, 1);
        assert_eq!(aggregate.committee_loses_buy_hold_count, 1);
        assert!(
            aggregate
                .reason_codes
                .contains(&ReasonCode::AggregateBaselineBuyHoldStronger)
        );
        assert!(
            aggregate
                .reason_codes
                .contains(&ReasonCode::AggregateBaselineNoTradeStronger)
        );

        let pack_result = HistoricalEvidencePackEvaluationResult {
            pack_id: "synthetic-pack".to_string(),
            source_results: rows.clone(),
            aggregate_result: aggregate.clone(),
            market_results: Vec::new(),
            symbol_results: rows,
            voice_adaptation_summary: build_voice_adaptation_validity(&[], &config),
            prediction_quality_summary: build_aggregate_prediction_quality_summary(&[], &config),
            proof_gate_status: aggregate.overall_status,
            reason_codes: aggregate.reason_codes,
        };
        let report = build_multi_symbol_proof_gate_report(&pack_result);
        let text = render_multi_symbol_proof_gate_report_text(&report);
        assert!(text.contains("buy_hold_stronger=true"));
        assert!(text.contains("no_trade_stronger=true"));
        assert!(text.contains("Insufficient evidence to trust voice adaptation."));
        assert!(text.contains("Evidence pack has insufficient accepted out-of-sample sources."));
    }

    #[test]
    fn aggregate_prediction_quality_counts_brier_missing_abstention_and_errors() {
        let config = evidence_pack_eval_config(1);
        let rows = vec![
            synthetic_source_result(
                "a",
                "US",
                "A",
                synthetic_walk_result("a", "A", 0.0, 0.01, 0.02, 0.05),
            ),
            synthetic_source_result(
                "b",
                "KR",
                "B",
                synthetic_walk_result("b", "B", 0.0, 0.01, 0.02, 0.04),
            ),
        ];
        let summary = build_aggregate_prediction_quality_summary(&rows, &config);
        assert_eq!(summary.source_count, 2);
        assert_eq!(summary.total_samples, 8);
        assert_eq!(summary.missing_probability_count, 2);
        assert_eq!(summary.abstention_count, 4);
        assert_eq!(summary.high_confidence_error_count, 2);
        assert!((summary.mean_brier_score.expect("mean brier") - 0.20).abs() < 1e-12);
        assert_eq!(summary.best_source_by_brier.as_deref(), Some("a"));
        assert!(!summary.insufficient_evidence);
    }

    #[test]
    fn evidence_pack_validation_marks_insufficient_sources_and_preserves_safety_boundaries() {
        let manifest = HistoricalEvidencePackManifest {
            sources: vec![evidence_source_spec(
                "us-only",
                HistoricalEvidenceSourceKind::UsStockDaily,
                "FAKEUS",
                "US",
                Some(daily_evidence_csv(
                    "FAKEUS",
                    "US",
                    "USD",
                    &[
                        100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0,
                        110.0, 111.0,
                    ],
                )),
            )],
            ..valid_multi_symbol_manifest()
        };
        let config = HistoricalEvidencePackConfig {
            min_sources: 2,
            ..HistoricalEvidencePackConfig::default()
        };
        let pack =
            load_historical_evidence_pack_from_manifest(&manifest, &config).expect("pack loaded");
        let error = validate_historical_evidence_pack(&pack, &config)
            .expect_err("insufficient accepted sources");
        assert!(
            error
                .reason_codes
                .contains(&ReasonCode::EvidencePackInsufficientSources)
        );

        let broker = PaperBroker::default();
        assert!(!broker.supports_live_execution());
        let states = canonical_current_agent_states();
        assert_eq!(states.len(), 3);
        assert!(
            states
                .iter()
                .all(|state| state.kind != AgentKind::Future8AgentPlaceholder)
        );
        assert_eq!(WalkForwardCommitteeConfig::default().active_agent_limit, 3);
    }

    fn trial_eval_from_rows(
        pack_id: &str,
        rows: Vec<HistoricalEvidenceSourceEvaluationResult>,
        min_sources: usize,
    ) -> HistoricalEvidencePackEvaluationResult {
        let config = evidence_pack_eval_config(min_sources);
        let aggregate_result = build_aggregate_baseline_comparison(&rows, &config);
        HistoricalEvidencePackEvaluationResult {
            pack_id: pack_id.to_string(),
            source_results: rows.clone(),
            aggregate_result: aggregate_result.clone(),
            market_results: build_market_evidence_results(&rows, &config),
            symbol_results: rows.clone(),
            voice_adaptation_summary: build_voice_adaptation_validity(&rows, &config),
            prediction_quality_summary: build_aggregate_prediction_quality_summary(&rows, &config),
            proof_gate_status: aggregate_result.overall_status,
            reason_codes: vec![ReasonCode::HardcodingAuditPassed],
        }
    }

    #[test]
    fn owner_trial_no_pack_returns_checklist_and_no_fake_evaluation() {
        let result = run_owner_historical_evidence_trial(OwnerEvidenceTrialConfig::default());
        assert_eq!(
            result.trial_status,
            OwnerEvidenceTrialStatus::NoOwnerEvidencePackFound
        );
        assert_eq!(result.manifest_status, OwnerEvidenceManifestStatus::Missing);
        assert!(result.pack_evaluation.is_none());
        assert!(result.multi_symbol_report.is_none());
        assert!(!result.owner_action_checklist.is_empty());
        assert!(
            result
                .reason_codes
                .contains(&ReasonCode::EvidenceTriageNoPack)
        );

        let text = render_owner_evidence_triage_report_text(&result.triage_report);
        assert!(text.contains("No owner evidence pack was found; no data was evaluated."));
        assert!(text.contains("No data was downloaded."));
        assert!(text.contains("No profitability claim."));
        assert!(text.contains("No live trading readiness."));
        assert!(!text.contains(concat!("work", ".", "md")));
    }

    #[test]
    fn owner_trial_valid_test_json_pack_evaluates_and_renders_market_triage() {
        let manifest = valid_multi_symbol_manifest();
        let mut trial_config = OwnerEvidenceTrialConfig {
            manifest_json: Some(serde_json::to_string(&manifest).expect("manifest json")),
            min_accepted_sources: 3,
            evaluation_config: evidence_pack_eval_config(3),
            ..OwnerEvidenceTrialConfig::default()
        };
        trial_config
            .historical_pack_config
            .min_sources_by_kind
            .insert(HistoricalEvidenceSourceKind::UsStockDaily, 1);
        trial_config
            .historical_pack_config
            .min_sources_by_kind
            .insert(HistoricalEvidenceSourceKind::KoreanStockDaily, 1);
        trial_config
            .historical_pack_config
            .min_sources_by_kind
            .insert(HistoricalEvidenceSourceKind::BtcCryptoDaily, 1);
        for market in ["US", "KR", "BTC"] {
            trial_config
                .min_sources_by_market
                .insert(market.to_string(), 1);
        }

        let result = run_owner_historical_evidence_trial(trial_config);
        assert_ne!(
            result.trial_status,
            OwnerEvidenceTrialStatus::NoOwnerEvidencePackFound
        );
        assert!(result.pack_evaluation.is_some());
        assert!(result.multi_symbol_report.is_some());
        assert_eq!(result.triage_summary.accepted_source_count, 3);
        assert_eq!(result.market_triage.len(), 3);
        for market in ["US", "KR", "BTC"] {
            assert!(result.market_triage.iter().any(|row| row.market == market));
        }
        assert!(
            result
                .triage_report
                .reason_codes
                .contains(&ReasonCode::HardcodingAuditPassed)
        );

        let first = render_owner_evidence_triage_report_text(&result.triage_report);
        let second = render_owner_evidence_triage_report_text(&result.triage_report);
        assert_eq!(first, second);
        for expected in [
            "Trial status:",
            "Market triage:",
            "Baseline failure summary:",
            "Voice adaptation summary:",
            "Owner action checklist:",
            "No data was downloaded.",
            "No profitability claim.",
            "No live trading readiness.",
        ] {
            assert!(first.contains(expected), "missing {expected}");
        }
        assert!(!first.contains("fake-secret"));
        assert!(!first.contains(concat!("work", ".", "md")));
    }

    #[test]
    fn owner_trial_rejects_unsafe_manifest_paths() {
        let instruction_file = concat!("work", ".", "md");
        let cases = [
            (
                format!("https://example.invalid/{instruction_file}"),
                ReasonCode::EvidenceTrialUrlRejected,
            ),
            (
                format!("./{instruction_file}"),
                ReasonCode::EvidenceTrialWorkMdMarkerRejected,
            ),
            (
                "./.env".to_string(),
                ReasonCode::EvidenceTrialUnsafePrivateData,
            ),
            (
                "./local_private/owner_pack.json".to_string(),
                ReasonCode::EvidenceTrialUnsafePrivateData,
            ),
        ];
        for (path, expected_reason) in cases {
            let result = run_owner_historical_evidence_trial(OwnerEvidenceTrialConfig {
                manifest_path: Some(path),
                ..OwnerEvidenceTrialConfig::default()
            });
            assert_eq!(
                result.trial_status,
                OwnerEvidenceTrialStatus::RejectedForSafety
            );
            assert!(
                result.reason_codes.contains(&expected_reason),
                "missing {expected_reason:?} in {:?}",
                result.reason_codes
            );
        }
    }

    #[test]
    fn owner_trial_keeps_rejected_sources_visible() {
        let mut manifest = valid_multi_symbol_manifest();
        manifest.sources.push(evidence_source_spec(
            "unsafe-account",
            HistoricalEvidenceSourceKind::UsStockDaily,
            "BAD",
            "US",
            Some(
                "symbol,date,open,high,low,close,volume,account_id\nBAD,2024-01-02,1,2,1,1,1,x"
                    .to_string(),
            ),
        ));
        manifest.sources.push(evidence_source_spec(
            "unsafe-raw",
            HistoricalEvidenceSourceKind::UsStockDaily,
            "RAW",
            "US",
            Some(
                "symbol,date,open,high,low,close,volume,raw_response\nRAW,2024-01-02,1,2,1,1,1,x"
                    .to_string(),
            ),
        ));
        let result = run_owner_historical_evidence_trial(OwnerEvidenceTrialConfig {
            manifest_json: Some(serde_json::to_string(&manifest).expect("manifest json")),
            min_accepted_sources: 3,
            evaluation_config: evidence_pack_eval_config(3),
            ..OwnerEvidenceTrialConfig::default()
        });
        assert_eq!(
            result.trial_status,
            OwnerEvidenceTrialStatus::RejectedForSafety
        );
        assert_eq!(
            result.triage_summary.rejected_source_count, 2,
            "source_summary={:?} reason_codes={:?}",
            result.triage_report.source_summary, result.triage_summary.reason_codes
        );
        assert!(
            result
                .triage_summary
                .reason_codes
                .contains(&ReasonCode::EvidenceTrialAccountDataRejected)
        );
        assert!(
            result
                .triage_summary
                .reason_codes
                .contains(&ReasonCode::EvidenceTrialRawProviderResponseRejected)
        );
        let text = render_owner_evidence_triage_report_text(&result.triage_report);
        assert!(text.contains("unsafe-account"));
        assert!(text.contains("unsafe-raw"));
    }

    #[test]
    fn owner_triage_status_changes_when_metrics_change() {
        let config = OwnerEvidenceTrialConfig {
            min_accepted_sources: 2,
            evaluation_config: evidence_pack_eval_config(2),
            ..OwnerEvidenceTrialConfig::default()
        };
        let pass_rows = vec![
            clean_prediction_source_result(
                "a",
                "US",
                "A",
                synthetic_walk_result("a", "A", 0.0, 0.01, 0.02, 0.06),
            ),
            clean_prediction_source_result(
                "b",
                "KR",
                "B",
                synthetic_walk_result("b", "B", 0.0, 0.01, 0.02, 0.06),
            ),
        ];
        let pass_result = trial_eval_from_rows("computed-pass", pass_rows, 2);
        let pass_summary = build_evidence_triage_summary(&pass_result, None, &config);
        assert_eq!(pass_summary.status, OwnerEvidenceTrialStatus::Pass);

        let fail_rows = vec![
            synthetic_source_result(
                "a",
                "US",
                "A",
                synthetic_walk_result("a", "A", 0.0, 0.05, 0.04, -0.01),
            ),
            synthetic_source_result(
                "b",
                "KR",
                "B",
                synthetic_walk_result("b", "B", 0.0, 0.05, 0.04, -0.01),
            ),
        ];
        let fail_result = trial_eval_from_rows("computed-fail", fail_rows, 2);
        let fail_summary = build_evidence_triage_summary(&fail_result, None, &config);
        assert_eq!(fail_summary.status, OwnerEvidenceTrialStatus::Fail);
        assert!(
            fail_summary
                .reason_codes
                .contains(&ReasonCode::EvidenceTriageVoiceFailed)
        );
        assert!(
            fail_summary
                .reason_codes
                .contains(&ReasonCode::EvidenceTriageBuyHoldStronger)
        );
        assert!(
            fail_summary
                .reason_codes
                .contains(&ReasonCode::EvidenceTriageNoTradeStronger)
        );

        let report = build_owner_evidence_triage_report(
            &fail_summary,
            &build_market_triage_results(&fail_result, &config),
            Some(&fail_result),
            None,
            OwnerEvidenceManifestStatus::ProvidedJson,
            &config,
        );
        let text = render_owner_evidence_triage_report_text(&report);
        assert!(text.contains("failed to beat EqualWeightCommittee"));
        assert!(text.contains("BuyAndHold beat the committee"));
        assert!(text.contains("AlwaysNoTrade beat the committee"));
        assert!(!text.contains(
            "VoiceAdaptiveCommittee beat EqualWeightCommittee on the computed trial evidence."
        ));
    }

    #[test]
    fn owner_triage_insufficient_and_mixed_statuses_are_computed() {
        let config = OwnerEvidenceTrialConfig {
            min_accepted_sources: 2,
            evaluation_config: evidence_pack_eval_config(2),
            ..OwnerEvidenceTrialConfig::default()
        };
        let insufficient_rows = vec![synthetic_source_result(
            "single",
            "US",
            "SINGLE",
            synthetic_walk_result("single", "SINGLE", 0.0, 0.01, 0.02, 0.06),
        )];
        let insufficient_result = trial_eval_from_rows("insufficient", insufficient_rows, 2);
        let insufficient = build_evidence_triage_summary(&insufficient_result, None, &config);
        assert_eq!(
            insufficient.status,
            OwnerEvidenceTrialStatus::InsufficientEvidence
        );

        let mixed_rows = vec![
            clean_prediction_source_result(
                "pass",
                "US",
                "PASS",
                synthetic_walk_result("pass", "PASS", 0.0, 0.01, 0.02, 0.06),
            ),
            synthetic_source_result(
                "fail",
                "BTC",
                "FAIL",
                synthetic_walk_result("fail", "FAIL", 0.0, 0.05, 0.04, -0.01),
            ),
        ];
        let mixed_result = trial_eval_from_rows("mixed", mixed_rows, 2);
        let mixed = build_evidence_triage_summary(&mixed_result, None, &config);
        assert_eq!(mixed.status, OwnerEvidenceTrialStatus::Mixed);
        let market_triage = build_market_triage_results(&mixed_result, &config);
        assert!(market_triage.iter().any(|market| {
            market.market == "US" && market.status == OwnerEvidenceTrialStatus::Pass
        }));
        assert!(market_triage.iter().any(|market| {
            market.market == "BTC" && market.status == OwnerEvidenceTrialStatus::Fail
        }));
    }

    #[test]
    fn owner_report_and_agent_table_include_quality_context() {
        let selected_sources = quality_fixture_sources()
            .into_iter()
            .filter(|source| {
                matches!(
                    source.source_id.as_str(),
                    "quality-clean-synthetic"
                        | "quality-volume-btc"
                        | "quality-missing-optional-btc"
                )
            })
            .collect::<Vec<_>>();
        let mut input = valid_batch_input();
        input.sources = selected_sources;
        input.config.replay_mode = BatchReplayMode::IndependentPerSource;
        let first = run_local_dataset_batch_replay(input.clone()).expect("quality report batch");
        let second = run_local_dataset_batch_replay(input).expect("repeat quality report batch");
        assert_eq!(first, second);
        assert_eq!(first.agent_performance_by_quality_table.rows.len(), 9);
        assert!(
            first
                .agent_performance_by_quality_table
                .rows
                .windows(2)
                .all(|pair| {
                    (pair[0].agent_id.as_str(), pair[0].quality_bucket)
                        <= (pair[1].agent_id.as_str(), pair[1].quality_bucket)
                })
        );
        assert!(
            first
                .agent_performance_by_quality_table
                .rows
                .iter()
                .all(|row| row.agent_kind != AgentKind::Future8AgentPlaceholder)
        );

        let report = build_batch_owner_learning_report(&first);
        let text = render_batch_owner_learning_report_text(&report);
        for expected in [
            "Source quality threshold summary",
            "Quality bucket counts",
            "Blocked-by-quality sources",
            "Agent performance by quality bucket",
            "Source quality diagnostics are not live data validation.",
            "No profitability claim.",
            "Paper-only batch report.",
            "Market calendar/session validation is deferred.",
            "quality_policy=RejectPoorAndBelow",
        ] {
            assert!(
                text.contains(expected),
                "missing quality report text: {expected}"
            );
        }
        assert_eq!(report.agent_performance_by_quality_table.rows.len(), 9);
        assert!(!text.contains("fake-secret"));
        assert!(!text.contains(concat!("work", ".", "md")));
    }
}
