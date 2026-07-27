//! Compact micro feature diagnostics and bounded future challenger preregistration.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    time::Instant,
};

use serde::{Deserialize, Serialize};

use crate::stable_hash_string;

use super::{
    MomentumCandleV0, MomentumFeatureConfigV0, build_momentum_features_v0,
    momentum_future_prediction_v4::{
        ArtifactBuilderV4_2, ArtifactReaderV4_2, as_u64, as_usize, persist_artifact, read_single,
    },
    momentum_micro_label_forensics_v1::{
        MomentumMicroChallengerDesignEvidenceClassV1, MomentumMicroLabelForensicsStatusV1,
        MomentumMicroProtectedBeforeStateV1, read_momentum_micro_label_forensics_report_v1,
        validate_momentum_micro_protected_before_state_v1,
    },
    momentum_multitimeframe_history_v1::{
        MomentumHistoricalTimeframeV1, MomentumQualifiedReplayCandleEvidenceV1,
        load_momentum_qualified_six_evidence_v1,
    },
    momentum_qualified_six_replay_v1::{
        MomentumQualifiedDiagnosticSourceV1, MomentumQualifiedParticipantV1,
        MomentumReplayPartitionV1, load_momentum_qualified_diagnostic_source_header_v1,
        load_momentum_qualified_diagnostic_source_v1,
    },
};

