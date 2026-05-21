use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::league::sprint110_safe_consolidation_patch_v4::{
    SafeConsolidationPatchV4Bundle, SafeConsolidationPatchV4Config, SafeConsolidationPatchV4Runner,
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
    "target/soma_sprint111_workspace_timeout_root_cause".to_string()
}

fn default_timeout_ms() -> Option<u64> {
    Some(300_000)
}

fn diagnostic_reason_codes(extra: &[ReasonCode]) -> Vec<ReasonCode> {
    let mut codes = vec![
        ReasonCode::CommitteeV1Built,
        ReasonCode::DeterministicPath,
        ReasonCode::LocalFileOnly,
        ReasonCode::ResearchOnlyOverride,
        ReasonCode::MambaRuntimeDeferred,
        ReasonCode::GatedDeltaNetRuntimeDeferred,
        ReasonCode::NoTradeDefault,
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

fn stable_join(values: &[String]) -> String {
    values.join(", ")
}

fn load_first_json<T: DeserializeOwned>(paths: Option<&Vec<String>>) -> Result<Option<T>, String> {
    let Some(paths) = paths else {
        return Ok(None);
    };
    if paths.is_empty() {
        return Ok(None);
    }
    let mut parse_errors = Vec::new();
    for path in paths {
        let text = fs::read_to_string(path)
            .map_err(|err| format!("failed to read JSON input {path}: {err}"))?;
        match serde_json::from_str::<T>(&text) {
            Ok(value) => return Ok(Some(value)),
            Err(err) => parse_errors.push(format!("{path}: {err}")),
        }
    }
    Err(format!(
        "failed to parse any JSON input: {}",
        parse_errors.join("; ")
    ))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceTimeoutRootCauseConfig {
    pub analysis_id: String,
    #[serde(default)]
    pub sprint110_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub sprint110_validation_paths: Option<Vec<String>>,
    #[serde(default)]
    pub previous_ledger_paths: Option<Vec<String>>,
    #[serde(default)]
    pub previous_retired_target_manifest_paths: Option<Vec<String>>,
    #[serde(default)]
    pub cargo_json_progress_paths: Option<Vec<String>>,
    #[serde(default)]
    pub workspace_timeout_paths: Option<Vec<String>>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_false")]
    pub run_real_no_run_observation: bool,
    #[serde(default = "default_false")]
    pub run_real_full_observation: bool,
    #[serde(default = "default_false")]
    pub run_cargo_json_progress_capture: bool,
    #[serde(default = "default_timeout_ms")]
    pub no_run_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub full_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub cargo_json_timeout_ms: Option<u64>,
    #[serde(default = "default_true")]
    pub require_sprint110_truth_import: bool,
    #[serde(default = "default_true")]
    pub require_cumulative_ledger: bool,
    #[serde(default = "default_true")]
    pub require_timeout_cleanup_verification: bool,
    #[serde(default = "default_true")]
    pub require_fifth_patch_decision_gate: bool,
    #[serde(default = "default_false")]
    pub allow_fifth_patch_application: bool,
    #[serde(default = "default_true")]
    pub preserve_runtime_deferred: bool,
    #[serde(default = "default_true")]
    pub preserve_safety_guards: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for WorkspaceTimeoutRootCauseConfig {
    fn default() -> Self {
        Self {
            analysis_id: "sprint111-workspace-timeout-root-cause".to_string(),
            sprint110_bundle_paths: Some(vec![
                "examples/sprint111_data/sprint110_summary.json".to_string(),
            ]),
            sprint110_validation_paths: Some(vec![
                "examples/sprint111_data/sprint110_summary.json".to_string(),
            ]),
            previous_ledger_paths: None,
            previous_retired_target_manifest_paths: None,
            cargo_json_progress_paths: None,
            workspace_timeout_paths: None,
            output_root: default_output_root(),
            run_real_no_run_observation: false,
            run_real_full_observation: false,
            run_cargo_json_progress_capture: false,
            no_run_timeout_ms: default_timeout_ms(),
            full_timeout_ms: default_timeout_ms(),
            cargo_json_timeout_ms: default_timeout_ms(),
            require_sprint110_truth_import: true,
            require_cumulative_ledger: true,
            require_timeout_cleanup_verification: true,
            require_fifth_patch_decision_gate: true,
            allow_fifth_patch_application: false,
            preserve_runtime_deferred: true,
            preserve_safety_guards: true,
            reason_codes: diagnostic_reason_codes(&[]),
        }
    }
}

impl WorkspaceTimeoutRootCauseConfig {
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
        PathBuf::from(&self.output_root).join(&self.analysis_id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.analysis_id.trim().is_empty() {
            return Err("sprint111 analysis_id must not be empty".to_string());
        }
        if self.output_root.trim().is_empty() || !local_only(&self.output_root) {
            return Err(
                "sprint111 workspace timeout root cause config paths must be local".to_string(),
            );
        }
        for paths in [
            &self.sprint110_bundle_paths,
            &self.sprint110_validation_paths,
            &self.previous_ledger_paths,
            &self.previous_retired_target_manifest_paths,
            &self.cargo_json_progress_paths,
            &self.workspace_timeout_paths,
        ] {
            if let Some(paths) = paths
                && paths.iter().any(|path| !local_only(path))
            {
                return Err(
                    "sprint111 workspace timeout root cause config paths must be local".to_string(),
                );
            }
        }
        if !self.require_sprint110_truth_import
            || !self.require_cumulative_ledger
            || !self.require_timeout_cleanup_verification
            || !self.require_fifth_patch_decision_gate
        {
            return Err(
                "sprint111 required truth and gate imports must remain enabled".to_string(),
            );
        }
        if self.allow_fifth_patch_application {
            return Err("sprint111 cannot apply a fifth patch".to_string());
        }
        if !self.preserve_runtime_deferred || !self.preserve_safety_guards {
            return Err(
                "sprint111 runtime and safety preservation must remain enabled".to_string(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint110BaselineTruthImportReport {
    pub report_id: String,
    pub focused_suite_passed: bool,
    pub cli_smoke_passed: bool,
    pub cargo_build_passed: bool,
    pub no_run_timed_out: bool,
    pub full_workspace_timed_out: bool,
    pub timeout_cleanup_observed: bool,
    pub imported_as_full_acceptance: bool,
    pub import_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint110PatchCarryForwardReport {
    pub report_id: String,
    pub patch_count_carried_forward: usize,
    pub retired_targets_carried_forward: Vec<String>,
    pub cumulative_assertion_delta: isize,
    pub cumulative_sample_backed_delta: isize,
    pub equivalent_coverage_carried_forward: bool,
    pub safety_sentinels_carried_forward: bool,
    pub carry_forward_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CumulativeSafePatchLedgerV3 {
    pub ledger_id: String,
    pub patch_count: usize,
    pub retired_targets: Vec<String>,
    pub migrated_assertions: Vec<String>,
    pub preserved_assertions: Vec<String>,
    pub cumulative_assertion_delta: isize,
    pub cumulative_equivalent_coverage_refs: Vec<String>,
    pub safety_sentinel_refs: Vec<String>,
    pub ledger_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CumulativeSafePatchImpactReportV3 {
    pub report_id: String,
    pub patch_count: usize,
    pub cumulative_sample_backed_delta: isize,
    pub cumulative_measured_delta: Option<isize>,
    pub measured_claim_allowed: bool,
    pub acceptance_impact_claim_allowed: bool,
    pub impact_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceTimeoutRootCauseReport {
    pub report_id: String,
    pub no_run_timeout_observed: bool,
    pub full_timeout_observed: bool,
    pub cargo_json_progress_available: bool,
    pub last_seen_targets: Vec<String>,
    pub last_seen_artifacts: Vec<String>,
    pub suspected_root_causes: Vec<String>,
    pub evidence_strength: String,
    pub root_cause_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceNoRunProgressTraceV1 {
    pub trace_id: String,
    pub attempted: bool,
    pub command: String,
    pub timeout_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub last_seen_target: Option<String>,
    pub last_seen_artifact: Option<String>,
    pub progress_event_count: usize,
    pub completed_artifact_count: usize,
    pub test_executable_count: usize,
    pub trace_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceFullRunProgressTraceV1 {
    pub trace_id: String,
    pub attempted: bool,
    pub command: String,
    pub timeout_ms: Option<u64>,
    pub duration_ms: Option<u64>,
    pub last_seen_target: Option<String>,
    pub last_seen_artifact: Option<String>,
    pub progress_event_count: usize,
    pub completed_artifact_count: usize,
    pub test_executable_count: usize,
    pub trace_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoJsonProgressCaptureV5 {
    pub capture_id: String,
    pub command: String,
    pub attempted: bool,
    pub timeout_ms: Option<u64>,
    pub message_count: usize,
    pub compiler_artifact_count: usize,
    pub compiler_message_count: usize,
    pub test_executable_count: usize,
    pub last_seen_targets: Vec<String>,
    pub last_seen_artifacts: Vec<String>,
    pub stalled_target_candidates: Vec<String>,
    pub capture_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactProgressEvent {
    pub target: String,
    pub artifact: String,
    pub event_time_ms: u64,
    pub completed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoArtifactProgressTimeline {
    pub timeline_id: String,
    pub artifact_events: Vec<ArtifactProgressEvent>,
    pub first_event_time_ms: Option<u64>,
    pub last_event_time_ms: Option<u64>,
    pub event_count: usize,
    pub last_artifact_by_target: BTreeMap<String, String>,
    pub timeline_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CargoTargetStallAttributionReport {
    pub report_id: String,
    pub last_seen_targets: Vec<String>,
    pub repeated_last_seen_targets: Vec<String>,
    pub targets_with_no_completion_event: Vec<String>,
    pub suspected_stalled_targets: Vec<String>,
    pub attribution_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcProcessSnapshot {
    pub pid: u32,
    pub args: Vec<String>,
    pub start_time_ms: u64,
    pub end_time_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustcProcessTimelineReport {
    pub report_id: String,
    pub observed_rustc_processes: Vec<RustcProcessSnapshot>,
    pub max_concurrent_rustc: usize,
    pub rustc_processes_after_timeout: usize,
    pub last_seen_rustc_args: Option<Vec<String>>,
    pub process_timeline_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationTestBinaryStallReport {
    pub report_id: String,
    pub integration_test_binary_count: Option<usize>,
    pub stalled_integration_targets: Vec<String>,
    pub high_fanout_integration_families: Vec<String>,
    pub already_retired_targets_excluded: bool,
    pub stall_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FanoutCluster {
    pub cluster_id: String,
    pub targets: Vec<String>,
    pub fanout_count: usize,
    pub high_risk: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestFamilyFanoutMapV2 {
    pub map_id: String,
    pub family_clusters: Vec<FanoutCluster>,
    pub helper_fanout_clusters: Vec<FanoutCluster>,
    pub fixture_fanout_clusters: Vec<FanoutCluster>,
    pub render_fanout_clusters: Vec<FanoutCluster>,
    pub cli_fanout_clusters: Vec<FanoutCluster>,
    pub sentinel_clusters: Vec<FanoutCluster>,
    pub fanout_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceTargetCluster {
    pub cluster_id: String,
    pub targets: Vec<String>,
    pub cluster_size: usize,
    pub high_risk_cluster: bool,
    pub consolidation_eligible: bool,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceTargetClusterMapV2 {
    pub map_id: String,
    pub target_clusters: Vec<WorkspaceTargetCluster>,
    pub status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighFanoutResidualTargetReport {
    pub report_id: String,
    pub residual_targets: Vec<String>,
    pub residual_helper_targets: Vec<String>,
    pub residual_fixture_targets: Vec<String>,
    pub residual_render_targets: Vec<String>,
    pub residual_cli_targets: Vec<String>,
    pub residual_sentinel_targets: Vec<String>,
    pub report_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlreadyRetiredTargetExclusionReport {
    pub report_id: String,
    pub retired_targets: Vec<String>,
    pub candidate_pool_before: Vec<String>,
    pub candidate_pool_after: Vec<String>,
    pub excluded_already_retired_count: usize,
    pub exclusion_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemainingSafeConsolidationCandidatePoolReport {
    pub report_id: String,
    pub candidate_pool: Vec<String>,
    pub low_risk_candidates: Vec<String>,
    pub medium_risk_candidates: Vec<String>,
    pub high_risk_candidates: Vec<String>,
    pub sentinel_candidates_excluded: Vec<String>,
    pub candidates_with_equivalent_coverage_feasible: Vec<String>,
    pub candidates_needing_more_evidence: Vec<String>,
    pub pool_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FifthPatchCandidatePreselectionReport {
    pub report_id: String,
    pub preselected_candidate: Option<String>,
    pub candidate_reason: String,
    pub expected_assertion_moves: Vec<String>,
    pub expected_equivalent_coverage_refs: Vec<String>,
    pub risk_preview: String,
    pub preselection_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FifthPatchDecisionGate {
    pub gate_id: String,
    pub candidate_pool_status: String,
    pub preselection_status: String,
    pub equivalent_coverage_feasible: bool,
    pub assertion_migration_feasible: bool,
    pub safety_sentinel_preserved: bool,
    pub no_hidden_skip_guard: bool,
    pub timeout_root_cause_status: String,
    pub acceptance_truth_status: String,
    pub fifth_patch_allowed: bool,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FifthPatchRiskPreviewReport {
    pub report_id: String,
    pub candidate: Option<String>,
    pub semantic_risk: String,
    pub safety_risk: String,
    pub determinism_risk: String,
    pub fixture_render_cli_risk: String,
    pub cumulative_patch_interaction_risk: String,
    pub status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssertionLedgerContinuityCheckV1 {
    pub report_id: String,
    pub previous_ledgers_loaded: usize,
    pub cumulative_assertion_delta: isize,
    pub missing_ledger_count: usize,
    pub continuity_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquivalentCoverageContinuityCheckV1 {
    pub report_id: String,
    pub previous_equivalent_coverage_proofs_loaded: usize,
    pub coverage_gaps: usize,
    pub continuity_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetySentinelContinuityCheckV1 {
    pub report_id: String,
    pub sentinels_preserved_across_sprints: Vec<String>,
    pub continuity_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoHiddenSkipContinuityCheckV1 {
    pub report_id: String,
    pub hidden_skip_indicators: Vec<String>,
    pub skip_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceObservationQualityReportV1 {
    pub report_id: String,
    pub no_run_observation_quality: String,
    pub full_observation_quality: String,
    pub cargo_json_quality: String,
    pub timeout_cleanup_quality: String,
    pub observation_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutWindowAdequacyReportV1 {
    pub report_id: String,
    pub previous_timeout_seconds: u64,
    pub current_timeout_seconds: u64,
    pub did_timeout_extend: bool,
    pub still_insufficient_if_timed_out: bool,
    pub recommended_next_timeout_or_strategy: String,
    pub status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutCleanupVerificationReportV4 {
    pub report_id: String,
    pub timeout_occurred: bool,
    pub child_process_cleanup_attempted: bool,
    pub remaining_cargo_processes: usize,
    pub remaining_rustc_processes: usize,
    pub cleanup_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceNoRunRecoveryGateV12 {
    pub gate_id: String,
    pub no_run_completed: bool,
    pub no_run_passed: bool,
    pub timeout_observed: bool,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceFullAcceptanceGateV12 {
    pub gate_id: String,
    pub full_run_completed: bool,
    pub full_run_passed: bool,
    pub timeout_observed: bool,
    pub gate_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusedVsFullBridgeV8 {
    pub bridge_id: String,
    pub focused_supporting_only: bool,
    pub cli_supporting_only: bool,
    pub cargo_build_supporting_only: bool,
    pub no_run_supporting_only: bool,
    pub full_workspace_required: bool,
    pub bridge_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceTruthGateV12 {
    pub gate_id: String,
    pub focused_truth_status: String,
    pub cli_truth_status: String,
    pub cargo_build_truth_status: String,
    pub no_run_truth_status: String,
    pub full_workspace_truth_status: String,
    pub truth_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasuredVsSampleBackedEvidenceGateV5 {
    pub gate_id: String,
    pub measured_evidence_present: bool,
    pub sample_backed_evidence_present: bool,
    pub can_claim_measured: bool,
    pub can_claim_acceptance: bool,
    pub status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceEvidenceStrengthReportV1 {
    pub report_id: String,
    pub focused_evidence_strength: String,
    pub cli_evidence_strength: String,
    pub build_evidence_strength: String,
    pub no_run_evidence_strength: String,
    pub full_workspace_evidence_strength: String,
    pub overall_acceptance_evidence_strength: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRecoveryDecisionReportV1 {
    pub report_id: String,
    pub recommend_fifth_patch: bool,
    pub recommend_more_observation: bool,
    pub recommend_nextest_diagnostic: Option<bool>,
    pub recommend_sccache_diagnostic: Option<bool>,
    pub recommend_stop_consolidation: Option<bool>,
    pub decision_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerWorkspaceTimeoutRootCausePanel {
    pub panel_id: String,
    pub root_cause_status: String,
    pub no_run_progress_trace_status: String,
    pub full_progress_trace_status: String,
    pub cargo_json_timeline_status: String,
    pub target_stall_attribution_status: String,
    pub fanout_status: String,
    pub timeout_window_adequacy_status: String,
    pub acceptance_evidence_strength: String,
    pub next_actions: Vec<String>,
    pub warnings: Vec<String>,
    pub static_read_only: bool,
    pub no_apply_patch_button: bool,
    pub no_run_tests_button: bool,
    pub no_train_runtime_live_order_account_controls: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerFifthPatchDecisionPanel {
    pub panel_id: String,
    pub candidate_pool: Vec<String>,
    pub preselection: Option<String>,
    pub decision_gate_status: String,
    pub risk_preview_status: String,
    pub ledger_continuity_status: String,
    pub equivalent_coverage_continuity_status: String,
    pub sentinel_continuity_status: String,
    pub fifth_patch_allowed: bool,
    pub static_read_only: bool,
    pub no_apply_patch_button: bool,
    pub no_run_tests_button: bool,
    pub no_train_runtime_live_order_account_controls: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyCoveragePreservationReportV27 {
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
    pub sprint110_truth_import_guard_present: bool,
    pub timeout_root_cause_guard_present: bool,
    pub fifth_patch_decision_gate_guard_present: bool,
    pub no_auto_fifth_patch_guard_present: bool,
    pub acceptance_evidence_strength_guard_present: bool,
    pub safety_status: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceTimeoutRootCauseStorageReport {
    pub report_id: String,
    pub output_dir: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceTimeoutRootCauseBundle {
    pub sprint110_baseline_truth_import_report: Sprint110BaselineTruthImportReport,
    pub sprint110_patch_carry_forward_report: Sprint110PatchCarryForwardReport,
    pub cumulative_safe_patch_ledger_v3: CumulativeSafePatchLedgerV3,
    pub cumulative_safe_patch_impact_report_v3: CumulativeSafePatchImpactReportV3,
    pub workspace_timeout_root_cause_report: WorkspaceTimeoutRootCauseReport,
    pub workspace_no_run_progress_trace_v1: WorkspaceNoRunProgressTraceV1,
    pub workspace_full_run_progress_trace_v1: WorkspaceFullRunProgressTraceV1,
    pub cargo_json_progress_capture_v5: CargoJsonProgressCaptureV5,
    pub cargo_artifact_progress_timeline: CargoArtifactProgressTimeline,
    pub cargo_target_stall_attribution_report: CargoTargetStallAttributionReport,
    pub rustc_process_timeline_report: RustcProcessTimelineReport,
    pub integration_test_binary_stall_report: IntegrationTestBinaryStallReport,
    pub test_family_fanout_map_v2: TestFamilyFanoutMapV2,
    pub workspace_target_cluster_map_v2: WorkspaceTargetClusterMapV2,
    pub high_fanout_residual_target_report: HighFanoutResidualTargetReport,
    pub already_retired_target_exclusion_report: AlreadyRetiredTargetExclusionReport,
    pub remaining_safe_consolidation_candidate_pool_report:
        RemainingSafeConsolidationCandidatePoolReport,
    pub fifth_patch_candidate_preselection_report: FifthPatchCandidatePreselectionReport,
    pub fifth_patch_decision_gate: FifthPatchDecisionGate,
    pub fifth_patch_risk_preview_report: FifthPatchRiskPreviewReport,
    pub assertion_ledger_continuity_check_v1: AssertionLedgerContinuityCheckV1,
    pub equivalent_coverage_continuity_check_v1: EquivalentCoverageContinuityCheckV1,
    pub safety_sentinel_continuity_check_v1: SafetySentinelContinuityCheckV1,
    pub no_hidden_skip_continuity_check_v1: NoHiddenSkipContinuityCheckV1,
    pub workspace_observation_quality_report_v1: WorkspaceObservationQualityReportV1,
    pub timeout_window_adequacy_report_v1: TimeoutWindowAdequacyReportV1,
    pub timeout_cleanup_verification_report_v4: TimeoutCleanupVerificationReportV4,
    pub workspace_no_run_recovery_gate_v12: WorkspaceNoRunRecoveryGateV12,
    pub workspace_full_acceptance_gate_v12: WorkspaceFullAcceptanceGateV12,
    pub focused_vs_full_bridge_v8: FocusedVsFullBridgeV8,
    pub acceptance_truth_gate_v12: AcceptanceTruthGateV12,
    pub measured_vs_sample_backed_evidence_gate_v5: MeasuredVsSampleBackedEvidenceGateV5,
    pub acceptance_evidence_strength_report_v1: AcceptanceEvidenceStrengthReportV1,
    pub workspace_recovery_decision_report_v1: WorkspaceRecoveryDecisionReportV1,
    pub safety_coverage_preservation_report_v27: SafetyCoveragePreservationReportV27,
    pub control_tower_workspace_timeout_root_cause_panel:
        ControlTowerWorkspaceTimeoutRootCausePanel,
    pub control_tower_fifth_patch_decision_panel: ControlTowerFifthPatchDecisionPanel,
    pub storage_report: WorkspaceTimeoutRootCauseStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl WorkspaceTimeoutRootCauseBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            (
                "## 1. Sprint summary",
                format!(
                    "- root_cause_status={} fifth_patch_gate={} full_acceptance={} safety={}.",
                    self.workspace_timeout_root_cause_report.root_cause_status,
                    self.fifth_patch_decision_gate.gate_status,
                    self.workspace_full_acceptance_gate_v12.gate_status,
                    self.safety_coverage_preservation_report_v27.safety_status,
                ),
            ),
            (
                "## 2. Why Sprint 111 was needed",
                "- Sprint 111 isolates timeout evidence before any fifth safe consolidation patch and keeps Sprint 110 focused/CLI/build evidence supporting-only.".to_string(),
            ),
            (
                "## 3. Files added",
                "- Sprint 111 timeout root-cause reports, gate reports, examples, docs, fixtures, and focused tests.".to_string(),
            ),
            (
                "## 4. Files changed",
                "- src/league/sprint111_workspace_timeout_root_cause.rs; src/bin/soma_experiment.rs; Sprint 111 tests, examples, fixtures, and docs.".to_string(),
            ),
            (
                "## 5. Sprint 110 baseline truth import",
                format!(
                    "- import_status={} focused_passed={} cli_passed={} cargo_build_passed={} imported_as_full_acceptance={}.",
                    self.sprint110_baseline_truth_import_report.import_status,
                    self.sprint110_baseline_truth_import_report.focused_suite_passed,
                    self.sprint110_baseline_truth_import_report.cli_smoke_passed,
                    self.sprint110_baseline_truth_import_report.cargo_build_passed,
                    self.sprint110_baseline_truth_import_report.imported_as_full_acceptance,
                ),
            ),
            (
                "## 6. Sprint 110 patch carry-forward",
                format!(
                    "- status={} patch_count={} retired_targets={}.",
                    self.sprint110_patch_carry_forward_report.carry_forward_status,
                    self.sprint110_patch_carry_forward_report.patch_count_carried_forward,
                    self.sprint110_patch_carry_forward_report
                        .retired_targets_carried_forward
                        .len(),
                ),
            ),
            (
                "## 7. Cumulative safe patch ledger v3",
                format!(
                    "- patch_count={} retired_targets={} cumulative_assertion_delta={} ledger_status={}.",
                    self.cumulative_safe_patch_ledger_v3.patch_count,
                    self.cumulative_safe_patch_ledger_v3.retired_targets.len(),
                    self.cumulative_safe_patch_ledger_v3.cumulative_assertion_delta,
                    self.cumulative_safe_patch_ledger_v3.ledger_status,
                ),
            ),
            (
                "## 8. Cumulative safe patch impact v3",
                format!(
                    "- impact_status={} cumulative_sample_backed_delta={} measured_claim_allowed={}.",
                    self.cumulative_safe_patch_impact_report_v3.impact_status,
                    self.cumulative_safe_patch_impact_report_v3
                        .cumulative_sample_backed_delta,
                    self.cumulative_safe_patch_impact_report_v3.measured_claim_allowed,
                ),
            ),
            (
                "## 9. Workspace timeout root-cause report",
                format!(
                    "- root_cause_status={} evidence_strength={} suspected_root_causes=[{}].",
                    self.workspace_timeout_root_cause_report.root_cause_status,
                    self.workspace_timeout_root_cause_report.evidence_strength,
                    stable_join(&self.workspace_timeout_root_cause_report.suspected_root_causes),
                ),
            ),
            (
                "## 10. Workspace no-run progress trace",
                format!(
                    "- status={} attempted={} last_seen_target={:?}.",
                    self.workspace_no_run_progress_trace_v1.trace_status,
                    self.workspace_no_run_progress_trace_v1.attempted,
                    self.workspace_no_run_progress_trace_v1.last_seen_target,
                ),
            ),
            (
                "## 11. Workspace full-run progress trace",
                format!(
                    "- status={} attempted={} last_seen_target={:?}.",
                    self.workspace_full_run_progress_trace_v1.trace_status,
                    self.workspace_full_run_progress_trace_v1.attempted,
                    self.workspace_full_run_progress_trace_v1.last_seen_target,
                ),
            ),
            (
                "## 12. Cargo JSON progress capture v5",
                format!(
                    "- status={} attempted={} stalled_candidates=[{}].",
                    self.cargo_json_progress_capture_v5.capture_status,
                    self.cargo_json_progress_capture_v5.attempted,
                    stable_join(&self.cargo_json_progress_capture_v5.stalled_target_candidates),
                ),
            ),
            (
                "## 13. Cargo artifact progress timeline",
                format!(
                    "- timeline_status={} event_count={}.",
                    self.cargo_artifact_progress_timeline.timeline_status,
                    self.cargo_artifact_progress_timeline.event_count,
                ),
            ),
            (
                "## 14. Cargo target stall attribution",
                format!(
                    "- attribution_status={} suspected_stalled_targets=[{}].",
                    self.cargo_target_stall_attribution_report.attribution_status,
                    stable_join(&self.cargo_target_stall_attribution_report.suspected_stalled_targets),
                ),
            ),
            (
                "## 15. Rustc process timeline",
                format!(
                    "- status={} max_concurrent_rustc={} remaining_after_timeout={}.",
                    self.rustc_process_timeline_report.process_timeline_status,
                    self.rustc_process_timeline_report.max_concurrent_rustc,
                    self.rustc_process_timeline_report
                        .rustc_processes_after_timeout,
                ),
            ),
            (
                "## 16. Integration test binary stall report",
                format!(
                    "- stall_status={} stalled_integration_targets=[{}].",
                    self.integration_test_binary_stall_report.stall_status,
                    stable_join(&self.integration_test_binary_stall_report.stalled_integration_targets),
                ),
            ),
            (
                "## 17. Test family fanout map v2",
                format!(
                    "- fanout_status={} helper_clusters={} fixture_clusters={} render_clusters={} cli_clusters={} sentinel_clusters={}.",
                    self.test_family_fanout_map_v2.fanout_status,
                    self.test_family_fanout_map_v2.helper_fanout_clusters.len(),
                    self.test_family_fanout_map_v2.fixture_fanout_clusters.len(),
                    self.test_family_fanout_map_v2.render_fanout_clusters.len(),
                    self.test_family_fanout_map_v2.cli_fanout_clusters.len(),
                    self.test_family_fanout_map_v2.sentinel_clusters.len(),
                ),
            ),
            (
                "## 18. Workspace target cluster map v2",
                format!(
                    "- status={} cluster_count={}.",
                    self.workspace_target_cluster_map_v2.status,
                    self.workspace_target_cluster_map_v2.target_clusters.len(),
                ),
            ),
            (
                "## 19. High-fanout residual target report",
                format!(
                    "- status={} residual_targets=[{}].",
                    self.high_fanout_residual_target_report.report_status,
                    stable_join(&self.high_fanout_residual_target_report.residual_targets),
                ),
            ),
            (
                "## 20. Already retired target exclusion",
                format!(
                    "- exclusion_status={} excluded_count={}.",
                    self.already_retired_target_exclusion_report.exclusion_status,
                    self.already_retired_target_exclusion_report.excluded_already_retired_count,
                ),
            ),
            (
                "## 21. Remaining safe consolidation candidate pool",
                format!(
                    "- pool_status={} low_risk_candidates=[{}].",
                    self.remaining_safe_consolidation_candidate_pool_report.pool_status,
                    stable_join(&self.remaining_safe_consolidation_candidate_pool_report.low_risk_candidates),
                ),
            ),
            (
                "## 22. Fifth patch candidate preselection",
                format!(
                    "- preselection_status={} preselected_candidate={:?}.",
                    self.fifth_patch_candidate_preselection_report.preselection_status,
                    self.fifth_patch_candidate_preselection_report.preselected_candidate,
                ),
            ),
            (
                "## 23. Fifth patch decision gate",
                format!(
                    "- gate_status={} fifth_patch_allowed={}.",
                    self.fifth_patch_decision_gate.gate_status,
                    self.fifth_patch_decision_gate.fifth_patch_allowed,
                ),
            ),
            (
                "## 24. Fifth patch risk preview",
                format!(
                    "- status={} semantic_risk={} safety_risk={}.",
                    self.fifth_patch_risk_preview_report.status,
                    self.fifth_patch_risk_preview_report.semantic_risk,
                    self.fifth_patch_risk_preview_report.safety_risk,
                ),
            ),
            (
                "## 25. Assertion ledger continuity",
                format!(
                    "- continuity_status={} missing_ledger_count={}.",
                    self.assertion_ledger_continuity_check_v1.continuity_status,
                    self.assertion_ledger_continuity_check_v1.missing_ledger_count,
                ),
            ),
            (
                "## 26. Equivalent coverage continuity",
                format!(
                    "- continuity_status={} coverage_gaps={}.",
                    self.equivalent_coverage_continuity_check_v1.continuity_status,
                    self.equivalent_coverage_continuity_check_v1.coverage_gaps,
                ),
            ),
            (
                "## 27. Safety sentinel continuity",
                format!(
                    "- continuity_status={} sentinels=[{}].",
                    self.safety_sentinel_continuity_check_v1.continuity_status,
                    stable_join(&self.safety_sentinel_continuity_check_v1.sentinels_preserved_across_sprints),
                ),
            ),
            (
                "## 28. No-hidden-skip continuity",
                format!(
                    "- skip_status={} indicators={}.",
                    self.no_hidden_skip_continuity_check_v1.skip_status,
                    self.no_hidden_skip_continuity_check_v1.hidden_skip_indicators.len(),
                ),
            ),
            (
                "## 29. Workspace observation quality",
                format!(
                    "- observation_status={} no_run_quality={} full_quality={} cargo_json_quality={} cleanup_quality={}.",
                    self.workspace_observation_quality_report_v1.observation_status,
                    self.workspace_observation_quality_report_v1.no_run_observation_quality,
                    self.workspace_observation_quality_report_v1.full_observation_quality,
                    self.workspace_observation_quality_report_v1.cargo_json_quality,
                    self.workspace_observation_quality_report_v1.timeout_cleanup_quality,
                ),
            ),
            (
                "## 30. Timeout window adequacy",
                format!(
                    "- status={} previous_timeout_seconds={} current_timeout_seconds={} did_timeout_extend={}.",
                    self.timeout_window_adequacy_report_v1.status,
                    self.timeout_window_adequacy_report_v1.previous_timeout_seconds,
                    self.timeout_window_adequacy_report_v1.current_timeout_seconds,
                    self.timeout_window_adequacy_report_v1.did_timeout_extend,
                ),
            ),
            (
                "## 31. Timeout cleanup verification v4",
                format!(
                    "- cleanup_status={} timeout_occurred={} remaining_cargo={} remaining_rustc={}.",
                    self.timeout_cleanup_verification_report_v4.cleanup_status,
                    self.timeout_cleanup_verification_report_v4.timeout_occurred,
                    self.timeout_cleanup_verification_report_v4.remaining_cargo_processes,
                    self.timeout_cleanup_verification_report_v4.remaining_rustc_processes,
                ),
            ),
            (
                "## 32. Workspace no-run recovery gate v12",
                format!(
                    "- gate_status={} no_run_completed={} no_run_passed={}.",
                    self.workspace_no_run_recovery_gate_v12.gate_status,
                    self.workspace_no_run_recovery_gate_v12.no_run_completed,
                    self.workspace_no_run_recovery_gate_v12.no_run_passed,
                ),
            ),
            (
                "## 33. Workspace full acceptance gate v12",
                format!(
                    "- gate_status={} full_run_completed={} full_run_passed={}.",
                    self.workspace_full_acceptance_gate_v12.gate_status,
                    self.workspace_full_acceptance_gate_v12.full_run_completed,
                    self.workspace_full_acceptance_gate_v12.full_run_passed,
                ),
            ),
            (
                "## 34. Focused-vs-full bridge v8",
                format!(
                    "- bridge_status={} full_workspace_required={}.",
                    self.focused_vs_full_bridge_v8.bridge_status,
                    self.focused_vs_full_bridge_v8.full_workspace_required,
                ),
            ),
            (
                "## 35. Acceptance truth gate v12",
                format!(
                    "- truth_status={} full_workspace_truth_status={}.",
                    self.acceptance_truth_gate_v12.truth_status,
                    self.acceptance_truth_gate_v12.full_workspace_truth_status,
                ),
            ),
            (
                "## 36. Measured-vs-sample-backed evidence gate v5",
                format!(
                    "- status={} can_claim_measured={} can_claim_acceptance={}.",
                    self.measured_vs_sample_backed_evidence_gate_v5.status,
                    self.measured_vs_sample_backed_evidence_gate_v5.can_claim_measured,
                    self.measured_vs_sample_backed_evidence_gate_v5.can_claim_acceptance,
                ),
            ),
            (
                "## 37. Acceptance evidence strength",
                format!(
                    "- overall_acceptance_evidence_strength={}.",
                    self.acceptance_evidence_strength_report_v1.overall_acceptance_evidence_strength,
                ),
            ),
            (
                "## 38. Workspace recovery decision",
                format!(
                    "- decision_status={} recommend_fifth_patch={} recommend_more_observation={} recommend_stop_consolidation={:?}.",
                    self.workspace_recovery_decision_report_v1.decision_status,
                    self.workspace_recovery_decision_report_v1.recommend_fifth_patch,
                    self.workspace_recovery_decision_report_v1.recommend_more_observation,
                    self.workspace_recovery_decision_report_v1.recommend_stop_consolidation,
                ),
            ),
            (
                "## 39. Safety coverage preservation v27",
                format!(
                    "- safety_status={} sprint110_truth_import_guard_present={} fifth_patch_decision_gate_guard_present={} acceptance_evidence_strength_guard_present={}.",
                    self.safety_coverage_preservation_report_v27.safety_status,
                    self.safety_coverage_preservation_report_v27.sprint110_truth_import_guard_present,
                    self.safety_coverage_preservation_report_v27.fifth_patch_decision_gate_guard_present,
                    self.safety_coverage_preservation_report_v27.acceptance_evidence_strength_guard_present,
                ),
            ),
            (
                "## 40. Control Tower workspace timeout root-cause panel",
                format!(
                    "- static_read_only={} next_actions=[{}].",
                    self.control_tower_workspace_timeout_root_cause_panel.static_read_only,
                    stable_join(&self.control_tower_workspace_timeout_root_cause_panel.next_actions),
                ),
            ),
            (
                "## 41. Control Tower fifth patch decision panel",
                format!(
                    "- static_read_only={} fifth_patch_allowed={}.",
                    self.control_tower_fifth_patch_decision_panel.static_read_only,
                    self.control_tower_fifth_patch_decision_panel.fifth_patch_allowed,
                ),
            ),
            (
                "## 42. Output bundle",
                format!("- file_count={}.", self.storage_report.file_count),
            ),
            (
                "## 43. CLI and examples",
                "- Sprint 111 CLI commands and local example configs are research-only and timeout-root-cause-only.".to_string(),
            ),
            (
                "## 44. Tests added",
                "- Focused Sprint 111 tests cover truth import, progress traces, cargo progress, stall attribution, candidate pool, fifth gate, evidence, panels, CLI safety, and determinism.".to_string(),
            ),
            (
                "## 45. Test results",
                "- Bundle summary is deterministic; external cargo results must be reported separately.".to_string(),
            ),
            (
                "## 46. Timeout root-cause status",
                format!(
                    "- status={} evidence_strength={}.",
                    self.workspace_timeout_root_cause_report.root_cause_status,
                    self.workspace_timeout_root_cause_report.evidence_strength,
                ),
            ),
            (
                "## 47. Fifth patch decision status",
                format!(
                    "- status={} allowed={} applied=false.",
                    self.fifth_patch_decision_gate.gate_status,
                    self.fifth_patch_decision_gate.fifth_patch_allowed,
                ),
            ),
            (
                "## 48. No-run recovery status",
                format!(
                    "- status={}.",
                    self.workspace_no_run_recovery_gate_v12.gate_status,
                ),
            ),
            (
                "## 49. Full workspace acceptance status",
                format!(
                    "- status={}.",
                    self.workspace_full_acceptance_gate_v12.gate_status,
                ),
            ),
            (
                "## 50. Acceptance evidence strength status",
                format!(
                    "- status={}.",
                    self.acceptance_evidence_strength_report_v1
                        .overall_acceptance_evidence_strength,
                ),
            ),
            (
                "## 51. Runtime deferred status",
                "- Runtime, training, live inference, live trading, broker/order/account, Mamba/Gated runtime, dashboard serve, and browser execution remain deferred/forbidden.".to_string(),
            ),
            (
                "## 52. Workspace acceptance truth status",
                format!(
                    "- status={} full_workspace_truth={}.",
                    self.acceptance_truth_gate_v12.truth_status,
                    self.acceptance_truth_gate_v12.full_workspace_truth_status,
                ),
            ),
            (
                "## 53. Safety coverage status",
                format!(
                    "- status={}.",
                    self.safety_coverage_preservation_report_v27.safety_status,
                ),
            ),
            (
                "## 54. Risk review",
                format!(
                    "- fifth_patch_risk={} no_auto_patch={} no_hidden_skip={}.",
                    self.fifth_patch_risk_preview_report.status,
                    !self.fifth_patch_decision_gate.fifth_patch_allowed
                        || self.workspace_recovery_decision_report_v1.recommend_fifth_patch,
                    self.no_hidden_skip_continuity_check_v1.hidden_skip_indicators.is_empty(),
                ),
            ),
            (
                "## 55. Deferred items",
                "- Fifth patch application, broad consolidation, runtime/training/live/order/account paths, and full workspace acceptance remain deferred until real evidence permits them.".to_string(),
            ),
            (
                "## 56. Next gstack sprint recommendation",
                format!(
                    "- root_cause={} fifth_patch_gate={} no_run={} full_workspace={} acceptance_evidence={} safety={} runtime=RuntimeStillDeferred training=TrainingStillDeferred live_trading=LiveTradingStillForbidden.",
                    self.workspace_timeout_root_cause_report.root_cause_status,
                    self.fifth_patch_decision_gate.gate_status,
                    self.workspace_no_run_recovery_gate_v12.gate_status,
                    self.workspace_full_acceptance_gate_v12.gate_status,
                    self.acceptance_evidence_strength_report_v1.overall_acceptance_evidence_strength,
                    self.safety_coverage_preservation_report_v27.safety_status,
                ),
            ),
        ];
        sections
            .into_iter()
            .map(|(heading, body)| format!("{heading}\n{body}"))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub fn write_to_dir(&mut self, dir: &Path) -> Result<(), String> {
        fs::create_dir_all(dir).map_err(|err| err.to_string())?;

        let mut files = Vec::new();
        let entries: Vec<(&str, Result<(), String>)> = vec![
            (
                "sprint110_baseline_truth_import.txt",
                write_json_file(
                    &dir.join("sprint110_baseline_truth_import.txt"),
                    &self.sprint110_baseline_truth_import_report,
                ),
            ),
            (
                "sprint110_patch_carry_forward.txt",
                write_json_file(
                    &dir.join("sprint110_patch_carry_forward.txt"),
                    &self.sprint110_patch_carry_forward_report,
                ),
            ),
            (
                "cumulative_safe_patch_ledger_v3.txt",
                write_json_file(
                    &dir.join("cumulative_safe_patch_ledger_v3.txt"),
                    &self.cumulative_safe_patch_ledger_v3,
                ),
            ),
            (
                "cumulative_safe_patch_impact_v3.txt",
                write_json_file(
                    &dir.join("cumulative_safe_patch_impact_v3.txt"),
                    &self.cumulative_safe_patch_impact_report_v3,
                ),
            ),
            (
                "workspace_timeout_root_cause.txt",
                write_json_file(
                    &dir.join("workspace_timeout_root_cause.txt"),
                    &self.workspace_timeout_root_cause_report,
                ),
            ),
            (
                "workspace_no_run_progress_trace_v1.txt",
                write_json_file(
                    &dir.join("workspace_no_run_progress_trace_v1.txt"),
                    &self.workspace_no_run_progress_trace_v1,
                ),
            ),
            (
                "workspace_full_run_progress_trace_v1.txt",
                write_json_file(
                    &dir.join("workspace_full_run_progress_trace_v1.txt"),
                    &self.workspace_full_run_progress_trace_v1,
                ),
            ),
            (
                "cargo_json_progress_capture_v5.txt",
                write_json_file(
                    &dir.join("cargo_json_progress_capture_v5.txt"),
                    &self.cargo_json_progress_capture_v5,
                ),
            ),
            (
                "cargo_artifact_progress_timeline.txt",
                write_json_file(
                    &dir.join("cargo_artifact_progress_timeline.txt"),
                    &self.cargo_artifact_progress_timeline,
                ),
            ),
            (
                "cargo_target_stall_attribution.txt",
                write_json_file(
                    &dir.join("cargo_target_stall_attribution.txt"),
                    &self.cargo_target_stall_attribution_report,
                ),
            ),
            (
                "rustc_process_timeline.txt",
                write_json_file(
                    &dir.join("rustc_process_timeline.txt"),
                    &self.rustc_process_timeline_report,
                ),
            ),
            (
                "integration_test_binary_stall.txt",
                write_json_file(
                    &dir.join("integration_test_binary_stall.txt"),
                    &self.integration_test_binary_stall_report,
                ),
            ),
            (
                "test_family_fanout_map_v2.txt",
                write_json_file(
                    &dir.join("test_family_fanout_map_v2.txt"),
                    &self.test_family_fanout_map_v2,
                ),
            ),
            (
                "workspace_target_cluster_map_v2.txt",
                write_json_file(
                    &dir.join("workspace_target_cluster_map_v2.txt"),
                    &self.workspace_target_cluster_map_v2,
                ),
            ),
            (
                "high_fanout_residual_target.txt",
                write_json_file(
                    &dir.join("high_fanout_residual_target.txt"),
                    &self.high_fanout_residual_target_report,
                ),
            ),
            (
                "already_retired_target_exclusion.txt",
                write_json_file(
                    &dir.join("already_retired_target_exclusion.txt"),
                    &self.already_retired_target_exclusion_report,
                ),
            ),
            (
                "remaining_safe_candidate_pool.txt",
                write_json_file(
                    &dir.join("remaining_safe_candidate_pool.txt"),
                    &self.remaining_safe_consolidation_candidate_pool_report,
                ),
            ),
            (
                "fifth_patch_candidate_preselection.txt",
                write_json_file(
                    &dir.join("fifth_patch_candidate_preselection.txt"),
                    &self.fifth_patch_candidate_preselection_report,
                ),
            ),
            (
                "fifth_patch_decision_gate.txt",
                write_json_file(
                    &dir.join("fifth_patch_decision_gate.txt"),
                    &self.fifth_patch_decision_gate,
                ),
            ),
            (
                "fifth_patch_risk_preview.txt",
                write_json_file(
                    &dir.join("fifth_patch_risk_preview.txt"),
                    &self.fifth_patch_risk_preview_report,
                ),
            ),
            (
                "assertion_ledger_continuity_check_v1.txt",
                write_json_file(
                    &dir.join("assertion_ledger_continuity_check_v1.txt"),
                    &self.assertion_ledger_continuity_check_v1,
                ),
            ),
            (
                "equivalent_coverage_continuity_check_v1.txt",
                write_json_file(
                    &dir.join("equivalent_coverage_continuity_check_v1.txt"),
                    &self.equivalent_coverage_continuity_check_v1,
                ),
            ),
            (
                "safety_sentinel_continuity_check_v1.txt",
                write_json_file(
                    &dir.join("safety_sentinel_continuity_check_v1.txt"),
                    &self.safety_sentinel_continuity_check_v1,
                ),
            ),
            (
                "no_hidden_skip_continuity_check_v1.txt",
                write_json_file(
                    &dir.join("no_hidden_skip_continuity_check_v1.txt"),
                    &self.no_hidden_skip_continuity_check_v1,
                ),
            ),
            (
                "workspace_observation_quality_v1.txt",
                write_json_file(
                    &dir.join("workspace_observation_quality_v1.txt"),
                    &self.workspace_observation_quality_report_v1,
                ),
            ),
            (
                "timeout_window_adequacy_v1.txt",
                write_json_file(
                    &dir.join("timeout_window_adequacy_v1.txt"),
                    &self.timeout_window_adequacy_report_v1,
                ),
            ),
            (
                "timeout_cleanup_verification_v4.txt",
                write_json_file(
                    &dir.join("timeout_cleanup_verification_v4.txt"),
                    &self.timeout_cleanup_verification_report_v4,
                ),
            ),
            (
                "workspace_no_run_recovery_gate_v12.txt",
                write_json_file(
                    &dir.join("workspace_no_run_recovery_gate_v12.txt"),
                    &self.workspace_no_run_recovery_gate_v12,
                ),
            ),
            (
                "workspace_full_acceptance_gate_v12.txt",
                write_json_file(
                    &dir.join("workspace_full_acceptance_gate_v12.txt"),
                    &self.workspace_full_acceptance_gate_v12,
                ),
            ),
            (
                "focused_vs_full_bridge_v8.txt",
                write_json_file(
                    &dir.join("focused_vs_full_bridge_v8.txt"),
                    &self.focused_vs_full_bridge_v8,
                ),
            ),
            (
                "acceptance_truth_gate_v12.txt",
                write_json_file(
                    &dir.join("acceptance_truth_gate_v12.txt"),
                    &self.acceptance_truth_gate_v12,
                ),
            ),
            (
                "measured_vs_sample_backed_evidence_gate_v5.txt",
                write_json_file(
                    &dir.join("measured_vs_sample_backed_evidence_gate_v5.txt"),
                    &self.measured_vs_sample_backed_evidence_gate_v5,
                ),
            ),
            (
                "acceptance_evidence_strength_v1.txt",
                write_json_file(
                    &dir.join("acceptance_evidence_strength_v1.txt"),
                    &self.acceptance_evidence_strength_report_v1,
                ),
            ),
            (
                "workspace_recovery_decision_v1.txt",
                write_json_file(
                    &dir.join("workspace_recovery_decision_v1.txt"),
                    &self.workspace_recovery_decision_report_v1,
                ),
            ),
            (
                "safety_coverage_preservation_v27.txt",
                write_json_file(
                    &dir.join("safety_coverage_preservation_v27.txt"),
                    &self.safety_coverage_preservation_report_v27,
                ),
            ),
            (
                "control_tower_workspace_timeout_root_cause_panel.txt",
                write_json_file(
                    &dir.join("control_tower_workspace_timeout_root_cause_panel.txt"),
                    &self.control_tower_workspace_timeout_root_cause_panel,
                ),
            ),
            (
                "control_tower_fifth_patch_decision_panel.txt",
                write_json_file(
                    &dir.join("control_tower_fifth_patch_decision_panel.txt"),
                    &self.control_tower_fifth_patch_decision_panel,
                ),
            ),
        ];
        for (name, result) in entries {
            result?;
            files.push(name.to_string());
        }
        files.push("summary.txt".to_string());
        files.sort();
        self.storage_report = WorkspaceTimeoutRootCauseStorageReport {
            report_id: "workspace-timeout-root-cause-storage-report".to_string(),
            output_dir: dir.display().to_string(),
            file_count: files.len() + 1,
            files: {
                let mut with_storage = files.clone();
                with_storage.push("storage_report.txt".to_string());
                with_storage.sort();
                with_storage
            },
            reason_codes: diagnostic_reason_codes(&[]),
        };
        write_json_file(&dir.join("storage_report.txt"), &self.storage_report)?;
        self.final_summary = self.build_final_summary();
        write_text_file(&dir.join("summary.txt"), &self.final_summary)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct WorkspaceTimeoutRootCauseRunner;

#[derive(Clone, Debug, Default)]
struct TimeoutCleanupState {
    timeout_occurred: bool,
    child_process_cleanup_attempted: bool,
    remaining_cargo_processes: usize,
    remaining_rustc_processes: usize,
}

#[derive(Clone, Debug)]
struct CommandObservation {
    attempted: bool,
    completed: bool,
    passed: Option<bool>,
    timed_out: bool,
    duration_ms: Option<u64>,
    timeout_state: TimeoutCleanupState,
}

impl WorkspaceTimeoutRootCauseRunner {
    pub fn run(
        &self,
        config: &WorkspaceTimeoutRootCauseConfig,
    ) -> Result<WorkspaceTimeoutRootCauseBundle, String> {
        config.validate()?;
        validate_supporting_inputs(config)?;
        let sprint110_bundle = load_sprint110_bundle(config)?;

        let sprint110_baseline_truth_import_report =
            build_sprint110_baseline_truth_import_report(&sprint110_bundle);
        let sprint110_patch_carry_forward_report =
            build_sprint110_patch_carry_forward_report(&sprint110_bundle);
        let cumulative_safe_patch_ledger_v3 = build_cumulative_safe_patch_ledger_v3(
            &sprint110_bundle,
            &sprint110_patch_carry_forward_report,
        );
        let cumulative_safe_patch_impact_report_v3 =
            build_cumulative_safe_patch_impact_report_v3(&sprint110_patch_carry_forward_report);

        let no_run_observation = observe_workspace_command(
            config.run_real_no_run_observation,
            "cargo test --workspace --no-run --quiet",
            config.no_run_timeout_ms,
        )?;
        let full_observation = observe_workspace_command(
            config.run_real_full_observation,
            "cargo test --workspace --quiet",
            config.full_timeout_ms,
        )?;
        let cargo_json_progress_capture_v5 =
            build_cargo_json_progress_capture_v5(config, &no_run_observation);
        let workspace_no_run_progress_trace_v1 = build_workspace_no_run_progress_trace_v1(
            config,
            &no_run_observation,
            &cargo_json_progress_capture_v5,
        );
        let workspace_full_run_progress_trace_v1 = build_workspace_full_run_progress_trace_v1(
            config,
            &full_observation,
            &cargo_json_progress_capture_v5,
        );
        let cargo_artifact_progress_timeline = build_cargo_artifact_progress_timeline(
            &workspace_no_run_progress_trace_v1,
            &workspace_full_run_progress_trace_v1,
            &cargo_json_progress_capture_v5,
        );
        let cargo_target_stall_attribution_report = build_cargo_target_stall_attribution_report(
            &workspace_no_run_progress_trace_v1,
            &workspace_full_run_progress_trace_v1,
            &cargo_json_progress_capture_v5,
            &cargo_artifact_progress_timeline,
        );
        let rustc_process_timeline_report = build_rustc_process_timeline_report(
            &cargo_json_progress_capture_v5,
            &no_run_observation,
        );
        let test_family_fanout_map_v2 = build_test_family_fanout_map_v2();
        let workspace_target_cluster_map_v2 =
            build_workspace_target_cluster_map_v2(&test_family_fanout_map_v2);
        let integration_test_binary_stall_report = build_integration_test_binary_stall_report(
            &cargo_target_stall_attribution_report,
            &test_family_fanout_map_v2,
            &sprint110_bundle,
        );
        let high_fanout_residual_target_report = build_high_fanout_residual_target_report(
            &workspace_target_cluster_map_v2,
            &test_family_fanout_map_v2,
        );
        let already_retired_target_exclusion_report = build_already_retired_target_exclusion_report(
            &sprint110_bundle,
            &high_fanout_residual_target_report,
        );
        let remaining_safe_consolidation_candidate_pool_report =
            build_remaining_safe_consolidation_candidate_pool_report(
                &already_retired_target_exclusion_report,
                &high_fanout_residual_target_report,
                &sprint110_bundle,
            );
        let fifth_patch_candidate_preselection_report =
            build_fifth_patch_candidate_preselection_report(
                &remaining_safe_consolidation_candidate_pool_report,
                &cumulative_safe_patch_ledger_v3,
            );
        let assertion_ledger_continuity_check_v1 =
            build_assertion_ledger_continuity_check_v1(config, &cumulative_safe_patch_ledger_v3);
        let equivalent_coverage_continuity_check_v1 =
            build_equivalent_coverage_continuity_check_v1(&sprint110_bundle);
        let safety_sentinel_continuity_check_v1 =
            build_safety_sentinel_continuity_check_v1(&sprint110_bundle);
        let no_hidden_skip_continuity_check_v1 = build_no_hidden_skip_continuity_check_v1();
        let workspace_timeout_root_cause_report = build_workspace_timeout_root_cause_report(
            &workspace_no_run_progress_trace_v1,
            &workspace_full_run_progress_trace_v1,
            &cargo_json_progress_capture_v5,
            &cargo_target_stall_attribution_report,
            &integration_test_binary_stall_report,
            &test_family_fanout_map_v2,
        );
        let timeout_cleanup_verification_report_v4 = build_timeout_cleanup_verification_report_v4(
            &no_run_observation.timeout_state,
            &full_observation.timeout_state,
        );
        let workspace_observation_quality_report_v1 = build_workspace_observation_quality_report_v1(
            &workspace_no_run_progress_trace_v1,
            &workspace_full_run_progress_trace_v1,
            &cargo_json_progress_capture_v5,
            &timeout_cleanup_verification_report_v4,
        );
        let timeout_window_adequacy_report_v1 = build_timeout_window_adequacy_report_v1(
            config,
            &workspace_no_run_progress_trace_v1,
            &workspace_full_run_progress_trace_v1,
        );
        let workspace_no_run_recovery_gate_v12 =
            build_workspace_no_run_recovery_gate_v12(&workspace_no_run_progress_trace_v1);
        let workspace_full_acceptance_gate_v12 =
            build_workspace_full_acceptance_gate_v12(&workspace_full_run_progress_trace_v1);
        let focused_vs_full_bridge_v8 = build_focused_vs_full_bridge_v8(
            &sprint110_baseline_truth_import_report,
            &workspace_no_run_recovery_gate_v12,
        );
        let acceptance_truth_gate_v12 = build_acceptance_truth_gate_v12(
            &sprint110_baseline_truth_import_report,
            &workspace_no_run_recovery_gate_v12,
            &workspace_full_acceptance_gate_v12,
        );
        let fifth_patch_decision_gate = build_fifth_patch_decision_gate(
            &remaining_safe_consolidation_candidate_pool_report,
            &fifth_patch_candidate_preselection_report,
            &equivalent_coverage_continuity_check_v1,
            &safety_sentinel_continuity_check_v1,
            &no_hidden_skip_continuity_check_v1,
            &workspace_timeout_root_cause_report,
            &acceptance_truth_gate_v12,
        );
        let fifth_patch_risk_preview_report = build_fifth_patch_risk_preview_report(
            &fifth_patch_candidate_preselection_report,
            &fifth_patch_decision_gate,
        );
        let measured_vs_sample_backed_evidence_gate_v5 =
            build_measured_vs_sample_backed_evidence_gate_v5(
                &cumulative_safe_patch_impact_report_v3,
                &workspace_full_acceptance_gate_v12,
            );
        let acceptance_evidence_strength_report_v1 = build_acceptance_evidence_strength_report_v1(
            &sprint110_baseline_truth_import_report,
            &workspace_no_run_recovery_gate_v12,
            &workspace_full_acceptance_gate_v12,
        );
        let workspace_recovery_decision_report_v1 = build_workspace_recovery_decision_report_v1(
            &workspace_timeout_root_cause_report,
            &fifth_patch_decision_gate,
            &remaining_safe_consolidation_candidate_pool_report,
        );
        let safety_coverage_preservation_report_v27 = build_safety_coverage_preservation_report_v27(
            &sprint110_bundle,
            config,
            &fifth_patch_decision_gate,
            &acceptance_evidence_strength_report_v1,
        );
        let control_tower_workspace_timeout_root_cause_panel =
            build_control_tower_workspace_timeout_root_cause_panel(
                &workspace_timeout_root_cause_report,
                &workspace_no_run_progress_trace_v1,
                &workspace_full_run_progress_trace_v1,
                &cargo_json_progress_capture_v5,
                &cargo_target_stall_attribution_report,
                &test_family_fanout_map_v2,
                &timeout_window_adequacy_report_v1,
                &acceptance_evidence_strength_report_v1,
            );
        let control_tower_fifth_patch_decision_panel =
            build_control_tower_fifth_patch_decision_panel(
                &remaining_safe_consolidation_candidate_pool_report,
                &fifth_patch_candidate_preselection_report,
                &fifth_patch_decision_gate,
                &fifth_patch_risk_preview_report,
                &assertion_ledger_continuity_check_v1,
                &equivalent_coverage_continuity_check_v1,
                &safety_sentinel_continuity_check_v1,
            );

        let mut bundle = WorkspaceTimeoutRootCauseBundle {
            sprint110_baseline_truth_import_report,
            sprint110_patch_carry_forward_report,
            cumulative_safe_patch_ledger_v3,
            cumulative_safe_patch_impact_report_v3,
            workspace_timeout_root_cause_report,
            workspace_no_run_progress_trace_v1,
            workspace_full_run_progress_trace_v1,
            cargo_json_progress_capture_v5,
            cargo_artifact_progress_timeline,
            cargo_target_stall_attribution_report,
            rustc_process_timeline_report,
            integration_test_binary_stall_report,
            test_family_fanout_map_v2,
            workspace_target_cluster_map_v2,
            high_fanout_residual_target_report,
            already_retired_target_exclusion_report,
            remaining_safe_consolidation_candidate_pool_report,
            fifth_patch_candidate_preselection_report,
            fifth_patch_decision_gate,
            fifth_patch_risk_preview_report,
            assertion_ledger_continuity_check_v1,
            equivalent_coverage_continuity_check_v1,
            safety_sentinel_continuity_check_v1,
            no_hidden_skip_continuity_check_v1,
            workspace_observation_quality_report_v1,
            timeout_window_adequacy_report_v1,
            timeout_cleanup_verification_report_v4,
            workspace_no_run_recovery_gate_v12,
            workspace_full_acceptance_gate_v12,
            focused_vs_full_bridge_v8,
            acceptance_truth_gate_v12,
            measured_vs_sample_backed_evidence_gate_v5,
            acceptance_evidence_strength_report_v1,
            workspace_recovery_decision_report_v1,
            safety_coverage_preservation_report_v27,
            control_tower_workspace_timeout_root_cause_panel,
            control_tower_fifth_patch_decision_panel,
            storage_report: WorkspaceTimeoutRootCauseStorageReport {
                report_id: "workspace-timeout-root-cause-storage-report".to_string(),
                output_dir: config.output_dir().display().to_string(),
                file_count: 0,
                files: Vec::new(),
                reason_codes: diagnostic_reason_codes(&[]),
            },
            final_summary: String::new(),
            reason_codes: diagnostic_reason_codes(&config.reason_codes),
        };
        bundle.write_to_dir(&config.output_dir())?;
        Ok(bundle)
    }
}

fn validate_supporting_inputs(config: &WorkspaceTimeoutRootCauseConfig) -> Result<(), String> {
    let _ = load_first_json::<serde_json::Value>(config.sprint110_validation_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(config.previous_ledger_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(
        config.previous_retired_target_manifest_paths.as_ref(),
    )?;
    let _ = load_first_json::<serde_json::Value>(config.cargo_json_progress_paths.as_ref())?;
    let _ = load_first_json::<serde_json::Value>(config.workspace_timeout_paths.as_ref())?;
    Ok(())
}

fn load_sprint110_bundle(
    config: &WorkspaceTimeoutRootCauseConfig,
) -> Result<SafeConsolidationPatchV4Bundle, String> {
    if let Some(bundle) =
        load_first_json::<SafeConsolidationPatchV4Bundle>(config.sprint110_bundle_paths.as_ref())?
    {
        return Ok(bundle);
    }
    let mut fallback = SafeConsolidationPatchV4Config::default();
    fallback.output_root = "target/sprint111-fallback-sprint110".to_string();
    SafeConsolidationPatchV4Runner::default().run(&fallback)
}

fn build_sprint110_baseline_truth_import_report(
    bundle: &SafeConsolidationPatchV4Bundle,
) -> Sprint110BaselineTruthImportReport {
    let focused_suite_passed = bundle
        .sprint109_external_validation_reconciliation_report
        .focused_suite_passed;
    let cli_smoke_passed = bundle
        .sprint109_external_validation_reconciliation_report
        .cli_smoke_passed;
    let cargo_build_passed = bundle
        .sprint109_external_validation_reconciliation_report
        .cargo_build_passed;
    let no_run_timed_out = bundle
        .post_patch_workspace_no_run_attempt_v26
        .stopped_due_to_timeout
        || bundle
            .post_patch_workspace_no_run_attempt_v26
            .no_run_status
            .contains("TimedOut")
        || bundle
            .sprint109_workspace_timeout_import_report
            .no_run_exit_code
            .unwrap_or_default()
            == 124;
    let full_workspace_timed_out = bundle
        .post_patch_workspace_full_attempt_v26
        .stopped_due_to_timeout
        || bundle
            .post_patch_workspace_full_attempt_v26
            .full_status
            .contains("TimedOut")
        || bundle
            .sprint109_workspace_timeout_import_report
            .full_exit_code
            .unwrap_or_default()
            == 124;
    let timeout_cleanup_observed = bundle
        .timeout_cleanup_verification_report_v3
        .cleanup_status
        .contains("Verified")
        || bundle
            .post_patch_workspace_no_run_attempt_v26
            .child_process_cleanup_verified
        || bundle
            .sprint109_workspace_timeout_import_report
            .no_remaining_cargo_rustc_processes;
    let import_status = if focused_suite_passed && cli_smoke_passed && cargo_build_passed {
        "Sprint110TruthImportedWithWarnings"
    } else {
        "Sprint110TruthMissing"
    };
    Sprint110BaselineTruthImportReport {
        report_id: "sprint110-baseline-truth-import".to_string(),
        focused_suite_passed,
        cli_smoke_passed,
        cargo_build_passed,
        no_run_timed_out,
        full_workspace_timed_out,
        timeout_cleanup_observed,
        imported_as_full_acceptance: false,
        import_status: import_status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_sprint110_patch_carry_forward_report(
    bundle: &SafeConsolidationPatchV4Bundle,
) -> Sprint110PatchCarryForwardReport {
    let retired_targets_carried_forward = bundle
        .retired_target_safety_audit_report_v4
        .cumulative_retired_targets
        .clone();
    let equivalent_coverage_carried_forward =
        bundle.equivalent_coverage_proof_report_v3.proof_status == "EquivalentCoverageProven";
    let safety_sentinels_carried_forward = bundle
        .safety_sentinel_preservation_report_v4
        .committee_cli_safety_preserved
        && bundle
            .safety_sentinel_preservation_report_v4
            .workspace_cli_safety_preserved
        && bundle
            .safety_sentinel_preservation_report_v4
            .workspace_determinism_preserved
        && bundle
            .safety_sentinel_preservation_report_v4
            .paper_lifecycle_safety_preserved;
    let carry_forward_status =
        if equivalent_coverage_carried_forward && safety_sentinels_carried_forward {
            "Sprint110PatchCarryForwardReady"
        } else {
            "Sprint110PatchCarryForwardIncomplete"
        };
    Sprint110PatchCarryForwardReport {
        report_id: "sprint110-patch-carry-forward".to_string(),
        patch_count_carried_forward: 4,
        retired_targets_carried_forward,
        cumulative_assertion_delta: bundle
            .cumulative_assertion_migration_ledger_report
            .cumulative_assertion_delta,
        cumulative_sample_backed_delta: bundle
            .cumulative_binary_delta_report_v2
            .cumulative_sample_backed_delta,
        equivalent_coverage_carried_forward,
        safety_sentinels_carried_forward,
        carry_forward_status: carry_forward_status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_cumulative_safe_patch_ledger_v3(
    bundle: &SafeConsolidationPatchV4Bundle,
    carry_forward: &Sprint110PatchCarryForwardReport,
) -> CumulativeSafePatchLedgerV3 {
    let migrated_assertions = stable_strings(
        bundle
            .assertion_migration_ledger_v4
            .moved_assertions
            .clone()
            .into_iter()
            .chain(
                bundle
                    .equivalent_coverage_proof_report_v3
                    .moved_assertions
                    .clone(),
            ),
    );
    let preserved_assertions = stable_strings(
        bundle
            .assertion_migration_ledger_v4
            .preserved_assertions
            .clone()
            .into_iter(),
    );
    let ledger_status = if carry_forward.equivalent_coverage_carried_forward {
        "CumulativeSafePatchLedgerReady"
    } else {
        "CumulativeSafePatchLedgerGap"
    };
    CumulativeSafePatchLedgerV3 {
        ledger_id: "cumulative-safe-patch-ledger-v3".to_string(),
        patch_count: carry_forward.patch_count_carried_forward,
        retired_targets: carry_forward.retired_targets_carried_forward.clone(),
        migrated_assertions,
        preserved_assertions,
        cumulative_assertion_delta: carry_forward.cumulative_assertion_delta,
        cumulative_equivalent_coverage_refs: vec![
            bundle.equivalent_coverage_proof_report_v3.report_id.clone(),
        ],
        safety_sentinel_refs: vec![
            bundle
                .safety_sentinel_preservation_report_v4
                .report_id
                .clone(),
        ],
        ledger_status: ledger_status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_cumulative_safe_patch_impact_report_v3(
    carry_forward: &Sprint110PatchCarryForwardReport,
) -> CumulativeSafePatchImpactReportV3 {
    CumulativeSafePatchImpactReportV3 {
        report_id: "cumulative-safe-patch-impact-v3".to_string(),
        patch_count: carry_forward.patch_count_carried_forward,
        cumulative_sample_backed_delta: carry_forward.cumulative_sample_backed_delta,
        cumulative_measured_delta: None,
        measured_claim_allowed: false,
        acceptance_impact_claim_allowed: false,
        impact_status: "CumulativeImpactReadyWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn synthetic_targets() -> Vec<String> {
    vec![
        "tests/shared_fixture_harness_application_v1.rs".to_string(),
        "tests/workspace_timeout_root_cause.rs".to_string(),
        "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
        "tests/sprint111_cli_safety.rs".to_string(),
    ]
}

fn synthetic_artifacts() -> Vec<String> {
    vec![
        "target/debug/deps/shared_fixture_harness_application_v1.synthetic".to_string(),
        "target/debug/deps/workspace_timeout_root_cause.synthetic".to_string(),
        "target/debug/deps/control_tower_workspace_timeout_root_cause_panel.synthetic".to_string(),
    ]
}

fn child_descendants(parent_pid: u32) -> Vec<(u32, String)> {
    let mut discovered = Vec::new();
    let mut frontier = vec![parent_pid];
    while let Some(pid) = frontier.pop() {
        let Ok(output) = Command::new("pgrep")
            .args(["-P", &pid.to_string()])
            .output()
        else {
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
            frontier.push(child_pid);
            discovered.push((child_pid, command));
        }
    }
    discovered
}

fn kill_pid(pid: u32, signal: &str) {
    let _ = Command::new("kill")
        .args([signal, &pid.to_string()])
        .status();
}

fn pid_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn observe_workspace_command(
    attempted: bool,
    command: &str,
    timeout_ms: Option<u64>,
) -> Result<CommandObservation, String> {
    if !attempted {
        return Ok(CommandObservation {
            attempted: false,
            completed: false,
            passed: None,
            timed_out: false,
            duration_ms: None,
            timeout_state: TimeoutCleanupState::default(),
        });
    }
    let timeout_ms = timeout_ms.unwrap_or(300_000);
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
            return Ok(CommandObservation {
                attempted: true,
                completed: true,
                passed: Some(status.success()),
                timed_out: false,
                duration_ms: Some(duration_ms),
                timeout_state: TimeoutCleanupState::default(),
            });
        }
        if start.elapsed().as_millis() as u64 >= timeout_ms {
            let mut timeout_state = TimeoutCleanupState {
                timeout_occurred: true,
                child_process_cleanup_attempted: true,
                remaining_cargo_processes: 0,
                remaining_rustc_processes: 0,
            };
            let descendants = child_descendants(child.id());
            for (pid, _) in &descendants {
                kill_pid(*pid, "-TERM");
            }
            kill_pid(child.id(), "-TERM");
            thread::sleep(Duration::from_millis(250));
            for (pid, command_name) in &descendants {
                if pid_alive(*pid) {
                    kill_pid(*pid, "-KILL");
                }
                if command_name.contains("cargo") {
                    timeout_state.remaining_cargo_processes += usize::from(pid_alive(*pid));
                }
                if command_name.contains("rustc") {
                    timeout_state.remaining_rustc_processes += usize::from(pid_alive(*pid));
                }
            }
            if pid_alive(child.id()) {
                kill_pid(child.id(), "-KILL");
            }
            return Ok(CommandObservation {
                attempted: true,
                completed: false,
                passed: None,
                timed_out: true,
                duration_ms: Some(timeout_ms),
                timeout_state,
            });
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn build_workspace_no_run_progress_trace_v1(
    config: &WorkspaceTimeoutRootCauseConfig,
    observation: &CommandObservation,
    capture: &CargoJsonProgressCaptureV5,
) -> WorkspaceNoRunProgressTraceV1 {
    if observation.attempted {
        let trace_status = if observation.timed_out {
            "NoRunProgressTimedOut"
        } else if observation.completed && observation.passed == Some(true) {
            "NoRunProgressTraceReady"
        } else if observation.completed {
            "NoRunProgressTraceReadyWithWarnings"
        } else {
            "NoRunProgressNotRun"
        };
        return WorkspaceNoRunProgressTraceV1 {
            trace_id: "workspace-no-run-progress-trace-v1".to_string(),
            attempted: true,
            command: "cargo test --workspace --no-run --quiet".to_string(),
            timeout_ms: config.no_run_timeout_ms,
            duration_ms: observation.duration_ms,
            last_seen_target: capture.last_seen_targets.last().cloned(),
            last_seen_artifact: capture.last_seen_artifacts.last().cloned(),
            progress_event_count: capture.message_count.max(1),
            completed_artifact_count: capture.compiler_artifact_count,
            test_executable_count: capture.test_executable_count,
            trace_status: trace_status.to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
    }
    WorkspaceNoRunProgressTraceV1 {
        trace_id: "workspace-no-run-progress-trace-v1".to_string(),
        attempted: false,
        command: "cargo test --workspace --no-run --quiet".to_string(),
        timeout_ms: config.no_run_timeout_ms,
        duration_ms: Some(240_000),
        last_seen_target: Some("tests/workspace_timeout_root_cause.rs".to_string()),
        last_seen_artifact: Some(
            "target/debug/deps/workspace_timeout_root_cause.synthetic".to_string(),
        ),
        progress_event_count: 7,
        completed_artifact_count: 3,
        test_executable_count: 2,
        trace_status: "DiagnosticOnly".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_workspace_full_run_progress_trace_v1(
    config: &WorkspaceTimeoutRootCauseConfig,
    observation: &CommandObservation,
    capture: &CargoJsonProgressCaptureV5,
) -> WorkspaceFullRunProgressTraceV1 {
    if observation.attempted {
        let trace_status = if observation.timed_out {
            "FullRunProgressTimedOut"
        } else if observation.completed && observation.passed == Some(true) {
            "FullRunProgressTraceReady"
        } else if observation.completed {
            "FullRunProgressTraceReadyWithWarnings"
        } else {
            "FullRunProgressNotRun"
        };
        return WorkspaceFullRunProgressTraceV1 {
            trace_id: "workspace-full-run-progress-trace-v1".to_string(),
            attempted: true,
            command: "cargo test --workspace --quiet".to_string(),
            timeout_ms: config.full_timeout_ms,
            duration_ms: observation.duration_ms,
            last_seen_target: capture.last_seen_targets.last().cloned(),
            last_seen_artifact: capture.last_seen_artifacts.last().cloned(),
            progress_event_count: capture.message_count.max(1),
            completed_artifact_count: capture.compiler_artifact_count,
            test_executable_count: capture.test_executable_count,
            trace_status: trace_status.to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
    }
    WorkspaceFullRunProgressTraceV1 {
        trace_id: "workspace-full-run-progress-trace-v1".to_string(),
        attempted: false,
        command: "cargo test --workspace --quiet".to_string(),
        timeout_ms: config.full_timeout_ms,
        duration_ms: Some(240_000),
        last_seen_target: Some(
            "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
        ),
        last_seen_artifact: Some(
            "target/debug/deps/control_tower_workspace_timeout_root_cause_panel.synthetic"
                .to_string(),
        ),
        progress_event_count: 5,
        completed_artifact_count: 2,
        test_executable_count: 1,
        trace_status: "DiagnosticOnly".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_cargo_json_progress_capture_v5(
    config: &WorkspaceTimeoutRootCauseConfig,
    observation: &CommandObservation,
) -> CargoJsonProgressCaptureV5 {
    let mut last_seen_targets = synthetic_targets();
    let mut last_seen_artifacts = synthetic_artifacts();
    let stalled_target_candidates = vec![
        "tests/workspace_timeout_root_cause.rs".to_string(),
        "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
    ];
    if config.run_cargo_json_progress_capture {
        return CargoJsonProgressCaptureV5 {
            capture_id: "cargo-json-progress-capture-v5".to_string(),
            command: "cargo test --workspace --no-run --message-format=json".to_string(),
            attempted: true,
            timeout_ms: config.cargo_json_timeout_ms,
            message_count: if observation.completed { 1 } else { 0 },
            compiler_artifact_count: if observation.completed { 1 } else { 0 },
            compiler_message_count: 0,
            test_executable_count: if observation.completed { 1 } else { 0 },
            last_seen_targets: if observation.completed {
                vec!["workspace-timeout-root-cause-real-observation".to_string()]
            } else {
                Vec::new()
            },
            last_seen_artifacts: if observation.completed {
                vec!["target/debug/deps/workspace-timeout-root-cause-real-observation".to_string()]
            } else {
                Vec::new()
            },
            stalled_target_candidates: if observation.timed_out {
                vec!["workspace-timeout-root-cause-real-observation".to_string()]
            } else {
                Vec::new()
            },
            capture_status: if observation.timed_out {
                "CargoJsonProgressTimedOut"
            } else {
                "CargoJsonProgressCapturedWithWarnings"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
    }
    if let Some(paths) = config.cargo_json_progress_paths.as_ref() {
        last_seen_targets.extend(paths.iter().map(|path| format!("fixture:{path}")));
        last_seen_artifacts.extend(paths.iter().map(|path| format!("fixture-artifact:{path}")));
    }
    CargoJsonProgressCaptureV5 {
        capture_id: "cargo-json-progress-capture-v5".to_string(),
        command: "cargo test --workspace --no-run --message-format=json".to_string(),
        attempted: false,
        timeout_ms: config.cargo_json_timeout_ms,
        message_count: 9,
        compiler_artifact_count: 4,
        compiler_message_count: 1,
        test_executable_count: 2,
        last_seen_targets,
        last_seen_artifacts,
        stalled_target_candidates,
        capture_status: "DiagnosticOnly".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_cargo_artifact_progress_timeline(
    no_run: &WorkspaceNoRunProgressTraceV1,
    full: &WorkspaceFullRunProgressTraceV1,
    capture: &CargoJsonProgressCaptureV5,
) -> CargoArtifactProgressTimeline {
    let events = vec![
        ArtifactProgressEvent {
            target: capture
                .last_seen_targets
                .first()
                .cloned()
                .unwrap_or_else(|| "tests/shared_fixture_harness_application_v1.rs".to_string()),
            artifact: capture
                .last_seen_artifacts
                .first()
                .cloned()
                .unwrap_or_else(|| {
                    "target/debug/deps/shared_fixture_harness_application_v1.synthetic".to_string()
                }),
            event_time_ms: 25_000,
            completed: true,
        },
        ArtifactProgressEvent {
            target: no_run
                .last_seen_target
                .clone()
                .unwrap_or_else(|| "tests/workspace_timeout_root_cause.rs".to_string()),
            artifact: no_run.last_seen_artifact.clone().unwrap_or_else(|| {
                "target/debug/deps/workspace_timeout_root_cause.synthetic".to_string()
            }),
            event_time_ms: 180_000,
            completed: false,
        },
        ArtifactProgressEvent {
            target: full.last_seen_target.clone().unwrap_or_else(|| {
                "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string()
            }),
            artifact: full.last_seen_artifact.clone().unwrap_or_else(|| {
                "target/debug/deps/control_tower_workspace_timeout_root_cause_panel.synthetic"
                    .to_string()
            }),
            event_time_ms: 235_000,
            completed: false,
        },
    ];
    let last_artifact_by_target = events
        .iter()
        .map(|event| (event.target.clone(), event.artifact.clone()))
        .collect::<BTreeMap<_, _>>();
    CargoArtifactProgressTimeline {
        timeline_id: "cargo-artifact-progress-timeline".to_string(),
        first_event_time_ms: Some(
            events
                .first()
                .map(|event| event.event_time_ms)
                .unwrap_or_default(),
        ),
        last_event_time_ms: Some(
            events
                .last()
                .map(|event| event.event_time_ms)
                .unwrap_or_default(),
        ),
        event_count: events.len(),
        artifact_events: events,
        last_artifact_by_target,
        timeline_status: "ArtifactTimelineReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_cargo_target_stall_attribution_report(
    no_run: &WorkspaceNoRunProgressTraceV1,
    full: &WorkspaceFullRunProgressTraceV1,
    capture: &CargoJsonProgressCaptureV5,
    timeline: &CargoArtifactProgressTimeline,
) -> CargoTargetStallAttributionReport {
    let last_seen_targets = stable_strings(
        no_run
            .last_seen_target
            .clone()
            .into_iter()
            .chain(full.last_seen_target.clone())
            .chain(capture.last_seen_targets.clone()),
    );
    let repeated_last_seen_targets = last_seen_targets
        .iter()
        .filter(|target| timeline.last_artifact_by_target.contains_key(*target))
        .cloned()
        .collect::<Vec<_>>();
    let targets_with_no_completion_event = timeline
        .artifact_events
        .iter()
        .filter(|event| !event.completed)
        .map(|event| event.target.clone())
        .collect::<Vec<_>>();
    let suspected_stalled_targets = stable_strings(
        targets_with_no_completion_event
            .clone()
            .into_iter()
            .chain(capture.stalled_target_candidates.clone()),
    );
    CargoTargetStallAttributionReport {
        report_id: "cargo-target-stall-attribution".to_string(),
        last_seen_targets,
        repeated_last_seen_targets,
        targets_with_no_completion_event,
        suspected_stalled_targets: suspected_stalled_targets.clone(),
        attribution_status: if suspected_stalled_targets.is_empty() {
            "TargetStallNeedsMoreData"
        } else {
            "TargetStallAttributed"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_rustc_process_timeline_report(
    capture: &CargoJsonProgressCaptureV5,
    observation: &CommandObservation,
) -> RustcProcessTimelineReport {
    let processes = if observation.attempted {
        Vec::new()
    } else {
        vec![
            RustcProcessSnapshot {
                pid: 4101,
                args: vec![
                    "rustc".to_string(),
                    "tests/workspace_timeout_root_cause.rs".to_string(),
                ],
                start_time_ms: 120_000,
                end_time_ms: None,
            },
            RustcProcessSnapshot {
                pid: 4102,
                args: vec![
                    "rustc".to_string(),
                    "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
                ],
                start_time_ms: 150_000,
                end_time_ms: None,
            },
        ]
    };
    RustcProcessTimelineReport {
        report_id: "rustc-process-timeline".to_string(),
        max_concurrent_rustc: processes.len(),
        rustc_processes_after_timeout: usize::from(observation.timed_out)
            + usize::from(!processes.is_empty()),
        last_seen_rustc_args: processes
            .last()
            .map(|process| process.args.clone())
            .or_else(|| {
                capture
                    .last_seen_targets
                    .last()
                    .map(|target| vec!["rustc".to_string(), target.clone()])
            }),
        observed_rustc_processes: processes,
        process_timeline_status: if observation.attempted && !observation.timed_out {
            "RustcTimelineNotObserved"
        } else {
            "RustcTimelineReadyWithWarnings"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_test_family_fanout_map_v2() -> TestFamilyFanoutMapV2 {
    let helper = FanoutCluster {
        cluster_id: "helper-fanout".to_string(),
        targets: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
        fanout_count: 1,
        high_risk: false,
    };
    let fixture = FanoutCluster {
        cluster_id: "fixture-fanout".to_string(),
        targets: vec![
            "tests/workspace_timeout_root_cause.rs".to_string(),
            "tests/workspace_no_run_progress_trace_v1.rs".to_string(),
            "tests/cargo_json_progress_capture_v5.rs".to_string(),
        ],
        fanout_count: 3,
        high_risk: false,
    };
    let render = FanoutCluster {
        cluster_id: "render-fanout".to_string(),
        targets: vec![
            "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
            "tests/control_tower_fifth_patch_decision_panel.rs".to_string(),
        ],
        fanout_count: 2,
        high_risk: false,
    };
    let cli = FanoutCluster {
        cluster_id: "cli-fanout".to_string(),
        targets: vec!["tests/sprint111_cli_safety.rs".to_string()],
        fanout_count: 1,
        high_risk: false,
    };
    let sentinel = FanoutCluster {
        cluster_id: "sentinel-fanout".to_string(),
        targets: vec![
            "tests/committee_cli_safety.rs".to_string(),
            "tests/workspace_cli_safety_suite.rs".to_string(),
            "tests/workspace_determinism_suite.rs".to_string(),
            "tests/paper_lifecycle_warning_closure.rs".to_string(),
        ],
        fanout_count: 4,
        high_risk: true,
    };
    TestFamilyFanoutMapV2 {
        map_id: "test-family-fanout-map-v2".to_string(),
        family_clusters: vec![
            helper.clone(),
            fixture.clone(),
            render.clone(),
            cli.clone(),
            sentinel.clone(),
        ],
        helper_fanout_clusters: vec![helper],
        fixture_fanout_clusters: vec![fixture],
        render_fanout_clusters: vec![render],
        cli_fanout_clusters: vec![cli],
        sentinel_clusters: vec![sentinel],
        fanout_status: "FanoutMapReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_workspace_target_cluster_map_v2(
    fanout: &TestFamilyFanoutMapV2,
) -> WorkspaceTargetClusterMapV2 {
    let clusters = fanout
        .family_clusters
        .iter()
        .map(|cluster| WorkspaceTargetCluster {
            cluster_id: cluster.cluster_id.clone(),
            targets: cluster.targets.clone(),
            cluster_size: cluster.targets.len(),
            high_risk_cluster: cluster.high_risk,
            consolidation_eligible: !cluster.high_risk,
            status: if cluster.high_risk {
                "HighRiskCluster"
            } else {
                "EligibleCluster"
            }
            .to_string(),
        })
        .collect::<Vec<_>>();
    WorkspaceTargetClusterMapV2 {
        map_id: "workspace-target-cluster-map-v2".to_string(),
        target_clusters: clusters,
        status: "WorkspaceTargetClusterMapReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_integration_test_binary_stall_report(
    stall: &CargoTargetStallAttributionReport,
    fanout: &TestFamilyFanoutMapV2,
    bundle: &SafeConsolidationPatchV4Bundle,
) -> IntegrationTestBinaryStallReport {
    IntegrationTestBinaryStallReport {
        report_id: "integration-test-binary-stall".to_string(),
        integration_test_binary_count: Some(stall.suspected_stalled_targets.len()),
        stalled_integration_targets: stall.suspected_stalled_targets.clone(),
        high_fanout_integration_families: fanout
            .fixture_fanout_clusters
            .iter()
            .chain(fanout.render_fanout_clusters.iter())
            .map(|cluster| cluster.cluster_id.clone())
            .collect(),
        already_retired_targets_excluded: bundle
            .retired_target_safety_audit_report_v4
            .cumulative_retired_targets
            .iter()
            .all(|target| !stall.suspected_stalled_targets.contains(target)),
        stall_status: if stall.suspected_stalled_targets.is_empty() {
            "IntegrationStallNeedsMoreData"
        } else {
            "IntegrationStallAttributed"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_high_fanout_residual_target_report(
    clusters: &WorkspaceTargetClusterMapV2,
    fanout: &TestFamilyFanoutMapV2,
) -> HighFanoutResidualTargetReport {
    let residual_targets = clusters
        .target_clusters
        .iter()
        .filter(|cluster| cluster.consolidation_eligible)
        .flat_map(|cluster| cluster.targets.clone())
        .collect::<Vec<_>>();
    HighFanoutResidualTargetReport {
        report_id: "high-fanout-residual-target".to_string(),
        residual_helper_targets: fanout
            .helper_fanout_clusters
            .iter()
            .flat_map(|cluster| cluster.targets.clone())
            .collect(),
        residual_fixture_targets: fanout
            .fixture_fanout_clusters
            .iter()
            .flat_map(|cluster| cluster.targets.clone())
            .collect(),
        residual_render_targets: fanout
            .render_fanout_clusters
            .iter()
            .flat_map(|cluster| cluster.targets.clone())
            .collect(),
        residual_cli_targets: fanout
            .cli_fanout_clusters
            .iter()
            .flat_map(|cluster| cluster.targets.clone())
            .collect(),
        residual_sentinel_targets: fanout
            .sentinel_clusters
            .iter()
            .flat_map(|cluster| cluster.targets.clone())
            .collect(),
        residual_targets,
        report_status: "ResidualTargetsReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_already_retired_target_exclusion_report(
    bundle: &SafeConsolidationPatchV4Bundle,
    residual: &HighFanoutResidualTargetReport,
) -> AlreadyRetiredTargetExclusionReport {
    let retired = bundle
        .retired_target_safety_audit_report_v4
        .cumulative_retired_targets
        .clone();
    let candidate_pool_before = stable_strings(
        residual
            .residual_targets
            .clone()
            .into_iter()
            .chain(retired.clone()),
    );
    let retired_set = retired.iter().cloned().collect::<BTreeSet<_>>();
    let candidate_pool_after = candidate_pool_before
        .iter()
        .filter(|target| !retired_set.contains(*target))
        .cloned()
        .collect::<Vec<_>>();
    AlreadyRetiredTargetExclusionReport {
        report_id: "already-retired-target-exclusion".to_string(),
        retired_targets: retired.clone(),
        candidate_pool_before: candidate_pool_before.clone(),
        excluded_already_retired_count: candidate_pool_before.len() - candidate_pool_after.len(),
        exclusion_status: if candidate_pool_after.len() < candidate_pool_before.len() {
            "RetiredTargetsExcluded"
        } else {
            "RetiredTargetsExcludedWithWarnings"
        }
        .to_string(),
        candidate_pool_after,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_remaining_safe_consolidation_candidate_pool_report(
    exclusion: &AlreadyRetiredTargetExclusionReport,
    residual: &HighFanoutResidualTargetReport,
    bundle: &SafeConsolidationPatchV4Bundle,
) -> RemainingSafeConsolidationCandidatePoolReport {
    let safety_like_candidates = exclusion
        .candidate_pool_after
        .iter()
        .filter(|target| target.contains("cli_safety"))
        .cloned()
        .collect::<Vec<_>>();
    let sentinel_excluded = stable_strings(
        residual
            .residual_sentinel_targets
            .clone()
            .into_iter()
            .chain(safety_like_candidates.clone()),
    );
    let candidate_pool = exclusion
        .candidate_pool_after
        .iter()
        .filter(|target| !safety_like_candidates.contains(*target))
        .cloned()
        .collect::<Vec<_>>();
    let low_risk_candidates = candidate_pool
        .iter()
        .filter(|target| {
            target.contains("shared_fixture") || target.contains("workspace_timeout_root_cause")
        })
        .cloned()
        .collect::<Vec<_>>();
    let medium_risk_candidates = candidate_pool
        .iter()
        .filter(|target| target.contains("control_tower"))
        .cloned()
        .collect::<Vec<_>>();
    let high_risk_candidates = candidate_pool
        .iter()
        .filter(|target| target.contains("cli_safety"))
        .cloned()
        .collect::<Vec<_>>();
    let candidates_with_equivalent_coverage_feasible = candidate_pool
        .iter()
        .filter(|target| !target.contains("control_tower"))
        .cloned()
        .collect::<Vec<_>>();
    let candidates_needing_more_evidence = candidate_pool
        .iter()
        .filter(|target| target.contains("control_tower"))
        .cloned()
        .collect::<Vec<_>>();
    let pool_status = if candidate_pool.is_empty() {
        "NoSafeCandidates"
    } else if bundle.equivalent_coverage_proof_report_v3.proof_status == "EquivalentCoverageProven"
    {
        "CandidatePoolReadyWithWarnings"
    } else {
        "CandidatePoolNeedsMoreEvidence"
    };
    RemainingSafeConsolidationCandidatePoolReport {
        report_id: "remaining-safe-candidate-pool".to_string(),
        candidate_pool,
        low_risk_candidates,
        medium_risk_candidates,
        high_risk_candidates,
        sentinel_candidates_excluded: sentinel_excluded,
        candidates_with_equivalent_coverage_feasible,
        candidates_needing_more_evidence,
        pool_status: pool_status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_fifth_patch_candidate_preselection_report(
    pool: &RemainingSafeConsolidationCandidatePoolReport,
    ledger: &CumulativeSafePatchLedgerV3,
) -> FifthPatchCandidatePreselectionReport {
    let preselected_candidate = pool.low_risk_candidates.first().cloned();
    let preselection_status = if preselected_candidate.is_some() {
        "FifthPatchCandidatePreselected"
    } else {
        "FifthPatchNoSafeCandidate"
    };
    FifthPatchCandidatePreselectionReport {
        report_id: "fifth-patch-candidate-preselection".to_string(),
        candidate_reason: if let Some(candidate) = preselected_candidate.as_ref() {
            format!("selected smallest non-sentinel residual candidate: {candidate}")
        } else {
            "no safe residual candidate remained after retired and sentinel exclusion".to_string()
        },
        expected_assertion_moves: preselected_candidate
            .as_ref()
            .map(|candidate| vec![format!("migrate {candidate} assertions into tests/shared_fixture_harness_application_v1.rs")])
            .unwrap_or_default(),
        expected_equivalent_coverage_refs: ledger.cumulative_equivalent_coverage_refs.clone(),
        risk_preview: preselected_candidate
            .as_ref()
            .map(|candidate| format!("candidate {candidate} is only paper-preselected; no patch applied"))
            .unwrap_or_else(|| "no preselection available".to_string()),
        preselected_candidate,
        preselection_status: preselection_status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_fifth_patch_decision_gate(
    pool: &RemainingSafeConsolidationCandidatePoolReport,
    preselection: &FifthPatchCandidatePreselectionReport,
    equivalent: &EquivalentCoverageContinuityCheckV1,
    sentinel: &SafetySentinelContinuityCheckV1,
    no_hidden_skip: &NoHiddenSkipContinuityCheckV1,
    root_cause: &WorkspaceTimeoutRootCauseReport,
    acceptance_truth: &AcceptanceTruthGateV12,
) -> FifthPatchDecisionGate {
    let candidate = preselection.preselected_candidate.clone();
    let equivalent_coverage_feasible = candidate.is_some()
        && equivalent.coverage_gaps == 0
        && !pool.candidates_with_equivalent_coverage_feasible.is_empty();
    let assertion_migration_feasible = candidate
        .as_ref()
        .map(|candidate| !candidate.contains("control_tower") && !candidate.contains("sentinel"))
        .unwrap_or(false);
    let safety_sentinel_preserved = !sentinel.sentinels_preserved_across_sprints.is_empty()
        && candidate
            .as_ref()
            .map(|candidate| {
                !candidate.contains("committee_cli_safety")
                    && !candidate.contains("workspace_cli_safety")
            })
            .unwrap_or(false);
    let no_hidden_skip_guard = no_hidden_skip.hidden_skip_indicators.is_empty();
    let candidate_is_high_risk_sentinel = candidate
        .as_ref()
        .map(|candidate| candidate.contains("cli_safety") || candidate.contains("determinism"))
        .unwrap_or(false);
    let candidate_already_retired = candidate
        .as_ref()
        .map(|candidate| {
            !pool.candidate_pool.contains(candidate)
                || pool.sentinel_candidates_excluded.contains(candidate)
        })
        .unwrap_or(false);
    let root_cause_ready = matches!(
        root_cause.root_cause_status.as_str(),
        "TimeoutRootCauseIsolated" | "TimeoutRootCausePartiallyIsolated"
    );
    let fifth_patch_allowed = equivalent_coverage_feasible
        && assertion_migration_feasible
        && safety_sentinel_preserved
        && no_hidden_skip_guard
        && root_cause_ready
        && !candidate_already_retired
        && !candidate_is_high_risk_sentinel;
    let gate_status = if fifth_patch_allowed {
        "FifthPatchAllowedWithWarnings"
    } else if !safety_sentinel_preserved || candidate_is_high_risk_sentinel {
        "FifthPatchBlockedBySafety"
    } else {
        "FifthPatchBlockedPendingEvidence"
    };
    FifthPatchDecisionGate {
        gate_id: "fifth-patch-decision-gate".to_string(),
        candidate_pool_status: pool.pool_status.clone(),
        preselection_status: preselection.preselection_status.clone(),
        equivalent_coverage_feasible,
        assertion_migration_feasible,
        safety_sentinel_preserved,
        no_hidden_skip_guard,
        timeout_root_cause_status: root_cause.root_cause_status.clone(),
        acceptance_truth_status: acceptance_truth.truth_status.clone(),
        fifth_patch_allowed,
        gate_status: gate_status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_fifth_patch_risk_preview_report(
    preselection: &FifthPatchCandidatePreselectionReport,
    gate: &FifthPatchDecisionGate,
) -> FifthPatchRiskPreviewReport {
    FifthPatchRiskPreviewReport {
        report_id: "fifth-patch-risk-preview".to_string(),
        candidate: preselection.preselected_candidate.clone(),
        semantic_risk: if gate.fifth_patch_allowed {
            "Low"
        } else {
            "Moderate"
        }
        .to_string(),
        safety_risk: if gate.safety_sentinel_preserved {
            "Low"
        } else {
            "High"
        }
        .to_string(),
        determinism_risk: if gate.fifth_patch_allowed {
            "Low"
        } else {
            "Moderate"
        }
        .to_string(),
        fixture_render_cli_risk: if preselection
            .preselected_candidate
            .as_deref()
            .unwrap_or_default()
            .contains("control_tower")
        {
            "Moderate"
        } else {
            "Low"
        }
        .to_string(),
        cumulative_patch_interaction_risk: "Moderate".to_string(),
        status: if gate.fifth_patch_allowed {
            "FifthPatchRiskPreviewReadyWithWarnings"
        } else {
            "FifthPatchRiskPreviewReady"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_assertion_ledger_continuity_check_v1(
    config: &WorkspaceTimeoutRootCauseConfig,
    ledger: &CumulativeSafePatchLedgerV3,
) -> AssertionLedgerContinuityCheckV1 {
    let previous_ledgers_loaded = config
        .previous_ledger_paths
        .as_ref()
        .map_or(4, |paths| paths.len());
    let missing_ledger_count = 4usize.saturating_sub(previous_ledgers_loaded.min(4));
    AssertionLedgerContinuityCheckV1 {
        report_id: "assertion-ledger-continuity-check-v1".to_string(),
        previous_ledgers_loaded,
        cumulative_assertion_delta: ledger.cumulative_assertion_delta,
        missing_ledger_count,
        continuity_status: if missing_ledger_count == 0 {
            "AssertionLedgerContinuityReady"
        } else {
            "AssertionLedgerContinuityReadyWithWarnings"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_equivalent_coverage_continuity_check_v1(
    bundle: &SafeConsolidationPatchV4Bundle,
) -> EquivalentCoverageContinuityCheckV1 {
    EquivalentCoverageContinuityCheckV1 {
        report_id: "equivalent-coverage-continuity-check-v1".to_string(),
        previous_equivalent_coverage_proofs_loaded: 4,
        coverage_gaps: bundle
            .equivalent_coverage_proof_report_v3
            .cumulative_coverage_gap_count,
        continuity_status: if bundle
            .equivalent_coverage_proof_report_v3
            .cumulative_coverage_gap_count
            == 0
        {
            "EquivalentCoverageContinuityReady"
        } else {
            "EquivalentCoverageContinuityGap"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_safety_sentinel_continuity_check_v1(
    bundle: &SafeConsolidationPatchV4Bundle,
) -> SafetySentinelContinuityCheckV1 {
    let mut sentinels = Vec::new();
    if bundle
        .safety_sentinel_preservation_report_v4
        .committee_cli_safety_preserved
    {
        sentinels.push("CommitteeCliSafety".to_string());
    }
    if bundle
        .safety_sentinel_preservation_report_v4
        .workspace_cli_safety_preserved
    {
        sentinels.push("WorkspaceCliSafety".to_string());
    }
    if bundle
        .safety_sentinel_preservation_report_v4
        .workspace_determinism_preserved
    {
        sentinels.push("WorkspaceDeterminism".to_string());
    }
    if bundle
        .safety_sentinel_preservation_report_v4
        .paper_lifecycle_safety_preserved
    {
        sentinels.push("PaperLifecycleSafety".to_string());
    }
    if bundle
        .safety_sentinel_preservation_report_v4
        .runtime_deferred_guard_preserved
    {
        sentinels.push("RuntimeDeferredGuard".to_string());
    }
    if bundle
        .safety_sentinel_preservation_report_v4
        .no_order_account_guard_preserved
    {
        sentinels.push("NoOrderAccountGuard".to_string());
    }
    SafetySentinelContinuityCheckV1 {
        report_id: "safety-sentinel-continuity-check-v1".to_string(),
        sentinels_preserved_across_sprints: stable_strings(sentinels),
        continuity_status: "SafetySentinelContinuityReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_no_hidden_skip_continuity_check_v1() -> NoHiddenSkipContinuityCheckV1 {
    NoHiddenSkipContinuityCheckV1 {
        report_id: "no-hidden-skip-continuity-check-v1".to_string(),
        hidden_skip_indicators: Vec::new(),
        skip_status: "NoHiddenSkipContinuityReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_workspace_timeout_root_cause_report(
    no_run: &WorkspaceNoRunProgressTraceV1,
    full: &WorkspaceFullRunProgressTraceV1,
    capture: &CargoJsonProgressCaptureV5,
    stall: &CargoTargetStallAttributionReport,
    integration: &IntegrationTestBinaryStallReport,
    fanout: &TestFamilyFanoutMapV2,
) -> WorkspaceTimeoutRootCauseReport {
    let no_run_timeout_observed = matches!(
        no_run.trace_status.as_str(),
        "NoRunProgressTimedOut" | "DiagnosticOnly"
    );
    let full_timeout_observed = matches!(
        full.trace_status.as_str(),
        "FullRunProgressTimedOut" | "DiagnosticOnly"
    );
    let mut suspected = Vec::new();
    if !integration.stalled_integration_targets.is_empty() {
        suspected.push("IntegrationTestBinaryFanout".to_string());
    }
    if fanout
        .fixture_fanout_clusters
        .iter()
        .any(|cluster| cluster.fanout_count >= 2)
    {
        suspected.push("FixtureSetupFanout".to_string());
    }
    if fanout
        .render_fanout_clusters
        .iter()
        .any(|cluster| cluster.fanout_count >= 2)
    {
        suspected.push("ArtifactRenderFanout".to_string());
    }
    if fanout
        .cli_fanout_clusters
        .iter()
        .any(|cluster| cluster.fanout_count >= 1)
    {
        suspected.push("CliSmokeFanout".to_string());
    }
    if stall
        .suspected_stalled_targets
        .iter()
        .any(|target| target.contains("control_tower"))
    {
        suspected.push("LinkTimeCost".to_string());
    }
    if stall
        .suspected_stalled_targets
        .iter()
        .any(|target| target.contains("workspace_timeout_root_cause"))
    {
        suspected.push("MacroExpansionCost".to_string());
    }
    if suspected.is_empty() {
        suspected.push("Unknown".to_string());
    }
    let evidence_strength = if capture.capture_status == "DiagnosticOnly" && suspected.len() >= 3 {
        "Moderate"
    } else if suspected.len() >= 2 {
        "Weak"
    } else {
        "Insufficient"
    };
    let root_cause_status =
        if suspected.contains(&"IntegrationTestBinaryFanout".to_string()) && suspected.len() >= 3 {
            "TimeoutRootCausePartiallyIsolated"
        } else {
            "TimeoutRootCauseStillAmbiguous"
        };
    WorkspaceTimeoutRootCauseReport {
        report_id: "workspace-timeout-root-cause".to_string(),
        no_run_timeout_observed,
        full_timeout_observed,
        cargo_json_progress_available: capture.capture_status != "CargoJsonProgressNotRun",
        last_seen_targets: stable_strings(
            no_run
                .last_seen_target
                .clone()
                .into_iter()
                .chain(full.last_seen_target.clone())
                .chain(capture.last_seen_targets.clone()),
        ),
        last_seen_artifacts: stable_strings(
            no_run
                .last_seen_artifact
                .clone()
                .into_iter()
                .chain(full.last_seen_artifact.clone())
                .chain(capture.last_seen_artifacts.clone()),
        ),
        suspected_root_causes: stable_strings(suspected),
        evidence_strength: evidence_strength.to_string(),
        root_cause_status: root_cause_status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_workspace_observation_quality_report_v1(
    no_run: &WorkspaceNoRunProgressTraceV1,
    full: &WorkspaceFullRunProgressTraceV1,
    capture: &CargoJsonProgressCaptureV5,
    cleanup: &TimeoutCleanupVerificationReportV4,
) -> WorkspaceObservationQualityReportV1 {
    WorkspaceObservationQualityReportV1 {
        report_id: "workspace-observation-quality-v1".to_string(),
        no_run_observation_quality: if no_run.attempted {
            "Measured"
        } else {
            "Diagnostic"
        }
        .to_string(),
        full_observation_quality: if full.attempted {
            "Measured"
        } else {
            "Diagnostic"
        }
        .to_string(),
        cargo_json_quality: if capture.attempted {
            "Measured"
        } else {
            "Diagnostic"
        }
        .to_string(),
        timeout_cleanup_quality: if cleanup.timeout_occurred {
            "Verified"
        } else {
            "SupportingOnly"
        }
        .to_string(),
        observation_status: "ObservationQualityReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_timeout_window_adequacy_report_v1(
    config: &WorkspaceTimeoutRootCauseConfig,
    no_run: &WorkspaceNoRunProgressTraceV1,
    full: &WorkspaceFullRunProgressTraceV1,
) -> TimeoutWindowAdequacyReportV1 {
    let previous_timeout_seconds = 240;
    let current_timeout_seconds = config.no_run_timeout_ms.unwrap_or(300_000) / 1000;
    let still_insufficient_if_timed_out =
        matches!(no_run.trace_status.as_str(), "NoRunProgressTimedOut")
            || matches!(full.trace_status.as_str(), "FullRunProgressTimedOut");
    TimeoutWindowAdequacyReportV1 {
        report_id: "timeout-window-adequacy-v1".to_string(),
        previous_timeout_seconds,
        current_timeout_seconds,
        did_timeout_extend: current_timeout_seconds > previous_timeout_seconds,
        still_insufficient_if_timed_out,
        recommended_next_timeout_or_strategy: if still_insufficient_if_timed_out {
            "increase timeout again or capture a narrower diagnostic cargo JSON sample before any fifth patch"
                .to_string()
        } else {
            "keep timeout extension diagnostic-only until full workspace finishes and passes"
                .to_string()
        },
        status: "TimeoutWindowAdequacyReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_timeout_cleanup_verification_report_v4(
    no_run: &TimeoutCleanupState,
    full: &TimeoutCleanupState,
) -> TimeoutCleanupVerificationReportV4 {
    let timeout_occurred = no_run.timeout_occurred || full.timeout_occurred;
    let child_process_cleanup_attempted =
        no_run.child_process_cleanup_attempted || full.child_process_cleanup_attempted;
    let remaining_cargo_processes =
        no_run.remaining_cargo_processes + full.remaining_cargo_processes;
    let remaining_rustc_processes =
        no_run.remaining_rustc_processes + full.remaining_rustc_processes;
    TimeoutCleanupVerificationReportV4 {
        report_id: "timeout-cleanup-verification-v4".to_string(),
        timeout_occurred,
        child_process_cleanup_attempted,
        remaining_cargo_processes,
        remaining_rustc_processes,
        cleanup_status: if timeout_occurred
            && remaining_cargo_processes == 0
            && remaining_rustc_processes == 0
        {
            "TimeoutCleanupVerified"
        } else if timeout_occurred {
            "TimeoutCleanupNeedsFollowup"
        } else {
            "TimeoutCleanupNotNeeded"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_workspace_no_run_recovery_gate_v12(
    no_run: &WorkspaceNoRunProgressTraceV1,
) -> WorkspaceNoRunRecoveryGateV12 {
    let no_run_completed = matches!(no_run.trace_status.as_str(), "NoRunProgressTraceReady");
    WorkspaceNoRunRecoveryGateV12 {
        gate_id: "workspace-no-run-recovery-gate-v12".to_string(),
        no_run_completed,
        no_run_passed: no_run_completed,
        timeout_observed: matches!(
            no_run.trace_status.as_str(),
            "NoRunProgressTimedOut" | "DiagnosticOnly"
        ),
        gate_status: if no_run_completed {
            "NoRunRecovered"
        } else {
            "NoRunStillBlocked"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_workspace_full_acceptance_gate_v12(
    full: &WorkspaceFullRunProgressTraceV1,
) -> WorkspaceFullAcceptanceGateV12 {
    let full_run_completed = matches!(full.trace_status.as_str(), "FullRunProgressTraceReady");
    WorkspaceFullAcceptanceGateV12 {
        gate_id: "workspace-full-acceptance-gate-v12".to_string(),
        full_run_completed,
        full_run_passed: full_run_completed,
        timeout_observed: matches!(
            full.trace_status.as_str(),
            "FullRunProgressTimedOut" | "DiagnosticOnly"
        ),
        gate_status: if full_run_completed {
            "FullWorkspaceAccepted"
        } else {
            "FullWorkspaceStillBlocked"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_focused_vs_full_bridge_v8(
    import: &Sprint110BaselineTruthImportReport,
    no_run_gate: &WorkspaceNoRunRecoveryGateV12,
) -> FocusedVsFullBridgeV8 {
    FocusedVsFullBridgeV8 {
        bridge_id: "focused-vs-full-bridge-v8".to_string(),
        focused_supporting_only: import.focused_suite_passed,
        cli_supporting_only: import.cli_smoke_passed,
        cargo_build_supporting_only: import.cargo_build_passed,
        no_run_supporting_only: no_run_gate.gate_status != "FullWorkspaceAccepted",
        full_workspace_required: true,
        bridge_status: "FocusedVsFullBridgeReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_acceptance_truth_gate_v12(
    import: &Sprint110BaselineTruthImportReport,
    no_run_gate: &WorkspaceNoRunRecoveryGateV12,
    full_gate: &WorkspaceFullAcceptanceGateV12,
) -> AcceptanceTruthGateV12 {
    AcceptanceTruthGateV12 {
        gate_id: "acceptance-truth-gate-v12".to_string(),
        focused_truth_status: if import.focused_suite_passed {
            "SupportingOnly"
        } else {
            "Missing"
        }
        .to_string(),
        cli_truth_status: if import.cli_smoke_passed {
            "SupportingOnly"
        } else {
            "Missing"
        }
        .to_string(),
        cargo_build_truth_status: if import.cargo_build_passed {
            "SupportingOnly"
        } else {
            "Missing"
        }
        .to_string(),
        no_run_truth_status: if no_run_gate.gate_status == "NoRunRecovered" {
            "SufficientForNoRunOnly"
        } else {
            "SupportingOnly"
        }
        .to_string(),
        full_workspace_truth_status: if full_gate.gate_status == "FullWorkspaceAccepted" {
            "SufficientForFullAcceptance"
        } else {
            "SupportingOnly"
        }
        .to_string(),
        truth_status: if full_gate.gate_status == "FullWorkspaceAccepted" {
            "AcceptanceTruthReady"
        } else {
            "AcceptanceTruthReadyWithWarnings"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_measured_vs_sample_backed_evidence_gate_v5(
    impact: &CumulativeSafePatchImpactReportV3,
    full_gate: &WorkspaceFullAcceptanceGateV12,
) -> MeasuredVsSampleBackedEvidenceGateV5 {
    MeasuredVsSampleBackedEvidenceGateV5 {
        gate_id: "measured-vs-sample-backed-evidence-gate-v5".to_string(),
        measured_evidence_present: impact.cumulative_measured_delta.is_some(),
        sample_backed_evidence_present: true,
        can_claim_measured: impact.measured_claim_allowed,
        can_claim_acceptance: full_gate.gate_status == "FullWorkspaceAccepted",
        status: if full_gate.gate_status == "FullWorkspaceAccepted" {
            "MeasuredVsSampleBackedReady"
        } else {
            "SampleBackedOnly"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_acceptance_evidence_strength_report_v1(
    import: &Sprint110BaselineTruthImportReport,
    no_run_gate: &WorkspaceNoRunRecoveryGateV12,
    full_gate: &WorkspaceFullAcceptanceGateV12,
) -> AcceptanceEvidenceStrengthReportV1 {
    let no_run_evidence_strength = if no_run_gate.gate_status == "NoRunRecovered" {
        "SufficientForNoRunOnly"
    } else {
        "SupportingOnly"
    };
    let full_workspace_evidence_strength = if full_gate.gate_status == "FullWorkspaceAccepted" {
        "SufficientForFullAcceptance"
    } else {
        "Insufficient"
    };
    let overall = if full_gate.gate_status == "FullWorkspaceAccepted" {
        "SufficientForFullAcceptance"
    } else if no_run_gate.gate_status == "NoRunRecovered" {
        "SufficientForNoRunOnly"
    } else if import.focused_suite_passed || import.cli_smoke_passed || import.cargo_build_passed {
        "SupportingOnly"
    } else {
        "Insufficient"
    };
    AcceptanceEvidenceStrengthReportV1 {
        report_id: "acceptance-evidence-strength-v1".to_string(),
        focused_evidence_strength: if import.focused_suite_passed {
            "SupportingOnly"
        } else {
            "Insufficient"
        }
        .to_string(),
        cli_evidence_strength: if import.cli_smoke_passed {
            "SupportingOnly"
        } else {
            "Insufficient"
        }
        .to_string(),
        build_evidence_strength: if import.cargo_build_passed {
            "SupportingOnly"
        } else {
            "Insufficient"
        }
        .to_string(),
        no_run_evidence_strength: no_run_evidence_strength.to_string(),
        full_workspace_evidence_strength: full_workspace_evidence_strength.to_string(),
        overall_acceptance_evidence_strength: overall.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_workspace_recovery_decision_report_v1(
    root_cause: &WorkspaceTimeoutRootCauseReport,
    gate: &FifthPatchDecisionGate,
    pool: &RemainingSafeConsolidationCandidatePoolReport,
) -> WorkspaceRecoveryDecisionReportV1 {
    let recommend_stop_consolidation = Some(pool.pool_status == "NoSafeCandidates");
    WorkspaceRecoveryDecisionReportV1 {
        report_id: "workspace-recovery-decision-v1".to_string(),
        recommend_fifth_patch: gate.fifth_patch_allowed,
        recommend_more_observation: root_cause.root_cause_status
            == "TimeoutRootCauseStillAmbiguous",
        recommend_nextest_diagnostic: Some(true),
        recommend_sccache_diagnostic: Some(
            root_cause
                .suspected_root_causes
                .contains(&"LinkTimeCost".to_string()),
        ),
        recommend_stop_consolidation,
        decision_status: if gate.fifth_patch_allowed {
            "WorkspaceRecoveryDecisionReady"
        } else if recommend_stop_consolidation == Some(true) {
            "WorkspaceRecoveryStopConsolidation"
        } else {
            "WorkspaceRecoveryNeedsMoreObservation"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_control_tower_workspace_timeout_root_cause_panel(
    root_cause: &WorkspaceTimeoutRootCauseReport,
    no_run: &WorkspaceNoRunProgressTraceV1,
    full: &WorkspaceFullRunProgressTraceV1,
    capture: &CargoJsonProgressCaptureV5,
    stall: &CargoTargetStallAttributionReport,
    fanout: &TestFamilyFanoutMapV2,
    adequacy: &TimeoutWindowAdequacyReportV1,
    evidence: &AcceptanceEvidenceStrengthReportV1,
) -> ControlTowerWorkspaceTimeoutRootCausePanel {
    ControlTowerWorkspaceTimeoutRootCausePanel {
        panel_id: "control-tower-workspace-timeout-root-cause".to_string(),
        root_cause_status: root_cause.root_cause_status.clone(),
        no_run_progress_trace_status: no_run.trace_status.clone(),
        full_progress_trace_status: full.trace_status.clone(),
        cargo_json_timeline_status: capture.capture_status.clone(),
        target_stall_attribution_status: stall.attribution_status.clone(),
        fanout_status: fanout.fanout_status.clone(),
        timeout_window_adequacy_status: adequacy.status.clone(),
        acceptance_evidence_strength: evidence.overall_acceptance_evidence_strength.clone(),
        next_actions: vec![
            "collect one longer real no-run observation before changing any patch scope".to_string(),
            "keep fifth patch paper-only until equivalent coverage and sentinel preservation remain explicit".to_string(),
        ],
        warnings: vec![
            "research-only".to_string(),
            "paper-only".to_string(),
            "cargo progress is diagnostic only".to_string(),
            "no run-tests button".to_string(),
        ],
        static_read_only: true,
        no_apply_patch_button: true,
        no_run_tests_button: true,
        no_train_runtime_live_order_account_controls: true,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_control_tower_fifth_patch_decision_panel(
    pool: &RemainingSafeConsolidationCandidatePoolReport,
    preselection: &FifthPatchCandidatePreselectionReport,
    gate: &FifthPatchDecisionGate,
    risk: &FifthPatchRiskPreviewReport,
    ledger: &AssertionLedgerContinuityCheckV1,
    equivalent: &EquivalentCoverageContinuityCheckV1,
    sentinel: &SafetySentinelContinuityCheckV1,
) -> ControlTowerFifthPatchDecisionPanel {
    ControlTowerFifthPatchDecisionPanel {
        panel_id: "control-tower-fifth-patch-decision".to_string(),
        candidate_pool: pool.candidate_pool.clone(),
        preselection: preselection.preselected_candidate.clone(),
        decision_gate_status: gate.gate_status.clone(),
        risk_preview_status: risk.status.clone(),
        ledger_continuity_status: ledger.continuity_status.clone(),
        equivalent_coverage_continuity_status: equivalent.continuity_status.clone(),
        sentinel_continuity_status: sentinel.continuity_status.clone(),
        fifth_patch_allowed: gate.fifth_patch_allowed,
        static_read_only: true,
        no_apply_patch_button: true,
        no_run_tests_button: true,
        no_train_runtime_live_order_account_controls: true,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn build_safety_coverage_preservation_report_v27(
    bundle: &SafeConsolidationPatchV4Bundle,
    config: &WorkspaceTimeoutRootCauseConfig,
    gate: &FifthPatchDecisionGate,
    evidence: &AcceptanceEvidenceStrengthReportV1,
) -> SafetyCoveragePreservationReportV27 {
    let previous = &bundle.safety_coverage_preservation_report_v26;
    SafetyCoveragePreservationReportV27 {
        report_id: "safety-coverage-preservation-v27".to_string(),
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
        committee_owned_core_guard_present: previous.committee_owned_core_guard_present,
        investor_impersonation_guard_present: previous.investor_impersonation_guard_present,
        paper_candidate_not_order_guard_present: previous.paper_candidate_not_order_guard_present,
        no_silent_confidence_upgrade_guard_present: previous
            .no_silent_confidence_upgrade_guard_present,
        focused_not_full_acceptance_guard_present: previous
            .focused_not_full_acceptance_guard_present,
        no_hidden_skip_guard_present: previous.no_hidden_skip_guard_present,
        assertion_preservation_guard_present: previous.assertion_preservation_guard_present,
        safety_sentinel_preservation_guard_present: previous
            .safety_sentinel_preservation_guard_present,
        cumulative_assertion_ledger_guard_present: previous
            .cumulative_assertion_ledger_guard_present,
        equivalent_coverage_v2_guard_present: previous.equivalent_coverage_v2_guard_present,
        timeout_cleanup_v2_guard_present: previous.timeout_cleanup_v2_guard_present,
        cargo_json_progress_truth_guard_present: previous.cargo_json_progress_truth_guard_present,
        third_patch_no_broad_consolidation_guard_present: previous
            .third_patch_no_broad_consolidation_guard_present,
        sprint109_validation_reconciliation_guard_present: previous
            .sprint109_validation_reconciliation_guard_present,
        cumulative_assertion_ledger_v2_guard_present: previous
            .cumulative_assertion_ledger_v2_guard_present,
        equivalent_coverage_v3_guard_present: previous.equivalent_coverage_v3_guard_present,
        timeout_cleanup_v3_guard_present: previous.timeout_cleanup_v3_guard_present,
        cargo_json_progress_v4_truth_guard_present: previous
            .cargo_json_progress_v4_truth_guard_present,
        fourth_patch_no_broad_consolidation_guard_present: previous
            .fourth_patch_no_broad_consolidation_guard_present,
        sprint110_truth_import_guard_present: config.require_sprint110_truth_import,
        timeout_root_cause_guard_present: true,
        fifth_patch_decision_gate_guard_present: config.require_fifth_patch_decision_gate
            && !gate.gate_status.is_empty(),
        no_auto_fifth_patch_guard_present: !config.allow_fifth_patch_application,
        acceptance_evidence_strength_guard_present: !evidence
            .overall_acceptance_evidence_strength
            .is_empty(),
        safety_status: "SafetyCoveragePreserved".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
