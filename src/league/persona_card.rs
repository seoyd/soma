use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::backtest::{
    AttributionRecord, BarrierHit, CounterfactualRole, OutcomeRecord, TripleBarrierOutcome,
    TripleBarrierResult,
};
use crate::chair::{ChairConfig, ChairEngine};
use crate::core::{
    ChairInput, ChairOutput, InvestorVote, MarketSnapshot, PaperOrder, PaperOrderStatus,
    PersonaTier, ReasonCode, RiskDecision, RiskDecisionKind, RiskSnapshot, SignalOutput, Stance,
    stable_hash_string, stable_reason_codes,
};
use crate::owner::{
    OwnerInput, OwnerTradeRequestReview, owner_rejection_explanation,
    review_owner_trade_request,
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
            vote.voice_power =
                (state.voice_state.voice_power * vote.conviction).clamp(0.0, 1.0);
        } else {
            vote.stance = Stance::Abstain;
            vote.voice_power = 0.0;
            vote.veto = false;
            vote.reason_codes.push(ReasonCode::AgentAbstained);
            if state.status == AgentStatus::Cooldown || state.voice_state.cooldown_bars > 0 {
                vote.reason_codes
                    .push(ReasonCode::CooldownAgentUnavailable);
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
    let trade_proposal = chair.build_trade_proposal(
        &input.market_snapshot,
        &input.signal_input,
        &chair_output,
    );
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
        if context
            .fill_evidence
            .as_ref()
            .is_none_or(|evidence| {
                evidence.filled_at_timestamp_ms > context.finalized_at_timestamp_ms
            })
        {
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
                doctrine_violation: context
                    .doctrine_violation_agents
                    .contains(&state.agent_id),
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
    if episode_ids.iter().any(|episode_id| episode_id.trim().is_empty()) {
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
                episode.input.market_snapshot.symbol,
                episode.input.market_snapshot.timestamp_ms
            )
        })
        .collect::<Vec<_>>();
    decision_ids.sort();
    if decision_ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(PaperLearningChainError::DuplicateDecisionId);
    }
    if input.episodes.windows(2).any(|pair| {
        pair[0].input.market_snapshot.timestamp_ms
            >= pair[1].input.market_snapshot.timestamp_ms
    }) {
        return Err(PaperLearningChainError::NonMonotonicEpisodeTime);
    }
    if input.episodes.windows(2).any(|pair| {
        pair[0]
            .input
            .paper_context
            .as_ref()
            .is_some_and(|context| {
                context.outcome_finalized
                    && context.finalized_at_timestamp_ms
                        >= pair[1].input.market_snapshot.timestamp_ms
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
    let mut reason_codes = vec![ReasonCode::DeterministicPath, ReasonCode::PaperExecutionOnly];

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
        accumulate_attribution(
            &mut attribution_summary,
            &input_states,
            &result,
        );
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
    if current_states.iter().any(|state| {
        match version_journal.latest_for_agent(&state.agent_id) {
            Some(snapshot) => snapshot.version_id != state.version.version_id,
            None => initial_states
                .iter()
                .find(|initial| initial.agent_id == state.agent_id)
                .is_none_or(|initial| initial.version.version_id != state.version.version_id),
        }
    }) {
        return Err(PaperLearningChainError::VersionFinalMismatch);
    }
    let any_live_mutation_detected = episode_results.iter().any(|episode| {
        episode.result.original_agent_states != episode.input_states
            || (episode.result.paper_outcome.is_none()
                && episode.result.updated_agent_states
                    != episode.result.original_agent_states)
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
            || episode.result.paper_outcome.as_ref().is_some_and(|outcome| {
                outcome.denied_by_risk && outcome.executed
            })
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
    let mut reason_codes = vec![ReasonCode::DeterministicPath, ReasonCode::PaperExecutionOnly];

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
        pair[0].input.market_snapshot.timestamp_ms
            >= pair[1].input.market_snapshot.timestamp_ms
    }) {
        return Err(PaperReplayError::NonMonotonicEpisodeTime);
    }
    if input.episode_inputs.windows(2).any(|pair| {
        pair[0]
            .input
            .paper_context
            .as_ref()
            .is_some_and(|context| {
                context.outcome_finalized
                    && context.finalized_at_timestamp_ms
                        >= pair[1].input.market_snapshot.timestamp_ms
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
        summary.reason_codes.extend(attribution.reason_codes.iter().cloned());
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
            summary.reason_codes.push(ReasonCode::AttributionUnavailable);
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
        summary.losing_selected_count +=
            (selected && feedback.realized_net_return < 0.0) as u64;
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
            summary.reason_codes.push(ReasonCode::AttributionUnavailable);
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
            summary
                .reason_codes
                .push(ReasonCode::AttributionAbstained);
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

fn validate_three_agent_set(
    states: &[CanonicalAgentState],
) -> Result<(), PaperLearningLoopError> {
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
                || matches!(state.status, AgentStatus::SandboxOnly | AgentStatus::Disabled)
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
        !context.outcome_finalized
            || context.finalized_at_timestamp_ms >= market.timestamp_ms
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
    chair_output: &ChairOutput,
    market: &MarketSnapshot,
    signal: &SignalOutput,
    market_name: &str,
    decision_id: &str,
) -> Vec<AgentProposal> {
    votes
        .iter()
        .map(|vote| {
            let mut reason_codes = vote.reason_codes.clone();
            if chair_output.lead_speaker == vote.persona_id {
                reason_codes.push(ReasonCode::AgentSelectedForDecision);
            } else if chair_output
                .selected_speakers
                .contains(&vote.persona_id)
            {
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
                horizon: horizon_from_bars(signal.horizon_bars),
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
    let denied_by_risk =
        !executed && risk_decision.kind != RiskDecisionKind::ApprovePaper
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
            synthetic_barrier_result(
                return_pct,
                market.price,
                context.max_adverse_excursion_pct,
            )
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
    let attribution_records =
        build_loop_attribution(votes, chair_output, denied_by_risk, no_trade);
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

fn paper_fill_evidence_matches(
    order: &PaperOrder,
    evidence: Option<&PaperFillEvidence>,
) -> bool {
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
            let selected_for_decision =
                chair_output.selected_speakers.contains(&vote.persona_id);
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
                    | CounterfactualRole::RiskVetoAligned => {
                        vote.voice_power * vote.conviction
                    }
                    CounterfactualRole::OpposedFinalDecision
                    | CounterfactualRole::RiskVetoOpposed => {
                        -(vote.voice_power * vote.conviction)
                    }
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
        CounterfactualRole::RiskVetoAligned => feedback
            .reason_codes
            .push(ReasonCode::AgentRiskVetoAligned),
        CounterfactualRole::RiskVetoOpposed => feedback
            .reason_codes
            .push(ReasonCode::AgentRiskVetoOpposed),
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
    let performance_return = if feedback.outcome_kind
        == AgentFeedbackOutcomeKind::ExecutedPaperTrade
    {
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
    let loss_penalty = (performance_return.min(0.0).abs() * loss_multiplier)
        .clamp(0.0, loss_penalty_cap);
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
    let payload =
        serde_json::to_string(feedback).unwrap_or_else(|_| format!("{feedback:?}"));
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
        ChairTierAction::Demote => demote_one_tier(next.tier),
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
    if proposal.symbol != paper_outcome.symbol || proposal.horizon != paper_outcome.horizon {
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
        return Err(feedback_error(
            ReasonCode::FeedbackProposalOutcomeMismatch,
        ));
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
            Some(
                CounterfactualRole::SupportedFinalDecision
                    | CounterfactualRole::RiskVetoAligned
            )
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
            CounterfactualRole::OpposedFinalDecision
            | CounterfactualRole::ForcedContrarian => {
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
    let realized_net_return =
        if outcome_kind == AgentFeedbackOutcomeKind::ExecutedPaperTrade {
            attributed_return
        } else {
            0.0
        };
    let counterfactual_net_return =
        (outcome_kind != AgentFeedbackOutcomeKind::ExecutedPaperTrade
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
        assert!(result.paper_order.as_ref().is_some_and(|order| order.paper_only));
        assert!(result.paper_outcome.as_ref().is_some_and(|outcome| outcome.executed));
        assert_eq!(selected_updated.memory_summary.wins, 1);
        assert!(selected_reward.reward_delta > selected_reward.penalty_delta);
        assert!(
            selected_updated.voice_state.voice_power
                > selected_original.voice_state.voice_power
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
        let result =
            run_3_agent_paper_learning_loop(input).expect("NoTrade avoided loss loop");
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
        assert!(result.paper_outcome.as_ref().is_some_and(|outcome| outcome.no_trade));
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

        let result =
            run_3_agent_paper_learning_loop(input).expect("NoTrade missed gain loop");
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
        assert!(
            abstained
                .reason_codes
                .contains(&ReasonCode::AgentAbstained)
        );
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
        assert!(
            no_trade.missed_gain_penalty > 0.0
                && !no_trade.no_trade_correct
        );
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
        assert!(result.paper_outcome.as_ref().is_some_and(|outcome| {
            outcome.denied_by_risk && !outcome.executed
        }));
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
        let second = run_3_agent_paper_learning_loop(input.clone())
            .expect("second deterministic loop");
        assert_eq!(first, second);

        let mut pending = input;
        pending
            .paper_context
            .as_mut()
            .expect("pending context")
            .outcome_finalized = false;
        let pending =
            run_3_agent_paper_learning_loop(pending).expect("pending paper outcome loop");
        assert!(pending.paper_outcome.is_none());
        assert!(pending.feedback_records.is_empty());
        assert!(pending.reward_penalties.is_empty());
        assert!(pending.version_snapshots.is_empty());
        assert!(pending.sandbox_candidates.is_empty());
        assert_eq!(
            pending.updated_agent_states,
            pending.original_agent_states
        );

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
        input.market_snapshot.timestamp_ms =
            input.market_snapshot.timestamp_ms.saturating_add(timestamp_offset);
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
        let first =
            run_3_agent_paper_learning_chain(input.clone()).expect("first learning chain");
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
        assert!(first.final_states.iter().zip(first.initial_states.iter()).all(
            |(final_state, initial)| {
                final_state.doctrine == initial.doctrine
                    && final_state.mutable_policy == initial.mutable_policy
            }
        ));
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
            episodes: vec![learning_episode(
                "episode-owner-pressure",
                owner_input,
                21,
            )],
            chain_config: PaperLearningChainConfig::default(),
        })
        .expect("owner pressure chain");
        let episode = &result.episode_results[0].result;
        let owner_review = episode
            .owner_explanation
            .as_ref()
            .expect("owner rejection");

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
        let pending_context = pending
            .paper_context
            .as_mut()
            .expect("pending context");
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
        duplicate_decision.episodes[1]
            .input
            .market_snapshot
            .symbol = duplicate_symbol;
        assert_eq!(
            run_3_agent_paper_learning_chain(duplicate_decision),
            Err(PaperLearningChainError::DuplicateDecisionId)
        );

        let mut reversed_time = improving_learning_chain_input();
        let later_timestamp = reversed_time.episodes[1]
            .input
            .market_snapshot
            .timestamp_ms;
        reversed_time.episodes[0]
            .input
            .market_snapshot
            .timestamp_ms = later_timestamp.saturating_add(10);
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
        assert!(
            first
                .final_states
                .iter()
                .all(|state| first
                    .version_journal
                    .latest_for_agent(&state.agent_id)
                    .is_some_and(|snapshot| snapshot.version_id == state.version.version_id))
        );
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
        assert!(first.final_states.iter().all(|state| {
            !matches!(
                state.status,
                AgentStatus::Cooldown | AgentStatus::Quarantined
            )
        }));
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
            initial_agent_states,
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
            initial_agent_states,
            episode_inputs: vec![learning_episode(
                "owner-cooldown-bypass",
                episode_input,
                71,
            )],
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
}
