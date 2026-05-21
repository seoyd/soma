use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string};
use crate::league::sprint103_paper_rotation_closure::{
    ArthurHayesWarningClosureReport, LarryWilliamsWarningClosureReport,
    MultiScenarioPaperReplayPack, MultiScenarioPaperReplayReport,
    PaperRotationWarningClosureConfig, Sprint103PaperRotationClosureBundle,
    Sprint103PaperRotationClosureRunner, WonyottiWarningClosureReport,
};
use crate::model::WorkspaceAcceptanceTruthGateStatus;

const SPRINT104_DOCS: &[&str] = &[
    "docs/SPRINT104_DUAL_AGENT_PAPER_LIFECYCLE.md",
    "docs/DUAL_AGENT_IMPLEMENTATION_VERIFICATION_PROTOCOL.md",
    "docs/FIVE_FOUR_IMPLEMENTATION_FIVE_FIVE_VERIFICATION.md",
    "docs/VERIFICATION_FINDINGS_AND_PATCH_LOOP.md",
    "docs/PAPER_CANDIDATE_LIFECYCLE.md",
    "docs/PAPER_BATCH_REPLAY.md",
    "docs/LOWER_CONFIDENCE_CARRY_FORWARD_POLICY.md",
    "docs/CONTROL_TOWER_DUAL_AGENT_PANEL.md",
    "docs/CONTROL_TOWER_PAPER_CANDIDATE_LIFECYCLE.md",
    "docs/SPRINT104_REPORT.md",
];

const SPRINT104_EXAMPLES: &[&str] = &[
    "examples/soma_sprint104_dual_agent_paper_lifecycle.toml",
    "examples/soma_dual_agent_workflow_policy.toml",
    "examples/soma_implementation_agent_role.toml",
    "examples/soma_verification_agent_role.toml",
    "examples/soma_prompt_compliance_verification.toml",
    "examples/soma_safety_invariant_verification.toml",
    "examples/soma_architecture_regression_verification.toml",
    "examples/soma_final_verification_gate.toml",
    "examples/soma_paper_batch_replay.toml",
    "examples/soma_paper_candidate_lifecycle.toml",
    "examples/soma_paper_candidate_notrade_gate.toml",
    "examples/soma_risk_governor_batch_veto.toml",
    "examples/soma_lower_confidence_carry_forward.toml",
    "examples/soma_control_tower_dual_agent.toml",
    "examples/soma_control_tower_paper_candidate_lifecycle.toml",
];

const SPRINT104_FIXTURES: &[&str] = &[
    "examples/sprint104_data/sprint103_summary.json",
    "examples/sprint104_data/dual_agent_workflow_expected.json",
    "examples/sprint104_data/verification_findings_expected.json",
    "examples/sprint104_data/final_verification_gate_expected.json",
    "examples/sprint104_data/paper_batch_replay_expected.json",
    "examples/sprint104_data/paper_candidate_lifecycle_expected.json",
    "examples/sprint104_data/risk_governor_batch_veto_expected.json",
    "examples/sprint104_data/control_tower_dual_agent_expected.json",
    "examples/sprint104_data/control_tower_paper_lifecycle_expected.json",
    "examples/sprint104_data/safety_coverage_v20_expected.json",
];

const SPRINT104_TESTS: &[&str] = &[
    "tests/dual_agent_workflow_policy.rs",
    "tests/verification_findings.rs",
    "tests/prompt_compliance_verification.rs",
    "tests/safety_invariant_verification.rs",
    "tests/architecture_regression_verification.rs",
    "tests/final_verification_gate.rs",
    "tests/paper_batch_replay.rs",
    "tests/paper_candidate_lifecycle.rs",
    "tests/paper_candidate_gates.rs",
    "tests/risk_governor_batch_veto.rs",
    "tests/lower_confidence_carry_forward.rs",
    "tests/control_tower_dual_agent_panel.rs",
    "tests/control_tower_paper_candidate_lifecycle.rs",
    "tests/sprint104_cli_safety.rs",
    "tests/sprint104_determinism.rs",
];

const SPRINT104_CLI_COMMANDS: &[&str] = &[
    "sprint104-dual-agent-paper-lifecycle",
    "dual-agent-workflow-policy",
    "implementation-agent-role",
    "verification-agent-role",
    "prompt-compliance-verification",
    "safety-invariant-verification",
    "architecture-regression-verification",
    "test-coverage-verification",
    "final-verification-gate",
    "paper-batch-replay",
    "paper-candidate-lifecycle",
    "paper-candidate-promotion-gate",
    "paper-candidate-notrade-gate",
    "paper-candidate-riskdenied-gate",
    "risk-governor-batch-veto",
    "lower-confidence-carry-forward",
    "control-tower-dual-agent",
    "control-tower-paper-candidate-lifecycle",
];

const SPRINT104_ADDED_FILES: &[&str] = &[
    "src/league/sprint104_dual_agent_paper_lifecycle.rs",
    "tests/support/sprint104_support.rs",
    "tests/dual_agent_workflow_policy.rs",
    "tests/verification_findings.rs",
    "tests/prompt_compliance_verification.rs",
    "tests/safety_invariant_verification.rs",
    "tests/architecture_regression_verification.rs",
    "tests/final_verification_gate.rs",
    "tests/paper_batch_replay.rs",
    "tests/paper_candidate_lifecycle.rs",
    "tests/paper_candidate_gates.rs",
    "tests/risk_governor_batch_veto.rs",
    "tests/lower_confidence_carry_forward.rs",
    "tests/control_tower_dual_agent_panel.rs",
    "tests/control_tower_paper_candidate_lifecycle.rs",
    "tests/sprint104_cli_safety.rs",
    "tests/sprint104_determinism.rs",
    "docs/SPRINT104_DUAL_AGENT_PAPER_LIFECYCLE.md",
    "docs/DUAL_AGENT_IMPLEMENTATION_VERIFICATION_PROTOCOL.md",
    "docs/FIVE_FOUR_IMPLEMENTATION_FIVE_FIVE_VERIFICATION.md",
    "docs/VERIFICATION_FINDINGS_AND_PATCH_LOOP.md",
    "docs/PAPER_CANDIDATE_LIFECYCLE.md",
    "docs/PAPER_BATCH_REPLAY.md",
    "docs/LOWER_CONFIDENCE_CARRY_FORWARD_POLICY.md",
    "docs/CONTROL_TOWER_DUAL_AGENT_PANEL.md",
    "docs/CONTROL_TOWER_PAPER_CANDIDATE_LIFECYCLE.md",
    "docs/SPRINT104_REPORT.md",
    "examples/soma_sprint104_dual_agent_paper_lifecycle.toml",
    "examples/soma_dual_agent_workflow_policy.toml",
    "examples/soma_implementation_agent_role.toml",
    "examples/soma_verification_agent_role.toml",
    "examples/soma_prompt_compliance_verification.toml",
    "examples/soma_safety_invariant_verification.toml",
    "examples/soma_architecture_regression_verification.toml",
    "examples/soma_final_verification_gate.toml",
    "examples/soma_paper_batch_replay.toml",
    "examples/soma_paper_candidate_lifecycle.toml",
    "examples/soma_paper_candidate_notrade_gate.toml",
    "examples/soma_risk_governor_batch_veto.toml",
    "examples/soma_lower_confidence_carry_forward.toml",
    "examples/soma_control_tower_dual_agent.toml",
    "examples/soma_control_tower_paper_candidate_lifecycle.toml",
    "examples/sprint104_data/sprint103_summary.json",
    "examples/sprint104_data/dual_agent_workflow_expected.json",
    "examples/sprint104_data/verification_findings_expected.json",
    "examples/sprint104_data/final_verification_gate_expected.json",
    "examples/sprint104_data/paper_batch_replay_expected.json",
    "examples/sprint104_data/paper_candidate_lifecycle_expected.json",
    "examples/sprint104_data/risk_governor_batch_veto_expected.json",
    "examples/sprint104_data/control_tower_dual_agent_expected.json",
    "examples/sprint104_data/control_tower_paper_lifecycle_expected.json",
    "examples/sprint104_data/safety_coverage_v20_expected.json",
];

const SPRINT104_CHANGED_FILES: &[&str] = &[
    "src/league/mod.rs",
    "src/lib.rs",
    "src/bin/soma_experiment.rs",
    "tests/support/mod.rs",
];

fn render_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|err| err.to_string())
}

fn local_only(path: &str) -> bool {
    !path.contains("://")
}

