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
    "target/soma_sprint118_timeout_reduction_queue".to_string()
}

fn default_reduction_id() -> String {
    "sprint118-timeout-reduction-queue".to_string()
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
        "timeout-reduction-only",
        "consolidation-paused",
        "fifth-patch-not-applied",
        "no-assertion-movement",
        "no-target-retirement",
        "no-run-is-not-full",
        "cargo-json-is-not-acceptance",
        "stderr-is-not-acceptance",
        "timeout-cleanup-is-not-pass",
        "focused-is-not-full",
        "CLI-smoke-is-not-full",
        "cargo-build-is-not-full",
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
pub struct WorkspaceTimeoutReductionQueueConfig {
    pub reduction_id: String,
    #[serde(default)]
    pub sprint117_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub sprint117_truth_paths: Option<Vec<String>>,
    #[serde(default)]
    pub cargo_json_execution_paths: Option<Vec<String>>,
    #[serde(default)]
    pub cargo_json_parse_paths: Option<Vec<String>>,
    #[serde(default)]
    pub no_run_execution_paths: Option<Vec<String>>,
    #[serde(default)]
    pub full_execution_paths: Option<Vec<String>>,
    #[serde(default)]
    pub timeout_cleanup_paths: Option<Vec<String>>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_false")]
    pub run_truthful_no_run_attempt: bool,
    #[serde(default = "default_false")]
    pub run_truthful_full_workspace_attempt: bool,
    #[serde(default = "default_false")]
    pub run_truthful_cargo_json_attempt: bool,
    #[serde(default = "default_timeout_ms")]
    pub no_run_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub full_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub cargo_json_timeout_ms: Option<u64>,
    #[serde(default = "default_true")]
    pub require_cargo_json_reason_analysis: bool,
    #[serde(default = "default_true")]
    pub require_timeout_reduction_queue: bool,
    #[serde(default = "default_true")]
    pub require_truthful_attempt_gate: bool,
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

impl Default for WorkspaceTimeoutReductionQueueConfig {
    fn default() -> Self {
        let default_paths = Some(vec![
            "examples/sprint118_data/sprint117_summary.json".to_string(),
        ]);
        Self {
            reduction_id: default_reduction_id(),
            sprint117_bundle_paths: default_paths.clone(),
            sprint117_truth_paths: default_paths.clone(),
            cargo_json_execution_paths: default_paths.clone(),
            cargo_json_parse_paths: default_paths.clone(),
            no_run_execution_paths: default_paths.clone(),
            full_execution_paths: default_paths.clone(),
            timeout_cleanup_paths: default_paths,
            output_root: default_output_root(),
            run_truthful_no_run_attempt: false,
            run_truthful_full_workspace_attempt: false,
            run_truthful_cargo_json_attempt: false,
            no_run_timeout_ms: default_timeout_ms(),
            full_timeout_ms: default_timeout_ms(),
            cargo_json_timeout_ms: default_timeout_ms(),
            require_cargo_json_reason_analysis: true,
            require_timeout_reduction_queue: true,
            require_truthful_attempt_gate: true,
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

impl WorkspaceTimeoutReductionQueueConfig {
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
        PathBuf::from(&self.output_root).join(&self.reduction_id)
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
        if self.reduction_id.trim().is_empty() {
            return Err("sprint118 reduction_id must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err("sprint118 output_root must be local-only".to_string());
        }
        Self::validate_paths(&self.sprint117_bundle_paths, "sprint117_bundle_paths")?;
        Self::validate_paths(&self.sprint117_truth_paths, "sprint117_truth_paths")?;
        Self::validate_paths(
            &self.cargo_json_execution_paths,
            "cargo_json_execution_paths",
        )?;
        Self::validate_paths(&self.cargo_json_parse_paths, "cargo_json_parse_paths")?;
        Self::validate_paths(&self.no_run_execution_paths, "no_run_execution_paths")?;
        Self::validate_paths(&self.full_execution_paths, "full_execution_paths")?;
        Self::validate_paths(&self.timeout_cleanup_paths, "timeout_cleanup_paths")?;
        if !self.require_cargo_json_reason_analysis {
            return Err("require_cargo_json_reason_analysis must remain true".to_string());
        }
        if !self.require_timeout_reduction_queue {
            return Err("require_timeout_reduction_queue must remain true".to_string());
        }
        if !self.require_truthful_attempt_gate {
            return Err("require_truthful_attempt_gate must remain true".to_string());
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

fn resolve_local_path(path: &str) -> PathBuf {
    if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        project_root().join(path)
    }
}

fn load_first_json<T: DeserializeOwned>(
    paths: &[Option<&Vec<String>>],
) -> Result<Option<T>, String> {
    for option in paths {
        if let Some(paths) = option {
            for path in *paths {
                let candidate = resolve_local_path(path);
                if candidate.exists() {
                    let text = fs::read_to_string(&candidate).map_err(|err| err.to_string())?;
                    return serde_json::from_str(&text)
                        .map(Some)
                        .map_err(|err| format!("{}: {err}", candidate.display()));
                }
            }
        }
    }
    Ok(None)
}

fn sample_reason_lines() -> Vec<String> {
    let mut lines = Vec::new();
    for idx in 0..48 {
        lines.push(
            serde_json::json!({
                "reason": "compiler-artifact",
                "target": { "name": "workspace_cli_integration" },
                "filenames": [format!("target/debug/deps/workspace_cli_integration-{idx}.rlib")],
                "profile": { "test": false }
            })
            .to_string(),
        );
    }
    for idx in 0..22 {
        lines.push(
            serde_json::json!({
                "reason": "compiler-message",
                "target": { "name": "macro_link_heavy_suite" },
                "message": { "rendered": format!("macro expansion still active at batch {idx}") }
            })
            .to_string(),
        );
    }
    for idx in 0..16 {
        lines.push(
            serde_json::json!({
                "reason": "build-script-executed",
                "target": { "name": "fixture_render_integration" },
                "package_id": format!("fixture-render-{idx}")
            })
            .to_string(),
        );
    }
    for idx in 0..12 {
        lines.push(
            serde_json::json!({
                "reason": "compiler-artifact",
                "target": { "name": "workspace_timeout_guard" },
                "profile": { "test": true },
                "executable": format!("target/debug/deps/workspace_timeout_guard-{idx}")
            })
            .to_string(),
        );
    }
    lines
}

fn sample_stderr_lines() -> Vec<String> {
    vec!["warning: cargo json observation timed out before full workspace acceptance".to_string()]
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sprint117SummaryFixture {
    pub report_id: String,
    pub no_run_status: String,
    pub full_workspace_status: String,
    pub cargo_json_status: String,
    pub cargo_json_reason_line_count: u64,
    pub cargo_json_stderr_line_count: u64,
    pub acceptance_truth_status: String,
    pub acceptance_evidence_status: String,
    pub consolidation_status: String,
    pub fifth_patch_status: String,
    pub assertion_movement_status: String,
    pub target_retirement_status: String,
    pub safety_status: String,
    pub focused_tests_passed: bool,
    pub cli_smoke_passed: bool,
    pub cargo_check_passed: bool,
    pub cargo_build_passed: bool,
    pub no_run_timeout_seconds: Option<u64>,
    pub no_run_exit_code: Option<i32>,
    pub full_timeout_seconds: Option<u64>,
    pub full_exit_code: Option<i32>,
    pub cargo_json_timeout_seconds: Option<u64>,
    pub cargo_json_exit_code: Option<i32>,
    #[serde(default)]
    pub remaining_cargo_processes_after_timeout: u64,
    #[serde(default)]
    pub remaining_rustc_processes_after_timeout: u64,
    #[serde(default = "sample_reason_lines")]
    pub reason_lines: Vec<String>,
    #[serde(default = "sample_stderr_lines")]
    pub stderr_lines: Vec<String>,
    #[serde(default)]
    pub suspect_targets: Vec<String>,
    #[serde(default)]
    pub artifact_blockers: Vec<String>,
    pub last_seen_message_pattern: String,
    pub artifact_tail_pattern: String,
}

impl Default for Sprint117SummaryFixture {
    fn default() -> Self {
        let reason_lines = sample_reason_lines();
        let stderr_lines = sample_stderr_lines();
        Self {
            report_id: "sprint117-summary".to_string(),
            no_run_status: "NoRunStillBlocked".to_string(),
            full_workspace_status: "FullWorkspaceStillBlocked".to_string(),
            cargo_json_status: "RealCargoJsonTimeoutExit124".to_string(),
            cargo_json_reason_line_count: reason_lines.len() as u64,
            cargo_json_stderr_line_count: stderr_lines.len() as u64,
            acceptance_truth_status: "AcceptanceTruthReadyWithWarnings".to_string(),
            acceptance_evidence_status: "AcceptanceEvidenceSupportingOnly".to_string(),
            consolidation_status: "ConsolidationStillPaused".to_string(),
            fifth_patch_status: "FifthPatchStillNotApplied".to_string(),
            assertion_movement_status: "AssertionMovementStillForbidden".to_string(),
            target_retirement_status: "TargetRetirementStillForbidden".to_string(),
            safety_status: "SafetyCoveragePreserved".to_string(),
            focused_tests_passed: true,
            cli_smoke_passed: true,
            cargo_check_passed: true,
            cargo_build_passed: true,
            no_run_timeout_seconds: Some(420),
            no_run_exit_code: Some(124),
            full_timeout_seconds: Some(420),
            full_exit_code: Some(124),
            cargo_json_timeout_seconds: Some(420),
            cargo_json_exit_code: Some(124),
            remaining_cargo_processes_after_timeout: 0,
            remaining_rustc_processes_after_timeout: 0,
            reason_lines,
            stderr_lines,
            suspect_targets: vec![
                "workspace_cli_integration".to_string(),
                "macro_link_heavy_suite".to_string(),
                "fixture_render_integration".to_string(),
            ],
            artifact_blockers: vec![
                "target/debug/deps/workspace_cli_integration-47.rlib".to_string(),
                "target/debug/deps/workspace_timeout_guard-11".to_string(),
            ],
            last_seen_message_pattern: "compiler-artifact:test-executable:workspace_timeout_guard"
                .to_string(),
            artifact_tail_pattern: "target/debug/deps/workspace_timeout_guard-*".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineClassificationV1 {
    pub line: String,
    pub class: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StderrClassificationV1 {
    pub line: String,
    pub class: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeoutReductionQueueItemV1 {
    pub item_id: String,
    pub item_kind: String,
    pub description: String,
    pub supporting_only: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExperimentStepV1 {
    pub experiment_id: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttemptSnapshotV1 {
    pub sprint: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceTimeoutEvidenceRowV4 {
    pub row_id: String,
    pub evidence_kind: String,
    pub status: String,
    pub supports_acceptance: bool,
}

report!(Sprint117BaselineTruthImportReport {
    report_id: String,
    no_run_status: String,
    full_workspace_status: String,
    cargo_json_status: String,
    cargo_json_reason_line_count: Option<u64>,
    cargo_json_stderr_line_count: Option<u64>,
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
report!(Sprint117RealObservationCarryForwardReport {
    report_id: String,
    real_no_run_carried_forward: bool,
    real_full_carried_forward: bool,
    real_cargo_json_carried_forward: bool,
    no_run_exit_code: Option<i32>,
    full_exit_code: Option<i32>,
    cargo_json_exit_code: Option<i32>,
    cleanup_counts_carried_forward: bool,
    actual_vs_carried_forward_separation_preserved: bool,
    carry_forward_status: String
});
report!(CargoJsonFailureReasonAnalysisReportV1 {
    report_id: String,
    reason_line_count: u64,
    stderr_line_count: u64,
    parsed_message_count: u64,
    compiler_artifact_count: u64,
    compiler_message_count: u64,
    test_executable_count: u64,
    dominant_reason_class: String,
    analysis_status: String
});
report!(CargoJsonReasonLineClassificationReportV1 {
    report_id: String,
    reason_lines: Vec<String>,
    line_classes: Vec<LineClassificationV1>,
    unknown_line_count: u64,
    status: String
});
report!(CargoJsonStderrClassificationReportV1 {
    report_id: String,
    stderr_line_count: u64,
    stderr_classes: Vec<StderrClassificationV1>,
    status: String
});
report!(CargoJsonTimeoutPatternReportV1 {
    report_id: String,
    timeout_boundary: String,
    last_seen_message_pattern: String,
    artifact_tail_pattern: String,
    status: String
});
report!(CargoJsonTargetBlockerExtractionReportV1 {
    report_id: String,
    target_blockers: Vec<String>,
    suspect_targets: Vec<String>,
    artifact_blockers: Vec<String>,
    status: String
});
report!(WorkspaceTimeoutReductionHypothesisReportV1 {
    report_id: String,
    hypotheses: Vec<String>,
    evidence_refs: Vec<String>,
    confidence: String,
    hypothesis_status: String
});
report!(WorkspaceTimeoutReductionQueueV1 {
    queue_id: String,
    ordered_items: Vec<TimeoutReductionQueueItemV1>,
    queue_status: String
});
report!(WorkspaceTimeoutReductionExperimentPlanV1 {
    plan_id: String,
    experiments: Vec<ExperimentStepV1>,
    preconditions: Vec<String>,
    no_acceptance_overclaim_warning: String,
    status: String
});
report!(WorkspaceTimeoutReductionExperimentReportV1 {
    report_id: String,
    planned_experiments: Vec<String>,
    run_experiments: Vec<String>,
    skipped_experiments: Vec<String>,
    status: String
});
report!(NoRunTimeoutReductionPlanV1 {
    plan_id: String,
    no_run_attempt_strategy: String,
    timeout: Option<u64>,
    cleanup: String,
    evidence_treatment: String,
    status: String
});
report!(FullWorkspaceTimeoutReductionPlanV1 {
    plan_id: String,
    full_attempt_strategy: String,
    timeout: Option<u64>,
    cleanup: String,
    acceptance_condition: String,
    status: String
});
report!(CargoJsonTimeoutReductionPlanV1 {
    plan_id: String,
    cargo_json_attempt_strategy: String,
    parse_strategy: String,
    reason_classification: String,
    status: String
});
report!(TargetFamilyTimeoutReductionPlanV1 {
    plan_id: String,
    integration_fanout: String,
    link_macro: String,
    fixture_render_cli: String,
    status: String
});
report!(SuspectTargetTimeoutReductionPlanV1 {
    plan_id: String,
    suspect_targets: Vec<String>,
    status: String
});
report!(LinkMacroTimeoutReductionPlanV1 {
    plan_id: String,
    link_macro_hypotheses: Vec<String>,
    status: String
});
report!(IntegrationFanoutTimeoutReductionPlanV1 {
    plan_id: String,
    integration_fanout_hypotheses: Vec<String>,
    status: String
});
report!(FixtureRenderCliTimeoutReductionPlanV1 {
    plan_id: String,
    fixture_render_cli_hypotheses: Vec<String>,
    status: String
});
report!(NextestDiagnosticFollowupPlanV2 {
    plan_id: String,
    nextest_followup: Vec<String>,
    no_acceptance_claim: bool,
    status: String
});
report!(SccacheDiagnosticFollowupPlanV2 {
    plan_id: String,
    local_only_sccache_followup: Vec<String>,
    no_guaranteed_speedup: bool,
    status: String
});
report!(TimeoutEnvironmentPolicyReportV1 {
    report_id: String,
    timeout_command_policy: String,
    observation_environment: String,
    no_fake_timing: bool,
    status: String
});
report!(TimeoutCommandWrapperSafetyReportV1 {
    report_id: String,
    wrapper_safety: String,
    child_cleanup: String,
    no_orphan_process_overclaim: bool,
    status: String
});
report!(TimeoutChildProcessCleanupPolicyV1 {
    report_id: String,
    cleanup_policy: String,
    actual_counts_required: bool,
    status: String
});
report!(TimeoutObservationRepeatPlanV1 {
    plan_id: String,
    repeat_attempts: Vec<String>,
    timeout_windows: Vec<u64>,
    reason: String,
    status: String
});
report!(TruthfulNoRunAttemptV19 {
    report_id: String,
    attempted: bool,
    finished: bool,
    passed: Option<bool>,
    exit_code: Option<i32>,
    timed_out: bool,
    timeout_ms: Option<u64>,
    recovered: bool,
    status: String
});
report!(TruthfulFullWorkspaceAttemptV19 {
    report_id: String,
    attempted: bool,
    finished: bool,
    passed: Option<bool>,
    exit_code: Option<i32>,
    timed_out: bool,
    timeout_ms: Option<u64>,
    full_workspace_accepted: bool,
    status: String
});
report!(TruthfulCargoJsonAttemptV19 {
    report_id: String,
    attempted: bool,
    parsed_json_message_count: u64,
    reason_line_count: u64,
    stderr_line_count: u64,
    timed_out: bool,
    status: String
});
report!(NoRunAttemptComparisonV1 {
    report_id: String,
    comparison: Vec<AttemptSnapshotV1>,
    status: String
});
report!(FullWorkspaceAttemptComparisonV1 {
    report_id: String,
    comparison: Vec<AttemptSnapshotV1>,
    status: String
});
report!(CargoJsonAttemptComparisonV1 {
    report_id: String,
    comparison: Vec<AttemptSnapshotV1>,
    status: String
});
report!(WorkspaceTimeoutEvidenceMatrixV4 {
    report_id: String,
    evidence_rows: Vec<WorkspaceTimeoutEvidenceRowV4>,
    supports_acceptance: bool,
    status: String
});
report!(WorkspaceTimeoutRootCauseReportV6 {
    report_id: String,
    previous_root_cause: String,
    cargo_json_reason_analysis: String,
    reduction_hypotheses: Vec<String>,
    confidence: String,
    status: String
});
report!(TimeoutReductionProgressReportV1 {
    report_id: String,
    reduction_queue_progress: Vec<String>,
    status: String
});
report!(TimeoutReductionRiskReportV1 {
    report_id: String,
    overclaim_risk: String,
    parse_risk: String,
    false_cleanup_risk: String,
    acceptance_confusion_risk: String,
    status: String
});
report!(ConsolidationTrackStillPausedReportV3 {
    report_id: String,
    paused: bool,
    stopped: bool,
    no_assertion_movement: bool,
    no_target_retirement: bool,
    status: String
});
report!(FifthPatchStillNotAppliedReportV3 {
    report_id: String,
    fifth_patch_applied: bool,
    no_assertions_moved: bool,
    no_targets_retired: bool,
    status: String
});
report!(AssertionMovementStillForbiddenReportV3 {
    report_id: String,
    movement_allowed: bool,
    status: String
});
report!(TargetRetirementStillForbiddenReportV3 {
    report_id: String,
    retirement_allowed: bool,
    status: String
});
report!(WorkspaceNoRunRecoveryGateV19 {
    gate_id: String,
    recovered: bool,
    truthful_no_run_status: String,
    status: String
});
report!(WorkspaceFullAcceptanceGateV19 {
    gate_id: String,
    accepted: bool,
    truthful_full_status: String,
    status: String
});
report!(FocusedVsFullBridgeV15 {
    bridge_id: String,
    supporting_only_labels: Vec<String>,
    status: String
});
report!(AcceptanceTruthGateV19 {
    gate_id: String,
    can_claim_full_acceptance: bool,
    full_gate_status: String,
    status: String
});
report!(AcceptanceEvidenceStrengthReportV8 {
    report_id: String,
    full_evidence_sufficient: bool,
    supporting_only_evidence: Vec<String>,
    status: String
});
report!(WorkspaceRecoveryDecisionReportV8 {
    report_id: String,
    continue_timeout_reduction_queue: bool,
    keep_consolidation_paused: bool,
    keep_fifth_patch_not_applied: bool,
    status: String
});
report!(TimeoutReductionNextActionQueueV1 {
    queue_id: String,
    next_queue_items: Vec<String>,
    status: String
});
report!(ControlTowerTimeoutReductionQueuePanel {
    panel_id: String,
    cargo_json_reason_analysis: String,
    timeout_reduction_hypotheses: Vec<String>,
    ordered_queue: Vec<String>,
    truthful_no_run_gate: String,
    truthful_full_gate: String,
    warnings: Vec<String>,
    static_read_only: bool,
    no_run_button: bool,
    no_apply_button: bool,
    no_train_runtime_live_order_account_controls: bool
});
report!(ControlTowerAcceptanceTruthPanelV19 {
    panel_id: String,
    full_gate: String,
    no_run_gate: String,
    supporting_evidence: Vec<String>,
    warnings: Vec<String>,
    static_read_only: bool,
    no_action_button: bool,
    no_train_runtime_live_order_account_controls: bool
});
report!(SafetyCoveragePreservationReportV34 {
    report_id: String,
    no_assertion_deletion: bool,
    no_safety_sentinel_deletion: bool,
    no_hidden_skips: bool,
    timeout_reduction_queue_guard_present: bool,
    cargo_json_reason_analysis_guard_present: bool,
    truthful_attempt_gate_guard_present: bool,
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
report!(WorkspaceTimeoutReductionStorageReport {
    report_id: String,
    output_dir: String,
    written_files: Vec<String>,
    file_count: u64
});

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceTimeoutReductionQueueBundle {
    pub sprint117_baseline_truth_import_report: Sprint117BaselineTruthImportReport,
    pub sprint117_real_observation_carry_forward_report: Sprint117RealObservationCarryForwardReport,
    pub cargo_json_failure_reason_analysis_report_v1: CargoJsonFailureReasonAnalysisReportV1,
    pub cargo_json_reason_line_classification_report_v1: CargoJsonReasonLineClassificationReportV1,
    pub cargo_json_stderr_classification_report_v1: CargoJsonStderrClassificationReportV1,
    pub cargo_json_timeout_pattern_report_v1: CargoJsonTimeoutPatternReportV1,
    pub cargo_json_target_blocker_extraction_report_v1: CargoJsonTargetBlockerExtractionReportV1,
    pub workspace_timeout_reduction_hypothesis_report_v1:
        WorkspaceTimeoutReductionHypothesisReportV1,
    pub workspace_timeout_reduction_queue_v1: WorkspaceTimeoutReductionQueueV1,
    pub workspace_timeout_reduction_experiment_plan_v1: WorkspaceTimeoutReductionExperimentPlanV1,
    pub workspace_timeout_reduction_experiment_report_v1:
        WorkspaceTimeoutReductionExperimentReportV1,
    pub no_run_timeout_reduction_plan_v1: NoRunTimeoutReductionPlanV1,
    pub full_workspace_timeout_reduction_plan_v1: FullWorkspaceTimeoutReductionPlanV1,
    pub cargo_json_timeout_reduction_plan_v1: CargoJsonTimeoutReductionPlanV1,
    pub target_family_timeout_reduction_plan_v1: TargetFamilyTimeoutReductionPlanV1,
    pub suspect_target_timeout_reduction_plan_v1: SuspectTargetTimeoutReductionPlanV1,
    pub link_macro_timeout_reduction_plan_v1: LinkMacroTimeoutReductionPlanV1,
    pub integration_fanout_timeout_reduction_plan_v1: IntegrationFanoutTimeoutReductionPlanV1,
    pub fixture_render_cli_timeout_reduction_plan_v1: FixtureRenderCliTimeoutReductionPlanV1,
    pub nextest_diagnostic_followup_plan_v2: NextestDiagnosticFollowupPlanV2,
    pub sccache_diagnostic_followup_plan_v2: SccacheDiagnosticFollowupPlanV2,
    pub timeout_environment_policy_report_v1: TimeoutEnvironmentPolicyReportV1,
    pub timeout_command_wrapper_safety_report_v1: TimeoutCommandWrapperSafetyReportV1,
    pub timeout_child_process_cleanup_policy_v1: TimeoutChildProcessCleanupPolicyV1,
    pub timeout_observation_repeat_plan_v1: TimeoutObservationRepeatPlanV1,
    pub truthful_no_run_attempt_v19: TruthfulNoRunAttemptV19,
    pub truthful_full_workspace_attempt_v19: TruthfulFullWorkspaceAttemptV19,
    pub truthful_cargo_json_attempt_v19: TruthfulCargoJsonAttemptV19,
    pub no_run_attempt_comparison_v1: NoRunAttemptComparisonV1,
    pub full_workspace_attempt_comparison_v1: FullWorkspaceAttemptComparisonV1,
    pub cargo_json_attempt_comparison_v1: CargoJsonAttemptComparisonV1,
    pub workspace_timeout_evidence_matrix_v4: WorkspaceTimeoutEvidenceMatrixV4,
    pub workspace_timeout_root_cause_report_v6: WorkspaceTimeoutRootCauseReportV6,
    pub timeout_reduction_progress_report_v1: TimeoutReductionProgressReportV1,
    pub timeout_reduction_risk_report_v1: TimeoutReductionRiskReportV1,
    pub consolidation_track_still_paused_report_v3: ConsolidationTrackStillPausedReportV3,
    pub fifth_patch_still_not_applied_report_v3: FifthPatchStillNotAppliedReportV3,
    pub assertion_movement_still_forbidden_report_v3: AssertionMovementStillForbiddenReportV3,
    pub target_retirement_still_forbidden_report_v3: TargetRetirementStillForbiddenReportV3,
    pub workspace_no_run_recovery_gate_v19: WorkspaceNoRunRecoveryGateV19,
    pub workspace_full_acceptance_gate_v19: WorkspaceFullAcceptanceGateV19,
    pub focused_vs_full_bridge_v15: FocusedVsFullBridgeV15,
    pub acceptance_truth_gate_v19: AcceptanceTruthGateV19,
    pub acceptance_evidence_strength_report_v8: AcceptanceEvidenceStrengthReportV8,
    pub workspace_recovery_decision_report_v8: WorkspaceRecoveryDecisionReportV8,
    pub timeout_reduction_next_action_queue_v1: TimeoutReductionNextActionQueueV1,
    pub safety_coverage_preservation_report_v34: SafetyCoveragePreservationReportV34,
    pub control_tower_timeout_reduction_queue_panel: ControlTowerTimeoutReductionQueuePanel,
    pub control_tower_acceptance_truth_panel_v19: ControlTowerAcceptanceTruthPanelV19,
    pub storage_report: WorkspaceTimeoutReductionStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CargoJsonDerivedAnalysis {
    parsed_message_count: u64,
    compiler_artifact_count: u64,
    compiler_message_count: u64,
    build_script_count: u64,
    test_executable_count: u64,
    line_classes: Vec<LineClassificationV1>,
    stderr_classes: Vec<StderrClassificationV1>,
    class_counts: BTreeMap<String, u64>,
    unknown_line_count: u64,
    target_counts: BTreeMap<String, u64>,
    artifact_blockers: Vec<String>,
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

fn classify_reason_value(value: &Value) -> String {
    let reason = value
        .get("reason")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    match reason {
        "compiler-artifact" => {
            let is_test = value
                .get("profile")
                .and_then(|value| value.get("test"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if value.get("executable").is_some() || is_test {
                "TestExecutable".to_string()
            } else if value
                .get("filenames")
                .and_then(Value::as_array)
                .map(|files| {
                    files.iter().any(|file| {
                        file.as_str()
                            .map(|name| name.ends_with(".rlib") || name.ends_with(".dylib"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
            {
                "LinkArtifact".to_string()
            } else {
                "BuildQueueProgress".to_string()
            }
        }
        "compiler-message" => "CompilerMessage".to_string(),
        "build-script-executed" => "BuildScript".to_string(),
        _ => "Unknown".to_string(),
    }
}

fn analyze_cargo_json(summary: &Sprint117SummaryFixture) -> CargoJsonDerivedAnalysis {
    let mut analysis = CargoJsonDerivedAnalysis::default();
    for line in &summary.reason_lines {
        match serde_json::from_str::<Value>(line) {
            Ok(value) => {
                analysis.parsed_message_count += 1;
                if let Some(target) = value
                    .get("target")
                    .and_then(|value| value.get("name"))
                    .and_then(Value::as_str)
                {
                    *analysis
                        .target_counts
                        .entry(target.to_string())
                        .or_insert(0) += 1;
                }
                let class = classify_reason_value(&value);
                *analysis.class_counts.entry(class.clone()).or_insert(0) += 1;
                if class == "Unknown" {
                    analysis.unknown_line_count += 1;
                }
                match class.as_str() {
                    "LinkArtifact" | "BuildQueueProgress" | "TestExecutable" => {
                        analysis.compiler_artifact_count += 1;
                    }
                    "CompilerMessage" => analysis.compiler_message_count += 1,
                    "BuildScript" => analysis.build_script_count += 1,
                    _ => {}
                }
                if class == "TestExecutable" {
                    analysis.test_executable_count += 1;
                }
                if class == "LinkArtifact" {
                    if let Some(files) = value.get("filenames").and_then(Value::as_array) {
                        for file in files {
                            if let Some(file) = file.as_str() {
                                analysis.artifact_blockers.push(file.to_string());
                            }
                        }
                    }
                }
                analysis.line_classes.push(LineClassificationV1 {
                    line: line.clone(),
                    class,
                });
            }
            Err(_) => {
                analysis.unknown_line_count += 1;
                *analysis
                    .class_counts
                    .entry("Unknown".to_string())
                    .or_insert(0) += 1;
                analysis.line_classes.push(LineClassificationV1 {
                    line: line.clone(),
                    class: "Unknown".to_string(),
                });
            }
        }
    }
    for line in &summary.stderr_lines {
        let class = if line.to_ascii_lowercase().contains("timed out") {
            "Timeout"
        } else if line.to_ascii_lowercase().contains("warning") {
            "Warning"
        } else {
            "Unknown"
        };
        analysis.stderr_classes.push(StderrClassificationV1 {
            line: line.clone(),
            class: class.to_string(),
        });
    }
    analysis.artifact_blockers = stable_strings(
        analysis
            .artifact_blockers
            .into_iter()
            .chain(summary.artifact_blockers.iter().cloned())
            .collect(),
    );
    analysis
}

fn dominant_class(class_counts: &BTreeMap<String, u64>) -> String {
    class_counts
        .iter()
        .max_by(|(class_a, count_a), (class_b, count_b)| {
            count_a.cmp(count_b).then_with(|| class_b.cmp(class_a))
        })
        .map(|(class, _)| class.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

pub fn build_sprint117_baseline_truth_import_report(
    summary: &Sprint117SummaryFixture,
) -> Sprint117BaselineTruthImportReport {
    Sprint117BaselineTruthImportReport {
        report_id: "sprint117-baseline-truth-import".to_string(),
        no_run_status: summary.no_run_status.clone(),
        full_workspace_status: summary.full_workspace_status.clone(),
        cargo_json_status: summary.cargo_json_status.clone(),
        cargo_json_reason_line_count: Some(summary.cargo_json_reason_line_count),
        cargo_json_stderr_line_count: Some(summary.cargo_json_stderr_line_count),
        acceptance_truth_status: summary.acceptance_truth_status.clone(),
        consolidation_status: summary.consolidation_status.clone(),
        fifth_patch_status: summary.fifth_patch_status.clone(),
        safety_status: summary.safety_status.clone(),
        focused_tests_passed: summary.focused_tests_passed,
        cli_smoke_passed: summary.cli_smoke_passed,
        cargo_check_passed: summary.cargo_check_passed,
        cargo_build_passed: summary.cargo_build_passed,
        imported_as_full_acceptance: false,
        import_status: if summary.acceptance_truth_status.contains("Warnings") {
            "Sprint117TruthImportedWithWarnings".to_string()
        } else {
            "Sprint117TruthImported".to_string()
        },
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_sprint117_real_observation_carry_forward_report(
    summary: &Sprint117SummaryFixture,
) -> Sprint117RealObservationCarryForwardReport {
    Sprint117RealObservationCarryForwardReport {
        report_id: "sprint117-real-observation-carry-forward".to_string(),
        real_no_run_carried_forward: true,
        real_full_carried_forward: true,
        real_cargo_json_carried_forward: true,
        no_run_exit_code: summary.no_run_exit_code,
        full_exit_code: summary.full_exit_code,
        cargo_json_exit_code: summary.cargo_json_exit_code,
        cleanup_counts_carried_forward: true,
        actual_vs_carried_forward_separation_preserved: true,
        carry_forward_status: "RealObservationCarriedForwardWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cargo_json_reason_line_classification_report_v1(
    summary: &Sprint117SummaryFixture,
) -> CargoJsonReasonLineClassificationReportV1 {
    let analysis = analyze_cargo_json(summary);
    CargoJsonReasonLineClassificationReportV1 {
        report_id: "cargo-json-reason-line-classification-v1".to_string(),
        reason_lines: summary.reason_lines.clone(),
        line_classes: analysis.line_classes,
        unknown_line_count: analysis.unknown_line_count,
        status: if summary.reason_lines.is_empty() {
            "DiagnosticOnly".to_string()
        } else if analysis.unknown_line_count > 0 {
            "CargoJsonReasonClassifiedWithWarnings".to_string()
        } else {
            "CargoJsonReasonClassified".to_string()
        },
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cargo_json_stderr_classification_report_v1(
    summary: &Sprint117SummaryFixture,
) -> CargoJsonStderrClassificationReportV1 {
    let analysis = analyze_cargo_json(summary);
    CargoJsonStderrClassificationReportV1 {
        report_id: "cargo-json-stderr-classification-v1".to_string(),
        stderr_line_count: summary.stderr_lines.len() as u64,
        stderr_classes: analysis.stderr_classes,
        status: if summary.stderr_lines.is_empty() {
            "DiagnosticOnly".to_string()
        } else {
            "CargoJsonStderrClassified".to_string()
        },
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cargo_json_failure_reason_analysis_report_v1(
    summary: &Sprint117SummaryFixture,
) -> CargoJsonFailureReasonAnalysisReportV1 {
    let analysis = analyze_cargo_json(summary);
    let dominant_reason_class = dominant_class(&analysis.class_counts);
    let analysis_status = if summary.reason_lines.is_empty() {
        "DiagnosticOnly".to_string()
    } else if dominant_reason_class == "Unknown" {
        "CargoJsonFailureReasonAmbiguous".to_string()
    } else if analysis.unknown_line_count > 0 {
        "CargoJsonFailureReasonClassifiedWithWarnings".to_string()
    } else {
        "CargoJsonFailureReasonClassified".to_string()
    };
    CargoJsonFailureReasonAnalysisReportV1 {
        report_id: "cargo-json-failure-reason-analysis-v1".to_string(),
        reason_line_count: summary.reason_lines.len() as u64,
        stderr_line_count: summary.stderr_lines.len() as u64,
        parsed_message_count: analysis.parsed_message_count,
        compiler_artifact_count: analysis.compiler_artifact_count,
        compiler_message_count: analysis.compiler_message_count,
        test_executable_count: analysis.test_executable_count,
        dominant_reason_class,
        analysis_status,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cargo_json_timeout_pattern_report_v1(
    summary: &Sprint117SummaryFixture,
) -> CargoJsonTimeoutPatternReportV1 {
    CargoJsonTimeoutPatternReportV1 {
        report_id: "cargo-json-timeout-pattern-v1".to_string(),
        timeout_boundary: format!(
            "{}s timeout exit {}",
            summary.cargo_json_timeout_seconds.unwrap_or(0),
            summary.cargo_json_exit_code.unwrap_or_default()
        ),
        last_seen_message_pattern: summary.last_seen_message_pattern.clone(),
        artifact_tail_pattern: summary.artifact_tail_pattern.clone(),
        status: "CargoJsonTimeoutPatternReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cargo_json_target_blocker_extraction_report_v1(
    summary: &Sprint117SummaryFixture,
) -> CargoJsonTargetBlockerExtractionReportV1 {
    let analysis = analyze_cargo_json(summary);
    let mut target_blockers = summary.suspect_targets.clone();
    target_blockers.extend(
        analysis
            .target_counts
            .into_iter()
            .filter(|(_, count)| *count >= 10)
            .map(|(target, _)| target),
    );
    let suspect_targets = stable_strings(
        summary
            .suspect_targets
            .iter()
            .cloned()
            .chain(target_blockers.iter().cloned())
            .collect(),
    );
    CargoJsonTargetBlockerExtractionReportV1 {
        report_id: "cargo-json-target-blocker-extraction-v1".to_string(),
        target_blockers: stable_strings(target_blockers),
        suspect_targets,
        artifact_blockers: analysis.artifact_blockers,
        status: "CargoJsonTargetBlockersExtracted".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_reduction_hypothesis_report_v1(
    blocker_report: &CargoJsonTargetBlockerExtractionReportV1,
    analysis: &CargoJsonFailureReasonAnalysisReportV1,
) -> WorkspaceTimeoutReductionHypothesisReportV1 {
    let mut hypotheses = Vec::new();
    let joined = blocker_report.suspect_targets.join(" ");
    if joined.contains("integration") {
        hypotheses.push("IntegrationTestBinaryFanout".to_string());
    }
    if joined.contains("link") || analysis.dominant_reason_class == "LinkArtifact" {
        hypotheses.push("LinkTimeCost".to_string());
    }
    if joined.contains("macro") {
        hypotheses.push("MacroExpansionCost".to_string());
    }
    if joined.contains("fixture") {
        hypotheses.push("FixtureSetupFanout".to_string());
    }
    if joined.contains("render") {
        hypotheses.push("ArtifactRenderFanout".to_string());
    }
    if joined.contains("cli") {
        hypotheses.push("CliSmokeFanout".to_string());
    }
    hypotheses.push("TimeoutWrapperOverhead".to_string());
    let hypotheses = stable_strings(hypotheses);
    let confidence = if blocker_report.suspect_targets.len() >= 3 {
        "Moderate"
    } else if blocker_report.suspect_targets.is_empty() {
        "Insufficient"
    } else {
        "Weak"
    };
    WorkspaceTimeoutReductionHypothesisReportV1 {
        report_id: "workspace-timeout-reduction-hypothesis-v1".to_string(),
        hypotheses,
        evidence_refs: vec![
            analysis.report_id.clone(),
            blocker_report.report_id.clone(),
            "sprint117-real-observation-carry-forward".to_string(),
        ],
        confidence: confidence.to_string(),
        hypothesis_status: if confidence == "Insufficient" {
            "TimeoutHypothesisInsufficient".to_string()
        } else if confidence == "Weak" {
            "TimeoutHypothesisReadyWithWarnings".to_string()
        } else {
            "TimeoutHypothesisReady".to_string()
        },
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_reduction_queue_v1(
    hypothesis: &WorkspaceTimeoutReductionHypothesisReportV1,
) -> WorkspaceTimeoutReductionQueueV1 {
    let ordered_items = vec![
        TimeoutReductionQueueItemV1 {
            item_id: "cargo-json-reason-analysis".to_string(),
            item_kind: "CargoJsonReasonAnalysis".to_string(),
            description: "Keep cargo JSON reason analysis supporting-only and classify blocker families.".to_string(),
            supporting_only: true,
        },
        TimeoutReductionQueueItemV1 {
            item_id: "timeout-boundary-repeat".to_string(),
            item_kind: "TimeoutBoundaryRepeat".to_string(),
            description: "Repeat timeout boundary with truthful wrappers and exact cleanup accounting.".to_string(),
            supporting_only: true,
        },
        TimeoutReductionQueueItemV1 {
            item_id: "target-family-drilldown".to_string(),
            item_kind: "TargetFamilyDrilldown".to_string(),
            description: "Drill down suspect integration, link, macro, fixture, render, and CLI families.".to_string(),
            supporting_only: true,
        },
        TimeoutReductionQueueItemV1 {
            item_id: "nextest-followup".to_string(),
            item_kind: "NextestDiagnosticFollowup".to_string(),
            description: "Use nextest only as diagnostic follow-up, never as full acceptance substitute.".to_string(),
            supporting_only: true,
        },
        TimeoutReductionQueueItemV1 {
            item_id: "sccache-followup".to_string(),
            item_kind: "SccacheDiagnosticFollowup".to_string(),
            description: "Check local-only sccache observations without claiming guaranteed speedup.".to_string(),
            supporting_only: true,
        },
        TimeoutReductionQueueItemV1 {
            item_id: "truthful-no-run".to_string(),
            item_kind: "NoRunAttempt".to_string(),
            description: "Attempt truthful no-run recovery without confusing it with full acceptance.".to_string(),
            supporting_only: true,
        },
        TimeoutReductionQueueItemV1 {
            item_id: "truthful-full-workspace".to_string(),
            item_kind: "FullWorkspaceAttempt".to_string(),
            description: "Attempt full workspace only after blocker evidence is narrowed; only a pass can accept.".to_string(),
            supporting_only: false,
        },
        TimeoutReductionQueueItemV1 {
            item_id: "stop-consolidation-hold".to_string(),
            item_kind: "StopConsolidationHold".to_string(),
            description: "Keep consolidation paused and keep the fifth patch unapplied.".to_string(),
            supporting_only: true,
        },
    ];
    WorkspaceTimeoutReductionQueueV1 {
        queue_id: "workspace-timeout-reduction-queue-v1".to_string(),
        ordered_items,
        queue_status: if hypothesis.hypothesis_status == "TimeoutHypothesisInsufficient" {
            "TimeoutReductionQueueNeedsMoreEvidence".to_string()
        } else if hypothesis.hypothesis_status.contains("Warnings") {
            "TimeoutReductionQueueReadyWithWarnings".to_string()
        } else {
            "TimeoutReductionQueueReady".to_string()
        },
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_reduction_experiment_plan_v1(
    queue: &WorkspaceTimeoutReductionQueueV1,
) -> WorkspaceTimeoutReductionExperimentPlanV1 {
    WorkspaceTimeoutReductionExperimentPlanV1 {
        plan_id: "workspace-timeout-reduction-experiment-plan-v1".to_string(),
        experiments: queue
            .ordered_items
            .iter()
            .map(|item| ExperimentStepV1 {
                experiment_id: item.item_id.clone(),
                description: item.description.clone(),
            })
            .collect(),
        preconditions: vec![
            "Consolidation must remain paused.".to_string(),
            "Fifth patch must remain unapplied.".to_string(),
            "No-run, cargo JSON, stderr, cleanup, focused tests, CLI smoke, and cargo build remain supporting-only."
                .to_string(),
        ],
        no_acceptance_overclaim_warning:
            "TimeoutReductionQueueReady does not mean timeout solved or accepted.".to_string(),
        status: "TimeoutReductionExperimentReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_reduction_experiment_report_v1(
    queue: &WorkspaceTimeoutReductionQueueV1,
    config: &WorkspaceTimeoutReductionQueueConfig,
) -> WorkspaceTimeoutReductionExperimentReportV1 {
    let planned_experiments = queue
        .ordered_items
        .iter()
        .map(|item| item.item_id.clone())
        .collect::<Vec<_>>();
    let mut run_experiments = Vec::new();
    if config.run_truthful_no_run_attempt {
        run_experiments.push("truthful-no-run".to_string());
    }
    if config.run_truthful_full_workspace_attempt {
        run_experiments.push("truthful-full-workspace".to_string());
    }
    if config.run_truthful_cargo_json_attempt {
        run_experiments.push("truthful-cargo-json".to_string());
    }
    let skipped_experiments = planned_experiments
        .iter()
        .filter(|item| !run_experiments.iter().any(|ran| ran == *item))
        .cloned()
        .collect::<Vec<_>>();
    WorkspaceTimeoutReductionExperimentReportV1 {
        report_id: "workspace-timeout-reduction-experiment-report-v1".to_string(),
        planned_experiments,
        run_experiments,
        skipped_experiments,
        status: "TimeoutReductionExperimentReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_no_run_timeout_reduction_plan_v1(
    config: &WorkspaceTimeoutReductionQueueConfig,
) -> NoRunTimeoutReductionPlanV1 {
    NoRunTimeoutReductionPlanV1 {
        plan_id: "no-run-timeout-reduction-plan-v1".to_string(),
        no_run_attempt_strategy: "Run cargo test --workspace --no-run --quiet under truthful timeout wrapper and preserve blocked truth on timeout.".to_string(),
        timeout: config.no_run_timeout_ms,
        cleanup: "Capture exact cargo/rustc counts after wrapper completion; cleanup evidence is supporting-only.".to_string(),
        evidence_treatment: "No-run recovery never upgrades to full workspace acceptance.".to_string(),
        status: "NoRunTimeoutReductionPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_full_workspace_timeout_reduction_plan_v1(
    config: &WorkspaceTimeoutReductionQueueConfig,
) -> FullWorkspaceTimeoutReductionPlanV1 {
    FullWorkspaceTimeoutReductionPlanV1 {
        plan_id: "full-workspace-timeout-reduction-plan-v1".to_string(),
        full_attempt_strategy: "Run cargo test --workspace --quiet only through truthful wrapper and keep supporting-only evidence separate.".to_string(),
        timeout: config.full_timeout_ms,
        cleanup: "Record exact child cleanup counts; timeout cleanup is not a pass.".to_string(),
        acceptance_condition: "Only finished and passed cargo test --workspace --quiet may set FullWorkspaceAccepted.".to_string(),
        status: "FullWorkspaceTimeoutReductionPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cargo_json_timeout_reduction_plan_v1(
    config: &WorkspaceTimeoutReductionQueueConfig,
) -> CargoJsonTimeoutReductionPlanV1 {
    CargoJsonTimeoutReductionPlanV1 {
        plan_id: "cargo-json-timeout-reduction-plan-v1".to_string(),
        cargo_json_attempt_strategy:
            "Run cargo test --workspace --no-run --message-format=json only as diagnostic evidence."
                .to_string(),
        parse_strategy:
            "Classify reason and stderr lines deterministically from actual or carried-forward local output."
                .to_string(),
        reason_classification: format!(
            "timeout_ms={:?}; cargo-json-is-not-acceptance",
            config.cargo_json_timeout_ms
        ),
        status: "CargoJsonTimeoutReductionPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_target_family_timeout_reduction_plan_v1(
    blocker_report: &CargoJsonTargetBlockerExtractionReportV1,
) -> TargetFamilyTimeoutReductionPlanV1 {
    TargetFamilyTimeoutReductionPlanV1 {
        plan_id: "target-family-timeout-reduction-plan-v1".to_string(),
        integration_fanout: "Inspect integration-heavy targets first.".to_string(),
        link_macro: format!(
            "Focus link/macro suspects: {}",
            blocker_report.suspect_targets.join(", ")
        ),
        fixture_render_cli: "Inspect fixture, render, and CLI layers as supporting-only blockers."
            .to_string(),
        status: "TargetFamilyTimeoutReductionPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_suspect_target_timeout_reduction_plan_v1(
    blocker_report: &CargoJsonTargetBlockerExtractionReportV1,
) -> SuspectTargetTimeoutReductionPlanV1 {
    SuspectTargetTimeoutReductionPlanV1 {
        plan_id: "suspect-target-timeout-reduction-plan-v1".to_string(),
        suspect_targets: blocker_report.suspect_targets.clone(),
        status: "SuspectTargetTimeoutReductionPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_link_macro_timeout_reduction_plan_v1(
    hypotheses: &WorkspaceTimeoutReductionHypothesisReportV1,
) -> LinkMacroTimeoutReductionPlanV1 {
    LinkMacroTimeoutReductionPlanV1 {
        plan_id: "link-macro-timeout-reduction-plan-v1".to_string(),
        link_macro_hypotheses: hypotheses
            .hypotheses
            .iter()
            .filter(|value| value.contains("Link") || value.contains("Macro"))
            .cloned()
            .collect(),
        status: "LinkMacroTimeoutReductionPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_integration_fanout_timeout_reduction_plan_v1(
    hypotheses: &WorkspaceTimeoutReductionHypothesisReportV1,
) -> IntegrationFanoutTimeoutReductionPlanV1 {
    IntegrationFanoutTimeoutReductionPlanV1 {
        plan_id: "integration-fanout-timeout-reduction-plan-v1".to_string(),
        integration_fanout_hypotheses: hypotheses
            .hypotheses
            .iter()
            .filter(|value| value.contains("Integration"))
            .cloned()
            .collect(),
        status: "IntegrationFanoutTimeoutReductionPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_fixture_render_cli_timeout_reduction_plan_v1(
    hypotheses: &WorkspaceTimeoutReductionHypothesisReportV1,
) -> FixtureRenderCliTimeoutReductionPlanV1 {
    FixtureRenderCliTimeoutReductionPlanV1 {
        plan_id: "fixture-render-cli-timeout-reduction-plan-v1".to_string(),
        fixture_render_cli_hypotheses: hypotheses
            .hypotheses
            .iter()
            .filter(|value| {
                value.contains("Fixture") || value.contains("Render") || value.contains("Cli")
            })
            .cloned()
            .collect(),
        status: "FixtureRenderCliTimeoutReductionPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_nextest_diagnostic_followup_plan_v2() -> NextestDiagnosticFollowupPlanV2 {
    NextestDiagnosticFollowupPlanV2 {
        plan_id: "nextest-diagnostic-followup-plan-v2".to_string(),
        nextest_followup: vec![
            "Use nextest only to narrow target families after truthful workspace boundaries are preserved.".to_string(),
            "Do not convert nextest evidence into full workspace acceptance.".to_string(),
        ],
        no_acceptance_claim: true,
        status: "NextestDiagnosticFollowupPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_sccache_diagnostic_followup_plan_v2() -> SccacheDiagnosticFollowupPlanV2 {
    SccacheDiagnosticFollowupPlanV2 {
        plan_id: "sccache-diagnostic-followup-plan-v2".to_string(),
        local_only_sccache_followup: vec![
            "Inspect local-only sccache hits or misses as optional diagnostics.".to_string(),
            "Do not promise speedup or acceptance from sccache evidence.".to_string(),
        ],
        no_guaranteed_speedup: true,
        status: "SccacheDiagnosticFollowupPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_environment_policy_report_v1() -> TimeoutEnvironmentPolicyReportV1 {
    TimeoutEnvironmentPolicyReportV1 {
        report_id: "timeout-environment-policy-v1".to_string(),
        timeout_command_policy: "Use local timeout wrapper when present; no remote execution."
            .to_string(),
        observation_environment:
            "Local-only workspace timeout observation with deterministic reporting.".to_string(),
        no_fake_timing: true,
        status: "TimeoutEnvironmentPolicyReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_command_wrapper_safety_report_v1() -> TimeoutCommandWrapperSafetyReportV1 {
    TimeoutCommandWrapperSafetyReportV1 {
        report_id: "timeout-command-wrapper-safety-v1".to_string(),
        wrapper_safety: "Wrapper may enforce timeout but cannot convert timeout into pass."
            .to_string(),
        child_cleanup: "Child cleanup must be measured with exact counts after wrapper completion."
            .to_string(),
        no_orphan_process_overclaim: true,
        status: "TimeoutCommandWrapperSafetyReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_child_process_cleanup_policy_v1() -> TimeoutChildProcessCleanupPolicyV1 {
    TimeoutChildProcessCleanupPolicyV1 {
        report_id: "timeout-child-process-cleanup-policy-v1".to_string(),
        cleanup_policy:
            "Collect actual cargo/rustc counts; cleanup evidence stays supporting-only.".to_string(),
        actual_counts_required: true,
        status: "TimeoutChildProcessCleanupPolicyReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_observation_repeat_plan_v1(
    config: &WorkspaceTimeoutReductionQueueConfig,
) -> TimeoutObservationRepeatPlanV1 {
    TimeoutObservationRepeatPlanV1 {
        plan_id: "timeout-observation-repeat-plan-v1".to_string(),
        repeat_attempts: vec![
            "cargo-json-boundary-repeat".to_string(),
            "no-run-boundary-repeat".to_string(),
            "full-workspace-boundary-repeat".to_string(),
        ],
        timeout_windows: vec![
            config.cargo_json_timeout_ms.unwrap_or(420_000),
            config.no_run_timeout_ms.unwrap_or(420_000),
            config.full_timeout_ms.unwrap_or(420_000),
        ],
        reason:
            "Repeat windows preserve truthful timeout boundaries while narrowing blocker families."
                .to_string(),
        status: "TimeoutObservationRepeatPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_truthful_no_run_attempt_v19(
    observation: Option<&CommandObservation>,
    timeout_ms: Option<u64>,
) -> TruthfulNoRunAttemptV19 {
    let attempted = observation.map(|value| value.attempted).unwrap_or(false);
    let finished = observation.map(|value| value.finished).unwrap_or(false);
    let timed_out = observation.map(|value| value.timed_out).unwrap_or(false);
    let passed = observation.and_then(|value| if value.finished { value.passed } else { None });
    let exit_code = observation.and_then(|value| value.exit_code);
    let recovered = finished && passed == Some(true);
    let status = if !attempted {
        "TruthfulNoRunAttemptNotRun"
    } else if timed_out {
        "TruthfulNoRunStillBlocked"
    } else if recovered {
        "TruthfulNoRunRecovered"
    } else if finished {
        "TruthfulNoRunFinishedFailed"
    } else {
        "TruthfulNoRunStillBlocked"
    };
    TruthfulNoRunAttemptV19 {
        report_id: "truthful-no-run-attempt-v19".to_string(),
        attempted,
        finished,
        passed,
        exit_code,
        timed_out,
        timeout_ms,
        recovered,
        status: status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_truthful_full_workspace_attempt_v19(
    observation: Option<&CommandObservation>,
    timeout_ms: Option<u64>,
) -> TruthfulFullWorkspaceAttemptV19 {
    let attempted = observation.map(|value| value.attempted).unwrap_or(false);
    let finished = observation.map(|value| value.finished).unwrap_or(false);
    let timed_out = observation.map(|value| value.timed_out).unwrap_or(false);
    let passed = observation.and_then(|value| if value.finished { value.passed } else { None });
    let exit_code = observation.and_then(|value| value.exit_code);
    let full_workspace_accepted = finished && passed == Some(true);
    let status = if !attempted {
        "TruthfulFullWorkspaceAttemptNotRun"
    } else if timed_out {
        "TruthfulFullWorkspaceStillBlocked"
    } else if full_workspace_accepted {
        "FullWorkspaceAccepted"
    } else if finished {
        "TruthfulFullWorkspaceFinishedFailed"
    } else {
        "TruthfulFullWorkspaceStillBlocked"
    };
    TruthfulFullWorkspaceAttemptV19 {
        report_id: "truthful-full-workspace-attempt-v19".to_string(),
        attempted,
        finished,
        passed,
        exit_code,
        timed_out,
        timeout_ms,
        full_workspace_accepted,
        status: status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_truthful_cargo_json_attempt_v19(
    observation: Option<&CommandObservation>,
    timeout_ms: Option<u64>,
) -> TruthfulCargoJsonAttemptV19 {
    let attempted = observation.map(|value| value.attempted).unwrap_or(false);
    let timed_out = observation.map(|value| value.timed_out).unwrap_or(false);
    let stdout = observation.map(|value| value.stdout.as_str()).unwrap_or("");
    let parsed_json_message_count = stdout
        .lines()
        .filter(|line| serde_json::from_str::<Value>(line).is_ok())
        .count() as u64;
    let reason_line_count = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count() as u64;
    let status = if !attempted {
        "TruthfulCargoJsonAttemptNotRun"
    } else if timed_out {
        "TruthfulCargoJsonTimedOut"
    } else {
        "TruthfulCargoJsonObserved"
    };
    TruthfulCargoJsonAttemptV19 {
        report_id: "truthful-cargo-json-attempt-v19".to_string(),
        attempted,
        parsed_json_message_count,
        reason_line_count,
        stderr_line_count: 0,
        timed_out,
        status: format!("{status}@{:?}", timeout_ms),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_no_run_attempt_comparison_v1(
    summary: &Sprint117SummaryFixture,
    attempt: &TruthfulNoRunAttemptV19,
) -> NoRunAttemptComparisonV1 {
    NoRunAttemptComparisonV1 {
        report_id: "no-run-attempt-comparison-v1".to_string(),
        comparison: vec![
            AttemptSnapshotV1 {
                sprint: "Sprint116".to_string(),
                status: "NoRunStillBlocked".to_string(),
                exit_code: Some(124),
                timed_out: true,
            },
            AttemptSnapshotV1 {
                sprint: "Sprint117".to_string(),
                status: summary.no_run_status.clone(),
                exit_code: summary.no_run_exit_code,
                timed_out: summary.no_run_exit_code == Some(124),
            },
            AttemptSnapshotV1 {
                sprint: "Sprint118".to_string(),
                status: attempt.status.clone(),
                exit_code: attempt.exit_code,
                timed_out: attempt.timed_out,
            },
        ],
        status: "NoRunAttemptComparisonReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_full_workspace_attempt_comparison_v1(
    summary: &Sprint117SummaryFixture,
    attempt: &TruthfulFullWorkspaceAttemptV19,
) -> FullWorkspaceAttemptComparisonV1 {
    FullWorkspaceAttemptComparisonV1 {
        report_id: "full-workspace-attempt-comparison-v1".to_string(),
        comparison: vec![
            AttemptSnapshotV1 {
                sprint: "Sprint116".to_string(),
                status: "FullWorkspaceStillBlocked".to_string(),
                exit_code: Some(124),
                timed_out: true,
            },
            AttemptSnapshotV1 {
                sprint: "Sprint117".to_string(),
                status: summary.full_workspace_status.clone(),
                exit_code: summary.full_exit_code,
                timed_out: summary.full_exit_code == Some(124),
            },
            AttemptSnapshotV1 {
                sprint: "Sprint118".to_string(),
                status: attempt.status.clone(),
                exit_code: attempt.exit_code,
                timed_out: attempt.timed_out,
            },
        ],
        status: "FullWorkspaceAttemptComparisonReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cargo_json_attempt_comparison_v1(
    summary: &Sprint117SummaryFixture,
    attempt: &TruthfulCargoJsonAttemptV19,
) -> CargoJsonAttemptComparisonV1 {
    CargoJsonAttemptComparisonV1 {
        report_id: "cargo-json-attempt-comparison-v1".to_string(),
        comparison: vec![
            AttemptSnapshotV1 {
                sprint: "Sprint116".to_string(),
                status: "CargoJsonStillNotRun".to_string(),
                exit_code: None,
                timed_out: false,
            },
            AttemptSnapshotV1 {
                sprint: "Sprint117".to_string(),
                status: summary.cargo_json_status.clone(),
                exit_code: summary.cargo_json_exit_code,
                timed_out: summary.cargo_json_exit_code == Some(124),
            },
            AttemptSnapshotV1 {
                sprint: "Sprint118".to_string(),
                status: attempt.status.clone(),
                exit_code: None,
                timed_out: attempt.timed_out,
            },
        ],
        status: "CargoJsonAttemptComparisonReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_no_run_recovery_gate_v19(
    no_run_attempt: &TruthfulNoRunAttemptV19,
) -> WorkspaceNoRunRecoveryGateV19 {
    WorkspaceNoRunRecoveryGateV19 {
        gate_id: "workspace-no-run-recovery-gate-v19".to_string(),
        recovered: no_run_attempt.finished && no_run_attempt.passed == Some(true),
        truthful_no_run_status: no_run_attempt.status.clone(),
        status: if no_run_attempt.finished && no_run_attempt.passed == Some(true) {
            "NoRunRecovered".to_string()
        } else {
            "NoRunStillBlocked".to_string()
        },
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_full_acceptance_gate_v19(
    full_attempt: &TruthfulFullWorkspaceAttemptV19,
) -> WorkspaceFullAcceptanceGateV19 {
    WorkspaceFullAcceptanceGateV19 {
        gate_id: "workspace-full-acceptance-gate-v19".to_string(),
        accepted: full_attempt.finished && full_attempt.passed == Some(true),
        truthful_full_status: full_attempt.status.clone(),
        status: if full_attempt.finished && full_attempt.passed == Some(true) {
            "FullWorkspaceAccepted".to_string()
        } else {
            "FullWorkspaceStillBlocked".to_string()
        },
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_focused_vs_full_bridge_v15() -> FocusedVsFullBridgeV15 {
    FocusedVsFullBridgeV15 {
        bridge_id: "focused-vs-full-bridge-v15".to_string(),
        supporting_only_labels: vec![
            "FocusedTests".to_string(),
            "CliSmoke".to_string(),
            "CargoBuild".to_string(),
            "CargoJsonReason".to_string(),
            "CargoJsonStderr".to_string(),
            "TimeoutCleanup".to_string(),
            "TruthfulNoRunV19".to_string(),
        ],
        status: "FocusedEvidenceSupportingOnly".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_acceptance_truth_gate_v19(
    full_gate: &WorkspaceFullAcceptanceGateV19,
) -> AcceptanceTruthGateV19 {
    AcceptanceTruthGateV19 {
        gate_id: "acceptance-truth-gate-v19".to_string(),
        can_claim_full_acceptance: full_gate.accepted,
        full_gate_status: full_gate.status.clone(),
        status: if full_gate.accepted {
            "AcceptanceTruthReady"
        } else {
            "AcceptanceTruthReadyWithWarnings"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_acceptance_evidence_strength_report_v8(
    acceptance_gate: &AcceptanceTruthGateV19,
    bridge: &FocusedVsFullBridgeV15,
) -> AcceptanceEvidenceStrengthReportV8 {
    AcceptanceEvidenceStrengthReportV8 {
        report_id: "acceptance-evidence-strength-v8".to_string(),
        full_evidence_sufficient: acceptance_gate.can_claim_full_acceptance,
        supporting_only_evidence: bridge.supporting_only_labels.clone(),
        status: if acceptance_gate.can_claim_full_acceptance {
            "AcceptanceEvidenceImproved"
        } else {
            "AcceptanceEvidenceStillSupportingOnly"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_evidence_matrix_v4(
    summary: &Sprint117SummaryFixture,
    no_run_attempt: &TruthfulNoRunAttemptV19,
    full_attempt: &TruthfulFullWorkspaceAttemptV19,
    cargo_attempt: &TruthfulCargoJsonAttemptV19,
    analysis: &CargoJsonFailureReasonAnalysisReportV1,
    hypothesis: &WorkspaceTimeoutReductionHypothesisReportV1,
    full_gate: &WorkspaceFullAcceptanceGateV19,
) -> WorkspaceTimeoutEvidenceMatrixV4 {
    let evidence_rows = vec![
        WorkspaceTimeoutEvidenceRowV4 {
            row_id: "Sprint117NoRun".to_string(),
            evidence_kind: "BaselineTruth".to_string(),
            status: summary.no_run_status.clone(),
            supports_acceptance: false,
        },
        WorkspaceTimeoutEvidenceRowV4 {
            row_id: "Sprint117Full".to_string(),
            evidence_kind: "BaselineTruth".to_string(),
            status: summary.full_workspace_status.clone(),
            supports_acceptance: false,
        },
        WorkspaceTimeoutEvidenceRowV4 {
            row_id: "Sprint117CargoJson".to_string(),
            evidence_kind: "BaselineTruth".to_string(),
            status: summary.cargo_json_status.clone(),
            supports_acceptance: false,
        },
        WorkspaceTimeoutEvidenceRowV4 {
            row_id: "CargoJsonReason".to_string(),
            evidence_kind: "SupportingOnly".to_string(),
            status: analysis.analysis_status.clone(),
            supports_acceptance: false,
        },
        WorkspaceTimeoutEvidenceRowV4 {
            row_id: "TimeoutReductionHypothesis".to_string(),
            evidence_kind: "SupportingOnly".to_string(),
            status: hypothesis.hypothesis_status.clone(),
            supports_acceptance: false,
        },
        WorkspaceTimeoutEvidenceRowV4 {
            row_id: "TruthfulNoRunV19".to_string(),
            evidence_kind: "SupportingOnly".to_string(),
            status: no_run_attempt.status.clone(),
            supports_acceptance: false,
        },
        WorkspaceTimeoutEvidenceRowV4 {
            row_id: "TruthfulFullV19".to_string(),
            evidence_kind: "AcceptanceGate".to_string(),
            status: full_attempt.status.clone(),
            supports_acceptance: full_gate.accepted,
        },
        WorkspaceTimeoutEvidenceRowV4 {
            row_id: "TruthfulCargoJsonV19".to_string(),
            evidence_kind: "SupportingOnly".to_string(),
            status: cargo_attempt.status.clone(),
            supports_acceptance: false,
        },
        WorkspaceTimeoutEvidenceRowV4 {
            row_id: "FocusedTests".to_string(),
            evidence_kind: "SupportingOnly".to_string(),
            status: format!("focused-tests-passed={}", summary.focused_tests_passed),
            supports_acceptance: false,
        },
        WorkspaceTimeoutEvidenceRowV4 {
            row_id: "CliSmoke".to_string(),
            evidence_kind: "SupportingOnly".to_string(),
            status: format!("cli-smoke-passed={}", summary.cli_smoke_passed),
            supports_acceptance: false,
        },
        WorkspaceTimeoutEvidenceRowV4 {
            row_id: "CargoBuild".to_string(),
            evidence_kind: "SupportingOnly".to_string(),
            status: format!("cargo-build-passed={}", summary.cargo_build_passed),
            supports_acceptance: false,
        },
    ];
    WorkspaceTimeoutEvidenceMatrixV4 {
        report_id: "workspace-timeout-evidence-matrix-v4".to_string(),
        evidence_rows,
        supports_acceptance: full_gate.accepted,
        status: if full_gate.accepted {
            "WorkspaceTimeoutEvidenceSupportsAcceptance"
        } else {
            "WorkspaceTimeoutEvidenceSupportingOnly"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_root_cause_report_v6(
    analysis: &CargoJsonFailureReasonAnalysisReportV1,
    hypothesis: &WorkspaceTimeoutReductionHypothesisReportV1,
) -> WorkspaceTimeoutRootCauseReportV6 {
    let narrowed = hypothesis.confidence == "Moderate" || hypothesis.confidence == "Strong";
    WorkspaceTimeoutRootCauseReportV6 {
        report_id: "workspace-timeout-root-cause-v6".to_string(),
        previous_root_cause:
            "Sprint 117 preserved truthful timeout observations without solving the blocker."
                .to_string(),
        cargo_json_reason_analysis: analysis.analysis_status.clone(),
        reduction_hypotheses: hypothesis.hypotheses.clone(),
        confidence: hypothesis.confidence.clone(),
        status: if narrowed {
            "TimeoutRootCauseNarrowed"
        } else {
            "TimeoutRootCauseStillOpen"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_reduction_progress_report_v1(
    queue: &WorkspaceTimeoutReductionQueueV1,
    experiment_report: &WorkspaceTimeoutReductionExperimentReportV1,
) -> TimeoutReductionProgressReportV1 {
    let reduction_queue_progress = vec![
        format!("queue_status={}", queue.queue_status),
        format!("planned={}", experiment_report.planned_experiments.len()),
        format!("ran={}", experiment_report.run_experiments.len()),
        format!("skipped={}", experiment_report.skipped_experiments.len()),
    ];
    TimeoutReductionProgressReportV1 {
        report_id: "timeout-reduction-progress-v1".to_string(),
        reduction_queue_progress,
        status: "TimeoutReductionProgressReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_reduction_risk_report_v1() -> TimeoutReductionRiskReportV1 {
    TimeoutReductionRiskReportV1 {
        report_id: "timeout-reduction-risk-v1".to_string(),
        overclaim_risk: "Queue readiness, cargo JSON classification, stderr, cleanup, focused tests, CLI smoke, and cargo build must stay supporting-only.".to_string(),
        parse_risk: "Cargo JSON may still be partial when timeout interrupts workspace progress.".to_string(),
        false_cleanup_risk: "Zero remaining children after wrapper completion is not equivalent to a passing test run.".to_string(),
        acceptance_confusion_risk: "No-run recovery never upgrades to full workspace acceptance.".to_string(),
        status: "TimeoutReductionRiskReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_consolidation_track_still_paused_report_v3() -> ConsolidationTrackStillPausedReportV3 {
    ConsolidationTrackStillPausedReportV3 {
        report_id: "consolidation-track-still-paused-v3".to_string(),
        paused: true,
        stopped: true,
        no_assertion_movement: true,
        no_target_retirement: true,
        status: "ConsolidationStillPaused".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_fifth_patch_still_not_applied_report_v3() -> FifthPatchStillNotAppliedReportV3 {
    FifthPatchStillNotAppliedReportV3 {
        report_id: "fifth-patch-still-not-applied-v3".to_string(),
        fifth_patch_applied: false,
        no_assertions_moved: true,
        no_targets_retired: true,
        status: "FifthPatchStillNotApplied".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_assertion_movement_still_forbidden_report_v3()
-> AssertionMovementStillForbiddenReportV3 {
    AssertionMovementStillForbiddenReportV3 {
        report_id: "assertion-movement-still-forbidden-v3".to_string(),
        movement_allowed: false,
        status: "AssertionMovementStillForbidden".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_target_retirement_still_forbidden_report_v3() -> TargetRetirementStillForbiddenReportV3
{
    TargetRetirementStillForbiddenReportV3 {
        report_id: "target-retirement-still-forbidden-v3".to_string(),
        retirement_allowed: false,
        status: "TargetRetirementStillForbidden".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_recovery_decision_report_v8(
    full_gate: &WorkspaceFullAcceptanceGateV19,
) -> WorkspaceRecoveryDecisionReportV8 {
    WorkspaceRecoveryDecisionReportV8 {
        report_id: "workspace-recovery-decision-v8".to_string(),
        continue_timeout_reduction_queue: !full_gate.accepted,
        keep_consolidation_paused: true,
        keep_fifth_patch_not_applied: true,
        status: if full_gate.accepted {
            "WorkspaceRecoveryDecisionAccepted"
        } else {
            "WorkspaceRecoveryDecisionContinueTimeoutReduction"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_reduction_next_action_queue_v1(
    queue: &WorkspaceTimeoutReductionQueueV1,
    full_gate: &WorkspaceFullAcceptanceGateV19,
) -> TimeoutReductionNextActionQueueV1 {
    TimeoutReductionNextActionQueueV1 {
        queue_id: "timeout-reduction-next-action-queue-v1".to_string(),
        next_queue_items: if full_gate.accepted {
            vec![
                "Hold truthful full pass evidence and keep consolidation paused until separately authorized."
                    .to_string(),
            ]
        } else {
            queue
                .ordered_items
                .iter()
                .map(|item| item.item_id.clone())
                .collect()
        },
        status: "TimeoutReductionNextActionQueueReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_control_tower_timeout_reduction_queue_panel(
    analysis: &CargoJsonFailureReasonAnalysisReportV1,
    hypothesis: &WorkspaceTimeoutReductionHypothesisReportV1,
    queue: &WorkspaceTimeoutReductionQueueV1,
    no_run_gate: &WorkspaceNoRunRecoveryGateV19,
    full_gate: &WorkspaceFullAcceptanceGateV19,
) -> ControlTowerTimeoutReductionQueuePanel {
    ControlTowerTimeoutReductionQueuePanel {
        panel_id: "control-tower-timeout-reduction-queue".to_string(),
        cargo_json_reason_analysis: analysis.analysis_status.clone(),
        timeout_reduction_hypotheses: hypothesis.hypotheses.clone(),
        ordered_queue: queue
            .ordered_items
            .iter()
            .map(|item| item.item_kind.clone())
            .collect(),
        truthful_no_run_gate: no_run_gate.status.clone(),
        truthful_full_gate: full_gate.status.clone(),
        warnings: warning_posture(),
        static_read_only: true,
        no_run_button: true,
        no_apply_button: true,
        no_train_runtime_live_order_account_controls: true,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_control_tower_acceptance_truth_panel_v19(
    no_run_gate: &WorkspaceNoRunRecoveryGateV19,
    full_gate: &WorkspaceFullAcceptanceGateV19,
    bridge: &FocusedVsFullBridgeV15,
) -> ControlTowerAcceptanceTruthPanelV19 {
    ControlTowerAcceptanceTruthPanelV19 {
        panel_id: "control-tower-acceptance-truth-v19".to_string(),
        full_gate: full_gate.status.clone(),
        no_run_gate: no_run_gate.status.clone(),
        supporting_evidence: bridge.supporting_only_labels.clone(),
        warnings: warning_posture(),
        static_read_only: true,
        no_action_button: true,
        no_train_runtime_live_order_account_controls: true,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_safety_coverage_preservation_report_v34() -> SafetyCoveragePreservationReportV34 {
    SafetyCoveragePreservationReportV34 {
        report_id: "safety-coverage-preservation-v34".to_string(),
        no_assertion_deletion: true,
        no_safety_sentinel_deletion: true,
        no_hidden_skips: true,
        timeout_reduction_queue_guard_present: true,
        cargo_json_reason_analysis_guard_present: true,
        truthful_attempt_gate_guard_present: true,
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

impl WorkspaceTimeoutReductionQueueBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            (
                "## 1. Sprint summary",
                format!(
                    "queue={} root_cause={} acceptance={}",
                    self.workspace_timeout_reduction_queue_v1.queue_status,
                    self.workspace_timeout_root_cause_report_v6.status,
                    self.acceptance_truth_gate_v19.status
                ),
            ),
            (
                "## 2. Why Sprint 118 was needed",
                "Sprint 117 produced truthful timeout observations but still left root-cause reduction and a new truthful full-workspace gate unresolved.".to_string(),
            ),
            (
                "## 3. Files added",
                "Sprint 118 adds timeout-reduction queue reports, truthful V19 gates, Control Tower panels, examples, fixtures, docs, and focused tests.".to_string(),
            ),
            (
                "## 4. Files changed",
                "Changes stay scoped to Sprint 118 timeout/root-cause reduction, acceptance truth, CLI, fixtures, docs, and tests.".to_string(),
            ),
            (
                "## 5. Sprint 117 baseline truth import",
                format!(
                    "{} imported_as_full_acceptance={}",
                    self.sprint117_baseline_truth_import_report.import_status,
                    self.sprint117_baseline_truth_import_report
                        .imported_as_full_acceptance
                ),
            ),
            (
                "## 6. Sprint 117 real observation carry-forward",
                format!(
                    "{} no_run={:?} full={:?} cargo_json={:?}",
                    self.sprint117_real_observation_carry_forward_report
                        .carry_forward_status,
                    self.sprint117_real_observation_carry_forward_report
                        .no_run_exit_code,
                    self.sprint117_real_observation_carry_forward_report
                        .full_exit_code,
                    self.sprint117_real_observation_carry_forward_report
                        .cargo_json_exit_code
                ),
            ),
            (
                "## 7. Cargo JSON failure reason analysis",
                format!(
                    "{} dominant={} parsed={} reason_lines={}",
                    self.cargo_json_failure_reason_analysis_report_v1
                        .analysis_status,
                    self.cargo_json_failure_reason_analysis_report_v1
                        .dominant_reason_class,
                    self.cargo_json_failure_reason_analysis_report_v1
                        .parsed_message_count,
                    self.cargo_json_failure_reason_analysis_report_v1
                        .reason_line_count
                ),
            ),
            (
                "## 8. Cargo JSON reason line classification",
                format!(
                    "{} unknown_lines={}",
                    self.cargo_json_reason_line_classification_report_v1.status,
                    self.cargo_json_reason_line_classification_report_v1
                        .unknown_line_count
                ),
            ),
            (
                "## 9. Cargo JSON stderr classification",
                format!(
                    "{} stderr_lines={}",
                    self.cargo_json_stderr_classification_report_v1.status,
                    self.cargo_json_stderr_classification_report_v1
                        .stderr_line_count
                ),
            ),
            (
                "## 10. Cargo JSON timeout pattern",
                format!(
                    "{} boundary={}",
                    self.cargo_json_timeout_pattern_report_v1.status,
                    self.cargo_json_timeout_pattern_report_v1.timeout_boundary
                ),
            ),
            (
                "## 11. Cargo JSON target blocker extraction",
                format!(
                    "{} suspects={}",
                    self.cargo_json_target_blocker_extraction_report_v1.status,
                    self.cargo_json_target_blocker_extraction_report_v1
                        .suspect_targets
                        .len()
                ),
            ),
            (
                "## 12. Workspace timeout reduction hypothesis",
                format!(
                    "{} confidence={} hypotheses={}",
                    self.workspace_timeout_reduction_hypothesis_report_v1
                        .hypothesis_status,
                    self.workspace_timeout_reduction_hypothesis_report_v1
                        .confidence,
                    self.workspace_timeout_reduction_hypothesis_report_v1
                        .hypotheses
                        .len()
                ),
            ),
            (
                "## 13. Workspace timeout reduction queue",
                format!(
                    "{} ordered_items={}",
                    self.workspace_timeout_reduction_queue_v1.queue_status,
                    self.workspace_timeout_reduction_queue_v1.ordered_items.len()
                ),
            ),
            (
                "## 14. Timeout reduction experiment plan",
                format!(
                    "{} experiments={}",
                    self.workspace_timeout_reduction_experiment_plan_v1.status,
                    self.workspace_timeout_reduction_experiment_plan_v1
                        .experiments
                        .len()
                ),
            ),
            (
                "## 15. Timeout reduction experiment report",
                format!(
                    "{} ran={} skipped={}",
                    self.workspace_timeout_reduction_experiment_report_v1.status,
                    self.workspace_timeout_reduction_experiment_report_v1
                        .run_experiments
                        .len(),
                    self.workspace_timeout_reduction_experiment_report_v1
                        .skipped_experiments
                        .len()
                ),
            ),
            (
                "## 16. No-run timeout reduction plan",
                self.no_run_timeout_reduction_plan_v1.status.clone(),
            ),
            (
                "## 17. Full workspace timeout reduction plan",
                self.full_workspace_timeout_reduction_plan_v1.status.clone(),
            ),
            (
                "## 18. Cargo JSON timeout reduction plan",
                self.cargo_json_timeout_reduction_plan_v1.status.clone(),
            ),
            (
                "## 19. Target family timeout reduction plan",
                self.target_family_timeout_reduction_plan_v1.status.clone(),
            ),
            (
                "## 20. Suspect target timeout reduction plan",
                format!(
                    "{} suspect_targets={}",
                    self.suspect_target_timeout_reduction_plan_v1.status,
                    self.suspect_target_timeout_reduction_plan_v1
                        .suspect_targets
                        .len()
                ),
            ),
            (
                "## 21. Link/macro timeout reduction plan",
                format!(
                    "{} hypotheses={}",
                    self.link_macro_timeout_reduction_plan_v1.status,
                    self.link_macro_timeout_reduction_plan_v1
                        .link_macro_hypotheses
                        .len()
                ),
            ),
            (
                "## 22. Integration fanout timeout reduction plan",
                format!(
                    "{} hypotheses={}",
                    self.integration_fanout_timeout_reduction_plan_v1.status,
                    self.integration_fanout_timeout_reduction_plan_v1
                        .integration_fanout_hypotheses
                        .len()
                ),
            ),
            (
                "## 23. Fixture/render/CLI timeout reduction plan",
                format!(
                    "{} hypotheses={}",
                    self.fixture_render_cli_timeout_reduction_plan_v1.status,
                    self.fixture_render_cli_timeout_reduction_plan_v1
                        .fixture_render_cli_hypotheses
                        .len()
                ),
            ),
            (
                "## 24. Nextest/sccache diagnostic follow-up plans",
                format!(
                    "nextest={} sccache={}",
                    self.nextest_diagnostic_followup_plan_v2.status,
                    self.sccache_diagnostic_followup_plan_v2.status
                ),
            ),
            (
                "## 25. Timeout environment policy",
                self.timeout_environment_policy_report_v1.status.clone(),
            ),
            (
                "## 26. Timeout command wrapper safety",
                self.timeout_command_wrapper_safety_report_v1.status.clone(),
            ),
            (
                "## 27. Timeout child process cleanup policy",
                self.timeout_child_process_cleanup_policy_v1.status.clone(),
            ),
            (
                "## 28. Timeout observation repeat plan",
                format!(
                    "{} attempts={}",
                    self.timeout_observation_repeat_plan_v1.status,
                    self.timeout_observation_repeat_plan_v1.repeat_attempts.len()
                ),
            ),
            (
                "## 29. Truthful no-run attempt v19",
                format!(
                    "{} attempted={} timed_out={} recovered={}",
                    self.truthful_no_run_attempt_v19.status,
                    self.truthful_no_run_attempt_v19.attempted,
                    self.truthful_no_run_attempt_v19.timed_out,
                    self.truthful_no_run_attempt_v19.recovered
                ),
            ),
            (
                "## 30. Truthful full workspace attempt v19",
                format!(
                    "{} attempted={} timed_out={} accepted={}",
                    self.truthful_full_workspace_attempt_v19.status,
                    self.truthful_full_workspace_attempt_v19.attempted,
                    self.truthful_full_workspace_attempt_v19.timed_out,
                    self.truthful_full_workspace_attempt_v19
                        .full_workspace_accepted
                ),
            ),
            (
                "## 31. Truthful cargo JSON attempt v19",
                format!(
                    "{} attempted={} parsed={} reason_lines={}",
                    self.truthful_cargo_json_attempt_v19.status,
                    self.truthful_cargo_json_attempt_v19.attempted,
                    self.truthful_cargo_json_attempt_v19
                        .parsed_json_message_count,
                    self.truthful_cargo_json_attempt_v19.reason_line_count
                ),
            ),
            (
                "## 32. Attempt comparisons",
                format!(
                    "no_run={} full={} cargo_json={}",
                    self.no_run_attempt_comparison_v1.status,
                    self.full_workspace_attempt_comparison_v1.status,
                    self.cargo_json_attempt_comparison_v1.status
                ),
            ),
            (
                "## 33. Workspace timeout evidence matrix v4",
                format!(
                    "{} supports_acceptance={}",
                    self.workspace_timeout_evidence_matrix_v4.status,
                    self.workspace_timeout_evidence_matrix_v4.supports_acceptance
                ),
            ),
            (
                "## 34. Workspace timeout root-cause v6",
                format!(
                    "{} confidence={}",
                    self.workspace_timeout_root_cause_report_v6.status,
                    self.workspace_timeout_root_cause_report_v6.confidence
                ),
            ),
            (
                "## 35. Timeout reduction progress",
                self.timeout_reduction_progress_report_v1.status.clone(),
            ),
            (
                "## 36. Timeout reduction risk",
                self.timeout_reduction_risk_report_v1.status.clone(),
            ),
            (
                "## 37. Consolidation track still paused v3",
                self.consolidation_track_still_paused_report_v3.status.clone(),
            ),
            (
                "## 38. Fifth patch still not applied v3",
                self.fifth_patch_still_not_applied_report_v3.status.clone(),
            ),
            (
                "## 39. Assertion movement still forbidden v3",
                self.assertion_movement_still_forbidden_report_v3
                    .status
                    .clone(),
            ),
            (
                "## 40. Target retirement still forbidden v3",
                self.target_retirement_still_forbidden_report_v3
                    .status
                    .clone(),
            ),
            (
                "## 41. Workspace no-run recovery gate v19",
                format!(
                    "{} recovered={}",
                    self.workspace_no_run_recovery_gate_v19.status,
                    self.workspace_no_run_recovery_gate_v19.recovered
                ),
            ),
            (
                "## 42. Workspace full acceptance gate v19",
                format!(
                    "{} accepted={}",
                    self.workspace_full_acceptance_gate_v19.status,
                    self.workspace_full_acceptance_gate_v19.accepted
                ),
            ),
            (
                "## 43. Focused-vs-full bridge v15",
                self.focused_vs_full_bridge_v15.status.clone(),
            ),
            (
                "## 44. Acceptance truth gate v19",
                format!(
                    "{} can_claim_full_acceptance={}",
                    self.acceptance_truth_gate_v19.status,
                    self.acceptance_truth_gate_v19.can_claim_full_acceptance
                ),
            ),
            (
                "## 45. Acceptance evidence strength v8",
                format!(
                    "{} full_evidence_sufficient={}",
                    self.acceptance_evidence_strength_report_v8.status,
                    self.acceptance_evidence_strength_report_v8
                        .full_evidence_sufficient
                ),
            ),
            (
                "## 46. Workspace recovery decision v8",
                format!(
                    "{} continue_timeout_reduction_queue={}",
                    self.workspace_recovery_decision_report_v8.status,
                    self.workspace_recovery_decision_report_v8
                        .continue_timeout_reduction_queue
                ),
            ),
            (
                "## 47. Timeout reduction next action queue",
                format!(
                    "{} items={}",
                    self.timeout_reduction_next_action_queue_v1.status,
                    self.timeout_reduction_next_action_queue_v1
                        .next_queue_items
                        .len()
                ),
            ),
            (
                "## 48. Safety coverage preservation v34",
                self.safety_coverage_preservation_report_v34
                    .safety_status
                    .clone(),
            ),
            (
                "## 49. Control Tower timeout reduction queue panel",
                format!(
                    "static_read_only={} ordered_queue={}",
                    self.control_tower_timeout_reduction_queue_panel
                        .static_read_only,
                    self.control_tower_timeout_reduction_queue_panel
                        .ordered_queue
                        .len()
                ),
            ),
            (
                "## 50. Control Tower acceptance truth panel v19",
                format!(
                    "static_read_only={} full_gate={}",
                    self.control_tower_acceptance_truth_panel_v19
                        .static_read_only,
                    self.control_tower_acceptance_truth_panel_v19.full_gate
                ),
            ),
            (
                "## 51. Output bundle",
                format!(
                    "file_count={} reduction_id={}",
                    self.storage_report.file_count,
                    self.workspace_timeout_reduction_queue_v1.queue_id
                ),
            ),
            (
                "## 52. CLI and examples",
                "Sprint 118 CLI examples are local-only, timeout-reduction-only, consolidation-paused, and report-only.".to_string(),
            ),
            (
                "## 53. Tests added",
                "Focused tests cover the queue, Sprint 117 import, cargo JSON analysis, target blockers, hypotheses, truthful gates, evidence matrix, acceptance truth, panels, CLI safety, determinism, and summary format.".to_string(),
            ),
            (
                "## 54. Test results",
                "Generated summary records Sprint 118 truth only; local verifier command results must be reported separately after execution.".to_string(),
            ),
            (
                "## 55. Timeout reduction queue status",
                self.workspace_timeout_reduction_queue_v1.queue_status.clone(),
            ),
            (
                "## 56. Cargo JSON reason status",
                self.cargo_json_failure_reason_analysis_report_v1
                    .analysis_status
                    .clone(),
            ),
            (
                "## 57. No-run recovery status",
                self.workspace_no_run_recovery_gate_v19.status.clone(),
            ),
            (
                "## 58. Full workspace acceptance status",
                self.workspace_full_acceptance_gate_v19.status.clone(),
            ),
            (
                "## 59. Acceptance evidence strength status",
                self.acceptance_evidence_strength_report_v8.status.clone(),
            ),
            (
                "## 60. Consolidation status",
                self.consolidation_track_still_paused_report_v3.status.clone(),
            ),
            (
                "## 61. Fifth patch status",
                self.fifth_patch_still_not_applied_report_v3.status.clone(),
            ),
            (
                "## 62. Runtime deferred status",
                format!(
                    "runtime_deferred={} training_deferred={} live_trading_forbidden={}",
                    self.safety_coverage_preservation_report_v34
                        .runtime_deferred,
                    self.safety_coverage_preservation_report_v34
                        .training_deferred,
                    self.safety_coverage_preservation_report_v34
                        .live_trading_forbidden
                ),
            ),
            (
                "## 63. Workspace acceptance truth status",
                format!(
                    "can_claim_full_acceptance={}",
                    self.acceptance_truth_gate_v19.can_claim_full_acceptance
                ),
            ),
            (
                "## 64. Safety coverage status",
                self.safety_coverage_preservation_report_v34
                    .safety_status
                    .clone(),
            ),
            (
                "## 65. Risk review",
                "Queue readiness, cargo JSON classification, stderr classification, cleanup, focused tests, CLI smoke, cargo build, and no-run cannot be treated as full workspace acceptance.".to_string(),
            ),
            (
                "## 66. Deferred items",
                "Runtime, training, live inference, live trading, broker/order/account, dashboard/browser/Tauri, broad consolidation, fifth patch, assertion movement, and target retirement remain out of scope.".to_string(),
            ),
            (
                "## 67. Next gstack sprint recommendation",
                "Continue timeout/root-cause reduction until a real cargo test --workspace --quiet run finishes and passes; keep consolidation paused meanwhile.".to_string(),
            ),
        ];
        sections
            .into_iter()
            .map(|(title, body)| format!("{title}\n\n{body}"))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub fn write_to_disk(
        &self,
        output_dir: &Path,
    ) -> Result<WorkspaceTimeoutReductionStorageReport, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let files = vec![
            (
                "sprint117_baseline_truth_import.txt",
                render_json(&self.sprint117_baseline_truth_import_report)?,
            ),
            (
                "sprint117_real_observation_carry_forward.txt",
                render_json(&self.sprint117_real_observation_carry_forward_report)?,
            ),
            (
                "cargo_json_failure_reason_analysis_v1.txt",
                render_json(&self.cargo_json_failure_reason_analysis_report_v1)?,
            ),
            (
                "cargo_json_reason_line_classification_v1.txt",
                render_json(&self.cargo_json_reason_line_classification_report_v1)?,
            ),
            (
                "cargo_json_stderr_classification_v1.txt",
                render_json(&self.cargo_json_stderr_classification_report_v1)?,
            ),
            (
                "cargo_json_timeout_pattern_v1.txt",
                render_json(&self.cargo_json_timeout_pattern_report_v1)?,
            ),
            (
                "cargo_json_target_blocker_extraction_v1.txt",
                render_json(&self.cargo_json_target_blocker_extraction_report_v1)?,
            ),
            (
                "workspace_timeout_reduction_hypothesis_v1.txt",
                render_json(&self.workspace_timeout_reduction_hypothesis_report_v1)?,
            ),
            (
                "workspace_timeout_reduction_queue_v1.txt",
                render_json(&self.workspace_timeout_reduction_queue_v1)?,
            ),
            (
                "workspace_timeout_reduction_experiment_plan_v1.txt",
                render_json(&self.workspace_timeout_reduction_experiment_plan_v1)?,
            ),
            (
                "workspace_timeout_reduction_experiment_report_v1.txt",
                render_json(&self.workspace_timeout_reduction_experiment_report_v1)?,
            ),
            (
                "no_run_timeout_reduction_plan_v1.txt",
                render_json(&self.no_run_timeout_reduction_plan_v1)?,
            ),
            (
                "full_workspace_timeout_reduction_plan_v1.txt",
                render_json(&self.full_workspace_timeout_reduction_plan_v1)?,
            ),
            (
                "cargo_json_timeout_reduction_plan_v1.txt",
                render_json(&self.cargo_json_timeout_reduction_plan_v1)?,
            ),
            (
                "target_family_timeout_reduction_plan_v1.txt",
                render_json(&self.target_family_timeout_reduction_plan_v1)?,
            ),
            (
                "suspect_target_timeout_reduction_plan_v1.txt",
                render_json(&self.suspect_target_timeout_reduction_plan_v1)?,
            ),
            (
                "link_macro_timeout_reduction_plan_v1.txt",
                render_json(&self.link_macro_timeout_reduction_plan_v1)?,
            ),
            (
                "integration_fanout_timeout_reduction_plan_v1.txt",
                render_json(&self.integration_fanout_timeout_reduction_plan_v1)?,
            ),
            (
                "fixture_render_cli_timeout_reduction_plan_v1.txt",
                render_json(&self.fixture_render_cli_timeout_reduction_plan_v1)?,
            ),
            (
                "nextest_diagnostic_followup_plan_v2.txt",
                render_json(&self.nextest_diagnostic_followup_plan_v2)?,
            ),
            (
                "sccache_diagnostic_followup_plan_v2.txt",
                render_json(&self.sccache_diagnostic_followup_plan_v2)?,
            ),
            (
                "timeout_environment_policy_v1.txt",
                render_json(&self.timeout_environment_policy_report_v1)?,
            ),
            (
                "timeout_command_wrapper_safety_v1.txt",
                render_json(&self.timeout_command_wrapper_safety_report_v1)?,
            ),
            (
                "timeout_child_process_cleanup_policy_v1.txt",
                render_json(&self.timeout_child_process_cleanup_policy_v1)?,
            ),
            (
                "timeout_observation_repeat_plan_v1.txt",
                render_json(&self.timeout_observation_repeat_plan_v1)?,
            ),
            (
                "truthful_no_run_attempt_v19.txt",
                render_json(&self.truthful_no_run_attempt_v19)?,
            ),
            (
                "truthful_full_workspace_attempt_v19.txt",
                render_json(&self.truthful_full_workspace_attempt_v19)?,
            ),
            (
                "truthful_cargo_json_attempt_v19.txt",
                render_json(&self.truthful_cargo_json_attempt_v19)?,
            ),
            (
                "no_run_attempt_comparison_v1.txt",
                render_json(&self.no_run_attempt_comparison_v1)?,
            ),
            (
                "full_workspace_attempt_comparison_v1.txt",
                render_json(&self.full_workspace_attempt_comparison_v1)?,
            ),
            (
                "cargo_json_attempt_comparison_v1.txt",
                render_json(&self.cargo_json_attempt_comparison_v1)?,
            ),
            (
                "workspace_timeout_evidence_matrix_v4.txt",
                render_json(&self.workspace_timeout_evidence_matrix_v4)?,
            ),
            (
                "workspace_timeout_root_cause_v6.txt",
                render_json(&self.workspace_timeout_root_cause_report_v6)?,
            ),
            (
                "timeout_reduction_progress_v1.txt",
                render_json(&self.timeout_reduction_progress_report_v1)?,
            ),
            (
                "timeout_reduction_risk_v1.txt",
                render_json(&self.timeout_reduction_risk_report_v1)?,
            ),
            (
                "consolidation_track_still_paused_v3.txt",
                render_json(&self.consolidation_track_still_paused_report_v3)?,
            ),
            (
                "fifth_patch_still_not_applied_v3.txt",
                render_json(&self.fifth_patch_still_not_applied_report_v3)?,
            ),
            (
                "assertion_movement_still_forbidden_v3.txt",
                render_json(&self.assertion_movement_still_forbidden_report_v3)?,
            ),
            (
                "target_retirement_still_forbidden_v3.txt",
                render_json(&self.target_retirement_still_forbidden_report_v3)?,
            ),
            (
                "workspace_no_run_recovery_gate_v19.txt",
                render_json(&self.workspace_no_run_recovery_gate_v19)?,
            ),
            (
                "workspace_full_acceptance_gate_v19.txt",
                render_json(&self.workspace_full_acceptance_gate_v19)?,
            ),
            (
                "focused_vs_full_bridge_v15.txt",
                render_json(&self.focused_vs_full_bridge_v15)?,
            ),
            (
                "acceptance_truth_gate_v19.txt",
                render_json(&self.acceptance_truth_gate_v19)?,
            ),
            (
                "acceptance_evidence_strength_v8.txt",
                render_json(&self.acceptance_evidence_strength_report_v8)?,
            ),
            (
                "workspace_recovery_decision_v8.txt",
                render_json(&self.workspace_recovery_decision_report_v8)?,
            ),
            (
                "timeout_reduction_next_action_queue_v1.txt",
                render_json(&self.timeout_reduction_next_action_queue_v1)?,
            ),
            (
                "safety_coverage_preservation_v34.txt",
                render_json(&self.safety_coverage_preservation_report_v34)?,
            ),
            (
                "control_tower_timeout_reduction_queue_panel.txt",
                render_json(&self.control_tower_timeout_reduction_queue_panel)?,
            ),
            (
                "control_tower_acceptance_truth_panel_v19.txt",
                render_json(&self.control_tower_acceptance_truth_panel_v19)?,
            ),
            ("summary.txt", self.final_summary.clone()),
        ];
        let mut written_files = Vec::new();
        for (name, value) in files {
            write_text_file(&output_dir.join(name), &value)?;
            written_files.push(name.to_string());
        }
        let storage_report = WorkspaceTimeoutReductionStorageReport {
            report_id: "workspace-timeout-reduction-storage-report".to_string(),
            output_dir: output_dir.display().to_string(),
            file_count: (written_files.len() + 1) as u64,
            written_files: {
                let mut files = written_files;
                files.push("storage_report.txt".to_string());
                files
            },
            reason_codes: diagnostic_reason_codes(&[]),
        };
        write_text_file(
            &output_dir.join("storage_report.txt"),
            &render_json(&storage_report)?,
        )?;
        Ok(storage_report)
    }
}

fn load_sprint117_summary(
    config: &WorkspaceTimeoutReductionQueueConfig,
) -> Result<Sprint117SummaryFixture, String> {
    load_first_json::<Sprint117SummaryFixture>(&[
        config.sprint117_truth_paths.as_ref(),
        config.sprint117_bundle_paths.as_ref(),
        config.cargo_json_parse_paths.as_ref(),
        config.cargo_json_execution_paths.as_ref(),
        config.no_run_execution_paths.as_ref(),
        config.full_execution_paths.as_ref(),
        config.timeout_cleanup_paths.as_ref(),
    ])
    .map(|value| value.unwrap_or_default())
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceTimeoutReductionQueueRunner;

impl WorkspaceTimeoutReductionQueueRunner {
    pub fn run(
        &self,
        config: &WorkspaceTimeoutReductionQueueConfig,
    ) -> Result<WorkspaceTimeoutReductionQueueBundle, String> {
        config.validate()?;
        let summary = load_sprint117_summary(config)?;
        let sprint117_baseline_truth_import_report =
            build_sprint117_baseline_truth_import_report(&summary);
        let sprint117_real_observation_carry_forward_report =
            build_sprint117_real_observation_carry_forward_report(&summary);
        let cargo_json_failure_reason_analysis_report_v1 =
            build_cargo_json_failure_reason_analysis_report_v1(&summary);
        let cargo_json_reason_line_classification_report_v1 =
            build_cargo_json_reason_line_classification_report_v1(&summary);
        let cargo_json_stderr_classification_report_v1 =
            build_cargo_json_stderr_classification_report_v1(&summary);
        let cargo_json_timeout_pattern_report_v1 =
            build_cargo_json_timeout_pattern_report_v1(&summary);
        let cargo_json_target_blocker_extraction_report_v1 =
            build_cargo_json_target_blocker_extraction_report_v1(&summary);
        let workspace_timeout_reduction_hypothesis_report_v1 =
            build_workspace_timeout_reduction_hypothesis_report_v1(
                &cargo_json_target_blocker_extraction_report_v1,
                &cargo_json_failure_reason_analysis_report_v1,
            );
        let workspace_timeout_reduction_queue_v1 = build_workspace_timeout_reduction_queue_v1(
            &workspace_timeout_reduction_hypothesis_report_v1,
        );
        let workspace_timeout_reduction_experiment_plan_v1 =
            build_workspace_timeout_reduction_experiment_plan_v1(
                &workspace_timeout_reduction_queue_v1,
            );
        let no_run_timeout_reduction_plan_v1 = build_no_run_timeout_reduction_plan_v1(config);
        let full_workspace_timeout_reduction_plan_v1 =
            build_full_workspace_timeout_reduction_plan_v1(config);
        let cargo_json_timeout_reduction_plan_v1 =
            build_cargo_json_timeout_reduction_plan_v1(config);
        let target_family_timeout_reduction_plan_v1 = build_target_family_timeout_reduction_plan_v1(
            &cargo_json_target_blocker_extraction_report_v1,
        );
        let suspect_target_timeout_reduction_plan_v1 =
            build_suspect_target_timeout_reduction_plan_v1(
                &cargo_json_target_blocker_extraction_report_v1,
            );
        let link_macro_timeout_reduction_plan_v1 = build_link_macro_timeout_reduction_plan_v1(
            &workspace_timeout_reduction_hypothesis_report_v1,
        );
        let integration_fanout_timeout_reduction_plan_v1 =
            build_integration_fanout_timeout_reduction_plan_v1(
                &workspace_timeout_reduction_hypothesis_report_v1,
            );
        let fixture_render_cli_timeout_reduction_plan_v1 =
            build_fixture_render_cli_timeout_reduction_plan_v1(
                &workspace_timeout_reduction_hypothesis_report_v1,
            );
        let nextest_diagnostic_followup_plan_v2 = build_nextest_diagnostic_followup_plan_v2();
        let sccache_diagnostic_followup_plan_v2 = build_sccache_diagnostic_followup_plan_v2();
        let timeout_environment_policy_report_v1 = build_timeout_environment_policy_report_v1();
        let timeout_command_wrapper_safety_report_v1 =
            build_timeout_command_wrapper_safety_report_v1();
        let timeout_child_process_cleanup_policy_v1 =
            build_timeout_child_process_cleanup_policy_v1();
        let timeout_observation_repeat_plan_v1 = build_timeout_observation_repeat_plan_v1(config);

        let cargo_json_observation = if config.run_truthful_cargo_json_attempt {
            Some(run_timed_command(
                "cargo test --workspace --no-run --message-format=json",
                config.cargo_json_timeout_ms,
            ))
        } else {
            None
        };
        let no_run_observation = if config.run_truthful_no_run_attempt {
            Some(run_timed_command(
                "cargo test --workspace --no-run --quiet",
                config.no_run_timeout_ms,
            ))
        } else {
            None
        };
        let full_observation = if config.run_truthful_full_workspace_attempt {
            Some(run_timed_command(
                "cargo test --workspace --quiet",
                config.full_timeout_ms,
            ))
        } else {
            None
        };
        let _cleanup_counts = (
            cleanup_counts_after_observation(&cargo_json_observation),
            cleanup_counts_after_observation(&no_run_observation),
            cleanup_counts_after_observation(&full_observation),
        );
        let truthful_no_run_attempt_v19 = build_truthful_no_run_attempt_v19(
            no_run_observation.as_ref(),
            config.no_run_timeout_ms,
        );
        let truthful_full_workspace_attempt_v19 = build_truthful_full_workspace_attempt_v19(
            full_observation.as_ref(),
            config.full_timeout_ms,
        );
        let truthful_cargo_json_attempt_v19 = build_truthful_cargo_json_attempt_v19(
            cargo_json_observation.as_ref(),
            config.cargo_json_timeout_ms,
        );
        let workspace_timeout_reduction_experiment_report_v1 =
            build_workspace_timeout_reduction_experiment_report_v1(
                &workspace_timeout_reduction_queue_v1,
                config,
            );
        let no_run_attempt_comparison_v1 =
            build_no_run_attempt_comparison_v1(&summary, &truthful_no_run_attempt_v19);
        let full_workspace_attempt_comparison_v1 = build_full_workspace_attempt_comparison_v1(
            &summary,
            &truthful_full_workspace_attempt_v19,
        );
        let cargo_json_attempt_comparison_v1 =
            build_cargo_json_attempt_comparison_v1(&summary, &truthful_cargo_json_attempt_v19);
        let workspace_no_run_recovery_gate_v19 =
            build_workspace_no_run_recovery_gate_v19(&truthful_no_run_attempt_v19);
        let workspace_full_acceptance_gate_v19 =
            build_workspace_full_acceptance_gate_v19(&truthful_full_workspace_attempt_v19);
        let focused_vs_full_bridge_v15 = build_focused_vs_full_bridge_v15();
        let acceptance_truth_gate_v19 =
            build_acceptance_truth_gate_v19(&workspace_full_acceptance_gate_v19);
        let acceptance_evidence_strength_report_v8 = build_acceptance_evidence_strength_report_v8(
            &acceptance_truth_gate_v19,
            &focused_vs_full_bridge_v15,
        );
        let workspace_timeout_evidence_matrix_v4 = build_workspace_timeout_evidence_matrix_v4(
            &summary,
            &truthful_no_run_attempt_v19,
            &truthful_full_workspace_attempt_v19,
            &truthful_cargo_json_attempt_v19,
            &cargo_json_failure_reason_analysis_report_v1,
            &workspace_timeout_reduction_hypothesis_report_v1,
            &workspace_full_acceptance_gate_v19,
        );
        let workspace_timeout_root_cause_report_v6 = build_workspace_timeout_root_cause_report_v6(
            &cargo_json_failure_reason_analysis_report_v1,
            &workspace_timeout_reduction_hypothesis_report_v1,
        );
        let timeout_reduction_progress_report_v1 = build_timeout_reduction_progress_report_v1(
            &workspace_timeout_reduction_queue_v1,
            &workspace_timeout_reduction_experiment_report_v1,
        );
        let timeout_reduction_risk_report_v1 = build_timeout_reduction_risk_report_v1();
        let consolidation_track_still_paused_report_v3 =
            build_consolidation_track_still_paused_report_v3();
        let fifth_patch_still_not_applied_report_v3 =
            build_fifth_patch_still_not_applied_report_v3();
        let assertion_movement_still_forbidden_report_v3 =
            build_assertion_movement_still_forbidden_report_v3();
        let target_retirement_still_forbidden_report_v3 =
            build_target_retirement_still_forbidden_report_v3();
        let workspace_recovery_decision_report_v8 =
            build_workspace_recovery_decision_report_v8(&workspace_full_acceptance_gate_v19);
        let timeout_reduction_next_action_queue_v1 = build_timeout_reduction_next_action_queue_v1(
            &workspace_timeout_reduction_queue_v1,
            &workspace_full_acceptance_gate_v19,
        );
        let safety_coverage_preservation_report_v34 =
            build_safety_coverage_preservation_report_v34();
        let control_tower_timeout_reduction_queue_panel =
            build_control_tower_timeout_reduction_queue_panel(
                &cargo_json_failure_reason_analysis_report_v1,
                &workspace_timeout_reduction_hypothesis_report_v1,
                &workspace_timeout_reduction_queue_v1,
                &workspace_no_run_recovery_gate_v19,
                &workspace_full_acceptance_gate_v19,
            );
        let control_tower_acceptance_truth_panel_v19 =
            build_control_tower_acceptance_truth_panel_v19(
                &workspace_no_run_recovery_gate_v19,
                &workspace_full_acceptance_gate_v19,
                &focused_vs_full_bridge_v15,
            );

        let mut bundle = WorkspaceTimeoutReductionQueueBundle {
            sprint117_baseline_truth_import_report,
            sprint117_real_observation_carry_forward_report,
            cargo_json_failure_reason_analysis_report_v1,
            cargo_json_reason_line_classification_report_v1,
            cargo_json_stderr_classification_report_v1,
            cargo_json_timeout_pattern_report_v1,
            cargo_json_target_blocker_extraction_report_v1,
            workspace_timeout_reduction_hypothesis_report_v1,
            workspace_timeout_reduction_queue_v1,
            workspace_timeout_reduction_experiment_plan_v1,
            workspace_timeout_reduction_experiment_report_v1,
            no_run_timeout_reduction_plan_v1,
            full_workspace_timeout_reduction_plan_v1,
            cargo_json_timeout_reduction_plan_v1,
            target_family_timeout_reduction_plan_v1,
            suspect_target_timeout_reduction_plan_v1,
            link_macro_timeout_reduction_plan_v1,
            integration_fanout_timeout_reduction_plan_v1,
            fixture_render_cli_timeout_reduction_plan_v1,
            nextest_diagnostic_followup_plan_v2,
            sccache_diagnostic_followup_plan_v2,
            timeout_environment_policy_report_v1,
            timeout_command_wrapper_safety_report_v1,
            timeout_child_process_cleanup_policy_v1,
            timeout_observation_repeat_plan_v1,
            truthful_no_run_attempt_v19,
            truthful_full_workspace_attempt_v19,
            truthful_cargo_json_attempt_v19,
            no_run_attempt_comparison_v1,
            full_workspace_attempt_comparison_v1,
            cargo_json_attempt_comparison_v1,
            workspace_timeout_evidence_matrix_v4,
            workspace_timeout_root_cause_report_v6,
            timeout_reduction_progress_report_v1,
            timeout_reduction_risk_report_v1,
            consolidation_track_still_paused_report_v3,
            fifth_patch_still_not_applied_report_v3,
            assertion_movement_still_forbidden_report_v3,
            target_retirement_still_forbidden_report_v3,
            workspace_no_run_recovery_gate_v19,
            workspace_full_acceptance_gate_v19,
            focused_vs_full_bridge_v15,
            acceptance_truth_gate_v19,
            acceptance_evidence_strength_report_v8,
            workspace_recovery_decision_report_v8,
            timeout_reduction_next_action_queue_v1,
            safety_coverage_preservation_report_v34,
            control_tower_timeout_reduction_queue_panel,
            control_tower_acceptance_truth_panel_v19,
            storage_report: WorkspaceTimeoutReductionStorageReport {
                report_id: "workspace-timeout-reduction-storage-report".to_string(),
                output_dir: config.output_dir().display().to_string(),
                written_files: Vec::new(),
                file_count: 51,
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
