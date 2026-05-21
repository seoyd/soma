use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_ordered_strings, stable_reason_codes};
use crate::model::{SequenceDatasetReadinessReport, SequenceDatasetReadinessStatus};
use crate::owner::{AllowedOwnerAction, OwnerReviewItem, OwnerReviewQueue};

fn default_output_root() -> String {
    "target/sprint60/evidence_hardening".to_string()
}

fn default_max_artifacts() -> usize {
    64
}

fn default_max_bytes() -> usize {
    20_000_000
}

fn default_min_official_rows() -> usize {
    16
}

fn default_min_complete_rows() -> usize {
    4
}

fn default_min_outcome_links() -> usize {
    4
}

fn default_min_no_trade_counterfactuals() -> usize {
    2
}

fn default_min_risk_denied_counterfactuals() -> usize {
    1
}

fn default_min_outcome_diversity() -> usize {
    3
}

fn default_min_symbol_diversity() -> usize {
    4
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceHardeningConfig {
    pub report_id: String,
    #[serde(default)]
    pub kis_smoke_report_paths: Vec<String>,
    #[serde(default)]
    pub system_review_bundle_paths: Vec<String>,
    #[serde(default)]
    pub control_tower_state_paths: Vec<String>,
    #[serde(default)]
    pub control_tower_html_paths: Vec<String>,
    #[serde(default)]
    pub owner_review_queue_paths: Vec<String>,
    #[serde(default)]
    pub sequence_readiness_report_paths: Vec<String>,
    #[serde(default)]
    pub mamba_readiness_snapshot_paths: Vec<String>,
    #[serde(default)]
    pub operational_runbook_paths: Vec<String>,
    #[serde(default)]
    pub supporting_artifact_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_artifacts")]
    pub max_artifacts: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_min_official_rows")]
    pub min_official_rows: usize,
    #[serde(default = "default_min_complete_rows")]
    pub min_complete_rows: usize,
    #[serde(default = "default_min_outcome_links")]
    pub min_outcome_links: usize,
    #[serde(default = "default_min_no_trade_counterfactuals")]
    pub min_no_trade_counterfactuals: usize,
    #[serde(default = "default_min_risk_denied_counterfactuals")]
    pub min_risk_denied_counterfactuals: usize,
    #[serde(default = "default_min_outcome_diversity")]
    pub min_outcome_diversity: usize,
    #[serde(default = "default_min_symbol_diversity")]
    pub min_symbol_diversity: usize,
    #[serde(default = "default_true")]
    pub require_future_windows: bool,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for EvidenceHardeningConfig {
    fn default() -> Self {
        Self {
            report_id: "sprint60-evidence-hardening".to_string(),
            kis_smoke_report_paths: Vec::new(),
            system_review_bundle_paths: Vec::new(),
            control_tower_state_paths: Vec::new(),
            control_tower_html_paths: Vec::new(),
            owner_review_queue_paths: Vec::new(),
            sequence_readiness_report_paths: Vec::new(),
            mamba_readiness_snapshot_paths: Vec::new(),
            operational_runbook_paths: Vec::new(),
            supporting_artifact_paths: Vec::new(),
            output_root: default_output_root(),
            max_artifacts: default_max_artifacts(),
            max_bytes: default_max_bytes(),
            min_official_rows: default_min_official_rows(),
            min_complete_rows: default_min_complete_rows(),
            min_outcome_links: default_min_outcome_links(),
            min_no_trade_counterfactuals: default_min_no_trade_counterfactuals(),
            min_risk_denied_counterfactuals: default_min_risk_denied_counterfactuals(),
            min_outcome_diversity: default_min_outcome_diversity(),
            min_symbol_diversity: default_min_symbol_diversity(),
            require_future_windows: true,
            require_no_lookahead_safe: true,
            reason_codes: vec![ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly],
        }
    }
}

impl EvidenceHardeningConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.report_id.trim().is_empty() {
            return Err("evidence hardening report id must not be empty".to_string());
        }
        if self.max_artifacts == 0 || self.max_artifacts > 256 {
            return Err("evidence hardening max_artifacts must be between 1 and 256".to_string());
        }
        if self.max_bytes == 0 || self.max_bytes > 50_000_000 {
            return Err("evidence hardening max_bytes must be between 1 and 50000000".to_string());
        }
        for threshold in [
            self.min_official_rows,
            self.min_complete_rows,
            self.min_outcome_links,
            self.min_no_trade_counterfactuals,
            self.min_risk_denied_counterfactuals,
            self.min_outcome_diversity,
            self.min_symbol_diversity,
        ] {
            if threshold == 0 {
                return Err("evidence hardening thresholds must be positive".to_string());
            }
        }
        let all_paths = self.all_paths();
        if all_paths.iter().any(|path| path.contains("://")) {
            return Err("evidence hardening paths must be local".to_string());
        }
        if all_paths.len() > self.max_artifacts {
            return Err("evidence hardening artifact count exceeds max_artifacts".to_string());
        }
        let total_bytes = all_paths
            .iter()
            .filter_map(|path| fs::metadata(path).ok().map(|meta| meta.len() as usize))
            .sum::<usize>();
        if total_bytes > self.max_bytes {
            return Err("evidence hardening input bytes exceed max_bytes".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.report_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        stable_ordered_strings(
            &self
                .kis_smoke_report_paths
                .iter()
                .chain(self.system_review_bundle_paths.iter())
                .chain(self.control_tower_state_paths.iter())
                .chain(self.control_tower_html_paths.iter())
                .chain(self.owner_review_queue_paths.iter())
                .chain(self.sequence_readiness_report_paths.iter())
                .chain(self.mamba_readiness_snapshot_paths.iter())
                .chain(self.operational_runbook_paths.iter())
                .chain(self.supporting_artifact_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceGapPrimaryGap {
    Healthy,
    NeedMoreKISEvidence,
    NeedOutcomeLinkDepth,
    NeedCounterfactualDepth,
    NeedOutcomeDiversity,
    NeedOwnerReview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceDepthGapReport {
    pub report_id: String,
    pub official_rows: usize,
    pub complete_rows: usize,
    pub outcome_links: usize,
    pub no_trade_counterfactuals: usize,
    pub risk_denied_counterfactuals: usize,
    pub outcome_diversity_count: usize,
    pub symbol_diversity_count: usize,
    pub primary_gap: EvidenceGapPrimaryGap,
    #[serde(default)]
    pub next_evidence_actions: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeLinkCoverageStatus {
    Healthy,
    NeedOutcomeLinkDepth,
    NeedFutureWindows,
    BlockedByNoLookahead,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeLinkCoverageReport {
    pub report_id: String,
    pub outcome_link_count: usize,
    pub future_window_count: usize,
    pub no_lookahead_safe: bool,
    pub take_profit_count: usize,
    pub stop_loss_count: usize,
    pub time_expired_count: usize,
    pub coverage_status: OutcomeLinkCoverageStatus,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CounterfactualCoverageStatus {
    Healthy,
    NeedCounterfactualDepth,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CounterfactualCoverageReport {
    pub report_id: String,
    pub no_trade_depth: usize,
    pub risk_denied_depth: usize,
    pub missing_risk_decision_count: usize,
    pub missing_committee_decision_count: usize,
    pub avoided_loss_total: f64,
    pub missed_gain_total: f64,
    pub coverage_status: CounterfactualCoverageStatus,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManualReviewErgonomicsStatus {
    Improved,
    NeedsBetterOwnerDiscipline,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualReviewErgonomicsReport {
    pub report_id: String,
    pub pending_review_count: usize,
    pub risk_blocked_count: usize,
    pub paper_confirm_allowed_count: usize,
    pub paper_confirm_blocked_count: usize,
    pub missing_reason_count: usize,
    pub unclear_action_count: usize,
    pub ergonomics_status: ManualReviewErgonomicsStatus,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatorReviewPrimaryAction {
    ReviewPendingQueue,
    InspectRiskBlockedQueue,
    ArchiveStablePaperQueue,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorReviewWorkflowV2 {
    pub workflow_id: String,
    pub primary_action: OperatorReviewPrimaryAction,
    #[serde(default)]
    pub queued_actions: Vec<String>,
    #[serde(default)]
    pub copyable_commands: Vec<String>,
    #[serde(default)]
    pub blocked_actions: Vec<String>,
    pub paper_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateCardStatusV1_5 {
    Official,
    ResearchOnly,
    DiagnosticOnly,
    RiskBlocked,
    NoTrade,
    HumanConfirmRequired,
    PaperApproved,
    PaperOpen,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateCardV1_5 {
    pub candidate_id: String,
    pub symbol: String,
    pub status: CandidateCardStatusV1_5,
    pub evidence_summary: String,
    pub review_summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceWarningBadge {
    NeedMoreKISEvidence,
    NeedOutcomeLinkDepth,
    NeedCounterfactualDepth,
    NeedOutcomeDiversity,
    NeedOwnerReview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MambaDeferredBannerStatus {
    Mamba3RuntimeDeferred,
    BuildSequenceDatasetFirst,
    ExternalPrototypeOnlyIfReady,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlTowerErgonomicsStatus {
    Ready,
    NeedsHardening,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerErgonomicsV1_5Report {
    pub report_id: String,
    #[serde(default)]
    pub evidence_badges: Vec<EvidenceWarningBadge>,
    #[serde(default)]
    pub candidate_cards: Vec<CandidateCardV1_5>,
    pub mamba_banner_status: MambaDeferredBannerStatus,
    #[serde(default)]
    pub copyable_commands: Vec<String>,
    pub no_execution_buttons: bool,
    pub no_account_controls: bool,
    pub render_status: ControlTowerErgonomicsStatus,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UIFrameworkCurrentChoice {
    StaticHtmlJsonTxt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UIFrameworkOptionalChoice {
    VanillaTypeScriptEnhancement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UIFrameworkFutureChoice {
    TauriSvelteDesktop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UIFrameworkRejectedOption {
    ReactNextWeb,
    CloudDashboard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UIFrameworkDecisionStatus {
    KeepStaticDashboardNow,
    PlanTauriSvelteLater,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UIFrameworkDecisionReport {
    pub report_id: String,
    pub current_choice: UIFrameworkCurrentChoice,
    pub optional_choice: UIFrameworkOptionalChoice,
    pub future_choice: UIFrameworkFutureChoice,
    #[serde(default)]
    pub rejected_options: Vec<UIFrameworkRejectedOption>,
    pub decision_status: UIFrameworkDecisionStatus,
    #[serde(default)]
    pub rationale: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3ApplicationStage {
    Deferred,
    ExternalPrototypeOnlyIfReady,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3ApplicationTimingDecision {
    HoldMamba3Deferred,
    BuildSequenceDatasetFirst,
    ExternalPrototypeOnlyIfReady,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mamba3ApplicationTimingReport {
    pub report_id: String,
    pub current_stage: Mamba3ApplicationStage,
    pub evidence_gate_passed: bool,
    pub sequence_gate_passed: bool,
    pub runtime_deferred: bool,
    pub rust_runtime_allowed: bool,
    pub external_prototype_allowed: bool,
    pub final_decision: Mamba3ApplicationTimingDecision,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceHardeningRecommendation {
    EvidenceHardeningReady,
    ReviewErgonomicsImproved,
    ControlTowerV1_5Ready,
    KeepStaticDashboardNow,
    PlanTauriSvelteLater,
    HoldMamba3Deferred,
    BuildSequenceDatasetFirst,
    NeedMoreKISEvidence,
    NeedOutcomeLinkDepth,
    NeedCounterfactualDepth,
    NeedOutcomeDiversity,
    NeedOwnerReview,
    KeepTrinity,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceHardeningBundle {
    pub evidence_depth_gap_report: EvidenceDepthGapReport,
    pub outcome_link_coverage_report: OutcomeLinkCoverageReport,
    pub counterfactual_coverage_report: CounterfactualCoverageReport,
    pub manual_review_ergonomics_report: ManualReviewErgonomicsReport,
    pub operator_review_workflow_v2: OperatorReviewWorkflowV2,
    pub control_tower_ergonomics_v1_5_report: ControlTowerErgonomicsV1_5Report,
    pub ui_framework_decision_report: UIFrameworkDecisionReport,
    pub mamba3_application_timing_report: Mamba3ApplicationTimingReport,
    #[serde(default)]
    pub final_recommendations: Vec<EvidenceHardeningRecommendation>,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EvidenceHardeningRunner;

#[derive(Clone, Debug, Default)]
struct HardeningArtifacts {
    kis_smoke_value: Option<Value>,
    system_review_value: Option<Value>,
    control_tower_value: Option<Value>,
    owner_queue: Option<OwnerReviewQueue>,
    sequence_readiness: Option<SequenceDatasetReadinessReport>,
    mamba_snapshot_value: Option<Value>,
    operational_runbook_value: Option<Value>,
    supporting_values: Vec<Value>,
}

impl HardeningArtifacts {
    fn load(config: &EvidenceHardeningConfig) -> Result<Self, String> {
        Ok(Self {
            kis_smoke_value: load_latest_json(&config.kis_smoke_report_paths)?,
            system_review_value: load_latest_json(&config.system_review_bundle_paths)?,
            control_tower_value: load_latest_json(&config.control_tower_state_paths)?,
            owner_queue: load_latest_owner_queue(&config.owner_review_queue_paths)?,
            sequence_readiness: load_latest_typed(&config.sequence_readiness_report_paths)?,
            mamba_snapshot_value: load_latest_json(&config.mamba_readiness_snapshot_paths)?,
            operational_runbook_value: load_latest_json(&config.operational_runbook_paths)?,
            supporting_values: load_json_values(&config.supporting_artifact_paths)?,
        })
    }

    fn json_values(&self) -> Vec<Value> {
        let mut values = Vec::new();
        values.extend(self.kis_smoke_value.clone());
        values.extend(self.system_review_value.clone());
        values.extend(self.control_tower_value.clone());
        if let Some(queue) = &self.owner_queue {
            values.push(serde_json::to_value(queue).unwrap_or(Value::Null));
        }
        if let Some(report) = &self.sequence_readiness {
            values.push(serde_json::to_value(report).unwrap_or(Value::Null));
        }
        values.extend(self.mamba_snapshot_value.clone());
        values.extend(self.operational_runbook_value.clone());
        values.extend(self.supporting_values.clone());
        values
    }
}

impl EvidenceHardeningBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            "research_only_warning=evidence hardening remains local-only, paper-only, and deterministic".to_string(),
            format!(
                "primary_gap={:?}",
                self.evidence_depth_gap_report.primary_gap
            ),
            format!(
                "outcome_link_status={:?}",
                self.outcome_link_coverage_report.coverage_status
            ),
            format!(
                "counterfactual_status={:?}",
                self.counterfactual_coverage_report.coverage_status
            ),
            format!(
                "review_ergonomics_status={:?}",
                self.manual_review_ergonomics_report.ergonomics_status
            ),
            format!(
                "ui_framework_status={:?}",
                self.ui_framework_decision_report.decision_status
            ),
            format!(
                "mamba_timing_decision={:?}",
                self.mamba3_application_timing_report.final_decision
            ),
            format!(
                "final_recommendations={}",
                self.final_recommendations
                    .iter()
                    .map(|item| format!("{item:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            self.final_summary.clone(),
        ]
        .join("\n")
    }
}

impl EvidenceHardeningRunner {
    pub fn run(&self, config: &EvidenceHardeningConfig) -> Result<EvidenceHardeningBundle, String> {
        config.validate()?;
        let artifacts = HardeningArtifacts::load(config)?;
        let json_values = artifacts.json_values();
        let evidence_depth_gap_report =
            build_evidence_depth_gap_report(config, &artifacts, &json_values);
        let outcome_link_coverage_report = build_outcome_link_coverage_report(config, &json_values);
        let counterfactual_coverage_report =
            build_counterfactual_coverage_report(config, &json_values);
        let manual_review_ergonomics_report =
            build_manual_review_ergonomics_report(config, artifacts.owner_queue.as_ref());
        let operator_review_workflow_v2 =
            build_operator_review_workflow_v2(config, artifacts.owner_queue.as_ref(), &json_values);
        let control_tower_ergonomics_v1_5_report = build_control_tower_ergonomics_report(
            config,
            artifacts.control_tower_value.as_ref(),
            &artifacts.owner_queue,
            &evidence_depth_gap_report,
            &manual_review_ergonomics_report,
        );
        let ui_framework_decision_report = build_ui_framework_decision_report(config);
        let mamba3_application_timing_report = build_mamba_application_timing_report(
            config,
            &evidence_depth_gap_report,
            &outcome_link_coverage_report,
            &counterfactual_coverage_report,
            artifacts.sequence_readiness.as_ref(),
            artifacts.mamba_snapshot_value.as_ref(),
        );
        let final_recommendations = build_final_recommendations(
            &evidence_depth_gap_report,
            &manual_review_ergonomics_report,
            &control_tower_ergonomics_v1_5_report,
            &ui_framework_decision_report,
            &mamba3_application_timing_report,
        );
        let final_summary = build_final_summary(
            &evidence_depth_gap_report,
            &manual_review_ergonomics_report,
            &ui_framework_decision_report,
            &mamba3_application_timing_report,
            &final_recommendations,
        );
        let bundle = EvidenceHardeningBundle {
            evidence_depth_gap_report,
            outcome_link_coverage_report,
            counterfactual_coverage_report,
            manual_review_ergonomics_report,
            operator_review_workflow_v2,
            control_tower_ergonomics_v1_5_report,
            ui_framework_decision_report,
            mamba3_application_timing_report,
            final_recommendations,
            final_summary,
            reason_codes: stable_reason_codes(&[
                ReasonCode::EvidenceHardeningBuilt,
                ReasonCode::EvidenceHardeningSummaryBuilt,
            ]),
        };
        write_bundle_outputs(config, &bundle)?;
        Ok(bundle)
    }
}

fn build_evidence_depth_gap_report(
    config: &EvidenceHardeningConfig,
    artifacts: &HardeningArtifacts,
    values: &[Value],
) -> EvidenceDepthGapReport {
    let mut evidence_values = Vec::new();
    evidence_values.extend(artifacts.kis_smoke_value.clone());
    evidence_values.extend(artifacts.control_tower_value.clone());
    evidence_values.extend(artifacts.system_review_value.clone());
    evidence_values.extend(artifacts.supporting_values.clone());
    let official_rows = max_usize(
        &evidence_values,
        &[
            "official_rows_after",
            "added_official_rows",
            "official_rows",
            "official_row_count",
        ],
    );
    let complete_rows = max_usize(
        &evidence_values,
        &[
            "complete_rows_after",
            "added_complete_rows",
            "complete_rows",
            "complete_row_count",
        ],
    );
    let outcome_links = max_usize(
        &evidence_values,
        &[
            "outcome_links_after",
            "added_outcome_links",
            "outcome_links",
        ],
    );
    let no_trade_counterfactuals = max_usize(
        &evidence_values,
        &[
            "no_trade_counterfactuals_after",
            "no_trade_counterfactuals",
            "no_trade_count",
        ],
    );
    let risk_denied_counterfactuals = max_usize(
        &evidence_values,
        &[
            "risk_denied_counterfactuals_after",
            "risk_denied_counterfactuals",
            "risk_denied_count",
            "denied_count",
        ],
    );
    let outcome_diversity_count = max_usize(
        &evidence_values,
        &["outcome_diversity_count", "outcome_diversity"],
    );
    let symbol_diversity_count = artifacts
        .sequence_readiness
        .as_ref()
        .map(|report| report.symbols.len())
        .unwrap_or_else(|| max_usize(values, &["symbol_diversity_count", "symbol_count"]));
    let mut warnings = Vec::new();
    let mut next_evidence_actions = Vec::new();
    let primary_gap = if official_rows < config.min_official_rows
        || complete_rows < config.min_complete_rows
    {
        warnings.push(
            "official or complete KIS evidence rows remain below the hardening threshold"
                .to_string(),
        );
        next_evidence_actions
            .push("expand bounded KIS official rows and complete comparable rows".to_string());
        EvidenceGapPrimaryGap::NeedMoreKISEvidence
    } else if outcome_links < config.min_outcome_links {
        warnings.push("outcome-link coverage remains below the hardening threshold".to_string());
        next_evidence_actions
            .push("increase outcome-linked comparable rows and future-window coverage".to_string());
        EvidenceGapPrimaryGap::NeedOutcomeLinkDepth
    } else if no_trade_counterfactuals < config.min_no_trade_counterfactuals
        || risk_denied_counterfactuals < config.min_risk_denied_counterfactuals
    {
        warnings.push("counterfactual coverage remains below the hardening threshold".to_string());
        next_evidence_actions
            .push("add more NoTrade and RiskDenied counterfactual evidence".to_string());
        EvidenceGapPrimaryGap::NeedCounterfactualDepth
    } else if outcome_diversity_count < config.min_outcome_diversity
        || symbol_diversity_count < config.min_symbol_diversity
    {
        warnings
            .push("outcome or symbol diversity remains below the hardening threshold".to_string());
        next_evidence_actions.push("expand diversity across outcome types and symbols".to_string());
        EvidenceGapPrimaryGap::NeedOutcomeDiversity
    } else if artifacts
        .owner_queue
        .as_ref()
        .is_some_and(|queue| !queue.pending_items.is_empty())
    {
        warnings.push("owner queue still needs manual review discipline".to_string());
        next_evidence_actions
            .push("clear or explicitly classify pending owner review items".to_string());
        EvidenceGapPrimaryGap::NeedOwnerReview
    } else {
        next_evidence_actions.push(
            "keep bounded evidence refresh cadence and preserve deterministic reporting"
                .to_string(),
        );
        EvidenceGapPrimaryGap::Healthy
    };
    EvidenceDepthGapReport {
        report_id: config.report_id.clone(),
        official_rows,
        complete_rows,
        outcome_links,
        no_trade_counterfactuals,
        risk_denied_counterfactuals,
        outcome_diversity_count,
        symbol_diversity_count,
        primary_gap,
        next_evidence_actions: stable_ordered_strings(&next_evidence_actions),
        warnings: stable_ordered_strings(&warnings),
        reason_codes: stable_reason_codes(&[ReasonCode::EvidenceDepthGapBuilt]),
    }
}

fn build_outcome_link_coverage_report(
    config: &EvidenceHardeningConfig,
    values: &[Value],
) -> OutcomeLinkCoverageReport {
    let outcome_link_count = max_usize(
        values,
        &[
            "outcome_links",
            "outcome_links_after",
            "added_outcome_links",
        ],
    );
    let future_window_count = max_usize(
        values,
        &[
            "future_window_count",
            "future_windows",
            "future_window_coverage_count",
        ],
    );
    let no_lookahead_flags = bools_for_keys(values, &["no_lookahead_safe", "no_lookahead_proof"]);
    let no_lookahead_safe =
        !no_lookahead_flags.is_empty() && no_lookahead_flags.into_iter().all(|flag| flag);
    let take_profit_count = max_usize(values, &["take_profit_count", "target_hit_count"]);
    let stop_loss_count = max_usize(values, &["stop_loss_count", "stop_hit_count"]);
    let time_expired_count = max_usize(values, &["time_expired_count", "time_exit_count"]);
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let coverage_status = if config.require_no_lookahead_safe && !no_lookahead_safe {
        blockers.push("no-lookahead proof is missing or false".to_string());
        OutcomeLinkCoverageStatus::BlockedByNoLookahead
    } else if config.require_future_windows && future_window_count == 0 {
        warnings.push("future-window coverage is missing".to_string());
        OutcomeLinkCoverageStatus::NeedFutureWindows
    } else if outcome_link_count < config.min_outcome_links {
        warnings.push("outcome-link depth remains below threshold".to_string());
        OutcomeLinkCoverageStatus::NeedOutcomeLinkDepth
    } else {
        OutcomeLinkCoverageStatus::Healthy
    };
    OutcomeLinkCoverageReport {
        report_id: config.report_id.clone(),
        outcome_link_count,
        future_window_count,
        no_lookahead_safe,
        take_profit_count,
        stop_loss_count,
        time_expired_count,
        coverage_status,
        blockers: stable_ordered_strings(&blockers),
        warnings: stable_ordered_strings(&warnings),
        reason_codes: stable_reason_codes(&[ReasonCode::OutcomeLinkCoverageBuilt]),
    }
}

fn build_counterfactual_coverage_report(
    config: &EvidenceHardeningConfig,
    values: &[Value],
) -> CounterfactualCoverageReport {
    let no_trade_depth = max_usize(
        values,
        &[
            "no_trade_counterfactuals",
            "no_trade_counterfactuals_after",
            "no_trade_count",
        ],
    );
    let risk_denied_depth = max_usize(
        values,
        &[
            "risk_denied_counterfactuals",
            "risk_denied_counterfactuals_after",
            "risk_denied_count",
            "denied_count",
        ],
    );
    let missing_risk_decision_count = max_usize(values, &["missing_risk_decision_count"]);
    let missing_committee_decision_count = max_usize(values, &["missing_committee_decision_count"]);
    let avoided_loss_total = max_f64(values, &["avoided_loss_total", "avoided_loss_total_bps"]);
    let missed_gain_total = max_f64(values, &["missed_gain_total", "missed_gain_total_bps"]);
    let mut warnings = Vec::new();
    let coverage_status = if no_trade_depth < config.min_no_trade_counterfactuals
        || risk_denied_depth < config.min_risk_denied_counterfactuals
        || missing_risk_decision_count > 0
        || missing_committee_decision_count > 0
    {
        warnings.push("counterfactual depth or decision linkage remains incomplete".to_string());
        CounterfactualCoverageStatus::NeedCounterfactualDepth
    } else {
        CounterfactualCoverageStatus::Healthy
    };
    CounterfactualCoverageReport {
        report_id: config.report_id.clone(),
        no_trade_depth,
        risk_denied_depth,
        missing_risk_decision_count,
        missing_committee_decision_count,
        avoided_loss_total,
        missed_gain_total,
        coverage_status,
        warnings: stable_ordered_strings(&warnings),
        reason_codes: stable_reason_codes(&[ReasonCode::CounterfactualCoverageBuilt]),
    }
}

fn build_manual_review_ergonomics_report(
    config: &EvidenceHardeningConfig,
    queue: Option<&OwnerReviewQueue>,
) -> ManualReviewErgonomicsReport {
    let Some(queue) = queue else {
        return ManualReviewErgonomicsReport {
            report_id: config.report_id.clone(),
            pending_review_count: 0,
            risk_blocked_count: 0,
            paper_confirm_allowed_count: 0,
            paper_confirm_blocked_count: 0,
            missing_reason_count: 0,
            unclear_action_count: 0,
            ergonomics_status: ManualReviewErgonomicsStatus::NeedsBetterOwnerDiscipline,
            warnings: vec!["owner review queue artifact is missing".to_string()],
            reason_codes: stable_reason_codes(&[ReasonCode::ManualReviewErgonomicsBuilt]),
        };
    };
    let items = owner_items(queue);
    let pending_review_count = queue.pending_items.len();
    let risk_blocked_count = queue.blocked_items.len();
    let paper_confirm_allowed_count = items
        .iter()
        .filter(|item| {
            item.allowed_owner_actions
                .contains(&AllowedOwnerAction::PaperConfirm)
        })
        .count();
    let paper_confirm_blocked_count = items.len().saturating_sub(paper_confirm_allowed_count);
    let missing_reason_count = items
        .iter()
        .filter(|item| item.reason_codes.is_empty())
        .count();
    let unclear_action_count = items
        .iter()
        .filter(|item| {
            item.allowed_owner_actions.is_empty() || item.forbidden_owner_actions.is_empty()
        })
        .count();
    let mut warnings = Vec::new();
    let ergonomics_status =
        if pending_review_count > 0 || missing_reason_count > 0 || unclear_action_count > 0 {
            warnings
                .push("owner review still needs clearer discipline or queue cleanup".to_string());
            ManualReviewErgonomicsStatus::NeedsBetterOwnerDiscipline
        } else {
            ManualReviewErgonomicsStatus::Improved
        };
    ManualReviewErgonomicsReport {
        report_id: config.report_id.clone(),
        pending_review_count,
        risk_blocked_count,
        paper_confirm_allowed_count,
        paper_confirm_blocked_count,
        missing_reason_count,
        unclear_action_count,
        ergonomics_status,
        warnings: stable_ordered_strings(&warnings),
        reason_codes: stable_reason_codes(&[ReasonCode::ManualReviewErgonomicsBuilt]),
    }
}

fn build_operator_review_workflow_v2(
    config: &EvidenceHardeningConfig,
    queue: Option<&OwnerReviewQueue>,
    values: &[Value],
) -> OperatorReviewWorkflowV2 {
    let (primary_action, queued_actions) = if let Some(queue) = queue {
        if !queue.pending_items.is_empty() {
            (
                OperatorReviewPrimaryAction::ReviewPendingQueue,
                queue
                    .pending_items
                    .iter()
                    .map(|item| format!("review:{}", item.review_id))
                    .collect::<Vec<_>>(),
            )
        } else if !queue.blocked_items.is_empty() {
            (
                OperatorReviewPrimaryAction::InspectRiskBlockedQueue,
                queue
                    .blocked_items
                    .iter()
                    .map(|item| format!("inspect-risk-block:{}", item.review_id))
                    .collect::<Vec<_>>(),
            )
        } else {
            (
                OperatorReviewPrimaryAction::ArchiveStablePaperQueue,
                vec!["archive-stable-paper-review".to_string()],
            )
        }
    } else {
        (
            OperatorReviewPrimaryAction::ReviewPendingQueue,
            vec!["review-owner-queue-artifact".to_string()],
        )
    };
    let mut copyable_commands =
        collect_string_items(values, &["command_suggestion", "command_suggestions"]);
    copyable_commands.extend([
        "cargo run --quiet --bin soma_experiment -- owner-review-queue --config examples/soma_owner_review_queue.toml".to_string(),
        "cargo run --quiet --bin soma_experiment -- evidence-hardening --config examples/soma_evidence_hardening.toml".to_string(),
    ]);
    OperatorReviewWorkflowV2 {
        workflow_id: config.report_id.clone(),
        primary_action,
        queued_actions: stable_ordered_strings(&queued_actions),
        copyable_commands: stable_ordered_strings(&copyable_commands),
        blocked_actions: stable_ordered_strings(&vec![
            "execute-order".to_string(),
            "place-trade".to_string(),
            "enable-live-trading".to_string(),
            "query-account-balance".to_string(),
        ]),
        paper_only: true,
        reason_codes: stable_reason_codes(&[ReasonCode::OperatorReviewWorkflowBuilt]),
    }
}

fn build_control_tower_ergonomics_report(
    config: &EvidenceHardeningConfig,
    control_tower_value: Option<&Value>,
    owner_queue: &Option<OwnerReviewQueue>,
    gap_report: &EvidenceDepthGapReport,
    review_report: &ManualReviewErgonomicsReport,
) -> ControlTowerErgonomicsV1_5Report {
    let candidate_cards = build_candidate_cards(control_tower_value, owner_queue.as_ref());
    let mut evidence_badges = Vec::new();
    match gap_report.primary_gap {
        EvidenceGapPrimaryGap::NeedMoreKISEvidence => {
            evidence_badges.push(EvidenceWarningBadge::NeedMoreKISEvidence);
        }
        EvidenceGapPrimaryGap::NeedOutcomeLinkDepth => {
            evidence_badges.push(EvidenceWarningBadge::NeedOutcomeLinkDepth);
        }
        EvidenceGapPrimaryGap::NeedCounterfactualDepth => {
            evidence_badges.push(EvidenceWarningBadge::NeedCounterfactualDepth);
        }
        EvidenceGapPrimaryGap::NeedOutcomeDiversity => {
            evidence_badges.push(EvidenceWarningBadge::NeedOutcomeDiversity);
        }
        EvidenceGapPrimaryGap::NeedOwnerReview => {
            evidence_badges.push(EvidenceWarningBadge::NeedOwnerReview);
        }
        EvidenceGapPrimaryGap::Healthy => {}
    }
    if matches!(
        review_report.ergonomics_status,
        ManualReviewErgonomicsStatus::NeedsBetterOwnerDiscipline
    ) {
        evidence_badges.push(EvidenceWarningBadge::NeedOwnerReview);
    }
    evidence_badges.sort();
    evidence_badges.dedup();
    let mut copyable_commands = vec![
        "cargo run --quiet --bin soma_experiment -- evidence-hardening --config examples/soma_evidence_hardening.toml".to_string(),
        "cargo run --quiet --bin soma_experiment -- system-review --config examples/soma_system_review_full.toml".to_string(),
        "cargo run --quiet --bin soma_experiment -- owner-review-queue --config examples/soma_owner_review_queue.toml".to_string(),
    ];
    let html = render_control_tower_ergonomics_html(
        config,
        &candidate_cards,
        &evidence_badges,
        &copyable_commands,
    );
    let no_execution_buttons =
        !html.contains("<button") && !html.to_ascii_lowercase().contains("execute");
    let no_account_controls = !html.to_ascii_lowercase().contains("account panel")
        && !html.to_ascii_lowercase().contains("balance panel")
        && !html.to_ascii_lowercase().contains("query-account");
    let mut warnings = Vec::new();
    let render_status = if !no_execution_buttons || !no_account_controls {
        warnings.push("rendered ergonomics output introduced unsafe control language".to_string());
        ControlTowerErgonomicsStatus::Blocked
    } else if candidate_cards.is_empty() {
        warnings.push(
            "candidate cards are empty; ergonomics report needs richer state input".to_string(),
        );
        ControlTowerErgonomicsStatus::NeedsHardening
    } else {
        ControlTowerErgonomicsStatus::Ready
    };
    copyable_commands.sort();
    copyable_commands.dedup();
    ControlTowerErgonomicsV1_5Report {
        report_id: config.report_id.clone(),
        evidence_badges,
        candidate_cards,
        mamba_banner_status: MambaDeferredBannerStatus::Mamba3RuntimeDeferred,
        copyable_commands,
        no_execution_buttons,
        no_account_controls,
        render_status,
        warnings: stable_ordered_strings(&warnings),
        reason_codes: stable_reason_codes(&[ReasonCode::ControlTowerErgonomicsBuilt]),
    }
}

fn build_ui_framework_decision_report(
    config: &EvidenceHardeningConfig,
) -> UIFrameworkDecisionReport {
    UIFrameworkDecisionReport {
        report_id: config.report_id.clone(),
        current_choice: UIFrameworkCurrentChoice::StaticHtmlJsonTxt,
        optional_choice: UIFrameworkOptionalChoice::VanillaTypeScriptEnhancement,
        future_choice: UIFrameworkFutureChoice::TauriSvelteDesktop,
        rejected_options: vec![
            UIFrameworkRejectedOption::ReactNextWeb,
            UIFrameworkRejectedOption::CloudDashboard,
        ],
        decision_status: UIFrameworkDecisionStatus::KeepStaticDashboardNow,
        rationale: stable_ordered_strings(&vec![
            "current local static outputs already satisfy read-only paper-ops monitoring".to_string(),
            "small vanilla enhancements are cheaper than a full framework migration right now".to_string(),
            "desktop interactivity should wait until evidence and operator ergonomics stabilize".to_string(),
            "cloud or server-heavy UI is rejected while the stack remains local-first and review-oriented".to_string(),
        ]),
        warnings: vec!["Tauri + Svelte remains a later option only; no migration is implemented in this sprint".to_string()],
        reason_codes: stable_reason_codes(&[ReasonCode::UIFrameworkDecisionBuilt]),
    }
}

fn build_mamba_application_timing_report(
    config: &EvidenceHardeningConfig,
    gap_report: &EvidenceDepthGapReport,
    outcome_report: &OutcomeLinkCoverageReport,
    counterfactual_report: &CounterfactualCoverageReport,
    sequence_readiness: Option<&SequenceDatasetReadinessReport>,
    mamba_snapshot_value: Option<&Value>,
) -> Mamba3ApplicationTimingReport {
    let evidence_gate_passed = matches!(gap_report.primary_gap, EvidenceGapPrimaryGap::Healthy)
        && matches!(
            outcome_report.coverage_status,
            OutcomeLinkCoverageStatus::Healthy
        )
        && matches!(
            counterfactual_report.coverage_status,
            CounterfactualCoverageStatus::Healthy
        );
    let sequence_gate_passed = sequence_readiness.is_some_and(|report| {
        matches!(
            report.readiness_status,
            SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport
        )
    });
    let snapshot_runtime_present = mamba_snapshot_value
        .map(|value| max_bool(std::slice::from_ref(value), &["mamba3_runtime_present"]))
        .unwrap_or(false);
    let external_prototype_allowed =
        evidence_gate_passed && sequence_gate_passed && !snapshot_runtime_present;
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let final_decision = if !sequence_gate_passed {
        blockers.push("sequence dataset readiness is still below the required gate".to_string());
        Mamba3ApplicationTimingDecision::BuildSequenceDatasetFirst
    } else if !evidence_gate_passed {
        blockers.push(
            "evidence quality remains below the required gate for any Mamba planning".to_string(),
        );
        Mamba3ApplicationTimingDecision::HoldMamba3Deferred
    } else {
        warnings.push(
            "only an external prototype bridge could be considered later; runtime remains deferred"
                .to_string(),
        );
        Mamba3ApplicationTimingDecision::ExternalPrototypeOnlyIfReady
    };
    let current_stage = if external_prototype_allowed {
        Mamba3ApplicationStage::ExternalPrototypeOnlyIfReady
    } else {
        Mamba3ApplicationStage::Deferred
    };
    Mamba3ApplicationTimingReport {
        report_id: config.report_id.clone(),
        current_stage,
        evidence_gate_passed,
        sequence_gate_passed,
        runtime_deferred: true,
        rust_runtime_allowed: false,
        external_prototype_allowed,
        final_decision,
        blockers: stable_ordered_strings(&blockers),
        warnings: stable_ordered_strings(&warnings),
        reason_codes: stable_reason_codes(&[ReasonCode::MambaApplicationTimingBuilt]),
    }
}

fn build_final_recommendations(
    gap_report: &EvidenceDepthGapReport,
    review_report: &ManualReviewErgonomicsReport,
    ergonomics_report: &ControlTowerErgonomicsV1_5Report,
    ui_report: &UIFrameworkDecisionReport,
    mamba_report: &Mamba3ApplicationTimingReport,
) -> Vec<EvidenceHardeningRecommendation> {
    let mut recommendations = BTreeSet::new();
    if matches!(gap_report.primary_gap, EvidenceGapPrimaryGap::Healthy) {
        recommendations.insert(EvidenceHardeningRecommendation::EvidenceHardeningReady);
    } else {
        recommendations.insert(EvidenceHardeningRecommendation::NeedMoreEvidence);
        match gap_report.primary_gap {
            EvidenceGapPrimaryGap::NeedMoreKISEvidence => {
                recommendations.insert(EvidenceHardeningRecommendation::NeedMoreKISEvidence);
            }
            EvidenceGapPrimaryGap::NeedOutcomeLinkDepth => {
                recommendations.insert(EvidenceHardeningRecommendation::NeedOutcomeLinkDepth);
            }
            EvidenceGapPrimaryGap::NeedCounterfactualDepth => {
                recommendations.insert(EvidenceHardeningRecommendation::NeedCounterfactualDepth);
            }
            EvidenceGapPrimaryGap::NeedOutcomeDiversity => {
                recommendations.insert(EvidenceHardeningRecommendation::NeedOutcomeDiversity);
            }
            EvidenceGapPrimaryGap::NeedOwnerReview => {
                recommendations.insert(EvidenceHardeningRecommendation::NeedOwnerReview);
            }
            EvidenceGapPrimaryGap::Healthy => {}
        }
    }
    if matches!(
        review_report.ergonomics_status,
        ManualReviewErgonomicsStatus::Improved
    ) {
        recommendations.insert(EvidenceHardeningRecommendation::ReviewErgonomicsImproved);
    } else {
        recommendations.insert(EvidenceHardeningRecommendation::NeedOwnerReview);
    }
    if matches!(
        ergonomics_report.render_status,
        ControlTowerErgonomicsStatus::Ready
    ) {
        recommendations.insert(EvidenceHardeningRecommendation::ControlTowerV1_5Ready);
    }
    match ui_report.decision_status {
        UIFrameworkDecisionStatus::KeepStaticDashboardNow => {
            recommendations.insert(EvidenceHardeningRecommendation::KeepStaticDashboardNow);
            recommendations.insert(EvidenceHardeningRecommendation::PlanTauriSvelteLater);
        }
        UIFrameworkDecisionStatus::PlanTauriSvelteLater => {
            recommendations.insert(EvidenceHardeningRecommendation::PlanTauriSvelteLater);
        }
    }
    match mamba_report.final_decision {
        Mamba3ApplicationTimingDecision::HoldMamba3Deferred => {
            recommendations.insert(EvidenceHardeningRecommendation::HoldMamba3Deferred);
        }
        Mamba3ApplicationTimingDecision::BuildSequenceDatasetFirst => {
            recommendations.insert(EvidenceHardeningRecommendation::HoldMamba3Deferred);
            recommendations.insert(EvidenceHardeningRecommendation::BuildSequenceDatasetFirst);
        }
        Mamba3ApplicationTimingDecision::ExternalPrototypeOnlyIfReady => {
            recommendations.insert(EvidenceHardeningRecommendation::HoldMamba3Deferred);
        }
    }
    recommendations.insert(EvidenceHardeningRecommendation::KeepTrinity);
    recommendations.into_iter().collect()
}

fn build_final_summary(
    gap_report: &EvidenceDepthGapReport,
    review_report: &ManualReviewErgonomicsReport,
    ui_report: &UIFrameworkDecisionReport,
    mamba_report: &Mamba3ApplicationTimingReport,
    recommendations: &[EvidenceHardeningRecommendation],
) -> String {
    [
        format!("primary_gap={:?}", gap_report.primary_gap),
        format!(
            "review_ergonomics_status={:?}",
            review_report.ergonomics_status
        ),
        format!("ui_framework_decision={:?}", ui_report.decision_status),
        format!("mamba_timing_decision={:?}", mamba_report.final_decision),
        format!(
            "recommendations={}",
            recommendations
                .iter()
                .map(|item| format!("{item:?}"))
                .collect::<Vec<_>>()
                .join("|")
        ),
    ]
    .join("\n")
}

fn build_candidate_cards(
    control_tower_value: Option<&Value>,
    owner_queue: Option<&OwnerReviewQueue>,
) -> Vec<CandidateCardV1_5> {
    let Some(control_tower_value) = control_tower_value else {
        return Vec::new();
    };
    let open_positions = string_map_by_id(
        control_tower_value,
        "paper_position_panel",
        &["open_positions"],
        "candidate_id",
    );
    let human_confirm_ids = item_ids(
        control_tower_value,
        &["human_confirm_panel", "items"],
        "item_id",
    );
    let blocked_ids = owner_queue
        .map(|queue| {
            queue
                .blocked_items
                .iter()
                .filter_map(|item| item.candidate_id.clone())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let paper_confirmed_ids = owner_queue
        .map(|queue| {
            queue
                .paper_confirmed_items
                .iter()
                .filter_map(|item| item.candidate_id.clone())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let deferred_ids = owner_queue
        .map(|queue| {
            queue
                .deferred_items
                .iter()
                .filter_map(|item| item.candidate_id.clone())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let mut cards = Vec::new();
    if let Some(candidates) = value_at_path(control_tower_value, &["candidate_panel", "candidates"])
        .and_then(Value::as_array)
    {
        for candidate in candidates {
            let candidate_id = value_string(candidate.get("candidate_id")).unwrap_or_default();
            let symbol = value_string(candidate.get("symbol")).unwrap_or_default();
            let raw_status = value_string(candidate.get("candidate_status")).unwrap_or_default();
            let status = if open_positions.contains_key(&candidate_id) {
                CandidateCardStatusV1_5::PaperOpen
            } else if paper_confirmed_ids.contains(&candidate_id) {
                CandidateCardStatusV1_5::PaperApproved
            } else if blocked_ids.contains(&candidate_id) || raw_status.contains("RiskBlocked") {
                CandidateCardStatusV1_5::RiskBlocked
            } else if raw_status.contains("NoTrade") {
                CandidateCardStatusV1_5::NoTrade
            } else if human_confirm_ids
                .iter()
                .any(|item_id| item_id.contains(&candidate_id))
                || raw_status.contains("HumanConfirm")
            {
                CandidateCardStatusV1_5::HumanConfirmRequired
            } else if deferred_ids.contains(&candidate_id) {
                CandidateCardStatusV1_5::DiagnosticOnly
            } else {
                CandidateCardStatusV1_5::Official
            };
            cards.push(CandidateCardV1_5 {
                candidate_id: candidate_id.clone(),
                symbol,
                status,
                evidence_summary: format!("candidate_status={raw_status}"),
                review_summary: if blocked_ids.contains(&candidate_id) {
                    "blocked_by_risk_governor=true".to_string()
                } else if paper_confirmed_ids.contains(&candidate_id) {
                    "paper_confirmed=true".to_string()
                } else {
                    "paper_only_review=true".to_string()
                },
            });
        }
    }
    cards.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
    cards
}

fn render_control_tower_ergonomics_html(
    config: &EvidenceHardeningConfig,
    candidate_cards: &[CandidateCardV1_5],
    evidence_badges: &[EvidenceWarningBadge],
    copyable_commands: &[String],
) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{}</title>\n<style>body{{font-family:-apple-system,BlinkMacSystemFont,Segoe UI,sans-serif;margin:24px;background:#0f172a;color:#e2e8f0}}section{{border:1px solid #334155;border-radius:12px;padding:16px;margin:0 0 16px 0;background:#111827}}.badge{{display:inline-block;padding:4px 8px;margin:0 8px 8px 0;border-radius:999px;background:#1e293b}}.grid{{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:16px}}pre,code{{background:#020617;color:#cbd5e1;padding:4px 6px;border-radius:6px;white-space:pre-wrap}}</style>\n</head>\n<body>\n<h1>{}</h1>\n<p>Static local ergonomics preview only. Copy commands manually outside the browser. No execution, no broker, no account, no order controls.</p>\n<section><h2>Evidence badges</h2>{}</section>\n<section><h2>Mamba deferred banner</h2><div class=\"badge\">Mamba3RuntimeDeferred</div><div class=\"badge\">BuildSequenceDatasetFirst</div><div class=\"badge\">ExternalPrototypeOnlyIfReady</div></section>\n<section><h2>Copyable runbook commands</h2><pre>{}</pre></section>\n<section><h2>Candidate cards</h2><div class=\"grid\">{}</div></section>\n</body>\n</html>",
        escape_html(&config.report_id),
        escape_html(&config.report_id),
        evidence_badges
            .iter()
            .map(|badge| format!("<div class=\"badge\">{:?}</div>", badge))
            .collect::<Vec<_>>()
            .join(""),
        escape_html(&copyable_commands.join("\n")),
        candidate_cards
            .iter()
            .map(|card| {
                format!(
                    "<section><h3>{}</h3><p>symbol={}</p><p>status={:?}</p><p>{}</p><p>{}</p></section>",
                    escape_html(&card.candidate_id),
                    escape_html(&card.symbol),
                    card.status,
                    escape_html(&card.evidence_summary),
                    escape_html(&card.review_summary),
                )
            })
            .collect::<Vec<_>>()
            .join("")
    )
}

fn write_bundle_outputs(
    config: &EvidenceHardeningConfig,
    bundle: &EvidenceHardeningBundle,
) -> Result<(), String> {
    let artifact_dir = config.artifact_dir();
    fs::create_dir_all(&artifact_dir).map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("evidence_hardening_bundle.json"),
        bundle.to_json_string()?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("evidence_hardening_bundle.txt"),
        bundle.to_text(),
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("evidence_depth_gap_report.json"),
        serde_json::to_string_pretty(&bundle.evidence_depth_gap_report)
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("outcome_link_coverage_report.json"),
        serde_json::to_string_pretty(&bundle.outcome_link_coverage_report)
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("counterfactual_coverage_report.json"),
        serde_json::to_string_pretty(&bundle.counterfactual_coverage_report)
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("manual_review_ergonomics_report.json"),
        serde_json::to_string_pretty(&bundle.manual_review_ergonomics_report)
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("control_tower_ergonomics_v1_5_report.json"),
        serde_json::to_string_pretty(&bundle.control_tower_ergonomics_v1_5_report)
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("control_tower_ergonomics_v1_5.html"),
        render_control_tower_ergonomics_html(
            config,
            &bundle.control_tower_ergonomics_v1_5_report.candidate_cards,
            &bundle.control_tower_ergonomics_v1_5_report.evidence_badges,
            &bundle
                .control_tower_ergonomics_v1_5_report
                .copyable_commands,
        ),
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("ui_framework_decision_report.json"),
        serde_json::to_string_pretty(&bundle.ui_framework_decision_report)
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    fs::write(
        artifact_dir.join("mamba3_application_timing_report.json"),
        serde_json::to_string_pretty(&bundle.mamba3_application_timing_report)
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn owner_items(queue: &OwnerReviewQueue) -> Vec<&OwnerReviewItem> {
    queue
        .pending_items
        .iter()
        .chain(queue.reviewed_items.iter())
        .chain(queue.deferred_items.iter())
        .chain(queue.dismissed_items.iter())
        .chain(queue.paper_confirmed_items.iter())
        .chain(queue.blocked_items.iter())
        .chain(queue.expired_items.iter())
        .collect()
}

fn max_usize(values: &[Value], keys: &[&str]) -> usize {
    keys.iter()
        .flat_map(|key| values.iter().flat_map(|value| numeric_matches(value, key)))
        .max()
        .unwrap_or_default()
}

fn max_f64(values: &[Value], keys: &[&str]) -> f64 {
    keys.iter()
        .flat_map(|key| values.iter().flat_map(|value| float_matches(value, key)))
        .fold(0.0_f64, f64::max)
}

fn max_bool(values: &[Value], keys: &[&str]) -> bool {
    bools_for_keys(values, keys).into_iter().any(|value| value)
}

fn bools_for_keys(values: &[Value], keys: &[&str]) -> Vec<bool> {
    keys.iter()
        .flat_map(|key| values.iter().flat_map(|value| bool_matches(value, key)))
        .collect()
}

fn collect_string_items(values: &[Value], keys: &[&str]) -> Vec<String> {
    let mut items = Vec::new();
    for key in keys {
        for value in values {
            collect_key_values(value, key, &mut |matched| match matched {
                Value::String(text) => items.push(text.clone()),
                Value::Array(entries) => {
                    for entry in entries {
                        if let Some(text) = value_string(Some(entry)) {
                            items.push(text);
                        }
                    }
                }
                _ => {}
            });
        }
    }
    stable_ordered_strings(&items)
}

fn numeric_matches(value: &Value, key: &str) -> Vec<usize> {
    let mut out = Vec::new();
    collect_key_values(value, key, &mut |matched| {
        if let Some(value) = value_usize(Some(matched)) {
            out.push(value);
        }
    });
    out
}

fn float_matches(value: &Value, key: &str) -> Vec<f64> {
    let mut out = Vec::new();
    collect_key_values(value, key, &mut |matched| {
        if let Some(value) = value_f64(Some(matched)) {
            out.push(value);
        }
    });
    out
}

fn bool_matches(value: &Value, key: &str) -> Vec<bool> {
    let mut out = Vec::new();
    collect_key_values(value, key, &mut |matched| {
        if let Some(value) = matched.as_bool() {
            out.push(value);
        }
    });
    out
}

fn collect_key_values(value: &Value, key: &str, f: &mut dyn FnMut(&Value)) {
    match value {
        Value::Object(map) => {
            if let Some(item) = map.get(key) {
                f(item);
            }
            for item in map.values() {
                collect_key_values(item, key, f);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_key_values(item, key, f);
            }
        }
        _ => {}
    }
}

fn value_at_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

fn item_ids(value: &Value, path: &[&str], key: &str) -> BTreeSet<String> {
    value_at_path(value, path)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| value_string(item.get(key)))
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default()
}

fn string_map_by_id(
    value: &Value,
    root_key: &str,
    collection_path: &[&str],
    id_key: &str,
) -> BTreeMap<String, String> {
    let mut path = vec![root_key];
    path.extend(collection_path.iter().copied());
    value_at_path(value, &path)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let id = value_string(item.get(id_key))?;
                    Some((id, serde_json::to_string(item).ok()?))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default()
}

fn value_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

fn value_usize(value: Option<&Value>) -> Option<usize> {
    match value? {
        Value::Number(number) => number.as_u64().map(|item| item as usize),
        Value::String(text) => text.parse::<usize>().ok(),
        _ => None,
    }
}

fn value_f64(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    }
}

fn load_latest_json(paths: &[String]) -> Result<Option<Value>, String> {
    let mut latest = None;
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        latest = Some(serde_json::from_str(&text).map_err(|err| err.to_string())?);
    }
    Ok(latest)
}

fn load_json_values(paths: &[String]) -> Result<Vec<Value>, String> {
    let mut values = Vec::new();
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        values.push(serde_json::from_str(&text).map_err(|err| err.to_string())?);
    }
    Ok(values)
}

fn load_latest_typed<T: for<'de> Deserialize<'de>>(paths: &[String]) -> Result<Option<T>, String> {
    let mut latest = None;
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        latest = Some(serde_json::from_str(&text).map_err(|err| err.to_string())?);
    }
    Ok(latest)
}

fn load_latest_owner_queue(paths: &[String]) -> Result<Option<OwnerReviewQueue>, String> {
    let mut latest = None;
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let value = serde_json::from_str::<Value>(&text).map_err(|err| err.to_string())?;
        latest = Some(
            serde_json::from_value::<OwnerReviewQueue>(value.clone())
                .or_else(|_| {
                    value
                        .get("owner_review_queue")
                        .cloned()
                        .ok_or_else(|| {
                            serde_json::Error::io(std::io::Error::other(
                                "missing owner_review_queue",
                            ))
                        })
                        .and_then(serde_json::from_value)
                })
                .map_err(|err| err.to_string())?,
        );
    }
    Ok(latest)
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
