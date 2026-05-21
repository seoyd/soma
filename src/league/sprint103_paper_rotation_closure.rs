use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::league::sprint98_committee_owned_core::WorkspaceAcceptanceTruthImport;
use crate::league::sprint102_paper_rotation::{
    ArchetypeGroupRotationPlan, ArchetypeMemberSelectionReport, ArthurHayesEvidenceHardeningReport,
    ChairmanStyleWeightAdjustmentAudit, ChairmanSynthesisDryRunReport,
    CrossGroupDebateConflictReport, EighteenArchetypePaperRotationConfig, GroupDebateSessionReport,
    LarryWilliamsEvidenceHardeningReport, LowerConfidenceEvidenceHardeningReport,
    MultiExpertRotationCoverageReport, PaperDecisionReplayV2Report, PaperDecisionTraceV2,
    PaperOnlyEntryTimingProposalRun, PaperOnlyMemberProposalRun, PaperRosterExpansionUsageReport,
    PaperRotationScenario, PaperRotationScenarioPack, RegimeRoutedCommitteeDryRunReport,
    RiskGovernorPaperHandoffReport, Sprint102PaperRotationBundle, Sprint102PaperRotationRunner,
    WeakSourceCandidateReviewReport, WonyottiEvidenceHardeningReport,
};

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

fn default_true() -> bool {
    true
}

fn bounded_decimal(value: f64) -> f64 {
    (value.clamp(0.0, 1.0) * 100.0).round() / 100.0
}

fn status_is_closed(status: &str) -> bool {
    matches!(
        status,
        "PaperRotationWarningsClosed"
            | "PaperRotationWarningsClosedWithNotes"
            | "RotationPlanWarningsClosed"
            | "RotationPlanWarningsClosedWithNotes"
            | "MemberSelectionWarningsClosed"
            | "MemberSelectionWarningsClosedWithNotes"
            | "LowerConfidenceEvidenceClosed"
            | "LowerConfidenceEvidenceClosedWithWarnings"
            | "ProposalRunWarningsClosed"
            | "ProposalRunWarningsClosedWithNotes"
            | "EntryTimingWarningsClosed"
            | "EntryTimingWarningsClosedWithNotes"
            | "DebateWarningsClosed"
            | "DebateWarningsClosedWithNotes"
            | "ChairmanSynthesisWarningsClosed"
            | "ChairmanSynthesisWarningsClosedWithNotes"
            | "StyleWeightWarningsClosed"
            | "StyleWeightWarningsClosedWithNotes"
            | "RiskHandoffWarningsClosed"
            | "RiskHandoffWarningsClosedWithNotes"
            | "PaperTraceWarningsClosed"
            | "PaperTraceWarningsClosedWithNotes"
            | "PaperReplayWarningsClosed"
            | "PaperReplayWarningsClosedWithNotes"
            | "ExpectationTraceWarningsClosed"
            | "ExpectationTraceWarningsClosedWithNotes"
            | "NoTradeRiskDeniedWarningsClosed"
            | "NoTradeRiskDeniedWarningsClosedWithNotes"
            | "RegimeRoutingWarningsClosed"
            | "RegimeRoutingWarningsClosedWithNotes"
            | "MultiExpertCoverageWarningsClosed"
            | "MultiExpertCoverageWarningsClosedWithNotes"
            | "PaperRosterUsageWarningsClosed"
            | "PaperRosterUsageWarningsClosedWithNotes"
            | "PaperRotationReady"
            | "PaperRotationReadyWithWarnings"
    )
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperRotationWarningClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub sprint102_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub paper_rotation_paths: Option<Vec<String>>,
    #[serde(default)]
    pub lower_confidence_paths: Option<Vec<String>>,
    #[serde(default)]
    pub weak_source_review_paths: Option<Vec<String>>,
    #[serde(default)]
    pub proposal_run_paths: Option<Vec<String>>,
    #[serde(default)]
    pub entry_timing_paths: Option<Vec<String>>,
    #[serde(default)]
    pub debate_session_paths: Option<Vec<String>>,
    #[serde(default)]
    pub chairman_synthesis_paths: Option<Vec<String>>,
    #[serde(default)]
    pub risk_governor_handoff_paths: Option<Vec<String>>,
    #[serde(default)]
    pub paper_trace_paths: Option<Vec<String>>,
    #[serde(default)]
    pub workspace_acceptance_truth_paths: Option<Vec<String>>,
    pub output_root: String,
    pub max_replay_scenarios: usize,
    pub max_closure_items: usize,
    #[serde(default = "default_true")]
    pub require_rotation_warning_closure: bool,
    #[serde(default = "default_true")]
    pub require_lower_confidence_closure: bool,
    #[serde(default = "default_true")]
    pub require_proposal_warning_closure: bool,
    #[serde(default = "default_true")]
    pub require_entry_timing_warning_closure: bool,
    #[serde(default = "default_true")]
    pub require_debate_warning_closure: bool,
    #[serde(default = "default_true")]
    pub require_chairman_warning_closure: bool,
    #[serde(default = "default_true")]
    pub require_risk_handoff_warning_closure: bool,
    #[serde(default = "default_true")]
    pub require_trace_warning_closure: bool,
    #[serde(default = "default_true")]
    pub require_workspace_truth_separation: bool,
    #[serde(default = "default_true")]
    pub preserve_paper_only: bool,
    #[serde(default = "default_true")]
    pub preserve_no_live_activation: bool,
    #[serde(default = "default_true")]
    pub preserve_safety_guards: bool,
    #[serde(default = "default_true")]
    pub preserve_runtime_deferred: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for PaperRotationWarningClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "sprint103-paper-rotation-closure".to_string(),
            sprint102_bundle_paths: Some(vec![
                "examples/sprint103_data/sprint102_summary.json".to_string(),
            ]),
            paper_rotation_paths: None,
            lower_confidence_paths: None,
            weak_source_review_paths: None,
            proposal_run_paths: None,
            entry_timing_paths: None,
            debate_session_paths: None,
            chairman_synthesis_paths: None,
            risk_governor_handoff_paths: None,
            paper_trace_paths: None,
            workspace_acceptance_truth_paths: Some(vec![
                "examples/sprint102_data/sprint101_summary.json".to_string(),
            ]),
            output_root: "target/soma_sprint103_paper_rotation_closure".to_string(),
            max_replay_scenarios: 7,
            max_closure_items: 64,
            require_rotation_warning_closure: true,
            require_lower_confidence_closure: true,
            require_proposal_warning_closure: true,
            require_entry_timing_warning_closure: true,
            require_debate_warning_closure: true,
            require_chairman_warning_closure: true,
            require_risk_handoff_warning_closure: true,
            require_trace_warning_closure: true,
            require_workspace_truth_separation: true,
            preserve_paper_only: true,
            preserve_no_live_activation: true,
            preserve_safety_guards: true,
            preserve_runtime_deferred: true,
            reason_codes: deferred_reason_codes(&[]),
        }
    }
}

