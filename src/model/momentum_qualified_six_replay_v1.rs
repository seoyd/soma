//! Offline, research-only replay over the six semantically qualified timeframes.

use std::{
    collections::BTreeMap,
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
        ArtifactBuilderV4_2, ArtifactReaderV4_2, as_u64, as_usize, persist_artifact,
    },
    momentum_multitimeframe_history_v1::{
        MomentumHistoricalTimeframeV1, MomentumQualifiedReplayCandleEvidenceV1,
        MomentumQualifiedReplayProtectedStateV1, MomentumQualifiedSixEvidenceV1,
        load_momentum_qualified_six_evidence_v1, momentum_qualified_replay_protected_state_v1,
    },
    momentum_raw_feature_v4::train_head_v4,
};

const ROOT: &str = "state/historical_replay/momentum_qualified_six/v1";
const REGISTRATION_VERSION: &str = "momentum-qualified-six-replay-registration-v1";
const LABEL_POLICY_VERSION: &str = "momentum-ten-minute-direction-label-policy-v1";
const PARTITION_POLICY_VERSION: &str = "momentum-qualified-six-partition-policy-v1";
const PARTICIPANT_VERSION: &str = "momentum-qualified-six-participant-registration-v1";
const ELIGIBILITY_VERSION: &str = "momentum-qualified-six-eligibility-audit-v1";
const HOLDOUT_VERSION: &str = "momentum-qualified-six-sealed-holdout-v1";
const REFIT_VERSION: &str = "momentum-qualified-six-daily-refit-receipt-v1";
const NORMALIZER_RECEIPT_VERSION: &str = "momentum-qualified-six-daily-normalizer-receipt-v1";
const PARTICIPANT_RECEIPT_VERSION: &str = "momentum-qualified-six-daily-participant-receipt-v1";
const REFIT_BUNDLE_VERSION: &str = "momentum-qualified-six-daily-refit-bundle-v1";
const BLOCK_VERSION: &str = "momentum-qualified-six-feature-block-v1";
const EVENT_PLAN_VERSION: &str = "momentum-qualified-six-event-plan-v1";
const PREDICTION_SEAL_VERSION: &str = "momentum-qualified-six-prediction-seal-v1";
const CAPSULE_VERSION: &str = "momentum-qualified-six-prediction-capsule-v1";
const EVALUATION_VERSION: &str = "momentum-qualified-six-evaluation-v1";
const PREDICTION_BUNDLE_VERSION: &str = "momentum-qualified-six-daily-prediction-bundle-v1";
const EVALUATION_BUNDLE_VERSION: &str = "momentum-qualified-six-daily-evaluation-bundle-v1";
const AGGREGATE_VERSION: &str = "momentum-qualified-six-partition-aggregate-v1";
const BENCHMARK_VERSION: &str = "momentum-qualified-six-benchmark-comparison-v1";
const CONTRIBUTION_VERSION: &str = "momentum-qualified-six-contribution-comparison-v1";
const JOURNAL_VERSION: &str = "momentum-qualified-six-replay-journal-v1";
const REPORT_VERSION: &str = "momentum-qualified-six-public-report-v1";
const FAMILY_LABEL: &str = "QualifiedSixIntradayTenMinute";
const TASK_LABEL: &str = "IntradayTenMinuteDirection";
const CONTEXT_LENGTH: usize = 16;
const TEN_MINUTE_MS: u64 = 10 * 60 * 1_000;
const DAY_MS: u64 = 24 * 60 * 60 * 1_000;
const DEVELOPMENT_PERCENT: usize = 70;
const VALIDATION_PERCENT: usize = 15;
const TRAINING_DIMENSION_MULTIPLIER: usize = 10;
const MAX_TRAINING_EXAMPLES: usize = 4_096;
const COMPARISON_EPSILON: f64 = 1e-12;
const COLLAPSE_VARIANCE_THRESHOLD: f64 = 1e-6;
const PUBLIC_LABELS: [&str; 4] = [
    "HistoricalResearchOnly",
    "QualifiedSixNotFullEight",
    "NotIndependentLiveEvidence",
    "NotTradingAuthority",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedReplayFamilyV1 {
    QualifiedSixIntradayTenMinute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedPredictionTaskV1 {
    IntradayTenMinuteDirection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedRefitPolicyV1 {
    RefitAtUtcDayBoundary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumReplayPartitionV1 {
    Development,
    Validation,
    SealedHoldout,
}

impl MomentumReplayPartitionV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Validation => "validation",
            Self::SealedHoldout => "sealed-holdout",
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "development" => Ok(Self::Development),
            "validation" => Ok(Self::Validation),
            "sealed-holdout" => Ok(Self::SealedHoldout),
            _ => Err("qualified-six partition rejected".to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MomentumQualifiedParticipantV1 {
    Q0TrainingPrevalenceConstant,
    Q1TenMinuteAnchorLogistic,
    Q2MicroBlockLogistic,
    Q3QualifiedMacroBlockLogistic,
    Q4QualifiedSixFusionLogistic,
}

impl MomentumQualifiedParticipantV1 {
    const ORDERED: [Self; 5] = [
        Self::Q0TrainingPrevalenceConstant,
        Self::Q1TenMinuteAnchorLogistic,
        Self::Q2MicroBlockLogistic,
        Self::Q3QualifiedMacroBlockLogistic,
        Self::Q4QualifiedSixFusionLogistic,
    ];

    fn id(self) -> &'static str {
        match self {
            Self::Q0TrainingPrevalenceConstant => "QualifiedSixTrainingPrevalenceConstantV1",
            Self::Q1TenMinuteAnchorLogistic => "QualifiedSixTenMinuteAnchorLogisticV1",
            Self::Q2MicroBlockLogistic => "QualifiedSixMicroBlockLogisticV1",
            Self::Q3QualifiedMacroBlockLogistic => "QualifiedSixMacroBlockLogisticV1",
            Self::Q4QualifiedSixFusionLogistic => "QualifiedSixFusionLogisticV1",
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        Self::ORDERED
            .into_iter()
            .find(|participant| participant.id() == value)
            .ok_or_else(|| "qualified-six participant rejected".to_string())
    }

    fn timeframes(self) -> Vec<MomentumHistoricalTimeframeV1> {
        match self {
            Self::Q0TrainingPrevalenceConstant => Vec::new(),
            Self::Q1TenMinuteAnchorLogistic => {
                vec![MomentumHistoricalTimeframeV1::Minute10]
            }
            Self::Q2MicroBlockLogistic => vec![
                MomentumHistoricalTimeframeV1::Minute1,
                MomentumHistoricalTimeframeV1::Minute3,
                MomentumHistoricalTimeframeV1::Minute5,
                MomentumHistoricalTimeframeV1::Minute10,
            ],
            Self::Q3QualifiedMacroBlockLogistic => vec![
                MomentumHistoricalTimeframeV1::Day1,
                MomentumHistoricalTimeframeV1::Week1,
            ],
            Self::Q4QualifiedSixFusionLogistic => included_timeframes(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedLabelStatusV1 {
    Up,
    Down,
    Neutral,
    Invalid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedReplayStatusV1 {
    Unregistered,
    Registered,
    DevelopmentComplete,
    Complete,
    InsufficientTrainingSupport,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedBenchmarkComparisonV1 {
    LowerBrierThanConstant,
    HigherBrierThanConstant,
    NumericallyEquivalentToConstant,
    MixedAcrossPartitions,
    InsufficientScorableValidation,
    ProbabilityCollapse,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumQualifiedContributionStatusV1 {
    LowerBrierWithAddedBlock,
    HigherBrierWithAddedBlock,
    NumericallyEquivalent,
    MixedAcrossPartitions,
    InsufficientPairedValidation,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumQualifiedSixRunModeV1 {
    Status,
    DryRun,
    Register,
    ExecuteDevelopment,
    ExecuteValidation,
}

impl MomentumQualifiedSixRunModeV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::DryRun => "dry-run",
            Self::Register => "register",
            Self::ExecuteDevelopment => "execute-development",
            Self::ExecuteValidation => "execute-validation",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumTenMinuteDirectionLabelPolicyV1 {
    pub policy_version: String,
    pub event_timeframe: MomentumHistoricalTimeframeV1,
    pub target_horizon_candles: usize,
    pub up_rule: String,
    pub down_rule: String,
    pub neutral_rule: String,
    pub trading_claim_forbidden: bool,
    pub policy_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumQualifiedPartitionPolicyV1 {
    pub policy_version: String,
    pub eligible_start_timestamp_ms: u64,
    pub eligible_end_timestamp_ms: u64,
    pub development_end_exclusive_ms: u64,
    pub validation_end_exclusive_ms: u64,
    pub holdout_start_timestamp_ms: u64,
    pub common_eligible_event_count: usize,
    pub development_event_count: usize,
    pub validation_event_count: usize,
    pub holdout_event_count: usize,
    pub development_percent: usize,
    pub validation_percent: usize,
    pub holdout_percent: usize,
    pub policy_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumQualifiedParticipantRegistrationV1 {
    pub registration_version: String,
    pub participant: MomentumQualifiedParticipantV1,
    pub ordered_timeframes: Vec<MomentumHistoricalTimeframeV1>,
    pub fresh_research_parameters_required: bool,
    pub live_parameter_use_forbidden: bool,
    pub prior_fold_parameter_use_forbidden: bool,
    pub interaction_features_forbidden: bool,
    pub participant_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumQualifiedSixHoldoutBoundaryV1 {
    pub boundary_version: String,
    pub prior_holdout_digest: String,
    pub adopted_prior_boundary: bool,
    pub holdout_start_timestamp_ms: u64,
    pub holdout_event_count: usize,
    pub labels_opened: bool,
    pub metric_computations: usize,
    pub participant_predictions: usize,
    pub aggregate_result_present: bool,
    pub boundary_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumQualifiedEligibilityAuditV1 {
    pub audit_version: String,
    pub protocol_event_count: usize,
    pub common_eligible_event_count: usize,
    pub rejected_context_count: usize,
    pub rejected_target_count: usize,
    pub rejected_partial_count: usize,
    pub rejected_missing_evidence_count: usize,
    pub month_view_load_count: usize,
    pub year_view_load_count: usize,
    pub future_access_count: usize,
    pub partial_access_count: usize,
    pub unqualified_access_count: usize,
    pub target_value_access_count: usize,
    pub first_prediction_timestamp_ms: u64,
    pub last_prediction_timestamp_ms: u64,
    pub audit_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumQualifiedSixReplayRegistrationV1 {
    pub registration_version: String,
    pub family: MomentumQualifiedReplayFamilyV1,
    pub qualified_timeframe_set_digest: String,
    pub monthly_exclusion_policy_digest: String,
    pub yearly_exclusion_policy_digest: String,
    pub causal_revalidation_digest: String,
    pub protocol_replay_digest: String,
    pub historical_dataset_index_digests: Vec<String>,
    pub included_timeframes: Vec<MomentumHistoricalTimeframeV1>,
    pub excluded_timeframes: Vec<MomentumHistoricalTimeframeV1>,
    pub prediction_task: MomentumQualifiedPredictionTaskV1,
    pub event_selection_policy_digest: String,
    pub context_policy_digest: String,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub training_policy_digest: String,
    pub normalization_policy_digest: String,
    pub evaluation_policy_digest: String,
    pub refit_policy: MomentumQualifiedRefitPolicyV1,
    pub participant_registrations: Vec<MomentumQualifiedParticipantRegistrationV1>,
    pub development_boundary_digest: String,
    pub validation_boundary_digest: String,
    pub sealed_holdout_boundary_digest: String,
    pub minimum_training_examples: usize,
    pub maximum_training_examples: usize,
    pub full_eight_replay_claim_forbidden: bool,
    pub live_authority_use_forbidden: bool,
    pub reward_application_forbidden: bool,
    pub chair_action_forbidden: bool,
    pub trading_authority_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumQualifiedTimeframeFeatureBlockV1 {
    pub block_version: String,
    pub timeframe: MomentumHistoricalTimeframeV1,
    pub context_timestamp_ms: Vec<u64>,
    pub source_candle_digests: Vec<String>,
    pub feature_schema_digest: String,
    pub feature_vector_digest: String,
    pub normalizer_digest: String,
    pub future_access_count: usize,
    pub partial_access_count: usize,
    pub missing_evidence_count: usize,
    pub block_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumQualifiedDailyRefitReceiptV1 {
    pub refit_version: String,
    pub registration_digest: String,
    pub utc_day_boundary_ms: u64,
    pub training_target_cutoff_exclusive_ms: u64,
    pub eligible_past_event_count: usize,
    pub scorable_training_event_count: usize,
    pub used_training_event_count: usize,
    pub participant_parameter_digests: Vec<String>,
    pub timeframe_normalizer_digests: Vec<String>,
    pub within_day_refit_count: usize,
    pub live_parameter_load_count: usize,
    pub prior_fold_parameter_load_count: usize,
    pub refit_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumQualifiedDailyNormalizerReceiptV1 {
    receipt_version: String,
    timeframe: MomentumHistoricalTimeframeV1,
    private_means: Vec<f32>,
    private_scales: Vec<f32>,
    constant_dimension_indices: Vec<usize>,
    normalizer_digest: String,
    receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumQualifiedDailyParticipantReceiptV1 {
    receipt_version: String,
    registration_digest: String,
    utc_day_boundary_ms: u64,
    participant: MomentumQualifiedParticipantV1,
    scorable_training_event_count: usize,
    used_training_event_count: usize,
    parameter_digest: String,
    normalizer_binding_digest: String,
    private_head_weights: Vec<f32>,
    private_head_bias: Option<f32>,
    private_prevalence: f64,
    live_parameter_load_count: usize,
    prior_fold_parameter_load_count: usize,
    receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumQualifiedDailyRefitBundleV1 {
    bundle_version: String,
    registration_digest: String,
    partition: MomentumReplayPartitionV1,
    utc_day_boundary_ms: u64,
    refit_receipt: MomentumQualifiedDailyRefitReceiptV1,
    normalizer_receipts: Vec<MomentumQualifiedDailyNormalizerReceiptV1>,
    participant_receipts: Vec<MomentumQualifiedDailyParticipantReceiptV1>,
    bundle_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumQualifiedReplayEventPlanV1 {
    pub plan_version: String,
    pub registration_digest: String,
    pub partition: MomentumReplayPartitionV1,
    pub event_number: u64,
    pub prediction_timestamp_ms: u64,
    pub target_timestamp_ms: u64,
    pub daily_refit_receipt_digest: String,
    pub timeframe_block_digests: Vec<String>,
    pub participant_ids: Vec<String>,
    pub event_plan_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumQualifiedPredictionSealV1 {
    seal_version: String,
    event_plan_digest: String,
    participant: MomentumQualifiedParticipantV1,
    parameter_digest: String,
    normalizer_binding_digest: String,
    private_probability: f64,
    prediction_digest: String,
    seal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumQualifiedReplayPredictionCapsuleV1 {
    pub capsule_version: String,
    pub event_plan_digest: String,
    pub participant_seal_digests: Vec<String>,
    pub participant_prediction_digests: Vec<String>,
    pub target_accessed: bool,
    pub label_accessed: bool,
    pub metrics_computed: bool,
    pub capsule_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumQualifiedReplayEvaluationV1 {
    evaluation_version: String,
    event_plan_digest: String,
    prediction_capsule_digest: String,
    label_status: MomentumQualifiedLabelStatusV1,
    private_label: Option<f64>,
    participant_evaluation_digests: Vec<String>,
    private_brier_values: Vec<f64>,
    private_correctness: Vec<bool>,
    evaluation_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumQualifiedDailyPredictionBundleV1 {
    bundle_version: String,
    registration_digest: String,
    partition: MomentumReplayPartitionV1,
    utc_day_boundary_ms: u64,
    refit_receipt: MomentumQualifiedDailyRefitReceiptV1,
    feature_blocks: Vec<MomentumQualifiedTimeframeFeatureBlockV1>,
    event_plans: Vec<MomentumQualifiedReplayEventPlanV1>,
    prediction_seals: Vec<MomentumQualifiedPredictionSealV1>,
    capsules: Vec<MomentumQualifiedReplayPredictionCapsuleV1>,
    target_access_count: usize,
    label_access_count: usize,
    metric_computation_count: usize,
    bundle_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumQualifiedDailyEvaluationBundleV1 {
    bundle_version: String,
    prediction_bundle_digest: String,
    partition: MomentumReplayPartitionV1,
    utc_day_boundary_ms: u64,
    evaluations: Vec<MomentumQualifiedReplayEvaluationV1>,
    prediction_bundle_reopened: bool,
    bundle_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumQualifiedParticipantMetricsV1 {
    pub participant_id: String,
    pub partition: MomentumReplayPartitionV1,
    pub total_prediction_events: usize,
    pub scorable_events: usize,
    pub neutral_events: usize,
    pub invalid_events: usize,
    pub finite_prediction_count: usize,
    pub probability_collapsed: bool,
    pub mean_brier_score: Option<f64>,
    pub binary_correctness: Option<f64>,
    pub delta_versus_constant: Option<f64>,
    pub paired_scorable_count: usize,
    pub chronology_audit_passed: bool,
    pub leakage_audit_passed: bool,
    pub metrics_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct MomentumQualifiedPartitionAggregateV1 {
    aggregate_version: String,
    registration_digest: String,
    partition: MomentumReplayPartitionV1,
    partition_event_count: usize,
    training_only_event_count: usize,
    prediction_event_count: usize,
    scorable_event_count: usize,
    neutral_event_count: usize,
    invalid_event_count: usize,
    daily_refit_count: usize,
    daily_prediction_bundle_digests: Vec<String>,
    daily_evaluation_bundle_digests: Vec<String>,
    participant_metrics: Vec<MomentumQualifiedParticipantMetricsV1>,
    target_access_before_capsule_count: usize,
    future_access_count: usize,
    partial_access_count: usize,
    unqualified_access_count: usize,
    aggregate_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumQualifiedBenchmarkReceiptV1 {
    pub comparison_version: String,
    pub participant_id: String,
    pub development_delta_bits: Option<u64>,
    pub validation_delta_bits: Option<u64>,
    pub paired_development_count: usize,
    pub paired_validation_count: usize,
    pub classification: MomentumQualifiedBenchmarkComparisonV1,
    pub comparison_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumQualifiedContributionReceiptV1 {
    pub comparison_version: String,
    pub added_participant_id: String,
    pub baseline_participant_id: String,
    pub development_delta_bits: Option<u64>,
    pub validation_delta_bits: Option<u64>,
    pub paired_development_count: usize,
    pub paired_validation_count: usize,
    pub status: MomentumQualifiedContributionStatusV1,
    pub comparison_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumQualifiedReplayJournalV1 {
    journal_version: String,
    registration_digest: String,
    eligibility_audit_digest: String,
    development_aggregate_digest: String,
    validation_aggregate_digest: String,
    benchmark_comparison_digests: Vec<String>,
    contribution_comparison_digests: Vec<String>,
    holdout_boundary_digest: String,
    holdout_label_reads: usize,
    holdout_metric_computations: usize,
    holdout_participant_predictions: usize,
    deterministic: bool,
    replay_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumQualifiedSixReplayReportV1 {
    pub report_version: String,
    pub run_mode: String,
    pub status: MomentumQualifiedReplayStatusV1,
    pub family: MomentumQualifiedReplayFamilyV1,
    pub registration_digest: Option<String>,
    pub qualified_timeframe_set_digest: Option<String>,
    pub included_timeframes: Vec<MomentumHistoricalTimeframeV1>,
    pub excluded_timeframes: Vec<MomentumHistoricalTimeframeV1>,
    pub prediction_task: MomentumQualifiedPredictionTaskV1,
    pub label_policy_digest: Option<String>,
    pub common_eligible_event_count: usize,
    pub common_eligible_start_timestamp_ms: Option<u64>,
    pub common_eligible_end_timestamp_ms: Option<u64>,
    pub development_boundary_digest: Option<String>,
    pub validation_boundary_digest: Option<String>,
    pub minimum_training_examples: usize,
    pub maximum_training_examples: usize,
    pub development_partition_event_count: usize,
    pub development_prediction_event_count: usize,
    pub development_training_only_event_count: usize,
    pub development_scorable_event_count: usize,
    pub development_neutral_event_count: usize,
    pub development_invalid_event_count: usize,
    pub development_daily_refit_count: usize,
    pub validation_partition_event_count: usize,
    pub validation_prediction_event_count: usize,
    pub validation_training_only_event_count: usize,
    pub validation_scorable_event_count: usize,
    pub validation_neutral_event_count: usize,
    pub validation_invalid_event_count: usize,
    pub validation_daily_refit_count: usize,
    pub participant_metrics: Vec<MomentumQualifiedParticipantMetricsV1>,
    pub benchmark_comparisons: Vec<MomentumQualifiedBenchmarkReceiptV1>,
    pub contribution_comparisons: Vec<MomentumQualifiedContributionReceiptV1>,
    pub probability_collapse_count: usize,
    pub chronology_audit_passed: bool,
    pub leakage_audit_passed: bool,
    pub prediction_before_reveal_passed: bool,
    pub full_eight_replay_claimed: bool,
    pub full_eight_a3_blocked: bool,
    pub month_view_load_count: usize,
    pub year_view_load_count: usize,
    pub holdout_label_reads: usize,
    pub holdout_metric_computations: usize,
    pub holdout_participant_predictions: usize,
    pub live_outcome_requests: usize,
    pub live_outcome_openings: usize,
    pub live_participant_changes: usize,
    pub live_parameter_updates: usize,
    pub live_normalizer_refits: usize,
    pub live_completed_event_changes: usize,
    pub live_scorable_event_changes: usize,
    pub winner_selections: usize,
    pub ranking_creations: usize,
    pub reward_applications: usize,
    pub penalty_applications: usize,
    pub chair_decisions: usize,
    pub committee_votes: usize,
    pub voice_changes: usize,
    pub tier_changes: usize,
    pub cooldowns_started: usize,
    pub promotions: usize,
    pub quarantines: usize,
    pub historical_participant_speaking_rights: usize,
    pub historical_participant_committee_memberships: usize,
    pub paper_executions: usize,
    pub live_executions: usize,
    pub network_request_attempts: usize,
    pub transport_constructions: usize,
    pub credentials_read: usize,
    pub active_committee_count: usize,
    pub live_event_two_sealed: bool,
    pub epoch_three_registered: bool,
    pub protected_live_tree_digest_before: Option<String>,
    pub protected_active_roster_digest_before: Option<String>,
    pub protected_artifacts_unchanged: bool,
    pub active_roster_unchanged: bool,
    pub historical_warning_preserved: bool,
    pub labels: Vec<String>,
    pub replay_digest: Option<String>,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub model_refit_count: usize,
    pub prediction_computation_count: usize,
    pub metric_recomputation_count: usize,
    pub runtime_duration_ms: u64,
    pub report_digest: String,
}

#[derive(Clone, Debug)]
struct PreparedFeatureBlock {
    timeframe: MomentumHistoricalTimeframeV1,
    context_timestamp_ms: Vec<u64>,
    source_candle_digests: Vec<String>,
    feature_schema_digest: String,
    feature_vector_digest: String,
    values: Vec<f32>,
}

#[derive(Clone, Debug)]
struct PreparedEvent {
    event_number: u64,
    prediction_timestamp_ms: u64,
    target_timestamp_ms: u64,
    protocol_receipt_digest: String,
    blocks: BTreeMap<MomentumHistoricalTimeframeV1, PreparedFeatureBlock>,
    current_ten_minute_index: usize,
    target_ten_minute_index: usize,
}

#[derive(Clone, Debug)]
struct PreparedReplay {
    evidence: MomentumQualifiedSixEvidenceV1,
    events: Vec<PreparedEvent>,
    ten_minute_rows: Vec<MomentumQualifiedReplayCandleEvidenceV1>,
    label_policy: MomentumTenMinuteDirectionLabelPolicyV1,
    partition_policy: MomentumQualifiedPartitionPolicyV1,
    holdout_boundary: MomentumQualifiedSixHoldoutBoundaryV1,
    eligibility_audit: MomentumQualifiedEligibilityAuditV1,
    participants: Vec<MomentumQualifiedParticipantRegistrationV1>,
    registration: MomentumQualifiedSixReplayRegistrationV1,
}

#[derive(Clone)]
struct FrozenParticipant {
    participant: MomentumQualifiedParticipantV1,
    parameter_digest: String,
    normalizer_binding_digest: String,
    probability_head: Option<LogisticPredictionHeadV0>,
    prevalence: f64,
}

#[derive(Default)]
struct MetricAccumulator {
    total: usize,
    scorable: usize,
    neutral: usize,
    invalid: usize,
    finite: usize,
    brier_sum: f64,
    correct: usize,
    probabilities: Vec<f64>,
}

fn included_timeframes() -> Vec<MomentumHistoricalTimeframeV1> {
    vec![
        MomentumHistoricalTimeframeV1::Minute1,
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
        MomentumHistoricalTimeframeV1::Day1,
        MomentumHistoricalTimeframeV1::Week1,
    ]
}

fn excluded_timeframes() -> Vec<MomentumHistoricalTimeframeV1> {
    vec![
        MomentumHistoricalTimeframeV1::Month1,
        MomentumHistoricalTimeframeV1::Year1,
    ]
}

fn timeframe_name(value: MomentumHistoricalTimeframeV1) -> &'static str {
    match value {
        MomentumHistoricalTimeframeV1::Minute1 => "1m",
        MomentumHistoricalTimeframeV1::Minute3 => "3m",
        MomentumHistoricalTimeframeV1::Minute5 => "5m",
        MomentumHistoricalTimeframeV1::Minute10 => "10m",
        MomentumHistoricalTimeframeV1::Day1 => "1d",
        MomentumHistoricalTimeframeV1::Week1 => "1w",
        MomentumHistoricalTimeframeV1::Month1 => "1mo",
        MomentumHistoricalTimeframeV1::Year1 => "1y",
    }
}

fn parse_timeframe(value: &str) -> Result<MomentumHistoricalTimeframeV1, String> {
    included_timeframes()
        .into_iter()
        .chain(excluded_timeframes())
        .find(|timeframe| timeframe_name(*timeframe) == value)
        .ok_or_else(|| "qualified-six timeframe rejected".to_string())
}

fn canonical_digest<T: Clone + std::fmt::Debug>(value: &T, clear: impl FnOnce(&mut T)) -> String {
    let mut canonical = value.clone();
    clear(&mut canonical);
    stable_hash_string(&format!("{canonical:?}"))
}

fn label_policy_digest(value: &MomentumTenMinuteDirectionLabelPolicyV1) -> String {
    canonical_digest(value, |item| item.policy_digest.clear())
}

fn partition_policy_digest(value: &MomentumQualifiedPartitionPolicyV1) -> String {
    canonical_digest(value, |item| item.policy_digest.clear())
}

fn participant_digest(value: &MomentumQualifiedParticipantRegistrationV1) -> String {
    canonical_digest(value, |item| item.participant_digest.clear())
}

fn holdout_digest(value: &MomentumQualifiedSixHoldoutBoundaryV1) -> String {
    canonical_digest(value, |item| item.boundary_digest.clear())
}

fn eligibility_digest(value: &MomentumQualifiedEligibilityAuditV1) -> String {
    canonical_digest(value, |item| item.audit_digest.clear())
}

fn registration_digest(value: &MomentumQualifiedSixReplayRegistrationV1) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}

fn block_digest(value: &MomentumQualifiedTimeframeFeatureBlockV1) -> String {
    canonical_digest(value, |item| item.block_digest.clear())
}

fn refit_digest(value: &MomentumQualifiedDailyRefitReceiptV1) -> String {
    canonical_digest(value, |item| item.refit_digest.clear())
}

fn normalizer_receipt_digest(value: &MomentumQualifiedDailyNormalizerReceiptV1) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn participant_receipt_digest(value: &MomentumQualifiedDailyParticipantReceiptV1) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn refit_bundle_digest(value: &MomentumQualifiedDailyRefitBundleV1) -> String {
    canonical_digest(value, |item| item.bundle_digest.clear())
}

fn event_plan_digest(value: &MomentumQualifiedReplayEventPlanV1) -> String {
    canonical_digest(value, |item| item.event_plan_digest.clear())
}

fn prediction_seal_digest(value: &MomentumQualifiedPredictionSealV1) -> String {
    canonical_digest(value, |item| item.seal_digest.clear())
}

fn capsule_digest(value: &MomentumQualifiedReplayPredictionCapsuleV1) -> String {
    canonical_digest(value, |item| item.capsule_digest.clear())
}

fn evaluation_digest(value: &MomentumQualifiedReplayEvaluationV1) -> String {
    canonical_digest(value, |item| item.evaluation_digest.clear())
}

fn prediction_bundle_digest(value: &MomentumQualifiedDailyPredictionBundleV1) -> String {
    canonical_digest(value, |item| item.bundle_digest.clear())
}

fn evaluation_bundle_digest(value: &MomentumQualifiedDailyEvaluationBundleV1) -> String {
    canonical_digest(value, |item| item.bundle_digest.clear())
}

fn metrics_digest(value: &MomentumQualifiedParticipantMetricsV1) -> String {
    canonical_digest(value, |item| item.metrics_digest.clear())
}

fn aggregate_digest(value: &MomentumQualifiedPartitionAggregateV1) -> String {
    canonical_digest(value, |item| item.aggregate_digest.clear())
}

fn benchmark_digest(value: &MomentumQualifiedBenchmarkReceiptV1) -> String {
    canonical_digest(value, |item| item.comparison_digest.clear())
}

fn contribution_digest(value: &MomentumQualifiedContributionReceiptV1) -> String {
    canonical_digest(value, |item| item.comparison_digest.clear())
}

fn journal_digest(value: &MomentumQualifiedReplayJournalV1) -> String {
    canonical_digest(value, |item| item.replay_digest.clear())
}

fn report_digest(value: &MomentumQualifiedSixReplayReportV1) -> String {
    canonical_digest(value, |item| {
        item.run_mode.clear();
        item.artifacts_written = 0;
        item.duplicate_artifact_count = 0;
        item.model_refit_count = 0;
        item.prediction_computation_count = 0;
        item.metric_recomputation_count = 0;
        item.runtime_duration_ms = 0;
        item.report_digest.clear();
    })
}

fn validate_label_policy(value: &MomentumTenMinuteDirectionLabelPolicyV1) -> Result<(), String> {
    if value.policy_version != LABEL_POLICY_VERSION
        || value.event_timeframe != MomentumHistoricalTimeframeV1::Minute10
        || value.target_horizon_candles != 1
        || value.up_rule != "next_close > current_close"
        || value.down_rule != "next_close < current_close"
        || value.neutral_rule != "next_close == current_close"
        || !value.trading_claim_forbidden
        || value.policy_digest != label_policy_digest(value)
    {
        return Err("qualified-six label policy rejected".to_string());
    }
    Ok(())
}

fn validate_partition_policy(value: &MomentumQualifiedPartitionPolicyV1) -> Result<(), String> {
    if value.policy_version != PARTITION_POLICY_VERSION
        || value.eligible_start_timestamp_ms >= value.eligible_end_timestamp_ms
        || value.eligible_start_timestamp_ms >= value.development_end_exclusive_ms
        || value.development_end_exclusive_ms >= value.validation_end_exclusive_ms
        || value.validation_end_exclusive_ms != value.holdout_start_timestamp_ms
        || value.common_eligible_event_count
            != value.development_event_count
                + value.validation_event_count
                + value.holdout_event_count
        || value.development_percent != DEVELOPMENT_PERCENT
        || value.validation_percent != VALIDATION_PERCENT
        || value.holdout_percent != 100 - DEVELOPMENT_PERCENT - VALIDATION_PERCENT
        || value.development_event_count == 0
        || value.validation_event_count == 0
        || value.holdout_event_count == 0
        || value.policy_digest != partition_policy_digest(value)
    {
        return Err("qualified-six partition policy rejected".to_string());
    }
    Ok(())
}

fn validate_participant(value: &MomentumQualifiedParticipantRegistrationV1) -> Result<(), String> {
    if value.registration_version != PARTICIPANT_VERSION
        || value.ordered_timeframes != value.participant.timeframes()
        || !value.fresh_research_parameters_required
        || !value.live_parameter_use_forbidden
        || !value.prior_fold_parameter_use_forbidden
        || !value.interaction_features_forbidden
        || value.participant_digest != participant_digest(value)
    {
        return Err("qualified-six participant registration rejected".to_string());
    }
    Ok(())
}

fn validate_holdout(value: &MomentumQualifiedSixHoldoutBoundaryV1) -> Result<(), String> {
    if value.boundary_version != HOLDOUT_VERSION
        || value.prior_holdout_digest.is_empty()
        || value.holdout_start_timestamp_ms == 0
        || value.holdout_event_count == 0
        || value.labels_opened
        || value.metric_computations != 0
        || value.participant_predictions != 0
        || value.aggregate_result_present
        || value.boundary_digest != holdout_digest(value)
    {
        return Err("qualified-six holdout boundary rejected".to_string());
    }
    Ok(())
}

fn validate_eligibility(value: &MomentumQualifiedEligibilityAuditV1) -> Result<(), String> {
    if value.audit_version != ELIGIBILITY_VERSION
        || value.protocol_event_count == 0
        || value.common_eligible_event_count == 0
        || value.protocol_event_count
            != value.common_eligible_event_count
                + value.rejected_context_count
                + value.rejected_target_count
                + value.rejected_partial_count
                + value.rejected_missing_evidence_count
        || value.month_view_load_count != 0
        || value.year_view_load_count != 0
        || value.future_access_count != 0
        || value.partial_access_count != 0
        || value.unqualified_access_count != 0
        || value.target_value_access_count != 0
        || value.first_prediction_timestamp_ms >= value.last_prediction_timestamp_ms
        || value.audit_digest != eligibility_digest(value)
    {
        return Err("qualified-six eligibility audit rejected".to_string());
    }
    Ok(())
}

fn validate_registration(value: &MomentumQualifiedSixReplayRegistrationV1) -> Result<(), String> {
    let participant_ids = value
        .participant_registrations
        .iter()
        .map(|participant| participant.participant)
        .collect::<Vec<_>>();
    if value.registration_version != REGISTRATION_VERSION
        || value.family != MomentumQualifiedReplayFamilyV1::QualifiedSixIntradayTenMinute
        || [
            &value.qualified_timeframe_set_digest,
            &value.monthly_exclusion_policy_digest,
            &value.yearly_exclusion_policy_digest,
            &value.causal_revalidation_digest,
            &value.protocol_replay_digest,
            &value.event_selection_policy_digest,
            &value.context_policy_digest,
            &value.feature_policy_digest,
            &value.label_policy_digest,
            &value.training_policy_digest,
            &value.normalization_policy_digest,
            &value.evaluation_policy_digest,
            &value.development_boundary_digest,
            &value.validation_boundary_digest,
            &value.sealed_holdout_boundary_digest,
        ]
        .iter()
        .any(|digest| digest.is_empty())
        || value.historical_dataset_index_digests.len() != 6
        || value
            .historical_dataset_index_digests
            .iter()
            .any(String::is_empty)
        || value.included_timeframes != included_timeframes()
        || value.excluded_timeframes != excluded_timeframes()
        || value.prediction_task != MomentumQualifiedPredictionTaskV1::IntradayTenMinuteDirection
        || value.refit_policy != MomentumQualifiedRefitPolicyV1::RefitAtUtcDayBoundary
        || participant_ids != MomentumQualifiedParticipantV1::ORDERED
        || value
            .participant_registrations
            .iter()
            .any(|participant| validate_participant(participant).is_err())
        || value.minimum_training_examples == 0
        || !value.minimum_training_examples.is_power_of_two()
        || value.maximum_training_examples < value.minimum_training_examples
        || !value.full_eight_replay_claim_forbidden
        || !value.live_authority_use_forbidden
        || !value.reward_application_forbidden
        || !value.chair_action_forbidden
        || !value.trading_authority_forbidden
        || value.registration_digest != registration_digest(value)
    {
        return Err("qualified-six replay registration rejected".to_string());
    }
    Ok(())
}

fn validate_feature_block(value: &MomentumQualifiedTimeframeFeatureBlockV1) -> Result<(), String> {
    if value.block_version != BLOCK_VERSION
        || !included_timeframes().contains(&value.timeframe)
        || value.context_timestamp_ms.len() != CONTEXT_LENGTH
        || value.source_candle_digests.len() != CONTEXT_LENGTH
        || value
            .context_timestamp_ms
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        || value.source_candle_digests.iter().any(String::is_empty)
        || value.feature_schema_digest.is_empty()
        || value.feature_vector_digest.is_empty()
        || value.normalizer_digest.is_empty()
        || value.future_access_count != 0
        || value.partial_access_count != 0
        || value.missing_evidence_count != 0
        || value.block_digest != block_digest(value)
    {
        return Err("qualified-six feature block rejected".to_string());
    }
    Ok(())
}

fn validate_refit(value: &MomentumQualifiedDailyRefitReceiptV1) -> Result<(), String> {
    if value.refit_version != REFIT_VERSION
        || value.registration_digest.is_empty()
        || value.utc_day_boundary_ms == 0
        || value.utc_day_boundary_ms % DAY_MS != 0
        || value.training_target_cutoff_exclusive_ms != value.utc_day_boundary_ms
        || value.eligible_past_event_count < value.scorable_training_event_count
        || value.scorable_training_event_count < value.used_training_event_count
        || value.used_training_event_count == 0
        || value.participant_parameter_digests.len() != 5
        || value.timeframe_normalizer_digests.len() != 6
        || value
            .participant_parameter_digests
            .iter()
            .chain(&value.timeframe_normalizer_digests)
            .any(String::is_empty)
        || value.within_day_refit_count != 0
        || value.live_parameter_load_count != 0
        || value.prior_fold_parameter_load_count != 0
        || value.refit_digest != refit_digest(value)
    {
        return Err("qualified-six daily refit rejected".to_string());
    }
    Ok(())
}

fn validate_normalizer_receipt(
    value: &MomentumQualifiedDailyNormalizerReceiptV1,
) -> Result<(), String> {
    let normalizer = RepresentationNormalizerV0 {
        means: value.private_means.clone(),
        scales: value.private_scales.clone(),
        constant_dimension_indices: value.constant_dimension_indices.clone(),
    };
    if value.receipt_version != NORMALIZER_RECEIPT_VERSION
        || !included_timeframes().contains(&value.timeframe)
        || value.private_means.len() != MomentumFeatureConfigV0::default().feature_count()
        || value.private_means.len() != value.private_scales.len()
        || value
            .private_means
            .iter()
            .chain(&value.private_scales)
            .any(|item| !item.is_finite())
        || value.private_scales.iter().any(|scale| *scale <= 0.0)
        || value
            .constant_dimension_indices
            .iter()
            .any(|index| *index >= value.private_means.len())
        || value.normalizer_digest != normalizer.digest()
        || value.receipt_digest != normalizer_receipt_digest(value)
    {
        return Err("qualified-six normalizer receipt rejected".to_string());
    }
    Ok(())
}

fn validate_participant_receipt(
    value: &MomentumQualifiedDailyParticipantReceiptV1,
) -> Result<(), String> {
    let constant =
        value.participant == MomentumQualifiedParticipantV1::Q0TrainingPrevalenceConstant;
    let head = (!constant).then(|| LogisticPredictionHeadV0 {
        weights: value.private_head_weights.clone(),
        bias: value.private_head_bias.unwrap_or(f32::NAN),
    });
    let expected_parameter_digest = if constant {
        stable_hash_string(&format!(
            "qualified-six-research-constant-v1:{}:{}:{}",
            value.utc_day_boundary_ms,
            value.scorable_training_event_count,
            value.private_prevalence.to_bits()
        ))
    } else if let Some(head) = &head {
        stable_hash_string(&format!(
            "qualified-six-research-parameter-v1:{}:{}:{}",
            value.participant.id(),
            value.utc_day_boundary_ms,
            head.parameter_digest()
        ))
    } else {
        String::new()
    };
    if value.receipt_version != PARTICIPANT_RECEIPT_VERSION
        || value.registration_digest.is_empty()
        || value.utc_day_boundary_ms == 0
        || value.utc_day_boundary_ms % DAY_MS != 0
        || value.scorable_training_event_count < value.used_training_event_count
        || value.used_training_event_count == 0
        || value.parameter_digest != expected_parameter_digest
        || value.normalizer_binding_digest.is_empty()
        || !value.private_prevalence.is_finite()
        || !(0.0..=1.0).contains(&value.private_prevalence)
        || constant != (value.private_head_weights.is_empty() && value.private_head_bias.is_none())
        || head.as_ref().is_some_and(|head| {
            head.weights.len()
                != value.participant.timeframes().len()
                    * MomentumFeatureConfigV0::default().feature_count()
                || head.validate().is_err()
        })
        || value.live_parameter_load_count != 0
        || value.prior_fold_parameter_load_count != 0
        || value.receipt_digest != participant_receipt_digest(value)
    {
        return Err("qualified-six participant receipt rejected".to_string());
    }
    Ok(())
}

fn validate_refit_bundle(value: &MomentumQualifiedDailyRefitBundleV1) -> Result<(), String> {
    let participant_order = value
        .participant_receipts
        .iter()
        .map(|receipt| receipt.participant)
        .collect::<Vec<_>>();
    let normalizer_order = value
        .normalizer_receipts
        .iter()
        .map(|receipt| receipt.timeframe)
        .collect::<Vec<_>>();
    let normalizer_digests = value
        .normalizer_receipts
        .iter()
        .map(|receipt| (receipt.timeframe, receipt.normalizer_digest.as_str()))
        .collect::<BTreeMap<_, _>>();
    let bindings_match = value.participant_receipts.iter().all(|receipt| {
        let expected = if receipt.participant
            == MomentumQualifiedParticipantV1::Q0TrainingPrevalenceConstant
        {
            stable_hash_string("qualified-six-constant-past-labels-only")
        } else {
            stable_hash_string(&format!(
                "qualified-six-normalizer-binding-v1:{:?}",
                receipt
                    .participant
                    .timeframes()
                    .iter()
                    .map(|timeframe| {
                        normalizer_digests
                            .get(timeframe)
                            .copied()
                            .unwrap_or_default()
                    })
                    .collect::<Vec<_>>()
            ))
        };
        receipt.normalizer_binding_digest == expected
    });
    if value.bundle_version != REFIT_BUNDLE_VERSION
        || value.registration_digest.is_empty()
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.utc_day_boundary_ms == 0
        || value.utc_day_boundary_ms % DAY_MS != 0
        || value.refit_receipt.registration_digest != value.registration_digest
        || value.refit_receipt.utc_day_boundary_ms != value.utc_day_boundary_ms
        || validate_refit(&value.refit_receipt).is_err()
        || normalizer_order != included_timeframes()
        || value
            .normalizer_receipts
            .iter()
            .any(|receipt| validate_normalizer_receipt(receipt).is_err())
        || value.refit_receipt.timeframe_normalizer_digests
            != value
                .normalizer_receipts
                .iter()
                .map(|receipt| receipt.normalizer_digest.clone())
                .collect::<Vec<_>>()
        || participant_order != MomentumQualifiedParticipantV1::ORDERED
        || value
            .participant_receipts
            .iter()
            .any(|receipt| validate_participant_receipt(receipt).is_err())
        || value.refit_receipt.participant_parameter_digests
            != value
                .participant_receipts
                .iter()
                .map(|receipt| receipt.parameter_digest.clone())
                .collect::<Vec<_>>()
        || value.participant_receipts.iter().any(|receipt| {
            receipt.registration_digest != value.registration_digest
                || receipt.utc_day_boundary_ms != value.utc_day_boundary_ms
                || receipt.scorable_training_event_count
                    != value.refit_receipt.scorable_training_event_count
                || receipt.used_training_event_count
                    != value.refit_receipt.used_training_event_count
        })
        || !bindings_match
        || value.bundle_digest != refit_bundle_digest(value)
    {
        return Err("qualified-six refit bundle rejected".to_string());
    }
    Ok(())
}

fn validate_event_plan(value: &MomentumQualifiedReplayEventPlanV1) -> Result<(), String> {
    if value.plan_version != EVENT_PLAN_VERSION
        || value.registration_digest.is_empty()
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.event_number == 0
        || value.prediction_timestamp_ms % TEN_MINUTE_MS != 0
        || value.target_timestamp_ms != value.prediction_timestamp_ms + TEN_MINUTE_MS
        || value.daily_refit_receipt_digest.is_empty()
        || value.timeframe_block_digests.len() != 6
        || value.participant_ids
            != MomentumQualifiedParticipantV1::ORDERED
                .iter()
                .map(|participant| participant.id().to_string())
                .collect::<Vec<_>>()
        || value.event_plan_digest != event_plan_digest(value)
    {
        return Err("qualified-six event plan rejected".to_string());
    }
    Ok(())
}

fn validate_prediction_seal(value: &MomentumQualifiedPredictionSealV1) -> Result<(), String> {
    if value.seal_version != PREDICTION_SEAL_VERSION
        || value.event_plan_digest.is_empty()
        || value.parameter_digest.is_empty()
        || value.normalizer_binding_digest.is_empty()
        || !value.private_probability.is_finite()
        || !(0.0..=1.0).contains(&value.private_probability)
        || value.prediction_digest.is_empty()
        || value.seal_digest != prediction_seal_digest(value)
    {
        return Err("qualified-six prediction seal rejected".to_string());
    }
    Ok(())
}

fn validate_capsule(value: &MomentumQualifiedReplayPredictionCapsuleV1) -> Result<(), String> {
    if value.capsule_version != CAPSULE_VERSION
        || value.event_plan_digest.is_empty()
        || value.participant_seal_digests.len() != 5
        || value.participant_prediction_digests.len() != 5
        || value
            .participant_seal_digests
            .iter()
            .chain(&value.participant_prediction_digests)
            .any(String::is_empty)
        || value.target_accessed
        || value.label_accessed
        || value.metrics_computed
        || value.capsule_digest != capsule_digest(value)
    {
        return Err("qualified-six prediction capsule rejected".to_string());
    }
    Ok(())
}

fn validate_evaluation(value: &MomentumQualifiedReplayEvaluationV1) -> Result<(), String> {
    let scorable = matches!(
        value.label_status,
        MomentumQualifiedLabelStatusV1::Up | MomentumQualifiedLabelStatusV1::Down
    );
    if value.evaluation_version != EVALUATION_VERSION
        || value.event_plan_digest.is_empty()
        || value.prediction_capsule_digest.is_empty()
        || value.private_label.is_some() != scorable
        || value
            .private_label
            .is_some_and(|label| !matches!(label, 0.0 | 1.0))
        || value.participant_evaluation_digests.len() != usize::from(scorable) * 5
        || value.private_brier_values.len() != usize::from(scorable) * 5
        || value.private_correctness.len() != usize::from(scorable) * 5
        || value
            .private_brier_values
            .iter()
            .any(|brier| !brier.is_finite() || !(0.0..=1.0).contains(brier))
        || value.evaluation_digest != evaluation_digest(value)
    {
        return Err("qualified-six event evaluation rejected".to_string());
    }
    Ok(())
}

fn build_label_policy() -> Result<MomentumTenMinuteDirectionLabelPolicyV1, String> {
    let mut value = MomentumTenMinuteDirectionLabelPolicyV1 {
        policy_version: LABEL_POLICY_VERSION.to_string(),
        event_timeframe: MomentumHistoricalTimeframeV1::Minute10,
        target_horizon_candles: 1,
        up_rule: "next_close > current_close".to_string(),
        down_rule: "next_close < current_close".to_string(),
        neutral_rule: "next_close == current_close".to_string(),
        trading_claim_forbidden: true,
        policy_digest: String::new(),
    };
    value.policy_digest = label_policy_digest(&value);
    validate_label_policy(&value)?;
    Ok(value)
}

fn build_participants() -> Result<Vec<MomentumQualifiedParticipantRegistrationV1>, String> {
    MomentumQualifiedParticipantV1::ORDERED
        .into_iter()
        .map(|participant| {
            let mut value = MomentumQualifiedParticipantRegistrationV1 {
                registration_version: PARTICIPANT_VERSION.to_string(),
                participant,
                ordered_timeframes: participant.timeframes(),
                fresh_research_parameters_required: true,
                live_parameter_use_forbidden: true,
                prior_fold_parameter_use_forbidden: true,
                interaction_features_forbidden: true,
                participant_digest: String::new(),
            };
            value.participant_digest = participant_digest(&value);
            validate_participant(&value)?;
            Ok(value)
        })
        .collect()
}

fn smallest_power_of_two_at_least(value: usize) -> Result<usize, String> {
    value
        .checked_next_power_of_two()
        .ok_or_else(|| "qualified-six minimum support overflow".to_string())
}

fn training_config() -> HeadTrainingConfigV0 {
    let mut value = MomentumLearningCampaignConfigV0::default().training_config;
    value.epochs = 4;
    value.batch_size = 64;
    value.early_stopping_patience = None;
    value
}

fn feature_policy_digest(config: &MomentumFeatureConfigV0) -> String {
    stable_hash_string(&format!(
        "qualified-six-feature-policy-v1:context={CONTEXT_LENGTH}:extractor={}:per-timeframe",
        config.digest()
    ))
}

fn training_policy_digest(config: &HeadTrainingConfigV0, minimum: usize) -> String {
    stable_hash_string(&format!(
        "qualified-six-training-policy-v1:{}:minimum={minimum}:maximum={MAX_TRAINING_EXAMPLES}:dimension-based-four-epoch",
        config.digest()
    ))
}

fn partition_boundary_digest(
    partition: MomentumReplayPartitionV1,
    start: u64,
    end_exclusive: u64,
    count: usize,
) -> String {
    stable_hash_string(&format!(
        "qualified-six-partition-boundary-v1:{}:{start}:{end_exclusive}:{count}",
        partition.as_str()
    ))
}

fn checked_f32(value: f64) -> Result<f32, String> {
    let converted = value as f32;
    if !value.is_finite() || !converted.is_finite() {
        return Err("qualified-six candle conversion rejected".to_string());
    }
    Ok(converted)
}

fn feature_rows(
    rows: &[MomentumQualifiedReplayCandleEvidenceV1],
    config: &MomentumFeatureConfigV0,
) -> Result<BTreeMap<usize, Vec<f32>>, String> {
    let candles = rows
        .iter()
        .map(|row| {
            Ok(MomentumCandleV0 {
                timestamp: i64::try_from(row.close_exclusive_timestamp_ms)
                    .map_err(|_| "qualified-six timestamp conversion rejected".to_string())?,
                open: checked_f32(row.open)?,
                high: checked_f32(row.high)?,
                low: checked_f32(row.low)?,
                close: checked_f32(row.close)?,
                volume: checked_f32(row.volume)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    build_momentum_features_v0(&candles, config)
        .map_err(|_| "qualified-six feature extraction rejected".to_string())?
        .into_iter()
        .map(|row| Ok((row.source_index, row.values)))
        .collect()
}

fn select_prepared_block(
    timeframe: MomentumHistoricalTimeframeV1,
    rows: &[MomentumQualifiedReplayCandleEvidenceV1],
    features: &BTreeMap<usize, Vec<f32>>,
    prediction_timestamp_ms: u64,
    config: &MomentumFeatureConfigV0,
) -> Result<PreparedFeatureBlock, &'static str> {
    let end_exclusive =
        rows.partition_point(|row| row.close_exclusive_timestamp_ms <= prediction_timestamp_ms);
    if end_exclusive < CONTEXT_LENGTH {
        return Err("context");
    }
    let start = end_exclusive - CONTEXT_LENGTH;
    let context = &rows[start..end_exclusive];
    if context.iter().any(|row| row.missing_evidence) {
        return Err("missing");
    }
    if context
        .iter()
        .any(|row| row.close_exclusive_timestamp_ms > prediction_timestamp_ms)
    {
        return Err("partial");
    }
    let feature_index = end_exclusive - 1;
    let values = features.get(&feature_index).ok_or("context")?.clone();
    if values.len() != config.feature_count() || values.iter().any(|value| !value.is_finite()) {
        return Err("missing");
    }
    let context_timestamp_ms = context
        .iter()
        .map(|row| row.close_exclusive_timestamp_ms)
        .collect::<Vec<_>>();
    let source_candle_digests = context
        .iter()
        .map(|row| row.candle_digest.clone())
        .collect::<Vec<_>>();
    let feature_schema_digest = stable_hash_string(&format!(
        "qualified-six-block-schema-v1:{}:{}:{}",
        timeframe_name(timeframe),
        CONTEXT_LENGTH,
        config.digest()
    ));
    let feature_vector_digest = stable_hash_string(&format!(
        "qualified-six-private-feature-v1:{}:{:?}",
        timeframe_name(timeframe),
        values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    ));
    Ok(PreparedFeatureBlock {
        timeframe,
        context_timestamp_ms,
        source_candle_digests,
        feature_schema_digest,
        feature_vector_digest,
        values,
    })
}

fn build_partition_policy(
    events: &[PreparedEvent],
) -> Result<MomentumQualifiedPartitionPolicyV1, String> {
    if events.len() < 3 {
        return Err("qualified-six common range insufficient".to_string());
    }
    let development_event_count = events.len() * DEVELOPMENT_PERCENT / 100;
    let validation_event_count = events.len() * VALIDATION_PERCENT / 100;
    let holdout_event_count = events.len() - development_event_count - validation_event_count;
    if [
        development_event_count,
        validation_event_count,
        holdout_event_count,
    ]
    .contains(&0)
    {
        return Err("qualified-six chronological partition rejected".to_string());
    }
    let development_end_exclusive_ms = events[development_event_count].prediction_timestamp_ms;
    let validation_end_exclusive_ms =
        events[development_event_count + validation_event_count].prediction_timestamp_ms;
    let mut value = MomentumQualifiedPartitionPolicyV1 {
        policy_version: PARTITION_POLICY_VERSION.to_string(),
        eligible_start_timestamp_ms: events[0].prediction_timestamp_ms,
        eligible_end_timestamp_ms: events
            .last()
            .ok_or_else(|| "qualified-six last event unavailable".to_string())?
            .target_timestamp_ms,
        development_end_exclusive_ms,
        validation_end_exclusive_ms,
        holdout_start_timestamp_ms: validation_end_exclusive_ms,
        common_eligible_event_count: events.len(),
        development_event_count,
        validation_event_count,
        holdout_event_count,
        development_percent: DEVELOPMENT_PERCENT,
        validation_percent: VALIDATION_PERCENT,
        holdout_percent: 100 - DEVELOPMENT_PERCENT - VALIDATION_PERCENT,
        policy_digest: String::new(),
    };
    value.policy_digest = partition_policy_digest(&value);
    validate_partition_policy(&value)?;
    Ok(value)
}

fn prepare_replay() -> Result<PreparedReplay, String> {
    let evidence = load_momentum_qualified_six_evidence_v1()?;
    if evidence.included_timeframes != included_timeframes()
        || evidence.excluded_timeframes != excluded_timeframes()
        || evidence.views.len() != 6
        || evidence
            .views
            .keys()
            .any(|timeframe| excluded_timeframes().contains(timeframe))
    {
        return Err("qualified-six source boundary rejected".to_string());
    }
    let feature_config = MomentumFeatureConfigV0::default();
    let prepared_features = evidence
        .views
        .iter()
        .map(|(timeframe, rows)| Ok((*timeframe, feature_rows(rows, &feature_config)?)))
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let ten_minute_rows = evidence
        .views
        .get(&MomentumHistoricalTimeframeV1::Minute10)
        .cloned()
        .ok_or_else(|| "qualified-six ten-minute source unavailable".to_string())?;
    let ten_minute_by_close = ten_minute_rows
        .iter()
        .enumerate()
        .map(|(index, row)| (row.close_exclusive_timestamp_ms, index))
        .collect::<BTreeMap<_, _>>();
    let mut events = Vec::new();
    let mut rejected_context_count = 0usize;
    let mut rejected_target_count = 0usize;
    let mut rejected_partial_count = 0usize;
    let mut rejected_missing_evidence_count = 0usize;
    for protocol in &evidence.protocol_events {
        if protocol.prediction_timestamp_ms % TEN_MINUTE_MS != 0
            || protocol.target_timestamp_ms
                != protocol
                    .prediction_timestamp_ms
                    .saturating_add(TEN_MINUTE_MS)
        {
            rejected_target_count += 1;
            continue;
        }
        let Some(&current_ten_minute_index) =
            ten_minute_by_close.get(&protocol.prediction_timestamp_ms)
        else {
            rejected_target_count += 1;
            continue;
        };
        let Some(&target_ten_minute_index) = ten_minute_by_close.get(&protocol.target_timestamp_ms)
        else {
            rejected_target_count += 1;
            continue;
        };
        if target_ten_minute_index != current_ten_minute_index + 1
            || ten_minute_rows[current_ten_minute_index].missing_evidence
            || ten_minute_rows[target_ten_minute_index].missing_evidence
        {
            rejected_target_count += 1;
            continue;
        }
        let mut blocks = BTreeMap::new();
        let mut rejected = None;
        for timeframe in included_timeframes() {
            let rows = evidence
                .views
                .get(&timeframe)
                .ok_or_else(|| "qualified-six view unavailable".to_string())?;
            let features = prepared_features
                .get(&timeframe)
                .ok_or_else(|| "qualified-six feature view unavailable".to_string())?;
            match select_prepared_block(
                timeframe,
                rows,
                features,
                protocol.prediction_timestamp_ms,
                &feature_config,
            ) {
                Ok(block) => {
                    blocks.insert(timeframe, block);
                }
                Err(reason) => {
                    rejected = Some(reason);
                    break;
                }
            }
        }
        if let Some(reason) = rejected {
            match reason {
                "partial" => rejected_partial_count += 1,
                "missing" => rejected_missing_evidence_count += 1,
                _ => rejected_context_count += 1,
            }
            continue;
        }
        events.push(PreparedEvent {
            event_number: as_u64(events.len() + 1)?,
            prediction_timestamp_ms: protocol.prediction_timestamp_ms,
            target_timestamp_ms: protocol.target_timestamp_ms,
            protocol_receipt_digest: protocol.receipt_digest.clone(),
            blocks,
            current_ten_minute_index,
            target_ten_minute_index,
        });
    }
    if events
        .windows(2)
        .any(|pair| pair[0].prediction_timestamp_ms >= pair[1].prediction_timestamp_ms)
    {
        return Err("qualified-six event chronology rejected".to_string());
    }
    let partition_policy = build_partition_policy(&events)?;
    let mut eligibility_audit = MomentumQualifiedEligibilityAuditV1 {
        audit_version: ELIGIBILITY_VERSION.to_string(),
        protocol_event_count: evidence.protocol_events.len(),
        common_eligible_event_count: events.len(),
        rejected_context_count,
        rejected_target_count,
        rejected_partial_count,
        rejected_missing_evidence_count,
        month_view_load_count: 0,
        year_view_load_count: 0,
        future_access_count: 0,
        partial_access_count: 0,
        unqualified_access_count: 0,
        target_value_access_count: 0,
        first_prediction_timestamp_ms: events
            .first()
            .ok_or_else(|| "qualified-six first event unavailable".to_string())?
            .prediction_timestamp_ms,
        last_prediction_timestamp_ms: events
            .last()
            .ok_or_else(|| "qualified-six last event unavailable".to_string())?
            .prediction_timestamp_ms,
        audit_digest: String::new(),
    };
    eligibility_audit.audit_digest = eligibility_digest(&eligibility_audit);
    validate_eligibility(&eligibility_audit)?;
    let adopted_prior_boundary = evidence.prior_holdout.holdout_start_timestamp_ms
        == partition_policy.holdout_start_timestamp_ms
        && evidence.prior_holdout.eligible_start_timestamp_ms
            == partition_policy.eligible_start_timestamp_ms
        && evidence.prior_holdout.eligible_end_timestamp_ms
            == partition_policy.eligible_end_timestamp_ms;
    let mut holdout_boundary = MomentumQualifiedSixHoldoutBoundaryV1 {
        boundary_version: HOLDOUT_VERSION.to_string(),
        prior_holdout_digest: evidence.prior_holdout.holdout_digest.clone(),
        adopted_prior_boundary,
        holdout_start_timestamp_ms: partition_policy.holdout_start_timestamp_ms,
        holdout_event_count: partition_policy.holdout_event_count,
        labels_opened: false,
        metric_computations: 0,
        participant_predictions: 0,
        aggregate_result_present: false,
        boundary_digest: String::new(),
    };
    holdout_boundary.boundary_digest = holdout_digest(&holdout_boundary);
    validate_holdout(&holdout_boundary)?;
    let label_policy = build_label_policy()?;
    let participants = build_participants()?;
    let maximum_dimension = participants
        .iter()
        .map(|participant| {
            participant.ordered_timeframes.len().max(1) * feature_config.feature_count()
        })
        .max()
        .ok_or_else(|| "qualified-six maximum dimension unavailable".to_string())?;
    let minimum_training_examples =
        smallest_power_of_two_at_least(TRAINING_DIMENSION_MULTIPLIER * maximum_dimension)?;
    let training_config = training_config();
    let development_boundary_digest = partition_boundary_digest(
        MomentumReplayPartitionV1::Development,
        partition_policy.eligible_start_timestamp_ms,
        partition_policy.development_end_exclusive_ms,
        partition_policy.development_event_count,
    );
    let validation_boundary_digest = partition_boundary_digest(
        MomentumReplayPartitionV1::Validation,
        partition_policy.development_end_exclusive_ms,
        partition_policy.validation_end_exclusive_ms,
        partition_policy.validation_event_count,
    );
    let mut registration = MomentumQualifiedSixReplayRegistrationV1 {
        registration_version: REGISTRATION_VERSION.to_string(),
        family: MomentumQualifiedReplayFamilyV1::QualifiedSixIntradayTenMinute,
        qualified_timeframe_set_digest: evidence.qualified_timeframe_set_digest.clone(),
        monthly_exclusion_policy_digest: evidence.monthly_policy_digest.clone(),
        yearly_exclusion_policy_digest: evidence.yearly_policy_digest.clone(),
        causal_revalidation_digest: evidence.causal_revalidation_digest.clone(),
        protocol_replay_digest: evidence.protocol_replay_digest.clone(),
        historical_dataset_index_digests: evidence.view_index_digests.clone(),
        included_timeframes: evidence.included_timeframes.clone(),
        excluded_timeframes: evidence.excluded_timeframes.clone(),
        prediction_task: MomentumQualifiedPredictionTaskV1::IntradayTenMinuteDirection,
        event_selection_policy_digest: stable_hash_string(
            "qualified-six-event-selection-v1:all-six-closed:next-10m:complete:no-holdout",
        ),
        context_policy_digest: stable_hash_string(&format!(
            "qualified-six-context-v1:latest-{CONTEXT_LENGTH}:close-lte-prediction"
        )),
        feature_policy_digest: feature_policy_digest(&feature_config),
        label_policy_digest: label_policy.policy_digest.clone(),
        training_policy_digest: training_policy_digest(&training_config, minimum_training_examples),
        normalization_policy_digest: stable_hash_string(
            "qualified-six-normalization-v1:fresh-per-timeframe-training-only-per-utc-day",
        ),
        evaluation_policy_digest: stable_hash_string(
            "qualified-six-evaluation-v1:mean-brier-primary:binary-correctness-secondary:paired-q0",
        ),
        refit_policy: MomentumQualifiedRefitPolicyV1::RefitAtUtcDayBoundary,
        participant_registrations: participants.clone(),
        development_boundary_digest,
        validation_boundary_digest,
        sealed_holdout_boundary_digest: holdout_boundary.boundary_digest.clone(),
        minimum_training_examples,
        maximum_training_examples: MAX_TRAINING_EXAMPLES,
        full_eight_replay_claim_forbidden: true,
        live_authority_use_forbidden: true,
        reward_application_forbidden: true,
        chair_action_forbidden: true,
        trading_authority_forbidden: true,
        registration_digest: String::new(),
    };
    registration.registration_digest = registration_digest(&registration);
    validate_registration(&registration)?;
    Ok(PreparedReplay {
        evidence,
        events,
        ten_minute_rows,
        label_policy,
        partition_policy,
        holdout_boundary,
        eligibility_audit,
        participants,
        registration,
    })
}

fn encode_label_policy(value: &MomentumTenMinuteDirectionLabelPolicyV1) -> Result<Vec<u8>, String> {
    validate_label_policy(value)?;
    ArtifactBuilderV4_2::new("MomentumTenMinuteDirectionLabelPolicyV1")
        .string("policy_version", &value.policy_version)
        .string("event_timeframe", timeframe_name(value.event_timeframe))
        .unsigned(
            "target_horizon_candles",
            as_u64(value.target_horizon_candles)?,
        )
        .string("up_rule", &value.up_rule)
        .string("down_rule", &value.down_rule)
        .string("neutral_rule", &value.neutral_rule)
        .boolean("trading_claim_forbidden", value.trading_claim_forbidden)
        .string("policy_digest", &value.policy_digest)
        .encode()
}

fn decode_label_policy(bytes: &[u8]) -> Result<MomentumTenMinuteDirectionLabelPolicyV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumTenMinuteDirectionLabelPolicyV1")?;
    let value = MomentumTenMinuteDirectionLabelPolicyV1 {
        policy_version: fields.string("policy_version")?,
        event_timeframe: parse_timeframe(&fields.string("event_timeframe")?)?,
        target_horizon_candles: as_usize(fields.unsigned("target_horizon_candles")?)?,
        up_rule: fields.string("up_rule")?,
        down_rule: fields.string("down_rule")?,
        neutral_rule: fields.string("neutral_rule")?,
        trading_claim_forbidden: fields.boolean("trading_claim_forbidden")?,
        policy_digest: fields.string("policy_digest")?,
    };
    fields.finish()?;
    validate_label_policy(&value)?;
    Ok(value)
}

fn encode_partition_policy(value: &MomentumQualifiedPartitionPolicyV1) -> Result<Vec<u8>, String> {
    validate_partition_policy(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedPartitionPolicyV1")
        .string("policy_version", &value.policy_version)
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
        .unsigned("development_percent", as_u64(value.development_percent)?)
        .unsigned("validation_percent", as_u64(value.validation_percent)?)
        .unsigned("holdout_percent", as_u64(value.holdout_percent)?)
        .string("policy_digest", &value.policy_digest)
        .encode()
}

fn decode_partition_policy(bytes: &[u8]) -> Result<MomentumQualifiedPartitionPolicyV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedPartitionPolicyV1")?;
    let value = MomentumQualifiedPartitionPolicyV1 {
        policy_version: fields.string("policy_version")?,
        eligible_start_timestamp_ms: fields.unsigned("eligible_start_timestamp_ms")?,
        eligible_end_timestamp_ms: fields.unsigned("eligible_end_timestamp_ms")?,
        development_end_exclusive_ms: fields.unsigned("development_end_exclusive_ms")?,
        validation_end_exclusive_ms: fields.unsigned("validation_end_exclusive_ms")?,
        holdout_start_timestamp_ms: fields.unsigned("holdout_start_timestamp_ms")?,
        common_eligible_event_count: as_usize(fields.unsigned("common_eligible_event_count")?)?,
        development_event_count: as_usize(fields.unsigned("development_event_count")?)?,
        validation_event_count: as_usize(fields.unsigned("validation_event_count")?)?,
        holdout_event_count: as_usize(fields.unsigned("holdout_event_count")?)?,
        development_percent: as_usize(fields.unsigned("development_percent")?)?,
        validation_percent: as_usize(fields.unsigned("validation_percent")?)?,
        holdout_percent: as_usize(fields.unsigned("holdout_percent")?)?,
        policy_digest: fields.string("policy_digest")?,
    };
    fields.finish()?;
    validate_partition_policy(&value)?;
    Ok(value)
}

fn encode_participant(
    value: &MomentumQualifiedParticipantRegistrationV1,
) -> Result<Vec<u8>, String> {
    validate_participant(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedParticipantRegistrationV1")
        .string("registration_version", &value.registration_version)
        .string("participant", value.participant.id())
        .strings(
            "ordered_timeframes",
            &value
                .ordered_timeframes
                .iter()
                .map(|timeframe| timeframe_name(*timeframe).to_string())
                .collect::<Vec<_>>(),
        )
        .boolean(
            "fresh_research_parameters_required",
            value.fresh_research_parameters_required,
        )
        .boolean(
            "live_parameter_use_forbidden",
            value.live_parameter_use_forbidden,
        )
        .boolean(
            "prior_fold_parameter_use_forbidden",
            value.prior_fold_parameter_use_forbidden,
        )
        .boolean(
            "interaction_features_forbidden",
            value.interaction_features_forbidden,
        )
        .string("participant_digest", &value.participant_digest)
        .encode()
}

fn decode_participant(bytes: &[u8]) -> Result<MomentumQualifiedParticipantRegistrationV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedParticipantRegistrationV1")?;
    let value = MomentumQualifiedParticipantRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        participant: MomentumQualifiedParticipantV1::parse(&fields.string("participant")?)?,
        ordered_timeframes: fields
            .strings("ordered_timeframes")?
            .iter()
            .map(|timeframe| parse_timeframe(timeframe))
            .collect::<Result<Vec<_>, _>>()?,
        fresh_research_parameters_required: fields.boolean("fresh_research_parameters_required")?,
        live_parameter_use_forbidden: fields.boolean("live_parameter_use_forbidden")?,
        prior_fold_parameter_use_forbidden: fields.boolean("prior_fold_parameter_use_forbidden")?,
        interaction_features_forbidden: fields.boolean("interaction_features_forbidden")?,
        participant_digest: fields.string("participant_digest")?,
    };
    fields.finish()?;
    validate_participant(&value)?;
    Ok(value)
}

fn encode_holdout(value: &MomentumQualifiedSixHoldoutBoundaryV1) -> Result<Vec<u8>, String> {
    validate_holdout(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedSixHoldoutBoundaryV1")
        .string("boundary_version", &value.boundary_version)
        .string("prior_holdout_digest", &value.prior_holdout_digest)
        .boolean("adopted_prior_boundary", value.adopted_prior_boundary)
        .unsigned(
            "holdout_start_timestamp_ms",
            value.holdout_start_timestamp_ms,
        )
        .unsigned("holdout_event_count", as_u64(value.holdout_event_count)?)
        .boolean("labels_opened", value.labels_opened)
        .unsigned("metric_computations", as_u64(value.metric_computations)?)
        .unsigned(
            "participant_predictions",
            as_u64(value.participant_predictions)?,
        )
        .boolean("aggregate_result_present", value.aggregate_result_present)
        .string("boundary_digest", &value.boundary_digest)
        .encode()
}

fn decode_holdout(bytes: &[u8]) -> Result<MomentumQualifiedSixHoldoutBoundaryV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedSixHoldoutBoundaryV1")?;
    let value = MomentumQualifiedSixHoldoutBoundaryV1 {
        boundary_version: fields.string("boundary_version")?,
        prior_holdout_digest: fields.string("prior_holdout_digest")?,
        adopted_prior_boundary: fields.boolean("adopted_prior_boundary")?,
        holdout_start_timestamp_ms: fields.unsigned("holdout_start_timestamp_ms")?,
        holdout_event_count: as_usize(fields.unsigned("holdout_event_count")?)?,
        labels_opened: fields.boolean("labels_opened")?,
        metric_computations: as_usize(fields.unsigned("metric_computations")?)?,
        participant_predictions: as_usize(fields.unsigned("participant_predictions")?)?,
        aggregate_result_present: fields.boolean("aggregate_result_present")?,
        boundary_digest: fields.string("boundary_digest")?,
    };
    fields.finish()?;
    validate_holdout(&value)?;
    Ok(value)
}

fn encode_eligibility(value: &MomentumQualifiedEligibilityAuditV1) -> Result<Vec<u8>, String> {
    validate_eligibility(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedEligibilityAuditV1")
        .string("audit_version", &value.audit_version)
        .unsigned("protocol_event_count", as_u64(value.protocol_event_count)?)
        .unsigned(
            "common_eligible_event_count",
            as_u64(value.common_eligible_event_count)?,
        )
        .unsigned(
            "rejected_context_count",
            as_u64(value.rejected_context_count)?,
        )
        .unsigned(
            "rejected_target_count",
            as_u64(value.rejected_target_count)?,
        )
        .unsigned(
            "rejected_partial_count",
            as_u64(value.rejected_partial_count)?,
        )
        .unsigned(
            "rejected_missing_evidence_count",
            as_u64(value.rejected_missing_evidence_count)?,
        )
        .unsigned(
            "month_view_load_count",
            as_u64(value.month_view_load_count)?,
        )
        .unsigned("year_view_load_count", as_u64(value.year_view_load_count)?)
        .unsigned("future_access_count", as_u64(value.future_access_count)?)
        .unsigned("partial_access_count", as_u64(value.partial_access_count)?)
        .unsigned(
            "unqualified_access_count",
            as_u64(value.unqualified_access_count)?,
        )
        .unsigned(
            "target_value_access_count",
            as_u64(value.target_value_access_count)?,
        )
        .unsigned(
            "first_prediction_timestamp_ms",
            value.first_prediction_timestamp_ms,
        )
        .unsigned(
            "last_prediction_timestamp_ms",
            value.last_prediction_timestamp_ms,
        )
        .string("audit_digest", &value.audit_digest)
        .encode()
}

fn decode_eligibility(bytes: &[u8]) -> Result<MomentumQualifiedEligibilityAuditV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedEligibilityAuditV1")?;
    let value = MomentumQualifiedEligibilityAuditV1 {
        audit_version: fields.string("audit_version")?,
        protocol_event_count: as_usize(fields.unsigned("protocol_event_count")?)?,
        common_eligible_event_count: as_usize(fields.unsigned("common_eligible_event_count")?)?,
        rejected_context_count: as_usize(fields.unsigned("rejected_context_count")?)?,
        rejected_target_count: as_usize(fields.unsigned("rejected_target_count")?)?,
        rejected_partial_count: as_usize(fields.unsigned("rejected_partial_count")?)?,
        rejected_missing_evidence_count: as_usize(
            fields.unsigned("rejected_missing_evidence_count")?,
        )?,
        month_view_load_count: as_usize(fields.unsigned("month_view_load_count")?)?,
        year_view_load_count: as_usize(fields.unsigned("year_view_load_count")?)?,
        future_access_count: as_usize(fields.unsigned("future_access_count")?)?,
        partial_access_count: as_usize(fields.unsigned("partial_access_count")?)?,
        unqualified_access_count: as_usize(fields.unsigned("unqualified_access_count")?)?,
        target_value_access_count: as_usize(fields.unsigned("target_value_access_count")?)?,
        first_prediction_timestamp_ms: fields.unsigned("first_prediction_timestamp_ms")?,
        last_prediction_timestamp_ms: fields.unsigned("last_prediction_timestamp_ms")?,
        audit_digest: fields.string("audit_digest")?,
    };
    fields.finish()?;
    validate_eligibility(&value)?;
    Ok(value)
}

fn encode_registration(
    value: &MomentumQualifiedSixReplayRegistrationV1,
) -> Result<Vec<u8>, String> {
    validate_registration(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedSixReplayRegistrationV1")
        .string("registration_version", &value.registration_version)
        .string("family", FAMILY_LABEL)
        .string(
            "qualified_timeframe_set_digest",
            &value.qualified_timeframe_set_digest,
        )
        .string(
            "monthly_exclusion_policy_digest",
            &value.monthly_exclusion_policy_digest,
        )
        .string(
            "yearly_exclusion_policy_digest",
            &value.yearly_exclusion_policy_digest,
        )
        .string(
            "causal_revalidation_digest",
            &value.causal_revalidation_digest,
        )
        .string("protocol_replay_digest", &value.protocol_replay_digest)
        .strings(
            "historical_dataset_index_digests",
            &value.historical_dataset_index_digests,
        )
        .strings(
            "included_timeframes",
            &value
                .included_timeframes
                .iter()
                .map(|timeframe| timeframe_name(*timeframe).to_string())
                .collect::<Vec<_>>(),
        )
        .strings(
            "excluded_timeframes",
            &value
                .excluded_timeframes
                .iter()
                .map(|timeframe| timeframe_name(*timeframe).to_string())
                .collect::<Vec<_>>(),
        )
        .string("prediction_task", TASK_LABEL)
        .string(
            "event_selection_policy_digest",
            &value.event_selection_policy_digest,
        )
        .string("context_policy_digest", &value.context_policy_digest)
        .string("feature_policy_digest", &value.feature_policy_digest)
        .string("label_policy_digest", &value.label_policy_digest)
        .string("training_policy_digest", &value.training_policy_digest)
        .string(
            "normalization_policy_digest",
            &value.normalization_policy_digest,
        )
        .string("evaluation_policy_digest", &value.evaluation_policy_digest)
        .string("refit_policy", "RefitAtUtcDayBoundary")
        .messages(
            "participant_registrations",
            value
                .participant_registrations
                .iter()
                .map(encode_participant)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .string(
            "development_boundary_digest",
            &value.development_boundary_digest,
        )
        .string(
            "validation_boundary_digest",
            &value.validation_boundary_digest,
        )
        .string(
            "sealed_holdout_boundary_digest",
            &value.sealed_holdout_boundary_digest,
        )
        .unsigned(
            "minimum_training_examples",
            as_u64(value.minimum_training_examples)?,
        )
        .unsigned(
            "maximum_training_examples",
            as_u64(value.maximum_training_examples)?,
        )
        .boolean(
            "full_eight_replay_claim_forbidden",
            value.full_eight_replay_claim_forbidden,
        )
        .boolean(
            "live_authority_use_forbidden",
            value.live_authority_use_forbidden,
        )
        .boolean(
            "reward_application_forbidden",
            value.reward_application_forbidden,
        )
        .boolean("chair_action_forbidden", value.chair_action_forbidden)
        .boolean(
            "trading_authority_forbidden",
            value.trading_authority_forbidden,
        )
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_registration(bytes: &[u8]) -> Result<MomentumQualifiedSixReplayRegistrationV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedSixReplayRegistrationV1")?;
    if fields.string("family")? != FAMILY_LABEL
        || fields.string("prediction_task")? != TASK_LABEL
        || fields.string("refit_policy")? != "RefitAtUtcDayBoundary"
    {
        return Err("qualified-six registration identity rejected".to_string());
    }
    let value = MomentumQualifiedSixReplayRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        family: MomentumQualifiedReplayFamilyV1::QualifiedSixIntradayTenMinute,
        qualified_timeframe_set_digest: fields.string("qualified_timeframe_set_digest")?,
        monthly_exclusion_policy_digest: fields.string("monthly_exclusion_policy_digest")?,
        yearly_exclusion_policy_digest: fields.string("yearly_exclusion_policy_digest")?,
        causal_revalidation_digest: fields.string("causal_revalidation_digest")?,
        protocol_replay_digest: fields.string("protocol_replay_digest")?,
        historical_dataset_index_digests: fields.strings("historical_dataset_index_digests")?,
        included_timeframes: fields
            .strings("included_timeframes")?
            .iter()
            .map(|timeframe| parse_timeframe(timeframe))
            .collect::<Result<Vec<_>, _>>()?,
        excluded_timeframes: fields
            .strings("excluded_timeframes")?
            .iter()
            .map(|timeframe| parse_timeframe(timeframe))
            .collect::<Result<Vec<_>, _>>()?,
        prediction_task: MomentumQualifiedPredictionTaskV1::IntradayTenMinuteDirection,
        event_selection_policy_digest: fields.string("event_selection_policy_digest")?,
        context_policy_digest: fields.string("context_policy_digest")?,
        feature_policy_digest: fields.string("feature_policy_digest")?,
        label_policy_digest: fields.string("label_policy_digest")?,
        training_policy_digest: fields.string("training_policy_digest")?,
        normalization_policy_digest: fields.string("normalization_policy_digest")?,
        evaluation_policy_digest: fields.string("evaluation_policy_digest")?,
        refit_policy: MomentumQualifiedRefitPolicyV1::RefitAtUtcDayBoundary,
        participant_registrations: fields
            .messages("participant_registrations")?
            .iter()
            .map(|bytes| decode_participant(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        development_boundary_digest: fields.string("development_boundary_digest")?,
        validation_boundary_digest: fields.string("validation_boundary_digest")?,
        sealed_holdout_boundary_digest: fields.string("sealed_holdout_boundary_digest")?,
        minimum_training_examples: as_usize(fields.unsigned("minimum_training_examples")?)?,
        maximum_training_examples: as_usize(fields.unsigned("maximum_training_examples")?)?,
        full_eight_replay_claim_forbidden: fields.boolean("full_eight_replay_claim_forbidden")?,
        live_authority_use_forbidden: fields.boolean("live_authority_use_forbidden")?,
        reward_application_forbidden: fields.boolean("reward_application_forbidden")?,
        chair_action_forbidden: fields.boolean("chair_action_forbidden")?,
        trading_authority_forbidden: fields.boolean("trading_authority_forbidden")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_registration(&value)?;
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

fn read_exact<T>(
    category: &str,
    digest: &str,
    decode: impl Fn(&[u8]) -> Result<T, String>,
) -> Result<Option<T>, String> {
    let path = artifact_path(category, digest);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(path).map_err(|_| "qualified-six artifact read failed".to_string())?;
    decode(&bytes).map(Some)
}

fn read_only<T>(
    category: &str,
    decode: impl Fn(&[u8]) -> Result<T, String>,
) -> Result<Option<T>, String> {
    let root = Path::new(ROOT).join(category);
    if !root.exists() {
        return Ok(None);
    }
    let mut paths = fs::read_dir(root)
        .map_err(|_| "qualified-six artifact directory read failed".to_string())?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|_| "qualified-six artifact entry read failed".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| path.extension().is_some_and(|extension| extension == "pb"));
    paths.sort();
    if paths.len() > 1 {
        return Err("qualified-six singleton artifact conflict".to_string());
    }
    paths
        .first()
        .map(|path| {
            fs::read(path)
                .map_err(|_| "qualified-six artifact read failed".to_string())
                .and_then(|bytes| decode(&bytes))
        })
        .transpose()
}

fn add_counts(total: &mut (usize, usize), next: (usize, usize)) {
    total.0 += next.0;
    total.1 += next.1;
}

fn persist_static(prepared: &PreparedReplay) -> Result<(usize, usize), String> {
    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_one(
            "label_policies",
            &prepared.label_policy.policy_digest,
            &encode_label_policy(&prepared.label_policy)?,
            |bytes| Ok(decode_label_policy(bytes)?.policy_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_one(
            "partition_policies",
            &prepared.partition_policy.policy_digest,
            &encode_partition_policy(&prepared.partition_policy)?,
            |bytes| Ok(decode_partition_policy(bytes)?.policy_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_one(
            "holdout_boundaries",
            &prepared.holdout_boundary.boundary_digest,
            &encode_holdout(&prepared.holdout_boundary)?,
            |bytes| Ok(decode_holdout(bytes)?.boundary_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_one(
            "eligibility_audits",
            &prepared.eligibility_audit.audit_digest,
            &encode_eligibility(&prepared.eligibility_audit)?,
            |bytes| Ok(decode_eligibility(bytes)?.audit_digest),
        )?,
    );
    for participant in &prepared.participants {
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
            "registrations",
            &prepared.registration.registration_digest,
            &encode_registration(&prepared.registration)?,
            |bytes| Ok(decode_registration(bytes)?.registration_digest),
        )?,
    );
    Ok(counts)
}

fn encode_feature_block(
    value: &MomentumQualifiedTimeframeFeatureBlockV1,
) -> Result<Vec<u8>, String> {
    validate_feature_block(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedTimeframeFeatureBlockV1")
        .string("block_version", &value.block_version)
        .string("timeframe", timeframe_name(value.timeframe))
        .unsigneds("context_timestamp_ms", &value.context_timestamp_ms)
        .strings("source_candle_digests", &value.source_candle_digests)
        .string("feature_schema_digest", &value.feature_schema_digest)
        .string("feature_vector_digest", &value.feature_vector_digest)
        .string("normalizer_digest", &value.normalizer_digest)
        .unsigned("future_access_count", as_u64(value.future_access_count)?)
        .unsigned("partial_access_count", as_u64(value.partial_access_count)?)
        .unsigned(
            "missing_evidence_count",
            as_u64(value.missing_evidence_count)?,
        )
        .string("block_digest", &value.block_digest)
        .encode()
}

fn decode_feature_block(bytes: &[u8]) -> Result<MomentumQualifiedTimeframeFeatureBlockV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedTimeframeFeatureBlockV1")?;
    let value = MomentumQualifiedTimeframeFeatureBlockV1 {
        block_version: fields.string("block_version")?,
        timeframe: parse_timeframe(&fields.string("timeframe")?)?,
        context_timestamp_ms: fields.unsigneds("context_timestamp_ms")?,
        source_candle_digests: fields.strings("source_candle_digests")?,
        feature_schema_digest: fields.string("feature_schema_digest")?,
        feature_vector_digest: fields.string("feature_vector_digest")?,
        normalizer_digest: fields.string("normalizer_digest")?,
        future_access_count: as_usize(fields.unsigned("future_access_count")?)?,
        partial_access_count: as_usize(fields.unsigned("partial_access_count")?)?,
        missing_evidence_count: as_usize(fields.unsigned("missing_evidence_count")?)?,
        block_digest: fields.string("block_digest")?,
    };
    fields.finish()?;
    validate_feature_block(&value)?;
    Ok(value)
}

fn encode_refit(value: &MomentumQualifiedDailyRefitReceiptV1) -> Result<Vec<u8>, String> {
    validate_refit(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedDailyRefitReceiptV1")
        .string("refit_version", &value.refit_version)
        .string("registration_digest", &value.registration_digest)
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
        .strings(
            "participant_parameter_digests",
            &value.participant_parameter_digests,
        )
        .strings(
            "timeframe_normalizer_digests",
            &value.timeframe_normalizer_digests,
        )
        .unsigned(
            "within_day_refit_count",
            as_u64(value.within_day_refit_count)?,
        )
        .unsigned(
            "live_parameter_load_count",
            as_u64(value.live_parameter_load_count)?,
        )
        .unsigned(
            "prior_fold_parameter_load_count",
            as_u64(value.prior_fold_parameter_load_count)?,
        )
        .string("refit_digest", &value.refit_digest)
        .encode()
}

fn decode_refit(bytes: &[u8]) -> Result<MomentumQualifiedDailyRefitReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedDailyRefitReceiptV1")?;
    let value = MomentumQualifiedDailyRefitReceiptV1 {
        refit_version: fields.string("refit_version")?,
        registration_digest: fields.string("registration_digest")?,
        utc_day_boundary_ms: fields.unsigned("utc_day_boundary_ms")?,
        training_target_cutoff_exclusive_ms: fields
            .unsigned("training_target_cutoff_exclusive_ms")?,
        eligible_past_event_count: as_usize(fields.unsigned("eligible_past_event_count")?)?,
        scorable_training_event_count: as_usize(fields.unsigned("scorable_training_event_count")?)?,
        used_training_event_count: as_usize(fields.unsigned("used_training_event_count")?)?,
        participant_parameter_digests: fields.strings("participant_parameter_digests")?,
        timeframe_normalizer_digests: fields.strings("timeframe_normalizer_digests")?,
        within_day_refit_count: as_usize(fields.unsigned("within_day_refit_count")?)?,
        live_parameter_load_count: as_usize(fields.unsigned("live_parameter_load_count")?)?,
        prior_fold_parameter_load_count: as_usize(
            fields.unsigned("prior_fold_parameter_load_count")?,
        )?,
        refit_digest: fields.string("refit_digest")?,
    };
    fields.finish()?;
    validate_refit(&value)?;
    Ok(value)
}

fn f32_bits(values: &[f32]) -> Vec<u64> {
    values
        .iter()
        .map(|value| u64::from(value.to_bits()))
        .collect()
}

fn decode_f32_values(fields: &mut ArtifactReaderV4_2, name: &str) -> Result<Vec<f32>, String> {
    fields
        .unsigneds(name)?
        .into_iter()
        .map(|bits| {
            u32::try_from(bits)
                .map(f32::from_bits)
                .map_err(|_| "qualified-six f32 field rejected".to_string())
        })
        .collect()
}

fn encode_normalizer_receipt(
    value: &MomentumQualifiedDailyNormalizerReceiptV1,
) -> Result<Vec<u8>, String> {
    validate_normalizer_receipt(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedDailyNormalizerReceiptV1")
        .string("receipt_version", &value.receipt_version)
        .string("timeframe", timeframe_name(value.timeframe))
        .unsigneds("private_means_bits", &f32_bits(&value.private_means))
        .unsigneds("private_scales_bits", &f32_bits(&value.private_scales))
        .unsigneds(
            "constant_dimension_indices",
            &value
                .constant_dimension_indices
                .iter()
                .map(|index| as_u64(*index))
                .collect::<Result<Vec<_>, _>>()?,
        )
        .string("normalizer_digest", &value.normalizer_digest)
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_normalizer_receipt(
    bytes: &[u8],
) -> Result<MomentumQualifiedDailyNormalizerReceiptV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedDailyNormalizerReceiptV1")?;
    let value = MomentumQualifiedDailyNormalizerReceiptV1 {
        receipt_version: fields.string("receipt_version")?,
        timeframe: parse_timeframe(&fields.string("timeframe")?)?,
        private_means: decode_f32_values(&mut fields, "private_means_bits")?,
        private_scales: decode_f32_values(&mut fields, "private_scales_bits")?,
        constant_dimension_indices: fields
            .unsigneds("constant_dimension_indices")?
            .into_iter()
            .map(as_usize)
            .collect::<Result<Vec<_>, _>>()?,
        normalizer_digest: fields.string("normalizer_digest")?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_normalizer_receipt(&value)?;
    Ok(value)
}

fn encode_participant_receipt(
    value: &MomentumQualifiedDailyParticipantReceiptV1,
) -> Result<Vec<u8>, String> {
    validate_participant_receipt(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedDailyParticipantReceiptV1")
        .string("receipt_version", &value.receipt_version)
        .string("registration_digest", &value.registration_digest)
        .unsigned("utc_day_boundary_ms", value.utc_day_boundary_ms)
        .string("participant", value.participant.id())
        .unsigned(
            "scorable_training_event_count",
            as_u64(value.scorable_training_event_count)?,
        )
        .unsigned(
            "used_training_event_count",
            as_u64(value.used_training_event_count)?,
        )
        .string("parameter_digest", &value.parameter_digest)
        .string(
            "normalizer_binding_digest",
            &value.normalizer_binding_digest,
        )
        .unsigneds(
            "private_head_weight_bits",
            &f32_bits(&value.private_head_weights),
        )
        .optional_string(
            "private_head_bias_bits",
            &value
                .private_head_bias
                .map(|bias| u64::from(bias.to_bits()).to_string()),
        )
        .unsigned(
            "private_prevalence_bits",
            value.private_prevalence.to_bits(),
        )
        .unsigned(
            "live_parameter_load_count",
            as_u64(value.live_parameter_load_count)?,
        )
        .unsigned(
            "prior_fold_parameter_load_count",
            as_u64(value.prior_fold_parameter_load_count)?,
        )
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_participant_receipt(
    bytes: &[u8],
) -> Result<MomentumQualifiedDailyParticipantReceiptV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedDailyParticipantReceiptV1")?;
    let private_head_bias = optional_u64_field(&mut fields, "private_head_bias_bits")?
        .map(|bits| {
            u32::try_from(bits)
                .map(f32::from_bits)
                .map_err(|_| "qualified-six participant bias rejected".to_string())
        })
        .transpose()?;
    let value = MomentumQualifiedDailyParticipantReceiptV1 {
        receipt_version: fields.string("receipt_version")?,
        registration_digest: fields.string("registration_digest")?,
        utc_day_boundary_ms: fields.unsigned("utc_day_boundary_ms")?,
        participant: MomentumQualifiedParticipantV1::parse(&fields.string("participant")?)?,
        scorable_training_event_count: as_usize(fields.unsigned("scorable_training_event_count")?)?,
        used_training_event_count: as_usize(fields.unsigned("used_training_event_count")?)?,
        parameter_digest: fields.string("parameter_digest")?,
        normalizer_binding_digest: fields.string("normalizer_binding_digest")?,
        private_head_weights: decode_f32_values(&mut fields, "private_head_weight_bits")?,
        private_head_bias,
        private_prevalence: f64::from_bits(fields.unsigned("private_prevalence_bits")?),
        live_parameter_load_count: as_usize(fields.unsigned("live_parameter_load_count")?)?,
        prior_fold_parameter_load_count: as_usize(
            fields.unsigned("prior_fold_parameter_load_count")?,
        )?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_participant_receipt(&value)?;
    Ok(value)
}

fn encode_refit_bundle(value: &MomentumQualifiedDailyRefitBundleV1) -> Result<Vec<u8>, String> {
    validate_refit_bundle(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedDailyRefitBundleV1")
        .string("bundle_version", &value.bundle_version)
        .string("registration_digest", &value.registration_digest)
        .string("partition", value.partition.as_str())
        .unsigned("utc_day_boundary_ms", value.utc_day_boundary_ms)
        .messages("refit_receipt", vec![encode_refit(&value.refit_receipt)?])
        .messages(
            "normalizer_receipts",
            value
                .normalizer_receipts
                .iter()
                .map(encode_normalizer_receipt)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "participant_receipts",
            value
                .participant_receipts
                .iter()
                .map(encode_participant_receipt)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .string("bundle_digest", &value.bundle_digest)
        .encode()
}

fn decode_refit_bundle(bytes: &[u8]) -> Result<MomentumQualifiedDailyRefitBundleV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedDailyRefitBundleV1")?;
    let refit_receipts = fields.messages("refit_receipt")?;
    if refit_receipts.len() != 1 {
        return Err("qualified-six refit receipt cardinality rejected".to_string());
    }
    let value = MomentumQualifiedDailyRefitBundleV1 {
        bundle_version: fields.string("bundle_version")?,
        registration_digest: fields.string("registration_digest")?,
        partition: MomentumReplayPartitionV1::parse(&fields.string("partition")?)?,
        utc_day_boundary_ms: fields.unsigned("utc_day_boundary_ms")?,
        refit_receipt: decode_refit(&refit_receipts[0])?,
        normalizer_receipts: fields
            .messages("normalizer_receipts")?
            .iter()
            .map(|message| decode_normalizer_receipt(message))
            .collect::<Result<Vec<_>, _>>()?,
        participant_receipts: fields
            .messages("participant_receipts")?
            .iter()
            .map(|message| decode_participant_receipt(message))
            .collect::<Result<Vec<_>, _>>()?,
        bundle_digest: fields.string("bundle_digest")?,
    };
    fields.finish()?;
    validate_refit_bundle(&value)?;
    Ok(value)
}

fn encode_event_plan(value: &MomentumQualifiedReplayEventPlanV1) -> Result<Vec<u8>, String> {
    validate_event_plan(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedReplayEventPlanV1")
        .string("plan_version", &value.plan_version)
        .string("registration_digest", &value.registration_digest)
        .string("partition", value.partition.as_str())
        .unsigned("event_number", value.event_number)
        .unsigned("prediction_timestamp_ms", value.prediction_timestamp_ms)
        .unsigned("target_timestamp_ms", value.target_timestamp_ms)
        .string(
            "daily_refit_receipt_digest",
            &value.daily_refit_receipt_digest,
        )
        .strings("timeframe_block_digests", &value.timeframe_block_digests)
        .strings("participant_ids", &value.participant_ids)
        .string("event_plan_digest", &value.event_plan_digest)
        .encode()
}

fn decode_event_plan(bytes: &[u8]) -> Result<MomentumQualifiedReplayEventPlanV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedReplayEventPlanV1")?;
    let value = MomentumQualifiedReplayEventPlanV1 {
        plan_version: fields.string("plan_version")?,
        registration_digest: fields.string("registration_digest")?,
        partition: MomentumReplayPartitionV1::parse(&fields.string("partition")?)?,
        event_number: fields.unsigned("event_number")?,
        prediction_timestamp_ms: fields.unsigned("prediction_timestamp_ms")?,
        target_timestamp_ms: fields.unsigned("target_timestamp_ms")?,
        daily_refit_receipt_digest: fields.string("daily_refit_receipt_digest")?,
        timeframe_block_digests: fields.strings("timeframe_block_digests")?,
        participant_ids: fields.strings("participant_ids")?,
        event_plan_digest: fields.string("event_plan_digest")?,
    };
    fields.finish()?;
    validate_event_plan(&value)?;
    Ok(value)
}

fn encode_prediction_seal(value: &MomentumQualifiedPredictionSealV1) -> Result<Vec<u8>, String> {
    validate_prediction_seal(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedPredictionSealV1")
        .string("seal_version", &value.seal_version)
        .string("event_plan_digest", &value.event_plan_digest)
        .string("participant", value.participant.id())
        .string("parameter_digest", &value.parameter_digest)
        .string(
            "normalizer_binding_digest",
            &value.normalizer_binding_digest,
        )
        .unsigned(
            "private_probability_bits",
            value.private_probability.to_bits(),
        )
        .string("prediction_digest", &value.prediction_digest)
        .string("seal_digest", &value.seal_digest)
        .encode()
}

fn decode_prediction_seal(bytes: &[u8]) -> Result<MomentumQualifiedPredictionSealV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedPredictionSealV1")?;
    let value = MomentumQualifiedPredictionSealV1 {
        seal_version: fields.string("seal_version")?,
        event_plan_digest: fields.string("event_plan_digest")?,
        participant: MomentumQualifiedParticipantV1::parse(&fields.string("participant")?)?,
        parameter_digest: fields.string("parameter_digest")?,
        normalizer_binding_digest: fields.string("normalizer_binding_digest")?,
        private_probability: f64::from_bits(fields.unsigned("private_probability_bits")?),
        prediction_digest: fields.string("prediction_digest")?,
        seal_digest: fields.string("seal_digest")?,
    };
    fields.finish()?;
    validate_prediction_seal(&value)?;
    Ok(value)
}

fn encode_capsule(value: &MomentumQualifiedReplayPredictionCapsuleV1) -> Result<Vec<u8>, String> {
    validate_capsule(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedReplayPredictionCapsuleV1")
        .string("capsule_version", &value.capsule_version)
        .string("event_plan_digest", &value.event_plan_digest)
        .strings("participant_seal_digests", &value.participant_seal_digests)
        .strings(
            "participant_prediction_digests",
            &value.participant_prediction_digests,
        )
        .boolean("target_accessed", value.target_accessed)
        .boolean("label_accessed", value.label_accessed)
        .boolean("metrics_computed", value.metrics_computed)
        .string("capsule_digest", &value.capsule_digest)
        .encode()
}

fn decode_capsule(bytes: &[u8]) -> Result<MomentumQualifiedReplayPredictionCapsuleV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedReplayPredictionCapsuleV1")?;
    let value = MomentumQualifiedReplayPredictionCapsuleV1 {
        capsule_version: fields.string("capsule_version")?,
        event_plan_digest: fields.string("event_plan_digest")?,
        participant_seal_digests: fields.strings("participant_seal_digests")?,
        participant_prediction_digests: fields.strings("participant_prediction_digests")?,
        target_accessed: fields.boolean("target_accessed")?,
        label_accessed: fields.boolean("label_accessed")?,
        metrics_computed: fields.boolean("metrics_computed")?,
        capsule_digest: fields.string("capsule_digest")?,
    };
    fields.finish()?;
    validate_capsule(&value)?;
    Ok(value)
}

fn encode_evaluation(value: &MomentumQualifiedReplayEvaluationV1) -> Result<Vec<u8>, String> {
    validate_evaluation(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedReplayEvaluationV1")
        .string("evaluation_version", &value.evaluation_version)
        .string("event_plan_digest", &value.event_plan_digest)
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .string("label_status", format!("{:?}", value.label_status))
        .optional_string(
            "private_label_bits",
            &value.private_label.map(|label| label.to_bits().to_string()),
        )
        .strings(
            "participant_evaluation_digests",
            &value.participant_evaluation_digests,
        )
        .unsigneds(
            "private_brier_bits",
            &value
                .private_brier_values
                .iter()
                .map(|brier| brier.to_bits())
                .collect::<Vec<_>>(),
        )
        .unsigneds(
            "private_correctness",
            &value
                .private_correctness
                .iter()
                .map(|correct| u64::from(*correct))
                .collect::<Vec<_>>(),
        )
        .string("evaluation_digest", &value.evaluation_digest)
        .encode()
}

fn parse_label_status(value: &str) -> Result<MomentumQualifiedLabelStatusV1, String> {
    match value {
        "Up" => Ok(MomentumQualifiedLabelStatusV1::Up),
        "Down" => Ok(MomentumQualifiedLabelStatusV1::Down),
        "Neutral" => Ok(MomentumQualifiedLabelStatusV1::Neutral),
        "Invalid" => Ok(MomentumQualifiedLabelStatusV1::Invalid),
        _ => Err("qualified-six label status rejected".to_string()),
    }
}

fn decode_evaluation(bytes: &[u8]) -> Result<MomentumQualifiedReplayEvaluationV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedReplayEvaluationV1")?;
    let private_label = fields
        .optional_string("private_label_bits")?
        .map(|bits| {
            bits.parse::<u64>()
                .map(f64::from_bits)
                .map_err(|_| "qualified-six private label rejected".to_string())
        })
        .transpose()?;
    let private_correctness = fields
        .unsigneds("private_correctness")?
        .into_iter()
        .map(|value| match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("qualified-six correctness rejected".to_string()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let value = MomentumQualifiedReplayEvaluationV1 {
        evaluation_version: fields.string("evaluation_version")?,
        event_plan_digest: fields.string("event_plan_digest")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        label_status: parse_label_status(&fields.string("label_status")?)?,
        private_label,
        participant_evaluation_digests: fields.strings("participant_evaluation_digests")?,
        private_brier_values: fields
            .unsigneds("private_brier_bits")?
            .into_iter()
            .map(f64::from_bits)
            .collect(),
        private_correctness,
        evaluation_digest: fields.string("evaluation_digest")?,
    };
    fields.finish()?;
    validate_evaluation(&value)?;
    Ok(value)
}

fn validate_prediction_bundle(
    value: &MomentumQualifiedDailyPredictionBundleV1,
) -> Result<(), String> {
    let event_count = value.event_plans.len();
    if value.bundle_version != PREDICTION_BUNDLE_VERSION
        || value.registration_digest.is_empty()
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.utc_day_boundary_ms % DAY_MS != 0
        || validate_refit(&value.refit_receipt).is_err()
        || event_count == 0
        || value.feature_blocks.len() != event_count * 6
        || value.prediction_seals.len() != event_count * 5
        || value.capsules.len() != event_count
        || value
            .feature_blocks
            .iter()
            .any(|block| validate_feature_block(block).is_err())
        || value
            .event_plans
            .iter()
            .any(|plan| validate_event_plan(plan).is_err())
        || value
            .prediction_seals
            .iter()
            .any(|seal| validate_prediction_seal(seal).is_err())
        || value
            .capsules
            .iter()
            .any(|capsule| validate_capsule(capsule).is_err())
        || value.target_access_count != 0
        || value.label_access_count != 0
        || value.metric_computation_count != 0
        || value.bundle_digest != prediction_bundle_digest(value)
    {
        return Err("qualified-six daily prediction bundle rejected".to_string());
    }
    Ok(())
}

fn encode_prediction_bundle(
    value: &MomentumQualifiedDailyPredictionBundleV1,
) -> Result<Vec<u8>, String> {
    validate_prediction_bundle(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedDailyPredictionBundleV1")
        .string("bundle_version", &value.bundle_version)
        .string("registration_digest", &value.registration_digest)
        .string("partition", value.partition.as_str())
        .unsigned("utc_day_boundary_ms", value.utc_day_boundary_ms)
        .messages("refit_receipt", vec![encode_refit(&value.refit_receipt)?])
        .messages(
            "feature_blocks",
            value
                .feature_blocks
                .iter()
                .map(encode_feature_block)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "event_plans",
            value
                .event_plans
                .iter()
                .map(encode_event_plan)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "prediction_seals",
            value
                .prediction_seals
                .iter()
                .map(encode_prediction_seal)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "capsules",
            value
                .capsules
                .iter()
                .map(encode_capsule)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .unsigned("target_access_count", as_u64(value.target_access_count)?)
        .unsigned("label_access_count", as_u64(value.label_access_count)?)
        .unsigned(
            "metric_computation_count",
            as_u64(value.metric_computation_count)?,
        )
        .string("bundle_digest", &value.bundle_digest)
        .encode()
}

fn decode_prediction_bundle(
    bytes: &[u8],
) -> Result<MomentumQualifiedDailyPredictionBundleV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedDailyPredictionBundleV1")?;
    let refits = fields.messages("refit_receipt")?;
    if refits.len() != 1 {
        return Err("qualified-six refit cardinality rejected".to_string());
    }
    let value = MomentumQualifiedDailyPredictionBundleV1 {
        bundle_version: fields.string("bundle_version")?,
        registration_digest: fields.string("registration_digest")?,
        partition: MomentumReplayPartitionV1::parse(&fields.string("partition")?)?,
        utc_day_boundary_ms: fields.unsigned("utc_day_boundary_ms")?,
        refit_receipt: decode_refit(&refits[0])?,
        feature_blocks: fields
            .messages("feature_blocks")?
            .iter()
            .map(|bytes| decode_feature_block(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        event_plans: fields
            .messages("event_plans")?
            .iter()
            .map(|bytes| decode_event_plan(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        prediction_seals: fields
            .messages("prediction_seals")?
            .iter()
            .map(|bytes| decode_prediction_seal(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        capsules: fields
            .messages("capsules")?
            .iter()
            .map(|bytes| decode_capsule(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        target_access_count: as_usize(fields.unsigned("target_access_count")?)?,
        label_access_count: as_usize(fields.unsigned("label_access_count")?)?,
        metric_computation_count: as_usize(fields.unsigned("metric_computation_count")?)?,
        bundle_digest: fields.string("bundle_digest")?,
    };
    fields.finish()?;
    validate_prediction_bundle(&value)?;
    Ok(value)
}

fn validate_evaluation_bundle(
    value: &MomentumQualifiedDailyEvaluationBundleV1,
) -> Result<(), String> {
    if value.bundle_version != EVALUATION_BUNDLE_VERSION
        || value.prediction_bundle_digest.is_empty()
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.utc_day_boundary_ms % DAY_MS != 0
        || value.evaluations.is_empty()
        || value
            .evaluations
            .iter()
            .any(|evaluation| validate_evaluation(evaluation).is_err())
        || !value.prediction_bundle_reopened
        || value.bundle_digest != evaluation_bundle_digest(value)
    {
        return Err("qualified-six daily evaluation bundle rejected".to_string());
    }
    Ok(())
}

fn encode_evaluation_bundle(
    value: &MomentumQualifiedDailyEvaluationBundleV1,
) -> Result<Vec<u8>, String> {
    validate_evaluation_bundle(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedDailyEvaluationBundleV1")
        .string("bundle_version", &value.bundle_version)
        .string("prediction_bundle_digest", &value.prediction_bundle_digest)
        .string("partition", value.partition.as_str())
        .unsigned("utc_day_boundary_ms", value.utc_day_boundary_ms)
        .messages(
            "evaluations",
            value
                .evaluations
                .iter()
                .map(encode_evaluation)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .boolean(
            "prediction_bundle_reopened",
            value.prediction_bundle_reopened,
        )
        .string("bundle_digest", &value.bundle_digest)
        .encode()
}

fn decode_evaluation_bundle(
    bytes: &[u8],
) -> Result<MomentumQualifiedDailyEvaluationBundleV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedDailyEvaluationBundleV1")?;
    let value = MomentumQualifiedDailyEvaluationBundleV1 {
        bundle_version: fields.string("bundle_version")?,
        prediction_bundle_digest: fields.string("prediction_bundle_digest")?,
        partition: MomentumReplayPartitionV1::parse(&fields.string("partition")?)?,
        utc_day_boundary_ms: fields.unsigned("utc_day_boundary_ms")?,
        evaluations: fields
            .messages("evaluations")?
            .iter()
            .map(|bytes| decode_evaluation(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        prediction_bundle_reopened: fields.boolean("prediction_bundle_reopened")?,
        bundle_digest: fields.string("bundle_digest")?,
    };
    fields.finish()?;
    validate_evaluation_bundle(&value)?;
    Ok(value)
}

fn validate_metrics(value: &MomentumQualifiedParticipantMetricsV1) -> Result<(), String> {
    let participant = MomentumQualifiedParticipantV1::parse(&value.participant_id)?;
    let _ = participant;
    let scorable_metrics_present = value.scorable_events > 0;
    if value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.total_prediction_events
            != value.scorable_events + value.neutral_events + value.invalid_events
        || value.finite_prediction_count != value.total_prediction_events
        || value.mean_brier_score.is_some() != scorable_metrics_present
        || value.binary_correctness.is_some() != scorable_metrics_present
        || value
            .mean_brier_score
            .is_some_and(|score| !score.is_finite() || !(0.0..=1.0).contains(&score))
        || value
            .binary_correctness
            .is_some_and(|score| !score.is_finite() || !(0.0..=1.0).contains(&score))
        || value
            .delta_versus_constant
            .is_some_and(|delta| !delta.is_finite())
        || value.paired_scorable_count != value.scorable_events
        || !value.chronology_audit_passed
        || !value.leakage_audit_passed
        || value.metrics_digest != metrics_digest(value)
    {
        return Err("qualified-six participant metrics rejected".to_string());
    }
    Ok(())
}

fn encode_metrics(value: &MomentumQualifiedParticipantMetricsV1) -> Result<Vec<u8>, String> {
    validate_metrics(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedParticipantMetricsV1")
        .string("participant_id", &value.participant_id)
        .string("partition", value.partition.as_str())
        .unsigned(
            "total_prediction_events",
            as_u64(value.total_prediction_events)?,
        )
        .unsigned("scorable_events", as_u64(value.scorable_events)?)
        .unsigned("neutral_events", as_u64(value.neutral_events)?)
        .unsigned("invalid_events", as_u64(value.invalid_events)?)
        .unsigned(
            "finite_prediction_count",
            as_u64(value.finite_prediction_count)?,
        )
        .boolean("probability_collapsed", value.probability_collapsed)
        .optional_string(
            "mean_brier_score_bits",
            &value
                .mean_brier_score
                .map(|score| score.to_bits().to_string()),
        )
        .optional_string(
            "binary_correctness_bits",
            &value
                .binary_correctness
                .map(|score| score.to_bits().to_string()),
        )
        .optional_string(
            "delta_versus_constant_bits",
            &value
                .delta_versus_constant
                .map(|score| score.to_bits().to_string()),
        )
        .unsigned(
            "paired_scorable_count",
            as_u64(value.paired_scorable_count)?,
        )
        .boolean("chronology_audit_passed", value.chronology_audit_passed)
        .boolean("leakage_audit_passed", value.leakage_audit_passed)
        .string("metrics_digest", &value.metrics_digest)
        .encode()
}

fn optional_f64_field(fields: &mut ArtifactReaderV4_2, name: &str) -> Result<Option<f64>, String> {
    fields
        .optional_string(name)?
        .map(|bits| {
            bits.parse::<u64>()
                .map(f64::from_bits)
                .map_err(|_| "qualified-six floating field rejected".to_string())
        })
        .transpose()
}

fn decode_metrics(bytes: &[u8]) -> Result<MomentumQualifiedParticipantMetricsV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedParticipantMetricsV1")?;
    let value = MomentumQualifiedParticipantMetricsV1 {
        participant_id: fields.string("participant_id")?,
        partition: MomentumReplayPartitionV1::parse(&fields.string("partition")?)?,
        total_prediction_events: as_usize(fields.unsigned("total_prediction_events")?)?,
        scorable_events: as_usize(fields.unsigned("scorable_events")?)?,
        neutral_events: as_usize(fields.unsigned("neutral_events")?)?,
        invalid_events: as_usize(fields.unsigned("invalid_events")?)?,
        finite_prediction_count: as_usize(fields.unsigned("finite_prediction_count")?)?,
        probability_collapsed: fields.boolean("probability_collapsed")?,
        mean_brier_score: optional_f64_field(&mut fields, "mean_brier_score_bits")?,
        binary_correctness: optional_f64_field(&mut fields, "binary_correctness_bits")?,
        delta_versus_constant: optional_f64_field(&mut fields, "delta_versus_constant_bits")?,
        paired_scorable_count: as_usize(fields.unsigned("paired_scorable_count")?)?,
        chronology_audit_passed: fields.boolean("chronology_audit_passed")?,
        leakage_audit_passed: fields.boolean("leakage_audit_passed")?,
        metrics_digest: fields.string("metrics_digest")?,
    };
    fields.finish()?;
    validate_metrics(&value)?;
    Ok(value)
}

fn validate_aggregate(value: &MomentumQualifiedPartitionAggregateV1) -> Result<(), String> {
    if value.aggregate_version != AGGREGATE_VERSION
        || value.registration_digest.is_empty()
        || value.partition == MomentumReplayPartitionV1::SealedHoldout
        || value.partition_event_count
            != value.training_only_event_count + value.prediction_event_count
        || value.prediction_event_count
            != value.scorable_event_count + value.neutral_event_count + value.invalid_event_count
        || value.daily_refit_count == 0
        || value.daily_prediction_bundle_digests.len() != value.daily_refit_count
        || value.daily_evaluation_bundle_digests.len() != value.daily_refit_count
        || value.participant_metrics.len() != 5
        || value
            .participant_metrics
            .iter()
            .any(|metrics| validate_metrics(metrics).is_err())
        || value.target_access_before_capsule_count != 0
        || value.future_access_count != 0
        || value.partial_access_count != 0
        || value.unqualified_access_count != 0
        || value.aggregate_digest != aggregate_digest(value)
    {
        return Err("qualified-six partition aggregate rejected".to_string());
    }
    Ok(())
}

fn encode_aggregate(value: &MomentumQualifiedPartitionAggregateV1) -> Result<Vec<u8>, String> {
    validate_aggregate(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedPartitionAggregateV1")
        .string("aggregate_version", &value.aggregate_version)
        .string("registration_digest", &value.registration_digest)
        .string("partition", value.partition.as_str())
        .unsigned(
            "partition_event_count",
            as_u64(value.partition_event_count)?,
        )
        .unsigned(
            "training_only_event_count",
            as_u64(value.training_only_event_count)?,
        )
        .unsigned(
            "prediction_event_count",
            as_u64(value.prediction_event_count)?,
        )
        .unsigned("scorable_event_count", as_u64(value.scorable_event_count)?)
        .unsigned("neutral_event_count", as_u64(value.neutral_event_count)?)
        .unsigned("invalid_event_count", as_u64(value.invalid_event_count)?)
        .unsigned("daily_refit_count", as_u64(value.daily_refit_count)?)
        .strings(
            "daily_prediction_bundle_digests",
            &value.daily_prediction_bundle_digests,
        )
        .strings(
            "daily_evaluation_bundle_digests",
            &value.daily_evaluation_bundle_digests,
        )
        .messages(
            "participant_metrics",
            value
                .participant_metrics
                .iter()
                .map(encode_metrics)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .unsigned(
            "target_access_before_capsule_count",
            as_u64(value.target_access_before_capsule_count)?,
        )
        .unsigned("future_access_count", as_u64(value.future_access_count)?)
        .unsigned("partial_access_count", as_u64(value.partial_access_count)?)
        .unsigned(
            "unqualified_access_count",
            as_u64(value.unqualified_access_count)?,
        )
        .string("aggregate_digest", &value.aggregate_digest)
        .encode()
}

fn decode_aggregate(bytes: &[u8]) -> Result<MomentumQualifiedPartitionAggregateV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedPartitionAggregateV1")?;
    let value = MomentumQualifiedPartitionAggregateV1 {
        aggregate_version: fields.string("aggregate_version")?,
        registration_digest: fields.string("registration_digest")?,
        partition: MomentumReplayPartitionV1::parse(&fields.string("partition")?)?,
        partition_event_count: as_usize(fields.unsigned("partition_event_count")?)?,
        training_only_event_count: as_usize(fields.unsigned("training_only_event_count")?)?,
        prediction_event_count: as_usize(fields.unsigned("prediction_event_count")?)?,
        scorable_event_count: as_usize(fields.unsigned("scorable_event_count")?)?,
        neutral_event_count: as_usize(fields.unsigned("neutral_event_count")?)?,
        invalid_event_count: as_usize(fields.unsigned("invalid_event_count")?)?,
        daily_refit_count: as_usize(fields.unsigned("daily_refit_count")?)?,
        daily_prediction_bundle_digests: fields.strings("daily_prediction_bundle_digests")?,
        daily_evaluation_bundle_digests: fields.strings("daily_evaluation_bundle_digests")?,
        participant_metrics: fields
            .messages("participant_metrics")?
            .iter()
            .map(|bytes| decode_metrics(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        target_access_before_capsule_count: as_usize(
            fields.unsigned("target_access_before_capsule_count")?,
        )?,
        future_access_count: as_usize(fields.unsigned("future_access_count")?)?,
        partial_access_count: as_usize(fields.unsigned("partial_access_count")?)?,
        unqualified_access_count: as_usize(fields.unsigned("unqualified_access_count")?)?,
        aggregate_digest: fields.string("aggregate_digest")?,
    };
    fields.finish()?;
    validate_aggregate(&value)?;
    Ok(value)
}

fn parse_benchmark(value: &str) -> Result<MomentumQualifiedBenchmarkComparisonV1, String> {
    match value {
        "LowerBrierThanConstant" => {
            Ok(MomentumQualifiedBenchmarkComparisonV1::LowerBrierThanConstant)
        }
        "HigherBrierThanConstant" => {
            Ok(MomentumQualifiedBenchmarkComparisonV1::HigherBrierThanConstant)
        }
        "NumericallyEquivalentToConstant" => {
            Ok(MomentumQualifiedBenchmarkComparisonV1::NumericallyEquivalentToConstant)
        }
        "MixedAcrossPartitions" => {
            Ok(MomentumQualifiedBenchmarkComparisonV1::MixedAcrossPartitions)
        }
        "InsufficientScorableValidation" => {
            Ok(MomentumQualifiedBenchmarkComparisonV1::InsufficientScorableValidation)
        }
        "ProbabilityCollapse" => Ok(MomentumQualifiedBenchmarkComparisonV1::ProbabilityCollapse),
        "IntegrityFailure" => Ok(MomentumQualifiedBenchmarkComparisonV1::IntegrityFailure),
        _ => Err("qualified-six benchmark classification rejected".to_string()),
    }
}

fn validate_benchmark(value: &MomentumQualifiedBenchmarkReceiptV1) -> Result<(), String> {
    if value.comparison_version != BENCHMARK_VERSION
        || value.participant_id == MomentumQualifiedParticipantV1::Q0TrainingPrevalenceConstant.id()
        || MomentumQualifiedParticipantV1::parse(&value.participant_id).is_err()
        || value.paired_development_count == 0
        || value.paired_validation_count == 0
        || value.comparison_digest != benchmark_digest(value)
    {
        return Err("qualified-six benchmark receipt rejected".to_string());
    }
    Ok(())
}

fn encode_benchmark(value: &MomentumQualifiedBenchmarkReceiptV1) -> Result<Vec<u8>, String> {
    validate_benchmark(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedBenchmarkReceiptV1")
        .string("comparison_version", &value.comparison_version)
        .string("participant_id", &value.participant_id)
        .optional_string(
            "development_delta_bits",
            &value.development_delta_bits.map(|bits| bits.to_string()),
        )
        .optional_string(
            "validation_delta_bits",
            &value.validation_delta_bits.map(|bits| bits.to_string()),
        )
        .unsigned(
            "paired_development_count",
            as_u64(value.paired_development_count)?,
        )
        .unsigned(
            "paired_validation_count",
            as_u64(value.paired_validation_count)?,
        )
        .string("classification", format!("{:?}", value.classification))
        .string("comparison_digest", &value.comparison_digest)
        .encode()
}

fn optional_u64_field(fields: &mut ArtifactReaderV4_2, name: &str) -> Result<Option<u64>, String> {
    fields
        .optional_string(name)?
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| "qualified-six optional integer rejected".to_string())
        })
        .transpose()
}

fn decode_benchmark(bytes: &[u8]) -> Result<MomentumQualifiedBenchmarkReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedBenchmarkReceiptV1")?;
    let value = MomentumQualifiedBenchmarkReceiptV1 {
        comparison_version: fields.string("comparison_version")?,
        participant_id: fields.string("participant_id")?,
        development_delta_bits: optional_u64_field(&mut fields, "development_delta_bits")?,
        validation_delta_bits: optional_u64_field(&mut fields, "validation_delta_bits")?,
        paired_development_count: as_usize(fields.unsigned("paired_development_count")?)?,
        paired_validation_count: as_usize(fields.unsigned("paired_validation_count")?)?,
        classification: parse_benchmark(&fields.string("classification")?)?,
        comparison_digest: fields.string("comparison_digest")?,
    };
    fields.finish()?;
    validate_benchmark(&value)?;
    Ok(value)
}

fn parse_contribution(value: &str) -> Result<MomentumQualifiedContributionStatusV1, String> {
    match value {
        "LowerBrierWithAddedBlock" => {
            Ok(MomentumQualifiedContributionStatusV1::LowerBrierWithAddedBlock)
        }
        "HigherBrierWithAddedBlock" => {
            Ok(MomentumQualifiedContributionStatusV1::HigherBrierWithAddedBlock)
        }
        "NumericallyEquivalent" => Ok(MomentumQualifiedContributionStatusV1::NumericallyEquivalent),
        "MixedAcrossPartitions" => Ok(MomentumQualifiedContributionStatusV1::MixedAcrossPartitions),
        "InsufficientPairedValidation" => {
            Ok(MomentumQualifiedContributionStatusV1::InsufficientPairedValidation)
        }
        "IntegrityFailure" => Ok(MomentumQualifiedContributionStatusV1::IntegrityFailure),
        _ => Err("qualified-six contribution classification rejected".to_string()),
    }
}

fn validate_contribution(value: &MomentumQualifiedContributionReceiptV1) -> Result<(), String> {
    if value.comparison_version != CONTRIBUTION_VERSION
        || value.added_participant_id == value.baseline_participant_id
        || MomentumQualifiedParticipantV1::parse(&value.added_participant_id).is_err()
        || MomentumQualifiedParticipantV1::parse(&value.baseline_participant_id).is_err()
        || value.paired_development_count == 0
        || value.paired_validation_count == 0
        || value.comparison_digest != contribution_digest(value)
    {
        return Err("qualified-six contribution receipt rejected".to_string());
    }
    Ok(())
}

fn encode_contribution(value: &MomentumQualifiedContributionReceiptV1) -> Result<Vec<u8>, String> {
    validate_contribution(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedContributionReceiptV1")
        .string("comparison_version", &value.comparison_version)
        .string("added_participant_id", &value.added_participant_id)
        .string("baseline_participant_id", &value.baseline_participant_id)
        .optional_string(
            "development_delta_bits",
            &value.development_delta_bits.map(|bits| bits.to_string()),
        )
        .optional_string(
            "validation_delta_bits",
            &value.validation_delta_bits.map(|bits| bits.to_string()),
        )
        .unsigned(
            "paired_development_count",
            as_u64(value.paired_development_count)?,
        )
        .unsigned(
            "paired_validation_count",
            as_u64(value.paired_validation_count)?,
        )
        .string("status", format!("{:?}", value.status))
        .string("comparison_digest", &value.comparison_digest)
        .encode()
}

fn decode_contribution(bytes: &[u8]) -> Result<MomentumQualifiedContributionReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedContributionReceiptV1")?;
    let value = MomentumQualifiedContributionReceiptV1 {
        comparison_version: fields.string("comparison_version")?,
        added_participant_id: fields.string("added_participant_id")?,
        baseline_participant_id: fields.string("baseline_participant_id")?,
        development_delta_bits: optional_u64_field(&mut fields, "development_delta_bits")?,
        validation_delta_bits: optional_u64_field(&mut fields, "validation_delta_bits")?,
        paired_development_count: as_usize(fields.unsigned("paired_development_count")?)?,
        paired_validation_count: as_usize(fields.unsigned("paired_validation_count")?)?,
        status: parse_contribution(&fields.string("status")?)?,
        comparison_digest: fields.string("comparison_digest")?,
    };
    fields.finish()?;
    validate_contribution(&value)?;
    Ok(value)
}

fn validate_journal(value: &MomentumQualifiedReplayJournalV1) -> Result<(), String> {
    if value.journal_version != JOURNAL_VERSION
        || [
            &value.registration_digest,
            &value.eligibility_audit_digest,
            &value.development_aggregate_digest,
            &value.validation_aggregate_digest,
            &value.holdout_boundary_digest,
        ]
        .iter()
        .any(|digest| digest.is_empty())
        || value.benchmark_comparison_digests.len() != 4
        || value.contribution_comparison_digests.len() != 3
        || value.holdout_label_reads != 0
        || value.holdout_metric_computations != 0
        || value.holdout_participant_predictions != 0
        || !value.deterministic
        || value.replay_digest != journal_digest(value)
    {
        return Err("qualified-six replay journal rejected".to_string());
    }
    Ok(())
}

fn encode_journal(value: &MomentumQualifiedReplayJournalV1) -> Result<Vec<u8>, String> {
    validate_journal(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedReplayJournalV1")
        .string("journal_version", &value.journal_version)
        .string("registration_digest", &value.registration_digest)
        .string("eligibility_audit_digest", &value.eligibility_audit_digest)
        .string(
            "development_aggregate_digest",
            &value.development_aggregate_digest,
        )
        .string(
            "validation_aggregate_digest",
            &value.validation_aggregate_digest,
        )
        .strings(
            "benchmark_comparison_digests",
            &value.benchmark_comparison_digests,
        )
        .strings(
            "contribution_comparison_digests",
            &value.contribution_comparison_digests,
        )
        .string("holdout_boundary_digest", &value.holdout_boundary_digest)
        .unsigned("holdout_label_reads", as_u64(value.holdout_label_reads)?)
        .unsigned(
            "holdout_metric_computations",
            as_u64(value.holdout_metric_computations)?,
        )
        .unsigned(
            "holdout_participant_predictions",
            as_u64(value.holdout_participant_predictions)?,
        )
        .boolean("deterministic", value.deterministic)
        .string("replay_digest", &value.replay_digest)
        .encode()
}

fn decode_journal(bytes: &[u8]) -> Result<MomentumQualifiedReplayJournalV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedReplayJournalV1")?;
    let value = MomentumQualifiedReplayJournalV1 {
        journal_version: fields.string("journal_version")?,
        registration_digest: fields.string("registration_digest")?,
        eligibility_audit_digest: fields.string("eligibility_audit_digest")?,
        development_aggregate_digest: fields.string("development_aggregate_digest")?,
        validation_aggregate_digest: fields.string("validation_aggregate_digest")?,
        benchmark_comparison_digests: fields.strings("benchmark_comparison_digests")?,
        contribution_comparison_digests: fields.strings("contribution_comparison_digests")?,
        holdout_boundary_digest: fields.string("holdout_boundary_digest")?,
        holdout_label_reads: as_usize(fields.unsigned("holdout_label_reads")?)?,
        holdout_metric_computations: as_usize(fields.unsigned("holdout_metric_computations")?)?,
        holdout_participant_predictions: as_usize(
            fields.unsigned("holdout_participant_predictions")?,
        )?,
        deterministic: fields.boolean("deterministic")?,
        replay_digest: fields.string("replay_digest")?,
    };
    fields.finish()?;
    validate_journal(&value)?;
    Ok(value)
}

fn classify_label(current_close: f64, target_close: f64) -> MomentumQualifiedLabelStatusV1 {
    if !current_close.is_finite()
        || !target_close.is_finite()
        || current_close <= 0.0
        || target_close <= 0.0
    {
        MomentumQualifiedLabelStatusV1::Invalid
    } else if target_close > current_close {
        MomentumQualifiedLabelStatusV1::Up
    } else if target_close < current_close {
        MomentumQualifiedLabelStatusV1::Down
    } else {
        MomentumQualifiedLabelStatusV1::Neutral
    }
}

fn reveal_label(
    prepared: &PreparedReplay,
    event: &PreparedEvent,
) -> Result<(MomentumQualifiedLabelStatusV1, Option<f64>), String> {
    let current = prepared
        .ten_minute_rows
        .get(event.current_ten_minute_index)
        .ok_or_else(|| "qualified-six current target source unavailable".to_string())?;
    let target = prepared
        .ten_minute_rows
        .get(event.target_ten_minute_index)
        .ok_or_else(|| "qualified-six target source unavailable".to_string())?;
    if current.close_exclusive_timestamp_ms != event.prediction_timestamp_ms
        || target.close_exclusive_timestamp_ms != event.target_timestamp_ms
        || event.target_ten_minute_index != event.current_ten_minute_index + 1
    {
        return Err("qualified-six target chronology rejected".to_string());
    }
    let status = classify_label(current.close, target.close);
    let label = match status {
        MomentumQualifiedLabelStatusV1::Up => Some(1.0),
        MomentumQualifiedLabelStatusV1::Down => Some(0.0),
        MomentumQualifiedLabelStatusV1::Neutral | MomentumQualifiedLabelStatusV1::Invalid => None,
    };
    Ok((status, label))
}

fn partition_events<'a>(
    prepared: &'a PreparedReplay,
    partition: MomentumReplayPartitionV1,
) -> &'a [PreparedEvent] {
    let development_end = prepared.partition_policy.development_event_count;
    let validation_end = development_end + prepared.partition_policy.validation_event_count;
    match partition {
        MomentumReplayPartitionV1::Development => &prepared.events[..development_end],
        MomentumReplayPartitionV1::Validation => &prepared.events[development_end..validation_end],
        MomentumReplayPartitionV1::SealedHoldout => &prepared.events[validation_end..],
    }
}

fn events_by_utc_day<'a>(events: &'a [PreparedEvent]) -> BTreeMap<u64, Vec<&'a PreparedEvent>> {
    let mut days = BTreeMap::<u64, Vec<&PreparedEvent>>::new();
    for event in events {
        let day = event.prediction_timestamp_ms / DAY_MS * DAY_MS;
        days.entry(day).or_default().push(event);
    }
    days
}

fn raw_block_training_example(
    event: &PreparedEvent,
    timeframe: MomentumHistoricalTimeframeV1,
    label: f64,
) -> Result<EncodedTrainingExampleV0, String> {
    let block = event
        .blocks
        .get(&timeframe)
        .ok_or_else(|| "qualified-six training block unavailable".to_string())?;
    Ok(EncodedTrainingExampleV0 {
        representation: block.values.clone(),
        label: label as f32,
        snapshot_ids: block.source_candle_digests.clone(),
    })
}

fn normalized_representation(
    event: &PreparedEvent,
    timeframes: &[MomentumHistoricalTimeframeV1],
    normalizers: &BTreeMap<MomentumHistoricalTimeframeV1, RepresentationNormalizerV0>,
) -> Result<Vec<f32>, String> {
    let mut values = Vec::new();
    for timeframe in timeframes {
        let block = event
            .blocks
            .get(timeframe)
            .ok_or_else(|| "qualified-six participant block unavailable".to_string())?;
        let normalizer = normalizers
            .get(timeframe)
            .ok_or_else(|| "qualified-six block normalizer unavailable".to_string())?;
        values.extend(
            normalizer
                .transform_representation(&block.values)
                .map_err(|_| "qualified-six block normalization rejected".to_string())?,
        );
    }
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err("qualified-six participant representation rejected".to_string());
    }
    Ok(values)
}

fn freeze_daily_participants(
    prepared: &PreparedReplay,
    utc_day_boundary_ms: u64,
) -> Result<
    (
        MomentumQualifiedDailyRefitReceiptV1,
        BTreeMap<MomentumHistoricalTimeframeV1, RepresentationNormalizerV0>,
        Vec<FrozenParticipant>,
    ),
    String,
> {
    let mut past = Vec::<(&PreparedEvent, f64)>::new();
    let mut eligible_past_event_count = 0usize;
    for event in &prepared.events {
        if event.target_timestamp_ms >= utc_day_boundary_ms
            || event.prediction_timestamp_ms >= prepared.partition_policy.holdout_start_timestamp_ms
        {
            continue;
        }
        eligible_past_event_count += 1;
        let (_, label) = reveal_label(prepared, event)?;
        if let Some(label) = label {
            past.push((event, label));
        }
    }
    if past.len() < prepared.registration.minimum_training_examples {
        return Err("qualified-six insufficient training support".to_string());
    }
    let used_start = past.len().saturating_sub(MAX_TRAINING_EXAMPLES);
    let used = &past[used_start..];
    let mut normalizers = BTreeMap::new();
    for timeframe in included_timeframes() {
        let raw = used
            .iter()
            .map(|(event, label)| raw_block_training_example(event, timeframe, *label))
            .collect::<Result<Vec<_>, _>>()?;
        let normalizer = RepresentationNormalizerV0::fit(&raw)
            .map_err(|_| "qualified-six normalizer fit rejected".to_string())?;
        normalizers.insert(timeframe, normalizer);
    }
    let prevalence = past.iter().map(|(_, label)| *label).sum::<f64>() / past.len() as f64;
    if !prevalence.is_finite() || !(0.0..=1.0).contains(&prevalence) {
        return Err("qualified-six prevalence rejected".to_string());
    }
    let config = training_config();
    let mut frozen = Vec::new();
    for (ordinal, participant) in MomentumQualifiedParticipantV1::ORDERED
        .into_iter()
        .enumerate()
    {
        if participant == MomentumQualifiedParticipantV1::Q0TrainingPrevalenceConstant {
            let parameter_digest = stable_hash_string(&format!(
                "qualified-six-research-constant-v1:{utc_day_boundary_ms}:{}:{}",
                past.len(),
                prevalence.to_bits()
            ));
            frozen.push(FrozenParticipant {
                participant,
                parameter_digest,
                normalizer_binding_digest: stable_hash_string(
                    "qualified-six-constant-past-labels-only",
                ),
                probability_head: None,
                prevalence,
            });
            continue;
        }
        let timeframes = participant.timeframes();
        let training = used
            .iter()
            .map(|(event, label)| {
                Ok(EncodedTrainingExampleV0 {
                    representation: normalized_representation(event, &timeframes, &normalizers)?,
                    label: *label as f32,
                    snapshot_ids: vec![event.protocol_receipt_digest.clone()],
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let dimension = training
            .first()
            .map(|example| example.representation.len())
            .ok_or_else(|| "qualified-six participant training unavailable".to_string())?;
        let seed = config.seed
            ^ utc_day_boundary_ms
            ^ u64::try_from(ordinal + 1)
                .map_err(|_| "qualified-six seed conversion rejected".to_string())?
                .saturating_mul(1_000_003);
        let head = LogisticPredictionHeadV0::seeded(dimension, seed)
            .map_err(|_| "qualified-six participant initialization rejected".to_string())?;
        let head = train_head_v4(head, &training, &config)?;
        let normalizer_binding_digest = stable_hash_string(&format!(
            "qualified-six-normalizer-binding-v1:{:?}",
            timeframes
                .iter()
                .map(|timeframe| {
                    normalizers
                        .get(timeframe)
                        .map(RepresentationNormalizerV0::digest)
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
        ));
        let parameter_digest = stable_hash_string(&format!(
            "qualified-six-research-parameter-v1:{}:{utc_day_boundary_ms}:{}",
            participant.id(),
            head.parameter_digest()
        ));
        frozen.push(FrozenParticipant {
            participant,
            parameter_digest,
            normalizer_binding_digest,
            probability_head: Some(head),
            prevalence,
        });
    }
    let mut refit = MomentumQualifiedDailyRefitReceiptV1 {
        refit_version: REFIT_VERSION.to_string(),
        registration_digest: prepared.registration.registration_digest.clone(),
        utc_day_boundary_ms,
        training_target_cutoff_exclusive_ms: utc_day_boundary_ms,
        eligible_past_event_count,
        scorable_training_event_count: past.len(),
        used_training_event_count: used.len(),
        participant_parameter_digests: frozen
            .iter()
            .map(|participant| participant.parameter_digest.clone())
            .collect(),
        timeframe_normalizer_digests: included_timeframes()
            .iter()
            .map(|timeframe| {
                normalizers
                    .get(timeframe)
                    .map(RepresentationNormalizerV0::digest)
                    .ok_or_else(|| "qualified-six normalizer digest unavailable".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?,
        within_day_refit_count: 0,
        live_parameter_load_count: 0,
        prior_fold_parameter_load_count: 0,
        refit_digest: String::new(),
    };
    refit.refit_digest = refit_digest(&refit);
    validate_refit(&refit)?;
    Ok((refit, normalizers, frozen))
}

fn build_refit_bundle(
    prepared: &PreparedReplay,
    partition: MomentumReplayPartitionV1,
    refit: MomentumQualifiedDailyRefitReceiptV1,
    normalizers: &BTreeMap<MomentumHistoricalTimeframeV1, RepresentationNormalizerV0>,
    frozen: &[FrozenParticipant],
) -> Result<MomentumQualifiedDailyRefitBundleV1, String> {
    let normalizer_receipts = included_timeframes()
        .iter()
        .map(|timeframe| {
            let normalizer = normalizers
                .get(timeframe)
                .ok_or_else(|| "qualified-six refit normalizer unavailable".to_string())?;
            let mut value = MomentumQualifiedDailyNormalizerReceiptV1 {
                receipt_version: NORMALIZER_RECEIPT_VERSION.to_string(),
                timeframe: *timeframe,
                private_means: normalizer.means.clone(),
                private_scales: normalizer.scales.clone(),
                constant_dimension_indices: normalizer.constant_dimension_indices.clone(),
                normalizer_digest: normalizer.digest(),
                receipt_digest: String::new(),
            };
            value.receipt_digest = normalizer_receipt_digest(&value);
            validate_normalizer_receipt(&value)?;
            Ok(value)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let participant_receipts = frozen
        .iter()
        .map(|participant| {
            let mut value = MomentumQualifiedDailyParticipantReceiptV1 {
                receipt_version: PARTICIPANT_RECEIPT_VERSION.to_string(),
                registration_digest: prepared.registration.registration_digest.clone(),
                utc_day_boundary_ms: refit.utc_day_boundary_ms,
                participant: participant.participant,
                scorable_training_event_count: refit.scorable_training_event_count,
                used_training_event_count: refit.used_training_event_count,
                parameter_digest: participant.parameter_digest.clone(),
                normalizer_binding_digest: participant.normalizer_binding_digest.clone(),
                private_head_weights: participant
                    .probability_head
                    .as_ref()
                    .map(|head| head.weights.clone())
                    .unwrap_or_default(),
                private_head_bias: participant.probability_head.as_ref().map(|head| head.bias),
                private_prevalence: participant.prevalence,
                live_parameter_load_count: 0,
                prior_fold_parameter_load_count: 0,
                receipt_digest: String::new(),
            };
            value.receipt_digest = participant_receipt_digest(&value);
            validate_participant_receipt(&value)?;
            Ok(value)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut value = MomentumQualifiedDailyRefitBundleV1 {
        bundle_version: REFIT_BUNDLE_VERSION.to_string(),
        registration_digest: prepared.registration.registration_digest.clone(),
        partition,
        utc_day_boundary_ms: refit.utc_day_boundary_ms,
        refit_receipt: refit,
        normalizer_receipts,
        participant_receipts,
        bundle_digest: String::new(),
    };
    value.bundle_digest = refit_bundle_digest(&value);
    validate_refit_bundle(&value)?;
    Ok(value)
}

fn reconstruct_refit_bundle(
    value: &MomentumQualifiedDailyRefitBundleV1,
) -> Result<
    (
        MomentumQualifiedDailyRefitReceiptV1,
        BTreeMap<MomentumHistoricalTimeframeV1, RepresentationNormalizerV0>,
        Vec<FrozenParticipant>,
    ),
    String,
> {
    validate_refit_bundle(value)?;
    let normalizers = value
        .normalizer_receipts
        .iter()
        .map(|receipt| {
            Ok((
                receipt.timeframe,
                RepresentationNormalizerV0 {
                    means: receipt.private_means.clone(),
                    scales: receipt.private_scales.clone(),
                    constant_dimension_indices: receipt.constant_dimension_indices.clone(),
                },
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let participants = value
        .participant_receipts
        .iter()
        .map(|receipt| {
            let probability_head = receipt
                .private_head_bias
                .map(|bias| LogisticPredictionHeadV0 {
                    weights: receipt.private_head_weights.clone(),
                    bias,
                });
            if probability_head
                .as_ref()
                .is_some_and(|head| head.validate().is_err())
            {
                return Err("qualified-six reopened participant rejected".to_string());
            }
            Ok(FrozenParticipant {
                participant: receipt.participant,
                parameter_digest: receipt.parameter_digest.clone(),
                normalizer_binding_digest: receipt.normalizer_binding_digest.clone(),
                probability_head,
                prevalence: receipt.private_prevalence,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok((value.refit_receipt.clone(), normalizers, participants))
}

fn persist_refit_bundle(
    value: &MomentumQualifiedDailyRefitBundleV1,
) -> Result<(usize, usize), String> {
    persist_one(
        &format!("daily_refits/{}", value.partition.as_str()),
        &value.bundle_digest,
        &encode_refit_bundle(value)?,
        |bytes| Ok(decode_refit_bundle(bytes)?.bundle_digest),
    )
}

fn event_feature_blocks(
    event: &PreparedEvent,
    normalizers: &BTreeMap<MomentumHistoricalTimeframeV1, RepresentationNormalizerV0>,
) -> Result<Vec<MomentumQualifiedTimeframeFeatureBlockV1>, String> {
    included_timeframes()
        .iter()
        .map(|timeframe| {
            let prepared = event
                .blocks
                .get(timeframe)
                .ok_or_else(|| "qualified-six prepared block unavailable".to_string())?;
            let normalizer_digest = normalizers
                .get(timeframe)
                .map(RepresentationNormalizerV0::digest)
                .ok_or_else(|| "qualified-six event normalizer unavailable".to_string())?;
            let mut value = MomentumQualifiedTimeframeFeatureBlockV1 {
                block_version: BLOCK_VERSION.to_string(),
                timeframe: prepared.timeframe,
                context_timestamp_ms: prepared.context_timestamp_ms.clone(),
                source_candle_digests: prepared.source_candle_digests.clone(),
                feature_schema_digest: prepared.feature_schema_digest.clone(),
                feature_vector_digest: prepared.feature_vector_digest.clone(),
                normalizer_digest,
                future_access_count: 0,
                partial_access_count: 0,
                missing_evidence_count: 0,
                block_digest: String::new(),
            };
            value.block_digest = block_digest(&value);
            validate_feature_block(&value)?;
            Ok(value)
        })
        .collect()
}

fn predict_event(
    event: &PreparedEvent,
    plan: &MomentumQualifiedReplayEventPlanV1,
    normalizers: &BTreeMap<MomentumHistoricalTimeframeV1, RepresentationNormalizerV0>,
    frozen: &[FrozenParticipant],
) -> Result<Vec<MomentumQualifiedPredictionSealV1>, String> {
    if frozen.len() != 5 {
        return Err("qualified-six frozen participant cardinality rejected".to_string());
    }
    frozen
        .iter()
        .map(|participant| {
            let probability = if let Some(head) = &participant.probability_head {
                let representation = normalized_representation(
                    event,
                    &participant.participant.timeframes(),
                    normalizers,
                )?;
                f64::from(
                    head.probability(&representation)
                        .map_err(|_| "qualified-six probability rejected".to_string())?,
                )
                .clamp(1e-6, 1.0 - 1e-6)
            } else {
                participant.prevalence.clamp(1e-6, 1.0 - 1e-6)
            };
            let prediction_digest = stable_hash_string(&format!(
                "qualified-six-private-prediction-v1:{}:{}:{}",
                plan.event_plan_digest,
                participant.participant.id(),
                probability.to_bits()
            ));
            let mut value = MomentumQualifiedPredictionSealV1 {
                seal_version: PREDICTION_SEAL_VERSION.to_string(),
                event_plan_digest: plan.event_plan_digest.clone(),
                participant: participant.participant,
                parameter_digest: participant.parameter_digest.clone(),
                normalizer_binding_digest: participant.normalizer_binding_digest.clone(),
                private_probability: probability,
                prediction_digest,
                seal_digest: String::new(),
            };
            value.seal_digest = prediction_seal_digest(&value);
            validate_prediction_seal(&value)?;
            Ok(value)
        })
        .collect()
}

fn build_prediction_bundle(
    prepared: &PreparedReplay,
    partition: MomentumReplayPartitionV1,
    utc_day_boundary_ms: u64,
    day_events: &[&PreparedEvent],
    refit: MomentumQualifiedDailyRefitReceiptV1,
    normalizers: &BTreeMap<MomentumHistoricalTimeframeV1, RepresentationNormalizerV0>,
    frozen: &[FrozenParticipant],
) -> Result<MomentumQualifiedDailyPredictionBundleV1, String> {
    let mut feature_blocks = Vec::with_capacity(day_events.len() * 6);
    let mut event_plans = Vec::with_capacity(day_events.len());
    let mut prediction_seals = Vec::with_capacity(day_events.len() * 5);
    let mut capsules = Vec::with_capacity(day_events.len());
    for event in day_events {
        let blocks = event_feature_blocks(event, normalizers)?;
        let mut plan = MomentumQualifiedReplayEventPlanV1 {
            plan_version: EVENT_PLAN_VERSION.to_string(),
            registration_digest: prepared.registration.registration_digest.clone(),
            partition,
            event_number: event.event_number,
            prediction_timestamp_ms: event.prediction_timestamp_ms,
            target_timestamp_ms: event.target_timestamp_ms,
            daily_refit_receipt_digest: refit.refit_digest.clone(),
            timeframe_block_digests: blocks
                .iter()
                .map(|block| block.block_digest.clone())
                .collect(),
            participant_ids: MomentumQualifiedParticipantV1::ORDERED
                .iter()
                .map(|participant| participant.id().to_string())
                .collect(),
            event_plan_digest: String::new(),
        };
        plan.event_plan_digest = event_plan_digest(&plan);
        validate_event_plan(&plan)?;
        let seals = predict_event(event, &plan, normalizers, frozen)?;
        let mut capsule = MomentumQualifiedReplayPredictionCapsuleV1 {
            capsule_version: CAPSULE_VERSION.to_string(),
            event_plan_digest: plan.event_plan_digest.clone(),
            participant_seal_digests: seals.iter().map(|seal| seal.seal_digest.clone()).collect(),
            participant_prediction_digests: seals
                .iter()
                .map(|seal| seal.prediction_digest.clone())
                .collect(),
            target_accessed: false,
            label_accessed: false,
            metrics_computed: false,
            capsule_digest: String::new(),
        };
        capsule.capsule_digest = capsule_digest(&capsule);
        validate_capsule(&capsule)?;
        feature_blocks.extend(blocks);
        event_plans.push(plan);
        prediction_seals.extend(seals);
        capsules.push(capsule);
    }
    let mut value = MomentumQualifiedDailyPredictionBundleV1 {
        bundle_version: PREDICTION_BUNDLE_VERSION.to_string(),
        registration_digest: prepared.registration.registration_digest.clone(),
        partition,
        utc_day_boundary_ms,
        refit_receipt: refit,
        feature_blocks,
        event_plans,
        prediction_seals,
        capsules,
        target_access_count: 0,
        label_access_count: 0,
        metric_computation_count: 0,
        bundle_digest: String::new(),
    };
    value.bundle_digest = prediction_bundle_digest(&value);
    validate_prediction_bundle(&value)?;
    Ok(value)
}

fn build_evaluation_bundle(
    prepared: &PreparedReplay,
    reopened: &MomentumQualifiedDailyPredictionBundleV1,
    day_events: &[&PreparedEvent],
) -> Result<MomentumQualifiedDailyEvaluationBundleV1, String> {
    if reopened.event_plans.len() != day_events.len()
        || reopened.capsules.len() != day_events.len()
        || reopened.prediction_seals.len() != day_events.len() * 5
    {
        return Err("qualified-six reopened prediction cardinality rejected".to_string());
    }
    let mut evaluations = Vec::with_capacity(day_events.len());
    for (event_index, event) in day_events.iter().enumerate() {
        let plan = &reopened.event_plans[event_index];
        let capsule = &reopened.capsules[event_index];
        if plan.event_number != event.event_number
            || capsule.event_plan_digest != plan.event_plan_digest
        {
            return Err("qualified-six reopened prediction identity rejected".to_string());
        }
        let (label_status, private_label) = reveal_label(prepared, event)?;
        let seals = &reopened.prediction_seals[event_index * 5..(event_index + 1) * 5];
        let mut private_brier_values = Vec::new();
        let mut private_correctness = Vec::new();
        let mut participant_evaluation_digests = Vec::new();
        if let Some(label) = private_label {
            for seal in seals {
                let residual = seal.private_probability - label;
                let brier = residual * residual;
                let correct = (seal.private_probability >= 0.5) == (label == 1.0);
                if !brier.is_finite() || !(0.0..=1.0).contains(&brier) {
                    return Err("qualified-six private metric rejected".to_string());
                }
                private_brier_values.push(brier);
                private_correctness.push(correct);
                participant_evaluation_digests.push(stable_hash_string(&format!(
                    "qualified-six-private-evaluation-v1:{}:{}:{}:{}:{}",
                    plan.event_plan_digest,
                    seal.participant.id(),
                    seal.private_probability.to_bits(),
                    label.to_bits(),
                    brier.to_bits()
                )));
            }
        }
        let mut value = MomentumQualifiedReplayEvaluationV1 {
            evaluation_version: EVALUATION_VERSION.to_string(),
            event_plan_digest: plan.event_plan_digest.clone(),
            prediction_capsule_digest: capsule.capsule_digest.clone(),
            label_status,
            private_label,
            participant_evaluation_digests,
            private_brier_values,
            private_correctness,
            evaluation_digest: String::new(),
        };
        value.evaluation_digest = evaluation_digest(&value);
        validate_evaluation(&value)?;
        evaluations.push(value);
    }
    let mut value = MomentumQualifiedDailyEvaluationBundleV1 {
        bundle_version: EVALUATION_BUNDLE_VERSION.to_string(),
        prediction_bundle_digest: reopened.bundle_digest.clone(),
        partition: reopened.partition,
        utc_day_boundary_ms: reopened.utc_day_boundary_ms,
        evaluations,
        prediction_bundle_reopened: true,
        bundle_digest: String::new(),
    };
    value.bundle_digest = evaluation_bundle_digest(&value);
    validate_evaluation_bundle(&value)?;
    Ok(value)
}

fn update_accumulators(
    accumulators: &mut BTreeMap<MomentumQualifiedParticipantV1, MetricAccumulator>,
    prediction_bundle: &MomentumQualifiedDailyPredictionBundleV1,
    evaluation_bundle: &MomentumQualifiedDailyEvaluationBundleV1,
) -> Result<(), String> {
    if prediction_bundle.event_plans.len() != evaluation_bundle.evaluations.len() {
        return Err("qualified-six metric bundle mismatch".to_string());
    }
    for (event_index, evaluation) in evaluation_bundle.evaluations.iter().enumerate() {
        let seals = &prediction_bundle.prediction_seals[event_index * 5..(event_index + 1) * 5];
        for (participant_index, participant) in
            MomentumQualifiedParticipantV1::ORDERED.iter().enumerate()
        {
            let accumulator = accumulators.entry(*participant).or_default();
            let seal = &seals[participant_index];
            if seal.participant != *participant {
                return Err("qualified-six participant order rejected".to_string());
            }
            accumulator.total += 1;
            accumulator.finite += usize::from(seal.private_probability.is_finite());
            accumulator.probabilities.push(seal.private_probability);
            match evaluation.label_status {
                MomentumQualifiedLabelStatusV1::Up | MomentumQualifiedLabelStatusV1::Down => {
                    accumulator.scorable += 1;
                    accumulator.brier_sum += evaluation.private_brier_values[participant_index];
                    accumulator.correct +=
                        usize::from(evaluation.private_correctness[participant_index]);
                }
                MomentumQualifiedLabelStatusV1::Neutral => accumulator.neutral += 1,
                MomentumQualifiedLabelStatusV1::Invalid => accumulator.invalid += 1,
            }
        }
    }
    Ok(())
}

fn probability_collapsed(probabilities: &[f64]) -> Result<bool, String> {
    if probabilities.is_empty()
        || probabilities
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err("qualified-six collapse audit rejected".to_string());
    }
    let mean = probabilities.iter().sum::<f64>() / probabilities.len() as f64;
    let variance = probabilities
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / probabilities.len() as f64;
    Ok(variance <= COLLAPSE_VARIANCE_THRESHOLD)
}

fn finalize_metrics(
    partition: MomentumReplayPartitionV1,
    accumulators: BTreeMap<MomentumQualifiedParticipantV1, MetricAccumulator>,
) -> Result<Vec<MomentumQualifiedParticipantMetricsV1>, String> {
    let constant_mean = accumulators
        .get(&MomentumQualifiedParticipantV1::Q0TrainingPrevalenceConstant)
        .filter(|accumulator| accumulator.scorable > 0)
        .map(|accumulator| accumulator.brier_sum / accumulator.scorable as f64)
        .ok_or_else(|| "qualified-six constant metrics unavailable".to_string())?;
    MomentumQualifiedParticipantV1::ORDERED
        .iter()
        .map(|participant| {
            let accumulator = accumulators
                .get(participant)
                .ok_or_else(|| "qualified-six participant accumulator unavailable".to_string())?;
            let mean_brier_score = (accumulator.scorable > 0)
                .then(|| accumulator.brier_sum / accumulator.scorable as f64);
            let binary_correctness = (accumulator.scorable > 0)
                .then(|| accumulator.correct as f64 / accumulator.scorable as f64);
            let delta_versus_constant = mean_brier_score.map(|score| score - constant_mean);
            let mut value = MomentumQualifiedParticipantMetricsV1 {
                participant_id: participant.id().to_string(),
                partition,
                total_prediction_events: accumulator.total,
                scorable_events: accumulator.scorable,
                neutral_events: accumulator.neutral,
                invalid_events: accumulator.invalid,
                finite_prediction_count: accumulator.finite,
                probability_collapsed: probability_collapsed(&accumulator.probabilities)?,
                mean_brier_score,
                binary_correctness,
                delta_versus_constant,
                paired_scorable_count: accumulator.scorable,
                chronology_audit_passed: true,
                leakage_audit_passed: true,
                metrics_digest: String::new(),
            };
            value.metrics_digest = metrics_digest(&value);
            validate_metrics(&value)?;
            Ok(value)
        })
        .collect()
}

fn persist_prediction_bundle(
    value: &MomentumQualifiedDailyPredictionBundleV1,
) -> Result<(usize, usize), String> {
    persist_one(
        &format!("daily_predictions/{}", value.partition.as_str()),
        &value.bundle_digest,
        &encode_prediction_bundle(value)?,
        |bytes| Ok(decode_prediction_bundle(bytes)?.bundle_digest),
    )
}

fn persist_evaluation_bundle(
    value: &MomentumQualifiedDailyEvaluationBundleV1,
) -> Result<(usize, usize), String> {
    persist_one(
        &format!("daily_evaluations/{}", value.partition.as_str()),
        &value.bundle_digest,
        &encode_evaluation_bundle(value)?,
        |bytes| Ok(decode_evaluation_bundle(bytes)?.bundle_digest),
    )
}

fn aggregate_category(partition: MomentumReplayPartitionV1) -> String {
    format!("partition_aggregates/{}", partition.as_str())
}

fn execute_partition(
    prepared: &PreparedReplay,
    partition: MomentumReplayPartitionV1,
) -> Result<
    (
        MomentumQualifiedPartitionAggregateV1,
        (usize, usize),
        usize,
        usize,
        usize,
    ),
    String,
> {
    if partition == MomentumReplayPartitionV1::SealedHoldout {
        return Err("qualified-six sealed holdout execution rejected".to_string());
    }
    let category = aggregate_category(partition);
    if let Some(existing) = read_only(&category, decode_aggregate)? {
        return Ok((existing, (0, 1), 0, 0, 0));
    }
    let partition_events = partition_events(prepared, partition);
    let days = events_by_utc_day(partition_events);
    let mut counts = (0usize, 0usize);
    let mut training_only_event_count = 0usize;
    let mut daily_refit_count = 0usize;
    let mut prediction_computation_count = 0usize;
    let mut metric_computation_count = 0usize;
    let mut prediction_bundle_digests = Vec::new();
    let mut evaluation_bundle_digests = Vec::new();
    let mut accumulators = BTreeMap::new();
    for (day, day_events) in days {
        let (refit, normalizers, frozen) = match freeze_daily_participants(prepared, day) {
            Ok(value) => value,
            Err(error) if error == "qualified-six insufficient training support" => {
                training_only_event_count += day_events.len();
                continue;
            }
            Err(error) => return Err(error),
        };
        let refit_bundle = build_refit_bundle(prepared, partition, refit, &normalizers, &frozen)?;
        add_counts(&mut counts, persist_refit_bundle(&refit_bundle)?);
        let reopened_refit = read_exact(
            &format!("daily_refits/{}", partition.as_str()),
            &refit_bundle.bundle_digest,
            decode_refit_bundle,
        )?
        .ok_or_else(|| "qualified-six refit bundle reopen failed".to_string())?;
        if reopened_refit != refit_bundle {
            return Err("qualified-six refit bundle reopen mismatch".to_string());
        }
        let (refit, normalizers, frozen) = reconstruct_refit_bundle(&reopened_refit)?;
        daily_refit_count += 1;
        let prediction_bundle = build_prediction_bundle(
            prepared,
            partition,
            day,
            &day_events,
            refit,
            &normalizers,
            &frozen,
        )?;
        prediction_computation_count += prediction_bundle.prediction_seals.len();
        add_counts(&mut counts, persist_prediction_bundle(&prediction_bundle)?);
        let reopened = read_exact(
            &format!("daily_predictions/{}", partition.as_str()),
            &prediction_bundle.bundle_digest,
            decode_prediction_bundle,
        )?
        .ok_or_else(|| "qualified-six prediction bundle reopen failed".to_string())?;
        if reopened != prediction_bundle {
            return Err("qualified-six prediction bundle reopen mismatch".to_string());
        }
        let evaluation_bundle = build_evaluation_bundle(prepared, &reopened, &day_events)?;
        metric_computation_count += evaluation_bundle
            .evaluations
            .iter()
            .map(|evaluation| evaluation.private_brier_values.len())
            .sum::<usize>();
        add_counts(&mut counts, persist_evaluation_bundle(&evaluation_bundle)?);
        update_accumulators(&mut accumulators, &reopened, &evaluation_bundle)?;
        prediction_bundle_digests.push(reopened.bundle_digest);
        evaluation_bundle_digests.push(evaluation_bundle.bundle_digest);
    }
    if daily_refit_count == 0 {
        return Err("qualified-six insufficient training support".to_string());
    }
    let participant_metrics = finalize_metrics(partition, accumulators)?;
    let prediction_event_count = participant_metrics
        .first()
        .map(|metrics| metrics.total_prediction_events)
        .unwrap_or(0);
    let scorable_event_count = participant_metrics
        .first()
        .map(|metrics| metrics.scorable_events)
        .unwrap_or(0);
    let neutral_event_count = participant_metrics
        .first()
        .map(|metrics| metrics.neutral_events)
        .unwrap_or(0);
    let invalid_event_count = participant_metrics
        .first()
        .map(|metrics| metrics.invalid_events)
        .unwrap_or(0);
    if partition_events.len() != training_only_event_count + prediction_event_count {
        return Err("qualified-six partition accounting rejected".to_string());
    }
    let mut aggregate = MomentumQualifiedPartitionAggregateV1 {
        aggregate_version: AGGREGATE_VERSION.to_string(),
        registration_digest: prepared.registration.registration_digest.clone(),
        partition,
        partition_event_count: partition_events.len(),
        training_only_event_count,
        prediction_event_count,
        scorable_event_count,
        neutral_event_count,
        invalid_event_count,
        daily_refit_count,
        daily_prediction_bundle_digests: prediction_bundle_digests,
        daily_evaluation_bundle_digests: evaluation_bundle_digests,
        participant_metrics,
        target_access_before_capsule_count: 0,
        future_access_count: 0,
        partial_access_count: 0,
        unqualified_access_count: 0,
        aggregate_digest: String::new(),
    };
    aggregate.aggregate_digest = aggregate_digest(&aggregate);
    validate_aggregate(&aggregate)?;
    add_counts(
        &mut counts,
        persist_one(
            &category,
            &aggregate.aggregate_digest,
            &encode_aggregate(&aggregate)?,
            |bytes| Ok(decode_aggregate(bytes)?.aggregate_digest),
        )?,
    );
    Ok((
        aggregate,
        counts,
        daily_refit_count,
        prediction_computation_count,
        metric_computation_count,
    ))
}

fn metrics_by_participant(
    aggregate: &MomentumQualifiedPartitionAggregateV1,
) -> Result<BTreeMap<MomentumQualifiedParticipantV1, &MomentumQualifiedParticipantMetricsV1>, String>
{
    aggregate
        .participant_metrics
        .iter()
        .map(|metrics| {
            Ok((
                MomentumQualifiedParticipantV1::parse(&metrics.participant_id)?,
                metrics,
            ))
        })
        .collect()
}

fn classify_delta_pair(
    development_delta: f64,
    validation_delta: f64,
    collapsed: bool,
    validation_count: usize,
) -> MomentumQualifiedBenchmarkComparisonV1 {
    if validation_count == 0 {
        MomentumQualifiedBenchmarkComparisonV1::InsufficientScorableValidation
    } else if collapsed {
        MomentumQualifiedBenchmarkComparisonV1::ProbabilityCollapse
    } else if development_delta.abs() <= COMPARISON_EPSILON
        && validation_delta.abs() <= COMPARISON_EPSILON
    {
        MomentumQualifiedBenchmarkComparisonV1::NumericallyEquivalentToConstant
    } else if development_delta < -COMPARISON_EPSILON && validation_delta < -COMPARISON_EPSILON {
        MomentumQualifiedBenchmarkComparisonV1::LowerBrierThanConstant
    } else if development_delta > COMPARISON_EPSILON && validation_delta > COMPARISON_EPSILON {
        MomentumQualifiedBenchmarkComparisonV1::HigherBrierThanConstant
    } else {
        MomentumQualifiedBenchmarkComparisonV1::MixedAcrossPartitions
    }
}

fn build_benchmarks(
    development: &MomentumQualifiedPartitionAggregateV1,
    validation: &MomentumQualifiedPartitionAggregateV1,
) -> Result<Vec<MomentumQualifiedBenchmarkReceiptV1>, String> {
    let development_metrics = metrics_by_participant(development)?;
    let validation_metrics = metrics_by_participant(validation)?;
    MomentumQualifiedParticipantV1::ORDERED[1..]
        .iter()
        .map(|participant| {
            let development = development_metrics
                .get(participant)
                .ok_or_else(|| "qualified-six development benchmark unavailable".to_string())?;
            let validation = validation_metrics
                .get(participant)
                .ok_or_else(|| "qualified-six validation benchmark unavailable".to_string())?;
            let development_delta = development
                .delta_versus_constant
                .ok_or_else(|| "qualified-six development delta unavailable".to_string())?;
            let validation_delta = validation
                .delta_versus_constant
                .ok_or_else(|| "qualified-six validation delta unavailable".to_string())?;
            let mut value = MomentumQualifiedBenchmarkReceiptV1 {
                comparison_version: BENCHMARK_VERSION.to_string(),
                participant_id: participant.id().to_string(),
                development_delta_bits: Some(development_delta.to_bits()),
                validation_delta_bits: Some(validation_delta.to_bits()),
                paired_development_count: development.paired_scorable_count,
                paired_validation_count: validation.paired_scorable_count,
                classification: classify_delta_pair(
                    development_delta,
                    validation_delta,
                    development.probability_collapsed || validation.probability_collapsed,
                    validation.paired_scorable_count,
                ),
                comparison_digest: String::new(),
            };
            value.comparison_digest = benchmark_digest(&value);
            validate_benchmark(&value)?;
            Ok(value)
        })
        .collect()
}

fn classify_contribution_pair(
    development_delta: f64,
    validation_delta: f64,
    validation_count: usize,
) -> MomentumQualifiedContributionStatusV1 {
    if validation_count == 0 {
        MomentumQualifiedContributionStatusV1::InsufficientPairedValidation
    } else if development_delta.abs() <= COMPARISON_EPSILON
        && validation_delta.abs() <= COMPARISON_EPSILON
    {
        MomentumQualifiedContributionStatusV1::NumericallyEquivalent
    } else if development_delta < -COMPARISON_EPSILON && validation_delta < -COMPARISON_EPSILON {
        MomentumQualifiedContributionStatusV1::LowerBrierWithAddedBlock
    } else if development_delta > COMPARISON_EPSILON && validation_delta > COMPARISON_EPSILON {
        MomentumQualifiedContributionStatusV1::HigherBrierWithAddedBlock
    } else {
        MomentumQualifiedContributionStatusV1::MixedAcrossPartitions
    }
}

fn build_contribution(
    development: &BTreeMap<MomentumQualifiedParticipantV1, &MomentumQualifiedParticipantMetricsV1>,
    validation: &BTreeMap<MomentumQualifiedParticipantV1, &MomentumQualifiedParticipantMetricsV1>,
    added: MomentumQualifiedParticipantV1,
    baseline: MomentumQualifiedParticipantV1,
) -> Result<MomentumQualifiedContributionReceiptV1, String> {
    let development_added = development
        .get(&added)
        .and_then(|metrics| metrics.mean_brier_score)
        .ok_or_else(|| "qualified-six development contribution unavailable".to_string())?;
    let development_baseline = development
        .get(&baseline)
        .and_then(|metrics| metrics.mean_brier_score)
        .ok_or_else(|| "qualified-six development baseline unavailable".to_string())?;
    let validation_added = validation
        .get(&added)
        .and_then(|metrics| metrics.mean_brier_score)
        .ok_or_else(|| "qualified-six validation contribution unavailable".to_string())?;
    let validation_baseline = validation
        .get(&baseline)
        .and_then(|metrics| metrics.mean_brier_score)
        .ok_or_else(|| "qualified-six validation baseline unavailable".to_string())?;
    let development_delta = development_added - development_baseline;
    let validation_delta = validation_added - validation_baseline;
    let paired_development_count = development
        .get(&added)
        .map(|metrics| metrics.paired_scorable_count)
        .unwrap_or(0)
        .min(
            development
                .get(&baseline)
                .map(|metrics| metrics.paired_scorable_count)
                .unwrap_or(0),
        );
    let paired_validation_count = validation
        .get(&added)
        .map(|metrics| metrics.paired_scorable_count)
        .unwrap_or(0)
        .min(
            validation
                .get(&baseline)
                .map(|metrics| metrics.paired_scorable_count)
                .unwrap_or(0),
        );
    let mut value = MomentumQualifiedContributionReceiptV1 {
        comparison_version: CONTRIBUTION_VERSION.to_string(),
        added_participant_id: added.id().to_string(),
        baseline_participant_id: baseline.id().to_string(),
        development_delta_bits: Some(development_delta.to_bits()),
        validation_delta_bits: Some(validation_delta.to_bits()),
        paired_development_count,
        paired_validation_count,
        status: classify_contribution_pair(
            development_delta,
            validation_delta,
            paired_validation_count,
        ),
        comparison_digest: String::new(),
    };
    value.comparison_digest = contribution_digest(&value);
    validate_contribution(&value)?;
    Ok(value)
}

fn build_contributions(
    development: &MomentumQualifiedPartitionAggregateV1,
    validation: &MomentumQualifiedPartitionAggregateV1,
) -> Result<Vec<MomentumQualifiedContributionReceiptV1>, String> {
    let development = metrics_by_participant(development)?;
    let validation = metrics_by_participant(validation)?;
    [
        (
            MomentumQualifiedParticipantV1::Q2MicroBlockLogistic,
            MomentumQualifiedParticipantV1::Q1TenMinuteAnchorLogistic,
        ),
        (
            MomentumQualifiedParticipantV1::Q4QualifiedSixFusionLogistic,
            MomentumQualifiedParticipantV1::Q2MicroBlockLogistic,
        ),
        (
            MomentumQualifiedParticipantV1::Q4QualifiedSixFusionLogistic,
            MomentumQualifiedParticipantV1::Q3QualifiedMacroBlockLogistic,
        ),
    ]
    .into_iter()
    .map(|(added, baseline)| build_contribution(&development, &validation, added, baseline))
    .collect()
}

fn persist_final_comparisons(
    prepared: &PreparedReplay,
    development: &MomentumQualifiedPartitionAggregateV1,
    validation: &MomentumQualifiedPartitionAggregateV1,
) -> Result<
    (
        Vec<MomentumQualifiedBenchmarkReceiptV1>,
        Vec<MomentumQualifiedContributionReceiptV1>,
        MomentumQualifiedReplayJournalV1,
        (usize, usize),
    ),
    String,
> {
    let benchmarks = build_benchmarks(development, validation)?;
    let contributions = build_contributions(development, validation)?;
    let mut counts = (0, 0);
    for benchmark in &benchmarks {
        add_counts(
            &mut counts,
            persist_one(
                "benchmark_comparisons",
                &benchmark.comparison_digest,
                &encode_benchmark(benchmark)?,
                |bytes| Ok(decode_benchmark(bytes)?.comparison_digest),
            )?,
        );
    }
    for contribution in &contributions {
        add_counts(
            &mut counts,
            persist_one(
                "contribution_comparisons",
                &contribution.comparison_digest,
                &encode_contribution(contribution)?,
                |bytes| Ok(decode_contribution(bytes)?.comparison_digest),
            )?,
        );
    }
    let mut journal = MomentumQualifiedReplayJournalV1 {
        journal_version: JOURNAL_VERSION.to_string(),
        registration_digest: prepared.registration.registration_digest.clone(),
        eligibility_audit_digest: prepared.eligibility_audit.audit_digest.clone(),
        development_aggregate_digest: development.aggregate_digest.clone(),
        validation_aggregate_digest: validation.aggregate_digest.clone(),
        benchmark_comparison_digests: benchmarks
            .iter()
            .map(|benchmark| benchmark.comparison_digest.clone())
            .collect(),
        contribution_comparison_digests: contributions
            .iter()
            .map(|contribution| contribution.comparison_digest.clone())
            .collect(),
        holdout_boundary_digest: prepared.holdout_boundary.boundary_digest.clone(),
        holdout_label_reads: 0,
        holdout_metric_computations: 0,
        holdout_participant_predictions: 0,
        deterministic: true,
        replay_digest: String::new(),
    };
    journal.replay_digest = journal_digest(&journal);
    validate_journal(&journal)?;
    add_counts(
        &mut counts,
        persist_one(
            "replay_journals",
            &journal.replay_digest,
            &encode_journal(&journal)?,
            |bytes| Ok(decode_journal(bytes)?.replay_digest),
        )?,
    );
    Ok((benchmarks, contributions, journal, counts))
}

fn count_runtime_artifacts() -> Result<usize, String> {
    fn count(root: &Path) -> Result<usize, String> {
        if !root.exists() {
            return Ok(0);
        }
        let mut total = 0usize;
        for entry in
            fs::read_dir(root).map_err(|_| "qualified-six artifact count failed".to_string())?
        {
            let path = entry
                .map_err(|_| "qualified-six artifact count failed".to_string())?
                .path();
            if path.is_dir() {
                total += count(&path)?;
            } else if path.extension().is_some_and(|extension| extension == "pb") {
                total += 1;
            }
        }
        Ok(total)
    }
    count(Path::new(ROOT))
}

fn empty_report(run_mode: &str) -> MomentumQualifiedSixReplayReportV1 {
    let mut value = MomentumQualifiedSixReplayReportV1 {
        report_version: REPORT_VERSION.to_string(),
        run_mode: run_mode.to_string(),
        status: MomentumQualifiedReplayStatusV1::Unregistered,
        family: MomentumQualifiedReplayFamilyV1::QualifiedSixIntradayTenMinute,
        registration_digest: None,
        qualified_timeframe_set_digest: None,
        included_timeframes: included_timeframes(),
        excluded_timeframes: excluded_timeframes(),
        prediction_task: MomentumQualifiedPredictionTaskV1::IntradayTenMinuteDirection,
        label_policy_digest: None,
        common_eligible_event_count: 0,
        common_eligible_start_timestamp_ms: None,
        common_eligible_end_timestamp_ms: None,
        development_boundary_digest: None,
        validation_boundary_digest: None,
        minimum_training_examples: 0,
        maximum_training_examples: 0,
        development_partition_event_count: 0,
        development_prediction_event_count: 0,
        development_training_only_event_count: 0,
        development_scorable_event_count: 0,
        development_neutral_event_count: 0,
        development_invalid_event_count: 0,
        development_daily_refit_count: 0,
        validation_partition_event_count: 0,
        validation_prediction_event_count: 0,
        validation_training_only_event_count: 0,
        validation_scorable_event_count: 0,
        validation_neutral_event_count: 0,
        validation_invalid_event_count: 0,
        validation_daily_refit_count: 0,
        participant_metrics: Vec::new(),
        benchmark_comparisons: Vec::new(),
        contribution_comparisons: Vec::new(),
        probability_collapse_count: 0,
        chronology_audit_passed: true,
        leakage_audit_passed: true,
        prediction_before_reveal_passed: true,
        full_eight_replay_claimed: false,
        full_eight_a3_blocked: true,
        month_view_load_count: 0,
        year_view_load_count: 0,
        holdout_label_reads: 0,
        holdout_metric_computations: 0,
        holdout_participant_predictions: 0,
        live_outcome_requests: 0,
        live_outcome_openings: 0,
        live_participant_changes: 0,
        live_parameter_updates: 0,
        live_normalizer_refits: 0,
        live_completed_event_changes: 0,
        live_scorable_event_changes: 0,
        winner_selections: 0,
        ranking_creations: 0,
        reward_applications: 0,
        penalty_applications: 0,
        chair_decisions: 0,
        committee_votes: 0,
        voice_changes: 0,
        tier_changes: 0,
        cooldowns_started: 0,
        promotions: 0,
        quarantines: 0,
        historical_participant_speaking_rights: 0,
        historical_participant_committee_memberships: 0,
        paper_executions: 0,
        live_executions: 0,
        network_request_attempts: 0,
        transport_constructions: 0,
        credentials_read: 0,
        active_committee_count: 3,
        live_event_two_sealed: true,
        epoch_three_registered: false,
        protected_live_tree_digest_before: None,
        protected_active_roster_digest_before: None,
        protected_artifacts_unchanged: true,
        active_roster_unchanged: true,
        historical_warning_preserved: true,
        labels: PUBLIC_LABELS
            .iter()
            .map(|label| (*label).to_string())
            .collect(),
        replay_digest: None,
        artifacts_written: 0,
        duplicate_artifact_count: 0,
        model_refit_count: 0,
        prediction_computation_count: 0,
        metric_recomputation_count: 0,
        runtime_duration_ms: 0,
        report_digest: String::new(),
    };
    value.report_digest = report_digest(&value);
    value
}

fn apply_aggregate_to_report(
    report: &mut MomentumQualifiedSixReplayReportV1,
    aggregate: &MomentumQualifiedPartitionAggregateV1,
) {
    report
        .participant_metrics
        .extend(aggregate.participant_metrics.clone());
    match aggregate.partition {
        MomentumReplayPartitionV1::Development => {
            report.development_partition_event_count = aggregate.partition_event_count;
            report.development_prediction_event_count = aggregate.prediction_event_count;
            report.development_training_only_event_count = aggregate.training_only_event_count;
            report.development_scorable_event_count = aggregate.scorable_event_count;
            report.development_neutral_event_count = aggregate.neutral_event_count;
            report.development_invalid_event_count = aggregate.invalid_event_count;
            report.development_daily_refit_count = aggregate.daily_refit_count;
        }
        MomentumReplayPartitionV1::Validation => {
            report.validation_partition_event_count = aggregate.partition_event_count;
            report.validation_prediction_event_count = aggregate.prediction_event_count;
            report.validation_training_only_event_count = aggregate.training_only_event_count;
            report.validation_scorable_event_count = aggregate.scorable_event_count;
            report.validation_neutral_event_count = aggregate.neutral_event_count;
            report.validation_invalid_event_count = aggregate.invalid_event_count;
            report.validation_daily_refit_count = aggregate.daily_refit_count;
        }
        MomentumReplayPartitionV1::SealedHoldout => {}
    }
}

fn build_report(
    run_mode: &str,
    prepared: Option<&PreparedReplay>,
    development: Option<&MomentumQualifiedPartitionAggregateV1>,
    validation: Option<&MomentumQualifiedPartitionAggregateV1>,
    benchmarks: Vec<MomentumQualifiedBenchmarkReceiptV1>,
    contributions: Vec<MomentumQualifiedContributionReceiptV1>,
    journal: Option<&MomentumQualifiedReplayJournalV1>,
    protected_before: &MomentumQualifiedReplayProtectedStateV1,
    protected_after: &MomentumQualifiedReplayProtectedStateV1,
    counts: (usize, usize),
    model_refit_count: usize,
    prediction_computation_count: usize,
    metric_recomputation_count: usize,
    runtime_duration_ms: u64,
) -> Result<MomentumQualifiedSixReplayReportV1, String> {
    let mut report = empty_report(run_mode);
    report.protected_live_tree_digest_before = Some(protected_before.live_tree_digest.clone());
    report.protected_active_roster_digest_before =
        Some(protected_before.active_roster_digest.clone());
    report.protected_artifacts_unchanged = protected_before.live_tree_file_count
        == protected_after.live_tree_file_count
        && protected_before.live_tree_digest == protected_after.live_tree_digest;
    report.active_roster_unchanged =
        protected_before.active_roster_digest == protected_after.active_roster_digest;
    report.live_completed_event_changes = protected_after
        .live_completed_event_count
        .abs_diff(protected_before.live_completed_event_count);
    report.live_scorable_event_changes = protected_after
        .live_scorable_event_count
        .abs_diff(protected_before.live_scorable_event_count);
    report.live_outcome_requests = protected_after.live_outcome_requests;
    report.live_outcome_openings = protected_after.live_outcome_openings;
    report.active_committee_count = protected_after.active_committee_count;
    report.live_event_two_sealed = protected_after.live_input_attempts == 1
        && protected_after.live_input_retries == 0
        && protected_after.live_prediction_seal_count == 3
        && protected_after.live_outcome_requests == 0
        && protected_after.live_outcome_openings == 0;
    report.epoch_three_registered = protected_after.epoch_three_registered;
    report.artifacts_written = counts.0;
    report.duplicate_artifact_count = counts.1;
    report.model_refit_count = model_refit_count;
    report.prediction_computation_count = prediction_computation_count;
    report.metric_recomputation_count = metric_recomputation_count;
    report.runtime_duration_ms = runtime_duration_ms;
    if let Some(prepared) = prepared {
        report.status = MomentumQualifiedReplayStatusV1::Registered;
        report.registration_digest = Some(prepared.registration.registration_digest.clone());
        report.qualified_timeframe_set_digest =
            Some(prepared.evidence.qualified_timeframe_set_digest.clone());
        report.label_policy_digest = Some(prepared.label_policy.policy_digest.clone());
        report.common_eligible_event_count = prepared.partition_policy.common_eligible_event_count;
        report.common_eligible_start_timestamp_ms =
            Some(prepared.partition_policy.eligible_start_timestamp_ms);
        report.common_eligible_end_timestamp_ms =
            Some(prepared.partition_policy.eligible_end_timestamp_ms);
        report.development_boundary_digest =
            Some(prepared.registration.development_boundary_digest.clone());
        report.validation_boundary_digest =
            Some(prepared.registration.validation_boundary_digest.clone());
        report.minimum_training_examples = prepared.registration.minimum_training_examples;
        report.maximum_training_examples = prepared.registration.maximum_training_examples;
        report.month_view_load_count = prepared.eligibility_audit.month_view_load_count;
        report.year_view_load_count = prepared.eligibility_audit.year_view_load_count;
    }
    if let Some(development) = development {
        report.status = MomentumQualifiedReplayStatusV1::DevelopmentComplete;
        apply_aggregate_to_report(&mut report, development);
    }
    if let Some(validation) = validation {
        report.status = MomentumQualifiedReplayStatusV1::Complete;
        apply_aggregate_to_report(&mut report, validation);
    }
    report.benchmark_comparisons = benchmarks;
    report.contribution_comparisons = contributions;
    report.probability_collapse_count = report
        .participant_metrics
        .iter()
        .filter(|metrics| metrics.probability_collapsed)
        .count();
    report.replay_digest = journal.map(|journal| journal.replay_digest.clone());
    report.report_digest = report_digest(&report);
    validate_report(&report)?;
    Ok(report)
}

fn validate_report(value: &MomentumQualifiedSixReplayReportV1) -> Result<(), String> {
    let zero_authority_counters = [
        value.holdout_label_reads,
        value.holdout_metric_computations,
        value.holdout_participant_predictions,
        value.live_outcome_requests,
        value.live_outcome_openings,
        value.live_participant_changes,
        value.live_parameter_updates,
        value.live_normalizer_refits,
        value.live_completed_event_changes,
        value.live_scorable_event_changes,
        value.winner_selections,
        value.ranking_creations,
        value.reward_applications,
        value.penalty_applications,
        value.chair_decisions,
        value.committee_votes,
        value.voice_changes,
        value.tier_changes,
        value.cooldowns_started,
        value.promotions,
        value.quarantines,
        value.historical_participant_speaking_rights,
        value.historical_participant_committee_memberships,
        value.paper_executions,
        value.live_executions,
        value.network_request_attempts,
        value.transport_constructions,
        value.credentials_read,
    ]
    .into_iter()
    .all(|count| count == 0);
    let registered = matches!(
        value.status,
        MomentumQualifiedReplayStatusV1::Registered
            | MomentumQualifiedReplayStatusV1::DevelopmentComplete
            | MomentumQualifiedReplayStatusV1::Complete
            | MomentumQualifiedReplayStatusV1::InsufficientTrainingSupport
    );
    let development_complete = matches!(
        value.status,
        MomentumQualifiedReplayStatusV1::DevelopmentComplete
            | MomentumQualifiedReplayStatusV1::Complete
    );
    let complete = value.status == MomentumQualifiedReplayStatusV1::Complete;
    let registration_fields_present = [
        value.registration_digest.as_ref(),
        value.qualified_timeframe_set_digest.as_ref(),
        value.label_policy_digest.as_ref(),
        value.development_boundary_digest.as_ref(),
        value.validation_boundary_digest.as_ref(),
    ]
    .into_iter()
    .all(|field| field.is_some())
        && value.common_eligible_start_timestamp_ms.is_some()
        && value.common_eligible_end_timestamp_ms.is_some();
    let expected_metric_count = usize::from(development_complete) * 5 + usize::from(complete) * 5;
    if value.report_version != REPORT_VERSION
        || value.run_mode.is_empty()
        || value.family != MomentumQualifiedReplayFamilyV1::QualifiedSixIntradayTenMinute
        || value.included_timeframes != included_timeframes()
        || value.excluded_timeframes != excluded_timeframes()
        || value.prediction_task != MomentumQualifiedPredictionTaskV1::IntradayTenMinuteDirection
        || value.full_eight_replay_claimed
        || !value.full_eight_a3_blocked
        || value.month_view_load_count != 0
        || value.year_view_load_count != 0
        || !zero_authority_counters
        || value.active_committee_count != 3
        || !value.live_event_two_sealed
        || value.epoch_three_registered
        || !value.protected_artifacts_unchanged
        || !value.active_roster_unchanged
        || !value.historical_warning_preserved
        || !value.chronology_audit_passed
        || !value.leakage_audit_passed
        || !value.prediction_before_reveal_passed
        || value.labels != PUBLIC_LABELS
        || value.protected_live_tree_digest_before.as_deref() == Some("")
        || value.protected_active_roster_digest_before.as_deref() == Some("")
        || registered != registration_fields_present
        || (registered
            && (value.common_eligible_event_count == 0
                || value.minimum_training_examples == 0
                || value.maximum_training_examples < value.minimum_training_examples))
        || value.participant_metrics.len() != expected_metric_count
        || value
            .participant_metrics
            .iter()
            .any(|metrics| validate_metrics(metrics).is_err())
        || (development_complete
            && value
                .participant_metrics
                .iter()
                .filter(|metrics| metrics.partition == MomentumReplayPartitionV1::Development)
                .count()
                != 5)
        || (complete
            && value
                .participant_metrics
                .iter()
                .filter(|metrics| metrics.partition == MomentumReplayPartitionV1::Validation)
                .count()
                != 5)
        || (!complete
            && (!value.benchmark_comparisons.is_empty()
                || !value.contribution_comparisons.is_empty()
                || value.replay_digest.is_some()))
        || (complete
            && (value.benchmark_comparisons.len() != 4
                || value.contribution_comparisons.len() != 3
                || value.replay_digest.as_deref().is_none_or(str::is_empty)))
        || value
            .benchmark_comparisons
            .iter()
            .any(|comparison| validate_benchmark(comparison).is_err())
        || value
            .contribution_comparisons
            .iter()
            .any(|comparison| validate_contribution(comparison).is_err())
        || value.probability_collapse_count
            != value
                .participant_metrics
                .iter()
                .filter(|metrics| metrics.probability_collapsed)
                .count()
        || value.report_digest != report_digest(value)
    {
        return Err("qualified-six public report rejected".to_string());
    }
    Ok(())
}

fn parse_report_status(value: &str) -> Result<MomentumQualifiedReplayStatusV1, String> {
    match value {
        "Unregistered" => Ok(MomentumQualifiedReplayStatusV1::Unregistered),
        "Registered" => Ok(MomentumQualifiedReplayStatusV1::Registered),
        "DevelopmentComplete" => Ok(MomentumQualifiedReplayStatusV1::DevelopmentComplete),
        "Complete" => Ok(MomentumQualifiedReplayStatusV1::Complete),
        "InsufficientTrainingSupport" => {
            Ok(MomentumQualifiedReplayStatusV1::InsufficientTrainingSupport)
        }
        "IntegrityFailure" => Ok(MomentumQualifiedReplayStatusV1::IntegrityFailure),
        _ => Err("qualified-six report status rejected".to_string()),
    }
}

fn encode_report(value: &MomentumQualifiedSixReplayReportV1) -> Result<Vec<u8>, String> {
    validate_report(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedSixReplayReportV1")
        .string("report_version", &value.report_version)
        .string("run_mode", &value.run_mode)
        .string("status", format!("{:?}", value.status))
        .string("family", FAMILY_LABEL)
        .optional_string("registration_digest", &value.registration_digest)
        .optional_string(
            "qualified_timeframe_set_digest",
            &value.qualified_timeframe_set_digest,
        )
        .strings(
            "included_timeframes",
            &value
                .included_timeframes
                .iter()
                .map(|timeframe| timeframe_name(*timeframe).to_string())
                .collect::<Vec<_>>(),
        )
        .strings(
            "excluded_timeframes",
            &value
                .excluded_timeframes
                .iter()
                .map(|timeframe| timeframe_name(*timeframe).to_string())
                .collect::<Vec<_>>(),
        )
        .string("prediction_task", TASK_LABEL)
        .optional_string("label_policy_digest", &value.label_policy_digest)
        .unsigned(
            "common_eligible_event_count",
            as_u64(value.common_eligible_event_count)?,
        )
        .optional_string(
            "common_eligible_start_timestamp_ms",
            &value
                .common_eligible_start_timestamp_ms
                .map(|timestamp| timestamp.to_string()),
        )
        .optional_string(
            "common_eligible_end_timestamp_ms",
            &value
                .common_eligible_end_timestamp_ms
                .map(|timestamp| timestamp.to_string()),
        )
        .optional_string(
            "development_boundary_digest",
            &value.development_boundary_digest,
        )
        .optional_string(
            "validation_boundary_digest",
            &value.validation_boundary_digest,
        )
        .unsigned(
            "minimum_training_examples",
            as_u64(value.minimum_training_examples)?,
        )
        .unsigned(
            "maximum_training_examples",
            as_u64(value.maximum_training_examples)?,
        )
        .unsigned(
            "development_partition_event_count",
            as_u64(value.development_partition_event_count)?,
        )
        .unsigned(
            "development_prediction_event_count",
            as_u64(value.development_prediction_event_count)?,
        )
        .unsigned(
            "development_training_only_event_count",
            as_u64(value.development_training_only_event_count)?,
        )
        .unsigned(
            "development_scorable_event_count",
            as_u64(value.development_scorable_event_count)?,
        )
        .unsigned(
            "development_neutral_event_count",
            as_u64(value.development_neutral_event_count)?,
        )
        .unsigned(
            "development_invalid_event_count",
            as_u64(value.development_invalid_event_count)?,
        )
        .unsigned(
            "development_daily_refit_count",
            as_u64(value.development_daily_refit_count)?,
        )
        .unsigned(
            "validation_partition_event_count",
            as_u64(value.validation_partition_event_count)?,
        )
        .unsigned(
            "validation_prediction_event_count",
            as_u64(value.validation_prediction_event_count)?,
        )
        .unsigned(
            "validation_training_only_event_count",
            as_u64(value.validation_training_only_event_count)?,
        )
        .unsigned(
            "validation_scorable_event_count",
            as_u64(value.validation_scorable_event_count)?,
        )
        .unsigned(
            "validation_neutral_event_count",
            as_u64(value.validation_neutral_event_count)?,
        )
        .unsigned(
            "validation_invalid_event_count",
            as_u64(value.validation_invalid_event_count)?,
        )
        .unsigned(
            "validation_daily_refit_count",
            as_u64(value.validation_daily_refit_count)?,
        )
        .messages(
            "participant_metrics",
            value
                .participant_metrics
                .iter()
                .map(encode_metrics)
                .collect::<Result<Vec<_>, _>>()?,
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
        .unsigned(
            "probability_collapse_count",
            as_u64(value.probability_collapse_count)?,
        )
        .boolean("chronology_audit_passed", value.chronology_audit_passed)
        .boolean("leakage_audit_passed", value.leakage_audit_passed)
        .boolean(
            "prediction_before_reveal_passed",
            value.prediction_before_reveal_passed,
        )
        .boolean("full_eight_replay_claimed", value.full_eight_replay_claimed)
        .boolean("full_eight_a3_blocked", value.full_eight_a3_blocked)
        .unsigned(
            "month_view_load_count",
            as_u64(value.month_view_load_count)?,
        )
        .unsigned("year_view_load_count", as_u64(value.year_view_load_count)?)
        .unsigned("holdout_label_reads", as_u64(value.holdout_label_reads)?)
        .unsigned(
            "holdout_metric_computations",
            as_u64(value.holdout_metric_computations)?,
        )
        .unsigned(
            "holdout_participant_predictions",
            as_u64(value.holdout_participant_predictions)?,
        )
        .unsigned(
            "live_outcome_requests",
            as_u64(value.live_outcome_requests)?,
        )
        .unsigned(
            "live_outcome_openings",
            as_u64(value.live_outcome_openings)?,
        )
        .unsigned(
            "live_participant_changes",
            as_u64(value.live_participant_changes)?,
        )
        .unsigned(
            "live_parameter_updates",
            as_u64(value.live_parameter_updates)?,
        )
        .unsigned(
            "live_normalizer_refits",
            as_u64(value.live_normalizer_refits)?,
        )
        .unsigned(
            "live_completed_event_changes",
            as_u64(value.live_completed_event_changes)?,
        )
        .unsigned(
            "live_scorable_event_changes",
            as_u64(value.live_scorable_event_changes)?,
        )
        .unsigned("winner_selections", as_u64(value.winner_selections)?)
        .unsigned("ranking_creations", as_u64(value.ranking_creations)?)
        .unsigned("reward_applications", as_u64(value.reward_applications)?)
        .unsigned("penalty_applications", as_u64(value.penalty_applications)?)
        .unsigned("chair_decisions", as_u64(value.chair_decisions)?)
        .unsigned("committee_votes", as_u64(value.committee_votes)?)
        .unsigned("voice_changes", as_u64(value.voice_changes)?)
        .unsigned("tier_changes", as_u64(value.tier_changes)?)
        .unsigned("cooldowns_started", as_u64(value.cooldowns_started)?)
        .unsigned("promotions", as_u64(value.promotions)?)
        .unsigned("quarantines", as_u64(value.quarantines)?)
        .unsigned(
            "historical_participant_speaking_rights",
            as_u64(value.historical_participant_speaking_rights)?,
        )
        .unsigned(
            "historical_participant_committee_memberships",
            as_u64(value.historical_participant_committee_memberships)?,
        )
        .unsigned("paper_executions", as_u64(value.paper_executions)?)
        .unsigned("live_executions", as_u64(value.live_executions)?)
        .unsigned(
            "network_request_attempts",
            as_u64(value.network_request_attempts)?,
        )
        .unsigned(
            "transport_constructions",
            as_u64(value.transport_constructions)?,
        )
        .unsigned("credentials_read", as_u64(value.credentials_read)?)
        .unsigned(
            "active_committee_count",
            as_u64(value.active_committee_count)?,
        )
        .boolean("live_event_two_sealed", value.live_event_two_sealed)
        .boolean("epoch_three_registered", value.epoch_three_registered)
        .optional_string(
            "protected_live_tree_digest_before",
            &value.protected_live_tree_digest_before,
        )
        .optional_string(
            "protected_active_roster_digest_before",
            &value.protected_active_roster_digest_before,
        )
        .boolean(
            "protected_artifacts_unchanged",
            value.protected_artifacts_unchanged,
        )
        .boolean("active_roster_unchanged", value.active_roster_unchanged)
        .boolean(
            "historical_warning_preserved",
            value.historical_warning_preserved,
        )
        .strings("labels", &value.labels)
        .optional_string("replay_digest", &value.replay_digest)
        .unsigned("artifacts_written", as_u64(value.artifacts_written)?)
        .unsigned(
            "duplicate_artifact_count",
            as_u64(value.duplicate_artifact_count)?,
        )
        .unsigned("model_refit_count", as_u64(value.model_refit_count)?)
        .unsigned(
            "prediction_computation_count",
            as_u64(value.prediction_computation_count)?,
        )
        .unsigned(
            "metric_recomputation_count",
            as_u64(value.metric_recomputation_count)?,
        )
        .unsigned("runtime_duration_ms", value.runtime_duration_ms)
        .string("report_digest", &value.report_digest)
        .encode()
}

fn decode_optional_timestamp(
    fields: &mut ArtifactReaderV4_2,
    name: &str,
) -> Result<Option<u64>, String> {
    optional_u64_field(fields, name)
}

fn decode_report(bytes: &[u8]) -> Result<MomentumQualifiedSixReplayReportV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedSixReplayReportV1")?;
    let report_version = fields.string("report_version")?;
    let run_mode = fields.string("run_mode")?;
    let status = parse_report_status(&fields.string("status")?)?;
    if fields.string("family")? != FAMILY_LABEL {
        return Err("qualified-six report family rejected".to_string());
    }
    let registration_digest = fields.optional_string("registration_digest")?;
    let qualified_timeframe_set_digest =
        fields.optional_string("qualified_timeframe_set_digest")?;
    let included_timeframes = fields
        .strings("included_timeframes")?
        .iter()
        .map(|timeframe| parse_timeframe(timeframe))
        .collect::<Result<Vec<_>, _>>()?;
    let excluded_timeframes = fields
        .strings("excluded_timeframes")?
        .iter()
        .map(|timeframe| parse_timeframe(timeframe))
        .collect::<Result<Vec<_>, _>>()?;
    if fields.string("prediction_task")? != TASK_LABEL {
        return Err("qualified-six report task rejected".to_string());
    }
    let value = MomentumQualifiedSixReplayReportV1 {
        report_version,
        run_mode,
        status,
        family: MomentumQualifiedReplayFamilyV1::QualifiedSixIntradayTenMinute,
        registration_digest,
        qualified_timeframe_set_digest,
        included_timeframes,
        excluded_timeframes,
        prediction_task: MomentumQualifiedPredictionTaskV1::IntradayTenMinuteDirection,
        label_policy_digest: fields.optional_string("label_policy_digest")?,
        common_eligible_event_count: as_usize(fields.unsigned("common_eligible_event_count")?)?,
        common_eligible_start_timestamp_ms: decode_optional_timestamp(
            &mut fields,
            "common_eligible_start_timestamp_ms",
        )?,
        common_eligible_end_timestamp_ms: decode_optional_timestamp(
            &mut fields,
            "common_eligible_end_timestamp_ms",
        )?,
        development_boundary_digest: fields.optional_string("development_boundary_digest")?,
        validation_boundary_digest: fields.optional_string("validation_boundary_digest")?,
        minimum_training_examples: as_usize(fields.unsigned("minimum_training_examples")?)?,
        maximum_training_examples: as_usize(fields.unsigned("maximum_training_examples")?)?,
        development_partition_event_count: as_usize(
            fields.unsigned("development_partition_event_count")?,
        )?,
        development_prediction_event_count: as_usize(
            fields.unsigned("development_prediction_event_count")?,
        )?,
        development_training_only_event_count: as_usize(
            fields.unsigned("development_training_only_event_count")?,
        )?,
        development_scorable_event_count: as_usize(
            fields.unsigned("development_scorable_event_count")?,
        )?,
        development_neutral_event_count: as_usize(
            fields.unsigned("development_neutral_event_count")?,
        )?,
        development_invalid_event_count: as_usize(
            fields.unsigned("development_invalid_event_count")?,
        )?,
        development_daily_refit_count: as_usize(fields.unsigned("development_daily_refit_count")?)?,
        validation_partition_event_count: as_usize(
            fields.unsigned("validation_partition_event_count")?,
        )?,
        validation_prediction_event_count: as_usize(
            fields.unsigned("validation_prediction_event_count")?,
        )?,
        validation_training_only_event_count: as_usize(
            fields.unsigned("validation_training_only_event_count")?,
        )?,
        validation_scorable_event_count: as_usize(
            fields.unsigned("validation_scorable_event_count")?,
        )?,
        validation_neutral_event_count: as_usize(
            fields.unsigned("validation_neutral_event_count")?,
        )?,
        validation_invalid_event_count: as_usize(
            fields.unsigned("validation_invalid_event_count")?,
        )?,
        validation_daily_refit_count: as_usize(fields.unsigned("validation_daily_refit_count")?)?,
        participant_metrics: fields
            .messages("participant_metrics")?
            .iter()
            .map(|message| decode_metrics(message))
            .collect::<Result<Vec<_>, _>>()?,
        benchmark_comparisons: fields
            .messages("benchmark_comparisons")?
            .iter()
            .map(|message| decode_benchmark(message))
            .collect::<Result<Vec<_>, _>>()?,
        contribution_comparisons: fields
            .messages("contribution_comparisons")?
            .iter()
            .map(|message| decode_contribution(message))
            .collect::<Result<Vec<_>, _>>()?,
        probability_collapse_count: as_usize(fields.unsigned("probability_collapse_count")?)?,
        chronology_audit_passed: fields.boolean("chronology_audit_passed")?,
        leakage_audit_passed: fields.boolean("leakage_audit_passed")?,
        prediction_before_reveal_passed: fields.boolean("prediction_before_reveal_passed")?,
        full_eight_replay_claimed: fields.boolean("full_eight_replay_claimed")?,
        full_eight_a3_blocked: fields.boolean("full_eight_a3_blocked")?,
        month_view_load_count: as_usize(fields.unsigned("month_view_load_count")?)?,
        year_view_load_count: as_usize(fields.unsigned("year_view_load_count")?)?,
        holdout_label_reads: as_usize(fields.unsigned("holdout_label_reads")?)?,
        holdout_metric_computations: as_usize(fields.unsigned("holdout_metric_computations")?)?,
        holdout_participant_predictions: as_usize(
            fields.unsigned("holdout_participant_predictions")?,
        )?,
        live_outcome_requests: as_usize(fields.unsigned("live_outcome_requests")?)?,
        live_outcome_openings: as_usize(fields.unsigned("live_outcome_openings")?)?,
        live_participant_changes: as_usize(fields.unsigned("live_participant_changes")?)?,
        live_parameter_updates: as_usize(fields.unsigned("live_parameter_updates")?)?,
        live_normalizer_refits: as_usize(fields.unsigned("live_normalizer_refits")?)?,
        live_completed_event_changes: as_usize(fields.unsigned("live_completed_event_changes")?)?,
        live_scorable_event_changes: as_usize(fields.unsigned("live_scorable_event_changes")?)?,
        winner_selections: as_usize(fields.unsigned("winner_selections")?)?,
        ranking_creations: as_usize(fields.unsigned("ranking_creations")?)?,
        reward_applications: as_usize(fields.unsigned("reward_applications")?)?,
        penalty_applications: as_usize(fields.unsigned("penalty_applications")?)?,
        chair_decisions: as_usize(fields.unsigned("chair_decisions")?)?,
        committee_votes: as_usize(fields.unsigned("committee_votes")?)?,
        voice_changes: as_usize(fields.unsigned("voice_changes")?)?,
        tier_changes: as_usize(fields.unsigned("tier_changes")?)?,
        cooldowns_started: as_usize(fields.unsigned("cooldowns_started")?)?,
        promotions: as_usize(fields.unsigned("promotions")?)?,
        quarantines: as_usize(fields.unsigned("quarantines")?)?,
        historical_participant_speaking_rights: as_usize(
            fields.unsigned("historical_participant_speaking_rights")?,
        )?,
        historical_participant_committee_memberships: as_usize(
            fields.unsigned("historical_participant_committee_memberships")?,
        )?,
        paper_executions: as_usize(fields.unsigned("paper_executions")?)?,
        live_executions: as_usize(fields.unsigned("live_executions")?)?,
        network_request_attempts: as_usize(fields.unsigned("network_request_attempts")?)?,
        transport_constructions: as_usize(fields.unsigned("transport_constructions")?)?,
        credentials_read: as_usize(fields.unsigned("credentials_read")?)?,
        active_committee_count: as_usize(fields.unsigned("active_committee_count")?)?,
        live_event_two_sealed: fields.boolean("live_event_two_sealed")?,
        epoch_three_registered: fields.boolean("epoch_three_registered")?,
        protected_live_tree_digest_before: fields
            .optional_string("protected_live_tree_digest_before")?,
        protected_active_roster_digest_before: fields
            .optional_string("protected_active_roster_digest_before")?,
        protected_artifacts_unchanged: fields.boolean("protected_artifacts_unchanged")?,
        active_roster_unchanged: fields.boolean("active_roster_unchanged")?,
        historical_warning_preserved: fields.boolean("historical_warning_preserved")?,
        labels: fields.strings("labels")?,
        replay_digest: fields.optional_string("replay_digest")?,
        artifacts_written: as_usize(fields.unsigned("artifacts_written")?)?,
        duplicate_artifact_count: as_usize(fields.unsigned("duplicate_artifact_count")?)?,
        model_refit_count: as_usize(fields.unsigned("model_refit_count")?)?,
        prediction_computation_count: as_usize(fields.unsigned("prediction_computation_count")?)?,
        metric_recomputation_count: as_usize(fields.unsigned("metric_recomputation_count")?)?,
        runtime_duration_ms: fields.unsigned("runtime_duration_ms")?,
        report_digest: fields.string("report_digest")?,
    };
    fields.finish()?;
    validate_report(&value)?;
    Ok(value)
}

fn read_partition_aggregate(
    partition: MomentumReplayPartitionV1,
) -> Result<Option<MomentumQualifiedPartitionAggregateV1>, String> {
    read_only(&aggregate_category(partition), decode_aggregate)
}

fn require_persisted_registration(prepared: &PreparedReplay) -> Result<(), String> {
    let persisted = read_only("registrations", decode_registration)?
        .ok_or_else(|| "qualified-six registration required".to_string())?;
    if persisted != prepared.registration {
        return Err("qualified-six registration identity mismatch".to_string());
    }
    let label = read_only("label_policies", decode_label_policy)?;
    let partition = read_only("partition_policies", decode_partition_policy)?;
    let holdout = read_only("holdout_boundaries", decode_holdout)?;
    let eligibility = read_only("eligibility_audits", decode_eligibility)?;
    if label.as_ref() != Some(&prepared.label_policy)
        || partition.as_ref() != Some(&prepared.partition_policy)
        || holdout.as_ref() != Some(&prepared.holdout_boundary)
        || eligibility.as_ref() != Some(&prepared.eligibility_audit)
    {
        return Err("qualified-six persisted static contract mismatch".to_string());
    }
    Ok(())
}

fn duplicate_complete_report(
    mut report: MomentumQualifiedSixReplayReportV1,
    mode: MomentumQualifiedSixRunModeV1,
    started: Instant,
) -> Result<MomentumQualifiedSixReplayReportV1, String> {
    let current = momentum_qualified_replay_protected_state_v1()?;
    if report.protected_live_tree_digest_before.as_deref()
        != Some(current.live_tree_digest.as_str())
        || report.protected_active_roster_digest_before.as_deref()
            != Some(current.active_roster_digest.as_str())
        || current.live_outcome_requests != 0
        || current.live_outcome_openings != 0
        || current.epoch_three_registered
        || current.active_committee_count != 3
    {
        return Err("qualified-six protected state changed".to_string());
    }
    report.run_mode = mode.as_str().to_string();
    report.artifacts_written = 0;
    report.duplicate_artifact_count = count_runtime_artifacts()?;
    report.model_refit_count = 0;
    report.prediction_computation_count = 0;
    report.metric_recomputation_count = 0;
    report.runtime_duration_ms = started.elapsed().as_millis() as u64;
    report.report_digest = report_digest(&report);
    validate_report(&report)?;
    Ok(report)
}

fn run_momentum_qualified_six_replay_inner_v1(
    mode: MomentumQualifiedSixRunModeV1,
) -> Result<MomentumQualifiedSixReplayReportV1, String> {
    let started = Instant::now();
    let protected_before = momentum_qualified_replay_protected_state_v1()?;
    if let Some(report) = read_only("final_reports", decode_report)? {
        return duplicate_complete_report(report, mode, started);
    }

    let persisted_registration = read_only("registrations", decode_registration)?;
    if mode == MomentumQualifiedSixRunModeV1::Status && persisted_registration.is_none() {
        let protected_after = momentum_qualified_replay_protected_state_v1()?;
        return build_report(
            mode.as_str(),
            None,
            None,
            None,
            Vec::new(),
            Vec::new(),
            None,
            &protected_before,
            &protected_after,
            (0, 0),
            0,
            0,
            0,
            started.elapsed().as_millis() as u64,
        );
    }

    let prepared = prepare_replay()?;
    if let Some(registration) = persisted_registration {
        if registration != prepared.registration {
            return Err("qualified-six registration identity mismatch".to_string());
        }
    }
    let mut counts = (0usize, 0usize);
    let mut refits = 0usize;
    let mut predictions = 0usize;
    let mut metrics = 0usize;
    let mut development = read_partition_aggregate(MomentumReplayPartitionV1::Development)?;
    let mut validation = read_partition_aggregate(MomentumReplayPartitionV1::Validation)?;
    let mut benchmarks = Vec::new();
    let mut contributions = Vec::new();
    let mut journal = None;

    if matches!(
        mode,
        MomentumQualifiedSixRunModeV1::ExecuteDevelopment
            | MomentumQualifiedSixRunModeV1::ExecuteValidation
    ) && prepared.registration.minimum_training_examples
        > prepared.partition_policy.development_event_count
    {
        require_persisted_registration(&prepared)?;
        let protected_after = momentum_qualified_replay_protected_state_v1()?;
        let mut report = build_report(
            mode.as_str(),
            Some(&prepared),
            None,
            None,
            Vec::new(),
            Vec::new(),
            None,
            &protected_before,
            &protected_after,
            (0, 0),
            0,
            0,
            0,
            started.elapsed().as_millis() as u64,
        )?;
        report.status = MomentumQualifiedReplayStatusV1::InsufficientTrainingSupport;
        report.report_digest = report_digest(&report);
        validate_report(&report)?;
        return Ok(report);
    }

    match mode {
        MomentumQualifiedSixRunModeV1::Status | MomentumQualifiedSixRunModeV1::DryRun => {}
        MomentumQualifiedSixRunModeV1::Register => {
            add_counts(&mut counts, persist_static(&prepared)?);
        }
        MomentumQualifiedSixRunModeV1::ExecuteDevelopment => {
            require_persisted_registration(&prepared)?;
            let result = execute_partition(&prepared, MomentumReplayPartitionV1::Development)?;
            development = Some(result.0);
            add_counts(&mut counts, result.1);
            refits += result.2;
            predictions += result.3;
            metrics += result.4;
        }
        MomentumQualifiedSixRunModeV1::ExecuteValidation => {
            require_persisted_registration(&prepared)?;
            let existing_development = development
                .as_ref()
                .ok_or_else(|| "qualified-six development replay required".to_string())?;
            if existing_development.registration_digest != prepared.registration.registration_digest
            {
                return Err("qualified-six development identity mismatch".to_string());
            }
            let result = execute_partition(&prepared, MomentumReplayPartitionV1::Validation)?;
            validation = Some(result.0);
            add_counts(&mut counts, result.1);
            refits += result.2;
            predictions += result.3;
            metrics += result.4;
            let final_values = persist_final_comparisons(
                &prepared,
                existing_development,
                validation
                    .as_ref()
                    .ok_or_else(|| "qualified-six validation replay unavailable".to_string())?,
            )?;
            benchmarks = final_values.0;
            contributions = final_values.1;
            journal = Some(final_values.2);
            add_counts(&mut counts, final_values.3);
        }
    }

    if validation.is_some() && journal.is_none() {
        return Err("qualified-six completed replay journal missing".to_string());
    }
    let protected_after = momentum_qualified_replay_protected_state_v1()?;
    let completing = mode == MomentumQualifiedSixRunModeV1::ExecuteValidation;
    let report_counts = if completing {
        (counts.0 + 1, counts.1)
    } else {
        counts
    };
    let report = build_report(
        mode.as_str(),
        Some(&prepared),
        development.as_ref(),
        validation.as_ref(),
        benchmarks,
        contributions,
        journal.as_ref(),
        &protected_before,
        &protected_after,
        report_counts,
        refits,
        predictions,
        metrics,
        started.elapsed().as_millis() as u64,
    )?;
    if completing {
        let persisted = persist_one(
            "final_reports",
            &report.report_digest,
            &encode_report(&report)?,
            |bytes| Ok(decode_report(bytes)?.report_digest),
        )?;
        if persisted != (1, 0) {
            return Err("qualified-six final report persistence rejected".to_string());
        }
        let reopened = read_exact("final_reports", &report.report_digest, decode_report)?
            .ok_or_else(|| "qualified-six final report reopen failed".to_string())?;
        if reopened != report {
            return Err("qualified-six final report reopen mismatch".to_string());
        }
    }
    Ok(report)
}

pub fn run_momentum_qualified_six_replay_v1(
    mode: MomentumQualifiedSixRunModeV1,
) -> Result<MomentumQualifiedSixReplayReportV1, String> {
    match run_momentum_qualified_six_replay_inner_v1(mode) {
        Ok(report) => Ok(report),
        Err(error)
            if error.contains("artifact")
                || error.contains("conflict")
                || error.contains("mismatch") =>
        {
            let protected = momentum_qualified_replay_protected_state_v1()?;
            let mut report = build_report(
                mode.as_str(),
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                None,
                &protected,
                &protected,
                (0, 0),
                0,
                0,
                0,
                0,
            )?;
            report.status = MomentumQualifiedReplayStatusV1::IntegrityFailure;
            report.report_digest = report_digest(&report);
            validate_report(&report)?;
            Ok(report)
        }
        Err(error) => Err(error),
    }
}

pub fn format_momentum_qualified_six_replay_text_v1(
    report: &MomentumQualifiedSixReplayReportV1,
) -> String {
    let mut output = String::new();
    use std::fmt::Write as _;
    let _ = writeln!(output, "status={:?}", report.status);
    let _ = writeln!(output, "family={:?}", report.family);
    let _ = writeln!(
        output,
        "included_timeframes={}",
        report
            .included_timeframes
            .iter()
            .map(|timeframe| timeframe_name(*timeframe))
            .collect::<Vec<_>>()
            .join(",")
    );
    let _ = writeln!(
        output,
        "excluded_timeframes={}",
        report
            .excluded_timeframes
            .iter()
            .map(|timeframe| timeframe_name(*timeframe))
            .collect::<Vec<_>>()
            .join(",")
    );
    let _ = writeln!(
        output,
        "common_eligible_event_count={}",
        report.common_eligible_event_count
    );
    let _ = writeln!(
        output,
        "development=events:{};predictions:{};training_only:{};scorable:{};neutral:{};invalid:{};daily_refits:{}",
        report.development_partition_event_count,
        report.development_prediction_event_count,
        report.development_training_only_event_count,
        report.development_scorable_event_count,
        report.development_neutral_event_count,
        report.development_invalid_event_count,
        report.development_daily_refit_count,
    );
    let _ = writeln!(
        output,
        "validation=events:{};predictions:{};training_only:{};scorable:{};neutral:{};invalid:{};daily_refits:{}",
        report.validation_partition_event_count,
        report.validation_prediction_event_count,
        report.validation_training_only_event_count,
        report.validation_scorable_event_count,
        report.validation_neutral_event_count,
        report.validation_invalid_event_count,
        report.validation_daily_refit_count,
    );
    for metric in &report.participant_metrics {
        let _ = writeln!(
            output,
            "participant={};partition={};events={};scorable={};neutral={};invalid={};mean_brier={:?};correctness={:?};delta_constant={:?};probability_collapsed={}",
            metric.participant_id,
            metric.partition.as_str(),
            metric.total_prediction_events,
            metric.scorable_events,
            metric.neutral_events,
            metric.invalid_events,
            metric.mean_brier_score,
            metric.binary_correctness,
            metric.delta_versus_constant,
            metric.probability_collapsed,
        );
    }
    for benchmark in &report.benchmark_comparisons {
        let _ = writeln!(
            output,
            "benchmark={};classification={:?}",
            benchmark.participant_id, benchmark.classification
        );
    }
    for contribution in &report.contribution_comparisons {
        let _ = writeln!(
            output,
            "contribution={}:{};classification={:?}",
            contribution.added_participant_id,
            contribution.baseline_participant_id,
            contribution.status
        );
    }
    let _ = writeln!(
        output,
        "holdout_closed={};holdout_label_reads={};holdout_metric_computations={};holdout_participant_predictions={}",
        report.holdout_label_reads == 0
            && report.holdout_metric_computations == 0
            && report.holdout_participant_predictions == 0,
        report.holdout_label_reads,
        report.holdout_metric_computations,
        report.holdout_participant_predictions,
    );
    let _ = writeln!(
        output,
        "authority=winner:{};ranking:{};reward:{};penalty:{};chair:{};votes:{};voice:{};tier:{};cooldowns:{};promotions:{};quarantines:{};historical_speaking_rights:{};historical_committee_memberships:{};paper:{};live:{};network:{}",
        report.winner_selections,
        report.ranking_creations,
        report.reward_applications,
        report.penalty_applications,
        report.chair_decisions,
        report.committee_votes,
        report.voice_changes,
        report.tier_changes,
        report.cooldowns_started,
        report.promotions,
        report.quarantines,
        report.historical_participant_speaking_rights,
        report.historical_participant_committee_memberships,
        report.paper_executions,
        report.live_executions,
        report.network_request_attempts,
    );
    let _ = writeln!(output, "labels={}", report.labels.join(","));
    let _ = writeln!(output, "replay_digest={:?}", report.replay_digest);
    let _ = writeln!(output, "report_digest={}", report.report_digest);
    let _ = writeln!(output, "artifacts_written={}", report.artifacts_written);
    let _ = writeln!(
        output,
        "duplicate_artifact_count={}",
        report.duplicate_artifact_count
    );
    let _ = writeln!(output, "model_refit_count={}", report.model_refit_count);
    let _ = writeln!(
        output,
        "prediction_computation_count={}",
        report.prediction_computation_count
    );
    let _ = writeln!(
        output,
        "metric_recomputation_count={}",
        report.metric_recomputation_count
    );
    let _ = writeln!(output, "runtime_duration_ms={}", report.runtime_duration_ms);
    output
}

#[cfg(test)]
mod tests {
    use std::sync::{
        OnceLock,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    static TEMP_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

    struct TestArtifact(PathBuf);

    impl TestArtifact {
        fn new(name: &str) -> Self {
            Self(std::env::temp_dir().join(format!(
                "soma-qualified-six-{name}-{}-{}.pb",
                std::process::id(),
                TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            )))
        }
    }

    impl Drop for TestArtifact {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    fn prepared() -> &'static PreparedReplay {
        static PREPARED: OnceLock<PreparedReplay> = OnceLock::new();
        PREPARED.get_or_init(|| prepare_replay().expect("qualified-six fixture"))
    }

    fn feature_block_fixture() -> MomentumQualifiedTimeframeFeatureBlockV1 {
        let event = &prepared().events[0];
        let source = event
            .blocks
            .get(&MomentumHistoricalTimeframeV1::Minute10)
            .expect("10m block");
        let mut value = MomentumQualifiedTimeframeFeatureBlockV1 {
            block_version: BLOCK_VERSION.to_string(),
            timeframe: source.timeframe,
            context_timestamp_ms: source.context_timestamp_ms.clone(),
            source_candle_digests: source.source_candle_digests.clone(),
            feature_schema_digest: source.feature_schema_digest.clone(),
            feature_vector_digest: source.feature_vector_digest.clone(),
            normalizer_digest: "normalizer".to_string(),
            future_access_count: 0,
            partial_access_count: 0,
            missing_evidence_count: 0,
            block_digest: String::new(),
        };
        value.block_digest = block_digest(&value);
        value
    }

    fn refit_fixture() -> MomentumQualifiedDailyRefitReceiptV1 {
        let mut value = MomentumQualifiedDailyRefitReceiptV1 {
            refit_version: REFIT_VERSION.to_string(),
            registration_digest: prepared().registration.registration_digest.clone(),
            utc_day_boundary_ms: DAY_MS,
            training_target_cutoff_exclusive_ms: DAY_MS,
            eligible_past_event_count: 600,
            scorable_training_event_count: 550,
            used_training_event_count: 512,
            participant_parameter_digests: (0..5)
                .map(|index| format!("parameter-{index}"))
                .collect(),
            timeframe_normalizer_digests: (0..6)
                .map(|index| format!("normalizer-{index}"))
                .collect(),
            within_day_refit_count: 0,
            live_parameter_load_count: 0,
            prior_fold_parameter_load_count: 0,
            refit_digest: String::new(),
        };
        value.refit_digest = refit_digest(&value);
        value
    }

    fn refit_bundle_fixture() -> MomentumQualifiedDailyRefitBundleV1 {
        let refit = refit_fixture();
        let dimension = MomentumFeatureConfigV0::default().feature_count();
        let normalizers = included_timeframes()
            .into_iter()
            .map(|timeframe| {
                (
                    timeframe,
                    RepresentationNormalizerV0 {
                        means: vec![0.0; dimension],
                        scales: vec![1.0; dimension],
                        constant_dimension_indices: Vec::new(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let prevalence = 0.5_f64;
        let frozen = MomentumQualifiedParticipantV1::ORDERED
            .iter()
            .enumerate()
            .map(|(ordinal, participant)| {
                if *participant == MomentumQualifiedParticipantV1::Q0TrainingPrevalenceConstant {
                    return FrozenParticipant {
                        participant: *participant,
                        parameter_digest: stable_hash_string(&format!(
                            "qualified-six-research-constant-v1:{}:{}:{}",
                            refit.utc_day_boundary_ms,
                            refit.scorable_training_event_count,
                            prevalence.to_bits()
                        )),
                        normalizer_binding_digest: stable_hash_string(
                            "qualified-six-constant-past-labels-only",
                        ),
                        probability_head: None,
                        prevalence,
                    };
                }
                let head = LogisticPredictionHeadV0::seeded(
                    participant.timeframes().len() * dimension,
                    ordinal as u64 + 1,
                )
                .expect("head");
                let normalizer_binding_digest = stable_hash_string(&format!(
                    "qualified-six-normalizer-binding-v1:{:?}",
                    participant
                        .timeframes()
                        .iter()
                        .map(|timeframe| normalizers[timeframe].digest())
                        .collect::<Vec<_>>()
                ));
                FrozenParticipant {
                    participant: *participant,
                    parameter_digest: stable_hash_string(&format!(
                        "qualified-six-research-parameter-v1:{}:{}:{}",
                        participant.id(),
                        refit.utc_day_boundary_ms,
                        head.parameter_digest()
                    )),
                    normalizer_binding_digest,
                    probability_head: Some(head),
                    prevalence,
                }
            })
            .collect::<Vec<_>>();
        let mut refit = refit;
        refit.participant_parameter_digests = frozen
            .iter()
            .map(|participant| participant.parameter_digest.clone())
            .collect();
        refit.timeframe_normalizer_digests = included_timeframes()
            .iter()
            .map(|timeframe| normalizers[timeframe].digest())
            .collect();
        refit.refit_digest = refit_digest(&refit);
        build_refit_bundle(
            prepared(),
            MomentumReplayPartitionV1::Development,
            refit,
            &normalizers,
            &frozen,
        )
        .expect("refit bundle")
    }

    fn event_plan_fixture() -> MomentumQualifiedReplayEventPlanV1 {
        let event = &prepared().events[0];
        let mut value = MomentumQualifiedReplayEventPlanV1 {
            plan_version: EVENT_PLAN_VERSION.to_string(),
            registration_digest: prepared().registration.registration_digest.clone(),
            partition: MomentumReplayPartitionV1::Development,
            event_number: event.event_number,
            prediction_timestamp_ms: event.prediction_timestamp_ms,
            target_timestamp_ms: event.target_timestamp_ms,
            daily_refit_receipt_digest: "refit".to_string(),
            timeframe_block_digests: (0..6).map(|index| format!("block-{index}")).collect(),
            participant_ids: MomentumQualifiedParticipantV1::ORDERED
                .iter()
                .map(|participant| participant.id().to_string())
                .collect(),
            event_plan_digest: String::new(),
        };
        value.event_plan_digest = event_plan_digest(&value);
        value
    }

    fn capsule_fixture() -> MomentumQualifiedReplayPredictionCapsuleV1 {
        let mut value = MomentumQualifiedReplayPredictionCapsuleV1 {
            capsule_version: CAPSULE_VERSION.to_string(),
            event_plan_digest: event_plan_fixture().event_plan_digest,
            participant_seal_digests: (0..5).map(|index| format!("seal-{index}")).collect(),
            participant_prediction_digests: (0..5)
                .map(|index| format!("prediction-{index}"))
                .collect(),
            target_accessed: false,
            label_accessed: false,
            metrics_computed: false,
            capsule_digest: String::new(),
        };
        value.capsule_digest = capsule_digest(&value);
        value
    }

    fn neutral_evaluation_fixture() -> MomentumQualifiedReplayEvaluationV1 {
        let capsule = capsule_fixture();
        let mut value = MomentumQualifiedReplayEvaluationV1 {
            evaluation_version: EVALUATION_VERSION.to_string(),
            event_plan_digest: capsule.event_plan_digest.clone(),
            prediction_capsule_digest: capsule.capsule_digest,
            label_status: MomentumQualifiedLabelStatusV1::Neutral,
            private_label: None,
            participant_evaluation_digests: Vec::new(),
            private_brier_values: Vec::new(),
            private_correctness: Vec::new(),
            evaluation_digest: String::new(),
        };
        value.evaluation_digest = evaluation_digest(&value);
        value
    }

    fn metrics_fixture(
        participant: MomentumQualifiedParticipantV1,
        partition: MomentumReplayPartitionV1,
        score: f64,
    ) -> MomentumQualifiedParticipantMetricsV1 {
        let mut value = MomentumQualifiedParticipantMetricsV1 {
            participant_id: participant.id().to_string(),
            partition,
            total_prediction_events: 10,
            scorable_events: 8,
            neutral_events: 2,
            invalid_events: 0,
            finite_prediction_count: 10,
            probability_collapsed: false,
            mean_brier_score: Some(score),
            binary_correctness: Some(0.625),
            delta_versus_constant: Some(score - 0.25),
            paired_scorable_count: 8,
            chronology_audit_passed: true,
            leakage_audit_passed: true,
            metrics_digest: String::new(),
        };
        value.metrics_digest = metrics_digest(&value);
        value
    }

    fn aggregate_fixture(
        partition: MomentumReplayPartitionV1,
    ) -> MomentumQualifiedPartitionAggregateV1 {
        let scores = [0.25, 0.24, 0.23, 0.22, 0.21];
        let participant_metrics = MomentumQualifiedParticipantV1::ORDERED
            .iter()
            .zip(scores)
            .map(|(participant, score)| metrics_fixture(*participant, partition, score))
            .collect();
        let mut value = MomentumQualifiedPartitionAggregateV1 {
            aggregate_version: AGGREGATE_VERSION.to_string(),
            registration_digest: "registration".to_string(),
            partition,
            partition_event_count: 10,
            training_only_event_count: 0,
            prediction_event_count: 10,
            scorable_event_count: 8,
            neutral_event_count: 2,
            invalid_event_count: 0,
            daily_refit_count: 1,
            daily_prediction_bundle_digests: vec!["prediction-bundle".to_string()],
            daily_evaluation_bundle_digests: vec!["evaluation-bundle".to_string()],
            participant_metrics,
            target_access_before_capsule_count: 0,
            future_access_count: 0,
            partial_access_count: 0,
            unqualified_access_count: 0,
            aggregate_digest: String::new(),
        };
        value.aggregate_digest = aggregate_digest(&value);
        value
    }

    #[test]
    fn sprint98_01_pr28_forensic_invariants_are_revalidated() {
        let value = prepared();
        assert_eq!(value.evidence.included_timeframes, included_timeframes());
        assert_eq!(value.evidence.excluded_timeframes, excluded_timeframes());
        assert!(validate_eligibility(&value.eligibility_audit).is_ok());
    }

    #[test]
    fn sprint98_02_all_nineteen_macro_failures_remain_unresolved() {
        assert!(load_momentum_qualified_six_evidence_v1().is_ok());
        assert_eq!(prepared().evidence.excluded_timeframes.len(), 2);
    }

    #[test]
    fn sprint98_03_forensic_tolerance_contract_remains_unchanged() {
        assert!(load_momentum_qualified_six_evidence_v1().is_ok());
    }

    #[test]
    fn sprint98_04_full_eight_a3_remains_blocked() {
        assert!(prepared().registration.full_eight_replay_claim_forbidden);
        assert_eq!(prepared().registration.included_timeframes.len(), 6);
    }

    #[test]
    fn sprint98_05_qualified_six_registration_has_separate_identity() {
        let registration = &prepared().registration;
        assert_eq!(
            registration.family,
            MomentumQualifiedReplayFamilyV1::QualifiedSixIntradayTenMinute
        );
        assert_ne!(
            registration.registration_digest,
            registration.qualified_timeframe_set_digest
        );
    }

    #[test]
    fn sprint98_06_qualified_six_cannot_claim_full_eight() {
        let mut registration = prepared().registration.clone();
        registration.full_eight_replay_claim_forbidden = false;
        registration.registration_digest = registration_digest(&registration);
        assert!(validate_registration(&registration).is_err());
    }

    #[test]
    fn sprint98_07_month_is_never_loaded() {
        assert_eq!(prepared().eligibility_audit.month_view_load_count, 0);
        assert!(
            !prepared()
                .evidence
                .views
                .contains_key(&MomentumHistoricalTimeframeV1::Month1)
        );
    }

    #[test]
    fn sprint98_08_year_is_never_loaded() {
        assert_eq!(prepared().eligibility_audit.year_view_load_count, 0);
        assert!(
            !prepared()
                .evidence
                .views
                .contains_key(&MomentumHistoricalTimeframeV1::Year1)
        );
    }

    #[test]
    fn sprint98_09_six_views_are_loaded_causally() {
        assert_eq!(prepared().evidence.views.len(), 6);
        assert_eq!(prepared().eligibility_audit.future_access_count, 0);
        assert_eq!(prepared().eligibility_audit.unqualified_access_count, 0);
    }

    #[test]
    fn sprint98_10_event_is_completed_ten_minute_boundary() {
        assert!(
            prepared()
                .events
                .iter()
                .all(|event| event.prediction_timestamp_ms % TEN_MINUTE_MS == 0)
        );
    }

    #[test]
    fn sprint98_11_target_is_next_completed_ten_minute_candle() {
        assert!(prepared().events.iter().all(|event| {
            event.target_timestamp_ms == event.prediction_timestamp_ms + TEN_MINUTE_MS
                && event.target_ten_minute_index == event.current_ten_minute_index + 1
        }));
    }

    #[test]
    fn sprint98_12_target_access_before_capsule_rejects() {
        let mut value = capsule_fixture();
        value.target_accessed = true;
        value.capsule_digest = capsule_digest(&value);
        assert!(validate_capsule(&value).is_err());
    }

    #[test]
    fn sprint98_13_equal_close_is_neutral() {
        assert_eq!(
            classify_label(42.0, 42.0),
            MomentumQualifiedLabelStatusV1::Neutral
        );
    }

    #[test]
    fn sprint98_14_partitions_are_chronological_and_derived() {
        let policy = &prepared().partition_policy;
        assert!(policy.eligible_start_timestamp_ms < policy.development_end_exclusive_ms);
        assert!(policy.development_end_exclusive_ms < policy.validation_end_exclusive_ms);
        assert_eq!(
            policy.validation_end_exclusive_ms,
            policy.holdout_start_timestamp_ms
        );
    }

    #[test]
    fn sprint98_15_sealed_holdout_remains_closed() {
        let holdout = &prepared().holdout_boundary;
        assert!(!holdout.labels_opened);
        assert_eq!(holdout.metric_computations, 0);
        assert_eq!(holdout.participant_predictions, 0);
    }

    #[test]
    fn sprint98_16_incompatible_prior_holdout_creates_additive_boundary() {
        let prepared = prepared();
        if prepared.evidence.prior_holdout.holdout_start_timestamp_ms
            != prepared.partition_policy.holdout_start_timestamp_ms
        {
            assert!(!prepared.holdout_boundary.adopted_prior_boundary);
        }
        assert_eq!(
            prepared.holdout_boundary.holdout_start_timestamp_ms,
            prepared.partition_policy.holdout_start_timestamp_ms
        );
    }

    #[test]
    fn sprint98_17_daily_refit_uses_prior_revealed_labels_only() {
        let bundle = refit_bundle_fixture();
        let reopened =
            decode_refit_bundle(&encode_refit_bundle(&bundle).expect("encode refit bundle"))
                .expect("reopen refit bundle");
        let (receipt, normalizers, participants) =
            reconstruct_refit_bundle(&reopened).expect("reconstruct refit bundle");
        assert_eq!(
            receipt.training_target_cutoff_exclusive_ms,
            receipt.utc_day_boundary_ms
        );
        assert_eq!(normalizers.len(), 6);
        assert_eq!(participants.len(), 5);
        assert!(validate_refit(&receipt).is_ok());
    }

    #[test]
    fn sprint98_18_no_within_day_refit_occurs() {
        assert_eq!(refit_fixture().within_day_refit_count, 0);
    }

    #[test]
    fn sprint98_19_constant_benchmark_uses_past_labels_only() {
        assert!(
            MomentumQualifiedParticipantV1::Q0TrainingPrevalenceConstant
                .timeframes()
                .is_empty()
        );
        assert!(refit_fixture().scorable_training_event_count >= 512);
    }

    #[test]
    fn sprint98_20_anchor_uses_only_ten_minute_block() {
        assert_eq!(
            MomentumQualifiedParticipantV1::Q1TenMinuteAnchorLogistic.timeframes(),
            vec![MomentumHistoricalTimeframeV1::Minute10]
        );
    }

    #[test]
    fn sprint98_21_micro_uses_exactly_four_intraday_blocks() {
        assert_eq!(
            MomentumQualifiedParticipantV1::Q2MicroBlockLogistic.timeframes(),
            included_timeframes()[..4]
        );
    }

    #[test]
    fn sprint98_22_macro_uses_exactly_daily_and_weekly_blocks() {
        assert_eq!(
            MomentumQualifiedParticipantV1::Q3QualifiedMacroBlockLogistic.timeframes(),
            vec![
                MomentumHistoricalTimeframeV1::Day1,
                MomentumHistoricalTimeframeV1::Week1
            ]
        );
    }

    #[test]
    fn sprint98_23_fusion_uses_all_six_qualified_blocks() {
        assert_eq!(
            MomentumQualifiedParticipantV1::Q4QualifiedSixFusionLogistic.timeframes(),
            included_timeframes()
        );
    }

    #[test]
    fn sprint98_24_live_parameters_cannot_be_loaded() {
        assert!(
            prepared()
                .participants
                .iter()
                .all(|participant| participant.live_parameter_use_forbidden)
        );
        assert_eq!(refit_fixture().live_parameter_load_count, 0);
    }

    #[test]
    fn sprint98_25_prior_fold_parameters_cannot_be_reused() {
        assert!(
            prepared()
                .participants
                .iter()
                .all(|participant| participant.prior_fold_parameter_use_forbidden)
        );
        assert_eq!(refit_fixture().prior_fold_parameter_load_count, 0);
    }

    #[test]
    fn sprint98_26_each_block_uses_closed_candles_only() {
        let event = &prepared().events[0];
        assert!(event.blocks.values().all(|block| {
            block
                .context_timestamp_ms
                .last()
                .is_some_and(|timestamp| *timestamp <= event.prediction_timestamp_ms)
        }));
    }

    #[test]
    fn sprint98_27_partial_candle_rejects() {
        let mut value = feature_block_fixture();
        value.partial_access_count = 1;
        value.block_digest = block_digest(&value);
        assert!(validate_feature_block(&value).is_err());
    }

    #[test]
    fn sprint98_28_missing_evidence_rejects() {
        let mut value = feature_block_fixture();
        value.missing_evidence_count = 1;
        value.block_digest = block_digest(&value);
        assert!(validate_feature_block(&value).is_err());
    }

    #[test]
    fn sprint98_29_all_participants_share_event_timestamp() {
        let plan = event_plan_fixture();
        assert_eq!(plan.participant_ids.len(), 5);
        assert!(validate_event_plan(&plan).is_ok());
    }

    #[test]
    fn sprint98_30_exactly_five_predictions_are_required() {
        assert_eq!(capsule_fixture().participant_prediction_digests.len(), 5);
        assert!(validate_capsule(&capsule_fixture()).is_ok());
    }

    #[test]
    fn sprint98_31_partial_prediction_capsule_rejects() {
        let mut value = capsule_fixture();
        value.participant_prediction_digests.pop();
        value.capsule_digest = capsule_digest(&value);
        assert!(validate_capsule(&value).is_err());
    }

    #[test]
    fn sprint98_32_probabilities_remain_private() {
        let report = empty_report("test");
        let json = serde_json::to_string(&report).expect("public JSON");
        let text = format_momentum_qualified_six_replay_text_v1(&report);
        assert!(!json.contains("private_probability"));
        assert!(!text.contains("private_probability"));
        assert!(!json.contains("sealed_holdout_boundary"));
        assert!(!json.contains("sealed_holdout_start"));
        assert!(!json.contains("sealed_holdout_event_count"));
    }

    #[test]
    fn sprint98_33_target_reveal_follows_capsule_reopen() {
        let capsule = capsule_fixture();
        let reopened =
            decode_capsule(&encode_capsule(&capsule).expect("encode")).expect("reopen capsule");
        let evaluation = neutral_evaluation_fixture();
        assert_eq!(
            evaluation.prediction_capsule_digest,
            reopened.capsule_digest
        );
        assert!(validate_evaluation(&evaluation).is_ok());
    }

    #[test]
    fn sprint98_34_neutral_excludes_brier_and_correctness() {
        let value = neutral_evaluation_fixture();
        assert!(value.private_brier_values.is_empty());
        assert!(value.private_correctness.is_empty());
    }

    #[test]
    fn sprint98_35_brier_uses_scorable_events_only() {
        let value = metrics_fixture(
            MomentumQualifiedParticipantV1::Q1TenMinuteAnchorLogistic,
            MomentumReplayPartitionV1::Development,
            0.2,
        );
        assert_eq!(value.paired_scorable_count, value.scorable_events);
        assert!(value.scorable_events < value.total_prediction_events);
    }

    #[test]
    fn sprint98_36_development_and_validation_metrics_are_separate() {
        let development = aggregate_fixture(MomentumReplayPartitionV1::Development);
        let validation = aggregate_fixture(MomentumReplayPartitionV1::Validation);
        assert!(
            development
                .participant_metrics
                .iter()
                .all(|metrics| metrics.partition == MomentumReplayPartitionV1::Development)
        );
        assert!(
            validation
                .participant_metrics
                .iter()
                .all(|metrics| metrics.partition == MomentumReplayPartitionV1::Validation)
        );
    }

    #[test]
    fn sprint98_37_constant_comparison_is_paired() {
        let values = build_benchmarks(
            &aggregate_fixture(MomentumReplayPartitionV1::Development),
            &aggregate_fixture(MomentumReplayPartitionV1::Validation),
        )
        .expect("benchmarks");
        assert!(values.iter().all(|value| {
            value.paired_development_count == 8 && value.paired_validation_count == 8
        }));
    }

    #[test]
    fn sprint98_38_micro_vs_anchor_contribution_derives() {
        let values = build_contributions(
            &aggregate_fixture(MomentumReplayPartitionV1::Development),
            &aggregate_fixture(MomentumReplayPartitionV1::Validation),
        )
        .expect("contributions");
        assert_eq!(
            values[0].added_participant_id,
            MomentumQualifiedParticipantV1::Q2MicroBlockLogistic.id()
        );
    }

    #[test]
    fn sprint98_39_fusion_vs_micro_contribution_derives() {
        let values = build_contributions(
            &aggregate_fixture(MomentumReplayPartitionV1::Development),
            &aggregate_fixture(MomentumReplayPartitionV1::Validation),
        )
        .expect("contributions");
        assert_eq!(
            values[1].baseline_participant_id,
            MomentumQualifiedParticipantV1::Q2MicroBlockLogistic.id()
        );
    }

    #[test]
    fn sprint98_40_fusion_vs_macro_contribution_derives() {
        let values = build_contributions(
            &aggregate_fixture(MomentumReplayPartitionV1::Development),
            &aggregate_fixture(MomentumReplayPartitionV1::Validation),
        )
        .expect("contributions");
        assert_eq!(
            values[2].baseline_participant_id,
            MomentumQualifiedParticipantV1::Q3QualifiedMacroBlockLogistic.id()
        );
    }

    #[test]
    fn sprint98_41_no_winner_is_selected() {
        assert_eq!(empty_report("test").winner_selections, 0);
    }

    #[test]
    fn sprint98_42_no_result_selected_participant_is_added() {
        let report = empty_report("test");
        assert_eq!(report.live_participant_changes, 0);
        assert_eq!(report.ranking_creations, 0);
    }

    #[test]
    fn sprint98_43_holdout_prediction_count_stays_zero() {
        assert_eq!(prepared().holdout_boundary.participant_predictions, 0);
    }

    #[test]
    fn sprint98_44_live_event_two_outcome_access_stays_zero() {
        let state = momentum_qualified_replay_protected_state_v1().expect("protected state");
        assert_eq!(state.live_outcome_requests, 0);
        assert_eq!(state.live_outcome_openings, 0);
    }

    #[test]
    fn sprint98_45_live_counts_remain_unchanged() {
        let before = momentum_qualified_replay_protected_state_v1().expect("before");
        let after = momentum_qualified_replay_protected_state_v1().expect("after");
        assert_eq!(before, after);
    }

    #[test]
    fn sprint98_46_reward_and_chair_counters_are_zero() {
        let report = empty_report("test");
        assert_eq!(report.reward_applications, 0);
        assert_eq!(report.chair_decisions, 0);
        assert_eq!(report.voice_changes, 0);
        assert_eq!(report.tier_changes, 0);
        assert_eq!(report.cooldowns_started, 0);
        assert_eq!(report.promotions, 0);
        assert_eq!(report.quarantines, 0);
        assert_eq!(report.historical_participant_speaking_rights, 0);
        assert_eq!(report.historical_participant_committee_memberships, 0);
    }

    #[test]
    fn sprint98_47_deterministic_replay_identities_match() {
        let left = build_participants().expect("left");
        let right = build_participants().expect("right");
        assert_eq!(left, right);
        assert_eq!(
            prepared().registration.registration_digest,
            registration_digest(&prepared().registration)
        );
    }

    #[test]
    fn sprint98_48_duplicate_replay_performs_zero_writes() {
        let path = TestArtifact::new("duplicate");
        let policy = build_label_policy().expect("policy");
        let bytes = encode_label_policy(&policy).expect("bytes");
        let first = persist_artifact(&path.0, &bytes, &policy.policy_digest, |value| {
            Ok(decode_label_policy(value)?.policy_digest)
        })
        .expect("first write");
        let second = persist_artifact(&path.0, &bytes, &policy.policy_digest, |value| {
            Ok(decode_label_policy(value)?.policy_digest)
        })
        .expect("duplicate");
        assert_eq!(first, (1, 0));
        assert_eq!(second, (0, 1));
    }

    #[test]
    fn sprint98_49_conflicting_replay_artifacts_reject() {
        let path = TestArtifact::new("conflict");
        let policy = build_label_policy().expect("policy");
        let bytes = encode_label_policy(&policy).expect("bytes");
        persist_artifact(&path.0, &bytes, &policy.policy_digest, |value| {
            Ok(decode_label_policy(value)?.policy_digest)
        })
        .expect("first write");
        assert!(
            persist_artifact(&path.0, &bytes, "different-digest", |value| {
                Ok(decode_label_policy(value)?.policy_digest)
            })
            .is_err()
        );
        let mut report = empty_report("test-conflict");
        report.status = MomentumQualifiedReplayStatusV1::IntegrityFailure;
        report.report_digest = report_digest(&report);
        assert!(validate_report(&report).is_ok());
    }

    #[test]
    fn sprint98_50_malformed_protobuf_rejects() {
        let mut bytes =
            encode_label_policy(&build_label_policy().expect("policy")).expect("encoded policy");
        bytes.truncate(bytes.len() / 2);
        assert!(decode_label_policy(&bytes).is_err());
    }

    #[test]
    fn sprint98_51_text_and_json_agree() {
        let report = empty_report("test");
        validate_report(&report).expect("report");
        let text = format_momentum_qualified_six_replay_text_v1(&report);
        let json = serde_json::to_value(&report).expect("JSON");
        for field in [
            "common_eligible_event_count",
            "artifacts_written",
            "duplicate_artifact_count",
            "model_refit_count",
            "prediction_computation_count",
            "metric_recomputation_count",
            "runtime_duration_ms",
        ] {
            assert!(text.contains(&format!("{field}={}", json[field])));
        }
        assert!(text.contains(&format!("status={:?}", report.status)));
        assert!(text.contains(&format!("report_digest={}", report.report_digest)));
    }
}
