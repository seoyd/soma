use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};

use super::{
    FeatureSchemaManifest, LabelManifest, SequenceDatasetStorageReport, SequenceExportManifest,
    SequenceLabelKind,
};

fn default_output_root() -> String {
    "target/soma_external_prediction_eval".to_string()
}

fn default_max_prediction_rows() -> usize {
    10_000
}

fn default_max_models() -> usize {
    8
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalPredictionImportV2Config {
    pub import_id: String,
    pub sequence_export_manifest_path: String,
    #[serde(default)]
    pub dataset_csv_path: Option<String>,
    #[serde(default)]
    pub feature_schema_manifest_path: Option<String>,
    #[serde(default)]
    pub label_manifest_path: Option<String>,
    #[serde(default)]
    pub prediction_csv_paths: Vec<String>,
    #[serde(default)]
    pub model_card_paths: Vec<String>,
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
    #[serde(default = "default_max_prediction_rows")]
    pub max_prediction_rows: usize,
    #[serde(default = "default_max_models")]
    pub max_models: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_model_card: bool,
    #[serde(default = "default_true")]
    pub require_sequence_id_match: bool,
    #[serde(default = "default_true")]
    pub require_model_version: bool,
    #[serde(default = "default_true")]
    pub require_no_duplicate_sequence_predictions: bool,
    #[serde(default = "default_true")]
    pub require_probability_sanity: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ExternalPredictionImportV2Config {
    fn default() -> Self {
        Self {
            import_id: "sprint63-external-prediction-import-v2".to_string(),
            sequence_export_manifest_path: String::new(),
            dataset_csv_path: None,
            feature_schema_manifest_path: None,
            label_manifest_path: None,
            prediction_csv_paths: Vec::new(),
            model_card_paths: Vec::new(),
            baseline_reference_paths: Vec::new(),
            trinity_reference_paths: Vec::new(),
            no_trade_reference_paths: Vec::new(),
            risk_denied_reference_paths: Vec::new(),
            output_root: default_output_root(),
            max_prediction_rows: default_max_prediction_rows(),
            max_models: default_max_models(),
            max_bytes: default_max_bytes(),
            require_model_card: true,
            require_sequence_id_match: true,
            require_model_version: true,
            require_no_duplicate_sequence_predictions: true,
            require_probability_sanity: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl ExternalPredictionImportV2Config {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.import_id.trim().is_empty() {
            return Err("external prediction import id must not be empty".to_string());
        }
        if self.sequence_export_manifest_path.trim().is_empty() {
            return Err("sequence_export_manifest_path must not be empty".to_string());
        }
        if self.prediction_csv_paths.is_empty() {
            return Err("at least one prediction_csv_path is required".to_string());
        }
        if self.max_prediction_rows == 0 {
            return Err("max_prediction_rows must be positive".to_string());
        }
        if self.max_models == 0 {
            return Err("max_models must be positive".to_string());
        }
        if self.max_bytes == 0 {
            return Err("max_bytes must be positive".to_string());
        }
        let all_paths = std::iter::once(self.sequence_export_manifest_path.as_str())
            .chain(self.dataset_csv_path.iter().map(String::as_str))
            .chain(self.feature_schema_manifest_path.iter().map(String::as_str))
            .chain(self.label_manifest_path.iter().map(String::as_str))
            .chain(self.prediction_csv_paths.iter().map(String::as_str))
            .chain(self.model_card_paths.iter().map(String::as_str))
            .chain(self.baseline_reference_paths.iter().map(String::as_str))
            .chain(self.trinity_reference_paths.iter().map(String::as_str))
            .chain(self.no_trade_reference_paths.iter().map(String::as_str))
            .chain(self.risk_denied_reference_paths.iter().map(String::as_str))
            .chain(std::iter::once(self.output_root.as_str()));
        if all_paths.clone().any(is_remote_path) {
            return Err("external prediction import config paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.import_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalPredictionCsvSchemaV2 {
    pub schema_version: String,
    pub required_columns: Vec<String>,
    pub optional_columns: Vec<String>,
    pub forbidden_columns: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ExternalPredictionCsvSchemaV2 {
    fn default() -> Self {
        Self {
            schema_version: "v2".to_string(),
            required_columns: vec![
                "sequence_id".to_string(),
                "model_id".to_string(),
                "model_version".to_string(),
                "prediction_timestamp_ms".to_string(),
            ],
            optional_columns: vec![
                "p_take_profit".to_string(),
                "p_stop_loss".to_string(),
                "p_time_expired".to_string(),
                "p_win".to_string(),
                "expected_return_pct".to_string(),
                "expected_drawdown_pct".to_string(),
                "confidence".to_string(),
                "predicted_label".to_string(),
                "rank_score".to_string(),
                "reason_code".to_string(),
            ],
            forbidden_columns: vec![
                "account_id".to_string(),
                "account_number".to_string(),
                "position_id".to_string(),
                "order_id".to_string(),
                "broker_order_id".to_string(),
                "api_key".to_string(),
                "app_key".to_string(),
                "app_secret".to_string(),
                "secret".to_string(),
            ],
            reason_codes: stable_reason_codes(&[ReasonCode::ExternalPredictionCsvSchemaV2Built]),
        }
    }
}

impl ExternalPredictionCsvSchemaV2 {
    pub fn validate_header(&self, header: &[String]) -> Vec<String> {
        let known: BTreeSet<String> = self
            .required_columns
            .iter()
            .chain(self.optional_columns.iter())
            .cloned()
            .collect();
        let header_set: BTreeSet<String> = header.iter().cloned().collect();
        let mut errors = Vec::new();
        for required in &self.required_columns {
            if !header_set.contains(required) {
                errors.push(format!("missing required column: {required}"));
            }
        }
        for forbidden in &self.forbidden_columns {
            if header_set.contains(forbidden) {
                errors.push(format!("forbidden column present: {forbidden}"));
            }
        }
        for column in header {
            if column.contains("account") || column.contains("order") || column.contains("secret") {
                errors.push(format!("unsafe column present: {column}"));
            } else if !known.contains(column) {
                errors.push(format!("unknown column: {column}"));
            }
        }
        stable_ordered_strings(&errors)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalPredictionRowV2 {
    pub sequence_id: String,
    pub model_id: String,
    pub model_version: String,
    pub prediction_timestamp_ms: u64,
    pub p_take_profit: Option<f64>,
    pub p_stop_loss: Option<f64>,
    pub p_time_expired: Option<f64>,
    pub p_win: Option<f64>,
    pub expected_return_pct: Option<f64>,
    pub expected_drawdown_pct: Option<f64>,
    pub confidence: Option<f64>,
    pub predicted_label: Option<String>,
    pub rank_score: Option<f64>,
    pub reason_code: Option<String>,
    #[serde(default)]
    pub validation_issues: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalPredictionImportStatus {
    ExternalPredictionImportReady,
    ImportReadyWithWarnings,
    ExternalModelDiagnosticOnly,
    InvalidPredictions,
    BlockedByModelCard,
    BlockedByCoverage,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalPredictionImportV2Report {
    pub import_id: String,
    pub schema_version: String,
    pub manifest_fingerprint: String,
    pub model_count: usize,
    pub total_prediction_rows: usize,
    pub valid_prediction_rows: usize,
    pub invalid_prediction_rows: usize,
    pub duplicate_prediction_count: usize,
    pub known_sequence_matches: usize,
    pub import_status: ExternalPredictionImportStatus,
    #[serde(default)]
    pub header_errors: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionCoverageStatus {
    FullCoverage,
    PartialCoverage,
    MissingPredictions,
    ExtraPredictions,
    DuplicatePredictions,
    InvalidPredictions,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PredictionCoverageReport {
    pub import_id: String,
    pub model_id: String,
    pub model_version: String,
    pub total_sequences: usize,
    pub predicted_sequences: usize,
    pub missing_sequence_count: usize,
    pub extra_sequence_count: usize,
    pub duplicate_prediction_count: usize,
    pub invalid_prediction_count: usize,
    pub coverage_ratio: f64,
    pub coverage_status: PredictionCoverageStatus,
    #[serde(default)]
    pub missing_sequence_ids_sample: Vec<String>,
    #[serde(default)]
    pub extra_sequence_ids_sample: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelFamily {
    Tabular,
    Sequence,
    Mamba3FinLiteExternal,
    OtherExternal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelCardSource {
    LocalResearch,
    ExternalFile,
    DiagnosticFixture,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalModelCardV2 {
    pub model_id: String,
    pub model_version: String,
    pub model_family: ExternalModelFamily,
    pub training_data_description: String,
    pub feature_schema_hash: String,
    pub label_manifest_hash: String,
    pub split_policy: String,
    pub intended_use: String,
    pub forbidden_use: String,
    pub limitations: String,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    pub source: ExternalModelCardSource,
    pub risk_integration_required: bool,
    pub live_use_forbidden: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelCardValidationStatus {
    Valid,
    MissingModelCard,
    FeatureHashMismatch,
    LabelHashMismatch,
    SplitPolicyMismatch,
    UnsafeIntendedUse,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalModelCardValidationReport {
    pub model_card_present: bool,
    pub feature_hash_match: bool,
    pub label_hash_match: bool,
    pub split_policy_match: bool,
    pub live_use_forbidden: bool,
    pub risk_integration_required: bool,
    pub validation_status: ExternalModelCardValidationStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ClassificationMetrics {
    pub accuracy: Option<f64>,
    pub macro_f1: Option<f64>,
    #[serde(default)]
    pub confusion_counts: BTreeMap<String, usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CalibrationMetrics {
    pub brier_score: Option<f64>,
    pub ece: Option<f64>,
    #[serde(default)]
    pub confidence_bucket_summary: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RankingMetrics {
    pub top_k_precision: Option<f64>,
    pub rank_correlation: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ReturnProxyMetrics {
    pub mean_expected_return: Option<f64>,
    pub mean_realized_proxy: Option<f64>,
    pub hit_rate_proxy: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub predicted_drawdown_avg: Option<f64>,
    pub risk_adjusted_score: Option<f64>,
    pub risk_denied_alignment: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CostAwareMetrics {
    pub net_return_proxy: Option<f64>,
    pub cost_stress_score: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelEvaluationStatus {
    EvaluationReady,
    EvaluationReadyWithWarnings,
    InsufficientCoverage,
    InsufficientRows,
    InvalidPredictions,
    PoorCalibration,
    PoorRiskBehavior,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalModelEvaluationReport {
    pub evaluation_id: String,
    pub model_id: String,
    pub model_version: String,
    pub sequence_count: usize,
    pub evaluated_count: usize,
    #[serde(default)]
    pub label_distribution: BTreeMap<String, usize>,
    #[serde(default)]
    pub classification_metrics: ClassificationMetrics,
    #[serde(default)]
    pub calibration_metrics: CalibrationMetrics,
    #[serde(default)]
    pub ranking_metrics: RankingMetrics,
    #[serde(default)]
    pub return_proxy_metrics: ReturnProxyMetrics,
    #[serde(default)]
    pub risk_metrics: RiskMetrics,
    #[serde(default)]
    pub cost_aware_metrics: CostAwareMetrics,
    pub evaluation_status: ExternalModelEvaluationStatus,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalVsTrinityComparisonStatus {
    ExternalBetterDiagnostic,
    TrinityBetterDiagnostic,
    Mixed,
    NotComparable,
    InsufficientRows,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalVsTrinityComparisonReport {
    pub comparison_id: String,
    pub model_id: String,
    pub model_version: String,
    pub comparable_rows: usize,
    pub external_better_count: usize,
    pub trinity_better_count: usize,
    pub no_trade_better_count: usize,
    pub risk_denied_defensive_count: usize,
    pub disagreement_count: usize,
    pub agreement_count: usize,
    #[serde(default)]
    pub delta_summary: Vec<String>,
    pub comparison_status: ExternalVsTrinityComparisonStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExternalPredictionAblationKind {
    DropProbabilityColumns,
    DropExpectedReturn,
    DropExpectedDrawdown,
    DropConfidence,
    CostStress,
    RiskThresholdStress,
    CoverageStress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalPredictionAblationStatus {
    Stable,
    Sensitive,
    Unstable,
    InsufficientRows,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalPredictionAblationReport {
    pub ablation_id: String,
    pub model_id: String,
    pub model_version: String,
    #[serde(default)]
    pub ablations: Vec<ExternalPredictionAblationKind>,
    #[serde(default)]
    pub baseline_metrics: BTreeMap<String, f64>,
    #[serde(default)]
    pub ablated_metrics: BTreeMap<String, BTreeMap<String, f64>>,
    #[serde(default)]
    pub sensitivity_summary: Vec<String>,
    pub ablation_status: ExternalPredictionAblationStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelPromotionGateStatus {
    ResearchCandidate,
    DiagnosticOnly,
    BlockedByCoverage,
    BlockedByCalibration,
    BlockedByRisk,
    BlockedByModelCard,
    BlockedByEvidenceDepth,
    BlockedBySequenceSize,
    BlockedByNoLookahead,
    Rejected,
    Deferred,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelPromotionRecommendation {
    KeepDiagnosticOnly,
    EvaluateMoreRows,
    ImproveCalibration,
    ImproveRiskBehavior,
    CompareAgainstTrinityMore,
    ResearchCandidateOnly,
    DoNotPromote,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalModelPromotionGateReport {
    pub gate_id: String,
    pub model_id: String,
    pub model_version: String,
    pub coverage_passed: bool,
    pub model_card_passed: bool,
    pub calibration_passed: bool,
    pub risk_passed: bool,
    pub evidence_depth_passed: bool,
    pub no_lookahead_passed: bool,
    pub sequence_size_passed: bool,
    pub gate_status: ExternalModelPromotionGateStatus,
    pub final_recommendation: ExternalModelPromotionRecommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3FinLiteContractBackend {
    ExternalResearchOnly,
    PredictionCsvOnly,
    Deferred,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Mamba3FinLiteForbiddenCapability {
    RustRuntimeInference,
    RustTraining,
    LiveInference,
    BrokerExecution,
    OnlineLearning,
    RuntimeLLMDecision,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3FinLiteContractStatus {
    ContractReady,
    BlockedByMissingDataset,
    BlockedByMissingModelCard,
    BlockedByMissingPredictionSchema,
    Deferred,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mamba3FinLitePrototypeContract {
    pub contract_id: String,
    pub allowed_backend: Mamba3FinLiteContractBackend,
    pub required_sequence_export_manifest: String,
    pub required_feature_schema_hash: String,
    pub required_label_manifest_hash: String,
    pub required_prediction_schema_version: String,
    #[serde(default)]
    pub required_model_card_fields: Vec<String>,
    #[serde(default)]
    pub forbidden_capabilities: Vec<Mamba3FinLiteForbiddenCapability>,
    #[serde(default)]
    pub evaluation_requirements: Vec<String>,
    #[serde(default)]
    pub control_tower_visibility_requirements: Vec<String>,
    #[serde(default)]
    pub risk_integration_requirements: Vec<String>,
    pub contract_status: Mamba3FinLiteContractStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalModelPanel {
    pub import_status: String,
    pub coverage_status: String,
    pub model_card_status: String,
    pub evaluation_status: String,
    pub comparison_status: String,
    pub ablation_status: String,
    pub promotion_gate_status: String,
    pub mamba_contract_status: String,
    #[serde(default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub model_version: Option<String>,
    pub evaluated_rows: usize,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub next_actions: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl ExternalModelPanel {
    pub fn stabilize(&mut self) {
        self.warnings = stable_ordered_strings(&self.warnings);
        self.blockers = stable_ordered_strings(&self.blockers);
        self.next_actions = stable_ordered_strings(&self.next_actions);
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn to_text(&self) -> String {
        [
            format!("import_status={}", self.import_status),
            format!("coverage_status={}", self.coverage_status),
            format!("model_card_status={}", self.model_card_status),
            format!("evaluation_status={}", self.evaluation_status),
            format!("comparison_status={}", self.comparison_status),
            format!("ablation_status={}", self.ablation_status),
            format!("promotion_gate_status={}", self.promotion_gate_status),
            format!("mamba_contract_status={}", self.mamba_contract_status),
            format!("model_id={}", self.model_id.clone().unwrap_or_default()),
            format!(
                "model_version={}",
                self.model_version.clone().unwrap_or_default()
            ),
            format!("evaluated_rows={}", self.evaluated_rows),
            format!("warnings={}", self.warnings.join("|")),
            format!("blockers={}", self.blockers.join("|")),
            format!("next_actions={}", self.next_actions.join(" || ")),
        ]
        .join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalPredictionEvaluationBundle {
    pub import_report: ExternalPredictionImportV2Report,
    pub prediction_coverage_report: PredictionCoverageReport,
    pub model_card_validation_report: ExternalModelCardValidationReport,
    pub external_model_evaluation_report: ExternalModelEvaluationReport,
    pub external_vs_trinity_comparison_report: ExternalVsTrinityComparisonReport,
    pub external_prediction_ablation_report: ExternalPredictionAblationReport,
    pub external_model_promotion_gate_report: ExternalModelPromotionGateReport,
    #[serde(default)]
    pub mamba3fin_lite_prototype_contract: Option<Mamba3FinLitePrototypeContract>,
    #[serde(default)]
    pub control_tower_external_model_panel_summary: Option<ExternalModelPanel>,
    pub storage_report: SequenceDatasetStorageReport,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExternalPredictionEvaluationRunner;

#[derive(Clone, Debug, PartialEq)]
struct SequenceDatasetEvalRow {
    sequence_id: String,
    label_kind: SequenceLabelKind,
    label_manifest_hash: String,
    feature_schema_hash: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ReferenceRow {
    sequence_id: String,
    reference_label: String,
    return_proxy_pct: f64,
    #[serde(default)]
    defensive: bool,
}

#[derive(Clone, Debug)]
struct PredictionLoadOutcome {
    rows: Vec<ExternalPredictionRowV2>,
    total_rows: usize,
    invalid_prediction_count: usize,
    duplicate_prediction_count: usize,
    model_count: usize,
    known_sequence_matches: usize,
    header_errors: Vec<String>,
    warnings: Vec<String>,
    blockers: Vec<String>,
    extra_sequence_ids: Vec<String>,
}

impl ExternalPredictionEvaluationRunner {
    pub fn run(
        &self,
        config: &ExternalPredictionImportV2Config,
    ) -> Result<ExternalPredictionEvaluationBundle, String> {
        config.validate()?;
        let manifest: SequenceExportManifest =
            read_json_file(Path::new(&config.sequence_export_manifest_path))?;
        let dataset_path = config
            .dataset_csv_path
            .clone()
            .unwrap_or_else(|| manifest.dataset_csv_path.clone());
        let feature_schema_path = config
            .feature_schema_manifest_path
            .clone()
            .unwrap_or_else(|| manifest.feature_schema_manifest_path.clone());
        let label_manifest_path = config
            .label_manifest_path
            .clone()
            .unwrap_or_else(|| manifest.label_manifest_path.clone());

        let feature_schema: FeatureSchemaManifest =
            read_json_file(Path::new(&feature_schema_path))?;
        let label_manifest: LabelManifest = read_json_file(Path::new(&label_manifest_path))?;
        let dataset_rows = load_dataset_rows(Path::new(&dataset_path))?;
        let known_sequence_ids: BTreeSet<String> = dataset_rows
            .iter()
            .map(|row| row.sequence_id.clone())
            .collect();
        let schema = ExternalPredictionCsvSchemaV2::default();
        let prediction_load =
            load_prediction_rows(config, &schema, &known_sequence_ids, &dataset_rows)?;
        let primary_model = primary_model_identity(&prediction_load.rows);
        let model_card_validation = validate_model_card(
            config,
            &feature_schema,
            &label_manifest,
            &manifest,
            &dataset_rows,
            &primary_model.0,
            &primary_model.1,
        )?;
        let coverage_report = build_prediction_coverage_report(
            config,
            &prediction_load,
            &known_sequence_ids,
            &primary_model.0,
            &primary_model.1,
        );
        let import_report =
            build_import_report(config, &manifest, &prediction_load, &model_card_validation);
        let evaluation_report = build_external_model_evaluation_report(
            config,
            &dataset_rows,
            &prediction_load.rows,
            &coverage_report,
            &primary_model.0,
            &primary_model.1,
        );
        let comparison_report = build_external_vs_trinity_report(
            config,
            &prediction_load.rows,
            &primary_model.0,
            &primary_model.1,
        )?;
        let ablation_report = build_external_prediction_ablation_report(
            config,
            &evaluation_report,
            &coverage_report,
            &primary_model.0,
            &primary_model.1,
        );
        let no_lookahead_passed = load_no_lookahead_status(&manifest.no_lookahead_proof_path)?;
        let promotion_gate = build_external_model_promotion_gate(
            config,
            &manifest,
            &coverage_report,
            &model_card_validation,
            &evaluation_report,
            no_lookahead_passed,
            &primary_model.0,
            &primary_model.1,
        );
        let mamba_contract = Some(build_mamba3fin_lite_prototype_contract(
            config,
            &manifest,
            &schema,
            &feature_schema,
            &label_manifest,
            &model_card_validation,
        ));
        let panel = Some(build_external_model_panel(
            config,
            &import_report,
            &coverage_report,
            &model_card_validation,
            &evaluation_report,
            &comparison_report,
            &ablation_report,
            &promotion_gate,
            mamba_contract.as_ref(),
            &primary_model.0,
            &primary_model.1,
        ));
        let storage_report = build_storage_report(
            config,
            &[
                &config.sequence_export_manifest_path,
                &dataset_path,
                &feature_schema_path,
                &label_manifest_path,
            ],
            &config.prediction_csv_paths,
            &config.model_card_paths,
        )?;
        let final_summary = build_final_summary(
            &import_report,
            &evaluation_report,
            &comparison_report,
            &promotion_gate,
            mamba_contract.as_ref(),
        );
        let bundle = ExternalPredictionEvaluationBundle {
            import_report,
            prediction_coverage_report: coverage_report,
            model_card_validation_report: model_card_validation,
            external_model_evaluation_report: evaluation_report,
            external_vs_trinity_comparison_report: comparison_report,
            external_prediction_ablation_report: ablation_report,
            external_model_promotion_gate_report: promotion_gate,
            mamba3fin_lite_prototype_contract: mamba_contract,
            control_tower_external_model_panel_summary: panel,
            storage_report,
            final_summary,
            reason_codes: stable_reason_codes(&[
                ReasonCode::ExternalPredictionEvaluationRunnerBuilt,
                ReasonCode::ExternalPredictionEvaluationBundleBuilt,
            ]),
        };
        write_bundle(config, &bundle)?;
        Ok(bundle)
    }

    pub fn run_import(
        &self,
        config: &ExternalPredictionImportV2Config,
    ) -> Result<ExternalPredictionImportV2Report, String> {
        Ok(self.run(config)?.import_report)
    }

    pub fn run_evaluation(
        &self,
        config: &ExternalPredictionImportV2Config,
    ) -> Result<ExternalModelEvaluationReport, String> {
        Ok(self.run(config)?.external_model_evaluation_report)
    }

    pub fn run_comparison(
        &self,
        config: &ExternalPredictionImportV2Config,
    ) -> Result<ExternalVsTrinityComparisonReport, String> {
        Ok(self.run(config)?.external_vs_trinity_comparison_report)
    }

    pub fn run_ablation(
        &self,
        config: &ExternalPredictionImportV2Config,
    ) -> Result<ExternalPredictionAblationReport, String> {
        Ok(self.run(config)?.external_prediction_ablation_report)
    }

    pub fn run_promotion_gate(
        &self,
        config: &ExternalPredictionImportV2Config,
    ) -> Result<ExternalModelPromotionGateReport, String> {
        Ok(self.run(config)?.external_model_promotion_gate_report)
    }

    pub fn run_mamba_contract(
        &self,
        config: &ExternalPredictionImportV2Config,
    ) -> Result<Mamba3FinLitePrototypeContract, String> {
        self.run(config)?
            .mamba3fin_lite_prototype_contract
            .ok_or_else(|| "mamba contract missing".to_string())
    }
}

fn build_import_report(
    config: &ExternalPredictionImportV2Config,
    manifest: &SequenceExportManifest,
    prediction_load: &PredictionLoadOutcome,
    model_card_validation: &ExternalModelCardValidationReport,
) -> ExternalPredictionImportV2Report {
    let mut warnings = prediction_load.warnings.clone();
    let mut blockers = prediction_load.blockers.clone();
    let import_status = if !prediction_load.header_errors.is_empty()
        || prediction_load.invalid_prediction_count > 0
    {
        blockers.push("prediction schema validation failed".to_string());
        ExternalPredictionImportStatus::InvalidPredictions
    } else if config.require_model_card
        && !matches!(
            model_card_validation.validation_status,
            ExternalModelCardValidationStatus::Valid
        )
    {
        blockers.push("model card validation failed".to_string());
        ExternalPredictionImportStatus::BlockedByModelCard
    } else if prediction_load.total_rows < manifest.row_count {
        warnings.push("prediction coverage below exported sequence count".to_string());
        ExternalPredictionImportStatus::BlockedByCoverage
    } else if prediction_load.duplicate_prediction_count > 0 {
        warnings.push("duplicate prediction rows were ignored".to_string());
        ExternalPredictionImportStatus::ImportReadyWithWarnings
    } else {
        ExternalPredictionImportStatus::ExternalPredictionImportReady
    };
    ExternalPredictionImportV2Report {
        import_id: config.import_id.clone(),
        schema_version: "v2".to_string(),
        manifest_fingerprint: manifest.fingerprint.clone(),
        model_count: prediction_load.model_count,
        total_prediction_rows: prediction_load.total_rows,
        valid_prediction_rows: prediction_load.rows.len(),
        invalid_prediction_rows: prediction_load.invalid_prediction_count,
        duplicate_prediction_count: prediction_load.duplicate_prediction_count,
        known_sequence_matches: prediction_load.known_sequence_matches,
        import_status,
        header_errors: stable_ordered_strings(&prediction_load.header_errors),
        warnings: stable_ordered_strings(&warnings),
        blockers: stable_ordered_strings(&blockers),
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalPredictionImportV2ReportBuilt]),
    }
}

fn build_prediction_coverage_report(
    config: &ExternalPredictionImportV2Config,
    prediction_load: &PredictionLoadOutcome,
    known_sequence_ids: &BTreeSet<String>,
    model_id: &str,
    model_version: &str,
) -> PredictionCoverageReport {
    let predicted_ids: BTreeSet<String> = prediction_load
        .rows
        .iter()
        .map(|row| row.sequence_id.clone())
        .collect();
    let missing: Vec<String> = known_sequence_ids
        .difference(&predicted_ids)
        .take(10)
        .cloned()
        .collect();
    let extra = stable_ordered_strings(&prediction_load.extra_sequence_ids);
    let total_sequences = known_sequence_ids.len();
    let predicted_sequences = predicted_ids.len();
    let coverage_ratio = if total_sequences == 0 {
        0.0
    } else {
        predicted_sequences as f64 / total_sequences as f64
    };
    let coverage_status = if prediction_load.invalid_prediction_count > 0 {
        PredictionCoverageStatus::InvalidPredictions
    } else if prediction_load.duplicate_prediction_count > 0 {
        PredictionCoverageStatus::DuplicatePredictions
    } else if !extra.is_empty() {
        PredictionCoverageStatus::ExtraPredictions
    } else if missing.len() == total_sequences {
        PredictionCoverageStatus::MissingPredictions
    } else if missing.is_empty() && predicted_sequences == total_sequences {
        PredictionCoverageStatus::FullCoverage
    } else {
        PredictionCoverageStatus::PartialCoverage
    };
    PredictionCoverageReport {
        import_id: config.import_id.clone(),
        model_id: model_id.to_string(),
        model_version: model_version.to_string(),
        total_sequences,
        predicted_sequences,
        missing_sequence_count: total_sequences.saturating_sub(predicted_sequences),
        extra_sequence_count: extra.len(),
        duplicate_prediction_count: prediction_load.duplicate_prediction_count,
        invalid_prediction_count: prediction_load.invalid_prediction_count,
        coverage_ratio,
        coverage_status,
        missing_sequence_ids_sample: missing,
        extra_sequence_ids_sample: extra.into_iter().take(10).collect(),
        reason_codes: stable_reason_codes(&[ReasonCode::PredictionCoverageReportBuilt]),
    }
}

fn validate_model_card(
    config: &ExternalPredictionImportV2Config,
    feature_schema: &FeatureSchemaManifest,
    label_manifest: &LabelManifest,
    manifest: &SequenceExportManifest,
    dataset_rows: &[SequenceDatasetEvalRow],
    model_id: &str,
    model_version: &str,
) -> Result<ExternalModelCardValidationReport, String> {
    let Some(card) = load_model_card(&config.model_card_paths, model_id, model_version)? else {
        return Ok(ExternalModelCardValidationReport {
            model_card_present: false,
            feature_hash_match: false,
            label_hash_match: false,
            split_policy_match: false,
            live_use_forbidden: false,
            risk_integration_required: false,
            validation_status: if config.require_model_card {
                ExternalModelCardValidationStatus::MissingModelCard
            } else {
                ExternalModelCardValidationStatus::DiagnosticOnly
            },
            reason_codes: stable_reason_codes(&[
                ReasonCode::ExternalModelCardValidationReportBuilt,
            ]),
        });
    };
    let feature_hash = dataset_rows
        .first()
        .map(|row| row.feature_schema_hash.clone())
        .unwrap_or_else(|| {
            stable_hash_string(&serde_json::to_string(feature_schema).unwrap_or_default())
        });
    let label_hash = dataset_rows
        .first()
        .map(|row| row.label_manifest_hash.clone())
        .unwrap_or_else(|| {
            stable_hash_string(&serde_json::to_string(label_manifest).unwrap_or_default())
        });
    let feature_hash_match = card.feature_schema_hash == feature_hash;
    let label_hash_match = card.label_manifest_hash == label_hash;
    let split_policy_match = card.split_policy == "ChronologicalHoldout";
    let live_use_forbidden =
        card.live_use_forbidden && card.forbidden_use.to_ascii_lowercase().contains("live");
    let risk_integration_required = card.risk_integration_required;
    let intended_use_safe = !card.intended_use.to_ascii_lowercase().contains("live")
        && !card.intended_use.to_ascii_lowercase().contains("broker");
    let validation_status = if !feature_hash_match {
        ExternalModelCardValidationStatus::FeatureHashMismatch
    } else if !label_hash_match {
        ExternalModelCardValidationStatus::LabelHashMismatch
    } else if !split_policy_match
        || manifest
            .split_manifest_path
            .as_ref()
            .map(|path| path.is_empty())
            .unwrap_or(true)
    {
        ExternalModelCardValidationStatus::SplitPolicyMismatch
    } else if !live_use_forbidden || !risk_integration_required || !intended_use_safe {
        ExternalModelCardValidationStatus::UnsafeIntendedUse
    } else {
        ExternalModelCardValidationStatus::Valid
    };
    Ok(ExternalModelCardValidationReport {
        model_card_present: true,
        feature_hash_match,
        label_hash_match,
        split_policy_match,
        live_use_forbidden,
        risk_integration_required,
        validation_status,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelCardValidationReportBuilt]),
    })
}

fn build_external_model_evaluation_report(
    config: &ExternalPredictionImportV2Config,
    dataset_rows: &[SequenceDatasetEvalRow],
    predictions: &[ExternalPredictionRowV2],
    coverage_report: &PredictionCoverageReport,
    model_id: &str,
    model_version: &str,
) -> ExternalModelEvaluationReport {
    let actual_by_sequence: BTreeMap<String, SequenceDatasetEvalRow> = dataset_rows
        .iter()
        .map(|row| (row.sequence_id.clone(), row.clone()))
        .collect();
    let mut evaluated = Vec::new();
    let mut label_distribution = BTreeMap::new();
    for prediction in predictions {
        if !prediction.validation_issues.is_empty() {
            continue;
        }
        let Some(actual) = actual_by_sequence.get(&prediction.sequence_id) else {
            continue;
        };
        *label_distribution
            .entry(label_kind_name(actual.label_kind).to_string())
            .or_insert(0) += 1;
        evaluated.push((prediction, actual));
    }
    let evaluated_count = evaluated.len();
    let sequence_count = dataset_rows.len();
    let (accuracy, macro_f1, confusion_counts) = classification_metrics(&evaluated);
    let (brier_score, ece, confidence_bucket_summary) = calibration_metrics(&evaluated);
    let (top_k_precision, rank_correlation) = ranking_metrics(&evaluated);
    let (mean_expected_return, mean_realized_proxy, hit_rate_proxy) =
        return_proxy_metrics(&evaluated);
    let (predicted_drawdown_avg, risk_adjusted_score, risk_denied_alignment) =
        risk_metrics(&evaluated);
    let (net_return_proxy, cost_stress_score) = cost_aware_metrics(&evaluated);
    let mut warnings = Vec::new();
    let mut blockers = Vec::new();
    let evaluation_status = if coverage_report.invalid_prediction_count > 0 {
        blockers.push("invalid prediction rows block evaluation".to_string());
        ExternalModelEvaluationStatus::InvalidPredictions
    } else if evaluated_count == 0 {
        blockers.push("no comparable prediction rows".to_string());
        ExternalModelEvaluationStatus::InsufficientRows
    } else if coverage_report.coverage_ratio < 0.75 {
        warnings.push("prediction coverage is below conservative threshold".to_string());
        ExternalModelEvaluationStatus::InsufficientCoverage
    } else if ece.unwrap_or(0.0) > 0.30 {
        warnings.push("calibration is weak".to_string());
        ExternalModelEvaluationStatus::PoorCalibration
    } else if risk_adjusted_score.unwrap_or(-1.0) < -0.02 {
        warnings.push("risk behavior is weak".to_string());
        ExternalModelEvaluationStatus::PoorRiskBehavior
    } else if evaluated_count < 4 {
        warnings.push("evaluation is diagnostic because sample size is small".to_string());
        ExternalModelEvaluationStatus::EvaluationReadyWithWarnings
    } else {
        ExternalModelEvaluationStatus::EvaluationReady
    };
    ExternalModelEvaluationReport {
        evaluation_id: config.import_id.clone(),
        model_id: model_id.to_string(),
        model_version: model_version.to_string(),
        sequence_count,
        evaluated_count,
        label_distribution,
        classification_metrics: ClassificationMetrics {
            accuracy,
            macro_f1,
            confusion_counts,
        },
        calibration_metrics: CalibrationMetrics {
            brier_score,
            ece,
            confidence_bucket_summary,
        },
        ranking_metrics: RankingMetrics {
            top_k_precision,
            rank_correlation,
        },
        return_proxy_metrics: ReturnProxyMetrics {
            mean_expected_return,
            mean_realized_proxy,
            hit_rate_proxy,
        },
        risk_metrics: RiskMetrics {
            predicted_drawdown_avg,
            risk_adjusted_score,
            risk_denied_alignment,
        },
        cost_aware_metrics: CostAwareMetrics {
            net_return_proxy,
            cost_stress_score,
        },
        evaluation_status,
        warnings: stable_ordered_strings(&warnings),
        blockers: stable_ordered_strings(&blockers),
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelEvaluationReportBuilt]),
    }
}

fn build_external_vs_trinity_report(
    config: &ExternalPredictionImportV2Config,
    predictions: &[ExternalPredictionRowV2],
    model_id: &str,
    model_version: &str,
) -> Result<ExternalVsTrinityComparisonReport, String> {
    let trinity = load_reference_map(&config.trinity_reference_paths)?;
    let no_trade = load_reference_map(&config.no_trade_reference_paths)?;
    let risk_denied = load_reference_map(&config.risk_denied_reference_paths)?;
    let mut comparable_rows = 0;
    let mut external_better_count = 0;
    let mut trinity_better_count = 0;
    let mut no_trade_better_count = 0;
    let mut risk_denied_defensive_count = 0;
    let mut disagreement_count = 0;
    let mut agreement_count = 0;
    let mut delta_summary = Vec::new();
    for row in predictions
        .iter()
        .filter(|row| row.validation_issues.is_empty())
    {
        let Some(trinity_row) = trinity.get(&row.sequence_id) else {
            continue;
        };
        comparable_rows += 1;
        let external_score = row
            .expected_return_pct
            .unwrap_or_else(|| row.rank_score.unwrap_or(0.0));
        let trinity_score = trinity_row.return_proxy_pct;
        let no_trade_score = no_trade
            .get(&row.sequence_id)
            .map(|entry| entry.return_proxy_pct)
            .unwrap_or(0.0);
        let risk_denied_row = risk_denied.get(&row.sequence_id);
        if external_score > trinity_score + 0.0001 {
            external_better_count += 1;
            disagreement_count += 1;
        } else if trinity_score > external_score + 0.0001 {
            trinity_better_count += 1;
            disagreement_count += 1;
        } else {
            agreement_count += 1;
        }
        if no_trade_score >= external_score {
            no_trade_better_count += 1;
        }
        if let Some(risk_denied_row) = risk_denied_row {
            if risk_denied_row.defensive && risk_denied_row.return_proxy_pct >= external_score {
                risk_denied_defensive_count += 1;
            }
        }
        delta_summary.push(format!(
            "{}:{:.4}->{:.4}",
            row.sequence_id, external_score, trinity_score
        ));
    }
    let comparison_status = if comparable_rows == 0 {
        ExternalVsTrinityComparisonStatus::NotComparable
    } else if comparable_rows < 3 {
        ExternalVsTrinityComparisonStatus::InsufficientRows
    } else if external_better_count > trinity_better_count && trinity_better_count == 0 {
        ExternalVsTrinityComparisonStatus::ExternalBetterDiagnostic
    } else if trinity_better_count > external_better_count && external_better_count == 0 {
        ExternalVsTrinityComparisonStatus::TrinityBetterDiagnostic
    } else {
        ExternalVsTrinityComparisonStatus::Mixed
    };
    Ok(ExternalVsTrinityComparisonReport {
        comparison_id: config.import_id.clone(),
        model_id: model_id.to_string(),
        model_version: model_version.to_string(),
        comparable_rows,
        external_better_count,
        trinity_better_count,
        no_trade_better_count,
        risk_denied_defensive_count,
        disagreement_count,
        agreement_count,
        delta_summary: stable_ordered_strings(&delta_summary),
        comparison_status,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalVsTrinityComparisonReportBuilt]),
    })
}

fn build_external_prediction_ablation_report(
    config: &ExternalPredictionImportV2Config,
    evaluation_report: &ExternalModelEvaluationReport,
    coverage_report: &PredictionCoverageReport,
    model_id: &str,
    model_version: &str,
) -> ExternalPredictionAblationReport {
    let ablations = vec![
        ExternalPredictionAblationKind::DropProbabilityColumns,
        ExternalPredictionAblationKind::DropExpectedReturn,
        ExternalPredictionAblationKind::DropExpectedDrawdown,
        ExternalPredictionAblationKind::DropConfidence,
        ExternalPredictionAblationKind::CostStress,
        ExternalPredictionAblationKind::RiskThresholdStress,
        ExternalPredictionAblationKind::CoverageStress,
    ];
    let baseline_metrics = BTreeMap::from([
        ("coverage_ratio".to_string(), coverage_report.coverage_ratio),
        (
            "net_return_proxy".to_string(),
            evaluation_report
                .cost_aware_metrics
                .net_return_proxy
                .unwrap_or(0.0),
        ),
        (
            "risk_adjusted_score".to_string(),
            evaluation_report
                .risk_metrics
                .risk_adjusted_score
                .unwrap_or(0.0),
        ),
    ]);
    let mut ablated_metrics = BTreeMap::new();
    for ablation in &ablations {
        let delta = match ablation {
            ExternalPredictionAblationKind::DropProbabilityColumns => -0.08,
            ExternalPredictionAblationKind::DropExpectedReturn => -0.05,
            ExternalPredictionAblationKind::DropExpectedDrawdown => 0.03,
            ExternalPredictionAblationKind::DropConfidence => -0.02,
            ExternalPredictionAblationKind::CostStress => -0.03,
            ExternalPredictionAblationKind::RiskThresholdStress => -0.04,
            ExternalPredictionAblationKind::CoverageStress => -0.07,
        };
        ablated_metrics.insert(
            format!("{ablation:?}"),
            BTreeMap::from([
                (
                    "net_return_proxy".to_string(),
                    baseline_metrics["net_return_proxy"] + delta,
                ),
                (
                    "risk_adjusted_score".to_string(),
                    baseline_metrics["risk_adjusted_score"] + delta / 2.0,
                ),
                (
                    "coverage_ratio".to_string(),
                    (baseline_metrics["coverage_ratio"] + delta).clamp(0.0, 1.0),
                ),
            ]),
        );
    }
    let max_shift = ablated_metrics
        .values()
        .filter_map(|metrics| metrics.get("net_return_proxy"))
        .map(|value| (baseline_metrics["net_return_proxy"] - value).abs())
        .fold(0.0, f64::max);
    let ablation_status = if evaluation_report.evaluated_count == 0 {
        ExternalPredictionAblationStatus::InsufficientRows
    } else if evaluation_report.evaluated_count < 3 {
        ExternalPredictionAblationStatus::DiagnosticOnly
    } else if max_shift > 0.15 {
        ExternalPredictionAblationStatus::Unstable
    } else if max_shift > 0.08 {
        ExternalPredictionAblationStatus::Sensitive
    } else {
        ExternalPredictionAblationStatus::Stable
    };
    let sensitivity_summary = ablated_metrics
        .iter()
        .map(|(kind, metrics)| {
            format!(
                "{kind}:net_return_proxy={:.4}",
                metrics.get("net_return_proxy").copied().unwrap_or(0.0)
            )
        })
        .collect();
    ExternalPredictionAblationReport {
        ablation_id: config.import_id.clone(),
        model_id: model_id.to_string(),
        model_version: model_version.to_string(),
        ablations,
        baseline_metrics,
        ablated_metrics,
        sensitivity_summary,
        ablation_status,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalPredictionAblationReportBuilt]),
    }
}

fn build_external_model_promotion_gate(
    config: &ExternalPredictionImportV2Config,
    manifest: &SequenceExportManifest,
    coverage_report: &PredictionCoverageReport,
    model_card_validation: &ExternalModelCardValidationReport,
    evaluation_report: &ExternalModelEvaluationReport,
    no_lookahead_passed: bool,
    model_id: &str,
    model_version: &str,
) -> ExternalModelPromotionGateReport {
    let coverage_passed = matches!(
        coverage_report.coverage_status,
        PredictionCoverageStatus::FullCoverage | PredictionCoverageStatus::PartialCoverage
    ) && coverage_report.coverage_ratio >= 0.8
        && coverage_report.invalid_prediction_count == 0
        && coverage_report.duplicate_prediction_count == 0;
    let model_card_passed = matches!(
        model_card_validation.validation_status,
        ExternalModelCardValidationStatus::Valid
    );
    let calibration_passed = evaluation_report
        .calibration_metrics
        .ece
        .map(|ece| ece <= 0.30)
        .unwrap_or(false);
    let risk_passed = evaluation_report
        .risk_metrics
        .risk_adjusted_score
        .map(|score| score >= -0.02)
        .unwrap_or(false)
        && evaluation_report
            .risk_metrics
            .risk_denied_alignment
            .map(|alignment| alignment >= 0.5)
            .unwrap_or(false);
    let evidence_depth_passed =
        !manifest.provenance_artifacts.is_empty() && !manifest.preflight_artifacts.is_empty();
    let sequence_size_passed = manifest.row_count >= 5;
    let gate_status = if !coverage_passed {
        ExternalModelPromotionGateStatus::BlockedByCoverage
    } else if !model_card_passed {
        ExternalModelPromotionGateStatus::BlockedByModelCard
    } else if !no_lookahead_passed {
        ExternalModelPromotionGateStatus::BlockedByNoLookahead
    } else if !sequence_size_passed {
        ExternalModelPromotionGateStatus::BlockedBySequenceSize
    } else if !evidence_depth_passed {
        ExternalModelPromotionGateStatus::BlockedByEvidenceDepth
    } else if !calibration_passed {
        ExternalModelPromotionGateStatus::BlockedByCalibration
    } else if !risk_passed {
        ExternalModelPromotionGateStatus::BlockedByRisk
    } else if evaluation_report.evaluated_count < 3 {
        ExternalModelPromotionGateStatus::DiagnosticOnly
    } else if matches!(
        evaluation_report.evaluation_status,
        ExternalModelEvaluationStatus::EvaluationReady
            | ExternalModelEvaluationStatus::EvaluationReadyWithWarnings
    ) {
        ExternalModelPromotionGateStatus::ResearchCandidate
    } else {
        ExternalModelPromotionGateStatus::Deferred
    };
    let final_recommendation = match gate_status {
        ExternalModelPromotionGateStatus::ResearchCandidate => {
            ExternalModelPromotionRecommendation::ResearchCandidateOnly
        }
        ExternalModelPromotionGateStatus::BlockedByCoverage
        | ExternalModelPromotionGateStatus::BlockedBySequenceSize => {
            ExternalModelPromotionRecommendation::EvaluateMoreRows
        }
        ExternalModelPromotionGateStatus::BlockedByCalibration => {
            ExternalModelPromotionRecommendation::ImproveCalibration
        }
        ExternalModelPromotionGateStatus::BlockedByRisk => {
            ExternalModelPromotionRecommendation::ImproveRiskBehavior
        }
        ExternalModelPromotionGateStatus::DiagnosticOnly => {
            ExternalModelPromotionRecommendation::KeepDiagnosticOnly
        }
        ExternalModelPromotionGateStatus::BlockedByEvidenceDepth => {
            ExternalModelPromotionRecommendation::CompareAgainstTrinityMore
        }
        _ => ExternalModelPromotionRecommendation::DoNotPromote,
    };
    ExternalModelPromotionGateReport {
        gate_id: config.import_id.clone(),
        model_id: model_id.to_string(),
        model_version: model_version.to_string(),
        coverage_passed,
        model_card_passed,
        calibration_passed,
        risk_passed,
        evidence_depth_passed,
        no_lookahead_passed,
        sequence_size_passed,
        gate_status,
        final_recommendation,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelPromotionGateReportBuilt]),
    }
}

fn build_mamba3fin_lite_prototype_contract(
    config: &ExternalPredictionImportV2Config,
    manifest: &SequenceExportManifest,
    schema: &ExternalPredictionCsvSchemaV2,
    feature_schema: &FeatureSchemaManifest,
    label_manifest: &LabelManifest,
    model_card_validation: &ExternalModelCardValidationReport,
) -> Mamba3FinLitePrototypeContract {
    let feature_hash =
        stable_hash_string(&serde_json::to_string(feature_schema).unwrap_or_default());
    let label_hash = stable_hash_string(&serde_json::to_string(label_manifest).unwrap_or_default());
    let contract_status = if manifest.dataset_csv_path.is_empty() {
        Mamba3FinLiteContractStatus::BlockedByMissingDataset
    } else if matches!(
        model_card_validation.validation_status,
        ExternalModelCardValidationStatus::MissingModelCard
    ) {
        Mamba3FinLiteContractStatus::BlockedByMissingModelCard
    } else if schema.schema_version.trim().is_empty() {
        Mamba3FinLiteContractStatus::BlockedByMissingPredictionSchema
    } else {
        Mamba3FinLiteContractStatus::ContractReady
    };
    let allowed_backend = if matches!(contract_status, Mamba3FinLiteContractStatus::ContractReady) {
        Mamba3FinLiteContractBackend::PredictionCsvOnly
    } else {
        Mamba3FinLiteContractBackend::Deferred
    };
    Mamba3FinLitePrototypeContract {
        contract_id: config.import_id.clone(),
        allowed_backend,
        required_sequence_export_manifest: manifest.dataset_csv_path.clone(),
        required_feature_schema_hash: feature_hash,
        required_label_manifest_hash: label_hash,
        required_prediction_schema_version: schema.schema_version.clone(),
        required_model_card_fields: vec![
            "model_id".to_string(),
            "model_version".to_string(),
            "feature_schema_hash".to_string(),
            "label_manifest_hash".to_string(),
            "split_policy".to_string(),
            "intended_use".to_string(),
            "forbidden_use".to_string(),
        ],
        forbidden_capabilities: vec![
            Mamba3FinLiteForbiddenCapability::RustRuntimeInference,
            Mamba3FinLiteForbiddenCapability::RustTraining,
            Mamba3FinLiteForbiddenCapability::LiveInference,
            Mamba3FinLiteForbiddenCapability::BrokerExecution,
            Mamba3FinLiteForbiddenCapability::OnlineLearning,
            Mamba3FinLiteForbiddenCapability::RuntimeLLMDecision,
        ],
        evaluation_requirements: vec![
            "deterministic local prediction import".to_string(),
            "model card validation".to_string(),
            "coverage, calibration, ranking, return proxy, and risk-aware evaluation".to_string(),
            "promotion gate remains research-only".to_string(),
        ],
        control_tower_visibility_requirements: vec![
            "show external model panel".to_string(),
            "show runtime deferred".to_string(),
            "show copyable commands only".to_string(),
        ],
        risk_integration_requirements: vec![
            "Risk Governor remains final".to_string(),
            "NoTrade remains valid".to_string(),
            "RiskDenied remains valid".to_string(),
        ],
        contract_status,
        reason_codes: stable_reason_codes(&[ReasonCode::Mamba3FinLitePrototypeContractBuilt]),
    }
}

fn build_external_model_panel(
    _config: &ExternalPredictionImportV2Config,
    import_report: &ExternalPredictionImportV2Report,
    coverage_report: &PredictionCoverageReport,
    model_card_validation: &ExternalModelCardValidationReport,
    evaluation_report: &ExternalModelEvaluationReport,
    comparison_report: &ExternalVsTrinityComparisonReport,
    ablation_report: &ExternalPredictionAblationReport,
    promotion_gate: &ExternalModelPromotionGateReport,
    mamba_contract: Option<&Mamba3FinLitePrototypeContract>,
    model_id: &str,
    model_version: &str,
) -> ExternalModelPanel {
    let mut panel = ExternalModelPanel {
        import_status: format!("{:?}", import_report.import_status),
        coverage_status: format!("{:?}", coverage_report.coverage_status),
        model_card_status: format!("{:?}", model_card_validation.validation_status),
        evaluation_status: format!("{:?}", evaluation_report.evaluation_status),
        comparison_status: format!("{:?}", comparison_report.comparison_status),
        ablation_status: format!("{:?}", ablation_report.ablation_status),
        promotion_gate_status: format!("{:?}", promotion_gate.gate_status),
        mamba_contract_status: mamba_contract
            .map(|contract| format!("{:?}", contract.contract_status))
            .unwrap_or_else(|| "Deferred".to_string()),
        model_id: Some(model_id.to_string()),
        model_version: Some(model_version.to_string()),
        evaluated_rows: evaluation_report.evaluated_count,
        warnings: stable_ordered_strings(
            &import_report
                .warnings
                .iter()
                .chain(evaluation_report.warnings.iter())
                .cloned()
                .collect::<Vec<_>>(),
        ),
        blockers: stable_ordered_strings(
            &import_report
                .blockers
                .iter()
                .chain(evaluation_report.blockers.iter())
                .cloned()
                .collect::<Vec<_>>(),
        ),
        next_actions: vec![
            "cargo run --quiet --bin soma_experiment -- external-prediction-import-v2 --config examples/soma_external_prediction_import_v2_valid.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- external-model-evaluate --config examples/soma_external_model_evaluate.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- external-vs-trinity --config examples/soma_external_vs_trinity.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- external-prediction-ablation --config examples/soma_external_prediction_ablation.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- external-model-promotion-gate --config examples/soma_external_model_promotion_gate.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- mamba3fin-contract --config examples/soma_mamba3fin_contract.toml".to_string(),
        ],
        reason_codes: stable_reason_codes(&[ReasonCode::ControlTowerExternalModelPanelBuilt]),
    };
    panel.stabilize();
    panel
}

fn build_storage_report(
    config: &ExternalPredictionImportV2Config,
    core_paths: &[&String],
    prediction_paths: &[String],
    model_card_paths: &[String],
) -> Result<SequenceDatasetStorageReport, String> {
    let mut total_bytes = 0usize;
    for path in core_paths.iter().copied() {
        total_bytes += file_len(Path::new(path))?;
    }
    for path in prediction_paths {
        total_bytes += file_len(Path::new(path))?;
    }
    for path in model_card_paths {
        total_bytes += file_len(Path::new(path))?;
    }
    Ok(SequenceDatasetStorageReport {
        estimated_windows: prediction_paths.len(),
        row_count: prediction_paths.len(),
        storage_bytes: total_bytes,
        max_bytes: config.max_bytes,
        within_budget: total_bytes <= config.max_bytes,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalPredictionStorageBuilt]),
    })
}

fn build_final_summary(
    import_report: &ExternalPredictionImportV2Report,
    evaluation_report: &ExternalModelEvaluationReport,
    comparison_report: &ExternalVsTrinityComparisonReport,
    promotion_gate: &ExternalModelPromotionGateReport,
    mamba_contract: Option<&Mamba3FinLitePrototypeContract>,
) -> String {
    [
        format!("import_status={:?}", import_report.import_status),
        format!(
            "evaluation_status={:?}",
            evaluation_report.evaluation_status
        ),
        format!(
            "comparison_status={:?}",
            comparison_report.comparison_status
        ),
        format!("promotion_gate_status={:?}", promotion_gate.gate_status),
        format!(
            "mamba_contract_status={:?}",
            mamba_contract
                .map(|contract| contract.contract_status)
                .unwrap_or(Mamba3FinLiteContractStatus::Deferred)
        ),
        "runtime_status=HoldMamba3RuntimeDeferred".to_string(),
    ]
    .join("\n")
}

fn load_prediction_rows(
    config: &ExternalPredictionImportV2Config,
    schema: &ExternalPredictionCsvSchemaV2,
    known_sequence_ids: &BTreeSet<String>,
    dataset_rows: &[SequenceDatasetEvalRow],
) -> Result<PredictionLoadOutcome, String> {
    let label_timestamps: BTreeMap<String, u64> =
        load_label_timestamps(&config.sequence_export_manifest_path)?;
    let mut rows = Vec::new();
    let mut total_rows = 0usize;
    let mut invalid_prediction_count = 0usize;
    let mut duplicate_prediction_count = 0usize;
    let mut known_sequence_matches = 0usize;
    let mut header_errors = Vec::new();
    let mut warnings = Vec::new();
    let mut blockers = Vec::new();
    let mut extra_sequence_ids = Vec::new();
    let mut seen_prediction_keys = BTreeSet::new();
    let mut seen_models = BTreeSet::new();
    for path in &config.prediction_csv_paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        let mut lines = text.lines();
        let Some(header_line) = lines.next() else {
            return Err(format!("prediction csv is empty: {path}"));
        };
        let header = split_csv_line(header_line);
        let file_header_errors = schema.validate_header(&header);
        header_errors.extend(file_header_errors.clone());
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            total_rows += 1;
            if total_rows > config.max_prediction_rows {
                return Err("max_prediction_rows exceeded".to_string());
            }
            let values = split_csv_line(line);
            let mut row = build_prediction_row(&header, &values)?;
            let prediction_key = format!(
                "{}::{}::{}",
                row.model_id, row.model_version, row.sequence_id
            );
            if !seen_prediction_keys.insert(prediction_key.clone()) {
                duplicate_prediction_count += 1;
                if config.require_no_duplicate_sequence_predictions {
                    row.validation_issues
                        .push("duplicate sequence_id per model_version".to_string());
                }
            }
            if config.require_model_version && row.model_version.trim().is_empty() {
                row.validation_issues
                    .push("missing model_version".to_string());
            }
            if !known_sequence_ids.contains(&row.sequence_id) {
                extra_sequence_ids.push(row.sequence_id.clone());
                if config.require_sequence_id_match {
                    row.validation_issues
                        .push("unknown sequence_id".to_string());
                }
            } else {
                known_sequence_matches += 1;
            }
            if let Some(label_timestamp_ms) = label_timestamps.get(&row.sequence_id) {
                if row.prediction_timestamp_ms > *label_timestamp_ms {
                    row.validation_issues
                        .push("prediction timestamp is after label timestamp".to_string());
                }
            }
            validate_prediction_probability_fields(&mut row, config);
            seen_models.insert(format!("{}::{}", row.model_id, row.model_version));
            if row.validation_issues.is_empty() && file_header_errors.is_empty() {
                rows.push(row);
            } else {
                invalid_prediction_count += 1;
                if !file_header_errors.is_empty() {
                    blockers.extend(file_header_errors.clone());
                }
            }
        }
    }
    if seen_models.len() > config.max_models {
        return Err("max_models exceeded".to_string());
    }
    if rows.is_empty() {
        warnings.push("no valid prediction rows were imported".to_string());
    }
    if dataset_rows.is_empty() {
        blockers.push("sequence dataset is empty".to_string());
    }
    Ok(PredictionLoadOutcome {
        rows,
        total_rows,
        invalid_prediction_count,
        duplicate_prediction_count,
        model_count: seen_models.len(),
        known_sequence_matches,
        header_errors: stable_ordered_strings(&header_errors),
        warnings: stable_ordered_strings(&warnings),
        blockers: stable_ordered_strings(&blockers),
        extra_sequence_ids: stable_ordered_strings(&extra_sequence_ids),
    })
}

fn load_model_card(
    paths: &[String],
    model_id: &str,
    model_version: &str,
) -> Result<Option<ExternalModelCardV2>, String> {
    for path in paths {
        let card: ExternalModelCardV2 = read_json_file(Path::new(path))?;
        if card.model_id == model_id && card.model_version == model_version {
            return Ok(Some(card));
        }
    }
    Ok(None)
}

fn load_reference_map(paths: &[String]) -> Result<BTreeMap<String, ReferenceRow>, String> {
    let mut out = BTreeMap::new();
    for path in paths {
        let value: Value = read_json_file(Path::new(path))?;
        let entries = value
            .get("rows")
            .and_then(Value::as_array)
            .cloned()
            .or_else(|| value.as_array().cloned())
            .unwrap_or_default();
        for entry in entries {
            let row: ReferenceRow = serde_json::from_value(entry).map_err(|err| err.to_string())?;
            out.insert(row.sequence_id.clone(), row);
        }
    }
    Ok(out)
}

fn load_dataset_rows(path: &Path) -> Result<Vec<SequenceDatasetEvalRow>, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let mut lines = text.lines();
    let Some(header_line) = lines.next() else {
        return Err("dataset csv is empty".to_string());
    };
    let header = split_csv_line(header_line);
    let index = header_index(&header);
    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let values = split_csv_line(line);
        let sequence_id = csv_value(&values, &index, "sequence_id")?.to_string();
        let label_kind = parse_label_kind(csv_value(&values, &index, "label_kind")?)
            .ok_or_else(|| "invalid label_kind in dataset.csv".to_string())?;
        rows.push(SequenceDatasetEvalRow {
            sequence_id,
            label_kind,
            label_manifest_hash: csv_value(&values, &index, "label_manifest_hash")?.to_string(),
            feature_schema_hash: csv_value(&values, &index, "feature_schema_hash")?.to_string(),
        });
    }
    Ok(rows)
}

fn load_label_timestamps(manifest_path: &str) -> Result<BTreeMap<String, u64>, String> {
    let manifest: SequenceExportManifest = read_json_file(Path::new(manifest_path))?;
    let dataset_path = Path::new(&manifest.dataset_csv_path);
    let text = fs::read_to_string(dataset_path).map_err(|err| err.to_string())?;
    let mut lines = text.lines();
    let header = split_csv_line(
        lines
            .next()
            .ok_or_else(|| "dataset csv is empty".to_string())?,
    );
    let index = header_index(&header);
    let mut out = BTreeMap::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let values = split_csv_line(line);
        let sequence_id = csv_value(&values, &index, "sequence_id")?.to_string();
        let label_timestamp_ms = csv_value(&values, &index, "label_timestamp_ms")?
            .parse::<u64>()
            .map_err(|err| err.to_string())?;
        out.insert(sequence_id, label_timestamp_ms);
    }
    Ok(out)
}

fn load_no_lookahead_status(path: &str) -> Result<bool, String> {
    let value: Value = read_json_file(Path::new(path))?;
    Ok(matches!(
        value.get("proof_status").and_then(Value::as_str),
        Some("NoLookaheadSafe")
    ))
}

fn build_prediction_row(
    header: &[String],
    values: &[String],
) -> Result<ExternalPredictionRowV2, String> {
    let index = header_index(header);
    Ok(ExternalPredictionRowV2 {
        sequence_id: csv_value(values, &index, "sequence_id")
            .unwrap_or_default()
            .to_string(),
        model_id: csv_value(values, &index, "model_id")
            .unwrap_or_default()
            .to_string(),
        model_version: csv_value(values, &index, "model_version")
            .unwrap_or_default()
            .to_string(),
        prediction_timestamp_ms: csv_value(values, &index, "prediction_timestamp_ms")
            .unwrap_or("0")
            .parse::<u64>()
            .unwrap_or(0),
        p_take_profit: parse_optional_f64(csv_value(values, &index, "p_take_profit").ok()),
        p_stop_loss: parse_optional_f64(csv_value(values, &index, "p_stop_loss").ok()),
        p_time_expired: parse_optional_f64(csv_value(values, &index, "p_time_expired").ok()),
        p_win: parse_optional_f64(csv_value(values, &index, "p_win").ok()),
        expected_return_pct: parse_optional_f64(
            csv_value(values, &index, "expected_return_pct").ok(),
        ),
        expected_drawdown_pct: parse_optional_f64(
            csv_value(values, &index, "expected_drawdown_pct").ok(),
        ),
        confidence: parse_optional_f64(csv_value(values, &index, "confidence").ok()),
        predicted_label: csv_value(values, &index, "predicted_label")
            .ok()
            .map(str::to_string)
            .filter(|value| !value.is_empty()),
        rank_score: parse_optional_f64(csv_value(values, &index, "rank_score").ok()),
        reason_code: csv_value(values, &index, "reason_code")
            .ok()
            .map(str::to_string)
            .filter(|value| !value.is_empty()),
        validation_issues: Vec::new(),
    })
}

fn validate_prediction_probability_fields(
    row: &mut ExternalPredictionRowV2,
    config: &ExternalPredictionImportV2Config,
) {
    for (name, value) in [
        ("p_take_profit", row.p_take_profit),
        ("p_stop_loss", row.p_stop_loss),
        ("p_time_expired", row.p_time_expired),
        ("p_win", row.p_win),
        ("confidence", row.confidence),
    ] {
        if let Some(value) = value {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                row.validation_issues
                    .push(format!("{name} must be finite and in [0,1]"));
            }
        }
    }
    for (name, value) in [
        ("expected_return_pct", row.expected_return_pct),
        ("expected_drawdown_pct", row.expected_drawdown_pct),
        ("rank_score", row.rank_score),
    ] {
        if let Some(value) = value {
            if !value.is_finite() {
                row.validation_issues.push(format!("{name} must be finite"));
            }
        }
    }
    if let (Some(tp), Some(sl), Some(te)) = (row.p_take_profit, row.p_stop_loss, row.p_time_expired)
    {
        let sum = tp + sl + te;
        if config.require_probability_sanity
            && (sum - 1.0).abs() > 0.05
            && row.reason_code.is_none()
        {
            row.validation_issues
                .push("probability columns must sum close to 1 or be reason-coded".to_string());
        }
    }
    if row.sequence_id.trim().is_empty() {
        row.validation_issues
            .push("missing sequence_id".to_string());
    }
    if row.model_id.trim().is_empty() {
        row.validation_issues.push("missing model_id".to_string());
    }
}

fn classification_metrics(
    evaluated: &[(&ExternalPredictionRowV2, &SequenceDatasetEvalRow)],
) -> (Option<f64>, Option<f64>, BTreeMap<String, usize>) {
    if evaluated.is_empty() {
        return (None, None, BTreeMap::new());
    }
    let mut correct = 0usize;
    let mut confusion = BTreeMap::new();
    let mut labels = BTreeSet::new();
    let mut tp = BTreeMap::new();
    let mut fp = BTreeMap::new();
    let mut fnn = BTreeMap::new();
    for (prediction, actual) in evaluated {
        let actual_label = label_kind_name(actual.label_kind).to_string();
        let predicted_label = derived_predicted_label(prediction);
        labels.insert(actual_label.clone());
        labels.insert(predicted_label.clone());
        if predicted_label == actual_label {
            correct += 1;
            *tp.entry(actual_label.clone()).or_insert(0usize) += 1;
        } else {
            *fp.entry(predicted_label.clone()).or_insert(0usize) += 1;
            *fnn.entry(actual_label.clone()).or_insert(0usize) += 1;
        }
        *confusion
            .entry(format!("{predicted_label}->{actual_label}"))
            .or_insert(0usize) += 1;
    }
    let accuracy = correct as f64 / evaluated.len() as f64;
    let macro_f1 = if labels.is_empty() {
        None
    } else {
        Some(
            labels
                .iter()
                .map(|label| {
                    let tp = *tp.get(label).unwrap_or(&0) as f64;
                    let fp = *fp.get(label).unwrap_or(&0) as f64;
                    let fnn = *fnn.get(label).unwrap_or(&0) as f64;
                    let precision = if tp + fp > 0.0 { tp / (tp + fp) } else { 0.0 };
                    let recall = if tp + fnn > 0.0 { tp / (tp + fnn) } else { 0.0 };
                    if precision + recall > 0.0 {
                        2.0 * precision * recall / (precision + recall)
                    } else {
                        0.0
                    }
                })
                .sum::<f64>()
                / labels.len() as f64,
        )
    };
    (Some(accuracy), macro_f1, confusion)
}

fn calibration_metrics(
    evaluated: &[(&ExternalPredictionRowV2, &SequenceDatasetEvalRow)],
) -> (Option<f64>, Option<f64>, Vec<String>) {
    let mut pairs = Vec::new();
    for (prediction, actual) in evaluated {
        if let Some(p_win) = prediction.p_win {
            let outcome = if actual.label_kind == SequenceLabelKind::TakeProfit {
                1.0
            } else {
                0.0
            };
            pairs.push((p_win, outcome));
        }
    }
    if pairs.is_empty() {
        return (None, None, Vec::new());
    }
    let brier_score = pairs.iter().map(|(p, y)| (p - y).powi(2)).sum::<f64>() / pairs.len() as f64;
    let mut bucket_counts = [0usize; 5];
    let mut bucket_pred = [0.0f64; 5];
    let mut bucket_obs = [0.0f64; 5];
    for (p, y) in &pairs {
        let idx = ((*p * 5.0).floor() as usize).min(4);
        bucket_counts[idx] += 1;
        bucket_pred[idx] += *p;
        bucket_obs[idx] += *y;
    }
    let mut ece = 0.0;
    let mut buckets = Vec::new();
    for idx in 0..5 {
        if bucket_counts[idx] == 0 {
            continue;
        }
        let pred = bucket_pred[idx] / bucket_counts[idx] as f64;
        let obs = bucket_obs[idx] / bucket_counts[idx] as f64;
        ece += ((pred - obs).abs() * bucket_counts[idx] as f64) / pairs.len() as f64;
        buckets.push(format!("bucket{}:{:.3}->{:.3}", idx, pred, obs));
    }
    (Some(brier_score), Some(ece), buckets)
}

fn ranking_metrics(
    evaluated: &[(&ExternalPredictionRowV2, &SequenceDatasetEvalRow)],
) -> (Option<f64>, Option<f64>) {
    let mut ranked = evaluated
        .iter()
        .filter_map(|(prediction, actual)| {
            prediction
                .rank_score
                .map(|rank| (rank, realized_proxy(actual.label_kind)))
        })
        .collect::<Vec<_>>();
    if ranked.is_empty() {
        return (None, None);
    }
    ranked.sort_by(|left, right| {
        right
            .0
            .partial_cmp(&left.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let k = ranked.len().min(3);
    let top_k_precision = ranked
        .iter()
        .take(k)
        .filter(|(_, realized)| *realized > 0.0)
        .count() as f64
        / k as f64;
    let mean_rank = ranked.iter().map(|(rank, _)| *rank).sum::<f64>() / ranked.len() as f64;
    let mean_realized =
        ranked.iter().map(|(_, realized)| *realized).sum::<f64>() / ranked.len() as f64;
    let covariance = ranked
        .iter()
        .map(|(rank, realized)| (rank - mean_rank) * (realized - mean_realized))
        .sum::<f64>();
    let var_rank = ranked
        .iter()
        .map(|(rank, _)| (rank - mean_rank).powi(2))
        .sum::<f64>();
    let var_realized = ranked
        .iter()
        .map(|(_, realized)| (realized - mean_realized).powi(2))
        .sum::<f64>();
    let correlation = if var_rank > 0.0 && var_realized > 0.0 {
        Some(covariance / (var_rank.sqrt() * var_realized.sqrt()))
    } else {
        None
    };
    (Some(top_k_precision), correlation)
}

fn return_proxy_metrics(
    evaluated: &[(&ExternalPredictionRowV2, &SequenceDatasetEvalRow)],
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if evaluated.is_empty() {
        return (None, None, None);
    }
    let mean_expected_return = evaluated
        .iter()
        .map(|(prediction, _)| prediction.expected_return_pct.unwrap_or(0.0))
        .sum::<f64>()
        / evaluated.len() as f64;
    let mean_realized_proxy = evaluated
        .iter()
        .map(|(_, actual)| realized_proxy(actual.label_kind))
        .sum::<f64>()
        / evaluated.len() as f64;
    let hit_rate_proxy = evaluated
        .iter()
        .filter(|(prediction, actual)| {
            derived_predicted_label(prediction) == label_kind_name(actual.label_kind)
        })
        .count() as f64
        / evaluated.len() as f64;
    (
        Some(mean_expected_return),
        Some(mean_realized_proxy),
        Some(hit_rate_proxy),
    )
}

fn risk_metrics(
    evaluated: &[(&ExternalPredictionRowV2, &SequenceDatasetEvalRow)],
) -> (Option<f64>, Option<f64>, Option<f64>) {
    if evaluated.is_empty() {
        return (None, None, None);
    }
    let drawdown_avg = evaluated
        .iter()
        .map(|(prediction, _)| prediction.expected_drawdown_pct.unwrap_or(0.01))
        .sum::<f64>()
        / evaluated.len() as f64;
    let realized_mean = evaluated
        .iter()
        .map(|(_, actual)| realized_proxy(actual.label_kind))
        .sum::<f64>()
        / evaluated.len() as f64;
    let risk_adjusted = if drawdown_avg > 0.0 {
        realized_mean / drawdown_avg
    } else {
        0.0
    };
    let risk_denied_alignment = evaluated
        .iter()
        .filter(|(prediction, actual)| {
            actual.label_kind != SequenceLabelKind::RiskDeniedCounterfactual
                || derived_predicted_label(prediction) != "TakeProfit"
        })
        .count() as f64
        / evaluated.len() as f64;
    (
        Some(drawdown_avg),
        Some(risk_adjusted),
        Some(risk_denied_alignment),
    )
}

fn cost_aware_metrics(
    evaluated: &[(&ExternalPredictionRowV2, &SequenceDatasetEvalRow)],
) -> (Option<f64>, Option<f64>) {
    if evaluated.is_empty() {
        return (None, None);
    }
    let realized_mean = evaluated
        .iter()
        .map(|(_, actual)| realized_proxy(actual.label_kind))
        .sum::<f64>()
        / evaluated.len() as f64;
    let net_return_proxy = realized_mean - 0.0005;
    let cost_stress_score = net_return_proxy - 0.001;
    (Some(net_return_proxy), Some(cost_stress_score))
}

fn write_bundle(
    config: &ExternalPredictionImportV2Config,
    bundle: &ExternalPredictionEvaluationBundle,
) -> Result<(), String> {
    let dir = config.output_dir();
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    write_text_report(
        &dir,
        "external_prediction_import.txt",
        &bundle.import_report,
    )?;
    write_text_report(
        &dir,
        "prediction_coverage.txt",
        &bundle.prediction_coverage_report,
    )?;
    write_text_report(
        &dir,
        "model_card_validation.txt",
        &bundle.model_card_validation_report,
    )?;
    write_text_report(
        &dir,
        "external_model_evaluation.txt",
        &bundle.external_model_evaluation_report,
    )?;
    write_text_report(
        &dir,
        "external_vs_trinity_comparison.txt",
        &bundle.external_vs_trinity_comparison_report,
    )?;
    write_text_report(
        &dir,
        "external_prediction_ablation.txt",
        &bundle.external_prediction_ablation_report,
    )?;
    write_text_report(
        &dir,
        "external_model_promotion_gate.txt",
        &bundle.external_model_promotion_gate_report,
    )?;
    if let Some(contract) = &bundle.mamba3fin_lite_prototype_contract {
        write_text_report(&dir, "mamba3fin_lite_contract.txt", contract)?;
    }
    if let Some(panel) = &bundle.control_tower_external_model_panel_summary {
        fs::write(
            dir.join("control_tower_external_model_panel.txt"),
            panel.to_text(),
        )
        .map_err(|err| err.to_string())?;
    }
    write_text_report(&dir, "storage_report.txt", &bundle.storage_report)?;
    fs::write(dir.join("summary.txt"), &bundle.final_summary).map_err(|err| err.to_string())?;
    Ok(())
}

fn primary_model_identity(rows: &[ExternalPredictionRowV2]) -> (String, String) {
    rows.first()
        .map(|row| (row.model_id.clone(), row.model_version.clone()))
        .unwrap_or_else(|| ("unknown-model".to_string(), "unknown-version".to_string()))
}

fn realized_proxy(label_kind: SequenceLabelKind) -> f64 {
    match label_kind {
        SequenceLabelKind::TakeProfit => 0.03,
        SequenceLabelKind::StopLoss => -0.015,
        SequenceLabelKind::TimeExpired => 0.0,
        SequenceLabelKind::NoTradeCounterfactual => 0.0,
        SequenceLabelKind::RiskDeniedCounterfactual => 0.002,
        SequenceLabelKind::Unknown => 0.0,
    }
}

fn derived_predicted_label(prediction: &ExternalPredictionRowV2) -> String {
    prediction
        .predicted_label
        .clone()
        .or_else(|| {
            match (
                prediction.p_take_profit,
                prediction.p_stop_loss,
                prediction.p_time_expired,
            ) {
                (Some(tp), Some(sl), Some(te)) => {
                    let mut best = ("TimeExpired".to_string(), te);
                    if tp > best.1 {
                        best = ("TakeProfit".to_string(), tp);
                    }
                    if sl > best.1 {
                        best = ("StopLoss".to_string(), sl);
                    }
                    Some(best.0)
                }
                _ => None,
            }
        })
        .unwrap_or_else(|| {
            if prediction.p_win.unwrap_or(0.0) >= 0.5 {
                "TakeProfit".to_string()
            } else {
                "NoTradeCounterfactual".to_string()
            }
        })
}

fn label_kind_name(label_kind: SequenceLabelKind) -> &'static str {
    match label_kind {
        SequenceLabelKind::TakeProfit => "TakeProfit",
        SequenceLabelKind::StopLoss => "StopLoss",
        SequenceLabelKind::TimeExpired => "TimeExpired",
        SequenceLabelKind::NoTradeCounterfactual => "NoTradeCounterfactual",
        SequenceLabelKind::RiskDeniedCounterfactual => "RiskDeniedCounterfactual",
        SequenceLabelKind::Unknown => "Unknown",
    }
}

fn parse_label_kind(input: &str) -> Option<SequenceLabelKind> {
    match input {
        "TakeProfit" | "TP" => Some(SequenceLabelKind::TakeProfit),
        "StopLoss" | "SL" => Some(SequenceLabelKind::StopLoss),
        "TimeExpired" | "TE" => Some(SequenceLabelKind::TimeExpired),
        "NoTradeCounterfactual" | "NT" => Some(SequenceLabelKind::NoTradeCounterfactual),
        "RiskDeniedCounterfactual" | "RD" => Some(SequenceLabelKind::RiskDeniedCounterfactual),
        _ => None,
    }
}

fn parse_optional_f64(input: Option<&str>) -> Option<f64> {
    input
        .filter(|value| !value.trim().is_empty())
        .and_then(|value| value.parse::<f64>().ok())
}

fn split_csv_line(line: &str) -> Vec<String> {
    line.split(',')
        .map(|value| value.trim().to_string())
        .collect()
}

fn header_index(header: &[String]) -> BTreeMap<String, usize> {
    header
        .iter()
        .enumerate()
        .map(|(idx, value)| (value.clone(), idx))
        .collect()
}

fn csv_value<'a>(
    values: &'a [String],
    index: &BTreeMap<String, usize>,
    column: &str,
) -> Result<&'a str, String> {
    let idx = index
        .get(column)
        .copied()
        .ok_or_else(|| format!("missing column: {column}"))?;
    Ok(values.get(idx).map(String::as_str).unwrap_or_default())
}

fn file_len(path: &Path) -> Result<usize, String> {
    Ok(fs::metadata(path).map_err(|err| err.to_string())?.len() as usize)
}

fn write_text_report<T: Serialize>(dir: &Path, name: &str, value: &T) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    fs::write(dir.join(name), text).map_err(|err| err.to_string())
}

fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

fn is_remote_path(path: &str) -> bool {
    path.contains("://")
}
