//! Offline, additive diagnostics and bounded repair for the frozen Momentum Mamba head.
//!
//! This module has no network, active-committee, reward, voting, or execution authority.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{
    core::stable_hash_string,
    data::{DataSnapshot, historical_replay_dataset_digest_v0},
    league::canonical_current_agent_states,
};

use super::agent_learning_session::{
    AgentPrivateLearningArtifactWriteStatusV0, atomic_write_verified_v0,
};
use super::{
    AgentCandidateFamilyV1, AgentCandidateUsageLedgerV1, AgentPrivateLearningRunModeV0,
    AgentPrivateLearningSessionV1, CandidateEvidenceUseV1, CandidateParticipantRoleV1,
    ConstantProbabilityBaselineV0, EvaluationMetricsV0, FeatureNormalizerV0,
    FrozenCandidateParticipantV1, HeadTrainingConfigV0, IndexRangeV0, LinearMomentumBaselineV0,
    LogisticPredictionHeadV0, ModelAgentDeploymentStatus, MomentumCandleV0,
    MomentumLearningCampaignConfigV0, ParticipantValidationQualificationV1,
    ProtectedEvaluationReservationV1, SequenceExampleV0, SequencePooling,
    ValidationQualificationStatusV1, brier_loss_and_gradients_v0, build_momentum_features_v0,
    build_momentum_sequence_examples_v0, decode_candidate_family_protobuf_v1,
    decode_participant_protobuf_v1, decode_qualification_receipt_protobuf_v1,
    decode_session_protobuf_v1, decode_trainer_projection_protobuf_v1,
    decode_usage_ledger_protobuf_v1, evaluate_head_v0, frozen_mamba3_encoder_from_seed_v0,
    read_persisted_learning_intent_migration_v1, train_frozen_mamba_head_v0,
};

const AGENT_ID_V2: &str = "momentum_trend_fast";
const AUDIT_VERSION_V2: &str = "momentum-mamba-collapse-audit-v2";
const SPLIT_VERSION_V2: &str = "momentum-mamba-repair-split-v2";
const REGISTRATION_VERSION_V2: &str = "momentum-mamba-repair-registration-v2";
const VARIANT_VERSION_V2: &str = "momentum-mamba-repair-variant-v2";
const PARTICIPANT_VERSION_V2: &str = "frozen-candidate-participant-v2";
const QUALIFICATION_VERSION_V2: &str = "participant-validation-qualification-v2";
const FAMILY_VERSION_V2: &str = "momentum-candidate-family-v2";
const ROSTER_VERSION_V2: &str = "momentum-future-evaluation-roster-v2";
const EVALUATION_VERSION_V2: &str = "momentum-future-evaluation-registration-v2";
const JOURNAL_VERSION_V2: &str = "momentum-mamba-repair-journal-v2";
const MAXIMUM_REPAIR_VARIANTS_V2: usize = 4;
const REPAIR_TRAINING_EXTENSION_ROWS_V2: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MomentumMambaCollapseRootCauseV2 {
    RepresentationNearConstant,
    RepresentationLowVariance,
    RepresentationLowEffectiveRank,
    HeadOptimizationStalled,
    HeadParameterDeltaNearZero,
    GradientNearZero,
    ProbabilityNearConstant,
    ProbabilitySingleSided,
    ProbabilitySaturatedLow,
    ProbabilitySaturatedHigh,
    ValidationClassImbalanceDominated,
    NumericalFailure,
    Mixed,
    InsufficientDiagnosticEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMambaRepairCapabilityStatusV2 {
    RepairableWithExistingHeadControls,
    RepairableWithExistingPoolingControls,
    RepairableWithBoundedHeadRegularization,
    RepresentationPathBlocked,
    FreshValidationInsufficient,
    UnsupportedRepairRequired,
    TechnicalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMambaCollapseAuditV2 {
    pub audit_version: String,
    pub agent_id: String,
    pub source_family_digest: String,
    pub failed_participant_digest: String,
    pub failed_qualification_receipt_digest: String,
    pub training_range_digest: String,
    pub prior_validation_range_digest: String,
    pub encoder_digest: String,
    pub representation_normalizer_digest: String,
    pub feature_normalizer_digest: String,
    pub head_parameter_digest: String,
    pub representation_diagnostic_digest: String,
    pub optimization_diagnostic_digest: String,
    pub probability_diagnostic_digest: String,
    pub class_balance_diagnostic_digest: String,
    pub root_causes: Vec<MomentumMambaCollapseRootCauseV2>,
    pub repair_capability_status: MomentumMambaRepairCapabilityStatusV2,
    pub audit_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMambaRepairSplitV2 {
    pub split_version: String,
    pub source_snapshot_digest: String,
    pub prior_usage_ledger_digest: String,
    pub repair_training_range: IndexRangeV0,
    pub repair_purge_range: IndexRangeV0,
    pub fresh_repair_validation_range: IndexRangeV0,
    pub remaining_reserved_range: Option<IndexRangeV0>,
    pub label_horizon: usize,
    pub minimum_validation_samples: usize,
    pub prior_validation_overlap_count: usize,
    pub prospective_overlap_count: usize,
    pub future_evaluation_overlap_count: usize,
    pub split_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMambaRepairVariantConfigV2 {
    pub variant_id: String,
    pub pooling_policy: SequencePooling,
    pub learning_rate_bits: u32,
    pub l2_regularization_bits: u32,
    pub maximum_epochs: usize,
    pub class_weight_policy: String,
    pub initialization_seed: u64,
    pub encoder_frozen: bool,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub training_policy_digest: String,
    pub variant_config_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMambaRepairRegistrationV2 {
    pub registration_version: String,
    pub agent_id: String,
    pub source_snapshot_digest: String,
    pub canonical_intent_digest: String,
    pub canonical_view_digest: String,
    pub source_family_digest: String,
    pub failed_participant_digest: String,
    pub collapse_audit_digest: String,
    pub repair_split_digest: String,
    pub allowed_variant_configs: Vec<MomentumMambaRepairVariantConfigV2>,
    pub maximum_repair_variants: usize,
    pub fresh_validation_hidden: bool,
    pub historical_test_forbidden: bool,
    pub future_evaluation_forbidden: bool,
    pub winner_selection_forbidden: bool,
    pub active_promotion_forbidden: bool,
    pub reward_application_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ParticipantQualificationRoleV2 {
    LearnedCandidate,
    LinearComparator,
    ConstantBenchmark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationQualificationStatusV2 {
    Qualified,
    BenchmarkQualified,
    RejectedInsufficientValidation,
    RejectedRepresentationCollapse,
    RejectedProbabilityCollapse,
    RejectedNumericalFailure,
    RejectedPolicyInvariant,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenCandidateParticipantV2 {
    pub participant_version: String,
    pub participant_id: String,
    pub participant_role: ParticipantQualificationRoleV2,
    pub model_kind: String,
    pub variant_config_digest: Option<String>,
    pub source_snapshot_digest: String,
    pub repair_training_range_digest: String,
    pub fresh_validation_range_digest: String,
    pub validation_timestamp_digest: String,
    pub model_artifact_digest: String,
    pub parameter_digest: String,
    pub feature_normalizer_digest: String,
    pub encoder_digest: Option<String>,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub training_policy_digest: String,
    pub initialization_digest: String,
    pub warm_start_from_v1: bool,
    pub v1_head_reused: bool,
    pub fresh_deterministic_initialization: bool,
    pub encoder_frozen: bool,
    pub deployment_status: ModelAgentDeploymentStatus,
    pub participant_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParticipantValidationQualificationV2 {
    pub receipt_version: String,
    pub participant_id: String,
    pub participant_role: ParticipantQualificationRoleV2,
    pub participant_digest: String,
    pub fresh_validation_range_digest: String,
    pub qualification_policy_digest: String,
    pub private_metric_digest: String,
    pub qualification_status: ValidationQualificationStatusV2,
    pub validation_parameter_updates: usize,
    pub historical_test_reads: usize,
    pub future_evaluation_reads: usize,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumCandidateFamilyV2 {
    pub family_version: String,
    pub agent_id: String,
    pub source_snapshot_digest: String,
    pub canonical_view_digest: String,
    pub repair_registration_digest: String,
    pub repair_split_digest: String,
    pub collapse_audit_digest: String,
    pub participants: Vec<FrozenCandidateParticipantV2>,
    pub qualification_receipts: Vec<ParticipantValidationQualificationV2>,
    pub learned_participant_count: usize,
    pub qualified_learned_participant_count: usize,
    pub qualified_comparator_count: usize,
    pub winner_selected: bool,
    pub historical_test_accessed: bool,
    pub eligible_for_active_committee: bool,
    pub eligible_for_promotion: bool,
    pub eligible_for_reward: bool,
    pub family_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumFutureEvaluationRosterV2 {
    pub roster_version: String,
    pub family_digest: String,
    pub qualified_learned_participant_digests: Vec<String>,
    pub qualified_comparator_digests: Vec<String>,
    pub excluded_participant_digests: Vec<String>,
    pub inclusion_policy_digest: String,
    pub roster_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumFutureEvaluationRosterStatusV2 {
    Registered,
    NoQualifiedLearnedParticipant,
    InsufficientComparators,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumFutureEvaluationRegistrationStatusV2 {
    Registered,
    NoQualifiedLearnedParticipant,
    InsufficientComparators,
    SafetyContractInvalid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumFutureEvaluationRegistrationV2 {
    pub registration_version: String,
    pub agent_id: String,
    pub family_digest: String,
    pub roster_digest: String,
    pub repair_registration_digest: String,
    pub collapse_audit_digest: String,
    pub qualification_receipt_digests: Vec<String>,
    pub source_snapshot_digest: String,
    pub source_boundary_timestamp_ms: u64,
    pub protected_registration_digests: Vec<String>,
    pub protected_timestamp_ms: Vec<u64>,
    pub prior_reserved_range_digests: Vec<String>,
    pub provider_finality_boundary_ms: u64,
    pub minimum_accepted_timestamp_ms: u64,
    pub labels_hidden_until_opening: bool,
    pub probabilities_hidden_until_opening: bool,
    pub one_time_opening_required: bool,
    pub winner_selection_forbidden_before_opening: bool,
    pub active_promotion_forbidden: bool,
    pub reward_application_forbidden: bool,
    pub maximum_requests: usize,
    pub maximum_concurrency: usize,
    pub maximum_retries: usize,
    pub status: MomentumFutureEvaluationRegistrationStatusV2,
    pub registration_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMambaRepairExecutionStatusV2 {
    Planned,
    Executed,
    AlreadyExecuted,
    FreshRepairValidationInsufficient,
    UnsupportedRepairRequired,
    TechnicalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMambaRepairJournalV2 {
    pub journal_version: String,
    pub agent_id: String,
    pub collapse_audit_digest: String,
    pub repair_split_digest: String,
    pub repair_registration_digest: String,
    pub family_digest: Option<String>,
    pub roster_digest: Option<String>,
    pub evaluation_registration_digest: Option<String>,
    pub prior_validation_used_for_repair_qualification: bool,
    pub warm_start_from_v1: bool,
    pub v1_head_reused: bool,
    pub fresh_deterministic_initialization: bool,
    pub status: MomentumMambaRepairExecutionStatusV2,
    pub journal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMambaRepairSafetyCountersV2 {
    pub network_requests: usize,
    pub transport_constructions: usize,
    pub credential_reads: usize,
    pub prospective_row_reads: usize,
    pub prospective_label_openings: usize,
    pub future_evaluation_reads: usize,
    pub historical_test_reads: usize,
    pub active_model_changes: usize,
    pub chair_decisions: usize,
    pub votes: usize,
    pub reward_applications: usize,
    pub penalty_applications: usize,
    pub voice_changes: usize,
    pub cooldowns_started: usize,
    pub promotions: usize,
    pub quarantines: usize,
    pub executions: usize,
    pub active_committee_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMambaRepairReportV2 {
    pub report_version: String,
    pub mode: AgentPrivateLearningRunModeV0,
    pub status: MomentumMambaRepairExecutionStatusV2,
    pub collapse_audit: Option<MomentumMambaCollapseAuditV2>,
    pub repair_split: Option<MomentumMambaRepairSplitV2>,
    pub repair_registration: Option<MomentumMambaRepairRegistrationV2>,
    pub family: Option<MomentumCandidateFamilyV2>,
    pub roster: Option<MomentumFutureEvaluationRosterV2>,
    pub roster_status: MomentumFutureEvaluationRosterStatusV2,
    pub evaluation_registration: Option<MomentumFutureEvaluationRegistrationV2>,
    pub evaluation_registration_status: MomentumFutureEvaluationRegistrationStatusV2,
    pub journal: Option<MomentumMambaRepairJournalV2>,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub storage_failure_count: usize,
    pub protected_artifacts_unchanged: bool,
    pub active_state_unchanged: bool,
    pub safety_counters: MomentumMambaRepairSafetyCountersV2,
    pub report_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VarianceStatusV2 {
    NearConstant,
    Low,
    Adequate,
    NumericalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EffectiveRankStatusV2 {
    RankOneOrLess,
    Low,
    Adequate,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NormStatusV2 {
    NearZero,
    Low,
    Material,
    NumericalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EarlyStopReasonV2 {
    MaximumEpochs,
    PatienceExhausted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RepresentationDiagnosticV2 {
    per_dimension_finite: Vec<bool>,
    per_dimension_variance: Vec<VarianceStatusV2>,
    constant_dimension_count: usize,
    aggregate_variance: VarianceStatusV2,
    effective_rank_status: EffectiveRankStatusV2,
    unique_representation_count: usize,
    representation_normalization_status: &'static str,
    diagnostic_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OptimizationDiagnosticV2 {
    initial_parameter_digest: String,
    final_parameter_digest: String,
    finite_parameter_status: bool,
    parameter_delta_norm: NormStatusV2,
    gradient_norm: NormStatusV2,
    update_count: usize,
    learning_rate_schedule_digest: String,
    loss_trajectory_digest: String,
    early_stop_reason: EarlyStopReasonV2,
    diagnostic_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProbabilityDiagnosticV2 {
    variance: VarianceStatusV2,
    entropy_classification: VarianceStatusV2,
    unique_bins: usize,
    positive_side_fraction_classification: &'static str,
    low_saturation_fraction_classification: &'static str,
    high_saturation_fraction_classification: &'static str,
    minimum_probability_bits: u32,
    maximum_probability_bits: u32,
    collapse_subtypes: Vec<MomentumMambaCollapseRootCauseV2>,
    diagnostic_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ClassBalanceDiagnosticV2 {
    training_positive_count: usize,
    training_negative_count: usize,
    prior_validation_positive_count: usize,
    prior_validation_negative_count: usize,
    training_single_class: bool,
    prior_validation_single_class: bool,
    imbalance_classification: &'static str,
    diagnostic_digest: String,
}

#[derive(Clone, Debug)]
pub(crate) struct V1FrozenStateV2 {
    pub(crate) input: super::AgentPrivateLearningInputV1,
    pub(crate) snapshot: DataSnapshot,
    pub(crate) session: AgentPrivateLearningSessionV1,
    pub(crate) family: AgentCandidateFamilyV1,
    pub(crate) usage_ledger: AgentCandidateUsageLedgerV1,
    pub(crate) failed_participant: FrozenCandidateParticipantV1,
    pub(crate) failed_receipt: ParticipantValidationQualificationV1,
    pub(crate) prior_training_range: IndexRangeV0,
    pub(crate) prior_validation_range: IndexRangeV0,
    pub(crate) prior_reserved_range: IndexRangeV0,
}

#[derive(Clone, Debug)]
struct RepairExperimentV2 {
    family: MomentumCandidateFamilyV2,
    roster: Option<MomentumFutureEvaluationRosterV2>,
    roster_status: MomentumFutureEvaluationRosterStatusV2,
    evaluation_registration: Option<MomentumFutureEvaluationRegistrationV2>,
    evaluation_registration_status: MomentumFutureEvaluationRegistrationStatusV2,
}

fn zero_safety_counters_v2() -> MomentumMambaRepairSafetyCountersV2 {
    MomentumMambaRepairSafetyCountersV2 {
        network_requests: 0,
        transport_constructions: 0,
        credential_reads: 0,
        prospective_row_reads: 0,
        prospective_label_openings: 0,
        future_evaluation_reads: 0,
        historical_test_reads: 0,
        active_model_changes: 0,
        chair_decisions: 0,
        votes: 0,
        reward_applications: 0,
        penalty_applications: 0,
        voice_changes: 0,
        cooldowns_started: 0,
        promotions: 0,
        quarantines: 0,
        executions: 0,
        active_committee_count: 3,
    }
}

fn range_digest_v2(label: &str, range: &IndexRangeV0) -> String {
    stable_hash_string(&format!(
        "momentum-repair-range-v2:{label}:{}:{}",
        range.start, range.end
    ))
}

fn audit_digest_v2(value: &MomentumMambaCollapseAuditV2) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}:{:?}",
        value.audit_version,
        value.agent_id,
        value.source_family_digest,
        value.failed_participant_digest,
        value.failed_qualification_receipt_digest,
        value.training_range_digest,
        value.prior_validation_range_digest,
        value.encoder_digest,
        value.representation_normalizer_digest,
        value.feature_normalizer_digest,
        value.head_parameter_digest,
        value.representation_diagnostic_digest,
        value.optimization_diagnostic_digest,
        value.probability_diagnostic_digest,
        (&value.class_balance_diagnostic_digest, &value.root_causes),
        value.repair_capability_status,
    ))
}

fn split_digest_v2(value: &MomentumMambaRepairSplitV2) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{:?}:{:?}:{:?}:{:?}:{}:{}:{}:{}:{}",
        value.split_version,
        value.source_snapshot_digest,
        value.prior_usage_ledger_digest,
        value.repair_training_range,
        value.repair_purge_range,
        value.fresh_repair_validation_range,
        value.remaining_reserved_range,
        value.label_horizon,
        value.minimum_validation_samples,
        value.prior_validation_overlap_count,
        value.prospective_overlap_count,
        value.future_evaluation_overlap_count,
    ))
}

fn variant_digest_v2(value: &MomentumMambaRepairVariantConfigV2) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        VARIANT_VERSION_V2,
        value.variant_id,
        value.pooling_policy,
        value.learning_rate_bits,
        value.l2_regularization_bits,
        value.maximum_epochs,
        value.class_weight_policy,
        value.initialization_seed,
        value.encoder_frozen,
        value.feature_policy_digest,
        value.label_policy_digest,
        value.training_policy_digest,
    ))
}

fn repair_registration_digest_v2(value: &MomentumMambaRepairRegistrationV2) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}",
        value.registration_version,
        value.agent_id,
        value.source_snapshot_digest,
        value.canonical_intent_digest,
        value.canonical_view_digest,
        value.source_family_digest,
        value.failed_participant_digest,
        value.collapse_audit_digest,
        value.repair_split_digest,
        value
            .allowed_variant_configs
            .iter()
            .map(|item| item.variant_config_digest.as_str())
            .collect::<Vec<_>>(),
        value.maximum_repair_variants,
        value.fresh_validation_hidden,
        value.historical_test_forbidden,
        value.future_evaluation_forbidden,
        value.winner_selection_forbidden,
        value.active_promotion_forbidden,
        value.reward_application_forbidden,
    ))
}

fn participant_digest_v2(value: &FrozenCandidateParticipantV2) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}:{}:{:?}:{}:{}:{}:{}:{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}",
        value.participant_version,
        value.participant_id,
        value.participant_role,
        value.model_kind,
        value.variant_config_digest,
        value.source_snapshot_digest,
        value.repair_training_range_digest,
        value.fresh_validation_range_digest,
        value.validation_timestamp_digest,
        value.model_artifact_digest,
        value.parameter_digest,
        value.encoder_digest,
        value.feature_normalizer_digest,
        value.feature_policy_digest,
        value.label_policy_digest,
        value.training_policy_digest,
        value.initialization_digest,
        value.warm_start_from_v1,
        value.v1_head_reused,
        value.fresh_deterministic_initialization,
        (value.encoder_frozen, value.deployment_status),
    ))
}

fn qualification_digest_v2(value: &ParticipantValidationQualificationV2) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}:{}:{}:{}:{}:{:?}:{}:{}:{}",
        value.receipt_version,
        value.participant_id,
        value.participant_role,
        value.participant_digest,
        value.fresh_validation_range_digest,
        value.qualification_policy_digest,
        value.private_metric_digest,
        value.qualification_status,
        value.validation_parameter_updates,
        value.historical_test_reads,
        value.future_evaluation_reads,
    ))
}

fn family_digest_v2(value: &MomentumCandidateFamilyV2) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{:?}:{:?}:{}:{}:{}:{}:{}:{}:{:?}",
        value.family_version,
        value.agent_id,
        value.source_snapshot_digest,
        value.canonical_view_digest,
        value.repair_registration_digest,
        value.repair_split_digest,
        value.collapse_audit_digest,
        value
            .participants
            .iter()
            .map(|item| item.participant_digest.as_str())
            .collect::<Vec<_>>(),
        value
            .qualification_receipts
            .iter()
            .map(|item| item.receipt_digest.as_str())
            .collect::<Vec<_>>(),
        value.learned_participant_count,
        value.qualified_learned_participant_count,
        value.qualified_comparator_count,
        value.winner_selected,
        value.historical_test_accessed,
        value.eligible_for_active_committee,
        (value.eligible_for_promotion, value.eligible_for_reward),
    ))
}

fn roster_digest_v2(value: &MomentumFutureEvaluationRosterV2) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}:{:?}:{:?}:{}",
        value.roster_version,
        value.family_digest,
        value.qualified_learned_participant_digests,
        value.qualified_comparator_digests,
        value.excluded_participant_digests,
        value.inclusion_policy_digest,
    ))
}

fn evaluation_registration_digest_v2(value: &MomentumFutureEvaluationRegistrationV2) -> String {
    stable_hash_string(&format!(
        "{:?}",
        (
            (
                value.registration_version.as_str(),
                value.agent_id.as_str(),
                value.family_digest.as_str(),
                value.roster_digest.as_str(),
                value.repair_registration_digest.as_str(),
                value.collapse_audit_digest.as_str(),
            ),
            (
                &value.qualification_receipt_digests,
                value.source_snapshot_digest.as_str(),
                value.source_boundary_timestamp_ms,
                &value.protected_registration_digests,
                &value.protected_timestamp_ms,
                &value.prior_reserved_range_digests,
                value.provider_finality_boundary_ms,
                value.minimum_accepted_timestamp_ms,
            ),
            (
                value.labels_hidden_until_opening,
                value.probabilities_hidden_until_opening,
                value.one_time_opening_required,
                value.winner_selection_forbidden_before_opening,
                value.active_promotion_forbidden,
                value.reward_application_forbidden,
                value.maximum_requests,
                value.maximum_concurrency,
                value.maximum_retries,
                value.status,
            ),
        )
    ))
}

fn journal_digest_v2(value: &MomentumMambaRepairJournalV2) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{:?}:{:?}:{:?}:{}:{}:{}:{}:{:?}",
        value.journal_version,
        value.agent_id,
        value.collapse_audit_digest,
        value.repair_split_digest,
        value.repair_registration_digest,
        value.family_digest,
        value.roster_digest,
        value.evaluation_registration_digest,
        value.prior_validation_used_for_repair_qualification,
        value.warm_start_from_v1,
        value.v1_head_reused,
        value.fresh_deterministic_initialization,
        value.status,
    ))
}

fn report_digest_v2(value: &MomentumMambaRepairReportV2) -> String {
    stable_hash_string(&format!(
        "{}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{}:{}:{}:{}:{}:{:?}",
        value.report_version,
        value.mode,
        value.status,
        value
            .collapse_audit
            .as_ref()
            .map(|item| item.audit_digest.as_str()),
        value
            .repair_split
            .as_ref()
            .map(|item| item.split_digest.as_str()),
        value
            .repair_registration
            .as_ref()
            .map(|item| item.registration_digest.as_str()),
        value
            .family
            .as_ref()
            .map(|item| item.family_digest.as_str()),
        value
            .roster
            .as_ref()
            .map(|item| item.roster_digest.as_str()),
        value.roster_status,
        value
            .evaluation_registration
            .as_ref()
            .map(|item| item.registration_digest.as_str()),
        value.evaluation_registration_status,
        value.artifacts_written,
        value.duplicate_artifact_count,
        value.storage_failure_count,
        value.protected_artifacts_unchanged,
        value.active_state_unchanged,
        value.safety_counters,
    ))
}

fn sorted_unique_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn range_valid(range: &IndexRangeV0) -> bool {
    range.start < range.end
}

fn validate_audit_v2(value: &MomentumMambaCollapseAuditV2) -> Result<(), String> {
    let mut causes = value.root_causes.clone();
    causes.sort();
    causes.dedup();
    if value.audit_version != AUDIT_VERSION_V2
        || value.agent_id != AGENT_ID_V2
        || value.source_family_digest.is_empty()
        || value.failed_participant_digest.is_empty()
        || value.failed_qualification_receipt_digest.is_empty()
        || value.training_range_digest.is_empty()
        || value.prior_validation_range_digest.is_empty()
        || value.encoder_digest.is_empty()
        || value.representation_normalizer_digest.is_empty()
        || value.feature_normalizer_digest.is_empty()
        || value.head_parameter_digest.is_empty()
        || value.representation_diagnostic_digest.is_empty()
        || value.optimization_diagnostic_digest.is_empty()
        || value.probability_diagnostic_digest.is_empty()
        || value.class_balance_diagnostic_digest.is_empty()
        || value.root_causes.is_empty()
        || value.root_causes != causes
        || value.audit_digest != audit_digest_v2(value)
    {
        return Err("Momentum Mamba collapse audit rejected".to_string());
    }
    Ok(())
}

