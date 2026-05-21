use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::league::sprint98_committee_owned_core::{
    CommitteeConsensusState, CommitteeProposalAction, EighteenInvestorCandidateRegistry,
    EighteenInvestorCommitteeRosterPlan, EntryTimingWindow, InvestorArchetypeIngestionConfig,
    InvestorArchetypeSourceConfidenceEntry, InvestorArchetypeSourceConfidenceReport,
    InvestorConfidenceGrade, MemberStyleConfidenceWeightPolicy, PaperOnlyCommitteeDecisionKind,
    PaperOnlyRosterExpansionGate, SafetyCoveragePreservationReportV17,
    SafetyCoveragePreservationReportV17Status, Sprint101InvestorArchetypeIngestionBundle,
    Sprint101InvestorArchetypeIngestionRunner, WorkspaceAcceptanceTruthClosureStatus,
    WorkspaceAcceptanceTruthImport,
};
use crate::model::WorkspaceAcceptanceTruthGateStatus;

fn render_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|err| err.to_string())
}

fn local_only(path: &str) -> bool {
    !path.contains("://")
}

fn write_json_file<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    fs::write(path, render_json(value)?).map_err(|err| err.to_string())
}

fn deferred_reason_codes(extra: &[ReasonCode]) -> Vec<ReasonCode> {
    let mut codes = vec![
        ReasonCode::CommitteeV1Built,
        ReasonCode::CommitteeV1RunnerBuilt,
        ReasonCode::ChairV0Built,
        ReasonCode::OwnerCannotBypassRiskGovernor,
        ReasonCode::NoTradeDefault,
        ReasonCode::MambaRuntimeDeferred,
        ReasonCode::GatedDeltaNetRuntimeDeferred,
        ReasonCode::ControlTowerUiReadinessBuilt,
    ];
    codes.extend_from_slice(extra);
    codes
}

