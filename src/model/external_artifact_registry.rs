use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};
use serde::{Deserialize, Serialize};

use super::{
    ExternalModelCardV2, ExternalModelEvaluationReport, ExternalModelPromotionGateReport,
    ExternalModelPromotionGateStatus, ExternalPredictionAblationReport,
    ExternalPredictionImportStatus, ExternalPredictionImportV2Report, Mamba3FinLiteContractStatus,
    Mamba3FinLitePrototypeContract, SequenceExportManifest,
};

fn default_output_root() -> String {
    "target/soma_external_artifact_registry".to_string()
}

fn default_max_models() -> usize {
    8
}

fn default_max_versions_per_model() -> usize {
    8
}

fn default_max_artifacts() -> usize {
    128
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalModelArtifactRegistryConfig {
    pub registry_id: String,
    #[serde(default)]
    pub sequence_export_manifest_paths: Vec<String>,
    #[serde(default)]
    pub feature_schema_manifest_paths: Vec<String>,
    #[serde(default)]
    pub label_manifest_paths: Vec<String>,
    #[serde(default)]
    pub external_prediction_import_report_paths: Vec<String>,
    #[serde(default)]
    pub external_model_card_paths: Vec<String>,
    #[serde(default)]
    pub external_model_evaluation_report_paths: Vec<String>,
    #[serde(default)]
    pub external_vs_trinity_report_paths: Vec<String>,
    #[serde(default)]
    pub external_prediction_ablation_report_paths: Vec<String>,
    #[serde(default)]
    pub external_model_promotion_gate_paths: Vec<String>,
    #[serde(default)]
    pub mamba3fin_contract_paths: Vec<String>,
    #[serde(default)]
    pub baseline_reference_paths: Vec<String>,
    #[serde(default)]
    pub trinity_reference_paths: Vec<String>,
    #[serde(default)]
    pub no_trade_reference_paths: Vec<String>,
    #[serde(default)]
    pub risk_denied_reference_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_models")]
    pub max_models: usize,
    #[serde(default = "default_max_versions_per_model")]
    pub max_versions_per_model: usize,
    #[serde(default = "default_max_artifacts")]
    pub max_artifacts: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_model_card: bool,
    #[serde(default = "default_true")]
    pub require_evaluation_report: bool,
    #[serde(default = "default_true")]
    pub require_same_dataset_fingerprint_for_comparison: bool,
    #[serde(default = "default_true")]
    pub require_same_feature_schema_hash_for_comparison: bool,
    #[serde(default = "default_true")]
    pub require_same_label_manifest_hash_for_comparison: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ExternalModelArtifactRegistryConfig {
    fn default() -> Self {
        Self {
            registry_id: "sprint64-external-artifact-registry".to_string(),
            sequence_export_manifest_paths: Vec::new(),
            feature_schema_manifest_paths: Vec::new(),
            label_manifest_paths: Vec::new(),
            external_prediction_import_report_paths: Vec::new(),
            external_model_card_paths: Vec::new(),
            external_model_evaluation_report_paths: Vec::new(),
            external_vs_trinity_report_paths: Vec::new(),
            external_prediction_ablation_report_paths: Vec::new(),
            external_model_promotion_gate_paths: Vec::new(),
            mamba3fin_contract_paths: Vec::new(),
            baseline_reference_paths: Vec::new(),
            trinity_reference_paths: Vec::new(),
            no_trade_reference_paths: Vec::new(),
            risk_denied_reference_paths: Vec::new(),
            output_root: default_output_root(),
            max_models: default_max_models(),
            max_versions_per_model: default_max_versions_per_model(),
            max_artifacts: default_max_artifacts(),
            max_bytes: default_max_bytes(),
            require_model_card: true,
            require_evaluation_report: true,
            require_same_dataset_fingerprint_for_comparison: true,
            require_same_feature_schema_hash_for_comparison: true,
            require_same_label_manifest_hash_for_comparison: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl ExternalModelArtifactRegistryConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.registry_id.trim().is_empty() {
            return Err("external artifact registry id must not be empty".to_string());
        }
        if self.sequence_export_manifest_paths.is_empty() {
            return Err("at least one sequence_export_manifest_path is required".to_string());
        }
        if self.max_models == 0
            || self.max_versions_per_model == 0
            || self.max_artifacts == 0
            || self.max_bytes == 0
        {
            return Err("registry caps must be positive".to_string());
        }
        let all_paths = self
            .sequence_export_manifest_paths
            .iter()
            .chain(self.feature_schema_manifest_paths.iter())
            .chain(self.label_manifest_paths.iter())
            .chain(self.external_prediction_import_report_paths.iter())
            .chain(self.external_model_card_paths.iter())
            .chain(self.external_model_evaluation_report_paths.iter())
            .chain(self.external_vs_trinity_report_paths.iter())
            .chain(self.external_prediction_ablation_report_paths.iter())
            .chain(self.external_model_promotion_gate_paths.iter())
            .chain(self.mamba3fin_contract_paths.iter())
            .chain(self.baseline_reference_paths.iter())
            .chain(self.trinity_reference_paths.iter())
            .chain(self.no_trade_reference_paths.iter())
            .chain(self.risk_denied_reference_paths.iter())
            .chain(std::iter::once(&self.output_root));
        if all_paths.clone().any(|path| path.contains("://")) {
            return Err("external artifact registry config paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.registry_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExternalModelArtifactKind {
    ModelCard,
    PredictionCsv,
    ImportReport,
    EvaluationReport,
    VsTrinityReport,
    AblationReport,
    PromotionGateReport,
    Mamba3FinContract,
    SequenceExportManifest,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalModelRegistryEntry {
    pub entry_id: String,
    pub model_id: String,
    pub model_version: String,
    pub model_family: String,
    pub artifact_kind: ExternalModelArtifactKind,
    pub artifact_path: String,
    #[serde(default)]
    pub dataset_fingerprint: Option<String>,
    #[serde(default)]
    pub feature_schema_hash: Option<String>,
    #[serde(default)]
    pub label_manifest_hash: Option<String>,
    #[serde(default)]
    pub split_policy: Option<String>,
    #[serde(default)]
    pub prediction_schema_version: Option<String>,
    #[serde(default)]
    pub evaluation_status: Option<String>,
    #[serde(default)]
    pub promotion_status: Option<String>,
    #[serde(default)]
    pub calibration_status: Option<String>,
    #[serde(default)]
    pub risk_status: Option<String>,
    pub comparable: bool,
    pub diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelArtifactRegistryStatus {
    RegistryReady,
    RegistryReadyWithWarnings,
    MissingModelCards,
    MissingEvaluations,
    IncompatibleContracts,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalModelArtifactRegistry {
    pub registry_id: String,
    pub entries: Vec<ExternalModelRegistryEntry>,
    #[serde(default)]
    pub model_ids: Vec<String>,
    #[serde(default)]
    pub model_versions: Vec<String>,
    pub comparable_entries: usize,
    pub diagnostic_entries: usize,
    pub incompatible_entries: usize,
    pub unknown_entries: usize,
    pub registry_status: ExternalModelArtifactRegistryStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelVersionLineageStatus {
    LineageValid,
    MissingParent,
    SchemaChanged,
    LabelChanged,
    DatasetChanged,
    SplitChanged,
    UnknownLineage,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelVersionLineageRecord {
    pub model_id: String,
    pub model_version: String,
    #[serde(default)]
    pub parent_model_version: Option<String>,
    pub feature_schema_hash: String,
    pub label_manifest_hash: String,
    pub dataset_fingerprint: String,
    pub split_policy: String,
    pub lineage_status: ModelVersionLineageStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelCardLineageReport {
    pub records: Vec<ModelVersionLineageRecord>,
    pub valid_lineage_count: usize,
    pub changed_schema_count: usize,
    pub changed_label_count: usize,
    pub changed_dataset_count: usize,
    pub unknown_lineage_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionArtifactLineageStatus {
    PredictionLineageValid,
    DatasetMismatch,
    FeatureSchemaMismatch,
    LabelManifestMismatch,
    SplitPolicyMismatch,
    CoverageWeak,
    InvalidPredictions,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PredictionArtifactLineageReport {
    pub prediction_artifact_id: String,
    pub model_id: String,
    pub model_version: String,
    pub sequence_export_manifest_id: String,
    pub dataset_fingerprint_match: bool,
    pub feature_schema_match: bool,
    pub label_manifest_match: bool,
    pub split_policy_match: bool,
    #[serde(default)]
    pub coverage_ratio: Option<f64>,
    #[serde(default)]
    pub duplicate_count: Option<usize>,
    #[serde(default)]
    pub invalid_count: Option<usize>,
    pub lineage_status: PredictionArtifactLineageStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelEvaluationHistory {
    pub model_id: String,
    pub versions: Vec<String>,
    #[serde(default)]
    pub metrics_by_version: BTreeMap<String, BTreeMap<String, f64>>,
    #[serde(default)]
    pub best_version_by_metric: BTreeMap<String, String>,
    #[serde(default)]
    pub latest_vs_previous_delta: BTreeMap<String, f64>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalEvaluationHistoryStatus {
    HistoryReady,
    NeedMoreVersions,
    NeedComparableContracts,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalEvaluationHistoryReport {
    pub history_id: String,
    pub model_histories: Vec<ModelEvaluationHistory>,
    pub baseline_reference_summary: String,
    pub trinity_reference_summary: String,
    pub no_trade_reference_summary: String,
    pub risk_denied_reference_summary: String,
    #[serde(default)]
    pub latest_model_versions: BTreeMap<String, String>,
    #[serde(default)]
    pub previous_model_versions: BTreeMap<String, String>,
    #[serde(default)]
    pub metric_deltas: BTreeMap<String, BTreeMap<String, f64>>,
    pub history_status: ExternalEvaluationHistoryStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalibrationDriftStatus {
    Stable,
    MildDrift,
    SevereDrift,
    InsufficientHistory,
    NotComparable,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CalibrationDriftRecord {
    pub model_id: String,
    pub model_version: String,
    #[serde(default)]
    pub previous_model_version: Option<String>,
    #[serde(default)]
    pub brier_score_current: Option<f64>,
    #[serde(default)]
    pub brier_score_previous: Option<f64>,
    #[serde(default)]
    pub brier_delta: Option<f64>,
    #[serde(default)]
    pub ece_current: Option<f64>,
    #[serde(default)]
    pub ece_previous: Option<f64>,
    #[serde(default)]
    pub ece_delta: Option<f64>,
    #[serde(default)]
    pub confidence_shift: Option<f64>,
    pub calibration_status: CalibrationDriftStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CalibrationDriftReport {
    pub records: Vec<CalibrationDriftRecord>,
    pub stable_count: usize,
    pub mild_drift_count: usize,
    pub severe_drift_count: usize,
    pub insufficient_history_count: usize,
    pub drift_status: CalibrationDriftStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelVersionComparisonStatus {
    Improved,
    Regressed,
    Mixed,
    NoComparablePrevious,
    NotComparable,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalModelVersionComparisonReport {
    pub comparison_id: String,
    pub records: Vec<CalibrationDriftRecord>,
    pub model_id: String,
    pub latest_version: String,
    #[serde(default)]
    pub previous_version: Option<String>,
    #[serde(default)]
    pub metric_delta_summary: BTreeMap<String, f64>,
    #[serde(default)]
    pub calibration_delta_summary: BTreeMap<String, f64>,
    #[serde(default)]
    pub risk_delta_summary: BTreeMap<String, f64>,
    pub comparison_status: ExternalModelVersionComparisonStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaderboardEntryStatus {
    Eligible,
    EligibleWithWarnings,
    DiagnosticOnly,
    BlockedByCoverage,
    BlockedByModelCard,
    BlockedByCalibration,
    BlockedByRisk,
    BlockedByContractMismatch,
    BlockedByInsufficientRows,
    BlockedByAblationInstability,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConservativeLeaderboardEntry {
    #[serde(default)]
    pub rank: Option<usize>,
    pub model_id: String,
    pub model_version: String,
    pub model_family: String,
    pub dataset_fingerprint: String,
    pub feature_schema_hash: String,
    pub label_manifest_hash: String,
    pub coverage_ratio: f64,
    #[serde(default)]
    pub brier_score: Option<f64>,
    #[serde(default)]
    pub ece: Option<f64>,
    #[serde(default)]
    pub risk_adjusted_score: Option<f64>,
    #[serde(default)]
    pub net_return_proxy: Option<f64>,
    #[serde(default)]
    pub top_k_precision: Option<f64>,
    #[serde(default)]
    pub rank_correlation: Option<f64>,
    #[serde(default)]
    pub ablation_status: Option<String>,
    #[serde(default)]
    pub promotion_status: Option<String>,
    pub conservative_score: f64,
    #[serde(default)]
    pub penalty_summary: Vec<String>,
    pub entry_status: LeaderboardEntryStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConservativeExternalLeaderboardStatus {
    LeaderboardReady,
    LeaderboardReadyWithWarnings,
    NeedMoreModels,
    NeedMoreHistory,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConservativeExternalLeaderboard {
    pub leaderboard_id: String,
    pub entries: Vec<ConservativeLeaderboardEntry>,
    pub eligible_entries: usize,
    pub diagnostic_entries: usize,
    pub blocked_entries: usize,
    #[serde(default)]
    pub baseline_entry: Option<String>,
    #[serde(default)]
    pub trinity_entry: Option<String>,
    #[serde(default)]
    pub no_trade_entry: Option<String>,
    #[serde(default)]
    pub risk_denied_entry: Option<String>,
    pub leaderboard_status: ConservativeExternalLeaderboardStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LeaderboardPromotionPolicyConfig {
    pub policy_id: String,
    pub min_coverage_ratio: f64,
    pub max_ece: f64,
    pub max_brier_score: f64,
    pub min_risk_adjusted_score: f64,
    pub require_model_card: bool,
    pub require_calibration_stable: bool,
    pub require_risk_passed: bool,
    pub require_ablation_stable: bool,
    pub require_comparable_contract: bool,
    pub require_no_live_use: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for LeaderboardPromotionPolicyConfig {
    fn default() -> Self {
        Self {
            policy_id: "sprint64-default-policy".to_string(),
            min_coverage_ratio: 0.8,
            max_ece: 0.3,
            max_brier_score: 0.15,
            min_risk_adjusted_score: 0.0,
            require_model_card: true,
            require_calibration_stable: true,
            require_risk_passed: true,
            require_ablation_stable: true,
            require_comparable_contract: true,
            require_no_live_use: true,
            reason_codes: stable_reason_codes(&[ReasonCode::LeaderboardPromotionPolicyBuilt]),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaderboardPromotionPolicyStatus {
    PolicyReady,
    TooStrictForCurrentEvidence,
    NeedMoreEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LeaderboardPromotionPolicyReport {
    pub policy_id: String,
    pub model_results: Vec<String>,
    pub research_candidate_count: usize,
    pub diagnostic_only_count: usize,
    pub blocked_count: usize,
    pub policy_status: LeaderboardPromotionPolicyStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreviousExternalComparisonRecommendation {
    KeepLatestAsResearchCandidate,
    KeepPreviousAsResearchCandidate,
    NeedsMoreHistory,
    DowngradeToDiagnostic,
    BlockedByDrift,
    BlockedByRisk,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreviousExternalComparisonReport {
    pub model_id: String,
    pub latest_version: String,
    pub previous_versions: Vec<String>,
    pub comparable_versions: Vec<String>,
    pub non_comparable_versions: Vec<String>,
    #[serde(default)]
    pub metric_deltas: BTreeMap<String, f64>,
    pub drift_summary: String,
    pub recommendation: PreviousExternalComparisonRecommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalArtifactRegistryAuditStatus {
    Passed,
    PassedWithWarnings,
    FailedMissingArtifacts,
    FailedContractMismatch,
    FailedSecretScan,
    FailedUnsafeFields,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalArtifactRegistryAuditReport {
    pub registry_id: String,
    pub artifacts_scanned: usize,
    pub missing_artifacts: usize,
    pub incompatible_artifacts: usize,
    pub secret_like_fields: usize,
    pub order_account_fields: usize,
    pub unsafe_intended_use_count: usize,
    pub audit_status: ExternalArtifactRegistryAuditStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalLeaderboardPanel {
    pub registry_status: String,
    pub leaderboard_status: String,
    #[serde(default)]
    pub top_entries: Vec<String>,
    #[serde(default)]
    pub blocked_entries: Vec<String>,
    pub calibration_drift_status: String,
    pub latest_vs_previous_summary: String,
    pub mamba3fin_family_status: String,
    pub baseline_summary: String,
    pub trinity_summary: String,
    pub no_trade_summary: String,
    pub risk_denied_summary: String,
    #[serde(default)]
    pub next_actions: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl ExternalLeaderboardPanel {
    pub fn stabilize(&mut self) {
        self.top_entries = stable_ordered_strings(&self.top_entries);
        self.blocked_entries = stable_ordered_strings(&self.blocked_entries);
        self.next_actions = stable_ordered_strings(&self.next_actions);
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn to_text(&self) -> String {
        [
            format!("registry_status={}", self.registry_status),
            format!("leaderboard_status={}", self.leaderboard_status),
            format!("calibration_drift_status={}", self.calibration_drift_status),
            format!(
                "latest_vs_previous_summary={}",
                self.latest_vs_previous_summary
            ),
            format!("mamba3fin_family_status={}", self.mamba3fin_family_status),
            format!("baseline_summary={}", self.baseline_summary),
            format!("trinity_summary={}", self.trinity_summary),
            format!("no_trade_summary={}", self.no_trade_summary),
            format!("risk_denied_summary={}", self.risk_denied_summary),
            format!("top_entries={}", self.top_entries.join("|")),
            format!("blocked_entries={}", self.blocked_entries.join("|")),
            format!("next_actions={}", self.next_actions.join(" || ")),
        ]
        .join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalArtifactRegistryBundle {
    pub artifact_registry: ExternalModelArtifactRegistry,
    pub model_card_lineage_report: ModelCardLineageReport,
    pub prediction_artifact_lineage_report: PredictionArtifactLineageReport,
    pub evaluation_history_report: ExternalEvaluationHistoryReport,
    pub calibration_drift_report: CalibrationDriftReport,
    pub model_version_comparison_report: ExternalModelVersionComparisonReport,
    pub conservative_leaderboard: ConservativeExternalLeaderboard,
    pub leaderboard_promotion_policy_report: LeaderboardPromotionPolicyReport,
    pub previous_external_comparison_report: PreviousExternalComparisonReport,
    pub artifact_registry_audit_report: ExternalArtifactRegistryAuditReport,
    #[serde(default)]
    pub control_tower_external_leaderboard_panel_summary: Option<ExternalLeaderboardPanel>,
    pub storage_report: RegistryStorageReport,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryStorageReport {
    pub artifact_count: usize,
    pub storage_bytes: usize,
    pub max_bytes: usize,
    pub within_budget: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExternalArtifactRegistryRunner;

#[derive(Clone, Debug)]
struct CanonicalContract {
    manifest_path: String,
    export_id: String,
    dataset_fingerprint: String,
    feature_schema_hash: String,
    label_manifest_hash: String,
    split_policy: String,
}

#[derive(Clone, Debug)]
struct EvaluationArtifact {
    path: String,
    report: ExternalModelEvaluationReport,
}

#[derive(Clone, Debug)]
struct AblationArtifact {
    path: String,
    report: ExternalPredictionAblationReport,
}

#[derive(Clone, Debug)]
struct PromotionArtifact {
    path: String,
    report: ExternalModelPromotionGateReport,
}

#[derive(Clone, Debug)]
struct ImportArtifact {
    path: String,
    report: ExternalPredictionImportV2Report,
}

impl ExternalArtifactRegistryRunner {
    pub fn run(
        &self,
        config: &ExternalModelArtifactRegistryConfig,
    ) -> Result<ExternalArtifactRegistryBundle, String> {
        config.validate()?;
        let canonical_contracts = load_canonical_contracts(config)?;
        let canonical = canonical_contracts
            .first()
            .ok_or_else(|| "missing canonical contract".to_string())?
            .clone();
        let model_cards = load_model_cards(&config.external_model_card_paths)?;
        let import_reports = load_import_reports(&config.external_prediction_import_report_paths)?;
        let evaluation_reports = load_evaluations(&config.external_model_evaluation_report_paths)?;
        let ablation_reports = load_ablations(&config.external_prediction_ablation_report_paths)?;
        let promotion_reports = load_promotions(&config.external_model_promotion_gate_paths)?;
        let mamba_contracts = load_mamba_contracts(&config.mamba3fin_contract_paths)?;
        let prediction_entries =
            load_prediction_csv_entries(config, &canonical_contracts, &model_cards)?;
        let registry = build_registry(
            config,
            &canonical,
            &canonical_contracts,
            &model_cards,
            &prediction_entries,
            &import_reports,
            &evaluation_reports,
            &ablation_reports,
            &promotion_reports,
            &mamba_contracts,
        )?;
        let lineage_report =
            build_model_card_lineage_report(config, &canonical_contracts, &model_cards);
        let prediction_lineage_report = build_prediction_artifact_lineage_report(
            config,
            &canonical,
            &prediction_entries,
            &import_reports,
        );
        let history_report = build_external_evaluation_history_report(
            config,
            &evaluation_reports,
            reference_summary(&config.baseline_reference_paths),
            reference_summary(&config.trinity_reference_paths),
            reference_summary(&config.no_trade_reference_paths),
            reference_summary(&config.risk_denied_reference_paths),
        );
        let drift_report = build_calibration_drift_report(&evaluation_reports);
        let version_comparison = build_external_model_version_comparison_report(
            config,
            &evaluation_reports,
            &drift_report,
        );
        let leaderboard = build_conservative_external_leaderboard(
            config,
            &canonical,
            &model_cards,
            &evaluation_reports,
            &ablation_reports,
            &promotion_reports,
        );
        let policy_report = build_leaderboard_promotion_policy_report(
            &LeaderboardPromotionPolicyConfig::default(),
            &leaderboard,
            &drift_report,
        );
        let previous_comparison =
            build_previous_external_comparison_report(&version_comparison, &drift_report);
        let audit_report = build_registry_audit_report(config, &registry, &model_cards)?;
        let panel = Some(build_external_leaderboard_panel(
            &registry,
            &leaderboard,
            &drift_report,
            &version_comparison,
            &policy_report,
            &previous_comparison,
        ));
        let storage_report = build_storage_report(config, &registry.entries)?;
        let final_summary =
            build_final_summary(&registry, &history_report, &drift_report, &leaderboard);
        let bundle = ExternalArtifactRegistryBundle {
            artifact_registry: registry,
            model_card_lineage_report: lineage_report,
            prediction_artifact_lineage_report: prediction_lineage_report,
            evaluation_history_report: history_report,
            calibration_drift_report: drift_report,
            model_version_comparison_report: version_comparison,
            conservative_leaderboard: leaderboard,
            leaderboard_promotion_policy_report: policy_report,
            previous_external_comparison_report: previous_comparison,
            artifact_registry_audit_report: audit_report,
            control_tower_external_leaderboard_panel_summary: panel,
            storage_report,
            final_summary,
            reason_codes: stable_reason_codes(&[
                ReasonCode::ExternalArtifactRegistryRunnerBuilt,
                ReasonCode::ExternalArtifactRegistryBundleBuilt,
            ]),
        };
        write_bundle(config, &bundle)?;
        Ok(bundle)
    }

    pub fn run_registry(
        &self,
        config: &ExternalModelArtifactRegistryConfig,
    ) -> Result<ExternalModelArtifactRegistry, String> {
        Ok(self.run(config)?.artifact_registry)
    }

    pub fn run_history(
        &self,
        config: &ExternalModelArtifactRegistryConfig,
    ) -> Result<ExternalEvaluationHistoryReport, String> {
        Ok(self.run(config)?.evaluation_history_report)
    }

    pub fn run_calibration_drift(
        &self,
        config: &ExternalModelArtifactRegistryConfig,
    ) -> Result<CalibrationDriftReport, String> {
        Ok(self.run(config)?.calibration_drift_report)
    }

    pub fn run_version_comparison(
        &self,
        config: &ExternalModelArtifactRegistryConfig,
    ) -> Result<ExternalModelVersionComparisonReport, String> {
        Ok(self.run(config)?.model_version_comparison_report)
    }

    pub fn run_leaderboard(
        &self,
        config: &ExternalModelArtifactRegistryConfig,
    ) -> Result<ConservativeExternalLeaderboard, String> {
        Ok(self.run(config)?.conservative_leaderboard)
    }

    pub fn run_audit(
        &self,
        config: &ExternalModelArtifactRegistryConfig,
    ) -> Result<ExternalArtifactRegistryAuditReport, String> {
        Ok(self.run(config)?.artifact_registry_audit_report)
    }
}

fn load_canonical_contracts(
    config: &ExternalModelArtifactRegistryConfig,
) -> Result<Vec<CanonicalContract>, String> {
    let mut out = Vec::new();
    for (idx, path) in config.sequence_export_manifest_paths.iter().enumerate() {
        let manifest: SequenceExportManifest = read_json_file(Path::new(path))?;
        let dataset_contract_hashes = read_dataset_contract_hashes(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(&manifest.dataset_csv_path)
                .as_path(),
        );
        let feature_hash = dataset_contract_hashes
            .as_ref()
            .map(|(feature_hash, _)| feature_hash.clone())
            .or_else(|| {
                config
                    .feature_schema_manifest_paths
                    .get(idx)
                    .or_else(|| config.feature_schema_manifest_paths.first())
                    .map(|path| stable_hash_string(&fs::read_to_string(path).unwrap_or_default()))
            })
            .unwrap_or_else(|| "unknown-feature-hash".to_string());
        let label_hash = dataset_contract_hashes
            .as_ref()
            .map(|(_, label_hash)| label_hash.clone())
            .or_else(|| {
                config
                    .label_manifest_paths
                    .get(idx)
                    .or_else(|| config.label_manifest_paths.first())
                    .map(|path| stable_hash_string(&fs::read_to_string(path).unwrap_or_default()))
            })
            .unwrap_or_else(|| "unknown-label-hash".to_string());
        out.push(CanonicalContract {
            manifest_path: path.clone(),
            export_id: manifest.export_id.clone(),
            dataset_fingerprint: manifest.fingerprint.clone(),
            feature_schema_hash: feature_hash,
            label_manifest_hash: label_hash,
            split_policy: "ChronologicalHoldout".to_string(),
        });
    }
    Ok(out)
}

fn read_dataset_contract_hashes(path: &Path) -> Option<(String, String)> {
    let text = fs::read_to_string(path).ok()?;
    let mut lines = text.lines();
    let header = lines.next()?;
    let first = lines.next()?;
    let header_cols: Vec<&str> = header.split(',').collect();
    let first_cols: Vec<&str> = first.split(',').collect();
    let feature_idx = header_cols
        .iter()
        .position(|column| *column == "feature_schema_hash")?;
    let label_idx = header_cols
        .iter()
        .position(|column| *column == "label_manifest_hash")?;
    Some((
        first_cols.get(feature_idx)?.to_string(),
        first_cols.get(label_idx)?.to_string(),
    ))
}

fn load_model_cards(paths: &[String]) -> Result<Vec<(String, ExternalModelCardV2)>, String> {
    let mut out = Vec::new();
    for path in paths {
        out.push((path.clone(), read_json_file(Path::new(path))?));
    }
    Ok(out)
}

fn load_import_reports(paths: &[String]) -> Result<Vec<ImportArtifact>, String> {
    let mut out = Vec::new();
    for path in paths {
        out.push(ImportArtifact {
            path: path.clone(),
            report: read_json_file(Path::new(path))?,
        });
    }
    Ok(out)
}

fn load_evaluations(paths: &[String]) -> Result<Vec<EvaluationArtifact>, String> {
    let mut out = Vec::new();
    for path in paths {
        out.push(EvaluationArtifact {
            path: path.clone(),
            report: read_json_file(Path::new(path))?,
        });
    }
    Ok(out)
}

fn load_ablations(paths: &[String]) -> Result<Vec<AblationArtifact>, String> {
    let mut out = Vec::new();
    for path in paths {
        out.push(AblationArtifact {
            path: path.clone(),
            report: read_json_file(Path::new(path))?,
        });
    }
    Ok(out)
}

fn load_promotions(paths: &[String]) -> Result<Vec<PromotionArtifact>, String> {
    let mut out = Vec::new();
    for path in paths {
        out.push(PromotionArtifact {
            path: path.clone(),
            report: read_json_file(Path::new(path))?,
        });
    }
    Ok(out)
}

fn load_mamba_contracts(
    paths: &[String],
) -> Result<Vec<(String, Mamba3FinLitePrototypeContract)>, String> {
    let mut out = Vec::new();
    for path in paths {
        out.push((path.clone(), read_json_file(Path::new(path))?));
    }
    Ok(out)
}

fn load_prediction_csv_entries(
    config: &ExternalModelArtifactRegistryConfig,
    contracts: &[CanonicalContract],
    model_cards: &[(String, ExternalModelCardV2)],
) -> Result<Vec<ExternalModelRegistryEntry>, String> {
    let mut entries = Vec::new();
    for path in &config.external_prediction_import_report_paths {
        let _ = path;
    }
    let card_map: BTreeMap<(String, String), ExternalModelCardV2> = model_cards
        .iter()
        .map(|(_, card)| {
            (
                (card.model_id.clone(), card.model_version.clone()),
                card.clone(),
            )
        })
        .collect();
    let contract = contracts
        .first()
        .cloned()
        .ok_or_else(|| "missing canonical contract".to_string())?;
    for path in discover_prediction_csv_paths(config) {
        let csv = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let mut lines = csv.lines();
        let header = lines.next().unwrap_or_default();
        let header_cols: Vec<&str> = header.split(',').collect();
        let first = lines.next().unwrap_or_default();
        let first_cols: Vec<&str> = first.split(',').collect();
        let model_id = csv_lookup(&header_cols, &first_cols, "model_id").unwrap_or("unknown");
        let model_version =
            csv_lookup(&header_cols, &first_cols, "model_version").unwrap_or("unknown");
        let card = card_map.get(&(model_id.to_string(), model_version.to_string()));
        let comparable = card
            .map(|card| {
                card.feature_schema_hash == contract.feature_schema_hash
                    && card.label_manifest_hash == contract.label_manifest_hash
            })
            .unwrap_or(false);
        entries.push(ExternalModelRegistryEntry {
            entry_id: format!("prediction-csv:{model_id}:{model_version}"),
            model_id: model_id.to_string(),
            model_version: model_version.to_string(),
            model_family: card
                .map(|card| format!("{:?}", card.model_family))
                .unwrap_or_else(|| "Unknown".to_string()),
            artifact_kind: ExternalModelArtifactKind::PredictionCsv,
            artifact_path: path,
            dataset_fingerprint: Some(contract.dataset_fingerprint.clone()),
            feature_schema_hash: card.map(|card| card.feature_schema_hash.clone()),
            label_manifest_hash: card.map(|card| card.label_manifest_hash.clone()),
            split_policy: card.map(|card| card.split_policy.clone()),
            prediction_schema_version: Some("v2".to_string()),
            evaluation_status: None,
            promotion_status: None,
            calibration_status: None,
            risk_status: None,
            comparable,
            diagnostic_only: false,
            reason_codes: stable_reason_codes(&[ReasonCode::ExternalArtifactRegistryEntryBuilt]),
        });
    }
    Ok(entries)
}

fn discover_prediction_csv_paths(_config: &ExternalModelArtifactRegistryConfig) -> Vec<String> {
    let mut out = Vec::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint64_data");
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_prediction_csv = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("external_predictions_"))
                .unwrap_or(false);
            if is_prediction_csv && path.extension().and_then(|ext| ext.to_str()) == Some("csv") {
                out.push(path.display().to_string());
            }
        }
    }
    stable_ordered_strings(&out)
}

fn build_registry(
    config: &ExternalModelArtifactRegistryConfig,
    canonical: &CanonicalContract,
    contracts: &[CanonicalContract],
    model_cards: &[(String, ExternalModelCardV2)],
    prediction_entries: &[ExternalModelRegistryEntry],
    import_reports: &[ImportArtifact],
    evaluation_reports: &[EvaluationArtifact],
    ablation_reports: &[AblationArtifact],
    promotion_reports: &[PromotionArtifact],
    mamba_contracts: &[(String, Mamba3FinLitePrototypeContract)],
) -> Result<ExternalModelArtifactRegistry, String> {
    let mut entries = Vec::new();
    for contract in contracts {
        entries.push(ExternalModelRegistryEntry {
            entry_id: format!("sequence-export:{}", contract.export_id),
            model_id: "registry".to_string(),
            model_version: contract.export_id.clone(),
            model_family: "SequenceExport".to_string(),
            artifact_kind: ExternalModelArtifactKind::SequenceExportManifest,
            artifact_path: contract.manifest_path.clone(),
            dataset_fingerprint: Some(contract.dataset_fingerprint.clone()),
            feature_schema_hash: Some(contract.feature_schema_hash.clone()),
            label_manifest_hash: Some(contract.label_manifest_hash.clone()),
            split_policy: Some(contract.split_policy.clone()),
            prediction_schema_version: None,
            evaluation_status: None,
            promotion_status: None,
            calibration_status: None,
            risk_status: None,
            comparable: true,
            diagnostic_only: true,
            reason_codes: stable_reason_codes(&[ReasonCode::ExternalArtifactRegistryEntryBuilt]),
        });
    }
    for (idx, (path, card)) in model_cards.iter().enumerate() {
        let contract = contracts
            .get(idx)
            .or_else(|| contracts.first())
            .unwrap_or(canonical);
        let comparable = (!config.require_same_feature_schema_hash_for_comparison
            || card.feature_schema_hash == contract.feature_schema_hash)
            && (!config.require_same_label_manifest_hash_for_comparison
                || card.label_manifest_hash == contract.label_manifest_hash)
            && (!config.require_same_dataset_fingerprint_for_comparison
                || contract.dataset_fingerprint == canonical.dataset_fingerprint);
        entries.push(ExternalModelRegistryEntry {
            entry_id: format!("model-card:{}:{}", card.model_id, card.model_version),
            model_id: card.model_id.clone(),
            model_version: card.model_version.clone(),
            model_family: format!("{:?}", card.model_family),
            artifact_kind: ExternalModelArtifactKind::ModelCard,
            artifact_path: path.clone(),
            dataset_fingerprint: Some(contract.dataset_fingerprint.clone()),
            feature_schema_hash: Some(card.feature_schema_hash.clone()),
            label_manifest_hash: Some(card.label_manifest_hash.clone()),
            split_policy: Some(card.split_policy.clone()),
            prediction_schema_version: None,
            evaluation_status: None,
            promotion_status: None,
            calibration_status: None,
            risk_status: None,
            comparable,
            diagnostic_only: !comparable,
            reason_codes: stable_reason_codes(&[ReasonCode::ExternalArtifactRegistryEntryBuilt]),
        });
    }
    entries.extend_from_slice(prediction_entries);
    for artifact in import_reports {
        let (import_model_id, import_model_version) =
            infer_model_identity_from_import_path(&artifact.path);
        entries.push(ExternalModelRegistryEntry {
            entry_id: format!(
                "import-report:{}:{}",
                artifact.report.import_id, artifact.report.schema_version
            ),
            model_id: import_model_id,
            model_version: import_model_version,
            model_family: "ExternalImport".to_string(),
            artifact_kind: ExternalModelArtifactKind::ImportReport,
            artifact_path: artifact.path.clone(),
            dataset_fingerprint: Some(artifact.report.manifest_fingerprint.clone()),
            feature_schema_hash: None,
            label_manifest_hash: None,
            split_policy: None,
            prediction_schema_version: Some(artifact.report.schema_version.clone()),
            evaluation_status: Some(format!("{:?}", artifact.report.import_status)),
            promotion_status: None,
            calibration_status: None,
            risk_status: None,
            comparable: artifact.report.import_status
                != ExternalPredictionImportStatus::BlockedByModelCard,
            diagnostic_only: false,
            reason_codes: stable_reason_codes(&[ReasonCode::ExternalArtifactRegistryEntryBuilt]),
        });
    }
    for artifact in evaluation_reports {
        let calibration_status = artifact
            .report
            .calibration_metrics
            .ece
            .map(|ece| if ece <= 0.3 { "Stable" } else { "DriftRisk" })
            .unwrap_or("Unknown");
        let risk_status = artifact
            .report
            .risk_metrics
            .risk_adjusted_score
            .map(|score| if score >= 0.0 { "Pass" } else { "Fail" })
            .unwrap_or("Unknown");
        entries.push(ExternalModelRegistryEntry {
            entry_id: format!(
                "evaluation:{}:{}",
                artifact.report.model_id, artifact.report.model_version
            ),
            model_id: artifact.report.model_id.clone(),
            model_version: artifact.report.model_version.clone(),
            model_family: "Evaluation".to_string(),
            artifact_kind: ExternalModelArtifactKind::EvaluationReport,
            artifact_path: artifact.path.clone(),
            dataset_fingerprint: Some(canonical.dataset_fingerprint.clone()),
            feature_schema_hash: Some(canonical.feature_schema_hash.clone()),
            label_manifest_hash: Some(canonical.label_manifest_hash.clone()),
            split_policy: Some(canonical.split_policy.clone()),
            prediction_schema_version: Some("v2".to_string()),
            evaluation_status: Some(format!("{:?}", artifact.report.evaluation_status)),
            promotion_status: None,
            calibration_status: Some(calibration_status.to_string()),
            risk_status: Some(risk_status.to_string()),
            comparable: true,
            diagnostic_only: false,
            reason_codes: stable_reason_codes(&[ReasonCode::ExternalArtifactRegistryEntryBuilt]),
        });
    }
    for artifact in ablation_reports {
        entries.push(ExternalModelRegistryEntry {
            entry_id: format!(
                "ablation:{}:{}",
                artifact.report.model_id, artifact.report.model_version
            ),
            model_id: artifact.report.model_id.clone(),
            model_version: artifact.report.model_version.clone(),
            model_family: "Ablation".to_string(),
            artifact_kind: ExternalModelArtifactKind::AblationReport,
            artifact_path: artifact.path.clone(),
            dataset_fingerprint: Some(canonical.dataset_fingerprint.clone()),
            feature_schema_hash: Some(canonical.feature_schema_hash.clone()),
            label_manifest_hash: Some(canonical.label_manifest_hash.clone()),
            split_policy: Some(canonical.split_policy.clone()),
            prediction_schema_version: Some("v2".to_string()),
            evaluation_status: None,
            promotion_status: Some(format!("{:?}", artifact.report.ablation_status)),
            calibration_status: None,
            risk_status: None,
            comparable: true,
            diagnostic_only: false,
            reason_codes: stable_reason_codes(&[ReasonCode::ExternalArtifactRegistryEntryBuilt]),
        });
    }
    for artifact in promotion_reports {
        entries.push(ExternalModelRegistryEntry {
            entry_id: format!(
                "promotion:{}:{}",
                artifact.report.model_id, artifact.report.model_version
            ),
            model_id: artifact.report.model_id.clone(),
            model_version: artifact.report.model_version.clone(),
            model_family: "PromotionGate".to_string(),
            artifact_kind: ExternalModelArtifactKind::PromotionGateReport,
            artifact_path: artifact.path.clone(),
            dataset_fingerprint: Some(canonical.dataset_fingerprint.clone()),
            feature_schema_hash: Some(canonical.feature_schema_hash.clone()),
            label_manifest_hash: Some(canonical.label_manifest_hash.clone()),
            split_policy: Some(canonical.split_policy.clone()),
            prediction_schema_version: Some("v2".to_string()),
            evaluation_status: None,
            promotion_status: Some(format!("{:?}", artifact.report.gate_status)),
            calibration_status: None,
            risk_status: None,
            comparable: true,
            diagnostic_only: !matches!(
                artifact.report.gate_status,
                ExternalModelPromotionGateStatus::ResearchCandidate
            ),
            reason_codes: stable_reason_codes(&[ReasonCode::ExternalArtifactRegistryEntryBuilt]),
        });
    }
    for (path, contract) in mamba_contracts {
        let (model_id, model_version) = infer_model_identity_from_path(path);
        entries.push(ExternalModelRegistryEntry {
            entry_id: format!("mamba-contract:{model_id}:{model_version}"),
            model_id,
            model_version,
            model_family: "Mamba3FinLiteFamily".to_string(),
            artifact_kind: ExternalModelArtifactKind::Mamba3FinContract,
            artifact_path: path.clone(),
            dataset_fingerprint: Some(canonical.dataset_fingerprint.clone()),
            feature_schema_hash: Some(contract.required_feature_schema_hash.clone()),
            label_manifest_hash: Some(contract.required_label_manifest_hash.clone()),
            split_policy: Some(canonical.split_policy.clone()),
            prediction_schema_version: Some(contract.required_prediction_schema_version.clone()),
            evaluation_status: None,
            promotion_status: Some(format!("{:?}", contract.contract_status)),
            calibration_status: None,
            risk_status: None,
            comparable: matches!(
                contract.contract_status,
                Mamba3FinLiteContractStatus::ContractReady
            ),
            diagnostic_only: true,
            reason_codes: stable_reason_codes(&[ReasonCode::ExternalArtifactRegistryEntryBuilt]),
        });
    }
    let artifact_count = entries.len();
    if artifact_count > config.max_artifacts {
        return Err("max_artifacts exceeded".to_string());
    }
    let model_ids = stable_ordered_strings(
        &entries
            .iter()
            .map(|entry| entry.model_id.clone())
            .filter(|model_id| model_id != "registry")
            .collect::<Vec<_>>(),
    );
    if BTreeSet::<String>::from_iter(model_ids.iter().cloned()).len() > config.max_models {
        return Err("max_models exceeded".to_string());
    }
    let mut versions_by_model: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for entry in &entries {
        versions_by_model
            .entry(entry.model_id.clone())
            .or_default()
            .insert(entry.model_version.clone());
    }
    if versions_by_model
        .values()
        .any(|versions| versions.len() > config.max_versions_per_model)
    {
        return Err("max_versions_per_model exceeded".to_string());
    }
    let comparable_entries = entries.iter().filter(|entry| entry.comparable).count();
    let diagnostic_entries = entries.iter().filter(|entry| entry.diagnostic_only).count();
    let incompatible_entries = entries.iter().filter(|entry| !entry.comparable).count();
    let unknown_entries = entries
        .iter()
        .filter(|entry| matches!(entry.artifact_kind, ExternalModelArtifactKind::Unknown))
        .count();
    let registry_status = if config.require_model_card && model_cards.is_empty() {
        ExternalModelArtifactRegistryStatus::MissingModelCards
    } else if config.require_evaluation_report && evaluation_reports.is_empty() {
        ExternalModelArtifactRegistryStatus::MissingEvaluations
    } else if incompatible_entries > 0 {
        ExternalModelArtifactRegistryStatus::IncompatibleContracts
    } else if diagnostic_entries > 0 {
        ExternalModelArtifactRegistryStatus::RegistryReadyWithWarnings
    } else {
        ExternalModelArtifactRegistryStatus::RegistryReady
    };
    Ok(ExternalModelArtifactRegistry {
        registry_id: config.registry_id.clone(),
        entries,
        model_ids: stable_ordered_strings(&versions_by_model.keys().cloned().collect::<Vec<_>>()),
        model_versions: stable_ordered_strings(
            &versions_by_model
                .values()
                .flat_map(|versions| versions.iter().cloned())
                .collect::<Vec<_>>(),
        ),
        comparable_entries,
        diagnostic_entries,
        incompatible_entries,
        unknown_entries,
        registry_status,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelArtifactRegistryBuilt]),
    })
}

fn build_model_card_lineage_report(
    _config: &ExternalModelArtifactRegistryConfig,
    contracts: &[CanonicalContract],
    model_cards: &[(String, ExternalModelCardV2)],
) -> ModelCardLineageReport {
    let mut grouped: BTreeMap<String, Vec<(usize, &ExternalModelCardV2)>> = BTreeMap::new();
    for (idx, (_, card)) in model_cards.iter().enumerate() {
        grouped
            .entry(card.model_id.clone())
            .or_default()
            .push((idx, card));
    }
    let mut records = Vec::new();
    for (model_id, mut versions) in grouped {
        versions.sort_by(|left, right| left.1.model_version.cmp(&right.1.model_version));
        for (idx, (_, card)) in versions.iter().enumerate() {
            let parent = idx.checked_sub(1).and_then(|prev| versions.get(prev));
            let contract = contracts
                .get(idx)
                .or_else(|| contracts.first())
                .cloned()
                .unwrap_or_else(|| CanonicalContract {
                    manifest_path: "unknown".to_string(),
                    export_id: "unknown".to_string(),
                    dataset_fingerprint: "unknown".to_string(),
                    feature_schema_hash: card.feature_schema_hash.clone(),
                    label_manifest_hash: card.label_manifest_hash.clone(),
                    split_policy: card.split_policy.clone(),
                });
            let lineage_status = if let Some((prev_idx, prev_card)) = parent {
                let prev_contract = contracts
                    .get(*prev_idx)
                    .or_else(|| contracts.first())
                    .unwrap_or(&contract);
                if card.feature_schema_hash != prev_card.feature_schema_hash {
                    ModelVersionLineageStatus::SchemaChanged
                } else if card.label_manifest_hash != prev_card.label_manifest_hash {
                    ModelVersionLineageStatus::LabelChanged
                } else if contract.dataset_fingerprint != prev_contract.dataset_fingerprint {
                    ModelVersionLineageStatus::DatasetChanged
                } else if card.split_policy != prev_card.split_policy {
                    ModelVersionLineageStatus::SplitChanged
                } else {
                    ModelVersionLineageStatus::LineageValid
                }
            } else if versions.len() == 1 {
                ModelVersionLineageStatus::UnknownLineage
            } else {
                ModelVersionLineageStatus::MissingParent
            };
            records.push(ModelVersionLineageRecord {
                model_id: model_id.clone(),
                model_version: card.model_version.clone(),
                parent_model_version: parent.map(|(_, card)| card.model_version.clone()),
                feature_schema_hash: card.feature_schema_hash.clone(),
                label_manifest_hash: card.label_manifest_hash.clone(),
                dataset_fingerprint: contract.dataset_fingerprint.clone(),
                split_policy: card.split_policy.clone(),
                lineage_status,
                reason_codes: stable_reason_codes(&[ReasonCode::ModelCardLineageReportBuilt]),
            });
        }
    }
    ModelCardLineageReport {
        valid_lineage_count: records
            .iter()
            .filter(|record| {
                matches!(
                    record.lineage_status,
                    ModelVersionLineageStatus::LineageValid
                )
            })
            .count(),
        changed_schema_count: records
            .iter()
            .filter(|record| {
                matches!(
                    record.lineage_status,
                    ModelVersionLineageStatus::SchemaChanged
                )
            })
            .count(),
        changed_label_count: records
            .iter()
            .filter(|record| {
                matches!(
                    record.lineage_status,
                    ModelVersionLineageStatus::LabelChanged
                )
            })
            .count(),
        changed_dataset_count: records
            .iter()
            .filter(|record| {
                matches!(
                    record.lineage_status,
                    ModelVersionLineageStatus::DatasetChanged
                )
            })
            .count(),
        unknown_lineage_count: records
            .iter()
            .filter(|record| {
                matches!(
                    record.lineage_status,
                    ModelVersionLineageStatus::UnknownLineage
                        | ModelVersionLineageStatus::MissingParent
                )
            })
            .count(),
        records,
        reason_codes: stable_reason_codes(&[ReasonCode::ModelCardLineageReportBuilt]),
    }
}

fn build_prediction_artifact_lineage_report(
    config: &ExternalModelArtifactRegistryConfig,
    canonical: &CanonicalContract,
    prediction_entries: &[ExternalModelRegistryEntry],
    import_reports: &[ImportArtifact],
) -> PredictionArtifactLineageReport {
    let import_map: BTreeMap<(String, String), &ExternalPredictionImportV2Report> = import_reports
        .iter()
        .map(|artifact| {
            let (model_id, model_version) = infer_model_identity_from_import_path(&artifact.path);
            ((model_id, model_version), &artifact.report)
        })
        .collect();
    let entry = prediction_entries
        .first()
        .cloned()
        .unwrap_or_else(|| ExternalModelRegistryEntry {
            entry_id: "prediction-csv:unknown".to_string(),
            model_id: "unknown".to_string(),
            model_version: "unknown".to_string(),
            model_family: "Unknown".to_string(),
            artifact_kind: ExternalModelArtifactKind::PredictionCsv,
            artifact_path: "unknown".to_string(),
            dataset_fingerprint: None,
            feature_schema_hash: None,
            label_manifest_hash: None,
            split_policy: None,
            prediction_schema_version: None,
            evaluation_status: None,
            promotion_status: None,
            calibration_status: None,
            risk_status: None,
            comparable: false,
            diagnostic_only: true,
            reason_codes: Vec::new(),
        });
    let import = import_map
        .get(&(entry.model_id.clone(), entry.model_version.clone()))
        .copied();
    let dataset_fingerprint_match = entry
        .dataset_fingerprint
        .as_ref()
        .map(|value| value == &canonical.dataset_fingerprint)
        .unwrap_or(false);
    let feature_schema_match = entry
        .feature_schema_hash
        .as_ref()
        .map(|value| value == &canonical.feature_schema_hash)
        .unwrap_or(false);
    let label_manifest_match = entry
        .label_manifest_hash
        .as_ref()
        .map(|value| value == &canonical.label_manifest_hash)
        .unwrap_or(false);
    let split_policy_match = entry
        .split_policy
        .as_ref()
        .map(|value| value == &canonical.split_policy)
        .unwrap_or(false);
    let coverage_ratio = import.map(|import| {
        if import.total_prediction_rows == 0 {
            0.0
        } else {
            import.valid_prediction_rows as f64 / import.total_prediction_rows as f64
        }
    });
    let invalid_count = import.map(|import| import.invalid_prediction_rows);
    let duplicate_count = import.map(|import| import.duplicate_prediction_count);
    let lineage_status = if invalid_count.unwrap_or(0) > 0 {
        PredictionArtifactLineageStatus::InvalidPredictions
    } else if coverage_ratio.unwrap_or(0.0) < 0.8 {
        PredictionArtifactLineageStatus::CoverageWeak
    } else if !dataset_fingerprint_match && config.require_same_dataset_fingerprint_for_comparison {
        PredictionArtifactLineageStatus::DatasetMismatch
    } else if !feature_schema_match && config.require_same_feature_schema_hash_for_comparison {
        PredictionArtifactLineageStatus::FeatureSchemaMismatch
    } else if !label_manifest_match && config.require_same_label_manifest_hash_for_comparison {
        PredictionArtifactLineageStatus::LabelManifestMismatch
    } else if !split_policy_match {
        PredictionArtifactLineageStatus::SplitPolicyMismatch
    } else {
        PredictionArtifactLineageStatus::PredictionLineageValid
    };
    PredictionArtifactLineageReport {
        prediction_artifact_id: entry.entry_id,
        model_id: entry.model_id,
        model_version: entry.model_version,
        sequence_export_manifest_id: canonical.export_id.clone(),
        dataset_fingerprint_match,
        feature_schema_match,
        label_manifest_match,
        split_policy_match,
        coverage_ratio,
        duplicate_count,
        invalid_count,
        lineage_status,
        reason_codes: stable_reason_codes(&[ReasonCode::PredictionArtifactLineageReportBuilt]),
    }
}

fn build_external_evaluation_history_report(
    config: &ExternalModelArtifactRegistryConfig,
    evaluation_reports: &[EvaluationArtifact],
    baseline_reference_summary: String,
    trinity_reference_summary: String,
    no_trade_reference_summary: String,
    risk_denied_reference_summary: String,
) -> ExternalEvaluationHistoryReport {
    let grouped = group_evaluations_by_model(evaluation_reports);
    let mut model_histories = Vec::new();
    let mut latest_model_versions = BTreeMap::new();
    let mut previous_model_versions = BTreeMap::new();
    let mut metric_deltas = BTreeMap::new();
    for (model_id, reports) in grouped {
        let versions: Vec<String> = reports
            .iter()
            .map(|artifact| artifact.report.model_version.clone())
            .collect();
        let mut metrics_by_version = BTreeMap::new();
        for artifact in &reports {
            metrics_by_version.insert(
                artifact.report.model_version.clone(),
                collect_metric_map(&artifact.report),
            );
        }
        let best_version_by_metric = best_version_by_metric(&metrics_by_version);
        let latest = versions.last().cloned().unwrap_or_default();
        let previous = versions.iter().rev().nth(1).cloned();
        if let Some(previous_version) = &previous {
            latest_model_versions.insert(model_id.clone(), latest.clone());
            previous_model_versions.insert(model_id.clone(), previous_version.clone());
            metric_deltas.insert(
                model_id.clone(),
                metric_delta(
                    metrics_by_version.get(&latest).unwrap_or(&BTreeMap::new()),
                    metrics_by_version
                        .get(previous_version)
                        .unwrap_or(&BTreeMap::new()),
                ),
            );
        }
        model_histories.push(ModelEvaluationHistory {
            model_id: model_id.clone(),
            versions: versions.clone(),
            metrics_by_version,
            best_version_by_metric,
            latest_vs_previous_delta: metric_deltas.get(&model_id).cloned().unwrap_or_default(),
            reason_codes: stable_reason_codes(&[ReasonCode::ExternalEvaluationHistoryReportBuilt]),
        });
    }
    let history_status = if model_histories
        .iter()
        .all(|history| history.versions.len() >= 2)
    {
        ExternalEvaluationHistoryStatus::HistoryReady
    } else {
        ExternalEvaluationHistoryStatus::NeedMoreVersions
    };
    ExternalEvaluationHistoryReport {
        history_id: config.registry_id.clone(),
        model_histories,
        baseline_reference_summary,
        trinity_reference_summary,
        no_trade_reference_summary,
        risk_denied_reference_summary,
        latest_model_versions,
        previous_model_versions,
        metric_deltas,
        history_status,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalEvaluationHistoryReportBuilt]),
    }
}

fn build_calibration_drift_report(
    evaluation_reports: &[EvaluationArtifact],
) -> CalibrationDriftReport {
    let grouped = group_evaluations_by_model(evaluation_reports);
    let mut records = Vec::new();
    for (model_id, reports) in grouped {
        let latest = reports.last();
        let previous = reports.iter().rev().nth(1);
        let Some(latest) = latest else {
            continue;
        };
        let record = if let Some(previous) = previous {
            let brier_delta = latest
                .report
                .calibration_metrics
                .brier_score
                .zip(previous.report.calibration_metrics.brier_score)
                .map(|(current, previous)| current - previous);
            let ece_delta = latest
                .report
                .calibration_metrics
                .ece
                .zip(previous.report.calibration_metrics.ece)
                .map(|(current, previous)| current - previous);
            let confidence_shift = latest
                .report
                .return_proxy_metrics
                .hit_rate_proxy
                .zip(previous.report.return_proxy_metrics.hit_rate_proxy)
                .map(|(current, previous)| current - previous);
            let calibration_status =
                if brier_delta.unwrap_or(0.0) > 0.05 || ece_delta.unwrap_or(0.0) > 0.10 {
                    CalibrationDriftStatus::SevereDrift
                } else if brier_delta.unwrap_or(0.0) > 0.02 || ece_delta.unwrap_or(0.0) > 0.03 {
                    CalibrationDriftStatus::MildDrift
                } else {
                    CalibrationDriftStatus::Stable
                };
            CalibrationDriftRecord {
                model_id: model_id.clone(),
                model_version: latest.report.model_version.clone(),
                previous_model_version: Some(previous.report.model_version.clone()),
                brier_score_current: latest.report.calibration_metrics.brier_score,
                brier_score_previous: previous.report.calibration_metrics.brier_score,
                brier_delta,
                ece_current: latest.report.calibration_metrics.ece,
                ece_previous: previous.report.calibration_metrics.ece,
                ece_delta,
                confidence_shift,
                calibration_status,
                reason_codes: stable_reason_codes(&[ReasonCode::CalibrationDriftReportBuilt]),
            }
        } else {
            CalibrationDriftRecord {
                model_id: model_id.clone(),
                model_version: latest.report.model_version.clone(),
                previous_model_version: None,
                brier_score_current: latest.report.calibration_metrics.brier_score,
                brier_score_previous: None,
                brier_delta: None,
                ece_current: latest.report.calibration_metrics.ece,
                ece_previous: None,
                ece_delta: None,
                confidence_shift: None,
                calibration_status: CalibrationDriftStatus::InsufficientHistory,
                reason_codes: stable_reason_codes(&[ReasonCode::CalibrationDriftReportBuilt]),
            }
        };
        records.push(record);
    }
    let stable_count = records
        .iter()
        .filter(|record| matches!(record.calibration_status, CalibrationDriftStatus::Stable))
        .count();
    let mild_drift_count = records
        .iter()
        .filter(|record| matches!(record.calibration_status, CalibrationDriftStatus::MildDrift))
        .count();
    let severe_drift_count = records
        .iter()
        .filter(|record| {
            matches!(
                record.calibration_status,
                CalibrationDriftStatus::SevereDrift
            )
        })
        .count();
    let insufficient_history_count = records
        .iter()
        .filter(|record| {
            matches!(
                record.calibration_status,
                CalibrationDriftStatus::InsufficientHistory
            )
        })
        .count();
    let drift_status = if severe_drift_count > 0 {
        CalibrationDriftStatus::SevereDrift
    } else if mild_drift_count > 0 {
        CalibrationDriftStatus::MildDrift
    } else if stable_count > 0 {
        CalibrationDriftStatus::Stable
    } else {
        CalibrationDriftStatus::InsufficientHistory
    };
    CalibrationDriftReport {
        records,
        stable_count,
        mild_drift_count,
        severe_drift_count,
        insufficient_history_count,
        drift_status,
        reason_codes: stable_reason_codes(&[ReasonCode::CalibrationDriftReportBuilt]),
    }
}

fn build_external_model_version_comparison_report(
    config: &ExternalModelArtifactRegistryConfig,
    evaluation_reports: &[EvaluationArtifact],
    drift_report: &CalibrationDriftReport,
) -> ExternalModelVersionComparisonReport {
    let grouped = group_evaluations_by_model(evaluation_reports);
    let (model_id, reports) = grouped
        .into_iter()
        .max_by_key(|(_, reports)| reports.len())
        .unwrap_or_else(|| ("unknown".to_string(), Vec::new()));
    let latest = reports.last();
    let previous = reports.iter().rev().nth(1);
    let drift_record = drift_report
        .records
        .iter()
        .find(|record| record.model_id == model_id)
        .cloned()
        .into_iter()
        .collect::<Vec<_>>();
    let (
        latest_version,
        previous_version,
        metric_delta_summary,
        calibration_delta_summary,
        risk_delta_summary,
        comparison_status,
    ) = if let (Some(latest), Some(previous)) = (latest, previous) {
        let current = collect_metric_map(&latest.report);
        let previous_map = collect_metric_map(&previous.report);
        let metric_delta_summary = metric_delta(&current, &previous_map);
        let calibration_delta_summary = filter_metric_prefix(&metric_delta_summary, "calibration:");
        let risk_delta_summary = filter_metric_prefix(&metric_delta_summary, "risk:");
        let improved = metric_delta_summary
            .get("return:net_return_proxy")
            .copied()
            .unwrap_or(0.0)
            > 0.0
            && metric_delta_summary
                .get("risk:risk_adjusted_score")
                .copied()
                .unwrap_or(0.0)
                >= 0.0
            && metric_delta_summary
                .get("calibration:brier_score")
                .copied()
                .unwrap_or(0.0)
                <= 0.0
            && metric_delta_summary
                .get("calibration:ece")
                .copied()
                .unwrap_or(0.0)
                <= 0.0;
        let regressed = metric_delta_summary
            .get("calibration:brier_score")
            .copied()
            .unwrap_or(0.0)
            > 0.02
            || metric_delta_summary
                .get("calibration:ece")
                .copied()
                .unwrap_or(0.0)
                > 0.03
            || metric_delta_summary
                .get("risk:risk_adjusted_score")
                .copied()
                .unwrap_or(0.0)
                < 0.0;
        let comparison_status = if improved {
            ExternalModelVersionComparisonStatus::Improved
        } else if regressed {
            ExternalModelVersionComparisonStatus::Regressed
        } else {
            ExternalModelVersionComparisonStatus::Mixed
        };
        (
            latest.report.model_version.clone(),
            Some(previous.report.model_version.clone()),
            metric_delta_summary,
            calibration_delta_summary,
            risk_delta_summary,
            comparison_status,
        )
    } else {
        (
            "unknown".to_string(),
            None,
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            ExternalModelVersionComparisonStatus::NoComparablePrevious,
        )
    };
    ExternalModelVersionComparisonReport {
        comparison_id: config.registry_id.clone(),
        records: drift_record,
        model_id,
        latest_version,
        previous_version,
        metric_delta_summary,
        calibration_delta_summary,
        risk_delta_summary,
        comparison_status,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelVersionComparisonReportBuilt]),
    }
}

fn build_conservative_external_leaderboard(
    config: &ExternalModelArtifactRegistryConfig,
    canonical: &CanonicalContract,
    model_cards: &[(String, ExternalModelCardV2)],
    evaluation_reports: &[EvaluationArtifact],
    ablation_reports: &[AblationArtifact],
    promotion_reports: &[PromotionArtifact],
) -> ConservativeExternalLeaderboard {
    let card_map: BTreeMap<(String, String), &ExternalModelCardV2> = model_cards
        .iter()
        .map(|(_, card)| ((card.model_id.clone(), card.model_version.clone()), card))
        .collect();
    let ablation_map: BTreeMap<(String, String), &ExternalPredictionAblationReport> =
        ablation_reports
            .iter()
            .map(|artifact| {
                (
                    (
                        artifact.report.model_id.clone(),
                        artifact.report.model_version.clone(),
                    ),
                    &artifact.report,
                )
            })
            .collect();
    let promotion_map: BTreeMap<(String, String), &ExternalModelPromotionGateReport> =
        promotion_reports
            .iter()
            .map(|artifact| {
                (
                    (
                        artifact.report.model_id.clone(),
                        artifact.report.model_version.clone(),
                    ),
                    &artifact.report,
                )
            })
            .collect();
    let mut entries = Vec::new();
    for artifact in evaluation_reports {
        let key = (
            artifact.report.model_id.clone(),
            artifact.report.model_version.clone(),
        );
        let card = card_map.get(&key);
        let ablation = ablation_map.get(&key);
        let promotion = promotion_map.get(&key);
        let coverage_ratio =
            artifact.report.evaluated_count as f64 / artifact.report.sequence_count.max(1) as f64;
        let mut penalties = Vec::new();
        let ece = artifact.report.calibration_metrics.ece;
        let brier = artifact.report.calibration_metrics.brier_score;
        let risk_adjusted = artifact.report.risk_metrics.risk_adjusted_score;
        let net_return = artifact.report.cost_aware_metrics.net_return_proxy;
        let top_k_precision = artifact.report.ranking_metrics.top_k_precision;
        let rank_correlation = artifact.report.ranking_metrics.rank_correlation;
        let compatible = card
            .map(|card| {
                card.feature_schema_hash == canonical.feature_schema_hash
                    && card.label_manifest_hash == canonical.label_manifest_hash
            })
            .unwrap_or(false);
        let entry_status = if card.is_none() && config.require_model_card {
            penalties.push("missing model card".to_string());
            LeaderboardEntryStatus::BlockedByModelCard
        } else if !compatible {
            penalties.push("contract mismatch".to_string());
            LeaderboardEntryStatus::BlockedByContractMismatch
        } else if coverage_ratio < 0.8 {
            penalties.push("low coverage".to_string());
            LeaderboardEntryStatus::BlockedByCoverage
        } else if ece.unwrap_or(1.0) > 0.3 || brier.unwrap_or(1.0) > 0.15 {
            penalties.push("poor calibration".to_string());
            LeaderboardEntryStatus::BlockedByCalibration
        } else if risk_adjusted.unwrap_or(-1.0) < 0.0 {
            penalties.push("poor risk".to_string());
            LeaderboardEntryStatus::BlockedByRisk
        } else if ablation
            .map(|report| format!("{:?}", report.ablation_status))
            .unwrap_or_default()
            == "Unstable"
        {
            penalties.push("unstable ablation".to_string());
            LeaderboardEntryStatus::BlockedByAblationInstability
        } else if artifact.report.evaluated_count < 5 {
            penalties.push("small sample".to_string());
            LeaderboardEntryStatus::EligibleWithWarnings
        } else {
            LeaderboardEntryStatus::Eligible
        };
        let conservative_score = net_return.unwrap_or(0.0)
            + risk_adjusted.unwrap_or(0.0) * 0.01
            + top_k_precision.unwrap_or(0.0) * 0.1
            + rank_correlation.unwrap_or(0.0) * 0.02
            - ece.unwrap_or(0.0)
            - brier.unwrap_or(0.0)
            - penalties.len() as f64 * 0.1;
        entries.push(ConservativeLeaderboardEntry {
            rank: None,
            model_id: artifact.report.model_id.clone(),
            model_version: artifact.report.model_version.clone(),
            model_family: card
                .map(|card| format!("{:?}", card.model_family))
                .unwrap_or_else(|| "Unknown".to_string()),
            dataset_fingerprint: canonical.dataset_fingerprint.clone(),
            feature_schema_hash: card
                .map(|card| card.feature_schema_hash.clone())
                .unwrap_or_default(),
            label_manifest_hash: card
                .map(|card| card.label_manifest_hash.clone())
                .unwrap_or_default(),
            coverage_ratio,
            brier_score: brier,
            ece,
            risk_adjusted_score: risk_adjusted,
            net_return_proxy: net_return,
            top_k_precision,
            rank_correlation,
            ablation_status: ablation.map(|report| format!("{:?}", report.ablation_status)),
            promotion_status: promotion.map(|report| format!("{:?}", report.gate_status)),
            conservative_score,
            penalty_summary: penalties,
            entry_status,
            reason_codes: stable_reason_codes(&[ReasonCode::ConservativeExternalLeaderboardBuilt]),
        });
    }
    entries.sort_by(|left, right| {
        right
            .conservative_score
            .partial_cmp(&left.conservative_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut rank = 1usize;
    for entry in entries.iter_mut().filter(|entry| {
        matches!(
            entry.entry_status,
            LeaderboardEntryStatus::Eligible | LeaderboardEntryStatus::EligibleWithWarnings
        )
    }) {
        entry.rank = Some(rank);
        rank += 1;
    }
    let eligible_entries = entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.entry_status,
                LeaderboardEntryStatus::Eligible | LeaderboardEntryStatus::EligibleWithWarnings
            )
        })
        .count();
    let diagnostic_entries = entries
        .iter()
        .filter(|entry| matches!(entry.entry_status, LeaderboardEntryStatus::DiagnosticOnly))
        .count();
    let blocked_entries = entries
        .len()
        .saturating_sub(eligible_entries + diagnostic_entries);
    let leaderboard_status = if eligible_entries >= 2 {
        ConservativeExternalLeaderboardStatus::LeaderboardReady
    } else if eligible_entries == 1 {
        ConservativeExternalLeaderboardStatus::LeaderboardReadyWithWarnings
    } else {
        ConservativeExternalLeaderboardStatus::NeedMoreModels
    };
    ConservativeExternalLeaderboard {
        leaderboard_id: config.registry_id.clone(),
        entries,
        eligible_entries,
        diagnostic_entries,
        blocked_entries,
        baseline_entry: Some(reference_summary(&config.baseline_reference_paths)),
        trinity_entry: Some(reference_summary(&config.trinity_reference_paths)),
        no_trade_entry: Some(reference_summary(&config.no_trade_reference_paths)),
        risk_denied_entry: Some(reference_summary(&config.risk_denied_reference_paths)),
        leaderboard_status,
        reason_codes: stable_reason_codes(&[ReasonCode::ConservativeExternalLeaderboardBuilt]),
    }
}

fn build_leaderboard_promotion_policy_report(
    policy: &LeaderboardPromotionPolicyConfig,
    leaderboard: &ConservativeExternalLeaderboard,
    drift_report: &CalibrationDriftReport,
) -> LeaderboardPromotionPolicyReport {
    let severe_models: BTreeSet<String> = drift_report
        .records
        .iter()
        .filter(|record| {
            matches!(
                record.calibration_status,
                CalibrationDriftStatus::SevereDrift
            )
        })
        .map(|record| format!("{}:{}", record.model_id, record.model_version))
        .collect();
    let mut model_results = Vec::new();
    let mut research_candidate_count = 0usize;
    let mut diagnostic_only_count = 0usize;
    let mut blocked_count = 0usize;
    for entry in &leaderboard.entries {
        let key = format!("{}:{}", entry.model_id, entry.model_version);
        let allowed = matches!(
            entry.entry_status,
            LeaderboardEntryStatus::Eligible | LeaderboardEntryStatus::EligibleWithWarnings
        ) && entry.coverage_ratio >= policy.min_coverage_ratio
            && entry.ece.unwrap_or(1.0) <= policy.max_ece
            && entry.brier_score.unwrap_or(1.0) <= policy.max_brier_score
            && entry.risk_adjusted_score.unwrap_or(-1.0) >= policy.min_risk_adjusted_score
            && !severe_models.contains(&key)
            && entry.ablation_status.as_deref() != Some("Unstable");
        let result = if allowed {
            research_candidate_count += 1;
            format!("{key}:ResearchCandidate")
        } else if matches!(entry.entry_status, LeaderboardEntryStatus::DiagnosticOnly) {
            diagnostic_only_count += 1;
            format!("{key}:DiagnosticOnly")
        } else {
            blocked_count += 1;
            format!("{key}:Blocked")
        };
        model_results.push(result);
    }
    let policy_status = if research_candidate_count > 0 {
        LeaderboardPromotionPolicyStatus::PolicyReady
    } else if blocked_count > 0 && diagnostic_only_count == 0 {
        LeaderboardPromotionPolicyStatus::TooStrictForCurrentEvidence
    } else if leaderboard.entries.is_empty() {
        LeaderboardPromotionPolicyStatus::NeedMoreEvidence
    } else {
        LeaderboardPromotionPolicyStatus::DiagnosticOnly
    };
    LeaderboardPromotionPolicyReport {
        policy_id: policy.policy_id.clone(),
        model_results,
        research_candidate_count,
        diagnostic_only_count,
        blocked_count,
        policy_status,
        reason_codes: stable_reason_codes(&[ReasonCode::LeaderboardPromotionPolicyBuilt]),
    }
}

fn build_previous_external_comparison_report(
    version_comparison: &ExternalModelVersionComparisonReport,
    drift_report: &CalibrationDriftReport,
) -> PreviousExternalComparisonReport {
    let severe = drift_report.records.iter().any(|record| {
        record.model_id == version_comparison.model_id
            && matches!(
                record.calibration_status,
                CalibrationDriftStatus::SevereDrift
            )
    });
    let risk_worse = version_comparison
        .risk_delta_summary
        .get("risk:risk_adjusted_score")
        .copied()
        .unwrap_or(0.0)
        < 0.0;
    let recommendation = if severe {
        PreviousExternalComparisonRecommendation::BlockedByDrift
    } else if risk_worse {
        PreviousExternalComparisonRecommendation::BlockedByRisk
    } else {
        match version_comparison.comparison_status {
            ExternalModelVersionComparisonStatus::Improved => {
                PreviousExternalComparisonRecommendation::KeepLatestAsResearchCandidate
            }
            ExternalModelVersionComparisonStatus::Regressed => {
                PreviousExternalComparisonRecommendation::KeepPreviousAsResearchCandidate
            }
            ExternalModelVersionComparisonStatus::Mixed => {
                PreviousExternalComparisonRecommendation::DowngradeToDiagnostic
            }
            _ => PreviousExternalComparisonRecommendation::NeedsMoreHistory,
        }
    };
    PreviousExternalComparisonReport {
        model_id: version_comparison.model_id.clone(),
        latest_version: version_comparison.latest_version.clone(),
        previous_versions: version_comparison
            .previous_version
            .clone()
            .into_iter()
            .collect(),
        comparable_versions: version_comparison
            .previous_version
            .clone()
            .into_iter()
            .collect(),
        non_comparable_versions: Vec::new(),
        metric_deltas: version_comparison.metric_delta_summary.clone(),
        drift_summary: format!("{:?}", drift_report.drift_status),
        recommendation,
        reason_codes: stable_reason_codes(&[ReasonCode::PreviousExternalComparisonReportBuilt]),
    }
}

fn build_registry_audit_report(
    config: &ExternalModelArtifactRegistryConfig,
    registry: &ExternalModelArtifactRegistry,
    model_cards: &[(String, ExternalModelCardV2)],
) -> Result<ExternalArtifactRegistryAuditReport, String> {
    let mut missing_artifacts = 0usize;
    let mut incompatible_artifacts = registry.incompatible_entries;
    let mut secret_like_fields = 0usize;
    let mut order_account_fields = 0usize;
    let mut unsafe_intended_use_count = 0usize;
    for entry in &registry.entries {
        let path = Path::new(&entry.artifact_path);
        if !path.exists()
            && !entry.artifact_path.starts_with("examples/")
            && !entry.artifact_path.starts_with("target/")
        {
            missing_artifacts += 1;
        }
        if entry.artifact_path.contains("://") {
            incompatible_artifacts += 1;
        }
        if path.exists() {
            let text = fs::read_to_string(path)
                .unwrap_or_default()
                .to_ascii_lowercase();
            if text.contains("app_secret") || text.contains("api_key") || text.contains("secret") {
                secret_like_fields += 1;
            }
            if text.contains("order_id")
                || text.contains("account_id")
                || text.contains("position_id")
            {
                order_account_fields += 1;
            }
        }
    }
    for (_, card) in model_cards {
        if card.intended_use.to_ascii_lowercase().contains("live")
            || !card.live_use_forbidden
            || !card.risk_integration_required
        {
            unsafe_intended_use_count += 1;
        }
    }
    let audit_status = if missing_artifacts > 0 {
        ExternalArtifactRegistryAuditStatus::FailedMissingArtifacts
    } else if incompatible_artifacts > 0 {
        ExternalArtifactRegistryAuditStatus::FailedContractMismatch
    } else if secret_like_fields > 0 {
        ExternalArtifactRegistryAuditStatus::FailedSecretScan
    } else if order_account_fields > 0 || unsafe_intended_use_count > 0 {
        ExternalArtifactRegistryAuditStatus::FailedUnsafeFields
    } else {
        ExternalArtifactRegistryAuditStatus::Passed
    };
    Ok(ExternalArtifactRegistryAuditReport {
        registry_id: config.registry_id.clone(),
        artifacts_scanned: registry.entries.len(),
        missing_artifacts,
        incompatible_artifacts,
        secret_like_fields,
        order_account_fields,
        unsafe_intended_use_count,
        audit_status,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalArtifactRegistryAuditReportBuilt]),
    })
}

fn build_external_leaderboard_panel(
    registry: &ExternalModelArtifactRegistry,
    leaderboard: &ConservativeExternalLeaderboard,
    drift_report: &CalibrationDriftReport,
    version_comparison: &ExternalModelVersionComparisonReport,
    policy_report: &LeaderboardPromotionPolicyReport,
    previous_comparison: &PreviousExternalComparisonReport,
) -> ExternalLeaderboardPanel {
    let mut panel = ExternalLeaderboardPanel {
        registry_status: format!("{:?}", registry.registry_status),
        leaderboard_status: format!("{:?}", leaderboard.leaderboard_status),
        top_entries: leaderboard
            .entries
            .iter()
            .filter_map(|entry| entry.rank.map(|rank| format!("{rank}:{}:{}", entry.model_id, entry.model_version)))
            .collect(),
        blocked_entries: leaderboard
            .entries
            .iter()
            .filter(|entry| entry.rank.is_none())
            .map(|entry| format!("{}:{}:{:?}", entry.model_id, entry.model_version, entry.entry_status))
            .collect(),
        calibration_drift_status: format!("{:?}", drift_report.drift_status),
        latest_vs_previous_summary: format!(
            "{}:{}->{:?}",
            version_comparison.model_id,
            version_comparison.latest_version,
            version_comparison.previous_version
        ),
        mamba3fin_family_status: "PredictionCsvOnly + ContractOnly + RuntimeDeferred".to_string(),
        baseline_summary: leaderboard.baseline_entry.clone().unwrap_or_default(),
        trinity_summary: leaderboard.trinity_entry.clone().unwrap_or_default(),
        no_trade_summary: leaderboard.no_trade_entry.clone().unwrap_or_default(),
        risk_denied_summary: leaderboard.risk_denied_entry.clone().unwrap_or_default(),
        next_actions: vec![
            "cargo run --quiet --bin soma_experiment -- external-artifact-registry --config examples/soma_external_artifact_registry.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- external-evaluation-history --config examples/soma_external_evaluation_history.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- calibration-drift --config examples/soma_calibration_drift.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- external-model-version-comparison --config examples/soma_external_model_version_comparison.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- conservative-external-leaderboard --config examples/soma_conservative_external_leaderboard.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- external-registry-audit --config examples/soma_external_registry_audit.toml".to_string(),
            format!("policy={:?}", policy_report.policy_status),
            format!("previous={:?}", previous_comparison.recommendation),
        ],
        reason_codes: stable_reason_codes(&[ReasonCode::ControlTowerExternalLeaderboardPanelBuilt]),
    };
    panel.stabilize();
    panel
}

fn build_storage_report(
    config: &ExternalModelArtifactRegistryConfig,
    entries: &[ExternalModelRegistryEntry],
) -> Result<RegistryStorageReport, String> {
    let mut storage_bytes = 0usize;
    for entry in entries {
        let path = Path::new(&entry.artifact_path);
        if path.exists() {
            storage_bytes += fs::metadata(path).map_err(|err| err.to_string())?.len() as usize;
        }
    }
    Ok(RegistryStorageReport {
        artifact_count: entries.len(),
        storage_bytes,
        max_bytes: config.max_bytes,
        within_budget: storage_bytes <= config.max_bytes,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalArtifactRegistryStorageBuilt]),
    })
}

fn build_final_summary(
    registry: &ExternalModelArtifactRegistry,
    history: &ExternalEvaluationHistoryReport,
    drift: &CalibrationDriftReport,
    leaderboard: &ConservativeExternalLeaderboard,
) -> String {
    [
        format!("registry_status={:?}", registry.registry_status),
        format!("history_status={:?}", history.history_status),
        format!("calibration_drift_status={:?}", drift.drift_status),
        format!("leaderboard_status={:?}", leaderboard.leaderboard_status),
        "runtime_status=HoldMamba3RuntimeDeferred".to_string(),
    ]
    .join("\n")
}

fn write_bundle(
    config: &ExternalModelArtifactRegistryConfig,
    bundle: &ExternalArtifactRegistryBundle,
) -> Result<(), String> {
    let dir = config.output_dir();
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    write_text_report(
        &dir,
        "external_model_artifact_registry.txt",
        &bundle.artifact_registry,
    )?;
    write_text_report(
        &dir,
        "model_card_lineage.txt",
        &bundle.model_card_lineage_report,
    )?;
    write_text_report(
        &dir,
        "prediction_artifact_lineage.txt",
        &bundle.prediction_artifact_lineage_report,
    )?;
    write_text_report(
        &dir,
        "external_evaluation_history.txt",
        &bundle.evaluation_history_report,
    )?;
    write_text_report(
        &dir,
        "calibration_drift.txt",
        &bundle.calibration_drift_report,
    )?;
    write_text_report(
        &dir,
        "external_model_version_comparison.txt",
        &bundle.model_version_comparison_report,
    )?;
    write_text_report(
        &dir,
        "conservative_external_leaderboard.txt",
        &bundle.conservative_leaderboard,
    )?;
    write_text_report(
        &dir,
        "leaderboard_promotion_policy.txt",
        &bundle.leaderboard_promotion_policy_report,
    )?;
    write_text_report(
        &dir,
        "previous_external_comparison.txt",
        &bundle.previous_external_comparison_report,
    )?;
    write_text_report(
        &dir,
        "external_artifact_registry_audit.txt",
        &bundle.artifact_registry_audit_report,
    )?;
    if let Some(panel) = &bundle.control_tower_external_leaderboard_panel_summary {
        fs::write(
            dir.join("control_tower_external_leaderboard_panel.txt"),
            panel.to_text(),
        )
        .map_err(|err| err.to_string())?;
    }
    write_text_report(&dir, "storage_report.txt", &bundle.storage_report)?;
    fs::write(dir.join("summary.txt"), &bundle.final_summary).map_err(|err| err.to_string())?;
    Ok(())
}

fn group_evaluations_by_model(
    evaluation_reports: &[EvaluationArtifact],
) -> BTreeMap<String, Vec<EvaluationArtifact>> {
    let mut out: BTreeMap<String, Vec<EvaluationArtifact>> = BTreeMap::new();
    for artifact in evaluation_reports {
        out.entry(artifact.report.model_id.clone())
            .or_default()
            .push(artifact.clone());
    }
    for reports in out.values_mut() {
        reports.sort_by(|left, right| left.report.model_version.cmp(&right.report.model_version));
    }
    out
}

fn collect_metric_map(report: &ExternalModelEvaluationReport) -> BTreeMap<String, f64> {
    let mut metrics = BTreeMap::new();
    if let Some(value) = report.calibration_metrics.brier_score {
        metrics.insert("calibration:brier_score".to_string(), value);
    }
    if let Some(value) = report.calibration_metrics.ece {
        metrics.insert("calibration:ece".to_string(), value);
    }
    if let Some(value) = report.risk_metrics.risk_adjusted_score {
        metrics.insert("risk:risk_adjusted_score".to_string(), value);
    }
    if let Some(value) = report.cost_aware_metrics.net_return_proxy {
        metrics.insert("return:net_return_proxy".to_string(), value);
    }
    if let Some(value) = report.ranking_metrics.top_k_precision {
        metrics.insert("ranking:top_k_precision".to_string(), value);
    }
    metrics
}

fn best_version_by_metric(
    metrics_by_version: &BTreeMap<String, BTreeMap<String, f64>>,
) -> BTreeMap<String, String> {
    let mut best = BTreeMap::new();
    for (version, metrics) in metrics_by_version {
        for (metric, value) in metrics {
            let is_better = if metric.contains("brier") || metric.contains("ece") {
                best.get(metric)
                    .and_then(|best_version| metrics_by_version.get(best_version))
                    .and_then(|metrics| metrics.get(metric))
                    .map(|best_value| value < best_value)
                    .unwrap_or(true)
            } else {
                best.get(metric)
                    .and_then(|best_version| metrics_by_version.get(best_version))
                    .and_then(|metrics| metrics.get(metric))
                    .map(|best_value| value > best_value)
                    .unwrap_or(true)
            };
            if is_better {
                best.insert(metric.clone(), version.clone());
            }
        }
    }
    best
}

fn metric_delta(
    current: &BTreeMap<String, f64>,
    previous: &BTreeMap<String, f64>,
) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::new();
    for key in current.keys().chain(previous.keys()) {
        let current_value = current.get(key).copied().unwrap_or(0.0);
        let previous_value = previous.get(key).copied().unwrap_or(0.0);
        out.insert(key.clone(), current_value - previous_value);
    }
    out
}

fn filter_metric_prefix(values: &BTreeMap<String, f64>, prefix: &str) -> BTreeMap<String, f64> {
    values
        .iter()
        .filter(|(key, _)| key.starts_with(prefix))
        .map(|(key, value)| (key.clone(), *value))
        .collect()
}

fn reference_summary(paths: &[String]) -> String {
    if paths.is_empty() {
        "none".to_string()
    } else {
        format!("{} reference artifact(s)", paths.len())
    }
}

fn infer_model_identity_from_path(path: &str) -> (String, String) {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown");
    let parts: Vec<&str> = stem.split('_').collect();
    if parts.len() >= 3 {
        (
            parts[0..parts.len() - 1].join("_"),
            parts[parts.len() - 1].to_string(),
        )
    } else {
        (stem.to_string(), "unknown".to_string())
    }
}

fn infer_model_identity_from_import_path(path: &str) -> (String, String) {
    let path_buf = PathBuf::from(path);
    let stem = path_buf
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown");
    let csv_candidate = if let Some(suffix) = stem.strip_prefix("import_") {
        path_buf
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(format!("external_predictions_model_{suffix}.csv"))
    } else {
        path_buf.with_extension("csv")
    };
    if let Some(identity) = read_prediction_csv_identity(&csv_candidate) {
        return identity;
    }
    ("external-import".to_string(), "v2".to_string())
}

fn read_prediction_csv_identity(path: &Path) -> Option<(String, String)> {
    let text = fs::read_to_string(path).ok()?;
    let mut lines = text.lines();
    let header = lines.next()?;
    let first = lines.next()?;
    let header_cols: Vec<&str> = header.split(',').collect();
    let first_cols: Vec<&str> = first.split(',').collect();
    Some((
        csv_lookup(&header_cols, &first_cols, "model_id")?.to_string(),
        csv_lookup(&header_cols, &first_cols, "model_version")?.to_string(),
    ))
}

fn csv_lookup<'a>(header: &[&str], values: &[&'a str], key: &str) -> Option<&'a str> {
    header
        .iter()
        .position(|column| *column == key)
        .and_then(|idx| values.get(idx).copied())
}

fn write_text_report<T: Serialize>(dir: &Path, name: &str, value: &T) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    fs::write(dir.join(name), text).map_err(|err| err.to_string())
}

fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}
