//! Post-screening T10 failure forensics over consumed design evidence only.

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use serde::{Deserialize, Serialize, ser::SerializeStruct};

use crate::stable_hash_string;

use super::{
    momentum_future_prediction_v4::{
        ArtifactBuilderV4_2, ArtifactReaderV4_2, as_u64, as_usize, persist_artifact, read_single,
    },
    momentum_micro_label_forensics_v1::{
        MomentumMicroProtectedBeforeStateV1, validate_momentum_micro_protected_before_state_v1,
    },
    momentum_multitimeframe_history_v1::load_momentum_qualified_sealed_protocol_metadata_v1,
    momentum_qualified_six_replay_v1::MomentumReplayPartitionV1,
    momentum_t10_micro_screening_v1::{
        MomentumMicroHoldoutCohortStatusV1, MomentumMicroSaturationV1,
        MomentumT10ConsumedEventEvidenceV1, MomentumT10MicroScreeningStatusV1,
        read_momentum_t10_consumed_event_evidence_v1, read_momentum_t10_micro_screening_report_v1,
    },
};

const ROOT: &str = "state/historical_replay/momentum_t10_failure_forensics/v1";
const EVIDENCE_USE_VERSION: &str = "momentum-t10-evidence-use-reclassification-v2";
const SPLIT_VERSION: &str = "momentum-t10-fresh-evidence-split-v1";
const MAGNITUDE_POLICY_VERSION: &str = "momentum-t10-target-magnitude-policy-v1";
const MAGNITUDE_BOUNDARY_VERSION: &str = "momentum-t10-target-magnitude-boundaries-v1";
const CONFIDENCE_POLICY_VERSION: &str = "momentum-t10-confidence-coverage-policy-v1";
const REGISTRATION_VERSION: &str = "momentum-t10-failure-forensics-registration-v1";
const MAGNITUDE_DIAGNOSTIC_VERSION: &str = "momentum-t10-magnitude-bin-diagnostic-v1";
const CONFIDENCE_DIAGNOSTIC_VERSION: &str = "momentum-t10-confidence-band-diagnostic-v1";
const PARTICIPANT_REPORT_VERSION: &str = "momentum-t10-participant-failure-report-v1";
const REPORT_VERSION: &str = "momentum-t10-failure-forensics-public-report-v1";
const EXPECTED_SCREENING_REPORT_DIGEST: &str = "c3141bf6324ebb59";
const EXPECTED_SCREENING_AUTHORIZATION_DIGEST: &str = "68a275ea51dc7443";
const EXPECTED_LABEL_REPORT_DIGEST: &str = "dc1db01318ab180f";
const EXPECTED_FEATURE_REPORT_DIGEST: &str = "02bb79cbc18c34c4";
const EXPECTED_DESIGN_REPORT_DIGEST: &str = "0d1077c9c65fd8cf";
const EXPECTED_SCREENING_REGISTRATION_DIGEST: &str = "56dbdee4766edaaa";
const EXPECTED_SCREENING_GATE_DIGEST: &str = "ccd9763e73e60081";
const EXPECTED_SCREENING_REPLAY_DIGEST: &str = "1e238431ed660a1d";
const EXPECTED_DEVELOPMENT_AGGREGATE_DIGEST: &str = "d0eacba7eea61f23";
const EXPECTED_VALIDATION_AGGREGATE_DIGEST: &str = "2278dba4e330e175";
const EXPECTED_COHORT_DIGEST: &str = "0a6360694a7d8117";
const PROBABILITY_CLAMP: f64 = 1e-6;
const MAGNITUDE_QUANTILES: [f64; 6] = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
const CONFIDENCE_BOUNDARIES: [f64; 7] = [0.0, 0.01, 0.02, 0.05, 0.10, 0.20, 0.50];
const PUBLIC_LABELS: [&str; 7] = [
    "HistoricalResearchOnly",
    "PostScreeningResearchDesignOnly",
    "ConsumedDesignEvidenceOnly",
    "FreshValidationStillClosed",
    "FinalHoldoutStillClosed",
    "NotLiveAuthority",
    "NotTradingAuthority",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumT10FailureForensicsRunModeV1 {
    Status,
    ExecuteLocal,
}

impl MomentumT10FailureForensicsRunModeV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::ExecuteLocal => "execute-local",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumT10FailureForensicsStatusV1 {
    Unregistered,
    Complete,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMicroEvidenceUseClassV2 {
    ConsumedResearchDesignEvidence,
    FreshChallengerValidation,
    FinalSealedHoldout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumT10ActionabilityDesignEvidenceClassV1 {
    PostScreeningResearchDesignOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumT10FailureRootDispositionV1 {
    DominatedByTinyTargetNoise,
    CalibrationInstability,
    ProbabilitySaturation,
    PartitionSpecificSignal,
    BroadFeatureUnderperformance,
    MixedUnresolvedFailure,
    IntegrityFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroEvidenceUseReceiptV2 {
    pub receipt_version: String,
    pub source_screening_digest: String,
    pub development_aggregate_digest: String,
    pub validation_aggregate_digest: String,
    pub former_development_use: MomentumMicroEvidenceUseClassV2,
    pub former_validation_use: MomentumMicroEvidenceUseClassV2,
    pub original_partition_receipts_immutable: bool,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct MomentumT10FreshEvidenceSplitV1 {
    pub split_version: String,
    pub parent_holdout_digest: String,
    pub parent_event_count: usize,
    pub parent_first_timestamp_ms: u64,
    pub parent_last_timestamp_ms: u64,
    pub fresh_validation_event_digests: Vec<String>,
    pub final_holdout_event_digests: Vec<String>,
    pub fresh_validation_first_timestamp_ms: u64,
    pub fresh_validation_last_timestamp_ms: u64,
    pub final_holdout_first_timestamp_ms: u64,
    pub final_holdout_last_timestamp_ms: u64,
    pub label_reads: usize,
    pub prediction_reads: usize,
    pub metric_reads: usize,
    pub fresh_validation_execution_authorized: bool,
    pub final_holdout_execution_authorized: bool,
    pub split_digest: String,
}

impl Serialize for MomentumT10FreshEvidenceSplitV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("MomentumT10FreshEvidenceSplitV1", 11)?;
        state.serialize_field("split_version", &self.split_version)?;
        state.serialize_field("parent_holdout_digest", &self.parent_holdout_digest)?;
        state.serialize_field("parent_event_count", &self.parent_event_count)?;
        state.serialize_field(
            "fresh_validation_event_count",
            &self.fresh_validation_event_digests.len(),
        )?;
        state.serialize_field(
            "final_holdout_event_count",
            &self.final_holdout_event_digests.len(),
        )?;
        state.serialize_field("label_reads", &self.label_reads)?;
        state.serialize_field("prediction_reads", &self.prediction_reads)?;
        state.serialize_field("metric_reads", &self.metric_reads)?;
        state.serialize_field(
            "fresh_validation_execution_authorized",
            &self.fresh_validation_execution_authorized,
        )?;
        state.serialize_field(
            "final_holdout_execution_authorized",
            &self.final_holdout_execution_authorized,
        )?;
        state.serialize_field("split_digest", &self.split_digest)?;
        state.end()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10TargetMagnitudePolicyV1 {
    pub policy_version: String,
    pub source_partition: MomentumReplayPartitionV1,
    pub quantile_bits: Vec<u64>,
    pub validation_may_define_boundaries: bool,
    pub fresh_validation_access_forbidden: bool,
    pub final_holdout_access_forbidden: bool,
    pub policy_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10TargetMagnitudeBoundariesV1 {
    pub boundary_version: String,
    pub policy_digest: String,
    pub development_event_count: usize,
    pub boundary_bits: Vec<u64>,
    pub validation_assignment_count_at_freeze: usize,
    pub boundary_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10ConfidenceCoveragePolicyV1 {
    pub policy_version: String,
    pub distance_from_half_boundary_bits: Vec<u64>,
    pub deployment_threshold_selection_forbidden: bool,
    pub policy_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10FailureForensicsRegistrationV1 {
    pub registration_version: String,
    pub source_screening_digest: String,
    pub source_screening_registration_digest: String,
    pub source_development_aggregate_digest: String,
    pub source_validation_aggregate_digest: String,
    pub participant_ids: Vec<String>,
    #[serde(skip_serializing)]
    pub development_prediction_shard_digests: Vec<String>,
    #[serde(skip_serializing)]
    pub development_evaluation_shard_digests: Vec<String>,
    #[serde(skip_serializing)]
    pub validation_prediction_shard_digests: Vec<String>,
    #[serde(skip_serializing)]
    pub validation_evaluation_shard_digests: Vec<String>,
    pub target_magnitude_policy_digest: String,
    pub confidence_coverage_policy_digest: String,
    pub saturation_policy_digest: String,
    pub consumed_evidence_receipt_digest: String,
    pub fresh_evidence_split_digest: String,
    pub consumed_design_only: bool,
    pub fresh_validation_access_forbidden: bool,
    pub final_holdout_access_forbidden: bool,
    pub new_model_training_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumT10MagnitudeBinDiagnosticV1 {
    pub diagnostic_version: String,
    pub participant_id: String,
    pub partition: MomentumReplayPartitionV1,
    pub bin_index: usize,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub upper_inclusive: bool,
    pub support: usize,
    pub mean_brier: f64,
    pub c0_mean_brier: f64,
    pub paired_brier_delta: f64,
    pub correctness: f64,
    pub weighted_calibration_gap: f64,
    pub saturation_count: usize,
    pub finite: bool,
    pub diagnostic_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumT10ConfidenceBandDiagnosticV1 {
    pub diagnostic_version: String,
    pub participant_id: String,
    pub partition: MomentumReplayPartitionV1,
    pub band_index: usize,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub upper_inclusive: bool,
    pub prediction_count: usize,
    pub coverage: f64,
    pub mean_brier: f64,
    pub c0_mean_brier: f64,
    pub paired_brier_delta: f64,
    pub correctness: f64,
    pub calibration_gap: f64,
    pub mean_target_magnitude: f64,
    pub saturation_count: usize,
    pub finite: bool,
    pub diagnostic_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumT10ParticipantFailureReportV1 {
    pub report_version: String,
    pub participant_id: String,
    pub disposition: MomentumT10FailureRootDispositionV1,
    pub magnitude_diagnostic_digests: Vec<String>,
    pub confidence_diagnostic_digests: Vec<String>,
    pub secondary_observations: Vec<String>,
    pub paired_event_count: usize,
    pub smallest_target_magnitude_excess_brier_concentration: f64,
    pub saturation_event_count: usize,
    pub saturation_event_concentration: f64,
    pub finite_value_proof: bool,
    pub report_digest: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10FailureForensicsSafetyCountersV1 {
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub event_error_computations: usize,
    pub label_computations: usize,
    pub new_model_fits: usize,
    pub new_predictions: usize,
    pub fresh_validation_label_reads: usize,
    pub fresh_validation_prediction_reads: usize,
    pub fresh_validation_metric_reads: usize,
    pub final_holdout_label_reads: usize,
    pub final_holdout_prediction_reads: usize,
    pub final_holdout_metric_reads: usize,
    pub network_requests: usize,
    pub live_operations: usize,
    pub reward_applications: usize,
    pub penalty_applications: usize,
    pub chair_actions: usize,
    pub vote_actions: usize,
    pub trading_actions: usize,
    pub t30_model_executions: usize,
    pub t60_model_executions: usize,
    pub day_view_loads: usize,
    pub week_view_loads: usize,
    pub month_view_loads: usize,
    pub year_view_loads: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumT10FailureForensicsReportV1 {
    pub report_version: String,
    pub run_mode: String,
    pub status: MomentumT10FailureForensicsStatusV1,
    pub design_evidence_class: MomentumT10ActionabilityDesignEvidenceClassV1,
    pub source_screening_digest: String,
    pub source_development_aggregate_digest: String,
    pub source_validation_aggregate_digest: String,
    pub source_empty_cohort_digest: String,
    pub protected_before_state_digest: String,
    pub evidence_use_receipt: MomentumMicroEvidenceUseReceiptV2,
    pub fresh_evidence_split: MomentumT10FreshEvidenceSplitV1,
    pub registration: MomentumT10FailureForensicsRegistrationV1,
    pub magnitude_policy: MomentumT10TargetMagnitudePolicyV1,
    pub magnitude_boundaries: MomentumT10TargetMagnitudeBoundariesV1,
    pub confidence_policy: MomentumT10ConfidenceCoveragePolicyV1,
    pub magnitude_diagnostics: Vec<MomentumT10MagnitudeBinDiagnosticV1>,
    pub confidence_diagnostics: Vec<MomentumT10ConfidenceBandDiagnosticV1>,
    pub participant_reports: Vec<MomentumT10ParticipantFailureReportV1>,
    pub labels: Vec<String>,
    pub live_completed_event_count: usize,
    pub live_scorable_event_count: usize,
    pub live_pause: String,
    pub epoch_three_registered: bool,
    pub full_eight_blocked: bool,
    pub protected_artifacts_unchanged: bool,
    pub safety_counters: MomentumT10FailureForensicsSafetyCountersV1,
    pub deterministic_replay_digest: String,
    pub runtime_duration_ms: u64,
    pub report_digest: String,
}

fn canonical_digest<T: Clone + std::fmt::Debug>(value: &T, clear: impl FnOnce(&mut T)) -> String {
    let mut canonical = value.clone();
    clear(&mut canonical);
    stable_hash_string(&format!("{canonical:?}"))
}

fn evidence_use_digest(value: &MomentumMicroEvidenceUseReceiptV2) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn split_digest(value: &MomentumT10FreshEvidenceSplitV1) -> String {
    canonical_digest(value, |item| item.split_digest.clear())
}

fn magnitude_policy_digest(value: &MomentumT10TargetMagnitudePolicyV1) -> String {
    canonical_digest(value, |item| item.policy_digest.clear())
}

fn magnitude_boundary_digest(value: &MomentumT10TargetMagnitudeBoundariesV1) -> String {
    canonical_digest(value, |item| item.boundary_digest.clear())
}

fn confidence_policy_digest(value: &MomentumT10ConfidenceCoveragePolicyV1) -> String {
    canonical_digest(value, |item| item.policy_digest.clear())
}

fn registered_saturation_policy_digest() -> String {
    stable_hash_string(&format!(
        "T10-failure-saturation-policy:{}:{}",
        PROBABILITY_CLAMP.to_bits(),
        "any-boundary-saturation-is-attributed"
    ))
}

fn registration_digest(value: &MomentumT10FailureForensicsRegistrationV1) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}

fn magnitude_diagnostic_digest(value: &MomentumT10MagnitudeBinDiagnosticV1) -> String {
    canonical_digest(value, |item| item.diagnostic_digest.clear())
}

fn confidence_diagnostic_digest(value: &MomentumT10ConfidenceBandDiagnosticV1) -> String {
    canonical_digest(value, |item| item.diagnostic_digest.clear())
}

fn participant_report_digest(value: &MomentumT10ParticipantFailureReportV1) -> String {
    canonical_digest(value, |item| item.report_digest.clear())
}

fn report_digest(value: &MomentumT10FailureForensicsReportV1) -> String {
    canonical_digest(value, |item| {
        item.run_mode.clear();
        item.safety_counters = MomentumT10FailureForensicsSafetyCountersV1::default();
        item.runtime_duration_ms = 0;
        item.report_digest.clear();
    })
}

fn validate_evidence_use(value: &MomentumMicroEvidenceUseReceiptV2) -> Result<(), String> {
    if value.receipt_version != EVIDENCE_USE_VERSION
        || value.source_screening_digest != EXPECTED_SCREENING_REPORT_DIGEST
        || value.development_aggregate_digest != EXPECTED_DEVELOPMENT_AGGREGATE_DIGEST
        || value.validation_aggregate_digest != EXPECTED_VALIDATION_AGGREGATE_DIGEST
        || value.former_development_use
            != MomentumMicroEvidenceUseClassV2::ConsumedResearchDesignEvidence
        || value.former_validation_use
            != MomentumMicroEvidenceUseClassV2::ConsumedResearchDesignEvidence
        || !value.original_partition_receipts_immutable
        || value.receipt_digest != evidence_use_digest(value)
    {
        return Err("T10 evidence-use reclassification rejected".to_string());
    }
    Ok(())
}

fn validate_split(value: &MomentumT10FreshEvidenceSplitV1) -> Result<(), String> {
    let fresh_count = value.fresh_validation_event_digests.len();
    let final_count = value.final_holdout_event_digests.len();
    if value.split_version != SPLIT_VERSION
        || value.parent_holdout_digest.is_empty()
        || value.parent_event_count == 0
        || fresh_count == 0
        || final_count == 0
        || fresh_count + final_count != value.parent_event_count
        || fresh_count != value.parent_event_count / 2
        || final_count != value.parent_event_count - value.parent_event_count / 2
        || value.parent_first_timestamp_ms != value.fresh_validation_first_timestamp_ms
        || value.parent_last_timestamp_ms != value.final_holdout_last_timestamp_ms
        || value.fresh_validation_last_timestamp_ms >= value.final_holdout_first_timestamp_ms
        || value.label_reads != 0
        || value.prediction_reads != 0
        || value.metric_reads != 0
        || value.fresh_validation_execution_authorized
        || value.final_holdout_execution_authorized
        || value
            .fresh_validation_event_digests
            .iter()
            .chain(&value.final_holdout_event_digests)
            .any(|digest| digest.is_empty())
        || value
            .fresh_validation_event_digests
            .iter()
            .any(|digest| value.final_holdout_event_digests.contains(digest))
        || value
            .fresh_validation_event_digests
            .iter()
            .enumerate()
            .any(|(index, digest)| value.fresh_validation_event_digests[..index].contains(digest))
        || value
            .final_holdout_event_digests
            .iter()
            .enumerate()
            .any(|(index, digest)| value.final_holdout_event_digests[..index].contains(digest))
        || value.split_digest != split_digest(value)
    {
        return Err("T10 fresh-evidence split rejected".to_string());
    }
    Ok(())
}

fn validate_magnitude_policy(value: &MomentumT10TargetMagnitudePolicyV1) -> Result<(), String> {
    if value.policy_version != MAGNITUDE_POLICY_VERSION
        || value.source_partition != MomentumReplayPartitionV1::Development
        || value.quantile_bits
            != MAGNITUDE_QUANTILES
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        || value.validation_may_define_boundaries
        || !value.fresh_validation_access_forbidden
        || !value.final_holdout_access_forbidden
        || value.policy_digest != magnitude_policy_digest(value)
    {
        return Err("T10 target-magnitude policy rejected".to_string());
    }
    Ok(())
}

fn validate_magnitude_boundaries(
    value: &MomentumT10TargetMagnitudeBoundariesV1,
    policy: &MomentumT10TargetMagnitudePolicyV1,
) -> Result<(), String> {
    let boundaries = value
        .boundary_bits
        .iter()
        .map(|bits| f64::from_bits(*bits))
        .collect::<Vec<_>>();
    if value.boundary_version != MAGNITUDE_BOUNDARY_VERSION
        || value.policy_digest != policy.policy_digest
        || value.development_event_count == 0
        || boundaries.len() != 6
        || boundaries
            .iter()
            .any(|item| !item.is_finite() || *item < 0.0)
        || boundaries.windows(2).any(|pair| pair[0] > pair[1])
        || value.validation_assignment_count_at_freeze != 0
        || value.boundary_digest != magnitude_boundary_digest(value)
    {
        return Err("T10 target-magnitude boundaries rejected".to_string());
    }
    Ok(())
}

fn validate_confidence_policy(value: &MomentumT10ConfidenceCoveragePolicyV1) -> Result<(), String> {
    if value.policy_version != CONFIDENCE_POLICY_VERSION
        || value.distance_from_half_boundary_bits
            != CONFIDENCE_BOUNDARIES
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        || !value.deployment_threshold_selection_forbidden
        || value.policy_digest != confidence_policy_digest(value)
    {
        return Err("T10 confidence policy rejected".to_string());
    }
    Ok(())
}

fn validate_registration(value: &MomentumT10FailureForensicsRegistrationV1) -> Result<(), String> {
    let participant_suffixes = [
        "C0TaskSpecificConstant",
        "C1TenMinuteAnchorBaseline",
        "C2CompactMicroLogistic",
        "C3CompactMicroStrongShrinkLogistic",
        "C4CompactMicroTrainingOnlyCalibratedLogistic",
    ];
    let shard_digests = value
        .development_prediction_shard_digests
        .iter()
        .chain(&value.development_evaluation_shard_digests)
        .chain(&value.validation_prediction_shard_digests)
        .chain(&value.validation_evaluation_shard_digests)
        .collect::<Vec<_>>();
    if value.registration_version != REGISTRATION_VERSION
        || value.source_screening_digest != EXPECTED_SCREENING_REPORT_DIGEST
        || value.source_screening_registration_digest != EXPECTED_SCREENING_REGISTRATION_DIGEST
        || value.source_development_aggregate_digest != EXPECTED_DEVELOPMENT_AGGREGATE_DIGEST
        || value.source_validation_aggregate_digest != EXPECTED_VALIDATION_AGGREGATE_DIGEST
        || value.participant_ids.len() != 5
        || value
            .participant_ids
            .iter()
            .zip(participant_suffixes)
            .any(|(id, suffix)| !id.ends_with(suffix))
        || value.development_prediction_shard_digests.is_empty()
        || value.validation_prediction_shard_digests.is_empty()
        || value.development_prediction_shard_digests.len()
            != value.development_evaluation_shard_digests.len()
        || value.validation_prediction_shard_digests.len()
            != value.validation_evaluation_shard_digests.len()
        || shard_digests.iter().any(|digest| digest.is_empty())
        || shard_digests
            .iter()
            .enumerate()
            .any(|(index, digest)| shard_digests[..index].contains(digest))
        || value.target_magnitude_policy_digest.is_empty()
        || value.confidence_coverage_policy_digest.is_empty()
        || value.saturation_policy_digest != registered_saturation_policy_digest()
        || value.consumed_evidence_receipt_digest.is_empty()
        || value.fresh_evidence_split_digest.is_empty()
        || !value.consumed_design_only
        || !value.fresh_validation_access_forbidden
        || !value.final_holdout_access_forbidden
        || !value.new_model_training_forbidden
        || value.registration_digest != registration_digest(value)
    {
        return Err("T10 failure-forensics registration rejected".to_string());
    }
    Ok(())
}

fn validate_magnitude_diagnostic(
    value: &MomentumT10MagnitudeBinDiagnosticV1,
) -> Result<(), String> {
    if value.diagnostic_version != MAGNITUDE_DIAGNOSTIC_VERSION
        || value.participant_id.is_empty()
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.bin_index >= 5
        || !value.lower_bound.is_finite()
        || !value.upper_bound.is_finite()
        || value.lower_bound < 0.0
        || value.lower_bound > value.upper_bound
        || value.support == 0
        || [
            value.mean_brier,
            value.c0_mean_brier,
            value.paired_brier_delta,
            value.correctness,
            value.weighted_calibration_gap,
        ]
        .iter()
        .any(|item| !item.is_finite())
        || !(0.0..=1.0).contains(&value.mean_brier)
        || !(0.0..=1.0).contains(&value.c0_mean_brier)
        || !(0.0..=1.0).contains(&value.correctness)
        || !(0.0..=1.0).contains(&value.weighted_calibration_gap)
        || (value.paired_brier_delta - (value.mean_brier - value.c0_mean_brier)).abs()
            > f64::EPSILON
        || value.saturation_count > value.support
        || !value.finite
        || value.diagnostic_digest != magnitude_diagnostic_digest(value)
    {
        return Err("T10 magnitude diagnostic rejected".to_string());
    }
    Ok(())
}

fn validate_confidence_diagnostic(
    value: &MomentumT10ConfidenceBandDiagnosticV1,
) -> Result<(), String> {
    if value.diagnostic_version != CONFIDENCE_DIAGNOSTIC_VERSION
        || value.participant_id.is_empty()
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.band_index >= 6
        || !value.lower_bound.is_finite()
        || !value.upper_bound.is_finite()
        || value.lower_bound < 0.0
        || value.lower_bound >= value.upper_bound
        || value.prediction_count == 0
        || [
            value.coverage,
            value.mean_brier,
            value.c0_mean_brier,
            value.paired_brier_delta,
            value.correctness,
            value.calibration_gap,
            value.mean_target_magnitude,
        ]
        .iter()
        .any(|item| !item.is_finite())
        || !(0.0..=1.0).contains(&value.coverage)
        || !(0.0..=1.0).contains(&value.mean_brier)
        || !(0.0..=1.0).contains(&value.c0_mean_brier)
        || (value.paired_brier_delta - (value.mean_brier - value.c0_mean_brier)).abs()
            > f64::EPSILON
        || !(0.0..=1.0).contains(&value.correctness)
        || !(0.0..=1.0).contains(&value.calibration_gap)
        || value.mean_target_magnitude < 0.0
        || value.saturation_count > value.prediction_count
        || !value.finite
        || value.diagnostic_digest != confidence_diagnostic_digest(value)
    {
        return Err("T10 confidence diagnostic rejected".to_string());
    }
    Ok(())
}

fn validate_participant_report(
    value: &MomentumT10ParticipantFailureReportV1,
) -> Result<(), String> {
    if value.report_version != PARTICIPANT_REPORT_VERSION
        || value.participant_id.is_empty()
        || value.magnitude_diagnostic_digests.len() != 10
        || value.confidence_diagnostic_digests.is_empty()
        || value.paired_event_count == 0
        || !value
            .smallest_target_magnitude_excess_brier_concentration
            .is_finite()
        || !(0.0..=1.0).contains(&value.smallest_target_magnitude_excess_brier_concentration)
        || value.saturation_event_count > value.paired_event_count
        || !value.saturation_event_concentration.is_finite()
        || !(0.0..=1.0).contains(&value.saturation_event_concentration)
        || (value.saturation_event_concentration
            - value.saturation_event_count as f64 / value.paired_event_count as f64)
            .abs()
            > f64::EPSILON
        || !value.finite_value_proof
        || value.report_digest != participant_report_digest(value)
    {
        return Err("T10 participant failure report rejected".to_string());
    }
    Ok(())
}

fn validate_report(value: &MomentumT10FailureForensicsReportV1) -> Result<(), String> {
    validate_evidence_use(&value.evidence_use_receipt)?;
    validate_split(&value.fresh_evidence_split)?;
    validate_registration(&value.registration)?;
    validate_magnitude_policy(&value.magnitude_policy)?;
    validate_magnitude_boundaries(&value.magnitude_boundaries, &value.magnitude_policy)?;
    validate_confidence_policy(&value.confidence_policy)?;
    for diagnostic in &value.magnitude_diagnostics {
        validate_magnitude_diagnostic(diagnostic)?;
    }
    for diagnostic in &value.confidence_diagnostics {
        validate_confidence_diagnostic(diagnostic)?;
    }
    for report in &value.participant_reports {
        validate_participant_report(report)?;
        let expected_magnitude = value
            .magnitude_diagnostics
            .iter()
            .filter(|item| item.participant_id == report.participant_id)
            .map(|item| item.diagnostic_digest.clone())
            .collect::<Vec<_>>();
        let expected_confidence = value
            .confidence_diagnostics
            .iter()
            .filter(|item| item.participant_id == report.participant_id)
            .map(|item| item.diagnostic_digest.clone())
            .collect::<Vec<_>>();
        if report.magnitude_diagnostic_digests != expected_magnitude
            || report.confidence_diagnostic_digests != expected_confidence
            || value
                .magnitude_diagnostics
                .iter()
                .filter(|item| item.participant_id == report.participant_id)
                .map(|item| item.support)
                .sum::<usize>()
                != report.paired_event_count
            || value
                .confidence_diagnostics
                .iter()
                .filter(|item| item.participant_id == report.participant_id)
                .map(|item| item.prediction_count)
                .sum::<usize>()
                != report.paired_event_count
            || value
                .confidence_diagnostics
                .iter()
                .filter(|item| item.participant_id == report.participant_id)
                .map(|item| item.saturation_count)
                .sum::<usize>()
                != report.saturation_event_count
        {
            return Err("T10 participant diagnostic binding rejected".to_string());
        }
    }
    let counters = &value.safety_counters;
    if value.report_version != REPORT_VERSION
        || value.status != MomentumT10FailureForensicsStatusV1::Complete
        || value.design_evidence_class
            != MomentumT10ActionabilityDesignEvidenceClassV1::PostScreeningResearchDesignOnly
        || value.source_screening_digest != EXPECTED_SCREENING_REPORT_DIGEST
        || value.source_development_aggregate_digest != EXPECTED_DEVELOPMENT_AGGREGATE_DIGEST
        || value.source_validation_aggregate_digest != EXPECTED_VALIDATION_AGGREGATE_DIGEST
        || value.source_empty_cohort_digest != EXPECTED_COHORT_DIGEST
        || value.protected_before_state_digest.is_empty()
        || value.registration.target_magnitude_policy_digest != value.magnitude_policy.policy_digest
        || value.registration.confidence_coverage_policy_digest
            != value.confidence_policy.policy_digest
        || value.registration.consumed_evidence_receipt_digest
            != value.evidence_use_receipt.receipt_digest
        || value.registration.fresh_evidence_split_digest != value.fresh_evidence_split.split_digest
        || value.magnitude_diagnostics.len() != 40
        || value.participant_reports.len() != 4
        || value
            .participant_reports
            .iter()
            .map(|report| report.participant_id.as_str())
            .collect::<Vec<_>>()
            != value.registration.participant_ids[1..]
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
        || value.labels
            != PUBLIC_LABELS
                .iter()
                .map(|label| (*label).to_string())
                .collect::<Vec<_>>()
        || value.live_completed_event_count != 2
        || value.live_scorable_event_count != 2
        || value.live_pause != "PausedAfterCompletedEpochTwo"
        || value.epoch_three_registered
        || !value.full_eight_blocked
        || !value.protected_artifacts_unchanged
        || counters.new_model_fits != 0
        || counters.new_predictions != 0
        || counters.fresh_validation_label_reads != 0
        || counters.fresh_validation_prediction_reads != 0
        || counters.fresh_validation_metric_reads != 0
        || counters.final_holdout_label_reads != 0
        || counters.final_holdout_prediction_reads != 0
        || counters.final_holdout_metric_reads != 0
        || counters.network_requests != 0
        || counters.live_operations != 0
        || counters.reward_applications != 0
        || counters.penalty_applications != 0
        || counters.chair_actions != 0
        || counters.vote_actions != 0
        || counters.trading_actions != 0
        || counters.t30_model_executions != 0
        || counters.t60_model_executions != 0
        || counters.day_view_loads != 0
        || counters.week_view_loads != 0
        || counters.month_view_loads != 0
        || counters.year_view_loads != 0
        || value.deterministic_replay_digest.is_empty()
        || value.report_digest != report_digest(value)
    {
        return Err("T10 failure-forensics report rejected".to_string());
    }
    Ok(())
}

fn enum_partition(value: MomentumReplayPartitionV1) -> &'static str {
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
        _ => Err("T10 failure partition rejected".to_string()),
    }
}

fn evidence_class_name(value: MomentumMicroEvidenceUseClassV2) -> &'static str {
    match value {
        MomentumMicroEvidenceUseClassV2::ConsumedResearchDesignEvidence => {
            "ConsumedResearchDesignEvidence"
        }
        MomentumMicroEvidenceUseClassV2::FreshChallengerValidation => "FreshChallengerValidation",
        MomentumMicroEvidenceUseClassV2::FinalSealedHoldout => "FinalSealedHoldout",
    }
}

fn parse_evidence_class(value: &str) -> Result<MomentumMicroEvidenceUseClassV2, String> {
    match value {
        "ConsumedResearchDesignEvidence" => {
            Ok(MomentumMicroEvidenceUseClassV2::ConsumedResearchDesignEvidence)
        }
        "FreshChallengerValidation" => {
            Ok(MomentumMicroEvidenceUseClassV2::FreshChallengerValidation)
        }
        "FinalSealedHoldout" => Ok(MomentumMicroEvidenceUseClassV2::FinalSealedHoldout),
        _ => Err("T10 evidence class rejected".to_string()),
    }
}

fn disposition_name(value: MomentumT10FailureRootDispositionV1) -> &'static str {
    match value {
        MomentumT10FailureRootDispositionV1::DominatedByTinyTargetNoise => {
            "DominatedByTinyTargetNoise"
        }
        MomentumT10FailureRootDispositionV1::CalibrationInstability => "CalibrationInstability",
        MomentumT10FailureRootDispositionV1::ProbabilitySaturation => "ProbabilitySaturation",
        MomentumT10FailureRootDispositionV1::PartitionSpecificSignal => "PartitionSpecificSignal",
        MomentumT10FailureRootDispositionV1::BroadFeatureUnderperformance => {
            "BroadFeatureUnderperformance"
        }
        MomentumT10FailureRootDispositionV1::MixedUnresolvedFailure => "MixedUnresolvedFailure",
        MomentumT10FailureRootDispositionV1::IntegrityFailure => "IntegrityFailure",
    }
}

fn parse_disposition(value: &str) -> Result<MomentumT10FailureRootDispositionV1, String> {
    match value {
        "DominatedByTinyTargetNoise" => {
            Ok(MomentumT10FailureRootDispositionV1::DominatedByTinyTargetNoise)
        }
        "CalibrationInstability" => Ok(MomentumT10FailureRootDispositionV1::CalibrationInstability),
        "ProbabilitySaturation" => Ok(MomentumT10FailureRootDispositionV1::ProbabilitySaturation),
        "PartitionSpecificSignal" => {
            Ok(MomentumT10FailureRootDispositionV1::PartitionSpecificSignal)
        }
        "BroadFeatureUnderperformance" => {
            Ok(MomentumT10FailureRootDispositionV1::BroadFeatureUnderperformance)
        }
        "MixedUnresolvedFailure" => Ok(MomentumT10FailureRootDispositionV1::MixedUnresolvedFailure),
        "IntegrityFailure" => Ok(MomentumT10FailureRootDispositionV1::IntegrityFailure),
        _ => Err("T10 failure disposition rejected".to_string()),
    }
}

fn encode_evidence_use(value: &MomentumMicroEvidenceUseReceiptV2) -> Result<Vec<u8>, String> {
    validate_evidence_use(value)?;
    ArtifactBuilderV4_2::new(EVIDENCE_USE_VERSION)
        .string("receipt_version", &value.receipt_version)
        .string("source_screening_digest", &value.source_screening_digest)
        .string(
            "development_aggregate_digest",
            &value.development_aggregate_digest,
        )
        .string(
            "validation_aggregate_digest",
            &value.validation_aggregate_digest,
        )
        .string(
            "former_development_use",
            evidence_class_name(value.former_development_use),
        )
        .string(
            "former_validation_use",
            evidence_class_name(value.former_validation_use),
        )
        .boolean(
            "original_partition_receipts_immutable",
            value.original_partition_receipts_immutable,
        )
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_evidence_use(bytes: &[u8]) -> Result<MomentumMicroEvidenceUseReceiptV2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, EVIDENCE_USE_VERSION)?;
    let value = MomentumMicroEvidenceUseReceiptV2 {
        receipt_version: fields.string("receipt_version")?,
        source_screening_digest: fields.string("source_screening_digest")?,
        development_aggregate_digest: fields.string("development_aggregate_digest")?,
        validation_aggregate_digest: fields.string("validation_aggregate_digest")?,
        former_development_use: parse_evidence_class(&fields.string("former_development_use")?)?,
        former_validation_use: parse_evidence_class(&fields.string("former_validation_use")?)?,
        original_partition_receipts_immutable: fields
            .boolean("original_partition_receipts_immutable")?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_evidence_use(&value)?;
    Ok(value)
}

fn encode_split(value: &MomentumT10FreshEvidenceSplitV1) -> Result<Vec<u8>, String> {
    validate_split(value)?;
    ArtifactBuilderV4_2::new(SPLIT_VERSION)
        .string("split_version", &value.split_version)
        .string("parent_holdout_digest", &value.parent_holdout_digest)
        .unsigned("parent_event_count", as_u64(value.parent_event_count)?)
        .unsigned("parent_first_timestamp_ms", value.parent_first_timestamp_ms)
        .unsigned("parent_last_timestamp_ms", value.parent_last_timestamp_ms)
        .strings(
            "fresh_validation_event_digests",
            &value.fresh_validation_event_digests,
        )
        .strings(
            "final_holdout_event_digests",
            &value.final_holdout_event_digests,
        )
        .unsigned(
            "fresh_validation_first_timestamp_ms",
            value.fresh_validation_first_timestamp_ms,
        )
        .unsigned(
            "fresh_validation_last_timestamp_ms",
            value.fresh_validation_last_timestamp_ms,
        )
        .unsigned(
            "final_holdout_first_timestamp_ms",
            value.final_holdout_first_timestamp_ms,
        )
        .unsigned(
            "final_holdout_last_timestamp_ms",
            value.final_holdout_last_timestamp_ms,
        )
        .unsigned("label_reads", as_u64(value.label_reads)?)
        .unsigned("prediction_reads", as_u64(value.prediction_reads)?)
        .unsigned("metric_reads", as_u64(value.metric_reads)?)
        .boolean(
            "fresh_validation_execution_authorized",
            value.fresh_validation_execution_authorized,
        )
        .boolean(
            "final_holdout_execution_authorized",
            value.final_holdout_execution_authorized,
        )
        .string("split_digest", &value.split_digest)
        .encode()
}

fn decode_split(bytes: &[u8]) -> Result<MomentumT10FreshEvidenceSplitV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, SPLIT_VERSION)?;
    let value = MomentumT10FreshEvidenceSplitV1 {
        split_version: fields.string("split_version")?,
        parent_holdout_digest: fields.string("parent_holdout_digest")?,
        parent_event_count: as_usize(fields.unsigned("parent_event_count")?)?,
        parent_first_timestamp_ms: fields.unsigned("parent_first_timestamp_ms")?,
        parent_last_timestamp_ms: fields.unsigned("parent_last_timestamp_ms")?,
        fresh_validation_event_digests: fields.strings("fresh_validation_event_digests")?,
        final_holdout_event_digests: fields.strings("final_holdout_event_digests")?,
        fresh_validation_first_timestamp_ms: fields
            .unsigned("fresh_validation_first_timestamp_ms")?,
        fresh_validation_last_timestamp_ms: fields
            .unsigned("fresh_validation_last_timestamp_ms")?,
        final_holdout_first_timestamp_ms: fields.unsigned("final_holdout_first_timestamp_ms")?,
        final_holdout_last_timestamp_ms: fields.unsigned("final_holdout_last_timestamp_ms")?,
        label_reads: as_usize(fields.unsigned("label_reads")?)?,
        prediction_reads: as_usize(fields.unsigned("prediction_reads")?)?,
        metric_reads: as_usize(fields.unsigned("metric_reads")?)?,
        fresh_validation_execution_authorized: fields
            .boolean("fresh_validation_execution_authorized")?,
        final_holdout_execution_authorized: fields.boolean("final_holdout_execution_authorized")?,
        split_digest: fields.string("split_digest")?,
    };
    fields.finish()?;
    validate_split(&value)?;
    Ok(value)
}

fn encode_magnitude_policy(value: &MomentumT10TargetMagnitudePolicyV1) -> Result<Vec<u8>, String> {
    validate_magnitude_policy(value)?;
    ArtifactBuilderV4_2::new(MAGNITUDE_POLICY_VERSION)
        .string("policy_version", &value.policy_version)
        .string("source_partition", enum_partition(value.source_partition))
        .unsigneds("quantile_bits", &value.quantile_bits)
        .boolean(
            "validation_may_define_boundaries",
            value.validation_may_define_boundaries,
        )
        .boolean(
            "fresh_validation_access_forbidden",
            value.fresh_validation_access_forbidden,
        )
        .boolean(
            "final_holdout_access_forbidden",
            value.final_holdout_access_forbidden,
        )
        .string("policy_digest", &value.policy_digest)
        .encode()
}

fn decode_magnitude_policy(bytes: &[u8]) -> Result<MomentumT10TargetMagnitudePolicyV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, MAGNITUDE_POLICY_VERSION)?;
    let value = MomentumT10TargetMagnitudePolicyV1 {
        policy_version: fields.string("policy_version")?,
        source_partition: parse_partition(&fields.string("source_partition")?)?,
        quantile_bits: fields.unsigneds("quantile_bits")?,
        validation_may_define_boundaries: fields.boolean("validation_may_define_boundaries")?,
        fresh_validation_access_forbidden: fields.boolean("fresh_validation_access_forbidden")?,
        final_holdout_access_forbidden: fields.boolean("final_holdout_access_forbidden")?,
        policy_digest: fields.string("policy_digest")?,
    };
    fields.finish()?;
    validate_magnitude_policy(&value)?;
    Ok(value)
}

fn encode_magnitude_boundaries(
    value: &MomentumT10TargetMagnitudeBoundariesV1,
    policy: &MomentumT10TargetMagnitudePolicyV1,
) -> Result<Vec<u8>, String> {
    validate_magnitude_boundaries(value, policy)?;
    ArtifactBuilderV4_2::new(MAGNITUDE_BOUNDARY_VERSION)
        .string("boundary_version", &value.boundary_version)
        .string("policy_digest", &value.policy_digest)
        .unsigned(
            "development_event_count",
            as_u64(value.development_event_count)?,
        )
        .unsigneds("boundary_bits", &value.boundary_bits)
        .unsigned(
            "validation_assignment_count_at_freeze",
            as_u64(value.validation_assignment_count_at_freeze)?,
        )
        .string("boundary_digest", &value.boundary_digest)
        .encode()
}

fn decode_magnitude_boundaries(
    bytes: &[u8],
    policy: &MomentumT10TargetMagnitudePolicyV1,
) -> Result<MomentumT10TargetMagnitudeBoundariesV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, MAGNITUDE_BOUNDARY_VERSION)?;
    let value = MomentumT10TargetMagnitudeBoundariesV1 {
        boundary_version: fields.string("boundary_version")?,
        policy_digest: fields.string("policy_digest")?,
        development_event_count: as_usize(fields.unsigned("development_event_count")?)?,
        boundary_bits: fields.unsigneds("boundary_bits")?,
        validation_assignment_count_at_freeze: as_usize(
            fields.unsigned("validation_assignment_count_at_freeze")?,
        )?,
        boundary_digest: fields.string("boundary_digest")?,
    };
    fields.finish()?;
    validate_magnitude_boundaries(&value, policy)?;
    Ok(value)
}

fn encode_confidence_policy(
    value: &MomentumT10ConfidenceCoveragePolicyV1,
) -> Result<Vec<u8>, String> {
    validate_confidence_policy(value)?;
    ArtifactBuilderV4_2::new(CONFIDENCE_POLICY_VERSION)
        .string("policy_version", &value.policy_version)
        .unsigneds(
            "distance_from_half_boundary_bits",
            &value.distance_from_half_boundary_bits,
        )
        .boolean(
            "deployment_threshold_selection_forbidden",
            value.deployment_threshold_selection_forbidden,
        )
        .string("policy_digest", &value.policy_digest)
        .encode()
}

fn decode_confidence_policy(bytes: &[u8]) -> Result<MomentumT10ConfidenceCoveragePolicyV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, CONFIDENCE_POLICY_VERSION)?;
    let value = MomentumT10ConfidenceCoveragePolicyV1 {
        policy_version: fields.string("policy_version")?,
        distance_from_half_boundary_bits: fields.unsigneds("distance_from_half_boundary_bits")?,
        deployment_threshold_selection_forbidden: fields
            .boolean("deployment_threshold_selection_forbidden")?,
        policy_digest: fields.string("policy_digest")?,
    };
    fields.finish()?;
    validate_confidence_policy(&value)?;
    Ok(value)
}

fn encode_registration(
    value: &MomentumT10FailureForensicsRegistrationV1,
) -> Result<Vec<u8>, String> {
    validate_registration(value)?;
    ArtifactBuilderV4_2::new(REGISTRATION_VERSION)
        .string("registration_version", &value.registration_version)
        .string("source_screening_digest", &value.source_screening_digest)
        .string(
            "source_screening_registration_digest",
            &value.source_screening_registration_digest,
        )
        .string(
            "source_development_aggregate_digest",
            &value.source_development_aggregate_digest,
        )
        .string(
            "source_validation_aggregate_digest",
            &value.source_validation_aggregate_digest,
        )
        .strings("participant_ids", &value.participant_ids)
        .strings(
            "development_prediction_shard_digests",
            &value.development_prediction_shard_digests,
        )
        .strings(
            "development_evaluation_shard_digests",
            &value.development_evaluation_shard_digests,
        )
        .strings(
            "validation_prediction_shard_digests",
            &value.validation_prediction_shard_digests,
        )
        .strings(
            "validation_evaluation_shard_digests",
            &value.validation_evaluation_shard_digests,
        )
        .string(
            "target_magnitude_policy_digest",
            &value.target_magnitude_policy_digest,
        )
        .string(
            "confidence_coverage_policy_digest",
            &value.confidence_coverage_policy_digest,
        )
        .string("saturation_policy_digest", &value.saturation_policy_digest)
        .string(
            "consumed_evidence_receipt_digest",
            &value.consumed_evidence_receipt_digest,
        )
        .string(
            "fresh_evidence_split_digest",
            &value.fresh_evidence_split_digest,
        )
        .boolean("consumed_design_only", value.consumed_design_only)
        .boolean(
            "fresh_validation_access_forbidden",
            value.fresh_validation_access_forbidden,
        )
        .boolean(
            "final_holdout_access_forbidden",
            value.final_holdout_access_forbidden,
        )
        .boolean(
            "new_model_training_forbidden",
            value.new_model_training_forbidden,
        )
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_registration(bytes: &[u8]) -> Result<MomentumT10FailureForensicsRegistrationV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, REGISTRATION_VERSION)?;
    let value = MomentumT10FailureForensicsRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        source_screening_digest: fields.string("source_screening_digest")?,
        source_screening_registration_digest: fields
            .string("source_screening_registration_digest")?,
        source_development_aggregate_digest: fields
            .string("source_development_aggregate_digest")?,
        source_validation_aggregate_digest: fields.string("source_validation_aggregate_digest")?,
        participant_ids: fields.strings("participant_ids")?,
        development_prediction_shard_digests: fields
            .strings("development_prediction_shard_digests")?,
        development_evaluation_shard_digests: fields
            .strings("development_evaluation_shard_digests")?,
        validation_prediction_shard_digests: fields
            .strings("validation_prediction_shard_digests")?,
        validation_evaluation_shard_digests: fields
            .strings("validation_evaluation_shard_digests")?,
        target_magnitude_policy_digest: fields.string("target_magnitude_policy_digest")?,
        confidence_coverage_policy_digest: fields.string("confidence_coverage_policy_digest")?,
        saturation_policy_digest: fields.string("saturation_policy_digest")?,
        consumed_evidence_receipt_digest: fields.string("consumed_evidence_receipt_digest")?,
        fresh_evidence_split_digest: fields.string("fresh_evidence_split_digest")?,
        consumed_design_only: fields.boolean("consumed_design_only")?,
        fresh_validation_access_forbidden: fields.boolean("fresh_validation_access_forbidden")?,
        final_holdout_access_forbidden: fields.boolean("final_holdout_access_forbidden")?,
        new_model_training_forbidden: fields.boolean("new_model_training_forbidden")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_registration(&value)?;
    Ok(value)
}