fn validate_split_v2(value: &MomentumMambaRepairSplitV2) -> Result<(), String> {
    if value.split_version != SPLIT_VERSION_V2
        || value.source_snapshot_digest.is_empty()
        || value.prior_usage_ledger_digest.is_empty()
        || !range_valid(&value.repair_training_range)
        || !range_valid(&value.repair_purge_range)
        || !range_valid(&value.fresh_repair_validation_range)
        || value.repair_training_range.end != value.repair_purge_range.start
        || value.repair_purge_range.end != value.fresh_repair_validation_range.start
        || value
            .remaining_reserved_range
            .as_ref()
            .is_some_and(|range| {
                !range_valid(range) || range.start != value.fresh_repair_validation_range.end
            })
        || value.label_horizon == 0
        || value.minimum_validation_samples == 0
        || value.prior_validation_overlap_count != 0
        || value.prospective_overlap_count != 0
        || value.future_evaluation_overlap_count != 0
        || value.split_digest != split_digest_v2(value)
    {
        return Err("Momentum Mamba repair split rejected".to_string());
    }
    Ok(())
}

fn validate_variant_v2(value: &MomentumMambaRepairVariantConfigV2) -> Result<(), String> {
    let learning_rate = f32::from_bits(value.learning_rate_bits);
    let l2 = f32::from_bits(value.l2_regularization_bits);
    if value.variant_id.trim().is_empty()
        || !learning_rate.is_finite()
        || !(0.0..=0.5).contains(&learning_rate)
        || learning_rate == 0.0
        || !l2.is_finite()
        || !(0.0..=0.1).contains(&l2)
        || !(1..=128).contains(&value.maximum_epochs)
        || value.class_weight_policy != "none-training-only"
        || !value.encoder_frozen
        || value.feature_policy_digest.is_empty()
        || value.label_policy_digest.is_empty()
        || value.training_policy_digest.is_empty()
        || value.variant_config_digest != variant_digest_v2(value)
    {
        return Err("Momentum Mamba repair variant rejected".to_string());
    }
    Ok(())
}

fn validate_registration_v2(value: &MomentumMambaRepairRegistrationV2) -> Result<(), String> {
    let mut ids = value
        .allowed_variant_configs
        .iter()
        .map(|variant| variant.variant_id.clone())
        .collect::<Vec<_>>();
    let id_count = ids.len();
    ids.sort();
    ids.dedup();
    if value.registration_version != REGISTRATION_VERSION_V2
        || value.agent_id != AGENT_ID_V2
        || value.source_snapshot_digest.is_empty()
        || value.canonical_intent_digest.is_empty()
        || value.canonical_view_digest.is_empty()
        || value.source_family_digest.is_empty()
        || value.failed_participant_digest.is_empty()
        || value.collapse_audit_digest.is_empty()
        || value.repair_split_digest.is_empty()
        || value.allowed_variant_configs.is_empty()
        || value.allowed_variant_configs.len() > MAXIMUM_REPAIR_VARIANTS_V2
        || value.maximum_repair_variants > MAXIMUM_REPAIR_VARIANTS_V2
        || value.maximum_repair_variants < value.allowed_variant_configs.len()
        || ids.len() != id_count
        || value
            .allowed_variant_configs
            .iter()
            .any(|variant| validate_variant_v2(variant).is_err())
        || !value.fresh_validation_hidden
        || !value.historical_test_forbidden
        || !value.future_evaluation_forbidden
        || !value.winner_selection_forbidden
        || !value.active_promotion_forbidden
        || !value.reward_application_forbidden
        || value.registration_digest != repair_registration_digest_v2(value)
    {
        return Err("Momentum Mamba repair registration rejected".to_string());
    }
    Ok(())
}

fn validate_participant_v2(value: &FrozenCandidateParticipantV2) -> Result<(), String> {
    let learned = value.participant_role == ParticipantQualificationRoleV2::LearnedCandidate;
    if value.participant_version != PARTICIPANT_VERSION_V2
        || value.participant_id.is_empty()
        || value.model_kind.is_empty()
        || learned != value.variant_config_digest.is_some()
        || value.source_snapshot_digest.is_empty()
        || value.repair_training_range_digest.is_empty()
        || value.fresh_validation_range_digest.is_empty()
        || value.validation_timestamp_digest.is_empty()
        || value.model_artifact_digest.is_empty()
        || value.parameter_digest.is_empty()
        || value.feature_normalizer_digest.is_empty()
        || learned != value.encoder_digest.is_some()
        || value.feature_policy_digest.is_empty()
        || value.label_policy_digest.is_empty()
        || value.training_policy_digest.is_empty()
        || value.initialization_digest.is_empty()
        || value.warm_start_from_v1
        || value.v1_head_reused
        || !value.fresh_deterministic_initialization
        || (learned && !value.encoder_frozen)
        || value.deployment_status != ModelAgentDeploymentStatus::ShadowOnly
        || value.participant_digest != participant_digest_v2(value)
    {
        return Err("Momentum V2 participant rejected".to_string());
    }
    Ok(())
}

fn validate_qualification_v2(value: &ParticipantValidationQualificationV2) -> Result<(), String> {
    let valid_status = match value.participant_role {
        ParticipantQualificationRoleV2::LearnedCandidate => matches!(
            value.qualification_status,
            ValidationQualificationStatusV2::Qualified
                | ValidationQualificationStatusV2::RejectedInsufficientValidation
                | ValidationQualificationStatusV2::RejectedRepresentationCollapse
                | ValidationQualificationStatusV2::RejectedProbabilityCollapse
                | ValidationQualificationStatusV2::RejectedNumericalFailure
                | ValidationQualificationStatusV2::RejectedPolicyInvariant
        ),
        ParticipantQualificationRoleV2::LinearComparator => matches!(
            value.qualification_status,
            ValidationQualificationStatusV2::Qualified
                | ValidationQualificationStatusV2::RejectedInsufficientValidation
                | ValidationQualificationStatusV2::RejectedNumericalFailure
                | ValidationQualificationStatusV2::RejectedPolicyInvariant
        ),
        ParticipantQualificationRoleV2::ConstantBenchmark => matches!(
            value.qualification_status,
            ValidationQualificationStatusV2::BenchmarkQualified
                | ValidationQualificationStatusV2::RejectedInsufficientValidation
                | ValidationQualificationStatusV2::RejectedNumericalFailure
                | ValidationQualificationStatusV2::RejectedPolicyInvariant
        ),
    };
    if value.receipt_version != QUALIFICATION_VERSION_V2
        || value.participant_id.is_empty()
        || value.participant_digest.is_empty()
        || value.fresh_validation_range_digest.is_empty()
        || value.qualification_policy_digest.is_empty()
        || value.private_metric_digest.is_empty()
        || !valid_status
        || value.validation_parameter_updates != 0
        || value.historical_test_reads != 0
        || value.future_evaluation_reads != 0
        || value.receipt_digest != qualification_digest_v2(value)
    {
        return Err("Momentum V2 qualification receipt rejected".to_string());
    }
    Ok(())
}

fn validate_family_v2(value: &MomentumCandidateFamilyV2) -> Result<(), String> {
    let participant_digests = value
        .participants
        .iter()
        .map(|participant| participant.participant_digest.as_str())
        .collect::<BTreeSet<_>>();
    let receipt_participants = value
        .qualification_receipts
        .iter()
        .map(|receipt| receipt.participant_digest.as_str())
        .collect::<BTreeSet<_>>();
    let learned_count = value
        .participants
        .iter()
        .filter(|participant| {
            participant.participant_role == ParticipantQualificationRoleV2::LearnedCandidate
        })
        .count();
    let qualified_learned = value
        .qualification_receipts
        .iter()
        .filter(|receipt| {
            receipt.participant_role == ParticipantQualificationRoleV2::LearnedCandidate
                && receipt.qualification_status == ValidationQualificationStatusV2::Qualified
        })
        .count();
    let qualified_comparators = value
        .qualification_receipts
        .iter()
        .filter(|receipt| {
            receipt.participant_role != ParticipantQualificationRoleV2::LearnedCandidate
                && matches!(
                    receipt.qualification_status,
                    ValidationQualificationStatusV2::Qualified
                        | ValidationQualificationStatusV2::BenchmarkQualified
                )
        })
        .count();
    let validation_timestamps = value
        .participants
        .iter()
        .map(|participant| participant.validation_timestamp_digest.as_str())
        .collect::<BTreeSet<_>>();
    if value.family_version != FAMILY_VERSION_V2
        || value.agent_id != AGENT_ID_V2
        || value.source_snapshot_digest.is_empty()
        || value.canonical_view_digest.is_empty()
        || value.repair_registration_digest.is_empty()
        || value.repair_split_digest.is_empty()
        || value.collapse_audit_digest.is_empty()
        || value.participants.len() < 3
        || participant_digests.len() != value.participants.len()
        || value.qualification_receipts.len() != value.participants.len()
        || receipt_participants != participant_digests
        || value
            .participants
            .iter()
            .any(|participant| validate_participant_v2(participant).is_err())
        || value
            .qualification_receipts
            .iter()
            .any(|receipt| validate_qualification_v2(receipt).is_err())
        || validation_timestamps.len() != 1
        || value.learned_participant_count != learned_count
        || value.qualified_learned_participant_count != qualified_learned
        || value.qualified_comparator_count != qualified_comparators
        || value.winner_selected
        || value.historical_test_accessed
        || value.eligible_for_active_committee
        || value.eligible_for_promotion
        || value.eligible_for_reward
        || value.family_digest != family_digest_v2(value)
    {
        return Err("Momentum V2 family rejected".to_string());
    }
    Ok(())
}

fn validate_roster_v2(
    value: &MomentumFutureEvaluationRosterV2,
    family: &MomentumCandidateFamilyV2,
) -> Result<(), String> {
    let learned = sorted_unique_strings(value.qualified_learned_participant_digests.clone());
    let comparators = sorted_unique_strings(value.qualified_comparator_digests.clone());
    let excluded = sorted_unique_strings(value.excluded_participant_digests.clone());
    let all = family
        .participants
        .iter()
        .map(|participant| participant.participant_digest.clone())
        .collect::<BTreeSet<_>>();
    let included = learned
        .iter()
        .chain(&comparators)
        .chain(&excluded)
        .cloned()
        .collect::<BTreeSet<_>>();
    if value.roster_version != ROSTER_VERSION_V2
        || value.family_digest != family.family_digest
        || learned != value.qualified_learned_participant_digests
        || comparators != value.qualified_comparator_digests
        || excluded != value.excluded_participant_digests
        || learned.is_empty()
        || comparators.is_empty()
        || included != all
        || value.inclusion_policy_digest.is_empty()
        || value.roster_digest != roster_digest_v2(value)
    {
        return Err("Momentum future evaluation roster rejected".to_string());
    }
    for digest in &learned {
        let receipt = family
            .qualification_receipts
            .iter()
            .find(|receipt| receipt.participant_digest == *digest)
            .ok_or_else(|| "qualified learned receipt unavailable".to_string())?;
        if receipt.participant_role != ParticipantQualificationRoleV2::LearnedCandidate
            || receipt.qualification_status != ValidationQualificationStatusV2::Qualified
        {
            return Err("unqualified learned participant entered roster".to_string());
        }
    }
    for digest in &comparators {
        let receipt = family
            .qualification_receipts
            .iter()
            .find(|receipt| receipt.participant_digest == *digest)
            .ok_or_else(|| "qualified comparator receipt unavailable".to_string())?;
        if receipt.participant_role == ParticipantQualificationRoleV2::LearnedCandidate
            || !matches!(
                receipt.qualification_status,
                ValidationQualificationStatusV2::Qualified
                    | ValidationQualificationStatusV2::BenchmarkQualified
            )
        {
            return Err("unqualified comparator entered roster".to_string());
        }
    }
    Ok(())
}

fn validate_evaluation_registration_v2(
    value: &MomentumFutureEvaluationRegistrationV2,
    family: &MomentumCandidateFamilyV2,
    roster: &MomentumFutureEvaluationRosterV2,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<(), String> {
    let participant_receipts = roster
        .qualified_learned_participant_digests
        .iter()
        .chain(&roster.qualified_comparator_digests)
        .filter_map(|digest| {
            family
                .qualification_receipts
                .iter()
                .find(|receipt| receipt.participant_digest == *digest)
                .map(|receipt| receipt.receipt_digest.clone())
        })
        .collect::<BTreeSet<_>>();
    let receipt_digests = value
        .qualification_receipt_digests
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if value.registration_version != EVALUATION_VERSION_V2
        || value.agent_id != AGENT_ID_V2
        || value.family_digest != family.family_digest
        || value.roster_digest != roster.roster_digest
        || value.repair_registration_digest != family.repair_registration_digest
        || value.collapse_audit_digest != family.collapse_audit_digest
        || participant_receipts != receipt_digests
        || value.source_snapshot_digest != family.source_snapshot_digest
        || value.source_boundary_timestamp_ms == 0
        || value.protected_registration_digests != reservation.protected_registration_digests
        || value.protected_timestamp_ms != reservation.reserved_timestamp_ms
        || value.prior_reserved_range_digests.is_empty()
        || value.provider_finality_boundary_ms != reservation.provider_finality_boundary_ms
        || value.minimum_accepted_timestamp_ms
            < value
                .source_boundary_timestamp_ms
                .saturating_add(reservation.cadence_ms)
        || value.minimum_accepted_timestamp_ms < reservation.provider_finality_boundary_ms
        || !value.labels_hidden_until_opening
        || !value.probabilities_hidden_until_opening
        || !value.one_time_opening_required
        || !value.winner_selection_forbidden_before_opening
        || !value.active_promotion_forbidden
        || !value.reward_application_forbidden
        || value.maximum_requests != 1
        || value.maximum_concurrency != 1
        || value.maximum_retries != 0
        || value.status != MomentumFutureEvaluationRegistrationStatusV2::Registered
        || value.registration_digest != evaluation_registration_digest_v2(value)
    {
        return Err("Momentum V2 future evaluation registration rejected".to_string());
    }
    Ok(())
}

fn validate_journal_v2(value: &MomentumMambaRepairJournalV2) -> Result<(), String> {
    if value.journal_version != JOURNAL_VERSION_V2
        || value.agent_id != AGENT_ID_V2
        || value.collapse_audit_digest.is_empty()
        || value.repair_split_digest.is_empty()
        || value.repair_registration_digest.is_empty()
        || value.prior_validation_used_for_repair_qualification
        || value.warm_start_from_v1
        || value.v1_head_reused
        || !value.fresh_deterministic_initialization
        || value.journal_digest != journal_digest_v2(value)
    {
        return Err("Momentum Mamba repair journal rejected".to_string());
    }
    Ok(())
}

fn protobuf_paths(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = fs::read_dir(directory)
        .map_err(|_| "required V1 artifact directory unavailable".to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|value| value == "pb"))
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn read_single_protobuf<T>(
    directory: &Path,
    decode: impl Fn(&[u8]) -> Result<T, String>,
) -> Result<T, String> {
    let paths = protobuf_paths(directory)?;
    if paths.len() != 1 {
        return Err("required V1 artifact identity is ambiguous".to_string());
    }
    let bytes = fs::read(&paths[0]).map_err(|_| "required V1 artifact read failed".to_string())?;
    decode(&bytes)
}

fn ledger_range(
    ledger: &AgentCandidateUsageLedgerV1,
    kind: CandidateEvidenceUseV1,
) -> Result<IndexRangeV0, String> {
    let mut ranges = ledger
        .entries
        .iter()
        .filter(|entry| entry.use_kind == kind)
        .filter_map(|entry| entry.range.clone())
        .collect::<Vec<_>>();
    ranges.sort_by_key(|range| (range.start, range.end));
    ranges.dedup();
    if ranges.len() != 1 {
        return Err("V1 evidence usage range is ambiguous".to_string());
    }
    Ok(ranges.remove(0))
}

pub(crate) fn load_v1_frozen_state_v2(
    root: &Path,
    snapshots: &[DataSnapshot],
) -> Result<V1FrozenStateV2, String> {
    let input = read_persisted_learning_intent_migration_v1(root, snapshots)?;
    if !input.persisted_view_verified || input.input.intent.agent_id != AGENT_ID_V2 {
        return Err("verified migrated Momentum view unavailable".to_string());
    }
    let snapshot = snapshots
        .iter()
        .find(|snapshot| {
            input.input.view.source_artifact_digests == vec![snapshot.content_digest.clone()]
        })
        .cloned()
        .ok_or_else(|| "verified Momentum snapshot unavailable".to_string())?;
    if historical_replay_dataset_digest_v0(&snapshot.normalized_dataset) != snapshot.content_digest
        || snapshot.row_count != snapshot.normalized_dataset.rows.len()
        || snapshot.row_count == 0
        || !snapshot.read_only
        || !snapshot.sanitized
    {
        return Err("verified Momentum snapshot rejected".to_string());
    }
    let v1_root = root.join("v1").join(AGENT_ID_V2);
    let session = read_single_protobuf(&v1_root.join("sessions"), decode_session_protobuf_v1)?;
    let projection = read_single_protobuf(
        &v1_root.join("projections"),
        decode_trainer_projection_protobuf_v1,
    )?;
    let family = read_single_protobuf(
        &v1_root.join("families"),
        decode_candidate_family_protobuf_v1,
    )?;
    let usage_ledger = read_single_protobuf(
        &v1_root.join("usage_ledgers"),
        decode_usage_ledger_protobuf_v1,
    )?;
    let participants = protobuf_paths(&v1_root.join("participants"))?
        .into_iter()
        .map(|path| fs::read(path).map_err(|_| "V1 participant read failed".to_string()))
        .map(|bytes| bytes.and_then(|bytes| decode_participant_protobuf_v1(&bytes)))
        .collect::<Result<Vec<_>, _>>()?;
    let receipts = protobuf_paths(&v1_root.join("qualification_receipts"))?
        .into_iter()
        .map(|path| fs::read(path).map_err(|_| "V1 qualification receipt read failed".to_string()))
        .map(|bytes| bytes.and_then(|bytes| decode_qualification_receipt_protobuf_v1(&bytes)))
        .collect::<Result<Vec<_>, _>>()?;
    let family_participants = family
        .participants
        .iter()
        .map(|participant| participant.participant_digest.clone())
        .collect::<BTreeSet<_>>();
    let stored_participants = participants
        .iter()
        .map(|participant| participant.participant_digest.clone())
        .collect::<BTreeSet<_>>();
    let family_receipts = family
        .validation_qualification_receipts
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let stored_receipts = receipts
        .iter()
        .map(|receipt| receipt.receipt_digest.clone())
        .collect::<BTreeSet<_>>();
    if session.agent_id != AGENT_ID_V2
        || session.intent_digest != input.input.intent.intent_digest
        || session.view_digest != input.input.view.view_digest
        || projection.projection_digest != session.projection_digest
        || family.agent_id != session.agent_id
        || family.session_digest != session.session_digest
        || family.view_digest != session.view_digest
        || family.projection_digest != projection.projection_digest
        || family_participants != stored_participants
        || family_receipts != stored_receipts
        || participants.len() != 3
        || receipts.len() != 3
        || usage_ledger.session_digest != session.session_digest
        || usage_ledger.family_digest != family.family_digest
        || usage_ledger.historical_test_row_reads != 0
        || usage_ledger.historical_test_label_reads != 0
        || usage_ledger.historical_test_inference_count != 0
        || usage_ledger.historical_test_metric_count != 0
        || usage_ledger.historical_test_checkpoint_selection_count != 0
        || usage_ledger.historical_test_identity_influence
        || family.winner_selected
        || family.historical_test_accessed
        || family.eligible_for_active_committee
        || family.eligible_for_promotion
        || family.eligible_for_reward
    {
        return Err("immutable V1 family binding rejected".to_string());
    }
    let failed = participants
        .iter()
        .filter(|participant| {
            participant.role == CandidateParticipantRoleV1::ModelCandidate
                && participant.model_kind == "FrozenMambaHeadV1"
        })
        .cloned()
        .collect::<Vec<_>>();
    if failed.len() != 1 {
        return Err("failed V1 Mamba participant is ambiguous".to_string());
    }
    let failed_participant = failed[0].clone();
    let failed_receipts = receipts
        .iter()
        .filter(|receipt| receipt.participant_digest == failed_participant.participant_digest)
        .cloned()
        .collect::<Vec<_>>();
    if failed_receipts.len() != 1
        || failed_receipts[0].qualification_status
            != ValidationQualificationStatusV1::RejectedProbabilityCollapse
    {
        return Err("failed V1 Mamba qualification identity rejected".to_string());
    }
    let prior_training_range =
        ledger_range(&usage_ledger, CandidateEvidenceUseV1::ParameterTraining)?;
    let prior_purge_range = ledger_range(
        &usage_ledger,
        CandidateEvidenceUseV1::ReferencedButUnconsumed,
    )?;
    let prior_validation_range =
        ledger_range(&usage_ledger, CandidateEvidenceUseV1::ValidationInference)?;
    let prior_reserved_range = ledger_range(
        &usage_ledger,
        CandidateEvidenceUseV1::ReservedRetrospectiveUnused,
    )?;
    if prior_training_range.end != prior_purge_range.start
        || prior_purge_range.end != prior_validation_range.start
        || prior_validation_range.end != prior_reserved_range.start
        || prior_reserved_range.end != snapshot.row_count
    {
        return Err("V1 evidence usage partition rejected".to_string());
    }
    Ok(V1FrozenStateV2 {
        input,
        snapshot,
        session,
        family,
        usage_ledger,
        failed_participant,
        failed_receipt: failed_receipts[0].clone(),
        prior_training_range,
        prior_validation_range,
        prior_reserved_range,
    })
}

pub(crate) fn candles_from_snapshot_prefix(
    snapshot: &DataSnapshot,
    end: usize,
) -> Result<Vec<MomentumCandleV0>, String> {
    if end == 0 || end > snapshot.normalized_dataset.rows.len() {
        return Err("Momentum diagnostic evidence range rejected".to_string());
    }
    snapshot.normalized_dataset.rows[..end]
        .iter()
        .map(|row| {
            Ok(MomentumCandleV0 {
                timestamp: i64::try_from(row.timestamp_ms)
                    .map_err(|_| "Momentum timestamp rejected".to_string())?,
                open: row.open as f32,
                high: row.high as f32,
                low: row.low as f32,
                close: row.close as f32,
                volume: row.volume as f32,
            })
        })
        .collect()
}

pub(crate) fn examples_in_range(
    examples: &[SequenceExampleV0],
    range: &IndexRangeV0,
) -> Vec<SequenceExampleV0> {
    examples
        .iter()
        .filter(|example| example.sequence_start >= range.start && example.label_index < range.end)
        .cloned()
        .collect()
}

fn mean_and_variance(values: &[f32]) -> Option<(f32, f32)> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return None;
    }
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f32>()
        / values.len() as f32;
    (mean.is_finite() && variance.is_finite()).then_some((mean, variance))
}

fn variance_status(variance: f32) -> VarianceStatusV2 {
    if !variance.is_finite() {
        VarianceStatusV2::NumericalFailure
    } else if variance <= 1e-12 {
        VarianceStatusV2::NearConstant
    } else if variance <= 1e-5 {
        VarianceStatusV2::Low
    } else {
        VarianceStatusV2::Adequate
    }
}

fn norm_status(norm: f32) -> NormStatusV2 {
    if !norm.is_finite() {
        NormStatusV2::NumericalFailure
    } else if norm <= 1e-8 {
        NormStatusV2::NearZero
    } else if norm <= 1e-4 {
        NormStatusV2::Low
    } else {
        NormStatusV2::Material
    }
}

fn representation_diagnostic_v2(
    representations: &[super::EncodedTrainingExampleV0],
    normalization_status: &'static str,
) -> RepresentationDiagnosticV2 {
    let dimension = representations
        .first()
        .map_or(0, |row| row.representation.len());
    let per_dimension_finite = (0..dimension)
        .map(|dimension| {
            representations.iter().all(|row| {
                row.representation
                    .get(dimension)
                    .is_some_and(|value| value.is_finite())
            })
        })
        .collect::<Vec<_>>();
    let variances = (0..dimension)
        .map(|dimension| {
            mean_and_variance(
                &representations
                    .iter()
                    .filter_map(|row| row.representation.get(dimension).copied())
                    .collect::<Vec<_>>(),
            )
            .map_or(f32::NAN, |(_, variance)| variance)
        })
        .collect::<Vec<_>>();
    let per_dimension_variance = variances
        .iter()
        .copied()
        .map(variance_status)
        .collect::<Vec<_>>();
    let constant_dimension_count = per_dimension_variance
        .iter()
        .filter(|status| **status == VarianceStatusV2::NearConstant)
        .count();
    let aggregate_variance_value = if variances.is_empty() {
        f32::NAN
    } else {
        variances.iter().sum::<f32>() / variances.len() as f32
    };
    let aggregate_variance = variance_status(aggregate_variance_value);
    let sum = variances.iter().sum::<f32>();
    let squared_sum = variances.iter().map(|value| value * value).sum::<f32>();
    let effective_rank = if sum.is_finite() && squared_sum.is_finite() && squared_sum > 0.0 {
        Some(sum * sum / squared_sum)
    } else {
        None
    };
    let effective_rank_status = match effective_rank {
        None => EffectiveRankStatusV2::Unavailable,
        Some(rank) if rank <= 1.5 => EffectiveRankStatusV2::RankOneOrLess,
        Some(rank) if rank < (dimension as f32 * 0.5).max(2.0) => EffectiveRankStatusV2::Low,
        Some(_) => EffectiveRankStatusV2::Adequate,
    };
    let unique_representation_count = representations
        .iter()
        .map(|row| {
            stable_hash_string(&format!(
                "representation-v2:{:?}",
                row.representation
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            ))
        })
        .collect::<BTreeSet<_>>()
        .len();
    let diagnostic_digest = stable_hash_string(&format!(
        "representation-diagnostic-v2:{:?}:{:?}:{}:{:?}:{}:{}",
        per_dimension_finite,
        per_dimension_variance,
        constant_dimension_count,
        effective_rank_status,
        unique_representation_count,
        normalization_status,
    ));
    RepresentationDiagnosticV2 {
        per_dimension_finite,
        per_dimension_variance,
        constant_dimension_count,
        aggregate_variance,
        effective_rank_status,
        unique_representation_count,
        representation_normalization_status: normalization_status,
        diagnostic_digest,
    }
}

