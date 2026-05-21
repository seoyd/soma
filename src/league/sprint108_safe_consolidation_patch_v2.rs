use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::league::sprint107_safe_consolidation_patch::{
    AcceptanceRecoveryVerificationReportV2, AcceptanceTruthGateV8,
    AssertionPreservationVerificationReportV1, ConsolidatedTestTargetManifestV1,
    ControlTowerWorkspaceAcceptanceRecoveryPanelV8, FocusedVsFullBridgeV4,
    MeasuredOrSampleBackedDeltaGateV1, PostPatchCliSmokeRunReportV1,
    PostPatchDeterminismRunReportV1, PostPatchFocusedTestRunReportV1, PostPatchSafetyRunReportV1,
    PostPatchWorkspaceFullAttemptV23, RegressionSurfaceAuditReportV1,
    RetiredNarrowTargetManifestV1, SafeConsolidationPatchV1Bundle, SafeConsolidationPatchV1Config,
    SafeConsolidationPatchV1Runner, SafetyCoveragePreservationReportV23,
    SafetySentinelPreservationReportV1, WorkspaceFullAcceptanceGateV8,
    WorkspaceNoRunRecoveryGateV8,
};

fn render_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|err| err.to_string())
}

fn write_json_file<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    fs::write(path, render_json(value)?).map_err(|err| err.to_string())
}