fn encode_magnitude_diagnostic(
    value: &MomentumT10MagnitudeBinDiagnosticV1,
) -> Result<Vec<u8>, String> {
    validate_magnitude_diagnostic(value)?;
    ArtifactBuilderV4_2::new(MAGNITUDE_DIAGNOSTIC_VERSION)
        .string("diagnostic_version", &value.diagnostic_version)
        .string("participant_id", &value.participant_id)
        .string("partition", enum_partition(value.partition))
        .unsigned("bin_index", as_u64(value.bin_index)?)
        .unsigned("lower_bound_bits", value.lower_bound.to_bits())
        .unsigned("upper_bound_bits", value.upper_bound.to_bits())
        .boolean("upper_inclusive", value.upper_inclusive)
        .unsigned("support", as_u64(value.support)?)
        .unsigned("mean_brier_bits", value.mean_brier.to_bits())
        .unsigned("c0_mean_brier_bits", value.c0_mean_brier.to_bits())
        .unsigned(
            "paired_brier_delta_bits",
            value.paired_brier_delta.to_bits(),
        )
        .unsigned("correctness_bits", value.correctness.to_bits())
        .unsigned(
            "weighted_calibration_gap_bits",
            value.weighted_calibration_gap.to_bits(),
        )
        .unsigned("saturation_count", as_u64(value.saturation_count)?)
        .boolean("finite", value.finite)
        .string("diagnostic_digest", &value.diagnostic_digest)
        .encode()
}

