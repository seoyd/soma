use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};
use crate::experiment::{NoLookaheadProofStatus, NoLookaheadSequenceProof};
use crate::ui::SequenceDatasetPanel;

fn default_output_root() -> String {
    "target/soma_sequence_dataset_export".to_string()
}

fn default_window_lengths() -> Vec<usize> {
    vec![32, 64]
}

fn default_horizons() -> Vec<usize> {
    vec![4, 8, 16]
}

fn default_max_windows() -> usize {
    1024
}

fn default_max_rows() -> usize {
    2048
}

fn default_max_symbols() -> usize {
    10
}

fn default_max_timeframes() -> usize {
    4
}

fn default_max_bytes() -> usize {
    20_000_000
}

fn default_true() -> bool {
    true
}

fn default_train_ratio() -> f64 {
    0.6
}

fn default_validation_ratio() -> f64 {
    0.2
}

fn default_test_ratio() -> f64 {
    0.2
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceSplitPolicy {
    #[default]
    ChronologicalHoldout,
    WalkForward,
    ExportOnlyNoSplit,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequenceDatasetExportConfig {
    pub export_id: String,
    #[serde(default)]
    pub sequence_readiness_hardening_paths: Vec<String>,
    #[serde(default)]
    pub kis_evidence_closure_paths: Vec<String>,
    #[serde(default)]
    pub outcome_link_depth_closure_paths: Vec<String>,
    #[serde(default)]
    pub kis_canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub feature_schema_lock_paths: Vec<String>,
    #[serde(default)]
    pub label_alignment_audit_paths: Vec<String>,
    #[serde(default)]
    pub no_lookahead_proof_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_window_lengths")]
    pub target_window_lengths: Vec<usize>,
    #[serde(default = "default_horizons")]
    pub target_horizons: Vec<usize>,
    #[serde(default = "default_max_windows")]
    pub max_windows: usize,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_timeframes")]
    pub max_timeframes: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_official_non_crypto: bool,
    #[serde(default = "default_true")]
    pub require_complete_rows: bool,
    #[serde(default = "default_true")]
    pub require_outcome_labels: bool,
    #[serde(default = "default_true")]
    pub require_feature_schema_lock: bool,
    #[serde(default = "default_true")]
    pub require_label_alignment: bool,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default)]
    pub split_policy: SequenceSplitPolicy,
    #[serde(default = "default_train_ratio")]
    pub train_ratio: f64,
    #[serde(default = "default_validation_ratio")]
    pub validation_ratio: f64,
    #[serde(default = "default_test_ratio")]
    pub test_ratio: f64,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for SequenceDatasetExportConfig {
    fn default() -> Self {
        Self {
            export_id: "sprint62-sequence-dataset-export".to_string(),
            sequence_readiness_hardening_paths: Vec::new(),
            kis_evidence_closure_paths: Vec::new(),
            outcome_link_depth_closure_paths: Vec::new(),
            kis_canonical_csv_paths: Vec::new(),
            feature_schema_lock_paths: Vec::new(),
            label_alignment_audit_paths: Vec::new(),
            no_lookahead_proof_paths: Vec::new(),
            output_root: default_output_root(),
            target_window_lengths: default_window_lengths(),
            target_horizons: default_horizons(),
            max_windows: default_max_windows(),
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_timeframes: default_max_timeframes(),
            max_bytes: default_max_bytes(),
            require_official_non_crypto: true,
            require_complete_rows: true,
            require_outcome_labels: true,
            require_feature_schema_lock: true,
            require_label_alignment: true,
            require_no_lookahead_safe: true,
            split_policy: SequenceSplitPolicy::ChronologicalHoldout,
            train_ratio: default_train_ratio(),
            validation_ratio: default_validation_ratio(),
            test_ratio: default_test_ratio(),
            reason_codes: vec![ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly],
        }
    }
}

impl SequenceDatasetExportConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.export_id.trim().is_empty() {
            return Err("sequence dataset export id must not be empty".to_string());
        }
        if self.all_paths().iter().any(|path| path.contains("://"))
            || self.output_root.contains("://")
        {
            return Err("sequence dataset export paths must be local".to_string());
        }
        if self.target_window_lengths.is_empty() || self.target_horizons.is_empty() {
            return Err(
                "sequence dataset export windows and horizons must not be empty".to_string(),
            );
        }
        if self.max_windows == 0 || self.max_windows > 4096 {
            return Err(
                "sequence dataset export max_windows must be between 1 and 4096".to_string(),
            );
        }
        if self.max_rows == 0 || self.max_rows > 8192 {
            return Err("sequence dataset export max_rows must be between 1 and 8192".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > 10 {
            return Err("sequence dataset export max_symbols must be between 1 and 10".to_string());
        }
        if self.max_timeframes == 0 || self.max_timeframes > 8 {
            return Err(
                "sequence dataset export max_timeframes must be between 1 and 8".to_string(),
            );
        }
        if self.max_bytes == 0 || self.max_bytes > 50_000_000 {
            return Err(
                "sequence dataset export max_bytes must be between 1 and 50000000".to_string(),
            );
        }
        let ratio_sum = self.train_ratio + self.validation_ratio + self.test_ratio;
        if !matches!(self.split_policy, SequenceSplitPolicy::ExportOnlyNoSplit)
            && !(0.99..=1.01).contains(&ratio_sum)
        {
            return Err("sequence dataset export split ratios must sum to 1.0".to_string());
        }
        if self.require_feature_schema_lock && self.feature_schema_lock_paths.is_empty() {
            return Err("missing schema lock blocks export".to_string());
        }
        if self.require_label_alignment && self.label_alignment_audit_paths.is_empty() {
            return Err("missing label manifest blocks export".to_string());
        }
        if self.require_no_lookahead_safe && self.no_lookahead_proof_paths.is_empty() {
            return Err("no-lookahead proof is required by default".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.export_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        stable_ordered_strings(
            &self
                .sequence_readiness_hardening_paths
                .iter()
                .chain(self.kis_evidence_closure_paths.iter())
                .chain(self.outcome_link_depth_closure_paths.iter())
                .chain(self.kis_canonical_csv_paths.iter())
                .chain(self.feature_schema_lock_paths.iter())
                .chain(self.label_alignment_audit_paths.iter())
                .chain(self.no_lookahead_proof_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SequenceLabelKind {
    TakeProfit,
    StopLoss,
    TimeExpired,
    NoTradeCounterfactual,
    RiskDeniedCounterfactual,
    Unknown,
}

impl SequenceLabelKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::TakeProfit => "TakeProfit",
            Self::StopLoss => "StopLoss",
            Self::TimeExpired => "TimeExpired",
            Self::NoTradeCounterfactual => "NoTradeCounterfactual",
            Self::RiskDeniedCounterfactual => "RiskDeniedCounterfactual",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SequenceDatasetRow {
    pub sequence_id: String,
    pub symbol: String,
    pub market: String,
    pub timeframe: String,
    pub window_length: usize,
    pub horizon_bars: usize,
    pub window_start_timestamp_ms: i64,
    pub window_end_timestamp_ms: i64,
    pub label_timestamp_ms: i64,
    pub feature_values: Vec<f64>,
    pub label_value: String,
    pub label_kind: SequenceLabelKind,
    pub source_kind: String,
    pub source_class: String,
    pub official_readiness_eligible: bool,
    pub no_lookahead_safe: bool,
    pub feature_schema_hash: String,
    pub label_manifest_hash: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SequenceDatasetExportArtifact {
    pub export_id: String,
    pub dataset_csv_path: String,
    #[serde(default)]
    pub rows: Vec<SequenceDatasetRow>,
    pub row_count: usize,
    pub sequence_count: usize,
    pub symbol_count: usize,
    pub timeframe_count: usize,
    pub horizon_count: usize,
    #[serde(default)]
    pub label_distribution: BTreeMap<String, usize>,
    pub no_lookahead_safe_count: usize,
    pub excluded_count: usize,
    #[serde(default)]
    pub exclusion_reasons: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureSchemaManifest {
    pub schema_id: String,
    #[serde(default)]
    pub feature_names: Vec<String>,
    #[serde(default)]
    pub feature_types: Vec<String>,
    #[serde(default)]
    pub feature_order: Vec<usize>,
    pub feature_order_hash: String,
    #[serde(default)]
    pub source_feature_groups: Vec<String>,
    #[serde(default)]
    pub required_features: Vec<String>,
    #[serde(default)]
    pub optional_features: Vec<String>,
    #[serde(default)]
    pub missing_features: Vec<String>,
    pub feature_normalization_policy: String,
    pub version: String,
    pub frozen: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LabelManifest {
    pub label_manifest_id: String,
    #[serde(default)]
    pub label_kinds: Vec<SequenceLabelKind>,
    #[serde(default)]
    pub horizon_bars: Vec<usize>,
    pub barrier_profile_id: String,
    #[serde(default)]
    pub take_profit_pct: Option<f64>,
    #[serde(default)]
    pub stop_loss_pct: Option<f64>,
    pub cost_bps: f64,
    pub slippage_bps: f64,
    pub tie_break_policy: String,
    pub label_timestamp_policy: String,
    pub no_trade_counterfactual_policy: String,
    pub risk_denied_counterfactual_policy: String,
    pub version: String,
    pub frozen: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceExportManifest {
    pub export_id: String,
    pub dataset_csv_path: String,
    pub feature_schema_manifest_path: String,
    pub label_manifest_path: String,
    #[serde(default)]
    pub split_manifest_path: Option<String>,
    pub no_lookahead_proof_path: String,
    pub storage_report_path: String,
    #[serde(default)]
    pub source_artifacts: Vec<String>,
    #[serde(default)]
    pub provenance_artifacts: Vec<String>,
    #[serde(default)]
    pub preflight_artifacts: Vec<String>,
    pub row_count: usize,
    pub sequence_count: usize,
    pub excluded_count: usize,
    pub fingerprint: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceFoldManifest {
    pub fold_id: String,
    #[serde(default)]
    pub train_sequence_ids: Vec<String>,
    #[serde(default)]
    pub validation_sequence_ids: Vec<String>,
    #[serde(default)]
    pub test_sequence_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceSplitCounts {
    pub train_count: usize,
    pub validation_count: usize,
    pub test_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceSplitManifest {
    pub split_id: String,
    pub policy: SequenceSplitPolicy,
    #[serde(default)]
    pub train_sequence_ids: Vec<String>,
    #[serde(default)]
    pub validation_sequence_ids: Vec<String>,
    #[serde(default)]
    pub test_sequence_ids: Vec<String>,
    #[serde(default)]
    pub fold_manifests: Vec<SequenceFoldManifest>,
    pub split_counts: SequenceSplitCounts,
    #[serde(default)]
    pub split_timestamp_boundaries: Vec<String>,
    pub random_seed_used: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceDatasetQualityStatus {
    ExportReady,
    ExportReadyWithWarnings,
    InsufficientRows,
    InsufficientSymbols,
    InsufficientLabels,
    FeatureSchemaMismatch,
    LabelManifestMismatch,
    NoLookaheadViolation,
    StorageBudgetExceeded,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SequenceDatasetQualityReport {
    pub export_id: String,
    pub row_count: usize,
    pub sequence_count: usize,
    pub symbol_count: usize,
    #[serde(default)]
    pub label_distribution: BTreeMap<String, usize>,
    pub no_lookahead_safe_ratio: f64,
    pub excluded_count: usize,
    pub missing_feature_count: usize,
    pub storage_bytes: usize,
    pub quality_status: SequenceDatasetQualityStatus,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequenceDatasetDriftGuardConfig {
    pub drift_id: String,
    #[serde(default)]
    pub baseline_manifest_path: Option<String>,
    pub current_manifest_path: String,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub allow_schema_version_change: bool,
    #[serde(default)]
    pub allow_label_manifest_version_change: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for SequenceDatasetDriftGuardConfig {
    fn default() -> Self {
        Self {
            drift_id: "sprint62-sequence-drift".to_string(),
            baseline_manifest_path: None,
            current_manifest_path: String::new(),
            output_root: default_output_root(),
            allow_schema_version_change: false,
            allow_label_manifest_version_change: false,
            reason_codes: vec![ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly],
        }
    }
}

impl SequenceDatasetDriftGuardConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.drift_id.trim().is_empty() {
            return Err("sequence dataset drift id must not be empty".to_string());
        }
        if self
            .baseline_manifest_path
            .iter()
            .chain(std::iter::once(&self.current_manifest_path))
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("sequence dataset drift paths must be local".to_string());
        }
        if self.current_manifest_path.trim().is_empty() {
            return Err(
                "sequence dataset drift current_manifest_path must not be empty".to_string(),
            );
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.drift_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceDatasetDriftStatus {
    NoDrift,
    ExpectedDrift,
    UnexpectedFeatureSchemaDrift,
    UnexpectedLabelDrift,
    SourceArtifactChanged,
    MissingBaseline,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SequenceDatasetDriftGuardReport {
    pub drift_id: String,
    pub feature_schema_changed: bool,
    pub label_manifest_changed: bool,
    pub source_artifacts_changed: bool,
    pub row_count_delta: isize,
    #[serde(default)]
    pub label_distribution_delta: BTreeMap<String, isize>,
    pub drift_status: SequenceDatasetDriftStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceReplayStatus {
    ReplayStable,
    ReplayMismatch,
    MissingArtifact,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceDatasetReplayCheck {
    pub replay_id: String,
    pub export_config_path: String,
    pub first_export_manifest_path: String,
    pub second_export_manifest_path: String,
    pub fingerprints_match: bool,
    pub row_order_match: bool,
    pub split_match: bool,
    pub replay_status: SequenceReplayStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalModelBridgeReadinessStatus {
    ReadyForPredictionCsvImport,
    NeedDatasetExport,
    NeedPredictionSchema,
    NeedModelCardTemplate,
    NeedEvaluationGate,
    NeedRiskIntegration,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalPredictionSchema {
    pub prediction_id: String,
    pub sequence_id: String,
    pub symbol: String,
    pub timestamp_ms: i64,
    #[serde(default)]
    pub p_win: Option<String>,
    #[serde(default)]
    pub p_stop: Option<String>,
    #[serde(default)]
    pub expected_return: Option<String>,
    #[serde(default)]
    pub expected_drawdown: Option<String>,
    #[serde(default)]
    pub confidence: Option<String>,
    pub model_version: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalModelBridgeReadinessReport {
    pub dataset_export_ready: bool,
    pub prediction_schema_ready: bool,
    pub model_card_template_ready: bool,
    pub evaluation_gate_ready: bool,
    pub risk_integration_ready: bool,
    pub readiness_status: ExternalModelBridgeReadinessStatus,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub example_prediction_schema: Vec<ExternalPredictionSchema>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3FinExternalPrototypeGateStatus {
    PlanningReady,
    BlockedBySequenceDataset,
    BlockedByEvidenceDepth,
    BlockedByNoLookahead,
    BlockedByStorage,
    BlockedByExternalBridge,
    RuntimeDeferred,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3FinExternalPrototypeRecommendation {
    PlanExternalPrototypeOnly,
    ExportMoreSequenceRows,
    ImproveEvidenceDepth,
    HoldMamba3Deferred,
    KeepBaselineAndTrinity,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mamba3FinExternalPrototypeGateReport {
    pub sequence_export_ready: bool,
    pub external_bridge_ready: bool,
    pub risk_integration_ready: bool,
    pub control_tower_visibility_ready: bool,
    pub rust_runtime_allowed: bool,
    pub training_allowed: bool,
    pub live_inference_allowed: bool,
    pub gate_status: Mamba3FinExternalPrototypeGateStatus,
    pub final_recommendation: Mamba3FinExternalPrototypeRecommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceDatasetStorageReport {
    pub estimated_windows: usize,
    pub row_count: usize,
    pub storage_bytes: usize,
    pub max_bytes: usize,
    pub within_budget: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SequenceDatasetExportBundle {
    pub sequence_dataset_export_artifact: SequenceDatasetExportArtifact,
    pub feature_schema_manifest: FeatureSchemaManifest,
    pub label_manifest: LabelManifest,
    pub sequence_export_manifest: SequenceExportManifest,
    #[serde(default)]
    pub split_manifest: Option<SequenceSplitManifest>,
    pub quality_report: SequenceDatasetQualityReport,
    pub no_lookahead_proof: NoLookaheadSequenceProof,
    pub storage_report: SequenceDatasetStorageReport,
    #[serde(default)]
    pub drift_guard_report: Option<SequenceDatasetDriftGuardReport>,
    #[serde(default)]
    pub replay_check: Option<SequenceDatasetReplayCheck>,
    pub external_bridge_readiness_report: ExternalModelBridgeReadinessReport,
    pub mamba3fin_external_prototype_gate_report: Mamba3FinExternalPrototypeGateReport,
    #[serde(default)]
    pub control_tower_sequence_panel_summary: Option<SequenceDatasetPanel>,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SequenceDatasetExportRunner;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct SourceRowSeed {
    pub symbol: String,
    pub market: String,
    pub timeframe: String,
    pub window_length: usize,
    pub horizon_bars: usize,
    pub window_start_timestamp_ms: i64,
    pub window_end_timestamp_ms: i64,
    pub label_timestamp_ms: i64,
    #[serde(default)]
    pub features: BTreeMap<String, f64>,
    pub label_kind: SequenceLabelKind,
    pub label_value: String,
    pub source_kind: String,
    pub source_class: String,
    #[serde(default = "default_true")]
    pub official_readiness_eligible: bool,
    #[serde(default = "default_true")]
    pub complete_row: bool,
    #[serde(default = "default_true")]
    pub no_lookahead_safe: bool,
}

impl SequenceDatasetExportArtifact {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl SequenceDatasetExportBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl SequenceDatasetQualityReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl SequenceDatasetDriftGuardReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl SequenceDatasetReplayCheck {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl ExternalModelBridgeReadinessReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl Mamba3FinExternalPrototypeGateReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl SequenceDatasetExportRunner {
    pub fn run(
        &self,
        config: &SequenceDatasetExportConfig,
    ) -> Result<SequenceDatasetExportBundle, String> {
        config.validate()?;
        let feature_schema_manifest = build_feature_schema_manifest(config)?;
        let label_manifest = build_label_manifest(config)?;
        let seeds = load_source_row_seeds(&config.kis_canonical_csv_paths)?;
        let mut exclusion_reasons = Vec::new();
        let exported_rows = build_export_rows(
            config,
            &feature_schema_manifest,
            &label_manifest,
            &seeds,
            &mut exclusion_reasons,
        );
        let storage_report =
            build_storage_report(config, exported_rows.len(), &feature_schema_manifest);
        let no_lookahead_proof = rerun_no_lookahead_proof(config, &exported_rows)?;
        let dataset_csv_path = write_dataset_csv(config, &feature_schema_manifest, &exported_rows)?;
        let sequence_ids = exported_rows
            .iter()
            .map(|row| row.sequence_id.clone())
            .collect::<Vec<_>>();
        let split_manifest = build_split_manifest(config, &exported_rows, &sequence_ids);
        let artifact = SequenceDatasetExportArtifact {
            export_id: config.export_id.clone(),
            dataset_csv_path: dataset_csv_path.display().to_string(),
            rows: exported_rows.clone(),
            row_count: exported_rows.len(),
            sequence_count: exported_rows.len(),
            symbol_count: exported_rows
                .iter()
                .map(|row| row.symbol.clone())
                .collect::<BTreeSet<_>>()
                .len(),
            timeframe_count: exported_rows
                .iter()
                .map(|row| row.timeframe.clone())
                .collect::<BTreeSet<_>>()
                .len(),
            horizon_count: exported_rows
                .iter()
                .map(|row| row.horizon_bars)
                .collect::<BTreeSet<_>>()
                .len(),
            label_distribution: label_distribution(&exported_rows),
            no_lookahead_safe_count: exported_rows
                .iter()
                .filter(|row| row.no_lookahead_safe)
                .count(),
            excluded_count: seeds.len().saturating_sub(exported_rows.len()),
            exclusion_reasons: stable_ordered_strings(&exclusion_reasons),
            reason_codes: stable_reason_codes(&[ReasonCode::SequenceDatasetExportArtifactBuilt]),
        };
        let quality_report = build_quality_report(
            config,
            &artifact,
            &feature_schema_manifest,
            &label_manifest,
            &no_lookahead_proof,
            &storage_report,
        );
        let sequence_export_manifest = build_export_manifest(
            config,
            &artifact,
            &feature_schema_manifest,
            &label_manifest,
            split_manifest.as_ref(),
            &no_lookahead_proof,
            &storage_report,
        );
        let external_bridge_readiness_report =
            build_external_bridge_readiness(&artifact, &quality_report);
        let panel_summary = Some(build_sequence_dataset_panel(
            &artifact,
            &feature_schema_manifest,
            &label_manifest,
            split_manifest.as_ref(),
            &no_lookahead_proof,
            &quality_report,
            &external_bridge_readiness_report,
        ));
        let mamba3fin_external_prototype_gate_report = build_mamba_gate(
            &quality_report,
            &no_lookahead_proof,
            &storage_report,
            &external_bridge_readiness_report,
            panel_summary.is_some(),
        );
        let final_summary = [
            format!("sequence_export_status={:?}", quality_report.quality_status),
            format!(
                "external_bridge_status={:?}",
                external_bridge_readiness_report.readiness_status
            ),
            format!(
                "mamba_gate_status={:?}",
                mamba3fin_external_prototype_gate_report.gate_status
            ),
            "runtime_status=HoldMamba3RuntimeDeferred".to_string(),
        ]
        .join("\n");
        let bundle = SequenceDatasetExportBundle {
            sequence_dataset_export_artifact: artifact,
            feature_schema_manifest,
            label_manifest,
            sequence_export_manifest,
            split_manifest,
            quality_report,
            no_lookahead_proof,
            storage_report,
            drift_guard_report: None,
            replay_check: None,
            external_bridge_readiness_report,
            mamba3fin_external_prototype_gate_report,
            control_tower_sequence_panel_summary: panel_summary,
            final_summary,
            reason_codes: stable_reason_codes(&[
                ReasonCode::SequenceDatasetExportRunnerBuilt,
                ReasonCode::SequenceDatasetExportBundleBuilt,
            ]),
        };
        write_bundle_outputs(config, &bundle)?;
        Ok(bundle)
    }

    pub fn run_quality(
        &self,
        config: &SequenceDatasetExportConfig,
    ) -> Result<SequenceDatasetQualityReport, String> {
        Ok(self.run(config)?.quality_report)
    }

    pub fn run_external_bridge_readiness(
        &self,
        config: &SequenceDatasetExportConfig,
    ) -> Result<ExternalModelBridgeReadinessReport, String> {
        Ok(self.run(config)?.external_bridge_readiness_report)
    }

    pub fn run_mamba3fin_prototype_gate(
        &self,
        config: &SequenceDatasetExportConfig,
    ) -> Result<Mamba3FinExternalPrototypeGateReport, String> {
        Ok(self.run(config)?.mamba3fin_external_prototype_gate_report)
    }

    pub fn run_replay_check(
        &self,
        config_path: &Path,
    ) -> Result<SequenceDatasetReplayCheck, String> {
        let config = SequenceDatasetExportConfig::from_toml_path(config_path)?;
        let first = self.run(&config)?;
        let second = self.run(&config)?;
        let row_order_match = first
            .sequence_dataset_export_artifact
            .rows
            .iter()
            .map(|row| &row.sequence_id)
            .eq(second
                .sequence_dataset_export_artifact
                .rows
                .iter()
                .map(|row| &row.sequence_id));
        let split_match = first.split_manifest == second.split_manifest;
        let fingerprints_match = first.sequence_export_manifest.fingerprint
            == second.sequence_export_manifest.fingerprint;
        let replay_status = if fingerprints_match && row_order_match && split_match {
            SequenceReplayStatus::ReplayStable
        } else {
            SequenceReplayStatus::ReplayMismatch
        };
        let report = SequenceDatasetReplayCheck {
            replay_id: config.export_id.clone(),
            export_config_path: config_path.display().to_string(),
            first_export_manifest_path: config
                .artifact_dir()
                .join("sequence_export_manifest.json")
                .display()
                .to_string(),
            second_export_manifest_path: config
                .artifact_dir()
                .join("sequence_export_manifest.json")
                .display()
                .to_string(),
            fingerprints_match,
            row_order_match,
            split_match,
            replay_status,
            reason_codes: stable_reason_codes(&[ReasonCode::SequenceDatasetReplayCheckBuilt]),
        };
        write_json_text(&config.artifact_dir(), "sequence_replay_check", &report)?;
        Ok(report)
    }

    pub fn run_drift_guard(
        &self,
        config: &SequenceDatasetDriftGuardConfig,
    ) -> Result<SequenceDatasetDriftGuardReport, String> {
        config.validate()?;
        let current = load_manifest_summary(Path::new(&config.current_manifest_path))?;
        let Some(baseline_path) = &config.baseline_manifest_path else {
            let report = SequenceDatasetDriftGuardReport {
                drift_id: config.drift_id.clone(),
                feature_schema_changed: false,
                label_manifest_changed: false,
                source_artifacts_changed: false,
                row_count_delta: 0,
                label_distribution_delta: BTreeMap::new(),
                drift_status: SequenceDatasetDriftStatus::MissingBaseline,
                reason_codes: stable_reason_codes(&[ReasonCode::SequenceDatasetDriftGuardBuilt]),
            };
            write_json_text(&config.artifact_dir(), "sequence_drift_guard", &report)?;
            return Ok(report);
        };
        let baseline = load_manifest_summary(Path::new(baseline_path))?;
        let feature_schema_changed =
            baseline.feature_schema_manifest_path != current.feature_schema_manifest_path;
        let label_manifest_changed = baseline.label_manifest_path != current.label_manifest_path;
        let source_artifacts_changed = baseline.source_artifacts != current.source_artifacts;
        let row_count_delta = current.row_count as isize - baseline.row_count as isize;
        let label_distribution_delta =
            diff_label_distribution(&baseline.label_distribution, &current.label_distribution);
        let drift_status = if feature_schema_changed && !config.allow_schema_version_change {
            SequenceDatasetDriftStatus::UnexpectedFeatureSchemaDrift
        } else if label_manifest_changed && !config.allow_label_manifest_version_change {
            SequenceDatasetDriftStatus::UnexpectedLabelDrift
        } else if source_artifacts_changed {
            SequenceDatasetDriftStatus::SourceArtifactChanged
        } else if feature_schema_changed || label_manifest_changed {
            SequenceDatasetDriftStatus::ExpectedDrift
        } else {
            SequenceDatasetDriftStatus::NoDrift
        };
        let report = SequenceDatasetDriftGuardReport {
            drift_id: config.drift_id.clone(),
            feature_schema_changed,
            label_manifest_changed,
            source_artifacts_changed,
            row_count_delta,
            label_distribution_delta,
            drift_status,
            reason_codes: stable_reason_codes(&[ReasonCode::SequenceDatasetDriftGuardBuilt]),
        };
        write_json_text(&config.artifact_dir(), "sequence_drift_guard", &report)?;
        Ok(report)
    }
}

#[derive(Clone, Debug)]
struct ManifestSummary {
    feature_schema_manifest_path: String,
    label_manifest_path: String,
    source_artifacts: Vec<String>,
    row_count: usize,
    label_distribution: BTreeMap<String, usize>,
}

fn build_feature_schema_manifest(
    config: &SequenceDatasetExportConfig,
) -> Result<FeatureSchemaManifest, String> {
    let values = load_json_values(&config.feature_schema_lock_paths)?;
    let feature_names = first_string_array(&values, &["feature_names"]);
    let required_features = first_string_array(&values, &["required_features", "feature_names"]);
    let optional_features = first_string_array(&values, &["optional_features"]);
    let missing_features = first_string_array(&values, &["missing_features"]);
    let feature_types = first_string_array(&values, &["feature_types"]);
    let source_feature_groups = first_string_array(&values, &["source_feature_groups"]);
    let feature_normalization_policy = first_string(
        &values,
        &["feature_normalization_policy", "normalization_policy"],
    )
    .unwrap_or_else(|| "explicit-zscore".to_string());
    let version = first_string(&values, &["version"]).unwrap_or_else(|| "v1".to_string());
    let frozen = all_bool(&values, &["frozen"]);
    let feature_order = (0..feature_names.len()).collect::<Vec<_>>();
    let feature_order_hash = stable_hash_string(&feature_names.join("|"));
    let manifest = FeatureSchemaManifest {
        schema_id: config.export_id.clone(),
        feature_names,
        feature_types,
        feature_order,
        feature_order_hash,
        source_feature_groups,
        required_features,
        optional_features,
        missing_features,
        feature_normalization_policy,
        version,
        frozen,
        reason_codes: stable_reason_codes(&[ReasonCode::FeatureSchemaManifestBuilt]),
    };
    if config.require_feature_schema_lock
        && (!manifest.frozen || !manifest.missing_features.is_empty())
    {
        return Err(
            "feature schema manifest is not frozen or has missing required features".to_string(),
        );
    }
    Ok(manifest)
}

fn build_label_manifest(config: &SequenceDatasetExportConfig) -> Result<LabelManifest, String> {
    let values = load_json_values(&config.label_alignment_audit_paths)?;
    let label_kinds = first_label_kind_array(&values, &["label_kinds"]);
    let horizon_bars = first_numeric_array(&values, &["horizon_bars", "horizons"]);
    let barrier_profile_id = first_string(&values, &["barrier_profile_id"])
        .unwrap_or_else(|| "default-barrier".to_string());
    let take_profit_pct = first_f64(&values, &["take_profit_pct"]);
    let stop_loss_pct = first_f64(&values, &["stop_loss_pct"]);
    let cost_bps = first_f64(&values, &["cost_bps"]).unwrap_or(3.0);
    let slippage_bps = first_f64(&values, &["slippage_bps"]).unwrap_or(2.0);
    let tie_break_policy = first_string(&values, &["tie_break_policy"])
        .unwrap_or_else(|| "deterministic-priority".to_string());
    let label_timestamp_policy = first_string(&values, &["label_timestamp_policy"])
        .unwrap_or_else(|| "strictly-after-window-end".to_string());
    let no_trade_counterfactual_policy = first_string(&values, &["no_trade_counterfactual_policy"])
        .unwrap_or_else(|| "exclude-live-unknown".to_string());
    let risk_denied_counterfactual_policy =
        first_string(&values, &["risk_denied_counterfactual_policy"])
            .unwrap_or_else(|| "risk-governor-final".to_string());
    let version = first_string(&values, &["version"]).unwrap_or_else(|| "v1".to_string());
    let frozen = all_bool(&values, &["frozen"]);
    let manifest = LabelManifest {
        label_manifest_id: config.export_id.clone(),
        label_kinds: if label_kinds.is_empty() {
            vec![
                SequenceLabelKind::TakeProfit,
                SequenceLabelKind::StopLoss,
                SequenceLabelKind::TimeExpired,
            ]
        } else {
            label_kinds
        },
        horizon_bars,
        barrier_profile_id,
        take_profit_pct,
        stop_loss_pct,
        cost_bps,
        slippage_bps,
        tie_break_policy,
        label_timestamp_policy,
        no_trade_counterfactual_policy,
        risk_denied_counterfactual_policy,
        version,
        frozen,
        reason_codes: stable_reason_codes(&[ReasonCode::LabelManifestBuilt]),
    };
    if config.require_label_alignment && (!manifest.frozen || manifest.horizon_bars.is_empty()) {
        return Err("label manifest is not frozen or has no horizons".to_string());
    }
    Ok(manifest)
}

fn build_export_rows(
    config: &SequenceDatasetExportConfig,
    feature_schema: &FeatureSchemaManifest,
    label_manifest: &LabelManifest,
    seeds: &[SourceRowSeed],
    exclusion_reasons: &mut Vec<String>,
) -> Vec<SequenceDatasetRow> {
    let allowed_window_lengths = config
        .target_window_lengths
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let allowed_horizons = config
        .target_horizons
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    let mut used_symbols = BTreeSet::new();
    let mut used_timeframes = BTreeSet::new();
    for seed in seeds {
        if rows.len() >= config.max_rows || rows.len() >= config.max_windows {
            exclusion_reasons.push("row budget reached".to_string());
            break;
        }
        if config.require_official_non_crypto && !seed.official_readiness_eligible {
            exclusion_reasons.push("non-official row excluded".to_string());
            continue;
        }
        if config.require_complete_rows && !seed.complete_row {
            exclusion_reasons.push("incomplete row excluded".to_string());
            continue;
        }
        if config.require_no_lookahead_safe && !seed.no_lookahead_safe {
            exclusion_reasons.push("no-lookahead unsafe row excluded".to_string());
            continue;
        }
        if config.require_outcome_labels && matches!(seed.label_kind, SequenceLabelKind::Unknown) {
            exclusion_reasons.push("unknown label excluded".to_string());
            continue;
        }
        if !allowed_window_lengths.contains(&seed.window_length)
            || !allowed_horizons.contains(&seed.horizon_bars)
        {
            exclusion_reasons.push("window/horizon excluded".to_string());
            continue;
        }
        if seed.label_timestamp_ms <= seed.window_end_timestamp_ms {
            exclusion_reasons.push("label timestamp must be after feature window".to_string());
            continue;
        }
        if !label_manifest.horizon_bars.is_empty()
            && !label_manifest.horizon_bars.contains(&seed.horizon_bars)
        {
            exclusion_reasons.push("label horizon mismatch".to_string());
            continue;
        }
        let missing_required = feature_schema
            .required_features
            .iter()
            .any(|name| !seed.features.contains_key(name));
        if missing_required {
            exclusion_reasons.push("missing required feature".to_string());
            continue;
        }
        used_symbols.insert(seed.symbol.clone());
        used_timeframes.insert(seed.timeframe.clone());
        if used_symbols.len() > config.max_symbols || used_timeframes.len() > config.max_timeframes
        {
            exclusion_reasons.push("symbol/timeframe bound reached".to_string());
            break;
        }
        let feature_values = feature_schema
            .feature_names
            .iter()
            .map(|name| seed.features.get(name).copied().unwrap_or_default())
            .collect::<Vec<_>>();
        let sequence_id = stable_hash_string(&format!(
            "{}|{}|{}|{}|{}|{}",
            seed.symbol,
            seed.timeframe,
            seed.window_length,
            seed.horizon_bars,
            seed.window_start_timestamp_ms,
            seed.label_timestamp_ms
        ));
        rows.push(SequenceDatasetRow {
            sequence_id,
            symbol: seed.symbol.clone(),
            market: seed.market.clone(),
            timeframe: seed.timeframe.clone(),
            window_length: seed.window_length,
            horizon_bars: seed.horizon_bars,
            window_start_timestamp_ms: seed.window_start_timestamp_ms,
            window_end_timestamp_ms: seed.window_end_timestamp_ms,
            label_timestamp_ms: seed.label_timestamp_ms,
            feature_values,
            label_value: seed.label_value.clone(),
            label_kind: seed.label_kind,
            source_kind: seed.source_kind.clone(),
            source_class: seed.source_class.clone(),
            official_readiness_eligible: seed.official_readiness_eligible,
            no_lookahead_safe: seed.no_lookahead_safe,
            feature_schema_hash: feature_schema.feature_order_hash.clone(),
            label_manifest_hash: stable_hash_string(&format!(
                "{}|{:?}|{:?}|{}",
                label_manifest.label_manifest_id,
                label_manifest.label_kinds,
                label_manifest.horizon_bars,
                label_manifest.version
            )),
            reason_codes: stable_reason_codes(&[ReasonCode::SequenceDatasetRowBuilt]),
        });
    }
    rows.sort_by(|left, right| {
        left.label_timestamp_ms
            .cmp(&right.label_timestamp_ms)
            .then_with(|| left.sequence_id.cmp(&right.sequence_id))
    });
    rows
}

fn build_storage_report(
    config: &SequenceDatasetExportConfig,
    row_count: usize,
    feature_schema: &FeatureSchemaManifest,
) -> SequenceDatasetStorageReport {
    let storage_bytes = row_count
        .saturating_mul(feature_schema.feature_names.len().max(1))
        .saturating_mul(16);
    SequenceDatasetStorageReport {
        estimated_windows: row_count,
        row_count,
        storage_bytes,
        max_bytes: config.max_bytes,
        within_budget: storage_bytes <= config.max_bytes,
        reason_codes: stable_reason_codes(&[ReasonCode::SequenceDatasetStorageBuilt]),
    }
}

fn rerun_no_lookahead_proof(
    config: &SequenceDatasetExportConfig,
    rows: &[SequenceDatasetRow],
) -> Result<NoLookaheadSequenceProof, String> {
    let values = load_json_values(&config.no_lookahead_proof_paths)?;
    let checked_windows = if rows.is_empty() {
        max_usize(&values, &["checked_windows"])
    } else {
        rows.len()
    };
    let failed_windows = max_usize(&values, &["failed_windows"])
        + rows.iter().filter(|row| !row.no_lookahead_safe).count();
    let passed_windows = checked_windows.saturating_sub(failed_windows);
    let proof_status = if checked_windows == 0 {
        NoLookaheadProofStatus::InsufficientWindows
    } else if failed_windows > 0 {
        NoLookaheadProofStatus::NoLookaheadViolation
    } else {
        NoLookaheadProofStatus::NoLookaheadSafe
    };
    Ok(NoLookaheadSequenceProof {
        proof_id: config.export_id.clone(),
        checked_windows,
        passed_windows,
        failed_windows,
        violation_examples: collect_string_items(&values, &["violation_examples"]),
        proof_status,
        reason_codes: stable_reason_codes(&[ReasonCode::NoLookaheadSequenceExportBuilt]),
    })
}

fn build_split_manifest(
    config: &SequenceDatasetExportConfig,
    rows: &[SequenceDatasetRow],
    sequence_ids: &[String],
) -> Option<SequenceSplitManifest> {
    if rows.is_empty() {
        return Some(SequenceSplitManifest {
            split_id: config.export_id.clone(),
            policy: SequenceSplitPolicy::ExportOnlyNoSplit,
            train_sequence_ids: Vec::new(),
            validation_sequence_ids: Vec::new(),
            test_sequence_ids: Vec::new(),
            fold_manifests: Vec::new(),
            split_counts: SequenceSplitCounts {
                train_count: 0,
                validation_count: 0,
                test_count: 0,
            },
            split_timestamp_boundaries: Vec::new(),
            random_seed_used: false,
            reason_codes: stable_reason_codes(&[
                ReasonCode::SequenceSplitManifestBuilt,
                ReasonCode::SequenceExportNoSplit,
            ]),
        });
    }
    let mut sorted = rows.to_vec();
    sorted.sort_by(|left, right| left.label_timestamp_ms.cmp(&right.label_timestamp_ms));
    match config.split_policy {
        SequenceSplitPolicy::ExportOnlyNoSplit => Some(SequenceSplitManifest {
            split_id: config.export_id.clone(),
            policy: SequenceSplitPolicy::ExportOnlyNoSplit,
            train_sequence_ids: Vec::new(),
            validation_sequence_ids: Vec::new(),
            test_sequence_ids: Vec::new(),
            fold_manifests: Vec::new(),
            split_counts: SequenceSplitCounts {
                train_count: 0,
                validation_count: 0,
                test_count: 0,
            },
            split_timestamp_boundaries: vec!["export-only-no-split".to_string()],
            random_seed_used: false,
            reason_codes: stable_reason_codes(&[
                ReasonCode::SequenceSplitManifestBuilt,
                ReasonCode::SequenceExportNoSplit,
            ]),
        }),
        SequenceSplitPolicy::ChronologicalHoldout => {
            let total = sequence_ids.len();
            let train_end = ((total as f64) * config.train_ratio).floor() as usize;
            let validation_end =
                train_end + ((total as f64) * config.validation_ratio).floor() as usize;
            let train_sequence_ids = sequence_ids[..train_end.min(total)].to_vec();
            let validation_sequence_ids =
                sequence_ids[train_end.min(total)..validation_end.min(total)].to_vec();
            let test_sequence_ids = sequence_ids[validation_end.min(total)..].to_vec();
            Some(SequenceSplitManifest {
                split_id: config.export_id.clone(),
                policy: SequenceSplitPolicy::ChronologicalHoldout,
                train_sequence_ids: train_sequence_ids.clone(),
                validation_sequence_ids: validation_sequence_ids.clone(),
                test_sequence_ids: test_sequence_ids.clone(),
                fold_manifests: Vec::new(),
                split_counts: SequenceSplitCounts {
                    train_count: train_sequence_ids.len(),
                    validation_count: validation_sequence_ids.len(),
                    test_count: test_sequence_ids.len(),
                },
                split_timestamp_boundaries: vec![
                    format!(
                        "train_end={}",
                        sorted[train_end.min(total.saturating_sub(1))].label_timestamp_ms
                    ),
                    format!(
                        "validation_end={}",
                        sorted[validation_end.min(total.saturating_sub(1))].label_timestamp_ms
                    ),
                ],
                random_seed_used: false,
                reason_codes: stable_reason_codes(&[ReasonCode::SequenceSplitManifestBuilt]),
            })
        }
        SequenceSplitPolicy::WalkForward => {
            let total = sequence_ids.len();
            let fold_size = (total / 3).max(1);
            let mut fold_manifests = Vec::new();
            let mut start = 0usize;
            let mut fold_index = 0usize;
            while start + fold_size < total && fold_manifests.len() < 2 {
                let train = sequence_ids[..start + fold_size].to_vec();
                let validation =
                    sequence_ids[start + fold_size..(start + fold_size * 2).min(total)].to_vec();
                let test = sequence_ids
                    [(start + fold_size * 2).min(total)..(start + fold_size * 3).min(total)]
                    .to_vec();
                fold_manifests.push(SequenceFoldManifest {
                    fold_id: format!("fold-{}", fold_index + 1),
                    train_sequence_ids: train.clone(),
                    validation_sequence_ids: validation.clone(),
                    test_sequence_ids: test.clone(),
                });
                fold_index += 1;
                start += fold_size;
            }
            let last = fold_manifests
                .last()
                .cloned()
                .unwrap_or(SequenceFoldManifest {
                    fold_id: "fold-1".to_string(),
                    train_sequence_ids: sequence_ids.to_vec(),
                    validation_sequence_ids: Vec::new(),
                    test_sequence_ids: Vec::new(),
                });
            Some(SequenceSplitManifest {
                split_id: config.export_id.clone(),
                policy: SequenceSplitPolicy::WalkForward,
                train_sequence_ids: last.train_sequence_ids.clone(),
                validation_sequence_ids: last.validation_sequence_ids.clone(),
                test_sequence_ids: last.test_sequence_ids.clone(),
                fold_manifests,
                split_counts: SequenceSplitCounts {
                    train_count: last.train_sequence_ids.len(),
                    validation_count: last.validation_sequence_ids.len(),
                    test_count: last.test_sequence_ids.len(),
                },
                split_timestamp_boundaries: vec!["walk-forward".to_string()],
                random_seed_used: false,
                reason_codes: stable_reason_codes(&[ReasonCode::SequenceSplitManifestBuilt]),
            })
        }
    }
}

fn build_quality_report(
    config: &SequenceDatasetExportConfig,
    artifact: &SequenceDatasetExportArtifact,
    feature_schema: &FeatureSchemaManifest,
    label_manifest: &LabelManifest,
    no_lookahead_proof: &NoLookaheadSequenceProof,
    storage_report: &SequenceDatasetStorageReport,
) -> SequenceDatasetQualityReport {
    let symbol_count = artifact.symbol_count;
    let no_lookahead_safe_ratio = if artifact.row_count == 0 {
        0.0
    } else {
        artifact.no_lookahead_safe_count as f64 / artifact.row_count as f64
    };
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let quality_status = if artifact.row_count == 0 {
        blockers.push("no exportable rows".to_string());
        SequenceDatasetQualityStatus::InsufficientRows
    } else if symbol_count < 2 {
        blockers.push("bounded export needs at least two symbols in the example".to_string());
        SequenceDatasetQualityStatus::InsufficientSymbols
    } else if artifact.label_distribution.is_empty() {
        blockers.push("no outcome labels exported".to_string());
        SequenceDatasetQualityStatus::InsufficientLabels
    } else if !feature_schema.frozen || !feature_schema.missing_features.is_empty() {
        blockers.push("feature schema mismatch".to_string());
        SequenceDatasetQualityStatus::FeatureSchemaMismatch
    } else if !label_manifest.frozen || label_manifest.horizon_bars.is_empty() {
        blockers.push("label manifest mismatch".to_string());
        SequenceDatasetQualityStatus::LabelManifestMismatch
    } else if !matches!(
        no_lookahead_proof.proof_status,
        NoLookaheadProofStatus::NoLookaheadSafe
    ) {
        blockers.push("no-lookahead proof failed on exported rows".to_string());
        SequenceDatasetQualityStatus::NoLookaheadViolation
    } else if !storage_report.within_budget {
        blockers.push("storage budget exceeded".to_string());
        SequenceDatasetQualityStatus::StorageBudgetExceeded
    } else if artifact.excluded_count > 0 {
        warnings.push("bounded export excluded some rows".to_string());
        SequenceDatasetQualityStatus::ExportReadyWithWarnings
    } else {
        SequenceDatasetQualityStatus::ExportReady
    };
    SequenceDatasetQualityReport {
        export_id: config.export_id.clone(),
        row_count: artifact.row_count,
        sequence_count: artifact.sequence_count,
        symbol_count,
        label_distribution: artifact.label_distribution.clone(),
        no_lookahead_safe_ratio,
        excluded_count: artifact.excluded_count,
        missing_feature_count: feature_schema.missing_features.len(),
        storage_bytes: storage_report.storage_bytes,
        quality_status,
        blockers: stable_ordered_strings(&blockers),
        warnings: stable_ordered_strings(&warnings),
        reason_codes: stable_reason_codes(&[ReasonCode::SequenceDatasetQualityBuilt]),
    }
}

fn build_export_manifest(
    config: &SequenceDatasetExportConfig,
    artifact: &SequenceDatasetExportArtifact,
    feature_schema: &FeatureSchemaManifest,
    label_manifest: &LabelManifest,
    split_manifest: Option<&SequenceSplitManifest>,
    _no_lookahead_proof: &NoLookaheadSequenceProof,
    _storage_report: &SequenceDatasetStorageReport,
) -> SequenceExportManifest {
    let feature_schema_manifest_path = config.artifact_dir().join("feature_schema.json");
    let label_manifest_path = config.artifact_dir().join("label_manifest.json");
    let split_manifest_path = split_manifest.as_ref().map(|_| {
        config
            .artifact_dir()
            .join("split_manifest.json")
            .display()
            .to_string()
    });
    let fingerprint = stable_hash_string(&format!(
        "{}|{}|{}|{}|{}",
        artifact.dataset_csv_path,
        feature_schema.feature_order_hash,
        label_manifest.version,
        artifact.row_count,
        artifact.sequence_count
    ));
    SequenceExportManifest {
        export_id: config.export_id.clone(),
        dataset_csv_path: artifact.dataset_csv_path.clone(),
        feature_schema_manifest_path: feature_schema_manifest_path.display().to_string(),
        label_manifest_path: label_manifest_path.display().to_string(),
        split_manifest_path,
        no_lookahead_proof_path: config
            .artifact_dir()
            .join("no_lookahead_proof.json")
            .display()
            .to_string(),
        storage_report_path: config
            .artifact_dir()
            .join("sequence_storage_report.json")
            .display()
            .to_string(),
        source_artifacts: config.kis_canonical_csv_paths.clone(),
        provenance_artifacts: config.kis_evidence_closure_paths.clone(),
        preflight_artifacts: config.outcome_link_depth_closure_paths.clone(),
        row_count: artifact.row_count,
        sequence_count: artifact.sequence_count,
        excluded_count: artifact.excluded_count,
        fingerprint,
        reason_codes: stable_reason_codes(&[ReasonCode::SequenceExportManifestBuilt]),
    }
}

fn build_external_bridge_readiness(
    artifact: &SequenceDatasetExportArtifact,
    quality_report: &SequenceDatasetQualityReport,
) -> ExternalModelBridgeReadinessReport {
    let dataset_export_ready = matches!(
        quality_report.quality_status,
        SequenceDatasetQualityStatus::ExportReady
            | SequenceDatasetQualityStatus::ExportReadyWithWarnings
    );
    let prediction_schema_ready = artifact.row_count > 0;
    let model_card_template_ready = true;
    let evaluation_gate_ready = true;
    let risk_integration_ready = true;
    let readiness_status = if !dataset_export_ready {
        ExternalModelBridgeReadinessStatus::NeedDatasetExport
    } else if !prediction_schema_ready {
        ExternalModelBridgeReadinessStatus::NeedPredictionSchema
    } else if !model_card_template_ready {
        ExternalModelBridgeReadinessStatus::NeedModelCardTemplate
    } else if !evaluation_gate_ready {
        ExternalModelBridgeReadinessStatus::NeedEvaluationGate
    } else if !risk_integration_ready {
        ExternalModelBridgeReadinessStatus::NeedRiskIntegration
    } else {
        ExternalModelBridgeReadinessStatus::ReadyForPredictionCsvImport
    };
    let example_prediction_schema = artifact
        .rows
        .first()
        .map(|row| {
            vec![ExternalPredictionSchema {
                prediction_id: stable_hash_string(&format!("prediction:{}", row.sequence_id)),
                sequence_id: row.sequence_id.clone(),
                symbol: row.symbol.clone(),
                timestamp_ms: row.label_timestamp_ms,
                p_win: Some("0.55".to_string()),
                p_stop: Some("0.20".to_string()),
                expected_return: Some("0.012".to_string()),
                expected_drawdown: Some("0.006".to_string()),
                confidence: Some("0.51".to_string()),
                model_version: "template-v1".to_string(),
                reason_codes: stable_reason_codes(&[ReasonCode::ExternalPredictionSchemaBuilt]),
            }]
        })
        .unwrap_or_default();
    ExternalModelBridgeReadinessReport {
        dataset_export_ready,
        prediction_schema_ready,
        model_card_template_ready,
        evaluation_gate_ready,
        risk_integration_ready,
        readiness_status,
        blockers: Vec::new(),
        warnings: vec!["external bridge remains import/evaluation only".to_string()],
        example_prediction_schema,
        reason_codes: stable_reason_codes(&[ReasonCode::ExternalModelBridgeReadinessBuilt]),
    }
}

fn build_mamba_gate(
    quality_report: &SequenceDatasetQualityReport,
    no_lookahead_proof: &NoLookaheadSequenceProof,
    storage_report: &SequenceDatasetStorageReport,
    bridge: &ExternalModelBridgeReadinessReport,
    control_tower_visibility_ready: bool,
) -> Mamba3FinExternalPrototypeGateReport {
    let sequence_export_ready = matches!(
        quality_report.quality_status,
        SequenceDatasetQualityStatus::ExportReady
            | SequenceDatasetQualityStatus::ExportReadyWithWarnings
    );
    let external_bridge_ready = matches!(
        bridge.readiness_status,
        ExternalModelBridgeReadinessStatus::ReadyForPredictionCsvImport
    );
    let risk_integration_ready = bridge.risk_integration_ready;
    let gate_status = if !matches!(
        no_lookahead_proof.proof_status,
        NoLookaheadProofStatus::NoLookaheadSafe
    ) {
        Mamba3FinExternalPrototypeGateStatus::BlockedByNoLookahead
    } else if !sequence_export_ready {
        Mamba3FinExternalPrototypeGateStatus::BlockedBySequenceDataset
    } else if !storage_report.within_budget {
        Mamba3FinExternalPrototypeGateStatus::BlockedByStorage
    } else if !external_bridge_ready {
        Mamba3FinExternalPrototypeGateStatus::BlockedByExternalBridge
    } else {
        Mamba3FinExternalPrototypeGateStatus::PlanningReady
    };
    let final_recommendation = match gate_status {
        Mamba3FinExternalPrototypeGateStatus::PlanningReady => {
            Mamba3FinExternalPrototypeRecommendation::PlanExternalPrototypeOnly
        }
        Mamba3FinExternalPrototypeGateStatus::BlockedBySequenceDataset => {
            Mamba3FinExternalPrototypeRecommendation::ExportMoreSequenceRows
        }
        Mamba3FinExternalPrototypeGateStatus::BlockedByNoLookahead
        | Mamba3FinExternalPrototypeGateStatus::BlockedByStorage
        | Mamba3FinExternalPrototypeGateStatus::BlockedByExternalBridge => {
            Mamba3FinExternalPrototypeRecommendation::HoldMamba3Deferred
        }
        Mamba3FinExternalPrototypeGateStatus::BlockedByEvidenceDepth => {
            Mamba3FinExternalPrototypeRecommendation::ImproveEvidenceDepth
        }
        _ => Mamba3FinExternalPrototypeRecommendation::KeepBaselineAndTrinity,
    };
    Mamba3FinExternalPrototypeGateReport {
        sequence_export_ready,
        external_bridge_ready,
        risk_integration_ready,
        control_tower_visibility_ready,
        rust_runtime_allowed: false,
        training_allowed: false,
        live_inference_allowed: false,
        gate_status,
        final_recommendation,
        reason_codes: stable_reason_codes(&[ReasonCode::Mamba3FinExternalPrototypeGateBuilt]),
    }
}

fn build_sequence_dataset_panel(
    artifact: &SequenceDatasetExportArtifact,
    feature_schema: &FeatureSchemaManifest,
    label_manifest: &LabelManifest,
    split_manifest: Option<&SequenceSplitManifest>,
    no_lookahead_proof: &NoLookaheadSequenceProof,
    quality_report: &SequenceDatasetQualityReport,
    external_bridge: &ExternalModelBridgeReadinessReport,
) -> SequenceDatasetPanel {
    SequenceDatasetPanel {
        export_status: format!("{:?}", quality_report.quality_status),
        dataset_csv_path: Some(artifact.dataset_csv_path.clone()),
        feature_schema_hash: feature_schema.feature_order_hash.clone(),
        label_manifest_hash: stable_hash_string(&format!(
            "{}|{}|{:?}",
            label_manifest.label_manifest_id, label_manifest.version, label_manifest.horizon_bars
        )),
        sequence_count: artifact.sequence_count,
        row_count: artifact.row_count,
        symbol_count: artifact.symbol_count,
        label_distribution: artifact.label_distribution.clone(),
        split_policy: split_manifest
            .map(|manifest| format!("{:?}", manifest.policy))
            .unwrap_or_else(|| "Unavailable".to_string()),
        no_lookahead_status: format!("{:?}", no_lookahead_proof.proof_status),
        storage_status: if quality_report.storage_bytes > 0 {
            "WithinBudget".to_string()
        } else {
            "Unavailable".to_string()
        },
        drift_status: None,
        external_bridge_status: format!("{:?}", external_bridge.readiness_status),
        mamba_gate_status: "RuntimeDeferred".to_string(),
        next_actions: vec![
            "cargo run --quiet --bin soma_experiment -- sequence-dataset-export --config examples/soma_sequence_dataset_export_small.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- sequence-dataset-quality --config examples/soma_sequence_dataset_quality.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- sequence-dataset-drift --config examples/soma_sequence_dataset_drift.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- external-bridge-readiness --config examples/soma_external_bridge_readiness.toml".to_string(),
            "cargo run --quiet --bin soma_experiment -- mamba3fin-prototype-gate --config examples/soma_mamba3fin_prototype_gate.toml".to_string(),
        ],
        no_train_button: true,
        no_live_button: true,
        no_order_account_controls: true,
        reason_codes: stable_reason_codes(&[ReasonCode::ControlTowerSequenceDatasetPanelBuilt]),
    }
}

fn write_bundle_outputs(
    config: &SequenceDatasetExportConfig,
    bundle: &SequenceDatasetExportBundle,
) -> Result<(), String> {
    let dir = config.artifact_dir();
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    write_json_text(&dir, "feature_schema", &bundle.feature_schema_manifest)?;
    write_json_text(&dir, "label_manifest", &bundle.label_manifest)?;
    write_json_text(
        &dir,
        "sequence_export_manifest",
        &bundle.sequence_export_manifest,
    )?;
    if let Some(split_manifest) = &bundle.split_manifest {
        write_json_text(&dir, "split_manifest", split_manifest)?;
    }
    write_json_text(&dir, "no_lookahead_proof", &bundle.no_lookahead_proof)?;
    write_json_text(&dir, "sequence_storage_report", &bundle.storage_report)?;
    write_text_file(
        &dir,
        "sequence_dataset_quality.txt",
        &serde_json::to_string_pretty(&bundle.quality_report).map_err(|err| err.to_string())?,
    )?;
    write_text_file(
        &dir,
        "no_lookahead_proof.txt",
        &serde_json::to_string_pretty(&bundle.no_lookahead_proof).map_err(|err| err.to_string())?,
    )?;
    write_text_file(
        &dir,
        "sequence_storage_report.txt",
        &serde_json::to_string_pretty(&bundle.storage_report).map_err(|err| err.to_string())?,
    )?;
    if let Some(drift) = &bundle.drift_guard_report {
        write_json_text(&dir, "sequence_drift_guard", drift)?;
        write_text_file(
            &dir,
            "sequence_drift_guard.txt",
            &serde_json::to_string_pretty(drift).map_err(|err| err.to_string())?,
        )?;
    }
    if let Some(replay) = &bundle.replay_check {
        write_json_text(&dir, "sequence_replay_check", replay)?;
        write_text_file(
            &dir,
            "sequence_replay_check.txt",
            &serde_json::to_string_pretty(replay).map_err(|err| err.to_string())?,
        )?;
    }
    write_text_file(
        &dir,
        "external_model_bridge_readiness.txt",
        &serde_json::to_string_pretty(&bundle.external_bridge_readiness_report)
            .map_err(|err| err.to_string())?,
    )?;
    write_text_file(
        &dir,
        "mamba3fin_external_prototype_gate.txt",
        &serde_json::to_string_pretty(&bundle.mamba3fin_external_prototype_gate_report)
            .map_err(|err| err.to_string())?,
    )?;
    if let Some(panel) = &bundle.control_tower_sequence_panel_summary {
        write_text_file(&dir, "control_tower_sequence_panel.txt", &panel.to_text())?;
    }
    fs::write(dir.join("summary.txt"), &bundle.final_summary).map_err(|err| err.to_string())?;
    Ok(())
}

fn write_dataset_csv(
    config: &SequenceDatasetExportConfig,
    feature_schema: &FeatureSchemaManifest,
    rows: &[SequenceDatasetRow],
) -> Result<PathBuf, String> {
    let dir = config.artifact_dir();
    fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    let path = dir.join("dataset.csv");
    let mut lines = Vec::new();
    let mut header = vec![
        "sequence_id".to_string(),
        "symbol".to_string(),
        "market".to_string(),
        "timeframe".to_string(),
        "window_length".to_string(),
        "horizon_bars".to_string(),
        "window_start_timestamp_ms".to_string(),
        "window_end_timestamp_ms".to_string(),
        "label_timestamp_ms".to_string(),
    ];
    header.extend(feature_schema.feature_names.clone());
    header.extend([
        "label_value".to_string(),
        "label_kind".to_string(),
        "source_kind".to_string(),
        "source_class".to_string(),
        "official_readiness_eligible".to_string(),
        "no_lookahead_safe".to_string(),
        "feature_schema_hash".to_string(),
        "label_manifest_hash".to_string(),
    ]);
    lines.push(header.join(","));
    for row in rows {
        let mut record = vec![
            row.sequence_id.clone(),
            row.symbol.clone(),
            row.market.clone(),
            row.timeframe.clone(),
            row.window_length.to_string(),
            row.horizon_bars.to_string(),
            row.window_start_timestamp_ms.to_string(),
            row.window_end_timestamp_ms.to_string(),
            row.label_timestamp_ms.to_string(),
        ];
        record.extend(row.feature_values.iter().map(|value| format!("{value:.6}")));
        record.extend([
            row.label_value.clone(),
            row.label_kind.as_str().to_string(),
            row.source_kind.clone(),
            row.source_class.clone(),
            row.official_readiness_eligible.to_string(),
            row.no_lookahead_safe.to_string(),
            row.feature_schema_hash.clone(),
            row.label_manifest_hash.clone(),
        ]);
        lines.push(record.join(","));
    }
    fs::write(&path, lines.join("\n")).map_err(|err| err.to_string())?;
    Ok(path)
}

fn write_json_text<T: Serialize>(dir: &Path, stem: &str, value: &T) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    let json = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    fs::write(dir.join(format!("{stem}.json")), &json).map_err(|err| err.to_string())?;
    Ok(())
}

fn write_text_file(dir: &Path, name: &str, text: &str) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    fs::write(dir.join(name), text).map_err(|err| err.to_string())
}

fn load_source_row_seeds(paths: &[String]) -> Result<Vec<SourceRowSeed>, String> {
    let values = load_json_values(paths)?;
    let mut rows = Vec::new();
    for value in values {
        if let Some(items) = value
            .get("source_rows")
            .and_then(|item| serde_json::from_value::<Vec<SourceRowSeed>>(item.clone()).ok())
        {
            rows.extend(items);
            continue;
        }
        if let Ok(items) = serde_json::from_value::<Vec<SourceRowSeed>>(value.clone()) {
            rows.extend(items);
            continue;
        }
        if let Ok(item) = serde_json::from_value::<SourceRowSeed>(value.clone()) {
            rows.push(item);
        }
    }
    rows.sort_by(|left, right| {
        left.window_end_timestamp_ms
            .cmp(&right.window_end_timestamp_ms)
            .then_with(|| left.symbol.cmp(&right.symbol))
    });
    Ok(rows)
}

fn load_manifest_summary(path: &Path) -> Result<ManifestSummary, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let value: Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;
    Ok(ManifestSummary {
        feature_schema_manifest_path: value
            .get("feature_schema_manifest_path")
            .and_then(|item| item.as_str())
            .unwrap_or_default()
            .to_string(),
        label_manifest_path: value
            .get("label_manifest_path")
            .and_then(|item| item.as_str())
            .unwrap_or_default()
            .to_string(),
        source_artifacts: value
            .get("source_artifacts")
            .and_then(|item| serde_json::from_value(item.clone()).ok())
            .unwrap_or_default(),
        row_count: value
            .get("row_count")
            .and_then(|item| item.as_u64())
            .unwrap_or_default() as usize,
        label_distribution: value
            .get("label_distribution")
            .and_then(|item| serde_json::from_value(item.clone()).ok())
            .unwrap_or_default(),
    })
}

fn diff_label_distribution(
    baseline: &BTreeMap<String, usize>,
    current: &BTreeMap<String, usize>,
) -> BTreeMap<String, isize> {
    baseline
        .keys()
        .chain(current.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|key| {
            let base = baseline.get(&key).copied().unwrap_or_default() as isize;
            let now = current.get(&key).copied().unwrap_or_default() as isize;
            (key, now - base)
        })
        .collect()
}

fn label_distribution(rows: &[SequenceDatasetRow]) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for row in rows {
        *out.entry(row.label_kind.as_str().to_string()).or_insert(0) += 1;
    }
    out
}

fn load_json_values(paths: &[String]) -> Result<Vec<Value>, String> {
    let mut values = Vec::new();
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        values.push(serde_json::from_str(&text).map_err(|err| err.to_string())?);
    }
    Ok(values)
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

fn collect_string_items(values: &[Value], keys: &[&str]) -> Vec<String> {
    let mut items = BTreeSet::new();
    for key in keys {
        for value in values {
            collect_key_values(value, key, &mut |matched| match matched {
                Value::String(text) => {
                    items.insert(text.clone());
                }
                Value::Array(entries) => {
                    for entry in entries {
                        if let Some(text) = entry.as_str() {
                            items.insert(text.to_string());
                        }
                    }
                }
                _ => {}
            });
        }
    }
    items.into_iter().collect()
}

fn first_string_array(values: &[Value], keys: &[&str]) -> Vec<String> {
    for key in keys {
        for value in values {
            let mut found = None;
            collect_key_values(value, key, &mut |matched| match matched {
                Value::Array(entries) if found.is_none() => {
                    found = Some(
                        entries
                            .iter()
                            .filter_map(|entry| entry.as_str().map(|text| text.to_string()))
                            .collect::<Vec<_>>(),
                    );
                }
                Value::String(text) if found.is_none() => {
                    found = Some(vec![text.clone()]);
                }
                _ => {}
            });
            if let Some(items) = found {
                return items;
            }
        }
    }
    Vec::new()
}

fn first_numeric_array(values: &[Value], keys: &[&str]) -> Vec<usize> {
    for key in keys {
        for value in values {
            let mut found = None;
            collect_key_values(value, key, &mut |matched| match matched {
                Value::Array(entries) if found.is_none() => {
                    found = Some(
                        entries
                            .iter()
                            .filter_map(|entry| entry.as_u64().map(|item| item as usize))
                            .collect::<Vec<_>>(),
                    );
                }
                Value::Number(number) if found.is_none() => {
                    found = number.as_u64().map(|item| vec![item as usize]);
                }
                _ => {}
            });
            if let Some(items) = found {
                return items;
            }
        }
    }
    Vec::new()
}

fn first_label_kind_array(values: &[Value], keys: &[&str]) -> Vec<SequenceLabelKind> {
    for key in keys {
        for value in values {
            let mut found = None;
            collect_key_values(value, key, &mut |matched| match matched {
                Value::Array(entries) if found.is_none() => {
                    found = Some(
                        entries
                            .iter()
                            .filter_map(|entry| entry.as_str().and_then(parse_label_kind))
                            .collect::<Vec<_>>(),
                    );
                }
                Value::String(text) if found.is_none() => {
                    found = parse_label_kind(text).map(|kind| vec![kind]);
                }
                _ => {}
            });
            if let Some(items) = found {
                return items;
            }
        }
    }
    Vec::new()
}

fn parse_label_kind(text: &str) -> Option<SequenceLabelKind> {
    match text {
        "TakeProfit" => Some(SequenceLabelKind::TakeProfit),
        "StopLoss" => Some(SequenceLabelKind::StopLoss),
        "TimeExpired" => Some(SequenceLabelKind::TimeExpired),
        "NoTradeCounterfactual" => Some(SequenceLabelKind::NoTradeCounterfactual),
        "RiskDeniedCounterfactual" => Some(SequenceLabelKind::RiskDeniedCounterfactual),
        "Unknown" => Some(SequenceLabelKind::Unknown),
        _ => None,
    }
}

fn max_usize(values: &[Value], keys: &[&str]) -> usize {
    keys.iter()
        .flat_map(|key| {
            values.iter().flat_map(move |value| {
                let mut out = Vec::new();
                collect_key_values(value, key, &mut |matched| {
                    if let Some(item) = matched.as_u64() {
                        out.push(item as usize);
                    }
                });
                out
            })
        })
        .max()
        .unwrap_or_default()
}

fn first_string(values: &[Value], keys: &[&str]) -> Option<String> {
    for key in keys {
        for value in values {
            let mut found = None;
            collect_key_values(value, key, &mut |matched| {
                if found.is_none() {
                    found = matched.as_str().map(|item| item.to_string());
                }
            });
            if found.is_some() {
                return found;
            }
        }
    }
    None
}

fn first_f64(values: &[Value], keys: &[&str]) -> Option<f64> {
    for key in keys {
        for value in values {
            let mut found = None;
            collect_key_values(value, key, &mut |matched| {
                if found.is_none() {
                    found = matched.as_f64();
                }
            });
            if found.is_some() {
                return found;
            }
        }
    }
    None
}

fn all_bool(values: &[Value], keys: &[&str]) -> bool {
    let mut flags = Vec::new();
    for key in keys {
        for value in values {
            collect_key_values(value, key, &mut |matched| {
                if let Some(flag) = matched.as_bool() {
                    flags.push(flag);
                }
            });
        }
    }
    !flags.is_empty() && flags.into_iter().all(|flag| flag)
}