const ROOT: &str = "state/historical_replay/momentum_micro_challenger_design/v1";
const FEATURE_REGISTRATION_VERSION: &str = "momentum-micro-feature-forensics-registration-v1";
const SOURCE_AUDIT_VERSION: &str = "momentum-micro-source-feature-schema-audit-v1";
const REDUNDANCY_VERSION: &str = "momentum-micro-feature-redundancy-audit-v1";
const BIN_POLICY_VERSION: &str = "momentum-micro-feature-drift-bin-policy-v1";
const FEATURE_SHIFT_RECEIPT_VERSION: &str = "momentum-micro-feature-partition-shift-receipt-v1";
const SHIFT_VERSION: &str = "momentum-micro-feature-partition-shift-audit-v1";
const NORMALIZER_VERSION: &str = "momentum-micro-normalizer-stability-audit-v1";
const COMPACT_POLICY_VERSION: &str = "momentum-compact-micro-feature-policy-v1";
const COMPACT_REPLAY_VERSION: &str = "momentum-compact-micro-integrity-replay-v1";
const FEATURE_JOURNAL_VERSION: &str = "momentum-micro-feature-forensics-journal-v1";
const FEATURE_REPORT_VERSION: &str = "momentum-micro-feature-forensics-public-report-v1";
const BOUNDARY_VERSION: &str = "momentum-micro-task-partition-boundary-v1";
const TASK_VERSION: &str = "momentum-micro-task-registration-v1";
const PARTICIPANT_VERSION: &str = "momentum-micro-participant-registration-v1";
const SCREENING_GATE_VERSION: &str = "momentum-micro-screening-gate-v1";
const SCREENING_REGISTRATION_VERSION: &str = "momentum-micro-challenger-screening-registration-v1";
const SCREENING_JOURNAL_VERSION: &str = "momentum-micro-screening-journal-v1";
const FINAL_REPORT_VERSION: &str = "momentum-micro-challenger-design-public-report-v1";
const TEN_MINUTE_MS: u64 = 10 * 60 * 1_000;
const COMPACT_CONTEXT_LENGTH: usize = 17;
const REDUNDANCY_THRESHOLD: f64 = 0.98;
const DRIFT_STABLE_THRESHOLD: f64 = 0.10;
const DRIFT_MODERATE_THRESHOLD: f64 = 0.25;
const EPSILON: f64 = 1e-12;
const PUBLIC_LABELS: [&str; 6] = [
    "HistoricalResearchOnly",
    "PostResultResearchDesignOnly",
    "MicroChallengerNotExecuted",
    "HoldoutClosed",
    "NotLiveAuthority",
    "NotTradingAuthority",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMicroFeatureForensicsStatusV1 {
    Unregistered,
    Registered,
    Complete,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumMicroFeatureForensicsRunModeV1 {
    Status,
    ExecuteLocal,
}

impl MomentumMicroFeatureForensicsRunModeV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::ExecuteLocal => "execute-local",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumMicroChallengerRegistrationRunModeV1 {
    Status,
    RegisterLocal,
}

impl MomentumMicroChallengerRegistrationRunModeV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::RegisterLocal => "register-local",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMicroPartitionShiftClassV1 {
    Stable,
    ModerateShift,
    MaterialShift,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMicroRankDeficiencyV1 {
    FullRankByFixedAudit,
    CorrelatedGroupsObserved,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MomentumMicroTaskV1 {
    T10NextTenMinuteDirection,
    T30NextThirtyMinuteDirection,
}

impl MomentumMicroTaskV1 {
    const ORDERED: [Self; 2] = [
        Self::T10NextTenMinuteDirection,
        Self::T30NextThirtyMinuteDirection,
    ];

    fn parse(value: &str) -> Result<Self, String> {
        Self::ORDERED
            .into_iter()
            .find(|candidate| format!("{candidate:?}") == value)
            .ok_or_else(|| "micro task rejected".to_string())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MomentumMicroParticipantV1 {
    C0TaskSpecificConstant,
    C1TenMinuteAnchorBaseline,
    C2CompactMicroLogistic,
    C3CompactMicroStrongShrinkLogistic,
    C4CompactMicroTrainingOnlyCalibratedLogistic,
}

impl MomentumMicroParticipantV1 {
    const ORDERED: [Self; 5] = [
        Self::C0TaskSpecificConstant,
        Self::C1TenMinuteAnchorBaseline,
        Self::C2CompactMicroLogistic,
        Self::C3CompactMicroStrongShrinkLogistic,
        Self::C4CompactMicroTrainingOnlyCalibratedLogistic,
    ];

    fn parse(value: &str) -> Result<Self, String> {
        Self::ORDERED
            .into_iter()
            .find(|candidate| format!("{candidate:?}") == value)
            .ok_or_else(|| "micro participant rejected".to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroFeatureForensicsRegistrationV1 {
    pub registration_version: String,
    pub source_replay_digest: String,
    pub source_diagnostic_digest: String,
    pub label_forensics_digest: String,
    pub protected_before_state_digest: String,
    pub audited_participants: Vec<String>,
    pub audited_timeframes: Vec<MomentumHistoricalTimeframeV1>,
    pub finite_policy_digest: String,
    pub constant_feature_policy_digest: String,
    pub redundancy_policy_digest: String,
    pub partition_shift_policy_digest: String,
    pub normalizer_drift_policy_digest: String,
    pub label_based_feature_selection_forbidden: bool,
    pub validation_selected_feature_selection_forbidden: bool,
    pub holdout_access_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroSourceFeatureAuditV1 {
    pub audit_version: String,
    pub participant_id: String,
    pub feature_dimension: usize,
    pub feature_order_digest: String,
    pub source_timeframes: Vec<MomentumHistoricalTimeframeV1>,
    pub finite_value_count: usize,
    pub constant_or_near_constant_count: usize,
    pub duplicate_semantic_feature_count: usize,
    pub normalizer_identity: String,
    pub normalizer_finite: bool,
    pub development_availability_count: usize,
    pub validation_availability_count: usize,
    pub source_candle_complete: bool,
    pub holdout_access_count: usize,
    pub audit_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumMicroRedundancyAuditV1 {
    pub audit_version: String,
    pub schema_id: String,
    pub absolute_pearson_threshold: f64,
    pub correlated_feature_pair_count: usize,
    pub correlated_feature_group_count: usize,
    pub maximum_group_size: usize,
    pub cross_timeframe_duplicate_pattern_count: usize,
    pub rank_deficiency: MomentumMicroRankDeficiencyV1,
    pub development_only_redundancy_identity: String,
    pub validation_confirmation_identity: String,
    pub validation_modified_policy: bool,
    pub audit_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumMicroFeatureDriftBinPolicyV1 {
    policy_version: String,
    schema_id: String,
    feature_ids: Vec<String>,
    boundary_offsets: Vec<u64>,
    private_boundary_bits: Vec<u64>,
    development_only: bool,
    validation_access_count_before_persist: usize,
    policy_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroFeaturePartitionShiftReceiptV1 {
    pub receipt_version: String,
    pub feature_id: String,
    pub development_finite_count: usize,
    pub validation_finite_count: usize,
    pub development_bin_support: Vec<usize>,
    pub validation_bin_support: Vec<usize>,
    pub population_stability_index_bits: u64,
    pub mean_shift_classification: MomentumMicroPartitionShiftClassV1,
    pub standard_deviation_shift_classification: MomentumMicroPartitionShiftClassV1,
    pub out_of_development_range_validation_count: usize,
    pub integrity_passed: bool,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroPartitionShiftAuditV1 {
    pub audit_version: String,
    pub schema_id: String,
    pub feature_count: usize,
    pub development_finite_count: usize,
    pub validation_finite_count: usize,
    pub stable_feature_count: usize,
    pub moderate_shift_feature_count: usize,
    pub material_shift_feature_count: usize,
    pub out_of_development_range_validation_count: usize,
    pub aggregate_classification: MomentumMicroPartitionShiftClassV1,
    pub feature_receipts: Vec<MomentumMicroFeaturePartitionShiftReceiptV1>,
    pub bin_policy_digest: String,
    pub integrity_passed: bool,
    pub audit_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumMicroNormalizerStabilityAuditV1 {
    pub audit_version: String,
    pub participant_id: String,
    pub refit_count: usize,
    pub finite_status: bool,
    pub maximum_shift: f64,
    pub median_shift: f64,
    pub percentile_95_shift: f64,
    pub shift_sign_change_count: usize,
    pub partition_boundary_shift: bool,
    pub normalizer_digest_trajectory: String,
    pub prior_sprint_drift_preserved: bool,
    pub audit_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumCompactMicroFeaturePolicyV1 {
    pub policy_version: String,
    pub included_timeframes: Vec<MomentumHistoricalTimeframeV1>,
    pub per_timeframe_feature_ids: Vec<String>,
    pub cross_timeframe_feature_ids: Vec<String>,
    pub context_length: usize,
    pub zero_range_policy_digest: String,
    pub zero_denominator_policy_digest: String,
    pub feature_order_digest: String,
    pub schema_digest: String,
    pub target_selected_features: bool,
    pub validation_selected_features: bool,
    pub holdout_selected_features: bool,
}

impl MomentumCompactMicroFeaturePolicyV1 {
    pub fn feature_dimension(&self) -> usize {
        self.included_timeframes.len() * self.per_timeframe_feature_ids.len()
            + self.cross_timeframe_feature_ids.len()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumCompactMicroIntegrityReplayV1 {
    pub replay_version: String,
    pub partition: MomentumReplayPartitionV1,
    pub eligible_event_count: usize,
    pub finite_block_count: usize,
    pub missing_evidence_count: usize,
    pub partial_candle_count: usize,
    pub zero_range_fallback_count: usize,
    pub zero_denominator_fallback_count: usize,
    pub constant_feature_count: usize,
    pub redundancy_audit_digest: String,
    pub distribution_shift_audit_digest: String,
    pub feature_schema_digest: String,
    pub future_access_count: usize,
    pub partial_access_count: usize,
    pub holdout_access_count: usize,
    pub deterministic_replay_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroTaskPartitionBoundaryV1 {
    pub boundary_version: String,
    pub task: MomentumMicroTaskV1,
    pub eligible_start_timestamp_ms: u64,
    pub eligible_end_timestamp_ms: u64,
    pub development_end_exclusive_ms: u64,
    pub validation_end_exclusive_ms: u64,
    pub holdout_start_timestamp_ms: u64,
    pub common_eligible_event_count: usize,
    pub development_event_count: usize,
    pub validation_event_count: usize,
    pub holdout_event_count: usize,
    pub label_values_read_for_boundary: usize,
    pub holdout_labels_opened: bool,
    pub boundary_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroTaskRegistrationV1 {
    pub registration_version: String,
    pub task: MomentumMicroTaskV1,
    pub event_cadence_ms: u64,
    pub target_horizon_candles: usize,
    pub feature_policy_digest: String,
    pub boundary_digest: String,
    pub t60_experiment: bool,
    pub model_execution_authorized: bool,
    pub task_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroParticipantRegistrationV1 {
    pub registration_version: String,
    pub task: MomentumMicroTaskV1,
    pub participant: MomentumMicroParticipantV1,
    pub participant_id: String,
    pub feature_policy_digest: String,
    pub fresh_task_parameters_required: bool,
    pub standard_l2_multiplier: usize,
    pub calibration_base_fit_percent: usize,
    pub calibration_fit_percent: usize,
    pub calibration_training_only: bool,
    pub validation_fit_forbidden: bool,
    pub holdout_fit_forbidden: bool,
    pub model_execution_authorized: bool,
    pub participant_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroScreeningGateV1 {
    pub gate_version: String,
    pub lower_brier_development_required: bool,
    pub lower_brier_validation_required: bool,
    pub finite_predictions_and_metrics_required: bool,
    pub probability_collapse_forbidden: bool,
    pub chronology_failure_forbidden: bool,
    pub leakage_failure_forbidden: bool,
    pub integrity_failure_forbidden: bool,
    pub sufficient_paired_support_required: bool,
    pub result_selected_mutation_forbidden: bool,
    pub holdout_access_forbidden: bool,
    pub correctness_override_forbidden: bool,
    pub gate_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroChallengerScreeningRegistrationV1 {
    pub registration_version: String,
    pub source_replay_digest: String,
    pub source_diagnostic_digest: String,
    pub compact_feature_policy_digest: String,
    pub label_forensics_digest: String,
    pub task_registrations: Vec<MomentumMicroTaskRegistrationV1>,
    pub participant_registrations: Vec<MomentumMicroParticipantRegistrationV1>,
    pub partition_policy_digest: String,
    pub training_policy_digest: String,
    pub screening_gate_digest: String,
    pub model_execution_authorized: bool,
    pub holdout_execution_authorized: bool,
    pub live_authority_forbidden: bool,
    pub governance_authority_forbidden: bool,
    pub trading_authority_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroDesignSafetyCountersV1 {
    pub holdout_feature_reads: usize,
    pub holdout_label_reads: usize,
    pub holdout_prediction_reads: usize,
    pub holdout_metric_reads: usize,
    pub new_model_fits: usize,
    pub new_predictions: usize,
    pub new_evaluations: usize,
    pub new_partition_aggregates: usize,
    pub live_network_requests: usize,
    pub live_parameter_updates: usize,
    pub live_normalizer_refits: usize,
    pub live_event_changes: usize,
    pub winner_selections: usize,
    pub rankings: usize,
    pub reward_applications: usize,
    pub penalty_applications: usize,
    pub chair_actions: usize,
    pub trading_actions: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumMicroFeatureForensicsReportV1 {
    pub report_version: String,
    pub run_mode: String,
    pub status: MomentumMicroFeatureForensicsStatusV1,
    pub evidence_class: MomentumMicroChallengerDesignEvidenceClassV1,
    pub registration_digest: Option<String>,
    pub label_forensics_digest: Option<String>,
    pub protected_before_state_digest: String,
    pub source_feature_audits: Vec<MomentumMicroSourceFeatureAuditV1>,
    pub redundancy_audits: Vec<MomentumMicroRedundancyAuditV1>,
    pub partition_shift_audits: Vec<MomentumMicroPartitionShiftAuditV1>,
    pub normalizer_stability_audits: Vec<MomentumMicroNormalizerStabilityAuditV1>,
    pub compact_feature_policy: Option<MomentumCompactMicroFeaturePolicyV1>,
    pub compact_integrity_replays: Vec<MomentumCompactMicroIntegrityReplayV1>,
    pub safety_counters: MomentumMicroDesignSafetyCountersV1,
    pub labels: Vec<String>,
    pub deterministic: bool,
    pub journal_digest: Option<String>,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub feature_computation_count: usize,
    pub runtime_duration_ms: u64,
    pub report_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroChallengerDesignReportV1 {
    pub report_version: String,
    pub run_mode: String,
    pub complete: bool,
    pub evidence_class: MomentumMicroChallengerDesignEvidenceClassV1,
    pub protected_before_state_digest: String,
    pub label_forensics_digest: String,
    pub feature_forensics_digest: String,
    pub compact_feature_policy_digest: String,
    pub compact_feature_dimension: usize,
    pub task_boundaries: Vec<MomentumMicroTaskPartitionBoundaryV1>,
    pub screening_registration: Option<MomentumMicroChallengerScreeningRegistrationV1>,
    pub screening_gate: Option<MomentumMicroScreeningGateV1>,
    pub model_execution_authorized: bool,
    pub holdout_execution_authorized: bool,
    pub safety_counters: MomentumMicroDesignSafetyCountersV1,
    pub labels: Vec<String>,
    pub deterministic: bool,
    pub journal_digest: Option<String>,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub runtime_duration_ms: u64,
    pub report_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumMicroFeatureForensicsJournalV1 {
    journal_version: String,
    registration_digest: String,
    source_audit_digests: Vec<String>,
    redundancy_audit_digests: Vec<String>,
    partition_shift_audit_digests: Vec<String>,
    normalizer_stability_audit_digests: Vec<String>,
    compact_feature_policy_digest: String,
    compact_integrity_replay_digests: Vec<String>,
    holdout_access_count: usize,
    model_execution_count: usize,
    journal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumMicroScreeningJournalV1 {
    journal_version: String,
    registration_digest: String,
    task_registration_digests: Vec<String>,
    participant_registration_digests: Vec<String>,
    screening_gate_digest: String,
    model_execution_count: usize,
    holdout_execution_count: usize,
    journal_digest: String,
}

#[derive(Clone)]
struct FeatureMatrix {
    feature_ids: Vec<String>,
    development: Vec<Vec<f64>>,
    validation: Vec<Vec<f64>>,
}

#[derive(Clone)]
struct CompactEventResult {
    values: Vec<f64>,
    zero_range_fallbacks: usize,
    zero_denominator_fallbacks: usize,
}

fn canonical_digest<T: Clone + std::fmt::Debug>(value: &T, clear: impl FnOnce(&mut T)) -> String {
    let mut canonical = value.clone();
    clear(&mut canonical);
    stable_hash_string(&format!("{canonical:?}"))
}

macro_rules! digest_fn {
    ($name:ident, $ty:ty, $field:ident) => {
        fn $name(value: &$ty) -> String {
            canonical_digest(value, |item| item.$field.clear())
        }
    };
}

digest_fn!(
    feature_registration_digest,
    MomentumMicroFeatureForensicsRegistrationV1,
    registration_digest
);
digest_fn!(
    source_audit_digest,
    MomentumMicroSourceFeatureAuditV1,
    audit_digest
);
digest_fn!(
    redundancy_digest,
    MomentumMicroRedundancyAuditV1,
    audit_digest
);
digest_fn!(
    bin_policy_digest,
    MomentumMicroFeatureDriftBinPolicyV1,
    policy_digest
);
digest_fn!(
    feature_shift_receipt_digest,
    MomentumMicroFeaturePartitionShiftReceiptV1,
    receipt_digest
);
digest_fn!(
    shift_digest,
    MomentumMicroPartitionShiftAuditV1,
    audit_digest
);
digest_fn!(
    normalizer_digest,
    MomentumMicroNormalizerStabilityAuditV1,
    audit_digest
);
digest_fn!(
    boundary_digest,
    MomentumMicroTaskPartitionBoundaryV1,
    boundary_digest
);
digest_fn!(task_digest, MomentumMicroTaskRegistrationV1, task_digest);
digest_fn!(
    participant_digest,
    MomentumMicroParticipantRegistrationV1,
    participant_digest
);
digest_fn!(gate_digest, MomentumMicroScreeningGateV1, gate_digest);
digest_fn!(
    screening_registration_digest,
    MomentumMicroChallengerScreeningRegistrationV1,
    registration_digest
);
digest_fn!(
    feature_journal_digest,
    MomentumMicroFeatureForensicsJournalV1,
    journal_digest
);
digest_fn!(
    screening_journal_digest,
    MomentumMicroScreeningJournalV1,
    journal_digest
);

fn compact_schema_digest(value: &MomentumCompactMicroFeaturePolicyV1) -> String {
    canonical_digest(value, |item| item.schema_digest.clear())
}

fn compact_replay_digest(value: &MomentumCompactMicroIntegrityReplayV1) -> String {
    canonical_digest(value, |item| item.deterministic_replay_digest.clear())
}

fn feature_report_digest(value: &MomentumMicroFeatureForensicsReportV1) -> String {
    canonical_digest(value, |item| {
        item.run_mode.clear();
        item.artifacts_written = 0;
        item.duplicate_artifact_count = 0;
        item.feature_computation_count = 0;
        item.runtime_duration_ms = 0;
        item.report_digest.clear();
    })
}

fn final_report_digest(value: &MomentumMicroChallengerDesignReportV1) -> String {
    canonical_digest(value, |item| {
        item.run_mode.clear();
        item.artifacts_written = 0;
        item.duplicate_artifact_count = 0;
        item.runtime_duration_ms = 0;
        item.report_digest.clear();
    })
}

fn timeframe_name(value: MomentumHistoricalTimeframeV1) -> &'static str {
    value.as_str()
}

fn parse_timeframe(value: &str) -> Result<MomentumHistoricalTimeframeV1, String> {
    MomentumHistoricalTimeframeV1::ORDERED
        .into_iter()
        .find(|timeframe| timeframe_name(*timeframe) == value)
        .ok_or_else(|| "micro timeframe rejected".to_string())
}

fn partition_name(value: MomentumReplayPartitionV1) -> &'static str {
    match value {
        MomentumReplayPartitionV1::Development => "development",
        MomentumReplayPartitionV1::Validation => "validation",
        MomentumReplayPartitionV1::SealedHoldout => "sealed-holdout",
    }
}

fn parse_partition(value: &str) -> Result<MomentumReplayPartitionV1, String> {
    match value {
        "development" => Ok(MomentumReplayPartitionV1::Development),
        "validation" => Ok(MomentumReplayPartitionV1::Validation),
        "sealed-holdout" => Ok(MomentumReplayPartitionV1::SealedHoldout),
        _ => Err("micro design partition rejected".to_string()),
    }
}

fn validate_feature_registration(
    value: &MomentumMicroFeatureForensicsRegistrationV1,
) -> Result<(), String> {
    let expected_timeframes = vec![
        MomentumHistoricalTimeframeV1::Minute1,
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
    ];
    if value.registration_version != FEATURE_REGISTRATION_VERSION
        || [
            &value.source_replay_digest,
            &value.source_diagnostic_digest,
            &value.label_forensics_digest,
            &value.protected_before_state_digest,
            &value.finite_policy_digest,
            &value.constant_feature_policy_digest,
            &value.redundancy_policy_digest,
            &value.partition_shift_policy_digest,
            &value.normalizer_drift_policy_digest,
        ]
        .iter()
        .any(|value| value.is_empty())
        || value.audited_participants != ["Q1", "Q2"]
        || value.audited_timeframes != expected_timeframes
        || !value.label_based_feature_selection_forbidden
        || !value.validation_selected_feature_selection_forbidden
        || !value.holdout_access_forbidden
        || value.registration_digest != feature_registration_digest(value)
    {
        return Err("micro feature registration rejected".to_string());
    }
    Ok(())
}

fn validate_compact_policy(value: &MomentumCompactMicroFeaturePolicyV1) -> Result<(), String> {
    let expected_timeframes = vec![
        MomentumHistoricalTimeframeV1::Minute1,
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
    ];
    if value.policy_version != COMPACT_POLICY_VERSION
        || value.included_timeframes != expected_timeframes
        || value.per_timeframe_feature_ids.len() != 16
        || value.cross_timeframe_feature_ids.len() != 5
        || value.context_length != COMPACT_CONTEXT_LENGTH
        || value.zero_range_policy_digest.is_empty()
        || value.zero_denominator_policy_digest.is_empty()
        || value.feature_order_digest.is_empty()
        || value.feature_dimension() != 69
        || value.target_selected_features
        || value.validation_selected_features
        || value.holdout_selected_features
        || value.schema_digest != compact_schema_digest(value)
    {
        return Err("compact micro feature policy rejected".to_string());
    }
    Ok(())
}

fn zero_safety(value: &MomentumMicroDesignSafetyCountersV1) -> bool {
    [
        value.holdout_feature_reads,
        value.holdout_label_reads,
        value.holdout_prediction_reads,
        value.holdout_metric_reads,
        value.new_model_fits,
        value.new_predictions,
        value.new_evaluations,
        value.new_partition_aggregates,
        value.live_network_requests,
        value.live_parameter_updates,
        value.live_normalizer_refits,
        value.live_event_changes,
        value.winner_selections,
        value.rankings,
        value.reward_applications,
        value.penalty_applications,
        value.chair_actions,
        value.trading_actions,
    ]
    .into_iter()
    .all(|value| value == 0)
}

fn validate_feature_report(value: &MomentumMicroFeatureForensicsReportV1) -> Result<(), String> {
    let complete = value.status == MomentumMicroFeatureForensicsStatusV1::Complete;
    if value.report_version != FEATURE_REPORT_VERSION
        || value.run_mode.is_empty()
        || value.evidence_class
            != MomentumMicroChallengerDesignEvidenceClassV1::PostResultResearchDesignOnly
        || value.protected_before_state_digest.is_empty()
        || value.labels != PUBLIC_LABELS.map(str::to_string)
        || !value.deterministic
        || !zero_safety(&value.safety_counters)
        || (complete
            && (value
                .registration_digest
                .as_deref()
                .is_none_or(str::is_empty)
                || value
                    .label_forensics_digest
                    .as_deref()
                    .is_none_or(str::is_empty)
                || value.source_feature_audits.len() != 2
                || value.redundancy_audits.len() != 3
                || value.partition_shift_audits.len() != 3
                || value.normalizer_stability_audits.len() != 2
                || value.compact_integrity_replays.len() != 2
                || value
                    .compact_feature_policy
                    .as_ref()
                    .is_none_or(|policy| validate_compact_policy(policy).is_err())
                || value.journal_digest.as_deref().is_none_or(str::is_empty)))
        || (!complete
            && (!value.source_feature_audits.is_empty() || value.compact_feature_policy.is_some()))
        || value.report_digest != feature_report_digest(value)
    {
        return Err("micro feature report rejected".to_string());
    }
    Ok(())
}

fn validate_screening_registration(
    value: &MomentumMicroChallengerScreeningRegistrationV1,
) -> Result<(), String> {
    let task_set = value
        .task_registrations
        .iter()
        .map(|registration| registration.task)
        .collect::<BTreeSet<_>>();
    let participant_set = value
        .participant_registrations
        .iter()
        .map(|registration| (registration.task, registration.participant))
        .collect::<BTreeSet<_>>();
    let expected_participant_set = MomentumMicroTaskV1::ORDERED
        .into_iter()
        .flat_map(|task| {
            MomentumMicroParticipantV1::ORDERED
                .into_iter()
                .map(move |participant| (task, participant))
        })
        .collect::<BTreeSet<_>>();
    let compact_participants_valid = value
        .participant_registrations
        .iter()
        .filter(|registration| {
            matches!(
                registration.participant,
                MomentumMicroParticipantV1::C2CompactMicroLogistic
                    | MomentumMicroParticipantV1::C3CompactMicroStrongShrinkLogistic
                    | MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic
            )
        })
        .all(|registration| {
            registration.feature_policy_digest == value.compact_feature_policy_digest
        });
    let anchor_digests = value
        .participant_registrations
        .iter()
        .filter(|registration| {
            registration.participant == MomentumMicroParticipantV1::C1TenMinuteAnchorBaseline
        })
        .map(|registration| &registration.feature_policy_digest)
        .collect::<BTreeSet<_>>();
    if value.registration_version != SCREENING_REGISTRATION_VERSION
        || [
            &value.source_replay_digest,
            &value.source_diagnostic_digest,
            &value.compact_feature_policy_digest,
            &value.label_forensics_digest,
            &value.partition_policy_digest,
            &value.training_policy_digest,
            &value.screening_gate_digest,
        ]
        .iter()
        .any(|value| value.is_empty())
        || value.task_registrations.len() != 2
        || value.participant_registrations.len() != 10
        || task_set != MomentumMicroTaskV1::ORDERED.into_iter().collect()
        || participant_set != expected_participant_set
        || !compact_participants_valid
        || anchor_digests.len() != 2
        || value.model_execution_authorized
        || value.holdout_execution_authorized
        || !value.live_authority_forbidden
        || !value.governance_authority_forbidden
        || !value.trading_authority_forbidden
        || value.registration_digest != screening_registration_digest(value)
    {
        return Err("micro screening registration rejected".to_string());
    }
    Ok(())
}

fn validate_final_report(value: &MomentumMicroChallengerDesignReportV1) -> Result<(), String> {
    let boundary_set = value
        .task_boundaries
        .iter()
        .map(|boundary| (boundary.task, boundary.boundary_digest.as_str()))
        .collect::<BTreeMap<_, _>>();
    let task_boundaries_match = value
        .screening_registration
        .as_ref()
        .is_none_or(|registration| {
            registration.task_registrations.iter().all(|task| {
                boundary_set.get(&task.task).copied() == Some(task.boundary_digest.as_str())
            })
        });
    let gate_matches = value
        .screening_gate
        .as_ref()
        .is_none_or(|gate| gate == &screening_gate());
    if value.report_version != FINAL_REPORT_VERSION
        || value.run_mode.is_empty()
        || value.evidence_class
            != MomentumMicroChallengerDesignEvidenceClassV1::PostResultResearchDesignOnly
        || [
            &value.protected_before_state_digest,
            &value.label_forensics_digest,
            &value.feature_forensics_digest,
            &value.compact_feature_policy_digest,
        ]
        .iter()
        .any(|value| value.is_empty())
        || value.compact_feature_dimension != 69
        || value.labels != PUBLIC_LABELS.map(str::to_string)
        || !value.deterministic
        || value.model_execution_authorized
        || value.holdout_execution_authorized
        || !zero_safety(&value.safety_counters)
        || (value.complete
            && (value.task_boundaries.len() != 2
                || boundary_set.len() != 2
                || !task_boundaries_match
                || value
                    .screening_registration
                    .as_ref()
                    .is_none_or(|registration| {
                        validate_screening_registration(registration).is_err()
                    })
                || !gate_matches
                || value.screening_gate.is_none()
                || value.journal_digest.as_deref().is_none_or(str::is_empty)))
        || value.report_digest != final_report_digest(value)
    {
        return Err("micro challenger design report rejected".to_string());
    }
    Ok(())
}

fn encode_feature_registration(
    value: &MomentumMicroFeatureForensicsRegistrationV1,
) -> Result<Vec<u8>, String> {
    validate_feature_registration(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroFeatureForensicsRegistrationV1")
        .string("registration_version", &value.registration_version)
        .string("source_replay_digest", &value.source_replay_digest)
        .string("source_diagnostic_digest", &value.source_diagnostic_digest)
        .string("label_forensics_digest", &value.label_forensics_digest)
        .string(
            "protected_before_state_digest",
            &value.protected_before_state_digest,
        )
        .strings("audited_participants", &value.audited_participants)
        .strings(
            "audited_timeframes",
            &value
                .audited_timeframes
                .iter()
                .map(|value| timeframe_name(*value).to_string())
                .collect::<Vec<_>>(),
        )
        .string("finite_policy_digest", &value.finite_policy_digest)
        .string(
            "constant_feature_policy_digest",
            &value.constant_feature_policy_digest,
        )
        .string("redundancy_policy_digest", &value.redundancy_policy_digest)
        .string(
            "partition_shift_policy_digest",
            &value.partition_shift_policy_digest,
        )
        .string(
            "normalizer_drift_policy_digest",
            &value.normalizer_drift_policy_digest,
        )
        .boolean(
            "label_based_feature_selection_forbidden",
            value.label_based_feature_selection_forbidden,
        )
        .boolean(
            "validation_selected_feature_selection_forbidden",
            value.validation_selected_feature_selection_forbidden,
        )
        .boolean("holdout_access_forbidden", value.holdout_access_forbidden)
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_feature_registration(
    bytes: &[u8],
) -> Result<MomentumMicroFeatureForensicsRegistrationV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumMicroFeatureForensicsRegistrationV1")?;
    let value = MomentumMicroFeatureForensicsRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        source_replay_digest: fields.string("source_replay_digest")?,
        source_diagnostic_digest: fields.string("source_diagnostic_digest")?,
        label_forensics_digest: fields.string("label_forensics_digest")?,
        protected_before_state_digest: fields.string("protected_before_state_digest")?,
        audited_participants: fields.strings("audited_participants")?,
        audited_timeframes: fields
            .strings("audited_timeframes")?
            .iter()
            .map(|value| parse_timeframe(value))
            .collect::<Result<Vec<_>, _>>()?,
        finite_policy_digest: fields.string("finite_policy_digest")?,
        constant_feature_policy_digest: fields.string("constant_feature_policy_digest")?,
        redundancy_policy_digest: fields.string("redundancy_policy_digest")?,
        partition_shift_policy_digest: fields.string("partition_shift_policy_digest")?,
        normalizer_drift_policy_digest: fields.string("normalizer_drift_policy_digest")?,
        label_based_feature_selection_forbidden: fields
            .boolean("label_based_feature_selection_forbidden")?,
        validation_selected_feature_selection_forbidden: fields
            .boolean("validation_selected_feature_selection_forbidden")?,
        holdout_access_forbidden: fields.boolean("holdout_access_forbidden")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_feature_registration(&value)?;
    Ok(value)
}

fn encode_compact_policy(value: &MomentumCompactMicroFeaturePolicyV1) -> Result<Vec<u8>, String> {
    validate_compact_policy(value)?;
    ArtifactBuilderV4_2::new("MomentumCompactMicroFeaturePolicyV1")
        .string("policy_version", &value.policy_version)
        .strings(
            "included_timeframes",
            &value
                .included_timeframes
                .iter()
                .map(|value| timeframe_name(*value).to_string())
                .collect::<Vec<_>>(),
        )
        .strings(
            "per_timeframe_feature_ids",
            &value.per_timeframe_feature_ids,
        )
        .strings(
            "cross_timeframe_feature_ids",
            &value.cross_timeframe_feature_ids,
        )
        .unsigned("context_length", as_u64(value.context_length)?)
        .string("zero_range_policy_digest", &value.zero_range_policy_digest)
        .string(
            "zero_denominator_policy_digest",
            &value.zero_denominator_policy_digest,
        )
        .string("feature_order_digest", &value.feature_order_digest)
        .string("schema_digest", &value.schema_digest)
        .boolean("target_selected_features", value.target_selected_features)
        .boolean(
            "validation_selected_features",
            value.validation_selected_features,
        )
        .boolean("holdout_selected_features", value.holdout_selected_features)
        .encode()
}

fn decode_compact_policy(bytes: &[u8]) -> Result<MomentumCompactMicroFeaturePolicyV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumCompactMicroFeaturePolicyV1")?;
    let value = MomentumCompactMicroFeaturePolicyV1 {
        policy_version: fields.string("policy_version")?,
        included_timeframes: fields
            .strings("included_timeframes")?
            .iter()
            .map(|value| parse_timeframe(value))
            .collect::<Result<Vec<_>, _>>()?,
        per_timeframe_feature_ids: fields.strings("per_timeframe_feature_ids")?,
        cross_timeframe_feature_ids: fields.strings("cross_timeframe_feature_ids")?,
        context_length: as_usize(fields.unsigned("context_length")?)?,
        zero_range_policy_digest: fields.string("zero_range_policy_digest")?,
        zero_denominator_policy_digest: fields.string("zero_denominator_policy_digest")?,
        feature_order_digest: fields.string("feature_order_digest")?,
        schema_digest: fields.string("schema_digest")?,
        target_selected_features: fields.boolean("target_selected_features")?,
        validation_selected_features: fields.boolean("validation_selected_features")?,
        holdout_selected_features: fields.boolean("holdout_selected_features")?,
    };
    fields.finish()?;
    validate_compact_policy(&value)?;
    Ok(value)
}

fn encode_safety(value: &MomentumMicroDesignSafetyCountersV1) -> Result<Vec<u8>, String> {
    let values = [
        value.holdout_feature_reads,
        value.holdout_label_reads,
        value.holdout_prediction_reads,
        value.holdout_metric_reads,
        value.new_model_fits,
        value.new_predictions,
        value.new_evaluations,
        value.new_partition_aggregates,
        value.live_network_requests,
        value.live_parameter_updates,
        value.live_normalizer_refits,
        value.live_event_changes,
        value.winner_selections,
        value.rankings,
        value.reward_applications,
        value.penalty_applications,
        value.chair_actions,
        value.trading_actions,
    ]
    .into_iter()
    .map(as_u64)
    .collect::<Result<Vec<_>, _>>()?;
    ArtifactBuilderV4_2::new("MomentumMicroDesignSafetyCountersV1")
        .unsigneds("counts", &values)
        .encode()
}

fn decode_safety(bytes: &[u8]) -> Result<MomentumMicroDesignSafetyCountersV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroDesignSafetyCountersV1")?;
    let values = fields
        .unsigneds("counts")?
        .into_iter()
        .map(as_usize)
        .collect::<Result<Vec<_>, _>>()?;
    fields.finish()?;
    if values.len() != 18 {
        return Err("micro design safety count rejected".to_string());
    }
    Ok(MomentumMicroDesignSafetyCountersV1 {
        holdout_feature_reads: values[0],
        holdout_label_reads: values[1],
        holdout_prediction_reads: values[2],
        holdout_metric_reads: values[3],
        new_model_fits: values[4],
        new_predictions: values[5],
        new_evaluations: values[6],
        new_partition_aggregates: values[7],
        live_network_requests: values[8],
        live_parameter_updates: values[9],
        live_normalizer_refits: values[10],
        live_event_changes: values[11],
        winner_selections: values[12],
        rankings: values[13],
        reward_applications: values[14],
        penalty_applications: values[15],
        chair_actions: values[16],
        trading_actions: values[17],
    })
}

fn encode_feature_report(value: &MomentumMicroFeatureForensicsReportV1) -> Result<Vec<u8>, String> {
    validate_feature_report(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroFeatureForensicsReportV1")
        .string("report_version", &value.report_version)
        .string("run_mode", &value.run_mode)
        .string("status", format!("{:?}", value.status))
        .string("evidence_class", format!("{:?}", value.evidence_class))
        .optional_string("registration_digest", &value.registration_digest)
        .optional_string("label_forensics_digest", &value.label_forensics_digest)
        .string(
            "protected_before_state_digest",
            &value.protected_before_state_digest,
        )
        .messages(
            "source_feature_audits",
            value
                .source_feature_audits
                .iter()
                .map(encode_source_audit)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "redundancy_audits",
            value
                .redundancy_audits
                .iter()
                .map(encode_redundancy)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "partition_shift_audits",
            value
                .partition_shift_audits
                .iter()
                .map(encode_shift)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "normalizer_stability_audits",
            value
                .normalizer_stability_audits
                .iter()
                .map(encode_normalizer)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "compact_feature_policy",
            value
                .compact_feature_policy
                .iter()
                .map(encode_compact_policy)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "compact_integrity_replays",
            value
                .compact_integrity_replays
                .iter()
                .map(encode_compact_replay)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "safety_counters",
            vec![encode_safety(&value.safety_counters)?],
        )
        .strings("labels", &value.labels)
        .boolean("deterministic", value.deterministic)
        .optional_string("journal_digest", &value.journal_digest)
        .unsigned("artifacts_written", as_u64(value.artifacts_written)?)
        .unsigned(
            "duplicate_artifact_count",
            as_u64(value.duplicate_artifact_count)?,
        )
        .unsigned(
            "feature_computation_count",
            as_u64(value.feature_computation_count)?,
        )
        .unsigned("runtime_duration_ms", value.runtime_duration_ms)
        .string("report_digest", &value.report_digest)
        .encode()
}

fn parse_feature_status(value: &str) -> Result<MomentumMicroFeatureForensicsStatusV1, String> {
    use MomentumMicroFeatureForensicsStatusV1 as S;
    match value {
        "Unregistered" => Ok(S::Unregistered),
        "Registered" => Ok(S::Registered),
        "Complete" => Ok(S::Complete),
        "IntegrityFailure" => Ok(S::IntegrityFailure),
        _ => Err("micro feature report status rejected".to_string()),
    }
}

fn encode_source_audit(value: &MomentumMicroSourceFeatureAuditV1) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumMicroSourceFeatureAuditV1")
        .string("audit_version", &value.audit_version)
        .string("participant_id", &value.participant_id)
        .unsigned("feature_dimension", as_u64(value.feature_dimension)?)
        .string("feature_order_digest", &value.feature_order_digest)
        .strings(
            "source_timeframes",
            &value
                .source_timeframes
                .iter()
                .map(|value| timeframe_name(*value).to_string())
                .collect::<Vec<_>>(),
        )
        .unsigned("finite_value_count", as_u64(value.finite_value_count)?)
        .unsigned(
            "constant_or_near_constant_count",
            as_u64(value.constant_or_near_constant_count)?,
        )
        .unsigned(
            "duplicate_semantic_feature_count",
            as_u64(value.duplicate_semantic_feature_count)?,
        )
        .string("normalizer_identity", &value.normalizer_identity)
        .boolean("normalizer_finite", value.normalizer_finite)
        .unsigned(
            "development_availability_count",
            as_u64(value.development_availability_count)?,
        )
        .unsigned(
            "validation_availability_count",
            as_u64(value.validation_availability_count)?,
        )
        .boolean("source_candle_complete", value.source_candle_complete)
        .unsigned("holdout_access_count", as_u64(value.holdout_access_count)?)
        .string("audit_digest", &value.audit_digest)
        .encode()
}

fn decode_source_audit(bytes: &[u8]) -> Result<MomentumMicroSourceFeatureAuditV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroSourceFeatureAuditV1")?;
    let value = MomentumMicroSourceFeatureAuditV1 {
        audit_version: fields.string("audit_version")?,
        participant_id: fields.string("participant_id")?,
        feature_dimension: as_usize(fields.unsigned("feature_dimension")?)?,
        feature_order_digest: fields.string("feature_order_digest")?,
        source_timeframes: fields
            .strings("source_timeframes")?
            .iter()
            .map(|value| parse_timeframe(value))
            .collect::<Result<Vec<_>, _>>()?,
        finite_value_count: as_usize(fields.unsigned("finite_value_count")?)?,
        constant_or_near_constant_count: as_usize(
            fields.unsigned("constant_or_near_constant_count")?,
        )?,
        duplicate_semantic_feature_count: as_usize(
            fields.unsigned("duplicate_semantic_feature_count")?,
        )?,
        normalizer_identity: fields.string("normalizer_identity")?,
        normalizer_finite: fields.boolean("normalizer_finite")?,
        development_availability_count: as_usize(
            fields.unsigned("development_availability_count")?,
        )?,
        validation_availability_count: as_usize(fields.unsigned("validation_availability_count")?)?,
        source_candle_complete: fields.boolean("source_candle_complete")?,
        holdout_access_count: as_usize(fields.unsigned("holdout_access_count")?)?,
        audit_digest: fields.string("audit_digest")?,
    };
    fields.finish()?;
    if value.audit_version != SOURCE_AUDIT_VERSION
        || value.audit_digest != source_audit_digest(&value)
    {
        return Err("micro source feature audit rejected".to_string());
    }
    Ok(value)
}

fn encode_redundancy(value: &MomentumMicroRedundancyAuditV1) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumMicroRedundancyAuditV1")
        .string("audit_version", &value.audit_version)
        .string("schema_id", &value.schema_id)
        .unsigned(
            "absolute_pearson_threshold_bits",
            value.absolute_pearson_threshold.to_bits(),
        )
        .unsigned(
            "correlated_feature_pair_count",
            as_u64(value.correlated_feature_pair_count)?,
        )
        .unsigned(
            "correlated_feature_group_count",
            as_u64(value.correlated_feature_group_count)?,
        )
        .unsigned("maximum_group_size", as_u64(value.maximum_group_size)?)
        .unsigned(
            "cross_timeframe_duplicate_pattern_count",
            as_u64(value.cross_timeframe_duplicate_pattern_count)?,
        )
        .string("rank_deficiency", format!("{:?}", value.rank_deficiency))
        .string(
            "development_only_redundancy_identity",
            &value.development_only_redundancy_identity,
        )
        .string(
            "validation_confirmation_identity",
            &value.validation_confirmation_identity,
        )
        .boolean(
            "validation_modified_policy",
            value.validation_modified_policy,
        )
        .string("audit_digest", &value.audit_digest)
        .encode()
}

fn decode_redundancy(bytes: &[u8]) -> Result<MomentumMicroRedundancyAuditV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroRedundancyAuditV1")?;
    let rank_deficiency = match fields.string("rank_deficiency")?.as_str() {
        "FullRankByFixedAudit" => MomentumMicroRankDeficiencyV1::FullRankByFixedAudit,
        "CorrelatedGroupsObserved" => MomentumMicroRankDeficiencyV1::CorrelatedGroupsObserved,
        "IntegrityFailure" => MomentumMicroRankDeficiencyV1::IntegrityFailure,
        _ => return Err("micro rank classification rejected".to_string()),
    };
    let value = MomentumMicroRedundancyAuditV1 {
        audit_version: fields.string("audit_version")?,
        schema_id: fields.string("schema_id")?,
        absolute_pearson_threshold: f64::from_bits(
            fields.unsigned("absolute_pearson_threshold_bits")?,
        ),
        correlated_feature_pair_count: as_usize(fields.unsigned("correlated_feature_pair_count")?)?,
        correlated_feature_group_count: as_usize(
            fields.unsigned("correlated_feature_group_count")?,
        )?,
        maximum_group_size: as_usize(fields.unsigned("maximum_group_size")?)?,
        cross_timeframe_duplicate_pattern_count: as_usize(
            fields.unsigned("cross_timeframe_duplicate_pattern_count")?,
        )?,
        rank_deficiency,
        development_only_redundancy_identity: fields
            .string("development_only_redundancy_identity")?,
        validation_confirmation_identity: fields.string("validation_confirmation_identity")?,
        validation_modified_policy: fields.boolean("validation_modified_policy")?,
        audit_digest: fields.string("audit_digest")?,
    };
    fields.finish()?;
    if value.audit_version != REDUNDANCY_VERSION
        || value.absolute_pearson_threshold.to_bits() != REDUNDANCY_THRESHOLD.to_bits()
        || value.validation_modified_policy
        || value.audit_digest != redundancy_digest(&value)
    {
        return Err("micro redundancy audit rejected".to_string());
    }
    Ok(value)
}

fn parse_shift_class(value: &str) -> Result<MomentumMicroPartitionShiftClassV1, String> {
    match value {
        "Stable" => Ok(MomentumMicroPartitionShiftClassV1::Stable),
        "ModerateShift" => Ok(MomentumMicroPartitionShiftClassV1::ModerateShift),
        "MaterialShift" => Ok(MomentumMicroPartitionShiftClassV1::MaterialShift),
        "IntegrityFailure" => Ok(MomentumMicroPartitionShiftClassV1::IntegrityFailure),
        _ => Err("micro shift classification rejected".to_string()),
    }
}

fn encode_feature_shift_receipt(
    value: &MomentumMicroFeaturePartitionShiftReceiptV1,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumMicroFeaturePartitionShiftReceiptV1")
        .string("receipt_version", &value.receipt_version)
        .string("feature_id", &value.feature_id)
        .unsigned(
            "development_finite_count",
            as_u64(value.development_finite_count)?,
        )
        .unsigned(
            "validation_finite_count",
            as_u64(value.validation_finite_count)?,
        )
        .unsigneds(
            "development_bin_support",
            &value
                .development_bin_support
                .iter()
                .map(|value| as_u64(*value))
                .collect::<Result<Vec<_>, _>>()?,
        )
        .unsigneds(
            "validation_bin_support",
            &value
                .validation_bin_support
                .iter()
                .map(|value| as_u64(*value))
                .collect::<Result<Vec<_>, _>>()?,
        )
        .unsigned(
            "population_stability_index_bits",
            value.population_stability_index_bits,
        )
        .string(
            "mean_shift_classification",
            format!("{:?}", value.mean_shift_classification),
        )
        .string(
            "standard_deviation_shift_classification",
            format!("{:?}", value.standard_deviation_shift_classification),
        )
        .unsigned(
            "out_of_development_range_validation_count",
            as_u64(value.out_of_development_range_validation_count)?,
        )
        .boolean("integrity_passed", value.integrity_passed)
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_feature_shift_receipt(
    bytes: &[u8],
) -> Result<MomentumMicroFeaturePartitionShiftReceiptV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumMicroFeaturePartitionShiftReceiptV1")?;
    let value = MomentumMicroFeaturePartitionShiftReceiptV1 {
        receipt_version: fields.string("receipt_version")?,
        feature_id: fields.string("feature_id")?,
        development_finite_count: as_usize(fields.unsigned("development_finite_count")?)?,
        validation_finite_count: as_usize(fields.unsigned("validation_finite_count")?)?,
        development_bin_support: fields
            .unsigneds("development_bin_support")?
            .iter()
            .map(|value| as_usize(*value))
            .collect::<Result<Vec<_>, _>>()?,
        validation_bin_support: fields
            .unsigneds("validation_bin_support")?
            .iter()
            .map(|value| as_usize(*value))
            .collect::<Result<Vec<_>, _>>()?,
        population_stability_index_bits: fields.unsigned("population_stability_index_bits")?,
        mean_shift_classification: parse_shift_class(&fields.string("mean_shift_classification")?)?,
        standard_deviation_shift_classification: parse_shift_class(
            &fields.string("standard_deviation_shift_classification")?,
        )?,
        out_of_development_range_validation_count: as_usize(
            fields.unsigned("out_of_development_range_validation_count")?,
        )?,
        integrity_passed: fields.boolean("integrity_passed")?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    let psi = f64::from_bits(value.population_stability_index_bits);
    if value.receipt_version != FEATURE_SHIFT_RECEIPT_VERSION
        || value.feature_id.is_empty()
        || value.development_bin_support.len() != 10
        || value.validation_bin_support.len() != 10
        || value.development_bin_support.iter().sum::<usize>() != value.development_finite_count
        || value.validation_bin_support.iter().sum::<usize>() != value.validation_finite_count
        || !psi.is_finite()
        || !value.integrity_passed
        || value.receipt_digest != feature_shift_receipt_digest(&value)
    {
        return Err("micro feature partition shift receipt rejected".to_string());
    }
    Ok(value)
}

fn encode_shift(value: &MomentumMicroPartitionShiftAuditV1) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumMicroPartitionShiftAuditV1")
        .string("audit_version", &value.audit_version)
        .string("schema_id", &value.schema_id)
        .unsigned("feature_count", as_u64(value.feature_count)?)
        .unsigned(
            "development_finite_count",
            as_u64(value.development_finite_count)?,
        )
        .unsigned(
            "validation_finite_count",
            as_u64(value.validation_finite_count)?,
        )
        .unsigned("stable_feature_count", as_u64(value.stable_feature_count)?)
        .unsigned(
            "moderate_shift_feature_count",
            as_u64(value.moderate_shift_feature_count)?,
        )
        .unsigned(
            "material_shift_feature_count",
            as_u64(value.material_shift_feature_count)?,
        )
        .unsigned(
            "out_of_development_range_validation_count",
            as_u64(value.out_of_development_range_validation_count)?,
        )
        .string(
            "aggregate_classification",
            format!("{:?}", value.aggregate_classification),
        )
        .messages(
            "feature_receipts",
            value
                .feature_receipts
                .iter()
                .map(encode_feature_shift_receipt)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .string("bin_policy_digest", &value.bin_policy_digest)
        .boolean("integrity_passed", value.integrity_passed)
        .string("audit_digest", &value.audit_digest)
        .encode()
}

fn decode_shift(bytes: &[u8]) -> Result<MomentumMicroPartitionShiftAuditV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroPartitionShiftAuditV1")?;
    let aggregate_classification = parse_shift_class(&fields.string("aggregate_classification")?)?;
    let value = MomentumMicroPartitionShiftAuditV1 {
        audit_version: fields.string("audit_version")?,
        schema_id: fields.string("schema_id")?,
        feature_count: as_usize(fields.unsigned("feature_count")?)?,
        development_finite_count: as_usize(fields.unsigned("development_finite_count")?)?,
        validation_finite_count: as_usize(fields.unsigned("validation_finite_count")?)?,
        stable_feature_count: as_usize(fields.unsigned("stable_feature_count")?)?,
        moderate_shift_feature_count: as_usize(fields.unsigned("moderate_shift_feature_count")?)?,
        material_shift_feature_count: as_usize(fields.unsigned("material_shift_feature_count")?)?,
        out_of_development_range_validation_count: as_usize(
            fields.unsigned("out_of_development_range_validation_count")?,
        )?,
        aggregate_classification,
        feature_receipts: fields
            .messages("feature_receipts")?
            .iter()
            .map(|bytes| decode_feature_shift_receipt(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        bin_policy_digest: fields.string("bin_policy_digest")?,
        integrity_passed: fields.boolean("integrity_passed")?,
        audit_digest: fields.string("audit_digest")?,
    };
    fields.finish()?;
    if value.audit_version != SHIFT_VERSION
        || value.feature_count != value.feature_receipts.len()
        || value
            .feature_receipts
            .iter()
            .any(|receipt| !receipt.integrity_passed)
        || !value.integrity_passed
        || value.audit_digest != shift_digest(&value)
    {
        return Err("micro partition shift audit rejected".to_string());
    }
    Ok(value)
}

fn encode_normalizer(value: &MomentumMicroNormalizerStabilityAuditV1) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumMicroNormalizerStabilityAuditV1")
        .string("audit_version", &value.audit_version)
        .string("participant_id", &value.participant_id)
        .unsigned("refit_count", as_u64(value.refit_count)?)
        .boolean("finite_status", value.finite_status)
        .unsigneds(
            "shift_value_bits",
            &[
                value.maximum_shift.to_bits(),
                value.median_shift.to_bits(),
                value.percentile_95_shift.to_bits(),
            ],
        )
        .unsigned(
            "shift_sign_change_count",
            as_u64(value.shift_sign_change_count)?,
        )
        .boolean("partition_boundary_shift", value.partition_boundary_shift)
        .string(
            "normalizer_digest_trajectory",
            &value.normalizer_digest_trajectory,
        )
        .boolean(
            "prior_sprint_drift_preserved",
            value.prior_sprint_drift_preserved,
        )
        .string("audit_digest", &value.audit_digest)
        .encode()
}

fn decode_normalizer(bytes: &[u8]) -> Result<MomentumMicroNormalizerStabilityAuditV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroNormalizerStabilityAuditV1")?;
    let bits = fields.unsigneds("shift_value_bits")?;
    if bits.len() != 3 {
        return Err("micro normalizer shift count rejected".to_string());
    }
    let value = MomentumMicroNormalizerStabilityAuditV1 {
        audit_version: fields.string("audit_version")?,
        participant_id: fields.string("participant_id")?,
        refit_count: as_usize(fields.unsigned("refit_count")?)?,
        finite_status: fields.boolean("finite_status")?,
        maximum_shift: f64::from_bits(bits[0]),
        median_shift: f64::from_bits(bits[1]),
        percentile_95_shift: f64::from_bits(bits[2]),
        shift_sign_change_count: as_usize(fields.unsigned("shift_sign_change_count")?)?,
        partition_boundary_shift: fields.boolean("partition_boundary_shift")?,
        normalizer_digest_trajectory: fields.string("normalizer_digest_trajectory")?,
        prior_sprint_drift_preserved: fields.boolean("prior_sprint_drift_preserved")?,
        audit_digest: fields.string("audit_digest")?,
    };
    fields.finish()?;
    if value.audit_version != NORMALIZER_VERSION
        || !value.finite_status
        || !value.prior_sprint_drift_preserved
        || value.audit_digest != normalizer_digest(&value)
    {
        return Err("micro normalizer stability audit rejected".to_string());
    }
    Ok(value)
}

fn encode_compact_replay(value: &MomentumCompactMicroIntegrityReplayV1) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumCompactMicroIntegrityReplayV1")
        .string("replay_version", &value.replay_version)
        .string("partition", partition_name(value.partition))
        .unsigned("eligible_event_count", as_u64(value.eligible_event_count)?)
        .unsigned("finite_block_count", as_u64(value.finite_block_count)?)
        .unsigned(
            "missing_evidence_count",
            as_u64(value.missing_evidence_count)?,
        )
        .unsigned("partial_candle_count", as_u64(value.partial_candle_count)?)
        .unsigned(
            "zero_range_fallback_count",
            as_u64(value.zero_range_fallback_count)?,
        )
        .unsigned(
            "zero_denominator_fallback_count",
            as_u64(value.zero_denominator_fallback_count)?,
        )
        .unsigned(
            "constant_feature_count",
            as_u64(value.constant_feature_count)?,
        )
        .string("redundancy_audit_digest", &value.redundancy_audit_digest)
        .string(
            "distribution_shift_audit_digest",
            &value.distribution_shift_audit_digest,
        )
        .string("feature_schema_digest", &value.feature_schema_digest)
        .unsigned("future_access_count", as_u64(value.future_access_count)?)
        .unsigned("partial_access_count", as_u64(value.partial_access_count)?)
        .unsigned("holdout_access_count", as_u64(value.holdout_access_count)?)
        .string(
            "deterministic_replay_digest",
            &value.deterministic_replay_digest,
        )
        .encode()
}

fn decode_compact_replay(bytes: &[u8]) -> Result<MomentumCompactMicroIntegrityReplayV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumCompactMicroIntegrityReplayV1")?;
    let value = MomentumCompactMicroIntegrityReplayV1 {
        replay_version: fields.string("replay_version")?,
        partition: parse_partition(&fields.string("partition")?)?,
        eligible_event_count: as_usize(fields.unsigned("eligible_event_count")?)?,
        finite_block_count: as_usize(fields.unsigned("finite_block_count")?)?,
        missing_evidence_count: as_usize(fields.unsigned("missing_evidence_count")?)?,
        partial_candle_count: as_usize(fields.unsigned("partial_candle_count")?)?,
        zero_range_fallback_count: as_usize(fields.unsigned("zero_range_fallback_count")?)?,
        zero_denominator_fallback_count: as_usize(
            fields.unsigned("zero_denominator_fallback_count")?,
        )?,
        constant_feature_count: as_usize(fields.unsigned("constant_feature_count")?)?,
        redundancy_audit_digest: fields.string("redundancy_audit_digest")?,
        distribution_shift_audit_digest: fields.string("distribution_shift_audit_digest")?,
        feature_schema_digest: fields.string("feature_schema_digest")?,
        future_access_count: as_usize(fields.unsigned("future_access_count")?)?,
        partial_access_count: as_usize(fields.unsigned("partial_access_count")?)?,
        holdout_access_count: as_usize(fields.unsigned("holdout_access_count")?)?,
        deterministic_replay_digest: fields.string("deterministic_replay_digest")?,
    };
    fields.finish()?;
    if value.replay_version != COMPACT_REPLAY_VERSION
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.eligible_event_count != value.finite_block_count
        || value.future_access_count != 0
        || value.partial_access_count != 0
        || value.holdout_access_count != 0
        || value.deterministic_replay_digest != compact_replay_digest(&value)
    {
        return Err("compact micro integrity replay rejected".to_string());
    }
    Ok(value)
}

fn decode_feature_report(bytes: &[u8]) -> Result<MomentumMicroFeatureForensicsReportV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroFeatureForensicsReportV1")?;
    let policies = fields.messages("compact_feature_policy")?;
    let safety = fields.messages("safety_counters")?;
    if policies.len() > 1 || safety.len() != 1 {
        return Err("micro feature report nested identity rejected".to_string());
    }
    let value = MomentumMicroFeatureForensicsReportV1 {
        report_version: fields.string("report_version")?,
        run_mode: fields.string("run_mode")?,
        status: parse_feature_status(&fields.string("status")?)?,
        evidence_class: match fields.string("evidence_class")?.as_str() {
            "PostResultResearchDesignOnly" => {
                MomentumMicroChallengerDesignEvidenceClassV1::PostResultResearchDesignOnly
            }
            _ => return Err("micro feature evidence class rejected".to_string()),
        },
        registration_digest: fields.optional_string("registration_digest")?,
        label_forensics_digest: fields.optional_string("label_forensics_digest")?,
        protected_before_state_digest: fields.string("protected_before_state_digest")?,
        source_feature_audits: fields
            .messages("source_feature_audits")?
            .iter()
            .map(|bytes| decode_source_audit(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        redundancy_audits: fields
            .messages("redundancy_audits")?
            .iter()
            .map(|bytes| decode_redundancy(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        partition_shift_audits: fields
            .messages("partition_shift_audits")?
            .iter()
            .map(|bytes| decode_shift(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        normalizer_stability_audits: fields
            .messages("normalizer_stability_audits")?
            .iter()
            .map(|bytes| decode_normalizer(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        compact_feature_policy: policies
            .first()
            .map(|bytes| decode_compact_policy(bytes))
            .transpose()?,
        compact_integrity_replays: fields
            .messages("compact_integrity_replays")?
            .iter()
            .map(|bytes| decode_compact_replay(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        safety_counters: decode_safety(&safety[0])?,
        labels: fields.strings("labels")?,
        deterministic: fields.boolean("deterministic")?,
        journal_digest: fields.optional_string("journal_digest")?,
        artifacts_written: as_usize(fields.unsigned("artifacts_written")?)?,
        duplicate_artifact_count: as_usize(fields.unsigned("duplicate_artifact_count")?)?,
        feature_computation_count: as_usize(fields.unsigned("feature_computation_count")?)?,
        runtime_duration_ms: fields.unsigned("runtime_duration_ms")?,
        report_digest: fields.string("report_digest")?,
    };
    fields.finish()?;
    validate_feature_report(&value)?;
    Ok(value)
}

fn encode_feature_journal(
    value: &MomentumMicroFeatureForensicsJournalV1,
) -> Result<Vec<u8>, String> {
    if value.journal_version != FEATURE_JOURNAL_VERSION
        || value.registration_digest.is_empty()
        || value.source_audit_digests.len() != 2
        || value.redundancy_audit_digests.len() != 3
        || value.partition_shift_audit_digests.len() != 3
        || value.normalizer_stability_audit_digests.len() != 2
        || value.compact_feature_policy_digest.is_empty()
        || value.compact_integrity_replay_digests.len() != 2
        || value.holdout_access_count != 0
        || value.model_execution_count != 0
        || value.journal_digest != feature_journal_digest(value)
    {
        return Err("micro feature journal rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumMicroFeatureForensicsJournalV1")
        .string("journal_version", &value.journal_version)
        .string("registration_digest", &value.registration_digest)
        .strings("source_audit_digests", &value.source_audit_digests)
        .strings("redundancy_audit_digests", &value.redundancy_audit_digests)
        .strings(
            "partition_shift_audit_digests",
            &value.partition_shift_audit_digests,
        )
        .strings(
            "normalizer_stability_audit_digests",
            &value.normalizer_stability_audit_digests,
        )
        .string(
            "compact_feature_policy_digest",
            &value.compact_feature_policy_digest,
        )
        .strings(
            "compact_integrity_replay_digests",
            &value.compact_integrity_replay_digests,
        )
        .unsigned("holdout_access_count", as_u64(value.holdout_access_count)?)
        .unsigned(
            "model_execution_count",
            as_u64(value.model_execution_count)?,
        )
        .string("journal_digest", &value.journal_digest)
        .encode()
}

fn decode_feature_journal(bytes: &[u8]) -> Result<MomentumMicroFeatureForensicsJournalV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroFeatureForensicsJournalV1")?;
    let value = MomentumMicroFeatureForensicsJournalV1 {
        journal_version: fields.string("journal_version")?,
        registration_digest: fields.string("registration_digest")?,
        source_audit_digests: fields.strings("source_audit_digests")?,
        redundancy_audit_digests: fields.strings("redundancy_audit_digests")?,
        partition_shift_audit_digests: fields.strings("partition_shift_audit_digests")?,
        normalizer_stability_audit_digests: fields.strings("normalizer_stability_audit_digests")?,
        compact_feature_policy_digest: fields.string("compact_feature_policy_digest")?,
        compact_integrity_replay_digests: fields.strings("compact_integrity_replay_digests")?,
        holdout_access_count: as_usize(fields.unsigned("holdout_access_count")?)?,
        model_execution_count: as_usize(fields.unsigned("model_execution_count")?)?,
        journal_digest: fields.string("journal_digest")?,
    };
    fields.finish()?;
    let encoded = encode_feature_journal(&value)?;
    if encoded.is_empty() {
        return Err("micro feature journal reopen rejected".to_string());
    }
    Ok(value)
}

fn encode_screening_journal(value: &MomentumMicroScreeningJournalV1) -> Result<Vec<u8>, String> {
    if value.journal_version != SCREENING_JOURNAL_VERSION
        || value.registration_digest.is_empty()
        || value.task_registration_digests.len() != 2
        || value.participant_registration_digests.len() != 10
        || value.screening_gate_digest.is_empty()
        || value.model_execution_count != 0
        || value.holdout_execution_count != 0
        || value.journal_digest != screening_journal_digest(value)
    {
        return Err("micro screening journal rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumMicroScreeningJournalV1")
        .string("journal_version", &value.journal_version)
        .string("registration_digest", &value.registration_digest)
        .strings(
            "task_registration_digests",
            &value.task_registration_digests,
        )
        .strings(
            "participant_registration_digests",
            &value.participant_registration_digests,
        )
        .string("screening_gate_digest", &value.screening_gate_digest)
        .unsigned(
            "model_execution_count",
            as_u64(value.model_execution_count)?,
        )
        .unsigned(
            "holdout_execution_count",
            as_u64(value.holdout_execution_count)?,
        )
        .string("journal_digest", &value.journal_digest)
        .encode()
}

fn decode_screening_journal(bytes: &[u8]) -> Result<MomentumMicroScreeningJournalV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroScreeningJournalV1")?;
    let value = MomentumMicroScreeningJournalV1 {
        journal_version: fields.string("journal_version")?,
        registration_digest: fields.string("registration_digest")?,
        task_registration_digests: fields.strings("task_registration_digests")?,
        participant_registration_digests: fields.strings("participant_registration_digests")?,
        screening_gate_digest: fields.string("screening_gate_digest")?,
        model_execution_count: as_usize(fields.unsigned("model_execution_count")?)?,
        holdout_execution_count: as_usize(fields.unsigned("holdout_execution_count")?)?,
        journal_digest: fields.string("journal_digest")?,
    };
    fields.finish()?;
    let encoded = encode_screening_journal(&value)?;
    if encoded.is_empty() {
        return Err("micro screening journal reopen rejected".to_string());
    }
    Ok(value)
}

fn persist_one(
    category: &str,
    digest: &str,
    bytes: &[u8],
    decode_digest: impl Fn(&[u8]) -> Result<String, String>,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &Path::new(ROOT).join(category).join(format!("{digest}.pb")),
        bytes,
        digest,
        decode_digest,
    )
}

fn add_counts(total: &mut (usize, usize), next: (usize, usize)) {
    total.0 += next.0;
    total.1 += next.1;
}

fn read_feature_registration() -> Result<Option<MomentumMicroFeatureForensicsRegistrationV1>, String>
{
    read_single(
        &Path::new(ROOT).join("feature_registrations"),
        decode_feature_registration,
    )
}

pub fn read_momentum_micro_feature_forensics_report_v1()
-> Result<Option<MomentumMicroFeatureForensicsReportV1>, String> {
    read_single(
        &Path::new(ROOT).join("feature_final_reports"),
        decode_feature_report,
    )
}

fn derive_feature_registration(
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> Result<MomentumMicroFeatureForensicsRegistrationV1, String> {
    let label = read_momentum_micro_label_forensics_report_v1()?
        .ok_or_else(|| "micro label forensics must complete first".to_string())?;
    if label.status != MomentumMicroLabelForensicsStatusV1::Complete
        || label.safety_counters.holdout_label_reads != 0
        || label.safety_counters.model_fits != 0
        || label.protected_before_state_digest != protected.state_digest
    {
        return Err("micro label prerequisite rejected".to_string());
    }
    let header = load_momentum_qualified_diagnostic_source_header_v1()?;
    let mut value = MomentumMicroFeatureForensicsRegistrationV1 {
        registration_version: FEATURE_REGISTRATION_VERSION.to_string(),
        source_replay_digest: header.replay_journal_digest,
        source_diagnostic_digest: label
            .source_diagnostic_digest
            .clone()
            .ok_or_else(|| "micro diagnostic identity unavailable".to_string())?,
        label_forensics_digest: label.report_digest,
        protected_before_state_digest: protected.state_digest.clone(),
        audited_participants: vec!["Q1".to_string(), "Q2".to_string()],
        audited_timeframes: vec![
            MomentumHistoricalTimeframeV1::Minute1,
            MomentumHistoricalTimeframeV1::Minute3,
            MomentumHistoricalTimeframeV1::Minute5,
            MomentumHistoricalTimeframeV1::Minute10,
        ],
        finite_policy_digest: stable_hash_string(
            "micro-feature-finite-v1:all-source-values-and-normalizers-finite",
        ),
        constant_feature_policy_digest: stable_hash_string(
            "micro-feature-constant-v1:development-range-absolute<=1e-12",
        ),
        redundancy_policy_digest: stable_hash_string(
            "micro-feature-redundancy-v1:development-pearson-absolute>=0.98:frozen-validation-confirmation",
        ),
        partition_shift_policy_digest: stable_hash_string(
            "micro-feature-shift-v1:ten-development-equal-frequency-bins:psi:stable<0.10:moderate<0.25",
        ),
        normalizer_drift_policy_digest: stable_hash_string(
            "micro-normalizer-drift-v1:all-daily-refits:finite:max:median:p95:sign-change:partition-boundary",
        ),
        label_based_feature_selection_forbidden: true,
        validation_selected_feature_selection_forbidden: true,
        holdout_access_forbidden: true,
        registration_digest: String::new(),
    };
    value.registration_digest = feature_registration_digest(&value);
    validate_feature_registration(&value)?;
    Ok(value)
}

fn compact_policy() -> Result<MomentumCompactMicroFeaturePolicyV1, String> {
    let per_timeframe_feature_ids = [
        "log_return_1",
        "log_return_3",
        "log_return_6",
        "log_return_12",
        "realized_std_6",
        "realized_std_12",
        "realized_std_16",
        "log_high_low_range",
        "body_over_range",
        "upper_wick_over_range",
        "lower_wick_over_range",
        "close_location_in_range",
        "log_volume_change_1",
        "volume_zscore_16",
        "trade_value_zscore_16",
        "normalized_close_slope_16",
    ]
    .map(str::to_string)
    .to_vec();
    let cross_timeframe_feature_ids = [
        "return_sign_agreement_count",
        "latest_return_dispersion",
        "realized_volatility_1m_over_10m",
        "realized_volatility_3m_over_10m",
        "normalized_volume_1m_over_10m",
    ]
    .map(str::to_string)
    .to_vec();
    let included_timeframes = vec![
        MomentumHistoricalTimeframeV1::Minute1,
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
    ];
    let feature_order = included_timeframes
        .iter()
        .flat_map(|timeframe| {
            per_timeframe_feature_ids
                .iter()
                .map(move |feature| format!("{}:{feature}", timeframe_name(*timeframe)))
        })
        .chain(cross_timeframe_feature_ids.iter().cloned())
        .collect::<Vec<_>>();
    let mut value = MomentumCompactMicroFeaturePolicyV1 {
        policy_version: COMPACT_POLICY_VERSION.to_string(),
        included_timeframes,
        per_timeframe_feature_ids,
        cross_timeframe_feature_ids,
        context_length: COMPACT_CONTEXT_LENGTH,
        zero_range_policy_digest: stable_hash_string(
            "compact-micro-zero-range-v1:shape-ratios-and-range-log-equal-zero",
        ),
        zero_denominator_policy_digest: stable_hash_string(
            "compact-micro-zero-denominator-v1:cross-ratio-equal-zero",
        ),
        feature_order_digest: stable_hash_string(&format!(
            "compact-micro-feature-order-v1:{feature_order:?}"
        )),
        schema_digest: String::new(),
        target_selected_features: false,
        validation_selected_features: false,
        holdout_selected_features: false,
    };
    value.schema_digest = compact_schema_digest(&value);
    validate_compact_policy(&value)?;
    Ok(value)
}

fn checked_f32(value: f64) -> Result<f32, String> {
    let converted = value as f32;
    if !value.is_finite() || !converted.is_finite() {
        return Err("micro existing feature conversion rejected".to_string());
    }
    Ok(converted)
}

fn existing_feature_map(
    rows: &[MomentumQualifiedReplayCandleEvidenceV1],
    config: &MomentumFeatureConfigV0,
) -> Result<BTreeMap<usize, Vec<f64>>, String> {
    let candles = rows
        .iter()
        .map(|row| {
            Ok(MomentumCandleV0 {
                timestamp: i64::try_from(row.close_exclusive_timestamp_ms)
                    .map_err(|_| "micro feature timestamp rejected".to_string())?,
                open: checked_f32(row.open)?,
                high: checked_f32(row.high)?,
                low: checked_f32(row.low)?,
                close: checked_f32(row.close)?,
                volume: checked_f32(row.volume)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(build_momentum_features_v0(&candles, config)
        .map_err(|_| "micro existing feature extraction rejected".to_string())?
        .into_iter()
        .map(|row| {
            (
                row.source_index,
                row.values.into_iter().map(f64::from).collect::<Vec<_>>(),
            )
        })
        .collect())
}

fn existing_matrix(
    source: &MomentumQualifiedDiagnosticSourceV1,
    views: &BTreeMap<MomentumHistoricalTimeframeV1, Vec<MomentumQualifiedReplayCandleEvidenceV1>>,
    participant_id: &str,
    timeframes: &[MomentumHistoricalTimeframeV1],
) -> Result<FeatureMatrix, String> {
    let config = MomentumFeatureConfigV0::default();
    let feature_names = config.feature_names();
    let maps = timeframes
        .iter()
        .map(|timeframe| {
            let rows = views
                .get(timeframe)
                .ok_or_else(|| "micro source timeframe unavailable".to_string())?;
            Ok((*timeframe, existing_feature_map(rows, &config)?))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let feature_ids = timeframes
        .iter()
        .flat_map(|timeframe| {
            feature_names
                .iter()
                .map(move |name| format!("{}:{name}", timeframe_name(*timeframe)))
        })
        .collect::<Vec<_>>();
    let mut development = Vec::new();
    let mut validation = Vec::new();
    for event in &source.events {
        if event.partition == MomentumReplayPartitionV1::SealedHoldout {
            return Err("micro source feature holdout event rejected".to_string());
        }
        let mut values = Vec::new();
        for timeframe in timeframes {
            let rows = views
                .get(timeframe)
                .ok_or_else(|| "micro source rows unavailable".to_string())?;
            let end = rows.partition_point(|row| {
                row.close_exclusive_timestamp_ms <= event.prediction_timestamp_ms
            });
            if end == 0
                || rows[..end].last().is_none_or(|row| row.missing_evidence)
                || rows[..end].last().is_some_and(|row| {
                    row.close_exclusive_timestamp_ms > event.prediction_timestamp_ms
                })
            {
                return Err("micro source feature candle completeness rejected".to_string());
            }
            values.extend(
                maps.get(timeframe)
                    .and_then(|map| map.get(&(end - 1)))
                    .ok_or_else(|| "micro source feature row unavailable".to_string())?
                    .iter()
                    .copied(),
            );
        }
        if values.len() != feature_ids.len() || values.iter().any(|value| !value.is_finite()) {
            return Err(format!(
                "{participant_id} source feature integrity rejected"
            ));
        }
        match event.partition {
            MomentumReplayPartitionV1::Development => development.push(values),
            MomentumReplayPartitionV1::Validation => validation.push(values),
            MomentumReplayPartitionV1::SealedHoldout => unreachable!(),
        }
    }
    if development.is_empty() || validation.is_empty() {
        return Err("micro source feature partition support unavailable".to_string());
    }
    Ok(FeatureMatrix {
        feature_ids,
        development,
        validation,
    })
}

fn constant_count(rows: &[Vec<f64>]) -> Result<usize, String> {
    let width = rows
        .first()
        .map(Vec::len)
        .ok_or_else(|| "micro constant audit empty".to_string())?;
    Ok((0..width)
        .filter(|column| {
            let minimum = rows
                .iter()
                .map(|row| row[*column])
                .fold(f64::INFINITY, f64::min);
            let maximum = rows
                .iter()
                .map(|row| row[*column])
                .fold(f64::NEG_INFINITY, f64::max);
            maximum - minimum <= EPSILON
        })
        .count())
}

fn pearson(rows: &[Vec<f64>], left: usize, right: usize) -> Option<f64> {
    if rows.len() < 2 {
        return None;
    }
    let mean_left = rows.iter().map(|row| row[left]).sum::<f64>() / rows.len() as f64;
    let mean_right = rows.iter().map(|row| row[right]).sum::<f64>() / rows.len() as f64;
    let covariance = rows
        .iter()
        .map(|row| (row[left] - mean_left) * (row[right] - mean_right))
        .sum::<f64>();
    let left_scale = rows
        .iter()
        .map(|row| (row[left] - mean_left).powi(2))
        .sum::<f64>()
        .sqrt();
    let right_scale = rows
        .iter()
        .map(|row| (row[right] - mean_right).powi(2))
        .sum::<f64>()
        .sqrt();
    if left_scale <= EPSILON || right_scale <= EPSILON {
        None
    } else {
        let value = covariance / (left_scale * right_scale);
        value.is_finite().then_some(value)
    }
}

fn redundancy_audit(schema_id: &str, matrix: &FeatureMatrix) -> MomentumMicroRedundancyAuditV1 {
    let width = matrix.feature_ids.len();
    let mut pairs = Vec::new();
    for left in 0..width {
        for right in left + 1..width {
            if pearson(&matrix.development, left, right)
                .is_some_and(|value| value.abs() >= REDUNDANCY_THRESHOLD)
            {
                pairs.push((left, right));
            }
        }
    }
    let mut parents = (0..width).collect::<Vec<_>>();
    fn root(parents: &mut [usize], mut value: usize) -> usize {
        while parents[value] != value {
            value = parents[value];
        }
        value
    }
    for (left, right) in &pairs {
        let left_root = root(&mut parents, *left);
        let right_root = root(&mut parents, *right);
        if left_root != right_root {
            parents[right_root] = left_root;
        }
    }
    let mut groups = BTreeMap::<usize, usize>::new();
    for index in 0..width {
        let group = root(&mut parents, index);
        *groups.entry(group).or_default() += 1;
    }
    let correlated_groups = groups
        .values()
        .filter(|size| **size > 1)
        .copied()
        .collect::<Vec<_>>();
    let validation_confirmed = pairs
        .iter()
        .filter(|(left, right)| {
            pearson(&matrix.validation, *left, *right)
                .is_some_and(|value| value.abs() >= REDUNDANCY_THRESHOLD)
        })
        .count();
    let cross = pairs
        .iter()
        .filter(|(left, right)| {
            matrix.feature_ids[*left].split(':').next()
                != matrix.feature_ids[*right].split(':').next()
        })
        .count();
    let mut value = MomentumMicroRedundancyAuditV1 {
        audit_version: REDUNDANCY_VERSION.to_string(),
        schema_id: schema_id.to_string(),
        absolute_pearson_threshold: REDUNDANCY_THRESHOLD,
        correlated_feature_pair_count: pairs.len(),
        correlated_feature_group_count: correlated_groups.len(),
        maximum_group_size: correlated_groups.into_iter().max().unwrap_or(1),
        cross_timeframe_duplicate_pattern_count: cross,
        rank_deficiency: if pairs.is_empty() {
            MomentumMicroRankDeficiencyV1::FullRankByFixedAudit
        } else {
            MomentumMicroRankDeficiencyV1::CorrelatedGroupsObserved
        },
        development_only_redundancy_identity: stable_hash_string(&format!(
            "micro-redundancy-development-v1:{schema_id}:{pairs:?}"
        )),
        validation_confirmation_identity: stable_hash_string(&format!(
            "micro-redundancy-validation-v1:{schema_id}:{validation_confirmed}"
        )),
        validation_modified_policy: false,
        audit_digest: String::new(),
    };
    value.audit_digest = redundancy_digest(&value);
    value
}

fn percentile(sorted: &[f64], fraction: f64) -> Result<f64, String> {
    if sorted.is_empty() {
        return Err("micro design percentile empty".to_string());
    }
    let index = ((sorted.len() - 1) as f64 * fraction).round() as usize;
    Ok(sorted[index])
}

fn bin_policy(
    schema_id: &str,
    matrix: &FeatureMatrix,
) -> Result<MomentumMicroFeatureDriftBinPolicyV1, String> {
    let mut boundary_offsets = vec![0_u64];
    let mut private_boundary_bits = Vec::new();
    for column in 0..matrix.feature_ids.len() {
        let mut values = matrix
            .development
            .iter()
            .map(|row| row[column])
            .collect::<Vec<_>>();
        values.sort_by(f64::total_cmp);
        for index in 1..10 {
            private_boundary_bits.push(percentile(&values, index as f64 / 10.0)?.to_bits());
        }
        boundary_offsets.push(as_u64(private_boundary_bits.len())?);
    }
    let mut value = MomentumMicroFeatureDriftBinPolicyV1 {
        policy_version: BIN_POLICY_VERSION.to_string(),
        schema_id: schema_id.to_string(),
        feature_ids: matrix.feature_ids.clone(),
        boundary_offsets,
        private_boundary_bits,
        development_only: true,
        validation_access_count_before_persist: 0,
        policy_digest: String::new(),
    };
    value.policy_digest = bin_policy_digest(&value);
    validate_bin_policy(&value)?;
    Ok(value)
}

fn validate_bin_policy(value: &MomentumMicroFeatureDriftBinPolicyV1) -> Result<(), String> {
    let offsets_valid = value.boundary_offsets.len() == value.feature_ids.len() + 1
        && value.boundary_offsets.first() == Some(&0)
        && value
            .boundary_offsets
            .windows(2)
            .all(|pair| pair[1] >= pair[0] && pair[1] - pair[0] == 9)
        && value
            .boundary_offsets
            .last()
            .is_some_and(|offset| *offset as usize == value.private_boundary_bits.len());
    let boundaries_valid = value.private_boundary_bits.chunks_exact(9).all(|chunk| {
        let values = chunk
            .iter()
            .map(|bits| f64::from_bits(*bits))
            .collect::<Vec<_>>();
        values.iter().all(|value| value.is_finite())
            && values.windows(2).all(|pair| pair[1] >= pair[0])
    });
    if value.policy_version != BIN_POLICY_VERSION
        || value.schema_id.is_empty()
        || value.feature_ids.is_empty()
        || !offsets_valid
        || !boundaries_valid
        || !value.development_only
        || value.validation_access_count_before_persist != 0
        || value.policy_digest != bin_policy_digest(value)
    {
        return Err("micro drift bin policy rejected".to_string());
    }
    Ok(())
}

fn encode_bin_policy(value: &MomentumMicroFeatureDriftBinPolicyV1) -> Result<Vec<u8>, String> {
    validate_bin_policy(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroFeatureDriftBinPolicyV1")
        .string("policy_version", &value.policy_version)
        .string("schema_id", &value.schema_id)
        .strings("feature_ids", &value.feature_ids)
        .unsigneds("boundary_offsets", &value.boundary_offsets)
        .unsigneds("private_boundary_bits", &value.private_boundary_bits)
        .boolean("development_only", value.development_only)
        .unsigned(
            "validation_access_count_before_persist",
            as_u64(value.validation_access_count_before_persist)?,
        )
        .string("policy_digest", &value.policy_digest)
        .encode()
}

fn decode_bin_policy(bytes: &[u8]) -> Result<MomentumMicroFeatureDriftBinPolicyV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroFeatureDriftBinPolicyV1")?;
    let value = MomentumMicroFeatureDriftBinPolicyV1 {
        policy_version: fields.string("policy_version")?,
        schema_id: fields.string("schema_id")?,
        feature_ids: fields.strings("feature_ids")?,
        boundary_offsets: fields.unsigneds("boundary_offsets")?,
        private_boundary_bits: fields.unsigneds("private_boundary_bits")?,
        development_only: fields.boolean("development_only")?,
        validation_access_count_before_persist: as_usize(
            fields.unsigned("validation_access_count_before_persist")?,
        )?,
        policy_digest: fields.string("policy_digest")?,
    };
    fields.finish()?;
    validate_bin_policy(&value)?;
    Ok(value)
}

fn feature_boundaries(
    policy: &MomentumMicroFeatureDriftBinPolicyV1,
    column: usize,
) -> Result<Vec<f64>, String> {
    let start = as_usize(
        *policy
            .boundary_offsets
            .get(column)
            .ok_or_else(|| "micro bin offset unavailable".to_string())?,
    )?;
    let end = as_usize(
        *policy
            .boundary_offsets
            .get(column + 1)
            .ok_or_else(|| "micro bin end unavailable".to_string())?,
    )?;
    Ok(policy.private_boundary_bits[start..end]
        .iter()
        .map(|bits| f64::from_bits(*bits))
        .collect())
}

fn bin_index(value: f64, boundaries: &[f64]) -> usize {
    boundaries.partition_point(|boundary| value > *boundary)
}

fn classify_shift(value: f64) -> MomentumMicroPartitionShiftClassV1 {
    if value.is_nan() {
        MomentumMicroPartitionShiftClassV1::IntegrityFailure
    } else if value < DRIFT_STABLE_THRESHOLD {
        MomentumMicroPartitionShiftClassV1::Stable
    } else if value < DRIFT_MODERATE_THRESHOLD {
        MomentumMicroPartitionShiftClassV1::ModerateShift
    } else {
        MomentumMicroPartitionShiftClassV1::MaterialShift
    }
}

fn shift_audit(
    schema_id: &str,
    matrix: &FeatureMatrix,
    policy: &MomentumMicroFeatureDriftBinPolicyV1,
) -> Result<MomentumMicroPartitionShiftAuditV1, String> {
    let width = matrix.feature_ids.len();
    let mut stable = 0;
    let mut moderate = 0;
    let mut material = 0;
    let mut out_of_range = 0;
    let mut feature_receipts = Vec::with_capacity(width);
    for column in 0..width {
        let boundaries = feature_boundaries(policy, column)?;
        let mut development_bins = [0_usize; 10];
        let mut validation_bins = [0_usize; 10];
        for row in &matrix.development {
            development_bins[bin_index(row[column], &boundaries)] += 1;
        }
        let development_min = matrix
            .development
            .iter()
            .map(|row| row[column])
            .fold(f64::INFINITY, f64::min);
        let development_max = matrix
            .development
            .iter()
            .map(|row| row[column])
            .fold(f64::NEG_INFINITY, f64::max);
        let mut feature_out_of_range = 0;
        for row in &matrix.validation {
            validation_bins[bin_index(row[column], &boundaries)] += 1;
            feature_out_of_range +=
                usize::from(row[column] < development_min || row[column] > development_max);
        }
        out_of_range += feature_out_of_range;
        let psi = (0..10)
            .map(|index| {
                let dev = (development_bins[index] as f64 + EPSILON)
                    / (matrix.development.len() as f64 + 10.0 * EPSILON);
                let val = (validation_bins[index] as f64 + EPSILON)
                    / (matrix.validation.len() as f64 + 10.0 * EPSILON);
                (val - dev) * (val / dev).ln()
            })
            .sum::<f64>();
        let psi_classification = classify_shift(psi);
        match psi_classification {
            MomentumMicroPartitionShiftClassV1::Stable => stable += 1,
            MomentumMicroPartitionShiftClassV1::ModerateShift => moderate += 1,
            MomentumMicroPartitionShiftClassV1::MaterialShift => material += 1,
            MomentumMicroPartitionShiftClassV1::IntegrityFailure => {
                return Err("micro feature PSI integrity rejected".to_string());
            }
        }
        let development_values = matrix
            .development
            .iter()
            .map(|row| row[column])
            .collect::<Vec<_>>();
        let validation_values = matrix
            .validation
            .iter()
            .map(|row| row[column])
            .collect::<Vec<_>>();
        let (development_mean, development_std) = mean_std(&development_values)?;
        let (validation_mean, validation_std) = mean_std(&validation_values)?;
        let mean_shift = if development_std <= EPSILON {
            if (validation_mean - development_mean).abs() <= EPSILON {
                0.0
            } else {
                f64::INFINITY
            }
        } else {
            (validation_mean - development_mean).abs() / development_std
        };
        let standard_deviation_shift = if development_std <= EPSILON {
            if validation_std <= EPSILON {
                0.0
            } else {
                f64::INFINITY
            }
        } else {
            (validation_std / development_std - 1.0).abs()
        };
        let mut receipt = MomentumMicroFeaturePartitionShiftReceiptV1 {
            receipt_version: FEATURE_SHIFT_RECEIPT_VERSION.to_string(),
            feature_id: matrix.feature_ids[column].clone(),
            development_finite_count: development_values.len(),
            validation_finite_count: validation_values.len(),
            development_bin_support: development_bins.to_vec(),
            validation_bin_support: validation_bins.to_vec(),
            population_stability_index_bits: psi.to_bits(),
            mean_shift_classification: classify_shift(mean_shift),
            standard_deviation_shift_classification: classify_shift(standard_deviation_shift),
            out_of_development_range_validation_count: feature_out_of_range,
            integrity_passed: psi.is_finite()
                && development_values.iter().all(|value| value.is_finite())
                && validation_values.iter().all(|value| value.is_finite()),
            receipt_digest: String::new(),
        };
        receipt.receipt_digest = feature_shift_receipt_digest(&receipt);
        feature_receipts.push(receipt);
    }
    let aggregate_classification = if material > 0 {
        MomentumMicroPartitionShiftClassV1::MaterialShift
    } else if moderate > 0 {
        MomentumMicroPartitionShiftClassV1::ModerateShift
    } else {
        MomentumMicroPartitionShiftClassV1::Stable
    };
    let mut value = MomentumMicroPartitionShiftAuditV1 {
        audit_version: SHIFT_VERSION.to_string(),
        schema_id: schema_id.to_string(),
        feature_count: width,
        development_finite_count: matrix.development.len() * width,
        validation_finite_count: matrix.validation.len() * width,
        stable_feature_count: stable,
        moderate_shift_feature_count: moderate,
        material_shift_feature_count: material,
        out_of_development_range_validation_count: out_of_range,
        aggregate_classification,
        feature_receipts,
        bin_policy_digest: policy.policy_digest.clone(),
        integrity_passed: matrix
            .development
            .iter()
            .chain(&matrix.validation)
            .flatten()
            .all(|value| value.is_finite()),
        audit_digest: String::new(),
    };
    value.audit_digest = shift_digest(&value);
    Ok(value)
}

fn source_audit(
    participant_id: &str,
    timeframes: Vec<MomentumHistoricalTimeframeV1>,
    matrix: &FeatureMatrix,
    normalizer_identity: String,
    normalizer_finite: bool,
) -> Result<MomentumMicroSourceFeatureAuditV1, String> {
    let duplicate =
        matrix.feature_ids.len() - matrix.feature_ids.iter().collect::<BTreeSet<_>>().len();
    let mut value = MomentumMicroSourceFeatureAuditV1 {
        audit_version: SOURCE_AUDIT_VERSION.to_string(),
        participant_id: participant_id.to_string(),
        feature_dimension: matrix.feature_ids.len(),
        feature_order_digest: stable_hash_string(&format!(
            "micro-source-feature-order-v1:{participant_id}:{:?}",
            matrix.feature_ids
        )),
        source_timeframes: timeframes,
        finite_value_count: matrix
            .development
            .iter()
            .chain(&matrix.validation)
            .flatten()
            .filter(|value| value.is_finite())
            .count(),
        constant_or_near_constant_count: constant_count(&matrix.development)?,
        duplicate_semantic_feature_count: duplicate,
        normalizer_identity,
        normalizer_finite,
        development_availability_count: matrix.development.len(),
        validation_availability_count: matrix.validation.len(),
        source_candle_complete: true,
        holdout_access_count: 0,
        audit_digest: String::new(),
    };
    value.audit_digest = source_audit_digest(&value);
    Ok(value)
}

fn normalizer_audit(
    participant_id: &str,
    source: &MomentumQualifiedDiagnosticSourceV1,
    profile_indices: &[usize],
) -> Result<MomentumMicroNormalizerStabilityAuditV1, String> {
    let mut refits = source.refits.clone();
    refits.sort_by_key(|value| {
        (
            match value.partition {
                MomentumReplayPartitionV1::Development => 0,
                MomentumReplayPartitionV1::Validation => 1,
                MomentumReplayPartitionV1::SealedHoldout => 2,
            },
            value.utc_day_boundary_ms,
        )
    });
    let profiles = refits
        .iter()
        .map(|refit| {
            profile_indices
                .iter()
                .flat_map(|index| {
                    refit
                        .normalizer_profiles
                        .get(*index)
                        .into_iter()
                        .flatten()
                        .copied()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    if profiles.len() < 2
        || profiles
            .iter()
            .any(|profile| profile.is_empty() || profile.iter().any(|value| !value.is_finite()))
    {
        return Err("micro normalizer profile rejected".to_string());
    }
    let signed = profiles
        .windows(2)
        .map(|pair| {
            pair[1]
                .iter()
                .zip(&pair[0])
                .map(|(next, prior)| next - prior)
                .sum::<f64>()
                / pair[0].len() as f64
        })
        .collect::<Vec<_>>();
    let shifts_in_order = profiles
        .windows(2)
        .map(|pair| {
            (pair[1]
                .iter()
                .zip(&pair[0])
                .map(|(next, prior)| (next - prior).powi(2))
                .sum::<f64>()
                / pair[0].len() as f64)
                .sqrt()
        })
        .collect::<Vec<_>>();
    let boundary_index = refits
        .windows(2)
        .position(|pair| pair[0].partition != pair[1].partition);
    let boundary_shift = boundary_index.is_some_and(|index| {
        shifts_in_order
            .get(index)
            .is_some_and(|shift| *shift > DRIFT_MODERATE_THRESHOLD)
    });
    let mut shifts = shifts_in_order;
    shifts.sort_by(f64::total_cmp);
    let mut value = MomentumMicroNormalizerStabilityAuditV1 {
        audit_version: NORMALIZER_VERSION.to_string(),
        participant_id: participant_id.to_string(),
        refit_count: refits.len(),
        finite_status: true,
        maximum_shift: *shifts.last().unwrap_or(&0.0),
        median_shift: percentile(&shifts, 0.50)?,
        percentile_95_shift: percentile(&shifts, 0.95)?,
        shift_sign_change_count: signed
            .windows(2)
            .filter(|pair| pair[0].is_sign_positive() != pair[1].is_sign_positive())
            .count(),
        partition_boundary_shift: boundary_shift,
        normalizer_digest_trajectory: stable_hash_string(&format!(
            "micro-normalizer-trajectory-v1:{participant_id}:{:?}",
            refits
                .iter()
                .map(|value| &value.normalizer_digests)
                .collect::<Vec<_>>()
        )),
        prior_sprint_drift_preserved: true,
        audit_digest: String::new(),
    };
    value.audit_digest = normalizer_digest(&value);
    Ok(value)
}

fn mean_std(values: &[f64]) -> Result<(f64, f64), String> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err("compact feature statistic rejected".to_string());
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64;
    Ok((mean, variance.sqrt()))
}

fn guarded_ratio(numerator: f64, denominator: f64, fallbacks: &mut usize) -> f64 {
    if denominator.abs() <= EPSILON {
        *fallbacks += 1;
        0.0
    } else {
        numerator / denominator
    }
}

fn timeframe_compact_features(
    rows: &[MomentumQualifiedReplayCandleEvidenceV1],
) -> Result<(Vec<f64>, usize), String> {
    if rows.len() != COMPACT_CONTEXT_LENGTH
        || rows.iter().any(|row| {
            row.missing_evidence
                || [
                    row.open,
                    row.high,
                    row.low,
                    row.close,
                    row.volume,
                    row.trade_value,
                ]
                .into_iter()
                .any(|value| !value.is_finite())
                || row.open <= 0.0
                || row.high <= 0.0
                || row.low <= 0.0
                || row.close <= 0.0
                || row.volume < 0.0
                || row.trade_value < 0.0
        })
    {
        return Err("compact timeframe context rejected".to_string());
    }
    let latest = rows
        .last()
        .ok_or_else(|| "compact latest candle unavailable".to_string())?;
    let returns = rows
        .windows(2)
        .map(|pair| (pair[1].close / pair[0].close).ln())
        .collect::<Vec<_>>();
    let mut values = [1_usize, 3, 6, 12]
        .into_iter()
        .map(|lookback| (latest.close / rows[rows.len() - 1 - lookback].close).ln())
        .collect::<Vec<_>>();
    for lookback in [6_usize, 12, 16] {
        values.push(mean_std(&returns[returns.len() - lookback..])?.1);
    }
    let range = latest.high - latest.low;
    let mut zero_range = 0;
    if range.abs() <= EPSILON {
        zero_range = 1;
        values.extend([0.0; 5]);
    } else {
        values.push((latest.high / latest.low).ln());
        values.push((latest.close - latest.open) / range);
        values.push((latest.high - latest.open.max(latest.close)) / range);
        values.push((latest.open.min(latest.close) - latest.low) / range);
        values.push((latest.close - latest.low) / range);
    }
    let previous = &rows[rows.len() - 2];
    values.push(((latest.volume + EPSILON) / (previous.volume + EPSILON)).ln());
    let volumes = rows[rows.len() - 16..]
        .iter()
        .map(|row| row.volume)
        .collect::<Vec<_>>();
    let trade_values = rows[rows.len() - 16..]
        .iter()
        .map(|row| row.trade_value)
        .collect::<Vec<_>>();
    let (volume_mean, volume_std) = mean_std(&volumes)?;
    let (trade_mean, trade_std) = mean_std(&trade_values)?;
    values.push(if volume_std <= EPSILON {
        0.0
    } else {
        (latest.volume - volume_mean) / volume_std
    });
    values.push(if trade_std <= EPSILON {
        0.0
    } else {
        (latest.trade_value - trade_mean) / trade_std
    });
    let closes = rows[rows.len() - 16..]
        .iter()
        .map(|row| row.close)
        .collect::<Vec<_>>();
    let close_mean = closes.iter().sum::<f64>() / closes.len() as f64;
    let normalized = closes
        .iter()
        .map(|close| close / close_mean - 1.0)
        .collect::<Vec<_>>();
    let x_mean = 7.5_f64;
    let numerator = normalized
        .iter()
        .enumerate()
        .map(|(index, value)| (index as f64 - x_mean) * value)
        .sum::<f64>();
    let denominator = (0..16)
        .map(|index| (index as f64 - x_mean).powi(2))
        .sum::<f64>();
    values.push(numerator / denominator);
    if values.len() != 16 || values.iter().any(|value| !value.is_finite()) {
        return Err("compact timeframe feature integrity rejected".to_string());
    }
    Ok((values, zero_range))
}

fn compact_event(
    policy: &MomentumCompactMicroFeaturePolicyV1,
    views: &BTreeMap<MomentumHistoricalTimeframeV1, Vec<MomentumQualifiedReplayCandleEvidenceV1>>,
    timestamp_ms: u64,
) -> Result<CompactEventResult, String> {
    let mut per_timeframe = Vec::new();
    let mut zero_range_fallbacks = 0;
    for timeframe in &policy.included_timeframes {
        let rows = views
            .get(timeframe)
            .ok_or_else(|| "compact source timeframe unavailable".to_string())?;
        let end = rows.partition_point(|row| row.close_exclusive_timestamp_ms <= timestamp_ms);
        if end < COMPACT_CONTEXT_LENGTH {
            return Err("compact context support unavailable".to_string());
        }
        let context = &rows[end - COMPACT_CONTEXT_LENGTH..end];
        if context
            .iter()
            .any(|row| row.close_exclusive_timestamp_ms > timestamp_ms)
        {
            return Err("compact partial candle access rejected".to_string());
        }
        let (features, fallbacks) = timeframe_compact_features(context)?;
        zero_range_fallbacks += fallbacks;
        per_timeframe.push(features);
    }
    let latest_returns = per_timeframe
        .iter()
        .map(|values| values[0])
        .collect::<Vec<_>>();
    let positives = latest_returns.iter().filter(|value| **value > 0.0).count();
    let negatives = latest_returns.iter().filter(|value| **value < 0.0).count();
    let agreement = positives.max(negatives) as f64;
    let dispersion = mean_std(&latest_returns)?.1;
    let mut zero_denominator_fallbacks = 0;
    let vol_1m_over_10m = guarded_ratio(
        per_timeframe[0][6],
        per_timeframe[3][6],
        &mut zero_denominator_fallbacks,
    );
    let vol_3m_over_10m = guarded_ratio(
        per_timeframe[1][6],
        per_timeframe[3][6],
        &mut zero_denominator_fallbacks,
    );
    let volume_1m_over_10m = guarded_ratio(
        per_timeframe[0][13],
        per_timeframe[3][13],
        &mut zero_denominator_fallbacks,
    );
    let mut values = per_timeframe.into_iter().flatten().collect::<Vec<_>>();
    values.extend([
        agreement,
        dispersion,
        vol_1m_over_10m,
        vol_3m_over_10m,
        volume_1m_over_10m,
    ]);
    if values.len() != policy.feature_dimension() || values.iter().any(|value| !value.is_finite()) {
        return Err("compact event feature integrity rejected".to_string());
    }
    Ok(CompactEventResult {
        values,
        zero_range_fallbacks,
        zero_denominator_fallbacks,
    })
}

fn compact_matrix(
    policy: &MomentumCompactMicroFeaturePolicyV1,
    source: &MomentumQualifiedDiagnosticSourceV1,
    views: &BTreeMap<MomentumHistoricalTimeframeV1, Vec<MomentumQualifiedReplayCandleEvidenceV1>>,
) -> Result<(FeatureMatrix, Vec<MomentumCompactMicroIntegrityReplayV1>), String> {
    let feature_ids = policy
        .included_timeframes
        .iter()
        .flat_map(|timeframe| {
            policy
                .per_timeframe_feature_ids
                .iter()
                .map(move |feature| format!("{}:{feature}", timeframe_name(*timeframe)))
        })
        .chain(policy.cross_timeframe_feature_ids.iter().cloned())
        .collect::<Vec<_>>();
    let mut development = Vec::new();
    let mut validation = Vec::new();
    let mut counters = BTreeMap::from([
        (
            MomentumReplayPartitionV1::Development,
            (0_usize, 0_usize, 0_usize),
        ),
        (
            MomentumReplayPartitionV1::Validation,
            (0_usize, 0_usize, 0_usize),
        ),
    ]);
    for event in &source.events {
        if event.partition == MomentumReplayPartitionV1::SealedHoldout {
            return Err("compact holdout event access rejected".to_string());
        }
        let result = compact_event(policy, views, event.prediction_timestamp_ms)?;
        let entry = counters
            .get_mut(&event.partition)
            .ok_or_else(|| "compact partition counter unavailable".to_string())?;
        entry.0 += 1;
        entry.1 += result.zero_range_fallbacks;
        entry.2 += result.zero_denominator_fallbacks;
        match event.partition {
            MomentumReplayPartitionV1::Development => development.push(result.values),
            MomentumReplayPartitionV1::Validation => validation.push(result.values),
            MomentumReplayPartitionV1::SealedHoldout => unreachable!(),
        }
    }
    let matrix = FeatureMatrix {
        feature_ids,
        development,
        validation,
    };
    let constant = constant_count(&matrix.development)?;
    let replays = [
        MomentumReplayPartitionV1::Development,
        MomentumReplayPartitionV1::Validation,
    ]
    .into_iter()
    .map(|partition| {
        let (eligible, zero_range, zero_denominator) = counters[&partition];
        MomentumCompactMicroIntegrityReplayV1 {
            replay_version: COMPACT_REPLAY_VERSION.to_string(),
            partition,
            eligible_event_count: eligible,
            finite_block_count: eligible,
            missing_evidence_count: 0,
            partial_candle_count: 0,
            zero_range_fallback_count: zero_range,
            zero_denominator_fallback_count: zero_denominator,
            constant_feature_count: constant,
            redundancy_audit_digest: String::new(),
            distribution_shift_audit_digest: String::new(),
            feature_schema_digest: policy.schema_digest.clone(),
            future_access_count: 0,
            partial_access_count: 0,
            holdout_access_count: 0,
            deterministic_replay_digest: String::new(),
        }
    })
    .collect::<Vec<_>>();
    Ok((matrix, replays))
}

fn empty_feature_report(
    mode: MomentumMicroFeatureForensicsRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> MomentumMicroFeatureForensicsReportV1 {
    MomentumMicroFeatureForensicsReportV1 {
        report_version: FEATURE_REPORT_VERSION.to_string(),
        run_mode: mode.as_str().to_string(),
        status: MomentumMicroFeatureForensicsStatusV1::Unregistered,
        evidence_class: MomentumMicroChallengerDesignEvidenceClassV1::PostResultResearchDesignOnly,
        registration_digest: None,
        label_forensics_digest: None,
        protected_before_state_digest: protected.state_digest.clone(),
        source_feature_audits: Vec::new(),
        redundancy_audits: Vec::new(),
        partition_shift_audits: Vec::new(),
        normalizer_stability_audits: Vec::new(),
        compact_feature_policy: None,
        compact_integrity_replays: Vec::new(),
        safety_counters: MomentumMicroDesignSafetyCountersV1::default(),
        labels: PUBLIC_LABELS.map(str::to_string).to_vec(),
        deterministic: true,
        journal_digest: None,
        artifacts_written: 0,
        duplicate_artifact_count: 0,
        feature_computation_count: 0,
        runtime_duration_ms: 0,
        report_digest: String::new(),
    }
}

fn run_feature_inner(
    mode: MomentumMicroFeatureForensicsRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> Result<MomentumMicroFeatureForensicsReportV1, String> {
    let started = Instant::now();
    validate_momentum_micro_protected_before_state_v1(protected)?;
    if let Some(mut report) = read_momentum_micro_feature_forensics_report_v1()? {
        if report.protected_before_state_digest != protected.state_digest {
            return Err("micro feature protected state changed".to_string());
        }
        report.run_mode = mode.as_str().to_string();
        report.artifacts_written = 0;
        report.duplicate_artifact_count = 0;
        report.feature_computation_count = 0;
        report.runtime_duration_ms = started.elapsed().as_millis() as u64;
        report.report_digest = feature_report_digest(&report);
        validate_feature_report(&report)?;
        return Ok(report);
    }
    let registration = derive_feature_registration(protected)?;
    if let Some(stored) = read_feature_registration()?
        && stored != registration
    {
        return Err("micro feature registration conflict".to_string());
    }
    if mode == MomentumMicroFeatureForensicsRunModeV1::Status {
        let mut report = empty_feature_report(mode, protected);
        report.status = if read_feature_registration()?.is_some() {
            MomentumMicroFeatureForensicsStatusV1::Registered
        } else {
            MomentumMicroFeatureForensicsStatusV1::Unregistered
        };
        report.registration_digest = Some(registration.registration_digest);
        report.label_forensics_digest = Some(registration.label_forensics_digest);
        report.runtime_duration_ms = started.elapsed().as_millis() as u64;
        report.report_digest = feature_report_digest(&report);
        validate_feature_report(&report)?;
        return Ok(report);
    }
    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_one(
            "feature_registrations",
            &registration.registration_digest,
            &encode_feature_registration(&registration)?,
            |bytes| Ok(decode_feature_registration(bytes)?.registration_digest),
        )?,
    );
    if read_feature_registration()?.as_ref() != Some(&registration) {
        return Err("micro feature registration reopen mismatch".to_string());
    }

    // Private feature and normalizer access starts only after registration reopen.
    let header = load_momentum_qualified_diagnostic_source_header_v1()?;
    if header.replay_journal_digest != registration.source_replay_digest {
        return Err("micro feature replay source changed".to_string());
    }
    let source = load_momentum_qualified_diagnostic_source_v1(&header)?;
    let evidence = load_momentum_qualified_six_evidence_v1()?;
    if evidence.prior_holdout.labels_opened
        || evidence.prior_holdout.metrics_computed
        || evidence.prior_holdout.aggregate_comparison_opened
    {
        return Err("micro feature holdout boundary opened".to_string());
    }
    let q1_timeframes = MomentumQualifiedParticipantV1::Q1TenMinuteAnchorLogistic.timeframes();
    let q2_timeframes = MomentumQualifiedParticipantV1::Q2MicroBlockLogistic.timeframes();
    if q2_timeframes != registration.audited_timeframes {
        return Err("micro Q2 source feature policy changed".to_string());
    }
    let q1 = existing_matrix(&source, &evidence.views, "Q1", &q1_timeframes)?;
    let q2 = existing_matrix(&source, &evidence.views, "Q2", &q2_timeframes)?;
    let q1_normalizer = normalizer_audit("Q1", &source, &[3])?;
    let q2_normalizer = normalizer_audit("Q2", &source, &[0, 1, 2, 3])?;
    let mut source_audits = vec![
        source_audit(
            "Q1",
            q1_timeframes,
            &q1,
            q1_normalizer.normalizer_digest_trajectory.clone(),
            q1_normalizer.finite_status,
        )?,
        source_audit(
            "Q2",
            q2_timeframes,
            &q2,
            q2_normalizer.normalizer_digest_trajectory.clone(),
            q2_normalizer.finite_status,
        )?,
    ];
    let policy = compact_policy()?;
    let (compact, mut compact_replays) = compact_matrix(&policy, &source, &evidence.views)?;
    let mut redundancy = vec![
        redundancy_audit("Q1", &q1),
        redundancy_audit("Q2", &q2),
        redundancy_audit("CompactMicroV1", &compact),
    ];
    let mut shifts = Vec::new();
    for (schema_id, matrix) in [("Q1", &q1), ("Q2", &q2), ("CompactMicroV1", &compact)] {
        let bins = bin_policy(schema_id, matrix)?;
        add_counts(
            &mut counts,
            persist_one(
                "drift_bin_policies",
                &bins.policy_digest,
                &encode_bin_policy(&bins)?,
                |bytes| Ok(decode_bin_policy(bytes)?.policy_digest),
            )?,
        );
        // Validation assignment is performed only after the development-derived bin policy persists.
        shifts.push(shift_audit(schema_id, matrix, &bins)?);
    }
    let compact_redundancy = redundancy
        .iter()
        .find(|value| value.schema_id == "CompactMicroV1")
        .ok_or_else(|| "compact redundancy audit unavailable".to_string())?;
    let compact_shift = shifts
        .iter()
        .find(|value| value.schema_id == "CompactMicroV1")
        .ok_or_else(|| "compact shift audit unavailable".to_string())?;
    for replay in &mut compact_replays {
        replay.redundancy_audit_digest = compact_redundancy.audit_digest.clone();
        replay.distribution_shift_audit_digest = compact_shift.audit_digest.clone();
        replay.deterministic_replay_digest = compact_replay_digest(replay);
        let encoded = encode_compact_replay(replay)?;
        add_counts(
            &mut counts,
            persist_one(
                "compact_integrity_replays",
                &replay.deterministic_replay_digest,
                &encoded,
                |bytes| Ok(decode_compact_replay(bytes)?.deterministic_replay_digest),
            )?,
        );
    }
    for value in &source_audits {
        add_counts(
            &mut counts,
            persist_one(
                "source_schema_audits",
                &value.audit_digest,
                &encode_source_audit(value)?,
                |bytes| Ok(decode_source_audit(bytes)?.audit_digest),
            )?,
        );
    }
    for value in &redundancy {
        add_counts(
            &mut counts,
            persist_one(
                "redundancy_audits",
                &value.audit_digest,
                &encode_redundancy(value)?,
                |bytes| Ok(decode_redundancy(bytes)?.audit_digest),
            )?,
        );
    }
    for value in &shifts {
        add_counts(
            &mut counts,
            persist_one(
                "partition_shift_audits",
                &value.audit_digest,
                &encode_shift(value)?,
                |bytes| Ok(decode_shift(bytes)?.audit_digest),
            )?,
        );
    }
    for value in [&q1_normalizer, &q2_normalizer] {
        add_counts(
            &mut counts,
            persist_one(
                "normalizer_stability_audits",
                &value.audit_digest,
                &encode_normalizer(value)?,
                |bytes| Ok(decode_normalizer(bytes)?.audit_digest),
            )?,
        );
    }
    add_counts(
        &mut counts,
        persist_one(
            "compact_feature_policies",
            &policy.schema_digest,
            &encode_compact_policy(&policy)?,
            |bytes| Ok(decode_compact_policy(bytes)?.schema_digest),
        )?,
    );
    let mut journal = MomentumMicroFeatureForensicsJournalV1 {
        journal_version: FEATURE_JOURNAL_VERSION.to_string(),
        registration_digest: registration.registration_digest.clone(),
        source_audit_digests: source_audits
            .iter()
            .map(|value| value.audit_digest.clone())
            .collect(),
        redundancy_audit_digests: redundancy
            .iter()
            .map(|value| value.audit_digest.clone())
            .collect(),
        partition_shift_audit_digests: shifts
            .iter()
            .map(|value| value.audit_digest.clone())
            .collect(),
        normalizer_stability_audit_digests: [&q1_normalizer, &q2_normalizer]
            .into_iter()
            .map(|value| value.audit_digest.clone())
            .collect(),
        compact_feature_policy_digest: policy.schema_digest.clone(),
        compact_integrity_replay_digests: compact_replays
            .iter()
            .map(|value| value.deterministic_replay_digest.clone())
            .collect(),
        holdout_access_count: 0,
        model_execution_count: 0,
        journal_digest: String::new(),
    };
    journal.journal_digest = feature_journal_digest(&journal);
    add_counts(
        &mut counts,
        persist_one(
            "feature_journals",
            &journal.journal_digest,
            &encode_feature_journal(&journal)?,
            |bytes| Ok(decode_feature_journal(bytes)?.journal_digest),
        )?,
    );
    let mut report = empty_feature_report(mode, protected);
    report.status = MomentumMicroFeatureForensicsStatusV1::Complete;
    report.registration_digest = Some(registration.registration_digest);
    report.label_forensics_digest = Some(registration.label_forensics_digest);
    report.source_feature_audits = std::mem::take(&mut source_audits);
    report.redundancy_audits = std::mem::take(&mut redundancy);
    report.partition_shift_audits = shifts;
    report.normalizer_stability_audits = vec![q1_normalizer, q2_normalizer];
    report.compact_feature_policy = Some(policy);
    report.compact_integrity_replays = compact_replays;
    report.journal_digest = Some(journal.journal_digest);
    report.artifacts_written = counts.0 + 1;
    report.duplicate_artifact_count = counts.1;
    report.feature_computation_count = (q1.development.len() + q1.validation.len())
        * q1.feature_ids.len()
        + (q2.development.len() + q2.validation.len()) * q2.feature_ids.len()
        + (compact.development.len() + compact.validation.len()) * compact.feature_ids.len();
    report.runtime_duration_ms = started.elapsed().as_millis() as u64;
    report.report_digest = feature_report_digest(&report);
    validate_feature_report(&report)?;
    add_counts(
        &mut counts,
        persist_one(
            "feature_final_reports",
            &report.report_digest,
            &encode_feature_report(&report)?,
            |bytes| Ok(decode_feature_report(bytes)?.report_digest),
        )?,
    );
    if counts.0 != report.artifacts_written {
        return Err("micro feature artifact accounting mismatch".to_string());
    }
    if read_momentum_micro_feature_forensics_report_v1()?.as_ref() != Some(&report) {
        return Err("micro feature final report reopen mismatch".to_string());
    }
    Ok(report)
}

pub fn run_momentum_micro_feature_forensics_v1(
    mode: MomentumMicroFeatureForensicsRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> Result<MomentumMicroFeatureForensicsReportV1, String> {
    match run_feature_inner(mode, protected) {
        Ok(report) => Ok(report),
        Err(error)
            if error.contains("artifact")
                || error.contains("conflict")
                || error.contains("mismatch")
                || error.contains("source changed") =>
        {
            let mut report = empty_feature_report(mode, protected);
            report.status = MomentumMicroFeatureForensicsStatusV1::IntegrityFailure;
            report.report_digest = feature_report_digest(&report);
            validate_feature_report(&report)?;
            Ok(report)
        }
        Err(error) => Err(error),
    }
}

fn derive_boundary(
    task: MomentumMicroTaskV1,
    timestamps: Vec<u64>,
) -> Result<MomentumMicroTaskPartitionBoundaryV1, String> {
    let mut timestamps = timestamps;
    timestamps.sort();
    timestamps.dedup();
    if timestamps.len() < 20 {
        return Err("micro task boundary support insufficient".to_string());
    }
    let development_count = timestamps.len() * 70 / 100;
    let validation_count = timestamps.len() * 15 / 100;
    let holdout_count = timestamps.len() - development_count - validation_count;
    let development_end_exclusive_ms = timestamps[development_count];
    let validation_end_exclusive_ms = timestamps[development_count + validation_count];
    let mut value = MomentumMicroTaskPartitionBoundaryV1 {
        boundary_version: BOUNDARY_VERSION.to_string(),
        task,
        eligible_start_timestamp_ms: timestamps[0],
        eligible_end_timestamp_ms: *timestamps.last().unwrap_or(&timestamps[0]),
        development_end_exclusive_ms,
        validation_end_exclusive_ms,
        holdout_start_timestamp_ms: validation_end_exclusive_ms,
        common_eligible_event_count: timestamps.len(),
        development_event_count: development_count,
        validation_event_count: validation_count,
        holdout_event_count: holdout_count,
        label_values_read_for_boundary: 0,
        holdout_labels_opened: false,
        boundary_digest: String::new(),
    };
    value.boundary_digest = boundary_digest(&value);
    Ok(value)
}

fn task_registration(
    task: MomentumMicroTaskV1,
    policy: &MomentumCompactMicroFeaturePolicyV1,
    boundary: &MomentumMicroTaskPartitionBoundaryV1,
) -> MomentumMicroTaskRegistrationV1 {
    let (cadence, horizon) = match task {
        MomentumMicroTaskV1::T10NextTenMinuteDirection => (TEN_MINUTE_MS, 1),
        MomentumMicroTaskV1::T30NextThirtyMinuteDirection => (3 * TEN_MINUTE_MS, 3),
    };
    let feature_policy_digest = if task == MomentumMicroTaskV1::T10NextTenMinuteDirection {
        policy.schema_digest.clone()
    } else {
        stable_hash_string(&format!(
            "micro-t30-task-feature-policy-v1:{}",
            policy.schema_digest
        ))
    };
    let mut value = MomentumMicroTaskRegistrationV1 {
        registration_version: TASK_VERSION.to_string(),
        task,
        event_cadence_ms: cadence,
        target_horizon_candles: horizon,
        feature_policy_digest,
        boundary_digest: boundary.boundary_digest.clone(),
        t60_experiment: false,
        model_execution_authorized: false,
        task_digest: String::new(),
    };
    value.task_digest = task_digest(&value);
    value
}

fn participant_registrations(
    tasks: &[MomentumMicroTaskRegistrationV1],
    policy: &MomentumCompactMicroFeaturePolicyV1,
) -> Vec<MomentumMicroParticipantRegistrationV1> {
    tasks
        .iter()
        .flat_map(|task| {
            MomentumMicroParticipantV1::ORDERED
                .into_iter()
                .map(move |participant| {
                    let feature_policy_digest = match participant {
                        MomentumMicroParticipantV1::C0TaskSpecificConstant => {
                            stable_hash_string(&format!(
                                "micro-task-constant-v1:{:?}",
                                task.task
                            ))
                        }
                        MomentumMicroParticipantV1::C1TenMinuteAnchorBaseline => {
                            stable_hash_string(&format!(
                                "micro-ten-minute-anchor-v1:{:?}:fresh-task-parameters",
                                task.task
                            ))
                        }
                        _ => policy.schema_digest.clone(),
                    };
                    let (standard_l2_multiplier, base, calibration, training_only) =
                        match participant {
                            MomentumMicroParticipantV1::C3CompactMicroStrongShrinkLogistic => {
                                (4, 0, 0, false)
                            }
                            MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic => {
                                (1, 80, 20, true)
                            }
                            _ => (1, 0, 0, false),
                        };
                    let mut value = MomentumMicroParticipantRegistrationV1 {
                        registration_version: PARTICIPANT_VERSION.to_string(),
                        task: task.task,
                        participant,
                        participant_id: format!("{:?}:{participant:?}", task.task),
                        feature_policy_digest,
                        fresh_task_parameters_required: true,
                        standard_l2_multiplier,
                        calibration_base_fit_percent: base,
                        calibration_fit_percent: calibration,
                        calibration_training_only: training_only,
                        validation_fit_forbidden: true,
                        holdout_fit_forbidden: true,
                        model_execution_authorized: false,
                        participant_digest: String::new(),
                    };
                    value.participant_digest = participant_digest(&value);
                    value
                })
        })
        .collect()
}

fn screening_gate() -> MomentumMicroScreeningGateV1 {
    let mut value = MomentumMicroScreeningGateV1 {
        gate_version: SCREENING_GATE_VERSION.to_string(),
        lower_brier_development_required: true,
        lower_brier_validation_required: true,
        finite_predictions_and_metrics_required: true,
        probability_collapse_forbidden: true,
        chronology_failure_forbidden: true,
        leakage_failure_forbidden: true,
        integrity_failure_forbidden: true,
        sufficient_paired_support_required: true,
        result_selected_mutation_forbidden: true,
        holdout_access_forbidden: true,
        correctness_override_forbidden: true,
        gate_digest: String::new(),
    };
    value.gate_digest = gate_digest(&value);
    value
}

fn empty_final_report(
    mode: MomentumMicroChallengerRegistrationRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
    label_digest: String,
    feature: &MomentumMicroFeatureForensicsReportV1,
    policy: &MomentumCompactMicroFeaturePolicyV1,
) -> MomentumMicroChallengerDesignReportV1 {
    MomentumMicroChallengerDesignReportV1 {
        report_version: FINAL_REPORT_VERSION.to_string(),
        run_mode: mode.as_str().to_string(),
        complete: false,
        evidence_class: MomentumMicroChallengerDesignEvidenceClassV1::PostResultResearchDesignOnly,
        protected_before_state_digest: protected.state_digest.clone(),
        label_forensics_digest: label_digest,
        feature_forensics_digest: feature.report_digest.clone(),
        compact_feature_policy_digest: policy.schema_digest.clone(),
        compact_feature_dimension: policy.feature_dimension(),
        task_boundaries: Vec::new(),
        screening_registration: None,
        screening_gate: None,
        model_execution_authorized: false,
        holdout_execution_authorized: false,
        safety_counters: MomentumMicroDesignSafetyCountersV1::default(),
        labels: PUBLIC_LABELS.map(str::to_string).to_vec(),
        deterministic: true,
        journal_digest: None,
        artifacts_written: 0,
        duplicate_artifact_count: 0,
        runtime_duration_ms: 0,
        report_digest: String::new(),
    }
}

fn encode_final_report(value: &MomentumMicroChallengerDesignReportV1) -> Result<Vec<u8>, String> {
    validate_final_report(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroChallengerDesignReportV1")
        .string("report_version", &value.report_version)
        .string("run_mode", &value.run_mode)
        .boolean("complete", value.complete)
        .string("evidence_class", format!("{:?}", value.evidence_class))
        .string(
            "protected_before_state_digest",
            &value.protected_before_state_digest,
        )
        .string("label_forensics_digest", &value.label_forensics_digest)
        .string("feature_forensics_digest", &value.feature_forensics_digest)
        .string(
            "compact_feature_policy_digest",
            &value.compact_feature_policy_digest,
        )
        .unsigned(
            "compact_feature_dimension",
            as_u64(value.compact_feature_dimension)?,
        )
        .messages(
            "task_boundaries",
            value
                .task_boundaries
                .iter()
                .map(encode_boundary)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "screening_registration",
            value
                .screening_registration
                .iter()
                .map(encode_screening_registration)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "screening_gate",
            value
                .screening_gate
                .iter()
                .map(encode_gate)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .boolean(
            "model_execution_authorized",
            value.model_execution_authorized,
        )
        .boolean(
            "holdout_execution_authorized",
            value.holdout_execution_authorized,
        )
        .messages(
            "safety_counters",
            vec![encode_safety(&value.safety_counters)?],
        )
        .strings("labels", &value.labels)
        .boolean("deterministic", value.deterministic)
        .optional_string("journal_digest", &value.journal_digest)
        .unsigned("artifacts_written", as_u64(value.artifacts_written)?)
        .unsigned(
            "duplicate_artifact_count",
            as_u64(value.duplicate_artifact_count)?,
        )
        .unsigned("runtime_duration_ms", value.runtime_duration_ms)
        .string("report_digest", &value.report_digest)
        .encode()
}

fn encode_boundary(value: &MomentumMicroTaskPartitionBoundaryV1) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumMicroTaskPartitionBoundaryV1")
        .string("boundary_version", &value.boundary_version)
        .string("task", format!("{:?}", value.task))
        .unsigned(
            "eligible_start_timestamp_ms",
            value.eligible_start_timestamp_ms,
        )
        .unsigned("eligible_end_timestamp_ms", value.eligible_end_timestamp_ms)
        .unsigned(
            "development_end_exclusive_ms",
            value.development_end_exclusive_ms,
        )
        .unsigned(
            "validation_end_exclusive_ms",
            value.validation_end_exclusive_ms,
        )
        .unsigned(
            "holdout_start_timestamp_ms",
            value.holdout_start_timestamp_ms,
        )
        .unsigned(
            "common_eligible_event_count",
            as_u64(value.common_eligible_event_count)?,
        )
        .unsigned(
            "development_event_count",
            as_u64(value.development_event_count)?,
        )
        .unsigned(
            "validation_event_count",
            as_u64(value.validation_event_count)?,
        )
        .unsigned("holdout_event_count", as_u64(value.holdout_event_count)?)
        .unsigned(
            "label_values_read_for_boundary",
            as_u64(value.label_values_read_for_boundary)?,
        )
        .boolean("holdout_labels_opened", value.holdout_labels_opened)
        .string("boundary_digest", &value.boundary_digest)
        .encode()
}

fn decode_boundary(bytes: &[u8]) -> Result<MomentumMicroTaskPartitionBoundaryV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroTaskPartitionBoundaryV1")?;
    let value = MomentumMicroTaskPartitionBoundaryV1 {
        boundary_version: fields.string("boundary_version")?,
        task: MomentumMicroTaskV1::parse(&fields.string("task")?)?,
        eligible_start_timestamp_ms: fields.unsigned("eligible_start_timestamp_ms")?,
        eligible_end_timestamp_ms: fields.unsigned("eligible_end_timestamp_ms")?,
        development_end_exclusive_ms: fields.unsigned("development_end_exclusive_ms")?,
        validation_end_exclusive_ms: fields.unsigned("validation_end_exclusive_ms")?,
        holdout_start_timestamp_ms: fields.unsigned("holdout_start_timestamp_ms")?,
        common_eligible_event_count: as_usize(fields.unsigned("common_eligible_event_count")?)?,
        development_event_count: as_usize(fields.unsigned("development_event_count")?)?,
        validation_event_count: as_usize(fields.unsigned("validation_event_count")?)?,
        holdout_event_count: as_usize(fields.unsigned("holdout_event_count")?)?,
        label_values_read_for_boundary: as_usize(
            fields.unsigned("label_values_read_for_boundary")?,
        )?,
        holdout_labels_opened: fields.boolean("holdout_labels_opened")?,
        boundary_digest: fields.string("boundary_digest")?,
    };
    fields.finish()?;
    let expected_development = value.common_eligible_event_count * 70 / 100;
    let expected_validation = value.common_eligible_event_count * 15 / 100;
    let expected_holdout =
        value.common_eligible_event_count - expected_development - expected_validation;
    if value.boundary_version != BOUNDARY_VERSION
        || value.common_eligible_event_count < 20
        || value.development_event_count != expected_development
        || value.validation_event_count != expected_validation
        || value.holdout_event_count != expected_holdout
        || value.eligible_start_timestamp_ms >= value.development_end_exclusive_ms
        || value.development_end_exclusive_ms >= value.validation_end_exclusive_ms
        || value.validation_end_exclusive_ms != value.holdout_start_timestamp_ms
        || value.holdout_start_timestamp_ms > value.eligible_end_timestamp_ms
        || value.label_values_read_for_boundary != 0
        || value.holdout_labels_opened
        || value.boundary_digest != boundary_digest(&value)
    {
        return Err("micro task boundary rejected".to_string());
    }
    Ok(value)
}

fn encode_task(value: &MomentumMicroTaskRegistrationV1) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumMicroTaskRegistrationV1")
        .string("registration_version", &value.registration_version)
        .string("task", format!("{:?}", value.task))
        .unsigned("event_cadence_ms", value.event_cadence_ms)
        .unsigned(
            "target_horizon_candles",
            as_u64(value.target_horizon_candles)?,
        )
        .string("feature_policy_digest", &value.feature_policy_digest)
        .string("boundary_digest", &value.boundary_digest)
        .boolean("t60_experiment", value.t60_experiment)
        .boolean(
            "model_execution_authorized",
            value.model_execution_authorized,
        )
        .string("task_digest", &value.task_digest)
        .encode()
}

fn decode_task(bytes: &[u8]) -> Result<MomentumMicroTaskRegistrationV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroTaskRegistrationV1")?;
    let value = MomentumMicroTaskRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        task: MomentumMicroTaskV1::parse(&fields.string("task")?)?,
        event_cadence_ms: fields.unsigned("event_cadence_ms")?,
        target_horizon_candles: as_usize(fields.unsigned("target_horizon_candles")?)?,
        feature_policy_digest: fields.string("feature_policy_digest")?,
        boundary_digest: fields.string("boundary_digest")?,
        t60_experiment: fields.boolean("t60_experiment")?,
        model_execution_authorized: fields.boolean("model_execution_authorized")?,
        task_digest: fields.string("task_digest")?,
    };
    fields.finish()?;
    let (expected_cadence, expected_horizon) = match value.task {
        MomentumMicroTaskV1::T10NextTenMinuteDirection => (TEN_MINUTE_MS, 1),
        MomentumMicroTaskV1::T30NextThirtyMinuteDirection => (3 * TEN_MINUTE_MS, 3),
    };
    if value.registration_version != TASK_VERSION
        || value.event_cadence_ms != expected_cadence
        || value.target_horizon_candles != expected_horizon
        || value.feature_policy_digest.is_empty()
        || value.boundary_digest.is_empty()
        || value.t60_experiment
        || value.model_execution_authorized
        || value.task_digest != task_digest(&value)
    {
        return Err("micro task registration rejected".to_string());
    }
    Ok(value)
}

fn encode_participant(value: &MomentumMicroParticipantRegistrationV1) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumMicroParticipantRegistrationV1")
        .string("registration_version", &value.registration_version)
        .string("task", format!("{:?}", value.task))
        .string("participant", format!("{:?}", value.participant))
        .string("participant_id", &value.participant_id)
        .string("feature_policy_digest", &value.feature_policy_digest)
        .boolean(
            "fresh_task_parameters_required",
            value.fresh_task_parameters_required,
        )
        .unsigned(
            "standard_l2_multiplier",
            as_u64(value.standard_l2_multiplier)?,
        )
        .unsigned(
            "calibration_base_fit_percent",
            as_u64(value.calibration_base_fit_percent)?,
        )
        .unsigned(
            "calibration_fit_percent",
            as_u64(value.calibration_fit_percent)?,
        )
        .boolean("calibration_training_only", value.calibration_training_only)
        .boolean("validation_fit_forbidden", value.validation_fit_forbidden)
        .boolean("holdout_fit_forbidden", value.holdout_fit_forbidden)
        .boolean(
            "model_execution_authorized",
            value.model_execution_authorized,
        )
        .string("participant_digest", &value.participant_digest)
        .encode()
}

fn decode_participant(bytes: &[u8]) -> Result<MomentumMicroParticipantRegistrationV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroParticipantRegistrationV1")?;
    let value = MomentumMicroParticipantRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        task: MomentumMicroTaskV1::parse(&fields.string("task")?)?,
        participant: MomentumMicroParticipantV1::parse(&fields.string("participant")?)?,
        participant_id: fields.string("participant_id")?,
        feature_policy_digest: fields.string("feature_policy_digest")?,
        fresh_task_parameters_required: fields.boolean("fresh_task_parameters_required")?,
        standard_l2_multiplier: as_usize(fields.unsigned("standard_l2_multiplier")?)?,
        calibration_base_fit_percent: as_usize(fields.unsigned("calibration_base_fit_percent")?)?,
        calibration_fit_percent: as_usize(fields.unsigned("calibration_fit_percent")?)?,
        calibration_training_only: fields.boolean("calibration_training_only")?,
        validation_fit_forbidden: fields.boolean("validation_fit_forbidden")?,
        holdout_fit_forbidden: fields.boolean("holdout_fit_forbidden")?,
        model_execution_authorized: fields.boolean("model_execution_authorized")?,
        participant_digest: fields.string("participant_digest")?,
    };
    fields.finish()?;
    let participant_policy_valid = match value.participant {
        MomentumMicroParticipantV1::C3CompactMicroStrongShrinkLogistic => {
            value.standard_l2_multiplier == 4
                && value.calibration_base_fit_percent == 0
                && value.calibration_fit_percent == 0
                && !value.calibration_training_only
        }
        MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic => {
            value.standard_l2_multiplier == 1
                && value.calibration_base_fit_percent == 80
                && value.calibration_fit_percent == 20
                && value.calibration_training_only
        }
        _ => {
            value.standard_l2_multiplier == 1
                && value.calibration_base_fit_percent == 0
                && value.calibration_fit_percent == 0
                && !value.calibration_training_only
        }
    };
    if value.registration_version != PARTICIPANT_VERSION
        || value.participant_id != format!("{:?}:{:?}", value.task, value.participant)
        || value.feature_policy_digest.is_empty()
        || !participant_policy_valid
        || !value.fresh_task_parameters_required
        || !value.validation_fit_forbidden
        || !value.holdout_fit_forbidden
        || value.model_execution_authorized
        || value.participant_digest != participant_digest(&value)
    {
        return Err("micro participant registration rejected".to_string());
    }
    Ok(value)
}

fn encode_gate(value: &MomentumMicroScreeningGateV1) -> Result<Vec<u8>, String> {
    if value != &screening_gate() {
        return Err("micro screening gate rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumMicroScreeningGateV1")
        .string("gate_version", &value.gate_version)
        .boolean(
            "lower_brier_development_required",
            value.lower_brier_development_required,
        )
        .boolean(
            "lower_brier_validation_required",
            value.lower_brier_validation_required,
        )
        .boolean(
            "finite_predictions_and_metrics_required",
            value.finite_predictions_and_metrics_required,
        )
        .boolean(
            "probability_collapse_forbidden",
            value.probability_collapse_forbidden,
        )
        .boolean(
            "chronology_failure_forbidden",
            value.chronology_failure_forbidden,
        )
        .boolean("leakage_failure_forbidden", value.leakage_failure_forbidden)
        .boolean(
            "integrity_failure_forbidden",
            value.integrity_failure_forbidden,
        )
        .boolean(
            "sufficient_paired_support_required",
            value.sufficient_paired_support_required,
        )
        .boolean(
            "result_selected_mutation_forbidden",
            value.result_selected_mutation_forbidden,
        )
        .boolean("holdout_access_forbidden", value.holdout_access_forbidden)
        .boolean(
            "correctness_override_forbidden",
            value.correctness_override_forbidden,
        )
        .string("gate_digest", &value.gate_digest)
        .encode()
}

fn decode_gate(bytes: &[u8]) -> Result<MomentumMicroScreeningGateV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroScreeningGateV1")?;
    let value = MomentumMicroScreeningGateV1 {
        gate_version: fields.string("gate_version")?,
        lower_brier_development_required: fields.boolean("lower_brier_development_required")?,
        lower_brier_validation_required: fields.boolean("lower_brier_validation_required")?,
        finite_predictions_and_metrics_required: fields
            .boolean("finite_predictions_and_metrics_required")?,
        probability_collapse_forbidden: fields.boolean("probability_collapse_forbidden")?,
        chronology_failure_forbidden: fields.boolean("chronology_failure_forbidden")?,
        leakage_failure_forbidden: fields.boolean("leakage_failure_forbidden")?,
        integrity_failure_forbidden: fields.boolean("integrity_failure_forbidden")?,
        sufficient_paired_support_required: fields.boolean("sufficient_paired_support_required")?,
        result_selected_mutation_forbidden: fields.boolean("result_selected_mutation_forbidden")?,
        holdout_access_forbidden: fields.boolean("holdout_access_forbidden")?,
        correctness_override_forbidden: fields.boolean("correctness_override_forbidden")?,
        gate_digest: fields.string("gate_digest")?,
    };
    fields.finish()?;
    if value != screening_gate() {
        return Err("micro screening gate rejected".to_string());
    }
    Ok(value)
}

fn encode_screening_registration(
    value: &MomentumMicroChallengerScreeningRegistrationV1,
) -> Result<Vec<u8>, String> {
    validate_screening_registration(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroChallengerScreeningRegistrationV1")
        .string("registration_version", &value.registration_version)
        .string("source_replay_digest", &value.source_replay_digest)
        .string("source_diagnostic_digest", &value.source_diagnostic_digest)
        .string(
            "compact_feature_policy_digest",
            &value.compact_feature_policy_digest,
        )
        .string("label_forensics_digest", &value.label_forensics_digest)
        .messages(
            "task_registrations",
            value
                .task_registrations
                .iter()
                .map(encode_task)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "participant_registrations",
            value
                .participant_registrations
                .iter()
                .map(encode_participant)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .string("partition_policy_digest", &value.partition_policy_digest)
        .string("training_policy_digest", &value.training_policy_digest)
        .string("screening_gate_digest", &value.screening_gate_digest)
        .boolean(
            "model_execution_authorized",
            value.model_execution_authorized,
        )
        .boolean(
            "holdout_execution_authorized",
            value.holdout_execution_authorized,
        )
        .boolean("live_authority_forbidden", value.live_authority_forbidden)
        .boolean(
            "governance_authority_forbidden",
            value.governance_authority_forbidden,
        )
        .boolean(
            "trading_authority_forbidden",
            value.trading_authority_forbidden,
        )
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_screening_registration(
    bytes: &[u8],
) -> Result<MomentumMicroChallengerScreeningRegistrationV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumMicroChallengerScreeningRegistrationV1")?;
    let value = MomentumMicroChallengerScreeningRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        source_replay_digest: fields.string("source_replay_digest")?,
        source_diagnostic_digest: fields.string("source_diagnostic_digest")?,
        compact_feature_policy_digest: fields.string("compact_feature_policy_digest")?,
        label_forensics_digest: fields.string("label_forensics_digest")?,
        task_registrations: fields
            .messages("task_registrations")?
            .iter()
            .map(|bytes| decode_task(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        participant_registrations: fields
            .messages("participant_registrations")?
            .iter()
            .map(|bytes| decode_participant(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        partition_policy_digest: fields.string("partition_policy_digest")?,
        training_policy_digest: fields.string("training_policy_digest")?,
        screening_gate_digest: fields.string("screening_gate_digest")?,
        model_execution_authorized: fields.boolean("model_execution_authorized")?,
        holdout_execution_authorized: fields.boolean("holdout_execution_authorized")?,
        live_authority_forbidden: fields.boolean("live_authority_forbidden")?,
        governance_authority_forbidden: fields.boolean("governance_authority_forbidden")?,
        trading_authority_forbidden: fields.boolean("trading_authority_forbidden")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_screening_registration(&value)?;
    Ok(value)
}

fn decode_final_report(bytes: &[u8]) -> Result<MomentumMicroChallengerDesignReportV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroChallengerDesignReportV1")?;
    let registrations = fields.messages("screening_registration")?;
    let gates = fields.messages("screening_gate")?;
    let safety = fields.messages("safety_counters")?;
    if registrations.len() > 1 || gates.len() > 1 || safety.len() != 1 {
        return Err("micro final report nested identity rejected".to_string());
    }
    let screening_registration = registrations
        .first()
        .map(|bytes| decode_screening_registration(bytes))
        .transpose()?;
    let screening_gate = if let (Some(registration), Some(bytes)) =
        (screening_registration.as_ref(), gates.first())
    {
        let gate = decode_gate(bytes)?;
        if gate.gate_digest != registration.screening_gate_digest {
            return Err("micro screening gate binding rejected".to_string());
        }
        Some(gate)
    } else {
        None
    };
    let value = MomentumMicroChallengerDesignReportV1 {
        report_version: fields.string("report_version")?,
        run_mode: fields.string("run_mode")?,
        complete: fields.boolean("complete")?,
        evidence_class: match fields.string("evidence_class")?.as_str() {
            "PostResultResearchDesignOnly" => {
                MomentumMicroChallengerDesignEvidenceClassV1::PostResultResearchDesignOnly
            }
            _ => return Err("micro final evidence class rejected".to_string()),
        },
        protected_before_state_digest: fields.string("protected_before_state_digest")?,
        label_forensics_digest: fields.string("label_forensics_digest")?,
        feature_forensics_digest: fields.string("feature_forensics_digest")?,
        compact_feature_policy_digest: fields.string("compact_feature_policy_digest")?,
        compact_feature_dimension: as_usize(fields.unsigned("compact_feature_dimension")?)?,
        task_boundaries: fields
            .messages("task_boundaries")?
            .iter()
            .map(|bytes| decode_boundary(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        screening_registration,
        screening_gate,
        model_execution_authorized: fields.boolean("model_execution_authorized")?,
        holdout_execution_authorized: fields.boolean("holdout_execution_authorized")?,
        safety_counters: decode_safety(&safety[0])?,
        labels: fields.strings("labels")?,
        deterministic: fields.boolean("deterministic")?,
        journal_digest: fields.optional_string("journal_digest")?,
        artifacts_written: as_usize(fields.unsigned("artifacts_written")?)?,
        duplicate_artifact_count: as_usize(fields.unsigned("duplicate_artifact_count")?)?,
        runtime_duration_ms: fields.unsigned("runtime_duration_ms")?,
        report_digest: fields.string("report_digest")?,
    };
    fields.finish()?;
    validate_final_report(&value)?;
    Ok(value)
}

pub fn read_momentum_micro_challenger_design_report_v1()
-> Result<Option<MomentumMicroChallengerDesignReportV1>, String> {
    read_single(&Path::new(ROOT).join("final_reports"), decode_final_report)
}

fn run_registration_inner(
    mode: MomentumMicroChallengerRegistrationRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> Result<MomentumMicroChallengerDesignReportV1, String> {
    let started = Instant::now();
    validate_momentum_micro_protected_before_state_v1(protected)?;
    let label = read_momentum_micro_label_forensics_report_v1()?
        .ok_or_else(|| "micro label report unavailable".to_string())?;
    let feature = read_momentum_micro_feature_forensics_report_v1()?
        .ok_or_else(|| "micro feature report unavailable".to_string())?;
    let policy = feature
        .compact_feature_policy
        .as_ref()
        .ok_or_else(|| "compact micro feature policy unavailable".to_string())?;
    if label.status != MomentumMicroLabelForensicsStatusV1::Complete
        || feature.status != MomentumMicroFeatureForensicsStatusV1::Complete
        || label.protected_before_state_digest != protected.state_digest
        || feature.protected_before_state_digest != protected.state_digest
    {
        return Err("micro registration prerequisite rejected".to_string());
    }
    if let Some(mut report) = read_momentum_micro_challenger_design_report_v1()? {
        if report.protected_before_state_digest != protected.state_digest {
            return Err("micro final protected state changed".to_string());
        }
        report.run_mode = mode.as_str().to_string();
        report.artifacts_written = 0;
        report.duplicate_artifact_count = 0;
        report.runtime_duration_ms = started.elapsed().as_millis() as u64;
        report.report_digest = final_report_digest(&report);
        validate_final_report(&report)?;
        return Ok(report);
    }
    let mut report = empty_final_report(
        mode,
        protected,
        label.report_digest.clone(),
        &feature,
        policy,
    );
    if mode == MomentumMicroChallengerRegistrationRunModeV1::Status {
        report.runtime_duration_ms = started.elapsed().as_millis() as u64;
        report.report_digest = final_report_digest(&report);
        validate_final_report(&report)?;
        return Ok(report);
    }
    let evidence = load_momentum_qualified_six_evidence_v1()?;
    if evidence.prior_holdout.labels_opened
        || evidence.prior_holdout.metrics_computed
        || evidence.prior_holdout.aggregate_comparison_opened
    {
        return Err("micro task boundary holdout opened".to_string());
    }
    let t10_timestamps = evidence
        .protocol_events
        .iter()
        .filter(|event| {
            event.target_timestamp_ms <= evidence.prior_holdout.eligible_end_timestamp_ms
        })
        .map(|event| event.prediction_timestamp_ms)
        .collect::<Vec<_>>();
    let t30_timestamps = t10_timestamps
        .iter()
        .copied()
        .filter(|timestamp| timestamp % (3 * TEN_MINUTE_MS) == 0)
        .collect::<Vec<_>>();
    let boundaries = vec![
        derive_boundary(
            MomentumMicroTaskV1::T10NextTenMinuteDirection,
            t10_timestamps,
        )?,
        derive_boundary(
            MomentumMicroTaskV1::T30NextThirtyMinuteDirection,
            t30_timestamps,
        )?,
    ];
    let tasks = boundaries
        .iter()
        .map(|boundary| task_registration(boundary.task, policy, boundary))
        .collect::<Vec<_>>();
    let participants = participant_registrations(&tasks, policy);
    let gate = screening_gate();
    let header = load_momentum_qualified_diagnostic_source_header_v1()?;
    let mut registration = MomentumMicroChallengerScreeningRegistrationV1 {
        registration_version: SCREENING_REGISTRATION_VERSION.to_string(),
        source_replay_digest: header.replay_journal_digest,
        source_diagnostic_digest: label
            .source_diagnostic_digest
            .clone()
            .ok_or_else(|| "micro source diagnostic digest unavailable".to_string())?,
        compact_feature_policy_digest: policy.schema_digest.clone(),
        label_forensics_digest: label.report_digest.clone(),
        task_registrations: tasks,
        participant_registrations: participants,
        partition_policy_digest: stable_hash_string(&format!(
            "micro-task-partition-policy-v1:oldest70:next15:newest15:{:?}",
            boundaries
                .iter()
                .map(|value| &value.boundary_digest)
                .collect::<Vec<_>>()
        )),
        training_policy_digest: stable_hash_string(
            "micro-future-training-policy-v1:daily-utc:past-revealed-labels:fixed-max-window:dimension-support:no-within-day-refit:receipt-reopen:prediction-seal:t30-three-candle-observability",
        ),
        screening_gate_digest: gate.gate_digest.clone(),
        model_execution_authorized: false,
        holdout_execution_authorized: false,
        live_authority_forbidden: true,
        governance_authority_forbidden: true,
        trading_authority_forbidden: true,
        registration_digest: String::new(),
    };
    registration.registration_digest = screening_registration_digest(&registration);
    validate_screening_registration(&registration)?;
    let mut counts = (0, 0);
    for boundary in &boundaries {
        add_counts(
            &mut counts,
            persist_one(
                "task_boundaries",
                &boundary.boundary_digest,
                &encode_boundary(boundary)?,
                |bytes| Ok(decode_boundary(bytes)?.boundary_digest),
            )?,
        );
    }
    for task in &registration.task_registrations {
        add_counts(
            &mut counts,
            persist_one(
                "task_registrations",
                &task.task_digest,
                &encode_task(task)?,
                |bytes| Ok(decode_task(bytes)?.task_digest),
            )?,
        );
    }
    for participant in &registration.participant_registrations {
        add_counts(
            &mut counts,
            persist_one(
                "participant_registrations",
                &participant.participant_digest,
                &encode_participant(participant)?,
                |bytes| Ok(decode_participant(bytes)?.participant_digest),
            )?,
        );
    }
    add_counts(
        &mut counts,
        persist_one(
            "screening_gates",
            &gate.gate_digest,
            &encode_gate(&gate)?,
            |bytes| Ok(decode_gate(bytes)?.gate_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_one(
            "screening_registrations",
            &registration.registration_digest,
            &encode_screening_registration(&registration)?,
            |bytes| Ok(decode_screening_registration(bytes)?.registration_digest),
        )?,
    );
    let mut journal = MomentumMicroScreeningJournalV1 {
        journal_version: SCREENING_JOURNAL_VERSION.to_string(),
        registration_digest: registration.registration_digest.clone(),
        task_registration_digests: registration
            .task_registrations
            .iter()
            .map(|value| value.task_digest.clone())
            .collect(),
        participant_registration_digests: registration
            .participant_registrations
            .iter()
            .map(|value| value.participant_digest.clone())
            .collect(),
        screening_gate_digest: gate.gate_digest.clone(),
        model_execution_count: 0,
        holdout_execution_count: 0,
        journal_digest: String::new(),
    };
    journal.journal_digest = screening_journal_digest(&journal);
    add_counts(
        &mut counts,
        persist_one(
            "screening_journals",
            &journal.journal_digest,
            &encode_screening_journal(&journal)?,
            |bytes| Ok(decode_screening_journal(bytes)?.journal_digest),
        )?,
    );
    report.complete = true;
    report.task_boundaries = boundaries;
    report.screening_registration = Some(registration);
    report.screening_gate = Some(gate);
    report.journal_digest = Some(journal.journal_digest);
    report.artifacts_written = counts.0 + 1;
    report.duplicate_artifact_count = counts.1;
    report.runtime_duration_ms = started.elapsed().as_millis() as u64;
    report.report_digest = final_report_digest(&report);
    validate_final_report(&report)?;
    add_counts(
        &mut counts,
        persist_one(
            "final_reports",
            &report.report_digest,
            &encode_final_report(&report)?,
            |bytes| Ok(decode_final_report(bytes)?.report_digest),
        )?,
    );
    if counts.0 != report.artifacts_written {
        return Err("micro registration artifact accounting mismatch".to_string());
    }
    if read_momentum_micro_challenger_design_report_v1()?.as_ref() != Some(&report) {
        return Err("micro challenger final report reopen mismatch".to_string());
    }
    Ok(report)
}

pub fn run_momentum_micro_challenger_registration_v1(
    mode: MomentumMicroChallengerRegistrationRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> Result<MomentumMicroChallengerDesignReportV1, String> {
    run_registration_inner(mode, protected)
}

pub fn format_momentum_micro_feature_forensics_text_v1(
    report: &MomentumMicroFeatureForensicsReportV1,
) -> String {
    use std::fmt::Write as _;
    let mut output = String::new();
    let _ = writeln!(output, "status={:?}", report.status);
    let _ = writeln!(
        output,
        "registration_digest={}",
        report.registration_digest.as_deref().unwrap_or("absent")
    );
    for audit in &report.source_feature_audits {
        let _ = writeln!(
            output,
            "source_feature_audit={} dimension={} constants={}",
            audit.participant_id, audit.feature_dimension, audit.constant_or_near_constant_count
        );
    }
    if let Some(policy) = &report.compact_feature_policy {
        let _ = writeln!(output, "compact_schema_digest={}", policy.schema_digest);
        let _ = writeln!(
            output,
            "compact_feature_dimension={}",
            policy.feature_dimension()
        );
    }
    let _ = writeln!(
        output,
        "holdout_feature_reads={}",
        report.safety_counters.holdout_feature_reads
    );
    let _ = writeln!(output, "report_digest={}", report.report_digest);
    output
}

pub fn format_momentum_micro_challenger_design_text_v1(
    report: &MomentumMicroChallengerDesignReportV1,
) -> String {
    use std::fmt::Write as _;
    let mut output = String::new();
    let _ = writeln!(output, "complete={}", report.complete);
    let _ = writeln!(
        output,
        "compact_feature_policy_digest={}",
        report.compact_feature_policy_digest
    );
    let _ = writeln!(
        output,
        "compact_feature_dimension={}",
        report.compact_feature_dimension
    );
    if let Some(registration) = &report.screening_registration {
        let _ = writeln!(
            output,
            "screening_registration_digest={}",
            registration.registration_digest
        );
        let _ = writeln!(
            output,
            "participant_registration_count={}",
            registration.participant_registrations.len()
        );
    }
    let _ = writeln!(
        output,
        "model_execution_authorized={}",
        report.model_execution_authorized
    );
    let _ = writeln!(
        output,
        "holdout_execution_authorized={}",
        report.holdout_execution_authorized
    );
    let _ = writeln!(output, "report_digest={}", report.report_digest);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compact_rows() -> Vec<MomentumQualifiedReplayCandleEvidenceV1> {
        (0..COMPACT_CONTEXT_LENGTH)
            .map(|index| MomentumQualifiedReplayCandleEvidenceV1 {
                timeframe: MomentumHistoricalTimeframeV1::Minute1,
                open_timestamp_ms: index as u64 * 60_000,
                close_exclusive_timestamp_ms: (index + 1) as u64 * 60_000,
                open: 100.0 + index as f64,
                high: 101.0 + index as f64,
                low: 99.0 + index as f64,
                close: 100.5 + index as f64,
                volume: 10.0 + index as f64,
                trade_value: 1_000.0 + index as f64,
                candle_digest: format!("candle-{index}"),
                missing_evidence: false,
            })
            .collect()
    }

    fn matrix_fixture() -> FeatureMatrix {
        FeatureMatrix {
            feature_ids: vec!["1m:a".into(), "3m:b".into()],
            development: (0..30)
                .map(|index| vec![index as f64, (index as f64).sqrt()])
                .collect(),
            validation: (30..40)
                .map(|index| vec![index as f64, (index as f64).sqrt()])
                .collect(),
        }
    }

    fn boundary_fixture(task: MomentumMicroTaskV1) -> MomentumMicroTaskPartitionBoundaryV1 {
        let cadence = match task {
            MomentumMicroTaskV1::T10NextTenMinuteDirection => TEN_MINUTE_MS,
            MomentumMicroTaskV1::T30NextThirtyMinuteDirection => 3 * TEN_MINUTE_MS,
        };
        derive_boundary(task, (1..=100).map(|index| index * cadence).collect()).unwrap()
    }

    fn task_and_participant_fixtures() -> (
        MomentumCompactMicroFeaturePolicyV1,
        Vec<MomentumMicroTaskRegistrationV1>,
        Vec<MomentumMicroParticipantRegistrationV1>,
    ) {
        let policy = compact_policy().unwrap();
        let tasks = MomentumMicroTaskV1::ORDERED
            .into_iter()
            .map(|task| task_registration(task, &policy, &boundary_fixture(task)))
            .collect::<Vec<_>>();
        let participants = participant_registrations(&tasks, &policy);
        (policy, tasks, participants)
    }

    fn protected_fixture() -> MomentumMicroProtectedBeforeStateV1 {
        let mut value = MomentumMicroProtectedBeforeStateV1 {
            series_digest: "series".into(),
            event_two_outcome_receipt_digest: "receipt".into(),
            event_two_outcome_capsule_digest: "capsule".into(),
            opening_authorization_digest: "authorization".into(),
            opening_bundle_digest: "bundle".into(),
            event_two_ledger_entry_digest: "ledger".into(),
            eligibility_receipt_digest: "eligibility".into(),
            completed_pause_digest: "pause".into(),
            completed_event_count: 2,
            scorable_event_count: 2,
            eligibility_status: "IneligibleMinimumSamples".into(),
            epoch_three_registered: false,
            live_parameter_digests: vec!["p1".into(), "p2".into(), "p3".into()],
            live_normalizer_digests: vec!["n1".into(), "n2".into(), "n3".into()],
            protected_live_aggregate_digest: "live".into(),
            historical_store_digest: "historical".into(),
            qualified_six_replay_digest: "replay".into(),
            diagnostic_store_digest: "diagnostic".into(),
            active_roster_digest: "roster".into(),
            zero_authority_and_action_counters: true,
            state_digest: String::new(),
        };
        value.state_digest =
            crate::model::momentum_micro_label_forensics_v1::
                momentum_micro_protected_before_state_digest_v1(&value);
        value
    }

    #[test]
    fn sprint101_20_compact_schema_is_exactly_four_micro_views_and_69_features() {
        let policy = compact_policy().unwrap();
        assert_eq!(policy.feature_dimension(), 69);
        assert_eq!(
            policy.included_timeframes,
            [
                MomentumHistoricalTimeframeV1::Minute1,
                MomentumHistoricalTimeframeV1::Minute3,
                MomentumHistoricalTimeframeV1::Minute5,
                MomentumHistoricalTimeframeV1::Minute10,
            ]
        );
        assert!(!policy.included_timeframes.iter().any(|value| matches!(
            value,
            MomentumHistoricalTimeframeV1::Day1
                | MomentumHistoricalTimeframeV1::Week1
                | MomentumHistoricalTimeframeV1::Month1
                | MomentumHistoricalTimeframeV1::Year1
        )));
    }

    #[test]
    fn sprint101_21_compact_timeframe_features_are_finite_and_past_only() {
        let (values, _) = timeframe_compact_features(&compact_rows()).unwrap();
        assert_eq!(values.len(), 16);
        assert!(values.iter().all(|value| value.is_finite()));
    }

    #[test]
    fn sprint101_22_zero_range_and_denominator_fallbacks_are_finite() {
        let mut rows = compact_rows();
        let last = rows.last_mut().unwrap();
        last.high = last.low;
        last.open = last.low;
        last.close = last.low;
        let (values, fallback) = timeframe_compact_features(&rows).unwrap();
        assert_eq!(fallback, 1);
        assert!(values.iter().all(|value| value.is_finite()));
        let mut denominator_fallbacks = 0;
        assert_eq!(guarded_ratio(1.0, 0.0, &mut denominator_fallbacks), 0.0);
        assert_eq!(denominator_fallbacks, 1);
    }

    #[test]
    fn sprint101_23_redundancy_threshold_is_fixed_before_validation() {
        let matrix = FeatureMatrix {
            feature_ids: vec!["1m:a".into(), "3m:a".into()],
            development: vec![vec![1.0, 2.0], vec![2.0, 4.0], vec![3.0, 6.0]],
            validation: vec![vec![4.0, 8.0], vec![5.0, 10.0]],
        };
        let audit = redundancy_audit("fixture", &matrix);
        assert_eq!(audit.absolute_pearson_threshold, 0.98);
        assert!(!audit.validation_modified_policy);
        assert_eq!(audit.correlated_feature_pair_count, 1);
    }

    #[test]
    fn sprint101_24_drift_bins_are_development_only() {
        let matrix = FeatureMatrix {
            feature_ids: vec!["a".into()],
            development: (0..20).map(|index| vec![index as f64]).collect(),
            validation: vec![vec![100.0]],
        };
        let policy = bin_policy("fixture", &matrix).unwrap();
        assert!(policy.development_only);
        assert_eq!(policy.validation_access_count_before_persist, 0);
        assert_eq!(policy.private_boundary_bits.len(), 9);
    }

    #[test]
    fn sprint101_25_tasks_and_participants_are_bounded_and_unexecuted() {
        let policy = compact_policy().unwrap();
        let boundary = derive_boundary(
            MomentumMicroTaskV1::T10NextTenMinuteDirection,
            (0..100).map(|index| index * TEN_MINUTE_MS).collect(),
        )
        .unwrap();
        let tasks = MomentumMicroTaskV1::ORDERED
            .into_iter()
            .map(|task| task_registration(task, &policy, &boundary))
            .collect::<Vec<_>>();
        let participants = participant_registrations(&tasks, &policy);
        assert_eq!(tasks.len(), 2);
        assert_eq!(participants.len(), 10);
        assert!(tasks.iter().all(|task| !task.t60_experiment));
        assert!(
            participants
                .iter()
                .all(|participant| !participant.model_execution_authorized)
        );
    }

    #[test]
    fn sprint101_26_c3_and_c4_policies_are_fixed() {
        let policy = compact_policy().unwrap();
        let boundary = derive_boundary(
            MomentumMicroTaskV1::T10NextTenMinuteDirection,
            (0..100).map(|index| index * TEN_MINUTE_MS).collect(),
        )
        .unwrap();
        let task = task_registration(
            MomentumMicroTaskV1::T10NextTenMinuteDirection,
            &policy,
            &boundary,
        );
        let participants = participant_registrations(&[task], &policy);
        let c3 = participants
            .iter()
            .find(|value| {
                value.participant == MomentumMicroParticipantV1::C3CompactMicroStrongShrinkLogistic
            })
            .unwrap();
        let c4 = participants
            .iter()
            .find(|value| {
                value.participant
                    == MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic
            })
            .unwrap();
        assert_eq!(c3.standard_l2_multiplier, 4);
        assert_eq!(
            (c4.calibration_base_fit_percent, c4.calibration_fit_percent),
            (80, 20)
        );
        assert!(c4.calibration_training_only);
        assert!(c4.validation_fit_forbidden && c4.holdout_fit_forbidden);
    }

    #[test]
    fn sprint101_27_screening_gate_requires_both_partitions() {
        let gate = screening_gate();
        assert!(gate.lower_brier_development_required);
        assert!(gate.lower_brier_validation_required);
        assert!(gate.correctness_override_forbidden);
        assert!(gate.holdout_access_forbidden);
    }

    #[test]
    fn sprint101_28_malformed_compact_policy_rejects() {
        let policy = compact_policy().unwrap();
        let mut bytes = encode_compact_policy(&policy).unwrap();
        bytes.truncate(bytes.len() / 2);
        assert!(decode_compact_policy(&bytes).is_err());
    }

    #[test]
    fn sprint101_29_compact_policy_round_trip_preserves_exact_order() {
        let policy = compact_policy().unwrap();
        let reopened = decode_compact_policy(&encode_compact_policy(&policy).unwrap()).unwrap();
        assert_eq!(reopened, policy);
        assert_eq!(reopened.per_timeframe_feature_ids.len(), 16);
    }

    #[test]
    fn sprint101_30_q1_source_policy_is_the_ten_minute_anchor() {
        assert_eq!(
            MomentumQualifiedParticipantV1::Q1TenMinuteAnchorLogistic.timeframes(),
            [MomentumHistoricalTimeframeV1::Minute10]
        );
    }

    #[test]
    fn sprint101_31_q2_source_policy_is_the_four_view_micro_block() {
        assert_eq!(
            MomentumQualifiedParticipantV1::Q2MicroBlockLogistic.timeframes(),
            [
                MomentumHistoricalTimeframeV1::Minute1,
                MomentumHistoricalTimeframeV1::Minute3,
                MomentumHistoricalTimeframeV1::Minute5,
                MomentumHistoricalTimeframeV1::Minute10,
            ]
        );
    }

    #[test]
    fn sprint101_32_source_audit_records_zero_holdout_access() {
        let audit = source_audit(
            "Q2",
            vec![MomentumHistoricalTimeframeV1::Minute1],
            &matrix_fixture(),
            "normalizer".into(),
            true,
        )
        .unwrap();
        assert_eq!(audit.holdout_access_count, 0);
        assert!(audit.source_candle_complete);
    }

    #[test]
    fn sprint101_33_validation_cannot_modify_redundancy_policy() {
        let audit = redundancy_audit("fixture", &matrix_fixture());
        assert!(!audit.validation_modified_policy);
        assert!(!audit.development_only_redundancy_identity.is_empty());
        assert!(!audit.validation_confirmation_identity.is_empty());
    }

    #[test]
    fn sprint101_34_development_bin_policy_round_trip_persists_boundaries() {
        let policy = bin_policy("fixture", &matrix_fixture()).unwrap();
        let reopened = decode_bin_policy(&encode_bin_policy(&policy).unwrap()).unwrap();
        assert_eq!(reopened, policy);
        assert_eq!(reopened.private_boundary_bits.len(), 18);
        assert_eq!(reopened.validation_access_count_before_persist, 0);
    }

    #[test]
    fn sprint101_35_feature_shift_receipts_include_bins_mean_and_std() {
        let matrix = matrix_fixture();
        let bins = bin_policy("fixture", &matrix).unwrap();
        let audit = shift_audit("fixture", &matrix, &bins).unwrap();
        assert_eq!(audit.feature_receipts.len(), 2);
        assert!(audit.feature_receipts.iter().all(|receipt| {
            receipt.development_bin_support.len() == 10
                && receipt.validation_bin_support.len() == 10
                && receipt.integrity_passed
        }));
        assert_eq!(decode_shift(&encode_shift(&audit).unwrap()).unwrap(), audit);
    }

    #[test]
    fn sprint101_36_compact_policy_excludes_all_macro_views() {
        let policy = compact_policy().unwrap();
        for excluded in [
            MomentumHistoricalTimeframeV1::Day1,
            MomentumHistoricalTimeframeV1::Week1,
            MomentumHistoricalTimeframeV1::Month1,
            MomentumHistoricalTimeframeV1::Year1,
        ] {
            assert!(!policy.included_timeframes.contains(&excluded));
        }
    }

    #[test]
    fn sprint101_37_return_features_use_only_closed_context_prices() {
        let rows = compact_rows();
        let (values, _) = timeframe_compact_features(&rows).unwrap();
        let expected = (rows.last().unwrap().close / rows[rows.len() - 2].close).ln();
        assert_eq!(values[0].to_bits(), expected.to_bits());
    }

    #[test]
    fn sprint101_38_realized_volatility_features_are_finite_and_nonnegative() {
        let (values, _) = timeframe_compact_features(&compact_rows()).unwrap();
        assert!(
            values[4..=6]
                .iter()
                .all(|value| value.is_finite() && *value >= 0.0)
        );
    }

    #[test]
    fn sprint101_39_volume_features_are_fixed_without_target_selection() {
        let policy = compact_policy().unwrap();
        assert_eq!(
            &policy.per_timeframe_feature_ids[12..15],
            [
                "log_volume_change_1",
                "volume_zscore_16",
                "trade_value_zscore_16",
            ]
        );
        assert!(!policy.target_selected_features);
    }

    #[test]
    fn sprint101_40_trend_slope_is_deterministic() {
        let first = timeframe_compact_features(&compact_rows()).unwrap().0[15];
        let second = timeframe_compact_features(&compact_rows()).unwrap().0[15];
        assert_eq!(first.to_bits(), second.to_bits());
    }

    #[test]
    fn sprint101_41_cross_timeframe_feature_list_is_exact() {
        assert_eq!(
            compact_policy().unwrap().cross_timeframe_feature_ids,
            [
                "return_sign_agreement_count",
                "latest_return_dispersion",
                "realized_volatility_1m_over_10m",
                "realized_volatility_3m_over_10m",
                "normalized_volume_1m_over_10m",
            ]
        );
    }

    #[test]
    fn sprint101_42_compact_integrity_receipt_proves_zero_forbidden_access() {
        let policy = compact_policy().unwrap();
        let mut receipt = MomentumCompactMicroIntegrityReplayV1 {
            replay_version: COMPACT_REPLAY_VERSION.into(),
            partition: MomentumReplayPartitionV1::Development,
            eligible_event_count: 10,
            finite_block_count: 10,
            missing_evidence_count: 0,
            partial_candle_count: 0,
            zero_range_fallback_count: 0,
            zero_denominator_fallback_count: 0,
            constant_feature_count: 0,
            redundancy_audit_digest: "redundancy".into(),
            distribution_shift_audit_digest: "shift".into(),
            feature_schema_digest: policy.schema_digest,
            future_access_count: 0,
            partial_access_count: 0,
            holdout_access_count: 0,
            deterministic_replay_digest: String::new(),
        };
        receipt.deterministic_replay_digest = compact_replay_digest(&receipt);
        assert_eq!(
            decode_compact_replay(&encode_compact_replay(&receipt).unwrap()).unwrap(),
            receipt
        );
    }

    #[test]
    fn sprint101_43_t10_and_t30_registrations_are_distinct() {
        let (_, tasks, _) = task_and_participant_fixtures();
        assert_ne!(tasks[0].task_digest, tasks[1].task_digest);
        assert_ne!(tasks[0].boundary_digest, tasks[1].boundary_digest);
        assert_eq!(
            (
                tasks[0].target_horizon_candles,
                tasks[1].target_horizon_candles
            ),
            (1, 3)
        );
    }

    #[test]
    fn sprint101_44_t60_remains_diagnostic_only() {
        let (_, tasks, _) = task_and_participant_fixtures();
        assert_eq!(tasks.len(), 2);
        assert!(tasks.iter().all(|task| !task.t60_experiment));
    }

    #[test]
    fn sprint101_45_c0_is_mandatory_for_each_task() {
        let (_, _, participants) = task_and_participant_fixtures();
        assert_eq!(
            participants
                .iter()
                .filter(|value| {
                    value.participant == MomentumMicroParticipantV1::C0TaskSpecificConstant
                })
                .count(),
            2
        );
    }

    #[test]
    fn sprint101_46_c1_anchor_parameters_are_task_specific() {
        let (_, _, participants) = task_and_participant_fixtures();
        let anchors = participants
            .iter()
            .filter(|value| {
                value.participant == MomentumMicroParticipantV1::C1TenMinuteAnchorBaseline
            })
            .collect::<Vec<_>>();
        assert_eq!(anchors.len(), 2);
        assert_ne!(
            anchors[0].feature_policy_digest,
            anchors[1].feature_policy_digest
        );
        assert!(
            anchors
                .iter()
                .all(|value| value.fresh_task_parameters_required)
        );
    }

    #[test]
    fn sprint101_47_c2_uses_the_compact_schema_exactly() {
        let (policy, _, participants) = task_and_participant_fixtures();
        assert!(
            participants
                .iter()
                .filter(|value| {
                    value.participant == MomentumMicroParticipantV1::C2CompactMicroLogistic
                })
                .all(|value| value.feature_policy_digest == policy.schema_digest)
        );
    }

    #[test]
    fn sprint101_48_c3_uses_fixed_four_times_strong_shrinkage() {
        let (policy, _, participants) = task_and_participant_fixtures();
        assert!(
            participants
                .iter()
                .filter(|value| {
                    value.participant
                        == MomentumMicroParticipantV1::C3CompactMicroStrongShrinkLogistic
                })
                .all(|value| {
                    value.feature_policy_digest == policy.schema_digest
                        && value.standard_l2_multiplier == 4
                })
        );
    }

    #[test]
    fn sprint101_49_c4_uses_nested_training_only_calibration() {
        let (_, _, participants) = task_and_participant_fixtures();
        assert!(
            participants
                .iter()
                .filter(|value| {
                    value.participant
                        == MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic
                })
                .all(|value| {
                    value.calibration_base_fit_percent == 80
                        && value.calibration_fit_percent == 20
                        && value.calibration_training_only
                })
        );
    }

    #[test]
    fn sprint101_50_validation_and_holdout_cannot_fit_calibration() {
        let (_, _, participants) = task_and_participant_fixtures();
        assert!(
            participants
                .iter()
                .all(|value| value.validation_fit_forbidden && value.holdout_fit_forbidden)
        );
    }

    #[test]
    fn sprint101_51_screening_gate_cannot_override_two_partition_brier() {
        let gate = screening_gate();
        assert!(gate.lower_brier_development_required);
        assert!(gate.lower_brier_validation_required);
        assert!(gate.correctness_override_forbidden);
        assert!(gate.result_selected_mutation_forbidden);
    }

    #[test]
    fn sprint101_52_all_design_authority_and_action_counters_are_zero() {
        let counters = MomentumMicroDesignSafetyCountersV1::default();
        assert!(zero_safety(&counters));
        assert_eq!(counters.reward_applications, 0);
        assert_eq!(counters.chair_actions, 0);
        assert_eq!(counters.live_network_requests, 0);
    }

    #[test]
    fn sprint101_53_completed_report_digest_is_idempotent_across_runtime_counters() {
        let protected = protected_fixture();
        let mut report =
            empty_feature_report(MomentumMicroFeatureForensicsRunModeV1::Status, &protected);
        report.report_digest = feature_report_digest(&report);
        let digest = report.report_digest.clone();
        report.artifacts_written = 99;
        report.feature_computation_count = 99;
        report.runtime_duration_ms = 99;
        assert_eq!(feature_report_digest(&report), digest);
    }

    #[test]
    fn sprint101_54_conflicting_compact_policy_digest_rejects() {
        let mut policy = compact_policy().unwrap();
        policy.per_timeframe_feature_ids[0] = "changed".into();
        assert!(validate_compact_policy(&policy).is_err());
    }

    #[test]
    fn sprint101_55_manual_protobuf_round_trips_task_participant_and_boundary() {
        let (policy, tasks, participants) = task_and_participant_fixtures();
        let boundary = boundary_fixture(MomentumMicroTaskV1::T10NextTenMinuteDirection);
        assert_eq!(
            decode_boundary(&encode_boundary(&boundary).unwrap()).unwrap(),
            boundary
        );
        assert_eq!(
            decode_task(&encode_task(&tasks[0]).unwrap()).unwrap(),
            tasks[0]
        );
        assert_eq!(
            decode_participant(&encode_participant(&participants[0]).unwrap()).unwrap(),
            participants[0]
        );
        let gate = screening_gate();
        assert_eq!(decode_gate(&encode_gate(&gate).unwrap()).unwrap(), gate);
        let mut feature_journal = MomentumMicroFeatureForensicsJournalV1 {
            journal_version: FEATURE_JOURNAL_VERSION.into(),
            registration_digest: "registration".into(),
            source_audit_digests: vec!["source-1".into(), "source-2".into()],
            redundancy_audit_digests: vec![
                "redundancy-1".into(),
                "redundancy-2".into(),
                "redundancy-3".into(),
            ],
            partition_shift_audit_digests: vec![
                "shift-1".into(),
                "shift-2".into(),
                "shift-3".into(),
            ],
            normalizer_stability_audit_digests: vec!["normalizer-1".into(), "normalizer-2".into()],
            compact_feature_policy_digest: policy.schema_digest,
            compact_integrity_replay_digests: vec!["replay-1".into(), "replay-2".into()],
            holdout_access_count: 0,
            model_execution_count: 0,
            journal_digest: String::new(),
        };
        feature_journal.journal_digest = feature_journal_digest(&feature_journal);
        assert_eq!(
            decode_feature_journal(&encode_feature_journal(&feature_journal).unwrap()).unwrap(),
            feature_journal
        );
    }

    #[test]
    fn sprint101_56_text_and_json_expose_the_same_public_status_fields() {
        let protected = protected_fixture();
        let mut report =
            empty_feature_report(MomentumMicroFeatureForensicsRunModeV1::Status, &protected);
        report.report_digest = feature_report_digest(&report);
        let text = format_momentum_micro_feature_forensics_text_v1(&report);
        let json = serde_json::to_value(&report).unwrap();
        assert!(text.contains(&format!("status={:?}", report.status)));
        assert!(text.contains(json["report_digest"].as_str().unwrap()));
        assert!(text.contains(&format!(
            "holdout_feature_reads={}",
            json["safety_counters"]["holdout_feature_reads"]
                .as_u64()
                .unwrap()
        )));
    }
}
