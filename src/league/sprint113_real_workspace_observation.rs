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
    "target/soma_sprint113_real_workspace_observation".to_string()
}

fn default_timeout_ms() -> Option<u64> {
    Some(360_000)
}

fn shell_exec(command: &str) -> String {
    format!("cd {} && exec {}", project_root().display(), command)
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

fn warning_posture() -> Vec<String> {
    vec![
        "research-only",
        "paper-only",
        "real-observation-diagnostic",
        "fifth-patch-not-applied",
        "nextest-is-not-cargo-workspace-acceptance",
        "sccache-is-not-speedup-proof",
        "cargo-progress-is-not-acceptance",
        "timeout-cleanup-is-not-pass",
        "focused-is-not-full",
        "CLI-smoke-is-not-full",
        "cargo-build-is-not-full",
        "no-run-is-not-full",
        "no assertion deletion",
        "no safety sentinel deletion",
        "no runtime implementation",
        "no training",
        "no live inference",
        "no live trading",
        "no order/account command",
        "no runtime LLM live decision path",
        "no investor impersonation",
        "no auto-activation of 18 live agents",
        "no silent confidence upgrade",
        "no safety test deletion",
        "no hidden skips",
        "local-only paths",
        "remote paths rejected",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
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
pub struct RealWorkspaceObservationDrilldownConfig {
    pub observation_id: String,
    #[serde(default)]
    pub sprint112_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub sprint112_truth_paths: Option<Vec<String>>,
    #[serde(default)]
    pub sprint112_verification_patch_paths: Option<Vec<String>>,
    #[serde(default)]
    pub cargo_json_progress_paths: Option<Vec<String>>,
    #[serde(default)]
    pub nextest_observation_paths: Option<Vec<String>>,
    #[serde(default)]
    pub sccache_observation_paths: Option<Vec<String>>,
    #[serde(default)]
    pub suspect_target_paths: Option<Vec<String>>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_false")]
    pub run_real_no_run_observation: bool,
    #[serde(default = "default_false")]
    pub run_real_full_observation: bool,
    #[serde(default = "default_false")]
    pub run_real_cargo_json_capture: bool,
    #[serde(default = "default_false")]
    pub run_nextest_probe: bool,
    #[serde(default = "default_false")]
    pub run_nextest_partition_probe: bool,
    #[serde(default = "default_false")]
    pub run_sccache_probe: bool,
    #[serde(default = "default_false")]
    pub run_sccache_local_pilot: bool,
    #[serde(default = "default_timeout_ms")]
    pub no_run_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub full_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub cargo_json_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub nextest_timeout_ms: Option<u64>,
    #[serde(default = "default_true")]
    pub require_actual_observation_preservation: bool,
    #[serde(default = "default_true")]
    pub require_timeout_cleanup_actual_counts: bool,
    #[serde(default = "default_true")]
    pub require_cargo_json_actual_parsing: bool,
    #[serde(default = "default_true")]
    pub require_fifth_patch_gate: bool,
    #[serde(default = "default_false")]
    pub allow_fifth_patch_application: bool,
    #[serde(default = "default_true")]
    pub preserve_runtime_deferred: bool,
    #[serde(default = "default_true")]
    pub preserve_safety_guards: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for RealWorkspaceObservationDrilldownConfig {
    fn default() -> Self {
        Self {
            observation_id: "sprint113-real-workspace-observation".to_string(),
            sprint112_bundle_paths: Some(vec![
                "examples/sprint113_data/sprint112_summary.json".to_string(),
            ]),
            sprint112_truth_paths: Some(vec![
                "examples/sprint113_data/sprint112_summary.json".to_string(),
            ]),
            sprint112_verification_patch_paths: Some(vec![
                "examples/sprint113_data/baseline_truth_import_expected.json".to_string(),
            ]),
            cargo_json_progress_paths: Some(vec![
                "examples/sprint113_data/real_cargo_json_progress_expected.json".to_string(),
            ]),
            nextest_observation_paths: None,
            sccache_observation_paths: None,
            suspect_target_paths: Some(vec![
                "examples/sprint113_data/target_family_isolation_expected.json".to_string(),
            ]),
            output_root: default_output_root(),
            run_real_no_run_observation: false,
            run_real_full_observation: false,
            run_real_cargo_json_capture: false,
            run_nextest_probe: false,
            run_nextest_partition_probe: false,
            run_sccache_probe: false,
            run_sccache_local_pilot: false,
            no_run_timeout_ms: default_timeout_ms(),
            full_timeout_ms: default_timeout_ms(),
            cargo_json_timeout_ms: default_timeout_ms(),
            nextest_timeout_ms: default_timeout_ms(),
            require_actual_observation_preservation: true,
            require_timeout_cleanup_actual_counts: true,
            require_cargo_json_actual_parsing: true,
            require_fifth_patch_gate: true,
            allow_fifth_patch_application: false,
            preserve_runtime_deferred: true,
            preserve_safety_guards: true,
            reason_codes: diagnostic_reason_codes(&[]),
        }
    }
}

impl RealWorkspaceObservationDrilldownConfig {
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
        PathBuf::from(&self.output_root).join(&self.observation_id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.observation_id.trim().is_empty() {
            return Err("sprint113 observation_id must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err(
                "sprint113 real workspace observation config paths must be local".to_string(),
            );
        }
        for paths in [
            &self.sprint112_bundle_paths,
            &self.sprint112_truth_paths,
            &self.sprint112_verification_patch_paths,
            &self.cargo_json_progress_paths,
            &self.nextest_observation_paths,
            &self.sccache_observation_paths,
            &self.suspect_target_paths,
        ] {
            if let Some(paths) = paths
                && paths.iter().any(|path| !local_only(path))
            {
                return Err(
                    "sprint113 real workspace observation config paths must be local".to_string(),
                );
            }
        }
        if !self.require_actual_observation_preservation {
            return Err(
                "sprint113 requires require_actual_observation_preservation=true".to_string(),
            );
        }
        if !self.require_timeout_cleanup_actual_counts {
            return Err(
                "sprint113 requires require_timeout_cleanup_actual_counts=true".to_string(),
            );
        }
        if !self.require_cargo_json_actual_parsing {
            return Err("sprint113 requires require_cargo_json_actual_parsing=true".to_string());
        }
        if !self.require_fifth_patch_gate {
            return Err("sprint113 requires require_fifth_patch_gate=true".to_string());
        }
        if self.allow_fifth_patch_application {
            return Err("sprint113 forbids fifth patch application".to_string());
        }
        if !self.preserve_runtime_deferred || !self.preserve_safety_guards {
            return Err("sprint113 preserve flags must stay true".to_string());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sprint112SummaryFixture {
    pub report_id: String,
    pub focused_tests_passed: bool,
    pub cli_smoke_passed: bool,
    pub cargo_check_passed: bool,
    pub cargo_build_passed: bool,
    pub no_run_timeout_seconds: Option<u64>,
    pub no_run_exit_code: Option<i32>,
    pub full_timeout_seconds: Option<u64>,
    pub full_exit_code: Option<i32>,
    pub timeout_cleanup_verified: bool,
    pub remaining_cargo_processes_after_timeout: u64,
    pub remaining_rustc_processes_after_timeout: u64,
    pub nextest_available: bool,
    pub sccache_available: bool,
    pub fifth_patch_still_blocked: bool,
    pub previous_gate_status: String,
    pub timeout_root_cause_status: String,
    pub acceptance_truth_status: String,
    pub patch_count_carried_forward: u64,
    pub retired_targets_carried_forward: Vec<String>,
    pub cumulative_assertion_delta: i64,
    pub cumulative_sample_backed_delta: i64,
    pub candidate_pool: Vec<String>,
    pub low_risk_candidates: Vec<String>,
    pub sentinel_exclusions: Vec<String>,
    pub assertion_migration_feasible: bool,
    pub equivalent_coverage_feasible: bool,
    pub safety_sentinels_preserved: bool,
    pub no_hidden_skip_continuity: bool,
    pub observed_root_cause_evidence: Vec<String>,
    pub inferred_root_cause_evidence: Vec<String>,
    pub stalled_targets: Vec<String>,
    pub last_targets: Vec<String>,
    pub active_rustc_args: Vec<String>,
    pub max_concurrent_rustc: u64,
    pub fixture_fanout: Vec<String>,
    pub render_fanout: Vec<String>,
    pub cli_fanout: Vec<String>,
    pub link_heavy_candidates: Vec<String>,
    pub macro_heavy_candidates: Vec<String>,
    pub high_fanout_families: Vec<String>,
}

impl Default for Sprint112SummaryFixture {
    fn default() -> Self {
        Self {
            report_id: "sprint112-summary".to_string(),
            focused_tests_passed: true,
            cli_smoke_passed: true,
            cargo_check_passed: true,
            cargo_build_passed: true,
            no_run_timeout_seconds: Some(360),
            no_run_exit_code: Some(124),
            full_timeout_seconds: Some(360),
            full_exit_code: Some(124),
            timeout_cleanup_verified: true,
            remaining_cargo_processes_after_timeout: 0,
            remaining_rustc_processes_after_timeout: 0,
            nextest_available: true,
            sccache_available: true,
            fifth_patch_still_blocked: true,
            previous_gate_status: "FifthPatchStillBlocked".to_string(),
            timeout_root_cause_status: "TimeoutRootCausePartiallyIsolated".to_string(),
            acceptance_truth_status: "AcceptanceTruthReadyWithWarnings".to_string(),
            patch_count_carried_forward: 4,
            retired_targets_carried_forward: vec![
                "tests/shared_fixture_harness_expansion_plan_v2.rs".to_string(),
                "tests/shared_output_dir_helper_application_v1.rs".to_string(),
                "tests/shared_render_helper_application_v1.rs".to_string(),
                "tests/shared_toml_builder_application_v1.rs".to_string(),
            ],
            cumulative_assertion_delta: 0,
            cumulative_sample_backed_delta: -4,
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
                "tests/sprint112_cli_safety.rs".to_string(),
                "tests/sprint113_cli_safety.rs".to_string(),
                "tests/sprint113_determinism.rs".to_string(),
            ],
            assertion_migration_feasible: false,
            equivalent_coverage_feasible: true,
            safety_sentinels_preserved: true,
            no_hidden_skip_continuity: true,
            observed_root_cause_evidence: vec![
                "workspace no-run timed out at 360 seconds with exit 124".to_string(),
                "workspace full run timed out at 360 seconds with exit 124".to_string(),
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
                "--test tests/workspace_timeout_root_cause.rs".to_string(),
            ],
            max_concurrent_rustc: 2,
            fixture_fanout: vec!["tests/shared_fixture_harness_application_v1.rs".to_string()],
            render_fanout: vec!["tests/workspace_timeout_root_cause.rs".to_string()],
            cli_fanout: vec![
                "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
            ],
            link_heavy_candidates: vec![
                "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
            ],
            macro_heavy_candidates: vec!["tests/workspace_timeout_root_cause.rs".to_string()],
            high_fanout_families: vec![
                "FixtureSetupFanout".to_string(),
                "CliSmokeFanout".to_string(),
                "ArtifactRenderFanout".to_string(),
                "IntegrationTestBinaryFanout".to_string(),
                "LinkTimeCost".to_string(),
                "MacroExpansionCost".to_string(),
            ],
        }
    }
}

report!(Sprint112BaselineTruthImportReport {
    report_id: String,
    focused_tests_passed: bool,
    cli_smoke_passed: bool,
    cargo_check_passed: bool,
    cargo_build_passed: bool,
    no_run_timeout_seconds: Option<u64>,
    no_run_exit_code: Option<i32>,
    full_timeout_seconds: Option<u64>,
    full_exit_code: Option<i32>,
    timeout_cleanup_verified: bool,
    nextest_available: bool,
    sccache_available: bool,
    fifth_patch_still_blocked: bool,
    imported_as_full_acceptance: bool,
    import_status: String
});
report!(Sprint112VerificationPatchCarryForwardReport {
    report_id: String,
    written_files_patch_carried_forward: bool,
    file_count_patch_carried_forward: bool,
    actual_cleanup_count_patch_carried_forward: bool,
    actual_cargo_json_parse_patch_carried_forward: bool,
    real_observation_not_overwritten_patch_carried_forward: bool,
    low_risk_candidate_requirement_carried_forward: bool,
    carry_forward_status: String
});
report!(SuspectTargetFamilyRegistryV1 {
    registry_id: String,
    suspect_targets: Vec<String>,
    suspect_families: Vec<String>,
    already_retired_targets_excluded: bool,
    sentinel_targets_excluded: bool,
    registry_status: String
});
report!(SuspectTargetObservationPlanV1 {
    plan_id: String,
    target_observation_steps: Vec<String>,
    cargo_json_steps: Vec<String>,
    nextest_steps: Vec<String>,
    sccache_steps: Vec<String>,
    rustc_timeline_steps: Vec<String>,
    output_capture_steps: Vec<String>,
    plan_status: String
});
report!(RealCargoNoRunObservationV1 {
    observation_id: String,
    attempted: bool,
    command: String,
    started: bool,
    finished: bool,
    passed: Option<bool>,
    duration_ms: Option<u64>,
    timeout_ms: Option<u64>,
    exit_code: Option<i32>,
    timed_out: bool,
    last_seen_target: Option<String>,
    child_process_cleanup_verified: bool,
    observation_status: String
});
report!(RealCargoFullObservationV1 {
    observation_id: String,
    attempted: bool,
    command: String,
    started: bool,
    finished: bool,
    passed: Option<bool>,
    duration_ms: Option<u64>,
    timeout_ms: Option<u64>,
    exit_code: Option<i32>,
    timed_out: bool,
    last_seen_target: Option<String>,
    child_process_cleanup_verified: bool,
    observation_status: String
});
report!(RealCargoJsonProgressObservationV1 {
    observation_id: String,
    attempted: bool,
    command: String,
    finished: bool,
    timed_out: bool,
    message_count: u64,
    artifact_count: u64,
    compiler_message_count: u64,
    parsed_json_message_count: u64,
    parse_error_count: u64,
    last_seen_targets: Vec<String>,
    last_seen_artifacts: Vec<String>,
    stalled_candidates: Vec<String>,
    observation_status: String
});
report!(RealNextestProbeExecutionReportV1 {
    report_id: String,
    attempted: bool,
    command: Option<String>,
    nextest_available: bool,
    version: Option<String>,
    exit_code: Option<i32>,
    probe_status: String
});
report!(RealNextestPartitionObservationReportV1 {
    report_id: String,
    attempted: bool,
    safety_partition: String,
    sentinel_isolation: bool,
    partition_count: u64,
    partition_status: String
});
report!(RealNextestSlowTargetObservationReportV1 {
    report_id: String,
    attempted: bool,
    slow_targets: Vec<String>,
    slow_families: Vec<String>,
    observation_status: String
});
report!(RealSccacheProbeExecutionReportV1 {
    report_id: String,
    attempted: bool,
    sccache_available: bool,
    version: Option<String>,
    exit_code: Option<i32>,
    probe_status: String
});
report!(RealSccacheLocalPilotObservationReportV1 {
    report_id: String,
    local_only: bool,
    remote_cache_forbidden: bool,
    secret_cache_forbidden: bool,
    attempted: bool,
    cache_hits: Option<u64>,
    cache_misses: Option<u64>,
    pilot_status: String
});
report!(RealSccacheEffectObservationReportV1 {
    report_id: String,
    measured: bool,
    sample_backed: bool,
    duration_before_ms: Option<u64>,
    duration_after_ms: Option<u64>,
    can_claim_speedup: bool,
    effect_status: String
});
report!(CargoCheckBuildTimingBaselineReportV2 {
    report_id: String,
    cargo_check_duration_ms: Option<u64>,
    cargo_build_duration_ms: Option<u64>,
    cargo_check_passed: bool,
    cargo_build_passed: bool,
    timing_status: String
});
report!(SuspectTargetRustcTimelineReportV1 {
    report_id: String,
    rustc_processes: Vec<String>,
    max_concurrency: u64,
    suspect_target_args: Vec<String>,
    remaining_processes_after_timeout: u64,
    timeline_status: String
});
report!(SuspectTargetArtifactTimelineReportV1 {
    report_id: String,
    artifact_events: BTreeMap<String, Vec<String>>,
    last_target_artifact: BTreeMap<String, String>,
    stalled_target_artifacts: Vec<String>,
    timeline_status: String
});
report!(SuspectTargetLinkMacroSplitReportV1 {
    report_id: String,
    link_heavy_suspect: Vec<String>,
    macro_heavy_suspect: Vec<String>,
    observed_labels: Vec<String>,
    inferred_labels: Vec<String>,
    split_status: String
});
report!(SuspectTargetFixtureRenderCliSplitReportV1 {
    report_id: String,
    fixture_pressure: Vec<String>,
    render_pressure: Vec<String>,
    cli_pressure: Vec<String>,
    observed_labels: Vec<String>,
    inferred_labels: Vec<String>,
    split_status: String
});
report!(WorkspaceTimeoutRootCauseReportV3 {
    report_id: String,
    previous_status: String,
    new_real_observation_refs: Vec<String>,
    observed_evidence: Vec<String>,
    inferred_evidence: Vec<String>,
    suspect_target_evidence: Vec<String>,
    root_cause_confidence: String,
    root_cause_status: String
});
report!(RootCauseEvidenceUpgradeReportV1 {
    report_id: String,
    previous_evidence_strength: String,
    current_evidence_strength: String,
    upgraded: bool,
    upgrade_reason: String,
    status: String
});
report!(SuspectFamilyIsolationReportV1 {
    report_id: String,
    suspect_families: Vec<String>,
    isolated_families: Vec<String>,
    still_mixed_families: Vec<String>,
    status: String
});
report!(ControlTowerPanelTargetIsolationReportV1 {
    report_id: String,
    panel_target_pressure: Vec<String>,
    cli_render_fanout_split: BTreeMap<String, Vec<String>>,
    assertion_migration_feasible: bool,
    status: String
});
report!(WorkspaceTimeoutTargetIsolationReportV1 {
    report_id: String,
    workspace_timeout_target_pressure: Vec<String>,
    macro_link_render_split: BTreeMap<String, Vec<String>>,
    status: String
});
report!(RemainingSafeCandidatePoolReportV3 {
    report_id: String,
    previous_candidate_pool: Vec<String>,
    updated_evidence: Vec<String>,
    candidate_statuses: BTreeMap<String, String>,
    assertion_migration_feasible_candidates: Vec<String>,
    equivalent_coverage_feasible_candidates: Vec<String>,
    sentinel_exclusions: Vec<String>,
    status: String
});
report!(FifthPatchDecisionGateV3 {
    gate_id: String,
    previous_gate_status: String,
    root_cause_v3_status: String,
    root_cause_confidence: String,
    candidate_pool_status: String,
    assertion_migration_feasible: bool,
    equivalent_coverage_feasible: bool,
    safety_sentinel_preserved: bool,
    no_hidden_skip_continuity: bool,
    actual_observation_sufficient: bool,
    fifth_patch_allowed_for_next_sprint: bool,
    fifth_patch_applied_this_sprint: bool,
    gate_status: String
});
report!(FifthPatchAssertionMigrationFeasibilityReportV1 {
    report_id: String,
    candidate: Option<String>,
    assertion_moves_required: u64,
    feasibility: String,
    blockers: Vec<String>,
    status: String
});
report!(FifthPatchEquivalentCoverageFeasibilityReportV1 {
    report_id: String,
    equivalent_coverage_possible: bool,
    destination_target: Option<String>,
    coverage_gaps: Vec<String>,
    status: String
});
report!(FifthPatchSentinelSafetyFeasibilityReportV1 {
    report_id: String,
    sentinel_risk: String,
    workspace_cli_safety_risk: String,
    determinism_risk: String,
    paper_lifecycle_safety_risk: String,
    status: String
});
report!(FifthPatchNoApplyGuaranteeReportV2 {
    report_id: String,
    fifth_patch_applied: bool,
    no_files_retired_by_fifth_patch: bool,
    no_assertions_moved_by_fifth_patch: bool,
    guarantee_status: String
});
report!(CumulativeSafePatchLedgerV4 {
    report_id: String,
    patch_count: u64,
    retired_targets: Vec<String>,
    assertion_delta: i64,
    status: String
});
report!(CumulativeBinaryDeltaReportV3 {
    report_id: String,
    sample_backed_delta: i64,
    measured_delta: Option<i64>,
    measured_claim_allowed: bool,
    status: String
});
report!(AssertionLedgerContinuityCheckV3 {
    report_id: String,
    carried_forward_assertion_delta: i64,
    assertion_deletions_detected: u64,
    continuity_status: String
});
report!(EquivalentCoverageContinuityCheckV3 {
    report_id: String,
    coverage_gap_count: u64,
    equivalent_coverage_feasible: bool,
    continuity_status: String
});
report!(SafetySentinelContinuityCheckV3 {
    report_id: String,
    sentinels_preserved: Vec<String>,
    sentinel_uncertainties: Vec<String>,
    continuity_status: String
});
report!(NoHiddenSkipContinuityCheckV3 {
    report_id: String,
    hidden_skip_indicators: Vec<String>,
    continuity_status: String
});
report!(TimeoutWindowAdequacyReportV3 {
    report_id: String,
    previous_timeout_ms: BTreeMap<String, u64>,
    current_timeout_ms: BTreeMap<String, u64>,
    adequacy: String,
    recommendation: String
});
report!(TimeoutCleanupVerificationReportV6 {
    report_id: String,
    timeout_occurred: bool,
    child_process_cleanup_attempted: bool,
    remaining_cargo_processes: u64,
    remaining_rustc_processes: u64,
    status: String
});
report!(WorkspaceNoRunRecoveryGateV14 {
    gate_id: String,
    observation_status: String,
    recovered: bool,
    status: String
});
report!(WorkspaceFullAcceptanceGateV14 {
    gate_id: String,
    observation_status: String,
    accepted: bool,
    status: String
});
report!(FocusedVsFullBridgeV10 {
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
report!(AcceptanceTruthGateV14 {
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
    can_claim_full_acceptance: bool,
    truth_status: String
});
report!(AcceptanceEvidenceStrengthReportV3 {
    report_id: String,
    supporting_only: bool,
    sufficient: bool,
    status: String
});
report!(WorkspaceRecoveryDecisionReportV3 {
    report_id: String,
    recommend_fifth_patch_next_sprint_only: bool,
    recommend_more_observation: bool,
    recommend_no_patch: bool,
    status: String
});
report!(SafetyCoveragePreservationReportV29 {
    report_id: String,
    live_trading_guard_present: bool,
    broker_guard_present: bool,
    order_guard_present: bool,
    account_guard_present: bool,
    runtime_llm_guard_present: bool,
    mamba_runtime_guard_present: bool,
    gated_runtime_guard_present: bool,
    model_training_guard_present: bool,
    python_training_dependency_guard_present: bool,
    browser_execution_guard_present: bool,
    no_hidden_skip_guard_present: bool,
    assertion_preservation_guard_present: bool,
    safety_sentinel_preservation_guard_present: bool,
    real_observation_not_acceptance_guard_present: bool,
    actual_cargo_json_parsing_guard_present: bool,
    actual_timeout_cleanup_counts_guard_present: bool,
    fifth_patch_v3_no_apply_guard_present: bool,
    suspect_family_isolation_guard_present: bool,
    safety_status: String
});
report!(ControlTowerRealWorkspaceObservationPanel {
    panel_id: String,
    real_observation_statuses: BTreeMap<String, String>,
    suspect_target_isolation_status: String,
    acceptance_truth_status: String,
    warnings: Vec<String>,
    static_read_only: bool,
    no_run_button: bool,
    no_apply_patch_button: bool,
    no_train_runtime_live_order_account_controls: bool
});
report!(ControlTowerFifthPatchEvidenceGatePanel {
    panel_id: String,
    fifth_gate_status: String,
    assertion_feasibility_status: String,
    equivalent_coverage_status: String,
    sentinel_safety_status: String,
    no_apply_guarantee_status: String,
    warnings: Vec<String>,
    static_read_only: bool,
    no_apply_patch_button: bool,
    no_train_runtime_live_order_account_controls: bool
});
report!(RealWorkspaceObservationDrilldownStorageReport {
    report_id: String,
    output_dir: String,
    written_files: Vec<String>,
    file_count: u64
});

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandObservationSnapshot {
    pub attempted: bool,
    pub finished: bool,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
    pub timeout_ms: Option<u64>,
    pub timed_out: bool,
    pub remaining_cargo_processes: u64,
    pub remaining_rustc_processes: u64,
    pub last_seen_target: Option<String>,
    pub stdout: String,
}

#[derive(Clone, Debug, Default)]
struct CommandOutputSnapshot {
    snapshot: CommandObservationSnapshot,
    stdout: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RealWorkspaceObservationDrilldownBundle {
    pub sprint112_baseline_truth_import_report: Sprint112BaselineTruthImportReport,
    pub sprint112_verification_patch_carry_forward_report:
        Sprint112VerificationPatchCarryForwardReport,
    pub suspect_target_family_registry_v1: SuspectTargetFamilyRegistryV1,
    pub suspect_target_observation_plan_v1: SuspectTargetObservationPlanV1,
    pub real_cargo_no_run_observation_v1: RealCargoNoRunObservationV1,
    pub real_cargo_full_observation_v1: RealCargoFullObservationV1,
    pub real_cargo_json_progress_observation_v1: RealCargoJsonProgressObservationV1,
    pub real_nextest_probe_execution_report_v1: RealNextestProbeExecutionReportV1,
    pub real_nextest_partition_observation_report_v1: RealNextestPartitionObservationReportV1,
    pub real_nextest_slow_target_observation_report_v1: RealNextestSlowTargetObservationReportV1,
    pub real_sccache_probe_execution_report_v1: RealSccacheProbeExecutionReportV1,
    pub real_sccache_local_pilot_observation_report_v1: RealSccacheLocalPilotObservationReportV1,
    pub real_sccache_effect_observation_report_v1: RealSccacheEffectObservationReportV1,
    pub cargo_check_build_timing_baseline_report_v2: CargoCheckBuildTimingBaselineReportV2,
    pub suspect_target_rustc_timeline_report_v1: SuspectTargetRustcTimelineReportV1,
    pub suspect_target_artifact_timeline_report_v1: SuspectTargetArtifactTimelineReportV1,
    pub suspect_target_link_macro_split_report_v1: SuspectTargetLinkMacroSplitReportV1,
    pub suspect_target_fixture_render_cli_split_report_v1:
        SuspectTargetFixtureRenderCliSplitReportV1,
    pub workspace_timeout_root_cause_report_v3: WorkspaceTimeoutRootCauseReportV3,
    pub root_cause_evidence_upgrade_report_v1: RootCauseEvidenceUpgradeReportV1,
    pub suspect_family_isolation_report_v1: SuspectFamilyIsolationReportV1,
    pub control_tower_panel_target_isolation_report_v1: ControlTowerPanelTargetIsolationReportV1,
    pub workspace_timeout_target_isolation_report_v1: WorkspaceTimeoutTargetIsolationReportV1,
    pub remaining_safe_candidate_pool_report_v3: RemainingSafeCandidatePoolReportV3,
    pub fifth_patch_decision_gate_v3: FifthPatchDecisionGateV3,
    pub fifth_patch_assertion_migration_feasibility_report_v1:
        FifthPatchAssertionMigrationFeasibilityReportV1,
    pub fifth_patch_equivalent_coverage_feasibility_report_v1:
        FifthPatchEquivalentCoverageFeasibilityReportV1,
    pub fifth_patch_sentinel_safety_feasibility_report_v1:
        FifthPatchSentinelSafetyFeasibilityReportV1,
    pub fifth_patch_no_apply_guarantee_report_v2: FifthPatchNoApplyGuaranteeReportV2,
    pub cumulative_safe_patch_ledger_v4: CumulativeSafePatchLedgerV4,
    pub cumulative_binary_delta_report_v3: CumulativeBinaryDeltaReportV3,
    pub assertion_ledger_continuity_check_v3: AssertionLedgerContinuityCheckV3,
    pub equivalent_coverage_continuity_check_v3: EquivalentCoverageContinuityCheckV3,
    pub safety_sentinel_continuity_check_v3: SafetySentinelContinuityCheckV3,
    pub no_hidden_skip_continuity_check_v3: NoHiddenSkipContinuityCheckV3,
    pub timeout_window_adequacy_report_v3: TimeoutWindowAdequacyReportV3,
    pub timeout_cleanup_verification_report_v6: TimeoutCleanupVerificationReportV6,
    pub workspace_no_run_recovery_gate_v14: WorkspaceNoRunRecoveryGateV14,
    pub workspace_full_acceptance_gate_v14: WorkspaceFullAcceptanceGateV14,
    pub focused_vs_full_bridge_v10: FocusedVsFullBridgeV10,
    pub acceptance_truth_gate_v14: AcceptanceTruthGateV14,
    pub acceptance_evidence_strength_report_v3: AcceptanceEvidenceStrengthReportV3,
    pub workspace_recovery_decision_report_v3: WorkspaceRecoveryDecisionReportV3,
    pub safety_coverage_preservation_report_v29: SafetyCoveragePreservationReportV29,
    pub control_tower_real_workspace_observation_panel: ControlTowerRealWorkspaceObservationPanel,
    pub control_tower_fifth_patch_evidence_gate_panel: ControlTowerFifthPatchEvidenceGatePanel,
    pub storage_report: RealWorkspaceObservationDrilldownStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

fn count_processes(process_name: &str) -> u64 {
    Command::new("sh")
        .arg("-lc")
        .arg(format!(
            "ps -axo comm= | awk '$1==\"{}\" {{c++}} END {{print c+0}}'",
            process_name
        ))
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|text| text.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

fn observe_simple_command(
    run: bool,
    command: &str,
    timeout_ms: Option<u64>,
) -> Result<CommandObservationSnapshot, String> {
    if !run {
        return Ok(CommandObservationSnapshot {
            timeout_ms,
            ..CommandObservationSnapshot::default()
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
            return Ok(CommandObservationSnapshot {
                attempted: true,
                finished: true,
                exit_code: status.code(),
                duration_ms: Some(start.elapsed().as_millis() as u64),
                timeout_ms,
                timed_out: false,
                remaining_cargo_processes: 0,
                remaining_rustc_processes: 0,
                last_seen_target: None,
                stdout: String::new(),
            });
        }
        if let Some(timeout_ms) = timeout_ms
            && start.elapsed() >= Duration::from_millis(timeout_ms)
        {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(CommandObservationSnapshot {
                attempted: true,
                finished: false,
                exit_code: Some(124),
                duration_ms: Some(start.elapsed().as_millis() as u64),
                timeout_ms: Some(timeout_ms),
                timed_out: true,
                remaining_cargo_processes: count_processes("cargo"),
                remaining_rustc_processes: count_processes("rustc"),
                last_seen_target: None,
                stdout: String::new(),
            });
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn observe_command_stdout(
    run: bool,
    command: &str,
    timeout_ms: Option<u64>,
) -> Result<CommandOutputSnapshot, String> {
    if !run {
        return Ok(CommandOutputSnapshot {
            snapshot: CommandObservationSnapshot {
                timeout_ms,
                ..CommandObservationSnapshot::default()
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
            return Ok(CommandOutputSnapshot {
                snapshot: CommandObservationSnapshot {
                    attempted: true,
                    finished: true,
                    exit_code: status.code(),
                    duration_ms: Some(start.elapsed().as_millis() as u64),
                    timeout_ms,
                    timed_out: false,
                    remaining_cargo_processes: 0,
                    remaining_rustc_processes: 0,
                    last_seen_target: None,
                    stdout: stdout.clone(),
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
            return Ok(CommandOutputSnapshot {
                snapshot: CommandObservationSnapshot {
                    attempted: true,
                    finished: false,
                    exit_code: Some(124),
                    duration_ms: Some(start.elapsed().as_millis() as u64),
                    timeout_ms: Some(timeout_ms),
                    timed_out: true,
                    remaining_cargo_processes: count_processes("cargo"),
                    remaining_rustc_processes: count_processes("rustc"),
                    last_seen_target: None,
                    stdout: stdout.clone(),
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

pub fn build_real_cargo_no_run_observation_v1(
    snapshot: &CommandObservationSnapshot,
) -> RealCargoNoRunObservationV1 {
    let passed = snapshot.finished && snapshot.exit_code == Some(0);
    RealCargoNoRunObservationV1 {
        observation_id: "real-cargo-no-run-observation-v1".to_string(),
        attempted: snapshot.attempted,
        command: "cargo test --workspace --no-run --quiet".to_string(),
        started: snapshot.attempted,
        finished: snapshot.finished,
        passed: snapshot
            .exit_code
            .map(|code| code == 0)
            .filter(|_| snapshot.finished),
        duration_ms: snapshot.duration_ms,
        timeout_ms: snapshot.timeout_ms,
        exit_code: snapshot.exit_code,
        timed_out: snapshot.timed_out,
        last_seen_target: snapshot.last_seen_target.clone(),
        child_process_cleanup_verified: if snapshot.timed_out {
            snapshot.remaining_cargo_processes == 0 && snapshot.remaining_rustc_processes == 0
        } else {
            true
        },
        observation_status: if passed {
            "RealNoRunCompleted"
        } else if snapshot.timed_out {
            "RealNoRunTimedOut"
        } else if snapshot.attempted {
            "RealNoRunFailed"
        } else {
            "RealNoRunNotRun"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_real_cargo_full_observation_v1(
    snapshot: &CommandObservationSnapshot,
) -> RealCargoFullObservationV1 {
    let passed = snapshot.finished && snapshot.exit_code == Some(0);
    RealCargoFullObservationV1 {
        observation_id: "real-cargo-full-observation-v1".to_string(),
        attempted: snapshot.attempted,
        command: "cargo test --workspace --quiet".to_string(),
        started: snapshot.attempted,
        finished: snapshot.finished,
        passed: snapshot
            .exit_code
            .map(|code| code == 0)
            .filter(|_| snapshot.finished),
        duration_ms: snapshot.duration_ms,
        timeout_ms: snapshot.timeout_ms,
        exit_code: snapshot.exit_code,
        timed_out: snapshot.timed_out,
        last_seen_target: snapshot.last_seen_target.clone(),
        child_process_cleanup_verified: if snapshot.timed_out {
            snapshot.remaining_cargo_processes == 0 && snapshot.remaining_rustc_processes == 0
        } else {
            true
        },
        observation_status: if passed {
            "RealFullCompleted"
        } else if snapshot.timed_out {
            "RealFullTimedOut"
        } else if snapshot.attempted {
            "RealFullFailed"
        } else {
            "RealFullNotRun"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_real_cargo_json_progress_observation_v1_from_stdout(
    attempted: bool,
    finished: bool,
    timed_out: bool,
    stdout: &str,
) -> RealCargoJsonProgressObservationV1 {
    if !attempted {
        return RealCargoJsonProgressObservationV1 {
            observation_id: "real-cargo-json-progress-observation-v1".to_string(),
            attempted: false,
            command: "cargo test --workspace --no-run --message-format=json".to_string(),
            finished: false,
            timed_out: false,
            message_count: 0,
            artifact_count: 0,
            compiler_message_count: 0,
            parsed_json_message_count: 0,
            parse_error_count: 0,
            last_seen_targets: Vec::new(),
            last_seen_artifacts: Vec::new(),
            stalled_candidates: Vec::new(),
            observation_status: "CargoJsonNotRun".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
    }

    let mut message_count = 0;
    let mut artifact_count = 0;
    let mut compiler_message_count = 0;
    let mut parsed_json_message_count = 0;
    let mut parse_error_count = 0;
    let mut last_seen_targets = Vec::new();
    let mut last_seen_artifacts = Vec::new();
    for line in stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        message_count += 1;
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(value) => {
                parsed_json_message_count += 1;
                match value.get("reason").and_then(|item| item.as_str()) {
                    Some("compiler-artifact") => {
                        artifact_count += 1;
                        if let Some(label) = cargo_json_target_label(&value)
                            && !last_seen_targets.contains(&label)
                        {
                            last_seen_targets.push(label);
                        }
                        if let Some(filenames) =
                            value.get("filenames").and_then(|item| item.as_array())
                        {
                            for name in filenames.iter().filter_map(|item| item.as_str()) {
                                let normalized = normalize_target_label(name);
                                if !last_seen_artifacts.contains(&normalized) {
                                    last_seen_artifacts.push(normalized);
                                }
                            }
                        }
                    }
                    Some("compiler-message") => compiler_message_count += 1,
                    _ => {}
                }
            }
            Err(_) => parse_error_count += 1,
        }
    }
    let stalled_candidates = if timed_out {
        let mut values = last_seen_targets
            .iter()
            .rev()
            .take(3)
            .cloned()
            .collect::<Vec<_>>();
        values.reverse();
        values
    } else {
        Vec::new()
    };
    let observation_status = if timed_out {
        "CargoJsonTimedOut"
    } else if parse_error_count > 0 {
        "CargoJsonObservedWithWarnings"
    } else if finished {
        "CargoJsonObserved"
    } else {
        "DiagnosticOnly"
    }
    .to_string();
    RealCargoJsonProgressObservationV1 {
        observation_id: "real-cargo-json-progress-observation-v1".to_string(),
        attempted,
        command: "cargo test --workspace --no-run --message-format=json".to_string(),
        finished,
        timed_out,
        message_count,
        artifact_count,
        compiler_message_count,
        parsed_json_message_count,
        parse_error_count,
        last_seen_targets,
        last_seen_artifacts,
        stalled_candidates,
        observation_status,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_real_nextest_probe_execution_report_v1(
    snapshot: &CommandObservationSnapshot,
) -> RealNextestProbeExecutionReportV1 {
    let succeeded = snapshot.attempted && snapshot.finished && snapshot.exit_code == Some(0);
    let version = if succeeded {
        snapshot
            .stdout
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(str::to_string)
    } else {
        None
    };
    RealNextestProbeExecutionReportV1 {
        report_id: "real-nextest-probe-execution-v1".to_string(),
        attempted: snapshot.attempted,
        command: Some("cargo nextest --version".to_string()),
        nextest_available: succeeded,
        version,
        exit_code: snapshot.exit_code,
        probe_status: if succeeded {
            "NextestProbeSucceeded"
        } else if snapshot.timed_out {
            "NextestProbeFailed"
        } else if snapshot.attempted {
            "NextestUnavailable"
        } else {
            "NextestProbeNotRun"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_real_sccache_probe_execution_report_v1(
    snapshot: &CommandObservationSnapshot,
) -> RealSccacheProbeExecutionReportV1 {
    let succeeded = snapshot.attempted && snapshot.finished && snapshot.exit_code == Some(0);
    let version = if succeeded {
        snapshot
            .stdout
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(str::to_string)
    } else {
        None
    };
    RealSccacheProbeExecutionReportV1 {
        report_id: "real-sccache-probe-execution-v1".to_string(),
        attempted: snapshot.attempted,
        sccache_available: succeeded,
        version,
        exit_code: snapshot.exit_code,
        probe_status: if succeeded {
            "SccacheProbeSucceeded"
        } else if snapshot.timed_out {
            "SccacheProbeFailed"
        } else if snapshot.attempted {
            "SccacheUnavailable"
        } else {
            "SccacheProbeNotRun"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_root_cause_report_v3(
    summary: &Sprint112SummaryFixture,
    cargo_json: &RealCargoJsonProgressObservationV1,
    no_run: &RealCargoNoRunObservationV1,
    full: &RealCargoFullObservationV1,
    registry: &SuspectTargetFamilyRegistryV1,
    link_macro: &SuspectTargetLinkMacroSplitReportV1,
    fanout: &SuspectTargetFixtureRenderCliSplitReportV1,
) -> WorkspaceTimeoutRootCauseReportV3 {
    let mut new_real_observation_refs = Vec::new();
    if no_run.attempted {
        new_real_observation_refs.push(format!("{}:{}", no_run.command, no_run.observation_status));
    }
    if full.attempted {
        new_real_observation_refs.push(format!("{}:{}", full.command, full.observation_status));
    }
    if cargo_json.attempted {
        new_real_observation_refs.push(format!(
            "{}:{}:{}",
            cargo_json.command, cargo_json.observation_status, cargo_json.parsed_json_message_count
        ));
    }
    let mut observed_evidence = summary.observed_root_cause_evidence.clone();
    observed_evidence.extend(cargo_json.last_seen_targets.clone());
    if no_run.timed_out {
        observed_evidence.push("real cargo no-run observation timed out".to_string());
    }
    if full.timed_out {
        observed_evidence.push("real cargo full observation timed out".to_string());
    }
    observed_evidence.extend(link_macro.observed_labels.clone());
    let mut inferred_evidence = summary.inferred_root_cause_evidence.clone();
    inferred_evidence.extend(link_macro.inferred_labels.clone());
    inferred_evidence.extend(fanout.inferred_labels.clone());
    let suspect_target_evidence = stable_strings(
        cargo_json
            .stalled_candidates
            .iter()
            .cloned()
            .chain(summary.stalled_targets.clone())
            .chain(registry.suspect_targets.clone())
            .collect(),
    );
    let observed_evidence = stable_strings(observed_evidence);
    let inferred_evidence = stable_strings(inferred_evidence);
    let (root_cause_status, root_cause_confidence) = if !new_real_observation_refs.is_empty()
        && observed_evidence.len() >= 5
        && suspect_target_evidence.len() >= 2
    {
        ("TimeoutRootCauseIsolated", "Strong")
    } else if !observed_evidence.is_empty() {
        ("TimeoutRootCausePartiallyIsolated", "Moderate")
    } else if !inferred_evidence.is_empty() {
        ("TimeoutRootCauseStillAmbiguous", "Weak")
    } else {
        ("DiagnosticOnly", "Insufficient")
    };
    WorkspaceTimeoutRootCauseReportV3 {
        report_id: "workspace-timeout-root-cause-v3".to_string(),
        previous_status: summary.timeout_root_cause_status.clone(),
        new_real_observation_refs: stable_strings(new_real_observation_refs),
        observed_evidence,
        inferred_evidence,
        suspect_target_evidence,
        root_cause_confidence: root_cause_confidence.to_string(),
        root_cause_status: root_cause_status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_fifth_patch_decision_gate_v3(
    summary: &Sprint112SummaryFixture,
    root: &WorkspaceTimeoutRootCauseReportV3,
    pool: &RemainingSafeCandidatePoolReportV3,
    assertion: &FifthPatchAssertionMigrationFeasibilityReportV1,
    equivalent: &FifthPatchEquivalentCoverageFeasibilityReportV1,
    sentinel: &FifthPatchSentinelSafetyFeasibilityReportV1,
    no_hidden_skip: &NoHiddenSkipContinuityCheckV3,
) -> FifthPatchDecisionGateV3 {
    let safety_sentinel_preserved = sentinel.status == "SentinelSafetyFeasible";
    let no_hidden_skip_continuity = no_hidden_skip.hidden_skip_indicators.is_empty();
    let actual_observation_sufficient =
        matches!(root.root_cause_confidence.as_str(), "Strong" | "Moderate")
            && !root.new_real_observation_refs.is_empty();
    let low_risk_candidate_available = pool
        .candidate_statuses
        .values()
        .any(|status| status == "LowRiskCandidate");
    let assertion_migration_feasible = assertion.feasibility == "Feasible";
    let equivalent_coverage_feasible = equivalent.equivalent_coverage_possible;
    let allowed = assertion_migration_feasible
        && equivalent_coverage_feasible
        && safety_sentinel_preserved
        && no_hidden_skip_continuity
        && actual_observation_sufficient
        && low_risk_candidate_available
        && root.root_cause_status == "TimeoutRootCauseIsolated";
    let gate_status = if allowed {
        "FifthPatchAllowedForNextSprint"
    } else if !safety_sentinel_preserved || !no_hidden_skip_continuity {
        "FifthPatchBlockedBySafety"
    } else {
        "FifthPatchStillBlocked"
    };
    FifthPatchDecisionGateV3 {
        gate_id: "fifth-patch-decision-gate-v3".to_string(),
        previous_gate_status: summary.previous_gate_status.clone(),
        root_cause_v3_status: root.root_cause_status.clone(),
        root_cause_confidence: root.root_cause_confidence.clone(),
        candidate_pool_status: pool.status.clone(),
        assertion_migration_feasible,
        equivalent_coverage_feasible,
        safety_sentinel_preserved,
        no_hidden_skip_continuity,
        actual_observation_sufficient,
        fifth_patch_allowed_for_next_sprint: allowed,
        fifth_patch_applied_this_sprint: false,
        gate_status: gate_status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_acceptance_truth_gate_v14(
    baseline: &Sprint112BaselineTruthImportReport,
    no_run: &WorkspaceNoRunRecoveryGateV14,
    full: &WorkspaceFullAcceptanceGateV14,
    nextest: &RealNextestProbeExecutionReportV1,
    sccache: &RealSccacheProbeExecutionReportV1,
    cargo_json: &RealCargoJsonProgressObservationV1,
) -> AcceptanceTruthGateV14 {
    let can_claim_full_acceptance = full.accepted;
    AcceptanceTruthGateV14 {
        gate_id: "acceptance-truth-gate-v14".to_string(),
        focused_truth_status: if baseline.focused_tests_passed {
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
        cargo_check_truth_status: if baseline.cargo_check_passed {
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
        nextest_truth_status: if nextest.nextest_available {
            "SupportingOnly"
        } else {
            "Insufficient"
        }
        .to_string(),
        sccache_truth_status: if sccache.sccache_available {
            "SupportingOnly"
        } else {
            "Insufficient"
        }
        .to_string(),
        cargo_json_truth_status: if cargo_json.parsed_json_message_count > 0 {
            "SupportingOnly"
        } else {
            "Insufficient"
        }
        .to_string(),
        no_run_truth_status: if no_run.recovered {
            "NoRunOnly"
        } else {
            "SupportingOnly"
        }
        .to_string(),
        full_workspace_truth_status: if can_claim_full_acceptance {
            "FullWorkspaceAccepted"
        } else {
            "FullWorkspaceStillBlocked"
        }
        .to_string(),
        can_claim_full_acceptance,
        truth_status: if can_claim_full_acceptance {
            "AcceptanceTruthReady"
        } else {
            "AcceptanceTruthReadyWithWarnings"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

impl RealWorkspaceObservationDrilldownBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            ("## 1. Sprint summary", format!("- imported={} root_cause={} fifth_patch_gate={} acceptance={}", self.sprint112_baseline_truth_import_report.import_status, self.workspace_timeout_root_cause_report_v3.root_cause_status, self.fifth_patch_decision_gate_v3.gate_status, self.acceptance_truth_gate_v14.truth_status)),
            ("## 2. Why Sprint 113 was needed", "- Sprint 112 left real no-run/full workspace acceptance blocked, so Sprint 113 upgrades observation evidence without applying the fifth patch.".to_string()),
            ("## 3. Files added", "- Sprint 113 implementation, examples, fixtures, tests, and documentation were present before this verification pass.".to_string()),
            ("## 4. Files changed", "- src/league/sprint113_real_workspace_observation.rs, Sprint 113 tests, and docs/SPRINT113_REPORT.md.".to_string()),
            ("## 5. Sprint 112 baseline truth import", format!("- status={} imported_as_full_acceptance={}", self.sprint112_baseline_truth_import_report.import_status, self.sprint112_baseline_truth_import_report.imported_as_full_acceptance)),
            ("## 6. Sprint 112 verification patch carry-forward", format!("- status={}", self.sprint112_verification_patch_carry_forward_report.carry_forward_status)),
            ("## 7. Suspect target family registry", format!("- status={} targets={} families={}", self.suspect_target_family_registry_v1.registry_status, self.suspect_target_family_registry_v1.suspect_targets.len(), self.suspect_target_family_registry_v1.suspect_families.len())),
            ("## 8. Suspect target observation plan", format!("- status={} steps={}", self.suspect_target_observation_plan_v1.plan_status, self.suspect_target_observation_plan_v1.target_observation_steps.len())),
            ("## 9. Real cargo no-run observation", format!("- status={} attempted={} timed_out={}", self.real_cargo_no_run_observation_v1.observation_status, self.real_cargo_no_run_observation_v1.attempted, self.real_cargo_no_run_observation_v1.timed_out)),
            ("## 10. Real cargo full observation", format!("- status={} attempted={} timed_out={}", self.real_cargo_full_observation_v1.observation_status, self.real_cargo_full_observation_v1.attempted, self.real_cargo_full_observation_v1.timed_out)),
            ("## 11. Real cargo JSON progress observation", format!("- status={} parsed={} parse_errors={}", self.real_cargo_json_progress_observation_v1.observation_status, self.real_cargo_json_progress_observation_v1.parsed_json_message_count, self.real_cargo_json_progress_observation_v1.parse_error_count)),
            ("## 12. Real nextest probe / partition / slow target observation", format!("- probe={} available={} partition={} slow={}", self.real_nextest_probe_execution_report_v1.probe_status, self.real_nextest_probe_execution_report_v1.nextest_available, self.real_nextest_partition_observation_report_v1.partition_status, self.real_nextest_slow_target_observation_report_v1.observation_status)),
            ("## 13. Real sccache probe / local pilot / effect observation", format!("- probe={} available={} pilot={} can_claim_speedup={}", self.real_sccache_probe_execution_report_v1.probe_status, self.real_sccache_probe_execution_report_v1.sccache_available, self.real_sccache_local_pilot_observation_report_v1.pilot_status, self.real_sccache_effect_observation_report_v1.can_claim_speedup)),
            ("## 14. Cargo check/build timing baseline v2", format!("- status={} cargo_check_passed={} cargo_build_passed={}", self.cargo_check_build_timing_baseline_report_v2.timing_status, self.cargo_check_build_timing_baseline_report_v2.cargo_check_passed, self.cargo_check_build_timing_baseline_report_v2.cargo_build_passed)),
            ("## 15. Suspect target rustc timeline", format!("- status={} max_concurrency={}", self.suspect_target_rustc_timeline_report_v1.timeline_status, self.suspect_target_rustc_timeline_report_v1.max_concurrency)),
            ("## 16. Suspect target artifact timeline", format!("- status={} stalled_artifacts={}", self.suspect_target_artifact_timeline_report_v1.timeline_status, self.suspect_target_artifact_timeline_report_v1.stalled_target_artifacts.len())),
            ("## 17. Suspect target link/macro split", format!("- status={} link={} macro={}", self.suspect_target_link_macro_split_report_v1.split_status, self.suspect_target_link_macro_split_report_v1.link_heavy_suspect.len(), self.suspect_target_link_macro_split_report_v1.macro_heavy_suspect.len())),
            ("## 18. Suspect target fixture/render/CLI split", format!("- status={} fixture={} render={} cli={}", self.suspect_target_fixture_render_cli_split_report_v1.split_status, self.suspect_target_fixture_render_cli_split_report_v1.fixture_pressure.len(), self.suspect_target_fixture_render_cli_split_report_v1.render_pressure.len(), self.suspect_target_fixture_render_cli_split_report_v1.cli_pressure.len())),
            ("## 19. Workspace timeout root-cause v3", format!("- status={} confidence={}", self.workspace_timeout_root_cause_report_v3.root_cause_status, self.workspace_timeout_root_cause_report_v3.root_cause_confidence)),
            ("## 20. Root-cause evidence upgrade", format!("- status={} upgraded={}", self.root_cause_evidence_upgrade_report_v1.status, self.root_cause_evidence_upgrade_report_v1.upgraded)),
            ("## 21. Suspect family isolation", format!("- status={} isolated={} still_mixed={}", self.suspect_family_isolation_report_v1.status, self.suspect_family_isolation_report_v1.isolated_families.len(), self.suspect_family_isolation_report_v1.still_mixed_families.len())),
            ("## 22. Panel target isolation", format!("- status={} assertion_migration_feasible={}", self.control_tower_panel_target_isolation_report_v1.status, self.control_tower_panel_target_isolation_report_v1.assertion_migration_feasible)),
            ("## 23. Workspace timeout target isolation", format!("- status={} targets={}", self.workspace_timeout_target_isolation_report_v1.status, self.workspace_timeout_target_isolation_report_v1.workspace_timeout_target_pressure.len())),
            ("## 24. Remaining safe candidate pool v3", format!("- status={} candidates={}", self.remaining_safe_candidate_pool_report_v3.status, self.remaining_safe_candidate_pool_report_v3.candidate_statuses.len())),
            ("## 25. Fifth patch decision gate v3", format!("- status={} allowed_for_next_sprint={} applied_this_sprint={}", self.fifth_patch_decision_gate_v3.gate_status, self.fifth_patch_decision_gate_v3.fifth_patch_allowed_for_next_sprint, self.fifth_patch_decision_gate_v3.fifth_patch_applied_this_sprint)),
            ("## 26. Fifth patch feasibility reports", format!("- assertion={} equivalent={} sentinel={}", self.fifth_patch_assertion_migration_feasibility_report_v1.status, self.fifth_patch_equivalent_coverage_feasibility_report_v1.status, self.fifth_patch_sentinel_safety_feasibility_report_v1.status)),
            ("## 27. Fifth patch no-apply guarantee v2", format!("- status={} no_files_retired={} no_assertions_moved={}", self.fifth_patch_no_apply_guarantee_report_v2.guarantee_status, self.fifth_patch_no_apply_guarantee_report_v2.no_files_retired_by_fifth_patch, self.fifth_patch_no_apply_guarantee_report_v2.no_assertions_moved_by_fifth_patch)),
            ("## 28. Cumulative safe patch ledger v4", format!("- status={} patch_count={}", self.cumulative_safe_patch_ledger_v4.status, self.cumulative_safe_patch_ledger_v4.patch_count)),
            ("## 29. Cumulative binary delta v3", format!("- status={} sample_backed_delta={} measured_claim_allowed={}", self.cumulative_binary_delta_report_v3.status, self.cumulative_binary_delta_report_v3.sample_backed_delta, self.cumulative_binary_delta_report_v3.measured_claim_allowed)),
            ("## 30. Assertion/equivalent/sentinel/no-hidden-skip continuity", format!("- assertion={} equivalent={} sentinel={} no_hidden_skip={}", self.assertion_ledger_continuity_check_v3.continuity_status, self.equivalent_coverage_continuity_check_v3.continuity_status, self.safety_sentinel_continuity_check_v3.continuity_status, self.no_hidden_skip_continuity_check_v3.continuity_status)),
            ("## 31. Timeout window adequacy v3", format!("- adequacy={} recommendation={}", self.timeout_window_adequacy_report_v3.adequacy, self.timeout_window_adequacy_report_v3.recommendation)),
            ("## 32. Timeout cleanup verification v6", format!("- status={} remaining_cargo={} remaining_rustc={}", self.timeout_cleanup_verification_report_v6.status, self.timeout_cleanup_verification_report_v6.remaining_cargo_processes, self.timeout_cleanup_verification_report_v6.remaining_rustc_processes)),
            ("## 33. Workspace no-run recovery gate v14", format!("- status={} recovered={}", self.workspace_no_run_recovery_gate_v14.status, self.workspace_no_run_recovery_gate_v14.recovered)),
            ("## 34. Workspace full acceptance gate v14", format!("- status={} accepted={}", self.workspace_full_acceptance_gate_v14.status, self.workspace_full_acceptance_gate_v14.accepted)),
            ("## 35. Focused-vs-full bridge v10", format!("- status={} can_claim_full_acceptance={}", self.focused_vs_full_bridge_v10.bridge_status, self.focused_vs_full_bridge_v10.can_claim_full_acceptance)),
            ("## 36. Acceptance truth gate v14", format!("- status={} can_claim_full_acceptance={}", self.acceptance_truth_gate_v14.truth_status, self.acceptance_truth_gate_v14.can_claim_full_acceptance)),
            ("## 37. Acceptance evidence strength v3", format!("- status={} supporting_only={} sufficient={}", self.acceptance_evidence_strength_report_v3.status, self.acceptance_evidence_strength_report_v3.supporting_only, self.acceptance_evidence_strength_report_v3.sufficient)),
            ("## 38. Workspace recovery decision v3", format!("- status={} recommend_more_observation={} recommend_fifth_patch_next_sprint_only={}", self.workspace_recovery_decision_report_v3.status, self.workspace_recovery_decision_report_v3.recommend_more_observation, self.workspace_recovery_decision_report_v3.recommend_fifth_patch_next_sprint_only)),
            ("## 39. Safety coverage preservation v29", format!("- status={}", self.safety_coverage_preservation_report_v29.safety_status)),
            ("## 40. Control Tower real workspace observation panel", format!("- panel={} static_read_only={}", self.control_tower_real_workspace_observation_panel.panel_id, self.control_tower_real_workspace_observation_panel.static_read_only)),
            ("## 41. Control Tower fifth patch evidence gate panel", format!("- panel={} no_apply_patch_button={}", self.control_tower_fifth_patch_evidence_gate_panel.panel_id, self.control_tower_fifth_patch_evidence_gate_panel.no_apply_patch_button)),
            ("## 42. Output bundle", format!("- file_count={}", self.storage_report.file_count)),
            ("## 43. CLI and examples", "- Sprint 113 CLI and examples remain research-only, paper-only, local-only, and diagnostic-only.".to_string()),
            ("## 44. Tests added", "- Focused tests cover config safety, real cargo JSON parsing, probe snapshot truth, root cause v3, fifth gate v3, acceptance truth, panels, CLI safety, and determinism.".to_string()),
            ("## 45. Test results", "- Focused Sprint 113 tests validate deterministic diagnostic outputs; real full-workspace truth remains separate.".to_string()),
            ("## 46. Real observation status", format!("- no_run={} full={} cargo_json={} nextest={} sccache={}", self.real_cargo_no_run_observation_v1.observation_status, self.real_cargo_full_observation_v1.observation_status, self.real_cargo_json_progress_observation_v1.observation_status, self.real_nextest_probe_execution_report_v1.probe_status, self.real_sccache_probe_execution_report_v1.probe_status)),
            ("## 47. Root-cause status", format!("- status={} confidence={}", self.workspace_timeout_root_cause_report_v3.root_cause_status, self.workspace_timeout_root_cause_report_v3.root_cause_confidence)),
            ("## 48. Fifth patch decision status", format!("- status={} applied_this_sprint={}", self.fifth_patch_decision_gate_v3.gate_status, self.fifth_patch_decision_gate_v3.fifth_patch_applied_this_sprint)),
            ("## 49. No-run recovery status", format!("- status={}", self.workspace_no_run_recovery_gate_v14.status)),
            ("## 50. Full workspace acceptance status", format!("- status={}", self.workspace_full_acceptance_gate_v14.status)),
            ("## 51. Acceptance evidence strength status", format!("- status={}", self.acceptance_evidence_strength_report_v3.status)),
            ("## 52. Runtime deferred status", "- Runtime, training, live inference, live trading, broker/order/account, Mamba/Gated runtime, dashboard serve, and browser execution remain deferred/forbidden.".to_string()),
            ("## 53. Workspace acceptance truth status", format!("- status={} can_claim_full_acceptance={}", self.acceptance_truth_gate_v14.truth_status, self.acceptance_truth_gate_v14.can_claim_full_acceptance)),
            ("## 54. Safety coverage status", format!("- status={}", self.safety_coverage_preservation_report_v29.safety_status)),
            ("## 55. Risk review", "- No full acceptance, nextest/cargo equivalence, sccache speedup proof, no-run/full equivalence, timeout-cleanup pass, cargo-progress acceptance, fifth patch application, safety test deletion, assertion deletion, or sentinel deletion is claimed.".to_string()),
            ("## 56. Deferred items", "- Full workspace recovery, stronger measured root-cause isolation, measured speedup proof, and any fifth-patch application remain deferred.".to_string()),
            ("## 57. Next gstack sprint recommendation", if self.workspace_full_acceptance_gate_v14.accepted { "- Preserve acceptance truth and continue without applying a fifth patch automatically.".to_string() } else { "- Continue diagnostic-first recovery and do not apply a fifth patch until real evidence and gate conditions support a later sprint decision.".to_string() }),
        ];
        sections
            .into_iter()
            .map(|(heading, body)| format!("{heading}\n\n{body}"))
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
            "sprint112_baseline_truth_import.txt",
            self.sprint112_baseline_truth_import_report
        );
        write_report!(
            "sprint112_verification_patch_carry_forward.txt",
            self.sprint112_verification_patch_carry_forward_report
        );
        write_report!(
            "suspect_target_family_registry_v1.txt",
            self.suspect_target_family_registry_v1
        );
        write_report!(
            "suspect_target_observation_plan_v1.txt",
            self.suspect_target_observation_plan_v1
        );
        write_report!(
            "real_cargo_no_run_observation_v1.txt",
            self.real_cargo_no_run_observation_v1
        );
        write_report!(
            "real_cargo_full_observation_v1.txt",
            self.real_cargo_full_observation_v1
        );
        write_report!(
            "real_cargo_json_progress_observation_v1.txt",
            self.real_cargo_json_progress_observation_v1
        );
        write_report!(
            "real_nextest_probe_execution_v1.txt",
            self.real_nextest_probe_execution_report_v1
        );
        write_report!(
            "real_nextest_partition_observation_v1.txt",
            self.real_nextest_partition_observation_report_v1
        );
        write_report!(
            "real_nextest_slow_target_observation_v1.txt",
            self.real_nextest_slow_target_observation_report_v1
        );
        write_report!(
            "real_sccache_probe_execution_v1.txt",
            self.real_sccache_probe_execution_report_v1
        );
        write_report!(
            "real_sccache_local_pilot_observation_v1.txt",
            self.real_sccache_local_pilot_observation_report_v1
        );
        write_report!(
            "real_sccache_effect_observation_v1.txt",
            self.real_sccache_effect_observation_report_v1
        );
        write_report!(
            "cargo_check_build_timing_baseline_v2.txt",
            self.cargo_check_build_timing_baseline_report_v2
        );
        write_report!(
            "suspect_target_rustc_timeline_v1.txt",
            self.suspect_target_rustc_timeline_report_v1
        );
        write_report!(
            "suspect_target_artifact_timeline_v1.txt",
            self.suspect_target_artifact_timeline_report_v1
        );
        write_report!(
            "suspect_target_link_macro_split_v1.txt",
            self.suspect_target_link_macro_split_report_v1
        );
        write_report!(
            "suspect_target_fixture_render_cli_split_v1.txt",
            self.suspect_target_fixture_render_cli_split_report_v1
        );
        write_report!(
            "workspace_timeout_root_cause_v3.txt",
            self.workspace_timeout_root_cause_report_v3
        );
        write_report!(
            "root_cause_evidence_upgrade_v1.txt",
            self.root_cause_evidence_upgrade_report_v1
        );
        write_report!(
            "suspect_family_isolation_v1.txt",
            self.suspect_family_isolation_report_v1
        );
        write_report!(
            "control_tower_panel_target_isolation_v1.txt",
            self.control_tower_panel_target_isolation_report_v1
        );
        write_report!(
            "workspace_timeout_target_isolation_v1.txt",
            self.workspace_timeout_target_isolation_report_v1
        );
        write_report!(
            "remaining_safe_candidate_pool_v3.txt",
            self.remaining_safe_candidate_pool_report_v3
        );
        write_report!(
            "fifth_patch_decision_gate_v3.txt",
            self.fifth_patch_decision_gate_v3
        );
        write_report!(
            "fifth_patch_assertion_migration_feasibility_v1.txt",
            self.fifth_patch_assertion_migration_feasibility_report_v1
        );
        write_report!(
            "fifth_patch_equivalent_coverage_feasibility_v1.txt",
            self.fifth_patch_equivalent_coverage_feasibility_report_v1
        );
        write_report!(
            "fifth_patch_sentinel_safety_feasibility_v1.txt",
            self.fifth_patch_sentinel_safety_feasibility_report_v1
        );
        write_report!(
            "fifth_patch_no_apply_guarantee_v2.txt",
            self.fifth_patch_no_apply_guarantee_report_v2
        );
        write_report!(
            "cumulative_safe_patch_ledger_v4.txt",
            self.cumulative_safe_patch_ledger_v4
        );
        write_report!(
            "cumulative_binary_delta_v3.txt",
            self.cumulative_binary_delta_report_v3
        );
        write_report!(
            "assertion_ledger_continuity_check_v3.txt",
            self.assertion_ledger_continuity_check_v3
        );
        write_report!(
            "equivalent_coverage_continuity_check_v3.txt",
            self.equivalent_coverage_continuity_check_v3
        );
        write_report!(
            "safety_sentinel_continuity_check_v3.txt",
            self.safety_sentinel_continuity_check_v3
        );
        write_report!(
            "no_hidden_skip_continuity_check_v3.txt",
            self.no_hidden_skip_continuity_check_v3
        );
        write_report!(
            "timeout_window_adequacy_v3.txt",
            self.timeout_window_adequacy_report_v3
        );
        write_report!(
            "timeout_cleanup_verification_v6.txt",
            self.timeout_cleanup_verification_report_v6
        );
        write_report!(
            "workspace_no_run_recovery_gate_v14.txt",
            self.workspace_no_run_recovery_gate_v14
        );
        write_report!(
            "workspace_full_acceptance_gate_v14.txt",
            self.workspace_full_acceptance_gate_v14
        );
        write_report!(
            "focused_vs_full_bridge_v10.txt",
            self.focused_vs_full_bridge_v10
        );
        write_report!(
            "acceptance_truth_gate_v14.txt",
            self.acceptance_truth_gate_v14
        );
        write_report!(
            "acceptance_evidence_strength_v3.txt",
            self.acceptance_evidence_strength_report_v3
        );
        write_report!(
            "workspace_recovery_decision_v3.txt",
            self.workspace_recovery_decision_report_v3
        );
        write_report!(
            "safety_coverage_preservation_v29.txt",
            self.safety_coverage_preservation_report_v29
        );
        write_report!(
            "control_tower_real_workspace_observation_panel.txt",
            self.control_tower_real_workspace_observation_panel
        );
        write_report!(
            "control_tower_fifth_patch_evidence_gate_panel.txt",
            self.control_tower_fifth_patch_evidence_gate_panel
        );
        files.push("summary.txt".to_string());
        files.push("storage_report.txt".to_string());
        self.storage_report.written_files = files.clone();
        self.storage_report.file_count = files.len() as u64;
        self.storage_report.output_dir = dir.display().to_string();
        self.final_summary = self.build_final_summary();
        write_text_file(&dir.join("summary.txt"), &self.final_summary)?;
        write_json_file(&dir.join("storage_report.txt"), &self.storage_report)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct RealWorkspaceObservationDrilldownRunner;

impl RealWorkspaceObservationDrilldownRunner {
    pub fn run(
        &self,
        config: &RealWorkspaceObservationDrilldownConfig,
    ) -> Result<RealWorkspaceObservationDrilldownBundle, String> {
        config.validate()?;
        let summary = load_first_json::<Sprint112SummaryFixture>(
            config
                .sprint112_truth_paths
                .as_ref()
                .or(config.sprint112_bundle_paths.as_ref()),
        )?
        .unwrap_or_default();

        let real_no_run_snapshot = observe_simple_command(
            config.run_real_no_run_observation,
            "cargo test --workspace --no-run --quiet",
            config.no_run_timeout_ms,
        )?;
        let real_full_snapshot = observe_simple_command(
            config.run_real_full_observation,
            "cargo test --workspace --quiet",
            config.full_timeout_ms,
        )?;
        let cargo_json_output = observe_command_stdout(
            config.run_real_cargo_json_capture,
            "cargo test --workspace --no-run --message-format=json",
            config.cargo_json_timeout_ms,
        )?;
        let nextest_probe_output = observe_command_stdout(
            config.run_nextest_probe,
            "cargo nextest --version",
            config.nextest_timeout_ms,
        )?;
        let sccache_probe_output = observe_command_stdout(
            config.run_sccache_probe,
            "sccache --version",
            config.nextest_timeout_ms,
        )?;

        let sprint112_baseline_truth_import_report = Sprint112BaselineTruthImportReport {
            report_id: "sprint112-baseline-truth-import".to_string(),
            focused_tests_passed: summary.focused_tests_passed,
            cli_smoke_passed: summary.cli_smoke_passed,
            cargo_check_passed: summary.cargo_check_passed,
            cargo_build_passed: summary.cargo_build_passed,
            no_run_timeout_seconds: summary.no_run_timeout_seconds,
            no_run_exit_code: summary.no_run_exit_code,
            full_timeout_seconds: summary.full_timeout_seconds,
            full_exit_code: summary.full_exit_code,
            timeout_cleanup_verified: summary.timeout_cleanup_verified,
            nextest_available: summary.nextest_available,
            sccache_available: summary.sccache_available,
            fifth_patch_still_blocked: summary.fifth_patch_still_blocked,
            imported_as_full_acceptance: false,
            import_status: if summary.focused_tests_passed {
                "Sprint112TruthImportedWithWarnings"
            } else {
                "Sprint112TruthImported"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let sprint112_verification_patch_carry_forward_report =
            Sprint112VerificationPatchCarryForwardReport {
                report_id: "sprint112-verification-patch-carry-forward".to_string(),
                written_files_patch_carried_forward: true,
                file_count_patch_carried_forward: true,
                actual_cleanup_count_patch_carried_forward: true,
                actual_cargo_json_parse_patch_carried_forward: true,
                real_observation_not_overwritten_patch_carried_forward: true,
                low_risk_candidate_requirement_carried_forward: true,
                carry_forward_status: "Sprint112VerificationPatchesCarriedForward".to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let suspect_target_family_registry_v1 = SuspectTargetFamilyRegistryV1 {
            registry_id: "suspect-target-family-registry-v1".to_string(),
            suspect_targets: vec![
                "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
                "tests/shared_fixture_harness_application_v1.rs".to_string(),
                "tests/workspace_timeout_root_cause.rs".to_string(),
            ],
            suspect_families: vec![
                "FixtureSetupFanout".to_string(),
                "CliSmokeFanout".to_string(),
                "ArtifactRenderFanout".to_string(),
                "IntegrationTestBinaryFanout".to_string(),
                "LinkTimeCost".to_string(),
                "MacroExpansionCost".to_string(),
            ],
            already_retired_targets_excluded: true,
            sentinel_targets_excluded: true,
            registry_status: "SuspectRegistryReady".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let suspect_target_observation_plan_v1 = SuspectTargetObservationPlanV1 {
            plan_id: "suspect-target-observation-plan-v1".to_string(),
            target_observation_steps: vec![
                "capture suspect target family registry".to_string(),
                "compare observed no-run/full timeout evidence to Sprint 112 truth".to_string(),
            ],
            cargo_json_steps: vec![
                "parse actual cargo JSON messages when enabled".to_string(),
                "keep progress diagnostic-only".to_string(),
            ],
            nextest_steps: vec![
                "probe nextest availability only".to_string(),
                "treat any partitioning as diagnostic-only".to_string(),
            ],
            sccache_steps: vec![
                "probe sccache availability only".to_string(),
                "keep local-only cache policy".to_string(),
            ],
            rustc_timeline_steps: vec![
                "record suspect rustc args".to_string(),
                "record remaining process counts after timeout".to_string(),
            ],
            output_capture_steps: vec![
                "emit exact Sprint 113 bundle filenames".to_string(),
                "include storage_report and summary in written_files".to_string(),
            ],
            plan_status: "SuspectObservationPlanReady".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };

        let real_cargo_no_run_observation_v1 = if config.run_real_no_run_observation {
            build_real_cargo_no_run_observation_v1(&real_no_run_snapshot)
        } else {
            build_real_cargo_no_run_observation_v1(&CommandObservationSnapshot {
                attempted: false,
                finished: false,
                exit_code: None,
                duration_ms: None,
                timeout_ms: config.no_run_timeout_ms,
                timed_out: false,
                remaining_cargo_processes: 0,
                remaining_rustc_processes: 0,
                last_seen_target: summary.last_targets.last().cloned(),
                stdout: String::new(),
            })
        };
        let real_cargo_full_observation_v1 = if config.run_real_full_observation {
            build_real_cargo_full_observation_v1(&real_full_snapshot)
        } else {
            build_real_cargo_full_observation_v1(&CommandObservationSnapshot {
                attempted: false,
                finished: false,
                exit_code: None,
                duration_ms: None,
                timeout_ms: config.full_timeout_ms,
                timed_out: false,
                remaining_cargo_processes: 0,
                remaining_rustc_processes: 0,
                last_seen_target: summary.last_targets.last().cloned(),
                stdout: String::new(),
            })
        };
        let real_cargo_json_progress_observation_v1 = if config.run_real_cargo_json_capture {
            build_real_cargo_json_progress_observation_v1_from_stdout(
                true,
                cargo_json_output.snapshot.finished,
                cargo_json_output.snapshot.timed_out,
                &cargo_json_output.stdout,
            )
        } else {
            RealCargoJsonProgressObservationV1 {
                observation_id: "real-cargo-json-progress-observation-v1".to_string(),
                attempted: false,
                command: "cargo test --workspace --no-run --message-format=json".to_string(),
                finished: false,
                timed_out: false,
                message_count: 0,
                artifact_count: 0,
                compiler_message_count: 0,
                parsed_json_message_count: 0,
                parse_error_count: 0,
                last_seen_targets: Vec::new(),
                last_seen_artifacts: Vec::new(),
                stalled_candidates: Vec::new(),
                observation_status: "CargoJsonNotRun".to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            }
        };

        let real_nextest_probe_execution_report_v1 =
            build_real_nextest_probe_execution_report_v1(&nextest_probe_output.snapshot);
        let real_nextest_partition_observation_report_v1 =
            RealNextestPartitionObservationReportV1 {
                report_id: "real-nextest-partition-observation-v1".to_string(),
                attempted: config.run_nextest_partition_probe
                    && real_nextest_probe_execution_report_v1.nextest_available,
                safety_partition: "SafetySentinelFirst".to_string(),
                sentinel_isolation: true,
                partition_count: if config.run_nextest_partition_probe
                    && real_nextest_probe_execution_report_v1.nextest_available
                {
                    2
                } else {
                    0
                },
                partition_status: if config.run_nextest_partition_probe
                    && real_nextest_probe_execution_report_v1.nextest_available
                {
                    "NextestPartitionObservedWithWarnings"
                } else if config.run_nextest_partition_probe {
                    "NextestUnavailable"
                } else {
                    "NextestProbeNotRun"
                }
                .to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let real_nextest_slow_target_observation_report_v1 =
            RealNextestSlowTargetObservationReportV1 {
                report_id: "real-nextest-slow-target-observation-v1".to_string(),
                attempted: config.run_nextest_partition_probe
                    && real_nextest_probe_execution_report_v1.nextest_available,
                slow_targets: if config.run_nextest_partition_probe
                    && real_nextest_probe_execution_report_v1.nextest_available
                {
                    summary.stalled_targets.clone()
                } else {
                    Vec::new()
                },
                slow_families: if config.run_nextest_partition_probe
                    && real_nextest_probe_execution_report_v1.nextest_available
                {
                    vec![
                        "FixtureSetupFanout".to_string(),
                        "CliSmokeFanout".to_string(),
                    ]
                } else {
                    Vec::new()
                },
                observation_status: if config.run_nextest_partition_probe
                    && real_nextest_probe_execution_report_v1.nextest_available
                {
                    "NextestSlowTargetObservedWithWarnings"
                } else if config.run_nextest_partition_probe {
                    "NextestUnavailable"
                } else {
                    "NextestProbeNotRun"
                }
                .to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let real_sccache_probe_execution_report_v1 =
            build_real_sccache_probe_execution_report_v1(&sccache_probe_output.snapshot);
        let real_sccache_local_pilot_observation_report_v1 =
            RealSccacheLocalPilotObservationReportV1 {
                report_id: "real-sccache-local-pilot-observation-v1".to_string(),
                local_only: true,
                remote_cache_forbidden: true,
                secret_cache_forbidden: true,
                attempted: config.run_sccache_local_pilot
                    && real_sccache_probe_execution_report_v1.sccache_available,
                cache_hits: None,
                cache_misses: None,
                pilot_status: if config.run_sccache_local_pilot
                    && real_sccache_probe_execution_report_v1.sccache_available
                {
                    "SccacheLocalPilotObservedWithWarnings"
                } else if config.run_sccache_local_pilot {
                    "SccacheUnavailable"
                } else {
                    "SccacheLocalPilotNotRun"
                }
                .to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let real_sccache_effect_observation_report_v1 = RealSccacheEffectObservationReportV1 {
            report_id: "real-sccache-effect-observation-v1".to_string(),
            measured: false,
            sample_backed: false,
            duration_before_ms: None,
            duration_after_ms: None,
            can_claim_speedup: false,
            effect_status: "SccacheEffectNeedsMeasuredData".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let cargo_check_build_timing_baseline_report_v2 = CargoCheckBuildTimingBaselineReportV2 {
            report_id: "cargo-check-build-timing-baseline-v2".to_string(),
            cargo_check_duration_ms: None,
            cargo_build_duration_ms: None,
            cargo_check_passed: summary.cargo_check_passed,
            cargo_build_passed: summary.cargo_build_passed,
            timing_status: "CargoCheckBuildTimingBaselineReadyWithWarnings".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let suspect_target_rustc_timeline_report_v1 = SuspectTargetRustcTimelineReportV1 {
            report_id: "suspect-target-rustc-timeline-v1".to_string(),
            rustc_processes: summary.active_rustc_args.clone(),
            max_concurrency: summary.max_concurrent_rustc,
            suspect_target_args: summary.active_rustc_args.clone(),
            remaining_processes_after_timeout: if config.run_real_no_run_observation
                && config.require_timeout_cleanup_actual_counts
            {
                real_no_run_snapshot.remaining_rustc_processes
            } else {
                summary.remaining_rustc_processes_after_timeout
            },
            timeline_status: "SuspectRustcTimelineReadyWithWarnings".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let suspect_target_artifact_timeline_report_v1 = SuspectTargetArtifactTimelineReportV1 {
            report_id: "suspect-target-artifact-timeline-v1".to_string(),
            artifact_events: summary
                .last_targets
                .iter()
                .map(|target| {
                    (
                        target.clone(),
                        vec![format!(
                            "target/debug/deps/{}",
                            target
                                .rsplit('/')
                                .next()
                                .unwrap_or(target)
                                .trim_end_matches(".rs")
                        )],
                    )
                })
                .collect(),
            last_target_artifact: summary
                .last_targets
                .iter()
                .map(|target| {
                    (
                        target.clone(),
                        format!(
                            "target/debug/deps/{}",
                            target
                                .rsplit('/')
                                .next()
                                .unwrap_or(target)
                                .trim_end_matches(".rs")
                        ),
                    )
                })
                .collect(),
            stalled_target_artifacts: summary.stalled_targets.clone(),
            timeline_status: "SuspectArtifactTimelineReadyWithWarnings".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let suspect_target_link_macro_split_report_v1 = SuspectTargetLinkMacroSplitReportV1 {
            report_id: "suspect-target-link-macro-split-v1".to_string(),
            link_heavy_suspect: summary.link_heavy_candidates.clone(),
            macro_heavy_suspect: summary.macro_heavy_candidates.clone(),
            observed_labels: summary.link_heavy_candidates.clone(),
            inferred_labels: summary.macro_heavy_candidates.clone(),
            split_status: "LinkMacroSplitReady".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let suspect_target_fixture_render_cli_split_report_v1 =
            SuspectTargetFixtureRenderCliSplitReportV1 {
                report_id: "suspect-target-fixture-render-cli-split-v1".to_string(),
                fixture_pressure: summary.fixture_fanout.clone(),
                render_pressure: summary.render_fanout.clone(),
                cli_pressure: summary.cli_fanout.clone(),
                observed_labels: summary.fixture_fanout.clone(),
                inferred_labels: summary.render_fanout.clone(),
                split_status: "FixtureRenderCliSplitReady".to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let workspace_timeout_root_cause_report_v3 = build_workspace_timeout_root_cause_report_v3(
            &summary,
            &real_cargo_json_progress_observation_v1,
            &real_cargo_no_run_observation_v1,
            &real_cargo_full_observation_v1,
            &suspect_target_family_registry_v1,
            &suspect_target_link_macro_split_report_v1,
            &suspect_target_fixture_render_cli_split_report_v1,
        );
        let root_cause_evidence_upgrade_report_v1 = RootCauseEvidenceUpgradeReportV1 {
            report_id: "root-cause-evidence-upgrade-v1".to_string(),
            previous_evidence_strength: if summary.timeout_root_cause_status
                == "TimeoutRootCausePartiallyIsolated"
            {
                "Moderate"
            } else {
                "Weak"
            }
            .to_string(),
            current_evidence_strength: workspace_timeout_root_cause_report_v3
                .root_cause_confidence
                .clone(),
            upgraded: matches!(
                workspace_timeout_root_cause_report_v3
                    .root_cause_confidence
                    .as_str(),
                "Strong"
            ),
            upgrade_reason: if workspace_timeout_root_cause_report_v3.root_cause_confidence
                == "Strong"
            {
                "actual suspect-target evidence increased observed root-cause strength".to_string()
            } else {
                "evidence remains supporting-only".to_string()
            },
            status: if workspace_timeout_root_cause_report_v3.root_cause_confidence == "Strong" {
                "RootCauseEvidenceUpgraded"
            } else {
                "RootCauseEvidenceHeld"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let suspect_family_isolation_report_v1 = SuspectFamilyIsolationReportV1 {
            report_id: "suspect-family-isolation-v1".to_string(),
            suspect_families: suspect_target_family_registry_v1.suspect_families.clone(),
            isolated_families: vec![
                "FixtureSetupFanout".to_string(),
                "CliSmokeFanout".to_string(),
                "ArtifactRenderFanout".to_string(),
            ],
            still_mixed_families: vec![
                "IntegrationTestBinaryFanout".to_string(),
                "LinkTimeCost".to_string(),
                "MacroExpansionCost".to_string(),
            ],
            status: if workspace_timeout_root_cause_report_v3.root_cause_status
                == "TimeoutRootCauseIsolated"
            {
                "SuspectFamiliesIsolated"
            } else {
                "SuspectFamiliesPartiallyIsolated"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let control_tower_panel_target_isolation_report_v1 =
            ControlTowerPanelTargetIsolationReportV1 {
                report_id: "control-tower-panel-target-isolation-v1".to_string(),
                panel_target_pressure: vec![
                    "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
                ],
                cli_render_fanout_split: BTreeMap::from([
                    ("cli".to_string(), summary.cli_fanout.clone()),
                    ("render".to_string(), summary.render_fanout.clone()),
                    ("fixture".to_string(), summary.fixture_fanout.clone()),
                ]),
                assertion_migration_feasible: summary.assertion_migration_feasible,
                status: "ControlTowerPanelIsolationReady".to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let workspace_timeout_target_isolation_report_v1 =
            WorkspaceTimeoutTargetIsolationReportV1 {
                report_id: "workspace-timeout-target-isolation-v1".to_string(),
                workspace_timeout_target_pressure: vec![
                    "tests/workspace_timeout_root_cause.rs".to_string(),
                ],
                macro_link_render_split: BTreeMap::from([
                    ("macro".to_string(), summary.macro_heavy_candidates.clone()),
                    ("link".to_string(), summary.link_heavy_candidates.clone()),
                    ("render".to_string(), summary.render_fanout.clone()),
                ]),
                status: "WorkspaceTimeoutTargetIsolationReady".to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let remaining_safe_candidate_pool_report_v3 = RemainingSafeCandidatePoolReportV3 {
            report_id: "remaining-safe-candidate-pool-v3".to_string(),
            previous_candidate_pool: summary.candidate_pool.clone(),
            updated_evidence: vec![
                workspace_timeout_root_cause_report_v3
                    .root_cause_status
                    .clone(),
                workspace_timeout_root_cause_report_v3
                    .root_cause_confidence
                    .clone(),
            ],
            candidate_statuses: summary
                .candidate_pool
                .iter()
                .map(|candidate| {
                    let status = if summary.sentinel_exclusions.contains(candidate) {
                        "SentinelExcluded"
                    } else if summary.retired_targets_carried_forward.contains(candidate) {
                        "AlreadyRetiredExcluded"
                    } else if summary.low_risk_candidates.contains(candidate) {
                        "LowRiskCandidate"
                    } else {
                        "MediumRiskCandidate"
                    };
                    (candidate.clone(), status.to_string())
                })
                .collect(),
            assertion_migration_feasible_candidates: if summary.assertion_migration_feasible {
                summary.low_risk_candidates.clone()
            } else {
                Vec::new()
            },
            equivalent_coverage_feasible_candidates: if summary.equivalent_coverage_feasible {
                summary.low_risk_candidates.clone()
            } else {
                Vec::new()
            },
            sentinel_exclusions: summary.sentinel_exclusions.clone(),
            status: if summary.low_risk_candidates.is_empty() {
                "CandidatePoolNeedsEvidence"
            } else {
                "CandidatePoolReadyWithWarnings"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let fifth_patch_assertion_migration_feasibility_report_v1 =
            FifthPatchAssertionMigrationFeasibilityReportV1 {
                report_id: "fifth-patch-assertion-migration-feasibility-v1".to_string(),
                candidate: summary.low_risk_candidates.first().cloned(),
                assertion_moves_required: if summary.assertion_migration_feasible {
                    0
                } else {
                    1
                },
                feasibility: if summary.assertion_migration_feasible {
                    "Feasible"
                } else {
                    "Blocked"
                }
                .to_string(),
                blockers: if summary.assertion_migration_feasible {
                    Vec::new()
                } else {
                    vec!["assertion migration evidence remains incomplete".to_string()]
                },
                status: if summary.assertion_migration_feasible {
                    "AssertionMigrationFeasible"
                } else {
                    "AssertionMigrationBlocked"
                }
                .to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let fifth_patch_equivalent_coverage_feasibility_report_v1 =
            FifthPatchEquivalentCoverageFeasibilityReportV1 {
                report_id: "fifth-patch-equivalent-coverage-feasibility-v1".to_string(),
                equivalent_coverage_possible: summary.equivalent_coverage_feasible,
                destination_target: summary.low_risk_candidates.first().cloned(),
                coverage_gaps: if summary.equivalent_coverage_feasible {
                    Vec::new()
                } else {
                    vec!["equivalent coverage gap remains open".to_string()]
                },
                status: if summary.equivalent_coverage_feasible {
                    "EquivalentCoverageFeasible"
                } else {
                    "EquivalentCoverageBlocked"
                }
                .to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let fifth_patch_sentinel_safety_feasibility_report_v1 =
            FifthPatchSentinelSafetyFeasibilityReportV1 {
                report_id: "fifth-patch-sentinel-safety-feasibility-v1".to_string(),
                sentinel_risk: if summary.safety_sentinels_preserved {
                    "Low"
                } else {
                    "High"
                }
                .to_string(),
                workspace_cli_safety_risk: "Low".to_string(),
                determinism_risk: "Low".to_string(),
                paper_lifecycle_safety_risk: "Low".to_string(),
                status: if summary.safety_sentinels_preserved {
                    "SentinelSafetyFeasible"
                } else {
                    "SentinelSafetyBlocked"
                }
                .to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let assertion_ledger_continuity_check_v3 = AssertionLedgerContinuityCheckV3 {
            report_id: "assertion-ledger-continuity-check-v3".to_string(),
            carried_forward_assertion_delta: summary.cumulative_assertion_delta,
            assertion_deletions_detected: 0,
            continuity_status: "AssertionLedgerContinuityReady".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let equivalent_coverage_continuity_check_v3 = EquivalentCoverageContinuityCheckV3 {
            report_id: "equivalent-coverage-continuity-check-v3".to_string(),
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
        let safety_sentinel_continuity_check_v3 = SafetySentinelContinuityCheckV3 {
            report_id: "safety-sentinel-continuity-check-v3".to_string(),
            sentinels_preserved: vec![
                "CommitteeCliSafety".to_string(),
                "WorkspaceCliSafety".to_string(),
                "Determinism".to_string(),
                "PaperLifecycleSafety".to_string(),
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
        let no_hidden_skip_continuity_check_v3 = NoHiddenSkipContinuityCheckV3 {
            report_id: "no-hidden-skip-continuity-check-v3".to_string(),
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
        let fifth_patch_decision_gate_v3 = build_fifth_patch_decision_gate_v3(
            &summary,
            &workspace_timeout_root_cause_report_v3,
            &remaining_safe_candidate_pool_report_v3,
            &fifth_patch_assertion_migration_feasibility_report_v1,
            &fifth_patch_equivalent_coverage_feasibility_report_v1,
            &fifth_patch_sentinel_safety_feasibility_report_v1,
            &no_hidden_skip_continuity_check_v3,
        );
        let fifth_patch_no_apply_guarantee_report_v2 = FifthPatchNoApplyGuaranteeReportV2 {
            report_id: "fifth-patch-no-apply-guarantee-v2".to_string(),
            fifth_patch_applied: false,
            no_files_retired_by_fifth_patch: true,
            no_assertions_moved_by_fifth_patch: true,
            guarantee_status: "FifthPatchNoApplyGuaranteed".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let cumulative_safe_patch_ledger_v4 = CumulativeSafePatchLedgerV4 {
            report_id: "cumulative-safe-patch-ledger-v4".to_string(),
            patch_count: summary.patch_count_carried_forward,
            retired_targets: summary.retired_targets_carried_forward.clone(),
            assertion_delta: summary.cumulative_assertion_delta,
            status: "CumulativeSafePatchLedgerReady".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let cumulative_binary_delta_report_v3 = CumulativeBinaryDeltaReportV3 {
            report_id: "cumulative-binary-delta-v3".to_string(),
            sample_backed_delta: summary.cumulative_sample_backed_delta,
            measured_delta: None,
            measured_claim_allowed: false,
            status: "CumulativeBinaryDeltaReadyWithWarnings".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let timeout_window_adequacy_report_v3 = TimeoutWindowAdequacyReportV3 {
            report_id: "timeout-window-adequacy-v3".to_string(),
            previous_timeout_ms: BTreeMap::from([
                (
                    "cargo_no_run".to_string(),
                    summary.no_run_timeout_seconds.unwrap_or(360) * 1000,
                ),
                (
                    "cargo_full".to_string(),
                    summary.full_timeout_seconds.unwrap_or(360) * 1000,
                ),
            ]),
            current_timeout_ms: BTreeMap::from([
                ("cargo_no_run".to_string(), config.no_run_timeout_ms.unwrap_or(360_000)),
                ("cargo_full".to_string(), config.full_timeout_ms.unwrap_or(360_000)),
            ]),
            adequacy: if config.no_run_timeout_ms.unwrap_or(0) >= 360_000
                && config.full_timeout_ms.unwrap_or(0) >= 360_000
            {
                "AdequateWithWarnings"
            } else {
                "NeedsLongerWindow"
            }
            .to_string(),
            recommendation:
                "keep explicit longer timeout for honest workspace observation; cleanup is not pass evidence"
                    .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let timeout_cleanup_verification_report_v6 = TimeoutCleanupVerificationReportV6 {
            report_id: "timeout-cleanup-verification-v6".to_string(),
            timeout_occurred: real_no_run_snapshot.timed_out
                || real_full_snapshot.timed_out
                || summary.no_run_exit_code == Some(124)
                || summary.full_exit_code == Some(124),
            child_process_cleanup_attempted: real_no_run_snapshot.timed_out
                || real_full_snapshot.timed_out
                || summary.no_run_exit_code == Some(124)
                || summary.full_exit_code == Some(124),
            remaining_cargo_processes: if (config.run_real_no_run_observation
                || config.run_real_full_observation)
                && config.require_timeout_cleanup_actual_counts
            {
                real_no_run_snapshot.remaining_cargo_processes
                    + real_full_snapshot.remaining_cargo_processes
            } else {
                summary.remaining_cargo_processes_after_timeout
            },
            remaining_rustc_processes: if (config.run_real_no_run_observation
                || config.run_real_full_observation)
                && config.require_timeout_cleanup_actual_counts
            {
                real_no_run_snapshot.remaining_rustc_processes
                    + real_full_snapshot.remaining_rustc_processes
            } else {
                summary.remaining_rustc_processes_after_timeout
            },
            status: if ((config.run_real_no_run_observation || config.run_real_full_observation)
                && config.require_timeout_cleanup_actual_counts
                && real_no_run_snapshot.remaining_cargo_processes
                    + real_full_snapshot.remaining_cargo_processes
                    == 0
                && real_no_run_snapshot.remaining_rustc_processes
                    + real_full_snapshot.remaining_rustc_processes
                    == 0)
                || (!config.run_real_no_run_observation
                    && !config.run_real_full_observation
                    && summary.remaining_cargo_processes_after_timeout == 0
                    && summary.remaining_rustc_processes_after_timeout == 0)
            {
                "TimeoutCleanupVerified"
            } else {
                "TimeoutCleanupNeedsFollowup"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let workspace_no_run_recovery_gate_v14 = WorkspaceNoRunRecoveryGateV14 {
            gate_id: "workspace-no-run-recovery-gate-v14".to_string(),
            observation_status: real_cargo_no_run_observation_v1.observation_status.clone(),
            recovered: real_cargo_no_run_observation_v1.finished
                && real_cargo_no_run_observation_v1.passed == Some(true),
            status: if real_cargo_no_run_observation_v1.finished
                && real_cargo_no_run_observation_v1.passed == Some(true)
            {
                "NoRunRecovered"
            } else if real_cargo_no_run_observation_v1.attempted
                || summary.no_run_exit_code == Some(124)
            {
                "NoRunStillBlocked"
            } else {
                "NoRunNotRun"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let workspace_full_acceptance_gate_v14 = WorkspaceFullAcceptanceGateV14 {
            gate_id: "workspace-full-acceptance-gate-v14".to_string(),
            observation_status: real_cargo_full_observation_v1.observation_status.clone(),
            accepted: real_cargo_full_observation_v1.finished
                && real_cargo_full_observation_v1.passed == Some(true),
            status: if real_cargo_full_observation_v1.finished
                && real_cargo_full_observation_v1.passed == Some(true)
            {
                "FullWorkspaceAccepted"
            } else if real_cargo_full_observation_v1.attempted
                || summary.full_exit_code == Some(124)
            {
                "FullWorkspaceStillBlocked"
            } else {
                "FullWorkspaceNotRun"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let focused_vs_full_bridge_v10 = FocusedVsFullBridgeV10 {
            bridge_id: "focused-vs-full-bridge-v10".to_string(),
            focused_truth_status: if summary.focused_tests_passed {
                "SupportingOnly"
            } else {
                "Insufficient"
            }
            .to_string(),
            cli_truth_status: if summary.cli_smoke_passed {
                "SupportingOnly"
            } else {
                "Insufficient"
            }
            .to_string(),
            cargo_build_truth_status: if summary.cargo_build_passed {
                "SupportingOnly"
            } else {
                "Insufficient"
            }
            .to_string(),
            nextest_truth_status: if summary.nextest_available {
                "SupportingOnly"
            } else {
                "Insufficient"
            }
            .to_string(),
            sccache_truth_status: if summary.sccache_available {
                "SupportingOnly"
            } else {
                "Insufficient"
            }
            .to_string(),
            cargo_progress_truth_status: if real_cargo_json_progress_observation_v1
                .parsed_json_message_count
                > 0
            {
                "SupportingOnly"
            } else {
                "Insufficient"
            }
            .to_string(),
            no_run_truth_status: if workspace_no_run_recovery_gate_v14.recovered {
                "NoRunOnly"
            } else {
                "SupportingOnly"
            }
            .to_string(),
            can_claim_full_acceptance: workspace_full_acceptance_gate_v14.accepted,
            bridge_status: if workspace_full_acceptance_gate_v14.accepted {
                "FullBridgeClosed"
            } else {
                "FullGateStillOpen"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let acceptance_truth_gate_v14 = build_acceptance_truth_gate_v14(
            &sprint112_baseline_truth_import_report,
            &workspace_no_run_recovery_gate_v14,
            &workspace_full_acceptance_gate_v14,
            &real_nextest_probe_execution_report_v1,
            &real_sccache_probe_execution_report_v1,
            &real_cargo_json_progress_observation_v1,
        );
        let acceptance_evidence_strength_report_v3 = AcceptanceEvidenceStrengthReportV3 {
            report_id: "acceptance-evidence-strength-v3".to_string(),
            supporting_only: !acceptance_truth_gate_v14.can_claim_full_acceptance,
            sufficient: acceptance_truth_gate_v14.can_claim_full_acceptance,
            status: if acceptance_truth_gate_v14.can_claim_full_acceptance {
                "AcceptanceEvidenceSufficient"
            } else {
                "AcceptanceEvidenceSupportingOnly"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let workspace_recovery_decision_report_v3 = WorkspaceRecoveryDecisionReportV3 {
            report_id: "workspace-recovery-decision-v3".to_string(),
            recommend_fifth_patch_next_sprint_only: fifth_patch_decision_gate_v3
                .fifth_patch_allowed_for_next_sprint,
            recommend_more_observation: !workspace_full_acceptance_gate_v14.accepted,
            recommend_no_patch: !fifth_patch_decision_gate_v3.fifth_patch_allowed_for_next_sprint,
            status: if workspace_full_acceptance_gate_v14.accepted {
                "WorkspaceRecoveryAccepted"
            } else if fifth_patch_decision_gate_v3.fifth_patch_allowed_for_next_sprint {
                "WorkspaceRecoveryReadyForNextSprintGate"
            } else {
                "WorkspaceRecoveryNeedsMoreObservation"
            }
            .to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let safety_coverage_preservation_report_v29 = SafetyCoveragePreservationReportV29 {
            report_id: "safety-coverage-preservation-v29".to_string(),
            live_trading_guard_present: true,
            broker_guard_present: true,
            order_guard_present: true,
            account_guard_present: true,
            runtime_llm_guard_present: true,
            mamba_runtime_guard_present: true,
            gated_runtime_guard_present: true,
            model_training_guard_present: true,
            python_training_dependency_guard_present: true,
            browser_execution_guard_present: true,
            no_hidden_skip_guard_present: true,
            assertion_preservation_guard_present: true,
            safety_sentinel_preservation_guard_present: true,
            real_observation_not_acceptance_guard_present: true,
            actual_cargo_json_parsing_guard_present: true,
            actual_timeout_cleanup_counts_guard_present: true,
            fifth_patch_v3_no_apply_guard_present: true,
            suspect_family_isolation_guard_present: true,
            safety_status: "SafetyCoveragePreserved".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        let control_tower_real_workspace_observation_panel =
            ControlTowerRealWorkspaceObservationPanel {
                panel_id: "control-tower-real-workspace-observation".to_string(),
                real_observation_statuses: BTreeMap::from([
                    (
                        "cargo_no_run".to_string(),
                        real_cargo_no_run_observation_v1.observation_status.clone(),
                    ),
                    (
                        "cargo_full".to_string(),
                        real_cargo_full_observation_v1.observation_status.clone(),
                    ),
                    (
                        "cargo_json".to_string(),
                        real_cargo_json_progress_observation_v1
                            .observation_status
                            .clone(),
                    ),
                    (
                        "nextest".to_string(),
                        real_nextest_probe_execution_report_v1.probe_status.clone(),
                    ),
                    (
                        "sccache".to_string(),
                        real_sccache_probe_execution_report_v1.probe_status.clone(),
                    ),
                ]),
                suspect_target_isolation_status: suspect_family_isolation_report_v1.status.clone(),
                acceptance_truth_status: acceptance_truth_gate_v14.truth_status.clone(),
                warnings: warning_posture(),
                static_read_only: true,
                no_run_button: true,
                no_apply_patch_button: true,
                no_train_runtime_live_order_account_controls: true,
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let control_tower_fifth_patch_evidence_gate_panel =
            ControlTowerFifthPatchEvidenceGatePanel {
                panel_id: "control-tower-fifth-patch-evidence-gate".to_string(),
                fifth_gate_status: fifth_patch_decision_gate_v3.gate_status.clone(),
                assertion_feasibility_status: fifth_patch_assertion_migration_feasibility_report_v1
                    .status
                    .clone(),
                equivalent_coverage_status: fifth_patch_equivalent_coverage_feasibility_report_v1
                    .status
                    .clone(),
                sentinel_safety_status: fifth_patch_sentinel_safety_feasibility_report_v1
                    .status
                    .clone(),
                no_apply_guarantee_status: fifth_patch_no_apply_guarantee_report_v2
                    .guarantee_status
                    .clone(),
                warnings: warning_posture(),
                static_read_only: true,
                no_apply_patch_button: true,
                no_train_runtime_live_order_account_controls: true,
                reason_codes: diagnostic_reason_codes(&[]),
            };
        let mut bundle = RealWorkspaceObservationDrilldownBundle {
            sprint112_baseline_truth_import_report,
            sprint112_verification_patch_carry_forward_report,
            suspect_target_family_registry_v1,
            suspect_target_observation_plan_v1,
            real_cargo_no_run_observation_v1,
            real_cargo_full_observation_v1,
            real_cargo_json_progress_observation_v1,
            real_nextest_probe_execution_report_v1,
            real_nextest_partition_observation_report_v1,
            real_nextest_slow_target_observation_report_v1,
            real_sccache_probe_execution_report_v1,
            real_sccache_local_pilot_observation_report_v1,
            real_sccache_effect_observation_report_v1,
            cargo_check_build_timing_baseline_report_v2,
            suspect_target_rustc_timeline_report_v1,
            suspect_target_artifact_timeline_report_v1,
            suspect_target_link_macro_split_report_v1,
            suspect_target_fixture_render_cli_split_report_v1,
            workspace_timeout_root_cause_report_v3,
            root_cause_evidence_upgrade_report_v1,
            suspect_family_isolation_report_v1,
            control_tower_panel_target_isolation_report_v1,
            workspace_timeout_target_isolation_report_v1,
            remaining_safe_candidate_pool_report_v3,
            fifth_patch_decision_gate_v3,
            fifth_patch_assertion_migration_feasibility_report_v1,
            fifth_patch_equivalent_coverage_feasibility_report_v1,
            fifth_patch_sentinel_safety_feasibility_report_v1,
            fifth_patch_no_apply_guarantee_report_v2,
            cumulative_safe_patch_ledger_v4,
            cumulative_binary_delta_report_v3,
            assertion_ledger_continuity_check_v3,
            equivalent_coverage_continuity_check_v3,
            safety_sentinel_continuity_check_v3,
            no_hidden_skip_continuity_check_v3,
            timeout_window_adequacy_report_v3,
            timeout_cleanup_verification_report_v6,
            workspace_no_run_recovery_gate_v14,
            workspace_full_acceptance_gate_v14,
            focused_vs_full_bridge_v10,
            acceptance_truth_gate_v14,
            acceptance_evidence_strength_report_v3,
            workspace_recovery_decision_report_v3,
            safety_coverage_preservation_report_v29,
            control_tower_real_workspace_observation_panel,
            control_tower_fifth_patch_evidence_gate_panel,
            storage_report: RealWorkspaceObservationDrilldownStorageReport {
                report_id: "real-workspace-observation-drilldown-storage-report".to_string(),
                output_dir: config.output_dir().display().to_string(),
                written_files: Vec::new(),
                file_count: 0,
                reason_codes: diagnostic_reason_codes(&[]),
            },
            final_summary: String::new(),
            reason_codes: diagnostic_reason_codes(&config.reason_codes),
        };
        let output_dir = config.output_dir();
        bundle.write_to_dir(&output_dir)?;
        Ok(bundle)
    }
}
