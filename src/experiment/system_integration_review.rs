use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_ordered_strings, stable_reason_codes};
use crate::security::{
    SecretRedactionAuditConfig, SecretRedactionAuditReport, SecretRedactionAuditRunner,
    SecretRedactionStatus,
};

const TRINITY_PERSONA_IDS: [&str; 3] = [
    "trend_breakout_fast",
    "defensive_value_risk",
    "cycle_regime_guard",
];

fn default_output_root() -> String {
    "target/soma_system_review".to_string()
}

fn default_max_artifacts() -> usize {
    64
}

fn default_max_bytes() -> usize {
    20_000_000
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemIntegrationReviewConfig {
    pub review_id: String,
    #[serde(default)]
    pub core_check_report_paths: Vec<String>,
    #[serde(default)]
    pub core_completion_audit_paths: Vec<String>,
    #[serde(default)]
    pub core_scorecard_paths: Vec<String>,
    #[serde(default)]
    pub kis_smoke_report_paths: Vec<String>,
    #[serde(default)]
    pub kis_evidence_depth_report_paths: Vec<String>,
    #[serde(default)]
    pub control_tower_state_paths: Vec<String>,
    #[serde(default)]
    pub control_tower_refresh_paths: Vec<String>,
    #[serde(default)]
    pub trinity_loop_report_paths: Vec<String>,
    #[serde(default)]
    pub committee_cycle_report_paths: Vec<String>,
    #[serde(default)]
    pub chair_report_paths: Vec<String>,
    #[serde(default)]
    pub risk_report_paths: Vec<String>,
    #[serde(default)]
    pub owner_review_queue_paths: Vec<String>,
    #[serde(default)]
    pub paper_lifecycle_paths: Vec<String>,
    #[serde(default)]
    pub operational_runbook_paths: Vec<String>,
    #[serde(default)]
    pub dashboard_html_paths: Vec<String>,
    #[serde(default)]
    pub dashboard_json_paths: Vec<String>,
    #[serde(default)]
    pub artifact_diff_config_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_artifacts")]
    pub max_artifacts: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_core_check: bool,
    #[serde(default = "default_true")]
    pub require_control_tower: bool,
    #[serde(default = "default_true")]
    pub require_chair_readiness: bool,
    #[serde(default = "default_true")]
    pub require_trinity_readiness: bool,
    #[serde(default = "default_true")]
    pub require_risk_veto: bool,
    #[serde(default = "default_true")]
    pub require_owner_policy: bool,
    #[serde(default = "default_true")]
    pub require_paper_only: bool,
    #[serde(default = "default_true")]
    pub require_secret_redaction: bool,
    #[serde(default = "default_true")]
    pub require_determinism: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for SystemIntegrationReviewConfig {
    fn default() -> Self {
        Self {
            review_id: "sprint59-system-review".to_string(),
            core_check_report_paths: Vec::new(),
            core_completion_audit_paths: Vec::new(),
            core_scorecard_paths: Vec::new(),
            kis_smoke_report_paths: Vec::new(),
            kis_evidence_depth_report_paths: Vec::new(),
            control_tower_state_paths: Vec::new(),
            control_tower_refresh_paths: Vec::new(),
            trinity_loop_report_paths: Vec::new(),
            committee_cycle_report_paths: Vec::new(),
            chair_report_paths: Vec::new(),
            risk_report_paths: Vec::new(),
            owner_review_queue_paths: Vec::new(),
            paper_lifecycle_paths: Vec::new(),
            operational_runbook_paths: Vec::new(),
            dashboard_html_paths: Vec::new(),
            dashboard_json_paths: Vec::new(),
            artifact_diff_config_path: None,
            output_root: default_output_root(),
            max_artifacts: default_max_artifacts(),
            max_bytes: default_max_bytes(),
            require_core_check: true,
            require_control_tower: true,
            require_chair_readiness: true,
            require_trinity_readiness: true,
            require_risk_veto: true,
            require_owner_policy: true,
            require_paper_only: true,
            require_secret_redaction: true,
            require_determinism: true,
            reason_codes: vec![ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly],
        }
    }
}

impl SystemIntegrationReviewConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.review_id.trim().is_empty() {
            return Err("system integration review id must not be empty".to_string());
        }
        if self.max_artifacts == 0 || self.max_artifacts > 256 {
            return Err(
                "system integration review max_artifacts must be between 1 and 256".to_string(),
            );
        }
        if self.max_bytes == 0 || self.max_bytes > 50_000_000 {
            return Err(
                "system integration review max_bytes must be between 1 and 50000000".to_string(),
            );
        }
        let all_paths = self.all_paths();
        if all_paths.iter().any(|path| path.contains("://")) {
            return Err("system integration review paths must be local".to_string());
        }
        if all_paths.len() > self.max_artifacts {
            return Err(
                "system integration review artifact count exceeds max_artifacts".to_string(),
            );
        }
        let total_bytes = all_paths
            .iter()
            .filter_map(|path| fs::metadata(path).ok().map(|meta| meta.len() as usize))
            .sum::<usize>();
        if total_bytes > self.max_bytes {
            return Err("system integration review input bytes exceed max_bytes".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.review_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        let mut paths = Vec::new();
        paths.extend(self.core_check_report_paths.clone());
        paths.extend(self.core_completion_audit_paths.clone());
        paths.extend(self.core_scorecard_paths.clone());
        paths.extend(self.kis_smoke_report_paths.clone());
        paths.extend(self.kis_evidence_depth_report_paths.clone());
        paths.extend(self.control_tower_state_paths.clone());
        paths.extend(self.control_tower_refresh_paths.clone());
        paths.extend(self.trinity_loop_report_paths.clone());
        paths.extend(self.committee_cycle_report_paths.clone());
        paths.extend(self.chair_report_paths.clone());
        paths.extend(self.risk_report_paths.clone());
        paths.extend(self.owner_review_queue_paths.clone());
        paths.extend(self.paper_lifecycle_paths.clone());
        paths.extend(self.operational_runbook_paths.clone());
        paths.extend(self.dashboard_html_paths.clone());
        paths.extend(self.dashboard_json_paths.clone());
        if let Some(path) = &self.artifact_diff_config_path {
            paths.push(path.clone());
        }
        stable_ordered_strings(&paths)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReadinessArea {
    Core,
    KISProvider,
    EvidenceDepth,
    ControlTowerUI,
    Chair,
    TrinityCommittee,
    RiskGovernor,
    OwnerWorkflow,
    CandidateLifecycle,
    PaperLifecycle,
    OperationalRunbook,
    Determinism,
    SecretSafety,
    ShipGate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadinessStatus {
    Ready,
    ReadyWithWarnings,
    NeedsHardening,
    Blocked,
    Deferred,
    Forbidden,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadinessMatrixOverallStatus {
    ReadyForPaperOpsMonitoring,
    ReadyWithWarnings,
    NeedsHardening,
    #[default]
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadinessMatrixRow {
    pub area: ReadinessArea,
    pub status: ReadinessStatus,
    pub evidence_summary: String,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub next_action: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreUiChairCommitteeReadinessMatrix {
    #[serde(default)]
    pub rows: Vec<ReadinessMatrixRow>,
    pub ready_count: usize,
    pub warning_count: usize,
    pub needs_hardening_count: usize,
    pub blocked_count: usize,
    pub deferred_count: usize,
    pub forbidden_count: usize,
    pub overall_status: ReadinessMatrixOverallStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairOperationalReadinessStatus {
    Ready,
    ReadyWithWarnings,
    NeedsHardening,
    #[default]
    Blocked,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairOperationalReadinessReport {
    pub chair_version: String,
    pub selected_speaker_trace_available: bool,
    pub filtered_speaker_trace_available: bool,
    pub weighted_score_available: bool,
    pub uncertainty_available: bool,
    pub disagreement_available: bool,
    pub groupthink_warning_available: bool,
    pub risk_handoff_available: bool,
    pub veto_respected: bool,
    pub human_confirm_route_available: bool,
    pub no_bypass_detected: bool,
    pub readiness_status: ChairOperationalReadinessStatus,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrinityCommitteeReadinessStatus {
    Ready,
    ReadyWithWarnings,
    NeedsHardening,
    #[default]
    Blocked,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrinityMemberReadiness {
    pub persona_id: String,
    pub active: bool,
    pub scoring_available: bool,
    pub vote_output_available: bool,
    pub reason_codes_available: bool,
    pub dashboard_visible: bool,
    #[serde(default)]
    pub last_status: Option<String>,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrinityCommitteeReadinessReport {
    #[serde(default)]
    pub members: Vec<TrinityMemberReadiness>,
    pub active_member_count: usize,
    pub all_three_active: bool,
    pub no_extra_active_personas: bool,
    pub vote_cycle_available: bool,
    pub committee_work_queue_available: bool,
    pub candidate_generation_available: bool,
    pub operational_loop_available: bool,
    pub readiness_status: TrinityCommitteeReadinessStatus,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlTowerUiReadinessStatus {
    Ready,
    ReadyWithWarnings,
    NeedsHardening,
    #[default]
    Blocked,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerUiReadinessReport {
    pub dashboard_json_present: bool,
    pub dashboard_html_present: bool,
    pub provider_panel_present: bool,
    pub kis_monitor_panel_present: bool,
    pub evidence_panel_present: bool,
    pub committee_panel_present: bool,
    pub chair_panel_present: bool,
    pub risk_panel_present: bool,
    pub candidate_panel_present: bool,
    pub paper_panel_present: bool,
    pub owner_panel_present: bool,
    pub human_confirm_panel_present: bool,
    pub bottleneck_panel_present: bool,
    pub next_action_panel_present: bool,
    pub audit_timeline_present: bool,
    pub operational_loop_panel_present: bool,
    pub no_order_buttons: bool,
    pub no_account_panels: bool,
    pub no_secret_values: bool,
    pub local_only: bool,
    pub readiness_status: ControlTowerUiReadinessStatus,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndToEndPaperLoopAcceptanceStatus {
    Passed,
    PassedWithWarnings,
    Failed,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndToEndPaperLoopAcceptanceReport {
    pub acceptance_id: String,
    #[serde(default)]
    pub input_artifacts: Vec<String>,
    pub candidate_generated: bool,
    pub committee_votes_recorded: bool,
    pub chair_decision_recorded: bool,
    pub risk_decision_recorded: bool,
    pub owner_review_route_recorded: bool,
    pub paper_transition_recorded: bool,
    pub paper_position_simulated: bool,
    pub dashboard_refreshed: bool,
    pub runbook_emitted: bool,
    pub risk_block_test_passed: bool,
    pub no_trade_test_passed: bool,
    pub no_real_order_path_detected: bool,
    pub no_broker_path_detected: bool,
    pub acceptance_status: EndToEndPaperLoopAcceptanceStatus,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactDiffStatus {
    NoDiff,
    ExpectedDiff,
    UnexpectedDiff,
    MissingBaseline,
    MissingCurrent,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeterministicArtifactDiffConfig {
    pub diff_id: String,
    #[serde(default)]
    pub baseline_artifact_paths: Vec<String>,
    #[serde(default)]
    pub current_artifact_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub ignore_paths: Vec<String>,
    #[serde(default)]
    pub ignore_fields: Vec<String>,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for DeterministicArtifactDiffConfig {
    fn default() -> Self {
        Self {
            diff_id: "sprint59-artifact-diff".to_string(),
            baseline_artifact_paths: Vec::new(),
            current_artifact_paths: Vec::new(),
            output_root: default_output_root(),
            ignore_paths: Vec::new(),
            ignore_fields: Vec::new(),
            max_bytes: default_max_bytes(),
            reason_codes: vec![ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly],
        }
    }
}

impl DeterministicArtifactDiffConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.diff_id.trim().is_empty() {
            return Err("artifact diff id must not be empty".to_string());
        }
        let mut paths = self.baseline_artifact_paths.clone();
        paths.extend(self.current_artifact_paths.clone());
        paths.extend(self.ignore_paths.clone());
        paths.push(self.output_root.clone());
        if paths.iter().any(|path| path.contains("://")) {
            return Err("artifact diff paths must be local".to_string());
        }
        if self.max_bytes == 0 || self.max_bytes > 50_000_000 {
            return Err("artifact diff max_bytes must be between 1 and 50000000".to_string());
        }
        let total_bytes = self
            .baseline_artifact_paths
            .iter()
            .chain(self.current_artifact_paths.iter())
            .filter_map(|path| fs::metadata(path).ok().map(|meta| meta.len() as usize))
            .sum::<usize>();
        if total_bytes > self.max_bytes {
            return Err("artifact diff input bytes exceed max_bytes".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.diff_id)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicArtifactDiffReport {
    pub diff_id: String,
    pub compared_artifacts: usize,
    pub no_diff_count: usize,
    pub expected_diff_count: usize,
    pub unexpected_diff_count: usize,
    pub missing_baseline_count: usize,
    pub missing_current_count: usize,
    pub diff_status: ArtifactDiffStatus,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipChecklistItemStatus {
    Pass,
    PassWithWarning,
    Fail,
    NotApplicable,
    #[default]
    Deferred,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShipChecklistItem {
    pub item_id: String,
    pub title: String,
    pub description: String,
    pub status: ShipChecklistItemStatus,
    pub evidence: String,
    pub required_for_ship: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipChecklistOverallStatus {
    AllRequiredPassed,
    PassedWithWarnings,
    #[default]
    Failed,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualShipAcceptanceChecklist {
    pub checklist_id: String,
    #[serde(default)]
    pub items: Vec<ShipChecklistItem>,
    pub pass_count: usize,
    pub warning_count: usize,
    pub fail_count: usize,
    pub deferred_count: usize,
    pub all_required_passed: bool,
    pub overall_status: ShipChecklistOverallStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemShipGateStatus {
    ReadyToShipPaperOpsMonitoring,
    ReadyWithManualWarnings,
    BlockedBySafety,
    BlockedByDeterminism,
    BlockedByMissingUI,
    BlockedByChairCommitteeGap,
    BlockedByEvidenceGap,
    #[default]
    HoldShipAndFixGaps,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemShipGateRecommendation {
    ShipPaperOpsMonitoring,
    ReviewWarningsManually,
    FixCoreGaps,
    FixControlTowerGaps,
    FixChairCommitteeGaps,
    FixRiskOwnerGaps,
    FixDeterminismGaps,
    NeedMoreKISEvidence,
    #[default]
    KeepTrinity,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemShipGateReport {
    pub gate_id: String,
    pub readiness_matrix_status: ReadinessMatrixOverallStatus,
    pub paper_loop_acceptance_status: EndToEndPaperLoopAcceptanceStatus,
    pub artifact_diff_status: ArtifactDiffStatus,
    pub checklist_status: ShipChecklistOverallStatus,
    #[serde(default)]
    pub hard_blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub final_status: SystemShipGateStatus,
    pub final_recommendation: SystemShipGateRecommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemReviewStorageReport {
    pub artifact_count: usize,
    pub total_bytes: usize,
    pub max_bytes: usize,
    pub budget_exceeded: bool,
    #[serde(default)]
    pub largest_artifacts: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemIntegrationReviewBundle {
    pub readiness_matrix: CoreUiChairCommitteeReadinessMatrix,
    pub chair_readiness_report: ChairOperationalReadinessReport,
    pub trinity_readiness_report: TrinityCommitteeReadinessReport,
    pub control_tower_ui_readiness_report: ControlTowerUiReadinessReport,
    pub end_to_end_paper_loop_acceptance_report: EndToEndPaperLoopAcceptanceReport,
    #[serde(default)]
    pub deterministic_artifact_diff_report: Option<DeterministicArtifactDiffReport>,
    pub manual_ship_acceptance_checklist: ManualShipAcceptanceChecklist,
    pub system_ship_gate_report: SystemShipGateReport,
    pub storage_report: SystemReviewStorageReport,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SystemIntegrationReviewRunner;

#[derive(Clone, Debug, Default)]
struct ReviewArtifacts {
    core_check_value: Option<Value>,
    core_completion_value: Option<Value>,
    core_scorecard_value: Option<Value>,
    kis_smoke_value: Option<Value>,
    kis_evidence_depth_value: Option<Value>,
    control_tower_state_value: Option<Value>,
    trinity_loop_value: Option<Value>,
    committee_cycle_value: Option<Value>,
    chair_value: Option<Value>,
    risk_value: Option<Value>,
    owner_review_value: Option<Value>,
    paper_lifecycle_value: Option<Value>,
    operational_runbook_value: Option<Value>,
    dashboard_json_value: Option<Value>,
    dashboard_html_text: Option<String>,
    input_paths: Vec<String>,
}

impl ReviewArtifacts {
    fn load(config: &SystemIntegrationReviewConfig) -> Result<Self, String> {
        Ok(Self {
            core_check_value: load_latest_json(&config.core_check_report_paths)?,
            core_completion_value: load_latest_json(&config.core_completion_audit_paths)?,
            core_scorecard_value: load_latest_json(&config.core_scorecard_paths)?,
            kis_smoke_value: load_latest_json(&config.kis_smoke_report_paths)?,
            kis_evidence_depth_value: load_latest_json(&config.kis_evidence_depth_report_paths)?,
            control_tower_state_value: load_latest_json(&config.control_tower_state_paths)?,
            trinity_loop_value: load_latest_json(&config.trinity_loop_report_paths)?,
            committee_cycle_value: load_latest_json(&config.committee_cycle_report_paths)?,
            chair_value: load_latest_json(&config.chair_report_paths)?,
            risk_value: load_latest_json(&config.risk_report_paths)?,
            owner_review_value: load_latest_json(&config.owner_review_queue_paths)?,
            paper_lifecycle_value: load_latest_json(&config.paper_lifecycle_paths)?,
            operational_runbook_value: load_latest_json(&config.operational_runbook_paths)?,
            dashboard_json_value: load_latest_json(&config.dashboard_json_paths)?,
            dashboard_html_text: load_latest_text(&config.dashboard_html_paths)?,
            input_paths: config
                .all_paths()
                .into_iter()
                .filter(|path| Path::new(path).is_file())
                .collect(),
        })
    }
}

impl CoreUiChairCommitteeReadinessMatrix {
    pub fn stabilize(&mut self) {
        self.rows.sort_by(|left, right| left.area.cmp(&right.area));
        for row in &mut self.rows {
            row.blockers = stable_ordered_strings(&row.blockers);
            row.warnings = stable_ordered_strings(&row.warnings);
            row.reason_codes = stable_reason_codes(&row.reason_codes);
        }
        self.ready_count = self
            .rows
            .iter()
            .filter(|row| row.status == ReadinessStatus::Ready)
            .count();
        self.warning_count = self
            .rows
            .iter()
            .filter(|row| row.status == ReadinessStatus::ReadyWithWarnings)
            .count();
        self.needs_hardening_count = self
            .rows
            .iter()
            .filter(|row| row.status == ReadinessStatus::NeedsHardening)
            .count();
        self.blocked_count = self
            .rows
            .iter()
            .filter(|row| row.status == ReadinessStatus::Blocked)
            .count();
        self.deferred_count = self
            .rows
            .iter()
            .filter(|row| row.status == ReadinessStatus::Deferred)
            .count();
        self.forbidden_count = self
            .rows
            .iter()
            .filter(|row| row.status == ReadinessStatus::Forbidden)
            .count();
        self.overall_status = if self.blocked_count > 0 {
            ReadinessMatrixOverallStatus::Blocked
        } else if self.needs_hardening_count > 0 {
            ReadinessMatrixOverallStatus::NeedsHardening
        } else if self.warning_count > 0 || self.deferred_count > 0 {
            ReadinessMatrixOverallStatus::ReadyWithWarnings
        } else {
            ReadinessMatrixOverallStatus::ReadyForPaperOpsMonitoring
        };
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            "paper_only_warning=system integration review remains research-only and paper-ops-monitoring only".to_string(),
            format!("overall_status={:?}", self.overall_status),
            format!("ready_count={}", self.ready_count),
            format!("warning_count={}", self.warning_count),
            format!("needs_hardening_count={}", self.needs_hardening_count),
            format!("blocked_count={}", self.blocked_count),
            format!("deferred_count={}", self.deferred_count),
            format!("forbidden_count={}", self.forbidden_count),
        ];
        for row in &self.rows {
            lines.push(format!(
                "row={:?};status={:?};next_action={};blockers={};warnings={};evidence={}",
                row.area,
                row.status,
                row.next_action,
                row.blockers.join(" | "),
                row.warnings.join(" | "),
                row.evidence_summary
            ));
        }
        lines.join("\n")
    }
}

impl ChairOperationalReadinessReport {
    pub fn to_text(&self) -> String {
        [
            format!("chair_version={}", self.chair_version),
            format!(
                "selected_speaker_trace_available={}",
                self.selected_speaker_trace_available
            ),
            format!(
                "filtered_speaker_trace_available={}",
                self.filtered_speaker_trace_available
            ),
            format!("weighted_score_available={}", self.weighted_score_available),
            format!("uncertainty_available={}", self.uncertainty_available),
            format!("disagreement_available={}", self.disagreement_available),
            format!(
                "groupthink_warning_available={}",
                self.groupthink_warning_available
            ),
            format!("risk_handoff_available={}", self.risk_handoff_available),
            format!("veto_respected={}", self.veto_respected),
            format!(
                "human_confirm_route_available={}",
                self.human_confirm_route_available
            ),
            format!("no_bypass_detected={}", self.no_bypass_detected),
            format!("readiness_status={:?}", self.readiness_status),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}

impl TrinityCommitteeReadinessReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("active_member_count={}", self.active_member_count),
            format!("all_three_active={}", self.all_three_active),
            format!("no_extra_active_personas={}", self.no_extra_active_personas),
            format!("vote_cycle_available={}", self.vote_cycle_available),
            format!(
                "committee_work_queue_available={}",
                self.committee_work_queue_available
            ),
            format!(
                "candidate_generation_available={}",
                self.candidate_generation_available
            ),
            format!(
                "operational_loop_available={}",
                self.operational_loop_available
            ),
            format!("readiness_status={:?}", self.readiness_status),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ];
        for member in &self.members {
            lines.push(format!(
                "member={};active={};scoring={};vote_output={};reason_codes={};dashboard_visible={};last_status={}",
                member.persona_id,
                member.active,
                member.scoring_available,
                member.vote_output_available,
                member.reason_codes_available,
                member.dashboard_visible,
                member.last_status.clone().unwrap_or_default()
            ));
        }
        lines.join("\n")
    }
}

impl ControlTowerUiReadinessReport {
    pub fn to_text(&self) -> String {
        [
            format!("dashboard_json_present={}", self.dashboard_json_present),
            format!("dashboard_html_present={}", self.dashboard_html_present),
            format!("provider_panel_present={}", self.provider_panel_present),
            format!(
                "kis_monitor_panel_present={}",
                self.kis_monitor_panel_present
            ),
            format!("evidence_panel_present={}", self.evidence_panel_present),
            format!("committee_panel_present={}", self.committee_panel_present),
            format!("chair_panel_present={}", self.chair_panel_present),
            format!("risk_panel_present={}", self.risk_panel_present),
            format!("candidate_panel_present={}", self.candidate_panel_present),
            format!("paper_panel_present={}", self.paper_panel_present),
            format!("owner_panel_present={}", self.owner_panel_present),
            format!(
                "human_confirm_panel_present={}",
                self.human_confirm_panel_present
            ),
            format!("bottleneck_panel_present={}", self.bottleneck_panel_present),
            format!(
                "next_action_panel_present={}",
                self.next_action_panel_present
            ),
            format!("audit_timeline_present={}", self.audit_timeline_present),
            format!(
                "operational_loop_panel_present={}",
                self.operational_loop_panel_present
            ),
            format!("no_order_buttons={}", self.no_order_buttons),
            format!("no_account_panels={}", self.no_account_panels),
            format!("no_secret_values={}", self.no_secret_values),
            format!("local_only={}", self.local_only),
            format!("readiness_status={:?}", self.readiness_status),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}

impl EndToEndPaperLoopAcceptanceReport {
    pub fn to_text(&self) -> String {
        [
            format!("acceptance_id={}", self.acceptance_id),
            format!("input_artifacts={}", self.input_artifacts.join("|")),
            format!("candidate_generated={}", self.candidate_generated),
            format!("committee_votes_recorded={}", self.committee_votes_recorded),
            format!("chair_decision_recorded={}", self.chair_decision_recorded),
            format!("risk_decision_recorded={}", self.risk_decision_recorded),
            format!(
                "owner_review_route_recorded={}",
                self.owner_review_route_recorded
            ),
            format!(
                "paper_transition_recorded={}",
                self.paper_transition_recorded
            ),
            format!("paper_position_simulated={}", self.paper_position_simulated),
            format!("dashboard_refreshed={}", self.dashboard_refreshed),
            format!("runbook_emitted={}", self.runbook_emitted),
            format!("risk_block_test_passed={}", self.risk_block_test_passed),
            format!("no_trade_test_passed={}", self.no_trade_test_passed),
            format!(
                "no_real_order_path_detected={}",
                self.no_real_order_path_detected
            ),
            format!("no_broker_path_detected={}", self.no_broker_path_detected),
            format!("acceptance_status={:?}", self.acceptance_status),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}

impl DeterministicArtifactDiffReport {
    pub fn to_text(&self) -> String {
        [
            "deterministic_warning=artifact diff compares local fixture outputs only".to_string(),
            format!("diff_id={}", self.diff_id),
            format!("compared_artifacts={}", self.compared_artifacts),
            format!("no_diff_count={}", self.no_diff_count),
            format!("expected_diff_count={}", self.expected_diff_count),
            format!("unexpected_diff_count={}", self.unexpected_diff_count),
            format!("missing_baseline_count={}", self.missing_baseline_count),
            format!("missing_current_count={}", self.missing_current_count),
            format!("diff_status={:?}", self.diff_status),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("deterministic_artifact_diff.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("deterministic_artifact_diff.json"),
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

impl ManualShipAcceptanceChecklist {
    pub fn stabilize(&mut self) {
        self.items
            .sort_by(|left, right| left.item_id.cmp(&right.item_id));
        for item in &mut self.items {
            item.reason_codes = stable_reason_codes(&item.reason_codes);
        }
        self.pass_count = self
            .items
            .iter()
            .filter(|item| item.status == ShipChecklistItemStatus::Pass)
            .count();
        self.warning_count = self
            .items
            .iter()
            .filter(|item| item.status == ShipChecklistItemStatus::PassWithWarning)
            .count();
        self.fail_count = self
            .items
            .iter()
            .filter(|item| item.status == ShipChecklistItemStatus::Fail)
            .count();
        self.deferred_count = self
            .items
            .iter()
            .filter(|item| item.status == ShipChecklistItemStatus::Deferred)
            .count();
        self.all_required_passed =
            self.items
                .iter()
                .filter(|item| item.required_for_ship)
                .all(|item| {
                    matches!(
                        item.status,
                        ShipChecklistItemStatus::Pass
                            | ShipChecklistItemStatus::PassWithWarning
                            | ShipChecklistItemStatus::NotApplicable
                    )
                });
        self.overall_status = if !self.all_required_passed || self.fail_count > 0 {
            ShipChecklistOverallStatus::Failed
        } else if self.warning_count > 0 || self.deferred_count > 0 {
            ShipChecklistOverallStatus::PassedWithWarnings
        } else {
            ShipChecklistOverallStatus::AllRequiredPassed
        };
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("checklist_id={}", self.checklist_id),
            format!("pass_count={}", self.pass_count),
            format!("warning_count={}", self.warning_count),
            format!("fail_count={}", self.fail_count),
            format!("deferred_count={}", self.deferred_count),
            format!("all_required_passed={}", self.all_required_passed),
            format!("overall_status={:?}", self.overall_status),
        ];
        for item in &self.items {
            lines.push(format!(
                "item={};status={:?};required={};title={};evidence={}",
                item.item_id, item.status, item.required_for_ship, item.title, item.evidence
            ));
        }
        lines.join("\n")
    }
}

impl SystemShipGateReport {
    pub fn to_text(&self) -> String {
        [
            format!("gate_id={}", self.gate_id),
            format!("readiness_matrix_status={:?}", self.readiness_matrix_status),
            format!(
                "paper_loop_acceptance_status={:?}",
                self.paper_loop_acceptance_status
            ),
            format!("artifact_diff_status={:?}", self.artifact_diff_status),
            format!("checklist_status={:?}", self.checklist_status),
            format!("hard_blockers={}", self.hard_blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
        ]
        .join("\n")
    }
}

impl SystemReviewStorageReport {
    pub fn to_text(&self) -> String {
        [
            format!("artifact_count={}", self.artifact_count),
            format!("total_bytes={}", self.total_bytes),
            format!("max_bytes={}", self.max_bytes),
            format!("budget_exceeded={}", self.budget_exceeded),
            format!("largest_artifacts={}", self.largest_artifacts.join("|")),
        ]
        .join("\n")
    }
}

impl SystemIntegrationReviewBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("readiness_matrix.txt"),
            self.readiness_matrix.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("chair_operational_readiness.txt"),
            self.chair_readiness_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("trinity_committee_readiness.txt"),
            self.trinity_readiness_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("control_tower_ui_readiness.txt"),
            self.control_tower_ui_readiness_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("end_to_end_paper_loop_acceptance.txt"),
            self.end_to_end_paper_loop_acceptance_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("deterministic_artifact_diff.txt"),
            self.deterministic_artifact_diff_report
                .as_ref()
                .map(DeterministicArtifactDiffReport::to_text)
                .unwrap_or_else(|| {
                    "deterministic_warning=artifact diff not configured\ndiff_status=DiagnosticOnly"
                        .to_string()
                }),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("manual_ship_acceptance_checklist.txt"),
            self.manual_ship_acceptance_checklist.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("system_ship_gate.txt"),
            self.system_ship_gate_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("storage_report.txt"),
            self.storage_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let summary_path = output_dir.join("summary.txt");
        fs::write(&summary_path, &self.final_summary).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("system_integration_review_bundle.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(summary_path)
    }
}

impl SystemIntegrationReviewRunner {
    pub fn run(
        &self,
        config: &SystemIntegrationReviewConfig,
    ) -> Result<SystemIntegrationReviewBundle, String> {
        config.validate()?;
        let output_dir = config.artifact_dir();
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;

        let artifacts = ReviewArtifacts::load(config)?;
        let secret_redaction_report = run_secret_redaction_audit(config, &artifacts.input_paths)?;
        let chair_readiness_report = build_chair_operational_readiness(
            artifacts
                .chair_value
                .as_ref()
                .or(artifacts.committee_cycle_value.as_ref()),
            artifacts.owner_review_value.as_ref(),
        );
        let trinity_readiness_report =
            build_trinity_committee_readiness(artifacts.trinity_loop_value.as_ref());
        let control_tower_ui_readiness_report = build_control_tower_ui_readiness(
            artifacts
                .dashboard_json_value
                .as_ref()
                .or(artifacts.control_tower_state_value.as_ref()),
            artifacts.dashboard_html_text.as_deref(),
            &secret_redaction_report,
        );
        let end_to_end_paper_loop_acceptance_report = build_end_to_end_paper_loop_acceptance(
            config,
            &artifacts,
            &chair_readiness_report,
            &control_tower_ui_readiness_report,
        );
        let deterministic_artifact_diff_report = match config.artifact_diff_config_path.as_deref() {
            Some(path) => {
                let diff_config = DeterministicArtifactDiffConfig::from_toml_path(Path::new(path))?;
                Some(run_deterministic_artifact_diff(&diff_config)?)
            }
            None => None,
        };
        let mut readiness_matrix = build_readiness_matrix(
            config,
            &artifacts,
            &chair_readiness_report,
            &trinity_readiness_report,
            &control_tower_ui_readiness_report,
            &end_to_end_paper_loop_acceptance_report,
            deterministic_artifact_diff_report.as_ref(),
            &secret_redaction_report,
        );
        let manual_ship_acceptance_checklist = build_manual_ship_acceptance_checklist(
            config,
            &artifacts,
            &readiness_matrix,
            &chair_readiness_report,
            &trinity_readiness_report,
            &control_tower_ui_readiness_report,
            &end_to_end_paper_loop_acceptance_report,
            deterministic_artifact_diff_report.as_ref(),
        );
        let system_ship_gate_report = build_system_ship_gate_report(
            config,
            &readiness_matrix,
            &chair_readiness_report,
            &trinity_readiness_report,
            &control_tower_ui_readiness_report,
            &end_to_end_paper_loop_acceptance_report,
            deterministic_artifact_diff_report.as_ref(),
            &manual_ship_acceptance_checklist,
        );
        readiness_matrix
            .rows
            .push(ship_gate_row(&system_ship_gate_report));
        readiness_matrix.reason_codes.extend([
            ReasonCode::SystemReadinessMatrixBuilt,
            ReasonCode::SystemShipGateBuilt,
        ]);
        readiness_matrix.stabilize();
        let storage_report = build_storage_report(config, &artifacts.input_paths);
        let final_summary = build_final_summary(
            config,
            &readiness_matrix,
            &chair_readiness_report,
            &trinity_readiness_report,
            &control_tower_ui_readiness_report,
            &end_to_end_paper_loop_acceptance_report,
            deterministic_artifact_diff_report.as_ref(),
            &manual_ship_acceptance_checklist,
            &system_ship_gate_report,
        );
        let bundle = SystemIntegrationReviewBundle {
            readiness_matrix,
            chair_readiness_report,
            trinity_readiness_report,
            control_tower_ui_readiness_report,
            end_to_end_paper_loop_acceptance_report,
            deterministic_artifact_diff_report,
            manual_ship_acceptance_checklist,
            system_ship_gate_report,
            storage_report,
            final_summary,
            reason_codes: stable_reason_codes(
                &[
                    config.reason_codes.clone(),
                    vec![
                        ReasonCode::SystemIntegrationReviewBuilt,
                        ReasonCode::SystemReviewSummaryBuilt,
                    ],
                ]
                .concat(),
            ),
        };
        bundle.write_to_dir(&output_dir)?;
        Ok(bundle)
    }
}

pub fn run_deterministic_artifact_diff(
    config: &DeterministicArtifactDiffConfig,
) -> Result<DeterministicArtifactDiffReport, String> {
    config.validate()?;
    let baseline_paths = stable_ordered_strings(&config.baseline_artifact_paths);
    let current_paths = stable_ordered_strings(&config.current_artifact_paths);
    let compared_artifacts = baseline_paths.len().max(current_paths.len());
    let mut no_diff_count = 0;
    let mut expected_diff_count = 0;
    let mut unexpected_diff_count = 0;
    let mut missing_baseline_count = 0;
    let mut missing_current_count = 0;

    for index in 0..compared_artifacts {
        let baseline_path = baseline_paths.get(index);
        let current_path = current_paths.get(index);
        match (baseline_path, current_path) {
            (Some(baseline), Some(current)) => {
                if !Path::new(baseline).is_file() {
                    missing_baseline_count += 1;
                    continue;
                }
                if !Path::new(current).is_file() {
                    missing_current_count += 1;
                    continue;
                }
                let baseline_text = normalize_artifact(
                    &fs::read_to_string(baseline).map_err(|err| err.to_string())?,
                    &config.ignore_fields,
                );
                let current_text = normalize_artifact(
                    &fs::read_to_string(current).map_err(|err| err.to_string())?,
                    &config.ignore_fields,
                );
                if baseline_text == current_text {
                    no_diff_count += 1;
                } else if path_is_ignored(baseline, current, &config.ignore_paths) {
                    expected_diff_count += 1;
                } else {
                    unexpected_diff_count += 1;
                }
            }
            (Some(_), None) => missing_current_count += 1,
            (None, Some(_)) => missing_baseline_count += 1,
            (None, None) => {}
        }
    }

    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let diff_status = if compared_artifacts == 0 {
        ArtifactDiffStatus::DiagnosticOnly
    } else if unexpected_diff_count > 0 {
        blockers.push("unexpected deterministic artifact drift detected".to_string());
        ArtifactDiffStatus::UnexpectedDiff
    } else if missing_baseline_count > 0 {
        blockers.push("baseline artifact is missing".to_string());
        ArtifactDiffStatus::MissingBaseline
    } else if missing_current_count > 0 {
        blockers.push("current artifact is missing".to_string());
        ArtifactDiffStatus::MissingCurrent
    } else if expected_diff_count > 0 {
        warnings.push("configured expected diffs remain present".to_string());
        ArtifactDiffStatus::ExpectedDiff
    } else {
        ArtifactDiffStatus::NoDiff
    };

    let report = DeterministicArtifactDiffReport {
        diff_id: config.diff_id.clone(),
        compared_artifacts,
        no_diff_count,
        expected_diff_count,
        unexpected_diff_count,
        missing_baseline_count,
        missing_current_count,
        diff_status,
        blockers: stable_ordered_strings(&blockers),
        warnings: stable_ordered_strings(&warnings),
        reason_codes: stable_reason_codes(
            &[
                config.reason_codes.clone(),
                vec![ReasonCode::DeterministicArtifactDiffBuilt],
            ]
            .concat(),
        ),
    };
    report.write_to_dir(&config.artifact_dir())?;
    Ok(report)
}

fn build_chair_operational_readiness(
    chair_value: Option<&Value>,
    owner_review_value: Option<&Value>,
) -> ChairOperationalReadinessReport {
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let selected_speaker_trace_available = chair_value
        .and_then(|value| lookup_array_len(value, "selected_speakers"))
        .unwrap_or(0)
        > 0;
    let filtered_speaker_trace_available = chair_value
        .and_then(|value| lookup_array_len(value, "filtered_speakers"))
        .unwrap_or_else(|| {
            chair_value
                .and_then(|value| lookup_array_len(value, "all_votes"))
                .unwrap_or(0)
        })
        > 0;
    let weighted_score_available = chair_value
        .and_then(|value| lookup_f64(value, "weighted_score"))
        .is_some();
    let uncertainty_available = chair_value
        .and_then(|value| lookup_f64(value, "uncertainty"))
        .is_some();
    let disagreement_available = chair_value
        .and_then(|value| lookup_f64(value, "disagreement_score"))
        .is_some();
    let groupthink_warning_available = chair_value.is_some_and(|value| {
        has_key(value, "groupthink_warning") || lookup_f64(value, "groupthink_risk").is_some()
    });
    let risk_handoff_available = chair_value
        .and_then(|value| lookup_string(value, "risk_decision"))
        .is_some()
        || chair_value
            .and_then(|value| lookup_bool(value, "risk_handoff_available"))
            .unwrap_or(false)
        || chair_value
            .and_then(|value| find_value(value, "risk_decision"))
            .is_some();
    let human_confirm_route_available = chair_value
        .and_then(|value| lookup_bool(value, "human_confirm_route_available"))
        .unwrap_or_else(|| {
            chair_value
                .and_then(|value| lookup_bool(value, "human_confirm_required"))
                .unwrap_or(false)
                || chair_value
                    .and_then(|value| find_value(value, "owner_review_item"))
                    .is_some()
                || owner_review_value.is_some()
        });
    let veto_respected = chair_value.is_some_and(|value| {
        let denied = lookup_string(value, "risk_decision")
            .map(|decision| decision.to_ascii_lowercase().contains("deny"))
            .unwrap_or(false);
        let vetoed = lookup_string(value, "final_decision")
            .map(|decision| {
                let lower = decision.to_ascii_lowercase();
                lower.contains("veto") || lower.contains("notrade")
            })
            .unwrap_or(false);
        let paper_transition =
            lookup_bool(value, "paper_transition_recorded").unwrap_or_else(|| {
                find_value(value, "paper_transition").is_some()
                    || find_value(value, "paper_position").is_some()
            });
        !(denied || vetoed) || !paper_transition
    });
    let no_bypass_detected = chair_value.is_some_and(|value| {
        !lookup_bool(value, "bypass_detected").unwrap_or(false)
            && !lookup_bool(value, "real_order_path").unwrap_or(false)
            && !has_non_paper_order_plan(value)
    });

    if chair_value.is_none() {
        blockers.push("chair evidence is missing".to_string());
    }
    if !selected_speaker_trace_available {
        blockers.push("selected speaker trace is missing".to_string());
    }
    if !filtered_speaker_trace_available {
        warnings.push("filtered speaker trace is missing".to_string());
    }
    if !weighted_score_available {
        blockers.push("weighted chair score is missing".to_string());
    }
    if !uncertainty_available {
        warnings.push("chair uncertainty is missing".to_string());
    }
    if !disagreement_available {
        warnings.push("chair disagreement is missing".to_string());
    }
    if !groupthink_warning_available {
        warnings.push("groupthink visibility is missing".to_string());
    }
    if !risk_handoff_available {
        blockers.push("chair risk handoff is missing".to_string());
    }
    if !human_confirm_route_available {
        warnings.push("human confirm route is not visible".to_string());
    }
    if !veto_respected {
        blockers.push("chair veto was not respected by downstream paper flow".to_string());
    }
    if !no_bypass_detected {
        blockers.push("chair/owner bypass path was detected".to_string());
    }

    let readiness_status = if !blockers.is_empty() {
        ChairOperationalReadinessStatus::Blocked
    } else if !warnings.is_empty() {
        if selected_speaker_trace_available && weighted_score_available && risk_handoff_available {
            ChairOperationalReadinessStatus::ReadyWithWarnings
        } else {
            ChairOperationalReadinessStatus::NeedsHardening
        }
    } else {
        ChairOperationalReadinessStatus::Ready
    };

    ChairOperationalReadinessReport {
        chair_version: lookup_string_opt(chair_value, "chair_version")
            .unwrap_or_else(|| "ChairV0".to_string()),
        selected_speaker_trace_available,
        filtered_speaker_trace_available,
        weighted_score_available,
        uncertainty_available,
        disagreement_available,
        groupthink_warning_available,
        risk_handoff_available,
        veto_respected,
        human_confirm_route_available,
        no_bypass_detected,
        readiness_status,
        blockers: stable_ordered_strings(&blockers),
        warnings: stable_ordered_strings(&warnings),
        reason_codes: stable_reason_codes(&[
            ReasonCode::ChairOperationalReadinessBuilt,
            ReasonCode::ChairV0Built,
        ]),
    }
}

fn build_trinity_committee_readiness(
    trinity_value: Option<&Value>,
) -> TrinityCommitteeReadinessReport {
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let persona_views = trinity_value
        .and_then(|value| find_value(value, "persona_views"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let active_member_count = trinity_value
        .and_then(|value| lookup_usize(value, "active_count"))
        .unwrap_or_else(|| persona_views.len());
    let vote_cycle_available = trinity_value
        .and_then(|value| lookup_usize(value, "cycle_count"))
        .unwrap_or(0)
        > 0;
    let committee_work_queue_available = vote_cycle_available
        || trinity_value
            .and_then(|value| lookup_usize(value, "active_cycle_count"))
            .unwrap_or(0)
            > 0;
    let candidate_generation_available = trinity_value
        .and_then(|value| lookup_usize(value, "generated_candidate_count"))
        .unwrap_or(0)
        > 0;
    let operational_loop_available = trinity_value.is_some();

    let members = TRINITY_PERSONA_IDS
        .iter()
        .map(|persona_id| {
            let source = persona_views.iter().find(|item| {
                item.get("persona_id")
                    .and_then(Value::as_str)
                    .is_some_and(|value| value == *persona_id)
            });
            let active = source.is_some();
            let scoring_available = source.is_some_and(|item| {
                item.get("last_conviction").is_some() || item.get("last_voice_power").is_some()
            });
            let vote_output_available = source
                .is_some_and(|item| item.get("last_stance").and_then(Value::as_str).is_some());
            let reason_codes_available = source.is_some_and(|item| {
                item.get("reason_codes")
                    .and_then(Value::as_array)
                    .is_some_and(|items| !items.is_empty())
            });
            let dashboard_visible = active;
            let last_status = source
                .and_then(|item| item.get("status"))
                .and_then(Value::as_str)
                .map(str::to_string);
            let mut member_blockers = Vec::new();
            let mut member_warnings = Vec::new();
            if !active {
                member_blockers.push("persona is not active".to_string());
            }
            if active && !scoring_available {
                member_warnings.push("persona scoring visibility is missing".to_string());
            }
            if active && !vote_output_available {
                member_warnings.push("persona vote output is missing".to_string());
            }
            if active && !reason_codes_available {
                member_warnings.push("persona reason codes are missing".to_string());
            }
            TrinityMemberReadiness {
                persona_id: (*persona_id).to_string(),
                active,
                scoring_available,
                vote_output_available,
                reason_codes_available,
                dashboard_visible,
                last_status,
                blockers: stable_ordered_strings(&member_blockers),
                warnings: stable_ordered_strings(&member_warnings),
                reason_codes: stable_reason_codes(&[ReasonCode::TrinityCommitteeReadinessBuilt]),
            }
        })
        .collect::<Vec<_>>();

    let all_three_active = members.iter().all(|member| member.active);
    let no_extra_active_personas = persona_views.iter().all(|item| {
        item.get("persona_id")
            .and_then(Value::as_str)
            .is_some_and(|value| TRINITY_PERSONA_IDS.contains(&value))
    }) && active_member_count == 3;

    if !operational_loop_available {
        blockers.push("trinity operational loop artifact is missing".to_string());
    }
    if !all_three_active {
        blockers.push("exactly three active trinity personas are required".to_string());
    }
    if !no_extra_active_personas {
        blockers.push("unexpected active persona expansion was detected".to_string());
    }
    if !vote_cycle_available {
        warnings.push("committee vote cycle visibility is missing".to_string());
    }
    if !committee_work_queue_available {
        warnings.push("committee work queue visibility is missing".to_string());
    }
    if !candidate_generation_available {
        warnings.push("candidate generation evidence is missing".to_string());
    }
    if members.iter().any(|member| !member.reason_codes_available) {
        warnings.push("one or more active personas are missing reason codes".to_string());
    }

    let readiness_status = if !blockers.is_empty() {
        TrinityCommitteeReadinessStatus::Blocked
    } else if !warnings.is_empty() {
        TrinityCommitteeReadinessStatus::ReadyWithWarnings
    } else {
        TrinityCommitteeReadinessStatus::Ready
    };

    TrinityCommitteeReadinessReport {
        members,
        active_member_count,
        all_three_active,
        no_extra_active_personas,
        vote_cycle_available,
        committee_work_queue_available,
        candidate_generation_available,
        operational_loop_available,
        readiness_status,
        blockers: stable_ordered_strings(&blockers),
        warnings: stable_ordered_strings(&warnings),
        reason_codes: stable_reason_codes(&[ReasonCode::TrinityCommitteeReadinessBuilt]),
    }
}

fn build_control_tower_ui_readiness(
    dashboard_value: Option<&Value>,
    dashboard_html: Option<&str>,
    secret_redaction_report: &SecretRedactionAuditReport,
) -> ControlTowerUiReadinessReport {
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let dashboard_json_present = dashboard_value.is_some();
    let dashboard_html_present = dashboard_html.is_some();
    let provider_panel_present =
        dashboard_value.is_some_and(|value| has_key(value, "provider_panel"));
    let kis_monitor_panel_present =
        dashboard_value.is_some_and(|value| has_key(value, "kis_monitor_panel"));
    let evidence_panel_present =
        dashboard_value.is_some_and(|value| has_key(value, "evidence_panel"));
    let committee_panel_present =
        dashboard_value.is_some_and(|value| has_key(value, "committee_panel"));
    let chair_panel_present = dashboard_value.is_some_and(|value| has_key(value, "chair_panel"));
    let risk_panel_present = dashboard_value.is_some_and(|value| has_key(value, "risk_panel"));
    let candidate_panel_present =
        dashboard_value.is_some_and(|value| has_key(value, "candidate_panel"));
    let paper_panel_present = dashboard_value.is_some_and(|value| {
        has_key(value, "paper_position_panel") || has_key(value, "paper_panel")
    });
    let owner_panel_present = dashboard_value.is_some_and(|value| has_key(value, "owner_panel"));
    let human_confirm_panel_present =
        dashboard_value.is_some_and(|value| has_key(value, "human_confirm_panel"));
    let bottleneck_panel_present =
        dashboard_value.is_some_and(|value| has_key(value, "bottleneck_panel"));
    let next_action_panel_present =
        dashboard_value.is_some_and(|value| has_key(value, "next_action_panel"));
    let audit_timeline_present =
        dashboard_value.is_some_and(|value| has_key(value, "audit_timeline"));
    let operational_loop_panel_present =
        dashboard_value.is_some_and(|value| has_key(value, "operational_loop_panel"));
    let html_lower = dashboard_html.unwrap_or_default().to_ascii_lowercase();
    let no_order_buttons = !contains_unsafe_button(&html_lower);
    let no_account_panels = !has_key_opt(dashboard_value, "account_panel")
        && !html_lower.contains("account panel")
        && !html_lower.contains("balance panel")
        && !secret_redaction_report.account_like_fields_detected;
    let no_secret_values = matches!(
        secret_redaction_report.redaction_status,
        SecretRedactionStatus::Passed | SecretRedactionStatus::DiagnosticOnly
    );
    let local_only = !html_lower.contains("http://") && !html_lower.contains("https://");

    if !dashboard_json_present {
        blockers.push("dashboard JSON artifact is missing".to_string());
    }
    if !dashboard_html_present {
        warnings.push("dashboard HTML artifact is missing".to_string());
    }
    for (present, name) in [
        (provider_panel_present, "provider panel"),
        (kis_monitor_panel_present, "KIS monitor panel"),
        (evidence_panel_present, "evidence panel"),
        (committee_panel_present, "committee panel"),
        (chair_panel_present, "chair panel"),
        (risk_panel_present, "risk panel"),
        (candidate_panel_present, "candidate panel"),
        (paper_panel_present, "paper panel"),
        (owner_panel_present, "owner panel"),
        (human_confirm_panel_present, "human confirm panel"),
        (bottleneck_panel_present, "bottleneck panel"),
        (next_action_panel_present, "next action panel"),
        (audit_timeline_present, "audit timeline"),
        (operational_loop_panel_present, "operational loop panel"),
    ] {
        if !present {
            warnings.push(format!("{name} is missing"));
        }
    }
    if !no_order_buttons {
        blockers.push("unsafe order/trade buttons were detected in dashboard HTML".to_string());
    }
    if !no_account_panels {
        blockers.push("account-style UI surface was detected".to_string());
    }
    if !no_secret_values {
        blockers.push("secret or token-like values were detected in UI artifacts".to_string());
    }
    if !local_only {
        blockers.push("dashboard HTML references non-local resources".to_string());
    }

    let readiness_status = if !blockers.is_empty() {
        ControlTowerUiReadinessStatus::Blocked
    } else if warnings.iter().any(|warning| warning.contains("missing")) {
        ControlTowerUiReadinessStatus::NeedsHardening
    } else {
        ControlTowerUiReadinessStatus::Ready
    };

    ControlTowerUiReadinessReport {
        dashboard_json_present,
        dashboard_html_present,
        provider_panel_present,
        kis_monitor_panel_present,
        evidence_panel_present,
        committee_panel_present,
        chair_panel_present,
        risk_panel_present,
        candidate_panel_present,
        paper_panel_present,
        owner_panel_present,
        human_confirm_panel_present,
        bottleneck_panel_present,
        next_action_panel_present,
        audit_timeline_present,
        operational_loop_panel_present,
        no_order_buttons,
        no_account_panels,
        no_secret_values,
        local_only,
        readiness_status,
        blockers: stable_ordered_strings(&blockers),
        warnings: stable_ordered_strings(&warnings),
        reason_codes: stable_reason_codes(&[ReasonCode::ControlTowerUiReadinessBuilt]),
    }
}

fn build_end_to_end_paper_loop_acceptance(
    config: &SystemIntegrationReviewConfig,
    artifacts: &ReviewArtifacts,
    chair_report: &ChairOperationalReadinessReport,
    ui_report: &ControlTowerUiReadinessReport,
) -> EndToEndPaperLoopAcceptanceReport {
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let candidate_generated = artifacts
        .trinity_loop_value
        .as_ref()
        .and_then(|value| lookup_usize(value, "generated_candidate_count"))
        .unwrap_or(0)
        > 0
        || artifacts
            .dashboard_json_value
            .as_ref()
            .and_then(|value| lookup_usize(value, "candidate_count"))
            .unwrap_or(0)
            > 0;
    let committee_votes_recorded = artifacts
        .chair_value
        .as_ref()
        .and_then(|value| lookup_array_len(value, "all_votes"))
        .unwrap_or(0)
        > 0
        || artifacts
            .trinity_loop_value
            .as_ref()
            .and_then(|value| lookup_usize(value, "cycle_count"))
            .unwrap_or(0)
            > 0;
    let chair_decision_recorded = artifacts
        .chair_value
        .as_ref()
        .and_then(|value| lookup_string(value, "final_decision"))
        .is_some();
    let risk_decision_recorded = chair_report.risk_handoff_available
        || artifacts.risk_value.is_some()
        || artifacts
            .chair_value
            .as_ref()
            .and_then(|value| lookup_string(value, "risk_decision"))
            .is_some();
    let owner_review_route_recorded =
        artifacts.owner_review_value.is_some() || chair_report.human_confirm_route_available;
    let paper_transition_recorded = artifacts.paper_lifecycle_value.is_some()
        || artifacts
            .chair_value
            .as_ref()
            .and_then(|value| lookup_bool(value, "paper_transition_recorded"))
            .unwrap_or_else(|| {
                artifacts.chair_value.as_ref().is_some_and(|value| {
                    has_key(value, "paper_transition") || has_key(value, "paper_position")
                })
            });
    let paper_position_simulated = artifacts.paper_lifecycle_value.is_some()
        || artifacts
            .chair_value
            .as_ref()
            .and_then(|value| lookup_bool(value, "paper_only"))
            .unwrap_or(false);
    let dashboard_refreshed = ui_report.dashboard_json_present && ui_report.dashboard_html_present;
    let runbook_emitted = artifacts.operational_runbook_value.is_some();
    let owner_text = artifacts
        .owner_review_value
        .as_ref()
        .map(value_text)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let risk_block_test_passed = owner_text.contains("blockedbyriskgovernor")
        || (artifacts
            .chair_value
            .as_ref()
            .and_then(|value| lookup_string(value, "risk_decision"))
            .map(|value| value.to_ascii_lowercase().contains("deny"))
            .unwrap_or(false)
            && !paper_transition_recorded);
    let no_trade_test_passed = owner_text.contains("notrade")
        || (artifacts
            .chair_value
            .as_ref()
            .and_then(|value| lookup_string(value, "final_decision"))
            .map(|value| value.to_ascii_lowercase().contains("notrade"))
            .unwrap_or(false)
            && !paper_transition_recorded);
    let no_real_order_path_detected = chair_report.no_bypass_detected && ui_report.no_order_buttons;
    let no_broker_path_detected = ui_report.no_account_panels && ui_report.local_only;

    if !candidate_generated {
        blockers.push("candidate generation was not recorded".to_string());
    }
    if !committee_votes_recorded {
        blockers.push("committee votes were not recorded".to_string());
    }
    if !chair_decision_recorded {
        blockers.push("chair decision was not recorded".to_string());
    }
    if !risk_decision_recorded {
        blockers.push("risk decision was not recorded".to_string());
    }
    if !owner_review_route_recorded && config.require_owner_policy {
        warnings.push("owner review route was not recorded".to_string());
    }
    if !paper_transition_recorded {
        warnings.push("paper transition was not recorded".to_string());
    }
    if !paper_position_simulated && config.require_paper_only {
        blockers.push("paper lifecycle simulation evidence is missing".to_string());
    }
    if !dashboard_refreshed && config.require_control_tower {
        blockers.push("dashboard refresh evidence is missing".to_string());
    }
    if !runbook_emitted {
        warnings.push("runbook emission evidence is missing".to_string());
    }
    if !risk_block_test_passed && config.require_risk_veto {
        blockers.push("risk-block test did not prove veto behavior".to_string());
    }
    if !no_trade_test_passed {
        warnings.push("no-trade path was not explicitly proven".to_string());
    }
    if !no_real_order_path_detected {
        blockers.push("real order path was detected".to_string());
    }
    if !no_broker_path_detected {
        blockers.push("broker/account path was detected".to_string());
    }

    let acceptance_status = if artifacts.input_paths.is_empty() {
        EndToEndPaperLoopAcceptanceStatus::DiagnosticOnly
    } else if !blockers.is_empty() {
        EndToEndPaperLoopAcceptanceStatus::Failed
    } else if !warnings.is_empty() {
        EndToEndPaperLoopAcceptanceStatus::PassedWithWarnings
    } else {
        EndToEndPaperLoopAcceptanceStatus::Passed
    };

    EndToEndPaperLoopAcceptanceReport {
        acceptance_id: format!("{}-paper-loop", config.review_id),
        input_artifacts: artifacts.input_paths.clone(),
        candidate_generated,
        committee_votes_recorded,
        chair_decision_recorded,
        risk_decision_recorded,
        owner_review_route_recorded,
        paper_transition_recorded,
        paper_position_simulated,
        dashboard_refreshed,
        runbook_emitted,
        risk_block_test_passed,
        no_trade_test_passed,
        no_real_order_path_detected,
        no_broker_path_detected,
        acceptance_status,
        blockers: stable_ordered_strings(&blockers),
        warnings: stable_ordered_strings(&warnings),
        reason_codes: stable_reason_codes(&[ReasonCode::EndToEndPaperLoopAcceptanceBuilt]),
    }
}

fn build_readiness_matrix(
    config: &SystemIntegrationReviewConfig,
    artifacts: &ReviewArtifacts,
    chair_report: &ChairOperationalReadinessReport,
    trinity_report: &TrinityCommitteeReadinessReport,
    ui_report: &ControlTowerUiReadinessReport,
    acceptance_report: &EndToEndPaperLoopAcceptanceReport,
    diff_report: Option<&DeterministicArtifactDiffReport>,
    secret_redaction_report: &SecretRedactionAuditReport,
) -> CoreUiChairCommitteeReadinessMatrix {
    let mut rows = Vec::new();

    let core_present = artifacts.core_check_value.is_some()
        || artifacts.core_completion_value.is_some()
        || artifacts.core_scorecard_value.is_some();
    let core_warnings = artifacts
        .core_completion_value
        .as_ref()
        .and_then(|value| lookup_array_len(value, "warnings"))
        .unwrap_or(0);
    let core_status = if config.require_core_check && !core_present {
        ReadinessStatus::Blocked
    } else if artifacts
        .core_completion_value
        .as_ref()
        .and_then(|value| lookup_bool(value, "mamba_runtime_present"))
        .unwrap_or(false)
    {
        ReadinessStatus::Blocked
    } else if core_warnings > 0 {
        ReadinessStatus::ReadyWithWarnings
    } else {
        ReadinessStatus::Ready
    };
    rows.push(row(
        ReadinessArea::Core,
        core_status,
        if core_present {
            "core readiness, completion audit, or scorecard evidence loaded"
        } else {
            "missing core readiness artifacts"
        },
        if core_status == ReadinessStatus::Blocked {
            vec!["core check evidence is missing or blocked".to_string()]
        } else {
            Vec::new()
        },
        if core_status == ReadinessStatus::ReadyWithWarnings {
            vec!["core completion still carries warnings".to_string()]
        } else {
            Vec::new()
        },
        "refresh core-check/core-completion evidence",
        vec![ReasonCode::SystemReadinessMatrixBuilt],
    ));

    let kis_final_status = artifacts
        .kis_smoke_value
        .as_ref()
        .and_then(|value| lookup_string(value, "final_status"))
        .unwrap_or_default();
    let kis_status = if kis_final_status.is_empty() {
        ReadinessStatus::NeedsHardening
    } else if kis_final_status.to_ascii_lowercase().contains("missing")
        || kis_final_status.to_ascii_lowercase().contains("blocked")
    {
        ReadinessStatus::Blocked
    } else if kis_final_status.to_ascii_lowercase().contains("need")
        || kis_final_status.to_ascii_lowercase().contains("still")
    {
        ReadinessStatus::ReadyWithWarnings
    } else {
        ReadinessStatus::Ready
    };
    rows.push(row(
        ReadinessArea::KISProvider,
        kis_status,
        if kis_final_status.is_empty() {
            "missing KIS smoke evidence".to_string()
        } else {
            format!("kis final_status={kis_final_status}")
        },
        if kis_status == ReadinessStatus::Blocked {
            vec!["KIS market-data-only path is not closed yet".to_string()]
        } else {
            Vec::new()
        },
        if kis_status == ReadinessStatus::ReadyWithWarnings {
            vec!["KIS evidence remains conservative and paper-only".to_string()]
        } else {
            Vec::new()
        },
        "rerun bounded KIS smoke or review auth/evidence blockers",
        vec![ReasonCode::SystemReadinessMatrixBuilt],
    ));

    let evidence_status_text = artifacts
        .kis_smoke_value
        .as_ref()
        .and_then(|value| lookup_string(value, "evidence_depth_status"))
        .or_else(|| {
            artifacts
                .kis_evidence_depth_value
                .as_ref()
                .and_then(|value| lookup_string(value, "final_recommendation"))
        })
        .unwrap_or_default();
    let evidence_status = if evidence_status_text.is_empty() {
        ReadinessStatus::NeedsHardening
    } else if evidence_status_text.to_ascii_lowercase().contains("block") {
        ReadinessStatus::Blocked
    } else if evidence_status_text.to_ascii_lowercase().contains("need") {
        ReadinessStatus::ReadyWithWarnings
    } else {
        ReadinessStatus::Ready
    };
    rows.push(row(
        ReadinessArea::EvidenceDepth,
        evidence_status,
        if evidence_status_text.is_empty() {
            "missing evidence-depth signal".to_string()
        } else {
            format!("evidence_status={evidence_status_text}")
        },
        if evidence_status == ReadinessStatus::Blocked {
            vec!["evidence depth is explicitly blocked".to_string()]
        } else {
            Vec::new()
        },
        if evidence_status == ReadinessStatus::ReadyWithWarnings {
            vec!["evidence depth still needs more KIS or outcome-link coverage".to_string()]
        } else {
            Vec::new()
        },
        "expand bounded evidence depth before stronger claims",
        vec![ReasonCode::SystemReadinessMatrixBuilt],
    ));

    rows.push(row(
        ReadinessArea::ControlTowerUI,
        map_ui_status(ui_report.readiness_status),
        format!(
            "dashboard_json_present={};dashboard_html_present={}",
            ui_report.dashboard_json_present, ui_report.dashboard_html_present
        ),
        ui_report.blockers.clone(),
        ui_report.warnings.clone(),
        "refresh Control Tower fixtures and verify required panels",
        vec![ReasonCode::SystemReadinessMatrixBuilt],
    ));
    rows.push(row(
        ReadinessArea::Chair,
        map_chair_status(chair_report.readiness_status),
        format!(
            "risk_handoff={};no_bypass={}",
            chair_report.risk_handoff_available, chair_report.no_bypass_detected
        ),
        chair_report.blockers.clone(),
        chair_report.warnings.clone(),
        "refresh Chair trace and risk handoff artifacts",
        vec![ReasonCode::SystemReadinessMatrixBuilt],
    ));
    rows.push(row(
        ReadinessArea::TrinityCommittee,
        map_trinity_status(trinity_report.readiness_status),
        format!(
            "active_member_count={};all_three_active={}",
            trinity_report.active_member_count, trinity_report.all_three_active
        ),
        trinity_report.blockers.clone(),
        trinity_report.warnings.clone(),
        "keep exactly three active personas and refresh committee loop evidence",
        vec![ReasonCode::SystemReadinessMatrixBuilt],
    ));

    let risk_status = if chair_report.veto_respected
        && acceptance_report.risk_block_test_passed
        && config.require_risk_veto
    {
        ReadinessStatus::Ready
    } else if config.require_risk_veto {
        ReadinessStatus::Blocked
    } else {
        ReadinessStatus::Deferred
    };
    rows.push(row(
        ReadinessArea::RiskGovernor,
        risk_status,
        format!(
            "risk_handoff={};risk_block_test_passed={}",
            chair_report.risk_handoff_available, acceptance_report.risk_block_test_passed
        ),
        if risk_status == ReadinessStatus::Blocked {
            vec!["Risk Governor veto behavior is not fully proven".to_string()]
        } else {
            Vec::new()
        },
        Vec::new(),
        "replay blocked/no-trade committee cases and verify veto remains final",
        vec![ReasonCode::SystemReadinessMatrixBuilt],
    ));

    let owner_status = if !config.require_owner_policy {
        ReadinessStatus::Deferred
    } else if artifacts.owner_review_value.is_none() {
        ReadinessStatus::NeedsHardening
    } else if chair_report.no_bypass_detected {
        ReadinessStatus::ReadyWithWarnings
    } else {
        ReadinessStatus::Blocked
    };
    rows.push(row(
        ReadinessArea::OwnerWorkflow,
        owner_status,
        if artifacts.owner_review_value.is_some() {
            "owner review queue present".to_string()
        } else {
            "owner review queue missing".to_string()
        },
        if owner_status == ReadinessStatus::Blocked {
            vec!["owner workflow bypasses risk constraints".to_string()]
        } else {
            Vec::new()
        },
        if owner_status == ReadinessStatus::ReadyWithWarnings {
            vec![
                "owner workflow remains paper-only and still needs manual review discipline"
                    .to_string(),
            ]
        } else if owner_status == ReadinessStatus::NeedsHardening {
            vec!["owner review evidence is incomplete".to_string()]
        } else {
            Vec::new()
        },
        "refresh owner review queue and paper-confirm evidence",
        vec![ReasonCode::SystemReadinessMatrixBuilt],
    ));

    let candidate_status = if acceptance_report.candidate_generated
        && acceptance_report.committee_votes_recorded
        && acceptance_report.chair_decision_recorded
    {
        if acceptance_report.no_real_order_path_detected {
            ReadinessStatus::Ready
        } else {
            ReadinessStatus::Blocked
        }
    } else {
        ReadinessStatus::NeedsHardening
    };
    rows.push(row(
        ReadinessArea::CandidateLifecycle,
        candidate_status,
        format!(
            "candidate_generated={};committee_votes_recorded={};chair_decision_recorded={}",
            acceptance_report.candidate_generated,
            acceptance_report.committee_votes_recorded,
            acceptance_report.chair_decision_recorded
        ),
        if candidate_status == ReadinessStatus::Blocked {
            vec!["candidate lifecycle crossed into a real-order path".to_string()]
        } else {
            Vec::new()
        },
        Vec::new(),
        "refresh candidate lifecycle evidence and preserve Candidate != Order semantics",
        vec![ReasonCode::SystemReadinessMatrixBuilt],
    ));

    let paper_status = if acceptance_report.paper_position_simulated
        && acceptance_report.no_broker_path_detected
    {
        ReadinessStatus::Ready
    } else if artifacts.paper_lifecycle_value.is_some() {
        ReadinessStatus::ReadyWithWarnings
    } else {
        ReadinessStatus::NeedsHardening
    };
    rows.push(row(
        ReadinessArea::PaperLifecycle,
        paper_status,
        format!(
            "paper_position_simulated={};no_broker_path_detected={}",
            acceptance_report.paper_position_simulated, acceptance_report.no_broker_path_detected
        ),
        Vec::new(),
        if paper_status == ReadinessStatus::ReadyWithWarnings {
            vec!["paper lifecycle exists but simulation evidence is incomplete".to_string()]
        } else if paper_status == ReadinessStatus::NeedsHardening {
            vec!["paper lifecycle evidence is missing".to_string()]
        } else {
            Vec::new()
        },
        "refresh paper lifecycle report and confirm paper-only transitions",
        vec![ReasonCode::SystemReadinessMatrixBuilt],
    ));

    let runbook_status = if artifacts.operational_runbook_value.is_none() {
        ReadinessStatus::NeedsHardening
    } else {
        ReadinessStatus::Ready
    };
    rows.push(row(
        ReadinessArea::OperationalRunbook,
        runbook_status,
        if artifacts.operational_runbook_value.is_some() {
            "operational runbook artifact present".to_string()
        } else {
            "operational runbook artifact missing".to_string()
        },
        Vec::new(),
        if runbook_status == ReadinessStatus::NeedsHardening {
            vec!["local-only runbook evidence is missing".to_string()]
        } else {
            Vec::new()
        },
        "emit and review local-only next-action runbook",
        vec![ReasonCode::SystemReadinessMatrixBuilt],
    ));

    rows.push(row(
        ReadinessArea::Determinism,
        map_diff_status(
            diff_report.map(|report| report.diff_status),
            config.require_determinism,
        ),
        diff_report
            .map(|report| format!("diff_status={:?}", report.diff_status))
            .unwrap_or_else(|| "diff not configured".to_string()),
        diff_report
            .map(|report| report.blockers.clone())
            .unwrap_or_default(),
        diff_report
            .map(|report| report.warnings.clone())
            .unwrap_or_else(|| {
                if config.require_determinism {
                    vec!["deterministic artifact diff was not configured".to_string()]
                } else {
                    Vec::new()
                }
            }),
        "run deterministic artifact diff before ship decisions",
        vec![ReasonCode::SystemReadinessMatrixBuilt],
    ));

    let secret_status = if !config.require_secret_redaction {
        ReadinessStatus::Deferred
    } else if matches!(
        secret_redaction_report.redaction_status,
        SecretRedactionStatus::Passed | SecretRedactionStatus::DiagnosticOnly
    ) && ui_report.no_secret_values
    {
        ReadinessStatus::Ready
    } else {
        ReadinessStatus::Blocked
    };
    rows.push(row(
        ReadinessArea::SecretSafety,
        secret_status,
        format!(
            "redaction_status={:?};ui_no_secret_values={}",
            secret_redaction_report.redaction_status, ui_report.no_secret_values
        ),
        if secret_status == ReadinessStatus::Blocked {
            vec!["secret redaction audit detected unsafe content".to_string()]
        } else {
            Vec::new()
        },
        Vec::new(),
        "remove token/account/order leaks from local artifacts",
        vec![ReasonCode::SystemReadinessMatrixBuilt],
    ));

    let mut matrix = CoreUiChairCommitteeReadinessMatrix {
        rows,
        ready_count: 0,
        warning_count: 0,
        needs_hardening_count: 0,
        blocked_count: 0,
        deferred_count: 0,
        forbidden_count: 0,
        overall_status: ReadinessMatrixOverallStatus::Blocked,
        reason_codes: vec![ReasonCode::SystemReadinessMatrixBuilt],
    };
    matrix.stabilize();
    matrix
}

fn build_manual_ship_acceptance_checklist(
    config: &SystemIntegrationReviewConfig,
    artifacts: &ReviewArtifacts,
    matrix: &CoreUiChairCommitteeReadinessMatrix,
    chair_report: &ChairOperationalReadinessReport,
    trinity_report: &TrinityCommitteeReadinessReport,
    ui_report: &ControlTowerUiReadinessReport,
    acceptance_report: &EndToEndPaperLoopAcceptanceReport,
    diff_report: Option<&DeterministicArtifactDiffReport>,
) -> ManualShipAcceptanceChecklist {
    let mut checklist = ManualShipAcceptanceChecklist {
        checklist_id: format!("{}-ship-checklist", config.review_id),
        items: vec![
            checklist_item(
                "core-check-passed",
                "Core check passed",
                "Core artifacts exist and core readiness is not blocked.",
                match matrix
                    .rows
                    .iter()
                    .find(|row| row.area == ReadinessArea::Core)
                    .map(|row| row.status)
                {
                    Some(ReadinessStatus::Ready) => ShipChecklistItemStatus::Pass,
                    Some(ReadinessStatus::ReadyWithWarnings) => {
                        ShipChecklistItemStatus::PassWithWarning
                    }
                    Some(ReadinessStatus::Deferred) => ShipChecklistItemStatus::Deferred,
                    _ => ShipChecklistItemStatus::Fail,
                },
                matrix
                    .rows
                    .iter()
                    .find(|row| row.area == ReadinessArea::Core)
                    .map(|row| row.evidence_summary.clone())
                    .unwrap_or_default(),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "cargo-validation",
                "Cargo fmt/check/test passed",
                "Manual ship review assumes repository validation stays green.",
                ShipChecklistItemStatus::PassWithWarning,
                "cargo fmt/check/test must be confirmed during release workflow".to_string(),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "kis-market-data-only",
                "KIS path market-data-only",
                "KIS review must stay market-data-only and bounded.",
                match matrix
                    .rows
                    .iter()
                    .find(|row| row.area == ReadinessArea::KISProvider)
                    .map(|row| row.status)
                {
                    Some(ReadinessStatus::Ready) => ShipChecklistItemStatus::Pass,
                    Some(ReadinessStatus::ReadyWithWarnings) => {
                        ShipChecklistItemStatus::PassWithWarning
                    }
                    _ => ShipChecklistItemStatus::Fail,
                },
                artifacts
                    .kis_smoke_value
                    .as_ref()
                    .and_then(|value| lookup_string(value, "final_status"))
                    .unwrap_or_default(),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "kis-secrets-redacted",
                "KIS secrets redacted",
                "No secret values or token-like leaks may appear in artifacts.",
                if ui_report.no_secret_values {
                    ShipChecklistItemStatus::Pass
                } else {
                    ShipChecklistItemStatus::Fail
                },
                format!("ui_no_secret_values={}", ui_report.no_secret_values),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "broker-order-account-absent",
                "Broker/order/account paths absent",
                "No broker/order/account path can appear in review outputs.",
                if acceptance_report.no_real_order_path_detected
                    && acceptance_report.no_broker_path_detected
                {
                    ShipChecklistItemStatus::Pass
                } else {
                    ShipChecklistItemStatus::Fail
                },
                format!(
                    "no_real_order_path_detected={};no_broker_path_detected={}",
                    acceptance_report.no_real_order_path_detected,
                    acceptance_report.no_broker_path_detected
                ),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "control-tower-renders",
                "Control Tower renders",
                "Dashboard JSON/HTML must exist for local monitoring.",
                if ui_report.dashboard_json_present && ui_report.dashboard_html_present {
                    ShipChecklistItemStatus::Pass
                } else if ui_report.dashboard_json_present {
                    ShipChecklistItemStatus::PassWithWarning
                } else {
                    ShipChecklistItemStatus::Fail
                },
                format!(
                    "dashboard_json_present={};dashboard_html_present={}",
                    ui_report.dashboard_json_present, ui_report.dashboard_html_present
                ),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "ui-no-unsafe-controls",
                "UI has no order/account controls",
                "Control Tower must remain read-only and local-only.",
                if ui_report.no_order_buttons && ui_report.no_account_panels {
                    ShipChecklistItemStatus::Pass
                } else {
                    ShipChecklistItemStatus::Fail
                },
                format!(
                    "no_order_buttons={};no_account_panels={}",
                    ui_report.no_order_buttons, ui_report.no_account_panels
                ),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "chair-risk-handoff",
                "Chair handoff to Risk Governor works",
                "Chair must hand off to Risk and never bypass veto.",
                if chair_report.risk_handoff_available && chair_report.no_bypass_detected {
                    ShipChecklistItemStatus::Pass
                } else {
                    ShipChecklistItemStatus::Fail
                },
                format!(
                    "risk_handoff_available={};no_bypass_detected={}",
                    chair_report.risk_handoff_available, chair_report.no_bypass_detected
                ),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "trinity-three-active",
                "Trinity has exactly three active personas",
                "No 6/12 persona expansion is allowed.",
                if trinity_report.all_three_active && trinity_report.no_extra_active_personas {
                    ShipChecklistItemStatus::Pass
                } else {
                    ShipChecklistItemStatus::Fail
                },
                format!(
                    "all_three_active={};no_extra_active_personas={}",
                    trinity_report.all_three_active, trinity_report.no_extra_active_personas
                ),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "candidate-not-order",
                "Candidate lifecycle forbids real order",
                "Candidate/Paper states must never imply broker execution.",
                if acceptance_report.no_real_order_path_detected {
                    ShipChecklistItemStatus::Pass
                } else {
                    ShipChecklistItemStatus::Fail
                },
                format!(
                    "candidate_generated={};no_real_order_path_detected={}",
                    acceptance_report.candidate_generated,
                    acceptance_report.no_real_order_path_detected
                ),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "paper-simulated-only",
                "Paper lifecycle simulated only",
                "PaperPositionOpen must remain simulated only.",
                if acceptance_report.paper_position_simulated {
                    ShipChecklistItemStatus::Pass
                } else {
                    ShipChecklistItemStatus::Fail
                },
                format!(
                    "paper_position_simulated={}",
                    acceptance_report.paper_position_simulated
                ),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "owner-cannot-bypass-risk",
                "Owner cannot bypass Risk",
                "Owner workflow remains read-only/paper-only under veto.",
                if chair_report.no_bypass_detected {
                    ShipChecklistItemStatus::Pass
                } else {
                    ShipChecklistItemStatus::Fail
                },
                format!("no_bypass_detected={}", chair_report.no_bypass_detected),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "riskblocked-cannot-paper-approve",
                "RiskBlocked cannot paper approve",
                "Denied candidates must not transition to paper approval.",
                if acceptance_report.risk_block_test_passed {
                    ShipChecklistItemStatus::Pass
                } else {
                    ShipChecklistItemStatus::Fail
                },
                format!(
                    "risk_block_test_passed={}",
                    acceptance_report.risk_block_test_passed
                ),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "notrade-cannot-paper-approve",
                "NoTrade cannot paper approve",
                "NoTrade remains final for the current cycle.",
                if acceptance_report.no_trade_test_passed {
                    ShipChecklistItemStatus::Pass
                } else {
                    ShipChecklistItemStatus::PassWithWarning
                },
                format!(
                    "no_trade_test_passed={}",
                    acceptance_report.no_trade_test_passed
                ),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "artifact-diff-acceptable",
                "Deterministic artifact diff acceptable",
                "Unexpected diff blocks ship gate unless manually accepted.",
                match diff_report.map(|report| report.diff_status) {
                    Some(ArtifactDiffStatus::NoDiff) => ShipChecklistItemStatus::Pass,
                    Some(ArtifactDiffStatus::ExpectedDiff) => {
                        ShipChecklistItemStatus::PassWithWarning
                    }
                    Some(ArtifactDiffStatus::DiagnosticOnly) => ShipChecklistItemStatus::Deferred,
                    Some(_) => ShipChecklistItemStatus::Fail,
                    None => {
                        if config.require_determinism {
                            ShipChecklistItemStatus::Deferred
                        } else {
                            ShipChecklistItemStatus::NotApplicable
                        }
                    }
                },
                diff_report
                    .map(|report| format!("{:?}", report.diff_status))
                    .unwrap_or_else(|| "not configured".to_string()),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "runbook-local-only",
                "Runbook emits local-only commands",
                "Ship review requires local-only CLI guidance.",
                if artifacts.operational_runbook_value.is_some() {
                    ShipChecklistItemStatus::Pass
                } else {
                    ShipChecklistItemStatus::Fail
                },
                if artifacts.operational_runbook_value.is_some() {
                    "operational runbook artifact present".to_string()
                } else {
                    "missing operational runbook artifact".to_string()
                },
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "no-runtime-llm",
                "No runtime LLM",
                "Runtime LLM remains forbidden.",
                ShipChecklistItemStatus::Pass,
                "no runtime LLM path added in Sprint 59 review stack".to_string(),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "no-mamba-runtime",
                "No Mamba runtime",
                "Mamba runtime remains deferred/forbidden.",
                if artifacts
                    .core_completion_value
                    .as_ref()
                    .and_then(|value| lookup_bool(value, "mamba_runtime_present"))
                    .unwrap_or(false)
                {
                    ShipChecklistItemStatus::Fail
                } else {
                    ShipChecklistItemStatus::Pass
                },
                "mamba_runtime_present=false".to_string(),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
            checklist_item(
                "no-6-12-expansion",
                "No 6/12 expansion",
                "Exactly three active personas remain in the Trinity loop.",
                if trinity_report.all_three_active && trinity_report.no_extra_active_personas {
                    ShipChecklistItemStatus::Pass
                } else {
                    ShipChecklistItemStatus::Fail
                },
                format!("active_member_count={}", trinity_report.active_member_count),
                true,
                vec![ReasonCode::ManualShipChecklistBuilt],
            ),
        ],
        pass_count: 0,
        warning_count: 0,
        fail_count: 0,
        deferred_count: 0,
        all_required_passed: false,
        overall_status: ShipChecklistOverallStatus::Failed,
        reason_codes: vec![ReasonCode::ManualShipChecklistBuilt],
    };
    checklist.stabilize();
    checklist
}

fn build_system_ship_gate_report(
    config: &SystemIntegrationReviewConfig,
    matrix: &CoreUiChairCommitteeReadinessMatrix,
    chair_report: &ChairOperationalReadinessReport,
    trinity_report: &TrinityCommitteeReadinessReport,
    ui_report: &ControlTowerUiReadinessReport,
    acceptance_report: &EndToEndPaperLoopAcceptanceReport,
    diff_report: Option<&DeterministicArtifactDiffReport>,
    checklist: &ManualShipAcceptanceChecklist,
) -> SystemShipGateReport {
    let mut hard_blockers = Vec::new();
    let mut warnings = Vec::new();
    let artifact_diff_status = diff_report
        .map(|report| report.diff_status)
        .unwrap_or(ArtifactDiffStatus::DiagnosticOnly);

    if !ui_report.no_secret_values || !ui_report.no_order_buttons || !ui_report.no_account_panels {
        hard_blockers.push("UI safety gate failed".to_string());
    }
    if !acceptance_report.no_real_order_path_detected || !acceptance_report.no_broker_path_detected
    {
        hard_blockers.push("paper loop safety boundary failed".to_string());
    }
    if config.require_risk_veto && !acceptance_report.risk_block_test_passed {
        hard_blockers.push("risk veto proof is missing".to_string());
    }
    if checklist.overall_status == ShipChecklistOverallStatus::PassedWithWarnings {
        warnings.push("manual checklist still carries warnings".to_string());
    }
    warnings.extend(
        matrix
            .rows
            .iter()
            .flat_map(|row| row.warnings.clone())
            .collect::<Vec<_>>(),
    );
    let (final_status, final_recommendation) = if !hard_blockers.is_empty() {
        (
            SystemShipGateStatus::BlockedBySafety,
            SystemShipGateRecommendation::FixRiskOwnerGaps,
        )
    } else if matches!(
        artifact_diff_status,
        ArtifactDiffStatus::UnexpectedDiff
            | ArtifactDiffStatus::MissingBaseline
            | ArtifactDiffStatus::MissingCurrent
    ) {
        (
            SystemShipGateStatus::BlockedByDeterminism,
            SystemShipGateRecommendation::FixDeterminismGaps,
        )
    } else if matches!(
        ui_report.readiness_status,
        ControlTowerUiReadinessStatus::Blocked | ControlTowerUiReadinessStatus::NeedsHardening
    ) {
        (
            SystemShipGateStatus::BlockedByMissingUI,
            SystemShipGateRecommendation::FixControlTowerGaps,
        )
    } else if matches!(
        chair_report.readiness_status,
        ChairOperationalReadinessStatus::Blocked | ChairOperationalReadinessStatus::NeedsHardening
    ) || matches!(
        trinity_report.readiness_status,
        TrinityCommitteeReadinessStatus::Blocked | TrinityCommitteeReadinessStatus::NeedsHardening
    ) {
        (
            SystemShipGateStatus::BlockedByChairCommitteeGap,
            SystemShipGateRecommendation::FixChairCommitteeGaps,
        )
    } else if matches!(
        matrix
            .rows
            .iter()
            .find(|row| row.area == ReadinessArea::EvidenceDepth)
            .map(|row| row.status),
        Some(ReadinessStatus::NeedsHardening | ReadinessStatus::Blocked)
    ) || acceptance_report.acceptance_status == EndToEndPaperLoopAcceptanceStatus::Failed
    {
        (
            SystemShipGateStatus::BlockedByEvidenceGap,
            SystemShipGateRecommendation::NeedMoreKISEvidence,
        )
    } else if checklist.all_required_passed
        && matrix.overall_status == ReadinessMatrixOverallStatus::ReadyForPaperOpsMonitoring
        && acceptance_report.acceptance_status == EndToEndPaperLoopAcceptanceStatus::Passed
        && matches!(
            artifact_diff_status,
            ArtifactDiffStatus::NoDiff | ArtifactDiffStatus::DiagnosticOnly
        )
    {
        (
            SystemShipGateStatus::ReadyToShipPaperOpsMonitoring,
            SystemShipGateRecommendation::ShipPaperOpsMonitoring,
        )
    } else if checklist.all_required_passed {
        (
            SystemShipGateStatus::ReadyWithManualWarnings,
            SystemShipGateRecommendation::ReviewWarningsManually,
        )
    } else {
        (
            SystemShipGateStatus::HoldShipAndFixGaps,
            SystemShipGateRecommendation::FixCoreGaps,
        )
    };

    SystemShipGateReport {
        gate_id: format!("{}-ship-gate", config.review_id),
        readiness_matrix_status: matrix.overall_status,
        paper_loop_acceptance_status: acceptance_report.acceptance_status,
        artifact_diff_status,
        checklist_status: checklist.overall_status,
        hard_blockers: stable_ordered_strings(&hard_blockers),
        warnings: stable_ordered_strings(&warnings),
        final_status,
        final_recommendation,
        reason_codes: stable_reason_codes(&[ReasonCode::SystemShipGateBuilt]),
    }
}

fn ship_gate_row(report: &SystemShipGateReport) -> ReadinessMatrixRow {
    row(
        ReadinessArea::ShipGate,
        match report.final_status {
            SystemShipGateStatus::ReadyToShipPaperOpsMonitoring => ReadinessStatus::Ready,
            SystemShipGateStatus::ReadyWithManualWarnings => ReadinessStatus::ReadyWithWarnings,
            SystemShipGateStatus::BlockedBySafety
            | SystemShipGateStatus::BlockedByDeterminism
            | SystemShipGateStatus::BlockedByMissingUI
            | SystemShipGateStatus::BlockedByChairCommitteeGap
            | SystemShipGateStatus::BlockedByEvidenceGap => ReadinessStatus::Blocked,
            SystemShipGateStatus::HoldShipAndFixGaps => ReadinessStatus::NeedsHardening,
        },
        format!("{:?}", report.final_recommendation),
        report.hard_blockers.clone(),
        report.warnings.clone(),
        "resolve ship blockers before paper-ops monitoring",
        vec![ReasonCode::SystemShipGateBuilt],
    )
}

fn build_storage_report(
    config: &SystemIntegrationReviewConfig,
    input_paths: &[String],
) -> SystemReviewStorageReport {
    let mut sizes = input_paths
        .iter()
        .filter_map(|path| {
            fs::metadata(path)
                .ok()
                .map(|meta| (meta.len() as usize, path.clone()))
        })
        .collect::<Vec<_>>();
    sizes.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    let total_bytes = sizes.iter().map(|(bytes, _)| *bytes).sum::<usize>();
    SystemReviewStorageReport {
        artifact_count: input_paths.len(),
        total_bytes,
        max_bytes: config.max_bytes,
        budget_exceeded: total_bytes > config.max_bytes,
        largest_artifacts: sizes
            .into_iter()
            .take(5)
            .map(|(bytes, path)| format!("{path}:{bytes}"))
            .collect(),
        reason_codes: stable_reason_codes(&[ReasonCode::SystemReviewStorageReportBuilt]),
    }
}

fn build_final_summary(
    config: &SystemIntegrationReviewConfig,
    matrix: &CoreUiChairCommitteeReadinessMatrix,
    chair_report: &ChairOperationalReadinessReport,
    trinity_report: &TrinityCommitteeReadinessReport,
    ui_report: &ControlTowerUiReadinessReport,
    acceptance_report: &EndToEndPaperLoopAcceptanceReport,
    diff_report: Option<&DeterministicArtifactDiffReport>,
    checklist: &ManualShipAcceptanceChecklist,
    gate_report: &SystemShipGateReport,
) -> String {
    [
        "paper_only_warning=system review is for paper ops monitoring only".to_string(),
        "no_live_warning=ship gate never implies live trading or profitability".to_string(),
        format!("review_id={}", config.review_id),
        format!("matrix_overall_status={:?}", matrix.overall_status),
        format!("chair_readiness_status={:?}", chair_report.readiness_status),
        format!(
            "trinity_readiness_status={:?}",
            trinity_report.readiness_status
        ),
        format!("ui_readiness_status={:?}", ui_report.readiness_status),
        format!(
            "paper_loop_acceptance_status={:?}",
            acceptance_report.acceptance_status
        ),
        format!(
            "artifact_diff_status={:?}",
            diff_report
                .map(|report| report.diff_status)
                .unwrap_or(ArtifactDiffStatus::DiagnosticOnly)
        ),
        format!("checklist_status={:?}", checklist.overall_status),
        format!("ship_gate_status={:?}", gate_report.final_status),
        format!(
            "ship_gate_recommendation={:?}",
            gate_report.final_recommendation
        ),
    ]
    .join("\n")
}

fn run_secret_redaction_audit(
    config: &SystemIntegrationReviewConfig,
    artifact_paths: &[String],
) -> Result<SecretRedactionAuditReport, String> {
    if artifact_paths.is_empty() {
        return Ok(SecretRedactionAuditReport {
            audit_id: format!("{}-input-audit", config.review_id),
            artifacts_scanned: 0,
            secret_leaks_detected: false,
            token_like_values_detected: false,
            account_like_fields_detected: false,
            order_like_fields_detected: false,
            redaction_status: SecretRedactionStatus::DiagnosticOnly,
            reason_codes: stable_reason_codes(&[
                ReasonCode::SecretRedactionAuditBuilt,
                ReasonCode::DeterministicPath,
            ]),
        });
    }
    SecretRedactionAuditRunner::default().run(&SecretRedactionAuditConfig {
        audit_id: format!("{}-input-audit", config.review_id),
        artifact_paths: artifact_paths.to_vec(),
        output_root: config.output_root.clone(),
        ..SecretRedactionAuditConfig::default()
    })
}

fn checklist_item(
    item_id: &str,
    title: &str,
    description: &str,
    status: ShipChecklistItemStatus,
    evidence: String,
    required_for_ship: bool,
    reason_codes: Vec<ReasonCode>,
) -> ShipChecklistItem {
    ShipChecklistItem {
        item_id: item_id.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        status,
        evidence,
        required_for_ship,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn row(
    area: ReadinessArea,
    status: ReadinessStatus,
    evidence_summary: impl Into<String>,
    blockers: Vec<String>,
    warnings: Vec<String>,
    next_action: impl Into<String>,
    reason_codes: Vec<ReasonCode>,
) -> ReadinessMatrixRow {
    ReadinessMatrixRow {
        area,
        status,
        evidence_summary: evidence_summary.into(),
        blockers: stable_ordered_strings(&blockers),
        warnings: stable_ordered_strings(&warnings),
        next_action: next_action.into(),
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn map_ui_status(status: ControlTowerUiReadinessStatus) -> ReadinessStatus {
    match status {
        ControlTowerUiReadinessStatus::Ready => ReadinessStatus::Ready,
        ControlTowerUiReadinessStatus::ReadyWithWarnings => ReadinessStatus::ReadyWithWarnings,
        ControlTowerUiReadinessStatus::NeedsHardening => ReadinessStatus::NeedsHardening,
        ControlTowerUiReadinessStatus::Blocked => ReadinessStatus::Blocked,
    }
}

fn map_chair_status(status: ChairOperationalReadinessStatus) -> ReadinessStatus {
    match status {
        ChairOperationalReadinessStatus::Ready => ReadinessStatus::Ready,
        ChairOperationalReadinessStatus::ReadyWithWarnings => ReadinessStatus::ReadyWithWarnings,
        ChairOperationalReadinessStatus::NeedsHardening => ReadinessStatus::NeedsHardening,
        ChairOperationalReadinessStatus::Blocked => ReadinessStatus::Blocked,
    }
}

fn map_trinity_status(status: TrinityCommitteeReadinessStatus) -> ReadinessStatus {
    match status {
        TrinityCommitteeReadinessStatus::Ready => ReadinessStatus::Ready,
        TrinityCommitteeReadinessStatus::ReadyWithWarnings => ReadinessStatus::ReadyWithWarnings,
        TrinityCommitteeReadinessStatus::NeedsHardening => ReadinessStatus::NeedsHardening,
        TrinityCommitteeReadinessStatus::Blocked => ReadinessStatus::Blocked,
    }
}

fn map_diff_status(status: Option<ArtifactDiffStatus>, required: bool) -> ReadinessStatus {
    match status {
        Some(ArtifactDiffStatus::NoDiff) => ReadinessStatus::Ready,
        Some(ArtifactDiffStatus::ExpectedDiff) => ReadinessStatus::ReadyWithWarnings,
        Some(ArtifactDiffStatus::UnexpectedDiff)
        | Some(ArtifactDiffStatus::MissingBaseline)
        | Some(ArtifactDiffStatus::MissingCurrent) => ReadinessStatus::Blocked,
        Some(ArtifactDiffStatus::DiagnosticOnly) => {
            if required {
                ReadinessStatus::Deferred
            } else {
                ReadinessStatus::Deferred
            }
        }
        None => {
            if required {
                ReadinessStatus::Deferred
            } else {
                ReadinessStatus::Deferred
            }
        }
    }
}

fn load_latest_json(paths: &[String]) -> Result<Option<Value>, String> {
    let mut latest = None;
    for path in stable_ordered_strings(paths) {
        if !Path::new(&path).is_file() {
            continue;
        }
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        latest = Some(serde_json::from_str(&text).map_err(|err| err.to_string())?);
    }
    Ok(latest)
}

fn load_latest_text(paths: &[String]) -> Result<Option<String>, String> {
    let mut latest = None;
    for path in stable_ordered_strings(paths) {
        if !Path::new(&path).is_file() {
            continue;
        }
        latest = Some(fs::read_to_string(path).map_err(|err| err.to_string())?);
    }
    Ok(latest)
}

fn normalize_artifact(text: &str, ignore_fields: &[String]) -> String {
    match serde_json::from_str::<Value>(text) {
        Ok(mut value) => {
            strip_ignored_fields(&mut value, ignore_fields);
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| text.replace("\r\n", "\n"))
        }
        Err(_) => text.replace("\r\n", "\n"),
    }
}

fn strip_ignored_fields(value: &mut Value, ignore_fields: &[String]) {
    match value {
        Value::Object(map) => {
            for field in ignore_fields {
                map.remove(field);
            }
            for nested in map.values_mut() {
                strip_ignored_fields(nested, ignore_fields);
            }
        }
        Value::Array(items) => {
            for item in items {
                strip_ignored_fields(item, ignore_fields);
            }
        }
        _ => {}
    }
}

fn path_is_ignored(baseline: &str, current: &str, ignore_paths: &[String]) -> bool {
    ignore_paths
        .iter()
        .any(|item| baseline.contains(item) || current.contains(item))
}

fn contains_unsafe_button(html_lower: &str) -> bool {
    ["<button", "role=\"button\"", "type=\"submit\""]
        .iter()
        .any(|needle| html_lower.contains(needle))
        && ["order", "trade", "buy", "sell", "account"]
            .iter()
            .any(|needle| html_lower.contains(needle))
}

fn has_non_paper_order_plan(value: &Value) -> bool {
    find_value(value, "approved_order_plan")
        .and_then(|item| find_value(item, "paper_only"))
        .and_then(Value::as_bool)
        .is_some_and(|paper_only| !paper_only)
}

fn has_key_opt(value: Option<&Value>, key: &str) -> bool {
    value.is_some_and(|item| has_key(item, key))
}

fn has_key(value: &Value, key: &str) -> bool {
    find_value(value, key).is_some()
}

fn lookup_bool(value: &Value, key: &str) -> Option<bool> {
    find_value(value, key).and_then(Value::as_bool)
}

fn lookup_usize(value: &Value, key: &str) -> Option<usize> {
    find_value(value, key).and_then(|item| match item {
        Value::Number(number) => number.as_u64().map(|value| value as usize),
        Value::String(text) => text.parse::<usize>().ok(),
        _ => None,
    })
}

fn lookup_f64(value: &Value, key: &str) -> Option<f64> {
    find_value(value, key).and_then(|item| match item {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    })
}

fn lookup_string(value: &Value, key: &str) -> Option<String> {
    find_value(value, key).and_then(|item| match item {
        Value::String(text) => Some(text.clone()),
        Value::Bool(flag) => Some(flag.to_string()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    })
}

fn lookup_string_opt(value: Option<&Value>, key: &str) -> Option<String> {
    value.and_then(|item| lookup_string(item, key))
}

fn lookup_array_len(value: &Value, key: &str) -> Option<usize> {
    find_value(value, key)
        .and_then(Value::as_array)
        .map(Vec::len)
}

fn value_text(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

fn find_value<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    match value {
        Value::Object(map) => {
            if let Some(found) = map.get(key) {
                return Some(found);
            }
            map.values().find_map(|item| find_value(item, key))
        }
        Value::Array(items) => items.iter().find_map(|item| find_value(item, key)),
        _ => None,
    }
}