fn load_first_json<T: for<'de> Deserialize<'de>>(
    paths: Option<&Vec<String>>,
) -> Result<Option<T>, String> {
    match paths {
        Some(paths) => {
            for path in paths {
                let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
                if let Ok(value) = serde_json::from_str::<T>(&text) {
                    return Ok(Some(value));
                }
            }
            Ok(None)
        }
        None => Ok(None),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PaperRotationMarketCoverage {
    KoreanEquity,
    USEquity,
    Crypto,
    MultiAsset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PaperRotationRegimeCoverage {
    TrendBreakout,
    RangeBound,
    HighVolatility,
    DrawdownRisk,
    MacroShift,
    CryptoCycle,
    InsufficientEvidence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperRotationScenario {
    pub scenario_id: String,
    pub market: PaperRotationMarketCoverage,
    pub regime: PaperRotationRegimeCoverage,
    pub thesis: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperRotationScenarioPackStatus {
    ScenarioPackReady,
    ScenarioPackReadyWithWarnings,
    ScenarioPackNeedsMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperRotationScenarioPack {
    pub pack_id: String,
    pub scenarios: Vec<PaperRotationScenario>,
    pub scenario_count: usize,
    pub market_coverage: Vec<PaperRotationMarketCoverage>,
    pub regime_coverage: Vec<PaperRotationRegimeCoverage>,
    pub pack_status: PaperRotationScenarioPackStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperRotationMarketContext {
    pub scenario_id: String,
    pub source_boundary_ref: String,
    pub no_lookahead_proof_ref: String,
    pub risk_refs: Vec<String>,
    pub regime_refs: Vec<String>,
    pub counterfactual_refs: Vec<String>,
    pub paper_only: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperRotationMarketContextSetStatus {
    MarketContextSetReady,
    MarketContextSetReadyWithWarnings,
    MarketContextSetNeedsMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperRotationMarketContextSet {
    pub context_set_id: String,
    pub contexts: Vec<PaperRotationMarketContext>,
    pub context_count: usize,
    pub contexts_with_source_boundary: usize,
    pub contexts_with_no_lookahead_proof: usize,
    pub contexts_with_risk_refs: usize,
    pub contexts_with_regime_refs: usize,
    pub contexts_with_counterfactual_refs: usize,
    pub context_status: PaperRotationMarketContextSetStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchetypeGroupRotationPlanStatus {
    RotationPlanReady,
    RotationPlanReadyWithWarnings,
    RotationPlanNeedsMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchetypeGroupRotationPlan {
    pub plan_id: String,
    pub scenario_to_group_routes: BTreeMap<String, Vec<String>>,
    pub short_term_swing_assignments: Vec<String>,
    pub long_term_equity_assignments: Vec<String>,
    pub crypto_assignments: Vec<String>,
    pub common_risk_assignments: Vec<String>,
    pub cross_group_debate_assignments: Vec<String>,
    pub rotation_order: Vec<String>,
    pub plan_status: ArchetypeGroupRotationPlanStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchetypeMemberSelectionStatus {
    MemberSelectionReady,
    MemberSelectionReadyWithWarnings,
    MemberSelectionNeedsReview,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchetypeMemberSelectionReport {
    pub report_id: String,
    pub selected_members: Vec<String>,
    pub skipped_members: Vec<String>,
    pub watchlist_members: Vec<String>,
    pub diagnostic_members: Vec<String>,
    pub inactive_members: Vec<String>,
    pub selection_by_group: BTreeMap<String, Vec<String>>,
    pub selection_by_confidence: BTreeMap<String, Vec<String>>,
    pub selection_status: ArchetypeMemberSelectionStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LowerConfidenceHardeningAction {
    AddPublishedSource,
    AddOfficialProfile,
    AddPrimaryInterview,
    AddExchangeOrProtocolEvidence,
    DownWeightCommunityAnecdote,
    KeepDiagnosticOnly,
    NoAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LowerConfidenceEvidenceHardeningPlanStatus {
    LowerConfidenceHardeningPlanReady,
    LowerConfidenceHardeningPlanReadyWithWarnings,
    NeedMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LowerConfidenceEvidenceHardeningPlan {
    pub plan_id: String,
    pub target_candidates: Vec<String>,
    pub evidence_gaps: BTreeMap<String, Vec<String>>,
    pub recommended_actions: BTreeMap<String, Vec<LowerConfidenceHardeningAction>>,
    pub plan_status: LowerConfidenceEvidenceHardeningPlanStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LowerConfidenceEvidenceHardeningReportStatus {
    LowerConfidenceEvidenceImproved,
    LowerConfidenceEvidenceImprovedWithWarnings,
    LowerConfidenceEvidenceStillNeedsReview,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LowerConfidenceEvidenceHardeningReport {
    pub report_id: String,
    pub candidate_count: usize,
    pub improved_candidates: Vec<String>,
    pub still_warning_candidates: Vec<String>,
    pub unchanged_candidates: Vec<String>,
    pub added_evidence_refs: Vec<String>,
    pub downweighted_items: Vec<String>,
    pub report_status: LowerConfidenceEvidenceHardeningReportStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeakSourceCandidateReview {
    pub candidate_id: String,
    pub weak_source_items: Vec<String>,
    pub community_anecdote_items: Vec<String>,
    pub unsupported_claim_items: Vec<String>,
    pub actions_taken: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeakSourceCandidateReviewReportStatus {
    WeakSourceReviewReady,
    WeakSourceReviewReadyWithWarnings,
    WeakSourceStillNeedsReview,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeakSourceCandidateReviewReport {
    pub report_id: String,
    pub candidate_reviews: Vec<WeakSourceCandidateReview>,
    pub weak_source_warning_count: usize,
    pub community_anecdote_count: usize,
    pub unsupported_claim_count: usize,
    pub action_taken_count: usize,
    pub review_status: WeakSourceCandidateReviewReportStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WonyottiEvidenceHardeningStatus {
    WonyottiEvidenceImproved,
    WonyottiEvidenceStillWarningBacked,
    WonyottiKeptDiagnostic,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WonyottiEvidenceHardeningReport {
    pub report_id: String,
    pub current_confidence_grade: InvestorConfidenceGrade,
    pub evidence_refs_added: Vec<String>,
    pub community_anecdote_items_downweighted: Vec<String>,
    pub crypto_cycle_scope_preserved: bool,
    pub leverage_claims_guarded: bool,
    pub exact_return_claims_blocked: bool,
    pub report_status: WonyottiEvidenceHardeningStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LarryWilliamsEvidenceHardeningStatus {
    LarryWilliamsEvidenceImproved,
    LarryWilliamsEvidenceStillWarningBacked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LarryWilliamsEvidenceHardeningReport {
    pub report_id: String,
    pub current_confidence_grade: InvestorConfidenceGrade,
    pub evidence_refs_added: Vec<String>,
    pub published_material_refs: Vec<String>,
    pub statistical_seasonality_scope_preserved: bool,
    pub exact_numeric_rule_claims_downweighted: bool,
    pub report_status: LarryWilliamsEvidenceHardeningStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArthurHayesEvidenceHardeningStatus {
    ArthurHayesEvidenceImproved,
    ArthurHayesEvidenceStillWarningBacked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArthurHayesEvidenceHardeningReport {
    pub report_id: String,
    pub current_confidence_grade: InvestorConfidenceGrade,
    pub evidence_refs_added: Vec<String>,
    pub public_essay_refs: Vec<String>,
    pub liquidity_derivatives_scope_preserved: bool,
    pub macro_crypto_narrative_downweighted_if_unverified: bool,
    pub leverage_risk_guard_present: bool,
    pub report_status: ArthurHayesEvidenceHardeningStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperOnlyProposalRecord {
    pub proposal_id: String,
    pub member_id: String,
    pub proposed_action: CommitteeProposalAction,
    pub confidence_weight_applied: f64,
    pub expected_return_proxy: Option<f64>,
    pub expected_risk_proxy: Option<f64>,
    pub invalidation_condition: String,
    pub wait_condition: String,
    pub evidence_refs: Vec<String>,
    pub entry_timing_ref: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperOnlyMemberProposalRunStatus {
    ProposalRunReady,
    ProposalRunReadyWithWarnings,
    ProposalRunNeedsMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperOnlyMemberProposalRun {
    pub run_id: String,
    pub scenario_id: String,
    pub selected_members: Vec<String>,
    pub generated_proposals: Vec<PaperOnlyProposalRecord>,
    pub enter_long_count: usize,
    pub enter_short_count: usize,
    pub wait_count: usize,
    pub no_trade_count: usize,
    pub risk_deny_count: usize,
    pub request_more_evidence_count: usize,
    pub proposals_with_entry_timing: usize,
    pub proposals_with_risk_fields: usize,
    pub proposals_with_evidence_refs: usize,
    pub run_status: PaperOnlyMemberProposalRunStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperEntryTimingProposalRecord {
    pub timing_id: String,
    pub member_id: String,
    pub entry_window: EntryTimingWindow,
    pub rationale: String,
    pub risk_checks: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperOnlyEntryTimingProposalRunStatus {
    EntryTimingRunReady,
    EntryTimingRunReadyWithWarnings,
    EntryTimingRunNeedsMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperOnlyEntryTimingProposalRun {
    pub run_id: String,
    pub scenario_id: String,
    pub timing_proposals: Vec<PaperEntryTimingProposalRecord>,
    pub immediate_paper_only_count: usize,
    pub next_candle_count: usize,
    pub next_n_candles_count: usize,
    pub pullback_confirmation_count: usize,
    pub breakout_retest_count: usize,
    pub volatility_cooldown_count: usize,
    pub no_entry_count: usize,
    pub timing_status: PaperOnlyEntryTimingProposalRunStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupDebateTriggerKind {
    EntryTimingProposed,
    RiskDenyProposed,
    CrossGroupConflict,
    NeedMoreEvidence,
    ChairmanReview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupDebateTriggerStatus {
    DebateTriggered,
    DebateNotRequired,
    DebateBlockedBySafety,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupDebateTriggerReport {
    pub report_id: String,
    pub scenario_id: String,
    pub triggering_member_id: String,
    pub triggering_proposal_id: String,
    pub trigger_kind: GroupDebateTriggerKind,
    pub debate_required: bool,
    pub trigger_status: GroupDebateTriggerStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupDebateSessionStatus {
    DebateSessionReady,
    DebateSessionReadyWithWarnings,
    DebateNeedsMoreEvidence,
    DebateBlockedBySafety,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupDebateSessionReport {
    pub report_id: String,
    pub scenario_id: String,
    pub participating_groups: Vec<String>,
    pub participating_members: Vec<String>,
    pub debate_turn_count: usize,
    pub support_entry_count: usize,
    pub oppose_entry_count: usize,
    pub wait_count: usize,
    pub no_trade_count: usize,
    pub risk_deny_count: usize,
    pub request_more_evidence_count: usize,
    pub consensus_state: CommitteeConsensusState,
    pub debate_status: GroupDebateSessionStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossGroupConflictKind {
    ShortTermVsLongTerm,
    CryptoVsEquity,
    TrendVsValue,
    MacroVsLiquidity,
    OpportunityCostVsRiskVeto,
    OnchainVsPriceAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossGroupConflictResolution {
    RiskGovernorVeto,
    ChairmanSynthesis,
    NoTradeDefault,
    NeedMoreEvidence,
    PaperConditionalOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossGroupDebateConflictStatus {
    ConflictHandled,
    ConflictHandledWithWarnings,
    ConflictNeedsMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossGroupDebateConflictReport {
    pub report_id: String,
    pub scenario_id: String,
    pub conflicts_detected: usize,
    pub conflict_kinds: Vec<CrossGroupConflictKind>,
    pub conflict_resolution: CrossGroupConflictResolution,
    pub conflict_status: CrossGroupDebateConflictStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanDryRunRecommendation {
    WatchCandidate,
    PaperConditionalCandidate,
    NoTrade,
    RiskDeny,
    NeedMoreEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanSynthesisDryRunStatus {
    ChairmanSynthesisReady,
    ChairmanSynthesisReadyWithWarnings,
    ChairmanNeedsMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairmanSynthesisDryRunReport {
    pub report_id: String,
    pub scenario_id: String,
    pub debate_session_id: String,
    pub member_proposal_summary: String,
    pub conflict_summary: String,
    pub chairman_recommendation: ChairmanDryRunRecommendation,
    pub rulebook_version_used: String,
    pub style_weight_adjustments: BTreeMap<String, f64>,
    pub risk_governor_review_required: bool,
    pub synthesis_status: ChairmanSynthesisDryRunStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanStyleWeightAdjustmentAuditStatus {
    StyleWeightAuditPassed,
    StyleWeightAuditPassedWithWarnings,
    UnsafeAdjustmentBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairmanStyleWeightAdjustmentAudit {
    pub audit_id: String,
    pub scenario_id: String,
    pub previous_weights: BTreeMap<String, f64>,
    pub adjusted_weights: BTreeMap<String, f64>,
    pub adjustment_reason: String,
    pub source_confidence_constraints_applied: bool,
    pub low_confidence_caps_applied: bool,
    pub risk_governor_override_attempted: bool,
    pub audit_status: ChairmanStyleWeightAdjustmentAuditStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskGovernorPaperVetoResult {
    ApprovedPaperOnly,
    NoTrade,
    RiskDenied,
    NeedMoreEvidence,
    Cooldown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskGovernorPaperHandoffStatus {
    RiskGovernorHandoffReady,
    RiskGovernorHandoffReadyWithWarnings,
    RiskGovernorHandoffMissing,
    RiskBypassDetected,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskGovernorPaperHandoffReport {
    pub report_id: String,
    pub scenario_id: String,
    pub chairman_recommendation: ChairmanDryRunRecommendation,
    pub risk_checks: Vec<String>,
    pub veto_result: RiskGovernorPaperVetoResult,
    pub veto_reason: String,
    pub broker_execution_allowed: bool,
    pub live_execution_allowed: bool,
    pub handoff_status: RiskGovernorPaperHandoffStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperDecisionTraceV2Status {
    PaperTraceComplete,
    PaperTraceCompleteWithWarnings,
    PaperTraceIncomplete,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperDecisionTraceV2 {
    pub trace_id: String,
    pub scenario_id: String,
    pub market_context_ref: String,
    pub proposal_run_ref: String,
    pub debate_trigger_ref: String,
    pub debate_session_ref: String,
    pub chairman_synthesis_ref: String,
    pub risk_governor_handoff_ref: String,
    pub paper_decision: PaperOnlyCommitteeDecisionKind,
    pub trace_complete: bool,
    pub broker_execution_allowed: bool,
    pub live_execution_allowed: bool,
    pub trace_status: PaperDecisionTraceV2Status,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperDecisionReplayV2Status {
    PaperReplayReady,
    PaperReplayReadyWithWarnings,
    PaperReplayNeedsMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperDecisionReplayV2Report {
    pub report_id: String,
    pub replay_count: usize,
    pub watch_candidate_count: usize,
    pub paper_conditional_count: usize,
    pub no_trade_count: usize,
    pub risk_denied_count: usize,
    pub need_more_evidence_count: usize,
    pub broker_execution_allowed_count: usize,
    pub live_execution_allowed_count: usize,
    pub replay_status: PaperDecisionReplayV2Status,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalOutcomeExpectationTraceStatus {
    ExpectationTraceReady,
    ExpectationTraceReadyWithWarnings,
    ExpectationTraceNeedsEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProposalOutcomeExpectationTrace {
    pub trace_id: String,
    pub scenario_id: String,
    pub proposal_id: String,
    pub expected_return_proxy: f64,
    pub expected_risk_proxy: f64,
    pub expected_drawdown_proxy: f64,
    pub confidence: f64,
    pub source_confidence_weight_applied: f64,
    pub expectation_not_profit_claim: bool,
    pub trace_status: ProposalOutcomeExpectationTraceStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoTradeRiskDeniedCommitteeTraceStatus {
    NoTradeRiskDeniedTraceReady,
    NoTradeRiskDeniedTraceReadyWithWarnings,
    TraceNeedsMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoTradeRiskDeniedCommitteeTrace {
    pub trace_id: String,
    pub scenario_id: String,
    pub no_trade_member_votes: Vec<String>,
    pub risk_deny_member_votes: Vec<String>,
    pub risk_governor_no_trade: bool,
    pub risk_governor_risk_denied: bool,
    pub no_trade_reason_codes: Vec<String>,
    pub risk_denied_reason_codes: Vec<String>,
    pub trace_status: NoTradeRiskDeniedCommitteeTraceStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegimeRoutedCommitteeDryRunStatus {
    RegimeRoutedDryRunReady,
    RegimeRoutedDryRunReadyWithWarnings,
    RegimeRoutingNeedsMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegimeRoutedCommitteeDryRunReport {
    pub report_id: String,
    pub scenario_count: usize,
    pub routed_to_short_term_count: usize,
    pub routed_to_long_term_count: usize,
    pub routed_to_crypto_count: usize,
    pub routed_to_common_risk_count: usize,
    pub routed_to_counterfactual_count: usize,
    pub no_trade_routed_count: usize,
    pub risk_denied_routed_count: usize,
    pub routing_status: RegimeRoutedCommitteeDryRunStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultiExpertRotationCoverageStatus {
    MultiExpertRotationCoverageReady,
    MultiExpertRotationCoverageReadyWithWarnings,
    RotationCoverageNeedsReview,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiExpertRotationCoverageReport {
    pub report_id: String,
    pub total_members_available: usize,
    pub total_members_selected: usize,
    pub selected_short_term_count: usize,
    pub selected_long_term_count: usize,
    pub selected_crypto_count: usize,
    pub selected_common_risk_count: usize,
    pub unselected_members: Vec<String>,
    pub diagnostic_members_excluded: Vec<String>,
    pub coverage_status: MultiExpertRotationCoverageStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperRosterExpansionUsageStatus {
    PaperRosterExpansionUsedSafely,
    PaperRosterExpansionUsedWithWarnings,
    PaperRosterExpansionBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperRosterExpansionUsageReport {
    pub report_id: String,
    pub paper_expansion_allowed: bool,
    pub live_expansion_allowed: bool,
    pub active_members_used: Vec<String>,
    pub watchlist_members_used: Vec<String>,
    pub diagnostic_members_used: Vec<String>,
    pub inactive_members_used: Vec<String>,
    pub activation_violation_count: usize,
    pub usage_status: PaperRosterExpansionUsageStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EighteenArchetypeActivationSafetyStatus {
    EighteenActivationSafetyPreserved,
    EighteenActivationSafetyPreservedWithWarnings,
    EighteenLiveActivationViolation,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EighteenArchetypeActivationSafetyReport {
    pub report_id: String,
    pub eighteen_live_activation_forbidden: bool,
    pub live_activation_attempt_count: usize,
    pub paper_only_activation_count: usize,
    pub diagnostic_only_activation_count: usize,
    pub watchlist_only_count: usize,
    pub safety_status: EighteenArchetypeActivationSafetyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAcceptanceTruthClosurePlanV3 {
    pub plan_id: String,
    pub previous_truth_status: WorkspaceAcceptanceTruthGateStatus,
    pub current_truth_status: WorkspaceAcceptanceTruthGateStatus,
    pub can_claim_full_acceptance: bool,
    pub no_run_gate_status: String,
    pub full_workspace_gate_status: String,
    pub recommended_actions: Vec<String>,
    pub closure_status: WorkspaceAcceptanceTruthClosureStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAcceptanceAttemptV18 {
    pub attempt_id: String,
    pub command_no_run: String,
    pub command_full: String,
    pub no_run_started: bool,
    pub no_run_finished: bool,
    pub no_run_passed: Option<bool>,
    pub full_started: bool,
    pub full_finished: bool,
    pub full_passed: Option<bool>,
    pub timeout_ms: Option<u64>,
    pub can_claim_full_acceptance: bool,
    pub attempt_status: WorkspaceAcceptanceTruthGateStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyCoveragePreservationReportV18 {
    pub report_id: String,
    pub live_trading_guard_present: bool,
    pub broker_guard_present: bool,
    pub order_guard_present: bool,
    pub account_guard_present: bool,
    pub runtime_llm_guard_present: bool,
    pub mamba_runtime_guard_present: bool,
    pub gated_runtime_guard_present: bool,
    pub model_training_guard_present: bool,
    pub rust_neural_training_guard_present: bool,
    pub python_training_dependency_guard_present: bool,
    pub secret_guard_present: bool,
    pub no_lookahead_guard_present: bool,
    pub source_boundary_guard_present: bool,
    pub browser_execution_guard_present: bool,
    pub ui_order_control_guard_present: bool,
    pub investor_impersonation_guard_present: bool,
    pub unverified_claim_filter_present: bool,
    pub do_not_learn_guard_present: bool,
    pub eighteen_live_activation_forbidden: bool,
    pub paper_roster_only_guard_present: bool,
    pub chairman_risk_bypass_guard_present: bool,
    pub paper_rotation_not_order_execution_guard_present: bool,
    pub safety_status: SafetyCoveragePreservationReportV17Status,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerPaperRotationPanel {
    pub panel_id: String,
    pub rotation_status: String,
    pub scenario_summary: String,
    pub member_selection_summary: String,
    pub lower_confidence_evidence_summary: String,
    pub proposal_run_summary: String,
    pub entry_timing_summary: String,
    pub debate_summary: String,
    pub conflict_summary: String,
    pub chairman_synthesis_summary: String,
    pub risk_governor_handoff_summary: String,
    pub paper_decision_trace_summary: String,
    pub roster_usage_summary: String,
    pub runtime_deferred_summary: String,
    pub workspace_truth_summary: String,
    pub safety_summary: String,
    pub next_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint102PaperRotationStorageReport {
    pub report_id: String,
    pub output_dir: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint102PaperRotationBundle {
    pub paper_rotation_scenario_pack: PaperRotationScenarioPack,
    pub paper_rotation_market_context_set: PaperRotationMarketContextSet,
    pub archetype_group_rotation_plan: ArchetypeGroupRotationPlan,
    pub archetype_member_selection_report: ArchetypeMemberSelectionReport,
    pub lower_confidence_evidence_hardening_plan: LowerConfidenceEvidenceHardeningPlan,
    pub lower_confidence_evidence_hardening_report: LowerConfidenceEvidenceHardeningReport,
    pub weak_source_candidate_review_report: WeakSourceCandidateReviewReport,
    pub wonyotti_evidence_hardening_report: WonyottiEvidenceHardeningReport,
    pub larry_williams_evidence_hardening_report: LarryWilliamsEvidenceHardeningReport,
    pub arthur_hayes_evidence_hardening_report: ArthurHayesEvidenceHardeningReport,
    pub paper_only_member_proposal_run: PaperOnlyMemberProposalRun,
    pub paper_only_entry_timing_proposal_run: PaperOnlyEntryTimingProposalRun,
    pub group_debate_trigger_report: GroupDebateTriggerReport,
    pub group_debate_session_report: GroupDebateSessionReport,
    pub cross_group_debate_conflict_report: CrossGroupDebateConflictReport,
    pub chairman_synthesis_dry_run_report: ChairmanSynthesisDryRunReport,
    pub chairman_style_weight_adjustment_audit: ChairmanStyleWeightAdjustmentAudit,
    pub risk_governor_paper_handoff_report: RiskGovernorPaperHandoffReport,
    pub paper_decision_trace_v2: PaperDecisionTraceV2,
    pub paper_decision_replay_v2_report: PaperDecisionReplayV2Report,
    pub proposal_outcome_expectation_trace: ProposalOutcomeExpectationTrace,
    pub no_trade_risk_denied_committee_trace: NoTradeRiskDeniedCommitteeTrace,
    pub regime_routed_committee_dry_run_report: RegimeRoutedCommitteeDryRunReport,
    pub multi_expert_rotation_coverage_report: MultiExpertRotationCoverageReport,
    pub paper_roster_expansion_usage_report: PaperRosterExpansionUsageReport,
    pub eighteen_archetype_activation_safety_report: EighteenArchetypeActivationSafetyReport,
    pub workspace_acceptance_truth_closure_plan_v3: WorkspaceAcceptanceTruthClosurePlanV3,
    pub workspace_acceptance_attempt_v18: WorkspaceAcceptanceAttemptV18,
    pub safety_coverage_preservation_report_v18: SafetyCoveragePreservationReportV18,
    pub control_tower_paper_rotation_panel: ControlTowerPaperRotationPanel,
    pub storage_report: Sprint102PaperRotationStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EighteenArchetypePaperRotationConfig {
    pub rotation_id: String,
    #[serde(default)]
    pub sprint101_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub investor_archetype_card_paths: Option<Vec<String>>,
    #[serde(default)]
    pub roster_plan_paths: Option<Vec<String>>,
    #[serde(default)]
    pub regime_routing_paths: Option<Vec<String>>,
    #[serde(default)]
    pub style_conflict_matrix_paths: Option<Vec<String>>,
    #[serde(default)]
    pub paper_readiness_paths: Option<Vec<String>>,
    #[serde(default)]
    pub workspace_acceptance_truth_paths: Option<Vec<String>>,
    pub output_root: String,
    pub max_scenarios: usize,
    pub max_member_proposals: usize,
    pub max_debate_sessions: usize,
    pub max_debate_turns: usize,
    #[serde(default = "default_true")]
    pub require_paper_only: bool,
    #[serde(default = "default_true")]
    pub require_no_live_activation: bool,
    #[serde(default = "default_true")]
    pub require_source_confidence_weighting: bool,
    #[serde(default = "default_true")]
    pub require_lower_confidence_review: bool,
    #[serde(default = "default_true")]
    pub require_risk_governor_handoff: bool,
    #[serde(default = "default_true")]
    pub require_chairman_synthesis: bool,
    #[serde(default = "default_true")]
    pub require_workspace_truth_separation: bool,
    #[serde(default = "default_true")]
    pub preserve_safety_guards: bool,
    #[serde(default = "default_true")]
    pub preserve_runtime_deferred: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

fn default_true() -> bool {
    true
}

impl Default for EighteenArchetypePaperRotationConfig {
    fn default() -> Self {
        Self {
            rotation_id: "sprint102-paper-rotation".to_string(),
            sprint101_bundle_paths: Some(vec![
                "examples/sprint102_data/sprint101_summary.json".to_string(),
            ]),
            investor_archetype_card_paths: Some(vec![
                "examples/sprint101_data/investor_material_sample.md".to_string(),
            ]),
            roster_plan_paths: Some(vec![
                "examples/sprint102_data/sprint101_summary.json".to_string(),
            ]),
            regime_routing_paths: Some(vec![
                "examples/sprint102_data/sprint101_summary.json".to_string(),
            ]),
            style_conflict_matrix_paths: Some(vec![
                "examples/sprint102_data/sprint101_summary.json".to_string(),
            ]),
            paper_readiness_paths: Some(vec![
                "examples/sprint101_data/sprint100_summary.json".to_string(),
            ]),
            workspace_acceptance_truth_paths: Some(vec![
                "examples/sprint102_data/sprint101_summary.json".to_string(),
            ]),
            output_root: "target/soma_sprint102_paper_rotation".to_string(),
            max_scenarios: 7,
            max_member_proposals: 7,
            max_debate_sessions: 4,
            max_debate_turns: 12,
            require_paper_only: true,
            require_no_live_activation: true,
            require_source_confidence_weighting: true,
            require_lower_confidence_review: true,
            require_risk_governor_handoff: true,
            require_chairman_synthesis: true,
            require_workspace_truth_separation: true,
            preserve_safety_guards: true,
            preserve_runtime_deferred: true,
            reason_codes: deferred_reason_codes(&[]),
        }
    }
}

impl EighteenArchetypePaperRotationConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.rotation_id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.rotation_id.trim().is_empty() {
            return Err("sprint102 rotation_id must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err("sprint102 paper rotation config paths must be local".to_string());
        }
        for paths in [
            &self.sprint101_bundle_paths,
            &self.investor_archetype_card_paths,
            &self.roster_plan_paths,
            &self.regime_routing_paths,
            &self.style_conflict_matrix_paths,
            &self.paper_readiness_paths,
            &self.workspace_acceptance_truth_paths,
        ] {
            if let Some(paths) = paths
                && paths.iter().any(|path| !local_only(path))
            {
                return Err("sprint102 paper rotation config paths must be local".to_string());
            }
        }
        if !(1..=64).contains(&self.max_scenarios)
            || !(1..=128).contains(&self.max_member_proposals)
            || !(1..=32).contains(&self.max_debate_sessions)
            || !(1..=256).contains(&self.max_debate_turns)
        {
            return Err("sprint102 paper rotation max bounds exceeded".to_string());
        }
        if self.require_paper_only && !self.preserve_runtime_deferred {
            return Err(
                "sprint102 paper-only rotation requires runtime deferred preservation".to_string(),
            );
        }
        Ok(())
    }
}

fn sprint101_entry<'a>(
    report: &'a InvestorArchetypeSourceConfidenceReport,
    candidate_id: &str,
) -> &'a InvestorArchetypeSourceConfidenceEntry {
    report
        .entries
        .iter()
        .find(|entry| entry.candidate_id == candidate_id)
        .unwrap_or_else(|| panic!("missing sprint101 entry for {candidate_id}"))
}

fn load_sprint101_bundle(
    config: &EighteenArchetypePaperRotationConfig,
) -> Result<Sprint101InvestorArchetypeIngestionBundle, String> {
    if let Some(bundle) = load_first_json::<Sprint101InvestorArchetypeIngestionBundle>(
        config.sprint101_bundle_paths.as_ref(),
    )? {
        return Ok(bundle);
    }
    let mut sprint101_config = InvestorArchetypeIngestionConfig::default();
    sprint101_config.ingestion_id = format!("{}-sprint101-base", config.rotation_id);
    sprint101_config.output_root = config.output_root.clone();
    Sprint101InvestorArchetypeIngestionRunner::default()
        .run_sprint101_investor_archetype_ingestion(&sprint101_config)
}

fn load_workspace_truth(
    config: &EighteenArchetypePaperRotationConfig,
    sprint101: &Sprint101InvestorArchetypeIngestionBundle,
) -> Result<WorkspaceAcceptanceTruthImport, String> {
    Ok(load_first_json::<WorkspaceAcceptanceTruthImport>(
        config.workspace_acceptance_truth_paths.as_ref(),
    )?
    .unwrap_or_else(|| sprint101.workspace_acceptance_truth_import.clone()))
}

fn build_scenario_pack(config: &EighteenArchetypePaperRotationConfig) -> PaperRotationScenarioPack {
    let mut scenarios = vec![
        PaperRotationScenario {
            scenario_id: "korean-equity-trend-breakout".to_string(),
            market: PaperRotationMarketCoverage::KoreanEquity,
            regime: PaperRotationRegimeCoverage::TrendBreakout,
            thesis: "Korean equity breakout with liquidity support".to_string(),
            evidence_refs: vec![
                "official-korean-equity-daily".to_string(),
                "breakout-liquidity-check".to_string(),
            ],
        },
        PaperRotationScenario {
            scenario_id: "us-equity-range-bound".to_string(),
            market: PaperRotationMarketCoverage::USEquity,
            regime: PaperRotationRegimeCoverage::RangeBound,
            thesis: "US equity range-bound paper review with valuation cross-check".to_string(),
            evidence_refs: vec![
                "official-us-equity-daily".to_string(),
                "range-bound-regime-proof".to_string(),
            ],
        },
        PaperRotationScenario {
            scenario_id: "crypto-high-volatility".to_string(),
            market: PaperRotationMarketCoverage::Crypto,
            regime: PaperRotationRegimeCoverage::HighVolatility,
            thesis: "Crypto leverage stress scenario".to_string(),
            evidence_refs: vec![
                "crypto-liquidity-check".to_string(),
                "funding-stress-snapshot".to_string(),
            ],
        },
        PaperRotationScenario {
            scenario_id: "multi-asset-drawdown-risk".to_string(),
            market: PaperRotationMarketCoverage::MultiAsset,
            regime: PaperRotationRegimeCoverage::DrawdownRisk,
            thesis: "Cross-asset drawdown regime with quality review".to_string(),
            evidence_refs: vec![
                "drawdown-risk-bundle".to_string(),
                "quality-balance-sheet-review".to_string(),
            ],
        },
        PaperRotationScenario {
            scenario_id: "macro-shift-cross-asset".to_string(),
            market: PaperRotationMarketCoverage::MultiAsset,
            regime: PaperRotationRegimeCoverage::MacroShift,
            thesis: "Macro shift regime requiring chairman synthesis".to_string(),
            evidence_refs: vec![
                "macro-shift-pack".to_string(),
                "cross-asset-confirmation".to_string(),
            ],
        },
        PaperRotationScenario {
            scenario_id: "crypto-cycle-liquidity".to_string(),
            market: PaperRotationMarketCoverage::Crypto,
            regime: PaperRotationRegimeCoverage::CryptoCycle,
            thesis: "Crypto cycle opportunity under liquidity guard".to_string(),
            evidence_refs: vec![
                "onchain-cycle-pack".to_string(),
                "exchange-liquidity-pack".to_string(),
            ],
        },
        PaperRotationScenario {
            scenario_id: "insufficient-evidence-halt".to_string(),
            market: PaperRotationMarketCoverage::USEquity,
            regime: PaperRotationRegimeCoverage::InsufficientEvidence,
            thesis: "Insufficient evidence halt scenario".to_string(),
            evidence_refs: vec![
                "insufficient-evidence-flag".to_string(),
                "counterfactual-review-request".to_string(),
            ],
        },
    ];
    scenarios.truncate(config.max_scenarios);
    let market_coverage = scenarios
        .iter()
        .map(|scenario| scenario.market)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let regime_coverage = scenarios
        .iter()
        .map(|scenario| scenario.regime)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let pack_status = if scenarios.len() < 4 {
        PaperRotationScenarioPackStatus::ScenarioPackNeedsMoreEvidence
    } else if market_coverage.len() < 4 || regime_coverage.len() < 7 {
        PaperRotationScenarioPackStatus::ScenarioPackReadyWithWarnings
    } else {
        PaperRotationScenarioPackStatus::ScenarioPackReady
    };
    PaperRotationScenarioPack {
        pack_id: format!("{}-scenario-pack", config.rotation_id),
        scenario_count: scenarios.len(),
        scenarios,
        market_coverage,
        regime_coverage,
        pack_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_market_context_set(pack: &PaperRotationScenarioPack) -> PaperRotationMarketContextSet {
    let contexts = pack
        .scenarios
        .iter()
        .map(|scenario| PaperRotationMarketContext {
            scenario_id: scenario.scenario_id.clone(),
            source_boundary_ref: format!("{}-source-boundary-proof", scenario.scenario_id),
            no_lookahead_proof_ref: format!("{}-no-lookahead-proof", scenario.scenario_id),
            risk_refs: vec![format!("{}-risk-ref", scenario.scenario_id)],
            regime_refs: vec![format!("{}-regime-ref", scenario.scenario_id)],
            counterfactual_refs: vec![format!("{}-counterfactual-ref", scenario.scenario_id)],
            paper_only: true,
        })
        .collect::<Vec<_>>();
    let count = contexts.len();
    PaperRotationMarketContextSet {
        context_set_id: "paper-rotation-market-context-set".to_string(),
        contexts,
        context_count: count,
        contexts_with_source_boundary: count,
        contexts_with_no_lookahead_proof: count,
        contexts_with_risk_refs: count,
        contexts_with_regime_refs: count,
        contexts_with_counterfactual_refs: count,
        context_status: if count == 0 {
            PaperRotationMarketContextSetStatus::MarketContextSetNeedsMoreEvidence
        } else {
            PaperRotationMarketContextSetStatus::MarketContextSetReady
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_group_rotation_plan(pack: &PaperRotationScenarioPack) -> ArchetypeGroupRotationPlan {
    let mut scenario_to_group_routes = BTreeMap::new();
    let mut short_term_swing_assignments = Vec::new();
    let mut long_term_equity_assignments = Vec::new();
    let mut crypto_assignments = Vec::new();
    let mut common_risk_assignments = Vec::new();
    let mut cross_group_debate_assignments = Vec::new();
    let mut rotation_order = Vec::new();
    for scenario in &pack.scenarios {
        let routes = match scenario.regime {
            PaperRotationRegimeCoverage::TrendBreakout => {
                vec!["ShortTermSwing".to_string(), "CommonRisk".to_string()]
            }
            PaperRotationRegimeCoverage::RangeBound => vec![
                "ShortTermSwing".to_string(),
                "LongTermEquity".to_string(),
                "CommonRisk".to_string(),
            ],
            PaperRotationRegimeCoverage::HighVolatility => {
                vec!["Crypto".to_string(), "CommonRisk".to_string()]
            }
            PaperRotationRegimeCoverage::DrawdownRisk => {
                vec!["LongTermEquity".to_string(), "CommonRisk".to_string()]
            }
            PaperRotationRegimeCoverage::MacroShift => vec![
                "ShortTermSwing".to_string(),
                "LongTermEquity".to_string(),
                "CommonRisk".to_string(),
            ],
            PaperRotationRegimeCoverage::CryptoCycle => {
                vec!["Crypto".to_string(), "CommonRisk".to_string()]
            }
            PaperRotationRegimeCoverage::InsufficientEvidence => {
                vec!["CounterfactualReview".to_string(), "CommonRisk".to_string()]
            }
        };
        if routes.iter().any(|group| group == "ShortTermSwing") {
            short_term_swing_assignments.push(scenario.scenario_id.clone());
        }
        if routes.iter().any(|group| group == "LongTermEquity") {
            long_term_equity_assignments.push(scenario.scenario_id.clone());
        }
        if routes.iter().any(|group| group == "Crypto") {
            crypto_assignments.push(scenario.scenario_id.clone());
        }
        if routes.iter().any(|group| group == "CommonRisk") {
            common_risk_assignments.push(scenario.scenario_id.clone());
        }
        if routes.len() > 2
            || matches!(
                scenario.regime,
                PaperRotationRegimeCoverage::InsufficientEvidence
            )
        {
            cross_group_debate_assignments.push(scenario.scenario_id.clone());
        }
        rotation_order.push(scenario.scenario_id.clone());
        scenario_to_group_routes.insert(scenario.scenario_id.clone(), routes);
    }
    ArchetypeGroupRotationPlan {
        plan_id: "archetype-group-rotation-plan".to_string(),
        scenario_to_group_routes,
        short_term_swing_assignments,
        long_term_equity_assignments,
        crypto_assignments,
        common_risk_assignments,
        cross_group_debate_assignments,
        rotation_order,
        plan_status: ArchetypeGroupRotationPlanStatus::RotationPlanReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_member_selection(
    registry: &EighteenInvestorCandidateRegistry,
    roster: &EighteenInvestorCommitteeRosterPlan,
) -> ArchetypeMemberSelectionReport {
    let selection_by_group = BTreeMap::from([
        (
            "ShortTermSwing".to_string(),
            vec![
                "LarryWilliamsStatisticalSeasonality".to_string(),
                "RaschkeProcessManager".to_string(),
            ],
        ),
        (
            "LongTermEquity".to_string(),
            vec![
                "GrahamMarginOfSafety".to_string(),
                "MarksCycleRiskPremium".to_string(),
            ],
        ),
        (
            "Crypto".to_string(),
            vec![
                "BurniskeTokenValuation".to_string(),
                "HayesLiquidityDerivatives".to_string(),
                "SaylorTreasury".to_string(),
            ],
        ),
        (
            "CommonRisk".to_string(),
            vec!["CommonRiskManager".to_string()],
        ),
    ]);
    let mut selected_members = selection_by_group
        .values()
        .flat_map(|members| members.iter().cloned())
        .collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    selected_members.retain(|member| seen.insert(member.clone()));
    selected_members.retain(|member| !roster.diagnostic_members.contains(member));
    let skipped_members = registry
        .candidates
        .iter()
        .map(|candidate| candidate.normalized_archetype_name.clone())
        .filter(|member| !selected_members.contains(member))
        .collect::<Vec<_>>();
    let watchlist_members = selected_members
        .iter()
        .filter(|member| roster.watchlist_members.contains(*member))
        .cloned()
        .collect::<Vec<_>>();
    let mut selection_by_confidence: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for member in &selected_members {
        let bucket = registry
            .candidates
            .iter()
            .find(|candidate| candidate.normalized_archetype_name == *member)
            .map(|candidate| format!("{:?}", candidate.confidence_grade))
            .unwrap_or_else(|| "Sentinel".to_string());
        selection_by_confidence
            .entry(bucket)
            .or_default()
            .push(member.clone());
    }
    ArchetypeMemberSelectionReport {
        report_id: "archetype-member-selection-report".to_string(),
        selected_members,
        skipped_members,
        watchlist_members,
        diagnostic_members: roster.diagnostic_members.clone(),
        inactive_members: roster.inactive_members.clone(),
        selection_by_group,
        selection_by_confidence,
        selection_status: ArchetypeMemberSelectionStatus::MemberSelectionReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_hardening_plan() -> LowerConfidenceEvidenceHardeningPlan {
    LowerConfidenceEvidenceHardeningPlan {
        plan_id: "lower-confidence-evidence-hardening-plan".to_string(),
        target_candidates: vec![
            "arthur-hayes".to_string(),
            "larry-williams".to_string(),
            "wonyotti".to_string(),
        ],
        evidence_gaps: BTreeMap::from([
            (
                "arthur-hayes".to_string(),
                vec![
                    "add public essay references".to_string(),
                    "down-weight narrative-only leverage commentary".to_string(),
                ],
            ),
            (
                "larry-williams".to_string(),
                vec![
                    "add more published seasonal study references".to_string(),
                    "down-weight anecdotal calendar lore".to_string(),
                ],
            ),
            (
                "wonyotti".to_string(),
                vec![
                    "add exchange or protocol evidence".to_string(),
                    "keep leverage and exact-return claims blocked".to_string(),
                ],
            ),
        ]),
        recommended_actions: BTreeMap::from([
            (
                "arthur-hayes".to_string(),
                vec![
                    LowerConfidenceHardeningAction::AddPrimaryInterview,
                    LowerConfidenceHardeningAction::AddExchangeOrProtocolEvidence,
                    LowerConfidenceHardeningAction::DownWeightCommunityAnecdote,
                ],
            ),
            (
                "larry-williams".to_string(),
                vec![
                    LowerConfidenceHardeningAction::AddPublishedSource,
                    LowerConfidenceHardeningAction::DownWeightCommunityAnecdote,
                ],
            ),
            (
                "wonyotti".to_string(),
                vec![
                    LowerConfidenceHardeningAction::AddOfficialProfile,
                    LowerConfidenceHardeningAction::AddExchangeOrProtocolEvidence,
                    LowerConfidenceHardeningAction::DownWeightCommunityAnecdote,
                ],
            ),
        ]),
        plan_status: LowerConfidenceEvidenceHardeningPlanStatus::LowerConfidenceHardeningPlanReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_weak_source_review(
    confidence: &InvestorArchetypeSourceConfidenceReport,
) -> WeakSourceCandidateReviewReport {
    let candidate_reviews = vec![
        WeakSourceCandidateReview {
            candidate_id: "arthur-hayes".to_string(),
            weak_source_items: sprint101_entry(confidence, "arthur-hayes")
                .weak_source_items
                .clone(),
            community_anecdote_items: Vec::new(),
            unsupported_claim_items: vec!["best crypto investor claim".to_string()],
            actions_taken: vec![
                "added public essay references".to_string(),
                "kept leverage commentary lower-authority".to_string(),
            ],
        },
        WeakSourceCandidateReview {
            candidate_id: "larry-williams".to_string(),
            weak_source_items: sprint101_entry(confidence, "larry-williams")
                .weak_source_items
                .clone(),
            community_anecdote_items: Vec::new(),
            unsupported_claim_items: vec!["best market timer claim".to_string()],
            actions_taken: vec![
                "added published seasonal study references".to_string(),
                "kept seasonal rules below multi-signal confirmation".to_string(),
            ],
        },
        WeakSourceCandidateReview {
            candidate_id: "wonyotti".to_string(),
            weak_source_items: sprint101_entry(confidence, "wonyotti")
                .weak_source_items
                .clone(),
            community_anecdote_items: sprint101_entry(confidence, "wonyotti")
                .community_anecdote_items
                .clone(),
            unsupported_claim_items: vec!["best crypto investor claim".to_string()],
            actions_taken: vec![
                "added public thesis archive refs".to_string(),
                "down-weighted community chat recap".to_string(),
            ],
        },
    ];
    WeakSourceCandidateReviewReport {
        report_id: "weak-source-candidate-review-report".to_string(),
        weak_source_warning_count: candidate_reviews
            .iter()
            .map(|review| review.weak_source_items.len())
            .sum(),
        community_anecdote_count: candidate_reviews
            .iter()
            .map(|review| review.community_anecdote_items.len())
            .sum(),
        unsupported_claim_count: candidate_reviews
            .iter()
            .map(|review| review.unsupported_claim_items.len())
            .sum(),
        action_taken_count: candidate_reviews
            .iter()
            .map(|review| review.actions_taken.len())
            .sum(),
        candidate_reviews,
        review_status: WeakSourceCandidateReviewReportStatus::WeakSourceReviewReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_wonyotti_report(
    confidence: &InvestorArchetypeSourceConfidenceReport,
) -> WonyottiEvidenceHardeningReport {
    let entry = sprint101_entry(confidence, "wonyotti");
    WonyottiEvidenceHardeningReport {
        report_id: "wonyotti-evidence-hardening-report".to_string(),
        current_confidence_grade: entry.confidence_grade,
        evidence_refs_added: vec![
            "wonyotti-public-thesis-archive".to_string(),
            "wonyotti-protocol-and-treasury-reference".to_string(),
        ],
        community_anecdote_items_downweighted: entry.community_anecdote_items.clone(),
        crypto_cycle_scope_preserved: true,
        leverage_claims_guarded: true,
        exact_return_claims_blocked: true,
        report_status: WonyottiEvidenceHardeningStatus::WonyottiEvidenceStillWarningBacked,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_larry_report(
    confidence: &InvestorArchetypeSourceConfidenceReport,
) -> LarryWilliamsEvidenceHardeningReport {
    let entry = sprint101_entry(confidence, "larry-williams");
    LarryWilliamsEvidenceHardeningReport {
        report_id: "larry-williams-evidence-hardening-report".to_string(),
        current_confidence_grade: entry.confidence_grade,
        evidence_refs_added: vec!["larry-published-seasonal-study".to_string()],
        published_material_refs: vec![
            "published seasonal study summary".to_string(),
            "archived statistics interview".to_string(),
        ],
        statistical_seasonality_scope_preserved: true,
        exact_numeric_rule_claims_downweighted: true,
        report_status:
            LarryWilliamsEvidenceHardeningStatus::LarryWilliamsEvidenceStillWarningBacked,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_arthur_report(
    confidence: &InvestorArchetypeSourceConfidenceReport,
) -> ArthurHayesEvidenceHardeningReport {
    let entry = sprint101_entry(confidence, "arthur-hayes");
    ArthurHayesEvidenceHardeningReport {
        report_id: "arthur-hayes-evidence-hardening-report".to_string(),
        current_confidence_grade: entry.confidence_grade,
        evidence_refs_added: vec![
            "arthur-hayes-public-essay-archive".to_string(),
            "arthur-hayes-derivatives-liquidity-interview".to_string(),
        ],
        public_essay_refs: vec![
            "public thesis archive".to_string(),
            "public protocol or treasury material".to_string(),
        ],
        liquidity_derivatives_scope_preserved: true,
        macro_crypto_narrative_downweighted_if_unverified: true,
        leverage_risk_guard_present: true,
        report_status: ArthurHayesEvidenceHardeningStatus::ArthurHayesEvidenceStillWarningBacked,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_hardening_report(
    wonyotti: &WonyottiEvidenceHardeningReport,
    larry: &LarryWilliamsEvidenceHardeningReport,
    arthur: &ArthurHayesEvidenceHardeningReport,
) -> LowerConfidenceEvidenceHardeningReport {
    LowerConfidenceEvidenceHardeningReport {
        report_id: "lower-confidence-evidence-hardening-report".to_string(),
        candidate_count: 3,
        improved_candidates: vec![
            "wonyotti".to_string(),
            "larry-williams".to_string(),
            "arthur-hayes".to_string(),
        ],
        still_warning_candidates: vec![
            "wonyotti".to_string(),
            "larry-williams".to_string(),
            "arthur-hayes".to_string(),
        ],
        unchanged_candidates: Vec::new(),
        added_evidence_refs: wonyotti
            .evidence_refs_added
            .iter()
            .chain(larry.evidence_refs_added.iter())
            .chain(arthur.evidence_refs_added.iter())
            .cloned()
            .collect(),
        downweighted_items: wonyotti
            .community_anecdote_items_downweighted
            .iter()
            .cloned()
            .chain(std::iter::once("larry-williams: old anecdotal forum summaries".to_string()))
            .chain(std::iter::once(
                "arthur-hayes: high-narrative macro commentary requires down-weighting".to_string(),
            ))
            .collect(),
        report_status: LowerConfidenceEvidenceHardeningReportStatus::LowerConfidenceEvidenceImprovedWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn weight(policy: &MemberStyleConfidenceWeightPolicy, member_id: &str) -> f64 {
    policy
        .candidate_weight_overrides
        .get(member_id)
        .copied()
        .unwrap_or(1.0)
}

fn build_proposal_run(
    selection: &ArchetypeMemberSelectionReport,
    policy: &MemberStyleConfidenceWeightPolicy,
    config: &EighteenArchetypePaperRotationConfig,
) -> PaperOnlyMemberProposalRun {
    let mut generated_proposals = vec![
        PaperOnlyProposalRecord {
            proposal_id: "proposal-burniske-enter-long".to_string(),
            member_id: "BurniskeTokenValuation".to_string(),
            proposed_action: CommitteeProposalAction::EnterLong,
            confidence_weight_applied: weight(policy, "BurniskeTokenValuation"),
            expected_return_proxy: Some(0.62),
            expected_risk_proxy: Some(0.41),
            invalidation_condition: "liquidity and on-chain confirmation breaks".to_string(),
            wait_condition: "wait if exchange liquidity decays".to_string(),
            evidence_refs: vec![
                "burniske-cycle-pack".to_string(),
                "onchain-confirmation".to_string(),
            ],
            entry_timing_ref: Some("timing-burniske-immediate".to_string()),
        },
        PaperOnlyProposalRecord {
            proposal_id: "proposal-hayes-risk-deny".to_string(),
            member_id: "HayesLiquidityDerivatives".to_string(),
            proposed_action: CommitteeProposalAction::RiskDeny,
            confidence_weight_applied: weight(policy, "HayesLiquidityDerivatives"),
            expected_return_proxy: None,
            expected_risk_proxy: Some(0.72),
            invalidation_condition: "derivatives stress eases only after leverage unwind"
                .to_string(),
            wait_condition: "wait until leverage and basis normalize".to_string(),
            evidence_refs: vec!["arthur-hayes-public-essay-archive".to_string()],
            entry_timing_ref: None,
        },
        PaperOnlyProposalRecord {
            proposal_id: "proposal-buffett-wait".to_string(),
            member_id: "BuffettQualityMoat".to_string(),
            proposed_action: CommitteeProposalAction::Wait,
            confidence_weight_applied: weight(policy, "BuffettQualityMoat"),
            expected_return_proxy: Some(0.38),
            expected_risk_proxy: Some(0.24),
            invalidation_condition: "durability and downside case remain incomplete".to_string(),
            wait_condition: "wait for balance-sheet and valuation alignment".to_string(),
            evidence_refs: vec!["buffett-durability-check".to_string()],
            entry_timing_ref: None,
        },
        PaperOnlyProposalRecord {
            proposal_id: "proposal-larry-enter-short".to_string(),
            member_id: "LarryWilliamsStatisticalSeasonality".to_string(),
            proposed_action: CommitteeProposalAction::EnterShort,
            confidence_weight_applied: weight(policy, "LarryWilliamsStatisticalSeasonality"),
            expected_return_proxy: Some(0.44),
            expected_risk_proxy: Some(0.47),
            invalidation_condition: "seasonality loses confirmation against trend".to_string(),
            wait_condition: "wait for current-cycle fit confirmation".to_string(),
            evidence_refs: vec!["larry-published-seasonal-study".to_string()],
            entry_timing_ref: Some("timing-larry-next-candle".to_string()),
        },
        PaperOnlyProposalRecord {
            proposal_id: "proposal-raschke-enter-long".to_string(),
            member_id: "RaschkeProcessManager".to_string(),
            proposed_action: CommitteeProposalAction::EnterLong,
            confidence_weight_applied: weight(policy, "RaschkeProcessManager"),
            expected_return_proxy: Some(0.53),
            expected_risk_proxy: Some(0.36),
            invalidation_condition: "pattern thesis breaks or pullback fails".to_string(),
            wait_condition: "wait if setup classification is unclear".to_string(),
            evidence_refs: vec!["raschke-pattern-checklist".to_string()],
            entry_timing_ref: Some("timing-raschke-pullback".to_string()),
        },
        PaperOnlyProposalRecord {
            proposal_id: "proposal-saylor-no-trade".to_string(),
            member_id: "SaylorTreasury".to_string(),
            proposed_action: CommitteeProposalAction::NoTrade,
            confidence_weight_applied: weight(policy, "SaylorTreasury"),
            expected_return_proxy: None,
            expected_risk_proxy: Some(0.51),
            invalidation_condition: "treasury framing alone cannot justify entry".to_string(),
            wait_condition: "wait for protocol and liquidity confirmation".to_string(),
            evidence_refs: vec!["saylor-treasury-thesis".to_string()],
            entry_timing_ref: None,
        },
        PaperOnlyProposalRecord {
            proposal_id: "proposal-common-risk-request-more-evidence".to_string(),
            member_id: "CommonRiskManager".to_string(),
            proposed_action: CommitteeProposalAction::RequestMoreEvidence,
            confidence_weight_applied: 1.0,
            expected_return_proxy: None,
            expected_risk_proxy: Some(0.67),
            invalidation_condition: "risk aggregation remains unresolved".to_string(),
            wait_condition: "wait for lower-confidence review closure".to_string(),
            evidence_refs: vec!["common-risk-aggregation-check".to_string()],
            entry_timing_ref: Some("timing-common-risk-no-entry".to_string()),
        },
    ];
    generated_proposals.truncate(config.max_member_proposals);
    let count = |action| {
        generated_proposals
            .iter()
            .filter(|proposal| proposal.proposed_action == action)
            .count()
    };
    PaperOnlyMemberProposalRun {
        run_id: "paper-only-member-proposal-run".to_string(),
        scenario_id: "crypto-cycle-liquidity".to_string(),
        selected_members: selection.selected_members.clone(),
        enter_long_count: count(CommitteeProposalAction::EnterLong),
        enter_short_count: count(CommitteeProposalAction::EnterShort),
        wait_count: count(CommitteeProposalAction::Wait),
        no_trade_count: count(CommitteeProposalAction::NoTrade),
        risk_deny_count: count(CommitteeProposalAction::RiskDeny),
        request_more_evidence_count: count(CommitteeProposalAction::RequestMoreEvidence),
        proposals_with_entry_timing: generated_proposals
            .iter()
            .filter(|proposal| proposal.entry_timing_ref.is_some())
            .count(),
        proposals_with_risk_fields: generated_proposals
            .iter()
            .filter(|proposal| proposal.expected_risk_proxy.is_some())
            .count(),
        proposals_with_evidence_refs: generated_proposals
            .iter()
            .filter(|proposal| !proposal.evidence_refs.is_empty())
            .count(),
        generated_proposals,
        run_status: PaperOnlyMemberProposalRunStatus::ProposalRunReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_entry_timing_run(
    config: &EighteenArchetypePaperRotationConfig,
) -> PaperOnlyEntryTimingProposalRun {
    let mut timing_proposals = vec![
        PaperEntryTimingProposalRecord {
            timing_id: "timing-burniske-immediate".to_string(),
            member_id: "BurniskeTokenValuation".to_string(),
            entry_window: EntryTimingWindow::ImmediatePaperOnly,
            rationale: "paper-only immediate rotation when liquidity confirmation is present"
                .to_string(),
            risk_checks: vec!["confirm CommonRisk budget".to_string()],
        },
        PaperEntryTimingProposalRecord {
            timing_id: "timing-larry-next-candle".to_string(),
            member_id: "LarryWilliamsStatisticalSeasonality".to_string(),
            entry_window: EntryTimingWindow::NextCandle,
            rationale: "wait one candle for seasonal confirmation".to_string(),
            risk_checks: vec!["confirm no lookahead leakage".to_string()],
        },
        PaperEntryTimingProposalRecord {
            timing_id: "timing-buffett-next-n-candles".to_string(),
            member_id: "BuffettQualityMoat".to_string(),
            entry_window: EntryTimingWindow::NextNCandles,
            rationale: "long-term review waits across several paper candles".to_string(),
            risk_checks: vec!["confirm balance-sheet refresh".to_string()],
        },
        PaperEntryTimingProposalRecord {
            timing_id: "timing-raschke-pullback".to_string(),
            member_id: "RaschkeProcessManager".to_string(),
            entry_window: EntryTimingWindow::PullbackConfirmation,
            rationale: "pattern process wants pullback confirmation".to_string(),
            risk_checks: vec!["confirm stop distance".to_string()],
        },
        PaperEntryTimingProposalRecord {
            timing_id: "timing-saylor-breakout-retest".to_string(),
            member_id: "SaylorTreasury".to_string(),
            entry_window: EntryTimingWindow::BreakoutRetest,
            rationale: "treasury framing requires retest before paper observation".to_string(),
            risk_checks: vec!["confirm liquidity depth".to_string()],
        },
        PaperEntryTimingProposalRecord {
            timing_id: "timing-hayes-cooldown".to_string(),
            member_id: "HayesLiquidityDerivatives".to_string(),
            entry_window: EntryTimingWindow::VolatilityCooldown,
            rationale: "derivatives stress requires cooldown".to_string(),
            risk_checks: vec!["confirm leverage reset".to_string()],
        },
        PaperEntryTimingProposalRecord {
            timing_id: "timing-common-risk-no-entry".to_string(),
            member_id: "CommonRiskManager".to_string(),
            entry_window: EntryTimingWindow::NoEntry,
            rationale: "risk sentinel keeps paper-only no-entry until evidence hardening closes"
                .to_string(),
            risk_checks: vec!["confirm unresolved risk remains".to_string()],
        },
    ];
    timing_proposals.truncate(config.max_member_proposals);
    let count = |window| {
        timing_proposals
            .iter()
            .filter(|proposal| proposal.entry_window == window)
            .count()
    };
    PaperOnlyEntryTimingProposalRun {
        run_id: "paper-only-entry-timing-proposal-run".to_string(),
        scenario_id: "crypto-cycle-liquidity".to_string(),
        immediate_paper_only_count: count(EntryTimingWindow::ImmediatePaperOnly),
        next_candle_count: count(EntryTimingWindow::NextCandle),
        next_n_candles_count: count(EntryTimingWindow::NextNCandles),
        pullback_confirmation_count: count(EntryTimingWindow::PullbackConfirmation),
        breakout_retest_count: count(EntryTimingWindow::BreakoutRetest),
        volatility_cooldown_count: count(EntryTimingWindow::VolatilityCooldown),
        no_entry_count: count(EntryTimingWindow::NoEntry),
        timing_proposals,
        timing_status: PaperOnlyEntryTimingProposalRunStatus::EntryTimingRunReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_debate_trigger(conflict_exists: bool) -> GroupDebateTriggerReport {
    GroupDebateTriggerReport {
        report_id: "group-debate-trigger-report".to_string(),
        scenario_id: "crypto-cycle-liquidity".to_string(),
        triggering_member_id: "HayesLiquidityDerivatives".to_string(),
        triggering_proposal_id: "proposal-hayes-risk-deny".to_string(),
        trigger_kind: if conflict_exists {
            GroupDebateTriggerKind::CrossGroupConflict
        } else {
            GroupDebateTriggerKind::RiskDenyProposed
        },
        debate_required: true,
        trigger_status: GroupDebateTriggerStatus::DebateTriggered,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_debate_session(
    selection: &ArchetypeMemberSelectionReport,
    config: &EighteenArchetypePaperRotationConfig,
) -> GroupDebateSessionReport {
    GroupDebateSessionReport {
        report_id: "group-debate-session-report".to_string(),
        scenario_id: "crypto-cycle-liquidity".to_string(),
        participating_groups: vec![
            "ShortTermSwing".to_string(),
            "LongTermEquity".to_string(),
            "Crypto".to_string(),
            "CommonRisk".to_string(),
        ],
        participating_members: selection.selected_members.clone(),
        debate_turn_count: selection
            .selected_members
            .len()
            .min(config.max_debate_turns),
        support_entry_count: 2,
        oppose_entry_count: 1,
        wait_count: 1,
        no_trade_count: 1,
        risk_deny_count: 1,
        request_more_evidence_count: 1,
        consensus_state: CommitteeConsensusState::NeedMoreEvidence,
        debate_status: GroupDebateSessionStatus::DebateSessionReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_conflict_report() -> CrossGroupDebateConflictReport {
    CrossGroupDebateConflictReport {
        report_id: "cross-group-debate-conflict-report".to_string(),
        scenario_id: "crypto-cycle-liquidity".to_string(),
        conflicts_detected: 4,
        conflict_kinds: vec![
            CrossGroupConflictKind::CryptoVsEquity,
            CrossGroupConflictKind::TrendVsValue,
            CrossGroupConflictKind::MacroVsLiquidity,
            CrossGroupConflictKind::OpportunityCostVsRiskVeto,
        ],
        conflict_resolution: CrossGroupConflictResolution::NeedMoreEvidence,
        conflict_status: CrossGroupDebateConflictStatus::ConflictHandledWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_weight_audit(
    selection: &ArchetypeMemberSelectionReport,
    policy: &MemberStyleConfidenceWeightPolicy,
) -> ChairmanStyleWeightAdjustmentAudit {
    let mut previous_weights = BTreeMap::new();
    let mut adjusted_weights = BTreeMap::new();
    for member in &selection.selected_members {
        let previous = weight(policy, member);
        previous_weights.insert(member.clone(), previous);
        let adjusted = match member.as_str() {
            "HayesLiquidityDerivatives" => previous.min(policy.low_confidence_cap),
            "LarryWilliamsStatisticalSeasonality" => previous.min(policy.low_confidence_cap + 0.18),
            "BuffettQualityMoat" => (previous + 0.02).min(1.0),
            _ => previous,
        };
        adjusted_weights.insert(member.clone(), adjusted);
    }
    ChairmanStyleWeightAdjustmentAudit {
        audit_id: "chairman-style-weight-adjustment-audit".to_string(),
        scenario_id: "crypto-cycle-liquidity".to_string(),
        previous_weights,
        adjusted_weights,
        adjustment_reason:
            "paper-only synthesis reweights lower-confidence members without upgrading them"
                .to_string(),
        source_confidence_constraints_applied: true,
        low_confidence_caps_applied: true,
        risk_governor_override_attempted: false,
        audit_status: ChairmanStyleWeightAdjustmentAuditStatus::StyleWeightAuditPassedWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_chairman_report(
    proposal_run: &PaperOnlyMemberProposalRun,
    conflict: &CrossGroupDebateConflictReport,
    audit: &ChairmanStyleWeightAdjustmentAudit,
) -> ChairmanSynthesisDryRunReport {
    ChairmanSynthesisDryRunReport {
        report_id: "chairman-synthesis-dry-run-report".to_string(),
        scenario_id: proposal_run.scenario_id.clone(),
        debate_session_id: "group-debate-session-report".to_string(),
        member_proposal_summary: format!(
            "enter_long={} enter_short={} wait={} no_trade={} risk_deny={} request_more_evidence={}",
            proposal_run.enter_long_count,
            proposal_run.enter_short_count,
            proposal_run.wait_count,
            proposal_run.no_trade_count,
            proposal_run.risk_deny_count,
            proposal_run.request_more_evidence_count
        ),
        conflict_summary: format!(
            "conflicts_detected={} resolution={:?}",
            conflict.conflicts_detected, conflict.conflict_resolution
        ),
        chairman_recommendation: ChairmanDryRunRecommendation::NoTrade,
        rulebook_version_used: "chairman-paper-rulebook-v2".to_string(),
        style_weight_adjustments: audit.adjusted_weights.clone(),
        risk_governor_review_required: true,
        synthesis_status: ChairmanSynthesisDryRunStatus::ChairmanSynthesisReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_risk_handoff(chairman: &ChairmanSynthesisDryRunReport) -> RiskGovernorPaperHandoffReport {
    RiskGovernorPaperHandoffReport {
        report_id: "risk-governor-paper-handoff-report".to_string(),
        scenario_id: chairman.scenario_id.clone(),
        chairman_recommendation: chairman.chairman_recommendation,
        risk_checks: vec![
            "final veto still required".to_string(),
            "paper proposal is not order execution".to_string(),
            "lower-confidence warnings remain in force".to_string(),
        ],
        veto_result: RiskGovernorPaperVetoResult::NoTrade,
        veto_reason:
            "cross-group conflict and lower-confidence evidence still require a paper-only NoTrade default"
                .to_string(),
        broker_execution_allowed: false,
        live_execution_allowed: false,
        handoff_status: RiskGovernorPaperHandoffStatus::RiskGovernorHandoffReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_trace(
    context_set: &PaperRotationMarketContextSet,
    proposal_run: &PaperOnlyMemberProposalRun,
    trigger: &GroupDebateTriggerReport,
    debate: &GroupDebateSessionReport,
    chairman: &ChairmanSynthesisDryRunReport,
    risk: &RiskGovernorPaperHandoffReport,
) -> PaperDecisionTraceV2 {
    let trace_complete = !context_set.contexts.is_empty()
        && !proposal_run.generated_proposals.is_empty()
        && trigger.debate_required
        && debate.debate_turn_count > 0
        && chairman.risk_governor_review_required;
    PaperDecisionTraceV2 {
        trace_id: "paper-decision-trace-v2".to_string(),
        scenario_id: proposal_run.scenario_id.clone(),
        market_context_ref: context_set.context_set_id.clone(),
        proposal_run_ref: proposal_run.run_id.clone(),
        debate_trigger_ref: trigger.report_id.clone(),
        debate_session_ref: debate.report_id.clone(),
        chairman_synthesis_ref: chairman.report_id.clone(),
        risk_governor_handoff_ref: risk.report_id.clone(),
        paper_decision: PaperOnlyCommitteeDecisionKind::NoTrade,
        trace_complete,
        broker_execution_allowed: false,
        live_execution_allowed: false,
        trace_status: if trace_complete {
            PaperDecisionTraceV2Status::PaperTraceCompleteWithWarnings
        } else {
            PaperDecisionTraceV2Status::PaperTraceIncomplete
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_replay(trace: &PaperDecisionTraceV2) -> PaperDecisionReplayV2Report {
    let (
        watch_candidate_count,
        paper_conditional_count,
        no_trade_count,
        risk_denied_count,
        need_more_evidence_count,
    ) = match trace.paper_decision {
        PaperOnlyCommitteeDecisionKind::WatchCandidate => (1, 0, 0, 0, 0),
        PaperOnlyCommitteeDecisionKind::PaperApproved => (0, 1, 0, 0, 0),
        PaperOnlyCommitteeDecisionKind::NoTrade => (0, 0, 1, 0, 0),
        PaperOnlyCommitteeDecisionKind::RiskDenied => (0, 0, 0, 1, 0),
        PaperOnlyCommitteeDecisionKind::NeedMoreEvidence => (0, 0, 0, 0, 1),
        PaperOnlyCommitteeDecisionKind::PaperRejected => (0, 0, 1, 0, 0),
    };
    PaperDecisionReplayV2Report {
        report_id: "paper-decision-replay-v2-report".to_string(),
        replay_count: 1,
        watch_candidate_count,
        paper_conditional_count,
        no_trade_count,
        risk_denied_count,
        need_more_evidence_count,
        broker_execution_allowed_count: 0,
        live_execution_allowed_count: 0,
        replay_status: PaperDecisionReplayV2Status::PaperReplayReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_expectation_trace(
    proposal_run: &PaperOnlyMemberProposalRun,
) -> ProposalOutcomeExpectationTrace {
    let proposal = proposal_run
        .generated_proposals
        .iter()
        .find(|proposal| proposal.proposal_id == "proposal-burniske-enter-long")
        .unwrap_or(&proposal_run.generated_proposals[0]);
    ProposalOutcomeExpectationTrace {
        trace_id: "proposal-outcome-expectation-trace".to_string(),
        scenario_id: proposal_run.scenario_id.clone(),
        proposal_id: proposal.proposal_id.clone(),
        expected_return_proxy: proposal.expected_return_proxy.unwrap_or(0.0),
        expected_risk_proxy: proposal.expected_risk_proxy.unwrap_or(0.0),
        expected_drawdown_proxy: 0.33,
        confidence: proposal.confidence_weight_applied,
        source_confidence_weight_applied: proposal.confidence_weight_applied,
        expectation_not_profit_claim: true,
        trace_status: ProposalOutcomeExpectationTraceStatus::ExpectationTraceReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_no_trade_trace(
    selection: &ArchetypeMemberSelectionReport,
) -> NoTradeRiskDeniedCommitteeTrace {
    NoTradeRiskDeniedCommitteeTrace {
        trace_id: "no-trade-risk-denied-committee-trace".to_string(),
        scenario_id: "crypto-cycle-liquidity".to_string(),
        no_trade_member_votes: selection
            .selected_members
            .iter()
            .filter(|member| *member == "SaylorTreasury")
            .cloned()
            .collect(),
        risk_deny_member_votes: selection
            .selected_members
            .iter()
            .filter(|member| *member == "HayesLiquidityDerivatives")
            .cloned()
            .collect(),
        risk_governor_no_trade: true,
        risk_governor_risk_denied: false,
        no_trade_reason_codes: vec!["paper_only_no_trade_default".to_string()],
        risk_denied_reason_codes: vec!["derivatives_stress_guard".to_string()],
        trace_status:
            NoTradeRiskDeniedCommitteeTraceStatus::NoTradeRiskDeniedTraceReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_regime_report(plan: &ArchetypeGroupRotationPlan) -> RegimeRoutedCommitteeDryRunReport {
    RegimeRoutedCommitteeDryRunReport {
        report_id: "regime-routed-committee-dry-run-report".to_string(),
        scenario_count: plan.scenario_to_group_routes.len(),
        routed_to_short_term_count: plan.short_term_swing_assignments.len(),
        routed_to_long_term_count: plan.long_term_equity_assignments.len(),
        routed_to_crypto_count: plan.crypto_assignments.len(),
        routed_to_common_risk_count: plan.common_risk_assignments.len(),
        routed_to_counterfactual_count: plan
            .scenario_to_group_routes
            .values()
            .filter(|groups| groups.iter().any(|group| group == "CounterfactualReview"))
            .count(),
        no_trade_routed_count: plan
            .scenario_to_group_routes
            .keys()
            .filter(|scenario_id| scenario_id.contains("insufficient-evidence"))
            .count(),
        risk_denied_routed_count: plan
            .scenario_to_group_routes
            .keys()
            .filter(|scenario_id| scenario_id.contains("high-volatility"))
            .count(),
        routing_status: RegimeRoutedCommitteeDryRunStatus::RegimeRoutedDryRunReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_coverage(
    registry: &EighteenInvestorCandidateRegistry,
    selection: &ArchetypeMemberSelectionReport,
    roster: &EighteenInvestorCommitteeRosterPlan,
) -> MultiExpertRotationCoverageReport {
    let selected = selection
        .selected_members
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let all_members = registry
        .candidates
        .iter()
        .map(|candidate| candidate.normalized_archetype_name.clone())
        .chain(std::iter::once("CommonRiskManager".to_string()))
        .collect::<Vec<_>>();
    MultiExpertRotationCoverageReport {
        report_id: "multi-expert-rotation-coverage-report".to_string(),
        total_members_available: all_members.len(),
        total_members_selected: selection.selected_members.len(),
        selected_short_term_count: selection
            .selection_by_group
            .get("ShortTermSwing")
            .map_or(0, Vec::len),
        selected_long_term_count: selection
            .selection_by_group
            .get("LongTermEquity")
            .map_or(0, Vec::len),
        selected_crypto_count: selection
            .selection_by_group
            .get("Crypto")
            .map_or(0, Vec::len),
        selected_common_risk_count: selection
            .selection_by_group
            .get("CommonRisk")
            .map_or(0, Vec::len),
        unselected_members: all_members
            .into_iter()
            .filter(|member| !selected.contains(member))
            .collect(),
        diagnostic_members_excluded: roster.diagnostic_members.clone(),
        coverage_status:
            MultiExpertRotationCoverageStatus::MultiExpertRotationCoverageReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_roster_usage(
    roster: &EighteenInvestorCommitteeRosterPlan,
    selection: &ArchetypeMemberSelectionReport,
    gate: &PaperOnlyRosterExpansionGate,
) -> PaperRosterExpansionUsageReport {
    let active_members_used = selection
        .selected_members
        .iter()
        .filter(|member| roster.active_paper_members.contains(*member))
        .cloned()
        .collect();
    let watchlist_members_used = selection
        .selected_members
        .iter()
        .filter(|member| roster.watchlist_members.contains(*member))
        .cloned()
        .collect::<Vec<_>>();
    let diagnostic_members_used = selection
        .selected_members
        .iter()
        .filter(|member| roster.diagnostic_members.contains(*member))
        .cloned()
        .collect();
    let inactive_members_used = selection
        .selected_members
        .iter()
        .filter(|member| roster.inactive_members.contains(*member))
        .cloned()
        .collect();
    PaperRosterExpansionUsageReport {
        report_id: "paper-roster-expansion-usage-report".to_string(),
        paper_expansion_allowed: gate.paper_roster_expansion_allowed,
        live_expansion_allowed: false,
        active_members_used,
        watchlist_members_used: watchlist_members_used.clone(),
        diagnostic_members_used,
        inactive_members_used,
        activation_violation_count: 0,
        usage_status: if watchlist_members_used.is_empty() {
            PaperRosterExpansionUsageStatus::PaperRosterExpansionUsedSafely
        } else {
            PaperRosterExpansionUsageStatus::PaperRosterExpansionUsedWithWarnings
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_activation_safety(
    selection: &ArchetypeMemberSelectionReport,
    usage: &PaperRosterExpansionUsageReport,
) -> EighteenArchetypeActivationSafetyReport {
    EighteenArchetypeActivationSafetyReport {
        report_id: "eighteen-archetype-activation-safety-report".to_string(),
        eighteen_live_activation_forbidden: true,
        live_activation_attempt_count: 0,
        paper_only_activation_count: selection
            .selected_members
            .iter()
            .filter(|member| *member != "CommonRiskManager")
            .count(),
        diagnostic_only_activation_count: 0,
        watchlist_only_count: usage.watchlist_members_used.len(),
        safety_status: if usage.watchlist_members_used.is_empty() {
            EighteenArchetypeActivationSafetyStatus::EighteenActivationSafetyPreserved
        } else {
            EighteenArchetypeActivationSafetyStatus::EighteenActivationSafetyPreservedWithWarnings
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_attempt(
    workspace_truth: &WorkspaceAcceptanceTruthImport,
) -> WorkspaceAcceptanceAttemptV18 {
    WorkspaceAcceptanceAttemptV18 {
        attempt_id: "workspace-acceptance-attempt-v18".to_string(),
        command_no_run: "cargo test --workspace --no-run --quiet".to_string(),
        command_full: "cargo test --workspace --quiet".to_string(),
        no_run_started: false,
        no_run_finished: false,
        no_run_passed: None,
        full_started: false,
        full_finished: workspace_truth.full_workspace_finished,
        full_passed: workspace_truth.full_workspace_passed,
        timeout_ms: None,
        can_claim_full_acceptance: workspace_truth.can_claim_full_acceptance,
        attempt_status: if workspace_truth.full_workspace_finished {
            match workspace_truth.full_workspace_passed {
                Some(true) => WorkspaceAcceptanceTruthGateStatus::FullWorkspaceAccepted,
                Some(false) => WorkspaceAcceptanceTruthGateStatus::FullWorkspaceFailed,
                None => WorkspaceAcceptanceTruthGateStatus::FullWorkspaceStillBlocked,
            }
        } else {
            workspace_truth.truth_status
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_closure(
    workspace_truth: &WorkspaceAcceptanceTruthImport,
    attempt: &WorkspaceAcceptanceAttemptV18,
) -> WorkspaceAcceptanceTruthClosurePlanV3 {
    WorkspaceAcceptanceTruthClosurePlanV3 {
        plan_id: "workspace-acceptance-truth-closure-plan-v3".to_string(),
        previous_truth_status: workspace_truth.truth_status,
        current_truth_status: attempt.attempt_status,
        can_claim_full_acceptance: workspace_truth.can_claim_full_acceptance,
        no_run_gate_status: "NoRunGatePending".to_string(),
        full_workspace_gate_status: format!("{:?}", attempt.attempt_status),
        recommended_actions: vec![
            "RunRealNoRunWithLongerTimeout".to_string(),
            "RunRealFullWorkspaceWithLongerTimeout".to_string(),
            "KeepFocusedTestsSeparate".to_string(),
            "ConsiderNextestLocalDiagnostic".to_string(),
            "ConsiderSccacheLocalDiagnostic".to_string(),
            "DoNotClaimFullAcceptance".to_string(),
        ],
        closure_status: if workspace_truth.can_claim_full_acceptance {
            WorkspaceAcceptanceTruthClosureStatus::FullWorkspaceAlreadyAccepted
        } else {
            WorkspaceAcceptanceTruthClosureStatus::WorkspaceTruthStillOpen
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_safety_v18(
    previous: &SafetyCoveragePreservationReportV17,
) -> SafetyCoveragePreservationReportV18 {
    let safety_status = if previous.live_trading_guard_present
        && previous.broker_guard_present
        && previous.order_guard_present
        && previous.account_guard_present
        && previous.runtime_llm_guard_present
        && previous.mamba_runtime_guard_present
        && previous.gated_runtime_guard_present
        && previous.model_training_guard_present
        && previous.rust_neural_training_guard_present
        && previous.python_training_dependency_guard_present
        && previous.secret_guard_present
        && previous.no_lookahead_guard_present
        && previous.source_boundary_guard_present
        && previous.browser_execution_guard_present
        && previous.ui_order_control_guard_present
        && previous.investor_impersonation_guard_present
        && previous.unverified_claim_filter_present
        && previous.do_not_learn_guard_present
        && previous.eighteen_live_activation_forbidden
        && previous.paper_roster_only_guard_present
        && previous.chairman_risk_bypass_guard_present
    {
        SafetyCoveragePreservationReportV17Status::SafetyCoveragePreserved
    } else {
        SafetyCoveragePreservationReportV17Status::SafetyCoverageMissing
    };
    SafetyCoveragePreservationReportV18 {
        report_id: "safety-coverage-preservation-report-v18".to_string(),
        live_trading_guard_present: previous.live_trading_guard_present,
        broker_guard_present: previous.broker_guard_present,
        order_guard_present: previous.order_guard_present,
        account_guard_present: previous.account_guard_present,
        runtime_llm_guard_present: previous.runtime_llm_guard_present,
        mamba_runtime_guard_present: previous.mamba_runtime_guard_present,
        gated_runtime_guard_present: previous.gated_runtime_guard_present,
        model_training_guard_present: previous.model_training_guard_present,
        rust_neural_training_guard_present: previous.rust_neural_training_guard_present,
        python_training_dependency_guard_present: previous.python_training_dependency_guard_present,
        secret_guard_present: previous.secret_guard_present,
        no_lookahead_guard_present: previous.no_lookahead_guard_present,
        source_boundary_guard_present: previous.source_boundary_guard_present,
        browser_execution_guard_present: previous.browser_execution_guard_present,
        ui_order_control_guard_present: previous.ui_order_control_guard_present,
        investor_impersonation_guard_present: previous.investor_impersonation_guard_present,
        unverified_claim_filter_present: previous.unverified_claim_filter_present,
        do_not_learn_guard_present: previous.do_not_learn_guard_present,
        eighteen_live_activation_forbidden: previous.eighteen_live_activation_forbidden,
        paper_roster_only_guard_present: previous.paper_roster_only_guard_present,
        chairman_risk_bypass_guard_present: previous.chairman_risk_bypass_guard_present,
        paper_rotation_not_order_execution_guard_present: true,
        safety_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_panel(
    pack: &PaperRotationScenarioPack,
    selection: &ArchetypeMemberSelectionReport,
    hardening: &LowerConfidenceEvidenceHardeningReport,
    proposal_run: &PaperOnlyMemberProposalRun,
    timing_run: &PaperOnlyEntryTimingProposalRun,
    debate: &GroupDebateSessionReport,
    conflict: &CrossGroupDebateConflictReport,
    chairman: &ChairmanSynthesisDryRunReport,
    risk: &RiskGovernorPaperHandoffReport,
    trace: &PaperDecisionTraceV2,
    usage: &PaperRosterExpansionUsageReport,
    workspace_truth: &WorkspaceAcceptanceTruthImport,
    safety: &SafetyCoveragePreservationReportV18,
) -> ControlTowerPaperRotationPanel {
    ControlTowerPaperRotationPanel {
        panel_id: "control-tower-paper-rotation-panel".to_string(),
        rotation_status: format!("{:?}", pack.pack_status),
        scenario_summary: format!("scenario_count={} regimes={}", pack.scenario_count, pack.regime_coverage.len()),
        member_selection_summary: format!(
            "selected={} watchlist_used={} diagnostic_excluded={}",
            selection.selected_members.len(),
            selection.watchlist_members.len(),
            selection.diagnostic_members.len()
        ),
        lower_confidence_evidence_summary: format!(
            "status={:?} still_warning_candidates={}",
            hardening.report_status,
            hardening.still_warning_candidates.len()
        ),
        proposal_run_summary: format!(
            "enter_long={} enter_short={} wait={} no_trade={} risk_deny={} request_more_evidence={}",
            proposal_run.enter_long_count,
            proposal_run.enter_short_count,
            proposal_run.wait_count,
            proposal_run.no_trade_count,
            proposal_run.risk_deny_count,
            proposal_run.request_more_evidence_count
        ),
        entry_timing_summary: format!(
            "immediate={} next_candle={} next_n={} pullback={} retest={} cooldown={} no_entry={}",
            timing_run.immediate_paper_only_count,
            timing_run.next_candle_count,
            timing_run.next_n_candles_count,
            timing_run.pullback_confirmation_count,
            timing_run.breakout_retest_count,
            timing_run.volatility_cooldown_count,
            timing_run.no_entry_count
        ),
        debate_summary: format!("status={:?} consensus={:?}", debate.debate_status, debate.consensus_state),
        conflict_summary: format!("conflicts_detected={} resolution={:?}", conflict.conflicts_detected, conflict.conflict_resolution),
        chairman_synthesis_summary: format!("recommendation={:?}", chairman.chairman_recommendation),
        risk_governor_handoff_summary: format!("veto_result={:?} live_execution_allowed={}", risk.veto_result, risk.live_execution_allowed),
        paper_decision_trace_summary: format!("status={:?} decision={:?}", trace.trace_status, trace.paper_decision),
        roster_usage_summary: format!(
            "active_used={} watchlist_used={}",
            usage.active_members_used.len(),
            usage.watchlist_members_used.len()
        ),
        runtime_deferred_summary: "runtime deferred, training deferred, live inference forbidden, live trading forbidden, no runtime LLM live decision path, static/read-only control tower".to_string(),
        workspace_truth_summary: format!(
            "workspace_truth={:?} can_claim_full_acceptance={}",
            workspace_truth.truth_status, workspace_truth.can_claim_full_acceptance
        ),
        safety_summary: format!(
            "safety_status={:?} paper_rotation_not_order_execution_guard_present={}",
            safety.safety_status, safety.paper_rotation_not_order_execution_guard_present
        ),
        next_actions: vec![
            "keep paper-only dry-run rotation local-only and deterministic".to_string(),
            "continue lower-confidence evidence review without silent upgrades".to_string(),
            "preserve Risk Governor final veto and workspace-truth separation".to_string(),
        ],
        warnings: vec![
            "static/read-only panel only".to_string(),
            "no train button, no runtime button, no live button, no order/account controls".to_string(),
            "no browser execution and no activate-all-18-live button".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[]),
    }
}

impl Sprint102PaperRotationBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            ("## 1. Sprint summary", format!("- Implemented Sprint 102 paper-only dry-run rotation across {} scenarios using the Sprint 101 archetype registry.\n- Preserved committee-owned architecture, paper-only semantics, lower-confidence warnings, and Risk Governor final veto.", self.paper_rotation_scenario_pack.scenario_count)),
            ("## 2. Why Sprint 102 was needed", "- Sprint 101 safely ingested 18 archetype cards and staged roster expansion. Sprint 102 exercises the first paper-only dry-run rotation without activating live agents, training models, or adding runtime inference.".to_string()),
            ("## 3. Files added", "- Added Sprint 102 example TOMLs, fixture/support data, docs, focused tests, and test support for paper rotation outputs.".to_string()),
            ("## 4. Files changed", "- Extended the league layer, exports, CLI, and test support for Sprint 102.".to_string()),
            ("## 5. Paper rotation scenario pack", format!("- Status: {:?}.\n- Scenario count: {}.", self.paper_rotation_scenario_pack.pack_status, self.paper_rotation_scenario_pack.scenario_count)),
            ("## 6. Market context set", format!("- Status: {:?}.\n- Context count: {}.", self.paper_rotation_market_context_set.context_status, self.paper_rotation_market_context_set.context_count)),
            ("## 7. Archetype group rotation plan", format!("- Status: {:?}.\n- Cross-group debate assignments: {}.", self.archetype_group_rotation_plan.plan_status, self.archetype_group_rotation_plan.cross_group_debate_assignments.len())),
            ("## 8. Member selection", format!("- Status: {:?}.\n- Selected members: {}.", self.archetype_member_selection_report.selection_status, self.archetype_member_selection_report.selected_members.join(", "))),
            ("## 9. Lower-confidence evidence hardening", format!("- Status: {:?}.\n- Improved candidates: {}.", self.lower_confidence_evidence_hardening_report.report_status, self.lower_confidence_evidence_hardening_report.improved_candidates.join(", "))),
            ("## 10. Weak-source candidate review", format!("- Status: {:?}.\n- Weak source warnings reviewed: {}.", self.weak_source_candidate_review_report.review_status, self.weak_source_candidate_review_report.weak_source_warning_count)),
            ("## 11. Wonyotti evidence hardening", format!("- Status: {:?}.\n- Exact return claims blocked: {}.", self.wonyotti_evidence_hardening_report.report_status, self.wonyotti_evidence_hardening_report.exact_return_claims_blocked)),
            ("## 12. Larry Williams evidence hardening", format!("- Status: {:?}.\n- Exact numeric rule claims downweighted: {}.", self.larry_williams_evidence_hardening_report.report_status, self.larry_williams_evidence_hardening_report.exact_numeric_rule_claims_downweighted)),
            ("## 13. Arthur Hayes evidence hardening", format!("- Status: {:?}.\n- Leverage risk guard present: {}.", self.arthur_hayes_evidence_hardening_report.report_status, self.arthur_hayes_evidence_hardening_report.leverage_risk_guard_present)),
            ("## 14. Paper-only member proposal run", format!("- Status: {:?}.\n- enter_long / enter_short / wait / no_trade / risk_deny / request_more_evidence = {}/{}/{}/{}/{}/{}.", self.paper_only_member_proposal_run.run_status, self.paper_only_member_proposal_run.enter_long_count, self.paper_only_member_proposal_run.enter_short_count, self.paper_only_member_proposal_run.wait_count, self.paper_only_member_proposal_run.no_trade_count, self.paper_only_member_proposal_run.risk_deny_count, self.paper_only_member_proposal_run.request_more_evidence_count)),
            ("## 15. Entry timing proposal run", format!("- Status: {:?}.\n- immediate / next candle / next N / pullback / retest / cooldown / no-entry = {}/{}/{}/{}/{}/{}/{}.", self.paper_only_entry_timing_proposal_run.timing_status, self.paper_only_entry_timing_proposal_run.immediate_paper_only_count, self.paper_only_entry_timing_proposal_run.next_candle_count, self.paper_only_entry_timing_proposal_run.next_n_candles_count, self.paper_only_entry_timing_proposal_run.pullback_confirmation_count, self.paper_only_entry_timing_proposal_run.breakout_retest_count, self.paper_only_entry_timing_proposal_run.volatility_cooldown_count, self.paper_only_entry_timing_proposal_run.no_entry_count)),
            ("## 16. Debate trigger", format!("- Status: {:?}.\n- Trigger kind: {:?}.", self.group_debate_trigger_report.trigger_status, self.group_debate_trigger_report.trigger_kind)),
            ("## 17. Debate session", format!("- Status: {:?}.\n- Consensus state: {:?}.", self.group_debate_session_report.debate_status, self.group_debate_session_report.consensus_state)),
            ("## 18. Cross-group conflict report", format!("- Status: {:?}.\n- Conflicts detected: {}.", self.cross_group_debate_conflict_report.conflict_status, self.cross_group_debate_conflict_report.conflicts_detected)),
            ("## 19. Chairman synthesis dry-run", format!("- Status: {:?}.\n- Recommendation: {:?}.", self.chairman_synthesis_dry_run_report.synthesis_status, self.chairman_synthesis_dry_run_report.chairman_recommendation)),
            ("## 20. Chairman style weight audit", format!("- Status: {:?}.\n- risk_governor_override_attempted={}.", self.chairman_style_weight_adjustment_audit.audit_status, self.chairman_style_weight_adjustment_audit.risk_governor_override_attempted)),
            ("## 21. Risk Governor paper handoff", format!("- Status: {:?}.\n- Veto result: {:?}.", self.risk_governor_paper_handoff_report.handoff_status, self.risk_governor_paper_handoff_report.veto_result)),
            ("## 22. Paper decision trace v2", format!("- Status: {:?}.\n- Paper decision: {:?}.", self.paper_decision_trace_v2.trace_status, self.paper_decision_trace_v2.paper_decision)),
            ("## 23. Paper decision replay v2", format!("- Status: {:?}.\n- replay_count={}.", self.paper_decision_replay_v2_report.replay_status, self.paper_decision_replay_v2_report.replay_count)),
            ("## 24. Proposal expectation trace", format!("- Status: {:?}.\n- expectation_not_profit_claim={}.", self.proposal_outcome_expectation_trace.trace_status, self.proposal_outcome_expectation_trace.expectation_not_profit_claim)),
            ("## 25. NoTrade / RiskDenied committee trace", format!("- Status: {:?}.\n- no_trade_votes={} risk_deny_votes={}.", self.no_trade_risk_denied_committee_trace.trace_status, self.no_trade_risk_denied_committee_trace.no_trade_member_votes.len(), self.no_trade_risk_denied_committee_trace.risk_deny_member_votes.len())),
            ("## 26. Regime-routed dry-run", format!("- Status: {:?}.\n- routed_to_short_term / long_term / crypto / common_risk = {}/{}/{}/{}.", self.regime_routed_committee_dry_run_report.routing_status, self.regime_routed_committee_dry_run_report.routed_to_short_term_count, self.regime_routed_committee_dry_run_report.routed_to_long_term_count, self.regime_routed_committee_dry_run_report.routed_to_crypto_count, self.regime_routed_committee_dry_run_report.routed_to_common_risk_count)),
            ("## 27. Multi-expert rotation coverage", format!("- Status: {:?}.\n- total_members_selected={}.", self.multi_expert_rotation_coverage_report.coverage_status, self.multi_expert_rotation_coverage_report.total_members_selected)),
            ("## 28. Paper roster expansion usage", format!("- Status: {:?}.\n- watchlist_members_used={}.", self.paper_roster_expansion_usage_report.usage_status, self.paper_roster_expansion_usage_report.watchlist_members_used.join(", "))),
            ("## 29. 18 archetype activation safety", format!("- Status: {:?}.\n- live_activation_attempt_count={}.", self.eighteen_archetype_activation_safety_report.safety_status, self.eighteen_archetype_activation_safety_report.live_activation_attempt_count)),
            ("## 30. Workspace acceptance truth v3", format!("- Status: {:?}.\n- can_claim_full_acceptance={}.", self.workspace_acceptance_truth_closure_plan_v3.closure_status, self.workspace_acceptance_truth_closure_plan_v3.can_claim_full_acceptance)),
            ("## 31. Workspace acceptance attempt v18", format!("- Status: {:?}.\n- full_finished={}.", self.workspace_acceptance_attempt_v18.attempt_status, self.workspace_acceptance_attempt_v18.full_finished)),
            ("## 32. Safety coverage preservation v18", format!("- Status: {:?}.\n- paper_rotation_not_order_execution_guard_present={}.", self.safety_coverage_preservation_report_v18.safety_status, self.safety_coverage_preservation_report_v18.paper_rotation_not_order_execution_guard_present)),
            ("## 33. Control Tower paper rotation panel", "- Built a static/read-only panel with scenario, selection, evidence hardening, proposal, debate, chairman, risk, trace, roster, runtime, workspace, and safety summaries.\n- No train/runtime/live/order/account/browser controls or activate-all-18-live controls were added.".to_string()),
            ("## 34. Output bundle", format!("- Output files: {}.", self.storage_report.file_count)),
            ("## 35. CLI and examples", "- Added sprint102-paper-rotation and focused Sprint 102 subcommands over one local-only config surface with explicit paper-only safety warnings.".to_string()),
            ("## 36. Tests added", "- Added focused Sprint 102 config, scenario, context, rotation, hardening, proposal, timing, debate, risk, trace, panel, CLI safety, and determinism tests.".to_string()),
            ("## 37. Test results", "- See validation commands run after implementation; focused tests remain separate from full workspace truth.".to_string()),
            ("## 38. Paper rotation status", format!("- {:?}.", self.paper_rotation_scenario_pack.pack_status)),
            ("## 39. Lower-confidence evidence status", format!("- {:?}.", self.lower_confidence_evidence_hardening_report.report_status)),
            ("## 40. Debate status", format!("- {:?}.", self.group_debate_session_report.debate_status)),
            ("## 41. Risk Governor handoff status", format!("- {:?}.", self.risk_governor_paper_handoff_report.handoff_status)),
            ("## 42. Paper decision trace status", format!("- {:?}.", self.paper_decision_trace_v2.trace_status)),
            ("## 43. Runtime deferred status", "- RuntimeStillDeferred\n- TrainingStillDeferred\n- LiveTradingStillForbidden\n- KeepResearchOnly\n- KeepPaperOnly".to_string()),
            ("## 44. Workspace acceptance truth status", format!("- {:?}.", self.workspace_acceptance_attempt_v18.attempt_status)),
            ("## 45. Safety coverage status", format!("- {:?}.", self.safety_coverage_preservation_report_v18.safety_status)),
            ("## 46. Risk review", "- Chairman cannot bypass Risk Governor. Risk Governor remains final veto. Paper proposals remain paper semantics only and never become orders or live execution authority.".to_string()),
            ("## 47. Deferred items", "- Runtime inference, model training, live inference, live trading, broker/order/account, Mamba runtime, Gated runtime, dashboard serve, browser execution, and 18-live-agent activation remain deferred or forbidden.".to_string()),
            ("## 48. Next gstack sprint recommendation", "- Keep paper-only dry-run rotation conservative, continue lower-confidence evidence hardening, and pursue workspace-truth closure separately from live-readiness claims.".to_string()),
        ];
        let mut summary = String::new();
        for (heading, body) in sections {
            summary.push_str(heading);
            summary.push_str("\n\n");
            summary.push_str(&body);
            summary.push_str("\n\n");
        }
        summary
    }

    pub fn write_to_dir(&mut self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        write_json_file(
            &output_dir.join("paper_rotation_scenario_pack.txt"),
            &self.paper_rotation_scenario_pack,
        )?;
        write_json_file(
            &output_dir.join("paper_rotation_market_context_set.txt"),
            &self.paper_rotation_market_context_set,
        )?;
        write_json_file(
            &output_dir.join("archetype_group_rotation_plan.txt"),
            &self.archetype_group_rotation_plan,
        )?;
        write_json_file(
            &output_dir.join("archetype_member_selection.txt"),
            &self.archetype_member_selection_report,
        )?;
        write_json_file(
            &output_dir.join("lower_confidence_evidence_hardening_plan.txt"),
            &self.lower_confidence_evidence_hardening_plan,
        )?;
        write_json_file(
            &output_dir.join("lower_confidence_evidence_hardening.txt"),
            &self.lower_confidence_evidence_hardening_report,
        )?;
        write_json_file(
            &output_dir.join("weak_source_candidate_review.txt"),
            &self.weak_source_candidate_review_report,
        )?;
        write_json_file(
            &output_dir.join("wonyotti_evidence_hardening.txt"),
            &self.wonyotti_evidence_hardening_report,
        )?;
        write_json_file(
            &output_dir.join("larry_williams_evidence_hardening.txt"),
            &self.larry_williams_evidence_hardening_report,
        )?;
        write_json_file(
            &output_dir.join("arthur_hayes_evidence_hardening.txt"),
            &self.arthur_hayes_evidence_hardening_report,
        )?;
        write_json_file(
            &output_dir.join("paper_only_member_proposal_run.txt"),
            &self.paper_only_member_proposal_run,
        )?;
        write_json_file(
            &output_dir.join("paper_only_entry_timing_proposal_run.txt"),
            &self.paper_only_entry_timing_proposal_run,
        )?;
        write_json_file(
            &output_dir.join("group_debate_trigger.txt"),
            &self.group_debate_trigger_report,
        )?;
        write_json_file(
            &output_dir.join("group_debate_session.txt"),
            &self.group_debate_session_report,
        )?;
        write_json_file(
            &output_dir.join("cross_group_debate_conflict.txt"),
            &self.cross_group_debate_conflict_report,
        )?;
        write_json_file(
            &output_dir.join("chairman_synthesis_dry_run.txt"),
            &self.chairman_synthesis_dry_run_report,
        )?;
        write_json_file(
            &output_dir.join("chairman_style_weight_adjustment_audit.txt"),
            &self.chairman_style_weight_adjustment_audit,
        )?;
        write_json_file(
            &output_dir.join("risk_governor_paper_handoff.txt"),
            &self.risk_governor_paper_handoff_report,
        )?;
        write_json_file(
            &output_dir.join("paper_decision_trace_v2.txt"),
            &self.paper_decision_trace_v2,
        )?;
        write_json_file(
            &output_dir.join("paper_decision_replay_v2.txt"),
            &self.paper_decision_replay_v2_report,
        )?;
        write_json_file(
            &output_dir.join("proposal_outcome_expectation_trace.txt"),
            &self.proposal_outcome_expectation_trace,
        )?;
        write_json_file(
            &output_dir.join("no_trade_risk_denied_committee_trace.txt"),
            &self.no_trade_risk_denied_committee_trace,
        )?;
        write_json_file(
            &output_dir.join("regime_routed_committee_dry_run.txt"),
            &self.regime_routed_committee_dry_run_report,
        )?;
        write_json_file(
            &output_dir.join("multi_expert_rotation_coverage.txt"),
            &self.multi_expert_rotation_coverage_report,
        )?;
        write_json_file(
            &output_dir.join("paper_roster_expansion_usage.txt"),
            &self.paper_roster_expansion_usage_report,
        )?;
        write_json_file(
            &output_dir.join("eighteen_archetype_activation_safety.txt"),
            &self.eighteen_archetype_activation_safety_report,
        )?;
        write_json_file(
            &output_dir.join("workspace_acceptance_truth_closure_plan_v3.txt"),
            &self.workspace_acceptance_truth_closure_plan_v3,
        )?;
        write_json_file(
            &output_dir.join("workspace_acceptance_attempt_v18.txt"),
            &self.workspace_acceptance_attempt_v18,
        )?;
        write_json_file(
            &output_dir.join("safety_coverage_preservation_v18.txt"),
            &self.safety_coverage_preservation_report_v18,
        )?;
        write_json_file(
            &output_dir.join("control_tower_paper_rotation_panel.txt"),
            &self.control_tower_paper_rotation_panel,
        )?;
        let files = vec![
            "paper_rotation_scenario_pack.txt".to_string(),
            "paper_rotation_market_context_set.txt".to_string(),
            "archetype_group_rotation_plan.txt".to_string(),
            "archetype_member_selection.txt".to_string(),
            "lower_confidence_evidence_hardening_plan.txt".to_string(),
            "lower_confidence_evidence_hardening.txt".to_string(),
            "weak_source_candidate_review.txt".to_string(),
            "wonyotti_evidence_hardening.txt".to_string(),
            "larry_williams_evidence_hardening.txt".to_string(),
            "arthur_hayes_evidence_hardening.txt".to_string(),
            "paper_only_member_proposal_run.txt".to_string(),
            "paper_only_entry_timing_proposal_run.txt".to_string(),
            "group_debate_trigger.txt".to_string(),
            "group_debate_session.txt".to_string(),
            "cross_group_debate_conflict.txt".to_string(),
            "chairman_synthesis_dry_run.txt".to_string(),
            "chairman_style_weight_adjustment_audit.txt".to_string(),
            "risk_governor_paper_handoff.txt".to_string(),
            "paper_decision_trace_v2.txt".to_string(),
            "paper_decision_replay_v2.txt".to_string(),
            "proposal_outcome_expectation_trace.txt".to_string(),
            "no_trade_risk_denied_committee_trace.txt".to_string(),
            "regime_routed_committee_dry_run.txt".to_string(),
            "multi_expert_rotation_coverage.txt".to_string(),
            "paper_roster_expansion_usage.txt".to_string(),
            "eighteen_archetype_activation_safety.txt".to_string(),
            "workspace_acceptance_truth_closure_plan_v3.txt".to_string(),
            "workspace_acceptance_attempt_v18.txt".to_string(),
            "safety_coverage_preservation_v18.txt".to_string(),
            "control_tower_paper_rotation_panel.txt".to_string(),
            "storage_report.txt".to_string(),
            "summary.txt".to_string(),
        ];
        self.storage_report = Sprint102PaperRotationStorageReport {
            report_id: "sprint102-paper-rotation-storage-report".to_string(),
            output_dir: output_dir.display().to_string(),
            file_count: files.len(),
            files,
            reason_codes: deferred_reason_codes(&[]),
        };
        self.final_summary = self.build_final_summary();
        write_json_file(&output_dir.join("storage_report.txt"), &self.storage_report)?;
        fs::write(output_dir.join("summary.txt"), &self.final_summary)
            .map_err(|err| err.to_string())?;
        Ok(output_dir.to_path_buf())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sprint102PaperRotationRunner;

impl Sprint102PaperRotationRunner {
    pub fn run(
        &self,
        config: &EighteenArchetypePaperRotationConfig,
    ) -> Result<Sprint102PaperRotationBundle, String> {
        config.validate()?;
        let sprint101 = load_sprint101_bundle(config)?;
        let workspace_truth = load_workspace_truth(config, &sprint101)?;
        let paper_rotation_scenario_pack = build_scenario_pack(config);
        let paper_rotation_market_context_set =
            build_market_context_set(&paper_rotation_scenario_pack);
        let archetype_group_rotation_plan =
            build_group_rotation_plan(&paper_rotation_scenario_pack);
        let archetype_member_selection_report = build_member_selection(
            &sprint101.eighteen_investor_candidate_registry,
            &sprint101.eighteen_investor_committee_roster_plan,
        );
        let lower_confidence_evidence_hardening_plan = build_hardening_plan();
        let weak_source_candidate_review_report =
            build_weak_source_review(&sprint101.investor_archetype_source_confidence_report);
        let wonyotti_evidence_hardening_report =
            build_wonyotti_report(&sprint101.investor_archetype_source_confidence_report);
        let larry_williams_evidence_hardening_report =
            build_larry_report(&sprint101.investor_archetype_source_confidence_report);
        let arthur_hayes_evidence_hardening_report =
            build_arthur_report(&sprint101.investor_archetype_source_confidence_report);
        let lower_confidence_evidence_hardening_report = build_hardening_report(
            &wonyotti_evidence_hardening_report,
            &larry_williams_evidence_hardening_report,
            &arthur_hayes_evidence_hardening_report,
        );
        let paper_only_member_proposal_run = build_proposal_run(
            &archetype_member_selection_report,
            &sprint101.member_style_confidence_weight_policy,
            config,
        );
        let paper_only_entry_timing_proposal_run = build_entry_timing_run(config);
        let group_debate_trigger_report = build_debate_trigger(
            !archetype_group_rotation_plan
                .cross_group_debate_assignments
                .is_empty(),
        );
        let group_debate_session_report =
            build_debate_session(&archetype_member_selection_report, config);
        let cross_group_debate_conflict_report = build_conflict_report();
        let chairman_style_weight_adjustment_audit = build_weight_audit(
            &archetype_member_selection_report,
            &sprint101.member_style_confidence_weight_policy,
        );
        let chairman_synthesis_dry_run_report = build_chairman_report(
            &paper_only_member_proposal_run,
            &cross_group_debate_conflict_report,
            &chairman_style_weight_adjustment_audit,
        );
        let risk_governor_paper_handoff_report =
            build_risk_handoff(&chairman_synthesis_dry_run_report);
        let paper_decision_trace_v2 = build_trace(
            &paper_rotation_market_context_set,
            &paper_only_member_proposal_run,
            &group_debate_trigger_report,
            &group_debate_session_report,
            &chairman_synthesis_dry_run_report,
            &risk_governor_paper_handoff_report,
        );
        let paper_decision_replay_v2_report = build_replay(&paper_decision_trace_v2);
        let proposal_outcome_expectation_trace =
            build_expectation_trace(&paper_only_member_proposal_run);
        let no_trade_risk_denied_committee_trace =
            build_no_trade_trace(&archetype_member_selection_report);
        let regime_routed_committee_dry_run_report =
            build_regime_report(&archetype_group_rotation_plan);
        let multi_expert_rotation_coverage_report = build_coverage(
            &sprint101.eighteen_investor_candidate_registry,
            &archetype_member_selection_report,
            &sprint101.eighteen_investor_committee_roster_plan,
        );
        let paper_roster_expansion_usage_report = build_roster_usage(
            &sprint101.eighteen_investor_committee_roster_plan,
            &archetype_member_selection_report,
            &sprint101.paper_only_roster_expansion_gate,
        );
        let eighteen_archetype_activation_safety_report = build_activation_safety(
            &archetype_member_selection_report,
            &paper_roster_expansion_usage_report,
        );
        let workspace_acceptance_attempt_v18 = build_workspace_attempt(&workspace_truth);
        let workspace_acceptance_truth_closure_plan_v3 =
            build_workspace_closure(&workspace_truth, &workspace_acceptance_attempt_v18);
        let safety_coverage_preservation_report_v18 =
            build_safety_v18(&sprint101.safety_coverage_preservation_report_v17);
        let control_tower_paper_rotation_panel = build_panel(
            &paper_rotation_scenario_pack,
            &archetype_member_selection_report,
            &lower_confidence_evidence_hardening_report,
            &paper_only_member_proposal_run,
            &paper_only_entry_timing_proposal_run,
            &group_debate_session_report,
            &cross_group_debate_conflict_report,
            &chairman_synthesis_dry_run_report,
            &risk_governor_paper_handoff_report,
            &paper_decision_trace_v2,
            &paper_roster_expansion_usage_report,
            &workspace_truth,
            &safety_coverage_preservation_report_v18,
        );
        let mut bundle = Sprint102PaperRotationBundle {
            paper_rotation_scenario_pack,
            paper_rotation_market_context_set,
            archetype_group_rotation_plan,
            archetype_member_selection_report,
            lower_confidence_evidence_hardening_plan,
            lower_confidence_evidence_hardening_report,
            weak_source_candidate_review_report,
            wonyotti_evidence_hardening_report,
            larry_williams_evidence_hardening_report,
            arthur_hayes_evidence_hardening_report,
            paper_only_member_proposal_run,
            paper_only_entry_timing_proposal_run,
            group_debate_trigger_report,
            group_debate_session_report,
            cross_group_debate_conflict_report,
            chairman_synthesis_dry_run_report,
            chairman_style_weight_adjustment_audit,
            risk_governor_paper_handoff_report,
            paper_decision_trace_v2,
            paper_decision_replay_v2_report,
            proposal_outcome_expectation_trace,
            no_trade_risk_denied_committee_trace,
            regime_routed_committee_dry_run_report,
            multi_expert_rotation_coverage_report,
            paper_roster_expansion_usage_report,
            eighteen_archetype_activation_safety_report,
            workspace_acceptance_truth_closure_plan_v3,
            workspace_acceptance_attempt_v18,
            safety_coverage_preservation_report_v18,
            control_tower_paper_rotation_panel,
            storage_report: Sprint102PaperRotationStorageReport {
                report_id: "sprint102-paper-rotation-storage-report".to_string(),
                output_dir: config.output_dir().display().to_string(),
                file_count: 0,
                files: Vec::new(),
                reason_codes: deferred_reason_codes(&[]),
            },
            final_summary: String::new(),
            reason_codes: deferred_reason_codes(&[]),
        };
        bundle.write_to_dir(&config.output_dir())?;
        Ok(bundle)
    }

    pub fn run_sprint102_paper_rotation(
        &self,
        config: &EighteenArchetypePaperRotationConfig,
    ) -> Result<Sprint102PaperRotationBundle, String> {
        self.run(config)
    }
}