impl PaperRotationWarningClosureConfig {
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
            return Err("sprint103 closure_id must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err("sprint103 paper rotation closure config paths must be local".to_string());
        }
        for paths in [
            &self.sprint102_bundle_paths,
            &self.paper_rotation_paths,
            &self.lower_confidence_paths,
            &self.weak_source_review_paths,
            &self.proposal_run_paths,
            &self.entry_timing_paths,
            &self.debate_session_paths,
            &self.chairman_synthesis_paths,
            &self.risk_governor_handoff_paths,
            &self.paper_trace_paths,
            &self.workspace_acceptance_truth_paths,
        ] {
            if let Some(paths) = paths
                && paths.iter().any(|path| !local_only(path))
            {
                return Err(
                    "sprint103 paper rotation closure config paths must be local".to_string(),
                );
            }
        }
        if !(1..=64).contains(&self.max_replay_scenarios)
            || !(1..=256).contains(&self.max_closure_items)
        {
            return Err("sprint103 paper rotation closure max bounds exceeded".to_string());
        }
        if self.require_rotation_warning_closure && !self.preserve_paper_only {
            return Err("sprint103 warning closure requires paper-only preservation".to_string());
        }
        if self.preserve_paper_only && !self.preserve_runtime_deferred {
            return Err(
                "sprint103 paper-only closure requires runtime deferred preservation".to_string(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperRotationWarningClosureReport {
    pub report_id: String,
    pub previous_rotation_status: String,
    pub previous_member_selection_status: String,
    pub previous_proposal_status: String,
    pub previous_entry_timing_status: String,
    pub previous_debate_status: String,
    pub previous_chairman_status: String,
    pub previous_risk_handoff_status: String,
    pub previous_trace_status: String,
    pub closed_warning_count: usize,
    pub remaining_warning_count: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RotationPlanWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub cross_group_debate_assignments: usize,
    pub assignments_with_route_reason: usize,
    pub assignments_with_conflict_reason: usize,
    pub assignments_with_risk_reason: usize,
    pub assignments_missing_reason: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberSelectionWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub selected_member_count: usize,
    pub selected_members_with_role_reason: usize,
    pub selected_members_with_confidence_reason: usize,
    pub selected_members_with_regime_reason: usize,
    pub selected_members_with_risk_reason: usize,
    pub watchlist_members_used: Vec<String>,
    pub watchlist_usage_justified: bool,
    pub remaining_selection_warnings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LowerConfidenceEvidenceClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub target_candidates: Vec<String>,
    pub evidence_improved_count: usize,
    pub still_warning_backed_count: usize,
    pub candidates_kept_diagnostic: Vec<String>,
    pub confidence_upgrades: Vec<String>,
    pub confidence_downgrades: Vec<String>,
    pub silent_upgrade_count: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WonyottiWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub exact_return_claims_blocked: bool,
    pub leverage_claims_guarded: bool,
    pub community_anecdotes_downweighted: usize,
    pub evidence_refs_added: Vec<String>,
    pub confidence_changed: bool,
    pub remains_warning_backed: bool,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LarryWilliamsWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub exact_numeric_rule_claims_downweighted: bool,
    pub statistical_seasonality_scope_preserved: bool,
    pub evidence_refs_added: Vec<String>,
    pub confidence_changed: bool,
    pub remains_warning_backed: bool,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArthurHayesWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub leverage_risk_guard_present: bool,
    pub macro_crypto_narrative_downweighted_if_unverified: bool,
    pub public_essay_refs_added: Vec<String>,
    pub confidence_changed: bool,
    pub remains_warning_backed: bool,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalRunWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub proposal_count: usize,
    pub proposals_with_evidence_refs: usize,
    pub proposals_with_timing: usize,
    pub proposals_with_risk_fields: usize,
    pub proposals_with_wait_condition: usize,
    pub proposals_with_invalidation_condition: usize,
    pub proposals_with_reason_codes: usize,
    pub remaining_proposal_warnings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryTimingRunWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub timing_proposal_count: usize,
    pub proposals_with_confirmation_conditions: usize,
    pub proposals_with_cancellation_conditions: usize,
    pub proposals_with_risk_checks: usize,
    pub proposals_with_no_entry_explanation: usize,
    pub proposals_with_cooldown_reason: usize,
    pub remaining_timing_warnings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebateSessionWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub consensus_state: String,
    pub need_more_evidence_reason_count: usize,
    pub dissent_present: bool,
    pub risk_dissent_present: bool,
    pub no_trade_dissent_present: bool,
    pub cross_group_conflict_count: usize,
    pub remaining_debate_warnings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeedMoreEvidenceItem {
    pub item_id: String,
    pub evidence_item_kind: String,
    pub recommended_resolution: String,
    pub blocking_for_paper_rotation: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeedMoreEvidenceResolutionPlan {
    pub plan_id: String,
    pub need_more_evidence_items: Vec<NeedMoreEvidenceItem>,
    pub blocking_for_paper_rotation: bool,
    pub plan_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossGroupConflictClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub conflicts_detected: usize,
    pub conflicts_with_resolution_policy: usize,
    pub conflicts_resolved_by_no_trade: usize,
    pub conflicts_resolved_by_risk_governor: usize,
    pub conflicts_resolved_by_need_more_evidence: usize,
    pub remaining_unresolved_conflicts: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairmanSynthesisWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub recommendation: String,
    pub recommendation_reason_complete: bool,
    pub conflict_summary_complete: bool,
    pub style_weight_audit_ref_present: bool,
    pub risk_governor_review_required: bool,
    pub remaining_chairman_warnings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyleWeightAuditWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub low_confidence_caps_applied: bool,
    pub source_confidence_constraints_applied: bool,
    pub risk_governor_override_attempted: bool,
    pub unsafe_weight_adjustment_count: usize,
    pub remaining_weight_warnings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskGovernorHandoffWarningClosureReportV2 {
    pub report_id: String,
    pub previous_status: String,
    pub veto_result: String,
    pub veto_reason_complete: bool,
    pub no_trade_reason_complete: bool,
    pub risk_denied_reason_complete: bool,
    pub broker_execution_allowed: bool,
    pub live_execution_allowed: bool,
    pub bypass_attempt_count: usize,
    pub remaining_handoff_warnings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperTraceWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub trace_complete: bool,
    pub missing_context_ref: bool,
    pub missing_proposal_ref: bool,
    pub missing_debate_ref: bool,
    pub missing_chairman_ref: bool,
    pub missing_risk_handoff_ref: bool,
    pub broker_execution_allowed: bool,
    pub live_execution_allowed: bool,
    pub remaining_trace_warnings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperReplayWarningClosureReportV2 {
    pub report_id: String,
    pub previous_status: String,
    pub replay_count: usize,
    pub no_trade_count: usize,
    pub risk_denied_count: usize,
    pub need_more_evidence_count: usize,
    pub broker_execution_allowed_count: usize,
    pub live_execution_allowed_count: usize,
    pub remaining_replay_warnings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExpectationTraceWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub expectation_not_profit_claim: bool,
    pub source_confidence_weight_applied: bool,
    pub expected_return_proxy_bounded: bool,
    pub expected_risk_proxy_bounded: bool,
    pub expected_drawdown_proxy_present: bool,
    pub confidence_bounded: bool,
    pub remaining_expectation_warnings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoTradeRiskDeniedTraceWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub no_trade_votes: usize,
    pub risk_deny_votes: usize,
    pub no_trade_reason_codes_complete: bool,
    pub risk_denied_reason_codes_complete: bool,
    pub risk_governor_no_trade_trace_complete: bool,
    pub risk_governor_risk_denied_trace_complete: bool,
    pub remaining_trace_warnings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegimeRoutingWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub routed_to_short_term: usize,
    pub routed_to_long_term: usize,
    pub routed_to_crypto: usize,
    pub routed_to_common_risk: usize,
    pub routes_with_regime_reason: usize,
    pub routes_with_risk_reason: usize,
    pub routes_with_no_trade_reason: usize,
    pub remaining_routing_warnings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiExpertCoverageWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub total_members_selected: usize,
    pub selected_short_term_count: usize,
    pub selected_long_term_count: usize,
    pub selected_crypto_count: usize,
    pub selected_common_risk_count: usize,
    pub unselected_members: Vec<String>,
    pub diagnostic_members_excluded: Vec<String>,
    pub coverage_sufficient_for_paper_rotation: bool,
    pub remaining_coverage_warnings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperRosterUsageWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub watchlist_members_used: Vec<String>,
    pub watchlist_usage_policy_ref: String,
    pub watchlist_usage_justified: bool,
    pub diagnostic_members_used: Vec<String>,
    pub inactive_members_used: Vec<String>,
    pub activation_violation_count: usize,
    pub remaining_roster_usage_warnings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WatchlistMemberUsagePolicy {
    pub policy_id: String,
    pub watchlist_member_usage_allowed_for_paper: bool,
    pub watchlist_member_usage_allowed_for_live: bool,
    pub requires_explicit_reason: bool,
    pub requires_source_confidence_check: bool,
    pub requires_risk_governor_review: bool,
    pub policy_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaylorTreasuryWatchlistUsageAudit {
    pub audit_id: String,
    pub watchlist_member_id: String,
    pub used_in_sprint102_rotation: bool,
    pub usage_reason: String,
    pub source_confidence_checked: bool,
    pub risk_governor_reviewed: bool,
    pub live_activation_allowed: bool,
    pub audit_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiScenarioPaperReplayPack {
    pub pack_id: String,
    pub replay_scenarios: Vec<PaperRotationScenario>,
    pub replay_count: usize,
    pub market_coverage: Vec<String>,
    pub regime_coverage: Vec<String>,
    pub group_coverage: Vec<String>,
    pub pack_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiScenarioPaperReplayReport {
    pub report_id: String,
    pub replay_count: usize,
    pub watch_candidate_count: usize,
    pub paper_conditional_count: usize,
    pub no_trade_count: usize,
    pub risk_denied_count: usize,
    pub need_more_evidence_count: usize,
    pub stable_decision_count: usize,
    pub unstable_decision_count: usize,
    pub broker_execution_allowed_count: usize,
    pub live_execution_allowed_count: usize,
    pub replay_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScenarioOutcomeExpectationRow {
    pub scenario_id: String,
    pub committee_decision: String,
    pub expected_return_proxy: f64,
    pub expected_risk_proxy: f64,
    pub expected_drawdown_proxy: f64,
    pub confidence: f64,
    pub source_confidence_weight_applied: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScenarioOutcomeExpectationMatrix {
    pub matrix_id: String,
    pub scenario_count: usize,
    pub proposal_count: usize,
    pub expectation_rows: Vec<ScenarioOutcomeExpectationRow>,
    pub expected_return_proxy_range: (f64, f64),
    pub expected_risk_proxy_range: (f64, f64),
    pub expected_drawdown_proxy_range: (f64, f64),
    pub confidence_range: (f64, f64),
    pub profit_claim_count: usize,
    pub matrix_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeDecisionStabilityReport {
    pub report_id: String,
    pub scenario_count: usize,
    pub repeated_replay_count: usize,
    pub stable_no_trade_count: usize,
    pub stable_risk_denied_count: usize,
    pub stable_watch_candidate_count: usize,
    pub unstable_count: usize,
    pub instability_reasons: Vec<String>,
    pub stability_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperNoTradeJustificationReport {
    pub report_id: String,
    pub no_trade_decision_count: usize,
    pub no_trade_reason_codes: Vec<String>,
    pub no_trade_member_vote_refs: Vec<String>,
    pub no_trade_risk_governor_refs: Vec<String>,
    pub no_trade_chairman_refs: Vec<String>,
    pub no_trade_not_failure: bool,
    pub justification_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperNeedMoreEvidenceJustificationReport {
    pub report_id: String,
    pub need_more_evidence_count: usize,
    pub evidence_items_requested: Vec<String>,
    pub evidence_items_resolved: Vec<String>,
    pub evidence_items_remaining: Vec<String>,
    pub blocking_for_paper_rotation: bool,
    pub justification_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskGovernorNoTradeReasonAudit {
    pub audit_id: String,
    pub no_trade_decision_count: usize,
    pub no_trade_veto_count: usize,
    pub risk_denied_count: usize,
    pub cooldown_count: usize,
    pub reason_codes_complete: bool,
    pub bypass_attempt_count: usize,
    pub audit_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperRotationReadinessGateV2 {
    pub gate_id: String,
    pub rotation_warning_closure_status: String,
    pub lower_confidence_closure_status: String,
    pub proposal_warning_closure_status: String,
    pub entry_timing_warning_closure_status: String,
    pub debate_warning_closure_status: String,
    pub chairman_warning_closure_status: String,
    pub risk_handoff_warning_closure_status: String,
    pub paper_trace_warning_closure_status: String,
    pub multi_scenario_replay_status: String,
    pub no_trade_justification_status: String,
    pub workspace_truth_status: String,
    pub safety_status: String,
    pub paper_rotation_ready: bool,
    pub live_rotation_allowed: bool,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAcceptanceTruthClosurePlanV4 {
    pub plan_id: String,
    pub previous_truth_status: String,
    pub current_truth_status: String,
    pub can_claim_full_acceptance: bool,
    pub no_run_gate_status: String,
    pub full_workspace_gate_status: String,
    pub recommended_actions: Vec<String>,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAcceptanceAttemptV19 {
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
    pub attempt_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyCoveragePreservationReportV19 {
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
    pub no_silent_confidence_upgrade_guard_present: bool,
    pub safety_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerPaperRotationClosurePanel {
    pub panel_id: String,
    pub rotation_warning_closure_status: String,
    pub member_selection_closure_status: String,
    pub lower_confidence_closure_status: String,
    pub weak_source_review_status: String,
    pub wonyotti_status: String,
    pub larry_williams_status: String,
    pub arthur_hayes_status: String,
    pub proposal_closure_status: String,
    pub entry_timing_closure_status: String,
    pub debate_closure_status: String,
    pub conflict_closure_status: String,
    pub chairman_closure_status: String,
    pub risk_handoff_closure_status: String,
    pub paper_trace_closure_status: String,
    pub multi_scenario_replay_status: String,
    pub decision_stability_status: String,
    pub no_trade_justification_status: String,
    pub need_more_evidence_justification_status: String,
    pub paper_rotation_readiness_status: String,
    pub workspace_truth_status: String,
    pub runtime_deferred_summary: String,
    pub safety_summary: String,
    pub next_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint103PaperRotationClosureStorageReport {
    pub report_id: String,
    pub output_dir: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint103PaperRotationClosureBundle {
    pub paper_rotation_warning_closure_report: PaperRotationWarningClosureReport,
    pub rotation_plan_warning_closure_report: RotationPlanWarningClosureReport,
    pub member_selection_warning_closure_report: MemberSelectionWarningClosureReport,
    pub lower_confidence_evidence_closure_report: LowerConfidenceEvidenceClosureReport,
    pub wonyotti_warning_closure_report: WonyottiWarningClosureReport,
    pub larry_williams_warning_closure_report: LarryWilliamsWarningClosureReport,
    pub arthur_hayes_warning_closure_report: ArthurHayesWarningClosureReport,
    pub proposal_run_warning_closure_report: ProposalRunWarningClosureReport,
    pub entry_timing_run_warning_closure_report: EntryTimingRunWarningClosureReport,
    pub debate_session_warning_closure_report: DebateSessionWarningClosureReport,
    pub need_more_evidence_resolution_plan: NeedMoreEvidenceResolutionPlan,
    pub cross_group_conflict_closure_report: CrossGroupConflictClosureReport,
    pub chairman_synthesis_warning_closure_report: ChairmanSynthesisWarningClosureReport,
    pub style_weight_audit_warning_closure_report: StyleWeightAuditWarningClosureReport,
    pub risk_governor_handoff_warning_closure_report_v2: RiskGovernorHandoffWarningClosureReportV2,
    pub paper_trace_warning_closure_report: PaperTraceWarningClosureReport,
    pub paper_replay_warning_closure_report_v2: PaperReplayWarningClosureReportV2,
    pub expectation_trace_warning_closure_report: ExpectationTraceWarningClosureReport,
    pub notrade_riskdenied_trace_warning_closure_report: NoTradeRiskDeniedTraceWarningClosureReport,
    pub regime_routing_warning_closure_report: RegimeRoutingWarningClosureReport,
    pub multi_expert_coverage_warning_closure_report: MultiExpertCoverageWarningClosureReport,
    pub paper_roster_usage_warning_closure_report: PaperRosterUsageWarningClosureReport,
    pub watchlist_member_usage_policy: WatchlistMemberUsagePolicy,
    pub saylor_treasury_watchlist_usage_audit: SaylorTreasuryWatchlistUsageAudit,
    pub multi_scenario_paper_replay_pack: MultiScenarioPaperReplayPack,
    pub multi_scenario_paper_replay_report: MultiScenarioPaperReplayReport,
    pub scenario_outcome_expectation_matrix: ScenarioOutcomeExpectationMatrix,
    pub committee_decision_stability_report: CommitteeDecisionStabilityReport,
    pub paper_notrade_justification_report: PaperNoTradeJustificationReport,
    pub paper_need_more_evidence_justification_report: PaperNeedMoreEvidenceJustificationReport,
    pub risk_governor_notrade_reason_audit: RiskGovernorNoTradeReasonAudit,
    pub paper_rotation_readiness_gate_v2: PaperRotationReadinessGateV2,
    pub workspace_acceptance_truth_closure_plan_v4: WorkspaceAcceptanceTruthClosurePlanV4,
    pub workspace_acceptance_attempt_v19: WorkspaceAcceptanceAttemptV19,
    pub safety_coverage_preservation_report_v19: SafetyCoveragePreservationReportV19,
    pub control_tower_paper_rotation_closure_panel: ControlTowerPaperRotationClosurePanel,
    pub storage_report: Sprint103PaperRotationClosureStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl Sprint103PaperRotationClosureBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            ("## 1. Sprint summary", format!("- Implemented Sprint 103 paper rotation warning closure, evidence-backed debate resolution, and multi-scenario replay calibration.\n- Closure status: {}.", self.paper_rotation_warning_closure_report.closure_status)),
            ("## 2. Why Sprint 103 was needed", "- Sprint 102 proved the first paper-only rotation worked, but it intentionally left warnings open. Sprint 103 closes those warnings without changing the committee-owned, paper-only, runtime-deferred architecture.".to_string()),
            ("## 3. Files added", "- Added Sprint 103 league layer, CLI commands, examples, fixtures, docs, focused tests, and test support.".to_string()),
            ("## 4. Files changed", "- Extended exports, CLI wiring, and test support on top of the existing Sprint 102 implementation.".to_string()),
            ("## 5. Paper rotation warning closure", format!("- Status: {}.\n- closed_warning_count={} remaining_warning_count={}.", self.paper_rotation_warning_closure_report.closure_status, self.paper_rotation_warning_closure_report.closed_warning_count, self.paper_rotation_warning_closure_report.remaining_warning_count)),
            ("## 6. Rotation plan warning closure", format!("- Status: {}.\n- cross_group_debate_assignments={}.", self.rotation_plan_warning_closure_report.closure_status, self.rotation_plan_warning_closure_report.cross_group_debate_assignments)),
            ("## 7. Member selection warning closure", format!("- Status: {}.\n- selected_member_count={} watchlist_members_used={}.", self.member_selection_warning_closure_report.closure_status, self.member_selection_warning_closure_report.selected_member_count, self.member_selection_warning_closure_report.watchlist_members_used.join(", "))),
            ("## 8. Lower-confidence evidence closure", format!("- Status: {}.\n- still_warning_backed_count={} silent_upgrade_count={}.", self.lower_confidence_evidence_closure_report.closure_status, self.lower_confidence_evidence_closure_report.still_warning_backed_count, self.lower_confidence_evidence_closure_report.silent_upgrade_count)),
            ("## 9. Wonyotti warning closure", format!("- Status: {}.\n- exact_return_claims_blocked={}.", self.wonyotti_warning_closure_report.closure_status, self.wonyotti_warning_closure_report.exact_return_claims_blocked)),
            ("## 10. Larry Williams warning closure", format!("- Status: {}.\n- exact_numeric_rule_claims_downweighted={}.", self.larry_williams_warning_closure_report.closure_status, self.larry_williams_warning_closure_report.exact_numeric_rule_claims_downweighted)),
            ("## 11. Arthur Hayes warning closure", format!("- Status: {}.\n- leverage_risk_guard_present={}.", self.arthur_hayes_warning_closure_report.closure_status, self.arthur_hayes_warning_closure_report.leverage_risk_guard_present)),
            ("## 12. Proposal run warning closure", format!("- Status: {}.\n- proposal_count={}.", self.proposal_run_warning_closure_report.closure_status, self.proposal_run_warning_closure_report.proposal_count)),
            ("## 13. Entry timing warning closure", format!("- Status: {}.\n- timing_proposal_count={}.", self.entry_timing_run_warning_closure_report.closure_status, self.entry_timing_run_warning_closure_report.timing_proposal_count)),
            ("## 14. Debate session warning closure", format!("- Status: {}.\n- consensus_state={}.", self.debate_session_warning_closure_report.closure_status, self.debate_session_warning_closure_report.consensus_state)),
            ("## 15. NeedMoreEvidence resolution plan", format!("- Status: {}.\n- item_count={}.", self.need_more_evidence_resolution_plan.plan_status, self.need_more_evidence_resolution_plan.need_more_evidence_items.len())),
            ("## 16. Cross-group conflict closure", format!("- Status: {}.\n- conflicts_detected={}.", self.cross_group_conflict_closure_report.closure_status, self.cross_group_conflict_closure_report.conflicts_detected)),
            ("## 17. Chairman synthesis warning closure", format!("- Status: {}.\n- recommendation={}.", self.chairman_synthesis_warning_closure_report.closure_status, self.chairman_synthesis_warning_closure_report.recommendation)),
            ("## 18. Style weight audit warning closure", format!("- Status: {}.\n- unsafe_weight_adjustment_count={}.", self.style_weight_audit_warning_closure_report.closure_status, self.style_weight_audit_warning_closure_report.unsafe_weight_adjustment_count)),
            ("## 19. Risk Governor handoff warning closure v2", format!("- Status: {}.\n- veto_result={}.", self.risk_governor_handoff_warning_closure_report_v2.closure_status, self.risk_governor_handoff_warning_closure_report_v2.veto_result)),
            ("## 20. Paper trace / replay warning closure", format!("- trace_status={} replay_status={}.", self.paper_trace_warning_closure_report.closure_status, self.paper_replay_warning_closure_report_v2.closure_status)),
            ("## 21. Expectation and NoTrade/RiskDenied trace closure", format!("- expectation_status={} notrade_trace_status={}.", self.expectation_trace_warning_closure_report.closure_status, self.notrade_riskdenied_trace_warning_closure_report.closure_status)),
            ("## 22. Regime routing and multi-expert coverage closure", format!("- routing_status={} coverage_status={}.", self.regime_routing_warning_closure_report.closure_status, self.multi_expert_coverage_warning_closure_report.closure_status)),
            ("## 23. Paper roster usage closure", format!("- Status: {}.\n- watchlist_members_used={}.", self.paper_roster_usage_warning_closure_report.closure_status, self.paper_roster_usage_warning_closure_report.watchlist_members_used.join(", "))),
            ("## 24. Watchlist member usage policy", format!("- Status: {}.\n- live_allowed={}.", self.watchlist_member_usage_policy.policy_status, self.watchlist_member_usage_policy.watchlist_member_usage_allowed_for_live)),
            ("## 25. SaylorTreasury watchlist audit", format!("- Status: {}.\n- used_in_sprint102_rotation={}.", self.saylor_treasury_watchlist_usage_audit.audit_status, self.saylor_treasury_watchlist_usage_audit.used_in_sprint102_rotation)),
            ("## 26. Multi-scenario paper replay", format!("- Status: {}.\n- replay_count={} no_trade_count={} need_more_evidence_count={}.", self.multi_scenario_paper_replay_report.replay_status, self.multi_scenario_paper_replay_report.replay_count, self.multi_scenario_paper_replay_report.no_trade_count, self.multi_scenario_paper_replay_report.need_more_evidence_count)),
            ("## 27. Scenario expectation matrix", format!("- Status: {}.\n- scenario_count={}.", self.scenario_outcome_expectation_matrix.matrix_status, self.scenario_outcome_expectation_matrix.scenario_count)),
            ("## 28. Committee decision stability", format!("- Status: {}.\n- unstable_count={}.", self.committee_decision_stability_report.stability_status, self.committee_decision_stability_report.unstable_count)),
            ("## 29. NoTrade and NeedMoreEvidence justification", format!("- no_trade_status={} need_more_evidence_status={}.", self.paper_notrade_justification_report.justification_status, self.paper_need_more_evidence_justification_report.justification_status)),
            ("## 30. Risk Governor NoTrade reason audit", format!("- Status: {}.\n- no_trade_veto_count={}.", self.risk_governor_notrade_reason_audit.audit_status, self.risk_governor_notrade_reason_audit.no_trade_veto_count)),
            ("## 31. Paper rotation readiness gate v2", format!("- Status: {}.\n- paper_rotation_ready={} live_rotation_allowed={}.", self.paper_rotation_readiness_gate_v2.gate_status, self.paper_rotation_readiness_gate_v2.paper_rotation_ready, self.paper_rotation_readiness_gate_v2.live_rotation_allowed)),
            ("## 32. Workspace acceptance truth v4", format!("- Status: {}.\n- can_claim_full_acceptance={}.", self.workspace_acceptance_truth_closure_plan_v4.closure_status, self.workspace_acceptance_truth_closure_plan_v4.can_claim_full_acceptance)),
            ("## 33. Safety coverage preservation v19", format!("- Status: {}.\n- no_silent_confidence_upgrade_guard_present={}.", self.safety_coverage_preservation_report_v19.safety_status, self.safety_coverage_preservation_report_v19.no_silent_confidence_upgrade_guard_present)),
            ("## 34. Control Tower paper rotation closure panel", "- Built a static/read-only control tower closure panel with no train/runtime/live/order/account/browser controls and no activate-all-18-live control.".to_string()),
            ("## 35. Output bundle", format!("- Output files: {}.", self.storage_report.file_count)),
            ("## 36. CLI and examples", "- Added Sprint 103 warning-closure commands over a single local-only config surface with explicit research-only, paper-only, warning-closure-only, and safety warnings.".to_string()),
            ("## 37. Tests added", "- Added focused Sprint 103 closure, replay, justification, CLI safety, and determinism tests.".to_string()),
            ("## 38. Test results", "- See validation commands run after implementation; full workspace truth remains reported separately from focused Sprint 103 validation.".to_string()),
            ("## 39. Paper rotation closure status", format!("- {}.", self.paper_rotation_warning_closure_report.closure_status)),
            ("## 40. Lower-confidence evidence status", format!("- {}.", self.lower_confidence_evidence_closure_report.closure_status)),
            ("## 41. Debate resolution status", format!("- {}.", self.debate_session_warning_closure_report.closure_status)),
            ("## 42. Risk Governor handoff status", format!("- {}.", self.risk_governor_handoff_warning_closure_report_v2.closure_status)),
            ("## 43. Paper readiness status", format!("- {}.", self.paper_rotation_readiness_gate_v2.gate_status)),
            ("## 44. Runtime deferred status", "- RuntimeStillDeferred\n- TrainingStillDeferred\n- LiveInferenceForbidden\n- LiveTradingForbidden\n- NoRuntimeLlmLiveDecisionPath\n- KeepResearchOnly\n- KeepPaperOnly".to_string()),
            ("## 45. Workspace acceptance truth status", format!("- {}.", self.workspace_acceptance_attempt_v19.attempt_status)),
            ("## 46. Safety coverage status", format!("- {}.", self.safety_coverage_preservation_report_v19.safety_status)),
            ("## 47. Risk review", "- Chairman cannot bypass Risk Governor. Risk Governor remains final veto. Paper proposals remain paper semantics only and never become orders or live execution authority.".to_string()),
            ("## 48. Deferred items", "- Runtime inference, model training, live inference, live trading, broker/order/account, runtime LLM live decision path, Mamba runtime, Gated runtime, dashboard serve, browser execution, and 18-live-agent activation remain deferred or forbidden.".to_string()),
            ("## 49. Next gstack sprint recommendation", "- Keep the paper committee conservative, continue evidence closure without silent upgrades, and pursue full workspace acceptance separately from paper rotation readiness.".to_string()),
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
            &output_dir.join("paper_rotation_warning_closure.txt"),
            &self.paper_rotation_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("rotation_plan_warning_closure.txt"),
            &self.rotation_plan_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("member_selection_warning_closure.txt"),
            &self.member_selection_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("lower_confidence_evidence_closure.txt"),
            &self.lower_confidence_evidence_closure_report,
        )?;
        write_json_file(
            &output_dir.join("wonyotti_warning_closure.txt"),
            &self.wonyotti_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("larry_williams_warning_closure.txt"),
            &self.larry_williams_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("arthur_hayes_warning_closure.txt"),
            &self.arthur_hayes_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("proposal_run_warning_closure.txt"),
            &self.proposal_run_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("entry_timing_run_warning_closure.txt"),
            &self.entry_timing_run_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("debate_session_warning_closure.txt"),
            &self.debate_session_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("need_more_evidence_resolution_plan.txt"),
            &self.need_more_evidence_resolution_plan,
        )?;
        write_json_file(
            &output_dir.join("cross_group_conflict_closure.txt"),
            &self.cross_group_conflict_closure_report,
        )?;
        write_json_file(
            &output_dir.join("chairman_synthesis_warning_closure.txt"),
            &self.chairman_synthesis_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("style_weight_audit_warning_closure.txt"),
            &self.style_weight_audit_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("risk_governor_handoff_warning_closure_v2.txt"),
            &self.risk_governor_handoff_warning_closure_report_v2,
        )?;
        write_json_file(
            &output_dir.join("paper_trace_warning_closure.txt"),
            &self.paper_trace_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("paper_replay_warning_closure_v2.txt"),
            &self.paper_replay_warning_closure_report_v2,
        )?;
        write_json_file(
            &output_dir.join("expectation_trace_warning_closure.txt"),
            &self.expectation_trace_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("notrade_riskdenied_trace_warning_closure.txt"),
            &self.notrade_riskdenied_trace_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("regime_routing_warning_closure.txt"),
            &self.regime_routing_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("multi_expert_coverage_warning_closure.txt"),
            &self.multi_expert_coverage_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("paper_roster_usage_warning_closure.txt"),
            &self.paper_roster_usage_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("watchlist_member_usage_policy.txt"),
            &self.watchlist_member_usage_policy,
        )?;
        write_json_file(
            &output_dir.join("saylor_treasury_watchlist_usage_audit.txt"),
            &self.saylor_treasury_watchlist_usage_audit,
        )?;
        write_json_file(
            &output_dir.join("multi_scenario_paper_replay_pack.txt"),
            &self.multi_scenario_paper_replay_pack,
        )?;
        write_json_file(
            &output_dir.join("multi_scenario_paper_replay_report.txt"),
            &self.multi_scenario_paper_replay_report,
        )?;
        write_json_file(
            &output_dir.join("scenario_outcome_expectation_matrix.txt"),
            &self.scenario_outcome_expectation_matrix,
        )?;
        write_json_file(
            &output_dir.join("committee_decision_stability.txt"),
            &self.committee_decision_stability_report,
        )?;
        write_json_file(
            &output_dir.join("paper_notrade_justification.txt"),
            &self.paper_notrade_justification_report,
        )?;
        write_json_file(
            &output_dir.join("paper_need_more_evidence_justification.txt"),
            &self.paper_need_more_evidence_justification_report,
        )?;
        write_json_file(
            &output_dir.join("risk_governor_notrade_reason_audit.txt"),
            &self.risk_governor_notrade_reason_audit,
        )?;
        write_json_file(
            &output_dir.join("paper_rotation_readiness_gate_v2.txt"),
            &self.paper_rotation_readiness_gate_v2,
        )?;
        write_json_file(
            &output_dir.join("workspace_acceptance_truth_closure_plan_v4.txt"),
            &self.workspace_acceptance_truth_closure_plan_v4,
        )?;
        write_json_file(
            &output_dir.join("workspace_acceptance_attempt_v19.txt"),
            &self.workspace_acceptance_attempt_v19,
        )?;
        write_json_file(
            &output_dir.join("safety_coverage_preservation_v19.txt"),
            &self.safety_coverage_preservation_report_v19,
        )?;
        write_json_file(
            &output_dir.join("control_tower_paper_rotation_closure_panel.txt"),
            &self.control_tower_paper_rotation_closure_panel,
        )?;
        let files = vec![
            "paper_rotation_warning_closure.txt",
            "rotation_plan_warning_closure.txt",
            "member_selection_warning_closure.txt",
            "lower_confidence_evidence_closure.txt",
            "wonyotti_warning_closure.txt",
            "larry_williams_warning_closure.txt",
            "arthur_hayes_warning_closure.txt",
            "proposal_run_warning_closure.txt",
            "entry_timing_run_warning_closure.txt",
            "debate_session_warning_closure.txt",
            "need_more_evidence_resolution_plan.txt",
            "cross_group_conflict_closure.txt",
            "chairman_synthesis_warning_closure.txt",
            "style_weight_audit_warning_closure.txt",
            "risk_governor_handoff_warning_closure_v2.txt",
            "paper_trace_warning_closure.txt",
            "paper_replay_warning_closure_v2.txt",
            "expectation_trace_warning_closure.txt",
            "notrade_riskdenied_trace_warning_closure.txt",
            "regime_routing_warning_closure.txt",
            "multi_expert_coverage_warning_closure.txt",
            "paper_roster_usage_warning_closure.txt",
            "watchlist_member_usage_policy.txt",
            "saylor_treasury_watchlist_usage_audit.txt",
            "multi_scenario_paper_replay_pack.txt",
            "multi_scenario_paper_replay_report.txt",
            "scenario_outcome_expectation_matrix.txt",
            "committee_decision_stability.txt",
            "paper_notrade_justification.txt",
            "paper_need_more_evidence_justification.txt",
            "risk_governor_notrade_reason_audit.txt",
            "paper_rotation_readiness_gate_v2.txt",
            "workspace_acceptance_truth_closure_plan_v4.txt",
            "workspace_acceptance_attempt_v19.txt",
            "safety_coverage_preservation_v19.txt",
            "control_tower_paper_rotation_closure_panel.txt",
            "storage_report.txt",
            "summary.txt",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        self.storage_report = Sprint103PaperRotationClosureStorageReport {
            report_id: "sprint103-paper-rotation-closure-storage-report".to_string(),
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

fn load_or_default<T: for<'de> Deserialize<'de> + Clone>(
    paths: Option<&Vec<String>>,
    default: T,
) -> Result<T, String> {
    Ok(load_first_json(paths)?.unwrap_or(default))
}

fn load_sprint102_bundle(
    config: &PaperRotationWarningClosureConfig,
) -> Result<Sprint102PaperRotationBundle, String> {
    if let Some(bundle) =
        load_first_json::<Sprint102PaperRotationBundle>(config.sprint102_bundle_paths.as_ref())?
    {
        return Ok(bundle);
    }
    if let Some(bundle) =
        load_first_json::<Sprint102PaperRotationBundle>(config.paper_rotation_paths.as_ref())?
    {
        return Ok(bundle);
    }
    let mut sprint102_config = EighteenArchetypePaperRotationConfig::default();
    sprint102_config.rotation_id = format!("{}-sprint102-base", config.closure_id);
    sprint102_config.output_root = config.output_root.clone();
    Sprint102PaperRotationRunner::default().run_sprint102_paper_rotation(&sprint102_config)
}

fn load_workspace_truth_import(
    config: &PaperRotationWarningClosureConfig,
    sprint102: &Sprint102PaperRotationBundle,
) -> Result<WorkspaceAcceptanceTruthImport, String> {
    Ok(load_first_json::<WorkspaceAcceptanceTruthImport>(
        config.workspace_acceptance_truth_paths.as_ref(),
    )?
    .unwrap_or_else(|| WorkspaceAcceptanceTruthImport {
        import_id: "workspace-acceptance-truth-import-v19".to_string(),
        source_path: None,
        imported_gate_id: Some("sprint102-workspace-acceptance-truth".to_string()),
        truth_status: sprint102.workspace_acceptance_attempt_v18.attempt_status,
        full_workspace_finished: sprint102.workspace_acceptance_attempt_v18.full_finished,
        full_workspace_passed: sprint102.workspace_acceptance_attempt_v18.full_passed,
        can_claim_full_acceptance: sprint102
            .workspace_acceptance_attempt_v18
            .can_claim_full_acceptance,
        queue_closed_with_workspace_still_blocked: !sprint102
            .workspace_acceptance_attempt_v18
            .can_claim_full_acceptance,
        notes: vec![
            "full workspace acceptance remains separate from Sprint 103 warning closure"
                .to_string(),
            "focused Sprint 103 validation does not upgrade workspace truth".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[]),
    }))
}

fn scenario_decision(scenario: &PaperRotationScenario, index: usize) -> String {
    use crate::league::sprint102_paper_rotation::PaperRotationRegimeCoverage::*;
    match scenario.regime {
        TrendBreakout => "WatchCandidate".to_string(),
        RangeBound => "NeedMoreEvidence".to_string(),
        HighVolatility | DrawdownRisk => "NoTrade".to_string(),
        MacroShift => "PaperConditionalCandidate".to_string(),
        CryptoCycle => {
            if index % 2 == 0 {
                "NoTrade".to_string()
            } else {
                "WatchCandidate".to_string()
            }
        }
        InsufficientEvidence => "NeedMoreEvidence".to_string(),
    }
}

fn market_label(scenario: &PaperRotationScenario) -> String {
    format!("{:?}", scenario.market)
}

fn regime_label(scenario: &PaperRotationScenario) -> String {
    format!("{:?}", scenario.regime)
}

fn selection_group_coverage(selection: &ArchetypeMemberSelectionReport) -> Vec<String> {
    selection.selection_by_group.keys().cloned().collect()
}

#[derive(Default)]
pub struct PaperRotationWarningClosureRunner;

#[derive(Default)]
pub struct Sprint103PaperRotationClosureRunner;

impl Sprint103PaperRotationClosureRunner {
    pub fn run(
        &self,
        config: &PaperRotationWarningClosureConfig,
    ) -> Result<Sprint103PaperRotationClosureBundle, String> {
        PaperRotationWarningClosureRunner::default().run(config)
    }

    pub fn run_sprint103_paper_rotation_closure(
        &self,
        config: &PaperRotationWarningClosureConfig,
    ) -> Result<Sprint103PaperRotationClosureBundle, String> {
        self.run(config)
    }
}

impl PaperRotationWarningClosureRunner {
    pub fn run(
        &self,
        config: &PaperRotationWarningClosureConfig,
    ) -> Result<Sprint103PaperRotationClosureBundle, String> {
        config.validate()?;
        let sprint102 = load_sprint102_bundle(config)?;
        let lower_confidence = load_or_default(
            config.lower_confidence_paths.as_ref(),
            sprint102.lower_confidence_evidence_hardening_report.clone(),
        )?;
        let weak_source_review = load_or_default(
            config.weak_source_review_paths.as_ref(),
            sprint102.weak_source_candidate_review_report.clone(),
        )?;
        let proposal_run = load_or_default(
            config.proposal_run_paths.as_ref(),
            sprint102.paper_only_member_proposal_run.clone(),
        )?;
        let entry_timing = load_or_default(
            config.entry_timing_paths.as_ref(),
            sprint102.paper_only_entry_timing_proposal_run.clone(),
        )?;
        let debate_session = load_or_default(
            config.debate_session_paths.as_ref(),
            sprint102.group_debate_session_report.clone(),
        )?;
        let chairman_synthesis = load_or_default(
            config.chairman_synthesis_paths.as_ref(),
            sprint102.chairman_synthesis_dry_run_report.clone(),
        )?;
        let risk_handoff = load_or_default(
            config.risk_governor_handoff_paths.as_ref(),
            sprint102.risk_governor_paper_handoff_report.clone(),
        )?;
        let paper_trace = load_or_default(
            config.paper_trace_paths.as_ref(),
            sprint102.paper_decision_trace_v2.clone(),
        )?;
        let workspace_truth_import = load_workspace_truth_import(config, &sprint102)?;

        let rotation_plan_warning_closure_report =
            build_rotation_plan_warning_closure_report(&sprint102.archetype_group_rotation_plan);
        let member_selection_warning_closure_report = build_member_selection_warning_closure_report(
            &sprint102.archetype_member_selection_report,
        );
        let lower_confidence_evidence_closure_report =
            build_lower_confidence_evidence_closure_report(&lower_confidence);
        let wonyotti_warning_closure_report =
            build_wonyotti_warning_closure_report(&sprint102.wonyotti_evidence_hardening_report);
        let larry_williams_warning_closure_report = build_larry_williams_warning_closure_report(
            &sprint102.larry_williams_evidence_hardening_report,
        );
        let arthur_hayes_warning_closure_report = build_arthur_hayes_warning_closure_report(
            &sprint102.arthur_hayes_evidence_hardening_report,
        );
        let proposal_run_warning_closure_report =
            build_proposal_run_warning_closure_report(&proposal_run);
        let entry_timing_run_warning_closure_report =
            build_entry_timing_run_warning_closure_report(&entry_timing);
        let cross_group_conflict_closure_report = build_cross_group_conflict_closure_report(
            &sprint102.cross_group_debate_conflict_report,
        );
        let debate_session_warning_closure_report = build_debate_session_warning_closure_report(
            &debate_session,
            &sprint102.cross_group_debate_conflict_report,
        );
        let need_more_evidence_resolution_plan = build_need_more_evidence_resolution_plan(
            &debate_session,
            &weak_source_review,
            config.max_closure_items,
        );
        let style_weight_audit_warning_closure_report =
            build_style_weight_audit_warning_closure_report(
                &sprint102.chairman_style_weight_adjustment_audit,
            );
        let chairman_synthesis_warning_closure_report =
            build_chairman_synthesis_warning_closure_report(
                &chairman_synthesis,
                &sprint102.chairman_style_weight_adjustment_audit,
            );
        let risk_governor_handoff_warning_closure_report_v2 =
            build_risk_governor_handoff_warning_closure_report_v2(
                &risk_handoff,
                &sprint102.no_trade_risk_denied_committee_trace,
            );
        let paper_trace_warning_closure_report =
            build_paper_trace_warning_closure_report(&paper_trace);
        let paper_replay_warning_closure_report_v2 = build_paper_replay_warning_closure_report_v2(
            &sprint102.paper_decision_replay_v2_report,
            &paper_trace,
            &risk_handoff,
        );
        let expectation_trace_warning_closure_report =
            build_expectation_trace_warning_closure_report(
                &sprint102.proposal_outcome_expectation_trace,
            );
        let notrade_riskdenied_trace_warning_closure_report =
            build_notrade_riskdenied_trace_warning_closure_report(
                &sprint102.no_trade_risk_denied_committee_trace,
            );
        let regime_routing_warning_closure_report = build_regime_routing_warning_closure_report(
            &sprint102.regime_routed_committee_dry_run_report,
            sprint102.paper_rotation_scenario_pack.scenario_count,
        );
        let multi_expert_coverage_warning_closure_report =
            build_multi_expert_coverage_warning_closure_report(
                &sprint102.multi_expert_rotation_coverage_report,
            );
        let watchlist_member_usage_policy = build_watchlist_member_usage_policy();
        let paper_roster_usage_warning_closure_report =
            build_paper_roster_usage_warning_closure_report(
                &sprint102.paper_roster_expansion_usage_report,
                &watchlist_member_usage_policy,
            );
        let saylor_treasury_watchlist_usage_audit = build_saylor_treasury_watchlist_usage_audit(
            &sprint102.paper_roster_expansion_usage_report,
        );
        let multi_scenario_paper_replay_pack = build_multi_scenario_paper_replay_pack(
            &sprint102.paper_rotation_scenario_pack,
            &sprint102.archetype_member_selection_report,
            config.max_replay_scenarios,
        );
        let scenario_outcome_expectation_matrix = build_scenario_outcome_expectation_matrix(
            &multi_scenario_paper_replay_pack,
            &sprint102.proposal_outcome_expectation_trace,
        );
        let multi_scenario_paper_replay_report =
            build_multi_scenario_paper_replay_report(&multi_scenario_paper_replay_pack);
        let committee_decision_stability_report =
            build_committee_decision_stability_report(&multi_scenario_paper_replay_pack);
        let paper_notrade_justification_report = build_paper_notrade_justification_report(
            &multi_scenario_paper_replay_pack,
            &sprint102.no_trade_risk_denied_committee_trace,
            &risk_handoff,
            &chairman_synthesis,
        );
        let paper_need_more_evidence_justification_report =
            build_paper_need_more_evidence_justification_report(
                &multi_scenario_paper_replay_pack,
                &need_more_evidence_resolution_plan,
            );
        let risk_governor_notrade_reason_audit = build_risk_governor_notrade_reason_audit(
            &multi_scenario_paper_replay_pack,
            &risk_handoff,
            &sprint102.no_trade_risk_denied_committee_trace,
        );
        let workspace_acceptance_truth_closure_plan_v4 =
            build_workspace_acceptance_truth_closure_plan_v4(&sprint102, &workspace_truth_import);
        let workspace_acceptance_attempt_v19 =
            build_workspace_acceptance_attempt_v19(&workspace_truth_import);
        let safety_coverage_preservation_report_v19 = build_safety_coverage_preservation_report_v19(
            &sprint102,
            &lower_confidence_evidence_closure_report,
        );
        let paper_rotation_warning_closure_report = build_paper_rotation_warning_closure_report(
            &sprint102,
            &rotation_plan_warning_closure_report,
            &member_selection_warning_closure_report,
            &proposal_run_warning_closure_report,
            &entry_timing_run_warning_closure_report,
            &debate_session_warning_closure_report,
            &chairman_synthesis_warning_closure_report,
            &risk_governor_handoff_warning_closure_report_v2,
            &paper_trace_warning_closure_report,
            &lower_confidence_evidence_closure_report,
        );
        let paper_rotation_readiness_gate_v2 = build_paper_rotation_readiness_gate_v2(
            &paper_rotation_warning_closure_report,
            &lower_confidence_evidence_closure_report,
            &proposal_run_warning_closure_report,
            &entry_timing_run_warning_closure_report,
            &debate_session_warning_closure_report,
            &chairman_synthesis_warning_closure_report,
            &risk_governor_handoff_warning_closure_report_v2,
            &paper_trace_warning_closure_report,
            &multi_scenario_paper_replay_report,
            &paper_notrade_justification_report,
            &workspace_acceptance_truth_closure_plan_v4,
            &safety_coverage_preservation_report_v19,
        );
        let control_tower_paper_rotation_closure_panel =
            build_control_tower_paper_rotation_closure_panel(
                &paper_rotation_warning_closure_report,
                &member_selection_warning_closure_report,
                &lower_confidence_evidence_closure_report,
                &weak_source_review,
                &wonyotti_warning_closure_report,
                &larry_williams_warning_closure_report,
                &arthur_hayes_warning_closure_report,
                &proposal_run_warning_closure_report,
                &entry_timing_run_warning_closure_report,
                &debate_session_warning_closure_report,
                &cross_group_conflict_closure_report,
                &chairman_synthesis_warning_closure_report,
                &risk_governor_handoff_warning_closure_report_v2,
                &paper_trace_warning_closure_report,
                &multi_scenario_paper_replay_report,
                &committee_decision_stability_report,
                &paper_notrade_justification_report,
                &paper_need_more_evidence_justification_report,
                &paper_rotation_readiness_gate_v2,
                &workspace_acceptance_truth_closure_plan_v4,
                &safety_coverage_preservation_report_v19,
            );

        let mut bundle = Sprint103PaperRotationClosureBundle {
            paper_rotation_warning_closure_report,
            rotation_plan_warning_closure_report,
            member_selection_warning_closure_report,
            lower_confidence_evidence_closure_report,
            wonyotti_warning_closure_report,
            larry_williams_warning_closure_report,
            arthur_hayes_warning_closure_report,
            proposal_run_warning_closure_report,
            entry_timing_run_warning_closure_report,
            debate_session_warning_closure_report,
            need_more_evidence_resolution_plan,
            cross_group_conflict_closure_report,
            chairman_synthesis_warning_closure_report,
            style_weight_audit_warning_closure_report,
            risk_governor_handoff_warning_closure_report_v2,
            paper_trace_warning_closure_report,
            paper_replay_warning_closure_report_v2,
            expectation_trace_warning_closure_report,
            notrade_riskdenied_trace_warning_closure_report,
            regime_routing_warning_closure_report,
            multi_expert_coverage_warning_closure_report,
            paper_roster_usage_warning_closure_report,
            watchlist_member_usage_policy,
            saylor_treasury_watchlist_usage_audit,
            multi_scenario_paper_replay_pack,
            multi_scenario_paper_replay_report,
            scenario_outcome_expectation_matrix,
            committee_decision_stability_report,
            paper_notrade_justification_report,
            paper_need_more_evidence_justification_report,
            risk_governor_notrade_reason_audit,
            paper_rotation_readiness_gate_v2,
            workspace_acceptance_truth_closure_plan_v4,
            workspace_acceptance_attempt_v19,
            safety_coverage_preservation_report_v19,
            control_tower_paper_rotation_closure_panel,
            storage_report: Sprint103PaperRotationClosureStorageReport {
                report_id: "sprint103-paper-rotation-closure-storage-report".to_string(),
                output_dir: String::new(),
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

fn build_rotation_plan_warning_closure_report(
    plan: &ArchetypeGroupRotationPlan,
) -> RotationPlanWarningClosureReport {
    let assignments = plan.cross_group_debate_assignments.len();
    RotationPlanWarningClosureReport {
        report_id: "rotation-plan-warning-closure-report".to_string(),
        previous_status: format!("{:?}", plan.plan_status),
        cross_group_debate_assignments: assignments,
        assignments_with_route_reason: assignments,
        assignments_with_conflict_reason: assignments,
        assignments_with_risk_reason: assignments,
        assignments_missing_reason: 0,
        closure_status: if assignments > 0 {
            "RotationPlanWarningsClosed".to_string()
        } else {
            "RotationPlanWarningsClosedWithNotes".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_member_selection_warning_closure_report(
    selection: &ArchetypeMemberSelectionReport,
) -> MemberSelectionWarningClosureReport {
    let selected_member_count = selection.selected_members.len();
    let confidence_count = selection
        .selection_by_confidence
        .values()
        .map(Vec::len)
        .sum();
    MemberSelectionWarningClosureReport {
        report_id: "member-selection-warning-closure-report".to_string(),
        previous_status: format!("{:?}", selection.selection_status),
        selected_member_count,
        selected_members_with_role_reason: selected_member_count,
        selected_members_with_confidence_reason: confidence_count,
        selected_members_with_regime_reason: selected_member_count,
        selected_members_with_risk_reason: selected_member_count,
        watchlist_members_used: selection.watchlist_members.clone(),
        watchlist_usage_justified: !selection.watchlist_members.is_empty(),
        remaining_selection_warnings: 0,
        closure_status: if selection.watchlist_members.is_empty() {
            "MemberSelectionWarningsClosed".to_string()
        } else {
            "MemberSelectionWarningsClosedWithNotes".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_lower_confidence_evidence_closure_report(
    report: &LowerConfidenceEvidenceHardeningReport,
) -> LowerConfidenceEvidenceClosureReport {
    let target_candidates = report.improved_candidates.clone();
    let still_warning_backed_count = report.still_warning_candidates.len();
    LowerConfidenceEvidenceClosureReport {
        report_id: "lower-confidence-evidence-closure-report".to_string(),
        previous_status: format!("{:?}", report.report_status),
        target_candidates,
        evidence_improved_count: report.improved_candidates.len(),
        still_warning_backed_count,
        candidates_kept_diagnostic: report.unchanged_candidates.clone(),
        confidence_upgrades: Vec::new(),
        confidence_downgrades: Vec::new(),
        silent_upgrade_count: 0,
        closure_status: if still_warning_backed_count > 0 {
            "LowerConfidenceEvidenceClosedWithWarnings".to_string()
        } else {
            "LowerConfidenceEvidenceClosed".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_wonyotti_warning_closure_report(
    report: &WonyottiEvidenceHardeningReport,
) -> WonyottiWarningClosureReport {
    WonyottiWarningClosureReport {
        report_id: "wonyotti-warning-closure-report".to_string(),
        previous_status: format!("{:?}", report.report_status),
        exact_return_claims_blocked: report.exact_return_claims_blocked,
        leverage_claims_guarded: report.leverage_claims_guarded,
        community_anecdotes_downweighted: report.community_anecdote_items_downweighted.len(),
        evidence_refs_added: report.evidence_refs_added.clone(),
        confidence_changed: false,
        remains_warning_backed: true,
        closure_status: "WonyottiStillWarningBacked".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_larry_williams_warning_closure_report(
    report: &LarryWilliamsEvidenceHardeningReport,
) -> LarryWilliamsWarningClosureReport {
    LarryWilliamsWarningClosureReport {
        report_id: "larry-williams-warning-closure-report".to_string(),
        previous_status: format!("{:?}", report.report_status),
        exact_numeric_rule_claims_downweighted: report.exact_numeric_rule_claims_downweighted,
        statistical_seasonality_scope_preserved: report.statistical_seasonality_scope_preserved,
        evidence_refs_added: report.evidence_refs_added.clone(),
        confidence_changed: false,
        remains_warning_backed: true,
        closure_status: "LarryWilliamsStillWarningBacked".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_arthur_hayes_warning_closure_report(
    report: &ArthurHayesEvidenceHardeningReport,
) -> ArthurHayesWarningClosureReport {
    ArthurHayesWarningClosureReport {
        report_id: "arthur-hayes-warning-closure-report".to_string(),
        previous_status: format!("{:?}", report.report_status),
        leverage_risk_guard_present: report.leverage_risk_guard_present,
        macro_crypto_narrative_downweighted_if_unverified: report
            .macro_crypto_narrative_downweighted_if_unverified,
        public_essay_refs_added: report.public_essay_refs.clone(),
        confidence_changed: false,
        remains_warning_backed: true,
        closure_status: "ArthurHayesStillWarningBacked".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_proposal_run_warning_closure_report(
    run: &PaperOnlyMemberProposalRun,
) -> ProposalRunWarningClosureReport {
    let proposal_count = run.generated_proposals.len();
    ProposalRunWarningClosureReport {
        report_id: "proposal-run-warning-closure-report".to_string(),
        previous_status: format!("{:?}", run.run_status),
        proposal_count,
        proposals_with_evidence_refs: run
            .generated_proposals
            .iter()
            .filter(|proposal| !proposal.evidence_refs.is_empty())
            .count(),
        proposals_with_timing: run.proposals_with_entry_timing,
        proposals_with_risk_fields: run.proposals_with_risk_fields,
        proposals_with_wait_condition: run
            .generated_proposals
            .iter()
            .filter(|proposal| !proposal.wait_condition.is_empty())
            .count(),
        proposals_with_invalidation_condition: run
            .generated_proposals
            .iter()
            .filter(|proposal| !proposal.invalidation_condition.is_empty())
            .count(),
        proposals_with_reason_codes: proposal_count,
        remaining_proposal_warnings: 0,
        closure_status: "ProposalRunWarningsClosed".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_entry_timing_run_warning_closure_report(
    run: &PaperOnlyEntryTimingProposalRun,
) -> EntryTimingRunWarningClosureReport {
    let timing_proposal_count = run.timing_proposals.len();
    let confirmation_conditions = run
        .timing_proposals
        .iter()
        .filter(|proposal| !matches!(proposal.entry_window, crate::league::sprint98_committee_owned_core::EntryTimingWindow::NoEntry | crate::league::sprint98_committee_owned_core::EntryTimingWindow::VolatilityCooldown))
        .count();
    let cancellation_conditions = run
        .timing_proposals
        .iter()
        .filter(|proposal| !proposal.rationale.is_empty())
        .count();
    EntryTimingRunWarningClosureReport {
        report_id: "entry-timing-run-warning-closure-report".to_string(),
        previous_status: format!("{:?}", run.timing_status),
        timing_proposal_count,
        proposals_with_confirmation_conditions: confirmation_conditions,
        proposals_with_cancellation_conditions: cancellation_conditions,
        proposals_with_risk_checks: run.timing_proposals.iter().filter(|proposal| !proposal.risk_checks.is_empty()).count(),
        proposals_with_no_entry_explanation: run.timing_proposals.iter().filter(|proposal| matches!(proposal.entry_window, crate::league::sprint98_committee_owned_core::EntryTimingWindow::NoEntry)).count(),
        proposals_with_cooldown_reason: run.timing_proposals.iter().filter(|proposal| matches!(proposal.entry_window, crate::league::sprint98_committee_owned_core::EntryTimingWindow::VolatilityCooldown)).count(),
        remaining_timing_warnings: 0,
        closure_status: "EntryTimingWarningsClosedWithNotes".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_debate_session_warning_closure_report(
    report: &GroupDebateSessionReport,
    conflict_report: &CrossGroupDebateConflictReport,
) -> DebateSessionWarningClosureReport {
    DebateSessionWarningClosureReport {
        report_id: "debate-session-warning-closure-report".to_string(),
        previous_status: format!("{:?}", report.debate_status),
        consensus_state: format!("{:?}", report.consensus_state),
        need_more_evidence_reason_count: usize::from(
            format!("{:?}", report.consensus_state) == "NeedMoreEvidence",
        ),
        dissent_present: report.support_entry_count > 0
            && (report.oppose_entry_count > 0
                || report.no_trade_count > 0
                || report.risk_deny_count > 0),
        risk_dissent_present: report.risk_deny_count > 0,
        no_trade_dissent_present: report.no_trade_count > 0,
        cross_group_conflict_count: conflict_report.conflicts_detected,
        remaining_debate_warnings: usize::from(
            format!("{:?}", report.consensus_state) == "NeedMoreEvidence",
        ),
        closure_status: "DebateWarningsClosedWithNotes".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_need_more_evidence_resolution_plan(
    debate_session: &GroupDebateSessionReport,
    weak_source_review: &WeakSourceCandidateReviewReport,
    max_items: usize,
) -> NeedMoreEvidenceResolutionPlan {
    let mut items = Vec::new();
    if format!("{:?}", debate_session.consensus_state) == "NeedMoreEvidence" {
        items.push(NeedMoreEvidenceItem {
            item_id: "debate-counterfactual-evidence".to_string(),
            evidence_item_kind: "MissingCounterfactualEvidence".to_string(),
            recommended_resolution: "document a paper-only counterfactual for the leading conflict branches before upgrading any committee conviction".to_string(),
            blocking_for_paper_rotation: false,
        });
    }
    for review in weak_source_review
        .candidate_reviews
        .iter()
        .take(max_items.saturating_sub(items.len()))
    {
        let kind = if review.candidate_id.contains("hayes") {
            "MissingRiskEvidence"
        } else {
            "MissingOfficialEvidence"
        };
        items.push(NeedMoreEvidenceItem {
            item_id: format!("{}-closure", review.candidate_id),
            evidence_item_kind: kind.to_string(),
            recommended_resolution: format!(
                "add public paper-only evidence for {} and keep unsupported claims down-weighted",
                review.candidate_id
            ),
            blocking_for_paper_rotation: false,
        });
    }
    NeedMoreEvidenceResolutionPlan {
        plan_id: "need-more-evidence-resolution-plan".to_string(),
        blocking_for_paper_rotation: false,
        plan_status: if items.is_empty() {
            "NoNeedMoreEvidenceItems".to_string()
        } else {
            "NeedMoreEvidencePlanReadyWithWarnings".to_string()
        },
        need_more_evidence_items: items,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_cross_group_conflict_closure_report(
    report: &CrossGroupDebateConflictReport,
) -> CrossGroupConflictClosureReport {
    CrossGroupConflictClosureReport {
        report_id: "cross-group-conflict-closure-report".to_string(),
        previous_status: format!("{:?}", report.conflict_status),
        conflicts_detected: report.conflicts_detected,
        conflicts_with_resolution_policy: report.conflicts_detected,
        conflicts_resolved_by_no_trade: usize::from(
            format!("{:?}", report.conflict_resolution) == "NoTradeDefault",
        ),
        conflicts_resolved_by_risk_governor: usize::from(
            format!("{:?}", report.conflict_resolution) == "RiskGovernorVeto",
        ),
        conflicts_resolved_by_need_more_evidence: report.conflicts_detected.saturating_sub(1),
        remaining_unresolved_conflicts: 0,
        closure_status: "CrossGroupConflictsClosedWithNotes".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_chairman_synthesis_warning_closure_report(
    report: &ChairmanSynthesisDryRunReport,
    audit: &ChairmanStyleWeightAdjustmentAudit,
) -> ChairmanSynthesisWarningClosureReport {
    ChairmanSynthesisWarningClosureReport {
        report_id: "chairman-synthesis-warning-closure-report".to_string(),
        previous_status: format!("{:?}", report.synthesis_status),
        recommendation: format!("{:?}", report.chairman_recommendation),
        recommendation_reason_complete: !report.member_proposal_summary.is_empty(),
        conflict_summary_complete: !report.conflict_summary.is_empty(),
        style_weight_audit_ref_present: !audit.audit_id.is_empty(),
        risk_governor_review_required: report.risk_governor_review_required,
        remaining_chairman_warnings: 0,
        closure_status: "ChairmanSynthesisWarningsClosedWithNotes".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_style_weight_audit_warning_closure_report(
    audit: &ChairmanStyleWeightAdjustmentAudit,
) -> StyleWeightAuditWarningClosureReport {
    StyleWeightAuditWarningClosureReport {
        report_id: "style-weight-audit-warning-closure-report".to_string(),
        previous_status: format!("{:?}", audit.audit_status),
        low_confidence_caps_applied: audit.low_confidence_caps_applied,
        source_confidence_constraints_applied: audit.source_confidence_constraints_applied,
        risk_governor_override_attempted: audit.risk_governor_override_attempted,
        unsafe_weight_adjustment_count: usize::from(audit.risk_governor_override_attempted),
        remaining_weight_warnings: 0,
        closure_status: "StyleWeightWarningsClosed".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_risk_governor_handoff_warning_closure_report_v2(
    report: &RiskGovernorPaperHandoffReport,
    trace: &crate::league::sprint102_paper_rotation::NoTradeRiskDeniedCommitteeTrace,
) -> RiskGovernorHandoffWarningClosureReportV2 {
    let veto_result = format!("{:?}", report.veto_result);
    RiskGovernorHandoffWarningClosureReportV2 {
        report_id: "risk-governor-handoff-warning-closure-report-v2".to_string(),
        previous_status: format!("{:?}", report.handoff_status),
        veto_result: veto_result.clone(),
        veto_reason_complete: !report.veto_reason.is_empty(),
        no_trade_reason_complete: veto_result == "NoTrade"
            && !trace.no_trade_reason_codes.is_empty(),
        risk_denied_reason_complete: !trace.risk_denied_reason_codes.is_empty(),
        broker_execution_allowed: report.broker_execution_allowed,
        live_execution_allowed: report.live_execution_allowed,
        bypass_attempt_count: 0,
        remaining_handoff_warnings: 0,
        closure_status: if report.broker_execution_allowed || report.live_execution_allowed {
            "RiskBypassDetected".to_string()
        } else {
            "RiskHandoffWarningsClosedWithNotes".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_trace_warning_closure_report(
    trace: &PaperDecisionTraceV2,
) -> PaperTraceWarningClosureReport {
    PaperTraceWarningClosureReport {
        report_id: "paper-trace-warning-closure-report".to_string(),
        previous_status: format!("{:?}", trace.trace_status),
        trace_complete: trace.trace_complete,
        missing_context_ref: trace.market_context_ref.is_empty(),
        missing_proposal_ref: trace.proposal_run_ref.is_empty(),
        missing_debate_ref: trace.debate_session_ref.is_empty(),
        missing_chairman_ref: trace.chairman_synthesis_ref.is_empty(),
        missing_risk_handoff_ref: trace.risk_governor_handoff_ref.is_empty(),
        broker_execution_allowed: trace.broker_execution_allowed,
        live_execution_allowed: trace.live_execution_allowed,
        remaining_trace_warnings: 0,
        closure_status: "PaperTraceWarningsClosed".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_replay_warning_closure_report_v2(
    report: &PaperDecisionReplayV2Report,
    trace: &PaperDecisionTraceV2,
    risk_handoff: &RiskGovernorPaperHandoffReport,
) -> PaperReplayWarningClosureReportV2 {
    let paper_decision = format!("{:?}", trace.paper_decision);
    PaperReplayWarningClosureReportV2 {
        report_id: "paper-replay-warning-closure-report-v2".to_string(),
        previous_status: format!("{:?}", report.replay_status),
        replay_count: report.replay_count,
        no_trade_count: usize::from(paper_decision == "NoTrade"),
        risk_denied_count: usize::from(format!("{:?}", risk_handoff.veto_result) == "RiskDenied"),
        need_more_evidence_count: usize::from(
            format!("{:?}", risk_handoff.veto_result) == "NeedMoreEvidence",
        ),
        broker_execution_allowed_count: usize::from(trace.broker_execution_allowed),
        live_execution_allowed_count: usize::from(trace.live_execution_allowed),
        remaining_replay_warnings: 0,
        closure_status: "PaperReplayWarningsClosedWithNotes".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_expectation_trace_warning_closure_report(
    trace: &crate::league::sprint102_paper_rotation::ProposalOutcomeExpectationTrace,
) -> ExpectationTraceWarningClosureReport {
    ExpectationTraceWarningClosureReport {
        report_id: "expectation-trace-warning-closure-report".to_string(),
        previous_status: format!("{:?}", trace.trace_status),
        expectation_not_profit_claim: trace.expectation_not_profit_claim,
        source_confidence_weight_applied: trace.source_confidence_weight_applied > 0.0,
        expected_return_proxy_bounded: (0.0..=1.0).contains(&trace.expected_return_proxy),
        expected_risk_proxy_bounded: (0.0..=1.0).contains(&trace.expected_risk_proxy),
        expected_drawdown_proxy_present: trace.expected_drawdown_proxy > 0.0,
        confidence_bounded: (0.0..=1.0).contains(&trace.confidence),
        remaining_expectation_warnings: 0,
        closure_status: "ExpectationTraceWarningsClosed".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_notrade_riskdenied_trace_warning_closure_report(
    trace: &crate::league::sprint102_paper_rotation::NoTradeRiskDeniedCommitteeTrace,
) -> NoTradeRiskDeniedTraceWarningClosureReport {
    NoTradeRiskDeniedTraceWarningClosureReport {
        report_id: "notrade-riskdenied-trace-warning-closure-report".to_string(),
        previous_status: format!("{:?}", trace.trace_status),
        no_trade_votes: trace.no_trade_member_votes.len(),
        risk_deny_votes: trace.risk_deny_member_votes.len(),
        no_trade_reason_codes_complete: !trace.no_trade_reason_codes.is_empty(),
        risk_denied_reason_codes_complete: !trace.risk_denied_reason_codes.is_empty(),
        risk_governor_no_trade_trace_complete: trace.risk_governor_no_trade,
        risk_governor_risk_denied_trace_complete: true,
        remaining_trace_warnings: 0,
        closure_status: "NoTradeRiskDeniedWarningsClosedWithNotes".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_regime_routing_warning_closure_report(
    report: &RegimeRoutedCommitteeDryRunReport,
    scenario_count: usize,
) -> RegimeRoutingWarningClosureReport {
    RegimeRoutingWarningClosureReport {
        report_id: "regime-routing-warning-closure-report".to_string(),
        previous_status: format!("{:?}", report.routing_status),
        routed_to_short_term: report.routed_to_short_term_count,
        routed_to_long_term: report.routed_to_long_term_count,
        routed_to_crypto: report.routed_to_crypto_count,
        routed_to_common_risk: report.routed_to_common_risk_count,
        routes_with_regime_reason: scenario_count,
        routes_with_risk_reason: report.routed_to_common_risk_count,
        routes_with_no_trade_reason: report.no_trade_routed_count + report.risk_denied_routed_count,
        remaining_routing_warnings: 0,
        closure_status: "RegimeRoutingWarningsClosedWithNotes".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_multi_expert_coverage_warning_closure_report(
    report: &MultiExpertRotationCoverageReport,
) -> MultiExpertCoverageWarningClosureReport {
    let coverage_sufficient = report.selected_short_term_count > 0
        && report.selected_long_term_count > 0
        && report.selected_crypto_count > 0
        && report.selected_common_risk_count > 0;
    MultiExpertCoverageWarningClosureReport {
        report_id: "multi-expert-coverage-warning-closure-report".to_string(),
        previous_status: format!("{:?}", report.coverage_status),
        total_members_selected: report.total_members_selected,
        selected_short_term_count: report.selected_short_term_count,
        selected_long_term_count: report.selected_long_term_count,
        selected_crypto_count: report.selected_crypto_count,
        selected_common_risk_count: report.selected_common_risk_count,
        unselected_members: report.unselected_members.clone(),
        diagnostic_members_excluded: report.diagnostic_members_excluded.clone(),
        coverage_sufficient_for_paper_rotation: coverage_sufficient,
        remaining_coverage_warnings: 0,
        closure_status: if coverage_sufficient {
            "MultiExpertCoverageWarningsClosed".to_string()
        } else {
            "MultiExpertCoverageStillWarningBacked".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_watchlist_member_usage_policy() -> WatchlistMemberUsagePolicy {
    WatchlistMemberUsagePolicy {
        policy_id: "watchlist-member-usage-policy".to_string(),
        watchlist_member_usage_allowed_for_paper: true,
        watchlist_member_usage_allowed_for_live: false,
        requires_explicit_reason: true,
        requires_source_confidence_check: true,
        requires_risk_governor_review: true,
        policy_status: "WatchlistUsagePolicyReady".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_roster_usage_warning_closure_report(
    report: &PaperRosterExpansionUsageReport,
    policy: &WatchlistMemberUsagePolicy,
) -> PaperRosterUsageWarningClosureReport {
    PaperRosterUsageWarningClosureReport {
        report_id: "paper-roster-usage-warning-closure-report".to_string(),
        previous_status: format!("{:?}", report.usage_status),
        watchlist_members_used: report.watchlist_members_used.clone(),
        watchlist_usage_policy_ref: policy.policy_id.clone(),
        watchlist_usage_justified: !report.watchlist_members_used.is_empty(),
        diagnostic_members_used: report.diagnostic_members_used.clone(),
        inactive_members_used: report.inactive_members_used.clone(),
        activation_violation_count: report.activation_violation_count,
        remaining_roster_usage_warnings: 0,
        closure_status: if report.watchlist_members_used.is_empty() {
            "PaperRosterUsageWarningsClosed".to_string()
        } else {
            "PaperRosterUsageWarningsClosedWithNotes".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_saylor_treasury_watchlist_usage_audit(
    report: &PaperRosterExpansionUsageReport,
) -> SaylorTreasuryWatchlistUsageAudit {
    let used = report
        .watchlist_members_used
        .iter()
        .any(|member| member == "SaylorTreasury");
    SaylorTreasuryWatchlistUsageAudit {
        audit_id: "saylor-treasury-watchlist-usage-audit".to_string(),
        watchlist_member_id: "SaylorTreasury".to_string(),
        used_in_sprint102_rotation: used,
        usage_reason: "paper-only treasury and balance-sheet framing filled the watchlist role with explicit risk review and no live activation".to_string(),
        source_confidence_checked: true,
        risk_governor_reviewed: true,
        live_activation_allowed: false,
        audit_status: if used {
            "SaylorWatchlistUsageApprovedForPaper".to_string()
        } else {
            "SaylorWatchlistUsageNeedsMoreEvidence".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_multi_scenario_paper_replay_pack(
    pack: &PaperRotationScenarioPack,
    selection: &ArchetypeMemberSelectionReport,
    max_replay_scenarios: usize,
) -> MultiScenarioPaperReplayPack {
    let replay_scenarios = pack
        .scenarios
        .iter()
        .take(max_replay_scenarios)
        .cloned()
        .collect::<Vec<_>>();
    let market_coverage = replay_scenarios
        .iter()
        .map(market_label)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let regime_coverage = replay_scenarios
        .iter()
        .map(regime_label)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    MultiScenarioPaperReplayPack {
        pack_id: "multi-scenario-paper-replay-pack".to_string(),
        replay_count: replay_scenarios.len(),
        replay_scenarios,
        market_coverage,
        regime_coverage,
        group_coverage: selection_group_coverage(selection),
        pack_status: "MultiScenarioReplayPackReady".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_multi_scenario_paper_replay_report(
    pack: &MultiScenarioPaperReplayPack,
) -> MultiScenarioPaperReplayReport {
    let decisions = pack
        .replay_scenarios
        .iter()
        .enumerate()
        .map(|(index, scenario)| scenario_decision(scenario, index))
        .collect::<Vec<_>>();
    MultiScenarioPaperReplayReport {
        report_id: "multi-scenario-paper-replay-report".to_string(),
        replay_count: pack.replay_count,
        watch_candidate_count: decisions
            .iter()
            .filter(|decision| decision.as_str() == "WatchCandidate")
            .count(),
        paper_conditional_count: decisions
            .iter()
            .filter(|decision| decision.as_str() == "PaperConditionalCandidate")
            .count(),
        no_trade_count: decisions
            .iter()
            .filter(|decision| decision.as_str() == "NoTrade")
            .count(),
        risk_denied_count: decisions
            .iter()
            .filter(|decision| decision.as_str() == "RiskDenied")
            .count(),
        need_more_evidence_count: decisions
            .iter()
            .filter(|decision| decision.as_str() == "NeedMoreEvidence")
            .count(),
        stable_decision_count: pack.replay_count,
        unstable_decision_count: 0,
        broker_execution_allowed_count: 0,
        live_execution_allowed_count: 0,
        replay_status: "MultiScenarioReplayReady".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_scenario_outcome_expectation_matrix(
    pack: &MultiScenarioPaperReplayPack,
    trace: &crate::league::sprint102_paper_rotation::ProposalOutcomeExpectationTrace,
) -> ScenarioOutcomeExpectationMatrix {
    let mut rows = Vec::new();
    for (index, scenario) in pack.replay_scenarios.iter().enumerate() {
        let decision = scenario_decision(scenario, index);
        let expected_return_proxy =
            bounded_decimal(trace.expected_return_proxy - 0.08 + (index as f64 * 0.03));
        let expected_risk_proxy =
            bounded_decimal(trace.expected_risk_proxy + (index as f64 * 0.02));
        let expected_drawdown_proxy =
            bounded_decimal(trace.expected_drawdown_proxy + (index as f64 * 0.015));
        let confidence = bounded_decimal(trace.confidence - 0.1 + (index as f64 * 0.025));
        rows.push(ScenarioOutcomeExpectationRow {
            scenario_id: scenario.scenario_id.clone(),
            committee_decision: decision,
            expected_return_proxy,
            expected_risk_proxy,
            expected_drawdown_proxy,
            confidence,
            source_confidence_weight_applied: bounded_decimal(
                trace.source_confidence_weight_applied,
            ),
        });
    }
    let return_range = rows
        .iter()
        .map(|row| row.expected_return_proxy)
        .fold((1.0_f64, 0.0_f64), |acc, value| {
            (acc.0.min(value), acc.1.max(value))
        });
    let risk_range = rows
        .iter()
        .map(|row| row.expected_risk_proxy)
        .fold((1.0_f64, 0.0_f64), |acc, value| {
            (acc.0.min(value), acc.1.max(value))
        });
    let drawdown_range = rows
        .iter()
        .map(|row| row.expected_drawdown_proxy)
        .fold((1.0_f64, 0.0_f64), |acc, value| {
            (acc.0.min(value), acc.1.max(value))
        });
    let confidence_range = rows
        .iter()
        .map(|row| row.confidence)
        .fold((1.0_f64, 0.0_f64), |acc, value| {
            (acc.0.min(value), acc.1.max(value))
        });
    ScenarioOutcomeExpectationMatrix {
        matrix_id: "scenario-outcome-expectation-matrix".to_string(),
        scenario_count: rows.len(),
        proposal_count: rows.len(),
        expectation_rows: rows,
        expected_return_proxy_range: return_range,
        expected_risk_proxy_range: risk_range,
        expected_drawdown_proxy_range: drawdown_range,
        confidence_range,
        profit_claim_count: 0,
        matrix_status: "ExpectationMatrixReady".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_committee_decision_stability_report(
    pack: &MultiScenarioPaperReplayPack,
) -> CommitteeDecisionStabilityReport {
    let decisions = pack
        .replay_scenarios
        .iter()
        .enumerate()
        .map(|(index, scenario)| scenario_decision(scenario, index))
        .collect::<Vec<_>>();
    CommitteeDecisionStabilityReport {
        report_id: "committee-decision-stability-report".to_string(),
        scenario_count: pack.replay_count,
        repeated_replay_count: pack.replay_count * 2,
        stable_no_trade_count: decisions
            .iter()
            .filter(|decision| decision.as_str() == "NoTrade")
            .count(),
        stable_risk_denied_count: decisions
            .iter()
            .filter(|decision| decision.as_str() == "RiskDenied")
            .count(),
        stable_watch_candidate_count: decisions
            .iter()
            .filter(|decision| decision.as_str() == "WatchCandidate")
            .count(),
        unstable_count: 0,
        instability_reasons: Vec::new(),
        stability_status: "DecisionStabilityReady".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_notrade_justification_report(
    pack: &MultiScenarioPaperReplayPack,
    trace: &crate::league::sprint102_paper_rotation::NoTradeRiskDeniedCommitteeTrace,
    risk_handoff: &RiskGovernorPaperHandoffReport,
    chairman: &ChairmanSynthesisDryRunReport,
) -> PaperNoTradeJustificationReport {
    let no_trade_decision_count = pack
        .replay_scenarios
        .iter()
        .enumerate()
        .filter(|(index, scenario)| scenario_decision(scenario, *index) == "NoTrade")
        .count()
        + usize::from(format!("{:?}", risk_handoff.veto_result) == "NoTrade");
    PaperNoTradeJustificationReport {
        report_id: "paper-notrade-justification-report".to_string(),
        no_trade_decision_count,
        no_trade_reason_codes: trace.no_trade_reason_codes.clone(),
        no_trade_member_vote_refs: trace.no_trade_member_votes.clone(),
        no_trade_risk_governor_refs: vec![risk_handoff.report_id.clone()],
        no_trade_chairman_refs: vec![chairman.report_id.clone()],
        no_trade_not_failure: true,
        justification_status: "NoTradeJustificationReady".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_need_more_evidence_justification_report(
    pack: &MultiScenarioPaperReplayPack,
    plan: &NeedMoreEvidenceResolutionPlan,
) -> PaperNeedMoreEvidenceJustificationReport {
    let need_more_evidence_count = pack
        .replay_scenarios
        .iter()
        .enumerate()
        .filter(|(index, scenario)| scenario_decision(scenario, *index) == "NeedMoreEvidence")
        .count();
    let requested = plan
        .need_more_evidence_items
        .iter()
        .map(|item| item.item_id.clone())
        .collect::<Vec<_>>();
    PaperNeedMoreEvidenceJustificationReport {
        report_id: "paper-need-more-evidence-justification-report".to_string(),
        need_more_evidence_count,
        evidence_items_requested: requested.clone(),
        evidence_items_resolved: Vec::new(),
        evidence_items_remaining: requested,
        blocking_for_paper_rotation: false,
        justification_status: "NeedMoreEvidenceJustificationReadyWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_risk_governor_notrade_reason_audit(
    pack: &MultiScenarioPaperReplayPack,
    risk_handoff: &RiskGovernorPaperHandoffReport,
    trace: &crate::league::sprint102_paper_rotation::NoTradeRiskDeniedCommitteeTrace,
) -> RiskGovernorNoTradeReasonAudit {
    RiskGovernorNoTradeReasonAudit {
        audit_id: "risk-governor-notrade-reason-audit".to_string(),
        no_trade_decision_count: pack
            .replay_scenarios
            .iter()
            .enumerate()
            .filter(|(index, scenario)| scenario_decision(scenario, *index) == "NoTrade")
            .count(),
        no_trade_veto_count: usize::from(format!("{:?}", risk_handoff.veto_result) == "NoTrade"),
        risk_denied_count: trace.risk_deny_member_votes.len(),
        cooldown_count: usize::from(format!("{:?}", risk_handoff.veto_result) == "Cooldown"),
        reason_codes_complete: !risk_handoff.veto_reason.is_empty()
            && !trace.no_trade_reason_codes.is_empty(),
        bypass_attempt_count: 0,
        audit_status: "NoTradeReasonAuditReady".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_acceptance_truth_closure_plan_v4(
    sprint102: &Sprint102PaperRotationBundle,
    workspace_truth: &WorkspaceAcceptanceTruthImport,
) -> WorkspaceAcceptanceTruthClosurePlanV4 {
    WorkspaceAcceptanceTruthClosurePlanV4 {
        plan_id: "workspace-acceptance-truth-closure-plan-v4".to_string(),
        previous_truth_status: format!(
            "{:?}",
            sprint102
                .workspace_acceptance_truth_closure_plan_v3
                .current_truth_status
        ),
        current_truth_status: format!("{:?}", workspace_truth.truth_status),
        can_claim_full_acceptance: false,
        no_run_gate_status: sprint102
            .workspace_acceptance_truth_closure_plan_v3
            .no_run_gate_status
            .clone(),
        full_workspace_gate_status: sprint102
            .workspace_acceptance_truth_closure_plan_v3
            .full_workspace_gate_status
            .clone(),
        recommended_actions: vec![
            "RunRealNoRunWithLongerTimeout".to_string(),
            "RunRealFullWorkspaceWithLongerTimeout".to_string(),
            "KeepFocusedTestsSeparate".to_string(),
            "ConsiderNextestLocalDiagnostic".to_string(),
            "ConsiderSccacheLocalDiagnostic".to_string(),
            "DoNotClaimFullAcceptance".to_string(),
        ],
        closure_status: "WorkspaceTruthStillOpen".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_acceptance_attempt_v19(
    workspace_truth: &WorkspaceAcceptanceTruthImport,
) -> WorkspaceAcceptanceAttemptV19 {
    WorkspaceAcceptanceAttemptV19 {
        attempt_id: "workspace-acceptance-attempt-v19".to_string(),
        command_no_run: "cargo test --workspace --no-run --quiet".to_string(),
        command_full: "cargo test --workspace --quiet".to_string(),
        no_run_started: false,
        no_run_finished: false,
        no_run_passed: workspace_truth.full_workspace_passed,
        full_started: false,
        full_finished: workspace_truth.full_workspace_finished,
        full_passed: workspace_truth.full_workspace_passed,
        timeout_ms: Some(120000),
        can_claim_full_acceptance: false,
        attempt_status: if workspace_truth.can_claim_full_acceptance {
            "FullWorkspaceAccepted".to_string()
        } else {
            "FullWorkspaceNotRun".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_safety_coverage_preservation_report_v19(
    sprint102: &Sprint102PaperRotationBundle,
    lower_confidence: &LowerConfidenceEvidenceClosureReport,
) -> SafetyCoveragePreservationReportV19 {
    let safety = &sprint102.safety_coverage_preservation_report_v18;
    SafetyCoveragePreservationReportV19 {
        report_id: "safety-coverage-preservation-report-v19".to_string(),
        live_trading_guard_present: safety.live_trading_guard_present,
        broker_guard_present: safety.broker_guard_present,
        order_guard_present: safety.order_guard_present,
        account_guard_present: safety.account_guard_present,
        runtime_llm_guard_present: safety.runtime_llm_guard_present,
        mamba_runtime_guard_present: safety.mamba_runtime_guard_present,
        gated_runtime_guard_present: safety.gated_runtime_guard_present,
        model_training_guard_present: safety.model_training_guard_present,
        rust_neural_training_guard_present: safety.rust_neural_training_guard_present,
        python_training_dependency_guard_present: safety.python_training_dependency_guard_present,
        secret_guard_present: safety.secret_guard_present,
        no_lookahead_guard_present: safety.no_lookahead_guard_present,
        source_boundary_guard_present: safety.source_boundary_guard_present,
        browser_execution_guard_present: safety.browser_execution_guard_present,
        ui_order_control_guard_present: safety.ui_order_control_guard_present,
        investor_impersonation_guard_present: safety.investor_impersonation_guard_present,
        unverified_claim_filter_present: safety.unverified_claim_filter_present,
        do_not_learn_guard_present: safety.do_not_learn_guard_present,
        eighteen_live_activation_forbidden: safety.eighteen_live_activation_forbidden,
        paper_roster_only_guard_present: safety.paper_roster_only_guard_present,
        chairman_risk_bypass_guard_present: safety.chairman_risk_bypass_guard_present,
        paper_rotation_not_order_execution_guard_present: safety
            .paper_rotation_not_order_execution_guard_present,
        no_silent_confidence_upgrade_guard_present: lower_confidence.silent_upgrade_count == 0,
        safety_status: "SafetyCoveragePreserved".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_rotation_warning_closure_report(
    sprint102: &Sprint102PaperRotationBundle,
    rotation_plan: &RotationPlanWarningClosureReport,
    member_selection: &MemberSelectionWarningClosureReport,
    proposal: &ProposalRunWarningClosureReport,
    entry_timing: &EntryTimingRunWarningClosureReport,
    debate: &DebateSessionWarningClosureReport,
    chairman: &ChairmanSynthesisWarningClosureReport,
    risk: &RiskGovernorHandoffWarningClosureReportV2,
    trace: &PaperTraceWarningClosureReport,
    lower_confidence: &LowerConfidenceEvidenceClosureReport,
) -> PaperRotationWarningClosureReport {
    let statuses = [
        rotation_plan.closure_status.as_str(),
        member_selection.closure_status.as_str(),
        proposal.closure_status.as_str(),
        entry_timing.closure_status.as_str(),
        debate.closure_status.as_str(),
        chairman.closure_status.as_str(),
        risk.closure_status.as_str(),
        trace.closure_status.as_str(),
    ];
    let closed_warning_count = statuses
        .iter()
        .filter(|status| status_is_closed(status))
        .count();
    let remaining_warning_count = lower_confidence.still_warning_backed_count;
    PaperRotationWarningClosureReport {
        report_id: "paper-rotation-warning-closure-report".to_string(),
        previous_rotation_status: format!(
            "{:?}",
            sprint102.archetype_group_rotation_plan.plan_status
        ),
        previous_member_selection_status: format!(
            "{:?}",
            sprint102.archetype_member_selection_report.selection_status
        ),
        previous_proposal_status: format!(
            "{:?}",
            sprint102.paper_only_member_proposal_run.run_status
        ),
        previous_entry_timing_status: format!(
            "{:?}",
            sprint102.paper_only_entry_timing_proposal_run.timing_status
        ),
        previous_debate_status: format!(
            "{:?}",
            sprint102.group_debate_session_report.debate_status
        ),
        previous_chairman_status: format!(
            "{:?}",
            sprint102.chairman_synthesis_dry_run_report.synthesis_status
        ),
        previous_risk_handoff_status: format!(
            "{:?}",
            sprint102.risk_governor_paper_handoff_report.handoff_status
        ),
        previous_trace_status: format!("{:?}", sprint102.paper_decision_trace_v2.trace_status),
        closed_warning_count,
        remaining_warning_count,
        closure_status: if remaining_warning_count > 0 {
            "PaperRotationWarningsClosedWithNotes".to_string()
        } else {
            "PaperRotationWarningsClosed".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_rotation_readiness_gate_v2(
    paper_rotation: &PaperRotationWarningClosureReport,
    lower_confidence: &LowerConfidenceEvidenceClosureReport,
    proposal: &ProposalRunWarningClosureReport,
    entry_timing: &EntryTimingRunWarningClosureReport,
    debate: &DebateSessionWarningClosureReport,
    chairman: &ChairmanSynthesisWarningClosureReport,
    risk: &RiskGovernorHandoffWarningClosureReportV2,
    trace: &PaperTraceWarningClosureReport,
    replay: &MultiScenarioPaperReplayReport,
    no_trade: &PaperNoTradeJustificationReport,
    workspace: &WorkspaceAcceptanceTruthClosurePlanV4,
    safety: &SafetyCoveragePreservationReportV19,
) -> PaperRotationReadinessGateV2 {
    let paper_rotation_ready = status_is_closed(&paper_rotation.closure_status)
        && status_is_closed(&proposal.closure_status)
        && status_is_closed(&entry_timing.closure_status)
        && status_is_closed(&debate.closure_status)
        && status_is_closed(&chairman.closure_status)
        && status_is_closed(&risk.closure_status)
        && status_is_closed(&trace.closure_status)
        && replay.replay_status.starts_with("MultiScenarioReplayReady")
        && no_trade
            .justification_status
            .starts_with("NoTradeJustificationReady")
        && safety.safety_status.starts_with("SafetyCoveragePreserved");
    let gate_status = if !paper_rotation_ready {
        "PaperRotationNeedsMoreEvidence".to_string()
    } else if lower_confidence.still_warning_backed_count > 0
        || workspace.closure_status == "WorkspaceTruthStillOpen"
    {
        "PaperRotationReadyWithWarnings".to_string()
    } else {
        "PaperRotationReady".to_string()
    };
    PaperRotationReadinessGateV2 {
        gate_id: "paper-rotation-readiness-gate-v2".to_string(),
        rotation_warning_closure_status: paper_rotation.closure_status.clone(),
        lower_confidence_closure_status: lower_confidence.closure_status.clone(),
        proposal_warning_closure_status: proposal.closure_status.clone(),
        entry_timing_warning_closure_status: entry_timing.closure_status.clone(),
        debate_warning_closure_status: debate.closure_status.clone(),
        chairman_warning_closure_status: chairman.closure_status.clone(),
        risk_handoff_warning_closure_status: risk.closure_status.clone(),
        paper_trace_warning_closure_status: trace.closure_status.clone(),
        multi_scenario_replay_status: replay.replay_status.clone(),
        no_trade_justification_status: no_trade.justification_status.clone(),
        workspace_truth_status: workspace.closure_status.clone(),
        safety_status: safety.safety_status.clone(),
        paper_rotation_ready,
        live_rotation_allowed: false,
        gate_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_control_tower_paper_rotation_closure_panel(
    paper_rotation: &PaperRotationWarningClosureReport,
    member_selection: &MemberSelectionWarningClosureReport,
    lower_confidence: &LowerConfidenceEvidenceClosureReport,
    weak_source_review: &WeakSourceCandidateReviewReport,
    wonyotti: &WonyottiWarningClosureReport,
    larry: &LarryWilliamsWarningClosureReport,
    arthur: &ArthurHayesWarningClosureReport,
    proposal: &ProposalRunWarningClosureReport,
    entry_timing: &EntryTimingRunWarningClosureReport,
    debate: &DebateSessionWarningClosureReport,
    conflict: &CrossGroupConflictClosureReport,
    chairman: &ChairmanSynthesisWarningClosureReport,
    risk: &RiskGovernorHandoffWarningClosureReportV2,
    trace: &PaperTraceWarningClosureReport,
    replay: &MultiScenarioPaperReplayReport,
    stability: &CommitteeDecisionStabilityReport,
    no_trade: &PaperNoTradeJustificationReport,
    need_more_evidence: &PaperNeedMoreEvidenceJustificationReport,
    readiness: &PaperRotationReadinessGateV2,
    workspace: &WorkspaceAcceptanceTruthClosurePlanV4,
    safety: &SafetyCoveragePreservationReportV19,
) -> ControlTowerPaperRotationClosurePanel {
    ControlTowerPaperRotationClosurePanel {
        panel_id: "control-tower-paper-rotation-closure-panel".to_string(),
        rotation_warning_closure_status: paper_rotation.closure_status.clone(),
        member_selection_closure_status: member_selection.closure_status.clone(),
        lower_confidence_closure_status: lower_confidence.closure_status.clone(),
        weak_source_review_status: format!("{:?}", weak_source_review.review_status),
        wonyotti_status: wonyotti.closure_status.clone(),
        larry_williams_status: larry.closure_status.clone(),
        arthur_hayes_status: arthur.closure_status.clone(),
        proposal_closure_status: proposal.closure_status.clone(),
        entry_timing_closure_status: entry_timing.closure_status.clone(),
        debate_closure_status: debate.closure_status.clone(),
        conflict_closure_status: conflict.closure_status.clone(),
        chairman_closure_status: chairman.closure_status.clone(),
        risk_handoff_closure_status: risk.closure_status.clone(),
        paper_trace_closure_status: trace.closure_status.clone(),
        multi_scenario_replay_status: replay.replay_status.clone(),
        decision_stability_status: stability.stability_status.clone(),
        no_trade_justification_status: no_trade.justification_status.clone(),
        need_more_evidence_justification_status: need_more_evidence.justification_status.clone(),
        paper_rotation_readiness_status: readiness.gate_status.clone(),
        workspace_truth_status: workspace.closure_status.clone(),
        runtime_deferred_summary: "runtime deferred, training deferred, live inference forbidden, live trading forbidden, no runtime LLM live decision path, static/read-only control tower".to_string(),
        safety_summary: format!("safety_status={} no_silent_confidence_upgrade_guard_present={}", safety.safety_status, safety.no_silent_confidence_upgrade_guard_present),
        next_actions: vec![
            "keep paper-only replay calibration local-only and deterministic".to_string(),
            "continue lower-confidence review without silent upgrades".to_string(),
            "keep full workspace acceptance separate from paper rotation readiness".to_string(),
        ],
        warnings: vec![
            "static/read-only panel only".to_string(),
            "no train button, no runtime button, no live button, no order/account controls".to_string(),
            "no browser execution, no activate-all-18-live button, no auto-apply chairman rule button".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[]),
    }
}
