//! Post-result, development/validation-only target diagnostics for micro challengers.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    time::Instant,
};

use serde::{Deserialize, Serialize};

use crate::stable_hash_string;

use super::{
    momentum_future_prediction_v4::{
        ArtifactBuilderV4_2, ArtifactReaderV4_2, as_u64, as_usize, persist_artifact, read_single,
    },
    momentum_multitimeframe_history_v1::{
        MomentumHistoricalTimeframeV1, MomentumQualifiedReplayCandleEvidenceV1,
        load_momentum_qualified_six_evidence_v1,
    },
    momentum_qualified_six_diagnostics_v1::{
        MomentumQualifiedDiagnosticEvidenceClassV1, MomentumQualifiedDiagnosticStatusV1,
        read_momentum_qualified_six_diagnostic_report_snapshot_v1,
    },
    momentum_qualified_six_replay_v1::{
        MomentumQualifiedDiagnosticEventEvidenceV1, MomentumQualifiedLabelStatusV1,
        MomentumReplayPartitionV1, load_momentum_qualified_diagnostic_source_header_v1,
        load_momentum_qualified_diagnostic_source_v1,
    },
};

pub(super) const ROOT: &str = "state/historical_replay/momentum_micro_label_forensics/v1";
const REGISTRATION_VERSION: &str = "momentum-micro-label-forensics-registration-v1";
const EVENT_PLAN_VERSION: &str = "momentum-micro-horizon-event-plan-v1";
const DISTRIBUTION_VERSION: &str = "momentum-micro-label-distribution-v1";
const TEMPORAL_VERSION: &str = "momentum-micro-label-temporal-stability-v1";
const OVERLAP_VERSION: &str = "momentum-micro-target-overlap-v1";
const SERIAL_VERSION: &str = "momentum-micro-label-serial-dependence-v1";
const DISPOSITION_VERSION: &str = "momentum-micro-horizon-disposition-v1";
const JOURNAL_VERSION: &str = "momentum-micro-label-forensics-journal-v1";
const REPORT_VERSION: &str = "momentum-micro-label-forensics-public-report-v1";
const TEN_MINUTE_MS: u64 = 10 * 60 * 1_000;
const DAY_MS: u64 = 24 * 60 * 60 * 1_000;
const WEEK_MS: u64 = 7 * DAY_MS;
const ROLLING_DAY_EVENTS: usize = 144;
const ROLLING_WEEK_EVENTS: usize = 1_008;
const SERIAL_LAGS: [usize; 5] = [1, 2, 3, 6, 12];
const PUBLIC_LABELS: [&str; 6] = [
    "HistoricalResearchOnly",
    "PostResultResearchDesignOnly",
    "MicroChallengerNotExecuted",
    "HoldoutClosed",
    "NotLiveAuthority",
    "NotTradingAuthority",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMicroChallengerDesignEvidenceClassV1 {
    PostResultResearchDesignOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MomentumMicroPredictionHorizonV1 {
    NextTenMinutes,
    NextThirtyMinutes,
    NextSixtyMinutes,
}

impl MomentumMicroPredictionHorizonV1 {
    pub(super) const ORDERED: [Self; 3] = [
        Self::NextTenMinutes,
        Self::NextThirtyMinutes,
        Self::NextSixtyMinutes,
    ];

    pub(super) fn horizon_candles(self) -> usize {
        match self {
            Self::NextTenMinutes => 1,
            Self::NextThirtyMinutes => 3,
            Self::NextSixtyMinutes => 6,
        }
    }

    pub(super) fn cadence_ms(self) -> u64 {
        match self {
            Self::NextTenMinutes => TEN_MINUTE_MS,
            Self::NextThirtyMinutes => 3 * TEN_MINUTE_MS,
            Self::NextSixtyMinutes => 6 * TEN_MINUTE_MS,
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        Self::ORDERED
            .into_iter()
            .find(|candidate| format!("{candidate:?}") == value)
            .ok_or_else(|| "micro label horizon rejected".to_string())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMicroLabelStatusV1 {
    Up,
    Down,
    Neutral,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMicroHorizonDiagnosticDispositionV1 {
    StableEnoughForFutureScreening,
    ClassBalanceShifted,
    TargetMagnitudeTooSmall,
    ExcessiveTemporalInstability,
    InsufficientScorableSupport,
    IntegrityFailure,
}

fn parse_disposition(value: &str) -> Result<MomentumMicroHorizonDiagnosticDispositionV1, String> {
    use MomentumMicroHorizonDiagnosticDispositionV1 as D;
    match value {
        "StableEnoughForFutureScreening" => Ok(D::StableEnoughForFutureScreening),
        "ClassBalanceShifted" => Ok(D::ClassBalanceShifted),
        "TargetMagnitudeTooSmall" => Ok(D::TargetMagnitudeTooSmall),
        "ExcessiveTemporalInstability" => Ok(D::ExcessiveTemporalInstability),
        "InsufficientScorableSupport" => Ok(D::InsufficientScorableSupport),
        "IntegrityFailure" => Ok(D::IntegrityFailure),
        _ => Err("micro horizon disposition rejected".to_string()),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMicroLabelForensicsStatusV1 {
    Unregistered,
    Registered,
    Complete,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumMicroLabelForensicsRunModeV1 {
    Status,
    DryRun,
    ExecuteLocal,
}

impl MomentumMicroLabelForensicsRunModeV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::DryRun => "dry-run",
            Self::ExecuteLocal => "execute-local",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroProtectedBeforeStateV1 {
    pub series_digest: String,
    pub event_two_outcome_receipt_digest: String,
    pub event_two_outcome_capsule_digest: String,
    pub opening_authorization_digest: String,
    pub opening_bundle_digest: String,
    pub event_two_ledger_entry_digest: String,
    pub eligibility_receipt_digest: String,
    pub completed_pause_digest: String,
    pub completed_event_count: usize,
    pub scorable_event_count: usize,
    pub eligibility_status: String,
    pub epoch_three_registered: bool,
    pub live_parameter_digests: Vec<String>,
    pub live_normalizer_digests: Vec<String>,
    pub protected_live_aggregate_digest: String,
    pub historical_store_digest: String,
    pub qualified_six_replay_digest: String,
    pub diagnostic_store_digest: String,
    pub active_roster_digest: String,
    pub zero_authority_and_action_counters: bool,
    pub state_digest: String,
}

pub fn momentum_micro_protected_before_state_digest_v1(
    value: &MomentumMicroProtectedBeforeStateV1,
) -> String {
    canonical_digest(value, |item| item.state_digest.clear())
}

pub fn validate_momentum_micro_protected_before_state_v1(
    value: &MomentumMicroProtectedBeforeStateV1,
) -> Result<(), String> {
    if [
        &value.series_digest,
        &value.event_two_outcome_receipt_digest,
        &value.event_two_outcome_capsule_digest,
        &value.opening_authorization_digest,
        &value.opening_bundle_digest,
        &value.event_two_ledger_entry_digest,
        &value.eligibility_receipt_digest,
        &value.completed_pause_digest,
        &value.protected_live_aggregate_digest,
        &value.historical_store_digest,
        &value.qualified_six_replay_digest,
        &value.diagnostic_store_digest,
        &value.active_roster_digest,
    ]
    .iter()
    .any(|value| value.is_empty())
        || value.completed_event_count != 2
        || value.scorable_event_count != 2
        || value.eligibility_status != "IneligibleMinimumSamples"
        || value.epoch_three_registered
        || value.live_parameter_digests.len() != 3
        || value.live_normalizer_digests.len() != 3
        || !value.zero_authority_and_action_counters
        || value.state_digest != momentum_micro_protected_before_state_digest_v1(value)
    {
        return Err("micro protected before-state rejected".to_string());
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroLabelForensicsRegistrationV1 {
    pub registration_version: String,
    pub source_replay_digest: String,
    pub source_diagnostic_digest: String,
    pub protected_before_state_digest: String,
    pub included_partitions: Vec<MomentumReplayPartitionV1>,
    pub candidate_horizons: Vec<MomentumMicroPredictionHorizonV1>,
    pub magnitude_distribution_policy_digest: String,
    pub prevalence_policy_digest: String,
    pub temporal_stability_policy_digest: String,
    pub target_overlap_policy_digest: String,
    pub serial_dependence_policy_digest: String,
    pub disposition_policy_digest: String,
    pub holdout_access_forbidden: bool,
    pub model_training_forbidden: bool,
    pub result_selected_threshold_forbidden: bool,
    pub post_result: bool,
    pub confirmatory_claim_allowed: bool,
    pub new_model_execution_allowed: bool,
    pub holdout_execution_allowed: bool,
    pub live_authority_allowed: bool,
    pub governance_authority_allowed: bool,
    pub trading_authority_allowed: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumMicroHorizonEventPlanV1 {
    plan_version: String,
    registration_digest: String,
    horizon: MomentumMicroPredictionHorizonV1,
    partition: MomentumReplayPartitionV1,
    event_timestamp_ms: Vec<u64>,
    target_timestamp_ms: Vec<u64>,
    non_overlapping_cadence: bool,
    holdout_event_count: usize,
    plan_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumMicroTargetDistributionV1 {
    pub distribution_version: String,
    pub horizon: MomentumMicroPredictionHorizonV1,
    pub partition: MomentumReplayPartitionV1,
    pub event_count: usize,
    pub scorable_count: usize,
    pub exact_zero_return_count: usize,
    pub positive_count: usize,
    pub negative_count: usize,
    pub neutral_count: usize,
    pub minimum: f64,
    pub percentile_01: f64,
    pub percentile_05: f64,
    pub percentile_25: f64,
    pub median: f64,
    pub percentile_75: f64,
    pub percentile_95: f64,
    pub percentile_99: f64,
    pub maximum: f64,
    pub mean: f64,
    pub standard_deviation: f64,
    pub mean_absolute_return: f64,
    pub median_absolute_return: f64,
    pub finite_value_proof: bool,
    pub extreme_value_integrity: String,
    pub distribution_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumMicroTemporalStabilityV1 {
    pub temporal_version: String,
    pub horizon: MomentumMicroPredictionHorizonV1,
    pub partition: MomentumReplayPartitionV1,
    pub grouping: String,
    pub group_count: usize,
    pub minimum_positive_prevalence: f64,
    pub maximum_positive_prevalence: f64,
    pub median_positive_prevalence: f64,
    pub prevalence_range: f64,
    pub sign_majority_flip_count: usize,
    pub rolling_prevalence_drift: f64,
    pub insufficient_support_groups: usize,
    pub temporal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroTargetOverlapReceiptV1 {
    pub overlap_version: String,
    pub horizon: MomentumMicroPredictionHorizonV1,
    pub event_cadence_ms: u64,
    pub target_horizon_ms: u64,
    pub event_count: usize,
    pub overlap_with_previous_count: usize,
    pub overlap_with_next_count: usize,
    pub maximum_simultaneous_target_overlap: usize,
    pub effective_unique_target_interval_count: usize,
    pub zero_overlap_required: bool,
    pub zero_overlap_verified: bool,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumMicroSerialDependenceReceiptV1 {
    pub serial_version: String,
    pub horizon: MomentumMicroPredictionHorizonV1,
    pub partition: MomentumReplayPartitionV1,
    pub lag: usize,
    pub paired_count: usize,
    pub sign_agreement_rate: Option<f64>,
    pub positive_after_positive_count: usize,
    pub positive_after_negative_count: usize,
    pub negative_after_positive_count: usize,
    pub negative_after_negative_count: usize,
    pub positive_after_positive_rate: Option<f64>,
    pub positive_after_negative_rate: Option<f64>,
    pub negative_after_positive_rate: Option<f64>,
    pub negative_after_negative_rate: Option<f64>,
    pub finite_integrity: bool,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroHorizonDispositionReceiptV1 {
    pub disposition_version: String,
    pub horizon: MomentumMicroPredictionHorizonV1,
    pub disposition: MomentumMicroHorizonDiagnosticDispositionV1,
    pub supporting_observation_digests: Vec<String>,
    pub model_execution_authorized: bool,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumMicroHorizonDiagnosticV1 {
    pub horizon: MomentumMicroPredictionHorizonV1,
    pub event_count: usize,
    pub scorable_count: usize,
    pub neutral_count: usize,
    pub distributions: Vec<MomentumMicroTargetDistributionV1>,
    pub temporal_stability: Vec<MomentumMicroTemporalStabilityV1>,
    pub overlap: MomentumMicroTargetOverlapReceiptV1,
    pub serial_dependence: Vec<MomentumMicroSerialDependenceReceiptV1>,
    pub disposition: MomentumMicroHorizonDispositionReceiptV1,
    pub diagnostic_digest: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroLabelForensicsSafetyCountersV1 {
    pub registration_writes_before_target_reads: usize,
    pub target_return_reads: usize,
    pub holdout_label_reads: usize,
    pub holdout_prediction_reads: usize,
    pub holdout_metric_reads: usize,
    pub holdout_execution_modes: usize,
    pub model_fits: usize,
    pub predictions: usize,
    pub evaluations: usize,
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
pub struct MomentumMicroLabelForensicsReportV1 {
    pub report_version: String,
    pub run_mode: String,
    pub status: MomentumMicroLabelForensicsStatusV1,
    pub evidence_class: MomentumMicroChallengerDesignEvidenceClassV1,
    pub post_result: bool,
    pub confirmatory_claim_allowed: bool,
    pub registration_digest: Option<String>,
    pub source_replay_digest: Option<String>,
    pub source_diagnostic_digest: Option<String>,
    pub protected_before_state_digest: String,
    pub horizons: Vec<MomentumMicroHorizonDiagnosticV1>,
    pub safety_counters: MomentumMicroLabelForensicsSafetyCountersV1,
    pub labels: Vec<String>,
    pub deterministic: bool,
    pub journal_digest: Option<String>,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub label_computation_count: usize,
    pub runtime_duration_ms: u64,
    pub report_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumMicroLabelForensicsJournalV1 {
    journal_version: String,
    registration_digest: String,
    horizon_diagnostic_digests: Vec<String>,
    holdout_label_reads: usize,
    holdout_prediction_reads: usize,
    holdout_metric_reads: usize,
    model_fits: usize,
    deterministic: bool,
    journal_digest: String,
}

#[derive(Clone, Debug)]
struct LabelObservation {
    partition: MomentumReplayPartitionV1,
    event_timestamp_ms: u64,
    target_timestamp_ms: u64,
    target_return: f64,
    status: MomentumMicroLabelStatusV1,
}

fn canonical_digest<T: Clone + std::fmt::Debug>(value: &T, clear: impl FnOnce(&mut T)) -> String {
    let mut canonical = value.clone();
    clear(&mut canonical);
    stable_hash_string(&format!("{canonical:?}"))
}

fn registration_digest(value: &MomentumMicroLabelForensicsRegistrationV1) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}

fn event_plan_digest(value: &MomentumMicroHorizonEventPlanV1) -> String {
    canonical_digest(value, |item| item.plan_digest.clear())
}

fn distribution_digest(value: &MomentumMicroTargetDistributionV1) -> String {
    canonical_digest(value, |item| item.distribution_digest.clear())
}

fn temporal_digest(value: &MomentumMicroTemporalStabilityV1) -> String {
    canonical_digest(value, |item| item.temporal_digest.clear())
}

fn overlap_digest(value: &MomentumMicroTargetOverlapReceiptV1) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn serial_digest(value: &MomentumMicroSerialDependenceReceiptV1) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn disposition_digest(value: &MomentumMicroHorizonDispositionReceiptV1) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn horizon_digest(value: &MomentumMicroHorizonDiagnosticV1) -> String {
    canonical_digest(value, |item| item.diagnostic_digest.clear())
}

fn journal_digest(value: &MomentumMicroLabelForensicsJournalV1) -> String {
    canonical_digest(value, |item| item.journal_digest.clear())
}

fn report_digest(value: &MomentumMicroLabelForensicsReportV1) -> String {
    canonical_digest(value, |item| {
        item.run_mode.clear();
        item.artifacts_written = 0;
        item.duplicate_artifact_count = 0;
        item.label_computation_count = 0;
        item.runtime_duration_ms = 0;
        item.safety_counters.registration_writes_before_target_reads = 0;
        item.safety_counters.target_return_reads = 0;
        item.report_digest.clear();
    })
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
        _ => Err("micro label partition rejected".to_string()),
    }
}

fn validate_registration(value: &MomentumMicroLabelForensicsRegistrationV1) -> Result<(), String> {
    if value.registration_version != REGISTRATION_VERSION
        || value.source_replay_digest.is_empty()
        || value.source_diagnostic_digest.is_empty()
        || value.protected_before_state_digest.is_empty()
        || value.included_partitions
            != [
                MomentumReplayPartitionV1::Development,
                MomentumReplayPartitionV1::Validation,
            ]
        || value.candidate_horizons != MomentumMicroPredictionHorizonV1::ORDERED
        || [
            &value.magnitude_distribution_policy_digest,
            &value.prevalence_policy_digest,
            &value.temporal_stability_policy_digest,
            &value.target_overlap_policy_digest,
            &value.serial_dependence_policy_digest,
            &value.disposition_policy_digest,
        ]
        .iter()
        .any(|digest| digest.is_empty())
        || !value.holdout_access_forbidden
        || !value.model_training_forbidden
        || !value.result_selected_threshold_forbidden
        || !value.post_result
        || value.confirmatory_claim_allowed
        || value.new_model_execution_allowed
        || value.holdout_execution_allowed
        || value.live_authority_allowed
        || value.governance_authority_allowed
        || value.trading_authority_allowed
        || value.registration_digest != registration_digest(value)
    {
        return Err("micro label registration rejected".to_string());
    }
    Ok(())
}

fn validate_distribution(value: &MomentumMicroTargetDistributionV1) -> Result<(), String> {
    let finite = [
        value.minimum,
        value.percentile_01,
        value.percentile_05,
        value.percentile_25,
        value.median,
        value.percentile_75,
        value.percentile_95,
        value.percentile_99,
        value.maximum,
        value.mean,
        value.standard_deviation,
        value.mean_absolute_return,
        value.median_absolute_return,
    ]
    .into_iter()
    .all(f64::is_finite);
    if value.distribution_version != DISTRIBUTION_VERSION
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.event_count == 0
        || value.positive_count + value.negative_count + value.neutral_count != value.event_count
        || value.scorable_count != value.positive_count + value.negative_count
        || value.exact_zero_return_count != value.neutral_count
        || !finite
        || !value.finite_value_proof
        || value.extreme_value_integrity.is_empty()
        || value.distribution_digest != distribution_digest(value)
    {
        return Err("micro target distribution rejected".to_string());
    }
    Ok(())
}

fn validate_temporal(value: &MomentumMicroTemporalStabilityV1) -> Result<(), String> {
    if value.temporal_version != TEMPORAL_VERSION
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.grouping.is_empty()
        || value.group_count == 0
        || [
            value.minimum_positive_prevalence,
            value.maximum_positive_prevalence,
            value.median_positive_prevalence,
            value.prevalence_range,
            value.rolling_prevalence_drift,
        ]
        .into_iter()
        .any(|value| !value.is_finite())
        || value.minimum_positive_prevalence < 0.0
        || value.maximum_positive_prevalence > 1.0
        || value.minimum_positive_prevalence > value.maximum_positive_prevalence
        || value.temporal_digest != temporal_digest(value)
    {
        return Err("micro temporal stability rejected".to_string());
    }
    Ok(())
}

fn validate_overlap(value: &MomentumMicroTargetOverlapReceiptV1) -> Result<(), String> {
    let non_overlapping = matches!(
        value.horizon,
        MomentumMicroPredictionHorizonV1::NextThirtyMinutes
            | MomentumMicroPredictionHorizonV1::NextSixtyMinutes
    );
    if value.overlap_version != OVERLAP_VERSION
        || value.event_cadence_ms != value.horizon.cadence_ms()
        || value.target_horizon_ms != value.horizon.horizon_candles() as u64 * TEN_MINUTE_MS
        || value.event_count == 0
        || value.effective_unique_target_interval_count != value.event_count
        || value.zero_overlap_required != non_overlapping
        || (non_overlapping
            && (value.overlap_with_previous_count != 0
                || value.overlap_with_next_count != 0
                || !value.zero_overlap_verified))
        || value.receipt_digest != overlap_digest(value)
    {
        return Err("micro target overlap receipt rejected".to_string());
    }
    Ok(())
}

fn validate_serial(value: &MomentumMicroSerialDependenceReceiptV1) -> Result<(), String> {
    let rates = [
        value.sign_agreement_rate,
        value.positive_after_positive_rate,
        value.positive_after_negative_rate,
        value.negative_after_positive_rate,
        value.negative_after_negative_rate,
    ];
    if value.serial_version != SERIAL_VERSION
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || !SERIAL_LAGS.contains(&value.lag)
        || rates
            .into_iter()
            .flatten()
            .any(|rate| !rate.is_finite() || !(0.0..=1.0).contains(&rate))
        || !value.finite_integrity
        || value.receipt_digest != serial_digest(value)
    {
        return Err("micro serial dependence rejected".to_string());
    }
    Ok(())
}

fn validate_horizon(value: &MomentumMicroHorizonDiagnosticV1) -> Result<(), String> {
    let partition_set = [
        MomentumReplayPartitionV1::Development,
        MomentumReplayPartitionV1::Validation,
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let distribution_partitions = value
        .distributions
        .iter()
        .map(|item| item.partition)
        .collect::<BTreeSet<_>>();
    let expected_temporal = partition_set
        .iter()
        .flat_map(|partition| {
            [
                "UtcDay",
                "UtcWeek",
                "UtcMonth",
                "Rolling144Events",
                "Rolling1008Events",
            ]
            .into_iter()
            .map(move |grouping| (*partition, grouping))
        })
        .collect::<BTreeSet<_>>();
    let temporal_set = value
        .temporal_stability
        .iter()
        .map(|item| (item.partition, item.grouping.as_str()))
        .collect::<BTreeSet<_>>();
    let expected_serial = partition_set
        .iter()
        .flat_map(|partition| SERIAL_LAGS.into_iter().map(move |lag| (*partition, lag)))
        .collect::<BTreeSet<_>>();
    let serial_set = value
        .serial_dependence
        .iter()
        .map(|item| (item.partition, item.lag))
        .collect::<BTreeSet<_>>();
    if value.event_count == 0
        || value.scorable_count + value.neutral_count != value.event_count
        || value.distributions.len() != 2
        || distribution_partitions != partition_set
        || value
            .distributions
            .iter()
            .map(|item| item.event_count)
            .sum::<usize>()
            != value.event_count
        || value.temporal_stability.len() != 10
        || temporal_set != expected_temporal
        || value.serial_dependence.len() != 10
        || serial_set != expected_serial
        || value
            .distributions
            .iter()
            .any(|item| item.horizon != value.horizon || validate_distribution(item).is_err())
        || value
            .temporal_stability
            .iter()
            .any(|item| item.horizon != value.horizon || validate_temporal(item).is_err())
        || value.overlap.horizon != value.horizon
        || validate_overlap(&value.overlap).is_err()
        || value
            .serial_dependence
            .iter()
            .any(|item| item.horizon != value.horizon || validate_serial(item).is_err())
        || value.disposition.horizon != value.horizon
        || value.disposition.disposition_version != DISPOSITION_VERSION
        || value.disposition.model_execution_authorized
        || value.disposition.receipt_digest != disposition_digest(&value.disposition)
        || value.diagnostic_digest != horizon_digest(value)
    {
        return Err("micro horizon diagnostic rejected".to_string());
    }
    Ok(())
}

fn zero_forbidden_counters(value: &MomentumMicroLabelForensicsSafetyCountersV1) -> bool {
    [
        value.holdout_label_reads,
        value.holdout_prediction_reads,
        value.holdout_metric_reads,
        value.holdout_execution_modes,
        value.model_fits,
        value.predictions,
        value.evaluations,
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

fn validate_report(value: &MomentumMicroLabelForensicsReportV1) -> Result<(), String> {
    let complete = value.status == MomentumMicroLabelForensicsStatusV1::Complete;
    let horizon_set = value
        .horizons
        .iter()
        .map(|item| item.horizon)
        .collect::<BTreeSet<_>>();
    if value.report_version != REPORT_VERSION
        || value.run_mode.is_empty()
        || value.evidence_class
            != MomentumMicroChallengerDesignEvidenceClassV1::PostResultResearchDesignOnly
        || !value.post_result
        || value.confirmatory_claim_allowed
        || value.protected_before_state_digest.is_empty()
        || value.labels != PUBLIC_LABELS.map(str::to_string)
        || !value.deterministic
        || !zero_forbidden_counters(&value.safety_counters)
        || (complete
            && (value
                .registration_digest
                .as_deref()
                .is_none_or(str::is_empty)
                || value
                    .source_replay_digest
                    .as_deref()
                    .is_none_or(str::is_empty)
                || value
                    .source_diagnostic_digest
                    .as_deref()
                    .is_none_or(str::is_empty)
                || value.horizons.len() != 3
                || horizon_set
                    != MomentumMicroPredictionHorizonV1::ORDERED
                        .into_iter()
                        .collect()
                || value.journal_digest.as_deref().is_none_or(str::is_empty)
                || value
                    .horizons
                    .iter()
                    .any(|item| validate_horizon(item).is_err())))
        || (!complete && !value.horizons.is_empty())
        || value.report_digest != report_digest(value)
    {
        return Err("micro label report rejected".to_string());
    }
    Ok(())
}

fn f64_bits(value: f64) -> u64 {
    value.to_bits()
}

fn optional_f64_bits(value: Option<f64>) -> Vec<u64> {
    value.into_iter().map(f64::to_bits).collect()
}

fn decode_optional_f64_bits(values: Vec<u64>) -> Result<Option<f64>, String> {
    if values.len() > 1 {
        return Err("micro optional float rejected".to_string());
    }
    Ok(values.into_iter().next().map(f64::from_bits))
}

fn encode_registration(
    value: &MomentumMicroLabelForensicsRegistrationV1,
) -> Result<Vec<u8>, String> {
    validate_registration(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroLabelForensicsRegistrationV1")
        .string("registration_version", &value.registration_version)
        .string("source_replay_digest", &value.source_replay_digest)
        .string("source_diagnostic_digest", &value.source_diagnostic_digest)
        .string(
            "protected_before_state_digest",
            &value.protected_before_state_digest,
        )
        .strings(
            "included_partitions",
            &value
                .included_partitions
                .iter()
                .map(|partition| partition_name(*partition).to_string())
                .collect::<Vec<_>>(),
        )
        .strings(
            "candidate_horizons",
            &value
                .candidate_horizons
                .iter()
                .map(|horizon| format!("{horizon:?}"))
                .collect::<Vec<_>>(),
        )
        .string(
            "magnitude_distribution_policy_digest",
            &value.magnitude_distribution_policy_digest,
        )
        .string("prevalence_policy_digest", &value.prevalence_policy_digest)
        .string(
            "temporal_stability_policy_digest",
            &value.temporal_stability_policy_digest,
        )
        .string(
            "target_overlap_policy_digest",
            &value.target_overlap_policy_digest,
        )
        .string(
            "serial_dependence_policy_digest",
            &value.serial_dependence_policy_digest,
        )
        .string(
            "disposition_policy_digest",
            &value.disposition_policy_digest,
        )
        .boolean("holdout_access_forbidden", value.holdout_access_forbidden)
        .boolean("model_training_forbidden", value.model_training_forbidden)
        .boolean(
            "result_selected_threshold_forbidden",
            value.result_selected_threshold_forbidden,
        )
        .boolean("post_result", value.post_result)
        .boolean(
            "confirmatory_claim_allowed",
            value.confirmatory_claim_allowed,
        )
        .boolean(
            "new_model_execution_allowed",
            value.new_model_execution_allowed,
        )
        .boolean("holdout_execution_allowed", value.holdout_execution_allowed)
        .boolean("live_authority_allowed", value.live_authority_allowed)
        .boolean(
            "governance_authority_allowed",
            value.governance_authority_allowed,
        )
        .boolean("trading_authority_allowed", value.trading_authority_allowed)
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_registration(bytes: &[u8]) -> Result<MomentumMicroLabelForensicsRegistrationV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumMicroLabelForensicsRegistrationV1")?;
    let value = MomentumMicroLabelForensicsRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        source_replay_digest: fields.string("source_replay_digest")?,
        source_diagnostic_digest: fields.string("source_diagnostic_digest")?,
        protected_before_state_digest: fields.string("protected_before_state_digest")?,
        included_partitions: fields
            .strings("included_partitions")?
            .iter()
            .map(|value| parse_partition(value))
            .collect::<Result<Vec<_>, _>>()?,
        candidate_horizons: fields
            .strings("candidate_horizons")?
            .iter()
            .map(|value| MomentumMicroPredictionHorizonV1::parse(value))
            .collect::<Result<Vec<_>, _>>()?,
        magnitude_distribution_policy_digest: fields
            .string("magnitude_distribution_policy_digest")?,
        prevalence_policy_digest: fields.string("prevalence_policy_digest")?,
        temporal_stability_policy_digest: fields.string("temporal_stability_policy_digest")?,
        target_overlap_policy_digest: fields.string("target_overlap_policy_digest")?,
        serial_dependence_policy_digest: fields.string("serial_dependence_policy_digest")?,
        disposition_policy_digest: fields.string("disposition_policy_digest")?,
        holdout_access_forbidden: fields.boolean("holdout_access_forbidden")?,
        model_training_forbidden: fields.boolean("model_training_forbidden")?,
        result_selected_threshold_forbidden: fields
            .boolean("result_selected_threshold_forbidden")?,
        post_result: fields.boolean("post_result")?,
        confirmatory_claim_allowed: fields.boolean("confirmatory_claim_allowed")?,
        new_model_execution_allowed: fields.boolean("new_model_execution_allowed")?,
        holdout_execution_allowed: fields.boolean("holdout_execution_allowed")?,
        live_authority_allowed: fields.boolean("live_authority_allowed")?,
        governance_authority_allowed: fields.boolean("governance_authority_allowed")?,
        trading_authority_allowed: fields.boolean("trading_authority_allowed")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_registration(&value)?;
    Ok(value)
}

fn encode_event_plan(value: &MomentumMicroHorizonEventPlanV1) -> Result<Vec<u8>, String> {
    if value.plan_version != EVENT_PLAN_VERSION
        || value.registration_digest.is_empty()
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.event_timestamp_ms.is_empty()
        || value.event_timestamp_ms.len() != value.target_timestamp_ms.len()
        || value
            .event_timestamp_ms
            .iter()
            .zip(&value.target_timestamp_ms)
            .any(|(event, target)| {
                *target
                    != event.saturating_add(value.horizon.horizon_candles() as u64 * TEN_MINUTE_MS)
            })
        || value.holdout_event_count != 0
        || value.plan_digest != event_plan_digest(value)
    {
        return Err("micro horizon event plan rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumMicroHorizonEventPlanV1")
        .string("plan_version", &value.plan_version)
        .string("registration_digest", &value.registration_digest)
        .string("horizon", format!("{:?}", value.horizon))
        .string("partition", partition_name(value.partition))
        .unsigneds("event_timestamp_ms", &value.event_timestamp_ms)
        .unsigneds("target_timestamp_ms", &value.target_timestamp_ms)
        .boolean("non_overlapping_cadence", value.non_overlapping_cadence)
        .unsigned("holdout_event_count", as_u64(value.holdout_event_count)?)
        .string("plan_digest", &value.plan_digest)
        .encode()
}

fn decode_event_plan(bytes: &[u8]) -> Result<MomentumMicroHorizonEventPlanV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroHorizonEventPlanV1")?;
    let value = MomentumMicroHorizonEventPlanV1 {
        plan_version: fields.string("plan_version")?,
        registration_digest: fields.string("registration_digest")?,
        horizon: MomentumMicroPredictionHorizonV1::parse(&fields.string("horizon")?)?,
        partition: parse_partition(&fields.string("partition")?)?,
        event_timestamp_ms: fields.unsigneds("event_timestamp_ms")?,
        target_timestamp_ms: fields.unsigneds("target_timestamp_ms")?,
        non_overlapping_cadence: fields.boolean("non_overlapping_cadence")?,
        holdout_event_count: as_usize(fields.unsigned("holdout_event_count")?)?,
        plan_digest: fields.string("plan_digest")?,
    };
    fields.finish()?;
    let encoded = encode_event_plan(&value)?;
    if encoded.is_empty() {
        return Err("micro horizon event plan reopen rejected".to_string());
    }
    Ok(value)
}

fn encode_distribution(value: &MomentumMicroTargetDistributionV1) -> Result<Vec<u8>, String> {
    validate_distribution(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroTargetDistributionV1")
        .string("distribution_version", &value.distribution_version)
        .string("horizon", format!("{:?}", value.horizon))
        .string("partition", partition_name(value.partition))
        .unsigned("event_count", as_u64(value.event_count)?)
        .unsigned("scorable_count", as_u64(value.scorable_count)?)
        .unsigned(
            "exact_zero_return_count",
            as_u64(value.exact_zero_return_count)?,
        )
        .unsigned("positive_count", as_u64(value.positive_count)?)
        .unsigned("negative_count", as_u64(value.negative_count)?)
        .unsigned("neutral_count", as_u64(value.neutral_count)?)
        .unsigneds(
            "distribution_value_bits",
            &[
                f64_bits(value.minimum),
                f64_bits(value.percentile_01),
                f64_bits(value.percentile_05),
                f64_bits(value.percentile_25),
                f64_bits(value.median),
                f64_bits(value.percentile_75),
                f64_bits(value.percentile_95),
                f64_bits(value.percentile_99),
                f64_bits(value.maximum),
                f64_bits(value.mean),
                f64_bits(value.standard_deviation),
                f64_bits(value.mean_absolute_return),
                f64_bits(value.median_absolute_return),
            ],
        )
        .boolean("finite_value_proof", value.finite_value_proof)
        .string("extreme_value_integrity", &value.extreme_value_integrity)
        .string("distribution_digest", &value.distribution_digest)
        .encode()
}

fn decode_distribution(bytes: &[u8]) -> Result<MomentumMicroTargetDistributionV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroTargetDistributionV1")?;
    let bits = fields.unsigneds("distribution_value_bits")?;
    if bits.len() != 13 {
        return Err("micro target distribution value count rejected".to_string());
    }
    let values = bits.into_iter().map(f64::from_bits).collect::<Vec<_>>();
    let value = MomentumMicroTargetDistributionV1 {
        distribution_version: fields.string("distribution_version")?,
        horizon: MomentumMicroPredictionHorizonV1::parse(&fields.string("horizon")?)?,
        partition: parse_partition(&fields.string("partition")?)?,
        event_count: as_usize(fields.unsigned("event_count")?)?,
        scorable_count: as_usize(fields.unsigned("scorable_count")?)?,
        exact_zero_return_count: as_usize(fields.unsigned("exact_zero_return_count")?)?,
        positive_count: as_usize(fields.unsigned("positive_count")?)?,
        negative_count: as_usize(fields.unsigned("negative_count")?)?,
        neutral_count: as_usize(fields.unsigned("neutral_count")?)?,
        minimum: values[0],
        percentile_01: values[1],
        percentile_05: values[2],
        percentile_25: values[3],
        median: values[4],
        percentile_75: values[5],
        percentile_95: values[6],
        percentile_99: values[7],
        maximum: values[8],
        mean: values[9],
        standard_deviation: values[10],
        mean_absolute_return: values[11],
        median_absolute_return: values[12],
        finite_value_proof: fields.boolean("finite_value_proof")?,
        extreme_value_integrity: fields.string("extreme_value_integrity")?,
        distribution_digest: fields.string("distribution_digest")?,
    };
    fields.finish()?;
    validate_distribution(&value)?;
    Ok(value)
}

fn encode_temporal(value: &MomentumMicroTemporalStabilityV1) -> Result<Vec<u8>, String> {
    validate_temporal(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroTemporalStabilityV1")
        .string("temporal_version", &value.temporal_version)
        .string("horizon", format!("{:?}", value.horizon))
        .string("partition", partition_name(value.partition))
        .string("grouping", &value.grouping)
        .unsigned("group_count", as_u64(value.group_count)?)
        .unsigneds(
            "prevalence_value_bits",
            &[
                f64_bits(value.minimum_positive_prevalence),
                f64_bits(value.maximum_positive_prevalence),
                f64_bits(value.median_positive_prevalence),
                f64_bits(value.prevalence_range),
                f64_bits(value.rolling_prevalence_drift),
            ],
        )
        .unsigned(
            "sign_majority_flip_count",
            as_u64(value.sign_majority_flip_count)?,
        )
        .unsigned(
            "insufficient_support_groups",
            as_u64(value.insufficient_support_groups)?,
        )
        .string("temporal_digest", &value.temporal_digest)
        .encode()
}

fn decode_temporal(bytes: &[u8]) -> Result<MomentumMicroTemporalStabilityV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroTemporalStabilityV1")?;
    let bits = fields.unsigneds("prevalence_value_bits")?;
    if bits.len() != 5 {
        return Err("micro temporal value count rejected".to_string());
    }
    let values = bits.into_iter().map(f64::from_bits).collect::<Vec<_>>();
    let value = MomentumMicroTemporalStabilityV1 {
        temporal_version: fields.string("temporal_version")?,
        horizon: MomentumMicroPredictionHorizonV1::parse(&fields.string("horizon")?)?,
        partition: parse_partition(&fields.string("partition")?)?,
        grouping: fields.string("grouping")?,
        group_count: as_usize(fields.unsigned("group_count")?)?,
        minimum_positive_prevalence: values[0],
        maximum_positive_prevalence: values[1],
        median_positive_prevalence: values[2],
        prevalence_range: values[3],
        rolling_prevalence_drift: values[4],
        sign_majority_flip_count: as_usize(fields.unsigned("sign_majority_flip_count")?)?,
        insufficient_support_groups: as_usize(fields.unsigned("insufficient_support_groups")?)?,
        temporal_digest: fields.string("temporal_digest")?,
    };
    fields.finish()?;
    validate_temporal(&value)?;
    Ok(value)
}

fn encode_overlap(value: &MomentumMicroTargetOverlapReceiptV1) -> Result<Vec<u8>, String> {
    validate_overlap(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroTargetOverlapReceiptV1")
        .string("overlap_version", &value.overlap_version)
        .string("horizon", format!("{:?}", value.horizon))
        .unsigned("event_cadence_ms", value.event_cadence_ms)
        .unsigned("target_horizon_ms", value.target_horizon_ms)
        .unsigned("event_count", as_u64(value.event_count)?)
        .unsigned(
            "overlap_with_previous_count",
            as_u64(value.overlap_with_previous_count)?,
        )
        .unsigned(
            "overlap_with_next_count",
            as_u64(value.overlap_with_next_count)?,
        )
        .unsigned(
            "maximum_simultaneous_target_overlap",
            as_u64(value.maximum_simultaneous_target_overlap)?,
        )
        .unsigned(
            "effective_unique_target_interval_count",
            as_u64(value.effective_unique_target_interval_count)?,
        )
        .boolean("zero_overlap_required", value.zero_overlap_required)
        .boolean("zero_overlap_verified", value.zero_overlap_verified)
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_overlap(bytes: &[u8]) -> Result<MomentumMicroTargetOverlapReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroTargetOverlapReceiptV1")?;
    let value = MomentumMicroTargetOverlapReceiptV1 {
        overlap_version: fields.string("overlap_version")?,
        horizon: MomentumMicroPredictionHorizonV1::parse(&fields.string("horizon")?)?,
        event_cadence_ms: fields.unsigned("event_cadence_ms")?,
        target_horizon_ms: fields.unsigned("target_horizon_ms")?,
        event_count: as_usize(fields.unsigned("event_count")?)?,
        overlap_with_previous_count: as_usize(fields.unsigned("overlap_with_previous_count")?)?,
        overlap_with_next_count: as_usize(fields.unsigned("overlap_with_next_count")?)?,
        maximum_simultaneous_target_overlap: as_usize(
            fields.unsigned("maximum_simultaneous_target_overlap")?,
        )?,
        effective_unique_target_interval_count: as_usize(
            fields.unsigned("effective_unique_target_interval_count")?,
        )?,
        zero_overlap_required: fields.boolean("zero_overlap_required")?,
        zero_overlap_verified: fields.boolean("zero_overlap_verified")?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_overlap(&value)?;
    Ok(value)
}

fn encode_serial(value: &MomentumMicroSerialDependenceReceiptV1) -> Result<Vec<u8>, String> {
    validate_serial(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroSerialDependenceReceiptV1")
        .string("serial_version", &value.serial_version)
        .string("horizon", format!("{:?}", value.horizon))
        .string("partition", partition_name(value.partition))
        .unsigned("lag", as_u64(value.lag)?)
        .unsigned("paired_count", as_u64(value.paired_count)?)
        .unsigneds(
            "sign_agreement_rate",
            &optional_f64_bits(value.sign_agreement_rate),
        )
        .unsigned(
            "positive_after_positive_count",
            as_u64(value.positive_after_positive_count)?,
        )
        .unsigned(
            "positive_after_negative_count",
            as_u64(value.positive_after_negative_count)?,
        )
        .unsigned(
            "negative_after_positive_count",
            as_u64(value.negative_after_positive_count)?,
        )
        .unsigned(
            "negative_after_negative_count",
            as_u64(value.negative_after_negative_count)?,
        )
        .unsigneds(
            "positive_after_positive_rate",
            &optional_f64_bits(value.positive_after_positive_rate),
        )
        .unsigneds(
            "positive_after_negative_rate",
            &optional_f64_bits(value.positive_after_negative_rate),
        )
        .unsigneds(
            "negative_after_positive_rate",
            &optional_f64_bits(value.negative_after_positive_rate),
        )
        .unsigneds(
            "negative_after_negative_rate",
            &optional_f64_bits(value.negative_after_negative_rate),
        )
        .boolean("finite_integrity", value.finite_integrity)
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_serial(bytes: &[u8]) -> Result<MomentumMicroSerialDependenceReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroSerialDependenceReceiptV1")?;
    let value = MomentumMicroSerialDependenceReceiptV1 {
        serial_version: fields.string("serial_version")?,
        horizon: MomentumMicroPredictionHorizonV1::parse(&fields.string("horizon")?)?,
        partition: parse_partition(&fields.string("partition")?)?,
        lag: as_usize(fields.unsigned("lag")?)?,
        paired_count: as_usize(fields.unsigned("paired_count")?)?,
        sign_agreement_rate: decode_optional_f64_bits(fields.unsigneds("sign_agreement_rate")?)?,
        positive_after_positive_count: as_usize(fields.unsigned("positive_after_positive_count")?)?,
        positive_after_negative_count: as_usize(fields.unsigned("positive_after_negative_count")?)?,
        negative_after_positive_count: as_usize(fields.unsigned("negative_after_positive_count")?)?,
        negative_after_negative_count: as_usize(fields.unsigned("negative_after_negative_count")?)?,
        positive_after_positive_rate: decode_optional_f64_bits(
            fields.unsigneds("positive_after_positive_rate")?,
        )?,
        positive_after_negative_rate: decode_optional_f64_bits(
            fields.unsigneds("positive_after_negative_rate")?,
        )?,
        negative_after_positive_rate: decode_optional_f64_bits(
            fields.unsigneds("negative_after_positive_rate")?,
        )?,
        negative_after_negative_rate: decode_optional_f64_bits(
            fields.unsigneds("negative_after_negative_rate")?,
        )?,
        finite_integrity: fields.boolean("finite_integrity")?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_serial(&value)?;
    Ok(value)
}

fn encode_disposition(value: &MomentumMicroHorizonDispositionReceiptV1) -> Result<Vec<u8>, String> {
    if value.disposition_version != DISPOSITION_VERSION
        || value.supporting_observation_digests.is_empty()
        || value.model_execution_authorized
        || value.receipt_digest != disposition_digest(value)
    {
        return Err("micro horizon disposition rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumMicroHorizonDispositionReceiptV1")
        .string("disposition_version", &value.disposition_version)
        .string("horizon", format!("{:?}", value.horizon))
        .string("disposition", format!("{:?}", value.disposition))
        .strings(
            "supporting_observation_digests",
            &value.supporting_observation_digests,
        )
        .boolean(
            "model_execution_authorized",
            value.model_execution_authorized,
        )
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_disposition(bytes: &[u8]) -> Result<MomentumMicroHorizonDispositionReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroHorizonDispositionReceiptV1")?;
    let value = MomentumMicroHorizonDispositionReceiptV1 {
        disposition_version: fields.string("disposition_version")?,
        horizon: MomentumMicroPredictionHorizonV1::parse(&fields.string("horizon")?)?,
        disposition: parse_disposition(&fields.string("disposition")?)?,
        supporting_observation_digests: fields.strings("supporting_observation_digests")?,
        model_execution_authorized: fields.boolean("model_execution_authorized")?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    let encoded = encode_disposition(&value)?;
    if encoded.is_empty() {
        return Err("micro horizon disposition reopen rejected".to_string());
    }
    Ok(value)
}

fn encode_horizon(value: &MomentumMicroHorizonDiagnosticV1) -> Result<Vec<u8>, String> {
    validate_horizon(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroHorizonDiagnosticV1")
        .string("horizon", format!("{:?}", value.horizon))
        .unsigned("event_count", as_u64(value.event_count)?)
        .unsigned("scorable_count", as_u64(value.scorable_count)?)
        .unsigned("neutral_count", as_u64(value.neutral_count)?)
        .messages(
            "distributions",
            value
                .distributions
                .iter()
                .map(encode_distribution)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "temporal_stability",
            value
                .temporal_stability
                .iter()
                .map(encode_temporal)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages("overlap", vec![encode_overlap(&value.overlap)?])
        .messages(
            "serial_dependence",
            value
                .serial_dependence
                .iter()
                .map(encode_serial)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages("disposition", vec![encode_disposition(&value.disposition)?])
        .string("diagnostic_digest", &value.diagnostic_digest)
        .encode()
}

fn decode_horizon(bytes: &[u8]) -> Result<MomentumMicroHorizonDiagnosticV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroHorizonDiagnosticV1")?;
    let overlap = fields.messages("overlap")?;
    let disposition = fields.messages("disposition")?;
    if overlap.len() != 1 || disposition.len() != 1 {
        return Err("micro horizon nested identity rejected".to_string());
    }
    let value = MomentumMicroHorizonDiagnosticV1 {
        horizon: MomentumMicroPredictionHorizonV1::parse(&fields.string("horizon")?)?,
        event_count: as_usize(fields.unsigned("event_count")?)?,
        scorable_count: as_usize(fields.unsigned("scorable_count")?)?,
        neutral_count: as_usize(fields.unsigned("neutral_count")?)?,
        distributions: fields
            .messages("distributions")?
            .iter()
            .map(|bytes| decode_distribution(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        temporal_stability: fields
            .messages("temporal_stability")?
            .iter()
            .map(|bytes| decode_temporal(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        overlap: decode_overlap(&overlap[0])?,
        serial_dependence: fields
            .messages("serial_dependence")?
            .iter()
            .map(|bytes| decode_serial(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        disposition: decode_disposition(&disposition[0])?,
        diagnostic_digest: fields.string("diagnostic_digest")?,
    };
    fields.finish()?;
    validate_horizon(&value)?;
    Ok(value)
}

fn encode_report(value: &MomentumMicroLabelForensicsReportV1) -> Result<Vec<u8>, String> {
    validate_report(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroLabelForensicsReportV1")
        .string("report_version", &value.report_version)
        .string("run_mode", &value.run_mode)
        .string("status", format!("{:?}", value.status))
        .string("evidence_class", format!("{:?}", value.evidence_class))
        .boolean("post_result", value.post_result)
        .boolean(
            "confirmatory_claim_allowed",
            value.confirmatory_claim_allowed,
        )
        .optional_string("registration_digest", &value.registration_digest)
        .optional_string("source_replay_digest", &value.source_replay_digest)
        .optional_string("source_diagnostic_digest", &value.source_diagnostic_digest)
        .string(
            "protected_before_state_digest",
            &value.protected_before_state_digest,
        )
        .messages(
            "horizons",
            value
                .horizons
                .iter()
                .map(encode_horizon)
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
            "label_computation_count",
            as_u64(value.label_computation_count)?,
        )
        .unsigned("runtime_duration_ms", value.runtime_duration_ms)
        .string("report_digest", &value.report_digest)
        .encode()
}

fn parse_report_status(value: &str) -> Result<MomentumMicroLabelForensicsStatusV1, String> {
    use MomentumMicroLabelForensicsStatusV1 as S;
    match value {
        "Unregistered" => Ok(S::Unregistered),
        "Registered" => Ok(S::Registered),
        "Complete" => Ok(S::Complete),
        "IntegrityFailure" => Ok(S::IntegrityFailure),
        _ => Err("micro label report status rejected".to_string()),
    }
}

fn encode_safety(value: &MomentumMicroLabelForensicsSafetyCountersV1) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumMicroLabelForensicsSafetyCountersV1")
        .unsigned(
            "registration_writes_before_target_reads",
            as_u64(value.registration_writes_before_target_reads)?,
        )
        .unsigned("target_return_reads", as_u64(value.target_return_reads)?)
        .unsigned("holdout_label_reads", as_u64(value.holdout_label_reads)?)
        .unsigned(
            "holdout_prediction_reads",
            as_u64(value.holdout_prediction_reads)?,
        )
        .unsigned("holdout_metric_reads", as_u64(value.holdout_metric_reads)?)
        .unsigned(
            "holdout_execution_modes",
            as_u64(value.holdout_execution_modes)?,
        )
        .unsigned("model_fits", as_u64(value.model_fits)?)
        .unsigned("predictions", as_u64(value.predictions)?)
        .unsigned("evaluations", as_u64(value.evaluations)?)
        .unsigned(
            "live_network_requests",
            as_u64(value.live_network_requests)?,
        )
        .unsigned(
            "live_parameter_updates",
            as_u64(value.live_parameter_updates)?,
        )
        .unsigned(
            "live_normalizer_refits",
            as_u64(value.live_normalizer_refits)?,
        )
        .unsigned("live_event_changes", as_u64(value.live_event_changes)?)
        .unsigned("winner_selections", as_u64(value.winner_selections)?)
        .unsigned("rankings", as_u64(value.rankings)?)
        .unsigned("reward_applications", as_u64(value.reward_applications)?)
        .unsigned("penalty_applications", as_u64(value.penalty_applications)?)
        .unsigned("chair_actions", as_u64(value.chair_actions)?)
        .unsigned("trading_actions", as_u64(value.trading_actions)?)
        .encode()
}

fn decode_safety(bytes: &[u8]) -> Result<MomentumMicroLabelForensicsSafetyCountersV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumMicroLabelForensicsSafetyCountersV1")?;
    let value = MomentumMicroLabelForensicsSafetyCountersV1 {
        registration_writes_before_target_reads: as_usize(
            fields.unsigned("registration_writes_before_target_reads")?,
        )?,
        target_return_reads: as_usize(fields.unsigned("target_return_reads")?)?,
        holdout_label_reads: as_usize(fields.unsigned("holdout_label_reads")?)?,
        holdout_prediction_reads: as_usize(fields.unsigned("holdout_prediction_reads")?)?,
        holdout_metric_reads: as_usize(fields.unsigned("holdout_metric_reads")?)?,
        holdout_execution_modes: as_usize(fields.unsigned("holdout_execution_modes")?)?,
        model_fits: as_usize(fields.unsigned("model_fits")?)?,
        predictions: as_usize(fields.unsigned("predictions")?)?,
        evaluations: as_usize(fields.unsigned("evaluations")?)?,
        live_network_requests: as_usize(fields.unsigned("live_network_requests")?)?,
        live_parameter_updates: as_usize(fields.unsigned("live_parameter_updates")?)?,
        live_normalizer_refits: as_usize(fields.unsigned("live_normalizer_refits")?)?,
        live_event_changes: as_usize(fields.unsigned("live_event_changes")?)?,
        winner_selections: as_usize(fields.unsigned("winner_selections")?)?,
        rankings: as_usize(fields.unsigned("rankings")?)?,
        reward_applications: as_usize(fields.unsigned("reward_applications")?)?,
        penalty_applications: as_usize(fields.unsigned("penalty_applications")?)?,
        chair_actions: as_usize(fields.unsigned("chair_actions")?)?,
        trading_actions: as_usize(fields.unsigned("trading_actions")?)?,
    };
    fields.finish()?;
    Ok(value)
}

fn decode_report(bytes: &[u8]) -> Result<MomentumMicroLabelForensicsReportV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroLabelForensicsReportV1")?;
    let safety = fields.messages("safety_counters")?;
    if safety.len() != 1 {
        return Err("micro label report safety identity rejected".to_string());
    }
    let value = MomentumMicroLabelForensicsReportV1 {
        report_version: fields.string("report_version")?,
        run_mode: fields.string("run_mode")?,
        status: parse_report_status(&fields.string("status")?)?,
        evidence_class: match fields.string("evidence_class")?.as_str() {
            "PostResultResearchDesignOnly" => {
                MomentumMicroChallengerDesignEvidenceClassV1::PostResultResearchDesignOnly
            }
            _ => return Err("micro label evidence class rejected".to_string()),
        },
        post_result: fields.boolean("post_result")?,
        confirmatory_claim_allowed: fields.boolean("confirmatory_claim_allowed")?,
        registration_digest: fields.optional_string("registration_digest")?,
        source_replay_digest: fields.optional_string("source_replay_digest")?,
        source_diagnostic_digest: fields.optional_string("source_diagnostic_digest")?,
        protected_before_state_digest: fields.string("protected_before_state_digest")?,
        horizons: fields
            .messages("horizons")?
            .iter()
            .map(|bytes| decode_horizon(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        safety_counters: decode_safety(&safety[0])?,
        labels: fields.strings("labels")?,
        deterministic: fields.boolean("deterministic")?,
        journal_digest: fields.optional_string("journal_digest")?,
        artifacts_written: as_usize(fields.unsigned("artifacts_written")?)?,
        duplicate_artifact_count: as_usize(fields.unsigned("duplicate_artifact_count")?)?,
        label_computation_count: as_usize(fields.unsigned("label_computation_count")?)?,
        runtime_duration_ms: fields.unsigned("runtime_duration_ms")?,
        report_digest: fields.string("report_digest")?,
    };
    fields.finish()?;
    validate_report(&value)?;
    Ok(value)
}

fn encode_journal(value: &MomentumMicroLabelForensicsJournalV1) -> Result<Vec<u8>, String> {
    if value.journal_version != JOURNAL_VERSION
        || value.registration_digest.is_empty()
        || value.horizon_diagnostic_digests.len() != 3
        || value.holdout_label_reads != 0
        || value.holdout_prediction_reads != 0
        || value.holdout_metric_reads != 0
        || value.model_fits != 0
        || !value.deterministic
        || value.journal_digest != journal_digest(value)
    {
        return Err("micro label journal rejected".to_string());
    }
    ArtifactBuilderV4_2::new("MomentumMicroLabelForensicsJournalV1")
        .string("journal_version", &value.journal_version)
        .string("registration_digest", &value.registration_digest)
        .strings(
            "horizon_diagnostic_digests",
            &value.horizon_diagnostic_digests,
        )
        .unsigned("holdout_label_reads", as_u64(value.holdout_label_reads)?)
        .unsigned(
            "holdout_prediction_reads",
            as_u64(value.holdout_prediction_reads)?,
        )
        .unsigned("holdout_metric_reads", as_u64(value.holdout_metric_reads)?)
        .unsigned("model_fits", as_u64(value.model_fits)?)
        .boolean("deterministic", value.deterministic)
        .string("journal_digest", &value.journal_digest)
        .encode()
}

fn decode_journal(bytes: &[u8]) -> Result<MomentumMicroLabelForensicsJournalV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroLabelForensicsJournalV1")?;
    let value = MomentumMicroLabelForensicsJournalV1 {
        journal_version: fields.string("journal_version")?,
        registration_digest: fields.string("registration_digest")?,
        horizon_diagnostic_digests: fields.strings("horizon_diagnostic_digests")?,
        holdout_label_reads: as_usize(fields.unsigned("holdout_label_reads")?)?,
        holdout_prediction_reads: as_usize(fields.unsigned("holdout_prediction_reads")?)?,
        holdout_metric_reads: as_usize(fields.unsigned("holdout_metric_reads")?)?,
        model_fits: as_usize(fields.unsigned("model_fits")?)?,
        deterministic: fields.boolean("deterministic")?,
        journal_digest: fields.string("journal_digest")?,
    };
    fields.finish()?;
    let encoded = encode_journal(&value)?;
    if encoded.is_empty() {
        return Err("micro label journal reopen rejected".to_string());
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

pub(super) fn read_momentum_micro_label_registration_v1()
-> Result<Option<MomentumMicroLabelForensicsRegistrationV1>, String> {
    read_single(&Path::new(ROOT).join("registrations"), decode_registration)
}

pub fn read_momentum_micro_label_forensics_report_v1()
-> Result<Option<MomentumMicroLabelForensicsReportV1>, String> {
    read_single(&Path::new(ROOT).join("final_reports"), decode_report)
}

fn derive_registration(
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> Result<MomentumMicroLabelForensicsRegistrationV1, String> {
    validate_momentum_micro_protected_before_state_v1(protected)?;
    let header = load_momentum_qualified_diagnostic_source_header_v1()?;
    let diagnostic = read_momentum_qualified_six_diagnostic_report_snapshot_v1()?
        .ok_or_else(|| "micro label source diagnostic unavailable".to_string())?;
    if diagnostic.status != MomentumQualifiedDiagnosticStatusV1::Complete
        || diagnostic.evidence_class
            != MomentumQualifiedDiagnosticEvidenceClassV1::PostResultDiagnosticOnly
        || diagnostic.confirmatory_claim_allowed
        || diagnostic.holdout_label_reads != 0
        || diagnostic.holdout_prediction_reads != 0
        || diagnostic.holdout_metric_reads != 0
        || diagnostic.holdout_execution_modes != 0
        || diagnostic.report_digest.is_empty()
    {
        return Err("micro label source diagnostic rejected".to_string());
    }
    let mut value = MomentumMicroLabelForensicsRegistrationV1 {
        registration_version: REGISTRATION_VERSION.to_string(),
        source_replay_digest: header.replay_journal_digest,
        source_diagnostic_digest: diagnostic.report_digest,
        protected_before_state_digest: protected.state_digest.clone(),
        included_partitions: vec![
            MomentumReplayPartitionV1::Development,
            MomentumReplayPartitionV1::Validation,
        ],
        candidate_horizons: MomentumMicroPredictionHorizonV1::ORDERED.to_vec(),
        magnitude_distribution_policy_digest: stable_hash_string(
            "micro-label-magnitude-v1:finite:min:p01:p05:p25:p50:p75:p95:p99:max:mean:std:mean-abs:median-abs",
        ),
        prevalence_policy_digest: stable_hash_string(
            "micro-label-prevalence-v1:up=strict-greater:down=strict-less:neutral=equal:no-dead-zone",
        ),
        temporal_stability_policy_digest: stable_hash_string(
            "micro-label-temporal-v1:utc-day:utc-week:utc-month:144-events:1008-events:no-removal",
        ),
        target_overlap_policy_digest: stable_hash_string(
            "micro-label-overlap-v1:t10-dense:t30-nonoverlap:t60-nonoverlap:half-open-intervals",
        ),
        serial_dependence_policy_digest: stable_hash_string(
            "micro-label-serial-v1:scorable-only:lags=1,2,3,6,12:aggregate-only",
        ),
        disposition_policy_digest: stable_hash_string(
            "micro-label-disposition-v1:min-scorable=144:median-abs-min=0.0001:prevalence=0.35..0.65:range-max=0.35:no-execution",
        ),
        holdout_access_forbidden: true,
        model_training_forbidden: true,
        result_selected_threshold_forbidden: true,
        post_result: true,
        confirmatory_claim_allowed: false,
        new_model_execution_allowed: false,
        holdout_execution_allowed: false,
        live_authority_allowed: false,
        governance_authority_allowed: false,
        trading_authority_allowed: false,
        registration_digest: String::new(),
    };
    value.registration_digest = registration_digest(&value);
    validate_registration(&value)?;
    Ok(value)
}

fn empty_report(
    mode: MomentumMicroLabelForensicsRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> MomentumMicroLabelForensicsReportV1 {
    MomentumMicroLabelForensicsReportV1 {
        report_version: REPORT_VERSION.to_string(),
        run_mode: mode.as_str().to_string(),
        status: MomentumMicroLabelForensicsStatusV1::Unregistered,
        evidence_class: MomentumMicroChallengerDesignEvidenceClassV1::PostResultResearchDesignOnly,
        post_result: true,
        confirmatory_claim_allowed: false,
        registration_digest: None,
        source_replay_digest: None,
        source_diagnostic_digest: None,
        protected_before_state_digest: protected.state_digest.clone(),
        horizons: Vec::new(),
        safety_counters: MomentumMicroLabelForensicsSafetyCountersV1::default(),
        labels: PUBLIC_LABELS.map(str::to_string).to_vec(),
        deterministic: true,
        journal_digest: None,
        artifacts_written: 0,
        duplicate_artifact_count: 0,
        label_computation_count: 0,
        runtime_duration_ms: 0,
        report_digest: String::new(),
    }
}

fn percentile(sorted: &[f64], fraction: f64) -> Result<f64, String> {
    if sorted.is_empty() || !(0.0..=1.0).contains(&fraction) {
        return Err("micro percentile input rejected".to_string());
    }
    let index = ((sorted.len() - 1) as f64 * fraction).round() as usize;
    Ok(sorted[index])
}

fn distribution(
    horizon: MomentumMicroPredictionHorizonV1,
    partition: MomentumReplayPartitionV1,
    observations: &[LabelObservation],
) -> Result<MomentumMicroTargetDistributionV1, String> {
    let mut values = observations
        .iter()
        .map(|item| item.target_return)
        .collect::<Vec<_>>();
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err("micro target distribution source rejected".to_string());
    }
    values.sort_by(f64::total_cmp);
    let mut absolute = values.iter().map(|value| value.abs()).collect::<Vec<_>>();
    absolute.sort_by(f64::total_cmp);
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64;
    let positive_count = observations
        .iter()
        .filter(|item| item.status == MomentumMicroLabelStatusV1::Up)
        .count();
    let negative_count = observations
        .iter()
        .filter(|item| item.status == MomentumMicroLabelStatusV1::Down)
        .count();
    let neutral_count = observations.len() - positive_count - negative_count;
    let mut value = MomentumMicroTargetDistributionV1 {
        distribution_version: DISTRIBUTION_VERSION.to_string(),
        horizon,
        partition,
        event_count: observations.len(),
        scorable_count: positive_count + negative_count,
        exact_zero_return_count: values.iter().filter(|value| **value == 0.0).count(),
        positive_count,
        negative_count,
        neutral_count,
        minimum: values[0],
        percentile_01: percentile(&values, 0.01)?,
        percentile_05: percentile(&values, 0.05)?,
        percentile_25: percentile(&values, 0.25)?,
        median: percentile(&values, 0.50)?,
        percentile_75: percentile(&values, 0.75)?,
        percentile_95: percentile(&values, 0.95)?,
        percentile_99: percentile(&values, 0.99)?,
        maximum: *values.last().unwrap_or(&values[0]),
        mean,
        standard_deviation: variance.sqrt(),
        mean_absolute_return: absolute.iter().sum::<f64>() / absolute.len() as f64,
        median_absolute_return: percentile(&absolute, 0.50)?,
        finite_value_proof: true,
        extreme_value_integrity: if values.iter().any(|value| value.abs() > 0.25) {
            "FiniteExtremeObserved"
        } else {
            "FiniteWithinRegisteredBounds"
        }
        .to_string(),
        distribution_digest: String::new(),
    };
    value.distribution_digest = distribution_digest(&value);
    validate_distribution(&value)?;
    Ok(value)
}

fn civil_month_key(timestamp_ms: u64) -> i64 {
    let days = (timestamp_ms / DAY_MS) as i64;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    year * 12 + month
}

fn prevalence_summary(
    horizon: MomentumMicroPredictionHorizonV1,
    partition: MomentumReplayPartitionV1,
    grouping: &str,
    groups: Vec<Vec<&LabelObservation>>,
    insufficient_support_groups: usize,
) -> Result<MomentumMicroTemporalStabilityV1, String> {
    let mut prevalence = groups
        .iter()
        .filter(|group| !group.is_empty())
        .map(|group| {
            group
                .iter()
                .filter(|item| item.status == MomentumMicroLabelStatusV1::Up)
                .count() as f64
                / group.len() as f64
        })
        .collect::<Vec<_>>();
    if prevalence.is_empty() || prevalence.iter().any(|value| !value.is_finite()) {
        return Err("micro prevalence source rejected".to_string());
    }
    let sign_majority_flip_count = prevalence
        .windows(2)
        .filter(|pair| (pair[0] >= 0.5) != (pair[1] >= 0.5))
        .count();
    let minimum = prevalence.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = prevalence.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    prevalence.sort_by(f64::total_cmp);
    let median = percentile(&prevalence, 0.50)?;
    let mut value = MomentumMicroTemporalStabilityV1 {
        temporal_version: TEMPORAL_VERSION.to_string(),
        horizon,
        partition,
        grouping: grouping.to_string(),
        group_count: prevalence.len(),
        minimum_positive_prevalence: minimum,
        maximum_positive_prevalence: maximum,
        median_positive_prevalence: median,
        prevalence_range: maximum - minimum,
        sign_majority_flip_count,
        rolling_prevalence_drift: maximum - minimum,
        insufficient_support_groups,
        temporal_digest: String::new(),
    };
    value.temporal_digest = temporal_digest(&value);
    validate_temporal(&value)?;
    Ok(value)
}

fn temporal_stability(
    horizon: MomentumMicroPredictionHorizonV1,
    partition: MomentumReplayPartitionV1,
    observations: &[LabelObservation],
) -> Result<Vec<MomentumMicroTemporalStabilityV1>, String> {
    let mut output = Vec::new();
    for (name, key) in [("UtcDay", 0_u8), ("UtcWeek", 1_u8), ("UtcMonth", 2_u8)] {
        let mut grouped: BTreeMap<i64, Vec<&LabelObservation>> = BTreeMap::new();
        for item in observations {
            let group = match key {
                0 => (item.event_timestamp_ms / DAY_MS) as i64,
                1 => (item.event_timestamp_ms / WEEK_MS) as i64,
                _ => civil_month_key(item.event_timestamp_ms),
            };
            grouped.entry(group).or_default().push(item);
        }
        let insufficient = grouped.values().filter(|group| group.len() < 2).count();
        output.push(prevalence_summary(
            horizon,
            partition,
            name,
            grouped.into_values().collect(),
            insufficient,
        )?);
    }
    for (name, window) in [
        ("Rolling144Events", ROLLING_DAY_EVENTS),
        ("Rolling1008Events", ROLLING_WEEK_EVENTS),
    ] {
        let groups = if observations.len() < window {
            vec![observations.iter().collect::<Vec<_>>()]
        } else {
            observations
                .windows(window)
                .map(|window| window.iter().collect::<Vec<_>>())
                .collect::<Vec<_>>()
        };
        output.push(prevalence_summary(
            horizon,
            partition,
            name,
            groups,
            usize::from(observations.len() < window),
        )?);
    }
    Ok(output)
}

fn overlap_receipt(
    horizon: MomentumMicroPredictionHorizonV1,
    observations: &[LabelObservation],
) -> Result<MomentumMicroTargetOverlapReceiptV1, String> {
    let mut intervals = observations
        .iter()
        .map(|item| (item.event_timestamp_ms, item.target_timestamp_ms))
        .collect::<Vec<_>>();
    intervals.sort();
    intervals.dedup();
    let previous = intervals
        .windows(2)
        .filter(|pair| pair[1].0 < pair[0].1)
        .count();
    let next = intervals
        .windows(2)
        .filter(|pair| pair[0].1 > pair[1].0)
        .count();
    let mut points = intervals
        .iter()
        .flat_map(|(start, end)| [(*start, 1_i8), (*end, -1_i8)])
        .collect::<Vec<_>>();
    points.sort_by_key(|(timestamp, delta)| (*timestamp, *delta));
    let mut active = 0_i64;
    let mut maximum = 0_i64;
    for (_, delta) in points {
        active += i64::from(delta);
        maximum = maximum.max(active);
    }
    let required = horizon != MomentumMicroPredictionHorizonV1::NextTenMinutes;
    let mut value = MomentumMicroTargetOverlapReceiptV1 {
        overlap_version: OVERLAP_VERSION.to_string(),
        horizon,
        event_cadence_ms: horizon.cadence_ms(),
        target_horizon_ms: horizon.horizon_candles() as u64 * TEN_MINUTE_MS,
        event_count: intervals.len(),
        overlap_with_previous_count: previous,
        overlap_with_next_count: next,
        maximum_simultaneous_target_overlap: usize::try_from(maximum.max(0)).unwrap_or(0),
        effective_unique_target_interval_count: intervals.len(),
        zero_overlap_required: required,
        zero_overlap_verified: previous == 0 && next == 0,
        receipt_digest: String::new(),
    };
    value.receipt_digest = overlap_digest(&value);
    validate_overlap(&value)?;
    Ok(value)
}

fn conditional_rate(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator != 0).then_some(numerator as f64 / denominator as f64)
}

fn serial_receipts(
    horizon: MomentumMicroPredictionHorizonV1,
    partition: MomentumReplayPartitionV1,
    observations: &[LabelObservation],
) -> Result<Vec<MomentumMicroSerialDependenceReceiptV1>, String> {
    let labels = observations
        .iter()
        .filter_map(|item| match item.status {
            MomentumMicroLabelStatusV1::Up => Some(true),
            MomentumMicroLabelStatusV1::Down => Some(false),
            MomentumMicroLabelStatusV1::Neutral => None,
        })
        .collect::<Vec<_>>();
    SERIAL_LAGS
        .iter()
        .map(|lag| {
            let pairs = labels
                .iter()
                .zip(labels.iter().skip(*lag))
                .collect::<Vec<_>>();
            let pp = pairs.iter().filter(|(a, b)| **a && **b).count();
            let pn = pairs.iter().filter(|(a, b)| **a && !**b).count();
            let np = pairs.iter().filter(|(a, b)| !**a && **b).count();
            let nn = pairs.iter().filter(|(a, b)| !**a && !**b).count();
            let agreement = pp + nn;
            let mut value = MomentumMicroSerialDependenceReceiptV1 {
                serial_version: SERIAL_VERSION.to_string(),
                horizon,
                partition,
                lag: *lag,
                paired_count: pairs.len(),
                sign_agreement_rate: conditional_rate(agreement, pairs.len()),
                positive_after_positive_count: pp,
                positive_after_negative_count: np,
                negative_after_positive_count: pn,
                negative_after_negative_count: nn,
                positive_after_positive_rate: conditional_rate(pp, pp + pn),
                positive_after_negative_rate: conditional_rate(np, np + nn),
                negative_after_positive_rate: conditional_rate(pn, pp + pn),
                negative_after_negative_rate: conditional_rate(nn, np + nn),
                finite_integrity: true,
                receipt_digest: String::new(),
            };
            value.receipt_digest = serial_digest(&value);
            validate_serial(&value)?;
            Ok(value)
        })
        .collect()
}

fn disposition(
    horizon: MomentumMicroPredictionHorizonV1,
    distributions: &[MomentumMicroTargetDistributionV1],
    temporal: &[MomentumMicroTemporalStabilityV1],
    overlap: &MomentumMicroTargetOverlapReceiptV1,
) -> Result<MomentumMicroHorizonDispositionReceiptV1, String> {
    let scorable = distributions
        .iter()
        .map(|item| item.scorable_count)
        .sum::<usize>();
    let total_positive = distributions
        .iter()
        .map(|item| item.positive_count)
        .sum::<usize>();
    let prevalence = conditional_rate(total_positive, scorable).unwrap_or(0.5);
    let median_abs = distributions
        .iter()
        .map(|item| item.median_absolute_return)
        .sum::<f64>()
        / distributions.len() as f64;
    let maximum_range = temporal
        .iter()
        .map(|item| item.prevalence_range)
        .fold(0.0_f64, f64::max);
    let disposition = if !overlap.zero_overlap_verified
        && horizon != MomentumMicroPredictionHorizonV1::NextTenMinutes
    {
        MomentumMicroHorizonDiagnosticDispositionV1::IntegrityFailure
    } else if scorable < ROLLING_DAY_EVENTS {
        MomentumMicroHorizonDiagnosticDispositionV1::InsufficientScorableSupport
    } else if median_abs < 0.0001 {
        MomentumMicroHorizonDiagnosticDispositionV1::TargetMagnitudeTooSmall
    } else if !(0.35..=0.65).contains(&prevalence) {
        MomentumMicroHorizonDiagnosticDispositionV1::ClassBalanceShifted
    } else if maximum_range > 0.35 {
        MomentumMicroHorizonDiagnosticDispositionV1::ExcessiveTemporalInstability
    } else {
        MomentumMicroHorizonDiagnosticDispositionV1::StableEnoughForFutureScreening
    };
    let mut value = MomentumMicroHorizonDispositionReceiptV1 {
        disposition_version: DISPOSITION_VERSION.to_string(),
        horizon,
        disposition,
        supporting_observation_digests: distributions
            .iter()
            .map(|item| item.distribution_digest.clone())
            .chain(temporal.iter().map(|item| item.temporal_digest.clone()))
            .chain(std::iter::once(overlap.receipt_digest.clone()))
            .collect(),
        model_execution_authorized: false,
        receipt_digest: String::new(),
    };
    value.receipt_digest = disposition_digest(&value);
    let encoded = encode_disposition(&value)?;
    if encoded.is_empty() {
        return Err("micro horizon disposition construction rejected".to_string());
    }
    Ok(value)
}

fn event_partition_limits(
    source_events: &[MomentumQualifiedDiagnosticEventEvidenceV1],
) -> BTreeMap<MomentumReplayPartitionV1, u64> {
    source_events
        .iter()
        .fold(BTreeMap::new(), |mut values, item| {
            values
                .entry(item.partition)
                .and_modify(|current| *current = (*current).max(item.target_timestamp_ms))
                .or_insert(item.target_timestamp_ms);
            values
        })
}

fn label_observations(
    horizon: MomentumMicroPredictionHorizonV1,
    source_events: &[MomentumQualifiedDiagnosticEventEvidenceV1],
    ten_minute_rows: &[MomentumQualifiedReplayCandleEvidenceV1],
) -> Result<Vec<LabelObservation>, String> {
    let closes = ten_minute_rows
        .iter()
        .filter(|row| !row.missing_evidence)
        .map(|row| (row.close_exclusive_timestamp_ms, row.close))
        .collect::<BTreeMap<_, _>>();
    let limits = event_partition_limits(source_events);
    let mut observations = Vec::new();
    let cadence = horizon.cadence_ms();
    let horizon_ms = horizon.horizon_candles() as u64 * TEN_MINUTE_MS;
    for event in source_events {
        if event.partition == MomentumReplayPartitionV1::SealedHoldout
            || (horizon != MomentumMicroPredictionHorizonV1::NextTenMinutes
                && event.prediction_timestamp_ms % cadence != 0)
        {
            continue;
        }
        let target_timestamp_ms = event
            .prediction_timestamp_ms
            .checked_add(horizon_ms)
            .ok_or_else(|| "micro target timestamp overflow".to_string())?;
        if limits
            .get(&event.partition)
            .is_none_or(|limit| target_timestamp_ms > *limit)
        {
            continue;
        }
        let (Some(event_close), Some(target_close)) = (
            closes.get(&event.prediction_timestamp_ms).copied(),
            closes.get(&target_timestamp_ms).copied(),
        ) else {
            continue;
        };
        if !event_close.is_finite()
            || !target_close.is_finite()
            || event_close <= 0.0
            || target_close <= 0.0
        {
            return Err("micro target close integrity rejected".to_string());
        }
        let target_return = target_close / event_close - 1.0;
        if !target_return.is_finite() {
            return Err("micro target return rejected".to_string());
        }
        let status = if target_close > event_close {
            MomentumMicroLabelStatusV1::Up
        } else if target_close < event_close {
            MomentumMicroLabelStatusV1::Down
        } else {
            MomentumMicroLabelStatusV1::Neutral
        };
        if horizon == MomentumMicroPredictionHorizonV1::NextTenMinutes {
            let expected = match event.label_status {
                MomentumQualifiedLabelStatusV1::Up => MomentumMicroLabelStatusV1::Up,
                MomentumQualifiedLabelStatusV1::Down => MomentumMicroLabelStatusV1::Down,
                MomentumQualifiedLabelStatusV1::Neutral => MomentumMicroLabelStatusV1::Neutral,
                MomentumQualifiedLabelStatusV1::Invalid => {
                    return Err("micro source invalid label rejected".to_string());
                }
            };
            if status != expected {
                return Err("micro ten-minute label identity mismatch".to_string());
            }
        }
        observations.push(LabelObservation {
            partition: event.partition,
            event_timestamp_ms: event.prediction_timestamp_ms,
            target_timestamp_ms,
            target_return,
            status,
        });
    }
    observations.sort_by_key(|item| {
        (
            match item.partition {
                MomentumReplayPartitionV1::Development => 0,
                MomentumReplayPartitionV1::Validation => 1,
                MomentumReplayPartitionV1::SealedHoldout => 2,
            },
            item.event_timestamp_ms,
        )
    });
    if observations.is_empty()
        || observations
            .iter()
            .any(|item| item.partition == MomentumReplayPartitionV1::SealedHoldout)
    {
        return Err("micro label observation set rejected".to_string());
    }
    Ok(observations)
}

fn diagnostic_for_horizon(
    registration: &MomentumMicroLabelForensicsRegistrationV1,
    horizon: MomentumMicroPredictionHorizonV1,
    observations: &[LabelObservation],
    counts: &mut (usize, usize),
) -> Result<MomentumMicroHorizonDiagnosticV1, String> {
    let mut distributions = Vec::new();
    let mut temporal = Vec::new();
    let mut serial = Vec::new();
    for partition in [
        MomentumReplayPartitionV1::Development,
        MomentumReplayPartitionV1::Validation,
    ] {
        let partition_observations = observations
            .iter()
            .filter(|item| item.partition == partition)
            .cloned()
            .collect::<Vec<_>>();
        if partition_observations.is_empty() {
            return Err("micro horizon partition support unavailable".to_string());
        }
        let mut plan = MomentumMicroHorizonEventPlanV1 {
            plan_version: EVENT_PLAN_VERSION.to_string(),
            registration_digest: registration.registration_digest.clone(),
            horizon,
            partition,
            event_timestamp_ms: partition_observations
                .iter()
                .map(|item| item.event_timestamp_ms)
                .collect(),
            target_timestamp_ms: partition_observations
                .iter()
                .map(|item| item.target_timestamp_ms)
                .collect(),
            non_overlapping_cadence: horizon != MomentumMicroPredictionHorizonV1::NextTenMinutes,
            holdout_event_count: 0,
            plan_digest: String::new(),
        };
        plan.plan_digest = event_plan_digest(&plan);
        add_counts(
            counts,
            persist_one(
                &format!("event_plans/{}", partition_name(partition)),
                &plan.plan_digest,
                &encode_event_plan(&plan)?,
                |bytes| Ok(decode_event_plan(bytes)?.plan_digest),
            )?,
        );
        let dist = distribution(horizon, partition, &partition_observations)?;
        add_counts(
            counts,
            persist_one(
                "distributions",
                &dist.distribution_digest,
                &encode_distribution(&dist)?,
                |bytes| Ok(decode_distribution(bytes)?.distribution_digest),
            )?,
        );
        distributions.push(dist);
        for item in temporal_stability(horizon, partition, &partition_observations)? {
            add_counts(
                counts,
                persist_one(
                    "temporal_stability",
                    &item.temporal_digest,
                    &encode_temporal(&item)?,
                    |bytes| Ok(decode_temporal(bytes)?.temporal_digest),
                )?,
            );
            temporal.push(item);
        }
        for item in serial_receipts(horizon, partition, &partition_observations)? {
            add_counts(
                counts,
                persist_one(
                    "serial_dependence",
                    &item.receipt_digest,
                    &encode_serial(&item)?,
                    |bytes| Ok(decode_serial(bytes)?.receipt_digest),
                )?,
            );
            serial.push(item);
        }
    }
    let overlap = overlap_receipt(horizon, observations)?;
    add_counts(
        counts,
        persist_one(
            "target_overlap",
            &overlap.receipt_digest,
            &encode_overlap(&overlap)?,
            |bytes| Ok(decode_overlap(bytes)?.receipt_digest),
        )?,
    );
    let disposition = disposition(horizon, &distributions, &temporal, &overlap)?;
    add_counts(
        counts,
        persist_one(
            "horizon_dispositions",
            &disposition.receipt_digest,
            &encode_disposition(&disposition)?,
            |bytes| Ok(decode_disposition(bytes)?.receipt_digest),
        )?,
    );
    let event_count = observations.len();
    let scorable_count = observations
        .iter()
        .filter(|item| item.status != MomentumMicroLabelStatusV1::Neutral)
        .count();
    let mut value = MomentumMicroHorizonDiagnosticV1 {
        horizon,
        event_count,
        scorable_count,
        neutral_count: event_count - scorable_count,
        distributions,
        temporal_stability: temporal,
        overlap,
        serial_dependence: serial,
        disposition,
        diagnostic_digest: String::new(),
    };
    value.diagnostic_digest = horizon_digest(&value);
    validate_horizon(&value)?;
    Ok(value)
}

fn run_inner(
    mode: MomentumMicroLabelForensicsRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> Result<MomentumMicroLabelForensicsReportV1, String> {
    let started = Instant::now();
    validate_momentum_micro_protected_before_state_v1(protected)?;
    if let Some(mut completed) = read_momentum_micro_label_forensics_report_v1()? {
        if completed.protected_before_state_digest != protected.state_digest {
            return Err("micro label protected state changed".to_string());
        }
        completed.run_mode = mode.as_str().to_string();
        completed.artifacts_written = 0;
        completed.duplicate_artifact_count = 0;
        completed.label_computation_count = 0;
        completed.runtime_duration_ms = started.elapsed().as_millis() as u64;
        completed.safety_counters = MomentumMicroLabelForensicsSafetyCountersV1::default();
        completed.report_digest = report_digest(&completed);
        validate_report(&completed)?;
        return Ok(completed);
    }
    let registration = derive_registration(protected)?;
    if let Some(stored) = read_momentum_micro_label_registration_v1()?
        && stored != registration
    {
        return Err("micro label registration conflict".to_string());
    }
    if mode == MomentumMicroLabelForensicsRunModeV1::Status
        || mode == MomentumMicroLabelForensicsRunModeV1::DryRun
    {
        let mut report = empty_report(mode, protected);
        report.status = if read_momentum_micro_label_registration_v1()?.is_some() {
            MomentumMicroLabelForensicsStatusV1::Registered
        } else {
            MomentumMicroLabelForensicsStatusV1::Unregistered
        };
        report.registration_digest = Some(registration.registration_digest);
        report.source_replay_digest = Some(registration.source_replay_digest);
        report.source_diagnostic_digest = Some(registration.source_diagnostic_digest);
        report.runtime_duration_ms = started.elapsed().as_millis() as u64;
        report.report_digest = report_digest(&report);
        validate_report(&report)?;
        return Ok(report);
    }
    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_one(
            "registrations",
            &registration.registration_digest,
            &encode_registration(&registration)?,
            |bytes| Ok(decode_registration(bytes)?.registration_digest),
        )?,
    );
    let reopened = read_momentum_micro_label_registration_v1()?
        .ok_or_else(|| "micro label registration reopen failed".to_string())?;
    if reopened != registration {
        return Err("micro label registration reopen mismatch".to_string());
    }

    // Target-return access starts only after the persisted registration has reopened.
    let header = load_momentum_qualified_diagnostic_source_header_v1()?;
    if header.replay_journal_digest != registration.source_replay_digest {
        return Err("micro label replay source changed".to_string());
    }
    let source = load_momentum_qualified_diagnostic_source_v1(&header)?;
    if source
        .events
        .iter()
        .any(|event| event.partition == MomentumReplayPartitionV1::SealedHoldout)
    {
        return Err("micro label holdout event source rejected".to_string());
    }
    let evidence = load_momentum_qualified_six_evidence_v1()?;
    if evidence.prior_holdout.labels_opened
        || evidence.prior_holdout.metrics_computed
        || evidence.prior_holdout.aggregate_comparison_opened
    {
        return Err("micro label sealed holdout opened".to_string());
    }
    let ten_minute_rows = evidence
        .views
        .get(&MomentumHistoricalTimeframeV1::Minute10)
        .ok_or_else(|| "micro label 10m source unavailable".to_string())?;
    let mut horizons = Vec::new();
    let mut target_return_reads = 0;
    for horizon in MomentumMicroPredictionHorizonV1::ORDERED {
        let observations = label_observations(horizon, &source.events, ten_minute_rows)?;
        target_return_reads += observations.len();
        horizons.push(diagnostic_for_horizon(
            &registration,
            horizon,
            &observations,
            &mut counts,
        )?);
    }
    let mut journal = MomentumMicroLabelForensicsJournalV1 {
        journal_version: JOURNAL_VERSION.to_string(),
        registration_digest: registration.registration_digest.clone(),
        horizon_diagnostic_digests: horizons
            .iter()
            .map(|item| item.diagnostic_digest.clone())
            .collect(),
        holdout_label_reads: 0,
        holdout_prediction_reads: 0,
        holdout_metric_reads: 0,
        model_fits: 0,
        deterministic: true,
        journal_digest: String::new(),
    };
    journal.journal_digest = journal_digest(&journal);
    add_counts(
        &mut counts,
        persist_one(
            "research_journals",
            &journal.journal_digest,
            &encode_journal(&journal)?,
            |bytes| Ok(decode_journal(bytes)?.journal_digest),
        )?,
    );
    let mut report = empty_report(mode, protected);
    report.status = MomentumMicroLabelForensicsStatusV1::Complete;
    report.registration_digest = Some(registration.registration_digest.clone());
    report.source_replay_digest = Some(registration.source_replay_digest.clone());
    report.source_diagnostic_digest = Some(registration.source_diagnostic_digest.clone());
    report.horizons = horizons;
    report
        .safety_counters
        .registration_writes_before_target_reads = 1;
    report.safety_counters.target_return_reads = target_return_reads;
    report.journal_digest = Some(journal.journal_digest);
    report.artifacts_written = counts.0 + 1;
    report.duplicate_artifact_count = counts.1;
    report.label_computation_count = target_return_reads;
    report.runtime_duration_ms = started.elapsed().as_millis() as u64;
    report.report_digest = report_digest(&report);
    validate_report(&report)?;
    add_counts(
        &mut counts,
        persist_one(
            "final_reports",
            &report.report_digest,
            &encode_report(&report)?,
            |bytes| Ok(decode_report(bytes)?.report_digest),
        )?,
    );
    if counts.0 != report.artifacts_written {
        return Err("micro label artifact accounting mismatch".to_string());
    }
    let reopened = read_momentum_micro_label_forensics_report_v1()?
        .ok_or_else(|| "micro label final report reopen failed".to_string())?;
    if reopened != report {
        return Err("micro label final report mismatch".to_string());
    }
    Ok(report)
}

pub fn run_momentum_micro_label_forensics_v1(
    mode: MomentumMicroLabelForensicsRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> Result<MomentumMicroLabelForensicsReportV1, String> {
    match run_inner(mode, protected) {
        Ok(report) => Ok(report),
        Err(error)
            if error.contains("artifact")
                || error.contains("conflict")
                || error.contains("mismatch")
                || error.contains("source changed") =>
        {
            let mut report = empty_report(mode, protected);
            report.status = MomentumMicroLabelForensicsStatusV1::IntegrityFailure;
            report.report_digest = report_digest(&report);
            validate_report(&report)?;
            Ok(report)
        }
        Err(error) => Err(error),
    }
}

pub fn format_momentum_micro_label_forensics_text_v1(
    report: &MomentumMicroLabelForensicsReportV1,
) -> String {
    use std::fmt::Write as _;
    let mut output = String::new();
    let _ = writeln!(output, "status={:?}", report.status);
    let _ = writeln!(output, "evidence_class={:?}", report.evidence_class);
    let _ = writeln!(
        output,
        "registration_digest={}",
        report.registration_digest.as_deref().unwrap_or("absent")
    );
    for horizon in &report.horizons {
        let _ = writeln!(output, "horizon={:?}", horizon.horizon);
        let _ = writeln!(output, "event_count={}", horizon.event_count);
        let _ = writeln!(output, "scorable_count={}", horizon.scorable_count);
        let _ = writeln!(output, "neutral_count={}", horizon.neutral_count);
        let _ = writeln!(output, "disposition={:?}", horizon.disposition.disposition);
        let _ = writeln!(
            output,
            "overlap_previous={}",
            horizon.overlap.overlap_with_previous_count
        );
    }
    let _ = writeln!(
        output,
        "holdout_label_reads={}",
        report.safety_counters.holdout_label_reads
    );
    let _ = writeln!(output, "model_fits={}", report.safety_counters.model_fits);
    let _ = writeln!(output, "report_digest={}", report.report_digest);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

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
        value.state_digest = momentum_micro_protected_before_state_digest_v1(&value);
        value
    }

    fn observations(horizon: MomentumMicroPredictionHorizonV1) -> Vec<LabelObservation> {
        (0..200)
            .map(|index| LabelObservation {
                partition: MomentumReplayPartitionV1::Development,
                event_timestamp_ms: index as u64 * horizon.cadence_ms(),
                target_timestamp_ms: index as u64 * horizon.cadence_ms()
                    + horizon.horizon_candles() as u64 * TEN_MINUTE_MS,
                target_return: if index % 2 == 0 { 0.01 } else { -0.01 },
                status: if index % 2 == 0 {
                    MomentumMicroLabelStatusV1::Up
                } else {
                    MomentumMicroLabelStatusV1::Down
                },
            })
            .collect()
    }

    #[test]
    fn sprint101_01_protected_live_state_requires_completed_two_event_pause() {
        assert!(validate_momentum_micro_protected_before_state_v1(&protected_fixture()).is_ok());
        let mut invalid = protected_fixture();
        invalid.completed_event_count = 3;
        invalid.state_digest = momentum_micro_protected_before_state_digest_v1(&invalid);
        assert!(validate_momentum_micro_protected_before_state_v1(&invalid).is_err());
    }

    #[test]
    fn sprint101_02_registration_round_trip_preserves_prohibitions() {
        let mut value = MomentumMicroLabelForensicsRegistrationV1 {
            registration_version: REGISTRATION_VERSION.into(),
            source_replay_digest: "replay".into(),
            source_diagnostic_digest: "diagnostic".into(),
            protected_before_state_digest: protected_fixture().state_digest,
            included_partitions: vec![
                MomentumReplayPartitionV1::Development,
                MomentumReplayPartitionV1::Validation,
            ],
            candidate_horizons: MomentumMicroPredictionHorizonV1::ORDERED.to_vec(),
            magnitude_distribution_policy_digest: "magnitude".into(),
            prevalence_policy_digest: "prevalence".into(),
            temporal_stability_policy_digest: "temporal".into(),
            target_overlap_policy_digest: "overlap".into(),
            serial_dependence_policy_digest: "serial".into(),
            disposition_policy_digest: "disposition".into(),
            holdout_access_forbidden: true,
            model_training_forbidden: true,
            result_selected_threshold_forbidden: true,
            post_result: true,
            confirmatory_claim_allowed: false,
            new_model_execution_allowed: false,
            holdout_execution_allowed: false,
            live_authority_allowed: false,
            governance_authority_allowed: false,
            trading_authority_allowed: false,
            registration_digest: String::new(),
        };
        value.registration_digest = registration_digest(&value);
        assert_eq!(
            decode_registration(&encode_registration(&value).unwrap()).unwrap(),
            value
        );
    }

    #[test]
    fn sprint101_03_horizon_semantics_are_fixed() {
        assert_eq!(
            MomentumMicroPredictionHorizonV1::NextTenMinutes.horizon_candles(),
            1
        );
        assert_eq!(
            MomentumMicroPredictionHorizonV1::NextThirtyMinutes.horizon_candles(),
            3
        );
        assert_eq!(
            MomentumMicroPredictionHorizonV1::NextSixtyMinutes.horizon_candles(),
            6
        );
    }

    #[test]
    fn sprint101_04_t30_and_t60_diagnostic_intervals_do_not_overlap() {
        for horizon in [
            MomentumMicroPredictionHorizonV1::NextThirtyMinutes,
            MomentumMicroPredictionHorizonV1::NextSixtyMinutes,
        ] {
            let receipt = overlap_receipt(horizon, &observations(horizon)).unwrap();
            assert_eq!(receipt.overlap_with_previous_count, 0);
            assert_eq!(receipt.overlap_with_next_count, 0);
            assert!(receipt.zero_overlap_verified);
        }
    }

    #[test]
    fn sprint101_05_neutral_returns_remain_neutral_and_finite() {
        let values = vec![LabelObservation {
            partition: MomentumReplayPartitionV1::Development,
            event_timestamp_ms: 0,
            target_timestamp_ms: TEN_MINUTE_MS,
            target_return: 0.0,
            status: MomentumMicroLabelStatusV1::Neutral,
        }];
        let result = distribution(
            MomentumMicroPredictionHorizonV1::NextTenMinutes,
            MomentumReplayPartitionV1::Development,
            &values,
        )
        .unwrap();
        assert_eq!(result.neutral_count, 1);
        assert_eq!(result.scorable_count, 0);
        assert!(result.finite_value_proof);
    }

    #[test]
    fn sprint101_06_temporal_and_serial_diagnostics_are_deterministic() {
        let values = observations(MomentumMicroPredictionHorizonV1::NextTenMinutes);
        let first = temporal_stability(
            MomentumMicroPredictionHorizonV1::NextTenMinutes,
            MomentumReplayPartitionV1::Development,
            &values,
        )
        .unwrap();
        let second = temporal_stability(
            MomentumMicroPredictionHorizonV1::NextTenMinutes,
            MomentumReplayPartitionV1::Development,
            &values,
        )
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(
            serial_receipts(
                MomentumMicroPredictionHorizonV1::NextTenMinutes,
                MomentumReplayPartitionV1::Development,
                &values,
            )
            .unwrap(),
            serial_receipts(
                MomentumMicroPredictionHorizonV1::NextTenMinutes,
                MomentumReplayPartitionV1::Development,
                &values,
            )
            .unwrap()
        );
    }

    #[test]
    fn sprint101_07_malformed_protobuf_rejects() {
        let mut value = MomentumMicroLabelForensicsRegistrationV1 {
            registration_version: REGISTRATION_VERSION.into(),
            source_replay_digest: "replay".into(),
            source_diagnostic_digest: "diagnostic".into(),
            protected_before_state_digest: protected_fixture().state_digest,
            included_partitions: vec![
                MomentumReplayPartitionV1::Development,
                MomentumReplayPartitionV1::Validation,
            ],
            candidate_horizons: MomentumMicroPredictionHorizonV1::ORDERED.to_vec(),
            magnitude_distribution_policy_digest: "magnitude".into(),
            prevalence_policy_digest: "prevalence".into(),
            temporal_stability_policy_digest: "temporal".into(),
            target_overlap_policy_digest: "overlap".into(),
            serial_dependence_policy_digest: "serial".into(),
            disposition_policy_digest: "disposition".into(),
            holdout_access_forbidden: true,
            model_training_forbidden: true,
            result_selected_threshold_forbidden: true,
            post_result: true,
            confirmatory_claim_allowed: false,
            new_model_execution_allowed: false,
            holdout_execution_allowed: false,
            live_authority_allowed: false,
            governance_authority_allowed: false,
            trading_authority_allowed: false,
            registration_digest: String::new(),
        };
        value.registration_digest = registration_digest(&value);
        let mut bytes = encode_registration(&value).unwrap();
        bytes.truncate(bytes.len() / 2);
        assert!(decode_registration(&bytes).is_err());
    }

    #[test]
    fn sprint101_08_public_text_does_not_expose_event_values() {
        let mut report = empty_report(
            MomentumMicroLabelForensicsRunModeV1::Status,
            &protected_fixture(),
        );
        report.report_digest = report_digest(&report);
        let text = format_momentum_micro_label_forensics_text_v1(&report);
        assert!(!text.contains("target_return"));
        assert!(!text.contains("event_timestamp_ms"));
        assert!(!text.contains("local path"));
    }

    #[test]
    fn sprint101_09_only_development_and_validation_are_registered() {
        let partitions = [
            MomentumReplayPartitionV1::Development,
            MomentumReplayPartitionV1::Validation,
        ];
        assert!(!partitions.contains(&MomentumReplayPartitionV1::SealedHoldout));
        assert_eq!(partitions.len(), 2);
    }

    #[test]
    fn sprint101_10_target_distribution_round_trip_keeps_finite_aggregates() {
        let value = distribution(
            MomentumMicroPredictionHorizonV1::NextTenMinutes,
            MomentumReplayPartitionV1::Development,
            &observations(MomentumMicroPredictionHorizonV1::NextTenMinutes),
        )
        .unwrap();
        let reopened = decode_distribution(&encode_distribution(&value).unwrap()).unwrap();
        assert_eq!(reopened, value);
        assert!(reopened.finite_value_proof);
    }

    #[test]
    fn sprint101_11_event_plan_round_trip_preserves_nonoverlap_policy() {
        let horizon = MomentumMicroPredictionHorizonV1::NextThirtyMinutes;
        let values = observations(horizon);
        let mut plan = MomentumMicroHorizonEventPlanV1 {
            plan_version: EVENT_PLAN_VERSION.into(),
            registration_digest: "registration".into(),
            horizon,
            partition: MomentumReplayPartitionV1::Development,
            event_timestamp_ms: values
                .iter()
                .map(|value| value.event_timestamp_ms)
                .collect(),
            target_timestamp_ms: values
                .iter()
                .map(|value| value.target_timestamp_ms)
                .collect(),
            non_overlapping_cadence: true,
            holdout_event_count: 0,
            plan_digest: String::new(),
        };
        plan.plan_digest = event_plan_digest(&plan);
        assert_eq!(
            decode_event_plan(&encode_event_plan(&plan).unwrap()).unwrap(),
            plan
        );
    }

    #[test]
    fn sprint101_12_all_three_diagnostic_horizons_are_distinct() {
        assert_eq!(
            MomentumMicroPredictionHorizonV1::ORDERED
                .into_iter()
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
    }

    #[test]
    fn sprint101_13_label_diagnostics_have_zero_model_and_authority_counters() {
        let counters = MomentumMicroLabelForensicsSafetyCountersV1::default();
        assert!(zero_forbidden_counters(&counters));
        assert_eq!(counters.model_fits, 0);
        assert_eq!(counters.live_network_requests, 0);
    }

    #[test]
    fn sprint101_14_report_rejects_duplicate_horizon_identity() {
        let mut report = empty_report(
            MomentumMicroLabelForensicsRunModeV1::Status,
            &protected_fixture(),
        );
        report.status = MomentumMicroLabelForensicsStatusV1::Complete;
        report.horizons = Vec::new();
        report.report_digest = report_digest(&report);
        assert!(validate_report(&report).is_err());
    }

    #[test]
    fn sprint101_15_temporal_groupings_are_complete_and_deterministic() {
        let values = temporal_stability(
            MomentumMicroPredictionHorizonV1::NextTenMinutes,
            MomentumReplayPartitionV1::Development,
            &observations(MomentumMicroPredictionHorizonV1::NextTenMinutes),
        )
        .unwrap();
        assert_eq!(
            values
                .iter()
                .map(|value| value.grouping.as_str())
                .collect::<Vec<_>>(),
            [
                "UtcDay",
                "UtcWeek",
                "UtcMonth",
                "Rolling144Events",
                "Rolling1008Events",
            ]
        );
    }

    #[test]
    fn sprint101_16_serial_receipts_use_all_fixed_lags() {
        let values = serial_receipts(
            MomentumMicroPredictionHorizonV1::NextTenMinutes,
            MomentumReplayPartitionV1::Development,
            &observations(MomentumMicroPredictionHorizonV1::NextTenMinutes),
        )
        .unwrap();
        assert_eq!(
            values.iter().map(|value| value.lag).collect::<Vec<_>>(),
            SERIAL_LAGS
        );
    }

    #[test]
    fn sprint101_17_ten_minute_overlap_is_measured_without_becoming_a_gate() {
        let receipt = overlap_receipt(
            MomentumMicroPredictionHorizonV1::NextTenMinutes,
            &observations(MomentumMicroPredictionHorizonV1::NextTenMinutes),
        )
        .unwrap();
        assert!(!receipt.zero_overlap_required);
        assert_eq!(receipt.event_cadence_ms, TEN_MINUTE_MS);
    }

    #[test]
    fn sprint101_18_horizon_disposition_never_authorizes_execution() {
        let horizon = MomentumMicroPredictionHorizonV1::NextThirtyMinutes;
        let values = observations(horizon);
        let distribution =
            distribution(horizon, MomentumReplayPartitionV1::Development, &values).unwrap();
        let temporal =
            temporal_stability(horizon, MomentumReplayPartitionV1::Development, &values).unwrap();
        let overlap = overlap_receipt(horizon, &values).unwrap();
        let receipt = disposition(horizon, &[distribution], &temporal, &overlap).unwrap();
        assert!(!receipt.model_execution_authorized);
    }

    #[test]
    fn sprint101_19_protected_parameter_and_normalizer_identity_changes_reject() {
        let baseline = protected_fixture();
        let mut changed = baseline.clone();
        changed.live_parameter_digests[0] = "changed".into();
        assert_ne!(
            momentum_micro_protected_before_state_digest_v1(&changed),
            baseline.state_digest
        );
        changed.state_digest = baseline.state_digest;
        assert!(validate_momentum_micro_protected_before_state_v1(&changed).is_err());
    }
}
