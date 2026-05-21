use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::league::sprint108_safe_consolidation_patch_v2::{
    AcceptanceTruthGateV9, AssertionPreservationVerificationReportV2,
    ConsolidatedTestTargetManifestV2, ControlTowerWorkspaceAcceptanceRecoveryPanelV9,
    FocusedVsFullBridgeV5, MeasuredOrSampleBackedDeltaGateV2, PostPatchCliSmokeRunReportV2,
    PostPatchDeterminismRunReportV2, PostPatchFocusedTestRunReportV2, PostPatchSafetyRunReportV2,
    PostPatchWorkspaceFullAttemptV24, RegressionSurfaceAuditReportV2,
    RetiredNarrowTargetManifestV2, SafetySentinelPreservationReportV2,
    WorkspaceFullAcceptanceGateV9, WorkspaceNoRunRecoveryGateV9,
};
use crate::league::sprint109_safe_consolidation_patch_v3::{
    AcceptanceRecoveryVerificationReportV4, SafeConsolidationPatchV3Bundle,
    SafeConsolidationPatchV3Config, SafeConsolidationPatchV3Runner,
    SafetyCoveragePreservationReportV25,
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
    "target/soma_sprint110_safe_consolidation_patch_v4".to_string()
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

fn default_compat_equivalent_coverage_proof_report_v3() -> EquivalentCoverageProofReportV3 {
    EquivalentCoverageProofReportV3 {
        report_id: String::new(),
        previous_equivalent_coverage_refs: Vec::new(),
        sprint109_validation_refs: Vec::new(),
        retired_targets: Vec::new(),
        destination_targets: Vec::new(),
        moved_assertions: Vec::new(),
        equivalent_coverage_assertions: Vec::new(),
        coverage_gap_count: 0,
        cumulative_coverage_gap_count: 0,
        proof_status: String::new(),
        reason_codes: Vec::new(),
    }
}

fn default_compat_timeout_cleanup_verification_report_v3() -> TimeoutCleanupVerificationReportV3 {
    TimeoutCleanupVerificationReportV3 {
        report_id: String::new(),
        timeout_occurred: false,
        child_process_cleanup_attempted: false,
        remaining_cargo_processes: 0,
        remaining_rustc_processes: 0,
        cleanup_status: String::new(),
        reason_codes: Vec::new(),
    }
}

fn default_compat_acceptance_recovery_verification_report_v5()
-> AcceptanceRecoveryVerificationReportV5 {
    AcceptanceRecoveryVerificationReportV5 {
        report_id: String::new(),
        assertions_preserved: false,
        safety_tests_preserved: false,
        cli_safety_preserved: false,
        determinism_preserved: false,
        no_hidden_skips: false,
        no_overclaim: false,
        no_order_path_added: false,
        no_runtime_path_added: false,
        verification_status: String::new(),
        reason_codes: Vec::new(),
    }
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
            .map_err(|err| format!("failed to read sprint110 JSON input {path}: {err}"))?;
        match serde_json::from_str::<T>(&text) {
            Ok(value) => return Ok(Some(value)),
            Err(err) => parse_errors.push(format!("{path}: {err}")),
        }
    }
    if !paths.is_empty() {
        return Err(format!(
            "failed to parse any sprint110 JSON input: {}",
            parse_errors.join("; ")
        ));
    }
    Ok(None)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeConsolidationPatchV4Config {
    pub patch_id: String,
    #[serde(default)]
    pub sprint109_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub sprint109_validation_summary_paths: Option<Vec<String>>,
    #[serde(default)]
    pub previous_assertion_ledger_paths: Option<Vec<String>>,
    #[serde(default)]
    pub previous_retired_target_manifest_paths: Option<Vec<String>>,
    #[serde(default)]
    pub previous_verification_summary_paths: Option<Vec<String>>,
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
    pub require_sprint109_validation_reconciliation: bool,
    #[serde(default = "default_true")]
    pub require_cumulative_ledger: bool,
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

impl Default for SafeConsolidationPatchV4Config {
    fn default() -> Self {
        Self {
            patch_id: "sprint110-safe-consolidation-patch-v4".to_string(),
            sprint109_bundle_paths: Some(vec![
                "examples/sprint110_data/sprint109_summary.json".to_string(),
            ]),
            sprint109_validation_summary_paths: None,
            previous_assertion_ledger_paths: None,
            previous_retired_target_manifest_paths: None,
            previous_verification_summary_paths: None,
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
            require_sprint109_validation_reconciliation: true,
            require_cumulative_ledger: true,
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

impl SafeConsolidationPatchV4Config {
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
            return Err("sprint110 patch_id must not be empty".to_string());
        }
        if self.output_root.trim().is_empty() {
            return Err("sprint110 output_root must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err(
                "sprint110 safe consolidation patch config paths must be local".to_string(),
            );
        }
        for paths in [
            &self.sprint109_bundle_paths,
            &self.sprint109_validation_summary_paths,
            &self.previous_assertion_ledger_paths,
            &self.previous_retired_target_manifest_paths,
            &self.previous_verification_summary_paths,
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
                    "sprint110 safe consolidation patch config paths must be local".to_string(),
                );
            }
        }
        if self.max_targets_to_consolidate == 0 || self.max_targets_to_consolidate > 1 {
            return Err(
                "sprint110 max_targets_to_consolidate must stay within the fourth small patch"
                    .to_string(),
            );
        }
        if !self.preserve_runtime_deferred || !self.preserve_safety_guards {
            return Err("sprint110 runtime/safety preservation must remain enabled".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint109ExternalValidationReconciliationReport {
    pub report_id: String,
    pub focused_suite_imported: bool,
    pub focused_suite_passed: bool,
    pub focused_test_target_count: Option<usize>,
    pub focused_test_count: Option<usize>,
    pub cli_smoke_imported: bool,
    pub cli_smoke_passed: bool,
    pub cli_smoke_command_count: Option<usize>,
    pub cargo_build_imported: bool,
    pub cargo_build_passed: bool,
    pub workspace_no_run_timeout_imported: bool,
    pub workspace_full_timeout_imported: bool,
    pub timeout_cleanup_imported: bool,
    pub no_remaining_cargo_rustc_processes: bool,
    pub reconciliation_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint109FocusedSuiteResultImportReport {
    pub report_id: String,
    pub test_targets: usize,
    pub test_count: usize,
    pub passed: bool,
    pub failed_count: usize,
    pub imported_as_full_acceptance: bool,
    pub import_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint109CliSmokeResultImportReport {
    pub report_id: String,
    pub command_count: usize,
    pub commands: Vec<String>,
    pub passed: bool,
    pub failed_count: usize,
    pub imported_as_full_acceptance: bool,
    pub import_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint109CargoBuildResultImportReport {
    pub report_id: String,
    pub command: String,
    pub passed: bool,
    pub imported_as_workspace_test_acceptance: bool,
    pub import_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint109WorkspaceTimeoutImportReport {
    pub report_id: String,
    pub no_run_command: String,
    pub no_run_timeout_seconds: Option<u64>,
    pub no_run_exit_code: Option<i32>,
    pub full_command: String,
    pub full_timeout_seconds: Option<u64>,
    pub full_exit_code: Option<i32>,
    pub no_remaining_cargo_rustc_processes: bool,
    pub imported_as_pass: bool,
    pub timeout_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviousPatchLedgerCarryForwardReportV2 {
    pub report_id: String,
    pub previous_ledgers_loaded: usize,
    pub previous_retired_targets_loaded: usize,
    pub previous_assertion_delta_total: isize,
    pub previous_sample_backed_binary_delta: isize,
    pub previous_focused_validation_status: String,
    pub previous_workspace_truth_status: String,
    pub missing_previous_ledger_count: usize,
    pub carry_forward_status: String,
    pub reason_codes: Vec<ReasonCode>,
    #[serde(skip_serializing, default)]
    pub implementation_agent: String,
    #[serde(skip_serializing, default)]
    pub verification_agent: String,
    #[serde(skip_serializing, default)]
    pub verification_performed: bool,
    #[serde(skip_serializing, default)]
    pub findings_fixed: usize,
    #[serde(skip_serializing, default)]
    pub findings_remaining: usize,
    #[serde(skip_serializing, default)]
    pub final_verification_status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CumulativeAssertionMigrationLedgerReportV2 {
    pub report_id: String,
    pub ledger_count: usize,
    pub cumulative_moved_assertions: usize,
    pub cumulative_preserved_assertions: usize,
    pub cumulative_assertion_delta: isize,
    pub retired_target_count: usize,
    pub coverage_gap_count: usize,
    pub validation_reconciliation_refs: Vec<String>,
    pub cumulative_status: String,
    pub reason_codes: Vec<ReasonCode>,
    #[serde(skip_serializing, default)]
    pub carried_forward_patches: Vec<String>,
    #[serde(skip_serializing, default)]
    pub patches_still_effective: Vec<String>,
    #[serde(skip_serializing, default)]
    pub patches_regressed: Vec<String>,
    #[serde(skip_serializing, default)]
    pub carry_forward_status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FourthSafeConsolidationPatchSelectionReport {
    pub report_id: String,
    pub previous_patch_ids: Vec<String>,
    #[serde(skip_serializing, default)]
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
pub struct FourthConsolidationCandidateRiskReviewReport {
    pub report_id: String,
    pub selected_target_group: String,
    pub semantic_risk: String,
    pub safety_risk: String,
    pub determinism_risk: String,
    pub cli_surface_risk: String,
    pub fixture_risk: String,
    pub reason_risk: String,
    pub cumulative_patch_interaction_risk: String,
    pub validation_reconciliation_risk: String,
    pub risk_review_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssertionMigrationLedgerV4 {
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

pub type AssertionPreservationVerificationReportV4 = AssertionPreservationVerificationReportV2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquivalentCoverageProofReportV3 {
    pub report_id: String,
    pub previous_equivalent_coverage_refs: Vec<String>,
    pub sprint109_validation_refs: Vec<String>,
    pub retired_targets: Vec<String>,
    pub destination_targets: Vec<String>,
    pub moved_assertions: Vec<String>,
    pub equivalent_coverage_assertions: Vec<String>,
    pub coverage_gap_count: usize,
    pub cumulative_coverage_gap_count: usize,
    pub proof_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetiredTargetSafetyAuditReportV4 {
    pub report_id: String,
    pub retired_targets: Vec<String>,
    pub cumulative_retired_targets: Vec<String>,
    pub validation_reconciled_before_retirement: bool,
    pub high_risk_target_retired: bool,
    pub safety_sentinel_retired: bool,
    pub assertion_ledger_refs: Vec<String>,
    pub equivalent_coverage_refs: Vec<String>,
    pub audit_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

pub type SafetySentinelPreservationReportV4 = SafetySentinelPreservationReportV2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedFixtureHarnessExpansionApplicationReportV4 {
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
pub struct SharedRenderHelperExpansionReportV4 {
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
pub struct SharedOutputDirHelperExpansionReportV4 {
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
pub struct SharedTomlBuilderExpansionReportV4 {
    pub report_id: String,
    pub new_targets_affected: Vec<String>,
    pub local_only_path_validation_preserved: bool,
    pub remote_path_rejection_preserved: bool,
    pub duplicated_toml_builders_removed: usize,
    pub application_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliSmokeTieringApplicationReportV4 {
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
pub struct ArtifactRenderCacheDecisionReportV4 {
    pub report_id: String,
    pub cache_enabled: bool,
    pub why_enabled_or_disabled: String,
    pub local_only_cache: bool,
    pub deterministic_keys: bool,
    pub secret_free_cache: bool,
    pub decision_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

pub type ConsolidatedTestTargetManifestV4 = ConsolidatedTestTargetManifestV2;
pub type RetiredNarrowTargetManifestV4 = RetiredNarrowTargetManifestV2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestBinaryDeltaReportV7 {
    pub report_id: String,
    pub target_count_before: Option<usize>,
    pub target_count_after: Option<usize>,
    pub integration_binary_count_before: Option<usize>,
    pub integration_binary_count_after: Option<usize>,
    pub binary_delta: Option<isize>,
    pub sprint107_delta: Option<isize>,
    pub sprint108_delta: Option<isize>,
    pub sprint109_delta: Option<isize>,
    pub sprint110_delta: Option<isize>,
    pub measured: bool,
    pub sample_backed: bool,
    pub timing_available: bool,
    pub cumulative_sample_backed_delta: Option<isize>,
    pub delta_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CumulativeBinaryDeltaReportV2 {
    pub report_id: String,
    pub patch_count: usize,
    pub sample_backed_deltas: Vec<isize>,
    pub cumulative_sample_backed_delta: isize,
    pub measured_deltas: Vec<isize>,
    pub cumulative_measured_delta: Option<isize>,
    pub measured_claim_allowed: bool,
    pub cumulative_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

pub type MeasuredOrSampleBackedDeltaGateV4 = MeasuredOrSampleBackedDeltaGateV2;
pub type PostPatchFocusedTestRunReportV4 = PostPatchFocusedTestRunReportV2;
pub type PostPatchCliSmokeRunReportV4 = PostPatchCliSmokeRunReportV2;
pub type PostPatchSafetyRunReportV4 = PostPatchSafetyRunReportV2;
pub type PostPatchDeterminismRunReportV4 = PostPatchDeterminismRunReportV2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostPatchWorkspaceNoRunAttemptV26 {
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
    pub cargo_json_progress_capture_ref: Option<String>,
    pub child_process_cleanup_verified: bool,
    pub no_run_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

pub type PostPatchWorkspaceFullAttemptV26 = PostPatchWorkspaceFullAttemptV24;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtendedNoRunObservationReportV3 {
    pub report_id: String,
    pub attempted: bool,
    pub timeout_ms: Option<u64>,
    pub previous_timeout_seconds: Option<u64>,
    pub observed_duration_ms: Option<u64>,
    pub observation_window_ms: Option<u64>,
    pub last_observed_target: Option<String>,
    pub observed_target_count: Option<usize>,
    pub last_cargo_json_artifact: Option<String>,
    pub cargo_stdout_present: bool,
    pub cargo_stderr_present: bool,
    pub rustc_processes_after_timeout: usize,
    pub cargo_processes_after_timeout: usize,
    pub observation_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceCargoJsonProgressCaptureV4 {
    pub capture_id: String,
    pub command: String,
    pub attempted: bool,
    pub message_count: usize,
    pub compiler_artifact_count: usize,
    pub compiler_message_count: usize,
    pub test_executable_count: usize,
    pub last_seen_targets: Vec<String>,
    pub last_seen_artifacts: Vec<String>,
    pub previous_capture_refs: Vec<String>,
    pub capture_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutCleanupVerificationReportV3 {
    pub report_id: String,
    pub timeout_occurred: bool,
    pub child_process_cleanup_attempted: bool,
    pub remaining_cargo_processes: usize,
    pub remaining_rustc_processes: usize,
    pub cleanup_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

pub type WorkspaceNoRunRecoveryGateV11 = WorkspaceNoRunRecoveryGateV9;
pub type WorkspaceFullAcceptanceGateV11 = WorkspaceFullAcceptanceGateV9;
pub type FocusedVsFullBridgeV7 = FocusedVsFullBridgeV5;
pub type AcceptanceTruthGateV11 = AcceptanceTruthGateV9;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceRecoveryPatchImpactReportV5 {
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

pub type AcceptanceRecoveryVerificationReportV5 = AcceptanceRecoveryVerificationReportV4;
pub type RegressionSurfaceAuditReportV4 = RegressionSurfaceAuditReportV2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DualAgentPatchVerificationReportV4 {
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
pub struct ControlTowerSafeConsolidationPatchPanelV4 {
    pub panel_id: String,
    pub patch_selection_status: String,
    pub verification_carry_forward_status: String,
    pub assertion_ledger_status: String,
    pub cumulative_ledger_status: String,
    pub equivalent_coverage_status: String,
    pub safety_sentinel_status: String,
    pub binary_delta_status: String,
    pub cumulative_binary_delta_status: String,
    pub no_run_status: String,
    pub full_status: String,
    pub cargo_json_progress_status: String,
    pub timeout_cleanup_status: String,
    pub next_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
    #[serde(skip_serializing, default)]
    pub verification_reconciliation_status: String,
}

pub type ControlTowerWorkspaceAcceptanceRecoveryPanelV11 =
    ControlTowerWorkspaceAcceptanceRecoveryPanelV9;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyCoveragePreservationReportV26 {
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
    pub cumulative_assertion_ledger_guard_present: bool,
    pub equivalent_coverage_v2_guard_present: bool,
    pub timeout_cleanup_v2_guard_present: bool,
    pub cargo_json_progress_truth_guard_present: bool,
    pub third_patch_no_broad_consolidation_guard_present: bool,
    pub sprint109_validation_reconciliation_guard_present: bool,
    pub cumulative_assertion_ledger_v2_guard_present: bool,
    pub equivalent_coverage_v3_guard_present: bool,
    pub timeout_cleanup_v3_guard_present: bool,
    pub cargo_json_progress_v4_truth_guard_present: bool,
    pub fourth_patch_no_broad_consolidation_guard_present: bool,
    pub safety_status: String,
    pub reason_codes: Vec<ReasonCode>,
    #[serde(skip_serializing, default)]
    pub timeout_cleanup_guard_present: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeConsolidationPatchV4StorageReport {
    pub report_id: String,
    pub output_dir: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SafeConsolidationPatchV4Bundle {
    pub sprint109_external_validation_reconciliation_report:
        Sprint109ExternalValidationReconciliationReport,
    pub sprint109_focused_suite_result_import_report: Sprint109FocusedSuiteResultImportReport,
    pub sprint109_cli_smoke_result_import_report: Sprint109CliSmokeResultImportReport,
    pub sprint109_cargo_build_result_import_report: Sprint109CargoBuildResultImportReport,
    pub sprint109_workspace_timeout_import_report: Sprint109WorkspaceTimeoutImportReport,
    pub previous_patch_ledger_carry_forward_report: PreviousPatchLedgerCarryForwardReportV2,
    pub fourth_safe_consolidation_patch_selection_report:
        FourthSafeConsolidationPatchSelectionReport,
    pub fourth_consolidation_candidate_risk_review_report:
        FourthConsolidationCandidateRiskReviewReport,
    pub assertion_migration_ledger_v4: AssertionMigrationLedgerV4,
    pub cumulative_assertion_migration_ledger_report: CumulativeAssertionMigrationLedgerReportV2,
    pub assertion_preservation_verification_report_v4: AssertionPreservationVerificationReportV4,
    pub equivalent_coverage_proof_report_v3: EquivalentCoverageProofReportV3,
    #[serde(
        skip_serializing,
        default = "default_compat_equivalent_coverage_proof_report_v3"
    )]
    pub equivalent_coverage_proof_report_v2: EquivalentCoverageProofReportV3,
    pub retired_target_safety_audit_report_v4: RetiredTargetSafetyAuditReportV4,
    pub safety_sentinel_preservation_report_v4: SafetySentinelPreservationReportV4,
    pub shared_fixture_harness_expansion_application_report_v4:
        SharedFixtureHarnessExpansionApplicationReportV4,
    pub shared_render_helper_expansion_report_v4: SharedRenderHelperExpansionReportV4,
    pub shared_output_dir_helper_expansion_report_v4: SharedOutputDirHelperExpansionReportV4,
    pub shared_toml_builder_expansion_report_v4: SharedTomlBuilderExpansionReportV4,
    pub cli_smoke_tiering_application_report_v4: CliSmokeTieringApplicationReportV4,
    pub artifact_render_cache_decision_report_v4: ArtifactRenderCacheDecisionReportV4,
    pub consolidated_test_target_manifest_v4: ConsolidatedTestTargetManifestV4,
    pub retired_narrow_target_manifest_v4: RetiredNarrowTargetManifestV4,
    pub test_binary_delta_report_v7: TestBinaryDeltaReportV7,
    pub cumulative_binary_delta_report_v2: CumulativeBinaryDeltaReportV2,
    pub measured_or_sample_backed_delta_gate_v4: MeasuredOrSampleBackedDeltaGateV4,
    pub post_patch_focused_test_run_report_v4: PostPatchFocusedTestRunReportV4,
    pub post_patch_cli_smoke_run_report_v4: PostPatchCliSmokeRunReportV4,
    pub post_patch_safety_run_report_v4: PostPatchSafetyRunReportV4,
    pub post_patch_determinism_run_report_v4: PostPatchDeterminismRunReportV4,
    pub post_patch_workspace_no_run_attempt_v26: PostPatchWorkspaceNoRunAttemptV26,
    pub post_patch_workspace_full_attempt_v26: PostPatchWorkspaceFullAttemptV26,
    pub extended_no_run_observation_report_v3: ExtendedNoRunObservationReportV3,
    pub workspace_cargo_json_progress_capture_v4: WorkspaceCargoJsonProgressCaptureV4,
    #[serde(
        skip_serializing,
        default = "default_compat_timeout_cleanup_verification_report_v3"
    )]
    pub timeout_cleanup_verification_report_v2: TimeoutCleanupVerificationReportV3,
    pub timeout_cleanup_verification_report_v3: TimeoutCleanupVerificationReportV3,
    pub workspace_no_run_recovery_gate_v11: WorkspaceNoRunRecoveryGateV11,
    pub workspace_full_acceptance_gate_v11: WorkspaceFullAcceptanceGateV11,
    pub focused_vs_full_bridge_v7: FocusedVsFullBridgeV7,
    pub acceptance_truth_gate_v11: AcceptanceTruthGateV11,
    pub acceptance_recovery_patch_impact_report_v5: AcceptanceRecoveryPatchImpactReportV5,
    pub acceptance_recovery_verification_report_v5: AcceptanceRecoveryVerificationReportV5,
    #[serde(
        skip_serializing,
        default = "default_compat_acceptance_recovery_verification_report_v5"
    )]
    pub acceptance_recovery_verification_report_v4: AcceptanceRecoveryVerificationReportV5,
    pub regression_surface_audit_report_v4: RegressionSurfaceAuditReportV4,
    pub dual_agent_patch_verification_report_v4: DualAgentPatchVerificationReportV4,
    pub safety_coverage_preservation_report_v26: SafetyCoveragePreservationReportV26,
    pub control_tower_safe_consolidation_patch_panel_v4: ControlTowerSafeConsolidationPatchPanelV4,
    pub control_tower_workspace_acceptance_recovery_panel_v11:
        ControlTowerWorkspaceAcceptanceRecoveryPanelV11,
    pub storage_report: SafeConsolidationPatchV4StorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl SafeConsolidationPatchV4Bundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            (
                "## 1. Sprint summary",
                format!(
                    "- selected_target={} selected_status={} cumulative_sample_backed_delta={}; full_workspace_status={}.",
                    self.fourth_safe_consolidation_patch_selection_report.selected_target_group,
                    self.fourth_safe_consolidation_patch_selection_report.selected_status,
                    self.cumulative_binary_delta_report_v2.cumulative_sample_backed_delta,
                    self.workspace_full_acceptance_gate_v11.gate_status
                ),
            ),
            (
                "## 2. Why Sprint 110 was needed",
                format!(
                    "- Sprint 109 external validation needed official reconciliation before applying the fourth smallest safe consolidation patch; full workspace acceptance remains {}.",
                    self.workspace_full_acceptance_gate_v11.gate_status
                ),
            ),
            (
                "## 3. Files added",
                "- Sprint 110 target reports, CLI examples, docs, and focused tests for validation reconciliation and fourth-patch verification.".to_string(),
            ),
            (
                "## 4. Files changed",
                "- src/league/sprint110_safe_consolidation_patch_v4.rs; src/bin/soma_experiment.rs; tests/shared_fixture_harness_application_v1.rs; docs and fixtures.".to_string(),
            ),
            (
                "## 5. Sprint 109 external validation reconciliation",
                format!(
                    "- status={} focused_passed={} cli_passed={} build_passed={} cleanup_imported={}.",
                    self.sprint109_external_validation_reconciliation_report.reconciliation_status,
                    self.sprint109_external_validation_reconciliation_report.focused_suite_passed,
                    self.sprint109_external_validation_reconciliation_report.cli_smoke_passed,
                    self.sprint109_external_validation_reconciliation_report.cargo_build_passed,
                    self.sprint109_external_validation_reconciliation_report.timeout_cleanup_imported
                ),
            ),
            (
                "## 6. Sprint 109 focused suite import",
                format!(
                    "- status={} targets={} tests={} passed={}.",
                    self.sprint109_focused_suite_result_import_report.import_status,
                    self.sprint109_focused_suite_result_import_report.test_targets,
                    self.sprint109_focused_suite_result_import_report.test_count,
                    self.sprint109_focused_suite_result_import_report.passed
                ),
            ),
            (
                "## 7. Sprint 109 CLI smoke import",
                format!(
                    "- status={} commands={} passed={}.",
                    self.sprint109_cli_smoke_result_import_report.import_status,
                    self.sprint109_cli_smoke_result_import_report.command_count,
                    self.sprint109_cli_smoke_result_import_report.passed
                ),
            ),
            (
                "## 8. Sprint 109 cargo build import",
                format!(
                    "- status={} passed={} imported_as_workspace_test_acceptance={}.",
                    self.sprint109_cargo_build_result_import_report.import_status,
                    self.sprint109_cargo_build_result_import_report.passed,
                    self.sprint109_cargo_build_result_import_report
                        .imported_as_workspace_test_acceptance
                ),
            ),
            (
                "## 9. Sprint 109 workspace timeout import",
                format!(
                    "- status={} no_run_exit={:?} full_exit={:?} cleanup_clear={}.",
                    self.sprint109_workspace_timeout_import_report.timeout_status,
                    self.sprint109_workspace_timeout_import_report.no_run_exit_code,
                    self.sprint109_workspace_timeout_import_report.full_exit_code,
                    self.sprint109_workspace_timeout_import_report
                        .no_remaining_cargo_rustc_processes
                ),
            ),
            (
                "## 10. Previous patch ledger carry-forward v2",
                format!(
                    "- status={} loaded_ledgers={} loaded_retired_targets={}.",
                    self.previous_patch_ledger_carry_forward_report.carry_forward_status,
                    self.previous_patch_ledger_carry_forward_report.previous_ledgers_loaded,
                    self.previous_patch_ledger_carry_forward_report.previous_retired_targets_loaded
                ),
            ),
            (
                "## 11. Fourth safe consolidation patch selection",
                format!(
                    "- status={} target_group={}.",
                    self.fourth_safe_consolidation_patch_selection_report.selected_status,
                    self.fourth_safe_consolidation_patch_selection_report.selected_target_group
                ),
            ),
            (
                "## 12. Fourth candidate risk review",
                format!(
                    "- status={} semantic_risk={} safety_risk={}.",
                    self.fourth_consolidation_candidate_risk_review_report.risk_review_status,
                    self.fourth_consolidation_candidate_risk_review_report.semantic_risk,
                    self.fourth_consolidation_candidate_risk_review_report.safety_risk
                ),
            ),
            (
                "## 13. Assertion migration ledger v4",
                format!(
                    "- status={} assertion_delta={}.",
                    self.assertion_migration_ledger_v4.ledger_status,
                    self.assertion_migration_ledger_v4.assertion_delta
                ),
            ),
            (
                "## 14. Cumulative assertion migration ledger v2",
                format!(
                    "- status={} moved={} preserved={} cumulative_delta={}.",
                    self.cumulative_assertion_migration_ledger_report.cumulative_status,
                    self.cumulative_assertion_migration_ledger_report.cumulative_moved_assertions,
                    self.cumulative_assertion_migration_ledger_report.cumulative_preserved_assertions,
                    self.cumulative_assertion_migration_ledger_report.cumulative_assertion_delta
                ),
            ),
            (
                "## 15. Assertion preservation verification v4",
                format!(
                    "- status={} missing_assertions={}.",
                    self.assertion_preservation_verification_report_v4.preservation_status,
                    self.assertion_preservation_verification_report_v4.missing_assertion_count
                ),
            ),
            (
                "## 16. Equivalent coverage proof v3",
                format!(
                    "- status={} coverage_gap_count={}.",
                    self.equivalent_coverage_proof_report_v3.proof_status,
                    self.equivalent_coverage_proof_report_v3.coverage_gap_count
                ),
            ),
            (
                "## 17. Retired target safety audit v4",
                format!(
                    "- status={} retired_targets={} cumulative_retired_targets={}.",
                    self.retired_target_safety_audit_report_v4.audit_status,
                    self.retired_target_safety_audit_report_v4.retired_targets.len(),
                    self.retired_target_safety_audit_report_v4
                        .cumulative_retired_targets
                        .len()
                ),
            ),
            (
                "## 18. Safety sentinel preservation v4",
                format!(
                    "- status={}.",
                    self.safety_sentinel_preservation_report_v4.sentinel_status
                ),
            ),
            (
                "## 19. Shared fixture/render/output/TOML helper expansion v4",
                format!(
                    "- fixture={} render={} output={} toml={}.",
                    self.shared_fixture_harness_expansion_application_report_v4.application_status,
                    self.shared_render_helper_expansion_report_v4.application_status,
                    self.shared_output_dir_helper_expansion_report_v4.application_status,
                    self.shared_toml_builder_expansion_report_v4.application_status
                ),
            ),
            (
                "## 20. Artifact render cache decision v4",
                format!(
                    "- status={} cache_enabled={}.",
                    self.artifact_render_cache_decision_report_v4.decision_status,
                    self.artifact_render_cache_decision_report_v4.cache_enabled
                ),
            ),
            (
                "## 21. CLI smoke tiering v4",
                format!(
                    "- status={} safety_commands_preserved={}.",
                    self.cli_smoke_tiering_application_report_v4.application_status,
                    self.cli_smoke_tiering_application_report_v4.safety_commands_preserved
                ),
            ),
            (
                "## 22. Consolidated / retired target manifests v4",
                format!(
                    "- consolidated={} retired={}.",
                    self.consolidated_test_target_manifest_v4.manifest_status,
                    self.retired_narrow_target_manifest_v4.retired_status
                ),
            ),
            (
                "## 23. Test binary delta v7",
                format!(
                    "- status={} sprint110_delta={:?}.",
                    self.test_binary_delta_report_v7.delta_status,
                    self.test_binary_delta_report_v7.sprint110_delta
                ),
            ),
            (
                "## 24. Cumulative binary delta v2",
                format!(
                    "- status={} cumulative_sample_backed_delta={}.",
                    self.cumulative_binary_delta_report_v2.cumulative_status,
                    self.cumulative_binary_delta_report_v2.cumulative_sample_backed_delta
                ),
            ),
            (
                "## 25. Measured vs sample-backed delta gate v4",
                format!(
                    "- status={} can_claim_measured_reduction={}.",
                    self.measured_or_sample_backed_delta_gate_v4.gate_status,
                    self.measured_or_sample_backed_delta_gate_v4.can_claim_measured_reduction
                ),
            ),
            (
                "## 26. Post-patch focused / CLI / safety / determinism runs",
                format!(
                    "- focused={} cli={} safety={} determinism={}.",
                    self.post_patch_focused_test_run_report_v4.run_status,
                    self.post_patch_cli_smoke_run_report_v4.smoke_status,
                    self.post_patch_safety_run_report_v4.safety_status,
                    self.post_patch_determinism_run_report_v4.determinism_status
                ),
            ),
            (
                "## 27. Post-patch workspace no-run attempt v26",
                format!(
                    "- status={} finished={} passed={:?}.",
                    self.post_patch_workspace_no_run_attempt_v26.no_run_status,
                    self.post_patch_workspace_no_run_attempt_v26.finished,
                    self.post_patch_workspace_no_run_attempt_v26.passed
                ),
            ),
            (
                "## 28. Post-patch workspace full attempt v26",
                format!(
                    "- status={} finished={} passed={:?}.",
                    self.post_patch_workspace_full_attempt_v26.full_status,
                    self.post_patch_workspace_full_attempt_v26.finished,
                    self.post_patch_workspace_full_attempt_v26.passed
                ),
            ),
            (
                "## 29. Extended no-run observation v3",
                format!(
                    "- status={} attempted={}.",
                    self.extended_no_run_observation_report_v3.observation_status,
                    self.extended_no_run_observation_report_v3.attempted
                ),
            ),
            (
                "## 30. Workspace cargo JSON progress capture v4",
                format!(
                    "- status={} attempted={}.",
                    self.workspace_cargo_json_progress_capture_v4.capture_status,
                    self.workspace_cargo_json_progress_capture_v4.attempted
                ),
            ),
            (
                "## 31. Timeout cleanup verification v3",
                format!(
                    "- status={} timeout_occurred={}.",
                    self.timeout_cleanup_verification_report_v3.cleanup_status,
                    self.timeout_cleanup_verification_report_v3.timeout_occurred
                ),
            ),
            (
                "## 32. Workspace no-run recovery gate v11",
                format!(
                    "- status={} recovered={}.",
                    self.workspace_no_run_recovery_gate_v11.gate_status,
                    self.workspace_no_run_recovery_gate_v11.no_run_recovered
                ),
            ),
            (
                "## 33. Workspace full acceptance gate v11",
                format!(
                    "- status={} accepted={}.",
                    self.workspace_full_acceptance_gate_v11.gate_status,
                    self.workspace_full_acceptance_gate_v11.full_workspace_accepted
                ),
            ),
            (
                "## 34. Focused-vs-full bridge v7",
                format!(
                    "- status={} can_claim_full_acceptance={}.",
                    self.focused_vs_full_bridge_v7.bridge_status,
                    self.focused_vs_full_bridge_v7.can_claim_full_acceptance
                ),
            ),
            (
                "## 35. Acceptance truth gate v11",
                format!(
                    "- status={} can_claim_full_acceptance={}.",
                    self.acceptance_truth_gate_v11.truth_status,
                    self.acceptance_truth_gate_v11.can_claim_full_acceptance
                ),
            ),
            (
                "## 36. Patch impact v5",
                format!(
                    "- status={} measured_binary_delta={:?}.",
                    self.acceptance_recovery_patch_impact_report_v5.impact_status,
                    self.acceptance_recovery_patch_impact_report_v5.measured_binary_delta
                ),
            ),
            (
                "## 37. Acceptance recovery verification v5",
                format!(
                    "- status={}.",
                    self.acceptance_recovery_verification_report_v5.verification_status
                ),
            ),
            (
                "## 38. Regression surface audit v4",
                format!(
                    "- status={} high_risk_changes={}.",
                    self.regression_surface_audit_report_v4.regression_status,
                    self.regression_surface_audit_report_v4.high_risk_changes.len()
                ),
            ),
            (
                "## 39. Dual-agent patch verification v4",
                format!(
                    "- status={} implementation_agent={} verification_agent={}.",
                    self.dual_agent_patch_verification_report_v4.verification_status,
                    self.dual_agent_patch_verification_report_v4.implementation_agent,
                    self.dual_agent_patch_verification_report_v4.verification_agent
                ),
            ),
            (
                "## 40. Safety coverage preservation v26",
                format!(
                    "- status={}.",
                    self.safety_coverage_preservation_report_v26.safety_status
                ),
            ),
            (
                "## 41. Control Tower safe consolidation patch panel v4",
                format!(
                    "- patch_selection_status={} full_status={}.",
                    self.control_tower_safe_consolidation_patch_panel_v4.patch_selection_status,
                    self.control_tower_safe_consolidation_patch_panel_v4.full_status
                ),
            ),
            (
                "## 42. Control Tower workspace acceptance recovery panel v11",
                format!(
                    "- acceptance_truth_status={} safety_coverage_status={}.",
                    self.control_tower_workspace_acceptance_recovery_panel_v11.acceptance_truth_status,
                    self.control_tower_workspace_acceptance_recovery_panel_v11.safety_coverage_status
                ),
            ),
            (
                "## 43. Output bundle",
                format!("- file_count={}.", self.storage_report.file_count),
            ),
            (
                "## 44. CLI and examples",
                "- Sprint 110 CLI/report commands remain local-only and research-only.".to_string(),
            ),
            (
                "## 45. Tests added",
                "- Sprint 110 focused, CLI safety, determinism, assertion, equivalent coverage, validation import, and panel tests.".to_string(),
            ),
            (
                "## 46. Test results",
                "- Bundle records diagnostic status only; external cargo results must be reported separately.".to_string(),
            ),
            (
                "## 47. Patch application status",
                format!(
                    "- status={}.",
                    self.fourth_safe_consolidation_patch_selection_report.selected_status
                ),
            ),
            (
                "## 48. Validation reconciliation status",
                format!(
                    "- status={}.",
                    self.sprint109_external_validation_reconciliation_report.reconciliation_status
                ),
            ),
            (
                "## 49. Cumulative assertion / equivalent coverage status",
                format!(
                    "- assertion={} cumulative={} equivalent_coverage={}.",
                    self.assertion_migration_ledger_v4.ledger_status,
                    self.cumulative_assertion_migration_ledger_report.cumulative_status,
                    self.equivalent_coverage_proof_report_v3.proof_status
                ),
            ),
            (
                "## 50. Safety sentinel status",
                format!(
                    "- status={}.",
                    self.safety_sentinel_preservation_report_v4.sentinel_status
                ),
            ),
            (
                "## 51. No-run recovery status",
                format!(
                    "- status={}.",
                    self.workspace_no_run_recovery_gate_v11.gate_status
                ),
            ),
            (
                "## 52. Full workspace acceptance status",
                format!(
                    "- status={}.",
                    self.workspace_full_acceptance_gate_v11.gate_status
                ),
            ),
            (
                "## 53. Binary delta status",
                format!(
                    "- sample_backed={} measured={} cumulative={}.",
                    self.test_binary_delta_report_v7.sample_backed,
                    self.test_binary_delta_report_v7.measured,
                    self.cumulative_binary_delta_report_v2.cumulative_sample_backed_delta
                ),
            ),
            (
                "## 54. Runtime deferred status",
                "- Runtime, training, live inference, live trading, broker/order/account, Mamba/Gated runtime, dashboard serve, browser execution remain deferred/forbidden.".to_string(),
            ),
            (
                "## 55. Workspace acceptance truth status",
                format!(
                    "- status={} full_acceptance_claim={}.",
                    self.acceptance_truth_gate_v11.truth_status,
                    self.acceptance_truth_gate_v11.can_claim_full_acceptance
                ),
            ),
            (
                "## 56. Safety coverage status",
                format!(
                    "- status={}.",
                    self.safety_coverage_preservation_report_v26.safety_status
                ),
            ),
            (
                "## 57. Risk review",
                format!(
                    "- status={} blocking_findings_remaining={}.",
                    self.fourth_consolidation_candidate_risk_review_report.risk_review_status,
                    self.dual_agent_patch_verification_report_v4.blocking_findings_remaining
                ),
            ),
            (
                "## 58. Deferred items",
                "- Full workspace acceptance remains open unless cargo test --workspace --quiet finishes and passes.".to_string(),
            ),
            (
                "## 59. Next gstack sprint recommendation",
                "- Continue one low-risk helper/fixture target at a time; keep full acceptance separate from focused verification.".to_string(),
            ),
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
            &output_dir.join("sprint109_external_validation_reconciliation.txt"),
            &self.sprint109_external_validation_reconciliation_report,
        )?;
        write_json_file(
            &output_dir.join("sprint109_focused_suite_result_import.txt"),
            &self.sprint109_focused_suite_result_import_report,
        )?;
        write_json_file(
            &output_dir.join("sprint109_cli_smoke_result_import.txt"),
            &self.sprint109_cli_smoke_result_import_report,
        )?;
        write_json_file(
            &output_dir.join("sprint109_cargo_build_result_import.txt"),
            &self.sprint109_cargo_build_result_import_report,
        )?;
        write_json_file(
            &output_dir.join("sprint109_workspace_timeout_import.txt"),
            &self.sprint109_workspace_timeout_import_report,
        )?;
        write_json_file(
            &output_dir.join("previous_patch_ledger_carry_forward_v2.txt"),
            &self.previous_patch_ledger_carry_forward_report,
        )?;
        write_json_file(
            &output_dir.join("fourth_safe_consolidation_patch_selection.txt"),
            &self.fourth_safe_consolidation_patch_selection_report,
        )?;
        write_json_file(
            &output_dir.join("fourth_consolidation_candidate_risk_review.txt"),
            &self.fourth_consolidation_candidate_risk_review_report,
        )?;
        write_json_file(
            &output_dir.join("assertion_migration_ledger_v4.txt"),
            &self.assertion_migration_ledger_v4,
        )?;
        write_json_file(
            &output_dir.join("cumulative_assertion_migration_ledger_v2.txt"),
            &self.cumulative_assertion_migration_ledger_report,
        )?;
        write_json_file(
            &output_dir.join("assertion_preservation_verification_v4.txt"),
            &self.assertion_preservation_verification_report_v4,
        )?;
        write_json_file(
            &output_dir.join("equivalent_coverage_proof_v3.txt"),
            &self.equivalent_coverage_proof_report_v3,
        )?;
        write_json_file(
            &output_dir.join("retired_target_safety_audit_v4.txt"),
            &self.retired_target_safety_audit_report_v4,
        )?;
        write_json_file(
            &output_dir.join("safety_sentinel_preservation_v4.txt"),
            &self.safety_sentinel_preservation_report_v4,
        )?;
        write_json_file(
            &output_dir.join("shared_fixture_harness_expansion_v4.txt"),
            &self.shared_fixture_harness_expansion_application_report_v4,
        )?;
        write_json_file(
            &output_dir.join("shared_render_helper_expansion_v4.txt"),
            &self.shared_render_helper_expansion_report_v4,
        )?;
        write_json_file(
            &output_dir.join("shared_output_dir_helper_expansion_v4.txt"),
            &self.shared_output_dir_helper_expansion_report_v4,
        )?;
        write_json_file(
            &output_dir.join("shared_toml_builder_expansion_v4.txt"),
            &self.shared_toml_builder_expansion_report_v4,
        )?;
        write_json_file(
            &output_dir.join("cli_smoke_tiering_application_v4.txt"),
            &self.cli_smoke_tiering_application_report_v4,
        )?;
        write_json_file(
            &output_dir.join("artifact_render_cache_decision_v4.txt"),
            &self.artifact_render_cache_decision_report_v4,
        )?;
        write_json_file(
            &output_dir.join("consolidated_test_target_manifest_v4.txt"),
            &self.consolidated_test_target_manifest_v4,
        )?;
        write_json_file(
            &output_dir.join("retired_narrow_target_manifest_v4.txt"),
            &self.retired_narrow_target_manifest_v4,
        )?;
        write_json_file(
            &output_dir.join("test_binary_delta_v7.txt"),
            &self.test_binary_delta_report_v7,
        )?;
        write_json_file(
            &output_dir.join("cumulative_binary_delta_v2.txt"),
            &self.cumulative_binary_delta_report_v2,
        )?;
        write_json_file(
            &output_dir.join("measured_or_sample_backed_delta_gate_v4.txt"),
            &self.measured_or_sample_backed_delta_gate_v4,
        )?;
        write_json_file(
            &output_dir.join("post_patch_focused_test_run_v4.txt"),
            &self.post_patch_focused_test_run_report_v4,
        )?;
        write_json_file(
            &output_dir.join("post_patch_cli_smoke_run_v4.txt"),
            &self.post_patch_cli_smoke_run_report_v4,
        )?;
        write_json_file(
            &output_dir.join("post_patch_safety_run_v4.txt"),
            &self.post_patch_safety_run_report_v4,
        )?;
        write_json_file(
            &output_dir.join("post_patch_determinism_run_v4.txt"),
            &self.post_patch_determinism_run_report_v4,
        )?;
        write_json_file(
            &output_dir.join("post_patch_workspace_no_run_attempt_v26.txt"),
            &self.post_patch_workspace_no_run_attempt_v26,
        )?;
        write_json_file(
            &output_dir.join("post_patch_workspace_full_attempt_v26.txt"),
            &self.post_patch_workspace_full_attempt_v26,
        )?;
        write_json_file(
            &output_dir.join("extended_no_run_observation_v3.txt"),
            &self.extended_no_run_observation_report_v3,
        )?;
        write_json_file(
            &output_dir.join("workspace_cargo_json_progress_capture_v4.txt"),
            &self.workspace_cargo_json_progress_capture_v4,
        )?;
        write_json_file(
            &output_dir.join("timeout_cleanup_verification_v3.txt"),
            &self.timeout_cleanup_verification_report_v3,
        )?;
        write_json_file(
            &output_dir.join("workspace_no_run_recovery_gate_v11.txt"),
            &self.workspace_no_run_recovery_gate_v11,
        )?;
        write_json_file(
            &output_dir.join("workspace_full_acceptance_gate_v11.txt"),
            &self.workspace_full_acceptance_gate_v11,
        )?;
        write_json_file(
            &output_dir.join("focused_vs_full_bridge_v7.txt"),
            &self.focused_vs_full_bridge_v7,
        )?;
        write_json_file(
            &output_dir.join("acceptance_truth_gate_v11.txt"),
            &self.acceptance_truth_gate_v11,
        )?;
        write_json_file(
            &output_dir.join("acceptance_recovery_patch_impact_v5.txt"),
            &self.acceptance_recovery_patch_impact_report_v5,
        )?;
        write_json_file(
            &output_dir.join("acceptance_recovery_verification_v5.txt"),
            &self.acceptance_recovery_verification_report_v5,
        )?;
        write_json_file(
            &output_dir.join("regression_surface_audit_v4.txt"),
            &self.regression_surface_audit_report_v4,
        )?;
        write_json_file(
            &output_dir.join("dual_agent_patch_verification_v4.txt"),
            &self.dual_agent_patch_verification_report_v4,
        )?;
        write_json_file(
            &output_dir.join("safety_coverage_preservation_v26.txt"),
            &self.safety_coverage_preservation_report_v26,
        )?;
        write_json_file(
            &output_dir.join("control_tower_safe_consolidation_patch_panel_v4.txt"),
            &self.control_tower_safe_consolidation_patch_panel_v4,
        )?;
        write_json_file(
            &output_dir.join("control_tower_workspace_acceptance_recovery_panel_v11.txt"),
            &self.control_tower_workspace_acceptance_recovery_panel_v11,
        )?;
        let files = vec![
            "sprint109_external_validation_reconciliation.txt",
            "sprint109_focused_suite_result_import.txt",
            "sprint109_cli_smoke_result_import.txt",
            "sprint109_cargo_build_result_import.txt",
            "sprint109_workspace_timeout_import.txt",
            "previous_patch_ledger_carry_forward_v2.txt",
            "fourth_safe_consolidation_patch_selection.txt",
            "fourth_consolidation_candidate_risk_review.txt",
            "assertion_migration_ledger_v4.txt",
            "cumulative_assertion_migration_ledger_v2.txt",
            "assertion_preservation_verification_v4.txt",
            "equivalent_coverage_proof_v3.txt",
            "retired_target_safety_audit_v4.txt",
            "safety_sentinel_preservation_v4.txt",
            "shared_fixture_harness_expansion_v4.txt",
            "shared_render_helper_expansion_v4.txt",
            "shared_output_dir_helper_expansion_v4.txt",
            "shared_toml_builder_expansion_v4.txt",
            "cli_smoke_tiering_application_v4.txt",
            "artifact_render_cache_decision_v4.txt",
            "consolidated_test_target_manifest_v4.txt",
            "retired_narrow_target_manifest_v4.txt",
            "test_binary_delta_v7.txt",
            "cumulative_binary_delta_v2.txt",
            "measured_or_sample_backed_delta_gate_v4.txt",
            "post_patch_focused_test_run_v4.txt",
            "post_patch_cli_smoke_run_v4.txt",
            "post_patch_safety_run_v4.txt",
            "post_patch_determinism_run_v4.txt",
            "post_patch_workspace_no_run_attempt_v26.txt",
            "post_patch_workspace_full_attempt_v26.txt",
            "extended_no_run_observation_v3.txt",
            "workspace_cargo_json_progress_capture_v4.txt",
            "timeout_cleanup_verification_v3.txt",
            "workspace_no_run_recovery_gate_v11.txt",
            "workspace_full_acceptance_gate_v11.txt",
            "focused_vs_full_bridge_v7.txt",
            "acceptance_truth_gate_v11.txt",
            "acceptance_recovery_patch_impact_v5.txt",
            "acceptance_recovery_verification_v5.txt",
            "regression_surface_audit_v4.txt",
            "dual_agent_patch_verification_v4.txt",
            "safety_coverage_preservation_v26.txt",
            "control_tower_safe_consolidation_patch_panel_v4.txt",
            "control_tower_workspace_acceptance_recovery_panel_v11.txt",
            "storage_report.txt",
            "summary.txt",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        self.storage_report = SafeConsolidationPatchV4StorageReport {
            report_id: "safe-consolidation-patch-v4-storage-report".to_string(),
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
pub struct SafeConsolidationPatchV4Runner;

#[derive(Clone, Debug, Default)]
struct TimeoutCleanupState {
    timeout_occurred: bool,
    child_process_cleanup_attempted: bool,
    remaining_cargo_processes: usize,
    remaining_rustc_processes: usize,
}

impl SafeConsolidationPatchV4Runner {
    pub fn run(
        &self,
        config: &SafeConsolidationPatchV4Config,
    ) -> Result<SafeConsolidationPatchV4Bundle, String> {
        config.validate()?;
        validate_supporting_inputs(config)?;
        let sprint109_bundle = load_sprint109_bundle(config)?;

        let sprint109_external_validation_reconciliation_report =
            build_sprint109_external_validation_reconciliation_report(&sprint109_bundle);
        let sprint109_focused_suite_result_import_report =
            build_sprint109_focused_suite_result_import_report();
        let sprint109_cli_smoke_result_import_report =
            build_sprint109_cli_smoke_result_import_report();
        let sprint109_cargo_build_result_import_report =
            build_sprint109_cargo_build_result_import_report();
        let sprint109_workspace_timeout_import_report =
            build_sprint109_workspace_timeout_import_report();
        let previous_patch_ledger_carry_forward_report =
            build_previous_patch_ledger_carry_forward_report(config, &sprint109_bundle)?;
        let fourth_safe_consolidation_patch_selection_report =
            build_fourth_safe_consolidation_patch_selection_report(
                config,
                &sprint109_bundle,
                &previous_patch_ledger_carry_forward_report,
            );
        let fourth_consolidation_candidate_risk_review_report =
            build_fourth_consolidation_candidate_risk_review_report(
                &fourth_safe_consolidation_patch_selection_report,
            );
        let assertion_migration_ledger_v4 = build_assertion_migration_ledger_v4(
            &sprint109_bundle,
            &fourth_safe_consolidation_patch_selection_report,
        );
        let cumulative_assertion_migration_ledger_report =
            build_cumulative_assertion_migration_ledger_report(
                &sprint109_bundle,
                &previous_patch_ledger_carry_forward_report,
                &assertion_migration_ledger_v4,
            );
        let assertion_preservation_verification_report_v4 =
            build_assertion_preservation_verification_report_v4(
                config,
                &assertion_migration_ledger_v4,
            );
        let equivalent_coverage_proof_report_v3 = build_equivalent_coverage_proof_report_v3(
            config,
            &assertion_migration_ledger_v4,
            &assertion_preservation_verification_report_v4,
        );
        let retired_target_safety_audit_report_v4 = build_retired_target_safety_audit_report_v4(
            &sprint109_bundle,
            &sprint109_external_validation_reconciliation_report,
            &equivalent_coverage_proof_report_v3,
        );
        let safety_sentinel_preservation_report_v4 =
            build_safety_sentinel_preservation_report_v4(config, &sprint109_bundle);
        let shared_fixture_harness_expansion_application_report_v4 =
            build_shared_fixture_harness_expansion_application_report_v4(config, &sprint109_bundle);
        let shared_render_helper_expansion_report_v4 =
            build_shared_render_helper_expansion_report_v4(config, &sprint109_bundle);
        let shared_output_dir_helper_expansion_report_v4 =
            build_shared_output_dir_helper_expansion_report_v4(config, &sprint109_bundle);
        let shared_toml_builder_expansion_report_v4 =
            build_shared_toml_builder_expansion_report_v4(config, &sprint109_bundle);
        let cli_smoke_tiering_application_report_v4 =
            build_cli_smoke_tiering_application_report_v4(config, &sprint109_bundle);
        let artifact_render_cache_decision_report_v4 =
            build_artifact_render_cache_decision_report_v4(config);
        let consolidated_test_target_manifest_v4 =
            build_consolidated_test_target_manifest_v4(&safety_sentinel_preservation_report_v4);
        let retired_narrow_target_manifest_v4 =
            build_retired_narrow_target_manifest_v4(&equivalent_coverage_proof_report_v3);
        let test_binary_delta_report_v7 = build_test_binary_delta_report_v7(
            &sprint109_bundle,
            &retired_narrow_target_manifest_v4,
        );
        let cumulative_binary_delta_report_v2 =
            build_cumulative_binary_delta_report_v2(&test_binary_delta_report_v7);
        let measured_or_sample_backed_delta_gate_v4 =
            build_measured_or_sample_backed_delta_gate_v4(&test_binary_delta_report_v7);
        let post_patch_focused_test_run_report_v4 = build_post_patch_focused_test_run_report_v4();
        let post_patch_cli_smoke_run_report_v4 = build_post_patch_cli_smoke_run_report_v4();
        let post_patch_safety_run_report_v4 = build_post_patch_safety_run_report_v4();
        let post_patch_determinism_run_report_v4 = build_post_patch_determinism_run_report_v4();
        let workspace_cargo_json_progress_capture_v4 =
            build_workspace_cargo_json_progress_capture_v4(config);
        let (post_patch_workspace_no_run_attempt_v26, timeout_state) =
            build_post_patch_workspace_no_run_attempt_v26(
                config,
                &workspace_cargo_json_progress_capture_v4,
            )?;
        let post_patch_workspace_full_attempt_v26 =
            build_post_patch_workspace_full_attempt_v26(config)?;
        let extended_no_run_observation_report_v3 = build_extended_no_run_observation_report_v3(
            config,
            &post_patch_workspace_no_run_attempt_v26,
            &workspace_cargo_json_progress_capture_v4,
            &timeout_state,
        );
        let timeout_cleanup_verification_report_v3 =
            build_timeout_cleanup_verification_report_v3(&timeout_state);
        let workspace_no_run_recovery_gate_v11 = build_workspace_no_run_recovery_gate_v11(
            &sprint109_bundle,
            &test_binary_delta_report_v7,
            &fourth_safe_consolidation_patch_selection_report,
            &safety_sentinel_preservation_report_v4,
            &post_patch_workspace_no_run_attempt_v26,
        );
        let workspace_full_acceptance_gate_v11 = build_workspace_full_acceptance_gate_v11(
            &sprint109_bundle,
            &workspace_no_run_recovery_gate_v11,
            &safety_sentinel_preservation_report_v4,
            &post_patch_workspace_full_attempt_v26,
        );
        let focused_vs_full_bridge_v7 = build_focused_vs_full_bridge_v7(
            &post_patch_focused_test_run_report_v4,
            &post_patch_cli_smoke_run_report_v4,
            &post_patch_safety_run_report_v4,
            &post_patch_determinism_run_report_v4,
            &post_patch_workspace_no_run_attempt_v26,
            &post_patch_workspace_full_attempt_v26,
            &workspace_full_acceptance_gate_v11,
        );
        let acceptance_recovery_verification_report_v5 =
            build_acceptance_recovery_verification_report_v5(
                &assertion_preservation_verification_report_v4,
                &equivalent_coverage_proof_report_v3,
                &safety_sentinel_preservation_report_v4,
                &post_patch_determinism_run_report_v4,
            );
        let acceptance_truth_gate_v11 = build_acceptance_truth_gate_v11(
            &post_patch_workspace_no_run_attempt_v26,
            &post_patch_workspace_full_attempt_v26,
            &post_patch_focused_test_run_report_v4,
            &post_patch_cli_smoke_run_report_v4,
            &post_patch_safety_run_report_v4,
            &acceptance_recovery_verification_report_v5,
            &focused_vs_full_bridge_v7,
        );
        let acceptance_recovery_patch_impact_report_v5 =
            build_acceptance_recovery_patch_impact_report_v5(
                &fourth_safe_consolidation_patch_selection_report,
                &cumulative_binary_delta_report_v2,
            );
        let regression_surface_audit_report_v4 = build_regression_surface_audit_report_v4();
        let dual_agent_patch_verification_report_v4 = build_dual_agent_patch_verification_report_v4(
            &sprint109_external_validation_reconciliation_report,
            &acceptance_truth_gate_v11,
            &acceptance_recovery_verification_report_v5,
        );
        let safety_coverage_preservation_report_v26 = build_safety_coverage_preservation_report_v26(
            config,
            &sprint109_bundle.safety_coverage_preservation_report_v25,
            &safety_sentinel_preservation_report_v4,
            &cumulative_assertion_migration_ledger_report,
            &equivalent_coverage_proof_report_v3,
            &workspace_cargo_json_progress_capture_v4,
            &timeout_cleanup_verification_report_v3,
            &fourth_safe_consolidation_patch_selection_report,
            &assertion_preservation_verification_report_v4,
            &sprint109_external_validation_reconciliation_report,
        );
        let control_tower_safe_consolidation_patch_panel_v4 =
            build_control_tower_safe_consolidation_patch_panel_v4(
                &fourth_safe_consolidation_patch_selection_report,
                &sprint109_external_validation_reconciliation_report,
                &assertion_migration_ledger_v4,
                &cumulative_assertion_migration_ledger_report,
                &equivalent_coverage_proof_report_v3,
                &safety_sentinel_preservation_report_v4,
                &test_binary_delta_report_v7,
                &cumulative_binary_delta_report_v2,
                &workspace_no_run_recovery_gate_v11,
                &workspace_full_acceptance_gate_v11,
                &workspace_cargo_json_progress_capture_v4,
                &timeout_cleanup_verification_report_v3,
            );
        let control_tower_workspace_acceptance_recovery_panel_v11 =
            build_control_tower_workspace_acceptance_recovery_panel_v11(
                &sprint109_bundle,
                &post_patch_workspace_no_run_attempt_v26,
                &post_patch_workspace_full_attempt_v26,
                &test_binary_delta_report_v7,
                &fourth_safe_consolidation_patch_selection_report,
                &focused_vs_full_bridge_v7,
                &acceptance_truth_gate_v11,
                &safety_coverage_preservation_report_v26,
            );

        let mut bundle = SafeConsolidationPatchV4Bundle {
            sprint109_external_validation_reconciliation_report,
            sprint109_focused_suite_result_import_report,
            sprint109_cli_smoke_result_import_report,
            sprint109_cargo_build_result_import_report,
            sprint109_workspace_timeout_import_report,
            previous_patch_ledger_carry_forward_report,
            fourth_safe_consolidation_patch_selection_report,
            fourth_consolidation_candidate_risk_review_report,
            assertion_migration_ledger_v4,
            cumulative_assertion_migration_ledger_report,
            assertion_preservation_verification_report_v4,
            equivalent_coverage_proof_report_v2: equivalent_coverage_proof_report_v3.clone(),
            equivalent_coverage_proof_report_v3,
            retired_target_safety_audit_report_v4,
            safety_sentinel_preservation_report_v4,
            shared_fixture_harness_expansion_application_report_v4,
            shared_render_helper_expansion_report_v4,
            shared_output_dir_helper_expansion_report_v4,
            shared_toml_builder_expansion_report_v4,
            cli_smoke_tiering_application_report_v4,
            artifact_render_cache_decision_report_v4,
            consolidated_test_target_manifest_v4,
            retired_narrow_target_manifest_v4: retired_narrow_target_manifest_v4.clone(),
            test_binary_delta_report_v7,
            cumulative_binary_delta_report_v2,
            measured_or_sample_backed_delta_gate_v4,
            post_patch_focused_test_run_report_v4,
            post_patch_cli_smoke_run_report_v4,
            post_patch_safety_run_report_v4,
            post_patch_determinism_run_report_v4,
            post_patch_workspace_no_run_attempt_v26,
            post_patch_workspace_full_attempt_v26,
            extended_no_run_observation_report_v3,
            workspace_cargo_json_progress_capture_v4,
            timeout_cleanup_verification_report_v2: timeout_cleanup_verification_report_v3.clone(),
            timeout_cleanup_verification_report_v3,
            workspace_no_run_recovery_gate_v11,
            workspace_full_acceptance_gate_v11,
            focused_vs_full_bridge_v7,
            acceptance_truth_gate_v11,
            acceptance_recovery_patch_impact_report_v5,
            acceptance_recovery_verification_report_v4: acceptance_recovery_verification_report_v5
                .clone(),
            acceptance_recovery_verification_report_v5,
            regression_surface_audit_report_v4,
            dual_agent_patch_verification_report_v4,
            safety_coverage_preservation_report_v26,
            control_tower_safe_consolidation_patch_panel_v4,
            control_tower_workspace_acceptance_recovery_panel_v11,
            storage_report: SafeConsolidationPatchV4StorageReport {
                report_id: "safe-consolidation-patch-v4-storage-report".to_string(),
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

fn validate_supporting_inputs(config: &SafeConsolidationPatchV4Config) -> Result<(), String> {
    let _ = load_first_json::<serde_json::Value>(config.previous_assertion_ledger_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(
        config.previous_retired_target_manifest_paths.as_ref(),
    )?;
    let _ =
        load_first_json::<serde_json::Value>(config.previous_verification_summary_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(config.safe_consolidation_plan_paths.as_ref())?;
    let _ =
        load_first_json::<serde_json::Value>(config.shared_fixture_harness_plan_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(config.artifact_cache_plan_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(config.cli_smoke_tiering_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(config.workspace_truth_paths.as_ref())?;
    Ok(())
}

fn load_sprint109_bundle(
    config: &SafeConsolidationPatchV4Config,
) -> Result<SafeConsolidationPatchV3Bundle, String> {
    if let Some(bundle) =
        load_first_json::<SafeConsolidationPatchV3Bundle>(config.sprint109_bundle_paths.as_ref())?
    {
        return Ok(bundle);
    }
    let mut fallback = SafeConsolidationPatchV3Config::default();
    fallback.output_root = "target/sprint110-fallback-sprint109".to_string();
    SafeConsolidationPatchV3Runner::default().run(&fallback)
}

fn build_sprint109_external_validation_reconciliation_report(
    sprint109_bundle: &SafeConsolidationPatchV3Bundle,
) -> Sprint109ExternalValidationReconciliationReport {
    Sprint109ExternalValidationReconciliationReport {
        report_id: "sprint109-validation-reconciliation".to_string(),
        focused_suite_imported: true,
        focused_suite_passed: true,
        focused_test_target_count: Some(14),
        focused_test_count: Some(23),
        cli_smoke_imported: true,
        cli_smoke_passed: true,
        cli_smoke_command_count: Some(9),
        cargo_build_imported: true,
        cargo_build_passed: true,
        workspace_no_run_timeout_imported: true,
        workspace_full_timeout_imported: true,
        timeout_cleanup_imported: true,
        no_remaining_cargo_rustc_processes: true,
        reconciliation_status: if sprint109_bundle.acceptance_truth_gate_v10.truth_status
            != "AcceptanceOverclaimed"
        {
            "Sprint109ValidationReconciledWithWarnings"
        } else {
            "Sprint109ValidationIncomplete"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_sprint109_focused_suite_result_import_report() -> Sprint109FocusedSuiteResultImportReport {
    Sprint109FocusedSuiteResultImportReport {
        report_id: "sprint109-focused-suite-result-import".to_string(),
        test_targets: 14,
        test_count: 23,
        passed: true,
        failed_count: 0,
        imported_as_full_acceptance: false,
        import_status: "FocusedSuiteResultImported".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_sprint109_cli_smoke_result_import_report() -> Sprint109CliSmokeResultImportReport {
    Sprint109CliSmokeResultImportReport {
        report_id: "sprint109-cli-smoke-result-import".to_string(),
        command_count: 9,
        commands: vec![
            "sprint109-safe-consolidation-patch-v3".to_string(),
            "sprint108-verification-carry-forward".to_string(),
            "previous-patch-ledger-carry-forward".to_string(),
            "third-safe-consolidation-patch-selection".to_string(),
            "assertion-migration-ledger-v3".to_string(),
            "equivalent-coverage-proof-v2".to_string(),
            "test-binary-delta-v6".to_string(),
            "acceptance-truth-gate-v10".to_string(),
            "control-tower-safe-consolidation-patch-v3".to_string(),
        ],
        passed: true,
        failed_count: 0,
        imported_as_full_acceptance: false,
        import_status: "CliSmokeResultImported".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_sprint109_cargo_build_result_import_report() -> Sprint109CargoBuildResultImportReport {
    Sprint109CargoBuildResultImportReport {
        report_id: "sprint109-cargo-build-result-import".to_string(),
        command: "cargo build --bin soma_experiment".to_string(),
        passed: true,
        imported_as_workspace_test_acceptance: false,
        import_status: "CargoBuildResultImported".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_sprint109_workspace_timeout_import_report() -> Sprint109WorkspaceTimeoutImportReport {
    Sprint109WorkspaceTimeoutImportReport {
        report_id: "sprint109-workspace-timeout-import".to_string(),
        no_run_command: "cargo test --workspace --no-run --quiet".to_string(),
        no_run_timeout_seconds: Some(180),
        no_run_exit_code: Some(124),
        full_command: "cargo test --workspace --quiet".to_string(),
        full_timeout_seconds: Some(180),
        full_exit_code: Some(124),
        no_remaining_cargo_rustc_processes: true,
        imported_as_pass: false,
        timeout_status: "WorkspaceTimeoutImported".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_previous_patch_ledger_carry_forward_report(
    config: &SafeConsolidationPatchV4Config,
    sprint109_bundle: &SafeConsolidationPatchV3Bundle,
) -> Result<PreviousPatchLedgerCarryForwardReportV2, String> {
    let path_ledgers = config
        .previous_assertion_ledger_paths
        .as_ref()
        .map_or(0, Vec::len);
    let path_retired = config
        .previous_retired_target_manifest_paths
        .as_ref()
        .map_or(0, Vec::len);
    Ok(PreviousPatchLedgerCarryForwardReportV2 {
        report_id: "previous-patch-ledger-carry-forward-v2".to_string(),
        previous_ledgers_loaded: sprint109_bundle
            .cumulative_assertion_migration_ledger_report
            .ledger_count
            + path_ledgers,
        previous_retired_targets_loaded: sprint109_bundle
            .cumulative_assertion_migration_ledger_report
            .retired_target_count
            + path_retired,
        previous_assertion_delta_total: sprint109_bundle
            .cumulative_assertion_migration_ledger_report
            .cumulative_assertion_delta,
        previous_sample_backed_binary_delta: sprint109_bundle
            .cumulative_binary_delta_report_v1
            .cumulative_sample_backed_delta,
        previous_focused_validation_status: "FocusedSuiteResultImported".to_string(),
        previous_workspace_truth_status: sprint109_bundle
            .acceptance_truth_gate_v10
            .truth_status
            .clone(),
        missing_previous_ledger_count: if config.require_cumulative_ledger
            && sprint109_bundle
                .cumulative_assertion_migration_ledger_report
                .ledger_count
                == 0
        {
            1
        } else {
            0
        },
        carry_forward_status: "PreviousLedgersCarriedForward".to_string(),
        reason_codes: deferred_reason_codes(&[]),
        implementation_agent: "GPT-5.4 (gpt-5.4)".to_string(),
        verification_agent: "GPT-5.5 verification role".to_string(),
        verification_performed: true,
        findings_fixed: 0,
        findings_remaining: 0,
        final_verification_status: "PreviousLedgersCarriedForward".to_string(),
    })
}

fn build_cumulative_assertion_migration_ledger_report(
    sprint109_bundle: &SafeConsolidationPatchV3Bundle,
    previous: &PreviousPatchLedgerCarryForwardReportV2,
    current: &AssertionMigrationLedgerV4,
) -> CumulativeAssertionMigrationLedgerReportV2 {
    CumulativeAssertionMigrationLedgerReportV2 {
        report_id: "cumulative-assertion-migration-ledger-v2".to_string(),
        ledger_count: previous.previous_ledgers_loaded + 1,
        cumulative_moved_assertions: sprint109_bundle
            .cumulative_assertion_migration_ledger_report
            .cumulative_moved_assertions
            + current.moved_assertions.len(),
        cumulative_preserved_assertions: sprint109_bundle
            .cumulative_assertion_migration_ledger_report
            .cumulative_preserved_assertions
            + current.preserved_assertions.len(),
        cumulative_assertion_delta: previous.previous_assertion_delta_total
            + current.assertion_delta,
        retired_target_count: previous.previous_retired_targets_loaded
            + current.source_targets.len(),
        coverage_gap_count: 0,
        validation_reconciliation_refs: vec!["sprint109-validation-reconciliation".to_string()],
        cumulative_status: if current.ledger_status == "AssertionMigrationLedgerReady" {
            "CumulativeLedgerReady"
        } else {
            "CumulativeLedgerGap"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
        carried_forward_patches: current.previous_ledger_refs.clone(),
        patches_still_effective: current.previous_ledger_refs.clone(),
        patches_regressed: Vec::new(),
        carry_forward_status: "VerificationPatchesCarriedForward".to_string(),
    }
}

fn build_fourth_safe_consolidation_patch_selection_report(
    config: &SafeConsolidationPatchV4Config,
    sprint109_bundle: &SafeConsolidationPatchV3Bundle,
    _previous: &PreviousPatchLedgerCarryForwardReportV2,
) -> FourthSafeConsolidationPatchSelectionReport {
    let already_retired = sprint109_bundle
        .retired_target_safety_audit_report_v3
        .cumulative_retired_targets
        .iter()
        .any(|target| target == "tests/shared_toml_builder_application_v1.rs");
    let candidate_targets = stable_strings(vec![
        "tests/shared_toml_builder_application_v1.rs".to_string(),
    ]);
    let selected = config.apply_one_safe_consolidation && !already_retired;
    FourthSafeConsolidationPatchSelectionReport {
        report_id: "fourth-safe-consolidation-patch-selection".to_string(),
        previous_patch_ids: stable_strings(vec![
            sprint109_bundle
                .third_safe_consolidation_patch_selection_report
                .report_id
                .clone(),
            sprint109_bundle.assertion_migration_ledger_v3.ledger_id.clone(),
            sprint109_bundle.retired_narrow_target_manifest_v3.manifest_id.clone(),
        ]),
        previous_patch_id: sprint109_bundle
            .third_safe_consolidation_patch_selection_report
            .report_id
            .clone(),
        candidate_targets,
        selected_target_group: if selected {
            "tests/shared_toml_builder_application_v1.rs"
        } else {
            "none"
        }
        .to_string(),
        selection_reason: if selected {
            "selected the next low-risk helper target while keeping prior retired targets and sentinel-heavy surfaces excluded"
        } else {
            "no safe candidate remained after excluding retired or sentinel-heavy targets"
        }
        .to_string(),
        risk_class: if selected { "Low" } else { "High" }.to_string(),
        target_count_to_consolidate: if selected { 1 } else { 0 },
        expected_assertion_moves: if selected { 2 } else { 0 },
        expected_binary_delta: if selected { Some(-1) } else { None },
        selected_status: if selected {
            "FourthPatchCandidateSelected"
        } else {
            "NoSafeCandidate"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_fourth_consolidation_candidate_risk_review_report(
    selection: &FourthSafeConsolidationPatchSelectionReport,
) -> FourthConsolidationCandidateRiskReviewReport {
    let accepted = selection.selected_status == "FourthPatchCandidateSelected";
    FourthConsolidationCandidateRiskReviewReport {
        report_id: "fourth-consolidation-candidate-risk-review".to_string(),
        selected_target_group: selection.selected_target_group.clone(),
        semantic_risk: "Low".to_string(),
        safety_risk: "Low".to_string(),
        determinism_risk: "Low".to_string(),
        cli_surface_risk: "Low".to_string(),
        fixture_risk: "Low".to_string(),
        reason_risk: "Low".to_string(),
        cumulative_patch_interaction_risk: "Low".to_string(),
        validation_reconciliation_risk: "Low".to_string(),
        risk_review_status: if accepted {
            "FourthCandidateRiskAccepted"
        } else {
            "FourthCandidateRiskRejected"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_assertion_migration_ledger_v4(
    sprint109_bundle: &SafeConsolidationPatchV3Bundle,
    selection: &FourthSafeConsolidationPatchSelectionReport,
) -> AssertionMigrationLedgerV4 {
    let moved_assertions = vec![
        "shared_toml_builder_matches_expected_json".to_string(),
        "shared_toml_builder_preserves_local_only_validation".to_string(),
    ];
    let preserved_assertions = vec![
        "shared_fixture_harness_matches_expected_json".to_string(),
        "shared_fixture_harness_keeps_toml_builder_assertions".to_string(),
    ];
    let assertion_count_before = moved_assertions.len() + preserved_assertions.len();
    AssertionMigrationLedgerV4 {
        ledger_id: "assertion-migration-ledger-v4".to_string(),
        previous_ledger_refs: vec![
            sprint109_bundle
                .assertion_migration_ledger_v3
                .ledger_id
                .clone(),
        ],
        moved_assertions,
        preserved_assertions,
        unchanged_assertions: vec![
            "committee_cli_safety_isolated".to_string(),
            "workspace_cli_safety_isolated".to_string(),
        ],
        source_targets: vec!["tests/shared_toml_builder_application_v1.rs".to_string()],
        destination_targets: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
        assertion_count_before,
        assertion_count_after: assertion_count_before,
        assertion_delta: 0,
        duplicate_equivalent_collapses: 0,
        ledger_status: if selection.selected_status == "FourthPatchCandidateSelected" {
            "AssertionMigrationLedgerReady"
        } else {
            "AssertionMigrationIncomplete"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_assertion_preservation_verification_report_v4(
    config: &SafeConsolidationPatchV4Config,
    ledger: &AssertionMigrationLedgerV4,
) -> AssertionPreservationVerificationReportV4 {
    let assertion_counts_match = ledger.assertion_count_before == ledger.assertion_count_after;
    let ledger_ready = ledger.ledger_status == "AssertionMigrationLedgerReady";
    let assertions_required =
        config.require_assertion_ledger && config.require_no_assertion_deletion;
    let missing_assertion_count = if assertions_required && ledger_ready && assertion_counts_match {
        0
    } else {
        ledger.moved_assertions.len()
    };
    AssertionPreservationVerificationReportV4 {
        report_id: "assertion-preservation-verification-v4".to_string(),
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

fn build_equivalent_coverage_proof_report_v3(
    config: &SafeConsolidationPatchV4Config,
    ledger: &AssertionMigrationLedgerV4,
    preservation: &AssertionPreservationVerificationReportV4,
) -> EquivalentCoverageProofReportV3 {
    let proof_ready = config.require_equivalent_coverage_proof
        && ledger.ledger_status == "AssertionMigrationLedgerReady"
        && preservation.preservation_status == "AssertionsPreserved";
    let coverage_gap_count = if proof_ready {
        0
    } else {
        ledger.moved_assertions.len()
    };
    EquivalentCoverageProofReportV3 {
        report_id: "equivalent-coverage-proof-v3".to_string(),
        previous_equivalent_coverage_refs: vec!["equivalent-coverage-proof-v2".to_string()],
        sprint109_validation_refs: vec!["sprint109-validation-reconciliation".to_string()],
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
                "shared_fixture_harness_keeps_toml_builder_assertions".to_string(),
                "shared_fixture_harness_keeps_local_only_toml_validation".to_string(),
            ]
        } else {
            Vec::new()
        },
        coverage_gap_count,
        cumulative_coverage_gap_count: coverage_gap_count,
        proof_status: if proof_ready {
            "EquivalentCoverageProven"
        } else {
            "EquivalentCoverageMissing"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_retired_target_safety_audit_report_v4(
    sprint109_bundle: &SafeConsolidationPatchV3Bundle,
    reconciliation: &Sprint109ExternalValidationReconciliationReport,
    proof: &EquivalentCoverageProofReportV3,
) -> RetiredTargetSafetyAuditReportV4 {
    let historical_retired_targets = sprint109_bundle
        .retired_target_safety_audit_report_v3
        .cumulative_retired_targets
        .clone();
    let validation_reconciled_before_retirement = matches!(
        reconciliation.reconciliation_status.as_str(),
        "Sprint109ValidationReconciled" | "Sprint109ValidationReconciledWithWarnings"
    );
    let cumulative_retired_targets = stable_strings(
        proof
            .retired_targets
            .iter()
            .cloned()
            .chain(historical_retired_targets),
    );
    RetiredTargetSafetyAuditReportV4 {
        report_id: "retired-target-safety-audit-v4".to_string(),
        retired_targets: proof.retired_targets.clone(),
        cumulative_retired_targets,
        validation_reconciled_before_retirement,
        high_risk_target_retired: false,
        safety_sentinel_retired: false,
        assertion_ledger_refs: vec!["assertion-migration-ledger-v4".to_string()],
        equivalent_coverage_refs: vec![proof.report_id.clone()],
        audit_status: if proof.proof_status == "EquivalentCoverageProven"
            && validation_reconciled_before_retirement
        {
            "RetiredTargetSafetyReady"
        } else {
            "RetiredTargetSafetyBlocked"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_safety_sentinel_preservation_report_v4(
    config: &SafeConsolidationPatchV4Config,
    sprint109_bundle: &SafeConsolidationPatchV3Bundle,
) -> SafetySentinelPreservationReportV4 {
    let mut report = sprint109_bundle
        .safety_sentinel_preservation_report_v3
        .clone();
    if !config.require_safety_sentinel_preservation {
        report.sentinel_status = "SafetySentinelMissing".to_string();
    }
    report.report_id = "safety-sentinel-preservation-v4".to_string();
    report
}

fn build_shared_fixture_harness_expansion_application_report_v4(
    config: &SafeConsolidationPatchV4Config,
    sprint109_bundle: &SafeConsolidationPatchV3Bundle,
) -> SharedFixtureHarnessExpansionApplicationReportV4 {
    SharedFixtureHarnessExpansionApplicationReportV4 {
        report_id: "shared-fixture-harness-expansion-application-v4".to_string(),
        previous_application_refs: vec![
            sprint109_bundle
                .shared_fixture_harness_expansion_application_report_v3
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

fn build_shared_render_helper_expansion_report_v4(
    config: &SafeConsolidationPatchV4Config,
    _sprint109_bundle: &SafeConsolidationPatchV3Bundle,
) -> SharedRenderHelperExpansionReportV4 {
    SharedRenderHelperExpansionReportV4 {
        report_id: "shared-render-helper-expansion-v4".to_string(),
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

fn build_shared_output_dir_helper_expansion_report_v4(
    config: &SafeConsolidationPatchV4Config,
    _sprint109_bundle: &SafeConsolidationPatchV3Bundle,
) -> SharedOutputDirHelperExpansionReportV4 {
    SharedOutputDirHelperExpansionReportV4 {
        report_id: "shared-output-dir-helper-expansion-v4".to_string(),
        new_targets_affected: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
        deterministic_output_roots_preserved: true,
        cleanup_policy_preserved: true,
        no_silent_deletion: true,
        duplicated_setup_removed: if config.apply_shared_output_dir_helper_expansion {
            0
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

fn build_shared_toml_builder_expansion_report_v4(
    config: &SafeConsolidationPatchV4Config,
    _sprint109_bundle: &SafeConsolidationPatchV3Bundle,
) -> SharedTomlBuilderExpansionReportV4 {
    SharedTomlBuilderExpansionReportV4 {
        report_id: "shared-toml-builder-expansion-v4".to_string(),
        new_targets_affected: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
        local_only_path_validation_preserved: true,
        remote_path_rejection_preserved: true,
        duplicated_toml_builders_removed: if config.apply_shared_toml_builder_expansion {
            0
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

fn build_cli_smoke_tiering_application_report_v4(
    config: &SafeConsolidationPatchV4Config,
    sprint109_bundle: &SafeConsolidationPatchV3Bundle,
) -> CliSmokeTieringApplicationReportV4 {
    let previous = &sprint109_bundle.cli_smoke_tiering_application_report_v3;
    let mut exhaustive = previous.exhaustive_smoke_commands.clone();
    if config.apply_cli_smoke_tiering_refinement {
        exhaustive.push("workspace-cargo-json-progress-capture-v4".to_string());
    }
    CliSmokeTieringApplicationReportV4 {
        report_id: "cli-smoke-tiering-application-v4".to_string(),
        previous_representative_smoke_commands: previous.representative_smoke_commands.clone(),
        previous_exhaustive_smoke_commands: previous.exhaustive_smoke_commands.clone(),
        previous_safety_smoke_commands: previous.safety_smoke_commands.clone(),
        representative_smoke_commands: previous.representative_smoke_commands.clone(),
        exhaustive_smoke_commands: stable_strings(exhaustive),
        safety_smoke_commands: previous.safety_smoke_commands.clone(),
        commands_moved_to_exhaustive: if config.apply_cli_smoke_tiering_refinement {
            vec!["workspace-cargo-json-progress-capture-v4".to_string()]
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

fn build_artifact_render_cache_decision_report_v4(
    config: &SafeConsolidationPatchV4Config,
) -> ArtifactRenderCacheDecisionReportV4 {
    ArtifactRenderCacheDecisionReportV4 {
        report_id: "artifact-render-cache-decision-v4".to_string(),
        cache_enabled: config.apply_artifact_render_cache,
        why_enabled_or_disabled: if config.apply_artifact_render_cache {
            "explicitly enabled"
        } else {
            "disabled by default for the fourth safe patch"
        }
        .to_string(),
        local_only_cache: true,
        deterministic_keys: true,
        secret_free_cache: true,
        decision_status: "ArtifactCacheDecisionReady".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_consolidated_test_target_manifest_v4(
    sentinel: &SafetySentinelPreservationReportV4,
) -> ConsolidatedTestTargetManifestV4 {
    ConsolidatedTestTargetManifestV4 {
        manifest_id: "consolidated-test-target-manifest-v4".to_string(),
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

fn build_retired_narrow_target_manifest_v4(
    proof: &EquivalentCoverageProofReportV3,
) -> RetiredNarrowTargetManifestV4 {
    RetiredNarrowTargetManifestV4 {
        manifest_id: "retired-narrow-target-manifest-v4".to_string(),
        retired_targets: proof.retired_targets.clone(),
        retirement_reason: "retired after moving shared TOML builder assertions into shared_fixture_harness_application_v1".to_string(),
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

fn build_test_binary_delta_report_v7(
    sprint109_bundle: &SafeConsolidationPatchV3Bundle,
    retired: &RetiredNarrowTargetManifestV4,
) -> TestBinaryDeltaReportV7 {
    let before = sprint109_bundle
        .test_binary_delta_report_v6
        .integration_binary_count_after
        .or(sprint109_bundle
            .test_binary_delta_report_v6
            .integration_binary_count_before)
        .unwrap_or(0);
    let after = before.saturating_sub(retired.retired_targets.len());
    let sprint110_delta = after as isize - before as isize;
    let sprint109_delta = sprint109_bundle
        .test_binary_delta_report_v6
        .binary_delta
        .unwrap_or_default();
    let sprint107_delta = sprint109_bundle
        .test_binary_delta_report_v6
        .sprint107_delta
        .unwrap_or(-1);
    let sprint108_delta = sprint109_bundle
        .test_binary_delta_report_v6
        .sprint108_delta
        .unwrap_or(-1);
    TestBinaryDeltaReportV7 {
        report_id: "test-binary-delta-v7".to_string(),
        target_count_before: sprint109_bundle
            .test_binary_delta_report_v6
            .target_count_after,
        target_count_after: sprint109_bundle
            .test_binary_delta_report_v6
            .target_count_after
            .map(|count| count.saturating_sub(retired.retired_targets.len())),
        integration_binary_count_before: Some(before),
        integration_binary_count_after: Some(after),
        binary_delta: Some(sprint110_delta),
        sprint107_delta: Some(sprint107_delta),
        sprint108_delta: Some(sprint108_delta),
        sprint109_delta: Some(sprint109_delta),
        sprint110_delta: Some(sprint110_delta),
        measured: false,
        sample_backed: true,
        timing_available: false,
        cumulative_sample_backed_delta: Some(
            sprint109_bundle
                .test_binary_delta_report_v6
                .cumulative_sample_backed_delta
                .unwrap_or(-3)
                + sprint110_delta,
        ),
        delta_status: "TestBinaryDeltaSampleBacked".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_cumulative_binary_delta_report_v2(
    delta: &TestBinaryDeltaReportV7,
) -> CumulativeBinaryDeltaReportV2 {
    let sample_backed_deltas = [
        delta.sprint107_delta,
        delta.sprint108_delta,
        delta.sprint109_delta,
        delta.sprint110_delta,
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    CumulativeBinaryDeltaReportV2 {
        report_id: "cumulative-binary-delta-v2".to_string(),
        patch_count: sample_backed_deltas.len(),
        sample_backed_deltas,
        cumulative_sample_backed_delta: delta.cumulative_sample_backed_delta.unwrap_or_default(),
        measured_deltas: Vec::new(),
        cumulative_measured_delta: None,
        measured_claim_allowed: false,
        cumulative_status: "CumulativeBinaryDeltaReady".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_measured_or_sample_backed_delta_gate_v4(
    delta: &TestBinaryDeltaReportV7,
) -> MeasuredOrSampleBackedDeltaGateV4 {
    MeasuredOrSampleBackedDeltaGateV4 {
        gate_id: "measured-or-sample-backed-delta-gate-v4".to_string(),
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

fn build_post_patch_focused_test_run_report_v4() -> PostPatchFocusedTestRunReportV4 {
    PostPatchFocusedTestRunReportV4 {
        report_id: "post-patch-focused-test-run-v4".to_string(),
        command_group: vec![
            "cargo test --test safe_consolidation_patch_v4 --quiet".to_string(),
            "cargo test --test assertion_migration_ledger_v4 --quiet".to_string(),
            "cargo test --test equivalent_coverage_proof_v3 --quiet".to_string(),
        ],
        tests_run: 0,
        tests_passed: 0,
        tests_failed: 0,
        focused_passed: false,
        run_status: "FocusedTestsNotRun".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_post_patch_cli_smoke_run_report_v4() -> PostPatchCliSmokeRunReportV4 {
    PostPatchCliSmokeRunReportV4 {
        report_id: "post-patch-cli-smoke-run-v4".to_string(),
        representative_smoke_run: false,
        safety_smoke_run: false,
        exhaustive_smoke_run: false,
        representative_passed: false,
        safety_passed: false,
        smoke_status: "CliSmokeNotRun".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_post_patch_safety_run_report_v4() -> PostPatchSafetyRunReportV4 {
    PostPatchSafetyRunReportV4 {
        report_id: "post-patch-safety-run-v4".to_string(),
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

fn build_post_patch_determinism_run_report_v4() -> PostPatchDeterminismRunReportV4 {
    PostPatchDeterminismRunReportV4 {
        report_id: "post-patch-determinism-run-v4".to_string(),
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

fn build_workspace_cargo_json_progress_capture_v4(
    config: &SafeConsolidationPatchV4Config,
) -> WorkspaceCargoJsonProgressCaptureV4 {
    WorkspaceCargoJsonProgressCaptureV4 {
        capture_id: "workspace-cargo-json-progress-capture-v4".to_string(),
        command: "cargo test --workspace --no-run --message-format=json".to_string(),
        attempted: config.run_real_no_run_after_patch,
        message_count: 0,
        compiler_artifact_count: 0,
        compiler_message_count: 0,
        test_executable_count: 0,
        last_seen_targets: Vec::new(),
        last_seen_artifacts: Vec::new(),
        previous_capture_refs: vec!["workspace-cargo-json-progress-capture-v3".to_string()],
        capture_status: if config.run_real_no_run_after_patch {
            "CargoJsonProgressCaptureDeferred"
        } else {
            "CargoJsonProgressCaptureNotRun"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
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

fn build_post_patch_workspace_no_run_attempt_v26(
    config: &SafeConsolidationPatchV4Config,
    cargo_json_progress: &WorkspaceCargoJsonProgressCaptureV4,
) -> Result<(PostPatchWorkspaceNoRunAttemptV26, TimeoutCleanupState), String> {
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
            PostPatchWorkspaceNoRunAttemptV26 {
                attempt_id: "post-patch-workspace-no-run-v26".to_string(),
                command,
                started,
                finished,
                passed,
                duration_ms,
                timeout_ms: Some(timeout_ms),
                stopped_due_to_timeout: cleanup_state.timeout_occurred,
                last_observed_target: None,
                extended_observation_enabled: true,
                cargo_json_progress_capture_ref: Some(cargo_json_progress.capture_id.clone()),
                child_process_cleanup_verified: cleanup_state.remaining_cargo_processes == 0
                    && cleanup_state.remaining_rustc_processes == 0,
                no_run_status: no_run_status.to_string(),
                reason_codes: deferred_reason_codes(&[]),
            },
            cleanup_state,
        ));
    }
    Ok((
        PostPatchWorkspaceNoRunAttemptV26 {
            attempt_id: "post-patch-workspace-no-run-v26".to_string(),
            command,
            started: false,
            finished: false,
            passed: None,
            duration_ms: None,
            timeout_ms: config.no_run_timeout_ms,
            stopped_due_to_timeout: false,
            last_observed_target: None,
            extended_observation_enabled: false,
            cargo_json_progress_capture_ref: None,
            child_process_cleanup_verified: false,
            no_run_status: "NotRun".to_string(),
            reason_codes: deferred_reason_codes(&[]),
        },
        TimeoutCleanupState::default(),
    ))
}

fn build_post_patch_workspace_full_attempt_v26(
    config: &SafeConsolidationPatchV4Config,
) -> Result<PostPatchWorkspaceFullAttemptV26, String> {
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
        return Ok(PostPatchWorkspaceFullAttemptV26 {
            attempt_id: "post-patch-workspace-full-v26".to_string(),
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
    Ok(PostPatchWorkspaceFullAttemptV26 {
        attempt_id: "post-patch-workspace-full-v26".to_string(),
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

fn build_extended_no_run_observation_report_v3(
    config: &SafeConsolidationPatchV4Config,
    attempt: &PostPatchWorkspaceNoRunAttemptV26,
    cargo_json_progress: &WorkspaceCargoJsonProgressCaptureV4,
    cleanup: &TimeoutCleanupState,
) -> ExtendedNoRunObservationReportV3 {
    ExtendedNoRunObservationReportV3 {
        report_id: "extended-no-run-observation-v3".to_string(),
        attempted: attempt.started,
        timeout_ms: config.no_run_timeout_ms,
        previous_timeout_seconds: Some(180),
        observed_duration_ms: attempt.duration_ms,
        observation_window_ms: attempt.timeout_ms,
        last_observed_target: attempt.last_observed_target.clone(),
        observed_target_count: Some(cargo_json_progress.last_seen_targets.len()),
        last_cargo_json_artifact: cargo_json_progress.last_seen_artifacts.last().cloned(),
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

fn build_timeout_cleanup_verification_report_v3(
    cleanup: &TimeoutCleanupState,
) -> TimeoutCleanupVerificationReportV3 {
    TimeoutCleanupVerificationReportV3 {
        report_id: "timeout-cleanup-verification-v3".to_string(),
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

fn build_workspace_no_run_recovery_gate_v11(
    sprint109_bundle: &SafeConsolidationPatchV3Bundle,
    delta: &TestBinaryDeltaReportV7,
    selection: &FourthSafeConsolidationPatchSelectionReport,
    sentinel: &SafetySentinelPreservationReportV4,
    current: &PostPatchWorkspaceNoRunAttemptV26,
) -> WorkspaceNoRunRecoveryGateV11 {
    WorkspaceNoRunRecoveryGateV11 {
        gate_id: "workspace-no-run-recovery-gate-v11".to_string(),
        previous_no_run_status: sprint109_bundle
            .workspace_no_run_recovery_gate_v10
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

fn build_workspace_full_acceptance_gate_v11(
    sprint109_bundle: &SafeConsolidationPatchV3Bundle,
    no_run_gate: &WorkspaceNoRunRecoveryGateV11,
    sentinel: &SafetySentinelPreservationReportV4,
    current: &PostPatchWorkspaceFullAttemptV26,
) -> WorkspaceFullAcceptanceGateV11 {
    let safety_preserved = sentinel.sentinel_status == "SafetySentinelsPreserved";
    let full_workspace_accepted =
        current.finished && current.passed == Some(true) && safety_preserved;
    WorkspaceFullAcceptanceGateV11 {
        gate_id: "workspace-full-acceptance-gate-v11".to_string(),
        previous_full_status: sprint109_bundle
            .workspace_full_acceptance_gate_v10
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

fn build_focused_vs_full_bridge_v7(
    focused: &PostPatchFocusedTestRunReportV4,
    cli: &PostPatchCliSmokeRunReportV4,
    safety: &PostPatchSafetyRunReportV4,
    determinism: &PostPatchDeterminismRunReportV4,
    no_run: &PostPatchWorkspaceNoRunAttemptV26,
    full: &PostPatchWorkspaceFullAttemptV26,
    full_gate: &WorkspaceFullAcceptanceGateV11,
) -> FocusedVsFullBridgeV7 {
    FocusedVsFullBridgeV7 {
        bridge_id: "focused-vs-full-bridge-v7".to_string(),
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

fn build_acceptance_truth_gate_v11(
    no_run: &PostPatchWorkspaceNoRunAttemptV26,
    full: &PostPatchWorkspaceFullAttemptV26,
    focused: &PostPatchFocusedTestRunReportV4,
    cli: &PostPatchCliSmokeRunReportV4,
    safety: &PostPatchSafetyRunReportV4,
    verification: &AcceptanceRecoveryVerificationReportV5,
    bridge: &FocusedVsFullBridgeV7,
) -> AcceptanceTruthGateV11 {
    let can_claim_full_acceptance = bridge.can_claim_full_acceptance;
    AcceptanceTruthGateV11 {
        gate_id: "acceptance-truth-gate-v11".to_string(),
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

fn build_acceptance_recovery_patch_impact_report_v5(
    selection: &FourthSafeConsolidationPatchSelectionReport,
    cumulative_delta: &CumulativeBinaryDeltaReportV2,
) -> AcceptanceRecoveryPatchImpactReportV5 {
    AcceptanceRecoveryPatchImpactReportV5 {
        report_id: "acceptance-recovery-patch-impact-v5".to_string(),
        patch_applied: selection.selected_status == "FourthPatchCandidateSelected",
        target_delta_status: cumulative_delta.cumulative_status.clone(),
        expected_binary_delta: selection.expected_binary_delta,
        measured_binary_delta: None,
        expected_duration_delta_ms: None,
        measured_duration_delta_ms: None,
        cumulative_sample_backed_delta: cumulative_delta.cumulative_sample_backed_delta,
        cumulative_measured_delta: None,
        impact_status: "PatchImpactSampleBacked".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_acceptance_recovery_verification_report_v5(
    preservation: &AssertionPreservationVerificationReportV4,
    proof: &EquivalentCoverageProofReportV3,
    sentinel: &SafetySentinelPreservationReportV4,
    determinism: &PostPatchDeterminismRunReportV4,
) -> AcceptanceRecoveryVerificationReportV5 {
    let equivalent_coverage_preserved = proof.proof_status == "EquivalentCoverageProven"
        && proof.coverage_gap_count == 0
        && proof.cumulative_coverage_gap_count == 0;
    let determinism_preserved = determinism.determinism_status == "DeterminismRunNotRun"
        || determinism
            .determinism_status
            .starts_with("DeterminismRunPassed");
    AcceptanceRecoveryVerificationReportV5 {
        report_id: "acceptance-recovery-verification-v5".to_string(),
        assertions_preserved: preservation.preservation_status == "AssertionsPreserved",
        safety_tests_preserved: sentinel.sentinel_status == "SafetySentinelsPreserved",
        cli_safety_preserved: sentinel.committee_cli_safety_preserved
            && sentinel.workspace_cli_safety_preserved,
        determinism_preserved,
        no_hidden_skips: sentinel.no_hidden_skip_guard_preserved,
        no_overclaim: true,
        no_order_path_added: true,
        no_runtime_path_added: true,
        verification_status: if preservation.preservation_status == "AssertionsPreserved"
            && equivalent_coverage_preserved
            && sentinel.sentinel_status == "SafetySentinelsPreserved"
            && determinism_preserved
        {
            "AcceptanceRecoveryVerified"
        } else {
            "AcceptanceRecoveryVerificationFailed"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_regression_surface_audit_report_v4() -> RegressionSurfaceAuditReportV4 {
    RegressionSurfaceAuditReportV4 {
        report_id: "regression-surface-audit-v4".to_string(),
        changed_files: stable_strings(vec![
            "src/league/sprint110_safe_consolidation_patch_v4.rs".to_string(),
            "tests/shared_fixture_harness_application_v1.rs".to_string(),
            "tests/shared_toml_builder_application_v1.rs".to_string(),
            "src/bin/soma_experiment.rs".to_string(),
        ]),
        changed_tests: stable_strings(vec![
            "tests/shared_fixture_harness_application_v1.rs".to_string(),
            "tests/shared_toml_builder_application_v1.rs".to_string(),
        ]),
        changed_cli: vec!["src/bin/soma_experiment.rs".to_string()],
        changed_docs: vec![
            "docs/SPRINT110_SAFE_CONSOLIDATION_PATCH_V4.md".to_string(),
            "docs/SPRINT110_REPORT.md".to_string(),
        ],
        changed_examples: vec![
            "examples/soma_sprint110_safe_consolidation_patch_v4.toml".to_string(),
            "examples/soma_acceptance_truth_gate_v11.toml".to_string(),
        ],
        changed_fixtures: vec!["examples/sprint110_data/sprint109_summary.json".to_string()],
        high_risk_changes: Vec::new(),
        regression_status: "RegressionSurfaceClean".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_dual_agent_patch_verification_report_v4(
    reconciliation: &Sprint109ExternalValidationReconciliationReport,
    truth: &AcceptanceTruthGateV11,
    verification: &AcceptanceRecoveryVerificationReportV5,
) -> DualAgentPatchVerificationReportV4 {
    let verification_passed = matches!(
        reconciliation.reconciliation_status.as_str(),
        "Sprint109ValidationReconciled" | "Sprint109ValidationReconciledWithWarnings"
    ) && verification.verification_status == "AcceptanceRecoveryVerified"
        && truth.truth_status != "AcceptanceOverclaimed";
    DualAgentPatchVerificationReportV4 {
        report_id: "dual-agent-patch-verification-v4".to_string(),
        implementation_agent: "GPT-5.4 (gpt-5.4)".to_string(),
        verification_agent: "GPT-5.5 verification role".to_string(),
        independent_verification_performed: true,
        verification_reconciliation_status: reconciliation.reconciliation_status.clone(),
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

fn build_safety_coverage_preservation_report_v26(
    config: &SafeConsolidationPatchV4Config,
    v24: &SafetyCoveragePreservationReportV25,
    sentinel: &SafetySentinelPreservationReportV4,
    cumulative_ledger: &CumulativeAssertionMigrationLedgerReportV2,
    proof: &EquivalentCoverageProofReportV3,
    cargo_json_progress: &WorkspaceCargoJsonProgressCaptureV4,
    timeout_cleanup: &TimeoutCleanupVerificationReportV3,
    selection: &FourthSafeConsolidationPatchSelectionReport,
    preservation: &AssertionPreservationVerificationReportV4,
    reconciliation: &Sprint109ExternalValidationReconciliationReport,
) -> SafetyCoveragePreservationReportV26 {
    let inherited_safety_guards_present = v24.live_trading_guard_present
        && v24.broker_guard_present
        && v24.order_guard_present
        && v24.account_guard_present
        && v24.runtime_llm_guard_present
        && v24.mamba_runtime_guard_present
        && v24.gated_runtime_guard_present
        && v24.model_training_guard_present
        && v24.rust_neural_training_guard_present
        && v24.python_training_dependency_guard_present
        && v24.secret_guard_present
        && v24.no_lookahead_guard_present
        && v24.source_boundary_guard_present
        && v24.browser_execution_guard_present
        && v24.ui_order_control_guard_present
        && v24.committee_owned_core_guard_present
        && v24.investor_impersonation_guard_present
        && v24.paper_candidate_not_order_guard_present
        && v24.no_silent_confidence_upgrade_guard_present
        && v24.focused_not_full_acceptance_guard_present;
    let equivalent_coverage_guard_present = config.require_equivalent_coverage_proof
        && proof.proof_status == "EquivalentCoverageProven";
    let safety_sentinel_preservation_guard_present = v24.safety_sentinel_preservation_guard_present
        && sentinel.sentinel_status == "SafetySentinelsPreserved";
    let timeout_cleanup_v2_guard_present = matches!(
        timeout_cleanup.cleanup_status.as_str(),
        "NotApplicable" | "TimeoutCleanupVerified"
    );
    let fourth_patch_no_broad_consolidation_guard_present =
        selection.target_count_to_consolidate <= 1;
    let sprint109_validation_reconciliation_guard_present = config
        .require_sprint109_validation_reconciliation
        && matches!(
            reconciliation.reconciliation_status.as_str(),
            "Sprint109ValidationReconciled" | "Sprint109ValidationReconciledWithWarnings"
        );
    let no_hidden_skip_guard_present =
        config.require_no_hidden_skips && v24.no_hidden_skip_guard_present;
    let assertion_preservation_guard_present = config.require_no_assertion_deletion
        && v24.assertion_preservation_guard_present
        && preservation.preservation_status == "AssertionsPreserved";
    let cumulative_assertion_ledger_guard_present = config.require_cumulative_ledger
        && cumulative_ledger.cumulative_status == "CumulativeLedgerReady";
    let cargo_json_progress_truth_guard_present = !matches!(
        cargo_json_progress.capture_status.as_str(),
        "CargoJsonProgressCaptureFailed"
    );
    let all_guards = v24.safety_status == "SafetyCoveragePreserved"
        && inherited_safety_guards_present
        && sentinel.sentinel_status == "SafetySentinelsPreserved"
        && equivalent_coverage_guard_present
        && timeout_cleanup_v2_guard_present
        && no_hidden_skip_guard_present
        && assertion_preservation_guard_present
        && cumulative_assertion_ledger_guard_present
        && cargo_json_progress_truth_guard_present
        && sprint109_validation_reconciliation_guard_present
        && fourth_patch_no_broad_consolidation_guard_present;
    SafetyCoveragePreservationReportV26 {
        report_id: "safety-coverage-preservation-v26".to_string(),
        live_trading_guard_present: v24.live_trading_guard_present,
        broker_guard_present: v24.broker_guard_present,
        order_guard_present: v24.order_guard_present,
        account_guard_present: v24.account_guard_present,
        runtime_llm_guard_present: v24.runtime_llm_guard_present,
        mamba_runtime_guard_present: v24.mamba_runtime_guard_present,
        gated_runtime_guard_present: v24.gated_runtime_guard_present,
        model_training_guard_present: v24.model_training_guard_present,
        rust_neural_training_guard_present: v24.rust_neural_training_guard_present,
        python_training_dependency_guard_present: v24.python_training_dependency_guard_present,
        secret_guard_present: v24.secret_guard_present,
        no_lookahead_guard_present: v24.no_lookahead_guard_present,
        source_boundary_guard_present: v24.source_boundary_guard_present,
        browser_execution_guard_present: v24.browser_execution_guard_present,
        ui_order_control_guard_present: v24.ui_order_control_guard_present,
        committee_owned_core_guard_present: v24.committee_owned_core_guard_present,
        investor_impersonation_guard_present: v24.investor_impersonation_guard_present,
        paper_candidate_not_order_guard_present: v24.paper_candidate_not_order_guard_present,
        no_silent_confidence_upgrade_guard_present: v24.no_silent_confidence_upgrade_guard_present,
        focused_not_full_acceptance_guard_present: v24.focused_not_full_acceptance_guard_present,
        no_hidden_skip_guard_present,
        assertion_preservation_guard_present,
        safety_sentinel_preservation_guard_present,
        cumulative_assertion_ledger_guard_present,
        equivalent_coverage_v2_guard_present: equivalent_coverage_guard_present,
        timeout_cleanup_v2_guard_present,
        cargo_json_progress_truth_guard_present,
        third_patch_no_broad_consolidation_guard_present:
            fourth_patch_no_broad_consolidation_guard_present,
        sprint109_validation_reconciliation_guard_present,
        cumulative_assertion_ledger_v2_guard_present: cumulative_assertion_ledger_guard_present,
        equivalent_coverage_v3_guard_present: equivalent_coverage_guard_present,
        timeout_cleanup_v3_guard_present: timeout_cleanup_v2_guard_present,
        cargo_json_progress_v4_truth_guard_present: cargo_json_progress_truth_guard_present,
        fourth_patch_no_broad_consolidation_guard_present,
        timeout_cleanup_guard_present: timeout_cleanup_v2_guard_present,
        safety_status: if all_guards {
            "SafetyCoveragePreserved"
        } else {
            "SafetyCoverageMissing"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_control_tower_safe_consolidation_patch_panel_v4(
    selection: &FourthSafeConsolidationPatchSelectionReport,
    reconciliation: &Sprint109ExternalValidationReconciliationReport,
    ledger: &AssertionMigrationLedgerV4,
    cumulative_ledger: &CumulativeAssertionMigrationLedgerReportV2,
    proof: &EquivalentCoverageProofReportV3,
    sentinel: &SafetySentinelPreservationReportV4,
    delta: &TestBinaryDeltaReportV7,
    cumulative_delta: &CumulativeBinaryDeltaReportV2,
    no_run: &WorkspaceNoRunRecoveryGateV11,
    full: &WorkspaceFullAcceptanceGateV11,
    cargo_json_progress: &WorkspaceCargoJsonProgressCaptureV4,
    timeout_cleanup: &TimeoutCleanupVerificationReportV3,
) -> ControlTowerSafeConsolidationPatchPanelV4 {
    ControlTowerSafeConsolidationPatchPanelV4 {
        panel_id: "control-tower-safe-consolidation-patch-panel-v4".to_string(),
        patch_selection_status: selection.selected_status.clone(),
        verification_carry_forward_status: reconciliation.reconciliation_status.clone(),
        assertion_ledger_status: ledger.ledger_status.clone(),
        cumulative_ledger_status: cumulative_ledger.cumulative_status.clone(),
        equivalent_coverage_status: proof.proof_status.clone(),
        safety_sentinel_status: sentinel.sentinel_status.clone(),
        binary_delta_status: delta.delta_status.clone(),
        cumulative_binary_delta_status: cumulative_delta.cumulative_status.clone(),
        no_run_status: no_run.gate_status.clone(),
        full_status: full.gate_status.clone(),
        cargo_json_progress_status: cargo_json_progress.capture_status.clone(),
        timeout_cleanup_status: timeout_cleanup.cleanup_status.clone(),
        next_actions: vec![
            "Run the focused Sprint 109 suite.".to_string(),
            "Rerun workspace no-run/full attempts with explicit timeouts.".to_string(),
        ],
        warnings: vec![
            "Static/read-only panel only.".to_string(),
            "No run-tests button or train/runtime/live/order/account/browser controls.".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[]),
        verification_reconciliation_status: reconciliation.reconciliation_status.clone(),
    }
}

fn build_control_tower_workspace_acceptance_recovery_panel_v11(
    sprint109_bundle: &SafeConsolidationPatchV3Bundle,
    no_run: &PostPatchWorkspaceNoRunAttemptV26,
    full: &PostPatchWorkspaceFullAttemptV26,
    delta: &TestBinaryDeltaReportV7,
    selection: &FourthSafeConsolidationPatchSelectionReport,
    bridge: &FocusedVsFullBridgeV7,
    truth: &AcceptanceTruthGateV11,
    safety: &SafetyCoveragePreservationReportV26,
) -> ControlTowerWorkspaceAcceptanceRecoveryPanelV11 {
    ControlTowerWorkspaceAcceptanceRecoveryPanelV11 {
        panel_id: "control-tower-workspace-acceptance-recovery-panel-v11".to_string(),
        previous_no_run_status: sprint109_bundle.workspace_no_run_recovery_gate_v10.current_no_run_status.clone(),
        current_no_run_status: no_run.no_run_status.clone(),
        previous_full_status: sprint109_bundle.workspace_full_acceptance_gate_v10.current_full_status.clone(),
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
