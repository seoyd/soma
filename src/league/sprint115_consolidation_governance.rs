use crate::ReasonCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

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
    "target/soma_sprint115_consolidation_governance".to_string()
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
        "consolidation-governance-only",
        "fifth-patch-not-applied",
        "no-target-retirement",
        "no-assertion-movement",
        "stop-consolidation-is-valid",
        "focused-is-not-full",
        "CLI-smoke-is-not-full",
        "cargo-build-is-not-full",
        "no-run-is-not-full",
        "cargo-progress-is-not-acceptance",
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
pub struct ConsolidationStopResumeGovernanceConfig {
    pub governance_id: String,
    #[serde(default)]
    pub sprint114_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub sprint114_truth_paths: Option<Vec<String>>,
    #[serde(default)]
    pub assertion_inventory_paths: Option<Vec<String>>,
    #[serde(default)]
    pub destination_capacity_paths: Option<Vec<String>>,
    #[serde(default)]
    pub evidence_blur_paths: Option<Vec<String>>,
    #[serde(default)]
    pub workspace_timeout_paths: Option<Vec<String>>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub require_stop_resume_decision: bool,
    #[serde(default = "default_true")]
    pub require_assertion_destination_proof: bool,
    #[serde(default = "default_true")]
    pub require_evidence_blur_gate: bool,
    #[serde(default = "default_true")]
    pub require_track_split: bool,
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

impl Default for ConsolidationStopResumeGovernanceConfig {
    fn default() -> Self {
        Self {
            governance_id: "sprint115-consolidation-governance".to_string(),
            sprint114_bundle_paths: Some(vec![
                "examples/sprint115_data/sprint114_summary.json".to_string(),
            ]),
            sprint114_truth_paths: Some(vec![
                "examples/sprint115_data/sprint114_summary.json".to_string(),
            ]),
            assertion_inventory_paths: Some(vec![
                "examples/sprint115_data/assertion_destination_capacity_expected.json".to_string(),
            ]),
            destination_capacity_paths: Some(vec![
                "examples/sprint115_data/assertion_destination_capacity_expected.json".to_string(),
            ]),
            evidence_blur_paths: Some(vec![
                "examples/sprint115_data/evidence_blur_risk_expected.json".to_string(),
            ]),
            workspace_timeout_paths: Some(vec![
                "examples/sprint115_data/acceptance_truth_gate_v16_expected.json".to_string(),
            ]),
            output_root: default_output_root(),
            require_stop_resume_decision: true,
            require_assertion_destination_proof: true,
            require_evidence_blur_gate: true,
            require_track_split: true,
            allow_fifth_patch_application: false,
            allow_assertion_movement: false,
            allow_test_target_retirement: false,
            preserve_runtime_deferred: true,
            preserve_safety_guards: true,
            reason_codes: diagnostic_reason_codes(&[]),
        }
    }
}

impl ConsolidationStopResumeGovernanceConfig {
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
        PathBuf::from(&self.output_root).join(&self.governance_id)
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
        if self.governance_id.trim().is_empty() {
            return Err("sprint115 governance_id must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err("sprint115 output_root must be local-only".to_string());
        }
        Self::validate_paths(&self.sprint114_bundle_paths, "sprint114_bundle_paths")?;
        Self::validate_paths(&self.sprint114_truth_paths, "sprint114_truth_paths")?;
        Self::validate_paths(&self.assertion_inventory_paths, "assertion_inventory_paths")?;
        Self::validate_paths(
            &self.destination_capacity_paths,
            "destination_capacity_paths",
        )?;
        Self::validate_paths(&self.evidence_blur_paths, "evidence_blur_paths")?;
        Self::validate_paths(&self.workspace_timeout_paths, "workspace_timeout_paths")?;
        if !self.require_stop_resume_decision {
            return Err("require_stop_resume_decision must remain true".to_string());
        }
        if !self.require_assertion_destination_proof {
            return Err("require_assertion_destination_proof must remain true".to_string());
        }
        if !self.require_evidence_blur_gate {
            return Err("require_evidence_blur_gate must remain true".to_string());
        }
        if !self.require_track_split {
            return Err("require_track_split must remain true".to_string());
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
pub struct Sprint114SummaryFixture {
    pub report_id: String,
    pub mixed_family_status: String,
    pub assertion_migration_status: String,
    pub fifth_patch_status: String,
    pub stop_consolidation_status: String,
    pub no_run_status: String,
    pub full_workspace_status: String,
    pub acceptance_truth_status: String,
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
    pub isolated_families: Vec<String>,
    pub still_mixed_families: Vec<String>,
    pub suspect_targets: Vec<String>,
    pub assertion_count_by_target: BTreeMap<String, u64>,
    pub assertion_kinds_by_target: BTreeMap<String, Vec<String>>,
    pub assertion_dependencies: BTreeMap<String, Vec<String>>,
    pub migration_complexity: BTreeMap<String, String>,
    pub existing_migrated_assertion_count: u64,
    pub further_migration_capacity: u64,
    pub equivalent_coverage_feasible: bool,
    pub sentinel_safety_preserved: bool,
    pub no_hidden_skip_preserved: bool,
    pub mixed_family_evidence_narrowed_enough: bool,
    pub integration_observed_evidence: Vec<String>,
    pub integration_inferred_evidence: Vec<String>,
    pub link_observed_evidence: Vec<String>,
    pub link_inferred_evidence: Vec<String>,
    pub macro_observed_evidence: Vec<String>,
    pub macro_inferred_evidence: Vec<String>,
    pub cumulative_sample_backed_delta: i64,
}

impl Default for Sprint114SummaryFixture {
    fn default() -> Self {
        let suspect_targets = vec![
            "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
            "tests/shared_fixture_harness_application_v1.rs".to_string(),
            "tests/workspace_timeout_root_cause.rs".to_string(),
        ];
        Self {
            report_id: "sprint114-summary".to_string(),
            mixed_family_status: "MixedFamiliesStillAmbiguous".to_string(),
            assertion_migration_status: "AssertionMigrationBlocked".to_string(),
            fifth_patch_status: "FifthPatchStillBlocked".to_string(),
            stop_consolidation_status: "StopConsolidationRecommended".to_string(),
            no_run_status: "NoRunStillBlocked".to_string(),
            full_workspace_status: "FullWorkspaceStillBlocked".to_string(),
            acceptance_truth_status: "AcceptanceTruthReadyWithWarnings".to_string(),
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
            suspect_targets,
            assertion_count_by_target: BTreeMap::from([
                (
                    "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
                    6,
                ),
                (
                    "tests/shared_fixture_harness_application_v1.rs".to_string(),
                    5,
                ),
                ("tests/workspace_timeout_root_cause.rs".to_string(), 9),
            ]),
            assertion_kinds_by_target: BTreeMap::from([
                (
                    "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
                    vec![
                        "cli-warning-posture".to_string(),
                        "panel-render".to_string(),
                        "timeout-interpretation".to_string(),
                    ],
                ),
                (
                    "tests/shared_fixture_harness_application_v1.rs".to_string(),
                    vec![
                        "fixture-setup".to_string(),
                        "shared-harness".to_string(),
                        "deterministic-output".to_string(),
                    ],
                ),
                (
                    "tests/workspace_timeout_root_cause.rs".to_string(),
                    vec![
                        "root-cause-split".to_string(),
                        "evidence-matrix".to_string(),
                        "acceptance-warning".to_string(),
                    ],
                ),
            ]),
            assertion_dependencies: BTreeMap::from([
                (
                    "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
                    vec![
                        "control tower warning renderer".to_string(),
                        "timeout panel bundle text".to_string(),
                    ],
                ),
                (
                    "tests/shared_fixture_harness_application_v1.rs".to_string(),
                    vec!["shared fixture harness helpers".to_string()],
                ),
                (
                    "tests/workspace_timeout_root_cause.rs".to_string(),
                    vec![
                        "root cause evidence split".to_string(),
                        "acceptance truth gate wording".to_string(),
                    ],
                ),
            ]),
            migration_complexity: BTreeMap::from([
                (
                    "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
                    "High".to_string(),
                ),
                (
                    "tests/shared_fixture_harness_application_v1.rs".to_string(),
                    "Medium".to_string(),
                ),
                (
                    "tests/workspace_timeout_root_cause.rs".to_string(),
                    "High".to_string(),
                ),
            ]),
            existing_migrated_assertion_count: 2,
            further_migration_capacity: 1,
            equivalent_coverage_feasible: true,
            sentinel_safety_preserved: true,
            no_hidden_skip_preserved: true,
            mixed_family_evidence_narrowed_enough: false,
            integration_observed_evidence: vec![
                "cargo no-run timeout still ends near integration test binary fanout".to_string(),
                "focused matrix passed while integration binary fanout stayed mixed".to_string(),
            ],
            integration_inferred_evidence: vec![
                "shared fixture harness likely amplifies integration binary count".to_string(),
                "control tower panel transitively depends on integration binary surfaces"
                    .to_string(),
            ],
            link_observed_evidence: vec![
                "rustc timeline last heavy link candidate was control tower timeout panel"
                    .to_string(),
            ],
            link_inferred_evidence: vec![
                "workspace timeout root cause target may still add linker pressure".to_string(),
            ],
            macro_observed_evidence: vec![
                "cargo json trace stalled after workspace_timeout_root_cause artifact emission"
                    .to_string(),
            ],
            macro_inferred_evidence: vec![
                "macro-heavy report rendering remains coupled to timeout target".to_string(),
            ],
            cumulative_sample_backed_delta: -4,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecisionMatrixRowV1 {
    pub decision: String,
    pub selected: bool,
    pub blockers: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssertionDestinationCapacityRowV1 {
    pub target: String,
    pub current_assertion_load: u64,
    pub migrated_assertion_load: u64,
    pub warning_assertion_load: u64,
    pub remaining_capacity: i64,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObservationBacklogRowV1 {
    pub item: String,
    pub priority: String,
    pub rationale: String,
}

report!(Sprint114BaselineTruthImportReport {
    report_id: String,
    mixed_family_status: String,
    assertion_migration_status: String,
    fifth_patch_status: String,
    stop_consolidation_status: String,
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
report!(Sprint114StopRecommendationCarryForwardReport {
    report_id: String,
    stop_recommendation_present: bool,
    stop_reason: String,
    fifth_patch_blocked: bool,
    assertion_migration_blocked: bool,
    risk_review_refs: Vec<String>,
    carry_forward_status: String
});
report!(ConsolidationStopDecisionReportV1 {
    report_id: String,
    stop_recommended: bool,
    stop_reasons: Vec<String>,
    stop_status: String
});
report!(ConsolidationResumeDecisionReportV1 {
    report_id: String,
    resume_recommended: bool,
    required_proofs: Vec<String>,
    missing_proofs: Vec<String>,
    resume_status: String
});
report!(ConsolidationDecisionMatrixV1 {
    matrix_id: String,
    decision_rows: Vec<DecisionMatrixRowV1>,
    selected_decision: String,
    matrix_status: String
});
report!(AssertionDestinationProofPlanV1 {
    plan_id: String,
    destination_candidates: Vec<String>,
    proof_requirements: Vec<String>,
    plan_status: String
});
report!(AssertionDestinationCapacityReportV1 {
    report_id: String,
    destination_targets: Vec<AssertionDestinationCapacityRowV1>,
    current_assertion_load: u64,
    migrated_assertion_load: u64,
    warning_assertion_load: u64,
    remaining_capacity: i64,
    capacity_status: String
});
report!(SharedFixtureHarnessCapacityReportV1 {
    report_id: String,
    target: String,
    current_migrated_assertions: u64,
    risk_of_overload: bool,
    deterministic_fixture_safety: bool,
    remaining_capacity: i64,
    status: String
});
report!(WorkspaceTimeoutTargetCapacityReportV1 {
    report_id: String,
    target: String,
    current_diagnostic_role: String,
    can_receive_assertions: bool,
    evidence_blur_risk: String,
    status: String
});
report!(ControlTowerAssertionMoveRiskReportV1 {
    report_id: String,
    target: String,
    warning_assertion_risk: String,
    ui_panel_evidence_blur_risk: String,
    status: String
});
report!(EvidenceBlurRiskReportV1 {
    report_id: String,
    candidate_moves: Vec<String>,
    blur_risks: Vec<String>,
    high_risk_moves: Vec<String>,
    blur_status: String
});
report!(AssertionMoveSemanticDriftReportV1 {
    report_id: String,
    semantic_drift_risk: String,
    blockers: Vec<String>,
    status: String
});
report!(AssertionMoveDeterminismRiskReportV1 {
    report_id: String,
    determinism_risk: String,
    blockers: Vec<String>,
    status: String
});
report!(AssertionMoveCliSurfaceRiskReportV1 {
    report_id: String,
    cli_surface_risk: String,
    blockers: Vec<String>,
    status: String
});
report!(AssertionMoveSafetyRiskReportV1 {
    report_id: String,
    safety_risk: String,
    blockers: Vec<String>,
    status: String
});
report!(AssertionDestinationProofGateV1 {
    gate_id: String,
    capacity_status: String,
    semantic_drift_status: String,
    determinism_risk_status: String,
    cli_surface_risk_status: String,
    safety_risk_status: String,
    equivalent_coverage_status: String,
    proof_complete: bool,
    gate_status: String
});
report!(EvidenceBlurRiskGateV1 {
    gate_id: String,
    evidence_blur_status: String,
    gate_status: String
});
report!(FifthPatchResumeGateV5 {
    gate_id: String,
    assertion_destination_proof_status: String,
    evidence_blur_gate_status: String,
    stop_decision_status: String,
    safety_status: String,
    acceptance_truth_status: String,
    resume_allowed_for_later_sprint: bool,
    fifth_patch_applied_this_sprint: bool,
    gate_status: String
});
report!(FifthPatchStopGateV1 {
    gate_id: String,
    stop_recommended: bool,
    fifth_patch_blocked: bool,
    gate_status: String
});
report!(FifthPatchNoApplyGuaranteeReportV4 {
    report_id: String,
    fifth_patch_applied: bool,
    retired_files: Vec<String>,
    moved_assertions: Vec<String>,
    retired_targets: Vec<String>,
    guarantee_status: String
});
report!(CandidateStopConsolidationReportV2 {
    report_id: String,
    stop_recommended: bool,
    stop_reason: String,
    resume_allowed_with_proof: bool,
    status: String
});
report!(ConsolidationTrackPauseReportV1 {
    report_id: String,
    paused: bool,
    pause_reason: String,
    resume_prerequisites: Vec<String>,
    status: String
});
report!(WorkspaceTimeoutTrackSplitReportV1 {
    report_id: String,
    consolidation_track_status: String,
    timeout_track_status: String,
    separation_complete: bool,
    status: String
});
report!(WorkspaceTimeoutDiagnosticTrackPlanV1 {
    plan_id: String,
    no_run_observation_backlog: Vec<String>,
    full_observation_backlog: Vec<String>,
    cargo_json_backlog: Vec<String>,
    target_level_diagnostics: Vec<String>,
    nextest_sccache_follow_up: Vec<String>,
    status: String
});
report!(WorkspaceTimeoutObservationBacklogV1 {
    report_id: String,
    queued_observations: Vec<ObservationBacklogRowV1>,
    priority: String,
    status: String
});
report!(WorkspaceNoRunObservationPlanV2 {
    plan_id: String,
    next_no_run_strategy: Vec<String>,
    timeout_seconds: u64,
    observation_quality: String,
    status: String
});
report!(WorkspaceFullObservationPlanV2 {
    plan_id: String,
    next_full_strategy: Vec<String>,
    timeout_seconds: u64,
    status: String
});
report!(CargoJsonObservationPlanV2 {
    plan_id: String,
    cargo_json_capture_strategy: Vec<String>,
    status: String
});
report!(TargetFamilyDiagnosticBacklogV1 {
    report_id: String,
    mixed_families: Vec<String>,
    backlog_status: String
});
report!(LinkMacroDiagnosticBacklogV1 {
    report_id: String,
    link_macro_diagnostics: Vec<String>,
    status: String
});
report!(IntegrationFanoutDiagnosticBacklogV1 {
    report_id: String,
    integration_fanout_diagnostics: Vec<String>,
    status: String
});
report!(CumulativeSafePatchLedgerV6 {
    report_id: String,
    carried_patch_ids: Vec<String>,
    fifth_patch_applied: bool,
    status: String
});
report!(CumulativeBinaryDeltaReportV5 {
    report_id: String,
    sample_backed_delta: i64,
    measured_claim_allowed: bool,
    status: String
});
report!(AssertionLedgerContinuityCheckV5 {
    report_id: String,
    continuity_preserved: bool,
    status: String
});
report!(EquivalentCoverageContinuityCheckV5 {
    report_id: String,
    continuity_preserved: bool,
    status: String
});
report!(SafetySentinelContinuityCheckV5 {
    report_id: String,
    continuity_preserved: bool,
    status: String
});
report!(NoHiddenSkipContinuityCheckV5 {
    report_id: String,
    continuity_preserved: bool,
    status: String
});
report!(TimeoutCleanupVerificationReportV8 {
    report_id: String,
    cleanup_verified: bool,
    remaining_cargo_processes: u64,
    remaining_rustc_processes: u64,
    cleanup_status: String
});
report!(WorkspaceNoRunRecoveryGateV16 {
    gate_id: String,
    command: String,
    finished: bool,
    passed: bool,
    timed_out: bool,
    recovered: bool,
    gate_status: String
});
report!(WorkspaceFullAcceptanceGateV16 {
    gate_id: String,
    command: String,
    finished: bool,
    passed: bool,
    accepted: bool,
    gate_status: String
});
report!(FocusedVsFullBridgeV12 {
    bridge_id: String,
    focused_tests_status: String,
    cli_smoke_status: String,
    cargo_build_status: String,
    no_run_status: String,
    full_status: String,
    bridge_status: String
});
report!(AcceptanceTruthGateV16 {
    gate_id: String,
    focused_truth_status: String,
    cli_truth_status: String,
    cargo_check_truth_status: String,
    cargo_build_truth_status: String,
    no_run_truth_status: String,
    full_workspace_truth_status: String,
    can_claim_full_acceptance: bool,
    truth_status: String
});
report!(AcceptanceEvidenceStrengthReportV5 {
    report_id: String,
    evidence_tiers: Vec<String>,
    strongest_claim: String,
    report_status: String
});
report!(WorkspaceRecoveryDecisionReportV5 {
    report_id: String,
    recommend_stop_consolidation: bool,
    recommend_timeout_diagnostic_track: bool,
    recommend_resume_only_with_proof: bool,
    no_run_recovered: bool,
    full_workspace_accepted: bool,
    decision_status: String
});
report!(SafetyCoveragePreservationReportV31 {
    report_id: String,
    no_assertion_deletion: bool,
    no_safety_sentinel_deletion: bool,
    no_hidden_skips: bool,
    consolidation_stop_resume_guard_present: bool,
    assertion_destination_proof_guard_present: bool,
    evidence_blur_risk_guard_present: bool,
    track_split_guard_present: bool,
    fifth_patch_v5_no_apply_guard_present: bool,
    no_target_retirement_guard_present: bool,
    no_assertion_movement_guard_present: bool,
    runtime_deferred: bool,
    training_deferred: bool,
    live_trading_forbidden: bool,
    safety_status: String
});
report!(ControlTowerConsolidationGovernancePanel {
    panel_id: String,
    stop_decision_status: String,
    resume_decision_status: String,
    proof_gate_status: String,
    evidence_blur_status: String,
    fifth_patch_status: String,
    warnings: Vec<String>,
    static_read_only: bool,
    no_apply_button: bool,
    no_run_button: bool,
    no_train_runtime_live_order_account_controls: bool
});
report!(ControlTowerWorkspaceTimeoutTrackPanel {
    panel_id: String,
    timeout_track_status: String,
    backlog_status: String,
    no_run_status: String,
    full_status: String,
    acceptance_truth_status: String,
    warnings: Vec<String>,
    static_read_only: bool,
    no_run_button: bool,
    no_train_runtime_live_order_account_controls: bool
});
report!(ConsolidationStopResumeGovernanceStorageReport {
    report_id: String,
    output_dir: String,
    written_files: Vec<String>,
    file_count: u64
});

pub fn build_sprint114_baseline_truth_import_report(
    summary: &Sprint114SummaryFixture,
) -> Sprint114BaselineTruthImportReport {
    Sprint114BaselineTruthImportReport {
        report_id: "sprint114-baseline-truth-import".to_string(),
        mixed_family_status: summary.mixed_family_status.clone(),
        assertion_migration_status: summary.assertion_migration_status.clone(),
        fifth_patch_status: summary.fifth_patch_status.clone(),
        stop_consolidation_status: summary.stop_consolidation_status.clone(),
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
            "Sprint114TruthImportedWithWarnings"
        } else {
            "Sprint114TruthImported"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_sprint114_stop_recommendation_carry_forward_report(
    baseline: &Sprint114BaselineTruthImportReport,
) -> Sprint114StopRecommendationCarryForwardReport {
    Sprint114StopRecommendationCarryForwardReport {
        report_id: "sprint114-stop-recommendation-carry-forward".to_string(),
        stop_recommendation_present: baseline.stop_consolidation_status == "StopConsolidationRecommended",
        stop_reason: "Sprint 114 left AssertionMigrationBlocked and FifthPatchStillBlocked, so StopConsolidationRecommended carries forward until proof and evidence blur gates pass.".to_string(),
        fifth_patch_blocked: baseline.fifth_patch_status == "FifthPatchStillBlocked",
        assertion_migration_blocked: baseline.assertion_migration_status == "AssertionMigrationBlocked",
        risk_review_refs: vec![
            "AssertionDestinationProofGateV1".to_string(),
            "EvidenceBlurRiskGateV1".to_string(),
            "WorkspaceTimeoutTrackSplitReportV1".to_string(),
        ],
        carry_forward_status: "StopRecommendationCarriedForwardWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_consolidation_stop_decision_report_v1(
    baseline: &Sprint114BaselineTruthImportReport,
) -> ConsolidationStopDecisionReportV1 {
    let mut stop_reasons = vec![
        "AssertionMigrationStillBlocked".to_string(),
        "NeedMoreProof".to_string(),
    ];
    if !baseline.mixed_family_status.is_empty() {
        stop_reasons.push("EvidenceBlurRiskTooHigh".to_string());
        stop_reasons.push("DestinationCapacityInsufficient".to_string());
        stop_reasons.push("WorkspaceTimeoutTrackDominates".to_string());
    }
    ConsolidationStopDecisionReportV1 {
        report_id: "consolidation-stop-decision-v1".to_string(),
        stop_recommended: true,
        stop_reasons,
        stop_status: "ConsolidationStopRecommendedWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_consolidation_resume_decision_report_v1(
    proof_gate: &AssertionDestinationProofGateV1,
    blur_gate: &EvidenceBlurRiskGateV1,
) -> ConsolidationResumeDecisionReportV1 {
    let required_proofs = vec![
        "AssertionDestinationProofReady".to_string(),
        "EvidenceBlurRiskControlled".to_string(),
        "EquivalentCoverageContinuityPreserved".to_string(),
        "SafetySentinelContinuityPreserved".to_string(),
    ];
    let mut missing_proofs = Vec::new();
    if !proof_gate.proof_complete {
        missing_proofs.push("AssertionDestinationProofReady".to_string());
    }
    if blur_gate.gate_status != "EvidenceBlurRiskControlled"
        && blur_gate.gate_status != "EvidenceBlurRiskControlledWithWarnings"
    {
        missing_proofs.push("EvidenceBlurRiskControlled".to_string());
    }
    if !proof_gate.equivalent_coverage_status.contains("Preserved") {
        missing_proofs.push("EquivalentCoverageContinuityPreserved".to_string());
    }
    if !proof_gate.safety_risk_status.contains("Preserved") {
        missing_proofs.push("SafetySentinelContinuityPreserved".to_string());
    }
    let resume_status = if missing_proofs.is_empty() {
        "ConsolidationResumeAllowedLater"
    } else {
        "ConsolidationResumeNeedsProof"
    };
    ConsolidationResumeDecisionReportV1 {
        report_id: "consolidation-resume-decision-v1".to_string(),
        resume_recommended: missing_proofs.is_empty(),
        required_proofs,
        missing_proofs,
        resume_status: resume_status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_consolidation_decision_matrix_v1(
    stop: &ConsolidationStopDecisionReportV1,
    resume: &ConsolidationResumeDecisionReportV1,
    split: &WorkspaceTimeoutTrackSplitReportV1,
) -> ConsolidationDecisionMatrixV1 {
    let selected_decision = if stop.stop_recommended {
        "StopConsolidation"
    } else if resume.resume_recommended {
        "ResumeOnlyWithProof"
    } else if split.separation_complete {
        "PauseConsolidation"
    } else {
        "SplitWorkspaceTimeoutTrack"
    };
    let decision_rows = vec![
        DecisionMatrixRowV1 {
            decision: "ContinueConsolidation".to_string(),
            selected: false,
            blockers: vec![
                stop.stop_status.clone(),
                resume.resume_status.clone(),
                "EvidenceBlurRiskTooHigh".to_string(),
            ],
        },
        DecisionMatrixRowV1 {
            decision: "PauseConsolidation".to_string(),
            selected: selected_decision == "PauseConsolidation",
            blockers: vec![],
        },
        DecisionMatrixRowV1 {
            decision: "StopConsolidation".to_string(),
            selected: selected_decision == "StopConsolidation",
            blockers: vec![],
        },
        DecisionMatrixRowV1 {
            decision: "SplitWorkspaceTimeoutTrack".to_string(),
            selected: selected_decision == "SplitWorkspaceTimeoutTrack",
            blockers: vec![],
        },
        DecisionMatrixRowV1 {
            decision: "ResumeOnlyWithProof".to_string(),
            selected: selected_decision == "ResumeOnlyWithProof",
            blockers: resume.missing_proofs.clone(),
        },
    ];
    ConsolidationDecisionMatrixV1 {
        matrix_id: "consolidation-decision-matrix-v1".to_string(),
        decision_rows,
        selected_decision: selected_decision.to_string(),
        matrix_status: "ConsolidationDecisionReadyWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_assertion_destination_proof_plan_v1() -> AssertionDestinationProofPlanV1 {
    AssertionDestinationProofPlanV1 {
        plan_id: "assertion-destination-proof-plan-v1".to_string(),
        destination_candidates: vec![
            "tests/shared_fixture_harness_application_v1.rs".to_string(),
            "tests/workspace_timeout_root_cause.rs".to_string(),
            "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
        ],
        proof_requirements: vec![
            "DestinationCapacity".to_string(),
            "SemanticIsolation".to_string(),
            "DeterminismPreservation".to_string(),
            "CliSurfacePreservation".to_string(),
            "SafetyPreservation".to_string(),
            "EvidenceClarityPreservation".to_string(),
            "EquivalentCoverage".to_string(),
        ],
        plan_status: "AssertionDestinationProofPlanReadyWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_shared_fixture_harness_capacity_report_v1(
    summary: &Sprint114SummaryFixture,
) -> SharedFixtureHarnessCapacityReportV1 {
    SharedFixtureHarnessCapacityReportV1 {
        report_id: "shared-fixture-harness-capacity-v1".to_string(),
        target: "tests/shared_fixture_harness_application_v1.rs".to_string(),
        current_migrated_assertions: summary.existing_migrated_assertion_count,
        risk_of_overload: true,
        deterministic_fixture_safety: true,
        remaining_capacity: summary.further_migration_capacity as i64,
        status: "DestinationCapacityReadyWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_target_capacity_report_v1(
    summary: &Sprint114SummaryFixture,
) -> WorkspaceTimeoutTargetCapacityReportV1 {
    WorkspaceTimeoutTargetCapacityReportV1 {
        report_id: "workspace-timeout-target-capacity-v1".to_string(),
        target: "tests/workspace_timeout_root_cause.rs".to_string(),
        current_diagnostic_role: format!(
            "workspace timeout track remains diagnostic-only while no-run={} and full={}",
            summary.no_run_status, summary.full_workspace_status
        ),
        can_receive_assertions: false,
        evidence_blur_risk: "High".to_string(),
        status: "DestinationCapacityInsufficient".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_control_tower_assertion_move_risk_report_v1() -> ControlTowerAssertionMoveRiskReportV1
{
    ControlTowerAssertionMoveRiskReportV1 {
        report_id: "control-tower-assertion-move-risk-v1".to_string(),
        target: "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
        warning_assertion_risk: "High".to_string(),
        ui_panel_evidence_blur_risk: "ControlTowerWarningBlur".to_string(),
        status: "EvidenceBlurRiskTooHigh".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_assertion_destination_capacity_report_v1(
    summary: &Sprint114SummaryFixture,
    shared_fixture: &SharedFixtureHarnessCapacityReportV1,
    workspace_target: &WorkspaceTimeoutTargetCapacityReportV1,
    control_tower: &ControlTowerAssertionMoveRiskReportV1,
) -> AssertionDestinationCapacityReportV1 {
    let rows = vec![
        AssertionDestinationCapacityRowV1 {
            target: shared_fixture.target.clone(),
            current_assertion_load: *summary
                .assertion_count_by_target
                .get(&shared_fixture.target)
                .unwrap_or(&0),
            migrated_assertion_load: summary.existing_migrated_assertion_count,
            warning_assertion_load: 0,
            remaining_capacity: shared_fixture.remaining_capacity,
            status: shared_fixture.status.clone(),
        },
        AssertionDestinationCapacityRowV1 {
            target: workspace_target.target.clone(),
            current_assertion_load: *summary
                .assertion_count_by_target
                .get(&workspace_target.target)
                .unwrap_or(&0),
            migrated_assertion_load: 0,
            warning_assertion_load: 2,
            remaining_capacity: -1,
            status: workspace_target.status.clone(),
        },
        AssertionDestinationCapacityRowV1 {
            target: control_tower.target.clone(),
            current_assertion_load: *summary
                .assertion_count_by_target
                .get(&control_tower.target)
                .unwrap_or(&0),
            migrated_assertion_load: 0,
            warning_assertion_load: 6,
            remaining_capacity: -6,
            status: control_tower.status.clone(),
        },
    ];
    let current_assertion_load = rows.iter().map(|row| row.current_assertion_load).sum();
    let migrated_assertion_load = rows.iter().map(|row| row.migrated_assertion_load).sum();
    let warning_assertion_load = rows.iter().map(|row| row.warning_assertion_load).sum();
    let remaining_capacity = rows.iter().map(|row| row.remaining_capacity).sum();
    AssertionDestinationCapacityReportV1 {
        report_id: "assertion-destination-capacity-v1".to_string(),
        destination_targets: rows,
        current_assertion_load,
        migrated_assertion_load,
        warning_assertion_load,
        remaining_capacity,
        capacity_status: "DestinationCapacityInsufficient".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_evidence_blur_risk_report_v1(
    summary: &Sprint114SummaryFixture,
) -> EvidenceBlurRiskReportV1 {
    let candidate_moves = vec![
        "control_tower_warning_assertions -> tests/shared_fixture_harness_application_v1.rs"
            .to_string(),
        "control_tower_warning_assertions -> tests/workspace_timeout_root_cause.rs".to_string(),
        "workspace_timeout_acceptance_assertions -> tests/shared_fixture_harness_application_v1.rs"
            .to_string(),
    ];
    let blur_risks = vec![
        "MixedFamilyEvidenceBlur".to_string(),
        "ControlTowerWarningBlur".to_string(),
        "TimeoutDiagnosticBlur".to_string(),
        "CliSurfaceBlur".to_string(),
        "DeterminismBlur".to_string(),
        "SafetySignalBlur".to_string(),
    ];
    let mut high_risk_moves = candidate_moves.clone();
    if summary.mixed_family_evidence_narrowed_enough {
        high_risk_moves.clear();
    }
    EvidenceBlurRiskReportV1 {
        report_id: "evidence-blur-risk-v1".to_string(),
        candidate_moves,
        blur_risks,
        high_risk_moves,
        blur_status: if summary.mixed_family_evidence_narrowed_enough {
            "EvidenceBlurControlledWithWarnings"
        } else {
            "EvidenceBlurRiskTooHigh"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_assertion_move_semantic_drift_report_v1() -> AssertionMoveSemanticDriftReportV1 {
    AssertionMoveSemanticDriftReportV1 {
        report_id: "assertion-move-semantic-drift-v1".to_string(),
        semantic_drift_risk: "High".to_string(),
        blockers: vec![
            "control tower warning assertions would blur diagnostic-vs-governance intent"
                .to_string(),
            "acceptance warning wording would drift away from supporting-only evidence".to_string(),
        ],
        status: "SemanticDriftRiskTooHigh".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_assertion_move_determinism_risk_report_v1() -> AssertionMoveDeterminismRiskReportV1 {
    AssertionMoveDeterminismRiskReportV1 {
        report_id: "assertion-move-determinism-risk-v1".to_string(),
        determinism_risk: "High".to_string(),
        blockers: vec![
            "shared fixture harness already runs near its safe assertion budget".to_string(),
            "workspace timeout diagnostics must stay comparable across repeated timeout runs"
                .to_string(),
        ],
        status: "DeterminismRiskTooHigh".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_assertion_move_cli_surface_risk_report_v1() -> AssertionMoveCliSurfaceRiskReportV1 {
    AssertionMoveCliSurfaceRiskReportV1 {
        report_id: "assertion-move-cli-surface-risk-v1".to_string(),
        cli_surface_risk: "High".to_string(),
        blockers: vec![
            "CLI warning posture must remain attached to governance-only outputs".to_string(),
            "control tower panel warnings would be harder to audit after migration".to_string(),
        ],
        status: "CliSurfaceRiskTooHigh".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_assertion_move_safety_risk_report_v1() -> AssertionMoveSafetyRiskReportV1 {
    AssertionMoveSafetyRiskReportV1 {
        report_id: "assertion-move-safety-risk-v1".to_string(),
        safety_risk: "SafetySentinelContinuityPreservedButMovementBlocked".to_string(),
        blockers: vec![
            "no assertion movement is allowed this sprint".to_string(),
            "no target retirement is allowed this sprint".to_string(),
            "safety sentinels must remain visible in place".to_string(),
        ],
        status: "SafetyMovementRiskTooHigh".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_assertion_destination_proof_gate_v1(
    capacity: &AssertionDestinationCapacityReportV1,
    semantic: &AssertionMoveSemanticDriftReportV1,
    determinism: &AssertionMoveDeterminismRiskReportV1,
    cli: &AssertionMoveCliSurfaceRiskReportV1,
    safety: &AssertionMoveSafetyRiskReportV1,
    equivalent_coverage: &EquivalentCoverageContinuityCheckV5,
) -> AssertionDestinationProofGateV1 {
    let proof_complete = capacity.capacity_status == "DestinationCapacityReady"
        && semantic.status == "SemanticDriftRiskControlled"
        && determinism.status == "DeterminismRiskControlled"
        && cli.status == "CliSurfaceRiskControlled"
        && safety.status == "SafetySentinelContinuityPreserved"
        && equivalent_coverage.continuity_preserved;
    AssertionDestinationProofGateV1 {
        gate_id: "assertion-destination-proof-gate-v1".to_string(),
        capacity_status: capacity.capacity_status.clone(),
        semantic_drift_status: semantic.status.clone(),
        determinism_risk_status: determinism.status.clone(),
        cli_surface_risk_status: cli.status.clone(),
        safety_risk_status: if safety.status == "SafetyMovementRiskTooHigh" {
            "SafetySentinelContinuityPreservedButMovementBlocked".to_string()
        } else {
            safety.status.clone()
        },
        equivalent_coverage_status: if equivalent_coverage.continuity_preserved {
            "EquivalentCoverageContinuityPreserved".to_string()
        } else {
            equivalent_coverage.status.clone()
        },
        proof_complete,
        gate_status: if proof_complete {
            "AssertionDestinationProofReady"
        } else {
            "AssertionDestinationProofStillMissing"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_evidence_blur_risk_gate_v1(blur: &EvidenceBlurRiskReportV1) -> EvidenceBlurRiskGateV1 {
    EvidenceBlurRiskGateV1 {
        gate_id: "evidence-blur-risk-gate-v1".to_string(),
        evidence_blur_status: blur.blur_status.clone(),
        gate_status: blur.blur_status.clone(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_fifth_patch_resume_gate_v5(
    proof_gate: &AssertionDestinationProofGateV1,
    blur_gate: &EvidenceBlurRiskGateV1,
    stop_decision: &ConsolidationStopDecisionReportV1,
    safety: &SafetyCoveragePreservationReportV31,
    acceptance: &AcceptanceTruthGateV16,
) -> FifthPatchResumeGateV5 {
    let resume_allowed = proof_gate.proof_complete
        && (blur_gate.gate_status == "EvidenceBlurRiskControlled"
            || blur_gate.gate_status == "EvidenceBlurRiskControlledWithWarnings")
        && !stop_decision.stop_recommended
        && acceptance.can_claim_full_acceptance;
    FifthPatchResumeGateV5 {
        gate_id: "fifth-patch-resume-gate-v5".to_string(),
        assertion_destination_proof_status: proof_gate.gate_status.clone(),
        evidence_blur_gate_status: blur_gate.gate_status.clone(),
        stop_decision_status: stop_decision.stop_status.clone(),
        safety_status: safety.safety_status.clone(),
        acceptance_truth_status: acceptance.truth_status.clone(),
        resume_allowed_for_later_sprint: resume_allowed,
        fifth_patch_applied_this_sprint: false,
        gate_status: if resume_allowed {
            "FifthPatchResumeAllowedForLaterSprint"
        } else {
            "FifthPatchStillBlocked"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_fifth_patch_stop_gate_v1(
    stop_decision: &ConsolidationStopDecisionReportV1,
) -> FifthPatchStopGateV1 {
    FifthPatchStopGateV1 {
        gate_id: "fifth-patch-stop-gate-v1".to_string(),
        stop_recommended: stop_decision.stop_recommended,
        fifth_patch_blocked: true,
        gate_status: if stop_decision.stop_recommended {
            "ConsolidationStopped"
        } else {
            "DiagnosticOnly"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_fifth_patch_no_apply_guarantee_report_v4() -> FifthPatchNoApplyGuaranteeReportV4 {
    FifthPatchNoApplyGuaranteeReportV4 {
        report_id: "fifth-patch-no-apply-guarantee-v4".to_string(),
        fifth_patch_applied: false,
        retired_files: Vec::new(),
        moved_assertions: Vec::new(),
        retired_targets: Vec::new(),
        guarantee_status: "FifthPatchNotAppliedGuaranteed".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_candidate_stop_consolidation_report_v2(
    stop_decision: &ConsolidationStopDecisionReportV1,
    resume_decision: &ConsolidationResumeDecisionReportV1,
) -> CandidateStopConsolidationReportV2 {
    CandidateStopConsolidationReportV2 {
        report_id: "candidate-stop-consolidation-v2".to_string(),
        stop_recommended: stop_decision.stop_recommended,
        stop_reason: stop_decision.stop_reasons.join(", "),
        resume_allowed_with_proof: resume_decision.resume_status
            == "ConsolidationResumeAllowedLater",
        status: "CandidateStopConsolidationReadyWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_consolidation_track_pause_report_v1(
    resume_decision: &ConsolidationResumeDecisionReportV1,
) -> ConsolidationTrackPauseReportV1 {
    ConsolidationTrackPauseReportV1 {
        report_id: "consolidation-track-pause-v1".to_string(),
        paused: true,
        pause_reason: "Stop recommendation carried forward until assertion destination proof and evidence blur risk are controlled.".to_string(),
        resume_prerequisites: resume_decision.required_proofs.clone(),
        status: "ConsolidationPaused".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_track_split_report_v1(
    pause: &ConsolidationTrackPauseReportV1,
) -> WorkspaceTimeoutTrackSplitReportV1 {
    WorkspaceTimeoutTrackSplitReportV1 {
        report_id: "workspace-timeout-track-split-v1".to_string(),
        consolidation_track_status: pause.status.clone(),
        timeout_track_status: "WorkspaceTimeoutDiagnosticTrackActive".to_string(),
        separation_complete: true,
        status: "WorkspaceTimeoutTrackSeparated".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_diagnostic_track_plan_v1() -> WorkspaceTimeoutDiagnosticTrackPlanV1 {
    WorkspaceTimeoutDiagnosticTrackPlanV1 {
        plan_id: "workspace-timeout-diagnostic-track-plan-v1".to_string(),
        no_run_observation_backlog: vec![
            "rerun cargo test --workspace --no-run --quiet under explicit timeout and preserve timeout truth".to_string(),
            "capture target ordering around mixed-family suspects before any consolidation change".to_string(),
        ],
        full_observation_backlog: vec![
            "rerun cargo test --workspace --quiet under explicit timeout and preserve timeout truth".to_string(),
            "keep full acceptance blocked unless the real full run finishes and passes".to_string(),
        ],
        cargo_json_backlog: vec![
            "capture cargo JSON progress without upgrading acceptance claims".to_string(),
            "separate observed events from inferred mixed-family explanations".to_string(),
        ],
        target_level_diagnostics: vec![
            "control tower timeout panel link/macro drilldown".to_string(),
            "workspace timeout root cause target capacity drilldown".to_string(),
            "integration binary fanout trace refresh".to_string(),
        ],
        nextest_sccache_follow_up: vec![
            "keep nextest and sccache diagnostic-only".to_string(),
            "do not treat cache or progress as acceptance".to_string(),
        ],
        status: "WorkspaceTimeoutDiagnosticTrackPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_timeout_observation_backlog_v1() -> WorkspaceTimeoutObservationBacklogV1 {
    WorkspaceTimeoutObservationBacklogV1 {
        report_id: "workspace-timeout-observation-backlog-v1".to_string(),
        queued_observations: vec![
            ObservationBacklogRowV1 {
                item: "Observe no-run timeout cleanup and process exit consistency".to_string(),
                priority: "P1".to_string(),
                rationale: "NoRunStillBlocked remains a core blocker.".to_string(),
            },
            ObservationBacklogRowV1 {
                item: "Observe full workspace timeout boundary and suspect target order"
                    .to_string(),
                priority: "P1".to_string(),
                rationale: "FullWorkspaceStillBlocked must remain honest.".to_string(),
            },
            ObservationBacklogRowV1 {
                item: "Capture cargo JSON artifact ordering for mixed-family suspects".to_string(),
                priority: "P2".to_string(),
                rationale: "Progress evidence stays supporting-only.".to_string(),
            },
        ],
        priority: "Stop consolidation, continue diagnostics".to_string(),
        status: "WorkspaceTimeoutObservationBacklogReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_no_run_observation_plan_v2(
    summary: &Sprint114SummaryFixture,
) -> WorkspaceNoRunObservationPlanV2 {
    WorkspaceNoRunObservationPlanV2 {
        plan_id: "workspace-no-run-observation-plan-v2".to_string(),
        next_no_run_strategy: vec![
            "keep no-run observation diagnostic-only".to_string(),
            "record timeout seconds and suspect target evidence separately".to_string(),
            "do not use no-run recovery as full acceptance".to_string(),
        ],
        timeout_seconds: summary.no_run_timeout_seconds.unwrap_or(420),
        observation_quality: "SupportingOnly".to_string(),
        status: "WorkspaceNoRunObservationPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_full_observation_plan_v2(
    summary: &Sprint114SummaryFixture,
) -> WorkspaceFullObservationPlanV2 {
    WorkspaceFullObservationPlanV2 {
        plan_id: "workspace-full-observation-plan-v2".to_string(),
        next_full_strategy: vec![
            "rerun the real full workspace test only under explicit timeout".to_string(),
            "preserve full blocked status unless the full run genuinely finishes and passes"
                .to_string(),
            "keep focused/CLI/build evidence supporting-only".to_string(),
        ],
        timeout_seconds: summary.full_timeout_seconds.unwrap_or(420),
        status: "WorkspaceFullObservationPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cargo_json_observation_plan_v2() -> CargoJsonObservationPlanV2 {
    CargoJsonObservationPlanV2 {
        plan_id: "cargo-json-observation-plan-v2".to_string(),
        cargo_json_capture_strategy: vec![
            "capture artifact order for suspect targets".to_string(),
            "preserve observed-vs-inferred separation".to_string(),
            "do not upgrade cargo progress into acceptance truth".to_string(),
        ],
        status: "CargoJsonObservationPlanReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_target_family_diagnostic_backlog_v1(
    summary: &Sprint114SummaryFixture,
) -> TargetFamilyDiagnosticBacklogV1 {
    TargetFamilyDiagnosticBacklogV1 {
        report_id: "target-family-diagnostic-backlog-v1".to_string(),
        mixed_families: summary.still_mixed_families.clone(),
        backlog_status: "TargetFamilyDiagnosticBacklogReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_link_macro_diagnostic_backlog_v1(
    summary: &Sprint114SummaryFixture,
) -> LinkMacroDiagnosticBacklogV1 {
    LinkMacroDiagnosticBacklogV1 {
        report_id: "link-macro-diagnostic-backlog-v1".to_string(),
        link_macro_diagnostics: stable_strings(
            summary
                .link_observed_evidence
                .iter()
                .chain(summary.link_inferred_evidence.iter())
                .chain(summary.macro_observed_evidence.iter())
                .chain(summary.macro_inferred_evidence.iter())
                .cloned()
                .collect(),
        ),
        status: "LinkMacroDiagnosticBacklogReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_integration_fanout_diagnostic_backlog_v1(
    summary: &Sprint114SummaryFixture,
) -> IntegrationFanoutDiagnosticBacklogV1 {
    IntegrationFanoutDiagnosticBacklogV1 {
        report_id: "integration-fanout-diagnostic-backlog-v1".to_string(),
        integration_fanout_diagnostics: stable_strings(
            summary
                .integration_observed_evidence
                .iter()
                .chain(summary.integration_inferred_evidence.iter())
                .cloned()
                .collect(),
        ),
        status: "IntegrationFanoutDiagnosticBacklogReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cumulative_safe_patch_ledger_v6() -> CumulativeSafePatchLedgerV6 {
    CumulativeSafePatchLedgerV6 {
        report_id: "cumulative-safe-patch-ledger-v6".to_string(),
        carried_patch_ids: vec![
            "sprint107-first-safe-consolidation-patch".to_string(),
            "sprint108-second-safe-consolidation-patch".to_string(),
            "sprint109-third-safe-consolidation-patch".to_string(),
            "sprint110-fourth-safe-consolidation-patch".to_string(),
        ],
        fifth_patch_applied: false,
        status: "CumulativeSafePatchLedgerReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_cumulative_binary_delta_report_v5(
    summary: &Sprint114SummaryFixture,
) -> CumulativeBinaryDeltaReportV5 {
    CumulativeBinaryDeltaReportV5 {
        report_id: "cumulative-binary-delta-v5".to_string(),
        sample_backed_delta: summary.cumulative_sample_backed_delta,
        measured_claim_allowed: false,
        status: "CumulativeBinaryDeltaSampleBackedOnly".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_assertion_ledger_continuity_check_v5() -> AssertionLedgerContinuityCheckV5 {
    AssertionLedgerContinuityCheckV5 {
        report_id: "assertion-ledger-continuity-check-v5".to_string(),
        continuity_preserved: true,
        status: "AssertionLedgerContinuityPreserved".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_equivalent_coverage_continuity_check_v5() -> EquivalentCoverageContinuityCheckV5 {
    EquivalentCoverageContinuityCheckV5 {
        report_id: "equivalent-coverage-continuity-check-v5".to_string(),
        continuity_preserved: true,
        status: "EquivalentCoverageContinuityPreserved".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_safety_sentinel_continuity_check_v5() -> SafetySentinelContinuityCheckV5 {
    SafetySentinelContinuityCheckV5 {
        report_id: "safety-sentinel-continuity-check-v5".to_string(),
        continuity_preserved: true,
        status: "SafetySentinelContinuityPreserved".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_no_hidden_skip_continuity_check_v5() -> NoHiddenSkipContinuityCheckV5 {
    NoHiddenSkipContinuityCheckV5 {
        report_id: "no-hidden-skip-continuity-check-v5".to_string(),
        continuity_preserved: true,
        status: "NoHiddenSkipContinuityPreserved".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_timeout_cleanup_verification_report_v8(
    summary: &Sprint114SummaryFixture,
) -> TimeoutCleanupVerificationReportV8 {
    TimeoutCleanupVerificationReportV8 {
        report_id: "timeout-cleanup-verification-v8".to_string(),
        cleanup_verified: summary.timeout_cleanup_verified,
        remaining_cargo_processes: summary.remaining_cargo_processes_after_timeout,
        remaining_rustc_processes: summary.remaining_rustc_processes_after_timeout,
        cleanup_status: if summary.timeout_cleanup_verified {
            "TimeoutCleanupVerifiedButNotPass"
        } else {
            "TimeoutCleanupUnverified"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_no_run_recovery_gate_v16(
    summary: &Sprint114SummaryFixture,
) -> WorkspaceNoRunRecoveryGateV16 {
    let finished = summary.no_run_exit_code == Some(0);
    let timed_out = summary.no_run_exit_code == Some(124)
        || summary
            .no_run_status
            .to_ascii_lowercase()
            .contains("timeout");
    WorkspaceNoRunRecoveryGateV16 {
        gate_id: "workspace-no-run-recovery-gate-v16".to_string(),
        command: "cargo test --workspace --no-run --quiet".to_string(),
        finished,
        passed: finished,
        timed_out,
        recovered: finished,
        gate_status: if finished {
            "NoRunRecovered"
        } else {
            "NoRunStillBlocked"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_full_acceptance_gate_v16(
    summary: &Sprint114SummaryFixture,
) -> WorkspaceFullAcceptanceGateV16 {
    let finished = summary.full_exit_code == Some(0);
    WorkspaceFullAcceptanceGateV16 {
        gate_id: "workspace-full-acceptance-gate-v16".to_string(),
        command: "cargo test --workspace --quiet".to_string(),
        finished,
        passed: finished,
        accepted: finished,
        gate_status: if finished {
            "FullWorkspaceAccepted"
        } else {
            "FullWorkspaceStillBlocked"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_focused_vs_full_bridge_v12(
    baseline: &Sprint114BaselineTruthImportReport,
    no_run: &WorkspaceNoRunRecoveryGateV16,
    full: &WorkspaceFullAcceptanceGateV16,
) -> FocusedVsFullBridgeV12 {
    FocusedVsFullBridgeV12 {
        bridge_id: "focused-vs-full-bridge-v12".to_string(),
        focused_tests_status: if baseline.focused_tests_passed {
            "FocusedTestsSupportingOnly"
        } else {
            "FocusedTestsMissing"
        }
        .to_string(),
        cli_smoke_status: if baseline.cli_smoke_passed {
            "CliSmokeSupportingOnly"
        } else {
            "CliSmokeMissing"
        }
        .to_string(),
        cargo_build_status: if baseline.cargo_build_passed {
            "CargoBuildSupportingOnly"
        } else {
            "CargoBuildMissing"
        }
        .to_string(),
        no_run_status: no_run.gate_status.clone(),
        full_status: full.gate_status.clone(),
        bridge_status: "FocusedEvidenceCannotClaimFullAcceptance".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_acceptance_truth_gate_v16(
    baseline: &Sprint114BaselineTruthImportReport,
    no_run: &WorkspaceNoRunRecoveryGateV16,
    full: &WorkspaceFullAcceptanceGateV16,
) -> AcceptanceTruthGateV16 {
    AcceptanceTruthGateV16 {
        gate_id: "acceptance-truth-gate-v16".to_string(),
        focused_truth_status: if baseline.focused_tests_passed {
            "FocusedTestsSupportingOnly"
        } else {
            "FocusedTestsMissing"
        }
        .to_string(),
        cli_truth_status: if baseline.cli_smoke_passed {
            "CliSmokeSupportingOnly"
        } else {
            "CliSmokeMissing"
        }
        .to_string(),
        cargo_check_truth_status: if baseline.cargo_check_passed {
            "CargoCheckSupportingOnly"
        } else {
            "CargoCheckMissing"
        }
        .to_string(),
        cargo_build_truth_status: if baseline.cargo_build_passed {
            "CargoBuildSupportingOnly"
        } else {
            "CargoBuildMissing"
        }
        .to_string(),
        no_run_truth_status: no_run.gate_status.clone(),
        full_workspace_truth_status: full.gate_status.clone(),
        can_claim_full_acceptance: full.accepted,
        truth_status: if full.accepted {
            "FullWorkspaceAccepted".to_string()
        } else {
            baseline.acceptance_truth_status.clone()
        },
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_acceptance_evidence_strength_report_v5(
    acceptance: &AcceptanceTruthGateV16,
) -> AcceptanceEvidenceStrengthReportV5 {
    AcceptanceEvidenceStrengthReportV5 {
        report_id: "acceptance-evidence-strength-v5".to_string(),
        evidence_tiers: vec![
            "focused-tests-supporting-only".to_string(),
            "cli-smoke-supporting-only".to_string(),
            "cargo-check-supporting-only".to_string(),
            "cargo-build-supporting-only".to_string(),
            "workspace-full-pass-required-for-acceptance".to_string(),
        ],
        strongest_claim: if acceptance.can_claim_full_acceptance {
            "FullWorkspaceAccepted".to_string()
        } else {
            "AcceptanceTruthReadyWithWarnings".to_string()
        },
        report_status: "AcceptanceEvidenceSupportingOnly".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_workspace_recovery_decision_report_v5(
    no_run: &WorkspaceNoRunRecoveryGateV16,
    full: &WorkspaceFullAcceptanceGateV16,
) -> WorkspaceRecoveryDecisionReportV5 {
    WorkspaceRecoveryDecisionReportV5 {
        report_id: "workspace-recovery-decision-v5".to_string(),
        recommend_stop_consolidation: true,
        recommend_timeout_diagnostic_track: true,
        recommend_resume_only_with_proof: true,
        no_run_recovered: no_run.recovered,
        full_workspace_accepted: full.accepted,
        decision_status: "WorkspaceRecoveryDecisionReadyWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_safety_coverage_preservation_report_v31() -> SafetyCoveragePreservationReportV31 {
    SafetyCoveragePreservationReportV31 {
        report_id: "safety-coverage-preservation-v31".to_string(),
        no_assertion_deletion: true,
        no_safety_sentinel_deletion: true,
        no_hidden_skips: true,
        consolidation_stop_resume_guard_present: true,
        assertion_destination_proof_guard_present: true,
        evidence_blur_risk_guard_present: true,
        track_split_guard_present: true,
        fifth_patch_v5_no_apply_guard_present: true,
        no_target_retirement_guard_present: true,
        no_assertion_movement_guard_present: true,
        runtime_deferred: true,
        training_deferred: true,
        live_trading_forbidden: true,
        safety_status: "SafetyCoveragePreserved".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_control_tower_consolidation_governance_panel(
    stop: &ConsolidationStopDecisionReportV1,
    resume: &ConsolidationResumeDecisionReportV1,
    proof: &AssertionDestinationProofGateV1,
    blur: &EvidenceBlurRiskGateV1,
    no_apply: &FifthPatchNoApplyGuaranteeReportV4,
) -> ControlTowerConsolidationGovernancePanel {
    ControlTowerConsolidationGovernancePanel {
        panel_id: "control-tower-consolidation-governance".to_string(),
        stop_decision_status: stop.stop_status.clone(),
        resume_decision_status: resume.resume_status.clone(),
        proof_gate_status: proof.gate_status.clone(),
        evidence_blur_status: blur.gate_status.clone(),
        fifth_patch_status: no_apply.guarantee_status.clone(),
        warnings: warning_posture(),
        static_read_only: true,
        no_apply_button: true,
        no_run_button: true,
        no_train_runtime_live_order_account_controls: true,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

pub fn build_control_tower_workspace_timeout_track_panel(
    split: &WorkspaceTimeoutTrackSplitReportV1,
    backlog: &WorkspaceTimeoutObservationBacklogV1,
    no_run: &WorkspaceNoRunRecoveryGateV16,
    full: &WorkspaceFullAcceptanceGateV16,
    acceptance: &AcceptanceTruthGateV16,
) -> ControlTowerWorkspaceTimeoutTrackPanel {
    ControlTowerWorkspaceTimeoutTrackPanel {
        panel_id: "control-tower-workspace-timeout-track".to_string(),
        timeout_track_status: split.timeout_track_status.clone(),
        backlog_status: backlog.status.clone(),
        no_run_status: no_run.gate_status.clone(),
        full_status: full.gate_status.clone(),
        acceptance_truth_status: acceptance.truth_status.clone(),
        warnings: warning_posture(),
        static_read_only: true,
        no_run_button: true,
        no_train_runtime_live_order_account_controls: true,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsolidationStopResumeGovernanceBundle {
    pub sprint114_baseline_truth_import_report: Sprint114BaselineTruthImportReport,
    pub sprint114_stop_recommendation_carry_forward_report:
        Sprint114StopRecommendationCarryForwardReport,
    pub consolidation_stop_decision_report_v1: ConsolidationStopDecisionReportV1,
    pub consolidation_resume_decision_report_v1: ConsolidationResumeDecisionReportV1,
    pub consolidation_decision_matrix_v1: ConsolidationDecisionMatrixV1,
    pub assertion_destination_proof_plan_v1: AssertionDestinationProofPlanV1,
    pub assertion_destination_capacity_report_v1: AssertionDestinationCapacityReportV1,
    pub shared_fixture_harness_capacity_report_v1: SharedFixtureHarnessCapacityReportV1,
    pub workspace_timeout_target_capacity_report_v1: WorkspaceTimeoutTargetCapacityReportV1,
    pub control_tower_assertion_move_risk_report_v1: ControlTowerAssertionMoveRiskReportV1,
    pub evidence_blur_risk_report_v1: EvidenceBlurRiskReportV1,
    pub assertion_move_semantic_drift_report_v1: AssertionMoveSemanticDriftReportV1,
    pub assertion_move_determinism_risk_report_v1: AssertionMoveDeterminismRiskReportV1,
    pub assertion_move_cli_surface_risk_report_v1: AssertionMoveCliSurfaceRiskReportV1,
    pub assertion_move_safety_risk_report_v1: AssertionMoveSafetyRiskReportV1,
    pub assertion_destination_proof_gate_v1: AssertionDestinationProofGateV1,
    pub evidence_blur_risk_gate_v1: EvidenceBlurRiskGateV1,
    pub fifth_patch_resume_gate_v5: FifthPatchResumeGateV5,
    pub fifth_patch_stop_gate_v1: FifthPatchStopGateV1,
    pub fifth_patch_no_apply_guarantee_report_v4: FifthPatchNoApplyGuaranteeReportV4,
    pub candidate_stop_consolidation_report_v2: CandidateStopConsolidationReportV2,
    pub consolidation_track_pause_report_v1: ConsolidationTrackPauseReportV1,
    pub workspace_timeout_track_split_report_v1: WorkspaceTimeoutTrackSplitReportV1,
    pub workspace_timeout_diagnostic_track_plan_v1: WorkspaceTimeoutDiagnosticTrackPlanV1,
    pub workspace_timeout_observation_backlog_v1: WorkspaceTimeoutObservationBacklogV1,
    pub workspace_no_run_observation_plan_v2: WorkspaceNoRunObservationPlanV2,
    pub workspace_full_observation_plan_v2: WorkspaceFullObservationPlanV2,
    pub cargo_json_observation_plan_v2: CargoJsonObservationPlanV2,
    pub target_family_diagnostic_backlog_v1: TargetFamilyDiagnosticBacklogV1,
    pub link_macro_diagnostic_backlog_v1: LinkMacroDiagnosticBacklogV1,
    pub integration_fanout_diagnostic_backlog_v1: IntegrationFanoutDiagnosticBacklogV1,
    pub cumulative_safe_patch_ledger_v6: CumulativeSafePatchLedgerV6,
    pub cumulative_binary_delta_report_v5: CumulativeBinaryDeltaReportV5,
    pub assertion_ledger_continuity_check_v5: AssertionLedgerContinuityCheckV5,
    pub equivalent_coverage_continuity_check_v5: EquivalentCoverageContinuityCheckV5,
    pub safety_sentinel_continuity_check_v5: SafetySentinelContinuityCheckV5,
    pub no_hidden_skip_continuity_check_v5: NoHiddenSkipContinuityCheckV5,
    pub timeout_cleanup_verification_report_v8: TimeoutCleanupVerificationReportV8,
    pub workspace_no_run_recovery_gate_v16: WorkspaceNoRunRecoveryGateV16,
    pub workspace_full_acceptance_gate_v16: WorkspaceFullAcceptanceGateV16,
    pub focused_vs_full_bridge_v12: FocusedVsFullBridgeV12,
    pub acceptance_truth_gate_v16: AcceptanceTruthGateV16,
    pub acceptance_evidence_strength_report_v5: AcceptanceEvidenceStrengthReportV5,
    pub workspace_recovery_decision_report_v5: WorkspaceRecoveryDecisionReportV5,
    pub safety_coverage_preservation_report_v31: SafetyCoveragePreservationReportV31,
    pub control_tower_consolidation_governance_panel: ControlTowerConsolidationGovernancePanel,
    pub control_tower_workspace_timeout_track_panel: ControlTowerWorkspaceTimeoutTrackPanel,
    pub storage_report: ConsolidationStopResumeGovernanceStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl ConsolidationStopResumeGovernanceBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }

    pub fn build_final_summary(&self) -> String {
        let sections = vec![
            (
                "## 1. Sprint summary",
                format!(
                    "stop={} resume={} proof={} blur={} fifth_patch={} acceptance={}",
                    self.consolidation_stop_decision_report_v1.stop_status,
                    self.consolidation_resume_decision_report_v1.resume_status,
                    self.assertion_destination_proof_gate_v1.gate_status,
                    self.evidence_blur_risk_gate_v1.gate_status,
                    self.fifth_patch_resume_gate_v5.gate_status,
                    self.acceptance_truth_gate_v16.truth_status,
                ),
            ),
            (
                "## 2. Why Sprint 115 was needed",
                "Sprint 114 ended with StopConsolidationRecommended, AssertionMigrationBlocked, and FifthPatchStillBlocked, so Sprint 115 formalizes stop/resume governance instead of applying another patch.".to_string(),
            ),
            (
                "## 3. Files added",
                "Governance-only local reports and fixtures were added; no runtime, training, live, broker, order, account, assertion movement, or target retirement surface was added.".to_string(),
            ),
            (
                "## 4. Files changed",
                "Changes are limited to Sprint 115 governance/report/CLI/test/docs surfaces and preserve prior Sprint 114 truth as imported evidence.".to_string(),
            ),
            (
                "## 5. Sprint 114 baseline truth import",
                format!(
                    "import_status={} imported_as_full_acceptance={}",
                    self.sprint114_baseline_truth_import_report.import_status,
                    self.sprint114_baseline_truth_import_report
                        .imported_as_full_acceptance,
                ),
            ),
            (
                "## 6. Stop recommendation carry-forward",
                format!(
                    "carry_forward_status={} stop_recommendation_present={}",
                    self.sprint114_stop_recommendation_carry_forward_report
                        .carry_forward_status,
                    self.sprint114_stop_recommendation_carry_forward_report
                        .stop_recommendation_present,
                ),
            ),
            (
                "## 7. Consolidation stop decision",
                format!(
                    "stop_status={} stop_reasons={}",
                    self.consolidation_stop_decision_report_v1.stop_status,
                    self.consolidation_stop_decision_report_v1
                        .stop_reasons
                        .join(","),
                ),
            ),
            (
                "## 8. Consolidation resume decision",
                format!(
                    "resume_status={} missing_proofs={}",
                    self.consolidation_resume_decision_report_v1.resume_status,
                    self.consolidation_resume_decision_report_v1
                        .missing_proofs
                        .join(","),
                ),
            ),
            (
                "## 9. Consolidation decision matrix",
                format!(
                    "selected_decision={} matrix_status={}",
                    self.consolidation_decision_matrix_v1.selected_decision,
                    self.consolidation_decision_matrix_v1.matrix_status,
                ),
            ),
            (
                "## 10. Assertion destination proof plan",
                format!(
                    "plan_status={} destination_candidates={}",
                    self.assertion_destination_proof_plan_v1.plan_status,
                    self.assertion_destination_proof_plan_v1
                        .destination_candidates
                        .len(),
                ),
            ),
            (
                "## 11. Assertion destination capacity",
                format!(
                    "capacity_status={} remaining_capacity={}",
                    self.assertion_destination_capacity_report_v1.capacity_status,
                    self.assertion_destination_capacity_report_v1
                        .remaining_capacity,
                ),
            ),
            (
                "## 12. Shared fixture harness capacity",
                format!(
                    "status={} remaining_capacity={}",
                    self.shared_fixture_harness_capacity_report_v1.status,
                    self.shared_fixture_harness_capacity_report_v1
                        .remaining_capacity,
                ),
            ),
            (
                "## 13. Workspace timeout target capacity",
                format!(
                    "status={} can_receive_assertions={}",
                    self.workspace_timeout_target_capacity_report_v1.status,
                    self.workspace_timeout_target_capacity_report_v1
                        .can_receive_assertions,
                ),
            ),
            (
                "## 14. Control Tower assertion move risk",
                format!(
                    "status={}",
                    self.control_tower_assertion_move_risk_report_v1.status,
                ),
            ),
            (
                "## 15. Evidence blur risk",
                format!(
                    "blur_status={} high_risk_moves={}",
                    self.evidence_blur_risk_report_v1.blur_status,
                    self.evidence_blur_risk_report_v1.high_risk_moves.len(),
                ),
            ),
            (
                "## 16. Assertion move semantic / determinism / CLI / safety risk",
                format!(
                    "semantic={} determinism={} cli={} safety={}",
                    self.assertion_move_semantic_drift_report_v1.status,
                    self.assertion_move_determinism_risk_report_v1.status,
                    self.assertion_move_cli_surface_risk_report_v1.status,
                    self.assertion_move_safety_risk_report_v1.status,
                ),
            ),
            (
                "## 17. Assertion destination proof gate",
                format!(
                    "gate_status={} proof_complete={}",
                    self.assertion_destination_proof_gate_v1.gate_status,
                    self.assertion_destination_proof_gate_v1.proof_complete,
                ),
            ),
            (
                "## 18. Evidence blur risk gate",
                format!(
                    "gate_status={}",
                    self.evidence_blur_risk_gate_v1.gate_status,
                ),
            ),
            (
                "## 19. Fifth patch resume gate v5",
                format!(
                    "gate_status={} resume_allowed_for_later_sprint={} fifth_patch_applied_this_sprint={}",
                    self.fifth_patch_resume_gate_v5.gate_status,
                    self.fifth_patch_resume_gate_v5
                        .resume_allowed_for_later_sprint,
                    self.fifth_patch_resume_gate_v5
                        .fifth_patch_applied_this_sprint,
                ),
            ),
            (
                "## 20. Fifth patch stop gate",
                format!(
                    "gate_status={} stop_recommended={}",
                    self.fifth_patch_stop_gate_v1.gate_status,
                    self.fifth_patch_stop_gate_v1.stop_recommended,
                ),
            ),
            (
                "## 21. Fifth patch no-apply guarantee v4",
                format!(
                    "guarantee_status={} moved_assertions={} retired_targets={}",
                    self.fifth_patch_no_apply_guarantee_report_v4
                        .guarantee_status,
                    self.fifth_patch_no_apply_guarantee_report_v4
                        .moved_assertions
                        .len(),
                    self.fifth_patch_no_apply_guarantee_report_v4
                        .retired_targets
                        .len(),
                ),
            ),
            (
                "## 22. Candidate stop consolidation report v2",
                format!(
                    "status={} resume_allowed_with_proof={}",
                    self.candidate_stop_consolidation_report_v2.status,
                    self.candidate_stop_consolidation_report_v2
                        .resume_allowed_with_proof,
                ),
            ),
            (
                "## 23. Consolidation track pause",
                format!(
                    "status={} paused={}",
                    self.consolidation_track_pause_report_v1.status,
                    self.consolidation_track_pause_report_v1.paused,
                ),
            ),
            (
                "## 24. Workspace timeout track split",
                format!(
                    "status={} separation_complete={}",
                    self.workspace_timeout_track_split_report_v1.status,
                    self.workspace_timeout_track_split_report_v1
                        .separation_complete,
                ),
            ),
            (
                "## 25. Workspace timeout diagnostic track plan",
                format!(
                    "status={}",
                    self.workspace_timeout_diagnostic_track_plan_v1.status,
                ),
            ),
            (
                "## 26. Workspace timeout observation backlog",
                format!(
                    "status={} queued_observations={}",
                    self.workspace_timeout_observation_backlog_v1.status,
                    self.workspace_timeout_observation_backlog_v1
                        .queued_observations
                        .len(),
                ),
            ),
            (
                "## 27. No-run / full / cargo JSON observation plans",
                format!(
                    "no_run={} full={} cargo_json={}",
                    self.workspace_no_run_observation_plan_v2.status,
                    self.workspace_full_observation_plan_v2.status,
                    self.cargo_json_observation_plan_v2.status,
                ),
            ),
            (
                "## 28. Target family diagnostic backlog",
                format!(
                    "backlog_status={} mixed_families={}",
                    self.target_family_diagnostic_backlog_v1.backlog_status,
                    self.target_family_diagnostic_backlog_v1
                        .mixed_families
                        .join(","),
                ),
            ),
            (
                "## 29. Link/macro diagnostic backlog",
                format!(
                    "status={}",
                    self.link_macro_diagnostic_backlog_v1.status,
                ),
            ),
            (
                "## 30. Integration fanout diagnostic backlog",
                format!(
                    "status={}",
                    self.integration_fanout_diagnostic_backlog_v1.status,
                ),
            ),
            (
                "## 31. Cumulative safe patch ledger v6",
                format!(
                    "status={} carried_patch_ids={} fifth_patch_applied={}",
                    self.cumulative_safe_patch_ledger_v6.status,
                    self.cumulative_safe_patch_ledger_v6.carried_patch_ids.len(),
                    self.cumulative_safe_patch_ledger_v6.fifth_patch_applied,
                ),
            ),
            (
                "## 32. Cumulative binary delta v5",
                format!(
                    "status={} sample_backed_delta={} measured_claim_allowed={}",
                    self.cumulative_binary_delta_report_v5.status,
                    self.cumulative_binary_delta_report_v5.sample_backed_delta,
                    self.cumulative_binary_delta_report_v5
                        .measured_claim_allowed,
                ),
            ),
            (
                "## 33. Continuity checks v5",
                format!(
                    "assertion={} coverage={} safety={} hidden_skip={}",
                    self.assertion_ledger_continuity_check_v5.status,
                    self.equivalent_coverage_continuity_check_v5.status,
                    self.safety_sentinel_continuity_check_v5.status,
                    self.no_hidden_skip_continuity_check_v5.status,
                ),
            ),
            (
                "## 34. Timeout cleanup verification v8",
                format!(
                    "cleanup_status={} remaining_cargo_processes={} remaining_rustc_processes={}",
                    self.timeout_cleanup_verification_report_v8.cleanup_status,
                    self.timeout_cleanup_verification_report_v8
                        .remaining_cargo_processes,
                    self.timeout_cleanup_verification_report_v8
                        .remaining_rustc_processes,
                ),
            ),
            (
                "## 35. Workspace no-run recovery gate v16",
                format!(
                    "gate_status={} recovered={} timed_out={}",
                    self.workspace_no_run_recovery_gate_v16.gate_status,
                    self.workspace_no_run_recovery_gate_v16.recovered,
                    self.workspace_no_run_recovery_gate_v16.timed_out,
                ),
            ),
            (
                "## 36. Workspace full acceptance gate v16",
                format!(
                    "gate_status={} accepted={}",
                    self.workspace_full_acceptance_gate_v16.gate_status,
                    self.workspace_full_acceptance_gate_v16.accepted,
                ),
            ),
            (
                "## 37. Focused-vs-full bridge v12",
                format!(
                    "bridge_status={}",
                    self.focused_vs_full_bridge_v12.bridge_status,
                ),
            ),
            (
                "## 38. Acceptance truth gate v16",
                format!(
                    "truth_status={} can_claim_full_acceptance={}",
                    self.acceptance_truth_gate_v16.truth_status,
                    self.acceptance_truth_gate_v16
                        .can_claim_full_acceptance,
                ),
            ),
            (
                "## 39. Acceptance evidence strength v5",
                format!(
                    "report_status={} strongest_claim={}",
                    self.acceptance_evidence_strength_report_v5.report_status,
                    self.acceptance_evidence_strength_report_v5
                        .strongest_claim,
                ),
            ),
            (
                "## 40. Workspace recovery decision v5",
                format!(
                    "decision_status={} stop={} timeout_track={} resume_only_with_proof={}",
                    self.workspace_recovery_decision_report_v5.decision_status,
                    self.workspace_recovery_decision_report_v5
                        .recommend_stop_consolidation,
                    self.workspace_recovery_decision_report_v5
                        .recommend_timeout_diagnostic_track,
                    self.workspace_recovery_decision_report_v5
                        .recommend_resume_only_with_proof,
                ),
            ),
            (
                "## 41. Safety coverage preservation v31",
                format!(
                    "safety_status={}",
                    self.safety_coverage_preservation_report_v31.safety_status,
                ),
            ),
            (
                "## 42. Control Tower consolidation governance panel",
                format!(
                    "static_read_only={} no_apply_button={} no_run_button={}",
                    self.control_tower_consolidation_governance_panel
                        .static_read_only,
                    self.control_tower_consolidation_governance_panel
                        .no_apply_button,
                    self.control_tower_consolidation_governance_panel
                        .no_run_button,
                ),
            ),
            (
                "## 43. Control Tower workspace timeout track panel",
                format!(
                    "static_read_only={} no_run_button={} timeout_track_status={}",
                    self.control_tower_workspace_timeout_track_panel
                        .static_read_only,
                    self.control_tower_workspace_timeout_track_panel.no_run_button,
                    self.control_tower_workspace_timeout_track_panel
                        .timeout_track_status,
                ),
            ),
            (
                "## 44. Output bundle",
                format!("file_count={}", self.storage_report.file_count),
            ),
            (
                "## 45. CLI and examples",
                "All Sprint 115 CLI surfaces are local-output/report-only and preserve no-apply/no-runtime safety warnings.".to_string(),
            ),
            (
                "## 46. Tests added",
                "Focused tests cover governance bundle, Sprint 114 import, stop decision, proof plan, blur gate, fifth patch gate, track split, acceptance truth, panels, CLI safety, and determinism.".to_string(),
            ),
            (
                "## 47. Test results",
                "Generated summary records implementation evidence only; command execution results must be reported by the verifier after running the tests.".to_string(),
            ),
            (
                "## 48. Consolidation governance status",
                format!(
                    "selected_decision={} stop_status={}",
                    self.consolidation_decision_matrix_v1.selected_decision,
                    self.consolidation_stop_decision_report_v1.stop_status,
                ),
            ),
            (
                "## 49. Assertion destination proof status",
                format!(
                    "gate_status={}",
                    self.assertion_destination_proof_gate_v1.gate_status,
                ),
            ),
            (
                "## 50. Evidence blur risk status",
                format!(
                    "gate_status={}",
                    self.evidence_blur_risk_gate_v1.gate_status,
                ),
            ),
            (
                "## 51. Fifth patch status",
                format!(
                    "resume_gate={} no_apply={}",
                    self.fifth_patch_resume_gate_v5.gate_status,
                    self.fifth_patch_no_apply_guarantee_report_v4
                        .guarantee_status,
                ),
            ),
            (
                "## 52. Workspace timeout track status",
                format!(
                    "status={}",
                    self.workspace_timeout_track_split_report_v1.status,
                ),
            ),
            (
                "## 53. No-run recovery status",
                format!(
                    "gate_status={}",
                    self.workspace_no_run_recovery_gate_v16.gate_status,
                ),
            ),
            (
                "## 54. Full workspace acceptance status",
                format!(
                    "gate_status={}",
                    self.workspace_full_acceptance_gate_v16.gate_status,
                ),
            ),
            (
                "## 55. Runtime deferred status",
                "Runtime, training, live inference, live trading, broker, order, account, runtime LLM, Mamba, and Gated runtime remain deferred or forbidden.".to_string(),
            ),
            (
                "## 56. Workspace acceptance truth status",
                format!(
                    "truth_status={}",
                    self.acceptance_truth_gate_v16.truth_status,
                ),
            ),
            (
                "## 57. Safety coverage status",
                format!(
                    "safety_status={}",
                    self.safety_coverage_preservation_report_v31.safety_status,
                ),
            ),
            (
                "## 58. Risk review",
                "No fifth patch was applied, no assertions were moved, no targets were retired, no skip was hidden, and no full-workspace acceptance was claimed.".to_string(),
            ),
            (
                "## 59. Deferred items",
                "Runtime/training/live/order/account/dashboard/browser/live-agent activation and broad consolidation remain out of scope.".to_string(),
            ),
            (
                "## 60. Next gstack sprint recommendation",
                "Keep consolidation stopped or paused until assertion destination proof and evidence blur gates pass; continue workspace timeout diagnostics without acceptance overclaim.".to_string(),
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
    ) -> Result<ConsolidationStopResumeGovernanceStorageReport, String> {
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
            "sprint114_baseline_truth_import.txt",
            self.sprint114_baseline_truth_import_report
        );
        write_report!(
            "sprint114_stop_recommendation_carry_forward.txt",
            self.sprint114_stop_recommendation_carry_forward_report
        );
        write_report!(
            "consolidation_stop_decision_v1.txt",
            self.consolidation_stop_decision_report_v1
        );
        write_report!(
            "consolidation_resume_decision_v1.txt",
            self.consolidation_resume_decision_report_v1
        );
        write_report!(
            "consolidation_decision_matrix_v1.txt",
            self.consolidation_decision_matrix_v1
        );
        write_report!(
            "assertion_destination_proof_plan_v1.txt",
            self.assertion_destination_proof_plan_v1
        );
        write_report!(
            "assertion_destination_capacity_v1.txt",
            self.assertion_destination_capacity_report_v1
        );
        write_report!(
            "shared_fixture_harness_capacity_v1.txt",
            self.shared_fixture_harness_capacity_report_v1
        );
        write_report!(
            "workspace_timeout_target_capacity_v1.txt",
            self.workspace_timeout_target_capacity_report_v1
        );
        write_report!(
            "control_tower_assertion_move_risk_v1.txt",
            self.control_tower_assertion_move_risk_report_v1
        );
        write_report!(
            "evidence_blur_risk_v1.txt",
            self.evidence_blur_risk_report_v1
        );
        write_report!(
            "assertion_move_semantic_drift_v1.txt",
            self.assertion_move_semantic_drift_report_v1
        );
        write_report!(
            "assertion_move_determinism_risk_v1.txt",
            self.assertion_move_determinism_risk_report_v1
        );
        write_report!(
            "assertion_move_cli_surface_risk_v1.txt",
            self.assertion_move_cli_surface_risk_report_v1
        );
        write_report!(
            "assertion_move_safety_risk_v1.txt",
            self.assertion_move_safety_risk_report_v1
        );
        write_report!(
            "assertion_destination_proof_gate_v1.txt",
            self.assertion_destination_proof_gate_v1
        );
        write_report!(
            "evidence_blur_risk_gate_v1.txt",
            self.evidence_blur_risk_gate_v1
        );
        write_report!(
            "fifth_patch_resume_gate_v5.txt",
            self.fifth_patch_resume_gate_v5
        );
        write_report!(
            "fifth_patch_stop_gate_v1.txt",
            self.fifth_patch_stop_gate_v1
        );
        write_report!(
            "fifth_patch_no_apply_guarantee_v4.txt",
            self.fifth_patch_no_apply_guarantee_report_v4
        );
        write_report!(
            "candidate_stop_consolidation_v2.txt",
            self.candidate_stop_consolidation_report_v2
        );
        write_report!(
            "consolidation_track_pause_v1.txt",
            self.consolidation_track_pause_report_v1
        );
        write_report!(
            "workspace_timeout_track_split_v1.txt",
            self.workspace_timeout_track_split_report_v1
        );
        write_report!(
            "workspace_timeout_diagnostic_track_plan_v1.txt",
            self.workspace_timeout_diagnostic_track_plan_v1
        );
        write_report!(
            "workspace_timeout_observation_backlog_v1.txt",
            self.workspace_timeout_observation_backlog_v1
        );
        write_report!(
            "workspace_no_run_observation_plan_v2.txt",
            self.workspace_no_run_observation_plan_v2
        );
        write_report!(
            "workspace_full_observation_plan_v2.txt",
            self.workspace_full_observation_plan_v2
        );
        write_report!(
            "cargo_json_observation_plan_v2.txt",
            self.cargo_json_observation_plan_v2
        );
        write_report!(
            "target_family_diagnostic_backlog_v1.txt",
            self.target_family_diagnostic_backlog_v1
        );
        write_report!(
            "link_macro_diagnostic_backlog_v1.txt",
            self.link_macro_diagnostic_backlog_v1
        );
        write_report!(
            "integration_fanout_diagnostic_backlog_v1.txt",
            self.integration_fanout_diagnostic_backlog_v1
        );
        write_report!(
            "cumulative_safe_patch_ledger_v6.txt",
            self.cumulative_safe_patch_ledger_v6
        );
        write_report!(
            "cumulative_binary_delta_v5.txt",
            self.cumulative_binary_delta_report_v5
        );
        write_report!(
            "assertion_ledger_continuity_check_v5.txt",
            self.assertion_ledger_continuity_check_v5
        );
        write_report!(
            "equivalent_coverage_continuity_check_v5.txt",
            self.equivalent_coverage_continuity_check_v5
        );
        write_report!(
            "safety_sentinel_continuity_check_v5.txt",
            self.safety_sentinel_continuity_check_v5
        );
        write_report!(
            "no_hidden_skip_continuity_check_v5.txt",
            self.no_hidden_skip_continuity_check_v5
        );
        write_report!(
            "timeout_cleanup_verification_v8.txt",
            self.timeout_cleanup_verification_report_v8
        );
        write_report!(
            "workspace_no_run_recovery_gate_v16.txt",
            self.workspace_no_run_recovery_gate_v16
        );
        write_report!(
            "workspace_full_acceptance_gate_v16.txt",
            self.workspace_full_acceptance_gate_v16
        );
        write_report!(
            "focused_vs_full_bridge_v12.txt",
            self.focused_vs_full_bridge_v12
        );
        write_report!(
            "acceptance_truth_gate_v16.txt",
            self.acceptance_truth_gate_v16
        );
        write_report!(
            "acceptance_evidence_strength_v5.txt",
            self.acceptance_evidence_strength_report_v5
        );
        write_report!(
            "workspace_recovery_decision_v5.txt",
            self.workspace_recovery_decision_report_v5
        );
        write_report!(
            "safety_coverage_preservation_v31.txt",
            self.safety_coverage_preservation_report_v31
        );
        write_report!(
            "control_tower_consolidation_governance_panel.txt",
            self.control_tower_consolidation_governance_panel
        );
        write_report!(
            "control_tower_workspace_timeout_track_panel.txt",
            self.control_tower_workspace_timeout_track_panel
        );
        let storage_report = ConsolidationStopResumeGovernanceStorageReport {
            report_id: "consolidation-stop-resume-governance-storage-report".to_string(),
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
        Ok(ConsolidationStopResumeGovernanceStorageReport {
            written_files,
            ..storage_report
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct ConsolidationStopResumeGovernanceRunner;

impl ConsolidationStopResumeGovernanceRunner {
    pub fn run(
        &self,
        config: &ConsolidationStopResumeGovernanceConfig,
    ) -> Result<ConsolidationStopResumeGovernanceBundle, String> {
        config.validate()?;
        let summary = load_first_json::<Sprint114SummaryFixture>(
            config
                .sprint114_truth_paths
                .as_ref()
                .or(config.sprint114_bundle_paths.as_ref()),
        )?
        .unwrap_or_default();
        let sprint114_baseline_truth_import_report =
            build_sprint114_baseline_truth_import_report(&summary);
        let sprint114_stop_recommendation_carry_forward_report =
            build_sprint114_stop_recommendation_carry_forward_report(
                &sprint114_baseline_truth_import_report,
            );
        let consolidation_stop_decision_report_v1 =
            build_consolidation_stop_decision_report_v1(&sprint114_baseline_truth_import_report);
        let assertion_destination_proof_plan_v1 = build_assertion_destination_proof_plan_v1();
        let shared_fixture_harness_capacity_report_v1 =
            build_shared_fixture_harness_capacity_report_v1(&summary);
        let workspace_timeout_target_capacity_report_v1 =
            build_workspace_timeout_target_capacity_report_v1(&summary);
        let control_tower_assertion_move_risk_report_v1 =
            build_control_tower_assertion_move_risk_report_v1();
        let assertion_destination_capacity_report_v1 =
            build_assertion_destination_capacity_report_v1(
                &summary,
                &shared_fixture_harness_capacity_report_v1,
                &workspace_timeout_target_capacity_report_v1,
                &control_tower_assertion_move_risk_report_v1,
            );
        let evidence_blur_risk_report_v1 = build_evidence_blur_risk_report_v1(&summary);
        let assertion_move_semantic_drift_report_v1 =
            build_assertion_move_semantic_drift_report_v1();
        let assertion_move_determinism_risk_report_v1 =
            build_assertion_move_determinism_risk_report_v1();
        let assertion_move_cli_surface_risk_report_v1 =
            build_assertion_move_cli_surface_risk_report_v1();
        let assertion_move_safety_risk_report_v1 = build_assertion_move_safety_risk_report_v1();
        let equivalent_coverage_continuity_check_v5 =
            build_equivalent_coverage_continuity_check_v5();
        let assertion_destination_proof_gate_v1 = build_assertion_destination_proof_gate_v1(
            &assertion_destination_capacity_report_v1,
            &assertion_move_semantic_drift_report_v1,
            &assertion_move_determinism_risk_report_v1,
            &assertion_move_cli_surface_risk_report_v1,
            &assertion_move_safety_risk_report_v1,
            &equivalent_coverage_continuity_check_v5,
        );
        let evidence_blur_risk_gate_v1 =
            build_evidence_blur_risk_gate_v1(&evidence_blur_risk_report_v1);
        let workspace_no_run_recovery_gate_v16 = build_workspace_no_run_recovery_gate_v16(&summary);
        let workspace_full_acceptance_gate_v16 = build_workspace_full_acceptance_gate_v16(&summary);
        let acceptance_truth_gate_v16 = build_acceptance_truth_gate_v16(
            &sprint114_baseline_truth_import_report,
            &workspace_no_run_recovery_gate_v16,
            &workspace_full_acceptance_gate_v16,
        );
        let focused_vs_full_bridge_v12 = build_focused_vs_full_bridge_v12(
            &sprint114_baseline_truth_import_report,
            &workspace_no_run_recovery_gate_v16,
            &workspace_full_acceptance_gate_v16,
        );
        let acceptance_evidence_strength_report_v5 =
            build_acceptance_evidence_strength_report_v5(&acceptance_truth_gate_v16);
        let timeout_cleanup_verification_report_v8 =
            build_timeout_cleanup_verification_report_v8(&summary);
        let assertion_ledger_continuity_check_v5 = build_assertion_ledger_continuity_check_v5();
        let safety_sentinel_continuity_check_v5 = build_safety_sentinel_continuity_check_v5();
        let no_hidden_skip_continuity_check_v5 = build_no_hidden_skip_continuity_check_v5();
        let safety_coverage_preservation_report_v31 =
            build_safety_coverage_preservation_report_v31();
        let consolidation_resume_decision_report_v1 = build_consolidation_resume_decision_report_v1(
            &assertion_destination_proof_gate_v1,
            &evidence_blur_risk_gate_v1,
        );
        let consolidation_track_pause_report_v1 =
            build_consolidation_track_pause_report_v1(&consolidation_resume_decision_report_v1);
        let workspace_timeout_track_split_report_v1 =
            build_workspace_timeout_track_split_report_v1(&consolidation_track_pause_report_v1);
        let consolidation_decision_matrix_v1 = build_consolidation_decision_matrix_v1(
            &consolidation_stop_decision_report_v1,
            &consolidation_resume_decision_report_v1,
            &workspace_timeout_track_split_report_v1,
        );
        let fifth_patch_no_apply_guarantee_report_v4 =
            build_fifth_patch_no_apply_guarantee_report_v4();
        let fifth_patch_stop_gate_v1 =
            build_fifth_patch_stop_gate_v1(&consolidation_stop_decision_report_v1);
        let fifth_patch_resume_gate_v5 = build_fifth_patch_resume_gate_v5(
            &assertion_destination_proof_gate_v1,
            &evidence_blur_risk_gate_v1,
            &consolidation_stop_decision_report_v1,
            &safety_coverage_preservation_report_v31,
            &acceptance_truth_gate_v16,
        );
        let candidate_stop_consolidation_report_v2 = build_candidate_stop_consolidation_report_v2(
            &consolidation_stop_decision_report_v1,
            &consolidation_resume_decision_report_v1,
        );
        let workspace_timeout_diagnostic_track_plan_v1 =
            build_workspace_timeout_diagnostic_track_plan_v1();
        let workspace_timeout_observation_backlog_v1 =
            build_workspace_timeout_observation_backlog_v1();
        let workspace_no_run_observation_plan_v2 =
            build_workspace_no_run_observation_plan_v2(&summary);
        let workspace_full_observation_plan_v2 = build_workspace_full_observation_plan_v2(&summary);
        let cargo_json_observation_plan_v2 = build_cargo_json_observation_plan_v2();
        let target_family_diagnostic_backlog_v1 =
            build_target_family_diagnostic_backlog_v1(&summary);
        let link_macro_diagnostic_backlog_v1 = build_link_macro_diagnostic_backlog_v1(&summary);
        let integration_fanout_diagnostic_backlog_v1 =
            build_integration_fanout_diagnostic_backlog_v1(&summary);
        let cumulative_safe_patch_ledger_v6 = build_cumulative_safe_patch_ledger_v6();
        let cumulative_binary_delta_report_v5 = build_cumulative_binary_delta_report_v5(&summary);
        let workspace_recovery_decision_report_v5 = build_workspace_recovery_decision_report_v5(
            &workspace_no_run_recovery_gate_v16,
            &workspace_full_acceptance_gate_v16,
        );
        let control_tower_consolidation_governance_panel =
            build_control_tower_consolidation_governance_panel(
                &consolidation_stop_decision_report_v1,
                &consolidation_resume_decision_report_v1,
                &assertion_destination_proof_gate_v1,
                &evidence_blur_risk_gate_v1,
                &fifth_patch_no_apply_guarantee_report_v4,
            );
        let control_tower_workspace_timeout_track_panel =
            build_control_tower_workspace_timeout_track_panel(
                &workspace_timeout_track_split_report_v1,
                &workspace_timeout_observation_backlog_v1,
                &workspace_no_run_recovery_gate_v16,
                &workspace_full_acceptance_gate_v16,
                &acceptance_truth_gate_v16,
            );
        let mut bundle = ConsolidationStopResumeGovernanceBundle {
            sprint114_baseline_truth_import_report,
            sprint114_stop_recommendation_carry_forward_report,
            consolidation_stop_decision_report_v1,
            consolidation_resume_decision_report_v1,
            consolidation_decision_matrix_v1,
            assertion_destination_proof_plan_v1,
            assertion_destination_capacity_report_v1,
            shared_fixture_harness_capacity_report_v1,
            workspace_timeout_target_capacity_report_v1,
            control_tower_assertion_move_risk_report_v1,
            evidence_blur_risk_report_v1,
            assertion_move_semantic_drift_report_v1,
            assertion_move_determinism_risk_report_v1,
            assertion_move_cli_surface_risk_report_v1,
            assertion_move_safety_risk_report_v1,
            assertion_destination_proof_gate_v1,
            evidence_blur_risk_gate_v1,
            fifth_patch_resume_gate_v5,
            fifth_patch_stop_gate_v1,
            fifth_patch_no_apply_guarantee_report_v4,
            candidate_stop_consolidation_report_v2,
            consolidation_track_pause_report_v1,
            workspace_timeout_track_split_report_v1,
            workspace_timeout_diagnostic_track_plan_v1,
            workspace_timeout_observation_backlog_v1,
            workspace_no_run_observation_plan_v2,
            workspace_full_observation_plan_v2,
            cargo_json_observation_plan_v2,
            target_family_diagnostic_backlog_v1,
            link_macro_diagnostic_backlog_v1,
            integration_fanout_diagnostic_backlog_v1,
            cumulative_safe_patch_ledger_v6,
            cumulative_binary_delta_report_v5,
            assertion_ledger_continuity_check_v5,
            equivalent_coverage_continuity_check_v5,
            safety_sentinel_continuity_check_v5,
            no_hidden_skip_continuity_check_v5,
            timeout_cleanup_verification_report_v8,
            workspace_no_run_recovery_gate_v16,
            workspace_full_acceptance_gate_v16,
            focused_vs_full_bridge_v12,
            acceptance_truth_gate_v16,
            acceptance_evidence_strength_report_v5,
            workspace_recovery_decision_report_v5,
            safety_coverage_preservation_report_v31,
            control_tower_consolidation_governance_panel,
            control_tower_workspace_timeout_track_panel,
            storage_report: ConsolidationStopResumeGovernanceStorageReport {
                report_id: "consolidation-stop-resume-governance-storage-report".to_string(),
                output_dir: config.output_dir().display().to_string(),
                written_files: Vec::new(),
                file_count: 49,
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
