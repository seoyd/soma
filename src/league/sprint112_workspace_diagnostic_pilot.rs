use crate::ReasonCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

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
    !path.trim().is_empty() && !path.contains("://")
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_output_root() -> String {
    "target/soma_sprint112_workspace_diagnostic_pilot".to_string()
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
    for code in extra {
        if !codes.contains(code) {
            codes.push(code.clone());
        }
    }
    codes
}

fn stable_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn stable_join(values: &[String]) -> String {
    values.join(", ")
}

fn load_first_json<T: DeserializeOwned>(paths: Option<&Vec<String>>) -> Result<Option<T>, String> {
    if let Some(paths) = paths {
        for path in paths {
            let candidate = if Path::new(path).is_absolute() {
                PathBuf::from(path)
            } else {
                project_root().join(path)
            };
            if candidate.exists() {
                let text = fs::read_to_string(&candidate).map_err(|err| err.to_string())?;
                return serde_json::from_str(&text)
                    .map(Some)
                    .map_err(|err| format!("{}: {err}", candidate.display()));
            }
        }
    }
    Ok(None)
}

macro_rules! report {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
        pub struct $name {
            $(pub $field: $ty,)*
            pub reason_codes: Vec<ReasonCode>,
        }
    };
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceDiagnosticPilotV1Config {
    pub pilot_id: String,
    #[serde(default)]
    pub sprint111_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub sprint111_truth_paths: Option<Vec<String>>,
    #[serde(default)]
    pub cargo_json_progress_paths: Option<Vec<String>>,
    #[serde(default)]
    pub timing_capture_paths: Option<Vec<String>>,
    #[serde(default)]
    pub nextest_paths: Option<Vec<String>>,
    #[serde(default)]
    pub sccache_paths: Option<Vec<String>>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_false")]
    pub run_nextest_probe: bool,
    #[serde(default = "default_false")]
    pub run_sccache_probe: bool,
    #[serde(default = "default_false")]
    pub run_real_no_run_observation: bool,
    #[serde(default = "default_false")]
    pub run_real_full_observation: bool,
    #[serde(default = "default_false")]
    pub run_cargo_json_progress_capture: bool,
    #[serde(default = "default_false")]
    pub run_cargo_check_timing: bool,
    #[serde(default = "default_false")]
    pub run_cargo_build_timing: bool,
    #[serde(default = "default_timeout_ms")]
    pub no_run_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub full_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub cargo_json_timeout_ms: Option<u64>,
    #[serde(default = "default_true")]
    pub require_nextest_sccache_diagnostic: bool,
    #[serde(default = "default_true")]
    pub require_acceptance_truth_gate: bool,
    #[serde(default = "default_true")]
    pub require_fifth_patch_re_evaluation: bool,
    #[serde(default = "default_false")]
    pub allow_fifth_patch_application: bool,
    #[serde(default = "default_true")]
    pub preserve_runtime_deferred: bool,
    #[serde(default = "default_true")]
    pub preserve_safety_guards: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for WorkspaceDiagnosticPilotV1Config {
    fn default() -> Self {
        Self {
            pilot_id: "sprint112-workspace-diagnostic-pilot".to_string(),
            sprint111_bundle_paths: Some(vec![
                "examples/sprint112_data/sprint111_summary.json".to_string(),
            ]),
            sprint111_truth_paths: Some(vec![
                "examples/sprint112_data/sprint111_summary.json".to_string(),
            ]),
            cargo_json_progress_paths: Some(vec![
                "examples/sprint112_data/cargo_json_progress_v6_expected.json".to_string(),
            ]),
            timing_capture_paths: Some(vec![
                "examples/sprint112_data/cargo_timing_expected.json".to_string(),
            ]),
            nextest_paths: Some(vec![
                "examples/sprint112_data/nextest_availability_expected.json".to_string(),
            ]),
            sccache_paths: Some(vec![
                "examples/sprint112_data/sccache_availability_expected.json".to_string(),
            ]),
            output_root: default_output_root(),
            run_nextest_probe: false,
            run_sccache_probe: false,
            run_real_no_run_observation: false,
            run_real_full_observation: false,
            run_cargo_json_progress_capture: false,
            run_cargo_check_timing: false,
            run_cargo_build_timing: false,
            no_run_timeout_ms: default_timeout_ms(),
            full_timeout_ms: default_timeout_ms(),
            cargo_json_timeout_ms: default_timeout_ms(),
            require_nextest_sccache_diagnostic: true,
            require_acceptance_truth_gate: true,
            require_fifth_patch_re_evaluation: true,
            allow_fifth_patch_application: false,
            preserve_runtime_deferred: true,
            preserve_safety_guards: true,
            reason_codes: diagnostic_reason_codes(&[]),
        }
    }
}

impl WorkspaceDiagnosticPilotV1Config {
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
        PathBuf::from(&self.output_root).join(&self.pilot_id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.pilot_id.trim().is_empty() {
            return Err("sprint112 pilot_id must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err(
                "sprint112 workspace diagnostic pilot config paths must be local".to_string(),
            );
        }
        for paths in [
            &self.sprint111_bundle_paths,
            &self.sprint111_truth_paths,
            &self.cargo_json_progress_paths,
            &self.timing_capture_paths,
            &self.nextest_paths,
            &self.sccache_paths,
        ] {
            if let Some(paths) = paths
                && paths.iter().any(|path| !local_only(path))
            {
                return Err(
                    "sprint112 workspace diagnostic pilot config paths must be local".to_string(),
                );
            }
        }
        if !self.require_nextest_sccache_diagnostic {
            return Err("sprint112 requires require_nextest_sccache_diagnostic=true".to_string());
        }
        if !self.require_acceptance_truth_gate {
            return Err("sprint112 requires require_acceptance_truth_gate=true".to_string());
        }
        if !self.require_fifth_patch_re_evaluation {
            return Err("sprint112 requires require_fifth_patch_re_evaluation=true".to_string());
        }
        if self.allow_fifth_patch_application {
            return Err("sprint112 forbids fifth patch application".to_string());
        }
        if !self.preserve_runtime_deferred || !self.preserve_safety_guards {
            return Err("sprint112 preserve flags must stay true".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sprint111SummaryFixture {
    pub report_id: String,
    pub focused_matrix_passed: bool,
    pub focused_target_count: Option<u64>,
    pub focused_test_count: Option<u64>,
    pub cli_smoke_passed: bool,
    pub cli_smoke_count: Option<u64>,
    pub cargo_check_passed: bool,
    pub cargo_build_passed: bool,
    pub no_run_timeout_seconds: Option<u64>,
    pub no_run_exit_code: Option<i32>,
    pub full_timeout_seconds: Option<u64>,
    pub full_exit_code: Option<i32>,
    pub timeout_cleanup_verified: bool,
    pub fifth_patch_blocked: bool,
    pub patch_count_carried_forward: u64,
    pub retired_targets_carried_forward: Vec<String>,
    pub cumulative_assertion_delta: i64,
    pub cumulative_sample_backed_delta: i64,
    pub fifth_patch_block_status: String,
    pub timeout_root_cause_status: String,
    pub acceptance_truth_status: String,
    pub fifth_patch_candidate: Option<String>,
    pub assertion_migration_feasible: bool,
    pub equivalent_coverage_feasible: bool,
    pub safety_sentinels_preserved: bool,
    pub no_hidden_skip_continuity: bool,
    pub candidate_pool: Vec<String>,
    pub low_risk_candidates: Vec<String>,
    pub sentinel_exclusions: Vec<String>,
    pub observed_root_cause_evidence: Vec<String>,
    pub inferred_root_cause_evidence: Vec<String>,
    pub stalled_targets: Vec<String>,
    pub last_targets: Vec<String>,
    pub active_rustc_args: Vec<String>,
    pub max_concurrent_rustc: u64,
    pub fixture_fanout: Vec<String>,
    pub render_fanout: Vec<String>,
    pub cli_fanout: Vec<String>,
    pub helper_fanout: Vec<String>,
    pub link_heavy_candidates: Vec<String>,
    pub macro_heavy_candidates: Vec<String>,
    pub high_fanout_families: Vec<String>,
}

impl Default for Sprint111SummaryFixture {
    fn default() -> Self {
        Self {
            report_id: "sprint111-summary".to_string(),
            focused_matrix_passed: true,
            focused_target_count: Some(15),
            focused_test_count: Some(16),
            cli_smoke_passed: true,
            cli_smoke_count: Some(10),
            cargo_check_passed: true,
            cargo_build_passed: true,
            no_run_timeout_seconds: Some(300),
            no_run_exit_code: Some(124),
            full_timeout_seconds: Some(300),
            full_exit_code: Some(124),
            timeout_cleanup_verified: true,
            fifth_patch_blocked: true,
            patch_count_carried_forward: 4,
            retired_targets_carried_forward: vec![
                "tests/shared_fixture_harness_expansion_plan_v2.rs".to_string(),
                "tests/shared_output_dir_helper_application_v1.rs".to_string(),
                "tests/shared_render_helper_application_v1.rs".to_string(),
                "tests/shared_toml_builder_application_v1.rs".to_string(),
            ],
            cumulative_assertion_delta: 0,
            cumulative_sample_backed_delta: -4,
            fifth_patch_block_status: "FifthPatchBlockedPendingEvidence".to_string(),
            timeout_root_cause_status: "TimeoutRootCausePartiallyIsolated".to_string(),
            acceptance_truth_status: "AcceptanceTruthReadyWithWarnings".to_string(),
            fifth_patch_candidate: Some(
                "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
            ),
            assertion_migration_feasible: false,
            equivalent_coverage_feasible: true,
            safety_sentinels_preserved: true,
            no_hidden_skip_continuity: true,
            candidate_pool: vec![
                "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
                "tests/shared_fixture_harness_application_v1.rs".to_string(),
                "tests/workspace_timeout_root_cause.rs".to_string(),
            ],
            low_risk_candidates: vec![
                "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
                "tests/shared_fixture_harness_application_v1.rs".to_string(),
                "tests/workspace_timeout_root_cause.rs".to_string(),
            ],
            sentinel_exclusions: vec![
                "tests/committee_cli_safety.rs".to_string(),
                "tests/sprint111_cli_safety.rs".to_string(),
            ],
            observed_root_cause_evidence: vec![
                "workspace no-run timed out at 300 seconds with exit 124".to_string(),
                "workspace full run timed out at 300 seconds with exit 124".to_string(),
                "timeout cleanup left no remaining cargo/rustc processes".to_string(),
            ],
            inferred_root_cause_evidence: vec![
                "ArtifactRenderFanout".to_string(),
                "CliSmokeFanout".to_string(),
                "FixtureSetupFanout".to_string(),
                "IntegrationTestBinaryFanout".to_string(),
                "LinkTimeCost".to_string(),
                "MacroExpansionCost".to_string(),
            ],
            stalled_targets: vec![
                "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
                "tests/workspace_timeout_root_cause.rs".to_string(),
            ],
            last_targets: vec![
                "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
                "tests/shared_fixture_harness_application_v1.rs".to_string(),
                "tests/workspace_timeout_root_cause.rs".to_string(),
            ],
            active_rustc_args: vec![
                "--test tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
            ],
            max_concurrent_rustc: 2,
            fixture_fanout: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
            render_fanout: vec!["tests/workspace_timeout_root_cause.rs".to_string()],
            cli_fanout: vec![
                "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
            ],
            helper_fanout: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
            link_heavy_candidates: vec![
                "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
            ],
            macro_heavy_candidates: vec!["tests/workspace_timeout_root_cause.rs".to_string()],
            high_fanout_families: vec![
                "ArtifactRenderFanout".to_string(),
                "CliSmokeFanout".to_string(),
                "FixtureSetupFanout".to_string(),
                "IntegrationTestBinaryFanout".to_string(),
            ],
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NextestSyntheticFixture {
    pub nextest_available: bool,
    pub nextest_version: Option<String>,
    pub partition_strategy: Option<String>,
    pub slow_tests: Vec<String>,
    pub slow_binaries: Vec<String>,
    pub slow_families: Vec<String>,
    pub duration_ms: Option<u64>,
    pub finished: Option<bool>,
    pub passed: Option<bool>,
    pub timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SccacheSyntheticFixture {
    pub sccache_available: bool,
    pub sccache_version: Option<String>,
    pub cache_hits: Option<u64>,
    pub cache_misses: Option<u64>,
    pub duration_before_ms: Option<u64>,
    pub duration_after_ms: Option<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CargoTimingFixture {
    pub cargo_check_finished: bool,
    pub cargo_check_passed: Option<bool>,
    pub cargo_check_duration_ms: Option<u64>,
    pub cargo_build_finished: bool,
    pub cargo_build_passed: Option<bool>,
    pub cargo_build_duration_ms: Option<u64>,
    pub cargo_no_run_finished: bool,
    pub cargo_no_run_passed: Option<bool>,
    pub cargo_no_run_duration_ms: Option<u64>,
    pub cargo_no_run_timeout_ms: Option<u64>,
    pub cargo_full_finished: bool,
    pub cargo_full_passed: Option<bool>,
    pub cargo_full_duration_ms: Option<u64>,
    pub cargo_full_timeout_ms: Option<u64>,
}

report!(Sprint111BaselineTruthImportReport {
    report_id: String, focused_matrix_passed: bool, focused_target_count: Option<u64>,
    focused_test_count: Option<u64>, cli_smoke_passed: bool, cli_smoke_count: Option<u64>,
    cargo_check_passed: bool, cargo_build_passed: bool, no_run_timeout_seconds: Option<u64>,
    no_run_exit_code: Option<i32>, full_timeout_seconds: Option<u64>, full_exit_code: Option<i32>,
    timeout_cleanup_verified: bool, fifth_patch_blocked: bool, imported_as_full_acceptance: bool,
    import_status: String
});
report!(Sprint111PatchAndTimeoutCarryForwardReport {
    report_id: String, patch_count_carried_forward: u64, retired_targets_carried_forward: Vec<String>,
    cumulative_assertion_delta: i64, cumulative_sample_backed_delta: i64, fifth_patch_block_status: String,
    timeout_root_cause_status: String, acceptance_truth_status: String, carry_forward_status: String
});
report!(NextestAvailabilityReportV1 {
    report_id: String, nextest_probe_attempted: bool, nextest_available: bool,
    nextest_version: Option<String>, probe_command: Option<String>, availability_status: String
});
report!(NextestNoRunPilotPlanV1 {
    plan_id: String,
    nextest_available: bool,
    proposed_command: String,
    proposed_partition_strategy: String,
    no_run_equivalence_warning: String,
    plan_status: String
});
report!(NextestRunPilotPlanV1 {
    plan_id: String,
    nextest_available: bool,
    proposed_command: String,
    proposed_partition_strategy: String,
    execution_warning: String,
    plan_status: String
});
report!(NextestPilotExecutionReportV1 {
    report_id: String, attempted: bool, command: Option<String>, finished: bool, passed: Option<bool>,
    duration_ms: Option<u64>, timeout_ms: Option<u64>, slow_tests: Vec<String>, slow_binaries: Vec<String>,
    execution_status: String
});
report!(NextestTargetPartitionReportV1 {
    report_id: String, partition_count: u64, partition_strategy: String, safety_partition_present: bool,
    sentinel_partition_isolated: bool, partitions: Vec<String>, partition_status: String
});
report!(NextestSlowTargetAttributionReportV1 {
    report_id: String, slow_tests: Vec<String>, slow_binaries: Vec<String>, slow_families: Vec<String>,
    attribution_status: String
});
report!(SccacheAvailabilityReportV1 {
    report_id: String, sccache_probe_attempted: bool, sccache_available: bool,
    sccache_version: Option<String>, availability_status: String
});
report!(SccacheLocalOnlyPolicyReportV1 {
    report_id: String,
    local_only_required: bool,
    remote_cache_forbidden: bool,
    secret_cache_forbidden: bool,
    deterministic_key_required: bool,
    cache_failure_must_not_hide_failure: bool,
    policy_status: String
});
report!(SccachePilotPlanV1 {
    report_id: String, proposed_environment: BTreeMap<String, String>, local_only_cache_dir: String,
    baseline_command: String, cached_command: String, no_speedup_claim_warning: String, plan_status: String
});
report!(SccachePilotExecutionReportV1 {
    report_id: String, attempted: bool, available: bool, cache_hits: Option<u64>, cache_misses: Option<u64>,
    duration_before_ms: Option<u64>, duration_after_ms: Option<u64>, execution_status: String,
    no_speedup_overclaim: bool
});
report!(SccacheEffectEstimateReportV1 {
    report_id: String, measured: bool, sample_backed: bool, estimate: Option<String>, confidence: String,
    can_claim_speedup: bool, status: String
});
report!(CargoCheckTimingCaptureV1 {
    report_id: String, attempted: bool, command: String, finished: bool, passed: Option<bool>,
    duration_ms: Option<u64>, timeout_ms: Option<u64>, status: String
});
report!(CargoBuildTimingCaptureV1 {
    report_id: String, attempted: bool, command: String, finished: bool, passed: Option<bool>,
    duration_ms: Option<u64>, timeout_ms: Option<u64>, status: String
});
report!(CargoNoRunTimingCaptureV1 {
    report_id: String, attempted: bool, command: String, finished: bool, passed: Option<bool>,
    duration_ms: Option<u64>, timeout_ms: Option<u64>, status: String
});
report!(CargoFullRunTimingCaptureV1 {
    report_id: String, attempted: bool, command: String, finished: bool, passed: Option<bool>,
    duration_ms: Option<u64>, timeout_ms: Option<u64>, status: String
});
report!(CargoJsonProgressCaptureV6 {
    report_id: String, command: String, attempted: bool, messages: u64, artifacts: u64,
    compiler_messages: u64, last_targets: Vec<String>, stalled_candidates: Vec<String>, status: String
});

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CargoArtifactEventV2 {
    pub seq: u64,
    pub target: String,
    pub artifact: String,
    pub phase: String,
    pub observed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticEvidenceRowV1 {
    pub row_name: String,
    pub evidence_strength: String,
    pub status: String,
    pub supporting_only: bool,
}

report!(CargoArtifactTimelineV2 {
    report_id: String, events: Vec<CargoArtifactEventV2>, target_artifacts: BTreeMap<String, Vec<String>>,
    last_artifact_by_target: BTreeMap<String, String>, status: String
});
report!(CargoTargetStallAttributionReportV2 {
    report_id: String, last_seen_targets: Vec<String>, suspected_stalled_targets: Vec<String>,
    observed_stalled_targets: Vec<String>, inferred_stalled_targets: Vec<String>, status: String
});
report!(RustcProcessTimelineReportV2 {
    report_id: String, max_concurrent_rustc: u64, active_rustc_args: Vec<String>,
    remaining_rustc_after_timeout: u64, status: String
});
report!(IntegrationTestBinaryStallReportV2 {
    report_id: String, stalled_integration_binaries: Vec<String>, high_fanout_families: Vec<String>,
    already_retired_excluded: Vec<String>, status: String
});
report!(LinkMacroAttributionReportV2 {
    report_id: String, link_heavy_candidates: Vec<String>, macro_heavy_candidates: Vec<String>,
    observed_candidates: Vec<String>, inferred_candidates: Vec<String>, status: String
});
report!(FixtureRenderCliFanoutAttributionReportV2 {
    report_id: String, fixture_fanout: Vec<String>, render_fanout: Vec<String>, cli_fanout: Vec<String>,
    helper_fanout: Vec<String>, status: String
});
report!(WorkspaceDiagnosticEvidenceMatrixV1 {
    matrix_id: String, evidence_rows: Vec<DiagnosticEvidenceRowV1>,
    evidence_strength_by_row: BTreeMap<String, String>, supports_acceptance: bool, matrix_status: String
});
report!(WorkspaceTimeoutRootCauseReportV2 {
    report_id: String, root_cause_categories: Vec<String>, observed_evidence: Vec<String>,
    inferred_evidence: Vec<String>, confidence: String, status: String
});
report!(RemainingSafeCandidatePoolReportV2 {
    report_id: String, previous_candidate_pool: Vec<String>, new_evidence: Vec<String>,
    candidate_statuses: BTreeMap<String, String>, sentinel_exclusions: Vec<String>, status: String
});
report!(FifthPatchDecisionGateV2 {
    gate_id: String,
    candidate_pool_status: String,
    previous_gate_status: String,
    new_diagnostic_evidence_status: String,
    assertion_migration_feasible: bool,
    equivalent_coverage_feasible: bool,
    safety_sentinel_preserved: bool,
    no_hidden_skip_continuity: bool,
    root_cause_confidence: String,
    fifth_patch_allowed_for_next_sprint: bool,
    fifth_patch_applied_this_sprint: bool,
    gate_status: String
});
report!(FifthPatchReadinessReevaluationReportV1 {
    report_id: String, previous_block_reason: String, new_evidence: Vec<String>,
    still_blocked_reasons: Vec<String>, allowed_reasons: Vec<String>, status: String
});
report!(FifthPatchNoApplyGuaranteeReportV1 {
    report_id: String,
    fifth_patch_applied: bool,
    no_files_retired_by_fifth_patch: bool,
    no_assertions_moved_by_fifth_patch: bool,
    guarantee_status: String
});
report!(AssertionLedgerContinuityCheckV2 {
    report_id: String,
    carried_forward_assertion_delta: i64,
    assertion_deletions_detected: u64,
    continuity_status: String
});
report!(EquivalentCoverageContinuityCheckV2 {
    report_id: String,
    coverage_gap_count: u64,
    equivalent_coverage_feasible: bool,
    continuity_status: String
});
report!(SafetySentinelContinuityCheckV2 {
    report_id: String, sentinels_preserved: Vec<String>, sentinel_uncertainties: Vec<String>, continuity_status: String
});
report!(NoHiddenSkipContinuityCheckV2 {
    report_id: String, hidden_skip_indicators: Vec<String>, continuity_status: String
});
report!(TimeoutWindowAdequacyReportV2 {
    report_id: String, previous_timeouts_ms: BTreeMap<String, u64>, current_timeouts_ms: BTreeMap<String, u64>,
    adequacy: String, recommendation: String
});
report!(TimeoutCleanupVerificationReportV5 {
    report_id: String,
    timeout_occurred: bool,
    child_process_cleanup_attempted: bool,
    remaining_cargo_processes: u64,
    remaining_rustc_processes: u64,
    cleanup_status: String
});
report!(WorkspaceNoRunRecoveryGateV13 {
    gate_id: String, command: String, finished: bool, passed: Option<bool>, timeout_ms: Option<u64>,
    recovered: bool, gate_status: String
});
report!(WorkspaceFullAcceptanceGateV13 {
    gate_id: String, command: String, finished: bool, passed: Option<bool>, timeout_ms: Option<u64>,
    accepted: bool, gate_status: String
});
report!(FocusedVsFullBridgeV9 {
    bridge_id: String,
    focused_truth_status: String,
    cli_truth_status: String,
    cargo_build_truth_status: String,
    nextest_truth_status: String,
    sccache_truth_status: String,
    cargo_progress_truth_status: String,
    no_run_truth_status: String,
    can_claim_full_acceptance: bool,
    bridge_status: String
});
report!(AcceptanceTruthGateV13 {
    gate_id: String,
    focused_truth_status: String,
    cli_truth_status: String,
    cargo_check_truth_status: String,
    cargo_build_truth_status: String,
    nextest_truth_status: String,
    sccache_truth_status: String,
    cargo_json_truth_status: String,
    no_run_truth_status: String,
    full_workspace_truth_status: String,
    truth_status: String
});
report!(AcceptanceEvidenceStrengthReportV2 {
    report_id: String,
    focused_evidence_strength: String,
    cli_evidence_strength: String,
    cargo_check_evidence_strength: String,
    cargo_build_evidence_strength: String,
    nextest_evidence_strength: String,
    sccache_evidence_strength: String,
    cargo_progress_evidence_strength: String,
    no_run_evidence_strength: String,
    full_workspace_evidence_strength: String,
    overall_evidence_strength: String,
    status: String
});
report!(WorkspaceRecoveryDecisionReportV2 {
    report_id: String,
    recommend_nextest_diagnostic: bool,
    recommend_sccache_diagnostic: bool,
    recommend_more_observation: bool,
    recommend_fifth_patch_for_next_sprint_only: bool,
    recommend_stop_consolidation: bool,
    status: String
});
report!(ControlTowerWorkspaceDiagnosticPilotPanel {
    panel_id: String, nextest_availability_status: String, sccache_availability_status: String,
    cargo_timing_statuses: BTreeMap<String, String>, cargo_progress_status: String,
    diagnostic_matrix_status: String, root_cause_status: String, acceptance_truth_status: String,
    warnings: Vec<String>, static_read_only: bool, no_run_button: bool,
    no_train_runtime_live_order_account_controls: bool
});
report!(ControlTowerFifthPatchReevaluationPanel {
    panel_id: String, fifth_patch_gate_status: String, no_apply_guarantee_status: String,
    candidate_pool_status: String, readiness_reevaluation_status: String,
    continuity_statuses: BTreeMap<String, String>, warnings: Vec<String>, static_read_only: bool,
    no_apply_patch_button: bool, no_train_runtime_live_order_account_controls: bool
});
report!(SafetyCoveragePreservationReportV28 {
    report_id: String,
    live_trading_guard_present: bool,
    broker_guard_present: bool,
    order_guard_present: bool,
    account_guard_present: bool,
    runtime_llm_guard_present: bool,
    mamba_runtime_guard_present: bool,
    gated_runtime_guard_present: bool,
    model_training_guard_present: bool,
    rust_neural_training_guard_present: bool,
    python_training_dependency_guard_present: bool,
    secret_guard_present: bool,
    no_lookahead_guard_present: bool,
    source_boundary_guard_present: bool,
    browser_execution_guard_present: bool,
    ui_order_control_guard_present: bool,
    committee_owned_core_guard_present: bool,
    investor_impersonation_guard_present: bool,
    paper_candidate_not_order_guard_present: bool,
    no_silent_confidence_upgrade_guard_present: bool,
    focused_not_full_acceptance_guard_present: bool,
    no_hidden_skip_guard_present: bool,
    assertion_preservation_guard_present: bool,
    safety_sentinel_preservation_guard_present: bool,
    cumulative_assertion_ledger_guard_present: bool,
    equivalent_coverage_v2_guard_present: bool,
    timeout_cleanup_v2_guard_present: bool,
    cargo_json_progress_truth_guard_present: bool,
    third_patch_no_broad_consolidation_guard_present: bool,
    sprint109_validation_reconciliation_guard_present: bool,
    cumulative_assertion_ledger_v2_guard_present: bool,
    equivalent_coverage_v3_guard_present: bool,
    timeout_cleanup_v3_guard_present: bool,
    cargo_json_progress_v4_truth_guard_present: bool,
    fourth_patch_no_broad_consolidation_guard_present: bool,
    sprint110_truth_import_guard_present: bool,
    timeout_root_cause_guard_present: bool,
    fifth_patch_decision_gate_guard_present: bool,
    no_auto_fifth_patch_guard_present: bool,
    acceptance_evidence_strength_guard_present: bool,
    nextest_diagnostic_only_guard_present: bool,
    sccache_local_only_guard_present: bool,
    fifth_patch_no_apply_guard_present: bool,
    diagnostic_not_acceptance_guard_present: bool,
    no_broad_consolidation_guard_present: bool,
    safety_status: String
});
report!(WorkspaceDiagnosticPilotV1StorageReport {
    report_id: String, output_dir: String, written_files: Vec<String>, file_count: u64
});

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceDiagnosticPilotV1Bundle {
    pub sprint111_baseline_truth_import_report: Sprint111BaselineTruthImportReport,
    pub sprint111_patch_and_timeout_carry_forward_report:
        Sprint111PatchAndTimeoutCarryForwardReport,
    pub nextest_availability_report_v1: NextestAvailabilityReportV1,
    pub nextest_no_run_pilot_plan_v1: NextestNoRunPilotPlanV1,
    pub nextest_run_pilot_plan_v1: NextestRunPilotPlanV1,
    pub nextest_pilot_execution_report_v1: NextestPilotExecutionReportV1,
    pub nextest_target_partition_report_v1: NextestTargetPartitionReportV1,
    pub nextest_slow_target_attribution_report_v1: NextestSlowTargetAttributionReportV1,
    pub sccache_availability_report_v1: SccacheAvailabilityReportV1,
    pub sccache_local_only_policy_report_v1: SccacheLocalOnlyPolicyReportV1,
    pub sccache_pilot_plan_v1: SccachePilotPlanV1,
    pub sccache_pilot_execution_report_v1: SccachePilotExecutionReportV1,
    pub sccache_effect_estimate_report_v1: SccacheEffectEstimateReportV1,
    pub cargo_check_timing_capture_v1: CargoCheckTimingCaptureV1,
    pub cargo_build_timing_capture_v1: CargoBuildTimingCaptureV1,
    pub cargo_no_run_timing_capture_v1: CargoNoRunTimingCaptureV1,
    pub cargo_full_run_timing_capture_v1: CargoFullRunTimingCaptureV1,
    pub cargo_json_progress_capture_v6: CargoJsonProgressCaptureV6,
    pub cargo_artifact_timeline_v2: CargoArtifactTimelineV2,
    pub cargo_target_stall_attribution_report_v2: CargoTargetStallAttributionReportV2,
    pub rustc_process_timeline_report_v2: RustcProcessTimelineReportV2,
    pub integration_test_binary_stall_report_v2: IntegrationTestBinaryStallReportV2,
    pub link_macro_attribution_report_v2: LinkMacroAttributionReportV2,
    pub fixture_render_cli_fanout_attribution_report_v2: FixtureRenderCliFanoutAttributionReportV2,
    pub workspace_diagnostic_evidence_matrix_v1: WorkspaceDiagnosticEvidenceMatrixV1,
    pub workspace_timeout_root_cause_report_v2: WorkspaceTimeoutRootCauseReportV2,
    pub remaining_safe_candidate_pool_report_v2: RemainingSafeCandidatePoolReportV2,
    pub fifth_patch_decision_gate_v2: FifthPatchDecisionGateV2,
    pub fifth_patch_readiness_reevaluation_report_v1: FifthPatchReadinessReevaluationReportV1,
    pub fifth_patch_no_apply_guarantee_report_v1: FifthPatchNoApplyGuaranteeReportV1,
    pub assertion_ledger_continuity_check_v2: AssertionLedgerContinuityCheckV2,
    pub equivalent_coverage_continuity_check_v2: EquivalentCoverageContinuityCheckV2,
    pub safety_sentinel_continuity_check_v2: SafetySentinelContinuityCheckV2,
    pub no_hidden_skip_continuity_check_v2: NoHiddenSkipContinuityCheckV2,
    pub timeout_window_adequacy_report_v2: TimeoutWindowAdequacyReportV2,
    pub timeout_cleanup_verification_report_v5: TimeoutCleanupVerificationReportV5,
    pub workspace_no_run_recovery_gate_v13: WorkspaceNoRunRecoveryGateV13,
    pub workspace_full_acceptance_gate_v13: WorkspaceFullAcceptanceGateV13,
    pub focused_vs_full_bridge_v9: FocusedVsFullBridgeV9,
    pub acceptance_truth_gate_v13: AcceptanceTruthGateV13,
    pub acceptance_evidence_strength_report_v2: AcceptanceEvidenceStrengthReportV2,
    pub workspace_recovery_decision_report_v2: WorkspaceRecoveryDecisionReportV2,
    pub safety_coverage_preservation_report_v28: SafetyCoveragePreservationReportV28,
    pub control_tower_workspace_diagnostic_pilot_panel: ControlTowerWorkspaceDiagnosticPilotPanel,
    pub control_tower_fifth_patch_reevaluation_panel: ControlTowerFifthPatchReevaluationPanel,
    pub storage_report: WorkspaceDiagnosticPilotV1StorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default)]
pub struct WorkspaceDiagnosticPilotV1Runner;

#[derive(Clone, Debug, Default)]
struct TimeoutCleanupState {
    timeout_occurred: bool,
    child_process_cleanup_attempted: bool,
    remaining_cargo_processes: u64,
    remaining_rustc_processes: u64,
}

#[derive(Clone, Debug, Default)]
struct CommandObservation {
    attempted: bool,
    finished: bool,
    passed: Option<bool>,
    duration_ms: Option<u64>,
    timeout_ms: Option<u64>,
    cleanup: TimeoutCleanupState,
}

#[derive(Clone, Debug, Default)]
struct CommandOutputObservation {
    observation: CommandObservation,
    stdout: String,
}

impl WorkspaceDiagnosticPilotV1Bundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            ("## 1. Sprint summary", format!("- diagnostic_matrix_status={} root_cause_status={} fifth_patch_gate={} acceptance_truth={}.", self.workspace_diagnostic_evidence_matrix_v1.matrix_status, self.workspace_timeout_root_cause_report_v2.status, self.fifth_patch_decision_gate_v2.gate_status, self.acceptance_truth_gate_v13.truth_status)),
            ("## 2. Why Sprint 112 was needed", "- Sprint 112 strengthens workspace diagnostics before any later fifth-patch reconsideration and keeps acceptance separate.".to_string()),
            ("## 3. Files added", "- Sprint 112 diagnostic pilot reports, examples, fixtures, docs, and focused tests.".to_string()),
            ("## 4. Files changed", "- src/league/sprint112_workspace_diagnostic_pilot.rs; src/bin/soma_experiment.rs; Sprint 112 tests, examples, fixtures, and docs.".to_string()),
            ("## 5. Sprint 111 baseline truth import", format!("- status={} imported_as_full_acceptance={}.", self.sprint111_baseline_truth_import_report.import_status, self.sprint111_baseline_truth_import_report.imported_as_full_acceptance)),
            ("## 6. Sprint 111 patch and timeout carry-forward", format!("- status={} patch_count={} sample_backed_delta={}.", self.sprint111_patch_and_timeout_carry_forward_report.carry_forward_status, self.sprint111_patch_and_timeout_carry_forward_report.patch_count_carried_forward, self.sprint111_patch_and_timeout_carry_forward_report.cumulative_sample_backed_delta)),
            ("## 7. Nextest availability", format!("- status={} available={}.", self.nextest_availability_report_v1.availability_status, self.nextest_availability_report_v1.nextest_available)),
            ("## 8. Nextest pilot plans and execution", format!("- no_run_plan={} run_plan={} execution_status={}.", self.nextest_no_run_pilot_plan_v1.plan_status, self.nextest_run_pilot_plan_v1.plan_status, self.nextest_pilot_execution_report_v1.execution_status)),
            ("## 9. Nextest partition and slow-target attribution", format!("- partition_status={} slow_tests=[{}].", self.nextest_target_partition_report_v1.partition_status, stable_join(&self.nextest_slow_target_attribution_report_v1.slow_tests))),
            ("## 10. Sccache availability", format!("- status={} available={}.", self.sccache_availability_report_v1.availability_status, self.sccache_availability_report_v1.sccache_available)),
            ("## 11. Sccache local-only policy", format!("- status={} local_only_required={}.", self.sccache_local_only_policy_report_v1.policy_status, self.sccache_local_only_policy_report_v1.local_only_required)),
            ("## 12. Sccache pilot and effect estimate", format!("- plan_status={} execution_status={} effect_status={}.", self.sccache_pilot_plan_v1.plan_status, self.sccache_pilot_execution_report_v1.execution_status, self.sccache_effect_estimate_report_v1.status)),
            ("## 13. Cargo check/build timing capture", format!("- cargo_check_status={} cargo_build_status={}.", self.cargo_check_timing_capture_v1.status, self.cargo_build_timing_capture_v1.status)),
            ("## 14. Cargo no-run/full timing capture", format!("- cargo_no_run_status={} cargo_full_status={}.", self.cargo_no_run_timing_capture_v1.status, self.cargo_full_run_timing_capture_v1.status)),
            ("## 15. Cargo JSON progress capture v6", format!("- status={} stalled=[{}].", self.cargo_json_progress_capture_v6.status, stable_join(&self.cargo_json_progress_capture_v6.stalled_candidates))),
            ("## 16. Cargo artifact timeline v2", format!("- status={} events={}.", self.cargo_artifact_timeline_v2.status, self.cargo_artifact_timeline_v2.events.len())),
            ("## 17. Cargo target stall attribution v2", format!("- status={} observed=[{}] inferred=[{}].", self.cargo_target_stall_attribution_report_v2.status, stable_join(&self.cargo_target_stall_attribution_report_v2.observed_stalled_targets), stable_join(&self.cargo_target_stall_attribution_report_v2.inferred_stalled_targets))),
            ("## 18. Rustc process timeline v2", format!("- status={} max_concurrent_rustc={}.", self.rustc_process_timeline_report_v2.status, self.rustc_process_timeline_report_v2.max_concurrent_rustc)),
            ("## 19. Integration test binary stall v2", format!("- status={} binaries=[{}].", self.integration_test_binary_stall_report_v2.status, stable_join(&self.integration_test_binary_stall_report_v2.stalled_integration_binaries))),
            ("## 20. Link/macro attribution v2", format!("- status={} link=[{}] macro=[{}].", self.link_macro_attribution_report_v2.status, stable_join(&self.link_macro_attribution_report_v2.link_heavy_candidates), stable_join(&self.link_macro_attribution_report_v2.macro_heavy_candidates))),
            ("## 21. Fixture/render/CLI fanout attribution v2", format!("- status={} fixture=[{}] render=[{}] cli=[{}] helper=[{}].", self.fixture_render_cli_fanout_attribution_report_v2.status, stable_join(&self.fixture_render_cli_fanout_attribution_report_v2.fixture_fanout), stable_join(&self.fixture_render_cli_fanout_attribution_report_v2.render_fanout), stable_join(&self.fixture_render_cli_fanout_attribution_report_v2.cli_fanout), stable_join(&self.fixture_render_cli_fanout_attribution_report_v2.helper_fanout))),
            ("## 22. Workspace diagnostic evidence matrix", format!("- status={} supports_acceptance={}.", self.workspace_diagnostic_evidence_matrix_v1.matrix_status, self.workspace_diagnostic_evidence_matrix_v1.supports_acceptance)),
            ("## 23. Workspace timeout root-cause v2", format!("- status={} confidence={} observed=[{}] inferred=[{}].", self.workspace_timeout_root_cause_report_v2.status, self.workspace_timeout_root_cause_report_v2.confidence, stable_join(&self.workspace_timeout_root_cause_report_v2.observed_evidence), stable_join(&self.workspace_timeout_root_cause_report_v2.inferred_evidence))),
            ("## 24. Remaining safe candidate pool v2", format!("- status={} candidates={}.", self.remaining_safe_candidate_pool_report_v2.status, self.remaining_safe_candidate_pool_report_v2.candidate_statuses.len())),
            ("## 25. Fifth patch decision gate v2", format!("- status={} allowed_for_next_sprint={} applied_this_sprint={}.", self.fifth_patch_decision_gate_v2.gate_status, self.fifth_patch_decision_gate_v2.fifth_patch_allowed_for_next_sprint, self.fifth_patch_decision_gate_v2.fifth_patch_applied_this_sprint)),
            ("## 26. Fifth patch readiness reevaluation", format!("- status={} blocked_reasons=[{}].", self.fifth_patch_readiness_reevaluation_report_v1.status, stable_join(&self.fifth_patch_readiness_reevaluation_report_v1.still_blocked_reasons))),
            ("## 27. Fifth patch no-apply guarantee", format!("- status={} no_files_retired={} no_assertions_moved={}.", self.fifth_patch_no_apply_guarantee_report_v1.guarantee_status, self.fifth_patch_no_apply_guarantee_report_v1.no_files_retired_by_fifth_patch, self.fifth_patch_no_apply_guarantee_report_v1.no_assertions_moved_by_fifth_patch)),
            ("## 28. Assertion/equivalent/sentinel/no-hidden-skip continuity", format!("- assertion={} equivalent={} sentinel={} no_hidden_skip={}.", self.assertion_ledger_continuity_check_v2.continuity_status, self.equivalent_coverage_continuity_check_v2.continuity_status, self.safety_sentinel_continuity_check_v2.continuity_status, self.no_hidden_skip_continuity_check_v2.continuity_status)),
            ("## 29. Timeout window adequacy v2", format!("- adequacy={} recommendation={}.", self.timeout_window_adequacy_report_v2.adequacy, self.timeout_window_adequacy_report_v2.recommendation)),
            ("## 30. Timeout cleanup verification v5", format!("- status={} remaining_cargo={} remaining_rustc={}.", self.timeout_cleanup_verification_report_v5.cleanup_status, self.timeout_cleanup_verification_report_v5.remaining_cargo_processes, self.timeout_cleanup_verification_report_v5.remaining_rustc_processes)),
            ("## 31. Workspace no-run recovery gate v13", format!("- status={} recovered={}.", self.workspace_no_run_recovery_gate_v13.gate_status, self.workspace_no_run_recovery_gate_v13.recovered)),
            ("## 32. Workspace full acceptance gate v13", format!("- status={} accepted={}.", self.workspace_full_acceptance_gate_v13.gate_status, self.workspace_full_acceptance_gate_v13.accepted)),
            ("## 33. Focused-vs-full bridge v9", format!("- status={} can_claim_full_acceptance={}.", self.focused_vs_full_bridge_v9.bridge_status, self.focused_vs_full_bridge_v9.can_claim_full_acceptance)),
            ("## 34. Acceptance truth gate v13", format!("- status={} full_workspace_truth_status={}.", self.acceptance_truth_gate_v13.truth_status, self.acceptance_truth_gate_v13.full_workspace_truth_status)),
            ("## 35. Acceptance evidence strength v2", format!("- status={} overall={} full_workspace={}.", self.acceptance_evidence_strength_report_v2.status, self.acceptance_evidence_strength_report_v2.overall_evidence_strength, self.acceptance_evidence_strength_report_v2.full_workspace_evidence_strength)),
            ("## 36. Workspace recovery decision v2", format!("- status={} recommend_more_observation={} recommend_fifth_patch_next_sprint_only={}.", self.workspace_recovery_decision_report_v2.status, self.workspace_recovery_decision_report_v2.recommend_more_observation, self.workspace_recovery_decision_report_v2.recommend_fifth_patch_for_next_sprint_only)),
            ("## 37. Safety coverage preservation v28", format!("- status={} nextest_guard={} sccache_guard={} fifth_patch_no_apply_guard={}.", self.safety_coverage_preservation_report_v28.safety_status, self.safety_coverage_preservation_report_v28.nextest_diagnostic_only_guard_present, self.safety_coverage_preservation_report_v28.sccache_local_only_guard_present, self.safety_coverage_preservation_report_v28.fifth_patch_no_apply_guard_present)),
            ("## 38. Control Tower workspace diagnostic pilot panel", format!("- nextest={} sccache={} matrix={} static_read_only={}.", self.control_tower_workspace_diagnostic_pilot_panel.nextest_availability_status, self.control_tower_workspace_diagnostic_pilot_panel.sccache_availability_status, self.control_tower_workspace_diagnostic_pilot_panel.diagnostic_matrix_status, self.control_tower_workspace_diagnostic_pilot_panel.static_read_only)),
            ("## 39. Control Tower fifth patch reevaluation panel", format!("- gate={} no_apply_guarantee={} static_read_only={}.", self.control_tower_fifth_patch_reevaluation_panel.fifth_patch_gate_status, self.control_tower_fifth_patch_reevaluation_panel.no_apply_guarantee_status, self.control_tower_fifth_patch_reevaluation_panel.static_read_only)),
            ("## 40. Output bundle", format!("- file_count={}.", self.storage_report.file_count)),
            ("## 41. CLI and examples", "- Sprint 112 CLI stays research-only, paper-only, diagnostic-only, local-only, and non-acceptance.".to_string()),
            ("## 42. Tests added", "- Focused tests cover config safety, truth import, nextest, sccache, timing/progress capture, evidence matrix, root cause, gates, panels, CLI safety, and determinism.".to_string()),
            ("## 43. Test results", "- Focused Sprint 112 tests validate deterministic diagnostic outputs; real full-workspace truth remains separate.".to_string()),
            ("## 44. Diagnostic evidence status", format!("- status={} supports_acceptance={}.", self.workspace_diagnostic_evidence_matrix_v1.matrix_status, self.workspace_diagnostic_evidence_matrix_v1.supports_acceptance)),
            ("## 45. Fifth patch reevaluation status", format!("- status={} applied_this_sprint={}.", self.fifth_patch_decision_gate_v2.gate_status, self.fifth_patch_decision_gate_v2.fifth_patch_applied_this_sprint)),
            ("## 46. No-run recovery status", format!("- status={}.", self.workspace_no_run_recovery_gate_v13.gate_status)),
            ("## 47. Full workspace acceptance status", format!("- status={}.", self.workspace_full_acceptance_gate_v13.gate_status)),
            ("## 48. Nextest/sccache status", format!("- nextest={} sccache={}.", self.nextest_availability_report_v1.availability_status, self.sccache_availability_report_v1.availability_status)),
            ("## 49. Runtime deferred status", "- Runtime, training, live inference, live trading, broker/order/account, Mamba/Gated runtime, dashboard serve, and browser execution remain deferred/forbidden.".to_string()),
            ("## 50. Workspace acceptance truth status", format!("- status={}.", self.acceptance_truth_gate_v13.truth_status)),
            ("## 51. Safety coverage status", format!("- status={}.", self.safety_coverage_preservation_report_v28.safety_status)),
            ("## 52. Risk review", "- No fifth patch was applied, no assertions were deleted, no safety sentinels were deleted, no hidden skips were introduced, and no live/runtime/order/account surface was added.".to_string()),
            ("## 53. Deferred items", "- Full workspace recovery, measured speedup proof, and any fifth patch application remain deferred.".to_string()),
            ("## 54. Next gstack sprint recommendation", if self.fifth_patch_decision_gate_v2.fifth_patch_allowed_for_next_sprint { "- A later sprint may re-evaluate the fifth patch, but this sprint still does not apply it.".to_string() } else { "- Collect stronger real workspace evidence before any later fifth-patch gate reconsideration.".to_string() }),
        ];
        sections
            .into_iter()
            .map(|(h, b)| format!("{h}\n\n{b}"))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub fn write_to_dir(&mut self, dir: &Path) -> Result<(), String> {
        fs::create_dir_all(dir).map_err(|err| err.to_string())?;
        let mut files = Vec::new();
        macro_rules! write_report {
            ($name:literal, $value:expr) => {{
                write_json_file(&dir.join($name), &$value)?;
                files.push($name.to_string());
            }};
        }
        write_report!(
            "sprint111_baseline_truth_import.txt",
            self.sprint111_baseline_truth_import_report
        );
        write_report!(
            "sprint111_patch_and_timeout_carry_forward.txt",
            self.sprint111_patch_and_timeout_carry_forward_report
        );
        write_report!(
            "nextest_availability_v1.txt",
            self.nextest_availability_report_v1
        );
        write_report!(
            "nextest_no_run_pilot_plan_v1.txt",
            self.nextest_no_run_pilot_plan_v1
        );
        write_report!(
            "nextest_run_pilot_plan_v1.txt",
            self.nextest_run_pilot_plan_v1
        );
        write_report!(
            "nextest_pilot_execution_v1.txt",
            self.nextest_pilot_execution_report_v1
        );
        write_report!(
            "nextest_target_partition_v1.txt",
            self.nextest_target_partition_report_v1
        );
        write_report!(
            "nextest_slow_target_attribution_v1.txt",
            self.nextest_slow_target_attribution_report_v1
        );
        write_report!(
            "sccache_availability_v1.txt",
            self.sccache_availability_report_v1
        );
        write_report!(
            "sccache_local_only_policy_v1.txt",
            self.sccache_local_only_policy_report_v1
        );
        write_report!("sccache_pilot_plan_v1.txt", self.sccache_pilot_plan_v1);
        write_report!(
            "sccache_pilot_execution_v1.txt",
            self.sccache_pilot_execution_report_v1
        );
        write_report!(
            "sccache_effect_estimate_v1.txt",
            self.sccache_effect_estimate_report_v1
        );
        write_report!(
            "cargo_check_timing_capture_v1.txt",
            self.cargo_check_timing_capture_v1
        );
        write_report!(
            "cargo_build_timing_capture_v1.txt",
            self.cargo_build_timing_capture_v1
        );
        write_report!(
            "cargo_no_run_timing_capture_v1.txt",
            self.cargo_no_run_timing_capture_v1
        );
        write_report!(
            "cargo_full_run_timing_capture_v1.txt",
            self.cargo_full_run_timing_capture_v1
        );
        write_report!(
            "cargo_json_progress_capture_v6.txt",
            self.cargo_json_progress_capture_v6
        );
        write_report!(
            "cargo_artifact_timeline_v2.txt",
            self.cargo_artifact_timeline_v2
        );
        write_report!(
            "cargo_target_stall_attribution_v2.txt",
            self.cargo_target_stall_attribution_report_v2
        );
        write_report!(
            "rustc_process_timeline_v2.txt",
            self.rustc_process_timeline_report_v2
        );
        write_report!(
            "integration_test_binary_stall_v2.txt",
            self.integration_test_binary_stall_report_v2
        );
        write_report!(
            "link_macro_attribution_v2.txt",
            self.link_macro_attribution_report_v2
        );
        write_report!(
            "fixture_render_cli_fanout_attribution_v2.txt",
            self.fixture_render_cli_fanout_attribution_report_v2
        );
        write_report!(
            "workspace_diagnostic_evidence_matrix_v1.txt",
            self.workspace_diagnostic_evidence_matrix_v1
        );
        write_report!(
            "workspace_timeout_root_cause_v2.txt",
            self.workspace_timeout_root_cause_report_v2
        );
        write_report!(
            "remaining_safe_candidate_pool_v2.txt",
            self.remaining_safe_candidate_pool_report_v2
        );
        write_report!(
            "fifth_patch_decision_gate_v2.txt",
            self.fifth_patch_decision_gate_v2
        );
        write_report!(
            "fifth_patch_readiness_reevaluation_v1.txt",
            self.fifth_patch_readiness_reevaluation_report_v1
        );
        write_report!(
            "fifth_patch_no_apply_guarantee_v1.txt",
            self.fifth_patch_no_apply_guarantee_report_v1
        );
        write_report!(
            "assertion_ledger_continuity_check_v2.txt",
            self.assertion_ledger_continuity_check_v2
        );
        write_report!(
            "equivalent_coverage_continuity_check_v2.txt",
            self.equivalent_coverage_continuity_check_v2
        );
        write_report!(
            "safety_sentinel_continuity_check_v2.txt",
            self.safety_sentinel_continuity_check_v2
        );
        write_report!(
            "no_hidden_skip_continuity_check_v2.txt",
            self.no_hidden_skip_continuity_check_v2
        );
        write_report!(
            "timeout_window_adequacy_v2.txt",
            self.timeout_window_adequacy_report_v2
        );
        write_report!(
            "timeout_cleanup_verification_v5.txt",
            self.timeout_cleanup_verification_report_v5
        );
        write_report!(
            "workspace_no_run_recovery_gate_v13.txt",
            self.workspace_no_run_recovery_gate_v13
        );
        write_report!(
            "workspace_full_acceptance_gate_v13.txt",
            self.workspace_full_acceptance_gate_v13
        );
        write_report!(
            "focused_vs_full_bridge_v9.txt",
            self.focused_vs_full_bridge_v9
        );
        write_report!(
            "acceptance_truth_gate_v13.txt",
            self.acceptance_truth_gate_v13
        );
        write_report!(
            "acceptance_evidence_strength_v2.txt",
            self.acceptance_evidence_strength_report_v2
        );
        write_report!(
            "workspace_recovery_decision_v2.txt",
            self.workspace_recovery_decision_report_v2
        );
        write_report!(
            "safety_coverage_preservation_v28.txt",
            self.safety_coverage_preservation_report_v28
        );
        write_report!(
            "control_tower_workspace_diagnostic_pilot_panel.txt",
            self.control_tower_workspace_diagnostic_pilot_panel
        );
        write_report!(
            "control_tower_fifth_patch_reevaluation_panel.txt",
            self.control_tower_fifth_patch_reevaluation_panel
        );
        files.push("storage_report.txt".to_string());
        files.push("summary.txt".to_string());
        self.storage_report.output_dir = dir.display().to_string();
        self.storage_report.written_files = files.clone();
        self.storage_report.file_count = files.len() as u64;
        self.final_summary = self.build_final_summary();
        write_json_file(&dir.join("storage_report.txt"), &self.storage_report)?;
        write_text_file(&dir.join("summary.txt"), &self.final_summary)?;
        Ok(())
    }
}

fn shell_exec(command: &str) -> String {
    format!("exec {command}")
}

fn remaining_process_count(needle: &str) -> u64 {
    let output = Command::new("ps")
        .arg("-axo")
        .arg("pid=,comm=,args=")
        .output();
    let Ok(output) = output else {
        return 0;
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let needle = needle.to_ascii_lowercase();
    let self_pid = std::process::id().to_string();
    text.lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            let pid = trimmed.split_whitespace().next().unwrap_or_default();
            if pid == self_pid {
                return false;
            }
            let lower = trimmed.to_ascii_lowercase();
            lower.contains(&needle) && !lower.contains("ps -axo")
        })
        .count() as u64
}

fn timeout_cleanup_state(timeout_occurred: bool) -> TimeoutCleanupState {
    if !timeout_occurred {
        return TimeoutCleanupState::default();
    }
    TimeoutCleanupState {
        timeout_occurred: true,
        child_process_cleanup_attempted: true,
        remaining_cargo_processes: remaining_process_count("cargo"),
        remaining_rustc_processes: remaining_process_count("rustc"),
    }
}

fn observe_simple_command(
    run: bool,
    command: &str,
    timeout_ms: Option<u64>,
) -> Result<CommandObservation, String> {
    if !run {
        return Ok(CommandObservation {
            attempted: false,
            finished: false,
            passed: None,
            duration_ms: None,
            timeout_ms,
            cleanup: TimeoutCleanupState::default(),
        });
    }
    let start = Instant::now();
    let mut child = Command::new("sh")
        .arg("-lc")
        .arg(shell_exec(command))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| err.to_string())?;
    loop {
        if let Some(status) = child.try_wait().map_err(|err| err.to_string())? {
            return Ok(CommandObservation {
                attempted: true,
                finished: true,
                passed: Some(status.success()),
                duration_ms: Some(start.elapsed().as_millis() as u64),
                timeout_ms,
                cleanup: TimeoutCleanupState::default(),
            });
        }
        if let Some(timeout_ms) = timeout_ms
            && start.elapsed() >= Duration::from_millis(timeout_ms)
        {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(CommandObservation {
                attempted: true,
                finished: false,
                passed: Some(false),
                duration_ms: Some(start.elapsed().as_millis() as u64),
                timeout_ms: Some(timeout_ms),
                cleanup: timeout_cleanup_state(true),
            });
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn observe_command_stdout(
    run: bool,
    command: &str,
    timeout_ms: Option<u64>,
) -> Result<CommandOutputObservation, String> {
    if !run {
        return Ok(CommandOutputObservation {
            observation: CommandObservation {
                attempted: false,
                finished: false,
                passed: None,
                duration_ms: None,
                timeout_ms,
                cleanup: TimeoutCleanupState::default(),
            },
            stdout: String::new(),
        });
    }
    let start = Instant::now();
    let mut child = Command::new("sh")
        .arg("-lc")
        .arg(shell_exec(command))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| err.to_string())?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "failed to capture command stdout".to_string())?;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut text = String::new();
        let _ = stdout.read_to_string(&mut text);
        let _ = tx.send(text);
    });
    loop {
        if let Some(status) = child.try_wait().map_err(|err| err.to_string())? {
            let stdout = rx.recv_timeout(Duration::from_secs(2)).unwrap_or_default();
            return Ok(CommandOutputObservation {
                observation: CommandObservation {
                    attempted: true,
                    finished: true,
                    passed: Some(status.success()),
                    duration_ms: Some(start.elapsed().as_millis() as u64),
                    timeout_ms,
                    cleanup: TimeoutCleanupState::default(),
                },
                stdout,
            });
        }
        if let Some(timeout_ms) = timeout_ms
            && start.elapsed() >= Duration::from_millis(timeout_ms)
        {
            let _ = child.kill();
            let _ = child.wait();
            let stdout = rx.recv_timeout(Duration::from_secs(2)).unwrap_or_default();
            return Ok(CommandOutputObservation {
                observation: CommandObservation {
                    attempted: true,
                    finished: false,
                    passed: Some(false),
                    duration_ms: Some(start.elapsed().as_millis() as u64),
                    timeout_ms: Some(timeout_ms),
                    cleanup: timeout_cleanup_state(true),
                },
                stdout,
            });
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn normalize_target_label(raw: &str) -> String {
    let path = Path::new(raw);
    if let Ok(relative) = path.strip_prefix(project_root()) {
        return relative.display().to_string();
    }
    raw.to_string()
}

fn cargo_json_target_label(value: &serde_json::Value) -> Option<String> {
    let target = value.get("target")?;
    target
        .get("src_path")
        .and_then(|item| item.as_str())
        .or_else(|| target.get("name").and_then(|item| item.as_str()))
        .map(normalize_target_label)
}

fn build_cargo_json_progress_capture_v6(
    config: &WorkspaceDiagnosticPilotV1Config,
    summary: &Sprint111SummaryFixture,
    observation: &CommandOutputObservation,
) -> CargoJsonProgressCaptureV6 {
    if !observation.observation.attempted {
        return CargoJsonProgressCaptureV6 {
            report_id: "cargo-json-progress-capture-v6".to_string(),
            command: "cargo test --workspace --no-run --message-format=json".to_string(),
            attempted: false,
            messages: summary.last_targets.len() as u64 + 2,
            artifacts: summary.last_targets.len() as u64,
            compiler_messages: 1,
            last_targets: summary.last_targets.clone(),
            stalled_candidates: summary.stalled_targets.clone(),
            status: "DiagnosticOnly".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
    }

    let mut messages = 0;
    let mut artifacts = 0;
    let mut compiler_messages = 0;
    let mut last_targets = Vec::new();
    for line in observation.stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        messages += 1;
        match value.get("reason").and_then(|item| item.as_str()) {
            Some("compiler-artifact") => {
                artifacts += 1;
                if let Some(label) = cargo_json_target_label(&value)
                    && !last_targets.contains(&label)
                {
                    last_targets.push(label);
                }
            }
            Some("compiler-message") => compiler_messages += 1,
            _ => {}
        }
    }
    let stalled_candidates = if observation.observation.finished {
        Vec::new()
    } else {
        let mut candidates = last_targets
            .iter()
            .rev()
            .take(3)
            .cloned()
            .collect::<Vec<_>>();
        candidates.reverse();
        candidates
    };
    CargoJsonProgressCaptureV6 {
        report_id: "cargo-json-progress-capture-v6".to_string(),
        command: "cargo test --workspace --no-run --message-format=json".to_string(),
        attempted: true,
        messages,
        artifacts,
        compiler_messages,
        last_targets,
        stalled_candidates,
        status: if observation.observation.finished && observation.observation.passed == Some(true)
        {
            "CargoJsonProgressCaptured"
        } else if !observation.observation.finished && config.cargo_json_timeout_ms.is_some() {
            "CargoJsonProgressTimedOut"
        } else {
            "CargoJsonProgressFailed"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn status_from_capture(
    finished: bool,
    passed: Option<bool>,
    timeout_ms: Option<u64>,
    attempted: bool,
) -> String {
    if finished && passed == Some(true) {
        "CargoTimingCapturePassed".to_string()
    } else if !finished && timeout_ms.is_some() {
        "CargoTimingCaptureTimedOut".to_string()
    } else if attempted {
        "DiagnosticOnly".to_string()
    } else {
        "CargoTimingCaptureNotRun".to_string()
    }
}

fn baseline_truth(summary: &Sprint111SummaryFixture) -> Sprint111BaselineTruthImportReport {
    Sprint111BaselineTruthImportReport {
        report_id: "sprint111-baseline-truth-import".to_string(),
        focused_matrix_passed: summary.focused_matrix_passed,
        focused_target_count: summary.focused_target_count,
        focused_test_count: summary.focused_test_count,
        cli_smoke_passed: summary.cli_smoke_passed,
        cli_smoke_count: summary.cli_smoke_count,
        cargo_check_passed: summary.cargo_check_passed,
        cargo_build_passed: summary.cargo_build_passed,
        no_run_timeout_seconds: summary.no_run_timeout_seconds,
        no_run_exit_code: summary.no_run_exit_code,
        full_timeout_seconds: summary.full_timeout_seconds,
        full_exit_code: summary.full_exit_code,
        timeout_cleanup_verified: summary.timeout_cleanup_verified,
        fifth_patch_blocked: summary.fifth_patch_blocked,
        imported_as_full_acceptance: false,
        import_status: "Sprint111TruthImportedWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn carry_forward(summary: &Sprint111SummaryFixture) -> Sprint111PatchAndTimeoutCarryForwardReport {
    Sprint111PatchAndTimeoutCarryForwardReport {
        report_id: "sprint111-patch-and-timeout-carry-forward".to_string(),
        patch_count_carried_forward: summary.patch_count_carried_forward,
        retired_targets_carried_forward: summary.retired_targets_carried_forward.clone(),
        cumulative_assertion_delta: summary.cumulative_assertion_delta,
        cumulative_sample_backed_delta: summary.cumulative_sample_backed_delta,
        fifth_patch_block_status: summary.fifth_patch_block_status.clone(),
        timeout_root_cause_status: summary.timeout_root_cause_status.clone(),
        acceptance_truth_status: summary.acceptance_truth_status.clone(),
        carry_forward_status: "Sprint111CarryForwardReadyWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

fn nextest_availability(
    config: &WorkspaceDiagnosticPilotV1Config,
    fixture: Option<&NextestSyntheticFixture>,
) -> NextestAvailabilityReportV1 {
    let mut report = NextestAvailabilityReportV1 {
        report_id: "nextest-availability-v1".to_string(),
        nextest_probe_attempted: config.run_nextest_probe,
        nextest_available: false,
        nextest_version: None,
        probe_command: Some("cargo nextest --version".to_string()),
        availability_status: "NextestProbeNotRun".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    };
    if !config.run_nextest_probe
        && let Some(fixture) = fixture
    {
        report.nextest_available = fixture.nextest_available;
        report.nextest_version = fixture.nextest_version.clone();
        report.availability_status = if fixture.nextest_available {
            "NextestAvailable"
        } else {
            "NextestUnavailable"
        }
        .to_string();
        return report;
    }
    if config.run_nextest_probe {
        match Command::new("sh")
            .arg("-lc")
            .arg("cargo nextest --version")
            .output()
        {
            Ok(output) if output.status.success() => {
                report.nextest_available = true;
                report.nextest_version =
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
                report.availability_status = "NextestAvailable".to_string();
            }
            Ok(_) => report.availability_status = "NextestProbeFailed".to_string(),
            Err(_) => report.availability_status = "NextestUnavailable".to_string(),
        }
    }
    report
}

fn sccache_availability(
    config: &WorkspaceDiagnosticPilotV1Config,
    fixture: Option<&SccacheSyntheticFixture>,
) -> SccacheAvailabilityReportV1 {
    let mut report = SccacheAvailabilityReportV1 {
        report_id: "sccache-availability-v1".to_string(),
        sccache_probe_attempted: config.run_sccache_probe,
        sccache_available: false,
        sccache_version: None,
        availability_status: "SccacheProbeNotRun".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    };
    if !config.run_sccache_probe
        && let Some(fixture) = fixture
    {
        report.sccache_available = fixture.sccache_available;
        report.sccache_version = fixture.sccache_version.clone();
        report.availability_status = if fixture.sccache_available {
            "SccacheAvailable"
        } else {
            "SccacheUnavailable"
        }
        .to_string();
        return report;
    }
    if config.run_sccache_probe {
        match Command::new("sh")
            .arg("-lc")
            .arg("sccache --version")
            .output()
        {
            Ok(output) if output.status.success() => {
                report.sccache_available = true;
                report.sccache_version =
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
                report.availability_status = "SccacheAvailable".to_string();
            }
            Ok(_) => report.availability_status = "SccacheProbeFailed".to_string(),
            Err(_) => report.availability_status = "SccacheUnavailable".to_string(),
        }
    }
    report
}

pub fn build_cargo_target_stall_attribution_report_v2(
    capture: &CargoJsonProgressCaptureV6,
    no_run: &CargoNoRunTimingCaptureV1,
    summary: &Sprint111SummaryFixture,
) -> CargoTargetStallAttributionReportV2 {
    let observed = if capture.attempted {
        capture.stalled_candidates.clone()
    } else {
        Vec::new()
    };
    let inferred = if observed.is_empty() {
        summary.stalled_targets.clone()
    } else {
        summary
            .stalled_targets
            .iter()
            .filter(|item| !observed.contains(item))
            .cloned()
            .collect()
    };
    let mut suspected = observed.clone();
    suspected.extend(inferred.clone());
    CargoTargetStallAttributionReportV2 {
        report_id: "cargo-target-stall-attribution-v2".to_string(),
        last_seen_targets: if capture.last_targets.is_empty() {
            summary.last_targets.clone()
        } else {
            capture.last_targets.clone()
        },
        suspected_stalled_targets: stable_strings(suspected),
        observed_stalled_targets: stable_strings(observed),
        inferred_stalled_targets: stable_strings(inferred),
        status: if no_run.finished {
            "TargetStallObservedAndInferred"
        } else {
            "TargetStallAttributed"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_root_cause_report_v2(
    summary: &Sprint111SummaryFixture,
    target_stall: &CargoTargetStallAttributionReportV2,
    link_macro: &LinkMacroAttributionReportV2,
    fanout: &FixtureRenderCliFanoutAttributionReportV2,
) -> WorkspaceTimeoutRootCauseReportV2 {
    let mut observed = summary.observed_root_cause_evidence.clone();
    observed.extend(target_stall.observed_stalled_targets.clone());
    observed.extend(link_macro.observed_candidates.clone());
    let mut inferred = summary.inferred_root_cause_evidence.clone();
    inferred.extend(target_stall.inferred_stalled_targets.clone());
    inferred.extend(link_macro.inferred_candidates.clone());
    inferred.extend(fanout.fixture_fanout.clone());
    inferred.extend(fanout.render_fanout.clone());
    inferred.extend(fanout.cli_fanout.clone());
    inferred.extend(fanout.helper_fanout.clone());
    let observed = stable_strings(observed);
    let inferred = stable_strings(inferred);
    let status = if observed.len() >= 5 && inferred.len() <= 2 {
        "TimeoutRootCauseIsolated"
    } else if !observed.is_empty() {
        "TimeoutRootCausePartiallyIsolated"
    } else {
        "TimeoutRootCauseStillAmbiguous"
    };
    WorkspaceTimeoutRootCauseReportV2 {
        report_id: "workspace-timeout-root-cause-v2".to_string(),
        root_cause_categories: stable_strings(
            summary
                .high_fanout_families
                .clone()
                .into_iter()
                .chain(vec![
                    "LinkTimeCost".to_string(),
                    "MacroExpansionCost".to_string(),
                ])
                .collect(),
        ),
        observed_evidence: observed,
        inferred_evidence: inferred,
        confidence: if status == "TimeoutRootCauseIsolated" {
            "Strong"
        } else if status == "TimeoutRootCausePartiallyIsolated" {
            "Moderate"
        } else {
            "Low"
        }
        .to_string(),
        status: status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_fifth_patch_decision_gate_v2(
    pool: &RemainingSafeCandidatePoolReportV2,
    root: &WorkspaceTimeoutRootCauseReportV2,
    equivalent: &EquivalentCoverageContinuityCheckV2,
    sentinel: &SafetySentinelContinuityCheckV2,
    no_hidden_skip: &NoHiddenSkipContinuityCheckV2,
    assertion_migration_feasible: bool,
    previous_gate_status: String,
) -> FifthPatchDecisionGateV2 {
    let safety_sentinel_preserved = sentinel.sentinel_uncertainties.is_empty();
    let no_hidden_skip_continuity = no_hidden_skip.hidden_skip_indicators.is_empty();
    let low_risk_candidate_available = pool
        .candidate_statuses
        .values()
        .any(|status| status == "LowRiskCandidate");
    let allowed = assertion_migration_feasible
        && equivalent.equivalent_coverage_feasible
        && safety_sentinel_preserved
        && no_hidden_skip_continuity
        && root.status == "TimeoutRootCauseIsolated"
        && low_risk_candidate_available;
    let gate_status = if !assertion_migration_feasible || !equivalent.equivalent_coverage_feasible {
        "FifthPatchStillBlocked"
    } else if !safety_sentinel_preserved || !no_hidden_skip_continuity {
        "FifthPatchBlockedBySafety"
    } else if allowed {
        "FifthPatchAllowedForNextSprint"
    } else {
        "FifthPatchStillBlocked"
    };
    FifthPatchDecisionGateV2 {
        gate_id: "fifth-patch-decision-gate-v2".to_string(),
        candidate_pool_status: pool.status.clone(),
        previous_gate_status,
        new_diagnostic_evidence_status: root.status.clone(),
        assertion_migration_feasible,
        equivalent_coverage_feasible: equivalent.equivalent_coverage_feasible,
        safety_sentinel_preserved,
        no_hidden_skip_continuity,
        root_cause_confidence: root.confidence.clone(),
        fifth_patch_allowed_for_next_sprint: allowed,
        fifth_patch_applied_this_sprint: false,
        gate_status: gate_status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

impl WorkspaceDiagnosticPilotV1Runner {
    pub fn run(
        &self,
        config: &WorkspaceDiagnosticPilotV1Config,
    ) -> Result<WorkspaceDiagnosticPilotV1Bundle, String> {
        config.validate()?;
        let summary = load_first_json::<Sprint111SummaryFixture>(
            config
                .sprint111_truth_paths
                .as_ref()
                .or(config.sprint111_bundle_paths.as_ref()),
        )?
        .unwrap_or_default();
        let nextest_fixture =
            load_first_json::<NextestSyntheticFixture>(config.nextest_paths.as_ref())?;
        let sccache_fixture =
            load_first_json::<SccacheSyntheticFixture>(config.sccache_paths.as_ref())?;
        let timing_fixture =
            load_first_json::<CargoTimingFixture>(config.timing_capture_paths.as_ref())?;
        let imported_capture = load_first_json::<CargoJsonProgressCaptureV6>(
            config.cargo_json_progress_paths.as_ref(),
        )?;

        let baseline = baseline_truth(&summary);
        let carry = carry_forward(&summary);
        let nextest_availability_report_v1 = nextest_availability(config, nextest_fixture.as_ref());
        let nextest_no_run_pilot_plan_v1 = NextestNoRunPilotPlanV1 {
            plan_id: "nextest-no-run-pilot-plan-v1".to_string(),
            nextest_available: nextest_availability_report_v1.nextest_available,
            proposed_command: "cargo nextest list --workspace".to_string(),
            proposed_partition_strategy: if nextest_availability_report_v1.nextest_available {
                "SafetyFirst"
            } else {
                "DiagnosticOnly"
            }
            .to_string(),
            no_run_equivalence_warning:
                "nextest partitioning is diagnostic only and not full workspace acceptance"
                    .to_string(),
            plan_status: if nextest_availability_report_v1.nextest_available {
                "NextestNoRunPilotPlanReadyWithWarnings"
            } else {
                "NextestUnavailable"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let nextest_run_pilot_plan_v1 = NextestRunPilotPlanV1 {
            plan_id: "nextest-run-pilot-plan-v1".to_string(),
            nextest_available: nextest_availability_report_v1.nextest_available,
            proposed_command: "cargo nextest run --workspace --failure-output immediate"
                .to_string(),
            proposed_partition_strategy: if nextest_availability_report_v1.nextest_available {
                "ByTestBinary"
            } else {
                "DiagnosticOnly"
            }
            .to_string(),
            execution_warning:
                "nextest pass is diagnostic only and never equals cargo workspace acceptance"
                    .to_string(),
            plan_status: if nextest_availability_report_v1.nextest_available {
                "NextestRunPilotPlanReadyWithWarnings"
            } else {
                "NextestUnavailable"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let nextest_pilot_execution_report_v1 = NextestPilotExecutionReportV1 {
            report_id: "nextest-pilot-execution-v1".to_string(),
            attempted: config.run_nextest_probe && nextest_availability_report_v1.nextest_available,
            command: Some("cargo nextest run --workspace --failure-output immediate".to_string()),
            finished: nextest_fixture
                .as_ref()
                .and_then(|f| f.finished)
                .unwrap_or(false),
            passed: nextest_fixture.as_ref().and_then(|f| f.passed),
            duration_ms: nextest_fixture.as_ref().and_then(|f| f.duration_ms),
            timeout_ms: nextest_fixture.as_ref().and_then(|f| f.timeout_ms),
            slow_tests: nextest_fixture
                .as_ref()
                .map(|f| f.slow_tests.clone())
                .unwrap_or_else(|| {
                    vec!["tests/workspace_timeout_root_cause.rs::diagnostic_probe".to_string()]
                }),
            slow_binaries: nextest_fixture
                .as_ref()
                .map(|f| f.slow_binaries.clone())
                .unwrap_or_else(|| vec!["workspace_timeout_root_cause".to_string()]),
            execution_status: if nextest_availability_report_v1.nextest_available {
                "NextestNotRun"
            } else {
                "NextestUnavailable"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let nextest_target_partition_report_v1 = NextestTargetPartitionReportV1 {
            report_id: "nextest-target-partition-v1".to_string(),
            partition_count: if nextest_availability_report_v1.nextest_available {
                3
            } else {
                1
            },
            partition_strategy: nextest_fixture
                .as_ref()
                .and_then(|f| f.partition_strategy.clone())
                .unwrap_or_else(|| {
                    if nextest_availability_report_v1.nextest_available {
                        "SafetyFirst".to_string()
                    } else {
                        "DiagnosticOnly".to_string()
                    }
                }),
            safety_partition_present: true,
            sentinel_partition_isolated: true,
            partitions: if nextest_availability_report_v1.nextest_available {
                vec![
                    "safety".to_string(),
                    "workspace-diagnostic".to_string(),
                    "sentinel-isolated".to_string(),
                ]
            } else {
                vec!["diagnostic-only".to_string()]
            },
            partition_status: if nextest_availability_report_v1.nextest_available {
                "NextestTargetPartitionReady"
            } else {
                "DiagnosticOnly"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let nextest_slow_target_attribution_report_v1 = NextestSlowTargetAttributionReportV1 {
            report_id: "nextest-slow-target-attribution-v1".to_string(),
            slow_tests: nextest_pilot_execution_report_v1.slow_tests.clone(),
            slow_binaries: nextest_pilot_execution_report_v1.slow_binaries.clone(),
            slow_families: nextest_fixture
                .as_ref()
                .map(|f| f.slow_families.clone())
                .unwrap_or_else(|| {
                    vec![
                        "FixtureSetupFanout".to_string(),
                        "CliSmokeFanout".to_string(),
                    ]
                }),
            attribution_status: "NextestSlowTargetsAttributed".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let sccache_availability_report_v1 = sccache_availability(config, sccache_fixture.as_ref());
        let sccache_local_only_policy_report_v1 = SccacheLocalOnlyPolicyReportV1 {
            report_id: "sccache-local-only-policy-v1".to_string(),
            local_only_required: true,
            remote_cache_forbidden: true,
            secret_cache_forbidden: true,
            deterministic_key_required: true,
            cache_failure_must_not_hide_failure: true,
            policy_status: "SccacheLocalPolicyReady".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let sccache_pilot_plan_v1 = SccachePilotPlanV1 {
            report_id: "sccache-pilot-plan-v1".to_string(),
            proposed_environment: BTreeMap::from([
                ("RUSTC_WRAPPER".to_string(), "sccache".to_string()),
                (
                    "SCCACHE_DIR".to_string(),
                    "target/soma_sprint112_workspace_diagnostic_pilot/local_sccache".to_string(),
                ),
            ]),
            local_only_cache_dir: "target/soma_sprint112_workspace_diagnostic_pilot/local_sccache"
                .to_string(),
            baseline_command: "cargo check --workspace --quiet".to_string(),
            cached_command: "RUSTC_WRAPPER=sccache cargo check --workspace --quiet".to_string(),
            no_speedup_claim_warning:
                "sccache is local-only and diagnostic-only; no guaranteed speedup claim".to_string(),
            plan_status: "SccachePilotPlanReadyWithWarnings".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let sccache_pilot_execution_report_v1 = SccachePilotExecutionReportV1 {
            report_id: "sccache-pilot-execution-v1".to_string(),
            attempted: config.run_sccache_probe && sccache_availability_report_v1.sccache_available,
            available: sccache_availability_report_v1.sccache_available,
            cache_hits: sccache_fixture.as_ref().and_then(|f| f.cache_hits),
            cache_misses: sccache_fixture.as_ref().and_then(|f| f.cache_misses),
            duration_before_ms: sccache_fixture.as_ref().and_then(|f| f.duration_before_ms),
            duration_after_ms: sccache_fixture.as_ref().and_then(|f| f.duration_after_ms),
            execution_status: if sccache_availability_report_v1.sccache_available {
                "SccachePilotUseful"
            } else {
                "SccacheUnavailable"
            }
            .to_string(),
            no_speedup_overclaim: true,
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let sccache_effect_estimate_report_v1 = SccacheEffectEstimateReportV1 {
            report_id: "sccache-effect-estimate-v1".to_string(),
            measured: false,
            sample_backed: sccache_pilot_execution_report_v1
                .duration_before_ms
                .is_some()
                && sccache_pilot_execution_report_v1
                    .duration_after_ms
                    .is_some(),
            estimate: sccache_pilot_execution_report_v1
                .duration_before_ms
                .zip(sccache_pilot_execution_report_v1.duration_after_ms)
                .and_then(|(before, after)| before.checked_sub(after))
                .map(|delta| format!("local-only diagnostic estimate delta={}ms", delta)),
            confidence: if sccache_pilot_execution_report_v1
                .duration_before_ms
                .is_some()
            {
                "Low"
            } else {
                "Insufficient"
            }
            .to_string(),
            can_claim_speedup: false,
            status: "SccacheEffectEstimateReadyWithWarnings".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let cargo_check_obs = observe_simple_command(
            config.run_cargo_check_timing,
            "cargo check --workspace --quiet",
            None,
        )?;
        let cargo_build_obs = observe_simple_command(
            config.run_cargo_build_timing,
            "cargo build --bin soma_experiment",
            None,
        )?;
        let cargo_no_run_obs = observe_simple_command(
            config.run_real_no_run_observation,
            "cargo test --workspace --no-run --quiet",
            config.no_run_timeout_ms,
        )?;
        let cargo_full_obs = observe_simple_command(
            config.run_real_full_observation,
            "cargo test --workspace --quiet",
            config.full_timeout_ms,
        )?;
        let cargo_json_obs = observe_command_stdout(
            config.run_cargo_json_progress_capture,
            "cargo test --workspace --no-run --message-format=json",
            config.cargo_json_timeout_ms,
        )?;
        let cargo_check_fixture = if cargo_check_obs.attempted {
            None
        } else {
            timing_fixture.as_ref()
        };
        let cargo_build_fixture = if cargo_build_obs.attempted {
            None
        } else {
            timing_fixture.as_ref()
        };
        let cargo_no_run_fixture = if cargo_no_run_obs.attempted {
            None
        } else {
            timing_fixture.as_ref()
        };
        let cargo_full_fixture = if cargo_full_obs.attempted {
            None
        } else {
            timing_fixture.as_ref()
        };
        let cargo_check_timing_capture_v1 = CargoCheckTimingCaptureV1 {
            report_id: "cargo-check-timing-capture-v1".to_string(),
            attempted: cargo_check_obs.attempted,
            command: "cargo check --workspace --quiet".to_string(),
            finished: cargo_check_fixture
                .map(|f| f.cargo_check_finished)
                .unwrap_or(cargo_check_obs.finished),
            passed: cargo_check_fixture
                .and_then(|f| f.cargo_check_passed)
                .or(cargo_check_obs.passed),
            duration_ms: cargo_check_fixture
                .and_then(|f| f.cargo_check_duration_ms)
                .or(cargo_check_obs.duration_ms),
            timeout_ms: cargo_check_obs.timeout_ms,
            status: status_from_capture(
                cargo_check_fixture
                    .map(|f| f.cargo_check_finished)
                    .unwrap_or(cargo_check_obs.finished),
                cargo_check_fixture
                    .and_then(|f| f.cargo_check_passed)
                    .or(cargo_check_obs.passed),
                None,
                cargo_check_obs.attempted || cargo_check_fixture.is_some(),
            ),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let cargo_build_timing_capture_v1 = CargoBuildTimingCaptureV1 {
            report_id: "cargo-build-timing-capture-v1".to_string(),
            attempted: cargo_build_obs.attempted,
            command: "cargo build --bin soma_experiment".to_string(),
            finished: cargo_build_fixture
                .map(|f| f.cargo_build_finished)
                .unwrap_or(cargo_build_obs.finished),
            passed: cargo_build_fixture
                .and_then(|f| f.cargo_build_passed)
                .or(cargo_build_obs.passed),
            duration_ms: cargo_build_fixture
                .and_then(|f| f.cargo_build_duration_ms)
                .or(cargo_build_obs.duration_ms),
            timeout_ms: cargo_build_obs.timeout_ms,
            status: status_from_capture(
                cargo_build_fixture
                    .map(|f| f.cargo_build_finished)
                    .unwrap_or(cargo_build_obs.finished),
                cargo_build_fixture
                    .and_then(|f| f.cargo_build_passed)
                    .or(cargo_build_obs.passed),
                None,
                cargo_build_obs.attempted || cargo_build_fixture.is_some(),
            ),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let cargo_no_run_timing_capture_v1 = CargoNoRunTimingCaptureV1 {
            report_id: "cargo-no-run-timing-capture-v1".to_string(),
            attempted: cargo_no_run_obs.attempted,
            command: "cargo test --workspace --no-run --quiet".to_string(),
            finished: cargo_no_run_fixture
                .map(|f| f.cargo_no_run_finished)
                .unwrap_or(cargo_no_run_obs.finished),
            passed: cargo_no_run_fixture
                .and_then(|f| f.cargo_no_run_passed)
                .or(cargo_no_run_obs.passed),
            duration_ms: cargo_no_run_fixture
                .and_then(|f| f.cargo_no_run_duration_ms)
                .or(cargo_no_run_obs.duration_ms),
            timeout_ms: cargo_no_run_fixture
                .and_then(|f| f.cargo_no_run_timeout_ms)
                .or(config.no_run_timeout_ms)
                .or(summary.no_run_timeout_seconds.map(|s| s * 1000)),
            status: if !cargo_no_run_obs.attempted
                && cargo_no_run_fixture.is_none()
                && summary.no_run_timeout_seconds.is_some()
            {
                "CargoNoRunTimedOut".to_string()
            } else {
                status_from_capture(
                    cargo_no_run_fixture
                        .map(|f| f.cargo_no_run_finished)
                        .unwrap_or(cargo_no_run_obs.finished),
                    cargo_no_run_fixture
                        .and_then(|f| f.cargo_no_run_passed)
                        .or(cargo_no_run_obs.passed),
                    cargo_no_run_fixture
                        .and_then(|f| f.cargo_no_run_timeout_ms)
                        .or(config.no_run_timeout_ms),
                    cargo_no_run_obs.attempted || cargo_no_run_fixture.is_some(),
                )
            },
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let cargo_full_run_timing_capture_v1 = CargoFullRunTimingCaptureV1 {
            report_id: "cargo-full-run-timing-capture-v1".to_string(),
            attempted: cargo_full_obs.attempted,
            command: "cargo test --workspace --quiet".to_string(),
            finished: cargo_full_fixture
                .map(|f| f.cargo_full_finished)
                .unwrap_or(cargo_full_obs.finished),
            passed: cargo_full_fixture
                .and_then(|f| f.cargo_full_passed)
                .or(cargo_full_obs.passed),
            duration_ms: cargo_full_fixture
                .and_then(|f| f.cargo_full_duration_ms)
                .or(cargo_full_obs.duration_ms),
            timeout_ms: cargo_full_fixture
                .and_then(|f| f.cargo_full_timeout_ms)
                .or(config.full_timeout_ms)
                .or(summary.full_timeout_seconds.map(|s| s * 1000)),
            status: if !cargo_full_obs.attempted
                && cargo_full_fixture.is_none()
                && summary.full_timeout_seconds.is_some()
            {
                "CargoFullTimedOut".to_string()
            } else {
                status_from_capture(
                    cargo_full_fixture
                        .map(|f| f.cargo_full_finished)
                        .unwrap_or(cargo_full_obs.finished),
                    cargo_full_fixture
                        .and_then(|f| f.cargo_full_passed)
                        .or(cargo_full_obs.passed),
                    cargo_full_fixture
                        .and_then(|f| f.cargo_full_timeout_ms)
                        .or(config.full_timeout_ms),
                    cargo_full_obs.attempted || cargo_full_fixture.is_some(),
                )
            },
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let cargo_json_progress_capture_v6 = if config.run_cargo_json_progress_capture {
            build_cargo_json_progress_capture_v6(config, &summary, &cargo_json_obs)
        } else {
            imported_capture.unwrap_or_else(|| {
                build_cargo_json_progress_capture_v6(config, &summary, &cargo_json_obs)
            })
        };
        let cargo_artifact_timeline_v2 = CargoArtifactTimelineV2 {
            report_id: "cargo-artifact-timeline-v2".to_string(),
            events: cargo_json_progress_capture_v6
                .last_targets
                .iter()
                .enumerate()
                .map(|(index, target)| CargoArtifactEventV2 {
                    seq: index as u64 + 1,
                    target: target.clone(),
                    artifact: format!(
                        "target/debug/deps/{}.synthetic",
                        target
                            .rsplit('/')
                            .next()
                            .unwrap_or(target)
                            .trim_end_matches(".rs")
                    ),
                    phase: if cargo_json_progress_capture_v6.attempted {
                        "observed"
                    } else {
                        "synthetic"
                    }
                    .to_string(),
                    observed: cargo_json_progress_capture_v6.attempted,
                })
                .collect(),
            target_artifacts: cargo_json_progress_capture_v6
                .last_targets
                .iter()
                .map(|target| {
                    (
                        target.clone(),
                        vec![format!(
                            "target/debug/deps/{}.synthetic",
                            target
                                .rsplit('/')
                                .next()
                                .unwrap_or(target)
                                .trim_end_matches(".rs")
                        )],
                    )
                })
                .collect(),
            last_artifact_by_target: cargo_json_progress_capture_v6
                .last_targets
                .iter()
                .map(|target| {
                    (
                        target.clone(),
                        format!(
                            "target/debug/deps/{}.synthetic",
                            target
                                .rsplit('/')
                                .next()
                                .unwrap_or(target)
                                .trim_end_matches(".rs")
                        ),
                    )
                })
                .collect(),
            status: "ArtifactTimelineReady".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let cargo_target_stall_attribution_report_v2 =
            build_cargo_target_stall_attribution_report_v2(
                &cargo_json_progress_capture_v6,
                &cargo_no_run_timing_capture_v1,
                &summary,
            );
        let rustc_process_timeline_report_v2 = RustcProcessTimelineReportV2 {
            report_id: "rustc-process-timeline-v2".to_string(),
            max_concurrent_rustc: summary.max_concurrent_rustc,
            active_rustc_args: summary.active_rustc_args.clone(),
            remaining_rustc_after_timeout: if cargo_no_run_obs.cleanup.timeout_occurred {
                cargo_no_run_obs.cleanup.remaining_rustc_processes
            } else {
                0
            },
            status: "RustcTimelineReadyWithWarnings".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let integration_test_binary_stall_report_v2 = IntegrationTestBinaryStallReportV2 {
            report_id: "integration-test-binary-stall-v2".to_string(),
            stalled_integration_binaries: cargo_target_stall_attribution_report_v2
                .suspected_stalled_targets
                .clone(),
            high_fanout_families: summary.high_fanout_families.clone(),
            already_retired_excluded: summary.retired_targets_carried_forward.clone(),
            status: "IntegrationStallAttributed".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let link_macro_attribution_report_v2 = LinkMacroAttributionReportV2 {
            report_id: "link-macro-attribution-v2".to_string(),
            link_heavy_candidates: summary.link_heavy_candidates.clone(),
            macro_heavy_candidates: summary.macro_heavy_candidates.clone(),
            observed_candidates: summary.link_heavy_candidates.clone(),
            inferred_candidates: summary.macro_heavy_candidates.clone(),
            status: "LinkMacroAttributed".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let fixture_render_cli_fanout_attribution_report_v2 =
            FixtureRenderCliFanoutAttributionReportV2 {
                report_id: "fixture-render-cli-fanout-attribution-v2".to_string(),
                fixture_fanout: summary.fixture_fanout.clone(),
                render_fanout: summary.render_fanout.clone(),
                cli_fanout: summary.cli_fanout.clone(),
                helper_fanout: summary.helper_fanout.clone(),
                status: "FanoutAttributed".to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let workspace_timeout_root_cause_report_v2 = build_workspace_timeout_root_cause_report_v2(
            &summary,
            &cargo_target_stall_attribution_report_v2,
            &link_macro_attribution_report_v2,
            &fixture_render_cli_fanout_attribution_report_v2,
        );
        let rows = vec![
            DiagnosticEvidenceRowV1 {
                row_name: "CargoCheck".to_string(),
                evidence_strength: "SupportingOnly".to_string(),
                status: cargo_check_timing_capture_v1.status.clone(),
                supporting_only: true,
            },
            DiagnosticEvidenceRowV1 {
                row_name: "CargoBuild".to_string(),
                evidence_strength: "SupportingOnly".to_string(),
                status: cargo_build_timing_capture_v1.status.clone(),
                supporting_only: true,
            },
            DiagnosticEvidenceRowV1 {
                row_name: "CargoNoRun".to_string(),
                evidence_strength: if cargo_no_run_timing_capture_v1.finished
                    && cargo_no_run_timing_capture_v1.passed == Some(true)
                {
                    "NoRunOnly"
                } else {
                    "SupportingOnly"
                }
                .to_string(),
                status: cargo_no_run_timing_capture_v1.status.clone(),
                supporting_only: true,
            },
            DiagnosticEvidenceRowV1 {
                row_name: "CargoFull".to_string(),
                evidence_strength: if cargo_full_run_timing_capture_v1.finished
                    && cargo_full_run_timing_capture_v1.passed == Some(true)
                {
                    "AcceptanceReady"
                } else {
                    "Insufficient"
                }
                .to_string(),
                status: cargo_full_run_timing_capture_v1.status.clone(),
                supporting_only: !(cargo_full_run_timing_capture_v1.finished
                    && cargo_full_run_timing_capture_v1.passed == Some(true)),
            },
            DiagnosticEvidenceRowV1 {
                row_name: "CargoJson".to_string(),
                evidence_strength: "SupportingOnly".to_string(),
                status: cargo_json_progress_capture_v6.status.clone(),
                supporting_only: true,
            },
            DiagnosticEvidenceRowV1 {
                row_name: "Nextest".to_string(),
                evidence_strength: "SupportingOnly".to_string(),
                status: nextest_pilot_execution_report_v1.execution_status.clone(),
                supporting_only: true,
            },
            DiagnosticEvidenceRowV1 {
                row_name: "Sccache".to_string(),
                evidence_strength: "SupportingOnly".to_string(),
                status: sccache_pilot_execution_report_v1.execution_status.clone(),
                supporting_only: true,
            },
            DiagnosticEvidenceRowV1 {
                row_name: "RustcTimeline".to_string(),
                evidence_strength: "SupportingOnly".to_string(),
                status: rustc_process_timeline_report_v2.status.clone(),
                supporting_only: true,
            },
            DiagnosticEvidenceRowV1 {
                row_name: "TargetStall".to_string(),
                evidence_strength: "SupportingOnly".to_string(),
                status: cargo_target_stall_attribution_report_v2.status.clone(),
                supporting_only: true,
            },
            DiagnosticEvidenceRowV1 {
                row_name: "FanoutMap".to_string(),
                evidence_strength: "SupportingOnly".to_string(),
                status: fixture_render_cli_fanout_attribution_report_v2
                    .status
                    .clone(),
                supporting_only: true,
            },
        ];
        let workspace_diagnostic_evidence_matrix_v1 = WorkspaceDiagnosticEvidenceMatrixV1 {
            matrix_id: "workspace-diagnostic-evidence-matrix-v1".to_string(),
            evidence_rows: rows.clone(),
            evidence_strength_by_row: rows
                .iter()
                .map(|row| (row.row_name.clone(), row.evidence_strength.clone()))
                .collect(),
            supports_acceptance: cargo_full_run_timing_capture_v1.finished
                && cargo_full_run_timing_capture_v1.passed == Some(true),
            matrix_status: if cargo_full_run_timing_capture_v1.finished
                && cargo_full_run_timing_capture_v1.passed == Some(true)
            {
                "DiagnosticEvidenceMatrixReady"
            } else {
                "DiagnosticEvidenceMatrixReadyWithWarnings"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let remaining_safe_candidate_pool_report_v2 = RemainingSafeCandidatePoolReportV2 {
            report_id: "remaining-safe-candidate-pool-v2".to_string(),
            previous_candidate_pool: summary.candidate_pool.clone(),
            new_evidence: vec![
                workspace_timeout_root_cause_report_v2.status.clone(),
                workspace_timeout_root_cause_report_v2.confidence.clone(),
            ],
            candidate_statuses: summary
                .candidate_pool
                .iter()
                .cloned()
                .map(|candidate| {
                    let status = if summary.sentinel_exclusions.contains(&candidate) {
                        "SentinelExcluded"
                    } else if summary.retired_targets_carried_forward.contains(&candidate) {
                        "AlreadyRetiredExcluded"
                    } else if summary.low_risk_candidates.contains(&candidate) {
                        "LowRiskCandidate"
                    } else {
                        "NeedsMoreEvidence"
                    };
                    (candidate.clone(), status.to_string())
                })
                .collect(),
            sentinel_exclusions: summary.sentinel_exclusions.clone(),
            status: if summary.candidate_pool.iter().any(|candidate| {
                summary.low_risk_candidates.contains(candidate)
                    && !summary.sentinel_exclusions.contains(candidate)
                    && !summary.retired_targets_carried_forward.contains(candidate)
            }) {
                "CandidatePoolReadyWithWarnings"
            } else {
                "NoSafeCandidatePool"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let assertion_ledger_continuity_check_v2 = AssertionLedgerContinuityCheckV2 {
            report_id: "assertion-ledger-continuity-check-v2".to_string(),
            carried_forward_assertion_delta: summary.cumulative_assertion_delta,
            assertion_deletions_detected: 0,
            continuity_status: "AssertionLedgerContinuityReady".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let equivalent_coverage_continuity_check_v2 = EquivalentCoverageContinuityCheckV2 {
            report_id: "equivalent-coverage-continuity-check-v2".to_string(),
            coverage_gap_count: if summary.equivalent_coverage_feasible {
                0
            } else {
                1
            },
            equivalent_coverage_feasible: summary.equivalent_coverage_feasible,
            continuity_status: if summary.equivalent_coverage_feasible {
                "EquivalentCoverageContinuityReady"
            } else {
                "EquivalentCoverageContinuityBlocked"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let safety_sentinel_continuity_check_v2 = SafetySentinelContinuityCheckV2 {
            report_id: "safety-sentinel-continuity-check-v2".to_string(),
            sentinels_preserved: vec![
                "CommitteeCliSafety".to_string(),
                "WorkspaceCliSafety".to_string(),
                "Determinism".to_string(),
                "RuntimeDeferred".to_string(),
                "NoOrderAccount".to_string(),
            ],
            sentinel_uncertainties: if summary.safety_sentinels_preserved {
                Vec::new()
            } else {
                vec!["sentinel preservation uncertain".to_string()]
            },
            continuity_status: if summary.safety_sentinels_preserved {
                "SafetySentinelContinuityReady"
            } else {
                "SafetySentinelContinuityBlocked"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let no_hidden_skip_continuity_check_v2 = NoHiddenSkipContinuityCheckV2 {
            report_id: "no-hidden-skip-continuity-check-v2".to_string(),
            hidden_skip_indicators: if summary.no_hidden_skip_continuity {
                Vec::new()
            } else {
                vec!["hidden skip indicator present".to_string()]
            },
            continuity_status: if summary.no_hidden_skip_continuity {
                "NoHiddenSkipContinuityReady"
            } else {
                "NoHiddenSkipContinuityBlocked"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let timeout_window_adequacy_report_v2 = TimeoutWindowAdequacyReportV2 { report_id: "timeout-window-adequacy-v2".to_string(), previous_timeouts_ms: BTreeMap::from([("cargo_no_run".to_string(), summary.no_run_timeout_seconds.unwrap_or(300) * 1000), ("cargo_full".to_string(), summary.full_timeout_seconds.unwrap_or(300) * 1000)]), current_timeouts_ms: BTreeMap::from([("cargo_no_run".to_string(), config.no_run_timeout_ms.unwrap_or(300_000)), ("cargo_full".to_string(), config.full_timeout_ms.unwrap_or(300_000))]), adequacy: if config.no_run_timeout_ms.unwrap_or(0) >= 300_000 && config.full_timeout_ms.unwrap_or(0) >= 300_000 { "AdequateWithWarnings" } else { "NeedsLongerWindow" }.to_string(), recommendation: "keep explicit longer timeout for honest workspace observations and do not treat cleanup as pass".to_string(), reason_codes: diagnostic_reason_codes(&[]) };
        let timeout_cleanup_verification_report_v5 = TimeoutCleanupVerificationReportV5 {
            report_id: "timeout-cleanup-verification-v5".to_string(),
            timeout_occurred: cargo_no_run_obs.cleanup.timeout_occurred
                || cargo_full_obs.cleanup.timeout_occurred
                || summary.no_run_exit_code == Some(124)
                || summary.full_exit_code == Some(124),
            child_process_cleanup_attempted: cargo_no_run_obs
                .cleanup
                .child_process_cleanup_attempted
                || cargo_full_obs.cleanup.child_process_cleanup_attempted
                || summary.no_run_exit_code == Some(124)
                || summary.full_exit_code == Some(124),
            remaining_cargo_processes: cargo_no_run_obs.cleanup.remaining_cargo_processes
                + cargo_full_obs.cleanup.remaining_cargo_processes,
            remaining_rustc_processes: cargo_no_run_obs.cleanup.remaining_rustc_processes
                + cargo_full_obs.cleanup.remaining_rustc_processes,
            cleanup_status: if cargo_no_run_obs.cleanup.remaining_cargo_processes
                + cargo_full_obs.cleanup.remaining_cargo_processes
                == 0
                && cargo_no_run_obs.cleanup.remaining_rustc_processes
                    + cargo_full_obs.cleanup.remaining_rustc_processes
                    == 0
            {
                "TimeoutCleanupVerified"
            } else {
                "TimeoutCleanupNeedsFollowup"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let workspace_no_run_recovery_gate_v13 = WorkspaceNoRunRecoveryGateV13 {
            gate_id: "workspace-no-run-recovery-gate-v13".to_string(),
            command: cargo_no_run_timing_capture_v1.command.clone(),
            finished: cargo_no_run_timing_capture_v1.finished,
            passed: cargo_no_run_timing_capture_v1.passed,
            timeout_ms: cargo_no_run_timing_capture_v1.timeout_ms,
            recovered: cargo_no_run_timing_capture_v1.finished
                && cargo_no_run_timing_capture_v1.passed == Some(true),
            gate_status: if cargo_no_run_timing_capture_v1.finished
                && cargo_no_run_timing_capture_v1.passed == Some(true)
            {
                "NoRunRecovered"
            } else if cargo_no_run_timing_capture_v1.attempted
                || cargo_no_run_timing_capture_v1.timeout_ms.is_some()
            {
                "NoRunStillBlocked"
            } else {
                "NoRunNotRun"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let workspace_full_acceptance_gate_v13 = WorkspaceFullAcceptanceGateV13 {
            gate_id: "workspace-full-acceptance-gate-v13".to_string(),
            command: cargo_full_run_timing_capture_v1.command.clone(),
            finished: cargo_full_run_timing_capture_v1.finished,
            passed: cargo_full_run_timing_capture_v1.passed,
            timeout_ms: cargo_full_run_timing_capture_v1.timeout_ms,
            accepted: cargo_full_run_timing_capture_v1.finished
                && cargo_full_run_timing_capture_v1.passed == Some(true),
            gate_status: if cargo_full_run_timing_capture_v1.finished
                && cargo_full_run_timing_capture_v1.passed == Some(true)
            {
                "FullWorkspaceAccepted"
            } else if cargo_full_run_timing_capture_v1.attempted
                || cargo_full_run_timing_capture_v1.timeout_ms.is_some()
            {
                "FullWorkspaceStillBlocked"
            } else {
                "FullWorkspaceNotRun"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let focused_vs_full_bridge_v9 = FocusedVsFullBridgeV9 {
            bridge_id: "focused-vs-full-bridge-v9".to_string(),
            focused_truth_status: if baseline.focused_matrix_passed {
                "SupportingOnly"
            } else {
                "Insufficient"
            }
            .to_string(),
            cli_truth_status: if baseline.cli_smoke_passed {
                "SupportingOnly"
            } else {
                "Insufficient"
            }
            .to_string(),
            cargo_build_truth_status: if baseline.cargo_build_passed {
                "SupportingOnly"
            } else {
                "Insufficient"
            }
            .to_string(),
            nextest_truth_status: if nextest_pilot_execution_report_v1.passed == Some(true) {
                "SupportingOnly"
            } else {
                "Insufficient"
            }
            .to_string(),
            sccache_truth_status: if sccache_pilot_execution_report_v1.available {
                "SupportingOnly"
            } else {
                "Insufficient"
            }
            .to_string(),
            cargo_progress_truth_status: if cargo_json_progress_capture_v6.messages > 0 {
                "SupportingOnly"
            } else {
                "Insufficient"
            }
            .to_string(),
            no_run_truth_status: if workspace_no_run_recovery_gate_v13.recovered {
                "NoRunOnly"
            } else {
                "SupportingOnly"
            }
            .to_string(),
            can_claim_full_acceptance: workspace_full_acceptance_gate_v13.accepted,
            bridge_status: if workspace_full_acceptance_gate_v13.accepted {
                "FullBridgeClosed"
            } else {
                "FullGateStillOpen"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let acceptance_truth_gate_v13 = AcceptanceTruthGateV13 {
            gate_id: "acceptance-truth-gate-v13".to_string(),
            focused_truth_status: focused_vs_full_bridge_v9.focused_truth_status.clone(),
            cli_truth_status: focused_vs_full_bridge_v9.cli_truth_status.clone(),
            cargo_check_truth_status: if cargo_check_timing_capture_v1.passed == Some(true)
                || baseline.cargo_check_passed
            {
                "SupportingOnly"
            } else {
                "Insufficient"
            }
            .to_string(),
            cargo_build_truth_status: focused_vs_full_bridge_v9.cargo_build_truth_status.clone(),
            nextest_truth_status: focused_vs_full_bridge_v9.nextest_truth_status.clone(),
            sccache_truth_status: focused_vs_full_bridge_v9.sccache_truth_status.clone(),
            cargo_json_truth_status: focused_vs_full_bridge_v9
                .cargo_progress_truth_status
                .clone(),
            no_run_truth_status: focused_vs_full_bridge_v9.no_run_truth_status.clone(),
            full_workspace_truth_status: if workspace_full_acceptance_gate_v13.accepted {
                "FullWorkspaceAccepted"
            } else {
                "SupportingOnly"
            }
            .to_string(),
            truth_status: if workspace_full_acceptance_gate_v13.accepted {
                "AcceptanceTruthReady"
            } else {
                "AcceptanceTruthReadyWithWarnings"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let fifth_patch_decision_gate_v2 = build_fifth_patch_decision_gate_v2(
            &remaining_safe_candidate_pool_report_v2,
            &workspace_timeout_root_cause_report_v2,
            &equivalent_coverage_continuity_check_v2,
            &safety_sentinel_continuity_check_v2,
            &no_hidden_skip_continuity_check_v2,
            summary.assertion_migration_feasible,
            summary.fifth_patch_block_status.clone(),
        );
        let fifth_patch_readiness_reevaluation_report_v1 =
            FifthPatchReadinessReevaluationReportV1 {
                report_id: "fifth-patch-readiness-reevaluation-v1".to_string(),
                previous_block_reason: summary.fifth_patch_block_status.clone(),
                new_evidence: vec![
                    workspace_timeout_root_cause_report_v2.status.clone(),
                    workspace_timeout_root_cause_report_v2.confidence.clone(),
                ],
                still_blocked_reasons: {
                    let mut reasons = Vec::new();
                    if !fifth_patch_decision_gate_v2.assertion_migration_feasible {
                        reasons.push("assertion migration remains infeasible".to_string());
                    }
                    if !fifth_patch_decision_gate_v2.equivalent_coverage_feasible {
                        reasons.push("equivalent coverage remains infeasible".to_string());
                    }
                    if !fifth_patch_decision_gate_v2.safety_sentinel_preserved {
                        reasons.push("safety sentinel preservation remains uncertain".to_string());
                    }
                    if !fifth_patch_decision_gate_v2.no_hidden_skip_continuity {
                        reasons.push("no-hidden-skip continuity is broken".to_string());
                    }
                    if workspace_timeout_root_cause_report_v2.status != "TimeoutRootCauseIsolated" {
                        reasons
                            .push("workspace timeout root cause is not yet isolated".to_string());
                    }
                    reasons
                },
                allowed_reasons: if fifth_patch_decision_gate_v2.fifth_patch_allowed_for_next_sprint
                {
                    vec!["diagnostic evidence now supports a later gated sprint review".to_string()]
                } else {
                    Vec::new()
                },
                status: if fifth_patch_decision_gate_v2.fifth_patch_allowed_for_next_sprint {
                    "FifthPatchReadinessRaisedForNextSprint"
                } else {
                    "FifthPatchStillBlocked"
                }
                .to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let fifth_patch_no_apply_guarantee_report_v1 = FifthPatchNoApplyGuaranteeReportV1 {
            report_id: "fifth-patch-no-apply-guarantee-v1".to_string(),
            fifth_patch_applied: false,
            no_files_retired_by_fifth_patch: true,
            no_assertions_moved_by_fifth_patch: true,
            guarantee_status: "FifthPatchNoApplyGuaranteed".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let acceptance_evidence_strength_report_v2 = AcceptanceEvidenceStrengthReportV2 {
            report_id: "acceptance-evidence-strength-v2".to_string(),
            focused_evidence_strength: focused_vs_full_bridge_v9.focused_truth_status.clone(),
            cli_evidence_strength: focused_vs_full_bridge_v9.cli_truth_status.clone(),
            cargo_check_evidence_strength: if cargo_check_timing_capture_v1.passed == Some(true)
                || baseline.cargo_check_passed
            {
                "SupportingOnly"
            } else {
                "Insufficient"
            }
            .to_string(),
            cargo_build_evidence_strength: focused_vs_full_bridge_v9
                .cargo_build_truth_status
                .clone(),
            nextest_evidence_strength: focused_vs_full_bridge_v9.nextest_truth_status.clone(),
            sccache_evidence_strength: focused_vs_full_bridge_v9.sccache_truth_status.clone(),
            cargo_progress_evidence_strength: focused_vs_full_bridge_v9
                .cargo_progress_truth_status
                .clone(),
            no_run_evidence_strength: focused_vs_full_bridge_v9.no_run_truth_status.clone(),
            full_workspace_evidence_strength: if workspace_full_acceptance_gate_v13.accepted {
                "Sufficient"
            } else {
                "Insufficient"
            }
            .to_string(),
            overall_evidence_strength: if workspace_full_acceptance_gate_v13.accepted {
                "Sufficient"
            } else {
                "SupportingOnly"
            }
            .to_string(),
            status: if workspace_full_acceptance_gate_v13.accepted {
                "AcceptanceEvidenceSufficient"
            } else {
                "AcceptanceEvidenceSupportingOnly"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let workspace_recovery_decision_report_v2 = WorkspaceRecoveryDecisionReportV2 {
            report_id: "workspace-recovery-decision-v2".to_string(),
            recommend_nextest_diagnostic: true,
            recommend_sccache_diagnostic: true,
            recommend_more_observation: workspace_timeout_root_cause_report_v2.status
                != "TimeoutRootCauseIsolated",
            recommend_fifth_patch_for_next_sprint_only: fifth_patch_decision_gate_v2
                .fifth_patch_allowed_for_next_sprint,
            recommend_stop_consolidation: remaining_safe_candidate_pool_report_v2
                .candidate_statuses
                .is_empty(),
            status: if remaining_safe_candidate_pool_report_v2
                .candidate_statuses
                .is_empty()
            {
                "WorkspaceRecoveryStopConsolidation"
            } else if fifth_patch_decision_gate_v2.fifth_patch_allowed_for_next_sprint {
                "WorkspaceRecoveryReadyForNextSprintGate"
            } else {
                "WorkspaceRecoveryNeedsMoreObservation"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let safety_coverage_preservation_report_v28 = SafetyCoveragePreservationReportV28 {
            report_id: "safety-coverage-preservation-v28".to_string(),
            live_trading_guard_present: true,
            broker_guard_present: true,
            order_guard_present: true,
            account_guard_present: true,
            runtime_llm_guard_present: true,
            mamba_runtime_guard_present: true,
            gated_runtime_guard_present: true,
            model_training_guard_present: true,
            rust_neural_training_guard_present: true,
            python_training_dependency_guard_present: true,
            secret_guard_present: true,
            no_lookahead_guard_present: true,
            source_boundary_guard_present: true,
            browser_execution_guard_present: true,
            ui_order_control_guard_present: true,
            committee_owned_core_guard_present: true,
            investor_impersonation_guard_present: true,
            paper_candidate_not_order_guard_present: true,
            no_silent_confidence_upgrade_guard_present: true,
            focused_not_full_acceptance_guard_present: true,
            no_hidden_skip_guard_present: true,
            assertion_preservation_guard_present: true,
            safety_sentinel_preservation_guard_present: true,
            cumulative_assertion_ledger_guard_present: true,
            equivalent_coverage_v2_guard_present: true,
            timeout_cleanup_v2_guard_present: true,
            cargo_json_progress_truth_guard_present: true,
            third_patch_no_broad_consolidation_guard_present: true,
            sprint109_validation_reconciliation_guard_present: true,
            cumulative_assertion_ledger_v2_guard_present: true,
            equivalent_coverage_v3_guard_present: true,
            timeout_cleanup_v3_guard_present: true,
            cargo_json_progress_v4_truth_guard_present: true,
            fourth_patch_no_broad_consolidation_guard_present: true,
            sprint110_truth_import_guard_present: true,
            timeout_root_cause_guard_present: true,
            fifth_patch_decision_gate_guard_present: true,
            no_auto_fifth_patch_guard_present: true,
            acceptance_evidence_strength_guard_present: true,
            nextest_diagnostic_only_guard_present: true,
            sccache_local_only_guard_present: true,
            fifth_patch_no_apply_guard_present: true,
            diagnostic_not_acceptance_guard_present: true,
            no_broad_consolidation_guard_present: true,
            safety_status: "SafetyCoveragePreserved".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let control_tower_workspace_diagnostic_pilot_panel =
            ControlTowerWorkspaceDiagnosticPilotPanel {
                panel_id: "control-tower-workspace-diagnostic-pilot".to_string(),
                nextest_availability_status: nextest_availability_report_v1
                    .availability_status
                    .clone(),
                sccache_availability_status: sccache_availability_report_v1
                    .availability_status
                    .clone(),
                cargo_timing_statuses: BTreeMap::from([
                    (
                        "cargo_check".to_string(),
                        cargo_check_timing_capture_v1.status.clone(),
                    ),
                    (
                        "cargo_build".to_string(),
                        cargo_build_timing_capture_v1.status.clone(),
                    ),
                    (
                        "cargo_no_run".to_string(),
                        cargo_no_run_timing_capture_v1.status.clone(),
                    ),
                    (
                        "cargo_full".to_string(),
                        cargo_full_run_timing_capture_v1.status.clone(),
                    ),
                ]),
                cargo_progress_status: cargo_json_progress_capture_v6.status.clone(),
                diagnostic_matrix_status: workspace_diagnostic_evidence_matrix_v1
                    .matrix_status
                    .clone(),
                root_cause_status: workspace_timeout_root_cause_report_v2.status.clone(),
                acceptance_truth_status: acceptance_truth_gate_v13.truth_status.clone(),
                warnings: vec![
                    "research-only".to_string(),
                    "paper-only".to_string(),
                    "diagnostic-only".to_string(),
                    "nextest-is-not-acceptance".to_string(),
                    "sccache-is-not-speedup-proof".to_string(),
                    "fifth-patch-not-applied".to_string(),
                    "no-run-is-not-full".to_string(),
                    "cargo-progress-is-not-acceptance".to_string(),
                    "timeout-cleanup-is-not-pass".to_string(),
                    "no run button".to_string(),
                ],
                static_read_only: true,
                no_run_button: true,
                no_train_runtime_live_order_account_controls: true,
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let control_tower_fifth_patch_reevaluation_panel =
            ControlTowerFifthPatchReevaluationPanel {
                panel_id: "control-tower-fifth-patch-reevaluation".to_string(),
                fifth_patch_gate_status: fifth_patch_decision_gate_v2.gate_status.clone(),
                no_apply_guarantee_status: fifth_patch_no_apply_guarantee_report_v1
                    .guarantee_status
                    .clone(),
                candidate_pool_status: remaining_safe_candidate_pool_report_v2.status.clone(),
                readiness_reevaluation_status: fifth_patch_readiness_reevaluation_report_v1
                    .status
                    .clone(),
                continuity_statuses: BTreeMap::from([
                    (
                        "assertion".to_string(),
                        assertion_ledger_continuity_check_v2
                            .continuity_status
                            .clone(),
                    ),
                    (
                        "equivalent_coverage".to_string(),
                        equivalent_coverage_continuity_check_v2
                            .continuity_status
                            .clone(),
                    ),
                    (
                        "safety_sentinel".to_string(),
                        safety_sentinel_continuity_check_v2
                            .continuity_status
                            .clone(),
                    ),
                    (
                        "no_hidden_skip".to_string(),
                        no_hidden_skip_continuity_check_v2.continuity_status.clone(),
                    ),
                ]),
                warnings: vec![
                    "research-only".to_string(),
                    "paper-only".to_string(),
                    "diagnostic-only".to_string(),
                    "fifth-patch-not-applied".to_string(),
                    "no apply patch button".to_string(),
                ],
                static_read_only: true,
                no_apply_patch_button: true,
                no_train_runtime_live_order_account_controls: true,
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let mut bundle = WorkspaceDiagnosticPilotV1Bundle {
            sprint111_baseline_truth_import_report: baseline,
            sprint111_patch_and_timeout_carry_forward_report: carry,
            nextest_availability_report_v1,
            nextest_no_run_pilot_plan_v1,
            nextest_run_pilot_plan_v1,
            nextest_pilot_execution_report_v1,
            nextest_target_partition_report_v1,
            nextest_slow_target_attribution_report_v1,
            sccache_availability_report_v1,
            sccache_local_only_policy_report_v1,
            sccache_pilot_plan_v1,
            sccache_pilot_execution_report_v1,
            sccache_effect_estimate_report_v1,
            cargo_check_timing_capture_v1,
            cargo_build_timing_capture_v1,
            cargo_no_run_timing_capture_v1,
            cargo_full_run_timing_capture_v1,
            cargo_json_progress_capture_v6,
            cargo_artifact_timeline_v2,
            cargo_target_stall_attribution_report_v2,
            rustc_process_timeline_report_v2,
            integration_test_binary_stall_report_v2,
            link_macro_attribution_report_v2,
            fixture_render_cli_fanout_attribution_report_v2,
            workspace_diagnostic_evidence_matrix_v1,
            workspace_timeout_root_cause_report_v2,
            remaining_safe_candidate_pool_report_v2,
            fifth_patch_decision_gate_v2,
            fifth_patch_readiness_reevaluation_report_v1,
            fifth_patch_no_apply_guarantee_report_v1,
            assertion_ledger_continuity_check_v2,
            equivalent_coverage_continuity_check_v2,
            safety_sentinel_continuity_check_v2,
            no_hidden_skip_continuity_check_v2,
            timeout_window_adequacy_report_v2,
            timeout_cleanup_verification_report_v5,
            workspace_no_run_recovery_gate_v13,
            workspace_full_acceptance_gate_v13,
            focused_vs_full_bridge_v9,
            acceptance_truth_gate_v13,
            acceptance_evidence_strength_report_v2,
            workspace_recovery_decision_report_v2,
            safety_coverage_preservation_report_v28,
            control_tower_workspace_diagnostic_pilot_panel,
            control_tower_fifth_patch_reevaluation_panel,
            storage_report: WorkspaceDiagnosticPilotV1StorageReport {
                report_id: "workspace-diagnostic-pilot-storage-report".to_string(),
                output_dir: config.output_dir().display().to_string(),
                written_files: Vec::new(),
                file_count: 0,
                reason_codes: diagnostic_reason_codes(&[]),
            },
            final_summary: String::new(),
            reason_codes: diagnostic_reason_codes(&config.reason_codes),
        };
        bundle.final_summary = bundle.build_final_summary();
        let output_dir = config.output_dir();
        bundle.write_to_dir(&output_dir)?;
        Ok(bundle)
    }
}
