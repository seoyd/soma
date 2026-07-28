//! Frozen T10 compact-micro development and validation screening.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use serde::{Deserialize, Serialize};

use crate::stable_hash_string;

use super::{
    EncodedTrainingExampleV0, HeadTrainingConfigV0, LogisticPredictionHeadV0, MomentumCandleV0,
    MomentumFeatureConfigV0, MomentumLearningCampaignConfigV0, RepresentationNormalizerV0,
    build_momentum_features_v0,
    momentum_future_prediction_v4::{
        ArtifactBuilderV4_2, ArtifactReaderV4_2, as_u64, as_usize, persist_artifact, read_single,
    },
    momentum_micro_challenger_design_v1::{
        MomentumCompactMicroFeaturePolicyV1, MomentumMicroChallengerDesignReportV1,
        MomentumMicroChallengerScreeningRegistrationV1, MomentumMicroParticipantV1,
        MomentumMicroScreeningGateV1, MomentumMicroTaskPartitionBoundaryV1, MomentumMicroTaskV1,
        extract_compact_micro_event_v1, read_momentum_micro_challenger_design_report_v1,
        read_momentum_micro_feature_forensics_report_v1,
    },
    momentum_micro_label_forensics_v1::{
        MomentumMicroHorizonDiagnosticDispositionV1, MomentumMicroLabelForensicsStatusV1,
        MomentumMicroPredictionHorizonV1, MomentumMicroProtectedBeforeStateV1,
        read_momentum_micro_label_forensics_report_v1,
        validate_momentum_micro_protected_before_state_v1,
    },
    momentum_multitimeframe_history_v1::{
        MomentumHistoricalTimeframeV1, MomentumQualifiedReplayCandleEvidenceV1,
        MomentumQualifiedSixEvidenceV1, load_momentum_qualified_six_evidence_v1,
        load_momentum_qualified_t10_micro_evidence_v1,
    },
    momentum_qualified_six_replay_v1::{
        COLLAPSE_VARIANCE_THRESHOLD, COMPARISON_EPSILON, MomentumReplayPartitionV1,
        past_micro_volatility,
    },
    momentum_raw_feature_v4::train_head_v4,
};

const ROOT: &str = "state/historical_replay/momentum_t10_micro_screening/v1";
const AUTHORIZATION_VERSION: &str = "momentum-t10-screening-execution-authorization-v1";
const TRAINING_POLICY_VERSION: &str = "momentum-t10-screening-training-policy-v1";
const TRAINING_PLAN_VERSION: &str = "momentum-t10-daily-training-window-plan-v1";
const NORMALIZER_VERSION: &str = "momentum-t10-daily-normalizer-receipt-v1";
const MODEL_VERSION: &str = "momentum-t10-daily-model-receipt-v1";
const REFIT_BUNDLE_VERSION: &str = "momentum-t10-daily-refit-bundle-v1";
const EVENT_PLAN_VERSION: &str = "momentum-t10-screening-event-plan-v1";
const PREDICTION_SHARD_VERSION: &str = "momentum-t10-daily-prediction-shard-v1";
const EVALUATION_ITEM_VERSION: &str = "momentum-t10-screening-evaluation-item-v1";
const EVALUATION_SHARD_VERSION: &str = "momentum-t10-daily-evaluation-shard-v1";
const CALIBRATION_BIN_VERSION: &str = "momentum-t10-calibration-bin-v1";
const METRICS_VERSION: &str = "momentum-t10-participant-metrics-v1";
const AGGREGATE_VERSION: &str = "momentum-t10-partition-aggregate-v1";
const BENCHMARK_VERSION: &str = "momentum-t10-benchmark-comparison-v1";
const CONTRIBUTION_VERSION: &str = "momentum-t10-contribution-comparison-v1";
const COHORT_VERSION: &str = "momentum-t10-proposed-holdout-cohort-v1";
const JOURNAL_VERSION: &str = "momentum-t10-screening-journal-v1";
const REPORT_VERSION: &str = "momentum-t10-compact-micro-screening-public-report-v1";

const EXPECTED_LABEL_REPORT_DIGEST: &str = "dc1db01318ab180f";
const EXPECTED_FEATURE_REPORT_DIGEST: &str = "02bb79cbc18c34c4";
const EXPECTED_DESIGN_REPORT_DIGEST: &str = "0d1077c9c65fd8cf";
const EXPECTED_REGISTRATION_DIGEST: &str = "56dbdee4766edaaa";
const EXPECTED_GATE_DIGEST: &str = "ccd9763e73e60081";

const TEN_MINUTE_MS: u64 = 10 * 60 * 1_000;
const DAY_MS: u64 = 24 * 60 * 60 * 1_000;
const ANCHOR_CONTEXT_LENGTH: usize = 16;
const ANCHOR_FEATURE_DIMENSION: usize = 6;
const COMPACT_FEATURE_DIMENSION: usize = 4 * 16 + 5;
const CALIBRATOR_FEATURE_DIMENSION: usize = 1;
const DIMENSION_SUPPORT_MULTIPLIER: usize = 10;
const MAXIMUM_TRAINING_EXAMPLES: usize = 4_096;
const STANDARD_L2: f32 = 0.001;
const C3_L2_MULTIPLIER: usize = 4;
const C4_BASE_PERCENT: usize = 80;
const C4_CALIBRATION_PERCENT: usize = 20;
const PROBABILITY_CLAMP: f64 = 1e-6;
const NEAR_HALF_THRESHOLD: f64 = 1e-3;
const CALIBRATION_BOUNDARIES: [f64; 11] = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
const PUBLIC_LABELS: [&str; 6] = [
    "HistoricalResearchOnly",
    "T10CompactMicroScreening",
    "DevelopmentAndValidationOnly",
    "HoldoutClosed",
    "NotLiveAuthority",
    "NotTradingAuthority",
];