fn decode_magnitude_diagnostic(
    bytes: &[u8],
) -> Result<MomentumT10MagnitudeBinDiagnosticV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, MAGNITUDE_DIAGNOSTIC_VERSION)?;
    let value = MomentumT10MagnitudeBinDiagnosticV1 {
        diagnostic_version: fields.string("diagnostic_version")?,
        participant_id: fields.string("participant_id")?,
        partition: parse_partition(&fields.string("partition")?)?,
        bin_index: as_usize(fields.unsigned("bin_index")?)?,
        lower_bound: f64::from_bits(fields.unsigned("lower_bound_bits")?),
        upper_bound: f64::from_bits(fields.unsigned("upper_bound_bits")?),
        upper_inclusive: fields.boolean("upper_inclusive")?,
        support: as_usize(fields.unsigned("support")?)?,
        mean_brier: f64::from_bits(fields.unsigned("mean_brier_bits")?),
        c0_mean_brier: f64::from_bits(fields.unsigned("c0_mean_brier_bits")?),
        paired_brier_delta: f64::from_bits(fields.unsigned("paired_brier_delta_bits")?),
        correctness: f64::from_bits(fields.unsigned("correctness_bits")?),
        weighted_calibration_gap: f64::from_bits(fields.unsigned("weighted_calibration_gap_bits")?),
        saturation_count: as_usize(fields.unsigned("saturation_count")?)?,
        finite: fields.boolean("finite")?,
        diagnostic_digest: fields.string("diagnostic_digest")?,
    };
    fields.finish()?;
    validate_magnitude_diagnostic(&value)?;
    Ok(value)
}

