use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_ordered_strings, stable_reason_codes};

use super::{
    ExternalModelCardV2, ExternalModelResearchOpsBundle, ExternalModelReviewItemKind,
    ExternalModelReviewQueue, ExternalModelWatchStatus, ExternalModelWatchlist,
    LeaderboardChangeKind, ModelEvidenceRecommendedAction, ModelEvidenceRiskProfile,
    ModelLeaderboardChangeLog, OwnerModelReviewAction, OwnerModelReviewActionKind,
    SequenceExportManifest,
};

fn default_closure_output_root() -> String {
    "target/soma_model_ops_review_closure".to_string()
}

fn default_history_output_root() -> String {
    "target/soma_prediction_history_pack".to_string()
}

fn default_max_review_items() -> usize {
    64
}

fn default_max_models() -> usize {
    16
}

fn default_max_versions_per_model() -> usize {
    16
}

fn default_max_prediction_files() -> usize {
    64
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelOpsReviewClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub external_model_research_ops_paths: Vec<String>,
    #[serde(default)]
    pub model_review_queue_paths: Vec<String>,
    #[serde(default)]
    pub owner_model_review_action_paths: Vec<String>,
    #[serde(default)]
    pub watchlist_paths: Vec<String>,
    #[serde(default)]
    pub model_risk_profile_paths: Vec<String>,
    #[serde(default)]
    pub leaderboard_changelog_paths: Vec<String>,
    #[serde(default)]
    pub prediction_history_pack_paths: Vec<String>,
    #[serde(default)]
    pub model_ops_baseline_paths: Vec<String>,
    #[serde(default)]
    pub model_ops_current_paths: Vec<String>,
    #[serde(default = "default_closure_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_review_items")]
    pub max_review_items: usize,
    #[serde(default = "default_true")]
    pub require_owner_reason_for_retire: bool,
    #[serde(default = "default_true")]
    pub require_owner_reason_for_downgrade: bool,
    #[serde(default = "default_true")]
    pub allow_keep_research_candidate: bool,
    #[serde(default = "default_true")]
    pub allow_mark_diagnostic_only: bool,
    #[serde(default = "default_true")]
    pub allow_retire_version: bool,
    #[serde(default = "default_true")]
    pub allow_request_more_predictions: bool,
    #[serde(default = "default_true")]
    pub allow_request_calibration_review: bool,
    #[serde(default = "default_true")]
    pub allow_request_risk_review: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ModelOpsReviewClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "sprint66-model-review-closure".to_string(),
            external_model_research_ops_paths: Vec::new(),
            model_review_queue_paths: Vec::new(),
            owner_model_review_action_paths: Vec::new(),
            watchlist_paths: Vec::new(),
            model_risk_profile_paths: Vec::new(),
            leaderboard_changelog_paths: Vec::new(),
            prediction_history_pack_paths: Vec::new(),
            model_ops_baseline_paths: Vec::new(),
            model_ops_current_paths: Vec::new(),
            output_root: default_closure_output_root(),
            max_review_items: default_max_review_items(),
            require_owner_reason_for_retire: true,
            require_owner_reason_for_downgrade: true,
            allow_keep_research_candidate: true,
            allow_mark_diagnostic_only: true,
            allow_retire_version: true,
            allow_request_more_predictions: true,
            allow_request_calibration_review: true,
            allow_request_risk_review: true,
            reason_codes: stable_reason_codes(&[ReasonCode::ModelOpsReviewClosureConfigBuilt]),
        }
    }
}

impl ModelOpsReviewClosureConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let mut config: Self = toml::from_str(&text).map_err(|err| err.to_string())?;
        config.reason_codes = stable_reason_codes(&[ReasonCode::ModelOpsReviewClosureConfigBuilt]);
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.closure_id.trim().is_empty() || self.max_review_items == 0 {
            return Err(
                "model ops review closure config requires a closure id and positive limits"
                    .to_string(),
            );
        }
        let all_paths = self
            .external_model_research_ops_paths
            .iter()
            .chain(self.model_review_queue_paths.iter())
            .chain(self.owner_model_review_action_paths.iter())
            .chain(self.watchlist_paths.iter())
            .chain(self.model_risk_profile_paths.iter())
            .chain(self.leaderboard_changelog_paths.iter())
            .chain(self.prediction_history_pack_paths.iter())
            .chain(self.model_ops_baseline_paths.iter())
            .chain(self.model_ops_current_paths.iter())
            .chain(std::iter::once(&self.output_root));
        if all_paths.clone().any(|path| path.contains("://")) {
            return Err("model ops review closure config paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.closure_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionHistoryPackConfig {
    pub history_pack_id: String,
    #[serde(default)]
    pub sequence_export_manifest_paths: Vec<String>,
    #[serde(default)]
    pub prediction_csv_paths: Vec<String>,
    #[serde(default)]
    pub model_card_paths: Vec<String>,
    #[serde(default)]
    pub evaluation_report_paths: Vec<String>,
    #[serde(default)]
    pub import_report_paths: Vec<String>,
    #[serde(default = "default_history_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_models")]
    pub max_models: usize,
    #[serde(default = "default_max_versions_per_model")]
    pub max_versions_per_model: usize,
    #[serde(default = "default_max_prediction_files")]
    pub max_prediction_files: usize,
    #[serde(default = "default_true")]
    pub require_same_sequence_export: bool,
    #[serde(default = "default_true")]
    pub require_model_cards: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for PredictionHistoryPackConfig {
    fn default() -> Self {
        Self {
            history_pack_id: "sprint66-prediction-history-pack".to_string(),
            sequence_export_manifest_paths: Vec::new(),
            prediction_csv_paths: Vec::new(),
            model_card_paths: Vec::new(),
            evaluation_report_paths: Vec::new(),
            import_report_paths: Vec::new(),
            output_root: default_history_output_root(),
            max_models: default_max_models(),
            max_versions_per_model: default_max_versions_per_model(),
            max_prediction_files: default_max_prediction_files(),
            require_same_sequence_export: true,
            require_model_cards: true,
            reason_codes: stable_reason_codes(&[ReasonCode::PredictionHistoryPackBuilt]),
        }
    }
}

impl PredictionHistoryPackConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let mut config: Self = toml::from_str(&text).map_err(|err| err.to_string())?;
        config.reason_codes = stable_reason_codes(&[ReasonCode::PredictionHistoryPackBuilt]);
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.history_pack_id.trim().is_empty()
            || self.max_models == 0
            || self.max_versions_per_model == 0
            || self.max_prediction_files == 0
        {
            return Err(
                "prediction history pack config requires non-empty ids and positive limits"
                    .to_string(),
            );
        }
        let all_paths = self
            .sequence_export_manifest_paths
            .iter()
            .chain(self.prediction_csv_paths.iter())
            .chain(self.model_card_paths.iter())
            .chain(self.evaluation_report_paths.iter())
            .chain(self.import_report_paths.iter())
            .chain(std::iter::once(&self.output_root));
        if all_paths.clone().any(|path| path.contains("://")) {
            return Err("prediction history pack config paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.history_pack_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelReviewClosureStatus {
    ReviewClosed,
    PartiallyClosed,
    NeedsOwnerReview,
    NeedsMorePredictions,
    NeedsCalibrationReview,
    NeedsRiskReview,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelReviewClosureDecision {
    KeepResearchCandidate,
    DowngradeToDiagnostic,
    RetireModelVersion,
    RequestMorePredictions,
    RequestCalibrationReview,
    RequestRiskReview,
    DeferReview,
    DismissReview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelReviewClosureAction {
    pub review_id: String,
    pub model_id: String,
    pub model_version: String,
    pub action: ModelReviewClosureDecision,
    pub allowed: bool,
    pub applied: bool,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelReviewClosureReport {
    pub closure_id: String,
    #[serde(default)]
    pub actions: Vec<ModelReviewClosureAction>,
    pub input_pending_count: usize,
    pub closed_count: usize,
    pub deferred_count: usize,
    pub retired_count: usize,
    pub downgraded_count: usize,
    pub request_more_predictions_count: usize,
    pub calibration_review_count: usize,
    pub risk_review_count: usize,
    pub output_pending_count: usize,
    pub closure_status: ModelReviewClosureStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionHistoryPackStatus {
    PredictionHistoryPackReady,
    NeedMorePredictionHistory,
    MissingModelCards,
    ContractMismatch,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionHistoryPackVersionRecord {
    pub model_id: String,
    pub model_version: String,
    pub comparable: bool,
    pub has_model_card: bool,
    pub has_prediction_file: bool,
    pub has_evaluation_report: bool,
    pub has_import_report: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionHistoryPackReport {
    pub history_pack_id: String,
    pub model_count: usize,
    pub version_count: usize,
    pub prediction_file_count: usize,
    pub comparable_version_count: usize,
    pub non_comparable_version_count: usize,
    pub missing_card_count: usize,
    pub missing_prediction_count: usize,
    #[serde(default)]
    pub records: Vec<PredictionHistoryPackVersionRecord>,
    pub history_status: PredictionHistoryPackStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelOpsDecisionKind {
    KeepResearchCandidate,
    DowngradeToDiagnostic,
    RetireModelVersion,
    RequestMorePredictions,
    RequestCalibrationReview,
    RequestRiskReview,
    WatchModel,
    UnwatchModel,
    DeferReview,
    DismissReview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelOpsDecisionSource {
    OwnerAction,
    PolicyRule,
    RiskProfile,
    LeaderboardChange,
    CalibrationDrift,
    CoverageGap,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelOpsDecisionRecord {
    pub decision_id: String,
    pub model_id: String,
    pub model_version: String,
    pub decision_kind: ModelOpsDecisionKind,
    pub source: ModelOpsDecisionSource,
    #[serde(default)]
    pub before_status: Option<String>,
    #[serde(default)]
    pub after_status: Option<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelOpsDecisionLog {
    #[serde(default)]
    pub records: Vec<ModelOpsDecisionRecord>,
    pub decision_count: usize,
    pub by_kind_counts: BTreeMap<String, usize>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelOpsOperatorQAStatus {
    ReadyForOperatorReview,
    NeedsMorePredictions,
    NeedsCalibrationReview,
    NeedsRiskReview,
    NeedsOwnerAction,
    BlockedBySafety,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelOpsOperatorQAReport {
    pub qa_id: String,
    #[serde(default)]
    pub checklist_items: Vec<String>,
    #[serde(default)]
    pub models_to_review: Vec<String>,
    #[serde(default)]
    pub models_to_retire: Vec<String>,
    #[serde(default)]
    pub models_to_downgrade: Vec<String>,
    #[serde(default)]
    pub models_to_request_predictions: Vec<String>,
    #[serde(default)]
    pub models_to_watch: Vec<String>,
    #[serde(default)]
    pub blocked_models: Vec<String>,
    #[serde(default)]
    pub unsafe_actions_detected: Vec<String>,
    pub qa_status: ModelOpsOperatorQAStatus,
    #[serde(default)]
    pub next_commands: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelOpsRegressionGuardStatus {
    NoRegression,
    RegressionDetected,
    MissingBaseline,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelOpsRegressionGuardReport {
    pub guard_id: String,
    #[serde(default)]
    pub coverage_regressions: Vec<String>,
    #[serde(default)]
    pub calibration_regressions: Vec<String>,
    #[serde(default)]
    pub risk_regressions: Vec<String>,
    #[serde(default)]
    pub comparability_regressions: Vec<String>,
    #[serde(default)]
    pub artifact_completeness_regressions: Vec<String>,
    #[serde(default)]
    pub leaderboard_regressions: Vec<String>,
    pub guard_status: ModelOpsRegressionGuardStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelOpsRegressionSnapshotEntry {
    pub model_id: String,
    pub model_version: String,
    pub coverage_band: String,
    pub calibration_band: String,
    pub risk_band: String,
    pub comparability_band: String,
    pub artifact_completeness_band: String,
    pub leaderboard_band: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelOpsRegressionSnapshot {
    pub snapshot_id: String,
    #[serde(default)]
    pub entries: Vec<ModelOpsRegressionSnapshotEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlTowerModelOpsRefreshStatus {
    ModelOpsRefreshed,
    ModelOpsRefreshedWithWarnings,
    MissingArtifacts,
    UnsafeControlDetected,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerModelOpsRefreshReport {
    pub refresh_id: String,
    pub model_ops_status: String,
    pub review_closure_status: String,
    pub prediction_history_status: String,
    pub operator_qa_status: String,
    #[serde(default)]
    pub regression_guard_status: Option<String>,
    #[serde(default)]
    pub model_ops_panel_path: Option<String>,
    #[serde(default)]
    pub dashboard_state_path: Option<String>,
    #[serde(default)]
    pub dashboard_html_path: Option<String>,
    pub refresh_status: ControlTowerModelOpsRefreshStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelOpsReviewClosureStorageReport {
    pub artifact_count: usize,
    pub storage_bytes: usize,
    pub max_review_items: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelOpsReviewClosureBundle {
    pub model_review_closure_report: ModelReviewClosureReport,
    pub prediction_history_pack_report: PredictionHistoryPackReport,
    pub model_ops_decision_log: ModelOpsDecisionLog,
    pub model_ops_operator_qa_report: ModelOpsOperatorQAReport,
    #[serde(default)]
    pub model_ops_regression_guard_report: Option<ModelOpsRegressionGuardReport>,
    #[serde(default)]
    pub control_tower_model_ops_refresh_report: Option<ControlTowerModelOpsRefreshReport>,
    pub storage_report: ModelOpsReviewClosureStorageReport,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelReviewClosureRunner;

impl ModelReviewClosureRunner {
    pub fn run(
        &self,
        config: &ModelOpsReviewClosureConfig,
    ) -> Result<ModelOpsReviewClosureBundle, String> {
        config.validate()?;
        let ops_bundle = load_first_report::<ExternalModelResearchOpsBundle>(
            &config.external_model_research_ops_paths,
        )?;
        let review_queue = if let Some(bundle) = &ops_bundle {
            bundle.external_model_review_queue.clone()
        } else {
            load_first_report::<ExternalModelReviewQueue>(&config.model_review_queue_paths)?
                .unwrap_or_else(empty_review_queue)
        };
        let owner_actions = load_owner_actions(&config.owner_model_review_action_paths)?;
        let watchlist = if let Some(bundle) = &ops_bundle {
            bundle.external_model_watchlist.clone()
        } else {
            load_first_report::<ExternalModelWatchlist>(&config.watchlist_paths)?
                .unwrap_or_else(empty_watchlist)
        };
        let risk_profiles = if let Some(bundle) = &ops_bundle {
            bundle.model_evidence_risk_profiles.clone()
        } else {
            load_first_report::<Vec<ModelEvidenceRiskProfile>>(&config.model_risk_profile_paths)?
                .unwrap_or_default()
        };
        let changelog = if let Some(bundle) = &ops_bundle {
            bundle.model_leaderboard_changelog.clone()
        } else {
            load_first_report::<ModelLeaderboardChangeLog>(&config.leaderboard_changelog_paths)?
                .unwrap_or_else(empty_changelog)
        };

        let closure_report = build_model_review_closure_report(
            config,
            &review_queue,
            &owner_actions,
            &watchlist,
            &risk_profiles,
            &changelog,
        );
        let prediction_history_pack_report = load_first_report::<PredictionHistoryPackReport>(
            &config.prediction_history_pack_paths,
        )?
        .unwrap_or_else(|| diagnostic_history_pack(&config.closure_id));
        let decision_log = build_model_ops_decision_log(
            &closure_report,
            &owner_actions,
            &risk_profiles,
            &changelog,
        );
        let operator_qa = build_operator_qa_report(
            config,
            &closure_report,
            &prediction_history_pack_report,
            &watchlist,
            &owner_actions,
        );
        let regression_guard = build_regression_guard_report(config)?;
        let refresh_report = build_control_tower_model_ops_refresh_report(
            config,
            ops_bundle.as_ref(),
            &closure_report,
            &prediction_history_pack_report,
            &operator_qa,
            regression_guard.as_ref(),
        )?;
        let storage_report = build_storage_report(config)?;
        let final_summary = build_final_summary(
            &closure_report,
            &prediction_history_pack_report,
            &operator_qa,
        );
        let bundle = ModelOpsReviewClosureBundle {
            model_review_closure_report: closure_report,
            prediction_history_pack_report,
            model_ops_decision_log: decision_log,
            model_ops_operator_qa_report: operator_qa,
            model_ops_regression_guard_report: regression_guard,
            control_tower_model_ops_refresh_report: Some(refresh_report),
            storage_report,
            final_summary,
            reason_codes: stable_reason_codes(&[
                ReasonCode::ModelOpsReviewClosureBundleBuilt,
                ReasonCode::ModelReviewClosureRunnerBuilt,
            ]),
        };
        write_bundle(config, &bundle)?;
        Ok(bundle)
    }

    pub fn run_prediction_history_pack(
        &self,
        config: &PredictionHistoryPackConfig,
    ) -> Result<PredictionHistoryPackReport, String> {
        build_prediction_history_pack_report(config)
    }

    pub fn run_decision_log(
        &self,
        config: &ModelOpsReviewClosureConfig,
    ) -> Result<ModelOpsDecisionLog, String> {
        Ok(self.run(config)?.model_ops_decision_log)
    }

    pub fn run_operator_qa(
        &self,
        config: &ModelOpsReviewClosureConfig,
    ) -> Result<ModelOpsOperatorQAReport, String> {
        Ok(self.run(config)?.model_ops_operator_qa_report)
    }

    pub fn run_regression_guard(
        &self,
        config: &ModelOpsReviewClosureConfig,
    ) -> Result<ModelOpsRegressionGuardReport, String> {
        self.run(config)?
            .model_ops_regression_guard_report
            .ok_or_else(|| "model ops regression guard report unavailable".to_string())
    }

    pub fn run_control_tower_refresh(
        &self,
        config: &ModelOpsReviewClosureConfig,
    ) -> Result<ControlTowerModelOpsRefreshReport, String> {
        self.run(config)?
            .control_tower_model_ops_refresh_report
            .ok_or_else(|| "control tower model ops refresh report unavailable".to_string())
    }
}

fn build_model_review_closure_report(
    config: &ModelOpsReviewClosureConfig,
    review_queue: &ExternalModelReviewQueue,
    owner_actions: &[OwnerModelReviewAction],
    watchlist: &ExternalModelWatchlist,
    risk_profiles: &[ModelEvidenceRiskProfile],
    changelog: &ModelLeaderboardChangeLog,
) -> ModelReviewClosureReport {
    let owner_map = owner_actions.iter().fold(
        BTreeMap::<String, Vec<&OwnerModelReviewAction>>::new(),
        |mut acc, action| {
            acc.entry(model_key(&action.model_id, &action.model_version))
                .or_default()
                .push(action);
            acc
        },
    );
    let risk_map = risk_profiles
        .iter()
        .map(|profile| {
            (
                model_key(&profile.model_id, &profile.model_version),
                profile,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let change_map = changelog
        .changes
        .iter()
        .map(|change| (model_key(&change.model_id, &change.model_version), change))
        .collect::<BTreeMap<_, _>>();
    let watch_map = watchlist
        .entries
        .iter()
        .map(|entry| (model_key(&entry.model_id, &entry.model_version), entry))
        .collect::<BTreeMap<_, _>>();

    let mut actions = Vec::new();
    for item in review_queue
        .pending_items
        .iter()
        .take(config.max_review_items)
    {
        let key = model_key(&item.model_id, &item.model_version);
        let derived = owner_map
            .get(&key)
            .and_then(|actions| actions.last().copied())
            .map(|action| decision_from_owner_action(action.action_kind))
            .unwrap_or_else(|| {
                derive_policy_decision(
                    item,
                    watch_map.get(&key).copied(),
                    risk_map.get(&key).copied(),
                    change_map.get(&key).copied(),
                )
            });
        let owner_note = owner_map
            .get(&key)
            .and_then(|actions| actions.last())
            .and_then(|action| action.note.clone());
        let allowed = closure_action_allowed(config, derived, owner_note.as_deref());
        let applied = allowed && derived != ModelReviewClosureDecision::DeferReview;
        actions.push(ModelReviewClosureAction {
            review_id: item.review_id.clone(),
            model_id: item.model_id.clone(),
            model_version: item.model_version.clone(),
            action: derived,
            allowed,
            applied,
            note: owner_note,
            reason_codes: stable_reason_codes(&[ReasonCode::ModelReviewClosureReportBuilt]),
        });
    }
    actions.sort_by(|left, right| {
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

    let deferred_count = actions
        .iter()
        .filter(|action| {
            action.action == ModelReviewClosureDecision::DeferReview || !action.allowed
        })
        .count();
    let retired_count = actions
        .iter()
        .filter(|action| {
            action.action == ModelReviewClosureDecision::RetireModelVersion && action.applied
        })
        .count();
    let downgraded_count = actions
        .iter()
        .filter(|action| {
            action.action == ModelReviewClosureDecision::DowngradeToDiagnostic && action.applied
        })
        .count();
    let request_more_predictions_count = actions
        .iter()
        .filter(|action| {
            action.action == ModelReviewClosureDecision::RequestMorePredictions && action.applied
        })
        .count();
    let calibration_review_count = actions
        .iter()
        .filter(|action| {
            action.action == ModelReviewClosureDecision::RequestCalibrationReview && action.applied
        })
        .count();
    let risk_review_count = actions
        .iter()
        .filter(|action| {
            action.action == ModelReviewClosureDecision::RequestRiskReview && action.applied
        })
        .count();
    let closed_count = actions
        .iter()
        .filter(|action| action.applied && action.action != ModelReviewClosureDecision::DeferReview)
        .count();
    let output_pending_count = deferred_count;
    let closure_status = if actions.is_empty() {
        ModelReviewClosureStatus::DiagnosticOnly
    } else if request_more_predictions_count > 0 {
        ModelReviewClosureStatus::NeedsMorePredictions
    } else if calibration_review_count > 0 {
        ModelReviewClosureStatus::NeedsCalibrationReview
    } else if risk_review_count > 0 {
        ModelReviewClosureStatus::NeedsRiskReview
    } else if downgraded_count > 0 && closed_count == downgraded_count {
        ModelReviewClosureStatus::DiagnosticOnly
    } else if output_pending_count > 0 && closed_count > 0 {
        ModelReviewClosureStatus::PartiallyClosed
    } else if output_pending_count > 0 {
        ModelReviewClosureStatus::NeedsOwnerReview
    } else {
        ModelReviewClosureStatus::ReviewClosed
    };

    ModelReviewClosureReport {
        closure_id: config.closure_id.clone(),
        actions,
        input_pending_count: review_queue.pending_items.len(),
        closed_count,
        deferred_count,
        retired_count,
        downgraded_count,
        request_more_predictions_count,
        calibration_review_count,
        risk_review_count,
        output_pending_count,
        closure_status,
        reason_codes: stable_reason_codes(&[ReasonCode::ModelReviewClosureReportBuilt]),
    }
}

fn build_prediction_history_pack_report(
    config: &PredictionHistoryPackConfig,
) -> Result<PredictionHistoryPackReport, String> {
    config.validate()?;
    if config.prediction_csv_paths.len() > config.max_prediction_files {
        return Err("max_prediction_files exceeded".to_string());
    }
    let manifest =
        load_first_report::<SequenceExportManifest>(&config.sequence_export_manifest_paths)?
            .ok_or_else(|| {
                "prediction history pack requires a sequence export manifest".to_string()
            })?;
    let model_cards = config
        .model_card_paths
        .iter()
        .map(|path| read_json_file::<ExternalModelCardV2>(Path::new(path)))
        .collect::<Result<Vec<_>, _>>()?;
    let predictions = config
        .prediction_csv_paths
        .iter()
        .map(|path| {
            prediction_identity_from_csv(Path::new(path)).map(|identity| (path.clone(), identity))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let evaluation_keys = collect_model_keys_from_paths(&config.evaluation_report_paths);
    let import_keys = collect_model_keys_from_paths(&config.import_report_paths);

    let mut card_by_key = BTreeMap::new();
    for card in model_cards {
        card_by_key.insert(model_key(&card.model_id, &card.model_version), card);
    }
    let mut prediction_by_key = BTreeMap::new();
    for (path, (model_id, model_version)) in predictions {
        prediction_by_key.insert(model_key(&model_id, &model_version), path);
    }

    let keys = card_by_key
        .keys()
        .chain(prediction_by_key.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    enforce_model_limits_for_history(config, &keys)?;

    let baseline_card = card_by_key.values().next().cloned();
    let mut comparable_version_count = 0usize;
    let mut non_comparable_version_count = 0usize;
    let mut missing_card_count = 0usize;
    let mut missing_prediction_count = 0usize;
    let mut records = Vec::new();

    for key in keys {
        let Some((model_id, model_version)) = key.split_once(':') else {
            continue;
        };
        let card = card_by_key.get(&key);
        let has_model_card = card.is_some();
        let has_prediction_file = prediction_by_key.contains_key(&key);
        let has_evaluation_report = evaluation_keys.contains(&key);
        let has_import_report = import_keys.contains(&key);
        if !has_model_card {
            missing_card_count += 1;
        }
        if !has_prediction_file {
            missing_prediction_count += 1;
        }
        let comparable = if config.require_same_sequence_export {
            card.zip(baseline_card.as_ref())
                .map(|(card, baseline)| {
                    card.feature_schema_hash == baseline.feature_schema_hash
                        && card.label_manifest_hash == baseline.label_manifest_hash
                        && card.split_policy == baseline.split_policy
                        && manifest.fingerprint == manifest.fingerprint
                })
                .unwrap_or(false)
        } else {
            has_model_card && has_prediction_file
        };
        if comparable {
            comparable_version_count += 1;
        } else {
            non_comparable_version_count += 1;
        }
        records.push(PredictionHistoryPackVersionRecord {
            model_id: model_id.to_string(),
            model_version: model_version.to_string(),
            comparable,
            has_model_card,
            has_prediction_file,
            has_evaluation_report,
            has_import_report,
            reason_codes: stable_reason_codes(&[ReasonCode::PredictionHistoryPackBuilt]),
        });
    }
    records.sort_by(|left, right| {
        (left.model_id.clone(), left.model_version.clone())
            .cmp(&(right.model_id.clone(), right.model_version.clone()))
    });

    let history_status = if records.is_empty() {
        PredictionHistoryPackStatus::DiagnosticOnly
    } else if missing_card_count > 0 && config.require_model_cards {
        PredictionHistoryPackStatus::MissingModelCards
    } else if non_comparable_version_count > 0 && config.require_same_sequence_export {
        PredictionHistoryPackStatus::ContractMismatch
    } else if records.len() < 3 || missing_prediction_count > 0 {
        PredictionHistoryPackStatus::NeedMorePredictionHistory
    } else {
        PredictionHistoryPackStatus::PredictionHistoryPackReady
    };
    Ok(PredictionHistoryPackReport {
        history_pack_id: config.history_pack_id.clone(),
        model_count: records
            .iter()
            .map(|record| record.model_id.clone())
            .collect::<BTreeSet<_>>()
            .len(),
        version_count: records.len(),
        prediction_file_count: config.prediction_csv_paths.len(),
        comparable_version_count,
        non_comparable_version_count,
        missing_card_count,
        missing_prediction_count,
        records,
        history_status,
        reason_codes: stable_reason_codes(&[ReasonCode::PredictionHistoryPackBuilt]),
    })
}

fn build_model_ops_decision_log(
    closure_report: &ModelReviewClosureReport,
    owner_actions: &[OwnerModelReviewAction],
    risk_profiles: &[ModelEvidenceRiskProfile],
    changelog: &ModelLeaderboardChangeLog,
) -> ModelOpsDecisionLog {
    let mut records = Vec::new();
    for action in &closure_report.actions {
        records.push(ModelOpsDecisionRecord {
            decision_id: format!("closure:{}", action.review_id),
            model_id: action.model_id.clone(),
            model_version: action.model_version.clone(),
            decision_kind: decision_kind_from_closure(action.action),
            source: ModelOpsDecisionSource::PolicyRule,
            before_status: Some("Pending".to_string()),
            after_status: Some(format!("{:?}", action.action)),
            reason_codes: stable_reason_codes(&[ReasonCode::ModelOpsDecisionLogBuilt]),
        });
    }
    for action in owner_actions {
        records.push(ModelOpsDecisionRecord {
            decision_id: format!("owner:{}", action.action_id),
            model_id: action.model_id.clone(),
            model_version: action.model_version.clone(),
            decision_kind: decision_kind_from_owner_action(action.action_kind),
            source: ModelOpsDecisionSource::OwnerAction,
            before_status: None,
            after_status: Some(format!("{:?}", action.action_kind)),
            reason_codes: stable_reason_codes(&[ReasonCode::ModelOpsDecisionLogBuilt]),
        });
    }
    for profile in risk_profiles {
        let (decision_kind, source) = match profile.recommended_action {
            ModelEvidenceRecommendedAction::RequestMorePredictions => (
                ModelOpsDecisionKind::RequestMorePredictions,
                ModelOpsDecisionSource::CoverageGap,
            ),
            ModelEvidenceRecommendedAction::RequestCalibrationReview => (
                ModelOpsDecisionKind::RequestCalibrationReview,
                ModelOpsDecisionSource::CalibrationDrift,
            ),
            ModelEvidenceRecommendedAction::RequestRiskReview => (
                ModelOpsDecisionKind::RequestRiskReview,
                ModelOpsDecisionSource::RiskProfile,
            ),
            ModelEvidenceRecommendedAction::DowngradeToDiagnostic => (
                ModelOpsDecisionKind::DowngradeToDiagnostic,
                ModelOpsDecisionSource::RiskProfile,
            ),
            ModelEvidenceRecommendedAction::RetireModelVersion => (
                ModelOpsDecisionKind::RetireModelVersion,
                ModelOpsDecisionSource::RiskProfile,
            ),
            ModelEvidenceRecommendedAction::KeepResearchCandidate => (
                ModelOpsDecisionKind::KeepResearchCandidate,
                ModelOpsDecisionSource::RiskProfile,
            ),
        };
        records.push(ModelOpsDecisionRecord {
            decision_id: format!("risk:{}:{}", profile.model_id, profile.model_version),
            model_id: profile.model_id.clone(),
            model_version: profile.model_version.clone(),
            decision_kind,
            source,
            before_status: Some(format!("{:?}", profile.evidence_risk_level)),
            after_status: Some(format!("{:?}", profile.recommended_action)),
            reason_codes: stable_reason_codes(&[ReasonCode::ModelOpsDecisionLogBuilt]),
        });
    }
    for change in &changelog.changes {
        if change.change_kind == LeaderboardChangeKind::NoChange {
            continue;
        }
        records.push(ModelOpsDecisionRecord {
            decision_id: format!("leaderboard:{}:{}", change.model_id, change.model_version),
            model_id: change.model_id.clone(),
            model_version: change.model_version.clone(),
            decision_kind: if change.change_kind == LeaderboardChangeKind::NewlyBlocked {
                ModelOpsDecisionKind::RequestMorePredictions
            } else {
                ModelOpsDecisionKind::KeepResearchCandidate
            },
            source: ModelOpsDecisionSource::LeaderboardChange,
            before_status: change.previous_rank.map(|rank| rank.to_string()),
            after_status: change.current_rank.map(|rank| rank.to_string()),
            reason_codes: stable_reason_codes(&[ReasonCode::ModelOpsDecisionLogBuilt]),
        });
    }
    records.sort_by(|left, right| {
        (
            left.model_id.clone(),
            left.model_version.clone(),
            left.decision_id.clone(),
        )
            .cmp(&(
                right.model_id.clone(),
                right.model_version.clone(),
                right.decision_id.clone(),
            ))
    });
    let mut by_kind_counts = BTreeMap::new();
    for record in &records {
        *by_kind_counts
            .entry(format!("{:?}", record.decision_kind))
            .or_insert(0) += 1;
    }
    ModelOpsDecisionLog {
        decision_count: records.len(),
        records,
        by_kind_counts,
        reason_codes: stable_reason_codes(&[ReasonCode::ModelOpsDecisionLogBuilt]),
    }
}

fn build_operator_qa_report(
    config: &ModelOpsReviewClosureConfig,
    closure_report: &ModelReviewClosureReport,
    prediction_history_pack_report: &PredictionHistoryPackReport,
    watchlist: &ExternalModelWatchlist,
    owner_actions: &[OwnerModelReviewAction],
) -> ModelOpsOperatorQAReport {
    let mut unsafe_actions_detected = Vec::new();
    for action in owner_actions {
        let note = action.note.clone().unwrap_or_default().to_ascii_lowercase();
        if note.contains("secret")
            || note.contains("order")
            || note.contains("account")
            || note.contains("broker")
            || note.contains("live")
            || note.contains("runtime")
        {
            unsafe_actions_detected.push(format!(
                "{}:{}:{}",
                action.model_id, action.model_version, action.action_id
            ));
        }
    }
    unsafe_actions_detected = stable_ordered_strings(&unsafe_actions_detected);

    let mut models_to_retire = Vec::new();
    let mut models_to_downgrade = Vec::new();
    let mut models_to_request_predictions = Vec::new();
    let mut models_to_review = Vec::new();
    for action in &closure_report.actions {
        let label = format!("{}:{}", action.model_id, action.model_version);
        models_to_review.push(label.clone());
        match action.action {
            ModelReviewClosureDecision::RetireModelVersion => models_to_retire.push(label),
            ModelReviewClosureDecision::DowngradeToDiagnostic => models_to_downgrade.push(label),
            ModelReviewClosureDecision::RequestMorePredictions => {
                models_to_request_predictions.push(label)
            }
            _ => {}
        }
    }
    let models_to_watch = watchlist
        .entries
        .iter()
        .filter(|entry| entry.watch_status == ExternalModelWatchStatus::Active)
        .map(|entry| format!("{}:{}", entry.model_id, entry.model_version))
        .collect::<Vec<_>>();
    let blocked_models = closure_report
        .actions
        .iter()
        .filter(|action| !action.allowed)
        .map(|action| format!("{}:{}", action.model_id, action.model_version))
        .collect::<Vec<_>>();
    let qa_status = if !unsafe_actions_detected.is_empty() {
        ModelOpsOperatorQAStatus::BlockedBySafety
    } else if closure_report.request_more_predictions_count > 0
        || prediction_history_pack_report.history_status
            == PredictionHistoryPackStatus::NeedMorePredictionHistory
    {
        ModelOpsOperatorQAStatus::NeedsMorePredictions
    } else if closure_report.calibration_review_count > 0 {
        ModelOpsOperatorQAStatus::NeedsCalibrationReview
    } else if closure_report.risk_review_count > 0 {
        ModelOpsOperatorQAStatus::NeedsRiskReview
    } else if closure_report.output_pending_count > 0 {
        ModelOpsOperatorQAStatus::NeedsOwnerAction
    } else if closure_report.closure_status == ModelReviewClosureStatus::DiagnosticOnly {
        ModelOpsOperatorQAStatus::DiagnosticOnly
    } else {
        ModelOpsOperatorQAStatus::ReadyForOperatorReview
    };
    ModelOpsOperatorQAReport {
        qa_id: config.closure_id.clone(),
        checklist_items: stable_ordered_strings(&[
            "calibration reviewed".to_string(),
            "evaluation exists".to_string(),
            "leaderboard position reviewed".to_string(),
            "model card exists".to_string(),
            "NoTrade/RiskDenied reference preserved".to_string(),
            "no live/runtime/training path".to_string(),
            "no secret/order/account fields".to_string(),
            "predictions cover known sequences".to_string(),
            "risk behavior reviewed".to_string(),
        ]),
        models_to_review: stable_ordered_strings(&models_to_review),
        models_to_retire: stable_ordered_strings(&models_to_retire),
        models_to_downgrade: stable_ordered_strings(&models_to_downgrade),
        models_to_request_predictions: stable_ordered_strings(&models_to_request_predictions),
        models_to_watch: stable_ordered_strings(&models_to_watch),
        blocked_models: stable_ordered_strings(&blocked_models),
        unsafe_actions_detected,
        qa_status,
        next_commands: stable_ordered_strings(&[
            "cargo run --quiet --bin soma_experiment -- control-tower-model-ops-refresh --config examples/soma_control_tower_model_ops_refresh.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- model-ops-decision-log --config examples/soma_model_ops_decision_log.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- model-ops-operator-qa --config examples/soma_model_ops_operator_qa.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- model-ops-regression-guard --config examples/soma_model_ops_regression_guard.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- model-review-close --config examples/soma_model_review_close.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- prediction-history-pack --config examples/soma_prediction_history_pack.toml".to_string(),
        ]),
        reason_codes: stable_reason_codes(&[ReasonCode::ModelOpsOperatorQaReportBuilt]),
    }
}

fn build_regression_guard_report(
    config: &ModelOpsReviewClosureConfig,
) -> Result<Option<ModelOpsRegressionGuardReport>, String> {
    if config.model_ops_current_paths.is_empty() {
        return Ok(None);
    }
    let current = load_first_report::<ModelOpsRegressionSnapshot>(&config.model_ops_current_paths)?
        .ok_or_else(|| "model ops regression guard requires current snapshot".to_string())?;
    let baseline =
        load_first_report::<ModelOpsRegressionSnapshot>(&config.model_ops_baseline_paths)?;
    let Some(baseline) = baseline else {
        return Ok(Some(ModelOpsRegressionGuardReport {
            guard_id: config.closure_id.clone(),
            coverage_regressions: Vec::new(),
            calibration_regressions: Vec::new(),
            risk_regressions: Vec::new(),
            comparability_regressions: Vec::new(),
            artifact_completeness_regressions: Vec::new(),
            leaderboard_regressions: Vec::new(),
            guard_status: ModelOpsRegressionGuardStatus::MissingBaseline,
            reason_codes: stable_reason_codes(&[ReasonCode::ModelOpsRegressionGuardBuilt]),
        }));
    };

    let baseline_map = baseline
        .entries
        .iter()
        .map(|entry| (model_key(&entry.model_id, &entry.model_version), entry))
        .collect::<BTreeMap<_, _>>();
    let mut coverage_regressions = Vec::new();
    let mut calibration_regressions = Vec::new();
    let mut risk_regressions = Vec::new();
    let mut comparability_regressions = Vec::new();
    let mut artifact_completeness_regressions = Vec::new();
    let mut leaderboard_regressions = Vec::new();
    for entry in &current.entries {
        let key = model_key(&entry.model_id, &entry.model_version);
        let Some(previous) = baseline_map.get(&key) else {
            continue;
        };
        if previous.coverage_band != entry.coverage_band {
            coverage_regressions.push(key.clone());
        }
        if previous.calibration_band != entry.calibration_band {
            calibration_regressions.push(key.clone());
        }
        if previous.risk_band != entry.risk_band {
            risk_regressions.push(key.clone());
        }
        if previous.comparability_band != entry.comparability_band {
            comparability_regressions.push(key.clone());
        }
        if previous.artifact_completeness_band != entry.artifact_completeness_band {
            artifact_completeness_regressions.push(key.clone());
        }
        if previous.leaderboard_band != entry.leaderboard_band {
            leaderboard_regressions.push(key);
        }
    }
    let has_regression = !coverage_regressions.is_empty()
        || !calibration_regressions.is_empty()
        || !risk_regressions.is_empty()
        || !comparability_regressions.is_empty()
        || !artifact_completeness_regressions.is_empty()
        || !leaderboard_regressions.is_empty();
    Ok(Some(ModelOpsRegressionGuardReport {
        guard_id: config.closure_id.clone(),
        coverage_regressions: stable_ordered_strings(&coverage_regressions),
        calibration_regressions: stable_ordered_strings(&calibration_regressions),
        risk_regressions: stable_ordered_strings(&risk_regressions),
        comparability_regressions: stable_ordered_strings(&comparability_regressions),
        artifact_completeness_regressions: stable_ordered_strings(
            &artifact_completeness_regressions,
        ),
        leaderboard_regressions: stable_ordered_strings(&leaderboard_regressions),
        guard_status: if has_regression {
            ModelOpsRegressionGuardStatus::RegressionDetected
        } else {
            ModelOpsRegressionGuardStatus::NoRegression
        },
        reason_codes: stable_reason_codes(&[ReasonCode::ModelOpsRegressionGuardBuilt]),
    }))
}

fn build_control_tower_model_ops_refresh_report(
    config: &ModelOpsReviewClosureConfig,
    ops_bundle: Option<&ExternalModelResearchOpsBundle>,
    closure_report: &ModelReviewClosureReport,
    prediction_history_pack_report: &PredictionHistoryPackReport,
    operator_qa: &ModelOpsOperatorQAReport,
    regression_guard: Option<&ModelOpsRegressionGuardReport>,
) -> Result<ControlTowerModelOpsRefreshReport, String> {
    let dir = config.output_dir();
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    let panel_path = dir.join("control_tower_model_ops_refresh_panel.txt");
    let mut lines = vec![
        format!("review_closure_status={:?}", closure_report.closure_status),
        format!(
            "prediction_history_status={:?}",
            prediction_history_pack_report.history_status
        ),
        format!("operator_qa_status={:?}", operator_qa.qa_status),
    ];
    if let Some(guard) = regression_guard {
        lines.push(format!("regression_guard_status={:?}", guard.guard_status));
    }
    if let Some(bundle) = ops_bundle {
        if let Some(panel) = &bundle.control_tower_model_ops_panel_summary {
            lines.push(format!(
                "existing_model_ops_status={}",
                panel.research_ops_status
            ));
        }
    }
    lines.push("controls=train/live/runtime/order/account forbidden".to_string());
    let panel_text = lines.join("\n");
    fs::write(&panel_path, &panel_text).map_err(|err| err.to_string())?;
    let lowered = panel_text.to_ascii_lowercase();
    let unsafe_control = lowered.contains("button") || lowered.contains("execute");
    let refresh_status = if unsafe_control {
        ControlTowerModelOpsRefreshStatus::UnsafeControlDetected
    } else if ops_bundle.is_none() && closure_report.actions.is_empty() {
        ControlTowerModelOpsRefreshStatus::MissingArtifacts
    } else if regression_guard
        .map(|guard| guard.guard_status == ModelOpsRegressionGuardStatus::RegressionDetected)
        .unwrap_or(false)
        || matches!(
            operator_qa.qa_status,
            ModelOpsOperatorQAStatus::BlockedBySafety
                | ModelOpsOperatorQAStatus::NeedsMorePredictions
                | ModelOpsOperatorQAStatus::NeedsCalibrationReview
                | ModelOpsOperatorQAStatus::NeedsRiskReview
                | ModelOpsOperatorQAStatus::NeedsOwnerAction
        )
    {
        ControlTowerModelOpsRefreshStatus::ModelOpsRefreshedWithWarnings
    } else {
        ControlTowerModelOpsRefreshStatus::ModelOpsRefreshed
    };
    Ok(ControlTowerModelOpsRefreshReport {
        refresh_id: config.closure_id.clone(),
        model_ops_status: ops_bundle
            .map(|bundle| {
                format!(
                    "{:?}",
                    bundle.external_model_research_ops_report.final_status
                )
            })
            .unwrap_or_else(|| "DiagnosticOnly".to_string()),
        review_closure_status: format!("{:?}", closure_report.closure_status),
        prediction_history_status: format!("{:?}", prediction_history_pack_report.history_status),
        operator_qa_status: format!("{:?}", operator_qa.qa_status),
        regression_guard_status: regression_guard.map(|guard| format!("{:?}", guard.guard_status)),
        model_ops_panel_path: Some(panel_path.display().to_string()),
        dashboard_state_path: None,
        dashboard_html_path: None,
        refresh_status,
        reason_codes: stable_reason_codes(&[ReasonCode::ControlTowerModelOpsRefreshBuilt]),
    })
}

fn build_storage_report(
    config: &ModelOpsReviewClosureConfig,
) -> Result<ModelOpsReviewClosureStorageReport, String> {
    let mut artifact_count = 0usize;
    let mut storage_bytes = 0usize;
    for path in config
        .external_model_research_ops_paths
        .iter()
        .chain(config.model_review_queue_paths.iter())
        .chain(config.owner_model_review_action_paths.iter())
        .chain(config.watchlist_paths.iter())
        .chain(config.model_risk_profile_paths.iter())
        .chain(config.leaderboard_changelog_paths.iter())
        .chain(config.prediction_history_pack_paths.iter())
        .chain(config.model_ops_baseline_paths.iter())
        .chain(config.model_ops_current_paths.iter())
    {
        artifact_count += 1;
        storage_bytes += fs::metadata(path).map_err(|err| err.to_string())?.len() as usize;
    }
    Ok(ModelOpsReviewClosureStorageReport {
        artifact_count,
        storage_bytes,
        max_review_items: config.max_review_items,
        reason_codes: stable_reason_codes(&[ReasonCode::ModelOpsReviewClosureStorageBuilt]),
    })
}

fn build_final_summary(
    closure_report: &ModelReviewClosureReport,
    prediction_history_pack_report: &PredictionHistoryPackReport,
    operator_qa: &ModelOpsOperatorQAReport,
) -> String {
    [
        format!("review_closure_status={:?}", closure_report.closure_status),
        format!(
            "prediction_history_status={:?}",
            prediction_history_pack_report.history_status
        ),
        format!("operator_qa_status={:?}", operator_qa.qa_status),
        "runtime_status=HoldMamba3RuntimeDeferred".to_string(),
    ]
    .join("\n")
}

fn write_bundle(
    config: &ModelOpsReviewClosureConfig,
    bundle: &ModelOpsReviewClosureBundle,
) -> Result<(), String> {
    let dir = config.output_dir();
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    write_text_report(
        &dir,
        "model_review_closure.txt",
        &bundle.model_review_closure_report,
    )?;
    write_text_report(
        &dir,
        "prediction_history_pack.txt",
        &bundle.prediction_history_pack_report,
    )?;
    write_text_report(
        &dir,
        "model_ops_decision_log.txt",
        &bundle.model_ops_decision_log,
    )?;
    write_text_report(
        &dir,
        "model_ops_operator_qa.txt",
        &bundle.model_ops_operator_qa_report,
    )?;
    if let Some(report) = &bundle.model_ops_regression_guard_report {
        write_text_report(&dir, "model_ops_regression_guard.txt", report)?;
    }
    if let Some(report) = &bundle.control_tower_model_ops_refresh_report {
        write_text_report(&dir, "control_tower_model_ops_refresh.txt", report)?;
    }
    write_text_report(&dir, "storage_report.txt", &bundle.storage_report)?;
    fs::write(dir.join("summary.txt"), &bundle.final_summary).map_err(|err| err.to_string())?;
    Ok(())
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
        let actions = read_json_file::<Vec<OwnerModelReviewAction>>(Path::new(path))?;
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

fn empty_review_queue() -> ExternalModelReviewQueue {
    ExternalModelReviewQueue {
        queue_id: "empty".to_string(),
        pending_items: Vec::new(),
        reviewed_items: Vec::new(),
        deferred_items: Vec::new(),
        retired_items: Vec::new(),
        downgraded_items: Vec::new(),
        blocked_items: Vec::new(),
        reason_codes: Vec::new(),
    }
}

fn empty_watchlist() -> ExternalModelWatchlist {
    ExternalModelWatchlist {
        entries: Vec::new(),
        active_count: 0,
        retired_count: 0,
        diagnostic_count: 0,
        reason_codes: Vec::new(),
    }
}

fn empty_changelog() -> ModelLeaderboardChangeLog {
    ModelLeaderboardChangeLog {
        changes: Vec::new(),
        newly_eligible_count: 0,
        newly_blocked_count: 0,
        rank_change_count: 0,
        no_change_count: 0,
        reason_codes: Vec::new(),
    }
}

fn diagnostic_history_pack(id: &str) -> PredictionHistoryPackReport {
    PredictionHistoryPackReport {
        history_pack_id: id.to_string(),
        model_count: 0,
        version_count: 0,
        prediction_file_count: 0,
        comparable_version_count: 0,
        non_comparable_version_count: 0,
        missing_card_count: 0,
        missing_prediction_count: 0,
        records: Vec::new(),
        history_status: PredictionHistoryPackStatus::DiagnosticOnly,
        reason_codes: Vec::new(),
    }
}

fn decision_from_owner_action(
    action_kind: OwnerModelReviewActionKind,
) -> ModelReviewClosureDecision {
    match action_kind {
        OwnerModelReviewActionKind::MarkModelDiagnosticOnly => {
            ModelReviewClosureDecision::DowngradeToDiagnostic
        }
        OwnerModelReviewActionKind::RetireModelVersion => {
            ModelReviewClosureDecision::RetireModelVersion
        }
        OwnerModelReviewActionKind::RequestMorePredictions => {
            ModelReviewClosureDecision::RequestMorePredictions
        }
        OwnerModelReviewActionKind::RequestCalibrationReview => {
            ModelReviewClosureDecision::RequestCalibrationReview
        }
        OwnerModelReviewActionKind::RequestRiskReview => {
            ModelReviewClosureDecision::RequestRiskReview
        }
        OwnerModelReviewActionKind::DeferReview => ModelReviewClosureDecision::DeferReview,
        OwnerModelReviewActionKind::DismissReview => ModelReviewClosureDecision::DismissReview,
        _ => ModelReviewClosureDecision::KeepResearchCandidate,
    }
}

fn derive_policy_decision(
    item: &super::ExternalModelReviewItem,
    watch_entry: Option<&super::ExternalModelWatchlistEntry>,
    risk_profile: Option<&ModelEvidenceRiskProfile>,
    change: Option<&super::ModelLeaderboardChange>,
) -> ModelReviewClosureDecision {
    if matches!(
        watch_entry.map(|entry| entry.watch_status),
        Some(ExternalModelWatchStatus::DiagnosticOnly)
    ) {
        return ModelReviewClosureDecision::DowngradeToDiagnostic;
    }
    if item.item_kind == ExternalModelReviewItemKind::RetirementReview {
        return ModelReviewClosureDecision::RetireModelVersion;
    }
    if item.item_kind == ExternalModelReviewItemKind::CoverageWeakReview {
        return ModelReviewClosureDecision::RequestMorePredictions;
    }
    if item.item_kind == ExternalModelReviewItemKind::CalibrationDriftReview {
        return ModelReviewClosureDecision::RequestCalibrationReview;
    }
    if item.item_kind == ExternalModelReviewItemKind::RiskBehaviorReview {
        return ModelReviewClosureDecision::RequestRiskReview;
    }
    if let Some(profile) = risk_profile {
        return match profile.recommended_action {
            ModelEvidenceRecommendedAction::RequestMorePredictions => {
                ModelReviewClosureDecision::RequestMorePredictions
            }
            ModelEvidenceRecommendedAction::RequestCalibrationReview => {
                ModelReviewClosureDecision::RequestCalibrationReview
            }
            ModelEvidenceRecommendedAction::RequestRiskReview => {
                ModelReviewClosureDecision::RequestRiskReview
            }
            ModelEvidenceRecommendedAction::DowngradeToDiagnostic => {
                ModelReviewClosureDecision::DowngradeToDiagnostic
            }
            ModelEvidenceRecommendedAction::RetireModelVersion => {
                ModelReviewClosureDecision::RetireModelVersion
            }
            ModelEvidenceRecommendedAction::KeepResearchCandidate => {
                ModelReviewClosureDecision::KeepResearchCandidate
            }
        };
    }
    if matches!(
        change.map(|change| change.change_kind),
        Some(LeaderboardChangeKind::NewlyBlocked)
    ) {
        return ModelReviewClosureDecision::RequestMorePredictions;
    }
    ModelReviewClosureDecision::KeepResearchCandidate
}

fn closure_action_allowed(
    config: &ModelOpsReviewClosureConfig,
    action: ModelReviewClosureDecision,
    note: Option<&str>,
) -> bool {
    let note = note.unwrap_or_default().to_ascii_lowercase();
    if note.contains("live")
        || note.contains("runtime")
        || note.contains("train")
        || note.contains("broker")
        || note.contains("order")
        || note.contains("account")
    {
        return false;
    }
    match action {
        ModelReviewClosureDecision::KeepResearchCandidate => config.allow_keep_research_candidate,
        ModelReviewClosureDecision::DowngradeToDiagnostic => {
            config.allow_mark_diagnostic_only
                && (!config.require_owner_reason_for_downgrade || !note.trim().is_empty())
        }
        ModelReviewClosureDecision::RetireModelVersion => {
            config.allow_retire_version
                && (!config.require_owner_reason_for_retire || !note.trim().is_empty())
        }
        ModelReviewClosureDecision::RequestMorePredictions => config.allow_request_more_predictions,
        ModelReviewClosureDecision::RequestCalibrationReview => {
            config.allow_request_calibration_review
        }
        ModelReviewClosureDecision::RequestRiskReview => config.allow_request_risk_review,
        ModelReviewClosureDecision::DeferReview | ModelReviewClosureDecision::DismissReview => true,
    }
}

fn decision_kind_from_closure(decision: ModelReviewClosureDecision) -> ModelOpsDecisionKind {
    match decision {
        ModelReviewClosureDecision::KeepResearchCandidate => {
            ModelOpsDecisionKind::KeepResearchCandidate
        }
        ModelReviewClosureDecision::DowngradeToDiagnostic => {
            ModelOpsDecisionKind::DowngradeToDiagnostic
        }
        ModelReviewClosureDecision::RetireModelVersion => ModelOpsDecisionKind::RetireModelVersion,
        ModelReviewClosureDecision::RequestMorePredictions => {
            ModelOpsDecisionKind::RequestMorePredictions
        }
        ModelReviewClosureDecision::RequestCalibrationReview => {
            ModelOpsDecisionKind::RequestCalibrationReview
        }
        ModelReviewClosureDecision::RequestRiskReview => ModelOpsDecisionKind::RequestRiskReview,
        ModelReviewClosureDecision::DeferReview => ModelOpsDecisionKind::DeferReview,
        ModelReviewClosureDecision::DismissReview => ModelOpsDecisionKind::DismissReview,
    }
}

fn decision_kind_from_owner_action(action: OwnerModelReviewActionKind) -> ModelOpsDecisionKind {
    match action {
        OwnerModelReviewActionKind::WatchModel => ModelOpsDecisionKind::WatchModel,
        OwnerModelReviewActionKind::UnwatchModel => ModelOpsDecisionKind::UnwatchModel,
        OwnerModelReviewActionKind::MarkModelDiagnosticOnly => {
            ModelOpsDecisionKind::DowngradeToDiagnostic
        }
        OwnerModelReviewActionKind::RetireModelVersion => ModelOpsDecisionKind::RetireModelVersion,
        OwnerModelReviewActionKind::RequestMorePredictions => {
            ModelOpsDecisionKind::RequestMorePredictions
        }
        OwnerModelReviewActionKind::RequestCalibrationReview => {
            ModelOpsDecisionKind::RequestCalibrationReview
        }
        OwnerModelReviewActionKind::RequestRiskReview => ModelOpsDecisionKind::RequestRiskReview,
        OwnerModelReviewActionKind::DeferReview => ModelOpsDecisionKind::DeferReview,
        OwnerModelReviewActionKind::DismissReview => ModelOpsDecisionKind::DismissReview,
        _ => ModelOpsDecisionKind::KeepResearchCandidate,
    }
}

fn collect_model_keys_from_paths(paths: &[String]) -> BTreeSet<String> {
    paths
        .iter()
        .filter_map(|path| {
            let stem = Path::new(path).file_stem()?.to_string_lossy().to_string();
            let parts = stem.split('_').collect::<Vec<_>>();
            if parts.len() >= 3 {
                Some(model_key(parts[parts.len() - 2], parts[parts.len() - 1]))
            } else {
                None
            }
        })
        .collect()
}

fn enforce_model_limits_for_history(
    config: &PredictionHistoryPackConfig,
    keys: &BTreeSet<String>,
) -> Result<(), String> {
    let model_ids = keys
        .iter()
        .filter_map(|key| {
            key.split_once(':')
                .map(|(model_id, _)| model_id.to_string())
        })
        .collect::<BTreeSet<_>>();
    if model_ids.len() > config.max_models {
        return Err("max_models exceeded".to_string());
    }
    let mut versions_by_model = BTreeMap::<String, usize>::new();
    for key in keys {
        if let Some((model_id, _)) = key.split_once(':') {
            *versions_by_model.entry(model_id.to_string()).or_default() += 1;
        }
    }
    if versions_by_model
        .values()
        .any(|count| *count > config.max_versions_per_model)
    {
        return Err("max_versions_per_model exceeded".to_string());
    }
    Ok(())
}

fn prediction_identity_from_csv(path: &Path) -> Result<(String, String), String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| "prediction csv missing header".to_string())?;
    let first = lines
        .next()
        .ok_or_else(|| "prediction csv missing rows".to_string())?;
    let headers = header.split(',').collect::<Vec<_>>();
    let values = first.split(',').collect::<Vec<_>>();
    let model_id_idx = headers
        .iter()
        .position(|column| *column == "model_id")
        .ok_or_else(|| "prediction csv missing model_id".to_string())?;
    let model_version_idx = headers
        .iter()
        .position(|column| *column == "model_version")
        .ok_or_else(|| "prediction csv missing model_version".to_string())?;
    Ok((
        values
            .get(model_id_idx)
            .copied()
            .unwrap_or("unknown")
            .to_string(),
        values
            .get(model_version_idx)
            .copied()
            .unwrap_or("unknown")
            .to_string(),
    ))
}

fn model_key(model_id: &str, model_version: &str) -> String {
    format!("{model_id}:{model_version}")
}

fn write_text_report<T: Serialize>(dir: &Path, name: &str, value: &T) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    fs::write(dir.join(name), text).map_err(|err| err.to_string())
}

fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}