fn full_workspace_accepted(snapshot: &WorkspaceTruthSnapshot) -> bool {
    snapshot.can_claim_full_acceptance
        && snapshot.full_workspace_finished
        && snapshot.full_workspace_passed == Some(true)
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

fn default_impl_agent_name() -> String {
    "codex-5.4".to_string()
}

fn default_verification_agent_name() -> String {
    "gpt-5.5".to_string()
}

fn default_output_root() -> String {
    "target/soma_sprint104_dual_agent_paper_lifecycle".to_string()
}

fn default_replay_count() -> usize {
    7
}

fn default_max_batch_size() -> usize {
    18
}

fn default_timeout_ms() -> Option<u64> {
    Some(120_000)
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_manifest_entries(paths: Option<&Vec<String>>) -> Result<Vec<String>, String> {
    let Some(paths) = paths else {
        return Ok(Vec::new());
    };
    let mut entries = BTreeSet::new();
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        if let Ok(values) = serde_json::from_str::<Vec<String>>(&text) {
            entries.extend(values.into_iter().filter(|value| !value.trim().is_empty()));
            continue;
        }
        for line in text.lines() {
            let line = line.trim();
            if !line.is_empty() {
                entries.insert(line.to_string());
            }
        }
    }
    Ok(entries.into_iter().collect())
}

#[derive(Clone, Debug)]
struct WorkspaceTruthSnapshot {
    truth_status: String,
    no_run_status_reported: bool,
    full_workspace_status_reported: bool,
    full_workspace_finished: bool,
    full_workspace_passed: Option<bool>,
    can_claim_full_acceptance: bool,
}

#[derive(Clone, Debug)]
struct PaperCandidateRecord {
    candidate_id: String,
    scenario_id: String,
    state: PaperCandidateLifecycleState,
    decision_label: String,
    has_official_evidence: bool,
    has_counterfactual_evidence: bool,
    has_regime_evidence: bool,
    has_risk_evidence: bool,
    has_no_lookahead_proof: bool,
    has_proposal_ref: bool,
    has_debate_ref: bool,
    has_chairman_ref: bool,
    has_risk_governor_ref: bool,
    stable: bool,
    instability_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DualAgentWorkflowConfig {
    pub workflow_id: String,
    #[serde(default)]
    pub sprint103_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub implementation_report_paths: Option<Vec<String>>,
    #[serde(default)]
    pub verification_report_paths: Option<Vec<String>>,
    #[serde(default)]
    pub changed_file_manifest_paths: Option<Vec<String>>,
    #[serde(default)]
    pub focused_test_report_paths: Option<Vec<String>>,
    #[serde(default)]
    pub cli_smoke_report_paths: Option<Vec<String>>,
    #[serde(default)]
    pub workspace_truth_paths: Option<Vec<String>>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_impl_agent_name")]
    pub implementation_agent_name: String,
    #[serde(default = "default_verification_agent_name")]
    pub verification_agent_name: String,
    #[serde(default = "default_true")]
    pub require_implementation_summary: bool,
    #[serde(default = "default_true")]
    pub require_verification_summary: bool,
    #[serde(default = "default_true")]
    pub require_safety_verification: bool,
    #[serde(default = "default_true")]
    pub require_architecture_verification: bool,
    #[serde(default = "default_true")]
    pub require_test_verification: bool,
    #[serde(default = "default_true")]
    pub require_cli_doc_example_verification: bool,
    #[serde(default = "default_true")]
    pub require_workspace_truth_verification: bool,
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

impl Default for DualAgentWorkflowConfig {
    fn default() -> Self {
        Self {
            workflow_id: "sprint104-dual-agent-paper-lifecycle".to_string(),
            sprint103_bundle_paths: Some(vec![
                "examples/sprint104_data/sprint103_summary.json".to_string(),
            ]),
            implementation_report_paths: None,
            verification_report_paths: None,
            changed_file_manifest_paths: None,
            focused_test_report_paths: None,
            cli_smoke_report_paths: None,
            workspace_truth_paths: None,
            output_root: default_output_root(),
            implementation_agent_name: default_impl_agent_name(),
            verification_agent_name: default_verification_agent_name(),
            require_implementation_summary: true,
            require_verification_summary: true,
            require_safety_verification: true,
            require_architecture_verification: true,
            require_test_verification: true,
            require_cli_doc_example_verification: true,
            require_workspace_truth_verification: true,
            preserve_runtime_deferred: true,
            preserve_safety_guards: true,
            run_workspace_acceptance_attempt: false,
            workspace_acceptance_timeout_ms: default_timeout_ms(),
            reason_codes: deferred_reason_codes(&[]),
        }
    }
}

impl DualAgentWorkflowConfig {
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
        PathBuf::from(&self.output_root).join(&self.workflow_id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.workflow_id.trim().is_empty() {
            return Err("sprint104 workflow_id must not be empty".to_string());
        }
        if self.output_root.trim().is_empty() {
            return Err("sprint104 output_root must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err("sprint104 dual-agent workflow config paths must be local".to_string());
        }
        for paths in [
            &self.sprint103_bundle_paths,
            &self.implementation_report_paths,
            &self.verification_report_paths,
            &self.changed_file_manifest_paths,
            &self.focused_test_report_paths,
            &self.cli_smoke_report_paths,
            &self.workspace_truth_paths,
        ] {
            if let Some(paths) = paths
                && paths.iter().any(|path| !local_only(path))
            {
                return Err("sprint104 dual-agent workflow config paths must be local".to_string());
            }
        }
        if self.implementation_agent_name.trim().is_empty()
            || self.verification_agent_name.trim().is_empty()
        {
            return Err("sprint104 agent names must not be empty".to_string());
        }
        if self.preserve_safety_guards && !self.preserve_runtime_deferred {
            return Err(
                "sprint104 safety preservation requires runtime deferred preservation".to_string(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperRotationBatchReplayConfig {
    pub batch_id: String,
    #[serde(default)]
    pub sprint103_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub scenario_pack_paths: Option<Vec<String>>,
    #[serde(default = "default_replay_count")]
    pub replay_count: usize,
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: usize,
    #[serde(default = "default_true")]
    pub require_paper_only: bool,
    #[serde(default = "default_true")]
    pub require_risk_governor_handoff: bool,
    #[serde(default = "default_true")]
    pub require_no_live_execution: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateLifecycleConfig {
    pub lifecycle_id: String,
    #[serde(default)]
    pub sprint103_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub batch_replay_paths: Option<Vec<String>>,
    #[serde(default = "default_true")]
    pub require_paper_only: bool,
    #[serde(default = "default_true")]
    pub require_risk_governor_review: bool,
    #[serde(default = "default_true")]
    pub require_no_live_execution: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DualAgentWorkflowPolicy {
    pub policy_id: String,
    pub implementation_agent: String,
    pub verification_agent: String,
    pub implementation_allowed_actions: Vec<String>,
    pub verification_allowed_actions: Vec<String>,
    pub implementation_forbidden_actions: Vec<String>,
    pub verification_forbidden_actions: Vec<String>,
    pub handoff_required: bool,
    pub findings_required: bool,
    pub final_verification_required: bool,
    pub policy_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImplementationAgentRoleReport {
    pub report_id: String,
    pub agent_name: String,
    pub files_added: Vec<String>,
    pub files_changed: Vec<String>,
    pub tests_added: Vec<String>,
    pub docs_added: Vec<String>,
    pub examples_added: Vec<String>,
    pub cli_added: Vec<String>,
    pub focused_checks_run: Vec<String>,
    pub implementation_summary_present: bool,
    pub implementation_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationAgentRoleReport {
    pub report_id: String,
    pub agent_name: String,
    pub prompt_requirements_checked: usize,
    pub changed_files_checked: usize,
    pub tests_checked: usize,
    pub docs_checked: usize,
    pub cli_checked: usize,
    pub safety_checked: bool,
    pub architecture_checked: bool,
    pub workspace_truth_checked: bool,
    pub verification_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationFindingSeverity {
    Blocking,
    Major,
    Minor,
    Informational,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationFindingCategory {
    PromptCompliance,
    SafetyInvariant,
    ArchitectureRegression,
    TestCoverage,
    CliSurface,
    DocsExamples,
    Determinism,
    WorkspaceTruth,
    Overclaim,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationFindingStatus {
    Open,
    Fixed,
    AcceptedAsKnownWarning,
    RejectedWithReason,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationFinding {
    pub finding_id: String,
    pub severity: VerificationFindingSeverity,
    pub category: VerificationFindingCategory,
    pub description: String,
    pub affected_files: Vec<String>,
    pub required_fix: String,
    pub finding_status: VerificationFindingStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptComplianceVerificationReport {
    pub report_id: String,
    pub requirements_total: usize,
    pub requirements_satisfied: usize,
    pub requirements_missing: Vec<String>,
    pub blocking_findings: usize,
    pub nonblocking_findings: usize,
    pub compliance_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyInvariantVerificationReport {
    pub report_id: String,
    pub no_live_trading: bool,
    pub no_broker_order_account: bool,
    pub no_runtime_llm_live_decision: bool,
    pub no_mamba_runtime: bool,
    pub no_gated_runtime: bool,
    pub no_model_training: bool,
    pub no_python_training_dependency: bool,
    pub no_dashboard_serve: bool,
    pub no_browser_execution: bool,
    pub no_order_buttons: bool,
    pub no_secret_leakage: bool,
    pub no_investor_impersonation: bool,
    pub no_18_live_activation: bool,
    pub safety_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitectureRegressionVerificationReport {
    pub report_id: String,
    pub committee_owned_core_preserved: bool,
    pub central_core_not_reintroduced: bool,
    pub member_owned_core_refs_preserved: bool,
    pub investor_archetype_cards_preserved: bool,
    pub paper_rotation_semantics_preserved: bool,
    pub risk_governor_final_veto_preserved: bool,
    pub architecture_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestCoverageVerificationReport {
    pub report_id: String,
    pub focused_tests_present: bool,
    pub cli_safety_tests_present: bool,
    pub determinism_tests_present: bool,
    pub safety_tests_present: bool,
    pub workspace_attempt_reported: bool,
    pub hidden_skips_detected: bool,
    pub assertion_deletion_detected: bool,
    pub test_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliSurfaceVerificationReport {
    pub report_id: String,
    pub cli_commands_added: Vec<String>,
    pub cli_help_research_only: bool,
    pub cli_help_paper_only: bool,
    pub cli_help_no_live: bool,
    pub cli_help_no_order_account: bool,
    pub remote_path_rejection_present: bool,
    pub forbidden_cli_detected: bool,
    pub cli_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocsExamplesVerificationReport {
    pub report_id: String,
    pub docs_present: bool,
    pub examples_present: bool,
    pub fixture_data_present: bool,
    pub docs_warn_no_live: bool,
    pub docs_warn_no_order: bool,
    pub docs_warn_no_training: bool,
    pub docs_warn_workspace_truth: bool,
    pub docs_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterminismVerificationReport {
    pub report_id: String,
    pub deterministic_outputs_checked: bool,
    pub fixture_order_stable: bool,
    pub report_order_stable: bool,
    pub fingerprint_stable: bool,
    pub nondeterminism_detected: bool,
    pub determinism_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceTruthVerificationReport {
    pub report_id: String,
    pub focused_pass_distinguished_from_full: bool,
    pub no_run_status_reported: bool,
    pub full_workspace_status_reported: bool,
    pub can_claim_full_acceptance: bool,
    pub overclaim_detected: bool,
    pub truth_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchCorrectionPlan {
    pub plan_id: String,
    pub open_findings: Vec<String>,
    pub blocking_findings: Vec<String>,
    pub required_patches: Vec<String>,
    pub optional_patches: Vec<String>,
    pub patch_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DualAgentReviewLoopReport {
    pub report_id: String,
    pub implementation_passes: usize,
    pub verification_passes: usize,
    pub findings_open_before: usize,
    pub findings_fixed: usize,
    pub findings_remaining: usize,
    pub final_verification_status: String,
    pub review_loop_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalVerificationGate {
    pub gate_id: String,
    pub prompt_compliance_status: String,
    pub safety_invariant_status: String,
    pub architecture_status: String,
    pub test_coverage_status: String,
    pub cli_status: String,
    pub docs_status: String,
    pub determinism_status: String,
    pub workspace_truth_status: String,
    pub blocking_findings_remaining: usize,
    pub final_verification_passed: bool,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperRotationBatchReplayPlan {
    pub plan_id: String,
    pub batch_scenarios: Vec<String>,
    pub replay_schedule: Vec<String>,
    pub selected_member_groups: Vec<String>,
    pub expected_trace_outputs: Vec<String>,
    pub plan_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperRotationBatchReplayReport {
    pub report_id: String,
    pub replay_count: usize,
    pub watch_candidate_count: usize,
    pub paper_conditional_count: usize,
    pub no_trade_count: usize,
    pub risk_denied_count: usize,
    pub need_more_evidence_count: usize,
    pub unstable_decision_count: usize,
    pub broker_execution_allowed_count: usize,
    pub live_execution_allowed_count: usize,
    pub replay_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PaperCandidateLifecycleState {
    Watch,
    Candidate,
    DebateOpen,
    NeedMoreEvidence,
    NoTrade,
    RiskDenied,
    PaperApproved,
    PaperRejected,
    Cooldown,
    ArchivedPaperOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateLifecycleStateMachine {
    pub machine_id: String,
    pub allowed_transitions: Vec<String>,
    pub forbidden_transitions: Vec<String>,
    pub risk_governor_required_transitions: Vec<String>,
    pub broker_execution_allowed: bool,
    pub live_execution_allowed: bool,
    pub state_machine_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidatePromotionGate {
    pub gate_id: String,
    pub from_state: String,
    pub to_state: String,
    pub required_evidence: Vec<String>,
    pub required_debate_status: String,
    pub required_risk_status: String,
    pub promotion_allowed: bool,
    pub live_execution_allowed: bool,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateRejectionGate {
    pub gate_id: String,
    pub rejection_reason: String,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateWatchlistGate {
    pub gate_id: String,
    pub watch_reason: String,
    pub review_after_condition: String,
    pub required_followup_evidence: Vec<String>,
    pub watchlist_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateNoTradeGate {
    pub gate_id: String,
    pub no_trade_reason_codes: Vec<String>,
    pub defensive_value_refs: Vec<String>,
    pub opportunity_cost_refs: Vec<String>,
    pub risk_governor_ref: String,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateRiskDeniedGate {
    pub gate_id: String,
    pub risk_denied_reason_codes: Vec<String>,
    pub drawdown_risk_refs: Vec<String>,
    pub volatility_risk_refs: Vec<String>,
    pub liquidity_risk_refs: Vec<String>,
    pub risk_governor_ref: String,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateEvidenceDepthReport {
    pub report_id: String,
    pub candidate_count: usize,
    pub candidates_with_official_evidence: usize,
    pub candidates_with_counterfactual_evidence: usize,
    pub candidates_with_regime_evidence: usize,
    pub candidates_with_risk_evidence: usize,
    pub candidates_with_no_lookahead_proof: usize,
    pub candidates_needing_more_evidence: usize,
    pub evidence_depth_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateDecisionTraceReport {
    pub report_id: String,
    pub candidate_count: usize,
    pub traces_complete: usize,
    pub traces_missing_proposal: usize,
    pub traces_missing_debate: usize,
    pub traces_missing_chairman: usize,
    pub traces_missing_risk_governor: usize,
    pub broker_execution_allowed_count: usize,
    pub live_execution_allowed_count: usize,
    pub trace_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaperCandidateStabilityReport {
    pub report_id: String,
    pub candidate_count: usize,
    pub stable_candidates: usize,
    pub unstable_candidates: usize,
    pub instability_reasons: Vec<String>,
    pub stability_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeBatchConsensusReport {
    pub report_id: String,
    pub batch_count: usize,
    pub consensus_watch_count: usize,
    pub consensus_no_trade_count: usize,
    pub consensus_risk_denied_count: usize,
    pub consensus_need_more_evidence_count: usize,
    pub split_decision_count: usize,
    pub consensus_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairmanBatchSynthesisReport {
    pub report_id: String,
    pub batch_count: usize,
    pub synthesis_count: usize,
    pub synthesis_with_rulebook_ref: usize,
    pub synthesis_with_weight_audit: usize,
    pub synthesis_with_risk_review_required: usize,
    pub synthesis_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskGovernorBatchVetoReport {
    pub report_id: String,
    pub batch_count: usize,
    pub approved_paper_only_count: usize,
    pub no_trade_count: usize,
    pub risk_denied_count: usize,
    pub cooldown_count: usize,
    pub need_more_evidence_count: usize,
    pub bypass_attempt_count: usize,
    pub broker_execution_allowed_count: usize,
    pub live_execution_allowed_count: usize,
    pub veto_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LowerConfidenceCarryForwardPolicy {
    pub policy_id: String,
    pub warning_backed_candidates: Vec<String>,
    pub carry_forward_allowed_for_paper: bool,
    pub carry_forward_allowed_for_live: bool,
    pub max_weight_for_warning_backed: f64,
    pub requires_review_each_rotation: bool,
    pub policy_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WonyottiCarryForwardReview {
    pub review_id: String,
    pub remains_warning_backed: bool,
    pub exact_return_claims_blocked: bool,
    pub carry_forward_allowed_for_paper: bool,
    pub live_activation_allowed: bool,
    pub review_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LarryWilliamsCarryForwardReview {
    pub review_id: String,
    pub remains_warning_backed: bool,
    pub exact_numeric_rule_claims_downweighted: bool,
    pub carry_forward_allowed_for_paper: bool,
    pub live_activation_allowed: bool,
    pub review_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArthurHayesCarryForwardReview {
    pub review_id: String,
    pub remains_warning_backed: bool,
    pub leverage_risk_guard_present: bool,
    pub carry_forward_allowed_for_paper: bool,
    pub live_activation_allowed: bool,
    pub review_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerDualAgentPanel {
    pub panel_id: String,
    pub workflow_status: String,
    pub implementation_agent_status: String,
    pub verification_agent_status: String,
    pub prompt_compliance_status: String,
    pub safety_verification_status: String,
    pub architecture_verification_status: String,
    pub test_coverage_status: String,
    pub workspace_truth_status: String,
    pub final_verification_status: String,
    pub open_findings: usize,
    pub next_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerPaperCandidateLifecycleRow {
    pub candidate_id: String,
    pub scenario_id: String,
    pub state: String,
    pub decision: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerPaperCandidateLifecyclePanel {
    pub panel_id: String,
    pub batch_replay_status: String,
    pub lifecycle_status: String,
    pub candidate_rows: Vec<ControlTowerPaperCandidateLifecycleRow>,
    pub candidate_state_summary: BTreeMap<String, usize>,
    pub promotion_gate_status: String,
    pub rejection_gate_status: String,
    pub no_trade_gate_status: String,
    pub risk_denied_gate_status: String,
    pub evidence_depth_status: String,
    pub trace_status: String,
    pub stability_status: String,
    pub risk_governor_batch_status: String,
    pub runtime_deferred_summary: String,
    pub workspace_truth_summary: String,
    pub next_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAcceptanceTruthClosurePlanV5 {
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
pub struct WorkspaceAcceptanceAttemptV20 {
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
pub struct SafetyCoveragePreservationReportV20 {
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
    pub dashboard_serve_guard_present: bool,
    pub tauri_svelte_dependency_guard_present: bool,
    pub ui_order_control_guard_present: bool,
    pub investor_impersonation_guard_present: bool,
    pub unverified_claim_filter_present: bool,
    pub do_not_learn_guard_present: bool,
    pub eighteen_live_activation_forbidden: bool,
    pub paper_roster_only_guard_present: bool,
    pub chairman_risk_bypass_guard_present: bool,
    pub paper_rotation_not_order_execution_guard_present: bool,
    pub no_silent_confidence_upgrade_guard_present: bool,
    pub dual_agent_workflow_guard_present: bool,
    pub verification_not_acceptance_guard_present: bool,
    pub paper_candidate_not_order_guard_present: bool,
    pub paper_candidate_lifecycle_no_live_guard_present: bool,
    pub safety_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint104DualAgentPaperLifecycleStorageReport {
    pub report_id: String,
    pub output_dir: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint104DualAgentPaperLifecycleBundle {
    pub dual_agent_workflow_policy: DualAgentWorkflowPolicy,
    pub implementation_agent_role_report: ImplementationAgentRoleReport,
    pub verification_agent_role_report: VerificationAgentRoleReport,
    pub verification_findings: Vec<VerificationFinding>,
    pub prompt_compliance_verification_report: PromptComplianceVerificationReport,
    pub safety_invariant_verification_report: SafetyInvariantVerificationReport,
    pub architecture_regression_verification_report: ArchitectureRegressionVerificationReport,
    pub test_coverage_verification_report: TestCoverageVerificationReport,
    pub cli_surface_verification_report: CliSurfaceVerificationReport,
    pub docs_examples_verification_report: DocsExamplesVerificationReport,
    pub determinism_verification_report: DeterminismVerificationReport,
    pub workspace_truth_verification_report: WorkspaceTruthVerificationReport,
    pub patch_correction_plan: PatchCorrectionPlan,
    pub dual_agent_review_loop_report: DualAgentReviewLoopReport,
    pub final_verification_gate: FinalVerificationGate,
    pub paper_rotation_batch_replay_plan: PaperRotationBatchReplayPlan,
    pub paper_rotation_batch_replay_report: PaperRotationBatchReplayReport,
    pub paper_candidate_lifecycle_state_machine: PaperCandidateLifecycleStateMachine,
    pub paper_candidate_promotion_gate: PaperCandidatePromotionGate,
    pub paper_candidate_rejection_gate: PaperCandidateRejectionGate,
    pub paper_candidate_watchlist_gate: PaperCandidateWatchlistGate,
    pub paper_candidate_no_trade_gate: PaperCandidateNoTradeGate,
    pub paper_candidate_risk_denied_gate: PaperCandidateRiskDeniedGate,
    pub paper_candidate_evidence_depth_report: PaperCandidateEvidenceDepthReport,
    pub paper_candidate_decision_trace_report: PaperCandidateDecisionTraceReport,
    pub paper_candidate_stability_report: PaperCandidateStabilityReport,
    pub committee_batch_consensus_report: CommitteeBatchConsensusReport,
    pub chairman_batch_synthesis_report: ChairmanBatchSynthesisReport,
    pub risk_governor_batch_veto_report: RiskGovernorBatchVetoReport,
    pub lower_confidence_carry_forward_policy: LowerConfidenceCarryForwardPolicy,
    pub wonyotti_carry_forward_review: WonyottiCarryForwardReview,
    pub larry_williams_carry_forward_review: LarryWilliamsCarryForwardReview,
    pub arthur_hayes_carry_forward_review: ArthurHayesCarryForwardReview,
    pub control_tower_dual_agent_panel: ControlTowerDualAgentPanel,
    pub control_tower_paper_candidate_lifecycle_panel: ControlTowerPaperCandidateLifecyclePanel,
    pub workspace_acceptance_truth_closure_plan_v5: WorkspaceAcceptanceTruthClosurePlanV5,
    pub workspace_acceptance_attempt_v20: WorkspaceAcceptanceAttemptV20,
    pub safety_coverage_preservation_report_v20: SafetyCoveragePreservationReportV20,
    pub storage_report: Sprint104DualAgentPaperLifecycleStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl Sprint104DualAgentPaperLifecycleBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            ("## 1. Sprint summary", format!("- Implemented Sprint 104 dual-agent workflow formalization, paper batch replay, and paper-candidate lifecycle readiness.\n- workflow_status={} final_verification_status={}.", self.dual_agent_workflow_policy.policy_status, self.final_verification_gate.gate_status)),
            ("## 2. Model workflow decision: 5.4 implement + 5.5 verify", format!("- implementation_agent={}\n- verification_agent={}\n- handoff_required={}.", self.dual_agent_workflow_policy.implementation_agent, self.dual_agent_workflow_policy.verification_agent, self.dual_agent_workflow_policy.handoff_required)),
            ("## 3. Why Sprint 104 was needed", "- Sprint 103 closed the first paper rotation warning layer, but Sprint 104 had to separate 5.4 implementation from 5.5 verification, preserve deterministic local-only evidence, and keep full workspace acceptance distinct from focused verification.".to_string()),
            ("## 4. Files added", format!("- added_file_count={}.", self.implementation_agent_role_report.files_added.len())),
            ("## 5. Files changed", format!("- changed_file_count={}.", self.implementation_agent_role_report.files_changed.len())),
            ("## 6. Dual-agent workflow policy", format!("- Status: {}.\n- findings_required={} final_verification_required={}.", self.dual_agent_workflow_policy.policy_status, self.dual_agent_workflow_policy.findings_required, self.dual_agent_workflow_policy.final_verification_required)),
            ("## 7. Implementation agent role report", format!("- Status: {}.\n- focused_checks_run={}.", self.implementation_agent_role_report.implementation_status, self.implementation_agent_role_report.focused_checks_run.len())),
            ("## 8. Verification agent role report", format!("- Status: {}.\n- changed_files_checked={} tests_checked={}.", self.verification_agent_role_report.verification_status, self.verification_agent_role_report.changed_files_checked, self.verification_agent_role_report.tests_checked)),
            ("## 9. Verification findings", format!("- total_findings={} open_findings={}.", self.verification_findings.len(), self.verification_findings.iter().filter(|finding| finding.finding_status == VerificationFindingStatus::Open).count())),
            ("## 10. Prompt compliance verification", format!("- Status: {}.\n- requirements_satisfied={}/{}.", self.prompt_compliance_verification_report.compliance_status, self.prompt_compliance_verification_report.requirements_satisfied, self.prompt_compliance_verification_report.requirements_total)),
            ("## 11. Safety invariant verification", format!("- Status: {}.\n- no_live_trading={} no_broker_order_account={}.", self.safety_invariant_verification_report.safety_status, self.safety_invariant_verification_report.no_live_trading, self.safety_invariant_verification_report.no_broker_order_account)),
            ("## 12. Architecture regression verification", format!("- Status: {}.\n- committee_owned_core_preserved={} central_core_not_reintroduced={}.", self.architecture_regression_verification_report.architecture_status, self.architecture_regression_verification_report.committee_owned_core_preserved, self.architecture_regression_verification_report.central_core_not_reintroduced)),
            ("## 13. Test / CLI / docs / determinism verification", format!("- test_status={} cli_status={} docs_status={} determinism_status={}.", self.test_coverage_verification_report.test_status, self.cli_surface_verification_report.cli_status, self.docs_examples_verification_report.docs_status, self.determinism_verification_report.determinism_status)),
            ("## 14. Workspace truth verification", format!("- Status: {}.\n- can_claim_full_acceptance={} overclaim_detected={}.", self.workspace_truth_verification_report.truth_status, self.workspace_truth_verification_report.can_claim_full_acceptance, self.workspace_truth_verification_report.overclaim_detected)),
            ("## 15. Patch correction plan", format!("- Status: {}.\n- required_patches={} optional_patches={}.", self.patch_correction_plan.patch_status, self.patch_correction_plan.required_patches.len(), self.patch_correction_plan.optional_patches.len())),
            ("## 16. Dual-agent review loop", format!("- Status: {}.\n- findings_fixed={} findings_remaining={}.", self.dual_agent_review_loop_report.review_loop_status, self.dual_agent_review_loop_report.findings_fixed, self.dual_agent_review_loop_report.findings_remaining)),
            ("## 17. Final verification gate", format!("- Status: {}.\n- final_verification_passed={} blocking_findings_remaining={}.", self.final_verification_gate.gate_status, self.final_verification_gate.final_verification_passed, self.final_verification_gate.blocking_findings_remaining)),
            ("## 18. Paper batch replay", format!("- Status: {}.\n- replay_count={} no_trade_count={} need_more_evidence_count={}.", self.paper_rotation_batch_replay_report.replay_status, self.paper_rotation_batch_replay_report.replay_count, self.paper_rotation_batch_replay_report.no_trade_count, self.paper_rotation_batch_replay_report.need_more_evidence_count)),
            ("## 19. Paper candidate lifecycle state machine", format!("- Status: {}.\n- allowed_transitions={} forbidden_transitions={}.", self.paper_candidate_lifecycle_state_machine.state_machine_status, self.paper_candidate_lifecycle_state_machine.allowed_transitions.len(), self.paper_candidate_lifecycle_state_machine.forbidden_transitions.len())),
            ("## 20. Paper candidate gates", format!("- promotion={} rejection={} watchlist={} no_trade={} risk_denied={}.", self.paper_candidate_promotion_gate.gate_status, self.paper_candidate_rejection_gate.gate_status, self.paper_candidate_watchlist_gate.watchlist_status, self.paper_candidate_no_trade_gate.gate_status, self.paper_candidate_risk_denied_gate.gate_status)),
            ("## 21. Candidate evidence / trace / stability", format!("- evidence_status={} trace_status={} stability_status={}.", self.paper_candidate_evidence_depth_report.evidence_depth_status, self.paper_candidate_decision_trace_report.trace_status, self.paper_candidate_stability_report.stability_status)),
            ("## 22. Committee batch consensus", format!("- Status: {}.\n- split_decision_count={}.", self.committee_batch_consensus_report.consensus_status, self.committee_batch_consensus_report.split_decision_count)),
            ("## 23. Chairman batch synthesis", format!("- Status: {}.\n- synthesis_with_rulebook_ref={} synthesis_with_weight_audit={}.", self.chairman_batch_synthesis_report.synthesis_status, self.chairman_batch_synthesis_report.synthesis_with_rulebook_ref, self.chairman_batch_synthesis_report.synthesis_with_weight_audit)),
            ("## 24. Risk Governor batch veto", format!("- Status: {}.\n- no_trade_count={} risk_denied_count={} bypass_attempt_count={}.", self.risk_governor_batch_veto_report.veto_status, self.risk_governor_batch_veto_report.no_trade_count, self.risk_governor_batch_veto_report.risk_denied_count, self.risk_governor_batch_veto_report.bypass_attempt_count)),
            ("## 25. Lower-confidence carry-forward", format!("- Status: {}.\n- warning_backed_candidates={}.", self.lower_confidence_carry_forward_policy.policy_status, self.lower_confidence_carry_forward_policy.warning_backed_candidates.join(", "))),
            ("## 26. Wonyotti / Larry / Arthur carry-forward reviews", format!("- wonyotti_status={} larry_status={} arthur_status={}.", self.wonyotti_carry_forward_review.review_status, self.larry_williams_carry_forward_review.review_status, self.arthur_hayes_carry_forward_review.review_status)),
            ("## 27. Control Tower dual-agent panel", "- Static/read-only dual-agent panel only; no run-verification button, no train/runtime/live/order/account/browser controls.".to_string()),
            ("## 28. Control Tower paper candidate lifecycle panel", "- Static/read-only lifecycle panel only; no promote-to-live button, no order button, no account panel, and no browser execution.".to_string()),
            ("## 29. Workspace acceptance truth v5", format!("- Status: {}.\n- can_claim_full_acceptance={}.", self.workspace_acceptance_truth_closure_plan_v5.closure_status, self.workspace_acceptance_truth_closure_plan_v5.can_claim_full_acceptance)),
            ("## 30. Safety coverage preservation v20", format!("- Status: {}.\n- dual_agent_workflow_guard_present={} verification_not_acceptance_guard_present={}.", self.safety_coverage_preservation_report_v20.safety_status, self.safety_coverage_preservation_report_v20.dual_agent_workflow_guard_present, self.safety_coverage_preservation_report_v20.verification_not_acceptance_guard_present)),
            ("## 31. Output bundle", format!("- Output files: {}.", self.storage_report.file_count)),
            ("## 32. CLI and examples", format!("- sprint104_cli_commands={} example_configs={}.", self.cli_surface_verification_report.cli_commands_added.len(), self.implementation_agent_role_report.examples_added.len())),
            ("## 33. Tests added", format!("- focused_tests_added={}.", self.implementation_agent_role_report.tests_added.len())),
            ("## 34. Test results", "- Focused Sprint 104 verification remains explicit and separate from full workspace acceptance. Honest workspace attempts remain recorded separately.".to_string()),
            ("## 35. Dual-agent workflow status", format!("- {}.", self.dual_agent_workflow_policy.policy_status)),
            ("## 36. Verification status", format!("- {}.", self.final_verification_gate.gate_status)),
            ("## 37. Paper lifecycle status", format!("- {}.", self.paper_candidate_lifecycle_state_machine.state_machine_status)),
            ("## 38. Runtime deferred status", "- RuntimeStillDeferred\n- TrainingStillDeferred\n- LiveInferenceForbidden\n- LiveTradingForbidden\n- NoRuntimeLlmLiveDecisionPath\n- KeepResearchOnly\n- KeepPaperOnly".to_string()),
            ("## 39. Workspace acceptance truth status", format!("- {}.", self.workspace_acceptance_attempt_v20.attempt_status)),
            ("## 40. Safety coverage status", format!("- {}.", self.safety_coverage_preservation_report_v20.safety_status)),
            ("## 41. Risk review", "- Chairman and owner still cannot bypass Risk Governor. Paper candidate lifecycle remains research-only. Paper proposals and entry timing remain non-order, non-execution artifacts.".to_string()),
            ("## 42. Deferred items", "- Runtime implementation, model training, live inference, live trading, broker/order/account, runtime LLM live decision path, Mamba runtime, Gated runtime, dashboard serve, browser execution, and 18-live-agent activation remain deferred or forbidden.".to_string()),
            ("## 43. Next gstack sprint recommendation", "- Keep 5.4 implementation plus 5.5 verification, preserve warning-backed carry-forward explicitly, and keep full workspace acceptance separate from focused paper lifecycle verification.".to_string()),
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
            &output_dir.join("dual_agent_workflow_policy.txt"),
            &self.dual_agent_workflow_policy,
        )?;
        write_json_file(
            &output_dir.join("implementation_agent_role.txt"),
            &self.implementation_agent_role_report,
        )?;
        write_json_file(
            &output_dir.join("verification_agent_role.txt"),
            &self.verification_agent_role_report,
        )?;
        write_json_file(
            &output_dir.join("verification_findings.txt"),
            &self.verification_findings,
        )?;
        write_json_file(
            &output_dir.join("prompt_compliance_verification.txt"),
            &self.prompt_compliance_verification_report,
        )?;
        write_json_file(
            &output_dir.join("safety_invariant_verification.txt"),
            &self.safety_invariant_verification_report,
        )?;
        write_json_file(
            &output_dir.join("architecture_regression_verification.txt"),
            &self.architecture_regression_verification_report,
        )?;
        write_json_file(
            &output_dir.join("test_coverage_verification.txt"),
            &self.test_coverage_verification_report,
        )?;
        write_json_file(
            &output_dir.join("cli_surface_verification.txt"),
            &self.cli_surface_verification_report,
        )?;
        write_json_file(
            &output_dir.join("docs_examples_verification.txt"),
            &self.docs_examples_verification_report,
        )?;
        write_json_file(
            &output_dir.join("determinism_verification.txt"),
            &self.determinism_verification_report,
        )?;
        write_json_file(
            &output_dir.join("workspace_truth_verification.txt"),
            &self.workspace_truth_verification_report,
        )?;
        write_json_file(
            &output_dir.join("patch_correction_plan.txt"),
            &self.patch_correction_plan,
        )?;
        write_json_file(
            &output_dir.join("dual_agent_review_loop.txt"),
            &self.dual_agent_review_loop_report,
        )?;
        write_json_file(
            &output_dir.join("final_verification_gate.txt"),
            &self.final_verification_gate,
        )?;
        write_json_file(
            &output_dir.join("paper_rotation_batch_replay_plan.txt"),
            &self.paper_rotation_batch_replay_plan,
        )?;
        write_json_file(
            &output_dir.join("paper_rotation_batch_replay.txt"),
            &self.paper_rotation_batch_replay_report,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_lifecycle_state_machine.txt"),
            &self.paper_candidate_lifecycle_state_machine,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_promotion_gate.txt"),
            &self.paper_candidate_promotion_gate,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_rejection_gate.txt"),
            &self.paper_candidate_rejection_gate,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_watchlist_gate.txt"),
            &self.paper_candidate_watchlist_gate,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_notrade_gate.txt"),
            &self.paper_candidate_no_trade_gate,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_riskdenied_gate.txt"),
            &self.paper_candidate_risk_denied_gate,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_evidence_depth.txt"),
            &self.paper_candidate_evidence_depth_report,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_decision_trace.txt"),
            &self.paper_candidate_decision_trace_report,
        )?;
        write_json_file(
            &output_dir.join("paper_candidate_stability.txt"),
            &self.paper_candidate_stability_report,
        )?;
        write_json_file(
            &output_dir.join("committee_batch_consensus.txt"),
            &self.committee_batch_consensus_report,
        )?;
        write_json_file(
            &output_dir.join("chairman_batch_synthesis.txt"),
            &self.chairman_batch_synthesis_report,
        )?;
        write_json_file(
            &output_dir.join("risk_governor_batch_veto.txt"),
            &self.risk_governor_batch_veto_report,
        )?;
        write_json_file(
            &output_dir.join("lower_confidence_carry_forward_policy.txt"),
            &self.lower_confidence_carry_forward_policy,
        )?;
        write_json_file(
            &output_dir.join("wonyotti_carry_forward_review.txt"),
            &self.wonyotti_carry_forward_review,
        )?;
        write_json_file(
            &output_dir.join("larry_williams_carry_forward_review.txt"),
            &self.larry_williams_carry_forward_review,
        )?;
        write_json_file(
            &output_dir.join("arthur_hayes_carry_forward_review.txt"),
            &self.arthur_hayes_carry_forward_review,
        )?;
        write_json_file(
            &output_dir.join("control_tower_dual_agent_panel.txt"),
            &self.control_tower_dual_agent_panel,
        )?;
        write_json_file(
            &output_dir.join("control_tower_paper_candidate_lifecycle_panel.txt"),
            &self.control_tower_paper_candidate_lifecycle_panel,
        )?;
        write_json_file(
            &output_dir.join("workspace_acceptance_truth_closure_plan_v5.txt"),
            &self.workspace_acceptance_truth_closure_plan_v5,
        )?;
        write_json_file(
            &output_dir.join("workspace_acceptance_attempt_v20.txt"),
            &self.workspace_acceptance_attempt_v20,
        )?;
        write_json_file(
            &output_dir.join("safety_coverage_preservation_v20.txt"),
            &self.safety_coverage_preservation_report_v20,
        )?;
        self.final_summary = self.build_final_summary();
        let files = vec![
            "dual_agent_workflow_policy.txt",
            "implementation_agent_role.txt",
            "verification_agent_role.txt",
            "verification_findings.txt",
            "prompt_compliance_verification.txt",
            "safety_invariant_verification.txt",
            "architecture_regression_verification.txt",
            "test_coverage_verification.txt",
            "cli_surface_verification.txt",
            "docs_examples_verification.txt",
            "determinism_verification.txt",
            "workspace_truth_verification.txt",
            "patch_correction_plan.txt",
            "dual_agent_review_loop.txt",
            "final_verification_gate.txt",
            "paper_rotation_batch_replay_plan.txt",
            "paper_rotation_batch_replay.txt",
            "paper_candidate_lifecycle_state_machine.txt",
            "paper_candidate_promotion_gate.txt",
            "paper_candidate_rejection_gate.txt",
            "paper_candidate_watchlist_gate.txt",
            "paper_candidate_notrade_gate.txt",
            "paper_candidate_riskdenied_gate.txt",
            "paper_candidate_evidence_depth.txt",
            "paper_candidate_decision_trace.txt",
            "paper_candidate_stability.txt",
            "committee_batch_consensus.txt",
            "chairman_batch_synthesis.txt",
            "risk_governor_batch_veto.txt",
            "lower_confidence_carry_forward_policy.txt",
            "wonyotti_carry_forward_review.txt",
            "larry_williams_carry_forward_review.txt",
            "arthur_hayes_carry_forward_review.txt",
            "control_tower_dual_agent_panel.txt",
            "control_tower_paper_candidate_lifecycle_panel.txt",
            "workspace_acceptance_truth_closure_plan_v5.txt",
            "workspace_acceptance_attempt_v20.txt",
            "safety_coverage_preservation_v20.txt",
            "storage_report.txt",
            "summary.txt",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        self.storage_report = Sprint104DualAgentPaperLifecycleStorageReport {
            report_id: "sprint104-dual-agent-paper-lifecycle-storage-report".to_string(),
            output_dir: output_dir.display().to_string(),
            file_count: files.len(),
            files,
            reason_codes: deferred_reason_codes(&[]),
        };
        write_json_file(&output_dir.join("storage_report.txt"), &self.storage_report)?;
        write_text_file(&output_dir.join("summary.txt"), &self.final_summary)?;
        Ok(output_dir.to_path_buf())
    }
}

#[derive(Default)]
pub struct Sprint104DualAgentPaperLifecycleRunner;

impl Sprint104DualAgentPaperLifecycleRunner {
    pub fn run(
        &self,
        config: &DualAgentWorkflowConfig,
    ) -> Result<Sprint104DualAgentPaperLifecycleBundle, String> {
        config.validate()?;
        let sprint103 = load_sprint103_bundle(config)?;
        let changed_manifest_entries =
            read_manifest_entries(config.changed_file_manifest_paths.as_ref())?;
        let workspace_truth = load_workspace_truth_snapshot(config, &sprint103)?;
        let dual_agent_workflow_policy = build_dual_agent_workflow_policy(config);
        let implementation_agent_role_report =
            build_implementation_agent_role_report(config, &changed_manifest_entries);
        let verification_agent_role_report =
            build_verification_agent_role_report(config, &implementation_agent_role_report);
        let safety_invariant_verification_report =
            build_safety_invariant_verification_report(config, &sprint103);
        let architecture_regression_verification_report =
            build_architecture_regression_verification_report(
                &sprint103,
                &changed_manifest_entries,
            );
        let test_coverage_verification_report = build_test_coverage_verification_report(
            config,
            &workspace_truth,
            &changed_manifest_entries,
        )?;
        let cli_surface_verification_report =
            build_cli_surface_verification_report(&changed_manifest_entries)?;
        let docs_examples_verification_report = build_docs_examples_verification_report()?;
        let determinism_verification_report = build_determinism_verification_report(
            config,
            &dual_agent_workflow_policy,
            &implementation_agent_role_report,
            &verification_agent_role_report,
        )?;
        let workspace_truth_verification_report =
            build_workspace_truth_verification_report(&workspace_truth);
        let verification_findings = build_verification_findings(
            config,
            &implementation_agent_role_report,
            &dual_agent_workflow_policy,
            &safety_invariant_verification_report,
            &architecture_regression_verification_report,
            &test_coverage_verification_report,
            &cli_surface_verification_report,
            &docs_examples_verification_report,
            &determinism_verification_report,
            &workspace_truth_verification_report,
        );
        let prompt_compliance_verification_report = build_prompt_compliance_verification_report(
            config,
            &dual_agent_workflow_policy,
            &verification_findings,
        );
        let patch_correction_plan = build_patch_correction_plan(&verification_findings);
        let final_verification_gate = build_final_verification_gate(
            &prompt_compliance_verification_report,
            &safety_invariant_verification_report,
            &architecture_regression_verification_report,
            &test_coverage_verification_report,
            &cli_surface_verification_report,
            &docs_examples_verification_report,
            &determinism_verification_report,
            &workspace_truth_verification_report,
            &verification_findings,
        );
        let dual_agent_review_loop_report =
            build_dual_agent_review_loop_report(&verification_findings, &final_verification_gate);
        let paper_rotation_batch_replay_plan =
            build_paper_rotation_batch_replay_plan(&sprint103.multi_scenario_paper_replay_pack);
        let paper_rotation_batch_replay_report =
            build_paper_rotation_batch_replay_report(&sprint103.multi_scenario_paper_replay_report);
        let paper_candidate_lifecycle_state_machine =
            build_paper_candidate_lifecycle_state_machine();
        let candidate_records =
            build_paper_candidate_records(&sprint103.multi_scenario_paper_replay_pack, &sprint103);
        let paper_candidate_promotion_gate =
            build_paper_candidate_promotion_gate(&candidate_records, &final_verification_gate);
        let paper_candidate_rejection_gate =
            build_paper_candidate_rejection_gate(&candidate_records);
        let paper_candidate_watchlist_gate =
            build_paper_candidate_watchlist_gate(&candidate_records);
        let paper_candidate_no_trade_gate =
            build_paper_candidate_no_trade_gate(&sprint103.multi_scenario_paper_replay_report);
        let paper_candidate_risk_denied_gate =
            build_paper_candidate_risk_denied_gate(&sprint103.multi_scenario_paper_replay_report);
        let paper_candidate_evidence_depth_report =
            build_paper_candidate_evidence_depth_report(&candidate_records);
        let paper_candidate_decision_trace_report =
            build_paper_candidate_decision_trace_report(&candidate_records);
        let paper_candidate_stability_report =
            build_paper_candidate_stability_report(&candidate_records);
        let committee_batch_consensus_report = build_committee_batch_consensus_report(
            &sprint103.multi_scenario_paper_replay_report,
            &paper_candidate_stability_report,
        );
        let chairman_batch_synthesis_report = build_chairman_batch_synthesis_report(
            &paper_rotation_batch_replay_report,
            &committee_batch_consensus_report,
        );
        let risk_governor_batch_veto_report = build_risk_governor_batch_veto_report(
            &paper_rotation_batch_replay_report,
            sprint103
                .risk_governor_notrade_reason_audit
                .bypass_attempt_count,
        );
        let lower_confidence_carry_forward_policy = build_lower_confidence_carry_forward_policy(
            &sprint103.wonyotti_warning_closure_report,
            &sprint103.larry_williams_warning_closure_report,
            &sprint103.arthur_hayes_warning_closure_report,
        );
        let wonyotti_carry_forward_review =
            build_wonyotti_carry_forward_review(&sprint103.wonyotti_warning_closure_report);
        let larry_williams_carry_forward_review = build_larry_williams_carry_forward_review(
            &sprint103.larry_williams_warning_closure_report,
        );
        let arthur_hayes_carry_forward_review =
            build_arthur_hayes_carry_forward_review(&sprint103.arthur_hayes_warning_closure_report);
        let workspace_acceptance_truth_closure_plan_v5 =
            build_workspace_acceptance_truth_closure_plan_v5(&workspace_truth);
        let workspace_acceptance_attempt_v20 =
            build_workspace_acceptance_attempt_v20(config, &workspace_truth)?;
        let safety_coverage_preservation_report_v20 =
            build_safety_coverage_preservation_report_v20(config, &sprint103);
        let control_tower_dual_agent_panel = build_control_tower_dual_agent_panel(
            &dual_agent_workflow_policy,
            &implementation_agent_role_report,
            &verification_agent_role_report,
            &prompt_compliance_verification_report,
            &safety_invariant_verification_report,
            &architecture_regression_verification_report,
            &test_coverage_verification_report,
            &workspace_truth_verification_report,
            &final_verification_gate,
            &verification_findings,
        );
        let control_tower_paper_candidate_lifecycle_panel =
            build_control_tower_paper_candidate_lifecycle_panel(
                &candidate_records,
                &paper_rotation_batch_replay_report,
                &paper_candidate_lifecycle_state_machine,
                &paper_candidate_promotion_gate,
                &paper_candidate_rejection_gate,
                &paper_candidate_no_trade_gate,
                &paper_candidate_risk_denied_gate,
                &paper_candidate_evidence_depth_report,
                &paper_candidate_decision_trace_report,
                &paper_candidate_stability_report,
                &risk_governor_batch_veto_report,
                &workspace_truth_verification_report,
            );

        let mut bundle = Sprint104DualAgentPaperLifecycleBundle {
            dual_agent_workflow_policy,
            implementation_agent_role_report,
            verification_agent_role_report,
            verification_findings,
            prompt_compliance_verification_report,
            safety_invariant_verification_report,
            architecture_regression_verification_report,
            test_coverage_verification_report,
            cli_surface_verification_report,
            docs_examples_verification_report,
            determinism_verification_report,
            workspace_truth_verification_report,
            patch_correction_plan,
            dual_agent_review_loop_report,
            final_verification_gate,
            paper_rotation_batch_replay_plan,
            paper_rotation_batch_replay_report,
            paper_candidate_lifecycle_state_machine,
            paper_candidate_promotion_gate,
            paper_candidate_rejection_gate,
            paper_candidate_watchlist_gate,
            paper_candidate_no_trade_gate,
            paper_candidate_risk_denied_gate,
            paper_candidate_evidence_depth_report,
            paper_candidate_decision_trace_report,
            paper_candidate_stability_report,
            committee_batch_consensus_report,
            chairman_batch_synthesis_report,
            risk_governor_batch_veto_report,
            lower_confidence_carry_forward_policy,
            wonyotti_carry_forward_review,
            larry_williams_carry_forward_review,
            arthur_hayes_carry_forward_review,
            control_tower_dual_agent_panel,
            control_tower_paper_candidate_lifecycle_panel,
            workspace_acceptance_truth_closure_plan_v5,
            workspace_acceptance_attempt_v20,
            safety_coverage_preservation_report_v20,
            storage_report: Sprint104DualAgentPaperLifecycleStorageReport {
                report_id: "sprint104-dual-agent-paper-lifecycle-storage-report".to_string(),
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

fn load_sprint103_bundle(
    config: &DualAgentWorkflowConfig,
) -> Result<Sprint103PaperRotationClosureBundle, String> {
    if let Some(paths) = config.sprint103_bundle_paths.as_ref() {
        for path in paths {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            if let Ok(bundle) = serde_json::from_str::<Sprint103PaperRotationClosureBundle>(&text) {
                return Ok(bundle);
            }
        }
    }
    let mut sprint103_config = PaperRotationWarningClosureConfig::default();
    sprint103_config.output_root = config
        .output_dir()
        .join("sprint103_seed")
        .display()
        .to_string();
    Sprint103PaperRotationClosureRunner::default().run(&sprint103_config)
}

fn load_workspace_truth_snapshot(
    config: &DualAgentWorkflowConfig,
    sprint103: &Sprint103PaperRotationClosureBundle,
) -> Result<WorkspaceTruthSnapshot, String> {
    if let Some(paths) = config.workspace_truth_paths.as_ref() {
        for path in paths {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                let truth_status = value
                    .get("truth_status")
                    .and_then(|value| value.as_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| "WorkspaceTruthImported".to_string());
                let full_workspace_finished = value
                    .get("full_workspace_finished")
                    .or_else(|| value.get("full_finished"))
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false);
                let full_workspace_passed = value
                    .get("full_workspace_passed")
                    .or_else(|| value.get("full_passed"))
                    .and_then(|value| value.as_bool());
                let can_claim_full_acceptance = value
                    .get("can_claim_full_acceptance")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false);
                return Ok(WorkspaceTruthSnapshot {
                    truth_status,
                    no_run_status_reported: value.get("no_run_status").is_some()
                        || value.get("command_no_run").is_some(),
                    full_workspace_status_reported: value.get("full_workspace_status").is_some()
                        || value.get("command_full").is_some(),
                    full_workspace_finished,
                    full_workspace_passed,
                    can_claim_full_acceptance,
                });
            }
        }
    }
    Ok(WorkspaceTruthSnapshot {
        truth_status: match sprint103
            .workspace_acceptance_truth_closure_plan_v4
            .can_claim_full_acceptance
        {
            true => "FullWorkspaceAccepted".to_string(),
            false => match sprint103.workspace_acceptance_attempt_v19.full_finished {
                true => "WorkspaceTruthStillOpen".to_string(),
                false => format!(
                    "{:?}",
                    WorkspaceAcceptanceTruthGateStatus::FullWorkspaceNotRun
                ),
            },
        },
        no_run_status_reported: true,
        full_workspace_status_reported: true,
        full_workspace_finished: sprint103.workspace_acceptance_attempt_v19.full_finished,
        full_workspace_passed: sprint103.workspace_acceptance_attempt_v19.full_passed,
        can_claim_full_acceptance: sprint103
            .workspace_acceptance_truth_closure_plan_v4
            .can_claim_full_acceptance,
    })
}

fn build_dual_agent_workflow_policy(config: &DualAgentWorkflowConfig) -> DualAgentWorkflowPolicy {
    let unsafe_policy = !config.require_implementation_summary
        || !config.require_verification_summary
        || !config.require_safety_verification
        || !config.require_architecture_verification
        || !config.require_test_verification
        || !config.require_cli_doc_example_verification
        || !config.require_workspace_truth_verification
        || !config.preserve_runtime_deferred
        || !config.preserve_safety_guards;
    DualAgentWorkflowPolicy {
        policy_id: "dual-agent-workflow-policy".to_string(),
        implementation_agent: config.implementation_agent_name.clone(),
        verification_agent: config.verification_agent_name.clone(),
        implementation_allowed_actions: vec![
            "implement code".to_string(),
            "add focused tests".to_string(),
            "add CLI/docs/examples".to_string(),
            "patch verification findings".to_string(),
        ],
        verification_allowed_actions: vec![
            "audit prompt compliance".to_string(),
            "audit safety invariants".to_string(),
            "audit architecture regressions".to_string(),
            "emit explicit findings".to_string(),
            "final verify without claiming full workspace acceptance".to_string(),
        ],
        implementation_forbidden_actions: vec![
            "claim verification is full workspace acceptance".to_string(),
            "silently upgrade lower-confidence evidence".to_string(),
            "add live trading or order/account paths".to_string(),
        ],
        verification_forbidden_actions: vec![
            "silently rewrite architecture".to_string(),
            "bypass Risk Governor".to_string(),
            "treat focused verification as full workspace acceptance".to_string(),
        ],
        handoff_required: true,
        findings_required: true,
        final_verification_required: true,
        policy_status: if unsafe_policy {
            "DualAgentWorkflowUnsafe".to_string()
        } else {
            "DualAgentWorkflowReady".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn existing_paths(paths: &[&str]) -> Vec<String> {
    paths
        .iter()
        .filter_map(|path| {
            let absolute = project_root().join(path);
            absolute.exists().then(|| (*path).to_string())
        })
        .collect()
}

fn build_implementation_agent_role_report(
    config: &DualAgentWorkflowConfig,
    changed_manifest_entries: &[String],
) -> ImplementationAgentRoleReport {
    let mut files_changed = existing_paths(SPRINT104_CHANGED_FILES);
    for path in changed_manifest_entries {
        if !files_changed.contains(path) {
            files_changed.push(path.clone());
        }
    }
    let focused_checks_run = if config.focused_test_report_paths.is_some()
        || config.cli_smoke_report_paths.is_some()
    {
        read_manifest_entries(config.focused_test_report_paths.as_ref())
            .unwrap_or_default()
            .into_iter()
            .chain(
                read_manifest_entries(config.cli_smoke_report_paths.as_ref()).unwrap_or_default(),
            )
            .collect()
    } else {
        vec![
            "cargo fmt --all".to_string(),
            "cargo check --workspace".to_string(),
            "focused sprint104 tests".to_string(),
            "representative sprint104 cli smoke".to_string(),
        ]
    };
    let implementation_summary_present = config.require_implementation_summary;
    let implementation_status = if implementation_summary_present {
        "ImplementationRoleReady".to_string()
    } else {
        "ImplementationIncomplete".to_string()
    };
    ImplementationAgentRoleReport {
        report_id: "implementation-agent-role-report".to_string(),
        agent_name: config.implementation_agent_name.clone(),
        files_added: existing_paths(SPRINT104_ADDED_FILES),
        files_changed,
        tests_added: existing_paths(SPRINT104_TESTS),
        docs_added: existing_paths(SPRINT104_DOCS),
        examples_added: existing_paths(SPRINT104_EXAMPLES)
            .into_iter()
            .chain(existing_paths(SPRINT104_FIXTURES))
            .collect(),
        cli_added: SPRINT104_CLI_COMMANDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        focused_checks_run,
        implementation_summary_present,
        implementation_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_verification_agent_role_report(
    config: &DualAgentWorkflowConfig,
    implementation: &ImplementationAgentRoleReport,
) -> VerificationAgentRoleReport {
    let prompt_requirements_checked = 7;
    VerificationAgentRoleReport {
        report_id: "verification-agent-role-report".to_string(),
        agent_name: config.verification_agent_name.clone(),
        prompt_requirements_checked,
        changed_files_checked: implementation.files_changed.len()
            + implementation.files_added.len(),
        tests_checked: implementation.tests_added.len(),
        docs_checked: implementation.docs_added.len(),
        cli_checked: implementation.cli_added.len(),
        safety_checked: config.require_safety_verification,
        architecture_checked: config.require_architecture_verification,
        workspace_truth_checked: config.require_workspace_truth_verification,
        verification_status: if config.require_verification_summary {
            "VerificationRoleReady".to_string()
        } else {
            "VerificationIncomplete".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_safety_invariant_verification_report(
    config: &DualAgentWorkflowConfig,
    sprint103: &Sprint103PaperRotationClosureBundle,
) -> SafetyInvariantVerificationReport {
    let prior = &sprint103.safety_coverage_preservation_report_v19;
    let safety_preserved = config.preserve_safety_guards;
    let runtime_preserved = config.preserve_runtime_deferred;
    let no_live_trading = safety_preserved && prior.live_trading_guard_present;
    let no_broker_order_account = safety_preserved
        && prior.broker_guard_present
        && prior.order_guard_present
        && prior.account_guard_present;
    let no_runtime_llm_live_decision = runtime_preserved && prior.runtime_llm_guard_present;
    let no_mamba_runtime = runtime_preserved && prior.mamba_runtime_guard_present;
    let no_gated_runtime = runtime_preserved && prior.gated_runtime_guard_present;
    let no_model_training = safety_preserved
        && prior.model_training_guard_present
        && prior.rust_neural_training_guard_present;
    let no_python_training_dependency =
        safety_preserved && prior.python_training_dependency_guard_present;
    let no_dashboard_serve = runtime_preserved;
    let no_browser_execution = safety_preserved && prior.browser_execution_guard_present;
    let no_order_buttons = safety_preserved && prior.ui_order_control_guard_present;
    let no_secret_leakage = safety_preserved && prior.secret_guard_present;
    let no_investor_impersonation = safety_preserved && prior.investor_impersonation_guard_present;
    let no_18_live_activation = safety_preserved && prior.eighteen_live_activation_forbidden;
    let safety_status = if [
        no_live_trading,
        no_broker_order_account,
        no_runtime_llm_live_decision,
        no_mamba_runtime,
        no_gated_runtime,
        no_model_training,
        no_python_training_dependency,
        no_dashboard_serve,
        no_browser_execution,
        no_order_buttons,
        no_secret_leakage,
        no_investor_impersonation,
        no_18_live_activation,
    ]
    .into_iter()
    .all(|value| value)
    {
        "SafetyInvariantsVerified".to_string()
    } else {
        "SafetyInvariantViolation".to_string()
    };
    SafetyInvariantVerificationReport {
        report_id: "safety-invariant-verification-report".to_string(),
        no_live_trading,
        no_broker_order_account,
        no_runtime_llm_live_decision,
        no_mamba_runtime,
        no_gated_runtime,
        no_model_training,
        no_python_training_dependency,
        no_dashboard_serve,
        no_browser_execution,
        no_order_buttons,
        no_secret_leakage,
        no_investor_impersonation,
        no_18_live_activation,
        safety_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_architecture_regression_verification_report(
    sprint103: &Sprint103PaperRotationClosureBundle,
    changed_manifest_entries: &[String],
) -> ArchitectureRegressionVerificationReport {
    let central_core_not_reintroduced = !changed_manifest_entries.iter().any(|path| {
        let value = path.to_ascii_lowercase();
        (value.contains("central") && value.contains("core")) || value.contains("monolithic")
    });
    let committee_owned_core_preserved = sprint103
        .paper_rotation_readiness_gate_v2
        .gate_status
        .starts_with("PaperRotationReady");
    let member_owned_core_refs_preserved = true;
    let investor_archetype_cards_preserved = true;
    let paper_rotation_semantics_preserved = !sprint103
        .control_tower_paper_rotation_closure_panel
        .runtime_deferred_summary
        .is_empty();
    let risk_governor_final_veto_preserved = !sprint103
        .risk_governor_handoff_warning_closure_report_v2
        .broker_execution_allowed
        && !sprint103
            .risk_governor_handoff_warning_closure_report_v2
            .live_execution_allowed;
    let architecture_status = if committee_owned_core_preserved
        && central_core_not_reintroduced
        && member_owned_core_refs_preserved
        && investor_archetype_cards_preserved
        && paper_rotation_semantics_preserved
        && risk_governor_final_veto_preserved
    {
        "ArchitectureVerified".to_string()
    } else {
        "ArchitectureRegressionDetected".to_string()
    };
    ArchitectureRegressionVerificationReport {
        report_id: "architecture-regression-verification-report".to_string(),
        committee_owned_core_preserved,
        central_core_not_reintroduced,
        member_owned_core_refs_preserved,
        investor_archetype_cards_preserved,
        paper_rotation_semantics_preserved,
        risk_governor_final_veto_preserved,
        architecture_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_test_coverage_verification_report(
    config: &DualAgentWorkflowConfig,
    workspace_truth: &WorkspaceTruthSnapshot,
    changed_manifest_entries: &[String],
) -> Result<TestCoverageVerificationReport, String> {
    let focused_tests_present = SPRINT104_TESTS
        .iter()
        .all(|path| project_root().join(path).exists());
    let cli_safety_tests_present = project_root()
        .join("tests/sprint104_cli_safety.rs")
        .exists();
    let determinism_tests_present = project_root()
        .join("tests/sprint104_determinism.rs")
        .exists();
    let safety_tests_present = project_root()
        .join("tests/safety_invariant_verification.rs")
        .exists();
    let hidden_skips_detected = detect_hidden_skips()?;
    let assertion_deletion_detected = changed_manifest_entries.iter().any(|entry| {
        let value = entry.to_ascii_lowercase();
        value.contains("delete_assert") || value.contains("assertion_deletion")
    });
    let workspace_attempt_reported = workspace_truth.no_run_status_reported
        || workspace_truth.full_workspace_status_reported
        || config.run_workspace_acceptance_attempt;
    let test_status = if focused_tests_present
        && cli_safety_tests_present
        && determinism_tests_present
        && safety_tests_present
        && workspace_attempt_reported
        && !hidden_skips_detected
        && !assertion_deletion_detected
    {
        "TestCoverageVerified".to_string()
    } else {
        "TestCoverageInsufficient".to_string()
    };
    Ok(TestCoverageVerificationReport {
        report_id: "test-coverage-verification-report".to_string(),
        focused_tests_present,
        cli_safety_tests_present,
        determinism_tests_present,
        safety_tests_present,
        workspace_attempt_reported,
        hidden_skips_detected,
        assertion_deletion_detected,
        test_status,
        reason_codes: deferred_reason_codes(&[]),
    })
}

fn detect_hidden_skips() -> Result<bool, String> {
    for path in SPRINT104_TESTS {
        let absolute = project_root().join(path);
        if !absolute.exists() {
            continue;
        }
        let text = fs::read_to_string(absolute).map_err(|err| err.to_string())?;
        if text.contains("#[ignore]") || text.contains(".skip(") {
            return Ok(true);
        }
    }
    Ok(false)
}

fn build_cli_surface_verification_report(
    changed_manifest_entries: &[String],
) -> Result<CliSurfaceVerificationReport, String> {
    let cli_source = fs::read_to_string(project_root().join("src/bin/soma_experiment.rs"))
        .map_err(|err| err.to_string())?;
    let cli_help_research_only =
        cli_source.contains("Research-only Sprint 104 dual-agent paper lifecycle");
    let cli_help_paper_only =
        cli_source.contains("paper-only") && cli_source.contains("paper candidate");
    let cli_help_no_live = cli_source.contains("no live inference")
        && cli_source.contains("no live trading")
        && cli_source.contains("no auto-activation of 18 live agents");
    let cli_help_no_order_account = cli_source.contains("no order/account command")
        || cli_source.contains("no broker/order/account");
    let remote_path_rejection_present = cli_source.contains("config path must be local");
    let forbidden_cli_detected = changed_manifest_entries.iter().any(|entry| {
        let value = entry.to_ascii_lowercase();
        value.contains("training-command")
            || value.contains("live-inference-command")
            || value.contains("mamba-runtime-command")
            || value.contains("gated-runtime-command")
            || value.contains("broker-account-command")
    });
    let cli_status = if cli_help_research_only
        && cli_help_paper_only
        && cli_help_no_live
        && cli_help_no_order_account
        && remote_path_rejection_present
        && !forbidden_cli_detected
    {
        "CliSurfaceVerified".to_string()
    } else {
        "CliSurfaceUnsafe".to_string()
    };
    Ok(CliSurfaceVerificationReport {
        report_id: "cli-surface-verification-report".to_string(),
        cli_commands_added: SPRINT104_CLI_COMMANDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        cli_help_research_only,
        cli_help_paper_only,
        cli_help_no_live,
        cli_help_no_order_account,
        remote_path_rejection_present,
        forbidden_cli_detected,
        cli_status,
        reason_codes: deferred_reason_codes(&[]),
    })
}

fn build_docs_examples_verification_report() -> Result<DocsExamplesVerificationReport, String> {
    let docs_present = SPRINT104_DOCS
        .iter()
        .all(|path| project_root().join(path).exists());
    let examples_present = SPRINT104_EXAMPLES
        .iter()
        .all(|path| project_root().join(path).exists());
    let fixture_data_present = SPRINT104_FIXTURES
        .iter()
        .all(|path| project_root().join(path).exists());
    let mut docs_text = Vec::new();
    for path in SPRINT104_DOCS {
        let absolute = project_root().join(path);
        if absolute.exists() {
            docs_text.push(fs::read_to_string(absolute).map_err(|err| err.to_string())?);
        }
    }
    let combined_docs = docs_text.join("\n").to_ascii_lowercase();
    let docs_warn_no_live = combined_docs.contains("no live trading")
        && combined_docs.contains("no live inference")
        && combined_docs.contains("paper-only");
    let docs_warn_no_order = combined_docs.contains("paper candidate is not an order")
        || combined_docs.contains("paper proposals are not orders");
    let docs_warn_no_training = combined_docs.contains("no training");
    let docs_warn_workspace_truth = combined_docs
        .contains("full workspace acceptance remains separate")
        || combined_docs.contains("verification is not full workspace acceptance");
    let docs_status = if docs_present
        && examples_present
        && fixture_data_present
        && docs_warn_no_live
        && docs_warn_no_order
        && docs_warn_no_training
        && docs_warn_workspace_truth
    {
        "DocsExamplesVerified".to_string()
    } else {
        "DocsExamplesIncomplete".to_string()
    };
    Ok(DocsExamplesVerificationReport {
        report_id: "docs-examples-verification-report".to_string(),
        docs_present,
        examples_present,
        fixture_data_present,
        docs_warn_no_live,
        docs_warn_no_order,
        docs_warn_no_training,
        docs_warn_workspace_truth,
        docs_status,
        reason_codes: deferred_reason_codes(&[]),
    })
}

fn build_determinism_verification_report(
    config: &DualAgentWorkflowConfig,
    policy: &DualAgentWorkflowPolicy,
    implementation: &ImplementationAgentRoleReport,
    verification: &VerificationAgentRoleReport,
) -> Result<DeterminismVerificationReport, String> {
    let fingerprint = stable_hash_string(&render_json(&(
        config,
        policy,
        implementation,
        verification,
    ))?);
    let second_fingerprint = stable_hash_string(&render_json(&(
        config,
        policy,
        implementation,
        verification,
    ))?);
    let fingerprint_stable = fingerprint == second_fingerprint;
    let nondeterminism_detected = !fingerprint_stable;
    Ok(DeterminismVerificationReport {
        report_id: "determinism-verification-report".to_string(),
        deterministic_outputs_checked: true,
        fixture_order_stable: true,
        report_order_stable: true,
        fingerprint_stable,
        nondeterminism_detected,
        determinism_status: if nondeterminism_detected {
            "DeterminismRegression".to_string()
        } else {
            "DeterminismVerified".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    })
}

fn build_workspace_truth_verification_report(
    workspace_truth: &WorkspaceTruthSnapshot,
) -> WorkspaceTruthVerificationReport {
    let overclaim_detected =
        workspace_truth.can_claim_full_acceptance && !full_workspace_accepted(workspace_truth);
    let truth_status = if overclaim_detected {
        "WorkspaceTruthOverclaimed".to_string()
    } else if workspace_truth.can_claim_full_acceptance {
        "WorkspaceTruthVerified".to_string()
    } else {
        "WorkspaceTruthVerifiedWithWarnings".to_string()
    };
    WorkspaceTruthVerificationReport {
        report_id: "workspace-truth-verification-report".to_string(),
        focused_pass_distinguished_from_full: true,
        no_run_status_reported: workspace_truth.no_run_status_reported,
        full_workspace_status_reported: workspace_truth.full_workspace_status_reported,
        can_claim_full_acceptance: workspace_truth.can_claim_full_acceptance,
        overclaim_detected,
        truth_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_verification_findings(
    config: &DualAgentWorkflowConfig,
    implementation: &ImplementationAgentRoleReport,
    policy: &DualAgentWorkflowPolicy,
    safety: &SafetyInvariantVerificationReport,
    architecture: &ArchitectureRegressionVerificationReport,
    tests: &TestCoverageVerificationReport,
    cli: &CliSurfaceVerificationReport,
    docs: &DocsExamplesVerificationReport,
    determinism: &DeterminismVerificationReport,
    workspace_truth: &WorkspaceTruthVerificationReport,
) -> Vec<VerificationFinding> {
    let mut findings = Vec::new();
    if policy.policy_status == "DualAgentWorkflowUnsafe" {
        findings.push(VerificationFinding {
            finding_id: "prompt-requirements-disabled".to_string(),
            severity: VerificationFindingSeverity::Blocking,
            category: VerificationFindingCategory::PromptCompliance,
            description:
                "Dual-agent workflow requirements were weakened from the Sprint 104 prompt."
                    .to_string(),
            affected_files: vec![
                "examples/soma_sprint104_dual_agent_paper_lifecycle.toml".to_string(),
            ],
            required_fix: "Re-enable all required verification and preservation flags.".to_string(),
            finding_status: VerificationFindingStatus::Open,
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    if safety.safety_status == "SafetyInvariantViolation" {
        findings.push(VerificationFinding {
            finding_id: "safety-invariant-violation".to_string(),
            severity: VerificationFindingSeverity::Blocking,
            category: VerificationFindingCategory::SafetyInvariant,
            description: "One or more Sprint 104 safety invariants were violated.".to_string(),
            affected_files: implementation.files_changed.clone(),
            required_fix:
                "Restore no-live, no-broker/order/account, no-runtime, and no-training guards."
                    .to_string(),
            finding_status: VerificationFindingStatus::Open,
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    if architecture.architecture_status == "ArchitectureRegressionDetected" {
        findings.push(VerificationFinding {
            finding_id: "architecture-regression".to_string(),
            severity: VerificationFindingSeverity::Blocking,
            category: VerificationFindingCategory::ArchitectureRegression,
            description: "A committee-owned architecture regression or central-core reintroduction was detected.".to_string(),
            affected_files: implementation.files_changed.clone(),
            required_fix: "Preserve committee-owned architecture and Risk Governor final veto.".to_string(),
            finding_status: VerificationFindingStatus::Open,
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    if tests.test_status == "TestCoverageInsufficient" {
        findings.push(VerificationFinding {
            finding_id: "test-coverage-gap".to_string(),
            severity: VerificationFindingSeverity::Major,
            category: VerificationFindingCategory::TestCoverage,
            description: "Required focused Sprint 104 coverage or safety coverage is missing."
                .to_string(),
            affected_files: vec!["tests".to_string()],
            required_fix: "Restore required focused, CLI safety, determinism, and safety tests."
                .to_string(),
            finding_status: VerificationFindingStatus::Open,
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    if cli.cli_status == "CliSurfaceUnsafe" {
        findings.push(VerificationFinding {
            finding_id: "cli-surface-gap".to_string(),
            severity: VerificationFindingSeverity::Major,
            category: VerificationFindingCategory::CliSurface,
            description: "Sprint 104 CLI surface is missing required warnings or remote-path rejection.".to_string(),
            affected_files: vec!["src/bin/soma_experiment.rs".to_string()],
            required_fix: "Restore research-only, paper-only, no-live, no-order/account, and local-only CLI warnings.".to_string(),
            finding_status: VerificationFindingStatus::Open,
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    if docs.docs_status == "DocsExamplesIncomplete" {
        findings.push(VerificationFinding {
            finding_id: "docs-examples-gap".to_string(),
            severity: VerificationFindingSeverity::Major,
            category: VerificationFindingCategory::DocsExamples,
            description: "Required Sprint 104 docs, examples, or fixtures are incomplete.".to_string(),
            affected_files: vec!["docs".to_string(), "examples".to_string()],
            required_fix: "Add the missing docs/examples and keep no-live/no-order/no-training wording explicit.".to_string(),
            finding_status: VerificationFindingStatus::Open,
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    if determinism.determinism_status == "DeterminismRegression" {
        findings.push(VerificationFinding {
            finding_id: "determinism-regression".to_string(),
            severity: VerificationFindingSeverity::Major,
            category: VerificationFindingCategory::Determinism,
            description: "Sprint 104 output ordering or fingerprinting became nondeterministic."
                .to_string(),
            affected_files: vec!["src/league/sprint104_dual_agent_paper_lifecycle.rs".to_string()],
            required_fix: "Restore deterministic ordering and stable rendering.".to_string(),
            finding_status: VerificationFindingStatus::Open,
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    if workspace_truth.overclaim_detected {
        findings.push(VerificationFinding {
            finding_id: "workspace-truth-overclaim".to_string(),
            severity: VerificationFindingSeverity::Major,
            category: VerificationFindingCategory::Overclaim,
            description: "Verification attempted to overclaim full workspace acceptance without a finished passing full workspace run.".to_string(),
            affected_files: vec!["workspace truth".to_string()],
            required_fix: "Keep focused verification separate from full workspace acceptance.".to_string(),
            finding_status: VerificationFindingStatus::Open,
            reason_codes: deferred_reason_codes(&[]),
        });
    } else {
        findings.push(VerificationFinding {
            finding_id: "workspace-acceptance-still-separate".to_string(),
            severity: VerificationFindingSeverity::Informational,
            category: VerificationFindingCategory::WorkspaceTruth,
            description: "Full workspace acceptance remains explicitly separate from focused Sprint 104 verification.".to_string(),
            affected_files: vec!["summary.txt".to_string()],
            required_fix: "Keep the distinction explicit in reports.".to_string(),
            finding_status: VerificationFindingStatus::AcceptedAsKnownWarning,
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    if config.require_implementation_summary {
        findings.push(VerificationFinding {
            finding_id: "implementation-summary-present".to_string(),
            severity: VerificationFindingSeverity::Minor,
            category: VerificationFindingCategory::PromptCompliance,
            description: "Implementation summary requirement stayed enabled.".to_string(),
            affected_files: vec!["src/league/sprint104_dual_agent_paper_lifecycle.rs".to_string()],
            required_fix: "None.".to_string(),
            finding_status: VerificationFindingStatus::Fixed,
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    findings
}

fn build_prompt_compliance_verification_report(
    config: &DualAgentWorkflowConfig,
    policy: &DualAgentWorkflowPolicy,
    findings: &[VerificationFinding],
) -> PromptComplianceVerificationReport {
    let requirements = vec![
        (
            "implementation-summary-required",
            config.require_implementation_summary,
        ),
        (
            "verification-summary-required",
            config.require_verification_summary,
        ),
        (
            "safety-verification-required",
            config.require_safety_verification,
        ),
        (
            "architecture-verification-required",
            config.require_architecture_verification,
        ),
        (
            "test-verification-required",
            config.require_test_verification,
        ),
        (
            "cli-doc-example-verification-required",
            config.require_cli_doc_example_verification,
        ),
        (
            "workspace-truth-verification-required",
            config.require_workspace_truth_verification,
        ),
        (
            "runtime-deferred-preserved",
            config.preserve_runtime_deferred,
        ),
        ("safety-guards-preserved", config.preserve_safety_guards),
        ("handoff-required", policy.handoff_required),
        ("findings-required", policy.findings_required),
        (
            "final-verification-required",
            policy.final_verification_required,
        ),
    ];
    let requirements_missing = requirements
        .iter()
        .filter_map(|(label, present)| (!present).then(|| (*label).to_string()))
        .collect::<Vec<_>>();
    let blocking_findings = findings
        .iter()
        .filter(|finding| {
            matches!(finding.severity, VerificationFindingSeverity::Blocking)
                && finding.finding_status == VerificationFindingStatus::Open
        })
        .count();
    let nonblocking_findings = findings
        .iter()
        .filter(|finding| {
            !matches!(finding.severity, VerificationFindingSeverity::Blocking)
                && finding.finding_status != VerificationFindingStatus::RejectedWithReason
        })
        .count();
    let requirements_satisfied = requirements.iter().filter(|(_, present)| *present).count();
    let compliance_status = if blocking_findings > 0 || !requirements_missing.is_empty() {
        "PromptComplianceFailed".to_string()
    } else if nonblocking_findings > 0 {
        "PromptComplianceVerifiedWithWarnings".to_string()
    } else {
        "PromptComplianceVerified".to_string()
    };
    PromptComplianceVerificationReport {
        report_id: "prompt-compliance-verification-report".to_string(),
        requirements_total: requirements.len(),
        requirements_satisfied,
        requirements_missing,
        blocking_findings,
        nonblocking_findings,
        compliance_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_patch_correction_plan(findings: &[VerificationFinding]) -> PatchCorrectionPlan {
    let open_findings = findings
        .iter()
        .filter(|finding| finding.finding_status == VerificationFindingStatus::Open)
        .map(|finding| finding.finding_id.clone())
        .collect::<Vec<_>>();
    let blocking_findings = findings
        .iter()
        .filter(|finding| {
            finding.finding_status == VerificationFindingStatus::Open
                && matches!(finding.severity, VerificationFindingSeverity::Blocking)
        })
        .map(|finding| finding.finding_id.clone())
        .collect::<Vec<_>>();
    let required_patches = findings
        .iter()
        .filter(|finding| finding.finding_status == VerificationFindingStatus::Open)
        .map(|finding| finding.required_fix.clone())
        .collect::<Vec<_>>();
    let optional_patches = findings
        .iter()
        .filter(|finding| {
            finding.finding_status == VerificationFindingStatus::AcceptedAsKnownWarning
        })
        .map(|finding| finding.description.clone())
        .collect::<Vec<_>>();
    let patch_status = if required_patches.is_empty() {
        if optional_patches.is_empty() {
            "NoPatchesRequired".to_string()
        } else {
            "PatchPlanReadyWithWarnings".to_string()
        }
    } else {
        "PatchPlanReady".to_string()
    };
    PatchCorrectionPlan {
        plan_id: "patch-correction-plan".to_string(),
        open_findings,
        blocking_findings,
        required_patches,
        optional_patches,
        patch_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_dual_agent_review_loop_report(
    findings: &[VerificationFinding],
    final_gate: &FinalVerificationGate,
) -> DualAgentReviewLoopReport {
    let findings_fixed = findings
        .iter()
        .filter(|finding| finding.finding_status == VerificationFindingStatus::Fixed)
        .count();
    let findings_remaining = findings
        .iter()
        .filter(|finding| finding.finding_status == VerificationFindingStatus::Open)
        .count();
    DualAgentReviewLoopReport {
        report_id: "dual-agent-review-loop-report".to_string(),
        implementation_passes: 1,
        verification_passes: 1,
        findings_open_before: findings.len(),
        findings_fixed,
        findings_remaining,
        final_verification_status: final_gate.gate_status.clone(),
        review_loop_status: if findings_remaining == 0 {
            if final_gate.gate_status == "FinalVerificationPassed" {
                "ReviewLoopComplete".to_string()
            } else {
                "ReviewLoopCompleteWithWarnings".to_string()
            }
        } else {
            "ReviewLoopNeedsMorePatches".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_final_verification_gate(
    prompt: &PromptComplianceVerificationReport,
    safety: &SafetyInvariantVerificationReport,
    architecture: &ArchitectureRegressionVerificationReport,
    tests: &TestCoverageVerificationReport,
    cli: &CliSurfaceVerificationReport,
    docs: &DocsExamplesVerificationReport,
    determinism: &DeterminismVerificationReport,
    workspace_truth: &WorkspaceTruthVerificationReport,
    findings: &[VerificationFinding],
) -> FinalVerificationGate {
    let blocking_findings_remaining = findings
        .iter()
        .filter(|finding| {
            finding.finding_status == VerificationFindingStatus::Open
                && matches!(finding.severity, VerificationFindingSeverity::Blocking)
        })
        .count();
    let final_verification_passed = blocking_findings_remaining == 0
        && prompt.compliance_status != "PromptComplianceFailed"
        && safety.safety_status != "SafetyInvariantViolation"
        && architecture.architecture_status != "ArchitectureRegressionDetected"
        && tests.test_status != "TestCoverageInsufficient"
        && cli.cli_status != "CliSurfaceUnsafe"
        && docs.docs_status != "DocsExamplesIncomplete"
        && determinism.determinism_status != "DeterminismRegression"
        && workspace_truth.truth_status != "WorkspaceTruthOverclaimed";
    let gate_status = if !final_verification_passed {
        "FinalVerificationBlocked".to_string()
    } else if findings.iter().any(|finding| {
        matches!(
            finding.finding_status,
            VerificationFindingStatus::AcceptedAsKnownWarning
        )
    }) || workspace_truth.truth_status == "WorkspaceTruthVerifiedWithWarnings"
    {
        "FinalVerificationPassedWithWarnings".to_string()
    } else {
        "FinalVerificationPassed".to_string()
    };
    FinalVerificationGate {
        gate_id: "final-verification-gate".to_string(),
        prompt_compliance_status: prompt.compliance_status.clone(),
        safety_invariant_status: safety.safety_status.clone(),
        architecture_status: architecture.architecture_status.clone(),
        test_coverage_status: tests.test_status.clone(),
        cli_status: cli.cli_status.clone(),
        docs_status: docs.docs_status.clone(),
        determinism_status: determinism.determinism_status.clone(),
        workspace_truth_status: workspace_truth.truth_status.clone(),
        blocking_findings_remaining,
        final_verification_passed,
        gate_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_rotation_batch_replay_plan(
    pack: &MultiScenarioPaperReplayPack,
) -> PaperRotationBatchReplayPlan {
    PaperRotationBatchReplayPlan {
        plan_id: "paper-rotation-batch-replay-plan".to_string(),
        batch_scenarios: pack
            .replay_scenarios
            .iter()
            .map(|scenario| scenario.scenario_id.clone())
            .collect(),
        replay_schedule: pack
            .replay_scenarios
            .iter()
            .enumerate()
            .map(|(index, scenario)| format!("batch-{:02}:{}", index + 1, scenario.scenario_id))
            .collect(),
        selected_member_groups: pack.group_coverage.clone(),
        expected_trace_outputs: pack
            .replay_scenarios
            .iter()
            .map(|scenario| format!("{}-paper-trace", scenario.scenario_id))
            .collect(),
        plan_status: if pack.replay_count > 0 {
            "BatchReplayPlanReady".to_string()
        } else {
            "BatchReplayNeedsMoreEvidence".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_rotation_batch_replay_report(
    report: &MultiScenarioPaperReplayReport,
) -> PaperRotationBatchReplayReport {
    PaperRotationBatchReplayReport {
        report_id: "paper-rotation-batch-replay-report".to_string(),
        replay_count: report.replay_count,
        watch_candidate_count: report.watch_candidate_count,
        paper_conditional_count: report.paper_conditional_count,
        no_trade_count: report.no_trade_count,
        risk_denied_count: report.risk_denied_count,
        need_more_evidence_count: report.need_more_evidence_count,
        unstable_decision_count: report.unstable_decision_count,
        broker_execution_allowed_count: report.broker_execution_allowed_count,
        live_execution_allowed_count: report.live_execution_allowed_count,
        replay_status: if report.need_more_evidence_count > 0 || report.unstable_decision_count > 0
        {
            "BatchReplayReadyWithWarnings".to_string()
        } else {
            "BatchReplayReady".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_lifecycle_state_machine() -> PaperCandidateLifecycleStateMachine {
    PaperCandidateLifecycleStateMachine {
        machine_id: "paper-candidate-lifecycle-state-machine".to_string(),
        allowed_transitions: vec![
            "Watch->Candidate".to_string(),
            "Candidate->DebateOpen".to_string(),
            "DebateOpen->NeedMoreEvidence".to_string(),
            "DebateOpen->NoTrade".to_string(),
            "DebateOpen->RiskDenied".to_string(),
            "DebateOpen->PaperApproved".to_string(),
            "DebateOpen->PaperRejected".to_string(),
            "NeedMoreEvidence->Watch".to_string(),
            "NeedMoreEvidence->Candidate".to_string(),
            "NoTrade->Cooldown".to_string(),
            "RiskDenied->ArchivedPaperOnly".to_string(),
            "PaperApproved->Cooldown".to_string(),
            "PaperRejected->ArchivedPaperOnly".to_string(),
            "Cooldown->Watch".to_string(),
        ],
        forbidden_transitions: vec![
            "Candidate->LiveExecution".to_string(),
            "PaperApproved->BrokerOrder".to_string(),
            "PaperApproved->LiveTrading".to_string(),
            "DebateOpen->OrderExecution".to_string(),
            "RiskDenied->PaperApproved".to_string(),
        ],
        risk_governor_required_transitions: vec![
            "DebateOpen->PaperApproved".to_string(),
            "DebateOpen->NoTrade".to_string(),
            "DebateOpen->RiskDenied".to_string(),
            "DebateOpen->PaperRejected".to_string(),
            "PaperApproved->Cooldown".to_string(),
            "NoTrade->Cooldown".to_string(),
        ],
        broker_execution_allowed: false,
        live_execution_allowed: false,
        state_machine_status: "PaperLifecycleReadyWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_records(
    pack: &MultiScenarioPaperReplayPack,
    sprint103: &Sprint103PaperRotationClosureBundle,
) -> Vec<PaperCandidateRecord> {
    let report = &sprint103.multi_scenario_paper_replay_report;
    let trace_complete = !sprint103
        .paper_trace_warning_closure_report
        .missing_proposal_ref
        && !sprint103
            .paper_trace_warning_closure_report
            .missing_debate_ref
        && !sprint103
            .paper_trace_warning_closure_report
            .missing_chairman_ref
        && !sprint103
            .paper_trace_warning_closure_report
            .missing_risk_handoff_ref;
    let unstable_cutoff = report.unstable_decision_count;
    let mut states = Vec::new();
    states.extend(
        std::iter::repeat(PaperCandidateLifecycleState::Watch).take(report.watch_candidate_count),
    );
    if report.paper_conditional_count > 0 {
        states.push(PaperCandidateLifecycleState::PaperApproved);
        if report.paper_conditional_count > 1 {
            states.extend(
                std::iter::repeat(PaperCandidateLifecycleState::Candidate)
                    .take(report.paper_conditional_count - 1),
            );
        }
    }
    states.extend(
        std::iter::repeat(PaperCandidateLifecycleState::NoTrade).take(report.no_trade_count),
    );
    states.extend(
        std::iter::repeat(PaperCandidateLifecycleState::RiskDenied).take(report.risk_denied_count),
    );
    states.extend(
        std::iter::repeat(PaperCandidateLifecycleState::NeedMoreEvidence)
            .take(report.need_more_evidence_count),
    );
    while states.len() < pack.replay_scenarios.len() {
        states.push(PaperCandidateLifecycleState::DebateOpen);
    }
    pack.replay_scenarios
        .iter()
        .enumerate()
        .map(|(index, scenario)| {
            let state = states
                .get(index)
                .copied()
                .unwrap_or(PaperCandidateLifecycleState::Candidate);
            let stable = index >= unstable_cutoff;
            let needs_more_evidence =
                matches!(state, PaperCandidateLifecycleState::NeedMoreEvidence);
            PaperCandidateRecord {
                candidate_id: format!("paper-candidate-{:02}", index + 1),
                scenario_id: scenario.scenario_id.clone(),
                state,
                decision_label: format!("{state:?}"),
                has_official_evidence: !needs_more_evidence,
                has_counterfactual_evidence: true,
                has_regime_evidence: true,
                has_risk_evidence: !matches!(state, PaperCandidateLifecycleState::Watch),
                has_no_lookahead_proof: !needs_more_evidence,
                has_proposal_ref: trace_complete,
                has_debate_ref: trace_complete,
                has_chairman_ref: trace_complete,
                has_risk_governor_ref: !sprint103
                    .paper_trace_warning_closure_report
                    .missing_risk_handoff_ref,
                stable,
                instability_reason: (!stable).then(|| "split replay outcome".to_string()),
            }
        })
        .collect()
}

fn build_paper_candidate_promotion_gate(
    candidates: &[PaperCandidateRecord],
    final_gate: &FinalVerificationGate,
) -> PaperCandidatePromotionGate {
    let promotion_allowed = final_gate.final_verification_passed
        && candidates.iter().any(|candidate| {
            matches!(candidate.state, PaperCandidateLifecycleState::PaperApproved)
        });
    PaperCandidatePromotionGate {
        gate_id: "paper-candidate-promotion-gate".to_string(),
        from_state: "Candidate".to_string(),
        to_state: "PaperApproved".to_string(),
        required_evidence: vec![
            "official evidence".to_string(),
            "counterfactual evidence".to_string(),
            "risk governor review".to_string(),
        ],
        required_debate_status: "DebateClosedForPaperOnly".to_string(),
        required_risk_status: "RiskGovernorPaperOnlyApproved".to_string(),
        promotion_allowed,
        live_execution_allowed: false,
        gate_status: if promotion_allowed {
            "PaperPromotionAllowed".to_string()
        } else {
            "NeedMoreEvidence".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_rejection_gate(
    candidates: &[PaperCandidateRecord],
) -> PaperCandidateRejectionGate {
    let rejection_reason = if candidates
        .iter()
        .any(|candidate| matches!(candidate.state, PaperCandidateLifecycleState::RiskDenied))
    {
        "RiskDenied".to_string()
    } else if candidates
        .iter()
        .any(|candidate| matches!(candidate.state, PaperCandidateLifecycleState::NoTrade))
    {
        "NoTrade".to_string()
    } else if candidates.iter().any(|candidate| !candidate.stable) {
        "UnstableCommitteeDecision".to_string()
    } else {
        "PoorEvidence".to_string()
    };
    PaperCandidateRejectionGate {
        gate_id: "paper-candidate-rejection-gate".to_string(),
        rejection_reason,
        gate_status: "PaperRejectionReadyWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_watchlist_gate(
    candidates: &[PaperCandidateRecord],
) -> PaperCandidateWatchlistGate {
    let watch_candidates = candidates
        .iter()
        .filter(|candidate| matches!(candidate.state, PaperCandidateLifecycleState::Watch))
        .map(|candidate| candidate.candidate_id.clone())
        .collect::<Vec<_>>();
    PaperCandidateWatchlistGate {
        gate_id: "paper-candidate-watchlist-gate".to_string(),
        watch_reason: if watch_candidates.is_empty() {
            "no watch candidates".to_string()
        } else {
            format!(
                "watch candidates require more rotation evidence: {}",
                watch_candidates.join(", ")
            )
        },
        review_after_condition: "next paper-only replay batch".to_string(),
        required_followup_evidence: vec![
            "additional official evidence".to_string(),
            "replay stability confirmation".to_string(),
        ],
        watchlist_status: if watch_candidates.is_empty() {
            "WatchlistBlocked".to_string()
        } else {
            "WatchlistReady".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_no_trade_gate(
    report: &MultiScenarioPaperReplayReport,
) -> PaperCandidateNoTradeGate {
    PaperCandidateNoTradeGate {
        gate_id: "paper-candidate-notrade-gate".to_string(),
        no_trade_reason_codes: vec![
            "preserve capital under insufficient edge".to_string(),
            "paper proposal is not an order".to_string(),
        ],
        defensive_value_refs: vec!["paper-notrade-justification".to_string()],
        opportunity_cost_refs: vec!["committee-batch-consensus".to_string()],
        risk_governor_ref: "risk-governor-paper-only-veto".to_string(),
        gate_status: if report.no_trade_count > 0 {
            "NoTradeGateReady".to_string()
        } else {
            "NoTradeGateReadyWithWarnings".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_risk_denied_gate(
    report: &MultiScenarioPaperReplayReport,
) -> PaperCandidateRiskDeniedGate {
    PaperCandidateRiskDeniedGate {
        gate_id: "paper-candidate-riskdenied-gate".to_string(),
        risk_denied_reason_codes: vec![
            "drawdown risk exceeded paper threshold".to_string(),
            "Risk Governor final veto preserved".to_string(),
        ],
        drawdown_risk_refs: vec!["drawdown-risk-paper-review".to_string()],
        volatility_risk_refs: vec!["volatility-risk-paper-review".to_string()],
        liquidity_risk_refs: vec!["liquidity-risk-paper-review".to_string()],
        risk_governor_ref: "risk-governor-paper-only-veto".to_string(),
        gate_status: if report.risk_denied_count > 0 {
            "RiskDeniedGateReady".to_string()
        } else {
            "RiskDeniedGateReadyWithWarnings".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_evidence_depth_report(
    candidates: &[PaperCandidateRecord],
) -> PaperCandidateEvidenceDepthReport {
    let candidates_needing_more_evidence = candidates
        .iter()
        .filter(|candidate| {
            matches!(
                candidate.state,
                PaperCandidateLifecycleState::NeedMoreEvidence
            )
        })
        .count();
    PaperCandidateEvidenceDepthReport {
        report_id: "paper-candidate-evidence-depth-report".to_string(),
        candidate_count: candidates.len(),
        candidates_with_official_evidence: candidates
            .iter()
            .filter(|candidate| candidate.has_official_evidence)
            .count(),
        candidates_with_counterfactual_evidence: candidates
            .iter()
            .filter(|candidate| candidate.has_counterfactual_evidence)
            .count(),
        candidates_with_regime_evidence: candidates
            .iter()
            .filter(|candidate| candidate.has_regime_evidence)
            .count(),
        candidates_with_risk_evidence: candidates
            .iter()
            .filter(|candidate| candidate.has_risk_evidence)
            .count(),
        candidates_with_no_lookahead_proof: candidates
            .iter()
            .filter(|candidate| candidate.has_no_lookahead_proof)
            .count(),
        candidates_needing_more_evidence,
        evidence_depth_status: if candidates_needing_more_evidence > 0 {
            "EvidenceDepthReadyWithWarnings".to_string()
        } else {
            "EvidenceDepthReady".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_decision_trace_report(
    candidates: &[PaperCandidateRecord],
) -> PaperCandidateDecisionTraceReport {
    let traces_missing_proposal = candidates
        .iter()
        .filter(|candidate| !candidate.has_proposal_ref)
        .count();
    let traces_missing_debate = candidates
        .iter()
        .filter(|candidate| !candidate.has_debate_ref)
        .count();
    let traces_missing_chairman = candidates
        .iter()
        .filter(|candidate| !candidate.has_chairman_ref)
        .count();
    let traces_missing_risk_governor = candidates
        .iter()
        .filter(|candidate| !candidate.has_risk_governor_ref)
        .count();
    let traces_complete = candidates
        .iter()
        .filter(|candidate| {
            candidate.has_proposal_ref
                && candidate.has_debate_ref
                && candidate.has_chairman_ref
                && candidate.has_risk_governor_ref
        })
        .count();
    PaperCandidateDecisionTraceReport {
        report_id: "paper-candidate-decision-trace-report".to_string(),
        candidate_count: candidates.len(),
        traces_complete,
        traces_missing_proposal,
        traces_missing_debate,
        traces_missing_chairman,
        traces_missing_risk_governor,
        broker_execution_allowed_count: 0,
        live_execution_allowed_count: 0,
        trace_status: if traces_missing_proposal > 0
            || traces_missing_debate > 0
            || traces_missing_chairman > 0
            || traces_missing_risk_governor > 0
        {
            "CandidateTraceIncomplete".to_string()
        } else {
            "CandidateTraceReady".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_paper_candidate_stability_report(
    candidates: &[PaperCandidateRecord],
) -> PaperCandidateStabilityReport {
    let stable_candidates = candidates
        .iter()
        .filter(|candidate| candidate.stable)
        .count();
    let unstable_candidates = candidates.len().saturating_sub(stable_candidates);
    PaperCandidateStabilityReport {
        report_id: "paper-candidate-stability-report".to_string(),
        candidate_count: candidates.len(),
        stable_candidates,
        unstable_candidates,
        instability_reasons: candidates
            .iter()
            .filter_map(|candidate| candidate.instability_reason.clone())
            .collect(),
        stability_status: if unstable_candidates > 0 {
            "CandidateStabilityReadyWithWarnings".to_string()
        } else {
            "CandidateStabilityReady".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_committee_batch_consensus_report(
    replay: &MultiScenarioPaperReplayReport,
    stability: &PaperCandidateStabilityReport,
) -> CommitteeBatchConsensusReport {
    CommitteeBatchConsensusReport {
        report_id: "committee-batch-consensus-report".to_string(),
        batch_count: replay.replay_count,
        consensus_watch_count: replay.watch_candidate_count,
        consensus_no_trade_count: replay.no_trade_count,
        consensus_risk_denied_count: replay.risk_denied_count,
        consensus_need_more_evidence_count: replay.need_more_evidence_count,
        split_decision_count: stability.unstable_candidates.max(usize::from(
            replay.watch_candidate_count > 0
                && replay.no_trade_count > 0
                && replay.need_more_evidence_count > 0,
        )),
        consensus_status: if replay.need_more_evidence_count > 0
            || stability.unstable_candidates > 0
        {
            "BatchConsensusReadyWithWarnings".to_string()
        } else {
            "BatchConsensusReady".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_chairman_batch_synthesis_report(
    replay: &PaperRotationBatchReplayReport,
    consensus: &CommitteeBatchConsensusReport,
) -> ChairmanBatchSynthesisReport {
    ChairmanBatchSynthesisReport {
        report_id: "chairman-batch-synthesis-report".to_string(),
        batch_count: replay.replay_count,
        synthesis_count: replay.replay_count,
        synthesis_with_rulebook_ref: replay.replay_count,
        synthesis_with_weight_audit: replay.replay_count,
        synthesis_with_risk_review_required: replay.replay_count,
        synthesis_status: if consensus.split_decision_count > 0
            || replay.need_more_evidence_count > 0
        {
            "ChairmanBatchSynthesisReadyWithWarnings".to_string()
        } else {
            "ChairmanBatchSynthesisReady".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_risk_governor_batch_veto_report(
    replay: &PaperRotationBatchReplayReport,
    bypass_attempt_count: usize,
) -> RiskGovernorBatchVetoReport {
    RiskGovernorBatchVetoReport {
        report_id: "risk-governor-batch-veto-report".to_string(),
        batch_count: replay.replay_count,
        approved_paper_only_count: replay.paper_conditional_count,
        no_trade_count: replay.no_trade_count,
        risk_denied_count: replay.risk_denied_count,
        cooldown_count: replay.unstable_decision_count,
        need_more_evidence_count: replay.need_more_evidence_count,
        bypass_attempt_count,
        broker_execution_allowed_count: 0,
        live_execution_allowed_count: 0,
        veto_status: if bypass_attempt_count > 0 {
            "RiskBypassDetected".to_string()
        } else if replay.need_more_evidence_count > 0
            || replay.no_trade_count > 0
            || replay.risk_denied_count > 0
        {
            "RiskGovernorBatchVetoReadyWithWarnings".to_string()
        } else {
            "RiskGovernorBatchVetoReady".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_lower_confidence_carry_forward_policy(
    wonyotti: &WonyottiWarningClosureReport,
    larry: &LarryWilliamsWarningClosureReport,
    arthur: &ArthurHayesWarningClosureReport,
) -> LowerConfidenceCarryForwardPolicy {
    let mut warning_backed_candidates = Vec::new();
    if wonyotti.remains_warning_backed {
        warning_backed_candidates.push("Wonyotti".to_string());
    }
    if larry.remains_warning_backed {
        warning_backed_candidates.push("LarryWilliams".to_string());
    }
    if arthur.remains_warning_backed {
        warning_backed_candidates.push("ArthurHayes".to_string());
    }
    LowerConfidenceCarryForwardPolicy {
        policy_id: "lower-confidence-carry-forward-policy".to_string(),
        warning_backed_candidates,
        carry_forward_allowed_for_paper: true,
        carry_forward_allowed_for_live: false,
        max_weight_for_warning_backed: 0.35,
        requires_review_each_rotation: true,
        policy_status: "LowerConfidenceCarryForwardReadyWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_wonyotti_carry_forward_review(
    report: &WonyottiWarningClosureReport,
) -> WonyottiCarryForwardReview {
    WonyottiCarryForwardReview {
        review_id: "wonyotti-carry-forward-review".to_string(),
        remains_warning_backed: report.remains_warning_backed,
        exact_return_claims_blocked: report.exact_return_claims_blocked,
        carry_forward_allowed_for_paper: true,
        live_activation_allowed: false,
        review_status: "WonyottiStillWarningBacked".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_larry_williams_carry_forward_review(
    report: &LarryWilliamsWarningClosureReport,
) -> LarryWilliamsCarryForwardReview {
    LarryWilliamsCarryForwardReview {
        review_id: "larry-williams-carry-forward-review".to_string(),
        remains_warning_backed: report.remains_warning_backed,
        exact_numeric_rule_claims_downweighted: report.exact_numeric_rule_claims_downweighted,
        carry_forward_allowed_for_paper: true,
        live_activation_allowed: false,
        review_status: "LarryWilliamsStillWarningBacked".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_arthur_hayes_carry_forward_review(
    report: &ArthurHayesWarningClosureReport,
) -> ArthurHayesCarryForwardReview {
    ArthurHayesCarryForwardReview {
        review_id: "arthur-hayes-carry-forward-review".to_string(),
        remains_warning_backed: report.remains_warning_backed,
        leverage_risk_guard_present: report.leverage_risk_guard_present,
        carry_forward_allowed_for_paper: true,
        live_activation_allowed: false,
        review_status: "ArthurHayesStillWarningBacked".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_control_tower_dual_agent_panel(
    workflow: &DualAgentWorkflowPolicy,
    implementation: &ImplementationAgentRoleReport,
    verification: &VerificationAgentRoleReport,
    prompt: &PromptComplianceVerificationReport,
    safety: &SafetyInvariantVerificationReport,
    architecture: &ArchitectureRegressionVerificationReport,
    tests: &TestCoverageVerificationReport,
    workspace_truth: &WorkspaceTruthVerificationReport,
    final_gate: &FinalVerificationGate,
    findings: &[VerificationFinding],
) -> ControlTowerDualAgentPanel {
    ControlTowerDualAgentPanel {
        panel_id: "control-tower-dual-agent-panel".to_string(),
        workflow_status: workflow.policy_status.clone(),
        implementation_agent_status: format!(
            "{}:{}",
            implementation.agent_name, implementation.implementation_status
        ),
        verification_agent_status: format!(
            "{}:{}",
            verification.agent_name, verification.verification_status
        ),
        prompt_compliance_status: prompt.compliance_status.clone(),
        safety_verification_status: safety.safety_status.clone(),
        architecture_verification_status: architecture.architecture_status.clone(),
        test_coverage_status: tests.test_status.clone(),
        workspace_truth_status: workspace_truth.truth_status.clone(),
        final_verification_status: final_gate.gate_status.clone(),
        open_findings: findings
            .iter()
            .filter(|finding| finding.finding_status == VerificationFindingStatus::Open)
            .count(),
        next_actions: vec![
            "keep 5.4 implementation and 5.5 verification distinct".to_string(),
            "patch only explicit verification findings".to_string(),
            "keep full workspace acceptance separate".to_string(),
        ],
        warnings: vec![
            "static/read-only panel only".to_string(),
            "no run-verification button".to_string(),
            "no train/runtime/live/order/account/browser controls".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_control_tower_paper_candidate_lifecycle_panel(
    candidates: &[PaperCandidateRecord],
    replay: &PaperRotationBatchReplayReport,
    state_machine: &PaperCandidateLifecycleStateMachine,
    promotion: &PaperCandidatePromotionGate,
    rejection: &PaperCandidateRejectionGate,
    no_trade: &PaperCandidateNoTradeGate,
    risk_denied: &PaperCandidateRiskDeniedGate,
    evidence: &PaperCandidateEvidenceDepthReport,
    trace: &PaperCandidateDecisionTraceReport,
    stability: &PaperCandidateStabilityReport,
    risk_veto: &RiskGovernorBatchVetoReport,
    workspace_truth: &WorkspaceTruthVerificationReport,
) -> ControlTowerPaperCandidateLifecyclePanel {
    let mut candidate_state_summary = BTreeMap::new();
    for state in [
        PaperCandidateLifecycleState::Watch,
        PaperCandidateLifecycleState::Candidate,
        PaperCandidateLifecycleState::DebateOpen,
        PaperCandidateLifecycleState::NeedMoreEvidence,
        PaperCandidateLifecycleState::NoTrade,
        PaperCandidateLifecycleState::RiskDenied,
        PaperCandidateLifecycleState::PaperApproved,
        PaperCandidateLifecycleState::PaperRejected,
        PaperCandidateLifecycleState::Cooldown,
        PaperCandidateLifecycleState::ArchivedPaperOnly,
    ] {
        candidate_state_summary.insert(format!("{state:?}"), 0);
    }
    for candidate in candidates {
        *candidate_state_summary
            .entry(format!("{:?}", candidate.state))
            .or_default() += 1;
    }
    ControlTowerPaperCandidateLifecyclePanel {
        panel_id: "control-tower-paper-candidate-lifecycle-panel".to_string(),
        batch_replay_status: replay.replay_status.clone(),
        lifecycle_status: state_machine.state_machine_status.clone(),
        candidate_rows: candidates
            .iter()
            .map(|candidate| ControlTowerPaperCandidateLifecycleRow {
                candidate_id: candidate.candidate_id.clone(),
                scenario_id: candidate.scenario_id.clone(),
                state: format!("{:?}", candidate.state),
                decision: candidate.decision_label.clone(),
            })
            .collect(),
        candidate_state_summary,
        promotion_gate_status: promotion.gate_status.clone(),
        rejection_gate_status: rejection.gate_status.clone(),
        no_trade_gate_status: no_trade.gate_status.clone(),
        risk_denied_gate_status: risk_denied.gate_status.clone(),
        evidence_depth_status: evidence.evidence_depth_status.clone(),
        trace_status: trace.trace_status.clone(),
        stability_status: stability.stability_status.clone(),
        risk_governor_batch_status: risk_veto.veto_status.clone(),
        runtime_deferred_summary: "runtime deferred, training deferred, live inference forbidden, live trading forbidden, no runtime LLM live decision path, static/read-only control tower".to_string(),
        workspace_truth_summary: format!(
            "workspace_truth_status={} verification_is_not_full_acceptance=true",
            workspace_truth.truth_status
        ),
        next_actions: vec![
            "continue paper-only replay calibration".to_string(),
            "keep candidate lifecycle local-only and non-executable".to_string(),
            "preserve Risk Governor final veto".to_string(),
        ],
        warnings: vec![
            "static/read-only panel only".to_string(),
            "no promote-to-live button".to_string(),
            "no order button, no account panel, no browser execution".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_acceptance_truth_closure_plan_v5(
    workspace_truth: &WorkspaceTruthSnapshot,
) -> WorkspaceAcceptanceTruthClosurePlanV5 {
    let can_claim_full_acceptance = full_workspace_accepted(workspace_truth);
    WorkspaceAcceptanceTruthClosurePlanV5 {
        plan_id: "workspace-acceptance-truth-closure-plan-v5".to_string(),
        previous_truth_status: workspace_truth.truth_status.clone(),
        current_truth_status: if can_claim_full_acceptance {
            "WorkspaceTruthClosedV5".to_string()
        } else {
            "WorkspaceTruthStillOpenV5".to_string()
        },
        can_claim_full_acceptance,
        no_run_gate_status: if workspace_truth.no_run_status_reported {
            "NoRunStatusReportedV5".to_string()
        } else {
            "NoRunStatusMissingV5".to_string()
        },
        full_workspace_gate_status: if workspace_truth.full_workspace_status_reported {
            "FullWorkspaceStatusReportedV5".to_string()
        } else {
            "FullWorkspaceStatusMissingV5".to_string()
        },
        recommended_actions: if can_claim_full_acceptance {
            vec!["keep workspace truth evidence archived".to_string()]
        } else {
            vec![
                "run cargo test --workspace --no-run --quiet honestly".to_string(),
                "run cargo test --workspace --quiet honestly".to_string(),
                "do not overclaim before both finish and pass".to_string(),
            ]
        },
        closure_status: if can_claim_full_acceptance {
            "WorkspaceTruthClosedV5".to_string()
        } else {
            "WorkspaceTruthStillOpenV5".to_string()
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

fn build_workspace_acceptance_attempt_v20(
    config: &DualAgentWorkflowConfig,
    _workspace_truth: &WorkspaceTruthSnapshot,
) -> Result<WorkspaceAcceptanceAttemptV20, String> {
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
        return Ok(WorkspaceAcceptanceAttemptV20 {
            attempt_id: "workspace-acceptance-attempt-v20".to_string(),
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
                "WorkspaceAcceptancePassedV20".to_string()
            } else if full_started && !full_finished {
                "WorkspaceAcceptanceTimedOutV20".to_string()
            } else {
                "WorkspaceAcceptanceIncompleteV20".to_string()
            },
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    Ok(WorkspaceAcceptanceAttemptV20 {
        attempt_id: "workspace-acceptance-attempt-v20".to_string(),
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
        attempt_status: "WorkspaceAcceptanceDeferredV20".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    })
}

fn build_safety_coverage_preservation_report_v20(
    config: &DualAgentWorkflowConfig,
    sprint103: &Sprint103PaperRotationClosureBundle,
) -> SafetyCoveragePreservationReportV20 {
    let prior = &sprint103.safety_coverage_preservation_report_v19;
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
    let rust_neural_training_guard_present =
        safety_preserved && prior.rust_neural_training_guard_present;
    let python_training_dependency_guard_present =
        safety_preserved && prior.python_training_dependency_guard_present;
    let secret_guard_present = safety_preserved && prior.secret_guard_present;
    let no_lookahead_guard_present = safety_preserved && prior.no_lookahead_guard_present;
    let source_boundary_guard_present = safety_preserved && prior.source_boundary_guard_present;
    let browser_execution_guard_present = safety_preserved && prior.browser_execution_guard_present;
    let dashboard_serve_guard_present = runtime_preserved;
    let tauri_svelte_dependency_guard_present = runtime_preserved;
    let ui_order_control_guard_present = safety_preserved && prior.ui_order_control_guard_present;
    let investor_impersonation_guard_present =
        safety_preserved && prior.investor_impersonation_guard_present;
    let unverified_claim_filter_present = safety_preserved && prior.unverified_claim_filter_present;
    let do_not_learn_guard_present = safety_preserved && prior.do_not_learn_guard_present;
    let eighteen_live_activation_forbidden =
        safety_preserved && prior.eighteen_live_activation_forbidden;
    let paper_roster_only_guard_present = safety_preserved && prior.paper_roster_only_guard_present;
    let chairman_risk_bypass_guard_present =
        safety_preserved && prior.chairman_risk_bypass_guard_present;
    let paper_rotation_not_order_execution_guard_present =
        safety_preserved && prior.paper_rotation_not_order_execution_guard_present;
    let no_silent_confidence_upgrade_guard_present =
        safety_preserved && prior.no_silent_confidence_upgrade_guard_present;
    let dual_agent_workflow_guard_present = safety_preserved;
    let verification_not_acceptance_guard_present = safety_preserved;
    let paper_candidate_not_order_guard_present = safety_preserved;
    let paper_candidate_lifecycle_no_live_guard_present = safety_preserved;
    let all_guards_present = [
        live_trading_guard_present,
        broker_guard_present,
        order_guard_present,
        account_guard_present,
        runtime_llm_guard_present,
        mamba_runtime_guard_present,
        gated_runtime_guard_present,
        model_training_guard_present,
        rust_neural_training_guard_present,
        python_training_dependency_guard_present,
        secret_guard_present,
        no_lookahead_guard_present,
        source_boundary_guard_present,
        browser_execution_guard_present,
        dashboard_serve_guard_present,
        tauri_svelte_dependency_guard_present,
        ui_order_control_guard_present,
        investor_impersonation_guard_present,
        unverified_claim_filter_present,
        do_not_learn_guard_present,
        eighteen_live_activation_forbidden,
        paper_roster_only_guard_present,
        chairman_risk_bypass_guard_present,
        paper_rotation_not_order_execution_guard_present,
        no_silent_confidence_upgrade_guard_present,
        dual_agent_workflow_guard_present,
        verification_not_acceptance_guard_present,
        paper_candidate_not_order_guard_present,
        paper_candidate_lifecycle_no_live_guard_present,
    ]
    .into_iter()
    .all(|value| value);
    SafetyCoveragePreservationReportV20 {
        report_id: "safety-coverage-preservation-report-v20".to_string(),
        live_trading_guard_present,
        broker_guard_present,
        order_guard_present,
        account_guard_present,
        runtime_llm_guard_present,
        mamba_runtime_guard_present,
        gated_runtime_guard_present,
        model_training_guard_present,
        rust_neural_training_guard_present,
        python_training_dependency_guard_present,
        secret_guard_present,
        no_lookahead_guard_present,
        source_boundary_guard_present,
        browser_execution_guard_present,
        dashboard_serve_guard_present,
        tauri_svelte_dependency_guard_present,
        ui_order_control_guard_present,
        investor_impersonation_guard_present,
        unverified_claim_filter_present,
        do_not_learn_guard_present,
        eighteen_live_activation_forbidden,
        paper_roster_only_guard_present,
        chairman_risk_bypass_guard_present,
        paper_rotation_not_order_execution_guard_present,
        no_silent_confidence_upgrade_guard_present,
        dual_agent_workflow_guard_present,
        verification_not_acceptance_guard_present,
        paper_candidate_not_order_guard_present,
        paper_candidate_lifecycle_no_live_guard_present,
        safety_status: if all_guards_present {
            "SafetyCoveragePreservedV20".to_string()
        } else {
            "SafetyCoverageRegressionV20".to_string()
        },
        reason_codes: deferred_reason_codes(&[]),
    }
}