fn probability_diagnostic_v2(probabilities: &[f32]) -> ProbabilityDiagnosticV2 {
    let (_, variance) = mean_and_variance(probabilities).unwrap_or((f32::NAN, f32::NAN));
    let entropy = if probabilities.is_empty()
        || probabilities
            .iter()
            .any(|probability| !probability.is_finite())
    {
        f32::NAN
    } else {
        probabilities
            .iter()
            .map(|probability| {
                let probability = probability.clamp(1e-6, 1.0 - 1e-6);
                -probability * probability.ln() - (1.0 - probability) * (1.0 - probability).ln()
            })
            .sum::<f32>()
            / probabilities.len() as f32
    };
    let positive_side_fraction = probabilities.iter().filter(|value| **value >= 0.5).count() as f32
        / probabilities.len().max(1) as f32;
    let low_saturation_fraction = probabilities.iter().filter(|value| **value <= 0.05).count()
        as f32
        / probabilities.len().max(1) as f32;
    let high_saturation_fraction = probabilities.iter().filter(|value| **value >= 0.95).count()
        as f32
        / probabilities.len().max(1) as f32;
    let classify_fraction = |value: f32, limit: f32| {
        if !value.is_finite() {
            "numerical-failure"
        } else if value >= limit {
            "dominated"
        } else {
            "not-dominated"
        }
    };
    let unique_bins = probabilities
        .iter()
        .map(|value| (value * 1000.0).round() as i32)
        .collect::<BTreeSet<_>>()
        .len();
    let minimum = probabilities
        .iter()
        .copied()
        .reduce(f32::min)
        .unwrap_or(f32::NAN);
    let maximum = probabilities
        .iter()
        .copied()
        .reduce(f32::max)
        .unwrap_or(f32::NAN);
    let mut collapse_subtypes = Vec::new();
    if variance_status(variance) != VarianceStatusV2::Adequate || unique_bins < 2 {
        collapse_subtypes.push(MomentumMambaCollapseRootCauseV2::ProbabilityNearConstant);
    }
    if positive_side_fraction >= 0.98 || positive_side_fraction <= 0.02 {
        collapse_subtypes.push(MomentumMambaCollapseRootCauseV2::ProbabilitySingleSided);
    }
    if low_saturation_fraction >= 0.9 {
        collapse_subtypes.push(MomentumMambaCollapseRootCauseV2::ProbabilitySaturatedLow);
    }
    if high_saturation_fraction >= 0.9 {
        collapse_subtypes.push(MomentumMambaCollapseRootCauseV2::ProbabilitySaturatedHigh);
    }
    collapse_subtypes.sort();
    collapse_subtypes.dedup();
    let diagnostic_digest = stable_hash_string(&format!(
        "probability-diagnostic-v2:{:?}:{:?}:{}:{}:{}:{}:{}:{}:{:?}",
        variance_status(variance),
        variance_status(entropy),
        unique_bins,
        classify_fraction(
            positive_side_fraction.max(1.0 - positive_side_fraction),
            0.98
        ),
        classify_fraction(low_saturation_fraction, 0.9),
        classify_fraction(high_saturation_fraction, 0.9),
        minimum.to_bits(),
        maximum.to_bits(),
        collapse_subtypes,
    ));
    ProbabilityDiagnosticV2 {
        variance: variance_status(variance),
        entropy_classification: variance_status(entropy),
        unique_bins,
        positive_side_fraction_classification: classify_fraction(
            positive_side_fraction.max(1.0 - positive_side_fraction),
            0.98,
        ),
        low_saturation_fraction_classification: classify_fraction(low_saturation_fraction, 0.9),
        high_saturation_fraction_classification: classify_fraction(high_saturation_fraction, 0.9),
        minimum_probability_bits: minimum.to_bits(),
        maximum_probability_bits: maximum.to_bits(),
        collapse_subtypes,
        diagnostic_digest,
    }
}

fn class_balance_diagnostic_v2(
    training: &[SequenceExampleV0],
    validation: &[SequenceExampleV0],
) -> ClassBalanceDiagnosticV2 {
    let count = |examples: &[SequenceExampleV0], positive: bool| {
        examples
            .iter()
            .filter(|example| (example.label >= 0.5) == positive)
            .count()
    };
    let training_positive_count = count(training, true);
    let training_negative_count = count(training, false);
    let prior_validation_positive_count = count(validation, true);
    let prior_validation_negative_count = count(validation, false);
    let training_single_class = training_positive_count == 0 || training_negative_count == 0;
    let prior_validation_single_class =
        prior_validation_positive_count == 0 || prior_validation_negative_count == 0;
    let imbalance = |positive: usize, negative: usize| {
        let total = positive + negative;
        total > 0 && positive.max(negative) as f32 / total as f32 >= 0.9
    };
    let imbalance_classification = if training_single_class || prior_validation_single_class {
        "single-class"
    } else if imbalance(training_positive_count, training_negative_count)
        || imbalance(
            prior_validation_positive_count,
            prior_validation_negative_count,
        )
    {
        "dominated"
    } else {
        "balanced-enough"
    };
    let diagnostic_digest = stable_hash_string(&format!(
        "class-balance-diagnostic-v2:{}:{}:{}:{}:{}:{}:{}",
        training_positive_count,
        training_negative_count,
        prior_validation_positive_count,
        prior_validation_negative_count,
        training_single_class,
        prior_validation_single_class,
        imbalance_classification,
    ));
    ClassBalanceDiagnosticV2 {
        training_positive_count,
        training_negative_count,
        prior_validation_positive_count,
        prior_validation_negative_count,
        training_single_class,
        prior_validation_single_class,
        imbalance_classification,
        diagnostic_digest,
    }
}

fn derive_collapse_audit_v2(
    state: &V1FrozenStateV2,
) -> Result<MomentumMambaCollapseAuditV2, String> {
    let mut config = MomentumLearningCampaignConfigV0::default();
    config.agent_id = state.session.agent_id.clone();
    config.initialization_policy = super::HeadInitializationPolicyV0::ColdStartEachWindow;
    config
        .validate()
        .map_err(|_| "V1 diagnostic policy rejected".to_string())?;
    let candles = candles_from_snapshot_prefix(&state.snapshot, state.prior_validation_range.end)?;
    let raw_features = build_momentum_features_v0(&candles, &config.feature_config)
        .map_err(|_| "V1 diagnostic feature derivation failed".to_string())?;
    let training_features = raw_features
        .iter()
        .filter(|row| row.source_index < state.prior_training_range.end)
        .cloned()
        .collect::<Vec<_>>();
    let normalizer = FeatureNormalizerV0::fit(&training_features)
        .map_err(|_| "V1 diagnostic feature normalization failed".to_string())?;
    if normalizer.digest() != state.failed_participant.normalizer_digest {
        return Err("V1 diagnostic feature normalizer identity mismatch".to_string());
    }
    let normalized = normalizer
        .transform(&raw_features)
        .map_err(|_| "V1 diagnostic feature transform failed".to_string())?;
    let examples = build_momentum_sequence_examples_v0(
        &candles,
        &normalized,
        &config.sequence_config,
        std::slice::from_ref(&state.snapshot.snapshot_id),
    )
    .map_err(|_| "V1 diagnostic sequence derivation failed".to_string())?;
    let training = examples_in_range(&examples, &state.prior_training_range);
    let validation = examples_in_range(&examples, &state.prior_validation_range);
    if training.is_empty()
        || validation.len() < config.validation_signal_gate.minimum_samples
        || training
            .iter()
            .any(|example| example.label_index >= state.prior_training_range.end)
        || validation
            .iter()
            .any(|example| example.label_index >= state.prior_validation_range.end)
    {
        return Err("V1 diagnostic evidence partition rejected".to_string());
    }
    let encoder = frozen_mamba3_encoder_from_seed_v0(
        &config.feature_config,
        config.campaign_seed,
        config.backend_preference,
        config.fallback_policy,
    )
    .map_err(|_| "V1 diagnostic encoder unavailable".to_string())?;
    let encoder_digest = encoder.parameter_digest();
    let encoded_training = encoder
        .encode_batch(&training)
        .map_err(|_| "V1 diagnostic training representation failed".to_string())?;
    let encoded_validation = encoder
        .encode_batch(&validation)
        .map_err(|_| "V1 diagnostic validation representation failed".to_string())?;
    let representation = representation_diagnostic_v2(&encoded_validation, "not-applied-v1");
    let initial_head = LogisticPredictionHeadV0::seeded(
        encoded_training[0].representation.len(),
        config.campaign_seed ^ 0x74A1_0001,
    )
    .map_err(|_| "V1 diagnostic head initialization failed".to_string())?;
    let (_, initial_gradient) = brier_loss_and_gradients_v0(&initial_head, &encoded_training)
        .map_err(|_| "V1 diagnostic gradient failed".to_string())?;
    let gradient_norm = (initial_gradient
        .weight_gradients
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        + initial_gradient.bias_gradient.powi(2))
    .sqrt();
    let trained = train_frozen_mamba_head_v0(
        &encoder,
        initial_head.clone(),
        &training,
        &validation,
        &config.training_config,
    )
    .map_err(|_| "V1 diagnostic head replay failed".to_string())?;
    if trained.encoder_digest_before != encoder_digest
        || trained.encoder_digest_after != encoder_digest
        || trained.final_head.parameter_digest() != state.failed_participant.parameter_digest
    {
        return Err("V1 diagnostic replay identity mismatch".to_string());
    }
    let parameter_delta = (trained
        .final_head
        .weights
        .iter()
        .zip(&initial_head.weights)
        .map(|(after, before)| (after - before).powi(2))
        .sum::<f32>()
        + (trained.final_head.bias - initial_head.bias).powi(2))
    .sqrt();
    let optimization = OptimizationDiagnosticV2 {
        initial_parameter_digest: initial_head.parameter_digest(),
        final_parameter_digest: trained.final_head.parameter_digest(),
        finite_parameter_status: trained.final_head.validate().is_ok(),
        parameter_delta_norm: norm_status(parameter_delta),
        gradient_norm: norm_status(gradient_norm),
        update_count: trained.epoch_metrics.len()
            * encoded_training
                .len()
                .div_ceil(config.training_config.batch_size),
        learning_rate_schedule_digest: stable_hash_string(&format!(
            "fixed-learning-rate-v2:{}",
            config.training_config.optimizer.learning_rate.to_bits()
        )),
        loss_trajectory_digest: stable_hash_string(&format!(
            "v1-loss-trajectory-v2:{:?}",
            trained.epoch_metrics
        )),
        early_stop_reason: if trained.stopped_epoch < config.training_config.epochs {
            EarlyStopReasonV2::PatienceExhausted
        } else {
            EarlyStopReasonV2::MaximumEpochs
        },
        diagnostic_digest: String::new(),
    };
    let mut optimization = optimization;
    optimization.diagnostic_digest = stable_hash_string(&format!(
        "optimization-diagnostic-v2:{}:{}:{}:{:?}:{:?}:{}:{}:{}:{:?}",
        optimization.initial_parameter_digest,
        optimization.final_parameter_digest,
        optimization.finite_parameter_status,
        optimization.parameter_delta_norm,
        optimization.gradient_norm,
        optimization.update_count,
        optimization.learning_rate_schedule_digest,
        optimization.loss_trajectory_digest,
        optimization.early_stop_reason,
    ));
    let probabilities = encoded_validation
        .iter()
        .map(|example| trained.final_head.probability(&example.representation))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "V1 diagnostic probability replay failed".to_string())?;
    let probability = probability_diagnostic_v2(&probabilities);
    let class_balance = class_balance_diagnostic_v2(&training, &validation);
    let mut root_causes = Vec::new();
    match representation.aggregate_variance {
        VarianceStatusV2::NearConstant => {
            root_causes.push(MomentumMambaCollapseRootCauseV2::RepresentationNearConstant)
        }
        VarianceStatusV2::Low => {
            root_causes.push(MomentumMambaCollapseRootCauseV2::RepresentationLowVariance)
        }
        VarianceStatusV2::NumericalFailure => {
            root_causes.push(MomentumMambaCollapseRootCauseV2::NumericalFailure)
        }
        VarianceStatusV2::Adequate => {}
    }
    if matches!(
        representation.effective_rank_status,
        EffectiveRankStatusV2::RankOneOrLess | EffectiveRankStatusV2::Low
    ) {
        root_causes.push(MomentumMambaCollapseRootCauseV2::RepresentationLowEffectiveRank);
    }
    if matches!(optimization.parameter_delta_norm, NormStatusV2::NearZero) {
        root_causes.push(MomentumMambaCollapseRootCauseV2::HeadParameterDeltaNearZero);
    } else if matches!(optimization.parameter_delta_norm, NormStatusV2::Low) {
        root_causes.push(MomentumMambaCollapseRootCauseV2::HeadOptimizationStalled);
    }
    if matches!(
        optimization.gradient_norm,
        NormStatusV2::NearZero | NormStatusV2::Low
    ) {
        root_causes.push(MomentumMambaCollapseRootCauseV2::GradientNearZero);
    }
    root_causes.extend(probability.collapse_subtypes.iter().copied());
    if class_balance.imbalance_classification != "balanced-enough" {
        root_causes.push(MomentumMambaCollapseRootCauseV2::ValidationClassImbalanceDominated);
    }
    if !optimization.finite_parameter_status
        || representation
            .per_dimension_finite
            .iter()
            .any(|finite| !finite)
        || probability.variance == VarianceStatusV2::NumericalFailure
    {
        root_causes.push(MomentumMambaCollapseRootCauseV2::NumericalFailure);
    }
    if root_causes.is_empty() {
        root_causes.push(MomentumMambaCollapseRootCauseV2::InsufficientDiagnosticEvidence);
    }
    root_causes.sort();
    root_causes.dedup();
    if root_causes.len() > 1 {
        root_causes.push(MomentumMambaCollapseRootCauseV2::Mixed);
        root_causes.sort();
    }
    let has = |cause| root_causes.contains(&cause);
    let repair_capability_status = if has(MomentumMambaCollapseRootCauseV2::NumericalFailure) {
        MomentumMambaRepairCapabilityStatusV2::TechnicalFailure
    } else if has(MomentumMambaCollapseRootCauseV2::RepresentationNearConstant)
        || has(MomentumMambaCollapseRootCauseV2::RepresentationLowVariance)
        || has(MomentumMambaCollapseRootCauseV2::RepresentationLowEffectiveRank)
    {
        MomentumMambaRepairCapabilityStatusV2::RepairableWithExistingPoolingControls
    } else if has(MomentumMambaCollapseRootCauseV2::HeadOptimizationStalled)
        || has(MomentumMambaCollapseRootCauseV2::HeadParameterDeltaNearZero)
        || has(MomentumMambaCollapseRootCauseV2::GradientNearZero)
    {
        MomentumMambaRepairCapabilityStatusV2::RepairableWithExistingHeadControls
    } else if has(MomentumMambaCollapseRootCauseV2::ProbabilityNearConstant)
        || has(MomentumMambaCollapseRootCauseV2::ProbabilitySingleSided)
        || has(MomentumMambaCollapseRootCauseV2::ProbabilitySaturatedLow)
        || has(MomentumMambaCollapseRootCauseV2::ProbabilitySaturatedHigh)
    {
        MomentumMambaRepairCapabilityStatusV2::RepairableWithBoundedHeadRegularization
    } else {
        MomentumMambaRepairCapabilityStatusV2::UnsupportedRepairRequired
    };
    let mut audit = MomentumMambaCollapseAuditV2 {
        audit_version: AUDIT_VERSION_V2.to_string(),
        agent_id: state.session.agent_id.clone(),
        source_family_digest: state.family.family_digest.clone(),
        failed_participant_digest: state.failed_participant.participant_digest.clone(),
        failed_qualification_receipt_digest: state.failed_receipt.receipt_digest.clone(),
        training_range_digest: range_digest_v2("v1-training", &state.prior_training_range),
        prior_validation_range_digest: range_digest_v2(
            "v1-validation",
            &state.prior_validation_range,
        ),
        encoder_digest,
        representation_normalizer_digest: stable_hash_string(
            "representation-normalizer-not-applied-v1",
        ),
        feature_normalizer_digest: normalizer.digest(),
        head_parameter_digest: trained.final_head.parameter_digest(),
        representation_diagnostic_digest: representation.diagnostic_digest,
        optimization_diagnostic_digest: optimization.diagnostic_digest,
        probability_diagnostic_digest: probability.diagnostic_digest,
        class_balance_diagnostic_digest: class_balance.diagnostic_digest,
        root_causes,
        repair_capability_status,
        audit_digest: String::new(),
    };
    audit.audit_digest = audit_digest_v2(&audit);
    validate_audit_v2(&audit)?;
    Ok(audit)
}

fn ranges_overlap(left: &IndexRangeV0, right: &IndexRangeV0) -> usize {
    left.end
        .min(right.end)
        .saturating_sub(left.start.max(right.start))
}

