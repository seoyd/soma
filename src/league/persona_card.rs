use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::backtest::{
    AttributionRecord, BarrierHit, Candle, CandleSeries, CounterfactualRole, OutcomeRecord,
    Timeframe, TripleBarrierOutcome, TripleBarrierResult,
};
use crate::chair::{ChairConfig, ChairEngine};
use crate::core::{
    ChairInput, ChairOutput, InvestorVote, MarketSnapshot, PaperOrder, PaperOrderStatus,
    PersonaTier, ReasonCode, Regime, RiskDecision, RiskDecisionKind, RiskSnapshot, SignalOutput,
    Stance, stable_hash_string, stable_reason_codes,
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
