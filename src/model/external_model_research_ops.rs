use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_ordered_strings, stable_reason_codes};

use super::{
    CalibrationDriftReport, CalibrationDriftStatus, ConservativeExternalLeaderboard,
    ConservativeExternalLeaderboardStatus, ExternalArtifactRegistryAuditReport,
    ExternalArtifactRegistryAuditStatus, ExternalEvaluationHistoryReport,
    ExternalEvaluationHistoryStatus, ExternalModelArtifactKind, ExternalModelArtifactRegistry,
    ExternalModelVersionComparisonReport, ExternalModelVersionComparisonStatus,
    PreviousExternalComparisonRecommendation, PreviousExternalComparisonReport,
};

fn default_output_root() -> String {
    "target/soma_external_model_research_ops".to_string()
}

fn default_max_models() -> usize {
    16
}

fn default_max_versions() -> usize {
    16
}

fn default_max_review_items() -> usize {
    64
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalModelResearchOpsConfig {
    pub ops_id: String,
    #[serde(default)]
    pub external_artifact_registry_paths: Vec<String>,
    #[serde(default)]
    pub conservative_leaderboard_paths: Vec<String>,
    #[serde(default)]
    pub calibration_drift_paths: Vec<String>,
    #[serde(default)]
    pub evaluation_history_paths: Vec<String>,
    #[serde(default)]
    pub model_version_comparison_paths: Vec<String>,
    #[serde(default)]
    pub previous_external_comparison_paths: Vec<String>,
    #[serde(default)]
    pub external_registry_audit_paths: Vec<String>,
    #[serde(default)]
    pub owner_model_review_paths: Vec<String>,
    #[serde(default)]
    pub control_tower_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_models")]
    pub max_models: usize,
    #[serde(default = "default_max_versions")]
    pub max_versions: usize,
    #[serde(default = "default_max_review_items")]
    pub max_review_items: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub enable_owner_model_review: bool,
    #[serde(default = "default_true")]
    pub enable_watchlist: bool,
    #[serde(default = "default_true")]
    pub enable_retirement_policy: bool,
    #[serde(default = "default_true")]
    pub enable_control_tower_model_ops_panel: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ExternalModelResearchOpsConfig {
    fn default() -> Self {
        Self {
            ops_id: "sprint65-external-model-research-ops".to_string(),
            external_artifact_registry_paths: Vec::new(),
            conservative_leaderboard_paths: Vec::new(),
            calibration_drift_paths: Vec::new(),
            evaluation_history_paths: Vec::new(),
            model_version_comparison_paths: Vec::new(),
            previous_external_comparison_paths: Vec::new(),
            external_registry_audit_paths: Vec::new(),
            owner_model_review_paths: Vec::new(),
            control_tower_paths: Vec::new(),
            output_root: default_output_root(),
            max_models: default_max_models(),
            max_versions: default_max_versions(),
            max_review_items: default_max_review_items(),
            max_bytes: default_max_bytes(),
            enable_owner_model_review: true,
            enable_watchlist: true,
            enable_retirement_policy: true,
            enable_control_tower_model_ops_panel: true,
            reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelResearchOpsConfigBuilt]),
        }
    }
}

impl ExternalModelResearchOpsConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let mut config: Self = toml::from_str(&text).map_err(|err| err.to_string())?;
        config.reason_codes =
            stable_reason_codes(&[ReasonCode::ExternalModelResearchOpsConfigBuilt]);
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.ops_id.trim().is_empty()
            || self.max_models == 0
            || self.max_versions == 0
            || self.max_review_items == 0
            || self.max_bytes == 0
        {
            return Err(
                "external model research ops config requires non-empty ids and positive limits"
                    .to_string(),
            );
        }
        let all_paths = self
            .external_artifact_registry_paths
            .iter()
            .chain(self.conservative_leaderboard_paths.iter())
            .chain(self.calibration_drift_paths.iter())
            .chain(self.evaluation_history_paths.iter())
            .chain(self.model_version_comparison_paths.iter())
            .chain(self.previous_external_comparison_paths.iter())
            .chain(self.external_registry_audit_paths.iter())
            .chain(self.owner_model_review_paths.iter())
            .chain(self.control_tower_paths.iter())
            .chain(std::iter::once(&self.output_root));
        if all_paths.clone().any(|path| path.contains("://")) {
            return Err("external model research ops config paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.ops_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelLifecycleStatus {
    Registered,
    Imported,
    Evaluated,
    ResearchCandidate,
    DiagnosticOnly,
    Watchlisted,
    NeedsMorePredictions,
    NeedsCalibrationReview,
    NeedsRiskReview,
    Downgraded,
    Retired,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalModelLifecycleRecord {
    pub model_id: String,
    pub model_version: String,
    pub current_status: ExternalModelLifecycleStatus,
    #[serde(default)]
    pub previous_status: Option<ExternalModelLifecycleStatus>,
    pub status_reason: String,
    #[serde(default)]
    pub allowed_transitions: Vec<String>,
    #[serde(default)]
    pub forbidden_transitions: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelReviewItemKind {
    NewModelVersion,
    CalibrationDriftReview,
    RiskBehaviorReview,
    CoverageWeakReview,
    LeaderboardChangeReview,
    MambaFamilyReview,
    OwnerRequestedReview,
    RetirementReview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelReviewItemStatus {
    Pending,
    Reviewed,
    Deferred,
    Dismissed,
    Retired,
    DowngradedToDiagnostic,
    KeptAsResearchCandidate,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalModelReviewItem {
    pub review_id: String,
    pub model_id: String,
    pub model_version: String,
    pub item_kind: ExternalModelReviewItemKind,
    pub status: ExternalModelReviewItemStatus,
    pub summary: String,
    #[serde(default)]
    pub recommended_actions: Vec<String>,
    #[serde(default)]
    pub forbidden_actions: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalModelReviewQueue {
    pub queue_id: String,
    #[serde(default)]
    pub pending_items: Vec<ExternalModelReviewItem>,
    #[serde(default)]
    pub reviewed_items: Vec<ExternalModelReviewItem>,
    #[serde(default)]
    pub deferred_items: Vec<ExternalModelReviewItem>,
    #[serde(default)]
    pub retired_items: Vec<ExternalModelReviewItem>,
    #[serde(default)]
    pub downgraded_items: Vec<ExternalModelReviewItem>,
    #[serde(default)]
    pub blocked_items: Vec<ExternalModelReviewItem>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OwnerModelReviewActionKind {
    AddModelNote,
    WatchModel,
    UnwatchModel,
    RequestMorePredictions,
    RequestCalibrationReview,
    RequestRiskReview,
    MarkModelDiagnosticOnly,
    RetireModelVersion,
    KeepResearchCandidate,
    DismissReview,
    DeferReview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerModelReviewAction {
    pub action_id: String,
    pub model_id: String,
    pub model_version: String,
    pub action_kind: OwnerModelReviewActionKind,
    #[serde(default)]
    pub note: Option<String>,
    pub allowed: bool,
    pub diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerModelReviewImpactReport {
    pub report_id: String,
    #[serde(default)]
    pub actions: Vec<OwnerModelReviewAction>,
    pub accepted_count: usize,
    pub blocked_count: usize,
    pub retired_count: usize,
    pub downgraded_count: usize,
    pub watchlisted_count: usize,
    pub review_requested_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelWatchStatus {
    Active,
    Removed,
    Retired,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalModelWatchlistEntry {
    pub model_id: String,
    pub model_version: String,
    pub watch_reason: String,
    pub watched_by_owner: bool,
    pub watch_status: ExternalModelWatchStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalModelWatchlist {
    #[serde(default)]
    pub entries: Vec<ExternalModelWatchlistEntry>,
    pub active_count: usize,
    pub retired_count: usize,
    pub diagnostic_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelComparabilityDimension {
    DatasetFingerprint,
    FeatureSchemaHash,
    LabelManifestHash,
    SplitPolicy,
    PredictionSchemaVersion,
    ModelCardValidity,
    EvaluationMetricAvailability,
    CoverageRatio,
    CalibrationMetricAvailability,
    RiskMetricAvailability,
    AblationAvailability,
    PromotionGateAvailability,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelComparabilityCell {
    pub model_id: String,
    pub model_version: String,
    pub dimension: ModelComparabilityDimension,
    pub comparable: bool,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub expected_value: Option<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelComparabilityMatrixStatus {
    FullyComparable,
    PartiallyComparable,
    NotComparable,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelComparabilityMatrix {
    #[serde(default)]
    pub cells: Vec<ModelComparabilityCell>,
    pub comparable_models: usize,
    pub non_comparable_models: usize,
    pub matrix_status: ModelComparabilityMatrixStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactCompletenessStatus {
    Complete,
    MostlyComplete,
    Incomplete,
    MissingCriticalArtifacts,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArtifactCompletenessScore {
    pub model_id: String,
    pub model_version: String,
    pub has_model_card: bool,
    pub has_prediction_csv: bool,
    pub has_import_report: bool,
    pub has_evaluation_report: bool,
    pub has_vs_trinity_report: bool,
    pub has_ablation_report: bool,
    pub has_promotion_gate: bool,
    pub has_registry_audit: bool,
    pub has_mamba_contract_if_applicable: bool,
    pub completeness_ratio: f64,
    pub completeness_status: ArtifactCompletenessStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelEvidenceRiskLevel {
    Low,
    Medium,
    High,
    Critical,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelEvidenceRecommendedAction {
    KeepResearchCandidate,
    DowngradeToDiagnostic,
    RetireModelVersion,
    RequestMorePredictions,
    RequestCalibrationReview,
    RequestRiskReview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelEvidenceRiskProfile {
    pub model_id: String,
    pub model_version: String,
    pub coverage_ratio: String,
    pub calibration_status: String,
    pub drift_status: String,
    pub risk_status: String,
    pub ablation_status: String,
    pub promotion_gate_status: String,
    pub leaderboard_status: String,
    pub sequence_size_warning: bool,
    pub small_sample_warning: bool,
    pub evidence_risk_level: ModelEvidenceRiskLevel,
    pub recommended_action: ModelEvidenceRecommendedAction,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaderboardChangeKind {
    RankUp,
    RankDown,
    NewlyEligible,
    NewlyBlocked,
    Removed,
    ScoreChanged,
    DriftChanged,
    PromotionStatusChanged,
    NoChange,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelLeaderboardChange {
    pub model_id: String,
    pub model_version: String,
    #[serde(default)]
    pub previous_rank: Option<usize>,
    #[serde(default)]
    pub current_rank: Option<usize>,
    pub change_kind: LeaderboardChangeKind,
    #[serde(default)]
    pub score_delta: Option<f64>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelLeaderboardChangeLog {
    #[serde(default)]
    pub changes: Vec<ModelLeaderboardChange>,
    pub newly_eligible_count: usize,
    pub newly_blocked_count: usize,
    pub rank_change_count: usize,
    pub no_change_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelResearchOpsStatus {
    ModelResearchOpsReady,
    NeedMorePredictionHistory,
    NeedBetterCoverage,
    NeedBetterCalibration,
    NeedBetterRiskBehavior,
    NeedOwnerReview,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelResearchOpsRecommendation {
    KeepResearchCandidate,
    DowngradeToDiagnostic,
    RetireWeakModelVersion,
    RequestMorePredictions,
    ImproveCalibration,
    ImproveRiskBehavior,
    KeepBaselineAndTrinity,
    HoldMamba3RuntimeDeferred,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalModelResearchOpsReport {
    pub ops_id: String,
    pub lifecycle_summary: String,
    pub review_queue_summary: String,
    pub watchlist_summary: String,
    pub comparability_matrix_status: String,
    pub artifact_completeness_summary: String,
    pub evidence_risk_profile_summary: String,
    pub leaderboard_change_summary: String,
    pub final_status: ExternalModelResearchOpsStatus,
    pub final_recommendation: ExternalModelResearchOpsRecommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerModelOpsPanel {
    pub research_ops_status: String,
    pub review_queue_summary: String,
    #[serde(default)]
    pub watchlist: Vec<String>,
    pub comparability_status: String,
    pub artifact_completeness_status: String,
    #[serde(default)]
    pub model_risk_profiles: Vec<String>,
    #[serde(default)]
    pub leaderboard_changes: Vec<String>,
    pub mamba3fin_family_status: String,
    #[serde(default)]
    pub next_actions: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl ControlTowerModelOpsPanel {
    pub fn stabilize(&mut self) {
        self.watchlist = stable_ordered_strings(&self.watchlist);
        self.model_risk_profiles = stable_ordered_strings(&self.model_risk_profiles);
        self.leaderboard_changes = stable_ordered_strings(&self.leaderboard_changes);
        self.next_actions = stable_ordered_strings(&self.next_actions);
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn to_text(&self) -> String {
        [
            format!("research_ops_status={}", self.research_ops_status),
            format!("review_queue_summary={}", self.review_queue_summary),
            format!("watchlist={}", self.watchlist.join("|")),
            format!("comparability_status={}", self.comparability_status),
            format!(
                "artifact_completeness_status={}",
                self.artifact_completeness_status
            ),
            format!("model_risk_profiles={}", self.model_risk_profiles.join("|")),
            format!("leaderboard_changes={}", self.leaderboard_changes.join("|")),
            format!("mamba3fin_family_status={}", self.mamba3fin_family_status),
            format!("next_actions={}", self.next_actions.join(" || ")),
        ]
        .join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalModelResearchOpsStorageReport {
    pub artifact_count: usize,
    pub storage_bytes: usize,
    pub max_bytes: usize,
    pub within_budget: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalModelResearchOpsBundle {
    #[serde(default)]
    pub lifecycle_records: Vec<ExternalModelLifecycleRecord>,
    pub external_model_review_queue: ExternalModelReviewQueue,
    #[serde(default)]
    pub owner_model_review_impact_report: Option<OwnerModelReviewImpactReport>,
    pub external_model_watchlist: ExternalModelWatchlist,
    pub model_comparability_matrix: ModelComparabilityMatrix,
    #[serde(default)]
    pub artifact_completeness_scores: Vec<ArtifactCompletenessScore>,
    #[serde(default)]
    pub model_evidence_risk_profiles: Vec<ModelEvidenceRiskProfile>,
    pub model_leaderboard_changelog: ModelLeaderboardChangeLog,
    pub external_model_research_ops_report: ExternalModelResearchOpsReport,
    #[serde(default)]
    pub control_tower_model_ops_panel_summary: Option<ControlTowerModelOpsPanel>,
    pub storage_report: ExternalModelResearchOpsStorageReport,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExternalModelResearchOpsRunner;

impl ExternalModelResearchOpsRunner {
    pub fn run(
        &self,
        config: &ExternalModelResearchOpsConfig,
    ) -> Result<ExternalModelResearchOpsBundle, String> {
        config.validate()?;
        let registry = load_first_report::<ExternalModelArtifactRegistry>(
            &config.external_artifact_registry_paths,
        )?
        .unwrap_or_else(empty_registry);
        let leaderboard = load_first_report::<ConservativeExternalLeaderboard>(
            &config.conservative_leaderboard_paths,
        )?
        .unwrap_or_else(|| empty_leaderboard(&config.ops_id));
        let drift = load_first_report::<CalibrationDriftReport>(&config.calibration_drift_paths)?
            .unwrap_or_else(empty_drift);
        let _history =
            load_first_report::<ExternalEvaluationHistoryReport>(&config.evaluation_history_paths)?
                .unwrap_or_else(|| empty_history(&config.ops_id));
        let version_comparison = load_first_report::<ExternalModelVersionComparisonReport>(
            &config.model_version_comparison_paths,
        )?
        .unwrap_or_else(|| empty_version_comparison(&config.ops_id));
        let _previous_comparison = load_first_report::<PreviousExternalComparisonReport>(
            &config.previous_external_comparison_paths,
        )?
        .unwrap_or_else(empty_previous_comparison);
        let audit = load_first_report::<ExternalArtifactRegistryAuditReport>(
            &config.external_registry_audit_paths,
        )?
        .unwrap_or_else(|| empty_audit(&config.ops_id));
        let owner_actions = load_owner_actions(&config.owner_model_review_paths)?;
        let model_keys = collect_model_keys(&registry, &leaderboard)?;
        enforce_limits(config, &model_keys, &owner_actions)?;

        let lifecycle_records =
            build_lifecycle_records(config, &registry, &leaderboard, &drift, &owner_actions);
        let watchlist = build_watchlist(config, &owner_actions);
        let comparability_matrix =
            build_comparability_matrix(&registry, &leaderboard, &lifecycle_records);
        let completeness_scores =
            build_artifact_completeness_scores(&registry, &audit, &model_keys);
        let risk_profiles =
            build_model_evidence_risk_profiles(&leaderboard, &drift, &lifecycle_records);
        let changelog =
            build_model_leaderboard_changelog(&leaderboard, &version_comparison, &drift);
        let review_queue = build_review_queue(
            config,
            &lifecycle_records,
            &leaderboard,
            &drift,
            &risk_profiles,
            &changelog,
            &owner_actions,
            &registry,
        );
        let owner_impact = if config.enable_owner_model_review {
            Some(build_owner_model_review_impact_report(
                config,
                &owner_actions,
            ))
        } else {
            None
        };
        let ops_report = build_external_model_research_ops_report(
            config,
            &lifecycle_records,
            &review_queue,
            &watchlist,
            &comparability_matrix,
            &completeness_scores,
            &risk_profiles,
            &changelog,
        );
        let panel = if config.enable_control_tower_model_ops_panel {
            Some(build_control_tower_model_ops_panel(
                &ops_report,
                &review_queue,
                &watchlist,
                &comparability_matrix,
                &completeness_scores,
                &risk_profiles,
                &changelog,
                &leaderboard,
            ))
        } else {
            None
        };
        let storage_report = build_storage_report(config, &registry, &owner_actions)?;
        let final_summary = build_final_summary(&ops_report, &review_queue, &watchlist);
        let bundle = ExternalModelResearchOpsBundle {
            lifecycle_records,
            external_model_review_queue: review_queue,
            owner_model_review_impact_report: owner_impact,
            external_model_watchlist: watchlist,
            model_comparability_matrix: comparability_matrix,
            artifact_completeness_scores: completeness_scores,
            model_evidence_risk_profiles: risk_profiles,
            model_leaderboard_changelog: changelog,
            external_model_research_ops_report: ops_report,
            control_tower_model_ops_panel_summary: panel,
            storage_report,
            final_summary,
            reason_codes: stable_reason_codes(&[
                ReasonCode::ExternalModelResearchOpsRunnerBuilt,
                ReasonCode::ExternalModelResearchOpsBundleBuilt,
            ]),
        };
        write_bundle(config, &bundle)?;
        Ok(bundle)
    }

    pub fn run_review_queue(
        &self,
        config: &ExternalModelResearchOpsConfig,
    ) -> Result<ExternalModelReviewQueue, String> {
        Ok(self.run(config)?.external_model_review_queue)
    }

    pub fn run_watchlist(
        &self,
        config: &ExternalModelResearchOpsConfig,
    ) -> Result<ExternalModelWatchlist, String> {
        Ok(self.run(config)?.external_model_watchlist)
    }

    pub fn run_comparability_matrix(
        &self,
        config: &ExternalModelResearchOpsConfig,
    ) -> Result<ModelComparabilityMatrix, String> {
        Ok(self.run(config)?.model_comparability_matrix)
    }

    pub fn run_artifact_completeness(
        &self,
        config: &ExternalModelResearchOpsConfig,
    ) -> Result<Vec<ArtifactCompletenessScore>, String> {
        Ok(self.run(config)?.artifact_completeness_scores)
    }

    pub fn run_risk_profile(
        &self,
        config: &ExternalModelResearchOpsConfig,
    ) -> Result<Vec<ModelEvidenceRiskProfile>, String> {
        Ok(self.run(config)?.model_evidence_risk_profiles)
    }

    pub fn run_leaderboard_changelog(
        &self,
        config: &ExternalModelResearchOpsConfig,
    ) -> Result<ModelLeaderboardChangeLog, String> {
        Ok(self.run(config)?.model_leaderboard_changelog)
    }
}

fn load_first_report<T: for<'de> Deserialize<'de>>(paths: &[String]) -> Result<Option<T>, String> {
    if let Some(path) = paths.first() {
        Ok(Some(read_json_file(Path::new(path))?))
    } else {
        Ok(None)
    }
}

fn load_owner_actions(paths: &[String]) -> Result<Vec<OwnerModelReviewAction>, String> {
    let mut out = Vec::new();
    for path in paths {
        let mut actions: Vec<OwnerModelReviewAction> = read_json_file(Path::new(path))?;
        for action in &mut actions {
            let note = action.note.clone().unwrap_or_default();
            let lowered = note.to_ascii_lowercase();
            action.allowed = !lowered.contains("live")
                && !lowered.contains("runtime")
                && !lowered.contains("train")
                && !lowered.contains("broker")
                && !lowered.contains("order")
                && !lowered.contains("account");
            action.diagnostic_only = matches!(
                action.action_kind,
                OwnerModelReviewActionKind::MarkModelDiagnosticOnly
                    | OwnerModelReviewActionKind::RetireModelVersion
            );
            action.reason_codes = stable_reason_codes(&[ReasonCode::OwnerModelReviewWorkflowBuilt]);
        }
        out.extend(actions);
    }
    out.sort_by(|left, right| {
        (
            left.model_id.clone(),
            left.model_version.clone(),
            left.action_id.clone(),
        )
            .cmp(&(
                right.model_id.clone(),
                right.model_version.clone(),
                right.action_id.clone(),
            ))
    });
    Ok(out)
}

fn collect_model_keys(
    registry: &ExternalModelArtifactRegistry,
    leaderboard: &ConservativeExternalLeaderboard,
) -> Result<Vec<(String, String)>, String> {
    let mut keys = BTreeSet::new();
    for entry in &registry.entries {
        if is_tracked_model(&entry.model_id) {
            keys.insert((entry.model_id.clone(), entry.model_version.clone()));
        }
    }
    for entry in &leaderboard.entries {
        keys.insert((entry.model_id.clone(), entry.model_version.clone()));
    }
    let out = keys.into_iter().collect::<Vec<_>>();
    if out.is_empty() {
        return Err("no external models available for research ops".to_string());
    }
    Ok(out)
}

fn enforce_limits(
    config: &ExternalModelResearchOpsConfig,
    model_keys: &[(String, String)],
    owner_actions: &[OwnerModelReviewAction],
) -> Result<(), String> {
    let model_ids = model_keys
        .iter()
        .map(|(model_id, _)| model_id.clone())
        .collect::<BTreeSet<_>>();
    if model_ids.len() > config.max_models {
        return Err("max_models exceeded".to_string());
    }
    let mut versions_by_model: BTreeMap<String, usize> = BTreeMap::new();
    for (model_id, _) in model_keys {
        *versions_by_model.entry(model_id.clone()).or_default() += 1;
    }
    if versions_by_model
        .values()
        .any(|count| *count > config.max_versions)
    {
        return Err("max_versions exceeded".to_string());
    }
    if owner_actions.len() > config.max_review_items {
        return Err("max_review_items exceeded".to_string());
    }
    Ok(())
}

fn build_lifecycle_records(
    config: &ExternalModelResearchOpsConfig,
    registry: &ExternalModelArtifactRegistry,
    leaderboard: &ConservativeExternalLeaderboard,
    drift: &CalibrationDriftReport,
    owner_actions: &[OwnerModelReviewAction],
) -> Vec<ExternalModelLifecycleRecord> {
    let registry_map = build_registry_lookup(registry);
    let leaderboard_map = leaderboard
        .entries
        .iter()
        .map(|entry| (model_key(&entry.model_id, &entry.model_version), entry))
        .collect::<BTreeMap<_, _>>();
    let drift_map = drift
        .records
        .iter()
        .map(|record| {
            (
                model_key(&record.model_id, &record.model_version),
                format!("{:?}", record.calibration_status),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let watch_map = owner_actions
        .iter()
        .filter(|action| {
            matches!(
                action.action_kind,
                OwnerModelReviewActionKind::WatchModel
                    | OwnerModelReviewActionKind::MarkModelDiagnosticOnly
                    | OwnerModelReviewActionKind::RetireModelVersion
                    | OwnerModelReviewActionKind::RequestMorePredictions
                    | OwnerModelReviewActionKind::RequestCalibrationReview
                    | OwnerModelReviewActionKind::RequestRiskReview
            )
        })
        .map(|action| {
            (
                model_key(&action.model_id, &action.model_version),
                action.action_kind,
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut out = Vec::new();
    let model_keys = registry_map
        .keys()
        .chain(leaderboard_map.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for key in model_keys {
        let Some((model_id, model_version)) = key.split_once(':') else {
            continue;
        };
        let leaderboard_entry = leaderboard_map.get(&key).copied();
        let current_status = if matches!(
            watch_map.get(&key),
            Some(OwnerModelReviewActionKind::RetireModelVersion)
        ) && config.enable_retirement_policy
        {
            ExternalModelLifecycleStatus::Retired
        } else if matches!(
            watch_map.get(&key),
            Some(OwnerModelReviewActionKind::MarkModelDiagnosticOnly)
        ) {
            ExternalModelLifecycleStatus::DiagnosticOnly
        } else if matches!(
            watch_map.get(&key),
            Some(OwnerModelReviewActionKind::RequestMorePredictions)
        ) || leaderboard_entry
            .map(|entry| entry.coverage_ratio < 0.8)
            .unwrap_or(false)
        {
            ExternalModelLifecycleStatus::NeedsMorePredictions
        } else if matches!(
            watch_map.get(&key),
            Some(OwnerModelReviewActionKind::RequestCalibrationReview)
        ) || drift_map.get(&key).map(String::as_str) == Some("SevereDrift")
        {
            ExternalModelLifecycleStatus::NeedsCalibrationReview
        } else if matches!(
            watch_map.get(&key),
            Some(OwnerModelReviewActionKind::RequestRiskReview)
        ) || leaderboard_entry
            .and_then(|entry| entry.risk_adjusted_score)
            .map(|value| value < 0.0)
            .unwrap_or(false)
        {
            ExternalModelLifecycleStatus::NeedsRiskReview
        } else if matches!(
            watch_map.get(&key),
            Some(OwnerModelReviewActionKind::WatchModel)
        ) {
            ExternalModelLifecycleStatus::Watchlisted
        } else if leaderboard_entry.and_then(|entry| entry.promotion_status.as_deref())
            == Some("ResearchCandidate")
        {
            ExternalModelLifecycleStatus::ResearchCandidate
        } else if leaderboard_entry
            .map(|entry| is_blocked_entry_status(entry.entry_status))
            .unwrap_or(false)
        {
            ExternalModelLifecycleStatus::Blocked
        } else if registry_has_kind(
            registry_map.get(&key),
            ExternalModelArtifactKind::EvaluationReport,
        ) {
            ExternalModelLifecycleStatus::Evaluated
        } else if registry_has_kind(
            registry_map.get(&key),
            ExternalModelArtifactKind::ImportReport,
        ) {
            ExternalModelLifecycleStatus::Imported
        } else {
            ExternalModelLifecycleStatus::Registered
        };
        let previous_status = match current_status {
            ExternalModelLifecycleStatus::ResearchCandidate => {
                Some(ExternalModelLifecycleStatus::Evaluated)
            }
            ExternalModelLifecycleStatus::Evaluated => Some(ExternalModelLifecycleStatus::Imported),
            ExternalModelLifecycleStatus::Imported => {
                Some(ExternalModelLifecycleStatus::Registered)
            }
            ExternalModelLifecycleStatus::Watchlisted
            | ExternalModelLifecycleStatus::NeedsMorePredictions
            | ExternalModelLifecycleStatus::NeedsCalibrationReview
            | ExternalModelLifecycleStatus::NeedsRiskReview => {
                Some(ExternalModelLifecycleStatus::ResearchCandidate)
            }
            ExternalModelLifecycleStatus::DiagnosticOnly
            | ExternalModelLifecycleStatus::Downgraded => {
                Some(ExternalModelLifecycleStatus::ResearchCandidate)
            }
            ExternalModelLifecycleStatus::Retired => {
                Some(ExternalModelLifecycleStatus::DiagnosticOnly)
            }
            ExternalModelLifecycleStatus::Blocked => Some(ExternalModelLifecycleStatus::Evaluated),
            ExternalModelLifecycleStatus::Registered => None,
        };
        out.push(ExternalModelLifecycleRecord {
            model_id: model_id.to_string(),
            model_version: model_version.to_string(),
            current_status,
            previous_status,
            status_reason: lifecycle_reason(leaderboard_entry, current_status),
            allowed_transitions: allowed_transitions(current_status),
            forbidden_transitions: vec![
                "Live".to_string(),
                "RuntimeIntegrated".to_string(),
                "BrokerExecutable".to_string(),
            ],
            reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelLifecycleBuilt]),
        });
    }
    out.sort_by(|left, right| {
        (left.model_id.clone(), left.model_version.clone())
            .cmp(&(right.model_id.clone(), right.model_version.clone()))
    });
    out
}

fn build_review_queue(
    config: &ExternalModelResearchOpsConfig,
    lifecycle: &[ExternalModelLifecycleRecord],
    leaderboard: &ConservativeExternalLeaderboard,
    drift: &CalibrationDriftReport,
    risk_profiles: &[ModelEvidenceRiskProfile],
    changelog: &ModelLeaderboardChangeLog,
    owner_actions: &[OwnerModelReviewAction],
    registry: &ExternalModelArtifactRegistry,
) -> ExternalModelReviewQueue {
    let drift_map = drift
        .records
        .iter()
        .map(|record| {
            (
                model_key(&record.model_id, &record.model_version),
                record.calibration_status,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let change_map = changelog
        .changes
        .iter()
        .map(|change| {
            (
                model_key(&change.model_id, &change.model_version),
                change.change_kind,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let risk_map = risk_profiles
        .iter()
        .map(|profile| {
            (
                model_key(&profile.model_id, &profile.model_version),
                profile.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let owner_request_map = owner_actions
        .iter()
        .fold(BTreeMap::new(), |mut acc, action| {
            acc.entry(model_key(&action.model_id, &action.model_version))
                .or_insert_with(BTreeSet::new)
                .insert(action.action_kind);
            acc
        });

    let mut items = Vec::new();
    for record in lifecycle {
        let key = model_key(&record.model_id, &record.model_version);
        items.push(build_review_item(
            &record.model_id,
            &record.model_version,
            ExternalModelReviewItemKind::NewModelVersion,
            ExternalModelReviewItemStatus::Pending,
            "new or current model version requires research review".to_string(),
        ));

        if drift_map
            .get(&key)
            .copied()
            .unwrap_or(CalibrationDriftStatus::InsufficientHistory)
            != CalibrationDriftStatus::Stable
        {
            items.push(build_review_item(
                &record.model_id,
                &record.model_version,
                ExternalModelReviewItemKind::CalibrationDriftReview,
                ExternalModelReviewItemStatus::Pending,
                "calibration drift or insufficient history requires conservative review"
                    .to_string(),
            ));
        }

        if let Some(profile) = risk_map.get(&key) {
            if matches!(
                profile.recommended_action,
                ModelEvidenceRecommendedAction::RequestRiskReview
            ) {
                items.push(build_review_item(
                    &record.model_id,
                    &record.model_version,
                    ExternalModelReviewItemKind::RiskBehaviorReview,
                    ExternalModelReviewItemStatus::Pending,
                    "risk behavior requires explicit review".to_string(),
                ));
            }
            if matches!(
                profile.recommended_action,
                ModelEvidenceRecommendedAction::RequestMorePredictions
            ) {
                items.push(build_review_item(
                    &record.model_id,
                    &record.model_version,
                    ExternalModelReviewItemKind::CoverageWeakReview,
                    ExternalModelReviewItemStatus::Pending,
                    "coverage or evidence depth is weak".to_string(),
                ));
            }
        }

        if change_map
            .get(&key)
            .copied()
            .unwrap_or(LeaderboardChangeKind::NoChange)
            != LeaderboardChangeKind::NoChange
        {
            items.push(build_review_item(
                &record.model_id,
                &record.model_version,
                ExternalModelReviewItemKind::LeaderboardChangeReview,
                ExternalModelReviewItemStatus::Pending,
                "leaderboard rank or score changed".to_string(),
            ));
        }

        if record.model_id.contains("mamba") || registry_family_is_mamba(registry, &key) {
            items.push(build_review_item(
                &record.model_id,
                &record.model_version,
                ExternalModelReviewItemKind::MambaFamilyReview,
                ExternalModelReviewItemStatus::Pending,
                "Mamba3Fin family remains artifact-only and runtime-deferred".to_string(),
            ));
        }

        if owner_request_map
            .get(&key)
            .map(|kinds| {
                kinds.contains(&OwnerModelReviewActionKind::RequestMorePredictions)
                    || kinds.contains(&OwnerModelReviewActionKind::RequestCalibrationReview)
                    || kinds.contains(&OwnerModelReviewActionKind::RequestRiskReview)
            })
            .unwrap_or(false)
        {
            items.push(build_review_item(
                &record.model_id,
                &record.model_version,
                ExternalModelReviewItemKind::OwnerRequestedReview,
                ExternalModelReviewItemStatus::Pending,
                "owner requested additional conservative review".to_string(),
            ));
        }

        if matches!(record.current_status, ExternalModelLifecycleStatus::Retired) {
            items.push(build_review_item(
                &record.model_id,
                &record.model_version,
                ExternalModelReviewItemKind::RetirementReview,
                ExternalModelReviewItemStatus::Retired,
                "model version was retired from active research comparison".to_string(),
            ));
        }
    }

    items.sort_by(|left, right| {
        (
            left.model_id.clone(),
            left.model_version.clone(),
            left.review_id.clone(),
        )
            .cmp(&(
                right.model_id.clone(),
                right.model_version.clone(),
                right.review_id.clone(),
            ))
    });

    let pending_items = items
        .iter()
        .filter(|item| item.status == ExternalModelReviewItemStatus::Pending)
        .cloned()
        .take(config.max_review_items)
        .collect::<Vec<_>>();
    let reviewed_items = items
        .iter()
        .filter(|item| item.status == ExternalModelReviewItemStatus::Reviewed)
        .cloned()
        .collect::<Vec<_>>();
    let deferred_items = items
        .iter()
        .filter(|item| item.status == ExternalModelReviewItemStatus::Deferred)
        .cloned()
        .collect::<Vec<_>>();
    let retired_items = items
        .iter()
        .filter(|item| item.status == ExternalModelReviewItemStatus::Retired)
        .cloned()
        .collect::<Vec<_>>();
    let downgraded_items = items
        .iter()
        .filter(|item| item.status == ExternalModelReviewItemStatus::DowngradedToDiagnostic)
        .cloned()
        .collect::<Vec<_>>();
    let blocked_items = items
        .iter()
        .filter(|item| item.status == ExternalModelReviewItemStatus::Blocked)
        .cloned()
        .collect::<Vec<_>>();

    let _ = leaderboard;
    ExternalModelReviewQueue {
        queue_id: config.ops_id.clone(),
        pending_items,
        reviewed_items,
        deferred_items,
        retired_items,
        downgraded_items,
        blocked_items,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelReviewQueueBuilt]),
    }
}

fn build_owner_model_review_impact_report(
    config: &ExternalModelResearchOpsConfig,
    actions: &[OwnerModelReviewAction],
) -> OwnerModelReviewImpactReport {
    OwnerModelReviewImpactReport {
        report_id: config.ops_id.clone(),
        actions: actions.to_vec(),
        accepted_count: actions.iter().filter(|action| action.allowed).count(),
        blocked_count: actions.iter().filter(|action| !action.allowed).count(),
        retired_count: actions
            .iter()
            .filter(|action| action.action_kind == OwnerModelReviewActionKind::RetireModelVersion)
            .count(),
        downgraded_count: actions
            .iter()
            .filter(|action| {
                action.action_kind == OwnerModelReviewActionKind::MarkModelDiagnosticOnly
            })
            .count(),
        watchlisted_count: actions
            .iter()
            .filter(|action| action.action_kind == OwnerModelReviewActionKind::WatchModel)
            .count(),
        review_requested_count: actions
            .iter()
            .filter(|action| {
                matches!(
                    action.action_kind,
                    OwnerModelReviewActionKind::RequestMorePredictions
                        | OwnerModelReviewActionKind::RequestCalibrationReview
                        | OwnerModelReviewActionKind::RequestRiskReview
                )
            })
            .count(),
        reason_codes: stable_reason_codes(&[ReasonCode::OwnerModelReviewWorkflowBuilt]),
    }
}

fn build_watchlist(
    config: &ExternalModelResearchOpsConfig,
    actions: &[OwnerModelReviewAction],
) -> ExternalModelWatchlist {
    if !config.enable_watchlist {
        return ExternalModelWatchlist {
            entries: Vec::new(),
            active_count: 0,
            retired_count: 0,
            diagnostic_count: 0,
            reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelWatchlistBuilt]),
        };
    }
    let mut by_model = BTreeMap::new();
    for action in actions {
        let status = match action.action_kind {
            OwnerModelReviewActionKind::WatchModel => Some(ExternalModelWatchStatus::Active),
            OwnerModelReviewActionKind::UnwatchModel => Some(ExternalModelWatchStatus::Removed),
            OwnerModelReviewActionKind::RetireModelVersion => {
                Some(ExternalModelWatchStatus::Retired)
            }
            OwnerModelReviewActionKind::MarkModelDiagnosticOnly => {
                Some(ExternalModelWatchStatus::DiagnosticOnly)
            }
            _ => None,
        };
        if let Some(watch_status) = status {
            by_model.insert(
                model_key(&action.model_id, &action.model_version),
                ExternalModelWatchlistEntry {
                    model_id: action.model_id.clone(),
                    model_version: action.model_version.clone(),
                    watch_reason: action
                        .note
                        .clone()
                        .unwrap_or_else(|| format!("{:?}", action.action_kind)),
                    watched_by_owner: true,
                    watch_status,
                    reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelWatchlistBuilt]),
                },
            );
        }
    }
    let entries = by_model.into_values().collect::<Vec<_>>();
    ExternalModelWatchlist {
        active_count: entries
            .iter()
            .filter(|entry| entry.watch_status == ExternalModelWatchStatus::Active)
            .count(),
        retired_count: entries
            .iter()
            .filter(|entry| entry.watch_status == ExternalModelWatchStatus::Retired)
            .count(),
        diagnostic_count: entries
            .iter()
            .filter(|entry| entry.watch_status == ExternalModelWatchStatus::DiagnosticOnly)
            .count(),
        entries,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelWatchlistBuilt]),
    }
}

fn build_comparability_matrix(
    registry: &ExternalModelArtifactRegistry,
    leaderboard: &ConservativeExternalLeaderboard,
    lifecycle: &[ExternalModelLifecycleRecord],
) -> ModelComparabilityMatrix {
    let registry_map = build_registry_lookup(registry);
    let expected_dataset = registry
        .entries
        .iter()
        .find(|entry| entry.artifact_kind == ExternalModelArtifactKind::SequenceExportManifest)
        .and_then(|entry| entry.dataset_fingerprint.clone())
        .unwrap_or_default();
    let expected_feature = registry
        .entries
        .iter()
        .find(|entry| entry.artifact_kind == ExternalModelArtifactKind::SequenceExportManifest)
        .and_then(|entry| entry.feature_schema_hash.clone())
        .unwrap_or_default();
    let expected_label = registry
        .entries
        .iter()
        .find(|entry| entry.artifact_kind == ExternalModelArtifactKind::SequenceExportManifest)
        .and_then(|entry| entry.label_manifest_hash.clone())
        .unwrap_or_default();
    let expected_split = "ChronologicalHoldout".to_string();
    let expected_prediction_schema = "v2".to_string();

    let leaderboard_map = leaderboard
        .entries
        .iter()
        .map(|entry| (model_key(&entry.model_id, &entry.model_version), entry))
        .collect::<BTreeMap<_, _>>();

    let mut cells = Vec::new();
    let mut comparable_models = 0usize;
    let mut non_comparable_models = 0usize;

    for record in lifecycle {
        let key = model_key(&record.model_id, &record.model_version);
        let entries = registry_map.get(&key);
        let leaderboard_entry = leaderboard_map.get(&key);
        let model_cells = vec![
            comparability_cell(
                &record.model_id,
                &record.model_version,
                ModelComparabilityDimension::DatasetFingerprint,
                entry_value(entries, |entry| entry.dataset_fingerprint.clone()),
                Some(expected_dataset.clone()),
            ),
            comparability_cell(
                &record.model_id,
                &record.model_version,
                ModelComparabilityDimension::FeatureSchemaHash,
                entry_value(entries, |entry| entry.feature_schema_hash.clone()),
                Some(expected_feature.clone()),
            ),
            comparability_cell(
                &record.model_id,
                &record.model_version,
                ModelComparabilityDimension::LabelManifestHash,
                entry_value(entries, |entry| entry.label_manifest_hash.clone()),
                Some(expected_label.clone()),
            ),
            comparability_cell(
                &record.model_id,
                &record.model_version,
                ModelComparabilityDimension::SplitPolicy,
                entry_value(entries, |entry| entry.split_policy.clone()),
                Some(expected_split.clone()),
            ),
            comparability_cell(
                &record.model_id,
                &record.model_version,
                ModelComparabilityDimension::PredictionSchemaVersion,
                entry_value(entries, |entry| entry.prediction_schema_version.clone()),
                Some(expected_prediction_schema.clone()),
            ),
            ModelComparabilityCell {
                model_id: record.model_id.clone(),
                model_version: record.model_version.clone(),
                dimension: ModelComparabilityDimension::ModelCardValidity,
                comparable: registry_has_kind(entries, ExternalModelArtifactKind::ModelCard),
                value: Some(
                    registry_has_kind(entries, ExternalModelArtifactKind::ModelCard).to_string(),
                ),
                expected_value: Some("true".to_string()),
                reason_codes: stable_reason_codes(&[ReasonCode::ModelComparabilityMatrixBuilt]),
            },
            ModelComparabilityCell {
                model_id: record.model_id.clone(),
                model_version: record.model_version.clone(),
                dimension: ModelComparabilityDimension::EvaluationMetricAvailability,
                comparable: leaderboard_entry
                    .and_then(|entry| entry.brier_score)
                    .is_some()
                    && leaderboard_entry
                        .and_then(|entry| entry.risk_adjusted_score)
                        .is_some(),
                value: Some(
                    (leaderboard_entry
                        .and_then(|entry| entry.brier_score)
                        .is_some()
                        && leaderboard_entry
                            .and_then(|entry| entry.risk_adjusted_score)
                            .is_some())
                    .to_string(),
                ),
                expected_value: Some("true".to_string()),
                reason_codes: stable_reason_codes(&[ReasonCode::ModelComparabilityMatrixBuilt]),
            },
            ModelComparabilityCell {
                model_id: record.model_id.clone(),
                model_version: record.model_version.clone(),
                dimension: ModelComparabilityDimension::CoverageRatio,
                comparable: leaderboard_entry
                    .map(|entry| entry.coverage_ratio >= 0.8)
                    .unwrap_or(false),
                value: leaderboard_entry.map(|entry| format!("{:.4}", entry.coverage_ratio)),
                expected_value: Some(">=0.8000".to_string()),
                reason_codes: stable_reason_codes(&[ReasonCode::ModelComparabilityMatrixBuilt]),
            },
            ModelComparabilityCell {
                model_id: record.model_id.clone(),
                model_version: record.model_version.clone(),
                dimension: ModelComparabilityDimension::CalibrationMetricAvailability,
                comparable: leaderboard_entry.and_then(|entry| entry.ece).is_some()
                    && leaderboard_entry
                        .and_then(|entry| entry.brier_score)
                        .is_some(),
                value: Some(
                    (leaderboard_entry.and_then(|entry| entry.ece).is_some()
                        && leaderboard_entry
                            .and_then(|entry| entry.brier_score)
                            .is_some())
                    .to_string(),
                ),
                expected_value: Some("true".to_string()),
                reason_codes: stable_reason_codes(&[ReasonCode::ModelComparabilityMatrixBuilt]),
            },
            ModelComparabilityCell {
                model_id: record.model_id.clone(),
                model_version: record.model_version.clone(),
                dimension: ModelComparabilityDimension::RiskMetricAvailability,
                comparable: leaderboard_entry
                    .and_then(|entry| entry.risk_adjusted_score)
                    .is_some(),
                value: Some(
                    leaderboard_entry
                        .and_then(|entry| entry.risk_adjusted_score)
                        .is_some()
                        .to_string(),
                ),
                expected_value: Some("true".to_string()),
                reason_codes: stable_reason_codes(&[ReasonCode::ModelComparabilityMatrixBuilt]),
            },
            ModelComparabilityCell {
                model_id: record.model_id.clone(),
                model_version: record.model_version.clone(),
                dimension: ModelComparabilityDimension::AblationAvailability,
                comparable: leaderboard_entry
                    .and_then(|entry| entry.ablation_status.clone())
                    .is_some(),
                value: Some(
                    leaderboard_entry
                        .and_then(|entry| entry.ablation_status.clone())
                        .is_some()
                        .to_string(),
                ),
                expected_value: Some("true".to_string()),
                reason_codes: stable_reason_codes(&[ReasonCode::ModelComparabilityMatrixBuilt]),
            },
            ModelComparabilityCell {
                model_id: record.model_id.clone(),
                model_version: record.model_version.clone(),
                dimension: ModelComparabilityDimension::PromotionGateAvailability,
                comparable: leaderboard_entry
                    .and_then(|entry| entry.promotion_status.clone())
                    .is_some(),
                value: Some(
                    leaderboard_entry
                        .and_then(|entry| entry.promotion_status.clone())
                        .is_some()
                        .to_string(),
                ),
                expected_value: Some("true".to_string()),
                reason_codes: stable_reason_codes(&[ReasonCode::ModelComparabilityMatrixBuilt]),
            },
        ];
        if model_cells.iter().all(|cell| cell.comparable) {
            comparable_models += 1;
        } else {
            non_comparable_models += 1;
        }
        cells.extend(model_cells);
    }

    let matrix_status = if comparable_models > 0 && non_comparable_models == 0 {
        ModelComparabilityMatrixStatus::FullyComparable
    } else if comparable_models > 0 {
        ModelComparabilityMatrixStatus::PartiallyComparable
    } else if cells.is_empty() {
        ModelComparabilityMatrixStatus::DiagnosticOnly
    } else {
        ModelComparabilityMatrixStatus::NotComparable
    };

    ModelComparabilityMatrix {
        cells,
        comparable_models,
        non_comparable_models,
        matrix_status,
        reason_codes: stable_reason_codes(&[ReasonCode::ModelComparabilityMatrixBuilt]),
    }
}

fn build_artifact_completeness_scores(
    registry: &ExternalModelArtifactRegistry,
    audit: &ExternalArtifactRegistryAuditReport,
    model_keys: &[(String, String)],
) -> Vec<ArtifactCompletenessScore> {
    let registry_map = build_registry_lookup(registry);
    model_keys
        .iter()
        .map(|(model_id, model_version)| {
            let key = model_key(model_id, model_version);
            let entries = registry_map.get(&key);
            let has_model_card = registry_has_kind(entries, ExternalModelArtifactKind::ModelCard);
            let has_prediction_csv =
                registry_has_kind(entries, ExternalModelArtifactKind::PredictionCsv);
            let has_import_report =
                registry_has_kind(entries, ExternalModelArtifactKind::ImportReport);
            let has_evaluation_report =
                registry_has_kind(entries, ExternalModelArtifactKind::EvaluationReport);
            let has_vs_trinity_report =
                registry_has_kind(entries, ExternalModelArtifactKind::VsTrinityReport);
            let has_ablation_report =
                registry_has_kind(entries, ExternalModelArtifactKind::AblationReport);
            let has_promotion_gate =
                registry_has_kind(entries, ExternalModelArtifactKind::PromotionGateReport);
            let has_registry_audit = matches!(
                audit.audit_status,
                ExternalArtifactRegistryAuditStatus::Passed
                    | ExternalArtifactRegistryAuditStatus::PassedWithWarnings
            );
            let mamba_applicable = entries
                .into_iter()
                .flat_map(|items| items.iter())
                .any(|entry| entry.model_family.contains("Mamba3FinLite"));
            let has_mamba_contract_if_applicable = if mamba_applicable {
                registry.entries.iter().any(|entry| {
                    entry.artifact_kind == ExternalModelArtifactKind::Mamba3FinContract
                })
            } else {
                true
            };
            let completeness_flags = [
                has_model_card,
                has_prediction_csv,
                has_import_report,
                has_evaluation_report,
                has_vs_trinity_report,
                has_ablation_report,
                has_promotion_gate,
                has_registry_audit,
                has_mamba_contract_if_applicable,
            ];
            let completeness_ratio = completeness_flags.iter().filter(|flag| **flag).count() as f64
                / completeness_flags.len() as f64;
            let completeness_status =
                if !has_model_card || !has_prediction_csv || !has_evaluation_report {
                    ArtifactCompletenessStatus::MissingCriticalArtifacts
                } else if (completeness_ratio - 1.0).abs() < f64::EPSILON {
                    ArtifactCompletenessStatus::Complete
                } else if completeness_ratio >= 0.75 {
                    ArtifactCompletenessStatus::MostlyComplete
                } else {
                    ArtifactCompletenessStatus::Incomplete
                };
            ArtifactCompletenessScore {
                model_id: model_id.clone(),
                model_version: model_version.clone(),
                has_model_card,
                has_prediction_csv,
                has_import_report,
                has_evaluation_report,
                has_vs_trinity_report,
                has_ablation_report,
                has_promotion_gate,
                has_registry_audit,
                has_mamba_contract_if_applicable,
                completeness_ratio,
                completeness_status,
                reason_codes: stable_reason_codes(&[ReasonCode::ArtifactCompletenessScoreBuilt]),
            }
        })
        .collect()
}

fn build_model_evidence_risk_profiles(
    leaderboard: &ConservativeExternalLeaderboard,
    drift: &CalibrationDriftReport,
    lifecycle: &[ExternalModelLifecycleRecord],
) -> Vec<ModelEvidenceRiskProfile> {
    let drift_map = drift
        .records
        .iter()
        .map(|record| {
            (
                model_key(&record.model_id, &record.model_version),
                format!("{:?}", record.calibration_status),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let lifecycle_map = lifecycle
        .iter()
        .map(|record| {
            (
                model_key(&record.model_id, &record.model_version),
                record.current_status,
            )
        })
        .collect::<BTreeMap<_, _>>();
    leaderboard
        .entries
        .iter()
        .map(|entry| {
            let key = model_key(&entry.model_id, &entry.model_version);
            let drift_status = drift_map
                .get(&key)
                .cloned()
                .unwrap_or_else(|| "InsufficientHistory".to_string());
            let ablation_unstable = entry
                .ablation_status
                .as_deref()
                .map(|status| status.contains("Unstable"))
                .unwrap_or(false);
            let sequence_size_warning = entry.rank.is_none() && entry.coverage_ratio < 1.0;
            let small_sample_warning = entry.coverage_ratio < 0.8;
            let evidence_risk_level = if matches!(
                lifecycle_map.get(&key),
                Some(
                    ExternalModelLifecycleStatus::DiagnosticOnly
                        | ExternalModelLifecycleStatus::Retired
                )
            ) {
                ModelEvidenceRiskLevel::DiagnosticOnly
            } else if entry.coverage_ratio < 0.8
                || drift_status == "SevereDrift"
                || is_blocked_entry_status(entry.entry_status)
            {
                ModelEvidenceRiskLevel::Critical
            } else if entry.ece.unwrap_or(1.0) > 0.3
                || entry.risk_adjusted_score.unwrap_or(-1.0) < 0.0
                || ablation_unstable
            {
                ModelEvidenceRiskLevel::High
            } else if sequence_size_warning
                || small_sample_warning
                || drift_status == "InsufficientHistory"
            {
                ModelEvidenceRiskLevel::Medium
            } else {
                ModelEvidenceRiskLevel::Low
            };
            let recommended_action = if entry.coverage_ratio < 0.8 {
                ModelEvidenceRecommendedAction::RequestMorePredictions
            } else if drift_status == "SevereDrift" || entry.ece.unwrap_or(0.0) > 0.3 {
                ModelEvidenceRecommendedAction::RequestCalibrationReview
            } else if entry.risk_adjusted_score.unwrap_or(-1.0) < 0.0 || ablation_unstable {
                ModelEvidenceRecommendedAction::RequestRiskReview
            } else if matches!(
                lifecycle_map.get(&key),
                Some(ExternalModelLifecycleStatus::DiagnosticOnly)
            ) {
                ModelEvidenceRecommendedAction::DowngradeToDiagnostic
            } else {
                ModelEvidenceRecommendedAction::KeepResearchCandidate
            };
            ModelEvidenceRiskProfile {
                model_id: entry.model_id.clone(),
                model_version: entry.model_version.clone(),
                coverage_ratio: format!("{:.4}", entry.coverage_ratio),
                calibration_status: entry
                    .ece
                    .map(|ece| if ece <= 0.3 { "Stable" } else { "Review" })
                    .unwrap_or("Unknown")
                    .to_string(),
                drift_status,
                risk_status: if entry.risk_adjusted_score.unwrap_or(-1.0) >= 0.0 {
                    "Pass".to_string()
                } else {
                    "Review".to_string()
                },
                ablation_status: entry
                    .ablation_status
                    .clone()
                    .unwrap_or_else(|| "Missing".to_string()),
                promotion_gate_status: entry
                    .promotion_status
                    .clone()
                    .unwrap_or_else(|| "Missing".to_string()),
                leaderboard_status: format!("{:?}", entry.entry_status),
                sequence_size_warning,
                small_sample_warning,
                evidence_risk_level,
                recommended_action,
                reason_codes: stable_reason_codes(&[ReasonCode::ModelEvidenceRiskProfileBuilt]),
            }
        })
        .collect()
}

fn build_model_leaderboard_changelog(
    leaderboard: &ConservativeExternalLeaderboard,
    version_comparison: &ExternalModelVersionComparisonReport,
    drift: &CalibrationDriftReport,
) -> ModelLeaderboardChangeLog {
    let rank_by_key = leaderboard
        .entries
        .iter()
        .filter_map(|entry| {
            entry
                .rank
                .map(|rank| (model_key(&entry.model_id, &entry.model_version), rank))
        })
        .collect::<BTreeMap<_, _>>();
    let drift_key = model_key(
        &version_comparison.model_id,
        &version_comparison.latest_version,
    );
    let drift_changed = drift.records.iter().any(|record| {
        model_key(&record.model_id, &record.model_version) == drift_key
            && record.calibration_status != CalibrationDriftStatus::Stable
    });

    let mut changes = Vec::new();
    for entry in &leaderboard.entries {
        let previous_rank = if entry.model_id == version_comparison.model_id {
            version_comparison
                .previous_version
                .as_ref()
                .and_then(|version| {
                    rank_by_key
                        .get(&model_key(&entry.model_id, version))
                        .copied()
                })
        } else {
            None
        };
        let current_rank = entry.rank;
        let change_kind = if is_blocked_entry_status(entry.entry_status) {
            LeaderboardChangeKind::NewlyBlocked
        } else if entry.model_id == version_comparison.model_id
            && entry.model_version == version_comparison.latest_version
        {
            match (previous_rank, current_rank) {
                (Some(previous), Some(current)) if current < previous => {
                    LeaderboardChangeKind::RankUp
                }
                (Some(previous), Some(current)) if current > previous => {
                    LeaderboardChangeKind::RankDown
                }
                (None, Some(_)) => LeaderboardChangeKind::NewlyEligible,
                (_, Some(_)) if drift_changed => LeaderboardChangeKind::DriftChanged,
                (_, Some(_))
                    if matches!(
                        version_comparison.comparison_status,
                        ExternalModelVersionComparisonStatus::Improved
                            | ExternalModelVersionComparisonStatus::Mixed
                            | ExternalModelVersionComparisonStatus::Regressed
                    ) =>
                {
                    LeaderboardChangeKind::ScoreChanged
                }
                _ => LeaderboardChangeKind::NoChange,
            }
        } else {
            LeaderboardChangeKind::NoChange
        };
        let score_delta = if entry.model_id == version_comparison.model_id
            && entry.model_version == version_comparison.latest_version
        {
            version_comparison
                .risk_delta_summary
                .get("risk:risk_adjusted_score")
                .copied()
        } else {
            None
        };
        changes.push(ModelLeaderboardChange {
            model_id: entry.model_id.clone(),
            model_version: entry.model_version.clone(),
            previous_rank,
            current_rank,
            change_kind,
            score_delta,
            reason_codes: stable_reason_codes(&[ReasonCode::ModelLeaderboardChangeLogBuilt]),
        });
    }
    ModelLeaderboardChangeLog {
        newly_eligible_count: changes
            .iter()
            .filter(|change| change.change_kind == LeaderboardChangeKind::NewlyEligible)
            .count(),
        newly_blocked_count: changes
            .iter()
            .filter(|change| change.change_kind == LeaderboardChangeKind::NewlyBlocked)
            .count(),
        rank_change_count: changes
            .iter()
            .filter(|change| {
                matches!(
                    change.change_kind,
                    LeaderboardChangeKind::RankUp | LeaderboardChangeKind::RankDown
                )
            })
            .count(),
        no_change_count: changes
            .iter()
            .filter(|change| change.change_kind == LeaderboardChangeKind::NoChange)
            .count(),
        changes,
        reason_codes: stable_reason_codes(&[ReasonCode::ModelLeaderboardChangeLogBuilt]),
    }
}

fn build_external_model_research_ops_report(
    config: &ExternalModelResearchOpsConfig,
    lifecycle: &[ExternalModelLifecycleRecord],
    review_queue: &ExternalModelReviewQueue,
    watchlist: &ExternalModelWatchlist,
    comparability: &ModelComparabilityMatrix,
    completeness: &[ArtifactCompletenessScore],
    risk_profiles: &[ModelEvidenceRiskProfile],
    changelog: &ModelLeaderboardChangeLog,
) -> ExternalModelResearchOpsReport {
    let has_missing_critical_artifacts = completeness.iter().any(|score| {
        score.completeness_status == ArtifactCompletenessStatus::MissingCriticalArtifacts
    });
    let has_coverage_gap = risk_profiles.iter().any(|profile| {
        profile.recommended_action == ModelEvidenceRecommendedAction::RequestMorePredictions
    });
    let lifecycle_summary = format!(
        "registered={} research_candidate={} watchlisted={} retired={}",
        lifecycle
            .iter()
            .filter(|record| record.current_status == ExternalModelLifecycleStatus::Registered)
            .count(),
        lifecycle
            .iter()
            .filter(
                |record| record.current_status == ExternalModelLifecycleStatus::ResearchCandidate
            )
            .count(),
        lifecycle
            .iter()
            .filter(|record| record.current_status == ExternalModelLifecycleStatus::Watchlisted)
            .count(),
        lifecycle
            .iter()
            .filter(|record| record.current_status == ExternalModelLifecycleStatus::Retired)
            .count(),
    );
    let review_queue_summary = format!(
        "pending={} retired={} blocked={}",
        review_queue.pending_items.len(),
        review_queue.retired_items.len(),
        review_queue.blocked_items.len(),
    );
    let watchlist_summary = format!(
        "active={} retired={} diagnostic={}",
        watchlist.active_count, watchlist.retired_count, watchlist.diagnostic_count
    );
    let artifact_completeness_summary = format!(
        "complete={} mostly_complete={} missing_critical={}",
        completeness
            .iter()
            .filter(|score| score.completeness_status == ArtifactCompletenessStatus::Complete)
            .count(),
        completeness
            .iter()
            .filter(|score| score.completeness_status == ArtifactCompletenessStatus::MostlyComplete)
            .count(),
        completeness
            .iter()
            .filter(|score| {
                score.completeness_status == ArtifactCompletenessStatus::MissingCriticalArtifacts
            })
            .count(),
    );
    let evidence_risk_profile_summary = format!(
        "low={} medium={} high={} critical={}",
        risk_profiles
            .iter()
            .filter(|profile| profile.evidence_risk_level == ModelEvidenceRiskLevel::Low)
            .count(),
        risk_profiles
            .iter()
            .filter(|profile| profile.evidence_risk_level == ModelEvidenceRiskLevel::Medium)
            .count(),
        risk_profiles
            .iter()
            .filter(|profile| profile.evidence_risk_level == ModelEvidenceRiskLevel::High)
            .count(),
        risk_profiles
            .iter()
            .filter(|profile| profile.evidence_risk_level == ModelEvidenceRiskLevel::Critical)
            .count(),
    );
    let leaderboard_change_summary = format!(
        "newly_eligible={} newly_blocked={} rank_changes={}",
        changelog.newly_eligible_count, changelog.newly_blocked_count, changelog.rank_change_count
    );
    let final_status = if has_missing_critical_artifacts
        || comparability.matrix_status == ModelComparabilityMatrixStatus::NotComparable
    {
        ExternalModelResearchOpsStatus::NeedBetterCoverage
    } else if has_coverage_gap {
        ExternalModelResearchOpsStatus::NeedMorePredictionHistory
    } else if risk_profiles.iter().any(|profile| {
        profile.recommended_action == ModelEvidenceRecommendedAction::RequestCalibrationReview
    }) {
        ExternalModelResearchOpsStatus::NeedBetterCalibration
    } else if risk_profiles.iter().any(|profile| {
        profile.recommended_action == ModelEvidenceRecommendedAction::RequestRiskReview
    }) {
        ExternalModelResearchOpsStatus::NeedBetterRiskBehavior
    } else if !review_queue.pending_items.is_empty() {
        ExternalModelResearchOpsStatus::NeedOwnerReview
    } else if comparability.matrix_status == ModelComparabilityMatrixStatus::DiagnosticOnly {
        ExternalModelResearchOpsStatus::DiagnosticOnly
    } else {
        ExternalModelResearchOpsStatus::ModelResearchOpsReady
    };
    let final_recommendation = match final_status {
        ExternalModelResearchOpsStatus::NeedMorePredictionHistory => {
            ExternalModelResearchOpsRecommendation::RequestMorePredictions
        }
        ExternalModelResearchOpsStatus::NeedBetterCoverage => {
            ExternalModelResearchOpsRecommendation::RequestMorePredictions
        }
        ExternalModelResearchOpsStatus::NeedBetterCalibration => {
            ExternalModelResearchOpsRecommendation::ImproveCalibration
        }
        ExternalModelResearchOpsStatus::NeedBetterRiskBehavior => {
            ExternalModelResearchOpsRecommendation::ImproveRiskBehavior
        }
        ExternalModelResearchOpsStatus::NeedOwnerReview => {
            ExternalModelResearchOpsRecommendation::NeedMoreEvidence
        }
        ExternalModelResearchOpsStatus::DiagnosticOnly => {
            ExternalModelResearchOpsRecommendation::HoldMamba3RuntimeDeferred
        }
        ExternalModelResearchOpsStatus::ModelResearchOpsReady => {
            ExternalModelResearchOpsRecommendation::KeepResearchCandidate
        }
    };
    ExternalModelResearchOpsReport {
        ops_id: config.ops_id.clone(),
        lifecycle_summary,
        review_queue_summary,
        watchlist_summary,
        comparability_matrix_status: format!("{:?}", comparability.matrix_status),
        artifact_completeness_summary,
        evidence_risk_profile_summary,
        leaderboard_change_summary,
        final_status,
        final_recommendation,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelResearchOpsReportBuilt]),
    }
}

fn build_control_tower_model_ops_panel(
    report: &ExternalModelResearchOpsReport,
    review_queue: &ExternalModelReviewQueue,
    watchlist: &ExternalModelWatchlist,
    comparability: &ModelComparabilityMatrix,
    completeness: &[ArtifactCompletenessScore],
    risk_profiles: &[ModelEvidenceRiskProfile],
    changelog: &ModelLeaderboardChangeLog,
    leaderboard: &ConservativeExternalLeaderboard,
) -> ControlTowerModelOpsPanel {
    let mut panel = ControlTowerModelOpsPanel {
        research_ops_status: format!("{:?}", report.final_status),
        review_queue_summary: report.review_queue_summary.clone(),
        watchlist: watchlist
            .entries
            .iter()
            .map(|entry| {
                format!(
                    "{}:{}:{:?}",
                    entry.model_id, entry.model_version, entry.watch_status
                )
            })
            .collect(),
        comparability_status: format!("{:?}", comparability.matrix_status),
        artifact_completeness_status: format!(
            "mostly_complete={} missing_critical={}",
            completeness
                .iter()
                .filter(|score| score.completeness_status == ArtifactCompletenessStatus::MostlyComplete)
                .count(),
            completeness
                .iter()
                .filter(|score| {
                    score.completeness_status == ArtifactCompletenessStatus::MissingCriticalArtifacts
                })
                .count(),
        ),
        model_risk_profiles: risk_profiles
            .iter()
            .map(|profile| {
                format!(
                    "{}:{}:{:?}:{:?}",
                    profile.model_id,
                    profile.model_version,
                    profile.evidence_risk_level,
                    profile.recommended_action
                )
            })
            .collect(),
        leaderboard_changes: changelog
            .changes
            .iter()
            .map(|change| {
                format!(
                    "{}:{}:{:?}",
                    change.model_id, change.model_version, change.change_kind
                )
            })
            .collect(),
        mamba3fin_family_status: format!(
            "{} + RuntimeDeferred",
            leaderboard
                .entries
                .iter()
                .find(|entry| entry.model_family.contains("Mamba3FinLite"))
                .map(|_| "ArtifactFamilyTracked")
                .unwrap_or("NoMambaFamily")
        ),
        next_actions: vec![
            "cargo run --quiet --bin soma_experiment -- artifact-completeness --config examples/soma_artifact_completeness.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- external-model-research-ops --config examples/soma_external_model_research_ops.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- external-model-review-queue --config examples/soma_external_model_review_queue.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- external-model-watchlist --config examples/soma_external_model_watchlist.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- model-comparability-matrix --config examples/soma_model_comparability_matrix.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- model-leaderboard-changelog --config examples/soma_model_leaderboard_changelog.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- model-risk-profile --config examples/soma_model_risk_profile.toml".to_string(),
            format!("pending_review_items={}", review_queue.pending_items.len()),
        ],
        reason_codes: stable_reason_codes(&[ReasonCode::ControlTowerModelOpsPanelBuilt]),
    };
    panel.stabilize();
    panel
}

fn build_storage_report(
    config: &ExternalModelResearchOpsConfig,
    registry: &ExternalModelArtifactRegistry,
    owner_actions: &[OwnerModelReviewAction],
) -> Result<ExternalModelResearchOpsStorageReport, String> {
    let mut artifact_count = 0usize;
    let mut storage_bytes = 0usize;
    for path in config
        .external_artifact_registry_paths
        .iter()
        .chain(config.conservative_leaderboard_paths.iter())
        .chain(config.calibration_drift_paths.iter())
        .chain(config.evaluation_history_paths.iter())
        .chain(config.model_version_comparison_paths.iter())
        .chain(config.previous_external_comparison_paths.iter())
        .chain(config.external_registry_audit_paths.iter())
        .chain(config.owner_model_review_paths.iter())
    {
        artifact_count += 1;
        storage_bytes += fs::metadata(path).map_err(|err| err.to_string())?.len() as usize;
    }
    artifact_count += registry.entries.len();
    storage_bytes += owner_actions.len() * 64;
    Ok(ExternalModelResearchOpsStorageReport {
        artifact_count,
        storage_bytes,
        max_bytes: config.max_bytes,
        within_budget: storage_bytes <= config.max_bytes,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelResearchOpsStorageBuilt]),
    })
}

fn build_final_summary(
    report: &ExternalModelResearchOpsReport,
    review_queue: &ExternalModelReviewQueue,
    watchlist: &ExternalModelWatchlist,
) -> String {
    [
        format!("research_ops_status={:?}", report.final_status),
        format!("recommendation={:?}", report.final_recommendation),
        format!("pending_reviews={}", review_queue.pending_items.len()),
        format!("active_watchlist={}", watchlist.active_count),
        "runtime_status=HoldMamba3RuntimeDeferred".to_string(),
    ]
    .join("\n")
}

fn write_bundle(
    config: &ExternalModelResearchOpsConfig,
    bundle: &ExternalModelResearchOpsBundle,
) -> Result<(), String> {
    let dir = config.output_dir();
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    write_text_report(
        &dir,
        "external_model_lifecycle.txt",
        &bundle.lifecycle_records,
    )?;
    write_text_report(
        &dir,
        "external_model_review_queue.txt",
        &bundle.external_model_review_queue,
    )?;
    if let Some(report) = &bundle.owner_model_review_impact_report {
        write_text_report(&dir, "owner_model_review_impact.txt", report)?;
    }
    write_text_report(
        &dir,
        "external_model_watchlist.txt",
        &bundle.external_model_watchlist,
    )?;
    write_text_report(
        &dir,
        "model_comparability_matrix.txt",
        &bundle.model_comparability_matrix,
    )?;
    write_text_report(
        &dir,
        "artifact_completeness_scores.txt",
        &bundle.artifact_completeness_scores,
    )?;
    write_text_report(
        &dir,
        "model_evidence_risk_profiles.txt",
        &bundle.model_evidence_risk_profiles,
    )?;
    write_text_report(
        &dir,
        "model_leaderboard_changelog.txt",
        &bundle.model_leaderboard_changelog,
    )?;
    write_text_report(
        &dir,
        "external_model_research_ops_report.txt",
        &bundle.external_model_research_ops_report,
    )?;
    if let Some(panel) = &bundle.control_tower_model_ops_panel_summary {
        fs::write(
            dir.join("control_tower_model_ops_panel.txt"),
            panel.to_text(),
        )
        .map_err(|err| err.to_string())?;
    }
    write_text_report(&dir, "storage_report.txt", &bundle.storage_report)?;
    fs::write(dir.join("summary.txt"), &bundle.final_summary).map_err(|err| err.to_string())?;
    Ok(())
}

fn build_registry_lookup(
    registry: &ExternalModelArtifactRegistry,
) -> BTreeMap<String, Vec<&super::external_artifact_registry::ExternalModelRegistryEntry>> {
    let mut out = BTreeMap::new();
    for entry in &registry.entries {
        if is_tracked_model(&entry.model_id) {
            out.entry(model_key(&entry.model_id, &entry.model_version))
                .or_insert_with(Vec::new)
                .push(entry);
        }
    }
    out
}

fn registry_has_kind(
    entries: Option<&Vec<&super::external_artifact_registry::ExternalModelRegistryEntry>>,
    kind: ExternalModelArtifactKind,
) -> bool {
    entries
        .map(|items| items.iter().any(|entry| entry.artifact_kind == kind))
        .unwrap_or(false)
}

fn entry_value(
    entries: Option<&Vec<&super::external_artifact_registry::ExternalModelRegistryEntry>>,
    extract: impl Fn(&super::external_artifact_registry::ExternalModelRegistryEntry) -> Option<String>,
) -> Option<String> {
    entries.and_then(|items| items.iter().find_map(|entry| extract(entry)))
}

fn registry_family_is_mamba(registry: &ExternalModelArtifactRegistry, key: &str) -> bool {
    registry.entries.iter().any(|entry| {
        model_key(&entry.model_id, &entry.model_version) == key
            && entry.model_family.contains("Mamba3FinLite")
    })
}

fn build_review_item(
    model_id: &str,
    model_version: &str,
    item_kind: ExternalModelReviewItemKind,
    status: ExternalModelReviewItemStatus,
    summary: String,
) -> ExternalModelReviewItem {
    ExternalModelReviewItem {
        review_id: format!("{model_id}:{model_version}:{item_kind:?}"),
        model_id: model_id.to_string(),
        model_version: model_version.to_string(),
        item_kind,
        status,
        summary,
        recommended_actions: stable_ordered_strings(&[
            "AddModelNote".to_string(),
            "KeepResearchCandidate".to_string(),
            "MarkModelDiagnosticOnly".to_string(),
            "RequestCalibrationReview".to_string(),
            "RequestMorePredictions".to_string(),
            "RequestRiskReview".to_string(),
            "RetireModelVersion".to_string(),
            "WatchModel".to_string(),
        ]),
        forbidden_actions: stable_ordered_strings(&[
            "BrokerIntegration".to_string(),
            "BypassRiskGovernor".to_string(),
            "EnableRuntimeInference".to_string(),
            "EnableTraining".to_string(),
            "ExecuteTrade".to_string(),
            "PromoteToLive".to_string(),
        ]),
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelReviewQueueBuilt]),
    }
}

fn allowed_transitions(status: ExternalModelLifecycleStatus) -> Vec<String> {
    match status {
        ExternalModelLifecycleStatus::Registered => {
            vec!["Imported".to_string(), "Blocked".to_string()]
        }
        ExternalModelLifecycleStatus::Imported => {
            vec!["Evaluated".to_string(), "DiagnosticOnly".to_string()]
        }
        ExternalModelLifecycleStatus::Evaluated => {
            vec![
                "ResearchCandidate".to_string(),
                "DiagnosticOnly".to_string(),
                "Watchlisted".to_string(),
            ]
        }
        ExternalModelLifecycleStatus::ResearchCandidate => {
            vec![
                "Watchlisted".to_string(),
                "DiagnosticOnly".to_string(),
                "Retired".to_string(),
            ]
        }
        ExternalModelLifecycleStatus::Watchlisted => {
            vec![
                "NeedsMorePredictions".to_string(),
                "NeedsCalibrationReview".to_string(),
                "NeedsRiskReview".to_string(),
            ]
        }
        ExternalModelLifecycleStatus::DiagnosticOnly | ExternalModelLifecycleStatus::Downgraded => {
            vec!["Retired".to_string(), "Watchlisted".to_string()]
        }
        ExternalModelLifecycleStatus::NeedsMorePredictions
        | ExternalModelLifecycleStatus::NeedsCalibrationReview
        | ExternalModelLifecycleStatus::NeedsRiskReview => {
            vec!["DiagnosticOnly".to_string(), "Retired".to_string()]
        }
        ExternalModelLifecycleStatus::Retired | ExternalModelLifecycleStatus::Blocked => Vec::new(),
    }
}

fn lifecycle_reason(
    entry: Option<&super::external_artifact_registry::ConservativeLeaderboardEntry>,
    status: ExternalModelLifecycleStatus,
) -> String {
    match status {
        ExternalModelLifecycleStatus::ResearchCandidate => {
            "leaderboard-eligible and research-candidate-only".to_string()
        }
        ExternalModelLifecycleStatus::Watchlisted => "owner watch enabled".to_string(),
        ExternalModelLifecycleStatus::NeedsMorePredictions => {
            "coverage or evidence depth is insufficient".to_string()
        }
        ExternalModelLifecycleStatus::NeedsCalibrationReview => {
            "calibration drift or calibration weakness needs review".to_string()
        }
        ExternalModelLifecycleStatus::NeedsRiskReview => {
            "risk behavior requires review".to_string()
        }
        ExternalModelLifecycleStatus::DiagnosticOnly => {
            "owner or policy downgraded model to diagnostic-only".to_string()
        }
        ExternalModelLifecycleStatus::Retired => {
            "retired from active research comparison".to_string()
        }
        ExternalModelLifecycleStatus::Blocked => entry
            .map(|entry| format!("{:?}", entry.entry_status))
            .unwrap_or_else(|| "Blocked".to_string()),
        ExternalModelLifecycleStatus::Evaluated => "evaluation artifacts are available".to_string(),
        ExternalModelLifecycleStatus::Imported => {
            "prediction import artifacts are available".to_string()
        }
        ExternalModelLifecycleStatus::Registered => {
            "registered in offline artifact registry".to_string()
        }
        ExternalModelLifecycleStatus::Downgraded => {
            "downgraded to conservative handling".to_string()
        }
    }
}

fn comparability_cell(
    model_id: &str,
    model_version: &str,
    dimension: ModelComparabilityDimension,
    value: Option<String>,
    expected_value: Option<String>,
) -> ModelComparabilityCell {
    let comparable = match (&value, &expected_value) {
        (Some(value), Some(expected)) if expected.starts_with(">=") => value
            .parse::<f64>()
            .ok()
            .zip(expected.trim_start_matches(">=").parse::<f64>().ok())
            .map(|(value, expected)| value >= expected)
            .unwrap_or(false),
        (Some(value), Some(expected)) => value == expected,
        (Some(_), None) => true,
        _ => false,
    };
    ModelComparabilityCell {
        model_id: model_id.to_string(),
        model_version: model_version.to_string(),
        dimension,
        comparable,
        value,
        expected_value,
        reason_codes: stable_reason_codes(&[ReasonCode::ModelComparabilityMatrixBuilt]),
    }
}

fn model_key(model_id: &str, model_version: &str) -> String {
    format!("{model_id}:{model_version}")
}

fn is_tracked_model(model_id: &str) -> bool {
    !matches!(
        model_id,
        "registry" | "external-import" | "mamba_contract" | "unknown"
    )
}

fn is_blocked_entry_status(
    status: super::external_artifact_registry::LeaderboardEntryStatus,
) -> bool {
    matches!(
        status,
        super::external_artifact_registry::LeaderboardEntryStatus::BlockedByCoverage
            | super::external_artifact_registry::LeaderboardEntryStatus::BlockedByModelCard
            | super::external_artifact_registry::LeaderboardEntryStatus::BlockedByCalibration
            | super::external_artifact_registry::LeaderboardEntryStatus::BlockedByRisk
            | super::external_artifact_registry::LeaderboardEntryStatus::BlockedByContractMismatch
            | super::external_artifact_registry::LeaderboardEntryStatus::BlockedByInsufficientRows
            | super::external_artifact_registry::LeaderboardEntryStatus::BlockedByAblationInstability
    )
}

fn empty_registry() -> ExternalModelArtifactRegistry {
    ExternalModelArtifactRegistry {
        registry_id: "empty".to_string(),
        entries: Vec::new(),
        model_ids: Vec::new(),
        model_versions: Vec::new(),
        comparable_entries: 0,
        diagnostic_entries: 0,
        incompatible_entries: 0,
        unknown_entries: 0,
        registry_status: super::ExternalModelArtifactRegistryStatus::DiagnosticOnly,
        reason_codes: Vec::new(),
    }
}

fn empty_leaderboard(id: &str) -> ConservativeExternalLeaderboard {
    ConservativeExternalLeaderboard {
        leaderboard_id: id.to_string(),
        entries: Vec::new(),
        eligible_entries: 0,
        diagnostic_entries: 0,
        blocked_entries: 0,
        baseline_entry: None,
        trinity_entry: None,
        no_trade_entry: None,
        risk_denied_entry: None,
        leaderboard_status: ConservativeExternalLeaderboardStatus::DiagnosticOnly,
        reason_codes: Vec::new(),
    }
}

fn empty_drift() -> CalibrationDriftReport {
    CalibrationDriftReport {
        records: Vec::new(),
        stable_count: 0,
        mild_drift_count: 0,
        severe_drift_count: 0,
        insufficient_history_count: 0,
        drift_status: CalibrationDriftStatus::InsufficientHistory,
        reason_codes: Vec::new(),
    }
}

fn empty_history(id: &str) -> ExternalEvaluationHistoryReport {
    ExternalEvaluationHistoryReport {
        history_id: id.to_string(),
        model_histories: Vec::new(),
        baseline_reference_summary: "none".to_string(),
        trinity_reference_summary: "none".to_string(),
        no_trade_reference_summary: "none".to_string(),
        risk_denied_reference_summary: "none".to_string(),
        latest_model_versions: BTreeMap::new(),
        previous_model_versions: BTreeMap::new(),
        metric_deltas: BTreeMap::new(),
        history_status: ExternalEvaluationHistoryStatus::DiagnosticOnly,
        reason_codes: Vec::new(),
    }
}

fn empty_version_comparison(id: &str) -> ExternalModelVersionComparisonReport {
    ExternalModelVersionComparisonReport {
        comparison_id: id.to_string(),
        records: Vec::new(),
        model_id: "unknown".to_string(),
        latest_version: "unknown".to_string(),
        previous_version: None,
        metric_delta_summary: BTreeMap::new(),
        calibration_delta_summary: BTreeMap::new(),
        risk_delta_summary: BTreeMap::new(),
        comparison_status: ExternalModelVersionComparisonStatus::DiagnosticOnly,
        reason_codes: Vec::new(),
    }
}

fn empty_previous_comparison() -> PreviousExternalComparisonReport {
    PreviousExternalComparisonReport {
        model_id: "unknown".to_string(),
        latest_version: "unknown".to_string(),
        previous_versions: Vec::new(),
        comparable_versions: Vec::new(),
        non_comparable_versions: Vec::new(),
        metric_deltas: BTreeMap::new(),
        drift_summary: "none".to_string(),
        recommendation: PreviousExternalComparisonRecommendation::NeedsMoreHistory,
        reason_codes: Vec::new(),
    }
}

fn empty_audit(id: &str) -> ExternalArtifactRegistryAuditReport {
    ExternalArtifactRegistryAuditReport {
        registry_id: id.to_string(),
        artifacts_scanned: 0,
        missing_artifacts: 0,
        incompatible_artifacts: 0,
        secret_like_fields: 0,
        order_account_fields: 0,
        unsafe_intended_use_count: 0,
        audit_status: ExternalArtifactRegistryAuditStatus::DiagnosticOnly,
        reason_codes: Vec::new(),
    }
}

fn write_text_report<T: Serialize>(dir: &Path, name: &str, value: &T) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    fs::write(dir.join(name), text).map_err(|err| err.to_string())
}

fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}