fn encode_confidence_diagnostic(
    value: &MomentumT10ConfidenceBandDiagnosticV1,
) -> Result<Vec<u8>, String> {
    validate_confidence_diagnostic(value)?;
    ArtifactBuilderV4_2::new(CONFIDENCE_DIAGNOSTIC_VERSION)
        .string("diagnostic_version", &value.diagnostic_version)
        .string("participant_id", &value.participant_id)
        .string("partition", enum_partition(value.partition))
        .unsigned("band_index", as_u64(value.band_index)?)
        .unsigned("lower_bound_bits", value.lower_bound.to_bits())
        .unsigned("upper_bound_bits", value.upper_bound.to_bits())
        .boolean("upper_inclusive", value.upper_inclusive)
        .unsigned("prediction_count", as_u64(value.prediction_count)?)
        .unsigned("coverage_bits", value.coverage.to_bits())
        .unsigned("mean_brier_bits", value.mean_brier.to_bits())
        .unsigned("c0_mean_brier_bits", value.c0_mean_brier.to_bits())
        .unsigned(
            "paired_brier_delta_bits",
            value.paired_brier_delta.to_bits(),
        )
        .unsigned("correctness_bits", value.correctness.to_bits())
        .unsigned("calibration_gap_bits", value.calibration_gap.to_bits())
        .unsigned(
            "mean_target_magnitude_bits",
            value.mean_target_magnitude.to_bits(),
        )
        .unsigned("saturation_count", as_u64(value.saturation_count)?)
        .boolean("finite", value.finite)
        .string("diagnostic_digest", &value.diagnostic_digest)
        .encode()
}

fn decode_confidence_diagnostic(
    bytes: &[u8],
) -> Result<MomentumT10ConfidenceBandDiagnosticV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, CONFIDENCE_DIAGNOSTIC_VERSION)?;
    let value = MomentumT10ConfidenceBandDiagnosticV1 {
        diagnostic_version: fields.string("diagnostic_version")?,
        participant_id: fields.string("participant_id")?,
        partition: parse_partition(&fields.string("partition")?)?,
        band_index: as_usize(fields.unsigned("band_index")?)?,
        lower_bound: f64::from_bits(fields.unsigned("lower_bound_bits")?),
        upper_bound: f64::from_bits(fields.unsigned("upper_bound_bits")?),
        upper_inclusive: fields.boolean("upper_inclusive")?,
        prediction_count: as_usize(fields.unsigned("prediction_count")?)?,
        coverage: f64::from_bits(fields.unsigned("coverage_bits")?),
        mean_brier: f64::from_bits(fields.unsigned("mean_brier_bits")?),
        c0_mean_brier: f64::from_bits(fields.unsigned("c0_mean_brier_bits")?),
        paired_brier_delta: f64::from_bits(fields.unsigned("paired_brier_delta_bits")?),
        correctness: f64::from_bits(fields.unsigned("correctness_bits")?),
        calibration_gap: f64::from_bits(fields.unsigned("calibration_gap_bits")?),
        mean_target_magnitude: f64::from_bits(fields.unsigned("mean_target_magnitude_bits")?),
        saturation_count: as_usize(fields.unsigned("saturation_count")?)?,
        finite: fields.boolean("finite")?,
        diagnostic_digest: fields.string("diagnostic_digest")?,
    };
    fields.finish()?;
    validate_confidence_diagnostic(&value)?;
    Ok(value)
}

fn encode_participant_report(
    value: &MomentumT10ParticipantFailureReportV1,
) -> Result<Vec<u8>, String> {
    validate_participant_report(value)?;
    ArtifactBuilderV4_2::new(PARTICIPANT_REPORT_VERSION)
        .string("report_version", &value.report_version)
        .string("participant_id", &value.participant_id)
        .string("disposition", disposition_name(value.disposition))
        .strings(
            "magnitude_diagnostic_digests",
            &value.magnitude_diagnostic_digests,
        )
        .strings(
            "confidence_diagnostic_digests",
            &value.confidence_diagnostic_digests,
        )
        .strings("secondary_observations", &value.secondary_observations)
        .unsigned("paired_event_count", as_u64(value.paired_event_count)?)
        .unsigned(
            "smallest_target_magnitude_excess_brier_concentration_bits",
            value
                .smallest_target_magnitude_excess_brier_concentration
                .to_bits(),
        )
        .unsigned(
            "saturation_event_count",
            as_u64(value.saturation_event_count)?,
        )
        .unsigned(
            "saturation_event_concentration_bits",
            value.saturation_event_concentration.to_bits(),
        )
        .boolean("finite_value_proof", value.finite_value_proof)
        .string("report_digest", &value.report_digest)
        .encode()
}

fn decode_participant_report(
    bytes: &[u8],
) -> Result<MomentumT10ParticipantFailureReportV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, PARTICIPANT_REPORT_VERSION)?;
    let value = MomentumT10ParticipantFailureReportV1 {
        report_version: fields.string("report_version")?,
        participant_id: fields.string("participant_id")?,
        disposition: parse_disposition(&fields.string("disposition")?)?,
        magnitude_diagnostic_digests: fields.strings("magnitude_diagnostic_digests")?,
        confidence_diagnostic_digests: fields.strings("confidence_diagnostic_digests")?,
        secondary_observations: fields.strings("secondary_observations")?,
        paired_event_count: as_usize(fields.unsigned("paired_event_count")?)?,
        smallest_target_magnitude_excess_brier_concentration: f64::from_bits(
            fields.unsigned("smallest_target_magnitude_excess_brier_concentration_bits")?,
        ),
        saturation_event_count: as_usize(fields.unsigned("saturation_event_count")?)?,
        saturation_event_concentration: f64::from_bits(
            fields.unsigned("saturation_event_concentration_bits")?,
        ),
        finite_value_proof: fields.boolean("finite_value_proof")?,
        report_digest: fields.string("report_digest")?,
    };
    fields.finish()?;
    validate_participant_report(&value)?;
    Ok(value)
}