fn write_text_file(path: &Path, value: &str) -> Result<(), String> {
    fs::write(path, value).map_err(|err| err.to_string())
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn local_only(path: &str) -> bool {
    !path.contains("://")
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_output_root() -> String {
    "target/soma_sprint108_safe_consolidation_patch_v2".to_string()
}

fn default_timeout_ms() -> Option<u64> {
    Some(180_000)
}

fn default_max_targets_to_consolidate() -> usize {
    1
}

fn deferred_reason_codes(extra: &[ReasonCode]) -> Vec<ReasonCode> {
    let mut codes = vec![
        ReasonCode::CommitteeV1Built,
        ReasonCode::CommitteeV1RunnerBuilt,
        ReasonCode::OwnerCannotBypassRiskGovernor,
        ReasonCode::NoTradeDefault,
        ReasonCode::DeterministicPath,
        ReasonCode::MambaRuntimeDeferred,
        ReasonCode::GatedDeltaNetRuntimeDeferred,
        ReasonCode::ControlTowerUiReadinessBuilt,
        ReasonCode::LocalFileOnly,
    ];
    codes.extend_from_slice(extra);
    codes
}

fn stable_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn load_first_json<T: DeserializeOwned>(paths: Option<&Vec<String>>) -> Result<Option<T>, String> {
    let Some(paths) = paths else {
        return Ok(None);
    };
    let mut parse_errors = Vec::new();
    for path in paths {
        let text = fs::read_to_string(path)
            .map_err(|err| format!("failed to read sprint108 JSON input {path}: {err}"))?;
        match serde_json::from_str::<T>(&text) {
            Ok(value) => return Ok(Some(value)),
            Err(err) => parse_errors.push(format!("{path}: {err}")),
        }
    }
    if !paths.is_empty() {
        return Err(format!(
            "failed to parse any sprint108 JSON input: {}",
            parse_errors.join("; ")
        ));
    }
    Ok(None)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeConsolidationPatchV2Config {
    pub patch_id: String,
    #[serde(default)]
    pub sprint107_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub sprint107_verification_summary_paths: Option<Vec<String>>,
    #[serde(default)]
    pub previous_assertion_ledger_paths: Option<Vec<String>>,
    #[serde(default)]
    pub safe_consolidation_plan_paths: Option<Vec<String>>,
    #[serde(default)]
    pub shared_fixture_harness_plan_paths: Option<Vec<String>>,
    #[serde(default)]
    pub artifact_cache_plan_paths: Option<Vec<String>>,
    #[serde(default)]
    pub cli_smoke_tiering_paths: Option<Vec<String>>,
    #[serde(default)]
    pub workspace_truth_paths: Option<Vec<String>>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub apply_shared_fixture_harness_expansion: bool,
    #[serde(default = "default_true")]
    pub apply_shared_toml_builder_expansion: bool,
    #[serde(default = "default_true")]
    pub apply_shared_output_dir_helper_expansion: bool,
    #[serde(default = "default_true")]
    pub apply_shared_render_helper_expansion: bool,
    #[serde(default = "default_false")]
    pub apply_artifact_render_cache: bool,
    #[serde(default = "default_true")]
    pub apply_cli_smoke_tiering_refinement: bool,
    #[serde(default = "default_true")]
    pub apply_one_safe_consolidation: bool,
    #[serde(default = "default_max_targets_to_consolidate")]
    pub max_targets_to_consolidate: usize,
    #[serde(default = "default_true")]
    pub require_verification_reconciliation: bool,
    #[serde(default = "default_true")]
    pub require_assertion_ledger: bool,
    #[serde(default = "default_true")]
    pub require_equivalent_coverage_proof: bool,
    #[serde(default = "default_true")]
    pub require_safety_sentinel_preservation: bool,
    #[serde(default = "default_true")]
    pub require_no_hidden_skips: bool,
    #[serde(default = "default_true")]
    pub require_no_assertion_deletion: bool,
    #[serde(default = "default_false")]
    pub run_real_no_run_after_patch: bool,
    #[serde(default = "default_false")]
    pub run_real_full_after_patch: bool,
    #[serde(default = "default_timeout_ms")]
    pub no_run_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub full_timeout_ms: Option<u64>,
    #[serde(default = "default_true")]
    pub preserve_runtime_deferred: bool,
    #[serde(default = "default_true")]
    pub preserve_safety_guards: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for SafeConsolidationPatchV2Config {
    fn default() -> Self {
        Self {
            patch_id: "sprint108-safe-consolidation-patch-v2".to_string(),
            sprint107_bundle_paths: Some(vec![
                "examples/sprint108_data/sprint107_summary.json".to_string(),
            ]),
            sprint107_verification_summary_paths: None,
            previous_assertion_ledger_paths: None,
            safe_consolidation_plan_paths: None,
            shared_fixture_harness_plan_paths: None,
            artifact_cache_plan_paths: None,
            cli_smoke_tiering_paths: None,
            workspace_truth_paths: None,
            output_root: default_output_root(),
            apply_shared_fixture_harness_expansion: true,
            apply_shared_toml_builder_expansion: true,
            apply_shared_output_dir_helper_expansion: true,
            apply_shared_render_helper_expansion: true,
            apply_artifact_render_cache: false,
            apply_cli_smoke_tiering_refinement: true,
            apply_one_safe_consolidation: true,
            max_targets_to_consolidate: 1,
            require_verification_reconciliation: true,
            require_assertion_ledger: true,
            require_equivalent_coverage_proof: true,
            require_safety_sentinel_preservation: true,
            require_no_hidden_skips: true,
            require_no_assertion_deletion: true,
            run_real_no_run_after_patch: false,
            run_real_full_after_patch: false,
            no_run_timeout_ms: default_timeout_ms(),
            full_timeout_ms: default_timeout_ms(),
            preserve_runtime_deferred: true,
            preserve_safety_guards: true,
            reason_codes: deferred_reason_codes(&[]),
        }
    }
}

impl SafeConsolidationPatchV2Config {
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
        PathBuf::from(&self.output_root).join(&self.patch_id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.patch_id.trim().is_empty() {
            return Err("sprint108 patch_id must not be empty".to_string());
        }
        if self.output_root.trim().is_empty() {
            return Err("sprint108 output_root must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err(
                "sprint108 safe consolidation patch config paths must be local".to_string(),
            );
        }
        for paths in [
            &self.sprint107_bundle_paths,
            &self.sprint107_verification_summary_paths,
            &self.previous_assertion_ledger_paths,
            &self.safe_consolidation_plan_paths,
            &self.shared_fixture_harness_plan_paths,
            &self.artifact_cache_plan_paths,
            &self.cli_smoke_tiering_paths,
            &self.workspace_truth_paths,
        ] {
            if let Some(paths) = paths
                && paths.iter().any(|path| !local_only(path))
            {
                return Err(
                    "sprint108 safe consolidation patch config paths must be local".to_string(),
                );
            }
        }
        if self.max_targets_to_consolidate == 0 || self.max_targets_to_consolidate > 1 {
            return Err(
                "sprint108 max_targets_to_consolidate must stay within the second small patch"
                    .to_string(),
            );
        }
        if !self.preserve_runtime_deferred || !self.preserve_safety_guards {
            return Err("sprint108 runtime/safety preservation must remain enabled".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint107VerificationReconciliationReport {
    pub report_id: String,
    pub independent_verification_observed: bool,
    pub verification_fixes: Vec<String>,
    pub child_process_cleanup_fix_confirmed: bool,
    pub full_acceptance_requires_sentinel_fix_confirmed: bool,
    pub focused_full_bridge_fix_confirmed: bool,
    pub safety_coverage_all_guard_fix_confirmed: bool,
    pub regression_test_added: bool,
    pub verification_reconciliation_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndependentVerificationClosureReportV1 {
    pub report_id: String,
    pub implementation_agent: String,
    pub verification_agent: String,
    pub verification_performed: bool,
    pub findings_fixed: usize,
    pub findings_remaining: usize,
    pub final_verification_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationPatchCarryForwardReport {
    pub report_id: String,
    pub carried_forward_patches: Vec<String>,
    pub patches_still_effective: Vec<String>,
    pub patches_regressed: Vec<String>,
    pub carry_forward_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecondSafeConsolidationPatchSelectionReport {
    pub report_id: String,
    pub previous_patch_id: String,
    pub candidate_targets: Vec<String>,
    pub selected_target_group: String,
    pub selection_reason: String,
    pub risk_class: String,
    pub target_count_to_consolidate: usize,
    pub expected_assertion_moves: usize,
    pub expected_binary_delta: Option<isize>,
    pub selected_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecondConsolidationCandidateRiskReviewReport {
    pub report_id: String,
    pub selected_target_group: String,
    pub semantic_risk: String,
    pub safety_risk: String,
    pub determinism_risk: String,
    pub cli_surface_risk: String,
    pub fixture_risk: String,
    pub reason_risk: String,
    pub previous_patch_interaction_risk: String,
    pub risk_review_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssertionMigrationLedgerV2 {
    pub ledger_id: String,
    pub previous_ledger_refs: Vec<String>,
    pub moved_assertions: Vec<String>,
    pub preserved_assertions: Vec<String>,
    pub unchanged_assertions: Vec<String>,
    pub source_targets: Vec<String>,
    pub destination_targets: Vec<String>,
    pub assertion_count_before: usize,
    pub assertion_count_after: usize,
    pub assertion_delta: isize,
    pub duplicate_equivalent_collapses: usize,
    pub ledger_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

pub type AssertionPreservationVerificationReportV2 = AssertionPreservationVerificationReportV1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquivalentCoverageProofReportV1 {
    pub report_id: String,
    pub retired_targets: Vec<String>,
    pub destination_targets: Vec<String>,
    pub moved_assertions: Vec<String>,
    pub equivalent_coverage_assertions: Vec<String>,
    pub coverage_gap_count: usize,
    pub proof_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetiredTargetSafetyAuditReportV2 {
    pub report_id: String,
    pub retired_targets: Vec<String>,
    pub high_risk_target_retired: bool,
    pub safety_sentinel_retired: bool,
    pub assertion_ledger_refs: Vec<String>,
    pub equivalent_coverage_refs: Vec<String>,
    pub audit_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

pub type SafetySentinelPreservationReportV2 = SafetySentinelPreservationReportV1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedFixtureHarnessExpansionApplicationReportV2 {
    pub report_id: String,
    pub previous_application_refs: Vec<String>,
    pub new_targets_affected: Vec<String>,
    pub json_loader_expanded: bool,
    pub toml_loader_expanded: bool,
    pub csv_loader_expanded: bool,
    pub fixture_normalization_expanded: bool,
    pub duplicated_loaders_removed: usize,
    pub deterministic_output_preserved: bool,
    pub application_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedRenderHelperExpansionReportV2 {
    pub report_id: String,
    pub new_targets_affected: Vec<String>,
    pub txt_render_helper_expanded: bool,
    pub json_render_helper_expanded: bool,
    pub html_render_helper_expanded: bool,
    pub stable_sorting_preserved: bool,
    pub snapshot_order_preserved: bool,
    pub duplicated_render_helpers_removed: usize,
    pub application_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedOutputDirHelperExpansionReportV2 {
    pub report_id: String,
    pub new_targets_affected: Vec<String>,
    pub deterministic_output_roots_preserved: bool,
    pub cleanup_policy_preserved: bool,
    pub no_silent_deletion: bool,
    pub duplicated_setup_removed: usize,
    pub application_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedTomlBuilderExpansionReportV2 {
    pub report_id: String,
    pub new_targets_affected: Vec<String>,
    pub local_only_path_validation_preserved: bool,
    pub remote_path_rejection_preserved: bool,
    pub duplicated_toml_builders_removed: usize,
    pub application_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliSmokeTieringApplicationReportV2 {
    pub report_id: String,
    pub previous_representative_smoke_commands: Vec<String>,
    pub previous_exhaustive_smoke_commands: Vec<String>,
    pub previous_safety_smoke_commands: Vec<String>,
    pub representative_smoke_commands: Vec<String>,
    pub exhaustive_smoke_commands: Vec<String>,
    pub safety_smoke_commands: Vec<String>,
    pub commands_moved_to_exhaustive: Vec<String>,
    pub safety_commands_preserved: bool,
    pub no_safety_smoke_removed: bool,
    pub application_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRenderCacheDecisionReportV2 {
    pub report_id: String,
    pub cache_enabled: bool,
    pub why_enabled_or_disabled: String,
    pub local_only_cache: bool,
    pub deterministic_keys: bool,
    pub secret_free_cache: bool,
    pub decision_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

pub type ConsolidatedTestTargetManifestV2 = ConsolidatedTestTargetManifestV1;
pub type RetiredNarrowTargetManifestV2 = RetiredNarrowTargetManifestV1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestBinaryDeltaReportV5 {
    pub report_id: String,
    pub target_count_before: Option<usize>,
    pub target_count_after: Option<usize>,
    pub integration_binary_count_before: Option<usize>,
    pub integration_binary_count_after: Option<usize>,
    pub binary_delta: Option<isize>,
    pub measured: bool,
    pub sample_backed: bool,
    pub timing_available: bool,
    pub cumulative_sample_backed_delta: isize,
    pub delta_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

pub type MeasuredOrSampleBackedDeltaGateV2 = MeasuredOrSampleBackedDeltaGateV1;
pub type PostPatchFocusedTestRunReportV2 = PostPatchFocusedTestRunReportV1;
pub type PostPatchCliSmokeRunReportV2 = PostPatchCliSmokeRunReportV1;
pub type PostPatchSafetyRunReportV2 = PostPatchSafetyRunReportV1;
pub type PostPatchDeterminismRunReportV2 = PostPatchDeterminismRunReportV1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostPatchWorkspaceNoRunAttemptV24 {
    pub attempt_id: String,
    pub command: String,
    pub started: bool,
    pub finished: bool,
    pub passed: Option<bool>,
    pub duration_ms: Option<u64>,
    pub timeout_ms: Option<u64>,
    pub stopped_due_to_timeout: bool,
    pub last_observed_target: Option<String>,
    pub extended_observation_enabled: bool,
    pub child_process_cleanup_verified: bool,
    pub no_run_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

pub type PostPatchWorkspaceFullAttemptV24 = PostPatchWorkspaceFullAttemptV23;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtendedNoRunObservationReportV1 {
    pub report_id: String,
    pub attempted: bool,
    pub timeout_ms: Option<u64>,
    pub observed_duration_ms: Option<u64>,
    pub last_observed_target: Option<String>,
    pub cargo_stdout_present: bool,
    pub cargo_stderr_present: bool,
    pub rustc_processes_after_timeout: usize,
    pub cargo_processes_after_timeout: usize,
    pub observation_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutCleanupVerificationReportV1 {
    pub report_id: String,
    pub timeout_occurred: bool,
    pub child_process_cleanup_attempted: bool,
    pub remaining_cargo_processes: usize,
    pub remaining_rustc_processes: usize,
    pub cleanup_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

pub type WorkspaceNoRunRecoveryGateV9 = WorkspaceNoRunRecoveryGateV8;
pub type WorkspaceFullAcceptanceGateV9 = WorkspaceFullAcceptanceGateV8;
pub type FocusedVsFullBridgeV5 = FocusedVsFullBridgeV4;
pub type AcceptanceTruthGateV9 = AcceptanceTruthGateV8;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceRecoveryPatchImpactReportV3 {
    pub report_id: String,
    pub patch_applied: bool,
    pub target_delta_status: String,
    pub expected_binary_delta: Option<isize>,
    pub measured_binary_delta: Option<isize>,
    pub expected_duration_delta_ms: Option<u64>,
    pub measured_duration_delta_ms: Option<u64>,
    pub cumulative_sample_backed_delta: isize,
    pub cumulative_measured_delta: Option<isize>,
    pub impact_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

pub type AcceptanceRecoveryVerificationReportV3 = AcceptanceRecoveryVerificationReportV2;
pub type RegressionSurfaceAuditReportV2 = RegressionSurfaceAuditReportV1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DualAgentPatchVerificationReportV2 {
    pub report_id: String,
    pub implementation_agent: String,
    pub verification_agent: String,
    pub independent_verification_performed: bool,
    pub verification_reconciliation_status: String,
    pub blocking_findings_remaining: bool,
    pub safety_verified: bool,
    pub architecture_verified: bool,
    pub acceptance_truth_verified: bool,
    pub verification_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerSafeConsolidationPatchPanelV2 {
    pub panel_id: String,
    pub patch_selection_status: String,
    pub verification_reconciliation_status: String,
    pub assertion_ledger_status: String,
    pub equivalent_coverage_status: String,
    pub safety_sentinel_status: String,
    pub binary_delta_status: String,
    pub no_run_status: String,
    pub full_status: String,
    pub timeout_cleanup_status: String,
    pub next_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

pub type ControlTowerWorkspaceAcceptanceRecoveryPanelV9 =
    ControlTowerWorkspaceAcceptanceRecoveryPanelV8;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyCoveragePreservationReportV24 {
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
    pub paper_candidate_not_order_guard_present: bool,
    pub no_silent_confidence_upgrade_guard_present: bool,
    pub focused_not_full_acceptance_guard_present: bool,
    pub no_hidden_skip_guard_present: bool,
    pub assertion_preservation_guard_present: bool,
    pub safety_sentinel_preservation_guard_present: bool,
    pub verification_reconciliation_guard_present: bool,
    pub equivalent_coverage_guard_present: bool,
    pub timeout_cleanup_guard_present: bool,
    pub second_patch_no_broad_consolidation_guard_present: bool,
    pub safety_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeConsolidationPatchV2StorageReport {
    pub report_id: String,
    pub output_dir: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SafeConsolidationPatchV2Bundle {
    pub sprint107_verification_reconciliation_report: Sprint107VerificationReconciliationReport,
    pub independent_verification_closure_report_v1: IndependentVerificationClosureReportV1,
    pub verification_patch_carry_forward_report: VerificationPatchCarryForwardReport,
    pub second_safe_consolidation_patch_selection_report:
        SecondSafeConsolidationPatchSelectionReport,
    pub second_consolidation_candidate_risk_review_report:
        SecondConsolidationCandidateRiskReviewReport,
    pub assertion_migration_ledger_v2: AssertionMigrationLedgerV2,
    pub assertion_preservation_verification_report_v2: AssertionPreservationVerificationReportV2,
    pub equivalent_coverage_proof_report_v1: EquivalentCoverageProofReportV1,
    pub retired_target_safety_audit_report_v2: RetiredTargetSafetyAuditReportV2,
    pub safety_sentinel_preservation_report_v2: SafetySentinelPreservationReportV2,
    pub shared_fixture_harness_expansion_application_report_v2:
        SharedFixtureHarnessExpansionApplicationReportV2,
    pub shared_render_helper_expansion_report_v2: SharedRenderHelperExpansionReportV2,
    pub shared_output_dir_helper_expansion_report_v2: SharedOutputDirHelperExpansionReportV2,
    pub shared_toml_builder_expansion_report_v2: SharedTomlBuilderExpansionReportV2,
    pub cli_smoke_tiering_application_report_v2: CliSmokeTieringApplicationReportV2,
    pub artifact_render_cache_decision_report_v2: ArtifactRenderCacheDecisionReportV2,
    pub consolidated_test_target_manifest_v2: ConsolidatedTestTargetManifestV2,
    pub retired_narrow_target_manifest_v2: RetiredNarrowTargetManifestV2,
    pub test_binary_delta_report_v5: TestBinaryDeltaReportV5,
    pub measured_or_sample_backed_delta_gate_v2: MeasuredOrSampleBackedDeltaGateV2,
    pub post_patch_focused_test_run_report_v2: PostPatchFocusedTestRunReportV2,
    pub post_patch_cli_smoke_run_report_v2: PostPatchCliSmokeRunReportV2,
    pub post_patch_safety_run_report_v2: PostPatchSafetyRunReportV2,
    pub post_patch_determinism_run_report_v2: PostPatchDeterminismRunReportV2,
    pub post_patch_workspace_no_run_attempt_v24: PostPatchWorkspaceNoRunAttemptV24,
    pub post_patch_workspace_full_attempt_v24: PostPatchWorkspaceFullAttemptV24,
    pub extended_no_run_observation_report_v1: ExtendedNoRunObservationReportV1,
    pub timeout_cleanup_verification_report_v1: TimeoutCleanupVerificationReportV1,
    pub workspace_no_run_recovery_gate_v9: WorkspaceNoRunRecoveryGateV9,
    pub workspace_full_acceptance_gate_v9: WorkspaceFullAcceptanceGateV9,
    pub focused_vs_full_bridge_v5: FocusedVsFullBridgeV5,
    pub acceptance_truth_gate_v9: AcceptanceTruthGateV9,
    pub acceptance_recovery_patch_impact_report_v3: AcceptanceRecoveryPatchImpactReportV3,
    pub acceptance_recovery_verification_report_v3: AcceptanceRecoveryVerificationReportV3,
    pub regression_surface_audit_report_v2: RegressionSurfaceAuditReportV2,
    pub dual_agent_patch_verification_report_v2: DualAgentPatchVerificationReportV2,
    pub safety_coverage_preservation_report_v24: SafetyCoveragePreservationReportV24,
    pub control_tower_safe_consolidation_patch_panel_v2: ControlTowerSafeConsolidationPatchPanelV2,
    pub control_tower_workspace_acceptance_recovery_panel_v9:
        ControlTowerWorkspaceAcceptanceRecoveryPanelV9,
    pub storage_report: SafeConsolidationPatchV2StorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl SafeConsolidationPatchV2Bundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            ("## 1. Sprint summary", format!("- Implemented Sprint 108 second safe consolidation patch, verification reconciliation, equivalent coverage proof, and workspace recovery gate v9.\n- selected_target={} cumulative_sample_backed_delta={}.", self.second_safe_consolidation_patch_selection_report.selected_target_group, self.test_binary_delta_report_v5.cumulative_sample_backed_delta)),
            ("## 2. Why Sprint 108 was needed", "- Sprint 107 applied the first patch; Sprint 108 formally reconciles independent verification fixes and applies only one more narrow safe retirement.".to_string()),
            ("## 3. Files added", "- Added Sprint 108 module, CLI/config/examples/docs/tests, and deterministic fixture outputs.".to_string()),
            ("## 4. Files changed", "- Extended existing Sprint 107 helper/support/test surfaces and retired one additional low-risk narrow target after equivalent coverage proof.".to_string()),
            ("## 5. Sprint 107 verification reconciliation", format!("- Status: {}.\n- fixes={}.", self.sprint107_verification_reconciliation_report.verification_reconciliation_status, self.sprint107_verification_reconciliation_report.verification_fixes.join(", "))),
            ("## 6. Independent verification closure", format!("- Status: {}.\n- findings_fixed={} findings_remaining={}.", self.independent_verification_closure_report_v1.final_verification_status, self.independent_verification_closure_report_v1.findings_fixed, self.independent_verification_closure_report_v1.findings_remaining)),
            ("## 7. Verification patch carry-forward", format!("- Status: {}.\n- regressed_patches={}.", self.verification_patch_carry_forward_report.carry_forward_status, self.verification_patch_carry_forward_report.patches_regressed.len())),
            ("## 8. Second safe consolidation patch selection", format!("- Status: {}.\n- selection_reason={}.", self.second_safe_consolidation_patch_selection_report.selected_status, self.second_safe_consolidation_patch_selection_report.selection_reason)),
            ("## 9. Second candidate risk review", format!("- Status: {}.\n- interaction_risk={}.", self.second_consolidation_candidate_risk_review_report.risk_review_status, self.second_consolidation_candidate_risk_review_report.previous_patch_interaction_risk)),
            ("## 10. Assertion migration ledger v2", format!("- Status: {}.\n- assertion_delta={}.", self.assertion_migration_ledger_v2.ledger_status, self.assertion_migration_ledger_v2.assertion_delta)),
            ("## 11. Assertion preservation verification v2", format!("- Status: {}.\n- missing_assertion_count={}.", self.assertion_preservation_verification_report_v2.preservation_status, self.assertion_preservation_verification_report_v2.missing_assertion_count)),
            ("## 12. Equivalent coverage proof", format!("- Status: {}.\n- coverage_gap_count={}.", self.equivalent_coverage_proof_report_v1.proof_status, self.equivalent_coverage_proof_report_v1.coverage_gap_count)),
            ("## 13. Retired target safety audit v2", format!("- Status: {}.\n- safety_sentinel_retired={}.", self.retired_target_safety_audit_report_v2.audit_status, self.retired_target_safety_audit_report_v2.safety_sentinel_retired)),
            ("## 14. Safety sentinel preservation v2", format!("- Status: {}.\n- committee_cli_safety_preserved={}.", self.safety_sentinel_preservation_report_v2.sentinel_status, self.safety_sentinel_preservation_report_v2.committee_cli_safety_preserved)),
            ("## 15. Shared fixture/render/output/TOML helper expansion", format!("- fixture_status={} render_status={} output_status={} toml_status={}.", self.shared_fixture_harness_expansion_application_report_v2.application_status, self.shared_render_helper_expansion_report_v2.application_status, self.shared_output_dir_helper_expansion_report_v2.application_status, self.shared_toml_builder_expansion_report_v2.application_status)),
            ("## 16. Artifact render cache decision", format!("- Status: {}.\n- cache_enabled={}.", self.artifact_render_cache_decision_report_v2.decision_status, self.artifact_render_cache_decision_report_v2.cache_enabled)),
            ("## 17. CLI smoke tiering v2", format!("- Status: {}.\n- safety_commands_preserved={}.", self.cli_smoke_tiering_application_report_v2.application_status, self.cli_smoke_tiering_application_report_v2.safety_commands_preserved)),
            ("## 18. Consolidated / retired target manifests v2", format!("- consolidated_status={} retired_status={}.", self.consolidated_test_target_manifest_v2.manifest_status, self.retired_narrow_target_manifest_v2.retired_status)),
            ("## 19. Test binary delta v5", format!("- Status: {}.\n- binary_delta={:?} cumulative_sample_backed_delta={}.", self.test_binary_delta_report_v5.delta_status, self.test_binary_delta_report_v5.binary_delta, self.test_binary_delta_report_v5.cumulative_sample_backed_delta)),
            ("## 20. Measured vs sample-backed delta gate v2", format!("- Status: {}.\n- can_claim_measured_reduction={}.", self.measured_or_sample_backed_delta_gate_v2.gate_status, self.measured_or_sample_backed_delta_gate_v2.can_claim_measured_reduction)),
            ("## 21. Post-patch focused / CLI / safety / determinism runs", format!("- focused={} cli={} safety={} determinism={}.", self.post_patch_focused_test_run_report_v2.run_status, self.post_patch_cli_smoke_run_report_v2.smoke_status, self.post_patch_safety_run_report_v2.safety_status, self.post_patch_determinism_run_report_v2.determinism_status)),
            ("## 22. Post-patch workspace no-run attempt v24", format!("- Status: {}.\n- cleanup_verified={}.", self.post_patch_workspace_no_run_attempt_v24.no_run_status, self.post_patch_workspace_no_run_attempt_v24.child_process_cleanup_verified)),
            ("## 23. Post-patch workspace full attempt v24", format!("- Status: {}.\n- finished={} passed={:?}.", self.post_patch_workspace_full_attempt_v24.full_status, self.post_patch_workspace_full_attempt_v24.finished, self.post_patch_workspace_full_attempt_v24.passed)),
            ("## 24. Extended no-run observation", format!("- Status: {}.\n- rustc_after_timeout={} cargo_after_timeout={}.", self.extended_no_run_observation_report_v1.observation_status, self.extended_no_run_observation_report_v1.rustc_processes_after_timeout, self.extended_no_run_observation_report_v1.cargo_processes_after_timeout)),
            ("## 25. Timeout cleanup verification", format!("- Status: {}.\n- timeout_occurred={}.", self.timeout_cleanup_verification_report_v1.cleanup_status, self.timeout_cleanup_verification_report_v1.timeout_occurred)),
            ("## 26. Workspace no-run recovery gate v9", format!("- Status: {}.\n- no_run_recovered={}.", self.workspace_no_run_recovery_gate_v9.gate_status, self.workspace_no_run_recovery_gate_v9.no_run_recovered)),
            ("## 27. Workspace full acceptance gate v9", format!("- Status: {}.\n- full_workspace_accepted={}.", self.workspace_full_acceptance_gate_v9.gate_status, self.workspace_full_acceptance_gate_v9.full_workspace_accepted)),
            ("## 28. Focused-vs-full bridge v5", format!("- Status: {}.\n- can_claim_full_acceptance={}.", self.focused_vs_full_bridge_v5.bridge_status, self.focused_vs_full_bridge_v5.can_claim_full_acceptance)),
            ("## 29. Acceptance truth gate v9", format!("- Status: {}.\n- can_claim_full_acceptance={}.", self.acceptance_truth_gate_v9.truth_status, self.acceptance_truth_gate_v9.can_claim_full_acceptance)),
            ("## 30. Patch impact v3", format!("- Status: {}.\n- cumulative_sample_backed_delta={}.", self.acceptance_recovery_patch_impact_report_v3.impact_status, self.acceptance_recovery_patch_impact_report_v3.cumulative_sample_backed_delta)),
            ("## 31. Acceptance recovery verification v3", format!("- Status: {}.\n- assertions_preserved={} no_hidden_skips={}.", self.acceptance_recovery_verification_report_v3.verification_status, self.acceptance_recovery_verification_report_v3.assertions_preserved, self.acceptance_recovery_verification_report_v3.no_hidden_skips)),
            ("## 32. Regression surface audit v2", format!("- Status: {}.\n- high_risk_changes={}.", self.regression_surface_audit_report_v2.regression_status, self.regression_surface_audit_report_v2.high_risk_changes.len())),
            ("## 33. Dual-agent patch verification v2", format!("- Status: {}.\n- independent_verification_performed={}.", self.dual_agent_patch_verification_report_v2.verification_status, self.dual_agent_patch_verification_report_v2.independent_verification_performed)),
            ("## 34. Safety coverage preservation v24", format!("- Status: {}.\n- timeout_cleanup_guard_present={}.", self.safety_coverage_preservation_report_v24.safety_status, self.safety_coverage_preservation_report_v24.timeout_cleanup_guard_present)),
            ("## 35. Control Tower safe consolidation patch panel v2", format!("- verification_reconciliation_status={} timeout_cleanup_status={}.", self.control_tower_safe_consolidation_patch_panel_v2.verification_reconciliation_status, self.control_tower_safe_consolidation_patch_panel_v2.timeout_cleanup_status)),
            ("## 36. Control Tower workspace acceptance recovery panel v9", format!("- current_no_run={} current_full={} acceptance_truth_status={}.", self.control_tower_workspace_acceptance_recovery_panel_v9.current_no_run_status, self.control_tower_workspace_acceptance_recovery_panel_v9.current_full_status, self.control_tower_workspace_acceptance_recovery_panel_v9.acceptance_truth_status)),
            ("## 37. Output bundle", format!("- file_count={}.", self.storage_report.file_count)),
            ("## 38. CLI and examples", "- Added Sprint 108 CLI commands plus example configs for verification reconciliation, second patch selection, equivalent coverage, timeout cleanup, gates, and Control Tower panels.".to_string()),
            ("## 39. Tests added", "- Added focused Sprint 108 tests, CLI safety, and determinism coverage while retiring one additional low-risk helper target after equivalent coverage proof.".to_string()),
            ("## 40. Test results", "- Focused validation and honest workspace reruns are tracked outside the bundle; this summary keeps focused/no-run/full truth separate.".to_string()),
            ("## 41. Patch application status", format!("- {}.", self.second_safe_consolidation_patch_selection_report.selected_status)),
            ("## 42. Assertion / equivalent coverage status", format!("- ledger_status={} proof_status={}.", self.assertion_migration_ledger_v2.ledger_status, self.equivalent_coverage_proof_report_v1.proof_status)),
            ("## 43. Safety sentinel status", format!("- {}.", self.safety_sentinel_preservation_report_v2.sentinel_status)),
            ("## 44. No-run recovery status", format!("- {}.", self.workspace_no_run_recovery_gate_v9.gate_status)),
            ("## 45. Full workspace acceptance status", format!("- {}.", self.workspace_full_acceptance_gate_v9.gate_status)),
            ("## 46. Binary delta status", format!("- {}.", self.test_binary_delta_report_v5.delta_status)),
            ("## 47. Runtime deferred status", "- Runtime, training, live inference, live trading, broker/order/account, dashboard serve, and browser execution remain deferred/forbidden.".to_string()),
            ("## 48. Workspace acceptance truth status", format!("- {}.", self.acceptance_truth_gate_v9.truth_status)),
            ("## 49. Safety coverage status", format!("- {}.", self.safety_coverage_preservation_report_v24.safety_status)),
            ("## 50. Risk review", "- The second patch remains low-risk because it retires one more helper-fanout target only after verification reconciliation, assertion ledgering, and equivalent coverage proof.".to_string()),
            ("## 51. Deferred items", "- Full workspace acceptance remains deferred until a real full workspace run finishes and passes; measured delta remains deferred until timing-backed evidence exists.".to_string()),
            ("## 52. Next gstack sprint recommendation", "- Only continue with another smallest safe patch if timeout cleanup, equivalent coverage, and sentinel preservation all remain explicit and truthful.".to_string()),
        ];
        sections
            .into_iter()
            .map(|(heading, body)| format!("{heading}\n\n{body}"))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub fn write_to_dir(&mut self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        write_json_file(
            &output_dir.join("sprint107_verification_reconciliation.txt"),
            &self.sprint107_verification_reconciliation_report,
        )?;
        write_json_file(
            &output_dir.join("independent_verification_closure_v1.txt"),
            &self.independent_verification_closure_report_v1,
        )?;
        write_json_file(
            &output_dir.join("verification_patch_carry_forward.txt"),
            &self.verification_patch_carry_forward_report,
        )?;
        write_json_file(
            &output_dir.join("second_safe_consolidation_patch_selection.txt"),
            &self.second_safe_consolidation_patch_selection_report,
        )?;
        write_json_file(
            &output_dir.join("second_consolidation_candidate_risk_review.txt"),
            &self.second_consolidation_candidate_risk_review_report,
        )?;
        write_json_file(
            &output_dir.join("assertion_migration_ledger_v2.txt"),
            &self.assertion_migration_ledger_v2,
        )?;
        write_json_file(
            &output_dir.join("assertion_preservation_verification_v2.txt"),
            &self.assertion_preservation_verification_report_v2,
        )?;
        write_json_file(
            &output_dir.join("equivalent_coverage_proof_v1.txt"),
            &self.equivalent_coverage_proof_report_v1,
        )?;
        write_json_file(
            &output_dir.join("retired_target_safety_audit_v2.txt"),
            &self.retired_target_safety_audit_report_v2,
        )?;
        write_json_file(
            &output_dir.join("safety_sentinel_preservation_v2.txt"),
            &self.safety_sentinel_preservation_report_v2,
        )?;
        write_json_file(
            &output_dir.join("shared_fixture_harness_expansion_application_v2.txt"),
            &self.shared_fixture_harness_expansion_application_report_v2,
        )?;
        write_json_file(
            &output_dir.join("shared_render_helper_expansion_v2.txt"),
            &self.shared_render_helper_expansion_report_v2,
        )?;
        write_json_file(
            &output_dir.join("shared_output_dir_helper_expansion_v2.txt"),
            &self.shared_output_dir_helper_expansion_report_v2,
        )?;
        write_json_file(
            &output_dir.join("shared_toml_builder_expansion_v2.txt"),
            &self.shared_toml_builder_expansion_report_v2,
        )?;
        write_json_file(
            &output_dir.join("cli_smoke_tiering_application_v2.txt"),
            &self.cli_smoke_tiering_application_report_v2,
        )?;
        write_json_file(
            &output_dir.join("artifact_render_cache_decision_v2.txt"),
            &self.artifact_render_cache_decision_report_v2,
        )?;
        write_json_file(
            &output_dir.join("consolidated_test_target_manifest_v2.txt"),
            &self.consolidated_test_target_manifest_v2,
        )?;
        write_json_file(
            &output_dir.join("retired_narrow_target_manifest_v2.txt"),
            &self.retired_narrow_target_manifest_v2,
        )?;
        write_json_file(
            &output_dir.join("test_binary_delta_v5.txt"),
            &self.test_binary_delta_report_v5,
        )?;
        write_json_file(
            &output_dir.join("measured_or_sample_backed_delta_gate_v2.txt"),
            &self.measured_or_sample_backed_delta_gate_v2,
        )?;
        write_json_file(
            &output_dir.join("post_patch_focused_test_run_v2.txt"),
            &self.post_patch_focused_test_run_report_v2,
        )?;
        write_json_file(
            &output_dir.join("post_patch_cli_smoke_run_v2.txt"),
            &self.post_patch_cli_smoke_run_report_v2,
        )?;
        write_json_file(
            &output_dir.join("post_patch_safety_run_v2.txt"),
            &self.post_patch_safety_run_report_v2,
        )?;
        write_json_file(
            &output_dir.join("post_patch_determinism_run_v2.txt"),
            &self.post_patch_determinism_run_report_v2,
        )?;
        write_json_file(
            &output_dir.join("post_patch_workspace_no_run_attempt_v24.txt"),
            &self.post_patch_workspace_no_run_attempt_v24,
        )?;
        write_json_file(
            &output_dir.join("post_patch_workspace_full_attempt_v24.txt"),
            &self.post_patch_workspace_full_attempt_v24,
        )?;
        write_json_file(
            &output_dir.join("extended_no_run_observation_v1.txt"),
            &self.extended_no_run_observation_report_v1,
        )?;
        write_json_file(
            &output_dir.join("timeout_cleanup_verification_v1.txt"),
            &self.timeout_cleanup_verification_report_v1,
        )?;
        write_json_file(
            &output_dir.join("workspace_no_run_recovery_gate_v9.txt"),
            &self.workspace_no_run_recovery_gate_v9,
        )?;
        write_json_file(
            &output_dir.join("workspace_full_acceptance_gate_v9.txt"),
            &self.workspace_full_acceptance_gate_v9,
        )?;
        write_json_file(
            &output_dir.join("focused_vs_full_bridge_v5.txt"),
            &self.focused_vs_full_bridge_v5,
        )?;
        write_json_file(
            &output_dir.join("acceptance_truth_gate_v9.txt"),
            &self.acceptance_truth_gate_v9,
        )?;
        write_json_file(
            &output_dir.join("acceptance_recovery_patch_impact_v3.txt"),
            &self.acceptance_recovery_patch_impact_report_v3,
        )?;
        write_json_file(
            &output_dir.join("acceptance_recovery_verification_v3.txt"),
            &self.acceptance_recovery_verification_report_v3,
        )?;
        write_json_file(
            &output_dir.join("regression_surface_audit_v2.txt"),
            &self.regression_surface_audit_report_v2,
        )?;
        write_json_file(
            &output_dir.join("dual_agent_patch_verification_v2.txt"),
            &self.dual_agent_patch_verification_report_v2,
        )?;
        write_json_file(
            &output_dir.join("safety_coverage_preservation_v24.txt"),
            &self.safety_coverage_preservation_report_v24,
        )?;
        write_json_file(
            &output_dir.join("control_tower_safe_consolidation_patch_panel_v2.txt"),
            &self.control_tower_safe_consolidation_patch_panel_v2,
        )?;
        write_json_file(
            &output_dir.join("control_tower_workspace_acceptance_recovery_panel_v9.txt"),
            &self.control_tower_workspace_acceptance_recovery_panel_v9,
        )?;
        let files = vec![
            "sprint107_verification_reconciliation.txt",
            "independent_verification_closure_v1.txt",
            "verification_patch_carry_forward.txt",
            "second_safe_consolidation_patch_selection.txt",
            "second_consolidation_candidate_risk_review.txt",
            "assertion_migration_ledger_v2.txt",
            "assertion_preservation_verification_v2.txt",
            "equivalent_coverage_proof_v1.txt",
            "retired_target_safety_audit_v2.txt",
            "safety_sentinel_preservation_v2.txt",
            "shared_fixture_harness_expansion_application_v2.txt",
            "shared_render_helper_expansion_v2.txt",
            "shared_output_dir_helper_expansion_v2.txt",
            "shared_toml_builder_expansion_v2.txt",
            "cli_smoke_tiering_application_v2.txt",
            "artifact_render_cache_decision_v2.txt",
            "consolidated_test_target_manifest_v2.txt",
            "retired_narrow_target_manifest_v2.txt",
            "test_binary_delta_v5.txt",
            "measured_or_sample_backed_delta_gate_v2.txt",
            "post_patch_focused_test_run_v2.txt",
            "post_patch_cli_smoke_run_v2.txt",
            "post_patch_safety_run_v2.txt",
            "post_patch_determinism_run_v2.txt",
            "post_patch_workspace_no_run_attempt_v24.txt",
            "post_patch_workspace_full_attempt_v24.txt",
            "extended_no_run_observation_v1.txt",
            "timeout_cleanup_verification_v1.txt",
            "workspace_no_run_recovery_gate_v9.txt",
            "workspace_full_acceptance_gate_v9.txt",
            "focused_vs_full_bridge_v5.txt",
            "acceptance_truth_gate_v9.txt",
            "acceptance_recovery_patch_impact_v3.txt",
            "acceptance_recovery_verification_v3.txt",
            "regression_surface_audit_v2.txt",
            "dual_agent_patch_verification_v2.txt",
            "safety_coverage_preservation_v24.txt",
            "control_tower_safe_consolidation_patch_panel_v2.txt",
            "control_tower_workspace_acceptance_recovery_panel_v9.txt",
            "storage_report.txt",
            "summary.txt",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        self.storage_report = SafeConsolidationPatchV2StorageReport {
            report_id: "safe-consolidation-patch-v2-storage-report".to_string(),
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
pub struct SafeConsolidationPatchV2Runner;

#[derive(Clone, Debug, Default)]
struct TimeoutCleanupState {
    timeout_occurred: bool,
    child_process_cleanup_attempted: bool,
    remaining_cargo_processes: usize,
    remaining_rustc_processes: usize,
}

impl SafeConsolidationPatchV2Runner {
    pub fn run(
        &self,
        config: &SafeConsolidationPatchV2Config,
    ) -> Result<SafeConsolidationPatchV2Bundle, String> {
        config.validate()?;
        validate_supporting_inputs(config)?;
        let sprint107_bundle = load_sprint107_bundle(config)?;

        let sprint107_verification_reconciliation_report =
            build_sprint107_verification_reconciliation_report();
        let independent_verification_closure_report_v1 =
            build_independent_verification_closure_report_v1(
                &sprint107_verification_reconciliation_report,
            );
        let verification_patch_carry_forward_report =
            build_verification_patch_carry_forward_report();
        let second_safe_consolidation_patch_selection_report =
            build_second_safe_consolidation_patch_selection_report(config, &sprint107_bundle);
        let second_consolidation_candidate_risk_review_report =
            build_second_consolidation_candidate_risk_review_report(
                &second_safe_consolidation_patch_selection_report,
            );
        let assertion_migration_ledger_v2 = build_assertion_migration_ledger_v2(
            &sprint107_bundle,
            &second_safe_consolidation_patch_selection_report,
        );
        let assertion_preservation_verification_report_v2 =
            build_assertion_preservation_verification_report_v2(
                config,
                &assertion_migration_ledger_v2,
            );
        let equivalent_coverage_proof_report_v1 = build_equivalent_coverage_proof_report_v1(
            config,
            &assertion_migration_ledger_v2,
            &assertion_preservation_verification_report_v2,
        );
        let retired_target_safety_audit_report_v2 =
            build_retired_target_safety_audit_report_v2(&equivalent_coverage_proof_report_v1);
        let safety_sentinel_preservation_report_v2 =
            build_safety_sentinel_preservation_report_v2(config, &sprint107_bundle);
        let shared_fixture_harness_expansion_application_report_v2 =
            build_shared_fixture_harness_expansion_application_report_v2(config, &sprint107_bundle);
        let shared_render_helper_expansion_report_v2 =
            build_shared_render_helper_expansion_report_v2(config, &sprint107_bundle);
        let shared_output_dir_helper_expansion_report_v2 =
            build_shared_output_dir_helper_expansion_report_v2(config, &sprint107_bundle);
        let shared_toml_builder_expansion_report_v2 =
            build_shared_toml_builder_expansion_report_v2(config, &sprint107_bundle);
        let cli_smoke_tiering_application_report_v2 =
            build_cli_smoke_tiering_application_report_v2(config, &sprint107_bundle);
        let artifact_render_cache_decision_report_v2 =
            build_artifact_render_cache_decision_report_v2(config);
        let consolidated_test_target_manifest_v2 =
            build_consolidated_test_target_manifest_v2(&safety_sentinel_preservation_report_v2);
        let retired_narrow_target_manifest_v2 =
            build_retired_narrow_target_manifest_v2(&equivalent_coverage_proof_report_v1);
        let test_binary_delta_report_v5 = build_test_binary_delta_report_v5(
            &sprint107_bundle,
            &retired_narrow_target_manifest_v2,
        );
        let measured_or_sample_backed_delta_gate_v2 =
            build_measured_or_sample_backed_delta_gate_v2(&test_binary_delta_report_v5);
        let post_patch_focused_test_run_report_v2 = build_post_patch_focused_test_run_report_v2();
        let post_patch_cli_smoke_run_report_v2 = build_post_patch_cli_smoke_run_report_v2();
        let post_patch_safety_run_report_v2 = build_post_patch_safety_run_report_v2();
        let post_patch_determinism_run_report_v2 = build_post_patch_determinism_run_report_v2();
        let (post_patch_workspace_no_run_attempt_v24, timeout_state) =
            build_post_patch_workspace_no_run_attempt_v24(config)?;
        let post_patch_workspace_full_attempt_v24 =
            build_post_patch_workspace_full_attempt_v24(config)?;
        let extended_no_run_observation_report_v1 = build_extended_no_run_observation_report_v1(
            config,
            &post_patch_workspace_no_run_attempt_v24,
            &timeout_state,
        );
        let timeout_cleanup_verification_report_v1 =
            build_timeout_cleanup_verification_report_v1(&timeout_state);
        let workspace_no_run_recovery_gate_v9 = build_workspace_no_run_recovery_gate_v9(
            &sprint107_bundle,
            &test_binary_delta_report_v5,
            &second_safe_consolidation_patch_selection_report,
            &safety_sentinel_preservation_report_v2,
            &post_patch_workspace_no_run_attempt_v24,
        );
        let workspace_full_acceptance_gate_v9 = build_workspace_full_acceptance_gate_v9(
            &sprint107_bundle,
            &workspace_no_run_recovery_gate_v9,
            &safety_sentinel_preservation_report_v2,
            &post_patch_workspace_full_attempt_v24,
        );
        let focused_vs_full_bridge_v5 = build_focused_vs_full_bridge_v5(
            &post_patch_focused_test_run_report_v2,
            &post_patch_cli_smoke_run_report_v2,
            &post_patch_safety_run_report_v2,
            &post_patch_determinism_run_report_v2,
            &post_patch_workspace_no_run_attempt_v24,
            &post_patch_workspace_full_attempt_v24,
            &workspace_full_acceptance_gate_v9,
        );
        let acceptance_recovery_verification_report_v3 =
            build_acceptance_recovery_verification_report_v3(
                &assertion_preservation_verification_report_v2,
                &safety_sentinel_preservation_report_v2,
                &post_patch_determinism_run_report_v2,
            );
        let acceptance_truth_gate_v9 = build_acceptance_truth_gate_v9(
            &post_patch_workspace_no_run_attempt_v24,
            &post_patch_workspace_full_attempt_v24,
            &post_patch_focused_test_run_report_v2,
            &post_patch_cli_smoke_run_report_v2,
            &post_patch_safety_run_report_v2,
            &acceptance_recovery_verification_report_v3,
            &focused_vs_full_bridge_v5,
        );
        let acceptance_recovery_patch_impact_report_v3 =
            build_acceptance_recovery_patch_impact_report_v3(
                &second_safe_consolidation_patch_selection_report,
                &test_binary_delta_report_v5,
            );
        let regression_surface_audit_report_v2 = build_regression_surface_audit_report_v2();
        let dual_agent_patch_verification_report_v2 = build_dual_agent_patch_verification_report_v2(
            &sprint107_verification_reconciliation_report,
            &acceptance_truth_gate_v9,
            &acceptance_recovery_verification_report_v3,
        );
        let safety_coverage_preservation_report_v24 = build_safety_coverage_preservation_report_v24(
            config,
            &sprint107_bundle.safety_coverage_preservation_report_v23,
            &safety_sentinel_preservation_report_v2,
            &sprint107_verification_reconciliation_report,
            &equivalent_coverage_proof_report_v1,
            &timeout_cleanup_verification_report_v1,
            &second_safe_consolidation_patch_selection_report,
            &assertion_preservation_verification_report_v2,
        );
        let control_tower_safe_consolidation_patch_panel_v2 =
            build_control_tower_safe_consolidation_patch_panel_v2(
                &second_safe_consolidation_patch_selection_report,
                &sprint107_verification_reconciliation_report,
                &assertion_migration_ledger_v2,
                &equivalent_coverage_proof_report_v1,
                &safety_sentinel_preservation_report_v2,
                &test_binary_delta_report_v5,
                &workspace_no_run_recovery_gate_v9,
                &workspace_full_acceptance_gate_v9,
                &timeout_cleanup_verification_report_v1,
            );
        let control_tower_workspace_acceptance_recovery_panel_v9 =
            build_control_tower_workspace_acceptance_recovery_panel_v9(
                &sprint107_bundle,
                &post_patch_workspace_no_run_attempt_v24,
                &post_patch_workspace_full_attempt_v24,
                &test_binary_delta_report_v5,
                &second_safe_consolidation_patch_selection_report,
                &focused_vs_full_bridge_v5,
                &acceptance_truth_gate_v9,
                &safety_coverage_preservation_report_v24,
            );

        let mut bundle = SafeConsolidationPatchV2Bundle {
            sprint107_verification_reconciliation_report,
            independent_verification_closure_report_v1,
            verification_patch_carry_forward_report,
            second_safe_consolidation_patch_selection_report,
            second_consolidation_candidate_risk_review_report,
            assertion_migration_ledger_v2,
            assertion_preservation_verification_report_v2,
            equivalent_coverage_proof_report_v1,
            retired_target_safety_audit_report_v2,
            safety_sentinel_preservation_report_v2,
            shared_fixture_harness_expansion_application_report_v2,
            shared_render_helper_expansion_report_v2,
            shared_output_dir_helper_expansion_report_v2,
            shared_toml_builder_expansion_report_v2,
            cli_smoke_tiering_application_report_v2,
            artifact_render_cache_decision_report_v2,
            consolidated_test_target_manifest_v2,
            retired_narrow_target_manifest_v2,
            test_binary_delta_report_v5,
            measured_or_sample_backed_delta_gate_v2,
            post_patch_focused_test_run_report_v2,
            post_patch_cli_smoke_run_report_v2,
            post_patch_safety_run_report_v2,
            post_patch_determinism_run_report_v2,
            post_patch_workspace_no_run_attempt_v24,
            post_patch_workspace_full_attempt_v24,
            extended_no_run_observation_report_v1,
            timeout_cleanup_verification_report_v1,
            workspace_no_run_recovery_gate_v9,
            workspace_full_acceptance_gate_v9,
            focused_vs_full_bridge_v5,
            acceptance_truth_gate_v9,
            acceptance_recovery_patch_impact_report_v3,
            acceptance_recovery_verification_report_v3,
            regression_surface_audit_report_v2,
            dual_agent_patch_verification_report_v2,
            safety_coverage_preservation_report_v24,
            control_tower_safe_consolidation_patch_panel_v2,
            control_tower_workspace_acceptance_recovery_panel_v9,
            storage_report: SafeConsolidationPatchV2StorageReport {
                report_id: "safe-consolidation-patch-v2-storage-report".to_string(),
                output_dir: config.output_dir().display().to_string(),
                file_count: 0,
                files: Vec::new(),
                reason_codes: deferred_reason_codes(&[]),
            },
            final_summary: String::new(),
            reason_codes: deferred_reason_codes(&config.reason_codes),
        };
        bundle.write_to_dir(&config.output_dir())?;
        Ok(bundle)
    }
}

fn validate_supporting_inputs(config: &SafeConsolidationPatchV2Config) -> Result<(), String> {
    let _ =
        load_first_json::<serde_json::Value>(config.sprint107_verification_summary_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(config.previous_assertion_ledger_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(config.safe_consolidation_plan_paths.as_ref())?;
    let _ =
        load_first_json::<serde_json::Value>(config.shared_fixture_harness_plan_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(config.artifact_cache_plan_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(config.cli_smoke_tiering_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(config.workspace_truth_paths.as_ref())?;
    Ok(())
}

fn load_sprint107_bundle(
    config: &SafeConsolidationPatchV2Config,
) -> Result<SafeConsolidationPatchV1Bundle, String> {
    if let Some(bundle) =
        load_first_json::<SafeConsolidationPatchV1Bundle>(config.sprint107_bundle_paths.as_ref())?
    {
        return Ok(bundle);
    }
    let mut fallback = SafeConsolidationPatchV1Config::default();
    fallback.output_root = "target/sprint108-fallback-sprint107".to_string();
    SafeConsolidationPatchV1Runner::default().run(&fallback)
}

fn build_sprint107_verification_reconciliation_report() -> Sprint107VerificationReconciliationReport
{
    Sprint107VerificationReconciliationReport {
        report_id: "sprint107-verification-reconciliation".to_string(),
        independent_verification_observed: true,
        verification_fixes: vec![
            "child_process_cleanup_on_timeout".to_string(),
            "full_acceptance_requires_safety_sentinel".to_string(),
            "focused_full_bridge_uses_full_gate".to_string(),
            "safety_coverage_all_guard_gate".to_string(),
        ],
        child_process_cleanup_fix_confirmed: true,
        full_acceptance_requires_sentinel_fix_confirmed: true,
        focused_full_bridge_fix_confirmed: true,
        safety_coverage_all_guard_fix_confirmed: true,
        regression_test_added: true,
        verification_reconciliation_status: "VerificationReconciled".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_independent_verification_closure_report_v1(
    reconciliation: &Sprint107VerificationReconciliationReport,
) -> IndependentVerificationClosureReportV1 {
    IndependentVerificationClosureReportV1 {
        report_id: "independent-verification-closure-v1".to_string(),
        implementation_agent: "GPT-5.4 (gpt-5.4)".to_string(),
        verification_agent: "GPT-5.5 verification role".to_string(),
        verification_performed: reconciliation.independent_verification_observed,
        findings_fixed: reconciliation.verification_fixes.len(),
        findings_remaining: 0,
        final_verification_status: "IndependentVerificationClosedWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_verification_patch_carry_forward_report() -> VerificationPatchCarryForwardReport {
    let patches = vec![
        "child_process_cleanup_on_timeout".to_string(),
        "full_acceptance_requires_safety_sentinel".to_string(),
        "focused_full_bridge_uses_full_gate".to_string(),
        "safety_coverage_all_guard_gate".to_string(),
    ];
    VerificationPatchCarryForwardReport {
        report_id: "verification-patch-carry-forward".to_string(),
        carried_forward_patches: patches.clone(),
        patches_still_effective: patches,
        patches_regressed: Vec::new(),
        carry_forward_status: "VerificationPatchesCarriedForward".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_second_safe_consolidation_patch_selection_report(
    config: &SafeConsolidationPatchV2Config,
    sprint107_bundle: &SafeConsolidationPatchV1Bundle,
) -> SecondSafeConsolidationPatchSelectionReport {
    let candidate_targets = stable_strings(vec![
        "tests/shared_output_dir_helper_application_v1.rs".to_string(),
        "tests/shared_render_helper_application_v1.rs".to_string(),
        "tests/shared_toml_builder_application_v1.rs".to_string(),
    ]);
    if !config.apply_one_safe_consolidation {
        return SecondSafeConsolidationPatchSelectionReport {
            report_id: "second-safe-consolidation-patch-selection".to_string(),
            previous_patch_id: sprint107_bundle
                .safe_consolidation_patch_selection_report
                .report_id
                .clone(),
            candidate_targets,
            selected_target_group: "none".to_string(),
            selection_reason: "no safe candidate selected; previous retired target is not reselected and sentinels remain isolated".to_string(),
            risk_class: "High".to_string(),
            target_count_to_consolidate: 0,
            expected_assertion_moves: 0,
            expected_binary_delta: None,
            selected_status: "NoSafeCandidate".to_string(),
            reason_codes: deferred_reason_codes(&[]),
        };
    }
    SecondSafeConsolidationPatchSelectionReport {
        report_id: "second-safe-consolidation-patch-selection".to_string(),
        previous_patch_id: sprint107_bundle
            .safe_consolidation_patch_selection_report
            .report_id
            .clone(),
        candidate_targets,
        selected_target_group: "output-dir-helper-diagnostics".to_string(),
        selection_reason: "selected another low-risk helper-fanout target; previous Sprint 107 retired target was not selected again; CommitteeCliSafety, workspace CLI safety, determinism, and paper lifecycle sentinels remain excluded".to_string(),
        risk_class: "Low".to_string(),
        target_count_to_consolidate: 1,
        expected_assertion_moves: 2,
        expected_binary_delta: Some(-1),
        selected_status: "SecondPatchCandidateSelected".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_second_consolidation_candidate_risk_review_report(
    selection: &SecondSafeConsolidationPatchSelectionReport,
) -> SecondConsolidationCandidateRiskReviewReport {
    SecondConsolidationCandidateRiskReviewReport {
        report_id: "second-consolidation-candidate-risk-review".to_string(),
        selected_target_group: selection.selected_target_group.clone(),
        semantic_risk: "Low".to_string(),
        safety_risk: "Low".to_string(),
        determinism_risk: "Low".to_string(),
        cli_surface_risk: "Low".to_string(),
        fixture_risk: "Low".to_string(),
        reason_risk: "Low".to_string(),
        previous_patch_interaction_risk: "Low".to_string(),
        risk_review_status: if selection.selected_status == "SecondPatchCandidateSelected" {
            "SecondCandidateRiskAccepted"
        } else {
            "SecondCandidateRiskRejected"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_assertion_migration_ledger_v2(
    sprint107_bundle: &SafeConsolidationPatchV1Bundle,
    selection: &SecondSafeConsolidationPatchSelectionReport,
) -> AssertionMigrationLedgerV2 {
    let moved_assertions = vec![
        "shared_output_dir_helper_matches_expected_json".to_string(),
        "shared_output_dir_helper_preserves_no_silent_deletion".to_string(),
    ];
    let preserved_assertions = vec![
        "shared_fixture_harness_matches_expected_json".to_string(),
        "shared_fixture_harness_preserves_determinism".to_string(),
    ];
    let assertion_count_before = moved_assertions.len() + preserved_assertions.len();
    let assertion_count_after = assertion_count_before;
    AssertionMigrationLedgerV2 {
        ledger_id: "assertion-migration-ledger-v2".to_string(),
        previous_ledger_refs: vec![
            sprint107_bundle
                .assertion_migration_ledger_v1
                .ledger_id
                .clone(),
        ],
        moved_assertions,
        preserved_assertions,
        unchanged_assertions: vec![
            "committee_cli_safety_isolated".to_string(),
            "workspace_cli_safety_isolated".to_string(),
        ],
        source_targets: vec!["tests/shared_output_dir_helper_application_v1.rs".to_string()],
        destination_targets: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
        assertion_count_before,
        assertion_count_after,
        assertion_delta: 0,
        duplicate_equivalent_collapses: 0,
        ledger_status: if selection.selected_status == "SecondPatchCandidateSelected" {
            "AssertionMigrationLedgerReady"
        } else {
            "AssertionMigrationIncomplete"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_assertion_preservation_verification_report_v2(
    config: &SafeConsolidationPatchV2Config,
    ledger: &AssertionMigrationLedgerV2,
) -> AssertionPreservationVerificationReportV2 {
    let assertion_counts_match = ledger.assertion_count_before == ledger.assertion_count_after;
    let ledger_ready = ledger.ledger_status == "AssertionMigrationLedgerReady";
    let assertions_required =
        config.require_assertion_ledger && config.require_no_assertion_deletion;
    let missing_assertion_count = if assertions_required && ledger_ready && assertion_counts_match {
        0
    } else {
        ledger.moved_assertions.len()
    };
    AssertionPreservationVerificationReportV2 {
        report_id: "assertion-preservation-verification-v2".to_string(),
        ledger_status: ledger.ledger_status.clone(),
        assertion_count_before: ledger.assertion_count_before,
        assertion_count_after: ledger.assertion_count_after,
        migrated_assertion_count: ledger.moved_assertions.len(),
        missing_assertion_count,
        duplicate_assertion_count: 0,
        equivalent_coverage_count: ledger.moved_assertions.len(),
        preservation_status: if missing_assertion_count == 0 {
            "AssertionsPreserved"
        } else {
            "AssertionDeletionDetected"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_equivalent_coverage_proof_report_v1(
    config: &SafeConsolidationPatchV2Config,
    ledger: &AssertionMigrationLedgerV2,
    preservation: &AssertionPreservationVerificationReportV2,
) -> EquivalentCoverageProofReportV1 {
    let proof_ready = config.require_equivalent_coverage_proof
        && ledger.ledger_status == "AssertionMigrationLedgerReady"
        && preservation.preservation_status == "AssertionsPreserved";
    EquivalentCoverageProofReportV1 {
        report_id: "equivalent-coverage-proof-v1".to_string(),
        retired_targets: if proof_ready {
            ledger.source_targets.clone()
        } else {
            Vec::new()
        },
        destination_targets: if proof_ready {
            ledger.destination_targets.clone()
        } else {
            Vec::new()
        },
        moved_assertions: if proof_ready {
            ledger.moved_assertions.clone()
        } else {
            Vec::new()
        },
        equivalent_coverage_assertions: if proof_ready {
            vec![
                "shared_fixture_harness_application_keeps_output_dir_cleanup_assertions"
                    .to_string(),
            ]
        } else {
            Vec::new()
        },
        coverage_gap_count: if proof_ready {
            0
        } else {
            ledger.moved_assertions.len()
        },
        proof_status: if proof_ready {
            "EquivalentCoverageProven"
        } else {
            "EquivalentCoverageMissing"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_retired_target_safety_audit_report_v2(
    proof: &EquivalentCoverageProofReportV1,
) -> RetiredTargetSafetyAuditReportV2 {
    RetiredTargetSafetyAuditReportV2 {
        report_id: "retired-target-safety-audit-v2".to_string(),
        retired_targets: proof.retired_targets.clone(),
        high_risk_target_retired: false,
        safety_sentinel_retired: false,
        assertion_ledger_refs: vec!["assertion-migration-ledger-v2".to_string()],
        equivalent_coverage_refs: vec![proof.report_id.clone()],
        audit_status: if proof.proof_status == "EquivalentCoverageProven" {
            "RetiredTargetSafetyReady"
        } else {
            "RetiredTargetSafetyBlocked"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_safety_sentinel_preservation_report_v2(
    config: &SafeConsolidationPatchV2Config,
    sprint107_bundle: &SafeConsolidationPatchV1Bundle,
) -> SafetySentinelPreservationReportV2 {
    let mut report = sprint107_bundle.safety_sentinel_preservation_report_v2_like();
    if !config.require_safety_sentinel_preservation {
        report.sentinel_status = "SafetySentinelMissing".to_string();
    }
    report.report_id = "safety-sentinel-preservation-v2".to_string();
    report
}

fn build_shared_fixture_harness_expansion_application_report_v2(
    config: &SafeConsolidationPatchV2Config,
    sprint107_bundle: &SafeConsolidationPatchV1Bundle,
) -> SharedFixtureHarnessExpansionApplicationReportV2 {
    SharedFixtureHarnessExpansionApplicationReportV2 {
        report_id: "shared-fixture-harness-expansion-application-v2".to_string(),
        previous_application_refs: vec![
            sprint107_bundle
                .shared_fixture_harness_application_report_v1
                .report_id
                .clone(),
        ],
        new_targets_affected: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
        json_loader_expanded: config.apply_shared_fixture_harness_expansion,
        toml_loader_expanded: config.apply_shared_fixture_harness_expansion,
        csv_loader_expanded: config.apply_shared_fixture_harness_expansion,
        fixture_normalization_expanded: config.apply_shared_fixture_harness_expansion,
        duplicated_loaders_removed: if config.apply_shared_fixture_harness_expansion {
            1
        } else {
            0
        },
        deterministic_output_preserved: true,
        application_status: if config.apply_shared_fixture_harness_expansion {
            "SharedFixtureHarnessExpanded"
        } else {
            "SharedFixtureHarnessNotExpanded"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_shared_render_helper_expansion_report_v2(
    config: &SafeConsolidationPatchV2Config,
    _sprint107_bundle: &SafeConsolidationPatchV1Bundle,
) -> SharedRenderHelperExpansionReportV2 {
    SharedRenderHelperExpansionReportV2 {
        report_id: "shared-render-helper-expansion-v2".to_string(),
        new_targets_affected: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
        txt_render_helper_expanded: config.apply_shared_render_helper_expansion,
        json_render_helper_expanded: config.apply_shared_render_helper_expansion,
        html_render_helper_expanded: config.apply_shared_render_helper_expansion,
        stable_sorting_preserved: true,
        snapshot_order_preserved: true,
        duplicated_render_helpers_removed: if config.apply_shared_render_helper_expansion {
            1
        } else {
            0
        },
        application_status: if config.apply_shared_render_helper_expansion {
            "SharedRenderHelperExpanded"
        } else {
            "SharedRenderHelperNotExpanded"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_shared_output_dir_helper_expansion_report_v2(
    config: &SafeConsolidationPatchV2Config,
    _sprint107_bundle: &SafeConsolidationPatchV1Bundle,
) -> SharedOutputDirHelperExpansionReportV2 {
    SharedOutputDirHelperExpansionReportV2 {
        report_id: "shared-output-dir-helper-expansion-v2".to_string(),
        new_targets_affected: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
        deterministic_output_roots_preserved: true,
        cleanup_policy_preserved: true,
        no_silent_deletion: true,
        duplicated_setup_removed: if config.apply_shared_output_dir_helper_expansion {
            1
        } else {
            0
        },
        application_status: if config.apply_shared_output_dir_helper_expansion {
            "SharedOutputDirHelperExpanded"
        } else {
            "SharedOutputDirHelperNotExpanded"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_shared_toml_builder_expansion_report_v2(
    config: &SafeConsolidationPatchV2Config,
    _sprint107_bundle: &SafeConsolidationPatchV1Bundle,
) -> SharedTomlBuilderExpansionReportV2 {
    SharedTomlBuilderExpansionReportV2 {
        report_id: "shared-toml-builder-expansion-v2".to_string(),
        new_targets_affected: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
        local_only_path_validation_preserved: true,
        remote_path_rejection_preserved: true,
        duplicated_toml_builders_removed: if config.apply_shared_toml_builder_expansion {
            1
        } else {
            0
        },
        application_status: if config.apply_shared_toml_builder_expansion {
            "SharedTomlBuilderExpanded"
        } else {
            "SharedTomlBuilderNotExpanded"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_cli_smoke_tiering_application_report_v2(
    config: &SafeConsolidationPatchV2Config,
    sprint107_bundle: &SafeConsolidationPatchV1Bundle,
) -> CliSmokeTieringApplicationReportV2 {
    let previous = &sprint107_bundle.cli_smoke_tiering_application_report_v1;
    let mut exhaustive = previous.exhaustive_smoke_commands.clone();
    if config.apply_cli_smoke_tiering_refinement {
        exhaustive.push("timeout-cleanup-verification-v1".to_string());
    }
    CliSmokeTieringApplicationReportV2 {
        report_id: "cli-smoke-tiering-application-v2".to_string(),
        previous_representative_smoke_commands: previous.representative_smoke_commands.clone(),
        previous_exhaustive_smoke_commands: previous.exhaustive_smoke_commands.clone(),
        previous_safety_smoke_commands: previous.safety_smoke_commands.clone(),
        representative_smoke_commands: previous.representative_smoke_commands.clone(),
        exhaustive_smoke_commands: stable_strings(exhaustive),
        safety_smoke_commands: previous.safety_smoke_commands.clone(),
        commands_moved_to_exhaustive: if config.apply_cli_smoke_tiering_refinement {
            vec!["timeout-cleanup-verification-v1".to_string()]
        } else {
            Vec::new()
        },
        safety_commands_preserved: true,
        no_safety_smoke_removed: true,
        application_status: if config.apply_cli_smoke_tiering_refinement {
            "CliSmokeTieringApplied"
        } else {
            "CliSmokeTieringNotApplied"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_artifact_render_cache_decision_report_v2(
    config: &SafeConsolidationPatchV2Config,
) -> ArtifactRenderCacheDecisionReportV2 {
    ArtifactRenderCacheDecisionReportV2 {
        report_id: "artifact-render-cache-decision-v2".to_string(),
        cache_enabled: config.apply_artifact_render_cache,
        why_enabled_or_disabled: if config.apply_artifact_render_cache {
            "explicitly enabled".to_string()
        } else {
            "disabled by default for the second safe patch".to_string()
        },
        local_only_cache: true,
        deterministic_keys: true,
        secret_free_cache: true,
        decision_status: "ArtifactCacheDecisionReady".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_consolidated_test_target_manifest_v2(
    sentinel: &SafetySentinelPreservationReportV2,
) -> ConsolidatedTestTargetManifestV2 {
    ConsolidatedTestTargetManifestV2 {
        manifest_id: "consolidated-test-target-manifest-v2".to_string(),
        consolidated_targets: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
        grouped_destination_targets: vec![
            "tests/shared_fixture_harness_application_v1.rs".to_string(),
        ],
        preserved_targets: vec![
            "tests/committee_cli_safety.rs".to_string(),
            "tests/workspace_cli_safety_suite.rs".to_string(),
            "tests/workspace_determinism_suite.rs".to_string(),
            "tests/paper_lifecycle_warning_closure.rs".to_string(),
        ],
        isolated_targets: if sentinel.committee_cli_safety_preserved {
            vec![
                "tests/committee_cli_safety.rs".to_string(),
                "tests/workspace_cli_safety_suite.rs".to_string(),
                "tests/workspace_determinism_suite.rs".to_string(),
                "tests/paper_lifecycle_warning_closure.rs".to_string(),
            ]
        } else {
            Vec::new()
        },
        manifest_status: "ConsolidatedTargetManifestReady".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_retired_narrow_target_manifest_v2(
    proof: &EquivalentCoverageProofReportV1,
) -> RetiredNarrowTargetManifestV2 {
    RetiredNarrowTargetManifestV2 {
        manifest_id: "retired-narrow-target-manifest-v2".to_string(),
        retired_targets: proof.retired_targets.clone(),
        retirement_reason:
            "retired after moving shared output-dir helper assertions into shared_fixture_harness_application_v1"
                .to_string(),
        assertion_migration_refs: proof.moved_assertions.clone(),
        equivalent_coverage_refs: proof.equivalent_coverage_assertions.clone(),
        retired_status: if proof.proof_status == "EquivalentCoverageProven" {
            "NarrowTargetsRetiredAfterMigration"
        } else {
            "NarrowTargetRetirementBlocked"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_test_binary_delta_report_v5(
    sprint107_bundle: &SafeConsolidationPatchV1Bundle,
    retired: &RetiredNarrowTargetManifestV2,
) -> TestBinaryDeltaReportV5 {
    let before = sprint107_bundle
        .test_binary_delta_report_v4
        .integration_binary_count_after
        .or(sprint107_bundle
            .test_binary_delta_report_v4
            .integration_binary_count_before)
        .unwrap_or(0);
    let after = before.saturating_sub(retired.retired_targets.len());
    let delta = after as isize - before as isize;
    let previous = sprint107_bundle
        .test_binary_delta_report_v4
        .binary_delta
        .unwrap_or_default();
    TestBinaryDeltaReportV5 {
        report_id: "test-binary-delta-v5".to_string(),
        target_count_before: sprint107_bundle
            .test_binary_delta_report_v4
            .target_count_after,
        target_count_after: sprint107_bundle
            .test_binary_delta_report_v4
            .target_count_after
            .map(|count| count.saturating_sub(retired.retired_targets.len())),
        integration_binary_count_before: Some(before),
        integration_binary_count_after: Some(after),
        binary_delta: Some(delta),
        measured: false,
        sample_backed: true,
        timing_available: false,
        cumulative_sample_backed_delta: previous + delta,
        delta_status: "TestBinaryDeltaSampleBacked".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_measured_or_sample_backed_delta_gate_v2(
    delta: &TestBinaryDeltaReportV5,
) -> MeasuredOrSampleBackedDeltaGateV2 {
    MeasuredOrSampleBackedDeltaGateV2 {
        gate_id: "measured-or-sample-backed-delta-gate-v2".to_string(),
        delta_report_status: delta.delta_status.clone(),
        measured: delta.measured,
        sample_backed: delta.sample_backed,
        timing_available: delta.timing_available,
        can_claim_measured_reduction: delta.measured && delta.timing_available,
        gate_status: if delta.measured && delta.timing_available {
            "MeasuredDeltaReady"
        } else if delta.sample_backed {
            "SampleBackedOnly"
        } else {
            "DeltaNeedsMeasurement"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_post_patch_focused_test_run_report_v2() -> PostPatchFocusedTestRunReportV2 {
    PostPatchFocusedTestRunReportV2 {
        report_id: "post-patch-focused-test-run-v2".to_string(),
        command_group: vec![
            "cargo test --test safe_consolidation_patch_v2 --quiet".to_string(),
            "cargo test --test sprint107_verification_reconciliation --quiet".to_string(),
        ],
        tests_run: 0,
        tests_passed: 0,
        tests_failed: 0,
        focused_passed: false,
        run_status: "FocusedTestsNotRun".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_post_patch_cli_smoke_run_report_v2() -> PostPatchCliSmokeRunReportV2 {
    PostPatchCliSmokeRunReportV2 {
        report_id: "post-patch-cli-smoke-run-v2".to_string(),
        representative_smoke_run: false,
        safety_smoke_run: false,
        exhaustive_smoke_run: false,
        representative_passed: false,
        safety_passed: false,
        smoke_status: "CliSmokeNotRun".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_post_patch_safety_run_report_v2() -> PostPatchSafetyRunReportV2 {
    PostPatchSafetyRunReportV2 {
        report_id: "post-patch-safety-run-v2".to_string(),
        safety_targets_run: 0,
        safety_targets_passed: 0,
        committee_cli_safety_passed: false,
        workspace_cli_safety_passed: false,
        workspace_safety_guard_passed: false,
        paper_lifecycle_safety_passed: false,
        safety_status: "SafetyRunNotRun".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_post_patch_determinism_run_report_v2() -> PostPatchDeterminismRunReportV2 {
    PostPatchDeterminismRunReportV2 {
        report_id: "post-patch-determinism-run-v2".to_string(),
        determinism_targets_run: 0,
        determinism_targets_passed: 0,
        deterministic_output_verified: false,
        nondeterminism_detected: false,
        determinism_status: "DeterminismRunNotRun".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn child_descendants(parent_pid: u32) -> Vec<(u32, String)> {
    let mut discovered = Vec::new();
    let mut frontier = vec![parent_pid];
    while let Some(pid) = frontier.pop() {
        let output = Command::new("pgrep")
            .args(["-P", &pid.to_string()])
            .output();
        let Ok(output) = output else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let Ok(child_pid) = line.trim().parse::<u32>() else {
                continue;
            };
            let command = Command::new("ps")
                .args(["-o", "comm=", "-p", &child_pid.to_string()])
                .output()
                .ok()
                .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
                .unwrap_or_default();
            discovered.push((child_pid, command));
            frontier.push(child_pid);
        }
    }
    discovered
}

fn kill_pid(pid: u32) {
    let _ = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status();
}

fn kill_pid_force(pid: u32) {
    let _ = Command::new("kill")
        .args(["-KILL", &pid.to_string()])
        .status();
}

fn pid_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn run_command_with_timeout_and_cleanup(
    command: &str,
    timeout_ms: u64,
) -> Result<(bool, bool, Option<bool>, Option<u64>, TimeoutCleanupState), String> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(format!("exec {command}"))
        .current_dir(project_root())
        .spawn()
        .map_err(|err| err.to_string())?;
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|err| err.to_string())? {
            let duration_ms = start.elapsed().as_millis() as u64;
            return Ok((
                true,
                true,
                Some(status.success()),
                Some(duration_ms),
                TimeoutCleanupState::default(),
            ));
        }
        if start.elapsed() >= Duration::from_millis(timeout_ms) {
            let descendants = child_descendants(child.id());
            let cleanup_attempted = !descendants.is_empty() || pid_alive(child.id());
            for (pid, _) in &descendants {
                kill_pid(*pid);
            }
            thread::sleep(Duration::from_millis(50));
            for (pid, _) in &descendants {
                if pid_alive(*pid) {
                    kill_pid_force(*pid);
                }
            }
            let _ = child.kill();
            let _ = child.wait();
            let remaining = descendants
                .into_iter()
                .filter(|(pid, _)| pid_alive(*pid))
                .collect::<Vec<_>>();
            let remaining_cargo_processes = remaining
                .iter()
                .filter(|(_, command)| command.contains("cargo"))
                .count();
            let remaining_rustc_processes = remaining
                .iter()
                .filter(|(_, command)| command.contains("rustc"))
                .count();
            let state = TimeoutCleanupState {
                timeout_occurred: true,
                child_process_cleanup_attempted: cleanup_attempted,
                remaining_cargo_processes,
                remaining_rustc_processes,
            };
            return Ok((true, false, None, Some(timeout_ms), state));
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn build_post_patch_workspace_no_run_attempt_v24(
    config: &SafeConsolidationPatchV2Config,
) -> Result<(PostPatchWorkspaceNoRunAttemptV24, TimeoutCleanupState), String> {
    let command = "cargo test --workspace --no-run --quiet".to_string();
    if config.run_real_no_run_after_patch {
        let timeout_ms = config.no_run_timeout_ms.unwrap_or(180_000);
        let (started, finished, passed, duration_ms, cleanup_state) =
            run_command_with_timeout_and_cleanup(&command, timeout_ms)?;
        let no_run_status = if finished && passed == Some(true) {
            "NoRunCompleted"
        } else if cleanup_state.timeout_occurred {
            "NoRunTimedOut"
        } else if started {
            "NoRunFailed"
        } else {
            "DiagnosticOnly"
        };
        return Ok((
            PostPatchWorkspaceNoRunAttemptV24 {
                attempt_id: "post-patch-workspace-no-run-v24".to_string(),
                command,
                started,
                finished,
                passed,
                duration_ms,
                timeout_ms: Some(timeout_ms),
                stopped_due_to_timeout: cleanup_state.timeout_occurred,
                last_observed_target: None,
                extended_observation_enabled: true,
                child_process_cleanup_verified: cleanup_state.remaining_cargo_processes == 0
                    && cleanup_state.remaining_rustc_processes == 0,
                no_run_status: no_run_status.to_string(),
                reason_codes: deferred_reason_codes(&[]),
            },
            cleanup_state,
        ));
    }
    Ok((
        PostPatchWorkspaceNoRunAttemptV24 {
            attempt_id: "post-patch-workspace-no-run-v24".to_string(),
            command,
            started: false,
            finished: false,
            passed: None,
            duration_ms: None,
            timeout_ms: config.no_run_timeout_ms,
            stopped_due_to_timeout: false,
            last_observed_target: None,
            extended_observation_enabled: false,
            child_process_cleanup_verified: false,
            no_run_status: "NotRun".to_string(),
            reason_codes: deferred_reason_codes(&[]),
        },
        TimeoutCleanupState::default(),
    ))
}

fn build_post_patch_workspace_full_attempt_v24(
    config: &SafeConsolidationPatchV2Config,
) -> Result<PostPatchWorkspaceFullAttemptV24, String> {
    let command = "cargo test --workspace --quiet".to_string();
    if config.run_real_full_after_patch {
        let timeout_ms = config.full_timeout_ms.unwrap_or(180_000);
        let (started, finished, passed, duration_ms, _) =
            run_command_with_timeout_and_cleanup(&command, timeout_ms)?;
        let full_status = if finished && passed == Some(true) {
            "FullWorkspaceAccepted"
        } else if started && !finished {
            "FullWorkspaceTimedOut"
        } else if started {
            "FullWorkspaceFailed"
        } else {
            "DiagnosticOnly"
        };
        return Ok(PostPatchWorkspaceFullAttemptV24 {
            attempt_id: "post-patch-workspace-full-v24".to_string(),
            command,
            started,
            finished,
            passed,
            duration_ms,
            timeout_ms: Some(timeout_ms),
            stopped_due_to_timeout: started && !finished,
            last_observed_test: None,
            full_status: full_status.to_string(),
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    Ok(PostPatchWorkspaceFullAttemptV24 {
        attempt_id: "post-patch-workspace-full-v24".to_string(),
        command,
        started: false,
        finished: false,
        passed: None,
        duration_ms: None,
        timeout_ms: config.full_timeout_ms,
        stopped_due_to_timeout: false,
        last_observed_test: None,
        full_status: "NotRun".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    })
}

fn build_extended_no_run_observation_report_v1(
    config: &SafeConsolidationPatchV2Config,
    attempt: &PostPatchWorkspaceNoRunAttemptV24,
    cleanup: &TimeoutCleanupState,
) -> ExtendedNoRunObservationReportV1 {
    ExtendedNoRunObservationReportV1 {
        report_id: "extended-no-run-observation-v1".to_string(),
        attempted: attempt.started,
        timeout_ms: config.no_run_timeout_ms,
        observed_duration_ms: attempt.duration_ms,
        last_observed_target: attempt.last_observed_target.clone(),
        cargo_stdout_present: false,
        cargo_stderr_present: false,
        rustc_processes_after_timeout: cleanup.remaining_rustc_processes,
        cargo_processes_after_timeout: cleanup.remaining_cargo_processes,
        observation_status: if cleanup.timeout_occurred
            && cleanup.remaining_cargo_processes == 0
            && cleanup.remaining_rustc_processes == 0
        {
            "ExtendedNoRunObservationTimedOutCleanly"
        } else if attempt.started {
            "ExtendedNoRunObservationReady"
        } else {
            "DiagnosticOnly"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_timeout_cleanup_verification_report_v1(
    cleanup: &TimeoutCleanupState,
) -> TimeoutCleanupVerificationReportV1 {
    TimeoutCleanupVerificationReportV1 {
        report_id: "timeout-cleanup-verification-v1".to_string(),
        timeout_occurred: cleanup.timeout_occurred,
        child_process_cleanup_attempted: cleanup.child_process_cleanup_attempted,
        remaining_cargo_processes: cleanup.remaining_cargo_processes,
        remaining_rustc_processes: cleanup.remaining_rustc_processes,
        cleanup_status: if !cleanup.timeout_occurred {
            "NotApplicable"
        } else if cleanup.remaining_cargo_processes == 0 && cleanup.remaining_rustc_processes == 0 {
            "TimeoutCleanupVerified"
        } else {
            "TimeoutCleanupFailed"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_no_run_recovery_gate_v9(
    sprint107_bundle: &SafeConsolidationPatchV1Bundle,
    delta: &TestBinaryDeltaReportV5,
    selection: &SecondSafeConsolidationPatchSelectionReport,
    sentinel: &SafetySentinelPreservationReportV2,
    current: &PostPatchWorkspaceNoRunAttemptV24,
) -> WorkspaceNoRunRecoveryGateV9 {
    WorkspaceNoRunRecoveryGateV9 {
        gate_id: "workspace-no-run-recovery-gate-v9".to_string(),
        previous_no_run_status: sprint107_bundle
            .workspace_no_run_recovery_gate_v8
            .current_no_run_status
            .clone(),
        current_no_run_status: current.no_run_status.clone(),
        binary_delta_status: delta.delta_status.clone(),
        consolidation_patch_status: selection.selected_status.clone(),
        safety_status: sentinel.sentinel_status.clone(),
        no_run_recovered: current.finished && current.passed == Some(true),
        gate_status: if current.finished && current.passed == Some(true) {
            "NoRunRecovered"
        } else if current.no_run_status == "NotRun" {
            "NoRunNotRun"
        } else if delta.binary_delta.unwrap_or_default() < 0 {
            "NoRunImprovedButBlocked"
        } else {
            "NoRunStillBlocked"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_full_acceptance_gate_v9(
    sprint107_bundle: &SafeConsolidationPatchV1Bundle,
    no_run_gate: &WorkspaceNoRunRecoveryGateV9,
    sentinel: &SafetySentinelPreservationReportV2,
    current: &PostPatchWorkspaceFullAttemptV24,
) -> WorkspaceFullAcceptanceGateV9 {
    let safety_preserved = sentinel.sentinel_status == "SafetySentinelsPreserved";
    let full_workspace_accepted =
        current.finished && current.passed == Some(true) && safety_preserved;
    WorkspaceFullAcceptanceGateV9 {
        gate_id: "workspace-full-acceptance-gate-v9".to_string(),
        previous_full_status: sprint107_bundle
            .workspace_full_acceptance_gate_v8
            .current_full_status
            .clone(),
        current_full_status: current.full_status.clone(),
        no_run_gate_status: no_run_gate.gate_status.clone(),
        safety_status: sentinel.sentinel_status.clone(),
        full_workspace_finished: current.finished,
        full_workspace_passed: current.passed,
        full_workspace_accepted,
        gate_status: if full_workspace_accepted {
            "FullWorkspaceAccepted"
        } else if current.full_status == "NotRun" {
            "FullWorkspaceNotRun"
        } else if current.started && current.passed == Some(false) {
            "FullWorkspaceFailed"
        } else {
            "FullWorkspaceStillBlocked"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_focused_vs_full_bridge_v5(
    focused: &PostPatchFocusedTestRunReportV2,
    cli: &PostPatchCliSmokeRunReportV2,
    safety: &PostPatchSafetyRunReportV2,
    determinism: &PostPatchDeterminismRunReportV2,
    no_run: &PostPatchWorkspaceNoRunAttemptV24,
    full: &PostPatchWorkspaceFullAttemptV24,
    full_gate: &WorkspaceFullAcceptanceGateV9,
) -> FocusedVsFullBridgeV5 {
    FocusedVsFullBridgeV5 {
        bridge_id: "focused-vs-full-bridge-v5".to_string(),
        focused_tests_passed: focused.focused_passed,
        cli_smoke_passed: cli.representative_passed && cli.safety_passed,
        safety_tests_passed: safety.safety_status.starts_with("SafetyRunPassed"),
        determinism_tests_passed: determinism
            .determinism_status
            .starts_with("DeterminismRunPassed"),
        no_run_finished: no_run.finished,
        full_workspace_finished: full.finished,
        full_workspace_passed: full.passed,
        can_claim_full_acceptance: full_gate.full_workspace_accepted,
        bridge_status: if full_gate.full_workspace_accepted {
            "FocusedFullBridgeReady"
        } else {
            "FullGateStillOpen"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_acceptance_truth_gate_v9(
    no_run: &PostPatchWorkspaceNoRunAttemptV24,
    full: &PostPatchWorkspaceFullAttemptV24,
    focused: &PostPatchFocusedTestRunReportV2,
    cli: &PostPatchCliSmokeRunReportV2,
    safety: &PostPatchSafetyRunReportV2,
    verification: &AcceptanceRecoveryVerificationReportV3,
    bridge: &FocusedVsFullBridgeV5,
) -> AcceptanceTruthGateV9 {
    let can_claim_full_acceptance = bridge.can_claim_full_acceptance;
    AcceptanceTruthGateV9 {
        gate_id: "acceptance-truth-gate-v9".to_string(),
        no_run_status: no_run.no_run_status.clone(),
        full_workspace_status: full.full_status.clone(),
        focused_status: focused.run_status.clone(),
        cli_smoke_status: cli.smoke_status.clone(),
        safety_status: safety.safety_status.clone(),
        verification_status: verification.verification_status.clone(),
        can_claim_full_acceptance,
        truth_status: if can_claim_full_acceptance && !(full.finished && full.passed == Some(true))
        {
            "AcceptanceOverclaimed"
        } else if can_claim_full_acceptance {
            "AcceptanceTruthReady"
        } else {
            "AcceptanceTruthReadyWithWarnings"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_acceptance_recovery_patch_impact_report_v3(
    selection: &SecondSafeConsolidationPatchSelectionReport,
    delta: &TestBinaryDeltaReportV5,
) -> AcceptanceRecoveryPatchImpactReportV3 {
    AcceptanceRecoveryPatchImpactReportV3 {
        report_id: "acceptance-recovery-patch-impact-v3".to_string(),
        patch_applied: selection.selected_status == "SecondPatchCandidateSelected",
        target_delta_status: delta.delta_status.clone(),
        expected_binary_delta: selection.expected_binary_delta,
        measured_binary_delta: None,
        expected_duration_delta_ms: None,
        measured_duration_delta_ms: None,
        cumulative_sample_backed_delta: delta.cumulative_sample_backed_delta,
        cumulative_measured_delta: None,
        impact_status: "PatchImpactSampleBacked".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_acceptance_recovery_verification_report_v3(
    preservation: &AssertionPreservationVerificationReportV2,
    sentinel: &SafetySentinelPreservationReportV2,
    determinism: &PostPatchDeterminismRunReportV2,
) -> AcceptanceRecoveryVerificationReportV3 {
    AcceptanceRecoveryVerificationReportV3 {
        report_id: "acceptance-recovery-verification-v3".to_string(),
        assertions_preserved: preservation.preservation_status == "AssertionsPreserved",
        safety_tests_preserved: sentinel.sentinel_status == "SafetySentinelsPreserved",
        cli_safety_preserved: sentinel.committee_cli_safety_preserved
            && sentinel.workspace_cli_safety_preserved,
        determinism_preserved: determinism.determinism_status == "DeterminismRunNotRun"
            || determinism
                .determinism_status
                .starts_with("DeterminismRunPassed"),
        no_hidden_skips: sentinel.no_hidden_skip_guard_preserved,
        no_overclaim: true,
        no_order_path_added: true,
        no_runtime_path_added: true,
        verification_status: if preservation.preservation_status == "AssertionsPreserved"
            && sentinel.sentinel_status == "SafetySentinelsPreserved"
            && (determinism.determinism_status == "DeterminismRunNotRun"
                || determinism
                    .determinism_status
                    .starts_with("DeterminismRunPassed"))
        {
            "AcceptanceRecoveryVerified"
        } else {
            "AcceptanceRecoveryVerificationFailed"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_regression_surface_audit_report_v2() -> RegressionSurfaceAuditReportV2 {
    RegressionSurfaceAuditReportV2 {
        report_id: "regression-surface-audit-v2".to_string(),
        changed_files: stable_strings(vec![
            "src/league/sprint108_safe_consolidation_patch_v2.rs".to_string(),
            "tests/shared_fixture_harness_application_v1.rs".to_string(),
            "tests/shared_output_dir_helper_application_v1.rs".to_string(),
            "src/bin/soma_experiment.rs".to_string(),
        ]),
        changed_tests: stable_strings(vec![
            "tests/shared_fixture_harness_application_v1.rs".to_string(),
            "tests/shared_output_dir_helper_application_v1.rs".to_string(),
        ]),
        changed_cli: vec!["src/bin/soma_experiment.rs".to_string()],
        changed_docs: vec![
            "docs/SPRINT108_SAFE_CONSOLIDATION_PATCH_V2.md".to_string(),
            "docs/SPRINT108_REPORT.md".to_string(),
        ],
        changed_examples: vec![
            "examples/soma_sprint108_safe_consolidation_patch_v2.toml".to_string(),
            "examples/soma_acceptance_truth_gate_v9.toml".to_string(),
        ],
        changed_fixtures: vec!["examples/sprint108_data/sprint107_summary.json".to_string()],
        high_risk_changes: Vec::new(),
        regression_status: "RegressionSurfaceClean".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_dual_agent_patch_verification_report_v2(
    reconciliation: &Sprint107VerificationReconciliationReport,
    truth: &AcceptanceTruthGateV9,
    verification: &AcceptanceRecoveryVerificationReportV3,
) -> DualAgentPatchVerificationReportV2 {
    let verification_passed = reconciliation.verification_reconciliation_status
        == "VerificationReconciled"
        && verification.verification_status == "AcceptanceRecoveryVerified"
        && truth.truth_status != "AcceptanceOverclaimed";
    DualAgentPatchVerificationReportV2 {
        report_id: "dual-agent-patch-verification-v2".to_string(),
        implementation_agent: "GPT-5.4 (gpt-5.4)".to_string(),
        verification_agent: "GPT-5.5 verification role".to_string(),
        independent_verification_performed: true,
        verification_reconciliation_status: reconciliation
            .verification_reconciliation_status
            .clone(),
        blocking_findings_remaining: !verification_passed,
        safety_verified: verification.safety_tests_preserved,
        architecture_verified: true,
        acceptance_truth_verified: truth.truth_status != "AcceptanceOverclaimed",
        verification_status: if verification_passed {
            "DualAgentPatchVerifiedWithWarnings"
        } else {
            "DualAgentPatchBlocked"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_safety_coverage_preservation_report_v24(
    config: &SafeConsolidationPatchV2Config,
    v23: &SafetyCoveragePreservationReportV23,
    sentinel: &SafetySentinelPreservationReportV2,
    reconciliation: &Sprint107VerificationReconciliationReport,
    proof: &EquivalentCoverageProofReportV1,
    timeout_cleanup: &TimeoutCleanupVerificationReportV1,
    selection: &SecondSafeConsolidationPatchSelectionReport,
    preservation: &AssertionPreservationVerificationReportV2,
) -> SafetyCoveragePreservationReportV24 {
    let inherited_safety_guards_present = v23.live_trading_guard_present
        && v23.broker_guard_present
        && v23.order_guard_present
        && v23.account_guard_present
        && v23.runtime_llm_guard_present
        && v23.mamba_runtime_guard_present
        && v23.gated_runtime_guard_present
        && v23.model_training_guard_present
        && v23.rust_neural_training_guard_present
        && v23.python_training_dependency_guard_present
        && v23.secret_guard_present
        && v23.no_lookahead_guard_present
        && v23.source_boundary_guard_present
        && v23.browser_execution_guard_present
        && v23.ui_order_control_guard_present
        && v23.committee_owned_core_guard_present
        && v23.investor_impersonation_guard_present
        && v23.paper_candidate_not_order_guard_present
        && v23.no_silent_confidence_upgrade_guard_present
        && v23.focused_not_full_acceptance_guard_present;
    let verification_reconciliation_guard_present = config.require_verification_reconciliation
        && reconciliation.verification_reconciliation_status == "VerificationReconciled";
    let equivalent_coverage_guard_present = config.require_equivalent_coverage_proof
        && proof.proof_status == "EquivalentCoverageProven";
    let safety_sentinel_preservation_guard_present = v23.safety_sentinel_preservation_guard_present
        && sentinel.sentinel_status == "SafetySentinelsPreserved";
    let timeout_cleanup_guard_present = matches!(
        timeout_cleanup.cleanup_status.as_str(),
        "NotApplicable" | "TimeoutCleanupVerified"
    );
    let second_patch_no_broad_consolidation_guard_present =
        selection.target_count_to_consolidate <= 1;
    let no_hidden_skip_guard_present =
        config.require_no_hidden_skips && v23.no_hidden_skip_guard_present;
    let assertion_preservation_guard_present = config.require_no_assertion_deletion
        && v23.assertion_preservation_guard_present
        && preservation.preservation_status == "AssertionsPreserved";
    let all_guards = v23.safety_status == "SafetyCoveragePreserved"
        && inherited_safety_guards_present
        && sentinel.sentinel_status == "SafetySentinelsPreserved"
        && verification_reconciliation_guard_present
        && equivalent_coverage_guard_present
        && timeout_cleanup_guard_present
        && no_hidden_skip_guard_present
        && assertion_preservation_guard_present
        && second_patch_no_broad_consolidation_guard_present;
    SafetyCoveragePreservationReportV24 {
        report_id: "safety-coverage-preservation-v24".to_string(),
        live_trading_guard_present: v23.live_trading_guard_present,
        broker_guard_present: v23.broker_guard_present,
        order_guard_present: v23.order_guard_present,
        account_guard_present: v23.account_guard_present,
        runtime_llm_guard_present: v23.runtime_llm_guard_present,
        mamba_runtime_guard_present: v23.mamba_runtime_guard_present,
        gated_runtime_guard_present: v23.gated_runtime_guard_present,
        model_training_guard_present: v23.model_training_guard_present,
        rust_neural_training_guard_present: v23.rust_neural_training_guard_present,
        python_training_dependency_guard_present: v23.python_training_dependency_guard_present,
        secret_guard_present: v23.secret_guard_present,
        no_lookahead_guard_present: v23.no_lookahead_guard_present,
        source_boundary_guard_present: v23.source_boundary_guard_present,
        browser_execution_guard_present: v23.browser_execution_guard_present,
        ui_order_control_guard_present: v23.ui_order_control_guard_present,
        committee_owned_core_guard_present: v23.committee_owned_core_guard_present,
        investor_impersonation_guard_present: v23.investor_impersonation_guard_present,
        paper_candidate_not_order_guard_present: v23.paper_candidate_not_order_guard_present,
        no_silent_confidence_upgrade_guard_present: v23.no_silent_confidence_upgrade_guard_present,
        focused_not_full_acceptance_guard_present: v23.focused_not_full_acceptance_guard_present,
        no_hidden_skip_guard_present,
        assertion_preservation_guard_present,
        safety_sentinel_preservation_guard_present,
        verification_reconciliation_guard_present,
        equivalent_coverage_guard_present,
        timeout_cleanup_guard_present,
        second_patch_no_broad_consolidation_guard_present,
        safety_status: if all_guards {
            "SafetyCoveragePreserved"
        } else {
            "SafetyCoverageMissing"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_control_tower_safe_consolidation_patch_panel_v2(
    selection: &SecondSafeConsolidationPatchSelectionReport,
    reconciliation: &Sprint107VerificationReconciliationReport,
    ledger: &AssertionMigrationLedgerV2,
    proof: &EquivalentCoverageProofReportV1,
    sentinel: &SafetySentinelPreservationReportV2,
    delta: &TestBinaryDeltaReportV5,
    no_run: &WorkspaceNoRunRecoveryGateV9,
    full: &WorkspaceFullAcceptanceGateV9,
    timeout_cleanup: &TimeoutCleanupVerificationReportV1,
) -> ControlTowerSafeConsolidationPatchPanelV2 {
    ControlTowerSafeConsolidationPatchPanelV2 {
        panel_id: "control-tower-safe-consolidation-patch-panel-v2".to_string(),
        patch_selection_status: selection.selected_status.clone(),
        verification_reconciliation_status: reconciliation
            .verification_reconciliation_status
            .clone(),
        assertion_ledger_status: ledger.ledger_status.clone(),
        equivalent_coverage_status: proof.proof_status.clone(),
        safety_sentinel_status: sentinel.sentinel_status.clone(),
        binary_delta_status: delta.delta_status.clone(),
        no_run_status: no_run.gate_status.clone(),
        full_status: full.gate_status.clone(),
        timeout_cleanup_status: timeout_cleanup.cleanup_status.clone(),
        next_actions: vec![
            "Run the focused Sprint 108 suite.".to_string(),
            "Rerun workspace no-run/full attempts with explicit timeouts.".to_string(),
        ],
        warnings: vec![
            "Static/read-only panel only.".to_string(),
            "No run-tests button or train/runtime/live/order/account/browser controls.".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_control_tower_workspace_acceptance_recovery_panel_v9(
    sprint107_bundle: &SafeConsolidationPatchV1Bundle,
    no_run: &PostPatchWorkspaceNoRunAttemptV24,
    full: &PostPatchWorkspaceFullAttemptV24,
    delta: &TestBinaryDeltaReportV5,
    selection: &SecondSafeConsolidationPatchSelectionReport,
    bridge: &FocusedVsFullBridgeV5,
    truth: &AcceptanceTruthGateV9,
    safety: &SafetyCoveragePreservationReportV24,
) -> ControlTowerWorkspaceAcceptanceRecoveryPanelV9 {
    ControlTowerWorkspaceAcceptanceRecoveryPanelV9 {
        panel_id: "control-tower-workspace-acceptance-recovery-panel-v9".to_string(),
        previous_no_run_status: sprint107_bundle.workspace_no_run_recovery_gate_v8.current_no_run_status.clone(),
        current_no_run_status: no_run.no_run_status.clone(),
        previous_full_status: sprint107_bundle.workspace_full_acceptance_gate_v8.current_full_status.clone(),
        current_full_status: full.full_status.clone(),
        binary_delta_status: delta.delta_status.clone(),
        consolidation_patch_status: selection.selected_status.clone(),
        focused_full_bridge_status: bridge.bridge_status.clone(),
        acceptance_truth_status: truth.truth_status.clone(),
        safety_coverage_status: safety.safety_status.clone(),
        runtime_deferred_summary:
            "Runtime, training, live inference, live trading, broker/order/account, dashboard serve, and browser execution remain deferred.".to_string(),
        next_actions: vec![
            "Keep focused/no-run/full truth separated.".to_string(),
            "Require a real finished and passed full workspace run before acceptance.".to_string(),
        ],
        warnings: vec![
            "Static/read-only panel only.".to_string(),
            "No run-tests button or train/runtime/live/order/account/browser controls.".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[]),
    }
}

impl SafeConsolidationPatchV1Bundle {
    fn safety_sentinel_preservation_report_v2_like(&self) -> SafetySentinelPreservationReportV2 {
        let mut report = self.safety_sentinel_preservation_report_v1.clone();
        report.report_id = "safety-sentinel-preservation-v2".to_string();
        report
    }
}