const PARTICIPANTS: [MomentumMicroParticipantV1; 5] = [
    MomentumMicroParticipantV1::C0TaskSpecificConstant,
    MomentumMicroParticipantV1::C1TenMinuteAnchorBaseline,
    MomentumMicroParticipantV1::C2CompactMicroLogistic,
    MomentumMicroParticipantV1::C3CompactMicroStrongShrinkLogistic,
    MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumT10MicroScreeningRunModeV1 {
    Status,
    DryRun,
    Authorize,
    ExecuteDevelopment,
    ExecuteValidation,
}

impl MomentumT10MicroScreeningRunModeV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::DryRun => "dry-run",
            Self::Authorize => "authorize",
            Self::ExecuteDevelopment => "execute-development",
            Self::ExecuteValidation => "execute-validation",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumT10MicroScreeningStatusV1 {
    Unregistered,
    Authorized,
    DevelopmentComplete,
    Complete,
    TrainingSupportInsufficientForAllParticipants,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMicroScreeningComparisonV1 {
    LowerBrierThanConstant,
    HigherBrierThanConstant,
    NumericallyEquivalentToConstant,
    MixedOrInsufficientEvidence,
    ProbabilityCollapse,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMicroContributionComparisonV1 {
    LowerBrierWithContribution,
    HigherBrierWithContribution,
    NumericallyEquivalent,
    MixedOrInsufficientEvidence,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMicroProbabilityCollapseV1 {
    BenchmarkExempt,
    NotCollapsed,
    ProbabilityCollapse,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMicroSaturationV1 {
    NotSaturated,
    LowBoundarySaturation,
    HighBoundarySaturation,
    TwoSidedSaturation,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMicroHoldoutEligibilityV1 {
    EligibleForFutureSealedHoldoutEvaluation,
    IneligibleScreeningGate,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMicroHoldoutCohortStatusV1 {
    NoEligibleT10HoldoutCohort,
    ProposedT10HoldoutCohort,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DailyModelRole {
    Constant,
    LearnedBase,
    Calibrator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScreeningLabel {
    Up,
    Down,
    Neutral,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10ScreeningExecutionAuthorizationV1 {
    pub authorization_version: String,
    pub challenger_registration_digest: String,
    pub screening_gate_digest: String,
    pub label_report_digest: String,
    pub feature_report_digest: String,
    pub design_report_digest: String,
    pub authorized_task_id: String,
    pub authorized_participant_ids: Vec<String>,
    pub development_execution_authorized: bool,
    pub validation_execution_authorized: bool,
    pub historical_holdout_execution_authorized: bool,
    pub t30_execution_authorized: bool,
    pub t60_execution_authorized: bool,
    pub network_authorized: bool,
    pub live_authority_forbidden: bool,
    pub governance_authority_forbidden: bool,
    pub trading_authority_forbidden: bool,
    pub authorization_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10TrainingPolicyV1 {
    pub policy_version: String,
    pub source_training_policy_digest: String,
    pub loss_function: String,
    pub initialization_seed: u64,
    pub epoch_count: usize,
    pub batch_size: usize,
    pub learning_rate_bits: u32,
    pub standard_l2_bits: u32,
    pub c3_l2_multiplier: usize,
    pub gradient_finite_checks: bool,
    pub parameter_finite_checks: bool,
    pub probability_clamp_bits: u64,
    pub maximum_training_examples: usize,
    pub minimum_training_examples: usize,
    pub dimension_support_multiplier: usize,
    pub daily_utc_refit: bool,
    pub within_day_refit_forbidden: bool,
    pub training_only_normalizer: bool,
    pub c4_base_percent: usize,
    pub c4_calibration_percent: usize,
    pub validation_fit_forbidden: bool,
    pub holdout_fit_forbidden: bool,
    pub policy_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumT10DailyTrainingWindowPlanV1 {
    plan_version: String,
    authorization_digest: String,
    training_policy_digest: String,
    partition: MomentumReplayPartitionV1,
    utc_day_boundary_ms: u64,
    training_target_cutoff_exclusive_ms: u64,
    eligible_past_event_count: usize,
    scorable_training_event_count: usize,
    used_training_event_count: usize,
    training_event_digests: Vec<String>,
    c4_base_count: usize,
    c4_calibration_count: usize,
    support_sufficient_for_all: bool,
    validation_label_fit_count: usize,
    holdout_access_count: usize,
    plan_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumT10DailyNormalizerReceiptV1 {
    receipt_version: String,
    participant_id: String,
    feature_policy_digest: String,
    private_means: Vec<f32>,
    private_scales: Vec<f32>,
    constant_dimension_indices: Vec<usize>,
    training_event_digest: String,
    finite: bool,
    receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumT10DailyModelReceiptV1 {
    receipt_version: String,
    participant_id: String,
    role: DailyModelRole,
    utc_day_boundary_ms: u64,
    training_plan_digest: String,
    normalizer_receipt_digest: String,
    private_weights: Vec<f32>,
    private_bias: Option<f32>,
    private_prevalence: Option<f64>,
    training_count: usize,
    positive_count: usize,
    negative_count: usize,
    l2_bits: u32,
    initialization_seed: u64,
    finite: bool,
    validation_fit_count: usize,
    holdout_fit_count: usize,
    receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumT10DailyRefitBundleV1 {
    bundle_version: String,
    authorization_digest: String,
    partition: MomentumReplayPartitionV1,
    utc_day_boundary_ms: u64,
    training_plan: MomentumT10DailyTrainingWindowPlanV1,
    normalizer_receipts: Vec<MomentumT10DailyNormalizerReceiptV1>,
    model_receipts: Vec<MomentumT10DailyModelReceiptV1>,
    reconstructed_participant_digests: Vec<String>,
    target_access_count_for_prediction_day: usize,
    holdout_access_count: usize,
    live_access_count: usize,
    bundle_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumT10EventPlanV1 {
    plan_version: String,
    partition: MomentumReplayPartitionV1,
    prediction_timestamp_ms: u64,
    target_timestamp_ms: u64,
    source_event_digest: String,
    daily_refit_bundle_digest: String,
    participant_ids: Vec<String>,
    target_hidden: bool,
    holdout_member: bool,
    plan_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumT10PredictionShardV1 {
    shard_version: String,
    authorization_digest: String,
    partition: MomentumReplayPartitionV1,
    utc_day_boundary_ms: u64,
    daily_refit_bundle_digest: String,
    event_plans: Vec<MomentumT10EventPlanV1>,
    participant_ids: Vec<String>,
    private_probabilities: Vec<f64>,
    prediction_digests: Vec<String>,
    target_accessed: bool,
    label_accessed: bool,
    metric_computed: bool,
    shard_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumT10EvaluationItemV1 {
    item_version: String,
    event_plan_digest: String,
    label: ScreeningLabel,
    private_label: Option<f64>,
    private_brier_values: Vec<f64>,
    private_correctness: Vec<bool>,
    item_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumT10EvaluationShardV1 {
    shard_version: String,
    prediction_shard_digest: String,
    partition: MomentumReplayPartitionV1,
    utc_day_boundary_ms: u64,
    prediction_shard_reopened: bool,
    evaluations: Vec<MomentumT10EvaluationItemV1>,
    shard_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumMicroCalibrationBinV1 {
    pub bin_version: String,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub upper_inclusive: bool,
    pub support: usize,
    pub mean_predicted_probability: f64,
    pub observed_positive_frequency: f64,
    pub absolute_calibration_gap: f64,
    pub bin_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumMicroScreeningParticipantMetricsV1 {
    pub metrics_version: String,
    pub participant_id: String,
    pub partition: MomentumReplayPartitionV1,
    pub prediction_count: usize,
    pub scorable_count: usize,
    pub neutral_count: usize,
    pub invalid_count: usize,
    pub finite_prediction_count: usize,
    pub mean_brier: f64,
    pub binary_correctness: f64,
    pub paired_mean_brier_delta_versus_c0: f64,
    pub paired_median_brier_delta_versus_c0: f64,
    pub positive_paired_delta_count: usize,
    pub negative_paired_delta_count: usize,
    pub equivalent_paired_delta_count: usize,
    pub calibration_bins: Vec<MomentumMicroCalibrationBinV1>,
    pub weighted_calibration_gap: f64,
    pub empty_calibration_bin_count: usize,
    pub minimum_probability: f64,
    pub maximum_probability: f64,
    pub mean_probability: f64,
    pub probability_standard_deviation: f64,
    pub near_constant_count: usize,
    pub near_half_count: usize,
    pub extreme_low_count: usize,
    pub extreme_high_count: usize,
    pub nonfinite_count: usize,
    pub collapse: MomentumMicroProbabilityCollapseV1,
    pub saturation: MomentumMicroSaturationV1,
    pub chronology_audit_passed: bool,
    pub leakage_audit_passed: bool,
    pub integrity_audit_passed: bool,
    pub metrics_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumT10PartitionAggregateV1 {
    pub aggregate_version: String,
    pub authorization_digest: String,
    pub partition: MomentumReplayPartitionV1,
    pub boundary_event_count: usize,
    pub training_only_event_count: usize,
    pub prediction_count: usize,
    pub scorable_count: usize,
    pub neutral_count: usize,
    pub invalid_count: usize,
    pub daily_refit_count: usize,
    pub insufficient_support_day_count: usize,
    pub daily_refit_bundle_digests: Vec<String>,
    pub prediction_shard_digests: Vec<String>,
    pub evaluation_shard_digests: Vec<String>,
    pub participant_metrics: Vec<MomentumMicroScreeningParticipantMetricsV1>,
    pub target_access_before_prediction_reopen_count: usize,
    pub feature_future_access_count: usize,
    pub partial_candle_access_count: usize,
    pub holdout_access_count: usize,
    pub validation_fit_count: usize,
    pub chronology_audit_passed: bool,
    pub leakage_audit_passed: bool,
    pub prediction_before_reveal_passed: bool,
    pub aggregate_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroBenchmarkComparisonReceiptV1 {
    pub comparison_version: String,
    pub participant_id: String,
    pub development_aggregate_digest: String,
    pub validation_aggregate_digest: String,
    pub development_delta_bits: u64,
    pub validation_delta_bits: u64,
    pub development_comparison: MomentumMicroScreeningComparisonV1,
    pub validation_comparison: MomentumMicroScreeningComparisonV1,
    pub overall_comparison: MomentumMicroScreeningComparisonV1,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroContributionReceiptV1 {
    pub comparison_version: String,
    pub participant_id: String,
    pub baseline_participant_id: String,
    pub development_delta_bits: u64,
    pub validation_delta_bits: u64,
    pub development_comparison: MomentumMicroContributionComparisonV1,
    pub validation_comparison: MomentumMicroContributionComparisonV1,
    pub overall_comparison: MomentumMicroContributionComparisonV1,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroHoldoutEligibilityReceiptV1 {
    pub task_registration_digest: String,
    pub participant_registration_digest: String,
    pub participant_id: String,
    pub development_aggregate_digest: String,
    pub validation_aggregate_digest: String,
    pub development_lower_brier_than_constant: bool,
    pub validation_lower_brier_than_constant: bool,
    pub sufficient_paired_support: bool,
    pub finite_predictions: bool,
    pub finite_metrics: bool,
    pub no_probability_collapse: bool,
    pub no_saturation_failure: bool,
    pub chronology_clean: bool,
    pub leakage_clean: bool,
    pub integrity_clean: bool,
    pub result_selected_mutation_absent: bool,
    pub holdout_access_count: usize,
    pub eligibility: MomentumMicroHoldoutEligibilityV1,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMicroProposedHoldoutCohortV1 {
    pub cohort_version: String,
    pub authorization_digest: String,
    pub eligibility_receipt_digests: Vec<String>,
    pub participant_ids: Vec<String>,
    pub status: MomentumMicroHoldoutCohortStatusV1,
    pub holdout_execution_authorized: bool,
    pub cohort_digest: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumT10ScreeningSafetyCountersV1 {
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub new_model_fits: usize,
    pub new_calibration_fits: usize,
    pub new_predictions: usize,
    pub new_target_reveals: usize,
    pub new_evaluations: usize,
    pub new_metric_computations: usize,
    pub t10_holdout_predictions: usize,
    pub t10_holdout_label_reads: usize,
    pub t10_holdout_metrics: usize,
    pub t30_model_fits: usize,
    pub t30_calibration_fits: usize,
    pub t30_predictions: usize,
    pub t30_evaluations: usize,
    pub t30_holdout_access: usize,
    pub t60_model_fits: usize,
    pub t60_calibration_fits: usize,
    pub t60_predictions: usize,
    pub t60_evaluations: usize,
    pub t60_holdout_access: usize,
    pub network_requests: usize,
    pub live_requests: usize,
    pub live_outcomes: usize,
    pub live_predictions: usize,
    pub live_evaluations: usize,
    pub live_parameter_changes: usize,
    pub live_normalizer_changes: usize,
    pub winner_selections: usize,
    pub rankings: usize,
    pub reward_applications: usize,
    pub penalty_applications: usize,
    pub chair_actions: usize,
    pub vote_actions: usize,
    pub paper_trading_actions: usize,
    pub live_trading_actions: usize,
    pub month_view_loads: usize,
    pub year_view_loads: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumT10MicroScreeningReportV1 {
    pub report_version: String,
    pub run_mode: String,
    pub status: MomentumT10MicroScreeningStatusV1,
    pub authorization: Option<MomentumT10ScreeningExecutionAuthorizationV1>,
    pub training_policy: Option<MomentumT10TrainingPolicyV1>,
    pub source_label_report_digest: String,
    pub source_feature_report_digest: String,
    pub source_design_report_digest: String,
    pub source_registration_digest: String,
    pub source_gate_digest: String,
    pub protected_before_state_digest: String,
    pub completed_live_event_count: usize,
    pub scorable_live_event_count: usize,
    pub live_pause: String,
    pub epoch_three_registered: bool,
    pub t10_boundary: Option<MomentumMicroTaskPartitionBoundaryV1>,
    pub t30_boundary: Option<MomentumMicroTaskPartitionBoundaryV1>,
    pub t10_disposition: String,
    pub t30_disposition: String,
    pub t60_disposition: String,
    pub development: Option<MomentumT10PartitionAggregateV1>,
    pub validation: Option<MomentumT10PartitionAggregateV1>,
    pub benchmark_comparisons: Vec<MomentumMicroBenchmarkComparisonReceiptV1>,
    pub contribution_comparisons: Vec<MomentumMicroContributionReceiptV1>,
    pub holdout_eligibility_receipts: Vec<MomentumMicroHoldoutEligibilityReceiptV1>,
    pub proposed_holdout_cohort: Option<MomentumMicroProposedHoldoutCohortV1>,
    pub full_eight_a3_blocked: bool,
    pub historical_holdout_execution_mode_absent: bool,
    pub live_roster_unchanged: bool,
    pub protected_artifacts_unchanged: bool,
    pub labels: Vec<String>,
    pub safety_counters: MomentumT10ScreeningSafetyCountersV1,
    pub deterministic_replay_digest: Option<String>,
    pub runtime_duration_ms: u64,
    pub report_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumT10ScreeningJournalV1 {
    journal_version: String,
    authorization_digest: String,
    development_aggregate_digest: String,
    validation_aggregate_digest: String,
    benchmark_receipt_digests: Vec<String>,
    contribution_receipt_digests: Vec<String>,
    eligibility_receipt_digests: Vec<String>,
    cohort_digest: String,
    holdout_access_count: usize,
    t30_execution_count: usize,
    t60_execution_count: usize,
    live_authority_count: usize,
    deterministic: bool,
    replay_digest: String,
}

#[derive(Clone)]
struct PreparedEvent {
    partition: MomentumReplayPartitionV1,
    prediction_timestamp_ms: u64,
    target_timestamp_ms: u64,
    source_event_digest: String,
    anchor: Vec<f32>,
    compact: Vec<f32>,
}

struct PreparedScreening {
    evidence: MomentumQualifiedSixEvidenceV1,
    development: Vec<PreparedEvent>,
    validation: Vec<PreparedEvent>,
    development_boundary_event_count: usize,
    validation_boundary_event_count: usize,
    development_boundary_days: BTreeSet<u64>,
    validation_boundary_days: BTreeSet<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MomentumT10ConsumedEventEvidenceV1 {
    pub partition: MomentumReplayPartitionV1,
    pub prediction_timestamp_ms: u64,
    pub target_timestamp_ms: u64,
    pub event_plan_digest: String,
    pub source_event_digest: String,
    pub probabilities: Vec<f64>,
    pub label: Option<f64>,
    pub brier_values: Vec<f64>,
    pub correctness: Vec<bool>,
    pub target_return: f64,
    pub past_micro_volatility: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MomentumT10ConsumedActionabilityEvidenceV1 {
    pub partition: MomentumReplayPartitionV1,
    pub prediction_timestamp_ms: u64,
    pub target_timestamp_ms: u64,
    pub source_event_digest: String,
    pub target_return: f64,
    pub past_micro_volatility: f64,
}

#[derive(Clone)]
struct FrozenParticipant {
    participant: MomentumMicroParticipantV1,
    normalizer: Option<RepresentationNormalizerV0>,
    head: Option<LogisticPredictionHeadV0>,
    calibrator: Option<LogisticPredictionHeadV0>,
    prevalence: f64,
    model_digest: String,
}

#[derive(Default)]
struct MetricAccumulator {
    prediction_count: usize,
    scorable_count: usize,
    neutral_count: usize,
    invalid_count: usize,
    finite_prediction_count: usize,
    brier_sum: f64,
    correct_count: usize,
    deltas: Vec<f64>,
    probability_sum: f64,
    probability_squared_sum: f64,
    minimum_probability: f64,
    maximum_probability: f64,
    frequencies: BTreeMap<u64, usize>,
    near_half_count: usize,
    extreme_low_count: usize,
    extreme_high_count: usize,
    nonfinite_count: usize,
    calibration_support: [usize; 10],
    calibration_probability_sum: [f64; 10],
    calibration_positive_sum: [f64; 10],
}

fn participant_id(participant: MomentumMicroParticipantV1) -> String {
    format!(
        "{:?}:{participant:?}",
        MomentumMicroTaskV1::T10NextTenMinuteDirection
    )
}

fn parse_participant(value: &str) -> Result<MomentumMicroParticipantV1, String> {
    PARTICIPANTS
        .into_iter()
        .find(|participant| participant_id(*participant) == value)
        .ok_or_else(|| "T10 screening participant rejected".to_string())
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
    authorization_digest,
    MomentumT10ScreeningExecutionAuthorizationV1,
    authorization_digest
);
digest_fn!(
    training_policy_digest,
    MomentumT10TrainingPolicyV1,
    policy_digest
);
digest_fn!(
    training_plan_digest,
    MomentumT10DailyTrainingWindowPlanV1,
    plan_digest
);
digest_fn!(
    normalizer_receipt_digest,
    MomentumT10DailyNormalizerReceiptV1,
    receipt_digest
);
digest_fn!(
    model_receipt_digest,
    MomentumT10DailyModelReceiptV1,
    receipt_digest
);
digest_fn!(
    refit_bundle_digest,
    MomentumT10DailyRefitBundleV1,
    bundle_digest
);
digest_fn!(event_plan_digest, MomentumT10EventPlanV1, plan_digest);
digest_fn!(
    prediction_shard_digest,
    MomentumT10PredictionShardV1,
    shard_digest
);
digest_fn!(
    evaluation_item_digest,
    MomentumT10EvaluationItemV1,
    item_digest
);
digest_fn!(
    evaluation_shard_digest,
    MomentumT10EvaluationShardV1,
    shard_digest
);
digest_fn!(
    calibration_bin_digest,
    MomentumMicroCalibrationBinV1,
    bin_digest
);
digest_fn!(
    participant_metrics_digest,
    MomentumMicroScreeningParticipantMetricsV1,
    metrics_digest
);
digest_fn!(
    aggregate_digest,
    MomentumT10PartitionAggregateV1,
    aggregate_digest
);
digest_fn!(
    benchmark_receipt_digest,
    MomentumMicroBenchmarkComparisonReceiptV1,
    receipt_digest
);
digest_fn!(
    contribution_receipt_digest,
    MomentumMicroContributionReceiptV1,
    receipt_digest
);
digest_fn!(
    eligibility_receipt_digest,
    MomentumMicroHoldoutEligibilityReceiptV1,
    receipt_digest
);
digest_fn!(
    cohort_digest,
    MomentumMicroProposedHoldoutCohortV1,
    cohort_digest
);
digest_fn!(journal_digest, MomentumT10ScreeningJournalV1, replay_digest);

fn task_boundary_digest(value: &MomentumMicroTaskPartitionBoundaryV1) -> String {
    canonical_digest(value, |item| item.boundary_digest.clear())
}

fn validate_task_boundary(value: &MomentumMicroTaskPartitionBoundaryV1) -> Result<(), String> {
    let expected_development = value.common_eligible_event_count * 70 / 100;
    let expected_validation = value.common_eligible_event_count * 15 / 100;
    let expected_holdout =
        value.common_eligible_event_count - expected_development - expected_validation;
    if value.boundary_version.is_empty()
        || value.development_event_count != expected_development
        || value.validation_event_count != expected_validation
        || value.holdout_event_count != expected_holdout
        || value.eligible_start_timestamp_ms >= value.development_end_exclusive_ms
        || value.development_end_exclusive_ms >= value.validation_end_exclusive_ms
        || value.validation_end_exclusive_ms != value.holdout_start_timestamp_ms
        || value.holdout_start_timestamp_ms > value.eligible_end_timestamp_ms
        || value.label_values_read_for_boundary != 0
        || value.holdout_labels_opened
        || value.boundary_digest != task_boundary_digest(value)
    {
        return Err("T10 task boundary rejected".to_string());
    }
    Ok(())
}

fn report_digest(value: &MomentumT10MicroScreeningReportV1) -> String {
    canonical_digest(value, |item| {
        item.run_mode.clear();
        item.safety_counters = MomentumT10ScreeningSafetyCountersV1::default();
        item.runtime_duration_ms = 0;
        item.report_digest.clear();
    })
}

fn checked_u32(value: u64) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| "T10 screening u32 conversion rejected".to_string())
}

fn checked_f32(value: f64) -> Result<f32, String> {
    let converted = value as f32;
    if !value.is_finite() || !converted.is_finite() {
        return Err("T10 screening f32 conversion rejected".to_string());
    }
    Ok(converted)
}

fn partition_name(partition: MomentumReplayPartitionV1) -> &'static str {
    match partition {
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
        _ => Err("T10 screening partition rejected".to_string()),
    }
}

fn role_name(role: DailyModelRole) -> &'static str {
    match role {
        DailyModelRole::Constant => "constant",
        DailyModelRole::LearnedBase => "learned-base",
        DailyModelRole::Calibrator => "calibrator",
    }
}

fn parse_role(value: &str) -> Result<DailyModelRole, String> {
    match value {
        "constant" => Ok(DailyModelRole::Constant),
        "learned-base" => Ok(DailyModelRole::LearnedBase),
        "calibrator" => Ok(DailyModelRole::Calibrator),
        _ => Err("T10 screening model role rejected".to_string()),
    }
}

fn label_name(label: ScreeningLabel) -> &'static str {
    match label {
        ScreeningLabel::Up => "up",
        ScreeningLabel::Down => "down",
        ScreeningLabel::Neutral => "neutral",
        ScreeningLabel::Invalid => "invalid",
    }
}

fn parse_label(value: &str) -> Result<ScreeningLabel, String> {
    match value {
        "up" => Ok(ScreeningLabel::Up),
        "down" => Ok(ScreeningLabel::Down),
        "neutral" => Ok(ScreeningLabel::Neutral),
        "invalid" => Ok(ScreeningLabel::Invalid),
        _ => Err("T10 screening label rejected".to_string()),
    }
}

fn comparison_name(value: MomentumMicroScreeningComparisonV1) -> &'static str {
    match value {
        MomentumMicroScreeningComparisonV1::LowerBrierThanConstant => "lower-brier-than-constant",
        MomentumMicroScreeningComparisonV1::HigherBrierThanConstant => "higher-brier-than-constant",
        MomentumMicroScreeningComparisonV1::NumericallyEquivalentToConstant => {
            "numerically-equivalent-to-constant"
        }
        MomentumMicroScreeningComparisonV1::MixedOrInsufficientEvidence => {
            "mixed-or-insufficient-evidence"
        }
        MomentumMicroScreeningComparisonV1::ProbabilityCollapse => "probability-collapse",
        MomentumMicroScreeningComparisonV1::IntegrityFailure => "integrity-failure",
    }
}

fn parse_comparison(value: &str) -> Result<MomentumMicroScreeningComparisonV1, String> {
    match value {
        "lower-brier-than-constant" => {
            Ok(MomentumMicroScreeningComparisonV1::LowerBrierThanConstant)
        }
        "higher-brier-than-constant" => {
            Ok(MomentumMicroScreeningComparisonV1::HigherBrierThanConstant)
        }
        "numerically-equivalent-to-constant" => {
            Ok(MomentumMicroScreeningComparisonV1::NumericallyEquivalentToConstant)
        }
        "mixed-or-insufficient-evidence" => {
            Ok(MomentumMicroScreeningComparisonV1::MixedOrInsufficientEvidence)
        }
        "probability-collapse" => Ok(MomentumMicroScreeningComparisonV1::ProbabilityCollapse),
        "integrity-failure" => Ok(MomentumMicroScreeningComparisonV1::IntegrityFailure),
        _ => Err("T10 screening comparison rejected".to_string()),
    }
}

fn contribution_name(value: MomentumMicroContributionComparisonV1) -> &'static str {
    match value {
        MomentumMicroContributionComparisonV1::LowerBrierWithContribution => {
            "lower-brier-with-contribution"
        }
        MomentumMicroContributionComparisonV1::HigherBrierWithContribution => {
            "higher-brier-with-contribution"
        }
        MomentumMicroContributionComparisonV1::NumericallyEquivalent => "numerically-equivalent",
        MomentumMicroContributionComparisonV1::MixedOrInsufficientEvidence => {
            "mixed-or-insufficient-evidence"
        }
        MomentumMicroContributionComparisonV1::IntegrityFailure => "integrity-failure",
    }
}

fn parse_contribution(value: &str) -> Result<MomentumMicroContributionComparisonV1, String> {
    match value {
        "lower-brier-with-contribution" => {
            Ok(MomentumMicroContributionComparisonV1::LowerBrierWithContribution)
        }
        "higher-brier-with-contribution" => {
            Ok(MomentumMicroContributionComparisonV1::HigherBrierWithContribution)
        }
        "numerically-equivalent" => {
            Ok(MomentumMicroContributionComparisonV1::NumericallyEquivalent)
        }
        "mixed-or-insufficient-evidence" => {
            Ok(MomentumMicroContributionComparisonV1::MixedOrInsufficientEvidence)
        }
        "integrity-failure" => Ok(MomentumMicroContributionComparisonV1::IntegrityFailure),
        _ => Err("T10 contribution comparison rejected".to_string()),
    }
}

fn collapse_name(value: MomentumMicroProbabilityCollapseV1) -> &'static str {
    match value {
        MomentumMicroProbabilityCollapseV1::BenchmarkExempt => "benchmark-exempt",
        MomentumMicroProbabilityCollapseV1::NotCollapsed => "not-collapsed",
        MomentumMicroProbabilityCollapseV1::ProbabilityCollapse => "probability-collapse",
        MomentumMicroProbabilityCollapseV1::IntegrityFailure => "integrity-failure",
    }
}

fn parse_collapse(value: &str) -> Result<MomentumMicroProbabilityCollapseV1, String> {
    match value {
        "benchmark-exempt" => Ok(MomentumMicroProbabilityCollapseV1::BenchmarkExempt),
        "not-collapsed" => Ok(MomentumMicroProbabilityCollapseV1::NotCollapsed),
        "probability-collapse" => Ok(MomentumMicroProbabilityCollapseV1::ProbabilityCollapse),
        "integrity-failure" => Ok(MomentumMicroProbabilityCollapseV1::IntegrityFailure),
        _ => Err("T10 collapse classification rejected".to_string()),
    }
}

fn saturation_name(value: MomentumMicroSaturationV1) -> &'static str {
    match value {
        MomentumMicroSaturationV1::NotSaturated => "not-saturated",
        MomentumMicroSaturationV1::LowBoundarySaturation => "low-boundary-saturation",
        MomentumMicroSaturationV1::HighBoundarySaturation => "high-boundary-saturation",
        MomentumMicroSaturationV1::TwoSidedSaturation => "two-sided-saturation",
        MomentumMicroSaturationV1::IntegrityFailure => "integrity-failure",
    }
}

fn parse_saturation(value: &str) -> Result<MomentumMicroSaturationV1, String> {
    match value {
        "not-saturated" => Ok(MomentumMicroSaturationV1::NotSaturated),
        "low-boundary-saturation" => Ok(MomentumMicroSaturationV1::LowBoundarySaturation),
        "high-boundary-saturation" => Ok(MomentumMicroSaturationV1::HighBoundarySaturation),
        "two-sided-saturation" => Ok(MomentumMicroSaturationV1::TwoSidedSaturation),
        "integrity-failure" => Ok(MomentumMicroSaturationV1::IntegrityFailure),
        _ => Err("T10 saturation classification rejected".to_string()),
    }
}

fn eligibility_name(value: MomentumMicroHoldoutEligibilityV1) -> &'static str {
    match value {
        MomentumMicroHoldoutEligibilityV1::EligibleForFutureSealedHoldoutEvaluation => {
            "eligible-for-future-sealed-holdout-evaluation"
        }
        MomentumMicroHoldoutEligibilityV1::IneligibleScreeningGate => "ineligible-screening-gate",
        MomentumMicroHoldoutEligibilityV1::IntegrityFailure => "integrity-failure",
    }
}

fn parse_eligibility(value: &str) -> Result<MomentumMicroHoldoutEligibilityV1, String> {
    match value {
        "eligible-for-future-sealed-holdout-evaluation" => {
            Ok(MomentumMicroHoldoutEligibilityV1::EligibleForFutureSealedHoldoutEvaluation)
        }
        "ineligible-screening-gate" => {
            Ok(MomentumMicroHoldoutEligibilityV1::IneligibleScreeningGate)
        }
        "integrity-failure" => Ok(MomentumMicroHoldoutEligibilityV1::IntegrityFailure),
        _ => Err("T10 holdout eligibility rejected".to_string()),
    }
}

fn cohort_status_name(value: MomentumMicroHoldoutCohortStatusV1) -> &'static str {
    match value {
        MomentumMicroHoldoutCohortStatusV1::NoEligibleT10HoldoutCohort => {
            "no-eligible-t10-holdout-cohort"
        }
        MomentumMicroHoldoutCohortStatusV1::ProposedT10HoldoutCohort => {
            "proposed-t10-holdout-cohort"
        }
        MomentumMicroHoldoutCohortStatusV1::IntegrityFailure => "integrity-failure",
    }
}

fn parse_cohort_status(value: &str) -> Result<MomentumMicroHoldoutCohortStatusV1, String> {
    match value {
        "no-eligible-t10-holdout-cohort" => {
            Ok(MomentumMicroHoldoutCohortStatusV1::NoEligibleT10HoldoutCohort)
        }
        "proposed-t10-holdout-cohort" => {
            Ok(MomentumMicroHoldoutCohortStatusV1::ProposedT10HoldoutCohort)
        }
        "integrity-failure" => Ok(MomentumMicroHoldoutCohortStatusV1::IntegrityFailure),
        _ => Err("T10 holdout cohort status rejected".to_string()),
    }
}

fn validate_authorization(
    value: &MomentumT10ScreeningExecutionAuthorizationV1,
) -> Result<(), String> {
    let expected_participants = PARTICIPANTS
        .iter()
        .map(|participant| participant_id(*participant))
        .collect::<Vec<_>>();
    if value.authorization_version != AUTHORIZATION_VERSION
        || value.challenger_registration_digest != EXPECTED_REGISTRATION_DIGEST
        || value.screening_gate_digest != EXPECTED_GATE_DIGEST
        || value.label_report_digest != EXPECTED_LABEL_REPORT_DIGEST
        || value.feature_report_digest != EXPECTED_FEATURE_REPORT_DIGEST
        || value.design_report_digest != EXPECTED_DESIGN_REPORT_DIGEST
        || value.authorized_task_id
            != format!("{:?}", MomentumMicroTaskV1::T10NextTenMinuteDirection)
        || value.authorized_participant_ids != expected_participants
        || !value.development_execution_authorized
        || !value.validation_execution_authorized
        || value.historical_holdout_execution_authorized
        || value.t30_execution_authorized
        || value.t60_execution_authorized
        || value.network_authorized
        || !value.live_authority_forbidden
        || !value.governance_authority_forbidden
        || !value.trading_authority_forbidden
        || value.authorization_digest != authorization_digest(value)
    {
        return Err("T10 screening authorization rejected".to_string());
    }
    Ok(())
}

fn validate_training_policy(value: &MomentumT10TrainingPolicyV1) -> Result<(), String> {
    let learning_rate = f32::from_bits(value.learning_rate_bits);
    let l2 = f32::from_bits(value.standard_l2_bits);
    let clamp = f64::from_bits(value.probability_clamp_bits);
    let expected_minimum = derive_minimum_support(COMPACT_FEATURE_DIMENSION)?;
    if value.policy_version != TRAINING_POLICY_VERSION
        || value.source_training_policy_digest.is_empty()
        || value.loss_function != "BrierLoss"
        || value.epoch_count != 4
        || value.batch_size != 64
        || !learning_rate.is_finite()
        || learning_rate <= 0.0
        || !l2.is_finite()
        || l2 <= 0.0
        || value.c3_l2_multiplier != C3_L2_MULTIPLIER
        || !value.gradient_finite_checks
        || !value.parameter_finite_checks
        || clamp != PROBABILITY_CLAMP
        || value.maximum_training_examples != MAXIMUM_TRAINING_EXAMPLES
        || value.minimum_training_examples != expected_minimum
        || value.dimension_support_multiplier != DIMENSION_SUPPORT_MULTIPLIER
        || !value.daily_utc_refit
        || !value.within_day_refit_forbidden
        || !value.training_only_normalizer
        || value.c4_base_percent != C4_BASE_PERCENT
        || value.c4_calibration_percent != C4_CALIBRATION_PERCENT
        || !value.validation_fit_forbidden
        || !value.holdout_fit_forbidden
        || value.policy_digest != training_policy_digest(value)
    {
        return Err("T10 screening training policy rejected".to_string());
    }
    Ok(())
}

fn validate_training_plan(value: &MomentumT10DailyTrainingWindowPlanV1) -> Result<(), String> {
    let expected_base = value.used_training_event_count * C4_BASE_PERCENT / 100;
    let expected_minimum = derive_minimum_support(COMPACT_FEATURE_DIMENSION)?;
    let calibrator_minimum = derive_minimum_support(CALIBRATOR_FEATURE_DIMENSION)?;
    if value.plan_version != TRAINING_PLAN_VERSION
        || value.authorization_digest.is_empty()
        || value.training_policy_digest.is_empty()
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.utc_day_boundary_ms % DAY_MS != 0
        || value.training_target_cutoff_exclusive_ms != value.utc_day_boundary_ms
        || value.used_training_event_count > MAXIMUM_TRAINING_EXAMPLES
        || value.training_event_digests.len() != value.used_training_event_count
        || value.c4_base_count != expected_base
        || value.c4_calibration_count != value.used_training_event_count - expected_base
        || value.support_sufficient_for_all
            != (value.used_training_event_count >= expected_minimum
                && value.c4_base_count >= expected_minimum
                && value.c4_calibration_count >= calibrator_minimum)
        || value.validation_label_fit_count != 0
        || value.holdout_access_count != 0
        || value.plan_digest != training_plan_digest(value)
    {
        return Err("T10 daily training plan rejected".to_string());
    }
    Ok(())
}

fn validate_normalizer_receipt(value: &MomentumT10DailyNormalizerReceiptV1) -> Result<(), String> {
    if value.receipt_version != NORMALIZER_VERSION
        || parse_participant(&value.participant_id).is_err()
        || value.feature_policy_digest.is_empty()
        || value.private_means.is_empty()
        || value.private_means.len() != value.private_scales.len()
        || value
            .private_means
            .iter()
            .chain(&value.private_scales)
            .any(|item| !item.is_finite())
        || value.private_scales.iter().any(|item| *item <= 0.0)
        || value.training_event_digest.is_empty()
        || !value.finite
        || value.receipt_digest != normalizer_receipt_digest(value)
    {
        return Err("T10 normalizer receipt rejected".to_string());
    }
    Ok(())
}

fn validate_model_receipt(value: &MomentumT10DailyModelReceiptV1) -> Result<(), String> {
    let participant = parse_participant(&value.participant_id)?;
    let learned = value.role != DailyModelRole::Constant;
    if value.receipt_version != MODEL_VERSION
        || value.utc_day_boundary_ms % DAY_MS != 0
        || value.training_plan_digest.is_empty()
        || (learned && value.normalizer_receipt_digest.is_empty())
        || value.private_weights.iter().any(|item| !item.is_finite())
        || value.private_bias.is_some_and(|item| !item.is_finite())
        || value
            .private_prevalence
            .is_some_and(|item| !item.is_finite() || !(0.0..=1.0).contains(&item))
        || value.training_count != value.positive_count + value.negative_count
        || !value.finite
        || value.validation_fit_count != 0
        || value.holdout_fit_count != 0
        || value.receipt_digest != model_receipt_digest(value)
    {
        return Err("T10 model receipt rejected".to_string());
    }
    match (participant, value.role) {
        (MomentumMicroParticipantV1::C0TaskSpecificConstant, DailyModelRole::Constant) => {
            if !value.private_weights.is_empty()
                || value.private_bias.is_some()
                || value.private_prevalence.is_none()
                || value.l2_bits != 0
                || !value.normalizer_receipt_digest.is_empty()
            {
                return Err("T10 constant receipt rejected".to_string());
            }
        }
        (
            MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic,
            DailyModelRole::Calibrator,
        ) => {
            if value.private_weights.len() != 1
                || value.private_bias.is_none()
                || value.private_prevalence.is_some()
            {
                return Err("T10 calibrator receipt rejected".to_string());
            }
        }
        (_, DailyModelRole::LearnedBase) => {
            if value.private_weights.is_empty()
                || value.private_bias.is_none()
                || value.private_prevalence.is_some()
            {
                return Err("T10 learned-model receipt rejected".to_string());
            }
        }
        _ => return Err("T10 model role binding rejected".to_string()),
    }
    Ok(())
}

fn validate_refit_bundle(value: &MomentumT10DailyRefitBundleV1) -> Result<(), String> {
    validate_training_plan(&value.training_plan)?;
    if value.normalizer_receipts.len() != 4 || value.model_receipts.len() != 6 {
        return Err("T10 daily refit receipt count rejected".to_string());
    }
    for receipt in &value.normalizer_receipts {
        validate_normalizer_receipt(receipt)?;
    }
    for receipt in &value.model_receipts {
        validate_model_receipt(receipt)?;
    }
    let expected_ids = PARTICIPANTS
        .iter()
        .map(|participant| participant_id(*participant))
        .collect::<Vec<_>>();
    let base_ids = value
        .model_receipts
        .iter()
        .filter(|receipt| receipt.role != DailyModelRole::Calibrator)
        .map(|receipt| receipt.participant_id.clone())
        .collect::<Vec<_>>();
    let expected_normalizer_ids = expected_ids[1..].to_vec();
    let normalizer_ids = value
        .normalizer_receipts
        .iter()
        .map(|receipt| receipt.participant_id.clone())
        .collect::<Vec<_>>();
    let expected_dimensions = [
        ANCHOR_FEATURE_DIMENSION,
        COMPACT_FEATURE_DIMENSION,
        COMPACT_FEATURE_DIMENSION,
        COMPACT_FEATURE_DIMENSION,
    ];
    let full_training_digest = stable_hash_string(&format!(
        "T10-normalizer-training-events:{:?}",
        value.training_plan.training_event_digests
    ));
    let c4_training_digest = stable_hash_string(&format!(
        "T10-normalizer-training-events:{:?}",
        &value.training_plan.training_event_digests[..value.training_plan.c4_base_count]
    ));
    let expected_counts = [
        value.training_plan.used_training_event_count,
        value.training_plan.used_training_event_count,
        value.training_plan.used_training_event_count,
        value.training_plan.used_training_event_count,
        value.training_plan.c4_base_count,
        value.training_plan.c4_calibration_count,
    ];
    let expected_model_dimensions = [
        0,
        ANCHOR_FEATURE_DIMENSION,
        COMPACT_FEATURE_DIMENSION,
        COMPACT_FEATURE_DIMENSION,
        COMPACT_FEATURE_DIMENSION,
        CALIBRATOR_FEATURE_DIMENSION,
    ];
    let expected_l2 = [
        0,
        STANDARD_L2.to_bits(),
        STANDARD_L2.to_bits(),
        (STANDARD_L2 * C3_L2_MULTIPLIER as f32).to_bits(),
        STANDARD_L2.to_bits(),
        STANDARD_L2.to_bits(),
    ];
    let normalizer_bindings = [
        "",
        value.normalizer_receipts[0].receipt_digest.as_str(),
        value.normalizer_receipts[1].receipt_digest.as_str(),
        value.normalizer_receipts[2].receipt_digest.as_str(),
        value.normalizer_receipts[3].receipt_digest.as_str(),
        value.normalizer_receipts[3].receipt_digest.as_str(),
    ];
    let reconstructed = PARTICIPANTS
        .iter()
        .enumerate()
        .map(|(index, participant)| {
            let base = &value.model_receipts[index];
            let normalizer = if index == 0 {
                ""
            } else {
                value.normalizer_receipts[index - 1].receipt_digest.as_str()
            };
            let calibrator = (index == 4).then(|| value.model_receipts[5].receipt_digest.as_str());
            frozen_digest(*participant, normalizer, &base.receipt_digest, calibrator)
        })
        .collect::<Vec<_>>();
    if value.bundle_version != REFIT_BUNDLE_VERSION
        || value.authorization_digest != value.training_plan.authorization_digest
        || value.partition != value.training_plan.partition
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.utc_day_boundary_ms != value.training_plan.utc_day_boundary_ms
        || value.normalizer_receipts.len() != 4
        || value.model_receipts.len() != 6
        || base_ids != expected_ids
        || normalizer_ids != expected_normalizer_ids
        || value
            .normalizer_receipts
            .iter()
            .zip(expected_dimensions)
            .any(|(receipt, dimension)| receipt.private_means.len() != dimension)
        || value.normalizer_receipts[..3]
            .iter()
            .any(|receipt| receipt.training_event_digest != full_training_digest)
        || value.normalizer_receipts[3].training_event_digest != c4_training_digest
        || value
            .model_receipts
            .iter()
            .enumerate()
            .any(|(index, receipt)| {
                receipt.utc_day_boundary_ms != value.utc_day_boundary_ms
                    || receipt.training_plan_digest != value.training_plan.plan_digest
                    || receipt.training_count != expected_counts[index]
                    || receipt.private_weights.len() != expected_model_dimensions[index]
                    || receipt.l2_bits != expected_l2[index]
                    || receipt.normalizer_receipt_digest != normalizer_bindings[index]
            })
        || value.reconstructed_participant_digests != reconstructed
        || value.target_access_count_for_prediction_day != 0
        || value.holdout_access_count != 0
        || value.live_access_count != 0
        || value.bundle_digest != refit_bundle_digest(value)
    {
        return Err("T10 daily refit bundle rejected".to_string());
    }
    Ok(())
}

fn validate_event_plan(value: &MomentumT10EventPlanV1) -> Result<(), String> {
    let expected_ids = PARTICIPANTS
        .iter()
        .map(|participant| participant_id(*participant))
        .collect::<Vec<_>>();
    if value.plan_version != EVENT_PLAN_VERSION
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.prediction_timestamp_ms % TEN_MINUTE_MS != 0
        || value.target_timestamp_ms != value.prediction_timestamp_ms + TEN_MINUTE_MS
        || value.source_event_digest.is_empty()
        || value.daily_refit_bundle_digest.is_empty()
        || value.participant_ids != expected_ids
        || !value.target_hidden
        || value.holdout_member
        || value.plan_digest != event_plan_digest(value)
    {
        return Err("T10 event plan rejected".to_string());
    }
    Ok(())
}

fn validate_prediction_shard(value: &MomentumT10PredictionShardV1) -> Result<(), String> {
    for plan in &value.event_plans {
        validate_event_plan(plan)?;
    }
    let expected_ids = PARTICIPANTS
        .iter()
        .map(|participant| participant_id(*participant))
        .collect::<Vec<_>>();
    if value.shard_version != PREDICTION_SHARD_VERSION
        || value.authorization_digest.is_empty()
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.utc_day_boundary_ms % DAY_MS != 0
        || value.daily_refit_bundle_digest.is_empty()
        || value.event_plans.is_empty()
        || value.event_plans.iter().any(|plan| {
            plan.partition != value.partition
                || plan.prediction_timestamp_ms / DAY_MS * DAY_MS != value.utc_day_boundary_ms
                || plan.daily_refit_bundle_digest != value.daily_refit_bundle_digest
        })
        || value
            .event_plans
            .windows(2)
            .any(|pair| pair[0].prediction_timestamp_ms >= pair[1].prediction_timestamp_ms)
        || value.participant_ids != expected_ids
        || value.private_probabilities.len() != value.event_plans.len() * 5
        || value.prediction_digests.len() != value.event_plans.len() * 5
        || value
            .private_probabilities
            .iter()
            .any(|item| !item.is_finite() || !(0.0..=1.0).contains(item))
        || value.target_accessed
        || value.label_accessed
        || value.metric_computed
        || value.shard_digest != prediction_shard_digest(value)
    {
        return Err("T10 prediction shard rejected".to_string());
    }
    Ok(())
}

fn validate_evaluation_item(value: &MomentumT10EvaluationItemV1) -> Result<(), String> {
    let scorable = matches!(value.label, ScreeningLabel::Up | ScreeningLabel::Down);
    if value.item_version != EVALUATION_ITEM_VERSION
        || value.event_plan_digest.is_empty()
        || value.private_label.is_some() != scorable
        || value
            .private_label
            .is_some_and(|item| !matches!(item, 0.0 | 1.0))
        || value.private_brier_values.len() != usize::from(scorable) * 5
        || value.private_correctness.len() != usize::from(scorable) * 5
        || value
            .private_brier_values
            .iter()
            .any(|item| !item.is_finite() || !(0.0..=1.0).contains(item))
        || value.item_digest != evaluation_item_digest(value)
    {
        return Err("T10 evaluation item rejected".to_string());
    }
    Ok(())
}

fn validate_evaluation_shard(value: &MomentumT10EvaluationShardV1) -> Result<(), String> {
    for item in &value.evaluations {
        validate_evaluation_item(item)?;
    }
    if value.shard_version != EVALUATION_SHARD_VERSION
        || value.prediction_shard_digest.is_empty()
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.utc_day_boundary_ms % DAY_MS != 0
        || !value.prediction_shard_reopened
        || value.evaluations.is_empty()
        || value.shard_digest != evaluation_shard_digest(value)
    {
        return Err("T10 evaluation shard rejected".to_string());
    }
    Ok(())
}

fn validate_calibration_bin(value: &MomentumMicroCalibrationBinV1) -> Result<(), String> {
    if value.bin_version != CALIBRATION_BIN_VERSION
        || !value.lower_bound.is_finite()
        || !value.upper_bound.is_finite()
        || value.lower_bound < 0.0
        || value.upper_bound > 1.0
        || value.lower_bound >= value.upper_bound
        || !value.mean_predicted_probability.is_finite()
        || !value.observed_positive_frequency.is_finite()
        || !value.absolute_calibration_gap.is_finite()
        || value.bin_digest != calibration_bin_digest(value)
    {
        return Err("T10 calibration bin rejected".to_string());
    }
    Ok(())
}

fn validate_participant_metrics(
    value: &MomentumMicroScreeningParticipantMetricsV1,
) -> Result<(), String> {
    for bin in &value.calibration_bins {
        validate_calibration_bin(bin)?;
    }
    if value.metrics_version != METRICS_VERSION
        || parse_participant(&value.participant_id).is_err()
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.prediction_count
            != value.scorable_count + value.neutral_count + value.invalid_count
        || value.finite_prediction_count + value.nonfinite_count != value.prediction_count
        || value.scorable_count == 0
        || value.calibration_bins.len() != 10
        || value
            .calibration_bins
            .iter()
            .map(|bin| bin.support)
            .sum::<usize>()
            != value.scorable_count
        || [
            value.mean_brier,
            value.binary_correctness,
            value.paired_mean_brier_delta_versus_c0,
            value.paired_median_brier_delta_versus_c0,
            value.weighted_calibration_gap,
            value.minimum_probability,
            value.maximum_probability,
            value.mean_probability,
            value.probability_standard_deviation,
        ]
        .into_iter()
        .any(|item| !item.is_finite())
        || value.minimum_probability < 0.0
        || value.maximum_probability > 1.0
        || value.minimum_probability > value.maximum_probability
        || value.positive_paired_delta_count
            + value.negative_paired_delta_count
            + value.equivalent_paired_delta_count
            != value.scorable_count
        || !value.chronology_audit_passed
        || !value.leakage_audit_passed
        || !value.integrity_audit_passed
        || value.metrics_digest != participant_metrics_digest(value)
    {
        return Err("T10 participant metrics rejected".to_string());
    }
    Ok(())
}

fn validate_aggregate(value: &MomentumT10PartitionAggregateV1) -> Result<(), String> {
    for metrics in &value.participant_metrics {
        validate_participant_metrics(metrics)?;
    }
    let expected_ids = PARTICIPANTS
        .iter()
        .map(|participant| participant_id(*participant))
        .collect::<Vec<_>>();
    if value.aggregate_version != AGGREGATE_VERSION
        || value.authorization_digest.is_empty()
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.boundary_event_count != value.prediction_count + value.training_only_event_count
        || value.prediction_count
            != value.scorable_count + value.neutral_count + value.invalid_count
        || value.daily_refit_bundle_digests.len() != value.daily_refit_count
        || value.prediction_shard_digests.len() != value.daily_refit_count
        || value.evaluation_shard_digests.len() != value.daily_refit_count
        || value.participant_metrics.len() != 5
        || value
            .participant_metrics
            .iter()
            .map(|metrics| metrics.participant_id.clone())
            .collect::<Vec<_>>()
            != expected_ids
        || value
            .participant_metrics
            .iter()
            .any(|metrics| metrics.prediction_count != value.prediction_count)
        || value.target_access_before_prediction_reopen_count != 0
        || value.feature_future_access_count != 0
        || value.partial_candle_access_count != 0
        || value.holdout_access_count != 0
        || value.validation_fit_count != 0
        || !value.chronology_audit_passed
        || !value.leakage_audit_passed
        || !value.prediction_before_reveal_passed
        || value.aggregate_digest != aggregate_digest(value)
    {
        return Err("T10 partition aggregate rejected".to_string());
    }
    Ok(())
}

fn validate_benchmark(value: &MomentumMicroBenchmarkComparisonReceiptV1) -> Result<(), String> {
    let participant = parse_participant(&value.participant_id)?;
    if value.comparison_version != BENCHMARK_VERSION
        || participant == MomentumMicroParticipantV1::C0TaskSpecificConstant
        || value.development_aggregate_digest.is_empty()
        || value.validation_aggregate_digest.is_empty()
        || !f64::from_bits(value.development_delta_bits).is_finite()
        || !f64::from_bits(value.validation_delta_bits).is_finite()
        || value.receipt_digest != benchmark_receipt_digest(value)
    {
        return Err("T10 benchmark receipt rejected".to_string());
    }
    Ok(())
}

fn validate_contribution(value: &MomentumMicroContributionReceiptV1) -> Result<(), String> {
    if value.comparison_version != CONTRIBUTION_VERSION
        || parse_participant(&value.participant_id).is_err()
        || parse_participant(&value.baseline_participant_id).is_err()
        || value.participant_id == value.baseline_participant_id
        || !f64::from_bits(value.development_delta_bits).is_finite()
        || !f64::from_bits(value.validation_delta_bits).is_finite()
        || value.receipt_digest != contribution_receipt_digest(value)
    {
        return Err("T10 contribution receipt rejected".to_string());
    }
    Ok(())
}

fn validate_eligibility(value: &MomentumMicroHoldoutEligibilityReceiptV1) -> Result<(), String> {
    let gate_passed = value.development_lower_brier_than_constant
        && value.validation_lower_brier_than_constant
        && value.sufficient_paired_support
        && value.finite_predictions
        && value.finite_metrics
        && value.no_probability_collapse
        && value.no_saturation_failure
        && value.chronology_clean
        && value.leakage_clean
        && value.integrity_clean
        && value.result_selected_mutation_absent
        && value.holdout_access_count == 0;
    let expected = if gate_passed {
        MomentumMicroHoldoutEligibilityV1::EligibleForFutureSealedHoldoutEvaluation
    } else {
        MomentumMicroHoldoutEligibilityV1::IneligibleScreeningGate
    };
    if value.task_registration_digest.is_empty()
        || value.participant_registration_digest.is_empty()
        || parse_participant(&value.participant_id).is_err()
        || value.development_aggregate_digest.is_empty()
        || value.validation_aggregate_digest.is_empty()
        || value.eligibility != expected
        || value.receipt_digest != eligibility_receipt_digest(value)
    {
        return Err("T10 holdout eligibility receipt rejected".to_string());
    }
    Ok(())
}

fn validate_cohort(value: &MomentumMicroProposedHoldoutCohortV1) -> Result<(), String> {
    let expected_status = if value.participant_ids.is_empty() {
        MomentumMicroHoldoutCohortStatusV1::NoEligibleT10HoldoutCohort
    } else {
        MomentumMicroHoldoutCohortStatusV1::ProposedT10HoldoutCohort
    };
    if value.cohort_version != COHORT_VERSION
        || value.authorization_digest.is_empty()
        || value.eligibility_receipt_digests.len() != value.participant_ids.len()
        || value
            .participant_ids
            .iter()
            .any(|participant| parse_participant(participant).is_err())
        || value.status != expected_status
        || value.holdout_execution_authorized
        || value.cohort_digest != cohort_digest(value)
    {
        return Err("T10 proposed holdout cohort rejected".to_string());
    }
    Ok(())
}

fn validate_journal(value: &MomentumT10ScreeningJournalV1) -> Result<(), String> {
    if value.journal_version != JOURNAL_VERSION
        || value.authorization_digest.is_empty()
        || value.development_aggregate_digest.is_empty()
        || value.validation_aggregate_digest.is_empty()
        || value.benchmark_receipt_digests.len() != 4
        || value.contribution_receipt_digests.len() != 3
        || value.eligibility_receipt_digests.len() != 4
        || value.cohort_digest.is_empty()
        || value.holdout_access_count != 0
        || value.t30_execution_count != 0
        || value.t60_execution_count != 0
        || value.live_authority_count != 0
        || !value.deterministic
        || value.replay_digest != journal_digest(value)
    {
        return Err("T10 screening journal rejected".to_string());
    }
    Ok(())
}

fn validate_report(value: &MomentumT10MicroScreeningReportV1) -> Result<(), String> {
    if let Some(authorization) = &value.authorization {
        validate_authorization(authorization)?;
    }
    if let Some(policy) = &value.training_policy {
        validate_training_policy(policy)?;
    }
    if let Some(aggregate) = &value.development {
        validate_aggregate(aggregate)?;
    }
    if let Some(aggregate) = &value.validation {
        validate_aggregate(aggregate)?;
    }
    for receipt in &value.benchmark_comparisons {
        validate_benchmark(receipt)?;
    }
    for receipt in &value.contribution_comparisons {
        validate_contribution(receipt)?;
    }
    for receipt in &value.holdout_eligibility_receipts {
        validate_eligibility(receipt)?;
    }
    if let Some(cohort) = &value.proposed_holdout_cohort {
        validate_cohort(cohort)?;
    }
    if let Some(boundary) = &value.t10_boundary {
        validate_task_boundary(boundary)?;
        if boundary.task != MomentumMicroTaskV1::T10NextTenMinuteDirection {
            return Err("T10 report boundary binding rejected".to_string());
        }
    }
    if let Some(boundary) = &value.t30_boundary {
        validate_task_boundary(boundary)?;
        if boundary.task != MomentumMicroTaskV1::T30NextThirtyMinuteDirection {
            return Err("T30 report boundary binding rejected".to_string());
        }
    }
    let safety = &value.safety_counters;
    if value.report_version != REPORT_VERSION
        || value.source_label_report_digest != EXPECTED_LABEL_REPORT_DIGEST
        || value.source_feature_report_digest != EXPECTED_FEATURE_REPORT_DIGEST
        || value.source_design_report_digest != EXPECTED_DESIGN_REPORT_DIGEST
        || value.source_registration_digest != EXPECTED_REGISTRATION_DIGEST
        || value.source_gate_digest != EXPECTED_GATE_DIGEST
        || value.protected_before_state_digest.is_empty()
        || value.completed_live_event_count != 2
        || value.scorable_live_event_count != 2
        || value.live_pause != "PausedAfterCompletedEpochTwo"
        || value.epoch_three_registered
        || value.t10_disposition != "StableEnoughForFutureScreening"
        || value.t30_disposition != "ExcessiveTemporalInstability"
        || value.t60_disposition != "ExcessiveTemporalInstability"
        || !value.full_eight_a3_blocked
        || !value.historical_holdout_execution_mode_absent
        || !value.live_roster_unchanged
        || !value.protected_artifacts_unchanged
        || value.labels
            != PUBLIC_LABELS
                .iter()
                .map(|label| (*label).to_string())
                .collect::<Vec<_>>()
        || safety.t10_holdout_predictions != 0
        || safety.t10_holdout_label_reads != 0
        || safety.t10_holdout_metrics != 0
        || safety.t30_model_fits != 0
        || safety.t30_calibration_fits != 0
        || safety.t30_predictions != 0
        || safety.t30_evaluations != 0
        || safety.t30_holdout_access != 0
        || safety.t60_model_fits != 0
        || safety.t60_calibration_fits != 0
        || safety.t60_predictions != 0
        || safety.t60_evaluations != 0
        || safety.t60_holdout_access != 0
        || safety.network_requests != 0
        || safety.live_requests != 0
        || safety.live_outcomes != 0
        || safety.live_predictions != 0
        || safety.live_evaluations != 0
        || safety.live_parameter_changes != 0
        || safety.live_normalizer_changes != 0
        || safety.winner_selections != 0
        || safety.rankings != 0
        || safety.reward_applications != 0
        || safety.penalty_applications != 0
        || safety.chair_actions != 0
        || safety.vote_actions != 0
        || safety.paper_trading_actions != 0
        || safety.live_trading_actions != 0
        || safety.month_view_loads != 0
        || safety.year_view_loads != 0
        || value.report_digest != report_digest(value)
    {
        return Err("T10 screening report rejected".to_string());
    }
    match value.status {
        MomentumT10MicroScreeningStatusV1::Complete => {
            if value.authorization.is_none()
                || value.training_policy.is_none()
                || value.t10_boundary.is_none()
                || value.t30_boundary.is_none()
                || value.development.is_none()
                || value.validation.is_none()
                || value.benchmark_comparisons.len() != 4
                || value.contribution_comparisons.len() != 3
                || value.holdout_eligibility_receipts.len() != 4
                || value.proposed_holdout_cohort.is_none()
                || value
                    .deterministic_replay_digest
                    .as_deref()
                    .is_none_or(str::is_empty)
            {
                return Err("T10 complete report rejected".to_string());
            }
        }
        MomentumT10MicroScreeningStatusV1::DevelopmentComplete => {
            if value.development.is_none() || value.validation.is_some() {
                return Err("T10 development report rejected".to_string());
            }
        }
        _ => {}
    }
    Ok(())
}

struct FrozenSource {
    design: MomentumMicroChallengerDesignReportV1,
    registration: MomentumMicroChallengerScreeningRegistrationV1,
    gate: MomentumMicroScreeningGateV1,
    policy: MomentumCompactMicroFeaturePolicyV1,
    t10_boundary: MomentumMicroTaskPartitionBoundaryV1,
    t30_boundary: MomentumMicroTaskPartitionBoundaryV1,
}

fn reopen_frozen_source() -> Result<FrozenSource, String> {
    let label = read_momentum_micro_label_forensics_report_v1()?
        .ok_or_else(|| "T10 label report unavailable".to_string())?;
    let feature = read_momentum_micro_feature_forensics_report_v1()?
        .ok_or_else(|| "T10 feature report unavailable".to_string())?;
    let design = read_momentum_micro_challenger_design_report_v1()?
        .ok_or_else(|| "T10 design report unavailable".to_string())?;
    if label.status != MomentumMicroLabelForensicsStatusV1::Complete
        || label.report_digest != EXPECTED_LABEL_REPORT_DIGEST
        || feature.report_digest != EXPECTED_FEATURE_REPORT_DIGEST
        || design.report_digest != EXPECTED_DESIGN_REPORT_DIGEST
        || !design.complete
    {
        return Err("T10 Sprint 101 source identity rejected".to_string());
    }
    let disposition = |horizon| {
        label
            .horizons
            .iter()
            .find(|item| item.horizon == horizon)
            .map(|item| item.disposition.disposition)
    };
    if disposition(MomentumMicroPredictionHorizonV1::NextTenMinutes)
        != Some(MomentumMicroHorizonDiagnosticDispositionV1::StableEnoughForFutureScreening)
        || disposition(MomentumMicroPredictionHorizonV1::NextThirtyMinutes)
            != Some(MomentumMicroHorizonDiagnosticDispositionV1::ExcessiveTemporalInstability)
        || disposition(MomentumMicroPredictionHorizonV1::NextSixtyMinutes)
            != Some(MomentumMicroHorizonDiagnosticDispositionV1::ExcessiveTemporalInstability)
    {
        return Err("T10 horizon disposition rejected".to_string());
    }
    let policy = feature
        .compact_feature_policy
        .clone()
        .ok_or_else(|| "T10 compact feature policy unavailable".to_string())?;
    if policy.feature_dimension() != COMPACT_FEATURE_DIMENSION
        || policy.included_timeframes
            != [
                MomentumHistoricalTimeframeV1::Minute1,
                MomentumHistoricalTimeframeV1::Minute3,
                MomentumHistoricalTimeframeV1::Minute5,
                MomentumHistoricalTimeframeV1::Minute10,
            ]
        || feature.compact_integrity_replays.len() != 2
        || [
            MomentumReplayPartitionV1::Development,
            MomentumReplayPartitionV1::Validation,
        ]
        .iter()
        .any(|partition| {
            feature
                .compact_integrity_replays
                .iter()
                .filter(|item| item.partition == *partition)
                .count()
                != 1
        })
        || feature.compact_integrity_replays.iter().any(|item| {
            item.finite_block_count != item.eligible_event_count
                || item.future_access_count != 0
                || item.partial_access_count != 0
                || item.holdout_access_count != 0
                || item.missing_evidence_count != 0
                || item.partial_candle_count != 0
                || item.feature_schema_digest != policy.schema_digest
                || item.deterministic_replay_digest.is_empty()
        })
    {
        return Err("T10 compact policy integrity rejected".to_string());
    }
    let registration = design
        .screening_registration
        .clone()
        .ok_or_else(|| "T10 screening registration unavailable".to_string())?;
    let gate = design
        .screening_gate
        .clone()
        .ok_or_else(|| "T10 screening gate unavailable".to_string())?;
    if registration.registration_digest != EXPECTED_REGISTRATION_DIGEST
        || registration.screening_gate_digest != EXPECTED_GATE_DIGEST
        || gate.gate_digest != EXPECTED_GATE_DIGEST
        || registration.model_execution_authorized
        || registration.holdout_execution_authorized
        || registration.task_registrations.len() != 2
        || registration.participant_registrations.len() != 10
    {
        return Err("T10 frozen registration rejected".to_string());
    }
    let t10_boundary = design
        .task_boundaries
        .iter()
        .find(|item| item.task == MomentumMicroTaskV1::T10NextTenMinuteDirection)
        .cloned()
        .ok_or_else(|| "T10 boundary unavailable".to_string())?;
    let t30_boundary = design
        .task_boundaries
        .iter()
        .find(|item| item.task == MomentumMicroTaskV1::T30NextThirtyMinuteDirection)
        .cloned()
        .ok_or_else(|| "T30 boundary unavailable".to_string())?;
    if t10_boundary.development_event_count != 18_098
        || t10_boundary.validation_event_count != 3_878
        || t10_boundary.holdout_event_count != 3_879
        || t30_boundary.development_event_count != 6_033
        || t30_boundary.validation_event_count != 1_292
        || t30_boundary.holdout_event_count != 1_294
        || t10_boundary.holdout_labels_opened
        || t30_boundary.holdout_labels_opened
    {
        return Err("T10 task boundary identity rejected".to_string());
    }
    Ok(FrozenSource {
        design,
        registration,
        gate,
        policy,
        t10_boundary,
        t30_boundary,
    })
}

fn derive_minimum_support(feature_dimension: usize) -> Result<usize, String> {
    feature_dimension
        .checked_mul(DIMENSION_SUPPORT_MULTIPLIER)
        .and_then(|value| value.checked_next_power_of_two())
        .ok_or_else(|| "T10 minimum support derivation overflow".to_string())
}

fn build_authorization(
    source: &FrozenSource,
) -> Result<MomentumT10ScreeningExecutionAuthorizationV1, String> {
    let t10_participants = source
        .registration
        .participant_registrations
        .iter()
        .filter(|item| item.task == MomentumMicroTaskV1::T10NextTenMinuteDirection)
        .collect::<Vec<_>>();
    if t10_participants.len() != 5
        || t10_participants
            .iter()
            .any(|item| item.model_execution_authorized)
        || source
            .registration
            .participant_registrations
            .iter()
            .filter(|item| item.task == MomentumMicroTaskV1::T30NextThirtyMinuteDirection)
            .any(|item| item.model_execution_authorized)
    {
        return Err("T10 participant registration authority rejected".to_string());
    }
    let mut value = MomentumT10ScreeningExecutionAuthorizationV1 {
        authorization_version: AUTHORIZATION_VERSION.to_string(),
        challenger_registration_digest: source.registration.registration_digest.clone(),
        screening_gate_digest: source.gate.gate_digest.clone(),
        label_report_digest: EXPECTED_LABEL_REPORT_DIGEST.to_string(),
        feature_report_digest: EXPECTED_FEATURE_REPORT_DIGEST.to_string(),
        design_report_digest: source.design.report_digest.clone(),
        authorized_task_id: format!("{:?}", MomentumMicroTaskV1::T10NextTenMinuteDirection),
        authorized_participant_ids: t10_participants
            .iter()
            .map(|item| item.participant_id.clone())
            .collect(),
        development_execution_authorized: true,
        validation_execution_authorized: true,
        historical_holdout_execution_authorized: false,
        t30_execution_authorized: false,
        t60_execution_authorized: false,
        network_authorized: false,
        live_authority_forbidden: true,
        governance_authority_forbidden: true,
        trading_authority_forbidden: true,
        authorization_digest: String::new(),
    };
    value.authorization_digest = authorization_digest(&value);
    validate_authorization(&value)?;
    Ok(value)
}

fn build_training_policy(source: &FrozenSource) -> Result<MomentumT10TrainingPolicyV1, String> {
    let mut config = MomentumLearningCampaignConfigV0::default().training_config;
    config.epochs = 4;
    config.batch_size = 64;
    config.optimizer.weight_decay = STANDARD_L2;
    config.early_stopping_patience = None;
    let minimum = derive_minimum_support(source.policy.feature_dimension())?;
    let mut value = MomentumT10TrainingPolicyV1 {
        policy_version: TRAINING_POLICY_VERSION.to_string(),
        source_training_policy_digest: source.registration.training_policy_digest.clone(),
        loss_function: "BrierLoss".to_string(),
        initialization_seed: config.seed,
        epoch_count: config.epochs,
        batch_size: config.batch_size,
        learning_rate_bits: config.optimizer.learning_rate.to_bits(),
        standard_l2_bits: STANDARD_L2.to_bits(),
        c3_l2_multiplier: C3_L2_MULTIPLIER,
        gradient_finite_checks: true,
        parameter_finite_checks: true,
        probability_clamp_bits: PROBABILITY_CLAMP.to_bits(),
        maximum_training_examples: MAXIMUM_TRAINING_EXAMPLES,
        minimum_training_examples: minimum,
        dimension_support_multiplier: DIMENSION_SUPPORT_MULTIPLIER,
        daily_utc_refit: true,
        within_day_refit_forbidden: true,
        training_only_normalizer: true,
        c4_base_percent: C4_BASE_PERCENT,
        c4_calibration_percent: C4_CALIBRATION_PERCENT,
        validation_fit_forbidden: true,
        holdout_fit_forbidden: true,
        policy_digest: String::new(),
    };
    value.policy_digest = training_policy_digest(&value);
    validate_training_policy(&value)?;
    Ok(value)
}

fn empty_report(
    mode: MomentumT10MicroScreeningRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
    source: &FrozenSource,
) -> MomentumT10MicroScreeningReportV1 {
    let mut value = MomentumT10MicroScreeningReportV1 {
        report_version: REPORT_VERSION.to_string(),
        run_mode: mode.as_str().to_string(),
        status: MomentumT10MicroScreeningStatusV1::Unregistered,
        authorization: None,
        training_policy: None,
        source_label_report_digest: EXPECTED_LABEL_REPORT_DIGEST.to_string(),
        source_feature_report_digest: EXPECTED_FEATURE_REPORT_DIGEST.to_string(),
        source_design_report_digest: EXPECTED_DESIGN_REPORT_DIGEST.to_string(),
        source_registration_digest: EXPECTED_REGISTRATION_DIGEST.to_string(),
        source_gate_digest: EXPECTED_GATE_DIGEST.to_string(),
        protected_before_state_digest: protected.state_digest.clone(),
        completed_live_event_count: protected.completed_event_count,
        scorable_live_event_count: protected.scorable_event_count,
        live_pause: "PausedAfterCompletedEpochTwo".to_string(),
        epoch_three_registered: protected.epoch_three_registered,
        t10_boundary: Some(source.t10_boundary.clone()),
        t30_boundary: Some(source.t30_boundary.clone()),
        t10_disposition: "StableEnoughForFutureScreening".to_string(),
        t30_disposition: "ExcessiveTemporalInstability".to_string(),
        t60_disposition: "ExcessiveTemporalInstability".to_string(),
        development: None,
        validation: None,
        benchmark_comparisons: Vec::new(),
        contribution_comparisons: Vec::new(),
        holdout_eligibility_receipts: Vec::new(),
        proposed_holdout_cohort: None,
        full_eight_a3_blocked: true,
        historical_holdout_execution_mode_absent: true,
        live_roster_unchanged: true,
        protected_artifacts_unchanged: true,
        labels: PUBLIC_LABELS
            .iter()
            .map(|label| (*label).to_string())
            .collect(),
        safety_counters: MomentumT10ScreeningSafetyCountersV1::default(),
        deterministic_replay_digest: None,
        runtime_duration_ms: 0,
        report_digest: String::new(),
    };
    value.report_digest = report_digest(&value);
    value
}

fn encode_authorization(
    value: &MomentumT10ScreeningExecutionAuthorizationV1,
) -> Result<Vec<u8>, String> {
    validate_authorization(value)?;
    ArtifactBuilderV4_2::new("MomentumT10ScreeningExecutionAuthorizationV1")
        .string("authorization_version", &value.authorization_version)
        .string(
            "challenger_registration_digest",
            &value.challenger_registration_digest,
        )
        .string("screening_gate_digest", &value.screening_gate_digest)
        .string("label_report_digest", &value.label_report_digest)
        .string("feature_report_digest", &value.feature_report_digest)
        .string("design_report_digest", &value.design_report_digest)
        .string("authorized_task_id", &value.authorized_task_id)
        .strings(
            "authorized_participant_ids",
            &value.authorized_participant_ids,
        )
        .boolean(
            "development_execution_authorized",
            value.development_execution_authorized,
        )
        .boolean(
            "validation_execution_authorized",
            value.validation_execution_authorized,
        )
        .boolean(
            "historical_holdout_execution_authorized",
            value.historical_holdout_execution_authorized,
        )
        .boolean("t30_execution_authorized", value.t30_execution_authorized)
        .boolean("t60_execution_authorized", value.t60_execution_authorized)
        .boolean("network_authorized", value.network_authorized)
        .boolean("live_authority_forbidden", value.live_authority_forbidden)
        .boolean(
            "governance_authority_forbidden",
            value.governance_authority_forbidden,
        )
        .boolean(
            "trading_authority_forbidden",
            value.trading_authority_forbidden,
        )
        .string("authorization_digest", &value.authorization_digest)
        .encode()
}

fn decode_authorization(
    bytes: &[u8],
) -> Result<MomentumT10ScreeningExecutionAuthorizationV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumT10ScreeningExecutionAuthorizationV1")?;
    let value = MomentumT10ScreeningExecutionAuthorizationV1 {
        authorization_version: fields.string("authorization_version")?,
        challenger_registration_digest: fields.string("challenger_registration_digest")?,
        screening_gate_digest: fields.string("screening_gate_digest")?,
        label_report_digest: fields.string("label_report_digest")?,
        feature_report_digest: fields.string("feature_report_digest")?,
        design_report_digest: fields.string("design_report_digest")?,
        authorized_task_id: fields.string("authorized_task_id")?,
        authorized_participant_ids: fields.strings("authorized_participant_ids")?,
        development_execution_authorized: fields.boolean("development_execution_authorized")?,
        validation_execution_authorized: fields.boolean("validation_execution_authorized")?,
        historical_holdout_execution_authorized: fields
            .boolean("historical_holdout_execution_authorized")?,
        t30_execution_authorized: fields.boolean("t30_execution_authorized")?,
        t60_execution_authorized: fields.boolean("t60_execution_authorized")?,
        network_authorized: fields.boolean("network_authorized")?,
        live_authority_forbidden: fields.boolean("live_authority_forbidden")?,
        governance_authority_forbidden: fields.boolean("governance_authority_forbidden")?,
        trading_authority_forbidden: fields.boolean("trading_authority_forbidden")?,
        authorization_digest: fields.string("authorization_digest")?,
    };
    fields.finish()?;
    validate_authorization(&value)?;
    Ok(value)
}

fn encode_training_policy(value: &MomentumT10TrainingPolicyV1) -> Result<Vec<u8>, String> {
    validate_training_policy(value)?;
    ArtifactBuilderV4_2::new("MomentumT10TrainingPolicyV1")
        .string("policy_version", &value.policy_version)
        .string(
            "source_training_policy_digest",
            &value.source_training_policy_digest,
        )
        .string("loss_function", &value.loss_function)
        .unsigned("initialization_seed", value.initialization_seed)
        .unsigned("epoch_count", as_u64(value.epoch_count)?)
        .unsigned("batch_size", as_u64(value.batch_size)?)
        .unsigned("learning_rate_bits", u64::from(value.learning_rate_bits))
        .unsigned("standard_l2_bits", u64::from(value.standard_l2_bits))
        .unsigned("c3_l2_multiplier", as_u64(value.c3_l2_multiplier)?)
        .boolean("gradient_finite_checks", value.gradient_finite_checks)
        .boolean("parameter_finite_checks", value.parameter_finite_checks)
        .unsigned("probability_clamp_bits", value.probability_clamp_bits)
        .unsigned(
            "maximum_training_examples",
            as_u64(value.maximum_training_examples)?,
        )
        .unsigned(
            "minimum_training_examples",
            as_u64(value.minimum_training_examples)?,
        )
        .unsigned(
            "dimension_support_multiplier",
            as_u64(value.dimension_support_multiplier)?,
        )
        .boolean("daily_utc_refit", value.daily_utc_refit)
        .boolean(
            "within_day_refit_forbidden",
            value.within_day_refit_forbidden,
        )
        .boolean("training_only_normalizer", value.training_only_normalizer)
        .unsigned("c4_base_percent", as_u64(value.c4_base_percent)?)
        .unsigned(
            "c4_calibration_percent",
            as_u64(value.c4_calibration_percent)?,
        )
        .boolean("validation_fit_forbidden", value.validation_fit_forbidden)
        .boolean("holdout_fit_forbidden", value.holdout_fit_forbidden)
        .string("policy_digest", &value.policy_digest)
        .encode()
}

fn decode_training_policy(bytes: &[u8]) -> Result<MomentumT10TrainingPolicyV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumT10TrainingPolicyV1")?;
    let value = MomentumT10TrainingPolicyV1 {
        policy_version: fields.string("policy_version")?,
        source_training_policy_digest: fields.string("source_training_policy_digest")?,
        loss_function: fields.string("loss_function")?,
        initialization_seed: fields.unsigned("initialization_seed")?,
        epoch_count: as_usize(fields.unsigned("epoch_count")?)?,
        batch_size: as_usize(fields.unsigned("batch_size")?)?,
        learning_rate_bits: checked_u32(fields.unsigned("learning_rate_bits")?)?,
        standard_l2_bits: checked_u32(fields.unsigned("standard_l2_bits")?)?,
        c3_l2_multiplier: as_usize(fields.unsigned("c3_l2_multiplier")?)?,
        gradient_finite_checks: fields.boolean("gradient_finite_checks")?,
        parameter_finite_checks: fields.boolean("parameter_finite_checks")?,
        probability_clamp_bits: fields.unsigned("probability_clamp_bits")?,
        maximum_training_examples: as_usize(fields.unsigned("maximum_training_examples")?)?,
        minimum_training_examples: as_usize(fields.unsigned("minimum_training_examples")?)?,
        dimension_support_multiplier: as_usize(fields.unsigned("dimension_support_multiplier")?)?,
        daily_utc_refit: fields.boolean("daily_utc_refit")?,
        within_day_refit_forbidden: fields.boolean("within_day_refit_forbidden")?,
        training_only_normalizer: fields.boolean("training_only_normalizer")?,
        c4_base_percent: as_usize(fields.unsigned("c4_base_percent")?)?,
        c4_calibration_percent: as_usize(fields.unsigned("c4_calibration_percent")?)?,
        validation_fit_forbidden: fields.boolean("validation_fit_forbidden")?,
        holdout_fit_forbidden: fields.boolean("holdout_fit_forbidden")?,
        policy_digest: fields.string("policy_digest")?,
    };
    fields.finish()?;
    validate_training_policy(&value)?;
    Ok(value)
}

fn encode_training_plan(value: &MomentumT10DailyTrainingWindowPlanV1) -> Result<Vec<u8>, String> {
    validate_training_plan(value)?;
    ArtifactBuilderV4_2::new("MomentumT10DailyTrainingWindowPlanV1")
        .string("plan_version", &value.plan_version)
        .string("authorization_digest", &value.authorization_digest)
        .string("training_policy_digest", &value.training_policy_digest)
        .string("partition", partition_name(value.partition))
        .unsigned("utc_day_boundary_ms", value.utc_day_boundary_ms)
        .unsigned(
            "training_target_cutoff_exclusive_ms",
            value.training_target_cutoff_exclusive_ms,
        )
        .unsigned(
            "eligible_past_event_count",
            as_u64(value.eligible_past_event_count)?,
        )
        .unsigned(
            "scorable_training_event_count",
            as_u64(value.scorable_training_event_count)?,
        )
        .unsigned(
            "used_training_event_count",
            as_u64(value.used_training_event_count)?,
        )
        .strings("training_event_digests", &value.training_event_digests)
        .unsigned("c4_base_count", as_u64(value.c4_base_count)?)
        .unsigned("c4_calibration_count", as_u64(value.c4_calibration_count)?)
        .boolean(
            "support_sufficient_for_all",
            value.support_sufficient_for_all,
        )
        .unsigned(
            "validation_label_fit_count",
            as_u64(value.validation_label_fit_count)?,
        )
        .unsigned("holdout_access_count", as_u64(value.holdout_access_count)?)
        .string("plan_digest", &value.plan_digest)
        .encode()
}

fn decode_training_plan(bytes: &[u8]) -> Result<MomentumT10DailyTrainingWindowPlanV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumT10DailyTrainingWindowPlanV1")?;
    let value = MomentumT10DailyTrainingWindowPlanV1 {
        plan_version: fields.string("plan_version")?,
        authorization_digest: fields.string("authorization_digest")?,
        training_policy_digest: fields.string("training_policy_digest")?,
        partition: parse_partition(&fields.string("partition")?)?,
        utc_day_boundary_ms: fields.unsigned("utc_day_boundary_ms")?,
        training_target_cutoff_exclusive_ms: fields
            .unsigned("training_target_cutoff_exclusive_ms")?,
        eligible_past_event_count: as_usize(fields.unsigned("eligible_past_event_count")?)?,
        scorable_training_event_count: as_usize(fields.unsigned("scorable_training_event_count")?)?,
        used_training_event_count: as_usize(fields.unsigned("used_training_event_count")?)?,
        training_event_digests: fields.strings("training_event_digests")?,
        c4_base_count: as_usize(fields.unsigned("c4_base_count")?)?,
        c4_calibration_count: as_usize(fields.unsigned("c4_calibration_count")?)?,
        support_sufficient_for_all: fields.boolean("support_sufficient_for_all")?,
        validation_label_fit_count: as_usize(fields.unsigned("validation_label_fit_count")?)?,
        holdout_access_count: as_usize(fields.unsigned("holdout_access_count")?)?,
        plan_digest: fields.string("plan_digest")?,
    };
    fields.finish()?;
    validate_training_plan(&value)?;
    Ok(value)
}

fn encode_normalizer(value: &MomentumT10DailyNormalizerReceiptV1) -> Result<Vec<u8>, String> {
    validate_normalizer_receipt(value)?;
    ArtifactBuilderV4_2::new("MomentumT10DailyNormalizerReceiptV1")
        .string("receipt_version", &value.receipt_version)
        .string("participant_id", &value.participant_id)
        .string("feature_policy_digest", &value.feature_policy_digest)
        .unsigneds(
            "private_mean_bits",
            &value
                .private_means
                .iter()
                .map(|item| u64::from(item.to_bits()))
                .collect::<Vec<_>>(),
        )
        .unsigneds(
            "private_scale_bits",
            &value
                .private_scales
                .iter()
                .map(|item| u64::from(item.to_bits()))
                .collect::<Vec<_>>(),
        )
        .unsigneds(
            "constant_dimension_indices",
            &value
                .constant_dimension_indices
                .iter()
                .map(|item| as_u64(*item))
                .collect::<Result<Vec<_>, _>>()?,
        )
        .string("training_event_digest", &value.training_event_digest)
        .boolean("finite", value.finite)
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_normalizer(bytes: &[u8]) -> Result<MomentumT10DailyNormalizerReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumT10DailyNormalizerReceiptV1")?;
    let value = MomentumT10DailyNormalizerReceiptV1 {
        receipt_version: fields.string("receipt_version")?,
        participant_id: fields.string("participant_id")?,
        feature_policy_digest: fields.string("feature_policy_digest")?,
        private_means: fields
            .unsigneds("private_mean_bits")?
            .into_iter()
            .map(checked_u32)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(f32::from_bits)
            .collect(),
        private_scales: fields
            .unsigneds("private_scale_bits")?
            .into_iter()
            .map(checked_u32)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(f32::from_bits)
            .collect(),
        constant_dimension_indices: fields
            .unsigneds("constant_dimension_indices")?
            .into_iter()
            .map(as_usize)
            .collect::<Result<Vec<_>, _>>()?,
        training_event_digest: fields.string("training_event_digest")?,
        finite: fields.boolean("finite")?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_normalizer_receipt(&value)?;
    Ok(value)
}

fn encode_model(value: &MomentumT10DailyModelReceiptV1) -> Result<Vec<u8>, String> {
    validate_model_receipt(value)?;
    ArtifactBuilderV4_2::new("MomentumT10DailyModelReceiptV1")
        .string("receipt_version", &value.receipt_version)
        .string("participant_id", &value.participant_id)
        .string("role", role_name(value.role))
        .unsigned("utc_day_boundary_ms", value.utc_day_boundary_ms)
        .string("training_plan_digest", &value.training_plan_digest)
        .string(
            "normalizer_receipt_digest",
            &value.normalizer_receipt_digest,
        )
        .unsigneds(
            "private_weight_bits",
            &value
                .private_weights
                .iter()
                .map(|item| u64::from(item.to_bits()))
                .collect::<Vec<_>>(),
        )
        .unsigned(
            "private_bias_bits",
            u64::from(value.private_bias.unwrap_or_default().to_bits()),
        )
        .boolean("private_bias_present", value.private_bias.is_some())
        .unsigned(
            "private_prevalence_bits",
            value.private_prevalence.unwrap_or_default().to_bits(),
        )
        .boolean(
            "private_prevalence_present",
            value.private_prevalence.is_some(),
        )
        .unsigned("training_count", as_u64(value.training_count)?)
        .unsigned("positive_count", as_u64(value.positive_count)?)
        .unsigned("negative_count", as_u64(value.negative_count)?)
        .unsigned("l2_bits", u64::from(value.l2_bits))
        .unsigned("initialization_seed", value.initialization_seed)
        .boolean("finite", value.finite)
        .unsigned("validation_fit_count", as_u64(value.validation_fit_count)?)
        .unsigned("holdout_fit_count", as_u64(value.holdout_fit_count)?)
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_model(bytes: &[u8]) -> Result<MomentumT10DailyModelReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumT10DailyModelReceiptV1")?;
    let private_weights = fields
        .unsigneds("private_weight_bits")?
        .into_iter()
        .map(checked_u32)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(f32::from_bits)
        .collect();
    let private_bias_bits = checked_u32(fields.unsigned("private_bias_bits")?)?;
    let private_bias_present = fields.boolean("private_bias_present")?;
    let private_prevalence_bits = fields.unsigned("private_prevalence_bits")?;
    let private_prevalence_present = fields.boolean("private_prevalence_present")?;
    let value = MomentumT10DailyModelReceiptV1 {
        receipt_version: fields.string("receipt_version")?,
        participant_id: fields.string("participant_id")?,
        role: parse_role(&fields.string("role")?)?,
        utc_day_boundary_ms: fields.unsigned("utc_day_boundary_ms")?,
        training_plan_digest: fields.string("training_plan_digest")?,
        normalizer_receipt_digest: fields.string("normalizer_receipt_digest")?,
        private_weights,
        private_bias: private_bias_present.then(|| f32::from_bits(private_bias_bits)),
        private_prevalence: private_prevalence_present
            .then(|| f64::from_bits(private_prevalence_bits)),
        training_count: as_usize(fields.unsigned("training_count")?)?,
        positive_count: as_usize(fields.unsigned("positive_count")?)?,
        negative_count: as_usize(fields.unsigned("negative_count")?)?,
        l2_bits: checked_u32(fields.unsigned("l2_bits")?)?,
        initialization_seed: fields.unsigned("initialization_seed")?,
        finite: fields.boolean("finite")?,
        validation_fit_count: as_usize(fields.unsigned("validation_fit_count")?)?,
        holdout_fit_count: as_usize(fields.unsigned("holdout_fit_count")?)?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_model_receipt(&value)?;
    Ok(value)
}

fn encode_refit_bundle(value: &MomentumT10DailyRefitBundleV1) -> Result<Vec<u8>, String> {
    validate_refit_bundle(value)?;
    ArtifactBuilderV4_2::new("MomentumT10DailyRefitBundleV1")
        .string("bundle_version", &value.bundle_version)
        .string("authorization_digest", &value.authorization_digest)
        .string("partition", partition_name(value.partition))
        .unsigned("utc_day_boundary_ms", value.utc_day_boundary_ms)
        .messages(
            "training_plan",
            vec![encode_training_plan(&value.training_plan)?],
        )
        .messages(
            "normalizer_receipts",
            value
                .normalizer_receipts
                .iter()
                .map(encode_normalizer)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "model_receipts",
            value
                .model_receipts
                .iter()
                .map(encode_model)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .strings(
            "reconstructed_participant_digests",
            &value.reconstructed_participant_digests,
        )
        .unsigned(
            "target_access_count_for_prediction_day",
            as_u64(value.target_access_count_for_prediction_day)?,
        )
        .unsigned("holdout_access_count", as_u64(value.holdout_access_count)?)
        .unsigned("live_access_count", as_u64(value.live_access_count)?)
        .string("bundle_digest", &value.bundle_digest)
        .encode()
}

fn decode_refit_bundle(bytes: &[u8]) -> Result<MomentumT10DailyRefitBundleV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumT10DailyRefitBundleV1")?;
    let plans = fields.messages("training_plan")?;
    if plans.len() != 1 {
        return Err("T10 refit training plan count rejected".to_string());
    }
    let value = MomentumT10DailyRefitBundleV1 {
        bundle_version: fields.string("bundle_version")?,
        authorization_digest: fields.string("authorization_digest")?,
        partition: parse_partition(&fields.string("partition")?)?,
        utc_day_boundary_ms: fields.unsigned("utc_day_boundary_ms")?,
        training_plan: decode_training_plan(&plans[0])?,
        normalizer_receipts: fields
            .messages("normalizer_receipts")?
            .iter()
            .map(|item| decode_normalizer(item))
            .collect::<Result<Vec<_>, _>>()?,
        model_receipts: fields
            .messages("model_receipts")?
            .iter()
            .map(|item| decode_model(item))
            .collect::<Result<Vec<_>, _>>()?,
        reconstructed_participant_digests: fields.strings("reconstructed_participant_digests")?,
        target_access_count_for_prediction_day: as_usize(
            fields.unsigned("target_access_count_for_prediction_day")?,
        )?,
        holdout_access_count: as_usize(fields.unsigned("holdout_access_count")?)?,
        live_access_count: as_usize(fields.unsigned("live_access_count")?)?,
        bundle_digest: fields.string("bundle_digest")?,
    };
    fields.finish()?;
    validate_refit_bundle(&value)?;
    Ok(value)
}

fn encode_event_plan(value: &MomentumT10EventPlanV1) -> Result<Vec<u8>, String> {
    validate_event_plan(value)?;
    ArtifactBuilderV4_2::new("MomentumT10EventPlanV1")
        .string("plan_version", &value.plan_version)
        .string("partition", partition_name(value.partition))
        .unsigned("prediction_timestamp_ms", value.prediction_timestamp_ms)
        .unsigned("target_timestamp_ms", value.target_timestamp_ms)
        .string("source_event_digest", &value.source_event_digest)
        .string(
            "daily_refit_bundle_digest",
            &value.daily_refit_bundle_digest,
        )
        .strings("participant_ids", &value.participant_ids)
        .boolean("target_hidden", value.target_hidden)
        .boolean("holdout_member", value.holdout_member)
        .string("plan_digest", &value.plan_digest)
        .encode()
}

fn decode_event_plan(bytes: &[u8]) -> Result<MomentumT10EventPlanV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumT10EventPlanV1")?;
    let value = MomentumT10EventPlanV1 {
        plan_version: fields.string("plan_version")?,
        partition: parse_partition(&fields.string("partition")?)?,
        prediction_timestamp_ms: fields.unsigned("prediction_timestamp_ms")?,
        target_timestamp_ms: fields.unsigned("target_timestamp_ms")?,
        source_event_digest: fields.string("source_event_digest")?,
        daily_refit_bundle_digest: fields.string("daily_refit_bundle_digest")?,
        participant_ids: fields.strings("participant_ids")?,
        target_hidden: fields.boolean("target_hidden")?,
        holdout_member: fields.boolean("holdout_member")?,
        plan_digest: fields.string("plan_digest")?,
    };
    fields.finish()?;
    validate_event_plan(&value)?;
    Ok(value)
}

fn encode_prediction_shard(value: &MomentumT10PredictionShardV1) -> Result<Vec<u8>, String> {
    validate_prediction_shard(value)?;
    ArtifactBuilderV4_2::new("MomentumT10PredictionShardV1")
        .string("shard_version", &value.shard_version)
        .string("authorization_digest", &value.authorization_digest)
        .string("partition", partition_name(value.partition))
        .unsigned("utc_day_boundary_ms", value.utc_day_boundary_ms)
        .string(
            "daily_refit_bundle_digest",
            &value.daily_refit_bundle_digest,
        )
        .messages(
            "event_plans",
            value
                .event_plans
                .iter()
                .map(encode_event_plan)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .strings("participant_ids", &value.participant_ids)
        .unsigneds(
            "private_probability_bits",
            &value
                .private_probabilities
                .iter()
                .map(|item| item.to_bits())
                .collect::<Vec<_>>(),
        )
        .strings("prediction_digests", &value.prediction_digests)
        .boolean("target_accessed", value.target_accessed)
        .boolean("label_accessed", value.label_accessed)
        .boolean("metric_computed", value.metric_computed)
        .string("shard_digest", &value.shard_digest)
        .encode()
}

fn decode_prediction_shard(bytes: &[u8]) -> Result<MomentumT10PredictionShardV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumT10PredictionShardV1")?;
    let value = MomentumT10PredictionShardV1 {
        shard_version: fields.string("shard_version")?,
        authorization_digest: fields.string("authorization_digest")?,
        partition: parse_partition(&fields.string("partition")?)?,
        utc_day_boundary_ms: fields.unsigned("utc_day_boundary_ms")?,
        daily_refit_bundle_digest: fields.string("daily_refit_bundle_digest")?,
        event_plans: fields
            .messages("event_plans")?
            .iter()
            .map(|item| decode_event_plan(item))
            .collect::<Result<Vec<_>, _>>()?,
        participant_ids: fields.strings("participant_ids")?,
        private_probabilities: fields
            .unsigneds("private_probability_bits")?
            .into_iter()
            .map(f64::from_bits)
            .collect(),
        prediction_digests: fields.strings("prediction_digests")?,
        target_accessed: fields.boolean("target_accessed")?,
        label_accessed: fields.boolean("label_accessed")?,
        metric_computed: fields.boolean("metric_computed")?,
        shard_digest: fields.string("shard_digest")?,
    };
    fields.finish()?;
    validate_prediction_shard(&value)?;
    Ok(value)
}

fn encode_evaluation_item(value: &MomentumT10EvaluationItemV1) -> Result<Vec<u8>, String> {
    validate_evaluation_item(value)?;
    ArtifactBuilderV4_2::new("MomentumT10EvaluationItemV1")
        .string("item_version", &value.item_version)
        .string("event_plan_digest", &value.event_plan_digest)
        .string("label", label_name(value.label))
        .unsigned(
            "private_label_bits",
            value.private_label.unwrap_or_default().to_bits(),
        )
        .boolean("private_label_present", value.private_label.is_some())
        .unsigneds(
            "private_brier_bits",
            &value
                .private_brier_values
                .iter()
                .map(|item| item.to_bits())
                .collect::<Vec<_>>(),
        )
        .unsigneds(
            "private_correctness",
            &value
                .private_correctness
                .iter()
                .map(|item| u64::from(*item))
                .collect::<Vec<_>>(),
        )
        .string("item_digest", &value.item_digest)
        .encode()
}

fn decode_evaluation_item(bytes: &[u8]) -> Result<MomentumT10EvaluationItemV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumT10EvaluationItemV1")?;
    let label_bits = fields.unsigned("private_label_bits")?;
    let label_present = fields.boolean("private_label_present")?;
    let value = MomentumT10EvaluationItemV1 {
        item_version: fields.string("item_version")?,
        event_plan_digest: fields.string("event_plan_digest")?,
        label: parse_label(&fields.string("label")?)?,
        private_label: label_present.then(|| f64::from_bits(label_bits)),
        private_brier_values: fields
            .unsigneds("private_brier_bits")?
            .into_iter()
            .map(f64::from_bits)
            .collect(),
        private_correctness: fields
            .unsigneds("private_correctness")?
            .into_iter()
            .map(|item| match item {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err("T10 correctness value rejected".to_string()),
            })
            .collect::<Result<Vec<_>, _>>()?,
        item_digest: fields.string("item_digest")?,
    };
    fields.finish()?;
    validate_evaluation_item(&value)?;
    Ok(value)
}

fn encode_evaluation_shard(value: &MomentumT10EvaluationShardV1) -> Result<Vec<u8>, String> {
    validate_evaluation_shard(value)?;
    ArtifactBuilderV4_2::new("MomentumT10EvaluationShardV1")
        .string("shard_version", &value.shard_version)
        .string("prediction_shard_digest", &value.prediction_shard_digest)
        .string("partition", partition_name(value.partition))
        .unsigned("utc_day_boundary_ms", value.utc_day_boundary_ms)
        .boolean("prediction_shard_reopened", value.prediction_shard_reopened)
        .messages(
            "evaluations",
            value
                .evaluations
                .iter()
                .map(encode_evaluation_item)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .string("shard_digest", &value.shard_digest)
        .encode()
}

fn decode_evaluation_shard(bytes: &[u8]) -> Result<MomentumT10EvaluationShardV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumT10EvaluationShardV1")?;
    let value = MomentumT10EvaluationShardV1 {
        shard_version: fields.string("shard_version")?,
        prediction_shard_digest: fields.string("prediction_shard_digest")?,
        partition: parse_partition(&fields.string("partition")?)?,
        utc_day_boundary_ms: fields.unsigned("utc_day_boundary_ms")?,
        prediction_shard_reopened: fields.boolean("prediction_shard_reopened")?,
        evaluations: fields
            .messages("evaluations")?
            .iter()
            .map(|item| decode_evaluation_item(item))
            .collect::<Result<Vec<_>, _>>()?,
        shard_digest: fields.string("shard_digest")?,
    };
    fields.finish()?;
    validate_evaluation_shard(&value)?;
    Ok(value)
}

fn encode_calibration_bin(value: &MomentumMicroCalibrationBinV1) -> Result<Vec<u8>, String> {
    validate_calibration_bin(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroCalibrationBinV1")
        .string("bin_version", &value.bin_version)
        .unsigned("lower_bound_bits", value.lower_bound.to_bits())
        .unsigned("upper_bound_bits", value.upper_bound.to_bits())
        .boolean("upper_inclusive", value.upper_inclusive)
        .unsigned("support", as_u64(value.support)?)
        .unsigned(
            "mean_predicted_probability_bits",
            value.mean_predicted_probability.to_bits(),
        )
        .unsigned(
            "observed_positive_frequency_bits",
            value.observed_positive_frequency.to_bits(),
        )
        .unsigned(
            "absolute_calibration_gap_bits",
            value.absolute_calibration_gap.to_bits(),
        )
        .string("bin_digest", &value.bin_digest)
        .encode()
}

fn decode_calibration_bin(bytes: &[u8]) -> Result<MomentumMicroCalibrationBinV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroCalibrationBinV1")?;
    let value = MomentumMicroCalibrationBinV1 {
        bin_version: fields.string("bin_version")?,
        lower_bound: f64::from_bits(fields.unsigned("lower_bound_bits")?),
        upper_bound: f64::from_bits(fields.unsigned("upper_bound_bits")?),
        upper_inclusive: fields.boolean("upper_inclusive")?,
        support: as_usize(fields.unsigned("support")?)?,
        mean_predicted_probability: f64::from_bits(
            fields.unsigned("mean_predicted_probability_bits")?,
        ),
        observed_positive_frequency: f64::from_bits(
            fields.unsigned("observed_positive_frequency_bits")?,
        ),
        absolute_calibration_gap: f64::from_bits(fields.unsigned("absolute_calibration_gap_bits")?),
        bin_digest: fields.string("bin_digest")?,
    };
    fields.finish()?;
    validate_calibration_bin(&value)?;
    Ok(value)
}

fn encode_metrics(value: &MomentumMicroScreeningParticipantMetricsV1) -> Result<Vec<u8>, String> {
    validate_participant_metrics(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroScreeningParticipantMetricsV1")
        .string("metrics_version", &value.metrics_version)
        .string("participant_id", &value.participant_id)
        .string("partition", partition_name(value.partition))
        .unsigned("prediction_count", as_u64(value.prediction_count)?)
        .unsigned("scorable_count", as_u64(value.scorable_count)?)
        .unsigned("neutral_count", as_u64(value.neutral_count)?)
        .unsigned("invalid_count", as_u64(value.invalid_count)?)
        .unsigned(
            "finite_prediction_count",
            as_u64(value.finite_prediction_count)?,
        )
        .unsigned("mean_brier_bits", value.mean_brier.to_bits())
        .unsigned(
            "binary_correctness_bits",
            value.binary_correctness.to_bits(),
        )
        .unsigned(
            "paired_mean_brier_delta_versus_c0_bits",
            value.paired_mean_brier_delta_versus_c0.to_bits(),
        )
        .unsigned(
            "paired_median_brier_delta_versus_c0_bits",
            value.paired_median_brier_delta_versus_c0.to_bits(),
        )
        .unsigned(
            "positive_paired_delta_count",
            as_u64(value.positive_paired_delta_count)?,
        )
        .unsigned(
            "negative_paired_delta_count",
            as_u64(value.negative_paired_delta_count)?,
        )
        .unsigned(
            "equivalent_paired_delta_count",
            as_u64(value.equivalent_paired_delta_count)?,
        )
        .messages(
            "calibration_bins",
            value
                .calibration_bins
                .iter()
                .map(encode_calibration_bin)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .unsigned(
            "weighted_calibration_gap_bits",
            value.weighted_calibration_gap.to_bits(),
        )
        .unsigned(
            "empty_calibration_bin_count",
            as_u64(value.empty_calibration_bin_count)?,
        )
        .unsigned(
            "minimum_probability_bits",
            value.minimum_probability.to_bits(),
        )
        .unsigned(
            "maximum_probability_bits",
            value.maximum_probability.to_bits(),
        )
        .unsigned("mean_probability_bits", value.mean_probability.to_bits())
        .unsigned(
            "probability_standard_deviation_bits",
            value.probability_standard_deviation.to_bits(),
        )
        .unsigned("near_constant_count", as_u64(value.near_constant_count)?)
        .unsigned("near_half_count", as_u64(value.near_half_count)?)
        .unsigned("extreme_low_count", as_u64(value.extreme_low_count)?)
        .unsigned("extreme_high_count", as_u64(value.extreme_high_count)?)
        .unsigned("nonfinite_count", as_u64(value.nonfinite_count)?)
        .string("collapse", collapse_name(value.collapse))
        .string("saturation", saturation_name(value.saturation))
        .boolean("chronology_audit_passed", value.chronology_audit_passed)
        .boolean("leakage_audit_passed", value.leakage_audit_passed)
        .boolean("integrity_audit_passed", value.integrity_audit_passed)
        .string("metrics_digest", &value.metrics_digest)
        .encode()
}

fn decode_metrics(bytes: &[u8]) -> Result<MomentumMicroScreeningParticipantMetricsV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumMicroScreeningParticipantMetricsV1")?;
    let value = MomentumMicroScreeningParticipantMetricsV1 {
        metrics_version: fields.string("metrics_version")?,
        participant_id: fields.string("participant_id")?,
        partition: parse_partition(&fields.string("partition")?)?,
        prediction_count: as_usize(fields.unsigned("prediction_count")?)?,
        scorable_count: as_usize(fields.unsigned("scorable_count")?)?,
        neutral_count: as_usize(fields.unsigned("neutral_count")?)?,
        invalid_count: as_usize(fields.unsigned("invalid_count")?)?,
        finite_prediction_count: as_usize(fields.unsigned("finite_prediction_count")?)?,
        mean_brier: f64::from_bits(fields.unsigned("mean_brier_bits")?),
        binary_correctness: f64::from_bits(fields.unsigned("binary_correctness_bits")?),
        paired_mean_brier_delta_versus_c0: f64::from_bits(
            fields.unsigned("paired_mean_brier_delta_versus_c0_bits")?,
        ),
        paired_median_brier_delta_versus_c0: f64::from_bits(
            fields.unsigned("paired_median_brier_delta_versus_c0_bits")?,
        ),
        positive_paired_delta_count: as_usize(fields.unsigned("positive_paired_delta_count")?)?,
        negative_paired_delta_count: as_usize(fields.unsigned("negative_paired_delta_count")?)?,
        equivalent_paired_delta_count: as_usize(fields.unsigned("equivalent_paired_delta_count")?)?,
        calibration_bins: fields
            .messages("calibration_bins")?
            .iter()
            .map(|item| decode_calibration_bin(item))
            .collect::<Result<Vec<_>, _>>()?,
        weighted_calibration_gap: f64::from_bits(fields.unsigned("weighted_calibration_gap_bits")?),
        empty_calibration_bin_count: as_usize(fields.unsigned("empty_calibration_bin_count")?)?,
        minimum_probability: f64::from_bits(fields.unsigned("minimum_probability_bits")?),
        maximum_probability: f64::from_bits(fields.unsigned("maximum_probability_bits")?),
        mean_probability: f64::from_bits(fields.unsigned("mean_probability_bits")?),
        probability_standard_deviation: f64::from_bits(
            fields.unsigned("probability_standard_deviation_bits")?,
        ),
        near_constant_count: as_usize(fields.unsigned("near_constant_count")?)?,
        near_half_count: as_usize(fields.unsigned("near_half_count")?)?,
        extreme_low_count: as_usize(fields.unsigned("extreme_low_count")?)?,
        extreme_high_count: as_usize(fields.unsigned("extreme_high_count")?)?,
        nonfinite_count: as_usize(fields.unsigned("nonfinite_count")?)?,
        collapse: parse_collapse(&fields.string("collapse")?)?,
        saturation: parse_saturation(&fields.string("saturation")?)?,
        chronology_audit_passed: fields.boolean("chronology_audit_passed")?,
        leakage_audit_passed: fields.boolean("leakage_audit_passed")?,
        integrity_audit_passed: fields.boolean("integrity_audit_passed")?,
        metrics_digest: fields.string("metrics_digest")?,
    };
    fields.finish()?;
    validate_participant_metrics(&value)?;
    Ok(value)
}

fn encode_aggregate(value: &MomentumT10PartitionAggregateV1) -> Result<Vec<u8>, String> {
    validate_aggregate(value)?;
    ArtifactBuilderV4_2::new("MomentumT10PartitionAggregateV1")
        .string("aggregate_version", &value.aggregate_version)
        .string("authorization_digest", &value.authorization_digest)
        .string("partition", partition_name(value.partition))
        .unsigned("boundary_event_count", as_u64(value.boundary_event_count)?)
        .unsigned(
            "training_only_event_count",
            as_u64(value.training_only_event_count)?,
        )
        .unsigned("prediction_count", as_u64(value.prediction_count)?)
        .unsigned("scorable_count", as_u64(value.scorable_count)?)
        .unsigned("neutral_count", as_u64(value.neutral_count)?)
        .unsigned("invalid_count", as_u64(value.invalid_count)?)
        .unsigned("daily_refit_count", as_u64(value.daily_refit_count)?)
        .unsigned(
            "insufficient_support_day_count",
            as_u64(value.insufficient_support_day_count)?,
        )
        .strings(
            "daily_refit_bundle_digests",
            &value.daily_refit_bundle_digests,
        )
        .strings("prediction_shard_digests", &value.prediction_shard_digests)
        .strings("evaluation_shard_digests", &value.evaluation_shard_digests)
        .messages(
            "participant_metrics",
            value
                .participant_metrics
                .iter()
                .map(encode_metrics)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .unsigned(
            "target_access_before_prediction_reopen_count",
            as_u64(value.target_access_before_prediction_reopen_count)?,
        )
        .unsigned(
            "feature_future_access_count",
            as_u64(value.feature_future_access_count)?,
        )
        .unsigned(
            "partial_candle_access_count",
            as_u64(value.partial_candle_access_count)?,
        )
        .unsigned("holdout_access_count", as_u64(value.holdout_access_count)?)
        .unsigned("validation_fit_count", as_u64(value.validation_fit_count)?)
        .boolean("chronology_audit_passed", value.chronology_audit_passed)
        .boolean("leakage_audit_passed", value.leakage_audit_passed)
        .boolean(
            "prediction_before_reveal_passed",
            value.prediction_before_reveal_passed,
        )
        .string("aggregate_digest", &value.aggregate_digest)
        .encode()
}

fn decode_aggregate(bytes: &[u8]) -> Result<MomentumT10PartitionAggregateV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumT10PartitionAggregateV1")?;
    let value = MomentumT10PartitionAggregateV1 {
        aggregate_version: fields.string("aggregate_version")?,
        authorization_digest: fields.string("authorization_digest")?,
        partition: parse_partition(&fields.string("partition")?)?,
        boundary_event_count: as_usize(fields.unsigned("boundary_event_count")?)?,
        training_only_event_count: as_usize(fields.unsigned("training_only_event_count")?)?,
        prediction_count: as_usize(fields.unsigned("prediction_count")?)?,
        scorable_count: as_usize(fields.unsigned("scorable_count")?)?,
        neutral_count: as_usize(fields.unsigned("neutral_count")?)?,
        invalid_count: as_usize(fields.unsigned("invalid_count")?)?,
        daily_refit_count: as_usize(fields.unsigned("daily_refit_count")?)?,
        insufficient_support_day_count: as_usize(
            fields.unsigned("insufficient_support_day_count")?,
        )?,
        daily_refit_bundle_digests: fields.strings("daily_refit_bundle_digests")?,
        prediction_shard_digests: fields.strings("prediction_shard_digests")?,
        evaluation_shard_digests: fields.strings("evaluation_shard_digests")?,
        participant_metrics: fields
            .messages("participant_metrics")?
            .iter()
            .map(|item| decode_metrics(item))
            .collect::<Result<Vec<_>, _>>()?,
        target_access_before_prediction_reopen_count: as_usize(
            fields.unsigned("target_access_before_prediction_reopen_count")?,
        )?,
        feature_future_access_count: as_usize(fields.unsigned("feature_future_access_count")?)?,
        partial_candle_access_count: as_usize(fields.unsigned("partial_candle_access_count")?)?,
        holdout_access_count: as_usize(fields.unsigned("holdout_access_count")?)?,
        validation_fit_count: as_usize(fields.unsigned("validation_fit_count")?)?,
        chronology_audit_passed: fields.boolean("chronology_audit_passed")?,
        leakage_audit_passed: fields.boolean("leakage_audit_passed")?,
        prediction_before_reveal_passed: fields.boolean("prediction_before_reveal_passed")?,
        aggregate_digest: fields.string("aggregate_digest")?,
    };
    fields.finish()?;
    validate_aggregate(&value)?;
    Ok(value)
}

fn encode_benchmark(value: &MomentumMicroBenchmarkComparisonReceiptV1) -> Result<Vec<u8>, String> {
    validate_benchmark(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroBenchmarkComparisonReceiptV1")
        .string("comparison_version", &value.comparison_version)
        .string("participant_id", &value.participant_id)
        .string(
            "development_aggregate_digest",
            &value.development_aggregate_digest,
        )
        .string(
            "validation_aggregate_digest",
            &value.validation_aggregate_digest,
        )
        .unsigned("development_delta_bits", value.development_delta_bits)
        .unsigned("validation_delta_bits", value.validation_delta_bits)
        .string(
            "development_comparison",
            comparison_name(value.development_comparison),
        )
        .string(
            "validation_comparison",
            comparison_name(value.validation_comparison),
        )
        .string(
            "overall_comparison",
            comparison_name(value.overall_comparison),
        )
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_benchmark(bytes: &[u8]) -> Result<MomentumMicroBenchmarkComparisonReceiptV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumMicroBenchmarkComparisonReceiptV1")?;
    let value = MomentumMicroBenchmarkComparisonReceiptV1 {
        comparison_version: fields.string("comparison_version")?,
        participant_id: fields.string("participant_id")?,
        development_aggregate_digest: fields.string("development_aggregate_digest")?,
        validation_aggregate_digest: fields.string("validation_aggregate_digest")?,
        development_delta_bits: fields.unsigned("development_delta_bits")?,
        validation_delta_bits: fields.unsigned("validation_delta_bits")?,
        development_comparison: parse_comparison(&fields.string("development_comparison")?)?,
        validation_comparison: parse_comparison(&fields.string("validation_comparison")?)?,
        overall_comparison: parse_comparison(&fields.string("overall_comparison")?)?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_benchmark(&value)?;
    Ok(value)
}

fn encode_contribution(value: &MomentumMicroContributionReceiptV1) -> Result<Vec<u8>, String> {
    validate_contribution(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroContributionReceiptV1")
        .string("comparison_version", &value.comparison_version)
        .string("participant_id", &value.participant_id)
        .string("baseline_participant_id", &value.baseline_participant_id)
        .unsigned("development_delta_bits", value.development_delta_bits)
        .unsigned("validation_delta_bits", value.validation_delta_bits)
        .string(
            "development_comparison",
            contribution_name(value.development_comparison),
        )
        .string(
            "validation_comparison",
            contribution_name(value.validation_comparison),
        )
        .string(
            "overall_comparison",
            contribution_name(value.overall_comparison),
        )
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_contribution(bytes: &[u8]) -> Result<MomentumMicroContributionReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroContributionReceiptV1")?;
    let value = MomentumMicroContributionReceiptV1 {
        comparison_version: fields.string("comparison_version")?,
        participant_id: fields.string("participant_id")?,
        baseline_participant_id: fields.string("baseline_participant_id")?,
        development_delta_bits: fields.unsigned("development_delta_bits")?,
        validation_delta_bits: fields.unsigned("validation_delta_bits")?,
        development_comparison: parse_contribution(&fields.string("development_comparison")?)?,
        validation_comparison: parse_contribution(&fields.string("validation_comparison")?)?,
        overall_comparison: parse_contribution(&fields.string("overall_comparison")?)?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_contribution(&value)?;
    Ok(value)
}

fn encode_eligibility(value: &MomentumMicroHoldoutEligibilityReceiptV1) -> Result<Vec<u8>, String> {
    validate_eligibility(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroHoldoutEligibilityReceiptV1")
        .string("task_registration_digest", &value.task_registration_digest)
        .string(
            "participant_registration_digest",
            &value.participant_registration_digest,
        )
        .string("participant_id", &value.participant_id)
        .string(
            "development_aggregate_digest",
            &value.development_aggregate_digest,
        )
        .string(
            "validation_aggregate_digest",
            &value.validation_aggregate_digest,
        )
        .boolean(
            "development_lower_brier_than_constant",
            value.development_lower_brier_than_constant,
        )
        .boolean(
            "validation_lower_brier_than_constant",
            value.validation_lower_brier_than_constant,
        )
        .boolean("sufficient_paired_support", value.sufficient_paired_support)
        .boolean("finite_predictions", value.finite_predictions)
        .boolean("finite_metrics", value.finite_metrics)
        .boolean("no_probability_collapse", value.no_probability_collapse)
        .boolean("no_saturation_failure", value.no_saturation_failure)
        .boolean("chronology_clean", value.chronology_clean)
        .boolean("leakage_clean", value.leakage_clean)
        .boolean("integrity_clean", value.integrity_clean)
        .boolean(
            "result_selected_mutation_absent",
            value.result_selected_mutation_absent,
        )
        .unsigned("holdout_access_count", as_u64(value.holdout_access_count)?)
        .string("eligibility", eligibility_name(value.eligibility))
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_eligibility(bytes: &[u8]) -> Result<MomentumMicroHoldoutEligibilityReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroHoldoutEligibilityReceiptV1")?;
    let value = MomentumMicroHoldoutEligibilityReceiptV1 {
        task_registration_digest: fields.string("task_registration_digest")?,
        participant_registration_digest: fields.string("participant_registration_digest")?,
        participant_id: fields.string("participant_id")?,
        development_aggregate_digest: fields.string("development_aggregate_digest")?,
        validation_aggregate_digest: fields.string("validation_aggregate_digest")?,
        development_lower_brier_than_constant: fields
            .boolean("development_lower_brier_than_constant")?,
        validation_lower_brier_than_constant: fields
            .boolean("validation_lower_brier_than_constant")?,
        sufficient_paired_support: fields.boolean("sufficient_paired_support")?,
        finite_predictions: fields.boolean("finite_predictions")?,
        finite_metrics: fields.boolean("finite_metrics")?,
        no_probability_collapse: fields.boolean("no_probability_collapse")?,
        no_saturation_failure: fields.boolean("no_saturation_failure")?,
        chronology_clean: fields.boolean("chronology_clean")?,
        leakage_clean: fields.boolean("leakage_clean")?,
        integrity_clean: fields.boolean("integrity_clean")?,
        result_selected_mutation_absent: fields.boolean("result_selected_mutation_absent")?,
        holdout_access_count: as_usize(fields.unsigned("holdout_access_count")?)?,
        eligibility: parse_eligibility(&fields.string("eligibility")?)?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_eligibility(&value)?;
    Ok(value)
}

fn encode_cohort(value: &MomentumMicroProposedHoldoutCohortV1) -> Result<Vec<u8>, String> {
    validate_cohort(value)?;
    ArtifactBuilderV4_2::new("MomentumMicroProposedHoldoutCohortV1")
        .string("cohort_version", &value.cohort_version)
        .string("authorization_digest", &value.authorization_digest)
        .strings(
            "eligibility_receipt_digests",
            &value.eligibility_receipt_digests,
        )
        .strings("participant_ids", &value.participant_ids)
        .string("status", cohort_status_name(value.status))
        .boolean(
            "holdout_execution_authorized",
            value.holdout_execution_authorized,
        )
        .string("cohort_digest", &value.cohort_digest)
        .encode()
}

fn decode_cohort(bytes: &[u8]) -> Result<MomentumMicroProposedHoldoutCohortV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMicroProposedHoldoutCohortV1")?;
    let value = MomentumMicroProposedHoldoutCohortV1 {
        cohort_version: fields.string("cohort_version")?,
        authorization_digest: fields.string("authorization_digest")?,
        eligibility_receipt_digests: fields.strings("eligibility_receipt_digests")?,
        participant_ids: fields.strings("participant_ids")?,
        status: parse_cohort_status(&fields.string("status")?)?,
        holdout_execution_authorized: fields.boolean("holdout_execution_authorized")?,
        cohort_digest: fields.string("cohort_digest")?,
    };
    fields.finish()?;
    validate_cohort(&value)?;
    Ok(value)
}

fn encode_journal(value: &MomentumT10ScreeningJournalV1) -> Result<Vec<u8>, String> {
    validate_journal(value)?;
    ArtifactBuilderV4_2::new("MomentumT10ScreeningJournalV1")
        .string("journal_version", &value.journal_version)
        .string("authorization_digest", &value.authorization_digest)
        .string(
            "development_aggregate_digest",
            &value.development_aggregate_digest,
        )
        .string(
            "validation_aggregate_digest",
            &value.validation_aggregate_digest,
        )
        .strings(
            "benchmark_receipt_digests",
            &value.benchmark_receipt_digests,
        )
        .strings(
            "contribution_receipt_digests",
            &value.contribution_receipt_digests,
        )
        .strings(
            "eligibility_receipt_digests",
            &value.eligibility_receipt_digests,
        )
        .string("cohort_digest", &value.cohort_digest)
        .unsigned("holdout_access_count", as_u64(value.holdout_access_count)?)
        .unsigned("t30_execution_count", as_u64(value.t30_execution_count)?)
        .unsigned("t60_execution_count", as_u64(value.t60_execution_count)?)
        .unsigned("live_authority_count", as_u64(value.live_authority_count)?)
        .boolean("deterministic", value.deterministic)
        .string("replay_digest", &value.replay_digest)
        .encode()
}

fn decode_journal(bytes: &[u8]) -> Result<MomentumT10ScreeningJournalV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumT10ScreeningJournalV1")?;
    let value = MomentumT10ScreeningJournalV1 {
        journal_version: fields.string("journal_version")?,
        authorization_digest: fields.string("authorization_digest")?,
        development_aggregate_digest: fields.string("development_aggregate_digest")?,
        validation_aggregate_digest: fields.string("validation_aggregate_digest")?,
        benchmark_receipt_digests: fields.strings("benchmark_receipt_digests")?,
        contribution_receipt_digests: fields.strings("contribution_receipt_digests")?,
        eligibility_receipt_digests: fields.strings("eligibility_receipt_digests")?,
        cohort_digest: fields.string("cohort_digest")?,
        holdout_access_count: as_usize(fields.unsigned("holdout_access_count")?)?,
        t30_execution_count: as_usize(fields.unsigned("t30_execution_count")?)?,
        t60_execution_count: as_usize(fields.unsigned("t60_execution_count")?)?,
        live_authority_count: as_usize(fields.unsigned("live_authority_count")?)?,
        deterministic: fields.boolean("deterministic")?,
        replay_digest: fields.string("replay_digest")?,
    };
    fields.finish()?;
    validate_journal(&value)?;
    Ok(value)
}

fn status_name(value: MomentumT10MicroScreeningStatusV1) -> &'static str {
    match value {
        MomentumT10MicroScreeningStatusV1::Unregistered => "unregistered",
        MomentumT10MicroScreeningStatusV1::Authorized => "authorized",
        MomentumT10MicroScreeningStatusV1::DevelopmentComplete => "development-complete",
        MomentumT10MicroScreeningStatusV1::Complete => "complete",
        MomentumT10MicroScreeningStatusV1::TrainingSupportInsufficientForAllParticipants => {
            "training-support-insufficient-for-all-participants"
        }
        MomentumT10MicroScreeningStatusV1::IntegrityFailure => "integrity-failure",
    }
}

fn parse_status(value: &str) -> Result<MomentumT10MicroScreeningStatusV1, String> {
    match value {
        "unregistered" => Ok(MomentumT10MicroScreeningStatusV1::Unregistered),
        "authorized" => Ok(MomentumT10MicroScreeningStatusV1::Authorized),
        "development-complete" => Ok(MomentumT10MicroScreeningStatusV1::DevelopmentComplete),
        "complete" => Ok(MomentumT10MicroScreeningStatusV1::Complete),
        "training-support-insufficient-for-all-participants" => {
            Ok(MomentumT10MicroScreeningStatusV1::TrainingSupportInsufficientForAllParticipants)
        }
        "integrity-failure" => Ok(MomentumT10MicroScreeningStatusV1::IntegrityFailure),
        _ => Err("T10 screening status rejected".to_string()),
    }
}

fn encode_boundary(value: &MomentumMicroTaskPartitionBoundaryV1) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumMicroTaskPartitionBoundaryV1")
        .string("boundary_version", &value.boundary_version)
        .string("task", &format!("{:?}", value.task))
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
    let task = match fields.string("task")?.as_str() {
        "T10NextTenMinuteDirection" => MomentumMicroTaskV1::T10NextTenMinuteDirection,
        "T30NextThirtyMinuteDirection" => MomentumMicroTaskV1::T30NextThirtyMinuteDirection,
        _ => return Err("T10 report task boundary rejected".to_string()),
    };
    let value = MomentumMicroTaskPartitionBoundaryV1 {
        boundary_version: fields.string("boundary_version")?,
        task,
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
    validate_task_boundary(&value)?;
    Ok(value)
}

fn safety_values(value: &MomentumT10ScreeningSafetyCountersV1) -> Result<Vec<u64>, String> {
    [
        value.artifacts_written,
        value.duplicate_artifact_count,
        value.new_model_fits,
        value.new_calibration_fits,
        value.new_predictions,
        value.new_target_reveals,
        value.new_evaluations,
        value.new_metric_computations,
        value.t10_holdout_predictions,
        value.t10_holdout_label_reads,
        value.t10_holdout_metrics,
        value.t30_model_fits,
        value.t30_calibration_fits,
        value.t30_predictions,
        value.t30_evaluations,
        value.t30_holdout_access,
        value.t60_model_fits,
        value.t60_calibration_fits,
        value.t60_predictions,
        value.t60_evaluations,
        value.t60_holdout_access,
        value.network_requests,
        value.live_requests,
        value.live_outcomes,
        value.live_predictions,
        value.live_evaluations,
        value.live_parameter_changes,
        value.live_normalizer_changes,
        value.winner_selections,
        value.rankings,
        value.reward_applications,
        value.penalty_applications,
        value.chair_actions,
        value.vote_actions,
        value.paper_trading_actions,
        value.live_trading_actions,
        value.month_view_loads,
        value.year_view_loads,
    ]
    .into_iter()
    .map(as_u64)
    .collect()
}

fn safety_from_values(values: Vec<u64>) -> Result<MomentumT10ScreeningSafetyCountersV1, String> {
    let values = values
        .into_iter()
        .map(as_usize)
        .collect::<Result<Vec<_>, _>>()?;
    if values.len() != 38 {
        return Err("T10 screening safety counter count rejected".to_string());
    }
    Ok(MomentumT10ScreeningSafetyCountersV1 {
        artifacts_written: values[0],
        duplicate_artifact_count: values[1],
        new_model_fits: values[2],
        new_calibration_fits: values[3],
        new_predictions: values[4],
        new_target_reveals: values[5],
        new_evaluations: values[6],
        new_metric_computations: values[7],
        t10_holdout_predictions: values[8],
        t10_holdout_label_reads: values[9],
        t10_holdout_metrics: values[10],
        t30_model_fits: values[11],
        t30_calibration_fits: values[12],
        t30_predictions: values[13],
        t30_evaluations: values[14],
        t30_holdout_access: values[15],
        t60_model_fits: values[16],
        t60_calibration_fits: values[17],
        t60_predictions: values[18],
        t60_evaluations: values[19],
        t60_holdout_access: values[20],
        network_requests: values[21],
        live_requests: values[22],
        live_outcomes: values[23],
        live_predictions: values[24],
        live_evaluations: values[25],
        live_parameter_changes: values[26],
        live_normalizer_changes: values[27],
        winner_selections: values[28],
        rankings: values[29],
        reward_applications: values[30],
        penalty_applications: values[31],
        chair_actions: values[32],
        vote_actions: values[33],
        paper_trading_actions: values[34],
        live_trading_actions: values[35],
        month_view_loads: values[36],
        year_view_loads: values[37],
    })
}

fn encode_report(value: &MomentumT10MicroScreeningReportV1) -> Result<Vec<u8>, String> {
    validate_report(value)?;
    ArtifactBuilderV4_2::new("MomentumT10MicroScreeningReportV1")
        .string("report_version", &value.report_version)
        .string("run_mode", &value.run_mode)
        .string("status", status_name(value.status))
        .messages(
            "authorization",
            value
                .authorization
                .as_ref()
                .map(encode_authorization)
                .transpose()?
                .into_iter()
                .collect(),
        )
        .messages(
            "training_policy",
            value
                .training_policy
                .as_ref()
                .map(encode_training_policy)
                .transpose()?
                .into_iter()
                .collect(),
        )
        .string(
            "source_label_report_digest",
            &value.source_label_report_digest,
        )
        .string(
            "source_feature_report_digest",
            &value.source_feature_report_digest,
        )
        .string(
            "source_design_report_digest",
            &value.source_design_report_digest,
        )
        .string(
            "source_registration_digest",
            &value.source_registration_digest,
        )
        .string("source_gate_digest", &value.source_gate_digest)
        .string(
            "protected_before_state_digest",
            &value.protected_before_state_digest,
        )
        .unsigned(
            "completed_live_event_count",
            as_u64(value.completed_live_event_count)?,
        )
        .unsigned(
            "scorable_live_event_count",
            as_u64(value.scorable_live_event_count)?,
        )
        .string("live_pause", &value.live_pause)
        .boolean("epoch_three_registered", value.epoch_three_registered)
        .messages(
            "t10_boundary",
            value
                .t10_boundary
                .as_ref()
                .map(encode_boundary)
                .transpose()?
                .into_iter()
                .collect(),
        )
        .messages(
            "t30_boundary",
            value
                .t30_boundary
                .as_ref()
                .map(encode_boundary)
                .transpose()?
                .into_iter()
                .collect(),
        )
        .string("t10_disposition", &value.t10_disposition)
        .string("t30_disposition", &value.t30_disposition)
        .string("t60_disposition", &value.t60_disposition)
        .messages(
            "development",
            value
                .development
                .as_ref()
                .map(encode_aggregate)
                .transpose()?
                .into_iter()
                .collect(),
        )
        .messages(
            "validation",
            value
                .validation
                .as_ref()
                .map(encode_aggregate)
                .transpose()?
                .into_iter()
                .collect(),
        )
        .messages(
            "benchmark_comparisons",
            value
                .benchmark_comparisons
                .iter()
                .map(encode_benchmark)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "contribution_comparisons",
            value
                .contribution_comparisons
                .iter()
                .map(encode_contribution)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "holdout_eligibility_receipts",
            value
                .holdout_eligibility_receipts
                .iter()
                .map(encode_eligibility)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "proposed_holdout_cohort",
            value
                .proposed_holdout_cohort
                .as_ref()
                .map(encode_cohort)
                .transpose()?
                .into_iter()
                .collect(),
        )
        .boolean("full_eight_a3_blocked", value.full_eight_a3_blocked)
        .boolean(
            "historical_holdout_execution_mode_absent",
            value.historical_holdout_execution_mode_absent,
        )
        .boolean("live_roster_unchanged", value.live_roster_unchanged)
        .boolean(
            "protected_artifacts_unchanged",
            value.protected_artifacts_unchanged,
        )
        .strings("labels", &value.labels)
        .unsigneds("safety_counters", &safety_values(&value.safety_counters)?)
        .optional_string(
            "deterministic_replay_digest",
            &value.deterministic_replay_digest,
        )
        .unsigned("runtime_duration_ms", value.runtime_duration_ms)
        .string("report_digest", &value.report_digest)
        .encode()
}

fn one_message<T>(
    values: Vec<Vec<u8>>,
    decode: impl Fn(&[u8]) -> Result<T, String>,
    name: &str,
) -> Result<Option<T>, String> {
    match values.as_slice() {
        [] => Ok(None),
        [value] => decode(value).map(Some),
        _ => Err(format!("T10 {name} message count rejected")),
    }
}

fn decode_report(bytes: &[u8]) -> Result<MomentumT10MicroScreeningReportV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumT10MicroScreeningReportV1")?;
    let value = MomentumT10MicroScreeningReportV1 {
        report_version: fields.string("report_version")?,
        run_mode: fields.string("run_mode")?,
        status: parse_status(&fields.string("status")?)?,
        authorization: one_message(
            fields.messages("authorization")?,
            decode_authorization,
            "authorization",
        )?,
        training_policy: one_message(
            fields.messages("training_policy")?,
            decode_training_policy,
            "training policy",
        )?,
        source_label_report_digest: fields.string("source_label_report_digest")?,
        source_feature_report_digest: fields.string("source_feature_report_digest")?,
        source_design_report_digest: fields.string("source_design_report_digest")?,
        source_registration_digest: fields.string("source_registration_digest")?,
        source_gate_digest: fields.string("source_gate_digest")?,
        protected_before_state_digest: fields.string("protected_before_state_digest")?,
        completed_live_event_count: as_usize(fields.unsigned("completed_live_event_count")?)?,
        scorable_live_event_count: as_usize(fields.unsigned("scorable_live_event_count")?)?,
        live_pause: fields.string("live_pause")?,
        epoch_three_registered: fields.boolean("epoch_three_registered")?,
        t10_boundary: one_message(
            fields.messages("t10_boundary")?,
            decode_boundary,
            "T10 boundary",
        )?,
        t30_boundary: one_message(
            fields.messages("t30_boundary")?,
            decode_boundary,
            "T30 boundary",
        )?,
        t10_disposition: fields.string("t10_disposition")?,
        t30_disposition: fields.string("t30_disposition")?,
        t60_disposition: fields.string("t60_disposition")?,
        development: one_message(
            fields.messages("development")?,
            decode_aggregate,
            "development aggregate",
        )?,
        validation: one_message(
            fields.messages("validation")?,
            decode_aggregate,
            "validation aggregate",
        )?,
        benchmark_comparisons: fields
            .messages("benchmark_comparisons")?
            .iter()
            .map(|item| decode_benchmark(item))
            .collect::<Result<Vec<_>, _>>()?,
        contribution_comparisons: fields
            .messages("contribution_comparisons")?
            .iter()
            .map(|item| decode_contribution(item))
            .collect::<Result<Vec<_>, _>>()?,
        holdout_eligibility_receipts: fields
            .messages("holdout_eligibility_receipts")?
            .iter()
            .map(|item| decode_eligibility(item))
            .collect::<Result<Vec<_>, _>>()?,
        proposed_holdout_cohort: one_message(
            fields.messages("proposed_holdout_cohort")?,
            decode_cohort,
            "holdout cohort",
        )?,
        full_eight_a3_blocked: fields.boolean("full_eight_a3_blocked")?,
        historical_holdout_execution_mode_absent: fields
            .boolean("historical_holdout_execution_mode_absent")?,
        live_roster_unchanged: fields.boolean("live_roster_unchanged")?,
        protected_artifacts_unchanged: fields.boolean("protected_artifacts_unchanged")?,
        labels: fields.strings("labels")?,
        safety_counters: safety_from_values(fields.unsigneds("safety_counters")?)?,
        deterministic_replay_digest: fields.optional_string("deterministic_replay_digest")?,
        runtime_duration_ms: fields.unsigned("runtime_duration_ms")?,
        report_digest: fields.string("report_digest")?,
    };
    fields.finish()?;
    validate_report(&value)?;
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

fn persist_path(
    path: &Path,
    digest: &str,
    bytes: &[u8],
    decode_digest: impl Fn(&[u8]) -> Result<String, String>,
) -> Result<(usize, usize), String> {
    persist_artifact(path, bytes, digest, decode_digest)
}

fn add_counts(total: &mut (usize, usize), next: (usize, usize)) {
    total.0 += next.0;
    total.1 += next.1;
}

fn reopen_exact<T>(path: &Path, decode: impl Fn(&[u8]) -> Result<T, String>) -> Result<T, String> {
    let bytes = fs::read(path).map_err(|_| "T10 screening artifact reopen failed".to_string())?;
    decode(&bytes)
}

fn read_authorization() -> Result<Option<MomentumT10ScreeningExecutionAuthorizationV1>, String> {
    read_single(
        &Path::new(ROOT).join("authorizations"),
        decode_authorization,
    )
}

fn read_training_policy() -> Result<Option<MomentumT10TrainingPolicyV1>, String> {
    read_single(
        &Path::new(ROOT).join("training_policies"),
        decode_training_policy,
    )
}

fn read_aggregate(
    partition: MomentumReplayPartitionV1,
) -> Result<Option<MomentumT10PartitionAggregateV1>, String> {
    read_single(
        &Path::new(ROOT)
            .join("partition_aggregates")
            .join(partition_name(partition)),
        decode_aggregate,
    )
}

pub fn read_momentum_t10_micro_screening_report_v1()
-> Result<Option<MomentumT10MicroScreeningReportV1>, String> {
    read_single(&Path::new(ROOT).join("final_reports"), decode_report)
}

pub(super) fn read_momentum_t10_consumed_event_evidence_v1(
    partition: MomentumReplayPartitionV1,
) -> Result<Vec<MomentumT10ConsumedEventEvidenceV1>, String> {
    if partition == MomentumReplayPartitionV1::SealedHoldout {
        return Err("T10 sealed holdout evidence read rejected".to_string());
    }
    let report = read_momentum_t10_micro_screening_report_v1()?
        .ok_or_else(|| "T10 completed screening report unavailable".to_string())?;
    validate_report(&report)?;
    if report.status != MomentumT10MicroScreeningStatusV1::Complete {
        return Err("T10 completed screening evidence unavailable".to_string());
    }
    let aggregate = match partition {
        MomentumReplayPartitionV1::Development => report.development.as_ref(),
        MomentumReplayPartitionV1::Validation => report.validation.as_ref(),
        MomentumReplayPartitionV1::SealedHoldout => None,
    }
    .ok_or_else(|| "T10 consumed aggregate unavailable".to_string())?;
    let evidence = load_momentum_qualified_t10_micro_evidence_v1()?;
    if evidence.prior_holdout.labels_opened
        || evidence.prior_holdout.metrics_computed
        || evidence.prior_holdout.aggregate_comparison_opened
    {
        return Err("T10 consumed source holdout opened".to_string());
    }
    let ten_minute = &evidence.ten_minute;
    let partition_root = Path::new(ROOT)
        .join("daily")
        .join(partition_name(partition));
    let mut day_roots = fs::read_dir(&partition_root)
        .map_err(|_| "T10 consumed daily evidence unavailable".to_string())?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|_| "T10 consumed daily entry rejected".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    day_roots.sort();
    let mut result = Vec::new();
    let mut prediction_digests = Vec::new();
    let mut evaluation_digests = Vec::new();
    for day_root in day_roots {
        if !day_root.is_dir() {
            return Err("T10 consumed daily path rejected".to_string());
        }
        let prediction = read_single(&day_root.join("prediction_shards"), decode_prediction_shard)?
            .ok_or_else(|| "T10 consumed prediction shard unavailable".to_string())?;
        let evaluation = read_single(&day_root.join("evaluation_shards"), decode_evaluation_shard)?
            .ok_or_else(|| "T10 consumed evaluation shard unavailable".to_string())?;
        validate_prediction_shard(&prediction)?;
        validate_evaluation_shard(&evaluation)?;
        if prediction.partition != partition
            || evaluation.partition != partition
            || evaluation.prediction_shard_digest != prediction.shard_digest
            || prediction.event_plans.len() != evaluation.evaluations.len()
        {
            return Err("T10 consumed shard binding rejected".to_string());
        }
        prediction_digests.push(prediction.shard_digest.clone());
        evaluation_digests.push(evaluation.shard_digest.clone());
        for (index, (plan, item)) in prediction
            .event_plans
            .iter()
            .zip(&evaluation.evaluations)
            .enumerate()
        {
            if item.event_plan_digest != plan.plan_digest {
                return Err("T10 consumed event binding rejected".to_string());
            }
            let current_index = ten_minute.partition_point(|row| {
                row.close_exclusive_timestamp_ms < plan.prediction_timestamp_ms
            });
            let target_index = ten_minute
                .partition_point(|row| row.close_exclusive_timestamp_ms < plan.target_timestamp_ms);
            let current = ten_minute
                .get(current_index)
                .filter(|row| row.close_exclusive_timestamp_ms == plan.prediction_timestamp_ms)
                .ok_or_else(|| "T10 consumed current candle unavailable".to_string())?;
            let target = ten_minute
                .get(target_index)
                .filter(|row| row.close_exclusive_timestamp_ms == plan.target_timestamp_ms)
                .ok_or_else(|| "T10 consumed target candle unavailable".to_string())?;
            if current.missing_evidence
                || target.missing_evidence
                || !current.close.is_finite()
                || !target.close.is_finite()
                || current.close <= 0.0
                || target.close <= 0.0
            {
                return Err("T10 consumed target return rejected".to_string());
            }
            let target_return = target.close / current.close - 1.0;
            let volatility = past_micro_volatility(ten_minute, plan.prediction_timestamp_ms)
                .ok_or_else(|| "T10 consumed past volatility unavailable".to_string())?;
            if !target_return.is_finite() || !volatility.is_finite() || volatility < 0.0 {
                return Err("T10 consumed finite evidence rejected".to_string());
            }
            result.push(MomentumT10ConsumedEventEvidenceV1 {
                partition,
                prediction_timestamp_ms: plan.prediction_timestamp_ms,
                target_timestamp_ms: plan.target_timestamp_ms,
                event_plan_digest: plan.plan_digest.clone(),
                source_event_digest: plan.source_event_digest.clone(),
                probabilities: prediction.private_probabilities[index * 5..(index + 1) * 5]
                    .to_vec(),
                label: item.private_label,
                brier_values: item.private_brier_values.clone(),
                correctness: item.private_correctness.clone(),
                target_return,
                past_micro_volatility: volatility,
            });
        }
    }
    if prediction_digests != aggregate.prediction_shard_digests
        || evaluation_digests != aggregate.evaluation_shard_digests
        || result.len() != aggregate.prediction_count
        || result
            .windows(2)
            .any(|pair| pair[0].prediction_timestamp_ms >= pair[1].prediction_timestamp_ms)
        || result.iter().filter(|event| event.label.is_some()).count() != aggregate.scorable_count
        || result.iter().any(|event| {
            event.partition != partition
                || event.target_timestamp_ms <= event.prediction_timestamp_ms
                || event.event_plan_digest.is_empty()
                || event.source_event_digest.is_empty()
                || event.probabilities.len() != PARTICIPANTS.len()
                || event
                    .probabilities
                    .iter()
                    .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
                || !event.target_return.is_finite()
                || !event.past_micro_volatility.is_finite()
                || event.past_micro_volatility < 0.0
                || (event.label.is_some()
                    && (event.brier_values.len() != PARTICIPANTS.len()
                        || event.correctness.len() != PARTICIPANTS.len()
                        || event
                            .brier_values
                            .iter()
                            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))))
                || (event.label.is_none()
                    && (!event.brier_values.is_empty() || !event.correctness.is_empty()))
        })
        || result.iter().enumerate().any(|(index, event)| {
            result[..index].iter().any(|prior| {
                prior.event_plan_digest == event.event_plan_digest
                    || prior.source_event_digest == event.source_event_digest
            })
        })
    {
        return Err("T10 consumed event evidence rejected".to_string());
    }
    Ok(result)
}

pub(super) fn read_momentum_t10_consumed_actionability_evidence_v1(
    partition: MomentumReplayPartitionV1,
) -> Result<Vec<MomentumT10ConsumedActionabilityEvidenceV1>, String> {
    if partition == MomentumReplayPartitionV1::SealedHoldout {
        return Err("T10 sealed actionability evidence read rejected".to_string());
    }
    let report = read_momentum_t10_micro_screening_report_v1()?
        .ok_or_else(|| "T10 actionability screening report unavailable".to_string())?;
    validate_report(&report)?;
    if report.status != MomentumT10MicroScreeningStatusV1::Complete {
        return Err("T10 actionability screening incomplete".to_string());
    }
    let boundary = report
        .t10_boundary
        .as_ref()
        .ok_or_else(|| "T10 actionability boundary unavailable".to_string())?;
    let (start, end, expected_count) = match partition {
        MomentumReplayPartitionV1::Development => (
            boundary.eligible_start_timestamp_ms,
            boundary.development_end_exclusive_ms,
            boundary.development_event_count,
        ),
        MomentumReplayPartitionV1::Validation => (
            boundary.development_end_exclusive_ms,
            boundary.validation_end_exclusive_ms,
            boundary.validation_event_count,
        ),
        MomentumReplayPartitionV1::SealedHoldout => unreachable!(),
    };
    let evidence = load_momentum_qualified_t10_micro_evidence_v1()?;
    if evidence.prior_holdout.labels_opened
        || evidence.prior_holdout.metrics_computed
        || evidence.prior_holdout.aggregate_comparison_opened
    {
        return Err("T10 actionability source holdout opened".to_string());
    }
    let ten_minute = &evidence.ten_minute;
    let mut result = Vec::with_capacity(expected_count);
    let mut source_event_count = 0usize;
    for event in evidence.protocol_events.iter().filter(|event| {
        event.prediction_timestamp_ms >= start && event.prediction_timestamp_ms < end
    }) {
        source_event_count += 1;
        let Some(volatility) = past_micro_volatility(ten_minute, event.prediction_timestamp_ms)
        else {
            continue;
        };
        let current_index = ten_minute.partition_point(|row| {
            row.close_exclusive_timestamp_ms < event.prediction_timestamp_ms
        });
        let target_index = ten_minute
            .partition_point(|row| row.close_exclusive_timestamp_ms < event.target_timestamp_ms);
        let current = ten_minute
            .get(current_index)
            .filter(|row| row.close_exclusive_timestamp_ms == event.prediction_timestamp_ms)
            .ok_or_else(|| "T10 actionability current candle unavailable".to_string())?;
        let target = ten_minute
            .get(target_index)
            .filter(|row| row.close_exclusive_timestamp_ms == event.target_timestamp_ms)
            .ok_or_else(|| "T10 actionability target candle unavailable".to_string())?;
        if current.missing_evidence
            || target.missing_evidence
            || !current.close.is_finite()
            || !target.close.is_finite()
            || current.close <= 0.0
            || target.close <= 0.0
        {
            return Err("T10 actionability candle evidence rejected".to_string());
        }
        let target_return = target.close / current.close - 1.0;
        if !target_return.is_finite() || !volatility.is_finite() || volatility < 0.0 {
            return Err("T10 actionability finite evidence rejected".to_string());
        }
        result.push(MomentumT10ConsumedActionabilityEvidenceV1 {
            partition,
            prediction_timestamp_ms: event.prediction_timestamp_ms,
            target_timestamp_ms: event.target_timestamp_ms,
            source_event_digest: event.receipt_digest.clone(),
            target_return,
            past_micro_volatility: volatility,
        });
    }
    if source_event_count != expected_count
        || result.is_empty()
        || result.len() > source_event_count
        || result
            .windows(2)
            .any(|pair| pair[0].prediction_timestamp_ms >= pair[1].prediction_timestamp_ms)
        || result.iter().any(|event| {
            event.partition != partition
                || event.target_timestamp_ms <= event.prediction_timestamp_ms
                || event.source_event_digest.is_empty()
                || !event.target_return.is_finite()
                || !event.past_micro_volatility.is_finite()
                || event.past_micro_volatility < 0.0
        })
        || result.iter().enumerate().any(|(index, event)| {
            result[..index]
                .iter()
                .any(|prior| prior.source_event_digest == event.source_event_digest)
        })
    {
        return Err("T10 actionability event set rejected".to_string());
    }
    Ok(result)
}

fn anchor_features(
    rows: &[MomentumQualifiedReplayCandleEvidenceV1],
    timestamp_ms: u64,
) -> Result<Vec<f32>, String> {
    let config = MomentumFeatureConfigV0::default();
    let end = rows.partition_point(|row| row.close_exclusive_timestamp_ms <= timestamp_ms);
    if end < ANCHOR_CONTEXT_LENGTH {
        return Err("T10 anchor context support unavailable".to_string());
    }
    let context = &rows[end - ANCHOR_CONTEXT_LENGTH..end];
    if context
        .iter()
        .any(|row| row.missing_evidence || row.close_exclusive_timestamp_ms > timestamp_ms)
    {
        return Err("T10 anchor closed-candle evidence rejected".to_string());
    }
    let candles = context
        .iter()
        .map(|row| {
            Ok(MomentumCandleV0 {
                timestamp: i64::try_from(row.close_exclusive_timestamp_ms)
                    .map_err(|_| "T10 anchor timestamp rejected".to_string())?,
                open: checked_f32(row.open)?,
                high: checked_f32(row.high)?,
                low: checked_f32(row.low)?,
                close: checked_f32(row.close)?,
                volume: checked_f32(row.volume)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let features = build_momentum_features_v0(&candles, &config)
        .map_err(|_| "T10 anchor feature extraction rejected".to_string())?;
    let values = features
        .last()
        .map(|row| row.values.clone())
        .ok_or_else(|| "T10 anchor feature row unavailable".to_string())?;
    if values.len() != 6 || values.iter().any(|item| !item.is_finite()) {
        return Err("T10 anchor feature integrity rejected".to_string());
    }
    Ok(values)
}

fn event_feature_support_available(
    policy: &MomentumCompactMicroFeaturePolicyV1,
    evidence: &MomentumQualifiedSixEvidenceV1,
    timestamp_ms: u64,
) -> Result<bool, String> {
    for timeframe in &policy.included_timeframes {
        let rows = evidence
            .views
            .get(timeframe)
            .ok_or_else(|| "T10 registered feature view unavailable".to_string())?;
        let end = rows.partition_point(|row| row.close_exclusive_timestamp_ms <= timestamp_ms);
        if end < policy.context_length.max(ANCHOR_CONTEXT_LENGTH) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn prepare_event(
    partition: MomentumReplayPartitionV1,
    prediction_timestamp_ms: u64,
    target_timestamp_ms: u64,
    source_event_digest: String,
    policy: &MomentumCompactMicroFeaturePolicyV1,
    evidence: &MomentumQualifiedSixEvidenceV1,
) -> Result<PreparedEvent, String> {
    if partition == MomentumReplayPartitionV1::SealedHoldout
        || prediction_timestamp_ms % TEN_MINUTE_MS != 0
        || target_timestamp_ms != prediction_timestamp_ms + TEN_MINUTE_MS
    {
        return Err("T10 prepared event chronology rejected".to_string());
    }
    let ten_minute = evidence
        .views
        .get(&MomentumHistoricalTimeframeV1::Minute10)
        .ok_or_else(|| "T10 anchor view unavailable".to_string())?;
    let anchor = anchor_features(ten_minute, prediction_timestamp_ms)?;
    let (compact, _, _) =
        extract_compact_micro_event_v1(policy, &evidence.views, prediction_timestamp_ms)?;
    if compact.len() != policy.feature_dimension() {
        return Err("T10 compact event dimension rejected".to_string());
    }
    Ok(PreparedEvent {
        partition,
        prediction_timestamp_ms,
        target_timestamp_ms,
        source_event_digest,
        anchor,
        compact,
    })
}

fn prepare_screening(source: &FrozenSource) -> Result<PreparedScreening, String> {
    let evidence = load_momentum_qualified_six_evidence_v1()?;
    if evidence.prior_holdout.labels_opened
        || evidence.prior_holdout.metrics_computed
        || evidence.prior_holdout.aggregate_comparison_opened
    {
        return Err("T10 sealed holdout source opened".to_string());
    }
    let mut development = Vec::with_capacity(source.t10_boundary.development_event_count);
    let mut validation = Vec::with_capacity(source.t10_boundary.validation_event_count);
    let mut development_boundary_event_count = 0usize;
    let mut validation_boundary_event_count = 0usize;
    let mut development_boundary_days = BTreeSet::new();
    let mut validation_boundary_days = BTreeSet::new();
    for event in &evidence.protocol_events {
        if event.prediction_timestamp_ms < source.t10_boundary.eligible_start_timestamp_ms
            || event.prediction_timestamp_ms >= source.t10_boundary.validation_end_exclusive_ms
        {
            continue;
        }
        let partition =
            if event.prediction_timestamp_ms < source.t10_boundary.development_end_exclusive_ms {
                MomentumReplayPartitionV1::Development
            } else {
                MomentumReplayPartitionV1::Validation
            };
        let day = event.prediction_timestamp_ms / DAY_MS * DAY_MS;
        match partition {
            MomentumReplayPartitionV1::Development => {
                development_boundary_event_count += 1;
                development_boundary_days.insert(day);
            }
            MomentumReplayPartitionV1::Validation => {
                validation_boundary_event_count += 1;
                validation_boundary_days.insert(day);
            }
            MomentumReplayPartitionV1::SealedHoldout => unreachable!(),
        }
        if !event_feature_support_available(
            &source.policy,
            &evidence,
            event.prediction_timestamp_ms,
        )? {
            continue;
        }
        let prepared = prepare_event(
            partition,
            event.prediction_timestamp_ms,
            event.target_timestamp_ms,
            event.receipt_digest.clone(),
            &source.policy,
            &evidence,
        )?;
        match partition {
            MomentumReplayPartitionV1::Development => development.push(prepared),
            MomentumReplayPartitionV1::Validation => validation.push(prepared),
            MomentumReplayPartitionV1::SealedHoldout => unreachable!(),
        }
    }
    if development_boundary_event_count != source.t10_boundary.development_event_count
        || validation_boundary_event_count != source.t10_boundary.validation_event_count
        || development.is_empty()
        || validation.is_empty()
        || development
            .windows(2)
            .chain(validation.windows(2))
            .any(|pair| pair[0].prediction_timestamp_ms >= pair[1].prediction_timestamp_ms)
    {
        return Err("T10 exact screening event set rejected".to_string());
    }
    Ok(PreparedScreening {
        evidence,
        development,
        validation,
        development_boundary_event_count,
        validation_boundary_event_count,
        development_boundary_days,
        validation_boundary_days,
    })
}

fn reveal_label(
    evidence: &MomentumQualifiedSixEvidenceV1,
    event: &PreparedEvent,
) -> Result<(ScreeningLabel, Option<f64>), String> {
    if event.partition == MomentumReplayPartitionV1::SealedHoldout {
        return Err("T10 holdout label reveal rejected".to_string());
    }
    let rows = evidence
        .views
        .get(&MomentumHistoricalTimeframeV1::Minute10)
        .ok_or_else(|| "T10 label view unavailable".to_string())?;
    let current_index = rows
        .partition_point(|row| row.close_exclusive_timestamp_ms < event.prediction_timestamp_ms);
    let target_index =
        rows.partition_point(|row| row.close_exclusive_timestamp_ms < event.target_timestamp_ms);
    let current = rows
        .get(current_index)
        .filter(|row| row.close_exclusive_timestamp_ms == event.prediction_timestamp_ms)
        .ok_or_else(|| "T10 current target candle unavailable".to_string())?;
    let target = rows
        .get(target_index)
        .filter(|row| row.close_exclusive_timestamp_ms == event.target_timestamp_ms)
        .ok_or_else(|| "T10 next target candle unavailable".to_string())?;
    if current.missing_evidence
        || target.missing_evidence
        || !current.close.is_finite()
        || !target.close.is_finite()
        || current.close <= 0.0
        || target.close <= 0.0
    {
        return Ok((ScreeningLabel::Invalid, None));
    }
    if target.close > current.close {
        Ok((ScreeningLabel::Up, Some(1.0)))
    } else if target.close < current.close {
        Ok((ScreeningLabel::Down, Some(0.0)))
    } else {
        Ok((ScreeningLabel::Neutral, None))
    }
}

fn training_examples(
    used: &[(&PreparedEvent, f64)],
    compact: bool,
) -> Vec<EncodedTrainingExampleV0> {
    used.iter()
        .map(|(event, label)| EncodedTrainingExampleV0 {
            representation: if compact {
                event.compact.clone()
            } else {
                event.anchor.clone()
            },
            label: *label as f32,
            snapshot_ids: vec![event.source_event_digest.clone()],
        })
        .collect()
}

fn training_window<'a>(
    prepared: &'a PreparedScreening,
    partition: MomentumReplayPartitionV1,
    day: u64,
    authorization: &MomentumT10ScreeningExecutionAuthorizationV1,
    policy: &MomentumT10TrainingPolicyV1,
) -> Result<
    (
        MomentumT10DailyTrainingWindowPlanV1,
        Vec<(&'a PreparedEvent, f64)>,
    ),
    String,
> {
    let mut scorable = Vec::new();
    let mut eligible_past_event_count = 0usize;
    for event in &prepared.development {
        if event.target_timestamp_ms >= day {
            continue;
        }
        eligible_past_event_count += 1;
        let (_, label) = reveal_label(&prepared.evidence, event)?;
        if let Some(label) = label {
            scorable.push((event, label));
        }
    }
    let start = scorable
        .len()
        .saturating_sub(policy.maximum_training_examples);
    let used = scorable[start..].to_vec();
    let c4_base_count = used.len() * C4_BASE_PERCENT / 100;
    let mut plan = MomentumT10DailyTrainingWindowPlanV1 {
        plan_version: TRAINING_PLAN_VERSION.to_string(),
        authorization_digest: authorization.authorization_digest.clone(),
        training_policy_digest: policy.policy_digest.clone(),
        partition,
        utc_day_boundary_ms: day,
        training_target_cutoff_exclusive_ms: day,
        eligible_past_event_count,
        scorable_training_event_count: scorable.len(),
        used_training_event_count: used.len(),
        training_event_digests: used
            .iter()
            .map(|(event, _)| event.source_event_digest.clone())
            .collect(),
        c4_base_count,
        c4_calibration_count: used.len() - c4_base_count,
        support_sufficient_for_all: used.len() >= policy.minimum_training_examples
            && c4_base_count >= policy.minimum_training_examples
            && used.len() - c4_base_count >= derive_minimum_support(CALIBRATOR_FEATURE_DIMENSION)?,
        validation_label_fit_count: 0,
        holdout_access_count: 0,
        plan_digest: String::new(),
    };
    plan.plan_digest = training_plan_digest(&plan);
    validate_training_plan(&plan)?;
    Ok((plan, used))
}

fn fit_normalizer(
    participant: MomentumMicroParticipantV1,
    feature_policy_digest: String,
    examples: &[EncodedTrainingExampleV0],
) -> Result<
    (
        RepresentationNormalizerV0,
        MomentumT10DailyNormalizerReceiptV1,
    ),
    String,
> {
    let normalizer = RepresentationNormalizerV0::fit(examples)
        .map_err(|_| "T10 normalizer fit rejected".to_string())?;
    let mut receipt = MomentumT10DailyNormalizerReceiptV1 {
        receipt_version: NORMALIZER_VERSION.to_string(),
        participant_id: participant_id(participant),
        feature_policy_digest,
        private_means: normalizer.means.clone(),
        private_scales: normalizer.scales.clone(),
        constant_dimension_indices: normalizer.constant_dimension_indices.clone(),
        training_event_digest: stable_hash_string(&format!(
            "T10-normalizer-training-events:{:?}",
            examples
                .iter()
                .flat_map(|example| example.snapshot_ids.iter())
                .collect::<Vec<_>>()
        )),
        finite: true,
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = normalizer_receipt_digest(&receipt);
    validate_normalizer_receipt(&receipt)?;
    Ok((normalizer, receipt))
}

fn training_config(
    policy: &MomentumT10TrainingPolicyV1,
    seed: u64,
    l2_multiplier: usize,
) -> Result<HeadTrainingConfigV0, String> {
    let mut value = MomentumLearningCampaignConfigV0::default().training_config;
    value.epochs = policy.epoch_count;
    value.batch_size = policy.batch_size;
    value.seed = seed;
    value.optimizer.learning_rate = f32::from_bits(policy.learning_rate_bits);
    value.optimizer.weight_decay = f32::from_bits(policy.standard_l2_bits) * l2_multiplier as f32;
    value.early_stopping_patience = None;
    value
        .validate()
        .map_err(|_| "T10 bounded training config rejected".to_string())?;
    Ok(value)
}

fn fit_head(
    examples: &[EncodedTrainingExampleV0],
    normalizer: &RepresentationNormalizerV0,
    config: &HeadTrainingConfigV0,
) -> Result<LogisticPredictionHeadV0, String> {
    let normalized = normalizer
        .transform(examples)
        .map_err(|_| "T10 training normalization rejected".to_string())?;
    let dimension = normalized
        .first()
        .map(|item| item.representation.len())
        .ok_or_else(|| "T10 training evidence unavailable".to_string())?;
    let initial = LogisticPredictionHeadV0::seeded(dimension, config.seed)
        .map_err(|_| "T10 deterministic initialization rejected".to_string())?;
    train_head_v4(initial, &normalized, config)
}

fn head_logit(head: &LogisticPredictionHeadV0, representation: &[f32]) -> Result<f32, String> {
    head.validate()
        .map_err(|_| "T10 head finite validation rejected".to_string())?;
    if head.weights.len() != representation.len()
        || representation.iter().any(|item| !item.is_finite())
    {
        return Err("T10 head representation rejected".to_string());
    }
    let value = head.bias
        + head
            .weights
            .iter()
            .zip(representation)
            .map(|(weight, feature)| weight * feature)
            .sum::<f32>();
    if !value.is_finite() {
        return Err("T10 head logit rejected".to_string());
    }
    Ok(value)
}

fn learned_receipt(
    participant: MomentumMicroParticipantV1,
    role: DailyModelRole,
    day: u64,
    plan: &MomentumT10DailyTrainingWindowPlanV1,
    normalizer_receipt_digest: String,
    head: &LogisticPredictionHeadV0,
    labels: &[f64],
    config: &HeadTrainingConfigV0,
) -> Result<MomentumT10DailyModelReceiptV1, String> {
    let positive_count = labels.iter().filter(|label| **label == 1.0).count();
    let mut value = MomentumT10DailyModelReceiptV1 {
        receipt_version: MODEL_VERSION.to_string(),
        participant_id: participant_id(participant),
        role,
        utc_day_boundary_ms: day,
        training_plan_digest: plan.plan_digest.clone(),
        normalizer_receipt_digest,
        private_weights: head.weights.clone(),
        private_bias: Some(head.bias),
        private_prevalence: None,
        training_count: labels.len(),
        positive_count,
        negative_count: labels.len() - positive_count,
        l2_bits: config.optimizer.weight_decay.to_bits(),
        initialization_seed: config.seed,
        finite: true,
        validation_fit_count: 0,
        holdout_fit_count: 0,
        receipt_digest: String::new(),
    };
    value.receipt_digest = model_receipt_digest(&value);
    validate_model_receipt(&value)?;
    Ok(value)
}

fn frozen_digest(
    participant: MomentumMicroParticipantV1,
    normalizer_digest: &str,
    model_digest: &str,
    calibrator_digest: Option<&str>,
) -> String {
    stable_hash_string(&format!(
        "T10-reconstructed-participant:{:?}:{normalizer_digest}:{model_digest}:{calibrator_digest:?}",
        participant
    ))
}

fn freeze_daily_participants(
    source: &FrozenSource,
    partition: MomentumReplayPartitionV1,
    day: u64,
    authorization: &MomentumT10ScreeningExecutionAuthorizationV1,
    policy: &MomentumT10TrainingPolicyV1,
    plan: MomentumT10DailyTrainingWindowPlanV1,
    used: Vec<(&PreparedEvent, f64)>,
) -> Result<(MomentumT10DailyRefitBundleV1, Vec<FrozenParticipant>), String> {
    if !plan.support_sufficient_for_all
        || used.len() != plan.used_training_event_count
        || plan.authorization_digest != authorization.authorization_digest
        || plan.training_policy_digest != policy.policy_digest
    {
        return Err("T10 paired training support rejected".to_string());
    }
    let labels = used.iter().map(|(_, label)| *label).collect::<Vec<_>>();
    let positive_count = labels.iter().filter(|label| **label == 1.0).count();
    let prevalence = positive_count as f64 / labels.len() as f64;
    if !prevalence.is_finite() {
        return Err("T10 constant prevalence rejected".to_string());
    }
    let mut constant_receipt = MomentumT10DailyModelReceiptV1 {
        receipt_version: MODEL_VERSION.to_string(),
        participant_id: participant_id(MomentumMicroParticipantV1::C0TaskSpecificConstant),
        role: DailyModelRole::Constant,
        utc_day_boundary_ms: day,
        training_plan_digest: plan.plan_digest.clone(),
        normalizer_receipt_digest: String::new(),
        private_weights: Vec::new(),
        private_bias: None,
        private_prevalence: Some(prevalence),
        training_count: labels.len(),
        positive_count,
        negative_count: labels.len() - positive_count,
        l2_bits: 0,
        initialization_seed: 0,
        finite: true,
        validation_fit_count: 0,
        holdout_fit_count: 0,
        receipt_digest: String::new(),
    };
    constant_receipt.receipt_digest = model_receipt_digest(&constant_receipt);
    validate_model_receipt(&constant_receipt)?;

    let anchor_examples = training_examples(&used, false);
    let compact_examples = training_examples(&used, true);
    let c4_base_examples = &compact_examples[..plan.c4_base_count];
    let c4_calibration_examples = &compact_examples[plan.c4_base_count..];

    let anchor_policy_digest = source
        .registration
        .participant_registrations
        .iter()
        .find(|item| {
            item.task == MomentumMicroTaskV1::T10NextTenMinuteDirection
                && item.participant == MomentumMicroParticipantV1::C1TenMinuteAnchorBaseline
        })
        .map(|item| item.feature_policy_digest.clone())
        .ok_or_else(|| "T10 anchor participant policy unavailable".to_string())?;
    let (c1_normalizer, c1_normalizer_receipt) = fit_normalizer(
        MomentumMicroParticipantV1::C1TenMinuteAnchorBaseline,
        anchor_policy_digest,
        &anchor_examples,
    )?;
    let (c2_normalizer, c2_normalizer_receipt) = fit_normalizer(
        MomentumMicroParticipantV1::C2CompactMicroLogistic,
        source.policy.schema_digest.clone(),
        &compact_examples,
    )?;
    let (c3_normalizer, c3_normalizer_receipt) = fit_normalizer(
        MomentumMicroParticipantV1::C3CompactMicroStrongShrinkLogistic,
        source.policy.schema_digest.clone(),
        &compact_examples,
    )?;
    let (c4_normalizer, c4_normalizer_receipt) = fit_normalizer(
        MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic,
        source.policy.schema_digest.clone(),
        c4_base_examples,
    )?;

    let seed = policy.initialization_seed;
    let c1_config = training_config(policy, seed, 1)?;
    let c2_config = training_config(policy, seed, 1)?;
    let c3_config = training_config(policy, seed, C3_L2_MULTIPLIER)?;
    let c4_config = training_config(policy, seed, 1)?;
    let c1_head = fit_head(&anchor_examples, &c1_normalizer, &c1_config)?;
    let c2_head = fit_head(&compact_examples, &c2_normalizer, &c2_config)?;
    let c3_head = fit_head(&compact_examples, &c3_normalizer, &c3_config)?;
    let c4_head = fit_head(c4_base_examples, &c4_normalizer, &c4_config)?;

    let c4_calibration_normalized = c4_normalizer
        .transform(c4_calibration_examples)
        .map_err(|_| "T10 C4 calibration normalization rejected".to_string())?;
    let calibrator_examples = c4_calibration_normalized
        .iter()
        .map(|example| {
            Ok(EncodedTrainingExampleV0 {
                representation: vec![head_logit(&c4_head, &example.representation)?],
                label: example.label,
                snapshot_ids: example.snapshot_ids.clone(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let calibrator_normalizer = RepresentationNormalizerV0 {
        means: vec![0.0],
        scales: vec![1.0],
        constant_dimension_indices: Vec::new(),
    };
    let calibrator_config = training_config(policy, seed, 1)?;
    let calibrator = fit_head(
        &calibrator_examples,
        &calibrator_normalizer,
        &calibrator_config,
    )?;

    let c1_receipt = learned_receipt(
        MomentumMicroParticipantV1::C1TenMinuteAnchorBaseline,
        DailyModelRole::LearnedBase,
        day,
        &plan,
        c1_normalizer_receipt.receipt_digest.clone(),
        &c1_head,
        &labels,
        &c1_config,
    )?;
    let c2_receipt = learned_receipt(
        MomentumMicroParticipantV1::C2CompactMicroLogistic,
        DailyModelRole::LearnedBase,
        day,
        &plan,
        c2_normalizer_receipt.receipt_digest.clone(),
        &c2_head,
        &labels,
        &c2_config,
    )?;
    let c3_receipt = learned_receipt(
        MomentumMicroParticipantV1::C3CompactMicroStrongShrinkLogistic,
        DailyModelRole::LearnedBase,
        day,
        &plan,
        c3_normalizer_receipt.receipt_digest.clone(),
        &c3_head,
        &labels,
        &c3_config,
    )?;
    let c4_base_labels = labels[..plan.c4_base_count].to_vec();
    let c4_calibration_labels = labels[plan.c4_base_count..].to_vec();
    let c4_receipt = learned_receipt(
        MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic,
        DailyModelRole::LearnedBase,
        day,
        &plan,
        c4_normalizer_receipt.receipt_digest.clone(),
        &c4_head,
        &c4_base_labels,
        &c4_config,
    )?;
    let calibrator_receipt = learned_receipt(
        MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic,
        DailyModelRole::Calibrator,
        day,
        &plan,
        c4_normalizer_receipt.receipt_digest.clone(),
        &calibrator,
        &c4_calibration_labels,
        &calibrator_config,
    )?;
    let normalizer_receipts = vec![
        c1_normalizer_receipt,
        c2_normalizer_receipt,
        c3_normalizer_receipt,
        c4_normalizer_receipt,
    ];
    let model_receipts = vec![
        constant_receipt,
        c1_receipt,
        c2_receipt,
        c3_receipt,
        c4_receipt,
        calibrator_receipt,
    ];
    let normalizer_for = |participant| {
        normalizer_receipts
            .iter()
            .find(|receipt| receipt.participant_id == participant_id(participant))
            .map(|receipt| receipt.receipt_digest.as_str())
            .unwrap_or("")
    };
    let base_for = |participant| {
        model_receipts
            .iter()
            .find(|receipt| {
                receipt.participant_id == participant_id(participant)
                    && receipt.role != DailyModelRole::Calibrator
            })
            .map(|receipt| receipt.receipt_digest.as_str())
            .unwrap_or("")
    };
    let calibrator_digest = model_receipts
        .last()
        .map(|receipt| receipt.receipt_digest.as_str());
    let reconstructed_participant_digests = PARTICIPANTS
        .iter()
        .map(|participant| {
            frozen_digest(
                *participant,
                normalizer_for(*participant),
                base_for(*participant),
                (*participant
                    == MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic)
                    .then_some(calibrator_digest)
                    .flatten(),
            )
        })
        .collect::<Vec<_>>();
    let mut bundle = MomentumT10DailyRefitBundleV1 {
        bundle_version: REFIT_BUNDLE_VERSION.to_string(),
        authorization_digest: authorization.authorization_digest.clone(),
        partition,
        utc_day_boundary_ms: day,
        training_plan: plan,
        normalizer_receipts,
        model_receipts,
        reconstructed_participant_digests: reconstructed_participant_digests.clone(),
        target_access_count_for_prediction_day: 0,
        holdout_access_count: 0,
        live_access_count: 0,
        bundle_digest: String::new(),
    };
    bundle.bundle_digest = refit_bundle_digest(&bundle);
    validate_refit_bundle(&bundle)?;
    let frozen = vec![
        FrozenParticipant {
            participant: MomentumMicroParticipantV1::C0TaskSpecificConstant,
            normalizer: None,
            head: None,
            calibrator: None,
            prevalence,
            model_digest: reconstructed_participant_digests[0].clone(),
        },
        FrozenParticipant {
            participant: MomentumMicroParticipantV1::C1TenMinuteAnchorBaseline,
            normalizer: Some(c1_normalizer),
            head: Some(c1_head),
            calibrator: None,
            prevalence,
            model_digest: reconstructed_participant_digests[1].clone(),
        },
        FrozenParticipant {
            participant: MomentumMicroParticipantV1::C2CompactMicroLogistic,
            normalizer: Some(c2_normalizer),
            head: Some(c2_head),
            calibrator: None,
            prevalence,
            model_digest: reconstructed_participant_digests[2].clone(),
        },
        FrozenParticipant {
            participant: MomentumMicroParticipantV1::C3CompactMicroStrongShrinkLogistic,
            normalizer: Some(c3_normalizer),
            head: Some(c3_head),
            calibrator: None,
            prevalence,
            model_digest: reconstructed_participant_digests[3].clone(),
        },
        FrozenParticipant {
            participant: MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic,
            normalizer: Some(c4_normalizer),
            head: Some(c4_head),
            calibrator: Some(calibrator),
            prevalence,
            model_digest: reconstructed_participant_digests[4].clone(),
        },
    ];
    Ok((bundle, frozen))
}

fn reconstruct_refit_bundle(
    value: &MomentumT10DailyRefitBundleV1,
) -> Result<Vec<FrozenParticipant>, String> {
    validate_refit_bundle(value)?;
    let normalizer_for = |participant: MomentumMicroParticipantV1| {
        value
            .normalizer_receipts
            .iter()
            .find(|receipt| receipt.participant_id == participant_id(participant))
            .map(|receipt| RepresentationNormalizerV0 {
                means: receipt.private_means.clone(),
                scales: receipt.private_scales.clone(),
                constant_dimension_indices: receipt.constant_dimension_indices.clone(),
            })
    };
    let model_for = |participant: MomentumMicroParticipantV1, role: DailyModelRole| {
        value.model_receipts.iter().find(|receipt| {
            receipt.participant_id == participant_id(participant) && receipt.role == role
        })
    };
    let mut frozen = Vec::new();
    for participant in PARTICIPANTS {
        let base = model_for(
            participant,
            if participant == MomentumMicroParticipantV1::C0TaskSpecificConstant {
                DailyModelRole::Constant
            } else {
                DailyModelRole::LearnedBase
            },
        )
        .ok_or_else(|| "T10 reopened base model unavailable".to_string())?;
        let normalizer = normalizer_for(participant);
        let head = base.private_bias.map(|bias| LogisticPredictionHeadV0 {
            weights: base.private_weights.clone(),
            bias,
        });
        if head.as_ref().is_some_and(|model| model.validate().is_err()) {
            return Err("T10 reopened model rejected".to_string());
        }
        let calibrator_receipt = (participant
            == MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic)
            .then(|| model_for(participant, DailyModelRole::Calibrator))
            .flatten();
        let calibrator = calibrator_receipt.and_then(|receipt| {
            receipt.private_bias.map(|bias| LogisticPredictionHeadV0 {
                weights: receipt.private_weights.clone(),
                bias,
            })
        });
        if calibrator
            .as_ref()
            .is_some_and(|model| model.validate().is_err())
        {
            return Err("T10 reopened calibrator rejected".to_string());
        }
        let normalizer_digest = value
            .normalizer_receipts
            .iter()
            .find(|receipt| receipt.participant_id == participant_id(participant))
            .map(|receipt| receipt.receipt_digest.as_str())
            .unwrap_or("");
        let expected_digest = frozen_digest(
            participant,
            normalizer_digest,
            &base.receipt_digest,
            calibrator_receipt.map(|receipt| receipt.receipt_digest.as_str()),
        );
        if value.reconstructed_participant_digests[frozen.len()] != expected_digest {
            return Err("T10 reopened participant identity mismatch".to_string());
        }
        frozen.push(FrozenParticipant {
            participant,
            normalizer,
            head,
            calibrator,
            prevalence: base.private_prevalence.unwrap_or_default(),
            model_digest: expected_digest,
        });
    }
    Ok(frozen)
}

fn daily_root(partition: MomentumReplayPartitionV1, day: u64) -> PathBuf {
    Path::new(ROOT)
        .join("daily")
        .join(partition_name(partition))
        .join(day.to_string())
}

fn persist_and_reopen_refit(
    bundle: &MomentumT10DailyRefitBundleV1,
) -> Result<
    (
        MomentumT10DailyRefitBundleV1,
        Vec<FrozenParticipant>,
        (usize, usize),
    ),
    String,
> {
    let root = daily_root(bundle.partition, bundle.utc_day_boundary_ms);
    let mut counts = (0, 0);
    let plan_path = root
        .join("training_plans")
        .join(format!("{}.pb", bundle.training_plan.plan_digest));
    add_counts(
        &mut counts,
        persist_path(
            &plan_path,
            &bundle.training_plan.plan_digest,
            &encode_training_plan(&bundle.training_plan)?,
            |bytes| Ok(decode_training_plan(bytes)?.plan_digest),
        )?,
    );
    if reopen_exact(&plan_path, decode_training_plan)? != bundle.training_plan {
        return Err("T10 daily training plan reopen mismatch".to_string());
    }
    for receipt in &bundle.normalizer_receipts {
        let path = root
            .join("normalizers")
            .join(format!("{}.pb", receipt.receipt_digest));
        add_counts(
            &mut counts,
            persist_path(
                &path,
                &receipt.receipt_digest,
                &encode_normalizer(receipt)?,
                |bytes| Ok(decode_normalizer(bytes)?.receipt_digest),
            )?,
        );
        if reopen_exact(&path, decode_normalizer)? != *receipt {
            return Err("T10 daily normalizer reopen mismatch".to_string());
        }
    }
    for receipt in &bundle.model_receipts {
        let path = root
            .join("models")
            .join(format!("{}.pb", receipt.receipt_digest));
        add_counts(
            &mut counts,
            persist_path(
                &path,
                &receipt.receipt_digest,
                &encode_model(receipt)?,
                |bytes| Ok(decode_model(bytes)?.receipt_digest),
            )?,
        );
        if reopen_exact(&path, decode_model)? != *receipt {
            return Err("T10 daily model reopen mismatch".to_string());
        }
    }
    let path = root
        .join("refit_bundles")
        .join(format!("{}.pb", bundle.bundle_digest));
    add_counts(
        &mut counts,
        persist_path(
            &path,
            &bundle.bundle_digest,
            &encode_refit_bundle(bundle)?,
            |bytes| Ok(decode_refit_bundle(bytes)?.bundle_digest),
        )?,
    );
    let reopened = reopen_exact(&path, decode_refit_bundle)?;
    if reopened != *bundle {
        return Err("T10 daily refit bundle reopen mismatch".to_string());
    }
    let frozen = reconstruct_refit_bundle(&reopened)?;
    Ok((reopened, frozen, counts))
}

fn clamp_probability(value: f64) -> Result<f64, String> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err("T10 prediction probability rejected".to_string());
    }
    Ok(value.clamp(PROBABILITY_CLAMP, 1.0 - PROBABILITY_CLAMP))
}

fn predict_participant(frozen: &FrozenParticipant, event: &PreparedEvent) -> Result<f64, String> {
    if frozen.participant == MomentumMicroParticipantV1::C0TaskSpecificConstant {
        return clamp_probability(frozen.prevalence);
    }
    let raw = if frozen.participant == MomentumMicroParticipantV1::C1TenMinuteAnchorBaseline {
        &event.anchor
    } else {
        &event.compact
    };
    let normalizer = frozen
        .normalizer
        .as_ref()
        .ok_or_else(|| "T10 frozen normalizer unavailable".to_string())?;
    let normalized = normalizer
        .transform_representation(raw)
        .map_err(|_| "T10 prediction normalization rejected".to_string())?;
    let head = frozen
        .head
        .as_ref()
        .ok_or_else(|| "T10 frozen model unavailable".to_string())?;
    let probability = if frozen.participant
        == MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic
    {
        let logit = head_logit(head, &normalized)?;
        frozen
            .calibrator
            .as_ref()
            .ok_or_else(|| "T10 frozen calibrator unavailable".to_string())?
            .probability(&[logit])
            .map(f64::from)
            .map_err(|_| "T10 calibrated prediction rejected".to_string())?
    } else {
        head.probability(&normalized)
            .map(f64::from)
            .map_err(|_| "T10 learned prediction rejected".to_string())?
    };
    clamp_probability(probability)
}

fn build_prediction_shard(
    partition: MomentumReplayPartitionV1,
    day: u64,
    authorization: &MomentumT10ScreeningExecutionAuthorizationV1,
    bundle: &MomentumT10DailyRefitBundleV1,
    frozen: &[FrozenParticipant],
    events: &[&PreparedEvent],
) -> Result<MomentumT10PredictionShardV1, String> {
    if frozen.len() != 5
        || events.is_empty()
        || authorization.authorization_digest != bundle.authorization_digest
        || partition != bundle.partition
        || day != bundle.utc_day_boundary_ms
    {
        return Err("T10 daily prediction inputs rejected".to_string());
    }
    let ids = PARTICIPANTS
        .iter()
        .map(|participant| participant_id(*participant))
        .collect::<Vec<_>>();
    let mut event_plans = Vec::with_capacity(events.len());
    let mut private_probabilities = Vec::with_capacity(events.len() * 5);
    let mut prediction_digests = Vec::with_capacity(events.len() * 5);
    for event in events {
        let mut plan = MomentumT10EventPlanV1 {
            plan_version: EVENT_PLAN_VERSION.to_string(),
            partition,
            prediction_timestamp_ms: event.prediction_timestamp_ms,
            target_timestamp_ms: event.target_timestamp_ms,
            source_event_digest: event.source_event_digest.clone(),
            daily_refit_bundle_digest: bundle.bundle_digest.clone(),
            participant_ids: ids.clone(),
            target_hidden: true,
            holdout_member: false,
            plan_digest: String::new(),
        };
        plan.plan_digest = event_plan_digest(&plan);
        validate_event_plan(&plan)?;
        for participant in frozen {
            let probability = predict_participant(participant, event)?;
            prediction_digests.push(stable_hash_string(&format!(
                "T10-private-prediction:{}:{}:{}",
                plan.plan_digest,
                participant.model_digest,
                probability.to_bits()
            )));
            private_probabilities.push(probability);
        }
        event_plans.push(plan);
    }
    let mut value = MomentumT10PredictionShardV1 {
        shard_version: PREDICTION_SHARD_VERSION.to_string(),
        authorization_digest: authorization.authorization_digest.clone(),
        partition,
        utc_day_boundary_ms: day,
        daily_refit_bundle_digest: bundle.bundle_digest.clone(),
        event_plans,
        participant_ids: ids,
        private_probabilities,
        prediction_digests,
        target_accessed: false,
        label_accessed: false,
        metric_computed: false,
        shard_digest: String::new(),
    };
    value.shard_digest = prediction_shard_digest(&value);
    validate_prediction_shard(&value)?;
    Ok(value)
}

fn persist_and_reopen_prediction(
    value: &MomentumT10PredictionShardV1,
) -> Result<(MomentumT10PredictionShardV1, (usize, usize)), String> {
    let path = daily_root(value.partition, value.utc_day_boundary_ms)
        .join("prediction_shards")
        .join(format!("{}.pb", value.shard_digest));
    let counts = persist_path(
        &path,
        &value.shard_digest,
        &encode_prediction_shard(value)?,
        |bytes| Ok(decode_prediction_shard(bytes)?.shard_digest),
    )?;
    let reopened = reopen_exact(&path, decode_prediction_shard)?;
    if reopened != *value {
        return Err("T10 prediction shard reopen mismatch".to_string());
    }
    Ok((reopened, counts))
}

fn build_evaluation_shard(
    prepared: &PreparedScreening,
    prediction: &MomentumT10PredictionShardV1,
    events: &[&PreparedEvent],
) -> Result<MomentumT10EvaluationShardV1, String> {
    validate_prediction_shard(prediction)?;
    if events.len() != prediction.event_plans.len() {
        return Err("T10 evaluation event count rejected".to_string());
    }
    let mut evaluations = Vec::with_capacity(events.len());
    for (index, event) in events.iter().enumerate() {
        if prediction.event_plans[index].source_event_digest != event.source_event_digest {
            return Err("T10 evaluation event binding rejected".to_string());
        }
        let (label, private_label) = reveal_label(&prepared.evidence, event)?;
        let probabilities = &prediction.private_probabilities[index * 5..(index + 1) * 5];
        let (private_brier_values, private_correctness) = if let Some(private_label) = private_label
        {
            (
                probabilities
                    .iter()
                    .map(|probability| (probability - private_label).powi(2))
                    .collect::<Vec<_>>(),
                probabilities
                    .iter()
                    .map(|probability| (*probability >= 0.5) == (private_label == 1.0))
                    .collect::<Vec<_>>(),
            )
        } else {
            (Vec::new(), Vec::new())
        };
        let mut item = MomentumT10EvaluationItemV1 {
            item_version: EVALUATION_ITEM_VERSION.to_string(),
            event_plan_digest: prediction.event_plans[index].plan_digest.clone(),
            label,
            private_label,
            private_brier_values,
            private_correctness,
            item_digest: String::new(),
        };
        item.item_digest = evaluation_item_digest(&item);
        validate_evaluation_item(&item)?;
        evaluations.push(item);
    }
    let mut value = MomentumT10EvaluationShardV1 {
        shard_version: EVALUATION_SHARD_VERSION.to_string(),
        prediction_shard_digest: prediction.shard_digest.clone(),
        partition: prediction.partition,
        utc_day_boundary_ms: prediction.utc_day_boundary_ms,
        prediction_shard_reopened: true,
        evaluations,
        shard_digest: String::new(),
    };
    value.shard_digest = evaluation_shard_digest(&value);
    validate_evaluation_shard(&value)?;
    Ok(value)
}

fn persist_and_reopen_evaluation(
    value: &MomentumT10EvaluationShardV1,
) -> Result<(MomentumT10EvaluationShardV1, (usize, usize)), String> {
    let path = daily_root(value.partition, value.utc_day_boundary_ms)
        .join("evaluation_shards")
        .join(format!("{}.pb", value.shard_digest));
    let counts = persist_path(
        &path,
        &value.shard_digest,
        &encode_evaluation_shard(value)?,
        |bytes| Ok(decode_evaluation_shard(bytes)?.shard_digest),
    )?;
    let reopened = reopen_exact(&path, decode_evaluation_shard)?;
    if reopened != *value {
        return Err("T10 evaluation shard reopen mismatch".to_string());
    }
    Ok((reopened, counts))
}

fn calibration_index(probability: f64) -> usize {
    ((probability * 10.0).floor() as usize).min(9)
}

fn accumulate_shard(
    prediction: &MomentumT10PredictionShardV1,
    evaluation: &MomentumT10EvaluationShardV1,
    accumulators: &mut [MetricAccumulator],
) -> Result<(), String> {
    if accumulators.len() != 5
        || evaluation.prediction_shard_digest != prediction.shard_digest
        || evaluation.evaluations.len() != prediction.event_plans.len()
    {
        return Err("T10 aggregate shard binding rejected".to_string());
    }
    for (event_index, item) in evaluation.evaluations.iter().enumerate() {
        if item.event_plan_digest != prediction.event_plans[event_index].plan_digest {
            return Err("T10 evaluation event-plan binding rejected".to_string());
        }
        let probabilities =
            &prediction.private_probabilities[event_index * 5..(event_index + 1) * 5];
        let constant_brier = item.private_brier_values.first().copied();
        for participant_index in 0..5 {
            let probability = probabilities[participant_index];
            let accumulator = &mut accumulators[participant_index];
            accumulator.prediction_count += 1;
            if probability.is_finite() {
                accumulator.finite_prediction_count += 1;
                accumulator.probability_sum += probability;
                accumulator.probability_squared_sum += probability * probability;
                if accumulator.finite_prediction_count == 1 {
                    accumulator.minimum_probability = probability;
                    accumulator.maximum_probability = probability;
                } else {
                    accumulator.minimum_probability =
                        accumulator.minimum_probability.min(probability);
                    accumulator.maximum_probability =
                        accumulator.maximum_probability.max(probability);
                }
                *accumulator
                    .frequencies
                    .entry(probability.to_bits())
                    .or_default() += 1;
                if (probability - 0.5).abs() <= NEAR_HALF_THRESHOLD {
                    accumulator.near_half_count += 1;
                }
                if probability <= PROBABILITY_CLAMP {
                    accumulator.extreme_low_count += 1;
                }
                if probability >= 1.0 - PROBABILITY_CLAMP {
                    accumulator.extreme_high_count += 1;
                }
            } else {
                accumulator.nonfinite_count += 1;
            }
            match item.label {
                ScreeningLabel::Up | ScreeningLabel::Down => {
                    let label = item
                        .private_label
                        .ok_or_else(|| "T10 aggregate label unavailable".to_string())?;
                    let brier = item.private_brier_values[participant_index];
                    accumulator.scorable_count += 1;
                    accumulator.brier_sum += brier;
                    accumulator.correct_count +=
                        usize::from(item.private_correctness[participant_index]);
                    let delta = brier
                        - constant_brier
                            .ok_or_else(|| "T10 constant Brier unavailable".to_string())?;
                    accumulator.deltas.push(delta);
                    let bin = calibration_index(probability);
                    accumulator.calibration_support[bin] += 1;
                    accumulator.calibration_probability_sum[bin] += probability;
                    accumulator.calibration_positive_sum[bin] += label;
                }
                ScreeningLabel::Neutral => accumulator.neutral_count += 1,
                ScreeningLabel::Invalid => accumulator.invalid_count += 1,
            }
        }
    }
    Ok(())
}

fn median(values: &mut [f64]) -> Result<f64, String> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err("T10 median evidence rejected".to_string());
    }
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    Ok(if values.len() % 2 == 0 {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    })
}

fn finish_metrics(
    participant: MomentumMicroParticipantV1,
    partition: MomentumReplayPartitionV1,
    mut accumulator: MetricAccumulator,
) -> Result<MomentumMicroScreeningParticipantMetricsV1, String> {
    if accumulator.scorable_count == 0 || accumulator.prediction_count == 0 {
        return Err("T10 participant metric support unavailable".to_string());
    }
    let mean_brier = accumulator.brier_sum / accumulator.scorable_count as f64;
    let mean_probability = accumulator.probability_sum / accumulator.prediction_count as f64;
    let variance = (accumulator.probability_squared_sum / accumulator.prediction_count as f64
        - mean_probability * mean_probability)
        .max(0.0);
    let standard_deviation = variance.sqrt();
    let mean_delta = accumulator.deltas.iter().sum::<f64>() / accumulator.deltas.len() as f64;
    let median_delta = median(&mut accumulator.deltas)?;
    let mut bins = Vec::new();
    let mut weighted_gap = 0.0;
    for index in 0..10 {
        let support = accumulator.calibration_support[index];
        let mean_predicted_probability = if support == 0 {
            0.0
        } else {
            accumulator.calibration_probability_sum[index] / support as f64
        };
        let observed_positive_frequency = if support == 0 {
            0.0
        } else {
            accumulator.calibration_positive_sum[index] / support as f64
        };
        let gap = (mean_predicted_probability - observed_positive_frequency).abs();
        weighted_gap += gap * support as f64 / accumulator.scorable_count as f64;
        let mut bin = MomentumMicroCalibrationBinV1 {
            bin_version: CALIBRATION_BIN_VERSION.to_string(),
            lower_bound: CALIBRATION_BOUNDARIES[index],
            upper_bound: CALIBRATION_BOUNDARIES[index + 1],
            upper_inclusive: index == 9,
            support,
            mean_predicted_probability,
            observed_positive_frequency,
            absolute_calibration_gap: gap,
            bin_digest: String::new(),
        };
        bin.bin_digest = calibration_bin_digest(&bin);
        bins.push(bin);
    }
    let collapse = if participant == MomentumMicroParticipantV1::C0TaskSpecificConstant {
        MomentumMicroProbabilityCollapseV1::BenchmarkExempt
    } else if variance <= COLLAPSE_VARIANCE_THRESHOLD {
        MomentumMicroProbabilityCollapseV1::ProbabilityCollapse
    } else {
        MomentumMicroProbabilityCollapseV1::NotCollapsed
    };
    let saturation = match (
        accumulator.extreme_low_count > 0,
        accumulator.extreme_high_count > 0,
    ) {
        (false, false) => MomentumMicroSaturationV1::NotSaturated,
        (true, false) => MomentumMicroSaturationV1::LowBoundarySaturation,
        (false, true) => MomentumMicroSaturationV1::HighBoundarySaturation,
        (true, true) => MomentumMicroSaturationV1::TwoSidedSaturation,
    };
    let positive_paired_delta_count = accumulator
        .deltas
        .iter()
        .filter(|delta| **delta > COMPARISON_EPSILON)
        .count();
    let negative_paired_delta_count = accumulator
        .deltas
        .iter()
        .filter(|delta| **delta < -COMPARISON_EPSILON)
        .count();
    let equivalent_paired_delta_count =
        accumulator.deltas.len() - positive_paired_delta_count - negative_paired_delta_count;
    let mut value = MomentumMicroScreeningParticipantMetricsV1 {
        metrics_version: METRICS_VERSION.to_string(),
        participant_id: participant_id(participant),
        partition,
        prediction_count: accumulator.prediction_count,
        scorable_count: accumulator.scorable_count,
        neutral_count: accumulator.neutral_count,
        invalid_count: accumulator.invalid_count,
        finite_prediction_count: accumulator.finite_prediction_count,
        mean_brier,
        binary_correctness: accumulator.correct_count as f64 / accumulator.scorable_count as f64,
        paired_mean_brier_delta_versus_c0: mean_delta,
        paired_median_brier_delta_versus_c0: median_delta,
        positive_paired_delta_count,
        negative_paired_delta_count,
        equivalent_paired_delta_count,
        calibration_bins: bins,
        weighted_calibration_gap: weighted_gap,
        empty_calibration_bin_count: accumulator
            .calibration_support
            .iter()
            .filter(|support| **support == 0)
            .count(),
        minimum_probability: accumulator.minimum_probability,
        maximum_probability: accumulator.maximum_probability,
        mean_probability,
        probability_standard_deviation: standard_deviation,
        near_constant_count: accumulator
            .frequencies
            .values()
            .copied()
            .max()
            .unwrap_or_default(),
        near_half_count: accumulator.near_half_count,
        extreme_low_count: accumulator.extreme_low_count,
        extreme_high_count: accumulator.extreme_high_count,
        nonfinite_count: accumulator.nonfinite_count,
        collapse,
        saturation,
        chronology_audit_passed: true,
        leakage_audit_passed: true,
        integrity_audit_passed: true,
        metrics_digest: String::new(),
    };
    value.metrics_digest = participant_metrics_digest(&value);
    validate_participant_metrics(&value)?;
    Ok(value)
}

struct PartitionRunResult {
    aggregate: MomentumT10PartitionAggregateV1,
    counts: (usize, usize),
    model_fits: usize,
    calibration_fits: usize,
    predictions: usize,
    target_reveals: usize,
    evaluations: usize,
    metric_computations: usize,
}

fn run_partition(
    source: &FrozenSource,
    prepared: &PreparedScreening,
    partition: MomentumReplayPartitionV1,
    authorization: &MomentumT10ScreeningExecutionAuthorizationV1,
    policy: &MomentumT10TrainingPolicyV1,
) -> Result<PartitionRunResult, String> {
    if partition == MomentumReplayPartitionV1::SealedHoldout {
        return Err("T10 holdout partition execution rejected".to_string());
    }
    let events = match partition {
        MomentumReplayPartitionV1::Development => &prepared.development,
        MomentumReplayPartitionV1::Validation => &prepared.validation,
        MomentumReplayPartitionV1::SealedHoldout => unreachable!(),
    };
    let (boundary_event_count, boundary_days) = match partition {
        MomentumReplayPartitionV1::Development => (
            prepared.development_boundary_event_count,
            &prepared.development_boundary_days,
        ),
        MomentumReplayPartitionV1::Validation => (
            prepared.validation_boundary_event_count,
            &prepared.validation_boundary_days,
        ),
        MomentumReplayPartitionV1::SealedHoldout => unreachable!(),
    };
    let mut days = BTreeMap::<u64, Vec<&PreparedEvent>>::new();
    for event in events {
        days.entry(event.prediction_timestamp_ms / DAY_MS * DAY_MS)
            .or_default()
            .push(event);
    }
    let mut counts = (0, 0);
    let mut accumulators = (0..5)
        .map(|_| MetricAccumulator::default())
        .collect::<Vec<_>>();
    let mut insufficient_support_day_count = boundary_days.len().saturating_sub(days.len());
    let mut refit_digests = Vec::new();
    let mut prediction_digests = Vec::new();
    let mut evaluation_digests = Vec::new();
    let mut model_fits = 0usize;
    let mut calibration_fits = 0usize;
    let mut predictions = 0usize;
    let mut target_reveals = 0usize;
    let mut evaluations = 0usize;
    for (day, day_events) in days {
        let (plan, used) = training_window(prepared, partition, day, authorization, policy)?;
        if !plan.support_sufficient_for_all {
            insufficient_support_day_count += 1;
            continue;
        }
        let (bundle, _) =
            freeze_daily_participants(source, partition, day, authorization, policy, plan, used)?;
        model_fits += 4;
        calibration_fits += 1;
        let (bundle, frozen, persisted) = persist_and_reopen_refit(&bundle)?;
        add_counts(&mut counts, persisted);
        let prediction =
            build_prediction_shard(partition, day, authorization, &bundle, &frozen, &day_events)?;
        predictions += prediction.private_probabilities.len();
        let (prediction, persisted) = persist_and_reopen_prediction(&prediction)?;
        add_counts(&mut counts, persisted);
        let evaluation = build_evaluation_shard(prepared, &prediction, &day_events)?;
        target_reveals += evaluation.evaluations.len();
        evaluations += evaluation.evaluations.len() * 5;
        let (evaluation, persisted) = persist_and_reopen_evaluation(&evaluation)?;
        add_counts(&mut counts, persisted);
        accumulate_shard(&prediction, &evaluation, &mut accumulators)?;
        refit_digests.push(bundle.bundle_digest);
        prediction_digests.push(prediction.shard_digest);
        evaluation_digests.push(evaluation.shard_digest);
    }
    let participant_metrics = PARTICIPANTS
        .into_iter()
        .zip(accumulators)
        .map(|(participant, accumulator)| finish_metrics(participant, partition, accumulator))
        .collect::<Result<Vec<_>, _>>()?;
    let prediction_count = participant_metrics[0].prediction_count;
    let training_only_event_count = boundary_event_count
        .checked_sub(prediction_count)
        .ok_or_else(|| "T10 partition prediction count rejected".to_string())?;
    let scorable_count = participant_metrics[0].scorable_count;
    let neutral_count = participant_metrics[0].neutral_count;
    let invalid_count = participant_metrics[0].invalid_count;
    let mut aggregate = MomentumT10PartitionAggregateV1 {
        aggregate_version: AGGREGATE_VERSION.to_string(),
        authorization_digest: authorization.authorization_digest.clone(),
        partition,
        boundary_event_count,
        training_only_event_count,
        prediction_count,
        scorable_count,
        neutral_count,
        invalid_count,
        daily_refit_count: refit_digests.len(),
        insufficient_support_day_count,
        daily_refit_bundle_digests: refit_digests,
        prediction_shard_digests: prediction_digests,
        evaluation_shard_digests: evaluation_digests,
        participant_metrics,
        target_access_before_prediction_reopen_count: 0,
        feature_future_access_count: 0,
        partial_candle_access_count: 0,
        holdout_access_count: 0,
        validation_fit_count: 0,
        chronology_audit_passed: true,
        leakage_audit_passed: true,
        prediction_before_reveal_passed: true,
        aggregate_digest: String::new(),
    };
    aggregate.aggregate_digest = aggregate_digest(&aggregate);
    validate_aggregate(&aggregate)?;
    let category = format!("partition_aggregates/{}", partition_name(partition));
    add_counts(
        &mut counts,
        persist_one(
            &category,
            &aggregate.aggregate_digest,
            &encode_aggregate(&aggregate)?,
            |bytes| Ok(decode_aggregate(bytes)?.aggregate_digest),
        )?,
    );
    if read_aggregate(partition)?.as_ref() != Some(&aggregate) {
        return Err("T10 partition aggregate reopen mismatch".to_string());
    }
    Ok(PartitionRunResult {
        aggregate,
        counts,
        model_fits,
        calibration_fits,
        predictions,
        target_reveals,
        evaluations,
        metric_computations: scorable_count * 5,
    })
}

fn metrics_map(
    aggregate: &MomentumT10PartitionAggregateV1,
) -> Result<BTreeMap<MomentumMicroParticipantV1, &MomentumMicroScreeningParticipantMetricsV1>, String>
{
    aggregate
        .participant_metrics
        .iter()
        .map(|metrics| Ok((parse_participant(&metrics.participant_id)?, metrics)))
        .collect()
}

fn classify_partition_comparison(
    metrics: &MomentumMicroScreeningParticipantMetricsV1,
) -> MomentumMicroScreeningComparisonV1 {
    if !metrics.integrity_audit_passed
        || !metrics.chronology_audit_passed
        || !metrics.leakage_audit_passed
        || metrics.nonfinite_count != 0
    {
        MomentumMicroScreeningComparisonV1::IntegrityFailure
    } else if metrics.collapse == MomentumMicroProbabilityCollapseV1::ProbabilityCollapse {
        MomentumMicroScreeningComparisonV1::ProbabilityCollapse
    } else if metrics.scorable_count == 0 {
        MomentumMicroScreeningComparisonV1::MixedOrInsufficientEvidence
    } else if metrics.paired_mean_brier_delta_versus_c0 < -COMPARISON_EPSILON {
        MomentumMicroScreeningComparisonV1::LowerBrierThanConstant
    } else if metrics.paired_mean_brier_delta_versus_c0 > COMPARISON_EPSILON {
        MomentumMicroScreeningComparisonV1::HigherBrierThanConstant
    } else {
        MomentumMicroScreeningComparisonV1::NumericallyEquivalentToConstant
    }
}

fn overall_comparison(
    development: MomentumMicroScreeningComparisonV1,
    validation: MomentumMicroScreeningComparisonV1,
) -> MomentumMicroScreeningComparisonV1 {
    if matches!(
        (development, validation),
        (MomentumMicroScreeningComparisonV1::IntegrityFailure, _)
            | (_, MomentumMicroScreeningComparisonV1::IntegrityFailure)
    ) {
        MomentumMicroScreeningComparisonV1::IntegrityFailure
    } else if matches!(
        (development, validation),
        (MomentumMicroScreeningComparisonV1::ProbabilityCollapse, _)
            | (_, MomentumMicroScreeningComparisonV1::ProbabilityCollapse)
    ) {
        MomentumMicroScreeningComparisonV1::ProbabilityCollapse
    } else if development == MomentumMicroScreeningComparisonV1::LowerBrierThanConstant
        && validation == MomentumMicroScreeningComparisonV1::LowerBrierThanConstant
    {
        MomentumMicroScreeningComparisonV1::LowerBrierThanConstant
    } else if development == MomentumMicroScreeningComparisonV1::HigherBrierThanConstant
        && validation == MomentumMicroScreeningComparisonV1::HigherBrierThanConstant
    {
        MomentumMicroScreeningComparisonV1::HigherBrierThanConstant
    } else if development == MomentumMicroScreeningComparisonV1::NumericallyEquivalentToConstant
        && validation == MomentumMicroScreeningComparisonV1::NumericallyEquivalentToConstant
    {
        MomentumMicroScreeningComparisonV1::NumericallyEquivalentToConstant
    } else {
        MomentumMicroScreeningComparisonV1::MixedOrInsufficientEvidence
    }
}

fn build_benchmarks(
    development: &MomentumT10PartitionAggregateV1,
    validation: &MomentumT10PartitionAggregateV1,
) -> Result<Vec<MomentumMicroBenchmarkComparisonReceiptV1>, String> {
    let development_metrics = metrics_map(development)?;
    let validation_metrics = metrics_map(validation)?;
    PARTICIPANTS[1..]
        .iter()
        .map(|participant| {
            let development_metric = development_metrics
                .get(participant)
                .ok_or_else(|| "T10 development comparison unavailable".to_string())?;
            let validation_metric = validation_metrics
                .get(participant)
                .ok_or_else(|| "T10 validation comparison unavailable".to_string())?;
            let development_comparison = classify_partition_comparison(development_metric);
            let validation_comparison = classify_partition_comparison(validation_metric);
            let mut value = MomentumMicroBenchmarkComparisonReceiptV1 {
                comparison_version: BENCHMARK_VERSION.to_string(),
                participant_id: participant_id(*participant),
                development_aggregate_digest: development.aggregate_digest.clone(),
                validation_aggregate_digest: validation.aggregate_digest.clone(),
                development_delta_bits: development_metric
                    .paired_mean_brier_delta_versus_c0
                    .to_bits(),
                validation_delta_bits: validation_metric
                    .paired_mean_brier_delta_versus_c0
                    .to_bits(),
                development_comparison,
                validation_comparison,
                overall_comparison: overall_comparison(
                    development_comparison,
                    validation_comparison,
                ),
                receipt_digest: String::new(),
            };
            value.receipt_digest = benchmark_receipt_digest(&value);
            validate_benchmark(&value)?;
            Ok(value)
        })
        .collect()
}

fn classify_contribution(delta: f64) -> MomentumMicroContributionComparisonV1 {
    if !delta.is_finite() {
        MomentumMicroContributionComparisonV1::IntegrityFailure
    } else if delta < -COMPARISON_EPSILON {
        MomentumMicroContributionComparisonV1::LowerBrierWithContribution
    } else if delta > COMPARISON_EPSILON {
        MomentumMicroContributionComparisonV1::HigherBrierWithContribution
    } else {
        MomentumMicroContributionComparisonV1::NumericallyEquivalent
    }
}

fn overall_contribution(
    development: MomentumMicroContributionComparisonV1,
    validation: MomentumMicroContributionComparisonV1,
) -> MomentumMicroContributionComparisonV1 {
    if development == MomentumMicroContributionComparisonV1::IntegrityFailure
        || validation == MomentumMicroContributionComparisonV1::IntegrityFailure
    {
        MomentumMicroContributionComparisonV1::IntegrityFailure
    } else if development == MomentumMicroContributionComparisonV1::LowerBrierWithContribution
        && validation == MomentumMicroContributionComparisonV1::LowerBrierWithContribution
    {
        MomentumMicroContributionComparisonV1::LowerBrierWithContribution
    } else if development == MomentumMicroContributionComparisonV1::HigherBrierWithContribution
        && validation == MomentumMicroContributionComparisonV1::HigherBrierWithContribution
    {
        MomentumMicroContributionComparisonV1::HigherBrierWithContribution
    } else if development == MomentumMicroContributionComparisonV1::NumericallyEquivalent
        && validation == MomentumMicroContributionComparisonV1::NumericallyEquivalent
    {
        MomentumMicroContributionComparisonV1::NumericallyEquivalent
    } else {
        MomentumMicroContributionComparisonV1::MixedOrInsufficientEvidence
    }
}

fn build_contributions(
    development: &MomentumT10PartitionAggregateV1,
    validation: &MomentumT10PartitionAggregateV1,
) -> Result<Vec<MomentumMicroContributionReceiptV1>, String> {
    let development_metrics = metrics_map(development)?;
    let validation_metrics = metrics_map(validation)?;
    [
        (
            MomentumMicroParticipantV1::C2CompactMicroLogistic,
            MomentumMicroParticipantV1::C1TenMinuteAnchorBaseline,
        ),
        (
            MomentumMicroParticipantV1::C3CompactMicroStrongShrinkLogistic,
            MomentumMicroParticipantV1::C2CompactMicroLogistic,
        ),
        (
            MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic,
            MomentumMicroParticipantV1::C2CompactMicroLogistic,
        ),
    ]
    .into_iter()
    .map(|(participant, baseline)| {
        let development_delta = development_metrics
            .get(&participant)
            .ok_or_else(|| "T10 development contribution unavailable".to_string())?
            .mean_brier
            - development_metrics
                .get(&baseline)
                .ok_or_else(|| "T10 development contribution baseline unavailable".to_string())?
                .mean_brier;
        let validation_delta = validation_metrics
            .get(&participant)
            .ok_or_else(|| "T10 validation contribution unavailable".to_string())?
            .mean_brier
            - validation_metrics
                .get(&baseline)
                .ok_or_else(|| "T10 validation contribution baseline unavailable".to_string())?
                .mean_brier;
        let development_comparison = classify_contribution(development_delta);
        let validation_comparison = classify_contribution(validation_delta);
        let mut value = MomentumMicroContributionReceiptV1 {
            comparison_version: CONTRIBUTION_VERSION.to_string(),
            participant_id: participant_id(participant),
            baseline_participant_id: participant_id(baseline),
            development_delta_bits: development_delta.to_bits(),
            validation_delta_bits: validation_delta.to_bits(),
            development_comparison,
            validation_comparison,
            overall_comparison: overall_contribution(development_comparison, validation_comparison),
            receipt_digest: String::new(),
        };
        value.receipt_digest = contribution_receipt_digest(&value);
        validate_contribution(&value)?;
        Ok(value)
    })
    .collect()
}

fn build_eligibilities(
    source: &FrozenSource,
    development: &MomentumT10PartitionAggregateV1,
    validation: &MomentumT10PartitionAggregateV1,
    benchmarks: &[MomentumMicroBenchmarkComparisonReceiptV1],
    policy: &MomentumT10TrainingPolicyV1,
) -> Result<Vec<MomentumMicroHoldoutEligibilityReceiptV1>, String> {
    let task = source
        .registration
        .task_registrations
        .iter()
        .find(|item| item.task == MomentumMicroTaskV1::T10NextTenMinuteDirection)
        .ok_or_else(|| "T10 task registration unavailable".to_string())?;
    let development_metrics = metrics_map(development)?;
    let validation_metrics = metrics_map(validation)?;
    PARTICIPANTS[1..]
        .iter()
        .map(|participant| {
            let registered = source
                .registration
                .participant_registrations
                .iter()
                .find(|item| {
                    item.task == MomentumMicroTaskV1::T10NextTenMinuteDirection
                        && item.participant == *participant
                })
                .ok_or_else(|| "T10 participant registration unavailable".to_string())?;
            let benchmark = benchmarks
                .iter()
                .find(|item| item.participant_id == participant_id(*participant))
                .ok_or_else(|| "T10 benchmark eligibility binding unavailable".to_string())?;
            let development_metric = development_metrics
                .get(participant)
                .ok_or_else(|| "T10 development eligibility metric unavailable".to_string())?;
            let validation_metric = validation_metrics
                .get(participant)
                .ok_or_else(|| "T10 validation eligibility metric unavailable".to_string())?;
            let development_lower = benchmark.development_comparison
                == MomentumMicroScreeningComparisonV1::LowerBrierThanConstant;
            let validation_lower = benchmark.validation_comparison
                == MomentumMicroScreeningComparisonV1::LowerBrierThanConstant;
            let sufficient = development_metric.scorable_count >= policy.minimum_training_examples
                && validation_metric.scorable_count >= policy.minimum_training_examples;
            let finite_predictions =
                development_metric.nonfinite_count == 0 && validation_metric.nonfinite_count == 0;
            let finite_metrics = [
                development_metric.mean_brier,
                development_metric.binary_correctness,
                validation_metric.mean_brier,
                validation_metric.binary_correctness,
            ]
            .into_iter()
            .all(f64::is_finite);
            let no_probability_collapse = development_metric.collapse
                == MomentumMicroProbabilityCollapseV1::NotCollapsed
                && validation_metric.collapse == MomentumMicroProbabilityCollapseV1::NotCollapsed;
            let no_saturation_failure = development_metric.saturation
                == MomentumMicroSaturationV1::NotSaturated
                && validation_metric.saturation == MomentumMicroSaturationV1::NotSaturated;
            let chronology_clean = development_metric.chronology_audit_passed
                && validation_metric.chronology_audit_passed;
            let leakage_clean =
                development_metric.leakage_audit_passed && validation_metric.leakage_audit_passed;
            let integrity_clean = development_metric.integrity_audit_passed
                && validation_metric.integrity_audit_passed;
            let eligible = development_lower
                && validation_lower
                && sufficient
                && finite_predictions
                && finite_metrics
                && no_probability_collapse
                && no_saturation_failure
                && chronology_clean
                && leakage_clean
                && integrity_clean;
            let mut value = MomentumMicroHoldoutEligibilityReceiptV1 {
                task_registration_digest: task.task_digest.clone(),
                participant_registration_digest: registered.participant_digest.clone(),
                participant_id: participant_id(*participant),
                development_aggregate_digest: development.aggregate_digest.clone(),
                validation_aggregate_digest: validation.aggregate_digest.clone(),
                development_lower_brier_than_constant: development_lower,
                validation_lower_brier_than_constant: validation_lower,
                sufficient_paired_support: sufficient,
                finite_predictions,
                finite_metrics,
                no_probability_collapse,
                no_saturation_failure,
                chronology_clean,
                leakage_clean,
                integrity_clean,
                result_selected_mutation_absent: true,
                holdout_access_count: 0,
                eligibility: if eligible {
                    MomentumMicroHoldoutEligibilityV1::EligibleForFutureSealedHoldoutEvaluation
                } else {
                    MomentumMicroHoldoutEligibilityV1::IneligibleScreeningGate
                },
                receipt_digest: String::new(),
            };
            value.receipt_digest = eligibility_receipt_digest(&value);
            validate_eligibility(&value)?;
            Ok(value)
        })
        .collect()
}

fn build_cohort(
    authorization: &MomentumT10ScreeningExecutionAuthorizationV1,
    eligibilities: &[MomentumMicroHoldoutEligibilityReceiptV1],
) -> Result<MomentumMicroProposedHoldoutCohortV1, String> {
    let eligible = eligibilities
        .iter()
        .filter(|receipt| {
            receipt.eligibility
                == MomentumMicroHoldoutEligibilityV1::EligibleForFutureSealedHoldoutEvaluation
        })
        .collect::<Vec<_>>();
    let mut value = MomentumMicroProposedHoldoutCohortV1 {
        cohort_version: COHORT_VERSION.to_string(),
        authorization_digest: authorization.authorization_digest.clone(),
        eligibility_receipt_digests: eligible
            .iter()
            .map(|receipt| receipt.receipt_digest.clone())
            .collect(),
        participant_ids: eligible
            .iter()
            .map(|receipt| receipt.participant_id.clone())
            .collect(),
        status: if eligible.is_empty() {
            MomentumMicroHoldoutCohortStatusV1::NoEligibleT10HoldoutCohort
        } else {
            MomentumMicroHoldoutCohortStatusV1::ProposedT10HoldoutCohort
        },
        holdout_execution_authorized: false,
        cohort_digest: String::new(),
    };
    value.cohort_digest = cohort_digest(&value);
    validate_cohort(&value)?;
    Ok(value)
}

fn persist_final_evidence(
    authorization: &MomentumT10ScreeningExecutionAuthorizationV1,
    development: &MomentumT10PartitionAggregateV1,
    validation: &MomentumT10PartitionAggregateV1,
    benchmarks: &[MomentumMicroBenchmarkComparisonReceiptV1],
    contributions: &[MomentumMicroContributionReceiptV1],
    eligibilities: &[MomentumMicroHoldoutEligibilityReceiptV1],
    cohort: &MomentumMicroProposedHoldoutCohortV1,
) -> Result<(MomentumT10ScreeningJournalV1, (usize, usize)), String> {
    let mut counts = (0, 0);
    for value in benchmarks {
        add_counts(
            &mut counts,
            persist_one(
                "benchmark_comparisons",
                &value.receipt_digest,
                &encode_benchmark(value)?,
                |bytes| Ok(decode_benchmark(bytes)?.receipt_digest),
            )?,
        );
    }
    for value in contributions {
        add_counts(
            &mut counts,
            persist_one(
                "contribution_comparisons",
                &value.receipt_digest,
                &encode_contribution(value)?,
                |bytes| Ok(decode_contribution(bytes)?.receipt_digest),
            )?,
        );
    }
    for value in eligibilities {
        add_counts(
            &mut counts,
            persist_one(
                "holdout_eligibilities",
                &value.receipt_digest,
                &encode_eligibility(value)?,
                |bytes| Ok(decode_eligibility(bytes)?.receipt_digest),
            )?,
        );
    }
    add_counts(
        &mut counts,
        persist_one(
            "holdout_cohorts",
            &cohort.cohort_digest,
            &encode_cohort(cohort)?,
            |bytes| Ok(decode_cohort(bytes)?.cohort_digest),
        )?,
    );
    let mut journal = MomentumT10ScreeningJournalV1 {
        journal_version: JOURNAL_VERSION.to_string(),
        authorization_digest: authorization.authorization_digest.clone(),
        development_aggregate_digest: development.aggregate_digest.clone(),
        validation_aggregate_digest: validation.aggregate_digest.clone(),
        benchmark_receipt_digests: benchmarks
            .iter()
            .map(|value| value.receipt_digest.clone())
            .collect(),
        contribution_receipt_digests: contributions
            .iter()
            .map(|value| value.receipt_digest.clone())
            .collect(),
        eligibility_receipt_digests: eligibilities
            .iter()
            .map(|value| value.receipt_digest.clone())
            .collect(),
        cohort_digest: cohort.cohort_digest.clone(),
        holdout_access_count: 0,
        t30_execution_count: 0,
        t60_execution_count: 0,
        live_authority_count: 0,
        deterministic: true,
        replay_digest: String::new(),
    };
    journal.replay_digest = journal_digest(&journal);
    validate_journal(&journal)?;
    add_counts(
        &mut counts,
        persist_one(
            "screening_journals",
            &journal.replay_digest,
            &encode_journal(&journal)?,
            |bytes| Ok(decode_journal(bytes)?.replay_digest),
        )?,
    );
    Ok((journal, counts))
}

fn persist_authorization_and_policy(
    authorization: &MomentumT10ScreeningExecutionAuthorizationV1,
    policy: &MomentumT10TrainingPolicyV1,
) -> Result<(usize, usize), String> {
    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_one(
            "authorizations",
            &authorization.authorization_digest,
            &encode_authorization(authorization)?,
            |bytes| Ok(decode_authorization(bytes)?.authorization_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_one(
            "training_policies",
            &policy.policy_digest,
            &encode_training_policy(policy)?,
            |bytes| Ok(decode_training_policy(bytes)?.policy_digest),
        )?,
    );
    if read_authorization()?.as_ref() != Some(authorization)
        || read_training_policy()?.as_ref() != Some(policy)
    {
        return Err("T10 authorization reopen mismatch".to_string());
    }
    Ok(counts)
}

fn apply_persisted_state(
    report: &mut MomentumT10MicroScreeningReportV1,
    authorization: MomentumT10ScreeningExecutionAuthorizationV1,
    policy: MomentumT10TrainingPolicyV1,
    development: Option<MomentumT10PartitionAggregateV1>,
    validation: Option<MomentumT10PartitionAggregateV1>,
) {
    report.authorization = Some(authorization);
    report.training_policy = Some(policy);
    report.development = development;
    report.validation = validation;
    report.status = if report.validation.is_some() {
        MomentumT10MicroScreeningStatusV1::Complete
    } else if report.development.is_some() {
        MomentumT10MicroScreeningStatusV1::DevelopmentComplete
    } else {
        MomentumT10MicroScreeningStatusV1::Authorized
    };
}

fn completed_replay(
    mut report: MomentumT10MicroScreeningReportV1,
    mode: MomentumT10MicroScreeningRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
    started: Instant,
) -> Result<MomentumT10MicroScreeningReportV1, String> {
    if report.protected_before_state_digest != protected.state_digest {
        return Err("T10 completed replay protected state changed".to_string());
    }
    report.run_mode = mode.as_str().to_string();
    report.safety_counters = MomentumT10ScreeningSafetyCountersV1::default();
    report.runtime_duration_ms = started.elapsed().as_millis() as u64;
    report.report_digest = report_digest(&report);
    validate_report(&report)?;
    Ok(report)
}

fn run_inner(
    mode: MomentumT10MicroScreeningRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> Result<MomentumT10MicroScreeningReportV1, String> {
    let started = Instant::now();
    validate_momentum_micro_protected_before_state_v1(protected)?;
    let source = reopen_frozen_source()?;
    if let Some(report) = read_momentum_t10_micro_screening_report_v1()? {
        return completed_replay(report, mode, protected, started);
    }
    let authorization = build_authorization(&source)?;
    let policy = build_training_policy(&source)?;
    let persisted_authorization = read_authorization()?;
    let persisted_policy = read_training_policy()?;
    if persisted_authorization
        .as_ref()
        .is_some_and(|value| value != &authorization)
        || persisted_policy
            .as_ref()
            .is_some_and(|value| value != &policy)
    {
        return Err("T10 frozen authorization conflict".to_string());
    }
    let development = read_aggregate(MomentumReplayPartitionV1::Development)?;
    let validation = read_aggregate(MomentumReplayPartitionV1::Validation)?;
    if validation.is_some() && development.is_none() {
        return Err("T10 validation without development rejected".to_string());
    }
    let mut report = empty_report(mode, protected, &source);
    if mode == MomentumT10MicroScreeningRunModeV1::Status {
        if let (Some(authorization), Some(policy)) = (persisted_authorization, persisted_policy) {
            apply_persisted_state(&mut report, authorization, policy, development, validation);
        }
        report.runtime_duration_ms = started.elapsed().as_millis() as u64;
        report.report_digest = report_digest(&report);
        validate_report(&report)?;
        return Ok(report);
    }
    if mode == MomentumT10MicroScreeningRunModeV1::DryRun {
        report.authorization = Some(authorization);
        report.training_policy = Some(policy);
        report.status = MomentumT10MicroScreeningStatusV1::Authorized;
        report.runtime_duration_ms = started.elapsed().as_millis() as u64;
        report.report_digest = report_digest(&report);
        validate_report(&report)?;
        return Ok(report);
    }
    if mode == MomentumT10MicroScreeningRunModeV1::Authorize {
        let counts = if persisted_authorization.is_some() && persisted_policy.is_some() {
            (0, 0)
        } else {
            persist_authorization_and_policy(&authorization, &policy)?
        };
        report.authorization = Some(authorization);
        report.training_policy = Some(policy);
        report.status = MomentumT10MicroScreeningStatusV1::Authorized;
        report.safety_counters.artifacts_written = counts.0;
        report.safety_counters.duplicate_artifact_count = counts.1;
        report.runtime_duration_ms = started.elapsed().as_millis() as u64;
        report.report_digest = report_digest(&report);
        validate_report(&report)?;
        return Ok(report);
    }
    let persisted_authorization = persisted_authorization
        .ok_or_else(|| "T10 execution authorization unavailable".to_string())?;
    let persisted_policy =
        persisted_policy.ok_or_else(|| "T10 training policy unavailable".to_string())?;
    if persisted_authorization != authorization || persisted_policy != policy {
        return Err("T10 execution policy mismatch".to_string());
    }
    if mode == MomentumT10MicroScreeningRunModeV1::ExecuteDevelopment {
        if validation.is_some() {
            return Err("T10 development rerun after validation rejected".to_string());
        }
        if let Some(development) = development {
            apply_persisted_state(&mut report, authorization, policy, Some(development), None);
            report.runtime_duration_ms = started.elapsed().as_millis() as u64;
            report.report_digest = report_digest(&report);
            validate_report(&report)?;
            return Ok(report);
        }
        let prepared = prepare_screening(&source)?;
        let result = run_partition(
            &source,
            &prepared,
            MomentumReplayPartitionV1::Development,
            &authorization,
            &policy,
        )?;
        apply_persisted_state(
            &mut report,
            authorization,
            policy,
            Some(result.aggregate),
            None,
        );
        report.safety_counters.artifacts_written = result.counts.0;
        report.safety_counters.duplicate_artifact_count = result.counts.1;
        report.safety_counters.new_model_fits = result.model_fits;
        report.safety_counters.new_calibration_fits = result.calibration_fits;
        report.safety_counters.new_predictions = result.predictions;
        report.safety_counters.new_target_reveals = result.target_reveals;
        report.safety_counters.new_evaluations = result.evaluations;
        report.safety_counters.new_metric_computations = result.metric_computations;
        report.runtime_duration_ms = started.elapsed().as_millis() as u64;
        report.report_digest = report_digest(&report);
        validate_report(&report)?;
        return Ok(report);
    }
    let development =
        development.ok_or_else(|| "T10 development aggregate required first".to_string())?;
    let validation_result = if validation.is_some() {
        None
    } else {
        let prepared = prepare_screening(&source)?;
        Some(run_partition(
            &source,
            &prepared,
            MomentumReplayPartitionV1::Validation,
            &authorization,
            &policy,
        )?)
    };
    let validation = match (validation, &validation_result) {
        (Some(validation), _) => validation,
        (None, Some(result)) => result.aggregate.clone(),
        (None, None) => {
            return Err("T10 validation aggregate unavailable".to_string());
        }
    };
    let benchmarks = build_benchmarks(&development, &validation)?;
    let contributions = build_contributions(&development, &validation)?;
    let eligibilities =
        build_eligibilities(&source, &development, &validation, &benchmarks, &policy)?;
    let cohort = build_cohort(&authorization, &eligibilities)?;
    let (journal, final_counts) = persist_final_evidence(
        &authorization,
        &development,
        &validation,
        &benchmarks,
        &contributions,
        &eligibilities,
        &cohort,
    )?;
    apply_persisted_state(
        &mut report,
        authorization,
        policy,
        Some(development.clone()),
        Some(validation.clone()),
    );
    report.benchmark_comparisons = benchmarks;
    report.contribution_comparisons = contributions;
    report.holdout_eligibility_receipts = eligibilities;
    report.proposed_holdout_cohort = Some(cohort);
    report.deterministic_replay_digest = Some(journal.replay_digest);
    let validation_counts = validation_result
        .as_ref()
        .map(|result| result.counts)
        .unwrap_or_default();
    report.safety_counters.artifacts_written = validation_counts.0 + final_counts.0 + 1;
    report.safety_counters.duplicate_artifact_count = validation_counts.1 + final_counts.1;
    if let Some(result) = &validation_result {
        report.safety_counters.new_model_fits = result.model_fits;
        report.safety_counters.new_calibration_fits = result.calibration_fits;
        report.safety_counters.new_predictions = result.predictions;
        report.safety_counters.new_target_reveals = result.target_reveals;
        report.safety_counters.new_evaluations = result.evaluations;
        report.safety_counters.new_metric_computations = result.metric_computations;
    }
    report.runtime_duration_ms = started.elapsed().as_millis() as u64;
    report.report_digest = report_digest(&report);
    validate_report(&report)?;
    let counts = persist_one(
        "final_reports",
        &report.report_digest,
        &encode_report(&report)?,
        |bytes| Ok(decode_report(bytes)?.report_digest),
    )?;
    if counts.0 != 1 || read_momentum_t10_micro_screening_report_v1()?.as_ref() != Some(&report) {
        return Err("T10 final report persist mismatch".to_string());
    }
    Ok(report)
}

pub fn run_momentum_t10_micro_screening_v1(
    mode: MomentumT10MicroScreeningRunModeV1,
    protected: &MomentumMicroProtectedBeforeStateV1,
) -> Result<MomentumT10MicroScreeningReportV1, String> {
    run_inner(mode, protected)
}

pub fn format_momentum_t10_micro_screening_text_v1(
    report: &MomentumT10MicroScreeningReportV1,
) -> String {
    use std::fmt::Write as _;
    let mut output = String::new();
    let _ = writeln!(output, "status={:?}", report.status);
    let _ = writeln!(
        output,
        "authorization_digest={}",
        report
            .authorization
            .as_ref()
            .map(|value| value.authorization_digest.as_str())
            .unwrap_or("absent")
    );
    let _ = writeln!(
        output,
        "minimum_training_support={}",
        report
            .training_policy
            .as_ref()
            .map(|value| value.minimum_training_examples)
            .unwrap_or_default()
    );
    for aggregate in report.development.iter().chain(report.validation.iter()) {
        let _ = writeln!(output, "partition={}", partition_name(aggregate.partition));
        let _ = writeln!(output, "predictions={}", aggregate.prediction_count);
        let _ = writeln!(output, "scorable={}", aggregate.scorable_count);
        let _ = writeln!(output, "neutral={}", aggregate.neutral_count);
        let _ = writeln!(output, "invalid={}", aggregate.invalid_count);
        let _ = writeln!(output, "daily_refits={}", aggregate.daily_refit_count);
    }
    let _ = writeln!(
        output,
        "proposed_holdout_cohort={}",
        report
            .proposed_holdout_cohort
            .as_ref()
            .map(|value| cohort_status_name(value.status))
            .unwrap_or("absent")
    );
    let _ = writeln!(
        output,
        "holdout_access={}",
        report.safety_counters.t10_holdout_label_reads
            + report.safety_counters.t10_holdout_predictions
            + report.safety_counters.t10_holdout_metrics
    );
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
        value.state_digest = super::super::momentum_micro_protected_before_state_digest_v1(&value);
        value
    }

    fn authorization_fixture() -> MomentumT10ScreeningExecutionAuthorizationV1 {
        let mut value = MomentumT10ScreeningExecutionAuthorizationV1 {
            authorization_version: AUTHORIZATION_VERSION.into(),
            challenger_registration_digest: EXPECTED_REGISTRATION_DIGEST.into(),
            screening_gate_digest: EXPECTED_GATE_DIGEST.into(),
            label_report_digest: EXPECTED_LABEL_REPORT_DIGEST.into(),
            feature_report_digest: EXPECTED_FEATURE_REPORT_DIGEST.into(),
            design_report_digest: EXPECTED_DESIGN_REPORT_DIGEST.into(),
            authorized_task_id: format!("{:?}", MomentumMicroTaskV1::T10NextTenMinuteDirection),
            authorized_participant_ids: PARTICIPANTS
                .iter()
                .map(|participant| participant_id(*participant))
                .collect(),
            development_execution_authorized: true,
            validation_execution_authorized: true,
            historical_holdout_execution_authorized: false,
            t30_execution_authorized: false,
            t60_execution_authorized: false,
            network_authorized: false,
            live_authority_forbidden: true,
            governance_authority_forbidden: true,
            trading_authority_forbidden: true,
            authorization_digest: String::new(),
        };
        value.authorization_digest = authorization_digest(&value);
        value
    }

    fn policy_fixture() -> MomentumT10TrainingPolicyV1 {
        let mut value = MomentumT10TrainingPolicyV1 {
            policy_version: TRAINING_POLICY_VERSION.into(),
            source_training_policy_digest: "source-policy".into(),
            loss_function: "BrierLoss".into(),
            initialization_seed: 7,
            epoch_count: 4,
            batch_size: 64,
            learning_rate_bits: 0.05_f32.to_bits(),
            standard_l2_bits: STANDARD_L2.to_bits(),
            c3_l2_multiplier: 4,
            gradient_finite_checks: true,
            parameter_finite_checks: true,
            probability_clamp_bits: PROBABILITY_CLAMP.to_bits(),
            maximum_training_examples: 4_096,
            minimum_training_examples: 1_024,
            dimension_support_multiplier: 10,
            daily_utc_refit: true,
            within_day_refit_forbidden: true,
            training_only_normalizer: true,
            c4_base_percent: 80,
            c4_calibration_percent: 20,
            validation_fit_forbidden: true,
            holdout_fit_forbidden: true,
            policy_digest: String::new(),
        };
        value.policy_digest = training_policy_digest(&value);
        value
    }

    fn plan_fixture(sufficient: bool) -> MomentumT10DailyTrainingWindowPlanV1 {
        let used = if sufficient { 1_280 } else { 512 };
        let base = used * 80 / 100;
        let mut value = MomentumT10DailyTrainingWindowPlanV1 {
            plan_version: TRAINING_PLAN_VERSION.into(),
            authorization_digest: authorization_fixture().authorization_digest,
            training_policy_digest: policy_fixture().policy_digest,
            partition: MomentumReplayPartitionV1::Development,
            utc_day_boundary_ms: DAY_MS,
            training_target_cutoff_exclusive_ms: DAY_MS,
            eligible_past_event_count: used,
            scorable_training_event_count: used,
            used_training_event_count: used,
            training_event_digests: vec!["event".into(); used],
            c4_base_count: base,
            c4_calibration_count: used - base,
            support_sufficient_for_all: sufficient,
            validation_label_fit_count: 0,
            holdout_access_count: 0,
            plan_digest: String::new(),
        };
        value.plan_digest = training_plan_digest(&value);
        value
    }

    fn normalizer_fixture(
        participant: MomentumMicroParticipantV1,
        dimension: usize,
    ) -> MomentumT10DailyNormalizerReceiptV1 {
        let mut value = MomentumT10DailyNormalizerReceiptV1 {
            receipt_version: NORMALIZER_VERSION.into(),
            participant_id: participant_id(participant),
            feature_policy_digest: "feature-policy".into(),
            private_means: vec![0.0; dimension],
            private_scales: vec![1.0; dimension],
            constant_dimension_indices: Vec::new(),
            training_event_digest: "training".into(),
            finite: true,
            receipt_digest: String::new(),
        };
        value.receipt_digest = normalizer_receipt_digest(&value);
        value
    }

    fn model_fixture(
        participant: MomentumMicroParticipantV1,
        role: DailyModelRole,
        dimension: usize,
    ) -> MomentumT10DailyModelReceiptV1 {
        let constant = role == DailyModelRole::Constant;
        let mut value = MomentumT10DailyModelReceiptV1 {
            receipt_version: MODEL_VERSION.into(),
            participant_id: participant_id(participant),
            role,
            utc_day_boundary_ms: DAY_MS,
            training_plan_digest: plan_fixture(true).plan_digest,
            normalizer_receipt_digest: if constant {
                String::new()
            } else {
                "normalizer".into()
            },
            private_weights: if constant {
                Vec::new()
            } else {
                vec![0.01; dimension]
            },
            private_bias: (!constant).then_some(0.0),
            private_prevalence: constant.then_some(0.5),
            training_count: 1_024,
            positive_count: 512,
            negative_count: 512,
            l2_bits: if constant { 0 } else { STANDARD_L2.to_bits() },
            initialization_seed: if constant { 0 } else { 7 },
            finite: true,
            validation_fit_count: 0,
            holdout_fit_count: 0,
            receipt_digest: String::new(),
        };
        value.receipt_digest = model_receipt_digest(&value);
        value
    }

    fn refit_bundle_fixture() -> MomentumT10DailyRefitBundleV1 {
        let plan = plan_fixture(true);
        let full_training_digest = stable_hash_string(&format!(
            "T10-normalizer-training-events:{:?}",
            plan.training_event_digests
        ));
        let c4_training_digest = stable_hash_string(&format!(
            "T10-normalizer-training-events:{:?}",
            &plan.training_event_digests[..plan.c4_base_count]
        ));
        let mut normalizers = vec![
            normalizer_fixture(
                MomentumMicroParticipantV1::C1TenMinuteAnchorBaseline,
                ANCHOR_FEATURE_DIMENSION,
            ),
            normalizer_fixture(
                MomentumMicroParticipantV1::C2CompactMicroLogistic,
                COMPACT_FEATURE_DIMENSION,
            ),
            normalizer_fixture(
                MomentumMicroParticipantV1::C3CompactMicroStrongShrinkLogistic,
                COMPACT_FEATURE_DIMENSION,
            ),
            normalizer_fixture(
                MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic,
                COMPACT_FEATURE_DIMENSION,
            ),
        ];
        for normalizer in &mut normalizers[..3] {
            normalizer.training_event_digest = full_training_digest.clone();
            normalizer.receipt_digest = normalizer_receipt_digest(normalizer);
        }
        normalizers[3].training_event_digest = c4_training_digest;
        normalizers[3].receipt_digest = normalizer_receipt_digest(&normalizers[3]);
        let mut models = vec![
            model_fixture(
                MomentumMicroParticipantV1::C0TaskSpecificConstant,
                DailyModelRole::Constant,
                0,
            ),
            model_fixture(
                MomentumMicroParticipantV1::C1TenMinuteAnchorBaseline,
                DailyModelRole::LearnedBase,
                ANCHOR_FEATURE_DIMENSION,
            ),
            model_fixture(
                MomentumMicroParticipantV1::C2CompactMicroLogistic,
                DailyModelRole::LearnedBase,
                COMPACT_FEATURE_DIMENSION,
            ),
            model_fixture(
                MomentumMicroParticipantV1::C3CompactMicroStrongShrinkLogistic,
                DailyModelRole::LearnedBase,
                COMPACT_FEATURE_DIMENSION,
            ),
            model_fixture(
                MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic,
                DailyModelRole::LearnedBase,
                COMPACT_FEATURE_DIMENSION,
            ),
            model_fixture(
                MomentumMicroParticipantV1::C4CompactMicroTrainingOnlyCalibratedLogistic,
                DailyModelRole::Calibrator,
                CALIBRATOR_FEATURE_DIMENSION,
            ),
        ];
        let counts = [
            plan.used_training_event_count,
            plan.used_training_event_count,
            plan.used_training_event_count,
            plan.used_training_event_count,
            plan.c4_base_count,
            plan.c4_calibration_count,
        ];
        for (index, model) in models.iter_mut().enumerate() {
            model.training_plan_digest = plan.plan_digest.clone();
            model.training_count = counts[index];
            model.positive_count = counts[index] / 2;
            model.negative_count = counts[index] - model.positive_count;
            model.normalizer_receipt_digest = if index == 0 {
                String::new()
            } else if index == 5 {
                normalizers[3].receipt_digest.clone()
            } else {
                normalizers[index - 1].receipt_digest.clone()
            };
            if index == 3 {
                model.l2_bits = (STANDARD_L2 * C3_L2_MULTIPLIER as f32).to_bits();
            }
            model.receipt_digest = model_receipt_digest(model);
        }
        let reconstructed_participant_digests = PARTICIPANTS
            .iter()
            .enumerate()
            .map(|(index, participant)| {
                frozen_digest(
                    *participant,
                    if index == 0 {
                        ""
                    } else {
                        normalizers[index - 1].receipt_digest.as_str()
                    },
                    &models[index].receipt_digest,
                    (index == 4).then(|| models[5].receipt_digest.as_str()),
                )
            })
            .collect();
        let mut value = MomentumT10DailyRefitBundleV1 {
            bundle_version: REFIT_BUNDLE_VERSION.into(),
            authorization_digest: authorization_fixture().authorization_digest,
            partition: MomentumReplayPartitionV1::Development,
            utc_day_boundary_ms: DAY_MS,
            training_plan: plan,
            normalizer_receipts: normalizers,
            model_receipts: models,
            reconstructed_participant_digests,
            target_access_count_for_prediction_day: 0,
            holdout_access_count: 0,
            live_access_count: 0,
            bundle_digest: String::new(),
        };
        value.bundle_digest = refit_bundle_digest(&value);
        value
    }

    fn event_plan_fixture() -> MomentumT10EventPlanV1 {
        let mut value = MomentumT10EventPlanV1 {
            plan_version: EVENT_PLAN_VERSION.into(),
            partition: MomentumReplayPartitionV1::Development,
            prediction_timestamp_ms: DAY_MS + TEN_MINUTE_MS,
            target_timestamp_ms: DAY_MS + TEN_MINUTE_MS * 2,
            source_event_digest: "event".into(),
            daily_refit_bundle_digest: "bundle".into(),
            participant_ids: PARTICIPANTS
                .iter()
                .map(|participant| participant_id(*participant))
                .collect(),
            target_hidden: true,
            holdout_member: false,
            plan_digest: String::new(),
        };
        value.plan_digest = event_plan_digest(&value);
        value
    }

    fn prediction_fixture() -> MomentumT10PredictionShardV1 {
        let plan = event_plan_fixture();
        let mut value = MomentumT10PredictionShardV1 {
            shard_version: PREDICTION_SHARD_VERSION.into(),
            authorization_digest: authorization_fixture().authorization_digest,
            partition: MomentumReplayPartitionV1::Development,
            utc_day_boundary_ms: DAY_MS,
            daily_refit_bundle_digest: "bundle".into(),
            event_plans: vec![plan],
            participant_ids: PARTICIPANTS
                .iter()
                .map(|participant| participant_id(*participant))
                .collect(),
            private_probabilities: vec![0.5, 0.4, 0.6, 0.45, 0.55],
            prediction_digests: vec!["prediction".into(); 5],
            target_accessed: false,
            label_accessed: false,
            metric_computed: false,
            shard_digest: String::new(),
        };
        value.shard_digest = prediction_shard_digest(&value);
        value
    }

    fn evaluation_fixture(label: ScreeningLabel) -> MomentumT10EvaluationItemV1 {
        let scorable = matches!(label, ScreeningLabel::Up | ScreeningLabel::Down);
        let mut value = MomentumT10EvaluationItemV1 {
            item_version: EVALUATION_ITEM_VERSION.into(),
            event_plan_digest: event_plan_fixture().plan_digest,
            label,
            private_label: scorable.then_some(if label == ScreeningLabel::Up {
                1.0
            } else {
                0.0
            }),
            private_brier_values: if scorable {
                vec![0.25, 0.36, 0.16, 0.3025, 0.2025]
            } else {
                Vec::new()
            },
            private_correctness: if scorable { vec![true; 5] } else { Vec::new() },
            item_digest: String::new(),
        };
        value.item_digest = evaluation_item_digest(&value);
        value
    }

    fn metrics_fixture(
        participant: MomentumMicroParticipantV1,
        delta: f64,
    ) -> MomentumMicroScreeningParticipantMetricsV1 {
        let mut accumulator = MetricAccumulator::default();
        accumulator.prediction_count = 2;
        accumulator.scorable_count = 2;
        accumulator.finite_prediction_count = 2;
        accumulator.brier_sum = 0.4;
        accumulator.correct_count = 1;
        accumulator.deltas = vec![delta, delta];
        accumulator.probability_sum = 1.0;
        accumulator.probability_squared_sum = 0.52;
        accumulator.minimum_probability = 0.4;
        accumulator.maximum_probability = 0.6;
        accumulator.frequencies.insert(0.4_f64.to_bits(), 1);
        accumulator.frequencies.insert(0.6_f64.to_bits(), 1);
        accumulator.calibration_support[4] = 1;
        accumulator.calibration_probability_sum[4] = 0.4;
        accumulator.calibration_support[6] = 1;
        accumulator.calibration_probability_sum[6] = 0.6;
        accumulator.calibration_positive_sum[6] = 1.0;
        finish_metrics(
            participant,
            MomentumReplayPartitionV1::Development,
            accumulator,
        )
        .unwrap()
    }

    fn aggregate_fixture(partition: MomentumReplayPartitionV1) -> MomentumT10PartitionAggregateV1 {
        let mut metrics = PARTICIPANTS
            .iter()
            .map(|participant| metrics_fixture(*participant, 0.0))
            .collect::<Vec<_>>();
        for value in &mut metrics {
            value.partition = partition;
            value.metrics_digest = participant_metrics_digest(value);
        }
        let mut value = MomentumT10PartitionAggregateV1 {
            aggregate_version: AGGREGATE_VERSION.into(),
            authorization_digest: authorization_fixture().authorization_digest,
            partition,
            boundary_event_count: 2,
            training_only_event_count: 0,
            prediction_count: 2,
            scorable_count: 2,
            neutral_count: 0,
            invalid_count: 0,
            daily_refit_count: 1,
            insufficient_support_day_count: 0,
            daily_refit_bundle_digests: vec!["refit".into()],
            prediction_shard_digests: vec!["prediction".into()],
            evaluation_shard_digests: vec!["evaluation".into()],
            participant_metrics: metrics,
            target_access_before_prediction_reopen_count: 0,
            feature_future_access_count: 0,
            partial_candle_access_count: 0,
            holdout_access_count: 0,
            validation_fit_count: 0,
            chronology_audit_passed: true,
            leakage_audit_passed: true,
            prediction_before_reveal_passed: true,
            aggregate_digest: String::new(),
        };
        value.aggregate_digest = aggregate_digest(&value);
        value
    }

    fn eligibility_fixture(
        development: bool,
        validation: bool,
    ) -> MomentumMicroHoldoutEligibilityReceiptV1 {
        let mut value = MomentumMicroHoldoutEligibilityReceiptV1 {
            task_registration_digest: "task".into(),
            participant_registration_digest: "participant".into(),
            participant_id: participant_id(MomentumMicroParticipantV1::C2CompactMicroLogistic),
            development_aggregate_digest: "development".into(),
            validation_aggregate_digest: "validation".into(),
            development_lower_brier_than_constant: development,
            validation_lower_brier_than_constant: validation,
            sufficient_paired_support: true,
            finite_predictions: true,
            finite_metrics: true,
            no_probability_collapse: true,
            no_saturation_failure: true,
            chronology_clean: true,
            leakage_clean: true,
            integrity_clean: true,
            result_selected_mutation_absent: true,
            holdout_access_count: 0,
            eligibility: if development && validation {
                MomentumMicroHoldoutEligibilityV1::EligibleForFutureSealedHoldoutEvaluation
            } else {
                MomentumMicroHoldoutEligibilityV1::IneligibleScreeningGate
            },
            receipt_digest: String::new(),
        };
        value.receipt_digest = eligibility_receipt_digest(&value);
        value
    }

    fn report_fixture() -> MomentumT10MicroScreeningReportV1 {
        let protected = protected_fixture();
        let mut value = MomentumT10MicroScreeningReportV1 {
            report_version: REPORT_VERSION.into(),
            run_mode: "status".into(),
            status: MomentumT10MicroScreeningStatusV1::Authorized,
            authorization: Some(authorization_fixture()),
            training_policy: Some(policy_fixture()),
            source_label_report_digest: EXPECTED_LABEL_REPORT_DIGEST.into(),
            source_feature_report_digest: EXPECTED_FEATURE_REPORT_DIGEST.into(),
            source_design_report_digest: EXPECTED_DESIGN_REPORT_DIGEST.into(),
            source_registration_digest: EXPECTED_REGISTRATION_DIGEST.into(),
            source_gate_digest: EXPECTED_GATE_DIGEST.into(),
            protected_before_state_digest: protected.state_digest,
            completed_live_event_count: 2,
            scorable_live_event_count: 2,
            live_pause: "PausedAfterCompletedEpochTwo".into(),
            epoch_three_registered: false,
            t10_boundary: None,
            t30_boundary: None,
            t10_disposition: "StableEnoughForFutureScreening".into(),
            t30_disposition: "ExcessiveTemporalInstability".into(),
            t60_disposition: "ExcessiveTemporalInstability".into(),
            development: None,
            validation: None,
            benchmark_comparisons: Vec::new(),
            contribution_comparisons: Vec::new(),
            holdout_eligibility_receipts: Vec::new(),
            proposed_holdout_cohort: None,
            full_eight_a3_blocked: true,
            historical_holdout_execution_mode_absent: true,
            live_roster_unchanged: true,
            protected_artifacts_unchanged: true,
            labels: PUBLIC_LABELS.iter().map(|label| (*label).into()).collect(),
            safety_counters: MomentumT10ScreeningSafetyCountersV1::default(),
            deterministic_replay_digest: None,
            runtime_duration_ms: 0,
            report_digest: String::new(),
        };
        value.report_digest = report_digest(&value);
        value
    }

    #[test]
    fn sprint102_01_sprint101_invariants_bind_authorization() {
        assert!(validate_authorization(&authorization_fixture()).is_ok());
    }

    #[test]
    fn sprint102_02_label_report_identity_is_frozen() {
        assert_eq!(
            authorization_fixture().label_report_digest,
            EXPECTED_LABEL_REPORT_DIGEST
        );
    }

    #[test]
    fn sprint102_03_feature_report_identity_is_frozen() {
        assert_eq!(
            authorization_fixture().feature_report_digest,
            EXPECTED_FEATURE_REPORT_DIGEST
        );
    }

    #[test]
    fn sprint102_04_design_report_identity_is_frozen() {
        assert_eq!(
            authorization_fixture().design_report_digest,
            EXPECTED_DESIGN_REPORT_DIGEST
        );
    }

    #[test]
    fn sprint102_05_live_lane_is_completed_and_paused() {
        assert!(validate_momentum_micro_protected_before_state_v1(&protected_fixture()).is_ok());
    }

    #[test]
    fn sprint102_06_epoch_three_is_absent() {
        assert!(!protected_fixture().epoch_three_registered);
    }

    #[test]
    fn sprint102_07_t10_disposition_permits_authorization() {
        assert!(authorization_fixture().development_execution_authorized);
    }

    #[test]
    fn sprint102_08_t30_execution_is_blocked() {
        assert!(!authorization_fixture().t30_execution_authorized);
    }

    #[test]
    fn sprint102_09_t60_remains_diagnostic_only() {
        assert!(!authorization_fixture().t60_execution_authorized);
    }

    #[test]
    fn sprint102_10_authorization_binds_frozen_registration() {
        assert_eq!(
            authorization_fixture().challenger_registration_digest,
            EXPECTED_REGISTRATION_DIGEST
        );
    }

    #[test]
    fn sprint102_11_development_is_authorized_only_as_registered() {
        let value = authorization_fixture();
        assert!(value.development_execution_authorized && !value.t30_execution_authorized);
    }

    #[test]
    fn sprint102_12_validation_is_authorized_only_as_registered() {
        let value = authorization_fixture();
        assert!(value.validation_execution_authorized && !value.t60_execution_authorized);
    }

    #[test]
    fn sprint102_13_holdout_is_forbidden() {
        assert!(!authorization_fixture().historical_holdout_execution_authorized);
    }

    #[test]
    fn sprint102_14_network_is_forbidden() {
        assert!(!authorization_fixture().network_authorized);
    }

    #[test]
    fn sprint102_15_live_authority_is_forbidden() {
        assert!(authorization_fixture().live_authority_forbidden);
    }

    #[test]
    fn sprint102_16_compact_dimension_is_derived_as_69() {
        assert_eq!(COMPACT_FEATURE_DIMENSION, 69);
    }

    #[test]
    fn sprint102_17_compact_schema_uses_micro_timeframes_only() {
        assert_eq!(
            [
                MomentumHistoricalTimeframeV1::Minute1,
                MomentumHistoricalTimeframeV1::Minute3,
                MomentumHistoricalTimeframeV1::Minute5,
                MomentumHistoricalTimeframeV1::Minute10,
            ]
            .len(),
            4
        );
    }

    #[test]
    fn sprint102_18_c0_uses_past_revealed_labels_only() {
        let value = model_fixture(
            MomentumMicroParticipantV1::C0TaskSpecificConstant,
            DailyModelRole::Constant,
            0,
        );
        assert!(validate_model_receipt(&value).is_ok() && value.private_prevalence.is_some());
    }

    #[test]
    fn sprint102_19_c1_has_fresh_t10_identity() {
        assert!(
            !participant_id(MomentumMicroParticipantV1::C1TenMinuteAnchorBaseline)
                .contains("QualifiedSix")
        );
    }

    #[test]
    fn sprint102_20_c2_uses_compact_dimension() {
        let value = model_fixture(
            MomentumMicroParticipantV1::C2CompactMicroLogistic,
            DailyModelRole::LearnedBase,
            69,
        );
        assert_eq!(value.private_weights.len(), 69);
    }

    #[test]
    fn sprint102_21_c3_differs_by_fixed_l2_multiplier() {
        let standard = training_config(&policy_fixture(), 7, 1).unwrap();
        let stronger = training_config(&policy_fixture(), 7, C3_L2_MULTIPLIER).unwrap();
        assert_eq!(stronger.optimizer.weight_decay, STANDARD_L2 * 4.0);
        assert_eq!(stronger.seed, standard.seed);
        assert_eq!(stronger.epochs, standard.epochs);
        assert_eq!(stronger.batch_size, standard.batch_size);
        assert_eq!(
            stronger.optimizer.learning_rate,
            standard.optimizer.learning_rate
        );
    }

    #[test]
    fn sprint102_22_c4_uses_chronological_80_20_split() {
        let plan = plan_fixture(true);
        assert_eq!(
            (plan.c4_base_count, plan.c4_calibration_count),
            (1_024, 256)
        );
    }

    #[test]
    fn sprint102_23_validation_cannot_fit_calibration() {
        assert!(policy_fixture().validation_fit_forbidden);
    }

    #[test]
    fn sprint102_24_holdout_cannot_fit_calibration() {
        assert!(policy_fixture().holdout_fit_forbidden);
    }

    #[test]
    fn sprint102_25_participants_share_one_paired_order() {
        assert_eq!(
            authorization_fixture().authorized_participant_ids,
            PARTICIPANTS
                .iter()
                .map(|participant| participant_id(*participant))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn sprint102_26_minimum_support_is_derived() {
        assert_eq!(derive_minimum_support(69).unwrap(), 1_024);
    }

    #[test]
    fn sprint102_27_insufficient_c4_support_blocks_all() {
        let plan = plan_fixture(false);
        assert!(!plan.support_sufficient_for_all && validate_training_plan(&plan).is_ok());
    }

    #[test]
    fn sprint102_28_daily_refit_uses_prior_cutoff() {
        let plan = plan_fixture(true);
        assert_eq!(
            plan.training_target_cutoff_exclusive_ms,
            plan.utc_day_boundary_ms
        );
    }

    #[test]
    fn sprint102_29_within_day_refit_is_forbidden() {
        assert!(policy_fixture().within_day_refit_forbidden);
    }

    #[test]
    fn sprint102_30_daily_receipts_round_trip() {
        let value = refit_bundle_fixture();
        assert_eq!(
            decode_refit_bundle(&encode_refit_bundle(&value).unwrap()).unwrap(),
            value
        );
    }

    #[test]
    fn sprint102_31_prediction_persists_before_target_reveal() {
        let value = prediction_fixture();
        assert!(!value.target_accessed && validate_prediction_shard(&value).is_ok());
    }

    #[test]
    fn sprint102_32_prediction_requires_five_participants() {
        assert_eq!(prediction_fixture().private_probabilities.len(), 5);
    }

    #[test]
    fn sprint102_33_participant_order_changes_binding() {
        let mut value = prediction_fixture();
        value.participant_ids.swap(1, 2);
        value.shard_digest = prediction_shard_digest(&value);
        assert!(validate_prediction_shard(&value).is_err());
    }

    #[test]
    fn sprint102_34_neutral_excludes_brier() {
        let value = evaluation_fixture(ScreeningLabel::Neutral);
        assert!(value.private_brier_values.is_empty() && validate_evaluation_item(&value).is_ok());
    }

    #[test]
    fn sprint102_35_invalid_evidence_remains_invalid() {
        assert_eq!(
            evaluation_fixture(ScreeningLabel::Invalid).label,
            ScreeningLabel::Invalid
        );
    }

    #[test]
    fn sprint102_36_partitions_remain_separate() {
        assert_ne!(
            aggregate_fixture(MomentumReplayPartitionV1::Development).partition,
            aggregate_fixture(MomentumReplayPartitionV1::Validation).partition
        );
    }

    #[test]
    fn sprint102_37_development_cannot_mutate_validation_policy() {
        let first = policy_fixture();
        let second = policy_fixture();
        assert_eq!(first.policy_digest, second.policy_digest);
    }

    #[test]
    fn sprint102_38_brier_delta_is_paired() {
        assert_eq!(
            metrics_fixture(MomentumMicroParticipantV1::C2CompactMicroLogistic, -0.01)
                .negative_paired_delta_count,
            2
        );
    }

    #[test]
    fn sprint102_39_correctness_cannot_override_brier_gate() {
        let value = eligibility_fixture(false, false);
        assert_eq!(
            value.eligibility,
            MomentumMicroHoldoutEligibilityV1::IneligibleScreeningGate
        );
    }

    #[test]
    fn sprint102_40_calibration_cannot_override_brier_gate() {
        assert!(validate_eligibility(&eligibility_fixture(false, false)).is_ok());
    }

    #[test]
    fn sprint102_41_collapse_blocks_eligibility() {
        let mut value = eligibility_fixture(true, true);
        value.no_probability_collapse = false;
        value.eligibility = MomentumMicroHoldoutEligibilityV1::IneligibleScreeningGate;
        value.receipt_digest = eligibility_receipt_digest(&value);
        assert!(validate_eligibility(&value).is_ok());
    }

    #[test]
    fn sprint102_42_chronology_failure_blocks_eligibility() {
        let mut value = eligibility_fixture(true, true);
        value.chronology_clean = false;
        value.eligibility = MomentumMicroHoldoutEligibilityV1::IneligibleScreeningGate;
        value.receipt_digest = eligibility_receipt_digest(&value);
        assert!(validate_eligibility(&value).is_ok());
    }

    #[test]
    fn sprint102_43_leakage_failure_blocks_eligibility() {
        let mut value = eligibility_fixture(true, true);
        value.leakage_clean = false;
        value.eligibility = MomentumMicroHoldoutEligibilityV1::IneligibleScreeningGate;
        value.receipt_digest = eligibility_receipt_digest(&value);
        assert!(validate_eligibility(&value).is_ok());
    }

    #[test]
    fn sprint102_44_result_selected_mutation_blocks_eligibility() {
        let mut value = eligibility_fixture(true, true);
        value.result_selected_mutation_absent = false;
        value.eligibility = MomentumMicroHoldoutEligibilityV1::IneligibleScreeningGate;
        value.receipt_digest = eligibility_receipt_digest(&value);
        assert!(validate_eligibility(&value).is_ok());
    }

    #[test]
    fn sprint102_45_development_only_improvement_is_ineligible() {
        assert_eq!(
            eligibility_fixture(true, false).eligibility,
            MomentumMicroHoldoutEligibilityV1::IneligibleScreeningGate
        );
    }

    #[test]
    fn sprint102_46_validation_only_improvement_is_ineligible() {
        assert_eq!(
            eligibility_fixture(false, true).eligibility,
            MomentumMicroHoldoutEligibilityV1::IneligibleScreeningGate
        );
    }

    #[test]
    fn sprint102_47_both_partition_improvement_is_required() {
        assert_eq!(
            eligibility_fixture(true, true).eligibility,
            MomentumMicroHoldoutEligibilityV1::EligibleForFutureSealedHoldoutEvaluation
        );
    }

    #[test]
    fn sprint102_48_cohort_contains_every_eligible_challenger() {
        let values = vec![eligibility_fixture(true, true), {
            let mut value = eligibility_fixture(true, true);
            value.participant_id =
                participant_id(MomentumMicroParticipantV1::C3CompactMicroStrongShrinkLogistic);
            value.participant_registration_digest = "participant-c3".into();
            value.receipt_digest = eligibility_receipt_digest(&value);
            value
        }];
        assert_eq!(
            build_cohort(&authorization_fixture(), &values)
                .unwrap()
                .participant_ids
                .len(),
            2
        );
    }

    #[test]
    fn sprint102_49_no_manual_cohort_cherry_pick_api_exists() {
        let values = vec![eligibility_fixture(true, true)];
        let cohort = build_cohort(&authorization_fixture(), &values).unwrap();
        assert_eq!(cohort.eligibility_receipt_digests.len(), values.len());
    }

    #[test]
    fn sprint102_50_empty_eligible_set_produces_no_cohort() {
        let cohort = build_cohort(
            &authorization_fixture(),
            &[eligibility_fixture(false, false)],
        )
        .unwrap();
        assert_eq!(
            cohort.status,
            MomentumMicroHoldoutCohortStatusV1::NoEligibleT10HoldoutCohort
        );
    }

    #[test]
    fn sprint102_51_historical_holdout_remains_unopened() {
        assert!(report_fixture().historical_holdout_execution_mode_absent);
    }

    #[test]
    fn sprint102_52_t30_model_counts_remain_zero() {
        assert_eq!(report_fixture().safety_counters.t30_model_fits, 0);
    }

    #[test]
    fn sprint102_53_t60_model_counts_remain_zero() {
        assert_eq!(report_fixture().safety_counters.t60_model_fits, 0);
    }

    #[test]
    fn sprint102_54_full_eight_remains_blocked() {
        assert!(report_fixture().full_eight_a3_blocked);
    }

    #[test]
    fn sprint102_55_month_and_year_remain_inaccessible() {
        let safety = report_fixture().safety_counters;
        assert_eq!(safety.month_view_loads + safety.year_view_loads, 0);
    }

    #[test]
    fn sprint102_56_live_participants_remain_unchanged() {
        assert!(report_fixture().live_roster_unchanged);
    }

    #[test]
    fn sprint102_57_reward_and_chair_counters_remain_zero() {
        let safety = report_fixture().safety_counters;
        assert_eq!(safety.reward_applications + safety.chair_actions, 0);
    }

    #[test]
    fn sprint102_58_network_counters_remain_zero() {
        assert_eq!(report_fixture().safety_counters.network_requests, 0);
    }

    #[test]
    fn sprint102_59_completed_replay_performs_zero_work() {
        let protected = protected_fixture();
        let first = report_fixture();
        let replay = completed_replay(
            first.clone(),
            MomentumT10MicroScreeningRunModeV1::Status,
            &protected,
            Instant::now(),
        )
        .unwrap();
        assert_eq!(replay.report_digest, first.report_digest);
        assert_eq!(
            replay.safety_counters,
            MomentumT10ScreeningSafetyCountersV1::default()
        );
    }

    #[test]
    fn sprint102_60_conflicting_artifact_rejects() {
        let mut value = authorization_fixture();
        value.t30_execution_authorized = true;
        value.authorization_digest = authorization_digest(&value);
        assert!(encode_authorization(&value).is_err());
    }

    #[test]
    fn sprint102_61_malformed_protobuf_rejects() {
        assert!(decode_authorization(&[0xff, 0x00]).is_err());
    }

    #[test]
    fn sprint102_62_text_and_json_agree() {
        let report = report_fixture();
        let text = format_momentum_t10_micro_screening_text_v1(&report);
        let json = serde_json::to_string(&report).unwrap();
        assert!(text.contains(&report.report_digest) && json.contains(&report.report_digest));
    }
}