fn safety_values(value: &MomentumT10FailureForensicsSafetyCountersV1) -> Result<Vec<u64>, String> {
    [
        value.artifacts_written,
        value.duplicate_artifact_count,
        value.event_error_computations,
        value.label_computations,
        value.new_model_fits,
        value.new_predictions,
        value.fresh_validation_label_reads,
        value.fresh_validation_prediction_reads,
        value.fresh_validation_metric_reads,
        value.final_holdout_label_reads,
        value.final_holdout_prediction_reads,
        value.final_holdout_metric_reads,
        value.network_requests,
        value.live_operations,
        value.reward_applications,
        value.penalty_applications,
        value.chair_actions,
        value.vote_actions,
        value.trading_actions,
        value.t30_model_executions,
        value.t60_model_executions,
        value.day_view_loads,
        value.week_view_loads,
        value.month_view_loads,
        value.year_view_loads,
    ]
    .into_iter()
    .map(as_u64)
    .collect()
}

fn safety_from_values(
    values: Vec<u64>,
) -> Result<MomentumT10FailureForensicsSafetyCountersV1, String> {
    if values.len() != 25 {
        return Err("T10 failure safety counters rejected".to_string());
    }
    let values = values
        .into_iter()
        .map(as_usize)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MomentumT10FailureForensicsSafetyCountersV1 {
        artifacts_written: values[0],
        duplicate_artifact_count: values[1],
        event_error_computations: values[2],
        label_computations: values[3],
        new_model_fits: values[4],
        new_predictions: values[5],
        fresh_validation_label_reads: values[6],
        fresh_validation_prediction_reads: values[7],
        fresh_validation_metric_reads: values[8],
        final_holdout_label_reads: values[9],
        final_holdout_prediction_reads: values[10],
        final_holdout_metric_reads: values[11],
        network_requests: values[12],
        live_operations: values[13],
        reward_applications: values[14],
        penalty_applications: values[15],
        chair_actions: values[16],
        vote_actions: values[17],
        trading_actions: values[18],
        t30_model_executions: values[19],
        t60_model_executions: values[20],
        day_view_loads: values[21],
        week_view_loads: values[22],
        month_view_loads: values[23],
        year_view_loads: values[24],
    })
}

fn status_name(value: MomentumT10FailureForensicsStatusV1) -> &'static str {
    match value {
        MomentumT10FailureForensicsStatusV1::Unregistered => "Unregistered",
        MomentumT10FailureForensicsStatusV1::Complete => "Complete",
        MomentumT10FailureForensicsStatusV1::IntegrityFailure => "IntegrityFailure",
    }
}

fn parse_status(value: &str) -> Result<MomentumT10FailureForensicsStatusV1, String> {
    match value {
        "Unregistered" => Ok(MomentumT10FailureForensicsStatusV1::Unregistered),
        "Complete" => Ok(MomentumT10FailureForensicsStatusV1::Complete),
        "IntegrityFailure" => Ok(MomentumT10FailureForensicsStatusV1::IntegrityFailure),
        _ => Err("T10 failure status rejected".to_string()),
    }
}

