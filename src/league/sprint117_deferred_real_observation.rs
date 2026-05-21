use super::sprint116_workspace_timeout_track::CommandObservation;
use crate::ReasonCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

fn render_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|err| err.to_string())
}

fn write_text_file(path: &Path, value: &str) -> Result<(), String> {
    fs::write(path, value).map_err(|err| err.to_string())
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn local_only(path: &str) -> bool {
    let path = path.trim();
    !path.is_empty()
        && !path.contains("://")
        && !path.starts_with("http:")
        && !path.starts_with("https:")
        && !path.starts_with("s3:")
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_output_root() -> String {
    "target/soma_sprint117_deferred_real_observation".to_string()
}

fn default_execution_id() -> String {
    "sprint117-deferred-real-observation".to_string()
}

fn default_timeout_ms() -> Option<u64> {
    Some(420_000)
}

fn stable_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
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

fn warning_posture() -> Vec<String> {
    vec![
        "research-only",
        "paper-only",
        "deferred-real-observation-only",
        "actual-observation-not-fixture",
        "consolidation-paused",
        "fifth-patch-not-applied",
        "no-assertion-movement",
        "no-target-retirement",
        "focused-is-not-full",
        "CLI-smoke-is-not-full",
        "cargo-build-is-not-full",
        "no-run-is-not-full",
        "cargo-json-is-not-acceptance",
        "timeout-cleanup-is-not-pass",
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
pub struct DeferredRealObservationExecutionConfig {
    pub execution_id: String,
    #[serde(default)]
    pub sprint116_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub sprint116_truth_paths: Option<Vec<String>>,
    #[serde(default)]
    pub observation_backlog_paths: Option<Vec<String>>,
    #[serde(default)]
    pub no_run_plan_paths: Option<Vec<String>>,
    #[serde(default)]
    pub full_plan_paths: Option<Vec<String>>,
    #[serde(default)]
    pub cargo_json_plan_paths: Option<Vec<String>>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_false")]
    pub run_real_no_run_observation: bool,
    #[serde(default = "default_false")]
    pub run_real_full_observation: bool,
    #[serde(default = "default_false")]
    pub run_real_cargo_json_observation: bool,
    #[serde(default = "default_timeout_ms")]
    pub no_run_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub full_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub cargo_json_timeout_ms: Option<u64>,
    #[serde(default = "default_true")]
    pub require_actual_vs_carried_forward_separation: bool,
    #[serde(default = "default_true")]
    pub require_timeout_cleanup_actual_counts: bool,
    #[serde(default = "default_true")]
    pub require_cargo_json_actual_parsing: bool,
    #[serde(default = "default_true")]
    pub require_acceptance_truth_gate: bool,
    #[serde(default = "default_true")]
    pub require_consolidation_still_paused: bool,
    #[serde(default = "default_false")]
    pub allow_fifth_patch_application: bool,
    #[serde(default = "default_false")]
    pub allow_assertion_movement: bool,
    #[serde(default = "default_false")]
    pub allow_test_target_retirement: bool,
    #[serde(default = "default_true")]
    pub preserve_runtime_deferred: bool,
    #[serde(default = "default_true")]
    pub preserve_safety_guards: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for DeferredRealObservationExecutionConfig {
    fn default() -> Self {
        Self {
            execution_id: default_execution_id(),
            sprint116_bundle_paths: Some(vec![
                "examples/sprint117_data/sprint116_summary.json".to_string(),
            ]),
            sprint116_truth_paths: Some(vec![
                "examples/sprint117_data/sprint116_summary.json".to_string(),
            ]),
            observation_backlog_paths: Some(vec![
                "examples/sprint117_data/sprint116_summary.json".to_string(),
            ]),
            no_run_plan_paths: None,
            full_plan_paths: None,
            cargo_json_plan_paths: None,
            output_root: default_output_root(),
            run_real_no_run_observation: false,
            run_real_full_observation: false,
            run_real_cargo_json_observation: false,
            no_run_timeout_ms: default_timeout_ms(),
            full_timeout_ms: default_timeout_ms(),
            cargo_json_timeout_ms: default_timeout_ms(),
            require_actual_vs_carried_forward_separation: true,
            require_timeout_cleanup_actual_counts: true,
            require_cargo_json_actual_parsing: true,
            require_acceptance_truth_gate: true,
            require_consolidation_still_paused: true,
            allow_fifth_patch_application: false,
            allow_assertion_movement: false,
            allow_test_target_retirement: false,
            preserve_runtime_deferred: true,
            preserve_safety_guards: true,
            reason_codes: diagnostic_reason_codes(&[]),
        }
    }
}

impl DeferredRealObservationExecutionConfig {
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
        PathBuf::from(&self.output_root).join(&self.execution_id)
    }

    fn validate_paths(paths: &Option<Vec<String>>, field_name: &str) -> Result<(), String> {
        if let Some(paths) = paths {
            for path in paths {
                if !local_only(path) {
                    return Err(format!("{field_name} must use local-only paths: {path}"));
                }
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.execution_id.trim().is_empty() {
            return Err("sprint117 execution_id must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err("sprint117 output_root must be local-only".to_string());
        }
        Self::validate_paths(&self.sprint116_bundle_paths, "sprint116_bundle_paths")?;
        Self::validate_paths(&self.sprint116_truth_paths, "sprint116_truth_paths")?;
        Self::validate_paths(&self.observation_backlog_paths, "observation_backlog_paths")?;
        Self::validate_paths(&self.no_run_plan_paths, "no_run_plan_paths")?;
        Self::validate_paths(&self.full_plan_paths, "full_plan_paths")?;
        Self::validate_paths(&self.cargo_json_plan_paths, "cargo_json_plan_paths")?;
        if !self.require_actual_vs_carried_forward_separation {
            return Err(
                "require_actual_vs_carried_forward_separation must remain true".to_string(),
            );
        }
        if !self.require_timeout_cleanup_actual_counts {
            return Err("require_timeout_cleanup_actual_counts must remain true".to_string());
        }
        if !self.require_cargo_json_actual_parsing {
            return Err("require_cargo_json_actual_parsing must remain true".to_string());
        }
        if !self.require_acceptance_truth_gate {
            return Err("require_acceptance_truth_gate must remain true".to_string());
        }
        if !self.require_consolidation_still_paused {
            return Err("require_consolidation_still_paused must remain true".to_string());
        }
        if self.allow_fifth_patch_application {
            return Err("allow_fifth_patch_application must remain false".to_string());
        }
        if self.allow_assertion_movement {
            return Err("allow_assertion_movement must remain false".to_string());
        }
        if self.allow_test_target_retirement {
            return Err("allow_test_target_retirement must remain false".to_string());
        }
        if !self.preserve_runtime_deferred {
            return Err("preserve_runtime_deferred must remain true".to_string());
        }
        if !self.preserve_safety_guards {
            return Err("preserve_safety_guards must remain true".to_string());
        }
        Ok(())
    }
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sprint116SummaryFixture {
    pub report_id: String,
    pub timeout_track_status: String,
    pub backlog_status: String,
    pub track_progress_status: String,
    pub no_run_status: String,
    pub full_workspace_status: String,
    pub cargo_json_status: String,
    pub acceptance_truth_status: String,
    pub acceptance_evidence_status: String,
    pub consolidation_status: String,
    pub fifth_patch_status: String,
    pub assertion_movement_status: String,
    pub target_retirement_status: String,
    pub safety_status: String,
    pub planned_count: u64,
    pub completed_count: u64,
    pub deferred_count: u64,
    pub remaining_count: u64,
    pub deferred_items: Vec<String>,
    pub completed_items: Vec<String>,
    pub focused_tests_passed: bool,
    pub cli_smoke_passed: bool,
    pub cargo_check_passed: bool,
    pub cargo_build_passed: bool,
    pub cargo_fmt_passed: bool,
    pub cargo_fmt_check_passed: bool,
    pub no_run_timeout_seconds: Option<u64>,
    pub no_run_exit_code: Option<i32>,
    pub full_timeout_seconds: Option<u64>,
    pub full_exit_code: Option<i32>,
    pub timeout_cleanup_verified: bool,
    #[serde(default)]
    pub remaining_cargo_processes_after_timeout: u64,
    #[serde(default)]
    pub remaining_rustc_processes_after_timeout: u64,
}

impl Default for Sprint116SummaryFixture {
    fn default() -> Self {
        Self {
            report_id: "sprint116-summary".to_string(),
            timeout_track_status: "TimeoutTrackActive".to_string(),
            backlog_status: "BacklogReducedWithWarnings".to_string(),
            track_progress_status: "WorkspaceTimeoutTrackNeedsMoreObservation".to_string(),
            no_run_status: "NoRunStillBlocked".to_string(),
            full_workspace_status: "FullWorkspaceStillBlocked".to_string(),
            cargo_json_status: "CargoJsonStillNotRun".to_string(),
            acceptance_truth_status: "AcceptanceTruthReadyWithWarnings".to_string(),
            acceptance_evidence_status: "AcceptanceEvidenceSupportingOnly".to_string(),
            consolidation_status: "ConsolidationStillPaused".to_string(),
            fifth_patch_status: "FifthPatchStillNotApplied".to_string(),
            assertion_movement_status: "AssertionMovementStillForbidden".to_string(),
            target_retirement_status: "TargetRetirementStillForbidden".to_string(),
            safety_status: "SafetyCoveragePreserved".to_string(),
            planned_count: 5,
            completed_count: 2,
            deferred_count: 3,
            remaining_count: 3,
            deferred_items: vec![
                "RealNoRun".to_string(),
                "RealFullWorkspace".to_string(),
                "RealCargoJson".to_string(),
            ],
            completed_items: vec![
                "TimeoutBoundaryObserved".to_string(),
                "TimeoutCleanupConsistencyObserved".to_string(),
            ],
            focused_tests_passed: true,
            cli_smoke_passed: true,
            cargo_check_passed: true,
            cargo_build_passed: true,
            cargo_fmt_passed: true,
            cargo_fmt_check_passed: true,
            no_run_timeout_seconds: Some(420),
            no_run_exit_code: Some(124),
            full_timeout_seconds: Some(420),
            full_exit_code: Some(124),
            timeout_cleanup_verified: true,
            remaining_cargo_processes_after_timeout: 0,
            remaining_rustc_processes_after_timeout: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceTimeoutEvidenceRowV3 {
    pub row_id: String,
    pub evidence_type: String,
    pub evidence_status: String,
    pub supports_acceptance: bool,
    pub supporting_only: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CargoJsonMessageKindCountV2 {
    pub kind: String,
    pub count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CargoJsonArtifactEventV3 {
    pub target: String,
    pub artifact: String,
}

report!(Sprint116BaselineTruthImportReport {
    report_id: String,
    timeout_track_status: String,
    backlog_status: String,
    no_run_status: String,
    full_workspace_status: String,
    cargo_json_status: String,
    acceptance_truth_status: String,
    consolidation_status: String,
    fifth_patch_status: String,
    safety_status: String,
    focused_tests_passed: bool,
    cli_smoke_passed: bool,
    cargo_check_passed: bool,
    cargo_build_passed: bool,
    imported_as_full_acceptance: bool,
    import_status: String
});
report!(Sprint116DeferredBacklogCarryForwardReport {
    report_id: String,
    deferred_items: Vec<String>,
    no_run_deferred: bool,
    full_workspace_deferred: bool,
    cargo_json_deferred: bool,
    completed_items: Vec<String>,
    remaining_count: u64,
    carry_forward_status: String
});
report!(DeferredObservationSelectionReportV1 {
    report_id: String,
    selected_observations: Vec<String>,
    not_selected_observations: Vec<String>,
    selection_status: String
});
report!(DeferredObservationExecutionPlanV1 {
    plan_id: String,
    execution_order: Vec<String>,
    timeout_policy: String,
    cleanup_policy: String,
    evidence_separation_policy: String,
    plan_status: String
});
report!(RealNoRunExecutionReportV18 {
    report_id: String,
    attempted: bool,
    command: String,
    started: bool,
    finished: bool,
    passed: Option<bool>,
    duration_ms: Option<u64>,
    timeout_ms: Option<u64>,
    exit_code: Option<i32>,
    timed_out: bool,
    cleanup_performed: bool,
    remaining_cargo_processes: Option<u64>,
    remaining_rustc_processes: Option<u64>,
    execution_status: String
});
report!(RealFullWorkspaceExecutionReportV18 {
    report_id: String,
    attempted: bool,
    command: String,
    started: bool,
    finished: bool,
    passed: Option<bool>,
    duration_ms: Option<u64>,
    timeout_ms: Option<u64>,
    exit_code: Option<i32>,
    timed_out: bool,
    cleanup_performed: bool,
    remaining_cargo_processes: Option<u64>,
    remaining_rustc_processes: Option<u64>,
    full_workspace_accepted: bool,
    execution_status: String
});
report!(RealCargoJsonExecutionReportV18 {
    report_id: String,
    attempted: bool,
    command: String,
    started: bool,
    finished: bool,
    duration_ms: Option<u64>,
    timeout_ms: Option<u64>,
    exit_code: Option<i32>,
    timed_out: bool,
    raw_line_count: u64,
    parsed_json_message_count: u64,
    parse_error_count: u64,
    malformed_line_count: u64,
    execution_status: String
});
report!(CargoJsonActualParseReportV2 {
    report_id: String,
    parsed_messages_by_kind: Vec<CargoJsonMessageKindCountV2>,
    compiler_artifact_count: u64,
    compiler_message_count: u64,
    build_script_count: u64,
    test_executable_count: u64,
    status: String
});
report!(CargoJsonParseErrorReportV2 {
    report_id: String,
    parse_error_count: u64,
    malformed_line_count: u64,
    error_samples: Vec<String>,
    status: String
});
report!(CargoJsonMalformedLineReportV1 {
    report_id: String,
    malformed_line_count: u64,
    line_classification: Vec<String>,
    status: String
});
report!(CargoJsonArtifactTimelineV3 {
    report_id: String,
    artifact_events: Vec<CargoJsonArtifactEventV3>,
    last_artifact_by_target: BTreeMap<String, String>,
    target_order: Vec<String>,
    status: String
});
report!(CargoJsonTargetProgressReportV2 {
    report_id: String,
    last_seen_targets: Vec<String>,
    completed_targets: Vec<String>,
    stalled_candidates: Vec<String>,
    status: String
});
report!(NoRunExecutionBoundaryReportV2 {
    report_id: String,
    command_boundary: String,
    timeout_boundary_ms: Option<u64>,
    exit_code: Option<i32>,
    status: String
});
report!(FullExecutionBoundaryReportV2 {
    report_id: String,
    command_boundary: String,
    timeout_boundary_ms: Option<u64>,
    exit_code: Option<i32>,
    status: String
});
report!(TimeoutCleanupActualCountsReportV2 {
    report_id: String,
    actual_cargo_counts: BTreeMap<String, u64>,
    actual_rustc_counts: BTreeMap<String, u64>,
    cleanup_timestamp: Option<String>,
    status: String
});
report!(TimeoutCleanupConsistencyReportV2 {
    report_id: String,
    no_run_cleanup: String,
    full_cleanup: String,
    cargo_json_cleanup: String,
    consistency_status: String
});
report!(ObservationFixtureSeparationReportV1 {
    report_id: String,
    actual_observation_fields: Vec<String>,
    carried_forward_fields: Vec<String>,
    fixture_fields: Vec<String>,
    overwritten_actual_count: u64,
    separation_status: String
});
report!(ActualVsCarriedForwardEvidenceReportV1 {
    report_id: String,
    actual_evidence_rows: Vec<String>,
    carried_forward_evidence_rows: Vec<String>,
    supporting_only_labels: Vec<String>,
    status: String
});
report!(ObservationBacklogCompletionReportV2 {
    report_id: String,
    previous_remaining_count: u64,
    completed_count: u64,
    deferred_count: u64,
    blocked_count: u64,
    remaining_count: u64,
    completion_status: String
});
report!(WorkspaceTimeoutEvidenceMatrixV3 {
    report_id: String,
    rows: Vec<WorkspaceTimeoutEvidenceRowV3>,
    supports_acceptance: bool,
    status: String
});
report!(WorkspaceTimeoutRootCauseReportV5 {
    report_id: String,
    previous_root_cause: String,
    actual_no_run_evidence: Vec<String>,
    actual_full_evidence: Vec<String>,
    actual_cargo_json_evidence: Vec<String>,
    parse_quality: String,
    cleanup_counts: String,
    confidence: String,
    status: String
});
report!(WorkspaceTimeoutDiagnosticTrackProgressReportV2 {
    report_id: String,
    track_active: bool,
    backlog_completion: String,
    actual_observations_attempted: u64,
    status: String
});
report!(WorkspaceTimeoutTrackRiskReportV2 {
    report_id: String,
    overclaim_risk: String,
    parse_error_risk: String,
    cleanup_false_positive_risk: String,
    fixture_overwrite_risk: String,
    acceptance_confusion_risk: String,
    status: String
});
report!(ConsolidationTrackStillPausedReportV2 {
    report_id: String,
    paused: bool,
    stopped: bool,
    no_assertion_movement: bool,
    no_target_retirement: bool,
    status: String
});
report!(FifthPatchStillNotAppliedReportV2 {
    report_id: String,
    fifth_patch_applied: bool,
    no_assertions_moved: bool,
    no_targets_retired: bool,
    status: String
});
report!(AssertionMovementStillForbiddenReportV2 {
    report_id: String,
    movement_allowed: bool,
    status: String
});
report!(TargetRetirementStillForbiddenReportV2 {
    report_id: String,
    retirement_allowed: bool,
    status: String
});
report!(WorkspaceNoRunRecoveryGateV18 {
    gate_id: String,
    command: String,
    finished: bool,
    passed: bool,
    timed_out: bool,
    recovered: bool,
    status: String
});
report!(WorkspaceFullAcceptanceGateV18 {
    gate_id: String,
    command: String,
    finished: bool,
    passed: bool,
    accepted: bool,
    status: String
});
report!(FocusedVsFullBridgeV14 {
    bridge_id: String,
    focused_tests_status: String,
    cli_smoke_status: String,
    cargo_build_status: String,
    no_run_status: String,
    cargo_json_status: String,
    cleanup_status: String,
    full_status: String,
    status: String
});
report!(AcceptanceTruthGateV18 {
    gate_id: String,
    focused_truth_status: String,
    cli_truth_status: String,
    cargo_build_truth_status: String,
    no_run_truth_status: String,
    cargo_json_truth_status: String,
    cleanup_truth_status: String,
    full_workspace_truth_status: String,
    can_claim_full_acceptance: bool,
    status: String
});
report!(AcceptanceEvidenceStrengthReportV7 {
    report_id: String,
    full_evidence_sufficient: bool,
    evidence_tiers: Vec<String>,
    strongest_claim: String,
    status: String
});
report!(WorkspaceRecoveryDecisionReportV7 {
    report_id: String,
    recommend_continue_timeout_track: bool,
    recommend_stop_consolidation: bool,
    recommend_no_fifth_patch: bool,
    recommend_real_observation_repeat: bool,
    no_run_recovered: bool,
    full_workspace_accepted: bool,
    status: String
});
report!(TimeoutTrackNextActionQueueV2 {
    queue_id: String,
    next_actions: Vec<String>,
    status: String
});
report!(ControlTowerDeferredObservationExecutionPanel {
    panel_id: String,
    selected_observations: Vec<String>,
    execution_status: String,
    cargo_json_parse_status: String,
    backlog_completion: String,
    acceptance_truth: String,
    warnings: Vec<String>,
    static_read_only: bool,
    no_run_button: bool,
    no_action_button: bool,
    no_train_runtime_live_order_account_controls: bool
});
report!(ControlTowerAcceptanceTruthPanelV18 {
    panel_id: String,
    no_run_gate_status: String,
    full_gate_status: String,
    actual_vs_carried_forward_evidence_status: String,
    supporting_only_evidence: Vec<String>,
    warnings: Vec<String>,
    static_read_only: bool,
    no_action_button: bool,
    no_train_runtime_live_order_account_controls: bool
});
report!(SafetyCoveragePreservationReportV33 {
    report_id: String,
    no_assertion_deletion: bool,
    no_safety_sentinel_deletion: bool,
    no_hidden_skips: bool,
    deferred_real_observation_guard_present: bool,
    actual_vs_carried_forward_guard_present: bool,
    cargo_json_actual_parse_guard_present: bool,
    timeout_cleanup_actual_counts_guard_present: bool,
    consolidation_paused_guard_present: bool,
    fifth_patch_not_applied_guard_present: bool,
    assertion_movement_forbidden_guard_present: bool,
    target_retirement_forbidden_guard_present: bool,
    runtime_deferred: bool,
    training_deferred: bool,
    live_trading_forbidden: bool,
    no_runtime_llm_live_path: bool,
    no_broker_order_account: bool,
    no_mamba_runtime: bool,
    no_gated_runtime: bool,
    no_model_training: bool,
    no_python_training_dependency: bool,
    no_tauri_svelte: bool,
    no_dashboard_serve: bool,
    no_browser_execution: bool,
    no_auto_activation_of_18_live_agents: bool,
    no_silent_confidence_upgrade: bool,
    safety_status: String
});
report!(DeferredRealObservationExecutionStorageReport {
    report_id: String,
    output_dir: String,
    written_files: Vec<String>,
    file_count: u64
});

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeferredRealObservationExecutionBundle {
    pub sprint116_baseline_truth_import_report: Sprint116BaselineTruthImportReport,
    pub sprint116_deferred_backlog_carry_forward_report: Sprint116DeferredBacklogCarryForwardReport,
    pub deferred_observation_selection_report_v1: DeferredObservationSelectionReportV1,
    pub deferred_observation_execution_plan_v1: DeferredObservationExecutionPlanV1,
    pub real_no_run_execution_report_v18: RealNoRunExecutionReportV18,
    pub real_full_workspace_execution_report_v18: RealFullWorkspaceExecutionReportV18,
    pub real_cargo_json_execution_report_v18: RealCargoJsonExecutionReportV18,
    pub cargo_json_actual_parse_report_v2: CargoJsonActualParseReportV2,
    pub cargo_json_parse_error_report_v2: CargoJsonParseErrorReportV2,
    pub cargo_json_malformed_line_report_v1: CargoJsonMalformedLineReportV1,
    pub cargo_json_artifact_timeline_v3: CargoJsonArtifactTimelineV3,
    pub cargo_json_target_progress_report_v2: CargoJsonTargetProgressReportV2,
    pub no_run_execution_boundary_report_v2: NoRunExecutionBoundaryReportV2,
    pub full_execution_boundary_report_v2: FullExecutionBoundaryReportV2,
    pub timeout_cleanup_actual_counts_report_v2: TimeoutCleanupActualCountsReportV2,
    pub timeout_cleanup_consistency_report_v2: TimeoutCleanupConsistencyReportV2,
    pub observation_fixture_separation_report_v1: ObservationFixtureSeparationReportV1,
    pub actual_vs_carried_forward_evidence_report_v1: ActualVsCarriedForwardEvidenceReportV1,
    pub observation_backlog_completion_report_v2: ObservationBacklogCompletionReportV2,
    pub workspace_timeout_evidence_matrix_v3: WorkspaceTimeoutEvidenceMatrixV3,
    pub workspace_timeout_root_cause_report_v5: WorkspaceTimeoutRootCauseReportV5,
    pub workspace_timeout_diagnostic_track_progress_report_v2:
        WorkspaceTimeoutDiagnosticTrackProgressReportV2,
    pub workspace_timeout_track_risk_report_v2: WorkspaceTimeoutTrackRiskReportV2,
    pub consolidation_track_still_paused_report_v2: ConsolidationTrackStillPausedReportV2,
    pub fifth_patch_still_not_applied_report_v2: FifthPatchStillNotAppliedReportV2,
    pub assertion_movement_still_forbidden_report_v2: AssertionMovementStillForbiddenReportV2,
    pub target_retirement_still_forbidden_report_v2: TargetRetirementStillForbiddenReportV2,
    pub workspace_no_run_recovery_gate_v18: WorkspaceNoRunRecoveryGateV18,
    pub workspace_full_acceptance_gate_v18: WorkspaceFullAcceptanceGateV18,
    pub focused_vs_full_bridge_v14: FocusedVsFullBridgeV14,
    pub acceptance_truth_gate_v18: AcceptanceTruthGateV18,
    pub acceptance_evidence_strength_report_v7: AcceptanceEvidenceStrengthReportV7,
    pub workspace_recovery_decision_report_v7: WorkspaceRecoveryDecisionReportV7,
    pub timeout_track_next_action_queue_v2: TimeoutTrackNextActionQueueV2,
    pub safety_coverage_preservation_report_v33: SafetyCoveragePreservationReportV33,
    pub control_tower_deferred_observation_execution_panel:
        ControlTowerDeferredObservationExecutionPanel,
    pub control_tower_acceptance_truth_panel_v18: ControlTowerAcceptanceTruthPanelV18,
    pub storage_report: DeferredRealObservationExecutionStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CargoJsonParseAnalysis {
    raw_line_count: u64,
    parsed_json_message_count: u64,
    parse_error_count: u64,
    malformed_line_count: u64,
    error_samples: Vec<String>,
    malformed_classification: Vec<String>,
    kind_counts: BTreeMap<String, u64>,
    compiler_artifact_count: u64,
    compiler_message_count: u64,
    build_script_count: u64,
    test_executable_count: u64,
    artifact_events: Vec<CargoJsonArtifactEventV3>,
    last_artifact_by_target: BTreeMap<String, String>,
    target_order: Vec<String>,
    last_seen_targets: Vec<String>,
    completed_targets: Vec<String>,
    stalled_candidates: Vec<String>,
}

fn timeout_command() -> Option<&'static str> {
    for candidate in [
        "/opt/homebrew/bin/timeout",
        "/usr/local/bin/gtimeout",
        "timeout",
    ] {
        if candidate == "timeout" || Path::new(candidate).exists() {
            return Some(candidate);
        }
    }
    None
}

fn process_count(pattern: &str) -> u64 {
    match Command::new("pgrep").args(["-fl", pattern]).output() {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| {
                let line = line.trim();
                !line.is_empty() && !line.contains("pgrep")
            })
            .count() as u64,
        _ => 0,
    }
}

fn cleanup_counts_after_observation(
    observation: &Option<CommandObservation>,
) -> Option<(u64, u64)> {
    observation
        .as_ref()
        .filter(|observation| observation.attempted)
        .map(|_| (process_count("cargo"), process_count("rustc")))
}

fn run_timed_command(command: &str, timeout_ms: Option<u64>) -> CommandObservation {
    let timeout_ms = timeout_ms.or(default_timeout_ms());
    let timeout_seconds = timeout_ms.map(|value| ((value + 999) / 1000).max(1));
    let mut shell_command = command.to_string();
    if let (Some(timeout_bin), Some(seconds)) = (timeout_command(), timeout_seconds) {
        shell_command = format!("{timeout_bin} -k 5s {seconds}s {command}");
    }
    let start = Instant::now();
    match Command::new("sh").arg("-c").arg(&shell_command).output() {
        Ok(output) => {
            let exit_code = output.status.code();
            let timed_out = matches!(exit_code, Some(124));
            CommandObservation {
                attempted: true,
                finished: !timed_out,
                passed: if timed_out {
                    None
                } else {
                    Some(output.status.success())
                },
                duration_ms: Some(start.elapsed().as_millis() as u64),
                timeout_ms,
                exit_code,
                timed_out,
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            }
        }
        Err(_) => CommandObservation {
            attempted: false,
            finished: false,
            passed: None,
            duration_ms: None,
            timeout_ms,
            exit_code: None,
            timed_out: false,
            stdout: String::new(),
        },
    }
}

fn analyze_cargo_json_stdout(stdout: &str) -> CargoJsonParseAnalysis {
    let mut analysis = CargoJsonParseAnalysis::default();
    let mut seen = BTreeMap::new();
    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        analysis.raw_line_count += 1;
        match serde_json::from_str::<Value>(line) {
            Ok(value) => {
                analysis.parsed_json_message_count += 1;
                let reason = value
                    .get("reason")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_string();
                *analysis.kind_counts.entry(reason.clone()).or_insert(0) += 1;
                let target = value
                    .get("target")
                    .and_then(|value| value.get("name"))
                    .and_then(Value::as_str)
                    .unwrap_or("unknown-target")
                    .to_string();
                if !seen.contains_key(&target) {
                    seen.insert(target.clone(), seen.len());
                    analysis.target_order.push(target.clone());
                }
                analysis.last_seen_targets.push(target.clone());
                match reason.as_str() {
                    "compiler-artifact" => {
                        analysis.compiler_artifact_count += 1;
                        let executable = value
                            .get("executable")
                            .and_then(Value::as_str)
                            .map(str::to_string);
                        if executable.is_some()
                            || value
                                .get("profile")
                                .and_then(|value| value.get("test"))
                                .and_then(Value::as_bool)
                                .unwrap_or(false)
                        {
                            analysis.test_executable_count += 1;
                            analysis.completed_targets.push(target.clone());
                        }
                        if let Some(filenames) = value.get("filenames").and_then(Value::as_array) {
                            for filename in filenames {
                                if let Some(filename) = filename.as_str() {
                                    analysis.artifact_events.push(CargoJsonArtifactEventV3 {
                                        target: target.clone(),
                                        artifact: filename.to_string(),
                                    });
                                    analysis
                                        .last_artifact_by_target
                                        .insert(target.clone(), filename.to_string());
                                }
                            }
                        } else if let Some(executable) = executable {
                            analysis.artifact_events.push(CargoJsonArtifactEventV3 {
                                target: target.clone(),
                                artifact: executable.clone(),
                            });
                            analysis
                                .last_artifact_by_target
                                .insert(target.clone(), executable);
                        }
                    }
                    "compiler-message" => analysis.compiler_message_count += 1,
                    "build-script-executed" => analysis.build_script_count += 1,
                    _ => {}
                }
            }
            Err(_) => {
                analysis.parse_error_count += 1;
                analysis.malformed_line_count += 1;
                analysis
                    .error_samples
                    .push(line.chars().take(120).collect());
                analysis
                    .malformed_classification
                    .push("MalformedJsonLine".to_string());
            }
        }
    }
    analysis.last_seen_targets = stable_strings(analysis.last_seen_targets);
    analysis.completed_targets = stable_strings(analysis.completed_targets);
    analysis.stalled_candidates = analysis
        .last_seen_targets
        .iter()
        .filter(|target| !analysis.completed_targets.contains(target))
        .cloned()
        .collect();
    analysis.error_samples.truncate(5);
    analysis
}

fn deferred_items(summary: &Sprint116SummaryFixture) -> Vec<String> {
    if summary.deferred_items.is_empty() {
        vec![
            "RealNoRun".to_string(),
            "RealFullWorkspace".to_string(),
            "RealCargoJson".to_string(),
        ]
    } else {
        stable_strings(summary.deferred_items.clone())
    }
}

pub fn build_sprint116_baseline_truth_import_report(
    summary: &Sprint116SummaryFixture,
) -> Sprint116BaselineTruthImportReport {
    Sprint116BaselineTruthImportReport {
        report_id: "sprint116-baseline-truth-import".to_string(),
        timeout_track_status: summary.timeout_track_status.clone(),
        backlog_status: summary.backlog_status.clone(),
        no_run_status: summary.no_run_status.clone(),
        full_workspace_status: summary.full_workspace_status.clone(),
        cargo_json_status: summary.cargo_json_status.clone(),
        acceptance_truth_status: summary.acceptance_truth_status.clone(),
        consolidation_status: summary.consolidation_status.clone(),
        fifth_patch_status: summary.fifth_patch_status.clone(),
        safety_status: summary.safety_status.clone(),
        focused_tests_passed: summary.focused_tests_passed,
        cli_smoke_passed: summary.cli_smoke_passed,
        cargo_check_passed: summary.cargo_check_passed,
        cargo_build_passed: summary.cargo_build_passed,
        imported_as_full_acceptance: false,
        import_status: if summary.focused_tests_passed
            && summary.cli_smoke_passed
            && summary.cargo_check_passed
            && summary.cargo_build_passed
        {
            "Sprint116TruthImportedWithWarnings"
        } else {
            "Sprint116TruthImported"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_sprint116_deferred_backlog_carry_forward_report(
    summary: &Sprint116SummaryFixture,
) -> Sprint116DeferredBacklogCarryForwardReport {
    let deferred_items = deferred_items(summary);
    Sprint116DeferredBacklogCarryForwardReport {
        report_id: "sprint116-deferred-backlog-carry-forward".to_string(),
        no_run_deferred: deferred_items.iter().any(|item| item == "RealNoRun"),
        full_workspace_deferred: deferred_items
            .iter()
            .any(|item| item == "RealFullWorkspace"),
        cargo_json_deferred: deferred_items.iter().any(|item| item == "RealCargoJson"),
        completed_items: stable_strings(summary.completed_items.clone()),
        remaining_count: summary.remaining_count.max(deferred_items.len() as u64),
        carry_forward_status: if deferred_items.is_empty() {
            "DeferredBacklogMissing"
        } else if summary.remaining_count > 0 {
            "DeferredBacklogCarriedForwardWithWarnings"
        } else {
            "DeferredBacklogCarriedForward"
        }
        .to_string(),
        deferred_items,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_deferred_observation_selection_report_v1(
    backlog: &Sprint116DeferredBacklogCarryForwardReport,
) -> DeferredObservationSelectionReportV1 {
    let mut selected_observations = Vec::new();
    for item in ["RealNoRun", "RealFullWorkspace", "RealCargoJson"] {
        if backlog
            .deferred_items
            .iter()
            .any(|candidate| candidate == item)
        {
            selected_observations.push(item.to_string());
        }
    }
    let not_selected_observations = ["RealNoRun", "RealFullWorkspace", "RealCargoJson"]
        .into_iter()
        .filter(|item| {
            !selected_observations
                .iter()
                .any(|selected| selected == item)
        })
        .map(str::to_string)
        .collect();
    DeferredObservationSelectionReportV1 {
        report_id: "deferred-observation-selection-v1".to_string(),
        selection_status: if selected_observations.is_empty() {
            "NoDeferredObservations"
        } else {
            "DeferredObservationsSelected"
        }
        .to_string(),
        selected_observations,
        not_selected_observations,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_deferred_observation_execution_plan_v1(
    selection: &DeferredObservationSelectionReportV1,
    config: &DeferredRealObservationExecutionConfig,
) -> DeferredObservationExecutionPlanV1 {
    DeferredObservationExecutionPlanV1 {
        plan_id: "deferred-observation-execution-plan-v1".to_string(),
        execution_order: vec![
            "RealCargoJson".to_string(),
            "RealNoRun".to_string(),
            "RealFullWorkspace".to_string(),
        ],
        timeout_policy: format!(
            "cargo_json_timeout_ms={:?}; no_run_timeout_ms={:?}; full_timeout_ms={:?}",
            config.cargo_json_timeout_ms, config.no_run_timeout_ms, config.full_timeout_ms
        ),
        cleanup_policy:
            "capture actual cargo/rustc counts after each attempted observation; timeout cleanup is not pass"
                .to_string(),
        evidence_separation_policy:
            "actual-observation-not-fixture; carried-forward evidence stays supporting-only"
                .to_string(),
        plan_status: if selection.selected_observations.is_empty() {
            "DeferredObservationPlanBlocked"
        } else {
            "DeferredObservationPlanReady"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_real_no_run_execution_report_v18(
    observation: Option<&CommandObservation>,
    timeout_ms: Option<u64>,
    cleanup_counts: Option<(u64, u64)>,
) -> RealNoRunExecutionReportV18 {
    match observation {
        Some(observation) if observation.attempted => {
            let passed = if observation.finished && observation.exit_code == Some(0) {
                Some(true)
            } else if observation.timed_out {
                None
            } else {
                Some(false)
            };
            RealNoRunExecutionReportV18 {
                report_id: "real-no-run-execution-v18".to_string(),
                attempted: true,
                command: "cargo test --workspace --no-run --quiet".to_string(),
                started: true,
                finished: observation.finished,
                passed,
                duration_ms: observation.duration_ms,
                timeout_ms,
                exit_code: observation.exit_code,
                timed_out: observation.timed_out,
                cleanup_performed: cleanup_counts.is_some(),
                remaining_cargo_processes: cleanup_counts.map(|counts| counts.0),
                remaining_rustc_processes: cleanup_counts.map(|counts| counts.1),
                execution_status: if observation.timed_out {
                    "RealNoRunTimedOut"
                } else if passed == Some(true) {
                    "RealNoRunCompleted"
                } else {
                    "RealNoRunFailed"
                }
                .to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            }
        }
        _ => RealNoRunExecutionReportV18 {
            report_id: "real-no-run-execution-v18".to_string(),
            attempted: false,
            command: "cargo test --workspace --no-run --quiet".to_string(),
            started: false,
            finished: false,
            passed: None,
            duration_ms: None,
            timeout_ms,
            exit_code: None,
            timed_out: false,
            cleanup_performed: false,
            remaining_cargo_processes: None,
            remaining_rustc_processes: None,
            execution_status: "RealNoRunDeferred".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        },
    }
}

pub fn build_real_full_workspace_execution_report_v18(
    observation: Option<&CommandObservation>,
    timeout_ms: Option<u64>,
    cleanup_counts: Option<(u64, u64)>,
) -> RealFullWorkspaceExecutionReportV18 {
    match observation {
        Some(observation) if observation.attempted => {
            let passed = if observation.finished && observation.exit_code == Some(0) {
                Some(true)
            } else if observation.timed_out {
                None
            } else {
                Some(false)
            };
            let full_workspace_accepted = observation.finished && passed == Some(true);
            RealFullWorkspaceExecutionReportV18 {
                report_id: "real-full-workspace-execution-v18".to_string(),
                attempted: true,
                command: "cargo test --workspace --quiet".to_string(),
                started: true,
                finished: observation.finished,
                passed,
                duration_ms: observation.duration_ms,
                timeout_ms,
                exit_code: observation.exit_code,
                timed_out: observation.timed_out,
                cleanup_performed: cleanup_counts.is_some(),
                remaining_cargo_processes: cleanup_counts.map(|counts| counts.0),
                remaining_rustc_processes: cleanup_counts.map(|counts| counts.1),
                full_workspace_accepted,
                execution_status: if observation.timed_out {
                    "RealFullWorkspaceTimedOut"
                } else if full_workspace_accepted {
                    "FullWorkspaceAccepted"
                } else {
                    "RealFullWorkspaceFailed"
                }
                .to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            }
        }
        _ => RealFullWorkspaceExecutionReportV18 {
            report_id: "real-full-workspace-execution-v18".to_string(),
            attempted: false,
            command: "cargo test --workspace --quiet".to_string(),
            started: false,
            finished: false,
            passed: None,
            duration_ms: None,
            timeout_ms,
            exit_code: None,
            timed_out: false,
            cleanup_performed: false,
            remaining_cargo_processes: None,
            remaining_rustc_processes: None,
            full_workspace_accepted: false,
            execution_status: "RealFullWorkspaceDeferred".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        },
    }
}

pub fn build_real_cargo_json_execution_report_v18(
    observation: Option<&CommandObservation>,
    timeout_ms: Option<u64>,
) -> RealCargoJsonExecutionReportV18 {
    match observation {
        Some(observation) if observation.attempted => {
            let analysis = analyze_cargo_json_stdout(&observation.stdout);
            RealCargoJsonExecutionReportV18 {
                report_id: "real-cargo-json-execution-v18".to_string(),
                attempted: true,
                command: "cargo test --workspace --no-run --message-format=json".to_string(),
                started: true,
                finished: observation.finished,
                duration_ms: observation.duration_ms,
                timeout_ms,
                exit_code: observation.exit_code,
                timed_out: observation.timed_out,
                raw_line_count: analysis.raw_line_count,
                parsed_json_message_count: analysis.parsed_json_message_count,
                parse_error_count: analysis.parse_error_count,
                malformed_line_count: analysis.malformed_line_count,
                execution_status: if observation.timed_out {
                    "RealCargoJsonTimedOut"
                } else if observation.exit_code == Some(0) {
                    "RealCargoJsonParsed"
                } else {
                    "RealCargoJsonFailed"
                }
                .to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            }
        }
        _ => RealCargoJsonExecutionReportV18 {
            report_id: "real-cargo-json-execution-v18".to_string(),
            attempted: false,
            command: "cargo test --workspace --no-run --message-format=json".to_string(),
            started: false,
            finished: false,
            duration_ms: None,
            timeout_ms,
            exit_code: None,
            timed_out: false,
            raw_line_count: 0,
            parsed_json_message_count: 0,
            parse_error_count: 0,
            malformed_line_count: 0,
            execution_status: "RealCargoJsonDeferred".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        },
    }
}

pub fn build_cargo_json_actual_parse_report_v2(
    observation: Option<&CommandObservation>,
) -> CargoJsonActualParseReportV2 {
    match observation {
        Some(observation) if observation.attempted => {
            let analysis = analyze_cargo_json_stdout(&observation.stdout);
            CargoJsonActualParseReportV2 {
                report_id: "cargo-json-actual-parse-v2".to_string(),
                parsed_messages_by_kind: analysis
                    .kind_counts
                    .into_iter()
                    .map(|(kind, count)| CargoJsonMessageKindCountV2 { kind, count })
                    .collect(),
                compiler_artifact_count: analysis.compiler_artifact_count,
                compiler_message_count: analysis.compiler_message_count,
                build_script_count: analysis.build_script_count,
                test_executable_count: analysis.test_executable_count,
                status: if analysis.parse_error_count == 0 {
                    "CargoJsonActualParseReady"
                } else {
                    "CargoJsonActualParseReadyWithWarnings"
                }
                .to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            }
        }
        _ => CargoJsonActualParseReportV2 {
            report_id: "cargo-json-actual-parse-v2".to_string(),
            parsed_messages_by_kind: Vec::new(),
            compiler_artifact_count: 0,
            compiler_message_count: 0,
            build_script_count: 0,
            test_executable_count: 0,
            status: "CargoJsonActualParseDeferred".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        },
    }
}

pub fn build_cargo_json_parse_error_report_v2(
    observation: Option<&CommandObservation>,
) -> CargoJsonParseErrorReportV2 {
    match observation {
        Some(observation) if observation.attempted => {
            let analysis = analyze_cargo_json_stdout(&observation.stdout);
            CargoJsonParseErrorReportV2 {
                report_id: "cargo-json-parse-error-v2".to_string(),
                parse_error_count: analysis.parse_error_count,
                malformed_line_count: analysis.malformed_line_count,
                error_samples: analysis.error_samples,
                status: if analysis.parse_error_count == 0 {
                    "CargoJsonParseErrorsClear"
                } else {
                    "CargoJsonParseErrorsObserved"
                }
                .to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            }
        }
        _ => CargoJsonParseErrorReportV2 {
            report_id: "cargo-json-parse-error-v2".to_string(),
            parse_error_count: 0,
            malformed_line_count: 0,
            error_samples: Vec::new(),
            status: "CargoJsonParseDeferred".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        },
    }
}

pub fn build_cargo_json_malformed_line_report_v1(
    observation: Option<&CommandObservation>,
) -> CargoJsonMalformedLineReportV1 {
    match observation {
        Some(observation) if observation.attempted => {
            let analysis = analyze_cargo_json_stdout(&observation.stdout);
            CargoJsonMalformedLineReportV1 {
                report_id: "cargo-json-malformed-line-v1".to_string(),
                malformed_line_count: analysis.malformed_line_count,
                line_classification: if analysis.malformed_classification.is_empty() {
                    vec!["NoMalformedLines".to_string()]
                } else {
                    analysis.malformed_classification
                },
                status: if analysis.malformed_line_count == 0 {
                    "CargoJsonMalformedLinesClear"
                } else {
                    "CargoJsonMalformedLinesObserved"
                }
                .to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            }
        }
        _ => CargoJsonMalformedLineReportV1 {
            report_id: "cargo-json-malformed-line-v1".to_string(),
            malformed_line_count: 0,
            line_classification: vec!["Deferred".to_string()],
            status: "CargoJsonMalformedLinesDeferred".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        },
    }
}

pub fn build_cargo_json_artifact_timeline_v3(
    observation: Option<&CommandObservation>,
) -> CargoJsonArtifactTimelineV3 {
    match observation {
        Some(observation) if observation.attempted => {
            let analysis = analyze_cargo_json_stdout(&observation.stdout);
            CargoJsonArtifactTimelineV3 {
                report_id: "cargo-json-artifact-timeline-v3".to_string(),
                artifact_events: analysis.artifact_events,
                last_artifact_by_target: analysis.last_artifact_by_target,
                target_order: analysis.target_order,
                status: "CargoJsonArtifactTimelineReady".to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            }
        }
        _ => CargoJsonArtifactTimelineV3 {
            report_id: "cargo-json-artifact-timeline-v3".to_string(),
            artifact_events: Vec::new(),
            last_artifact_by_target: BTreeMap::new(),
            target_order: Vec::new(),
            status: "CargoJsonArtifactTimelineDeferred".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        },
    }
}

pub fn build_cargo_json_target_progress_report_v2(
    observation: Option<&CommandObservation>,
) -> CargoJsonTargetProgressReportV2 {
    match observation {
        Some(observation) if observation.attempted => {
            let analysis = analyze_cargo_json_stdout(&observation.stdout);
            CargoJsonTargetProgressReportV2 {
                report_id: "cargo-json-target-progress-v2".to_string(),
                last_seen_targets: analysis.last_seen_targets,
                completed_targets: analysis.completed_targets,
                stalled_candidates: analysis.stalled_candidates,
                status: "CargoJsonTargetProgressReady".to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            }
        }
        _ => CargoJsonTargetProgressReportV2 {
            report_id: "cargo-json-target-progress-v2".to_string(),
            last_seen_targets: Vec::new(),
            completed_targets: Vec::new(),
            stalled_candidates: Vec::new(),
            status: "CargoJsonTargetProgressDeferred".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        },
    }
}

pub fn build_no_run_execution_boundary_report_v2(
    summary: &Sprint116SummaryFixture,
    report: &RealNoRunExecutionReportV18,
    timeout_ms: Option<u64>,
) -> NoRunExecutionBoundaryReportV2 {
    NoRunExecutionBoundaryReportV2 {
        report_id: "no-run-execution-boundary-v2".to_string(),
        command_boundary: report.command.clone(),
        timeout_boundary_ms: if report.attempted {
            report.timeout_ms
        } else {
            timeout_ms.or(summary
                .no_run_timeout_seconds
                .map(|seconds| seconds * 1_000))
        },
        exit_code: if report.attempted {
            report.exit_code
        } else {
            summary.no_run_exit_code
        },
        status: if report.timed_out {
            "NoRunTimeoutObserved"
        } else if report.attempted {
            "NoRunExecutionBoundaryObserved"
        } else {
            "NoRunExecutionBoundaryDeferred"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_full_execution_boundary_report_v2(
    summary: &Sprint116SummaryFixture,
    report: &RealFullWorkspaceExecutionReportV18,
    timeout_ms: Option<u64>,
) -> FullExecutionBoundaryReportV2 {
    FullExecutionBoundaryReportV2 {
        report_id: "full-execution-boundary-v2".to_string(),
        command_boundary: report.command.clone(),
        timeout_boundary_ms: if report.attempted {
            report.timeout_ms
        } else {
            timeout_ms.or(summary.full_timeout_seconds.map(|seconds| seconds * 1_000))
        },
        exit_code: if report.attempted {
            report.exit_code
        } else {
            summary.full_exit_code
        },
        status: if report.timed_out {
            "FullExecutionTimedOut"
        } else if report.attempted {
            "FullExecutionBoundaryObserved"
        } else {
            "FullExecutionBoundaryDeferred"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_cleanup_actual_counts_report_v2(
    no_run_cleanup: Option<(u64, u64)>,
    full_cleanup: Option<(u64, u64)>,
    cargo_json_cleanup: Option<(u64, u64)>,
) -> TimeoutCleanupActualCountsReportV2 {
    let mut actual_cargo_counts = BTreeMap::new();
    let mut actual_rustc_counts = BTreeMap::new();
    for (label, counts) in [
        ("RealCargoJson", cargo_json_cleanup),
        ("RealFullWorkspace", full_cleanup),
        ("RealNoRun", no_run_cleanup),
    ] {
        if let Some((cargo, rustc)) = counts {
            actual_cargo_counts.insert(label.to_string(), cargo);
            actual_rustc_counts.insert(label.to_string(), rustc);
        }
    }
    TimeoutCleanupActualCountsReportV2 {
        report_id: "timeout-cleanup-actual-counts-v2".to_string(),
        cleanup_timestamp: None,
        status: if actual_cargo_counts.is_empty() {
            "TimeoutCleanupActualCountsDeferred"
        } else {
            "TimeoutCleanupActualCountsReady"
        }
        .to_string(),
        actual_cargo_counts,
        actual_rustc_counts,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_cleanup_consistency_report_v2(
    counts: &TimeoutCleanupActualCountsReportV2,
) -> TimeoutCleanupConsistencyReportV2 {
    let render = |label: &str| {
        format!(
            "cargo={} rustc={}",
            counts.actual_cargo_counts.get(label).copied().unwrap_or(0),
            counts.actual_rustc_counts.get(label).copied().unwrap_or(0)
        )
    };
    let all_zero = counts.actual_cargo_counts.values().all(|value| *value == 0)
        && counts.actual_rustc_counts.values().all(|value| *value == 0);
    TimeoutCleanupConsistencyReportV2 {
        report_id: "timeout-cleanup-consistency-v2".to_string(),
        no_run_cleanup: render("RealNoRun"),
        full_cleanup: render("RealFullWorkspace"),
        cargo_json_cleanup: render("RealCargoJson"),
        consistency_status: if counts.actual_cargo_counts.is_empty() {
            "TimeoutCleanupConsistencyDeferred"
        } else if all_zero {
            "TimeoutCleanupConsistent"
        } else {
            "TimeoutCleanupNeedsManualCheck"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_observation_fixture_separation_report_v1(
    config: &DeferredRealObservationExecutionConfig,
) -> ObservationFixtureSeparationReportV1 {
    let mut fixture_fields = Vec::new();
    if let Some(paths) = &config.sprint116_truth_paths {
        fixture_fields.extend(paths.clone());
    }
    if let Some(paths) = &config.sprint116_bundle_paths {
        fixture_fields.extend(paths.clone());
    }
    ObservationFixtureSeparationReportV1 {
        report_id: "observation-fixture-separation-v1".to_string(),
        actual_observation_fields: vec![
            "real_cargo_json_execution_report_v18".to_string(),
            "real_no_run_execution_report_v18".to_string(),
            "real_full_workspace_execution_report_v18".to_string(),
        ],
        carried_forward_fields: vec![
            "sprint116_baseline_truth_import_report".to_string(),
            "sprint116_deferred_backlog_carry_forward_report".to_string(),
        ],
        fixture_fields: stable_strings(fixture_fields),
        overwritten_actual_count: 0,
        separation_status: "ObservationSeparationReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_actual_vs_carried_forward_evidence_report_v1(
    baseline: &Sprint116BaselineTruthImportReport,
    no_run: &RealNoRunExecutionReportV18,
    full: &RealFullWorkspaceExecutionReportV18,
    cargo_json: &RealCargoJsonExecutionReportV18,
) -> ActualVsCarriedForwardEvidenceReportV1 {
    ActualVsCarriedForwardEvidenceReportV1 {
        report_id: "actual-vs-carried-forward-evidence-v1".to_string(),
        actual_evidence_rows: vec![
            format!("RealCargoJson={}", cargo_json.execution_status),
            format!("RealNoRun={}", no_run.execution_status),
            format!("RealFullWorkspace={}", full.execution_status),
        ],
        carried_forward_evidence_rows: vec![
            format!("Sprint116TimeoutTrack={}", baseline.timeout_track_status),
            format!("Sprint116Backlog={}", baseline.backlog_status),
            format!("Sprint116Acceptance={}", baseline.acceptance_truth_status),
        ],
        supporting_only_labels: vec![
            "focused-tests".to_string(),
            "cli-smoke".to_string(),
            "cargo-build".to_string(),
            "no-run".to_string(),
            "cargo-json".to_string(),
            "timeout-cleanup".to_string(),
        ],
        status: if no_run.attempted || full.attempted || cargo_json.attempted {
            "ActualVsCarriedForwardEvidenceReady"
        } else {
            "ActualVsCarriedForwardEvidenceSupportingOnly"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_observation_backlog_completion_report_v2(
    carry_forward: &Sprint116DeferredBacklogCarryForwardReport,
    no_run: &RealNoRunExecutionReportV18,
    full: &RealFullWorkspaceExecutionReportV18,
    cargo_json: &RealCargoJsonExecutionReportV18,
) -> ObservationBacklogCompletionReportV2 {
    let reports = [
        (no_run.attempted, no_run.finished || no_run.timed_out),
        (full.attempted, full.finished || full.timed_out),
        (
            cargo_json.attempted,
            cargo_json.finished || cargo_json.timed_out,
        ),
    ];
    let completed_count = reports
        .iter()
        .filter(|(attempted, done)| *attempted && *done)
        .count() as u64;
    let blocked_count = reports
        .iter()
        .filter(|(attempted, done)| *attempted && !*done)
        .count() as u64;
    let deferred_count = reports.iter().filter(|(attempted, _)| !*attempted).count() as u64;
    let remaining_count = deferred_count + blocked_count;
    ObservationBacklogCompletionReportV2 {
        report_id: "observation-backlog-completion-v2".to_string(),
        previous_remaining_count: carry_forward.remaining_count,
        completed_count,
        deferred_count,
        blocked_count,
        remaining_count,
        completion_status: if remaining_count == 0 {
            "BacklogCompleted"
        } else if completed_count > 0 {
            "BacklogReducedWithWarnings"
        } else {
            "BacklogStillOpen"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_evidence_matrix_v3(
    summary: &Sprint116SummaryFixture,
    no_run: &RealNoRunExecutionReportV18,
    full: &RealFullWorkspaceExecutionReportV18,
    cargo_json: &RealCargoJsonExecutionReportV18,
    parse: &CargoJsonActualParseReportV2,
    cleanup: &TimeoutCleanupConsistencyReportV2,
    artifact_timeline: &CargoJsonArtifactTimelineV3,
) -> WorkspaceTimeoutEvidenceMatrixV3 {
    let supports_acceptance = full.full_workspace_accepted;
    WorkspaceTimeoutEvidenceMatrixV3 {
        report_id: "workspace-timeout-evidence-matrix-v3".to_string(),
        rows: vec![
            WorkspaceTimeoutEvidenceRowV3 {
                row_id: "RealNoRun".to_string(),
                evidence_type: "RealNoRun".to_string(),
                evidence_status: no_run.execution_status.clone(),
                supports_acceptance: false,
                supporting_only: true,
            },
            WorkspaceTimeoutEvidenceRowV3 {
                row_id: "RealFullWorkspace".to_string(),
                evidence_type: "RealFullWorkspace".to_string(),
                evidence_status: full.execution_status.clone(),
                supports_acceptance,
                supporting_only: !supports_acceptance,
            },
            WorkspaceTimeoutEvidenceRowV3 {
                row_id: "RealCargoJson".to_string(),
                evidence_type: "RealCargoJson".to_string(),
                evidence_status: cargo_json.execution_status.clone(),
                supports_acceptance: false,
                supporting_only: true,
            },
            WorkspaceTimeoutEvidenceRowV3 {
                row_id: "CargoJsonParse".to_string(),
                evidence_type: "CargoJsonParse".to_string(),
                evidence_status: parse.status.clone(),
                supports_acceptance: false,
                supporting_only: true,
            },
            WorkspaceTimeoutEvidenceRowV3 {
                row_id: "TimeoutCleanup".to_string(),
                evidence_type: "TimeoutCleanup".to_string(),
                evidence_status: cleanup.consistency_status.clone(),
                supports_acceptance: false,
                supporting_only: true,
            },
            WorkspaceTimeoutEvidenceRowV3 {
                row_id: "ArtifactTimeline".to_string(),
                evidence_type: "ArtifactTimeline".to_string(),
                evidence_status: artifact_timeline.status.clone(),
                supports_acceptance: false,
                supporting_only: true,
            },
            WorkspaceTimeoutEvidenceRowV3 {
                row_id: "FocusedTests".to_string(),
                evidence_type: "FocusedTests".to_string(),
                evidence_status: if summary.focused_tests_passed {
                    "FocusedSupportingOnly"
                } else {
                    "FocusedMissing"
                }
                .to_string(),
                supports_acceptance: false,
                supporting_only: true,
            },
            WorkspaceTimeoutEvidenceRowV3 {
                row_id: "CliSmoke".to_string(),
                evidence_type: "CliSmoke".to_string(),
                evidence_status: if summary.cli_smoke_passed {
                    "CliSupportingOnly"
                } else {
                    "CliMissing"
                }
                .to_string(),
                supports_acceptance: false,
                supporting_only: true,
            },
            WorkspaceTimeoutEvidenceRowV3 {
                row_id: "CargoBuild".to_string(),
                evidence_type: "CargoBuild".to_string(),
                evidence_status: if summary.cargo_build_passed {
                    "CargoBuildSupportingOnly"
                } else {
                    "CargoBuildMissing"
                }
                .to_string(),
                supports_acceptance: false,
                supporting_only: true,
            },
        ],
        supports_acceptance,
        status: if supports_acceptance {
            "WorkspaceTimeoutEvidenceMatrixReady"
        } else {
            "WorkspaceTimeoutEvidenceMatrixSupportingOnly"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_root_cause_report_v5(
    summary: &Sprint116SummaryFixture,
    no_run: &RealNoRunExecutionReportV18,
    full: &RealFullWorkspaceExecutionReportV18,
    cargo_json: &RealCargoJsonExecutionReportV18,
    parse: &CargoJsonActualParseReportV2,
    cleanup: &TimeoutCleanupActualCountsReportV2,
) -> WorkspaceTimeoutRootCauseReportV5 {
    WorkspaceTimeoutRootCauseReportV5 {
        report_id: "workspace-timeout-root-cause-v5".to_string(),
        previous_root_cause: format!(
            "Sprint116 left real no-run/full/cargo-json deferred with acceptance still supporting-only ({})",
            summary.track_progress_status
        ),
        actual_no_run_evidence: vec![format!(
            "attempted={} finished={} timed_out={} status={}",
            no_run.attempted, no_run.finished, no_run.timed_out, no_run.execution_status
        )],
        actual_full_evidence: vec![format!(
            "attempted={} finished={} timed_out={} accepted={} status={}",
            full.attempted,
            full.finished,
            full.timed_out,
            full.full_workspace_accepted,
            full.execution_status
        )],
        actual_cargo_json_evidence: vec![format!(
            "attempted={} parsed={} errors={} malformed={} status={}",
            cargo_json.attempted,
            cargo_json.parsed_json_message_count,
            cargo_json.parse_error_count,
            cargo_json.malformed_line_count,
            cargo_json.execution_status
        )],
        parse_quality: parse.status.clone(),
        cleanup_counts: cleanup.status.clone(),
        confidence: if no_run.attempted || full.attempted || cargo_json.attempted {
            "ConservativeEvidenceBacked"
        } else {
            "ConservativeCarryForwardOnly"
        }
        .to_string(),
        status: "WorkspaceTimeoutRootCauseReadyV5".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_diagnostic_track_progress_report_v2(
    summary: &Sprint116SummaryFixture,
    backlog: &ObservationBacklogCompletionReportV2,
    no_run: &RealNoRunExecutionReportV18,
    full: &RealFullWorkspaceExecutionReportV18,
    cargo_json: &RealCargoJsonExecutionReportV18,
) -> WorkspaceTimeoutDiagnosticTrackProgressReportV2 {
    WorkspaceTimeoutDiagnosticTrackProgressReportV2 {
        report_id: "workspace-timeout-diagnostic-track-progress-v2".to_string(),
        track_active: summary.timeout_track_status == "TimeoutTrackActive",
        backlog_completion: backlog.completion_status.clone(),
        actual_observations_attempted: [no_run.attempted, full.attempted, cargo_json.attempted]
            .into_iter()
            .filter(|value| *value)
            .count() as u64,
        status: "WorkspaceTimeoutDiagnosticTrackProgressReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_track_risk_report_v2() -> WorkspaceTimeoutTrackRiskReportV2 {
    WorkspaceTimeoutTrackRiskReportV2 {
        report_id: "workspace-timeout-track-risk-v2".to_string(),
        overclaim_risk:
            "High if deferred-real-observation execution is upgraded to acceptance without a real full pass"
                .to_string(),
        parse_error_risk: "Moderate when cargo JSON emits malformed lines or partial output"
            .to_string(),
        cleanup_false_positive_risk:
            "Moderate if cleanup counts are mistaken for test pass".to_string(),
        fixture_overwrite_risk:
            "Low when actual-observation-not-fixture separation is preserved".to_string(),
        acceptance_confusion_risk:
            "High unless cargo test --workspace --quiet really finishes and passes".to_string(),
        status: "WorkspaceTimeoutTrackRiskReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_consolidation_track_still_paused_report_v2(
    baseline: &Sprint116BaselineTruthImportReport,
) -> ConsolidationTrackStillPausedReportV2 {
    ConsolidationTrackStillPausedReportV2 {
        report_id: "consolidation-track-still-paused-v2".to_string(),
        paused: baseline.consolidation_status == "ConsolidationStillPaused",
        stopped: true,
        no_assertion_movement: true,
        no_target_retirement: true,
        status: "ConsolidationStillPaused".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_fifth_patch_still_not_applied_report_v2() -> FifthPatchStillNotAppliedReportV2 {
    FifthPatchStillNotAppliedReportV2 {
        report_id: "fifth-patch-still-not-applied-v2".to_string(),
        fifth_patch_applied: false,
        no_assertions_moved: true,
        no_targets_retired: true,
        status: "FifthPatchStillNotApplied".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_assertion_movement_still_forbidden_report_v2()
-> AssertionMovementStillForbiddenReportV2 {
    AssertionMovementStillForbiddenReportV2 {
        report_id: "assertion-movement-still-forbidden-v2".to_string(),
        movement_allowed: false,
        status: "AssertionMovementStillForbidden".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_target_retirement_still_forbidden_report_v2() -> TargetRetirementStillForbiddenReportV2
{
    TargetRetirementStillForbiddenReportV2 {
        report_id: "target-retirement-still-forbidden-v2".to_string(),
        retirement_allowed: false,
        status: "TargetRetirementStillForbidden".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_no_run_recovery_gate_v18(
    no_run: &RealNoRunExecutionReportV18,
) -> WorkspaceNoRunRecoveryGateV18 {
    let recovered = no_run.finished && no_run.passed == Some(true);
    WorkspaceNoRunRecoveryGateV18 {
        gate_id: "workspace-no-run-recovery-gate-v18".to_string(),
        command: no_run.command.clone(),
        finished: no_run.finished,
        passed: no_run.passed == Some(true),
        timed_out: no_run.timed_out,
        recovered,
        status: if recovered {
            "RealNoRunRecovered"
        } else {
            "NoRunStillBlocked"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_full_acceptance_gate_v18(
    summary: &Sprint116SummaryFixture,
    full: &RealFullWorkspaceExecutionReportV18,
) -> WorkspaceFullAcceptanceGateV18 {
    let accepted = full.full_workspace_accepted;
    WorkspaceFullAcceptanceGateV18 {
        gate_id: "workspace-full-acceptance-gate-v18".to_string(),
        command: full.command.clone(),
        finished: full.finished,
        passed: full.passed == Some(true),
        accepted,
        status: if accepted {
            "FullWorkspaceAccepted"
        } else if full.timed_out || summary.full_workspace_status == "FullWorkspaceStillBlocked" {
            "FullWorkspaceStillBlocked"
        } else {
            "FullWorkspaceNeedsMoreEvidence"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_focused_vs_full_bridge_v14(
    summary: &Sprint116SummaryFixture,
    no_run_gate: &WorkspaceNoRunRecoveryGateV18,
    full_gate: &WorkspaceFullAcceptanceGateV18,
) -> FocusedVsFullBridgeV14 {
    FocusedVsFullBridgeV14 {
        bridge_id: "focused-vs-full-bridge-v14".to_string(),
        focused_tests_status: if summary.focused_tests_passed {
            "focused-is-supporting-only"
        } else {
            "focused-missing"
        }
        .to_string(),
        cli_smoke_status: if summary.cli_smoke_passed {
            "cli-smoke-is-supporting-only"
        } else {
            "cli-smoke-missing"
        }
        .to_string(),
        cargo_build_status: if summary.cargo_build_passed {
            "cargo-build-is-supporting-only"
        } else {
            "cargo-build-missing"
        }
        .to_string(),
        no_run_status: no_run_gate.status.clone(),
        cargo_json_status: "cargo-json-is-supporting-only".to_string(),
        cleanup_status: "timeout-cleanup-is-supporting-only".to_string(),
        full_status: full_gate.status.clone(),
        status: if full_gate.accepted {
            "FocusedVsFullBridgeResolved"
        } else {
            "FocusedVsFullBridgeSupportingOnly"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_acceptance_truth_gate_v18(
    summary: &Sprint116SummaryFixture,
    no_run_gate: &WorkspaceNoRunRecoveryGateV18,
    full_gate: &WorkspaceFullAcceptanceGateV18,
) -> AcceptanceTruthGateV18 {
    let can_claim_full_acceptance = full_gate.accepted;
    AcceptanceTruthGateV18 {
        gate_id: "acceptance-truth-gate-v18".to_string(),
        focused_truth_status: if summary.focused_tests_passed {
            "FocusedSupportingOnly"
        } else {
            "FocusedMissing"
        }
        .to_string(),
        cli_truth_status: if summary.cli_smoke_passed {
            "CliSmokeSupportingOnly"
        } else {
            "CliSmokeMissing"
        }
        .to_string(),
        cargo_build_truth_status: if summary.cargo_build_passed {
            "CargoBuildSupportingOnly"
        } else {
            "CargoBuildMissing"
        }
        .to_string(),
        no_run_truth_status: if no_run_gate.recovered {
            "RealNoRunRecoveredSupportingOnly"
        } else {
            "NoRunSupportingOnly"
        }
        .to_string(),
        cargo_json_truth_status: "CargoJsonSupportingOnly".to_string(),
        cleanup_truth_status: "TimeoutCleanupSupportingOnly".to_string(),
        full_workspace_truth_status: if can_claim_full_acceptance {
            "FullWorkspaceAccepted".to_string()
        } else {
            summary.acceptance_truth_status.clone()
        },
        can_claim_full_acceptance,
        status: if can_claim_full_acceptance {
            "AcceptanceTruthReady"
        } else {
            "AcceptanceTruthReadyWithWarnings"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_acceptance_evidence_strength_report_v7(
    acceptance: &AcceptanceTruthGateV18,
) -> AcceptanceEvidenceStrengthReportV7 {
    let full_evidence_sufficient = acceptance.can_claim_full_acceptance;
    AcceptanceEvidenceStrengthReportV7 {
        report_id: "acceptance-evidence-strength-v7".to_string(),
        full_evidence_sufficient,
        evidence_tiers: vec![
            "focused-tests-supporting-only".to_string(),
            "cli-smoke-supporting-only".to_string(),
            "cargo-build-supporting-only".to_string(),
            "no-run-supporting-only".to_string(),
            "cargo-json-supporting-only".to_string(),
            "timeout-cleanup-supporting-only".to_string(),
            "full-workspace-required-for-acceptance".to_string(),
        ],
        strongest_claim: if full_evidence_sufficient {
            "FullWorkspaceAccepted"
        } else {
            "AcceptanceEvidenceStillSupportingOnly"
        }
        .to_string(),
        status: if full_evidence_sufficient {
            "AcceptanceEvidenceStrongEnough"
        } else {
            "AcceptanceEvidenceSupportingOnly"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_recovery_decision_report_v7(
    backlog: &ObservationBacklogCompletionReportV2,
    no_run_gate: &WorkspaceNoRunRecoveryGateV18,
    full_gate: &WorkspaceFullAcceptanceGateV18,
    cargo_json: &RealCargoJsonExecutionReportV18,
) -> WorkspaceRecoveryDecisionReportV7 {
    WorkspaceRecoveryDecisionReportV7 {
        report_id: "workspace-recovery-decision-v7".to_string(),
        recommend_continue_timeout_track: !full_gate.accepted,
        recommend_stop_consolidation: true,
        recommend_no_fifth_patch: true,
        recommend_real_observation_repeat: backlog.remaining_count > 0,
        no_run_recovered: no_run_gate.recovered,
        full_workspace_accepted: full_gate.accepted,
        status: if full_gate.accepted && cargo_json.attempted && no_run_gate.recovered {
            "DeferredObservationExecuted"
        } else if no_run_gate.recovered || cargo_json.attempted || full_gate.finished {
            "DeferredObservationPartiallyExecuted"
        } else {
            "DeferredObservationStillOpen"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_track_next_action_queue_v2(
    backlog: &ObservationBacklogCompletionReportV2,
    selection: &DeferredObservationSelectionReportV1,
    full_gate: &WorkspaceFullAcceptanceGateV18,
) -> TimeoutTrackNextActionQueueV2 {
    let mut next_actions = Vec::new();
    if backlog.deferred_count > 0 {
        for observation in &selection.selected_observations {
            next_actions.push(format!("run {observation} if explicitly configured"));
        }
    }
    if backlog.blocked_count > 0 {
        next_actions
            .push("repeat timed observations with explicit timeout cleanup capture".to_string());
    }
    if !full_gate.accepted {
        next_actions.push(
            "keep acceptance warnings until cargo test --workspace --quiet finishes and passes"
                .to_string(),
        );
    }
    if next_actions.is_empty() {
        next_actions
            .push("finalize acceptance evidence from the real full workspace pass".to_string());
    }
    TimeoutTrackNextActionQueueV2 {
        queue_id: "timeout-track-next-action-queue-v2".to_string(),
        next_actions,
        status: "TimeoutTrackNextActionsReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_control_tower_deferred_observation_execution_panel(
    selection: &DeferredObservationSelectionReportV1,
    backlog: &ObservationBacklogCompletionReportV2,
    parse: &CargoJsonActualParseReportV2,
    acceptance: &AcceptanceTruthGateV18,
    decision: &WorkspaceRecoveryDecisionReportV7,
) -> ControlTowerDeferredObservationExecutionPanel {
    ControlTowerDeferredObservationExecutionPanel {
        panel_id: "control-tower-deferred-observation-execution".to_string(),
        selected_observations: selection.selected_observations.clone(),
        execution_status: decision.status.clone(),
        cargo_json_parse_status: parse.status.clone(),
        backlog_completion: backlog.completion_status.clone(),
        acceptance_truth: acceptance.status.clone(),
        warnings: warning_posture(),
        static_read_only: true,
        no_run_button: true,
        no_action_button: true,
        no_train_runtime_live_order_account_controls: true,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_control_tower_acceptance_truth_panel_v18(
    no_run_gate: &WorkspaceNoRunRecoveryGateV18,
    full_gate: &WorkspaceFullAcceptanceGateV18,
    evidence: &ActualVsCarriedForwardEvidenceReportV1,
) -> ControlTowerAcceptanceTruthPanelV18 {
    ControlTowerAcceptanceTruthPanelV18 {
        panel_id: "control-tower-acceptance-truth-v18".to_string(),
        no_run_gate_status: no_run_gate.status.clone(),
        full_gate_status: full_gate.status.clone(),
        actual_vs_carried_forward_evidence_status: evidence.status.clone(),
        supporting_only_evidence: evidence.supporting_only_labels.clone(),
        warnings: warning_posture(),
        static_read_only: true,
        no_action_button: true,
        no_train_runtime_live_order_account_controls: true,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_safety_coverage_preservation_report_v33() -> SafetyCoveragePreservationReportV33 {
    SafetyCoveragePreservationReportV33 {
        report_id: "safety-coverage-preservation-v33".to_string(),
        no_assertion_deletion: true,
        no_safety_sentinel_deletion: true,
        no_hidden_skips: true,
        deferred_real_observation_guard_present: true,
        actual_vs_carried_forward_guard_present: true,
        cargo_json_actual_parse_guard_present: true,
        timeout_cleanup_actual_counts_guard_present: true,
        consolidation_paused_guard_present: true,
        fifth_patch_not_applied_guard_present: true,
        assertion_movement_forbidden_guard_present: true,
        target_retirement_forbidden_guard_present: true,
        runtime_deferred: true,
        training_deferred: true,
        live_trading_forbidden: true,
        no_runtime_llm_live_path: true,
        no_broker_order_account: true,
        no_mamba_runtime: true,
        no_gated_runtime: true,
        no_model_training: true,
        no_python_training_dependency: true,
        no_tauri_svelte: true,
        no_dashboard_serve: true,
        no_browser_execution: true,
        no_auto_activation_of_18_live_agents: true,
        no_silent_confidence_upgrade: true,
        safety_status: "SafetyCoveragePreserved".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

impl DeferredRealObservationExecutionBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            (
                "## 1. Sprint summary",
                format!(
                    "timeout_track={} backlog={} acceptance={} decision={}",
                    self.sprint116_baseline_truth_import_report.timeout_track_status,
                    self.observation_backlog_completion_report_v2
                        .completion_status,
                    self.acceptance_truth_gate_v18.status,
                    self.workspace_recovery_decision_report_v7.status,
                ),
            ),
            (
                "## 2. Why Sprint 117 was needed",
                "Sprint 116 left real no-run, real full workspace, and real cargo JSON observations deferred, so Sprint 117 executes or explicitly keeps them deferred without resuming consolidation.".to_string(),
            ),
            (
                "## 3. Files added",
                "Sprint 117 adds local deferred-observation reports, fixtures, docs, and tests only.".to_string(),
            ),
            (
                "## 4. Files changed",
                "Changes stay within Sprint 117 deferred-observation/report/CLI/test/docs surfaces and preserve Sprint 116 truth.".to_string(),
            ),
            (
                "## 5. Sprint 116 baseline truth import",
                format!(
                    "import_status={} imported_as_full_acceptance={}",
                    self.sprint116_baseline_truth_import_report.import_status,
                    self.sprint116_baseline_truth_import_report
                        .imported_as_full_acceptance,
                ),
            ),
            (
                "## 6. Deferred backlog carry-forward",
                format!(
                    "status={} remaining_count={}",
                    self.sprint116_deferred_backlog_carry_forward_report
                        .carry_forward_status,
                    self.sprint116_deferred_backlog_carry_forward_report
                        .remaining_count,
                ),
            ),
            (
                "## 7. Deferred observation selection",
                format!(
                    "status={} selected={}",
                    self.deferred_observation_selection_report_v1
                        .selection_status,
                    self.deferred_observation_selection_report_v1
                        .selected_observations
                        .join(","),
                ),
            ),
            (
                "## 8. Deferred observation execution plan",
                format!(
                    "status={} order={}",
                    self.deferred_observation_execution_plan_v1.plan_status,
                    self.deferred_observation_execution_plan_v1
                        .execution_order
                        .join(","),
                ),
            ),
            (
                "## 9. Real no-run execution v18",
                format!(
                    "status={} attempted={} timed_out={} passed={:?}",
                    self.real_no_run_execution_report_v18.execution_status,
                    self.real_no_run_execution_report_v18.attempted,
                    self.real_no_run_execution_report_v18.timed_out,
                    self.real_no_run_execution_report_v18.passed,
                ),
            ),
            (
                "## 10. Real full workspace execution v18",
                format!(
                    "status={} attempted={} accepted={}",
                    self.real_full_workspace_execution_report_v18
                        .execution_status,
                    self.real_full_workspace_execution_report_v18.attempted,
                    self.real_full_workspace_execution_report_v18
                        .full_workspace_accepted,
                ),
            ),
            (
                "## 11. Real cargo JSON execution v18",
                format!(
                    "status={} attempted={} parsed={} malformed={}",
                    self.real_cargo_json_execution_report_v18.execution_status,
                    self.real_cargo_json_execution_report_v18.attempted,
                    self.real_cargo_json_execution_report_v18
                        .parsed_json_message_count,
                    self.real_cargo_json_execution_report_v18
                        .malformed_line_count,
                ),
            ),
            (
                "## 12. Cargo JSON actual parse v2",
                format!(
                    "status={} artifacts={} messages={}",
                    self.cargo_json_actual_parse_report_v2.status,
                    self.cargo_json_actual_parse_report_v2
                        .compiler_artifact_count,
                    self.cargo_json_actual_parse_report_v2
                        .compiler_message_count,
                ),
            ),
            (
                "## 13. Cargo JSON parse errors and malformed lines",
                format!(
                    "parse_errors={} malformed={}",
                    self.cargo_json_parse_error_report_v2.parse_error_count,
                    self.cargo_json_malformed_line_report_v1
                        .malformed_line_count,
                ),
            ),
            (
                "## 14. Cargo JSON artifact timeline v3",
                format!(
                    "status={} events={}",
                    self.cargo_json_artifact_timeline_v3.status,
                    self.cargo_json_artifact_timeline_v3.artifact_events.len(),
                ),
            ),
            (
                "## 15. Cargo JSON target progress v2",
                format!(
                    "status={} last_seen={} stalled={}",
                    self.cargo_json_target_progress_report_v2.status,
                    self.cargo_json_target_progress_report_v2
                        .last_seen_targets
                        .len(),
                    self.cargo_json_target_progress_report_v2
                        .stalled_candidates
                        .len(),
                ),
            ),
            (
                "## 16. No-run/full execution boundaries",
                format!(
                    "no_run={} full={}",
                    self.no_run_execution_boundary_report_v2.status,
                    self.full_execution_boundary_report_v2.status,
                ),
            ),
            (
                "## 17. Timeout cleanup actual counts",
                format!(
                    "status={} cargo_entries={} rustc_entries={}",
                    self.timeout_cleanup_actual_counts_report_v2.status,
                    self.timeout_cleanup_actual_counts_report_v2
                        .actual_cargo_counts
                        .len(),
                    self.timeout_cleanup_actual_counts_report_v2
                        .actual_rustc_counts
                        .len(),
                ),
            ),
            (
                "## 18. Timeout cleanup consistency v2",
                format!(
                    "status={}",
                    self.timeout_cleanup_consistency_report_v2
                        .consistency_status,
                ),
            ),
            (
                "## 19. Observation fixture separation",
                format!(
                    "status={} overwritten_actual_count={}",
                    self.observation_fixture_separation_report_v1
                        .separation_status,
                    self.observation_fixture_separation_report_v1
                        .overwritten_actual_count,
                ),
            ),
            (
                "## 20. Actual vs carried-forward evidence",
                format!(
                    "status={} actual_rows={} carried_rows={}",
                    self.actual_vs_carried_forward_evidence_report_v1.status,
                    self.actual_vs_carried_forward_evidence_report_v1
                        .actual_evidence_rows
                        .len(),
                    self.actual_vs_carried_forward_evidence_report_v1
                        .carried_forward_evidence_rows
                        .len(),
                ),
            ),
            (
                "## 21. Observation backlog completion v2",
                format!(
                    "status={} completed={} deferred={} remaining={}",
                    self.observation_backlog_completion_report_v2
                        .completion_status,
                    self.observation_backlog_completion_report_v2
                        .completed_count,
                    self.observation_backlog_completion_report_v2
                        .deferred_count,
                    self.observation_backlog_completion_report_v2
                        .remaining_count,
                ),
            ),
            (
                "## 22. Workspace timeout evidence matrix v3",
                format!(
                    "status={} supports_acceptance={}",
                    self.workspace_timeout_evidence_matrix_v3.status,
                    self.workspace_timeout_evidence_matrix_v3
                        .supports_acceptance,
                ),
            ),
            (
                "## 23. Workspace timeout root-cause v5",
                format!(
                    "status={} confidence={}",
                    self.workspace_timeout_root_cause_report_v5.status,
                    self.workspace_timeout_root_cause_report_v5.confidence,
                ),
            ),
            (
                "## 24. Diagnostic track progress v2",
                format!(
                    "status={} attempted={}",
                    self.workspace_timeout_diagnostic_track_progress_report_v2
                        .status,
                    self.workspace_timeout_diagnostic_track_progress_report_v2
                        .actual_observations_attempted,
                ),
            ),
            (
                "## 25. Timeout track risk v2",
                format!("status={}", self.workspace_timeout_track_risk_report_v2.status),
            ),
            (
                "## 26. Consolidation track still paused v2",
                format!(
                    "status={} paused={}",
                    self.consolidation_track_still_paused_report_v2.status,
                    self.consolidation_track_still_paused_report_v2.paused,
                ),
            ),
            (
                "## 27. Fifth patch still not applied v2",
                format!(
                    "status={} applied={}",
                    self.fifth_patch_still_not_applied_report_v2.status,
                    self.fifth_patch_still_not_applied_report_v2
                        .fifth_patch_applied,
                ),
            ),
            (
                "## 28. Assertion movement still forbidden v2",
                format!(
                    "status={} movement_allowed={}",
                    self.assertion_movement_still_forbidden_report_v2.status,
                    self.assertion_movement_still_forbidden_report_v2
                        .movement_allowed,
                ),
            ),
            (
                "## 29. Target retirement still forbidden v2",
                format!(
                    "status={} retirement_allowed={}",
                    self.target_retirement_still_forbidden_report_v2.status,
                    self.target_retirement_still_forbidden_report_v2
                        .retirement_allowed,
                ),
            ),
            (
                "## 30. Workspace no-run recovery gate v18",
                format!(
                    "status={} recovered={}",
                    self.workspace_no_run_recovery_gate_v18.status,
                    self.workspace_no_run_recovery_gate_v18.recovered,
                ),
            ),
            (
                "## 31. Workspace full acceptance gate v18",
                format!(
                    "status={} accepted={}",
                    self.workspace_full_acceptance_gate_v18.status,
                    self.workspace_full_acceptance_gate_v18.accepted,
                ),
            ),
            (
                "## 32. Focused-vs-full bridge v14",
                format!("status={}", self.focused_vs_full_bridge_v14.status),
            ),
            (
                "## 33. Acceptance truth gate v18",
                format!(
                    "status={} can_claim_full_acceptance={}",
                    self.acceptance_truth_gate_v18.status,
                    self.acceptance_truth_gate_v18
                        .can_claim_full_acceptance,
                ),
            ),
            (
                "## 34. Acceptance evidence strength v7",
                format!(
                    "status={} strongest_claim={}",
                    self.acceptance_evidence_strength_report_v7.status,
                    self.acceptance_evidence_strength_report_v7
                        .strongest_claim,
                ),
            ),
            (
                "## 35. Workspace recovery decision v7",
                format!(
                    "status={} repeat={}",
                    self.workspace_recovery_decision_report_v7.status,
                    self.workspace_recovery_decision_report_v7
                        .recommend_real_observation_repeat,
                ),
            ),
            (
                "## 36. Timeout track next action queue v2",
                format!(
                    "status={} next_actions={}",
                    self.timeout_track_next_action_queue_v2.status,
                    self.timeout_track_next_action_queue_v2.next_actions.len(),
                ),
            ),
            (
                "## 37. Safety coverage preservation v33",
                format!(
                    "status={}",
                    self.safety_coverage_preservation_report_v33
                        .safety_status,
                ),
            ),
            (
                "## 38. Control Tower deferred observation execution panel",
                format!(
                    "read_only={} no_action_button={} status={}",
                    self.control_tower_deferred_observation_execution_panel
                        .static_read_only,
                    self.control_tower_deferred_observation_execution_panel
                        .no_action_button,
                    self.control_tower_deferred_observation_execution_panel
                        .execution_status,
                ),
            ),
            (
                "## 39. Control Tower acceptance truth panel v18",
                format!(
                    "read_only={} no_action_button={} evidence={}",
                    self.control_tower_acceptance_truth_panel_v18
                        .static_read_only,
                    self.control_tower_acceptance_truth_panel_v18
                        .no_action_button,
                    self.control_tower_acceptance_truth_panel_v18
                        .actual_vs_carried_forward_evidence_status,
                ),
            ),
            (
                "## 40. Output bundle",
                format!("file_count={}", self.storage_report.file_count),
            ),
            (
                "## 41. CLI and examples",
                "Sprint 117 CLI examples are local-output, deferred-real-observation-only, consolidation-paused, and report-only.".to_string(),
            ),
            (
                "## 42. Tests added",
                "Focused tests cover config, Sprint 116 import, selection, real executions, cargo JSON parsing, fixture separation, evidence matrix, acceptance truth, panels, CLI safety, and determinism.".to_string(),
            ),
            (
                "## 43. Test results",
                "Generated summary records implementation evidence only; command execution results must be reported by the verifier after running tests.".to_string(),
            ),
            (
                "## 44. Deferred observation execution status",
                format!("status={}", self.workspace_recovery_decision_report_v7.status),
            ),
            (
                "## 45. Cargo JSON parse status",
                format!("status={}", self.cargo_json_actual_parse_report_v2.status),
            ),
            (
                "## 46. Observation backlog status",
                format!(
                    "status={}",
                    self.observation_backlog_completion_report_v2
                        .completion_status,
                ),
            ),
            (
                "## 47. No-run recovery status",
                format!("status={}", self.workspace_no_run_recovery_gate_v18.status),
            ),
            (
                "## 48. Full workspace acceptance status",
                format!(
                    "status={}",
                    self.workspace_full_acceptance_gate_v18.status,
                ),
            ),
            (
                "## 49. Acceptance evidence strength status",
                format!("status={}", self.acceptance_evidence_strength_report_v7.status),
            ),
            (
                "## 50. Consolidation status",
                format!(
                    "status={}",
                    self.consolidation_track_still_paused_report_v2.status,
                ),
            ),
            (
                "## 51. Fifth patch status",
                format!("status={}", self.fifth_patch_still_not_applied_report_v2.status),
            ),
            (
                "## 52. Runtime deferred status",
                "Runtime, training, live inference, live trading, broker/order/account, runtime LLM, Mamba, and Gated runtime remain deferred or forbidden.".to_string(),
            ),
            (
                "## 53. Workspace acceptance truth status",
                format!(
                    "status={} can_claim_full_acceptance={}",
                    self.acceptance_truth_gate_v18.status,
                    self.acceptance_truth_gate_v18
                        .can_claim_full_acceptance,
                ),
            ),
            (
                "## 54. Safety coverage status",
                format!(
                    "status={}",
                    self.safety_coverage_preservation_report_v33
                        .safety_status,
                ),
            ),
            (
                "## 55. Risk review",
                "No consolidation resume, fifth patch, assertion movement, target retirement, hidden skip, fake timing, fake pass/fail, or acceptance overclaim is made.".to_string(),
            ),
            (
                "## 56. Deferred items",
                "Runtime/training/live/order/account/dashboard/browser/Tauri/Svelte/live-agent activation remain out of scope.".to_string(),
            ),
            (
                "## 57. Next gstack sprint recommendation",
                self.timeout_track_next_action_queue_v2.next_actions.join("; "),
            ),
        ];
        sections
            .into_iter()
            .map(|(heading, body)| format!("{heading}\n- {body}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn write_to_disk(
        &self,
        output_dir: &Path,
    ) -> Result<DeferredRealObservationExecutionStorageReport, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let mut written_files = Vec::new();
        macro_rules! write_report {
            ($filename:literal, $value:expr) => {{
                let path = output_dir.join($filename);
                write_text_file(&path, &render_json(&$value)?)?;
                written_files.push($filename.to_string());
            }};
        }
        write_report!(
            "sprint116_baseline_truth_import.txt",
            self.sprint116_baseline_truth_import_report
        );
        write_report!(
            "sprint116_deferred_backlog_carry_forward.txt",
            self.sprint116_deferred_backlog_carry_forward_report
        );
        write_report!(
            "deferred_observation_selection_v1.txt",
            self.deferred_observation_selection_report_v1
        );
        write_report!(
            "deferred_observation_execution_plan_v1.txt",
            self.deferred_observation_execution_plan_v1
        );
        write_report!(
            "real_no_run_execution_v18.txt",
            self.real_no_run_execution_report_v18
        );
        write_report!(
            "real_full_workspace_execution_v18.txt",
            self.real_full_workspace_execution_report_v18
        );
        write_report!(
            "real_cargo_json_execution_v18.txt",
            self.real_cargo_json_execution_report_v18
        );
        write_report!(
            "cargo_json_actual_parse_v2.txt",
            self.cargo_json_actual_parse_report_v2
        );
        write_report!(
            "cargo_json_parse_error_v2.txt",
            self.cargo_json_parse_error_report_v2
        );
        write_report!(
            "cargo_json_malformed_line_v1.txt",
            self.cargo_json_malformed_line_report_v1
        );
        write_report!(
            "cargo_json_artifact_timeline_v3.txt",
            self.cargo_json_artifact_timeline_v3
        );
        write_report!(
            "cargo_json_target_progress_v2.txt",
            self.cargo_json_target_progress_report_v2
        );
        write_report!(
            "no_run_execution_boundary_v2.txt",
            self.no_run_execution_boundary_report_v2
        );
        write_report!(
            "full_execution_boundary_v2.txt",
            self.full_execution_boundary_report_v2
        );
        write_report!(
            "timeout_cleanup_actual_counts_v2.txt",
            self.timeout_cleanup_actual_counts_report_v2
        );
        write_report!(
            "timeout_cleanup_consistency_v2.txt",
            self.timeout_cleanup_consistency_report_v2
        );
        write_report!(
            "observation_fixture_separation_v1.txt",
            self.observation_fixture_separation_report_v1
        );
        write_report!(
            "actual_vs_carried_forward_evidence_v1.txt",
            self.actual_vs_carried_forward_evidence_report_v1
        );
        write_report!(
            "observation_backlog_completion_v2.txt",
            self.observation_backlog_completion_report_v2
        );
        write_report!(
            "workspace_timeout_evidence_matrix_v3.txt",
            self.workspace_timeout_evidence_matrix_v3
        );
        write_report!(
            "workspace_timeout_root_cause_v5.txt",
            self.workspace_timeout_root_cause_report_v5
        );
        write_report!(
            "workspace_timeout_diagnostic_track_progress_v2.txt",
            self.workspace_timeout_diagnostic_track_progress_report_v2
        );
        write_report!(
            "workspace_timeout_track_risk_v2.txt",
            self.workspace_timeout_track_risk_report_v2
        );
        write_report!(
            "consolidation_track_still_paused_v2.txt",
            self.consolidation_track_still_paused_report_v2
        );
        write_report!(
            "fifth_patch_still_not_applied_v2.txt",
            self.fifth_patch_still_not_applied_report_v2
        );
        write_report!(
            "assertion_movement_still_forbidden_v2.txt",
            self.assertion_movement_still_forbidden_report_v2
        );
        write_report!(
            "target_retirement_still_forbidden_v2.txt",
            self.target_retirement_still_forbidden_report_v2
        );
        write_report!(
            "workspace_no_run_recovery_gate_v18.txt",
            self.workspace_no_run_recovery_gate_v18
        );
        write_report!(
            "workspace_full_acceptance_gate_v18.txt",
            self.workspace_full_acceptance_gate_v18
        );
        write_report!(
            "focused_vs_full_bridge_v14.txt",
            self.focused_vs_full_bridge_v14
        );
        write_report!(
            "acceptance_truth_gate_v18.txt",
            self.acceptance_truth_gate_v18
        );
        write_report!(
            "acceptance_evidence_strength_v7.txt",
            self.acceptance_evidence_strength_report_v7
        );
        write_report!(
            "workspace_recovery_decision_v7.txt",
            self.workspace_recovery_decision_report_v7
        );
        write_report!(
            "timeout_track_next_action_queue_v2.txt",
            self.timeout_track_next_action_queue_v2
        );
        write_report!(
            "safety_coverage_preservation_v33.txt",
            self.safety_coverage_preservation_report_v33
        );
        write_report!(
            "control_tower_deferred_observation_execution_panel.txt",
            self.control_tower_deferred_observation_execution_panel
        );
        write_report!(
            "control_tower_acceptance_truth_panel_v18.txt",
            self.control_tower_acceptance_truth_panel_v18
        );
        let storage_report = DeferredRealObservationExecutionStorageReport {
            report_id: "deferred-real-observation-execution-storage-report".to_string(),
            output_dir: output_dir.display().to_string(),
            written_files: written_files.clone(),
            file_count: (written_files.len() + 2) as u64,
            reason_codes: diagnostic_reason_codes(&[]),
        };
        write_text_file(
            &output_dir.join("storage_report.txt"),
            &render_json(&storage_report)?,
        )?;
        write_text_file(&output_dir.join("summary.txt"), &self.final_summary)?;
        written_files.push("storage_report.txt".to_string());
        written_files.push("summary.txt".to_string());
        Ok(DeferredRealObservationExecutionStorageReport {
            written_files,
            ..storage_report
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct DeferredRealObservationExecutionRunner;

impl DeferredRealObservationExecutionRunner {
    pub fn run(
        &self,
        config: &DeferredRealObservationExecutionConfig,
    ) -> Result<DeferredRealObservationExecutionBundle, String> {
        config.validate()?;
        let summary = load_first_json::<Sprint116SummaryFixture>(
            config
                .sprint116_truth_paths
                .as_ref()
                .or(config.sprint116_bundle_paths.as_ref()),
        )?
        .unwrap_or_default();
        let sprint116_baseline_truth_import_report =
            build_sprint116_baseline_truth_import_report(&summary);
        let sprint116_deferred_backlog_carry_forward_report =
            build_sprint116_deferred_backlog_carry_forward_report(&summary);
        let deferred_observation_selection_report_v1 =
            build_deferred_observation_selection_report_v1(
                &sprint116_deferred_backlog_carry_forward_report,
            );
        let deferred_observation_execution_plan_v1 = build_deferred_observation_execution_plan_v1(
            &deferred_observation_selection_report_v1,
            config,
        );

        let mut no_run_observation = None;
        let mut full_observation = None;
        let mut cargo_json_observation = None;
        for step in &deferred_observation_execution_plan_v1.execution_order {
            match step.as_str() {
                "RealCargoJson" if config.run_real_cargo_json_observation => {
                    cargo_json_observation = Some(run_timed_command(
                        "cargo test --workspace --no-run --message-format=json",
                        config.cargo_json_timeout_ms,
                    ));
                }
                "RealNoRun" if config.run_real_no_run_observation => {
                    no_run_observation = Some(run_timed_command(
                        "cargo test --workspace --no-run --quiet",
                        config.no_run_timeout_ms,
                    ));
                }
                "RealFullWorkspace" if config.run_real_full_observation => {
                    full_observation = Some(run_timed_command(
                        "cargo test --workspace --quiet",
                        config.full_timeout_ms,
                    ));
                }
                _ => {}
            }
        }

        let cargo_json_cleanup_counts = cleanup_counts_after_observation(&cargo_json_observation);
        let no_run_cleanup_counts = cleanup_counts_after_observation(&no_run_observation);
        let full_cleanup_counts = cleanup_counts_after_observation(&full_observation);

        let real_no_run_execution_report_v18 = build_real_no_run_execution_report_v18(
            no_run_observation.as_ref(),
            config.no_run_timeout_ms,
            no_run_cleanup_counts,
        );
        let real_full_workspace_execution_report_v18 =
            build_real_full_workspace_execution_report_v18(
                full_observation.as_ref(),
                config.full_timeout_ms,
                full_cleanup_counts,
            );
        let real_cargo_json_execution_report_v18 = build_real_cargo_json_execution_report_v18(
            cargo_json_observation.as_ref(),
            config.cargo_json_timeout_ms,
        );
        let cargo_json_actual_parse_report_v2 =
            build_cargo_json_actual_parse_report_v2(cargo_json_observation.as_ref());
        let cargo_json_parse_error_report_v2 =
            build_cargo_json_parse_error_report_v2(cargo_json_observation.as_ref());
        let cargo_json_malformed_line_report_v1 =
            build_cargo_json_malformed_line_report_v1(cargo_json_observation.as_ref());
        let cargo_json_artifact_timeline_v3 =
            build_cargo_json_artifact_timeline_v3(cargo_json_observation.as_ref());
        let cargo_json_target_progress_report_v2 =
            build_cargo_json_target_progress_report_v2(cargo_json_observation.as_ref());
        let no_run_execution_boundary_report_v2 = build_no_run_execution_boundary_report_v2(
            &summary,
            &real_no_run_execution_report_v18,
            config.no_run_timeout_ms,
        );
        let full_execution_boundary_report_v2 = build_full_execution_boundary_report_v2(
            &summary,
            &real_full_workspace_execution_report_v18,
            config.full_timeout_ms,
        );
        let timeout_cleanup_actual_counts_report_v2 = build_timeout_cleanup_actual_counts_report_v2(
            no_run_cleanup_counts,
            full_cleanup_counts,
            cargo_json_cleanup_counts,
        );
        let timeout_cleanup_consistency_report_v2 =
            build_timeout_cleanup_consistency_report_v2(&timeout_cleanup_actual_counts_report_v2);
        let observation_fixture_separation_report_v1 =
            build_observation_fixture_separation_report_v1(config);
        let actual_vs_carried_forward_evidence_report_v1 =
            build_actual_vs_carried_forward_evidence_report_v1(
                &sprint116_baseline_truth_import_report,
                &real_no_run_execution_report_v18,
                &real_full_workspace_execution_report_v18,
                &real_cargo_json_execution_report_v18,
            );
        let observation_backlog_completion_report_v2 =
            build_observation_backlog_completion_report_v2(
                &sprint116_deferred_backlog_carry_forward_report,
                &real_no_run_execution_report_v18,
                &real_full_workspace_execution_report_v18,
                &real_cargo_json_execution_report_v18,
            );
        let workspace_timeout_evidence_matrix_v3 = build_workspace_timeout_evidence_matrix_v3(
            &summary,
            &real_no_run_execution_report_v18,
            &real_full_workspace_execution_report_v18,
            &real_cargo_json_execution_report_v18,
            &cargo_json_actual_parse_report_v2,
            &timeout_cleanup_consistency_report_v2,
            &cargo_json_artifact_timeline_v3,
        );
        let workspace_timeout_root_cause_report_v5 = build_workspace_timeout_root_cause_report_v5(
            &summary,
            &real_no_run_execution_report_v18,
            &real_full_workspace_execution_report_v18,
            &real_cargo_json_execution_report_v18,
            &cargo_json_actual_parse_report_v2,
            &timeout_cleanup_actual_counts_report_v2,
        );
        let workspace_timeout_diagnostic_track_progress_report_v2 =
            build_workspace_timeout_diagnostic_track_progress_report_v2(
                &summary,
                &observation_backlog_completion_report_v2,
                &real_no_run_execution_report_v18,
                &real_full_workspace_execution_report_v18,
                &real_cargo_json_execution_report_v18,
            );
        let workspace_timeout_track_risk_report_v2 = build_workspace_timeout_track_risk_report_v2();
        let consolidation_track_still_paused_report_v2 =
            build_consolidation_track_still_paused_report_v2(
                &sprint116_baseline_truth_import_report,
            );
        let fifth_patch_still_not_applied_report_v2 =
            build_fifth_patch_still_not_applied_report_v2();
        let assertion_movement_still_forbidden_report_v2 =
            build_assertion_movement_still_forbidden_report_v2();
        let target_retirement_still_forbidden_report_v2 =
            build_target_retirement_still_forbidden_report_v2();
        let workspace_no_run_recovery_gate_v18 =
            build_workspace_no_run_recovery_gate_v18(&real_no_run_execution_report_v18);
        let workspace_full_acceptance_gate_v18 = build_workspace_full_acceptance_gate_v18(
            &summary,
            &real_full_workspace_execution_report_v18,
        );
        let focused_vs_full_bridge_v14 = build_focused_vs_full_bridge_v14(
            &summary,
            &workspace_no_run_recovery_gate_v18,
            &workspace_full_acceptance_gate_v18,
        );
        let acceptance_truth_gate_v18 = build_acceptance_truth_gate_v18(
            &summary,
            &workspace_no_run_recovery_gate_v18,
            &workspace_full_acceptance_gate_v18,
        );
        let acceptance_evidence_strength_report_v7 =
            build_acceptance_evidence_strength_report_v7(&acceptance_truth_gate_v18);
        let workspace_recovery_decision_report_v7 = build_workspace_recovery_decision_report_v7(
            &observation_backlog_completion_report_v2,
            &workspace_no_run_recovery_gate_v18,
            &workspace_full_acceptance_gate_v18,
            &real_cargo_json_execution_report_v18,
        );
        let timeout_track_next_action_queue_v2 = build_timeout_track_next_action_queue_v2(
            &observation_backlog_completion_report_v2,
            &deferred_observation_selection_report_v1,
            &workspace_full_acceptance_gate_v18,
        );
        let safety_coverage_preservation_report_v33 =
            build_safety_coverage_preservation_report_v33();
        let control_tower_deferred_observation_execution_panel =
            build_control_tower_deferred_observation_execution_panel(
                &deferred_observation_selection_report_v1,
                &observation_backlog_completion_report_v2,
                &cargo_json_actual_parse_report_v2,
                &acceptance_truth_gate_v18,
                &workspace_recovery_decision_report_v7,
            );
        let control_tower_acceptance_truth_panel_v18 =
            build_control_tower_acceptance_truth_panel_v18(
                &workspace_no_run_recovery_gate_v18,
                &workspace_full_acceptance_gate_v18,
                &actual_vs_carried_forward_evidence_report_v1,
            );

        let mut bundle = DeferredRealObservationExecutionBundle {
            sprint116_baseline_truth_import_report,
            sprint116_deferred_backlog_carry_forward_report,
            deferred_observation_selection_report_v1,
            deferred_observation_execution_plan_v1,
            real_no_run_execution_report_v18,
            real_full_workspace_execution_report_v18,
            real_cargo_json_execution_report_v18,
            cargo_json_actual_parse_report_v2,
            cargo_json_parse_error_report_v2,
            cargo_json_malformed_line_report_v1,
            cargo_json_artifact_timeline_v3,
            cargo_json_target_progress_report_v2,
            no_run_execution_boundary_report_v2,
            full_execution_boundary_report_v2,
            timeout_cleanup_actual_counts_report_v2,
            timeout_cleanup_consistency_report_v2,
            observation_fixture_separation_report_v1,
            actual_vs_carried_forward_evidence_report_v1,
            observation_backlog_completion_report_v2,
            workspace_timeout_evidence_matrix_v3,
            workspace_timeout_root_cause_report_v5,
            workspace_timeout_diagnostic_track_progress_report_v2,
            workspace_timeout_track_risk_report_v2,
            consolidation_track_still_paused_report_v2,
            fifth_patch_still_not_applied_report_v2,
            assertion_movement_still_forbidden_report_v2,
            target_retirement_still_forbidden_report_v2,
            workspace_no_run_recovery_gate_v18,
            workspace_full_acceptance_gate_v18,
            focused_vs_full_bridge_v14,
            acceptance_truth_gate_v18,
            acceptance_evidence_strength_report_v7,
            workspace_recovery_decision_report_v7,
            timeout_track_next_action_queue_v2,
            safety_coverage_preservation_report_v33,
            control_tower_deferred_observation_execution_panel,
            control_tower_acceptance_truth_panel_v18,
            storage_report: DeferredRealObservationExecutionStorageReport {
                report_id: "deferred-real-observation-execution-storage-report".to_string(),
                output_dir: config.output_dir().display().to_string(),
                written_files: Vec::new(),
                file_count: 39,
                reason_codes: diagnostic_reason_codes(&[]),
            },
            final_summary: String::new(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        bundle.final_summary = bundle.build_final_summary();
        bundle.storage_report = bundle.write_to_disk(&config.output_dir())?;
        Ok(bundle)
    }
}
