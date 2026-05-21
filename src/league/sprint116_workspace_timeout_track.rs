use crate::ReasonCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    "target/soma_sprint116_workspace_timeout_track".to_string()
}

fn default_execution_id() -> String {
    "sprint116-workspace-timeout-track".to_string()
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
        "timeout-track-only",
        "consolidation-paused",
        "fifth-patch-not-applied",
        "no-assertion-movement",
        "no-target-retirement",
        "focused-is-not-full",
        "CLI-smoke-is-not-full",
        "cargo-build-is-not-full",
        "no-run-is-not-full",
        "cargo-progress-is-not-acceptance",
        "artifact-ordering-is-not-acceptance",
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
pub struct WorkspaceTimeoutTrackExecutionConfig {
    pub execution_id: String,
    #[serde(default)]
    pub sprint115_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub sprint115_truth_paths: Option<Vec<String>>,
    #[serde(default)]
    pub workspace_timeout_backlog_paths: Option<Vec<String>>,
    #[serde(default)]
    pub no_run_observation_plan_paths: Option<Vec<String>>,
    #[serde(default)]
    pub full_observation_plan_paths: Option<Vec<String>>,
    #[serde(default)]
    pub cargo_json_observation_plan_paths: Option<Vec<String>>,
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
    pub require_backlog_burndown: bool,
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

impl Default for WorkspaceTimeoutTrackExecutionConfig {
    fn default() -> Self {
        Self {
            execution_id: default_execution_id(),
            sprint115_bundle_paths: Some(vec![
                "examples/sprint116_data/sprint115_summary.json".to_string(),
            ]),
            sprint115_truth_paths: Some(vec![
                "examples/sprint116_data/sprint115_summary.json".to_string(),
            ]),
            workspace_timeout_backlog_paths: None,
            no_run_observation_plan_paths: None,
            full_observation_plan_paths: None,
            cargo_json_observation_plan_paths: None,
            output_root: default_output_root(),
            run_real_no_run_observation: false,
            run_real_full_observation: false,
            run_real_cargo_json_observation: false,
            no_run_timeout_ms: default_timeout_ms(),
            full_timeout_ms: default_timeout_ms(),
            cargo_json_timeout_ms: default_timeout_ms(),
            require_backlog_burndown: true,
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

impl WorkspaceTimeoutTrackExecutionConfig {
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
            return Err("sprint116 execution_id must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err("sprint116 output_root must be local-only".to_string());
        }
        Self::validate_paths(&self.sprint115_bundle_paths, "sprint115_bundle_paths")?;
        Self::validate_paths(&self.sprint115_truth_paths, "sprint115_truth_paths")?;
        Self::validate_paths(
            &self.workspace_timeout_backlog_paths,
            "workspace_timeout_backlog_paths",
        )?;
        Self::validate_paths(
            &self.no_run_observation_plan_paths,
            "no_run_observation_plan_paths",
        )?;
        Self::validate_paths(
            &self.full_observation_plan_paths,
            "full_observation_plan_paths",
        )?;
        Self::validate_paths(
            &self.cargo_json_observation_plan_paths,
            "cargo_json_observation_plan_paths",
        )?;
        if !self.require_backlog_burndown {
            return Err("require_backlog_burndown must remain true".to_string());
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
pub struct Sprint115SummaryFixture {
    pub report_id: String,
    pub consolidation_status: String,
    pub consolidation_paused_status: String,
    pub consolidation_resume_status: String,
    pub assertion_destination_proof_status: String,
    pub evidence_blur_status: String,
    pub fifth_patch_status: String,
    pub consolidation_stopped_status: String,
    pub workspace_timeout_track_status: String,
    pub workspace_timeout_diagnostic_status: String,
    pub no_run_status: String,
    pub full_workspace_status: String,
    pub acceptance_truth_status: String,
    pub acceptance_evidence_status: String,
    pub safety_status: String,
    pub focused_tests_passed: bool,
    pub cli_smoke_passed: bool,
    pub cargo_check_passed: bool,
    pub cargo_build_passed: bool,
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

impl Default for Sprint115SummaryFixture {
    fn default() -> Self {
        Self {
            report_id: "sprint115-summary".to_string(),
            consolidation_status: "ConsolidationStopRecommendedWithWarnings".to_string(),
            consolidation_paused_status: "ConsolidationPaused".to_string(),
            consolidation_resume_status: "ConsolidationResumeNeedsProof".to_string(),
            assertion_destination_proof_status: "AssertionDestinationProofStillMissing".to_string(),
            evidence_blur_status: "EvidenceBlurRiskTooHigh".to_string(),
            fifth_patch_status: "FifthPatchStillBlocked".to_string(),
            consolidation_stopped_status: "ConsolidationStopped".to_string(),
            workspace_timeout_track_status: "WorkspaceTimeoutTrackSeparated".to_string(),
            workspace_timeout_diagnostic_status: "WorkspaceTimeoutDiagnosticTrackActive"
                .to_string(),
            no_run_status: "NoRunStillBlocked".to_string(),
            full_workspace_status: "FullWorkspaceStillBlocked".to_string(),
            acceptance_truth_status: "AcceptanceTruthReadyWithWarnings".to_string(),
            acceptance_evidence_status: "AcceptanceEvidenceSupportingOnly".to_string(),
            safety_status: "SafetyCoveragePreserved".to_string(),
            focused_tests_passed: true,
            cli_smoke_passed: true,
            cargo_check_passed: true,
            cargo_build_passed: true,
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
pub struct WorkspaceTimeoutBacklogItemV1 {
    pub item: String,
    pub category: String,
    pub required: bool,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceTimeoutEvidenceRowV2 {
    pub row_id: String,
    pub evidence_type: String,
    pub evidence_status: String,
    pub supports_acceptance: bool,
    pub supporting_only: bool,
}

report!(Sprint115BaselineTruthImportReport {
    report_id: String,
    consolidation_status: String,
    fifth_patch_status: String,
    assertion_destination_proof_status: String,
    evidence_blur_status: String,
    workspace_timeout_track_status: String,
    no_run_status: String,
    full_workspace_status: String,
    acceptance_truth_status: String,
    safety_status: String,
    focused_tests_passed: bool,
    cli_smoke_passed: bool,
    cargo_check_passed: bool,
    cargo_build_passed: bool,
    imported_as_full_acceptance: bool,
    import_status: String
});
report!(ConsolidationPausedCarryForwardReport {
    report_id: String,
    consolidation_paused: bool,
    consolidation_stopped: bool,
    fifth_patch_blocked: bool,
    no_assertion_movement: bool,
    no_target_retirement: bool,
    carry_forward_status: String
});
report!(WorkspaceTimeoutTrackActivationReportV1 {
    report_id: String,
    previous_track_status: String,
    diagnostic_track_active: bool,
    consolidation_track_separated: bool,
    observation_backlog_present: bool,
    activation_status: String
});
report!(WorkspaceTimeoutObservationBacklogImportReportV1 {
    report_id: String,
    backlog_items_imported: Vec<WorkspaceTimeoutBacklogItemV1>,
    no_run_items: u64,
    full_items: u64,
    cargo_json_items: u64,
    artifact_ordering_items: u64,
    cleanup_items: u64,
    import_status: String
});
report!(WorkspaceTimeoutObservationBacklogBurnDownPlanV1 {
    plan_id: String,
    planned_tasks: Vec<String>,
    task_order: Vec<String>,
    required_tasks: Vec<String>,
    optional_tasks: Vec<String>,
    blocked_tasks: Vec<String>,
    plan_status: String
});
report!(WorkspaceTimeoutObservationBacklogBurnDownReportV1 {
    report_id: String,
    planned_count: u64,
    completed_count: u64,
    skipped_count: u64,
    blocked_count: u64,
    deferred_count: u64,
    remaining_count: u64,
    burn_down_status: String
});
report!(NoRunObservationTaskReportV1 {
    task_id: String,
    planned: bool,
    attempted: bool,
    completed: bool,
    timed_out: bool,
    supporting_only: bool,
    task_status: String
});
report!(FullWorkspaceObservationTaskReportV1 {
    task_id: String,
    planned: bool,
    attempted: bool,
    completed: bool,
    timed_out: bool,
    supporting_only: bool,
    full_accepted: bool,
    task_status: String
});
report!(CargoJsonObservationTaskReportV1 {
    task_id: String,
    planned: bool,
    attempted: bool,
    completed: bool,
    supporting_only: bool,
    actual_json_parsing_enabled: bool,
    parsed_message_count: u64,
    task_status: String
});
report!(TimeoutBoundaryObservationTaskReportV1 {
    task_id: String,
    observed_timeout_boundary: bool,
    no_run_timeout_ms: Option<u64>,
    no_run_exit_code: Option<i32>,
    full_timeout_ms: Option<u64>,
    full_exit_code: Option<i32>,
    task_status: String
});
report!(TimeoutCleanupConsistencyTaskReportV1 {
    task_id: String,
    planned: bool,
    actual_remaining_cargo_count: u64,
    actual_remaining_rustc_count: u64,
    cleanup_verified: bool,
    task_status: String
});
report!(CargoArtifactOrderingObservationTaskReportV1 {
    task_id: String,
    planned: bool,
    observed_artifact_count: u64,
    deterministic: bool,
    task_status: String
});
report!(RealNoRunObservationAttemptV17 {
    attempt_id: String,
    command: String,
    attempted: bool,
    started: bool,
    finished: bool,
    passed: Option<bool>,
    duration_ms: Option<u64>,
    timeout_ms: Option<u64>,
    exit_code: Option<i32>,
    timed_out: bool,
    supporting_only: bool,
    attempt_status: String
});
report!(RealFullWorkspaceObservationAttemptV17 {
    attempt_id: String,
    command: String,
    attempted: bool,
    started: bool,
    finished: bool,
    passed: Option<bool>,
    duration_ms: Option<u64>,
    timeout_ms: Option<u64>,
    exit_code: Option<i32>,
    timed_out: bool,
    supporting_only: bool,
    full_accepted: bool,
    attempt_status: String
});
report!(RealCargoJsonObservationAttemptV17 {
    attempt_id: String,
    command: String,
    attempted: bool,
    started: bool,
    finished: bool,
    duration_ms: Option<u64>,
    timeout_ms: Option<u64>,
    exit_code: Option<i32>,
    timed_out: bool,
    actual_json_parsing_enabled: bool,
    parsed_message_count: u64,
    parse_error_count: u64,
    malformed_line_count: u64,
    last_seen_targets: Vec<String>,
    last_seen_artifacts: Vec<String>,
    supporting_only: bool,
    attempt_status: String
});
report!(NoRunTimeoutBoundaryReportV1 {
    report_id: String,
    timeout_ms: Option<u64>,
    exit_code: Option<i32>,
    actual_observation: bool,
    timed_out: bool,
    status: String
});
report!(FullTimeoutBoundaryReportV1 {
    report_id: String,
    timeout_ms: Option<u64>,
    exit_code: Option<i32>,
    actual_observation: bool,
    timed_out: bool,
    status: String
});
report!(TimeoutCleanupConsistencyReportV1 {
    report_id: String,
    no_run_remaining_cargo_processes: u64,
    no_run_remaining_rustc_processes: u64,
    full_remaining_cargo_processes: u64,
    full_remaining_rustc_processes: u64,
    consistent_with_baseline: bool,
    timeout_cleanup_is_pass: bool,
    status: String
});
report!(CargoArtifactOrderingReportV1 {
    report_id: String,
    observed_artifact_order: Vec<String>,
    deterministic_ordering_analysis: String,
    status: String
});
report!(CargoJsonArtifactOrderingReportV1 {
    report_id: String,
    observed_artifact_order: Vec<String>,
    status: String
});
report!(CargoJsonParseQualityReportV1 {
    report_id: String,
    parsed_count: u64,
    parse_error_count: u64,
    malformed_line_count: u64,
    quality_status: String
});
report!(WorkspaceTimeoutEvidenceMatrixV2 {
    report_id: String,
    evidence_rows: Vec<WorkspaceTimeoutEvidenceRowV2>,
    status: String
});
report!(WorkspaceTimeoutRootCauseReportV4 {
    report_id: String,
    previous_root_cause: String,
    new_timeout_evidence: Vec<String>,
    new_artifact_ordering_evidence: Vec<String>,
    new_cleanup_evidence: Vec<String>,
    root_cause_confidence: String,
    status: String
});
report!(WorkspaceTimeoutDiagnosticTrackProgressReportV1 {
    report_id: String,
    diagnostic_track_active: bool,
    backlog_status: String,
    attempted_observations: u64,
    completed_observations: u64,
    status: String
});
report!(WorkspaceTimeoutTrackRiskReportV1 {
    report_id: String,
    overclaim_risk: String,
    fixture_overwrite_risk: String,
    parse_error_risk: String,
    cleanup_false_positive_risk: String,
    acceptance_confusion_risk: String,
    status: String
});
report!(ConsolidationTrackStillPausedReportV1 {
    report_id: String,
    paused: bool,
    stopped: bool,
    no_assertion_movement: bool,
    no_target_retirement: bool,
    status: String
});
report!(FifthPatchStillNotAppliedReportV1 {
    report_id: String,
    fifth_patch_applied: bool,
    no_assertions_moved: bool,
    no_targets_retired: bool,
    status: String
});
report!(AssertionMovementStillForbiddenReportV1 {
    report_id: String,
    movement_allowed: bool,
    status: String
});
report!(TargetRetirementStillForbiddenReportV1 {
    report_id: String,
    retirement_allowed: bool,
    status: String
});
report!(AcceptanceTruthGateV17 {
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
report!(FocusedVsFullBridgeV13 {
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
report!(WorkspaceNoRunRecoveryGateV17 {
    gate_id: String,
    command: String,
    finished: bool,
    passed: bool,
    timed_out: bool,
    recovered: bool,
    status: String
});
report!(WorkspaceFullAcceptanceGateV17 {
    gate_id: String,
    command: String,
    finished: bool,
    passed: bool,
    accepted: bool,
    status: String
});
report!(AcceptanceEvidenceStrengthReportV6 {
    report_id: String,
    full_evidence_sufficient: bool,
    evidence_tiers: Vec<String>,
    strongest_claim: String,
    status: String
});
report!(WorkspaceRecoveryDecisionReportV6 {
    report_id: String,
    recommend_continue_timeout_track: bool,
    recommend_stop_consolidation: bool,
    recommend_no_fifth_patch: bool,
    no_run_recovered: bool,
    full_workspace_accepted: bool,
    status: String
});
report!(TimeoutTrackNextActionQueueV1 {
    queue_id: String,
    next_actions: Vec<String>,
    status: String
});
report!(ControlTowerWorkspaceTimeoutTrackExecutionPanel {
    panel_id: String,
    backlog_burn_down_status: String,
    no_run_status: String,
    full_status: String,
    cargo_json_status: String,
    cleanup_status: String,
    artifact_ordering_status: String,
    root_cause_status: String,
    next_actions: Vec<String>,
    warnings: Vec<String>,
    static_read_only: bool,
    no_run_button: bool,
    no_action_button: bool,
    no_train_runtime_live_order_account_controls: bool
});
report!(ControlTowerAcceptanceTruthPanelV17 {
    panel_id: String,
    no_run_gate_status: String,
    full_gate_status: String,
    acceptance_truth_status: String,
    supporting_only_evidence: Vec<String>,
    warnings: Vec<String>,
    static_read_only: bool,
    no_action_button: bool,
    no_train_runtime_live_order_account_controls: bool
});
report!(SafetyCoveragePreservationReportV32 {
    report_id: String,
    no_assertion_deletion: bool,
    no_safety_sentinel_deletion: bool,
    no_hidden_skips: bool,
    timeout_track_execution_guard_present: bool,
    consolidation_paused_guard_present: bool,
    fifth_patch_still_not_applied_guard_present: bool,
    assertion_movement_forbidden_guard_present: bool,
    target_retirement_forbidden_guard_present: bool,
    actual_cargo_json_parse_guard_present: bool,
    actual_timeout_cleanup_counts_guard_present: bool,
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
report!(WorkspaceTimeoutTrackExecutionStorageReport {
    report_id: String,
    output_dir: String,
    written_files: Vec<String>,
    file_count: u64
});

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceTimeoutTrackExecutionBundle {
    pub sprint115_baseline_truth_import_report: Sprint115BaselineTruthImportReport,
    pub consolidation_paused_carry_forward_report: ConsolidationPausedCarryForwardReport,
    pub workspace_timeout_track_activation_report_v1: WorkspaceTimeoutTrackActivationReportV1,
    pub workspace_timeout_observation_backlog_import_report_v1:
        WorkspaceTimeoutObservationBacklogImportReportV1,
    pub workspace_timeout_observation_backlog_burn_down_plan_v1:
        WorkspaceTimeoutObservationBacklogBurnDownPlanV1,
    pub workspace_timeout_observation_backlog_burn_down_report_v1:
        WorkspaceTimeoutObservationBacklogBurnDownReportV1,
    pub no_run_observation_task_report_v1: NoRunObservationTaskReportV1,
    pub full_workspace_observation_task_report_v1: FullWorkspaceObservationTaskReportV1,
    pub cargo_json_observation_task_report_v1: CargoJsonObservationTaskReportV1,
    pub timeout_boundary_observation_task_report_v1: TimeoutBoundaryObservationTaskReportV1,
    pub timeout_cleanup_consistency_task_report_v1: TimeoutCleanupConsistencyTaskReportV1,
    pub cargo_artifact_ordering_observation_task_report_v1:
        CargoArtifactOrderingObservationTaskReportV1,
    pub real_no_run_observation_attempt_v17: RealNoRunObservationAttemptV17,
    pub real_full_workspace_observation_attempt_v17: RealFullWorkspaceObservationAttemptV17,
    pub real_cargo_json_observation_attempt_v17: RealCargoJsonObservationAttemptV17,
    pub no_run_timeout_boundary_report_v1: NoRunTimeoutBoundaryReportV1,
    pub full_timeout_boundary_report_v1: FullTimeoutBoundaryReportV1,
    pub timeout_cleanup_consistency_report_v1: TimeoutCleanupConsistencyReportV1,
    pub cargo_artifact_ordering_report_v1: CargoArtifactOrderingReportV1,
    pub cargo_json_artifact_ordering_report_v1: CargoJsonArtifactOrderingReportV1,
    pub cargo_json_parse_quality_report_v1: CargoJsonParseQualityReportV1,
    pub workspace_timeout_evidence_matrix_v2: WorkspaceTimeoutEvidenceMatrixV2,
    pub workspace_timeout_root_cause_report_v4: WorkspaceTimeoutRootCauseReportV4,
    pub workspace_timeout_diagnostic_track_progress_report_v1:
        WorkspaceTimeoutDiagnosticTrackProgressReportV1,
    pub workspace_timeout_track_risk_report_v1: WorkspaceTimeoutTrackRiskReportV1,
    pub consolidation_track_still_paused_report_v1: ConsolidationTrackStillPausedReportV1,
    pub fifth_patch_still_not_applied_report_v1: FifthPatchStillNotAppliedReportV1,
    pub assertion_movement_still_forbidden_report_v1: AssertionMovementStillForbiddenReportV1,
    pub target_retirement_still_forbidden_report_v1: TargetRetirementStillForbiddenReportV1,
    pub acceptance_truth_gate_v17: AcceptanceTruthGateV17,
    pub focused_vs_full_bridge_v13: FocusedVsFullBridgeV13,
    pub workspace_no_run_recovery_gate_v17: WorkspaceNoRunRecoveryGateV17,
    pub workspace_full_acceptance_gate_v17: WorkspaceFullAcceptanceGateV17,
    pub acceptance_evidence_strength_report_v6: AcceptanceEvidenceStrengthReportV6,
    pub workspace_recovery_decision_report_v6: WorkspaceRecoveryDecisionReportV6,
    pub timeout_track_next_action_queue_v1: TimeoutTrackNextActionQueueV1,
    pub safety_coverage_preservation_report_v32: SafetyCoveragePreservationReportV32,
    pub control_tower_workspace_timeout_track_execution_panel:
        ControlTowerWorkspaceTimeoutTrackExecutionPanel,
    pub control_tower_acceptance_truth_panel_v17: ControlTowerAcceptanceTruthPanelV17,
    pub storage_report: WorkspaceTimeoutTrackExecutionStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

fn default_backlog_items() -> Vec<WorkspaceTimeoutBacklogItemV1> {
    vec![
        WorkspaceTimeoutBacklogItemV1 {
            item: "no-run-observation".to_string(),
            category: "no-run".to_string(),
            required: true,
            status: "Queued".to_string(),
        },
        WorkspaceTimeoutBacklogItemV1 {
            item: "full-workspace-observation".to_string(),
            category: "full".to_string(),
            required: true,
            status: "Queued".to_string(),
        },
        WorkspaceTimeoutBacklogItemV1 {
            item: "cargo-json-observation".to_string(),
            category: "cargo-json".to_string(),
            required: true,
            status: "Queued".to_string(),
        },
        WorkspaceTimeoutBacklogItemV1 {
            item: "cargo-artifact-ordering".to_string(),
            category: "artifact-ordering".to_string(),
            required: true,
            status: "Queued".to_string(),
        },
        WorkspaceTimeoutBacklogItemV1 {
            item: "timeout-cleanup-consistency".to_string(),
            category: "cleanup".to_string(),
            required: true,
            status: "Queued".to_string(),
        },
    ]
}

pub fn build_sprint115_baseline_truth_import_report(
    summary: &Sprint115SummaryFixture,
) -> Sprint115BaselineTruthImportReport {
    Sprint115BaselineTruthImportReport {
        report_id: "sprint115-baseline-truth-import".to_string(),
        consolidation_status: summary.consolidation_status.clone(),
        fifth_patch_status: summary.fifth_patch_status.clone(),
        assertion_destination_proof_status: summary.assertion_destination_proof_status.clone(),
        evidence_blur_status: summary.evidence_blur_status.clone(),
        workspace_timeout_track_status: summary.workspace_timeout_track_status.clone(),
        no_run_status: summary.no_run_status.clone(),
        full_workspace_status: summary.full_workspace_status.clone(),
        acceptance_truth_status: summary.acceptance_truth_status.clone(),
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
            "Sprint115TruthImportedWithWarnings"
        } else {
            "Sprint115TruthImported"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_consolidation_paused_carry_forward_report(
    summary: &Sprint115SummaryFixture,
) -> ConsolidationPausedCarryForwardReport {
    ConsolidationPausedCarryForwardReport {
        report_id: "consolidation-paused-carry-forward".to_string(),
        consolidation_paused: summary.consolidation_paused_status == "ConsolidationPaused",
        consolidation_stopped: summary.consolidation_stopped_status == "ConsolidationStopped",
        fifth_patch_blocked: summary.fifth_patch_status == "FifthPatchStillBlocked",
        no_assertion_movement: true,
        no_target_retirement: true,
        carry_forward_status: "ConsolidationPausedCarriedForward".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_track_activation_report_v1(
    summary: &Sprint115SummaryFixture,
    backlog_present: bool,
) -> WorkspaceTimeoutTrackActivationReportV1 {
    WorkspaceTimeoutTrackActivationReportV1 {
        report_id: "workspace-timeout-track-activation-v1".to_string(),
        previous_track_status: summary.workspace_timeout_track_status.clone(),
        diagnostic_track_active: summary.workspace_timeout_diagnostic_status
            == "WorkspaceTimeoutDiagnosticTrackActive",
        consolidation_track_separated: summary.workspace_timeout_track_status
            == "WorkspaceTimeoutTrackSeparated",
        observation_backlog_present: backlog_present,
        activation_status: if backlog_present {
            "TimeoutTrackActive"
        } else {
            "TimeoutTrackActiveWithWarnings"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_observation_backlog_import_report_v1()
-> WorkspaceTimeoutObservationBacklogImportReportV1 {
    let backlog_items_imported = default_backlog_items();
    WorkspaceTimeoutObservationBacklogImportReportV1 {
        report_id: "workspace-timeout-observation-backlog-import-v1".to_string(),
        no_run_items: backlog_items_imported
            .iter()
            .filter(|item| item.category == "no-run")
            .count() as u64,
        full_items: backlog_items_imported
            .iter()
            .filter(|item| item.category == "full")
            .count() as u64,
        cargo_json_items: backlog_items_imported
            .iter()
            .filter(|item| item.category == "cargo-json")
            .count() as u64,
        artifact_ordering_items: backlog_items_imported
            .iter()
            .filter(|item| item.category == "artifact-ordering")
            .count() as u64,
        cleanup_items: backlog_items_imported
            .iter()
            .filter(|item| item.category == "cleanup")
            .count() as u64,
        backlog_items_imported,
        import_status: "BacklogImported".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_observation_backlog_burn_down_plan_v1(
    backlog: &WorkspaceTimeoutObservationBacklogImportReportV1,
) -> WorkspaceTimeoutObservationBacklogBurnDownPlanV1 {
    let required_tasks = backlog
        .backlog_items_imported
        .iter()
        .filter(|item| item.required)
        .map(|item| item.item.clone())
        .collect::<Vec<_>>();
    let task_order = vec![
        "no-run-observation".to_string(),
        "full-workspace-observation".to_string(),
        "cargo-json-observation".to_string(),
        "cargo-artifact-ordering".to_string(),
        "timeout-cleanup-consistency".to_string(),
    ];
    WorkspaceTimeoutObservationBacklogBurnDownPlanV1 {
        plan_id: "workspace-timeout-observation-backlog-burndown-plan-v1".to_string(),
        planned_tasks: task_order.clone(),
        task_order,
        required_tasks,
        optional_tasks: vec![
            "no-run-timeout-boundary".to_string(),
            "full-timeout-boundary".to_string(),
        ],
        blocked_tasks: Vec::new(),
        plan_status: "BacklogBurnDownPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandObservation {
    pub attempted: bool,
    pub finished: bool,
    pub passed: Option<bool>,
    pub duration_ms: Option<u64>,
    pub timeout_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub stdout: String,
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
    let output = Command::new("pgrep").args(["-fl", pattern]).output();
    match output {
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
    let output = Command::new("sh").arg("-c").arg(&shell_command).output();
    match output {
        Ok(output) => {
            let duration_ms = Some(start.elapsed().as_millis() as u64);
            let exit_code = output.status.code();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let timed_out = matches!(exit_code, Some(124));
            let finished = !timed_out;
            let passed = if timed_out {
                None
            } else {
                Some(output.status.success())
            };
            CommandObservation {
                attempted: true,
                finished,
                passed,
                duration_ms,
                timeout_ms,
                exit_code,
                timed_out,
                stdout,
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

pub fn build_real_no_run_observation_attempt_v17(
    observation: Option<&CommandObservation>,
    timeout_ms: Option<u64>,
) -> RealNoRunObservationAttemptV17 {
    match observation {
        Some(observation) if observation.attempted => {
            let passed = observation.passed;
            let attempt_status = if observation.timed_out {
                "NoRunTimedOut"
            } else if passed == Some(true) {
                "NoRunCompleted"
            } else {
                "NoRunFailed"
            };
            RealNoRunObservationAttemptV17 {
                attempt_id: "real-no-run-observation-attempt-v17".to_string(),
                command: "cargo test --workspace --no-run --quiet".to_string(),
                attempted: true,
                started: true,
                finished: observation.finished,
                passed,
                duration_ms: observation.duration_ms,
                timeout_ms,
                exit_code: observation.exit_code,
                timed_out: observation.timed_out,
                supporting_only: true,
                attempt_status: attempt_status.to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            }
        }
        _ => RealNoRunObservationAttemptV17 {
            attempt_id: "real-no-run-observation-attempt-v17".to_string(),
            command: "cargo test --workspace --no-run --quiet".to_string(),
            attempted: false,
            started: false,
            finished: false,
            passed: None,
            duration_ms: None,
            timeout_ms,
            exit_code: None,
            timed_out: false,
            supporting_only: true,
            attempt_status: "NoRunNotRun".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        },
    }
}

pub fn build_real_full_workspace_observation_attempt_v17(
    observation: Option<&CommandObservation>,
    timeout_ms: Option<u64>,
) -> RealFullWorkspaceObservationAttemptV17 {
    match observation {
        Some(observation) if observation.attempted => {
            let passed = observation.passed;
            let full_accepted = observation.finished && passed == Some(true);
            let attempt_status = if observation.timed_out {
                "FullWorkspaceTimedOut"
            } else if full_accepted {
                "FullWorkspaceCompleted"
            } else {
                "FullWorkspaceFailed"
            };
            RealFullWorkspaceObservationAttemptV17 {
                attempt_id: "real-full-workspace-observation-attempt-v17".to_string(),
                command: "cargo test --workspace --quiet".to_string(),
                attempted: true,
                started: true,
                finished: observation.finished,
                passed,
                duration_ms: observation.duration_ms,
                timeout_ms,
                exit_code: observation.exit_code,
                timed_out: observation.timed_out,
                supporting_only: !full_accepted,
                full_accepted,
                attempt_status: attempt_status.to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            }
        }
        _ => RealFullWorkspaceObservationAttemptV17 {
            attempt_id: "real-full-workspace-observation-attempt-v17".to_string(),
            command: "cargo test --workspace --quiet".to_string(),
            attempted: false,
            started: false,
            finished: false,
            passed: None,
            duration_ms: None,
            timeout_ms,
            exit_code: None,
            timed_out: false,
            supporting_only: true,
            full_accepted: false,
            attempt_status: "FullWorkspaceNotRun".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        },
    }
}

fn parse_cargo_json_stdout(stdout: &str) -> (u64, u64, u64, Vec<String>, Vec<String>) {
    let mut parsed_count = 0;
    let mut parse_error_count = 0;
    let mut malformed_line_count = 0;
    let mut targets = Vec::new();
    let mut artifacts = Vec::new();
    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        match serde_json::from_str::<Value>(line) {
            Ok(value) => {
                parsed_count += 1;
                if let Some(target) = value
                    .get("target")
                    .and_then(|value| value.get("name"))
                    .and_then(Value::as_str)
                {
                    targets.push(target.to_string());
                }
                if let Some(filenames) = value.get("filenames").and_then(Value::as_array) {
                    for filename in filenames {
                        if let Some(filename) = filename.as_str() {
                            artifacts.push(filename.to_string());
                        }
                    }
                }
            }
            Err(_) => {
                parse_error_count += 1;
                malformed_line_count += 1;
            }
        }
    }
    (
        parsed_count,
        parse_error_count,
        malformed_line_count,
        stable_strings(targets),
        stable_strings(artifacts),
    )
}

pub fn build_real_cargo_json_observation_attempt_v17(
    observation: Option<&CommandObservation>,
    timeout_ms: Option<u64>,
) -> RealCargoJsonObservationAttemptV17 {
    match observation {
        Some(observation) if observation.attempted => {
            let (
                parsed_message_count,
                parse_error_count,
                malformed_line_count,
                last_seen_targets,
                last_seen_artifacts,
            ) = parse_cargo_json_stdout(&observation.stdout);
            let attempt_status = if observation.timed_out {
                "CargoJsonTimedOut"
            } else if parse_error_count > 0 {
                "CargoJsonParseErrorsObserved"
            } else {
                "CargoJsonCompleted"
            };
            RealCargoJsonObservationAttemptV17 {
                attempt_id: "real-cargo-json-observation-attempt-v17".to_string(),
                command: "cargo test --workspace --no-run --message-format=json".to_string(),
                attempted: true,
                started: true,
                finished: observation.finished,
                duration_ms: observation.duration_ms,
                timeout_ms,
                exit_code: observation.exit_code,
                timed_out: observation.timed_out,
                actual_json_parsing_enabled: true,
                parsed_message_count,
                parse_error_count,
                malformed_line_count,
                last_seen_targets,
                last_seen_artifacts,
                supporting_only: true,
                attempt_status: attempt_status.to_string(),
                reason_codes: diagnostic_reason_codes(&[]),
            }
        }
        _ => RealCargoJsonObservationAttemptV17 {
            attempt_id: "real-cargo-json-observation-attempt-v17".to_string(),
            command: "cargo test --workspace --no-run --message-format=json".to_string(),
            attempted: false,
            started: false,
            finished: false,
            duration_ms: None,
            timeout_ms,
            exit_code: None,
            timed_out: false,
            actual_json_parsing_enabled: true,
            parsed_message_count: 0,
            parse_error_count: 0,
            malformed_line_count: 0,
            last_seen_targets: Vec::new(),
            last_seen_artifacts: Vec::new(),
            supporting_only: true,
            attempt_status: "CargoJsonNotRun".to_string(),
            reason_codes: diagnostic_reason_codes(&[]),
        },
    }
}

pub fn build_no_run_observation_task_report_v1(
    attempt: &RealNoRunObservationAttemptV17,
) -> NoRunObservationTaskReportV1 {
    NoRunObservationTaskReportV1 {
        task_id: "no-run-observation-task-v1".to_string(),
        planned: true,
        attempted: attempt.attempted,
        completed: attempt.finished || attempt.timed_out,
        timed_out: attempt.timed_out,
        supporting_only: true,
        task_status: if attempt.attempted {
            if attempt.timed_out {
                "NoRunObservationTimedOut"
            } else {
                "NoRunObservationObserved"
            }
        } else {
            "NoRunObservationDeferred"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_full_workspace_observation_task_report_v1(
    attempt: &RealFullWorkspaceObservationAttemptV17,
) -> FullWorkspaceObservationTaskReportV1 {
    FullWorkspaceObservationTaskReportV1 {
        task_id: "full-workspace-observation-task-v1".to_string(),
        planned: true,
        attempted: attempt.attempted,
        completed: attempt.finished || attempt.timed_out,
        timed_out: attempt.timed_out,
        supporting_only: attempt.supporting_only,
        full_accepted: attempt.full_accepted,
        task_status: if attempt.full_accepted {
            "FullWorkspaceObservationAccepted"
        } else if attempt.timed_out {
            "FullWorkspaceObservationTimedOut"
        } else if attempt.attempted {
            "FullWorkspaceObservationObserved"
        } else {
            "FullWorkspaceObservationDeferred"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cargo_json_observation_task_report_v1(
    attempt: &RealCargoJsonObservationAttemptV17,
) -> CargoJsonObservationTaskReportV1 {
    CargoJsonObservationTaskReportV1 {
        task_id: "cargo-json-observation-task-v1".to_string(),
        planned: true,
        attempted: attempt.attempted,
        completed: attempt.finished || attempt.timed_out,
        supporting_only: true,
        actual_json_parsing_enabled: attempt.actual_json_parsing_enabled,
        parsed_message_count: attempt.parsed_message_count,
        task_status: if attempt.attempted {
            "CargoJsonObservationReady"
        } else {
            "CargoJsonObservationDeferred"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_no_run_timeout_boundary_report_v1(
    summary: &Sprint115SummaryFixture,
    attempt: &RealNoRunObservationAttemptV17,
    timeout_ms: Option<u64>,
) -> NoRunTimeoutBoundaryReportV1 {
    let actual_observation = attempt.attempted;
    NoRunTimeoutBoundaryReportV1 {
        report_id: "no-run-timeout-boundary-v1".to_string(),
        timeout_ms,
        exit_code: attempt.exit_code.or(summary.no_run_exit_code),
        actual_observation,
        timed_out: if actual_observation {
            attempt.timed_out
        } else {
            summary.no_run_exit_code == Some(124)
        },
        status: if actual_observation {
            "NoRunTimeoutBoundaryReady"
        } else {
            "NoRunTimeoutBoundaryCarriedForward"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_full_timeout_boundary_report_v1(
    summary: &Sprint115SummaryFixture,
    attempt: &RealFullWorkspaceObservationAttemptV17,
    timeout_ms: Option<u64>,
) -> FullTimeoutBoundaryReportV1 {
    let actual_observation = attempt.attempted;
    FullTimeoutBoundaryReportV1 {
        report_id: "full-timeout-boundary-v1".to_string(),
        timeout_ms,
        exit_code: attempt.exit_code.or(summary.full_exit_code),
        actual_observation,
        timed_out: if actual_observation {
            attempt.timed_out
        } else {
            summary.full_exit_code == Some(124)
        },
        status: if actual_observation {
            "FullTimeoutBoundaryReady"
        } else {
            "FullTimeoutBoundaryCarriedForward"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_boundary_observation_task_report_v1(
    no_run: &NoRunTimeoutBoundaryReportV1,
    full: &FullTimeoutBoundaryReportV1,
) -> TimeoutBoundaryObservationTaskReportV1 {
    let observed_timeout_boundary = no_run.actual_observation || full.actual_observation;
    TimeoutBoundaryObservationTaskReportV1 {
        task_id: "timeout-boundary-observation-task-v1".to_string(),
        observed_timeout_boundary,
        no_run_timeout_ms: no_run.timeout_ms,
        no_run_exit_code: no_run.exit_code,
        full_timeout_ms: full.timeout_ms,
        full_exit_code: full.exit_code,
        task_status: if observed_timeout_boundary {
            "TimeoutBoundaryObserved"
        } else {
            "TimeoutBoundaryCarriedForward"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_cleanup_consistency_report_v1(
    summary: &Sprint115SummaryFixture,
) -> TimeoutCleanupConsistencyReportV1 {
    build_timeout_cleanup_consistency_report_v1_with_actual_counts(summary, None, None)
}

pub fn build_timeout_cleanup_consistency_report_v1_with_actual_counts(
    summary: &Sprint115SummaryFixture,
    no_run_actual_counts: Option<(u64, u64)>,
    full_actual_counts: Option<(u64, u64)>,
) -> TimeoutCleanupConsistencyReportV1 {
    let baseline_counts = (
        summary.remaining_cargo_processes_after_timeout,
        summary.remaining_rustc_processes_after_timeout,
    );
    let no_run_counts = no_run_actual_counts.unwrap_or(baseline_counts);
    let full_counts = full_actual_counts.unwrap_or(baseline_counts);
    let consistent_with_baseline = match (no_run_actual_counts, full_actual_counts) {
        (None, None) => summary.timeout_cleanup_verified,
        _ => {
            no_run_counts == baseline_counts
                && full_counts == baseline_counts
                && summary.timeout_cleanup_verified
        }
    };
    TimeoutCleanupConsistencyReportV1 {
        report_id: "timeout-cleanup-consistency-v1".to_string(),
        no_run_remaining_cargo_processes: no_run_counts.0,
        no_run_remaining_rustc_processes: no_run_counts.1,
        full_remaining_cargo_processes: full_counts.0,
        full_remaining_rustc_processes: full_counts.1,
        consistent_with_baseline,
        timeout_cleanup_is_pass: false,
        status: if consistent_with_baseline {
            "TimeoutCleanupConsistent"
        } else {
            "TimeoutCleanupNeedsVerification"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_cleanup_consistency_task_report_v1(
    report: &TimeoutCleanupConsistencyReportV1,
) -> TimeoutCleanupConsistencyTaskReportV1 {
    TimeoutCleanupConsistencyTaskReportV1 {
        task_id: "timeout-cleanup-consistency-task-v1".to_string(),
        planned: true,
        actual_remaining_cargo_count: report.no_run_remaining_cargo_processes
            + report.full_remaining_cargo_processes,
        actual_remaining_rustc_count: report.no_run_remaining_rustc_processes
            + report.full_remaining_rustc_processes,
        cleanup_verified: report.consistent_with_baseline,
        task_status: report.status.clone(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cargo_artifact_ordering_report_v1(
    cargo_json: &RealCargoJsonObservationAttemptV17,
) -> CargoArtifactOrderingReportV1 {
    let observed_artifact_order = cargo_json.last_seen_artifacts.clone();
    let observed = cargo_json.attempted && !observed_artifact_order.is_empty();
    CargoArtifactOrderingReportV1 {
        report_id: "cargo-artifact-ordering-v1".to_string(),
        observed_artifact_order,
        deterministic_ordering_analysis:
            "artifact ordering remains diagnostic-only and does not upgrade acceptance".to_string(),
        status: if observed {
            "CargoArtifactOrderingReady"
        } else {
            "CargoArtifactOrderingDeferred"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cargo_json_artifact_ordering_report_v1(
    cargo_json: &RealCargoJsonObservationAttemptV17,
) -> CargoJsonArtifactOrderingReportV1 {
    CargoJsonArtifactOrderingReportV1 {
        report_id: "cargo-json-artifact-ordering-v1".to_string(),
        observed_artifact_order: cargo_json.last_seen_artifacts.clone(),
        status: "CargoJsonArtifactOrderingReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cargo_artifact_ordering_observation_task_report_v1(
    report: &CargoArtifactOrderingReportV1,
) -> CargoArtifactOrderingObservationTaskReportV1 {
    let deterministic = report.status == "CargoArtifactOrderingReady";
    CargoArtifactOrderingObservationTaskReportV1 {
        task_id: "cargo-artifact-ordering-observation-task-v1".to_string(),
        planned: true,
        observed_artifact_count: report.observed_artifact_order.len() as u64,
        deterministic,
        task_status: report.status.clone(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cargo_json_parse_quality_report_v1(
    cargo_json: &RealCargoJsonObservationAttemptV17,
) -> CargoJsonParseQualityReportV1 {
    CargoJsonParseQualityReportV1 {
        report_id: "cargo-json-parse-quality-v1".to_string(),
        parsed_count: cargo_json.parsed_message_count,
        parse_error_count: cargo_json.parse_error_count,
        malformed_line_count: cargo_json.malformed_line_count,
        quality_status: if cargo_json.parse_error_count == 0 {
            "CargoJsonParseQualityReady"
        } else {
            "CargoJsonParseQualityWithWarnings"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_no_run_recovery_gate_v17(
    attempt: &RealNoRunObservationAttemptV17,
    _summary: &Sprint115SummaryFixture,
) -> WorkspaceNoRunRecoveryGateV17 {
    let finished = attempt.finished;
    let passed = attempt.passed == Some(true);
    let recovered = finished && passed;
    WorkspaceNoRunRecoveryGateV17 {
        gate_id: "workspace-no-run-recovery-gate-v17".to_string(),
        command: attempt.command.clone(),
        finished,
        passed,
        timed_out: if attempt.attempted {
            attempt.timed_out
        } else {
            false
        },
        recovered,
        status: if recovered {
            "NoRunRecovered"
        } else {
            "NoRunStillBlocked"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_full_acceptance_gate_v17(
    attempt: &RealFullWorkspaceObservationAttemptV17,
    summary: &Sprint115SummaryFixture,
) -> WorkspaceFullAcceptanceGateV17 {
    let accepted = attempt.full_accepted;
    WorkspaceFullAcceptanceGateV17 {
        gate_id: "workspace-full-acceptance-gate-v17".to_string(),
        command: attempt.command.clone(),
        finished: attempt.finished,
        passed: attempt.passed == Some(true),
        accepted,
        status: if accepted {
            "FullWorkspaceAccepted"
        } else if attempt.timed_out || summary.full_workspace_status == "FullWorkspaceStillBlocked"
        {
            "FullWorkspaceStillBlocked"
        } else {
            "FullWorkspaceNeedsMoreEvidence"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_acceptance_truth_gate_v17(
    summary: &Sprint115SummaryFixture,
    no_run_gate: &WorkspaceNoRunRecoveryGateV17,
    full_gate: &WorkspaceFullAcceptanceGateV17,
) -> AcceptanceTruthGateV17 {
    let can_claim_full_acceptance = full_gate.accepted;
    AcceptanceTruthGateV17 {
        gate_id: "acceptance-truth-gate-v17".to_string(),
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
            "NoRunRecoveredSupportingOnly"
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

pub fn build_focused_vs_full_bridge_v13(
    summary: &Sprint115SummaryFixture,
    no_run_gate: &WorkspaceNoRunRecoveryGateV17,
    full_gate: &WorkspaceFullAcceptanceGateV17,
) -> FocusedVsFullBridgeV13 {
    FocusedVsFullBridgeV13 {
        bridge_id: "focused-vs-full-bridge-v13".to_string(),
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

pub fn build_workspace_timeout_evidence_matrix_v2(
    summary: &Sprint115SummaryFixture,
    no_run: &RealNoRunObservationAttemptV17,
    full: &RealFullWorkspaceObservationAttemptV17,
    cargo_json: &RealCargoJsonObservationAttemptV17,
    no_run_boundary: &NoRunTimeoutBoundaryReportV1,
    cleanup: &TimeoutCleanupConsistencyReportV1,
    artifact_ordering: &CargoArtifactOrderingReportV1,
) -> WorkspaceTimeoutEvidenceMatrixV2 {
    let full_supports_acceptance = full.full_accepted;
    let rows = vec![
        WorkspaceTimeoutEvidenceRowV2 {
            row_id: "NoRunObservation".to_string(),
            evidence_type: "NoRunObservation".to_string(),
            evidence_status: no_run.attempt_status.clone(),
            supports_acceptance: false,
            supporting_only: true,
        },
        WorkspaceTimeoutEvidenceRowV2 {
            row_id: "FullObservation".to_string(),
            evidence_type: "FullObservation".to_string(),
            evidence_status: full.attempt_status.clone(),
            supports_acceptance: full_supports_acceptance,
            supporting_only: !full_supports_acceptance,
        },
        WorkspaceTimeoutEvidenceRowV2 {
            row_id: "CargoJsonObservation".to_string(),
            evidence_type: "CargoJsonObservation".to_string(),
            evidence_status: cargo_json.attempt_status.clone(),
            supports_acceptance: false,
            supporting_only: true,
        },
        WorkspaceTimeoutEvidenceRowV2 {
            row_id: "TimeoutBoundary".to_string(),
            evidence_type: "TimeoutBoundary".to_string(),
            evidence_status: no_run_boundary.status.clone(),
            supports_acceptance: false,
            supporting_only: true,
        },
        WorkspaceTimeoutEvidenceRowV2 {
            row_id: "TimeoutCleanup".to_string(),
            evidence_type: "TimeoutCleanup".to_string(),
            evidence_status: cleanup.status.clone(),
            supports_acceptance: false,
            supporting_only: true,
        },
        WorkspaceTimeoutEvidenceRowV2 {
            row_id: "ArtifactOrdering".to_string(),
            evidence_type: "ArtifactOrdering".to_string(),
            evidence_status: artifact_ordering.status.clone(),
            supports_acceptance: false,
            supporting_only: true,
        },
        WorkspaceTimeoutEvidenceRowV2 {
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
        WorkspaceTimeoutEvidenceRowV2 {
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
        WorkspaceTimeoutEvidenceRowV2 {
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
    ];
    WorkspaceTimeoutEvidenceMatrixV2 {
        report_id: "workspace-timeout-evidence-matrix-v2".to_string(),
        evidence_rows: rows,
        status: if full_supports_acceptance {
            "WorkspaceTimeoutEvidenceMatrixReady"
        } else {
            "WorkspaceTimeoutEvidenceMatrixSupportingOnly"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_root_cause_report_v4(
    summary: &Sprint115SummaryFixture,
    cargo_json: &RealCargoJsonObservationAttemptV17,
    cleanup: &TimeoutCleanupConsistencyReportV1,
    artifact_ordering: &CargoArtifactOrderingReportV1,
) -> WorkspaceTimeoutRootCauseReportV4 {
    let mut new_timeout_evidence = vec![format!(
        "Sprint115 carried forward no-run={} and full={}",
        summary.no_run_status, summary.full_workspace_status
    )];
    if cargo_json.attempted {
        new_timeout_evidence.push(format!(
            "cargo-json parsed_count={} parse_errors={}",
            cargo_json.parsed_message_count, cargo_json.parse_error_count
        ));
    }
    WorkspaceTimeoutRootCauseReportV4 {
        report_id: "workspace-timeout-root-cause-v4".to_string(),
        previous_root_cause:
            "workspace timeout remains dominated by diagnostic-only no-run/full blockage evidence"
                .to_string(),
        new_timeout_evidence,
        new_artifact_ordering_evidence: vec![format!(
            "artifact-ordering count={}",
            artifact_ordering.observed_artifact_order.len()
        )],
        new_cleanup_evidence: vec![format!(
            "cleanup consistent={} remaining cargo={} rustc={}",
            cleanup.consistent_with_baseline,
            cleanup.no_run_remaining_cargo_processes,
            cleanup.no_run_remaining_rustc_processes
        )],
        root_cause_confidence: if cargo_json.attempted {
            "ConservativeEvidenceBacked"
        } else {
            "ConservativeCarryForward"
        }
        .to_string(),
        status: "WorkspaceTimeoutRootCauseReadyV4".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_diagnostic_track_progress_report_v1(
    activation: &WorkspaceTimeoutTrackActivationReportV1,
    burn_down: &WorkspaceTimeoutObservationBacklogBurnDownReportV1,
) -> WorkspaceTimeoutDiagnosticTrackProgressReportV1 {
    WorkspaceTimeoutDiagnosticTrackProgressReportV1 {
        report_id: "workspace-timeout-diagnostic-track-progress-v1".to_string(),
        diagnostic_track_active: activation.diagnostic_track_active,
        backlog_status: burn_down.burn_down_status.clone(),
        attempted_observations: burn_down.completed_count + burn_down.blocked_count,
        completed_observations: burn_down.completed_count,
        status: "WorkspaceTimeoutDiagnosticTrackProgressReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_track_risk_report_v1() -> WorkspaceTimeoutTrackRiskReportV1 {
    WorkspaceTimeoutTrackRiskReportV1 {
        report_id: "workspace-timeout-track-risk-v1".to_string(),
        overclaim_risk: "High if supporting-only evidence is upgraded to acceptance".to_string(),
        fixture_overwrite_risk: "Low with local deterministic output directories".to_string(),
        parse_error_risk: "Moderate when cargo JSON emits malformed lines".to_string(),
        cleanup_false_positive_risk: "Moderate if timeout cleanup is confused with pass"
            .to_string(),
        acceptance_confusion_risk: "High unless full workspace finishes and passes".to_string(),
        status: "WorkspaceTimeoutTrackRiskReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_consolidation_track_still_paused_report_v1(
    carry_forward: &ConsolidationPausedCarryForwardReport,
) -> ConsolidationTrackStillPausedReportV1 {
    ConsolidationTrackStillPausedReportV1 {
        report_id: "consolidation-track-still-paused-v1".to_string(),
        paused: carry_forward.consolidation_paused,
        stopped: carry_forward.consolidation_stopped,
        no_assertion_movement: carry_forward.no_assertion_movement,
        no_target_retirement: carry_forward.no_target_retirement,
        status: "ConsolidationStillPaused".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_fifth_patch_still_not_applied_report_v1() -> FifthPatchStillNotAppliedReportV1 {
    FifthPatchStillNotAppliedReportV1 {
        report_id: "fifth-patch-still-not-applied-v1".to_string(),
        fifth_patch_applied: false,
        no_assertions_moved: true,
        no_targets_retired: true,
        status: "FifthPatchStillNotApplied".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_assertion_movement_still_forbidden_report_v1()
-> AssertionMovementStillForbiddenReportV1 {
    AssertionMovementStillForbiddenReportV1 {
        report_id: "assertion-movement-still-forbidden-v1".to_string(),
        movement_allowed: false,
        status: "AssertionMovementStillForbidden".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_target_retirement_still_forbidden_report_v1() -> TargetRetirementStillForbiddenReportV1
{
    TargetRetirementStillForbiddenReportV1 {
        report_id: "target-retirement-still-forbidden-v1".to_string(),
        retirement_allowed: false,
        status: "TargetRetirementStillForbidden".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_acceptance_evidence_strength_report_v6(
    acceptance: &AcceptanceTruthGateV17,
) -> AcceptanceEvidenceStrengthReportV6 {
    let full_evidence_sufficient = acceptance.can_claim_full_acceptance;
    AcceptanceEvidenceStrengthReportV6 {
        report_id: "acceptance-evidence-strength-v6".to_string(),
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

pub fn build_workspace_recovery_decision_report_v6(
    no_run_gate: &WorkspaceNoRunRecoveryGateV17,
    full_gate: &WorkspaceFullAcceptanceGateV17,
) -> WorkspaceRecoveryDecisionReportV6 {
    WorkspaceRecoveryDecisionReportV6 {
        report_id: "workspace-recovery-decision-v6".to_string(),
        recommend_continue_timeout_track: !full_gate.accepted,
        recommend_stop_consolidation: true,
        recommend_no_fifth_patch: true,
        no_run_recovered: no_run_gate.recovered,
        full_workspace_accepted: full_gate.accepted,
        status: if full_gate.accepted {
            "WorkspaceTimeoutTrackExecuted"
        } else {
            "WorkspaceTimeoutTrackNeedsMoreObservation"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_track_next_action_queue_v1(
    full_gate: &WorkspaceFullAcceptanceGateV17,
    cargo_json: &RealCargoJsonObservationAttemptV17,
) -> TimeoutTrackNextActionQueueV1 {
    let mut next_actions = vec![
        "repeat no-run observation with explicit timeout boundary capture".to_string(),
        "repeat full workspace observation with explicit timeout boundary capture".to_string(),
        "preserve cleanup counts after each timed observation".to_string(),
    ];
    if !cargo_json.attempted {
        next_actions
            .push("run cargo JSON observation and parse actual compiler messages".to_string());
    }
    if !full_gate.accepted {
        next_actions.push(
            "keep acceptance warnings until full cargo test --workspace --quiet finishes and passes"
                .to_string(),
        );
    }
    TimeoutTrackNextActionQueueV1 {
        queue_id: "timeout-track-next-action-queue-v1".to_string(),
        next_actions,
        status: "TimeoutTrackNextActionsReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_control_tower_workspace_timeout_track_execution_panel(
    burn_down: &WorkspaceTimeoutObservationBacklogBurnDownReportV1,
    no_run: &NoRunObservationTaskReportV1,
    full: &FullWorkspaceObservationTaskReportV1,
    cargo_json: &CargoJsonObservationTaskReportV1,
    cleanup: &TimeoutCleanupConsistencyTaskReportV1,
    artifact_ordering: &CargoArtifactOrderingObservationTaskReportV1,
    root_cause: &WorkspaceTimeoutRootCauseReportV4,
    next_actions: &TimeoutTrackNextActionQueueV1,
) -> ControlTowerWorkspaceTimeoutTrackExecutionPanel {
    ControlTowerWorkspaceTimeoutTrackExecutionPanel {
        panel_id: "control-tower-workspace-timeout-track-execution".to_string(),
        backlog_burn_down_status: burn_down.burn_down_status.clone(),
        no_run_status: no_run.task_status.clone(),
        full_status: full.task_status.clone(),
        cargo_json_status: cargo_json.task_status.clone(),
        cleanup_status: cleanup.task_status.clone(),
        artifact_ordering_status: artifact_ordering.task_status.clone(),
        root_cause_status: root_cause.status.clone(),
        next_actions: next_actions.next_actions.clone(),
        warnings: warning_posture(),
        static_read_only: true,
        no_run_button: true,
        no_action_button: true,
        no_train_runtime_live_order_account_controls: true,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_control_tower_acceptance_truth_panel_v17(
    no_run_gate: &WorkspaceNoRunRecoveryGateV17,
    full_gate: &WorkspaceFullAcceptanceGateV17,
    acceptance: &AcceptanceTruthGateV17,
) -> ControlTowerAcceptanceTruthPanelV17 {
    ControlTowerAcceptanceTruthPanelV17 {
        panel_id: "control-tower-acceptance-truth-v17".to_string(),
        no_run_gate_status: no_run_gate.status.clone(),
        full_gate_status: full_gate.status.clone(),
        acceptance_truth_status: acceptance.status.clone(),
        supporting_only_evidence: vec![
            "focused-tests".to_string(),
            "cli-smoke".to_string(),
            "cargo-build".to_string(),
            "no-run".to_string(),
            "cargo-json".to_string(),
            "timeout-cleanup".to_string(),
        ],
        warnings: warning_posture(),
        static_read_only: true,
        no_action_button: true,
        no_train_runtime_live_order_account_controls: true,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_safety_coverage_preservation_report_v32() -> SafetyCoveragePreservationReportV32 {
    SafetyCoveragePreservationReportV32 {
        report_id: "safety-coverage-preservation-v32".to_string(),
        no_assertion_deletion: true,
        no_safety_sentinel_deletion: true,
        no_hidden_skips: true,
        timeout_track_execution_guard_present: true,
        consolidation_paused_guard_present: true,
        fifth_patch_still_not_applied_guard_present: true,
        assertion_movement_forbidden_guard_present: true,
        target_retirement_forbidden_guard_present: true,
        actual_cargo_json_parse_guard_present: true,
        actual_timeout_cleanup_counts_guard_present: true,
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

pub fn build_workspace_timeout_observation_backlog_burn_down_report_v1(
    plan: &WorkspaceTimeoutObservationBacklogBurnDownPlanV1,
    no_run: &NoRunObservationTaskReportV1,
    full: &FullWorkspaceObservationTaskReportV1,
    cargo_json: &CargoJsonObservationTaskReportV1,
    cleanup: &TimeoutCleanupConsistencyTaskReportV1,
    artifact_ordering: &CargoArtifactOrderingObservationTaskReportV1,
) -> WorkspaceTimeoutObservationBacklogBurnDownReportV1 {
    let tasks = [
        no_run.completed,
        full.completed,
        cargo_json.completed,
        cleanup.cleanup_verified,
        artifact_ordering.deterministic,
    ];
    let completed_count = tasks.iter().filter(|value| **value).count() as u64;
    let deferred_count = plan.planned_tasks.len() as u64 - completed_count;
    WorkspaceTimeoutObservationBacklogBurnDownReportV1 {
        report_id: "workspace-timeout-observation-backlog-burndown-v1".to_string(),
        planned_count: plan.planned_tasks.len() as u64,
        completed_count,
        skipped_count: 0,
        blocked_count: if full.timed_out { 1 } else { 0 },
        deferred_count,
        remaining_count: plan.planned_tasks.len() as u64 - completed_count,
        burn_down_status: if completed_count == plan.planned_tasks.len() as u64 {
            "BacklogReduced"
        } else if completed_count > 0 {
            "BacklogReducedWithWarnings"
        } else {
            "BacklogStillOpen"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

impl WorkspaceTimeoutTrackExecutionBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            (
                "## 1. Sprint summary",
                format!(
                    "activation={} backlog={} acceptance={} decision={}",
                    self.workspace_timeout_track_activation_report_v1.activation_status,
                    self.workspace_timeout_observation_backlog_burn_down_report_v1
                        .burn_down_status,
                    self.acceptance_truth_gate_v17.status,
                    self.workspace_recovery_decision_report_v6.status,
                ),
            ),
            (
                "## 2. Why Sprint 116 was needed",
                "Sprint 115 paused consolidation and left the workspace timeout track active, so Sprint 116 executes diagnostics without resuming consolidation.".to_string(),
            ),
            (
                "## 3. Files added",
                "Sprint 116 adds local timeout-track execution reports, fixtures, docs, and tests only.".to_string(),
            ),
            (
                "## 4. Files changed",
                "Changes stay within Sprint 116 timeout-track/report/CLI/test/docs surfaces and preserve Sprint 115 governance truth.".to_string(),
            ),
            (
                "## 5. Sprint 115 baseline truth import",
                format!(
                    "import_status={} imported_as_full_acceptance={}",
                    self.sprint115_baseline_truth_import_report.import_status,
                    self.sprint115_baseline_truth_import_report
                        .imported_as_full_acceptance,
                ),
            ),
            (
                "## 6. Consolidation paused carry-forward",
                format!(
                    "status={} paused={} stopped={}",
                    self.consolidation_paused_carry_forward_report
                        .carry_forward_status,
                    self.consolidation_paused_carry_forward_report.consolidation_paused,
                    self.consolidation_paused_carry_forward_report.consolidation_stopped,
                ),
            ),
            (
                "## 7. Workspace timeout track activation",
                format!(
                    "activation_status={} diagnostic_track_active={} separated={}",
                    self.workspace_timeout_track_activation_report_v1.activation_status,
                    self.workspace_timeout_track_activation_report_v1
                        .diagnostic_track_active,
                    self.workspace_timeout_track_activation_report_v1
                        .consolidation_track_separated,
                ),
            ),
            (
                "## 8. Observation backlog import",
                format!(
                    "import_status={} imported_items={}",
                    self.workspace_timeout_observation_backlog_import_report_v1
                        .import_status,
                    self.workspace_timeout_observation_backlog_import_report_v1
                        .backlog_items_imported
                        .len(),
                ),
            ),
            (
                "## 9. Observation backlog burn-down plan",
                format!(
                    "plan_status={} planned_tasks={}",
                    self.workspace_timeout_observation_backlog_burn_down_plan_v1
                        .plan_status,
                    self.workspace_timeout_observation_backlog_burn_down_plan_v1
                        .planned_tasks
                        .len(),
                ),
            ),
            (
                "## 10. Observation backlog burn-down report",
                format!(
                    "status={} completed={} remaining={}",
                    self.workspace_timeout_observation_backlog_burn_down_report_v1
                        .burn_down_status,
                    self.workspace_timeout_observation_backlog_burn_down_report_v1
                        .completed_count,
                    self.workspace_timeout_observation_backlog_burn_down_report_v1
                        .remaining_count,
                ),
            ),
            (
                "## 11. No-run observation task",
                format!(
                    "task_status={} attempted={} supporting_only={}",
                    self.no_run_observation_task_report_v1.task_status,
                    self.no_run_observation_task_report_v1.attempted,
                    self.no_run_observation_task_report_v1.supporting_only,
                ),
            ),
            (
                "## 12. Full workspace observation task",
                format!(
                    "task_status={} attempted={} full_accepted={}",
                    self.full_workspace_observation_task_report_v1.task_status,
                    self.full_workspace_observation_task_report_v1.attempted,
                    self.full_workspace_observation_task_report_v1.full_accepted,
                ),
            ),
            (
                "## 13. Cargo JSON observation task",
                format!(
                    "task_status={} attempted={} parsed={}",
                    self.cargo_json_observation_task_report_v1.task_status,
                    self.cargo_json_observation_task_report_v1.attempted,
                    self.cargo_json_observation_task_report_v1
                        .parsed_message_count,
                ),
            ),
            (
                "## 14. Timeout boundary observation",
                format!(
                    "task_status={} observed={}",
                    self.timeout_boundary_observation_task_report_v1.task_status,
                    self.timeout_boundary_observation_task_report_v1
                        .observed_timeout_boundary,
                ),
            ),
            (
                "## 15. Timeout cleanup consistency task",
                format!(
                    "task_status={} cargo={} rustc={}",
                    self.timeout_cleanup_consistency_task_report_v1.task_status,
                    self.timeout_cleanup_consistency_task_report_v1
                        .actual_remaining_cargo_count,
                    self.timeout_cleanup_consistency_task_report_v1
                        .actual_remaining_rustc_count,
                ),
            ),
            (
                "## 16. Cargo artifact ordering observation",
                format!(
                    "task_status={} observed_artifact_count={}",
                    self.cargo_artifact_ordering_observation_task_report_v1
                        .task_status,
                    self.cargo_artifact_ordering_observation_task_report_v1
                        .observed_artifact_count,
                ),
            ),
            (
                "## 17. Real no-run observation attempt v17",
                format!(
                    "attempt_status={} attempted={} timed_out={}",
                    self.real_no_run_observation_attempt_v17.attempt_status,
                    self.real_no_run_observation_attempt_v17.attempted,
                    self.real_no_run_observation_attempt_v17.timed_out,
                ),
            ),
            (
                "## 18. Real full workspace observation attempt v17",
                format!(
                    "attempt_status={} attempted={} full_accepted={}",
                    self.real_full_workspace_observation_attempt_v17
                        .attempt_status,
                    self.real_full_workspace_observation_attempt_v17.attempted,
                    self.real_full_workspace_observation_attempt_v17
                        .full_accepted,
                ),
            ),
            (
                "## 19. Real cargo JSON observation attempt v17",
                format!(
                    "attempt_status={} attempted={} parsed={}",
                    self.real_cargo_json_observation_attempt_v17.attempt_status,
                    self.real_cargo_json_observation_attempt_v17.attempted,
                    self.real_cargo_json_observation_attempt_v17
                        .parsed_message_count,
                ),
            ),
            (
                "## 20. No-run / full timeout boundary reports",
                format!(
                    "no_run={} full={}",
                    self.no_run_timeout_boundary_report_v1.status,
                    self.full_timeout_boundary_report_v1.status,
                ),
            ),
            (
                "## 21. Timeout cleanup consistency",
                format!(
                    "status={} timeout_cleanup_is_pass={}",
                    self.timeout_cleanup_consistency_report_v1.status,
                    self.timeout_cleanup_consistency_report_v1
                        .timeout_cleanup_is_pass,
                ),
            ),
            (
                "## 22. Cargo artifact ordering",
                format!("status={}", self.cargo_artifact_ordering_report_v1.status),
            ),
            (
                "## 23. Cargo JSON artifact ordering",
                format!(
                    "status={} observed_artifacts={}",
                    self.cargo_json_artifact_ordering_report_v1.status,
                    self.cargo_json_artifact_ordering_report_v1
                        .observed_artifact_order
                        .len(),
                ),
            ),
            (
                "## 24. Cargo JSON parse quality",
                format!(
                    "quality_status={} parsed={} errors={}",
                    self.cargo_json_parse_quality_report_v1.quality_status,
                    self.cargo_json_parse_quality_report_v1.parsed_count,
                    self.cargo_json_parse_quality_report_v1.parse_error_count,
                ),
            ),
            (
                "## 25. Workspace timeout evidence matrix v2",
                format!(
                    "status={} rows={}",
                    self.workspace_timeout_evidence_matrix_v2.status,
                    self.workspace_timeout_evidence_matrix_v2.evidence_rows.len(),
                ),
            ),
            (
                "## 26. Workspace timeout root-cause v4",
                format!(
                    "status={} confidence={}",
                    self.workspace_timeout_root_cause_report_v4.status,
                    self.workspace_timeout_root_cause_report_v4
                        .root_cause_confidence,
                ),
            ),
            (
                "## 27. Workspace timeout diagnostic track progress",
                format!(
                    "status={} attempted={} completed={}",
                    self.workspace_timeout_diagnostic_track_progress_report_v1.status,
                    self.workspace_timeout_diagnostic_track_progress_report_v1
                        .attempted_observations,
                    self.workspace_timeout_diagnostic_track_progress_report_v1
                        .completed_observations,
                ),
            ),
            (
                "## 28. Workspace timeout track risk",
                format!("status={}", self.workspace_timeout_track_risk_report_v1.status),
            ),
            (
                "## 29. Consolidation track still paused",
                format!(
                    "status={} paused={}",
                    self.consolidation_track_still_paused_report_v1.status,
                    self.consolidation_track_still_paused_report_v1.paused,
                ),
            ),
            (
                "## 30. Fifth patch still not applied",
                format!(
                    "status={} applied={}",
                    self.fifth_patch_still_not_applied_report_v1.status,
                    self.fifth_patch_still_not_applied_report_v1
                        .fifth_patch_applied,
                ),
            ),
            (
                "## 31. Assertion movement still forbidden",
                format!(
                    "status={} movement_allowed={}",
                    self.assertion_movement_still_forbidden_report_v1.status,
                    self.assertion_movement_still_forbidden_report_v1
                        .movement_allowed,
                ),
            ),
            (
                "## 32. Target retirement still forbidden",
                format!(
                    "status={} retirement_allowed={}",
                    self.target_retirement_still_forbidden_report_v1.status,
                    self.target_retirement_still_forbidden_report_v1
                        .retirement_allowed,
                ),
            ),
            (
                "## 33. Acceptance truth gate v17",
                format!(
                    "status={} can_claim_full_acceptance={}",
                    self.acceptance_truth_gate_v17.status,
                    self.acceptance_truth_gate_v17.can_claim_full_acceptance,
                ),
            ),
            (
                "## 34. Focused-vs-full bridge v13",
                format!("status={}", self.focused_vs_full_bridge_v13.status),
            ),
            (
                "## 35. Workspace no-run recovery gate v17",
                format!(
                    "status={} recovered={} timed_out={}",
                    self.workspace_no_run_recovery_gate_v17.status,
                    self.workspace_no_run_recovery_gate_v17.recovered,
                    self.workspace_no_run_recovery_gate_v17.timed_out,
                ),
            ),
            (
                "## 36. Workspace full acceptance gate v17",
                format!(
                    "status={} accepted={}",
                    self.workspace_full_acceptance_gate_v17.status,
                    self.workspace_full_acceptance_gate_v17.accepted,
                ),
            ),
            (
                "## 37. Acceptance evidence strength v6",
                format!(
                    "status={} strongest_claim={}",
                    self.acceptance_evidence_strength_report_v6.status,
                    self.acceptance_evidence_strength_report_v6.strongest_claim,
                ),
            ),
            (
                "## 38. Workspace recovery decision v6",
                format!(
                    "status={} continue_timeout_track={} no_fifth_patch={}",
                    self.workspace_recovery_decision_report_v6.status,
                    self.workspace_recovery_decision_report_v6
                        .recommend_continue_timeout_track,
                    self.workspace_recovery_decision_report_v6
                        .recommend_no_fifth_patch,
                ),
            ),
            (
                "## 39. Timeout track next action queue",
                format!(
                    "status={} next_actions={}",
                    self.timeout_track_next_action_queue_v1.status,
                    self.timeout_track_next_action_queue_v1.next_actions.len(),
                ),
            ),
            (
                "## 40. Safety coverage preservation v32",
                format!(
                    "safety_status={}",
                    self.safety_coverage_preservation_report_v32.safety_status,
                ),
            ),
            (
                "## 41. Control Tower workspace timeout track execution panel",
                format!(
                    "read_only={} no_action_button={} no_run_button={}",
                    self.control_tower_workspace_timeout_track_execution_panel
                        .static_read_only,
                    self.control_tower_workspace_timeout_track_execution_panel
                        .no_action_button,
                    self.control_tower_workspace_timeout_track_execution_panel
                        .no_run_button,
                ),
            ),
            (
                "## 42. Control Tower acceptance truth panel v17",
                format!(
                    "read_only={} no_action_button={} acceptance={}",
                    self.control_tower_acceptance_truth_panel_v17.static_read_only,
                    self.control_tower_acceptance_truth_panel_v17
                        .no_action_button,
                    self.control_tower_acceptance_truth_panel_v17
                        .acceptance_truth_status,
                ),
            ),
            (
                "## 43. Output bundle",
                format!("file_count={}", self.storage_report.file_count),
            ),
            (
                "## 44. CLI and examples",
                "Sprint 116 CLI examples remain local-output, timeout-track-only, consolidation-paused, and report-only.".to_string(),
            ),
            (
                "## 45. Tests added",
                "Focused tests cover config, Sprint 115 import, paused carry-forward, track activation, backlog, observations, cargo JSON, cleanup, evidence matrix, acceptance truth, panels, CLI safety, and determinism.".to_string(),
            ),
            (
                "## 46. Test results",
                "Generated summary records implementation evidence only; command execution results must be reported by the verifier after running tests.".to_string(),
            ),
            (
                "## 47. Timeout track execution status",
                format!(
                    "status={}",
                    self.workspace_timeout_diagnostic_track_progress_report_v1.status,
                ),
            ),
            (
                "## 48. Observation backlog status",
                format!(
                    "status={}",
                    self.workspace_timeout_observation_backlog_burn_down_report_v1
                        .burn_down_status,
                ),
            ),
            (
                "## 49. No-run recovery status",
                format!("status={}", self.workspace_no_run_recovery_gate_v17.status),
            ),
            (
                "## 50. Full workspace acceptance status",
                format!(
                    "status={}",
                    self.workspace_full_acceptance_gate_v17.status,
                ),
            ),
            (
                "## 51. Acceptance evidence strength status",
                format!("status={}", self.acceptance_evidence_strength_report_v6.status),
            ),
            (
                "## 52. Consolidation status",
                format!(
                    "paused={} stopped={}",
                    self.consolidation_track_still_paused_report_v1.paused,
                    self.consolidation_track_still_paused_report_v1.stopped,
                ),
            ),
            (
                "## 53. Fifth patch status",
                format!("status={}", self.fifth_patch_still_not_applied_report_v1.status),
            ),
            (
                "## 54. Runtime deferred status",
                "Runtime, training, live inference, live trading, broker/order/account, runtime LLM, Mamba, and Gated runtime remain deferred or forbidden.".to_string(),
            ),
            (
                "## 55. Workspace acceptance truth status",
                format!(
                    "status={} can_claim_full_acceptance={}",
                    self.acceptance_truth_gate_v17.status,
                    self.acceptance_truth_gate_v17.can_claim_full_acceptance,
                ),
            ),
            (
                "## 56. Safety coverage status",
                format!(
                    "status={}",
                    self.safety_coverage_preservation_report_v32.safety_status,
                ),
            ),
            (
                "## 57. Risk review",
                "No consolidation resume, fifth patch, assertion movement, target retirement, hidden skip, fake timing, fake pass/fail, or acceptance overclaim is made.".to_string(),
            ),
            (
                "## 58. Deferred items",
                "Runtime/training/live/order/account/dashboard/browser/Tauri/Svelte/live-agent activation remain out of scope.".to_string(),
            ),
            (
                "## 59. Next gstack sprint recommendation",
                "Continue timeout-track observation with real no-run/full/cargo-JSON attempts when explicitly configured; keep consolidation paused until full acceptance evidence improves.".to_string(),
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
    ) -> Result<WorkspaceTimeoutTrackExecutionStorageReport, String> {
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
            "sprint115_baseline_truth_import.txt",
            self.sprint115_baseline_truth_import_report
        );
        write_report!(
            "consolidation_paused_carry_forward.txt",
            self.consolidation_paused_carry_forward_report
        );
        write_report!(
            "workspace_timeout_track_activation_v1.txt",
            self.workspace_timeout_track_activation_report_v1
        );
        write_report!(
            "workspace_timeout_observation_backlog_import_v1.txt",
            self.workspace_timeout_observation_backlog_import_report_v1
        );
        write_report!(
            "workspace_timeout_observation_backlog_burndown_plan_v1.txt",
            self.workspace_timeout_observation_backlog_burn_down_plan_v1
        );
        write_report!(
            "workspace_timeout_observation_backlog_burndown_v1.txt",
            self.workspace_timeout_observation_backlog_burn_down_report_v1
        );
        write_report!(
            "no_run_observation_task_v1.txt",
            self.no_run_observation_task_report_v1
        );
        write_report!(
            "full_workspace_observation_task_v1.txt",
            self.full_workspace_observation_task_report_v1
        );
        write_report!(
            "cargo_json_observation_task_v1.txt",
            self.cargo_json_observation_task_report_v1
        );
        write_report!(
            "timeout_boundary_observation_task_v1.txt",
            self.timeout_boundary_observation_task_report_v1
        );
        write_report!(
            "timeout_cleanup_consistency_task_v1.txt",
            self.timeout_cleanup_consistency_task_report_v1
        );
        write_report!(
            "cargo_artifact_ordering_observation_task_v1.txt",
            self.cargo_artifact_ordering_observation_task_report_v1
        );
        write_report!(
            "real_no_run_observation_attempt_v17.txt",
            self.real_no_run_observation_attempt_v17
        );
        write_report!(
            "real_full_workspace_observation_attempt_v17.txt",
            self.real_full_workspace_observation_attempt_v17
        );
        write_report!(
            "real_cargo_json_observation_attempt_v17.txt",
            self.real_cargo_json_observation_attempt_v17
        );
        write_report!(
            "no_run_timeout_boundary_v1.txt",
            self.no_run_timeout_boundary_report_v1
        );
        write_report!(
            "full_timeout_boundary_v1.txt",
            self.full_timeout_boundary_report_v1
        );
        write_report!(
            "timeout_cleanup_consistency_v1.txt",
            self.timeout_cleanup_consistency_report_v1
        );
        write_report!(
            "cargo_artifact_ordering_v1.txt",
            self.cargo_artifact_ordering_report_v1
        );
        write_report!(
            "cargo_json_artifact_ordering_v1.txt",
            self.cargo_json_artifact_ordering_report_v1
        );
        write_report!(
            "cargo_json_parse_quality_v1.txt",
            self.cargo_json_parse_quality_report_v1
        );
        write_report!(
            "workspace_timeout_evidence_matrix_v2.txt",
            self.workspace_timeout_evidence_matrix_v2
        );
        write_report!(
            "workspace_timeout_root_cause_v4.txt",
            self.workspace_timeout_root_cause_report_v4
        );
        write_report!(
            "workspace_timeout_diagnostic_track_progress_v1.txt",
            self.workspace_timeout_diagnostic_track_progress_report_v1
        );
        write_report!(
            "workspace_timeout_track_risk_v1.txt",
            self.workspace_timeout_track_risk_report_v1
        );
        write_report!(
            "consolidation_track_still_paused_v1.txt",
            self.consolidation_track_still_paused_report_v1
        );
        write_report!(
            "fifth_patch_still_not_applied_v1.txt",
            self.fifth_patch_still_not_applied_report_v1
        );
        write_report!(
            "assertion_movement_still_forbidden_v1.txt",
            self.assertion_movement_still_forbidden_report_v1
        );
        write_report!(
            "target_retirement_still_forbidden_v1.txt",
            self.target_retirement_still_forbidden_report_v1
        );
        write_report!(
            "acceptance_truth_gate_v17.txt",
            self.acceptance_truth_gate_v17
        );
        write_report!(
            "focused_vs_full_bridge_v13.txt",
            self.focused_vs_full_bridge_v13
        );
        write_report!(
            "workspace_no_run_recovery_gate_v17.txt",
            self.workspace_no_run_recovery_gate_v17
        );
        write_report!(
            "workspace_full_acceptance_gate_v17.txt",
            self.workspace_full_acceptance_gate_v17
        );
        write_report!(
            "acceptance_evidence_strength_v6.txt",
            self.acceptance_evidence_strength_report_v6
        );
        write_report!(
            "workspace_recovery_decision_v6.txt",
            self.workspace_recovery_decision_report_v6
        );
        write_report!(
            "timeout_track_next_action_queue_v1.txt",
            self.timeout_track_next_action_queue_v1
        );
        write_report!(
            "safety_coverage_preservation_v32.txt",
            self.safety_coverage_preservation_report_v32
        );
        write_report!(
            "control_tower_workspace_timeout_track_execution_panel.txt",
            self.control_tower_workspace_timeout_track_execution_panel
        );
        write_report!(
            "control_tower_acceptance_truth_panel_v17.txt",
            self.control_tower_acceptance_truth_panel_v17
        );
        let storage_report = WorkspaceTimeoutTrackExecutionStorageReport {
            report_id: "workspace-timeout-track-execution-storage-report".to_string(),
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
        Ok(WorkspaceTimeoutTrackExecutionStorageReport {
            written_files,
            ..storage_report
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct WorkspaceTimeoutTrackExecutionRunner;

impl WorkspaceTimeoutTrackExecutionRunner {
    pub fn run(
        &self,
        config: &WorkspaceTimeoutTrackExecutionConfig,
    ) -> Result<WorkspaceTimeoutTrackExecutionBundle, String> {
        config.validate()?;
        let summary = load_first_json::<Sprint115SummaryFixture>(
            config
                .sprint115_truth_paths
                .as_ref()
                .or(config.sprint115_bundle_paths.as_ref()),
        )?
        .unwrap_or_default();
        let sprint115_baseline_truth_import_report =
            build_sprint115_baseline_truth_import_report(&summary);
        let consolidation_paused_carry_forward_report =
            build_consolidation_paused_carry_forward_report(&summary);
        let workspace_timeout_observation_backlog_import_report_v1 =
            build_workspace_timeout_observation_backlog_import_report_v1();
        let workspace_timeout_track_activation_report_v1 =
            build_workspace_timeout_track_activation_report_v1(
                &summary,
                !workspace_timeout_observation_backlog_import_report_v1
                    .backlog_items_imported
                    .is_empty(),
            );
        let workspace_timeout_observation_backlog_burn_down_plan_v1 =
            build_workspace_timeout_observation_backlog_burn_down_plan_v1(
                &workspace_timeout_observation_backlog_import_report_v1,
            );

        let no_run_observation = if config.run_real_no_run_observation {
            Some(run_timed_command(
                "cargo test --workspace --no-run --quiet",
                config.no_run_timeout_ms,
            ))
        } else {
            None
        };
        let full_observation = if config.run_real_full_observation {
            Some(run_timed_command(
                "cargo test --workspace --quiet",
                config.full_timeout_ms,
            ))
        } else {
            None
        };
        let cargo_json_observation = if config.run_real_cargo_json_observation {
            Some(run_timed_command(
                "cargo test --workspace --no-run --message-format=json",
                config.cargo_json_timeout_ms,
            ))
        } else {
            None
        };
        let no_run_cleanup_counts = cleanup_counts_after_observation(&no_run_observation);
        let full_cleanup_counts = cleanup_counts_after_observation(&full_observation);

        let real_no_run_observation_attempt_v17 = build_real_no_run_observation_attempt_v17(
            no_run_observation.as_ref(),
            config.no_run_timeout_ms,
        );
        let real_full_workspace_observation_attempt_v17 =
            build_real_full_workspace_observation_attempt_v17(
                full_observation.as_ref(),
                config.full_timeout_ms,
            );
        let real_cargo_json_observation_attempt_v17 = build_real_cargo_json_observation_attempt_v17(
            cargo_json_observation.as_ref(),
            config.cargo_json_timeout_ms,
        );
        let no_run_observation_task_report_v1 =
            build_no_run_observation_task_report_v1(&real_no_run_observation_attempt_v17);
        let full_workspace_observation_task_report_v1 =
            build_full_workspace_observation_task_report_v1(
                &real_full_workspace_observation_attempt_v17,
            );
        let cargo_json_observation_task_report_v1 =
            build_cargo_json_observation_task_report_v1(&real_cargo_json_observation_attempt_v17);
        let no_run_timeout_boundary_report_v1 = build_no_run_timeout_boundary_report_v1(
            &summary,
            &real_no_run_observation_attempt_v17,
            config.no_run_timeout_ms,
        );
        let full_timeout_boundary_report_v1 = build_full_timeout_boundary_report_v1(
            &summary,
            &real_full_workspace_observation_attempt_v17,
            config.full_timeout_ms,
        );
        let timeout_boundary_observation_task_report_v1 =
            build_timeout_boundary_observation_task_report_v1(
                &no_run_timeout_boundary_report_v1,
                &full_timeout_boundary_report_v1,
            );
        let timeout_cleanup_consistency_report_v1 =
            build_timeout_cleanup_consistency_report_v1_with_actual_counts(
                &summary,
                no_run_cleanup_counts,
                full_cleanup_counts,
            );
        let timeout_cleanup_consistency_task_report_v1 =
            build_timeout_cleanup_consistency_task_report_v1(
                &timeout_cleanup_consistency_report_v1,
            );
        let cargo_artifact_ordering_report_v1 =
            build_cargo_artifact_ordering_report_v1(&real_cargo_json_observation_attempt_v17);
        let cargo_json_artifact_ordering_report_v1 =
            build_cargo_json_artifact_ordering_report_v1(&real_cargo_json_observation_attempt_v17);
        let cargo_artifact_ordering_observation_task_report_v1 =
            build_cargo_artifact_ordering_observation_task_report_v1(
                &cargo_artifact_ordering_report_v1,
            );
        let cargo_json_parse_quality_report_v1 =
            build_cargo_json_parse_quality_report_v1(&real_cargo_json_observation_attempt_v17);
        let workspace_timeout_observation_backlog_burn_down_report_v1 =
            build_workspace_timeout_observation_backlog_burn_down_report_v1(
                &workspace_timeout_observation_backlog_burn_down_plan_v1,
                &no_run_observation_task_report_v1,
                &full_workspace_observation_task_report_v1,
                &cargo_json_observation_task_report_v1,
                &timeout_cleanup_consistency_task_report_v1,
                &cargo_artifact_ordering_observation_task_report_v1,
            );
        let workspace_no_run_recovery_gate_v17 = build_workspace_no_run_recovery_gate_v17(
            &real_no_run_observation_attempt_v17,
            &summary,
        );
        let workspace_full_acceptance_gate_v17 = build_workspace_full_acceptance_gate_v17(
            &real_full_workspace_observation_attempt_v17,
            &summary,
        );
        let acceptance_truth_gate_v17 = build_acceptance_truth_gate_v17(
            &summary,
            &workspace_no_run_recovery_gate_v17,
            &workspace_full_acceptance_gate_v17,
        );
        let focused_vs_full_bridge_v13 = build_focused_vs_full_bridge_v13(
            &summary,
            &workspace_no_run_recovery_gate_v17,
            &workspace_full_acceptance_gate_v17,
        );
        let workspace_timeout_evidence_matrix_v2 = build_workspace_timeout_evidence_matrix_v2(
            &summary,
            &real_no_run_observation_attempt_v17,
            &real_full_workspace_observation_attempt_v17,
            &real_cargo_json_observation_attempt_v17,
            &no_run_timeout_boundary_report_v1,
            &timeout_cleanup_consistency_report_v1,
            &cargo_artifact_ordering_report_v1,
        );
        let workspace_timeout_root_cause_report_v4 = build_workspace_timeout_root_cause_report_v4(
            &summary,
            &real_cargo_json_observation_attempt_v17,
            &timeout_cleanup_consistency_report_v1,
            &cargo_artifact_ordering_report_v1,
        );
        let workspace_timeout_diagnostic_track_progress_report_v1 =
            build_workspace_timeout_diagnostic_track_progress_report_v1(
                &workspace_timeout_track_activation_report_v1,
                &workspace_timeout_observation_backlog_burn_down_report_v1,
            );
        let workspace_timeout_track_risk_report_v1 = build_workspace_timeout_track_risk_report_v1();
        let consolidation_track_still_paused_report_v1 =
            build_consolidation_track_still_paused_report_v1(
                &consolidation_paused_carry_forward_report,
            );
        let fifth_patch_still_not_applied_report_v1 =
            build_fifth_patch_still_not_applied_report_v1();
        let assertion_movement_still_forbidden_report_v1 =
            build_assertion_movement_still_forbidden_report_v1();
        let target_retirement_still_forbidden_report_v1 =
            build_target_retirement_still_forbidden_report_v1();
        let acceptance_evidence_strength_report_v6 =
            build_acceptance_evidence_strength_report_v6(&acceptance_truth_gate_v17);
        let workspace_recovery_decision_report_v6 = build_workspace_recovery_decision_report_v6(
            &workspace_no_run_recovery_gate_v17,
            &workspace_full_acceptance_gate_v17,
        );
        let timeout_track_next_action_queue_v1 = build_timeout_track_next_action_queue_v1(
            &workspace_full_acceptance_gate_v17,
            &real_cargo_json_observation_attempt_v17,
        );
        let safety_coverage_preservation_report_v32 =
            build_safety_coverage_preservation_report_v32();
        let control_tower_workspace_timeout_track_execution_panel =
            build_control_tower_workspace_timeout_track_execution_panel(
                &workspace_timeout_observation_backlog_burn_down_report_v1,
                &no_run_observation_task_report_v1,
                &full_workspace_observation_task_report_v1,
                &cargo_json_observation_task_report_v1,
                &timeout_cleanup_consistency_task_report_v1,
                &cargo_artifact_ordering_observation_task_report_v1,
                &workspace_timeout_root_cause_report_v4,
                &timeout_track_next_action_queue_v1,
            );
        let control_tower_acceptance_truth_panel_v17 =
            build_control_tower_acceptance_truth_panel_v17(
                &workspace_no_run_recovery_gate_v17,
                &workspace_full_acceptance_gate_v17,
                &acceptance_truth_gate_v17,
            );

        let mut bundle = WorkspaceTimeoutTrackExecutionBundle {
            sprint115_baseline_truth_import_report,
            consolidation_paused_carry_forward_report,
            workspace_timeout_track_activation_report_v1,
            workspace_timeout_observation_backlog_import_report_v1,
            workspace_timeout_observation_backlog_burn_down_plan_v1,
            workspace_timeout_observation_backlog_burn_down_report_v1,
            no_run_observation_task_report_v1,
            full_workspace_observation_task_report_v1,
            cargo_json_observation_task_report_v1,
            timeout_boundary_observation_task_report_v1,
            timeout_cleanup_consistency_task_report_v1,
            cargo_artifact_ordering_observation_task_report_v1,
            real_no_run_observation_attempt_v17,
            real_full_workspace_observation_attempt_v17,
            real_cargo_json_observation_attempt_v17,
            no_run_timeout_boundary_report_v1,
            full_timeout_boundary_report_v1,
            timeout_cleanup_consistency_report_v1,
            cargo_artifact_ordering_report_v1,
            cargo_json_artifact_ordering_report_v1,
            cargo_json_parse_quality_report_v1,
            workspace_timeout_evidence_matrix_v2,
            workspace_timeout_root_cause_report_v4,
            workspace_timeout_diagnostic_track_progress_report_v1,
            workspace_timeout_track_risk_report_v1,
            consolidation_track_still_paused_report_v1,
            fifth_patch_still_not_applied_report_v1,
            assertion_movement_still_forbidden_report_v1,
            target_retirement_still_forbidden_report_v1,
            acceptance_truth_gate_v17,
            focused_vs_full_bridge_v13,
            workspace_no_run_recovery_gate_v17,
            workspace_full_acceptance_gate_v17,
            acceptance_evidence_strength_report_v6,
            workspace_recovery_decision_report_v6,
            timeout_track_next_action_queue_v1,
            safety_coverage_preservation_report_v32,
            control_tower_workspace_timeout_track_execution_panel,
            control_tower_acceptance_truth_panel_v17,
            storage_report: WorkspaceTimeoutTrackExecutionStorageReport {
                report_id: "workspace-timeout-track-execution-storage-report".to_string(),
                output_dir: config.output_dir().display().to_string(),
                written_files: Vec::new(),
                file_count: 41,
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
