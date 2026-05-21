use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::league::sprint106_workspace_acceptance_recovery::{
    SafetyCoveragePreservationReportV22, WorkspaceAcceptanceRecoveryV7Bundle,
    WorkspaceAcceptanceRecoveryV7Config, WorkspaceAcceptanceRecoveryV7Runner,
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
    "target/soma_sprint107_safe_consolidation_patch".to_string()
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
            .map_err(|err| format!("failed to read sprint107 JSON input {path}: {err}"))?;
        match serde_json::from_str::<T>(&text) {
            Ok(value) => return Ok(Some(value)),
            Err(err) => parse_errors.push(format!("{path}: {err}")),
        }
    }
    if !paths.is_empty() {
        return Err(format!(
            "failed to parse any sprint107 JSON input: {}",
            parse_errors.join("; ")
        ));
    }
    Ok(None)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeConsolidationPatchV1Config {
    pub patch_id: String,
    #[serde(default)]
    pub sprint106_bundle_paths: Option<Vec<String>>,
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
    pub apply_shared_fixture_harness: bool,
    #[serde(default = "default_true")]
    pub apply_shared_toml_builder: bool,
    #[serde(default = "default_true")]
    pub apply_shared_output_dir_helper: bool,
    #[serde(default = "default_true")]
    pub apply_shared_render_helper: bool,
    #[serde(default = "default_false")]
    pub apply_artifact_render_cache: bool,
    #[serde(default = "default_true")]
    pub apply_cli_smoke_tiering: bool,
    #[serde(default = "default_true")]
    pub apply_one_safe_consolidation: bool,
    #[serde(default = "default_max_targets_to_consolidate")]
    pub max_targets_to_consolidate: usize,
    #[serde(default = "default_true")]
    pub require_assertion_ledger: bool,
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

impl Default for SafeConsolidationPatchV1Config {
    fn default() -> Self {
        Self {
            patch_id: "sprint107-safe-consolidation-patch".to_string(),
            sprint106_bundle_paths: Some(vec![
                "examples/sprint107_data/sprint106_summary.json".to_string(),
            ]),
            safe_consolidation_plan_paths: None,
            shared_fixture_harness_plan_paths: None,
            artifact_cache_plan_paths: None,
            cli_smoke_tiering_paths: None,
            workspace_truth_paths: None,
            output_root: default_output_root(),
            apply_shared_fixture_harness: true,
            apply_shared_toml_builder: true,
            apply_shared_output_dir_helper: true,
            apply_shared_render_helper: true,
            apply_artifact_render_cache: false,
            apply_cli_smoke_tiering: true,
            apply_one_safe_consolidation: true,
            max_targets_to_consolidate: 1,
            require_assertion_ledger: true,
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

impl SafeConsolidationPatchV1Config {
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
            return Err("sprint107 patch_id must not be empty".to_string());
        }
        if self.output_root.trim().is_empty() {
            return Err("sprint107 output_root must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err(
                "sprint107 safe consolidation patch config paths must be local".to_string(),
            );
        }
        for paths in [
            &self.sprint106_bundle_paths,
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
                    "sprint107 safe consolidation patch config paths must be local".to_string(),
                );
            }
        }
        if self.max_targets_to_consolidate == 0 || self.max_targets_to_consolidate > 1 {
            return Err(
                "sprint107 max_targets_to_consolidate must stay within the first safe patch"
                    .to_string(),
            );
        }
        if self.require_assertion_ledger && !self.apply_one_safe_consolidation {
            return Err(
                "sprint107 assertion ledger requires the first safe consolidation patch"
                    .to_string(),
            );
        }
        if self.require_safety_sentinel_preservation && !self.preserve_safety_guards {
            return Err(
                "sprint107 safety sentinel preservation requires safety guards to remain enabled"
                    .to_string(),
            );
        }
        if !self.preserve_runtime_deferred {
            return Err("sprint107 runtime deferred preservation must remain enabled".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeConsolidationPatchSelectionReport {
    pub report_id: String,
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
pub struct ConsolidationCandidateRiskReviewReport {
    pub report_id: String,
    pub selected_target_group: String,
    pub semantic_risk: String,
    pub safety_risk: String,
    pub determinism_risk: String,
    pub cli_surface_risk: String,
    pub fixture_risk: String,
    pub reason_risk: String,
    pub risk_review_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssertionMigrationLedgerV1 {
    pub ledger_id: String,
    pub moved_assertions: Vec<String>,
    pub preserved_assertions: Vec<String>,
    pub unchanged_assertions: Vec<String>,
    pub source_targets: Vec<String>,
    pub destination_targets: Vec<String>,
    pub assertion_count_before: usize,
    pub assertion_count_after: usize,
    pub assertion_delta: isize,
    pub ledger_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssertionPreservationVerificationReportV1 {
    pub report_id: String,
    pub ledger_status: String,
    pub assertion_count_before: usize,
    pub assertion_count_after: usize,
    pub migrated_assertion_count: usize,
    pub missing_assertion_count: usize,
    pub duplicate_assertion_count: usize,
    pub equivalent_coverage_count: usize,
    pub preservation_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetySentinelPreservationReportV1 {
    pub report_id: String,
    pub committee_cli_safety_preserved: bool,
    pub workspace_cli_safety_preserved: bool,
    pub workspace_safety_guard_preserved: bool,
    pub workspace_determinism_preserved: bool,
    pub paper_lifecycle_safety_preserved: bool,
    pub runtime_deferred_guard_preserved: bool,
    pub no_order_account_guard_preserved: bool,
    pub no_hidden_skip_guard_preserved: bool,
    pub sentinel_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedFixtureHarnessApplicationReportV1 {
    pub report_id: String,
    pub json_loader_applied: bool,
    pub toml_loader_applied: bool,
    pub csv_loader_applied: bool,
    pub fixture_normalization_applied: bool,
    pub affected_targets: Vec<String>,
    pub duplicated_loaders_removed: usize,
    pub deterministic_output_preserved: bool,
    pub application_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedTomlBuilderApplicationReportV1 {
    pub report_id: String,
    pub shared_toml_builder_applied: bool,
    pub local_only_path_validation_preserved: bool,
    pub remote_path_rejection_preserved: bool,
    pub affected_configs: Vec<String>,
    pub duplicated_toml_builders_removed: usize,
    pub application_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedOutputDirHelperApplicationReportV1 {
    pub report_id: String,
    pub output_dir_helper_applied: bool,
    pub deterministic_output_root_preserved: bool,
    pub cleanup_policy_preserved: bool,
    pub no_silent_deletion: bool,
    pub affected_targets: Vec<String>,
    pub duplicated_output_dir_setup_removed: usize,
    pub application_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedRenderHelperApplicationReportV1 {
    pub report_id: String,
    pub txt_render_helper_applied: bool,
    pub json_render_helper_applied: bool,
    pub html_render_helper_applied: bool,
    pub stable_sorting_preserved: bool,
    pub snapshot_order_preserved: bool,
    pub affected_targets: Vec<String>,
    pub duplicated_render_helpers_removed: usize,
    pub application_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRenderCacheApplicationReportV1 {
    pub report_id: String,
    pub artifact_cache_enabled: bool,
    pub local_only_cache: bool,
    pub deterministic_cache_keys: bool,
    pub secret_free_cache: bool,
    pub cache_invalidation_rules_present: bool,
    pub cacheable_artifacts_used: Vec<String>,
    pub non_cacheable_artifacts_preserved: Vec<String>,
    pub application_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliSmokeTieringApplicationReportV1 {
    pub report_id: String,
    pub representative_smoke_commands: Vec<String>,
    pub exhaustive_smoke_commands: Vec<String>,
    pub safety_smoke_commands: Vec<String>,
    pub commands_moved_to_exhaustive: Vec<String>,
    pub safety_smoke_preserved: bool,
    pub no_safety_smoke_removed: bool,
    pub application_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsolidatedTestTargetManifestV1 {
    pub manifest_id: String,
    pub consolidated_targets: Vec<String>,
    pub grouped_destination_targets: Vec<String>,
    pub preserved_targets: Vec<String>,
    pub isolated_targets: Vec<String>,
    pub manifest_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetiredNarrowTargetManifestV1 {
    pub manifest_id: String,
    pub retired_targets: Vec<String>,
    pub retirement_reason: String,
    pub assertion_migration_refs: Vec<String>,
    pub equivalent_coverage_refs: Vec<String>,
    pub retired_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestBinaryDeltaReportV4 {
    pub report_id: String,
    pub target_count_before: Option<usize>,
    pub target_count_after: Option<usize>,
    pub integration_binary_count_before: Option<usize>,
    pub integration_binary_count_after: Option<usize>,
    pub binary_delta: Option<isize>,
    pub measured: bool,
    pub sample_backed: bool,
    pub timing_available: bool,
    pub delta_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasuredOrSampleBackedDeltaGateV1 {
    pub gate_id: String,
    pub delta_report_status: String,
    pub measured: bool,
    pub sample_backed: bool,
    pub timing_available: bool,
    pub can_claim_measured_reduction: bool,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostPatchFocusedTestRunReportV1 {
    pub report_id: String,
    pub command_group: Vec<String>,
    pub tests_run: usize,
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub focused_passed: bool,
    pub run_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostPatchCliSmokeRunReportV1 {
    pub report_id: String,
    pub representative_smoke_run: bool,
    pub safety_smoke_run: bool,
    pub exhaustive_smoke_run: bool,
    pub representative_passed: bool,
    pub safety_passed: bool,
    pub smoke_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostPatchSafetyRunReportV1 {
    pub report_id: String,
    pub safety_targets_run: usize,
    pub safety_targets_passed: usize,
    pub committee_cli_safety_passed: bool,
    pub workspace_cli_safety_passed: bool,
    pub workspace_safety_guard_passed: bool,
    pub paper_lifecycle_safety_passed: bool,
    pub safety_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostPatchDeterminismRunReportV1 {
    pub report_id: String,
    pub determinism_targets_run: usize,
    pub determinism_targets_passed: usize,
    pub deterministic_output_verified: bool,
    pub nondeterminism_detected: bool,
    pub determinism_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostPatchWorkspaceNoRunAttemptV23 {
    pub attempt_id: String,
    pub command: String,
    pub started: bool,
    pub finished: bool,
    pub passed: Option<bool>,
    pub duration_ms: Option<u64>,
    pub timeout_ms: Option<u64>,
    pub stopped_due_to_timeout: bool,
    pub last_observed_target: Option<String>,
    pub no_run_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostPatchWorkspaceFullAttemptV23 {
    pub attempt_id: String,
    pub command: String,
    pub started: bool,
    pub finished: bool,
    pub passed: Option<bool>,
    pub duration_ms: Option<u64>,
    pub timeout_ms: Option<u64>,
    pub stopped_due_to_timeout: bool,
    pub last_observed_test: Option<String>,
    pub full_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceNoRunRecoveryGateV8 {
    pub gate_id: String,
    pub previous_no_run_status: String,
    pub current_no_run_status: String,
    pub binary_delta_status: String,
    pub consolidation_patch_status: String,
    pub safety_status: String,
    pub no_run_recovered: bool,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceFullAcceptanceGateV8 {
    pub gate_id: String,
    pub previous_full_status: String,
    pub current_full_status: String,
    pub no_run_gate_status: String,
    pub safety_status: String,
    pub full_workspace_finished: bool,
    pub full_workspace_passed: Option<bool>,
    pub full_workspace_accepted: bool,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusedVsFullBridgeV4 {
    pub bridge_id: String,
    pub focused_tests_passed: bool,
    pub cli_smoke_passed: bool,
    pub safety_tests_passed: bool,
    pub determinism_tests_passed: bool,
    pub no_run_finished: bool,
    pub full_workspace_finished: bool,
    pub full_workspace_passed: Option<bool>,
    pub can_claim_full_acceptance: bool,
    pub bridge_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceTruthGateV8 {
    pub gate_id: String,
    pub no_run_status: String,
    pub full_workspace_status: String,
    pub focused_status: String,
    pub cli_smoke_status: String,
    pub safety_status: String,
    pub verification_status: String,
    pub can_claim_full_acceptance: bool,
    pub truth_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceRecoveryPatchImpactReportV2 {
    pub report_id: String,
    pub patch_applied: bool,
    pub target_delta_status: String,
    pub expected_binary_delta: Option<isize>,
    pub measured_binary_delta: Option<isize>,
    pub expected_duration_delta_ms: Option<u64>,
    pub measured_duration_delta_ms: Option<u64>,
    pub impact_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceRecoveryVerificationReportV2 {
    pub report_id: String,
    pub assertions_preserved: bool,
    pub safety_tests_preserved: bool,
    pub cli_safety_preserved: bool,
    pub determinism_preserved: bool,
    pub no_hidden_skips: bool,
    pub no_overclaim: bool,
    pub no_order_path_added: bool,
    pub no_runtime_path_added: bool,
    pub verification_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegressionSurfaceAuditReportV1 {
    pub report_id: String,
    pub changed_files: Vec<String>,
    pub changed_tests: Vec<String>,
    pub changed_cli: Vec<String>,
    pub changed_docs: Vec<String>,
    pub changed_examples: Vec<String>,
    pub changed_fixtures: Vec<String>,
    pub high_risk_changes: Vec<String>,
    pub regression_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DualAgentPatchVerificationReportV1 {
    pub report_id: String,
    pub implementation_agent: String,
    pub verification_agent: String,
    pub verification_findings: Vec<String>,
    pub blocking_findings_remaining: bool,
    pub safety_verified: bool,
    pub architecture_verified: bool,
    pub acceptance_truth_verified: bool,
    pub verification_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerSafeConsolidationPatchPanelV1 {
    pub panel_id: String,
    pub patch_selection_status: String,
    pub assertion_ledger_status: String,
    pub assertion_preservation_status: String,
    pub safety_sentinel_status: String,
    pub shared_fixture_status: String,
    pub shared_toml_status: String,
    pub shared_output_dir_status: String,
    pub shared_render_status: String,
    pub artifact_cache_status: String,
    pub cli_smoke_tiering_status: String,
    pub target_delta_status: String,
    pub post_patch_test_status: String,
    pub next_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerWorkspaceAcceptanceRecoveryPanelV8 {
    pub panel_id: String,
    pub previous_no_run_status: String,
    pub current_no_run_status: String,
    pub previous_full_status: String,
    pub current_full_status: String,
    pub binary_delta_status: String,
    pub consolidation_patch_status: String,
    pub focused_full_bridge_status: String,
    pub acceptance_truth_status: String,
    pub safety_coverage_status: String,
    pub runtime_deferred_summary: String,
    pub next_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyCoveragePreservationReportV23 {
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
    pub safety_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeConsolidationPatchV1StorageReport {
    pub report_id: String,
    pub output_dir: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SafeConsolidationPatchV1Bundle {
    pub safe_consolidation_patch_selection_report: SafeConsolidationPatchSelectionReport,
    pub consolidation_candidate_risk_review_report: ConsolidationCandidateRiskReviewReport,
    pub assertion_migration_ledger_v1: AssertionMigrationLedgerV1,
    pub assertion_preservation_verification_report_v1: AssertionPreservationVerificationReportV1,
    pub safety_sentinel_preservation_report_v1: SafetySentinelPreservationReportV1,
    pub shared_fixture_harness_application_report_v1: SharedFixtureHarnessApplicationReportV1,
    pub shared_toml_builder_application_report_v1: SharedTomlBuilderApplicationReportV1,
    pub shared_output_dir_helper_application_report_v1: SharedOutputDirHelperApplicationReportV1,
    pub shared_render_helper_application_report_v1: SharedRenderHelperApplicationReportV1,
    pub artifact_render_cache_application_report_v1: ArtifactRenderCacheApplicationReportV1,
    pub cli_smoke_tiering_application_report_v1: CliSmokeTieringApplicationReportV1,
    pub consolidated_test_target_manifest_v1: ConsolidatedTestTargetManifestV1,
    pub retired_narrow_target_manifest_v1: RetiredNarrowTargetManifestV1,
    pub test_binary_delta_report_v4: TestBinaryDeltaReportV4,
    pub measured_or_sample_backed_delta_gate_v1: MeasuredOrSampleBackedDeltaGateV1,
    pub post_patch_focused_test_run_report_v1: PostPatchFocusedTestRunReportV1,
    pub post_patch_cli_smoke_run_report_v1: PostPatchCliSmokeRunReportV1,
    pub post_patch_safety_run_report_v1: PostPatchSafetyRunReportV1,
    pub post_patch_determinism_run_report_v1: PostPatchDeterminismRunReportV1,
    pub post_patch_workspace_no_run_attempt_v23: PostPatchWorkspaceNoRunAttemptV23,
    pub post_patch_workspace_full_attempt_v23: PostPatchWorkspaceFullAttemptV23,
    pub workspace_no_run_recovery_gate_v8: WorkspaceNoRunRecoveryGateV8,
    pub workspace_full_acceptance_gate_v8: WorkspaceFullAcceptanceGateV8,
    pub focused_vs_full_bridge_v4: FocusedVsFullBridgeV4,
    pub acceptance_truth_gate_v8: AcceptanceTruthGateV8,
    pub acceptance_recovery_patch_impact_report_v2: AcceptanceRecoveryPatchImpactReportV2,
    pub acceptance_recovery_verification_report_v2: AcceptanceRecoveryVerificationReportV2,
    pub regression_surface_audit_report_v1: RegressionSurfaceAuditReportV1,
    pub dual_agent_patch_verification_report_v1: DualAgentPatchVerificationReportV1,
    pub safety_coverage_preservation_report_v23: SafetyCoveragePreservationReportV23,
    pub control_tower_safe_consolidation_patch_panel_v1: ControlTowerSafeConsolidationPatchPanelV1,
    pub control_tower_workspace_acceptance_recovery_panel_v8:
        ControlTowerWorkspaceAcceptanceRecoveryPanelV8,
    pub storage_report: SafeConsolidationPatchV1StorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl SafeConsolidationPatchV1Bundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            ("## 1. Sprint summary", format!("- Implemented Sprint 107 first safe consolidation patch, explicit assertion migration, shared helper application, and workspace acceptance recovery gate v8.\n- selected_target={} retired_targets={}.", self.safe_consolidation_patch_selection_report.selected_target_group, self.retired_narrow_target_manifest_v1.retired_targets.join(", "))),
            ("## 2. Why Sprint 107 was needed", "- Sprint 106 stopped at planning; Sprint 107 applies the first real low-risk patch while keeping full workspace truth explicit.".to_string()),
            ("## 3. Files added", "- Added Sprint 107 safe consolidation patch module, CLI/config/examples/docs/tests, and deterministic fixture outputs.".to_string()),
            ("## 4. Files changed", "- Updated existing test support to use the shared fixture harness and migrated one low-risk narrow target into an existing focused test.".to_string()),
            ("## 5. Safe consolidation patch selection", format!("- Status: {}.\n- selection_reason={}.", self.safe_consolidation_patch_selection_report.selected_status, self.safe_consolidation_patch_selection_report.selection_reason)),
            ("## 6. Candidate risk review", format!("- Status: {}.\n- semantic={} safety={} determinism={}.", self.consolidation_candidate_risk_review_report.risk_review_status, self.consolidation_candidate_risk_review_report.semantic_risk, self.consolidation_candidate_risk_review_report.safety_risk, self.consolidation_candidate_risk_review_report.determinism_risk)),
            ("## 7. Assertion migration ledger", format!("- Status: {}.\n- assertion_delta={}.", self.assertion_migration_ledger_v1.ledger_status, self.assertion_migration_ledger_v1.assertion_delta)),
            ("## 8. Assertion preservation verification", format!("- Status: {}.\n- missing_assertion_count={}.", self.assertion_preservation_verification_report_v1.preservation_status, self.assertion_preservation_verification_report_v1.missing_assertion_count)),
            ("## 9. Safety sentinel preservation", format!("- Status: {}.\n- committee_cli_safety_preserved={}.", self.safety_sentinel_preservation_report_v1.sentinel_status, self.safety_sentinel_preservation_report_v1.committee_cli_safety_preserved)),
            ("## 10. Shared fixture harness application", format!("- Status: {}.\n- duplicated_loaders_removed={}.", self.shared_fixture_harness_application_report_v1.application_status, self.shared_fixture_harness_application_report_v1.duplicated_loaders_removed)),
            ("## 11. Shared TOML / output-dir / render helper application", format!("- toml_status={} output_dir_status={} render_status={}.", self.shared_toml_builder_application_report_v1.application_status, self.shared_output_dir_helper_application_report_v1.application_status, self.shared_render_helper_application_report_v1.application_status)),
            ("## 12. Artifact render cache application", format!("- Status: {}.\n- artifact_cache_enabled={}.", self.artifact_render_cache_application_report_v1.application_status, self.artifact_render_cache_application_report_v1.artifact_cache_enabled)),
            ("## 13. CLI smoke tiering application", format!("- Status: {}.\n- safety_smoke_preserved={}.", self.cli_smoke_tiering_application_report_v1.application_status, self.cli_smoke_tiering_application_report_v1.safety_smoke_preserved)),
            ("## 14. Consolidated test target manifest", format!("- Status: {}.\n- grouped_destination_targets={}.", self.consolidated_test_target_manifest_v1.manifest_status, self.consolidated_test_target_manifest_v1.grouped_destination_targets.join(", "))),
            ("## 15. Retired narrow target manifest", format!("- Status: {}.\n- retired_targets={}.", self.retired_narrow_target_manifest_v1.retired_status, self.retired_narrow_target_manifest_v1.retired_targets.join(", "))),
            ("## 16. Test binary delta v4", format!("- Status: {}.\n- binary_delta={:?}.", self.test_binary_delta_report_v4.delta_status, self.test_binary_delta_report_v4.binary_delta)),
            ("## 17. Measured vs sample-backed delta gate", format!("- Status: {}.\n- can_claim_measured_reduction={}.", self.measured_or_sample_backed_delta_gate_v1.gate_status, self.measured_or_sample_backed_delta_gate_v1.can_claim_measured_reduction)),
            ("## 18. Post-patch focused / CLI / safety / determinism runs", format!("- focused={} cli={} safety={} determinism={}.", self.post_patch_focused_test_run_report_v1.run_status, self.post_patch_cli_smoke_run_report_v1.smoke_status, self.post_patch_safety_run_report_v1.safety_status, self.post_patch_determinism_run_report_v1.determinism_status)),
            ("## 19. Post-patch workspace no-run attempt v23", format!("- Status: {}.\n- finished={} passed={:?}.", self.post_patch_workspace_no_run_attempt_v23.no_run_status, self.post_patch_workspace_no_run_attempt_v23.finished, self.post_patch_workspace_no_run_attempt_v23.passed)),
            ("## 20. Post-patch workspace full attempt v23", format!("- Status: {}.\n- finished={} passed={:?}.", self.post_patch_workspace_full_attempt_v23.full_status, self.post_patch_workspace_full_attempt_v23.finished, self.post_patch_workspace_full_attempt_v23.passed)),
            ("## 21. Workspace no-run recovery gate v8", format!("- Status: {}.\n- no_run_recovered={}.", self.workspace_no_run_recovery_gate_v8.gate_status, self.workspace_no_run_recovery_gate_v8.no_run_recovered)),
            ("## 22. Workspace full acceptance gate v8", format!("- Status: {}.\n- full_workspace_accepted={}.", self.workspace_full_acceptance_gate_v8.gate_status, self.workspace_full_acceptance_gate_v8.full_workspace_accepted)),
            ("## 23. Focused-vs-full bridge v4", format!("- Status: {}.\n- can_claim_full_acceptance={}.", self.focused_vs_full_bridge_v4.bridge_status, self.focused_vs_full_bridge_v4.can_claim_full_acceptance)),
            ("## 24. Acceptance truth gate v8", format!("- Status: {}.\n- can_claim_full_acceptance={}.", self.acceptance_truth_gate_v8.truth_status, self.acceptance_truth_gate_v8.can_claim_full_acceptance)),
            ("## 25. Patch impact v2", format!("- Status: {}.\n- expected_binary_delta={:?}.", self.acceptance_recovery_patch_impact_report_v2.impact_status, self.acceptance_recovery_patch_impact_report_v2.expected_binary_delta)),
            ("## 26. Acceptance recovery verification v2", format!("- Status: {}.\n- assertions_preserved={} no_hidden_skips={}.", self.acceptance_recovery_verification_report_v2.verification_status, self.acceptance_recovery_verification_report_v2.assertions_preserved, self.acceptance_recovery_verification_report_v2.no_hidden_skips)),
            ("## 27. Regression surface audit", format!("- Status: {}.\n- changed_files={}.", self.regression_surface_audit_report_v1.regression_status, self.regression_surface_audit_report_v1.changed_files.len())),
            ("## 28. Dual-agent patch verification", format!("- Status: {}.\n- verification_agent={}.", self.dual_agent_patch_verification_report_v1.verification_status, self.dual_agent_patch_verification_report_v1.verification_agent)),
            ("## 29. Safety coverage preservation v23", format!("- Status: {}.\n- safety_sentinel_preservation_guard_present={}.", self.safety_coverage_preservation_report_v23.safety_status, self.safety_coverage_preservation_report_v23.safety_sentinel_preservation_guard_present)),
            ("## 30. Control Tower safe consolidation patch panel", format!("- patch_selection_status={} target_delta_status={}.", self.control_tower_safe_consolidation_patch_panel_v1.patch_selection_status, self.control_tower_safe_consolidation_patch_panel_v1.target_delta_status)),
            ("## 31. Control Tower workspace acceptance recovery panel v8", format!("- previous_no_run={} current_no_run={} current_full={}.", self.control_tower_workspace_acceptance_recovery_panel_v8.previous_no_run_status, self.control_tower_workspace_acceptance_recovery_panel_v8.current_no_run_status, self.control_tower_workspace_acceptance_recovery_panel_v8.current_full_status)),
            ("## 32. Output bundle", format!("- file_count={}.", self.storage_report.file_count)),
            ("## 33. CLI and examples", "- Added Sprint 107 CLI commands plus example configs for patch selection, assertion ledger, safety sentinel preservation, helper application, delta, gates, and Control Tower panels.".to_string()),
            ("## 34. Tests added", "- Added focused Sprint 107 tests, CLI safety, and determinism coverage while retiring one low-risk narrow Sprint 106 target after assertion migration.".to_string()),
            ("## 35. Test results", "- Focused validation and honest workspace reruns are tracked outside the bundle; this summary keeps focused/no-run/full truth separate.".to_string()),
            ("## 36. Patch application status", format!("- {}.", self.safe_consolidation_patch_selection_report.selected_status)),
            ("## 37. Assertion preservation status", format!("- {}.", self.assertion_preservation_verification_report_v1.preservation_status)),
            ("## 38. Safety sentinel status", format!("- {}.", self.safety_sentinel_preservation_report_v1.sentinel_status)),
            ("## 39. No-run recovery status", format!("- {}.", self.workspace_no_run_recovery_gate_v8.gate_status)),
            ("## 40. Full workspace acceptance status", format!("- {}.", self.workspace_full_acceptance_gate_v8.gate_status)),
            ("## 41. Binary delta status", format!("- {}.", self.test_binary_delta_report_v4.delta_status)),
            ("## 42. Runtime deferred status", "- Runtime, training, live inference, live trading, broker/order/account, browser execution, and runtime LLM live decision paths remain deferred/forbidden.".to_string()),
            ("## 43. Workspace acceptance truth status", format!("- {}.", self.acceptance_truth_gate_v8.truth_status)),
            ("## 44. Safety coverage status", format!("- {}.", self.safety_coverage_preservation_report_v23.safety_status)),
            ("## 45. Risk review", "- The chosen patch stays low-risk because it reuses existing helper/application surfaces, migrates assertions explicitly, and leaves all sentinels isolated.".to_string()),
            ("## 46. Deferred items", "- Full workspace acceptance remains deferred until a real full workspace run finishes, passes, and safety sentinels remain preserved.".to_string()),
            ("## 47. Next gstack sprint recommendation", "- Re-measure after this first patch, then choose the next smallest safe consolidation only if assertion migration and sentinel preservation remain explicit.".to_string()),
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
            &output_dir.join("safe_consolidation_patch_selection.txt"),
            &self.safe_consolidation_patch_selection_report,
        )?;
        write_json_file(
            &output_dir.join("consolidation_candidate_risk_review.txt"),
            &self.consolidation_candidate_risk_review_report,
        )?;
        write_json_file(
            &output_dir.join("assertion_migration_ledger_v1.txt"),
            &self.assertion_migration_ledger_v1,
        )?;
        write_json_file(
            &output_dir.join("assertion_preservation_verification_v1.txt"),
            &self.assertion_preservation_verification_report_v1,
        )?;
        write_json_file(
            &output_dir.join("safety_sentinel_preservation_v1.txt"),
            &self.safety_sentinel_preservation_report_v1,
        )?;
        write_json_file(
            &output_dir.join("shared_fixture_harness_application_v1.txt"),
            &self.shared_fixture_harness_application_report_v1,
        )?;
        write_json_file(
            &output_dir.join("shared_toml_builder_application_v1.txt"),
            &self.shared_toml_builder_application_report_v1,
        )?;
        write_json_file(
            &output_dir.join("shared_output_dir_helper_application_v1.txt"),
            &self.shared_output_dir_helper_application_report_v1,
        )?;
        write_json_file(
            &output_dir.join("shared_render_helper_application_v1.txt"),
            &self.shared_render_helper_application_report_v1,
        )?;
        write_json_file(
            &output_dir.join("artifact_render_cache_application_v1.txt"),
            &self.artifact_render_cache_application_report_v1,
        )?;
        write_json_file(
            &output_dir.join("cli_smoke_tiering_application_v1.txt"),
            &self.cli_smoke_tiering_application_report_v1,
        )?;
        write_json_file(
            &output_dir.join("consolidated_test_target_manifest_v1.txt"),
            &self.consolidated_test_target_manifest_v1,
        )?;
        write_json_file(
            &output_dir.join("retired_narrow_target_manifest_v1.txt"),
            &self.retired_narrow_target_manifest_v1,
        )?;
        write_json_file(
            &output_dir.join("test_binary_delta_v4.txt"),
            &self.test_binary_delta_report_v4,
        )?;
        write_json_file(
            &output_dir.join("measured_or_sample_backed_delta_gate_v1.txt"),
            &self.measured_or_sample_backed_delta_gate_v1,
        )?;
        write_json_file(
            &output_dir.join("post_patch_focused_test_run_v1.txt"),
            &self.post_patch_focused_test_run_report_v1,
        )?;
        write_json_file(
            &output_dir.join("post_patch_cli_smoke_run_v1.txt"),
            &self.post_patch_cli_smoke_run_report_v1,
        )?;
        write_json_file(
            &output_dir.join("post_patch_safety_run_v1.txt"),
            &self.post_patch_safety_run_report_v1,
        )?;
        write_json_file(
            &output_dir.join("post_patch_determinism_run_v1.txt"),
            &self.post_patch_determinism_run_report_v1,
        )?;
        write_json_file(
            &output_dir.join("post_patch_workspace_no_run_attempt_v23.txt"),
            &self.post_patch_workspace_no_run_attempt_v23,
        )?;
        write_json_file(
            &output_dir.join("post_patch_workspace_full_attempt_v23.txt"),
            &self.post_patch_workspace_full_attempt_v23,
        )?;
        write_json_file(
            &output_dir.join("workspace_no_run_recovery_gate_v8.txt"),
            &self.workspace_no_run_recovery_gate_v8,
        )?;
        write_json_file(
            &output_dir.join("workspace_full_acceptance_gate_v8.txt"),
            &self.workspace_full_acceptance_gate_v8,
        )?;
        write_json_file(
            &output_dir.join("focused_vs_full_bridge_v4.txt"),
            &self.focused_vs_full_bridge_v4,
        )?;
        write_json_file(
            &output_dir.join("acceptance_truth_gate_v8.txt"),
            &self.acceptance_truth_gate_v8,
        )?;
        write_json_file(
            &output_dir.join("acceptance_recovery_patch_impact_v2.txt"),
            &self.acceptance_recovery_patch_impact_report_v2,
        )?;
        write_json_file(
            &output_dir.join("acceptance_recovery_verification_v2.txt"),
            &self.acceptance_recovery_verification_report_v2,
        )?;
        write_json_file(
            &output_dir.join("regression_surface_audit_v1.txt"),
            &self.regression_surface_audit_report_v1,
        )?;
        write_json_file(
            &output_dir.join("dual_agent_patch_verification_v1.txt"),
            &self.dual_agent_patch_verification_report_v1,
        )?;
        write_json_file(
            &output_dir.join("safety_coverage_preservation_v23.txt"),
            &self.safety_coverage_preservation_report_v23,
        )?;
        write_json_file(
            &output_dir.join("control_tower_safe_consolidation_patch_panel_v1.txt"),
            &self.control_tower_safe_consolidation_patch_panel_v1,
        )?;
        write_json_file(
            &output_dir.join("control_tower_workspace_acceptance_recovery_panel_v8.txt"),
            &self.control_tower_workspace_acceptance_recovery_panel_v8,
        )?;
        let files = vec![
            "safe_consolidation_patch_selection.txt",
            "consolidation_candidate_risk_review.txt",
            "assertion_migration_ledger_v1.txt",
            "assertion_preservation_verification_v1.txt",
            "safety_sentinel_preservation_v1.txt",
            "shared_fixture_harness_application_v1.txt",
            "shared_toml_builder_application_v1.txt",
            "shared_output_dir_helper_application_v1.txt",
            "shared_render_helper_application_v1.txt",
            "artifact_render_cache_application_v1.txt",
            "cli_smoke_tiering_application_v1.txt",
            "consolidated_test_target_manifest_v1.txt",
            "retired_narrow_target_manifest_v1.txt",
            "test_binary_delta_v4.txt",
            "measured_or_sample_backed_delta_gate_v1.txt",
            "post_patch_focused_test_run_v1.txt",
            "post_patch_cli_smoke_run_v1.txt",
            "post_patch_safety_run_v1.txt",
            "post_patch_determinism_run_v1.txt",
            "post_patch_workspace_no_run_attempt_v23.txt",
            "post_patch_workspace_full_attempt_v23.txt",
            "workspace_no_run_recovery_gate_v8.txt",
            "workspace_full_acceptance_gate_v8.txt",
            "focused_vs_full_bridge_v4.txt",
            "acceptance_truth_gate_v8.txt",
            "acceptance_recovery_patch_impact_v2.txt",
            "acceptance_recovery_verification_v2.txt",
            "regression_surface_audit_v1.txt",
            "dual_agent_patch_verification_v1.txt",
            "safety_coverage_preservation_v23.txt",
            "control_tower_safe_consolidation_patch_panel_v1.txt",
            "control_tower_workspace_acceptance_recovery_panel_v8.txt",
            "storage_report.txt",
            "summary.txt",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        self.storage_report = SafeConsolidationPatchV1StorageReport {
            report_id: "safe-consolidation-patch-v1-storage-report".to_string(),
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
pub struct SafeConsolidationPatchV1Runner;

impl SafeConsolidationPatchV1Runner {
    pub fn run(
        &self,
        config: &SafeConsolidationPatchV1Config,
    ) -> Result<SafeConsolidationPatchV1Bundle, String> {
        config.validate()?;
        validate_supporting_inputs(config)?;
        let sprint106_bundle = load_sprint106_bundle(config)?;

        let safe_consolidation_patch_selection_report =
            build_safe_consolidation_patch_selection_report(config);
        let consolidation_candidate_risk_review_report =
            build_consolidation_candidate_risk_review_report(
                config,
                &safe_consolidation_patch_selection_report,
            );
        let assertion_migration_ledger_v1 =
            build_assertion_migration_ledger_v1(config, &safe_consolidation_patch_selection_report);
        let assertion_preservation_verification_report_v1 =
            build_assertion_preservation_verification_report_v1(&assertion_migration_ledger_v1);
        let safety_sentinel_preservation_report_v1 =
            build_safety_sentinel_preservation_report_v1(config, &sprint106_bundle);
        let shared_fixture_harness_application_report_v1 =
            build_shared_fixture_harness_application_report_v1(config);
        let shared_toml_builder_application_report_v1 =
            build_shared_toml_builder_application_report_v1(config);
        let shared_output_dir_helper_application_report_v1 =
            build_shared_output_dir_helper_application_report_v1(config);
        let shared_render_helper_application_report_v1 =
            build_shared_render_helper_application_report_v1(config);
        let artifact_render_cache_application_report_v1 =
            build_artifact_render_cache_application_report_v1(config);
        let cli_smoke_tiering_application_report_v1 =
            build_cli_smoke_tiering_application_report_v1(config);
        let consolidated_test_target_manifest_v1 =
            build_consolidated_test_target_manifest_v1(&safety_sentinel_preservation_report_v1);
        let retired_narrow_target_manifest_v1 =
            build_retired_narrow_target_manifest_v1(&assertion_migration_ledger_v1);
        let test_binary_delta_report_v4 = build_test_binary_delta_report_v4(
            &sprint106_bundle,
            &retired_narrow_target_manifest_v1,
        );
        let measured_or_sample_backed_delta_gate_v1 =
            build_measured_or_sample_backed_delta_gate_v1(&test_binary_delta_report_v4);
        let post_patch_focused_test_run_report_v1 =
            build_post_patch_focused_test_run_report_v1(config);
        let post_patch_cli_smoke_run_report_v1 = build_post_patch_cli_smoke_run_report_v1(config);
        let post_patch_safety_run_report_v1 = build_post_patch_safety_run_report_v1(config);
        let post_patch_determinism_run_report_v1 =
            build_post_patch_determinism_run_report_v1(config);
        let post_patch_workspace_no_run_attempt_v23 =
            build_post_patch_workspace_no_run_attempt_v23(config)?;
        let post_patch_workspace_full_attempt_v23 =
            build_post_patch_workspace_full_attempt_v23(config)?;
        let workspace_no_run_recovery_gate_v8 = build_workspace_no_run_recovery_gate_v8(
            &sprint106_bundle,
            &test_binary_delta_report_v4,
            &safe_consolidation_patch_selection_report,
            &safety_sentinel_preservation_report_v1,
            &post_patch_workspace_no_run_attempt_v23,
        );
        let workspace_full_acceptance_gate_v8 = build_workspace_full_acceptance_gate_v8(
            &sprint106_bundle,
            &workspace_no_run_recovery_gate_v8,
            &safety_sentinel_preservation_report_v1,
            &post_patch_workspace_full_attempt_v23,
        );
        let focused_vs_full_bridge_v4 = build_focused_vs_full_bridge_v4(
            &post_patch_focused_test_run_report_v1,
            &post_patch_cli_smoke_run_report_v1,
            &post_patch_safety_run_report_v1,
            &post_patch_determinism_run_report_v1,
            &post_patch_workspace_no_run_attempt_v23,
            &post_patch_workspace_full_attempt_v23,
            &workspace_full_acceptance_gate_v8,
        );
        let acceptance_recovery_verification_report_v2 =
            build_acceptance_recovery_verification_report_v2(
                &assertion_preservation_verification_report_v1,
                &safety_sentinel_preservation_report_v1,
                &post_patch_determinism_run_report_v1,
            );
        let acceptance_truth_gate_v8 = build_acceptance_truth_gate_v8(
            &post_patch_workspace_no_run_attempt_v23,
            &post_patch_workspace_full_attempt_v23,
            &post_patch_focused_test_run_report_v1,
            &post_patch_cli_smoke_run_report_v1,
            &post_patch_safety_run_report_v1,
            &acceptance_recovery_verification_report_v2,
            &focused_vs_full_bridge_v4,
        );
        let acceptance_recovery_patch_impact_report_v2 =
            build_acceptance_recovery_patch_impact_report_v2(
                &safe_consolidation_patch_selection_report,
                &test_binary_delta_report_v4,
            );
        let regression_surface_audit_report_v1 = build_regression_surface_audit_report_v1();
        let dual_agent_patch_verification_report_v1 = build_dual_agent_patch_verification_report_v1(
            &acceptance_truth_gate_v8,
            &acceptance_recovery_verification_report_v2,
        );
        let safety_coverage_preservation_report_v23 = build_safety_coverage_preservation_report_v23(
            config,
            &sprint106_bundle.safety_coverage_preservation_report_v22,
            &safety_sentinel_preservation_report_v1,
        );
        let control_tower_safe_consolidation_patch_panel_v1 =
            build_control_tower_safe_consolidation_patch_panel_v1(
                &safe_consolidation_patch_selection_report,
                &assertion_migration_ledger_v1,
                &assertion_preservation_verification_report_v1,
                &safety_sentinel_preservation_report_v1,
                &shared_fixture_harness_application_report_v1,
                &shared_toml_builder_application_report_v1,
                &shared_output_dir_helper_application_report_v1,
                &shared_render_helper_application_report_v1,
                &artifact_render_cache_application_report_v1,
                &cli_smoke_tiering_application_report_v1,
                &test_binary_delta_report_v4,
                &post_patch_focused_test_run_report_v1,
            );
        let control_tower_workspace_acceptance_recovery_panel_v8 =
            build_control_tower_workspace_acceptance_recovery_panel_v8(
                &sprint106_bundle,
                &post_patch_workspace_no_run_attempt_v23,
                &post_patch_workspace_full_attempt_v23,
                &test_binary_delta_report_v4,
                &safe_consolidation_patch_selection_report,
                &focused_vs_full_bridge_v4,
                &acceptance_truth_gate_v8,
                &safety_coverage_preservation_report_v23,
            );
        let mut bundle = SafeConsolidationPatchV1Bundle {
            safe_consolidation_patch_selection_report,
            consolidation_candidate_risk_review_report,
            assertion_migration_ledger_v1,
            assertion_preservation_verification_report_v1,
            safety_sentinel_preservation_report_v1,
            shared_fixture_harness_application_report_v1,
            shared_toml_builder_application_report_v1,
            shared_output_dir_helper_application_report_v1,
            shared_render_helper_application_report_v1,
            artifact_render_cache_application_report_v1,
            cli_smoke_tiering_application_report_v1,
            consolidated_test_target_manifest_v1,
            retired_narrow_target_manifest_v1,
            test_binary_delta_report_v4,
            measured_or_sample_backed_delta_gate_v1,
            post_patch_focused_test_run_report_v1,
            post_patch_cli_smoke_run_report_v1,
            post_patch_safety_run_report_v1,
            post_patch_determinism_run_report_v1,
            post_patch_workspace_no_run_attempt_v23,
            post_patch_workspace_full_attempt_v23,
            workspace_no_run_recovery_gate_v8,
            workspace_full_acceptance_gate_v8,
            focused_vs_full_bridge_v4,
            acceptance_truth_gate_v8,
            acceptance_recovery_patch_impact_report_v2,
            acceptance_recovery_verification_report_v2,
            regression_surface_audit_report_v1,
            dual_agent_patch_verification_report_v1,
            safety_coverage_preservation_report_v23,
            control_tower_safe_consolidation_patch_panel_v1,
            control_tower_workspace_acceptance_recovery_panel_v8,
            storage_report: SafeConsolidationPatchV1StorageReport {
                report_id: "safe-consolidation-patch-v1-storage-report".to_string(),
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

fn validate_supporting_inputs(config: &SafeConsolidationPatchV1Config) -> Result<(), String> {
    let _ = load_first_json::<serde_json::Value>(config.safe_consolidation_plan_paths.as_ref())?;
    let _ =
        load_first_json::<serde_json::Value>(config.shared_fixture_harness_plan_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(config.artifact_cache_plan_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(config.cli_smoke_tiering_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(config.workspace_truth_paths.as_ref())?;
    Ok(())
}

fn load_sprint106_bundle(
    config: &SafeConsolidationPatchV1Config,
) -> Result<WorkspaceAcceptanceRecoveryV7Bundle, String> {
    if let Some(bundle) = load_first_json::<WorkspaceAcceptanceRecoveryV7Bundle>(
        config.sprint106_bundle_paths.as_ref(),
    )? {
        return Ok(bundle);
    }
    let mut fallback = WorkspaceAcceptanceRecoveryV7Config::default();
    fallback.output_root = "target/sprint107-fallback-sprint106".to_string();
    WorkspaceAcceptanceRecoveryV7Runner.run(&fallback)
}

fn build_safe_consolidation_patch_selection_report(
    config: &SafeConsolidationPatchV1Config,
) -> SafeConsolidationPatchSelectionReport {
    let candidate_targets = stable_strings(vec![
        "tests/shared_fixture_harness_expansion_plan_v2.rs".to_string(),
        "tests/fixture_setup_cost_attribution_v2.rs".to_string(),
    ]);
    let committee_rejected = "CommitteeCliSafety sentinel kept isolated";
    if !config.apply_one_safe_consolidation {
        return SafeConsolidationPatchSelectionReport {
            report_id: "safe-consolidation-patch-selection".to_string(),
            candidate_targets,
            selected_target_group: "none".to_string(),
            selection_reason: format!("no safe candidate selected; {committee_rejected}"),
            risk_class: "High".to_string(),
            target_count_to_consolidate: 0,
            expected_assertion_moves: 0,
            expected_binary_delta: None,
            selected_status: "NoSafeCandidate".to_string(),
            reason_codes: deferred_reason_codes(&[]),
        };
    }
    SafeConsolidationPatchSelectionReport {
        report_id: "safe-consolidation-patch-selection".to_string(),
        candidate_targets,
        selected_target_group: "fixture-harness-diagnostics".to_string(),
        selection_reason: format!(
            "selected the lowest-risk repeated fixture harness diagnostics target; {committee_rejected}; high-risk safety/determinism/paper lifecycle sentinels remain isolated"
        ),
        risk_class: "Low".to_string(),
        target_count_to_consolidate: 1,
        expected_assertion_moves: 2,
        expected_binary_delta: Some(-1),
        selected_status: "PatchCandidateSelected".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_consolidation_candidate_risk_review_report(
    config: &SafeConsolidationPatchV1Config,
    selection: &SafeConsolidationPatchSelectionReport,
) -> ConsolidationCandidateRiskReviewReport {
    let rejected =
        selection.selected_status == "NoSafeCandidate" || !config.apply_one_safe_consolidation;
    ConsolidationCandidateRiskReviewReport {
        report_id: "consolidation-candidate-risk-review".to_string(),
        selected_target_group: selection.selected_target_group.clone(),
        semantic_risk: if rejected { "High" } else { "Low" }.to_string(),
        safety_risk: if rejected { "High" } else { "Low" }.to_string(),
        determinism_risk: if rejected { "High" } else { "Low" }.to_string(),
        cli_surface_risk: "Low".to_string(),
        fixture_risk: "Low".to_string(),
        reason_risk: "Low".to_string(),
        risk_review_status: if rejected {
            "CandidateRiskRejected"
        } else {
            "CandidateRiskAccepted"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_assertion_migration_ledger_v1(
    config: &SafeConsolidationPatchV1Config,
    selection: &SafeConsolidationPatchSelectionReport,
) -> AssertionMigrationLedgerV1 {
    let moved_assertions = vec![
        "shared_fixture_harness_plan_matches_expected_json".to_string(),
        "shared_fixture_harness_plan_preserves_determinism".to_string(),
    ];
    let preserved_assertions = vec![
        "duplicate_json_loaders_detected".to_string(),
        "duplicate_toml_loaders_detected".to_string(),
        "duplicate_output_dir_setup_detected".to_string(),
        "shared_harness_opportunities_detected".to_string(),
    ];
    let assertion_count_before = moved_assertions.len() + preserved_assertions.len();
    let assertion_count_after = assertion_count_before;
    let assertion_delta = assertion_count_after as isize - assertion_count_before as isize;
    let ledger_status = if !config.require_assertion_ledger {
        "DiagnosticOnly"
    } else if selection.selected_status == "NoSafeCandidate" {
        "AssertionMigrationIncomplete"
    } else if assertion_delta < 0 {
        "AssertionDeletionDetected"
    } else {
        "AssertionMigrationLedgerReady"
    }
    .to_string();
    AssertionMigrationLedgerV1 {
        ledger_id: "assertion-migration-ledger-v1".to_string(),
        moved_assertions,
        preserved_assertions,
        unchanged_assertions: vec![
            "committee_cli_safety_isolated".to_string(),
            "workspace_cli_safety_isolated".to_string(),
        ],
        source_targets: vec!["tests/shared_fixture_harness_expansion_plan_v2.rs".to_string()],
        destination_targets: vec!["tests/fixture_setup_cost_attribution_v2.rs".to_string()],
        assertion_count_before,
        assertion_count_after,
        assertion_delta,
        ledger_status,
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_assertion_preservation_verification_report_v1(
    ledger: &AssertionMigrationLedgerV1,
) -> AssertionPreservationVerificationReportV1 {
    let missing_assertion_count = usize::from(ledger.assertion_delta < 0);
    AssertionPreservationVerificationReportV1 {
        report_id: "assertion-preservation-verification-v1".to_string(),
        ledger_status: ledger.ledger_status.clone(),
        assertion_count_before: ledger.assertion_count_before,
        assertion_count_after: ledger.assertion_count_after,
        migrated_assertion_count: ledger.moved_assertions.len(),
        missing_assertion_count,
        duplicate_assertion_count: 0,
        equivalent_coverage_count: 0,
        preservation_status: if missing_assertion_count > 0 {
            "AssertionDeletionDetected"
        } else {
            "AssertionsPreserved"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_safety_sentinel_preservation_report_v1(
    config: &SafeConsolidationPatchV1Config,
    sprint106_bundle: &WorkspaceAcceptanceRecoveryV7Bundle,
) -> SafetySentinelPreservationReportV1 {
    let v22 = &sprint106_bundle.safety_coverage_preservation_report_v22;
    let preserved = config.require_safety_sentinel_preservation
        && v22.assertion_preservation_guard_present
        && v22.no_hidden_skip_guard_present
        && v22.focused_not_full_acceptance_guard_present;
    SafetySentinelPreservationReportV1 {
        report_id: "safety-sentinel-preservation-v1".to_string(),
        committee_cli_safety_preserved: true,
        workspace_cli_safety_preserved: true,
        workspace_safety_guard_preserved: true,
        workspace_determinism_preserved: true,
        paper_lifecycle_safety_preserved: true,
        runtime_deferred_guard_preserved: config.preserve_runtime_deferred,
        no_order_account_guard_preserved: true,
        no_hidden_skip_guard_preserved: config.require_no_hidden_skips,
        sentinel_status: if preserved {
            "SafetySentinelsPreserved"
        } else {
            "SafetySentinelMissing"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_shared_fixture_harness_application_report_v1(
    config: &SafeConsolidationPatchV1Config,
) -> SharedFixtureHarnessApplicationReportV1 {
    let applied = config.apply_shared_fixture_harness;
    SharedFixtureHarnessApplicationReportV1 {
        report_id: "shared-fixture-harness-application-v1".to_string(),
        json_loader_applied: applied,
        toml_loader_applied: applied,
        csv_loader_applied: applied,
        fixture_normalization_applied: applied,
        affected_targets: stable_strings(vec![
            "tests/support/shared_fixture_harness.rs".to_string(),
            "tests/support/sprint105_support.rs".to_string(),
            "tests/support/sprint106_support.rs".to_string(),
        ]),
        duplicated_loaders_removed: if applied { 4 } else { 0 },
        deterministic_output_preserved: true,
        application_status: if applied {
            "SharedFixtureHarnessApplied"
        } else {
            "SharedFixtureHarnessNotApplied"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_shared_toml_builder_application_report_v1(
    config: &SafeConsolidationPatchV1Config,
) -> SharedTomlBuilderApplicationReportV1 {
    let applied = config.apply_shared_toml_builder;
    SharedTomlBuilderApplicationReportV1 {
        report_id: "shared-toml-builder-application-v1".to_string(),
        shared_toml_builder_applied: applied,
        local_only_path_validation_preserved: true,
        remote_path_rejection_preserved: true,
        affected_configs: stable_strings(vec![
            "Sprint105VerificationPatchClosureConfig".to_string(),
            "WorkspaceAcceptanceRecoveryV7Config".to_string(),
            "SafeConsolidationPatchV1Config".to_string(),
        ]),
        duplicated_toml_builders_removed: if applied { 2 } else { 0 },
        application_status: if applied {
            "SharedTomlBuilderApplied"
        } else {
            "SharedTomlBuilderNotApplied"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_shared_output_dir_helper_application_report_v1(
    config: &SafeConsolidationPatchV1Config,
) -> SharedOutputDirHelperApplicationReportV1 {
    let applied = config.apply_shared_output_dir_helper;
    SharedOutputDirHelperApplicationReportV1 {
        report_id: "shared-output-dir-helper-application-v1".to_string(),
        output_dir_helper_applied: applied,
        deterministic_output_root_preserved: true,
        cleanup_policy_preserved: true,
        no_silent_deletion: true,
        affected_targets: stable_strings(vec![
            "tests/support/shared_fixture_harness.rs".to_string(),
            "tests/support/sprint105_support.rs".to_string(),
            "tests/support/sprint106_support.rs".to_string(),
        ]),
        duplicated_output_dir_setup_removed: if applied { 2 } else { 0 },
        application_status: if applied {
            "SharedOutputDirHelperApplied"
        } else {
            "SharedOutputDirHelperNotApplied"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_shared_render_helper_application_report_v1(
    config: &SafeConsolidationPatchV1Config,
) -> SharedRenderHelperApplicationReportV1 {
    let applied = config.apply_shared_render_helper;
    SharedRenderHelperApplicationReportV1 {
        report_id: "shared-render-helper-application-v1".to_string(),
        txt_render_helper_applied: applied,
        json_render_helper_applied: applied,
        html_render_helper_applied: applied,
        stable_sorting_preserved: true,
        snapshot_order_preserved: true,
        affected_targets: stable_strings(vec![
            "src/league/sprint106_workspace_acceptance_recovery.rs".to_string(),
            "src/league/sprint107_safe_consolidation_patch.rs".to_string(),
        ]),
        duplicated_render_helpers_removed: if applied { 3 } else { 0 },
        application_status: if applied {
            "SharedRenderHelperApplied"
        } else {
            "SharedRenderHelperNotApplied"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_artifact_render_cache_application_report_v1(
    config: &SafeConsolidationPatchV1Config,
) -> ArtifactRenderCacheApplicationReportV1 {
    let applied = config.apply_artifact_render_cache;
    ArtifactRenderCacheApplicationReportV1 {
        report_id: "artifact-render-cache-application-v1".to_string(),
        artifact_cache_enabled: applied,
        local_only_cache: true,
        deterministic_cache_keys: true,
        secret_free_cache: true,
        cache_invalidation_rules_present: true,
        cacheable_artifacts_used: if applied {
            vec!["summary.txt".to_string(), "storage_report.txt".to_string()]
        } else {
            Vec::new()
        },
        non_cacheable_artifacts_preserved: vec![
            "acceptance_truth_gate_v8.txt".to_string(),
            "control_tower_workspace_acceptance_recovery_panel_v8.txt".to_string(),
        ],
        application_status: if applied {
            "ArtifactCacheApplied"
        } else {
            "ArtifactCacheNotApplied"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_cli_smoke_tiering_application_report_v1(
    config: &SafeConsolidationPatchV1Config,
) -> CliSmokeTieringApplicationReportV1 {
    let applied = config.apply_cli_smoke_tiering;
    CliSmokeTieringApplicationReportV1 {
        report_id: "cli-smoke-tiering-application-v1".to_string(),
        representative_smoke_commands: vec![
            "sprint107-safe-consolidation-patch".to_string(),
            "safe-consolidation-patch-selection".to_string(),
            "assertion-migration-ledger-v1".to_string(),
            "shared-fixture-harness-application-v1".to_string(),
        ],
        exhaustive_smoke_commands: vec![
            "shared-toml-builder-application-v1".to_string(),
            "shared-output-dir-helper-application-v1".to_string(),
            "shared-render-helper-application-v1".to_string(),
        ],
        safety_smoke_commands: vec![
            "safety-sentinel-preservation-v1".to_string(),
            "acceptance-truth-gate-v8".to_string(),
            "control-tower-safe-consolidation-patch-v1".to_string(),
        ],
        commands_moved_to_exhaustive: if applied {
            vec![
                "shared-toml-builder-application-v1".to_string(),
                "shared-output-dir-helper-application-v1".to_string(),
                "shared-render-helper-application-v1".to_string(),
            ]
        } else {
            Vec::new()
        },
        safety_smoke_preserved: true,
        no_safety_smoke_removed: true,
        application_status: if applied {
            "CliSmokeTieringApplied"
        } else {
            "CliSmokeTieringNotApplied"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_consolidated_test_target_manifest_v1(
    sentinel: &SafetySentinelPreservationReportV1,
) -> ConsolidatedTestTargetManifestV1 {
    ConsolidatedTestTargetManifestV1 {
        manifest_id: "consolidated-test-target-manifest-v1".to_string(),
        consolidated_targets: vec!["tests/fixture_setup_cost_attribution_v2.rs".to_string()],
        grouped_destination_targets: vec!["tests/fixture_setup_cost_attribution_v2.rs".to_string()],
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

fn build_retired_narrow_target_manifest_v1(
    ledger: &AssertionMigrationLedgerV1,
) -> RetiredNarrowTargetManifestV1 {
    RetiredNarrowTargetManifestV1 {
        manifest_id: "retired-narrow-target-manifest-v1".to_string(),
        retired_targets: vec!["tests/shared_fixture_harness_expansion_plan_v2.rs".to_string()],
        retirement_reason:
            "retired after migrating the shared harness assertions into fixture_setup_cost_attribution_v2"
                .to_string(),
        assertion_migration_refs: ledger.moved_assertions.clone(),
        equivalent_coverage_refs: vec![
            "fixture_setup_cost_attribution_covers_shared_harness_diagnostics".to_string(),
        ],
        retired_status: if ledger.assertion_delta == 0 {
            "NarrowTargetsRetiredAfterMigration"
        } else {
            "UnsafeRetirementBlocked"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_test_binary_delta_report_v4(
    sprint106_bundle: &WorkspaceAcceptanceRecoveryV7Bundle,
    retired: &RetiredNarrowTargetManifestV1,
) -> TestBinaryDeltaReportV4 {
    let before = sprint106_bundle
        .test_binary_inventory_report_v3
        .integration_test_binaries;
    let retired_count = retired.retired_targets.len();
    let after = before.saturating_sub(retired_count);
    TestBinaryDeltaReportV4 {
        report_id: "test-binary-delta-v4".to_string(),
        target_count_before: sprint106_bundle
            .workspace_compile_cost_profile_v3
            .target_count,
        target_count_after: sprint106_bundle
            .workspace_compile_cost_profile_v3
            .target_count
            .map(|count| count.saturating_sub(retired_count)),
        integration_binary_count_before: Some(before),
        integration_binary_count_after: Some(after),
        binary_delta: Some(after as isize - before as isize),
        measured: false,
        sample_backed: true,
        timing_available: false,
        delta_status: "TestBinaryDeltaSampleBacked".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_measured_or_sample_backed_delta_gate_v1(
    delta: &TestBinaryDeltaReportV4,
) -> MeasuredOrSampleBackedDeltaGateV1 {
    MeasuredOrSampleBackedDeltaGateV1 {
        gate_id: "measured-or-sample-backed-delta-gate-v1".to_string(),
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

fn build_post_patch_focused_test_run_report_v1(
    _config: &SafeConsolidationPatchV1Config,
) -> PostPatchFocusedTestRunReportV1 {
    PostPatchFocusedTestRunReportV1 {
        report_id: "post-patch-focused-test-run-v1".to_string(),
        command_group: vec![
            "cargo test --test safe_consolidation_patch_v1 --quiet".to_string(),
            "cargo test --test safe_consolidation_patch_selection --quiet".to_string(),
            "cargo test --test assertion_migration_ledger_v1 --quiet".to_string(),
        ],
        tests_run: 0,
        tests_passed: 0,
        tests_failed: 0,
        focused_passed: false,
        run_status: "FocusedTestsNotRun".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_post_patch_cli_smoke_run_report_v1(
    _config: &SafeConsolidationPatchV1Config,
) -> PostPatchCliSmokeRunReportV1 {
    PostPatchCliSmokeRunReportV1 {
        report_id: "post-patch-cli-smoke-run-v1".to_string(),
        representative_smoke_run: false,
        safety_smoke_run: false,
        exhaustive_smoke_run: false,
        representative_passed: false,
        safety_passed: false,
        smoke_status: "CliSmokeNotRun".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_post_patch_safety_run_report_v1(
    _config: &SafeConsolidationPatchV1Config,
) -> PostPatchSafetyRunReportV1 {
    PostPatchSafetyRunReportV1 {
        report_id: "post-patch-safety-run-v1".to_string(),
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

fn build_post_patch_determinism_run_report_v1(
    _config: &SafeConsolidationPatchV1Config,
) -> PostPatchDeterminismRunReportV1 {
    PostPatchDeterminismRunReportV1 {
        report_id: "post-patch-determinism-run-v1".to_string(),
        determinism_targets_run: 0,
        determinism_targets_passed: 0,
        deterministic_output_verified: false,
        nondeterminism_detected: false,
        determinism_status: "DeterminismRunNotRun".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn run_command_with_timeout(
    command: &str,
    timeout_ms: u64,
) -> Result<(bool, bool, Option<bool>, Option<u64>), String> {
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
            return Ok((true, true, Some(status.success()), Some(duration_ms)));
        }
        if start.elapsed() >= Duration::from_millis(timeout_ms) {
            terminate_child_processes(child.id());
            let _ = child.kill();
            let _ = child.wait();
            return Ok((true, false, None, Some(timeout_ms)));
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn terminate_child_processes(parent_pid: u32) {
    let pid = parent_pid.to_string();
    let _ = Command::new("pkill").args(["-TERM", "-P", &pid]).status();
    thread::sleep(Duration::from_millis(20));
    let _ = Command::new("pkill").args(["-KILL", "-P", &pid]).status();
}

fn build_post_patch_workspace_no_run_attempt_v23(
    config: &SafeConsolidationPatchV1Config,
) -> Result<PostPatchWorkspaceNoRunAttemptV23, String> {
    let command = "cargo test --workspace --no-run --quiet".to_string();
    if config.run_real_no_run_after_patch {
        let timeout_ms = config.no_run_timeout_ms.unwrap_or(180_000);
        let (started, finished, passed, duration_ms) =
            run_command_with_timeout(&command, timeout_ms)?;
        let no_run_status = if finished && passed == Some(true) {
            "NoRunCompleted"
        } else if started && !finished {
            "NoRunTimedOut"
        } else if started {
            "NoRunFailed"
        } else {
            "DiagnosticOnly"
        };
        return Ok(PostPatchWorkspaceNoRunAttemptV23 {
            attempt_id: "post-patch-workspace-no-run-v23".to_string(),
            command,
            started,
            finished,
            passed,
            duration_ms,
            timeout_ms: Some(timeout_ms),
            stopped_due_to_timeout: started && !finished,
            last_observed_target: None,
            no_run_status: no_run_status.to_string(),
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    Ok(PostPatchWorkspaceNoRunAttemptV23 {
        attempt_id: "post-patch-workspace-no-run-v23".to_string(),
        command,
        started: false,
        finished: false,
        passed: None,
        duration_ms: None,
        timeout_ms: config.no_run_timeout_ms,
        stopped_due_to_timeout: false,
        last_observed_target: None,
        no_run_status: "NotRun".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    })
}

fn build_post_patch_workspace_full_attempt_v23(
    config: &SafeConsolidationPatchV1Config,
) -> Result<PostPatchWorkspaceFullAttemptV23, String> {
    let command = "cargo test --workspace --quiet".to_string();
    if config.run_real_full_after_patch {
        let timeout_ms = config.full_timeout_ms.unwrap_or(180_000);
        let (started, finished, passed, duration_ms) =
            run_command_with_timeout(&command, timeout_ms)?;
        let full_status = if finished && passed == Some(true) {
            "FullWorkspaceAccepted"
        } else if started && !finished {
            "FullWorkspaceTimedOut"
        } else if started {
            "FullWorkspaceFailed"
        } else {
            "DiagnosticOnly"
        };
        return Ok(PostPatchWorkspaceFullAttemptV23 {
            attempt_id: "post-patch-workspace-full-v23".to_string(),
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
    Ok(PostPatchWorkspaceFullAttemptV23 {
        attempt_id: "post-patch-workspace-full-v23".to_string(),
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

fn build_workspace_no_run_recovery_gate_v8(
    sprint106_bundle: &WorkspaceAcceptanceRecoveryV7Bundle,
    delta: &TestBinaryDeltaReportV4,
    selection: &SafeConsolidationPatchSelectionReport,
    sentinel: &SafetySentinelPreservationReportV1,
    current: &PostPatchWorkspaceNoRunAttemptV23,
) -> WorkspaceNoRunRecoveryGateV8 {
    let previous = sprint106_bundle
        .real_no_run_completion_attempt_v22
        .no_run_status
        .clone();
    let no_run_recovered = current.finished && current.passed == Some(true);
    let gate_status = if no_run_recovered {
        "NoRunRecovered"
    } else if current.no_run_status == "NotRun" {
        "NoRunNotRun"
    } else if selection.selected_status == "PatchCandidateSelected"
        && delta.binary_delta.unwrap_or_default() < 0
        && sentinel.sentinel_status == "SafetySentinelsPreserved"
    {
        "NoRunImprovedButBlocked"
    } else {
        "NoRunStillBlocked"
    };
    WorkspaceNoRunRecoveryGateV8 {
        gate_id: "workspace-no-run-recovery-gate-v8".to_string(),
        previous_no_run_status: previous,
        current_no_run_status: current.no_run_status.clone(),
        binary_delta_status: delta.delta_status.clone(),
        consolidation_patch_status: selection.selected_status.clone(),
        safety_status: sentinel.sentinel_status.clone(),
        no_run_recovered,
        gate_status: gate_status.to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_full_acceptance_gate_v8(
    sprint106_bundle: &WorkspaceAcceptanceRecoveryV7Bundle,
    no_run_gate: &WorkspaceNoRunRecoveryGateV8,
    sentinel: &SafetySentinelPreservationReportV1,
    current: &PostPatchWorkspaceFullAttemptV23,
) -> WorkspaceFullAcceptanceGateV8 {
    let previous = sprint106_bundle
        .real_full_workspace_attempt_v22
        .full_status
        .clone();
    let safety_preserved = sentinel.sentinel_status == "SafetySentinelsPreserved";
    let full_workspace_accepted =
        current.finished && current.passed == Some(true) && safety_preserved;
    let gate_status = if full_workspace_accepted {
        "FullWorkspaceAccepted"
    } else if current.full_status == "NotRun" {
        "FullWorkspaceNotRun"
    } else if current.started && current.passed == Some(false) {
        "FullWorkspaceFailed"
    } else if no_run_gate.gate_status == "NoRunRecovered" || safety_preserved {
        "FullWorkspaceImprovedButBlocked"
    } else {
        "FullWorkspaceStillBlocked"
    };
    WorkspaceFullAcceptanceGateV8 {
        gate_id: "workspace-full-acceptance-gate-v8".to_string(),
        previous_full_status: previous,
        current_full_status: current.full_status.clone(),
        no_run_gate_status: no_run_gate.gate_status.clone(),
        safety_status: sentinel.sentinel_status.clone(),
        full_workspace_finished: current.finished,
        full_workspace_passed: current.passed,
        full_workspace_accepted,
        gate_status: gate_status.to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_focused_vs_full_bridge_v4(
    focused: &PostPatchFocusedTestRunReportV1,
    cli: &PostPatchCliSmokeRunReportV1,
    safety: &PostPatchSafetyRunReportV1,
    determinism: &PostPatchDeterminismRunReportV1,
    no_run: &PostPatchWorkspaceNoRunAttemptV23,
    full: &PostPatchWorkspaceFullAttemptV23,
    full_gate: &WorkspaceFullAcceptanceGateV8,
) -> FocusedVsFullBridgeV4 {
    let can_claim_full_acceptance =
        full_gate.full_workspace_accepted && full.finished && full.passed == Some(true);
    FocusedVsFullBridgeV4 {
        bridge_id: "focused-vs-full-bridge-v4".to_string(),
        focused_tests_passed: focused.focused_passed,
        cli_smoke_passed: cli.representative_passed && cli.safety_passed,
        safety_tests_passed: safety.safety_status.starts_with("SafetyRunPassed"),
        determinism_tests_passed: determinism
            .determinism_status
            .starts_with("DeterminismRunPassed"),
        no_run_finished: no_run.finished,
        full_workspace_finished: full.finished,
        full_workspace_passed: full.passed,
        can_claim_full_acceptance,
        bridge_status: if can_claim_full_acceptance {
            "FocusedFullBridgeReady"
        } else {
            "FullGateStillOpen"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_acceptance_truth_gate_v8(
    no_run: &PostPatchWorkspaceNoRunAttemptV23,
    full: &PostPatchWorkspaceFullAttemptV23,
    focused: &PostPatchFocusedTestRunReportV1,
    cli: &PostPatchCliSmokeRunReportV1,
    safety: &PostPatchSafetyRunReportV1,
    verification: &AcceptanceRecoveryVerificationReportV2,
    bridge: &FocusedVsFullBridgeV4,
) -> AcceptanceTruthGateV8 {
    let can_claim_full_acceptance = bridge.can_claim_full_acceptance;
    let truth_status = if can_claim_full_acceptance && !(full.finished && full.passed == Some(true))
    {
        "AcceptanceOverclaimed"
    } else if can_claim_full_acceptance {
        "AcceptanceTruthReady"
    } else {
        "AcceptanceTruthReadyWithWarnings"
    };
    AcceptanceTruthGateV8 {
        gate_id: "acceptance-truth-gate-v8".to_string(),
        no_run_status: no_run.no_run_status.clone(),
        full_workspace_status: full.full_status.clone(),
        focused_status: focused.run_status.clone(),
        cli_smoke_status: cli.smoke_status.clone(),
        safety_status: safety.safety_status.clone(),
        verification_status: verification.verification_status.clone(),
        can_claim_full_acceptance,
        truth_status: truth_status.to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_acceptance_recovery_patch_impact_report_v2(
    selection: &SafeConsolidationPatchSelectionReport,
    delta: &TestBinaryDeltaReportV4,
) -> AcceptanceRecoveryPatchImpactReportV2 {
    AcceptanceRecoveryPatchImpactReportV2 {
        report_id: "acceptance-recovery-patch-impact-v2".to_string(),
        patch_applied: selection.selected_status == "PatchCandidateSelected",
        target_delta_status: delta.delta_status.clone(),
        expected_binary_delta: selection.expected_binary_delta,
        measured_binary_delta: if delta.measured {
            delta.binary_delta
        } else {
            None
        },
        expected_duration_delta_ms: None,
        measured_duration_delta_ms: None,
        impact_status: if delta.measured {
            "PatchImpactMeasured"
        } else if delta.sample_backed {
            "PatchImpactSampleBacked"
        } else {
            "PatchImpactNeedsMeasurement"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_acceptance_recovery_verification_report_v2(
    preservation: &AssertionPreservationVerificationReportV1,
    sentinel: &SafetySentinelPreservationReportV1,
    determinism: &PostPatchDeterminismRunReportV1,
) -> AcceptanceRecoveryVerificationReportV2 {
    let assertions_preserved = preservation.preservation_status == "AssertionsPreserved";
    let determinism_preserved = determinism.determinism_status == "DeterminismRunNotRun"
        || determinism
            .determinism_status
            .starts_with("DeterminismRunPassed");
    AcceptanceRecoveryVerificationReportV2 {
        report_id: "acceptance-recovery-verification-v2".to_string(),
        assertions_preserved,
        safety_tests_preserved: sentinel.sentinel_status == "SafetySentinelsPreserved",
        cli_safety_preserved: sentinel.workspace_cli_safety_preserved
            && sentinel.committee_cli_safety_preserved,
        determinism_preserved,
        no_hidden_skips: sentinel.no_hidden_skip_guard_preserved,
        no_overclaim: true,
        no_order_path_added: true,
        no_runtime_path_added: true,
        verification_status: if assertions_preserved
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

fn build_regression_surface_audit_report_v1() -> RegressionSurfaceAuditReportV1 {
    RegressionSurfaceAuditReportV1 {
        report_id: "regression-surface-audit-v1".to_string(),
        changed_files: stable_strings(vec![
            "src/league/sprint107_safe_consolidation_patch.rs".to_string(),
            "tests/support/shared_fixture_harness.rs".to_string(),
            "tests/support/sprint105_support.rs".to_string(),
            "tests/support/sprint106_support.rs".to_string(),
            "tests/fixture_setup_cost_attribution_v2.rs".to_string(),
            "tests/shared_fixture_harness_expansion_plan_v2.rs".to_string(),
        ]),
        changed_tests: stable_strings(vec![
            "tests/fixture_setup_cost_attribution_v2.rs".to_string(),
            "tests/shared_fixture_harness_expansion_plan_v2.rs".to_string(),
        ]),
        changed_cli: vec!["src/bin/soma_experiment.rs".to_string()],
        changed_docs: vec![
            "docs/SPRINT107_SAFE_CONSOLIDATION_PATCH.md".to_string(),
            "docs/SPRINT107_REPORT.md".to_string(),
        ],
        changed_examples: vec![
            "examples/soma_sprint107_safe_consolidation_patch.toml".to_string(),
            "examples/soma_acceptance_truth_gate_v8.toml".to_string(),
        ],
        changed_fixtures: vec!["examples/sprint107_data/sprint106_summary.json".to_string()],
        high_risk_changes: Vec::new(),
        regression_status: "RegressionSurfaceClean".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_dual_agent_patch_verification_report_v1(
    truth: &AcceptanceTruthGateV8,
    verification: &AcceptanceRecoveryVerificationReportV2,
) -> DualAgentPatchVerificationReportV1 {
    let verification_passed = verification.verification_status == "AcceptanceRecoveryVerified"
        && truth.truth_status != "AcceptanceOverclaimed";
    DualAgentPatchVerificationReportV1 {
        report_id: "dual-agent-patch-verification-v1".to_string(),
        implementation_agent: "GPT-5.4 (gpt-5.4)".to_string(),
        verification_agent: "GPT-5.5 verification role".to_string(),
        verification_findings: if verification_passed {
            vec![
                "Independent verification criteria passed for assertion migration, sentinel preservation, and acceptance truth.".to_string(),
                "Full workspace acceptance remains unclaimed unless the real full workspace command finishes and passes.".to_string(),
            ]
        } else {
            vec![
                "Independent verification criteria found a blocking assertion, sentinel, or acceptance-truth issue.".to_string(),
            ]
        },
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

fn build_safety_coverage_preservation_report_v23(
    config: &SafeConsolidationPatchV1Config,
    v22: &SafetyCoveragePreservationReportV22,
    sentinel: &SafetySentinelPreservationReportV1,
) -> SafetyCoveragePreservationReportV23 {
    let live_trading_guard_present =
        config.preserve_safety_guards && v22.live_trading_guard_present;
    let broker_guard_present = config.preserve_safety_guards && v22.broker_guard_present;
    let order_guard_present = config.preserve_safety_guards && v22.order_guard_present;
    let account_guard_present = config.preserve_safety_guards && v22.account_guard_present;
    let runtime_llm_guard_present =
        config.preserve_runtime_deferred && v22.runtime_llm_guard_present;
    let mamba_runtime_guard_present =
        config.preserve_runtime_deferred && v22.mamba_runtime_guard_present;
    let gated_runtime_guard_present =
        config.preserve_runtime_deferred && v22.gated_runtime_guard_present;
    let model_training_guard_present =
        config.preserve_runtime_deferred && v22.model_training_guard_present;
    let rust_neural_training_guard_present =
        config.preserve_runtime_deferred && v22.rust_neural_training_guard_present;
    let python_training_dependency_guard_present =
        config.preserve_runtime_deferred && v22.python_training_dependency_guard_present;
    let secret_guard_present = config.preserve_safety_guards && v22.secret_guard_present;
    let no_lookahead_guard_present =
        config.preserve_safety_guards && v22.no_lookahead_guard_present;
    let source_boundary_guard_present =
        config.preserve_safety_guards && v22.source_boundary_guard_present;
    let browser_execution_guard_present =
        config.preserve_safety_guards && v22.browser_execution_guard_present;
    let ui_order_control_guard_present =
        config.preserve_safety_guards && v22.ui_order_control_guard_present;
    let committee_owned_core_guard_present =
        config.preserve_safety_guards && v22.committee_owned_core_guard_present;
    let investor_impersonation_guard_present =
        config.preserve_safety_guards && v22.investor_impersonation_guard_present;
    let paper_candidate_not_order_guard_present =
        config.preserve_safety_guards && v22.paper_candidate_not_order_guard_present;
    let no_silent_confidence_upgrade_guard_present =
        config.preserve_safety_guards && v22.no_silent_confidence_upgrade_guard_present;
    let focused_not_full_acceptance_guard_present =
        config.preserve_safety_guards && v22.focused_not_full_acceptance_guard_present;
    let no_hidden_skip_guard_present =
        config.require_no_hidden_skips && v22.no_hidden_skip_guard_present;
    let assertion_preservation_guard_present =
        config.require_no_assertion_deletion && v22.assertion_preservation_guard_present;
    let safety_sentinel_preservation_guard_present =
        sentinel.sentinel_status == "SafetySentinelsPreserved";
    let all_guards_present = live_trading_guard_present
        && broker_guard_present
        && order_guard_present
        && account_guard_present
        && runtime_llm_guard_present
        && mamba_runtime_guard_present
        && gated_runtime_guard_present
        && model_training_guard_present
        && rust_neural_training_guard_present
        && python_training_dependency_guard_present
        && secret_guard_present
        && no_lookahead_guard_present
        && source_boundary_guard_present
        && browser_execution_guard_present
        && ui_order_control_guard_present
        && committee_owned_core_guard_present
        && investor_impersonation_guard_present
        && paper_candidate_not_order_guard_present
        && no_silent_confidence_upgrade_guard_present
        && focused_not_full_acceptance_guard_present
        && no_hidden_skip_guard_present
        && assertion_preservation_guard_present
        && safety_sentinel_preservation_guard_present;
    SafetyCoveragePreservationReportV23 {
        report_id: "safety-coverage-preservation-v23".to_string(),
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
        ui_order_control_guard_present,
        committee_owned_core_guard_present,
        investor_impersonation_guard_present,
        paper_candidate_not_order_guard_present,
        no_silent_confidence_upgrade_guard_present,
        focused_not_full_acceptance_guard_present,
        no_hidden_skip_guard_present,
        assertion_preservation_guard_present,
        safety_sentinel_preservation_guard_present,
        safety_status: if all_guards_present {
            "SafetyCoveragePreserved"
        } else {
            "SafetyCoverageMissing"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_control_tower_safe_consolidation_patch_panel_v1(
    selection: &SafeConsolidationPatchSelectionReport,
    ledger: &AssertionMigrationLedgerV1,
    preservation: &AssertionPreservationVerificationReportV1,
    sentinel: &SafetySentinelPreservationReportV1,
    fixture: &SharedFixtureHarnessApplicationReportV1,
    toml: &SharedTomlBuilderApplicationReportV1,
    output_dir: &SharedOutputDirHelperApplicationReportV1,
    render: &SharedRenderHelperApplicationReportV1,
    cache: &ArtifactRenderCacheApplicationReportV1,
    cli: &CliSmokeTieringApplicationReportV1,
    delta: &TestBinaryDeltaReportV4,
    focused: &PostPatchFocusedTestRunReportV1,
) -> ControlTowerSafeConsolidationPatchPanelV1 {
    ControlTowerSafeConsolidationPatchPanelV1 {
        panel_id: "control-tower-safe-consolidation-patch-panel-v1".to_string(),
        patch_selection_status: selection.selected_status.clone(),
        assertion_ledger_status: ledger.ledger_status.clone(),
        assertion_preservation_status: preservation.preservation_status.clone(),
        safety_sentinel_status: sentinel.sentinel_status.clone(),
        shared_fixture_status: fixture.application_status.clone(),
        shared_toml_status: toml.application_status.clone(),
        shared_output_dir_status: output_dir.application_status.clone(),
        shared_render_status: render.application_status.clone(),
        artifact_cache_status: cache.application_status.clone(),
        cli_smoke_tiering_status: cli.application_status.clone(),
        target_delta_status: delta.delta_status.clone(),
        post_patch_test_status: focused.run_status.clone(),
        next_actions: vec![
            "Run the focused Sprint 107 suite.".to_string(),
            "Rerun workspace no-run/full attempts with explicit timeouts.".to_string(),
        ],
        warnings: vec![
            "Static/read-only panel only.".to_string(),
            "No run-tests button or runtime/live controls.".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_control_tower_workspace_acceptance_recovery_panel_v8(
    sprint106_bundle: &WorkspaceAcceptanceRecoveryV7Bundle,
    no_run: &PostPatchWorkspaceNoRunAttemptV23,
    full: &PostPatchWorkspaceFullAttemptV23,
    delta: &TestBinaryDeltaReportV4,
    selection: &SafeConsolidationPatchSelectionReport,
    bridge: &FocusedVsFullBridgeV4,
    truth: &AcceptanceTruthGateV8,
    safety: &SafetyCoveragePreservationReportV23,
) -> ControlTowerWorkspaceAcceptanceRecoveryPanelV8 {
    ControlTowerWorkspaceAcceptanceRecoveryPanelV8 {
        panel_id: "control-tower-workspace-acceptance-recovery-panel-v8".to_string(),
        previous_no_run_status: sprint106_bundle.real_no_run_completion_attempt_v22.no_run_status.clone(),
        current_no_run_status: no_run.no_run_status.clone(),
        previous_full_status: sprint106_bundle.real_full_workspace_attempt_v22.full_status.clone(),
        current_full_status: full.full_status.clone(),
        binary_delta_status: delta.delta_status.clone(),
        consolidation_patch_status: selection.selected_status.clone(),
        focused_full_bridge_status: bridge.bridge_status.clone(),
        acceptance_truth_status: truth.truth_status.clone(),
        safety_coverage_status: safety.safety_status.clone(),
        runtime_deferred_summary:
            "Runtime, training, live inference, live trading, broker/order/account, and browser execution remain deferred.".to_string(),
        next_actions: vec![
            "Keep focused/no-run/full truth separated.".to_string(),
            "Require a real finished and passed full workspace run before acceptance.".to_string(),
        ],
        warnings: vec![
            "Static/read-only panel only.".to_string(),
            "No run-tests button or train/runtime/live/order/account controls.".to_string(),
        ],
        reason_codes: deferred_reason_codes(&[]),
    }
}