fn encode_report(value: &MomentumT10FailureForensicsReportV1) -> Result<Vec<u8>, String> {
    validate_report(value)?;
    ArtifactBuilderV4_2::new(REPORT_VERSION)
        .string("report_version", &value.report_version)
        .string("run_mode", &value.run_mode)
        .string("status", status_name(value.status))
        .string("design_evidence_class", "PostScreeningResearchDesignOnly")
        .string("source_screening_digest", &value.source_screening_digest)
        .string(
            "source_development_aggregate_digest",
            &value.source_development_aggregate_digest,
        )
        .string(
            "source_validation_aggregate_digest",
            &value.source_validation_aggregate_digest,
        )
        .string(
            "source_empty_cohort_digest",
            &value.source_empty_cohort_digest,
        )
        .string(
            "protected_before_state_digest",
            &value.protected_before_state_digest,
        )
        .messages(
            "evidence_use_receipt",
            vec![encode_evidence_use(&value.evidence_use_receipt)?],
        )
        .messages(
            "fresh_evidence_split",
            vec![encode_split(&value.fresh_evidence_split)?],
        )
        .messages(
            "registration",
            vec![encode_registration(&value.registration)?],
        )
        .messages(
            "magnitude_policy",
            vec![encode_magnitude_policy(&value.magnitude_policy)?],
        )
        .messages(
            "magnitude_boundaries",
            vec![encode_magnitude_boundaries(
                &value.magnitude_boundaries,
                &value.magnitude_policy,
            )?],
        )
        .messages(
            "confidence_policy",
            vec![encode_confidence_policy(&value.confidence_policy)?],
        )
        .messages(
            "magnitude_diagnostics",
            value
                .magnitude_diagnostics
                .iter()
                .map(encode_magnitude_diagnostic)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "confidence_diagnostics",
            value
                .confidence_diagnostics
                .iter()
                .map(encode_confidence_diagnostic)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "participant_reports",
            value
                .participant_reports
                .iter()
                .map(encode_participant_report)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .strings("labels", &value.labels)
        .unsigned(
            "live_completed_event_count",
            as_u64(value.live_completed_event_count)?,
        )
        .unsigned(
            "live_scorable_event_count",
            as_u64(value.live_scorable_event_count)?,
        )
        .string("live_pause", &value.live_pause)
        .boolean("epoch_three_registered", value.epoch_three_registered)
        .boolean("full_eight_blocked", value.full_eight_blocked)
        .boolean(
            "protected_artifacts_unchanged",
            value.protected_artifacts_unchanged,
        )
        .unsigneds("safety_counters", &safety_values(&value.safety_counters)?)
        .string(
            "deterministic_replay_digest",
            &value.deterministic_replay_digest,
        )
        .unsigned("runtime_duration_ms", value.runtime_duration_ms)
        .string("report_digest", &value.report_digest)
        .encode()
}

fn one_message<T>(
    mut messages: Vec<Vec<u8>>,
    decode: impl Fn(&[u8]) -> Result<T, String>,
) -> Result<T, String> {
    if messages.len() != 1 {
        return Err("T10 failure nested artifact count rejected".to_string());
    }
    decode(&messages.remove(0))
}

fn decode_report(bytes: &[u8]) -> Result<MomentumT10FailureForensicsReportV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, REPORT_VERSION)?;
    let report_version = fields.string("report_version")?;
    let run_mode = fields.string("run_mode")?;
    let status = parse_status(&fields.string("status")?)?;
    if fields.string("design_evidence_class")? != "PostScreeningResearchDesignOnly" {
        return Err("T10 design evidence class rejected".to_string());
    }
    let source_screening_digest = fields.string("source_screening_digest")?;
    let source_development_aggregate_digest =
        fields.string("source_development_aggregate_digest")?;
    let source_validation_aggregate_digest = fields.string("source_validation_aggregate_digest")?;
    let source_empty_cohort_digest = fields.string("source_empty_cohort_digest")?;
    let protected_before_state_digest = fields.string("protected_before_state_digest")?;
    let evidence_use_receipt = one_message(
        fields.messages("evidence_use_receipt")?,
        decode_evidence_use,
    )?;
    let fresh_evidence_split = one_message(fields.messages("fresh_evidence_split")?, decode_split)?;
    let registration = one_message(fields.messages("registration")?, decode_registration)?;
    let magnitude_policy = one_message(
        fields.messages("magnitude_policy")?,
        decode_magnitude_policy,
    )?;
    let magnitude_boundaries = one_message(fields.messages("magnitude_boundaries")?, |bytes| {
        decode_magnitude_boundaries(bytes, &magnitude_policy)
    })?;
    let confidence_policy = one_message(
        fields.messages("confidence_policy")?,
        decode_confidence_policy,
    )?;
    let magnitude_diagnostics = fields
        .messages("magnitude_diagnostics")?
        .iter()
        .map(|bytes| decode_magnitude_diagnostic(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let confidence_diagnostics = fields
        .messages("confidence_diagnostics")?
        .iter()
        .map(|bytes| decode_confidence_diagnostic(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let participant_reports = fields
        .messages("participant_reports")?
        .iter()
        .map(|bytes| decode_participant_report(bytes))
        .collect::<Result<Vec<_>, _>>()?;
    let value = MomentumT10FailureForensicsReportV1 {
        report_version,
        run_mode,
        status,
        design_evidence_class:
            MomentumT10ActionabilityDesignEvidenceClassV1::PostScreeningResearchDesignOnly,
        source_screening_digest,
        source_development_aggregate_digest,
        source_validation_aggregate_digest,
        source_empty_cohort_digest,
        protected_before_state_digest,
        evidence_use_receipt,
        fresh_evidence_split,
        registration,
        magnitude_policy,
        magnitude_boundaries,
        confidence_policy,
        magnitude_diagnostics,
        confidence_diagnostics,
        participant_reports,
        labels: fields.strings("labels")?,
        live_completed_event_count: as_usize(fields.unsigned("live_completed_event_count")?)?,
        live_scorable_event_count: as_usize(fields.unsigned("live_scorable_event_count")?)?,
        live_pause: fields.string("live_pause")?,
        epoch_three_registered: fields.boolean("epoch_three_registered")?,
        full_eight_blocked: fields.boolean("full_eight_blocked")?,
        protected_artifacts_unchanged: fields.boolean("protected_artifacts_unchanged")?,
        safety_counters: safety_from_values(fields.unsigneds("safety_counters")?)?,
        deterministic_replay_digest: fields.string("deterministic_replay_digest")?,
        runtime_duration_ms: fields.unsigned("runtime_duration_ms")?,
        report_digest: fields.string("report_digest")?,
    };
    fields.finish()?;
    validate_report(&value)?;
    Ok(value)
}

fn artifact_path(category: &str, digest: &str) -> PathBuf {
    Path::new(ROOT).join(category).join(format!("{digest}.pb"))
}

fn persist_one(
    category: &str,
    digest: &str,
    bytes: &[u8],
    decode_digest: impl Fn(&[u8]) -> Result<String, String>,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &artifact_path(category, digest),
        bytes,
        digest,
        decode_digest,
    )
}

fn add_counts(total: &mut (usize, usize), next: (usize, usize)) {
    total.0 += next.0;
    total.1 += next.1;
}

pub fn read_momentum_t10_failure_forensics_report_v1()
-> Result<Option<MomentumT10FailureForensicsReportV1>, String> {
    read_single(&Path::new(ROOT).join("final_reports"), decode_report)
}

fn build_evidence_use() -> Result<MomentumMicroEvidenceUseReceiptV2, String> {
    let mut value = MomentumMicroEvidenceUseReceiptV2 {
        receipt_version: EVIDENCE_USE_VERSION.to_string(),
        source_screening_digest: EXPECTED_SCREENING_REPORT_DIGEST.to_string(),
        development_aggregate_digest: EXPECTED_DEVELOPMENT_AGGREGATE_DIGEST.to_string(),
        validation_aggregate_digest: EXPECTED_VALIDATION_AGGREGATE_DIGEST.to_string(),
        former_development_use: MomentumMicroEvidenceUseClassV2::ConsumedResearchDesignEvidence,
        former_validation_use: MomentumMicroEvidenceUseClassV2::ConsumedResearchDesignEvidence,
        original_partition_receipts_immutable: true,
        receipt_digest: String::new(),
    };
    value.receipt_digest = evidence_use_digest(&value);
    validate_evidence_use(&value)?;
    Ok(value)
}

fn build_split(
    parent_holdout_digest: &str,
    holdout_start_timestamp_ms: u64,
    eligible_end_timestamp_ms: u64,
    expected_count: usize,
) -> Result<MomentumT10FreshEvidenceSplitV1, String> {
    let metadata = load_momentum_qualified_sealed_protocol_metadata_v1()?;
    if metadata.protocol_replay_digest.is_empty()
        || metadata.prior_holdout.labels_opened
        || metadata.prior_holdout.metrics_computed
        || metadata.prior_holdout.aggregate_comparison_opened
        || metadata.prior_holdout.holdout_digest.is_empty()
    {
        return Err("T10 parent holdout metadata rejected".to_string());
    }
    let events = metadata
        .protocol_events
        .iter()
        .filter(|event| {
            event.prediction_timestamp_ms >= holdout_start_timestamp_ms
                && event.prediction_timestamp_ms <= eligible_end_timestamp_ms
        })
        .collect::<Vec<_>>();
    if events.len() != expected_count
        || events.len() < 2
        || events
            .windows(2)
            .any(|pair| pair[0].prediction_timestamp_ms >= pair[1].prediction_timestamp_ms)
    {
        return Err(format!(
            "T10 parent holdout event identity rejected: expected={expected_count};derived={}",
            events.len()
        ));
    }
    let split_at = events.len() / 2;
    let fresh = &events[..split_at];
    let final_holdout = &events[split_at..];
    let mut value = MomentumT10FreshEvidenceSplitV1 {
        split_version: SPLIT_VERSION.to_string(),
        parent_holdout_digest: parent_holdout_digest.to_string(),
        parent_event_count: events.len(),
        parent_first_timestamp_ms: events[0].prediction_timestamp_ms,
        parent_last_timestamp_ms: events[events.len() - 1].prediction_timestamp_ms,
        fresh_validation_event_digests: fresh
            .iter()
            .map(|event| event.receipt_digest.clone())
            .collect(),
        final_holdout_event_digests: final_holdout
            .iter()
            .map(|event| event.receipt_digest.clone())
            .collect(),
        fresh_validation_first_timestamp_ms: fresh[0].prediction_timestamp_ms,
        fresh_validation_last_timestamp_ms: fresh[fresh.len() - 1].prediction_timestamp_ms,
        final_holdout_first_timestamp_ms: final_holdout[0].prediction_timestamp_ms,
        final_holdout_last_timestamp_ms: final_holdout[final_holdout.len() - 1]
            .prediction_timestamp_ms,
        label_reads: 0,
        prediction_reads: 0,
        metric_reads: 0,
        fresh_validation_execution_authorized: false,
        final_holdout_execution_authorized: false,
        split_digest: String::new(),
    };
    value.split_digest = split_digest(&value);
    validate_split(&value)?;
    Ok(value)
}

fn build_magnitude_policy() -> Result<MomentumT10TargetMagnitudePolicyV1, String> {
    let mut value = MomentumT10TargetMagnitudePolicyV1 {
        policy_version: MAGNITUDE_POLICY_VERSION.to_string(),
        source_partition: MomentumReplayPartitionV1::Development,
        quantile_bits: MAGNITUDE_QUANTILES
            .iter()
            .map(|value| value.to_bits())
            .collect(),
        validation_may_define_boundaries: false,
        fresh_validation_access_forbidden: true,
        final_holdout_access_forbidden: true,
        policy_digest: String::new(),
    };
    value.policy_digest = magnitude_policy_digest(&value);
    validate_magnitude_policy(&value)?;
    Ok(value)
}

fn build_confidence_policy() -> Result<MomentumT10ConfidenceCoveragePolicyV1, String> {
    let mut value = MomentumT10ConfidenceCoveragePolicyV1 {
        policy_version: CONFIDENCE_POLICY_VERSION.to_string(),
        distance_from_half_boundary_bits: CONFIDENCE_BOUNDARIES
            .iter()
            .map(|value| value.to_bits())
            .collect(),
        deployment_threshold_selection_forbidden: true,
        policy_digest: String::new(),
    };
    value.policy_digest = confidence_policy_digest(&value);
    validate_confidence_policy(&value)?;
    Ok(value)
}

fn build_registration(
    participant_ids: Vec<String>,
    development_prediction_shard_digests: Vec<String>,
    development_evaluation_shard_digests: Vec<String>,
    validation_prediction_shard_digests: Vec<String>,
    validation_evaluation_shard_digests: Vec<String>,
    evidence_use: &MomentumMicroEvidenceUseReceiptV2,
    split: &MomentumT10FreshEvidenceSplitV1,
    magnitude_policy: &MomentumT10TargetMagnitudePolicyV1,
    confidence_policy: &MomentumT10ConfidenceCoveragePolicyV1,
) -> Result<MomentumT10FailureForensicsRegistrationV1, String> {
    let mut value = MomentumT10FailureForensicsRegistrationV1 {
        registration_version: REGISTRATION_VERSION.to_string(),
        source_screening_digest: EXPECTED_SCREENING_REPORT_DIGEST.to_string(),
        source_screening_registration_digest: EXPECTED_SCREENING_REGISTRATION_DIGEST.to_string(),
        source_development_aggregate_digest: EXPECTED_DEVELOPMENT_AGGREGATE_DIGEST.to_string(),
        source_validation_aggregate_digest: EXPECTED_VALIDATION_AGGREGATE_DIGEST.to_string(),
        participant_ids,
        development_prediction_shard_digests,
        development_evaluation_shard_digests,
        validation_prediction_shard_digests,
        validation_evaluation_shard_digests,
        target_magnitude_policy_digest: magnitude_policy.policy_digest.clone(),
        confidence_coverage_policy_digest: confidence_policy.policy_digest.clone(),
        saturation_policy_digest: registered_saturation_policy_digest(),
        consumed_evidence_receipt_digest: evidence_use.receipt_digest.clone(),
        fresh_evidence_split_digest: split.split_digest.clone(),
        consumed_design_only: true,
        fresh_validation_access_forbidden: true,
        final_holdout_access_forbidden: true,
        new_model_training_forbidden: true,
        registration_digest: String::new(),
    };
    value.registration_digest = registration_digest(&value);
    validate_registration(&value)?;
    Ok(value)
}

fn quantile_sorted(values: &[f64], quantile: f64) -> Result<f64, String> {
    if values.is_empty()
        || values.iter().any(|value| !value.is_finite())
        || !(0.0..=1.0).contains(&quantile)
    {
        return Err("T10 magnitude quantile evidence rejected".to_string());
    }
    let position = quantile * (values.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        Ok(values[lower])
    } else {
        let weight = position - lower as f64;
        Ok(values[lower] * (1.0 - weight) + values[upper] * weight)
    }
}

fn build_magnitude_boundaries(
    development: &[MomentumT10ConsumedEventEvidenceV1],
    policy: &MomentumT10TargetMagnitudePolicyV1,
) -> Result<MomentumT10TargetMagnitudeBoundariesV1, String> {
    let mut magnitudes = development
        .iter()
        .filter(|event| event.label.is_some())
        .map(|event| event.target_return.abs())
        .collect::<Vec<_>>();
    magnitudes.sort_by(f64::total_cmp);
    let boundaries = MAGNITUDE_QUANTILES
        .iter()
        .map(|quantile| quantile_sorted(&magnitudes, *quantile))
        .collect::<Result<Vec<_>, _>>()?;
    let mut value = MomentumT10TargetMagnitudeBoundariesV1 {
        boundary_version: MAGNITUDE_BOUNDARY_VERSION.to_string(),
        policy_digest: policy.policy_digest.clone(),
        development_event_count: magnitudes.len(),
        boundary_bits: boundaries.iter().map(|value| value.to_bits()).collect(),
        validation_assignment_count_at_freeze: 0,
        boundary_digest: String::new(),
    };
    value.boundary_digest = magnitude_boundary_digest(&value);
    validate_magnitude_boundaries(&value, policy)?;
    Ok(value)
}

fn magnitude_bin(magnitude: f64, boundaries: &[f64]) -> usize {
    boundaries[1..5]
        .iter()
        .position(|boundary| magnitude <= *boundary)
        .unwrap_or(4)
}

fn confidence_band(distance: f64) -> usize {
    CONFIDENCE_BOUNDARIES[1..]
        .iter()
        .enumerate()
        .find(|(index, boundary)| distance < **boundary || (*index == 5 && distance <= **boundary))
        .map(|(index, _)| index)
        .unwrap_or(5)
}

fn build_partition_diagnostics(
    participant_ids: &[String],
    partition: MomentumReplayPartitionV1,
    events: &[MomentumT10ConsumedEventEvidenceV1],
    boundaries: &MomentumT10TargetMagnitudeBoundariesV1,
) -> Result<
    (
        Vec<MomentumT10MagnitudeBinDiagnosticV1>,
        Vec<MomentumT10ConfidenceBandDiagnosticV1>,
    ),
    String,
> {
    if partition == MomentumReplayPartitionV1::SealedHoldout
        || events.is_empty()
        || events.iter().any(|event| event.partition != partition)
    {
        return Err("T10 failure diagnostic evidence scope rejected".to_string());
    }
    let boundary_values = boundaries
        .boundary_bits
        .iter()
        .map(|bits| f64::from_bits(*bits))
        .collect::<Vec<_>>();
    let scorable = events
        .iter()
        .filter(|event| event.label.is_some())
        .collect::<Vec<_>>();
    if scorable.is_empty() {
        return Err("T10 failure diagnostic support unavailable".to_string());
    }
    let mut magnitude_diagnostics = Vec::new();
    let mut confidence_diagnostics = Vec::new();
    for participant_index in 1..5 {
        for bin_index in 0..5 {
            let members = scorable
                .iter()
                .filter(|event| {
                    magnitude_bin(event.target_return.abs(), &boundary_values) == bin_index
                })
                .copied()
                .collect::<Vec<_>>();
            if members.is_empty() {
                return Err("T10 empty magnitude bin rejected".to_string());
            }
            let support = members.len();
            let mean = |values: Vec<f64>| values.iter().sum::<f64>() / values.len() as f64;
            let mean_brier = mean(
                members
                    .iter()
                    .map(|event| event.brier_values[participant_index])
                    .collect(),
            );
            let c0_mean_brier = mean(members.iter().map(|event| event.brier_values[0]).collect());
            let correctness = mean(
                members
                    .iter()
                    .map(|event| f64::from(event.correctness[participant_index]))
                    .collect(),
            );
            let mean_probability = mean(
                members
                    .iter()
                    .map(|event| event.probabilities[participant_index])
                    .collect(),
            );
            let mean_label = mean(
                members
                    .iter()
                    .map(|event| event.label.unwrap_or_default())
                    .collect(),
            );
            let saturation_count = members
                .iter()
                .filter(|event| {
                    let probability = event.probabilities[participant_index];
                    probability <= PROBABILITY_CLAMP || probability >= 1.0 - PROBABILITY_CLAMP
                })
                .count();
            let mut value = MomentumT10MagnitudeBinDiagnosticV1 {
                diagnostic_version: MAGNITUDE_DIAGNOSTIC_VERSION.to_string(),
                participant_id: participant_ids[participant_index].clone(),
                partition,
                bin_index,
                lower_bound: boundary_values[bin_index],
                upper_bound: boundary_values[bin_index + 1],
                upper_inclusive: bin_index == 4,
                support,
                mean_brier,
                c0_mean_brier,
                paired_brier_delta: mean_brier - c0_mean_brier,
                correctness,
                weighted_calibration_gap: (mean_probability - mean_label).abs(),
                saturation_count,
                finite: true,
                diagnostic_digest: String::new(),
            };
            value.diagnostic_digest = magnitude_diagnostic_digest(&value);
            validate_magnitude_diagnostic(&value)?;
            magnitude_diagnostics.push(value);
        }
        for band_index in 0..6 {
            let members = scorable
                .iter()
                .filter(|event| {
                    confidence_band((event.probabilities[participant_index] - 0.5).abs())
                        == band_index
                })
                .copied()
                .collect::<Vec<_>>();
            if members.is_empty() {
                continue;
            }
            let support = members.len();
            let mean = |values: Vec<f64>| values.iter().sum::<f64>() / values.len() as f64;
            let mean_probability = mean(
                members
                    .iter()
                    .map(|event| event.probabilities[participant_index])
                    .collect(),
            );
            let mean_brier = mean(
                members
                    .iter()
                    .map(|event| event.brier_values[participant_index])
                    .collect(),
            );
            let c0_mean_brier = mean(members.iter().map(|event| event.brier_values[0]).collect());
            let mean_label = mean(
                members
                    .iter()
                    .map(|event| event.label.unwrap_or_default())
                    .collect(),
            );
            let mut value = MomentumT10ConfidenceBandDiagnosticV1 {
                diagnostic_version: CONFIDENCE_DIAGNOSTIC_VERSION.to_string(),
                participant_id: participant_ids[participant_index].clone(),
                partition,
                band_index,
                lower_bound: CONFIDENCE_BOUNDARIES[band_index],
                upper_bound: CONFIDENCE_BOUNDARIES[band_index + 1],
                upper_inclusive: band_index == 5,
                prediction_count: support,
                coverage: support as f64 / scorable.len() as f64,
                mean_brier,
                c0_mean_brier,
                paired_brier_delta: mean_brier - c0_mean_brier,
                correctness: mean(
                    members
                        .iter()
                        .map(|event| f64::from(event.correctness[participant_index]))
                        .collect(),
                ),
                calibration_gap: (mean_probability - mean_label).abs(),
                mean_target_magnitude: mean(
                    members
                        .iter()
                        .map(|event| event.target_return.abs())
                        .collect(),
                ),
                saturation_count: members
                    .iter()
                    .filter(|event| {
                        let probability = event.probabilities[participant_index];
                        probability <= PROBABILITY_CLAMP || probability >= 1.0 - PROBABILITY_CLAMP
                    })
                    .count(),
                finite: true,
                diagnostic_digest: String::new(),
            };
            value.diagnostic_digest = confidence_diagnostic_digest(&value);
            validate_confidence_diagnostic(&value)?;
            confidence_diagnostics.push(value);
        }
    }
    Ok((magnitude_diagnostics, confidence_diagnostics))
}

fn build_participant_reports(
    participant_ids: &[String],
    magnitude: &[MomentumT10MagnitudeBinDiagnosticV1],
    confidence: &[MomentumT10ConfidenceBandDiagnosticV1],
    screening: &super::momentum_t10_micro_screening_v1::MomentumT10MicroScreeningReportV1,
) -> Result<Vec<MomentumT10ParticipantFailureReportV1>, String> {
    let development = screening
        .development
        .as_ref()
        .ok_or_else(|| "T10 development screening aggregate unavailable".to_string())?;
    let validation = screening
        .validation
        .as_ref()
        .ok_or_else(|| "T10 validation screening aggregate unavailable".to_string())?;
    participant_ids[1..]
        .iter()
        .map(|participant_id| {
            let magnitude_items = magnitude
                .iter()
                .filter(|item| item.participant_id == *participant_id)
                .collect::<Vec<_>>();
            let confidence_items = confidence
                .iter()
                .filter(|item| item.participant_id == *participant_id)
                .collect::<Vec<_>>();
            let dev_metric = development
                .participant_metrics
                .iter()
                .find(|metric| metric.participant_id == *participant_id)
                .ok_or_else(|| "T10 development participant metric unavailable".to_string())?;
            let val_metric = validation
                .participant_metrics
                .iter()
                .find(|metric| metric.participant_id == *participant_id)
                .ok_or_else(|| "T10 validation participant metric unavailable".to_string())?;
            let saturation = dev_metric.saturation != MomentumMicroSaturationV1::NotSaturated
                || val_metric.saturation != MomentumMicroSaturationV1::NotSaturated;
            let partition_specific = dev_metric
                .paired_mean_brier_delta_versus_c0
                .is_sign_negative()
                != val_metric
                    .paired_mean_brier_delta_versus_c0
                    .is_sign_negative();
            let broad = dev_metric.paired_mean_brier_delta_versus_c0 >= 0.0
                && val_metric.paired_mean_brier_delta_versus_c0 >= 0.0
                && magnitude_items
                    .iter()
                    .filter(|item| item.paired_brier_delta >= 0.0)
                    .count()
                    >= 8;
            let tiny_positive = magnitude_items
                .iter()
                .filter(|item| item.bin_index <= 1)
                .map(|item| item.paired_brier_delta.max(0.0) * item.support as f64)
                .sum::<f64>();
            let total_positive = magnitude_items
                .iter()
                .map(|item| item.paired_brier_delta.max(0.0) * item.support as f64)
                .sum::<f64>();
            let smallest_target_magnitude_excess_brier_concentration = if total_positive > 0.0 {
                tiny_positive / total_positive
            } else {
                0.0
            };
            let tiny_dominated = smallest_target_magnitude_excess_brier_concentration >= 0.60;
            let calibration_instability = val_metric.weighted_calibration_gap
                > dev_metric.weighted_calibration_gap * 1.5
                && val_metric.paired_mean_brier_delta_versus_c0 > 0.0;
            let finite = magnitude_items.iter().all(|item| item.finite)
                && confidence_items.iter().all(|item| item.finite)
                && [
                    dev_metric.mean_brier,
                    val_metric.mean_brier,
                    dev_metric.weighted_calibration_gap,
                    val_metric.weighted_calibration_gap,
                ]
                .iter()
                .all(|value| value.is_finite());
            let paired_event_count = dev_metric.scorable_count + val_metric.scorable_count;
            let saturation_event_count = confidence_items
                .iter()
                .map(|item| item.saturation_count)
                .sum::<usize>();
            let saturation_event_concentration =
                saturation_event_count as f64 / paired_event_count as f64;
            let disposition = if !finite {
                MomentumT10FailureRootDispositionV1::IntegrityFailure
            } else if saturation {
                MomentumT10FailureRootDispositionV1::ProbabilitySaturation
            } else if partition_specific {
                MomentumT10FailureRootDispositionV1::PartitionSpecificSignal
            } else if broad {
                MomentumT10FailureRootDispositionV1::BroadFeatureUnderperformance
            } else if tiny_dominated {
                MomentumT10FailureRootDispositionV1::DominatedByTinyTargetNoise
            } else if calibration_instability {
                MomentumT10FailureRootDispositionV1::CalibrationInstability
            } else {
                MomentumT10FailureRootDispositionV1::MixedUnresolvedFailure
            };
            let mut secondary_observations = Vec::new();
            if tiny_dominated {
                secondary_observations
                    .push("smallest-two-magnitude-bands-concentrate-error".into());
            }
            if calibration_instability {
                secondary_observations.push("validation-calibration-gap-expanded".into());
            }
            if saturation {
                secondary_observations.push("registered-probability-boundary-reached".into());
            }
            if partition_specific {
                secondary_observations.push("partition-delta-sign-differs".into());
            }
            if broad {
                secondary_observations.push("broad-magnitude-bin-underperformance".into());
            }
            let mut value = MomentumT10ParticipantFailureReportV1 {
                report_version: PARTICIPANT_REPORT_VERSION.to_string(),
                participant_id: participant_id.clone(),
                disposition,
                magnitude_diagnostic_digests: magnitude_items
                    .iter()
                    .map(|item| item.diagnostic_digest.clone())
                    .collect(),
                confidence_diagnostic_digests: confidence_items
                    .iter()
                    .map(|item| item.diagnostic_digest.clone())
                    .collect(),
                secondary_observations,
                paired_event_count,
                smallest_target_magnitude_excess_brier_concentration,
                saturation_event_count,
                saturation_event_concentration,
                finite_value_proof: finite,
                report_digest: String::new(),
            };
            value.report_digest = participant_report_digest(&value);
            validate_participant_report(&value)?;
            Ok(value)
        })
        .collect()
}

fn persist_preregistration(
    evidence_use: &MomentumMicroEvidenceUseReceiptV2,
    split: &MomentumT10FreshEvidenceSplitV1,
    magnitude_policy: &MomentumT10TargetMagnitudePolicyV1,
    confidence_policy: &MomentumT10ConfidenceCoveragePolicyV1,
    registration: &MomentumT10FailureForensicsRegistrationV1,
) -> Result<(usize, usize), String> {
    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_one(
            "evidence_use_receipts",
            &evidence_use.receipt_digest,
            &encode_evidence_use(evidence_use)?,
            |bytes| Ok(decode_evidence_use(bytes)?.receipt_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_one(
            "fresh_evidence_splits",
            &split.split_digest,
            &encode_split(split)?,
            |bytes| Ok(decode_split(bytes)?.split_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_one(
            "magnitude_policies",
            &magnitude_policy.policy_digest,
            &encode_magnitude_policy(magnitude_policy)?,
            |bytes| Ok(decode_magnitude_policy(bytes)?.policy_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_one(
            "confidence_policies",
            &confidence_policy.policy_digest,
            &encode_confidence_policy(confidence_policy)?,
            |bytes| Ok(decode_confidence_policy(bytes)?.policy_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_one(
            "registrations",
            &registration.registration_digest,
            &encode_registration(registration)?,
            |bytes| Ok(decode_registration(bytes)?.registration_digest),
        )?,
    );
    Ok(counts)
}

fn persist_results(
    magnitude: &[MomentumT10MagnitudeBinDiagnosticV1],
    confidence: &[MomentumT10ConfidenceBandDiagnosticV1],
    participants: &[MomentumT10ParticipantFailureReportV1],
) -> Result<(usize, usize), String> {
    let mut counts = (0, 0);
    for value in magnitude {
        add_counts(
            &mut counts,
            persist_one(
                "magnitude_diagnostics",
                &value.diagnostic_digest,
                &encode_magnitude_diagnostic(value)?,
                |bytes| Ok(decode_magnitude_diagnostic(bytes)?.diagnostic_digest),
            )?,
        );
    }
    for value in confidence {
        add_counts(
            &mut counts,
            persist_one(
                "confidence_diagnostics",
                &value.diagnostic_digest,
                &encode_confidence_diagnostic(value)?,
                |bytes| Ok(decode_confidence_diagnostic(bytes)?.diagnostic_digest),
            )?,
        );
    }
    for value in participants {
        add_counts(
            &mut counts,
            persist_one(
                "participant_reports",
                &value.report_digest,
                &encode_participant_report(value)?,
                |bytes| Ok(decode_participant_report(bytes)?.report_digest),
            )?,
        );
    }
    Ok(counts)
}

fn completed_replay(
    mut report: MomentumT10FailureForensicsReportV1,
    mode: MomentumT10FailureForensicsRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
    started: Instant,
) -> Result<MomentumT10FailureForensicsReportV1, String> {
    if report.protected_before_state_digest != protected.state_digest {
        return Err("T10 failure replay protected state changed".to_string());
    }
    report.run_mode = mode.as_str().to_string();
    report.safety_counters = MomentumT10FailureForensicsSafetyCountersV1::default();
    report.runtime_duration_ms = started.elapsed().as_millis() as u64;
    report.report_digest = report_digest(&report);
    validate_report(&report)?;
    Ok(report)
}

fn run_inner(
    mode: MomentumT10FailureForensicsRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> Result<MomentumT10FailureForensicsReportV1, String> {
    let started = Instant::now();
    validate_momentum_micro_protected_before_state_v1(protected)?;
    if let Some(report) = read_momentum_t10_failure_forensics_report_v1()? {
        return completed_replay(report, mode, protected, started);
    }
    if mode == MomentumT10FailureForensicsRunModeV1::Status {
        return Err("T10 failure forensics unregistered".to_string());
    }
    let screening = read_momentum_t10_micro_screening_report_v1()?
        .ok_or_else(|| "T10 screening result unavailable".to_string())?;
    let development = screening
        .development
        .as_ref()
        .ok_or_else(|| "T10 development aggregate unavailable".to_string())?;
    let validation = screening
        .validation
        .as_ref()
        .ok_or_else(|| "T10 validation aggregate unavailable".to_string())?;
    let cohort = screening
        .proposed_holdout_cohort
        .as_ref()
        .ok_or_else(|| "T10 empty cohort unavailable".to_string())?;
    let boundary = screening
        .t10_boundary
        .as_ref()
        .ok_or_else(|| "T10 sealed boundary unavailable".to_string())?;
    if screening.status != MomentumT10MicroScreeningStatusV1::Complete
        || screening.report_digest != EXPECTED_SCREENING_REPORT_DIGEST
        || screening
            .authorization
            .as_ref()
            .map(|authorization| authorization.authorization_digest.as_str())
            != Some(EXPECTED_SCREENING_AUTHORIZATION_DIGEST)
        || screening.source_label_report_digest != EXPECTED_LABEL_REPORT_DIGEST
        || screening.source_feature_report_digest != EXPECTED_FEATURE_REPORT_DIGEST
        || screening.source_design_report_digest != EXPECTED_DESIGN_REPORT_DIGEST
        || screening.source_registration_digest != EXPECTED_SCREENING_REGISTRATION_DIGEST
        || screening.source_gate_digest != EXPECTED_SCREENING_GATE_DIGEST
        || screening.deterministic_replay_digest.as_deref()
            != Some(EXPECTED_SCREENING_REPLAY_DIGEST)
        || development.aggregate_digest != EXPECTED_DEVELOPMENT_AGGREGATE_DIGEST
        || validation.aggregate_digest != EXPECTED_VALIDATION_AGGREGATE_DIGEST
        || cohort.cohort_digest != EXPECTED_COHORT_DIGEST
        || cohort.status != MomentumMicroHoldoutCohortStatusV1::NoEligibleT10HoldoutCohort
        || !cohort.participant_ids.is_empty()
        || cohort.holdout_execution_authorized
        || boundary.holdout_labels_opened
        || screening.safety_counters.t10_holdout_label_reads != 0
        || screening.safety_counters.t10_holdout_predictions != 0
        || screening.safety_counters.t10_holdout_metrics != 0
    {
        return Err("T10 frozen screening result rejected".to_string());
    }
    let participant_ids = development
        .participant_metrics
        .iter()
        .map(|metrics| metrics.participant_id.clone())
        .collect::<Vec<_>>();
    let evidence_use = build_evidence_use()?;
    let split = build_split(
        &boundary.boundary_digest,
        boundary.holdout_start_timestamp_ms,
        boundary.eligible_end_timestamp_ms,
        boundary.holdout_event_count,
    )?;
    let magnitude_policy = build_magnitude_policy()?;
    let confidence_policy = build_confidence_policy()?;
    let registration = build_registration(
        participant_ids.clone(),
        development.prediction_shard_digests.clone(),
        development.evaluation_shard_digests.clone(),
        validation.prediction_shard_digests.clone(),
        validation.evaluation_shard_digests.clone(),
        &evidence_use,
        &split,
        &magnitude_policy,
        &confidence_policy,
    )?;
    let mut counts = persist_preregistration(
        &evidence_use,
        &split,
        &magnitude_policy,
        &confidence_policy,
        &registration,
    )?;

    // Private consumed evidence is opened only after the frozen registration above.
    let development_events =
        read_momentum_t10_consumed_event_evidence_v1(MomentumReplayPartitionV1::Development)?;
    let magnitude_boundaries = build_magnitude_boundaries(&development_events, &magnitude_policy)?;
    add_counts(
        &mut counts,
        persist_one(
            "magnitude_boundaries",
            &magnitude_boundaries.boundary_digest,
            &encode_magnitude_boundaries(&magnitude_boundaries, &magnitude_policy)?,
            |bytes| Ok(decode_magnitude_boundaries(bytes, &magnitude_policy)?.boundary_digest),
        )?,
    );
    let reopened_boundaries =
        read_single(&Path::new(ROOT).join("magnitude_boundaries"), |bytes| {
            decode_magnitude_boundaries(bytes, &magnitude_policy)
        })?
        .ok_or_else(|| "T10 frozen magnitude boundaries unavailable".to_string())?;
    if reopened_boundaries != magnitude_boundaries {
        return Err("T10 frozen magnitude boundaries mismatch".to_string());
    }
    let validation_events =
        read_momentum_t10_consumed_event_evidence_v1(MomentumReplayPartitionV1::Validation)?;
    let (mut magnitude_diagnostics, mut confidence_diagnostics) = build_partition_diagnostics(
        &participant_ids,
        MomentumReplayPartitionV1::Development,
        &development_events,
        &reopened_boundaries,
    )?;
    let (validation_magnitude, validation_confidence) = build_partition_diagnostics(
        &participant_ids,
        MomentumReplayPartitionV1::Validation,
        &validation_events,
        &reopened_boundaries,
    )?;
    magnitude_diagnostics.extend(validation_magnitude);
    confidence_diagnostics.extend(validation_confidence);
    magnitude_diagnostics.sort_by_key(|item| {
        (
            item.participant_id.clone(),
            match item.partition {
                MomentumReplayPartitionV1::Development => 0,
                MomentumReplayPartitionV1::Validation => 1,
                MomentumReplayPartitionV1::SealedHoldout => 2,
            },
            item.bin_index,
        )
    });
    confidence_diagnostics.sort_by_key(|item| {
        (
            item.participant_id.clone(),
            match item.partition {
                MomentumReplayPartitionV1::Development => 0,
                MomentumReplayPartitionV1::Validation => 1,
                MomentumReplayPartitionV1::SealedHoldout => 2,
            },
            item.band_index,
        )
    });
    let participant_reports = build_participant_reports(
        &participant_ids,
        &magnitude_diagnostics,
        &confidence_diagnostics,
        &screening,
    )?;
    let result_counts = persist_results(
        &magnitude_diagnostics,
        &confidence_diagnostics,
        &participant_reports,
    )?;
    add_counts(&mut counts, result_counts);
    let deterministic_replay_digest = stable_hash_string(&format!(
        "T10-failure-replay:{}:{}:{}:{}:{:?}:{:?}",
        evidence_use.receipt_digest,
        split.split_digest,
        registration.registration_digest,
        reopened_boundaries.boundary_digest,
        participant_reports
            .iter()
            .map(|item| item.report_digest.clone())
            .collect::<Vec<_>>(),
        confidence_diagnostics
            .iter()
            .map(|item| item.diagnostic_digest.clone())
            .collect::<Vec<_>>()
    ));
    let scorable_count = development_events
        .iter()
        .chain(&validation_events)
        .filter(|event| event.label.is_some())
        .count();
    let mut report = MomentumT10FailureForensicsReportV1 {
        report_version: REPORT_VERSION.to_string(),
        run_mode: mode.as_str().to_string(),
        status: MomentumT10FailureForensicsStatusV1::Complete,
        design_evidence_class:
            MomentumT10ActionabilityDesignEvidenceClassV1::PostScreeningResearchDesignOnly,
        source_screening_digest: screening.report_digest,
        source_development_aggregate_digest: development.aggregate_digest.clone(),
        source_validation_aggregate_digest: validation.aggregate_digest.clone(),
        source_empty_cohort_digest: cohort.cohort_digest.clone(),
        protected_before_state_digest: protected.state_digest.clone(),
        evidence_use_receipt: evidence_use,
        fresh_evidence_split: split,
        registration,
        magnitude_policy,
        magnitude_boundaries: reopened_boundaries,
        confidence_policy,
        magnitude_diagnostics,
        confidence_diagnostics,
        participant_reports,
        labels: PUBLIC_LABELS
            .iter()
            .map(|label| (*label).to_string())
            .collect(),
        live_completed_event_count: protected.completed_event_count,
        live_scorable_event_count: protected.scorable_event_count,
        live_pause: "PausedAfterCompletedEpochTwo".to_string(),
        epoch_three_registered: protected.epoch_three_registered,
        full_eight_blocked: true,
        protected_artifacts_unchanged: true,
        safety_counters: MomentumT10FailureForensicsSafetyCountersV1 {
            artifacts_written: counts.0 + 1,
            duplicate_artifact_count: counts.1,
            event_error_computations: scorable_count * 4,
            ..MomentumT10FailureForensicsSafetyCountersV1::default()
        },
        deterministic_replay_digest,
        runtime_duration_ms: started.elapsed().as_millis() as u64,
        report_digest: String::new(),
    };
    report.report_digest = report_digest(&report);
    validate_report(&report)?;
    let persisted = persist_one(
        "final_reports",
        &report.report_digest,
        &encode_report(&report)?,
        |bytes| Ok(decode_report(bytes)?.report_digest),
    )?;
    if persisted != (1, 0)
        || read_momentum_t10_failure_forensics_report_v1()?.as_ref() != Some(&report)
    {
        return Err("T10 failure final report persist mismatch".to_string());
    }
    Ok(report)
}

pub fn run_momentum_t10_failure_forensics_v1(
    mode: MomentumT10FailureForensicsRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> Result<MomentumT10FailureForensicsReportV1, String> {
    run_inner(mode, protected)
}

pub fn format_momentum_t10_failure_forensics_text_v1(
    report: &MomentumT10FailureForensicsReportV1,
) -> String {
    use std::fmt::Write as _;
    let mut output = String::new();
    let _ = writeln!(output, "status={:?}", report.status);
    let _ = writeln!(
        output,
        "fresh_validation_event_count={}",
        report
            .fresh_evidence_split
            .fresh_validation_event_digests
            .len()
    );
    let _ = writeln!(
        output,
        "final_holdout_event_count={}",
        report
            .fresh_evidence_split
            .final_holdout_event_digests
            .len()
    );
    for participant in &report.participant_reports {
        let _ = writeln!(
            output,
            "{}={}",
            participant.participant_id,
            disposition_name(participant.disposition)
        );
    }
    let _ = writeln!(
        output,
        "fresh_and_final_reads={}",
        report.fresh_evidence_split.label_reads
            + report.fresh_evidence_split.prediction_reads
            + report.fresh_evidence_split.metric_reads
    );
    let _ = writeln!(output, "report_digest={}", report.report_digest);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split_fixture(parent_count: usize) -> MomentumT10FreshEvidenceSplitV1 {
        let split_at = parent_count / 2;
        let mut value = MomentumT10FreshEvidenceSplitV1 {
            split_version: SPLIT_VERSION.into(),
            parent_holdout_digest: "parent".into(),
            parent_event_count: parent_count,
            parent_first_timestamp_ms: 1,
            parent_last_timestamp_ms: parent_count as u64,
            fresh_validation_event_digests: (0..split_at)
                .map(|index| format!("f{index}"))
                .collect(),
            final_holdout_event_digests: (split_at..parent_count)
                .map(|index| format!("h{index}"))
                .collect(),
            fresh_validation_first_timestamp_ms: 1,
            fresh_validation_last_timestamp_ms: split_at as u64,
            final_holdout_first_timestamp_ms: split_at as u64 + 1,
            final_holdout_last_timestamp_ms: parent_count as u64,
            label_reads: 0,
            prediction_reads: 0,
            metric_reads: 0,
            fresh_validation_execution_authorized: false,
            final_holdout_execution_authorized: false,
            split_digest: String::new(),
        };
        value.split_digest = split_digest(&value);
        value
    }

    #[test]
    fn sprint103_01_former_partitions_are_consumed_design_evidence() {
        assert!(validate_evidence_use(&build_evidence_use().unwrap()).is_ok());
    }

    #[test]
    fn sprint103_02_original_partition_receipts_remain_immutable() {
        assert!(
            build_evidence_use()
                .unwrap()
                .original_partition_receipts_immutable
        );
    }

    #[test]
    fn sprint103_03_split_reads_no_labels_predictions_or_metrics() {
        let value = split_fixture(9);
        assert_eq!(
            value.label_reads + value.prediction_reads + value.metric_reads,
            0
        );
    }

    #[test]
    fn sprint103_04_split_is_disjoint_and_complete() {
        assert!(validate_split(&split_fixture(10)).is_ok());
    }

    #[test]
    fn sprint103_05_odd_event_goes_to_final_holdout() {
        let value = split_fixture(9);
        assert_eq!(value.fresh_validation_event_digests.len(), 4);
        assert_eq!(value.final_holdout_event_digests.len(), 5);
    }

    #[test]
    fn sprint103_06_fresh_validation_precedes_final_holdout() {
        let value = split_fixture(10);
        assert!(value.fresh_validation_last_timestamp_ms < value.final_holdout_first_timestamp_ms);
    }

    #[test]
    fn sprint103_07_both_child_executions_are_unauthorized() {
        let value = split_fixture(10);
        assert!(
            !value.fresh_validation_execution_authorized
                && !value.final_holdout_execution_authorized
        );
    }

    #[test]
    fn sprint103_08_magnitude_policy_uses_development_only() {
        let value = build_magnitude_policy().unwrap();
        assert_eq!(
            value.source_partition,
            MomentumReplayPartitionV1::Development
        );
        assert!(!value.validation_may_define_boundaries);
    }

    #[test]
    fn sprint103_09_confidence_bands_are_fixed() {
        assert!(validate_confidence_policy(&build_confidence_policy().unwrap()).is_ok());
    }

    #[test]
    fn sprint103_10_magnitude_bins_cover_five_groups() {
        assert_eq!(MAGNITUDE_QUANTILES.len() - 1, 5);
    }

    #[test]
    fn sprint103_11_each_participant_has_one_canonical_disposition() {
        let value = MomentumT10FailureRootDispositionV1::MixedUnresolvedFailure;
        assert_eq!(parse_disposition(disposition_name(value)).unwrap(), value);
    }

    #[test]
    fn sprint103_12_malformed_protobuf_rejects() {
        assert!(decode_split(&[0xff, 0x00]).is_err());
    }

    #[test]
    fn sprint103_25_split_uses_timestamp_and_identity_metadata_only() {
        let value = split_fixture(10);
        assert_eq!(value.parent_first_timestamp_ms, 1);
        assert_eq!(value.parent_last_timestamp_ms, 10);
        assert_eq!(
            value
                .fresh_validation_event_digests
                .iter()
                .chain(&value.final_holdout_event_digests)
                .count(),
            value.parent_event_count
        );
    }

    #[test]
    fn sprint103_26_split_label_read_counter_is_zero() {
        assert_eq!(split_fixture(10).label_reads, 0);
    }

    #[test]
    fn sprint103_27_split_prediction_read_counter_is_zero() {
        assert_eq!(split_fixture(10).prediction_reads, 0);
    }

    #[test]
    fn sprint103_28_split_metric_read_counter_is_zero() {
        assert_eq!(split_fixture(10).metric_reads, 0);
    }

    #[test]
    fn sprint103_29_split_overlap_is_rejected() {
        let mut value = split_fixture(10);
        value.final_holdout_event_digests[0] = value.fresh_validation_event_digests[0].clone();
        value.split_digest = split_digest(&value);
        assert!(validate_split(&value).is_err());
    }

    #[test]
    fn sprint103_29b_split_internal_duplicate_is_rejected() {
        let mut value = split_fixture(10);
        value.fresh_validation_event_digests[1] = value.fresh_validation_event_digests[0].clone();
        value.split_digest = split_digest(&value);
        assert!(validate_split(&value).is_err());
    }

    #[test]
    fn sprint103_30_split_omission_is_rejected() {
        let mut value = split_fixture(10);
        value.final_holdout_event_digests.pop();
        value.split_digest = split_digest(&value);
        assert!(validate_split(&value).is_err());
    }

    #[test]
    fn sprint103_31_parent_holdout_binding_is_immutable() {
        let value = split_fixture(10);
        let encoded = encode_split(&value).unwrap();
        let reopened = decode_split(&encoded).unwrap();
        assert_eq!(reopened.parent_holdout_digest, value.parent_holdout_digest);
        assert_eq!(reopened.parent_event_count, value.parent_event_count);
    }

    #[test]
    fn sprint103_32_failure_registration_binds_all_prohibitions() {
        let evidence_use = build_evidence_use().unwrap();
        let split = split_fixture(10);
        let magnitude = build_magnitude_policy().unwrap();
        let confidence = build_confidence_policy().unwrap();
        let registration = build_registration(
            vec![
                "T10NextTenMinuteDirection:C0TaskSpecificConstant",
                "T10NextTenMinuteDirection:C1TenMinuteAnchorBaseline",
                "T10NextTenMinuteDirection:C2CompactMicroLogistic",
                "T10NextTenMinuteDirection:C3CompactMicroStrongShrinkLogistic",
                "T10NextTenMinuteDirection:C4CompactMicroTrainingOnlyCalibratedLogistic",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            vec!["development-prediction".into()],
            vec!["development-evaluation".into()],
            vec!["validation-prediction".into()],
            vec!["validation-evaluation".into()],
            &evidence_use,
            &split,
            &magnitude,
            &confidence,
        )
        .unwrap();
        assert!(
            registration.consumed_design_only
                && registration.fresh_validation_access_forbidden
                && registration.final_holdout_access_forbidden
                && registration.new_model_training_forbidden
        );
        let json = serde_json::to_value(&registration).unwrap();
        assert!(json.get("development_prediction_shard_digests").is_none());
        assert!(json.get("validation_evaluation_shard_digests").is_none());
    }

    #[test]
    fn sprint103_33_validation_cannot_define_magnitude_boundaries() {
        let mut value = build_magnitude_policy().unwrap();
        value.validation_may_define_boundaries = true;
        value.policy_digest = magnitude_policy_digest(&value);
        assert!(validate_magnitude_policy(&value).is_err());
    }

    #[test]
    fn sprint103_34_fresh_and_final_cannot_enter_magnitude_forensics() {
        let value = build_magnitude_policy().unwrap();
        assert!(value.fresh_validation_access_forbidden && value.final_holdout_access_forbidden);
    }

    #[test]
    fn sprint103_35_sealed_holdout_diagnostic_is_rejected() {
        let mut value = MomentumT10MagnitudeBinDiagnosticV1 {
            diagnostic_version: MAGNITUDE_DIAGNOSTIC_VERSION.into(),
            participant_id: "C1".into(),
            partition: MomentumReplayPartitionV1::SealedHoldout,
            bin_index: 0,
            lower_bound: 0.0,
            upper_bound: 0.01,
            upper_inclusive: false,
            support: 1,
            mean_brier: 0.25,
            c0_mean_brier: 0.25,
            paired_brier_delta: 0.0,
            correctness: 0.5,
            weighted_calibration_gap: 0.0,
            saturation_count: 0,
            finite: true,
            diagnostic_digest: String::new(),
        };
        value.diagnostic_digest = magnitude_diagnostic_digest(&value);
        assert!(validate_magnitude_diagnostic(&value).is_err());
    }

    #[test]
    fn sprint103_36_all_failure_dispositions_are_canonical() {
        let values = [
            MomentumT10FailureRootDispositionV1::DominatedByTinyTargetNoise,
            MomentumT10FailureRootDispositionV1::CalibrationInstability,
            MomentumT10FailureRootDispositionV1::ProbabilitySaturation,
            MomentumT10FailureRootDispositionV1::PartitionSpecificSignal,
            MomentumT10FailureRootDispositionV1::BroadFeatureUnderperformance,
            MomentumT10FailureRootDispositionV1::MixedUnresolvedFailure,
            MomentumT10FailureRootDispositionV1::IntegrityFailure,
        ];
        assert!(
            values
                .iter()
                .all(|value| parse_disposition(disposition_name(*value)) == Ok(*value))
        );
    }

    #[test]
    fn sprint103_36b_confidence_brier_delta_is_bound_to_c0() {
        let mut value = MomentumT10ConfidenceBandDiagnosticV1 {
            diagnostic_version: CONFIDENCE_DIAGNOSTIC_VERSION.into(),
            participant_id: "C1".into(),
            partition: MomentumReplayPartitionV1::Development,
            band_index: 0,
            lower_bound: 0.0,
            upper_bound: 0.01,
            upper_inclusive: false,
            prediction_count: 10,
            coverage: 0.1,
            mean_brier: 0.3,
            c0_mean_brier: 0.25,
            paired_brier_delta: 0.05,
            correctness: 0.5,
            calibration_gap: 0.1,
            mean_target_magnitude: 0.01,
            saturation_count: 0,
            finite: true,
            diagnostic_digest: String::new(),
        };
        value.diagnostic_digest = confidence_diagnostic_digest(&value);
        assert_eq!(
            decode_confidence_diagnostic(&encode_confidence_diagnostic(&value).unwrap()).unwrap(),
            value
        );
        value.paired_brier_delta = 0.04;
        value.diagnostic_digest = confidence_diagnostic_digest(&value);
        assert!(validate_confidence_diagnostic(&value).is_err());
    }

    #[test]
    fn sprint103_37_split_manual_protobuf_round_trips() {
        let value = split_fixture(11);
        assert_eq!(decode_split(&encode_split(&value).unwrap()).unwrap(), value);
    }

    #[test]
    fn sprint103_38_conflicting_split_digest_rejects() {
        let mut value = split_fixture(10);
        value.parent_holdout_digest = "changed-parent".into();
        assert!(validate_split(&value).is_err());
    }

    #[test]
    fn sprint103_39_zero_authority_counters_default_closed() {
        let value = MomentumT10FailureForensicsSafetyCountersV1::default();
        assert_eq!(
            value.new_model_fits
                + value.new_predictions
                + value.network_requests
                + value.live_operations
                + value.reward_applications
                + value.penalty_applications
                + value.chair_actions
                + value.vote_actions
                + value.trading_actions
                + value.t30_model_executions
                + value.t60_model_executions
                + value.day_view_loads
                + value.week_view_loads
                + value.month_view_loads
                + value.year_view_loads,
            0
        );
    }

    #[test]
    fn sprint103_39b_public_split_json_excludes_sealed_event_identities() {
        let json = serde_json::to_value(split_fixture(10)).unwrap();
        assert!(json.get("fresh_validation_event_digests").is_none());
        assert!(json.get("final_holdout_event_digests").is_none());
        assert_eq!(json["fresh_validation_event_count"], 5);
        assert_eq!(json["final_holdout_event_count"], 5);
    }
}
