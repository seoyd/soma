use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::league::sprint104_dual_agent_paper_lifecycle::{
    ArthurHayesCarryForwardReview, DualAgentReviewLoopReport, DualAgentWorkflowConfig,
    FinalVerificationGate, LarryWilliamsCarryForwardReview, LowerConfidenceCarryForwardPolicy,
    PaperCandidateDecisionTraceReport, PaperCandidateEvidenceDepthReport,
    PaperCandidateLifecycleStateMachine, PaperCandidateNoTradeGate, PaperCandidatePromotionGate,
    PaperCandidateRejectionGate, PaperCandidateRiskDeniedGate, PaperCandidateStabilityReport,
    PaperCandidateWatchlistGate, PaperRotationBatchReplayReport, RiskGovernorBatchVetoReport,
    SafetyCoveragePreservationReportV20, Sprint104DualAgentPaperLifecycleBundle,
    Sprint104DualAgentPaperLifecycleRunner, VerificationFinding, VerificationFindingSeverity,
    VerificationFindingStatus, WonyottiCarryForwardReview, WorkspaceAcceptanceTruthClosurePlanV5,
};

const SPRINT105_DOCS: &[&str] = &[
    "docs/SPRINT105_VERIFICATION_PATCH_CLOSURE.md",
    "docs/OVERCLAIM_REGRESSION_GUARD.md",
    "docs/PAPER_CANDIDATE_LIFECYCLE_CLOSURE.md",
    "docs/RISK_GOVERNOR_REQUIRED_TRANSITIONS.md",
    "docs/SAFETY_BOOLEAN_COVERAGE_AUDIT.md",
    "docs/MISSING_ARTIFACT_FINDING_POLICY.md",
    "docs/PAPER_LIFECYCLE_READINESS_GATE_V2.md",
    "docs/WORKSPACE_ACCEPTANCE_TRUTH_RECOVERY_V6.md",
    "docs/SPRINT105_REPORT.md",
];

const SPRINT105_EXAMPLES: &[&str] = &[
    "examples/soma_sprint105_verification_patch_close.toml",
    "examples/soma_verification_finding_closure.toml",
    "examples/soma_review_patch_effect.toml",
    "examples/soma_overclaim_regression_guard.toml",
    "examples/soma_workspace_attempt_truth_hardening.toml",
    "examples/soma_safety_boolean_coverage_audit.toml",
    "examples/soma_paper_rejected_transition_audit.toml",
    "examples/soma_risk_required_transition_audit.toml",
    "examples/soma_missing_artifact_finding_policy.toml",
    "examples/soma_final_verification_gate_v2.toml",
    "examples/soma_dual_agent_review_loop_v2.toml",
    "examples/soma_paper_lifecycle_warning_closure.toml",
    "examples/soma_paper_candidate_transition_coverage.toml",
    "examples/soma_paper_candidate_gate_completeness.toml",
    "examples/soma_paper_candidate_evidence_depth_closure.toml",
    "examples/soma_paper_candidate_trace_closure.toml",
    "examples/soma_paper_candidate_stability_closure.toml",
    "examples/soma_risk_governor_batch_veto_warning_closure.toml",
    "examples/soma_risk_governor_no_bypass_audit_v2.toml",
    "examples/soma_lower_confidence_carry_forward_closure.toml",
    "examples/soma_paper_lifecycle_readiness_gate_v2.toml",
    "examples/soma_paper_candidate_batch_replay_v2.toml",
    "examples/soma_workspace_acceptance_truth_recovery_plan_v6.toml",
    "examples/soma_workspace_compile_cost_diagnosis_v2.toml",
    "examples/soma_focused_vs_full_gate_bridge_v2.toml",
    "examples/soma_safety_coverage_preservation_v21.toml",
    "examples/soma_control_tower_verification_patch_closure.toml",
    "examples/soma_control_tower_paper_lifecycle_closure.toml",
];

const SPRINT105_TESTS: &[&str] = &[
    "tests/verification_finding_closure.rs",
    "tests/review_patch_effect.rs",
    "tests/overclaim_regression_guard.rs",
    "tests/workspace_attempt_truth_hardening.rs",
    "tests/safety_boolean_coverage_audit.rs",
    "tests/paper_rejected_transition_audit.rs",
    "tests/risk_required_transition_audit.rs",
    "tests/final_verification_gate_v2.rs",
    "tests/paper_lifecycle_warning_closure.rs",
    "tests/paper_candidate_transition_coverage.rs",
    "tests/risk_governor_batch_veto_warning_closure.rs",
    "tests/lower_confidence_carry_forward_closure.rs",
    "tests/paper_lifecycle_readiness_gate_v2.rs",
    "tests/control_tower_verification_patch_closure_panel.rs",
    "tests/control_tower_paper_lifecycle_closure_panel.rs",
    "tests/sprint105_cli_safety.rs",
    "tests/sprint105_determinism.rs",
];

const SPRINT105_CLI_COMMANDS: &[&str] = &[
    "sprint105-verification-patch-close",
    "verification-finding-closure",
    "review-patch-effect",
    "overclaim-regression-guard",
    "workspace-attempt-truth-hardening",
    "safety-boolean-coverage-audit",
    "paper-rejected-transition-audit",
    "risk-required-transition-audit",
    "missing-artifact-finding-policy",
    "final-verification-gate-v2",
    "dual-agent-review-loop-v2",
    "paper-lifecycle-warning-closure",
    "paper-candidate-transition-coverage",
    "paper-candidate-gate-completeness",
    "paper-candidate-evidence-depth-closure",
    "paper-candidate-trace-closure",
    "paper-candidate-stability-closure",
    "risk-governor-batch-veto-warning-closure",
    "risk-governor-no-bypass-audit-v2",
    "lower-confidence-carry-forward-closure",
    "paper-lifecycle-readiness-gate-v2",
    "paper-candidate-batch-replay-v2",
    "workspace-acceptance-truth-recovery-plan-v6",
    "workspace-compile-cost-diagnosis-v2",
    "focused-vs-full-gate-bridge-v2",
    "safety-coverage-preservation-v21",
    "control-tower-verification-patch-closure",
    "control-tower-paper-lifecycle-closure",
];

fn render_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|err| err.to_string())
}

fn local_only(path: &str) -> bool {
    !path.contains("://")
}

fn full_workspace_accepted(snapshot: &WorkspaceTruthSnapshot) -> bool {
    snapshot.can_claim_full_acceptance
        && snapshot.full_finished
        && snapshot.full_passed == Some(true)
}

fn write_json_file<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    fs::write(path, render_json(value)?).map_err(|err| err.to_string())
}