fn derive_repair_split_v2(
    state: &V1FrozenStateV2,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<MomentumMambaRepairSplitV2, MomentumMambaRepairCapabilityStatusV2> {
    let config = MomentumLearningCampaignConfigV0::default();
    let (
        repair_training_range,
        repair_purge_range,
        fresh_repair_validation_range,
        remaining_reserved_range,
    ) = bounded_repair_ranges_v2(&state.prior_reserved_range, &config)?;
    let prospective_overlap_count = match state.snapshot.actual_start_timestamp_ms {
        Some(start) => (fresh_repair_validation_range.start..fresh_repair_validation_range.end)
            .filter_map(|index| {
                u64::try_from(index)
                    .ok()
                    .and_then(|index| index.checked_mul(reservation.cadence_ms))
                    .and_then(|offset| start.checked_add(offset))
            })
            .filter(|timestamp| reservation.reserved_timestamp_ms.contains(timestamp))
            .count(),
        None => 1,
    };
    let mut split = MomentumMambaRepairSplitV2 {
        split_version: SPLIT_VERSION_V2.to_string(),
        source_snapshot_digest: state.snapshot.content_digest.clone(),
        prior_usage_ledger_digest: state.usage_ledger.ledger_digest.clone(),
        repair_training_range,
        repair_purge_range,
        fresh_repair_validation_range: fresh_repair_validation_range.clone(),
        remaining_reserved_range,
        label_horizon: config.sequence_config.prediction_horizon,
        minimum_validation_samples: config.validation_signal_gate.minimum_samples,
        prior_validation_overlap_count: ranges_overlap(
            &state.prior_validation_range,
            &fresh_repair_validation_range,
        ),
        prospective_overlap_count,
        future_evaluation_overlap_count: 0,
        split_digest: String::new(),
    };
    split.split_digest = split_digest_v2(&split);
    validate_split_v2(&split)
        .map_err(|_| MomentumMambaRepairCapabilityStatusV2::FreshValidationInsufficient)?;
    Ok(split)
}

fn bounded_repair_ranges_v2(
    prior_reserved_range: &IndexRangeV0,
    config: &MomentumLearningCampaignConfigV0,
) -> Result<
    (
        IndexRangeV0,
        IndexRangeV0,
        IndexRangeV0,
        Option<IndexRangeV0>,
    ),
    MomentumMambaRepairCapabilityStatusV2,
> {
    let feature_history = config
        .feature_config
        .minimum_history()
        .map_err(|_| MomentumMambaRepairCapabilityStatusV2::TechnicalFailure)?;
    let purge_length = feature_history
        .checked_sub(1)
        .and_then(|value| value.checked_add(config.sequence_config.sequence_length - 1))
        .and_then(|value| value.checked_add(config.sequence_config.prediction_horizon))
        .ok_or(MomentumMambaRepairCapabilityStatusV2::TechnicalFailure)?;
    let training_end = prior_reserved_range
        .start
        .checked_add(REPAIR_TRAINING_EXTENSION_ROWS_V2)
        .ok_or(MomentumMambaRepairCapabilityStatusV2::TechnicalFailure)?;
    let validation_start = training_end
        .checked_add(purge_length)
        .ok_or(MomentumMambaRepairCapabilityStatusV2::TechnicalFailure)?;
    let validation_end = validation_start
        .checked_add(config.validation_rows)
        .ok_or(MomentumMambaRepairCapabilityStatusV2::TechnicalFailure)?;
    if validation_end > prior_reserved_range.end
        || config.validation_rows < config.validation_signal_gate.minimum_samples
    {
        return Err(MomentumMambaRepairCapabilityStatusV2::FreshValidationInsufficient);
    }
    let repair_training_range = IndexRangeV0 {
        start: 0,
        end: training_end,
    };
    let repair_purge_range = IndexRangeV0 {
        start: training_end,
        end: validation_start,
    };
    let fresh_repair_validation_range = IndexRangeV0 {
        start: validation_start,
        end: validation_end,
    };
    let remaining_reserved_range =
        (validation_end < prior_reserved_range.end).then_some(IndexRangeV0 {
            start: validation_end,
            end: prior_reserved_range.end,
        });
    if repair_purge_range.end - repair_purge_range.start < purge_length
        || fresh_repair_validation_range.start < prior_reserved_range.start
        || fresh_repair_validation_range.end > prior_reserved_range.end
    {
        return Err(MomentumMambaRepairCapabilityStatusV2::FreshValidationInsufficient);
    }
    Ok((
        repair_training_range,
        repair_purge_range,
        fresh_repair_validation_range,
        remaining_reserved_range,
    ))
}

fn variant_v2(
    variant_id: &str,
    pooling_policy: SequencePooling,
    learning_rate: f32,
    l2_regularization: f32,
    maximum_epochs: usize,
    initialization_seed: u64,
    state: &V1FrozenStateV2,
) -> Result<MomentumMambaRepairVariantConfigV2, String> {
    let mut value = MomentumMambaRepairVariantConfigV2 {
        variant_id: variant_id.to_string(),
        pooling_policy,
        learning_rate_bits: learning_rate.to_bits(),
        l2_regularization_bits: l2_regularization.to_bits(),
        maximum_epochs,
        class_weight_policy: "none-training-only".to_string(),
        initialization_seed,
        encoder_frozen: true,
        feature_policy_digest: state.session.feature_policy_digest.clone(),
        label_policy_digest: state.session.label_policy_digest.clone(),
        training_policy_digest: stable_hash_string(&format!(
            "bounded-frozen-head-training-v2:{:?}:{}:{}:{}:{}:finite-logit-guard",
            pooling_policy,
            learning_rate.to_bits(),
            l2_regularization.to_bits(),
            maximum_epochs,
            initialization_seed,
        )),
        variant_config_digest: String::new(),
    };
    value.variant_config_digest = variant_digest_v2(&value);
    validate_variant_v2(&value)?;
    Ok(value)
}

fn derive_variant_configs_v2(
    state: &V1FrozenStateV2,
    audit: &MomentumMambaCollapseAuditV2,
) -> Result<Vec<MomentumMambaRepairVariantConfigV2>, String> {
    let config = MomentumLearningCampaignConfigV0::default();
    let control_seed = config.campaign_seed ^ 0x74A1_0001;
    let mut variants = vec![variant_v2(
        "v1-control",
        SequencePooling::LastOutput,
        config.training_config.optimizer.learning_rate,
        config.training_config.optimizer.weight_decay,
        config.training_config.epochs,
        control_seed,
        state,
    )?];
    match audit.repair_capability_status {
        MomentumMambaRepairCapabilityStatusV2::RepairableWithExistingPoolingControls => {
            variants.push(variant_v2(
                "mean-pooling",
                SequencePooling::MeanOutput,
                config.training_config.optimizer.learning_rate,
                0.0,
                config.training_config.epochs,
                control_seed ^ 0x7802,
                state,
            )?);
            variants.push(variant_v2(
                "mean-pooling-low-rate",
                SequencePooling::MeanOutput,
                0.01,
                0.0,
                60,
                control_seed ^ 0x7803,
                state,
            )?);
            variants.push(variant_v2(
                "mean-pooling-l2",
                SequencePooling::MeanOutput,
                0.02,
                0.001,
                60,
                control_seed ^ 0x7804,
                state,
            )?);
        }
        MomentumMambaRepairCapabilityStatusV2::RepairableWithExistingHeadControls => {
            variants.push(variant_v2(
                "low-rate",
                SequencePooling::LastOutput,
                0.01,
                0.0,
                60,
                control_seed ^ 0x7802,
                state,
            )?);
            variants.push(variant_v2(
                "bounded-epochs",
                SequencePooling::LastOutput,
                0.02,
                0.0,
                90,
                control_seed ^ 0x7803,
                state,
            )?);
        }
        MomentumMambaRepairCapabilityStatusV2::RepairableWithBoundedHeadRegularization => {
            variants.push(variant_v2(
                "l2-regularized",
                SequencePooling::LastOutput,
                0.02,
                0.001,
                60,
                control_seed ^ 0x7802,
                state,
            )?);
            variants.push(variant_v2(
                "low-rate-l2",
                SequencePooling::LastOutput,
                0.01,
                0.005,
                90,
                control_seed ^ 0x7803,
                state,
            )?);
        }
        _ => return Err("bounded repair capability unavailable".to_string()),
    }
    if variants.len() > MAXIMUM_REPAIR_VARIANTS_V2 {
        return Err("repair variant cap exceeded".to_string());
    }
    Ok(variants)
}

fn derive_repair_registration_v2(
    state: &V1FrozenStateV2,
    audit: &MomentumMambaCollapseAuditV2,
    split: &MomentumMambaRepairSplitV2,
) -> Result<MomentumMambaRepairRegistrationV2, String> {
    let allowed_variant_configs = derive_variant_configs_v2(state, audit)?;
    let mut registration = MomentumMambaRepairRegistrationV2 {
        registration_version: REGISTRATION_VERSION_V2.to_string(),
        agent_id: state.session.agent_id.clone(),
        source_snapshot_digest: state.snapshot.content_digest.clone(),
        canonical_intent_digest: state.input.input.intent.intent_digest.clone(),
        canonical_view_digest: state.input.input.view.view_digest.clone(),
        source_family_digest: state.family.family_digest.clone(),
        failed_participant_digest: state.failed_participant.participant_digest.clone(),
        collapse_audit_digest: audit.audit_digest.clone(),
        repair_split_digest: split.split_digest.clone(),
        maximum_repair_variants: MAXIMUM_REPAIR_VARIANTS_V2,
        allowed_variant_configs,
        fresh_validation_hidden: true,
        historical_test_forbidden: true,
        future_evaluation_forbidden: true,
        winner_selection_forbidden: true,
        active_promotion_forbidden: true,
        reward_application_forbidden: true,
        registration_digest: String::new(),
    };
    registration.registration_digest = repair_registration_digest_v2(&registration);
    validate_registration_v2(&registration)?;
    Ok(registration)
}

fn metric_is_finite(metric: &EvaluationMetricsV0) -> bool {
    metric.brier_score.is_finite()
        && metric.accuracy.is_finite()
        && metric.positive_label_rate.is_finite()
        && metric.mean_predicted_probability.is_finite()
}

fn private_metric_digest_v2(model_kind: &str, metric: &EvaluationMetricsV0) -> String {
    stable_hash_string(&format!(
        "private-fresh-validation-metric-v2:{model_kind}:{metric:?}"
    ))
}

fn make_participant_and_receipt_v2(
    role: ParticipantQualificationRoleV2,
    model_kind: String,
    variant_config_digest: Option<String>,
    parameter_digest: String,
    feature_normalizer_digest: String,
    encoder_digest: Option<String>,
    training_policy_digest: String,
    initialization_digest: String,
    source_snapshot_digest: &str,
    repair_training_range_digest: &str,
    fresh_validation_range_digest: &str,
    validation_timestamp_digest: &str,
    feature_policy_digest: &str,
    label_policy_digest: &str,
    private_metric_digest: String,
    qualification_status: ValidationQualificationStatusV2,
) -> Result<
    (
        FrozenCandidateParticipantV2,
        ParticipantValidationQualificationV2,
    ),
    String,
> {
    let participant_id = format!(
        "{}-{}",
        AGENT_ID_V2,
        stable_hash_string(&format!(
            "participant-id-v2:{role:?}:{model_kind}:{parameter_digest}:{initialization_digest}"
        ))
    );
    let model_artifact_digest = stable_hash_string(&format!(
        "model-artifact-v2:{model_kind}:{parameter_digest}:{feature_normalizer_digest}:{encoder_digest:?}:{training_policy_digest}"
    ));
    let mut participant = FrozenCandidateParticipantV2 {
        participant_version: PARTICIPANT_VERSION_V2.to_string(),
        participant_id: participant_id.clone(),
        participant_role: role,
        model_kind,
        variant_config_digest,
        source_snapshot_digest: source_snapshot_digest.to_string(),
        repair_training_range_digest: repair_training_range_digest.to_string(),
        fresh_validation_range_digest: fresh_validation_range_digest.to_string(),
        validation_timestamp_digest: validation_timestamp_digest.to_string(),
        model_artifact_digest,
        parameter_digest,
        feature_normalizer_digest,
        encoder_digest,
        feature_policy_digest: feature_policy_digest.to_string(),
        label_policy_digest: label_policy_digest.to_string(),
        training_policy_digest,
        initialization_digest,
        warm_start_from_v1: false,
        v1_head_reused: false,
        fresh_deterministic_initialization: true,
        encoder_frozen: role == ParticipantQualificationRoleV2::LearnedCandidate,
        deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
        participant_digest: String::new(),
    };
    participant.participant_digest = participant_digest_v2(&participant);
    validate_participant_v2(&participant)?;
    let mut receipt = ParticipantValidationQualificationV2 {
        receipt_version: QUALIFICATION_VERSION_V2.to_string(),
        participant_id,
        participant_role: role,
        participant_digest: participant.participant_digest.clone(),
        fresh_validation_range_digest: fresh_validation_range_digest.to_string(),
        qualification_policy_digest: stable_hash_string(&format!(
            "role-aware-fresh-validation-qualification-v2:{role:?}:finite:min-samples:no-validation-updates:no-ranking"
        )),
        private_metric_digest,
        qualification_status,
        validation_parameter_updates: 0,
        historical_test_reads: 0,
        future_evaluation_reads: 0,
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = qualification_digest_v2(&receipt);
    validate_qualification_v2(&receipt)?;
    Ok((participant, receipt))
}

fn learned_qualification_v2(
    metric: &EvaluationMetricsV0,
    probabilities: &[f32],
    representation: &RepresentationDiagnosticV2,
    minimum_samples: usize,
    encoder_unchanged: bool,
) -> ValidationQualificationStatusV2 {
    if metric.sample_count < minimum_samples || probabilities.len() != metric.sample_count {
        return ValidationQualificationStatusV2::RejectedInsufficientValidation;
    }
    if !metric_is_finite(metric)
        || probabilities.iter().any(|value| !value.is_finite())
        || representation
            .per_dimension_finite
            .iter()
            .any(|finite| !finite)
    {
        return ValidationQualificationStatusV2::RejectedNumericalFailure;
    }
    if !encoder_unchanged {
        return ValidationQualificationStatusV2::RejectedPolicyInvariant;
    }
    if matches!(
        representation.aggregate_variance,
        VarianceStatusV2::NearConstant | VarianceStatusV2::Low
    ) || matches!(
        representation.effective_rank_status,
        EffectiveRankStatusV2::RankOneOrLess
    ) {
        return ValidationQualificationStatusV2::RejectedRepresentationCollapse;
    }
    let probability = probability_diagnostic_v2(probabilities);
    if !probability.collapse_subtypes.is_empty() {
        ValidationQualificationStatusV2::RejectedProbabilityCollapse
    } else {
        ValidationQualificationStatusV2::Qualified
    }
}

fn comparator_qualification_v2(
    role: ParticipantQualificationRoleV2,
    metric: &EvaluationMetricsV0,
    probabilities: &[f32],
    minimum_samples: usize,
) -> ValidationQualificationStatusV2 {
    if metric.sample_count < minimum_samples || probabilities.len() != metric.sample_count {
        return ValidationQualificationStatusV2::RejectedInsufficientValidation;
    }
    if !metric_is_finite(metric) || probabilities.iter().any(|value| !value.is_finite()) {
        return ValidationQualificationStatusV2::RejectedNumericalFailure;
    }
    match role {
        ParticipantQualificationRoleV2::LinearComparator => {
            ValidationQualificationStatusV2::Qualified
        }
        ParticipantQualificationRoleV2::ConstantBenchmark => {
            ValidationQualificationStatusV2::BenchmarkQualified
        }
        ParticipantQualificationRoleV2::LearnedCandidate => {
            ValidationQualificationStatusV2::RejectedPolicyInvariant
        }
    }
}

fn run_repair_experiment_v2(
    state: &V1FrozenStateV2,
    audit: &MomentumMambaCollapseAuditV2,
    split: &MomentumMambaRepairSplitV2,
    registration: &MomentumMambaRepairRegistrationV2,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<RepairExperimentV2, String> {
    validate_audit_v2(audit)?;
    validate_split_v2(split)?;
    validate_registration_v2(registration)?;
    if registration.collapse_audit_digest != audit.audit_digest
        || registration.repair_split_digest != split.split_digest
        || registration.source_family_digest != state.family.family_digest
        || registration.failed_participant_digest != state.failed_participant.participant_digest
    {
        return Err("repair execution preregistration binding rejected".to_string());
    }
    let config = MomentumLearningCampaignConfigV0::default();
    let candles = candles_from_snapshot_prefix(&state.snapshot, state.snapshot.row_count)?;
    let raw_features = build_momentum_features_v0(&candles, &config.feature_config)
        .map_err(|_| "repair feature derivation failed".to_string())?;
    let repair_training_features = raw_features
        .iter()
        .filter(|row| row.source_index < split.repair_training_range.end)
        .cloned()
        .collect::<Vec<_>>();
    let feature_normalizer = FeatureNormalizerV0::fit(&repair_training_features)
        .map_err(|_| "repair feature normalization failed".to_string())?;
    if feature_normalizer.fitted_on_end >= split.repair_training_range.end {
        return Err("repair normalizer crossed training boundary".to_string());
    }
    let normalized = feature_normalizer
        .transform(&raw_features)
        .map_err(|_| "repair feature transform failed".to_string())?;
    let all_examples = build_momentum_sequence_examples_v0(
        &candles,
        &normalized,
        &config.sequence_config,
        std::slice::from_ref(&state.snapshot.snapshot_id),
    )
    .map_err(|_| "repair sequence derivation failed".to_string())?;
    let training = examples_in_range(&all_examples, &split.repair_training_range);
    let validation = examples_in_range(&all_examples, &split.fresh_repair_validation_range);
    if training.is_empty()
        || validation.len() < split.minimum_validation_samples
        || training
            .iter()
            .any(|example| example.label_index >= split.repair_training_range.end)
        || validation.iter().any(|example| {
            example.sequence_start < split.fresh_repair_validation_range.start
                || example.label_index >= split.fresh_repair_validation_range.end
        })
    {
        return Err("fresh repair validation partition rejected".to_string());
    }
    let validation_indices = validation
        .iter()
        .map(|example| example.label_index)
        .collect::<Vec<_>>();
    let validation_timestamp_digest = stable_hash_string(&format!(
        "fresh-validation-timestamps-v2:{:?}",
        validation_indices
            .iter()
            .map(|index| state.snapshot.normalized_dataset.rows[*index].timestamp_ms)
            .collect::<Vec<_>>()
    ));
    let training_range_digest = range_digest_v2("repair-training", &split.repair_training_range);
    let validation_range_digest = range_digest_v2(
        "fresh-repair-validation",
        &split.fresh_repair_validation_range,
    );
    let normalizer_digest = feature_normalizer.digest();
    let mut participants = Vec::new();
    let mut receipts = Vec::new();
    for variant in &registration.allowed_variant_configs {
        validate_variant_v2(variant)?;
        let mut encoder = frozen_mamba3_encoder_from_seed_v0(
            &config.feature_config,
            config.campaign_seed,
            config.backend_preference,
            config.fallback_policy,
        )
        .map_err(|_| "repair encoder unavailable".to_string())?;
        encoder.pooling = variant.pooling_policy;
        let encoder_digest_before = encoder.parameter_digest();
        let representation_dimension = encoder
            .encode_sequence(&training[0].input)
            .map_err(|_| "repair representation unavailable".to_string())?
            .representation
            .len();
        let initial_head =
            LogisticPredictionHeadV0::seeded(representation_dimension, variant.initialization_seed)
                .map_err(|_| "repair head initialization failed".to_string())?;
        let initial_head_digest = initial_head.parameter_digest();
        let mut training_config = HeadTrainingConfigV0::default();
        training_config.epochs = variant.maximum_epochs;
        training_config.seed = variant.initialization_seed;
        training_config.early_stopping_patience = None;
        training_config.optimizer.learning_rate = f32::from_bits(variant.learning_rate_bits);
        training_config.optimizer.weight_decay = f32::from_bits(variant.l2_regularization_bits);
        training_config
            .validate()
            .map_err(|_| "repair head policy rejected".to_string())?;
        let trained = train_frozen_mamba_head_v0(
            &encoder,
            initial_head,
            &training,
            &training,
            &training_config,
        )
        .map_err(|_| "repair head training failed".to_string())?;
        let encoder_unchanged = trained.encoder_digest_before == encoder_digest_before
            && trained.encoder_digest_after == encoder_digest_before;
        let encoded_validation = encoder
            .encode_batch(&validation)
            .map_err(|_| "fresh repair representation failed".to_string())?;
        let metric = evaluate_head_v0(&trained.final_head, &encoded_validation)
            .map_err(|_| "fresh repair metric failed".to_string())?;
        let probabilities = encoded_validation
            .iter()
            .map(|example| trained.final_head.probability(&example.representation))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "fresh repair probability failed".to_string())?;
        let representation = representation_diagnostic_v2(&encoded_validation, "not-applied-v2");
        let qualification_status = learned_qualification_v2(
            &metric,
            &probabilities,
            &representation,
            split.minimum_validation_samples,
            encoder_unchanged,
        );
        let (participant, receipt) = make_participant_and_receipt_v2(
            ParticipantQualificationRoleV2::LearnedCandidate,
            format!("FrozenMambaHeadV2/{}", variant.variant_id),
            Some(variant.variant_config_digest.clone()),
            trained.final_head.parameter_digest(),
            normalizer_digest.clone(),
            Some(encoder_digest_before),
            variant.training_policy_digest.clone(),
            stable_hash_string(&format!(
                "fresh-deterministic-head-initialization-v2:{}:{}",
                variant.initialization_seed, initial_head_digest
            )),
            &state.snapshot.content_digest,
            &training_range_digest,
            &validation_range_digest,
            &validation_timestamp_digest,
            &state.session.feature_policy_digest,
            &state.session.label_policy_digest,
            private_metric_digest_v2(&variant.variant_id, &metric),
            qualification_status,
        )?;
        participants.push(participant);
        receipts.push(receipt);
    }
    let mut linear_config = HeadTrainingConfigV0::default();
    linear_config.seed = config.campaign_seed ^ 0x78A1_0002;
    linear_config.early_stopping_patience = None;
    let linear = LinearMomentumBaselineV0::train(&training, &training, &linear_config)
        .map_err(|_| "V2 linear comparator training failed".to_string())?;
    let linear_metric = linear
        .evaluate(&validation)
        .map_err(|_| "V2 linear comparator evaluation failed".to_string())?;
    let linear_probabilities = validation
        .iter()
        .map(|example| {
            example
                .input
                .last()
                .ok_or_else(|| "V2 linear validation input unavailable".to_string())
                .and_then(|representation| {
                    linear
                        .head
                        .probability(representation)
                        .map_err(|_| "V2 linear probability failed".to_string())
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let linear_status = comparator_qualification_v2(
        ParticipantQualificationRoleV2::LinearComparator,
        &linear_metric,
        &linear_probabilities,
        split.minimum_validation_samples,
    );
    let (linear_participant, linear_receipt) = make_participant_and_receipt_v2(
        ParticipantQualificationRoleV2::LinearComparator,
        "LinearMomentumBaselineV2".to_string(),
        None,
        linear.head.parameter_digest(),
        normalizer_digest.clone(),
        None,
        stable_hash_string(&format!(
            "linear-training-policy-v2:{}",
            linear_config.digest()
        )),
        stable_hash_string(&format!(
            "fresh-linear-initialization-v2:{}",
            linear_config.seed
        )),
        &state.snapshot.content_digest,
        &training_range_digest,
        &validation_range_digest,
        &validation_timestamp_digest,
        &state.session.feature_policy_digest,
        &state.session.label_policy_digest,
        private_metric_digest_v2("LinearMomentumBaselineV2", &linear_metric),
        linear_status,
    )?;
    participants.push(linear_participant);
    receipts.push(linear_receipt);
    let constant = ConstantProbabilityBaselineV0::fit(&training)
        .map_err(|_| "V2 constant benchmark training failed".to_string())?;
    let constant_metric = constant
        .evaluate(&validation)
        .map_err(|_| "V2 constant benchmark evaluation failed".to_string())?;
    let constant_probabilities = vec![constant.probability; validation.len()];
    let constant_status = comparator_qualification_v2(
        ParticipantQualificationRoleV2::ConstantBenchmark,
        &constant_metric,
        &constant_probabilities,
        split.minimum_validation_samples,
    );
    let constant_parameter_digest = stable_hash_string(&format!(
        "training-only-prevalence-constant-v2:{}:{}",
        constant.probability.to_bits(),
        training.len()
    ));
    let (constant_participant, constant_receipt) = make_participant_and_receipt_v2(
        ParticipantQualificationRoleV2::ConstantBenchmark,
        "ConstantProbabilityBaselineV2".to_string(),
        None,
        constant_parameter_digest,
        normalizer_digest,
        None,
        stable_hash_string("constant-training-prevalence-policy-v2"),
        stable_hash_string(&format!("fresh-constant-fit-v2:{}", training.len())),
        &state.snapshot.content_digest,
        &training_range_digest,
        &validation_range_digest,
        &validation_timestamp_digest,
        &state.session.feature_policy_digest,
        &state.session.label_policy_digest,
        private_metric_digest_v2("ConstantProbabilityBaselineV2", &constant_metric),
        constant_status,
    )?;
    participants.push(constant_participant);
    receipts.push(constant_receipt);
    participants.sort_by(|left, right| left.participant_id.cmp(&right.participant_id));
    receipts.sort_by(|left, right| left.participant_digest.cmp(&right.participant_digest));
    let learned_participant_count = participants
        .iter()
        .filter(|participant| {
            participant.participant_role == ParticipantQualificationRoleV2::LearnedCandidate
        })
        .count();
    let qualified_learned_participant_count = receipts
        .iter()
        .filter(|receipt| {
            receipt.participant_role == ParticipantQualificationRoleV2::LearnedCandidate
                && receipt.qualification_status == ValidationQualificationStatusV2::Qualified
        })
        .count();
    let qualified_comparator_count = receipts
        .iter()
        .filter(|receipt| {
            receipt.participant_role != ParticipantQualificationRoleV2::LearnedCandidate
                && matches!(
                    receipt.qualification_status,
                    ValidationQualificationStatusV2::Qualified
                        | ValidationQualificationStatusV2::BenchmarkQualified
                )
        })
        .count();
    let mut family = MomentumCandidateFamilyV2 {
        family_version: FAMILY_VERSION_V2.to_string(),
        agent_id: AGENT_ID_V2.to_string(),
        source_snapshot_digest: state.snapshot.content_digest.clone(),
        canonical_view_digest: state.input.input.view.view_digest.clone(),
        repair_registration_digest: registration.registration_digest.clone(),
        repair_split_digest: split.split_digest.clone(),
        collapse_audit_digest: audit.audit_digest.clone(),
        participants,
        qualification_receipts: receipts,
        learned_participant_count,
        qualified_learned_participant_count,
        qualified_comparator_count,
        winner_selected: false,
        historical_test_accessed: false,
        eligible_for_active_committee: false,
        eligible_for_promotion: false,
        eligible_for_reward: false,
        family_digest: String::new(),
    };
    family.family_digest = family_digest_v2(&family);
    validate_family_v2(&family)?;
    let (roster, roster_status) = derive_roster_v2(&family)?;
    let (evaluation_registration, evaluation_registration_status) = if let Some(roster) = &roster {
        let registration =
            derive_evaluation_registration_v2(state, split, &family, roster, reservation)?;
        (
            Some(registration),
            MomentumFutureEvaluationRegistrationStatusV2::Registered,
        )
    } else {
        let status = match roster_status {
            MomentumFutureEvaluationRosterStatusV2::NoQualifiedLearnedParticipant => {
                MomentumFutureEvaluationRegistrationStatusV2::NoQualifiedLearnedParticipant
            }
            MomentumFutureEvaluationRosterStatusV2::InsufficientComparators => {
                MomentumFutureEvaluationRegistrationStatusV2::InsufficientComparators
            }
            MomentumFutureEvaluationRosterStatusV2::Registered => {
                MomentumFutureEvaluationRegistrationStatusV2::SafetyContractInvalid
            }
        };
        (None, status)
    };
    Ok(RepairExperimentV2 {
        family,
        roster,
        roster_status,
        evaluation_registration,
        evaluation_registration_status,
    })
}

fn derive_roster_v2(
    family: &MomentumCandidateFamilyV2,
) -> Result<
    (
        Option<MomentumFutureEvaluationRosterV2>,
        MomentumFutureEvaluationRosterStatusV2,
    ),
    String,
> {
    validate_family_v2(family)?;
    let mut qualified_learned_participant_digests = Vec::new();
    let mut qualified_comparator_digests = Vec::new();
    let mut excluded_participant_digests = Vec::new();
    for receipt in &family.qualification_receipts {
        match (receipt.participant_role, receipt.qualification_status) {
            (
                ParticipantQualificationRoleV2::LearnedCandidate,
                ValidationQualificationStatusV2::Qualified,
            ) => qualified_learned_participant_digests.push(receipt.participant_digest.clone()),
            (
                ParticipantQualificationRoleV2::LinearComparator,
                ValidationQualificationStatusV2::Qualified,
            )
            | (
                ParticipantQualificationRoleV2::ConstantBenchmark,
                ValidationQualificationStatusV2::BenchmarkQualified,
            ) => qualified_comparator_digests.push(receipt.participant_digest.clone()),
            _ => excluded_participant_digests.push(receipt.participant_digest.clone()),
        }
    }
    qualified_learned_participant_digests =
        sorted_unique_strings(qualified_learned_participant_digests);
    qualified_comparator_digests = sorted_unique_strings(qualified_comparator_digests);
    excluded_participant_digests = sorted_unique_strings(excluded_participant_digests);
    if qualified_learned_participant_digests.is_empty() {
        return Ok((
            None,
            MomentumFutureEvaluationRosterStatusV2::NoQualifiedLearnedParticipant,
        ));
    }
    if qualified_comparator_digests.is_empty() {
        return Ok((
            None,
            MomentumFutureEvaluationRosterStatusV2::InsufficientComparators,
        ));
    }
    let mut roster = MomentumFutureEvaluationRosterV2 {
        roster_version: ROSTER_VERSION_V2.to_string(),
        family_digest: family.family_digest.clone(),
        qualified_learned_participant_digests,
        qualified_comparator_digests,
        excluded_participant_digests,
        inclusion_policy_digest: stable_hash_string(
            "future-roster-v2:all-qualified-learned:all-qualified-comparators:no-ranking",
        ),
        roster_digest: String::new(),
    };
    roster.roster_digest = roster_digest_v2(&roster);
    validate_roster_v2(&roster, family)?;
    Ok((
        Some(roster),
        MomentumFutureEvaluationRosterStatusV2::Registered,
    ))
}

fn derive_evaluation_registration_v2(
    state: &V1FrozenStateV2,
    split: &MomentumMambaRepairSplitV2,
    family: &MomentumCandidateFamilyV2,
    roster: &MomentumFutureEvaluationRosterV2,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<MomentumFutureEvaluationRegistrationV2, String> {
    validate_roster_v2(roster, family)?;
    let source_boundary_timestamp_ms = state
        .snapshot
        .actual_end_timestamp_ms
        .ok_or_else(|| "Momentum source boundary unavailable".to_string())?;
    let protected_next = reservation
        .reserved_timestamp_ms
        .last()
        .and_then(|timestamp| timestamp.checked_add(reservation.cadence_ms))
        .ok_or_else(|| "protected future boundary unavailable".to_string())?;
    let source_next = source_boundary_timestamp_ms
        .checked_add(reservation.cadence_ms)
        .ok_or_else(|| "Momentum source boundary overflow".to_string())?;
    let minimum_accepted_timestamp_ms = source_next
        .max(protected_next)
        .max(reservation.provider_finality_boundary_ms);
    let included = roster
        .qualified_learned_participant_digests
        .iter()
        .chain(&roster.qualified_comparator_digests)
        .collect::<BTreeSet<_>>();
    let qualification_receipt_digests = sorted_unique_strings(
        family
            .qualification_receipts
            .iter()
            .filter(|receipt| included.contains(&receipt.participant_digest))
            .map(|receipt| receipt.receipt_digest.clone())
            .collect(),
    );
    let mut prior_reserved_range_digests = vec![range_digest_v2(
        "v1-reserved-retrospective-unused",
        &state.prior_reserved_range,
    )];
    if let Some(range) = &split.remaining_reserved_range {
        prior_reserved_range_digests.push(range_digest_v2(
            "v2-remaining-reserved-retrospective-unused",
            range,
        ));
    }
    prior_reserved_range_digests.extend(reservation.reserved_timestamp_ms.iter().map(
        |timestamp| {
            stable_hash_string(&format!(
                "protected-prospective-timestamp-range-v1:{timestamp}:{}",
                reservation.cadence_ms
            ))
        },
    ));
    prior_reserved_range_digests = sorted_unique_strings(prior_reserved_range_digests);
    let mut registration = MomentumFutureEvaluationRegistrationV2 {
        registration_version: EVALUATION_VERSION_V2.to_string(),
        agent_id: AGENT_ID_V2.to_string(),
        family_digest: family.family_digest.clone(),
        roster_digest: roster.roster_digest.clone(),
        repair_registration_digest: family.repair_registration_digest.clone(),
        collapse_audit_digest: family.collapse_audit_digest.clone(),
        qualification_receipt_digests,
        source_snapshot_digest: state.snapshot.content_digest.clone(),
        source_boundary_timestamp_ms,
        protected_registration_digests: reservation.protected_registration_digests.clone(),
        protected_timestamp_ms: reservation.reserved_timestamp_ms.clone(),
        prior_reserved_range_digests,
        provider_finality_boundary_ms: reservation.provider_finality_boundary_ms,
        minimum_accepted_timestamp_ms,
        labels_hidden_until_opening: true,
        probabilities_hidden_until_opening: true,
        one_time_opening_required: true,
        winner_selection_forbidden_before_opening: true,
        active_promotion_forbidden: true,
        reward_application_forbidden: true,
        maximum_requests: 1,
        maximum_concurrency: 1,
        maximum_retries: 0,
        status: MomentumFutureEvaluationRegistrationStatusV2::Registered,
        registration_digest: String::new(),
    };
    registration.registration_digest = evaluation_registration_digest_v2(&registration);
    validate_evaluation_registration_v2(&registration, family, roster, reservation)?;
    Ok(registration)
}

#[derive(Clone, PartialEq, Message)]
struct RangeProtobufV2 {
    #[prost(uint64, tag = "1")]
    start: u64,
    #[prost(uint64, tag = "2")]
    end: u64,
}

#[derive(Clone, PartialEq, Message)]
struct CollapseAuditProtobufV2 {
    #[prost(string, tag = "1")]
    audit_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    source_family_digest: String,
    #[prost(string, tag = "4")]
    failed_participant_digest: String,
    #[prost(string, tag = "5")]
    failed_qualification_receipt_digest: String,
    #[prost(string, tag = "6")]
    training_range_digest: String,
    #[prost(string, tag = "7")]
    prior_validation_range_digest: String,
    #[prost(string, tag = "8")]
    encoder_digest: String,
    #[prost(string, tag = "9")]
    representation_normalizer_digest: String,
    #[prost(string, tag = "10")]
    feature_normalizer_digest: String,
    #[prost(string, tag = "11")]
    head_parameter_digest: String,
    #[prost(string, tag = "12")]
    representation_diagnostic_digest: String,
    #[prost(string, tag = "13")]
    optimization_diagnostic_digest: String,
    #[prost(string, tag = "14")]
    probability_diagnostic_digest: String,
    #[prost(string, tag = "15")]
    class_balance_diagnostic_digest: String,
    #[prost(string, repeated, tag = "16")]
    root_causes: Vec<String>,
    #[prost(string, tag = "17")]
    repair_capability_status: String,
    #[prost(string, tag = "18")]
    audit_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct RepairSplitProtobufV2 {
    #[prost(string, tag = "1")]
    split_version: String,
    #[prost(string, tag = "2")]
    source_snapshot_digest: String,
    #[prost(string, tag = "3")]
    prior_usage_ledger_digest: String,
    #[prost(message, optional, tag = "4")]
    repair_training_range: Option<RangeProtobufV2>,
    #[prost(message, optional, tag = "5")]
    repair_purge_range: Option<RangeProtobufV2>,
    #[prost(message, optional, tag = "6")]
    fresh_repair_validation_range: Option<RangeProtobufV2>,
    #[prost(message, optional, tag = "7")]
    remaining_reserved_range: Option<RangeProtobufV2>,
    #[prost(uint64, tag = "8")]
    label_horizon: u64,
    #[prost(uint64, tag = "9")]
    minimum_validation_samples: u64,
    #[prost(uint64, tag = "10")]
    prior_validation_overlap_count: u64,
    #[prost(uint64, tag = "11")]
    prospective_overlap_count: u64,
    #[prost(uint64, tag = "12")]
    future_evaluation_overlap_count: u64,
    #[prost(string, tag = "13")]
    split_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct RepairVariantProtobufV2 {
    #[prost(string, tag = "1")]
    variant_id: String,
    #[prost(string, tag = "2")]
    pooling_policy: String,
    #[prost(uint32, tag = "3")]
    learning_rate_bits: u32,
    #[prost(uint32, tag = "4")]
    l2_regularization_bits: u32,
    #[prost(uint64, tag = "5")]
    maximum_epochs: u64,
    #[prost(string, tag = "6")]
    class_weight_policy: String,
    #[prost(uint64, tag = "7")]
    initialization_seed: u64,
    #[prost(bool, tag = "8")]
    encoder_frozen: bool,
    #[prost(string, tag = "9")]
    feature_policy_digest: String,
    #[prost(string, tag = "10")]
    label_policy_digest: String,
    #[prost(string, tag = "11")]
    training_policy_digest: String,
    #[prost(string, tag = "12")]
    variant_config_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct RepairRegistrationProtobufV2 {
    #[prost(string, tag = "1")]
    registration_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    source_snapshot_digest: String,
    #[prost(string, tag = "4")]
    canonical_intent_digest: String,
    #[prost(string, tag = "5")]
    canonical_view_digest: String,
    #[prost(string, tag = "6")]
    source_family_digest: String,
    #[prost(string, tag = "7")]
    failed_participant_digest: String,
    #[prost(string, tag = "8")]
    collapse_audit_digest: String,
    #[prost(string, tag = "9")]
    repair_split_digest: String,
    #[prost(message, repeated, tag = "10")]
    allowed_variant_configs: Vec<RepairVariantProtobufV2>,
    #[prost(uint64, tag = "11")]
    maximum_repair_variants: u64,
    #[prost(bool, tag = "12")]
    fresh_validation_hidden: bool,
    #[prost(bool, tag = "13")]
    historical_test_forbidden: bool,
    #[prost(bool, tag = "14")]
    future_evaluation_forbidden: bool,
    #[prost(bool, tag = "15")]
    winner_selection_forbidden: bool,
    #[prost(bool, tag = "16")]
    active_promotion_forbidden: bool,
    #[prost(bool, tag = "17")]
    reward_application_forbidden: bool,
    #[prost(string, tag = "18")]
    registration_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct ParticipantProtobufV2 {
    #[prost(string, tag = "1")]
    participant_version: String,
    #[prost(string, tag = "2")]
    participant_id: String,
    #[prost(string, tag = "3")]
    participant_role: String,
    #[prost(string, tag = "4")]
    model_kind: String,
    #[prost(string, optional, tag = "5")]
    variant_config_digest: Option<String>,
    #[prost(string, tag = "6")]
    source_snapshot_digest: String,
    #[prost(string, tag = "7")]
    repair_training_range_digest: String,
    #[prost(string, tag = "8")]
    fresh_validation_range_digest: String,
    #[prost(string, tag = "9")]
    validation_timestamp_digest: String,
    #[prost(string, tag = "10")]
    model_artifact_digest: String,
    #[prost(string, tag = "11")]
    parameter_digest: String,
    #[prost(string, tag = "12")]
    feature_normalizer_digest: String,
    #[prost(string, optional, tag = "13")]
    encoder_digest: Option<String>,
    #[prost(string, tag = "14")]
    feature_policy_digest: String,
    #[prost(string, tag = "15")]
    label_policy_digest: String,
    #[prost(string, tag = "16")]
    training_policy_digest: String,
    #[prost(string, tag = "17")]
    initialization_digest: String,
    #[prost(bool, tag = "18")]
    warm_start_from_v1: bool,
    #[prost(bool, tag = "19")]
    v1_head_reused: bool,
    #[prost(bool, tag = "20")]
    fresh_deterministic_initialization: bool,
    #[prost(bool, tag = "21")]
    encoder_frozen: bool,
    #[prost(string, tag = "22")]
    deployment_status: String,
    #[prost(string, tag = "23")]
    participant_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct QualificationProtobufV2 {
    #[prost(string, tag = "1")]
    receipt_version: String,
    #[prost(string, tag = "2")]
    participant_id: String,
    #[prost(string, tag = "3")]
    participant_role: String,
    #[prost(string, tag = "4")]
    participant_digest: String,
    #[prost(string, tag = "5")]
    fresh_validation_range_digest: String,
    #[prost(string, tag = "6")]
    qualification_policy_digest: String,
    #[prost(string, tag = "7")]
    private_metric_digest: String,
    #[prost(string, tag = "8")]
    qualification_status: String,
    #[prost(uint64, tag = "9")]
    validation_parameter_updates: u64,
    #[prost(uint64, tag = "10")]
    historical_test_reads: u64,
    #[prost(uint64, tag = "11")]
    future_evaluation_reads: u64,
    #[prost(string, tag = "12")]
    receipt_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct FamilyProtobufV2 {
    #[prost(string, tag = "1")]
    family_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    source_snapshot_digest: String,
    #[prost(string, tag = "4")]
    canonical_view_digest: String,
    #[prost(string, tag = "5")]
    repair_registration_digest: String,
    #[prost(string, tag = "6")]
    repair_split_digest: String,
    #[prost(string, tag = "7")]
    collapse_audit_digest: String,
    #[prost(message, repeated, tag = "8")]
    participants: Vec<ParticipantProtobufV2>,
    #[prost(message, repeated, tag = "9")]
    qualification_receipts: Vec<QualificationProtobufV2>,
    #[prost(uint64, tag = "10")]
    learned_participant_count: u64,
    #[prost(uint64, tag = "11")]
    qualified_learned_participant_count: u64,
    #[prost(uint64, tag = "12")]
    qualified_comparator_count: u64,
    #[prost(bool, tag = "13")]
    winner_selected: bool,
    #[prost(bool, tag = "14")]
    historical_test_accessed: bool,
    #[prost(bool, tag = "15")]
    eligible_for_active_committee: bool,
    #[prost(bool, tag = "16")]
    eligible_for_promotion: bool,
    #[prost(bool, tag = "17")]
    eligible_for_reward: bool,
    #[prost(string, tag = "18")]
    family_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct RosterProtobufV2 {
    #[prost(string, tag = "1")]
    roster_version: String,
    #[prost(string, tag = "2")]
    family_digest: String,
    #[prost(string, repeated, tag = "3")]
    qualified_learned_participant_digests: Vec<String>,
    #[prost(string, repeated, tag = "4")]
    qualified_comparator_digests: Vec<String>,
    #[prost(string, repeated, tag = "5")]
    excluded_participant_digests: Vec<String>,
    #[prost(string, tag = "6")]
    inclusion_policy_digest: String,
    #[prost(string, tag = "7")]
    roster_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct EvaluationRegistrationProtobufV2 {
    #[prost(string, tag = "1")]
    registration_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    family_digest: String,
    #[prost(string, tag = "4")]
    roster_digest: String,
    #[prost(string, tag = "5")]
    repair_registration_digest: String,
    #[prost(string, tag = "6")]
    collapse_audit_digest: String,
    #[prost(string, repeated, tag = "7")]
    qualification_receipt_digests: Vec<String>,
    #[prost(string, tag = "8")]
    source_snapshot_digest: String,
    #[prost(uint64, tag = "9")]
    source_boundary_timestamp_ms: u64,
    #[prost(string, repeated, tag = "10")]
    protected_registration_digests: Vec<String>,
    #[prost(uint64, repeated, tag = "11")]
    protected_timestamp_ms: Vec<u64>,
    #[prost(string, repeated, tag = "12")]
    prior_reserved_range_digests: Vec<String>,
    #[prost(uint64, tag = "13")]
    provider_finality_boundary_ms: u64,
    #[prost(uint64, tag = "14")]
    minimum_accepted_timestamp_ms: u64,
    #[prost(bool, tag = "15")]
    labels_hidden_until_opening: bool,
    #[prost(bool, tag = "16")]
    probabilities_hidden_until_opening: bool,
    #[prost(bool, tag = "17")]
    one_time_opening_required: bool,
    #[prost(bool, tag = "18")]
    winner_selection_forbidden_before_opening: bool,
    #[prost(bool, tag = "19")]
    active_promotion_forbidden: bool,
    #[prost(bool, tag = "20")]
    reward_application_forbidden: bool,
    #[prost(uint64, tag = "21")]
    maximum_requests: u64,
    #[prost(uint64, tag = "22")]
    maximum_concurrency: u64,
    #[prost(uint64, tag = "23")]
    maximum_retries: u64,
    #[prost(string, tag = "24")]
    status: String,
    #[prost(string, tag = "25")]
    registration_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct JournalProtobufV2 {
    #[prost(string, tag = "1")]
    journal_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    collapse_audit_digest: String,
    #[prost(string, tag = "4")]
    repair_split_digest: String,
    #[prost(string, tag = "5")]
    repair_registration_digest: String,
    #[prost(string, optional, tag = "6")]
    family_digest: Option<String>,
    #[prost(string, optional, tag = "7")]
    roster_digest: Option<String>,
    #[prost(string, optional, tag = "8")]
    evaluation_registration_digest: Option<String>,
    #[prost(bool, tag = "9")]
    prior_validation_used_for_repair_qualification: bool,
    #[prost(bool, tag = "10")]
    warm_start_from_v1: bool,
    #[prost(bool, tag = "11")]
    v1_head_reused: bool,
    #[prost(bool, tag = "12")]
    fresh_deterministic_initialization: bool,
    #[prost(string, tag = "13")]
    status: String,
    #[prost(string, tag = "14")]
    journal_digest: String,
}

fn usize_v2(value: u64) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| "V2 Protobuf integer overflow".to_string())
}

fn range_to_protobuf_v2(value: &IndexRangeV0) -> Result<RangeProtobufV2, String> {
    Ok(RangeProtobufV2 {
        start: u64::try_from(value.start).map_err(|_| "V2 range start overflow".to_string())?,
        end: u64::try_from(value.end).map_err(|_| "V2 range end overflow".to_string())?,
    })
}

fn range_from_protobuf_v2(value: Option<RangeProtobufV2>) -> Result<IndexRangeV0, String> {
    let value = value.ok_or_else(|| "V2 Protobuf range unavailable".to_string())?;
    Ok(IndexRangeV0 {
        start: usize_v2(value.start)?,
        end: usize_v2(value.end)?,
    })
}

fn parse_root_cause_v2(value: &str) -> Result<MomentumMambaCollapseRootCauseV2, String> {
    match value {
        "RepresentationNearConstant" => {
            Ok(MomentumMambaCollapseRootCauseV2::RepresentationNearConstant)
        }
        "RepresentationLowVariance" => {
            Ok(MomentumMambaCollapseRootCauseV2::RepresentationLowVariance)
        }
        "RepresentationLowEffectiveRank" => {
            Ok(MomentumMambaCollapseRootCauseV2::RepresentationLowEffectiveRank)
        }
        "HeadOptimizationStalled" => Ok(MomentumMambaCollapseRootCauseV2::HeadOptimizationStalled),
        "HeadParameterDeltaNearZero" => {
            Ok(MomentumMambaCollapseRootCauseV2::HeadParameterDeltaNearZero)
        }
        "GradientNearZero" => Ok(MomentumMambaCollapseRootCauseV2::GradientNearZero),
        "ProbabilityNearConstant" => Ok(MomentumMambaCollapseRootCauseV2::ProbabilityNearConstant),
        "ProbabilitySingleSided" => Ok(MomentumMambaCollapseRootCauseV2::ProbabilitySingleSided),
        "ProbabilitySaturatedLow" => Ok(MomentumMambaCollapseRootCauseV2::ProbabilitySaturatedLow),
        "ProbabilitySaturatedHigh" => {
            Ok(MomentumMambaCollapseRootCauseV2::ProbabilitySaturatedHigh)
        }
        "ValidationClassImbalanceDominated" => {
            Ok(MomentumMambaCollapseRootCauseV2::ValidationClassImbalanceDominated)
        }
        "NumericalFailure" => Ok(MomentumMambaCollapseRootCauseV2::NumericalFailure),
        "Mixed" => Ok(MomentumMambaCollapseRootCauseV2::Mixed),
        "InsufficientDiagnosticEvidence" => {
            Ok(MomentumMambaCollapseRootCauseV2::InsufficientDiagnosticEvidence)
        }
        _ => Err("unknown Momentum Mamba collapse root cause".to_string()),
    }
}

fn parse_capability_v2(value: &str) -> Result<MomentumMambaRepairCapabilityStatusV2, String> {
    match value {
        "RepairableWithExistingHeadControls" => {
            Ok(MomentumMambaRepairCapabilityStatusV2::RepairableWithExistingHeadControls)
        }
        "RepairableWithExistingPoolingControls" => {
            Ok(MomentumMambaRepairCapabilityStatusV2::RepairableWithExistingPoolingControls)
        }
        "RepairableWithBoundedHeadRegularization" => {
            Ok(MomentumMambaRepairCapabilityStatusV2::RepairableWithBoundedHeadRegularization)
        }
        "RepresentationPathBlocked" => {
            Ok(MomentumMambaRepairCapabilityStatusV2::RepresentationPathBlocked)
        }
        "FreshValidationInsufficient" => {
            Ok(MomentumMambaRepairCapabilityStatusV2::FreshValidationInsufficient)
        }
        "UnsupportedRepairRequired" => {
            Ok(MomentumMambaRepairCapabilityStatusV2::UnsupportedRepairRequired)
        }
        "TechnicalFailure" => Ok(MomentumMambaRepairCapabilityStatusV2::TechnicalFailure),
        _ => Err("unknown Momentum Mamba repair capability".to_string()),
    }
}

fn parse_pooling_v2(value: &str) -> Result<SequencePooling, String> {
    match value {
        "LastOutput" => Ok(SequencePooling::LastOutput),
        "MeanOutput" => Ok(SequencePooling::MeanOutput),
        _ => Err("unknown V2 pooling policy".to_string()),
    }
}

fn parse_role_v2(value: &str) -> Result<ParticipantQualificationRoleV2, String> {
    match value {
        "LearnedCandidate" => Ok(ParticipantQualificationRoleV2::LearnedCandidate),
        "LinearComparator" => Ok(ParticipantQualificationRoleV2::LinearComparator),
        "ConstantBenchmark" => Ok(ParticipantQualificationRoleV2::ConstantBenchmark),
        _ => Err("unknown V2 participant role".to_string()),
    }
}

fn parse_qualification_v2(value: &str) -> Result<ValidationQualificationStatusV2, String> {
    match value {
        "Qualified" => Ok(ValidationQualificationStatusV2::Qualified),
        "BenchmarkQualified" => Ok(ValidationQualificationStatusV2::BenchmarkQualified),
        "RejectedInsufficientValidation" => {
            Ok(ValidationQualificationStatusV2::RejectedInsufficientValidation)
        }
        "RejectedRepresentationCollapse" => {
            Ok(ValidationQualificationStatusV2::RejectedRepresentationCollapse)
        }
        "RejectedProbabilityCollapse" => {
            Ok(ValidationQualificationStatusV2::RejectedProbabilityCollapse)
        }
        "RejectedNumericalFailure" => Ok(ValidationQualificationStatusV2::RejectedNumericalFailure),
        "RejectedPolicyInvariant" => Ok(ValidationQualificationStatusV2::RejectedPolicyInvariant),
        _ => Err("unknown V2 qualification status".to_string()),
    }
}

fn parse_evaluation_status_v2(
    value: &str,
) -> Result<MomentumFutureEvaluationRegistrationStatusV2, String> {
    match value {
        "Registered" => Ok(MomentumFutureEvaluationRegistrationStatusV2::Registered),
        "NoQualifiedLearnedParticipant" => {
            Ok(MomentumFutureEvaluationRegistrationStatusV2::NoQualifiedLearnedParticipant)
        }
        "InsufficientComparators" => {
            Ok(MomentumFutureEvaluationRegistrationStatusV2::InsufficientComparators)
        }
        "SafetyContractInvalid" => {
            Ok(MomentumFutureEvaluationRegistrationStatusV2::SafetyContractInvalid)
        }
        _ => Err("unknown V2 evaluation registration status".to_string()),
    }
}

fn parse_execution_status_v2(value: &str) -> Result<MomentumMambaRepairExecutionStatusV2, String> {
    match value {
        "Planned" => Ok(MomentumMambaRepairExecutionStatusV2::Planned),
        "Executed" => Ok(MomentumMambaRepairExecutionStatusV2::Executed),
        "AlreadyExecuted" => Ok(MomentumMambaRepairExecutionStatusV2::AlreadyExecuted),
        "FreshRepairValidationInsufficient" => {
            Ok(MomentumMambaRepairExecutionStatusV2::FreshRepairValidationInsufficient)
        }
        "UnsupportedRepairRequired" => {
            Ok(MomentumMambaRepairExecutionStatusV2::UnsupportedRepairRequired)
        }
        "TechnicalFailure" => Ok(MomentumMambaRepairExecutionStatusV2::TechnicalFailure),
        _ => Err("unknown V2 repair execution status".to_string()),
    }
}

fn variant_to_protobuf_v2(
    value: &MomentumMambaRepairVariantConfigV2,
) -> Result<RepairVariantProtobufV2, String> {
    Ok(RepairVariantProtobufV2 {
        variant_id: value.variant_id.clone(),
        pooling_policy: format!("{:?}", value.pooling_policy),
        learning_rate_bits: value.learning_rate_bits,
        l2_regularization_bits: value.l2_regularization_bits,
        maximum_epochs: u64::try_from(value.maximum_epochs)
            .map_err(|_| "V2 epoch overflow".to_string())?,
        class_weight_policy: value.class_weight_policy.clone(),
        initialization_seed: value.initialization_seed,
        encoder_frozen: value.encoder_frozen,
        feature_policy_digest: value.feature_policy_digest.clone(),
        label_policy_digest: value.label_policy_digest.clone(),
        training_policy_digest: value.training_policy_digest.clone(),
        variant_config_digest: value.variant_config_digest.clone(),
    })
}

fn variant_from_protobuf_v2(
    value: RepairVariantProtobufV2,
) -> Result<MomentumMambaRepairVariantConfigV2, String> {
    let result = MomentumMambaRepairVariantConfigV2 {
        variant_id: value.variant_id,
        pooling_policy: parse_pooling_v2(&value.pooling_policy)?,
        learning_rate_bits: value.learning_rate_bits,
        l2_regularization_bits: value.l2_regularization_bits,
        maximum_epochs: usize_v2(value.maximum_epochs)?,
        class_weight_policy: value.class_weight_policy,
        initialization_seed: value.initialization_seed,
        encoder_frozen: value.encoder_frozen,
        feature_policy_digest: value.feature_policy_digest,
        label_policy_digest: value.label_policy_digest,
        training_policy_digest: value.training_policy_digest,
        variant_config_digest: value.variant_config_digest,
    };
    validate_variant_v2(&result)?;
    Ok(result)
}

fn participant_to_protobuf_v2(value: &FrozenCandidateParticipantV2) -> ParticipantProtobufV2 {
    ParticipantProtobufV2 {
        participant_version: value.participant_version.clone(),
        participant_id: value.participant_id.clone(),
        participant_role: format!("{:?}", value.participant_role),
        model_kind: value.model_kind.clone(),
        variant_config_digest: value.variant_config_digest.clone(),
        source_snapshot_digest: value.source_snapshot_digest.clone(),
        repair_training_range_digest: value.repair_training_range_digest.clone(),
        fresh_validation_range_digest: value.fresh_validation_range_digest.clone(),
        validation_timestamp_digest: value.validation_timestamp_digest.clone(),
        model_artifact_digest: value.model_artifact_digest.clone(),
        parameter_digest: value.parameter_digest.clone(),
        feature_normalizer_digest: value.feature_normalizer_digest.clone(),
        encoder_digest: value.encoder_digest.clone(),
        feature_policy_digest: value.feature_policy_digest.clone(),
        label_policy_digest: value.label_policy_digest.clone(),
        training_policy_digest: value.training_policy_digest.clone(),
        initialization_digest: value.initialization_digest.clone(),
        warm_start_from_v1: value.warm_start_from_v1,
        v1_head_reused: value.v1_head_reused,
        fresh_deterministic_initialization: value.fresh_deterministic_initialization,
        encoder_frozen: value.encoder_frozen,
        deployment_status: format!("{:?}", value.deployment_status),
        participant_digest: value.participant_digest.clone(),
    }
}

fn participant_from_protobuf_v2(
    value: ParticipantProtobufV2,
) -> Result<FrozenCandidateParticipantV2, String> {
    if value.deployment_status != "ShadowOnly" {
        return Err("unknown V2 deployment status".to_string());
    }
    let result = FrozenCandidateParticipantV2 {
        participant_version: value.participant_version,
        participant_id: value.participant_id,
        participant_role: parse_role_v2(&value.participant_role)?,
        model_kind: value.model_kind,
        variant_config_digest: value.variant_config_digest,
        source_snapshot_digest: value.source_snapshot_digest,
        repair_training_range_digest: value.repair_training_range_digest,
        fresh_validation_range_digest: value.fresh_validation_range_digest,
        validation_timestamp_digest: value.validation_timestamp_digest,
        model_artifact_digest: value.model_artifact_digest,
        parameter_digest: value.parameter_digest,
        feature_normalizer_digest: value.feature_normalizer_digest,
        encoder_digest: value.encoder_digest,
        feature_policy_digest: value.feature_policy_digest,
        label_policy_digest: value.label_policy_digest,
        training_policy_digest: value.training_policy_digest,
        initialization_digest: value.initialization_digest,
        warm_start_from_v1: value.warm_start_from_v1,
        v1_head_reused: value.v1_head_reused,
        fresh_deterministic_initialization: value.fresh_deterministic_initialization,
        encoder_frozen: value.encoder_frozen,
        deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
        participant_digest: value.participant_digest,
    };
    validate_participant_v2(&result)?;
    Ok(result)
}

fn qualification_to_protobuf_v2(
    value: &ParticipantValidationQualificationV2,
) -> Result<QualificationProtobufV2, String> {
    Ok(QualificationProtobufV2 {
        receipt_version: value.receipt_version.clone(),
        participant_id: value.participant_id.clone(),
        participant_role: format!("{:?}", value.participant_role),
        participant_digest: value.participant_digest.clone(),
        fresh_validation_range_digest: value.fresh_validation_range_digest.clone(),
        qualification_policy_digest: value.qualification_policy_digest.clone(),
        private_metric_digest: value.private_metric_digest.clone(),
        qualification_status: format!("{:?}", value.qualification_status),
        validation_parameter_updates: u64::try_from(value.validation_parameter_updates)
            .map_err(|_| "V2 validation update overflow".to_string())?,
        historical_test_reads: u64::try_from(value.historical_test_reads)
            .map_err(|_| "V2 historical read overflow".to_string())?,
        future_evaluation_reads: u64::try_from(value.future_evaluation_reads)
            .map_err(|_| "V2 future read overflow".to_string())?,
        receipt_digest: value.receipt_digest.clone(),
    })
}

fn qualification_from_protobuf_v2(
    value: QualificationProtobufV2,
) -> Result<ParticipantValidationQualificationV2, String> {
    let result = ParticipantValidationQualificationV2 {
        receipt_version: value.receipt_version,
        participant_id: value.participant_id,
        participant_role: parse_role_v2(&value.participant_role)?,
        participant_digest: value.participant_digest,
        fresh_validation_range_digest: value.fresh_validation_range_digest,
        qualification_policy_digest: value.qualification_policy_digest,
        private_metric_digest: value.private_metric_digest,
        qualification_status: parse_qualification_v2(&value.qualification_status)?,
        validation_parameter_updates: usize_v2(value.validation_parameter_updates)?,
        historical_test_reads: usize_v2(value.historical_test_reads)?,
        future_evaluation_reads: usize_v2(value.future_evaluation_reads)?,
        receipt_digest: value.receipt_digest,
    };
    validate_qualification_v2(&result)?;
    Ok(result)
}

pub fn encode_momentum_mamba_collapse_audit_protobuf_v2(
    value: &MomentumMambaCollapseAuditV2,
) -> Result<Vec<u8>, String> {
    validate_audit_v2(value)?;
    Ok(CollapseAuditProtobufV2 {
        audit_version: value.audit_version.clone(),
        agent_id: value.agent_id.clone(),
        source_family_digest: value.source_family_digest.clone(),
        failed_participant_digest: value.failed_participant_digest.clone(),
        failed_qualification_receipt_digest: value.failed_qualification_receipt_digest.clone(),
        training_range_digest: value.training_range_digest.clone(),
        prior_validation_range_digest: value.prior_validation_range_digest.clone(),
        encoder_digest: value.encoder_digest.clone(),
        representation_normalizer_digest: value.representation_normalizer_digest.clone(),
        feature_normalizer_digest: value.feature_normalizer_digest.clone(),
        head_parameter_digest: value.head_parameter_digest.clone(),
        representation_diagnostic_digest: value.representation_diagnostic_digest.clone(),
        optimization_diagnostic_digest: value.optimization_diagnostic_digest.clone(),
        probability_diagnostic_digest: value.probability_diagnostic_digest.clone(),
        class_balance_diagnostic_digest: value.class_balance_diagnostic_digest.clone(),
        root_causes: value
            .root_causes
            .iter()
            .map(|cause| format!("{cause:?}"))
            .collect(),
        repair_capability_status: format!("{:?}", value.repair_capability_status),
        audit_digest: value.audit_digest.clone(),
    }
    .encode_to_vec())
}

pub fn decode_momentum_mamba_collapse_audit_protobuf_v2(
    bytes: &[u8],
) -> Result<MomentumMambaCollapseAuditV2, String> {
    let value = CollapseAuditProtobufV2::decode(bytes)
        .map_err(|_| "Momentum Mamba collapse audit Protobuf rejected".to_string())?;
    let result = MomentumMambaCollapseAuditV2 {
        audit_version: value.audit_version,
        agent_id: value.agent_id,
        source_family_digest: value.source_family_digest,
        failed_participant_digest: value.failed_participant_digest,
        failed_qualification_receipt_digest: value.failed_qualification_receipt_digest,
        training_range_digest: value.training_range_digest,
        prior_validation_range_digest: value.prior_validation_range_digest,
        encoder_digest: value.encoder_digest,
        representation_normalizer_digest: value.representation_normalizer_digest,
        feature_normalizer_digest: value.feature_normalizer_digest,
        head_parameter_digest: value.head_parameter_digest,
        representation_diagnostic_digest: value.representation_diagnostic_digest,
        optimization_diagnostic_digest: value.optimization_diagnostic_digest,
        probability_diagnostic_digest: value.probability_diagnostic_digest,
        class_balance_diagnostic_digest: value.class_balance_diagnostic_digest,
        root_causes: value
            .root_causes
            .iter()
            .map(|cause| parse_root_cause_v2(cause))
            .collect::<Result<Vec<_>, _>>()?,
        repair_capability_status: parse_capability_v2(&value.repair_capability_status)?,
        audit_digest: value.audit_digest,
    };
    validate_audit_v2(&result)?;
    Ok(result)
}

pub fn encode_momentum_mamba_repair_split_protobuf_v2(
    value: &MomentumMambaRepairSplitV2,
) -> Result<Vec<u8>, String> {
    validate_split_v2(value)?;
    Ok(RepairSplitProtobufV2 {
        split_version: value.split_version.clone(),
        source_snapshot_digest: value.source_snapshot_digest.clone(),
        prior_usage_ledger_digest: value.prior_usage_ledger_digest.clone(),
        repair_training_range: Some(range_to_protobuf_v2(&value.repair_training_range)?),
        repair_purge_range: Some(range_to_protobuf_v2(&value.repair_purge_range)?),
        fresh_repair_validation_range: Some(range_to_protobuf_v2(
            &value.fresh_repair_validation_range,
        )?),
        remaining_reserved_range: value
            .remaining_reserved_range
            .as_ref()
            .map(range_to_protobuf_v2)
            .transpose()?,
        label_horizon: u64::try_from(value.label_horizon)
            .map_err(|_| "V2 label horizon overflow".to_string())?,
        minimum_validation_samples: u64::try_from(value.minimum_validation_samples)
            .map_err(|_| "V2 minimum validation overflow".to_string())?,
        prior_validation_overlap_count: u64::try_from(value.prior_validation_overlap_count)
            .map_err(|_| "V2 prior overlap overflow".to_string())?,
        prospective_overlap_count: u64::try_from(value.prospective_overlap_count)
            .map_err(|_| "V2 prospective overlap overflow".to_string())?,
        future_evaluation_overlap_count: u64::try_from(value.future_evaluation_overlap_count)
            .map_err(|_| "V2 future overlap overflow".to_string())?,
        split_digest: value.split_digest.clone(),
    }
    .encode_to_vec())
}

pub fn decode_momentum_mamba_repair_split_protobuf_v2(
    bytes: &[u8],
) -> Result<MomentumMambaRepairSplitV2, String> {
    let value = RepairSplitProtobufV2::decode(bytes)
        .map_err(|_| "Momentum Mamba repair split Protobuf rejected".to_string())?;
    let result = MomentumMambaRepairSplitV2 {
        split_version: value.split_version,
        source_snapshot_digest: value.source_snapshot_digest,
        prior_usage_ledger_digest: value.prior_usage_ledger_digest,
        repair_training_range: range_from_protobuf_v2(value.repair_training_range)?,
        repair_purge_range: range_from_protobuf_v2(value.repair_purge_range)?,
        fresh_repair_validation_range: range_from_protobuf_v2(value.fresh_repair_validation_range)?,
        remaining_reserved_range: value
            .remaining_reserved_range
            .map(|range| range_from_protobuf_v2(Some(range)))
            .transpose()?,
        label_horizon: usize_v2(value.label_horizon)?,
        minimum_validation_samples: usize_v2(value.minimum_validation_samples)?,
        prior_validation_overlap_count: usize_v2(value.prior_validation_overlap_count)?,
        prospective_overlap_count: usize_v2(value.prospective_overlap_count)?,
        future_evaluation_overlap_count: usize_v2(value.future_evaluation_overlap_count)?,
        split_digest: value.split_digest,
    };
    validate_split_v2(&result)?;
    Ok(result)
}

pub fn encode_momentum_mamba_repair_registration_protobuf_v2(
    value: &MomentumMambaRepairRegistrationV2,
) -> Result<Vec<u8>, String> {
    validate_registration_v2(value)?;
    Ok(RepairRegistrationProtobufV2 {
        registration_version: value.registration_version.clone(),
        agent_id: value.agent_id.clone(),
        source_snapshot_digest: value.source_snapshot_digest.clone(),
        canonical_intent_digest: value.canonical_intent_digest.clone(),
        canonical_view_digest: value.canonical_view_digest.clone(),
        source_family_digest: value.source_family_digest.clone(),
        failed_participant_digest: value.failed_participant_digest.clone(),
        collapse_audit_digest: value.collapse_audit_digest.clone(),
        repair_split_digest: value.repair_split_digest.clone(),
        allowed_variant_configs: value
            .allowed_variant_configs
            .iter()
            .map(variant_to_protobuf_v2)
            .collect::<Result<Vec<_>, _>>()?,
        maximum_repair_variants: u64::try_from(value.maximum_repair_variants)
            .map_err(|_| "V2 variant cap overflow".to_string())?,
        fresh_validation_hidden: value.fresh_validation_hidden,
        historical_test_forbidden: value.historical_test_forbidden,
        future_evaluation_forbidden: value.future_evaluation_forbidden,
        winner_selection_forbidden: value.winner_selection_forbidden,
        active_promotion_forbidden: value.active_promotion_forbidden,
        reward_application_forbidden: value.reward_application_forbidden,
        registration_digest: value.registration_digest.clone(),
    }
    .encode_to_vec())
}

pub fn decode_momentum_mamba_repair_registration_protobuf_v2(
    bytes: &[u8],
) -> Result<MomentumMambaRepairRegistrationV2, String> {
    let value = RepairRegistrationProtobufV2::decode(bytes)
        .map_err(|_| "Momentum Mamba repair registration Protobuf rejected".to_string())?;
    let result = MomentumMambaRepairRegistrationV2 {
        registration_version: value.registration_version,
        agent_id: value.agent_id,
        source_snapshot_digest: value.source_snapshot_digest,
        canonical_intent_digest: value.canonical_intent_digest,
        canonical_view_digest: value.canonical_view_digest,
        source_family_digest: value.source_family_digest,
        failed_participant_digest: value.failed_participant_digest,
        collapse_audit_digest: value.collapse_audit_digest,
        repair_split_digest: value.repair_split_digest,
        allowed_variant_configs: value
            .allowed_variant_configs
            .into_iter()
            .map(variant_from_protobuf_v2)
            .collect::<Result<Vec<_>, _>>()?,
        maximum_repair_variants: usize_v2(value.maximum_repair_variants)?,
        fresh_validation_hidden: value.fresh_validation_hidden,
        historical_test_forbidden: value.historical_test_forbidden,
        future_evaluation_forbidden: value.future_evaluation_forbidden,
        winner_selection_forbidden: value.winner_selection_forbidden,
        active_promotion_forbidden: value.active_promotion_forbidden,
        reward_application_forbidden: value.reward_application_forbidden,
        registration_digest: value.registration_digest,
    };
    validate_registration_v2(&result)?;
    Ok(result)
}

pub fn encode_momentum_candidate_participant_protobuf_v2(
    value: &FrozenCandidateParticipantV2,
) -> Result<Vec<u8>, String> {
    validate_participant_v2(value)?;
    Ok(participant_to_protobuf_v2(value).encode_to_vec())
}

pub fn decode_momentum_candidate_participant_protobuf_v2(
    bytes: &[u8],
) -> Result<FrozenCandidateParticipantV2, String> {
    participant_from_protobuf_v2(
        ParticipantProtobufV2::decode(bytes)
            .map_err(|_| "Momentum V2 participant Protobuf rejected".to_string())?,
    )
}

pub fn encode_momentum_qualification_receipt_protobuf_v2(
    value: &ParticipantValidationQualificationV2,
) -> Result<Vec<u8>, String> {
    validate_qualification_v2(value)?;
    Ok(qualification_to_protobuf_v2(value)?.encode_to_vec())
}

pub fn decode_momentum_qualification_receipt_protobuf_v2(
    bytes: &[u8],
) -> Result<ParticipantValidationQualificationV2, String> {
    qualification_from_protobuf_v2(
        QualificationProtobufV2::decode(bytes)
            .map_err(|_| "Momentum V2 qualification Protobuf rejected".to_string())?,
    )
}

pub fn encode_momentum_candidate_family_protobuf_v2(
    value: &MomentumCandidateFamilyV2,
) -> Result<Vec<u8>, String> {
    validate_family_v2(value)?;
    Ok(FamilyProtobufV2 {
        family_version: value.family_version.clone(),
        agent_id: value.agent_id.clone(),
        source_snapshot_digest: value.source_snapshot_digest.clone(),
        canonical_view_digest: value.canonical_view_digest.clone(),
        repair_registration_digest: value.repair_registration_digest.clone(),
        repair_split_digest: value.repair_split_digest.clone(),
        collapse_audit_digest: value.collapse_audit_digest.clone(),
        participants: value
            .participants
            .iter()
            .map(participant_to_protobuf_v2)
            .collect(),
        qualification_receipts: value
            .qualification_receipts
            .iter()
            .map(qualification_to_protobuf_v2)
            .collect::<Result<Vec<_>, _>>()?,
        learned_participant_count: u64::try_from(value.learned_participant_count)
            .map_err(|_| "V2 learned count overflow".to_string())?,
        qualified_learned_participant_count: u64::try_from(
            value.qualified_learned_participant_count,
        )
        .map_err(|_| "V2 qualified learned count overflow".to_string())?,
        qualified_comparator_count: u64::try_from(value.qualified_comparator_count)
            .map_err(|_| "V2 comparator count overflow".to_string())?,
        winner_selected: value.winner_selected,
        historical_test_accessed: value.historical_test_accessed,
        eligible_for_active_committee: value.eligible_for_active_committee,
        eligible_for_promotion: value.eligible_for_promotion,
        eligible_for_reward: value.eligible_for_reward,
        family_digest: value.family_digest.clone(),
    }
    .encode_to_vec())
}

pub fn decode_momentum_candidate_family_protobuf_v2(
    bytes: &[u8],
) -> Result<MomentumCandidateFamilyV2, String> {
    let value = FamilyProtobufV2::decode(bytes)
        .map_err(|_| "Momentum V2 family Protobuf rejected".to_string())?;
    let result = MomentumCandidateFamilyV2 {
        family_version: value.family_version,
        agent_id: value.agent_id,
        source_snapshot_digest: value.source_snapshot_digest,
        canonical_view_digest: value.canonical_view_digest,
        repair_registration_digest: value.repair_registration_digest,
        repair_split_digest: value.repair_split_digest,
        collapse_audit_digest: value.collapse_audit_digest,
        participants: value
            .participants
            .into_iter()
            .map(participant_from_protobuf_v2)
            .collect::<Result<Vec<_>, _>>()?,
        qualification_receipts: value
            .qualification_receipts
            .into_iter()
            .map(qualification_from_protobuf_v2)
            .collect::<Result<Vec<_>, _>>()?,
        learned_participant_count: usize_v2(value.learned_participant_count)?,
        qualified_learned_participant_count: usize_v2(value.qualified_learned_participant_count)?,
        qualified_comparator_count: usize_v2(value.qualified_comparator_count)?,
        winner_selected: value.winner_selected,
        historical_test_accessed: value.historical_test_accessed,
        eligible_for_active_committee: value.eligible_for_active_committee,
        eligible_for_promotion: value.eligible_for_promotion,
        eligible_for_reward: value.eligible_for_reward,
        family_digest: value.family_digest,
    };
    validate_family_v2(&result)?;
    Ok(result)
}

pub fn encode_momentum_future_evaluation_roster_protobuf_v2(
    value: &MomentumFutureEvaluationRosterV2,
    family: &MomentumCandidateFamilyV2,
) -> Result<Vec<u8>, String> {
    validate_roster_v2(value, family)?;
    Ok(RosterProtobufV2 {
        roster_version: value.roster_version.clone(),
        family_digest: value.family_digest.clone(),
        qualified_learned_participant_digests: value.qualified_learned_participant_digests.clone(),
        qualified_comparator_digests: value.qualified_comparator_digests.clone(),
        excluded_participant_digests: value.excluded_participant_digests.clone(),
        inclusion_policy_digest: value.inclusion_policy_digest.clone(),
        roster_digest: value.roster_digest.clone(),
    }
    .encode_to_vec())
}

fn decode_roster_unbound_v2(bytes: &[u8]) -> Result<MomentumFutureEvaluationRosterV2, String> {
    let value = RosterProtobufV2::decode(bytes)
        .map_err(|_| "Momentum V2 roster Protobuf rejected".to_string())?;
    let result = MomentumFutureEvaluationRosterV2 {
        roster_version: value.roster_version,
        family_digest: value.family_digest,
        qualified_learned_participant_digests: value.qualified_learned_participant_digests,
        qualified_comparator_digests: value.qualified_comparator_digests,
        excluded_participant_digests: value.excluded_participant_digests,
        inclusion_policy_digest: value.inclusion_policy_digest,
        roster_digest: value.roster_digest,
    };
    if result.roster_version != ROSTER_VERSION_V2
        || result.roster_digest != roster_digest_v2(&result)
    {
        return Err("Momentum V2 roster identity rejected".to_string());
    }
    Ok(result)
}

pub fn decode_momentum_future_evaluation_roster_protobuf_v2(
    bytes: &[u8],
    family: &MomentumCandidateFamilyV2,
) -> Result<MomentumFutureEvaluationRosterV2, String> {
    let result = decode_roster_unbound_v2(bytes)?;
    validate_roster_v2(&result, family)?;
    Ok(result)
}

pub fn encode_momentum_future_evaluation_registration_protobuf_v2(
    value: &MomentumFutureEvaluationRegistrationV2,
    family: &MomentumCandidateFamilyV2,
    roster: &MomentumFutureEvaluationRosterV2,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<Vec<u8>, String> {
    validate_evaluation_registration_v2(value, family, roster, reservation)?;
    Ok(EvaluationRegistrationProtobufV2 {
        registration_version: value.registration_version.clone(),
        agent_id: value.agent_id.clone(),
        family_digest: value.family_digest.clone(),
        roster_digest: value.roster_digest.clone(),
        repair_registration_digest: value.repair_registration_digest.clone(),
        collapse_audit_digest: value.collapse_audit_digest.clone(),
        qualification_receipt_digests: value.qualification_receipt_digests.clone(),
        source_snapshot_digest: value.source_snapshot_digest.clone(),
        source_boundary_timestamp_ms: value.source_boundary_timestamp_ms,
        protected_registration_digests: value.protected_registration_digests.clone(),
        protected_timestamp_ms: value.protected_timestamp_ms.clone(),
        prior_reserved_range_digests: value.prior_reserved_range_digests.clone(),
        provider_finality_boundary_ms: value.provider_finality_boundary_ms,
        minimum_accepted_timestamp_ms: value.minimum_accepted_timestamp_ms,
        labels_hidden_until_opening: value.labels_hidden_until_opening,
        probabilities_hidden_until_opening: value.probabilities_hidden_until_opening,
        one_time_opening_required: value.one_time_opening_required,
        winner_selection_forbidden_before_opening: value.winner_selection_forbidden_before_opening,
        active_promotion_forbidden: value.active_promotion_forbidden,
        reward_application_forbidden: value.reward_application_forbidden,
        maximum_requests: u64::try_from(value.maximum_requests)
            .map_err(|_| "V2 maximum request overflow".to_string())?,
        maximum_concurrency: u64::try_from(value.maximum_concurrency)
            .map_err(|_| "V2 maximum concurrency overflow".to_string())?,
        maximum_retries: u64::try_from(value.maximum_retries)
            .map_err(|_| "V2 maximum retry overflow".to_string())?,
        status: format!("{:?}", value.status),
        registration_digest: value.registration_digest.clone(),
    }
    .encode_to_vec())
}

fn decode_evaluation_registration_unbound_v2(
    bytes: &[u8],
) -> Result<MomentumFutureEvaluationRegistrationV2, String> {
    let value = EvaluationRegistrationProtobufV2::decode(bytes)
        .map_err(|_| "Momentum V2 evaluation registration Protobuf rejected".to_string())?;
    let result = MomentumFutureEvaluationRegistrationV2 {
        registration_version: value.registration_version,
        agent_id: value.agent_id,
        family_digest: value.family_digest,
        roster_digest: value.roster_digest,
        repair_registration_digest: value.repair_registration_digest,
        collapse_audit_digest: value.collapse_audit_digest,
        qualification_receipt_digests: value.qualification_receipt_digests,
        source_snapshot_digest: value.source_snapshot_digest,
        source_boundary_timestamp_ms: value.source_boundary_timestamp_ms,
        protected_registration_digests: value.protected_registration_digests,
        protected_timestamp_ms: value.protected_timestamp_ms,
        prior_reserved_range_digests: value.prior_reserved_range_digests,
        provider_finality_boundary_ms: value.provider_finality_boundary_ms,
        minimum_accepted_timestamp_ms: value.minimum_accepted_timestamp_ms,
        labels_hidden_until_opening: value.labels_hidden_until_opening,
        probabilities_hidden_until_opening: value.probabilities_hidden_until_opening,
        one_time_opening_required: value.one_time_opening_required,
        winner_selection_forbidden_before_opening: value.winner_selection_forbidden_before_opening,
        active_promotion_forbidden: value.active_promotion_forbidden,
        reward_application_forbidden: value.reward_application_forbidden,
        maximum_requests: usize_v2(value.maximum_requests)?,
        maximum_concurrency: usize_v2(value.maximum_concurrency)?,
        maximum_retries: usize_v2(value.maximum_retries)?,
        status: parse_evaluation_status_v2(&value.status)?,
        registration_digest: value.registration_digest,
    };
    if result.registration_version != EVALUATION_VERSION_V2
        || result.registration_digest != evaluation_registration_digest_v2(&result)
    {
        return Err("Momentum V2 evaluation registration identity rejected".to_string());
    }
    Ok(result)
}

pub fn decode_momentum_future_evaluation_registration_protobuf_v2(
    bytes: &[u8],
    family: &MomentumCandidateFamilyV2,
    roster: &MomentumFutureEvaluationRosterV2,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<MomentumFutureEvaluationRegistrationV2, String> {
    let result = decode_evaluation_registration_unbound_v2(bytes)?;
    validate_evaluation_registration_v2(&result, family, roster, reservation)?;
    Ok(result)
}

pub fn encode_momentum_mamba_repair_journal_protobuf_v2(
    value: &MomentumMambaRepairJournalV2,
) -> Result<Vec<u8>, String> {
    validate_journal_v2(value)?;
    Ok(JournalProtobufV2 {
        journal_version: value.journal_version.clone(),
        agent_id: value.agent_id.clone(),
        collapse_audit_digest: value.collapse_audit_digest.clone(),
        repair_split_digest: value.repair_split_digest.clone(),
        repair_registration_digest: value.repair_registration_digest.clone(),
        family_digest: value.family_digest.clone(),
        roster_digest: value.roster_digest.clone(),
        evaluation_registration_digest: value.evaluation_registration_digest.clone(),
        prior_validation_used_for_repair_qualification: value
            .prior_validation_used_for_repair_qualification,
        warm_start_from_v1: value.warm_start_from_v1,
        v1_head_reused: value.v1_head_reused,
        fresh_deterministic_initialization: value.fresh_deterministic_initialization,
        status: format!("{:?}", value.status),
        journal_digest: value.journal_digest.clone(),
    }
    .encode_to_vec())
}

pub fn decode_momentum_mamba_repair_journal_protobuf_v2(
    bytes: &[u8],
) -> Result<MomentumMambaRepairJournalV2, String> {
    let value = JournalProtobufV2::decode(bytes)
        .map_err(|_| "Momentum Mamba repair journal Protobuf rejected".to_string())?;
    let result = MomentumMambaRepairJournalV2 {
        journal_version: value.journal_version,
        agent_id: value.agent_id,
        collapse_audit_digest: value.collapse_audit_digest,
        repair_split_digest: value.repair_split_digest,
        repair_registration_digest: value.repair_registration_digest,
        family_digest: value.family_digest,
        roster_digest: value.roster_digest,
        evaluation_registration_digest: value.evaluation_registration_digest,
        prior_validation_used_for_repair_qualification: value
            .prior_validation_used_for_repair_qualification,
        warm_start_from_v1: value.warm_start_from_v1,
        v1_head_reused: value.v1_head_reused,
        fresh_deterministic_initialization: value.fresh_deterministic_initialization,
        status: parse_execution_status_v2(&value.status)?,
        journal_digest: value.journal_digest,
    };
    validate_journal_v2(&result)?;
    Ok(result)
}

fn v2_root(root: &Path) -> PathBuf {
    root.join("v2").join(AGENT_ID_V2)
}

fn record_write_v2(
    status: AgentPrivateLearningArtifactWriteStatusV0,
    written: &mut usize,
    duplicates: &mut usize,
) {
    match status {
        AgentPrivateLearningArtifactWriteStatusV0::Written => *written += 1,
        AgentPrivateLearningArtifactWriteStatusV0::DuplicateRejected => *duplicates += 1,
    }
}

fn write_verified_v2<T>(
    path: &Path,
    bytes: &[u8],
    expected_digest: &str,
    decode: impl Fn(&[u8]) -> Result<T, String>,
    digest: impl Fn(&T) -> &str,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    atomic_write_verified_v0(path, bytes, expected_digest, |stored| {
        let value = decode(stored)?;
        Ok(digest(&value).to_string())
    })
}

fn round_trip_initial_v2(
    audit: &MomentumMambaCollapseAuditV2,
    split: &MomentumMambaRepairSplitV2,
    registration: &MomentumMambaRepairRegistrationV2,
) -> Result<(), String> {
    if decode_momentum_mamba_collapse_audit_protobuf_v2(
        &encode_momentum_mamba_collapse_audit_protobuf_v2(audit)?,
    )? != *audit
        || decode_momentum_mamba_repair_split_protobuf_v2(
            &encode_momentum_mamba_repair_split_protobuf_v2(split)?,
        )? != *split
        || decode_momentum_mamba_repair_registration_protobuf_v2(
            &encode_momentum_mamba_repair_registration_protobuf_v2(registration)?,
        )? != *registration
    {
        return Err("Momentum repair preregistration Protobuf round trip rejected".to_string());
    }
    Ok(())
}

fn persist_preregistration_v2(
    root: &Path,
    audit: &MomentumMambaCollapseAuditV2,
    split: &MomentumMambaRepairSplitV2,
    registration: &MomentumMambaRepairRegistrationV2,
) -> Result<(usize, usize), String> {
    round_trip_initial_v2(audit, split, registration)?;
    let root = v2_root(root);
    let mut written = 0;
    let mut duplicates = 0;
    let status = write_verified_v2(
        &root
            .join("collapse_audits")
            .join(format!("{}.pb", audit.audit_digest)),
        &encode_momentum_mamba_collapse_audit_protobuf_v2(audit)?,
        &audit.audit_digest,
        decode_momentum_mamba_collapse_audit_protobuf_v2,
        |value| value.audit_digest.as_str(),
    )?;
    record_write_v2(status, &mut written, &mut duplicates);
    let status = write_verified_v2(
        &root
            .join("repair_splits")
            .join(format!("{}.pb", split.split_digest)),
        &encode_momentum_mamba_repair_split_protobuf_v2(split)?,
        &split.split_digest,
        decode_momentum_mamba_repair_split_protobuf_v2,
        |value| value.split_digest.as_str(),
    )?;
    record_write_v2(status, &mut written, &mut duplicates);
    let status = write_verified_v2(
        &root
            .join("repair_registrations")
            .join(format!("{}.pb", registration.registration_digest)),
        &encode_momentum_mamba_repair_registration_protobuf_v2(registration)?,
        &registration.registration_digest,
        decode_momentum_mamba_repair_registration_protobuf_v2,
        |value| value.registration_digest.as_str(),
    )?;
    record_write_v2(status, &mut written, &mut duplicates);
    Ok((written, duplicates))
}

fn reopen_preregistration_v2(
    root: &Path,
) -> Result<
    (
        MomentumMambaCollapseAuditV2,
        MomentumMambaRepairSplitV2,
        MomentumMambaRepairRegistrationV2,
    ),
    String,
> {
    let root = v2_root(root);
    Ok((
        read_single_protobuf(
            &root.join("collapse_audits"),
            decode_momentum_mamba_collapse_audit_protobuf_v2,
        )?,
        read_single_protobuf(
            &root.join("repair_splits"),
            decode_momentum_mamba_repair_split_protobuf_v2,
        )?,
        read_single_protobuf(
            &root.join("repair_registrations"),
            decode_momentum_mamba_repair_registration_protobuf_v2,
        )?,
    ))
}

fn persist_experiment_v2(
    root: &Path,
    experiment: &RepairExperimentV2,
    journal: &MomentumMambaRepairJournalV2,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<(usize, usize), String> {
    validate_family_v2(&experiment.family)?;
    validate_journal_v2(journal)?;
    let root = v2_root(root);
    let mut written = 0;
    let mut duplicates = 0;
    for participant in &experiment.family.participants {
        let status = write_verified_v2(
            &root
                .join("participants")
                .join(format!("{}.pb", participant.participant_digest)),
            &encode_momentum_candidate_participant_protobuf_v2(participant)?,
            &participant.participant_digest,
            decode_momentum_candidate_participant_protobuf_v2,
            |value| value.participant_digest.as_str(),
        )?;
        record_write_v2(status, &mut written, &mut duplicates);
    }
    for receipt in &experiment.family.qualification_receipts {
        let status = write_verified_v2(
            &root
                .join("qualification_receipts")
                .join(format!("{}.pb", receipt.receipt_digest)),
            &encode_momentum_qualification_receipt_protobuf_v2(receipt)?,
            &receipt.receipt_digest,
            decode_momentum_qualification_receipt_protobuf_v2,
            |value| value.receipt_digest.as_str(),
        )?;
        record_write_v2(status, &mut written, &mut duplicates);
    }
    let family_bytes = encode_momentum_candidate_family_protobuf_v2(&experiment.family)?;
    let status = write_verified_v2(
        &root
            .join("families")
            .join(format!("{}.pb", experiment.family.family_digest)),
        &family_bytes,
        &experiment.family.family_digest,
        decode_momentum_candidate_family_protobuf_v2,
        |value| value.family_digest.as_str(),
    )?;
    record_write_v2(status, &mut written, &mut duplicates);
    if let Some(roster) = &experiment.roster {
        let family = &experiment.family;
        let status = write_verified_v2(
            &root
                .join("rosters")
                .join(format!("{}.pb", roster.roster_digest)),
            &encode_momentum_future_evaluation_roster_protobuf_v2(roster, family)?,
            &roster.roster_digest,
            |bytes| decode_momentum_future_evaluation_roster_protobuf_v2(bytes, family),
            |value| value.roster_digest.as_str(),
        )?;
        record_write_v2(status, &mut written, &mut duplicates);
    }
    if let (Some(registration), Some(roster)) =
        (&experiment.evaluation_registration, &experiment.roster)
    {
        let family = &experiment.family;
        let status = write_verified_v2(
            &root
                .join("evaluation_registrations")
                .join(format!("{}.pb", registration.registration_digest)),
            &encode_momentum_future_evaluation_registration_protobuf_v2(
                registration,
                family,
                roster,
                reservation,
            )?,
            &registration.registration_digest,
            |bytes| {
                decode_momentum_future_evaluation_registration_protobuf_v2(
                    bytes,
                    family,
                    roster,
                    reservation,
                )
            },
            |value| value.registration_digest.as_str(),
        )?;
        record_write_v2(status, &mut written, &mut duplicates);
    }
    let status = write_verified_v2(
        &root
            .join("journals")
            .join(format!("{}.pb", journal.journal_digest)),
        &encode_momentum_mamba_repair_journal_protobuf_v2(journal)?,
        &journal.journal_digest,
        decode_momentum_mamba_repair_journal_protobuf_v2,
        |value| value.journal_digest.as_str(),
    )?;
    record_write_v2(status, &mut written, &mut duplicates);
    Ok((written, duplicates))
}

fn read_persisted_experiment_v2(
    root: &Path,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<(RepairExperimentV2, MomentumMambaRepairJournalV2), String> {
    let root = v2_root(root);
    let family = read_single_protobuf(
        &root.join("families"),
        decode_momentum_candidate_family_protobuf_v2,
    )?;
    let participants = protobuf_paths(&root.join("participants"))?
        .into_iter()
        .map(|path| fs::read(path).map_err(|_| "V2 participant read failed".to_string()))
        .map(|bytes| {
            bytes.and_then(|bytes| decode_momentum_candidate_participant_protobuf_v2(&bytes))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let receipts = protobuf_paths(&root.join("qualification_receipts"))?
        .into_iter()
        .map(|path| fs::read(path).map_err(|_| "V2 receipt read failed".to_string()))
        .map(|bytes| {
            bytes.and_then(|bytes| decode_momentum_qualification_receipt_protobuf_v2(&bytes))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let embedded_participants = family
        .participants
        .iter()
        .map(|value| value.participant_digest.clone())
        .collect::<BTreeSet<_>>();
    let stored_participants = participants
        .iter()
        .map(|value| value.participant_digest.clone())
        .collect::<BTreeSet<_>>();
    let embedded_receipts = family
        .qualification_receipts
        .iter()
        .map(|value| value.receipt_digest.clone())
        .collect::<BTreeSet<_>>();
    let stored_receipts = receipts
        .iter()
        .map(|value| value.receipt_digest.clone())
        .collect::<BTreeSet<_>>();
    if embedded_participants != stored_participants || embedded_receipts != stored_receipts {
        return Err("V2 family sidecar binding rejected".to_string());
    }
    let roster_paths = protobuf_paths(&root.join("rosters")).unwrap_or_default();
    let roster = if roster_paths.is_empty() {
        None
    } else if roster_paths.len() == 1 {
        Some(decode_momentum_future_evaluation_roster_protobuf_v2(
            &fs::read(&roster_paths[0]).map_err(|_| "V2 roster read failed".to_string())?,
            &family,
        )?)
    } else {
        return Err("V2 roster identity is ambiguous".to_string());
    };
    let evaluation_paths =
        protobuf_paths(&root.join("evaluation_registrations")).unwrap_or_default();
    let evaluation_registration = match (&roster, evaluation_paths.len()) {
        (Some(roster), 1) => Some(decode_momentum_future_evaluation_registration_protobuf_v2(
            &fs::read(&evaluation_paths[0])
                .map_err(|_| "V2 evaluation registration read failed".to_string())?,
            &family,
            roster,
            reservation,
        )?),
        (None, 0) => None,
        _ => return Err("V2 evaluation registration binding rejected".to_string()),
    };
    let roster_status = if roster.is_some() {
        MomentumFutureEvaluationRosterStatusV2::Registered
    } else if family.qualified_learned_participant_count == 0 {
        MomentumFutureEvaluationRosterStatusV2::NoQualifiedLearnedParticipant
    } else {
        MomentumFutureEvaluationRosterStatusV2::InsufficientComparators
    };
    let evaluation_registration_status = if evaluation_registration.is_some() {
        MomentumFutureEvaluationRegistrationStatusV2::Registered
    } else if family.qualified_learned_participant_count == 0 {
        MomentumFutureEvaluationRegistrationStatusV2::NoQualifiedLearnedParticipant
    } else {
        MomentumFutureEvaluationRegistrationStatusV2::InsufficientComparators
    };
    let journal = read_single_protobuf(
        &root.join("journals"),
        decode_momentum_mamba_repair_journal_protobuf_v2,
    )?;
    if journal.family_digest.as_deref() != Some(family.family_digest.as_str())
        || journal.roster_digest.as_deref()
            != roster.as_ref().map(|value| value.roster_digest.as_str())
        || journal.evaluation_registration_digest.as_deref()
            != evaluation_registration
                .as_ref()
                .map(|value| value.registration_digest.as_str())
    {
        return Err("V2 journal cross-artifact binding rejected".to_string());
    }
    Ok((
        RepairExperimentV2 {
            family,
            roster,
            roster_status,
            evaluation_registration,
            evaluation_registration_status,
        },
        journal,
    ))
}

fn collect_protected_artifacts_v2(
    root: &Path,
    current: &Path,
    values: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), String> {
    if current == root.join("v2") {
        return Ok(());
    }
    if current.is_file() {
        let relative = current
            .strip_prefix(root)
            .map_err(|_| "protected artifact path rejected".to_string())?
            .to_path_buf();
        values.push((
            relative,
            fs::read(current).map_err(|_| "protected artifact read failed".to_string())?,
        ));
        return Ok(());
    }
    if !current.is_dir() {
        return Ok(());
    }
    let mut children = fs::read_dir(current)
        .map_err(|_| "protected artifact directory unavailable".to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    children.sort();
    for child in children {
        collect_protected_artifacts_v2(root, &child, values)?;
    }
    Ok(())
}

fn base_report_v2(
    mode: AgentPrivateLearningRunModeV0,
    status: MomentumMambaRepairExecutionStatusV2,
    protected_artifacts_unchanged: bool,
    active_state_unchanged: bool,
) -> MomentumMambaRepairReportV2 {
    let mut report = MomentumMambaRepairReportV2 {
        report_version: "momentum-mamba-repair-report-v2".to_string(),
        mode,
        status,
        collapse_audit: None,
        repair_split: None,
        repair_registration: None,
        family: None,
        roster: None,
        roster_status: MomentumFutureEvaluationRosterStatusV2::NoQualifiedLearnedParticipant,
        evaluation_registration: None,
        evaluation_registration_status:
            MomentumFutureEvaluationRegistrationStatusV2::NoQualifiedLearnedParticipant,
        journal: None,
        artifacts_written: 0,
        duplicate_artifact_count: 0,
        storage_failure_count: usize::from(
            status == MomentumMambaRepairExecutionStatusV2::TechnicalFailure,
        ),
        protected_artifacts_unchanged,
        active_state_unchanged,
        safety_counters: zero_safety_counters_v2(),
        report_digest: String::new(),
    };
    report.report_digest = report_digest_v2(&report);
    report
}

pub fn run_momentum_mamba_repair_v2(
    root: &Path,
    snapshots: &[DataSnapshot],
    reservation: &ProtectedEvaluationReservationV1,
    mode: AgentPrivateLearningRunModeV0,
) -> MomentumMambaRepairReportV2 {
    let active_before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let mut protected_before = Vec::new();
    if collect_protected_artifacts_v2(root, root, &mut protected_before).is_err() {
        return base_report_v2(
            mode,
            MomentumMambaRepairExecutionStatusV2::TechnicalFailure,
            false,
            true,
        );
    }
    let state = match load_v1_frozen_state_v2(root, snapshots) {
        Ok(value) => value,
        Err(_) => {
            return base_report_v2(
                mode,
                MomentumMambaRepairExecutionStatusV2::TechnicalFailure,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
        }
    };
    let audit = match derive_collapse_audit_v2(&state) {
        Ok(value) => value,
        Err(_) => {
            return base_report_v2(
                mode,
                MomentumMambaRepairExecutionStatusV2::TechnicalFailure,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
        }
    };
    let split = match derive_repair_split_v2(&state, reservation) {
        Ok(value) => value,
        Err(MomentumMambaRepairCapabilityStatusV2::FreshValidationInsufficient) => {
            let mut report = base_report_v2(
                mode,
                MomentumMambaRepairExecutionStatusV2::FreshRepairValidationInsufficient,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
            report.collapse_audit = Some(audit);
            report.report_digest = report_digest_v2(&report);
            return report;
        }
        Err(_) => {
            return base_report_v2(
                mode,
                MomentumMambaRepairExecutionStatusV2::TechnicalFailure,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
        }
    };
    let registration = match derive_repair_registration_v2(&state, &audit, &split) {
        Ok(value) => value,
        Err(_) => {
            let status = if matches!(
                audit.repair_capability_status,
                MomentumMambaRepairCapabilityStatusV2::UnsupportedRepairRequired
                    | MomentumMambaRepairCapabilityStatusV2::RepresentationPathBlocked
            ) {
                MomentumMambaRepairExecutionStatusV2::UnsupportedRepairRequired
            } else {
                MomentumMambaRepairExecutionStatusV2::TechnicalFailure
            };
            let mut report = base_report_v2(
                mode,
                status,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
            report.collapse_audit = Some(audit);
            report.repair_split = Some(split);
            report.report_digest = report_digest_v2(&report);
            return report;
        }
    };
    let persisted_before = read_persisted_experiment_v2(root, reservation).ok();
    if mode == AgentPrivateLearningRunModeV0::Status {
        let (experiment, journal) = match persisted_before {
            Some(value) => value,
            None => {
                let mut report = base_report_v2(
                    mode,
                    MomentumMambaRepairExecutionStatusV2::Planned,
                    true,
                    stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                        == active_before,
                );
                report.collapse_audit = Some(audit);
                report.repair_split = Some(split);
                report.repair_registration = Some(registration);
                report.report_digest = report_digest_v2(&report);
                return report;
            }
        };
        let mut report = base_report_v2(
            mode,
            MomentumMambaRepairExecutionStatusV2::AlreadyExecuted,
            true,
            stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before,
        );
        report.collapse_audit = Some(audit);
        report.repair_split = Some(split);
        report.repair_registration = Some(registration);
        report.family = Some(experiment.family);
        report.roster = experiment.roster;
        report.roster_status = experiment.roster_status;
        report.evaluation_registration = experiment.evaluation_registration;
        report.evaluation_registration_status = experiment.evaluation_registration_status;
        report.journal = Some(journal);
        report.report_digest = report_digest_v2(&report);
        return report;
    }
    if mode == AgentPrivateLearningRunModeV0::DryRun {
        let round_trip_ok = round_trip_initial_v2(&audit, &split, &registration).is_ok();
        let mut report = base_report_v2(
            mode,
            if round_trip_ok {
                MomentumMambaRepairExecutionStatusV2::Planned
            } else {
                MomentumMambaRepairExecutionStatusV2::TechnicalFailure
            },
            true,
            stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before,
        );
        report.storage_failure_count = usize::from(!round_trip_ok);
        report.collapse_audit = Some(audit);
        report.repair_split = Some(split);
        report.repair_registration = Some(registration);
        report.report_digest = report_digest_v2(&report);
        return report;
    }
    let mut written = 0;
    let mut duplicates = 0;
    let (preregistered_written, preregistered_duplicates) =
        match persist_preregistration_v2(root, &audit, &split, &registration) {
            Ok(value) => value,
            Err(_) => {
                let mut report = base_report_v2(
                    mode,
                    MomentumMambaRepairExecutionStatusV2::TechnicalFailure,
                    true,
                    stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                        == active_before,
                );
                report.collapse_audit = Some(audit);
                report.repair_split = Some(split);
                report.repair_registration = Some(registration);
                report.report_digest = report_digest_v2(&report);
                return report;
            }
        };
    written += preregistered_written;
    duplicates += preregistered_duplicates;
    let reopened = reopen_preregistration_v2(root);
    let reopened_invalid = match reopened.as_ref() {
        Ok((stored_audit, stored_split, stored_registration)) => {
            stored_audit != &audit || stored_split != &split || stored_registration != &registration
        }
        Err(_) => true,
    };
    if reopened_invalid {
        let mut report = base_report_v2(
            mode,
            MomentumMambaRepairExecutionStatusV2::TechnicalFailure,
            true,
            stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before,
        );
        report.collapse_audit = Some(audit);
        report.repair_split = Some(split);
        report.repair_registration = Some(registration);
        report.artifacts_written = written;
        report.duplicate_artifact_count = duplicates;
        report.report_digest = report_digest_v2(&report);
        return report;
    }
    let experiment =
        match run_repair_experiment_v2(&state, &audit, &split, &registration, reservation) {
            Ok(value) => value,
            Err(_) => {
                let mut report = base_report_v2(
                    mode,
                    MomentumMambaRepairExecutionStatusV2::TechnicalFailure,
                    true,
                    stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                        == active_before,
                );
                report.collapse_audit = Some(audit);
                report.repair_split = Some(split);
                report.repair_registration = Some(registration);
                report.artifacts_written = written;
                report.duplicate_artifact_count = duplicates;
                report.report_digest = report_digest_v2(&report);
                return report;
            }
        };
    let mut journal = MomentumMambaRepairJournalV2 {
        journal_version: JOURNAL_VERSION_V2.to_string(),
        agent_id: AGENT_ID_V2.to_string(),
        collapse_audit_digest: audit.audit_digest.clone(),
        repair_split_digest: split.split_digest.clone(),
        repair_registration_digest: registration.registration_digest.clone(),
        family_digest: Some(experiment.family.family_digest.clone()),
        roster_digest: experiment
            .roster
            .as_ref()
            .map(|value| value.roster_digest.clone()),
        evaluation_registration_digest: experiment
            .evaluation_registration
            .as_ref()
            .map(|value| value.registration_digest.clone()),
        prior_validation_used_for_repair_qualification: false,
        warm_start_from_v1: false,
        v1_head_reused: false,
        fresh_deterministic_initialization: true,
        status: MomentumMambaRepairExecutionStatusV2::Executed,
        journal_digest: String::new(),
    };
    journal.journal_digest = journal_digest_v2(&journal);
    let (experiment_written, experiment_duplicates) =
        match persist_experiment_v2(root, &experiment, &journal, reservation) {
            Ok(value) => value,
            Err(_) => {
                let mut report = base_report_v2(
                    mode,
                    MomentumMambaRepairExecutionStatusV2::TechnicalFailure,
                    true,
                    stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                        == active_before,
                );
                report.collapse_audit = Some(audit);
                report.repair_split = Some(split);
                report.repair_registration = Some(registration);
                report.family = Some(experiment.family);
                report.artifacts_written = written;
                report.duplicate_artifact_count = duplicates;
                report.report_digest = report_digest_v2(&report);
                return report;
            }
        };
    written += experiment_written;
    duplicates += experiment_duplicates;
    let persisted = match read_persisted_experiment_v2(root, reservation) {
        Ok(value) => value,
        Err(_) => {
            let mut report = base_report_v2(
                mode,
                MomentumMambaRepairExecutionStatusV2::TechnicalFailure,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
            report.collapse_audit = Some(audit);
            report.repair_split = Some(split);
            report.repair_registration = Some(registration);
            report.artifacts_written = written;
            report.duplicate_artifact_count = duplicates;
            report.report_digest = report_digest_v2(&report);
            return report;
        }
    };
    if persisted.0.family != experiment.family
        || persisted.0.roster != experiment.roster
        || persisted.0.evaluation_registration != experiment.evaluation_registration
        || persisted.1 != journal
    {
        let mut report = base_report_v2(
            mode,
            MomentumMambaRepairExecutionStatusV2::TechnicalFailure,
            true,
            stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before,
        );
        report.storage_failure_count = 1;
        report.report_digest = report_digest_v2(&report);
        return report;
    }
    let mut protected_after = Vec::new();
    let protected_artifacts_unchanged =
        collect_protected_artifacts_v2(root, root, &mut protected_after).is_ok()
            && protected_before == protected_after;
    let active_state_unchanged =
        stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before;
    let status = if persisted_before.is_some() {
        MomentumMambaRepairExecutionStatusV2::AlreadyExecuted
    } else {
        MomentumMambaRepairExecutionStatusV2::Executed
    };
    let mut report = base_report_v2(
        mode,
        status,
        protected_artifacts_unchanged,
        active_state_unchanged,
    );
    report.collapse_audit = Some(audit);
    report.repair_split = Some(split);
    report.repair_registration = Some(registration);
    report.family = Some(experiment.family);
    report.roster = experiment.roster;
    report.roster_status = experiment.roster_status;
    report.evaluation_registration = experiment.evaluation_registration;
    report.evaluation_registration_status = experiment.evaluation_registration_status;
    report.journal = Some(journal);
    report.artifacts_written = written;
    report.duplicate_artifact_count = duplicates;
    report.storage_failure_count =
        usize::from(!protected_artifacts_unchanged || !active_state_unchanged);
    report.report_digest = report_digest_v2(&report);
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn audit_fixture() -> MomentumMambaCollapseAuditV2 {
        let mut value = MomentumMambaCollapseAuditV2 {
            audit_version: AUDIT_VERSION_V2.to_string(),
            agent_id: AGENT_ID_V2.to_string(),
            source_family_digest: "family-v1".to_string(),
            failed_participant_digest: "failed-participant".to_string(),
            failed_qualification_receipt_digest: "failed-receipt".to_string(),
            training_range_digest: "training-range".to_string(),
            prior_validation_range_digest: "prior-validation-range".to_string(),
            encoder_digest: "encoder".to_string(),
            representation_normalizer_digest: "representation-normalizer".to_string(),
            feature_normalizer_digest: "feature-normalizer".to_string(),
            head_parameter_digest: "head".to_string(),
            representation_diagnostic_digest: "representation-diagnostic".to_string(),
            optimization_diagnostic_digest: "optimization-diagnostic".to_string(),
            probability_diagnostic_digest: "probability-diagnostic".to_string(),
            class_balance_diagnostic_digest: "class-balance-diagnostic".to_string(),
            root_causes: vec![MomentumMambaCollapseRootCauseV2::ProbabilitySingleSided],
            repair_capability_status:
                MomentumMambaRepairCapabilityStatusV2::RepairableWithBoundedHeadRegularization,
            audit_digest: String::new(),
        };
        value.audit_digest = audit_digest_v2(&value);
        value
    }

    fn split_fixture() -> MomentumMambaRepairSplitV2 {
        let mut value = MomentumMambaRepairSplitV2 {
            split_version: SPLIT_VERSION_V2.to_string(),
            source_snapshot_digest: "snapshot".to_string(),
            prior_usage_ledger_digest: "ledger".to_string(),
            repair_training_range: IndexRangeV0 { start: 0, end: 160 },
            repair_purge_range: IndexRangeV0 {
                start: 160,
                end: 176,
            },
            fresh_repair_validation_range: IndexRangeV0 {
                start: 176,
                end: 200,
            },
            remaining_reserved_range: Some(IndexRangeV0 {
                start: 200,
                end: 312,
            }),
            label_horizon: 1,
            minimum_validation_samples: 4,
            prior_validation_overlap_count: 0,
            prospective_overlap_count: 0,
            future_evaluation_overlap_count: 0,
            split_digest: String::new(),
        };
        value.split_digest = split_digest_v2(&value);
        value
    }

    fn variant_fixture(id: &str) -> MomentumMambaRepairVariantConfigV2 {
        let mut value = MomentumMambaRepairVariantConfigV2 {
            variant_id: id.to_string(),
            pooling_policy: SequencePooling::LastOutput,
            learning_rate_bits: 0.02f32.to_bits(),
            l2_regularization_bits: 0.001f32.to_bits(),
            maximum_epochs: 60,
            class_weight_policy: "none-training-only".to_string(),
            initialization_seed: 78,
            encoder_frozen: true,
            feature_policy_digest: "feature-policy".to_string(),
            label_policy_digest: "label-policy".to_string(),
            training_policy_digest: "training-policy".to_string(),
            variant_config_digest: String::new(),
        };
        value.variant_config_digest = variant_digest_v2(&value);
        value
    }

    fn registration_fixture() -> MomentumMambaRepairRegistrationV2 {
        let mut value = MomentumMambaRepairRegistrationV2 {
            registration_version: REGISTRATION_VERSION_V2.to_string(),
            agent_id: AGENT_ID_V2.to_string(),
            source_snapshot_digest: "snapshot".to_string(),
            canonical_intent_digest: "intent".to_string(),
            canonical_view_digest: "view".to_string(),
            source_family_digest: "family-v1".to_string(),
            failed_participant_digest: "failed-participant".to_string(),
            collapse_audit_digest: "audit".to_string(),
            repair_split_digest: "split".to_string(),
            allowed_variant_configs: vec![variant_fixture("control")],
            maximum_repair_variants: MAXIMUM_REPAIR_VARIANTS_V2,
            fresh_validation_hidden: true,
            historical_test_forbidden: true,
            future_evaluation_forbidden: true,
            winner_selection_forbidden: true,
            active_promotion_forbidden: true,
            reward_application_forbidden: true,
            registration_digest: String::new(),
        };
        value.registration_digest = repair_registration_digest_v2(&value);
        value
    }

    fn participant_pair(
        role: ParticipantQualificationRoleV2,
        model: &str,
        status: ValidationQualificationStatusV2,
        metric: &str,
    ) -> (
        FrozenCandidateParticipantV2,
        ParticipantValidationQualificationV2,
    ) {
        make_participant_and_receipt_v2(
            role,
            model.to_string(),
            (role == ParticipantQualificationRoleV2::LearnedCandidate)
                .then(|| "variant".to_string()),
            format!("parameter-{model}"),
            "normalizer".to_string(),
            (role == ParticipantQualificationRoleV2::LearnedCandidate)
                .then(|| "encoder".to_string()),
            "training-policy".to_string(),
            "initialization".to_string(),
            "snapshot",
            "training-range",
            "validation-range",
            "validation-timestamps",
            "feature-policy",
            "label-policy",
            metric.to_string(),
            status,
        )
        .unwrap()
    }

    fn family_fixture(
        learned_status: ValidationQualificationStatusV2,
    ) -> MomentumCandidateFamilyV2 {
        let pairs = vec![
            participant_pair(
                ParticipantQualificationRoleV2::LearnedCandidate,
                "Mamba",
                learned_status,
                "learned-metric",
            ),
            participant_pair(
                ParticipantQualificationRoleV2::LinearComparator,
                "Linear",
                ValidationQualificationStatusV2::Qualified,
                "linear-metric",
            ),
            participant_pair(
                ParticipantQualificationRoleV2::ConstantBenchmark,
                "Constant",
                ValidationQualificationStatusV2::BenchmarkQualified,
                "constant-metric",
            ),
        ];
        let mut participants = pairs
            .iter()
            .map(|(participant, _)| participant.clone())
            .collect::<Vec<_>>();
        let mut qualification_receipts = pairs
            .into_iter()
            .map(|(_, receipt)| receipt)
            .collect::<Vec<_>>();
        participants.sort_by(|left, right| left.participant_id.cmp(&right.participant_id));
        qualification_receipts
            .sort_by(|left, right| left.participant_digest.cmp(&right.participant_digest));
        let qualified_learned_participant_count =
            usize::from(learned_status == ValidationQualificationStatusV2::Qualified);
        let mut family = MomentumCandidateFamilyV2 {
            family_version: FAMILY_VERSION_V2.to_string(),
            agent_id: AGENT_ID_V2.to_string(),
            source_snapshot_digest: "snapshot".to_string(),
            canonical_view_digest: "view".to_string(),
            repair_registration_digest: "registration".to_string(),
            repair_split_digest: "split".to_string(),
            collapse_audit_digest: "audit".to_string(),
            participants,
            qualification_receipts,
            learned_participant_count: 1,
            qualified_learned_participant_count,
            qualified_comparator_count: 2,
            winner_selected: false,
            historical_test_accessed: false,
            eligible_for_active_committee: false,
            eligible_for_promotion: false,
            eligible_for_reward: false,
            family_digest: String::new(),
        };
        family.family_digest = family_digest_v2(&family);
        family
    }

    fn reservation_fixture() -> ProtectedEvaluationReservationV1 {
        ProtectedEvaluationReservationV1 {
            protected_registration_digests: vec!["protected-a".to_string()],
            reserved_timestamp_ms: vec![200],
            cadence_ms: 10,
            provider_finality_boundary_ms: 210,
            reservation_digest: "reservation".to_string(),
        }
    }

    fn evaluation_fixture(
        family: &MomentumCandidateFamilyV2,
        roster: &MomentumFutureEvaluationRosterV2,
        reservation: &ProtectedEvaluationReservationV1,
    ) -> MomentumFutureEvaluationRegistrationV2 {
        let included = roster
            .qualified_learned_participant_digests
            .iter()
            .chain(&roster.qualified_comparator_digests)
            .collect::<BTreeSet<_>>();
        let mut value = MomentumFutureEvaluationRegistrationV2 {
            registration_version: EVALUATION_VERSION_V2.to_string(),
            agent_id: AGENT_ID_V2.to_string(),
            family_digest: family.family_digest.clone(),
            roster_digest: roster.roster_digest.clone(),
            repair_registration_digest: family.repair_registration_digest.clone(),
            collapse_audit_digest: family.collapse_audit_digest.clone(),
            qualification_receipt_digests: sorted_unique_strings(
                family
                    .qualification_receipts
                    .iter()
                    .filter(|receipt| included.contains(&receipt.participant_digest))
                    .map(|receipt| receipt.receipt_digest.clone())
                    .collect(),
            ),
            source_snapshot_digest: family.source_snapshot_digest.clone(),
            source_boundary_timestamp_ms: 100,
            protected_registration_digests: reservation.protected_registration_digests.clone(),
            protected_timestamp_ms: reservation.reserved_timestamp_ms.clone(),
            prior_reserved_range_digests: vec!["prior-range".to_string()],
            provider_finality_boundary_ms: reservation.provider_finality_boundary_ms,
            minimum_accepted_timestamp_ms: 210,
            labels_hidden_until_opening: true,
            probabilities_hidden_until_opening: true,
            one_time_opening_required: true,
            winner_selection_forbidden_before_opening: true,
            active_promotion_forbidden: true,
            reward_application_forbidden: true,
            maximum_requests: 1,
            maximum_concurrency: 1,
            maximum_retries: 0,
            status: MomentumFutureEvaluationRegistrationStatusV2::Registered,
            registration_digest: String::new(),
        };
        value.registration_digest = evaluation_registration_digest_v2(&value);
        value
    }

    fn journal_fixture(family: &MomentumCandidateFamilyV2) -> MomentumMambaRepairJournalV2 {
        let mut value = MomentumMambaRepairJournalV2 {
            journal_version: JOURNAL_VERSION_V2.to_string(),
            agent_id: AGENT_ID_V2.to_string(),
            collapse_audit_digest: family.collapse_audit_digest.clone(),
            repair_split_digest: family.repair_split_digest.clone(),
            repair_registration_digest: family.repair_registration_digest.clone(),
            family_digest: Some(family.family_digest.clone()),
            roster_digest: None,
            evaluation_registration_digest: None,
            prior_validation_used_for_repair_qualification: false,
            warm_start_from_v1: false,
            v1_head_reused: false,
            fresh_deterministic_initialization: true,
            status: MomentumMambaRepairExecutionStatusV2::Executed,
            journal_digest: String::new(),
        };
        value.journal_digest = journal_digest_v2(&value);
        value
    }

    #[test]
    fn collapse_audit_preserves_exact_failed_participant_binding() {
        let value = audit_fixture();
        assert_eq!(value.failed_participant_digest, "failed-participant");
        assert!(validate_audit_v2(&value).is_ok());
        let mut changed = value.clone();
        changed.failed_participant_digest = "other".to_string();
        assert!(validate_audit_v2(&changed).is_err());
    }

    #[test]
    fn representation_and_head_collapse_diagnostics_are_distinct() {
        let rows = vec![
            super::super::EncodedTrainingExampleV0 {
                representation: vec![1.0, 2.0],
                label: 0.0,
                snapshot_ids: vec!["s".to_string()],
            },
            super::super::EncodedTrainingExampleV0 {
                representation: vec![1.0, 3.0],
                label: 1.0,
                snapshot_ids: vec!["s".to_string()],
            },
        ];
        let diagnostic = representation_diagnostic_v2(&rows, "not-applied");
        assert_eq!(diagnostic.constant_dimension_count, 1);
        assert_ne!(norm_status(0.0), NormStatusV2::Material);
        assert_ne!(diagnostic.diagnostic_digest, "");
    }

    #[test]
    fn probability_collapse_subtypes_are_preserved() {
        let low = probability_diagnostic_v2(&vec![0.01; 8]);
        assert!(
            low.collapse_subtypes
                .contains(&MomentumMambaCollapseRootCauseV2::ProbabilityNearConstant)
        );
        assert!(
            low.collapse_subtypes
                .contains(&MomentumMambaCollapseRootCauseV2::ProbabilitySaturatedLow)
        );
        let high = probability_diagnostic_v2(&vec![0.99; 8]);
        assert!(
            high.collapse_subtypes
                .contains(&MomentumMambaCollapseRootCauseV2::ProbabilitySaturatedHigh)
        );
    }

    #[test]
    fn fresh_split_uses_only_prior_reserved_evidence_with_full_purge() {
        let config = MomentumLearningCampaignConfigV0::default();
        let (training, purge, validation, remaining) = bounded_repair_ranges_v2(
            &IndexRangeV0 {
                start: 96,
                end: 312,
            },
            &config,
        )
        .unwrap();
        assert_eq!(training, IndexRangeV0 { start: 0, end: 160 });
        assert_eq!(
            purge,
            IndexRangeV0 {
                start: 160,
                end: 176
            }
        );
        assert_eq!(
            validation,
            IndexRangeV0 {
                start: 176,
                end: 200
            }
        );
        assert_eq!(
            remaining,
            Some(IndexRangeV0 {
                start: 200,
                end: 312
            })
        );
        assert_eq!(
            ranges_overlap(&IndexRangeV0 { start: 72, end: 96 }, &validation),
            0
        );
    }

    #[test]
    fn insufficient_fresh_rows_block_without_shrinking_the_gate() {
        let config = MomentumLearningCampaignConfigV0::default();
        let result = bounded_repair_ranges_v2(
            &IndexRangeV0 {
                start: 96,
                end: 150,
            },
            &config,
        );
        assert_eq!(
            result.unwrap_err(),
            MomentumMambaRepairCapabilityStatusV2::FreshValidationInsufficient
        );
    }

    #[test]
    fn split_rejects_prior_validation_and_prospective_overlap() {
        let mut value = split_fixture();
        value.prior_validation_overlap_count = 1;
        value.split_digest = split_digest_v2(&value);
        assert!(validate_split_v2(&value).is_err());
        let mut value = split_fixture();
        value.prospective_overlap_count = 1;
        value.split_digest = split_digest_v2(&value);
        assert!(validate_split_v2(&value).is_err());
    }

    #[test]
    fn preregistration_caps_variants_and_freezes_concrete_configs() {
        let mut value = registration_fixture();
        value.allowed_variant_configs = (0..=MAXIMUM_REPAIR_VARIANTS_V2)
            .map(|index| variant_fixture(&format!("variant-{index}")))
            .collect();
        value.registration_digest = repair_registration_digest_v2(&value);
        assert!(validate_registration_v2(&value).is_err());
        let value = registration_fixture();
        assert!(value.fresh_validation_hidden);
        assert!(value.historical_test_forbidden);
        assert!(value.winner_selection_forbidden);
    }

    #[test]
    fn variant_rejects_nonfinite_unbounded_and_unsupported_values() {
        let mut value = variant_fixture("invalid");
        value.learning_rate_bits = f32::NAN.to_bits();
        value.variant_config_digest = variant_digest_v2(&value);
        assert!(validate_variant_v2(&value).is_err());
        let mut value = variant_fixture("invalid");
        value.maximum_epochs = 129;
        value.variant_config_digest = variant_digest_v2(&value);
        assert!(validate_variant_v2(&value).is_err());
        let mut value = variant_fixture("invalid");
        value.class_weight_policy = "validation-derived".to_string();
        value.variant_config_digest = variant_digest_v2(&value);
        assert!(validate_variant_v2(&value).is_err());
    }

    #[test]
    fn encoder_mutation_and_v1_head_reuse_reject() {
        let (mut participant, _) = participant_pair(
            ParticipantQualificationRoleV2::LearnedCandidate,
            "Mamba",
            ValidationQualificationStatusV2::Qualified,
            "metric",
        );
        participant.encoder_frozen = false;
        participant.participant_digest = participant_digest_v2(&participant);
        assert!(validate_participant_v2(&participant).is_err());
        participant.encoder_frozen = true;
        participant.v1_head_reused = true;
        participant.participant_digest = participant_digest_v2(&participant);
        assert!(validate_participant_v2(&participant).is_err());
    }

    #[test]
    fn every_participant_requires_fresh_deterministic_initialization() {
        let (mut participant, _) = participant_pair(
            ParticipantQualificationRoleV2::LinearComparator,
            "Linear",
            ValidationQualificationStatusV2::Qualified,
            "metric",
        );
        participant.fresh_deterministic_initialization = false;
        participant.participant_digest = participant_digest_v2(&participant);
        assert!(validate_participant_v2(&participant).is_err());
    }

    #[test]
    fn constant_benchmark_is_not_rejected_for_zero_variance() {
        let metric = EvaluationMetricsV0 {
            brier_score: 0.25,
            sample_count: 8,
            accuracy: 0.5,
            positive_label_rate: 0.5,
            mean_predicted_probability: 0.5,
            high_confidence_error_count: 0,
            abstention_count: 0,
            calibration_buckets: vec![],
        };
        assert_eq!(
            comparator_qualification_v2(
                ParticipantQualificationRoleV2::ConstantBenchmark,
                &metric,
                &vec![0.5; 8],
                4,
            ),
            ValidationQualificationStatusV2::BenchmarkQualified
        );
    }

    #[test]
    fn learned_candidates_still_require_noncollapse() {
        let metric = EvaluationMetricsV0 {
            brier_score: 0.25,
            sample_count: 8,
            accuracy: 0.5,
            positive_label_rate: 0.5,
            mean_predicted_probability: 0.5,
            high_confidence_error_count: 0,
            abstention_count: 0,
            calibration_buckets: vec![],
        };
        let representations = (0..8)
            .map(|index| super::super::EncodedTrainingExampleV0 {
                representation: vec![
                    if index % 2 == 0 { -1.0 } else { 1.0 },
                    if (index / 2) % 2 == 0 { -1.0 } else { 1.0 },
                ],
                label: (index % 2) as f32,
                snapshot_ids: vec!["s".to_string()],
            })
            .collect::<Vec<_>>();
        let diagnostic = representation_diagnostic_v2(&representations, "not-applied");
        assert_eq!(
            learned_qualification_v2(&metric, &vec![0.5; 8], &diagnostic, 4, true),
            ValidationQualificationStatusV2::RejectedProbabilityCollapse
        );
    }

    #[test]
    fn validation_parameter_updates_and_boundary_reads_must_be_zero() {
        let (_, mut receipt) = participant_pair(
            ParticipantQualificationRoleV2::LinearComparator,
            "Linear",
            ValidationQualificationStatusV2::Qualified,
            "metric",
        );
        receipt.validation_parameter_updates = 1;
        receipt.receipt_digest = qualification_digest_v2(&receipt);
        assert!(validate_qualification_v2(&receipt).is_err());
        receipt.validation_parameter_updates = 0;
        receipt.historical_test_reads = 1;
        receipt.receipt_digest = qualification_digest_v2(&receipt);
        assert!(validate_qualification_v2(&receipt).is_err());
    }

    #[test]
    fn all_participants_share_exact_validation_timestamps() {
        let mut family = family_fixture(ValidationQualificationStatusV2::Qualified);
        family.participants[0].validation_timestamp_digest = "different".to_string();
        family.participants[0].participant_digest = participant_digest_v2(&family.participants[0]);
        family.family_digest = family_digest_v2(&family);
        assert!(validate_family_v2(&family).is_err());
    }

    #[test]
    fn rejected_variants_stay_outside_roster_and_all_qualified_enter() {
        let rejected = family_fixture(ValidationQualificationStatusV2::RejectedProbabilityCollapse);
        let (roster, status) = derive_roster_v2(&rejected).unwrap();
        assert!(roster.is_none());
        assert_eq!(
            status,
            MomentumFutureEvaluationRosterStatusV2::NoQualifiedLearnedParticipant
        );
        let qualified = family_fixture(ValidationQualificationStatusV2::Qualified);
        let (roster, status) = derive_roster_v2(&qualified).unwrap();
        let roster = roster.unwrap();
        assert_eq!(status, MomentumFutureEvaluationRosterStatusV2::Registered);
        assert_eq!(roster.qualified_learned_participant_digests.len(), 1);
        assert_eq!(roster.qualified_comparator_digests.len(), 2);
    }

    #[test]
    fn baselines_only_future_registration_is_forbidden() {
        let family = family_fixture(ValidationQualificationStatusV2::RejectedProbabilityCollapse);
        let (roster, status) = derive_roster_v2(&family).unwrap();
        assert!(roster.is_none());
        assert_eq!(
            status,
            MomentumFutureEvaluationRosterStatusV2::NoQualifiedLearnedParticipant
        );
    }

    #[test]
    fn roster_uses_status_only_and_never_metric_ranking() {
        let mut family = family_fixture(ValidationQualificationStatusV2::Qualified);
        let first = derive_roster_v2(&family).unwrap().0.unwrap();
        for receipt in &mut family.qualification_receipts {
            receipt.private_metric_digest = format!("changed-{}", receipt.private_metric_digest);
            receipt.receipt_digest = qualification_digest_v2(receipt);
        }
        family.family_digest = family_digest_v2(&family);
        let second = derive_roster_v2(&family).unwrap().0.unwrap();
        assert_eq!(
            first.qualified_learned_participant_digests,
            second.qualified_learned_participant_digests
        );
        assert_eq!(
            first.qualified_comparator_digests,
            second.qualified_comparator_digests
        );
    }

    #[test]
    fn participant_identity_excludes_validation_metrics() {
        let (first, first_receipt) = participant_pair(
            ParticipantQualificationRoleV2::LearnedCandidate,
            "Mamba",
            ValidationQualificationStatusV2::Qualified,
            "metric-a",
        );
        let (second, second_receipt) = participant_pair(
            ParticipantQualificationRoleV2::LearnedCandidate,
            "Mamba",
            ValidationQualificationStatusV2::Qualified,
            "metric-b",
        );
        assert_eq!(first.participant_digest, second.participant_digest);
        assert_ne!(first_receipt.receipt_digest, second_receipt.receipt_digest);
    }

    #[test]
    fn variants_cannot_be_added_after_family_binding() {
        let family = family_fixture(ValidationQualificationStatusV2::Qualified);
        let mut registration = registration_fixture();
        let original = registration.registration_digest.clone();
        registration
            .allowed_variant_configs
            .push(variant_fixture("later"));
        registration.registration_digest = repair_registration_digest_v2(&registration);
        assert_ne!(registration.registration_digest, original);
        assert_ne!(
            family.repair_registration_digest,
            registration.registration_digest
        );
    }

    #[test]
    fn family_forbids_winner_historical_test_and_authority_eligibility() {
        let mut family = family_fixture(ValidationQualificationStatusV2::Qualified);
        family.winner_selected = true;
        family.family_digest = family_digest_v2(&family);
        assert!(validate_family_v2(&family).is_err());
        let mut family = family_fixture(ValidationQualificationStatusV2::Qualified);
        family.historical_test_accessed = true;
        family.family_digest = family_digest_v2(&family);
        assert!(validate_family_v2(&family).is_err());
        let mut family = family_fixture(ValidationQualificationStatusV2::Qualified);
        family.eligible_for_reward = true;
        family.family_digest = family_digest_v2(&family);
        assert!(validate_family_v2(&family).is_err());
    }

    #[test]
    fn future_registration_preserves_every_exclusion_and_safety_gate() {
        let family = family_fixture(ValidationQualificationStatusV2::Qualified);
        let roster = derive_roster_v2(&family).unwrap().0.unwrap();
        let reservation = reservation_fixture();
        let value = evaluation_fixture(&family, &roster, &reservation);
        assert!(
            validate_evaluation_registration_v2(&value, &family, &roster, &reservation).is_ok()
        );
        let mut changed = value.clone();
        changed.protected_timestamp_ms.clear();
        changed.registration_digest = evaluation_registration_digest_v2(&changed);
        assert!(
            validate_evaluation_registration_v2(&changed, &family, &roster, &reservation).is_err()
        );
        assert!(value.labels_hidden_until_opening);
        assert!(value.probabilities_hidden_until_opening);
        assert_eq!(value.maximum_concurrency, 1);
        assert_eq!(value.maximum_retries, 0);
    }

    #[test]
    fn journal_forbids_prior_validation_qualification_and_warm_start() {
        let family = family_fixture(ValidationQualificationStatusV2::Qualified);
        let mut journal = journal_fixture(&family);
        assert!(validate_journal_v2(&journal).is_ok());
        journal.prior_validation_used_for_repair_qualification = true;
        journal.journal_digest = journal_digest_v2(&journal);
        assert!(validate_journal_v2(&journal).is_err());
    }

    #[test]
    fn all_network_and_authority_counters_are_zero() {
        let value = zero_safety_counters_v2();
        assert_eq!(value.active_committee_count, 3);
        assert_eq!(
            value.network_requests
                + value.transport_constructions
                + value.credential_reads
                + value.prospective_row_reads
                + value.prospective_label_openings
                + value.future_evaluation_reads
                + value.historical_test_reads
                + value.active_model_changes
                + value.chair_decisions
                + value.votes
                + value.reward_applications
                + value.penalty_applications
                + value.voice_changes
                + value.cooldowns_started
                + value.promotions
                + value.quarantines
                + value.executions,
            0
        );
    }

    #[test]
    fn all_manual_protobuf_types_round_trip() {
        let audit = audit_fixture();
        assert_eq!(
            decode_momentum_mamba_collapse_audit_protobuf_v2(
                &encode_momentum_mamba_collapse_audit_protobuf_v2(&audit).unwrap()
            )
            .unwrap(),
            audit
        );
        let split = split_fixture();
        assert_eq!(
            decode_momentum_mamba_repair_split_protobuf_v2(
                &encode_momentum_mamba_repair_split_protobuf_v2(&split).unwrap()
            )
            .unwrap(),
            split
        );
        let registration = registration_fixture();
        assert_eq!(
            decode_momentum_mamba_repair_registration_protobuf_v2(
                &encode_momentum_mamba_repair_registration_protobuf_v2(&registration).unwrap()
            )
            .unwrap(),
            registration
        );
        let family = family_fixture(ValidationQualificationStatusV2::Qualified);
        for participant in &family.participants {
            assert_eq!(
                decode_momentum_candidate_participant_protobuf_v2(
                    &encode_momentum_candidate_participant_protobuf_v2(participant).unwrap()
                )
                .unwrap(),
                *participant
            );
        }
        for receipt in &family.qualification_receipts {
            assert_eq!(
                decode_momentum_qualification_receipt_protobuf_v2(
                    &encode_momentum_qualification_receipt_protobuf_v2(receipt).unwrap()
                )
                .unwrap(),
                *receipt
            );
        }
        assert_eq!(
            decode_momentum_candidate_family_protobuf_v2(
                &encode_momentum_candidate_family_protobuf_v2(&family).unwrap()
            )
            .unwrap(),
            family
        );
        let roster = derive_roster_v2(&family).unwrap().0.unwrap();
        assert_eq!(
            decode_momentum_future_evaluation_roster_protobuf_v2(
                &encode_momentum_future_evaluation_roster_protobuf_v2(&roster, &family).unwrap(),
                &family,
            )
            .unwrap(),
            roster
        );
        let reservation = reservation_fixture();
        let evaluation = evaluation_fixture(&family, &roster, &reservation);
        assert_eq!(
            decode_momentum_future_evaluation_registration_protobuf_v2(
                &encode_momentum_future_evaluation_registration_protobuf_v2(
                    &evaluation,
                    &family,
                    &roster,
                    &reservation,
                )
                .unwrap(),
                &family,
                &roster,
                &reservation,
            )
            .unwrap(),
            evaluation
        );
        let journal = journal_fixture(&family);
        assert_eq!(
            decode_momentum_mamba_repair_journal_protobuf_v2(
                &encode_momentum_mamba_repair_journal_protobuf_v2(&journal).unwrap()
            )
            .unwrap(),
            journal
        );
    }

    #[test]
    fn protobuf_corruption_rejects() {
        let mut bytes = encode_momentum_mamba_collapse_audit_protobuf_v2(&audit_fixture()).unwrap();
        bytes.truncate(bytes.len() / 2);
        assert!(decode_momentum_mamba_collapse_audit_protobuf_v2(&bytes).is_err());
        let mut bytes = encode_momentum_mamba_repair_split_protobuf_v2(&split_fixture()).unwrap();
        bytes[0] ^= 0xff;
        assert!(decode_momentum_mamba_repair_split_protobuf_v2(&bytes).is_err());
    }

    #[test]
    fn preregistration_persists_before_any_participant_and_replays_idempotently() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "soma-momentum-repair-v2-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let audit = audit_fixture();
        let split = split_fixture();
        let registration = registration_fixture();
        assert_eq!(
            persist_preregistration_v2(&root, &audit, &split, &registration).unwrap(),
            (3, 0)
        );
        assert!(!v2_root(&root).join("participants").exists());
        assert_eq!(
            reopen_preregistration_v2(&root).unwrap(),
            (audit.clone(), split.clone(), registration.clone())
        );
        assert_eq!(
            persist_preregistration_v2(&root, &audit, &split, &registration).unwrap(),
            (0, 3)
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn protected_collection_excludes_only_additive_v2_artifacts() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "soma-momentum-protected-v2-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("v1")).unwrap();
        fs::create_dir_all(root.join("v2")).unwrap();
        fs::write(root.join("v1/protected.pb"), b"protected").unwrap();
        let mut before = Vec::new();
        collect_protected_artifacts_v2(&root, &root, &mut before).unwrap();
        fs::write(root.join("v2/additive.pb"), b"additive").unwrap();
        let mut after = Vec::new();
        collect_protected_artifacts_v2(&root, &root, &mut after).unwrap();
        assert_eq!(before, after);
        fs::remove_dir_all(&root).unwrap();
    }
}
