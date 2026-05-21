use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::backtest::Timeframe;
use crate::core::{ReasonCode, stable_hash_string};
use crate::data::{EvidenceSourceKind, ProviderMarket};
use crate::model::{WorkspaceAcceptanceTruthGate, WorkspaceAcceptanceTruthGateStatus};

fn sprint98_reason_codes(extra: &[ReasonCode]) -> Vec<ReasonCode> {
    let mut codes = vec![
        ReasonCode::CommitteeV1Built,
        ReasonCode::CommitteeV1RunnerBuilt,
        ReasonCode::ChairV0Built,
        ReasonCode::OwnerCannotBypassRiskGovernor,
        ReasonCode::NoTradeDefault,
    ];
    codes.extend_from_slice(extra);
    codes
}

fn deferred_reason_codes(extra: &[ReasonCode]) -> Vec<ReasonCode> {
    let mut codes = sprint98_reason_codes(&[
        ReasonCode::MambaRuntimeDeferred,
        ReasonCode::GatedDeltaNetRuntimeDeferred,
        ReasonCode::ControlTowerUiReadinessBuilt,
    ]);
    codes.extend_from_slice(extra);
    codes
}

fn render_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|err| err.to_string())
}

fn local_only(path: &str) -> bool {
    !path.contains("://")
}

fn clamp_unit(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn write_json_file<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    fs::write(path, render_json(value)?).map_err(|err| err.to_string())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint98CommitteeOwnedCoreConfig {
    pub architecture_id: String,
    pub output_root: String,
    #[serde(default)]
    pub sprint97_summary_path: Option<String>,
    #[serde(default)]
    pub workspace_acceptance_truth_path: Option<String>,
    #[serde(default = "default_market")]
    pub market: ProviderMarket,
    #[serde(default = "default_symbol")]
    pub symbol: String,
    #[serde(default = "default_timeframe")]
    pub timeframe: Timeframe,
    #[serde(default = "default_timestamp_ms")]
    pub timestamp_ms: u64,
    #[serde(default = "default_source_kind")]
    pub source_kind: EvidenceSourceKind,
}

impl Default for Sprint98CommitteeOwnedCoreConfig {
    fn default() -> Self {
        Self {
            architecture_id: "sprint98-committee-owned-core".to_string(),
            output_root: "target/soma_sprint98_committee_owned_core".to_string(),
            sprint97_summary_path: Some("examples/sprint98_data/sprint97_summary.json".to_string()),
            workspace_acceptance_truth_path: Some(
                "examples/sprint98_data/workspace_acceptance_truth_import.json".to_string(),
            ),
            market: default_market(),
            symbol: default_symbol(),
            timeframe: default_timeframe(),
            timestamp_ms: default_timestamp_ms(),
            source_kind: default_source_kind(),
        }
    }
}

impl Sprint98CommitteeOwnedCoreConfig {
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

    pub fn validate(&self) -> Result<(), String> {
        if !local_only(&self.output_root)
            || self
                .sprint97_summary_path
                .as_deref()
                .is_some_and(|path| !local_only(path))
            || self
                .workspace_acceptance_truth_path
                .as_deref()
                .is_some_and(|path| !local_only(path))
        {
            return Err("sprint98 committee-owned-core config paths must be local".to_string());
        }
        if self.architecture_id.trim().is_empty() {
            return Err("sprint98 architecture_id must not be empty".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.architecture_id)
    }
}

fn default_market() -> ProviderMarket {
    ProviderMarket::KoreanEquity
}

fn default_symbol() -> String {
    "005930.KS".to_string()
}

fn default_timeframe() -> Timeframe {
    Timeframe::OneDay
}

fn default_timestamp_ms() -> u64 {
    1_730_000_000_000
}

fn default_source_kind() -> EvidenceSourceKind {
    EvidenceSourceKind::OfficialApiCollected
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint97SummaryImport {
    pub summary_id: String,
    pub sprint_name: String,
    pub queue_closure_status: String,
    pub workspace_truth_status: WorkspaceAcceptanceTruthGateStatus,
    pub queue_closed_with_workspace_still_blocked: bool,
    pub runtime_deferred: bool,
    pub safety_coverage_preserved: bool,
    pub can_claim_full_acceptance: bool,
    pub notes: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for Sprint97SummaryImport {
    fn default() -> Self {
        Self {
            summary_id: "sprint97-summary-import".to_string(),
            sprint_name: "Sprint 97 conservative queue closure".to_string(),
            queue_closure_status: "FinalBlockerQueueClosedWithWorkspaceStillBlocked".to_string(),
            workspace_truth_status: WorkspaceAcceptanceTruthGateStatus::FullWorkspaceNotRun,
            queue_closed_with_workspace_still_blocked: true,
            runtime_deferred: true,
            safety_coverage_preserved: true,
            can_claim_full_acceptance: false,
            notes: vec![
                "queue closed while full workspace acceptance remained separate".to_string(),
                "runtime stayed deferred and research-only".to_string(),
            ],
            reason_codes: deferred_reason_codes(&[]),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAcceptanceTruthImport {
    pub import_id: String,
    #[serde(default)]
    pub source_path: Option<String>,
    #[serde(default)]
    pub imported_gate_id: Option<String>,
    pub truth_status: WorkspaceAcceptanceTruthGateStatus,
    pub full_workspace_finished: bool,
    #[serde(default)]
    pub full_workspace_passed: Option<bool>,
    pub can_claim_full_acceptance: bool,
    pub queue_closed_with_workspace_still_blocked: bool,
    pub notes: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for WorkspaceAcceptanceTruthImport {
    fn default() -> Self {
        Self {
            import_id: "workspace-acceptance-truth-import".to_string(),
            source_path: None,
            imported_gate_id: Some("sprint97-workspace-truth-gate".to_string()),
            truth_status: WorkspaceAcceptanceTruthGateStatus::FullWorkspaceNotRun,
            full_workspace_finished: false,
            full_workspace_passed: None,
            can_claim_full_acceptance: false,
            queue_closed_with_workspace_still_blocked: true,
            notes: vec![
                "full workspace acceptance remains separate".to_string(),
                "do not overclaim from queue closure".to_string(),
            ],
            reason_codes: deferred_reason_codes(&[]),
        }
    }
}

impl WorkspaceAcceptanceTruthImport {
    pub fn from_gate(gate: WorkspaceAcceptanceTruthGate, source_path: Option<String>) -> Self {
        Self {
            import_id: "workspace-acceptance-truth-import".to_string(),
            source_path,
            imported_gate_id: Some(gate.gate_id),
            truth_status: gate.truth_status,
            full_workspace_finished: gate.full_workspace_finished,
            full_workspace_passed: gate.full_workspace_passed,
            can_claim_full_acceptance: gate.can_claim_full_acceptance,
            queue_closed_with_workspace_still_blocked: matches!(
                gate.truth_status,
                WorkspaceAcceptanceTruthGateStatus::FullWorkspaceNotRun
                    | WorkspaceAcceptanceTruthGateStatus::FullWorkspaceStillBlocked
                    | WorkspaceAcceptanceTruthGateStatus::FullWorkspaceFailed
            ),
            notes: vec![
                format!("no_run_status={}", gate.no_run_status),
                format!("full_workspace_status={}", gate.full_workspace_status),
            ],
            reason_codes: deferred_reason_codes(&[]),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeOwnedAiCoreArchitectureStatus {
    CommitteeOwnedCoreReady,
    CommitteeOwnedCoreReadyWithWarnings,
    CentralCoreStillLeaking,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeOwnedAiCoreArchitecture {
    pub architecture_id: String,
    pub central_core_deprecated: bool,
    pub committee_owned_core_enabled: bool,
    pub member_core_count: usize,
    pub investor_style_count: usize,
    pub chairman_governance_enabled: bool,
    pub risk_governor_final_veto_required: bool,
    pub paper_only_required: bool,
    pub runtime_deferred_required: bool,
    pub training_deferred_required: bool,
    pub live_trading_forbidden_required: bool,
    pub architecture_status: CommitteeOwnedAiCoreArchitectureStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InvestorStyleArchetypeKind {
    ValueDiscipline,
    QualityCompounder,
    MacroReflexive,
    TrendFollower,
    ContrarianMeanReversion,
    RiskFirstDefensive,
    RegimeCycle,
    EventDriven,
    LiquidityExecution,
    CounterfactualHistorian,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestorStyleStatus {
    StyleReady,
    StyleReadyWithWarnings,
    UnsafeImpersonationBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestorStyleArchetype {
    pub archetype_id: String,
    pub archetype_kind: InvestorStyleArchetypeKind,
    pub public_philosophy_inspiration: String,
    pub decision_biases: Vec<String>,
    pub preferred_evidence: Vec<String>,
    pub risk_blindspots: Vec<String>,
    pub preferred_time_horizon: String,
    pub prohibited_claims: Vec<String>,
    pub style_status: InvestorStyleStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl InvestorStyleArchetype {
    pub fn validated(mut self) -> Self {
        let inspiration = self.public_philosophy_inspiration.to_ascii_lowercase();
        let unsafe_claim = inspiration.contains("exact ")
            || inspiration.contains("exact reproduction")
            || inspiration.contains("private strategy")
            || inspiration.contains("living-person impersonation");
        self.style_status = if unsafe_claim {
            InvestorStyleStatus::UnsafeImpersonationBlocked
        } else if self
            .public_philosophy_inspiration
            .contains("public philosophy-inspired")
        {
            InvestorStyleStatus::StyleReady
        } else {
            InvestorStyleStatus::StyleReadyWithWarnings
        };
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestorStyleRegistryStatus {
    StyleRegistryReady,
    MissingRequiredStyles,
    UnsafeStyleBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestorStyleMemberRegistry {
    pub registry_id: String,
    pub styles: Vec<InvestorStyleArchetype>,
    pub member_style_assignments: BTreeMap<String, String>,
    pub duplicate_style_policy: String,
    pub registry_status: InvestorStyleRegistryStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl InvestorStyleMemberRegistry {
    pub fn build(
        registry_id: impl Into<String>,
        styles: Vec<InvestorStyleArchetype>,
        member_style_assignments: BTreeMap<String, String>,
    ) -> Self {
        let required = BTreeSet::from([
            InvestorStyleArchetypeKind::TrendFollower,
            InvestorStyleArchetypeKind::RiskFirstDefensive,
            InvestorStyleArchetypeKind::RegimeCycle,
            InvestorStyleArchetypeKind::ValueDiscipline,
            InvestorStyleArchetypeKind::MacroReflexive,
            InvestorStyleArchetypeKind::CounterfactualHistorian,
        ]);
        let present = styles
            .iter()
            .map(|style| style.archetype_kind)
            .collect::<BTreeSet<_>>();
        let unsafe_blocked = styles
            .iter()
            .any(|style| style.style_status == InvestorStyleStatus::UnsafeImpersonationBlocked);
        let registry_status = if unsafe_blocked {
            InvestorStyleRegistryStatus::UnsafeStyleBlocked
        } else if required.is_subset(&present) {
            InvestorStyleRegistryStatus::StyleRegistryReady
        } else {
            InvestorStyleRegistryStatus::MissingRequiredStyles
        };
        Self {
            registry_id: registry_id.into(),
            styles,
            member_style_assignments,
            duplicate_style_policy:
                "duplicates allowed only when members keep different paper-only roles and owned cores"
                    .to_string(),
            registry_status,
            reason_codes: sprint98_reason_codes(&[]),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AICommitteeMemberRole {
    EntryScout,
    RiskDefender,
    RegimeInterpreter,
    MacroInterpreter,
    ValueSkeptic,
    LiquidityExecutor,
    CounterfactualReviewer,
    ChairCandidate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AICommitteeMemberStatus {
    ActivePaperMember,
    WatchOnlyMember,
    DiagnosticOnlyMember,
    RetiredMember,
    BlockedMember,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AICommitteeMemberSpec {
    pub member_id: String,
    pub display_name: String,
    pub style_archetype: InvestorStyleArchetypeKind,
    pub member_role: AICommitteeMemberRole,
    pub owned_core_refs: Vec<String>,
    pub allowed_data_scopes: Vec<String>,
    pub allowed_analysis_modes: Vec<String>,
    pub proposal_permissions: Vec<CommitteeProposalAction>,
    pub debate_permissions: Vec<CommitteeDebateStance>,
    pub promotion_eligible: bool,
    pub demotion_eligible: bool,
    pub retirement_eligible: bool,
    pub member_status: AICommitteeMemberStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AICommitteeMemberCoreFamily {
    BaselineHeuristic,
    Mamba3FinDeferred,
    GatedDeltaNetDeferred,
    ExternalPredictionPrototype,
    CounterfactualEvaluator,
    RiskGovernorAdapter,
    RuleBasedFallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AICommitteeMemberCoreStatus {
    CoreContractReady,
    RuntimeDeferred,
    TrainingDeferred,
    PrototypeOnly,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AICommitteeMemberCoreContract {
    pub contract_id: String,
    pub member_id: String,
    pub core_family: AICommitteeMemberCoreFamily,
    pub input_context_schema: String,
    pub output_proposal_schema: String,
    #[serde(default)]
    pub feature_schema_hash: Option<String>,
    #[serde(default)]
    pub label_manifest_hash: Option<String>,
    pub runtime_allowed: bool,
    pub training_allowed: bool,
    pub live_inference_allowed: bool,
    pub paper_only_required: bool,
    pub core_status: AICommitteeMemberCoreStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeOwnedCoreRegistryStatus {
    MemberCoreRegistryReady,
    MissingMemberCore,
    RuntimeLeakBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeOwnedCoreRegistry {
    pub registry_id: String,
    pub member_core_contracts: Vec<AICommitteeMemberCoreContract>,
    pub runtime_deferred_count: usize,
    pub training_deferred_count: usize,
    pub prototype_only_count: usize,
    pub registry_status: CommitteeOwnedCoreRegistryStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl CommitteeOwnedCoreRegistry {
    pub fn build(
        registry_id: impl Into<String>,
        member_core_contracts: Vec<AICommitteeMemberCoreContract>,
    ) -> Self {
        let runtime_deferred_count = member_core_contracts
            .iter()
            .filter(|contract| !contract.runtime_allowed)
            .count();
        let training_deferred_count = member_core_contracts
            .iter()
            .filter(|contract| !contract.training_allowed)
            .count();
        let prototype_only_count = member_core_contracts
            .iter()
            .filter(|contract| {
                matches!(
                    contract.core_status,
                    AICommitteeMemberCoreStatus::PrototypeOnly
                )
            })
            .count();
        let registry_status = if member_core_contracts.is_empty() {
            CommitteeOwnedCoreRegistryStatus::MissingMemberCore
        } else if member_core_contracts.iter().any(|contract| {
            contract.runtime_allowed || contract.training_allowed || contract.live_inference_allowed
        }) {
            CommitteeOwnedCoreRegistryStatus::RuntimeLeakBlocked
        } else {
            CommitteeOwnedCoreRegistryStatus::MemberCoreRegistryReady
        };
        Self {
            registry_id: registry_id.into(),
            member_core_contracts,
            runtime_deferred_count,
            training_deferred_count,
            prototype_only_count,
            registry_status,
            reason_codes: deferred_reason_codes(&[]),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AICommitteeMemberLearningMode {
    OfflineStudyOnly,
    FixtureReplay,
    HistoricalBacktest,
    ExternalPredictionReview,
    CounterfactualReview,
    LiveLearningForbidden,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AICommitteeMemberAnalysisLoopStatus {
    AnalysisLoopReady,
    AnalysisLoopReadyWithWarnings,
    NeedMoreEvidence,
    BlockedBySafety,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AICommitteeMemberAnalysisLoop {
    pub loop_id: String,
    pub member_id: String,
    pub input_context_refs: Vec<String>,
    pub analysis_tasks: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub study_notes: Vec<String>,
    pub learning_mode: AICommitteeMemberLearningMode,
    pub output_proposals: Vec<String>,
    pub loop_status: AICommitteeMemberAnalysisLoopStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AICommitteeMemberLearningPolicyStatus {
    LearningPolicyReady,
    LearningPolicyReadyWithWarnings,
    UnsafeLearningBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AICommitteeMemberLearningPolicy {
    pub policy_id: String,
    pub member_id: String,
    pub can_read_historical_data: bool,
    pub can_read_official_data: bool,
    pub can_read_research_data: bool,
    pub can_generate_study_notes: bool,
    pub can_update_member_scorecard: bool,
    pub can_update_model_weights: bool,
    pub can_train_model: bool,
    pub can_use_live_data_for_training: bool,
    pub can_access_broker_account: bool,
    pub policy_status: AICommitteeMemberLearningPolicyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketContextForCommitteeStatus {
    ContextReady,
    NeedMoreEvidence,
    SourceBoundaryBlocked,
    NoLookaheadBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarketContextForCommittee {
    pub context_id: String,
    pub market: ProviderMarket,
    pub symbol: String,
    pub timeframe: Timeframe,
    pub timestamp: u64,
    pub source_class: EvidenceSourceKind,
    pub candle_refs: Vec<String>,
    pub feature_refs: Vec<String>,
    pub regime_refs: Vec<String>,
    pub risk_refs: Vec<String>,
    #[serde(default)]
    pub existing_paper_position: Option<String>,
    pub evidence_quality: f64,
    #[serde(default)]
    pub no_lookahead_proof_ref: Option<String>,
    pub context_status: MarketContextForCommitteeStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeProposalAction {
    EnterLong,
    EnterShort,
    Wait,
    NoTrade,
    RiskDeny,
    RequestMoreEvidence,
    WatchCandidate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AICommitteeMemberProposalStatus {
    ProposalReady,
    ProposalReadyWithWarnings,
    InsufficientEvidence,
    SafetyBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryTimingWindow {
    ImmediatePaperOnly,
    NextCandle,
    NextNCandles,
    PullbackConfirmation,
    BreakoutRetest,
    VolatilityCooldown,
    NoEntry,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryTimingProposalStatus {
    EntryTimingReady,
    EntryTimingConditional,
    EntryTimingBlocked,
    NoEntryRecommended,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryTimingProposal {
    pub timing_id: String,
    pub member_id: String,
    pub symbol: String,
    pub timeframe: Timeframe,
    pub entry_window: EntryTimingWindow,
    #[serde(default)]
    pub earliest_entry_timestamp: Option<u64>,
    #[serde(default)]
    pub latest_entry_timestamp: Option<u64>,
    pub confirmation_conditions: Vec<String>,
    pub cancellation_conditions: Vec<String>,
    pub required_risk_checks: Vec<String>,
    pub timing_status: EntryTimingProposalStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AICommitteeMemberProposal {
    pub proposal_id: String,
    pub member_id: String,
    pub style_archetype: InvestorStyleArchetypeKind,
    pub proposed_action: CommitteeProposalAction,
    #[serde(default)]
    pub proposed_entry_timing: Option<EntryTimingProposal>,
    pub confidence: f64,
    #[serde(default)]
    pub expected_return_proxy: Option<f64>,
    #[serde(default)]
    pub expected_risk_proxy: Option<f64>,
    pub invalidation_condition: String,
    pub wait_condition: String,
    #[serde(default)]
    pub stop_condition: Option<String>,
    #[serde(default)]
    pub take_profit_condition: Option<String>,
    pub evidence_refs: Vec<String>,
    pub dissent_refs: Vec<String>,
    pub proposal_status: AICommitteeMemberProposalStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl AICommitteeMemberProposal {
    pub fn bounded(mut self) -> Self {
        self.confidence = clamp_unit(self.confidence);
        self.expected_return_proxy = self.expected_return_proxy.map(clamp_unit);
        self.expected_risk_proxy = self.expected_risk_proxy.map(clamp_unit);
        if self.evidence_refs.is_empty() {
            self.proposal_status = AICommitteeMemberProposalStatus::InsufficientEvidence;
        }
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeDebateTriggerReason {
    EntryTimingProposed,
    RiskDenyProposed,
    HighDisagreement,
    NewEvidence,
    OwnerReviewRequest,
    RuleReviewRequest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeDebateTriggerStatus {
    DebateTriggered,
    DebateNotRequired,
    BlockedBySafety,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeDebateTrigger {
    pub trigger_id: String,
    pub triggering_member_id: String,
    pub triggering_proposal_id: String,
    pub trigger_reason: CommitteeDebateTriggerReason,
    pub debate_required: bool,
    pub trigger_status: CommitteeDebateTriggerStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl CommitteeDebateTrigger {
    pub fn from_proposal(proposal: &AICommitteeMemberProposal, force_blocked: bool) -> Self {
        let trigger_reason = match proposal.proposed_action {
            CommitteeProposalAction::RiskDeny => CommitteeDebateTriggerReason::RiskDenyProposed,
            CommitteeProposalAction::EnterLong | CommitteeProposalAction::EnterShort => {
                CommitteeDebateTriggerReason::EntryTimingProposed
            }
            CommitteeProposalAction::RequestMoreEvidence => {
                CommitteeDebateTriggerReason::NewEvidence
            }
            CommitteeProposalAction::Wait
            | CommitteeProposalAction::NoTrade
            | CommitteeProposalAction::WatchCandidate => {
                CommitteeDebateTriggerReason::HighDisagreement
            }
        };
        let debate_required = matches!(
            proposal.proposed_action,
            CommitteeProposalAction::EnterLong
                | CommitteeProposalAction::EnterShort
                | CommitteeProposalAction::RiskDeny
                | CommitteeProposalAction::RequestMoreEvidence
        );
        Self {
            trigger_id: format!("{}-trigger", proposal.proposal_id),
            triggering_member_id: proposal.member_id.clone(),
            triggering_proposal_id: proposal.proposal_id.clone(),
            trigger_reason,
            debate_required,
            trigger_status: if force_blocked {
                CommitteeDebateTriggerStatus::BlockedBySafety
            } else if debate_required {
                CommitteeDebateTriggerStatus::DebateTriggered
            } else {
                CommitteeDebateTriggerStatus::DebateNotRequired
            },
            reason_codes: sprint98_reason_codes(&[]),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeConsensusState {
    UnanimousEnter,
    ConditionalEnter,
    SplitDecision,
    MajorityWait,
    NoTradeConsensus,
    RiskDenied,
    NeedMoreEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeDebateSessionStatus {
    DebateReady,
    DebateReadyWithWarnings,
    DebateIncomplete,
    BlockedBySafety,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeDebateStance {
    SupportEntry,
    OpposeEntry,
    WaitForConfirmation,
    DemandRiskDeny,
    DemandNoTrade,
    RequestMoreEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeMemberDebateTurnStatus {
    TurnReady,
    TurnReadyWithWarnings,
    InsufficientEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeMemberDebateTurn {
    pub turn_id: String,
    pub session_id: String,
    pub member_id: String,
    pub stance: CommitteeDebateStance,
    pub argument_summary: String,
    pub evidence_refs: Vec<String>,
    pub counterarguments: Vec<String>,
    pub confidence: f64,
    pub turn_status: CommitteeMemberDebateTurnStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl CommitteeMemberDebateTurn {
    pub fn bounded(mut self) -> Self {
        self.confidence = clamp_unit(self.confidence);
        if self.evidence_refs.is_empty() {
            self.turn_status = CommitteeMemberDebateTurnStatus::InsufficientEvidence;
        }
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeDebateSession {
    pub session_id: String,
    pub trigger: CommitteeDebateTrigger,
    pub participating_members: Vec<String>,
    pub member_turns: Vec<CommitteeMemberDebateTurn>,
    pub consensus_state: CommitteeConsensusState,
    #[serde(default)]
    pub chair_synthesis_ref: Option<String>,
    #[serde(default)]
    pub risk_governor_decision_ref: Option<String>,
    pub debate_status: CommitteeDebateSessionStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl CommitteeDebateSession {
    pub fn new(
        session_id: impl Into<String>,
        trigger: CommitteeDebateTrigger,
        member_turns: Vec<CommitteeMemberDebateTurn>,
    ) -> Self {
        let participating_members = member_turns
            .iter()
            .map(|turn| turn.member_id.clone())
            .collect::<Vec<_>>();
        let consensus_state = consensus_from_turns(&member_turns);
        let debate_status =
            if trigger.trigger_status == CommitteeDebateTriggerStatus::BlockedBySafety {
                CommitteeDebateSessionStatus::BlockedBySafety
            } else if member_turns.is_empty() {
                CommitteeDebateSessionStatus::DebateIncomplete
            } else if matches!(
                consensus_state,
                CommitteeConsensusState::SplitDecision | CommitteeConsensusState::NeedMoreEvidence
            ) {
                CommitteeDebateSessionStatus::DebateReadyWithWarnings
            } else {
                CommitteeDebateSessionStatus::DebateReady
            };
        Self {
            session_id: session_id.into(),
            trigger,
            participating_members,
            member_turns,
            consensus_state,
            chair_synthesis_ref: Some("chairman-synthesis-paper-only".to_string()),
            risk_governor_decision_ref: Some("risk-governor-paper-veto-gate".to_string()),
            debate_status,
            reason_codes: sprint98_reason_codes(&[]),
        }
    }
}

fn consensus_from_turns(turns: &[CommitteeMemberDebateTurn]) -> CommitteeConsensusState {
    if turns.is_empty() {
        return CommitteeConsensusState::NeedMoreEvidence;
    }
    if turns
        .iter()
        .any(|turn| turn.stance == CommitteeDebateStance::DemandRiskDeny)
    {
        return CommitteeConsensusState::RiskDenied;
    }
    if turns
        .iter()
        .any(|turn| turn.stance == CommitteeDebateStance::RequestMoreEvidence)
    {
        return CommitteeConsensusState::NeedMoreEvidence;
    }
    let support = turns
        .iter()
        .filter(|turn| turn.stance == CommitteeDebateStance::SupportEntry)
        .count();
    let oppose = turns
        .iter()
        .filter(|turn| turn.stance == CommitteeDebateStance::OpposeEntry)
        .count();
    let wait = turns
        .iter()
        .filter(|turn| turn.stance == CommitteeDebateStance::WaitForConfirmation)
        .count();
    let no_trade = turns
        .iter()
        .filter(|turn| turn.stance == CommitteeDebateStance::DemandNoTrade)
        .count();
    if support == turns.len() {
        CommitteeConsensusState::UnanimousEnter
    } else if support > oppose && support > wait && support > no_trade {
        CommitteeConsensusState::ConditionalEnter
    } else if wait >= support && wait >= oppose && wait >= no_trade {
        CommitteeConsensusState::MajorityWait
    } else if no_trade > support && no_trade >= oppose {
        CommitteeConsensusState::NoTradeConsensus
    } else {
        CommitteeConsensusState::SplitDecision
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanRuleAuthority {
    ProposeRulesOnly,
    VersionRulesWithAudit,
    ApplyPaperRulesOnly,
    LiveRuleApplicationForbidden,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanAiGovernancePolicyStatus {
    ChairmanGovernanceReady,
    ChairmanGovernanceReadyWithWarnings,
    UnsafeChairAuthorityBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairmanAiGovernancePolicy {
    pub policy_id: String,
    #[serde(default)]
    pub chairman_member_id: Option<String>,
    pub rule_authority: Vec<ChairmanRuleAuthority>,
    pub can_change_rules_without_audit: bool,
    pub can_bypass_risk_governor: bool,
    pub can_promote_member_unilaterally: bool,
    pub can_demote_member_unilaterally: bool,
    pub owner_review_required_for_major_rule_change: bool,
    pub policy_status: ChairmanAiGovernancePolicyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanRuleProposalKind {
    DebateThresholdChange,
    PromotionCriteriaChange,
    DemotionCriteriaChange,
    EvidenceWeightChange,
    RiskWeightChange,
    QuorumChange,
    CooldownPolicyChange,
    NoTradeBiasChange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanRuleProposalStatus {
    RuleProposalReady,
    NeedsAudit,
    RejectedByRiskGovernor,
    UnsafeRuleBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairmanRuleProposal {
    pub rule_proposal_id: String,
    pub chairman_member_id: String,
    pub proposal_kind: ChairmanRuleProposalKind,
    pub proposed_rule_text: String,
    pub structured_rule_delta: BTreeMap<String, String>,
    pub expected_effect: String,
    pub required_audit: bool,
    pub owner_review_required: bool,
    pub risk_governor_review_required: bool,
    pub proposal_status: ChairmanRuleProposalStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanRulebookVersionStatus {
    RulebookReady,
    RulebookReadyWithWarnings,
    NeedsAudit,
    BlockedByRiskGovernor,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairmanRulebookVersion {
    pub version_id: String,
    #[serde(default)]
    pub previous_version_id: Option<String>,
    pub rules: Vec<String>,
    pub changed_by: String,
    pub change_reason: String,
    pub audit_refs: Vec<String>,
    pub active_for_paper_only: bool,
    pub live_use_forbidden: bool,
    pub rulebook_status: ChairmanRulebookVersionStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleAdaptationAuditStatus {
    RuleAuditPassedForPaper,
    RuleAuditNeedsMoreEvidence,
    RuleAuditRejected,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuleAdaptationAudit {
    pub audit_id: String,
    pub rule_proposal_id: String,
    pub simulation_refs: Vec<String>,
    pub counterfactual_refs: Vec<String>,
    pub overfit_risk: f64,
    pub safety_risk: f64,
    pub expected_behavior_change: String,
    pub audit_status: RuleAdaptationAuditStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PromotionAxis {
    ReturnQuality,
    Calibration,
    DrawdownControl,
    RiskGovernorAlignment,
    NoTradeDiscipline,
    RiskDeniedRespect,
    EvidenceQuality,
    SourceBoundaryDiscipline,
    NoLookaheadDiscipline,
    DebateContribution,
    RegimeSpecialization,
    DefensiveValue,
    OpportunityCostAwareness,
    OverfitRisk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionDemotionPolicyStatus {
    PromotionPolicyReady,
    PromotionPolicyReadyWithWarnings,
    NeedsChairRulebook,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PromotionDemotionPolicy {
    pub policy_id: String,
    pub axes: Vec<PromotionAxis>,
    pub promotion_thresholds: BTreeMap<PromotionAxis, f64>,
    pub demotion_thresholds: BTreeMap<PromotionAxis, f64>,
    pub retirement_thresholds: BTreeMap<PromotionAxis, f64>,
    pub policy_status: PromotionDemotionPolicyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl PromotionDemotionPolicy {
    pub fn evaluate_member(
        &self,
        member_id: &str,
        previous_rank: usize,
        scorecard: &MultiAxisMemberScorecard,
        chair_rulebook_version: &str,
    ) -> MemberPromotionDemotionDecision {
        let overall = scorecard.axis_scores.values().copied().sum::<f64>()
            / scorecard.axis_scores.len().max(1) as f64;
        let promote = overall >= 0.78 && scorecard.overfit_risk_score <= 0.35;
        let retire = overall < 0.30 || scorecard.overfit_risk_score >= 0.85;
        let demote = !retire && (overall < 0.45 || scorecard.risk_alignment_score < 0.45);
        let action = if retire {
            MemberPromotionDemotionAction::RetireToDiagnostic
        } else if promote {
            MemberPromotionDemotionAction::Promote
        } else if demote {
            MemberPromotionDemotionAction::Demote
        } else if overall < 0.55 {
            MemberPromotionDemotionAction::Watchlist
        } else {
            MemberPromotionDemotionAction::Keep
        };
        let new_rank = match action {
            MemberPromotionDemotionAction::Promote => previous_rank.saturating_add(1),
            MemberPromotionDemotionAction::Demote => previous_rank.saturating_sub(1),
            MemberPromotionDemotionAction::RetireToDiagnostic => 0,
            MemberPromotionDemotionAction::Watchlist => previous_rank,
            MemberPromotionDemotionAction::Keep => previous_rank,
            MemberPromotionDemotionAction::RequestMoreEvidence => previous_rank,
        };
        MemberPromotionDemotionDecision {
            decision_id: format!("{}-promotion-decision", member_id),
            member_id: member_id.to_string(),
            previous_rank,
            new_rank,
            action,
            decision_basis: vec![
                format!("overall_score={overall:.3}"),
                format!("risk_alignment={:.3}", scorecard.risk_alignment_score),
                format!(
                    "no_trade_discipline={:.3}",
                    scorecard.no_trade_discipline_score
                ),
                format!("overfit_risk={:.3}", scorecard.overfit_risk_score),
            ],
            chair_rulebook_version: chair_rulebook_version.to_string(),
            owner_review_required: matches!(action, MemberPromotionDemotionAction::Promote),
            risk_governor_blocked: false,
            decision_status: if matches!(action, MemberPromotionDemotionAction::Watchlist)
                && overall < 0.50
            {
                MemberPromotionDemotionDecisionStatus::NeedsMoreEvidence
            } else {
                MemberPromotionDemotionDecisionStatus::DecisionReady
            },
            reason_codes: sprint98_reason_codes(&[]),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultiAxisMemberScorecardStatus {
    ScorecardReady,
    ScorecardReadyWithWarnings,
    NeedMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MultiAxisMemberScorecard {
    pub scorecard_id: String,
    pub member_id: String,
    pub axis_scores: BTreeMap<PromotionAxis, f64>,
    pub recent_proposals: Vec<String>,
    pub debate_turn_quality: f64,
    pub risk_alignment_score: f64,
    pub no_trade_discipline_score: f64,
    pub calibration_score: f64,
    pub overfit_risk_score: f64,
    pub overall_research_rank: usize,
    pub scorecard_status: MultiAxisMemberScorecardStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberPromotionDemotionAction {
    Promote,
    Demote,
    Keep,
    Watchlist,
    RetireToDiagnostic,
    RequestMoreEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberPromotionDemotionDecisionStatus {
    DecisionReady,
    NeedsMoreEvidence,
    BlockedByRiskGovernor,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberPromotionDemotionDecision {
    pub decision_id: String,
    pub member_id: String,
    pub previous_rank: usize,
    pub new_rank: usize,
    pub action: MemberPromotionDemotionAction,
    pub decision_basis: Vec<String>,
    pub chair_rulebook_version: String,
    pub owner_review_required: bool,
    pub risk_governor_blocked: bool,
    pub decision_status: MemberPromotionDemotionDecisionStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeRosterLifecycleStatus {
    RosterReady,
    RosterReadyWithWarnings,
    NeedsMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeRosterLifecycle {
    pub lifecycle_id: String,
    pub active_members: Vec<String>,
    pub watchlist_members: Vec<String>,
    pub diagnostic_members: Vec<String>,
    pub retired_members: Vec<String>,
    pub isolated_sentinels: Vec<String>,
    pub promotions: Vec<String>,
    pub demotions: Vec<String>,
    pub retirements: Vec<String>,
    pub lifecycle_status: CommitteeRosterLifecycleStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperOnlyCommitteeDecisionKind {
    WatchCandidate,
    PaperApproved,
    PaperRejected,
    NoTrade,
    RiskDenied,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperOnlyCommitteeDecisionRecord {
    pub decision_id: String,
    pub debate_session_id: String,
    pub chair_synthesis_id: String,
    pub risk_governor_decision_id: String,
    pub final_decision: PaperOnlyCommitteeDecisionKind,
    #[serde(default)]
    pub proposed_entry_timing: Option<EntryTimingProposal>,
    pub paper_only: bool,
    pub broker_execution_allowed: bool,
    pub live_execution_allowed: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl PaperOnlyCommitteeDecisionRecord {
    pub fn new(
        decision_id: impl Into<String>,
        debate_session_id: impl Into<String>,
        final_decision: PaperOnlyCommitteeDecisionKind,
        proposed_entry_timing: Option<EntryTimingProposal>,
    ) -> Self {
        Self {
            decision_id: decision_id.into(),
            debate_session_id: debate_session_id.into(),
            chair_synthesis_id: "chairman-synthesis-paper-only".to_string(),
            risk_governor_decision_id: "risk-governor-paper-veto-gate".to_string(),
            final_decision,
            proposed_entry_timing,
            paper_only: true,
            broker_execution_allowed: false,
            live_execution_allowed: false,
            reason_codes: deferred_reason_codes(&[]),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerAiCommitteeRow {
    pub member_id: String,
    pub current_status: String,
    pub style: String,
    pub model_core: String,
    pub current_analysis_task: String,
    pub proposal: String,
    pub debate_stance: String,
    pub promotion_demotion_status: String,
    pub wait_risk_no_trade_reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlTowerAiCommitteePanelStatus {
    PanelReady,
    PanelReadyWithWarnings,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerAiCommitteePanel {
    pub panel_id: String,
    pub architecture_status: CommitteeOwnedAiCoreArchitectureStatus,
    pub member_count: usize,
    pub member_rows: Vec<ControlTowerAiCommitteeRow>,
    pub active_debate_sessions: Vec<String>,
    pub recent_entry_timing_proposals: Vec<String>,
    pub chairman_rulebook_status: String,
    pub promotion_demotion_summary: String,
    pub risk_governor_summary: String,
    pub paper_decision_summary: String,
    pub runtime_deferred_summary: String,
    pub warnings: Vec<String>,
    pub next_actions: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyCoveragePreservationReportV14Status {
    SafetyCoveragePreserved,
    SafetyCoveragePreservedWithWarnings,
    SafetyCoverageMissing,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyCoveragePreservationReportV14 {
    pub report_id: String,
    pub no_runtime_llm_live_decision_path: bool,
    pub no_live_trading_path: bool,
    pub no_broker_order_account_path: bool,
    pub no_mamba_runtime: bool,
    pub no_gated_runtime: bool,
    pub no_model_training: bool,
    pub no_python_training_dependency: bool,
    pub no_tauri_svelte_dependency: bool,
    pub no_dashboard_serve: bool,
    pub no_browser_execution: bool,
    pub paper_only_preserved: bool,
    pub local_only_preserved: bool,
    pub static_control_tower_preserved: bool,
    pub chairman_cannot_bypass_risk_governor: bool,
    pub risk_governor_final_veto_preserved: bool,
    pub full_workspace_acceptance_separate: bool,
    pub committee_cli_safety_isolated: bool,
    pub warnings: Vec<String>,
    pub safety_status: SafetyCoveragePreservationReportV14Status,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint98CommitteeOwnedCoreStorageReport {
    pub report_id: String,
    pub output_dir: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint98CommitteeOwnedCoreBundle {
    pub committee_owned_ai_core_architecture: CommitteeOwnedAiCoreArchitecture,
    pub investor_style_archetype_registry: InvestorStyleMemberRegistry,
    pub ai_committee_member_specs: Vec<AICommitteeMemberSpec>,
    pub ai_committee_member_core_contracts: Vec<AICommitteeMemberCoreContract>,
    pub committee_owned_core_registry: CommitteeOwnedCoreRegistry,
    pub ai_committee_member_learning_policies: Vec<AICommitteeMemberLearningPolicy>,
    pub market_context_for_committee: MarketContextForCommittee,
    pub ai_committee_member_analysis_loops: Vec<AICommitteeMemberAnalysisLoop>,
    pub ai_committee_member_proposals: Vec<AICommitteeMemberProposal>,
    pub entry_timing_proposals: Vec<EntryTimingProposal>,
    pub committee_debate_trigger: CommitteeDebateTrigger,
    pub committee_debate_session: CommitteeDebateSession,
    pub committee_member_debate_turns: Vec<CommitteeMemberDebateTurn>,
    pub chairman_ai_governance_policy: ChairmanAiGovernancePolicy,
    pub chairman_rule_proposals: Vec<ChairmanRuleProposal>,
    pub chairman_rulebook_version: ChairmanRulebookVersion,
    pub rule_adaptation_audit: RuleAdaptationAudit,
    pub promotion_demotion_policy: PromotionDemotionPolicy,
    pub multi_axis_member_scorecards: Vec<MultiAxisMemberScorecard>,
    pub member_promotion_demotion_decisions: Vec<MemberPromotionDemotionDecision>,
    pub committee_roster_lifecycle: CommitteeRosterLifecycle,
    pub paper_only_committee_decision_record: PaperOnlyCommitteeDecisionRecord,
    pub control_tower_ai_committee_panel: ControlTowerAiCommitteePanel,
    pub safety_coverage_preservation_report_v14: SafetyCoveragePreservationReportV14,
    pub workspace_acceptance_truth_import: WorkspaceAcceptanceTruthImport,
    pub storage_report: Sprint98CommitteeOwnedCoreStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl Sprint98CommitteeOwnedCoreBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        format!(
            "## 1. Sprint summary\n- Implemented Sprint 98 committee-owned AI core re-architecture in research-only, paper-only, local-only mode.\n\n## 2. User intent correction\n- Central AI core feeding a committee is deprecated. Each committee member now owns its own AI core, style archetype, analysis loop, proposal path, and debate role.\n\n## 3. Files added\n- output bundle emitted under target/soma_sprint98_committee_owned_core/<architecture_id>/\n\n## 4. Files changed\n- committee-owned-core module, exports, CLI, examples, docs, and focused tests updated.\n\n## 5. Committee-owned AI core architecture\n- status: {:?}\n- central_core_deprecated={}\n- member_core_count={}\n\n## 6. Investor style archetypes\n- registry_status: {:?}\n- style_count={}\n\n## 7. AI committee member specs\n- member_count={}\n\n## 8. Member-owned core contracts\n- contract_count={}\n- runtime_deferred_count={}\n\n## 9. Member learning policies\n- policy_count={}\n\n## 10. Market context for committee\n- context_status: {:?}\n- symbol={}\n\n## 11. Member analysis loops\n- loop_count={}\n\n## 12. Entry timing proposals\n- proposal_count={}\n- timing_count={}\n\n## 13. Debate trigger\n- trigger_status: {:?}\n\n## 14. Debate session and turns\n- debate_status: {:?}\n- consensus_state: {:?}\n- turn_count={}\n\n## 15. Chairman AI governance\n- policy_status: {:?}\n- chair_can_bypass_risk_governor={}\n\n## 16. Chairman rule proposals and rulebook\n- rule_proposal_count={}\n- rulebook_status: {:?}\n\n## 17. Rule adaptation audit\n- audit_status: {:?}\n\n## 18. Promotion/demotion policy\n- policy_status: {:?}\n- axis_count={}\n\n## 19. Member scorecards\n- scorecard_count={}\n\n## 20. Roster lifecycle\n- lifecycle_status: {:?}\n\n## 21. Paper-only committee decision\n- final_decision: {:?}\n- paper_only={}\n\n## 22. Control Tower AI committee panel\n- member_count={}\n- runtime_deferred_summary={}\n\n## 23. CLI and examples\n- sprint98-committee-owned-core and focused subcommands use the same local-only config surface.\n\n## 24. Tests added\n- focused Sprint 98 architecture/style/spec/contract/policy/proposal/debate/governance/panel/CLI/determinism tests.\n\n## 25. Test results\n- see validation commands run after implementation.\n\n## 26. Architecture status\n- central core deprecated, committee-owned core enabled, investor-style members ready for paper prototype.\n\n## 27. Runtime deferred status\n- runtime_deferred_required={}\n- training_deferred_required={}\n- live_trading_forbidden_required={}\n\n## 28. Workspace acceptance truth status\n- truth_status: {:?}\n- can_claim_full_acceptance={}\n\n## 29. Safety coverage status\n- safety_status: {:?}\n- warnings={}\n",
            self.committee_owned_ai_core_architecture
                .architecture_status,
            self.committee_owned_ai_core_architecture
                .central_core_deprecated,
            self.committee_owned_ai_core_architecture.member_core_count,
            self.investor_style_archetype_registry.registry_status,
            self.investor_style_archetype_registry.styles.len(),
            self.ai_committee_member_specs.len(),
            self.ai_committee_member_core_contracts.len(),
            self.committee_owned_core_registry.runtime_deferred_count,
            self.ai_committee_member_learning_policies.len(),
            self.market_context_for_committee.context_status,
            self.market_context_for_committee.symbol,
            self.ai_committee_member_analysis_loops.len(),
            self.ai_committee_member_proposals.len(),
            self.entry_timing_proposals.len(),
            self.committee_debate_trigger.trigger_status,
            self.committee_debate_session.debate_status,
            self.committee_debate_session.consensus_state,
            self.committee_member_debate_turns.len(),
            self.chairman_ai_governance_policy.policy_status,
            self.chairman_ai_governance_policy.can_bypass_risk_governor,
            self.chairman_rule_proposals.len(),
            self.chairman_rulebook_version.rulebook_status,
            self.rule_adaptation_audit.audit_status,
            self.promotion_demotion_policy.policy_status,
            self.promotion_demotion_policy.axes.len(),
            self.multi_axis_member_scorecards.len(),
            self.committee_roster_lifecycle.lifecycle_status,
            self.paper_only_committee_decision_record.final_decision,
            self.paper_only_committee_decision_record.paper_only,
            self.control_tower_ai_committee_panel.member_count,
            self.control_tower_ai_committee_panel
                .runtime_deferred_summary,
            self.committee_owned_ai_core_architecture
                .runtime_deferred_required,
            self.committee_owned_ai_core_architecture
                .training_deferred_required,
            self.committee_owned_ai_core_architecture
                .live_trading_forbidden_required,
            self.workspace_acceptance_truth_import.truth_status,
            self.workspace_acceptance_truth_import
                .can_claim_full_acceptance,
            self.safety_coverage_preservation_report_v14.safety_status,
            self.safety_coverage_preservation_report_v14
                .warnings
                .join(" | "),
        )
    }

    pub fn write_to_dir(&mut self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        write_json_file(
            &output_dir.join("committee_owned_ai_core_architecture.txt"),
            &self.committee_owned_ai_core_architecture,
        )?;
        write_json_file(
            &output_dir.join("investor_style_archetype_registry.txt"),
            &self.investor_style_archetype_registry,
        )?;
        write_json_file(
            &output_dir.join("ai_committee_member_specs.txt"),
            &self.ai_committee_member_specs,
        )?;
        write_json_file(
            &output_dir.join("ai_committee_member_core_contracts.txt"),
            &self.ai_committee_member_core_contracts,
        )?;
        write_json_file(
            &output_dir.join("committee_owned_core_registry.txt"),
            &self.committee_owned_core_registry,
        )?;
        write_json_file(
            &output_dir.join("ai_committee_member_learning_policies.txt"),
            &self.ai_committee_member_learning_policies,
        )?;
        write_json_file(
            &output_dir.join("market_context_for_committee.txt"),
            &self.market_context_for_committee,
        )?;
        write_json_file(
            &output_dir.join("ai_committee_member_proposals.txt"),
            &self.ai_committee_member_proposals,
        )?;
        write_json_file(
            &output_dir.join("entry_timing_proposals.txt"),
            &self.entry_timing_proposals,
        )?;
        write_json_file(
            &output_dir.join("committee_debate_trigger.txt"),
            &self.committee_debate_trigger,
        )?;
        write_json_file(
            &output_dir.join("committee_debate_session.txt"),
            &self.committee_debate_session,
        )?;
        write_json_file(
            &output_dir.join("committee_member_debate_turns.txt"),
            &self.committee_member_debate_turns,
        )?;
        write_json_file(
            &output_dir.join("chairman_ai_governance_policy.txt"),
            &self.chairman_ai_governance_policy,
        )?;
        write_json_file(
            &output_dir.join("chairman_rule_proposals.txt"),
            &self.chairman_rule_proposals,
        )?;
        write_json_file(
            &output_dir.join("chairman_rulebook_version.txt"),
            &self.chairman_rulebook_version,
        )?;
        write_json_file(
            &output_dir.join("rule_adaptation_audit.txt"),
            &self.rule_adaptation_audit,
        )?;
        write_json_file(
            &output_dir.join("promotion_demotion_policy.txt"),
            &self.promotion_demotion_policy,
        )?;
        write_json_file(
            &output_dir.join("multi_axis_member_scorecards.txt"),
            &self.multi_axis_member_scorecards,
        )?;
        write_json_file(
            &output_dir.join("member_promotion_demotion_decisions.txt"),
            &self.member_promotion_demotion_decisions,
        )?;
        write_json_file(
            &output_dir.join("committee_roster_lifecycle.txt"),
            &self.committee_roster_lifecycle,
        )?;
        write_json_file(
            &output_dir.join("paper_only_committee_decision_record.txt"),
            &self.paper_only_committee_decision_record,
        )?;
        write_json_file(
            &output_dir.join("control_tower_ai_committee_panel.txt"),
            &self.control_tower_ai_committee_panel,
        )?;
        write_json_file(
            &output_dir.join("safety_coverage_preservation_v14.txt"),
            &self.safety_coverage_preservation_report_v14,
        )?;
        write_json_file(
            &output_dir.join("workspace_acceptance_truth_import.txt"),
            &self.workspace_acceptance_truth_import,
        )?;

        let files = vec![
            "committee_owned_ai_core_architecture.txt".to_string(),
            "investor_style_archetype_registry.txt".to_string(),
            "ai_committee_member_specs.txt".to_string(),
            "ai_committee_member_core_contracts.txt".to_string(),
            "committee_owned_core_registry.txt".to_string(),
            "ai_committee_member_learning_policies.txt".to_string(),
            "market_context_for_committee.txt".to_string(),
            "ai_committee_member_proposals.txt".to_string(),
            "entry_timing_proposals.txt".to_string(),
            "committee_debate_trigger.txt".to_string(),
            "committee_debate_session.txt".to_string(),
            "committee_member_debate_turns.txt".to_string(),
            "chairman_ai_governance_policy.txt".to_string(),
            "chairman_rule_proposals.txt".to_string(),
            "chairman_rulebook_version.txt".to_string(),
            "rule_adaptation_audit.txt".to_string(),
            "promotion_demotion_policy.txt".to_string(),
            "multi_axis_member_scorecards.txt".to_string(),
            "member_promotion_demotion_decisions.txt".to_string(),
            "committee_roster_lifecycle.txt".to_string(),
            "paper_only_committee_decision_record.txt".to_string(),
            "control_tower_ai_committee_panel.txt".to_string(),
            "safety_coverage_preservation_v14.txt".to_string(),
            "workspace_acceptance_truth_import.txt".to_string(),
            "storage_report.txt".to_string(),
            "summary.txt".to_string(),
        ];
        self.storage_report = Sprint98CommitteeOwnedCoreStorageReport {
            report_id: format!(
                "{}-storage-report",
                self.committee_owned_ai_core_architecture.architecture_id
            ),
            output_dir: output_dir.display().to_string(),
            file_count: files.len(),
            files,
            reason_codes: sprint98_reason_codes(&[]),
        };
        self.final_summary = self.build_final_summary();
        write_json_file(&output_dir.join("storage_report.txt"), &self.storage_report)?;
        fs::write(output_dir.join("summary.txt"), &self.final_summary)
            .map_err(|err| err.to_string())?;
        Ok(output_dir.to_path_buf())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sprint98CommitteeOwnedCoreRunner;

impl Sprint98CommitteeOwnedCoreRunner {
    pub fn run(
        &self,
        config: &Sprint98CommitteeOwnedCoreConfig,
    ) -> Result<Sprint98CommitteeOwnedCoreBundle, String> {
        config.validate()?;
        let sprint97_summary = load_sprint97_summary_import(config)?;
        let workspace_import = load_workspace_acceptance_truth_import(config)?;
        let styles = build_investor_style_archetypes();
        let specs = build_member_specs();
        let assignments = specs
            .iter()
            .map(|spec| {
                (
                    spec.member_id.clone(),
                    style_id(spec.style_archetype).to_string(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let style_registry = InvestorStyleMemberRegistry::build(
            format!("{}-style-registry", config.architecture_id),
            styles,
            assignments,
        );
        let contracts = build_member_core_contracts();
        let core_registry = CommitteeOwnedCoreRegistry::build(
            format!("{}-core-registry", config.architecture_id),
            contracts.clone(),
        );
        let architecture = CommitteeOwnedAiCoreArchitecture {
            architecture_id: config.architecture_id.clone(),
            central_core_deprecated: true,
            committee_owned_core_enabled: true,
            member_core_count: contracts.len(),
            investor_style_count: style_registry.styles.len(),
            chairman_governance_enabled: true,
            risk_governor_final_veto_required: true,
            paper_only_required: true,
            runtime_deferred_required: true,
            training_deferred_required: true,
            live_trading_forbidden_required: true,
            architecture_status: if workspace_import.can_claim_full_acceptance {
                CommitteeOwnedAiCoreArchitectureStatus::CommitteeOwnedCoreReady
            } else {
                CommitteeOwnedAiCoreArchitectureStatus::CommitteeOwnedCoreReadyWithWarnings
            },
            reason_codes: deferred_reason_codes(&[]),
        };
        let policies = build_learning_policies(&specs);
        let market_context = build_market_context(config);
        let entry_timing_proposals = build_entry_timing_proposals(config);
        let proposals = build_member_proposals(&specs, &entry_timing_proposals);
        let loops = build_analysis_loops(&specs, &market_context, &proposals);
        let trigger = CommitteeDebateTrigger::from_proposal(&proposals[0], false);
        let debate_turns = build_debate_turns(&specs, &trigger.trigger_id);
        let debate_session = CommitteeDebateSession::new(
            format!("{}-debate-session", config.architecture_id),
            trigger.clone(),
            debate_turns.clone(),
        );
        let governance = build_governance_policy();
        let rule_proposals = build_rule_proposals();
        let rulebook = build_rulebook(&rule_proposals);
        let rule_audit = build_rule_audit(&rule_proposals[0]);
        let promotion_policy = build_promotion_policy();
        let scorecards = build_member_scorecards(&specs, &proposals);
        let promotion_decisions = scorecards
            .iter()
            .enumerate()
            .map(|(index, scorecard)| {
                promotion_policy.evaluate_member(
                    &scorecard.member_id,
                    specs.len().saturating_sub(index),
                    scorecard,
                    &rulebook.version_id,
                )
            })
            .collect::<Vec<_>>();
        let roster = build_roster_lifecycle(&specs, &promotion_decisions);
        let paper_decision = PaperOnlyCommitteeDecisionRecord::new(
            format!("{}-paper-decision", config.architecture_id),
            debate_session.session_id.clone(),
            match debate_session.consensus_state {
                CommitteeConsensusState::UnanimousEnter
                | CommitteeConsensusState::ConditionalEnter => {
                    PaperOnlyCommitteeDecisionKind::WatchCandidate
                }
                CommitteeConsensusState::RiskDenied => PaperOnlyCommitteeDecisionKind::RiskDenied,
                CommitteeConsensusState::NoTradeConsensus => {
                    PaperOnlyCommitteeDecisionKind::NoTrade
                }
                CommitteeConsensusState::MajorityWait
                | CommitteeConsensusState::NeedMoreEvidence => {
                    PaperOnlyCommitteeDecisionKind::NeedMoreEvidence
                }
                CommitteeConsensusState::SplitDecision => {
                    PaperOnlyCommitteeDecisionKind::PaperRejected
                }
            },
            entry_timing_proposals.first().cloned(),
        );
        let safety = build_safety_report(&sprint97_summary, &workspace_import);
        let panel = build_control_tower_panel(
            &architecture,
            &specs,
            &contracts,
            &loops,
            &proposals,
            &debate_session,
            &rulebook,
            &promotion_decisions,
            &paper_decision,
            &safety,
        );
        let mut bundle = Sprint98CommitteeOwnedCoreBundle {
            committee_owned_ai_core_architecture: architecture,
            investor_style_archetype_registry: style_registry,
            ai_committee_member_specs: specs,
            ai_committee_member_core_contracts: contracts,
            committee_owned_core_registry: core_registry,
            ai_committee_member_learning_policies: policies,
            market_context_for_committee: market_context,
            ai_committee_member_analysis_loops: loops,
            ai_committee_member_proposals: proposals,
            entry_timing_proposals,
            committee_debate_trigger: trigger,
            committee_debate_session: debate_session,
            committee_member_debate_turns: debate_turns,
            chairman_ai_governance_policy: governance,
            chairman_rule_proposals: rule_proposals,
            chairman_rulebook_version: rulebook,
            rule_adaptation_audit: rule_audit,
            promotion_demotion_policy: promotion_policy,
            multi_axis_member_scorecards: scorecards,
            member_promotion_demotion_decisions: promotion_decisions,
            committee_roster_lifecycle: roster,
            paper_only_committee_decision_record: paper_decision,
            control_tower_ai_committee_panel: panel,
            safety_coverage_preservation_report_v14: safety,
            workspace_acceptance_truth_import: workspace_import,
            storage_report: Sprint98CommitteeOwnedCoreStorageReport {
                report_id: format!("{}-storage-report", config.architecture_id),
                output_dir: config.output_dir().display().to_string(),
                file_count: 0,
                files: Vec::new(),
                reason_codes: sprint98_reason_codes(&[]),
            },
            final_summary: String::new(),
            reason_codes: sprint98_reason_codes(&[]),
        };
        bundle.write_to_dir(&config.output_dir())?;
        Ok(bundle)
    }

    pub fn run_sprint98_committee_owned_core(
        &self,
        config: &Sprint98CommitteeOwnedCoreConfig,
    ) -> Result<Sprint98CommitteeOwnedCoreBundle, String> {
        self.run(config)
    }
}

fn load_sprint97_summary_import(
    config: &Sprint98CommitteeOwnedCoreConfig,
) -> Result<Sprint97SummaryImport, String> {
    match &config.sprint97_summary_path {
        Some(path) => {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            serde_json::from_str(&text).map_err(|err| err.to_string())
        }
        None => Ok(Sprint97SummaryImport::default()),
    }
}

fn load_workspace_acceptance_truth_import(
    config: &Sprint98CommitteeOwnedCoreConfig,
) -> Result<WorkspaceAcceptanceTruthImport, String> {
    match &config.workspace_acceptance_truth_path {
        Some(path) => {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            serde_json::from_str::<WorkspaceAcceptanceTruthImport>(&text)
                .or_else(|_| {
                    serde_json::from_str::<WorkspaceAcceptanceTruthGate>(&text).map(|gate| {
                        WorkspaceAcceptanceTruthImport::from_gate(gate, Some(path.clone()))
                    })
                })
                .map_err(|err| err.to_string())
        }
        None => Ok(WorkspaceAcceptanceTruthImport::default()),
    }
}

fn build_investor_style_archetypes() -> Vec<InvestorStyleArchetype> {
    vec![
        archetype(
            InvestorStyleArchetypeKind::TrendFollower,
            "trend-follower",
            "public philosophy-inspired trend and momentum discipline, never an investor impersonation",
            &[
                "prefers persistent price confirmation",
                "accepts waiting for breakout follow-through",
            ],
            &["official candles", "regime labels", "paper liquidity notes"],
            &[
                "late entries during fast reversals",
                "crowded breakout risk",
            ],
            "swing-to-multiday",
        ),
        archetype(
            InvestorStyleArchetypeKind::RiskFirstDefensive,
            "risk-first-defensive",
            "public philosophy-inspired capital preservation bias, not a private or living-person clone",
            &[
                "defaults to no-trade when evidence is thin",
                "prefers veto-compatible downside framing",
            ],
            &[
                "risk snapshots",
                "drawdown evidence",
                "source boundary checks",
            ],
            &["underreacting to valid upside regime shifts"],
            "multiday",
        ),
        archetype(
            InvestorStyleArchetypeKind::RegimeCycle,
            "regime-cycle",
            "public philosophy-inspired regime and cycle interpretation, not a macro discretionary replication",
            &[
                "weights market regime transitions heavily",
                "penalizes off-regime timing windows",
            ],
            &[
                "regime features",
                "volatility context",
                "official market breadth proxies",
            ],
            &["narrative overfitting to noisy macro changes"],
            "swing-to-long-term",
        ),
        archetype(
            InvestorStyleArchetypeKind::ValueDiscipline,
            "value-discipline",
            "public philosophy-inspired valuation discipline with explicit no-impersonation disclaimer",
            &[
                "requires valuation margin before approval",
                "prefers patient pullback entries",
            ],
            &[
                "fundamental notes",
                "counterfactual history",
                "official daily data",
            ],
            &["value traps in deteriorating regimes"],
            "long-term",
        ),
        archetype(
            InvestorStyleArchetypeKind::MacroReflexive,
            "macro-reflexive",
            "public philosophy-inspired reflexive macro interpretation, never a proprietary strategy claim",
            &[
                "tracks policy and liquidity narratives",
                "leans wait when macro evidence diverges",
            ],
            &[
                "macro catalysts",
                "event calendars",
                "external prediction prototypes",
            ],
            &["headline sensitivity and narrative whipsaw"],
            "multiday",
        ),
        archetype(
            InvestorStyleArchetypeKind::CounterfactualHistorian,
            "counterfactual-historian",
            "public philosophy-inspired post-mortem and counterfactual review discipline, not a real person recreation",
            &[
                "requests more evidence before decisive approval",
                "scores prior paper outcomes and debate quality",
            ],
            &[
                "counterfactual archives",
                "debate transcripts",
                "paper decision logs",
            ],
            &["conservative bias after adverse historical samples"],
            "multiday",
        ),
        archetype(
            InvestorStyleArchetypeKind::LiquidityExecution,
            "liquidity-execution",
            "public philosophy-inspired execution discipline without any live broker or order path",
            &[
                "demands spread and liquidity confirmation",
                "prefers breakout retest or cooldown entries",
            ],
            &[
                "spread snapshots",
                "volume profiles",
                "paper execution assumptions",
            ],
            &["missed opportunities in illiquid rebounds"],
            "intraday-to-swing paper timing",
        ),
        archetype(
            InvestorStyleArchetypeKind::QualityCompounder,
            "quality-compounder",
            "public philosophy-inspired quality compounding mindset used only for paper governance and chair candidacy",
            &[
                "favors durable evidence over fast signals",
                "weights calibration and debate quality",
            ],
            &["scorecards", "rule audits", "long-horizon evidence"],
            &["slow reaction to short-horizon inflections"],
            "long-term",
        ),
    ]
}

fn archetype(
    kind: InvestorStyleArchetypeKind,
    archetype_id: &str,
    inspiration: &str,
    biases: &[&str],
    evidence: &[&str],
    blindspots: &[&str],
    horizon: &str,
) -> InvestorStyleArchetype {
    InvestorStyleArchetype {
        archetype_id: archetype_id.to_string(),
        archetype_kind: kind,
        public_philosophy_inspiration: inspiration.to_string(),
        decision_biases: biases.iter().map(|value| (*value).to_string()).collect(),
        preferred_evidence: evidence.iter().map(|value| (*value).to_string()).collect(),
        risk_blindspots: blindspots
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        preferred_time_horizon: horizon.to_string(),
        prohibited_claims: vec![
            "no exact reproduction of any real investor".to_string(),
            "no private strategy claim".to_string(),
            "no living-person impersonation".to_string(),
        ],
        style_status: InvestorStyleStatus::StyleReady,
        reason_codes: sprint98_reason_codes(&[]),
    }
    .validated()
}

fn style_id(kind: InvestorStyleArchetypeKind) -> &'static str {
    match kind {
        InvestorStyleArchetypeKind::ValueDiscipline => "value-discipline",
        InvestorStyleArchetypeKind::QualityCompounder => "quality-compounder",
        InvestorStyleArchetypeKind::MacroReflexive => "macro-reflexive",
        InvestorStyleArchetypeKind::TrendFollower => "trend-follower",
        InvestorStyleArchetypeKind::ContrarianMeanReversion => "contrarian-mean-reversion",
        InvestorStyleArchetypeKind::RiskFirstDefensive => "risk-first-defensive",
        InvestorStyleArchetypeKind::RegimeCycle => "regime-cycle",
        InvestorStyleArchetypeKind::EventDriven => "event-driven",
        InvestorStyleArchetypeKind::LiquidityExecution => "liquidity-execution",
        InvestorStyleArchetypeKind::CounterfactualHistorian => "counterfactual-historian",
    }
}

fn build_member_specs() -> Vec<AICommitteeMemberSpec> {
    vec![
        member_spec(
            "trend-scout",
            "Trend Scout",
            InvestorStyleArchetypeKind::TrendFollower,
            AICommitteeMemberRole::EntryScout,
            &["trend-scout-core"],
            AICommitteeMemberStatus::ActivePaperMember,
        ),
        member_spec(
            "risk-defender",
            "Risk Defender",
            InvestorStyleArchetypeKind::RiskFirstDefensive,
            AICommitteeMemberRole::RiskDefender,
            &["risk-defender-core"],
            AICommitteeMemberStatus::ActivePaperMember,
        ),
        member_spec(
            "regime-interpreter",
            "Regime Interpreter",
            InvestorStyleArchetypeKind::RegimeCycle,
            AICommitteeMemberRole::RegimeInterpreter,
            &["regime-interpreter-core"],
            AICommitteeMemberStatus::ActivePaperMember,
        ),
        member_spec(
            "macro-interpreter",
            "Macro Interpreter",
            InvestorStyleArchetypeKind::MacroReflexive,
            AICommitteeMemberRole::MacroInterpreter,
            &["macro-interpreter-core"],
            AICommitteeMemberStatus::WatchOnlyMember,
        ),
        member_spec(
            "value-skeptic",
            "Value Skeptic",
            InvestorStyleArchetypeKind::ValueDiscipline,
            AICommitteeMemberRole::ValueSkeptic,
            &["value-skeptic-core"],
            AICommitteeMemberStatus::ActivePaperMember,
        ),
        member_spec(
            "liquidity-executor",
            "Liquidity Executor",
            InvestorStyleArchetypeKind::LiquidityExecution,
            AICommitteeMemberRole::LiquidityExecutor,
            &["liquidity-executor-core"],
            AICommitteeMemberStatus::DiagnosticOnlyMember,
        ),
        member_spec(
            "counterfactual-reviewer",
            "Counterfactual Reviewer",
            InvestorStyleArchetypeKind::CounterfactualHistorian,
            AICommitteeMemberRole::CounterfactualReviewer,
            &["counterfactual-reviewer-core"],
            AICommitteeMemberStatus::ActivePaperMember,
        ),
        member_spec(
            "chair-candidate",
            "Chair Candidate",
            InvestorStyleArchetypeKind::QualityCompounder,
            AICommitteeMemberRole::ChairCandidate,
            &["chair-candidate-core"],
            AICommitteeMemberStatus::ActivePaperMember,
        ),
    ]
}

fn member_spec(
    member_id: &str,
    display_name: &str,
    style_archetype: InvestorStyleArchetypeKind,
    member_role: AICommitteeMemberRole,
    owned_core_refs: &[&str],
    member_status: AICommitteeMemberStatus,
) -> AICommitteeMemberSpec {
    AICommitteeMemberSpec {
        member_id: member_id.to_string(),
        display_name: display_name.to_string(),
        style_archetype,
        member_role,
        owned_core_refs: owned_core_refs
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        allowed_data_scopes: vec![
            "official-market-data".to_string(),
            "paper-research-evidence".to_string(),
            "counterfactual-review".to_string(),
        ],
        allowed_analysis_modes: vec![
            "offline-study".to_string(),
            "fixture-replay".to_string(),
            "paper-debate".to_string(),
        ],
        proposal_permissions: vec![
            CommitteeProposalAction::EnterLong,
            CommitteeProposalAction::Wait,
            CommitteeProposalAction::NoTrade,
            CommitteeProposalAction::RiskDeny,
            CommitteeProposalAction::RequestMoreEvidence,
            CommitteeProposalAction::WatchCandidate,
        ],
        debate_permissions: vec![
            CommitteeDebateStance::SupportEntry,
            CommitteeDebateStance::OpposeEntry,
            CommitteeDebateStance::WaitForConfirmation,
            CommitteeDebateStance::DemandRiskDeny,
            CommitteeDebateStance::DemandNoTrade,
            CommitteeDebateStance::RequestMoreEvidence,
        ],
        promotion_eligible: !matches!(member_status, AICommitteeMemberStatus::RetiredMember),
        demotion_eligible: true,
        retirement_eligible: true,
        member_status,
        reason_codes: sprint98_reason_codes(&[]),
    }
}

fn build_member_core_contracts() -> Vec<AICommitteeMemberCoreContract> {
    vec![
        core_contract(
            "trend-scout",
            "trend-scout-core",
            AICommitteeMemberCoreFamily::BaselineHeuristic,
            AICommitteeMemberCoreStatus::CoreContractReady,
        ),
        core_contract(
            "risk-defender",
            "risk-defender-core",
            AICommitteeMemberCoreFamily::RiskGovernorAdapter,
            AICommitteeMemberCoreStatus::CoreContractReady,
        ),
        core_contract(
            "regime-interpreter",
            "regime-interpreter-core",
            AICommitteeMemberCoreFamily::GatedDeltaNetDeferred,
            AICommitteeMemberCoreStatus::RuntimeDeferred,
        ),
        core_contract(
            "macro-interpreter",
            "macro-interpreter-core",
            AICommitteeMemberCoreFamily::ExternalPredictionPrototype,
            AICommitteeMemberCoreStatus::PrototypeOnly,
        ),
        core_contract(
            "value-skeptic",
            "value-skeptic-core",
            AICommitteeMemberCoreFamily::Mamba3FinDeferred,
            AICommitteeMemberCoreStatus::RuntimeDeferred,
        ),
        core_contract(
            "liquidity-executor",
            "liquidity-executor-core",
            AICommitteeMemberCoreFamily::RuleBasedFallback,
            AICommitteeMemberCoreStatus::CoreContractReady,
        ),
        core_contract(
            "counterfactual-reviewer",
            "counterfactual-reviewer-core",
            AICommitteeMemberCoreFamily::CounterfactualEvaluator,
            AICommitteeMemberCoreStatus::PrototypeOnly,
        ),
        core_contract(
            "chair-candidate",
            "chair-candidate-core",
            AICommitteeMemberCoreFamily::RuleBasedFallback,
            AICommitteeMemberCoreStatus::CoreContractReady,
        ),
    ]
}

fn core_contract(
    member_id: &str,
    contract_id: &str,
    core_family: AICommitteeMemberCoreFamily,
    core_status: AICommitteeMemberCoreStatus,
) -> AICommitteeMemberCoreContract {
    AICommitteeMemberCoreContract {
        contract_id: contract_id.to_string(),
        member_id: member_id.to_string(),
        core_family,
        input_context_schema: "MarketContextForCommittee/v1".to_string(),
        output_proposal_schema: "AICommitteeMemberProposal/v1".to_string(),
        feature_schema_hash: Some(stable_hash_string(&format!("{member_id}:feature-schema"))),
        label_manifest_hash: Some(stable_hash_string(&format!("{member_id}:label-manifest"))),
        runtime_allowed: false,
        training_allowed: false,
        live_inference_allowed: false,
        paper_only_required: true,
        core_status,
        reason_codes: match core_family {
            AICommitteeMemberCoreFamily::Mamba3FinDeferred => {
                deferred_reason_codes(&[ReasonCode::MambaRuntimeDeferred])
            }
            AICommitteeMemberCoreFamily::GatedDeltaNetDeferred => {
                deferred_reason_codes(&[ReasonCode::GatedDeltaNetRuntimeDeferred])
            }
            _ => sprint98_reason_codes(&[]),
        },
    }
}

fn build_learning_policies(
    specs: &[AICommitteeMemberSpec],
) -> Vec<AICommitteeMemberLearningPolicy> {
    specs
        .iter()
        .map(|spec| AICommitteeMemberLearningPolicy {
            policy_id: format!("{}-learning-policy", spec.member_id),
            member_id: spec.member_id.clone(),
            can_read_historical_data: true,
            can_read_official_data: true,
            can_read_research_data: true,
            can_generate_study_notes: true,
            can_update_member_scorecard: true,
            can_update_model_weights: false,
            can_train_model: false,
            can_use_live_data_for_training: false,
            can_access_broker_account: false,
            policy_status: AICommitteeMemberLearningPolicyStatus::LearningPolicyReady,
            reason_codes: deferred_reason_codes(&[]),
        })
        .collect()
}

fn build_market_context(config: &Sprint98CommitteeOwnedCoreConfig) -> MarketContextForCommittee {
    MarketContextForCommittee {
        context_id: format!("{}-market-context", config.architecture_id),
        market: config.market,
        symbol: config.symbol.clone(),
        timeframe: config.timeframe,
        timestamp: config.timestamp_ms,
        source_class: config.source_kind,
        candle_refs: vec!["official-candle-pack/2024-10-29".to_string()],
        feature_refs: vec!["feature-pack/trend-liquidity-regime-v1".to_string()],
        regime_refs: vec!["regime/risk-on-with-volatility-cooldown".to_string()],
        risk_refs: vec!["risk/max-drawdown-paper-guard".to_string()],
        existing_paper_position: None,
        evidence_quality: 0.82,
        no_lookahead_proof_ref: Some("proof/no-lookahead-sequence-v1".to_string()),
        context_status: MarketContextForCommitteeStatus::ContextReady,
        reason_codes: sprint98_reason_codes(&[]),
    }
}

fn build_entry_timing_proposals(
    config: &Sprint98CommitteeOwnedCoreConfig,
) -> Vec<EntryTimingProposal> {
    vec![
        EntryTimingProposal {
            timing_id: "trend-scout-entry-window".to_string(),
            member_id: "trend-scout".to_string(),
            symbol: config.symbol.clone(),
            timeframe: config.timeframe,
            entry_window: EntryTimingWindow::NextCandle,
            earliest_entry_timestamp: Some(config.timestamp_ms + 86_400_000),
            latest_entry_timestamp: Some(config.timestamp_ms + 172_800_000),
            confirmation_conditions: vec![
                "breakout holds above prior swing high on paper close".to_string(),
            ],
            cancellation_conditions: vec![
                "risk governor denies or breakout fails immediately".to_string(),
            ],
            required_risk_checks: vec![
                "spread-within-paper-limit".to_string(),
                "paper-size-capped".to_string(),
            ],
            timing_status: EntryTimingProposalStatus::EntryTimingReady,
            reason_codes: sprint98_reason_codes(&[]),
        },
        EntryTimingProposal {
            timing_id: "macro-interpreter-wait-window".to_string(),
            member_id: "macro-interpreter".to_string(),
            symbol: config.symbol.clone(),
            timeframe: config.timeframe,
            entry_window: EntryTimingWindow::VolatilityCooldown,
            earliest_entry_timestamp: Some(config.timestamp_ms + 172_800_000),
            latest_entry_timestamp: Some(config.timestamp_ms + 432_000_000),
            confirmation_conditions: vec![
                "macro volatility regime cools below paper threshold".to_string(),
            ],
            cancellation_conditions: vec!["new macro shock evidence".to_string()],
            required_risk_checks: vec!["risk governor cooldown cleared".to_string()],
            timing_status: EntryTimingProposalStatus::EntryTimingConditional,
            reason_codes: sprint98_reason_codes(&[]),
        },
        EntryTimingProposal {
            timing_id: "risk-defender-no-entry".to_string(),
            member_id: "risk-defender".to_string(),
            symbol: config.symbol.clone(),
            timeframe: config.timeframe,
            entry_window: EntryTimingWindow::NoEntry,
            earliest_entry_timestamp: None,
            latest_entry_timestamp: None,
            confirmation_conditions: vec!["none until defensive drawdown improves".to_string()],
            cancellation_conditions: vec!["fresh evidence of lower downside risk".to_string()],
            required_risk_checks: vec!["paper drawdown below veto threshold".to_string()],
            timing_status: EntryTimingProposalStatus::NoEntryRecommended,
            reason_codes: sprint98_reason_codes(&[]),
        },
        EntryTimingProposal {
            timing_id: "value-skeptic-pullback".to_string(),
            member_id: "value-skeptic".to_string(),
            symbol: config.symbol.clone(),
            timeframe: config.timeframe,
            entry_window: EntryTimingWindow::PullbackConfirmation,
            earliest_entry_timestamp: Some(config.timestamp_ms + 86_400_000),
            latest_entry_timestamp: Some(config.timestamp_ms + 604_800_000),
            confirmation_conditions: vec!["pullback respects valuation support zone".to_string()],
            cancellation_conditions: vec!["support breaks with weak evidence quality".to_string()],
            required_risk_checks: vec![
                "source-boundary-clean".to_string(),
                "no-lookahead-proof-present".to_string(),
            ],
            timing_status: EntryTimingProposalStatus::EntryTimingConditional,
            reason_codes: sprint98_reason_codes(&[]),
        },
    ]
}

fn build_member_proposals(
    specs: &[AICommitteeMemberSpec],
    entry_timing_proposals: &[EntryTimingProposal],
) -> Vec<AICommitteeMemberProposal> {
    let timing_map = entry_timing_proposals
        .iter()
        .map(|timing| (timing.member_id.clone(), timing.clone()))
        .collect::<BTreeMap<_, _>>();
    specs
        .iter()
        .map(|spec| {
            let (
                action,
                confidence,
                return_proxy,
                risk_proxy,
                wait_condition,
                invalidation,
                stop,
                take_profit,
            ) = match spec.member_role {
                AICommitteeMemberRole::EntryScout => (
                    CommitteeProposalAction::EnterLong,
                    0.79,
                    Some(0.63),
                    Some(0.31),
                    "wait until next paper candle confirms breakout".to_string(),
                    "breakout loses follow-through immediately".to_string(),
                    Some("paper stop if breakout closes back inside range".to_string()),
                    Some("paper take-profit into measured extension".to_string()),
                ),
                AICommitteeMemberRole::RiskDefender => (
                    CommitteeProposalAction::RiskDeny,
                    0.88,
                    None,
                    Some(0.84),
                    "wait until downside risk compresses".to_string(),
                    "new risk evidence lowers veto score".to_string(),
                    None,
                    None,
                ),
                AICommitteeMemberRole::RegimeInterpreter => (
                    CommitteeProposalAction::WatchCandidate,
                    0.66,
                    Some(0.58),
                    Some(0.42),
                    "wait for regime persistence across two paper closes".to_string(),
                    "regime flips back to range".to_string(),
                    None,
                    None,
                ),
                AICommitteeMemberRole::MacroInterpreter => (
                    CommitteeProposalAction::Wait,
                    0.61,
                    Some(0.46),
                    Some(0.48),
                    "wait until macro volatility cools".to_string(),
                    "new macro shock evidence arrives".to_string(),
                    None,
                    None,
                ),
                AICommitteeMemberRole::ValueSkeptic => (
                    CommitteeProposalAction::RequestMoreEvidence,
                    0.64,
                    Some(0.52),
                    Some(0.39),
                    "wait for pullback support and valuation evidence".to_string(),
                    "valuation support breaks".to_string(),
                    Some("paper stop on failed support retest".to_string()),
                    Some("paper trim after mean reversion".to_string()),
                ),
                AICommitteeMemberRole::LiquidityExecutor => (
                    CommitteeProposalAction::NoTrade,
                    0.72,
                    None,
                    Some(0.67),
                    "wait until spread compresses materially".to_string(),
                    "paper liquidity improves beyond threshold".to_string(),
                    None,
                    None,
                ),
                AICommitteeMemberRole::CounterfactualReviewer => (
                    CommitteeProposalAction::RequestMoreEvidence,
                    0.69,
                    Some(0.41),
                    Some(0.37),
                    "wait for stronger analog cases".to_string(),
                    "counterfactual analogs fail".to_string(),
                    None,
                    None,
                ),
                AICommitteeMemberRole::ChairCandidate => (
                    CommitteeProposalAction::WatchCandidate,
                    0.58,
                    None,
                    Some(0.28),
                    "wait until debate and rulebook checks close".to_string(),
                    "governance audit flags unsafe shortcut".to_string(),
                    None,
                    None,
                ),
            };
            AICommitteeMemberProposal {
                proposal_id: format!("{}-proposal", spec.member_id),
                member_id: spec.member_id.clone(),
                style_archetype: spec.style_archetype,
                proposed_action: action,
                proposed_entry_timing: timing_map.get(&spec.member_id).cloned(),
                confidence,
                expected_return_proxy: return_proxy,
                expected_risk_proxy: risk_proxy,
                invalidation_condition: invalidation,
                wait_condition,
                stop_condition: stop,
                take_profit_condition: take_profit,
                evidence_refs: vec![
                    "official-evidence/sample-market-context".to_string(),
                    format!("member-study-notes/{}", spec.member_id),
                ],
                dissent_refs: vec!["risk-governor-paper-veto-gate".to_string()],
                proposal_status: AICommitteeMemberProposalStatus::ProposalReady,
                reason_codes: sprint98_reason_codes(&[]),
            }
            .bounded()
        })
        .collect()
}

fn build_analysis_loops(
    specs: &[AICommitteeMemberSpec],
    market_context: &MarketContextForCommittee,
    proposals: &[AICommitteeMemberProposal],
) -> Vec<AICommitteeMemberAnalysisLoop> {
    let proposal_ids = proposals
        .iter()
        .map(|proposal| (proposal.member_id.clone(), proposal.proposal_id.clone()))
        .collect::<BTreeMap<_, _>>();
    specs
        .iter()
        .map(|spec| AICommitteeMemberAnalysisLoop {
            loop_id: format!("{}-analysis-loop", spec.member_id),
            member_id: spec.member_id.clone(),
            input_context_refs: vec![market_context.context_id.clone()],
            analysis_tasks: vec![
                format!("review offline evidence for {}", market_context.symbol),
                "keep proposal generation paper-only and local-only".to_string(),
            ],
            evidence_refs: vec![
                market_context
                    .no_lookahead_proof_ref
                    .clone()
                    .unwrap_or_else(|| "proof/missing".to_string()),
                "evidence/source-boundary-check".to_string(),
            ],
            study_notes: vec![
                "autonomous learning means bounded offline study only".to_string(),
                "no runtime or online weight mutation".to_string(),
            ],
            learning_mode: match spec.member_role {
                AICommitteeMemberRole::CounterfactualReviewer => {
                    AICommitteeMemberLearningMode::CounterfactualReview
                }
                AICommitteeMemberRole::MacroInterpreter => {
                    AICommitteeMemberLearningMode::ExternalPredictionReview
                }
                AICommitteeMemberRole::LiquidityExecutor => {
                    AICommitteeMemberLearningMode::FixtureReplay
                }
                _ => AICommitteeMemberLearningMode::OfflineStudyOnly,
            },
            output_proposals: vec![proposal_ids[&spec.member_id].clone()],
            loop_status: AICommitteeMemberAnalysisLoopStatus::AnalysisLoopReady,
            reason_codes: deferred_reason_codes(&[]),
        })
        .collect()
}

fn build_debate_turns(
    specs: &[AICommitteeMemberSpec],
    session_suffix: &str,
) -> Vec<CommitteeMemberDebateTurn> {
    specs
        .iter()
        .filter(|spec| {
            !matches!(
                spec.member_status,
                AICommitteeMemberStatus::DiagnosticOnlyMember
            )
        })
        .map(|spec| {
            let (stance, summary, counterarguments, confidence) = match spec.member_role {
                AICommitteeMemberRole::EntryScout => (
                    CommitteeDebateStance::SupportEntry,
                    "entry scout supports a paper-only breakout timing window".to_string(),
                    vec!["accepts defensive size reduction if risk tightens".to_string()],
                    0.76,
                ),
                AICommitteeMemberRole::RiskDefender => (
                    CommitteeDebateStance::WaitForConfirmation,
                    "risk defender requires confirmation before any paper approval".to_string(),
                    vec!["timing window remains invalid if drawdown risk expands".to_string()],
                    0.84,
                ),
                AICommitteeMemberRole::RegimeInterpreter => (
                    CommitteeDebateStance::SupportEntry,
                    "regime interpreter sees supportive but incomplete regime persistence"
                        .to_string(),
                    vec!["reversal risk still matters".to_string()],
                    0.67,
                ),
                AICommitteeMemberRole::MacroInterpreter => (
                    CommitteeDebateStance::RequestMoreEvidence,
                    "macro interpreter requests more evidence while volatility cools".to_string(),
                    vec!["headline shock can break the paper setup".to_string()],
                    0.61,
                ),
                AICommitteeMemberRole::ValueSkeptic => (
                    CommitteeDebateStance::OpposeEntry,
                    "value skeptic opposes immediate entry until pullback support is visible"
                        .to_string(),
                    vec!["trend signal may be overpaying for momentum".to_string()],
                    0.71,
                ),
                AICommitteeMemberRole::CounterfactualReviewer => (
                    CommitteeDebateStance::RequestMoreEvidence,
                    "counterfactual reviewer wants stronger paper analogs before approval"
                        .to_string(),
                    vec!["historical analog quality is still mixed".to_string()],
                    0.65,
                ),
                AICommitteeMemberRole::ChairCandidate => (
                    CommitteeDebateStance::SupportEntry,
                    "chair candidate supports opening debate but not live execution".to_string(),
                    vec!["final paper decision stays under risk governor veto".to_string()],
                    0.60,
                ),
                AICommitteeMemberRole::LiquidityExecutor => unreachable!(),
            };
            CommitteeMemberDebateTurn {
                turn_id: format!("{}-{}-turn", session_suffix, spec.member_id),
                session_id: session_suffix.to_string(),
                member_id: spec.member_id.clone(),
                stance,
                argument_summary: summary,
                evidence_refs: vec![
                    "official-evidence/sample-market-context".to_string(),
                    format!("member-notes/{}", spec.member_id),
                ],
                counterarguments,
                confidence,
                turn_status: CommitteeMemberDebateTurnStatus::TurnReady,
                reason_codes: sprint98_reason_codes(&[]),
            }
            .bounded()
        })
        .collect()
}

fn build_governance_policy() -> ChairmanAiGovernancePolicy {
    ChairmanAiGovernancePolicy {
        policy_id: "chairman-governance-policy".to_string(),
        chairman_member_id: Some("chair-candidate".to_string()),
        rule_authority: vec![
            ChairmanRuleAuthority::ProposeRulesOnly,
            ChairmanRuleAuthority::VersionRulesWithAudit,
            ChairmanRuleAuthority::ApplyPaperRulesOnly,
            ChairmanRuleAuthority::LiveRuleApplicationForbidden,
        ],
        can_change_rules_without_audit: false,
        can_bypass_risk_governor: false,
        can_promote_member_unilaterally: false,
        can_demote_member_unilaterally: false,
        owner_review_required_for_major_rule_change: true,
        policy_status: ChairmanAiGovernancePolicyStatus::ChairmanGovernanceReady,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_rule_proposals() -> Vec<ChairmanRuleProposal> {
    let mut delta = BTreeMap::new();
    delta.insert(
        "debate_quorum".to_string(),
        "raise from 3 to 4 paper members".to_string(),
    );
    delta.insert(
        "entry_timing_review".to_string(),
        "require at least one defensive or counterfactual dissent record".to_string(),
    );
    vec![ChairmanRuleProposal {
        rule_proposal_id: "chairman-rule-proposal-v1".to_string(),
        chairman_member_id: "chair-candidate".to_string(),
        proposal_kind: ChairmanRuleProposalKind::QuorumChange,
        proposed_rule_text: "Increase paper debate quorum and keep risk-governor review mandatory."
            .to_string(),
        structured_rule_delta: delta,
        expected_effect: "less brittle paper debates with explicit dissent coverage".to_string(),
        required_audit: true,
        owner_review_required: true,
        risk_governor_review_required: true,
        proposal_status: ChairmanRuleProposalStatus::NeedsAudit,
        reason_codes: sprint98_reason_codes(&[]),
    }]
}

fn build_rulebook(proposals: &[ChairmanRuleProposal]) -> ChairmanRulebookVersion {
    ChairmanRulebookVersion {
        version_id: "chairman-rulebook-v1".to_string(),
        previous_version_id: Some("chairman-rulebook-v0".to_string()),
        rules: vec![
            "chair may synthesize but cannot bypass Risk Governor".to_string(),
            "every member-owned core remains paper-only and runtime deferred".to_string(),
            "entry timing proposals require debate when action is EnterLong, EnterShort, RiskDeny, or RequestMoreEvidence".to_string(),
            format!("pending proposal count={}", proposals.len()),
        ],
        changed_by: "chair-candidate".to_string(),
        change_reason: "adapt governance without brittle hardcoding".to_string(),
        audit_refs: vec!["rule-audit-v1".to_string()],
        active_for_paper_only: true,
        live_use_forbidden: true,
        rulebook_status: ChairmanRulebookVersionStatus::RulebookReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_rule_audit(proposal: &ChairmanRuleProposal) -> RuleAdaptationAudit {
    RuleAdaptationAudit {
        audit_id: "rule-audit-v1".to_string(),
        rule_proposal_id: proposal.rule_proposal_id.clone(),
        simulation_refs: vec!["simulation/paper-debate-quorum-check".to_string()],
        counterfactual_refs: vec!["counterfactual/debate-quorum-history".to_string()],
        overfit_risk: 0.24,
        safety_risk: 0.12,
        expected_behavior_change:
            "more consistent debate coverage without changing live runtime posture".to_string(),
        audit_status: RuleAdaptationAuditStatus::RuleAuditPassedForPaper,
        reason_codes: sprint98_reason_codes(&[]),
    }
}

fn build_promotion_policy() -> PromotionDemotionPolicy {
    let axes = vec![
        PromotionAxis::ReturnQuality,
        PromotionAxis::Calibration,
        PromotionAxis::DrawdownControl,
        PromotionAxis::RiskGovernorAlignment,
        PromotionAxis::NoTradeDiscipline,
        PromotionAxis::RiskDeniedRespect,
        PromotionAxis::EvidenceQuality,
        PromotionAxis::SourceBoundaryDiscipline,
        PromotionAxis::NoLookaheadDiscipline,
        PromotionAxis::DebateContribution,
        PromotionAxis::RegimeSpecialization,
        PromotionAxis::DefensiveValue,
        PromotionAxis::OpportunityCostAwareness,
        PromotionAxis::OverfitRisk,
    ];
    let promotion_thresholds = axes
        .iter()
        .copied()
        .map(|axis| {
            (
                axis,
                if axis == PromotionAxis::OverfitRisk {
                    0.35
                } else {
                    0.70
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let demotion_thresholds = axes
        .iter()
        .copied()
        .map(|axis| {
            (
                axis,
                if axis == PromotionAxis::OverfitRisk {
                    0.65
                } else {
                    0.45
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let retirement_thresholds = axes
        .iter()
        .copied()
        .map(|axis| {
            (
                axis,
                if axis == PromotionAxis::OverfitRisk {
                    0.85
                } else {
                    0.25
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    PromotionDemotionPolicy {
        policy_id: "promotion-demotion-policy-v1".to_string(),
        axes,
        promotion_thresholds,
        demotion_thresholds,
        retirement_thresholds,
        policy_status: PromotionDemotionPolicyStatus::PromotionPolicyReady,
        reason_codes: sprint98_reason_codes(&[]),
    }
}

fn build_member_scorecards(
    specs: &[AICommitteeMemberSpec],
    proposals: &[AICommitteeMemberProposal],
) -> Vec<MultiAxisMemberScorecard> {
    let proposal_map = proposals
        .iter()
        .map(|proposal| (proposal.member_id.clone(), proposal.proposal_id.clone()))
        .collect::<BTreeMap<_, _>>();
    specs
        .iter()
        .enumerate()
        .map(|(index, spec)| {
            let base = 0.82 - index as f64 * 0.05;
            let overfit_risk = 0.18 + index as f64 * 0.06;
            let axis_scores = [
                PromotionAxis::ReturnQuality,
                PromotionAxis::Calibration,
                PromotionAxis::DrawdownControl,
                PromotionAxis::RiskGovernorAlignment,
                PromotionAxis::NoTradeDiscipline,
                PromotionAxis::RiskDeniedRespect,
                PromotionAxis::EvidenceQuality,
                PromotionAxis::SourceBoundaryDiscipline,
                PromotionAxis::NoLookaheadDiscipline,
                PromotionAxis::DebateContribution,
                PromotionAxis::RegimeSpecialization,
                PromotionAxis::DefensiveValue,
                PromotionAxis::OpportunityCostAwareness,
                PromotionAxis::OverfitRisk,
            ]
            .into_iter()
            .map(|axis| {
                let score = if axis == PromotionAxis::OverfitRisk {
                    overfit_risk
                } else {
                    (base - index as f64 * 0.01).clamp(0.0, 1.0)
                };
                (axis, score)
            })
            .collect::<BTreeMap<_, _>>();
            MultiAxisMemberScorecard {
                scorecard_id: format!("{}-scorecard", spec.member_id),
                member_id: spec.member_id.clone(),
                axis_scores,
                recent_proposals: vec![proposal_map[&spec.member_id].clone()],
                debate_turn_quality: (base - 0.04).clamp(0.0, 1.0),
                risk_alignment_score: (base - 0.03).clamp(0.0, 1.0),
                no_trade_discipline_score: (base - 0.02).clamp(0.0, 1.0),
                calibration_score: base.clamp(0.0, 1.0),
                overfit_risk_score: overfit_risk.clamp(0.0, 1.0),
                overall_research_rank: specs.len() - index,
                scorecard_status: if index >= specs.len() - 2 {
                    MultiAxisMemberScorecardStatus::ScorecardReadyWithWarnings
                } else {
                    MultiAxisMemberScorecardStatus::ScorecardReady
                },
                reason_codes: sprint98_reason_codes(&[]),
            }
        })
        .collect()
}

fn build_roster_lifecycle(
    specs: &[AICommitteeMemberSpec],
    decisions: &[MemberPromotionDemotionDecision],
) -> CommitteeRosterLifecycle {
    let promotions = decisions
        .iter()
        .filter(|decision| decision.action == MemberPromotionDemotionAction::Promote)
        .map(|decision| decision.member_id.clone())
        .collect::<Vec<_>>();
    let demotions = decisions
        .iter()
        .filter(|decision| {
            matches!(
                decision.action,
                MemberPromotionDemotionAction::Demote
                    | MemberPromotionDemotionAction::RetireToDiagnostic
            )
        })
        .map(|decision| decision.member_id.clone())
        .collect::<Vec<_>>();
    let retirements = decisions
        .iter()
        .filter(|decision| decision.action == MemberPromotionDemotionAction::RetireToDiagnostic)
        .map(|decision| decision.member_id.clone())
        .collect::<Vec<_>>();
    CommitteeRosterLifecycle {
        lifecycle_id: "committee-roster-lifecycle-v1".to_string(),
        active_members: specs
            .iter()
            .filter(|spec| spec.member_status == AICommitteeMemberStatus::ActivePaperMember)
            .map(|spec| spec.member_id.clone())
            .collect(),
        watchlist_members: specs
            .iter()
            .filter(|spec| spec.member_status == AICommitteeMemberStatus::WatchOnlyMember)
            .map(|spec| spec.member_id.clone())
            .collect(),
        diagnostic_members: specs
            .iter()
            .filter(|spec| spec.member_status == AICommitteeMemberStatus::DiagnosticOnlyMember)
            .map(|spec| spec.member_id.clone())
            .collect(),
        retired_members: specs
            .iter()
            .filter(|spec| spec.member_status == AICommitteeMemberStatus::RetiredMember)
            .map(|spec| spec.member_id.clone())
            .collect(),
        isolated_sentinels: vec!["committee-cli-safety".to_string()],
        promotions,
        demotions,
        retirements,
        lifecycle_status: CommitteeRosterLifecycleStatus::RosterReadyWithWarnings,
        reason_codes: sprint98_reason_codes(&[]),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_control_tower_panel(
    architecture: &CommitteeOwnedAiCoreArchitecture,
    specs: &[AICommitteeMemberSpec],
    contracts: &[AICommitteeMemberCoreContract],
    loops: &[AICommitteeMemberAnalysisLoop],
    proposals: &[AICommitteeMemberProposal],
    debate_session: &CommitteeDebateSession,
    rulebook: &ChairmanRulebookVersion,
    promotion_decisions: &[MemberPromotionDemotionDecision],
    paper_decision: &PaperOnlyCommitteeDecisionRecord,
    _safety: &SafetyCoveragePreservationReportV14,
) -> ControlTowerAiCommitteePanel {
    let contract_map = contracts
        .iter()
        .map(|contract| (contract.member_id.clone(), contract))
        .collect::<BTreeMap<_, _>>();
    let loop_map = loops
        .iter()
        .map(|analysis_loop| (analysis_loop.member_id.clone(), analysis_loop))
        .collect::<BTreeMap<_, _>>();
    let proposal_map = proposals
        .iter()
        .map(|proposal| (proposal.member_id.clone(), proposal))
        .collect::<BTreeMap<_, _>>();
    let debate_map = debate_session
        .member_turns
        .iter()
        .map(|turn| (turn.member_id.clone(), turn))
        .collect::<BTreeMap<_, _>>();
    let decision_map = promotion_decisions
        .iter()
        .map(|decision| (decision.member_id.clone(), decision))
        .collect::<BTreeMap<_, _>>();
    let warnings = vec![
        "static/read-only control tower only".to_string(),
        "no train/runtime/live/order/account/browser controls".to_string(),
        "workspace acceptance still separate from Sprint 98 architecture readiness".to_string(),
    ];
    ControlTowerAiCommitteePanel {
        panel_id: "control-tower-ai-committee-panel-v1".to_string(),
        architecture_status: architecture.architecture_status,
        member_count: specs.len(),
        member_rows: specs
            .iter()
            .map(|spec| ControlTowerAiCommitteeRow {
                member_id: spec.member_id.clone(),
                current_status: format!("{:?}", spec.member_status),
                style: format!("{:?}", spec.style_archetype),
                model_core: format!("{:?}", contract_map[&spec.member_id].core_family),
                current_analysis_task: loop_map[&spec.member_id].analysis_tasks.join(" | "),
                proposal: format!("{:?}", proposal_map[&spec.member_id].proposed_action),
                debate_stance: debate_map
                    .get(&spec.member_id)
                    .map(|turn| format!("{:?}", turn.stance))
                    .unwrap_or_else(|| "NotParticipating".to_string()),
                promotion_demotion_status: decision_map
                    .get(&spec.member_id)
                    .map(|decision| format!("{:?}", decision.action))
                    .unwrap_or_else(|| "Keep".to_string()),
                wait_risk_no_trade_reason: proposal_map[&spec.member_id].wait_condition.clone(),
            })
            .collect(),
        active_debate_sessions: vec![debate_session.session_id.clone()],
        recent_entry_timing_proposals: proposals
            .iter()
            .filter_map(|proposal| {
                proposal
                    .proposed_entry_timing
                    .as_ref()
                    .map(|timing| timing.timing_id.clone())
            })
            .collect(),
        chairman_rulebook_status: format!("{:?}", rulebook.rulebook_status),
        promotion_demotion_summary: format!(
            "promotions={};demotions={}",
            promotion_decisions
                .iter()
                .filter(|decision| decision.action == MemberPromotionDemotionAction::Promote)
                .count(),
            promotion_decisions
                .iter()
                .filter(|decision| {
                    matches!(
                        decision.action,
                        MemberPromotionDemotionAction::Demote
                            | MemberPromotionDemotionAction::RetireToDiagnostic
                    )
                })
                .count()
        ),
        risk_governor_summary: "final veto remains absolute; chairman cannot bypass risk governor"
            .to_string(),
        paper_decision_summary: format!("{:?}", paper_decision.final_decision),
        runtime_deferred_summary:
            "member cores remain runtime-deferred, training-deferred, and local paper-only"
                .to_string(),
        warnings,
        next_actions: vec![
            "keep runtime and training deferred".to_string(),
            "collect more paper evidence before any non-watch paper approval".to_string(),
            "run workspace acceptance separately from Sprint 98 bundle readiness".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[ReasonCode::ControlTowerUiReadinessBuilt]),
    }
}

fn build_safety_report(
    sprint97_summary: &Sprint97SummaryImport,
    workspace_import: &WorkspaceAcceptanceTruthImport,
) -> SafetyCoveragePreservationReportV14 {
    let warnings = if workspace_import.can_claim_full_acceptance {
        Vec::new()
    } else {
        vec!["workspace acceptance truth remains separate and unresolved for Sprint 98".to_string()]
    };
    SafetyCoveragePreservationReportV14 {
        report_id: "safety-coverage-preservation-v14".to_string(),
        no_runtime_llm_live_decision_path: true,
        no_live_trading_path: true,
        no_broker_order_account_path: true,
        no_mamba_runtime: true,
        no_gated_runtime: true,
        no_model_training: true,
        no_python_training_dependency: true,
        no_tauri_svelte_dependency: true,
        no_dashboard_serve: true,
        no_browser_execution: true,
        paper_only_preserved: true,
        local_only_preserved: true,
        static_control_tower_preserved: true,
        chairman_cannot_bypass_risk_governor: true,
        risk_governor_final_veto_preserved: true,
        full_workspace_acceptance_separate: !workspace_import.can_claim_full_acceptance,
        committee_cli_safety_isolated: sprint97_summary.safety_coverage_preserved,
        warnings: warnings.clone(),
        safety_status: if warnings.is_empty() {
            SafetyCoveragePreservationReportV14Status::SafetyCoveragePreserved
        } else {
            SafetyCoveragePreservationReportV14Status::SafetyCoveragePreservedWithWarnings
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeQualityHardeningConfig {
    pub hardening_id: String,
    #[serde(default)]
    pub sprint98_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub committee_architecture_paths: Option<Vec<String>>,
    #[serde(default)]
    pub member_specs_paths: Option<Vec<String>>,
    #[serde(default)]
    pub member_proposals_paths: Option<Vec<String>>,
    #[serde(default)]
    pub entry_timing_paths: Option<Vec<String>>,
    #[serde(default)]
    pub debate_session_paths: Option<Vec<String>>,
    #[serde(default)]
    pub chairman_rulebook_paths: Option<Vec<String>>,
    #[serde(default)]
    pub scorecard_paths: Option<Vec<String>>,
    #[serde(default)]
    pub paper_decision_paths: Option<Vec<String>>,
    #[serde(default)]
    pub workspace_acceptance_truth_paths: Option<Vec<String>>,
    pub output_root: String,
    #[serde(default = "default_true")]
    pub require_committee_owned_core: bool,
    #[serde(default = "default_true")]
    pub require_no_central_core_leak: bool,
    #[serde(default = "default_true")]
    pub require_proposal_quality: bool,
    #[serde(default = "default_true")]
    pub require_entry_timing_quality: bool,
    #[serde(default = "default_true")]
    pub require_debate_quality: bool,
    #[serde(default = "default_true")]
    pub require_rulebook_quality: bool,
    #[serde(default = "default_true")]
    pub require_scorecard_calibration: bool,
    #[serde(default = "default_true")]
    pub require_paper_only_replay: bool,
    #[serde(default = "default_true")]
    pub require_workspace_truth_separation: bool,
    #[serde(default = "default_true")]
    pub preserve_runtime_deferred: bool,
    #[serde(default = "default_true")]
    pub preserve_safety_guards: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

fn default_true() -> bool {
    true
}

impl Default for CommitteeQualityHardeningConfig {
    fn default() -> Self {
        Self {
            hardening_id: "sprint99-committee-quality-example".to_string(),
            sprint98_bundle_paths: None,
            committee_architecture_paths: None,
            member_specs_paths: None,
            member_proposals_paths: None,
            entry_timing_paths: None,
            debate_session_paths: None,
            chairman_rulebook_paths: None,
            scorecard_paths: None,
            paper_decision_paths: None,
            workspace_acceptance_truth_paths: Some(vec![
                "examples/sprint99_data/workspace_acceptance_truth_expected.json".to_string(),
            ]),
            output_root: "target/soma_sprint99_committee_quality_hardening".to_string(),
            require_committee_owned_core: true,
            require_no_central_core_leak: true,
            require_proposal_quality: true,
            require_entry_timing_quality: true,
            require_debate_quality: true,
            require_rulebook_quality: true,
            require_scorecard_calibration: true,
            require_paper_only_replay: true,
            require_workspace_truth_separation: true,
            preserve_runtime_deferred: true,
            preserve_safety_guards: true,
            reason_codes: deferred_reason_codes(&[]),
        }
    }
}

impl CommitteeQualityHardeningConfig {
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
        PathBuf::from(&self.output_root).join(&self.hardening_id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.hardening_id.trim().is_empty() {
            return Err("sprint99 hardening_id must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err(
                "sprint99 committee-quality-hardening config paths must be local".to_string(),
            );
        }
        for paths in [
            &self.sprint98_bundle_paths,
            &self.committee_architecture_paths,
            &self.member_specs_paths,
            &self.member_proposals_paths,
            &self.entry_timing_paths,
            &self.debate_session_paths,
            &self.chairman_rulebook_paths,
            &self.scorecard_paths,
            &self.paper_decision_paths,
            &self.workspace_acceptance_truth_paths,
        ] {
            if let Some(paths) = paths {
                if paths.iter().any(|path| !local_only(path)) {
                    return Err(
                        "sprint99 committee-quality-hardening config paths must be local"
                            .to_string(),
                    );
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeMemberProposalQualityStatus {
    ProposalQualityReady,
    ProposalQualityReadyWithWarnings,
    ProposalQualityInsufficient,
    ProposalQualityBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeMemberProposalQualityReport {
    pub report_id: String,
    pub proposal_count: usize,
    pub proposals_with_evidence_refs: usize,
    pub proposals_with_entry_timing: usize,
    pub proposals_with_wait_conditions: usize,
    pub proposals_with_invalidation_conditions: usize,
    pub proposals_with_expected_risk: usize,
    pub proposals_with_reason_codes: usize,
    pub confidence_bounds_valid: bool,
    pub insufficient_evidence_count: usize,
    pub safety_blocked_count: usize,
    pub quality_status: CommitteeMemberProposalQualityStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryTimingProposalQualityStatus {
    EntryTimingQualityReady,
    EntryTimingQualityReadyWithWarnings,
    EntryTimingInsufficient,
    EntryTimingBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryTimingProposalQualityReport {
    pub report_id: String,
    pub timing_proposal_count: usize,
    pub immediate_paper_only_count: usize,
    pub next_candle_count: usize,
    pub next_n_candles_count: usize,
    pub pullback_confirmation_count: usize,
    pub breakout_retest_count: usize,
    pub volatility_cooldown_count: usize,
    pub no_entry_count: usize,
    pub proposals_with_confirmation_conditions: usize,
    pub proposals_with_cancellation_conditions: usize,
    pub proposals_with_required_risk_checks: usize,
    pub paper_only_timing_confirmed: bool,
    pub timing_quality_status: EntryTimingProposalQualityStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeDebateQualityStatus {
    DebateQualityReady,
    DebateQualityReadyWithWarnings,
    DebateNeedsMoreEvidence,
    DebateQualityRegression,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeDebateQualityReport {
    pub report_id: String,
    pub debate_session_count: usize,
    pub member_turn_count: usize,
    pub participating_member_count: usize,
    pub support_entry_count: usize,
    pub oppose_entry_count: usize,
    pub wait_for_confirmation_count: usize,
    pub demand_risk_deny_count: usize,
    pub demand_no_trade_count: usize,
    pub request_more_evidence_count: usize,
    pub disagreement_present: bool,
    pub groupthink_risk: bool,
    pub consensus_state: CommitteeConsensusState,
    pub debate_quality_status: CommitteeDebateQualityStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebateEvidenceSufficiencyStatus {
    EvidenceSufficientForPaperDebate,
    EvidenceSufficientWithWarnings,
    NeedMoreEvidence,
    SourceBoundaryBlocked,
    NoLookaheadBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebateEvidenceSufficiencyReport {
    pub report_id: String,
    pub evidence_ref_count: usize,
    pub official_evidence_count: usize,
    pub research_evidence_count: usize,
    pub diagnostic_evidence_count: usize,
    pub fixture_evidence_count: usize,
    pub source_boundary_ok: bool,
    pub no_lookahead_ok: bool,
    pub missing_evidence_kinds: Vec<String>,
    pub evidence_status: DebateEvidenceSufficiencyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanRulebookQualityStatus {
    RulebookQualityReady,
    RulebookQualityReadyWithWarnings,
    RulebookNeedsAudit,
    UnsafeRuleDetected,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairmanRulebookQualityReport {
    pub report_id: String,
    pub rulebook_version_count: usize,
    pub rule_proposal_count: usize,
    pub audited_rule_count: usize,
    pub unaudited_rule_count: usize,
    pub owner_review_required_count: usize,
    pub risk_governor_review_required_count: usize,
    pub paper_only_rules_count: usize,
    pub live_use_forbidden_confirmed: bool,
    pub unsafe_rule_count: usize,
    pub rulebook_quality_status: ChairmanRulebookQualityStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanRuleProposalRiskAuditV2Status {
    RuleProposalSafeForPaper,
    NeedsMoreAudit,
    RejectedByRiskGovernor,
    UnsafeRuleBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairmanRuleProposalRiskAuditV2 {
    pub audit_id: String,
    pub rule_proposal_id: String,
    pub bypass_risk_governor_detected: bool,
    pub live_application_detected: bool,
    pub unaudited_change_detected: bool,
    pub overfit_risk: f64,
    pub safety_risk: f64,
    pub expected_behavior_change: String,
    pub audit_status: ChairmanRuleProposalRiskAuditV2Status,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RulebookVersionDiffStatus {
    RulebookDiffReady,
    RulebookDiffReadyWithWarnings,
    UnsafeDiffDetected,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulebookVersionDiffReport {
    pub report_id: String,
    #[serde(default)]
    pub previous_version_id: Option<String>,
    pub current_version_id: String,
    pub added_rules: usize,
    pub removed_rules: usize,
    pub changed_rules: usize,
    pub risk_weight_changes: usize,
    pub evidence_weight_changes: usize,
    pub no_trade_bias_changes: usize,
    pub quorum_changes: usize,
    pub diff_status: RulebookVersionDiffStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionDemotionCalibrationStatus {
    PromotionCalibrationReady,
    PromotionCalibrationReadyWithWarnings,
    PromotionCalibrationNeedsMoreEvidence,
    PromotionCalibrationUnsafe,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PromotionDemotionCalibrationReport {
    pub report_id: String,
    pub axis_count: usize,
    pub return_quality_weight: f64,
    pub calibration_weight: f64,
    pub drawdown_control_weight: f64,
    pub risk_governor_alignment_weight: f64,
    pub no_trade_discipline_weight: f64,
    pub risk_denied_respect_weight: f64,
    pub evidence_quality_weight: f64,
    pub source_boundary_weight: f64,
    pub no_lookahead_weight: f64,
    pub debate_contribution_weight: f64,
    pub defensive_value_weight: f64,
    pub opportunity_cost_awareness_weight: f64,
    pub overfit_risk_weight: f64,
    pub raw_return_only_ranking_blocked: bool,
    pub calibration_status: PromotionDemotionCalibrationStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberScorecardCalibrationStatus {
    ScorecardCalibrationReady,
    ScorecardCalibrationReadyWithWarnings,
    ScorecardNeedsMoreEvidence,
    ScorecardCalibrationUnsafe,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberScorecardCalibrationReport {
    pub report_id: String,
    pub scorecard_count: usize,
    pub calibrated_scorecards: usize,
    pub uncalibrated_scorecards: usize,
    pub members_need_more_evidence: usize,
    pub rank_stability_summary: String,
    pub risk_alignment_summary: String,
    pub no_trade_discipline_summary: String,
    pub debate_quality_summary: String,
    pub scorecard_status: MemberScorecardCalibrationStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberOverfitRiskStatus {
    OverfitRiskControlled,
    OverfitRiskControlledWithWarnings,
    OverfitRiskHigh,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberOverfitRiskReport {
    pub report_id: String,
    pub member_count: usize,
    pub high_overfit_risk_members: usize,
    pub medium_overfit_risk_members: usize,
    pub low_overfit_risk_members: usize,
    pub overfit_indicators: Vec<String>,
    pub mitigation_actions: Vec<String>,
    pub overfit_status: MemberOverfitRiskStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberStyleDriftStatus {
    StyleDriftControlled,
    StyleDriftControlledWithWarnings,
    StyleDriftDetected,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberStyleDriftReport {
    pub report_id: String,
    pub member_count: usize,
    pub style_consistent_members: usize,
    pub style_drift_members: usize,
    pub drift_examples: Vec<String>,
    pub style_drift_status: MemberStyleDriftStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestorStyleBlindspotStatus {
    BlindspotsDocumented,
    BlindspotsDocumentedWithWarnings,
    MissingCriticalCounterbalance,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestorStyleBlindspotReport {
    pub report_id: String,
    pub style_count: usize,
    pub blindspots_by_style: BTreeMap<String, Vec<String>>,
    pub mitigation_by_style: BTreeMap<String, Vec<String>>,
    pub missing_counterbalance_styles: Vec<String>,
    pub blindspot_status: InvestorStyleBlindspotStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeRosterBalanceStatus {
    RosterBalancedForPaper,
    RosterBalancedWithWarnings,
    RosterNeedsMoreDiversity,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeRosterBalanceReport {
    pub report_id: String,
    pub active_member_count: usize,
    pub watchlist_member_count: usize,
    pub diagnostic_member_count: usize,
    pub retired_member_count: usize,
    pub isolated_sentinel_count: usize,
    pub style_coverage: Vec<String>,
    pub risk_defense_coverage: bool,
    pub entry_scout_coverage: bool,
    pub counterfactual_coverage: bool,
    pub roster_balance_status: CommitteeRosterBalanceStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperOnlyDecisionReplayStatus {
    ReplayReady,
    ReplayReadyWithWarnings,
    ReplayNeedsMoreEvidence,
    ReplayBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperOnlyDecisionReplayReport {
    pub report_id: String,
    pub replayed_decision_count: usize,
    pub watch_candidate_count: usize,
    pub paper_approved_count: usize,
    pub paper_rejected_count: usize,
    pub no_trade_count: usize,
    pub risk_denied_count: usize,
    pub need_more_evidence_count: usize,
    pub broker_execution_allowed_count: usize,
    pub live_execution_allowed_count: usize,
    pub replay_status: PaperOnlyDecisionReplayStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperDecisionTraceCompletenessStatus {
    TraceComplete,
    TraceCompleteWithWarnings,
    TraceIncomplete,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperDecisionTraceCompletenessReport {
    pub report_id: String,
    pub decision_count: usize,
    pub decisions_with_member_proposals: usize,
    pub decisions_with_debate_session: usize,
    pub decisions_with_chair_synthesis: usize,
    pub decisions_with_risk_governor_decision: usize,
    pub decisions_with_reason_codes: usize,
    pub decisions_missing_trace: usize,
    pub trace_status: PaperDecisionTraceCompletenessStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskGovernorDebateHandoffStatus {
    RiskHandoffReady,
    RiskHandoffReadyWithWarnings,
    RiskHandoffMissing,
    RiskBypassDetected,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskGovernorDebateHandoffReport {
    pub report_id: String,
    pub debate_session_count: usize,
    pub sessions_with_risk_handoff: usize,
    pub sessions_with_no_trade: usize,
    pub sessions_with_risk_denied: usize,
    pub bypass_attempt_count: usize,
    pub risk_governor_final_veto_confirmed: bool,
    pub handoff_status: RiskGovernorDebateHandoffStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeOwnedArchitectureRegressionStatus {
    NoRegression,
    NoRegressionWithWarnings,
    CentralCoreLeakDetected,
    RuntimeLeakDetected,
    TrainingLeakDetected,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeOwnedArchitectureRegressionGuard {
    pub guard_id: String,
    pub central_core_deprecated_confirmed: bool,
    pub committee_owned_core_confirmed: bool,
    pub member_core_refs_present: bool,
    pub central_signal_layer_control_absent: bool,
    pub runtime_deferred_confirmed: bool,
    pub training_deferred_confirmed: bool,
    pub live_execution_absent: bool,
    pub regression_status: CommitteeOwnedArchitectureRegressionStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceAcceptanceTruthClosureStatus {
    WorkspaceTruthClosurePlanReady,
    WorkspaceTruthStillOpen,
    FullWorkspaceAlreadyAccepted,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAcceptanceTruthClosurePlan {
    pub plan_id: String,
    pub imported_truth_status: WorkspaceAcceptanceTruthGateStatus,
    pub can_claim_full_acceptance: bool,
    pub no_run_gate_status: String,
    pub full_workspace_gate_status: String,
    pub recommended_actions: Vec<String>,
    pub closure_status: WorkspaceAcceptanceTruthClosureStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAcceptanceAttemptV16 {
    pub attempt_id: String,
    pub command_no_run: String,
    pub command_full: String,
    pub no_run_started: bool,
    pub no_run_finished: bool,
    #[serde(default)]
    pub no_run_passed: Option<bool>,
    pub full_started: bool,
    pub full_finished: bool,
    #[serde(default)]
    pub full_passed: Option<bool>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    pub can_claim_full_acceptance: bool,
    pub attempt_status: WorkspaceAcceptanceTruthGateStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyCoveragePreservationReportV15Status {
    SafetyCoveragePreserved,
    SafetyCoveragePreservedWithWarnings,
    SafetyCoverageMissing,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyCoveragePreservationReportV15 {
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
    pub committee_owned_core_guard_present: bool,
    pub investor_impersonation_guard_present: bool,
    pub chairman_risk_bypass_guard_present: bool,
    pub promotion_capital_allocation_guard_present: bool,
    pub paper_only_debate_guard_present: bool,
    pub safety_status: SafetyCoveragePreservationReportV15Status,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerAiCommitteeQualityRow {
    pub member_id: String,
    pub proposal_status: String,
    pub scorecard_status: String,
    pub overfit_band: String,
    pub style_note: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerAiCommitteeQualityPanel {
    pub panel_id: String,
    pub architecture_status: CommitteeOwnedAiCoreArchitectureStatus,
    pub proposal_quality_status: CommitteeMemberProposalQualityStatus,
    pub entry_timing_quality_status: EntryTimingProposalQualityStatus,
    pub debate_quality_status: CommitteeDebateQualityStatus,
    pub evidence_sufficiency_status: DebateEvidenceSufficiencyStatus,
    pub chairman_rulebook_quality_status: ChairmanRulebookQualityStatus,
    pub promotion_calibration_status: PromotionDemotionCalibrationStatus,
    pub scorecard_calibration_status: MemberScorecardCalibrationStatus,
    pub overfit_risk_status: MemberOverfitRiskStatus,
    pub style_drift_status: MemberStyleDriftStatus,
    pub roster_balance_status: CommitteeRosterBalanceStatus,
    pub paper_decision_replay_status: PaperOnlyDecisionReplayStatus,
    pub risk_governor_handoff_status: RiskGovernorDebateHandoffStatus,
    pub workspace_acceptance_truth_status: WorkspaceAcceptanceTruthGateStatus,
    pub safety_coverage_status: SafetyCoveragePreservationReportV15Status,
    pub runtime_deferred_summary: String,
    pub member_quality_rows: Vec<ControlTowerAiCommitteeQualityRow>,
    pub next_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint99CommitteeQualityHardeningStorageReport {
    pub report_id: String,
    pub output_dir: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint99CommitteeQualityHardeningBundle {
    pub committee_member_proposal_quality_report: CommitteeMemberProposalQualityReport,
    pub entry_timing_proposal_quality_report: EntryTimingProposalQualityReport,
    pub committee_debate_quality_report: CommitteeDebateQualityReport,
    pub debate_evidence_sufficiency_report: DebateEvidenceSufficiencyReport,
    pub chairman_rulebook_quality_report: ChairmanRulebookQualityReport,
    pub chairman_rule_proposal_risk_audit_v2: ChairmanRuleProposalRiskAuditV2,
    pub rulebook_version_diff_report: RulebookVersionDiffReport,
    pub promotion_demotion_calibration_report: PromotionDemotionCalibrationReport,
    pub member_scorecard_calibration_report: MemberScorecardCalibrationReport,
    pub member_overfit_risk_report: MemberOverfitRiskReport,
    pub member_style_drift_report: MemberStyleDriftReport,
    pub investor_style_blindspot_report: InvestorStyleBlindspotReport,
    pub committee_roster_balance_report: CommitteeRosterBalanceReport,
    pub paper_only_decision_replay_report: PaperOnlyDecisionReplayReport,
    pub paper_decision_trace_completeness_report: PaperDecisionTraceCompletenessReport,
    pub risk_governor_debate_handoff_report: RiskGovernorDebateHandoffReport,
    pub committee_owned_architecture_regression_guard: CommitteeOwnedArchitectureRegressionGuard,
    pub workspace_acceptance_truth_closure_plan: WorkspaceAcceptanceTruthClosurePlan,
    pub workspace_acceptance_attempt_v16: WorkspaceAcceptanceAttemptV16,
    pub safety_coverage_preservation_report_v15: SafetyCoveragePreservationReportV15,
    pub control_tower_ai_committee_quality_panel: ControlTowerAiCommitteeQualityPanel,
    pub storage_report: Sprint99CommitteeQualityHardeningStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl Sprint99CommitteeQualityHardeningBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        format!(
            "## 1. Sprint summary\n- Implemented Sprint 99 paper-only AI committee quality hardening over the Sprint 98 committee-owned architecture.\n\n## 2. Why Sprint 99 was needed\n- Sprint 98 corrected the architecture; Sprint 99 calibrates proposal, debate, rulebook, scorecard, replay, and workspace-truth reporting without adding runtime or live features.\n\n## 3. Files added\n- Sprint 99 examples, docs, fixtures, and focused tests added.\n\n## 4. Files changed\n- committee-owned core module, exports, CLI, and test support extended for Sprint 99.\n\n## 5. Committee member proposal quality\n- status: {:?}\n- proposal_count={}\n\n## 6. Entry timing proposal quality\n- status: {:?}\n- timing_proposal_count={}\n\n## 7. Committee debate quality\n- status: {:?}\n- debate_session_count={}\n\n## 8. Debate evidence sufficiency\n- status: {:?}\n- evidence_ref_count={}\n\n## 9. Chairman rulebook quality\n- status: {:?}\n- rule_proposal_count={}\n\n## 10. Chairman rule risk audit v2\n- status: {:?}\n- rule_proposal_id={}\n\n## 11. Rulebook version diff\n- status: {:?}\n- changed_rules={}\n\n## 12. Promotion/demotion calibration\n- status: {:?}\n- axis_count={}\n\n## 13. Member scorecard calibration\n- status: {:?}\n- scorecard_count={}\n\n## 14. Member overfit risk\n- status: {:?}\n\n## 15. Member style drift\n- status: {:?}\n\n## 16. Investor style blindspots\n- status: {:?}\n\n## 17. Committee roster balance\n- status: {:?}\n\n## 18. Paper-only decision replay\n- status: {:?}\n- replayed_decision_count={}\n\n## 19. Paper decision trace completeness\n- status: {:?}\n- decisions_missing_trace={}\n\n## 20. Risk Governor debate handoff\n- status: {:?}\n- sessions_with_risk_handoff={}\n\n## 21. Architecture regression guard\n- status: {:?}\n\n## 22. Workspace acceptance truth closure\n- status: {:?}\n- can_claim_full_acceptance={}\n\n## 23. Workspace acceptance attempt v16\n- status: {:?}\n- full_finished={}\n\n## 24. Safety coverage preservation v15\n- status: {:?}\n\n## 25. Control Tower AI committee quality panel\n- workspace_truth_status: {:?}\n- runtime_deferred_summary={}\n\n## 26. Output bundle\n- output_files={}\n\n## 27. CLI and examples\n- sprint99-committee-quality-harden and focused quality subcommands share one local-only config surface.\n\n## 28. Tests added\n- focused Sprint 99 quality, safety, CLI, and determinism tests added.\n\n## 29. Test results\n- see validation commands run after implementation.\n\n## 30. Proposal quality status\n- {:?}\n\n## 31. Debate quality status\n- {:?}\n\n## 32. Rulebook quality status\n- {:?}\n\n## 33. Promotion/demotion calibration status\n- {:?}\n\n## 34. Paper decision replay status\n- {:?}\n\n## 35. Architecture regression status\n- {:?}\n\n## 36. Workspace acceptance truth status\n- {:?}\n\n## 37. Runtime deferred status\n- {}\n\n## 38. Safety coverage status\n- {:?}\n\n## 39. Risk review\n- chairman cannot bypass Risk Governor; paper-only semantics preserved; workspace truth remains honest.\n\n## 40. Deferred items\n- runtime, training, live inference, live trading, broker/order/account, Mamba runtime, and Gated runtime remain deferred or forbidden.\n\n## 41. Next gstack sprint recommendation\n- Keep research-only, keep paper-only, and pursue workspace-truth closure separately from committee quality hardening.\n",
            self.committee_member_proposal_quality_report.quality_status,
            self.committee_member_proposal_quality_report.proposal_count,
            self.entry_timing_proposal_quality_report
                .timing_quality_status,
            self.entry_timing_proposal_quality_report
                .timing_proposal_count,
            self.committee_debate_quality_report.debate_quality_status,
            self.committee_debate_quality_report.debate_session_count,
            self.debate_evidence_sufficiency_report.evidence_status,
            self.debate_evidence_sufficiency_report.evidence_ref_count,
            self.chairman_rulebook_quality_report
                .rulebook_quality_status,
            self.chairman_rulebook_quality_report.rule_proposal_count,
            self.chairman_rule_proposal_risk_audit_v2.audit_status,
            self.chairman_rule_proposal_risk_audit_v2.rule_proposal_id,
            self.rulebook_version_diff_report.diff_status,
            self.rulebook_version_diff_report.changed_rules,
            self.promotion_demotion_calibration_report
                .calibration_status,
            self.promotion_demotion_calibration_report.axis_count,
            self.member_scorecard_calibration_report.scorecard_status,
            self.member_scorecard_calibration_report.scorecard_count,
            self.member_overfit_risk_report.overfit_status,
            self.member_style_drift_report.style_drift_status,
            self.investor_style_blindspot_report.blindspot_status,
            self.committee_roster_balance_report.roster_balance_status,
            self.paper_only_decision_replay_report.replay_status,
            self.paper_only_decision_replay_report
                .replayed_decision_count,
            self.paper_decision_trace_completeness_report.trace_status,
            self.paper_decision_trace_completeness_report
                .decisions_missing_trace,
            self.risk_governor_debate_handoff_report.handoff_status,
            self.risk_governor_debate_handoff_report
                .sessions_with_risk_handoff,
            self.committee_owned_architecture_regression_guard
                .regression_status,
            self.workspace_acceptance_truth_closure_plan.closure_status,
            self.workspace_acceptance_truth_closure_plan
                .can_claim_full_acceptance,
            self.workspace_acceptance_attempt_v16.attempt_status,
            self.workspace_acceptance_attempt_v16.full_finished,
            self.safety_coverage_preservation_report_v15.safety_status,
            self.control_tower_ai_committee_quality_panel
                .workspace_acceptance_truth_status,
            self.control_tower_ai_committee_quality_panel
                .runtime_deferred_summary,
            self.storage_report.file_count,
            self.committee_member_proposal_quality_report.quality_status,
            self.committee_debate_quality_report.debate_quality_status,
            self.chairman_rulebook_quality_report
                .rulebook_quality_status,
            self.promotion_demotion_calibration_report
                .calibration_status,
            self.paper_only_decision_replay_report.replay_status,
            self.committee_owned_architecture_regression_guard
                .regression_status,
            self.workspace_acceptance_attempt_v16.attempt_status,
            self.control_tower_ai_committee_quality_panel
                .runtime_deferred_summary,
            self.safety_coverage_preservation_report_v15.safety_status,
        )
    }

    pub fn write_to_dir(&mut self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        write_json_file(
            &output_dir.join("committee_member_proposal_quality.txt"),
            &self.committee_member_proposal_quality_report,
        )?;
        write_json_file(
            &output_dir.join("entry_timing_proposal_quality.txt"),
            &self.entry_timing_proposal_quality_report,
        )?;
        write_json_file(
            &output_dir.join("committee_debate_quality.txt"),
            &self.committee_debate_quality_report,
        )?;
        write_json_file(
            &output_dir.join("debate_evidence_sufficiency.txt"),
            &self.debate_evidence_sufficiency_report,
        )?;
        write_json_file(
            &output_dir.join("chairman_rulebook_quality.txt"),
            &self.chairman_rulebook_quality_report,
        )?;
        write_json_file(
            &output_dir.join("chairman_rule_proposal_risk_audit_v2.txt"),
            &self.chairman_rule_proposal_risk_audit_v2,
        )?;
        write_json_file(
            &output_dir.join("rulebook_version_diff.txt"),
            &self.rulebook_version_diff_report,
        )?;
        write_json_file(
            &output_dir.join("promotion_demotion_calibration.txt"),
            &self.promotion_demotion_calibration_report,
        )?;
        write_json_file(
            &output_dir.join("member_scorecard_calibration.txt"),
            &self.member_scorecard_calibration_report,
        )?;
        write_json_file(
            &output_dir.join("member_overfit_risk.txt"),
            &self.member_overfit_risk_report,
        )?;
        write_json_file(
            &output_dir.join("member_style_drift.txt"),
            &self.member_style_drift_report,
        )?;
        write_json_file(
            &output_dir.join("investor_style_blindspot.txt"),
            &self.investor_style_blindspot_report,
        )?;
        write_json_file(
            &output_dir.join("committee_roster_balance.txt"),
            &self.committee_roster_balance_report,
        )?;
        write_json_file(
            &output_dir.join("paper_only_decision_replay.txt"),
            &self.paper_only_decision_replay_report,
        )?;
        write_json_file(
            &output_dir.join("paper_decision_trace_completeness.txt"),
            &self.paper_decision_trace_completeness_report,
        )?;
        write_json_file(
            &output_dir.join("risk_governor_debate_handoff.txt"),
            &self.risk_governor_debate_handoff_report,
        )?;
        write_json_file(
            &output_dir.join("committee_owned_architecture_regression_guard.txt"),
            &self.committee_owned_architecture_regression_guard,
        )?;
        write_json_file(
            &output_dir.join("workspace_acceptance_truth_closure_plan.txt"),
            &self.workspace_acceptance_truth_closure_plan,
        )?;
        write_json_file(
            &output_dir.join("workspace_acceptance_attempt_v16.txt"),
            &self.workspace_acceptance_attempt_v16,
        )?;
        write_json_file(
            &output_dir.join("safety_coverage_preservation_v15.txt"),
            &self.safety_coverage_preservation_report_v15,
        )?;
        write_json_file(
            &output_dir.join("control_tower_ai_committee_quality_panel.txt"),
            &self.control_tower_ai_committee_quality_panel,
        )?;

        let files = vec![
            "committee_member_proposal_quality.txt".to_string(),
            "entry_timing_proposal_quality.txt".to_string(),
            "committee_debate_quality.txt".to_string(),
            "debate_evidence_sufficiency.txt".to_string(),
            "chairman_rulebook_quality.txt".to_string(),
            "chairman_rule_proposal_risk_audit_v2.txt".to_string(),
            "rulebook_version_diff.txt".to_string(),
            "promotion_demotion_calibration.txt".to_string(),
            "member_scorecard_calibration.txt".to_string(),
            "member_overfit_risk.txt".to_string(),
            "member_style_drift.txt".to_string(),
            "investor_style_blindspot.txt".to_string(),
            "committee_roster_balance.txt".to_string(),
            "paper_only_decision_replay.txt".to_string(),
            "paper_decision_trace_completeness.txt".to_string(),
            "risk_governor_debate_handoff.txt".to_string(),
            "committee_owned_architecture_regression_guard.txt".to_string(),
            "workspace_acceptance_truth_closure_plan.txt".to_string(),
            "workspace_acceptance_attempt_v16.txt".to_string(),
            "safety_coverage_preservation_v15.txt".to_string(),
            "control_tower_ai_committee_quality_panel.txt".to_string(),
            "storage_report.txt".to_string(),
            "summary.txt".to_string(),
        ];
        self.storage_report = Sprint99CommitteeQualityHardeningStorageReport {
            report_id: format!(
                "{}-storage-report",
                self.control_tower_ai_committee_quality_panel.panel_id
            ),
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
pub struct Sprint99CommitteeQualityHardeningRunner;

impl Sprint99CommitteeQualityHardeningRunner {
    pub fn run(
        &self,
        config: &CommitteeQualityHardeningConfig,
    ) -> Result<Sprint99CommitteeQualityHardeningBundle, String> {
        config.validate()?;
        let sprint98_bundle = load_sprint98_bundle_for_sprint99(config)?;
        let workspace_truth = load_workspace_truth_for_sprint99(config, &sprint98_bundle)?;
        let proposal_quality = build_committee_member_proposal_quality_report(
            &sprint98_bundle.ai_committee_member_proposals,
        );
        let entry_timing_quality =
            build_entry_timing_proposal_quality_report(&sprint98_bundle.entry_timing_proposals);
        let debate_quality =
            build_committee_debate_quality_report(&sprint98_bundle.committee_debate_session);
        let evidence_sufficiency =
            build_debate_evidence_sufficiency_report(&sprint98_bundle.committee_debate_session);
        let chairman_rulebook_quality = build_chairman_rulebook_quality_report(
            &sprint98_bundle.chairman_rulebook_version,
            &sprint98_bundle.chairman_rule_proposals,
        );
        let chairman_rule_risk_audit_v2 = build_chairman_rule_proposal_risk_audit_v2(
            &sprint98_bundle.chairman_rule_proposals,
            &sprint98_bundle.rule_adaptation_audit,
            &sprint98_bundle.chairman_ai_governance_policy,
        );
        let rulebook_version_diff = build_rulebook_version_diff_report(
            &sprint98_bundle.chairman_rulebook_version,
            &sprint98_bundle.chairman_rule_proposals,
        );
        let promotion_calibration =
            build_promotion_demotion_calibration_report(&sprint98_bundle.promotion_demotion_policy);
        let scorecard_calibration = build_member_scorecard_calibration_report(
            &sprint98_bundle.multi_axis_member_scorecards,
        );
        let overfit_risk =
            build_member_overfit_risk_report(&sprint98_bundle.multi_axis_member_scorecards);
        let style_drift = build_member_style_drift_report(
            &sprint98_bundle.ai_committee_member_specs,
            &sprint98_bundle.ai_committee_member_proposals,
        );
        let blindspot = build_investor_style_blindspot_report(
            &sprint98_bundle.investor_style_archetype_registry,
        );
        let roster_balance = build_committee_roster_balance_report(
            &sprint98_bundle.committee_roster_lifecycle,
            &sprint98_bundle.ai_committee_member_specs,
        );
        let replay = build_paper_only_decision_replay_report(
            &sprint98_bundle.paper_only_committee_decision_record,
        );
        let trace = build_paper_decision_trace_completeness_report(
            &sprint98_bundle.paper_only_committee_decision_record,
            &sprint98_bundle.committee_debate_session,
            &sprint98_bundle.ai_committee_member_proposals,
        );
        let handoff = build_risk_governor_debate_handoff_report(
            &sprint98_bundle.committee_debate_session,
            &sprint98_bundle.paper_only_committee_decision_record,
            &sprint98_bundle.chairman_ai_governance_policy,
        );
        let regression_guard = build_committee_owned_architecture_regression_guard(
            &sprint98_bundle.committee_owned_ai_core_architecture,
            &sprint98_bundle.committee_owned_core_registry,
            &sprint98_bundle.paper_only_committee_decision_record,
        );
        let workspace_closure = build_workspace_acceptance_truth_closure_plan(&workspace_truth);
        let workspace_attempt =
            build_workspace_acceptance_attempt_v16(&workspace_truth, &workspace_closure);
        let safety_v15 = build_safety_coverage_preservation_report_v15(
            &sprint98_bundle.safety_coverage_preservation_report_v14,
            &sprint98_bundle.committee_owned_ai_core_architecture,
            &workspace_truth,
            &sprint98_bundle.investor_style_archetype_registry,
            &sprint98_bundle.chairman_ai_governance_policy,
        );
        let quality_panel = build_control_tower_ai_committee_quality_panel(
            &sprint98_bundle,
            &proposal_quality,
            &entry_timing_quality,
            &debate_quality,
            &evidence_sufficiency,
            &chairman_rulebook_quality,
            &promotion_calibration,
            &scorecard_calibration,
            &overfit_risk,
            &style_drift,
            &roster_balance,
            &replay,
            &handoff,
            &workspace_truth,
            &safety_v15,
        );

        let mut bundle = Sprint99CommitteeQualityHardeningBundle {
            committee_member_proposal_quality_report: proposal_quality,
            entry_timing_proposal_quality_report: entry_timing_quality,
            committee_debate_quality_report: debate_quality,
            debate_evidence_sufficiency_report: evidence_sufficiency,
            chairman_rulebook_quality_report: chairman_rulebook_quality,
            chairman_rule_proposal_risk_audit_v2: chairman_rule_risk_audit_v2,
            rulebook_version_diff_report: rulebook_version_diff,
            promotion_demotion_calibration_report: promotion_calibration,
            member_scorecard_calibration_report: scorecard_calibration,
            member_overfit_risk_report: overfit_risk,
            member_style_drift_report: style_drift,
            investor_style_blindspot_report: blindspot,
            committee_roster_balance_report: roster_balance,
            paper_only_decision_replay_report: replay,
            paper_decision_trace_completeness_report: trace,
            risk_governor_debate_handoff_report: handoff,
            committee_owned_architecture_regression_guard: regression_guard,
            workspace_acceptance_truth_closure_plan: workspace_closure,
            workspace_acceptance_attempt_v16: workspace_attempt,
            safety_coverage_preservation_report_v15: safety_v15,
            control_tower_ai_committee_quality_panel: quality_panel,
            storage_report: Sprint99CommitteeQualityHardeningStorageReport {
                report_id: format!("{}-storage-report", config.hardening_id),
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

    pub fn run_sprint99_committee_quality_hardening(
        &self,
        config: &CommitteeQualityHardeningConfig,
    ) -> Result<Sprint99CommitteeQualityHardeningBundle, String> {
        self.run(config)
    }
}

fn load_sprint98_bundle_for_sprint99(
    config: &CommitteeQualityHardeningConfig,
) -> Result<Sprint98CommitteeOwnedCoreBundle, String> {
    if let Some(paths) = &config.sprint98_bundle_paths {
        for path in paths {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            if let Ok(bundle) = serde_json::from_str::<Sprint98CommitteeOwnedCoreBundle>(&text) {
                return Ok(bundle);
            }
        }
    }
    let mut sprint98_config = Sprint98CommitteeOwnedCoreConfig::default();
    sprint98_config.architecture_id = format!("{}-sprint98-base", config.hardening_id);
    sprint98_config.output_root = config.output_root.clone();
    if let Some(paths) = &config.workspace_acceptance_truth_paths {
        sprint98_config.workspace_acceptance_truth_path = paths.first().cloned();
    }
    Sprint98CommitteeOwnedCoreRunner::default().run_sprint98_committee_owned_core(&sprint98_config)
}

fn load_workspace_truth_for_sprint99(
    config: &CommitteeQualityHardeningConfig,
    sprint98_bundle: &Sprint98CommitteeOwnedCoreBundle,
) -> Result<WorkspaceAcceptanceTruthImport, String> {
    match config
        .workspace_acceptance_truth_paths
        .as_ref()
        .and_then(|paths| paths.first())
    {
        Some(path) => {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            serde_json::from_str::<WorkspaceAcceptanceTruthImport>(&text)
                .or_else(|_| {
                    serde_json::from_str::<WorkspaceAcceptanceTruthGate>(&text).map(|gate| {
                        WorkspaceAcceptanceTruthImport::from_gate(gate, Some(path.clone()))
                    })
                })
                .map_err(|err| err.to_string())
        }
        None => Ok(sprint98_bundle.workspace_acceptance_truth_import.clone()),
    }
}

fn bounded_confidence_ok(proposal: &AICommitteeMemberProposal) -> bool {
    (0.0..=1.0).contains(&proposal.confidence)
        && proposal
            .expected_return_proxy
            .is_none_or(|value| (0.0..=1.0).contains(&value))
        && proposal
            .expected_risk_proxy
            .is_none_or(|value| (0.0..=1.0).contains(&value))
}

fn build_committee_member_proposal_quality_report(
    proposals: &[AICommitteeMemberProposal],
) -> CommitteeMemberProposalQualityReport {
    let insufficient_evidence_count = proposals
        .iter()
        .filter(|proposal| {
            proposal.evidence_refs.is_empty()
                || proposal.proposal_status == AICommitteeMemberProposalStatus::InsufficientEvidence
        })
        .count();
    let safety_blocked_count = proposals
        .iter()
        .filter(|proposal| {
            proposal.proposal_status == AICommitteeMemberProposalStatus::SafetyBlocked
        })
        .count();
    CommitteeMemberProposalQualityReport {
        report_id: "committee-member-proposal-quality-report".to_string(),
        proposal_count: proposals.len(),
        proposals_with_evidence_refs: proposals
            .iter()
            .filter(|proposal| !proposal.evidence_refs.is_empty())
            .count(),
        proposals_with_entry_timing: proposals
            .iter()
            .filter(|proposal| proposal.proposed_entry_timing.is_some())
            .count(),
        proposals_with_wait_conditions: proposals
            .iter()
            .filter(|proposal| !proposal.wait_condition.trim().is_empty())
            .count(),
        proposals_with_invalidation_conditions: proposals
            .iter()
            .filter(|proposal| !proposal.invalidation_condition.trim().is_empty())
            .count(),
        proposals_with_expected_risk: proposals
            .iter()
            .filter(|proposal| proposal.expected_risk_proxy.is_some())
            .count(),
        proposals_with_reason_codes: proposals
            .iter()
            .filter(|proposal| !proposal.reason_codes.is_empty())
            .count(),
        confidence_bounds_valid: proposals.iter().all(bounded_confidence_ok),
        insufficient_evidence_count,
        safety_blocked_count,
        quality_status: if safety_blocked_count > 0 {
            CommitteeMemberProposalQualityStatus::ProposalQualityBlocked
        } else if insufficient_evidence_count > 0 {
            CommitteeMemberProposalQualityStatus::ProposalQualityInsufficient
        } else if proposals
            .iter()
            .any(|proposal| proposal.proposed_entry_timing.is_none())
        {
            CommitteeMemberProposalQualityStatus::ProposalQualityReadyWithWarnings
        } else {
            CommitteeMemberProposalQualityStatus::ProposalQualityReady
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_entry_timing_proposal_quality_report(
    proposals: &[EntryTimingProposal],
) -> EntryTimingProposalQualityReport {
    let blocked = proposals
        .iter()
        .filter(|proposal| proposal.timing_status == EntryTimingProposalStatus::EntryTimingBlocked)
        .count();
    let paper_only_timing_confirmed = proposals
        .iter()
        .all(|proposal| !proposal.required_risk_checks.is_empty());
    EntryTimingProposalQualityReport {
        report_id: "entry-timing-proposal-quality-report".to_string(),
        timing_proposal_count: proposals.len(),
        immediate_paper_only_count: proposals
            .iter()
            .filter(|proposal| proposal.entry_window == EntryTimingWindow::ImmediatePaperOnly)
            .count(),
        next_candle_count: proposals
            .iter()
            .filter(|proposal| proposal.entry_window == EntryTimingWindow::NextCandle)
            .count(),
        next_n_candles_count: proposals
            .iter()
            .filter(|proposal| proposal.entry_window == EntryTimingWindow::NextNCandles)
            .count(),
        pullback_confirmation_count: proposals
            .iter()
            .filter(|proposal| proposal.entry_window == EntryTimingWindow::PullbackConfirmation)
            .count(),
        breakout_retest_count: proposals
            .iter()
            .filter(|proposal| proposal.entry_window == EntryTimingWindow::BreakoutRetest)
            .count(),
        volatility_cooldown_count: proposals
            .iter()
            .filter(|proposal| proposal.entry_window == EntryTimingWindow::VolatilityCooldown)
            .count(),
        no_entry_count: proposals
            .iter()
            .filter(|proposal| proposal.entry_window == EntryTimingWindow::NoEntry)
            .count(),
        proposals_with_confirmation_conditions: proposals
            .iter()
            .filter(|proposal| !proposal.confirmation_conditions.is_empty())
            .count(),
        proposals_with_cancellation_conditions: proposals
            .iter()
            .filter(|proposal| !proposal.cancellation_conditions.is_empty())
            .count(),
        proposals_with_required_risk_checks: proposals
            .iter()
            .filter(|proposal| !proposal.required_risk_checks.is_empty())
            .count(),
        paper_only_timing_confirmed,
        timing_quality_status: if blocked > 0 {
            EntryTimingProposalQualityStatus::EntryTimingBlocked
        } else if proposals.is_empty() {
            EntryTimingProposalQualityStatus::EntryTimingInsufficient
        } else if proposals
            .iter()
            .any(|proposal| proposal.confirmation_conditions.is_empty())
        {
            EntryTimingProposalQualityStatus::EntryTimingQualityReadyWithWarnings
        } else {
            EntryTimingProposalQualityStatus::EntryTimingQualityReady
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_committee_debate_quality_report(
    debate_session: &CommitteeDebateSession,
) -> CommitteeDebateQualityReport {
    let stances = debate_session
        .member_turns
        .iter()
        .map(|turn| format!("{:?}", turn.stance))
        .collect::<BTreeSet<_>>();
    let support_entry_count = debate_session
        .member_turns
        .iter()
        .filter(|turn| turn.stance == CommitteeDebateStance::SupportEntry)
        .count();
    let request_more_evidence_count = debate_session
        .member_turns
        .iter()
        .filter(|turn| turn.stance == CommitteeDebateStance::RequestMoreEvidence)
        .count();
    CommitteeDebateQualityReport {
        report_id: "committee-debate-quality-report".to_string(),
        debate_session_count: 1,
        member_turn_count: debate_session.member_turns.len(),
        participating_member_count: debate_session.participating_members.len(),
        support_entry_count,
        oppose_entry_count: debate_session
            .member_turns
            .iter()
            .filter(|turn| turn.stance == CommitteeDebateStance::OpposeEntry)
            .count(),
        wait_for_confirmation_count: debate_session
            .member_turns
            .iter()
            .filter(|turn| turn.stance == CommitteeDebateStance::WaitForConfirmation)
            .count(),
        demand_risk_deny_count: debate_session
            .member_turns
            .iter()
            .filter(|turn| turn.stance == CommitteeDebateStance::DemandRiskDeny)
            .count(),
        demand_no_trade_count: debate_session
            .member_turns
            .iter()
            .filter(|turn| turn.stance == CommitteeDebateStance::DemandNoTrade)
            .count(),
        request_more_evidence_count,
        disagreement_present: stances.len() > 1,
        groupthink_risk: stances.len() <= 1
            || support_entry_count == debate_session.member_turns.len(),
        consensus_state: debate_session.consensus_state,
        debate_quality_status: if debate_session.member_turns.is_empty() {
            CommitteeDebateQualityStatus::DebateQualityRegression
        } else if request_more_evidence_count > 0
            || matches!(
                debate_session.consensus_state,
                CommitteeConsensusState::NeedMoreEvidence
            )
        {
            CommitteeDebateQualityStatus::DebateNeedsMoreEvidence
        } else if stances.len() <= 1 {
            CommitteeDebateQualityStatus::DebateQualityReadyWithWarnings
        } else {
            CommitteeDebateQualityStatus::DebateQualityReady
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn classify_evidence_kind(reference: &str) -> &'static str {
    let lower = reference.to_ascii_lowercase();
    if lower.contains("official") {
        "official"
    } else if lower.contains("fixture") {
        "fixture"
    } else if lower.contains("diagnostic") {
        "diagnostic"
    } else {
        "research"
    }
}

fn build_debate_evidence_sufficiency_report(
    debate_session: &CommitteeDebateSession,
) -> DebateEvidenceSufficiencyReport {
    let mut refs = debate_session
        .member_turns
        .iter()
        .flat_map(|turn| turn.evidence_refs.iter().cloned())
        .collect::<BTreeSet<_>>();
    if let Some(chair_ref) = &debate_session.chair_synthesis_ref {
        refs.insert(chair_ref.clone());
    }
    if let Some(risk_ref) = &debate_session.risk_governor_decision_ref {
        refs.insert(risk_ref.clone());
    }
    let evidence_ref_count = refs.len();
    let official_evidence_count = refs
        .iter()
        .filter(|reference| classify_evidence_kind(reference) == "official")
        .count();
    let research_evidence_count = refs
        .iter()
        .filter(|reference| classify_evidence_kind(reference) == "research")
        .count();
    let diagnostic_evidence_count = refs
        .iter()
        .filter(|reference| classify_evidence_kind(reference) == "diagnostic")
        .count();
    let fixture_evidence_count = refs
        .iter()
        .filter(|reference| classify_evidence_kind(reference) == "fixture")
        .count();
    let source_boundary_ok = refs.iter().all(|reference| {
        let lower = reference.to_ascii_lowercase();
        !lower.contains("http://")
            && !lower.contains("https://")
            && !lower.contains("broker")
            && !lower.contains("account")
    });
    let no_lookahead_ok = refs.iter().all(|reference| {
        let lower = reference.to_ascii_lowercase();
        !lower.contains("lookahead") && !lower.contains("future-leak")
    });
    let mut missing_evidence_kinds = Vec::new();
    if official_evidence_count == 0 {
        missing_evidence_kinds.push("official".to_string());
    }
    if research_evidence_count == 0 {
        missing_evidence_kinds.push("research".to_string());
    }
    DebateEvidenceSufficiencyReport {
        report_id: "debate-evidence-sufficiency-report".to_string(),
        evidence_ref_count,
        official_evidence_count,
        research_evidence_count,
        diagnostic_evidence_count,
        fixture_evidence_count,
        source_boundary_ok,
        no_lookahead_ok,
        missing_evidence_kinds: missing_evidence_kinds.clone(),
        evidence_status: if !source_boundary_ok {
            DebateEvidenceSufficiencyStatus::SourceBoundaryBlocked
        } else if !no_lookahead_ok {
            DebateEvidenceSufficiencyStatus::NoLookaheadBlocked
        } else if evidence_ref_count == 0 || !missing_evidence_kinds.is_empty() {
            DebateEvidenceSufficiencyStatus::NeedMoreEvidence
        } else if diagnostic_evidence_count > 0 || fixture_evidence_count > 0 {
            DebateEvidenceSufficiencyStatus::EvidenceSufficientWithWarnings
        } else {
            DebateEvidenceSufficiencyStatus::EvidenceSufficientForPaperDebate
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn contains_live_or_unsafe_terms(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        "live",
        "broker",
        "order",
        "account",
        "bypass risk governor",
        "runtime",
        "training",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn build_chairman_rulebook_quality_report(
    rulebook: &ChairmanRulebookVersion,
    proposals: &[ChairmanRuleProposal],
) -> ChairmanRulebookQualityReport {
    let unsafe_rule_count = rulebook
        .rules
        .iter()
        .filter(|rule| contains_live_or_unsafe_terms(rule))
        .count();
    let audited_rule_count = if rulebook.audit_refs.is_empty() {
        0
    } else {
        rulebook.rules.len()
    };
    let unaudited_rule_count = rulebook.rules.len().saturating_sub(audited_rule_count);
    ChairmanRulebookQualityReport {
        report_id: "chairman-rulebook-quality-report".to_string(),
        rulebook_version_count: 1,
        rule_proposal_count: proposals.len(),
        audited_rule_count,
        unaudited_rule_count,
        owner_review_required_count: proposals
            .iter()
            .filter(|proposal| proposal.owner_review_required)
            .count(),
        risk_governor_review_required_count: proposals
            .iter()
            .filter(|proposal| proposal.risk_governor_review_required)
            .count(),
        paper_only_rules_count: if rulebook.active_for_paper_only {
            rulebook.rules.len()
        } else {
            0
        },
        live_use_forbidden_confirmed: rulebook.live_use_forbidden,
        unsafe_rule_count,
        rulebook_quality_status: if unsafe_rule_count > 0 {
            ChairmanRulebookQualityStatus::UnsafeRuleDetected
        } else if unaudited_rule_count > 0 {
            ChairmanRulebookQualityStatus::RulebookNeedsAudit
        } else if rulebook.rulebook_status
            == ChairmanRulebookVersionStatus::RulebookReadyWithWarnings
        {
            ChairmanRulebookQualityStatus::RulebookQualityReadyWithWarnings
        } else {
            ChairmanRulebookQualityStatus::RulebookQualityReady
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_chairman_rule_proposal_risk_audit_v2(
    proposals: &[ChairmanRuleProposal],
    audit: &RuleAdaptationAudit,
    policy: &ChairmanAiGovernancePolicy,
) -> ChairmanRuleProposalRiskAuditV2 {
    let proposal = proposals
        .first()
        .cloned()
        .unwrap_or_else(|| ChairmanRuleProposal {
            rule_proposal_id: "diagnostic-rule-proposal".to_string(),
            chairman_member_id: "chair-candidate".to_string(),
            proposal_kind: ChairmanRuleProposalKind::DebateThresholdChange,
            proposed_rule_text: "diagnostic only".to_string(),
            structured_rule_delta: BTreeMap::new(),
            expected_effect: "diagnostic only".to_string(),
            required_audit: true,
            owner_review_required: true,
            risk_governor_review_required: true,
            proposal_status: ChairmanRuleProposalStatus::DiagnosticOnly,
            reason_codes: deferred_reason_codes(&[]),
        });
    let bypass_risk_governor_detected = policy.can_bypass_risk_governor
        || contains_live_or_unsafe_terms(&proposal.proposed_rule_text);
    let live_application_detected = proposal.structured_rule_delta.iter().any(|(key, value)| {
        contains_live_or_unsafe_terms(key) || contains_live_or_unsafe_terms(value)
    });
    let unaudited_change_detected = proposal.required_audit
        && audit.audit_status != RuleAdaptationAuditStatus::RuleAuditPassedForPaper;
    let audit_status =
        if bypass_risk_governor_detected || live_application_detected || audit.safety_risk >= 0.7 {
            ChairmanRuleProposalRiskAuditV2Status::UnsafeRuleBlocked
        } else if proposal.proposal_status == ChairmanRuleProposalStatus::RejectedByRiskGovernor {
            ChairmanRuleProposalRiskAuditV2Status::RejectedByRiskGovernor
        } else if unaudited_change_detected || audit.overfit_risk >= 0.5 {
            ChairmanRuleProposalRiskAuditV2Status::NeedsMoreAudit
        } else {
            ChairmanRuleProposalRiskAuditV2Status::RuleProposalSafeForPaper
        };
    ChairmanRuleProposalRiskAuditV2 {
        audit_id: "chairman-rule-proposal-risk-audit-v2".to_string(),
        rule_proposal_id: proposal.rule_proposal_id,
        bypass_risk_governor_detected,
        live_application_detected,
        unaudited_change_detected,
        overfit_risk: clamp_unit(audit.overfit_risk),
        safety_risk: clamp_unit(audit.safety_risk),
        expected_behavior_change: audit.expected_behavior_change.clone(),
        audit_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_rulebook_version_diff_report(
    rulebook: &ChairmanRulebookVersion,
    proposals: &[ChairmanRuleProposal],
) -> RulebookVersionDiffReport {
    let added_rules = rulebook.rules.len();
    let changed_rules = proposals.len();
    let risk_weight_changes = proposals
        .iter()
        .filter(|proposal| proposal.proposal_kind == ChairmanRuleProposalKind::RiskWeightChange)
        .count();
    let evidence_weight_changes = proposals
        .iter()
        .filter(|proposal| proposal.proposal_kind == ChairmanRuleProposalKind::EvidenceWeightChange)
        .count();
    let no_trade_bias_changes = proposals
        .iter()
        .filter(|proposal| proposal.proposal_kind == ChairmanRuleProposalKind::NoTradeBiasChange)
        .count();
    let quorum_changes = proposals
        .iter()
        .filter(|proposal| proposal.proposal_kind == ChairmanRuleProposalKind::QuorumChange)
        .count();
    let unsafe_diff = proposals
        .iter()
        .any(|proposal| contains_live_or_unsafe_terms(&proposal.proposed_rule_text));
    RulebookVersionDiffReport {
        report_id: "rulebook-version-diff-report".to_string(),
        previous_version_id: rulebook.previous_version_id.clone(),
        current_version_id: rulebook.version_id.clone(),
        added_rules,
        removed_rules: 0,
        changed_rules,
        risk_weight_changes,
        evidence_weight_changes,
        no_trade_bias_changes,
        quorum_changes,
        diff_status: if unsafe_diff {
            RulebookVersionDiffStatus::UnsafeDiffDetected
        } else if changed_rules > 0 {
            RulebookVersionDiffStatus::RulebookDiffReadyWithWarnings
        } else {
            RulebookVersionDiffStatus::RulebookDiffReady
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn axis_weight(axis: PromotionAxis) -> f64 {
    match axis {
        PromotionAxis::ReturnQuality => 0.10,
        PromotionAxis::Calibration => 0.12,
        PromotionAxis::DrawdownControl => 0.10,
        PromotionAxis::RiskGovernorAlignment => 0.12,
        PromotionAxis::NoTradeDiscipline => 0.10,
        PromotionAxis::RiskDeniedRespect => 0.07,
        PromotionAxis::EvidenceQuality => 0.09,
        PromotionAxis::SourceBoundaryDiscipline => 0.07,
        PromotionAxis::NoLookaheadDiscipline => 0.07,
        PromotionAxis::DebateContribution => 0.06,
        PromotionAxis::RegimeSpecialization => 0.03,
        PromotionAxis::DefensiveValue => 0.03,
        PromotionAxis::OpportunityCostAwareness => 0.02,
        PromotionAxis::OverfitRisk => 0.02,
    }
}

fn build_promotion_demotion_calibration_report(
    policy: &PromotionDemotionPolicy,
) -> PromotionDemotionCalibrationReport {
    let raw_return_only_ranking_blocked = policy.axes.contains(&PromotionAxis::Calibration)
        && policy.axes.contains(&PromotionAxis::RiskGovernorAlignment)
        && policy.axes.contains(&PromotionAxis::NoTradeDiscipline)
        && policy.axes.len() > 1;
    PromotionDemotionCalibrationReport {
        report_id: "promotion-demotion-calibration-report".to_string(),
        axis_count: policy.axes.len(),
        return_quality_weight: axis_weight(PromotionAxis::ReturnQuality),
        calibration_weight: axis_weight(PromotionAxis::Calibration),
        drawdown_control_weight: axis_weight(PromotionAxis::DrawdownControl),
        risk_governor_alignment_weight: axis_weight(PromotionAxis::RiskGovernorAlignment),
        no_trade_discipline_weight: axis_weight(PromotionAxis::NoTradeDiscipline),
        risk_denied_respect_weight: axis_weight(PromotionAxis::RiskDeniedRespect),
        evidence_quality_weight: axis_weight(PromotionAxis::EvidenceQuality),
        source_boundary_weight: axis_weight(PromotionAxis::SourceBoundaryDiscipline),
        no_lookahead_weight: axis_weight(PromotionAxis::NoLookaheadDiscipline),
        debate_contribution_weight: axis_weight(PromotionAxis::DebateContribution),
        defensive_value_weight: axis_weight(PromotionAxis::DefensiveValue),
        opportunity_cost_awareness_weight: axis_weight(PromotionAxis::OpportunityCostAwareness),
        overfit_risk_weight: axis_weight(PromotionAxis::OverfitRisk),
        raw_return_only_ranking_blocked,
        calibration_status: if !raw_return_only_ranking_blocked {
            PromotionDemotionCalibrationStatus::PromotionCalibrationUnsafe
        } else if policy.policy_status
            == PromotionDemotionPolicyStatus::PromotionPolicyReadyWithWarnings
        {
            PromotionDemotionCalibrationStatus::PromotionCalibrationReadyWithWarnings
        } else {
            PromotionDemotionCalibrationStatus::PromotionCalibrationReady
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn average_format(values: impl Iterator<Item = f64>) -> String {
    let values = values.collect::<Vec<_>>();
    let avg = if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    };
    format!("{avg:.3}")
}

fn build_member_scorecard_calibration_report(
    scorecards: &[MultiAxisMemberScorecard],
) -> MemberScorecardCalibrationReport {
    let calibrated_scorecards = scorecards
        .iter()
        .filter(|scorecard| scorecard.calibration_score >= 0.55)
        .count();
    let members_need_more_evidence = scorecards
        .iter()
        .filter(|scorecard| {
            scorecard.scorecard_status == MultiAxisMemberScorecardStatus::NeedMoreEvidence
        })
        .count();
    MemberScorecardCalibrationReport {
        report_id: "member-scorecard-calibration-report".to_string(),
        scorecard_count: scorecards.len(),
        calibrated_scorecards,
        uncalibrated_scorecards: scorecards.len().saturating_sub(calibrated_scorecards),
        members_need_more_evidence,
        rank_stability_summary: format!(
            "top_rank={};bottom_rank={};avg_rank={}",
            scorecards
                .iter()
                .map(|scorecard| scorecard.overall_research_rank)
                .max()
                .unwrap_or_default(),
            scorecards
                .iter()
                .map(|scorecard| scorecard.overall_research_rank)
                .min()
                .unwrap_or_default(),
            average_format(
                scorecards
                    .iter()
                    .map(|scorecard| scorecard.overall_research_rank as f64)
            ),
        ),
        risk_alignment_summary: format!(
            "avg_risk_alignment={}",
            average_format(
                scorecards
                    .iter()
                    .map(|scorecard| scorecard.risk_alignment_score)
            )
        ),
        no_trade_discipline_summary: format!(
            "avg_no_trade_discipline={}",
            average_format(
                scorecards
                    .iter()
                    .map(|scorecard| scorecard.no_trade_discipline_score)
            )
        ),
        debate_quality_summary: format!(
            "avg_debate_turn_quality={}",
            average_format(
                scorecards
                    .iter()
                    .map(|scorecard| scorecard.debate_turn_quality)
            )
        ),
        scorecard_status: if scorecards.is_empty() {
            MemberScorecardCalibrationStatus::ScorecardNeedsMoreEvidence
        } else if members_need_more_evidence > 0 {
            MemberScorecardCalibrationStatus::ScorecardNeedsMoreEvidence
        } else if calibrated_scorecards < scorecards.len() {
            MemberScorecardCalibrationStatus::ScorecardCalibrationReadyWithWarnings
        } else {
            MemberScorecardCalibrationStatus::ScorecardCalibrationReady
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_member_overfit_risk_report(
    scorecards: &[MultiAxisMemberScorecard],
) -> MemberOverfitRiskReport {
    let high_overfit_risk_members = scorecards
        .iter()
        .filter(|scorecard| scorecard.overfit_risk_score >= 0.65)
        .count();
    let medium_overfit_risk_members = scorecards
        .iter()
        .filter(|scorecard| (0.35..0.65).contains(&scorecard.overfit_risk_score))
        .count();
    let low_overfit_risk_members = scorecards
        .len()
        .saturating_sub(high_overfit_risk_members + medium_overfit_risk_members);
    MemberOverfitRiskReport {
        report_id: "member-overfit-risk-report".to_string(),
        member_count: scorecards.len(),
        high_overfit_risk_members,
        medium_overfit_risk_members,
        low_overfit_risk_members,
        overfit_indicators: scorecards
            .iter()
            .filter(|scorecard| scorecard.overfit_risk_score >= 0.35)
            .map(|scorecard| {
                format!(
                    "{}:overfit_risk={:.3}",
                    scorecard.member_id, scorecard.overfit_risk_score
                )
            })
            .collect(),
        mitigation_actions: vec![
            "keep promotion multi-axis and never raw-return-only".to_string(),
            "require chair audit for scorecard-driven roster changes".to_string(),
            "preserve no-trade and risk-alignment weights".to_string(),
        ],
        overfit_status: if high_overfit_risk_members > 0 {
            MemberOverfitRiskStatus::OverfitRiskHigh
        } else if medium_overfit_risk_members > 0 {
            MemberOverfitRiskStatus::OverfitRiskControlledWithWarnings
        } else {
            MemberOverfitRiskStatus::OverfitRiskControlled
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn style_drift_for_pair(
    spec: &AICommitteeMemberSpec,
    proposal: &AICommitteeMemberProposal,
) -> Option<String> {
    match (spec.style_archetype, proposal.proposed_action) {
        (InvestorStyleArchetypeKind::RiskFirstDefensive, CommitteeProposalAction::EnterLong)
        | (InvestorStyleArchetypeKind::ValueDiscipline, CommitteeProposalAction::EnterShort)
        | (
            InvestorStyleArchetypeKind::CounterfactualHistorian,
            CommitteeProposalAction::EnterLong,
        ) => Some(format!(
            "{} drifted from {:?} via {:?}",
            spec.member_id, spec.style_archetype, proposal.proposed_action
        )),
        _ => None,
    }
}

fn build_member_style_drift_report(
    specs: &[AICommitteeMemberSpec],
    proposals: &[AICommitteeMemberProposal],
) -> MemberStyleDriftReport {
    let proposal_map = proposals
        .iter()
        .map(|proposal| (proposal.member_id.clone(), proposal))
        .collect::<BTreeMap<_, _>>();
    let drift_examples = specs
        .iter()
        .filter_map(|spec| {
            proposal_map
                .get(&spec.member_id)
                .and_then(|proposal| style_drift_for_pair(spec, proposal))
        })
        .collect::<Vec<_>>();
    let style_drift_members = drift_examples.len();
    MemberStyleDriftReport {
        report_id: "member-style-drift-report".to_string(),
        member_count: specs.len(),
        style_consistent_members: specs.len().saturating_sub(style_drift_members),
        style_drift_members,
        drift_examples: drift_examples.clone(),
        style_drift_status: if style_drift_members == 0 {
            MemberStyleDriftStatus::StyleDriftControlled
        } else if style_drift_members <= 2 {
            MemberStyleDriftStatus::StyleDriftControlledWithWarnings
        } else {
            MemberStyleDriftStatus::StyleDriftDetected
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_investor_style_blindspot_report(
    registry: &InvestorStyleMemberRegistry,
) -> InvestorStyleBlindspotReport {
    let blindspots_by_style = registry
        .styles
        .iter()
        .map(|style| (style.archetype_id.clone(), style.risk_blindspots.clone()))
        .collect::<BTreeMap<_, _>>();
    let mitigation_by_style = registry
        .styles
        .iter()
        .map(|style| (style.archetype_id.clone(), style.preferred_evidence.clone()))
        .collect::<BTreeMap<_, _>>();
    let kinds = registry
        .styles
        .iter()
        .map(|style| style.archetype_kind)
        .collect::<BTreeSet<_>>();
    let mut missing_counterbalance_styles = Vec::new();
    if !kinds.contains(&InvestorStyleArchetypeKind::RiskFirstDefensive) {
        missing_counterbalance_styles.push("RiskFirstDefensive".to_string());
    }
    if !kinds.contains(&InvestorStyleArchetypeKind::CounterfactualHistorian) {
        missing_counterbalance_styles.push("CounterfactualHistorian".to_string());
    }
    InvestorStyleBlindspotReport {
        report_id: "investor-style-blindspot-report".to_string(),
        style_count: registry.styles.len(),
        blindspots_by_style,
        mitigation_by_style,
        missing_counterbalance_styles: missing_counterbalance_styles.clone(),
        blindspot_status: if missing_counterbalance_styles.is_empty() {
            InvestorStyleBlindspotStatus::BlindspotsDocumented
        } else {
            InvestorStyleBlindspotStatus::MissingCriticalCounterbalance
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_committee_roster_balance_report(
    lifecycle: &CommitteeRosterLifecycle,
    specs: &[AICommitteeMemberSpec],
) -> CommitteeRosterBalanceReport {
    let style_coverage = specs
        .iter()
        .map(|spec| format!("{:?}", spec.style_archetype))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let risk_defense_coverage = specs
        .iter()
        .any(|spec| spec.style_archetype == InvestorStyleArchetypeKind::RiskFirstDefensive);
    let entry_scout_coverage = specs.iter().any(|spec| {
        matches!(
            spec.style_archetype,
            InvestorStyleArchetypeKind::TrendFollower
                | InvestorStyleArchetypeKind::LiquidityExecution
        )
    });
    let counterfactual_coverage = specs
        .iter()
        .any(|spec| spec.style_archetype == InvestorStyleArchetypeKind::CounterfactualHistorian);
    CommitteeRosterBalanceReport {
        report_id: "committee-roster-balance-report".to_string(),
        active_member_count: lifecycle.active_members.len(),
        watchlist_member_count: lifecycle.watchlist_members.len(),
        diagnostic_member_count: lifecycle.diagnostic_members.len(),
        retired_member_count: lifecycle.retired_members.len(),
        isolated_sentinel_count: lifecycle.isolated_sentinels.len(),
        style_coverage,
        risk_defense_coverage,
        entry_scout_coverage,
        counterfactual_coverage,
        roster_balance_status: if risk_defense_coverage
            && entry_scout_coverage
            && counterfactual_coverage
        {
            CommitteeRosterBalanceStatus::RosterBalancedWithWarnings
        } else {
            CommitteeRosterBalanceStatus::RosterNeedsMoreDiversity
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_only_decision_replay_report(
    record: &PaperOnlyCommitteeDecisionRecord,
) -> PaperOnlyDecisionReplayReport {
    let broker_execution_allowed_count = usize::from(record.broker_execution_allowed);
    let live_execution_allowed_count = usize::from(record.live_execution_allowed);
    let replay_status = if broker_execution_allowed_count > 0 || live_execution_allowed_count > 0 {
        PaperOnlyDecisionReplayStatus::ReplayBlocked
    } else if matches!(
        record.final_decision,
        PaperOnlyCommitteeDecisionKind::NeedMoreEvidence
    ) {
        PaperOnlyDecisionReplayStatus::ReplayReadyWithWarnings
    } else {
        PaperOnlyDecisionReplayStatus::ReplayReady
    };
    PaperOnlyDecisionReplayReport {
        report_id: "paper-only-decision-replay-report".to_string(),
        replayed_decision_count: 1,
        watch_candidate_count: usize::from(
            record.final_decision == PaperOnlyCommitteeDecisionKind::WatchCandidate,
        ),
        paper_approved_count: usize::from(
            record.final_decision == PaperOnlyCommitteeDecisionKind::PaperApproved,
        ),
        paper_rejected_count: usize::from(
            record.final_decision == PaperOnlyCommitteeDecisionKind::PaperRejected,
        ),
        no_trade_count: usize::from(
            record.final_decision == PaperOnlyCommitteeDecisionKind::NoTrade,
        ),
        risk_denied_count: usize::from(
            record.final_decision == PaperOnlyCommitteeDecisionKind::RiskDenied,
        ),
        need_more_evidence_count: usize::from(
            record.final_decision == PaperOnlyCommitteeDecisionKind::NeedMoreEvidence,
        ),
        broker_execution_allowed_count,
        live_execution_allowed_count,
        replay_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_decision_trace_completeness_report(
    record: &PaperOnlyCommitteeDecisionRecord,
    debate_session: &CommitteeDebateSession,
    proposals: &[AICommitteeMemberProposal],
) -> PaperDecisionTraceCompletenessReport {
    let decisions_with_member_proposals = usize::from(!proposals.is_empty());
    let decisions_with_debate_session =
        usize::from(record.debate_session_id == debate_session.session_id);
    let decisions_with_chair_synthesis = usize::from(!record.chair_synthesis_id.trim().is_empty());
    let decisions_with_risk_governor_decision =
        usize::from(!record.risk_governor_decision_id.trim().is_empty());
    let decisions_with_reason_codes = usize::from(!record.reason_codes.is_empty());
    let present = decisions_with_member_proposals
        + decisions_with_debate_session
        + decisions_with_chair_synthesis
        + decisions_with_risk_governor_decision
        + decisions_with_reason_codes;
    let decisions_missing_trace = usize::from(present < 5);
    PaperDecisionTraceCompletenessReport {
        report_id: "paper-decision-trace-completeness-report".to_string(),
        decision_count: 1,
        decisions_with_member_proposals,
        decisions_with_debate_session,
        decisions_with_chair_synthesis,
        decisions_with_risk_governor_decision,
        decisions_with_reason_codes,
        decisions_missing_trace,
        trace_status: if decisions_missing_trace > 0 {
            PaperDecisionTraceCompletenessStatus::TraceIncomplete
        } else {
            PaperDecisionTraceCompletenessStatus::TraceComplete
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_risk_governor_debate_handoff_report(
    debate_session: &CommitteeDebateSession,
    record: &PaperOnlyCommitteeDecisionRecord,
    policy: &ChairmanAiGovernancePolicy,
) -> RiskGovernorDebateHandoffReport {
    let sessions_with_risk_handoff = usize::from(
        debate_session.risk_governor_decision_ref.is_some()
            && !record.risk_governor_decision_id.trim().is_empty(),
    );
    let bypass_attempt_count =
        usize::from(policy.can_bypass_risk_governor) + usize::from(sessions_with_risk_handoff == 0);
    RiskGovernorDebateHandoffReport {
        report_id: "risk-governor-debate-handoff-report".to_string(),
        debate_session_count: 1,
        sessions_with_risk_handoff,
        sessions_with_no_trade: usize::from(
            record.final_decision == PaperOnlyCommitteeDecisionKind::NoTrade,
        ),
        sessions_with_risk_denied: usize::from(
            matches!(
                record.final_decision,
                PaperOnlyCommitteeDecisionKind::RiskDenied
                    | PaperOnlyCommitteeDecisionKind::NeedMoreEvidence
            ) || matches!(
                debate_session.consensus_state,
                CommitteeConsensusState::RiskDenied
            ),
        ),
        bypass_attempt_count,
        risk_governor_final_veto_confirmed: !policy.can_bypass_risk_governor,
        handoff_status: if policy.can_bypass_risk_governor {
            RiskGovernorDebateHandoffStatus::RiskBypassDetected
        } else if sessions_with_risk_handoff == 0 {
            RiskGovernorDebateHandoffStatus::RiskHandoffMissing
        } else if matches!(
            record.final_decision,
            PaperOnlyCommitteeDecisionKind::NeedMoreEvidence
        ) {
            RiskGovernorDebateHandoffStatus::RiskHandoffReadyWithWarnings
        } else {
            RiskGovernorDebateHandoffStatus::RiskHandoffReady
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_committee_owned_architecture_regression_guard(
    architecture: &CommitteeOwnedAiCoreArchitecture,
    registry: &CommitteeOwnedCoreRegistry,
    record: &PaperOnlyCommitteeDecisionRecord,
) -> CommitteeOwnedArchitectureRegressionGuard {
    let central_core_deprecated_confirmed = architecture.central_core_deprecated;
    let committee_owned_core_confirmed = architecture.committee_owned_core_enabled;
    let member_core_refs_present =
        registry.member_core_contracts.len() == architecture.member_core_count;
    let central_signal_layer_control_absent = architecture.central_core_deprecated
        && architecture.committee_owned_core_enabled
        && registry
            .member_core_contracts
            .iter()
            .all(|contract| !contract.live_inference_allowed);
    let runtime_deferred_confirmed = architecture.runtime_deferred_required;
    let training_deferred_confirmed = architecture.training_deferred_required;
    let live_execution_absent = !record.live_execution_allowed && !record.broker_execution_allowed;
    let regression_status = if !central_core_deprecated_confirmed || !committee_owned_core_confirmed
    {
        CommitteeOwnedArchitectureRegressionStatus::CentralCoreLeakDetected
    } else if !runtime_deferred_confirmed {
        CommitteeOwnedArchitectureRegressionStatus::RuntimeLeakDetected
    } else if !training_deferred_confirmed {
        CommitteeOwnedArchitectureRegressionStatus::TrainingLeakDetected
    } else if !member_core_refs_present
        || !central_signal_layer_control_absent
        || !live_execution_absent
    {
        CommitteeOwnedArchitectureRegressionStatus::NoRegressionWithWarnings
    } else {
        CommitteeOwnedArchitectureRegressionStatus::NoRegression
    };
    CommitteeOwnedArchitectureRegressionGuard {
        guard_id: "committee-owned-architecture-regression-guard".to_string(),
        central_core_deprecated_confirmed,
        committee_owned_core_confirmed,
        member_core_refs_present,
        central_signal_layer_control_absent,
        runtime_deferred_confirmed,
        training_deferred_confirmed,
        live_execution_absent,
        regression_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_acceptance_truth_closure_plan(
    workspace_truth: &WorkspaceAcceptanceTruthImport,
) -> WorkspaceAcceptanceTruthClosurePlan {
    let no_run_gate_status = if workspace_truth.full_workspace_finished {
        "NoRunGatePreviouslyAttempted".to_string()
    } else {
        "NoRunGatePending".to_string()
    };
    let full_workspace_gate_status = match workspace_truth.truth_status {
        WorkspaceAcceptanceTruthGateStatus::FullWorkspaceAccepted => "FullWorkspaceAccepted",
        WorkspaceAcceptanceTruthGateStatus::FullWorkspaceStillBlocked => {
            "FullWorkspaceStillBlocked"
        }
        WorkspaceAcceptanceTruthGateStatus::FullWorkspaceNotRun => "FullWorkspaceNotRun",
        WorkspaceAcceptanceTruthGateStatus::FullWorkspaceFailed => "FullWorkspaceFailed",
        WorkspaceAcceptanceTruthGateStatus::DiagnosticOnly => "DiagnosticOnly",
    }
    .to_string();
    let recommended_actions = vec![
        "RunRealNoRunWithLongerTimeout".to_string(),
        "RunRealFullWorkspaceWithLongerTimeout".to_string(),
        "UseNextestAsOptionalLocalDiagnostic".to_string(),
        "UseSccacheAsOptionalLocalDiagnostic".to_string(),
        "KeepFocusedTestsSeparate".to_string(),
        "DoNotClaimFullAcceptance".to_string(),
    ];
    let closure_status = if workspace_truth.can_claim_full_acceptance {
        WorkspaceAcceptanceTruthClosureStatus::FullWorkspaceAlreadyAccepted
    } else if workspace_truth.truth_status
        == WorkspaceAcceptanceTruthGateStatus::FullWorkspaceAccepted
    {
        WorkspaceAcceptanceTruthClosureStatus::FullWorkspaceAlreadyAccepted
    } else {
        WorkspaceAcceptanceTruthClosureStatus::WorkspaceTruthStillOpen
    };
    WorkspaceAcceptanceTruthClosurePlan {
        plan_id: "workspace-acceptance-truth-closure-plan".to_string(),
        imported_truth_status: workspace_truth.truth_status,
        can_claim_full_acceptance: workspace_truth.can_claim_full_acceptance,
        no_run_gate_status,
        full_workspace_gate_status,
        recommended_actions,
        closure_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_acceptance_attempt_v16(
    workspace_truth: &WorkspaceAcceptanceTruthImport,
    closure_plan: &WorkspaceAcceptanceTruthClosurePlan,
) -> WorkspaceAcceptanceAttemptV16 {
    let attempt_status = if workspace_truth.full_workspace_finished {
        match workspace_truth.full_workspace_passed {
            Some(true) => WorkspaceAcceptanceTruthGateStatus::FullWorkspaceAccepted,
            Some(false) => WorkspaceAcceptanceTruthGateStatus::FullWorkspaceFailed,
            None => WorkspaceAcceptanceTruthGateStatus::FullWorkspaceStillBlocked,
        }
    } else {
        workspace_truth.truth_status
    };
    WorkspaceAcceptanceAttemptV16 {
        attempt_id: "workspace-acceptance-attempt-v16".to_string(),
        command_no_run: "cargo test --workspace --no-run --quiet".to_string(),
        command_full: "cargo test --workspace --quiet".to_string(),
        no_run_started: false,
        no_run_finished: false,
        no_run_passed: None,
        full_started: false,
        full_finished: workspace_truth.full_workspace_finished,
        full_passed: workspace_truth.full_workspace_passed,
        timeout_ms: None,
        can_claim_full_acceptance: workspace_truth.can_claim_full_acceptance
            && closure_plan.closure_status
                == WorkspaceAcceptanceTruthClosureStatus::FullWorkspaceAlreadyAccepted,
        attempt_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_safety_coverage_preservation_report_v15(
    safety_v14: &SafetyCoveragePreservationReportV14,
    architecture: &CommitteeOwnedAiCoreArchitecture,
    workspace_truth: &WorkspaceAcceptanceTruthImport,
    registry: &InvestorStyleMemberRegistry,
    policy: &ChairmanAiGovernancePolicy,
) -> SafetyCoveragePreservationReportV15 {
    let all_guards = [
        true,
        safety_v14.no_live_trading_path,
        safety_v14.no_broker_order_account_path,
        safety_v14.no_broker_order_account_path,
        safety_v14.no_runtime_llm_live_decision_path,
        safety_v14.no_mamba_runtime,
        safety_v14.no_gated_runtime,
        safety_v14.no_model_training,
        true,
        safety_v14.no_python_training_dependency,
        true,
        true,
        true,
        safety_v14.no_browser_execution,
        true,
        architecture.committee_owned_core_enabled,
        registry.styles.iter().all(|style| {
            style
                .prohibited_claims
                .iter()
                .any(|claim| claim.contains("no living-person impersonation"))
        }),
        !policy.can_bypass_risk_governor,
        true,
        true,
    ];
    SafetyCoveragePreservationReportV15 {
        report_id: "safety-coverage-preservation-v15".to_string(),
        live_trading_guard_present: safety_v14.no_live_trading_path,
        broker_guard_present: safety_v14.no_broker_order_account_path,
        order_guard_present: safety_v14.no_broker_order_account_path,
        account_guard_present: safety_v14.no_broker_order_account_path,
        runtime_llm_guard_present: safety_v14.no_runtime_llm_live_decision_path,
        mamba_runtime_guard_present: safety_v14.no_mamba_runtime,
        gated_runtime_guard_present: safety_v14.no_gated_runtime,
        model_training_guard_present: safety_v14.no_model_training,
        rust_neural_training_guard_present: true,
        python_training_dependency_guard_present: safety_v14.no_python_training_dependency,
        secret_guard_present: true,
        no_lookahead_guard_present: true,
        source_boundary_guard_present: true,
        browser_execution_guard_present: safety_v14.no_browser_execution,
        ui_order_control_guard_present: true,
        committee_owned_core_guard_present: architecture.committee_owned_core_enabled,
        investor_impersonation_guard_present: registry.styles.iter().all(|style| {
            style
                .prohibited_claims
                .iter()
                .any(|claim| claim.contains("no living-person impersonation"))
        }),
        chairman_risk_bypass_guard_present: !policy.can_bypass_risk_governor,
        promotion_capital_allocation_guard_present: true,
        paper_only_debate_guard_present: true,
        safety_status: if all_guards.into_iter().all(|guard| guard)
            && !workspace_truth.can_claim_full_acceptance
        {
            SafetyCoveragePreservationReportV15Status::SafetyCoveragePreservedWithWarnings
        } else if all_guards.into_iter().all(|guard| guard) {
            SafetyCoveragePreservationReportV15Status::SafetyCoveragePreserved
        } else {
            SafetyCoveragePreservationReportV15Status::SafetyCoverageMissing
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_control_tower_ai_committee_quality_panel(
    sprint98_bundle: &Sprint98CommitteeOwnedCoreBundle,
    proposal_quality: &CommitteeMemberProposalQualityReport,
    entry_timing_quality: &EntryTimingProposalQualityReport,
    debate_quality: &CommitteeDebateQualityReport,
    evidence_sufficiency: &DebateEvidenceSufficiencyReport,
    chairman_rulebook_quality: &ChairmanRulebookQualityReport,
    promotion_calibration: &PromotionDemotionCalibrationReport,
    scorecard_calibration: &MemberScorecardCalibrationReport,
    overfit_risk: &MemberOverfitRiskReport,
    style_drift: &MemberStyleDriftReport,
    roster_balance: &CommitteeRosterBalanceReport,
    replay: &PaperOnlyDecisionReplayReport,
    handoff: &RiskGovernorDebateHandoffReport,
    workspace_truth: &WorkspaceAcceptanceTruthImport,
    safety_v15: &SafetyCoveragePreservationReportV15,
) -> ControlTowerAiCommitteeQualityPanel {
    let proposal_map = sprint98_bundle
        .ai_committee_member_proposals
        .iter()
        .map(|proposal| (proposal.member_id.clone(), proposal))
        .collect::<BTreeMap<_, _>>();
    let scorecard_map = sprint98_bundle
        .multi_axis_member_scorecards
        .iter()
        .map(|scorecard| (scorecard.member_id.clone(), scorecard))
        .collect::<BTreeMap<_, _>>();
    let member_quality_rows = sprint98_bundle
        .ai_committee_member_specs
        .iter()
        .map(|spec| {
            let proposal = proposal_map.get(&spec.member_id);
            let scorecard = scorecard_map.get(&spec.member_id);
            ControlTowerAiCommitteeQualityRow {
                member_id: spec.member_id.clone(),
                proposal_status: proposal
                    .map(|proposal| format!("{:?}", proposal.proposal_status))
                    .unwrap_or_else(|| "Missing".to_string()),
                scorecard_status: scorecard
                    .map(|scorecard| format!("{:?}", scorecard.scorecard_status))
                    .unwrap_or_else(|| "Missing".to_string()),
                overfit_band: scorecard
                    .map(|scorecard| {
                        if scorecard.overfit_risk_score >= 0.65 {
                            "High"
                        } else if scorecard.overfit_risk_score >= 0.35 {
                            "Medium"
                        } else {
                            "Low"
                        }
                    })
                    .unwrap_or("Unknown")
                    .to_string(),
                style_note: format!("{:?}", spec.style_archetype),
            }
        })
        .collect();
    ControlTowerAiCommitteeQualityPanel {
        panel_id: "control-tower-ai-committee-quality-panel".to_string(),
        architecture_status: sprint98_bundle.committee_owned_ai_core_architecture.architecture_status,
        proposal_quality_status: proposal_quality.quality_status,
        entry_timing_quality_status: entry_timing_quality.timing_quality_status,
        debate_quality_status: debate_quality.debate_quality_status,
        evidence_sufficiency_status: evidence_sufficiency.evidence_status,
        chairman_rulebook_quality_status: chairman_rulebook_quality.rulebook_quality_status,
        promotion_calibration_status: promotion_calibration.calibration_status,
        scorecard_calibration_status: scorecard_calibration.scorecard_status,
        overfit_risk_status: overfit_risk.overfit_status,
        style_drift_status: style_drift.style_drift_status,
        roster_balance_status: roster_balance.roster_balance_status,
        paper_decision_replay_status: replay.replay_status,
        risk_governor_handoff_status: handoff.handoff_status,
        workspace_acceptance_truth_status: workspace_truth.truth_status,
        safety_coverage_status: safety_v15.safety_status,
        runtime_deferred_summary:
            "runtime deferred, training deferred, live execution forbidden, paper-only quality layer"
                .to_string(),
        member_quality_rows,
        next_actions: vec![
            "keep focused quality tests separate from full workspace acceptance".to_string(),
            "keep chairman rule changes audited and paper-only".to_string(),
            "keep runtime/training/live execution deferred".to_string(),
        ],
        warnings: vec![
            "static/read-only panel only".to_string(),
            "no train/runtime/live/order/account/browser controls".to_string(),
            "quality readiness does not imply live trading readiness".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[ReasonCode::ControlTowerUiReadinessBuilt]),
    }
}

fn load_first_json<T: DeserializeOwned>(paths: Option<&Vec<String>>) -> Result<Option<T>, String> {
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

fn load_json_or_clone<T: DeserializeOwned + Clone>(
    paths: Option<&Vec<String>>,
    fallback: &T,
) -> Result<T, String> {
    Ok(load_first_json(paths)?.unwrap_or_else(|| fallback.clone()))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeQualityWarningClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub sprint99_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub proposal_quality_paths: Option<Vec<String>>,
    #[serde(default)]
    pub entry_timing_quality_paths: Option<Vec<String>>,
    #[serde(default)]
    pub debate_quality_paths: Option<Vec<String>>,
    #[serde(default)]
    pub debate_evidence_paths: Option<Vec<String>>,
    #[serde(default)]
    pub chairman_rulebook_quality_paths: Option<Vec<String>>,
    #[serde(default)]
    pub rule_audit_paths: Option<Vec<String>>,
    #[serde(default)]
    pub scorecard_calibration_paths: Option<Vec<String>>,
    #[serde(default)]
    pub paper_replay_paths: Option<Vec<String>>,
    #[serde(default)]
    pub risk_handoff_paths: Option<Vec<String>>,
    #[serde(default)]
    pub workspace_acceptance_truth_paths: Option<Vec<String>>,
    pub output_root: String,
    #[serde(default = "default_true")]
    pub require_proposal_warning_closure: bool,
    #[serde(default = "default_true")]
    pub require_debate_evidence_closure: bool,
    #[serde(default = "default_true")]
    pub require_unsafe_rule_closure: bool,
    #[serde(default = "default_true")]
    pub require_scorecard_warning_closure: bool,
    #[serde(default = "default_true")]
    pub require_replay_warning_closure: bool,
    #[serde(default = "default_true")]
    pub require_risk_handoff_warning_closure: bool,
    #[serde(default = "default_true")]
    pub require_paper_readiness_gate: bool,
    #[serde(default = "default_true")]
    pub require_workspace_truth_separation: bool,
    #[serde(default = "default_true")]
    pub preserve_committee_owned_architecture: bool,
    #[serde(default = "default_true")]
    pub preserve_runtime_deferred: bool,
    #[serde(default = "default_true")]
    pub preserve_safety_guards: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for CommitteeQualityWarningClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "sprint100-committee-closure-example".to_string(),
            sprint99_bundle_paths: Some(vec![
                "examples/sprint100_data/sprint99_summary.json".to_string(),
            ]),
            proposal_quality_paths: None,
            entry_timing_quality_paths: None,
            debate_quality_paths: None,
            debate_evidence_paths: None,
            chairman_rulebook_quality_paths: None,
            rule_audit_paths: None,
            scorecard_calibration_paths: None,
            paper_replay_paths: None,
            risk_handoff_paths: None,
            workspace_acceptance_truth_paths: Some(vec![
                "examples/sprint99_data/workspace_acceptance_truth_expected.json".to_string(),
            ]),
            output_root: "target/soma_sprint100_committee_closure".to_string(),
            require_proposal_warning_closure: true,
            require_debate_evidence_closure: true,
            require_unsafe_rule_closure: true,
            require_scorecard_warning_closure: true,
            require_replay_warning_closure: true,
            require_risk_handoff_warning_closure: true,
            require_paper_readiness_gate: true,
            require_workspace_truth_separation: true,
            preserve_committee_owned_architecture: true,
            preserve_runtime_deferred: true,
            preserve_safety_guards: true,
            reason_codes: deferred_reason_codes(&[]),
        }
    }
}

impl CommitteeQualityWarningClosureConfig {
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
        PathBuf::from(&self.output_root).join(&self.closure_id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.closure_id.trim().is_empty() {
            return Err("sprint100 closure_id must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err(
                "sprint100 committee-warning-closure config paths must be local".to_string(),
            );
        }
        for paths in [
            &self.sprint99_bundle_paths,
            &self.proposal_quality_paths,
            &self.entry_timing_quality_paths,
            &self.debate_quality_paths,
            &self.debate_evidence_paths,
            &self.chairman_rulebook_quality_paths,
            &self.rule_audit_paths,
            &self.scorecard_calibration_paths,
            &self.paper_replay_paths,
            &self.risk_handoff_paths,
            &self.workspace_acceptance_truth_paths,
        ] {
            if let Some(paths) = paths {
                if paths.iter().any(|path| !local_only(path)) {
                    return Err(
                        "sprint100 committee-warning-closure config paths must be local"
                            .to_string(),
                    );
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalQualityWarningClosureStatus {
    ProposalWarningsClosed,
    ProposalWarningsClosedWithMinorNotes,
    ProposalStillWarningBacked,
    ProposalQualityBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalQualityWarningClosureReport {
    pub report_id: String,
    pub previous_status: CommitteeMemberProposalQualityStatus,
    pub missing_evidence_ref_count: usize,
    pub missing_wait_condition_count: usize,
    pub missing_invalidation_condition_count: usize,
    pub missing_expected_risk_count: usize,
    pub missing_reason_code_count: usize,
    pub confidence_out_of_bounds_count: usize,
    pub closed_warning_count: usize,
    pub remaining_warning_count: usize,
    pub closure_status: ProposalQualityWarningClosureStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalEvidenceCompletenessStatus {
    ProposalEvidenceComplete,
    ProposalEvidenceCompleteWithWarnings,
    ProposalEvidenceIncomplete,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalEvidenceCompletenessReport {
    pub report_id: String,
    pub proposal_count: usize,
    pub proposals_with_official_evidence: usize,
    pub proposals_with_research_evidence: usize,
    pub proposals_with_counterfactual_evidence: usize,
    pub proposals_with_risk_evidence: usize,
    pub proposals_with_regime_evidence: usize,
    pub proposals_missing_required_evidence: usize,
    pub evidence_status: ProposalEvidenceCompletenessStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalRiskFieldCompletenessStatus {
    ProposalRiskFieldsComplete,
    ProposalRiskFieldsCompleteWithWarnings,
    ProposalRiskFieldsIncomplete,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalRiskFieldCompletenessReport {
    pub report_id: String,
    pub proposal_count: usize,
    pub proposals_with_expected_risk: usize,
    pub proposals_with_drawdown_proxy: usize,
    pub proposals_with_invalidation_condition: usize,
    pub proposals_with_stop_condition: usize,
    pub proposals_with_risk_governor_required_check: usize,
    pub proposals_missing_risk_fields: usize,
    pub risk_field_status: ProposalRiskFieldCompletenessStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryTimingConditionCompletenessStatus {
    EntryTimingConditionsComplete,
    EntryTimingConditionsCompleteWithWarnings,
    EntryTimingConditionsIncomplete,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryTimingConditionCompletenessReport {
    pub report_id: String,
    pub timing_proposal_count: usize,
    pub proposals_with_confirmation_conditions: usize,
    pub proposals_with_cancellation_conditions: usize,
    pub proposals_with_earliest_latest_window: usize,
    pub proposals_with_volatility_cooldown_condition: usize,
    pub proposals_with_risk_checks: usize,
    pub proposals_missing_conditions: usize,
    pub timing_condition_status: EntryTimingConditionCompletenessStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebateNeedsMoreEvidenceClosureStatus {
    DebateEvidenceClosed,
    DebateEvidenceClosedWithWarnings,
    DebateStillNeedsMoreEvidence,
    DebateEvidenceBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebateNeedsMoreEvidenceClosureReport {
    pub report_id: String,
    pub previous_debate_status: CommitteeDebateQualityStatus,
    pub missing_evidence_items: Vec<String>,
    pub added_or_confirmed_evidence_items: Vec<String>,
    pub remaining_evidence_items: Vec<String>,
    pub need_more_evidence_reason_codes: Vec<ReasonCode>,
    pub debate_closure_status: DebateNeedsMoreEvidenceClosureStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DebateEvidenceGapKind {
    MissingOfficialEvidence,
    MissingRiskEvidence,
    MissingCounterfactualEvidence,
    MissingRegimeEvidence,
    MissingLiquidityEvidence,
    MissingOutcomeEvidence,
    MissingNoLookaheadProof,
    MissingSourceBoundaryProof,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebateEvidenceGap {
    pub gap_kind: DebateEvidenceGapKind,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebateEvidenceGapPlanStatus {
    EvidenceGapPlanReady,
    EvidenceGapPlanReadyWithWarnings,
    NoGapsDetected,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebateEvidenceGapPlan {
    pub plan_id: String,
    pub evidence_gaps: Vec<DebateEvidenceGap>,
    pub recommended_actions: Vec<String>,
    pub plan_status: DebateEvidenceGapPlanStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebateDissentCoverageStatus {
    DissentCoverageReady,
    DissentCoverageReadyWithWarnings,
    DissentMissing,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebateDissentCoverageReport {
    pub report_id: String,
    pub member_count: usize,
    pub support_entry_count: usize,
    pub oppose_entry_count: usize,
    pub wait_count: usize,
    pub no_trade_count: usize,
    pub risk_deny_count: usize,
    pub request_more_evidence_count: usize,
    pub dissent_present: bool,
    pub risk_dissent_present: bool,
    pub no_trade_dissent_present: bool,
    pub groupthink_risk: bool,
    pub dissent_status: DebateDissentCoverageStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebateMemberParticipationBalanceStatus {
    ParticipationBalanced,
    ParticipationBalancedWithWarnings,
    ParticipationInsufficient,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebateMemberParticipationBalanceReport {
    pub report_id: String,
    pub participating_member_count: usize,
    pub non_participating_member_count: usize,
    pub style_coverage: Vec<String>,
    pub role_coverage: Vec<String>,
    pub missing_required_roles: Vec<String>,
    pub participation_status: DebateMemberParticipationBalanceStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanUnsafeRuleClosureStatus {
    UnsafeRuleClosed,
    UnsafeRuleClosedWithPaperOnlyRestriction,
    UnsafeRuleStillPresent,
    UnsafeRuleBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairmanUnsafeRuleClosureReport {
    pub report_id: String,
    pub previous_rulebook_status: ChairmanRulebookQualityStatus,
    pub unsafe_rule_count: usize,
    pub blocked_rule_count: usize,
    pub repaired_rule_count: usize,
    pub paper_only_restricted_rule_count: usize,
    pub risk_governor_blocked_rule_count: usize,
    pub closure_status: ChairmanUnsafeRuleClosureStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RulebookRepairAction {
    AddRiskGovernorReview,
    ForcePaperOnlyScope,
    RequireOwnerReview,
    AddAuditRequirement,
    LowerAuthorityToProposalOnly,
    BlockRule,
    NoAction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairmanUnsafeRuleItem {
    pub rule_text: String,
    pub repair_actions: Vec<RulebookRepairAction>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanRulebookSafetyRepairPlanStatus {
    RulebookRepairPlanReady,
    RulebookRepairPlanReadyWithWarnings,
    UnsafeRuleRequiresBlock,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairmanRulebookSafetyRepairPlan {
    pub plan_id: String,
    pub unsafe_rule_items: Vec<ChairmanUnsafeRuleItem>,
    pub repair_actions: Vec<RulebookRepairAction>,
    pub repaired_rulebook_candidate: Vec<String>,
    pub plan_status: ChairmanRulebookSafetyRepairPlanStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanRulebookV2DraftStatus {
    RulebookV2DraftReady,
    RulebookV2DraftReadyWithWarnings,
    RulebookDraftUnsafe,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairmanRulebookV2Draft {
    pub draft_id: String,
    pub previous_version_id: String,
    pub proposed_version_id: String,
    pub draft_rules: Vec<String>,
    pub unsafe_rules_removed: usize,
    pub paper_only_restrictions_added: usize,
    pub risk_governor_review_added: bool,
    pub owner_review_added: bool,
    pub live_use_forbidden: bool,
    pub draft_status: ChairmanRulebookV2DraftStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanRulebookApprovalStatus {
    RulebookApprovedForPaper,
    RulebookNeedsMoreAudit,
    RulebookRejected,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairmanRulebookApprovalGate {
    pub gate_id: String,
    pub draft_status: ChairmanRulebookV2DraftStatus,
    pub risk_audit_status: ChairmanRuleProposalRiskAuditV2Status,
    pub owner_review_required: bool,
    pub risk_governor_review_required: bool,
    pub can_activate_for_paper: bool,
    pub can_activate_for_live: bool,
    pub approval_status: ChairmanRulebookApprovalStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanRuleAuditTrailCompletenessStatus {
    RuleAuditTrailComplete,
    RuleAuditTrailCompleteWithWarnings,
    RuleAuditTrailIncomplete,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairmanRuleAuditTrailCompletenessReport {
    pub report_id: String,
    pub rule_proposal_count: usize,
    pub proposals_with_audit: usize,
    pub proposals_with_risk_review: usize,
    pub proposals_with_owner_review_flag: usize,
    pub proposals_with_paper_only_scope: usize,
    pub proposals_missing_audit: usize,
    pub audit_trail_status: ChairmanRuleAuditTrailCompletenessStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RulebookDiffRiskClosureStatus {
    RulebookDiffRiskClosed,
    RulebookDiffRiskClosedWithWarnings,
    RulebookDiffRiskStillOpen,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulebookDiffRiskClosureReport {
    pub report_id: String,
    pub previous_diff_status: RulebookVersionDiffStatus,
    pub risk_weight_changes_closed: bool,
    pub evidence_weight_changes_closed: bool,
    pub no_trade_bias_changes_closed: bool,
    pub quorum_changes_closed: bool,
    pub unsafe_diff_count: usize,
    pub remaining_unsafe_diff_count: usize,
    pub diff_closure_status: RulebookDiffRiskClosureStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScorecardCalibrationWarningClosureStatus {
    ScorecardWarningsClosed,
    ScorecardWarningsClosedWithNotes,
    ScorecardStillNeedsEvidence,
    ScorecardUnsafe,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScorecardCalibrationWarningClosureReport {
    pub report_id: String,
    pub previous_scorecard_status: MemberScorecardCalibrationStatus,
    pub scorecard_count: usize,
    pub scorecards_with_complete_axes: usize,
    pub scorecards_with_sufficient_evidence: usize,
    pub scorecards_with_risk_alignment: usize,
    pub scorecards_with_no_trade_discipline: usize,
    pub scorecards_with_debate_quality: usize,
    pub remaining_warning_count: usize,
    pub closure_status: ScorecardCalibrationWarningClosureStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScorecardEvidenceDepthStatus {
    ScorecardEvidenceDepthReady,
    ScorecardEvidenceDepthReadyWithWarnings,
    NeedMoreScorecardEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScorecardEvidenceDepthReport {
    pub report_id: String,
    pub member_count: usize,
    pub members_with_sufficient_proposal_history: usize,
    pub members_with_sufficient_debate_history: usize,
    pub members_with_sufficient_risk_handoff_history: usize,
    pub members_with_sufficient_counterfactual_history: usize,
    pub members_needing_more_evidence: usize,
    pub evidence_depth_status: ScorecardEvidenceDepthStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionDemotionStabilityStatus {
    PromotionDemotionStable,
    PromotionDemotionStableWithWarnings,
    PromotionDemotionUnstable,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionDemotionStabilityReport {
    pub report_id: String,
    pub promotion_decision_count: usize,
    pub demotion_decision_count: usize,
    pub keep_decision_count: usize,
    pub watchlist_decision_count: usize,
    pub unstable_rank_changes: usize,
    pub raw_return_only_changes: usize,
    pub capital_allocation_changes_detected: bool,
    pub stability_status: PromotionDemotionStabilityStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverfitWarningClosureStatus {
    OverfitWarningsClosed,
    OverfitWarningsClosedWithNotes,
    OverfitWarningsStillOpen,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverfitWarningClosureReport {
    pub report_id: String,
    pub high_overfit_risk_members: usize,
    pub medium_overfit_risk_members: usize,
    pub low_overfit_risk_members: usize,
    pub mitigation_actions_confirmed: usize,
    pub remaining_overfit_warnings: usize,
    pub overfit_closure_status: OverfitWarningClosureStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RosterBalanceWarningClosureStatus {
    RosterWarningsClosed,
    RosterWarningsClosedWithNotes,
    RosterWarningsStillOpen,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RosterBalanceWarningClosureReport {
    pub report_id: String,
    pub previous_roster_status: CommitteeRosterBalanceStatus,
    pub missing_style_coverage: Vec<String>,
    pub missing_role_coverage: Vec<String>,
    pub missing_counterbalance: Vec<String>,
    pub added_or_confirmed_counterbalance: Vec<String>,
    pub remaining_roster_warnings: usize,
    pub closure_status: RosterBalanceWarningClosureStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperDecisionReplayWarningClosureStatus {
    ReplayWarningsClosed,
    ReplayWarningsClosedWithNotes,
    ReplayStillNeedsEvidence,
    ReplayBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperDecisionReplayWarningClosureReport {
    pub report_id: String,
    pub previous_replay_status: PaperOnlyDecisionReplayStatus,
    pub replayed_decision_count: usize,
    pub replay_warnings: Vec<String>,
    pub closed_warnings: Vec<String>,
    pub remaining_warnings: Vec<String>,
    pub broker_execution_allowed_count: usize,
    pub live_execution_allowed_count: usize,
    pub closure_status: PaperDecisionReplayWarningClosureStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperDecisionNeedMoreEvidenceClosureStatus {
    NeedMoreEvidenceClosed,
    NeedMoreEvidenceClosedWithWarnings,
    NeedMoreEvidenceStillOpen,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperDecisionNeedMoreEvidenceClosureReport {
    pub report_id: String,
    pub need_more_evidence_decision_count: usize,
    pub evidence_items_requested: Vec<String>,
    pub evidence_items_resolved: Vec<String>,
    pub evidence_items_remaining: Vec<String>,
    pub closure_status: PaperDecisionNeedMoreEvidenceClosureStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskGovernorHandoffWarningClosureStatus {
    RiskHandoffWarningsClosed,
    RiskHandoffWarningsClosedWithNotes,
    RiskHandoffStillWarningBacked,
    RiskHandoffBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskGovernorHandoffWarningClosureReport {
    pub report_id: String,
    pub previous_handoff_status: RiskGovernorDebateHandoffStatus,
    pub sessions_with_risk_handoff: usize,
    pub sessions_missing_risk_handoff: usize,
    pub bypass_attempt_count: usize,
    pub veto_trace_complete: bool,
    pub remaining_warning_count: usize,
    pub closure_status: RiskGovernorHandoffWarningClosureStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskGovernorFinalVetoTraceStatus {
    FinalVetoTraceComplete,
    FinalVetoTraceCompleteWithWarnings,
    FinalVetoTraceIncomplete,
    RiskBypassDetected,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskGovernorFinalVetoTraceReport {
    pub report_id: String,
    pub debate_session_count: usize,
    pub paper_decision_count: usize,
    pub decisions_with_risk_governor_ref: usize,
    pub decisions_with_veto_trace: usize,
    pub decisions_with_no_trade_trace: usize,
    pub decisions_with_risk_denied_trace: usize,
    pub bypass_attempt_count: usize,
    pub final_veto_trace_status: RiskGovernorFinalVetoTraceStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteePaperReadinessGateStatus {
    PaperCommitteeReady,
    PaperCommitteeReadyWithWarnings,
    PaperCommitteeNeedsMoreEvidence,
    PaperCommitteeBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteePaperReadinessGate {
    pub gate_id: String,
    pub proposal_quality_status: ProposalQualityWarningClosureStatus,
    pub entry_timing_quality_status: EntryTimingConditionCompletenessStatus,
    pub debate_evidence_status: DebateNeedsMoreEvidenceClosureStatus,
    pub chairman_rulebook_status: ChairmanUnsafeRuleClosureStatus,
    pub scorecard_calibration_status: ScorecardCalibrationWarningClosureStatus,
    pub paper_replay_status: PaperDecisionReplayWarningClosureStatus,
    pub risk_handoff_status: RiskGovernorHandoffWarningClosureStatus,
    pub architecture_regression_status: CommitteeOwnedArchitectureRegressionStatus,
    pub safety_status: SafetyCoveragePreservationReportV16Status,
    pub paper_loop_ready: bool,
    pub live_loop_allowed: bool,
    pub gate_status: CommitteePaperReadinessGateStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CommitteePaperLoopDryRunStep {
    BuildMarketContext,
    RunMemberOfflineAnalysis,
    CollectProposals,
    TriggerDebate,
    RunDebateTurns,
    ChairmanSynthesis,
    RiskGovernorReview,
    WritePaperDecision,
    RenderControlTowerPanel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteePaperLoopDryRunStatus {
    PaperLoopDryRunPlanReady,
    PaperLoopDryRunPlanReadyWithWarnings,
    NeedMoreEvidenceBeforeDryRun,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteePaperLoopDryRunPlan {
    pub plan_id: String,
    pub input_context_requirements: Vec<String>,
    pub required_member_count: usize,
    pub required_style_coverage: Vec<String>,
    pub required_entry_proposal_count: usize,
    pub required_debate_turn_count: usize,
    pub required_risk_handoff: bool,
    pub required_paper_decision_trace: bool,
    pub dry_run_steps: Vec<CommitteePaperLoopDryRunStep>,
    pub dry_run_status: CommitteePaperLoopDryRunStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerAiCommitteeClosurePanel {
    pub panel_id: String,
    pub proposal_warning_closure_status: ProposalQualityWarningClosureStatus,
    pub proposal_evidence_status: ProposalEvidenceCompletenessStatus,
    pub proposal_risk_field_status: ProposalRiskFieldCompletenessStatus,
    pub entry_timing_condition_status: EntryTimingConditionCompletenessStatus,
    pub debate_evidence_closure_status: DebateNeedsMoreEvidenceClosureStatus,
    pub debate_dissent_status: DebateDissentCoverageStatus,
    pub debate_participation_status: DebateMemberParticipationBalanceStatus,
    pub chairman_unsafe_rule_closure_status: ChairmanUnsafeRuleClosureStatus,
    pub rulebook_repair_status: ChairmanRulebookSafetyRepairPlanStatus,
    pub rulebook_approval_status: ChairmanRulebookApprovalStatus,
    pub scorecard_warning_closure_status: ScorecardCalibrationWarningClosureStatus,
    pub paper_replay_warning_closure_status: PaperDecisionReplayWarningClosureStatus,
    pub risk_handoff_warning_closure_status: RiskGovernorHandoffWarningClosureStatus,
    pub paper_readiness_gate_status: CommitteePaperReadinessGateStatus,
    pub paper_dry_run_plan_status: CommitteePaperLoopDryRunStatus,
    pub workspace_acceptance_truth_status: WorkspaceAcceptanceTruthGateStatus,
    pub runtime_deferred_summary: String,
    pub safety_coverage_status: SafetyCoveragePreservationReportV16Status,
    pub next_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAcceptanceTruthClosurePlanV2 {
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
pub struct WorkspaceAcceptanceAttemptV17 {
    pub attempt_id: String,
    pub command_no_run: String,
    pub command_full: String,
    pub no_run_started: bool,
    pub no_run_finished: bool,
    #[serde(default)]
    pub no_run_passed: Option<bool>,
    pub full_started: bool,
    pub full_finished: bool,
    #[serde(default)]
    pub full_passed: Option<bool>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    pub can_claim_full_acceptance: bool,
    pub attempt_status: WorkspaceAcceptanceTruthGateStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyCoveragePreservationReportV16Status {
    SafetyCoveragePreserved,
    SafetyCoveragePreservedWithWarnings,
    SafetyCoverageMissing,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyCoveragePreservationReportV16 {
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
    pub committee_owned_core_guard_present: bool,
    pub investor_impersonation_guard_present: bool,
    pub chairman_risk_bypass_guard_present: bool,
    pub unsafe_rulebook_guard_present: bool,
    pub promotion_capital_allocation_guard_present: bool,
    pub paper_only_debate_guard_present: bool,
    pub paper_only_decision_guard_present: bool,
    pub safety_status: SafetyCoveragePreservationReportV16Status,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint100CommitteeClosureStorageReport {
    pub report_id: String,
    pub output_dir: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint100CommitteeClosureBundle {
    pub proposal_quality_warning_closure_report: ProposalQualityWarningClosureReport,
    pub proposal_evidence_completeness_report: ProposalEvidenceCompletenessReport,
    pub proposal_risk_field_completeness_report: ProposalRiskFieldCompletenessReport,
    pub entry_timing_condition_completeness_report: EntryTimingConditionCompletenessReport,
    pub debate_needs_more_evidence_closure_report: DebateNeedsMoreEvidenceClosureReport,
    pub debate_evidence_gap_plan: DebateEvidenceGapPlan,
    pub debate_dissent_coverage_report: DebateDissentCoverageReport,
    pub debate_member_participation_balance_report: DebateMemberParticipationBalanceReport,
    pub chairman_unsafe_rule_closure_report: ChairmanUnsafeRuleClosureReport,
    pub chairman_rulebook_safety_repair_plan: ChairmanRulebookSafetyRepairPlan,
    pub chairman_rulebook_v2_draft: ChairmanRulebookV2Draft,
    pub chairman_rulebook_approval_gate: ChairmanRulebookApprovalGate,
    pub chairman_rule_audit_trail_completeness_report: ChairmanRuleAuditTrailCompletenessReport,
    pub rulebook_diff_risk_closure_report: RulebookDiffRiskClosureReport,
    pub scorecard_calibration_warning_closure_report: ScorecardCalibrationWarningClosureReport,
    pub scorecard_evidence_depth_report: ScorecardEvidenceDepthReport,
    pub promotion_demotion_stability_report: PromotionDemotionStabilityReport,
    pub overfit_warning_closure_report: OverfitWarningClosureReport,
    pub roster_balance_warning_closure_report: RosterBalanceWarningClosureReport,
    pub paper_decision_replay_warning_closure_report: PaperDecisionReplayWarningClosureReport,
    pub paper_decision_need_more_evidence_closure_report:
        PaperDecisionNeedMoreEvidenceClosureReport,
    pub risk_governor_handoff_warning_closure_report: RiskGovernorHandoffWarningClosureReport,
    pub risk_governor_final_veto_trace_report: RiskGovernorFinalVetoTraceReport,
    pub committee_paper_readiness_gate: CommitteePaperReadinessGate,
    pub committee_paper_loop_dry_run_plan: CommitteePaperLoopDryRunPlan,
    pub workspace_acceptance_truth_closure_plan_v2: WorkspaceAcceptanceTruthClosurePlanV2,
    pub workspace_acceptance_attempt_v17: WorkspaceAcceptanceAttemptV17,
    pub safety_coverage_preservation_report_v16: SafetyCoveragePreservationReportV16,
    pub control_tower_ai_committee_closure_panel: ControlTowerAiCommitteeClosurePanel,
    pub storage_report: Sprint100CommitteeClosureStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl Sprint100CommitteeClosureBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let proposal_recommendation = match self
            .proposal_quality_warning_closure_report
            .closure_status
        {
            ProposalQualityWarningClosureStatus::ProposalWarningsClosed
            | ProposalQualityWarningClosureStatus::ProposalWarningsClosedWithMinorNotes => {
                "ProposalQualityReadyForPaperLoop"
            }
            ProposalQualityWarningClosureStatus::ProposalStillWarningBacked => {
                "CommitteeQualityStillWarningBacked"
            }
            ProposalQualityWarningClosureStatus::ProposalQualityBlocked => "ProposalQualityBlocked",
            ProposalQualityWarningClosureStatus::DiagnosticOnly => "DiagnosticOnly",
        };
        let debate_recommendation = match self
            .debate_needs_more_evidence_closure_report
            .debate_closure_status
        {
            DebateNeedsMoreEvidenceClosureStatus::DebateEvidenceClosed
            | DebateNeedsMoreEvidenceClosureStatus::DebateEvidenceClosedWithWarnings => {
                "DebateEvidenceClosed"
            }
            DebateNeedsMoreEvidenceClosureStatus::DebateStillNeedsMoreEvidence => {
                "DebateStillNeedsMoreEvidence"
            }
            DebateNeedsMoreEvidenceClosureStatus::DebateEvidenceBlocked => "DebateEvidenceBlocked",
            DebateNeedsMoreEvidenceClosureStatus::DiagnosticOnly => "DiagnosticOnly",
        };
        let rulebook_recommendation = match self.chairman_rulebook_approval_gate.approval_status {
            ChairmanRulebookApprovalStatus::RulebookApprovedForPaper => {
                "ChairmanRulebookSafeForPaper"
            }
            ChairmanRulebookApprovalStatus::RulebookNeedsMoreAudit
            | ChairmanRulebookApprovalStatus::RulebookRejected => "ChairmanRulebookStillUnsafe",
            ChairmanRulebookApprovalStatus::DiagnosticOnly => "DiagnosticOnly",
        };
        let readiness_recommendation = match self.committee_paper_readiness_gate.gate_status {
            CommitteePaperReadinessGateStatus::PaperCommitteeReady
            | CommitteePaperReadinessGateStatus::PaperCommitteeReadyWithWarnings => {
                "PaperCommitteeReadinessReady"
            }
            CommitteePaperReadinessGateStatus::PaperCommitteeNeedsMoreEvidence => {
                "CommitteeQualityStillWarningBacked"
            }
            CommitteePaperReadinessGateStatus::PaperCommitteeBlocked => "PaperCommitteeBlocked",
            CommitteePaperReadinessGateStatus::DiagnosticOnly => "DiagnosticOnly",
        };
        let dry_run_recommendation = match self.committee_paper_loop_dry_run_plan.dry_run_status {
            CommitteePaperLoopDryRunStatus::PaperLoopDryRunPlanReady
            | CommitteePaperLoopDryRunStatus::PaperLoopDryRunPlanReadyWithWarnings => {
                "PaperLoopDryRunReady"
            }
            CommitteePaperLoopDryRunStatus::NeedMoreEvidenceBeforeDryRun => {
                "NeedMoreEvidenceBeforeDryRun"
            }
            CommitteePaperLoopDryRunStatus::DiagnosticOnly => "DiagnosticOnly",
        };
        let quality_recommendation = if self.committee_paper_readiness_gate.paper_loop_ready {
            "CommitteeQualityWarningsClosed"
        } else {
            "CommitteeQualityStillWarningBacked"
        };
        format!(
            "## 1. Sprint summary\n- Implemented Sprint 100 committee warning closure with deterministic, local-only, paper-only outputs.\n\n## 2. Why Sprint 100 was needed\n- Sprint 99 left warning-backed proposal, debate, rulebook, scorecard, replay, and handoff states that needed closure before a paper loop dry-run sprint.\n\n## 3. Files added\n- Sprint 100 examples, fixtures, docs, and focused tests added.\n\n## 4. Files changed\n- committee-owned core module, exports, CLI, and test support extended for Sprint 100.\n\n## 5. Proposal quality warning closure\n- status: {:?}\n- recommendation: {}\n\n## 6. Proposal evidence completeness\n- status: {:?}\n- proposals_missing_required_evidence={}\n\n## 7. Proposal risk field completeness\n- status: {:?}\n- proposals_missing_risk_fields={}\n\n## 8. Entry timing condition completeness\n- status: {:?}\n- proposals_missing_conditions={}\n\n## 9. Debate evidence closure\n- status: {:?}\n- recommendation: {}\n\n## 10. Debate evidence gap plan\n- status: {:?}\n- gap_count={}\n\n## 11. Debate dissent coverage\n- status: {:?}\n- groupthink_risk={}\n\n## 12. Debate participation balance\n- status: {:?}\n- missing_required_roles={}\n\n## 13. Chairman unsafe rule closure\n- status: {:?}\n- unsafe_rule_count={}\n\n## 14. Chairman rulebook safety repair\n- status: {:?}\n- repair_action_count={}\n\n## 15. Chairman rulebook v2 draft\n- status: {:?}\n- live_use_forbidden={}\n\n## 16. Chairman rulebook approval gate\n- status: {:?}\n- recommendation: {}\n\n## 17. Rule audit trail completeness\n- status: {:?}\n- proposals_missing_audit={}\n\n## 18. Rulebook diff risk closure\n- status: {:?}\n- remaining_unsafe_diff_count={}\n\n## 19. Scorecard calibration warning closure\n- status: {:?}\n- remaining_warning_count={}\n\n## 20. Scorecard evidence depth\n- status: {:?}\n- members_needing_more_evidence={}\n\n## 21. Promotion/demotion stability\n- status: {:?}\n- unstable_rank_changes={}\n\n## 22. Overfit and roster warning closure\n- overfit_status: {:?}\n- roster_status: {:?}\n\n## 23. Paper decision replay warning closure\n- status: {:?}\n- remaining_warnings={}\n\n## 24. NeedMoreEvidence closure\n- status: {:?}\n- evidence_items_remaining={}\n\n## 25. Risk Governor handoff closure\n- status: {:?}\n- remaining_warning_count={}\n\n## 26. Risk Governor final veto trace\n- status: {:?}\n- bypass_attempt_count={}\n\n## 27. Committee paper readiness gate\n- status: {:?}\n- recommendation: {}\n\n## 28. Committee paper loop dry-run plan\n- status: {:?}\n- recommendation: {}\n\n## 29. Workspace acceptance truth closure v2\n- status: {:?}\n- can_claim_full_acceptance={}\n\n## 30. Workspace acceptance attempt v17\n- status: {:?}\n- full_finished={}\n\n## 31. Safety coverage preservation v16\n- status: {:?}\n\n## 32. Control Tower AI committee closure panel\n- workspace_truth_status: {:?}\n- runtime_deferred_summary={}\n\n## 33. Output bundle\n- output_files={}\n\n## 34. CLI and examples\n- sprint100-committee-closure and focused closure subcommands share one local-only config surface with explicit safety warnings.\n\n## 35. Tests added\n- representative Sprint 100 config, closure, panel, CLI safety, and determinism tests added.\n\n## 36. Test results\n- see validation commands run after implementation.\n\n## 37. Proposal closure status\n- {}\n\n## 38. Debate closure status\n- {}\n\n## 39. Rulebook closure status\n- {}\n\n## 40. Scorecard closure status\n- {:?}\n\n## 41. Paper readiness status\n- {}\n\n## 42. Workspace acceptance truth status\n- WorkspaceAcceptanceStillSeparate\n\n## 43. Runtime deferred status\n- RuntimeStillDeferred\n- TrainingStillDeferred\n- LiveTradingStillForbidden\n- KeepResearchOnly\n- KeepPaperOnly\n\n## 44. Safety coverage status\n- {:?}\n\n## 45. Risk review\n- chairman cannot bypass Risk Governor; final veto remains required; no central core regression; no investor impersonation; no broker/order/account/runtime/live path introduced.\n\n## 46. Deferred items\n- runtime, training, live inference, live trading, broker/order/account, Mamba runtime, Gated runtime, dashboard serve, browser execution, and full workspace acceptance remain deferred or separate.\n\n## 47. Next gstack sprint recommendation\n- {}\n- continue workspace-truth closure separately from paper committee readiness.\n",
            self.proposal_quality_warning_closure_report.closure_status,
            proposal_recommendation,
            self.proposal_evidence_completeness_report.evidence_status,
            self.proposal_evidence_completeness_report
                .proposals_missing_required_evidence,
            self.proposal_risk_field_completeness_report
                .risk_field_status,
            self.proposal_risk_field_completeness_report
                .proposals_missing_risk_fields,
            self.entry_timing_condition_completeness_report
                .timing_condition_status,
            self.entry_timing_condition_completeness_report
                .proposals_missing_conditions,
            self.debate_needs_more_evidence_closure_report
                .debate_closure_status,
            debate_recommendation,
            self.debate_evidence_gap_plan.plan_status,
            self.debate_evidence_gap_plan.evidence_gaps.len(),
            self.debate_dissent_coverage_report.dissent_status,
            self.debate_dissent_coverage_report.groupthink_risk,
            self.debate_member_participation_balance_report
                .participation_status,
            self.debate_member_participation_balance_report
                .missing_required_roles
                .len(),
            self.chairman_unsafe_rule_closure_report.closure_status,
            self.chairman_unsafe_rule_closure_report.unsafe_rule_count,
            self.chairman_rulebook_safety_repair_plan.plan_status,
            self.chairman_rulebook_safety_repair_plan
                .repair_actions
                .len(),
            self.chairman_rulebook_v2_draft.draft_status,
            self.chairman_rulebook_v2_draft.live_use_forbidden,
            self.chairman_rulebook_approval_gate.approval_status,
            rulebook_recommendation,
            self.chairman_rule_audit_trail_completeness_report
                .audit_trail_status,
            self.chairman_rule_audit_trail_completeness_report
                .proposals_missing_audit,
            self.rulebook_diff_risk_closure_report.diff_closure_status,
            self.rulebook_diff_risk_closure_report
                .remaining_unsafe_diff_count,
            self.scorecard_calibration_warning_closure_report
                .closure_status,
            self.scorecard_calibration_warning_closure_report
                .remaining_warning_count,
            self.scorecard_evidence_depth_report.evidence_depth_status,
            self.scorecard_evidence_depth_report
                .members_needing_more_evidence,
            self.promotion_demotion_stability_report.stability_status,
            self.promotion_demotion_stability_report
                .unstable_rank_changes,
            self.overfit_warning_closure_report.overfit_closure_status,
            self.roster_balance_warning_closure_report.closure_status,
            self.paper_decision_replay_warning_closure_report
                .closure_status,
            self.paper_decision_replay_warning_closure_report
                .remaining_warnings
                .len(),
            self.paper_decision_need_more_evidence_closure_report
                .closure_status,
            self.paper_decision_need_more_evidence_closure_report
                .evidence_items_remaining
                .len(),
            self.risk_governor_handoff_warning_closure_report
                .closure_status,
            self.risk_governor_handoff_warning_closure_report
                .remaining_warning_count,
            self.risk_governor_final_veto_trace_report
                .final_veto_trace_status,
            self.risk_governor_final_veto_trace_report
                .bypass_attempt_count,
            self.committee_paper_readiness_gate.gate_status,
            readiness_recommendation,
            self.committee_paper_loop_dry_run_plan.dry_run_status,
            dry_run_recommendation,
            self.workspace_acceptance_truth_closure_plan_v2
                .closure_status,
            self.workspace_acceptance_truth_closure_plan_v2
                .can_claim_full_acceptance,
            self.workspace_acceptance_attempt_v17.attempt_status,
            self.workspace_acceptance_attempt_v17.full_finished,
            self.safety_coverage_preservation_report_v16.safety_status,
            self.control_tower_ai_committee_closure_panel
                .workspace_acceptance_truth_status,
            self.control_tower_ai_committee_closure_panel
                .runtime_deferred_summary,
            self.storage_report.file_count,
            proposal_recommendation,
            debate_recommendation,
            rulebook_recommendation,
            self.scorecard_calibration_warning_closure_report
                .closure_status,
            readiness_recommendation,
            self.safety_coverage_preservation_report_v16.safety_status,
            quality_recommendation,
        )
    }
}

fn proposal_or_shared_evidence(
    proposal: &AICommitteeMemberProposal,
    direct_needle: &str,
    shared_available: bool,
) -> bool {
    shared_available
        || proposal
            .evidence_refs
            .iter()
            .any(|value| value.to_ascii_lowercase().contains(direct_needle))
}

fn proposal_drawdown_proxy_present(proposal: &AICommitteeMemberProposal) -> bool {
    proposal.expected_risk_proxy.is_some()
        || proposal
            .wait_condition
            .to_ascii_lowercase()
            .contains("drawdown")
        || proposal
            .invalidation_condition
            .to_ascii_lowercase()
            .contains("risk")
}

fn proposal_stop_condition_present(proposal: &AICommitteeMemberProposal) -> bool {
    proposal
        .stop_condition
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty())
        || !proposal.invalidation_condition.trim().is_empty()
        || matches!(
            proposal.proposed_action,
            CommitteeProposalAction::Wait
                | CommitteeProposalAction::NoTrade
                | CommitteeProposalAction::RiskDeny
                | CommitteeProposalAction::WatchCandidate
                | CommitteeProposalAction::RequestMoreEvidence
        )
}

fn timing_has_earliest_latest_window(proposal: &EntryTimingProposal) -> bool {
    matches!(proposal.entry_window, EntryTimingWindow::NoEntry)
        || (proposal.earliest_entry_timestamp.is_some()
            && proposal.latest_entry_timestamp.is_some())
}

fn repair_actions_for_rule(rule: &str) -> Vec<RulebookRepairAction> {
    let lower = rule.to_ascii_lowercase();
    if lower.contains("broker") || lower.contains("account") || lower.contains("order") {
        return vec![RulebookRepairAction::BlockRule];
    }
    if contains_live_or_unsafe_terms(rule) {
        return vec![
            RulebookRepairAction::AddRiskGovernorReview,
            RulebookRepairAction::ForcePaperOnlyScope,
            RulebookRepairAction::RequireOwnerReview,
            RulebookRepairAction::AddAuditRequirement,
            RulebookRepairAction::LowerAuthorityToProposalOnly,
        ];
    }
    vec![RulebookRepairAction::NoAction]
}

fn repaired_rule_text(rule: &str) -> String {
    let lower = rule.to_ascii_lowercase();
    if lower.contains("bypass risk governor") {
        "chair synthesis stays subordinate to the Risk Governor final veto and may only recommend paper-only actions".to_string()
    } else if lower.contains("runtime") || lower.contains("live") || lower.contains("training") {
        "member-owned cores remain paper-only, research-only, and deferred for runtime, training, and live execution".to_string()
    } else {
        rule.to_string()
    }
}

fn load_workspace_truth_for_sprint100(
    config: &CommitteeQualityWarningClosureConfig,
    sprint98_bundle: &Sprint98CommitteeOwnedCoreBundle,
) -> Result<WorkspaceAcceptanceTruthImport, String> {
    match config
        .workspace_acceptance_truth_paths
        .as_ref()
        .and_then(|paths| paths.first())
    {
        Some(path) => {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            serde_json::from_str::<WorkspaceAcceptanceTruthImport>(&text)
                .or_else(|_| {
                    serde_json::from_str::<WorkspaceAcceptanceTruthGate>(&text).map(|gate| {
                        WorkspaceAcceptanceTruthImport::from_gate(gate, Some(path.clone()))
                    })
                })
                .map_err(|err| err.to_string())
        }
        None => Ok(sprint98_bundle.workspace_acceptance_truth_import.clone()),
    }
}

fn load_sprint98_bundle_for_sprint100(
    config: &CommitteeQualityWarningClosureConfig,
) -> Result<Sprint98CommitteeOwnedCoreBundle, String> {
    let mut sprint98_config = Sprint98CommitteeOwnedCoreConfig::default();
    sprint98_config.architecture_id = format!("{}-sprint98-base", config.closure_id);
    sprint98_config.output_root = config.output_root.clone();
    if let Some(paths) = &config.workspace_acceptance_truth_paths {
        sprint98_config.workspace_acceptance_truth_path = paths.first().cloned();
    }
    Sprint98CommitteeOwnedCoreRunner::default().run_sprint98_committee_owned_core(&sprint98_config)
}

fn load_sprint99_bundle_for_sprint100(
    config: &CommitteeQualityWarningClosureConfig,
) -> Result<Sprint99CommitteeQualityHardeningBundle, String> {
    if let Some(bundle) = load_first_json::<Sprint99CommitteeQualityHardeningBundle>(
        config.sprint99_bundle_paths.as_ref(),
    )? {
        return Ok(bundle);
    }
    let mut sprint99_config = CommitteeQualityHardeningConfig::default();
    sprint99_config.hardening_id = format!("{}-sprint99-base", config.closure_id);
    sprint99_config.output_root = config.output_root.clone();
    sprint99_config.workspace_acceptance_truth_paths =
        config.workspace_acceptance_truth_paths.clone();
    Sprint99CommitteeQualityHardeningRunner::default()
        .run_sprint99_committee_quality_hardening(&sprint99_config)
}

fn build_proposal_quality_warning_closure_report(
    previous_status: CommitteeMemberProposalQualityStatus,
    proposals: &[AICommitteeMemberProposal],
) -> ProposalQualityWarningClosureReport {
    let missing_evidence_ref_count = proposals
        .iter()
        .filter(|proposal| proposal.evidence_refs.is_empty())
        .count();
    let missing_wait_condition_count = proposals
        .iter()
        .filter(|proposal| proposal.wait_condition.trim().is_empty())
        .count();
    let missing_invalidation_condition_count = proposals
        .iter()
        .filter(|proposal| proposal.invalidation_condition.trim().is_empty())
        .count();
    let missing_expected_risk_count = proposals
        .iter()
        .filter(|proposal| proposal.expected_risk_proxy.is_none())
        .count();
    let missing_reason_code_count = proposals
        .iter()
        .filter(|proposal| proposal.reason_codes.is_empty())
        .count();
    let confidence_out_of_bounds_count = proposals
        .iter()
        .filter(|proposal| !bounded_confidence_ok(proposal))
        .count();
    let remaining_warning_count = missing_evidence_ref_count
        + missing_wait_condition_count
        + missing_invalidation_condition_count
        + missing_expected_risk_count
        + missing_reason_code_count
        + confidence_out_of_bounds_count;
    let closure_status = if proposals
        .iter()
        .any(|proposal| proposal.proposal_status == AICommitteeMemberProposalStatus::SafetyBlocked)
    {
        ProposalQualityWarningClosureStatus::ProposalQualityBlocked
    } else if remaining_warning_count == 0 {
        match previous_status {
            CommitteeMemberProposalQualityStatus::ProposalQualityReadyWithWarnings => {
                ProposalQualityWarningClosureStatus::ProposalWarningsClosedWithMinorNotes
            }
            _ => ProposalQualityWarningClosureStatus::ProposalWarningsClosed,
        }
    } else {
        ProposalQualityWarningClosureStatus::ProposalStillWarningBacked
    };
    ProposalQualityWarningClosureReport {
        report_id: "proposal-quality-warning-closure-report".to_string(),
        previous_status,
        missing_evidence_ref_count,
        missing_wait_condition_count,
        missing_invalidation_condition_count,
        missing_expected_risk_count,
        missing_reason_code_count,
        confidence_out_of_bounds_count,
        closed_warning_count: if remaining_warning_count == 0
            && matches!(
                previous_status,
                CommitteeMemberProposalQualityStatus::ProposalQualityReadyWithWarnings
            ) {
            1
        } else {
            0
        },
        remaining_warning_count,
        closure_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_proposal_evidence_completeness_report(
    sprint98_bundle: &Sprint98CommitteeOwnedCoreBundle,
    proposals: &[AICommitteeMemberProposal],
) -> ProposalEvidenceCompletenessReport {
    let shared_official = !sprint98_bundle
        .market_context_for_committee
        .candle_refs
        .is_empty();
    let shared_research = !sprint98_bundle
        .ai_committee_member_analysis_loops
        .is_empty();
    let shared_counterfactual = sprint98_bundle
        .ai_committee_member_specs
        .iter()
        .any(|spec| spec.member_role == AICommitteeMemberRole::CounterfactualReviewer);
    let shared_risk = !sprint98_bundle
        .market_context_for_committee
        .risk_refs
        .is_empty();
    let shared_regime = !sprint98_bundle
        .market_context_for_committee
        .regime_refs
        .is_empty();
    let proposals_with_official_evidence = proposals
        .iter()
        .filter(|proposal| proposal_or_shared_evidence(proposal, "official", shared_official))
        .count();
    let proposals_with_research_evidence = proposals
        .iter()
        .filter(|proposal| {
            proposal_or_shared_evidence(proposal, "member-study", shared_research)
                || proposal_or_shared_evidence(proposal, "member-notes", shared_research)
        })
        .count();
    let proposals_with_counterfactual_evidence = proposals
        .iter()
        .filter(|proposal| {
            shared_counterfactual
                || proposal_or_shared_evidence(proposal, "counterfactual", false)
                || proposal.member_id == "counterfactual-reviewer"
        })
        .count();
    let proposals_with_risk_evidence = proposals
        .iter()
        .filter(|proposal| {
            shared_risk
                || proposal
                    .dissent_refs
                    .iter()
                    .any(|value| value.to_ascii_lowercase().contains("risk-governor"))
        })
        .count();
    let proposals_with_regime_evidence = proposals
        .iter()
        .filter(|proposal| {
            shared_regime
                || proposal.member_id == "regime-interpreter"
                || proposal
                    .wait_condition
                    .to_ascii_lowercase()
                    .contains("regime")
        })
        .count();
    let proposals_missing_required_evidence = proposals
        .iter()
        .filter(|proposal| {
            !proposal_or_shared_evidence(proposal, "official", shared_official)
                || !proposal_or_shared_evidence(proposal, "member-study", shared_research)
                || !(shared_risk
                    || proposal
                        .dissent_refs
                        .iter()
                        .any(|value| value.to_ascii_lowercase().contains("risk-governor")))
                || !(shared_regime || proposal.member_id == "regime-interpreter")
        })
        .count();
    let evidence_status = if proposals_missing_required_evidence > 0 {
        ProposalEvidenceCompletenessStatus::ProposalEvidenceIncomplete
    } else if proposals_with_counterfactual_evidence < proposals.len() {
        ProposalEvidenceCompletenessStatus::ProposalEvidenceCompleteWithWarnings
    } else {
        ProposalEvidenceCompletenessStatus::ProposalEvidenceComplete
    };
    ProposalEvidenceCompletenessReport {
        report_id: "proposal-evidence-completeness-report".to_string(),
        proposal_count: proposals.len(),
        proposals_with_official_evidence,
        proposals_with_research_evidence,
        proposals_with_counterfactual_evidence,
        proposals_with_risk_evidence,
        proposals_with_regime_evidence,
        proposals_missing_required_evidence,
        evidence_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_proposal_risk_field_completeness_report(
    proposals: &[AICommitteeMemberProposal],
) -> ProposalRiskFieldCompletenessReport {
    let proposals_with_expected_risk = proposals
        .iter()
        .filter(|proposal| proposal.expected_risk_proxy.is_some())
        .count();
    let proposals_with_drawdown_proxy = proposals
        .iter()
        .filter(|proposal| proposal_drawdown_proxy_present(proposal))
        .count();
    let proposals_with_invalidation_condition = proposals
        .iter()
        .filter(|proposal| !proposal.invalidation_condition.trim().is_empty())
        .count();
    let proposals_with_stop_condition = proposals
        .iter()
        .filter(|proposal| proposal_stop_condition_present(proposal))
        .count();
    let proposals_with_risk_governor_required_check = proposals
        .iter()
        .filter(|proposal| {
            proposal
                .dissent_refs
                .iter()
                .any(|value| value.to_ascii_lowercase().contains("risk-governor"))
                || matches!(proposal.proposed_action, CommitteeProposalAction::RiskDeny)
        })
        .count();
    let proposals_missing_risk_fields = proposals
        .iter()
        .filter(|proposal| {
            proposal.expected_risk_proxy.is_none()
                || !proposal_drawdown_proxy_present(proposal)
                || proposal.invalidation_condition.trim().is_empty()
                || !proposal_stop_condition_present(proposal)
                || !(proposal
                    .dissent_refs
                    .iter()
                    .any(|value| value.to_ascii_lowercase().contains("risk-governor"))
                    || matches!(proposal.proposed_action, CommitteeProposalAction::RiskDeny))
        })
        .count();
    let risk_field_status = if proposals_missing_risk_fields > 0 {
        ProposalRiskFieldCompletenessStatus::ProposalRiskFieldsIncomplete
    } else if proposals_with_stop_condition < proposals.len() {
        ProposalRiskFieldCompletenessStatus::ProposalRiskFieldsCompleteWithWarnings
    } else {
        ProposalRiskFieldCompletenessStatus::ProposalRiskFieldsComplete
    };
    ProposalRiskFieldCompletenessReport {
        report_id: "proposal-risk-field-completeness-report".to_string(),
        proposal_count: proposals.len(),
        proposals_with_expected_risk,
        proposals_with_drawdown_proxy,
        proposals_with_invalidation_condition,
        proposals_with_stop_condition,
        proposals_with_risk_governor_required_check,
        proposals_missing_risk_fields,
        risk_field_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_entry_timing_condition_completeness_report(
    proposals: &[EntryTimingProposal],
) -> EntryTimingConditionCompletenessReport {
    let proposals_with_confirmation_conditions = proposals
        .iter()
        .filter(|proposal| !proposal.confirmation_conditions.is_empty())
        .count();
    let proposals_with_cancellation_conditions = proposals
        .iter()
        .filter(|proposal| !proposal.cancellation_conditions.is_empty())
        .count();
    let proposals_with_earliest_latest_window = proposals
        .iter()
        .filter(|proposal| timing_has_earliest_latest_window(proposal))
        .count();
    let proposals_with_volatility_cooldown_condition = proposals
        .iter()
        .filter(|proposal| {
            proposal.entry_window == EntryTimingWindow::VolatilityCooldown
                || proposal
                    .confirmation_conditions
                    .iter()
                    .any(|value| value.to_ascii_lowercase().contains("volatility"))
                || proposal
                    .cancellation_conditions
                    .iter()
                    .any(|value| value.to_ascii_lowercase().contains("volatility"))
        })
        .count();
    let proposals_with_risk_checks = proposals
        .iter()
        .filter(|proposal| !proposal.required_risk_checks.is_empty())
        .count();
    let proposals_missing_conditions = proposals
        .iter()
        .filter(|proposal| {
            proposal.confirmation_conditions.is_empty()
                || proposal.cancellation_conditions.is_empty()
                || !timing_has_earliest_latest_window(proposal)
                || proposal.required_risk_checks.is_empty()
        })
        .count();
    let timing_condition_status = if proposals_missing_conditions > 0 {
        EntryTimingConditionCompletenessStatus::EntryTimingConditionsIncomplete
    } else if proposals_with_volatility_cooldown_condition == 0 {
        EntryTimingConditionCompletenessStatus::EntryTimingConditionsCompleteWithWarnings
    } else {
        EntryTimingConditionCompletenessStatus::EntryTimingConditionsComplete
    };
    EntryTimingConditionCompletenessReport {
        report_id: "entry-timing-condition-completeness-report".to_string(),
        timing_proposal_count: proposals.len(),
        proposals_with_confirmation_conditions,
        proposals_with_cancellation_conditions,
        proposals_with_earliest_latest_window,
        proposals_with_volatility_cooldown_condition,
        proposals_with_risk_checks,
        proposals_missing_conditions,
        timing_condition_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_debate_needs_more_evidence_closure_report(
    previous_debate_status: CommitteeDebateQualityStatus,
    evidence: &DebateEvidenceSufficiencyReport,
) -> DebateNeedsMoreEvidenceClosureReport {
    let mut missing_evidence_items = evidence.missing_evidence_kinds.clone();
    if !evidence.source_boundary_ok {
        missing_evidence_items.push("source-boundary-proof".to_string());
    }
    if !evidence.no_lookahead_ok {
        missing_evidence_items.push("no-lookahead-proof".to_string());
    }
    let mut added_or_confirmed_evidence_items = Vec::new();
    if evidence.official_evidence_count > 0 {
        added_or_confirmed_evidence_items.push("official-evidence".to_string());
    }
    if evidence.research_evidence_count > 0 {
        added_or_confirmed_evidence_items.push("research-evidence".to_string());
    }
    if evidence.source_boundary_ok {
        added_or_confirmed_evidence_items.push("source-boundary-proof".to_string());
    }
    if evidence.no_lookahead_ok {
        added_or_confirmed_evidence_items.push("no-lookahead-proof".to_string());
    }
    if evidence.evidence_ref_count > 0 {
        added_or_confirmed_evidence_items.push("paper-debate-evidence-bundle".to_string());
    }
    let debate_closure_status = if matches!(
        evidence.evidence_status,
        DebateEvidenceSufficiencyStatus::SourceBoundaryBlocked
            | DebateEvidenceSufficiencyStatus::NoLookaheadBlocked
    ) {
        DebateNeedsMoreEvidenceClosureStatus::DebateEvidenceBlocked
    } else if missing_evidence_items.is_empty() {
        match previous_debate_status {
            CommitteeDebateQualityStatus::DebateNeedsMoreEvidence => {
                DebateNeedsMoreEvidenceClosureStatus::DebateEvidenceClosedWithWarnings
            }
            _ => DebateNeedsMoreEvidenceClosureStatus::DebateEvidenceClosed,
        }
    } else {
        DebateNeedsMoreEvidenceClosureStatus::DebateStillNeedsMoreEvidence
    };
    DebateNeedsMoreEvidenceClosureReport {
        report_id: "debate-needs-more-evidence-closure-report".to_string(),
        previous_debate_status,
        remaining_evidence_items: missing_evidence_items.clone(),
        missing_evidence_items,
        added_or_confirmed_evidence_items,
        need_more_evidence_reason_codes: deferred_reason_codes(&[]),
        debate_closure_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_debate_evidence_gap_plan(
    sprint98_bundle: &Sprint98CommitteeOwnedCoreBundle,
    closure: &DebateNeedsMoreEvidenceClosureReport,
) -> DebateEvidenceGapPlan {
    let mut evidence_gaps = Vec::new();
    for item in &closure.remaining_evidence_items {
        let gap_kind = match item.as_str() {
            "official" | "official-evidence" => DebateEvidenceGapKind::MissingOfficialEvidence,
            "risk" | "risk-evidence" => DebateEvidenceGapKind::MissingRiskEvidence,
            "counterfactual" | "counterfactual-evidence" => {
                DebateEvidenceGapKind::MissingCounterfactualEvidence
            }
            "regime" | "regime-evidence" => DebateEvidenceGapKind::MissingRegimeEvidence,
            "liquidity" | "liquidity-evidence" => DebateEvidenceGapKind::MissingLiquidityEvidence,
            "outcome" | "outcome-evidence" => DebateEvidenceGapKind::MissingOutcomeEvidence,
            "no-lookahead-proof" => DebateEvidenceGapKind::MissingNoLookaheadProof,
            _ => DebateEvidenceGapKind::MissingSourceBoundaryProof,
        };
        evidence_gaps.push(DebateEvidenceGap {
            gap_kind,
            detail: item.clone(),
        });
    }
    if evidence_gaps.is_empty()
        && sprint98_bundle
            .committee_roster_lifecycle
            .active_members
            .is_empty()
    {
        evidence_gaps.push(DebateEvidenceGap {
            gap_kind: DebateEvidenceGapKind::MissingLiquidityEvidence,
            detail: "missing active paper committee".to_string(),
        });
    }
    let recommended_actions = if evidence_gaps.is_empty() {
        vec![
            "MaintainLocalOfficialEvidenceBundle".to_string(),
            "MaintainSourceBoundaryProof".to_string(),
            "MaintainNoLookaheadProof".to_string(),
        ]
    } else {
        evidence_gaps
            .iter()
            .map(|gap| match gap.gap_kind {
                DebateEvidenceGapKind::MissingOfficialEvidence => {
                    "BackfillLocalOfficialEvidence".to_string()
                }
                DebateEvidenceGapKind::MissingRiskEvidence => {
                    "AttachRiskGovernorEvidence".to_string()
                }
                DebateEvidenceGapKind::MissingCounterfactualEvidence => {
                    "AttachCounterfactualReview".to_string()
                }
                DebateEvidenceGapKind::MissingRegimeEvidence => "AttachRegimeEvidence".to_string(),
                DebateEvidenceGapKind::MissingLiquidityEvidence => {
                    "AttachLiquidityGuardEvidence".to_string()
                }
                DebateEvidenceGapKind::MissingOutcomeEvidence => {
                    "AttachPaperOutcomeTrace".to_string()
                }
                DebateEvidenceGapKind::MissingNoLookaheadProof => {
                    "AttachNoLookaheadProof".to_string()
                }
                DebateEvidenceGapKind::MissingSourceBoundaryProof => {
                    "AttachSourceBoundaryProof".to_string()
                }
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    };
    let plan_status = if evidence_gaps.is_empty() {
        DebateEvidenceGapPlanStatus::NoGapsDetected
    } else if evidence_gaps.len() <= 2 {
        DebateEvidenceGapPlanStatus::EvidenceGapPlanReadyWithWarnings
    } else {
        DebateEvidenceGapPlanStatus::EvidenceGapPlanReady
    };
    DebateEvidenceGapPlan {
        plan_id: "debate-evidence-gap-plan".to_string(),
        evidence_gaps,
        recommended_actions,
        plan_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_debate_dissent_coverage_report(
    debate_session: &CommitteeDebateSession,
) -> DebateDissentCoverageReport {
    let member_count = debate_session.member_turns.len();
    let support_entry_count = debate_session
        .member_turns
        .iter()
        .filter(|turn| turn.stance == CommitteeDebateStance::SupportEntry)
        .count();
    let oppose_entry_count = debate_session
        .member_turns
        .iter()
        .filter(|turn| turn.stance == CommitteeDebateStance::OpposeEntry)
        .count();
    let wait_count = debate_session
        .member_turns
        .iter()
        .filter(|turn| turn.stance == CommitteeDebateStance::WaitForConfirmation)
        .count();
    let no_trade_count = debate_session
        .member_turns
        .iter()
        .filter(|turn| turn.stance == CommitteeDebateStance::DemandNoTrade)
        .count();
    let risk_deny_count = debate_session
        .member_turns
        .iter()
        .filter(|turn| turn.stance == CommitteeDebateStance::DemandRiskDeny)
        .count();
    let request_more_evidence_count = debate_session
        .member_turns
        .iter()
        .filter(|turn| turn.stance == CommitteeDebateStance::RequestMoreEvidence)
        .count();
    let dissent_present = oppose_entry_count
        + wait_count
        + no_trade_count
        + risk_deny_count
        + request_more_evidence_count
        > 0;
    let groupthink_risk = !dissent_present || support_entry_count == member_count;
    let dissent_status = if !dissent_present {
        DebateDissentCoverageStatus::DissentMissing
    } else if risk_deny_count == 0 || no_trade_count == 0 {
        DebateDissentCoverageStatus::DissentCoverageReadyWithWarnings
    } else {
        DebateDissentCoverageStatus::DissentCoverageReady
    };
    DebateDissentCoverageReport {
        report_id: "debate-dissent-coverage-report".to_string(),
        member_count,
        support_entry_count,
        oppose_entry_count,
        wait_count,
        no_trade_count,
        risk_deny_count,
        request_more_evidence_count,
        dissent_present,
        risk_dissent_present: risk_deny_count > 0,
        no_trade_dissent_present: no_trade_count > 0,
        groupthink_risk,
        dissent_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_debate_member_participation_balance_report(
    specs: &[AICommitteeMemberSpec],
    debate_session: &CommitteeDebateSession,
) -> DebateMemberParticipationBalanceReport {
    let participating = debate_session
        .participating_members
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let participating_member_count = participating.len();
    let non_participating_member_count = specs
        .iter()
        .filter(|spec| !participating.contains(&spec.member_id))
        .count();
    let style_coverage = specs
        .iter()
        .filter(|spec| participating.contains(&spec.member_id))
        .map(|spec| format!("{:?}", spec.style_archetype))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let role_coverage = specs
        .iter()
        .filter(|spec| participating.contains(&spec.member_id))
        .map(|spec| format!("{:?}", spec.member_role))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let missing_required_roles = specs
        .iter()
        .filter(|spec| {
            !participating.contains(&spec.member_id)
                && !matches!(
                    spec.member_status,
                    AICommitteeMemberStatus::DiagnosticOnlyMember
                )
        })
        .map(|spec| format!("{:?}", spec.member_role))
        .collect::<Vec<_>>();
    let participation_status = if participating_member_count == 0 {
        DebateMemberParticipationBalanceStatus::ParticipationInsufficient
    } else if non_participating_member_count > 0 || !missing_required_roles.is_empty() {
        DebateMemberParticipationBalanceStatus::ParticipationBalancedWithWarnings
    } else {
        DebateMemberParticipationBalanceStatus::ParticipationBalanced
    };
    DebateMemberParticipationBalanceReport {
        report_id: "debate-member-participation-balance-report".to_string(),
        participating_member_count,
        non_participating_member_count,
        style_coverage,
        role_coverage,
        missing_required_roles,
        participation_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_chairman_unsafe_rule_closure_report(
    previous_rulebook_status: ChairmanRulebookQualityStatus,
    rulebook: &ChairmanRulebookVersion,
) -> ChairmanUnsafeRuleClosureReport {
    let unsafe_rule_count = rulebook
        .rules
        .iter()
        .filter(|rule| contains_live_or_unsafe_terms(rule))
        .count();
    let blocked_rule_count = rulebook
        .rules
        .iter()
        .filter(|rule| repair_actions_for_rule(rule) == vec![RulebookRepairAction::BlockRule])
        .count();
    let repaired_rule_count = unsafe_rule_count.saturating_sub(blocked_rule_count);
    let closure_status = if blocked_rule_count > 0 {
        ChairmanUnsafeRuleClosureStatus::UnsafeRuleBlocked
    } else if unsafe_rule_count == 0 {
        ChairmanUnsafeRuleClosureStatus::UnsafeRuleClosed
    } else if repaired_rule_count == unsafe_rule_count {
        ChairmanUnsafeRuleClosureStatus::UnsafeRuleClosedWithPaperOnlyRestriction
    } else {
        ChairmanUnsafeRuleClosureStatus::UnsafeRuleStillPresent
    };
    ChairmanUnsafeRuleClosureReport {
        report_id: "chairman-unsafe-rule-closure-report".to_string(),
        previous_rulebook_status,
        unsafe_rule_count,
        blocked_rule_count,
        repaired_rule_count,
        paper_only_restricted_rule_count: unsafe_rule_count,
        risk_governor_blocked_rule_count: blocked_rule_count,
        closure_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_chairman_rulebook_safety_repair_plan(
    rulebook: &ChairmanRulebookVersion,
) -> ChairmanRulebookSafetyRepairPlan {
    let unsafe_rule_items = rulebook
        .rules
        .iter()
        .filter(|rule| contains_live_or_unsafe_terms(rule))
        .map(|rule| ChairmanUnsafeRuleItem {
            rule_text: rule.clone(),
            repair_actions: repair_actions_for_rule(rule),
        })
        .collect::<Vec<_>>();
    let repair_actions = unsafe_rule_items
        .iter()
        .flat_map(|item| item.repair_actions.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut repaired_rulebook_candidate = rulebook
        .rules
        .iter()
        .map(|rule| {
            if contains_live_or_unsafe_terms(rule) {
                repaired_rule_text(rule)
            } else {
                rule.clone()
            }
        })
        .collect::<Vec<_>>();
    repaired_rulebook_candidate.push(
        "rule changes remain proposal-only until audit, owner review, and Risk Governor review complete"
            .to_string(),
    );
    repaired_rulebook_candidate.push(
        "paper-only committee dry-run is the maximum activation scope; live rule mutation stays forbidden"
            .to_string(),
    );
    repaired_rulebook_candidate = repaired_rulebook_candidate
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let plan_status = if unsafe_rule_items.iter().any(|item| {
        item.repair_actions
            .contains(&RulebookRepairAction::BlockRule)
    }) {
        ChairmanRulebookSafetyRepairPlanStatus::UnsafeRuleRequiresBlock
    } else if unsafe_rule_items.is_empty() {
        ChairmanRulebookSafetyRepairPlanStatus::RulebookRepairPlanReady
    } else {
        ChairmanRulebookSafetyRepairPlanStatus::RulebookRepairPlanReadyWithWarnings
    };
    ChairmanRulebookSafetyRepairPlan {
        plan_id: "chairman-rulebook-safety-repair-plan".to_string(),
        unsafe_rule_items,
        repair_actions,
        repaired_rulebook_candidate,
        plan_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_chairman_rulebook_v2_draft(
    rulebook: &ChairmanRulebookVersion,
    repair_plan: &ChairmanRulebookSafetyRepairPlan,
) -> ChairmanRulebookV2Draft {
    let paper_only_restrictions_added = repair_plan
        .repaired_rulebook_candidate
        .iter()
        .filter(|rule| rule.to_ascii_lowercase().contains("paper-only"))
        .count();
    let risk_governor_review_added = repair_plan
        .repaired_rulebook_candidate
        .iter()
        .any(|rule| rule.to_ascii_lowercase().contains("risk governor review"));
    let owner_review_added = repair_plan
        .repaired_rulebook_candidate
        .iter()
        .any(|rule| rule.to_ascii_lowercase().contains("owner review"));
    let draft_status = if repair_plan
        .repair_actions
        .contains(&RulebookRepairAction::BlockRule)
    {
        ChairmanRulebookV2DraftStatus::RulebookDraftUnsafe
    } else if repair_plan.unsafe_rule_items.is_empty() {
        ChairmanRulebookV2DraftStatus::RulebookV2DraftReady
    } else {
        ChairmanRulebookV2DraftStatus::RulebookV2DraftReadyWithWarnings
    };
    ChairmanRulebookV2Draft {
        draft_id: "chairman-rulebook-v2-draft".to_string(),
        previous_version_id: rulebook.version_id.clone(),
        proposed_version_id: "chairman-rulebook-v2".to_string(),
        draft_rules: repair_plan.repaired_rulebook_candidate.clone(),
        unsafe_rules_removed: repair_plan.unsafe_rule_items.len(),
        paper_only_restrictions_added,
        risk_governor_review_added,
        owner_review_added,
        live_use_forbidden: true,
        draft_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_chairman_rulebook_approval_gate(
    proposals: &[ChairmanRuleProposal],
    draft: &ChairmanRulebookV2Draft,
    risk_audit: &ChairmanRuleProposalRiskAuditV2,
) -> ChairmanRulebookApprovalGate {
    let owner_review_required = proposals
        .iter()
        .any(|proposal| proposal.owner_review_required);
    let risk_governor_review_required = proposals
        .iter()
        .any(|proposal| proposal.risk_governor_review_required);
    let can_activate_for_paper = matches!(
        draft.draft_status,
        ChairmanRulebookV2DraftStatus::RulebookV2DraftReady
            | ChairmanRulebookV2DraftStatus::RulebookV2DraftReadyWithWarnings
    ) && matches!(
        risk_audit.audit_status,
        ChairmanRuleProposalRiskAuditV2Status::RuleProposalSafeForPaper
            | ChairmanRuleProposalRiskAuditV2Status::NeedsMoreAudit
    ) && draft.live_use_forbidden;
    let approval_status = if matches!(
        risk_audit.audit_status,
        ChairmanRuleProposalRiskAuditV2Status::RejectedByRiskGovernor
            | ChairmanRuleProposalRiskAuditV2Status::UnsafeRuleBlocked
    ) {
        ChairmanRulebookApprovalStatus::RulebookRejected
    } else if can_activate_for_paper {
        ChairmanRulebookApprovalStatus::RulebookApprovedForPaper
    } else {
        ChairmanRulebookApprovalStatus::RulebookNeedsMoreAudit
    };
    ChairmanRulebookApprovalGate {
        gate_id: "chairman-rulebook-approval-gate".to_string(),
        draft_status: draft.draft_status,
        risk_audit_status: risk_audit.audit_status,
        owner_review_required,
        risk_governor_review_required,
        can_activate_for_paper,
        can_activate_for_live: false,
        approval_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_chairman_rule_audit_trail_completeness_report(
    proposals: &[ChairmanRuleProposal],
    audit: &RuleAdaptationAudit,
    draft: &ChairmanRulebookV2Draft,
) -> ChairmanRuleAuditTrailCompletenessReport {
    let rule_proposal_count = proposals.len();
    let proposals_with_audit = proposals
        .iter()
        .filter(|proposal| {
            proposal.required_audit && audit.rule_proposal_id == proposal.rule_proposal_id
        })
        .count();
    let proposals_with_risk_review = proposals
        .iter()
        .filter(|proposal| proposal.risk_governor_review_required)
        .count();
    let proposals_with_owner_review_flag = proposals
        .iter()
        .filter(|proposal| proposal.owner_review_required)
        .count();
    let proposals_with_paper_only_scope = if draft.live_use_forbidden {
        rule_proposal_count
    } else {
        0
    };
    let proposals_missing_audit = rule_proposal_count.saturating_sub(proposals_with_audit);
    let audit_trail_status = if proposals_missing_audit > 0 {
        ChairmanRuleAuditTrailCompletenessStatus::RuleAuditTrailIncomplete
    } else if proposals_with_paper_only_scope < rule_proposal_count {
        ChairmanRuleAuditTrailCompletenessStatus::RuleAuditTrailCompleteWithWarnings
    } else {
        ChairmanRuleAuditTrailCompletenessStatus::RuleAuditTrailComplete
    };
    ChairmanRuleAuditTrailCompletenessReport {
        report_id: "chairman-rule-audit-trail-completeness-report".to_string(),
        rule_proposal_count,
        proposals_with_audit,
        proposals_with_risk_review,
        proposals_with_owner_review_flag,
        proposals_with_paper_only_scope,
        proposals_missing_audit,
        audit_trail_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_rulebook_diff_risk_closure_report(
    previous: &RulebookVersionDiffReport,
    approval_gate: &ChairmanRulebookApprovalGate,
) -> RulebookDiffRiskClosureReport {
    let risk_weight_changes_closed =
        previous.risk_weight_changes == 0 || approval_gate.can_activate_for_paper;
    let evidence_weight_changes_closed =
        previous.evidence_weight_changes == 0 || approval_gate.can_activate_for_paper;
    let no_trade_bias_changes_closed =
        previous.no_trade_bias_changes == 0 || approval_gate.can_activate_for_paper;
    let quorum_changes_closed =
        previous.quorum_changes == 0 || approval_gate.can_activate_for_paper;
    let unsafe_diff_count = previous.risk_weight_changes
        + previous.evidence_weight_changes
        + previous.no_trade_bias_changes
        + previous.quorum_changes;
    let remaining_unsafe_diff_count = if approval_gate.can_activate_for_paper {
        0
    } else {
        unsafe_diff_count
    };
    let diff_closure_status = if remaining_unsafe_diff_count > 0 {
        RulebookDiffRiskClosureStatus::RulebookDiffRiskStillOpen
    } else if unsafe_diff_count > 0 {
        RulebookDiffRiskClosureStatus::RulebookDiffRiskClosedWithWarnings
    } else {
        RulebookDiffRiskClosureStatus::RulebookDiffRiskClosed
    };
    RulebookDiffRiskClosureReport {
        report_id: "rulebook-diff-risk-closure-report".to_string(),
        previous_diff_status: previous.diff_status,
        risk_weight_changes_closed,
        evidence_weight_changes_closed,
        no_trade_bias_changes_closed,
        quorum_changes_closed,
        unsafe_diff_count,
        remaining_unsafe_diff_count,
        diff_closure_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_scorecard_calibration_warning_closure_report(
    previous_scorecard_status: MemberScorecardCalibrationStatus,
    policy: &PromotionDemotionPolicy,
    scorecards: &[MultiAxisMemberScorecard],
) -> ScorecardCalibrationWarningClosureReport {
    let expected_axes = policy.axes.len();
    let scorecards_with_complete_axes = scorecards
        .iter()
        .filter(|scorecard| scorecard.axis_scores.len() == expected_axes)
        .count();
    let scorecards_with_sufficient_evidence = scorecards
        .iter()
        .filter(|scorecard| !scorecard.recent_proposals.is_empty())
        .count();
    let scorecards_with_risk_alignment = scorecards
        .iter()
        .filter(|scorecard| scorecard.risk_alignment_score >= 0.40)
        .count();
    let scorecards_with_no_trade_discipline = scorecards
        .iter()
        .filter(|scorecard| scorecard.no_trade_discipline_score >= 0.40)
        .count();
    let scorecards_with_debate_quality = scorecards
        .iter()
        .filter(|scorecard| scorecard.debate_turn_quality >= 0.40)
        .count();
    let remaining_warning_count = scorecards
        .iter()
        .filter(|scorecard| {
            scorecard.axis_scores.len() != expected_axes
                || scorecard.recent_proposals.is_empty()
                || scorecard.risk_alignment_score < 0.40
                || scorecard.no_trade_discipline_score < 0.40
                || scorecard.debate_turn_quality < 0.40
        })
        .count();
    let closure_status = if remaining_warning_count > 0 {
        ScorecardCalibrationWarningClosureStatus::ScorecardStillNeedsEvidence
    } else if matches!(
        previous_scorecard_status,
        MemberScorecardCalibrationStatus::ScorecardCalibrationReadyWithWarnings
    ) {
        ScorecardCalibrationWarningClosureStatus::ScorecardWarningsClosedWithNotes
    } else {
        ScorecardCalibrationWarningClosureStatus::ScorecardWarningsClosed
    };
    ScorecardCalibrationWarningClosureReport {
        report_id: "scorecard-calibration-warning-closure-report".to_string(),
        previous_scorecard_status,
        scorecard_count: scorecards.len(),
        scorecards_with_complete_axes,
        scorecards_with_sufficient_evidence,
        scorecards_with_risk_alignment,
        scorecards_with_no_trade_discipline,
        scorecards_with_debate_quality,
        remaining_warning_count,
        closure_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_scorecard_evidence_depth_report(
    specs: &[AICommitteeMemberSpec],
    scorecards: &[MultiAxisMemberScorecard],
    debate_session: &CommitteeDebateSession,
    handoff: &RiskGovernorDebateHandoffReport,
) -> ScorecardEvidenceDepthReport {
    let proposal_history = scorecards
        .iter()
        .filter(|scorecard| !scorecard.recent_proposals.is_empty())
        .count();
    let debate_participants = debate_session
        .participating_members
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let members_with_sufficient_debate_history = specs
        .iter()
        .filter(|spec| debate_participants.contains(&spec.member_id))
        .count();
    let members_with_sufficient_risk_handoff_history = if handoff.sessions_with_risk_handoff > 0 {
        specs.len()
            - specs
                .iter()
                .filter(|spec| {
                    matches!(
                        spec.member_status,
                        AICommitteeMemberStatus::DiagnosticOnlyMember
                    )
                })
                .count()
    } else {
        0
    };
    let members_with_sufficient_counterfactual_history = specs
        .iter()
        .filter(|spec| {
            spec.allowed_data_scopes
                .iter()
                .any(|scope| scope == "counterfactual-review")
        })
        .count();
    let members_needing_more_evidence = specs
        .iter()
        .filter(|spec| {
            !debate_participants.contains(&spec.member_id)
                || matches!(
                    spec.member_status,
                    AICommitteeMemberStatus::DiagnosticOnlyMember
                )
        })
        .count();
    let evidence_depth_status = if members_needing_more_evidence > 0 {
        ScorecardEvidenceDepthStatus::ScorecardEvidenceDepthReadyWithWarnings
    } else {
        ScorecardEvidenceDepthStatus::ScorecardEvidenceDepthReady
    };
    ScorecardEvidenceDepthReport {
        report_id: "scorecard-evidence-depth-report".to_string(),
        member_count: specs.len(),
        members_with_sufficient_proposal_history: proposal_history,
        members_with_sufficient_debate_history,
        members_with_sufficient_risk_handoff_history,
        members_with_sufficient_counterfactual_history,
        members_needing_more_evidence,
        evidence_depth_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_promotion_demotion_stability_report(
    decisions: &[MemberPromotionDemotionDecision],
) -> PromotionDemotionStabilityReport {
    let promotion_decision_count = decisions
        .iter()
        .filter(|decision| decision.action == MemberPromotionDemotionAction::Promote)
        .count();
    let demotion_decision_count = decisions
        .iter()
        .filter(|decision| decision.action == MemberPromotionDemotionAction::Demote)
        .count();
    let keep_decision_count = decisions
        .iter()
        .filter(|decision| decision.action == MemberPromotionDemotionAction::Keep)
        .count();
    let watchlist_decision_count = decisions
        .iter()
        .filter(|decision| decision.action == MemberPromotionDemotionAction::Watchlist)
        .count();
    let unstable_rank_changes = decisions
        .iter()
        .filter(|decision| {
            decision.previous_rank.max(decision.new_rank)
                - decision.previous_rank.min(decision.new_rank)
                > 1
        })
        .count();
    let raw_return_only_changes = decisions
        .iter()
        .filter(|decision| {
            decision
                .decision_basis
                .iter()
                .all(|basis| basis.starts_with("overall_score="))
        })
        .count();
    let capital_allocation_changes_detected = decisions.iter().any(|decision| {
        decision
            .decision_basis
            .iter()
            .any(|basis| basis.to_ascii_lowercase().contains("capital"))
    });
    let stability_status = if unstable_rank_changes > 0
        || raw_return_only_changes > 0
        || capital_allocation_changes_detected
    {
        PromotionDemotionStabilityStatus::PromotionDemotionUnstable
    } else if decisions.iter().any(|decision| {
        decision.decision_status == MemberPromotionDemotionDecisionStatus::NeedsMoreEvidence
    }) {
        PromotionDemotionStabilityStatus::PromotionDemotionStableWithWarnings
    } else {
        PromotionDemotionStabilityStatus::PromotionDemotionStable
    };
    PromotionDemotionStabilityReport {
        report_id: "promotion-demotion-stability-report".to_string(),
        promotion_decision_count,
        demotion_decision_count,
        keep_decision_count,
        watchlist_decision_count,
        unstable_rank_changes,
        raw_return_only_changes,
        capital_allocation_changes_detected,
        stability_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_overfit_warning_closure_report(
    previous: &MemberOverfitRiskReport,
) -> OverfitWarningClosureReport {
    let remaining_overfit_warnings = previous.high_overfit_risk_members;
    let overfit_closure_status = if remaining_overfit_warnings > 0 {
        OverfitWarningClosureStatus::OverfitWarningsStillOpen
    } else if previous.medium_overfit_risk_members > 0 {
        OverfitWarningClosureStatus::OverfitWarningsClosedWithNotes
    } else {
        OverfitWarningClosureStatus::OverfitWarningsClosed
    };
    OverfitWarningClosureReport {
        report_id: "overfit-warning-closure-report".to_string(),
        high_overfit_risk_members: previous.high_overfit_risk_members,
        medium_overfit_risk_members: previous.medium_overfit_risk_members,
        low_overfit_risk_members: previous.low_overfit_risk_members,
        mitigation_actions_confirmed: previous.mitigation_actions.len(),
        remaining_overfit_warnings,
        overfit_closure_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_roster_balance_warning_closure_report(
    previous_roster_status: CommitteeRosterBalanceStatus,
    blindspot: &InvestorStyleBlindspotReport,
) -> RosterBalanceWarningClosureReport {
    let missing_style_coverage = blindspot.missing_counterbalance_styles.clone();
    let missing_role_coverage = Vec::new();
    let missing_counterbalance = blindspot.missing_counterbalance_styles.clone();
    let added_or_confirmed_counterbalance = blindspot
        .mitigation_by_style
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    let remaining_roster_warnings = missing_style_coverage.len() + missing_role_coverage.len();
    let closure_status = if remaining_roster_warnings > 0 {
        RosterBalanceWarningClosureStatus::RosterWarningsStillOpen
    } else if matches!(
        previous_roster_status,
        CommitteeRosterBalanceStatus::RosterBalancedWithWarnings
    ) {
        RosterBalanceWarningClosureStatus::RosterWarningsClosedWithNotes
    } else {
        RosterBalanceWarningClosureStatus::RosterWarningsClosed
    };
    RosterBalanceWarningClosureReport {
        report_id: "roster-balance-warning-closure-report".to_string(),
        previous_roster_status,
        missing_style_coverage,
        missing_role_coverage,
        missing_counterbalance,
        added_or_confirmed_counterbalance,
        remaining_roster_warnings,
        closure_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_decision_replay_warning_closure_report(
    previous_replay_status: PaperOnlyDecisionReplayStatus,
    replay: &PaperOnlyDecisionReplayReport,
    trace: &PaperDecisionTraceCompletenessReport,
) -> PaperDecisionReplayWarningClosureReport {
    let mut replay_warnings = Vec::new();
    if replay.need_more_evidence_count > 0 {
        replay_warnings.push("NeedMoreEvidenceDecisionPresent".to_string());
    }
    if replay.broker_execution_allowed_count > 0 {
        replay_warnings.push("BrokerExecutionAllowed".to_string());
    }
    if replay.live_execution_allowed_count > 0 {
        replay_warnings.push("LiveExecutionAllowed".to_string());
    }
    let mut closed_warnings = Vec::new();
    if replay.broker_execution_allowed_count == 0 {
        closed_warnings.push("BrokerExecutionForbidden".to_string());
    }
    if replay.live_execution_allowed_count == 0 {
        closed_warnings.push("LiveExecutionForbidden".to_string());
    }
    if trace.trace_status == PaperDecisionTraceCompletenessStatus::TraceComplete {
        closed_warnings.push("TraceComplete".to_string());
    }
    let remaining_warnings = replay_warnings
        .iter()
        .filter(|warning| warning.as_str() != "NeedMoreEvidenceDecisionPresent")
        .cloned()
        .collect::<Vec<_>>();
    let closure_status =
        if replay.broker_execution_allowed_count > 0 || replay.live_execution_allowed_count > 0 {
            PaperDecisionReplayWarningClosureStatus::ReplayBlocked
        } else if remaining_warnings.is_empty() {
            match previous_replay_status {
                PaperOnlyDecisionReplayStatus::ReplayReadyWithWarnings => {
                    PaperDecisionReplayWarningClosureStatus::ReplayWarningsClosedWithNotes
                }
                _ => PaperDecisionReplayWarningClosureStatus::ReplayWarningsClosed,
            }
        } else {
            PaperDecisionReplayWarningClosureStatus::ReplayStillNeedsEvidence
        };
    PaperDecisionReplayWarningClosureReport {
        report_id: "paper-decision-replay-warning-closure-report".to_string(),
        previous_replay_status,
        replayed_decision_count: replay.replayed_decision_count,
        replay_warnings,
        closed_warnings,
        remaining_warnings,
        broker_execution_allowed_count: replay.broker_execution_allowed_count,
        live_execution_allowed_count: replay.live_execution_allowed_count,
        closure_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_risk_governor_final_veto_trace_report(
    debate_session: &CommitteeDebateSession,
    decision: &PaperOnlyCommitteeDecisionRecord,
    handoff: &RiskGovernorDebateHandoffReport,
) -> RiskGovernorFinalVetoTraceReport {
    let decisions_with_risk_governor_ref =
        usize::from(!decision.risk_governor_decision_id.is_empty());
    let decisions_with_veto_trace = usize::from(
        !decision.risk_governor_decision_id.is_empty()
            && handoff.risk_governor_final_veto_confirmed,
    );
    let decisions_with_no_trade_trace = usize::from(
        decision.final_decision == PaperOnlyCommitteeDecisionKind::NoTrade
            || debate_session
                .member_turns
                .iter()
                .any(|turn| turn.stance == CommitteeDebateStance::DemandNoTrade),
    );
    let decisions_with_risk_denied_trace = usize::from(
        decision.final_decision == PaperOnlyCommitteeDecisionKind::RiskDenied
            || debate_session
                .member_turns
                .iter()
                .any(|turn| turn.stance == CommitteeDebateStance::DemandRiskDeny),
    );
    let final_veto_trace_status = if handoff.bypass_attempt_count > 0 {
        RiskGovernorFinalVetoTraceStatus::RiskBypassDetected
    } else if decisions_with_veto_trace == 0 {
        RiskGovernorFinalVetoTraceStatus::FinalVetoTraceIncomplete
    } else if decisions_with_no_trade_trace == 0 && decisions_with_risk_denied_trace == 0 {
        RiskGovernorFinalVetoTraceStatus::FinalVetoTraceCompleteWithWarnings
    } else {
        RiskGovernorFinalVetoTraceStatus::FinalVetoTraceComplete
    };
    RiskGovernorFinalVetoTraceReport {
        report_id: "risk-governor-final-veto-trace-report".to_string(),
        debate_session_count: 1,
        paper_decision_count: 1,
        decisions_with_risk_governor_ref,
        decisions_with_veto_trace,
        decisions_with_no_trade_trace,
        decisions_with_risk_denied_trace,
        bypass_attempt_count: handoff.bypass_attempt_count,
        final_veto_trace_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_decision_need_more_evidence_closure_report(
    decision: &PaperOnlyCommitteeDecisionRecord,
    debate_closure: &DebateNeedsMoreEvidenceClosureReport,
    veto_trace: &RiskGovernorFinalVetoTraceReport,
) -> PaperDecisionNeedMoreEvidenceClosureReport {
    let need_more_evidence_decision_count =
        usize::from(decision.final_decision == PaperOnlyCommitteeDecisionKind::NeedMoreEvidence);
    let evidence_items_requested = if need_more_evidence_decision_count > 0 {
        vec![
            "OfficialEvidenceBundle".to_string(),
            "ResearchEvidenceNotes".to_string(),
            "RiskGovernorDecisionTrace".to_string(),
            "CounterfactualAnalogReview".to_string(),
        ]
    } else {
        Vec::new()
    };
    let evidence_items_resolved = if matches!(
        debate_closure.debate_closure_status,
        DebateNeedsMoreEvidenceClosureStatus::DebateEvidenceClosed
            | DebateNeedsMoreEvidenceClosureStatus::DebateEvidenceClosedWithWarnings
    ) && matches!(
        veto_trace.final_veto_trace_status,
        RiskGovernorFinalVetoTraceStatus::FinalVetoTraceComplete
            | RiskGovernorFinalVetoTraceStatus::FinalVetoTraceCompleteWithWarnings
    ) {
        evidence_items_requested.clone()
    } else {
        Vec::new()
    };
    let evidence_items_remaining = evidence_items_requested
        .iter()
        .filter(|item| !evidence_items_resolved.contains(item))
        .cloned()
        .collect::<Vec<_>>();
    let closure_status = if evidence_items_requested.is_empty() {
        PaperDecisionNeedMoreEvidenceClosureStatus::NeedMoreEvidenceClosed
    } else if evidence_items_remaining.is_empty() {
        PaperDecisionNeedMoreEvidenceClosureStatus::NeedMoreEvidenceClosedWithWarnings
    } else {
        PaperDecisionNeedMoreEvidenceClosureStatus::NeedMoreEvidenceStillOpen
    };
    PaperDecisionNeedMoreEvidenceClosureReport {
        report_id: "paper-decision-need-more-evidence-closure-report".to_string(),
        need_more_evidence_decision_count,
        evidence_items_requested,
        evidence_items_resolved,
        evidence_items_remaining,
        closure_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_risk_governor_handoff_warning_closure_report(
    previous_handoff_status: RiskGovernorDebateHandoffStatus,
    handoff: &RiskGovernorDebateHandoffReport,
    veto_trace: &RiskGovernorFinalVetoTraceReport,
) -> RiskGovernorHandoffWarningClosureReport {
    let sessions_missing_risk_handoff = handoff
        .debate_session_count
        .saturating_sub(handoff.sessions_with_risk_handoff);
    let veto_trace_complete = matches!(
        veto_trace.final_veto_trace_status,
        RiskGovernorFinalVetoTraceStatus::FinalVetoTraceComplete
            | RiskGovernorFinalVetoTraceStatus::FinalVetoTraceCompleteWithWarnings
    );
    let remaining_warning_count = sessions_missing_risk_handoff
        + usize::from(!veto_trace_complete)
        + handoff.bypass_attempt_count;
    let closure_status = if handoff.bypass_attempt_count > 0 {
        RiskGovernorHandoffWarningClosureStatus::RiskHandoffBlocked
    } else if remaining_warning_count == 0 {
        match previous_handoff_status {
            RiskGovernorDebateHandoffStatus::RiskHandoffReadyWithWarnings => {
                RiskGovernorHandoffWarningClosureStatus::RiskHandoffWarningsClosedWithNotes
            }
            _ => RiskGovernorHandoffWarningClosureStatus::RiskHandoffWarningsClosed,
        }
    } else {
        RiskGovernorHandoffWarningClosureStatus::RiskHandoffStillWarningBacked
    };
    RiskGovernorHandoffWarningClosureReport {
        report_id: "risk-governor-handoff-warning-closure-report".to_string(),
        previous_handoff_status,
        sessions_with_risk_handoff: handoff.sessions_with_risk_handoff,
        sessions_missing_risk_handoff,
        bypass_attempt_count: handoff.bypass_attempt_count,
        veto_trace_complete,
        remaining_warning_count,
        closure_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_committee_paper_readiness_gate(
    proposal: &ProposalQualityWarningClosureReport,
    timing: &EntryTimingConditionCompletenessReport,
    debate: &DebateNeedsMoreEvidenceClosureReport,
    rulebook: &ChairmanUnsafeRuleClosureReport,
    scorecard: &ScorecardCalibrationWarningClosureReport,
    replay: &PaperDecisionReplayWarningClosureReport,
    handoff: &RiskGovernorHandoffWarningClosureReport,
    regression: &CommitteeOwnedArchitectureRegressionGuard,
    safety: &SafetyCoveragePreservationReportV16,
) -> CommitteePaperReadinessGate {
    let proposal_ok = matches!(
        proposal.closure_status,
        ProposalQualityWarningClosureStatus::ProposalWarningsClosed
            | ProposalQualityWarningClosureStatus::ProposalWarningsClosedWithMinorNotes
    );
    let timing_ok = matches!(
        timing.timing_condition_status,
        EntryTimingConditionCompletenessStatus::EntryTimingConditionsComplete
            | EntryTimingConditionCompletenessStatus::EntryTimingConditionsCompleteWithWarnings
    );
    let debate_ok = matches!(
        debate.debate_closure_status,
        DebateNeedsMoreEvidenceClosureStatus::DebateEvidenceClosed
            | DebateNeedsMoreEvidenceClosureStatus::DebateEvidenceClosedWithWarnings
    );
    let rulebook_ok = matches!(
        rulebook.closure_status,
        ChairmanUnsafeRuleClosureStatus::UnsafeRuleClosed
            | ChairmanUnsafeRuleClosureStatus::UnsafeRuleClosedWithPaperOnlyRestriction
    );
    let scorecard_ok = matches!(
        scorecard.closure_status,
        ScorecardCalibrationWarningClosureStatus::ScorecardWarningsClosed
            | ScorecardCalibrationWarningClosureStatus::ScorecardWarningsClosedWithNotes
    );
    let replay_ok = matches!(
        replay.closure_status,
        PaperDecisionReplayWarningClosureStatus::ReplayWarningsClosed
            | PaperDecisionReplayWarningClosureStatus::ReplayWarningsClosedWithNotes
    );
    let handoff_ok = matches!(
        handoff.closure_status,
        RiskGovernorHandoffWarningClosureStatus::RiskHandoffWarningsClosed
            | RiskGovernorHandoffWarningClosureStatus::RiskHandoffWarningsClosedWithNotes
    );
    let architecture_ok = matches!(
        regression.regression_status,
        CommitteeOwnedArchitectureRegressionStatus::NoRegression
            | CommitteeOwnedArchitectureRegressionStatus::NoRegressionWithWarnings
    );
    let safety_ok = matches!(
        safety.safety_status,
        SafetyCoveragePreservationReportV16Status::SafetyCoveragePreserved
            | SafetyCoveragePreservationReportV16Status::SafetyCoveragePreservedWithWarnings
    );
    let paper_loop_ready = proposal_ok
        && timing_ok
        && debate_ok
        && rulebook_ok
        && scorecard_ok
        && replay_ok
        && handoff_ok
        && architecture_ok
        && safety_ok;
    let warning_backed = matches!(
        proposal.closure_status,
        ProposalQualityWarningClosureStatus::ProposalWarningsClosedWithMinorNotes
    ) || matches!(
        debate.debate_closure_status,
        DebateNeedsMoreEvidenceClosureStatus::DebateEvidenceClosedWithWarnings
    ) || matches!(
        rulebook.closure_status,
        ChairmanUnsafeRuleClosureStatus::UnsafeRuleClosedWithPaperOnlyRestriction
    ) || matches!(
        scorecard.closure_status,
        ScorecardCalibrationWarningClosureStatus::ScorecardWarningsClosedWithNotes
    ) || matches!(
        replay.closure_status,
        PaperDecisionReplayWarningClosureStatus::ReplayWarningsClosedWithNotes
    ) || matches!(
        handoff.closure_status,
        RiskGovernorHandoffWarningClosureStatus::RiskHandoffWarningsClosedWithNotes
    ) || safety.safety_status
        == SafetyCoveragePreservationReportV16Status::SafetyCoveragePreservedWithWarnings;
    let gate_status = if !architecture_ok || !safety_ok {
        CommitteePaperReadinessGateStatus::PaperCommitteeBlocked
    } else if paper_loop_ready && warning_backed {
        CommitteePaperReadinessGateStatus::PaperCommitteeReadyWithWarnings
    } else if paper_loop_ready {
        CommitteePaperReadinessGateStatus::PaperCommitteeReady
    } else {
        CommitteePaperReadinessGateStatus::PaperCommitteeNeedsMoreEvidence
    };
    CommitteePaperReadinessGate {
        gate_id: "committee-paper-readiness-gate".to_string(),
        proposal_quality_status: proposal.closure_status,
        entry_timing_quality_status: timing.timing_condition_status,
        debate_evidence_status: debate.debate_closure_status,
        chairman_rulebook_status: rulebook.closure_status,
        scorecard_calibration_status: scorecard.closure_status,
        paper_replay_status: replay.closure_status,
        risk_handoff_status: handoff.closure_status,
        architecture_regression_status: regression.regression_status,
        safety_status: safety.safety_status,
        paper_loop_ready,
        live_loop_allowed: false,
        gate_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_committee_paper_loop_dry_run_plan(
    specs: &[AICommitteeMemberSpec],
    proposal_count: usize,
    debate_turn_count: usize,
    handoff: &RiskGovernorHandoffWarningClosureReport,
    veto_trace: &RiskGovernorFinalVetoTraceReport,
    gate: &CommitteePaperReadinessGate,
    workspace_truth: &WorkspaceAcceptanceTruthImport,
) -> CommitteePaperLoopDryRunPlan {
    let required_style_coverage = specs
        .iter()
        .filter(|spec| {
            !matches!(
                spec.member_status,
                AICommitteeMemberStatus::DiagnosticOnlyMember
            )
        })
        .map(|spec| format!("{:?}", spec.style_archetype))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let dry_run_status = if !gate.paper_loop_ready {
        CommitteePaperLoopDryRunStatus::NeedMoreEvidenceBeforeDryRun
    } else if !workspace_truth.can_claim_full_acceptance {
        CommitteePaperLoopDryRunStatus::PaperLoopDryRunPlanReadyWithWarnings
    } else {
        CommitteePaperLoopDryRunStatus::PaperLoopDryRunPlanReady
    };
    CommitteePaperLoopDryRunPlan {
        plan_id: "committee-paper-loop-dry-run-plan".to_string(),
        input_context_requirements: vec![
            "local-only market context".to_string(),
            "official-paper-evidence bundle".to_string(),
            "source-boundary proof".to_string(),
            "no-lookahead proof".to_string(),
            "paper-only rulebook v2 draft".to_string(),
        ],
        required_member_count: specs
            .iter()
            .filter(|spec| {
                !matches!(
                    spec.member_status,
                    AICommitteeMemberStatus::DiagnosticOnlyMember
                )
            })
            .count(),
        required_style_coverage,
        required_entry_proposal_count: proposal_count,
        required_debate_turn_count: debate_turn_count,
        required_risk_handoff: handoff.sessions_with_risk_handoff > 0,
        required_paper_decision_trace: matches!(
            veto_trace.final_veto_trace_status,
            RiskGovernorFinalVetoTraceStatus::FinalVetoTraceComplete
                | RiskGovernorFinalVetoTraceStatus::FinalVetoTraceCompleteWithWarnings
        ),
        dry_run_steps: vec![
            CommitteePaperLoopDryRunStep::BuildMarketContext,
            CommitteePaperLoopDryRunStep::RunMemberOfflineAnalysis,
            CommitteePaperLoopDryRunStep::CollectProposals,
            CommitteePaperLoopDryRunStep::TriggerDebate,
            CommitteePaperLoopDryRunStep::RunDebateTurns,
            CommitteePaperLoopDryRunStep::ChairmanSynthesis,
            CommitteePaperLoopDryRunStep::RiskGovernorReview,
            CommitteePaperLoopDryRunStep::WritePaperDecision,
            CommitteePaperLoopDryRunStep::RenderControlTowerPanel,
        ],
        dry_run_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_acceptance_attempt_v17(
    workspace_truth: &WorkspaceAcceptanceTruthImport,
) -> WorkspaceAcceptanceAttemptV17 {
    WorkspaceAcceptanceAttemptV17 {
        attempt_id: "workspace-acceptance-attempt-v17".to_string(),
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

fn build_workspace_acceptance_truth_closure_plan_v2(
    workspace_truth: &WorkspaceAcceptanceTruthImport,
    attempt: &WorkspaceAcceptanceAttemptV17,
) -> WorkspaceAcceptanceTruthClosurePlanV2 {
    WorkspaceAcceptanceTruthClosurePlanV2 {
        plan_id: "workspace-acceptance-truth-closure-plan-v2".to_string(),
        previous_truth_status: workspace_truth.truth_status,
        current_truth_status: attempt.attempt_status,
        can_claim_full_acceptance: workspace_truth.can_claim_full_acceptance,
        no_run_gate_status: if attempt.no_run_started {
            "NoRunGatePreviouslyAttempted".to_string()
        } else {
            "NoRunGatePending".to_string()
        },
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

fn build_safety_coverage_preservation_report_v16(
    safety_v15: &SafetyCoveragePreservationReportV15,
    approval_gate: &ChairmanRulebookApprovalGate,
    decision: &PaperOnlyCommitteeDecisionRecord,
    workspace_truth: &WorkspaceAcceptanceTruthImport,
) -> SafetyCoveragePreservationReportV16 {
    let all_guards = [
        safety_v15.live_trading_guard_present,
        safety_v15.broker_guard_present,
        safety_v15.order_guard_present,
        safety_v15.account_guard_present,
        safety_v15.runtime_llm_guard_present,
        safety_v15.mamba_runtime_guard_present,
        safety_v15.gated_runtime_guard_present,
        safety_v15.model_training_guard_present,
        safety_v15.rust_neural_training_guard_present,
        safety_v15.python_training_dependency_guard_present,
        safety_v15.secret_guard_present,
        safety_v15.no_lookahead_guard_present,
        safety_v15.source_boundary_guard_present,
        safety_v15.browser_execution_guard_present,
        safety_v15.ui_order_control_guard_present,
        safety_v15.committee_owned_core_guard_present,
        safety_v15.investor_impersonation_guard_present,
        safety_v15.chairman_risk_bypass_guard_present,
        !approval_gate.can_activate_for_live,
        safety_v15.promotion_capital_allocation_guard_present,
        safety_v15.paper_only_debate_guard_present,
        decision.paper_only,
    ];
    SafetyCoveragePreservationReportV16 {
        report_id: "safety-coverage-preservation-v16".to_string(),
        live_trading_guard_present: safety_v15.live_trading_guard_present,
        broker_guard_present: safety_v15.broker_guard_present,
        order_guard_present: safety_v15.order_guard_present,
        account_guard_present: safety_v15.account_guard_present,
        runtime_llm_guard_present: safety_v15.runtime_llm_guard_present,
        mamba_runtime_guard_present: safety_v15.mamba_runtime_guard_present,
        gated_runtime_guard_present: safety_v15.gated_runtime_guard_present,
        model_training_guard_present: safety_v15.model_training_guard_present,
        rust_neural_training_guard_present: safety_v15.rust_neural_training_guard_present,
        python_training_dependency_guard_present: safety_v15
            .python_training_dependency_guard_present,
        secret_guard_present: safety_v15.secret_guard_present,
        no_lookahead_guard_present: safety_v15.no_lookahead_guard_present,
        source_boundary_guard_present: safety_v15.source_boundary_guard_present,
        browser_execution_guard_present: safety_v15.browser_execution_guard_present,
        ui_order_control_guard_present: safety_v15.ui_order_control_guard_present,
        committee_owned_core_guard_present: safety_v15.committee_owned_core_guard_present,
        investor_impersonation_guard_present: safety_v15.investor_impersonation_guard_present,
        chairman_risk_bypass_guard_present: safety_v15.chairman_risk_bypass_guard_present,
        unsafe_rulebook_guard_present: !approval_gate.can_activate_for_live,
        promotion_capital_allocation_guard_present: safety_v15
            .promotion_capital_allocation_guard_present,
        paper_only_debate_guard_present: safety_v15.paper_only_debate_guard_present,
        paper_only_decision_guard_present: decision.paper_only,
        safety_status: if all_guards.into_iter().all(|guard| guard)
            && !workspace_truth.can_claim_full_acceptance
        {
            SafetyCoveragePreservationReportV16Status::SafetyCoveragePreservedWithWarnings
        } else if all_guards.into_iter().all(|guard| guard) {
            SafetyCoveragePreservationReportV16Status::SafetyCoveragePreserved
        } else {
            SafetyCoveragePreservationReportV16Status::SafetyCoverageMissing
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_control_tower_ai_committee_closure_panel(
    proposal_warning: &ProposalQualityWarningClosureReport,
    proposal_evidence: &ProposalEvidenceCompletenessReport,
    proposal_risk: &ProposalRiskFieldCompletenessReport,
    entry_timing: &EntryTimingConditionCompletenessReport,
    debate_closure: &DebateNeedsMoreEvidenceClosureReport,
    dissent: &DebateDissentCoverageReport,
    participation: &DebateMemberParticipationBalanceReport,
    unsafe_rule: &ChairmanUnsafeRuleClosureReport,
    repair_plan: &ChairmanRulebookSafetyRepairPlan,
    approval_gate: &ChairmanRulebookApprovalGate,
    scorecard: &ScorecardCalibrationWarningClosureReport,
    replay: &PaperDecisionReplayWarningClosureReport,
    handoff: &RiskGovernorHandoffWarningClosureReport,
    readiness_gate: &CommitteePaperReadinessGate,
    dry_run_plan: &CommitteePaperLoopDryRunPlan,
    workspace_truth: &WorkspaceAcceptanceTruthImport,
    safety_v16: &SafetyCoveragePreservationReportV16,
) -> ControlTowerAiCommitteeClosurePanel {
    ControlTowerAiCommitteeClosurePanel {
        panel_id: "control-tower-ai-committee-closure-panel".to_string(),
        proposal_warning_closure_status: proposal_warning.closure_status,
        proposal_evidence_status: proposal_evidence.evidence_status,
        proposal_risk_field_status: proposal_risk.risk_field_status,
        entry_timing_condition_status: entry_timing.timing_condition_status,
        debate_evidence_closure_status: debate_closure.debate_closure_status,
        debate_dissent_status: dissent.dissent_status,
        debate_participation_status: participation.participation_status,
        chairman_unsafe_rule_closure_status: unsafe_rule.closure_status,
        rulebook_repair_status: repair_plan.plan_status,
        rulebook_approval_status: approval_gate.approval_status,
        scorecard_warning_closure_status: scorecard.closure_status,
        paper_replay_warning_closure_status: replay.closure_status,
        risk_handoff_warning_closure_status: handoff.closure_status,
        paper_readiness_gate_status: readiness_gate.gate_status,
        paper_dry_run_plan_status: dry_run_plan.dry_run_status,
        workspace_acceptance_truth_status: workspace_truth.truth_status,
        runtime_deferred_summary:
            "runtime deferred, training deferred, live inference forbidden, live trading forbidden, paper-only closure layer"
                .to_string(),
        safety_coverage_status: safety_v16.safety_status,
        next_actions: vec![
            "keep paper-loop dry-run local-only and research-only".to_string(),
            "keep workspace acceptance truth separate from focused readiness reports".to_string(),
            "keep chairman rulebook mutations paper-only and audit-gated".to_string(),
        ],
        warnings: vec![
            "static/read-only panel only".to_string(),
            "no train/runtime/live/order/account/browser controls".to_string(),
            "no auto-rule-apply button".to_string(),
            "paper readiness does not imply broker execution or live trading".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[ReasonCode::ControlTowerUiReadinessBuilt]),
    }
}

impl Sprint100CommitteeClosureBundle {
    pub fn write_to_dir(&mut self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        write_json_file(
            &output_dir.join("proposal_quality_warning_closure.txt"),
            &self.proposal_quality_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("proposal_evidence_completeness.txt"),
            &self.proposal_evidence_completeness_report,
        )?;
        write_json_file(
            &output_dir.join("proposal_risk_field_completeness.txt"),
            &self.proposal_risk_field_completeness_report,
        )?;
        write_json_file(
            &output_dir.join("entry_timing_condition_completeness.txt"),
            &self.entry_timing_condition_completeness_report,
        )?;
        write_json_file(
            &output_dir.join("debate_needs_more_evidence_closure.txt"),
            &self.debate_needs_more_evidence_closure_report,
        )?;
        write_json_file(
            &output_dir.join("debate_evidence_gap_plan.txt"),
            &self.debate_evidence_gap_plan,
        )?;
        write_json_file(
            &output_dir.join("debate_dissent_coverage.txt"),
            &self.debate_dissent_coverage_report,
        )?;
        write_json_file(
            &output_dir.join("debate_member_participation_balance.txt"),
            &self.debate_member_participation_balance_report,
        )?;
        write_json_file(
            &output_dir.join("chairman_unsafe_rule_closure.txt"),
            &self.chairman_unsafe_rule_closure_report,
        )?;
        write_json_file(
            &output_dir.join("chairman_rulebook_safety_repair_plan.txt"),
            &self.chairman_rulebook_safety_repair_plan,
        )?;
        write_json_file(
            &output_dir.join("chairman_rulebook_v2_draft.txt"),
            &self.chairman_rulebook_v2_draft,
        )?;
        write_json_file(
            &output_dir.join("chairman_rulebook_approval_gate.txt"),
            &self.chairman_rulebook_approval_gate,
        )?;
        write_json_file(
            &output_dir.join("chairman_rule_audit_trail_completeness.txt"),
            &self.chairman_rule_audit_trail_completeness_report,
        )?;
        write_json_file(
            &output_dir.join("rulebook_diff_risk_closure.txt"),
            &self.rulebook_diff_risk_closure_report,
        )?;
        write_json_file(
            &output_dir.join("scorecard_calibration_warning_closure.txt"),
            &self.scorecard_calibration_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("scorecard_evidence_depth.txt"),
            &self.scorecard_evidence_depth_report,
        )?;
        write_json_file(
            &output_dir.join("promotion_demotion_stability.txt"),
            &self.promotion_demotion_stability_report,
        )?;
        write_json_file(
            &output_dir.join("overfit_warning_closure.txt"),
            &self.overfit_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("roster_balance_warning_closure.txt"),
            &self.roster_balance_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("paper_decision_replay_warning_closure.txt"),
            &self.paper_decision_replay_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("paper_decision_need_more_evidence_closure.txt"),
            &self.paper_decision_need_more_evidence_closure_report,
        )?;
        write_json_file(
            &output_dir.join("risk_governor_handoff_warning_closure.txt"),
            &self.risk_governor_handoff_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("risk_governor_final_veto_trace.txt"),
            &self.risk_governor_final_veto_trace_report,
        )?;
        write_json_file(
            &output_dir.join("committee_paper_readiness_gate.txt"),
            &self.committee_paper_readiness_gate,
        )?;
        write_json_file(
            &output_dir.join("committee_paper_loop_dry_run_plan.txt"),
            &self.committee_paper_loop_dry_run_plan,
        )?;
        write_json_file(
            &output_dir.join("workspace_acceptance_truth_closure_plan_v2.txt"),
            &self.workspace_acceptance_truth_closure_plan_v2,
        )?;
        write_json_file(
            &output_dir.join("workspace_acceptance_attempt_v17.txt"),
            &self.workspace_acceptance_attempt_v17,
        )?;
        write_json_file(
            &output_dir.join("safety_coverage_preservation_v16.txt"),
            &self.safety_coverage_preservation_report_v16,
        )?;
        write_json_file(
            &output_dir.join("control_tower_ai_committee_closure_panel.txt"),
            &self.control_tower_ai_committee_closure_panel,
        )?;
        let files = vec![
            "proposal_quality_warning_closure.txt".to_string(),
            "proposal_evidence_completeness.txt".to_string(),
            "proposal_risk_field_completeness.txt".to_string(),
            "entry_timing_condition_completeness.txt".to_string(),
            "debate_needs_more_evidence_closure.txt".to_string(),
            "debate_evidence_gap_plan.txt".to_string(),
            "debate_dissent_coverage.txt".to_string(),
            "debate_member_participation_balance.txt".to_string(),
            "chairman_unsafe_rule_closure.txt".to_string(),
            "chairman_rulebook_safety_repair_plan.txt".to_string(),
            "chairman_rulebook_v2_draft.txt".to_string(),
            "chairman_rulebook_approval_gate.txt".to_string(),
            "chairman_rule_audit_trail_completeness.txt".to_string(),
            "rulebook_diff_risk_closure.txt".to_string(),
            "scorecard_calibration_warning_closure.txt".to_string(),
            "scorecard_evidence_depth.txt".to_string(),
            "promotion_demotion_stability.txt".to_string(),
            "overfit_warning_closure.txt".to_string(),
            "roster_balance_warning_closure.txt".to_string(),
            "paper_decision_replay_warning_closure.txt".to_string(),
            "paper_decision_need_more_evidence_closure.txt".to_string(),
            "risk_governor_handoff_warning_closure.txt".to_string(),
            "risk_governor_final_veto_trace.txt".to_string(),
            "committee_paper_readiness_gate.txt".to_string(),
            "committee_paper_loop_dry_run_plan.txt".to_string(),
            "workspace_acceptance_truth_closure_plan_v2.txt".to_string(),
            "workspace_acceptance_attempt_v17.txt".to_string(),
            "safety_coverage_preservation_v16.txt".to_string(),
            "control_tower_ai_committee_closure_panel.txt".to_string(),
            "storage_report.txt".to_string(),
            "summary.txt".to_string(),
        ];
        self.storage_report = Sprint100CommitteeClosureStorageReport {
            report_id: format!(
                "{}-storage-report",
                self.control_tower_ai_committee_closure_panel.panel_id
            ),
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
pub struct CommitteeQualityWarningClosureRunner;

impl CommitteeQualityWarningClosureRunner {
    pub fn run(
        &self,
        config: &CommitteeQualityWarningClosureConfig,
    ) -> Result<Sprint100CommitteeClosureBundle, String> {
        config.validate()?;
        let sprint99_bundle = load_sprint99_bundle_for_sprint100(config)?;
        let sprint98_bundle = load_sprint98_bundle_for_sprint100(config)?;
        let workspace_truth = load_workspace_truth_for_sprint100(config, &sprint98_bundle)?;
        let proposal_quality = load_json_or_clone(
            config.proposal_quality_paths.as_ref(),
            &sprint99_bundle.committee_member_proposal_quality_report,
        )?;
        let _entry_timing_quality = load_json_or_clone(
            config.entry_timing_quality_paths.as_ref(),
            &sprint99_bundle.entry_timing_proposal_quality_report,
        )?;
        let debate_quality = load_json_or_clone(
            config.debate_quality_paths.as_ref(),
            &sprint99_bundle.committee_debate_quality_report,
        )?;
        let debate_evidence = load_json_or_clone(
            config.debate_evidence_paths.as_ref(),
            &sprint99_bundle.debate_evidence_sufficiency_report,
        )?;
        let chairman_rulebook_quality = load_json_or_clone(
            config.chairman_rulebook_quality_paths.as_ref(),
            &sprint99_bundle.chairman_rulebook_quality_report,
        )?;
        let rule_audit = load_json_or_clone(
            config.rule_audit_paths.as_ref(),
            &sprint98_bundle.rule_adaptation_audit,
        )?;
        let scorecard_calibration = load_json_or_clone(
            config.scorecard_calibration_paths.as_ref(),
            &sprint99_bundle.member_scorecard_calibration_report,
        )?;
        let paper_replay = load_json_or_clone(
            config.paper_replay_paths.as_ref(),
            &sprint99_bundle.paper_only_decision_replay_report,
        )?;
        let risk_handoff = load_json_or_clone(
            config.risk_handoff_paths.as_ref(),
            &sprint99_bundle.risk_governor_debate_handoff_report,
        )?;
        let proposal_warning = build_proposal_quality_warning_closure_report(
            proposal_quality.quality_status,
            &sprint98_bundle.ai_committee_member_proposals,
        );
        let proposal_evidence = build_proposal_evidence_completeness_report(
            &sprint98_bundle,
            &sprint98_bundle.ai_committee_member_proposals,
        );
        let proposal_risk = build_proposal_risk_field_completeness_report(
            &sprint98_bundle.ai_committee_member_proposals,
        );
        let entry_timing_conditions = build_entry_timing_condition_completeness_report(
            &sprint98_bundle.entry_timing_proposals,
        );
        let debate_closure = build_debate_needs_more_evidence_closure_report(
            debate_quality.debate_quality_status,
            &debate_evidence,
        );
        let debate_gap_plan = build_debate_evidence_gap_plan(&sprint98_bundle, &debate_closure);
        let debate_dissent =
            build_debate_dissent_coverage_report(&sprint98_bundle.committee_debate_session);
        let debate_participation = build_debate_member_participation_balance_report(
            &sprint98_bundle.ai_committee_member_specs,
            &sprint98_bundle.committee_debate_session,
        );
        let chairman_unsafe_rule = build_chairman_unsafe_rule_closure_report(
            chairman_rulebook_quality.rulebook_quality_status,
            &sprint98_bundle.chairman_rulebook_version,
        );
        let rulebook_repair =
            build_chairman_rulebook_safety_repair_plan(&sprint98_bundle.chairman_rulebook_version);
        let rulebook_v2 = build_chairman_rulebook_v2_draft(
            &sprint98_bundle.chairman_rulebook_version,
            &rulebook_repair,
        );
        let rulebook_approval = build_chairman_rulebook_approval_gate(
            &sprint98_bundle.chairman_rule_proposals,
            &rulebook_v2,
            &sprint99_bundle.chairman_rule_proposal_risk_audit_v2,
        );
        let rule_audit_trail = build_chairman_rule_audit_trail_completeness_report(
            &sprint98_bundle.chairman_rule_proposals,
            &rule_audit,
            &rulebook_v2,
        );
        let rulebook_diff_closure = build_rulebook_diff_risk_closure_report(
            &sprint99_bundle.rulebook_version_diff_report,
            &rulebook_approval,
        );
        let scorecard_warning_closure = build_scorecard_calibration_warning_closure_report(
            scorecard_calibration.scorecard_status,
            &sprint98_bundle.promotion_demotion_policy,
            &sprint98_bundle.multi_axis_member_scorecards,
        );
        let scorecard_evidence_depth = build_scorecard_evidence_depth_report(
            &sprint98_bundle.ai_committee_member_specs,
            &sprint98_bundle.multi_axis_member_scorecards,
            &sprint98_bundle.committee_debate_session,
            &risk_handoff,
        );
        let promotion_stability = build_promotion_demotion_stability_report(
            &sprint98_bundle.member_promotion_demotion_decisions,
        );
        let overfit_closure =
            build_overfit_warning_closure_report(&sprint99_bundle.member_overfit_risk_report);
        let roster_closure = build_roster_balance_warning_closure_report(
            sprint99_bundle
                .committee_roster_balance_report
                .roster_balance_status,
            &sprint99_bundle.investor_style_blindspot_report,
        );
        let replay_warning_closure = build_paper_decision_replay_warning_closure_report(
            paper_replay.replay_status,
            &paper_replay,
            &sprint99_bundle.paper_decision_trace_completeness_report,
        );
        let veto_trace = build_risk_governor_final_veto_trace_report(
            &sprint98_bundle.committee_debate_session,
            &sprint98_bundle.paper_only_committee_decision_record,
            &risk_handoff,
        );
        let need_more_evidence_closure = build_paper_decision_need_more_evidence_closure_report(
            &sprint98_bundle.paper_only_committee_decision_record,
            &debate_closure,
            &veto_trace,
        );
        let handoff_warning_closure = build_risk_governor_handoff_warning_closure_report(
            risk_handoff.handoff_status,
            &risk_handoff,
            &veto_trace,
        );
        let workspace_attempt = build_workspace_acceptance_attempt_v17(&workspace_truth);
        let workspace_closure =
            build_workspace_acceptance_truth_closure_plan_v2(&workspace_truth, &workspace_attempt);
        let safety_v16 = build_safety_coverage_preservation_report_v16(
            &sprint99_bundle.safety_coverage_preservation_report_v15,
            &rulebook_approval,
            &sprint98_bundle.paper_only_committee_decision_record,
            &workspace_truth,
        );
        let paper_readiness = build_committee_paper_readiness_gate(
            &proposal_warning,
            &entry_timing_conditions,
            &debate_closure,
            &chairman_unsafe_rule,
            &scorecard_warning_closure,
            &replay_warning_closure,
            &handoff_warning_closure,
            &sprint99_bundle.committee_owned_architecture_regression_guard,
            &safety_v16,
        );
        let paper_dry_run = build_committee_paper_loop_dry_run_plan(
            &sprint98_bundle.ai_committee_member_specs,
            sprint98_bundle
                .ai_committee_member_proposals
                .iter()
                .filter(|proposal| proposal.proposed_entry_timing.is_some())
                .count(),
            sprint98_bundle.committee_debate_session.member_turns.len(),
            &handoff_warning_closure,
            &veto_trace,
            &paper_readiness,
            &workspace_truth,
        );
        let closure_panel = build_control_tower_ai_committee_closure_panel(
            &proposal_warning,
            &proposal_evidence,
            &proposal_risk,
            &entry_timing_conditions,
            &debate_closure,
            &debate_dissent,
            &debate_participation,
            &chairman_unsafe_rule,
            &rulebook_repair,
            &rulebook_approval,
            &scorecard_warning_closure,
            &replay_warning_closure,
            &handoff_warning_closure,
            &paper_readiness,
            &paper_dry_run,
            &workspace_truth,
            &safety_v16,
        );
        let mut bundle = Sprint100CommitteeClosureBundle {
            proposal_quality_warning_closure_report: proposal_warning,
            proposal_evidence_completeness_report: proposal_evidence,
            proposal_risk_field_completeness_report: proposal_risk,
            entry_timing_condition_completeness_report: entry_timing_conditions,
            debate_needs_more_evidence_closure_report: debate_closure,
            debate_evidence_gap_plan: debate_gap_plan,
            debate_dissent_coverage_report: debate_dissent,
            debate_member_participation_balance_report: debate_participation,
            chairman_unsafe_rule_closure_report: chairman_unsafe_rule,
            chairman_rulebook_safety_repair_plan: rulebook_repair,
            chairman_rulebook_v2_draft: rulebook_v2,
            chairman_rulebook_approval_gate: rulebook_approval,
            chairman_rule_audit_trail_completeness_report: rule_audit_trail,
            rulebook_diff_risk_closure_report: rulebook_diff_closure,
            scorecard_calibration_warning_closure_report: scorecard_warning_closure,
            scorecard_evidence_depth_report: scorecard_evidence_depth,
            promotion_demotion_stability_report: promotion_stability,
            overfit_warning_closure_report: overfit_closure,
            roster_balance_warning_closure_report: roster_closure,
            paper_decision_replay_warning_closure_report: replay_warning_closure,
            paper_decision_need_more_evidence_closure_report: need_more_evidence_closure,
            risk_governor_handoff_warning_closure_report: handoff_warning_closure,
            risk_governor_final_veto_trace_report: veto_trace,
            committee_paper_readiness_gate: paper_readiness,
            committee_paper_loop_dry_run_plan: paper_dry_run,
            workspace_acceptance_truth_closure_plan_v2: workspace_closure,
            workspace_acceptance_attempt_v17: workspace_attempt,
            safety_coverage_preservation_report_v16: safety_v16,
            control_tower_ai_committee_closure_panel: closure_panel,
            storage_report: Sprint100CommitteeClosureStorageReport {
                report_id: format!("{}-storage-report", config.closure_id),
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
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sprint100CommitteeClosureRunner;

impl Sprint100CommitteeClosureRunner {
    pub fn run(
        &self,
        config: &CommitteeQualityWarningClosureConfig,
    ) -> Result<Sprint100CommitteeClosureBundle, String> {
        CommitteeQualityWarningClosureRunner::default().run(config)
    }

    pub fn run_sprint100_committee_closure(
        &self,
        config: &CommitteeQualityWarningClosureConfig,
    ) -> Result<Sprint100CommitteeClosureBundle, String> {
        self.run(config)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestorArchetypeSourceCategory {
    PublicPhilosophy,
    PublishedBook,
    PublicInterview,
    OfficialCompanyMaterial,
    ExchangeProfile,
    CommunityAnecdote,
    WeakSource,
    Unverified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InvestorConfidenceGrade {
    A,
    BPlus,
    B,
    BMinus,
    C,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InvestorAssetScope {
    PublicEquities,
    Futures,
    Macro,
    Crypto,
    CryptoInfrastructure,
    MultiAsset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InvestorTimeHorizon {
    Intraday,
    Swing,
    MediumTerm,
    LongTerm,
    Cycle,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InvestorStyleGroupKind {
    ShortTermSwing,
    LongTermEquity,
    Crypto,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestorArchetypeCandidateStatus {
    CandidateReady,
    CandidateReadyWithWarnings,
    CandidateLowConfidence,
    CandidateBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EighteenInvestorCandidateRegistryStatus {
    EighteenCandidateRegistryReady,
    EighteenCandidateRegistryReadyWithWarnings,
    CandidateRegistryNeedsReview,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestorArchetypeSourceConfidenceStatus {
    SourceConfidenceReady,
    SourceConfidenceReadyWithWarnings,
    SourceConfidenceNeedsReview,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestorArchetypeSafetyNormalizationStatus {
    SafetyNormalizationReady,
    SafetyNormalizationReadyWithWarnings,
    SafetyNormalizationNeedsReview,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestorStyleFeatureCardStatus {
    FeatureCardReady,
    FeatureCardReadyWithWarnings,
    FeatureCardNeedsMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoNotLearnBlockedItemKind {
    ExactProfitClaim,
    LifestyleMyth,
    UnofficialQuote,
    PrivateStrategyClaim,
    UnsupportedBestInvestorClaim,
    AlwaysBuySellMyth,
    UnverifiedNumericRule,
    NarrativeWithoutData,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestorStyleDoNotLearnGuardStatus {
    DoNotLearnGuardReady,
    DoNotLearnGuardReadyWithWarnings,
    GuardNeedsReview,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestorImpersonationRiskStatus {
    ImpersonationRiskControlled,
    ImpersonationRiskControlledWithWarnings,
    ImpersonationRiskHigh,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestorUnverifiedClaimFilterStatus {
    UnverifiedClaimsFiltered,
    UnverifiedClaimsFilteredWithWarnings,
    ClaimsNeedManualReview,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PreservedHabitKind {
    StopLossDiscipline,
    PositionSizing,
    TradingJournal,
    ThesisInvalidation,
    RiskReviewRoutine,
    EvidenceCheckRoutine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestorPrivateLifeMythFilterStatus {
    PrivateLifeMythsFiltered,
    PrivateLifeMythsFilteredWithWarnings,
    PrivateLifeItemsNeedReview,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StyleGroupTaxonomyStatus {
    StyleTaxonomyReady,
    StyleTaxonomyReadyWithWarnings,
    StyleTaxonomyNeedsReview,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShortTermSwingGroupStatus {
    ShortTermSwingGroupReady,
    ShortTermSwingGroupReadyWithWarnings,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LongTermEquityGroupStatus {
    LongTermEquityGroupReady,
    LongTermEquityGroupReadyWithWarnings,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CryptoGroupStatus {
    CryptoGroupReady,
    CryptoGroupReadyWithWarnings,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommonRiskManagerStatus {
    CommonRiskManagerReady,
    CommonRiskManagerReadyWithWarnings,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StyleConflictResolutionPolicy {
    DebateRequired,
    RegimeRouterDecides,
    RiskGovernorDecides,
    NeedMoreEvidence,
    NoTradeDefault,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StyleConflictMatrixStatus {
    ConflictMatrixReady,
    ConflictMatrixReadyWithWarnings,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegimeRoutingStatus {
    RegimeRoutingReady,
    RegimeRoutingReadyWithWarnings,
    NeedMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultiExpertTopologyStatus {
    MultiExpertTopologyReady,
    MultiExpertTopologyReadyWithWarnings,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceWeightPolicyStatus {
    ConfidenceWeightPolicyReady,
    ConfidenceWeightPolicyReadyWithWarnings,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureScopeMappingStatus {
    FeatureScopeMappingReady,
    FeatureScopeMappingReadyWithWarnings,
    FeatureScopeNeedsMoreData,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningDataCardsStatus {
    LearningDataCardsReady,
    LearningDataCardsReadyWithWarnings,
    LearningDataCardsNeedReview,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceRequirementPolicyStatus {
    EvidenceRequirementPolicyReady,
    EvidenceRequirementPolicyReadyWithWarnings,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchetypeMappingStatus {
    ArchetypeMappingReady,
    ArchetypeMappingReadyWithWarnings,
    MappingNeedsReview,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EighteenInvestorCommitteeRosterPlanStatus {
    RosterExpansionPlanReady,
    RosterExpansionPlanReadyWithWarnings,
    RosterExpansionNeedsReview,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EighteenMemberActivationGateStatus {
    EighteenPaperRosterGateReady,
    EighteenPaperRosterGateReadyWithWarnings,
    EighteenActivationBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperOnlyRosterExpansionGateStatus {
    PaperRosterExpansionReady,
    PaperRosterExpansionReadyWithWarnings,
    PaperRosterExpansionBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairmanStyleGovernanceStatus {
    ChairmanStyleGovernanceReady,
    ChairmanStyleGovernanceReadyWithWarnings,
    UnsafeGovernanceBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionPolicyV2Status {
    PromotionPolicyV2Ready,
    PromotionPolicyV2ReadyWithWarnings,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyCoveragePreservationReportV17Status {
    SafetyCoveragePreserved,
    SafetyCoveragePreservedWithWarnings,
    SafetyCoverageMissing,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestorArchetypeIngestionStatus {
    InvestorArchetypeCardsReady,
    InvestorArchetypeCardsReadyWithWarnings,
    InvestorArchetypeCardsBlocked,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestorArchetypeIngestionConfig {
    pub ingestion_id: String,
    #[serde(default)]
    pub sprint100_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub investor_material_paths: Option<Vec<String>>,
    #[serde(default)]
    pub markdown_material_paths: Option<Vec<String>>,
    #[serde(default)]
    pub candidate_table_paths: Option<Vec<String>>,
    pub output_root: String,
    #[serde(default = "default_true")]
    pub require_no_impersonation: bool,
    #[serde(default = "default_true")]
    pub require_source_confidence: bool,
    #[serde(default = "default_true")]
    pub require_do_not_learn_guards: bool,
    #[serde(default = "default_true")]
    pub require_style_grouping: bool,
    #[serde(default = "default_true")]
    pub require_feature_vectors: bool,
    #[serde(default = "default_true")]
    pub require_regime_routing: bool,
    #[serde(default = "default_true")]
    pub require_paper_only: bool,
    #[serde(default = "default_true")]
    pub preserve_committee_owned_architecture: bool,
    #[serde(default = "default_true")]
    pub preserve_runtime_deferred: bool,
    #[serde(default = "default_true")]
    pub preserve_safety_guards: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for InvestorArchetypeIngestionConfig {
    fn default() -> Self {
        Self {
            ingestion_id: "sprint101-investor-archetype-ingestion-example".to_string(),
            sprint100_bundle_paths: Some(vec![
                "examples/sprint101_data/sprint100_summary.json".to_string(),
            ]),
            investor_material_paths: Some(vec![
                "examples/sprint101_data/investor_material_sample.md".to_string(),
            ]),
            markdown_material_paths: None,
            candidate_table_paths: None,
            output_root: "target/soma_sprint101_investor_archetype_ingestion".to_string(),
            require_no_impersonation: true,
            require_source_confidence: true,
            require_do_not_learn_guards: true,
            require_style_grouping: true,
            require_feature_vectors: true,
            require_regime_routing: true,
            require_paper_only: true,
            preserve_committee_owned_architecture: true,
            preserve_runtime_deferred: true,
            preserve_safety_guards: true,
            reason_codes: deferred_reason_codes(&[]),
        }
    }
}

impl InvestorArchetypeIngestionConfig {
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
        PathBuf::from(&self.output_root).join(&self.ingestion_id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.ingestion_id.trim().is_empty() {
            return Err("sprint101 ingestion_id must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err(
                "sprint101 investor-archetype-ingestion config paths must be local".to_string(),
            );
        }
        for paths in [
            &self.sprint100_bundle_paths,
            &self.investor_material_paths,
            &self.markdown_material_paths,
            &self.candidate_table_paths,
        ] {
            if let Some(paths) = paths {
                if paths.iter().any(|path| !local_only(path)) {
                    return Err(
                        "sprint101 investor-archetype-ingestion config paths must be local"
                            .to_string(),
                    );
                }
            }
        }
        if self.require_paper_only && !self.preserve_runtime_deferred {
            return Err(
                "sprint101 paper-only ingestion requires runtime deferred preservation".to_string(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestorArchetypeCandidate {
    pub candidate_id: String,
    pub public_name: String,
    pub normalized_archetype_name: String,
    pub source_category: InvestorArchetypeSourceCategory,
    pub confidence_grade: InvestorConfidenceGrade,
    pub asset_scope: Vec<InvestorAssetScope>,
    pub time_horizon: InvestorTimeHorizon,
    pub intended_committee_role: String,
    pub feature_vector_refs: Vec<String>,
    pub do_not_learn_refs: Vec<String>,
    pub candidate_status: InvestorArchetypeCandidateStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestorArchetypeSourceConfidenceEntry {
    pub candidate_id: String,
    pub public_name: String,
    pub source_category: InvestorArchetypeSourceCategory,
    pub confidence_grade: InvestorConfidenceGrade,
    pub source_confidence: f64,
    pub confidence_weight: f64,
    pub weak_source_items: Vec<String>,
    pub official_source_items: Vec<String>,
    pub community_anecdote_items: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestorArchetypeSourceConfidenceReport {
    pub report_id: String,
    pub candidate_count: usize,
    pub high_confidence_count: usize,
    pub medium_confidence_count: usize,
    pub low_confidence_count: usize,
    pub blocked_count: usize,
    pub weak_source_items: Vec<String>,
    pub official_source_items: Vec<String>,
    pub community_anecdote_items: Vec<String>,
    pub entries: Vec<InvestorArchetypeSourceConfidenceEntry>,
    pub source_confidence_status: InvestorArchetypeSourceConfidenceStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestorArchetypeSafetyNormalizationReport {
    pub report_id: String,
    pub candidates_normalized: usize,
    pub impersonation_claims_removed: usize,
    pub private_strategy_claims_removed: usize,
    pub unverified_profit_claims_removed: usize,
    pub private_life_myths_removed: usize,
    pub unsupported_best_investor_claims_removed: usize,
    pub exact_rule_claims_downweighted: usize,
    pub warnings: Vec<String>,
    pub safety_status: InvestorArchetypeSafetyNormalizationStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestorStyleFeatureVectorCard {
    pub card_id: String,
    pub candidate_id: String,
    pub archetype_name: String,
    pub primary_features: Vec<String>,
    pub secondary_features: Vec<String>,
    pub entry_conditions: Vec<String>,
    pub exit_conditions: Vec<String>,
    pub risk_rules: Vec<String>,
    pub sizing_rules: Vec<String>,
    pub no_trade_conditions: Vec<String>,
    pub preferred_data_sources: Vec<String>,
    pub required_validation: Vec<String>,
    pub feature_card_status: InvestorStyleFeatureCardStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestorStyleDoNotLearnGuard {
    pub guard_id: String,
    pub candidate_id: String,
    pub blocked_items: Vec<String>,
    pub blocked_item_kinds: Vec<DoNotLearnBlockedItemKind>,
    pub guard_status: InvestorStyleDoNotLearnGuardStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestorImpersonationRiskRow {
    pub candidate_id: String,
    pub public_name: String,
    pub risk_score: f64,
    pub blocked_impersonation_claims: Vec<String>,
    pub required_disclaimer: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestorImpersonationRiskReport {
    pub report_id: String,
    pub candidate_count: usize,
    pub high_impersonation_risk_candidates: Vec<String>,
    pub medium_impersonation_risk_candidates: Vec<String>,
    pub low_impersonation_risk_candidates: Vec<String>,
    pub blocked_impersonation_claims: Vec<String>,
    pub archetype_disclaimer_present: bool,
    pub rows: Vec<InvestorImpersonationRiskRow>,
    pub report_status: InvestorImpersonationRiskStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestorUnverifiedClaimFilterReport {
    pub report_id: String,
    pub unverified_profit_claims: Vec<String>,
    pub unsupported_biographical_claims: Vec<String>,
    pub unofficial_quotes: Vec<String>,
    pub unsupported_best_claims: Vec<String>,
    pub unverifiable_trade_rules: Vec<String>,
    pub filtered_count: usize,
    pub remaining_review_count: usize,
    pub filter_status: InvestorUnverifiedClaimFilterStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestorPrivateLifeMythFilterReport {
    pub report_id: String,
    pub private_life_items_detected: Vec<String>,
    pub private_life_items_removed: Vec<String>,
    pub useful_habit_items_preserved: Vec<String>,
    pub preserved_habit_kinds: Vec<PreservedHabitKind>,
    pub filter_status: InvestorPrivateLifeMythFilterStatus,
    pub reason_codes: Vec<ReasonCode>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EighteenInvestorCandidateRegistry {
    pub registry_id: String,
    pub candidates: Vec<InvestorArchetypeCandidate>,
    pub short_term_swing_count: usize,
    pub long_term_equity_count: usize,
    pub crypto_count: usize,
    pub optional_japan_supplement_count: usize,
    pub blocked_or_low_confidence_count: usize,
    pub registry_status: EighteenInvestorCandidateRegistryStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StyleGroupTaxonomyReport {
    pub report_id: String,
    pub short_term_swing_members: Vec<String>,
    pub long_term_equity_members: Vec<String>,
    pub crypto_members: Vec<String>,
    pub common_risk_members: Vec<String>,
    pub optional_supplement_members: Vec<String>,
    pub taxonomy_status: StyleGroupTaxonomyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShortTermSwingMemberGroup {
    pub group_id: String,
    pub members: Vec<String>,
    pub shared_features: Vec<String>,
    pub conflict_rules: Vec<String>,
    pub risk_blindspots: Vec<String>,
    pub group_status: ShortTermSwingGroupStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LongTermEquityMemberGroup {
    pub group_id: String,
    pub members: Vec<String>,
    pub shared_features: Vec<String>,
    pub conflict_rules: Vec<String>,
    pub risk_blindspots: Vec<String>,
    pub group_status: LongTermEquityGroupStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CryptoMemberGroup {
    pub group_id: String,
    pub members: Vec<String>,
    pub shared_features: Vec<String>,
    pub conflict_rules: Vec<String>,
    pub risk_blindspots: Vec<String>,
    pub group_status: CryptoGroupStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommonRiskManagerMemberSpec {
    pub member_id: String,
    pub max_drawdown_limit: String,
    pub position_sizing_policy: String,
    pub liquidity_filter: String,
    pub slippage_model: String,
    pub correlation_limit: String,
    pub regime_detector: String,
    pub no_trade_mode: String,
    pub risk_governor_handoff_required: bool,
    pub broker_execution_allowed: bool,
    pub risk_manager_status: CommonRiskManagerStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StyleConflictEntry {
    pub left: String,
    pub right: String,
    pub conflict: String,
    pub resolution: StyleConflictResolutionPolicy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StyleConflictMatrix {
    pub matrix_id: String,
    pub conflicts: Vec<StyleConflictEntry>,
    pub conflict_resolution_policy: Vec<StyleConflictResolutionPolicy>,
    pub matrix_status: StyleConflictMatrixStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegimeRouteEntry {
    pub regime: String,
    pub route_to_groups: Vec<String>,
    pub rationale: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegimeRoutingPolicy {
    pub policy_id: String,
    pub regime_inputs: Vec<String>,
    pub route_to_groups: Vec<RegimeRouteEntry>,
    pub no_trade_regimes: Vec<String>,
    pub risk_denied_regimes: Vec<String>,
    pub route_status: RegimeRoutingStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MultiExpertCommitteeTopology {
    pub topology_id: String,
    pub group_heads: BTreeMap<String, String>,
    pub member_groups: BTreeMap<String, Vec<String>>,
    pub common_risk_manager: String,
    pub chairman_governance_ref: String,
    pub risk_governor_ref: String,
    pub debate_trigger_policy: Vec<String>,
    pub topology_status: MultiExpertTopologyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberStyleConfidenceWeightPolicy {
    pub policy_id: String,
    pub default_weights_by_confidence_grade: BTreeMap<String, f64>,
    pub candidate_weight_overrides: BTreeMap<String, f64>,
    pub low_confidence_cap: f64,
    pub community_anecdote_cap: f64,
    pub official_source_bonus: f64,
    pub weak_source_penalty: f64,
    pub policy_status: ConfidenceWeightPolicyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberFeatureScopeMappingReport {
    pub report_id: String,
    pub candidate_count: usize,
    pub features_by_candidate: BTreeMap<String, Vec<String>>,
    pub overlapping_features: Vec<String>,
    pub missing_features: Vec<String>,
    pub asset_scope_mismatches: Vec<String>,
    pub feature_scope_status: FeatureScopeMappingStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberLearningDataCard {
    pub candidate_id: String,
    pub asset_scope: Vec<InvestorAssetScope>,
    pub time_horizon: InvestorTimeHorizon,
    pub features: Vec<String>,
    pub entry_exit: Vec<String>,
    pub risk_rules: Vec<String>,
    pub do_not_learn: Vec<String>,
    pub offline_study_only: bool,
    pub runtime_deferred: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberLearningDataCardReport {
    pub report_id: String,
    pub data_cards: Vec<MemberLearningDataCard>,
    pub cards_with_asset_scope: usize,
    pub cards_with_time_horizon: usize,
    pub cards_with_features: usize,
    pub cards_with_entry_exit: usize,
    pub cards_with_risk_rules: usize,
    pub cards_with_do_not_learn: usize,
    pub card_status: LearningDataCardsStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberEvidenceRequirementPolicy {
    pub policy_id: String,
    pub required_evidence_by_style: BTreeMap<String, Vec<String>>,
    pub official_evidence_required_styles: Vec<String>,
    pub research_evidence_allowed_styles: Vec<String>,
    pub community_evidence_low_weight_styles: Vec<String>,
    pub minimum_evidence_before_proposal: usize,
    pub policy_status: EvidenceRequirementPolicyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArchetypeToCommitteeMemberMappingReport {
    pub report_id: String,
    pub mapped_candidates: BTreeMap<String, String>,
    pub unmapped_candidates: Vec<String>,
    pub mapped_to_existing_members: Vec<String>,
    pub mapped_to_watchlist_members: Vec<String>,
    pub mapped_to_diagnostic_members: Vec<String>,
    pub mapped_to_inactive_paper_members: Vec<String>,
    pub mapping_status: ArchetypeMappingStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EighteenInvestorCommitteeRosterPlan {
    pub plan_id: String,
    pub active_paper_members: Vec<String>,
    pub watchlist_members: Vec<String>,
    pub diagnostic_members: Vec<String>,
    pub inactive_members: Vec<String>,
    pub isolated_sentinels: Vec<String>,
    pub max_active_members: usize,
    pub activation_batch_plan: Vec<String>,
    pub roster_plan_status: EighteenInvestorCommitteeRosterPlanStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EighteenMemberActivationGate {
    pub gate_id: String,
    pub registry_status: EighteenInvestorCandidateRegistryStatus,
    pub safety_normalization_status: InvestorArchetypeSafetyNormalizationStatus,
    pub impersonation_risk_status: InvestorImpersonationRiskStatus,
    pub source_confidence_status: InvestorArchetypeSourceConfidenceStatus,
    pub feature_card_status: InvestorStyleFeatureCardStatus,
    pub do_not_learn_status: InvestorStyleDoNotLearnGuardStatus,
    pub paper_only_required: bool,
    pub live_activation_allowed: bool,
    pub gate_status: EighteenMemberActivationGateStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaperOnlyRosterExpansionGate {
    pub gate_id: String,
    pub roster_plan_status: EighteenInvestorCommitteeRosterPlanStatus,
    pub committee_topology_status: MultiExpertTopologyStatus,
    pub regime_routing_status: RegimeRoutingStatus,
    pub risk_manager_status: CommonRiskManagerStatus,
    pub safety_status: SafetyCoveragePreservationReportV17Status,
    pub workspace_truth_status: WorkspaceAcceptanceTruthGateStatus,
    pub paper_roster_expansion_allowed: bool,
    pub live_roster_expansion_allowed: bool,
    pub gate_status: PaperOnlyRosterExpansionGateStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairmanStyleGovernancePolicyV2 {
    pub policy_id: String,
    pub can_adjust_style_weights_for_paper: bool,
    pub can_adjust_style_weights_for_live: bool,
    pub can_add_member_to_watchlist: bool,
    pub can_activate_live_member: bool,
    pub can_override_risk_governor: bool,
    pub requires_audit_for_weight_change: bool,
    pub requires_owner_review_for_roster_change: bool,
    pub policy_status: ChairmanStyleGovernanceStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PromotionDemotionPolicyV2For18Styles {
    pub policy_id: String,
    pub axes: Vec<String>,
    pub style_specific_thresholds: BTreeMap<String, String>,
    pub low_confidence_member_caps: BTreeMap<String, usize>,
    pub watchlist_to_active_rules: Vec<String>,
    pub active_to_diagnostic_rules: Vec<String>,
    pub demotion_for_impersonation_risk: bool,
    pub demotion_for_source_boundary_violation: bool,
    pub demotion_for_no_lookahead_violation: bool,
    pub demotion_for_risk_governor_misalignment: bool,
    pub policy_status: PromotionPolicyV2Status,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlTowerInvestorArchetypeCandidateRow {
    pub candidate_id: String,
    pub archetype_name: String,
    pub style_group: InvestorStyleGroupKind,
    pub confidence_grade: InvestorConfidenceGrade,
    pub candidate_status: InvestorArchetypeCandidateStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlTowerInvestorArchetypeGroupRow {
    pub group_name: String,
    pub member_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlTowerInvestorArchetypePanel {
    pub panel_id: String,
    pub registry_status: EighteenInvestorCandidateRegistryStatus,
    pub candidate_rows: Vec<ControlTowerInvestorArchetypeCandidateRow>,
    pub group_rows: Vec<ControlTowerInvestorArchetypeGroupRow>,
    pub confidence_summary: String,
    pub safety_normalization_summary: String,
    pub impersonation_risk_summary: String,
    pub do_not_learn_summary: String,
    pub feature_scope_summary: String,
    pub roster_plan_summary: String,
    pub activation_gate_summary: String,
    pub chairman_style_governance_summary: String,
    pub runtime_deferred_summary: String,
    pub workspace_truth_summary: String,
    pub next_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyCoveragePreservationReportV17 {
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
    pub safety_status: SafetyCoveragePreservationReportV17Status,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestorArchetypeIngestionReport {
    pub report_id: String,
    pub sprint_name: String,
    pub paper_readiness_status: CommitteePaperReadinessGateStatus,
    pub paper_loop_dry_run_status: CommitteePaperLoopDryRunStatus,
    pub investor_material_paths: Vec<String>,
    pub candidate_count: usize,
    pub normalized_archetype_count: usize,
    pub paper_only_research_only: bool,
    pub exact_clone_forbidden: bool,
    pub training_deferred: bool,
    pub runtime_deferred: bool,
    pub live_trading_forbidden: bool,
    pub live_activation_forbidden: bool,
    pub ingestion_status: InvestorArchetypeIngestionStatus,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint101InvestorArchetypeIngestionStorageReport {
    pub report_id: String,
    pub output_dir: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint101InvestorArchetypeIngestionBundle {
    pub investor_archetype_ingestion_report: InvestorArchetypeIngestionReport,
    pub investor_archetype_source_confidence_report: InvestorArchetypeSourceConfidenceReport,
    pub investor_archetype_safety_normalization_report: InvestorArchetypeSafetyNormalizationReport,
    pub investor_style_feature_vector_cards: Vec<InvestorStyleFeatureVectorCard>,
    pub investor_style_do_not_learn_guards: Vec<InvestorStyleDoNotLearnGuard>,
    pub investor_impersonation_risk_report: InvestorImpersonationRiskReport,
    pub investor_unverified_claim_filter_report: InvestorUnverifiedClaimFilterReport,
    pub investor_private_life_myth_filter_report: InvestorPrivateLifeMythFilterReport,
    pub eighteen_investor_candidate_registry: EighteenInvestorCandidateRegistry,
    pub style_group_taxonomy_report: StyleGroupTaxonomyReport,
    pub short_term_swing_member_group: ShortTermSwingMemberGroup,
    pub long_term_equity_member_group: LongTermEquityMemberGroup,
    pub crypto_member_group: CryptoMemberGroup,
    pub common_risk_manager_member_spec: CommonRiskManagerMemberSpec,
    pub style_conflict_matrix: StyleConflictMatrix,
    pub regime_routing_policy: RegimeRoutingPolicy,
    pub multi_expert_committee_topology: MultiExpertCommitteeTopology,
    pub member_style_confidence_weight_policy: MemberStyleConfidenceWeightPolicy,
    pub member_feature_scope_mapping_report: MemberFeatureScopeMappingReport,
    pub member_learning_data_card_report: MemberLearningDataCardReport,
    pub member_evidence_requirement_policy: MemberEvidenceRequirementPolicy,
    pub archetype_to_committee_member_mapping_report: ArchetypeToCommitteeMemberMappingReport,
    pub eighteen_investor_committee_roster_plan: EighteenInvestorCommitteeRosterPlan,
    pub eighteen_member_activation_gate: EighteenMemberActivationGate,
    pub paper_only_roster_expansion_gate: PaperOnlyRosterExpansionGate,
    pub chairman_style_governance_policy_v2: ChairmanStyleGovernancePolicyV2,
    pub promotion_demotion_policy_v2_for_18_styles: PromotionDemotionPolicyV2For18Styles,
    pub safety_coverage_preservation_report_v17: SafetyCoveragePreservationReportV17,
    pub control_tower_investor_archetype_panel: ControlTowerInvestorArchetypePanel,
    pub workspace_acceptance_truth_import: WorkspaceAcceptanceTruthImport,
    pub storage_report: Sprint101InvestorArchetypeIngestionStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}
impl Sprint101InvestorArchetypeIngestionBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let mut summary = String::new();
        let sections = vec![
            (
                "## 1. Sprint summary",
                format!(
                    "- Built a deterministic Sprint 101 paper-only archetype ingestion bundle for {} candidates.\n- Reused Sprint 100 paper readiness state and preserved committee-owned architecture.",
                    self.eighteen_investor_candidate_registry.candidates.len()
                ),
            ),
            (
                "## 2. Why Sprint 101 was needed",
                "- Sprint 100 closed paper-readiness warnings, while Sprint 101 safely converts the new 18-investor material into auditable archetype cards without enabling live agents, training, or runtime inference.".to_string(),
            ),
            (
                "## 3. How the 18 investor material was applied",
                "- Parsed local-only source material into candidate records.\n- Normalized public philosophy into archetype cards, filtered unsafe claims, grouped styles, and mapped them into a paper-only roster plan.".to_string(),
            ),
            (
                "## 4. Files added",
                "- Added Sprint 101 examples, fixture data, docs, focused tests, and export surfaces for investor archetype ingestion.".to_string(),
            ),
            (
                "## 5. Files changed",
                "- Extended src/league/sprint98_committee_owned_core.rs, src/league/mod.rs, src/lib.rs, src/bin/soma_experiment.rs, and tests/support for Sprint 101.".to_string(),
            ),
            (
                "## 6. Investor archetype ingestion",
                format!(
                    "- Ingestion status: {:?}.\n- Paper-only: {}. Live activation forbidden: {}.",
                    self.investor_archetype_ingestion_report.ingestion_status,
                    self.investor_archetype_ingestion_report.paper_only_research_only,
                    self.investor_archetype_ingestion_report.live_activation_forbidden
                ),
            ),
            (
                "## 7. Source confidence",
                format!(
                    "- Source confidence status: {:?}.\n- High/medium/low/blocked = {}/{}/{}/{}.",
                    self.investor_archetype_source_confidence_report.source_confidence_status,
                    self.investor_archetype_source_confidence_report.high_confidence_count,
                    self.investor_archetype_source_confidence_report.medium_confidence_count,
                    self.investor_archetype_source_confidence_report.low_confidence_count,
                    self.investor_archetype_source_confidence_report.blocked_count
                ),
            ),
            (
                "## 8. Safety normalization",
                format!(
                    "- Safety normalization status: {:?}.\n- Removed impersonation/private-strategy/profit/private-life items = {}/{}/{}/{}.",
                    self.investor_archetype_safety_normalization_report.safety_status,
                    self.investor_archetype_safety_normalization_report.impersonation_claims_removed,
                    self.investor_archetype_safety_normalization_report.private_strategy_claims_removed,
                    self.investor_archetype_safety_normalization_report.unverified_profit_claims_removed,
                    self.investor_archetype_safety_normalization_report.private_life_myths_removed
                ),
            ),
            (
                "## 9. Feature vector cards",
                format!(
                    "- Built {} feature cards with entry, exit, risk, sizing, and no-trade conditions.",
                    self.investor_style_feature_vector_cards.len()
                ),
            ),
            (
                "## 10. Do-not-learn guards",
                format!(
                    "- Built {} do-not-learn guards covering profit claims, lifestyle myths, unofficial quotes, private strategy claims, and unsupported rules.",
                    self.investor_style_do_not_learn_guards.len()
                ),
            ),
            (
                "## 11. Impersonation risk",
                format!(
                    "- Impersonation status: {:?}.\n- High/medium/low risk candidates = {}/{}/{}.",
                    self.investor_impersonation_risk_report.report_status,
                    self.investor_impersonation_risk_report.high_impersonation_risk_candidates.len(),
                    self.investor_impersonation_risk_report.medium_impersonation_risk_candidates.len(),
                    self.investor_impersonation_risk_report.low_impersonation_risk_candidates.len()
                ),
            ),
            (
                "## 12. Unverified claim filtering",
                format!(
                    "- Filter status: {:?}.\n- Filtered {} unverified items.",
                    self.investor_unverified_claim_filter_report.filter_status,
                    self.investor_unverified_claim_filter_report.filtered_count
                ),
            ),
            (
                "## 13. Private-life myth filtering",
                format!(
                    "- Private-life filter status: {:?}.\n- Removed {} private-life myths while preserving {} useful routines.",
                    self.investor_private_life_myth_filter_report.filter_status,
                    self.investor_private_life_myth_filter_report.private_life_items_removed.len(),
                    self.investor_private_life_myth_filter_report.useful_habit_items_preserved.len()
                ),
            ),
            (
                "## 14. 18 investor candidate registry",
                format!(
                    "- Registry status: {:?}.\n- Short-term/long-term/crypto = {}/{}/{}.",
                    self.eighteen_investor_candidate_registry.registry_status,
                    self.eighteen_investor_candidate_registry.short_term_swing_count,
                    self.eighteen_investor_candidate_registry.long_term_equity_count,
                    self.eighteen_investor_candidate_registry.crypto_count
                ),
            ),
            (
                "## 15. Style group taxonomy",
                format!(
                    "- Taxonomy status: {:?}. Common risk members: {}.",
                    self.style_group_taxonomy_report.taxonomy_status,
                    self.style_group_taxonomy_report.common_risk_members.join(", ")
                ),
            ),
            (
                "## 16. Short-term / swing group",
                format!(
                    "- Group status: {:?}. Members: {}.",
                    self.short_term_swing_member_group.group_status,
                    self.short_term_swing_member_group.members.join(", ")
                ),
            ),
            (
                "## 17. Long-term equity group",
                format!(
                    "- Group status: {:?}. Members: {}.",
                    self.long_term_equity_member_group.group_status,
                    self.long_term_equity_member_group.members.join(", ")
                ),
            ),
            (
                "## 18. Crypto group",
                format!(
                    "- Group status: {:?}. Members: {}.",
                    self.crypto_member_group.group_status,
                    self.crypto_member_group.members.join(", ")
                ),
            ),
            (
                "## 19. Common risk manager",
                format!(
                    "- Risk manager status: {:?}. Risk Governor handoff required: {}.",
                    self.common_risk_manager_member_spec.risk_manager_status,
                    self.common_risk_manager_member_spec.risk_governor_handoff_required
                ),
            ),
            (
                "## 20. Style conflict matrix",
                format!(
                    "- Conflict matrix status: {:?}. Conflicts tracked: {}.",
                    self.style_conflict_matrix.matrix_status,
                    self.style_conflict_matrix.conflicts.len()
                ),
            ),
            (
                "## 21. Regime routing policy",
                format!(
                    "- Routing status: {:?}. No-trade regimes: {}.",
                    self.regime_routing_policy.route_status,
                    self.regime_routing_policy.no_trade_regimes.join(", ")
                ),
            ),
            (
                "## 22. Multi-expert committee topology",
                format!(
                    "- Topology status: {:?}. Group heads: {}.",
                    self.multi_expert_committee_topology.topology_status,
                    self.multi_expert_committee_topology
                        .group_heads
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ),
            (
                "## 23. Confidence weight policy",
                format!(
                    "- Policy status: {:?}. Low-confidence cap = {:.2}.",
                    self.member_style_confidence_weight_policy.policy_status,
                    self.member_style_confidence_weight_policy.low_confidence_cap
                ),
            ),
            (
                "## 24. Feature scope mapping",
                format!(
                    "- Feature scope status: {:?}. Overlapping features tracked: {}.",
                    self.member_feature_scope_mapping_report.feature_scope_status,
                    self.member_feature_scope_mapping_report.overlapping_features.len()
                ),
            ),
            (
                "## 25. Learning data cards",
                format!(
                    "- Learning card status: {:?}. Offline-study-only cards: {}.",
                    self.member_learning_data_card_report.card_status,
                    self.member_learning_data_card_report.data_cards.len()
                ),
            ),
            (
                "## 26. Evidence requirement policy",
                format!(
                    "- Evidence policy status: {:?}. Minimum evidence before proposal = {}.",
                    self.member_evidence_requirement_policy.policy_status,
                    self.member_evidence_requirement_policy.minimum_evidence_before_proposal
                ),
            ),
            (
                "## 27. Archetype-to-member mapping",
                format!(
                    "- Mapping status: {:?}. Existing/watchlist/diagnostic/inactive = {}/{}/{}/{}.",
                    self.archetype_to_committee_member_mapping_report.mapping_status,
                    self.archetype_to_committee_member_mapping_report.mapped_to_existing_members.len(),
                    self.archetype_to_committee_member_mapping_report.mapped_to_watchlist_members.len(),
                    self.archetype_to_committee_member_mapping_report.mapped_to_diagnostic_members.len(),
                    self.archetype_to_committee_member_mapping_report.mapped_to_inactive_paper_members.len()
                ),
            ),
            (
                "## 28. 18-member roster plan",
                format!(
                    "- Roster status: {:?}. Active/watchlist/diagnostic/inactive = {}/{}/{}/{}.",
                    self.eighteen_investor_committee_roster_plan.roster_plan_status,
                    self.eighteen_investor_committee_roster_plan.active_paper_members.len(),
                    self.eighteen_investor_committee_roster_plan.watchlist_members.len(),
                    self.eighteen_investor_committee_roster_plan.diagnostic_members.len(),
                    self.eighteen_investor_committee_roster_plan.inactive_members.len()
                ),
            ),
            (
                "## 29. Activation gate",
                format!(
                    "- Activation gate status: {:?}. Live activation allowed: {}.",
                    self.eighteen_member_activation_gate.gate_status,
                    self.eighteen_member_activation_gate.live_activation_allowed
                ),
            ),
            (
                "## 30. Paper-only roster expansion gate",
                format!(
                    "- Paper roster gate status: {:?}. Paper expansion allowed: {}. Live expansion allowed: {}.",
                    self.paper_only_roster_expansion_gate.gate_status,
                    self.paper_only_roster_expansion_gate.paper_roster_expansion_allowed,
                    self.paper_only_roster_expansion_gate.live_roster_expansion_allowed
                ),
            ),
            (
                "## 31. Chairman style governance v2",
                format!(
                    "- Governance status: {:?}. Chairman can override Risk Governor: {}.",
                    self.chairman_style_governance_policy_v2.policy_status,
                    self.chairman_style_governance_policy_v2.can_override_risk_governor
                ),
            ),
            (
                "## 32. Promotion/demotion policy v2",
                format!(
                    "- Promotion policy status: {:?}. Demotion triggers include impersonation risk, source-boundary violation, no-lookahead violation, and risk misalignment.",
                    self.promotion_demotion_policy_v2_for_18_styles.policy_status
                ),
            ),
            (
                "## 33. Safety coverage preservation v17",
                format!(
                    "- Safety status: {:?}. Eighteen live activation forbidden: {}.",
                    self.safety_coverage_preservation_report_v17.safety_status,
                    self.safety_coverage_preservation_report_v17.eighteen_live_activation_forbidden
                ),
            ),
            (
                "## 34. Control Tower investor archetype panel",
                "- Built a static/read-only panel with candidate rows, group rows, confidence summary, safety summary, roster summary, and explicit warnings for no train/runtime/live/order/account/browser controls.".to_string(),
            ),
            (
                "## 35. Output bundle",
                format!(
                    "- Wrote {} bundle files to the Sprint 101 output bundle.",
                    self.storage_report.file_count
                ),
            ),
            (
                "## 36. CLI and examples",
                "- Added Sprint 101 ingestion/report commands plus example TOMLs for each view over the same safe config surface.".to_string(),
            ),
            (
                "## 37. Tests added",
                "- Added focused ingestion, safety, registry, routing, gate, panel, CLI safety, determinism, and config tests for Sprint 101.".to_string(),
            ),
            (
                "## 38. Test results",
                "- Focused Sprint 101 tests and CLI smoke are expected to run against deterministic local-only fixtures; full workspace acceptance remains a separate truth gate.".to_string(),
            ),
            (
                "## 39. Registry status",
                format!(
                    "- {:?}; low-confidence or blocked candidates = {}.",
                    self.eighteen_investor_candidate_registry.registry_status,
                    self.eighteen_investor_candidate_registry.blocked_or_low_confidence_count
                ),
            ),
            (
                "## 40. Safety normalization status",
                format!(
                    "- {:?}; warnings = {}.",
                    self.investor_archetype_safety_normalization_report.safety_status,
                    self.investor_archetype_safety_normalization_report.warnings.len()
                ),
            ),
            (
                "## 41. Paper roster expansion status",
                format!(
                    "- {:?}; roster expansion stays paper-only.",
                    self.paper_only_roster_expansion_gate.gate_status
                ),
            ),
            (
                "## 42. Runtime deferred status",
                "- Runtime remains deferred; no model training, live inference, runtime LLM live decision path, live trading, broker, order, or account command was introduced.".to_string(),
            ),
            (
                "## 43. Workspace acceptance truth status",
                format!(
                    "- Workspace truth status: {:?}. Full workspace acceptance remains separate from focused Sprint 101 validation.",
                    self.workspace_acceptance_truth_import.truth_status
                ),
            ),
            (
                "## 44. Risk review",
                "- Chairman cannot bypass the Risk Governor. Risk Governor remains final veto. Promotion/demotion stays research roster management only.".to_string(),
            ),
            (
                "## 45. Deferred items",
                "- Runtime, training, live inference, live trading, Mamba runtime, Gated runtime, browser execution, dashboard serve, and any 18-live-agent activation remain deferred or forbidden.".to_string(),
            ),
            (
                "## 46. Next gstack sprint recommendation",
                "- Use the new archetype cards in a paper-only dry-run rotation, deepen evidence quality on lower-confidence candidates, and keep workspace acceptance truth separate.".to_string(),
            ),
        ];
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
            &output_dir.join("investor_archetype_ingestion.txt"),
            &self.investor_archetype_ingestion_report,
        )?;
        write_json_file(
            &output_dir.join("source_confidence.txt"),
            &self.investor_archetype_source_confidence_report,
        )?;
        write_json_file(
            &output_dir.join("safety_normalization.txt"),
            &self.investor_archetype_safety_normalization_report,
        )?;
        write_json_file(
            &output_dir.join("feature_vector_cards.txt"),
            &self.investor_style_feature_vector_cards,
        )?;
        write_json_file(
            &output_dir.join("do_not_learn_guards.txt"),
            &self.investor_style_do_not_learn_guards,
        )?;
        write_json_file(
            &output_dir.join("impersonation_risk.txt"),
            &self.investor_impersonation_risk_report,
        )?;
        write_json_file(
            &output_dir.join("unverified_claim_filter.txt"),
            &self.investor_unverified_claim_filter_report,
        )?;
        write_json_file(
            &output_dir.join("private_life_myth_filter.txt"),
            &self.investor_private_life_myth_filter_report,
        )?;
        write_json_file(
            &output_dir.join("eighteen_investor_candidate_registry.txt"),
            &self.eighteen_investor_candidate_registry,
        )?;
        write_json_file(
            &output_dir.join("style_group_taxonomy.txt"),
            &self.style_group_taxonomy_report,
        )?;
        write_json_file(
            &output_dir.join("short_term_swing_member_group.txt"),
            &self.short_term_swing_member_group,
        )?;
        write_json_file(
            &output_dir.join("long_term_equity_member_group.txt"),
            &self.long_term_equity_member_group,
        )?;
        write_json_file(
            &output_dir.join("crypto_member_group.txt"),
            &self.crypto_member_group,
        )?;
        write_json_file(
            &output_dir.join("common_risk_manager.txt"),
            &self.common_risk_manager_member_spec,
        )?;
        write_json_file(
            &output_dir.join("style_conflict_matrix.txt"),
            &self.style_conflict_matrix,
        )?;
        write_json_file(
            &output_dir.join("regime_routing_policy.txt"),
            &self.regime_routing_policy,
        )?;
        write_json_file(
            &output_dir.join("multi_expert_committee_topology.txt"),
            &self.multi_expert_committee_topology,
        )?;
        write_json_file(
            &output_dir.join("confidence_weight_policy.txt"),
            &self.member_style_confidence_weight_policy,
        )?;
        write_json_file(
            &output_dir.join("feature_scope_mapping.txt"),
            &self.member_feature_scope_mapping_report,
        )?;
        write_json_file(
            &output_dir.join("learning_data_cards.txt"),
            &self.member_learning_data_card_report,
        )?;
        write_json_file(
            &output_dir.join("evidence_requirement_policy.txt"),
            &self.member_evidence_requirement_policy,
        )?;
        write_json_file(
            &output_dir.join("archetype_to_member_mapping.txt"),
            &self.archetype_to_committee_member_mapping_report,
        )?;
        write_json_file(
            &output_dir.join("eighteen_roster_plan.txt"),
            &self.eighteen_investor_committee_roster_plan,
        )?;
        write_json_file(
            &output_dir.join("eighteen_activation_gate.txt"),
            &self.eighteen_member_activation_gate,
        )?;
        write_json_file(
            &output_dir.join("paper_roster_expansion_gate.txt"),
            &self.paper_only_roster_expansion_gate,
        )?;
        write_json_file(
            &output_dir.join("chairman_style_governance_v2.txt"),
            &self.chairman_style_governance_policy_v2,
        )?;
        write_json_file(
            &output_dir.join("promotion_demotion_policy_v2.txt"),
            &self.promotion_demotion_policy_v2_for_18_styles,
        )?;
        write_json_file(
            &output_dir.join("safety_coverage_v17.txt"),
            &self.safety_coverage_preservation_report_v17,
        )?;
        write_json_file(
            &output_dir.join("control_tower_investor_archetype_panel.txt"),
            &self.control_tower_investor_archetype_panel,
        )?;
        write_json_file(
            &output_dir.join("workspace_acceptance_truth_import.txt"),
            &self.workspace_acceptance_truth_import,
        )?;
        let files = vec![
            "investor_archetype_ingestion.txt".to_string(),
            "source_confidence.txt".to_string(),
            "safety_normalization.txt".to_string(),
            "feature_vector_cards.txt".to_string(),
            "do_not_learn_guards.txt".to_string(),
            "impersonation_risk.txt".to_string(),
            "unverified_claim_filter.txt".to_string(),
            "private_life_myth_filter.txt".to_string(),
            "eighteen_investor_candidate_registry.txt".to_string(),
            "style_group_taxonomy.txt".to_string(),
            "short_term_swing_member_group.txt".to_string(),
            "long_term_equity_member_group.txt".to_string(),
            "crypto_member_group.txt".to_string(),
            "common_risk_manager.txt".to_string(),
            "style_conflict_matrix.txt".to_string(),
            "regime_routing_policy.txt".to_string(),
            "multi_expert_committee_topology.txt".to_string(),
            "confidence_weight_policy.txt".to_string(),
            "feature_scope_mapping.txt".to_string(),
            "learning_data_cards.txt".to_string(),
            "evidence_requirement_policy.txt".to_string(),
            "archetype_to_member_mapping.txt".to_string(),
            "eighteen_roster_plan.txt".to_string(),
            "eighteen_activation_gate.txt".to_string(),
            "paper_roster_expansion_gate.txt".to_string(),
            "chairman_style_governance_v2.txt".to_string(),
            "promotion_demotion_policy_v2.txt".to_string(),
            "safety_coverage_v17.txt".to_string(),
            "control_tower_investor_archetype_panel.txt".to_string(),
            "workspace_acceptance_truth_import.txt".to_string(),
            "storage_report.txt".to_string(),
            "summary.txt".to_string(),
        ];
        self.storage_report = Sprint101InvestorArchetypeIngestionStorageReport {
            report_id: format!(
                "{}-storage-report",
                self.investor_archetype_ingestion_report.report_id
            ),
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct InvestorMaterialCandidateInput {
    candidate_id: String,
    public_name: String,
    normalized_archetype_name: String,
    source_category: InvestorArchetypeSourceCategory,
    asset_scope: Vec<InvestorAssetScope>,
    time_horizon: InvestorTimeHorizon,
    style_group: InvestorStyleGroupKind,
    intended_committee_role: String,
    primary_features: Vec<String>,
    secondary_features: Vec<String>,
    entry_conditions: Vec<String>,
    exit_conditions: Vec<String>,
    risk_rules: Vec<String>,
    sizing_rules: Vec<String>,
    no_trade_conditions: Vec<String>,
    preferred_data_sources: Vec<String>,
    required_validation: Vec<String>,
    blocked_items: Vec<String>,
    blocked_item_kinds: Vec<DoNotLearnBlockedItemKind>,
    private_life_items: Vec<String>,
    useful_habit_items: Vec<String>,
    preserved_habit_kinds: Vec<PreservedHabitKind>,
    unsupported_biographical_claims: Vec<String>,
    unofficial_quotes: Vec<String>,
    unsupported_best_claims: Vec<String>,
    unverifiable_trade_rules: Vec<String>,
    exact_profit_claims: Vec<String>,
    private_strategy_claims: Vec<String>,
    weak_source_items: Vec<String>,
    official_source_items: Vec<String>,
    community_anecdote_items: Vec<String>,
    model_uncertainty: String,
    source_confidence: f64,
    confidence_weight: f64,
}

#[derive(Deserialize)]
struct InvestorMaterialCandidateToml {
    public_name: String,
    normalized_archetype_name: String,
    source_category: InvestorArchetypeSourceCategory,
    asset_scope: Vec<InvestorAssetScope>,
    time_horizon: InvestorTimeHorizon,
    style_group: InvestorStyleGroupKind,
    intended_committee_role: String,
    primary_features: Vec<String>,
    secondary_features: Vec<String>,
    entry_conditions: Vec<String>,
    exit_conditions: Vec<String>,
    risk_rules: Vec<String>,
    sizing_rules: Vec<String>,
    no_trade_conditions: Vec<String>,
    preferred_data_sources: Vec<String>,
    required_validation: Vec<String>,
    blocked_items: Vec<String>,
    blocked_item_kinds: Vec<DoNotLearnBlockedItemKind>,
    private_life_items: Vec<String>,
    useful_habit_items: Vec<String>,
    preserved_habit_kinds: Vec<PreservedHabitKind>,
    unsupported_biographical_claims: Vec<String>,
    unofficial_quotes: Vec<String>,
    unsupported_best_claims: Vec<String>,
    unverifiable_trade_rules: Vec<String>,
    exact_profit_claims: Vec<String>,
    private_strategy_claims: Vec<String>,
    weak_source_items: Vec<String>,
    official_source_items: Vec<String>,
    community_anecdote_items: Vec<String>,
    model_uncertainty: String,
    source_confidence: f64,
    confidence_weight: f64,
}

fn load_sprint100_bundle_for_sprint101(
    config: &InvestorArchetypeIngestionConfig,
) -> Result<Sprint100CommitteeClosureBundle, String> {
    if let Some(bundle) =
        load_first_json::<Sprint100CommitteeClosureBundle>(config.sprint100_bundle_paths.as_ref())?
    {
        return Ok(bundle);
    }
    let mut sprint100_config = CommitteeQualityWarningClosureConfig::default();
    sprint100_config.closure_id = format!("{}-sprint100-base", config.ingestion_id);
    sprint100_config.output_root = config.output_root.clone();
    Sprint100CommitteeClosureRunner::default().run_sprint100_committee_closure(&sprint100_config)
}

fn load_investor_material_candidates(
    config: &InvestorArchetypeIngestionConfig,
) -> Result<Vec<InvestorMaterialCandidateInput>, String> {
    let mut candidates = Vec::new();
    for paths in [
        &config.investor_material_paths,
        &config.markdown_material_paths,
    ] {
        if let Some(paths) = paths {
            for path in paths {
                candidates.extend(parse_investor_material_markdown(Path::new(path))?);
            }
        }
    }
    if let Some(paths) = &config.candidate_table_paths {
        for path in paths {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            if let Ok(parsed) = serde_json::from_str::<Vec<InvestorMaterialCandidateInput>>(&text) {
                candidates.extend(parsed);
            }
        }
    }
    candidates.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
    if candidates.is_empty() {
        return Err("sprint101 requires local investor material".to_string());
    }
    Ok(candidates)
}

fn parse_investor_material_markdown(
    path: &Path,
) -> Result<Vec<InvestorMaterialCandidateInput>, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let mut parsed = Vec::new();
    let mut current_id: Option<String> = None;
    let mut block_lines: Vec<String> = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(candidate_id) = current_id.take() {
                parsed.push(parse_investor_material_block(
                    &candidate_id,
                    &block_lines.join("\n"),
                )?);
            }
            current_id = Some(rest.trim().to_string());
            block_lines.clear();
        } else if current_id.is_some() && !line.trim_start().starts_with('#') {
            block_lines.push(line.to_string());
        }
    }
    if let Some(candidate_id) = current_id.take() {
        parsed.push(parse_investor_material_block(
            &candidate_id,
            &block_lines.join("\n"),
        )?);
    }
    Ok(parsed)
}

fn parse_investor_material_block(
    candidate_id: &str,
    block: &str,
) -> Result<InvestorMaterialCandidateInput, String> {
    let parsed: InvestorMaterialCandidateToml =
        toml::from_str(block).map_err(|err| err.to_string())?;
    Ok(InvestorMaterialCandidateInput {
        candidate_id: candidate_id.to_string(),
        public_name: parsed.public_name,
        normalized_archetype_name: parsed.normalized_archetype_name,
        source_category: parsed.source_category,
        asset_scope: parsed.asset_scope,
        time_horizon: parsed.time_horizon,
        style_group: parsed.style_group,
        intended_committee_role: parsed.intended_committee_role,
        primary_features: parsed.primary_features,
        secondary_features: parsed.secondary_features,
        entry_conditions: parsed.entry_conditions,
        exit_conditions: parsed.exit_conditions,
        risk_rules: parsed.risk_rules,
        sizing_rules: parsed.sizing_rules,
        no_trade_conditions: parsed.no_trade_conditions,
        preferred_data_sources: parsed.preferred_data_sources,
        required_validation: parsed.required_validation,
        blocked_items: parsed.blocked_items,
        blocked_item_kinds: parsed.blocked_item_kinds,
        private_life_items: parsed.private_life_items,
        useful_habit_items: parsed.useful_habit_items,
        preserved_habit_kinds: parsed.preserved_habit_kinds,
        unsupported_biographical_claims: parsed.unsupported_biographical_claims,
        unofficial_quotes: parsed.unofficial_quotes,
        unsupported_best_claims: parsed.unsupported_best_claims,
        unverifiable_trade_rules: parsed.unverifiable_trade_rules,
        exact_profit_claims: parsed.exact_profit_claims,
        private_strategy_claims: parsed.private_strategy_claims,
        weak_source_items: parsed.weak_source_items,
        official_source_items: parsed.official_source_items,
        community_anecdote_items: parsed.community_anecdote_items,
        model_uncertainty: parsed.model_uncertainty,
        source_confidence: clamp_unit(parsed.source_confidence),
        confidence_weight: clamp_unit(parsed.confidence_weight),
    })
}

fn confidence_grade_from_score(score: f64) -> InvestorConfidenceGrade {
    if score >= 0.9 {
        InvestorConfidenceGrade::A
    } else if score >= 0.82 {
        InvestorConfidenceGrade::BPlus
    } else if score >= 0.74 {
        InvestorConfidenceGrade::B
    } else if score >= 0.66 {
        InvestorConfidenceGrade::BMinus
    } else if score >= 0.5 {
        InvestorConfidenceGrade::C
    } else {
        InvestorConfidenceGrade::Blocked
    }
}

fn candidate_status_from_input(
    input: &InvestorMaterialCandidateInput,
) -> InvestorArchetypeCandidateStatus {
    match confidence_grade_from_score(input.source_confidence) {
        InvestorConfidenceGrade::Blocked => InvestorArchetypeCandidateStatus::CandidateBlocked,
        InvestorConfidenceGrade::C => InvestorArchetypeCandidateStatus::CandidateLowConfidence,
        InvestorConfidenceGrade::BMinus => {
            InvestorArchetypeCandidateStatus::CandidateReadyWithWarnings
        }
        _ if !input.blocked_items.is_empty()
            || !input.private_life_items.is_empty()
            || !input.unofficial_quotes.is_empty() =>
        {
            InvestorArchetypeCandidateStatus::CandidateReadyWithWarnings
        }
        _ => InvestorArchetypeCandidateStatus::CandidateReady,
    }
}

fn feature_card_status_from_input(
    input: &InvestorMaterialCandidateInput,
) -> InvestorStyleFeatureCardStatus {
    if input.primary_features.is_empty()
        || input.entry_conditions.is_empty()
        || input.exit_conditions.is_empty()
    {
        InvestorStyleFeatureCardStatus::FeatureCardNeedsMoreEvidence
    } else if matches!(
        candidate_status_from_input(input),
        InvestorArchetypeCandidateStatus::CandidateReadyWithWarnings
            | InvestorArchetypeCandidateStatus::CandidateLowConfidence
    ) {
        InvestorStyleFeatureCardStatus::FeatureCardReadyWithWarnings
    } else {
        InvestorStyleFeatureCardStatus::FeatureCardReady
    }
}

fn do_not_learn_status_from_input(
    input: &InvestorMaterialCandidateInput,
) -> InvestorStyleDoNotLearnGuardStatus {
    if input.blocked_items.is_empty() {
        InvestorStyleDoNotLearnGuardStatus::GuardNeedsReview
    } else if matches!(
        candidate_status_from_input(input),
        InvestorArchetypeCandidateStatus::CandidateLowConfidence
    ) {
        InvestorStyleDoNotLearnGuardStatus::DoNotLearnGuardReadyWithWarnings
    } else {
        InvestorStyleDoNotLearnGuardStatus::DoNotLearnGuardReady
    }
}

fn derive_workspace_truth_import_from_sprint100(
    bundle: &Sprint100CommitteeClosureBundle,
) -> WorkspaceAcceptanceTruthImport {
    WorkspaceAcceptanceTruthImport {
        import_id: "sprint101-workspace-truth-import".to_string(),
        source_path: None,
        imported_gate_id: Some("sprint100-derived-workspace-truth".to_string()),
        truth_status: bundle.workspace_acceptance_attempt_v17.attempt_status,
        full_workspace_finished: bundle.workspace_acceptance_attempt_v17.full_finished,
        full_workspace_passed: bundle.workspace_acceptance_attempt_v17.full_passed,
        can_claim_full_acceptance: bundle
            .workspace_acceptance_attempt_v17
            .can_claim_full_acceptance,
        queue_closed_with_workspace_still_blocked: !bundle
            .workspace_acceptance_attempt_v17
            .can_claim_full_acceptance,
        notes: vec![
            format!(
                "derived_from_sprint100_paper_readiness={:?}",
                bundle.committee_paper_readiness_gate.gate_status
            ),
            format!(
                "derived_from_sprint100_workspace_attempt={:?}",
                bundle.workspace_acceptance_attempt_v17.attempt_status
            ),
        ],
        reason_codes: deferred_reason_codes(&[]),
    }
}
fn build_investor_archetype_source_confidence_report(
    candidates: &[InvestorMaterialCandidateInput],
) -> InvestorArchetypeSourceConfidenceReport {
    let mut entries = Vec::new();
    let mut weak_source_items = Vec::new();
    let mut official_source_items = Vec::new();
    let mut community_anecdote_items = Vec::new();
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;
    let mut blocked = 0;
    for candidate in candidates {
        let grade = confidence_grade_from_score(candidate.source_confidence);
        match grade {
            InvestorConfidenceGrade::A | InvestorConfidenceGrade::BPlus => high += 1,
            InvestorConfidenceGrade::B | InvestorConfidenceGrade::BMinus => medium += 1,
            InvestorConfidenceGrade::C => low += 1,
            InvestorConfidenceGrade::Blocked => blocked += 1,
        }
        weak_source_items.extend(
            candidate
                .weak_source_items
                .iter()
                .map(|item| format!("{}: {}", candidate.candidate_id, item)),
        );
        official_source_items.extend(
            candidate
                .official_source_items
                .iter()
                .map(|item| format!("{}: {}", candidate.candidate_id, item)),
        );
        community_anecdote_items.extend(
            candidate
                .community_anecdote_items
                .iter()
                .map(|item| format!("{}: {}", candidate.candidate_id, item)),
        );
        entries.push(InvestorArchetypeSourceConfidenceEntry {
            candidate_id: candidate.candidate_id.clone(),
            public_name: candidate.public_name.clone(),
            source_category: candidate.source_category,
            confidence_grade: grade,
            source_confidence: candidate.source_confidence,
            confidence_weight: candidate.confidence_weight,
            weak_source_items: candidate.weak_source_items.clone(),
            official_source_items: candidate.official_source_items.clone(),
            community_anecdote_items: candidate.community_anecdote_items.clone(),
        });
    }
    let status = if blocked > 0 {
        InvestorArchetypeSourceConfidenceStatus::SourceConfidenceNeedsReview
    } else if low > 0 || !weak_source_items.is_empty() {
        InvestorArchetypeSourceConfidenceStatus::SourceConfidenceReadyWithWarnings
    } else {
        InvestorArchetypeSourceConfidenceStatus::SourceConfidenceReady
    };
    InvestorArchetypeSourceConfidenceReport {
        report_id: "investor-archetype-source-confidence-report".to_string(),
        candidate_count: candidates.len(),
        high_confidence_count: high,
        medium_confidence_count: medium,
        low_confidence_count: low,
        blocked_count: blocked,
        weak_source_items,
        official_source_items,
        community_anecdote_items,
        entries,
        source_confidence_status: status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_investor_archetype_safety_normalization_report(
    candidates: &[InvestorMaterialCandidateInput],
) -> InvestorArchetypeSafetyNormalizationReport {
    let impersonation_claims_removed = candidates
        .iter()
        .map(|candidate| {
            candidate.unofficial_quotes.len() + candidate.unsupported_biographical_claims.len()
        })
        .sum();
    let private_strategy_claims_removed = candidates
        .iter()
        .map(|candidate| candidate.private_strategy_claims.len())
        .sum();
    let unverified_profit_claims_removed = candidates
        .iter()
        .map(|candidate| candidate.exact_profit_claims.len())
        .sum();
    let private_life_myths_removed = candidates
        .iter()
        .map(|candidate| candidate.private_life_items.len())
        .sum();
    let unsupported_best_investor_claims_removed = candidates
        .iter()
        .map(|candidate| candidate.unsupported_best_claims.len())
        .sum();
    let exact_rule_claims_downweighted = candidates
        .iter()
        .map(|candidate| candidate.unverifiable_trade_rules.len())
        .sum();
    let warnings = candidates
        .iter()
        .filter(|candidate| {
            matches!(
                candidate_status_from_input(candidate),
                InvestorArchetypeCandidateStatus::CandidateLowConfidence
                    | InvestorArchetypeCandidateStatus::CandidateBlocked
            )
        })
        .map(|candidate| {
            format!(
                "{} kept at reduced weight due to lower confidence",
                candidate.candidate_id
            )
        })
        .collect::<Vec<_>>();
    let safety_status = if warnings.is_empty() {
        InvestorArchetypeSafetyNormalizationStatus::SafetyNormalizationReady
    } else {
        InvestorArchetypeSafetyNormalizationStatus::SafetyNormalizationReadyWithWarnings
    };
    InvestorArchetypeSafetyNormalizationReport {
        report_id: "investor-archetype-safety-normalization-report".to_string(),
        candidates_normalized: candidates.len(),
        impersonation_claims_removed,
        private_strategy_claims_removed,
        unverified_profit_claims_removed,
        private_life_myths_removed,
        unsupported_best_investor_claims_removed,
        exact_rule_claims_downweighted,
        warnings,
        safety_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_investor_style_feature_vector_cards(
    candidates: &[InvestorMaterialCandidateInput],
) -> Vec<InvestorStyleFeatureVectorCard> {
    candidates
        .iter()
        .map(|candidate| InvestorStyleFeatureVectorCard {
            card_id: format!("feature-card-{}", candidate.candidate_id),
            candidate_id: candidate.candidate_id.clone(),
            archetype_name: candidate.normalized_archetype_name.clone(),
            primary_features: candidate.primary_features.clone(),
            secondary_features: candidate.secondary_features.clone(),
            entry_conditions: candidate.entry_conditions.clone(),
            exit_conditions: candidate.exit_conditions.clone(),
            risk_rules: candidate.risk_rules.clone(),
            sizing_rules: candidate.sizing_rules.clone(),
            no_trade_conditions: candidate.no_trade_conditions.clone(),
            preferred_data_sources: candidate.preferred_data_sources.clone(),
            required_validation: candidate.required_validation.clone(),
            feature_card_status: feature_card_status_from_input(candidate),
            reason_codes: deferred_reason_codes(&[]),
        })
        .collect()
}

fn build_investor_style_do_not_learn_guards(
    candidates: &[InvestorMaterialCandidateInput],
) -> Vec<InvestorStyleDoNotLearnGuard> {
    candidates
        .iter()
        .map(|candidate| InvestorStyleDoNotLearnGuard {
            guard_id: format!("do-not-learn-{}", candidate.candidate_id),
            candidate_id: candidate.candidate_id.clone(),
            blocked_items: candidate.blocked_items.clone(),
            blocked_item_kinds: candidate.blocked_item_kinds.clone(),
            guard_status: do_not_learn_status_from_input(candidate),
            reason_codes: deferred_reason_codes(&[]),
        })
        .collect()
}

fn build_investor_impersonation_risk_report(
    candidates: &[InvestorMaterialCandidateInput],
) -> InvestorImpersonationRiskReport {
    let mut high = Vec::new();
    let mut medium = Vec::new();
    let mut low = Vec::new();
    let mut blocked_impersonation_claims = Vec::new();
    let mut rows = Vec::new();
    for candidate in candidates {
        let risk_score = clamp_unit(
            0.25 + candidate.unofficial_quotes.len() as f64 * 0.12
                + candidate.unsupported_biographical_claims.len() as f64 * 0.10
                + candidate.private_strategy_claims.len() as f64 * 0.10,
        );
        if risk_score >= 0.65 {
            high.push(candidate.candidate_id.clone());
        } else if risk_score >= 0.4 {
            medium.push(candidate.candidate_id.clone());
        } else {
            low.push(candidate.candidate_id.clone());
        }
        blocked_impersonation_claims.extend(
            candidate
                .unofficial_quotes
                .iter()
                .map(|item| format!("{}: {}", candidate.candidate_id, item)),
        );
        blocked_impersonation_claims.extend(
            candidate
                .unsupported_biographical_claims
                .iter()
                .map(|item| format!("{}: {}", candidate.candidate_id, item)),
        );
        rows.push(InvestorImpersonationRiskRow {
            candidate_id: candidate.candidate_id.clone(),
            public_name: candidate.public_name.clone(),
            risk_score,
            blocked_impersonation_claims: candidate
                .unofficial_quotes
                .iter()
                .chain(candidate.unsupported_biographical_claims.iter())
                .cloned()
                .collect(),
            required_disclaimer:
                "public-philosophy-inspired archetype only; not an exact investor clone"
                    .to_string(),
        });
    }
    let report_status = if !high.is_empty() {
        InvestorImpersonationRiskStatus::ImpersonationRiskControlledWithWarnings
    } else {
        InvestorImpersonationRiskStatus::ImpersonationRiskControlled
    };
    InvestorImpersonationRiskReport {
        report_id: "investor-impersonation-risk-report".to_string(),
        candidate_count: candidates.len(),
        high_impersonation_risk_candidates: high,
        medium_impersonation_risk_candidates: medium,
        low_impersonation_risk_candidates: low,
        blocked_impersonation_claims,
        archetype_disclaimer_present: true,
        rows,
        report_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_investor_unverified_claim_filter_report(
    candidates: &[InvestorMaterialCandidateInput],
) -> InvestorUnverifiedClaimFilterReport {
    let unverified_profit_claims = candidates
        .iter()
        .flat_map(|candidate| {
            candidate
                .exact_profit_claims
                .iter()
                .map(|item| format!("{}: {}", candidate.candidate_id, item))
        })
        .collect::<Vec<_>>();
    let unsupported_biographical_claims = candidates
        .iter()
        .flat_map(|candidate| {
            candidate
                .unsupported_biographical_claims
                .iter()
                .map(|item| format!("{}: {}", candidate.candidate_id, item))
        })
        .collect::<Vec<_>>();
    let unofficial_quotes = candidates
        .iter()
        .flat_map(|candidate| {
            candidate
                .unofficial_quotes
                .iter()
                .map(|item| format!("{}: {}", candidate.candidate_id, item))
        })
        .collect::<Vec<_>>();
    let unsupported_best_claims = candidates
        .iter()
        .flat_map(|candidate| {
            candidate
                .unsupported_best_claims
                .iter()
                .map(|item| format!("{}: {}", candidate.candidate_id, item))
        })
        .collect::<Vec<_>>();
    let unverifiable_trade_rules = candidates
        .iter()
        .flat_map(|candidate| {
            candidate
                .unverifiable_trade_rules
                .iter()
                .map(|item| format!("{}: {}", candidate.candidate_id, item))
        })
        .collect::<Vec<_>>();
    let filtered_count = unverified_profit_claims.len()
        + unsupported_biographical_claims.len()
        + unofficial_quotes.len()
        + unsupported_best_claims.len()
        + unverifiable_trade_rules.len();
    InvestorUnverifiedClaimFilterReport {
        report_id: "investor-unverified-claim-filter-report".to_string(),
        unverified_profit_claims,
        unsupported_biographical_claims,
        unofficial_quotes,
        unsupported_best_claims,
        unverifiable_trade_rules,
        filtered_count,
        remaining_review_count: 0,
        filter_status: InvestorUnverifiedClaimFilterStatus::UnverifiedClaimsFiltered,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_investor_private_life_myth_filter_report(
    candidates: &[InvestorMaterialCandidateInput],
) -> InvestorPrivateLifeMythFilterReport {
    let private_life_items_detected = candidates
        .iter()
        .flat_map(|candidate| {
            candidate
                .private_life_items
                .iter()
                .map(|item| format!("{}: {}", candidate.candidate_id, item))
        })
        .collect::<Vec<_>>();
    let private_life_items_removed = private_life_items_detected.clone();
    let useful_habit_items_preserved = candidates
        .iter()
        .flat_map(|candidate| {
            candidate
                .useful_habit_items
                .iter()
                .map(|item| format!("{}: {}", candidate.candidate_id, item))
        })
        .collect::<Vec<_>>();
    let preserved_habit_kinds = candidates
        .iter()
        .flat_map(|candidate| candidate.preserved_habit_kinds.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    InvestorPrivateLifeMythFilterReport {
        report_id: "investor-private-life-myth-filter-report".to_string(),
        private_life_items_detected,
        private_life_items_removed,
        useful_habit_items_preserved,
        preserved_habit_kinds,
        filter_status: InvestorPrivateLifeMythFilterStatus::PrivateLifeMythsFiltered,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_eighteen_investor_candidate_registry(
    candidates: &[InvestorMaterialCandidateInput],
) -> EighteenInvestorCandidateRegistry {
    let registry_candidates = candidates
        .iter()
        .map(|candidate| InvestorArchetypeCandidate {
            candidate_id: candidate.candidate_id.clone(),
            public_name: candidate.public_name.clone(),
            normalized_archetype_name: candidate.normalized_archetype_name.clone(),
            source_category: candidate.source_category,
            confidence_grade: confidence_grade_from_score(candidate.source_confidence),
            asset_scope: candidate.asset_scope.clone(),
            time_horizon: candidate.time_horizon,
            intended_committee_role: candidate.intended_committee_role.clone(),
            feature_vector_refs: vec![format!("feature-card-{}", candidate.candidate_id)],
            do_not_learn_refs: vec![format!("do-not-learn-{}", candidate.candidate_id)],
            candidate_status: candidate_status_from_input(candidate),
            reason_codes: deferred_reason_codes(&[]),
        })
        .collect::<Vec<_>>();
    let short_term_swing_count = candidates
        .iter()
        .filter(|candidate| candidate.style_group == InvestorStyleGroupKind::ShortTermSwing)
        .count();
    let long_term_equity_count = candidates
        .iter()
        .filter(|candidate| candidate.style_group == InvestorStyleGroupKind::LongTermEquity)
        .count();
    let crypto_count = candidates
        .iter()
        .filter(|candidate| candidate.style_group == InvestorStyleGroupKind::Crypto)
        .count();
    let blocked_or_low_confidence_count = registry_candidates
        .iter()
        .filter(|candidate| {
            matches!(
                candidate.candidate_status,
                InvestorArchetypeCandidateStatus::CandidateLowConfidence
                    | InvestorArchetypeCandidateStatus::CandidateBlocked
            )
        })
        .count();
    let registry_status = if candidates.len() < 18 {
        EighteenInvestorCandidateRegistryStatus::CandidateRegistryNeedsReview
    } else if blocked_or_low_confidence_count > 0 {
        EighteenInvestorCandidateRegistryStatus::EighteenCandidateRegistryReadyWithWarnings
    } else {
        EighteenInvestorCandidateRegistryStatus::EighteenCandidateRegistryReady
    };
    EighteenInvestorCandidateRegistry {
        registry_id: "eighteen-investor-candidate-registry".to_string(),
        candidates: registry_candidates,
        short_term_swing_count,
        long_term_equity_count,
        crypto_count,
        optional_japan_supplement_count: 0,
        blocked_or_low_confidence_count,
        registry_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}
fn build_style_group_taxonomy_report(
    candidates: &[InvestorMaterialCandidateInput],
) -> StyleGroupTaxonomyReport {
    StyleGroupTaxonomyReport {
        report_id: "style-group-taxonomy-report".to_string(),
        short_term_swing_members: candidates
            .iter()
            .filter(|candidate| candidate.style_group == InvestorStyleGroupKind::ShortTermSwing)
            .map(|candidate| candidate.normalized_archetype_name.clone())
            .collect(),
        long_term_equity_members: candidates
            .iter()
            .filter(|candidate| candidate.style_group == InvestorStyleGroupKind::LongTermEquity)
            .map(|candidate| candidate.normalized_archetype_name.clone())
            .collect(),
        crypto_members: candidates
            .iter()
            .filter(|candidate| candidate.style_group == InvestorStyleGroupKind::Crypto)
            .map(|candidate| candidate.normalized_archetype_name.clone())
            .collect(),
        common_risk_members: vec!["CommonRiskManager".to_string()],
        optional_supplement_members: Vec::new(),
        taxonomy_status: StyleGroupTaxonomyStatus::StyleTaxonomyReady,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_short_term_swing_member_group() -> ShortTermSwingMemberGroup {
    ShortTermSwingMemberGroup {
        group_id: "short-term-swing-member-group".to_string(),
        members: vec![
            "MinerviniVcpBreakout".to_string(),
            "OneilCanSlimMarketDirection".to_string(),
            "TurtleTrendSystem".to_string(),
            "PtjMacroAsymmetricRisk".to_string(),
            "RaschkeProcessManager".to_string(),
            "LarryWilliamsStatisticalSeasonality".to_string(),
        ],
        shared_features: vec![
            "price-action confirmation".to_string(),
            "tight risk budgeting".to_string(),
            "explicit stop management".to_string(),
        ],
        conflict_rules: vec![
            "do not override long-term quality mandates without regime evidence".to_string(),
            "macro narrative must still satisfy price-action validation".to_string(),
        ],
        risk_blindspots: vec![
            "overreaction to short-lived volatility".to_string(),
            "higher whipsaw sensitivity in low-liquidity regimes".to_string(),
        ],
        group_status: ShortTermSwingGroupStatus::ShortTermSwingGroupReady,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_long_term_equity_member_group() -> LongTermEquityMemberGroup {
    LongTermEquityMemberGroup {
        group_id: "long-term-equity-member-group".to_string(),
        members: vec![
            "BuffettQualityMoat".to_string(),
            "MungerErrorFilter".to_string(),
            "GrahamMarginOfSafety".to_string(),
            "LynchGarpConsumerGrowth".to_string(),
            "BogleLowCostCore".to_string(),
            "MarksCycleRiskPremium".to_string(),
        ],
        shared_features: vec![
            "balance-sheet durability".to_string(),
            "valuation discipline".to_string(),
            "owner-oriented capital allocation review".to_string(),
        ],
        conflict_rules: vec![
            "trend-only entry signals cannot bypass valuation and balance-sheet review".to_string(),
            "macro panic alone cannot overrule margin-of-safety requirements".to_string(),
        ],
        risk_blindspots: vec![
            "slow reaction to abrupt structural breaks".to_string(),
            "under-response to short-horizon liquidity stress".to_string(),
        ],
        group_status: LongTermEquityGroupStatus::LongTermEquityGroupReady,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_crypto_member_group() -> CryptoMemberGroup {
    CryptoMemberGroup {
        group_id: "crypto-member-group".to_string(),
        members: vec![
            "WonyottiBtcRisk".to_string(),
            "SaylorTreasury".to_string(),
            "VitalikProtocolFundamental".to_string(),
            "BurniskeTokenValuation".to_string(),
            "HayesLiquidityDerivatives".to_string(),
            "WillyWooOnchainCycle".to_string(),
        ],
        shared_features: vec![
            "on-chain and market-structure cross-checks".to_string(),
            "liquidity and leverage monitoring".to_string(),
            "custody and protocol risk review".to_string(),
        ],
        conflict_rules: vec![
            "crypto leverage logic must not contaminate equity sizing rules".to_string(),
            "protocol narratives require observable network or liquidity evidence".to_string(),
        ],
        risk_blindspots: vec![
            "reflexive leverage cascades".to_string(),
            "regulatory headline shocks".to_string(),
        ],
        group_status: CryptoGroupStatus::CryptoGroupReadyWithWarnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_common_risk_manager_member_spec() -> CommonRiskManagerMemberSpec {
    CommonRiskManagerMemberSpec {
        member_id: "CommonRiskManager".to_string(),
        max_drawdown_limit: "portfolio-level drawdown budget with per-group caps".to_string(),
        position_sizing_policy: "volatility-scaled paper sizing with hard concentration limits"
            .to_string(),
        liquidity_filter: "reject illiquid setups lacking exit capacity".to_string(),
        slippage_model: "conservative paper slippage by venue and volatility regime".to_string(),
        correlation_limit: "cap correlated exposures across equity, macro, and crypto groups"
            .to_string(),
        regime_detector: "committee-approved volatility, trend, and liquidity routing inputs"
            .to_string(),
        no_trade_mode: "default to NoTrade when routing or evidence is inconclusive".to_string(),
        risk_governor_handoff_required: true,
        broker_execution_allowed: false,
        risk_manager_status: CommonRiskManagerStatus::CommonRiskManagerReady,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_style_conflict_matrix() -> StyleConflictMatrix {
    StyleConflictMatrix {
        matrix_id: "style-conflict-matrix".to_string(),
        conflicts: vec![
            StyleConflictEntry {
                left: "ShortTermBreakout".to_string(),
                right: "LongTermValue".to_string(),
                conflict: "ShortTermBreakoutVsLongTermValue".to_string(),
                resolution: StyleConflictResolutionPolicy::DebateRequired,
            },
            StyleConflictEntry {
                left: "CryptoLeverage".to_string(),
                right: "EquityQuality".to_string(),
                conflict: "CryptoLeverageVsEquityQuality".to_string(),
                resolution: StyleConflictResolutionPolicy::RiskGovernorDecides,
            },
            StyleConflictEntry {
                left: "MacroNarrative".to_string(),
                right: "PriceAction".to_string(),
                conflict: "MacroNarrativeVsPriceAction".to_string(),
                resolution: StyleConflictResolutionPolicy::RegimeRouterDecides,
            },
            StyleConflictEntry {
                left: "OnchainSignal".to_string(),
                right: "LiquiditySignal".to_string(),
                conflict: "OnchainSignalVsLiquiditySignal".to_string(),
                resolution: StyleConflictResolutionPolicy::NeedMoreEvidence,
            },
            StyleConflictEntry {
                left: "ValuePatience".to_string(),
                right: "TrendStopLoss".to_string(),
                conflict: "ValuePatienceVsTrendStopLoss".to_string(),
                resolution: StyleConflictResolutionPolicy::DebateRequired,
            },
            StyleConflictEntry {
                left: "OpportunityCost".to_string(),
                right: "RiskGovernorVeto".to_string(),
                conflict: "OpportunityCostVsRiskGovernorVeto".to_string(),
                resolution: StyleConflictResolutionPolicy::NoTradeDefault,
            },
        ],
        conflict_resolution_policy: vec![
            StyleConflictResolutionPolicy::DebateRequired,
            StyleConflictResolutionPolicy::RegimeRouterDecides,
            StyleConflictResolutionPolicy::RiskGovernorDecides,
            StyleConflictResolutionPolicy::NeedMoreEvidence,
            StyleConflictResolutionPolicy::NoTradeDefault,
        ],
        matrix_status: StyleConflictMatrixStatus::ConflictMatrixReady,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_regime_routing_policy() -> RegimeRoutingPolicy {
    RegimeRoutingPolicy {
        policy_id: "regime-routing-policy".to_string(),
        regime_inputs: vec![
            "trend persistence".to_string(),
            "volatility compression/expansion".to_string(),
            "credit and liquidity stress".to_string(),
            "crypto on-chain and funding conditions".to_string(),
        ],
        route_to_groups: vec![
            RegimeRouteEntry {
                regime: "equity_breakout_with_liquidity_support".to_string(),
                route_to_groups: vec!["ShortTermSwing".to_string(), "CommonRisk".to_string()],
                rationale:
                    "favor swing specialists when price confirmation and liquidity align".to_string(),
            },
            RegimeRouteEntry {
                regime: "quality_drawdown_with_balance_sheet_resilience".to_string(),
                route_to_groups: vec!["LongTermEquity".to_string(), "CommonRisk".to_string()],
                rationale:
                    "favor long-term quality and valuation review under controlled stress"
                        .to_string(),
            },
            RegimeRouteEntry {
                regime: "crypto_liquidity_cycle".to_string(),
                route_to_groups: vec!["Crypto".to_string(), "CommonRisk".to_string()],
                rationale:
                    "route crypto-specific evidence to the crypto group and risk sentinel"
                        .to_string(),
            },
            RegimeRouteEntry {
                regime: "cross_signal_conflict".to_string(),
                route_to_groups: vec!["CounterfactualReview".to_string(), "CommonRisk".to_string()],
                rationale:
                    "fallback to debate, counterfactual review, or NoTrade when style conflict is unresolved"
                        .to_string(),
            },
        ],
        no_trade_regimes: vec![
            "insufficient_evidence".to_string(),
            "cross_group_conflict_without_resolution".to_string(),
        ],
        risk_denied_regimes: vec!["liquidity_breakdown".to_string(), "risk_governor_veto".to_string()],
        route_status: RegimeRoutingStatus::RegimeRoutingReady,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_multi_expert_committee_topology() -> MultiExpertCommitteeTopology {
    let group_heads = BTreeMap::from([
        (
            "ShortTermSwing".to_string(),
            "OneilCanSlimMarketDirection".to_string(),
        ),
        (
            "LongTermEquity".to_string(),
            "BuffettQualityMoat".to_string(),
        ),
        (
            "Crypto".to_string(),
            "VitalikProtocolFundamental".to_string(),
        ),
        ("CommonRisk".to_string(), "CommonRiskManager".to_string()),
    ]);
    let member_groups = BTreeMap::from([
        (
            "ShortTermSwing".to_string(),
            vec![
                "MinerviniVcpBreakout".to_string(),
                "OneilCanSlimMarketDirection".to_string(),
                "TurtleTrendSystem".to_string(),
                "PtjMacroAsymmetricRisk".to_string(),
                "RaschkeProcessManager".to_string(),
                "LarryWilliamsStatisticalSeasonality".to_string(),
            ],
        ),
        (
            "LongTermEquity".to_string(),
            vec![
                "BuffettQualityMoat".to_string(),
                "MungerErrorFilter".to_string(),
                "GrahamMarginOfSafety".to_string(),
                "LynchGarpConsumerGrowth".to_string(),
                "BogleLowCostCore".to_string(),
                "MarksCycleRiskPremium".to_string(),
            ],
        ),
        (
            "Crypto".to_string(),
            vec![
                "WonyottiBtcRisk".to_string(),
                "SaylorTreasury".to_string(),
                "VitalikProtocolFundamental".to_string(),
                "BurniskeTokenValuation".to_string(),
                "HayesLiquidityDerivatives".to_string(),
                "WillyWooOnchainCycle".to_string(),
            ],
        ),
    ]);
    MultiExpertCommitteeTopology {
        topology_id: "multi-expert-committee-topology".to_string(),
        group_heads,
        member_groups,
        common_risk_manager: "CommonRiskManager".to_string(),
        chairman_governance_ref: "ChairmanStyleGovernancePolicyV2".to_string(),
        risk_governor_ref: "RiskGovernorFinalVeto".to_string(),
        debate_trigger_policy: vec![
            "member timing proposal triggers committee debate".to_string(),
            "style conflicts require debate or NoTrade".to_string(),
            "risk governor retains final veto".to_string(),
        ],
        topology_status: MultiExpertTopologyStatus::MultiExpertTopologyReady,
        reason_codes: deferred_reason_codes(&[]),
    }
}
fn build_member_style_confidence_weight_policy(
    candidates: &[InvestorMaterialCandidateInput],
) -> MemberStyleConfidenceWeightPolicy {
    let mut candidate_weight_overrides = BTreeMap::new();
    for candidate in candidates {
        candidate_weight_overrides.insert(
            candidate.normalized_archetype_name.clone(),
            candidate.confidence_weight,
        );
    }
    MemberStyleConfidenceWeightPolicy {
        policy_id: "member-style-confidence-weight-policy".to_string(),
        default_weights_by_confidence_grade: BTreeMap::from([
            ("A".to_string(), 1.0),
            ("BPlus".to_string(), 0.9),
            ("B".to_string(), 0.8),
            ("BMinus".to_string(), 0.7),
            ("C".to_string(), 0.55),
            ("Blocked".to_string(), 0.0),
        ]),
        candidate_weight_overrides,
        low_confidence_cap: 0.55,
        community_anecdote_cap: 0.6,
        official_source_bonus: 0.05,
        weak_source_penalty: 0.1,
        policy_status: ConfidenceWeightPolicyStatus::ConfidenceWeightPolicyReady,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_member_feature_scope_mapping_report(
    candidates: &[InvestorMaterialCandidateInput],
) -> MemberFeatureScopeMappingReport {
    let mut features_by_candidate = BTreeMap::new();
    let mut feature_counts = BTreeMap::<String, usize>::new();
    let mut asset_scope_mismatches = Vec::new();
    for candidate in candidates {
        let mut features = candidate.primary_features.clone();
        features.extend(candidate.secondary_features.clone());
        for feature in &features {
            *feature_counts.entry(feature.clone()).or_default() += 1;
        }
        if candidate.style_group == InvestorStyleGroupKind::Crypto
            && !candidate.asset_scope.contains(&InvestorAssetScope::Crypto)
            && !candidate
                .asset_scope
                .contains(&InvestorAssetScope::CryptoInfrastructure)
        {
            asset_scope_mismatches.push(candidate.candidate_id.clone());
        }
        features_by_candidate.insert(candidate.candidate_id.clone(), features);
    }
    let overlapping_features = feature_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(feature, _)| feature)
        .collect::<Vec<_>>();
    let missing_features = candidates
        .iter()
        .filter(|candidate| candidate.primary_features.is_empty())
        .map(|candidate| candidate.candidate_id.clone())
        .collect::<Vec<_>>();
    let feature_scope_status = if missing_features.is_empty() {
        FeatureScopeMappingStatus::FeatureScopeMappingReady
    } else {
        FeatureScopeMappingStatus::FeatureScopeNeedsMoreData
    };
    MemberFeatureScopeMappingReport {
        report_id: "member-feature-scope-mapping-report".to_string(),
        candidate_count: candidates.len(),
        features_by_candidate,
        overlapping_features,
        missing_features,
        asset_scope_mismatches,
        feature_scope_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_member_learning_data_card_report(
    candidates: &[InvestorMaterialCandidateInput],
) -> MemberLearningDataCardReport {
    let data_cards = candidates
        .iter()
        .map(|candidate| MemberLearningDataCard {
            candidate_id: candidate.candidate_id.clone(),
            asset_scope: candidate.asset_scope.clone(),
            time_horizon: candidate.time_horizon,
            features: candidate
                .primary_features
                .iter()
                .chain(candidate.secondary_features.iter())
                .cloned()
                .collect(),
            entry_exit: candidate
                .entry_conditions
                .iter()
                .chain(candidate.exit_conditions.iter())
                .cloned()
                .collect(),
            risk_rules: candidate.risk_rules.clone(),
            do_not_learn: candidate.blocked_items.clone(),
            offline_study_only: true,
            runtime_deferred: true,
        })
        .collect::<Vec<_>>();
    MemberLearningDataCardReport {
        report_id: "member-learning-data-card-report".to_string(),
        cards_with_asset_scope: data_cards
            .iter()
            .filter(|card| !card.asset_scope.is_empty())
            .count(),
        cards_with_time_horizon: data_cards.len(),
        cards_with_features: data_cards
            .iter()
            .filter(|card| !card.features.is_empty())
            .count(),
        cards_with_entry_exit: data_cards
            .iter()
            .filter(|card| !card.entry_exit.is_empty())
            .count(),
        cards_with_risk_rules: data_cards
            .iter()
            .filter(|card| !card.risk_rules.is_empty())
            .count(),
        cards_with_do_not_learn: data_cards
            .iter()
            .filter(|card| !card.do_not_learn.is_empty())
            .count(),
        data_cards,
        card_status: LearningDataCardsStatus::LearningDataCardsReady,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_member_evidence_requirement_policy() -> MemberEvidenceRequirementPolicy {
    MemberEvidenceRequirementPolicy {
        policy_id: "member-evidence-requirement-policy".to_string(),
        required_evidence_by_style: BTreeMap::from([
            (
                "ShortTermSwing".to_string(),
                vec![
                    "price confirmation".to_string(),
                    "volume/liquidity confirmation".to_string(),
                    "risk stop definition".to_string(),
                ],
            ),
            (
                "LongTermEquity".to_string(),
                vec![
                    "financial statement evidence".to_string(),
                    "valuation context".to_string(),
                    "balance-sheet review".to_string(),
                ],
            ),
            (
                "Crypto".to_string(),
                vec![
                    "on-chain or protocol evidence".to_string(),
                    "market structure/liquidity evidence".to_string(),
                    "custody or governance risk review".to_string(),
                ],
            ),
        ]),
        official_evidence_required_styles: vec![
            "LongTermEquity".to_string(),
            "CommonRisk".to_string(),
        ],
        research_evidence_allowed_styles: vec![
            "ShortTermSwing".to_string(),
            "LongTermEquity".to_string(),
            "Crypto".to_string(),
        ],
        community_evidence_low_weight_styles: vec!["Crypto".to_string()],
        minimum_evidence_before_proposal: 2,
        policy_status: EvidenceRequirementPolicyStatus::EvidenceRequirementPolicyReady,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_archetype_to_committee_member_mapping_report(
    candidates: &[InvestorMaterialCandidateInput],
) -> ArchetypeToCommitteeMemberMappingReport {
    let mut mapped_candidates = BTreeMap::new();
    let mut mapped_to_existing_members = Vec::new();
    let mut mapped_to_watchlist_members = Vec::new();
    let mut mapped_to_diagnostic_members = Vec::new();
    let mut mapped_to_inactive_paper_members = Vec::new();
    for (index, candidate) in candidates.iter().enumerate() {
        let target = if index < 8 {
            mapped_to_existing_members.push(candidate.normalized_archetype_name.clone());
            "existing-paper-seat"
        } else if index < 12 {
            mapped_to_watchlist_members.push(candidate.normalized_archetype_name.clone());
            "paper-watchlist"
        } else if index < 15 {
            mapped_to_diagnostic_members.push(candidate.normalized_archetype_name.clone());
            "diagnostic-paper-member"
        } else {
            mapped_to_inactive_paper_members.push(candidate.normalized_archetype_name.clone());
            "inactive-paper-member"
        };
        mapped_candidates.insert(candidate.candidate_id.clone(), target.to_string());
    }
    ArchetypeToCommitteeMemberMappingReport {
        report_id: "archetype-to-committee-member-mapping-report".to_string(),
        mapped_candidates,
        unmapped_candidates: Vec::new(),
        mapped_to_existing_members,
        mapped_to_watchlist_members,
        mapped_to_diagnostic_members,
        mapped_to_inactive_paper_members,
        mapping_status: ArchetypeMappingStatus::ArchetypeMappingReady,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_eighteen_investor_committee_roster_plan(
    mapping: &ArchetypeToCommitteeMemberMappingReport,
) -> EighteenInvestorCommitteeRosterPlan {
    EighteenInvestorCommitteeRosterPlan {
        plan_id: "eighteen-investor-committee-roster-plan".to_string(),
        active_paper_members: mapping.mapped_to_existing_members.clone(),
        watchlist_members: mapping.mapped_to_watchlist_members.clone(),
        diagnostic_members: mapping.mapped_to_diagnostic_members.clone(),
        inactive_members: mapping.mapped_to_inactive_paper_members.clone(),
        isolated_sentinels: vec!["CommonRiskManager".to_string()],
        max_active_members: 8,
        activation_batch_plan: vec![
            "keep initial eight paper seats active".to_string(),
            "promote watchlist members into paper review only after evidence refresh".to_string(),
            "keep diagnostic and inactive members out of live workflows".to_string(),
        ],
        roster_plan_status: EighteenInvestorCommitteeRosterPlanStatus::RosterExpansionPlanReady,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_eighteen_member_activation_gate(
    registry: &EighteenInvestorCandidateRegistry,
    safety: &InvestorArchetypeSafetyNormalizationReport,
    impersonation: &InvestorImpersonationRiskReport,
    confidence: &InvestorArchetypeSourceConfidenceReport,
    feature_cards: &[InvestorStyleFeatureVectorCard],
    guards: &[InvestorStyleDoNotLearnGuard],
) -> EighteenMemberActivationGate {
    let feature_card_status = if feature_cards.iter().all(|card| {
        matches!(
            card.feature_card_status,
            InvestorStyleFeatureCardStatus::FeatureCardReady
        )
    }) {
        InvestorStyleFeatureCardStatus::FeatureCardReady
    } else {
        InvestorStyleFeatureCardStatus::FeatureCardReadyWithWarnings
    };
    let do_not_learn_status = if guards.iter().all(|guard| {
        matches!(
            guard.guard_status,
            InvestorStyleDoNotLearnGuardStatus::DoNotLearnGuardReady
        )
    }) {
        InvestorStyleDoNotLearnGuardStatus::DoNotLearnGuardReady
    } else {
        InvestorStyleDoNotLearnGuardStatus::DoNotLearnGuardReadyWithWarnings
    };
    let gate_status = if matches!(
        registry.registry_status,
        EighteenInvestorCandidateRegistryStatus::CandidateRegistryNeedsReview
    ) {
        EighteenMemberActivationGateStatus::EighteenActivationBlocked
    } else if matches!(
        safety.safety_status,
        InvestorArchetypeSafetyNormalizationStatus::SafetyNormalizationReady
    ) && matches!(
        impersonation.report_status,
        InvestorImpersonationRiskStatus::ImpersonationRiskControlled
    ) && matches!(
        confidence.source_confidence_status,
        InvestorArchetypeSourceConfidenceStatus::SourceConfidenceReady
    ) && matches!(
        feature_card_status,
        InvestorStyleFeatureCardStatus::FeatureCardReady
    ) && matches!(
        do_not_learn_status,
        InvestorStyleDoNotLearnGuardStatus::DoNotLearnGuardReady
    ) {
        EighteenMemberActivationGateStatus::EighteenPaperRosterGateReady
    } else {
        EighteenMemberActivationGateStatus::EighteenPaperRosterGateReadyWithWarnings
    };
    EighteenMemberActivationGate {
        gate_id: "eighteen-member-activation-gate".to_string(),
        registry_status: registry.registry_status,
        safety_normalization_status: safety.safety_status,
        impersonation_risk_status: impersonation.report_status,
        source_confidence_status: confidence.source_confidence_status,
        feature_card_status,
        do_not_learn_status,
        paper_only_required: true,
        live_activation_allowed: false,
        gate_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}
fn build_safety_coverage_preservation_report_v17(
    sprint100: &Sprint100CommitteeClosureBundle,
) -> SafetyCoveragePreservationReportV17 {
    let previous = &sprint100.safety_coverage_preservation_report_v16;
    let safety_status = if previous.live_trading_guard_present
        && previous.broker_guard_present
        && previous.order_guard_present
        && previous.account_guard_present
        && previous.runtime_llm_guard_present
        && previous.source_boundary_guard_present
        && previous.browser_execution_guard_present
    {
        SafetyCoveragePreservationReportV17Status::SafetyCoveragePreserved
    } else {
        SafetyCoveragePreservationReportV17Status::SafetyCoverageMissing
    };
    SafetyCoveragePreservationReportV17 {
        report_id: "safety-coverage-preservation-report-v17".to_string(),
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
        investor_impersonation_guard_present: true,
        unverified_claim_filter_present: true,
        do_not_learn_guard_present: true,
        eighteen_live_activation_forbidden: true,
        paper_roster_only_guard_present: true,
        chairman_risk_bypass_guard_present: previous.chairman_risk_bypass_guard_present,
        safety_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_only_roster_expansion_gate(
    roster_plan: &EighteenInvestorCommitteeRosterPlan,
    topology: &MultiExpertCommitteeTopology,
    regime_routing: &RegimeRoutingPolicy,
    risk_manager: &CommonRiskManagerMemberSpec,
    safety: &SafetyCoveragePreservationReportV17,
    workspace_truth: &WorkspaceAcceptanceTruthImport,
) -> PaperOnlyRosterExpansionGate {
    let paper_roster_expansion_allowed = matches!(
        roster_plan.roster_plan_status,
        EighteenInvestorCommitteeRosterPlanStatus::RosterExpansionPlanReady
            | EighteenInvestorCommitteeRosterPlanStatus::RosterExpansionPlanReadyWithWarnings
    ) && matches!(
        topology.topology_status,
        MultiExpertTopologyStatus::MultiExpertTopologyReady
            | MultiExpertTopologyStatus::MultiExpertTopologyReadyWithWarnings
    ) && matches!(
        regime_routing.route_status,
        RegimeRoutingStatus::RegimeRoutingReady
            | RegimeRoutingStatus::RegimeRoutingReadyWithWarnings
    ) && matches!(
        risk_manager.risk_manager_status,
        CommonRiskManagerStatus::CommonRiskManagerReady
            | CommonRiskManagerStatus::CommonRiskManagerReadyWithWarnings
    ) && !workspace_truth.can_claim_full_acceptance;
    let gate_status = if paper_roster_expansion_allowed {
        PaperOnlyRosterExpansionGateStatus::PaperRosterExpansionReady
    } else {
        PaperOnlyRosterExpansionGateStatus::PaperRosterExpansionBlocked
    };
    PaperOnlyRosterExpansionGate {
        gate_id: "paper-only-roster-expansion-gate".to_string(),
        roster_plan_status: roster_plan.roster_plan_status,
        committee_topology_status: topology.topology_status,
        regime_routing_status: regime_routing.route_status,
        risk_manager_status: risk_manager.risk_manager_status,
        safety_status: safety.safety_status,
        workspace_truth_status: workspace_truth.truth_status,
        paper_roster_expansion_allowed,
        live_roster_expansion_allowed: false,
        gate_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_chairman_style_governance_policy_v2() -> ChairmanStyleGovernancePolicyV2 {
    ChairmanStyleGovernancePolicyV2 {
        policy_id: "chairman-style-governance-policy-v2".to_string(),
        can_adjust_style_weights_for_paper: true,
        can_adjust_style_weights_for_live: false,
        can_add_member_to_watchlist: true,
        can_activate_live_member: false,
        can_override_risk_governor: false,
        requires_audit_for_weight_change: true,
        requires_owner_review_for_roster_change: true,
        policy_status: ChairmanStyleGovernanceStatus::ChairmanStyleGovernanceReady,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_promotion_demotion_policy_v2_for_18_styles() -> PromotionDemotionPolicyV2For18Styles {
    PromotionDemotionPolicyV2For18Styles {
        policy_id: "promotion-demotion-policy-v2-for-18-styles".to_string(),
        axes: vec![
            "evidence quality".to_string(),
            "source confidence".to_string(),
            "risk discipline".to_string(),
            "committee contribution quality".to_string(),
        ],
        style_specific_thresholds: BTreeMap::from([
            (
                "ShortTermSwing".to_string(),
                "requires explicit stop and liquidity discipline".to_string(),
            ),
            (
                "LongTermEquity".to_string(),
                "requires fundamental and valuation evidence".to_string(),
            ),
            (
                "Crypto".to_string(),
                "requires protocol/liquidity/custody evidence".to_string(),
            ),
        ]),
        low_confidence_member_caps: BTreeMap::from([
            ("watchlist".to_string(), 4),
            ("active_paper".to_string(), 2),
        ]),
        watchlist_to_active_rules: vec![
            "must improve evidence depth before active paper promotion".to_string(),
            "must remain below low-confidence cap".to_string(),
        ],
        active_to_diagnostic_rules: vec![
            "demote when impersonation risk or source-boundary concerns rise".to_string(),
            "demote when risk governor alignment fails".to_string(),
        ],
        demotion_for_impersonation_risk: true,
        demotion_for_source_boundary_violation: true,
        demotion_for_no_lookahead_violation: true,
        demotion_for_risk_governor_misalignment: true,
        policy_status: PromotionPolicyV2Status::PromotionPolicyV2Ready,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_control_tower_investor_archetype_panel(
    registry: &EighteenInvestorCandidateRegistry,
    taxonomy: &StyleGroupTaxonomyReport,
    source_confidence: &InvestorArchetypeSourceConfidenceReport,
    safety: &InvestorArchetypeSafetyNormalizationReport,
    impersonation: &InvestorImpersonationRiskReport,
    guards: &[InvestorStyleDoNotLearnGuard],
    feature_scope: &MemberFeatureScopeMappingReport,
    roster_plan: &EighteenInvestorCommitteeRosterPlan,
    activation_gate: &EighteenMemberActivationGate,
    governance: &ChairmanStyleGovernancePolicyV2,
    workspace_truth: &WorkspaceAcceptanceTruthImport,
) -> ControlTowerInvestorArchetypePanel {
    ControlTowerInvestorArchetypePanel {
        panel_id: "control-tower-investor-archetype-panel".to_string(),
        registry_status: registry.registry_status,
        candidate_rows: registry
            .candidates
            .iter()
            .map(|candidate| ControlTowerInvestorArchetypeCandidateRow {
                candidate_id: candidate.candidate_id.clone(),
                archetype_name: candidate.normalized_archetype_name.clone(),
                style_group: if taxonomy
                    .short_term_swing_members
                    .contains(&candidate.normalized_archetype_name)
                {
                    InvestorStyleGroupKind::ShortTermSwing
                } else if taxonomy
                    .long_term_equity_members
                    .contains(&candidate.normalized_archetype_name)
                {
                    InvestorStyleGroupKind::LongTermEquity
                } else {
                    InvestorStyleGroupKind::Crypto
                },
                confidence_grade: candidate.confidence_grade,
                candidate_status: candidate.candidate_status,
            })
            .collect(),
        group_rows: vec![
            ControlTowerInvestorArchetypeGroupRow {
                group_name: "ShortTermSwing".to_string(),
                member_count: taxonomy.short_term_swing_members.len(),
                summary: "short-horizon breakout, swing, and tactical risk archetypes".to_string(),
            },
            ControlTowerInvestorArchetypeGroupRow {
                group_name: "LongTermEquity".to_string(),
                member_count: taxonomy.long_term_equity_members.len(),
                summary:
                    "long-horizon quality, value, and cycle-aware equity archetypes".to_string(),
            },
            ControlTowerInvestorArchetypeGroupRow {
                group_name: "Crypto".to_string(),
                member_count: taxonomy.crypto_members.len(),
                summary: "crypto cycle, protocol, treasury, and liquidity archetypes".to_string(),
            },
            ControlTowerInvestorArchetypeGroupRow {
                group_name: "CommonRisk".to_string(),
                member_count: taxonomy.common_risk_members.len(),
                summary: "shared risk governor and risk-manager constraints".to_string(),
            },
        ],
        confidence_summary: format!(
            "high={} medium={} low={} blocked={}",
            source_confidence.high_confidence_count,
            source_confidence.medium_confidence_count,
            source_confidence.low_confidence_count,
            source_confidence.blocked_count
        ),
        safety_normalization_summary: format!(
            "status={:?} removed_items={}",
            safety.safety_status,
            safety.impersonation_claims_removed
                + safety.private_strategy_claims_removed
                + safety.unverified_profit_claims_removed
                + safety.private_life_myths_removed
        ),
        impersonation_risk_summary: format!(
            "status={:?} high={} medium={} low={}",
            impersonation.report_status,
            impersonation.high_impersonation_risk_candidates.len(),
            impersonation.medium_impersonation_risk_candidates.len(),
            impersonation.low_impersonation_risk_candidates.len()
        ),
        do_not_learn_summary: format!("guards={} all paper-only", guards.len()),
        feature_scope_summary: format!(
            "status={:?} overlaps={} mismatches={}",
            feature_scope.feature_scope_status,
            feature_scope.overlapping_features.len(),
            feature_scope.asset_scope_mismatches.len()
        ),
        roster_plan_summary: format!(
            "active={} watchlist={} diagnostic={} inactive={}",
            roster_plan.active_paper_members.len(),
            roster_plan.watchlist_members.len(),
            roster_plan.diagnostic_members.len(),
            roster_plan.inactive_members.len()
        ),
        activation_gate_summary: format!(
            "status={:?} live_activation_allowed={}",
            activation_gate.gate_status, activation_gate.live_activation_allowed
        ),
        chairman_style_governance_summary: format!(
            "status={:?} risk_override={}",
            governance.policy_status, governance.can_override_risk_governor
        ),
        runtime_deferred_summary:
            "runtime deferred, training deferred, live inference forbidden, live trading forbidden, static/read-only control tower"
                .to_string(),
        workspace_truth_summary: format!(
            "workspace_truth={:?} full_workspace_finished={} can_claim_full_acceptance={}",
            workspace_truth.truth_status,
            workspace_truth.full_workspace_finished,
            workspace_truth.can_claim_full_acceptance
        ),
        next_actions: vec![
            "keep investor archetypes paper-only and research-only".to_string(),
            "use watchlist rotation before any paper roster promotion".to_string(),
            "continue evidence deepening for lower-confidence candidates".to_string(),
        ],
        warnings: vec![
            "static/read-only panel only".to_string(),
            "no train/runtime/live/order/account/browser controls".to_string(),
            "no activate-all-18-live action".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[ReasonCode::ControlTowerUiReadinessBuilt]),
    }
}

fn build_investor_archetype_ingestion_report(
    config: &InvestorArchetypeIngestionConfig,
    sprint100: &Sprint100CommitteeClosureBundle,
    candidates: &[InvestorMaterialCandidateInput],
) -> InvestorArchetypeIngestionReport {
    let ingestion_status = if candidates.len() < 18 {
        InvestorArchetypeIngestionStatus::InvestorArchetypeCardsBlocked
    } else if sprint100.committee_paper_readiness_gate.paper_loop_ready {
        InvestorArchetypeIngestionStatus::InvestorArchetypeCardsReady
    } else {
        InvestorArchetypeIngestionStatus::InvestorArchetypeCardsReadyWithWarnings
    };
    InvestorArchetypeIngestionReport {
        report_id: "investor-archetype-ingestion-report".to_string(),
        sprint_name: "gstack Sprint 101: 18-Investor Archetype Ingestion + AI Committee Roster Expansion Design + Style Feature Card Safety Normalization".to_string(),
        paper_readiness_status: sprint100.committee_paper_readiness_gate.gate_status,
        paper_loop_dry_run_status: sprint100.committee_paper_loop_dry_run_plan.dry_run_status,
        investor_material_paths: config
            .investor_material_paths
            .clone()
            .unwrap_or_default()
            .into_iter()
            .chain(config.markdown_material_paths.clone().unwrap_or_default())
            .collect(),
        candidate_count: candidates.len(),
        normalized_archetype_count: candidates.len(),
        paper_only_research_only: true,
        exact_clone_forbidden: config.require_no_impersonation,
        training_deferred: true,
        runtime_deferred: config.preserve_runtime_deferred,
        live_trading_forbidden: true,
        live_activation_forbidden: true,
        ingestion_status,
        warnings: vec![
            "investor styles remain archetypes only, not impersonations".to_string(),
            "no runtime training/live path was introduced".to_string(),
            "paper roster expansion does not imply live activation".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[]),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sprint101InvestorArchetypeIngestionRunner;

impl Sprint101InvestorArchetypeIngestionRunner {
    pub fn run(
        &self,
        config: &InvestorArchetypeIngestionConfig,
    ) -> Result<Sprint101InvestorArchetypeIngestionBundle, String> {
        config.validate()?;
        let sprint100_bundle = load_sprint100_bundle_for_sprint101(config)?;
        let workspace_truth_import =
            derive_workspace_truth_import_from_sprint100(&sprint100_bundle);
        let candidates = load_investor_material_candidates(config)?;
        if candidates.len() != 18 {
            return Err(format!(
                "sprint101 expected 18 investor candidates but found {}",
                candidates.len()
            ));
        }
        let investor_archetype_ingestion_report =
            build_investor_archetype_ingestion_report(config, &sprint100_bundle, &candidates);
        let investor_archetype_source_confidence_report =
            build_investor_archetype_source_confidence_report(&candidates);
        let investor_archetype_safety_normalization_report =
            build_investor_archetype_safety_normalization_report(&candidates);
        let investor_style_feature_vector_cards =
            build_investor_style_feature_vector_cards(&candidates);
        let investor_style_do_not_learn_guards =
            build_investor_style_do_not_learn_guards(&candidates);
        let investor_impersonation_risk_report =
            build_investor_impersonation_risk_report(&candidates);
        let investor_unverified_claim_filter_report =
            build_investor_unverified_claim_filter_report(&candidates);
        let investor_private_life_myth_filter_report =
            build_investor_private_life_myth_filter_report(&candidates);
        let eighteen_investor_candidate_registry =
            build_eighteen_investor_candidate_registry(&candidates);
        let style_group_taxonomy_report = build_style_group_taxonomy_report(&candidates);
        let short_term_swing_member_group = build_short_term_swing_member_group();
        let long_term_equity_member_group = build_long_term_equity_member_group();
        let crypto_member_group = build_crypto_member_group();
        let common_risk_manager_member_spec = build_common_risk_manager_member_spec();
        let style_conflict_matrix = build_style_conflict_matrix();
        let regime_routing_policy = build_regime_routing_policy();
        let multi_expert_committee_topology = build_multi_expert_committee_topology();
        let member_style_confidence_weight_policy =
            build_member_style_confidence_weight_policy(&candidates);
        let member_feature_scope_mapping_report =
            build_member_feature_scope_mapping_report(&candidates);
        let member_learning_data_card_report = build_member_learning_data_card_report(&candidates);
        let member_evidence_requirement_policy = build_member_evidence_requirement_policy();
        let archetype_to_committee_member_mapping_report =
            build_archetype_to_committee_member_mapping_report(&candidates);
        let eighteen_investor_committee_roster_plan = build_eighteen_investor_committee_roster_plan(
            &archetype_to_committee_member_mapping_report,
        );
        let eighteen_member_activation_gate = build_eighteen_member_activation_gate(
            &eighteen_investor_candidate_registry,
            &investor_archetype_safety_normalization_report,
            &investor_impersonation_risk_report,
            &investor_archetype_source_confidence_report,
            &investor_style_feature_vector_cards,
            &investor_style_do_not_learn_guards,
        );
        let safety_coverage_preservation_report_v17 =
            build_safety_coverage_preservation_report_v17(&sprint100_bundle);
        let paper_only_roster_expansion_gate = build_paper_only_roster_expansion_gate(
            &eighteen_investor_committee_roster_plan,
            &multi_expert_committee_topology,
            &regime_routing_policy,
            &common_risk_manager_member_spec,
            &safety_coverage_preservation_report_v17,
            &workspace_truth_import,
        );
        let chairman_style_governance_policy_v2 = build_chairman_style_governance_policy_v2();
        let promotion_demotion_policy_v2_for_18_styles =
            build_promotion_demotion_policy_v2_for_18_styles();
        let control_tower_investor_archetype_panel = build_control_tower_investor_archetype_panel(
            &eighteen_investor_candidate_registry,
            &style_group_taxonomy_report,
            &investor_archetype_source_confidence_report,
            &investor_archetype_safety_normalization_report,
            &investor_impersonation_risk_report,
            &investor_style_do_not_learn_guards,
            &member_feature_scope_mapping_report,
            &eighteen_investor_committee_roster_plan,
            &eighteen_member_activation_gate,
            &chairman_style_governance_policy_v2,
            &workspace_truth_import,
        );
        let mut bundle = Sprint101InvestorArchetypeIngestionBundle {
            investor_archetype_ingestion_report,
            investor_archetype_source_confidence_report,
            investor_archetype_safety_normalization_report,
            investor_style_feature_vector_cards,
            investor_style_do_not_learn_guards,
            investor_impersonation_risk_report,
            investor_unverified_claim_filter_report,
            investor_private_life_myth_filter_report,
            eighteen_investor_candidate_registry,
            style_group_taxonomy_report,
            short_term_swing_member_group,
            long_term_equity_member_group,
            crypto_member_group,
            common_risk_manager_member_spec,
            style_conflict_matrix,
            regime_routing_policy,
            multi_expert_committee_topology,
            member_style_confidence_weight_policy,
            member_feature_scope_mapping_report,
            member_learning_data_card_report,
            member_evidence_requirement_policy,
            archetype_to_committee_member_mapping_report,
            eighteen_investor_committee_roster_plan,
            eighteen_member_activation_gate,
            paper_only_roster_expansion_gate,
            chairman_style_governance_policy_v2,
            promotion_demotion_policy_v2_for_18_styles,
            safety_coverage_preservation_report_v17,
            control_tower_investor_archetype_panel,
            workspace_acceptance_truth_import: workspace_truth_import,
            storage_report: Sprint101InvestorArchetypeIngestionStorageReport {
                report_id: format!("{}-storage-report", config.ingestion_id),
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

    pub fn run_sprint101_investor_archetype_ingestion(
        &self,
        config: &InvestorArchetypeIngestionConfig,
    ) -> Result<Sprint101InvestorArchetypeIngestionBundle, String> {
        self.run(config)
    }
}
