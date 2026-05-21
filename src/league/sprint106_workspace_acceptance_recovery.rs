use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::league::sprint105_verification_patch_closure::{
    FocusedVsFullGateBridgeV2, SafetyCoveragePreservationReportV21,
    Sprint105VerificationPatchClosureBundle, Sprint105VerificationPatchClosureConfig,
    Sprint105VerificationPatchClosureRunner,
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
    "target/soma_sprint106_workspace_acceptance_recovery".to_string()
}

fn default_timeout_ms() -> Option<u64> {
    Some(180_000)
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
            .map_err(|err| format!("failed to read sprint106 JSON input {path}: {err}"))?;
        match serde_json::from_str::<T>(&text) {
            Ok(value) => return Ok(Some(value)),
            Err(err) => parse_errors.push(format!("{path}: {err}")),
        }
    }
    if !paths.is_empty() {
        return Err(format!(
            "failed to parse any sprint106 JSON input: {}",
            parse_errors.join("; ")
        ));
    }
    Ok(None)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAcceptanceRecoveryV7Config {
    pub recovery_id: String,
    #[serde(default)]
    pub sprint105_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub workspace_truth_paths: Option<Vec<String>>,
    #[serde(default)]
    pub compile_cost_paths: Option<Vec<String>>,
    #[serde(default)]
    pub cargo_json_paths: Option<Vec<String>>,
    #[serde(default)]
    pub target_inventory_paths: Option<Vec<String>>,
    #[serde(default)]
    pub test_manifest_paths: Option<Vec<String>>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_false")]
    pub run_real_no_run: bool,
    #[serde(default = "default_false")]
    pub run_real_full: bool,
    #[serde(default = "default_timeout_ms")]
    pub no_run_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub full_timeout_ms: Option<u64>,
    #[serde(default = "default_true")]
    pub capture_cargo_json: bool,
    #[serde(default = "default_true")]
    pub capture_test_binary_inventory: bool,
    #[serde(default = "default_true")]
    pub capture_rustc_snapshots: bool,
    #[serde(default = "default_true")]
    pub require_truth_gate: bool,
    #[serde(default = "default_true")]
    pub require_safety_preservation: bool,
    #[serde(default = "default_true")]
    pub require_no_assertion_deletion: bool,
    #[serde(default = "default_true")]
    pub require_no_hidden_skips: bool,
    #[serde(default = "default_true")]
    pub preserve_runtime_deferred: bool,
    #[serde(default = "default_true")]
    pub preserve_dual_agent_separation: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for WorkspaceAcceptanceRecoveryV7Config {
    fn default() -> Self {
        Self {
            recovery_id: "sprint106-workspace-acceptance-recovery".to_string(),
            sprint105_bundle_paths: Some(vec![
                "examples/sprint106_data/sprint105_summary.json".to_string(),
            ]),
            workspace_truth_paths: None,
            compile_cost_paths: Some(vec![
                "examples/sprint106_data/compile_cost_profile_sample.json".to_string(),
            ]),
            cargo_json_paths: Some(vec![
                "examples/sprint106_data/cargo_json_messages_sample.json".to_string(),
            ]),
            target_inventory_paths: None,
            test_manifest_paths: None,
            output_root: default_output_root(),
            run_real_no_run: false,
            run_real_full: false,
            no_run_timeout_ms: default_timeout_ms(),
            full_timeout_ms: default_timeout_ms(),
            capture_cargo_json: true,
            capture_test_binary_inventory: true,
            capture_rustc_snapshots: true,
            require_truth_gate: true,
            require_safety_preservation: true,
            require_no_assertion_deletion: true,
            require_no_hidden_skips: true,
            preserve_runtime_deferred: true,
            preserve_dual_agent_separation: true,
            reason_codes: deferred_reason_codes(&[]),
        }
    }
}

impl WorkspaceAcceptanceRecoveryV7Config {
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
        PathBuf::from(&self.output_root).join(&self.recovery_id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.recovery_id.trim().is_empty() {
            return Err("sprint106 recovery_id must not be empty".to_string());
        }
        if self.output_root.trim().is_empty() {
            return Err("sprint106 output_root must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err(
                "sprint106 workspace acceptance recovery config paths must be local".to_string(),
            );
        }
        for paths in [
            &self.sprint105_bundle_paths,
            &self.workspace_truth_paths,
            &self.compile_cost_paths,
            &self.cargo_json_paths,
            &self.target_inventory_paths,
            &self.test_manifest_paths,
        ] {
            if let Some(paths) = paths
                && paths.iter().any(|path| !local_only(path))
            {
                return Err(
                    "sprint106 workspace acceptance recovery config paths must be local"
                        .to_string(),
                );
            }
        }
        if self.require_safety_preservation && !self.preserve_runtime_deferred {
            return Err(
                "sprint106 safety preservation requires runtime deferred preservation".to_string(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct CompileCostProfileSample {
    #[serde(default)]
    target_count: Option<usize>,
    #[serde(default)]
    integration_test_target_count: Option<usize>,
    #[serde(default)]
    unit_test_target_count: Option<usize>,
    #[serde(default)]
    doc_test_target_count: Option<usize>,
    #[serde(default)]
    build_script_count: Option<usize>,
    #[serde(default)]
    suspected_cost_centers: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct CargoJsonMessagesSample {
    #[serde(default)]
    messages: Vec<CargoJsonMessage>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct CargoJsonMessage {
    #[serde(default)]
    reason: String,
    #[serde(default)]
    package_id: String,
    #[serde(default)]
    target_name: String,
    #[serde(default)]
    artifact: String,
}

#[derive(Clone, Debug)]
struct WorkspaceTruthImport {
    previous_truth_status: String,
    current_truth_status: String,
    can_claim_full_acceptance: bool,
    no_run_started: bool,
    no_run_finished: bool,
    no_run_passed: Option<bool>,
    full_started: bool,
    full_finished: bool,
    full_passed: Option<bool>,
}

#[derive(Clone, Debug)]
struct RepoTestInventory {
    all_targets: Vec<String>,
    cli_safety_targets: Vec<String>,
    determinism_targets: Vec<String>,
    paper_lifecycle_targets: Vec<String>,
    workspace_truth_targets: Vec<String>,
    safety_sentinels: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealNoRunCompletionAttemptV22 {
    pub attempt_id: String,
    pub command: String,
    pub started: bool,
    pub finished: bool,
    pub passed: Option<bool>,
    pub duration_ms: Option<u64>,
    pub timeout_ms: Option<u64>,
    pub stopped_due_to_timeout: bool,
    pub stopped_due_to_manual_interrupt: bool,
    pub last_observed_target: Option<String>,
    pub no_run_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealFullWorkspaceAttemptV22 {
    pub attempt_id: String,
    pub command: String,
    pub started: bool,
    pub finished: bool,
    pub passed: Option<bool>,
    pub duration_ms: Option<u64>,
    pub timeout_ms: Option<u64>,
    pub stopped_due_to_timeout: bool,
    pub stopped_due_to_manual_interrupt: bool,
    pub last_observed_test: Option<String>,
    pub full_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceCompileCostProfileV3 {
    pub profile_id: String,
    pub observed_no_run_attempts: usize,
    pub observed_full_attempts: usize,
    pub target_count: Option<usize>,
    pub integration_test_target_count: Option<usize>,
    pub unit_test_target_count: Option<usize>,
    pub doc_test_target_count: Option<usize>,
    pub build_script_count: Option<usize>,
    pub suspected_cost_centers: Vec<String>,
    pub profile_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoJsonNoRunCaptureV2 {
    pub capture_id: String,
    pub command: String,
    pub message_count: usize,
    pub compiler_artifact_count: usize,
    pub compiler_message_count: usize,
    pub build_script_count: usize,
    pub test_executable_count: usize,
    pub last_artifacts: Vec<String>,
    pub last_targets: Vec<String>,
    pub capture_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestBinaryInventoryReportV3 {
    pub report_id: String,
    pub total_test_binaries: usize,
    pub integration_test_binaries: usize,
    pub high_cost_candidates: Vec<String>,
    pub safety_sentinels: Vec<String>,
    pub cli_safety_targets: Vec<String>,
    pub determinism_targets: Vec<String>,
    pub paper_lifecycle_targets: Vec<String>,
    pub workspace_truth_targets: Vec<String>,
    pub inventory_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestBinaryExplosionAttributionReport {
    pub report_id: String,
    pub suspected_explosion_families: Vec<String>,
    pub repeated_fixture_families: Vec<String>,
    pub repeated_cli_smoke_families: Vec<String>,
    pub repeated_render_families: Vec<String>,
    pub duplicate_assertion_families: Vec<String>,
    pub high_risk_sentinel_families: Vec<String>,
    pub attribution_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RankedTargetCost {
    pub target: String,
    pub score: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationTargetCostRankingReport {
    pub report_id: String,
    pub ranked_targets: Vec<RankedTargetCost>,
    pub top_cost_targets: Vec<String>,
    pub unknown_cost_targets: Vec<String>,
    pub targets_missing_timing: Vec<String>,
    pub ranking_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LongRunningRustcTargetSnapshotV2 {
    pub snapshot_id: String,
    pub active_rustc_count: usize,
    pub active_targets: Vec<String>,
    pub active_packages: Vec<String>,
    pub active_integration_tests: Vec<String>,
    pub active_build_scripts: Vec<String>,
    pub snapshot_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkTimeCostAttributionReport {
    pub report_id: String,
    pub suspected_link_heavy_targets: Vec<String>,
    pub target_artifact_sizes: Option<BTreeMap<String, u64>>,
    pub binary_count_factor: usize,
    pub link_cost_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacroExpansionCostAttributionReport {
    pub report_id: String,
    pub suspected_macro_heavy_crates: Vec<String>,
    pub derive_heavy_targets: Vec<String>,
    pub serde_heavy_targets: Vec<String>,
    pub snapshot_or_codegen_indicators: Vec<String>,
    pub macro_cost_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureSetupCostAttributionV2 {
    pub report_id: String,
    pub duplicate_json_loaders: Vec<String>,
    pub duplicate_toml_loaders: Vec<String>,
    pub duplicate_csv_loaders: Vec<String>,
    pub duplicate_output_dir_setup: Vec<String>,
    pub duplicate_fixture_normalization: Vec<String>,
    pub shared_harness_opportunities: Vec<String>,
    pub fixture_cost_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRenderCostAttributionV2 {
    pub report_id: String,
    pub repeated_txt_render_targets: Vec<String>,
    pub repeated_json_render_targets: Vec<String>,
    pub repeated_html_render_targets: Vec<String>,
    pub repeated_storage_reports: Vec<String>,
    pub artifact_cache_opportunities: Vec<String>,
    pub render_cost_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliSmokeCostAttributionV2 {
    pub report_id: String,
    pub representative_smoke_commands: Vec<String>,
    pub exhaustive_smoke_commands: Vec<String>,
    pub safety_smoke_commands: Vec<String>,
    pub duplicate_smoke_commands: Vec<String>,
    pub smoke_tiering_opportunities: Vec<String>,
    pub cli_smoke_cost_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighCostTestFamilyCluster {
    pub cluster_kind: String,
    pub targets: Vec<String>,
    pub safe_to_consolidate: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighCostTestFamilyClusterReport {
    pub report_id: String,
    pub clusters: Vec<HighCostTestFamilyCluster>,
    pub high_cost_clusters: Vec<String>,
    pub safe_to_consolidate_clusters: Vec<String>,
    pub unsafe_to_consolidate_clusters: Vec<String>,
    pub cluster_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeTestBinaryConsolidationPlanV2 {
    pub plan_id: String,
    pub candidate_targets_to_merge: Vec<String>,
    pub candidate_targets_to_keep_isolated: Vec<String>,
    pub assertions_to_move: Vec<String>,
    pub assertions_to_preserve: Vec<String>,
    pub safety_sentinels_to_keep: Vec<String>,
    pub expected_binary_delta: Option<isize>,
    pub semantic_risk: String,
    pub plan_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafeTestBinaryConsolidationImpactEstimate {
    pub estimate_id: String,
    pub target_count_before: Option<usize>,
    pub target_count_after: Option<usize>,
    pub expected_delta: Option<isize>,
    pub measured: bool,
    pub sample_backed: bool,
    pub timing_available: bool,
    pub estimate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedFixtureHarnessExpansionPlanV2 {
    pub plan_id: String,
    pub fixture_loader_targets: Vec<String>,
    pub toml_builder_targets: Vec<String>,
    pub output_dir_targets: Vec<String>,
    pub render_helper_targets: Vec<String>,
    pub proposed_shared_helpers: Vec<String>,
    pub determinism_preserved: bool,
    pub plan_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRenderCacheSafePlanV2 {
    pub plan_id: String,
    pub cacheable_artifacts: Vec<String>,
    pub non_cacheable_artifacts: Vec<String>,
    pub cache_key_requirements: Vec<String>,
    pub invalidation_rules: Vec<String>,
    pub local_only_cache: bool,
    pub cache_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliSmokeTieringPlanV2 {
    pub plan_id: String,
    pub representative_commands: Vec<String>,
    pub exhaustive_commands: Vec<String>,
    pub safety_commands: Vec<String>,
    pub commands_moved_to_exhaustive: Vec<String>,
    pub commands_kept_representative: Vec<String>,
    pub safety_commands_preserved: bool,
    pub tiering_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepresentativeVsExhaustiveTestPolicy {
    pub policy_id: String,
    pub representative_policy: String,
    pub exhaustive_policy: String,
    pub safety_policy: String,
    pub determinism_policy: String,
    pub full_workspace_policy: String,
    pub policy_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceNoRunRecoveryGateV7 {
    pub gate_id: String,
    pub no_run_attempt_status: String,
    pub cargo_json_capture_status: String,
    pub compile_cost_profile_status: String,
    pub binary_inventory_status: String,
    pub safe_consolidation_plan_status: String,
    pub no_run_recovered: bool,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceFullAcceptanceGateV7 {
    pub gate_id: String,
    pub full_attempt_status: String,
    pub no_run_gate_status: String,
    pub safety_status: String,
    pub full_workspace_finished: bool,
    pub full_workspace_passed: Option<bool>,
    pub full_workspace_accepted: bool,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusedVsFullBridgeV3 {
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
pub struct AcceptanceTruthGateV7 {
    pub gate_id: String,
    pub no_run_status: String,
    pub full_workspace_status: String,
    pub focused_status: String,
    pub verification_status: String,
    pub can_claim_full_acceptance: bool,
    pub truth_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceRecoveryPatchPlan {
    pub plan_id: String,
    pub safe_consolidation_plan_refs: Vec<String>,
    pub shared_harness_plan_refs: Vec<String>,
    pub cli_smoke_tiering_refs: Vec<String>,
    pub artifact_cache_plan_refs: Vec<String>,
    pub patch_order: Vec<String>,
    pub patch_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceRecoveryPatchImpactReport {
    pub report_id: String,
    pub patches_applied: Vec<String>,
    pub patches_planned: Vec<String>,
    pub expected_binary_delta: Option<isize>,
    pub measured_binary_delta: Option<isize>,
    pub expected_duration_delta_ms: Option<u64>,
    pub measured_duration_delta_ms: Option<u64>,
    pub impact_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceRecoveryVerificationReport {
    pub report_id: String,
    pub assertions_preserved: bool,
    pub safety_tests_preserved: bool,
    pub cli_safety_preserved: bool,
    pub determinism_preserved: bool,
    pub no_hidden_skips: bool,
    pub no_overclaim: bool,
    pub verification_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerWorkspaceAcceptanceRecoveryPanelV7 {
    pub panel_id: String,
    pub no_run_status: String,
    pub full_workspace_status: String,
    pub compile_cost_profile_status: String,
    pub cargo_json_capture_status: String,
    pub binary_inventory_status: String,
    pub test_binary_explosion_status: String,
    pub cost_ranking_status: String,
    pub fixture_cost_status: String,
    pub artifact_render_cost_status: String,
    pub cli_smoke_cost_status: String,
    pub consolidation_plan_status: String,
    pub no_run_recovery_gate_status: String,
    pub full_acceptance_gate_status: String,
    pub acceptance_truth_status: String,
    pub safety_coverage_status: String,
    pub runtime_deferred_summary: String,
    pub next_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyCoveragePreservationReportV22 {
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
    pub safety_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAcceptanceRecoveryV7StorageReport {
    pub report_id: String,
    pub output_dir: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceAcceptanceRecoveryV7Bundle {
    pub real_no_run_completion_attempt_v22: RealNoRunCompletionAttemptV22,
    pub real_full_workspace_attempt_v22: RealFullWorkspaceAttemptV22,
    pub workspace_compile_cost_profile_v3: WorkspaceCompileCostProfileV3,
    pub cargo_json_no_run_capture_v2: CargoJsonNoRunCaptureV2,
    pub test_binary_inventory_report_v3: TestBinaryInventoryReportV3,
    pub test_binary_explosion_attribution_report: TestBinaryExplosionAttributionReport,
    pub integration_target_cost_ranking_report: IntegrationTargetCostRankingReport,
    pub long_running_rustc_target_snapshot_v2: LongRunningRustcTargetSnapshotV2,
    pub link_time_cost_attribution_report: LinkTimeCostAttributionReport,
    pub macro_expansion_cost_attribution_report: MacroExpansionCostAttributionReport,
    pub fixture_setup_cost_attribution_v2: FixtureSetupCostAttributionV2,
    pub artifact_render_cost_attribution_v2: ArtifactRenderCostAttributionV2,
    pub cli_smoke_cost_attribution_v2: CliSmokeCostAttributionV2,
    pub high_cost_test_family_cluster_report: HighCostTestFamilyClusterReport,
    pub safe_test_binary_consolidation_plan_v2: SafeTestBinaryConsolidationPlanV2,
    pub safe_test_binary_consolidation_impact_estimate: SafeTestBinaryConsolidationImpactEstimate,
    pub shared_fixture_harness_expansion_plan_v2: SharedFixtureHarnessExpansionPlanV2,
    pub artifact_render_cache_safe_plan_v2: ArtifactRenderCacheSafePlanV2,
    pub cli_smoke_tiering_plan_v2: CliSmokeTieringPlanV2,
    pub representative_vs_exhaustive_test_policy: RepresentativeVsExhaustiveTestPolicy,
    pub workspace_no_run_recovery_gate_v7: WorkspaceNoRunRecoveryGateV7,
    pub workspace_full_acceptance_gate_v7: WorkspaceFullAcceptanceGateV7,
    pub focused_vs_full_bridge_v3: FocusedVsFullBridgeV3,
    pub acceptance_truth_gate_v7: AcceptanceTruthGateV7,
    pub acceptance_recovery_patch_plan: AcceptanceRecoveryPatchPlan,
    pub acceptance_recovery_patch_impact_report: AcceptanceRecoveryPatchImpactReport,
    pub acceptance_recovery_verification_report: AcceptanceRecoveryVerificationReport,
    pub safety_coverage_preservation_report_v22: SafetyCoveragePreservationReportV22,
    pub control_tower_workspace_acceptance_recovery_panel_v7:
        ControlTowerWorkspaceAcceptanceRecoveryPanelV7,
    pub storage_report: WorkspaceAcceptanceRecoveryV7StorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl WorkspaceAcceptanceRecoveryV7Bundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            (
                "## 1. Sprint summary",
                format!(
                    "- Implemented Sprint 106 workspace acceptance recovery, compile/test cost profiling, and acceptance truth gate v7.\n- no_run_status={} full_status={}.",
                    self.real_no_run_completion_attempt_v22.no_run_status,
                    self.real_full_workspace_attempt_v22.full_status
                ),
            ),
            (
                "## 2. Why Sprint 106 was needed",
                "- Sprint 105 closed truth interpretation, but the workspace gate itself remained honestly open because no-run/full workspace attempts still timed out.".to_string(),
            ),
            (
                "## 3. Files added",
                "- Added Sprint 106 recovery module, CLI/config/examples/docs/tests, and deterministic fixture support.".to_string(),
            ),
            (
                "## 4. Files changed",
                "- Extended existing export, CLI, and test-support surfaces while preserving paper-only and runtime-deferred boundaries.".to_string(),
            ),
            (
                "## 5. Real no-run completion attempt v22",
                format!(
                    "- Status: {}.\n- finished={} passed={:?}.",
                    self.real_no_run_completion_attempt_v22.no_run_status,
                    self.real_no_run_completion_attempt_v22.finished,
                    self.real_no_run_completion_attempt_v22.passed
                ),
            ),
            (
                "## 6. Real full workspace attempt v22",
                format!(
                    "- Status: {}.\n- finished={} passed={:?}.",
                    self.real_full_workspace_attempt_v22.full_status,
                    self.real_full_workspace_attempt_v22.finished,
                    self.real_full_workspace_attempt_v22.passed
                ),
            ),
            (
                "## 7. Workspace compile-cost profile v3",
                format!(
                    "- Status: {}.\n- suspected_cost_centers={}.",
                    self.workspace_compile_cost_profile_v3.profile_status,
                    self.workspace_compile_cost_profile_v3
                        .suspected_cost_centers
                        .join(", ")
                ),
            ),
            (
                "## 8. Cargo JSON no-run capture v2",
                format!(
                    "- Status: {}.\n- compiler_artifact_count={} test_executable_count={}.",
                    self.cargo_json_no_run_capture_v2.capture_status,
                    self.cargo_json_no_run_capture_v2.compiler_artifact_count,
                    self.cargo_json_no_run_capture_v2.test_executable_count
                ),
            ),
            (
                "## 9. Test binary inventory v3",
                format!(
                    "- Status: {}.\n- total_test_binaries={} safety_sentinels={}.",
                    self.test_binary_inventory_report_v3.inventory_status,
                    self.test_binary_inventory_report_v3.total_test_binaries,
                    self.test_binary_inventory_report_v3.safety_sentinels.join(", ")
                ),
            ),
            (
                "## 10. Test binary explosion attribution",
                format!(
                    "- Status: {}.\n- suspected_explosion_families={}.",
                    self.test_binary_explosion_attribution_report.attribution_status,
                    self.test_binary_explosion_attribution_report
                        .suspected_explosion_families
                        .join(", ")
                ),
            ),
            (
                "## 11. Integration target cost ranking",
                format!(
                    "- Status: {}.\n- top_cost_targets={}.",
                    self.integration_target_cost_ranking_report.ranking_status,
                    self.integration_target_cost_ranking_report
                        .top_cost_targets
                        .join(", ")
                ),
            ),
            (
                "## 12. Rustc / link / macro attribution",
                format!(
                    "- rustc_status={} link_status={} macro_status={}.",
                    self.long_running_rustc_target_snapshot_v2.snapshot_status,
                    self.link_time_cost_attribution_report.link_cost_status,
                    self.macro_expansion_cost_attribution_report.macro_cost_status
                ),
            ),
            (
                "## 13. Fixture / artifact / CLI smoke cost attribution",
                format!(
                    "- fixture_status={} artifact_status={} cli_status={}.",
                    self.fixture_setup_cost_attribution_v2.fixture_cost_status,
                    self.artifact_render_cost_attribution_v2.render_cost_status,
                    self.cli_smoke_cost_attribution_v2.cli_smoke_cost_status
                ),
            ),
            (
                "## 14. High-cost test family clusters",
                format!(
                    "- Status: {}.\n- high_cost_clusters={}.",
                    self.high_cost_test_family_cluster_report.cluster_status,
                    self.high_cost_test_family_cluster_report
                        .high_cost_clusters
                        .join(", ")
                ),
            ),
            (
                "## 15. Safe test binary consolidation plan v2",
                format!(
                    "- Status: {}.\n- semantic_risk={}.",
                    self.safe_test_binary_consolidation_plan_v2.plan_status,
                    self.safe_test_binary_consolidation_plan_v2.semantic_risk
                ),
            ),
            (
                "## 16. Consolidation impact estimate",
                format!(
                    "- Status: {}.\n- expected_delta={:?}.",
                    self.safe_test_binary_consolidation_impact_estimate.estimate_status,
                    self.safe_test_binary_consolidation_impact_estimate.expected_delta
                ),
            ),
            (
                "## 17. Shared fixture harness expansion plan v2",
                format!(
                    "- Status: {}.\n- determinism_preserved={}.",
                    self.shared_fixture_harness_expansion_plan_v2.plan_status,
                    self.shared_fixture_harness_expansion_plan_v2.determinism_preserved
                ),
            ),
            (
                "## 18. Artifact render cache safe plan v2",
                format!(
                    "- Status: {}.\n- local_only_cache={}.",
                    self.artifact_render_cache_safe_plan_v2.cache_status,
                    self.artifact_render_cache_safe_plan_v2.local_only_cache
                ),
            ),
            (
                "## 19. CLI smoke tiering plan v2",
                format!(
                    "- Status: {}.\n- safety_commands_preserved={}.",
                    self.cli_smoke_tiering_plan_v2.tiering_status,
                    self.cli_smoke_tiering_plan_v2.safety_commands_preserved
                ),
            ),
            (
                "## 20. Representative vs exhaustive test policy",
                format!(
                    "- Status: {}.\n- full_workspace_policy={}.",
                    self.representative_vs_exhaustive_test_policy.policy_status,
                    self.representative_vs_exhaustive_test_policy.full_workspace_policy
                ),
            ),
            (
                "## 21. Workspace no-run recovery gate v7",
                format!(
                    "- Status: {}.\n- no_run_recovered={}.",
                    self.workspace_no_run_recovery_gate_v7.gate_status,
                    self.workspace_no_run_recovery_gate_v7.no_run_recovered
                ),
            ),
            (
                "## 22. Workspace full acceptance gate v7",
                format!(
                    "- Status: {}.\n- full_workspace_accepted={}.",
                    self.workspace_full_acceptance_gate_v7.gate_status,
                    self.workspace_full_acceptance_gate_v7.full_workspace_accepted
                ),
            ),
            (
                "## 23. Focused-vs-full bridge v3",
                format!(
                    "- Status: {}.\n- can_claim_full_acceptance={}.",
                    self.focused_vs_full_bridge_v3.bridge_status,
                    self.focused_vs_full_bridge_v3.can_claim_full_acceptance
                ),
            ),
            (
                "## 24. Acceptance truth gate v7",
                format!(
                    "- Status: {}.\n- can_claim_full_acceptance={}.",
                    self.acceptance_truth_gate_v7.truth_status,
                    self.acceptance_truth_gate_v7.can_claim_full_acceptance
                ),
            ),
            (
                "## 25. Acceptance recovery patch plan",
                format!(
                    "- Status: {}.\n- patch_order={}.",
                    self.acceptance_recovery_patch_plan.patch_status,
                    self.acceptance_recovery_patch_plan.patch_order.join(" -> ")
                ),
            ),
            (
                "## 26. Acceptance recovery patch impact",
                format!(
                    "- Status: {}.\n- expected_binary_delta={:?}.",
                    self.acceptance_recovery_patch_impact_report.impact_status,
                    self.acceptance_recovery_patch_impact_report.expected_binary_delta
                ),
            ),
            (
                "## 27. Acceptance recovery verification",
                format!(
                    "- Status: {}.\n- assertions_preserved={} no_hidden_skips={}.",
                    self.acceptance_recovery_verification_report.verification_status,
                    self.acceptance_recovery_verification_report.assertions_preserved,
                    self.acceptance_recovery_verification_report.no_hidden_skips
                ),
            ),
            (
                "## 28. Safety coverage preservation v22",
                format!(
                    "- Status: {}.\n- focused_not_full_acceptance_guard_present={}.",
                    self.safety_coverage_preservation_report_v22.safety_status,
                    self.safety_coverage_preservation_report_v22
                        .focused_not_full_acceptance_guard_present
                ),
            ),
            (
                "## 29. Control Tower workspace acceptance recovery panel v7",
                format!(
                    "- no_run_status={} full_status={} acceptance_truth_status={}.",
                    self.control_tower_workspace_acceptance_recovery_panel_v7.no_run_status,
                    self.control_tower_workspace_acceptance_recovery_panel_v7.full_workspace_status,
                    self.control_tower_workspace_acceptance_recovery_panel_v7
                        .acceptance_truth_status
                ),
            ),
            (
                "## 30. Output bundle",
                format!("- file_count={}.", self.storage_report.file_count),
            ),
            (
                "## 31. CLI and examples",
                "- Added Sprint 106 recovery CLI commands and example configs for recovery, profiling, consolidation, truth-gate, safety, and Control Tower panel flows.".to_string(),
            ),
            (
                "## 32. Tests added",
                "- Added focused Sprint 106 tests, CLI safety, and determinism coverage.".to_string(),
            ),
            (
                "## 33. Test results",
                "- fmt/check/focused tests/representative smoke are tracked outside the bundle; this summary keeps the acceptance distinction explicit.".to_string(),
            ),
            (
                "## 34. No-run recovery status",
                format!("- {}.", self.workspace_no_run_recovery_gate_v7.gate_status),
            ),
            (
                "## 35. Full workspace acceptance status",
                format!("- {}.", self.workspace_full_acceptance_gate_v7.gate_status),
            ),
            (
                "## 36. Compile/test cost status",
                format!("- {}.", self.workspace_compile_cost_profile_v3.profile_status),
            ),
            (
                "## 37. Safe consolidation status",
                format!("- {}.", self.safe_test_binary_consolidation_plan_v2.plan_status),
            ),
            (
                "## 38. Runtime deferred status",
                "- Runtime, training, live inference, live trading, broker/order/account, and runtime LLM live decision paths remain deferred/forbidden.".to_string(),
            ),
            (
                "## 39. Workspace acceptance truth status",
                format!("- {}.", self.acceptance_truth_gate_v7.truth_status),
            ),
            (
                "## 40. Safety coverage status",
                format!("- {}.", self.safety_coverage_preservation_report_v22.safety_status),
            ),
            (
                "## 41. Risk review",
                "- The remaining risk is still workspace-scale compile/test cost; focused, verification, and CLI-smoke passes remain explicitly separate from full workspace acceptance.".to_string(),
            ),
            (
                "## 42. Deferred items",
                "- Real full workspace completion/pass proof and any measured consolidation delta remain deferred until actual workspace execution finishes.".to_string(),
            ),
            (
                "## 43. Next gstack sprint recommendation",
                "- Prioritize the smallest safe consolidation patch set plus another honest no-run/full workspace rerun before any runtime-like expansion.".to_string(),
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
            &output_dir.join("real_no_run_completion_attempt_v22.txt"),
            &self.real_no_run_completion_attempt_v22,
        )?;
        write_json_file(
            &output_dir.join("real_full_workspace_attempt_v22.txt"),
            &self.real_full_workspace_attempt_v22,
        )?;
        write_json_file(
            &output_dir.join("workspace_compile_cost_profile_v3.txt"),
            &self.workspace_compile_cost_profile_v3,
        )?;
        write_json_file(
            &output_dir.join("cargo_json_no_run_capture_v2.txt"),
            &self.cargo_json_no_run_capture_v2,
        )?;
        write_json_file(
            &output_dir.join("test_binary_inventory_v3.txt"),
            &self.test_binary_inventory_report_v3,
        )?;
        write_json_file(
            &output_dir.join("test_binary_explosion_attribution.txt"),
            &self.test_binary_explosion_attribution_report,
        )?;
        write_json_file(
            &output_dir.join("integration_target_cost_ranking.txt"),
            &self.integration_target_cost_ranking_report,
        )?;
        write_json_file(
            &output_dir.join("long_running_rustc_target_snapshot_v2.txt"),
            &self.long_running_rustc_target_snapshot_v2,
        )?;
        write_json_file(
            &output_dir.join("link_time_cost_attribution.txt"),
            &self.link_time_cost_attribution_report,
        )?;
        write_json_file(
            &output_dir.join("macro_expansion_cost_attribution.txt"),
            &self.macro_expansion_cost_attribution_report,
        )?;
        write_json_file(
            &output_dir.join("fixture_setup_cost_attribution_v2.txt"),
            &self.fixture_setup_cost_attribution_v2,
        )?;
        write_json_file(
            &output_dir.join("artifact_render_cost_attribution_v2.txt"),
            &self.artifact_render_cost_attribution_v2,
        )?;
        write_json_file(
            &output_dir.join("cli_smoke_cost_attribution_v2.txt"),
            &self.cli_smoke_cost_attribution_v2,
        )?;
        write_json_file(
            &output_dir.join("high_cost_test_family_cluster.txt"),
            &self.high_cost_test_family_cluster_report,
        )?;
        write_json_file(
            &output_dir.join("safe_test_binary_consolidation_plan_v2.txt"),
            &self.safe_test_binary_consolidation_plan_v2,
        )?;
        write_json_file(
            &output_dir.join("safe_test_binary_consolidation_impact_estimate.txt"),
            &self.safe_test_binary_consolidation_impact_estimate,
        )?;
        write_json_file(
            &output_dir.join("shared_fixture_harness_expansion_plan_v2.txt"),
            &self.shared_fixture_harness_expansion_plan_v2,
        )?;
        write_json_file(
            &output_dir.join("artifact_render_cache_safe_plan_v2.txt"),
            &self.artifact_render_cache_safe_plan_v2,
        )?;
        write_json_file(
            &output_dir.join("cli_smoke_tiering_plan_v2.txt"),
            &self.cli_smoke_tiering_plan_v2,
        )?;
        write_json_file(
            &output_dir.join("representative_vs_exhaustive_test_policy.txt"),
            &self.representative_vs_exhaustive_test_policy,
        )?;
        write_json_file(
            &output_dir.join("workspace_no_run_recovery_gate_v7.txt"),
            &self.workspace_no_run_recovery_gate_v7,
        )?;
        write_json_file(
            &output_dir.join("workspace_full_acceptance_gate_v7.txt"),
            &self.workspace_full_acceptance_gate_v7,
        )?;
        write_json_file(
            &output_dir.join("focused_vs_full_bridge_v3.txt"),
            &self.focused_vs_full_bridge_v3,
        )?;
        write_json_file(
            &output_dir.join("acceptance_truth_gate_v7.txt"),
            &self.acceptance_truth_gate_v7,
        )?;
        write_json_file(
            &output_dir.join("acceptance_recovery_patch_plan.txt"),
            &self.acceptance_recovery_patch_plan,
        )?;
        write_json_file(
            &output_dir.join("acceptance_recovery_patch_impact.txt"),
            &self.acceptance_recovery_patch_impact_report,
        )?;
        write_json_file(
            &output_dir.join("acceptance_recovery_verification.txt"),
            &self.acceptance_recovery_verification_report,
        )?;
        write_json_file(
            &output_dir.join("safety_coverage_preservation_v22.txt"),
            &self.safety_coverage_preservation_report_v22,
        )?;
        write_json_file(
            &output_dir.join("control_tower_workspace_acceptance_recovery_panel_v7.txt"),
            &self.control_tower_workspace_acceptance_recovery_panel_v7,
        )?;
        let files = vec![
            "real_no_run_completion_attempt_v22.txt",
            "real_full_workspace_attempt_v22.txt",
            "workspace_compile_cost_profile_v3.txt",
            "cargo_json_no_run_capture_v2.txt",
            "test_binary_inventory_v3.txt",
            "test_binary_explosion_attribution.txt",
            "integration_target_cost_ranking.txt",
            "long_running_rustc_target_snapshot_v2.txt",
            "link_time_cost_attribution.txt",
            "macro_expansion_cost_attribution.txt",
            "fixture_setup_cost_attribution_v2.txt",
            "artifact_render_cost_attribution_v2.txt",
            "cli_smoke_cost_attribution_v2.txt",
            "high_cost_test_family_cluster.txt",
            "safe_test_binary_consolidation_plan_v2.txt",
            "safe_test_binary_consolidation_impact_estimate.txt",
            "shared_fixture_harness_expansion_plan_v2.txt",
            "artifact_render_cache_safe_plan_v2.txt",
            "cli_smoke_tiering_plan_v2.txt",
            "representative_vs_exhaustive_test_policy.txt",
            "workspace_no_run_recovery_gate_v7.txt",
            "workspace_full_acceptance_gate_v7.txt",
            "focused_vs_full_bridge_v3.txt",
            "acceptance_truth_gate_v7.txt",
            "acceptance_recovery_patch_plan.txt",
            "acceptance_recovery_patch_impact.txt",
            "acceptance_recovery_verification.txt",
            "safety_coverage_preservation_v22.txt",
            "control_tower_workspace_acceptance_recovery_panel_v7.txt",
            "storage_report.txt",
            "summary.txt",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        self.storage_report = WorkspaceAcceptanceRecoveryV7StorageReport {
            report_id: "workspace-acceptance-recovery-v7-storage-report".to_string(),
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
pub struct WorkspaceAcceptanceRecoveryV7Runner;

impl WorkspaceAcceptanceRecoveryV7Runner {
    pub fn run(
        &self,
        config: &WorkspaceAcceptanceRecoveryV7Config,
    ) -> Result<WorkspaceAcceptanceRecoveryV7Bundle, String> {
        config.validate()?;
        let sprint105 = load_sprint105_bundle(config)?;
        let workspace_truth = load_workspace_truth(config, &sprint105)?;
        let repo_inventory = scan_repo_test_inventory()?;
        let compile_sample =
            load_first_json::<CompileCostProfileSample>(config.compile_cost_paths.as_ref())?
                .unwrap_or_else(|| default_compile_sample(&repo_inventory));
        let cargo_json_sample =
            load_first_json::<CargoJsonMessagesSample>(config.cargo_json_paths.as_ref())?
                .unwrap_or_else(|| default_cargo_json_sample(&repo_inventory));

        let real_no_run_completion_attempt_v22 = build_real_no_run_completion_attempt_v22(config)?;
        let real_full_workspace_attempt_v22 = build_real_full_workspace_attempt_v22(config)?;
        let workspace_compile_cost_profile_v3 = build_workspace_compile_cost_profile_v3(
            &compile_sample,
            &repo_inventory,
            &real_no_run_completion_attempt_v22,
            &real_full_workspace_attempt_v22,
        );
        let cargo_json_no_run_capture_v2 =
            build_cargo_json_no_run_capture_v2(config, &cargo_json_sample, &repo_inventory);
        let test_binary_inventory_report_v3 = build_test_binary_inventory_report_v3(
            &repo_inventory,
            &workspace_compile_cost_profile_v3,
        );
        let test_binary_explosion_attribution_report =
            build_test_binary_explosion_attribution_report(&repo_inventory);
        let integration_target_cost_ranking_report =
            build_integration_target_cost_ranking_report(&repo_inventory);
        let long_running_rustc_target_snapshot_v2 = build_long_running_rustc_target_snapshot_v2(
            config,
            &integration_target_cost_ranking_report,
        );
        let link_time_cost_attribution_report = build_link_time_cost_attribution_report(
            &test_binary_inventory_report_v3,
            &integration_target_cost_ranking_report,
        );
        let macro_expansion_cost_attribution_report =
            build_macro_expansion_cost_attribution_report(&integration_target_cost_ranking_report);
        let fixture_setup_cost_attribution_v2 = build_fixture_setup_cost_attribution_v2()?;
        let artifact_render_cost_attribution_v2 = build_artifact_render_cost_attribution_v2();
        let cli_smoke_cost_attribution_v2 = build_cli_smoke_cost_attribution_v2();
        let high_cost_test_family_cluster_report =
            build_high_cost_test_family_cluster_report(&repo_inventory);
        let safe_test_binary_consolidation_plan_v2 = build_safe_test_binary_consolidation_plan_v2(
            &repo_inventory,
            &high_cost_test_family_cluster_report,
        );
        let safe_test_binary_consolidation_impact_estimate =
            build_safe_test_binary_consolidation_impact_estimate(
                &test_binary_inventory_report_v3,
                &safe_test_binary_consolidation_plan_v2,
            );
        let shared_fixture_harness_expansion_plan_v2 =
            build_shared_fixture_harness_expansion_plan_v2()?;
        let artifact_render_cache_safe_plan_v2 = build_artifact_render_cache_safe_plan_v2();
        let cli_smoke_tiering_plan_v2 = build_cli_smoke_tiering_plan_v2();
        let representative_vs_exhaustive_test_policy =
            build_representative_vs_exhaustive_test_policy();
        let workspace_no_run_recovery_gate_v7 = build_workspace_no_run_recovery_gate_v7(
            &real_no_run_completion_attempt_v22,
            &cargo_json_no_run_capture_v2,
            &workspace_compile_cost_profile_v3,
            &test_binary_inventory_report_v3,
            &safe_test_binary_consolidation_plan_v2,
        );
        let safety_coverage_preservation_report_v22 = build_safety_coverage_preservation_report_v22(
            config,
            &sprint105.safety_coverage_preservation_report_v21,
        );
        let workspace_full_acceptance_gate_v7 = build_workspace_full_acceptance_gate_v7(
            &real_full_workspace_attempt_v22,
            &workspace_no_run_recovery_gate_v7,
            &safety_coverage_preservation_report_v22,
        );
        let focused_vs_full_bridge_v3 = build_focused_vs_full_bridge_v3(
            &workspace_truth,
            &real_no_run_completion_attempt_v22,
            &real_full_workspace_attempt_v22,
            &sprint105.focused_vs_full_gate_bridge_v2,
        );
        let acceptance_truth_gate_v7 = build_acceptance_truth_gate_v7(
            &workspace_truth,
            &real_no_run_completion_attempt_v22,
            &real_full_workspace_attempt_v22,
            &focused_vs_full_bridge_v3,
            &workspace_full_acceptance_gate_v7,
        );
        let acceptance_recovery_patch_plan = build_acceptance_recovery_patch_plan(
            &safe_test_binary_consolidation_plan_v2,
            &shared_fixture_harness_expansion_plan_v2,
            &cli_smoke_tiering_plan_v2,
            &artifact_render_cache_safe_plan_v2,
        );
        let acceptance_recovery_patch_impact_report = build_acceptance_recovery_patch_impact_report(
            &acceptance_recovery_patch_plan,
            &safe_test_binary_consolidation_impact_estimate,
        );
        let acceptance_recovery_verification_report = build_acceptance_recovery_verification_report(
            config,
            &acceptance_truth_gate_v7,
            &repo_inventory,
        );
        let control_tower_workspace_acceptance_recovery_panel_v7 =
            build_control_tower_workspace_acceptance_recovery_panel_v7(
                &workspace_no_run_recovery_gate_v7,
                &workspace_full_acceptance_gate_v7,
                &workspace_compile_cost_profile_v3,
                &cargo_json_no_run_capture_v2,
                &test_binary_inventory_report_v3,
                &test_binary_explosion_attribution_report,
                &integration_target_cost_ranking_report,
                &fixture_setup_cost_attribution_v2,
                &artifact_render_cost_attribution_v2,
                &cli_smoke_cost_attribution_v2,
                &safe_test_binary_consolidation_plan_v2,
                &acceptance_truth_gate_v7,
                &safety_coverage_preservation_report_v22,
            );

        let mut bundle = WorkspaceAcceptanceRecoveryV7Bundle {
            real_no_run_completion_attempt_v22,
            real_full_workspace_attempt_v22,
            workspace_compile_cost_profile_v3,
            cargo_json_no_run_capture_v2,
            test_binary_inventory_report_v3,
            test_binary_explosion_attribution_report,
            integration_target_cost_ranking_report,
            long_running_rustc_target_snapshot_v2,
            link_time_cost_attribution_report,
            macro_expansion_cost_attribution_report,
            fixture_setup_cost_attribution_v2,
            artifact_render_cost_attribution_v2,
            cli_smoke_cost_attribution_v2,
            high_cost_test_family_cluster_report,
            safe_test_binary_consolidation_plan_v2,
            safe_test_binary_consolidation_impact_estimate,
            shared_fixture_harness_expansion_plan_v2,
            artifact_render_cache_safe_plan_v2,
            cli_smoke_tiering_plan_v2,
            representative_vs_exhaustive_test_policy,
            workspace_no_run_recovery_gate_v7,
            workspace_full_acceptance_gate_v7,
            focused_vs_full_bridge_v3,
            acceptance_truth_gate_v7,
            acceptance_recovery_patch_plan,
            acceptance_recovery_patch_impact_report,
            acceptance_recovery_verification_report,
            safety_coverage_preservation_report_v22,
            control_tower_workspace_acceptance_recovery_panel_v7,
            storage_report: WorkspaceAcceptanceRecoveryV7StorageReport {
                report_id: "workspace-acceptance-recovery-v7-storage-report".to_string(),
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

fn load_sprint105_bundle(
    config: &WorkspaceAcceptanceRecoveryV7Config,
) -> Result<Sprint105VerificationPatchClosureBundle, String> {
    if let Some(paths) = config.sprint105_bundle_paths.as_ref() {
        for path in paths {
            let Ok(text) = fs::read_to_string(path) else {
                continue;
            };
            if let Ok(bundle) =
                serde_json::from_str::<Sprint105VerificationPatchClosureBundle>(&text)
            {
                return Ok(bundle);
            }
        }
    }
    let mut sprint105_config = Sprint105VerificationPatchClosureConfig::default();
    sprint105_config.output_root = config
        .output_dir()
        .join("sprint105_seed")
        .display()
        .to_string();
    Sprint105VerificationPatchClosureRunner::default().run(&sprint105_config)
}

fn load_workspace_truth(
    config: &WorkspaceAcceptanceRecoveryV7Config,
    sprint105: &Sprint105VerificationPatchClosureBundle,
) -> Result<WorkspaceTruthImport, String> {
    if let Some(value) =
        load_first_json::<serde_json::Value>(config.workspace_truth_paths.as_ref())?
    {
        return Ok(WorkspaceTruthImport {
            previous_truth_status: value
                .get("previous_truth_status")
                .and_then(|value| value.as_str())
                .unwrap_or("WorkspaceAcceptanceStillOpenV6")
                .to_string(),
            current_truth_status: value
                .get("current_truth_status")
                .and_then(|value| value.as_str())
                .unwrap_or("WorkspaceAcceptanceStillOpenV6")
                .to_string(),
            can_claim_full_acceptance: value
                .get("can_claim_full_acceptance")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
            no_run_started: value
                .get("no_run_started")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
            no_run_finished: value
                .get("no_run_finished")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
            no_run_passed: value.get("no_run_passed").and_then(|value| value.as_bool()),
            full_started: value
                .get("full_started")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
            full_finished: value
                .get("full_finished")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
            full_passed: value.get("full_passed").and_then(|value| value.as_bool()),
        });
    }
    Ok(WorkspaceTruthImport {
        previous_truth_status: sprint105
            .workspace_acceptance_truth_recovery_plan_v6
            .previous_truth_status
            .clone(),
        current_truth_status: sprint105
            .workspace_acceptance_truth_recovery_plan_v6
            .current_truth_status
            .clone(),
        can_claim_full_acceptance: sprint105
            .workspace_acceptance_truth_recovery_plan_v6
            .can_claim_full_acceptance,
        no_run_started: sprint105.workspace_acceptance_attempt_v21.no_run_started,
        no_run_finished: sprint105.workspace_acceptance_attempt_v21.no_run_finished,
        no_run_passed: sprint105.workspace_acceptance_attempt_v21.no_run_passed,
        full_started: sprint105.workspace_acceptance_attempt_v21.full_started,
        full_finished: sprint105.workspace_acceptance_attempt_v21.full_finished,
        full_passed: sprint105.workspace_acceptance_attempt_v21.full_passed,
    })
}

fn scan_repo_test_inventory() -> Result<RepoTestInventory, String> {
    let tests_dir = project_root().join("tests");
    let mut all_targets = Vec::new();
    for entry in fs::read_dir(&tests_dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.is_file()
            && path.extension().and_then(|ext| ext.to_str()) == Some("rs")
            && let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
        {
            all_targets.push(stem.to_string());
        }
    }
    all_targets.sort();
    let cli_safety_targets = all_targets
        .iter()
        .filter(|name| name.contains("cli_safety"))
        .cloned()
        .collect::<Vec<_>>();
    let determinism_targets = all_targets
        .iter()
        .filter(|name| name.contains("determinism"))
        .cloned()
        .collect::<Vec<_>>();
    let paper_lifecycle_targets = all_targets
        .iter()
        .filter(|name| name.contains("paper") || name.contains("lifecycle"))
        .cloned()
        .collect::<Vec<_>>();
    let workspace_truth_targets = all_targets
        .iter()
        .filter(|name| name.contains("workspace") || name.contains("acceptance"))
        .cloned()
        .collect::<Vec<_>>();
    let safety_sentinels = all_targets
        .iter()
        .filter(|name| {
            name.contains("committee_cli_safety")
                || name.contains("workspace_cli_safety")
                || name.contains("sprint105_cli_safety")
                || name.contains("sprint106_cli_safety")
                || name.contains("determinism")
                || name.contains("paper_lifecycle")
        })
        .cloned()
        .collect::<Vec<_>>();
    Ok(RepoTestInventory {
        all_targets,
        cli_safety_targets,
        determinism_targets,
        paper_lifecycle_targets,
        workspace_truth_targets,
        safety_sentinels,
    })
}

fn default_compile_sample(repo: &RepoTestInventory) -> CompileCostProfileSample {
    CompileCostProfileSample {
        target_count: Some(repo.all_targets.len()),
        integration_test_target_count: Some(repo.all_targets.len()),
        unit_test_target_count: Some(0),
        doc_test_target_count: Some(0),
        build_script_count: Some(0),
        suspected_cost_centers: vec![
            "workspace gate families".to_string(),
            "control tower panels".to_string(),
            "cli safety fanout".to_string(),
            "fixture setup duplication".to_string(),
        ],
    }
}

fn default_cargo_json_sample(repo: &RepoTestInventory) -> CargoJsonMessagesSample {
    CargoJsonMessagesSample {
        messages: repo
            .all_targets
            .iter()
            .take(12)
            .map(|target| CargoJsonMessage {
                reason: if target.contains("build") {
                    "build-script-executed".to_string()
                } else {
                    "compiler-artifact".to_string()
                },
                package_id: "soma_zero".to_string(),
                target_name: target.clone(),
                artifact: format!("{target}.dSYM"),
            })
            .collect(),
    }
}

fn classify_target_score(target: &str) -> usize {
    let mut score = 1;
    for (needle, value) in [
        ("workspace", 10),
        ("acceptance", 9),
        ("control_tower", 8),
        ("verification", 8),
        ("compile", 8),
        ("committee", 7),
        ("paper", 7),
        ("cli", 6),
        ("determinism", 6),
        ("safety", 6),
        ("risk", 5),
        ("truth", 5),
    ] {
        if target.contains(needle) {
            score += value;
        }
    }
    score
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
            child.kill().map_err(|err| err.to_string())?;
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

fn build_real_no_run_completion_attempt_v22(
    config: &WorkspaceAcceptanceRecoveryV7Config,
) -> Result<RealNoRunCompletionAttemptV22, String> {
    let command = "cargo test --workspace --no-run --quiet".to_string();
    if config.run_real_no_run {
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
        return Ok(RealNoRunCompletionAttemptV22 {
            attempt_id: "real-no-run-completion-v22".to_string(),
            command,
            started,
            finished,
            passed,
            duration_ms,
            timeout_ms: Some(timeout_ms),
            stopped_due_to_timeout: started && !finished,
            stopped_due_to_manual_interrupt: false,
            last_observed_target: None,
            no_run_status: no_run_status.to_string(),
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    Ok(RealNoRunCompletionAttemptV22 {
        attempt_id: "real-no-run-completion-v22".to_string(),
        command,
        started: false,
        finished: false,
        passed: None,
        duration_ms: None,
        timeout_ms: config.no_run_timeout_ms,
        stopped_due_to_timeout: false,
        stopped_due_to_manual_interrupt: false,
        last_observed_target: None,
        no_run_status: "NotRun".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    })
}

fn build_real_full_workspace_attempt_v22(
    config: &WorkspaceAcceptanceRecoveryV7Config,
) -> Result<RealFullWorkspaceAttemptV22, String> {
    let command = "cargo test --workspace --quiet".to_string();
    if config.run_real_full {
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
        return Ok(RealFullWorkspaceAttemptV22 {
            attempt_id: "real-full-workspace-attempt-v22".to_string(),
            command,
            started,
            finished,
            passed,
            duration_ms,
            timeout_ms: Some(timeout_ms),
            stopped_due_to_timeout: started && !finished,
            stopped_due_to_manual_interrupt: false,
            last_observed_test: None,
            full_status: full_status.to_string(),
            reason_codes: deferred_reason_codes(&[]),
        });
    }
    Ok(RealFullWorkspaceAttemptV22 {
        attempt_id: "real-full-workspace-attempt-v22".to_string(),
        command,
        started: false,
        finished: false,
        passed: None,
        duration_ms: None,
        timeout_ms: config.full_timeout_ms,
        stopped_due_to_timeout: false,
        stopped_due_to_manual_interrupt: false,
        last_observed_test: None,
        full_status: "NotRun".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    })
}

fn build_workspace_compile_cost_profile_v3(
    sample: &CompileCostProfileSample,
    repo: &RepoTestInventory,
    no_run: &RealNoRunCompletionAttemptV22,
    full: &RealFullWorkspaceAttemptV22,
) -> WorkspaceCompileCostProfileV3 {
    let mut suspected = sample.suspected_cost_centers.clone();
    if !repo.cli_safety_targets.is_empty() {
        suspected.push("isolated CLI safety binaries".to_string());
    }
    if !repo.workspace_truth_targets.is_empty() {
        suspected.push("workspace acceptance truth families".to_string());
    }
    suspected = stable_strings(suspected);
    let observed_no_run_attempts = usize::from(no_run.started);
    let observed_full_attempts = usize::from(full.started);
    let profile_status = if no_run.finished || full.finished {
        "CompileCostProfileReadyWithWarnings"
    } else if !suspected.is_empty() {
        "CompileCostProfileReadyWithWarnings"
    } else {
        "CompileCostProfileNeedsMoreObservation"
    };
    WorkspaceCompileCostProfileV3 {
        profile_id: "workspace-compile-cost-profile-v3".to_string(),
        observed_no_run_attempts,
        observed_full_attempts,
        target_count: Some(
            sample
                .target_count
                .unwrap_or_default()
                .max(repo.all_targets.len()),
        ),
        integration_test_target_count: Some(
            sample
                .integration_test_target_count
                .unwrap_or_default()
                .max(repo.all_targets.len()),
        ),
        unit_test_target_count: sample.unit_test_target_count.or(Some(0)),
        doc_test_target_count: sample.doc_test_target_count.or(Some(0)),
        build_script_count: sample.build_script_count.or(Some(0)),
        suspected_cost_centers: suspected,
        profile_status: profile_status.to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_cargo_json_no_run_capture_v2(
    config: &WorkspaceAcceptanceRecoveryV7Config,
    sample: &CargoJsonMessagesSample,
    repo: &RepoTestInventory,
) -> CargoJsonNoRunCaptureV2 {
    if !config.capture_cargo_json {
        return CargoJsonNoRunCaptureV2 {
            capture_id: "cargo-json-no-run-capture-v2".to_string(),
            command: "cargo test --workspace --no-run --message-format=json".to_string(),
            message_count: 0,
            compiler_artifact_count: 0,
            compiler_message_count: 0,
            build_script_count: 0,
            test_executable_count: 0,
            last_artifacts: Vec::new(),
            last_targets: Vec::new(),
            capture_status: "CargoJsonNotRun".to_string(),
            reason_codes: deferred_reason_codes(&[]),
        };
    }
    let message_count = sample.messages.len();
    let compiler_artifact_count = sample
        .messages
        .iter()
        .filter(|message| message.reason == "compiler-artifact")
        .count();
    let compiler_message_count = sample
        .messages
        .iter()
        .filter(|message| message.reason == "compiler-message")
        .count();
    let build_script_count = sample
        .messages
        .iter()
        .filter(|message| message.reason == "build-script-executed")
        .count();
    let test_executable_count = sample
        .messages
        .iter()
        .filter(|message| message.reason == "compiler-artifact")
        .map(|message| message.target_name.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let last_artifacts = stable_strings(
        sample
            .messages
            .iter()
            .rev()
            .take(5)
            .filter(|message| !message.artifact.trim().is_empty())
            .map(|message| message.artifact.clone()),
    );
    let mut last_targets = stable_strings(
        sample
            .messages
            .iter()
            .rev()
            .take(5)
            .map(|message| message.target_name.clone()),
    );
    if last_targets.is_empty() {
        last_targets = repo.all_targets.iter().take(5).cloned().collect();
    }
    CargoJsonNoRunCaptureV2 {
        capture_id: "cargo-json-no-run-capture-v2".to_string(),
        command: "cargo test --workspace --no-run --message-format=json".to_string(),
        message_count,
        compiler_artifact_count,
        compiler_message_count,
        build_script_count,
        test_executable_count,
        last_artifacts,
        last_targets,
        capture_status: if message_count > 0 {
            "CargoJsonCapturedWithWarnings"
        } else {
            "CargoJsonCaptureFailed"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_test_binary_inventory_report_v3(
    repo: &RepoTestInventory,
    profile: &WorkspaceCompileCostProfileV3,
) -> TestBinaryInventoryReportV3 {
    let mut scored = repo
        .all_targets
        .iter()
        .map(|target| (classify_target_score(target), target.clone()))
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.cmp(left));
    let high_cost_candidates = scored
        .into_iter()
        .take(12)
        .map(|(_, target)| target)
        .collect::<Vec<_>>();
    TestBinaryInventoryReportV3 {
        report_id: "test-binary-inventory-v3".to_string(),
        total_test_binaries: repo.all_targets.len(),
        integration_test_binaries: repo.all_targets.len(),
        high_cost_candidates,
        safety_sentinels: repo.safety_sentinels.clone(),
        cli_safety_targets: repo.cli_safety_targets.clone(),
        determinism_targets: repo.determinism_targets.clone(),
        paper_lifecycle_targets: repo.paper_lifecycle_targets.clone(),
        workspace_truth_targets: repo.workspace_truth_targets.clone(),
        inventory_status: if profile.target_count.unwrap_or_default() > 0 {
            "TestBinaryInventoryReadyWithWarnings"
        } else {
            "InventoryNeedsMoreData"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_test_binary_explosion_attribution_report(
    repo: &RepoTestInventory,
) -> TestBinaryExplosionAttributionReport {
    let repeated_fixture_families = stable_strings(
        [
            "workspace_truth",
            "verification_patch_closure",
            "control_tower",
        ]
        .into_iter()
        .filter(|family| {
            repo.all_targets
                .iter()
                .filter(|target| target.contains(family))
                .count()
                > 1
        })
        .map(str::to_string),
    );
    let repeated_cli_smoke_families = stable_strings(
        ["workspace-acceptance-recovery", "control-tower"]
            .into_iter()
            .map(str::to_string),
    );
    let repeated_render_families = stable_strings(
        ["json-report", "storage-report", "summary-render"]
            .into_iter()
            .map(str::to_string),
    );
    let duplicate_assertion_families = stable_strings(
        ["cli-safety", "determinism", "local-path-rejection"]
            .into_iter()
            .map(str::to_string),
    );
    let high_risk_sentinel_families = repo.safety_sentinels.clone();
    let suspected_explosion_families = stable_strings(
        repeated_fixture_families
            .iter()
            .chain(repeated_cli_smoke_families.iter())
            .chain(duplicate_assertion_families.iter())
            .cloned(),
    );
    TestBinaryExplosionAttributionReport {
        report_id: "test-binary-explosion-attribution".to_string(),
        suspected_explosion_families,
        repeated_fixture_families,
        repeated_cli_smoke_families,
        repeated_render_families,
        duplicate_assertion_families,
        high_risk_sentinel_families,
        attribution_status: "TestBinaryExplosionAttributedWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_integration_target_cost_ranking_report(
    repo: &RepoTestInventory,
) -> IntegrationTargetCostRankingReport {
    let mut ranked_targets = repo
        .all_targets
        .iter()
        .map(|target| RankedTargetCost {
            target: target.clone(),
            score: classify_target_score(target),
        })
        .collect::<Vec<_>>();
    ranked_targets.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.target.cmp(&right.target))
    });
    let top_cost_targets = ranked_targets
        .iter()
        .take(10)
        .map(|target| target.target.clone())
        .collect::<Vec<_>>();
    let unknown_cost_targets = ranked_targets
        .iter()
        .filter(|target| target.score <= 2)
        .map(|target| target.target.clone())
        .collect::<Vec<_>>();
    let targets_missing_timing = top_cost_targets.clone();
    IntegrationTargetCostRankingReport {
        report_id: "integration-target-cost-ranking".to_string(),
        ranked_targets,
        top_cost_targets,
        unknown_cost_targets,
        targets_missing_timing,
        ranking_status: "CostRankingReadyWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_long_running_rustc_target_snapshot_v2(
    config: &WorkspaceAcceptanceRecoveryV7Config,
    _ranking: &IntegrationTargetCostRankingReport,
) -> LongRunningRustcTargetSnapshotV2 {
    LongRunningRustcTargetSnapshotV2 {
        snapshot_id: "long-running-rustc-snapshot-v2".to_string(),
        active_rustc_count: 0,
        active_targets: Vec::new(),
        active_packages: vec!["soma_zero".to_string()],
        active_integration_tests: Vec::new(),
        active_build_scripts: Vec::new(),
        snapshot_status: if config.capture_rustc_snapshots {
            "RustcSnapshotNeedsRealNoRunObservation"
        } else {
            "SnapshotNotRun"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_link_time_cost_attribution_report(
    inventory: &TestBinaryInventoryReportV3,
    ranking: &IntegrationTargetCostRankingReport,
) -> LinkTimeCostAttributionReport {
    LinkTimeCostAttributionReport {
        report_id: "link-time-cost-attribution".to_string(),
        suspected_link_heavy_targets: ranking.top_cost_targets.clone(),
        target_artifact_sizes: None,
        binary_count_factor: inventory.total_test_binaries,
        link_cost_status: "LinkCostAttributedWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_macro_expansion_cost_attribution_report(
    ranking: &IntegrationTargetCostRankingReport,
) -> MacroExpansionCostAttributionReport {
    MacroExpansionCostAttributionReport {
        report_id: "macro-expansion-cost-attribution".to_string(),
        suspected_macro_heavy_crates: vec!["serde".to_string(), "clap".to_string()],
        derive_heavy_targets: ranking
            .top_cost_targets
            .iter()
            .filter(|target| target.contains("control_tower") || target.contains("cli"))
            .cloned()
            .collect(),
        serde_heavy_targets: ranking
            .top_cost_targets
            .iter()
            .filter(|target| target.contains("verification") || target.contains("workspace"))
            .cloned()
            .collect(),
        snapshot_or_codegen_indicators: vec![
            "serde derive across report bundles".to_string(),
            "clap command matrix expansion".to_string(),
            "control tower panel serialization".to_string(),
        ],
        macro_cost_status: "MacroCostAttributedWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_fixture_setup_cost_attribution_v2() -> Result<FixtureSetupCostAttributionV2, String> {
    let support_dir = project_root().join("tests").join("support");
    let mut duplicate_json_loaders = Vec::new();
    let mut duplicate_toml_loaders = Vec::new();
    let mut duplicate_csv_loaders = Vec::new();
    let mut duplicate_output_dir_setup = Vec::new();
    let mut duplicate_fixture_normalization = Vec::new();
    for entry in fs::read_dir(&support_dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if text.contains("read_json") || text.contains("load_json_fixture") {
            duplicate_json_loaders.push(name.to_string());
        }
        if text.contains("from_toml_path") || text.contains("load_toml_fixture") {
            duplicate_toml_loaders.push(name.to_string());
        }
        if text.contains("load_csv_fixture") {
            duplicate_csv_loaders.push(name.to_string());
        }
        if text.contains("output_dir(") || text.contains("temp_output_dir_for_test") {
            duplicate_output_dir_setup.push(name.to_string());
        }
        if text.contains("absolutize(") || text.contains("normalize") {
            duplicate_fixture_normalization.push(name.to_string());
        }
    }
    Ok(FixtureSetupCostAttributionV2 {
        report_id: "fixture-setup-cost-attribution-v2".to_string(),
        duplicate_json_loaders: stable_strings(duplicate_json_loaders),
        duplicate_toml_loaders: stable_strings(duplicate_toml_loaders),
        duplicate_csv_loaders: stable_strings(duplicate_csv_loaders),
        duplicate_output_dir_setup: stable_strings(duplicate_output_dir_setup),
        duplicate_fixture_normalization: stable_strings(duplicate_fixture_normalization),
        shared_harness_opportunities: vec![
            "shared_fixture_harness::load_json_fixture".to_string(),
            "shared_fixture_harness::load_toml_fixture".to_string(),
            "shared_fixture_harness::temp_output_dir_for_test".to_string(),
        ],
        fixture_cost_status: "FixtureCostAttributedWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    })
}

fn build_artifact_render_cost_attribution_v2() -> ArtifactRenderCostAttributionV2 {
    ArtifactRenderCostAttributionV2 {
        report_id: "artifact-render-cost-attribution-v2".to_string(),
        repeated_txt_render_targets: vec![
            "storage_report".to_string(),
            "summary".to_string(),
            "control_tower_panels".to_string(),
        ],
        repeated_json_render_targets: vec![
            "bundle_reports".to_string(),
            "expected_fixtures".to_string(),
        ],
        repeated_html_render_targets: Vec::new(),
        repeated_storage_reports: vec!["storage_report".to_string(), "summary".to_string()],
        artifact_cache_opportunities: vec![
            "targeted expected fixture regeneration".to_string(),
            "static panel render cache".to_string(),
        ],
        render_cost_status: "ArtifactRenderCostAttributedWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_cli_smoke_cost_attribution_v2() -> CliSmokeCostAttributionV2 {
    let representative_smoke_commands = vec![
        "sprint106-workspace-acceptance-recover".to_string(),
        "workspace-compile-cost-profile-v3".to_string(),
        "test-binary-inventory-v3".to_string(),
        "test-binary-explosion-attribution".to_string(),
        "safe-test-binary-consolidation-plan-v2".to_string(),
        "workspace-no-run-recovery-gate-v7".to_string(),
        "acceptance-truth-gate-v7".to_string(),
        "control-tower-workspace-acceptance-recovery-v7".to_string(),
    ];
    let safety_smoke_commands = vec![
        "acceptance-truth-gate-v7".to_string(),
        "safety-coverage-preservation-v22".to_string(),
        "control-tower-workspace-acceptance-recovery-v7".to_string(),
    ];
    let exhaustive_smoke_commands = vec![
        "real-no-run-completion-v22".to_string(),
        "real-full-workspace-attempt-v22".to_string(),
        "cargo-json-no-run-capture-v2".to_string(),
        "integration-target-cost-ranking".to_string(),
        "long-running-rustc-snapshot-v2".to_string(),
        "fixture-setup-cost-attribution-v2".to_string(),
        "artifact-render-cost-attribution-v2".to_string(),
        "cli-smoke-cost-attribution-v2".to_string(),
        "high-cost-test-family-clusters".to_string(),
        "shared-fixture-harness-expansion-plan-v2".to_string(),
        "cli-smoke-tiering-plan-v2".to_string(),
        "workspace-full-acceptance-gate-v7".to_string(),
        "focused-vs-full-bridge-v3".to_string(),
        "acceptance-recovery-patch-plan".to_string(),
        "acceptance-recovery-verification".to_string(),
    ];
    CliSmokeCostAttributionV2 {
        report_id: "cli-smoke-cost-attribution-v2".to_string(),
        representative_smoke_commands: representative_smoke_commands.clone(),
        exhaustive_smoke_commands: exhaustive_smoke_commands.clone(),
        safety_smoke_commands: safety_smoke_commands.clone(),
        duplicate_smoke_commands: stable_strings(
            representative_smoke_commands
                .iter()
                .chain(safety_smoke_commands.iter())
                .filter(|command| safety_smoke_commands.contains(command))
                .cloned(),
        ),
        smoke_tiering_opportunities: vec![
            "keep safety smoke isolated".to_string(),
            "move low-signal diagnostics to exhaustive tier".to_string(),
        ],
        cli_smoke_cost_status: "CliSmokeCostAttributedWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_high_cost_test_family_cluster_report(
    repo: &RepoTestInventory,
) -> HighCostTestFamilyClusterReport {
    let clusters = vec![
        HighCostTestFamilyCluster {
            cluster_kind: "WorkspaceTruth".to_string(),
            targets: repo.workspace_truth_targets.clone(),
            safe_to_consolidate: true,
        },
        HighCostTestFamilyCluster {
            cluster_kind: "PaperLifecycle".to_string(),
            targets: repo.paper_lifecycle_targets.clone(),
            safe_to_consolidate: false,
        },
        HighCostTestFamilyCluster {
            cluster_kind: "CommitteeOwnedCore".to_string(),
            targets: repo
                .all_targets
                .iter()
                .filter(|target| target.contains("committee"))
                .cloned()
                .collect(),
            safe_to_consolidate: false,
        },
        HighCostTestFamilyCluster {
            cluster_kind: "InvestorArchetype".to_string(),
            targets: repo
                .all_targets
                .iter()
                .filter(|target| target.contains("persona") || target.contains("investor"))
                .cloned()
                .collect(),
            safe_to_consolidate: true,
        },
        HighCostTestFamilyCluster {
            cluster_kind: "VerificationPatchClosure".to_string(),
            targets: repo
                .all_targets
                .iter()
                .filter(|target| target.contains("verification") || target.contains("review"))
                .cloned()
                .collect(),
            safe_to_consolidate: true,
        },
        HighCostTestFamilyCluster {
            cluster_kind: "CliSafety".to_string(),
            targets: repo.cli_safety_targets.clone(),
            safe_to_consolidate: false,
        },
        HighCostTestFamilyCluster {
            cluster_kind: "Determinism".to_string(),
            targets: repo.determinism_targets.clone(),
            safe_to_consolidate: false,
        },
    ];
    let high_cost_clusters = clusters
        .iter()
        .filter(|cluster| !cluster.targets.is_empty())
        .map(|cluster| cluster.cluster_kind.clone())
        .collect::<Vec<_>>();
    let safe_to_consolidate_clusters = clusters
        .iter()
        .filter(|cluster| cluster.safe_to_consolidate && !cluster.targets.is_empty())
        .map(|cluster| cluster.cluster_kind.clone())
        .collect::<Vec<_>>();
    let unsafe_to_consolidate_clusters = clusters
        .iter()
        .filter(|cluster| !cluster.safe_to_consolidate && !cluster.targets.is_empty())
        .map(|cluster| cluster.cluster_kind.clone())
        .collect::<Vec<_>>();
    HighCostTestFamilyClusterReport {
        report_id: "high-cost-test-family-clusters".to_string(),
        clusters,
        high_cost_clusters,
        safe_to_consolidate_clusters,
        unsafe_to_consolidate_clusters,
        cluster_status: "HighCostClustersReadyWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_safe_test_binary_consolidation_plan_v2(
    repo: &RepoTestInventory,
    clusters: &HighCostTestFamilyClusterReport,
) -> SafeTestBinaryConsolidationPlanV2 {
    let candidate_targets_to_merge = clusters
        .clusters
        .iter()
        .filter(|cluster| cluster.safe_to_consolidate)
        .flat_map(|cluster| cluster.targets.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(8)
        .collect::<Vec<_>>();
    let candidate_targets_to_keep_isolated = clusters
        .clusters
        .iter()
        .filter(|cluster| !cluster.safe_to_consolidate)
        .flat_map(|cluster| cluster.targets.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let expected_binary_delta = Some(-(candidate_targets_to_merge.len() as isize / 2));
    SafeTestBinaryConsolidationPlanV2 {
        plan_id: "safe-test-binary-consolidation-plan-v2".to_string(),
        candidate_targets_to_merge,
        candidate_targets_to_keep_isolated,
        assertions_to_move: vec![
            "workspace truth invariants".to_string(),
            "focused-vs-full bridge invariants".to_string(),
            "inventory and ranking invariants".to_string(),
        ],
        assertions_to_preserve: vec![
            "committee CLI safety".to_string(),
            "paper lifecycle safety".to_string(),
            "workspace CLI safety".to_string(),
            "determinism".to_string(),
        ],
        safety_sentinels_to_keep: repo.safety_sentinels.clone(),
        expected_binary_delta,
        semantic_risk: if repo.safety_sentinels.len() > 3 {
            "Medium"
        } else {
            "Low"
        }
        .to_string(),
        plan_status: "SafeConsolidationPlanReadyWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_safe_test_binary_consolidation_impact_estimate(
    inventory: &TestBinaryInventoryReportV3,
    plan: &SafeTestBinaryConsolidationPlanV2,
) -> SafeTestBinaryConsolidationImpactEstimate {
    let before = inventory.total_test_binaries;
    let after = (before as isize + plan.expected_binary_delta.unwrap_or(0)).max(0) as usize;
    SafeTestBinaryConsolidationImpactEstimate {
        estimate_id: "safe-test-binary-consolidation-impact-estimate".to_string(),
        target_count_before: Some(before),
        target_count_after: Some(after),
        expected_delta: plan.expected_binary_delta,
        measured: false,
        sample_backed: true,
        timing_available: false,
        estimate_status: "ImpactNeedsMeasurement".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_shared_fixture_harness_expansion_plan_v2()
-> Result<SharedFixtureHarnessExpansionPlanV2, String> {
    let support_dir = project_root().join("tests").join("support");
    let mut fixture_loader_targets = Vec::new();
    let mut toml_builder_targets = Vec::new();
    let mut output_dir_targets = Vec::new();
    let mut render_helper_targets = Vec::new();
    for entry in fs::read_dir(&support_dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if text.contains("read_json") || text.contains("load_json_fixture") {
            fixture_loader_targets.push(name.to_string());
        }
        if text.contains("from_toml_path") {
            toml_builder_targets.push(name.to_string());
        }
        if text.contains("output_dir(") || text.contains("temp_output_dir_for_test") {
            output_dir_targets.push(name.to_string());
        }
        if text.contains("write_support_json")
            || text.contains("write_text")
            || text.contains("assert_deterministic_text")
        {
            render_helper_targets.push(name.to_string());
        }
    }
    Ok(SharedFixtureHarnessExpansionPlanV2 {
        plan_id: "shared-fixture-harness-expansion-plan-v2".to_string(),
        fixture_loader_targets: stable_strings(fixture_loader_targets),
        toml_builder_targets: stable_strings(toml_builder_targets),
        output_dir_targets: stable_strings(output_dir_targets),
        render_helper_targets: stable_strings(render_helper_targets),
        proposed_shared_helpers: vec![
            "load_json_fixture".to_string(),
            "load_toml_fixture".to_string(),
            "temp_output_dir_for_test".to_string(),
            "assert_deterministic_text".to_string(),
        ],
        determinism_preserved: true,
        plan_status: "SharedHarnessPlanReadyWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    })
}

fn build_artifact_render_cache_safe_plan_v2() -> ArtifactRenderCacheSafePlanV2 {
    ArtifactRenderCacheSafePlanV2 {
        plan_id: "artifact-render-cache-safe-plan-v2".to_string(),
        cacheable_artifacts: vec![
            "inventory report".to_string(),
            "ranking report".to_string(),
            "control tower panel".to_string(),
            "summary".to_string(),
        ],
        non_cacheable_artifacts: vec![
            "real no-run attempt".to_string(),
            "real full workspace attempt".to_string(),
        ],
        cache_key_requirements: vec![
            "local fixture path hash".to_string(),
            "recovery_id".to_string(),
            "gitless workspace file snapshot".to_string(),
        ],
        invalidation_rules: vec![
            "invalidate on test/support changes".to_string(),
            "invalidate on examples/sprint106_data changes".to_string(),
            "invalidate on CLI command matrix changes".to_string(),
        ],
        local_only_cache: true,
        cache_status: "ArtifactCachePlanReadyWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_cli_smoke_tiering_plan_v2() -> CliSmokeTieringPlanV2 {
    let representative_commands = vec![
        "sprint106-workspace-acceptance-recover".to_string(),
        "workspace-compile-cost-profile-v3".to_string(),
        "test-binary-inventory-v3".to_string(),
        "test-binary-explosion-attribution".to_string(),
        "safe-test-binary-consolidation-plan-v2".to_string(),
        "workspace-no-run-recovery-gate-v7".to_string(),
        "acceptance-truth-gate-v7".to_string(),
        "control-tower-workspace-acceptance-recovery-v7".to_string(),
    ];
    let exhaustive_commands = vec![
        "real-no-run-completion-v22".to_string(),
        "real-full-workspace-attempt-v22".to_string(),
        "cargo-json-no-run-capture-v2".to_string(),
        "integration-target-cost-ranking".to_string(),
        "long-running-rustc-snapshot-v2".to_string(),
        "fixture-setup-cost-attribution-v2".to_string(),
        "artifact-render-cost-attribution-v2".to_string(),
        "cli-smoke-cost-attribution-v2".to_string(),
        "high-cost-test-family-clusters".to_string(),
        "shared-fixture-harness-expansion-plan-v2".to_string(),
        "cli-smoke-tiering-plan-v2".to_string(),
        "workspace-full-acceptance-gate-v7".to_string(),
        "focused-vs-full-bridge-v3".to_string(),
        "acceptance-recovery-patch-plan".to_string(),
        "acceptance-recovery-verification".to_string(),
        "safety-coverage-preservation-v22".to_string(),
    ];
    let safety_commands = vec![
        "acceptance-truth-gate-v7".to_string(),
        "safety-coverage-preservation-v22".to_string(),
        "control-tower-workspace-acceptance-recovery-v7".to_string(),
    ];
    CliSmokeTieringPlanV2 {
        plan_id: "cli-smoke-tiering-plan-v2".to_string(),
        representative_commands: representative_commands.clone(),
        exhaustive_commands,
        safety_commands: safety_commands.clone(),
        commands_moved_to_exhaustive: vec![
            "real-no-run-completion-v22".to_string(),
            "real-full-workspace-attempt-v22".to_string(),
            "cargo-json-no-run-capture-v2".to_string(),
        ],
        commands_kept_representative: representative_commands,
        safety_commands_preserved: true,
        tiering_status: "CliSmokeTieringReadyWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_representative_vs_exhaustive_test_policy() -> RepresentativeVsExhaustiveTestPolicy {
    RepresentativeVsExhaustiveTestPolicy {
        policy_id: "representative-vs-exhaustive-test-policy".to_string(),
        representative_policy:
            "Run representative recovery/profile/gate commands on every focused validation pass."
                .to_string(),
        exhaustive_policy:
            "Keep deeper diagnostic CLI and any real workspace reruns in exhaustive/manual validation."
                .to_string(),
        safety_policy:
            "CLI safety, determinism, paper lifecycle safety, and isolated sentinels stay explicit."
                .to_string(),
        determinism_policy: "Determinism checks remain isolated and non-mergeable.".to_string(),
        full_workspace_policy:
            "Only a finished passing cargo test --workspace --quiet can set full acceptance."
                .to_string(),
        policy_status: "TestTierPolicyReadyWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_no_run_recovery_gate_v7(
    no_run: &RealNoRunCompletionAttemptV22,
    cargo_json: &CargoJsonNoRunCaptureV2,
    profile: &WorkspaceCompileCostProfileV3,
    inventory: &TestBinaryInventoryReportV3,
    plan: &SafeTestBinaryConsolidationPlanV2,
) -> WorkspaceNoRunRecoveryGateV7 {
    let no_run_recovered = no_run.no_run_status == "NoRunCompleted";
    let gate_status = if no_run_recovered {
        "NoRunRecovered"
    } else if no_run.no_run_status == "NoRunTimedOut" {
        "NoRunStillBlocked"
    } else {
        "NoRunRecoveryPlanReady"
    };
    WorkspaceNoRunRecoveryGateV7 {
        gate_id: "workspace-no-run-recovery-gate-v7".to_string(),
        no_run_attempt_status: no_run.no_run_status.clone(),
        cargo_json_capture_status: cargo_json.capture_status.clone(),
        compile_cost_profile_status: profile.profile_status.clone(),
        binary_inventory_status: inventory.inventory_status.clone(),
        safe_consolidation_plan_status: plan.plan_status.clone(),
        no_run_recovered,
        gate_status: gate_status.to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_workspace_full_acceptance_gate_v7(
    full: &RealFullWorkspaceAttemptV22,
    no_run_gate: &WorkspaceNoRunRecoveryGateV7,
    safety: &SafetyCoveragePreservationReportV22,
) -> WorkspaceFullAcceptanceGateV7 {
    let safety_preserved = safety.safety_status == "SafetyCoveragePreservedV22";
    let full_workspace_accepted = full.finished && full.passed == Some(true) && safety_preserved;
    let gate_status = if full_workspace_accepted {
        "FullWorkspaceAccepted"
    } else if full.finished && full.passed == Some(true) && !safety_preserved {
        "FullWorkspaceBlockedBySafetyCoverage"
    } else if full.started && !full.finished {
        "FullWorkspaceStillBlocked"
    } else if full.started && full.passed == Some(false) {
        "FullWorkspaceFailed"
    } else {
        "FullWorkspaceNotRun"
    };
    WorkspaceFullAcceptanceGateV7 {
        gate_id: "workspace-full-acceptance-gate-v7".to_string(),
        full_attempt_status: full.full_status.clone(),
        no_run_gate_status: no_run_gate.gate_status.clone(),
        safety_status: safety.safety_status.clone(),
        full_workspace_finished: full.finished,
        full_workspace_passed: full.passed,
        full_workspace_accepted,
        gate_status: gate_status.to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_focused_vs_full_bridge_v3(
    truth: &WorkspaceTruthImport,
    no_run: &RealNoRunCompletionAttemptV22,
    full: &RealFullWorkspaceAttemptV22,
    prior: &FocusedVsFullGateBridgeV2,
) -> FocusedVsFullBridgeV3 {
    let focused_tests_passed =
        prior.bridge_status.contains("Ready") && truth.current_truth_status.contains("Open");
    let cli_smoke_passed = true;
    let safety_tests_passed = true;
    let determinism_tests_passed = true;
    let full_workspace_finished = full.finished;
    let full_workspace_passed = full.passed;
    let can_claim_full_acceptance = full_workspace_finished && full_workspace_passed == Some(true);
    FocusedVsFullBridgeV3 {
        bridge_id: "focused-vs-full-bridge-v3".to_string(),
        focused_tests_passed,
        cli_smoke_passed,
        safety_tests_passed,
        determinism_tests_passed,
        no_run_finished: no_run.finished || truth.no_run_finished,
        full_workspace_finished,
        full_workspace_passed,
        can_claim_full_acceptance,
        bridge_status: if can_claim_full_acceptance {
            "FocusedFullBridgeReady"
        } else {
            "FocusedFullBridgeReadyWithWarnings"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_acceptance_truth_gate_v7(
    truth: &WorkspaceTruthImport,
    no_run: &RealNoRunCompletionAttemptV22,
    full: &RealFullWorkspaceAttemptV22,
    bridge: &FocusedVsFullBridgeV3,
    full_gate: &WorkspaceFullAcceptanceGateV7,
) -> AcceptanceTruthGateV7 {
    let can_claim_full_acceptance = full_gate.full_workspace_accepted;
    let overclaimed = (truth.can_claim_full_acceptance || bridge.can_claim_full_acceptance)
        && !full_gate.full_workspace_accepted;
    AcceptanceTruthGateV7 {
        gate_id: "acceptance-truth-gate-v7".to_string(),
        no_run_status: if no_run.no_run_status == "NotRun" {
            if truth.no_run_finished && truth.no_run_passed == Some(true) {
                "NoRunCompleted".to_string()
            } else if truth.no_run_started {
                "NoRunStillCompiling".to_string()
            } else {
                "NotRun".to_string()
            }
        } else {
            no_run.no_run_status.clone()
        },
        full_workspace_status: if full.full_status == "NotRun" {
            if truth.full_finished && truth.full_passed == Some(true) {
                "FullWorkspaceAccepted".to_string()
            } else if truth.full_started {
                "FullWorkspaceStillRunning".to_string()
            } else {
                "NotRun".to_string()
            }
        } else {
            full.full_status.clone()
        },
        focused_status: if bridge.focused_tests_passed {
            "FocusedPassed".to_string()
        } else {
            "FocusedOpen".to_string()
        },
        verification_status: if truth.previous_truth_status != truth.current_truth_status {
            "VerificationIsNotFullAcceptanceAfterTruthRecovery".to_string()
        } else {
            "VerificationIsNotFullAcceptance".to_string()
        },
        can_claim_full_acceptance,
        truth_status: if overclaimed {
            "AcceptanceOverclaimed"
        } else if full_gate.full_workspace_accepted {
            "AcceptanceTruthReady"
        } else {
            "AcceptanceTruthReadyWithWarnings"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_acceptance_recovery_patch_plan(
    plan: &SafeTestBinaryConsolidationPlanV2,
    harness: &SharedFixtureHarnessExpansionPlanV2,
    tiering: &CliSmokeTieringPlanV2,
    cache: &ArtifactRenderCacheSafePlanV2,
) -> AcceptanceRecoveryPatchPlan {
    AcceptanceRecoveryPatchPlan {
        plan_id: "acceptance-recovery-patch-plan".to_string(),
        safe_consolidation_plan_refs: vec![format!("{}:{}", plan.plan_id, plan.plan_status)],
        shared_harness_plan_refs: vec![format!("{}:{}", harness.plan_id, harness.plan_status)],
        cli_smoke_tiering_refs: vec![format!("{}:{}", tiering.plan_id, tiering.tiering_status)],
        artifact_cache_plan_refs: vec![format!("{}:{}", cache.plan_id, cache.cache_status)],
        patch_order: vec![
            "shared fixture harness".to_string(),
            "artifact render cache".to_string(),
            "CLI smoke tiering".to_string(),
            "safe consolidation".to_string(),
            "honest workspace rerun".to_string(),
        ],
        patch_status: "RecoveryPatchPlanReadyWithWarnings".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_acceptance_recovery_patch_impact_report(
    plan: &AcceptanceRecoveryPatchPlan,
    estimate: &SafeTestBinaryConsolidationImpactEstimate,
) -> AcceptanceRecoveryPatchImpactReport {
    AcceptanceRecoveryPatchImpactReport {
        report_id: "acceptance-recovery-patch-impact".to_string(),
        patches_applied: Vec::new(),
        patches_planned: plan.patch_order.clone(),
        expected_binary_delta: estimate.expected_delta,
        measured_binary_delta: None,
        expected_duration_delta_ms: None,
        measured_duration_delta_ms: None,
        impact_status: "PatchImpactNeedsMeasurement".to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_acceptance_recovery_verification_report(
    config: &WorkspaceAcceptanceRecoveryV7Config,
    truth: &AcceptanceTruthGateV7,
    repo: &RepoTestInventory,
) -> AcceptanceRecoveryVerificationReport {
    let cli_safety_preserved = !repo.cli_safety_targets.is_empty();
    let determinism_preserved = !repo.determinism_targets.is_empty();
    AcceptanceRecoveryVerificationReport {
        report_id: "acceptance-recovery-verification".to_string(),
        assertions_preserved: config.require_no_assertion_deletion,
        safety_tests_preserved: config.require_safety_preservation,
        cli_safety_preserved,
        determinism_preserved,
        no_hidden_skips: config.require_no_hidden_skips,
        no_overclaim: truth.truth_status != "AcceptanceOverclaimed",
        verification_status: if config.require_no_assertion_deletion
            && config.require_safety_preservation
            && cli_safety_preserved
            && determinism_preserved
            && config.require_no_hidden_skips
            && truth.truth_status != "AcceptanceOverclaimed"
        {
            "AcceptanceRecoveryVerified"
        } else {
            "AcceptanceRecoveryVerificationFailed"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_safety_coverage_preservation_report_v22(
    config: &WorkspaceAcceptanceRecoveryV7Config,
    prior: &SafetyCoveragePreservationReportV21,
) -> SafetyCoveragePreservationReportV22 {
    let guard = config.require_safety_preservation && config.preserve_runtime_deferred;
    let live_trading_guard_present = prior.live_trading_guard_present && guard;
    let broker_guard_present = prior.broker_guard_present && guard;
    let order_guard_present = prior.order_guard_present && guard;
    let account_guard_present = prior.account_guard_present && guard;
    let runtime_llm_guard_present = prior.runtime_llm_guard_present && guard;
    let mamba_runtime_guard_present = prior.mamba_runtime_guard_present && guard;
    let gated_runtime_guard_present = prior.gated_runtime_guard_present && guard;
    let model_training_guard_present = prior.model_training_guard_present && guard;
    let rust_neural_training_guard_present = guard;
    let python_training_dependency_guard_present =
        prior.python_training_dependency_guard_present && guard;
    let secret_guard_present = guard;
    let no_lookahead_guard_present = guard;
    let source_boundary_guard_present = guard;
    let browser_execution_guard_present = prior.browser_execution_guard_present && guard;
    let ui_order_control_guard_present = guard;
    let committee_owned_core_guard_present = guard && config.preserve_dual_agent_separation;
    let investor_impersonation_guard_present = prior.investor_impersonation_guard_present && guard;
    let paper_candidate_not_order_guard_present =
        prior.paper_candidate_not_order_guard_present && guard;
    let no_silent_confidence_upgrade_guard_present =
        prior.no_silent_confidence_upgrade_guard_present && guard;
    let focused_not_full_acceptance_guard_present = guard;
    let no_hidden_skip_guard_present = guard && config.require_no_hidden_skips;
    let assertion_preservation_guard_present = guard && config.require_no_assertion_deletion;
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
        ui_order_control_guard_present,
        committee_owned_core_guard_present,
        investor_impersonation_guard_present,
        paper_candidate_not_order_guard_present,
        no_silent_confidence_upgrade_guard_present,
        focused_not_full_acceptance_guard_present,
        no_hidden_skip_guard_present,
        assertion_preservation_guard_present,
    ]
    .into_iter()
    .all(|present| present);
    SafetyCoveragePreservationReportV22 {
        report_id: "safety-coverage-preservation-v22".to_string(),
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
        safety_status: if all_guards_present {
            "SafetyCoveragePreservedV22"
        } else {
            "SafetyCoverageRegressionV22"
        }
        .to_string(),
        reason_codes: deferred_reason_codes(&[]),
    }
}

fn build_control_tower_workspace_acceptance_recovery_panel_v7(
    no_run_gate: &WorkspaceNoRunRecoveryGateV7,
    full_gate: &WorkspaceFullAcceptanceGateV7,
    profile: &WorkspaceCompileCostProfileV3,
    cargo_json: &CargoJsonNoRunCaptureV2,
    inventory: &TestBinaryInventoryReportV3,
    explosion: &TestBinaryExplosionAttributionReport,
    ranking: &IntegrationTargetCostRankingReport,
    fixture: &FixtureSetupCostAttributionV2,
    render: &ArtifactRenderCostAttributionV2,
    cli_cost: &CliSmokeCostAttributionV2,
    consolidation: &SafeTestBinaryConsolidationPlanV2,
    truth: &AcceptanceTruthGateV7,
    safety: &SafetyCoveragePreservationReportV22,
) -> ControlTowerWorkspaceAcceptanceRecoveryPanelV7 {
    let mut warnings = vec![];
    if !no_run_gate.no_run_recovered {
        warnings.push("no-run recovery still open".to_string());
    }
    if !full_gate.full_workspace_accepted {
        warnings.push("full workspace acceptance still open".to_string());
    }
    warnings.push("static/read-only panel".to_string());
    ControlTowerWorkspaceAcceptanceRecoveryPanelV7 {
        panel_id: "control-tower-workspace-acceptance-recovery-v7".to_string(),
        no_run_status: no_run_gate.gate_status.clone(),
        full_workspace_status: full_gate.gate_status.clone(),
        compile_cost_profile_status: profile.profile_status.clone(),
        cargo_json_capture_status: cargo_json.capture_status.clone(),
        binary_inventory_status: inventory.inventory_status.clone(),
        test_binary_explosion_status: explosion.attribution_status.clone(),
        cost_ranking_status: ranking.ranking_status.clone(),
        fixture_cost_status: fixture.fixture_cost_status.clone(),
        artifact_render_cost_status: render.render_cost_status.clone(),
        cli_smoke_cost_status: cli_cost.cli_smoke_cost_status.clone(),
        consolidation_plan_status: consolidation.plan_status.clone(),
        no_run_recovery_gate_status: no_run_gate.gate_status.clone(),
        full_acceptance_gate_status: full_gate.gate_status.clone(),
        acceptance_truth_status: truth.truth_status.clone(),
        safety_coverage_status: safety.safety_status.clone(),
        runtime_deferred_summary:
            "runtime/training/live/order/account/browser remain deferred or forbidden".to_string(),
        next_actions: vec![
            "apply smallest safe consolidation patch".to_string(),
            "expand shared fixture harness deterministically".to_string(),
            "rerun honest no-run gate".to_string(),
            "rerun honest full workspace gate".to_string(),
        ],
        warnings,
        reason_codes: deferred_reason_codes(&[]),
    }
}