fn write_text_file(path: &Path, value: &str) -> Result<(), String> {
    fs::write(path, value).map_err(|err| err.to_string())
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

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_output_root() -> String {
    "target/soma_sprint105_verification_patch_closure".to_string()
}

fn default_timeout_ms() -> Option<u64> {
    Some(120_000)
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_first_json<T: DeserializeOwned>(paths: Option<&Vec<String>>) -> Result<Option<T>, String> {
    let Some(paths) = paths else {
        return Ok(None);
    };
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        if let Ok(value) = serde_json::from_str::<T>(&text) {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint105VerificationPatchClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub sprint104_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub verification_finding_paths: Option<Vec<String>>,
    #[serde(default)]
    pub review_patch_summary_paths: Option<Vec<String>>,
    #[serde(default)]
    pub paper_lifecycle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub risk_governor_batch_paths: Option<Vec<String>>,
    #[serde(default)]
    pub lower_confidence_paths: Option<Vec<String>>,
    #[serde(default)]
    pub workspace_truth_paths: Option<Vec<String>>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub require_review_patch_closure: bool,
    #[serde(default = "default_true")]
    pub require_overclaim_guard: bool,
    #[serde(default = "default_true")]
    pub require_safety_boolean_audit: bool,
    #[serde(default = "default_true")]
    pub require_lifecycle_warning_closure: bool,
    #[serde(default = "default_true")]
    pub require_risk_transition_audit: bool,
    #[serde(default = "default_true")]
    pub require_lower_confidence_carry_forward: bool,
    #[serde(default = "default_true")]
    pub require_workspace_truth_recovery_plan: bool,
    #[serde(default = "default_true")]
    pub preserve_dual_agent_separation: bool,
    #[serde(default = "default_true")]
    pub preserve_runtime_deferred: bool,
    #[serde(default = "default_true")]
    pub preserve_safety_guards: bool,
    #[serde(default = "default_false")]
    pub run_workspace_acceptance_attempt: bool,
    #[serde(default = "default_timeout_ms")]
    pub workspace_acceptance_timeout_ms: Option<u64>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for Sprint105VerificationPatchClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "sprint105-verification-patch-closure".to_string(),
            sprint104_bundle_paths: Some(vec![
                "examples/sprint105_data/sprint104_summary.json".to_string(),
            ]),
            verification_finding_paths: None,
            review_patch_summary_paths: Some(vec![
                "examples/sprint105_data/review_patch_summary.json".to_string(),
            ]),
            paper_lifecycle_paths: None,
            risk_governor_batch_paths: None,
            lower_confidence_paths: None,
            workspace_truth_paths: None,
            output_root: default_output_root(),
            require_review_patch_closure: true,
            require_overclaim_guard: true,
            require_safety_boolean_audit: true,
            require_lifecycle_warning_closure: true,
            require_risk_transition_audit: true,
            require_lower_confidence_carry_forward: true,
            require_workspace_truth_recovery_plan: true,
            preserve_dual_agent_separation: true,
            preserve_runtime_deferred: true,
            preserve_safety_guards: true,
            run_workspace_acceptance_attempt: false,
            workspace_acceptance_timeout_ms: default_timeout_ms(),
            reason_codes: deferred_reason_codes(&[]),
        }
    }
}

impl Sprint105VerificationPatchClosureConfig {
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
            return Err("sprint105 closure_id must not be empty".to_string());
        }
        if self.output_root.trim().is_empty() {
            return Err("sprint105 output_root must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err(
                "sprint105 verification patch closure config paths must be local".to_string(),
            );
        }
        for paths in [
            &self.sprint104_bundle_paths,
            &self.verification_finding_paths,
            &self.review_patch_summary_paths,
            &self.paper_lifecycle_paths,
            &self.risk_governor_batch_paths,
            &self.lower_confidence_paths,
            &self.workspace_truth_paths,
        ] {
            if let Some(paths) = paths
                && paths.iter().any(|path| !local_only(path))
            {
                return Err(
                    "sprint105 verification patch closure config paths must be local".to_string(),
                );
            }
        }
        if self.preserve_safety_guards && !self.preserve_runtime_deferred {
            return Err(
                "sprint105 safety preservation requires runtime deferred preservation".to_string(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ReviewPatchSummary {
    overclaim_patch_detected: bool,
    workspace_attempt_patch_detected: bool,
    safety_boolean_patch_detected: bool,
    missing_artifact_policy_patch_detected: bool,
    paper_rejected_transition_patch_detected: bool,
    risk_transition_patch_detected: bool,
    no_run_long_running_observed: bool,
    full_run_long_running_observed: bool,
    review_status: String,
}

impl Default for ReviewPatchSummary {
    fn default() -> Self {
        Self {
            overclaim_patch_detected: true,
            workspace_attempt_patch_detected: true,
            safety_boolean_patch_detected: true,
            missing_artifact_policy_patch_detected: true,
            paper_rejected_transition_patch_detected: true,
            risk_transition_patch_detected: true,
            no_run_long_running_observed: true,
            full_run_long_running_observed: true,
            review_status: "Sprint104ReviewPatchesApplied".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
struct WorkspaceTruthSnapshot {
    truth_status: String,
    no_run_started: bool,
    no_run_finished: bool,
    no_run_passed: Option<bool>,
    full_started: bool,
    full_finished: bool,
    full_passed: Option<bool>,
    can_claim_full_acceptance: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationFindingClosureReport {
    pub report_id: String,
    pub blocking_findings_closed: usize,
    pub major_findings_closed: usize,
    pub known_warnings_preserved: usize,
    pub remaining_findings: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewPatchEffectReport {
    pub report_id: String,
    pub overclaim_patch_detected: bool,
    pub workspace_attempt_patch_detected: bool,
    pub safety_boolean_patch_detected: bool,
    pub missing_artifact_policy_patch_detected: bool,
    pub paper_rejected_transition_patch_detected: bool,
    pub risk_transition_patch_detected: bool,
    pub effect_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverclaimRegressionGuardReport {
    pub report_id: String,
    pub full_acceptance_requires_finished_and_passed: bool,
    pub unrun_attempt_finished: bool,
    pub focused_pass_is_full_acceptance: bool,
    pub verification_pass_is_full_acceptance: bool,
    pub no_run_pass_is_full_acceptance: bool,
    pub regression_detected: bool,
    pub guard_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAttemptTruthHardeningReport {
    pub report_id: String,
    pub attempts_not_run_count: usize,
    pub stopped_due_to_long_compile_count: usize,
    pub can_claim_full_acceptance: bool,
    pub truth_hardening_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyBooleanCoverageAuditReport {
    pub report_id: String,
    pub actual_guard_booleans_count: usize,
    pub missing_guard_booleans: Vec<String>,
    pub guard_mismatch_count: usize,
    pub audit_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperRejectedTransitionAuditReport {
    pub report_id: String,
    pub paper_rejected_state_present: bool,
    pub paper_rejected_reachable: bool,
    pub paper_rejected_can_go_live: bool,
    pub paper_rejected_can_become_order: bool,
    pub archive_transition_present: bool,
    pub audit_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskGovernorRequiredTransitionAuditReport {
    pub report_id: String,
    pub paper_approved_requires_risk: bool,
    pub paper_rejected_requires_risk: bool,
    pub risk_denied_requires_risk: bool,
    pub no_trade_requires_risk: bool,
    pub cooldown_requires_risk: bool,
    pub live_transition_present: bool,
    pub bypass_transition_count: usize,
    pub audit_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingArtifactFindingPolicyReport {
    pub report_id: String,
    pub missing_docs_become_findings: bool,
    pub missing_tests_become_findings: bool,
    pub missing_examples_become_findings: bool,
    pub silent_success_detected: bool,
    pub policy_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalVerificationGateV2 {
    pub gate_id: String,
    pub previous_gate_status: String,
    pub finding_closure_status: String,
    pub overclaim_guard_status: String,
    pub safety_boolean_audit_status: String,
    pub paper_rejected_transition_status: String,
    pub risk_transition_status: String,
    pub missing_artifact_policy_status: String,
    pub workspace_truth_status: String,
    pub blocking_findings_remaining: usize,
    pub full_workspace_accepted: bool,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DualAgentReviewLoopV2Report {
    pub report_id: String,
    pub findings_before_review: usize,
    pub findings_closed: usize,
    pub accepted_known_warnings: usize,
    pub findings_remaining: usize,
    pub patch_iterations: usize,
    pub final_gate_status: String,
    pub review_loop_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperLifecycleWarningClosureReport {
    pub report_id: String,
    pub previous_lifecycle_status: String,
    pub transition_warning_count: usize,
    pub gate_warning_count: usize,
    pub evidence_warning_count: usize,
    pub trace_warning_count: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateTransitionCoverageReport {
    pub report_id: String,
    pub total_states: usize,
    pub reachable_or_explained_states: usize,
    pub unsafe_transition_count: usize,
    pub missing_transition_explanations: Vec<String>,
    pub coverage_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateGateCompletenessReport {
    pub report_id: String,
    pub promotion_gate_present: bool,
    pub rejection_gate_present: bool,
    pub watchlist_gate_present: bool,
    pub no_trade_gate_present: bool,
    pub risk_denied_gate_present: bool,
    pub missing_gate_count: usize,
    pub completeness_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateEvidenceDepthClosureReport {
    pub report_id: String,
    pub candidate_count: usize,
    pub candidates_needing_more_evidence: usize,
    pub closed_evidence_gap_count: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateTraceClosureReport {
    pub report_id: String,
    pub traces_complete: usize,
    pub traces_missing_risk_governor: usize,
    pub broker_execution_allowed_count: usize,
    pub live_execution_allowed_count: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateStabilityClosureReport {
    pub report_id: String,
    pub stable_candidates: usize,
    pub unstable_candidates: usize,
    pub instability_reasons: Vec<String>,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskGovernorBatchVetoWarningClosureReport {
    pub report_id: String,
    pub previous_status: String,
    pub warning_count_remaining: usize,
    pub bypass_attempt_count: usize,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskGovernorTransitionCompletenessReport {
    pub report_id: String,
    pub required_transition_count: usize,
    pub missing_transition_count: usize,
    pub live_transition_present: bool,
    pub bypass_transition_count: usize,
    pub completeness_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskGovernorNoBypassAuditV2 {
    pub report_id: String,
    pub chairman_bypass_detected: bool,
    pub owner_bypass_detected: bool,
    pub member_bypass_detected: bool,
    pub bypass_transition_count: usize,
    pub audit_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LowerConfidenceCarryForwardClosureReport {
    pub report_id: String,
    pub warning_backed_candidates: Vec<String>,
    pub silent_upgrade_count: usize,
    pub live_activation_allowed: bool,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WonyottiCarryForwardClosureReport {
    pub report_id: String,
    pub remains_warning_backed: bool,
    pub exact_return_claims_blocked: bool,
    pub carry_forward_allowed_for_paper: bool,
    pub live_activation_allowed: bool,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LarryWilliamsCarryForwardClosureReport {
    pub report_id: String,
    pub remains_warning_backed: bool,
    pub exact_numeric_rule_claims_downweighted: bool,
    pub carry_forward_allowed_for_paper: bool,
    pub live_activation_allowed: bool,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArthurHayesCarryForwardClosureReport {
    pub report_id: String,
    pub remains_warning_backed: bool,
    pub leverage_risk_guard_present: bool,
    pub carry_forward_allowed_for_paper: bool,
    pub live_activation_allowed: bool,
    pub closure_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperLifecycleReadinessGateV2 {
    pub gate_id: String,
    pub transition_coverage_status: String,
    pub gate_completeness_status: String,
    pub evidence_closure_status: String,
    pub trace_closure_status: String,
    pub stability_closure_status: String,
    pub risk_veto_closure_status: String,
    pub lower_confidence_status: String,
    pub lifecycle_ready: bool,
    pub live_lifecycle_allowed: bool,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateBatchReplayV2Plan {
    pub plan_id: String,
    pub replay_count: usize,
    pub expected_paper_approved_count: usize,
    pub expected_paper_rejected_count: usize,
    pub replay_schedule: Vec<String>,
    pub plan_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateBatchReplayV2Report {
    pub report_id: String,
    pub replay_count: usize,
    pub paper_approved_count: usize,
    pub paper_rejected_count: usize,
    pub broker_execution_allowed_count: usize,
    pub live_execution_allowed_count: usize,
    pub report_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAcceptanceTruthRecoveryPlanV6 {
    pub plan_id: String,
    pub previous_truth_status: String,
    pub current_truth_status: String,
    pub compile_diagnosis_status: String,
    pub bridge_status: String,
    pub can_claim_full_acceptance: bool,
    pub recommended_actions: Vec<String>,
    pub recovery_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAcceptanceAttemptV21 {
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
pub struct WorkspaceCompileCostDiagnosisV2 {
    pub report_id: String,
    pub no_run_long_running_observed: bool,
    pub full_run_long_running_observed: bool,
    pub suspected_compile_cost_causes: Vec<String>,
    pub diagnosis_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusedVsFullGateBridgeV2 {
    pub report_id: String,
    pub focused_pass_recorded: bool,
    pub full_workspace_open: bool,
    pub bridge_rules: Vec<String>,
    pub bridge_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyCoveragePreservationReportV21 {
    pub report_id: String,
    pub live_trading_guard_present: bool,
    pub broker_guard_present: bool,
    pub order_guard_present: bool,
    pub account_guard_present: bool,
    pub runtime_llm_guard_present: bool,
    pub mamba_runtime_guard_present: bool,
    pub gated_runtime_guard_present: bool,
    pub model_training_guard_present: bool,
    pub python_training_dependency_guard_present: bool,
    pub browser_execution_guard_present: bool,
    pub dashboard_serve_guard_present: bool,
    pub investor_impersonation_guard_present: bool,
    pub no_silent_confidence_upgrade_guard_present: bool,
    pub dual_agent_workflow_guard_present: bool,
    pub verification_not_acceptance_guard_present: bool,
    pub paper_candidate_not_order_guard_present: bool,
    pub paper_candidate_lifecycle_no_live_guard_present: bool,
    pub overclaim_guard_present: bool,
    pub paper_rejected_transition_guard_present: bool,
    pub risk_governor_transition_guard_present: bool,
    pub missing_artifact_finding_guard_present: bool,
    pub safety_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerVerificationPatchClosurePanel {
    pub panel_id: String,
    pub finding_closure_status: String,
    pub review_patch_effect_status: String,
    pub overclaim_guard_status: String,
    pub workspace_truth_status: String,
    pub safety_boolean_audit_status: String,
    pub final_gate_status: String,
    pub warnings: Vec<String>,
    pub next_actions: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerPaperLifecycleClosurePanel {
    pub panel_id: String,
    pub lifecycle_closure_status: String,
    pub transition_coverage_status: String,
    pub gate_completeness_status: String,
    pub trace_closure_status: String,
    pub stability_closure_status: String,
    pub risk_veto_status: String,
    pub lower_confidence_status: String,
    pub readiness_gate_status: String,
    pub workspace_truth_status: String,
    pub warnings: Vec<String>,
    pub next_actions: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint105VerificationPatchClosureStorageReport {
    pub report_id: String,
    pub output_dir: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint105VerificationPatchClosureBundle {
    pub verification_finding_closure_report: VerificationFindingClosureReport,
    pub review_patch_effect_report: ReviewPatchEffectReport,
    pub overclaim_regression_guard_report: OverclaimRegressionGuardReport,
    pub workspace_attempt_truth_hardening_report: WorkspaceAttemptTruthHardeningReport,
    pub safety_boolean_coverage_audit_report: SafetyBooleanCoverageAuditReport,
    pub paper_rejected_transition_audit_report: PaperRejectedTransitionAuditReport,
    pub risk_governor_required_transition_audit_report: RiskGovernorRequiredTransitionAuditReport,
    pub missing_artifact_finding_policy_report: MissingArtifactFindingPolicyReport,
    pub final_verification_gate_v2: FinalVerificationGateV2,
    pub dual_agent_review_loop_v2_report: DualAgentReviewLoopV2Report,
    pub paper_lifecycle_warning_closure_report: PaperLifecycleWarningClosureReport,
    pub paper_candidate_transition_coverage_report: PaperCandidateTransitionCoverageReport,
    pub paper_candidate_gate_completeness_report: PaperCandidateGateCompletenessReport,
    pub paper_candidate_evidence_depth_closure_report: PaperCandidateEvidenceDepthClosureReport,
    pub paper_candidate_trace_closure_report: PaperCandidateTraceClosureReport,
    pub paper_candidate_stability_closure_report: PaperCandidateStabilityClosureReport,
    pub risk_governor_batch_veto_warning_closure_report: RiskGovernorBatchVetoWarningClosureReport,
    pub risk_governor_transition_completeness_report: RiskGovernorTransitionCompletenessReport,
    pub risk_governor_no_bypass_audit_v2: RiskGovernorNoBypassAuditV2,
    pub lower_confidence_carry_forward_closure_report: LowerConfidenceCarryForwardClosureReport,
    pub wonyotti_carry_forward_closure_report: WonyottiCarryForwardClosureReport,
    pub larry_williams_carry_forward_closure_report: LarryWilliamsCarryForwardClosureReport,
    pub arthur_hayes_carry_forward_closure_report: ArthurHayesCarryForwardClosureReport,
    pub paper_lifecycle_readiness_gate_v2: PaperLifecycleReadinessGateV2,
    pub paper_candidate_batch_replay_v2_plan: PaperCandidateBatchReplayV2Plan,
    pub paper_candidate_batch_replay_v2_report: PaperCandidateBatchReplayV2Report,
    pub workspace_acceptance_truth_recovery_plan_v6: WorkspaceAcceptanceTruthRecoveryPlanV6,
    pub workspace_acceptance_attempt_v21: WorkspaceAcceptanceAttemptV21,
    pub workspace_compile_cost_diagnosis_v2: WorkspaceCompileCostDiagnosisV2,
    pub focused_vs_full_gate_bridge_v2: FocusedVsFullGateBridgeV2,
    pub safety_coverage_preservation_report_v21: SafetyCoveragePreservationReportV21,
    pub control_tower_verification_patch_closure_panel: ControlTowerVerificationPatchClosurePanel,
    pub control_tower_paper_lifecycle_closure_panel: ControlTowerPaperLifecycleClosurePanel,
    pub storage_report: Sprint105VerificationPatchClosureStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl Sprint105VerificationPatchClosureBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            ("## 1. Sprint summary", format!("- Implemented Sprint 105 verification patch closure, paper lifecycle warning closure, and workspace truth recovery v6.\n- closure_status={} readiness_status={}.", self.verification_finding_closure_report.closure_status, self.paper_lifecycle_readiness_gate_v2.gate_status)),
            ("## 2. Why Sprint 105 was needed", "- Sprint 104 proved the workflow and lifecycle surfaces, but its review patches had to become explicit closure artifacts and warning-backed lifecycle states still needed a conservative closure layer.".to_string()),
            ("## 3. Files added", "- Added Sprint 105 league layer, CLI commands, docs, examples, fixtures, focused tests, and support wiring.".to_string()),
            ("## 4. Files changed", "- Extended the existing export, CLI, and test-support surfaces on top of Sprint 104.".to_string()),
            ("## 5. Verification finding closure", format!("- Status: {}.\n- blocking_findings_closed={} remaining_findings={}.", self.verification_finding_closure_report.closure_status, self.verification_finding_closure_report.blocking_findings_closed, self.verification_finding_closure_report.remaining_findings)),
            ("## 6. Review patch effect", format!("- Status: {}.\n- overclaim_patch_detected={} workspace_attempt_patch_detected={}.", self.review_patch_effect_report.effect_status, self.review_patch_effect_report.overclaim_patch_detected, self.review_patch_effect_report.workspace_attempt_patch_detected)),
            ("## 7. Overclaim regression guard", format!("- Status: {}.\n- full_acceptance_requires_finished_and_passed={} regression_detected={}.", self.overclaim_regression_guard_report.guard_status, self.overclaim_regression_guard_report.full_acceptance_requires_finished_and_passed, self.overclaim_regression_guard_report.regression_detected)),
            ("## 8. Workspace attempt truth hardening", format!("- Status: {}.\n- attempts_not_run_count={} stopped_due_to_long_compile_count={}.", self.workspace_attempt_truth_hardening_report.truth_hardening_status, self.workspace_attempt_truth_hardening_report.attempts_not_run_count, self.workspace_attempt_truth_hardening_report.stopped_due_to_long_compile_count)),
            ("## 9. Safety boolean coverage audit", format!("- Status: {}.\n- actual_guard_booleans_count={} missing_guard_booleans={}.", self.safety_boolean_coverage_audit_report.audit_status, self.safety_boolean_coverage_audit_report.actual_guard_booleans_count, self.safety_boolean_coverage_audit_report.missing_guard_booleans.join(", "))),
            ("## 10. PaperRejected transition audit", format!("- Status: {}.\n- paper_rejected_reachable={} archive_transition_present={}.", self.paper_rejected_transition_audit_report.audit_status, self.paper_rejected_transition_audit_report.paper_rejected_reachable, self.paper_rejected_transition_audit_report.archive_transition_present)),
            ("## 11. Risk Governor required transition audit", format!("- Status: {}.\n- paper_rejected_requires_risk={} cooldown_requires_risk={}.", self.risk_governor_required_transition_audit_report.audit_status, self.risk_governor_required_transition_audit_report.paper_rejected_requires_risk, self.risk_governor_required_transition_audit_report.cooldown_requires_risk)),
            ("## 12. Missing artifact finding policy", format!("- Status: {}.\n- missing_docs_become_findings={} missing_tests_become_findings={}.", self.missing_artifact_finding_policy_report.policy_status, self.missing_artifact_finding_policy_report.missing_docs_become_findings, self.missing_artifact_finding_policy_report.missing_tests_become_findings)),
            ("## 13. Final verification gate v2", format!("- Status: {}.\n- full_workspace_accepted={} blocking_findings_remaining={}.", self.final_verification_gate_v2.gate_status, self.final_verification_gate_v2.full_workspace_accepted, self.final_verification_gate_v2.blocking_findings_remaining)),
            ("## 14. Dual-agent review loop v2", format!("- Status: {}.\n- findings_closed={} findings_remaining={}.", self.dual_agent_review_loop_v2_report.review_loop_status, self.dual_agent_review_loop_v2_report.findings_closed, self.dual_agent_review_loop_v2_report.findings_remaining)),
            ("## 15. Paper lifecycle warning closure", format!("- Status: {}.\n- transition_warning_count={} evidence_warning_count={}.", self.paper_lifecycle_warning_closure_report.closure_status, self.paper_lifecycle_warning_closure_report.transition_warning_count, self.paper_lifecycle_warning_closure_report.evidence_warning_count)),
            ("## 16. Candidate transition coverage", format!("- Status: {}.\n- reachable_or_explained_states={}/{}.", self.paper_candidate_transition_coverage_report.coverage_status, self.paper_candidate_transition_coverage_report.reachable_or_explained_states, self.paper_candidate_transition_coverage_report.total_states)),
            ("## 17. Candidate gate completeness", format!("- Status: {}.\n- missing_gate_count={}.", self.paper_candidate_gate_completeness_report.completeness_status, self.paper_candidate_gate_completeness_report.missing_gate_count)),
            ("## 18. Candidate evidence / trace / stability closure", format!("- evidence_status={} trace_status={} stability_status={}.", self.paper_candidate_evidence_depth_closure_report.closure_status, self.paper_candidate_trace_closure_report.closure_status, self.paper_candidate_stability_closure_report.closure_status)),
            ("## 19. Risk Governor batch veto warning closure", format!("- Status: {}.\n- warning_count_remaining={} bypass_attempt_count={}.", self.risk_governor_batch_veto_warning_closure_report.closure_status, self.risk_governor_batch_veto_warning_closure_report.warning_count_remaining, self.risk_governor_batch_veto_warning_closure_report.bypass_attempt_count)),
            ("## 20. Risk Governor transition completeness", format!("- Status: {}.\n- missing_transition_count={}.", self.risk_governor_transition_completeness_report.completeness_status, self.risk_governor_transition_completeness_report.missing_transition_count)),
            ("## 21. Risk Governor no-bypass audit v2", format!("- Status: {}.\n- bypass_transition_count={}.", self.risk_governor_no_bypass_audit_v2.audit_status, self.risk_governor_no_bypass_audit_v2.bypass_transition_count)),
            ("## 22. Lower-confidence carry-forward closure", format!("- Status: {}.\n- silent_upgrade_count={}.", self.lower_confidence_carry_forward_closure_report.closure_status, self.lower_confidence_carry_forward_closure_report.silent_upgrade_count)),
            ("## 23. Wonyotti / Larry / Arthur carry-forward closure", format!("- wonyotti_status={} larry_status={} arthur_status={}.", self.wonyotti_carry_forward_closure_report.closure_status, self.larry_williams_carry_forward_closure_report.closure_status, self.arthur_hayes_carry_forward_closure_report.closure_status)),
            ("## 24. Paper lifecycle readiness gate v2", format!("- Status: {}.\n- lifecycle_ready={} live_lifecycle_allowed={}.", self.paper_lifecycle_readiness_gate_v2.gate_status, self.paper_lifecycle_readiness_gate_v2.lifecycle_ready, self.paper_lifecycle_readiness_gate_v2.live_lifecycle_allowed)),
            ("## 25. Paper candidate batch replay v2", format!("- Status: {}.\n- replay_count={} paper_approved_count={} paper_rejected_count={}.", self.paper_candidate_batch_replay_v2_report.report_status, self.paper_candidate_batch_replay_v2_report.replay_count, self.paper_candidate_batch_replay_v2_report.paper_approved_count, self.paper_candidate_batch_replay_v2_report.paper_rejected_count)),
            ("## 26. Workspace acceptance truth recovery v6", format!("- Status: {}.\n- can_claim_full_acceptance={}.", self.workspace_acceptance_truth_recovery_plan_v6.recovery_status, self.workspace_acceptance_truth_recovery_plan_v6.can_claim_full_acceptance)),
            ("## 27. Workspace compile-cost diagnosis v2", format!("- Status: {}.\n- no_run_long_running_observed={} full_run_long_running_observed={}.", self.workspace_compile_cost_diagnosis_v2.diagnosis_status, self.workspace_compile_cost_diagnosis_v2.no_run_long_running_observed, self.workspace_compile_cost_diagnosis_v2.full_run_long_running_observed)),
            ("## 28. Focused-vs-full gate bridge v2", format!("- Status: {}.\n- focused_pass_recorded={} full_workspace_open={}.", self.focused_vs_full_gate_bridge_v2.bridge_status, self.focused_vs_full_gate_bridge_v2.focused_pass_recorded, self.focused_vs_full_gate_bridge_v2.full_workspace_open)),
            ("## 29. Safety coverage preservation v21", format!("- Status: {}.\n- overclaim_guard_present={} paper_rejected_transition_guard_present={}.", self.safety_coverage_preservation_report_v21.safety_status, self.safety_coverage_preservation_report_v21.overclaim_guard_present, self.safety_coverage_preservation_report_v21.paper_rejected_transition_guard_present)),
            ("## 30. Control Tower verification patch closure panel", format!("- Status: {}.", self.control_tower_verification_patch_closure_panel.final_gate_status)),
            ("## 31. Control Tower paper lifecycle closure panel", format!("- Status: {}.", self.control_tower_paper_lifecycle_closure_panel.readiness_gate_status)),
            ("## 32. Output bundle", format!("- Output files: {}.", self.storage_report.file_count)),
            ("## 33. CLI and examples", format!("- sprint105_cli_commands={} example_configs={}.", SPRINT105_CLI_COMMANDS.len(), SPRINT105_EXAMPLES.len())),
            ("## 34. Tests added", format!("- focused_tests_added={}.", SPRINT105_TESTS.len())),
            ("## 35. Test results", "- Focused Sprint 105 validation and honest workspace attempts are reported separately from verification/acceptance semantics.".to_string()),
            ("## 36. Verification patch closure status", format!("- {}.", self.verification_finding_closure_report.closure_status)),
            ("## 37. Paper lifecycle closure status", format!("- {}.", self.paper_lifecycle_warning_closure_report.closure_status)),
            ("## 38. Risk Governor transition status", format!("- {}.", self.risk_governor_transition_completeness_report.completeness_status)),
            ("## 39. Lower-confidence carry-forward status", format!("- {}.", self.lower_confidence_carry_forward_closure_report.closure_status)),
            ("## 40. Runtime deferred status", "- RuntimeStillDeferred\n- TrainingStillDeferred\n- LiveInferenceForbidden\n- LiveTradingStillForbidden\n- NoRuntimeLlmLiveDecisionPath\n- KeepResearchOnly\n- KeepPaperOnly".to_string()),
            ("## 41. Workspace acceptance truth status", format!("- {}.", self.workspace_acceptance_attempt_v21.attempt_status)),
            ("## 42. Safety coverage status", format!("- {}.", self.safety_coverage_preservation_report_v21.safety_status)),
            ("## 43. Risk review", "- Verification remains distinct from cargo acceptance. Paper lifecycle remains non-order, non-execution. Chairman/member/owner still cannot bypass Risk Governor.".to_string()),
            ("## 44. Deferred items", "- Full workspace acceptance, runtime inference, model training, live inference, live trading, broker/order/account, runtime LLM live decision path, Mamba runtime, Gated runtime, dashboard serve, browser execution, and 18-live-agent activation remain deferred or forbidden.".to_string()),
            ("## 45. Next gstack sprint recommendation", "- Keep the closure layer conservative: preserve explicit lower-confidence warnings, continue workspace truth recovery honestly, and avoid treating verification or focused passes as full acceptance.".to_string()),
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
            &output_dir.join("verification_finding_closure.txt"),
            &self.verification_finding_closure_report,
        )?;
        write_json_file(
            &output_dir.join("review_patch_effect.txt"),
            &self.review_patch_effect_report,
        )?;
        write_json_file(
            &output_dir.join("overclaim_regression_guard.txt"),
            &self.overclaim_regression_guard_report,
        )?;
        write_json_file(
            &output_dir.join("workspace_attempt_truth_hardening.txt"),
            &self.workspace_attempt_truth_hardening_report,
        )?;
        write_json_file(
            &output_dir.join("safety_boolean_coverage_audit.txt"),
            &self.safety_boolean_coverage_audit_report,
        )?;
        write_json_file(
            &output_dir.join("paper_rejected_transition_audit.txt"),
            &self.paper_rejected_transition_audit_report,
        )?;
        write_json_file(
            &output_dir.join("risk_governor_required_transition_audit.txt"),
            &self.risk_governor_required_transition_audit_report,
        )?;
        write_json_file(
            &output_dir.join("missing_artifact_finding_policy.txt"),
            &self.missing_artifact_finding_policy_report,
        )?;
        write_json_file(
            &output_dir.join("final_verification_gate_v2.txt"),
            &self.final_verification_gate_v2,
        )?;
        write_json_file(
            &output_dir.join("dual_agent_review_loop_v2.txt"),
            &self.dual_agent_review_loop_v2_report,
        )?;
        write_json_file(
            &output_dir.join("paper_lifecycle_warning_closure.txt"),
            &self.paper_lifecycle_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_transition_coverage.txt"),
            &self.paper_candidate_transition_coverage_report,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_gate_completeness.txt"),
            &self.paper_candidate_gate_completeness_report,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_evidence_depth_closure.txt"),
            &self.paper_candidate_evidence_depth_closure_report,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_trace_closure.txt"),
            &self.paper_candidate_trace_closure_report,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_stability_closure.txt"),
            &self.paper_candidate_stability_closure_report,
        )?;
        write_json_file(
            &output_dir.join("risk_governor_batch_veto_warning_closure.txt"),
            &self.risk_governor_batch_veto_warning_closure_report,
        )?;
        write_json_file(
            &output_dir.join("risk_governor_transition_completeness.txt"),
            &self.risk_governor_transition_completeness_report,
        )?;
        write_json_file(
            &output_dir.join("risk_governor_no_bypass_audit_v2.txt"),
            &self.risk_governor_no_bypass_audit_v2,
        )?;
        write_json_file(
            &output_dir.join("lower_confidence_carry_forward_closure.txt"),
            &self.lower_confidence_carry_forward_closure_report,
        )?;
        write_json_file(
            &output_dir.join("wonyotti_carry_forward_closure.txt"),
            &self.wonyotti_carry_forward_closure_report,
        )?;
        write_json_file(
            &output_dir.join("larry_williams_carry_forward_closure.txt"),
            &self.larry_williams_carry_forward_closure_report,
        )?;
        write_json_file(
            &output_dir.join("arthur_hayes_carry_forward_closure.txt"),
            &self.arthur_hayes_carry_forward_closure_report,
        )?;
        write_json_file(
            &output_dir.join("paper_lifecycle_readiness_gate_v2.txt"),
            &self.paper_lifecycle_readiness_gate_v2,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_batch_replay_v2_plan.txt"),
            &self.paper_candidate_batch_replay_v2_plan,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_batch_replay_v2.txt"),
            &self.paper_candidate_batch_replay_v2_report,
        )?;
        write_json_file(
            &output_dir.join("workspace_acceptance_truth_recovery_plan_v6.txt"),
            &self.workspace_acceptance_truth_recovery_plan_v6,
        )?;
        write_json_file(
            &output_dir.join("workspace_acceptance_attempt_v21.txt"),
            &self.workspace_acceptance_attempt_v21,
        )?;
        write_json_file(
            &output_dir.join("workspace_compile_cost_diagnosis_v2.txt"),
            &self.workspace_compile_cost_diagnosis_v2,
        )?;
        write_json_file(
            &output_dir.join("focused_vs_full_gate_bridge_v2.txt"),
            &self.focused_vs_full_gate_bridge_v2,
        )?;
        write_json_file(
            &output_dir.join("safety_coverage_preservation_v21.txt"),
            &self.safety_coverage_preservation_report_v21,
        )?;
        write_json_file(
            &output_dir.join("control_tower_verification_patch_closure_panel.txt"),
            &self.control_tower_verification_patch_closure_panel,
        )?;
        write_json_file(
            &output_dir.join("control_tower_paper_lifecycle_closure_panel.txt"),
            &self.control_tower_paper_lifecycle_closure_panel,
        )?;
        let files = vec![
            "verification_finding_closure.txt",
            "review_patch_effect.txt",
            "overclaim_regression_guard.txt",
            "workspace_attempt_truth_hardening.txt",
            "safety_boolean_coverage_audit.txt",
            "paper_rejected_transition_audit.txt",
            "risk_governor_required_transition_audit.txt",
            "missing_artifact_finding_policy.txt",
            "final_verification_gate_v2.txt",
            "dual_agent_review_loop_v2.txt",
            "paper_lifecycle_warning_closure.txt",
            "paper_candidate_transition_coverage.txt",
            "paper_candidate_gate_completeness.txt",
            "paper_candidate_evidence_depth_closure.txt",
            "paper_candidate_trace_closure.txt",
            "paper_candidate_stability_closure.txt",
            "risk_governor_batch_veto_warning_closure.txt",
            "risk_governor_transition_completeness.txt",
            "risk_governor_no_bypass_audit_v2.txt",
            "lower_confidence_carry_forward_closure.txt",
            "wonyotti_carry_forward_closure.txt",
            "larry_williams_carry_forward_closure.txt",
            "arthur_hayes_carry_forward_closure.txt",
            "paper_lifecycle_readiness_gate_v2.txt",
            "paper_candidate_batch_replay_v2_plan.txt",
            "paper_candidate_batch_replay_v2.txt",
            "workspace_acceptance_truth_recovery_plan_v6.txt",
            "workspace_acceptance_attempt_v21.txt",
            "workspace_compile_cost_diagnosis_v2.txt",
            "focused_vs_full_gate_bridge_v2.txt",
            "safety_coverage_preservation_v21.txt",
            "control_tower_verification_patch_closure_panel.txt",
            "control_tower_paper_lifecycle_closure_panel.txt",
            "storage_report.txt",
            "summary.txt",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        self.storage_report = Sprint105VerificationPatchClosureStorageReport {
            report_id: "sprint105-verification-patch-closure-storage-report".to_string(),
            output_dir: output_dir.display().to_string(),
            file_count: files.len(),
            files,
            reason_codes: deferred_reason_codes(&[]),
        };
        self.final_summary = self.build_final_summary();
        write_json_file(&output_dir.join("storage_report.txt"), &self.storage_report)?;
        write_text_file(&output_dir.join("summary.txt"), &self.final_summary)?;
        Ok(output_dir.to_path_buf())
    }
}

#[derive(Default)]
pub struct Sprint105VerificationPatchClosureRunner;

impl Sprint105VerificationPatchClosureRunner {
    pub fn run(
        &self,
        config: &Sprint105VerificationPatchClosureConfig,
    ) -> Result<Sprint105VerificationPatchClosureBundle, String> {
        config.validate()?;
        let sprint104 = load_sprint104_bundle(config)?;
        let review_patch_summary = load_review_patch_summary(config)?;
        let verification_findings = load_first_json::<Vec<VerificationFinding>>(
            config.verification_finding_paths.as_ref(),
        )?
        .unwrap_or_else(|| sprint104.verification_findings.clone());
        let paper_lifecycle = load_first_json::<PaperCandidateLifecycleStateMachine>(
            config.paper_lifecycle_paths.as_ref(),
        )?
        .unwrap_or_else(|| sprint104.paper_candidate_lifecycle_state_machine.clone());
        let risk_governor_batch = load_first_json::<RiskGovernorBatchVetoReport>(
            config.risk_governor_batch_paths.as_ref(),
        )?
        .unwrap_or_else(|| sprint104.risk_governor_batch_veto_report.clone());
        let lower_confidence = load_first_json::<LowerConfidenceCarryForwardPolicy>(
            config.lower_confidence_paths.as_ref(),
        )?
        .unwrap_or_else(|| sprint104.lower_confidence_carry_forward_policy.clone());
        let workspace_truth = load_workspace_truth_snapshot(config, &sprint104)?;

        let verification_finding_closure_report =
            build_verification_finding_closure_report(&verification_findings);
        let review_patch_effect_report = build_review_patch_effect_report(&review_patch_summary);
        let overclaim_regression_guard_report =
            build_overclaim_regression_guard_report(&workspace_truth);
        let workspace_attempt_truth_hardening_report =
            build_workspace_attempt_truth_hardening_report(&workspace_truth, &review_patch_summary);
        let safety_boolean_coverage_audit_report = build_safety_boolean_coverage_audit_report(
            config,
            &sprint104.safety_coverage_preservation_report_v20,
        );
        let paper_rejected_transition_audit_report =
            build_paper_rejected_transition_audit_report(&paper_lifecycle);
        let risk_governor_required_transition_audit_report =
            build_risk_governor_required_transition_audit_report(
                &paper_lifecycle,
                &risk_governor_batch,
            );
        let missing_artifact_finding_policy_report =
            build_missing_artifact_finding_policy_report(&verification_findings);
        let final_verification_gate_v2 = build_final_verification_gate_v2(
            &sprint104.final_verification_gate,
            &verification_finding_closure_report,
            &overclaim_regression_guard_report,
            &safety_boolean_coverage_audit_report,
            &paper_rejected_transition_audit_report,
            &risk_governor_required_transition_audit_report,
            &missing_artifact_finding_policy_report,
            &workspace_truth,
        );
        let dual_agent_review_loop_v2_report = build_dual_agent_review_loop_v2_report(
            &sprint104.dual_agent_review_loop_report,
            &verification_finding_closure_report,
            &review_patch_effect_report,
            &final_verification_gate_v2,
        );
        let paper_candidate_transition_coverage_report =
            build_paper_candidate_transition_coverage_report(&paper_lifecycle);
        let paper_candidate_gate_completeness_report =
            build_paper_candidate_gate_completeness_report(
                &sprint104.paper_candidate_promotion_gate,
                &sprint104.paper_candidate_rejection_gate,
                &sprint104.paper_candidate_watchlist_gate,
                &sprint104.paper_candidate_no_trade_gate,
                &sprint104.paper_candidate_risk_denied_gate,
            );
        let paper_candidate_evidence_depth_closure_report =
            build_paper_candidate_evidence_depth_closure_report(
                &sprint104.paper_candidate_evidence_depth_report,
            );
        let paper_candidate_trace_closure_report = build_paper_candidate_trace_closure_report(
            &sprint104.paper_candidate_decision_trace_report,
        );
        let paper_candidate_stability_closure_report =
            build_paper_candidate_stability_closure_report(
                &sprint104.paper_candidate_stability_report,
            );
        let paper_lifecycle_warning_closure_report = build_paper_lifecycle_warning_closure_report(
            &paper_lifecycle,
            &paper_candidate_transition_coverage_report,
            &paper_candidate_gate_completeness_report,
            &paper_candidate_evidence_depth_closure_report,
            &paper_candidate_trace_closure_report,
        );
        let risk_governor_batch_veto_warning_closure_report =
            build_risk_governor_batch_veto_warning_closure_report(&risk_governor_batch);
        let risk_governor_transition_completeness_report =
            build_risk_governor_transition_completeness_report(
                &risk_governor_required_transition_audit_report,
            );
        let risk_governor_no_bypass_audit_v2 =
            build_risk_governor_no_bypass_audit_v2(&risk_governor_batch);
        let lower_confidence_carry_forward_closure_report =
            build_lower_confidence_carry_forward_closure_report(
                &lower_confidence,
                &sprint104.wonyotti_carry_forward_review,
                &sprint104.larry_williams_carry_forward_review,
                &sprint104.arthur_hayes_carry_forward_review,
            );
        let wonyotti_carry_forward_closure_report =
            build_wonyotti_carry_forward_closure_report(&sprint104.wonyotti_carry_forward_review);
        let larry_williams_carry_forward_closure_report =
            build_larry_williams_carry_forward_closure_report(
                &sprint104.larry_williams_carry_forward_review,
            );
        let arthur_hayes_carry_forward_closure_report =
            build_arthur_hayes_carry_forward_closure_report(
                &sprint104.arthur_hayes_carry_forward_review,
            );
        let paper_lifecycle_readiness_gate_v2 = build_paper_lifecycle_readiness_gate_v2(
            &paper_candidate_transition_coverage_report,
            &paper_candidate_gate_completeness_report,
            &paper_candidate_evidence_depth_closure_report,
            &paper_candidate_trace_closure_report,
            &paper_candidate_stability_closure_report,
            &risk_governor_batch_veto_warning_closure_report,
            &lower_confidence_carry_forward_closure_report,
        );
        let paper_candidate_batch_replay_v2_plan = build_paper_candidate_batch_replay_v2_plan(
            &sprint104.paper_rotation_batch_replay_report,
        );
        let paper_candidate_batch_replay_v2_report = build_paper_candidate_batch_replay_v2_report(
            &sprint104.paper_rotation_batch_replay_report,
            &paper_lifecycle,
        );
        let workspace_compile_cost_diagnosis_v2 =
            build_workspace_compile_cost_diagnosis_v2(&review_patch_summary);
        let focused_vs_full_gate_bridge_v2 = build_focused_vs_full_gate_bridge_v2(
            &sprint104.final_verification_gate,
            &workspace_truth,
        );
        let workspace_acceptance_truth_recovery_plan_v6 =
            build_workspace_acceptance_truth_recovery_plan_v6(
                &sprint104.workspace_acceptance_truth_closure_plan_v5,
                &workspace_compile_cost_diagnosis_v2,
                &focused_vs_full_gate_bridge_v2,
                &workspace_truth,
            );
        let workspace_acceptance_attempt_v21 =
            build_workspace_acceptance_attempt_v21(config, &workspace_truth)?;
        let safety_coverage_preservation_report_v21 = build_safety_coverage_preservation_report_v21(
            config,
            &sprint104.safety_coverage_preservation_report_v20,
            &overclaim_regression_guard_report,
            &paper_rejected_transition_audit_report,
            &risk_governor_required_transition_audit_report,
            &missing_artifact_finding_policy_report,
        );
        let control_tower_verification_patch_closure_panel =
            build_control_tower_verification_patch_closure_panel(
                &verification_finding_closure_report,
                &review_patch_effect_report,
                &overclaim_regression_guard_report,
                &workspace_attempt_truth_hardening_report,
                &safety_boolean_coverage_audit_report,
                &final_verification_gate_v2,
            );
        let control_tower_paper_lifecycle_closure_panel =
            build_control_tower_paper_lifecycle_closure_panel(
                &paper_lifecycle_warning_closure_report,
                &paper_candidate_transition_coverage_report,
                &paper_candidate_gate_completeness_report,
                &paper_candidate_trace_closure_report,
                &paper_candidate_stability_closure_report,
                &risk_governor_batch_veto_warning_closure_report,
                &lower_confidence_carry_forward_closure_report,
                &paper_lifecycle_readiness_gate_v2,
                &workspace_truth,
            );

        let mut bundle = Sprint105VerificationPatchClosureBundle {
            verification_finding_closure_report,
            review_patch_effect_report,
            overclaim_regression_guard_report,
            workspace_attempt_truth_hardening_report,
            safety_boolean_coverage_audit_report,
            paper_rejected_transition_audit_report,
            risk_governor_required_transition_audit_report,
            missing_artifact_finding_policy_report,
            final_verification_gate_v2,
            dual_agent_review_loop_v2_report,
            paper_lifecycle_warning_closure_report,
            paper_candidate_transition_coverage_report,
            paper_candidate_gate_completeness_report,
            paper_candidate_evidence_depth_closure_report,
            paper_candidate_trace_closure_report,
            paper_candidate_stability_closure_report,
            risk_governor_batch_veto_warning_closure_report,
            risk_governor_transition_completeness_report,
            risk_governor_no_bypass_audit_v2,
            lower_confidence_carry_forward_closure_report,
            wonyotti_carry_forward_closure_report,
            larry_williams_carry_forward_closure_report,
            arthur_hayes_carry_forward_closure_report,
            paper_lifecycle_readiness_gate_v2,
            paper_candidate_batch_replay_v2_plan,
            paper_candidate_batch_replay_v2_report,
            workspace_acceptance_truth_recovery_plan_v6,
            workspace_acceptance_attempt_v21,
            workspace_compile_cost_diagnosis_v2,
            focused_vs_full_gate_bridge_v2,
            safety_coverage_preservation_report_v21,
            control_tower_verification_patch_closure_panel,
            control_tower_paper_lifecycle_closure_panel,
            storage_report: Sprint105VerificationPatchClosureStorageReport {
                report_id: "sprint105-verification-patch-closure-storage-report".to_string(),
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

fn load_sprint104_bundle(
    config: &Sprint105VerificationPatchClosureConfig,
) -> Result<Sprint104DualAgentPaperLifecycleBundle, String> {
    if let Some(paths) = config.sprint104_bundle_paths.as_ref() {
        for path in paths {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            if let Ok(bundle) =
                serde_json::from_str::<Sprint104DualAgentPaperLifecycleBundle>(&text)
            {
                return Ok(bundle);
            }
        }
    }
    let mut sprint104_config = DualAgentWorkflowConfig::default();
    sprint104_config.output_root = config
        .output_dir()
        .join("sprint104_seed")
        .display()
        .to_string();
    Sprint104DualAgentPaperLifecycleRunner::default().run(&sprint104_config)
}

fn load_review_patch_summary(
    config: &Sprint105VerificationPatchClosureConfig,
) -> Result<ReviewPatchSummary, String> {
    Ok(
        load_first_json::<ReviewPatchSummary>(config.review_patch_summary_paths.as_ref())?
            .unwrap_or_default(),
    )
}

fn load_workspace_truth_snapshot(
    config: &Sprint105VerificationPatchClosureConfig,
    sprint104: &Sprint104DualAgentPaperLifecycleBundle,
) -> Result<WorkspaceTruthSnapshot, String> {
    if let Some(value) =
        load_first_json::<serde_json::Value>(config.workspace_truth_paths.as_ref())?
    {
        let no_run_started = value
            .get("no_run_started")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let no_run_finished = value
            .get("no_run_finished")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let no_run_passed = value.get("no_run_passed").and_then(|value| value.as_bool());
        let full_started = value
            .get("full_started")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let full_finished = value
            .get("full_finished")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let full_passed = value.get("full_passed").and_then(|value| value.as_bool());
        let can_claim_full_acceptance = value
            .get("can_claim_full_acceptance")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        return Ok(WorkspaceTruthSnapshot {
            truth_status: value
                .get("truth_status")
                .and_then(|value| value.as_str())
                .unwrap_or("WorkspaceTruthImported")
                .to_string(),
            no_run_started,
            no_run_finished,
            no_run_passed,
            full_started,
            full_finished,
            full_passed,
            can_claim_full_acceptance,
        });
    }
    Ok(WorkspaceTruthSnapshot {
        truth_status: sprint104
            .workspace_acceptance_truth_closure_plan_v5
            .current_truth_status
            .clone(),
        no_run_started: sprint104.workspace_acceptance_attempt_v20.no_run_started,
        no_run_finished: sprint104.workspace_acceptance_attempt_v20.no_run_finished,
        no_run_passed: sprint104.workspace_acceptance_attempt_v20.no_run_passed,
        full_started: sprint104.workspace_acceptance_attempt_v20.full_started,
        full_finished: sprint104.workspace_acceptance_attempt_v20.full_finished,
        full_passed: sprint104.workspace_acceptance_attempt_v20.full_passed,
        can_claim_full_acceptance: sprint104
            .workspace_acceptance_attempt_v20
            .can_claim_full_acceptance,
    })
}

fn build_verification_finding_closure_report(
    findings: &[VerificationFinding],
) -> VerificationFindingClosureReport {
    let remaining_findings = findings
        .iter()
        .filter(|finding| finding.finding_status == VerificationFindingStatus::Open)
        .count();
    let blocking_findings_closed = findings
        .iter()
        .filter(|finding| {
            finding.severity == VerificationFindingSeverity::Blocking
                && finding.finding_status != VerificationFindingStatus::Open
        })
        .count();
    let major_findings_closed = findings
        .iter()
        .filter(|finding| {
            finding.severity == VerificationFindingSeverity::Major
                && finding.finding_status != VerificationFindingStatus::Open
        })
        .count();
    let known_warnings_preserved = findings
        .iter()
        .filter(|finding| {
            finding.finding_status == VerificationFindingStatus::AcceptedAsKnownWarning
        })
        .count();
    let closure_status = if remaining_findings == 0 && known_warnings_preserved > 0 {
        "VerificationFindingsClosedWithWarnings"
    } else if remaining_findings == 0 {
        "VerificationFindingsClosed"
    } else {
        "VerificationFindingsStillOpen"
    };
    VerificationFindingClosureReport {
        report_id: "verification-finding-closure-report".to_string(),
        blocking_findings_closed,
        major_findings_closed,
        known_warnings_preserved,
        remaining_findings,
        closure_status: closure_status.to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_review_patch_effect_report(review: &ReviewPatchSummary) -> ReviewPatchEffectReport {
    let patches_ready = review.overclaim_patch_detected
        && review.workspace_attempt_patch_detected
        && review.safety_boolean_patch_detected
        && review.missing_artifact_policy_patch_detected
        && review.paper_rejected_transition_patch_detected
        && review.risk_transition_patch_detected;
    ReviewPatchEffectReport {
        report_id: "review-patch-effect-report".to_string(),
        overclaim_patch_detected: review.overclaim_patch_detected,
        workspace_attempt_patch_detected: review.workspace_attempt_patch_detected,
        safety_boolean_patch_detected: review.safety_boolean_patch_detected,
        missing_artifact_policy_patch_detected: review.missing_artifact_policy_patch_detected,
        paper_rejected_transition_patch_detected: review.paper_rejected_transition_patch_detected,
        risk_transition_patch_detected: review.risk_transition_patch_detected,
        effect_status: if patches_ready {
            "ReviewPatchEffectsReady"
        } else {
            "ReviewPatchEffectsIncomplete"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_overclaim_regression_guard_report(
    workspace_truth: &WorkspaceTruthSnapshot,
) -> OverclaimRegressionGuardReport {
    let full_acceptance_requires_finished_and_passed = true;
    let unrun_attempt_finished = !workspace_truth.no_run_started && workspace_truth.no_run_finished;
    let focused_pass_is_full_acceptance = false;
    let verification_pass_is_full_acceptance = false;
    let no_run_pass_is_full_acceptance = workspace_truth.no_run_passed == Some(true)
        && workspace_truth.can_claim_full_acceptance
        && !full_workspace_accepted(workspace_truth);
    let regression_detected = (workspace_truth.can_claim_full_acceptance
        && !full_workspace_accepted(workspace_truth))
        || unrun_attempt_finished
        || no_run_pass_is_full_acceptance;
    OverclaimRegressionGuardReport {
        report_id: "overclaim-regression-guard-report".to_string(),
        full_acceptance_requires_finished_and_passed,
        unrun_attempt_finished,
        focused_pass_is_full_acceptance,
        verification_pass_is_full_acceptance,
        no_run_pass_is_full_acceptance,
        regression_detected,
        guard_status: if regression_detected {
            "OverclaimRegressionDetected"
        } else {
            "OverclaimRegressionGuardReady"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_attempt_truth_hardening_report(
    workspace_truth: &WorkspaceTruthSnapshot,
    review: &ReviewPatchSummary,
) -> WorkspaceAttemptTruthHardeningReport {
    let attempts_not_run_count =
        usize::from(!workspace_truth.no_run_started) + usize::from(!workspace_truth.full_started);
    let stopped_due_to_long_compile_count = usize::from(review.no_run_long_running_observed)
        + usize::from(review.full_run_long_running_observed);
    let can_claim_full_acceptance = full_workspace_accepted(workspace_truth);
    WorkspaceAttemptTruthHardeningReport {
        report_id: "workspace-attempt-truth-hardening-report".to_string(),
        attempts_not_run_count,
        stopped_due_to_long_compile_count,
        can_claim_full_acceptance,
        truth_hardening_status: if can_claim_full_acceptance {
            "WorkspaceAttemptTruthClosed"
        } else {
            "WorkspaceAttemptTruthHardenedWithWarnings"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_safety_boolean_coverage_audit_report(
    config: &Sprint105VerificationPatchClosureConfig,
    safety_v20: &SafetyCoveragePreservationReportV20,
) -> SafetyBooleanCoverageAuditReport {
    let guards = [
        (
            "live_trading_guard_present",
            safety_v20.live_trading_guard_present,
        ),
        ("broker_guard_present", safety_v20.broker_guard_present),
        ("order_guard_present", safety_v20.order_guard_present),
        ("account_guard_present", safety_v20.account_guard_present),
        (
            "runtime_llm_guard_present",
            safety_v20.runtime_llm_guard_present,
        ),
        (
            "no_silent_confidence_upgrade_guard_present",
            safety_v20.no_silent_confidence_upgrade_guard_present,
        ),
        (
            "dual_agent_workflow_guard_present",
            safety_v20.dual_agent_workflow_guard_present,
        ),
        (
            "verification_not_acceptance_guard_present",
            safety_v20.verification_not_acceptance_guard_present,
        ),
        (
            "paper_candidate_not_order_guard_present",
            safety_v20.paper_candidate_not_order_guard_present,
        ),
        (
            "paper_candidate_lifecycle_no_live_guard_present",
            safety_v20.paper_candidate_lifecycle_no_live_guard_present,
        ),
    ];
    let actual_guard_booleans_count = guards.iter().filter(|(_, present)| *present).count();
    let missing_guard_booleans = guards
        .iter()
        .filter_map(|(name, present)| (!present).then_some((*name).to_string()))
        .collect::<Vec<_>>();
    let mut guard_mismatch_count = 0usize;
    if !config.preserve_safety_guards || !config.preserve_runtime_deferred {
        guard_mismatch_count += 1;
    }
    if config.preserve_safety_guards
        && (!safety_v20.live_trading_guard_present || !safety_v20.broker_guard_present)
    {
        guard_mismatch_count += 1;
    }
    if config.preserve_dual_agent_separation
        && (!safety_v20.dual_agent_workflow_guard_present
            || !safety_v20.verification_not_acceptance_guard_present)
    {
        guard_mismatch_count += 1;
    }
    SafetyBooleanCoverageAuditReport {
        report_id: "safety-boolean-coverage-audit-report".to_string(),
        actual_guard_booleans_count,
        missing_guard_booleans,
        guard_mismatch_count,
        audit_status: if guard_mismatch_count == 0 {
            "SafetyBooleanCoverageVerified"
        } else {
            "SafetyBooleanCoverageMissingGuards"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_rejected_transition_audit_report(
    lifecycle: &PaperCandidateLifecycleStateMachine,
) -> PaperRejectedTransitionAuditReport {
    let paper_rejected_reachable = lifecycle
        .allowed_transitions
        .iter()
        .any(|transition| transition == "DebateOpen->PaperRejected");
    let archive_transition_present = lifecycle
        .allowed_transitions
        .iter()
        .any(|transition| transition == "PaperRejected->ArchivedPaperOnly");
    let paper_rejected_can_go_live = lifecycle
        .allowed_transitions
        .iter()
        .any(|transition| transition.contains("PaperRejected->Live"));
    let paper_rejected_can_become_order = lifecycle
        .allowed_transitions
        .iter()
        .any(|transition| transition.contains("PaperRejected->BrokerOrder"));
    PaperRejectedTransitionAuditReport {
        report_id: "paper-rejected-transition-audit-report".to_string(),
        paper_rejected_state_present: true,
        paper_rejected_reachable,
        paper_rejected_can_go_live,
        paper_rejected_can_become_order,
        archive_transition_present,
        audit_status: if paper_rejected_reachable
            && archive_transition_present
            && !paper_rejected_can_go_live
            && !paper_rejected_can_become_order
        {
            "PaperRejectedTransitionAudited"
        } else {
            "PaperRejectedTransitionWarning"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_risk_governor_required_transition_audit_report(
    lifecycle: &PaperCandidateLifecycleStateMachine,
    risk_batch: &RiskGovernorBatchVetoReport,
) -> RiskGovernorRequiredTransitionAuditReport {
    let required = &lifecycle.risk_governor_required_transitions;
    let paper_approved_requires_risk = required
        .iter()
        .any(|transition| transition == "DebateOpen->PaperApproved");
    let paper_rejected_requires_risk = required
        .iter()
        .any(|transition| transition == "DebateOpen->PaperRejected");
    let risk_denied_requires_risk = required
        .iter()
        .any(|transition| transition == "DebateOpen->RiskDenied");
    let no_trade_requires_risk = required
        .iter()
        .any(|transition| transition == "DebateOpen->NoTrade");
    let cooldown_requires_risk = required
        .iter()
        .any(|transition| transition == "PaperApproved->Cooldown")
        || required
            .iter()
            .any(|transition| transition == "NoTrade->Cooldown")
        || risk_batch.cooldown_count == 0;
    let live_transition_present = lifecycle
        .allowed_transitions
        .iter()
        .chain(lifecycle.risk_governor_required_transitions.iter())
        .any(|transition| transition.contains("Live"));
    let bypass_transition_count = lifecycle
        .allowed_transitions
        .iter()
        .chain(lifecycle.risk_governor_required_transitions.iter())
        .filter(|transition| transition.to_ascii_lowercase().contains("bypass"))
        .count();
    RiskGovernorRequiredTransitionAuditReport {
        report_id: "risk-governor-required-transition-audit-report".to_string(),
        paper_approved_requires_risk,
        paper_rejected_requires_risk,
        risk_denied_requires_risk,
        no_trade_requires_risk,
        cooldown_requires_risk,
        live_transition_present,
        bypass_transition_count,
        audit_status: if paper_approved_requires_risk
            && paper_rejected_requires_risk
            && risk_denied_requires_risk
            && no_trade_requires_risk
            && cooldown_requires_risk
            && !live_transition_present
            && bypass_transition_count == 0
        {
            "RiskGovernorRequiredTransitionsReady"
        } else {
            "RiskGovernorRequiredTransitionsIncomplete"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_missing_artifact_finding_policy_report(
    findings: &[VerificationFinding],
) -> MissingArtifactFindingPolicyReport {
    let missing_docs_count = SPRINT105_DOCS
        .iter()
        .filter(|path| !project_root().join(path).exists())
        .count();
    let missing_tests_count = SPRINT105_TESTS
        .iter()
        .filter(|path| !project_root().join(path).exists())
        .count();
    let missing_examples_count = SPRINT105_EXAMPLES
        .iter()
        .filter(|path| !project_root().join(path).exists())
        .count();
    let missing_docs_become_findings = findings.iter().any(|finding| {
        finding.description.to_ascii_lowercase().contains("docs")
            || finding.required_fix.to_ascii_lowercase().contains("docs")
    });
    let missing_tests_become_findings = findings.iter().any(|finding| {
        finding.description.to_ascii_lowercase().contains("test")
            || finding.required_fix.to_ascii_lowercase().contains("test")
    });
    let missing_examples_become_findings = findings.iter().any(|finding| {
        finding.description.to_ascii_lowercase().contains("example")
            || finding
                .required_fix
                .to_ascii_lowercase()
                .contains("example")
    });
    let missing_docs_become_findings = missing_docs_count == 0 || missing_docs_become_findings;
    let missing_tests_become_findings = missing_tests_count == 0 || missing_tests_become_findings;
    let missing_examples_become_findings =
        missing_examples_count == 0 || missing_examples_become_findings;
    let silent_success_detected = !missing_docs_become_findings
        || !missing_tests_become_findings
        || !missing_examples_become_findings;
    MissingArtifactFindingPolicyReport {
        report_id: "missing-artifact-finding-policy-report".to_string(),
        missing_docs_become_findings,
        missing_tests_become_findings,
        missing_examples_become_findings,
        silent_success_detected,
        policy_status: if silent_success_detected {
            "MissingArtifactFindingPolicyBlocked"
        } else {
            "MissingArtifactFindingPolicyReady"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_final_verification_gate_v2(
    prior: &FinalVerificationGate,
    finding_closure: &VerificationFindingClosureReport,
    overclaim_guard: &OverclaimRegressionGuardReport,
    safety_audit: &SafetyBooleanCoverageAuditReport,
    paper_rejected_audit: &PaperRejectedTransitionAuditReport,
    risk_transition_audit: &RiskGovernorRequiredTransitionAuditReport,
    missing_artifact_policy: &MissingArtifactFindingPolicyReport,
    workspace_truth: &WorkspaceTruthSnapshot,
) -> FinalVerificationGateV2 {
    let full_workspace_accepted = full_workspace_accepted(workspace_truth);
    let gate_status = if finding_closure.remaining_findings == 0
        && !overclaim_guard.regression_detected
        && safety_audit.guard_mismatch_count == 0
        && paper_rejected_audit.audit_status == "PaperRejectedTransitionAudited"
        && risk_transition_audit.audit_status == "RiskGovernorRequiredTransitionsReady"
        && !missing_artifact_policy.silent_success_detected
    {
        if finding_closure.known_warnings_preserved > 0 || !full_workspace_accepted {
            "FinalVerificationGateV2ReadyWithWarnings"
        } else {
            "FinalVerificationGateV2Ready"
        }
    } else {
        "FinalVerificationGateV2Blocked"
    };
    FinalVerificationGateV2 {
        gate_id: "final-verification-gate-v2".to_string(),
        previous_gate_status: prior.gate_status.clone(),
        finding_closure_status: finding_closure.closure_status.clone(),
        overclaim_guard_status: overclaim_guard.guard_status.clone(),
        safety_boolean_audit_status: safety_audit.audit_status.clone(),
        paper_rejected_transition_status: paper_rejected_audit.audit_status.clone(),
        risk_transition_status: risk_transition_audit.audit_status.clone(),
        missing_artifact_policy_status: missing_artifact_policy.policy_status.clone(),
        workspace_truth_status: workspace_truth.truth_status.clone(),
        blocking_findings_remaining: finding_closure.remaining_findings,
        full_workspace_accepted,
        gate_status: gate_status.to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_dual_agent_review_loop_v2_report(
    prior: &DualAgentReviewLoopReport,
    closure: &VerificationFindingClosureReport,
    review: &ReviewPatchEffectReport,
    gate: &FinalVerificationGateV2,
) -> DualAgentReviewLoopV2Report {
    let patch_iterations = [
        review.overclaim_patch_detected,
        review.workspace_attempt_patch_detected,
        review.safety_boolean_patch_detected,
        review.missing_artifact_policy_patch_detected,
        review.paper_rejected_transition_patch_detected,
        review.risk_transition_patch_detected,
    ]
    .into_iter()
    .filter(|value| *value)
    .count();
    DualAgentReviewLoopV2Report {
        report_id: "dual-agent-review-loop-v2-report".to_string(),
        findings_before_review: prior.findings_open_before,
        findings_closed: closure.blocking_findings_closed + closure.major_findings_closed,
        accepted_known_warnings: closure.known_warnings_preserved,
        findings_remaining: closure.remaining_findings,
        patch_iterations,
        final_gate_status: gate.gate_status.clone(),
        review_loop_status: if closure.remaining_findings == 0 {
            "DualAgentReviewLoopV2Ready"
        } else {
            "DualAgentReviewLoopV2NeedsMorePatch"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_transition_coverage_report(
    lifecycle: &PaperCandidateLifecycleStateMachine,
) -> PaperCandidateTransitionCoverageReport {
    let mut explained = 8usize;
    if lifecycle
        .allowed_transitions
        .iter()
        .any(|transition| transition == "DebateOpen->PaperRejected")
    {
        explained += 1;
    }
    if lifecycle
        .allowed_transitions
        .iter()
        .any(|transition| transition == "PaperRejected->ArchivedPaperOnly")
    {
        explained += 1;
    }
    let unsafe_transition_count = lifecycle
        .allowed_transitions
        .iter()
        .filter(|transition| {
            transition.contains("Live")
                || transition.contains("Order")
                || transition.contains("Broker")
        })
        .count();
    let mut missing_transition_explanations = Vec::new();
    if explained < 10 {
        missing_transition_explanations.push("PaperRejected reachability incomplete".to_string());
    }
    PaperCandidateTransitionCoverageReport {
        report_id: "paper-candidate-transition-coverage-report".to_string(),
        total_states: 10,
        reachable_or_explained_states: explained,
        unsafe_transition_count,
        missing_transition_explanations,
        coverage_status: if explained == 10 && unsafe_transition_count == 0 {
            "PaperCandidateTransitionCoverageReady"
        } else {
            "PaperCandidateTransitionCoverageWarning"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_gate_completeness_report(
    promotion: &PaperCandidatePromotionGate,
    rejection: &PaperCandidateRejectionGate,
    watchlist: &PaperCandidateWatchlistGate,
    no_trade: &PaperCandidateNoTradeGate,
    risk_denied: &PaperCandidateRiskDeniedGate,
) -> PaperCandidateGateCompletenessReport {
    let promotion_gate_present = !promotion.gate_id.is_empty();
    let rejection_gate_present = !rejection.gate_id.is_empty();
    let watchlist_gate_present = !watchlist.gate_id.is_empty();
    let no_trade_gate_present = !no_trade.gate_id.is_empty();
    let risk_denied_gate_present = !risk_denied.gate_id.is_empty();
    let missing_gate_count = [
        promotion_gate_present,
        rejection_gate_present,
        watchlist_gate_present,
        no_trade_gate_present,
        risk_denied_gate_present,
    ]
    .into_iter()
    .filter(|present| !present)
    .count();
    PaperCandidateGateCompletenessReport {
        report_id: "paper-candidate-gate-completeness-report".to_string(),
        promotion_gate_present,
        rejection_gate_present,
        watchlist_gate_present,
        no_trade_gate_present,
        risk_denied_gate_present,
        missing_gate_count,
        completeness_status: if missing_gate_count == 0 {
            "PaperCandidateGatesComplete"
        } else {
            "PaperCandidateGatesIncomplete"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_evidence_depth_closure_report(
    evidence: &PaperCandidateEvidenceDepthReport,
) -> PaperCandidateEvidenceDepthClosureReport {
    PaperCandidateEvidenceDepthClosureReport {
        report_id: "paper-candidate-evidence-depth-closure-report".to_string(),
        candidate_count: evidence.candidate_count,
        candidates_needing_more_evidence: evidence.candidates_needing_more_evidence,
        closed_evidence_gap_count: evidence
            .candidate_count
            .saturating_sub(evidence.candidates_needing_more_evidence),
        closure_status: if evidence.candidates_needing_more_evidence == 0 {
            "PaperCandidateEvidenceDepthClosed"
        } else {
            "PaperCandidateEvidenceDepthClosedWithWarnings"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_trace_closure_report(
    trace: &PaperCandidateDecisionTraceReport,
) -> PaperCandidateTraceClosureReport {
    PaperCandidateTraceClosureReport {
        report_id: "paper-candidate-trace-closure-report".to_string(),
        traces_complete: trace.traces_complete,
        traces_missing_risk_governor: trace.traces_missing_risk_governor,
        broker_execution_allowed_count: trace.broker_execution_allowed_count,
        live_execution_allowed_count: trace.live_execution_allowed_count,
        closure_status: if trace.traces_missing_risk_governor == 0
            && trace.broker_execution_allowed_count == 0
            && trace.live_execution_allowed_count == 0
        {
            "PaperCandidateTraceClosed"
        } else {
            "PaperCandidateTraceClosedWithWarnings"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_stability_closure_report(
    stability: &PaperCandidateStabilityReport,
) -> PaperCandidateStabilityClosureReport {
    PaperCandidateStabilityClosureReport {
        report_id: "paper-candidate-stability-closure-report".to_string(),
        stable_candidates: stability.stable_candidates,
        unstable_candidates: stability.unstable_candidates,
        instability_reasons: stability.instability_reasons.clone(),
        closure_status: if stability.unstable_candidates == 0 {
            "PaperCandidateStabilityClosed"
        } else {
            "PaperCandidateStabilityClosedWithWarnings"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_lifecycle_warning_closure_report(
    lifecycle: &PaperCandidateLifecycleStateMachine,
    transitions: &PaperCandidateTransitionCoverageReport,
    gates: &PaperCandidateGateCompletenessReport,
    evidence: &PaperCandidateEvidenceDepthClosureReport,
    trace: &PaperCandidateTraceClosureReport,
) -> PaperLifecycleWarningClosureReport {
    let transition_warning_count =
        usize::from(transitions.coverage_status != "PaperCandidateTransitionCoverageReady")
            + transitions.unsafe_transition_count;
    let gate_warning_count = gates.missing_gate_count;
    let evidence_warning_count = evidence.candidates_needing_more_evidence;
    let trace_warning_count = trace.traces_missing_risk_governor;
    let total = transition_warning_count
        + gate_warning_count
        + evidence_warning_count
        + trace_warning_count;
    let closure_status = if total == 0 {
        "PaperLifecycleWarningsClosed"
    } else {
        "PaperLifecycleStillWarningBacked"
    };
    PaperLifecycleWarningClosureReport {
        report_id: "paper-lifecycle-warning-closure-report".to_string(),
        previous_lifecycle_status: lifecycle.state_machine_status.clone(),
        transition_warning_count,
        gate_warning_count,
        evidence_warning_count,
        trace_warning_count,
        closure_status: closure_status.to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_risk_governor_batch_veto_warning_closure_report(
    batch: &RiskGovernorBatchVetoReport,
) -> RiskGovernorBatchVetoWarningClosureReport {
    let warning_count_remaining = usize::from(batch.broker_execution_allowed_count > 0)
        + usize::from(batch.live_execution_allowed_count > 0)
        + usize::from(batch.bypass_attempt_count > 0);
    RiskGovernorBatchVetoWarningClosureReport {
        report_id: "risk-governor-batch-veto-warning-closure-report".to_string(),
        previous_status: batch.veto_status.clone(),
        warning_count_remaining,
        bypass_attempt_count: batch.bypass_attempt_count,
        closure_status: if warning_count_remaining == 0 {
            "RiskGovernorBatchVetoWarningsClosed"
        } else {
            "RiskGovernorBatchVetoWarningsClosedWithWarnings"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_risk_governor_transition_completeness_report(
    audit: &RiskGovernorRequiredTransitionAuditReport,
) -> RiskGovernorTransitionCompletenessReport {
    let missing_transition_count = [
        audit.paper_approved_requires_risk,
        audit.paper_rejected_requires_risk,
        audit.risk_denied_requires_risk,
        audit.no_trade_requires_risk,
        audit.cooldown_requires_risk,
    ]
    .into_iter()
    .filter(|present| !present)
    .count();
    RiskGovernorTransitionCompletenessReport {
        report_id: "risk-governor-transition-completeness-report".to_string(),
        required_transition_count: 5,
        missing_transition_count,
        live_transition_present: audit.live_transition_present,
        bypass_transition_count: audit.bypass_transition_count,
        completeness_status: if missing_transition_count == 0
            && !audit.live_transition_present
            && audit.bypass_transition_count == 0
        {
            "RiskGovernorTransitionsComplete"
        } else {
            "RiskGovernorTransitionsIncomplete"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_risk_governor_no_bypass_audit_v2(
    batch: &RiskGovernorBatchVetoReport,
) -> RiskGovernorNoBypassAuditV2 {
    let bypass_detected = batch.bypass_attempt_count > 0;
    RiskGovernorNoBypassAuditV2 {
        report_id: "risk-governor-no-bypass-audit-v2".to_string(),
        chairman_bypass_detected: bypass_detected,
        owner_bypass_detected: bypass_detected,
        member_bypass_detected: bypass_detected,
        bypass_transition_count: batch.bypass_attempt_count,
        audit_status: if bypass_detected {
            "RiskGovernorBypassDetected"
        } else {
            "RiskGovernorNoBypassReadyV2"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_lower_confidence_carry_forward_closure_report(
    policy: &LowerConfidenceCarryForwardPolicy,
    wonyotti: &WonyottiCarryForwardReview,
    larry: &LarryWilliamsCarryForwardReview,
    arthur: &ArthurHayesCarryForwardReview,
) -> LowerConfidenceCarryForwardClosureReport {
    let mut warning_backed_candidates = policy.warning_backed_candidates.clone();
    if wonyotti.remains_warning_backed {
        warning_backed_candidates.push("wonyotti".to_string());
    }
    if larry.remains_warning_backed {
        warning_backed_candidates.push("larry-williams".to_string());
    }
    if arthur.remains_warning_backed {
        warning_backed_candidates.push("arthur-hayes".to_string());
    }
    warning_backed_candidates.sort();
    warning_backed_candidates.dedup();
    let silent_upgrade_count = usize::from(!wonyotti.remains_warning_backed)
        + usize::from(!larry.remains_warning_backed)
        + usize::from(!arthur.remains_warning_backed);
    LowerConfidenceCarryForwardClosureReport {
        report_id: "lower-confidence-carry-forward-closure-report".to_string(),
        warning_backed_candidates,
        silent_upgrade_count,
        live_activation_allowed: policy.carry_forward_allowed_for_live
            || wonyotti.live_activation_allowed
            || larry.live_activation_allowed
            || arthur.live_activation_allowed,
        closure_status: if silent_upgrade_count == 0 {
            "LowerConfidenceCarryForwardStillExplicit"
        } else {
            "LowerConfidenceCarryForwardViolation"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_wonyotti_carry_forward_closure_report(
    review: &WonyottiCarryForwardReview,
) -> WonyottiCarryForwardClosureReport {
    WonyottiCarryForwardClosureReport {
        report_id: "wonyotti-carry-forward-closure-report".to_string(),
        remains_warning_backed: review.remains_warning_backed,
        exact_return_claims_blocked: review.exact_return_claims_blocked,
        carry_forward_allowed_for_paper: review.carry_forward_allowed_for_paper,
        live_activation_allowed: review.live_activation_allowed,
        closure_status: if review.remains_warning_backed {
            "WonyottiCarryForwardStillWarningBacked"
        } else {
            "WonyottiCarryForwardClosed"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_larry_williams_carry_forward_closure_report(
    review: &LarryWilliamsCarryForwardReview,
) -> LarryWilliamsCarryForwardClosureReport {
    LarryWilliamsCarryForwardClosureReport {
        report_id: "larry-williams-carry-forward-closure-report".to_string(),
        remains_warning_backed: review.remains_warning_backed,
        exact_numeric_rule_claims_downweighted: review.exact_numeric_rule_claims_downweighted,
        carry_forward_allowed_for_paper: review.carry_forward_allowed_for_paper,
        live_activation_allowed: review.live_activation_allowed,
        closure_status: if review.remains_warning_backed {
            "LarryWilliamsCarryForwardStillWarningBacked"
        } else {
            "LarryWilliamsCarryForwardClosed"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_arthur_hayes_carry_forward_closure_report(
    review: &ArthurHayesCarryForwardReview,
) -> ArthurHayesCarryForwardClosureReport {
    ArthurHayesCarryForwardClosureReport {
        report_id: "arthur-hayes-carry-forward-closure-report".to_string(),
        remains_warning_backed: review.remains_warning_backed,
        leverage_risk_guard_present: review.leverage_risk_guard_present,
        carry_forward_allowed_for_paper: review.carry_forward_allowed_for_paper,
        live_activation_allowed: review.live_activation_allowed,
        closure_status: if review.remains_warning_backed {
            "ArthurHayesCarryForwardStillWarningBacked"
        } else {
            "ArthurHayesCarryForwardClosed"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_lifecycle_readiness_gate_v2(
    transitions: &PaperCandidateTransitionCoverageReport,
    gates: &PaperCandidateGateCompletenessReport,
    evidence: &PaperCandidateEvidenceDepthClosureReport,
    trace: &PaperCandidateTraceClosureReport,
    stability: &PaperCandidateStabilityClosureReport,
    risk_veto: &RiskGovernorBatchVetoWarningClosureReport,
    lower_confidence: &LowerConfidenceCarryForwardClosureReport,
) -> PaperLifecycleReadinessGateV2 {
    let no_blockers = transitions.coverage_status == "PaperCandidateTransitionCoverageReady"
        && gates.completeness_status == "PaperCandidateGatesComplete"
        && trace.broker_execution_allowed_count == 0
        && trace.live_execution_allowed_count == 0
        && risk_veto.bypass_attempt_count == 0;
    let lifecycle_ready = no_blockers;
    let gate_status = if !no_blockers {
        "PaperLifecycleBlocked"
    } else if evidence.candidates_needing_more_evidence == 0
        && stability.unstable_candidates == 0
        && lower_confidence.silent_upgrade_count == 0
        && risk_veto.warning_count_remaining == 0
    {
        "PaperLifecycleReady"
    } else {
        "PaperLifecycleReadyWithWarnings"
    };
    PaperLifecycleReadinessGateV2 {
        gate_id: "paper-lifecycle-readiness-gate-v2".to_string(),
        transition_coverage_status: transitions.coverage_status.clone(),
        gate_completeness_status: gates.completeness_status.clone(),
        evidence_closure_status: evidence.closure_status.clone(),
        trace_closure_status: trace.closure_status.clone(),
        stability_closure_status: stability.closure_status.clone(),
        risk_veto_closure_status: risk_veto.closure_status.clone(),
        lower_confidence_status: lower_confidence.closure_status.clone(),
        lifecycle_ready,
        live_lifecycle_allowed: false,
        gate_status: gate_status.to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_batch_replay_v2_plan(
    replay: &PaperRotationBatchReplayReport,
) -> PaperCandidateBatchReplayV2Plan {
    PaperCandidateBatchReplayV2Plan {
        plan_id: "paper-candidate-batch-replay-v2-plan".to_string(),
        replay_count: replay.replay_count,
        expected_paper_approved_count: replay.paper_conditional_count,
        expected_paper_rejected_count: 1,
        replay_schedule: (0..replay.replay_count)
            .map(|index| format!("batch-replay-v2-scenario-{:02}", index + 1))
            .collect(),
        plan_status: "PaperCandidateBatchReplayV2PlanReady".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_batch_replay_v2_report(
    replay: &PaperRotationBatchReplayReport,
    lifecycle: &PaperCandidateLifecycleStateMachine,
) -> PaperCandidateBatchReplayV2Report {
    let paper_rejected_count = usize::from(
        lifecycle
            .allowed_transitions
            .iter()
            .any(|transition| transition == "DebateOpen->PaperRejected"),
    );
    PaperCandidateBatchReplayV2Report {
        report_id: "paper-candidate-batch-replay-v2-report".to_string(),
        replay_count: replay.replay_count,
        paper_approved_count: replay.paper_conditional_count,
        paper_rejected_count,
        broker_execution_allowed_count: replay.broker_execution_allowed_count,
        live_execution_allowed_count: replay.live_execution_allowed_count,
        report_status: "PaperCandidateBatchReplayV2Ready".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_compile_cost_diagnosis_v2(
    review: &ReviewPatchSummary,
) -> WorkspaceCompileCostDiagnosisV2 {
    let mut suspected_compile_cost_causes = vec![
        "workspace target graph remains large".to_string(),
        "full workspace compile includes many historical crates".to_string(),
    ];
    if review.no_run_long_running_observed || review.full_run_long_running_observed {
        suspected_compile_cost_causes
            .push("workspace compile cost observed in practice".to_string());
    }
    WorkspaceCompileCostDiagnosisV2 {
        report_id: "workspace-compile-cost-diagnosis-v2".to_string(),
        no_run_long_running_observed: review.no_run_long_running_observed,
        full_run_long_running_observed: review.full_run_long_running_observed,
        suspected_compile_cost_causes,
        diagnosis_status: "WorkspaceCompileCostDiagnosisReadyV2".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_focused_vs_full_gate_bridge_v2(
    final_gate: &FinalVerificationGate,
    workspace_truth: &WorkspaceTruthSnapshot,
) -> FocusedVsFullGateBridgeV2 {
    let full_workspace_accepted = full_workspace_accepted(workspace_truth);
    FocusedVsFullGateBridgeV2 {
        report_id: "focused-vs-full-gate-bridge-v2".to_string(),
        focused_pass_recorded: final_gate.final_verification_passed,
        full_workspace_open: !full_workspace_accepted,
        bridge_rules: vec![
            "focused pass is not full workspace acceptance".to_string(),
            "verification pass is not full workspace acceptance".to_string(),
            "full workspace requires finished && passed cargo test --workspace --quiet".to_string(),
        ],
        bridge_status: "FocusedVsFullGateBridgeReadyV2".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_acceptance_truth_recovery_plan_v6(
    prior: &WorkspaceAcceptanceTruthClosurePlanV5,
    compile_diagnosis: &WorkspaceCompileCostDiagnosisV2,
    bridge: &FocusedVsFullGateBridgeV2,
    workspace_truth: &WorkspaceTruthSnapshot,
) -> WorkspaceAcceptanceTruthRecoveryPlanV6 {
    let can_claim_full_acceptance = full_workspace_accepted(workspace_truth);
    WorkspaceAcceptanceTruthRecoveryPlanV6 {
        plan_id: "workspace-acceptance-truth-recovery-plan-v6".to_string(),
        previous_truth_status: prior.current_truth_status.clone(),
        current_truth_status: if can_claim_full_acceptance {
            "WorkspaceAcceptanceRecoveredV6".to_string()
        } else {
            "WorkspaceAcceptanceStillOpenV6".to_string()
        },
        compile_diagnosis_status: compile_diagnosis.diagnosis_status.clone(),
        bridge_status: bridge.bridge_status.clone(),
        can_claim_full_acceptance,
        recommended_actions: if can_claim_full_acceptance {
            vec!["archive finished && passed workspace truth".to_string()]
        } else {
            vec![
                "keep focused validation distinct from full workspace acceptance".to_string(),
                "run cargo test --workspace --no-run --quiet honestly".to_string(),
                "run cargo test --workspace --quiet honestly".to_string(),
                "do not claim acceptance until both finish and full pass succeeds".to_string(),
            ]
        },
        recovery_status: if can_claim_full_acceptance {
            "WorkspaceAcceptanceRecoveredV6".to_string()
        } else {
            "WorkspaceAcceptanceStillOpenV6".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn run_command_with_timeout(
    command: &str,
    timeout_ms: u64,
) -> Result<(bool, bool, Option<bool>), String> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(project_root())
        .spawn()
        .map_err(|err| err.to_string())?;
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|err| err.to_string())? {
            return Ok((true, true, Some(status.success())));
        }
        if start.elapsed() >= Duration::from_millis(timeout_ms) {
            child.kill().map_err(|err| err.to_string())?;
            let _ = child.wait();
            return Ok((true, false, None));
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn build_workspace_acceptance_attempt_v21(
    config: &Sprint105VerificationPatchClosureConfig,
    _workspace_truth: &WorkspaceTruthSnapshot,
) -> Result<WorkspaceAcceptanceAttemptV21, String> {
    let timeout_ms = config.workspace_acceptance_timeout_ms;
    if config.run_workspace_acceptance_attempt {
        let timeout_ms = timeout_ms.unwrap_or(120_000);
        let command_no_run = "cargo test --workspace --no-run --quiet".to_string();
        let command_full = "cargo test --workspace --quiet".to_string();
        let (no_run_started, no_run_finished, no_run_passed) =
            run_command_with_timeout(&command_no_run, timeout_ms)?;
        let (full_started, full_finished, full_passed) =
            run_command_with_timeout(&command_full, timeout_ms)?;
        let can_claim_full_acceptance = no_run_finished
            && no_run_passed == Some(true)
            && full_finished
            && full_passed == Some(true);
        return Ok(WorkspaceAcceptanceAttemptV21 {
            attempt_id: "workspace-acceptance-attempt-v21".to_string(),
            command_no_run,
            command_full,
            no_run_started,
            no_run_finished,
            no_run_passed,
            full_started,
            full_finished,
            full_passed,
            timeout_ms: Some(timeout_ms),
            can_claim_full_acceptance,
            attempt_status: if can_claim_full_acceptance {
                "WorkspaceAcceptancePassedV21"
            } else if full_started && !full_finished {
                "WorkspaceAcceptanceTimedOutV21"
            } else {
                "WorkspaceAcceptanceIncompleteV21"
            }
            .to_string(),
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    Ok(WorkspaceAcceptanceAttemptV21 {
        attempt_id: "workspace-acceptance-attempt-v21".to_string(),
        command_no_run: "cargo test --workspace --no-run --quiet".to_string(),
        command_full: "cargo test --workspace --quiet".to_string(),
        no_run_started: false,
        no_run_finished: false,
        no_run_passed: None,
        full_started: false,
        full_finished: false,
        full_passed: None,
        timeout_ms,
        can_claim_full_acceptance: false,
        attempt_status: "WorkspaceAcceptanceDeferredV21".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    })
}

fn build_safety_coverage_preservation_report_v21(
    config: &Sprint105VerificationPatchClosureConfig,
    prior: &SafetyCoveragePreservationReportV20,
    overclaim_guard: &OverclaimRegressionGuardReport,
    paper_rejected_audit: &PaperRejectedTransitionAuditReport,
    risk_transition_audit: &RiskGovernorRequiredTransitionAuditReport,
    artifact_policy: &MissingArtifactFindingPolicyReport,
) -> SafetyCoveragePreservationReportV21 {
    let safety_preserved = config.preserve_safety_guards;
    let runtime_preserved = config.preserve_runtime_deferred;
    let live_trading_guard_present = safety_preserved && prior.live_trading_guard_present;
    let broker_guard_present = safety_preserved && prior.broker_guard_present;
    let order_guard_present = safety_preserved && prior.order_guard_present;
    let account_guard_present = safety_preserved && prior.account_guard_present;
    let runtime_llm_guard_present = runtime_preserved && prior.runtime_llm_guard_present;
    let mamba_runtime_guard_present = runtime_preserved && prior.mamba_runtime_guard_present;
    let gated_runtime_guard_present = runtime_preserved && prior.gated_runtime_guard_present;
    let model_training_guard_present = safety_preserved && prior.model_training_guard_present;
    let python_training_dependency_guard_present =
        safety_preserved && prior.python_training_dependency_guard_present;
    let browser_execution_guard_present =
        runtime_preserved && prior.browser_execution_guard_present;
    let dashboard_serve_guard_present = runtime_preserved && prior.dashboard_serve_guard_present;
    let investor_impersonation_guard_present =
        safety_preserved && prior.investor_impersonation_guard_present;
    let no_silent_confidence_upgrade_guard_present =
        safety_preserved && prior.no_silent_confidence_upgrade_guard_present;
    let dual_agent_workflow_guard_present =
        safety_preserved && prior.dual_agent_workflow_guard_present;
    let verification_not_acceptance_guard_present =
        safety_preserved && prior.verification_not_acceptance_guard_present;
    let paper_candidate_not_order_guard_present =
        safety_preserved && prior.paper_candidate_not_order_guard_present;
    let paper_candidate_lifecycle_no_live_guard_present =
        safety_preserved && prior.paper_candidate_lifecycle_no_live_guard_present;
    let overclaim_guard_present = !overclaim_guard.regression_detected;
    let paper_rejected_transition_guard_present = paper_rejected_audit.paper_rejected_reachable
        && !paper_rejected_audit.paper_rejected_can_go_live
        && !paper_rejected_audit.paper_rejected_can_become_order;
    let risk_governor_transition_guard_present = risk_transition_audit.audit_status
        == "RiskGovernorRequiredTransitionsReady"
        && risk_transition_audit.bypass_transition_count == 0
        && !risk_transition_audit.live_transition_present;
    let missing_artifact_finding_guard_present = artifact_policy.missing_docs_become_findings
        && artifact_policy.missing_tests_become_findings
        && artifact_policy.missing_examples_become_findings
        && !artifact_policy.silent_success_detected;
    let all_guards_present = [
        live_trading_guard_present,
        broker_guard_present,
        order_guard_present,
        account_guard_present,
        runtime_llm_guard_present,
        mamba_runtime_guard_present,
        gated_runtime_guard_present,
        model_training_guard_present,
        python_training_dependency_guard_present,
        browser_execution_guard_present,
        dashboard_serve_guard_present,
        investor_impersonation_guard_present,
        no_silent_confidence_upgrade_guard_present,
        dual_agent_workflow_guard_present,
        verification_not_acceptance_guard_present,
        paper_candidate_not_order_guard_present,
        paper_candidate_lifecycle_no_live_guard_present,
        overclaim_guard_present,
        paper_rejected_transition_guard_present,
        risk_governor_transition_guard_present,
        missing_artifact_finding_guard_present,
    ]
    .into_iter()
    .all(|value| value);
    SafetyCoveragePreservationReportV21 {
        report_id: "safety-coverage-preservation-report-v21".to_string(),
        live_trading_guard_present,
        broker_guard_present,
        order_guard_present,
        account_guard_present,
        runtime_llm_guard_present,
        mamba_runtime_guard_present,
        gated_runtime_guard_present,
        model_training_guard_present,
        python_training_dependency_guard_present,
        browser_execution_guard_present,
        dashboard_serve_guard_present,
        investor_impersonation_guard_present,
        no_silent_confidence_upgrade_guard_present,
        dual_agent_workflow_guard_present,
        verification_not_acceptance_guard_present,
        paper_candidate_not_order_guard_present,
        paper_candidate_lifecycle_no_live_guard_present,
        overclaim_guard_present,
        paper_rejected_transition_guard_present,
        risk_governor_transition_guard_present,
        missing_artifact_finding_guard_present,
        safety_status: if all_guards_present {
            "SafetyCoveragePreservedV21".to_string()
        } else {
            "SafetyCoverageRegressionV21".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_control_tower_verification_patch_closure_panel(
    closure: &VerificationFindingClosureReport,
    review: &ReviewPatchEffectReport,
    overclaim: &OverclaimRegressionGuardReport,
    workspace_truth: &WorkspaceAttemptTruthHardeningReport,
    safety: &SafetyBooleanCoverageAuditReport,
    final_gate: &FinalVerificationGateV2,
) -> ControlTowerVerificationPatchClosurePanel {
    ControlTowerVerificationPatchClosurePanel {
        panel_id: "control-tower-verification-patch-closure-panel".to_string(),
        finding_closure_status: closure.closure_status.clone(),
        review_patch_effect_status: review.effect_status.clone(),
        overclaim_guard_status: overclaim.guard_status.clone(),
        workspace_truth_status: workspace_truth.truth_hardening_status.clone(),
        safety_boolean_audit_status: safety.audit_status.clone(),
        final_gate_status: final_gate.gate_status.clone(),
        warnings: vec![
            "static/read-only panel only".to_string(),
            "verification is not full workspace acceptance".to_string(),
            "no verification execution button".to_string(),
        ],
        next_actions: vec![
            "keep finished && passed as the only full acceptance rule".to_string(),
            "keep missing artifacts visible as findings".to_string(),
            "keep full workspace truth separate from focused passes".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_control_tower_paper_lifecycle_closure_panel(
    lifecycle: &PaperLifecycleWarningClosureReport,
    transitions: &PaperCandidateTransitionCoverageReport,
    gates: &PaperCandidateGateCompletenessReport,
    trace: &PaperCandidateTraceClosureReport,
    stability: &PaperCandidateStabilityClosureReport,
    risk_veto: &RiskGovernorBatchVetoWarningClosureReport,
    lower_confidence: &LowerConfidenceCarryForwardClosureReport,
    readiness: &PaperLifecycleReadinessGateV2,
    workspace_truth: &WorkspaceTruthSnapshot,
) -> ControlTowerPaperLifecycleClosurePanel {
    ControlTowerPaperLifecycleClosurePanel {
        panel_id: "control-tower-paper-lifecycle-closure-panel".to_string(),
        lifecycle_closure_status: lifecycle.closure_status.clone(),
        transition_coverage_status: transitions.coverage_status.clone(),
        gate_completeness_status: gates.completeness_status.clone(),
        trace_closure_status: trace.closure_status.clone(),
        stability_closure_status: stability.closure_status.clone(),
        risk_veto_status: risk_veto.closure_status.clone(),
        lower_confidence_status: lower_confidence.closure_status.clone(),
        readiness_gate_status: readiness.gate_status.clone(),
        workspace_truth_status: workspace_truth.truth_status.clone(),
        warnings: vec![
            "static/read-only panel only".to_string(),
            "paper candidate is not order execution".to_string(),
            "no promote-to-live button and no order/account controls".to_string(),
        ],
        next_actions: vec![
            "close remaining paper lifecycle warnings conservatively".to_string(),
            "preserve Risk Governor final veto".to_string(),
            "keep lower-confidence carry-forward explicit".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[]),
    }
}
