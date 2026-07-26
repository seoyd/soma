//! Offline, additive raw-feature Momentum research path V4.
//!
//! This module closes only the current frozen-Mamba evidence/policy path. It
//! has no network, active-committee, reward, voting, or execution authority.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use prost::Message;
use serde::{Deserialize, Serialize};

use crate::league::HistoricalOhlcvRow;
use crate::{core::stable_hash_string, data::DataSnapshot, league::canonical_current_agent_states};

use super::agent_learning_session::{
    AgentPrivateLearningArtifactWriteStatusV0, atomic_write_verified_v0,
};
use super::momentum_mamba_repair::{
    MomentumCandidateFamilyV2, MomentumMambaRepairSplitV2, V1FrozenStateV2,
    candles_from_snapshot_prefix, decode_momentum_candidate_family_protobuf_v2,
    decode_momentum_mamba_repair_split_protobuf_v2, load_v1_frozen_state_v2,
};
use super::momentum_mamba_representation::{
    MomentumRepresentationFamilyV3, MomentumRepresentationParticipantRoleV3,
    MomentumRepresentationQualificationStatusV3, MomentumRepresentationRouteDecisionArtifactV3,
    MomentumRepresentationRouteDecisionV3, MomentumRepresentationSplitV3,
    decode_momentum_representation_decision_protobuf_v3,
    decode_momentum_representation_family_protobuf_v3,
    decode_momentum_representation_split_protobuf_v3,
};
use super::{
    AgentPrivateLearningRunModeV0, EncodedTrainingExampleV0, EvaluationMetricsV0,
    FeatureNormalizerV0, HeadTrainingConfigV0, IndexRangeV0, LogisticPredictionHeadV0,
    ModelAgentDeploymentStatus, MomentumCandleV0, MomentumFeatureRowV0,
    MomentumLearningCampaignConfigV0, MomentumSequenceConfigV0, ProtectedEvaluationReservationV1,
    RepresentationNormalizerV0, SequenceExampleV0, apply_sgd_v0, brier_loss_and_gradients_v0,
    build_momentum_features_v0, build_momentum_sequence_examples_v0, evaluate_head_v0,
    evaluate_probabilities_v0,
};

const AGENT_ID_V4: &str = "momentum_trend_fast";
const CLOSURE_VERSION_V4: &str = "momentum-frozen-mamba-path-closure-v4";
const SPLIT_VERSION_V4: &str = "momentum-raw-feature-split-v4";
const VALIDATION_YIELD_AUDIT_VERSION_V4: &str = "momentum-validation-yield-audit-v4";
const REGISTRATION_VERSION_V4: &str = "momentum-raw-feature-registration-v4";
const PARTICIPANT_VERSION_V4: &str = "frozen-candidate-participant-v4";
const RECEIPT_VERSION_V4: &str = "momentum-raw-feature-qualification-v4";
const FAMILY_VERSION_V4: &str = "momentum-raw-feature-family-v4";
const DECISION_VERSION_V4: &str = "momentum-raw-feature-path-decision-v4";
const ROSTER_VERSION_V4: &str = "momentum-raw-feature-future-roster-v4";
const EVALUATION_VERSION_V4: &str = "momentum-raw-feature-evaluation-registration-v4";
const JOURNAL_VERSION_V4: &str = "momentum-raw-feature-journal-v4";
const MAXIMUM_LEARNED_PARTICIPANTS_V4: usize = 2;
const MATERIAL_INTERACTION_EFFECT_BITS_V4: u32 = 0.001_f32.to_bits();
const DETECTABLE_INTERACTION_EFFECT_BITS_V4: u32 = 0.000001_f32.to_bits();

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumFrozenMambaClosureDecisionV4 {
    ClosedForCurrentEvidenceAndPolicy,
    ClosureIntegrityFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumFrozenMambaPathClosureV4 {
    pub closure_version: String,
    pub agent_id: String,
    pub source_snapshot_digest: String,
    pub canonical_intent_digest: String,
    pub canonical_view_digest: String,
    pub v1_family_digest: String,
    pub v2_family_digest: String,
    pub v3_family_digest: String,
    pub v3_route_decision_digest: String,
    pub frozen_encoder_digest: String,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub genuine_mamba_qualified_count: usize,
    pub head_only_repair_forbidden: bool,
    pub frozen_representation_sweep_forbidden: bool,
    pub frozen_mamba_parent_use_forbidden: bool,
    pub reopening_requires_new_encoder_identity: bool,
    pub reopening_requires_new_evidence_identity: bool,
    pub reopening_requires_new_preregistration: bool,
    pub decision: MomentumFrozenMambaClosureDecisionV4,
    pub closure_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRawFeatureSplitV4 {
    pub split_version: String,
    pub source_snapshot_digest: String,
    pub v3_split_digest: String,
    pub v3_route_decision_digest: String,
    pub training_range: IndexRangeV0,
    pub purge_range: IndexRangeV0,
    pub fresh_validation_range: IndexRangeV0,
    pub final_untouched_range: IndexRangeV0,
    pub minimum_validation_samples: usize,
    pub minimum_final_reserve_samples: usize,
    pub prior_qualification_overlap_count: usize,
    pub prospective_overlap_count: usize,
    pub historical_test_overlap_count: usize,
    pub future_evaluation_overlap_count: usize,
    pub split_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumValidationYieldAuditV4 {
    pub audit_version: String,
    pub source_snapshot_digest: String,
    pub label_policy_digest: String,
    pub validation_index_range: IndexRangeV0,
    pub validation_index_count: usize,
    pub minimum_required_valid_samples: usize,
    pub valid_labelled_sample_count: usize,
    pub neutral_excluded_count: usize,
    pub horizon_unavailable_count: usize,
    pub feature_unavailable_count: usize,
    pub substantive_qualification_possible: bool,
    pub audit_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MomentumRawFeatureModelKindV4 {
    RawFeatureLogistic,
    RawFeatureInteractionLogistic,
    TrainingPrevalenceConstant,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRawFeatureParticipantConfigV4 {
    pub participant_id: String,
    pub model_kind: MomentumRawFeatureModelKindV4,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub input_feature_schema_digest: String,
    pub learning_rate_bits: u32,
    pub l2_regularization_bits: u32,
    pub maximum_epochs: usize,
    pub initialization_seed: u64,
    pub fresh_initialization: bool,
    pub training_only_normalizer: bool,
    pub config_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRawFeatureRegistrationV4 {
    pub registration_version: String,
    pub agent_id: String,
    pub source_snapshot_digest: String,
    pub canonical_intent_digest: String,
    pub canonical_view_digest: String,
    pub frozen_mamba_closure_digest: String,
    pub split_digest: String,
    pub participants: Vec<MomentumRawFeatureParticipantConfigV4>,
    pub maximum_learned_participants: usize,
    pub interaction_contribution_policy_digest: String,
    pub fresh_validation_hidden: bool,
    pub final_reserve_forbidden: bool,
    pub historical_test_forbidden: bool,
    pub future_evaluation_forbidden: bool,
    pub winner_selection_forbidden: bool,
    pub active_promotion_forbidden: bool,
    pub reward_application_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumRawFeatureRoleV4 {
    LearnedRawLogistic,
    LearnedInteractionLogistic,
    ConstantBenchmark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumRawFeatureQualificationStatusV4 {
    QualifiedLearned,
    QualifiedLinearEquivalent,
    BenchmarkQualified,
    RejectedInsufficientValidation,
    RejectedProbabilityCollapse,
    RejectedNumericalFailure,
    RejectedFeatureIntegrity,
    RejectedPolicyInvariant,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenCandidateParticipantV4 {
    pub participant_version: String,
    pub participant_id: String,
    pub participant_role: MomentumRawFeatureRoleV4,
    pub model_kind: MomentumRawFeatureModelKindV4,
    pub config_digest: String,
    pub source_snapshot_digest: String,
    pub training_range_digest: String,
    pub fresh_validation_range_digest: String,
    pub validation_timestamp_digest: String,
    pub input_feature_schema_digest: String,
    pub model_artifact_digest: String,
    pub parameter_digest: String,
    pub normalizer_digest: String,
    pub training_identity_digest: String,
    pub fresh_initialization: bool,
    pub prior_parameters_reused: bool,
    pub prior_normalizer_reused: bool,
    pub prior_predictions_reused: bool,
    pub validation_parameter_updates: usize,
    pub deployment_status: ModelAgentDeploymentStatus,
    pub participant_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionContributionStatusV4 {
    MaterialInteractionContribution,
    DetectableButBelowPolicy,
    LinearEquivalent,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumInteractionContributionAuditV4 {
    pub participant_digest: String,
    pub original_feature_parameter_digest: String,
    pub squared_feature_parameter_digest: String,
    pub pairwise_feature_parameter_digest: String,
    pub original_block_nonzero: bool,
    pub nonlinear_blocks_nonzero: bool,
    pub full_prediction_digest: String,
    pub nonlinear_ablated_prediction_digest: String,
    pub contribution_policy_digest: String,
    pub contribution_status: InteractionContributionStatusV4,
    pub audit_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRawFeatureQualificationReceiptV4 {
    pub receipt_version: String,
    pub participant_id: String,
    pub participant_role: MomentumRawFeatureRoleV4,
    pub participant_digest: String,
    pub fresh_validation_range_digest: String,
    pub qualification_policy_digest: String,
    pub private_metric_digest: String,
    pub interaction_contribution_audit_digest: Option<String>,
    pub status: MomentumRawFeatureQualificationStatusV4,
    pub validation_parameter_updates: usize,
    pub final_reserve_reads: usize,
    pub historical_test_reads: usize,
    pub future_evaluation_reads: usize,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRawFeatureFamilyV4 {
    pub family_version: String,
    pub agent_id: String,
    pub source_snapshot_digest: String,
    pub canonical_view_digest: String,
    pub frozen_mamba_closure_digest: String,
    pub split_digest: String,
    pub registration_digest: String,
    pub participants: Vec<FrozenCandidateParticipantV4>,
    pub qualification_receipts: Vec<MomentumRawFeatureQualificationReceiptV4>,
    pub interaction_contribution_audit: Option<MomentumInteractionContributionAuditV4>,
    pub qualified_learned_count: usize,
    pub qualified_benchmark_count: usize,
    pub winner_selected: bool,
    pub final_reserve_accessed: bool,
    pub eligible_for_active_committee: bool,
    pub eligible_for_promotion: bool,
    pub eligible_for_reward: bool,
    pub family_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumRawFeaturePathDecisionV4 {
    RawFeatureLearnedPathViable,
    OnlyLinearRawPathViable,
    NoQualifiedRawFeatureLearner,
    InsufficientFreshValidation,
    TechnicalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRawFeaturePathDecisionArtifactV4 {
    pub decision_version: String,
    pub family_digest: String,
    pub qualified_raw_logistic: bool,
    pub qualified_material_interaction: bool,
    pub decision: MomentumRawFeaturePathDecisionV4,
    pub decision_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumRawFeatureRosterStatusV4 {
    Ready,
    QualificationEvidenceInsufficient,
    NoQualifiedLearnedParticipant,
    BenchmarkUnavailable,
    SemanticDuplicateOnly,
    IntegrityFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRawFeatureFutureRosterV4 {
    pub roster_version: String,
    pub family_digest: String,
    pub learned_participant_digests: Vec<String>,
    pub benchmark_participant_digests: Vec<String>,
    pub excluded_semantic_duplicate_digests: Vec<String>,
    pub rejected_participant_digests: Vec<String>,
    pub inclusion_policy_digest: String,
    pub status: MomentumRawFeatureRosterStatusV4,
    pub roster_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumRawFeatureEvaluationStatusV4 {
    Registered,
    QualificationEvidenceInsufficient,
    NoQualifiedLearnedParticipant,
    BenchmarkUnavailable,
    SemanticDuplicateOnly,
    SafetyContractInvalid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRawFeatureEvaluationRegistrationV4 {
    pub registration_version: String,
    pub agent_id: String,
    pub family_digest: String,
    pub roster_digest: String,
    pub frozen_mamba_closure_digest: String,
    pub split_digest: String,
    pub raw_feature_registration_digest: String,
    pub qualification_receipt_digests: Vec<String>,
    pub interaction_contribution_audit_digest: Option<String>,
    pub source_snapshot_digest: String,
    pub source_boundary_timestamp_ms: u64,
    pub protected_registration_digests: Vec<String>,
    pub protected_timestamp_ms: Vec<u64>,
    pub provider_finality_boundary_ms: u64,
    pub prior_validation_identity_digests: Vec<String>,
    pub v4_final_untouched_reserve_digest: String,
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
    pub registration_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumRawFeatureExecutionStatusV4 {
    Planned,
    Executed,
    AlreadyExecuted,
    InsufficientFreshValidation,
    TechnicalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRawFeatureJournalV4 {
    pub journal_version: String,
    pub agent_id: String,
    pub closure_digest: String,
    pub split_digest: String,
    pub registration_digest: String,
    pub family_digest: Option<String>,
    pub decision_digest: Option<String>,
    pub roster_digest: Option<String>,
    pub evaluation_registration_digest: Option<String>,
    pub preregistration_reopened_before_validation: bool,
    pub final_reserve_accessed: bool,
    pub prior_parameters_reused: bool,
    pub active_registry_mutated: bool,
    pub legacy_trainer_capability: String,
    pub raw_feature_trainer_capability: String,
    pub status: MomentumRawFeatureExecutionStatusV4,
    pub journal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRawFeatureSafetyCountersV4 {
    pub network_requests: usize,
    pub transport_constructions: usize,
    pub credential_reads: usize,
    pub prospective_row_reads: usize,
    pub prospective_label_openings: usize,
    pub historical_test_reads: usize,
    pub future_evaluation_reads: usize,
    pub final_reserve_row_reads: usize,
    pub final_reserve_label_reads: usize,
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
pub struct MomentumRawFeatureReportV4 {
    pub report_version: String,
    pub mode: AgentPrivateLearningRunModeV0,
    pub status: MomentumRawFeatureExecutionStatusV4,
    pub closure: Option<MomentumFrozenMambaPathClosureV4>,
    pub split: Option<MomentumRawFeatureSplitV4>,
    pub registration: Option<MomentumRawFeatureRegistrationV4>,
    pub validation_yield_audit: Option<MomentumValidationYieldAuditV4>,
    pub family: Option<MomentumRawFeatureFamilyV4>,
    pub decision: Option<MomentumRawFeaturePathDecisionArtifactV4>,
    pub roster: Option<MomentumRawFeatureFutureRosterV4>,
    pub roster_status: MomentumRawFeatureRosterStatusV4,
    pub evaluation_registration: Option<MomentumRawFeatureEvaluationRegistrationV4>,
    pub evaluation_registration_status: MomentumRawFeatureEvaluationStatusV4,
    pub journal: Option<MomentumRawFeatureJournalV4>,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub storage_failure_count: usize,
    pub protected_artifacts_unchanged: bool,
    pub active_state_unchanged: bool,
    pub safety_counters: MomentumRawFeatureSafetyCountersV4,
    pub report_digest: String,
}

#[derive(Clone, Debug)]
pub(crate) struct FrozenHistoryV4 {
    pub(crate) v1: V1FrozenStateV2,
    pub(crate) v2_split: MomentumMambaRepairSplitV2,
    pub(crate) v2_family: MomentumCandidateFamilyV2,
    pub(crate) v3_split: MomentumRepresentationSplitV3,
    pub(crate) v3_family: MomentumRepresentationFamilyV3,
    pub(crate) v3_decision: MomentumRepresentationRouteDecisionArtifactV3,
}

#[derive(Clone, Debug)]
struct ExperimentV4 {
    validation_yield_audit: MomentumValidationYieldAuditV4,
    family: MomentumRawFeatureFamilyV4,
    decision: MomentumRawFeaturePathDecisionArtifactV4,
    roster: Option<MomentumRawFeatureFutureRosterV4>,
    roster_status: MomentumRawFeatureRosterStatusV4,
    evaluation: Option<MomentumRawFeatureEvaluationRegistrationV4>,
    evaluation_status: MomentumRawFeatureEvaluationStatusV4,
}

#[derive(Clone, Debug)]
pub(crate) struct MomentumFrozenReplayV4 {
    pub(crate) history: FrozenHistoryV4,
    pub(crate) closure: MomentumFrozenMambaPathClosureV4,
    pub(crate) split: MomentumRawFeatureSplitV4,
    pub(crate) registration: MomentumRawFeatureRegistrationV4,
    pub(crate) validation_yield_audit: MomentumValidationYieldAuditV4,
    pub(crate) family: MomentumRawFeatureFamilyV4,
    pub(crate) decision: MomentumRawFeaturePathDecisionArtifactV4,
    pub(crate) feature_normalizer: FeatureNormalizerV0,
    pub(crate) raw_normalizer: RepresentationNormalizerV0,
    pub(crate) interaction_normalizer: RepresentationNormalizerV0,
    pub(crate) raw_head: LogisticPredictionHeadV0,
    pub(crate) interaction_head: LogisticPredictionHeadV0,
    pub(crate) training_prevalence: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct MomentumAccumulatedParticipantEvaluationV4 {
    pub(crate) participant_digest: String,
    pub(crate) original_receipt_digest: String,
    pub(crate) status: MomentumRawFeatureQualificationStatusV4,
    pub(crate) private_metric_digest: String,
}

#[derive(Clone, Debug)]
pub(crate) struct MomentumAccumulatedReplayEvaluationV4 {
    pub(crate) original_valid_sample_count: usize,
    pub(crate) supplemental_valid_sample_count: usize,
    pub(crate) original_neutral_excluded_count: usize,
    pub(crate) supplemental_neutral_excluded_count: usize,
    pub(crate) accumulated_validation_identity_digest: String,
    pub(crate) source_boundary_timestamp_ms: u64,
    pub(crate) participant_evaluations: Vec<MomentumAccumulatedParticipantEvaluationV4>,
    pub(crate) interaction_contribution: MomentumInteractionContributionAuditV4,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MomentumFrozenParticipantPredictionV4 {
    pub(crate) participant_digest: String,
    pub(crate) config_digest: String,
    pub(crate) parameter_digest: String,
    pub(crate) normalizer_digest: String,
    pub(crate) model_artifact_digest: String,
    pub(crate) feature_schema_digest: String,
    pub(crate) training_identity_digest: String,
    pub(crate) probability_bits: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MomentumFrozenPredictionV4 {
    pub(crate) feature_identity_digest: String,
    pub(crate) participant_predictions: Vec<MomentumFrozenParticipantPredictionV4>,
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn range_digest_v4(label: &str, range: &IndexRangeV0) -> String {
    stable_hash_string(&format!(
        "momentum-raw-feature-range-v4:{label}:{}:{}",
        range.start, range.end
    ))
}

fn overlap_count(left: &IndexRangeV0, right: &IndexRangeV0) -> usize {
    left.end
        .min(right.end)
        .saturating_sub(left.start.max(right.start))
}

fn interaction_policy_digest_v4() -> String {
    stable_hash_string(&format!(
        "interaction-contribution-policy-v4:{}:{}:zero-nonlinear-parameter-block",
        MATERIAL_INTERACTION_EFFECT_BITS_V4, DETECTABLE_INTERACTION_EFFECT_BITS_V4
    ))
}

fn qualification_policy_digest_v4() -> String {
    stable_hash_string(
        "momentum-raw-feature-qualification-v4:finite:minimum-24:no-collapse:feature-integrity:zero-validation-updates",
    )
}

fn closure_digest_v4(value: &MomentumFrozenMambaPathClosureV4) -> String {
    let mut canonical = value.clone();
    canonical.closure_digest.clear();
    stable_hash_string(&format!("{canonical:?}"))
}

fn split_digest_v4(value: &MomentumRawFeatureSplitV4) -> String {
    let mut canonical = value.clone();
    canonical.split_digest.clear();
    stable_hash_string(&format!("{canonical:?}"))
}

fn validation_yield_audit_digest_v4(value: &MomentumValidationYieldAuditV4) -> String {
    let mut canonical = value.clone();
    canonical.audit_digest.clear();
    stable_hash_string(&format!("{canonical:?}"))
}

fn config_digest_v4(value: &MomentumRawFeatureParticipantConfigV4) -> String {
    stable_hash_string(&format!(
        "{:?}",
        (
            &value.participant_id,
            value.model_kind,
            &value.feature_policy_digest,
            &value.label_policy_digest,
            &value.input_feature_schema_digest,
            value.learning_rate_bits,
            value.l2_regularization_bits,
            value.maximum_epochs,
            value.initialization_seed,
            value.fresh_initialization,
            value.training_only_normalizer,
        )
    ))
}

fn registration_digest_v4(value: &MomentumRawFeatureRegistrationV4) -> String {
    let mut canonical = value.clone();
    canonical.registration_digest.clear();
    stable_hash_string(&format!("{canonical:?}"))
}

fn participant_digest_v4(value: &FrozenCandidateParticipantV4) -> String {
    let mut canonical = value.clone();
    canonical.participant_digest.clear();
    stable_hash_string(&format!("{canonical:?}"))
}

fn contribution_digest_v4(value: &MomentumInteractionContributionAuditV4) -> String {
    stable_hash_string(&format!(
        "{:?}",
        (
            &value.participant_digest,
            &value.original_feature_parameter_digest,
            &value.squared_feature_parameter_digest,
            &value.pairwise_feature_parameter_digest,
            value.original_block_nonzero,
            value.nonlinear_blocks_nonzero,
            &value.full_prediction_digest,
            &value.nonlinear_ablated_prediction_digest,
            &value.contribution_policy_digest,
            value.contribution_status,
        )
    ))
}

fn receipt_digest_v4(value: &MomentumRawFeatureQualificationReceiptV4) -> String {
    let mut canonical = value.clone();
    canonical.receipt_digest.clear();
    stable_hash_string(&format!("{canonical:?}"))
}

fn family_digest_v4(value: &MomentumRawFeatureFamilyV4) -> String {
    let mut canonical = value.clone();
    canonical.family_digest.clear();
    stable_hash_string(&format!("{canonical:?}"))
}

fn decision_digest_v4(value: &MomentumRawFeaturePathDecisionArtifactV4) -> String {
    stable_hash_string(&format!(
        "{:?}",
        (
            &value.decision_version,
            &value.family_digest,
            value.qualified_raw_logistic,
            value.qualified_material_interaction,
            value.decision,
        )
    ))
}

fn roster_digest_v4(value: &MomentumRawFeatureFutureRosterV4) -> String {
    stable_hash_string(&format!(
        "{:?}",
        (
            &value.roster_version,
            &value.family_digest,
            &value.learned_participant_digests,
            &value.benchmark_participant_digests,
            &value.excluded_semantic_duplicate_digests,
            &value.rejected_participant_digests,
            &value.inclusion_policy_digest,
            value.status,
        )
    ))
}

fn evaluation_digest_v4(value: &MomentumRawFeatureEvaluationRegistrationV4) -> String {
    let mut canonical = value.clone();
    canonical.registration_digest.clear();
    stable_hash_string(&format!("{canonical:?}"))
}

fn journal_digest_v4(value: &MomentumRawFeatureJournalV4) -> String {
    let mut canonical = value.clone();
    canonical.journal_digest.clear();
    stable_hash_string(&format!("{canonical:?}"))
}

fn zero_safety_counters_v4() -> MomentumRawFeatureSafetyCountersV4 {
    MomentumRawFeatureSafetyCountersV4 {
        network_requests: 0,
        transport_constructions: 0,
        credential_reads: 0,
        prospective_row_reads: 0,
        prospective_label_openings: 0,
        historical_test_reads: 0,
        future_evaluation_reads: 0,
        final_reserve_row_reads: 0,
        final_reserve_label_reads: 0,
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

fn validate_closure_v4(value: &MomentumFrozenMambaPathClosureV4) -> Result<(), String> {
    if value.closure_version != CLOSURE_VERSION_V4
        || value.agent_id != AGENT_ID_V4
        || value.source_snapshot_digest.is_empty()
        || value.canonical_intent_digest.is_empty()
        || value.canonical_view_digest.is_empty()
        || value.v1_family_digest.is_empty()
        || value.v2_family_digest.is_empty()
        || value.v3_family_digest.is_empty()
        || value.v3_route_decision_digest.is_empty()
        || value.frozen_encoder_digest.is_empty()
        || value.feature_policy_digest.is_empty()
        || value.label_policy_digest.is_empty()
        || value.genuine_mamba_qualified_count != 0
        || !value.head_only_repair_forbidden
        || !value.frozen_representation_sweep_forbidden
        || !value.frozen_mamba_parent_use_forbidden
        || !value.reopening_requires_new_encoder_identity
        || !value.reopening_requires_new_evidence_identity
        || !value.reopening_requires_new_preregistration
        || value.decision != MomentumFrozenMambaClosureDecisionV4::ClosedForCurrentEvidenceAndPolicy
        || value.closure_digest != closure_digest_v4(value)
    {
        return Err("V4 frozen-Mamba closure rejected".to_string());
    }
    Ok(())
}

fn validate_split_v4(value: &MomentumRawFeatureSplitV4) -> Result<(), String> {
    let config = MomentumLearningCampaignConfigV0::default();
    let minimum = config.validation_rows;
    let minimum_purge = config
        .feature_config
        .minimum_history()
        .map_err(|_| "V4 purge policy unavailable".to_string())?
        .checked_sub(1)
        .and_then(|count| count.checked_add(config.sequence_config.sequence_length - 1))
        .and_then(|count| count.checked_add(config.sequence_config.prediction_horizon))
        .ok_or_else(|| "V4 purge policy overflow".to_string())?;
    if value.split_version != SPLIT_VERSION_V4
        || value.source_snapshot_digest.is_empty()
        || value.v3_split_digest.is_empty()
        || value.v3_route_decision_digest.is_empty()
        || value.training_range.start != 0
        || value.training_range.end != value.purge_range.start
        || value.purge_range.end != value.fresh_validation_range.start
        || value.fresh_validation_range.end != value.final_untouched_range.start
        || value.minimum_validation_samples != minimum
        || value.minimum_final_reserve_samples != minimum
        || value.purge_range.end - value.purge_range.start < minimum_purge
        || value.fresh_validation_range.end - value.fresh_validation_range.start != minimum
        || value.final_untouched_range.end - value.final_untouched_range.start != minimum
        || value.prior_qualification_overlap_count != 0
        || value.prospective_overlap_count != 0
        || value.historical_test_overlap_count != 0
        || value.future_evaluation_overlap_count != 0
        || value.split_digest != split_digest_v4(value)
    {
        return Err("V4 raw-feature split rejected".to_string());
    }
    Ok(())
}

fn validate_validation_yield_audit_v4(
    value: &MomentumValidationYieldAuditV4,
) -> Result<(), String> {
    let index_count = value
        .validation_index_range
        .end
        .checked_sub(value.validation_index_range.start)
        .ok_or_else(|| "V4 validation-yield range rejected".to_string())?;
    let classified_count = value
        .valid_labelled_sample_count
        .checked_add(value.neutral_excluded_count)
        .and_then(|count| count.checked_add(value.horizon_unavailable_count))
        .and_then(|count| count.checked_add(value.feature_unavailable_count))
        .ok_or_else(|| "V4 validation-yield count overflow".to_string())?;
    if value.audit_version != VALIDATION_YIELD_AUDIT_VERSION_V4
        || value.source_snapshot_digest.is_empty()
        || value.label_policy_digest.is_empty()
        || value.validation_index_count != index_count
        || classified_count != value.validation_index_count
        || value.minimum_required_valid_samples == 0
        || value.substantive_qualification_possible
            != (value.valid_labelled_sample_count >= value.minimum_required_valid_samples)
        || value.audit_digest != validation_yield_audit_digest_v4(value)
    {
        return Err("V4 validation-yield audit rejected".to_string());
    }
    Ok(())
}

fn expected_config_identity(kind: MomentumRawFeatureModelKindV4) -> (&'static str, bool) {
    match kind {
        MomentumRawFeatureModelKindV4::RawFeatureLogistic => ("RawFeatureLogisticV4", true),
        MomentumRawFeatureModelKindV4::RawFeatureInteractionLogistic => {
            ("RawFeatureInteractionLogisticV4", true)
        }
        MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant => {
            ("TrainingPrevalenceConstantV4", false)
        }
    }
}

fn validate_config_v4(value: &MomentumRawFeatureParticipantConfigV4) -> Result<(), String> {
    let (expected_id, learned) = expected_config_identity(value.model_kind);
    let learning_rate = f32::from_bits(value.learning_rate_bits);
    let l2 = f32::from_bits(value.l2_regularization_bits);
    if value.participant_id != expected_id
        || value.feature_policy_digest.is_empty()
        || value.label_policy_digest.is_empty()
        || value.input_feature_schema_digest.is_empty()
        || learned != value.fresh_initialization
        || learned != value.training_only_normalizer
        || (learned
            && (!learning_rate.is_finite()
                || learning_rate <= 0.0
                || !l2.is_finite()
                || l2 < 0.0
                || value.maximum_epochs == 0))
        || (!learned
            && (value.learning_rate_bits != 0
                || value.l2_regularization_bits != 0
                || value.maximum_epochs != 0
                || value.initialization_seed != 0))
        || value.config_digest != config_digest_v4(value)
    {
        return Err("V4 participant configuration rejected".to_string());
    }
    Ok(())
}

fn validate_registration_v4(value: &MomentumRawFeatureRegistrationV4) -> Result<(), String> {
    if value.registration_version != REGISTRATION_VERSION_V4
        || value.agent_id != AGENT_ID_V4
        || value.source_snapshot_digest.is_empty()
        || value.canonical_intent_digest.is_empty()
        || value.canonical_view_digest.is_empty()
        || value.frozen_mamba_closure_digest.is_empty()
        || value.split_digest.is_empty()
        || value.participants.len() != 3
        || value.maximum_learned_participants != MAXIMUM_LEARNED_PARTICIPANTS_V4
        || value.interaction_contribution_policy_digest != interaction_policy_digest_v4()
        || !value.fresh_validation_hidden
        || !value.final_reserve_forbidden
        || !value.historical_test_forbidden
        || !value.future_evaluation_forbidden
        || !value.winner_selection_forbidden
        || !value.active_promotion_forbidden
        || !value.reward_application_forbidden
        || value.registration_digest != registration_digest_v4(value)
    {
        return Err("V4 registration rejected".to_string());
    }
    let kinds = value
        .participants
        .iter()
        .map(|item| item.model_kind)
        .collect::<BTreeSet<_>>();
    let learned = value
        .participants
        .iter()
        .filter(|item| item.model_kind != MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant)
        .count();
    if kinds.len() != 3 || learned != MAXIMUM_LEARNED_PARTICIPANTS_V4 {
        return Err("V4 fixed participant set rejected".to_string());
    }
    for item in &value.participants {
        validate_config_v4(item)?;
    }
    Ok(())
}

fn validate_participant_v4(value: &FrozenCandidateParticipantV4) -> Result<(), String> {
    let role_matches = matches!(
        (value.participant_role, value.model_kind),
        (
            MomentumRawFeatureRoleV4::LearnedRawLogistic,
            MomentumRawFeatureModelKindV4::RawFeatureLogistic
        ) | (
            MomentumRawFeatureRoleV4::LearnedInteractionLogistic,
            MomentumRawFeatureModelKindV4::RawFeatureInteractionLogistic
        ) | (
            MomentumRawFeatureRoleV4::ConstantBenchmark,
            MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant
        )
    );
    let learned = value.participant_role != MomentumRawFeatureRoleV4::ConstantBenchmark;
    if value.participant_version != PARTICIPANT_VERSION_V4
        || value.participant_id != expected_config_identity(value.model_kind).0
        || !role_matches
        || value.config_digest.is_empty()
        || value.source_snapshot_digest.is_empty()
        || value.training_range_digest.is_empty()
        || value.fresh_validation_range_digest.is_empty()
        || value.validation_timestamp_digest.is_empty()
        || value.input_feature_schema_digest.is_empty()
        || value.model_artifact_digest.is_empty()
        || value.parameter_digest.is_empty()
        || value.normalizer_digest.is_empty()
        || value.training_identity_digest.is_empty()
        || learned != value.fresh_initialization
        || value.prior_parameters_reused
        || value.prior_normalizer_reused
        || value.prior_predictions_reused
        || value.validation_parameter_updates != 0
        || value.deployment_status != ModelAgentDeploymentStatus::ShadowOnly
        || value.participant_digest != participant_digest_v4(value)
    {
        return Err("V4 participant rejected".to_string());
    }
    Ok(())
}

fn validate_contribution_v4(value: &MomentumInteractionContributionAuditV4) -> Result<(), String> {
    if value.participant_digest.is_empty()
        || value.original_feature_parameter_digest.is_empty()
        || value.squared_feature_parameter_digest.is_empty()
        || value.pairwise_feature_parameter_digest.is_empty()
        || value.full_prediction_digest.is_empty()
        || value.nonlinear_ablated_prediction_digest.is_empty()
        || value.contribution_policy_digest != interaction_policy_digest_v4()
        || value.audit_digest != contribution_digest_v4(value)
    {
        return Err("V4 interaction contribution audit rejected".to_string());
    }
    let valid = match value.contribution_status {
        InteractionContributionStatusV4::MaterialInteractionContribution
        | InteractionContributionStatusV4::DetectableButBelowPolicy => {
            value.original_block_nonzero && value.nonlinear_blocks_nonzero
        }
        InteractionContributionStatusV4::LinearEquivalent => value.original_block_nonzero,
        InteractionContributionStatusV4::Invalid => {
            !value.original_block_nonzero || !value.nonlinear_blocks_nonzero
        }
    };
    if !valid {
        return Err("V4 interaction contribution classification rejected".to_string());
    }
    Ok(())
}

fn validate_receipt_v4(value: &MomentumRawFeatureQualificationReceiptV4) -> Result<(), String> {
    let interaction =
        value.participant_role == MomentumRawFeatureRoleV4::LearnedInteractionLogistic;
    if value.receipt_version != RECEIPT_VERSION_V4
        || value.participant_id.is_empty()
        || value.participant_digest.is_empty()
        || value.fresh_validation_range_digest.is_empty()
        || value.qualification_policy_digest != qualification_policy_digest_v4()
        || value.private_metric_digest.is_empty()
        || interaction != value.interaction_contribution_audit_digest.is_some()
        || value.validation_parameter_updates != 0
        || value.final_reserve_reads != 0
        || value.historical_test_reads != 0
        || value.future_evaluation_reads != 0
        || value.receipt_digest != receipt_digest_v4(value)
    {
        return Err("V4 qualification receipt rejected".to_string());
    }
    let allowed = match value.participant_role {
        MomentumRawFeatureRoleV4::LearnedRawLogistic => matches!(
            value.status,
            MomentumRawFeatureQualificationStatusV4::QualifiedLearned
                | MomentumRawFeatureQualificationStatusV4::RejectedInsufficientValidation
                | MomentumRawFeatureQualificationStatusV4::RejectedProbabilityCollapse
                | MomentumRawFeatureQualificationStatusV4::RejectedNumericalFailure
                | MomentumRawFeatureQualificationStatusV4::RejectedFeatureIntegrity
                | MomentumRawFeatureQualificationStatusV4::RejectedPolicyInvariant
        ),
        MomentumRawFeatureRoleV4::LearnedInteractionLogistic => matches!(
            value.status,
            MomentumRawFeatureQualificationStatusV4::QualifiedLearned
                | MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent
                | MomentumRawFeatureQualificationStatusV4::RejectedInsufficientValidation
                | MomentumRawFeatureQualificationStatusV4::RejectedProbabilityCollapse
                | MomentumRawFeatureQualificationStatusV4::RejectedNumericalFailure
                | MomentumRawFeatureQualificationStatusV4::RejectedFeatureIntegrity
                | MomentumRawFeatureQualificationStatusV4::RejectedPolicyInvariant
        ),
        MomentumRawFeatureRoleV4::ConstantBenchmark => matches!(
            value.status,
            MomentumRawFeatureQualificationStatusV4::BenchmarkQualified
                | MomentumRawFeatureQualificationStatusV4::RejectedInsufficientValidation
                | MomentumRawFeatureQualificationStatusV4::RejectedNumericalFailure
                | MomentumRawFeatureQualificationStatusV4::RejectedPolicyInvariant
        ),
    };
    if !allowed {
        return Err("V4 qualification role rejected".to_string());
    }
    Ok(())
}

fn validate_family_v4(value: &MomentumRawFeatureFamilyV4) -> Result<(), String> {
    if value.family_version != FAMILY_VERSION_V4
        || value.agent_id != AGENT_ID_V4
        || value.source_snapshot_digest.is_empty()
        || value.canonical_view_digest.is_empty()
        || value.frozen_mamba_closure_digest.is_empty()
        || value.split_digest.is_empty()
        || value.registration_digest.is_empty()
        || value.participants.len() != 3
        || value.qualification_receipts.len() != 3
        || value.interaction_contribution_audit.is_none()
        || value.winner_selected
        || value.final_reserve_accessed
        || value.eligible_for_active_committee
        || value.eligible_for_promotion
        || value.eligible_for_reward
        || value.family_digest != family_digest_v4(value)
    {
        return Err("V4 family rejected".to_string());
    }
    let participant_digests = value
        .participants
        .iter()
        .map(|item| item.participant_digest.as_str())
        .collect::<BTreeSet<_>>();
    let receipt_digests = value
        .qualification_receipts
        .iter()
        .map(|item| item.participant_digest.as_str())
        .collect::<BTreeSet<_>>();
    if participant_digests != receipt_digests {
        return Err("V4 family receipt coverage rejected".to_string());
    }
    let interaction = value
        .participants
        .iter()
        .find(|item| item.participant_role == MomentumRawFeatureRoleV4::LearnedInteractionLogistic)
        .ok_or_else(|| "V4 interaction participant missing".to_string())?;
    if value
        .interaction_contribution_audit
        .as_ref()
        .map(|item| item.participant_digest.as_str())
        != Some(interaction.participant_digest.as_str())
    {
        return Err("V4 interaction audit binding rejected".to_string());
    }
    let validation_identities = value
        .participants
        .iter()
        .map(|item| item.validation_timestamp_digest.as_str())
        .collect::<BTreeSet<_>>();
    if validation_identities.len() != 1 {
        return Err("V4 validation timestamps diverged".to_string());
    }
    let actual_learned = value
        .qualification_receipts
        .iter()
        .filter(|item| {
            matches!(
                item.status,
                MomentumRawFeatureQualificationStatusV4::QualifiedLearned
                    | MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent
            )
        })
        .count();
    let actual_benchmark = value
        .qualification_receipts
        .iter()
        .filter(|item| item.status == MomentumRawFeatureQualificationStatusV4::BenchmarkQualified)
        .count();
    if value.qualified_learned_count != actual_learned
        || value.qualified_benchmark_count != actual_benchmark
    {
        return Err("V4 family qualification counts rejected".to_string());
    }
    for participant in &value.participants {
        validate_participant_v4(participant)?;
    }
    for receipt in &value.qualification_receipts {
        validate_receipt_v4(receipt)?;
    }
    validate_contribution_v4(value.interaction_contribution_audit.as_ref().unwrap())?;
    Ok(())
}

fn decision_inputs_v4(
    family: &MomentumRawFeatureFamilyV4,
) -> (bool, bool, MomentumRawFeaturePathDecisionV4) {
    let status_for_role = |role| {
        family
            .participants
            .iter()
            .find(|item| item.participant_role == role)
            .and_then(|participant| {
                family
                    .qualification_receipts
                    .iter()
                    .find(|receipt| receipt.participant_digest == participant.participant_digest)
            })
            .map(|receipt| receipt.status)
    };
    let qualified_raw = status_for_role(MomentumRawFeatureRoleV4::LearnedRawLogistic)
        == Some(MomentumRawFeatureQualificationStatusV4::QualifiedLearned);
    let qualified_interaction =
        status_for_role(MomentumRawFeatureRoleV4::LearnedInteractionLogistic)
            == Some(MomentumRawFeatureQualificationStatusV4::QualifiedLearned)
            && family
                .interaction_contribution_audit
                .as_ref()
                .is_some_and(|audit| {
                    audit.contribution_status
                        == InteractionContributionStatusV4::MaterialInteractionContribution
                });
    let all_insufficient = family.qualification_receipts.iter().all(|receipt| {
        receipt.status == MomentumRawFeatureQualificationStatusV4::RejectedInsufficientValidation
    });
    let decision = if qualified_interaction {
        MomentumRawFeaturePathDecisionV4::RawFeatureLearnedPathViable
    } else if qualified_raw {
        MomentumRawFeaturePathDecisionV4::OnlyLinearRawPathViable
    } else if all_insufficient {
        MomentumRawFeaturePathDecisionV4::InsufficientFreshValidation
    } else {
        MomentumRawFeaturePathDecisionV4::NoQualifiedRawFeatureLearner
    };
    (qualified_raw, qualified_interaction, decision)
}

fn derive_decision_v4(
    family: &MomentumRawFeatureFamilyV4,
) -> MomentumRawFeaturePathDecisionArtifactV4 {
    let (qualified_raw_logistic, qualified_material_interaction, decision) =
        decision_inputs_v4(family);
    let mut value = MomentumRawFeaturePathDecisionArtifactV4 {
        decision_version: DECISION_VERSION_V4.to_string(),
        family_digest: family.family_digest.clone(),
        qualified_raw_logistic,
        qualified_material_interaction,
        decision,
        decision_digest: String::new(),
    };
    value.decision_digest = decision_digest_v4(&value);
    value
}

fn validate_decision_v4(
    value: &MomentumRawFeaturePathDecisionArtifactV4,
    family: &MomentumRawFeatureFamilyV4,
) -> Result<(), String> {
    let (qualified_raw, qualified_interaction, expected) = decision_inputs_v4(family);
    if value.decision_version != DECISION_VERSION_V4
        || value.family_digest != family.family_digest
        || value.qualified_raw_logistic != qualified_raw
        || value.qualified_material_interaction != qualified_interaction
        || value.decision != expected
        || value.decision_digest != decision_digest_v4(value)
    {
        return Err("V4 path decision rejected".to_string());
    }
    Ok(())
}

fn legacy_insufficient_decision_v4(
    family: &MomentumRawFeatureFamilyV4,
) -> Option<MomentumRawFeaturePathDecisionArtifactV4> {
    family
        .qualification_receipts
        .iter()
        .all(|receipt| {
            receipt.status
                == MomentumRawFeatureQualificationStatusV4::RejectedInsufficientValidation
        })
        .then(|| {
            let mut value = MomentumRawFeaturePathDecisionArtifactV4 {
                decision_version: DECISION_VERSION_V4.to_string(),
                family_digest: family.family_digest.clone(),
                qualified_raw_logistic: false,
                qualified_material_interaction: false,
                decision: MomentumRawFeaturePathDecisionV4::NoQualifiedRawFeatureLearner,
                decision_digest: String::new(),
            };
            value.decision_digest = decision_digest_v4(&value);
            value
        })
}

fn validate_roster_v4(
    value: &MomentumRawFeatureFutureRosterV4,
    family: &MomentumRawFeatureFamilyV4,
) -> Result<(), String> {
    if value.roster_version != ROSTER_VERSION_V4
        || value.family_digest != family.family_digest
        || value.learned_participant_digests.is_empty()
        || value.benchmark_participant_digests.is_empty()
        || value.status != MomentumRawFeatureRosterStatusV4::Ready
        || value.roster_digest != roster_digest_v4(value)
    {
        return Err("V4 future roster rejected".to_string());
    }
    let status_for = |digest: &str| {
        family
            .qualification_receipts
            .iter()
            .find(|item| item.participant_digest == digest)
            .map(|item| item.status)
    };
    let expected_learned = sorted_unique(
        family
            .participants
            .iter()
            .filter(|item| {
                item.participant_role != MomentumRawFeatureRoleV4::ConstantBenchmark
                    && status_for(&item.participant_digest)
                        == Some(MomentumRawFeatureQualificationStatusV4::QualifiedLearned)
            })
            .map(|item| item.participant_digest.clone())
            .collect(),
    );
    let expected_benchmark = sorted_unique(
        family
            .participants
            .iter()
            .filter(|item| {
                item.participant_role == MomentumRawFeatureRoleV4::ConstantBenchmark
                    && status_for(&item.participant_digest)
                        == Some(MomentumRawFeatureQualificationStatusV4::BenchmarkQualified)
            })
            .map(|item| item.participant_digest.clone())
            .collect(),
    );
    let expected_duplicates = sorted_unique(
        family
            .participants
            .iter()
            .filter(|item| {
                status_for(&item.participant_digest)
                    == Some(MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent)
            })
            .map(|item| item.participant_digest.clone())
            .collect(),
    );
    let included = expected_learned
        .iter()
        .chain(&expected_benchmark)
        .chain(&expected_duplicates)
        .collect::<BTreeSet<_>>();
    let expected_rejected = sorted_unique(
        family
            .participants
            .iter()
            .filter(|item| !included.contains(&item.participant_digest))
            .map(|item| item.participant_digest.clone())
            .collect(),
    );
    if value.learned_participant_digests != expected_learned
        || value.benchmark_participant_digests != expected_benchmark
        || value.excluded_semantic_duplicate_digests != expected_duplicates
        || value.rejected_participant_digests != expected_rejected
    {
        return Err("V4 roster participant sets rejected".to_string());
    }
    Ok(())
}

fn validate_evaluation_v4(
    value: &MomentumRawFeatureEvaluationRegistrationV4,
    family: &MomentumRawFeatureFamilyV4,
    roster: &MomentumRawFeatureFutureRosterV4,
) -> Result<(), String> {
    if value.registration_version != EVALUATION_VERSION_V4
        || value.agent_id != AGENT_ID_V4
        || value.family_digest != family.family_digest
        || value.roster_digest != roster.roster_digest
        || value.frozen_mamba_closure_digest != family.frozen_mamba_closure_digest
        || value.split_digest != family.split_digest
        || value.raw_feature_registration_digest != family.registration_digest
        || value.qualification_receipt_digests.is_empty()
        || value.source_snapshot_digest != family.source_snapshot_digest
        || value.source_boundary_timestamp_ms == 0
        || value.protected_registration_digests.is_empty()
        || value.protected_timestamp_ms.len() != 4
        || value
            .protected_timestamp_ms
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        || value.provider_finality_boundary_ms == 0
        || value.prior_validation_identity_digests.len() != 3
        || value.v4_final_untouched_reserve_digest.is_empty()
        || value.minimum_accepted_timestamp_ms <= value.source_boundary_timestamp_ms
        || value
            .protected_timestamp_ms
            .last()
            .is_none_or(|last| value.minimum_accepted_timestamp_ms <= *last)
        || value.minimum_accepted_timestamp_ms < value.provider_finality_boundary_ms
        || !value.labels_hidden_until_opening
        || !value.probabilities_hidden_until_opening
        || !value.one_time_opening_required
        || !value.winner_selection_forbidden_before_opening
        || !value.active_promotion_forbidden
        || !value.reward_application_forbidden
        || value.maximum_requests != 1
        || value.maximum_concurrency != 1
        || value.maximum_retries != 0
        || value.registration_digest != evaluation_digest_v4(value)
    {
        return Err("V4 evaluation registration rejected".to_string());
    }
    let included = roster
        .learned_participant_digests
        .iter()
        .chain(&roster.benchmark_participant_digests)
        .collect::<BTreeSet<_>>();
    let expected_receipts = sorted_unique(
        family
            .qualification_receipts
            .iter()
            .filter(|item| included.contains(&item.participant_digest))
            .map(|item| item.receipt_digest.clone())
            .collect(),
    );
    if value.qualification_receipt_digests != expected_receipts {
        return Err("V4 evaluation receipt binding rejected".to_string());
    }
    Ok(())
}

fn validate_journal_v4(value: &MomentumRawFeatureJournalV4) -> Result<(), String> {
    if value.journal_version != JOURNAL_VERSION_V4
        || value.agent_id != AGENT_ID_V4
        || value.closure_digest.is_empty()
        || value.split_digest.is_empty()
        || value.registration_digest.is_empty()
        || !value.preregistration_reopened_before_validation
        || value.final_reserve_accessed
        || value.prior_parameters_reused
        || value.active_registry_mutated
        || value.legacy_trainer_capability
            != "MomentumFrozenMambaLegacy/terminal-current-evidence-policy"
        || value.raw_feature_trainer_capability != "MomentumRawFeatureV4/ShadowOnly"
        || value.status != MomentumRawFeatureExecutionStatusV4::Executed
        || value.journal_digest != journal_digest_v4(value)
    {
        return Err("V4 journal rejected".to_string());
    }
    Ok(())
}

fn protobuf_paths(directory: &Path) -> Result<Vec<PathBuf>, String> {
    if !directory.is_dir() {
        return Err("V4 artifact directory unavailable".to_string());
    }
    let mut paths = fs::read_dir(directory)
        .map_err(|_| "V4 artifact directory read failed".to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|value| value == "pb"))
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn read_single<T>(
    directory: &Path,
    decode: impl Fn(&[u8]) -> Result<T, String>,
) -> Result<T, String> {
    let paths = protobuf_paths(directory)?;
    if paths.len() != 1 {
        return Err("V4 single artifact identity rejected".to_string());
    }
    decode(&fs::read(&paths[0]).map_err(|_| "V4 artifact read failed".to_string())?)
}

fn read_corrected_decision_v4(
    directory: &Path,
    family: &MomentumRawFeatureFamilyV4,
) -> Result<MomentumRawFeaturePathDecisionArtifactV4, String> {
    let legacy = legacy_insufficient_decision_v4(family);
    let mut corrected = Vec::new();
    for path in protobuf_paths(directory)? {
        let bytes = fs::read(path).map_err(|_| "V4 artifact read failed".to_string())?;
        let value = decision_from_pb_unvalidated(
            DecisionProtobufV4::decode(bytes.as_slice())
                .map_err(|_| "V4 decision Protobuf rejected".to_string())?,
        )?;
        if validate_decision_v4(&value, family).is_ok() {
            corrected.push(value);
        } else if legacy.as_ref() != Some(&value) {
            return Err("V4 unexpected decision artifact rejected".to_string());
        }
    }
    if corrected.len() != 1 {
        return Err("V4 corrected decision identity rejected".to_string());
    }
    Ok(corrected.remove(0))
}

fn load_frozen_history_v4(
    root: &Path,
    snapshots: &[DataSnapshot],
) -> Result<FrozenHistoryV4, String> {
    let v1 = load_v1_frozen_state_v2(root, snapshots)?;
    let v2_root = root.join("v2").join(AGENT_ID_V4);
    let v2_split = read_single(
        &v2_root.join("repair_splits"),
        decode_momentum_mamba_repair_split_protobuf_v2,
    )?;
    let v2_family = read_single(
        &v2_root.join("families"),
        decode_momentum_candidate_family_protobuf_v2,
    )?;
    let v3_root = root.join("v3").join(AGENT_ID_V4);
    let v3_split = read_single(
        &v3_root.join("representation_splits"),
        decode_momentum_representation_split_protobuf_v3,
    )?;
    let v3_family = read_single(
        &v3_root.join("families"),
        decode_momentum_representation_family_protobuf_v3,
    )?;
    let v3_decision = read_single(&v3_root.join("route_decisions"), |bytes| {
        decode_momentum_representation_decision_protobuf_v3(bytes, &v3_family)
    })?;
    let learned_v3 = v3_family
        .participants
        .iter()
        .filter(|item| {
            matches!(
                item.participant_role,
                MomentumRepresentationParticipantRoleV3::MambaOnly
                    | MomentumRepresentationParticipantRoleV3::MambaResidualHybrid
            )
        })
        .collect::<Vec<_>>();
    let all_routes_rejected = learned_v3.iter().all(|participant| {
        v3_family.qualification_receipts.iter().any(|receipt| {
            receipt.participant_digest == participant.participant_digest
                && receipt.status
                    == MomentumRepresentationQualificationStatusV3::RejectedRepresentationInvariant
        })
    });
    if v2_family.qualified_learned_participant_count != 0
        || v3_family.qualified_mamba_only_count + v3_family.qualified_mamba_hybrid_count != 0
        || learned_v3.len() != 4
        || !all_routes_rejected
        || v3_decision.decision
            != MomentumRepresentationRouteDecisionV3::AllRepresentationRoutesCollapsed
        || !v3_decision.further_head_only_repair_forbidden
        || !v3_decision.further_frozen_representation_sweep_forbidden
        || v3_root.join("rosters").exists()
        || v3_root.join("evaluation_registrations").exists()
    {
        return Err("V4 immutable V1-V3 history rejected".to_string());
    }
    Ok(FrozenHistoryV4 {
        v1,
        v2_split,
        v2_family,
        v3_split,
        v3_family,
        v3_decision,
    })
}

fn derive_closure_v4(
    history: &FrozenHistoryV4,
) -> Result<MomentumFrozenMambaPathClosureV4, String> {
    let encoders = history
        .v3_family
        .participants
        .iter()
        .filter_map(|item| item.encoder_digest.clone())
        .collect::<BTreeSet<_>>();
    if encoders.len() != 1 {
        return Err("V4 frozen encoder identity rejected".to_string());
    }
    let mut closure = MomentumFrozenMambaPathClosureV4 {
        closure_version: CLOSURE_VERSION_V4.to_string(),
        agent_id: AGENT_ID_V4.to_string(),
        source_snapshot_digest: history.v1.snapshot.content_digest.clone(),
        canonical_intent_digest: history.v1.input.input.intent.intent_digest.clone(),
        canonical_view_digest: history.v1.input.input.view.view_digest.clone(),
        v1_family_digest: history.v1.family.family_digest.clone(),
        v2_family_digest: history.v2_family.family_digest.clone(),
        v3_family_digest: history.v3_family.family_digest.clone(),
        v3_route_decision_digest: history.v3_decision.decision_digest.clone(),
        frozen_encoder_digest: encoders.into_iter().next().unwrap(),
        feature_policy_digest: history.v1.session.feature_policy_digest.clone(),
        label_policy_digest: history.v1.session.label_policy_digest.clone(),
        genuine_mamba_qualified_count: history.v3_family.qualified_mamba_only_count
            + history.v3_family.qualified_mamba_hybrid_count,
        head_only_repair_forbidden: history.v3_decision.further_head_only_repair_forbidden,
        frozen_representation_sweep_forbidden: history
            .v3_decision
            .further_frozen_representation_sweep_forbidden,
        frozen_mamba_parent_use_forbidden: true,
        reopening_requires_new_encoder_identity: true,
        reopening_requires_new_evidence_identity: true,
        reopening_requires_new_preregistration: true,
        decision: MomentumFrozenMambaClosureDecisionV4::ClosedForCurrentEvidenceAndPolicy,
        closure_digest: String::new(),
    };
    closure.closure_digest = closure_digest_v4(&closure);
    validate_closure_v4(&closure)?;
    Ok(closure)
}

fn derive_split_v4(
    history: &FrozenHistoryV4,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<MomentumRawFeatureSplitV4, String> {
    let minimum = MomentumLearningCampaignConfigV0::default().validation_rows;
    let reserve = &history.v3_split.final_reserved_range;
    let fresh_end = reserve
        .start
        .checked_add(minimum)
        .ok_or_else(|| "V4 validation split overflow".to_string())?;
    let final_end = fresh_end
        .checked_add(minimum)
        .ok_or_else(|| "V4 reserve split overflow".to_string())?;
    if final_end != reserve.end || history.v3_split.fresh_validation_range.end != reserve.start {
        return Err("V4 split is not the exact V3 final reserve".to_string());
    }
    let training_range = IndexRangeV0 {
        start: 0,
        end: history.v3_split.fresh_validation_range.start,
    };
    let purge_range = history.v3_split.fresh_validation_range.clone();
    let fresh_validation_range = IndexRangeV0 {
        start: reserve.start,
        end: fresh_end,
    };
    let final_untouched_range = IndexRangeV0 {
        start: fresh_end,
        end: final_end,
    };
    let prior_qualification_overlap_count =
        overlap_count(&history.v1.prior_validation_range, &fresh_validation_range)
            + overlap_count(
                &history.v2_split.fresh_repair_validation_range,
                &fresh_validation_range,
            )
            + overlap_count(
                &history.v3_split.fresh_validation_range,
                &fresh_validation_range,
            );
    let prospective_overlap_count = history
        .v1
        .snapshot
        .actual_start_timestamp_ms
        .map(|start| {
            (fresh_validation_range.start..fresh_validation_range.end)
                .filter_map(|index| {
                    u64::try_from(index)
                        .ok()
                        .and_then(|index| index.checked_mul(reservation.cadence_ms))
                        .and_then(|offset| start.checked_add(offset))
                })
                .filter(|timestamp| reservation.reserved_timestamp_ms.contains(timestamp))
                .count()
        })
        .unwrap_or(1);
    let mut split = MomentumRawFeatureSplitV4 {
        split_version: SPLIT_VERSION_V4.to_string(),
        source_snapshot_digest: history.v1.snapshot.content_digest.clone(),
        v3_split_digest: history.v3_split.split_digest.clone(),
        v3_route_decision_digest: history.v3_decision.decision_digest.clone(),
        training_range,
        purge_range,
        fresh_validation_range,
        final_untouched_range,
        minimum_validation_samples: minimum,
        minimum_final_reserve_samples: minimum,
        prior_qualification_overlap_count,
        prospective_overlap_count,
        historical_test_overlap_count: 0,
        future_evaluation_overlap_count: 0,
        split_digest: String::new(),
    };
    split.split_digest = split_digest_v4(&split);
    validate_split_v4(&split)?;
    Ok(split)
}

fn schema_digest_v4(kind: MomentumRawFeatureModelKindV4, original_dimension: usize) -> String {
    match kind {
        MomentumRawFeatureModelKindV4::RawFeatureLogistic => stable_hash_string(&format!(
            "raw-feature-schema-v4:ordered-original:{original_dimension}"
        )),
        MomentumRawFeatureModelKindV4::RawFeatureInteractionLogistic => {
            let order =
                (0..original_dimension)
                    .map(|i| format!("x{i}"))
                    .chain((0..original_dimension).map(|i| format!("x{i}^2")))
                    .chain((0..original_dimension).flat_map(|i| {
                        ((i + 1)..original_dimension).map(move |j| format!("x{i}*x{j}"))
                    }))
                    .collect::<Vec<_>>();
            stable_hash_string(&format!("interaction-feature-schema-v4:{order:?}"))
        }
        MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant => {
            stable_hash_string("training-prevalence-constant-schema-v4")
        }
    }
}

fn derive_registration_v4(
    history: &FrozenHistoryV4,
    closure: &MomentumFrozenMambaPathClosureV4,
    split: &MomentumRawFeatureSplitV4,
) -> Result<MomentumRawFeatureRegistrationV4, String> {
    validate_closure_v4(closure)?;
    validate_split_v4(split)?;
    let config = MomentumLearningCampaignConfigV0::default();
    let training_candles =
        candles_from_snapshot_prefix(&history.v1.snapshot, split.training_range.end)?;
    let dimension = build_momentum_features_v0(&training_candles, &config.feature_config)
        .map_err(|_| "V4 registration feature schema unavailable".to_string())?
        .first()
        .map(|row| row.values.len())
        .filter(|dimension| *dimension > 0)
        .ok_or_else(|| "V4 registration feature dimension unavailable".to_string())?;
    let base_seed = config.campaign_seed ^ 0x80A0_0000;
    let make = |participant_id: &str, model_kind, seed: u64, learned: bool| {
        let mut value = MomentumRawFeatureParticipantConfigV4 {
            participant_id: participant_id.to_string(),
            model_kind,
            feature_policy_digest: history.v1.session.feature_policy_digest.clone(),
            label_policy_digest: history.v1.session.label_policy_digest.clone(),
            input_feature_schema_digest: schema_digest_v4(model_kind, dimension),
            learning_rate_bits: if learned {
                config.training_config.optimizer.learning_rate.to_bits()
            } else {
                0
            },
            l2_regularization_bits: if learned {
                config.training_config.optimizer.weight_decay.to_bits()
            } else {
                0
            },
            maximum_epochs: if learned {
                config.training_config.epochs
            } else {
                0
            },
            initialization_seed: if learned { seed } else { 0 },
            fresh_initialization: learned,
            training_only_normalizer: learned,
            config_digest: String::new(),
        };
        value.config_digest = config_digest_v4(&value);
        value
    };
    let participants = vec![
        make(
            "RawFeatureLogisticV4",
            MomentumRawFeatureModelKindV4::RawFeatureLogistic,
            base_seed ^ 1,
            true,
        ),
        make(
            "RawFeatureInteractionLogisticV4",
            MomentumRawFeatureModelKindV4::RawFeatureInteractionLogistic,
            base_seed ^ 2,
            true,
        ),
        make(
            "TrainingPrevalenceConstantV4",
            MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant,
            0,
            false,
        ),
    ];
    let mut registration = MomentumRawFeatureRegistrationV4 {
        registration_version: REGISTRATION_VERSION_V4.to_string(),
        agent_id: AGENT_ID_V4.to_string(),
        source_snapshot_digest: history.v1.snapshot.content_digest.clone(),
        canonical_intent_digest: history.v1.input.input.intent.intent_digest.clone(),
        canonical_view_digest: history.v1.input.input.view.view_digest.clone(),
        frozen_mamba_closure_digest: closure.closure_digest.clone(),
        split_digest: split.split_digest.clone(),
        participants,
        maximum_learned_participants: MAXIMUM_LEARNED_PARTICIPANTS_V4,
        interaction_contribution_policy_digest: interaction_policy_digest_v4(),
        fresh_validation_hidden: true,
        final_reserve_forbidden: true,
        historical_test_forbidden: true,
        future_evaluation_forbidden: true,
        winner_selection_forbidden: true,
        active_promotion_forbidden: true,
        reward_application_forbidden: true,
        registration_digest: String::new(),
    };
    registration.registration_digest = registration_digest_v4(&registration);
    validate_registration_v4(&registration)?;
    Ok(registration)
}

fn examples_with_labels(
    examples: &[SequenceExampleV0],
    range: &IndexRangeV0,
) -> Vec<SequenceExampleV0> {
    examples
        .iter()
        .filter(|item| item.label_index >= range.start && item.label_index < range.end)
        .cloned()
        .collect()
}

fn derive_validation_yield_audit_v4(
    source_snapshot_digest: &str,
    label_policy_digest: &str,
    range: &IndexRangeV0,
    minimum_required_valid_samples: usize,
    candles: &[MomentumCandleV0],
    features: &[MomentumFeatureRowV0],
    config: &MomentumSequenceConfigV0,
) -> Result<MomentumValidationYieldAuditV4, String> {
    config
        .validate()
        .map_err(|_| "V4 validation-yield policy rejected".to_string())?;
    let validation_index_count = range
        .end
        .checked_sub(range.start)
        .ok_or_else(|| "V4 validation-yield range rejected".to_string())?;
    let mut valid_labelled_sample_count = 0usize;
    let mut neutral_excluded_count = 0usize;
    let mut horizon_unavailable_count = 0usize;
    let mut feature_unavailable_count = 0usize;
    for label_index in range.start..range.end {
        let Some(sequence_end) = label_index.checked_sub(config.prediction_horizon) else {
            horizon_unavailable_count += 1;
            continue;
        };
        if label_index >= candles.len() {
            horizon_unavailable_count += 1;
            continue;
        }
        let Some(end_position) = features
            .iter()
            .position(|feature| feature.source_index == sequence_end)
        else {
            feature_unavailable_count += 1;
            continue;
        };
        if end_position + 1 < config.sequence_length
            || features[end_position + 1 - config.sequence_length..=end_position]
                .windows(2)
                .any(|pair| pair[1].source_index != pair[0].source_index + 1)
        {
            feature_unavailable_count += 1;
            continue;
        }
        let future_return = candles[label_index].close / candles[sequence_end].close - 1.0;
        if !future_return.is_finite() {
            return Err("V4 validation-yield return rejected".to_string());
        }
        if !config.include_neutral_labels && future_return.abs() <= config.label_dead_zone {
            neutral_excluded_count += 1;
        } else {
            valid_labelled_sample_count += 1;
        }
    }
    let mut audit = MomentumValidationYieldAuditV4 {
        audit_version: VALIDATION_YIELD_AUDIT_VERSION_V4.to_string(),
        source_snapshot_digest: source_snapshot_digest.to_string(),
        label_policy_digest: label_policy_digest.to_string(),
        validation_index_range: range.clone(),
        validation_index_count,
        minimum_required_valid_samples,
        valid_labelled_sample_count,
        neutral_excluded_count,
        horizon_unavailable_count,
        feature_unavailable_count,
        substantive_qualification_possible: valid_labelled_sample_count
            >= minimum_required_valid_samples,
        audit_digest: String::new(),
    };
    audit.audit_digest = validation_yield_audit_digest_v4(&audit);
    validate_validation_yield_audit_v4(&audit)?;
    Ok(audit)
}

pub(crate) fn raw_encoded(
    examples: &[SequenceExampleV0],
) -> Result<Vec<EncodedTrainingExampleV0>, String> {
    examples
        .iter()
        .map(|item| {
            Ok(EncodedTrainingExampleV0 {
                representation: item
                    .input
                    .last()
                    .cloned()
                    .ok_or_else(|| "V4 raw feature vector unavailable".to_string())?,
                label: item.label,
                snapshot_ids: item.snapshot_ids.clone(),
            })
        })
        .collect()
}

fn expand_interactions_v4(
    rows: &[EncodedTrainingExampleV0],
) -> Result<Vec<EncodedTrainingExampleV0>, String> {
    let dimension = rows.first().map_or(0, |item| item.representation.len());
    if dimension == 0 {
        return Err("V4 interaction dimension unavailable".to_string());
    }
    rows.iter()
        .map(|item| {
            if item.representation.len() != dimension
                || item.representation.iter().any(|value| !value.is_finite())
            {
                return Err("V4 interaction input rejected".to_string());
            }
            let expanded = expand_interaction_representation_v4(&item.representation)?;
            Ok(EncodedTrainingExampleV0 {
                representation: expanded,
                label: item.label,
                snapshot_ids: item.snapshot_ids.clone(),
            })
        })
        .collect()
}

pub(crate) fn expand_interaction_representation_v4(values: &[f32]) -> Result<Vec<f32>, String> {
    let dimension = values.len();
    if dimension == 0 || values.iter().any(|value| !value.is_finite()) {
        return Err("V4 interaction input rejected".to_string());
    }
    let expected =
        dimension + dimension + dimension.saturating_mul(dimension.saturating_sub(1)) / 2;
    let mut expanded = Vec::with_capacity(expected);
    expanded.extend_from_slice(values);
    expanded.extend(values.iter().map(|value| value * value));
    for i in 0..dimension {
        for j in (i + 1)..dimension {
            expanded.push(values[i] * values[j]);
        }
    }
    if expanded.len() != expected || expanded.iter().any(|value| !value.is_finite()) {
        return Err("V4 interaction expansion rejected".to_string());
    }
    Ok(expanded)
}

pub(crate) fn train_head_v4(
    mut head: LogisticPredictionHeadV0,
    training: &[EncodedTrainingExampleV0],
    config: &HeadTrainingConfigV0,
) -> Result<LogisticPredictionHeadV0, String> {
    config
        .validate()
        .map_err(|_| "V4 training policy rejected".to_string())?;
    if training.is_empty() {
        return Err("V4 training evidence unavailable".to_string());
    }
    for _ in 0..config.epochs {
        for batch in training.chunks(config.batch_size) {
            let (_, gradients) = brier_loss_and_gradients_v0(&head, batch)
                .map_err(|_| "V4 finite gradient guard rejected".to_string())?;
            apply_sgd_v0(&mut head, &gradients, &config.optimizer)
                .map_err(|_| "V4 finite parameter guard rejected".to_string())?;
        }
    }
    head.validate()
        .map_err(|_| "V4 trained head rejected".to_string())?;
    Ok(head)
}

fn prediction_digest_v4(values: &[f32]) -> String {
    stable_hash_string(&format!(
        "private-v4-predictions:{:?}",
        values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    ))
}

fn metric_digest_v4(kind: MomentumRawFeatureModelKindV4, metric: &EvaluationMetricsV0) -> String {
    stable_hash_string(&format!("private-v4-metric:{kind:?}:{metric:?}"))
}

fn probabilities_collapsed_v4(probabilities: &[f32]) -> Result<bool, String> {
    if probabilities.is_empty()
        || probabilities
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return Err("V4 probability diagnostic rejected".to_string());
    }
    let mean = probabilities.iter().sum::<f32>() / probabilities.len() as f32;
    let variance = probabilities
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f32>()
        / probabilities.len() as f32;
    Ok(variance <= 1e-6
        || probabilities.iter().all(|value| *value < 0.5)
        || probabilities.iter().all(|value| *value >= 0.5))
}

fn base_qualification_v4(
    metric: &EvaluationMetricsV0,
    probabilities: &[f32],
    minimum: usize,
) -> MomentumRawFeatureQualificationStatusV4 {
    if metric.sample_count < minimum {
        MomentumRawFeatureQualificationStatusV4::RejectedInsufficientValidation
    } else if !metric.brier_score.is_finite()
        || !metric.accuracy.is_finite()
        || probabilities.iter().any(|value| !value.is_finite())
    {
        MomentumRawFeatureQualificationStatusV4::RejectedNumericalFailure
    } else if probabilities_collapsed_v4(probabilities).unwrap_or(true) {
        MomentumRawFeatureQualificationStatusV4::RejectedProbabilityCollapse
    } else {
        MomentumRawFeatureQualificationStatusV4::QualifiedLearned
    }
}

fn validation_timestamp_digest_v4(rows: &[SequenceExampleV0]) -> String {
    stable_hash_string(&format!(
        "v4-validation-label-identities:{:?}",
        rows.iter()
            .map(|item| (item.label_index, &item.snapshot_ids))
            .collect::<Vec<_>>()
    ))
}

fn make_participant_v4(
    config: &MomentumRawFeatureParticipantConfigV4,
    role: MomentumRawFeatureRoleV4,
    history: &FrozenHistoryV4,
    split: &MomentumRawFeatureSplitV4,
    validation_identity: &str,
    parameter_digest: String,
    normalizer_digest: String,
    training_identity_digest: String,
) -> FrozenCandidateParticipantV4 {
    let mut value = FrozenCandidateParticipantV4 {
        participant_version: PARTICIPANT_VERSION_V4.to_string(),
        participant_id: config.participant_id.clone(),
        participant_role: role,
        model_kind: config.model_kind,
        config_digest: config.config_digest.clone(),
        source_snapshot_digest: history.v1.snapshot.content_digest.clone(),
        training_range_digest: range_digest_v4("training", &split.training_range),
        fresh_validation_range_digest: range_digest_v4(
            "fresh-validation",
            &split.fresh_validation_range,
        ),
        validation_timestamp_digest: validation_identity.to_string(),
        input_feature_schema_digest: config.input_feature_schema_digest.clone(),
        model_artifact_digest: stable_hash_string(&format!(
            "v4-model-artifact:{}:{parameter_digest}:{normalizer_digest}",
            config.config_digest
        )),
        parameter_digest,
        normalizer_digest,
        training_identity_digest,
        fresh_initialization: config.fresh_initialization,
        prior_parameters_reused: false,
        prior_normalizer_reused: false,
        prior_predictions_reused: false,
        validation_parameter_updates: 0,
        deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
        participant_digest: String::new(),
    };
    value.participant_digest = participant_digest_v4(&value);
    value
}

fn make_receipt_v4(
    participant: &FrozenCandidateParticipantV4,
    status: MomentumRawFeatureQualificationStatusV4,
    metric: &EvaluationMetricsV0,
    audit_digest: Option<String>,
) -> MomentumRawFeatureQualificationReceiptV4 {
    let mut value = MomentumRawFeatureQualificationReceiptV4 {
        receipt_version: RECEIPT_VERSION_V4.to_string(),
        participant_id: participant.participant_id.clone(),
        participant_role: participant.participant_role,
        participant_digest: participant.participant_digest.clone(),
        fresh_validation_range_digest: participant.fresh_validation_range_digest.clone(),
        qualification_policy_digest: qualification_policy_digest_v4(),
        private_metric_digest: metric_digest_v4(participant.model_kind, metric),
        interaction_contribution_audit_digest: audit_digest,
        status,
        validation_parameter_updates: 0,
        final_reserve_reads: 0,
        historical_test_reads: 0,
        future_evaluation_reads: 0,
        receipt_digest: String::new(),
    };
    value.receipt_digest = receipt_digest_v4(&value);
    value
}

fn contribution_audit_v4(
    participant: &FrozenCandidateParticipantV4,
    head: &LogisticPredictionHeadV0,
    validation: &[EncodedTrainingExampleV0],
    original_dimension: usize,
) -> Result<MomentumInteractionContributionAuditV4, String> {
    let squared_end = original_dimension
        .checked_mul(2)
        .ok_or_else(|| "V4 contribution dimension overflow".to_string())?;
    if head.weights.len() <= squared_end
        || validation
            .iter()
            .any(|row| row.representation.len() != head.weights.len())
    {
        return Err("V4 contribution dimension rejected".to_string());
    }
    let full = validation
        .iter()
        .map(|row| head.probability(&row.representation))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "V4 full contribution prediction rejected".to_string())?;
    let mut ablated_head = head.clone();
    for weight in &mut ablated_head.weights[original_dimension..] {
        *weight = 0.0;
    }
    let ablated = validation
        .iter()
        .map(|row| ablated_head.probability(&row.representation))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "V4 ablated contribution prediction rejected".to_string())?;
    let effect = full
        .iter()
        .zip(&ablated)
        .map(|(left, right)| (left - right).abs())
        .sum::<f32>()
        / full.len() as f32;
    let original_nonzero = head.weights[..original_dimension]
        .iter()
        .any(|value| value.abs() > 1e-8);
    let nonlinear_nonzero = head.weights[original_dimension..]
        .iter()
        .any(|value| value.abs() > 1e-8);
    let status = if !effect.is_finite() || !original_nonzero || !nonlinear_nonzero {
        InteractionContributionStatusV4::Invalid
    } else if effect >= f32::from_bits(MATERIAL_INTERACTION_EFFECT_BITS_V4) {
        InteractionContributionStatusV4::MaterialInteractionContribution
    } else if effect >= f32::from_bits(DETECTABLE_INTERACTION_EFFECT_BITS_V4) {
        InteractionContributionStatusV4::DetectableButBelowPolicy
    } else {
        InteractionContributionStatusV4::LinearEquivalent
    };
    let digest_block = |label: &str, values: &[f32]| {
        stable_hash_string(&format!(
            "v4-{label}-parameter-block:{:?}",
            values
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        ))
    };
    let mut audit = MomentumInteractionContributionAuditV4 {
        participant_digest: participant.participant_digest.clone(),
        original_feature_parameter_digest: digest_block(
            "original",
            &head.weights[..original_dimension],
        ),
        squared_feature_parameter_digest: digest_block(
            "squared",
            &head.weights[original_dimension..squared_end],
        ),
        pairwise_feature_parameter_digest: digest_block("pairwise", &head.weights[squared_end..]),
        original_block_nonzero: original_nonzero,
        nonlinear_blocks_nonzero: nonlinear_nonzero,
        full_prediction_digest: prediction_digest_v4(&full),
        nonlinear_ablated_prediction_digest: prediction_digest_v4(&ablated),
        contribution_policy_digest: interaction_policy_digest_v4(),
        contribution_status: status,
        audit_digest: String::new(),
    };
    audit.audit_digest = contribution_digest_v4(&audit);
    validate_contribution_v4(&audit)?;
    Ok(audit)
}

fn derive_roster_v4(
    family: &MomentumRawFeatureFamilyV4,
) -> Result<
    (
        Option<MomentumRawFeatureFutureRosterV4>,
        MomentumRawFeatureRosterStatusV4,
    ),
    String,
> {
    validate_family_v4(family)?;
    let status_for = |digest: &str| {
        family
            .qualification_receipts
            .iter()
            .find(|item| item.participant_digest == digest)
            .map(|item| item.status)
    };
    let learned = sorted_unique(
        family
            .participants
            .iter()
            .filter(|item| {
                item.participant_role != MomentumRawFeatureRoleV4::ConstantBenchmark
                    && status_for(&item.participant_digest)
                        == Some(MomentumRawFeatureQualificationStatusV4::QualifiedLearned)
            })
            .map(|item| item.participant_digest.clone())
            .collect(),
    );
    let benchmarks = sorted_unique(
        family
            .participants
            .iter()
            .filter(|item| {
                item.participant_role == MomentumRawFeatureRoleV4::ConstantBenchmark
                    && status_for(&item.participant_digest)
                        == Some(MomentumRawFeatureQualificationStatusV4::BenchmarkQualified)
            })
            .map(|item| item.participant_digest.clone())
            .collect(),
    );
    let duplicates = sorted_unique(
        family
            .participants
            .iter()
            .filter(|item| {
                status_for(&item.participant_digest)
                    == Some(MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent)
            })
            .map(|item| item.participant_digest.clone())
            .collect(),
    );
    if learned.is_empty() {
        if family.qualification_receipts.iter().all(|receipt| {
            receipt.status
                == MomentumRawFeatureQualificationStatusV4::RejectedInsufficientValidation
        }) {
            return Ok((
                None,
                MomentumRawFeatureRosterStatusV4::QualificationEvidenceInsufficient,
            ));
        }
        return Ok((
            None,
            if duplicates.is_empty() {
                MomentumRawFeatureRosterStatusV4::NoQualifiedLearnedParticipant
            } else {
                MomentumRawFeatureRosterStatusV4::SemanticDuplicateOnly
            },
        ));
    }
    if benchmarks.is_empty() {
        return Ok((None, MomentumRawFeatureRosterStatusV4::BenchmarkUnavailable));
    }
    let included = learned
        .iter()
        .chain(&benchmarks)
        .chain(&duplicates)
        .collect::<BTreeSet<_>>();
    let rejected = sorted_unique(
        family
            .participants
            .iter()
            .filter(|item| !included.contains(&item.participant_digest))
            .map(|item| item.participant_digest.clone())
            .collect(),
    );
    let mut roster = MomentumRawFeatureFutureRosterV4 {
        roster_version: ROSTER_VERSION_V4.to_string(),
        family_digest: family.family_digest.clone(),
        learned_participant_digests: learned,
        benchmark_participant_digests: benchmarks,
        excluded_semantic_duplicate_digests: duplicates,
        rejected_participant_digests: rejected,
        inclusion_policy_digest: stable_hash_string(
            "v4-roster:all-qualified-learned:qualified-benchmark:deduplicate-linear-equivalent:no-ranking",
        ),
        status: MomentumRawFeatureRosterStatusV4::Ready,
        roster_digest: String::new(),
    };
    roster.roster_digest = roster_digest_v4(&roster);
    validate_roster_v4(&roster, family)?;
    Ok((Some(roster), MomentumRawFeatureRosterStatusV4::Ready))
}

fn derive_evaluation_v4(
    history: &FrozenHistoryV4,
    closure: &MomentumFrozenMambaPathClosureV4,
    split: &MomentumRawFeatureSplitV4,
    registration: &MomentumRawFeatureRegistrationV4,
    family: &MomentumRawFeatureFamilyV4,
    roster: Option<&MomentumRawFeatureFutureRosterV4>,
    roster_status: MomentumRawFeatureRosterStatusV4,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<
    (
        Option<MomentumRawFeatureEvaluationRegistrationV4>,
        MomentumRawFeatureEvaluationStatusV4,
    ),
    String,
> {
    let Some(roster) = roster else {
        return Ok((
            None,
            match roster_status {
                MomentumRawFeatureRosterStatusV4::QualificationEvidenceInsufficient => {
                    MomentumRawFeatureEvaluationStatusV4::QualificationEvidenceInsufficient
                }
                MomentumRawFeatureRosterStatusV4::BenchmarkUnavailable => {
                    MomentumRawFeatureEvaluationStatusV4::BenchmarkUnavailable
                }
                MomentumRawFeatureRosterStatusV4::SemanticDuplicateOnly => {
                    MomentumRawFeatureEvaluationStatusV4::SemanticDuplicateOnly
                }
                _ => MomentumRawFeatureEvaluationStatusV4::NoQualifiedLearnedParticipant,
            },
        ));
    };
    let included = roster
        .learned_participant_digests
        .iter()
        .chain(&roster.benchmark_participant_digests)
        .collect::<BTreeSet<_>>();
    let source_boundary = history
        .v1
        .snapshot
        .actual_end_timestamp_ms
        .ok_or_else(|| "V4 source boundary unavailable".to_string())?;
    let protected_last = *reservation
        .reserved_timestamp_ms
        .last()
        .ok_or_else(|| "V4 protected timestamp unavailable".to_string())?;
    let minimum = source_boundary
        .max(protected_last)
        .max(reservation.provider_finality_boundary_ms)
        .checked_add(reservation.cadence_ms)
        .ok_or_else(|| "V4 future timestamp overflow".to_string())?;
    let mut value = MomentumRawFeatureEvaluationRegistrationV4 {
        registration_version: EVALUATION_VERSION_V4.to_string(),
        agent_id: AGENT_ID_V4.to_string(),
        family_digest: family.family_digest.clone(),
        roster_digest: roster.roster_digest.clone(),
        frozen_mamba_closure_digest: closure.closure_digest.clone(),
        split_digest: split.split_digest.clone(),
        raw_feature_registration_digest: registration.registration_digest.clone(),
        qualification_receipt_digests: sorted_unique(
            family
                .qualification_receipts
                .iter()
                .filter(|item| included.contains(&item.participant_digest))
                .map(|item| item.receipt_digest.clone())
                .collect(),
        ),
        interaction_contribution_audit_digest: family
            .interaction_contribution_audit
            .as_ref()
            .filter(|item| included.contains(&item.participant_digest))
            .map(|item| item.audit_digest.clone()),
        source_snapshot_digest: history.v1.snapshot.content_digest.clone(),
        source_boundary_timestamp_ms: source_boundary,
        protected_registration_digests: sorted_unique(
            reservation.protected_registration_digests.clone(),
        ),
        protected_timestamp_ms: reservation.reserved_timestamp_ms.clone(),
        provider_finality_boundary_ms: reservation.provider_finality_boundary_ms,
        prior_validation_identity_digests: vec![
            range_digest_v4("v1-validation", &history.v1.prior_validation_range),
            range_digest_v4(
                "v2-validation",
                &history.v2_split.fresh_repair_validation_range,
            ),
            range_digest_v4("v3-validation", &history.v3_split.fresh_validation_range),
        ],
        v4_final_untouched_reserve_digest: range_digest_v4(
            "final-untouched",
            &split.final_untouched_range,
        ),
        minimum_accepted_timestamp_ms: minimum,
        labels_hidden_until_opening: true,
        probabilities_hidden_until_opening: true,
        one_time_opening_required: true,
        winner_selection_forbidden_before_opening: true,
        active_promotion_forbidden: true,
        reward_application_forbidden: true,
        maximum_requests: 1,
        maximum_concurrency: 1,
        maximum_retries: 0,
        registration_digest: String::new(),
    };
    value.registration_digest = evaluation_digest_v4(&value);
    validate_evaluation_v4(&value, family, roster)?;
    Ok((
        Some(value),
        MomentumRawFeatureEvaluationStatusV4::Registered,
    ))
}

fn run_experiment_v4(
    history: &FrozenHistoryV4,
    closure: &MomentumFrozenMambaPathClosureV4,
    split: &MomentumRawFeatureSplitV4,
    registration: &MomentumRawFeatureRegistrationV4,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<ExperimentV4, String> {
    validate_closure_v4(closure)?;
    validate_split_v4(split)?;
    validate_registration_v4(registration)?;
    if registration.frozen_mamba_closure_digest != closure.closure_digest
        || registration.split_digest != split.split_digest
    {
        return Err("V4 preregistration binding rejected".to_string());
    }
    let config = MomentumLearningCampaignConfigV0::default();
    let candles =
        candles_from_snapshot_prefix(&history.v1.snapshot, split.fresh_validation_range.end)?;
    let features = build_momentum_features_v0(&candles, &config.feature_config)
        .map_err(|_| "V4 feature derivation rejected".to_string())?;
    let training_features = features
        .iter()
        .filter(|row| row.source_index < split.training_range.end)
        .cloned()
        .collect::<Vec<_>>();
    let feature_normalizer = FeatureNormalizerV0::fit(&training_features)
        .map_err(|_| "V4 feature normalizer rejected".to_string())?;
    if feature_normalizer.fitted_on_end >= split.training_range.end {
        return Err("V4 feature normalization leakage rejected".to_string());
    }
    let normalized = feature_normalizer
        .transform(&features)
        .map_err(|_| "V4 normalized feature derivation rejected".to_string())?;
    let examples = build_momentum_sequence_examples_v0(
        &candles,
        &normalized,
        &config.sequence_config,
        std::slice::from_ref(&history.v1.snapshot.snapshot_id),
    )
    .map_err(|_| "V4 sequence derivation rejected".to_string())?;
    let training_examples = examples_with_labels(&examples, &split.training_range);
    let validation_examples = examples_with_labels(&examples, &split.fresh_validation_range);
    let validation_yield_audit = derive_validation_yield_audit_v4(
        &history.v1.snapshot.content_digest,
        &closure.label_policy_digest,
        &split.fresh_validation_range,
        split.minimum_validation_samples,
        &candles,
        &normalized,
        &config.sequence_config,
    )?;
    if training_examples.is_empty()
        || validation_examples.is_empty()
        || validation_examples.len() != validation_yield_audit.valid_labelled_sample_count
        || validation_examples.iter().any(|item| {
            item.label_index < split.fresh_validation_range.start
                || item.label_index >= split.fresh_validation_range.end
                || item.label_index >= split.final_untouched_range.start
        })
    {
        return Err("V4 fresh validation evidence rejected".to_string());
    }
    let validation_identity = validation_timestamp_digest_v4(&validation_examples);
    let raw_training = raw_encoded(&training_examples)?;
    let raw_validation = raw_encoded(&validation_examples)?;
    let training_identity = stable_hash_string(&format!(
        "v4-training-label-identities:{:?}",
        training_examples
            .iter()
            .map(|item| (item.label_index, &item.snapshot_ids))
            .collect::<Vec<_>>()
    ));
    let mut participants = Vec::new();
    let mut receipts = Vec::new();

    let raw_config = registration
        .participants
        .iter()
        .find(|item| item.model_kind == MomentumRawFeatureModelKindV4::RawFeatureLogistic)
        .ok_or_else(|| "V4 raw configuration missing".to_string())?;
    let raw_rep_normalizer = RepresentationNormalizerV0::fit(&raw_training)
        .map_err(|_| "V4 raw normalizer rejected".to_string())?;
    let raw_train_normalized = raw_rep_normalizer
        .transform(&raw_training)
        .map_err(|_| "V4 raw training transform rejected".to_string())?;
    let raw_validation_normalized = raw_rep_normalizer
        .transform(&raw_validation)
        .map_err(|_| "V4 raw validation transform rejected".to_string())?;
    let mut raw_training_config = config.training_config.clone();
    raw_training_config.seed = raw_config.initialization_seed;
    raw_training_config.early_stopping_patience = None;
    let raw_initial = LogisticPredictionHeadV0::seeded(
        raw_train_normalized[0].representation.len(),
        raw_config.initialization_seed,
    )
    .map_err(|_| "V4 raw initialization rejected".to_string())?;
    let raw_head = train_head_v4(raw_initial, &raw_train_normalized, &raw_training_config)?;
    let raw_metric = evaluate_head_v0(&raw_head, &raw_validation_normalized)
        .map_err(|_| "V4 raw evaluation rejected".to_string())?;
    let raw_probabilities = raw_validation_normalized
        .iter()
        .map(|item| raw_head.probability(&item.representation))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "V4 raw probabilities rejected".to_string())?;
    let raw_participant = make_participant_v4(
        raw_config,
        MomentumRawFeatureRoleV4::LearnedRawLogistic,
        history,
        split,
        &validation_identity,
        raw_head.parameter_digest(),
        stable_hash_string(&format!(
            "{}:{}",
            feature_normalizer.digest(),
            raw_rep_normalizer.digest()
        )),
        training_identity.clone(),
    );
    let raw_status = base_qualification_v4(
        &raw_metric,
        &raw_probabilities,
        split.minimum_validation_samples,
    );
    let raw_receipt = make_receipt_v4(&raw_participant, raw_status, &raw_metric, None);
    participants.push(raw_participant);
    receipts.push(raw_receipt);

    let interaction_config = registration
        .participants
        .iter()
        .find(|item| {
            item.model_kind == MomentumRawFeatureModelKindV4::RawFeatureInteractionLogistic
        })
        .ok_or_else(|| "V4 interaction configuration missing".to_string())?;
    let interaction_training = expand_interactions_v4(&raw_training)?;
    let interaction_validation = expand_interactions_v4(&raw_validation)?;
    let interaction_normalizer = RepresentationNormalizerV0::fit(&interaction_training)
        .map_err(|_| "V4 interaction normalizer rejected".to_string())?;
    let interaction_training = interaction_normalizer
        .transform(&interaction_training)
        .map_err(|_| "V4 interaction training transform rejected".to_string())?;
    let interaction_validation = interaction_normalizer
        .transform(&interaction_validation)
        .map_err(|_| "V4 interaction validation transform rejected".to_string())?;
    let mut interaction_training_config = config.training_config.clone();
    interaction_training_config.seed = interaction_config.initialization_seed;
    interaction_training_config.early_stopping_patience = None;
    let interaction_initial = LogisticPredictionHeadV0::seeded(
        interaction_training[0].representation.len(),
        interaction_config.initialization_seed,
    )
    .map_err(|_| "V4 interaction initialization rejected".to_string())?;
    let interaction_head = train_head_v4(
        interaction_initial,
        &interaction_training,
        &interaction_training_config,
    )?;
    let interaction_metric = evaluate_head_v0(&interaction_head, &interaction_validation)
        .map_err(|_| "V4 interaction evaluation rejected".to_string())?;
    let interaction_probabilities = interaction_validation
        .iter()
        .map(|item| interaction_head.probability(&item.representation))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "V4 interaction probabilities rejected".to_string())?;
    let interaction_participant = make_participant_v4(
        interaction_config,
        MomentumRawFeatureRoleV4::LearnedInteractionLogistic,
        history,
        split,
        &validation_identity,
        interaction_head.parameter_digest(),
        stable_hash_string(&format!(
            "{}:{}",
            feature_normalizer.digest(),
            interaction_normalizer.digest()
        )),
        training_identity.clone(),
    );
    let contribution = contribution_audit_v4(
        &interaction_participant,
        &interaction_head,
        &interaction_validation,
        raw_training[0].representation.len(),
    )?;
    let interaction_base = base_qualification_v4(
        &interaction_metric,
        &interaction_probabilities,
        split.minimum_validation_samples,
    );
    let interaction_status =
        if interaction_base == MomentumRawFeatureQualificationStatusV4::QualifiedLearned {
            match contribution.contribution_status {
                InteractionContributionStatusV4::MaterialInteractionContribution => {
                    MomentumRawFeatureQualificationStatusV4::QualifiedLearned
                }
                InteractionContributionStatusV4::DetectableButBelowPolicy
                | InteractionContributionStatusV4::LinearEquivalent => {
                    MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent
                }
                InteractionContributionStatusV4::Invalid => {
                    MomentumRawFeatureQualificationStatusV4::RejectedFeatureIntegrity
                }
            }
        } else {
            interaction_base
        };
    let interaction_receipt = make_receipt_v4(
        &interaction_participant,
        interaction_status,
        &interaction_metric,
        Some(contribution.audit_digest.clone()),
    );
    participants.push(interaction_participant);
    receipts.push(interaction_receipt);

    let constant_config = registration
        .participants
        .iter()
        .find(|item| item.model_kind == MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant)
        .ok_or_else(|| "V4 benchmark configuration missing".to_string())?;
    let prevalence = training_examples.iter().map(|item| item.label).sum::<f32>()
        / training_examples.len() as f32;
    if !prevalence.is_finite() || !(0.0..=1.0).contains(&prevalence) {
        return Err("V4 training prevalence rejected".to_string());
    }
    let constant_probabilities = vec![prevalence; validation_examples.len()];
    let constant_labels = validation_examples
        .iter()
        .map(|item| item.label)
        .collect::<Vec<_>>();
    let constant_metric = evaluate_probabilities_v0(&constant_probabilities, &constant_labels)
        .map_err(|_| "V4 benchmark evaluation rejected".to_string())?;
    let constant_participant = make_participant_v4(
        constant_config,
        MomentumRawFeatureRoleV4::ConstantBenchmark,
        history,
        split,
        &validation_identity,
        stable_hash_string(&format!("training-prevalence-v4:{}", prevalence.to_bits())),
        stable_hash_string("training-labels-only-no-normalizer-v4"),
        training_identity,
    );
    let constant_status = if constant_metric.sample_count < split.minimum_validation_samples {
        MomentumRawFeatureQualificationStatusV4::RejectedInsufficientValidation
    } else {
        MomentumRawFeatureQualificationStatusV4::BenchmarkQualified
    };
    let constant_receipt = make_receipt_v4(
        &constant_participant,
        constant_status,
        &constant_metric,
        None,
    );
    participants.push(constant_participant);
    receipts.push(constant_receipt);

    let qualified_learned_count = receipts
        .iter()
        .filter(|item| {
            matches!(
                item.status,
                MomentumRawFeatureQualificationStatusV4::QualifiedLearned
                    | MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent
            )
        })
        .count();
    let qualified_benchmark_count = receipts
        .iter()
        .filter(|item| item.status == MomentumRawFeatureQualificationStatusV4::BenchmarkQualified)
        .count();
    let mut family = MomentumRawFeatureFamilyV4 {
        family_version: FAMILY_VERSION_V4.to_string(),
        agent_id: AGENT_ID_V4.to_string(),
        source_snapshot_digest: history.v1.snapshot.content_digest.clone(),
        canonical_view_digest: history.v1.input.input.view.view_digest.clone(),
        frozen_mamba_closure_digest: closure.closure_digest.clone(),
        split_digest: split.split_digest.clone(),
        registration_digest: registration.registration_digest.clone(),
        participants,
        qualification_receipts: receipts,
        interaction_contribution_audit: Some(contribution),
        qualified_learned_count,
        qualified_benchmark_count,
        winner_selected: false,
        final_reserve_accessed: false,
        eligible_for_active_committee: false,
        eligible_for_promotion: false,
        eligible_for_reward: false,
        family_digest: String::new(),
    };
    family.family_digest = family_digest_v4(&family);
    validate_family_v4(&family)?;
    let decision = derive_decision_v4(&family);
    validate_decision_v4(&decision, &family)?;
    let (roster, roster_status) = derive_roster_v4(&family)?;
    let (evaluation, evaluation_status) = derive_evaluation_v4(
        history,
        closure,
        split,
        registration,
        &family,
        roster.as_ref(),
        roster_status,
        reservation,
    )?;
    Ok(ExperimentV4 {
        validation_yield_audit,
        family,
        decision,
        roster,
        roster_status,
        evaluation,
        evaluation_status,
    })
}

pub(crate) fn reconstruct_frozen_momentum_v4(
    root: &Path,
    snapshots: &[DataSnapshot],
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<MomentumFrozenReplayV4, String> {
    let history = load_frozen_history_v4(root, snapshots)?;
    let closure = derive_closure_v4(&history)?;
    let split = derive_split_v4(&history, reservation)?;
    let registration = derive_registration_v4(&history, &closure, &split)?;
    let persisted_preregistration = reopen_preregistration_v4(&root.join("v4").join(AGENT_ID_V4))?;
    if persisted_preregistration != (closure.clone(), split.clone(), registration.clone()) {
        return Err("V4 frozen preregistration identity mismatch".to_string());
    }
    let (persisted, _) = reopen_experiment_v4(&root.join("v4").join(AGENT_ID_V4))?;
    let config = MomentumLearningCampaignConfigV0::default();
    let candles =
        candles_from_snapshot_prefix(&history.v1.snapshot, split.fresh_validation_range.end)?;
    let features = build_momentum_features_v0(&candles, &config.feature_config)
        .map_err(|_| "V4 replay feature derivation rejected".to_string())?;
    let training_features = features
        .iter()
        .filter(|row| row.source_index < split.training_range.end)
        .cloned()
        .collect::<Vec<_>>();
    let feature_normalizer = FeatureNormalizerV0::fit(&training_features)
        .map_err(|_| "V4 replay feature normalizer rejected".to_string())?;
    let normalized = feature_normalizer
        .transform(&features)
        .map_err(|_| "V4 replay normalized features rejected".to_string())?;
    let examples = build_momentum_sequence_examples_v0(
        &candles,
        &normalized,
        &config.sequence_config,
        std::slice::from_ref(&history.v1.snapshot.snapshot_id),
    )
    .map_err(|_| "V4 replay examples rejected".to_string())?;
    let training_examples = examples_with_labels(&examples, &split.training_range);
    let validation_examples = examples_with_labels(&examples, &split.fresh_validation_range);
    let validation_yield_audit = derive_validation_yield_audit_v4(
        &history.v1.snapshot.content_digest,
        &closure.label_policy_digest,
        &split.fresh_validation_range,
        split.minimum_validation_samples,
        &candles,
        &normalized,
        &config.sequence_config,
    )?;
    if validation_yield_audit != persisted.validation_yield_audit
        || validation_examples.len() != validation_yield_audit.valid_labelled_sample_count
    {
        return Err("V4 replay validation-yield identity mismatch".to_string());
    }
    let validation_identity = validation_timestamp_digest_v4(&validation_examples);
    let training_identity = stable_hash_string(&format!(
        "v4-training-label-identities:{:?}",
        training_examples
            .iter()
            .map(|item| (item.label_index, &item.snapshot_ids))
            .collect::<Vec<_>>()
    ));
    let raw_training = raw_encoded(&training_examples)?;
    let raw_validation = raw_encoded(&validation_examples)?;
    let raw_config = registration
        .participants
        .iter()
        .find(|item| item.model_kind == MomentumRawFeatureModelKindV4::RawFeatureLogistic)
        .ok_or_else(|| "V4 replay raw configuration missing".to_string())?;
    let raw_normalizer = RepresentationNormalizerV0::fit(&raw_training)
        .map_err(|_| "V4 replay raw normalizer rejected".to_string())?;
    let raw_training_normalized = raw_normalizer
        .transform(&raw_training)
        .map_err(|_| "V4 replay raw training transform rejected".to_string())?;
    let mut raw_training_config = config.training_config.clone();
    raw_training_config.seed = raw_config.initialization_seed;
    raw_training_config.epochs = raw_config.maximum_epochs;
    raw_training_config.optimizer.learning_rate = f32::from_bits(raw_config.learning_rate_bits);
    raw_training_config.optimizer.weight_decay = f32::from_bits(raw_config.l2_regularization_bits);
    raw_training_config.early_stopping_patience = None;
    let raw_initial = LogisticPredictionHeadV0::seeded(
        raw_training_normalized[0].representation.len(),
        raw_config.initialization_seed,
    )
    .map_err(|_| "V4 replay raw initialization rejected".to_string())?;
    let raw_head = train_head_v4(raw_initial, &raw_training_normalized, &raw_training_config)?;
    let replayed_raw = make_participant_v4(
        raw_config,
        MomentumRawFeatureRoleV4::LearnedRawLogistic,
        &history,
        &split,
        &validation_identity,
        raw_head.parameter_digest(),
        stable_hash_string(&format!(
            "{}:{}",
            feature_normalizer.digest(),
            raw_normalizer.digest()
        )),
        training_identity.clone(),
    );
    let persisted_raw = persisted
        .family
        .participants
        .iter()
        .find(|item| item.participant_role == MomentumRawFeatureRoleV4::LearnedRawLogistic)
        .ok_or_else(|| "V4 frozen raw participant missing".to_string())?;
    if &replayed_raw != persisted_raw {
        return Err("V4 frozen raw participant reconstruction mismatch".to_string());
    }

    let interaction_config = registration
        .participants
        .iter()
        .find(|item| {
            item.model_kind == MomentumRawFeatureModelKindV4::RawFeatureInteractionLogistic
        })
        .ok_or_else(|| "V4 replay interaction configuration missing".to_string())?;
    let interaction_training = expand_interactions_v4(&raw_training)?;
    let interaction_validation = expand_interactions_v4(&raw_validation)?;
    let interaction_normalizer = RepresentationNormalizerV0::fit(&interaction_training)
        .map_err(|_| "V4 replay interaction normalizer rejected".to_string())?;
    let interaction_training_normalized =
        interaction_normalizer
            .transform(&interaction_training)
            .map_err(|_| "V4 replay interaction training transform rejected".to_string())?;
    let interaction_validation_normalized = interaction_normalizer
        .transform(&interaction_validation)
        .map_err(|_| "V4 replay interaction validation transform rejected".to_string())?;
    let mut interaction_training_config = config.training_config.clone();
    interaction_training_config.seed = interaction_config.initialization_seed;
    interaction_training_config.epochs = interaction_config.maximum_epochs;
    interaction_training_config.optimizer.learning_rate =
        f32::from_bits(interaction_config.learning_rate_bits);
    interaction_training_config.optimizer.weight_decay =
        f32::from_bits(interaction_config.l2_regularization_bits);
    interaction_training_config.early_stopping_patience = None;
    let interaction_initial = LogisticPredictionHeadV0::seeded(
        interaction_training_normalized[0].representation.len(),
        interaction_config.initialization_seed,
    )
    .map_err(|_| "V4 replay interaction initialization rejected".to_string())?;
    let interaction_head = train_head_v4(
        interaction_initial,
        &interaction_training_normalized,
        &interaction_training_config,
    )?;
    let replayed_interaction = make_participant_v4(
        interaction_config,
        MomentumRawFeatureRoleV4::LearnedInteractionLogistic,
        &history,
        &split,
        &validation_identity,
        interaction_head.parameter_digest(),
        stable_hash_string(&format!(
            "{}:{}",
            feature_normalizer.digest(),
            interaction_normalizer.digest()
        )),
        training_identity.clone(),
    );
    let persisted_interaction = persisted
        .family
        .participants
        .iter()
        .find(|item| item.participant_role == MomentumRawFeatureRoleV4::LearnedInteractionLogistic)
        .ok_or_else(|| "V4 frozen interaction participant missing".to_string())?;
    if &replayed_interaction != persisted_interaction {
        return Err("V4 frozen interaction participant reconstruction mismatch".to_string());
    }
    let replayed_contribution = contribution_audit_v4(
        &replayed_interaction,
        &interaction_head,
        &interaction_validation_normalized,
        raw_training[0].representation.len(),
    )?;
    if persisted.family.interaction_contribution_audit.as_ref() != Some(&replayed_contribution) {
        return Err("V4 frozen interaction audit reconstruction mismatch".to_string());
    }

    let training_prevalence = training_examples.iter().map(|item| item.label).sum::<f32>()
        / training_examples.len() as f32;
    let constant_config = registration
        .participants
        .iter()
        .find(|item| item.model_kind == MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant)
        .ok_or_else(|| "V4 replay benchmark configuration missing".to_string())?;
    let replayed_constant = make_participant_v4(
        constant_config,
        MomentumRawFeatureRoleV4::ConstantBenchmark,
        &history,
        &split,
        &validation_identity,
        stable_hash_string(&format!(
            "training-prevalence-v4:{}",
            training_prevalence.to_bits()
        )),
        stable_hash_string("training-labels-only-no-normalizer-v4"),
        training_identity,
    );
    let persisted_constant = persisted
        .family
        .participants
        .iter()
        .find(|item| item.participant_role == MomentumRawFeatureRoleV4::ConstantBenchmark)
        .ok_or_else(|| "V4 frozen benchmark participant missing".to_string())?;
    if &replayed_constant != persisted_constant {
        return Err("V4 frozen benchmark reconstruction mismatch".to_string());
    }
    if persisted.decision != derive_decision_v4(&persisted.family) {
        return Err("V4 corrected decision reconstruction mismatch".to_string());
    }
    Ok(MomentumFrozenReplayV4 {
        history,
        closure,
        split,
        registration,
        validation_yield_audit,
        family: persisted.family,
        decision: persisted.decision,
        feature_normalizer,
        raw_normalizer,
        interaction_normalizer,
        raw_head,
        interaction_head,
        training_prevalence,
    })
}

pub(crate) fn predict_frozen_momentum_v4_event(
    replay: &MomentumFrozenReplayV4,
    roster_participant_digests: &[String],
    context_rows: &[HistoricalOhlcvRow],
) -> Result<MomentumFrozenPredictionV4, String> {
    let config = MomentumLearningCampaignConfigV0::default();
    let required_row_count = config
        .feature_config
        .minimum_history()
        .map_err(|_| "V4.2 feature-history policy rejected".to_string())?
        .checked_add(config.sequence_config.sequence_length.saturating_sub(1))
        .ok_or_else(|| "V4.2 feature-history length overflow".to_string())?;
    if context_rows.len() != required_row_count
        || context_rows.is_empty()
        || roster_participant_digests.len() != replay.family.participants.len()
        || context_rows.windows(2).any(|pair| {
            pair[1].timestamp_ms
                != pair[0]
                    .timestamp_ms
                    .checked_add(86_400_000)
                    .unwrap_or_default()
        })
    {
        return Err("V4.2 prospective context identity rejected".to_string());
    }
    let candles = context_rows
        .iter()
        .map(|row| {
            if row.timestamp_ms > i64::MAX as u64
                || row.symbol.trim().is_empty()
                || ![
                    row.open,
                    row.high,
                    row.low,
                    row.close,
                    row.volume,
                    row.trade_value.unwrap_or_default(),
                ]
                .iter()
                .all(|value| value.is_finite())
                || row.open <= 0.0
                || row.high <= 0.0
                || row.low <= 0.0
                || row.close <= 0.0
                || row.volume < 0.0
                || row.trade_value.is_some_and(|value| value < 0.0)
                || row.high < row.open.max(row.close)
                || row.low > row.open.min(row.close)
            {
                return Err("V4.2 prospective candle rejected".to_string());
            }
            Ok(MomentumCandleV0 {
                timestamp: row.timestamp_ms as i64,
                open: row.open as f32,
                high: row.high as f32,
                low: row.low as f32,
                close: row.close as f32,
                volume: row.volume as f32,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let features = build_momentum_features_v0(&candles, &config.feature_config)
        .map_err(|_| "V4.2 prospective feature derivation rejected".to_string())?;
    let normalized = replay
        .feature_normalizer
        .transform(&features)
        .map_err(|_| "V4.2 frozen feature normalization rejected".to_string())?;
    if normalized.len() != config.sequence_config.sequence_length {
        return Err("V4.2 prospective sequence dimension rejected".to_string());
    }
    let raw_representation = normalized
        .last()
        .map(|row| row.values.clone())
        .ok_or_else(|| "V4.2 prospective feature vector unavailable".to_string())?;
    let raw_normalized = replay
        .raw_normalizer
        .transform_representation(&raw_representation)
        .map_err(|_| "V4.2 frozen raw normalization rejected".to_string())?;
    let interaction_representation = expand_interaction_representation_v4(&raw_representation)?;
    let interaction_normalized = replay
        .interaction_normalizer
        .transform_representation(&interaction_representation)
        .map_err(|_| "V4.2 frozen interaction normalization rejected".to_string())?;
    let raw_probability = replay
        .raw_head
        .probability(&raw_normalized)
        .map_err(|_| "V4.2 raw prediction rejected".to_string())?;
    let interaction_probability = replay
        .interaction_head
        .probability(&interaction_normalized)
        .map_err(|_| "V4.2 interaction prediction rejected".to_string())?;
    if !replay.training_prevalence.is_finite() || !(0.0..=1.0).contains(&replay.training_prevalence)
    {
        return Err("V4.2 benchmark prediction rejected".to_string());
    }
    let feature_identity_digest = stable_hash_string(&format!(
        "momentum-v4.2-prospective-feature:{:?}:{:?}",
        context_rows
            .iter()
            .map(|row| row.timestamp_ms)
            .collect::<Vec<_>>(),
        normalized
            .iter()
            .map(|row| {
                row.values
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    ));
    let mut predictions = Vec::with_capacity(roster_participant_digests.len());
    for participant_digest in roster_participant_digests {
        let participant = replay
            .family
            .participants
            .iter()
            .find(|item| &item.participant_digest == participant_digest)
            .ok_or_else(|| "V4.2 roster participant identity rejected".to_string())?;
        let probability_bits = match participant.participant_role {
            MomentumRawFeatureRoleV4::LearnedRawLogistic => {
                if replay.raw_head.parameter_digest() != participant.parameter_digest
                    || stable_hash_string(&format!(
                        "{}:{}",
                        replay.feature_normalizer.digest(),
                        replay.raw_normalizer.digest()
                    )) != participant.normalizer_digest
                {
                    return Err("V4.2 raw participant reconstruction changed".to_string());
                }
                raw_probability.to_bits()
            }
            MomentumRawFeatureRoleV4::LearnedInteractionLogistic => {
                if replay.interaction_head.parameter_digest() != participant.parameter_digest
                    || stable_hash_string(&format!(
                        "{}:{}",
                        replay.feature_normalizer.digest(),
                        replay.interaction_normalizer.digest()
                    )) != participant.normalizer_digest
                {
                    return Err("V4.2 interaction participant reconstruction changed".to_string());
                }
                interaction_probability.to_bits()
            }
            MomentumRawFeatureRoleV4::ConstantBenchmark => {
                if stable_hash_string(&format!(
                    "training-prevalence-v4:{}",
                    replay.training_prevalence.to_bits()
                )) != participant.parameter_digest
                {
                    return Err("V4.2 benchmark participant reconstruction changed".to_string());
                }
                replay.training_prevalence.to_bits()
            }
        };
        predictions.push(MomentumFrozenParticipantPredictionV4 {
            participant_digest: participant.participant_digest.clone(),
            config_digest: participant.config_digest.clone(),
            parameter_digest: participant.parameter_digest.clone(),
            normalizer_digest: participant.normalizer_digest.clone(),
            model_artifact_digest: participant.model_artifact_digest.clone(),
            feature_schema_digest: participant.input_feature_schema_digest.clone(),
            training_identity_digest: participant.training_identity_digest.clone(),
            probability_bits,
        });
    }
    Ok(MomentumFrozenPredictionV4 {
        feature_identity_digest,
        participant_predictions: predictions,
    })
}

pub(crate) fn evaluate_frozen_momentum_v4_accumulated(
    replay: &MomentumFrozenReplayV4,
) -> Result<MomentumAccumulatedReplayEvaluationV4, String> {
    let config = MomentumLearningCampaignConfigV0::default();
    let candles = candles_from_snapshot_prefix(
        &replay.history.v1.snapshot,
        replay.split.final_untouched_range.end,
    )?;
    let features = build_momentum_features_v0(&candles, &config.feature_config)
        .map_err(|_| "V4.1 accumulated feature derivation rejected".to_string())?;
    let normalized = replay
        .feature_normalizer
        .transform(&features)
        .map_err(|_| "V4.1 accumulated feature normalization rejected".to_string())?;
    let examples = build_momentum_sequence_examples_v0(
        &candles,
        &normalized,
        &config.sequence_config,
        std::slice::from_ref(&replay.history.v1.snapshot.snapshot_id),
    )
    .map_err(|_| "V4.1 accumulated examples rejected".to_string())?;
    let original = examples_with_labels(&examples, &replay.split.fresh_validation_range);
    let supplemental = examples_with_labels(&examples, &replay.split.final_untouched_range);
    let supplemental_yield = derive_validation_yield_audit_v4(
        &replay.history.v1.snapshot.content_digest,
        &replay.closure.label_policy_digest,
        &replay.split.final_untouched_range,
        replay.split.minimum_validation_samples,
        &candles,
        &normalized,
        &config.sequence_config,
    )?;
    if original.len() != replay.validation_yield_audit.valid_labelled_sample_count
        || supplemental.len() != supplemental_yield.valid_labelled_sample_count
    {
        return Err("V4.1 accumulated yield mismatch".to_string());
    }
    let mut accumulated = original.clone();
    accumulated.extend(supplemental.clone());
    let identities = accumulated
        .iter()
        .map(|item| item.label_index)
        .collect::<BTreeSet<_>>();
    if identities.len() != accumulated.len()
        || accumulated.iter().any(|item| {
            item.label_index >= replay.split.final_untouched_range.end
                || (item.label_index < replay.split.fresh_validation_range.start)
                || (item.label_index >= replay.split.fresh_validation_range.end
                    && item.label_index < replay.split.final_untouched_range.start)
        })
    {
        return Err("V4.1 accumulated evidence union rejected".to_string());
    }
    let accumulated_validation_identity_digest = validation_timestamp_digest_v4(&accumulated);
    let raw_rows = raw_encoded(&accumulated)?;
    let raw_normalized = replay
        .raw_normalizer
        .transform(&raw_rows)
        .map_err(|_| "V4.1 raw accumulated transform rejected".to_string())?;
    let raw_metric = evaluate_head_v0(&replay.raw_head, &raw_normalized)
        .map_err(|_| "V4.1 raw accumulated evaluation rejected".to_string())?;
    let raw_probabilities = raw_normalized
        .iter()
        .map(|item| replay.raw_head.probability(&item.representation))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "V4.1 raw accumulated probabilities rejected".to_string())?;
    let raw_status = base_qualification_v4(
        &raw_metric,
        &raw_probabilities,
        replay.split.minimum_validation_samples,
    );

    let interaction_rows = expand_interactions_v4(&raw_rows)?;
    let interaction_normalized = replay
        .interaction_normalizer
        .transform(&interaction_rows)
        .map_err(|_| "V4.1 interaction accumulated transform rejected".to_string())?;
    let interaction_metric = evaluate_head_v0(&replay.interaction_head, &interaction_normalized)
        .map_err(|_| "V4.1 interaction accumulated evaluation rejected".to_string())?;
    let interaction_probabilities = interaction_normalized
        .iter()
        .map(|item| replay.interaction_head.probability(&item.representation))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "V4.1 interaction accumulated probabilities rejected".to_string())?;
    let interaction_participant = replay
        .family
        .participants
        .iter()
        .find(|item| item.participant_role == MomentumRawFeatureRoleV4::LearnedInteractionLogistic)
        .ok_or_else(|| "V4.1 interaction participant missing".to_string())?;
    let interaction_contribution = contribution_audit_v4(
        interaction_participant,
        &replay.interaction_head,
        &interaction_normalized,
        raw_rows[0].representation.len(),
    )?;
    let interaction_base = base_qualification_v4(
        &interaction_metric,
        &interaction_probabilities,
        replay.split.minimum_validation_samples,
    );
    let interaction_status =
        if interaction_base == MomentumRawFeatureQualificationStatusV4::QualifiedLearned {
            match interaction_contribution.contribution_status {
                InteractionContributionStatusV4::MaterialInteractionContribution => {
                    MomentumRawFeatureQualificationStatusV4::QualifiedLearned
                }
                InteractionContributionStatusV4::DetectableButBelowPolicy
                | InteractionContributionStatusV4::LinearEquivalent => {
                    MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent
                }
                InteractionContributionStatusV4::Invalid => {
                    MomentumRawFeatureQualificationStatusV4::RejectedFeatureIntegrity
                }
            }
        } else {
            interaction_base
        };

    let constant_probabilities = vec![replay.training_prevalence; accumulated.len()];
    let labels = accumulated
        .iter()
        .map(|item| item.label)
        .collect::<Vec<_>>();
    let constant_metric = evaluate_probabilities_v0(&constant_probabilities, &labels)
        .map_err(|_| "V4.1 benchmark accumulated evaluation rejected".to_string())?;
    let constant_status = if constant_metric.sample_count < replay.split.minimum_validation_samples
    {
        MomentumRawFeatureQualificationStatusV4::RejectedInsufficientValidation
    } else {
        MomentumRawFeatureQualificationStatusV4::BenchmarkQualified
    };
    let evaluation_for = |role,
                          status,
                          metric: &EvaluationMetricsV0|
     -> Result<MomentumAccumulatedParticipantEvaluationV4, String> {
        let participant = replay
            .family
            .participants
            .iter()
            .find(|item| item.participant_role == role)
            .ok_or_else(|| "V4.1 frozen participant missing".to_string())?;
        let receipt = replay
            .family
            .qualification_receipts
            .iter()
            .find(|item| item.participant_digest == participant.participant_digest)
            .ok_or_else(|| "V4.1 original receipt missing".to_string())?;
        Ok(MomentumAccumulatedParticipantEvaluationV4 {
            participant_digest: participant.participant_digest.clone(),
            original_receipt_digest: receipt.receipt_digest.clone(),
            status,
            private_metric_digest: metric_digest_v4(participant.model_kind, metric),
        })
    };
    Ok(MomentumAccumulatedReplayEvaluationV4 {
        original_valid_sample_count: original.len(),
        supplemental_valid_sample_count: supplemental.len(),
        original_neutral_excluded_count: replay.validation_yield_audit.neutral_excluded_count,
        supplemental_neutral_excluded_count: supplemental_yield.neutral_excluded_count,
        accumulated_validation_identity_digest,
        source_boundary_timestamp_ms: candles
            .last()
            .and_then(|candle| u64::try_from(candle.timestamp).ok())
            .ok_or_else(|| "V4.1 source boundary timestamp rejected".to_string())?,
        participant_evaluations: vec![
            evaluation_for(
                MomentumRawFeatureRoleV4::LearnedRawLogistic,
                raw_status,
                &raw_metric,
            )?,
            evaluation_for(
                MomentumRawFeatureRoleV4::LearnedInteractionLogistic,
                interaction_status,
                &interaction_metric,
            )?,
            evaluation_for(
                MomentumRawFeatureRoleV4::ConstantBenchmark,
                constant_status,
                &constant_metric,
            )?,
        ],
        interaction_contribution,
    })
}

#[derive(Clone, PartialEq, Message)]
struct FieldProtobufV4 {
    #[prost(string, tag = "1")]
    key: String,
    #[prost(string, tag = "2")]
    value: String,
}

#[derive(Clone, PartialEq, Message)]
struct ClosureProtobufV4 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4>,
}

#[derive(Clone, PartialEq, Message)]
struct SplitProtobufV4 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4>,
}

#[derive(Clone, PartialEq, Message)]
struct ValidationYieldAuditProtobufV4 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4>,
}

#[derive(Clone, PartialEq, Message)]
struct ConfigProtobufV4 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4>,
}

#[derive(Clone, PartialEq, Message)]
struct RegistrationProtobufV4 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4>,
    #[prost(message, repeated, tag = "2")]
    participants: Vec<ConfigProtobufV4>,
}

#[derive(Clone, PartialEq, Message)]
struct ParticipantProtobufV4 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4>,
}

#[derive(Clone, PartialEq, Message)]
struct ReceiptProtobufV4 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4>,
}

#[derive(Clone, PartialEq, Message)]
struct ContributionProtobufV4 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4>,
}

#[derive(Clone, PartialEq, Message)]
struct FamilyProtobufV4 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4>,
    #[prost(message, repeated, tag = "2")]
    participants: Vec<ParticipantProtobufV4>,
    #[prost(message, repeated, tag = "3")]
    receipts: Vec<ReceiptProtobufV4>,
    #[prost(message, optional, tag = "4")]
    contribution: Option<ContributionProtobufV4>,
}

#[derive(Clone, PartialEq, Message)]
struct DecisionProtobufV4 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4>,
}

#[derive(Clone, PartialEq, Message)]
struct RosterProtobufV4 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4>,
    #[prost(string, repeated, tag = "2")]
    learned: Vec<String>,
    #[prost(string, repeated, tag = "3")]
    benchmarks: Vec<String>,
    #[prost(string, repeated, tag = "4")]
    duplicates: Vec<String>,
    #[prost(string, repeated, tag = "5")]
    rejected: Vec<String>,
}

#[derive(Clone, PartialEq, Message)]
struct EvaluationProtobufV4 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4>,
    #[prost(string, repeated, tag = "2")]
    receipts: Vec<String>,
    #[prost(string, repeated, tag = "3")]
    protected_registrations: Vec<String>,
    #[prost(uint64, repeated, tag = "4")]
    protected_timestamps: Vec<u64>,
    #[prost(string, repeated, tag = "5")]
    prior_validation_identities: Vec<String>,
}

#[derive(Clone, PartialEq, Message)]
struct JournalProtobufV4 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4>,
}

fn field(key: &str, value: impl ToString) -> FieldProtobufV4 {
    FieldProtobufV4 {
        key: key.to_string(),
        value: value.to_string(),
    }
}

fn field_map(fields: Vec<FieldProtobufV4>) -> Result<BTreeMap<String, String>, String> {
    let mut values = BTreeMap::new();
    for item in fields {
        if item.key.is_empty() || values.insert(item.key, item.value).is_some() {
            return Err("V4 Protobuf field identity rejected".to_string());
        }
    }
    Ok(values)
}

fn take(map: &mut BTreeMap<String, String>, key: &str) -> Result<String, String> {
    map.remove(key)
        .ok_or_else(|| format!("V4 Protobuf field missing: {key}"))
}

fn take_bool(map: &mut BTreeMap<String, String>, key: &str) -> Result<bool, String> {
    take(map, key)?
        .parse()
        .map_err(|_| format!("V4 Protobuf bool rejected: {key}"))
}

fn take_usize(map: &mut BTreeMap<String, String>, key: &str) -> Result<usize, String> {
    take(map, key)?
        .parse()
        .map_err(|_| format!("V4 Protobuf usize rejected: {key}"))
}

fn take_u64(map: &mut BTreeMap<String, String>, key: &str) -> Result<u64, String> {
    take(map, key)?
        .parse()
        .map_err(|_| format!("V4 Protobuf u64 rejected: {key}"))
}

fn take_u32(map: &mut BTreeMap<String, String>, key: &str) -> Result<u32, String> {
    take(map, key)?
        .parse()
        .map_err(|_| format!("V4 Protobuf u32 rejected: {key}"))
}

fn take_optional(map: &mut BTreeMap<String, String>, key: &str) -> Result<Option<String>, String> {
    let value = take(map, key)?;
    Ok((!value.is_empty()).then_some(value))
}

fn finish_fields(map: BTreeMap<String, String>) -> Result<(), String> {
    if map.is_empty() {
        Ok(())
    } else {
        Err("V4 unexpected Protobuf fields rejected".to_string())
    }
}

fn encode_message_v4(value: &impl Message) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    value
        .encode(&mut bytes)
        .map_err(|_| "V4 Protobuf encode failed".to_string())?;
    Ok(bytes)
}

fn parse_closure_decision(value: &str) -> Result<MomentumFrozenMambaClosureDecisionV4, String> {
    match value {
        "ClosedForCurrentEvidenceAndPolicy" => {
            Ok(MomentumFrozenMambaClosureDecisionV4::ClosedForCurrentEvidenceAndPolicy)
        }
        "ClosureIntegrityFailure" => {
            Ok(MomentumFrozenMambaClosureDecisionV4::ClosureIntegrityFailure)
        }
        _ => Err("V4 closure decision rejected".to_string()),
    }
}

fn parse_model_kind(value: &str) -> Result<MomentumRawFeatureModelKindV4, String> {
    match value {
        "RawFeatureLogistic" => Ok(MomentumRawFeatureModelKindV4::RawFeatureLogistic),
        "RawFeatureInteractionLogistic" => {
            Ok(MomentumRawFeatureModelKindV4::RawFeatureInteractionLogistic)
        }
        "TrainingPrevalenceConstant" => {
            Ok(MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant)
        }
        _ => Err("V4 model kind rejected".to_string()),
    }
}

fn parse_role(value: &str) -> Result<MomentumRawFeatureRoleV4, String> {
    match value {
        "LearnedRawLogistic" => Ok(MomentumRawFeatureRoleV4::LearnedRawLogistic),
        "LearnedInteractionLogistic" => Ok(MomentumRawFeatureRoleV4::LearnedInteractionLogistic),
        "ConstantBenchmark" => Ok(MomentumRawFeatureRoleV4::ConstantBenchmark),
        _ => Err("V4 participant role rejected".to_string()),
    }
}

fn parse_qualification(value: &str) -> Result<MomentumRawFeatureQualificationStatusV4, String> {
    match value {
        "QualifiedLearned" => Ok(MomentumRawFeatureQualificationStatusV4::QualifiedLearned),
        "QualifiedLinearEquivalent" => {
            Ok(MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent)
        }
        "BenchmarkQualified" => Ok(MomentumRawFeatureQualificationStatusV4::BenchmarkQualified),
        "RejectedInsufficientValidation" => {
            Ok(MomentumRawFeatureQualificationStatusV4::RejectedInsufficientValidation)
        }
        "RejectedProbabilityCollapse" => {
            Ok(MomentumRawFeatureQualificationStatusV4::RejectedProbabilityCollapse)
        }
        "RejectedNumericalFailure" => {
            Ok(MomentumRawFeatureQualificationStatusV4::RejectedNumericalFailure)
        }
        "RejectedFeatureIntegrity" => {
            Ok(MomentumRawFeatureQualificationStatusV4::RejectedFeatureIntegrity)
        }
        "RejectedPolicyInvariant" => {
            Ok(MomentumRawFeatureQualificationStatusV4::RejectedPolicyInvariant)
        }
        _ => Err("V4 qualification status rejected".to_string()),
    }
}

fn parse_contribution(value: &str) -> Result<InteractionContributionStatusV4, String> {
    match value {
        "MaterialInteractionContribution" => {
            Ok(InteractionContributionStatusV4::MaterialInteractionContribution)
        }
        "DetectableButBelowPolicy" => Ok(InteractionContributionStatusV4::DetectableButBelowPolicy),
        "LinearEquivalent" => Ok(InteractionContributionStatusV4::LinearEquivalent),
        "Invalid" => Ok(InteractionContributionStatusV4::Invalid),
        _ => Err("V4 contribution status rejected".to_string()),
    }
}

fn parse_path_decision(value: &str) -> Result<MomentumRawFeaturePathDecisionV4, String> {
    match value {
        "RawFeatureLearnedPathViable" => {
            Ok(MomentumRawFeaturePathDecisionV4::RawFeatureLearnedPathViable)
        }
        "OnlyLinearRawPathViable" => Ok(MomentumRawFeaturePathDecisionV4::OnlyLinearRawPathViable),
        "NoQualifiedRawFeatureLearner" => {
            Ok(MomentumRawFeaturePathDecisionV4::NoQualifiedRawFeatureLearner)
        }
        "InsufficientFreshValidation" => {
            Ok(MomentumRawFeaturePathDecisionV4::InsufficientFreshValidation)
        }
        "TechnicalFailure" => Ok(MomentumRawFeaturePathDecisionV4::TechnicalFailure),
        _ => Err("V4 path decision rejected".to_string()),
    }
}

fn parse_roster_status(value: &str) -> Result<MomentumRawFeatureRosterStatusV4, String> {
    match value {
        "Ready" => Ok(MomentumRawFeatureRosterStatusV4::Ready),
        "QualificationEvidenceInsufficient" => {
            Ok(MomentumRawFeatureRosterStatusV4::QualificationEvidenceInsufficient)
        }
        "NoQualifiedLearnedParticipant" => {
            Ok(MomentumRawFeatureRosterStatusV4::NoQualifiedLearnedParticipant)
        }
        "BenchmarkUnavailable" => Ok(MomentumRawFeatureRosterStatusV4::BenchmarkUnavailable),
        "SemanticDuplicateOnly" => Ok(MomentumRawFeatureRosterStatusV4::SemanticDuplicateOnly),
        "IntegrityFailure" => Ok(MomentumRawFeatureRosterStatusV4::IntegrityFailure),
        _ => Err("V4 roster status rejected".to_string()),
    }
}

fn parse_execution_status(value: &str) -> Result<MomentumRawFeatureExecutionStatusV4, String> {
    match value {
        "Planned" => Ok(MomentumRawFeatureExecutionStatusV4::Planned),
        "Executed" => Ok(MomentumRawFeatureExecutionStatusV4::Executed),
        "AlreadyExecuted" => Ok(MomentumRawFeatureExecutionStatusV4::AlreadyExecuted),
        "InsufficientFreshValidation" => {
            Ok(MomentumRawFeatureExecutionStatusV4::InsufficientFreshValidation)
        }
        "TechnicalFailure" => Ok(MomentumRawFeatureExecutionStatusV4::TechnicalFailure),
        _ => Err("V4 execution status rejected".to_string()),
    }
}

fn closure_to_pb(value: &MomentumFrozenMambaPathClosureV4) -> ClosureProtobufV4 {
    ClosureProtobufV4 {
        fields: vec![
            field("closure_version", &value.closure_version),
            field("agent_id", &value.agent_id),
            field("source_snapshot_digest", &value.source_snapshot_digest),
            field("canonical_intent_digest", &value.canonical_intent_digest),
            field("canonical_view_digest", &value.canonical_view_digest),
            field("v1_family_digest", &value.v1_family_digest),
            field("v2_family_digest", &value.v2_family_digest),
            field("v3_family_digest", &value.v3_family_digest),
            field("v3_route_decision_digest", &value.v3_route_decision_digest),
            field("frozen_encoder_digest", &value.frozen_encoder_digest),
            field("feature_policy_digest", &value.feature_policy_digest),
            field("label_policy_digest", &value.label_policy_digest),
            field(
                "genuine_mamba_qualified_count",
                value.genuine_mamba_qualified_count,
            ),
            field(
                "head_only_repair_forbidden",
                value.head_only_repair_forbidden,
            ),
            field(
                "frozen_representation_sweep_forbidden",
                value.frozen_representation_sweep_forbidden,
            ),
            field(
                "frozen_mamba_parent_use_forbidden",
                value.frozen_mamba_parent_use_forbidden,
            ),
            field(
                "reopening_requires_new_encoder_identity",
                value.reopening_requires_new_encoder_identity,
            ),
            field(
                "reopening_requires_new_evidence_identity",
                value.reopening_requires_new_evidence_identity,
            ),
            field(
                "reopening_requires_new_preregistration",
                value.reopening_requires_new_preregistration,
            ),
            field("decision", format!("{:?}", value.decision)),
            field("closure_digest", &value.closure_digest),
        ],
    }
}

fn closure_from_pb(value: ClosureProtobufV4) -> Result<MomentumFrozenMambaPathClosureV4, String> {
    let mut f = field_map(value.fields)?;
    let result = MomentumFrozenMambaPathClosureV4 {
        closure_version: take(&mut f, "closure_version")?,
        agent_id: take(&mut f, "agent_id")?,
        source_snapshot_digest: take(&mut f, "source_snapshot_digest")?,
        canonical_intent_digest: take(&mut f, "canonical_intent_digest")?,
        canonical_view_digest: take(&mut f, "canonical_view_digest")?,
        v1_family_digest: take(&mut f, "v1_family_digest")?,
        v2_family_digest: take(&mut f, "v2_family_digest")?,
        v3_family_digest: take(&mut f, "v3_family_digest")?,
        v3_route_decision_digest: take(&mut f, "v3_route_decision_digest")?,
        frozen_encoder_digest: take(&mut f, "frozen_encoder_digest")?,
        feature_policy_digest: take(&mut f, "feature_policy_digest")?,
        label_policy_digest: take(&mut f, "label_policy_digest")?,
        genuine_mamba_qualified_count: take_usize(&mut f, "genuine_mamba_qualified_count")?,
        head_only_repair_forbidden: take_bool(&mut f, "head_only_repair_forbidden")?,
        frozen_representation_sweep_forbidden: take_bool(
            &mut f,
            "frozen_representation_sweep_forbidden",
        )?,
        frozen_mamba_parent_use_forbidden: take_bool(&mut f, "frozen_mamba_parent_use_forbidden")?,
        reopening_requires_new_encoder_identity: take_bool(
            &mut f,
            "reopening_requires_new_encoder_identity",
        )?,
        reopening_requires_new_evidence_identity: take_bool(
            &mut f,
            "reopening_requires_new_evidence_identity",
        )?,
        reopening_requires_new_preregistration: take_bool(
            &mut f,
            "reopening_requires_new_preregistration",
        )?,
        decision: parse_closure_decision(&take(&mut f, "decision")?)?,
        closure_digest: take(&mut f, "closure_digest")?,
    };
    finish_fields(f)?;
    validate_closure_v4(&result)?;
    Ok(result)
}

fn split_to_pb(value: &MomentumRawFeatureSplitV4) -> SplitProtobufV4 {
    SplitProtobufV4 {
        fields: vec![
            field("split_version", &value.split_version),
            field("source_snapshot_digest", &value.source_snapshot_digest),
            field("v3_split_digest", &value.v3_split_digest),
            field("v3_route_decision_digest", &value.v3_route_decision_digest),
            field("training_start", value.training_range.start),
            field("training_end", value.training_range.end),
            field("purge_start", value.purge_range.start),
            field("purge_end", value.purge_range.end),
            field("validation_start", value.fresh_validation_range.start),
            field("validation_end", value.fresh_validation_range.end),
            field("final_start", value.final_untouched_range.start),
            field("final_end", value.final_untouched_range.end),
            field(
                "minimum_validation_samples",
                value.minimum_validation_samples,
            ),
            field(
                "minimum_final_reserve_samples",
                value.minimum_final_reserve_samples,
            ),
            field(
                "prior_qualification_overlap_count",
                value.prior_qualification_overlap_count,
            ),
            field("prospective_overlap_count", value.prospective_overlap_count),
            field(
                "historical_test_overlap_count",
                value.historical_test_overlap_count,
            ),
            field(
                "future_evaluation_overlap_count",
                value.future_evaluation_overlap_count,
            ),
            field("split_digest", &value.split_digest),
        ],
    }
}

fn split_from_pb(value: SplitProtobufV4) -> Result<MomentumRawFeatureSplitV4, String> {
    let mut f = field_map(value.fields)?;
    let result = MomentumRawFeatureSplitV4 {
        split_version: take(&mut f, "split_version")?,
        source_snapshot_digest: take(&mut f, "source_snapshot_digest")?,
        v3_split_digest: take(&mut f, "v3_split_digest")?,
        v3_route_decision_digest: take(&mut f, "v3_route_decision_digest")?,
        training_range: IndexRangeV0 {
            start: take_usize(&mut f, "training_start")?,
            end: take_usize(&mut f, "training_end")?,
        },
        purge_range: IndexRangeV0 {
            start: take_usize(&mut f, "purge_start")?,
            end: take_usize(&mut f, "purge_end")?,
        },
        fresh_validation_range: IndexRangeV0 {
            start: take_usize(&mut f, "validation_start")?,
            end: take_usize(&mut f, "validation_end")?,
        },
        final_untouched_range: IndexRangeV0 {
            start: take_usize(&mut f, "final_start")?,
            end: take_usize(&mut f, "final_end")?,
        },
        minimum_validation_samples: take_usize(&mut f, "minimum_validation_samples")?,
        minimum_final_reserve_samples: take_usize(&mut f, "minimum_final_reserve_samples")?,
        prior_qualification_overlap_count: take_usize(&mut f, "prior_qualification_overlap_count")?,
        prospective_overlap_count: take_usize(&mut f, "prospective_overlap_count")?,
        historical_test_overlap_count: take_usize(&mut f, "historical_test_overlap_count")?,
        future_evaluation_overlap_count: take_usize(&mut f, "future_evaluation_overlap_count")?,
        split_digest: take(&mut f, "split_digest")?,
    };
    finish_fields(f)?;
    validate_split_v4(&result)?;
    Ok(result)
}

fn validation_yield_audit_to_pb(
    value: &MomentumValidationYieldAuditV4,
) -> ValidationYieldAuditProtobufV4 {
    ValidationYieldAuditProtobufV4 {
        fields: vec![
            field("audit_version", &value.audit_version),
            field("source_snapshot_digest", &value.source_snapshot_digest),
            field("label_policy_digest", &value.label_policy_digest),
            field("validation_start", value.validation_index_range.start),
            field("validation_end", value.validation_index_range.end),
            field("validation_index_count", value.validation_index_count),
            field(
                "minimum_required_valid_samples",
                value.minimum_required_valid_samples,
            ),
            field(
                "valid_labelled_sample_count",
                value.valid_labelled_sample_count,
            ),
            field("neutral_excluded_count", value.neutral_excluded_count),
            field("horizon_unavailable_count", value.horizon_unavailable_count),
            field("feature_unavailable_count", value.feature_unavailable_count),
            field(
                "substantive_qualification_possible",
                value.substantive_qualification_possible,
            ),
            field("audit_digest", &value.audit_digest),
        ],
    }
}

fn validation_yield_audit_from_pb(
    value: ValidationYieldAuditProtobufV4,
) -> Result<MomentumValidationYieldAuditV4, String> {
    let mut f = field_map(value.fields)?;
    let result = MomentumValidationYieldAuditV4 {
        audit_version: take(&mut f, "audit_version")?,
        source_snapshot_digest: take(&mut f, "source_snapshot_digest")?,
        label_policy_digest: take(&mut f, "label_policy_digest")?,
        validation_index_range: IndexRangeV0 {
            start: take_usize(&mut f, "validation_start")?,
            end: take_usize(&mut f, "validation_end")?,
        },
        validation_index_count: take_usize(&mut f, "validation_index_count")?,
        minimum_required_valid_samples: take_usize(&mut f, "minimum_required_valid_samples")?,
        valid_labelled_sample_count: take_usize(&mut f, "valid_labelled_sample_count")?,
        neutral_excluded_count: take_usize(&mut f, "neutral_excluded_count")?,
        horizon_unavailable_count: take_usize(&mut f, "horizon_unavailable_count")?,
        feature_unavailable_count: take_usize(&mut f, "feature_unavailable_count")?,
        substantive_qualification_possible: take_bool(
            &mut f,
            "substantive_qualification_possible",
        )?,
        audit_digest: take(&mut f, "audit_digest")?,
    };
    finish_fields(f)?;
    validate_validation_yield_audit_v4(&result)?;
    Ok(result)
}

fn config_to_pb(value: &MomentumRawFeatureParticipantConfigV4) -> ConfigProtobufV4 {
    ConfigProtobufV4 {
        fields: vec![
            field("participant_id", &value.participant_id),
            field("model_kind", format!("{:?}", value.model_kind)),
            field("feature_policy_digest", &value.feature_policy_digest),
            field("label_policy_digest", &value.label_policy_digest),
            field(
                "input_feature_schema_digest",
                &value.input_feature_schema_digest,
            ),
            field("learning_rate_bits", value.learning_rate_bits),
            field("l2_regularization_bits", value.l2_regularization_bits),
            field("maximum_epochs", value.maximum_epochs),
            field("initialization_seed", value.initialization_seed),
            field("fresh_initialization", value.fresh_initialization),
            field("training_only_normalizer", value.training_only_normalizer),
            field("config_digest", &value.config_digest),
        ],
    }
}

fn config_from_pb(
    value: ConfigProtobufV4,
) -> Result<MomentumRawFeatureParticipantConfigV4, String> {
    let mut f = field_map(value.fields)?;
    let result = MomentumRawFeatureParticipantConfigV4 {
        participant_id: take(&mut f, "participant_id")?,
        model_kind: parse_model_kind(&take(&mut f, "model_kind")?)?,
        feature_policy_digest: take(&mut f, "feature_policy_digest")?,
        label_policy_digest: take(&mut f, "label_policy_digest")?,
        input_feature_schema_digest: take(&mut f, "input_feature_schema_digest")?,
        learning_rate_bits: take_u32(&mut f, "learning_rate_bits")?,
        l2_regularization_bits: take_u32(&mut f, "l2_regularization_bits")?,
        maximum_epochs: take_usize(&mut f, "maximum_epochs")?,
        initialization_seed: take_u64(&mut f, "initialization_seed")?,
        fresh_initialization: take_bool(&mut f, "fresh_initialization")?,
        training_only_normalizer: take_bool(&mut f, "training_only_normalizer")?,
        config_digest: take(&mut f, "config_digest")?,
    };
    finish_fields(f)?;
    validate_config_v4(&result)?;
    Ok(result)
}

fn registration_to_pb(value: &MomentumRawFeatureRegistrationV4) -> RegistrationProtobufV4 {
    RegistrationProtobufV4 {
        fields: vec![
            field("registration_version", &value.registration_version),
            field("agent_id", &value.agent_id),
            field("source_snapshot_digest", &value.source_snapshot_digest),
            field("canonical_intent_digest", &value.canonical_intent_digest),
            field("canonical_view_digest", &value.canonical_view_digest),
            field(
                "frozen_mamba_closure_digest",
                &value.frozen_mamba_closure_digest,
            ),
            field("split_digest", &value.split_digest),
            field(
                "maximum_learned_participants",
                value.maximum_learned_participants,
            ),
            field(
                "interaction_contribution_policy_digest",
                &value.interaction_contribution_policy_digest,
            ),
            field("fresh_validation_hidden", value.fresh_validation_hidden),
            field("final_reserve_forbidden", value.final_reserve_forbidden),
            field("historical_test_forbidden", value.historical_test_forbidden),
            field(
                "future_evaluation_forbidden",
                value.future_evaluation_forbidden,
            ),
            field(
                "winner_selection_forbidden",
                value.winner_selection_forbidden,
            ),
            field(
                "active_promotion_forbidden",
                value.active_promotion_forbidden,
            ),
            field(
                "reward_application_forbidden",
                value.reward_application_forbidden,
            ),
            field("registration_digest", &value.registration_digest),
        ],
        participants: value.participants.iter().map(config_to_pb).collect(),
    }
}

fn registration_from_pb(
    value: RegistrationProtobufV4,
) -> Result<MomentumRawFeatureRegistrationV4, String> {
    let participants = value
        .participants
        .into_iter()
        .map(config_from_pb)
        .collect::<Result<Vec<_>, _>>()?;
    let mut f = field_map(value.fields)?;
    let result = MomentumRawFeatureRegistrationV4 {
        registration_version: take(&mut f, "registration_version")?,
        agent_id: take(&mut f, "agent_id")?,
        source_snapshot_digest: take(&mut f, "source_snapshot_digest")?,
        canonical_intent_digest: take(&mut f, "canonical_intent_digest")?,
        canonical_view_digest: take(&mut f, "canonical_view_digest")?,
        frozen_mamba_closure_digest: take(&mut f, "frozen_mamba_closure_digest")?,
        split_digest: take(&mut f, "split_digest")?,
        participants,
        maximum_learned_participants: take_usize(&mut f, "maximum_learned_participants")?,
        interaction_contribution_policy_digest: take(
            &mut f,
            "interaction_contribution_policy_digest",
        )?,
        fresh_validation_hidden: take_bool(&mut f, "fresh_validation_hidden")?,
        final_reserve_forbidden: take_bool(&mut f, "final_reserve_forbidden")?,
        historical_test_forbidden: take_bool(&mut f, "historical_test_forbidden")?,
        future_evaluation_forbidden: take_bool(&mut f, "future_evaluation_forbidden")?,
        winner_selection_forbidden: take_bool(&mut f, "winner_selection_forbidden")?,
        active_promotion_forbidden: take_bool(&mut f, "active_promotion_forbidden")?,
        reward_application_forbidden: take_bool(&mut f, "reward_application_forbidden")?,
        registration_digest: take(&mut f, "registration_digest")?,
    };
    finish_fields(f)?;
    validate_registration_v4(&result)?;
    Ok(result)
}

fn participant_to_pb(value: &FrozenCandidateParticipantV4) -> ParticipantProtobufV4 {
    ParticipantProtobufV4 {
        fields: vec![
            field("participant_version", &value.participant_version),
            field("participant_id", &value.participant_id),
            field("participant_role", format!("{:?}", value.participant_role)),
            field("model_kind", format!("{:?}", value.model_kind)),
            field("config_digest", &value.config_digest),
            field("source_snapshot_digest", &value.source_snapshot_digest),
            field("training_range_digest", &value.training_range_digest),
            field(
                "fresh_validation_range_digest",
                &value.fresh_validation_range_digest,
            ),
            field(
                "validation_timestamp_digest",
                &value.validation_timestamp_digest,
            ),
            field(
                "input_feature_schema_digest",
                &value.input_feature_schema_digest,
            ),
            field("model_artifact_digest", &value.model_artifact_digest),
            field("parameter_digest", &value.parameter_digest),
            field("normalizer_digest", &value.normalizer_digest),
            field("training_identity_digest", &value.training_identity_digest),
            field("fresh_initialization", value.fresh_initialization),
            field("prior_parameters_reused", value.prior_parameters_reused),
            field("prior_normalizer_reused", value.prior_normalizer_reused),
            field("prior_predictions_reused", value.prior_predictions_reused),
            field(
                "validation_parameter_updates",
                value.validation_parameter_updates,
            ),
            field(
                "deployment_status",
                format!("{:?}", value.deployment_status),
            ),
            field("participant_digest", &value.participant_digest),
        ],
    }
}

fn participant_from_pb(
    value: ParticipantProtobufV4,
) -> Result<FrozenCandidateParticipantV4, String> {
    let mut f = field_map(value.fields)?;
    let deployment = take(&mut f, "deployment_status")?;
    if deployment != "ShadowOnly" {
        return Err("V4 deployment status rejected".to_string());
    }
    let result = FrozenCandidateParticipantV4 {
        participant_version: take(&mut f, "participant_version")?,
        participant_id: take(&mut f, "participant_id")?,
        participant_role: parse_role(&take(&mut f, "participant_role")?)?,
        model_kind: parse_model_kind(&take(&mut f, "model_kind")?)?,
        config_digest: take(&mut f, "config_digest")?,
        source_snapshot_digest: take(&mut f, "source_snapshot_digest")?,
        training_range_digest: take(&mut f, "training_range_digest")?,
        fresh_validation_range_digest: take(&mut f, "fresh_validation_range_digest")?,
        validation_timestamp_digest: take(&mut f, "validation_timestamp_digest")?,
        input_feature_schema_digest: take(&mut f, "input_feature_schema_digest")?,
        model_artifact_digest: take(&mut f, "model_artifact_digest")?,
        parameter_digest: take(&mut f, "parameter_digest")?,
        normalizer_digest: take(&mut f, "normalizer_digest")?,
        training_identity_digest: take(&mut f, "training_identity_digest")?,
        fresh_initialization: take_bool(&mut f, "fresh_initialization")?,
        prior_parameters_reused: take_bool(&mut f, "prior_parameters_reused")?,
        prior_normalizer_reused: take_bool(&mut f, "prior_normalizer_reused")?,
        prior_predictions_reused: take_bool(&mut f, "prior_predictions_reused")?,
        validation_parameter_updates: take_usize(&mut f, "validation_parameter_updates")?,
        deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
        participant_digest: take(&mut f, "participant_digest")?,
    };
    finish_fields(f)?;
    validate_participant_v4(&result)?;
    Ok(result)
}

fn receipt_to_pb(value: &MomentumRawFeatureQualificationReceiptV4) -> ReceiptProtobufV4 {
    ReceiptProtobufV4 {
        fields: vec![
            field("receipt_version", &value.receipt_version),
            field("participant_id", &value.participant_id),
            field("participant_role", format!("{:?}", value.participant_role)),
            field("participant_digest", &value.participant_digest),
            field(
                "fresh_validation_range_digest",
                &value.fresh_validation_range_digest,
            ),
            field(
                "qualification_policy_digest",
                &value.qualification_policy_digest,
            ),
            field("private_metric_digest", &value.private_metric_digest),
            field(
                "interaction_contribution_audit_digest",
                value
                    .interaction_contribution_audit_digest
                    .as_deref()
                    .unwrap_or(""),
            ),
            field("status", format!("{:?}", value.status)),
            field(
                "validation_parameter_updates",
                value.validation_parameter_updates,
            ),
            field("final_reserve_reads", value.final_reserve_reads),
            field("historical_test_reads", value.historical_test_reads),
            field("future_evaluation_reads", value.future_evaluation_reads),
            field("receipt_digest", &value.receipt_digest),
        ],
    }
}

fn receipt_from_pb(
    value: ReceiptProtobufV4,
) -> Result<MomentumRawFeatureQualificationReceiptV4, String> {
    let mut f = field_map(value.fields)?;
    let result = MomentumRawFeatureQualificationReceiptV4 {
        receipt_version: take(&mut f, "receipt_version")?,
        participant_id: take(&mut f, "participant_id")?,
        participant_role: parse_role(&take(&mut f, "participant_role")?)?,
        participant_digest: take(&mut f, "participant_digest")?,
        fresh_validation_range_digest: take(&mut f, "fresh_validation_range_digest")?,
        qualification_policy_digest: take(&mut f, "qualification_policy_digest")?,
        private_metric_digest: take(&mut f, "private_metric_digest")?,
        interaction_contribution_audit_digest: take_optional(
            &mut f,
            "interaction_contribution_audit_digest",
        )?,
        status: parse_qualification(&take(&mut f, "status")?)?,
        validation_parameter_updates: take_usize(&mut f, "validation_parameter_updates")?,
        final_reserve_reads: take_usize(&mut f, "final_reserve_reads")?,
        historical_test_reads: take_usize(&mut f, "historical_test_reads")?,
        future_evaluation_reads: take_usize(&mut f, "future_evaluation_reads")?,
        receipt_digest: take(&mut f, "receipt_digest")?,
    };
    finish_fields(f)?;
    validate_receipt_v4(&result)?;
    Ok(result)
}

fn contribution_to_pb(value: &MomentumInteractionContributionAuditV4) -> ContributionProtobufV4 {
    ContributionProtobufV4 {
        fields: vec![
            field("participant_digest", &value.participant_digest),
            field(
                "original_feature_parameter_digest",
                &value.original_feature_parameter_digest,
            ),
            field(
                "squared_feature_parameter_digest",
                &value.squared_feature_parameter_digest,
            ),
            field(
                "pairwise_feature_parameter_digest",
                &value.pairwise_feature_parameter_digest,
            ),
            field("original_block_nonzero", value.original_block_nonzero),
            field("nonlinear_blocks_nonzero", value.nonlinear_blocks_nonzero),
            field("full_prediction_digest", &value.full_prediction_digest),
            field(
                "nonlinear_ablated_prediction_digest",
                &value.nonlinear_ablated_prediction_digest,
            ),
            field(
                "contribution_policy_digest",
                &value.contribution_policy_digest,
            ),
            field(
                "contribution_status",
                format!("{:?}", value.contribution_status),
            ),
            field("audit_digest", &value.audit_digest),
        ],
    }
}

fn contribution_from_pb(
    value: ContributionProtobufV4,
) -> Result<MomentumInteractionContributionAuditV4, String> {
    let mut f = field_map(value.fields)?;
    let result = MomentumInteractionContributionAuditV4 {
        participant_digest: take(&mut f, "participant_digest")?,
        original_feature_parameter_digest: take(&mut f, "original_feature_parameter_digest")?,
        squared_feature_parameter_digest: take(&mut f, "squared_feature_parameter_digest")?,
        pairwise_feature_parameter_digest: take(&mut f, "pairwise_feature_parameter_digest")?,
        original_block_nonzero: take_bool(&mut f, "original_block_nonzero")?,
        nonlinear_blocks_nonzero: take_bool(&mut f, "nonlinear_blocks_nonzero")?,
        full_prediction_digest: take(&mut f, "full_prediction_digest")?,
        nonlinear_ablated_prediction_digest: take(&mut f, "nonlinear_ablated_prediction_digest")?,
        contribution_policy_digest: take(&mut f, "contribution_policy_digest")?,
        contribution_status: parse_contribution(&take(&mut f, "contribution_status")?)?,
        audit_digest: take(&mut f, "audit_digest")?,
    };
    finish_fields(f)?;
    validate_contribution_v4(&result)?;
    Ok(result)
}

fn family_to_pb(value: &MomentumRawFeatureFamilyV4) -> FamilyProtobufV4 {
    FamilyProtobufV4 {
        fields: vec![
            field("family_version", &value.family_version),
            field("agent_id", &value.agent_id),
            field("source_snapshot_digest", &value.source_snapshot_digest),
            field("canonical_view_digest", &value.canonical_view_digest),
            field(
                "frozen_mamba_closure_digest",
                &value.frozen_mamba_closure_digest,
            ),
            field("split_digest", &value.split_digest),
            field("registration_digest", &value.registration_digest),
            field("qualified_learned_count", value.qualified_learned_count),
            field("qualified_benchmark_count", value.qualified_benchmark_count),
            field("winner_selected", value.winner_selected),
            field("final_reserve_accessed", value.final_reserve_accessed),
            field(
                "eligible_for_active_committee",
                value.eligible_for_active_committee,
            ),
            field("eligible_for_promotion", value.eligible_for_promotion),
            field("eligible_for_reward", value.eligible_for_reward),
            field("family_digest", &value.family_digest),
        ],
        participants: value.participants.iter().map(participant_to_pb).collect(),
        receipts: value
            .qualification_receipts
            .iter()
            .map(receipt_to_pb)
            .collect(),
        contribution: value
            .interaction_contribution_audit
            .as_ref()
            .map(contribution_to_pb),
    }
}

fn family_from_pb(value: FamilyProtobufV4) -> Result<MomentumRawFeatureFamilyV4, String> {
    let participants = value
        .participants
        .into_iter()
        .map(participant_from_pb)
        .collect::<Result<Vec<_>, _>>()?;
    let qualification_receipts = value
        .receipts
        .into_iter()
        .map(receipt_from_pb)
        .collect::<Result<Vec<_>, _>>()?;
    let interaction_contribution_audit =
        value.contribution.map(contribution_from_pb).transpose()?;
    let mut f = field_map(value.fields)?;
    let result = MomentumRawFeatureFamilyV4 {
        family_version: take(&mut f, "family_version")?,
        agent_id: take(&mut f, "agent_id")?,
        source_snapshot_digest: take(&mut f, "source_snapshot_digest")?,
        canonical_view_digest: take(&mut f, "canonical_view_digest")?,
        frozen_mamba_closure_digest: take(&mut f, "frozen_mamba_closure_digest")?,
        split_digest: take(&mut f, "split_digest")?,
        registration_digest: take(&mut f, "registration_digest")?,
        participants,
        qualification_receipts,
        interaction_contribution_audit,
        qualified_learned_count: take_usize(&mut f, "qualified_learned_count")?,
        qualified_benchmark_count: take_usize(&mut f, "qualified_benchmark_count")?,
        winner_selected: take_bool(&mut f, "winner_selected")?,
        final_reserve_accessed: take_bool(&mut f, "final_reserve_accessed")?,
        eligible_for_active_committee: take_bool(&mut f, "eligible_for_active_committee")?,
        eligible_for_promotion: take_bool(&mut f, "eligible_for_promotion")?,
        eligible_for_reward: take_bool(&mut f, "eligible_for_reward")?,
        family_digest: take(&mut f, "family_digest")?,
    };
    finish_fields(f)?;
    validate_family_v4(&result)?;
    Ok(result)
}

fn decision_to_pb(value: &MomentumRawFeaturePathDecisionArtifactV4) -> DecisionProtobufV4 {
    DecisionProtobufV4 {
        fields: vec![
            field("decision_version", &value.decision_version),
            field("family_digest", &value.family_digest),
            field("qualified_raw_logistic", value.qualified_raw_logistic),
            field(
                "qualified_material_interaction",
                value.qualified_material_interaction,
            ),
            field("decision", format!("{:?}", value.decision)),
            field("decision_digest", &value.decision_digest),
        ],
    }
}

fn decision_from_pb_unvalidated(
    value: DecisionProtobufV4,
) -> Result<MomentumRawFeaturePathDecisionArtifactV4, String> {
    let mut f = field_map(value.fields)?;
    let result = MomentumRawFeaturePathDecisionArtifactV4 {
        decision_version: take(&mut f, "decision_version")?,
        family_digest: take(&mut f, "family_digest")?,
        qualified_raw_logistic: take_bool(&mut f, "qualified_raw_logistic")?,
        qualified_material_interaction: take_bool(&mut f, "qualified_material_interaction")?,
        decision: parse_path_decision(&take(&mut f, "decision")?)?,
        decision_digest: take(&mut f, "decision_digest")?,
    };
    finish_fields(f)?;
    Ok(result)
}

fn decision_from_pb(
    value: DecisionProtobufV4,
    family: &MomentumRawFeatureFamilyV4,
) -> Result<MomentumRawFeaturePathDecisionArtifactV4, String> {
    let result = decision_from_pb_unvalidated(value)?;
    validate_decision_v4(&result, family)?;
    Ok(result)
}

fn roster_to_pb(value: &MomentumRawFeatureFutureRosterV4) -> RosterProtobufV4 {
    RosterProtobufV4 {
        fields: vec![
            field("roster_version", &value.roster_version),
            field("family_digest", &value.family_digest),
            field("inclusion_policy_digest", &value.inclusion_policy_digest),
            field("status", format!("{:?}", value.status)),
            field("roster_digest", &value.roster_digest),
        ],
        learned: value.learned_participant_digests.clone(),
        benchmarks: value.benchmark_participant_digests.clone(),
        duplicates: value.excluded_semantic_duplicate_digests.clone(),
        rejected: value.rejected_participant_digests.clone(),
    }
}

fn roster_from_pb(
    value: RosterProtobufV4,
    family: &MomentumRawFeatureFamilyV4,
) -> Result<MomentumRawFeatureFutureRosterV4, String> {
    let mut f = field_map(value.fields)?;
    let result = MomentumRawFeatureFutureRosterV4 {
        roster_version: take(&mut f, "roster_version")?,
        family_digest: take(&mut f, "family_digest")?,
        learned_participant_digests: value.learned,
        benchmark_participant_digests: value.benchmarks,
        excluded_semantic_duplicate_digests: value.duplicates,
        rejected_participant_digests: value.rejected,
        inclusion_policy_digest: take(&mut f, "inclusion_policy_digest")?,
        status: parse_roster_status(&take(&mut f, "status")?)?,
        roster_digest: take(&mut f, "roster_digest")?,
    };
    finish_fields(f)?;
    validate_roster_v4(&result, family)?;
    Ok(result)
}

fn evaluation_to_pb(value: &MomentumRawFeatureEvaluationRegistrationV4) -> EvaluationProtobufV4 {
    EvaluationProtobufV4 {
        fields: vec![
            field("registration_version", &value.registration_version),
            field("agent_id", &value.agent_id),
            field("family_digest", &value.family_digest),
            field("roster_digest", &value.roster_digest),
            field(
                "frozen_mamba_closure_digest",
                &value.frozen_mamba_closure_digest,
            ),
            field("split_digest", &value.split_digest),
            field(
                "raw_feature_registration_digest",
                &value.raw_feature_registration_digest,
            ),
            field(
                "interaction_contribution_audit_digest",
                value
                    .interaction_contribution_audit_digest
                    .as_deref()
                    .unwrap_or(""),
            ),
            field("source_snapshot_digest", &value.source_snapshot_digest),
            field(
                "source_boundary_timestamp_ms",
                value.source_boundary_timestamp_ms,
            ),
            field(
                "provider_finality_boundary_ms",
                value.provider_finality_boundary_ms,
            ),
            field(
                "v4_final_untouched_reserve_digest",
                &value.v4_final_untouched_reserve_digest,
            ),
            field(
                "minimum_accepted_timestamp_ms",
                value.minimum_accepted_timestamp_ms,
            ),
            field(
                "labels_hidden_until_opening",
                value.labels_hidden_until_opening,
            ),
            field(
                "probabilities_hidden_until_opening",
                value.probabilities_hidden_until_opening,
            ),
            field("one_time_opening_required", value.one_time_opening_required),
            field(
                "winner_selection_forbidden_before_opening",
                value.winner_selection_forbidden_before_opening,
            ),
            field(
                "active_promotion_forbidden",
                value.active_promotion_forbidden,
            ),
            field(
                "reward_application_forbidden",
                value.reward_application_forbidden,
            ),
            field("maximum_requests", value.maximum_requests),
            field("maximum_concurrency", value.maximum_concurrency),
            field("maximum_retries", value.maximum_retries),
            field("registration_digest", &value.registration_digest),
        ],
        receipts: value.qualification_receipt_digests.clone(),
        protected_registrations: value.protected_registration_digests.clone(),
        protected_timestamps: value.protected_timestamp_ms.clone(),
        prior_validation_identities: value.prior_validation_identity_digests.clone(),
    }
}

fn evaluation_from_pb(
    value: EvaluationProtobufV4,
    family: &MomentumRawFeatureFamilyV4,
    roster: &MomentumRawFeatureFutureRosterV4,
) -> Result<MomentumRawFeatureEvaluationRegistrationV4, String> {
    let mut f = field_map(value.fields)?;
    let result = MomentumRawFeatureEvaluationRegistrationV4 {
        registration_version: take(&mut f, "registration_version")?,
        agent_id: take(&mut f, "agent_id")?,
        family_digest: take(&mut f, "family_digest")?,
        roster_digest: take(&mut f, "roster_digest")?,
        frozen_mamba_closure_digest: take(&mut f, "frozen_mamba_closure_digest")?,
        split_digest: take(&mut f, "split_digest")?,
        raw_feature_registration_digest: take(&mut f, "raw_feature_registration_digest")?,
        qualification_receipt_digests: value.receipts,
        interaction_contribution_audit_digest: take_optional(
            &mut f,
            "interaction_contribution_audit_digest",
        )?,
        source_snapshot_digest: take(&mut f, "source_snapshot_digest")?,
        source_boundary_timestamp_ms: take_u64(&mut f, "source_boundary_timestamp_ms")?,
        protected_registration_digests: value.protected_registrations,
        protected_timestamp_ms: value.protected_timestamps,
        provider_finality_boundary_ms: take_u64(&mut f, "provider_finality_boundary_ms")?,
        prior_validation_identity_digests: value.prior_validation_identities,
        v4_final_untouched_reserve_digest: take(&mut f, "v4_final_untouched_reserve_digest")?,
        minimum_accepted_timestamp_ms: take_u64(&mut f, "minimum_accepted_timestamp_ms")?,
        labels_hidden_until_opening: take_bool(&mut f, "labels_hidden_until_opening")?,
        probabilities_hidden_until_opening: take_bool(
            &mut f,
            "probabilities_hidden_until_opening",
        )?,
        one_time_opening_required: take_bool(&mut f, "one_time_opening_required")?,
        winner_selection_forbidden_before_opening: take_bool(
            &mut f,
            "winner_selection_forbidden_before_opening",
        )?,
        active_promotion_forbidden: take_bool(&mut f, "active_promotion_forbidden")?,
        reward_application_forbidden: take_bool(&mut f, "reward_application_forbidden")?,
        maximum_requests: take_usize(&mut f, "maximum_requests")?,
        maximum_concurrency: take_usize(&mut f, "maximum_concurrency")?,
        maximum_retries: take_usize(&mut f, "maximum_retries")?,
        registration_digest: take(&mut f, "registration_digest")?,
    };
    finish_fields(f)?;
    validate_evaluation_v4(&result, family, roster)?;
    Ok(result)
}

fn journal_to_pb(value: &MomentumRawFeatureJournalV4) -> JournalProtobufV4 {
    JournalProtobufV4 {
        fields: vec![
            field("journal_version", &value.journal_version),
            field("agent_id", &value.agent_id),
            field("closure_digest", &value.closure_digest),
            field("split_digest", &value.split_digest),
            field("registration_digest", &value.registration_digest),
            field(
                "family_digest",
                value.family_digest.as_deref().unwrap_or(""),
            ),
            field(
                "decision_digest",
                value.decision_digest.as_deref().unwrap_or(""),
            ),
            field(
                "roster_digest",
                value.roster_digest.as_deref().unwrap_or(""),
            ),
            field(
                "evaluation_registration_digest",
                value
                    .evaluation_registration_digest
                    .as_deref()
                    .unwrap_or(""),
            ),
            field(
                "preregistration_reopened_before_validation",
                value.preregistration_reopened_before_validation,
            ),
            field("final_reserve_accessed", value.final_reserve_accessed),
            field("prior_parameters_reused", value.prior_parameters_reused),
            field("active_registry_mutated", value.active_registry_mutated),
            field(
                "legacy_trainer_capability",
                &value.legacy_trainer_capability,
            ),
            field(
                "raw_feature_trainer_capability",
                &value.raw_feature_trainer_capability,
            ),
            field("status", format!("{:?}", value.status)),
            field("journal_digest", &value.journal_digest),
        ],
    }
}

fn journal_from_pb(value: JournalProtobufV4) -> Result<MomentumRawFeatureJournalV4, String> {
    let mut f = field_map(value.fields)?;
    let result = MomentumRawFeatureJournalV4 {
        journal_version: take(&mut f, "journal_version")?,
        agent_id: take(&mut f, "agent_id")?,
        closure_digest: take(&mut f, "closure_digest")?,
        split_digest: take(&mut f, "split_digest")?,
        registration_digest: take(&mut f, "registration_digest")?,
        family_digest: take_optional(&mut f, "family_digest")?,
        decision_digest: take_optional(&mut f, "decision_digest")?,
        roster_digest: take_optional(&mut f, "roster_digest")?,
        evaluation_registration_digest: take_optional(&mut f, "evaluation_registration_digest")?,
        preregistration_reopened_before_validation: take_bool(
            &mut f,
            "preregistration_reopened_before_validation",
        )?,
        final_reserve_accessed: take_bool(&mut f, "final_reserve_accessed")?,
        prior_parameters_reused: take_bool(&mut f, "prior_parameters_reused")?,
        active_registry_mutated: take_bool(&mut f, "active_registry_mutated")?,
        legacy_trainer_capability: take(&mut f, "legacy_trainer_capability")?,
        raw_feature_trainer_capability: take(&mut f, "raw_feature_trainer_capability")?,
        status: parse_execution_status(&take(&mut f, "status")?)?,
        journal_digest: take(&mut f, "journal_digest")?,
    };
    finish_fields(f)?;
    validate_journal_v4(&result)?;
    Ok(result)
}

pub fn encode_momentum_frozen_mamba_closure_protobuf_v4(
    value: &MomentumFrozenMambaPathClosureV4,
) -> Result<Vec<u8>, String> {
    validate_closure_v4(value)?;
    encode_message_v4(&closure_to_pb(value))
}
pub fn decode_momentum_frozen_mamba_closure_protobuf_v4(
    bytes: &[u8],
) -> Result<MomentumFrozenMambaPathClosureV4, String> {
    closure_from_pb(
        ClosureProtobufV4::decode(bytes).map_err(|_| "V4 closure Protobuf rejected".to_string())?,
    )
}
pub fn encode_momentum_raw_feature_split_protobuf_v4(
    value: &MomentumRawFeatureSplitV4,
) -> Result<Vec<u8>, String> {
    validate_split_v4(value)?;
    encode_message_v4(&split_to_pb(value))
}
pub fn decode_momentum_raw_feature_split_protobuf_v4(
    bytes: &[u8],
) -> Result<MomentumRawFeatureSplitV4, String> {
    split_from_pb(
        SplitProtobufV4::decode(bytes).map_err(|_| "V4 split Protobuf rejected".to_string())?,
    )
}
pub fn encode_momentum_validation_yield_audit_protobuf_v4(
    value: &MomentumValidationYieldAuditV4,
) -> Result<Vec<u8>, String> {
    validate_validation_yield_audit_v4(value)?;
    encode_message_v4(&validation_yield_audit_to_pb(value))
}
pub fn decode_momentum_validation_yield_audit_protobuf_v4(
    bytes: &[u8],
) -> Result<MomentumValidationYieldAuditV4, String> {
    validation_yield_audit_from_pb(
        ValidationYieldAuditProtobufV4::decode(bytes)
            .map_err(|_| "V4 validation-yield audit Protobuf rejected".to_string())?,
    )
}
pub fn encode_momentum_raw_feature_registration_protobuf_v4(
    value: &MomentumRawFeatureRegistrationV4,
) -> Result<Vec<u8>, String> {
    validate_registration_v4(value)?;
    encode_message_v4(&registration_to_pb(value))
}
pub fn decode_momentum_raw_feature_registration_protobuf_v4(
    bytes: &[u8],
) -> Result<MomentumRawFeatureRegistrationV4, String> {
    registration_from_pb(
        RegistrationProtobufV4::decode(bytes)
            .map_err(|_| "V4 registration Protobuf rejected".to_string())?,
    )
}
pub fn encode_frozen_candidate_participant_protobuf_v4(
    value: &FrozenCandidateParticipantV4,
) -> Result<Vec<u8>, String> {
    validate_participant_v4(value)?;
    encode_message_v4(&participant_to_pb(value))
}
pub fn decode_frozen_candidate_participant_protobuf_v4(
    bytes: &[u8],
) -> Result<FrozenCandidateParticipantV4, String> {
    participant_from_pb(
        ParticipantProtobufV4::decode(bytes)
            .map_err(|_| "V4 participant Protobuf rejected".to_string())?,
    )
}
pub fn encode_momentum_raw_feature_qualification_protobuf_v4(
    value: &MomentumRawFeatureQualificationReceiptV4,
) -> Result<Vec<u8>, String> {
    validate_receipt_v4(value)?;
    encode_message_v4(&receipt_to_pb(value))
}
pub fn decode_momentum_raw_feature_qualification_protobuf_v4(
    bytes: &[u8],
) -> Result<MomentumRawFeatureQualificationReceiptV4, String> {
    receipt_from_pb(
        ReceiptProtobufV4::decode(bytes).map_err(|_| "V4 receipt Protobuf rejected".to_string())?,
    )
}
pub fn encode_momentum_interaction_contribution_protobuf_v4(
    value: &MomentumInteractionContributionAuditV4,
) -> Result<Vec<u8>, String> {
    validate_contribution_v4(value)?;
    encode_message_v4(&contribution_to_pb(value))
}
pub fn decode_momentum_interaction_contribution_protobuf_v4(
    bytes: &[u8],
) -> Result<MomentumInteractionContributionAuditV4, String> {
    contribution_from_pb(
        ContributionProtobufV4::decode(bytes)
            .map_err(|_| "V4 contribution Protobuf rejected".to_string())?,
    )
}
pub fn encode_momentum_raw_feature_family_protobuf_v4(
    value: &MomentumRawFeatureFamilyV4,
) -> Result<Vec<u8>, String> {
    validate_family_v4(value)?;
    encode_message_v4(&family_to_pb(value))
}
pub fn decode_momentum_raw_feature_family_protobuf_v4(
    bytes: &[u8],
) -> Result<MomentumRawFeatureFamilyV4, String> {
    family_from_pb(
        FamilyProtobufV4::decode(bytes).map_err(|_| "V4 family Protobuf rejected".to_string())?,
    )
}
pub fn encode_momentum_raw_feature_decision_protobuf_v4(
    value: &MomentumRawFeaturePathDecisionArtifactV4,
    family: &MomentumRawFeatureFamilyV4,
) -> Result<Vec<u8>, String> {
    validate_decision_v4(value, family)?;
    encode_message_v4(&decision_to_pb(value))
}
pub fn decode_momentum_raw_feature_decision_protobuf_v4(
    bytes: &[u8],
    family: &MomentumRawFeatureFamilyV4,
) -> Result<MomentumRawFeaturePathDecisionArtifactV4, String> {
    decision_from_pb(
        DecisionProtobufV4::decode(bytes)
            .map_err(|_| "V4 decision Protobuf rejected".to_string())?,
        family,
    )
}
pub fn encode_momentum_raw_feature_roster_protobuf_v4(
    value: &MomentumRawFeatureFutureRosterV4,
    family: &MomentumRawFeatureFamilyV4,
) -> Result<Vec<u8>, String> {
    validate_roster_v4(value, family)?;
    encode_message_v4(&roster_to_pb(value))
}
pub fn decode_momentum_raw_feature_roster_protobuf_v4(
    bytes: &[u8],
    family: &MomentumRawFeatureFamilyV4,
) -> Result<MomentumRawFeatureFutureRosterV4, String> {
    roster_from_pb(
        RosterProtobufV4::decode(bytes).map_err(|_| "V4 roster Protobuf rejected".to_string())?,
        family,
    )
}
pub fn encode_momentum_raw_feature_evaluation_protobuf_v4(
    value: &MomentumRawFeatureEvaluationRegistrationV4,
    family: &MomentumRawFeatureFamilyV4,
    roster: &MomentumRawFeatureFutureRosterV4,
) -> Result<Vec<u8>, String> {
    validate_evaluation_v4(value, family, roster)?;
    encode_message_v4(&evaluation_to_pb(value))
}
pub fn decode_momentum_raw_feature_evaluation_protobuf_v4(
    bytes: &[u8],
    family: &MomentumRawFeatureFamilyV4,
    roster: &MomentumRawFeatureFutureRosterV4,
) -> Result<MomentumRawFeatureEvaluationRegistrationV4, String> {
    evaluation_from_pb(
        EvaluationProtobufV4::decode(bytes)
            .map_err(|_| "V4 evaluation Protobuf rejected".to_string())?,
        family,
        roster,
    )
}
pub fn encode_momentum_raw_feature_journal_protobuf_v4(
    value: &MomentumRawFeatureJournalV4,
) -> Result<Vec<u8>, String> {
    validate_journal_v4(value)?;
    encode_message_v4(&journal_to_pb(value))
}
pub fn decode_momentum_raw_feature_journal_protobuf_v4(
    bytes: &[u8],
) -> Result<MomentumRawFeatureJournalV4, String> {
    journal_from_pb(
        JournalProtobufV4::decode(bytes).map_err(|_| "V4 journal Protobuf rejected".to_string())?,
    )
}

fn persist_artifact_v4(
    path: &Path,
    bytes: &[u8],
    digest: &str,
    decode_digest: impl Fn(&[u8]) -> Result<String, String>,
) -> Result<(usize, usize), String> {
    match atomic_write_verified_v0(path, bytes, digest, decode_digest)? {
        AgentPrivateLearningArtifactWriteStatusV0::Written => Ok((1, 0)),
        AgentPrivateLearningArtifactWriteStatusV0::DuplicateRejected => Ok((0, 1)),
    }
}

fn add_counts(total: &mut (usize, usize), next: (usize, usize)) {
    total.0 += next.0;
    total.1 += next.1;
}

fn persist_preregistration_v4(
    root: &Path,
    closure: &MomentumFrozenMambaPathClosureV4,
    split: &MomentumRawFeatureSplitV4,
    registration: &MomentumRawFeatureRegistrationV4,
) -> Result<(usize, usize), String> {
    validate_closure_v4(closure)?;
    validate_split_v4(split)?;
    validate_registration_v4(registration)?;
    if registration.frozen_mamba_closure_digest != closure.closure_digest
        || registration.split_digest != split.split_digest
    {
        return Err("V4 preregistration cross-binding rejected".to_string());
    }
    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_artifact_v4(
            &root
                .join("closures")
                .join(format!("{}.pb", closure.closure_digest)),
            &encode_momentum_frozen_mamba_closure_protobuf_v4(closure)?,
            &closure.closure_digest,
            |bytes| Ok(decode_momentum_frozen_mamba_closure_protobuf_v4(bytes)?.closure_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_artifact_v4(
            &root
                .join("splits")
                .join(format!("{}.pb", split.split_digest)),
            &encode_momentum_raw_feature_split_protobuf_v4(split)?,
            &split.split_digest,
            |bytes| Ok(decode_momentum_raw_feature_split_protobuf_v4(bytes)?.split_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_artifact_v4(
            &root
                .join("registrations")
                .join(format!("{}.pb", registration.registration_digest)),
            &encode_momentum_raw_feature_registration_protobuf_v4(registration)?,
            &registration.registration_digest,
            |bytes| {
                Ok(
                    decode_momentum_raw_feature_registration_protobuf_v4(bytes)?
                        .registration_digest,
                )
            },
        )?,
    );
    Ok(counts)
}

fn reopen_preregistration_v4(
    root: &Path,
) -> Result<
    (
        MomentumFrozenMambaPathClosureV4,
        MomentumRawFeatureSplitV4,
        MomentumRawFeatureRegistrationV4,
    ),
    String,
> {
    let closure = read_single(
        &root.join("closures"),
        decode_momentum_frozen_mamba_closure_protobuf_v4,
    )?;
    let split = read_single(
        &root.join("splits"),
        decode_momentum_raw_feature_split_protobuf_v4,
    )?;
    let registration = read_single(
        &root.join("registrations"),
        decode_momentum_raw_feature_registration_protobuf_v4,
    )?;
    if registration.frozen_mamba_closure_digest != closure.closure_digest
        || registration.split_digest != split.split_digest
    {
        return Err("V4 reopened preregistration rejected".to_string());
    }
    Ok((closure, split, registration))
}

fn persist_experiment_v4(
    root: &Path,
    experiment: &ExperimentV4,
    journal: &MomentumRawFeatureJournalV4,
) -> Result<(usize, usize), String> {
    validate_validation_yield_audit_v4(&experiment.validation_yield_audit)?;
    validate_family_v4(&experiment.family)?;
    validate_decision_v4(&experiment.decision, &experiment.family)?;
    validate_journal_v4(journal)?;
    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_artifact_v4(
            &root.join("validation_yield_audits").join(format!(
                "{}.pb",
                experiment.validation_yield_audit.audit_digest
            )),
            &encode_momentum_validation_yield_audit_protobuf_v4(
                &experiment.validation_yield_audit,
            )?,
            &experiment.validation_yield_audit.audit_digest,
            |bytes| Ok(decode_momentum_validation_yield_audit_protobuf_v4(bytes)?.audit_digest),
        )?,
    );
    for participant in &experiment.family.participants {
        add_counts(
            &mut counts,
            persist_artifact_v4(
                &root
                    .join("participants")
                    .join(format!("{}.pb", participant.participant_digest)),
                &encode_frozen_candidate_participant_protobuf_v4(participant)?,
                &participant.participant_digest,
                |bytes| {
                    Ok(decode_frozen_candidate_participant_protobuf_v4(bytes)?.participant_digest)
                },
            )?,
        );
    }
    for receipt in &experiment.family.qualification_receipts {
        add_counts(
            &mut counts,
            persist_artifact_v4(
                &root
                    .join("qualification_receipts")
                    .join(format!("{}.pb", receipt.receipt_digest)),
                &encode_momentum_raw_feature_qualification_protobuf_v4(receipt)?,
                &receipt.receipt_digest,
                |bytes| {
                    Ok(
                        decode_momentum_raw_feature_qualification_protobuf_v4(bytes)?
                            .receipt_digest,
                    )
                },
            )?,
        );
    }
    if let Some(audit) = &experiment.family.interaction_contribution_audit {
        add_counts(
            &mut counts,
            persist_artifact_v4(
                &root
                    .join("interaction_contribution_audits")
                    .join(format!("{}.pb", audit.audit_digest)),
                &encode_momentum_interaction_contribution_protobuf_v4(audit)?,
                &audit.audit_digest,
                |bytes| {
                    Ok(decode_momentum_interaction_contribution_protobuf_v4(bytes)?.audit_digest)
                },
            )?,
        );
    }
    add_counts(
        &mut counts,
        persist_artifact_v4(
            &root
                .join("families")
                .join(format!("{}.pb", experiment.family.family_digest)),
            &encode_momentum_raw_feature_family_protobuf_v4(&experiment.family)?,
            &experiment.family.family_digest,
            |bytes| Ok(decode_momentum_raw_feature_family_protobuf_v4(bytes)?.family_digest),
        )?,
    );
    let family_for_decision = experiment.family.clone();
    add_counts(
        &mut counts,
        persist_artifact_v4(
            &root
                .join("path_decisions")
                .join(format!("{}.pb", experiment.decision.decision_digest)),
            &encode_momentum_raw_feature_decision_protobuf_v4(
                &experiment.decision,
                &experiment.family,
            )?,
            &experiment.decision.decision_digest,
            move |bytes| {
                Ok(
                    decode_momentum_raw_feature_decision_protobuf_v4(bytes, &family_for_decision)?
                        .decision_digest,
                )
            },
        )?,
    );
    if let Some(roster) = &experiment.roster {
        let family_for_roster = experiment.family.clone();
        add_counts(
            &mut counts,
            persist_artifact_v4(
                &root
                    .join("rosters")
                    .join(format!("{}.pb", roster.roster_digest)),
                &encode_momentum_raw_feature_roster_protobuf_v4(roster, &experiment.family)?,
                &roster.roster_digest,
                move |bytes| {
                    Ok(
                        decode_momentum_raw_feature_roster_protobuf_v4(bytes, &family_for_roster)?
                            .roster_digest,
                    )
                },
            )?,
        );
    }
    if let (Some(evaluation), Some(roster)) = (&experiment.evaluation, &experiment.roster) {
        let family_for_evaluation = experiment.family.clone();
        let roster_for_evaluation = roster.clone();
        add_counts(
            &mut counts,
            persist_artifact_v4(
                &root
                    .join("evaluation_registrations")
                    .join(format!("{}.pb", evaluation.registration_digest)),
                &encode_momentum_raw_feature_evaluation_protobuf_v4(
                    evaluation,
                    &experiment.family,
                    roster,
                )?,
                &evaluation.registration_digest,
                move |bytes| {
                    Ok(decode_momentum_raw_feature_evaluation_protobuf_v4(
                        bytes,
                        &family_for_evaluation,
                        &roster_for_evaluation,
                    )?
                    .registration_digest)
                },
            )?,
        );
    }
    add_counts(
        &mut counts,
        persist_artifact_v4(
            &root
                .join("journals")
                .join(format!("{}.pb", journal.journal_digest)),
            &encode_momentum_raw_feature_journal_protobuf_v4(journal)?,
            &journal.journal_digest,
            |bytes| Ok(decode_momentum_raw_feature_journal_protobuf_v4(bytes)?.journal_digest),
        )?,
    );
    Ok(counts)
}

fn reopen_experiment_v4(
    root: &Path,
) -> Result<(ExperimentV4, MomentumRawFeatureJournalV4), String> {
    let validation_yield_audit = read_single(
        &root.join("validation_yield_audits"),
        decode_momentum_validation_yield_audit_protobuf_v4,
    )?;
    let family = read_single(
        &root.join("families"),
        decode_momentum_raw_feature_family_protobuf_v4,
    )?;
    if validation_yield_audit.source_snapshot_digest != family.source_snapshot_digest {
        return Err("V4 validation-yield family binding rejected".to_string());
    }
    let decision = read_corrected_decision_v4(&root.join("path_decisions"), &family)?;
    let (expected_roster, roster_status) = derive_roster_v4(&family)?;
    let roster = if expected_roster.is_some() {
        Some(read_single(&root.join("rosters"), |bytes| {
            decode_momentum_raw_feature_roster_protobuf_v4(bytes, &family)
        })?)
    } else {
        if root.join("rosters").exists() {
            return Err("V4 unexpected roster artifact rejected".to_string());
        }
        None
    };
    if roster != expected_roster {
        return Err("V4 reopened roster diverged".to_string());
    }
    let (evaluation, evaluation_status) = if let Some(roster) = &roster {
        let value = read_single(&root.join("evaluation_registrations"), |bytes| {
            decode_momentum_raw_feature_evaluation_protobuf_v4(bytes, &family, roster)
        })?;
        (
            Some(value),
            MomentumRawFeatureEvaluationStatusV4::Registered,
        )
    } else {
        if root.join("evaluation_registrations").exists() {
            return Err("V4 unexpected evaluation registration rejected".to_string());
        }
        let status = match roster_status {
            MomentumRawFeatureRosterStatusV4::QualificationEvidenceInsufficient => {
                MomentumRawFeatureEvaluationStatusV4::QualificationEvidenceInsufficient
            }
            MomentumRawFeatureRosterStatusV4::BenchmarkUnavailable => {
                MomentumRawFeatureEvaluationStatusV4::BenchmarkUnavailable
            }
            MomentumRawFeatureRosterStatusV4::SemanticDuplicateOnly => {
                MomentumRawFeatureEvaluationStatusV4::SemanticDuplicateOnly
            }
            _ => MomentumRawFeatureEvaluationStatusV4::NoQualifiedLearnedParticipant,
        };
        (None, status)
    };
    let legacy_decision_digest =
        legacy_insufficient_decision_v4(&family).map(|value| value.decision_digest);
    let mut matching_journals = Vec::new();
    for path in protobuf_paths(&root.join("journals"))? {
        let journal = decode_momentum_raw_feature_journal_protobuf_v4(
            &fs::read(path).map_err(|_| "V4 artifact read failed".to_string())?,
        )?;
        if journal.decision_digest.as_deref() == Some(decision.decision_digest.as_str()) {
            matching_journals.push(journal);
        } else if journal.family_digest.as_deref() != Some(family.family_digest.as_str())
            || journal.decision_digest != legacy_decision_digest
        {
            return Err("V4 unexpected journal artifact rejected".to_string());
        }
    }
    if matching_journals.len() != 1 {
        return Err("V4 corrected journal identity rejected".to_string());
    }
    let journal = matching_journals.remove(0);
    if journal.family_digest.as_deref() != Some(family.family_digest.as_str())
        || journal.decision_digest.as_deref() != Some(decision.decision_digest.as_str())
        || journal.roster_digest.as_deref()
            != roster.as_ref().map(|item| item.roster_digest.as_str())
        || journal.evaluation_registration_digest.as_deref()
            != evaluation
                .as_ref()
                .map(|item| item.registration_digest.as_str())
    {
        return Err("V4 journal cross-binding rejected".to_string());
    }
    Ok((
        ExperimentV4 {
            validation_yield_audit,
            family,
            decision,
            roster,
            roster_status,
            evaluation,
            evaluation_status,
        },
        journal,
    ))
}

fn collect_protected_v4(
    root: &Path,
    current: &Path,
    values: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), String> {
    if current == root.join("v4") {
        return Ok(());
    }
    if current.is_file() {
        values.push((
            current
                .strip_prefix(root)
                .map_err(|_| "V4 protected path rejected".to_string())?
                .to_path_buf(),
            fs::read(current).map_err(|_| "V4 protected read failed".to_string())?,
        ));
        return Ok(());
    }
    if !current.is_dir() {
        return Ok(());
    }
    let mut children = fs::read_dir(current)
        .map_err(|_| "V4 protected directory rejected".to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    children.sort();
    for child in children {
        collect_protected_v4(root, &child, values)?;
    }
    Ok(())
}

fn report_digest_v4(value: &MomentumRawFeatureReportV4) -> String {
    let mut canonical = value.clone();
    canonical.report_digest.clear();
    stable_hash_string(&format!("{canonical:?}"))
}

fn base_report_v4(
    mode: AgentPrivateLearningRunModeV0,
    status: MomentumRawFeatureExecutionStatusV4,
    protected_artifacts_unchanged: bool,
    active_state_unchanged: bool,
) -> MomentumRawFeatureReportV4 {
    let mut report = MomentumRawFeatureReportV4 {
        report_version: "momentum-raw-feature-report-v4".to_string(),
        mode,
        status,
        closure: None,
        split: None,
        registration: None,
        validation_yield_audit: None,
        family: None,
        decision: None,
        roster: None,
        roster_status: MomentumRawFeatureRosterStatusV4::QualificationEvidenceInsufficient,
        evaluation_registration: None,
        evaluation_registration_status:
            MomentumRawFeatureEvaluationStatusV4::QualificationEvidenceInsufficient,
        journal: None,
        artifacts_written: 0,
        duplicate_artifact_count: 0,
        storage_failure_count: 0,
        protected_artifacts_unchanged,
        active_state_unchanged,
        safety_counters: zero_safety_counters_v4(),
        report_digest: String::new(),
    };
    report.report_digest = report_digest_v4(&report);
    report
}

fn populate_report_v4(
    report: &mut MomentumRawFeatureReportV4,
    preregistration: (
        MomentumFrozenMambaPathClosureV4,
        MomentumRawFeatureSplitV4,
        MomentumRawFeatureRegistrationV4,
    ),
    experiment: Option<(ExperimentV4, MomentumRawFeatureJournalV4)>,
) {
    report.closure = Some(preregistration.0);
    report.split = Some(preregistration.1);
    report.registration = Some(preregistration.2);
    if let Some((experiment, journal)) = experiment {
        report.validation_yield_audit = Some(experiment.validation_yield_audit);
        report.family = Some(experiment.family);
        report.decision = Some(experiment.decision);
        report.roster = experiment.roster;
        report.roster_status = experiment.roster_status;
        report.evaluation_registration = experiment.evaluation;
        report.evaluation_registration_status = experiment.evaluation_status;
        report.journal = Some(journal);
    }
    report.report_digest = report_digest_v4(report);
}

pub fn run_momentum_raw_feature_v4(
    root: &Path,
    snapshots: &[DataSnapshot],
    reservation: &ProtectedEvaluationReservationV1,
    mode: AgentPrivateLearningRunModeV0,
) -> MomentumRawFeatureReportV4 {
    let active_before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let mut protected_before = Vec::new();
    if collect_protected_v4(root, root, &mut protected_before).is_err() {
        return base_report_v4(
            mode,
            MomentumRawFeatureExecutionStatusV4::TechnicalFailure,
            false,
            true,
        );
    }
    let history = match load_frozen_history_v4(root, snapshots) {
        Ok(value) => value,
        Err(_) => {
            return base_report_v4(
                mode,
                MomentumRawFeatureExecutionStatusV4::TechnicalFailure,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
        }
    };
    let closure = match derive_closure_v4(&history) {
        Ok(value) => value,
        Err(_) => {
            return base_report_v4(
                mode,
                MomentumRawFeatureExecutionStatusV4::TechnicalFailure,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
        }
    };
    let split = match derive_split_v4(&history, reservation) {
        Ok(value) => value,
        Err(_) => {
            let mut report = base_report_v4(
                mode,
                MomentumRawFeatureExecutionStatusV4::InsufficientFreshValidation,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
            report.closure = Some(closure);
            report.report_digest = report_digest_v4(&report);
            return report;
        }
    };
    let registration = match derive_registration_v4(&history, &closure, &split) {
        Ok(value) => value,
        Err(_) => {
            let mut report = base_report_v4(
                mode,
                MomentumRawFeatureExecutionStatusV4::TechnicalFailure,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
            report.closure = Some(closure);
            report.split = Some(split);
            report.report_digest = report_digest_v4(&report);
            return report;
        }
    };
    let expected_prereg = (closure, split, registration);
    let v4_root = root.join("v4").join(AGENT_ID_V4);
    if mode != AgentPrivateLearningRunModeV0::ExecuteLocal {
        let persisted = reopen_preregistration_v4(&v4_root)
            .and_then(|prereg| {
                reopen_experiment_v4(&v4_root).map(|experiment| (prereg, experiment))
            })
            .ok();
        let mut report = base_report_v4(
            mode,
            if persisted.is_some() {
                MomentumRawFeatureExecutionStatusV4::AlreadyExecuted
            } else {
                MomentumRawFeatureExecutionStatusV4::Planned
            },
            true,
            stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before,
        );
        if let Some((prereg, experiment)) = persisted {
            populate_report_v4(&mut report, prereg, Some(experiment));
        } else {
            populate_report_v4(&mut report, expected_prereg, None);
        }
        return report;
    }
    let persisted_before = reopen_preregistration_v4(&v4_root)
        .and_then(|prereg| reopen_experiment_v4(&v4_root).map(|experiment| (prereg, experiment)))
        .ok();
    let mut counts = (0, 0);
    let (stored_prereg, stored_experiment) = if let Some((prereg, experiment)) = persisted_before {
        if prereg != expected_prereg {
            let mut report = base_report_v4(
                mode,
                MomentumRawFeatureExecutionStatusV4::TechnicalFailure,
                true,
                true,
            );
            report.storage_failure_count = 1;
            report.report_digest = report_digest_v4(&report);
            return report;
        }
        (prereg, experiment)
    } else {
        let prereg_counts = match persist_preregistration_v4(
            &v4_root,
            &expected_prereg.0,
            &expected_prereg.1,
            &expected_prereg.2,
        ) {
            Ok(value) => value,
            Err(_) => {
                let mut report = base_report_v4(
                    mode,
                    MomentumRawFeatureExecutionStatusV4::TechnicalFailure,
                    true,
                    true,
                );
                report.storage_failure_count = 1;
                report.report_digest = report_digest_v4(&report);
                return report;
            }
        };
        add_counts(&mut counts, prereg_counts);
        let prereg = match reopen_preregistration_v4(&v4_root) {
            Ok(value) if value == expected_prereg => value,
            _ => {
                let mut report = base_report_v4(
                    mode,
                    MomentumRawFeatureExecutionStatusV4::TechnicalFailure,
                    true,
                    true,
                );
                report.storage_failure_count = 1;
                report.report_digest = report_digest_v4(&report);
                return report;
            }
        };
        let experiment =
            match run_experiment_v4(&history, &prereg.0, &prereg.1, &prereg.2, reservation) {
                Ok(value) => value,
                Err(_) => {
                    let mut report = base_report_v4(
                        mode,
                        MomentumRawFeatureExecutionStatusV4::TechnicalFailure,
                        true,
                        true,
                    );
                    populate_report_v4(&mut report, prereg, None);
                    return report;
                }
            };
        let mut journal = MomentumRawFeatureJournalV4 {
            journal_version: JOURNAL_VERSION_V4.to_string(),
            agent_id: AGENT_ID_V4.to_string(),
            closure_digest: prereg.0.closure_digest.clone(),
            split_digest: prereg.1.split_digest.clone(),
            registration_digest: prereg.2.registration_digest.clone(),
            family_digest: Some(experiment.family.family_digest.clone()),
            decision_digest: Some(experiment.decision.decision_digest.clone()),
            roster_digest: experiment
                .roster
                .as_ref()
                .map(|item| item.roster_digest.clone()),
            evaluation_registration_digest: experiment
                .evaluation
                .as_ref()
                .map(|item| item.registration_digest.clone()),
            preregistration_reopened_before_validation: true,
            final_reserve_accessed: false,
            prior_parameters_reused: false,
            active_registry_mutated: false,
            legacy_trainer_capability: "MomentumFrozenMambaLegacy/terminal-current-evidence-policy"
                .to_string(),
            raw_feature_trainer_capability: "MomentumRawFeatureV4/ShadowOnly".to_string(),
            status: MomentumRawFeatureExecutionStatusV4::Executed,
            journal_digest: String::new(),
        };
        journal.journal_digest = journal_digest_v4(&journal);
        if validate_journal_v4(&journal).is_err() {
            let mut report = base_report_v4(
                mode,
                MomentumRawFeatureExecutionStatusV4::TechnicalFailure,
                true,
                true,
            );
            report.storage_failure_count = 1;
            report.report_digest = report_digest_v4(&report);
            return report;
        }
        match persist_experiment_v4(&v4_root, &experiment, &journal) {
            Ok(value) => add_counts(&mut counts, value),
            Err(_) => {
                let mut report = base_report_v4(
                    mode,
                    MomentumRawFeatureExecutionStatusV4::TechnicalFailure,
                    true,
                    true,
                );
                report.storage_failure_count = 1;
                report.report_digest = report_digest_v4(&report);
                return report;
            }
        }
        (prereg, (experiment, journal))
    };
    if counts.0 == 0 {
        match persist_preregistration_v4(
            &v4_root,
            &stored_prereg.0,
            &stored_prereg.1,
            &stored_prereg.2,
        ) {
            Ok(value) => add_counts(&mut counts, value),
            Err(_) => {}
        }
        match persist_experiment_v4(&v4_root, &stored_experiment.0, &stored_experiment.1) {
            Ok(value) => add_counts(&mut counts, value),
            Err(_) => {}
        }
    }
    let reopened = reopen_preregistration_v4(&v4_root)
        .and_then(|prereg| reopen_experiment_v4(&v4_root).map(|experiment| (prereg, experiment)));
    let (reopened_prereg, reopened_experiment) = match reopened {
        Ok(value) => value,
        Err(_) => {
            let mut report = base_report_v4(
                mode,
                MomentumRawFeatureExecutionStatusV4::TechnicalFailure,
                true,
                true,
            );
            report.storage_failure_count = 1;
            report.report_digest = report_digest_v4(&report);
            return report;
        }
    };
    let mut protected_after = Vec::new();
    let protected_unchanged = collect_protected_v4(root, root, &mut protected_after).is_ok()
        && protected_before == protected_after;
    let active_unchanged =
        stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before;
    let status = if counts.0 == 0 {
        MomentumRawFeatureExecutionStatusV4::AlreadyExecuted
    } else {
        MomentumRawFeatureExecutionStatusV4::Executed
    };
    let mut report = base_report_v4(mode, status, protected_unchanged, active_unchanged);
    report.artifacts_written = counts.0;
    report.duplicate_artifact_count = counts.1;
    report.storage_failure_count = usize::from(
        !protected_unchanged
            || !active_unchanged
            || (status == MomentumRawFeatureExecutionStatusV4::AlreadyExecuted && counts.0 > 0),
    );
    populate_report_v4(&mut report, reopened_prereg, Some(reopened_experiment));
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "soma-momentum-raw-feature-v4-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn closure_fixture() -> MomentumFrozenMambaPathClosureV4 {
        let mut value = MomentumFrozenMambaPathClosureV4 {
            closure_version: CLOSURE_VERSION_V4.to_string(),
            agent_id: AGENT_ID_V4.to_string(),
            source_snapshot_digest: "snapshot".to_string(),
            canonical_intent_digest: "intent".to_string(),
            canonical_view_digest: "view".to_string(),
            v1_family_digest: "v1-family".to_string(),
            v2_family_digest: "v2-family".to_string(),
            v3_family_digest: "v3-family".to_string(),
            v3_route_decision_digest: "v3-decision".to_string(),
            frozen_encoder_digest: "encoder".to_string(),
            feature_policy_digest: "feature-policy".to_string(),
            label_policy_digest: "label-policy".to_string(),
            genuine_mamba_qualified_count: 0,
            head_only_repair_forbidden: true,
            frozen_representation_sweep_forbidden: true,
            frozen_mamba_parent_use_forbidden: true,
            reopening_requires_new_encoder_identity: true,
            reopening_requires_new_evidence_identity: true,
            reopening_requires_new_preregistration: true,
            decision: MomentumFrozenMambaClosureDecisionV4::ClosedForCurrentEvidenceAndPolicy,
            closure_digest: String::new(),
        };
        value.closure_digest = closure_digest_v4(&value);
        value
    }

    fn split_fixture() -> MomentumRawFeatureSplitV4 {
        let mut value = MomentumRawFeatureSplitV4 {
            split_version: SPLIT_VERSION_V4.to_string(),
            source_snapshot_digest: "snapshot".to_string(),
            v3_split_digest: "v3-split".to_string(),
            v3_route_decision_digest: "v3-decision".to_string(),
            training_range: IndexRangeV0 { start: 0, end: 240 },
            purge_range: IndexRangeV0 {
                start: 240,
                end: 264,
            },
            fresh_validation_range: IndexRangeV0 {
                start: 264,
                end: 288,
            },
            final_untouched_range: IndexRangeV0 {
                start: 288,
                end: 312,
            },
            minimum_validation_samples: 24,
            minimum_final_reserve_samples: 24,
            prior_qualification_overlap_count: 0,
            prospective_overlap_count: 0,
            historical_test_overlap_count: 0,
            future_evaluation_overlap_count: 0,
            split_digest: String::new(),
        };
        value.split_digest = split_digest_v4(&value);
        value
    }

    fn validation_yield_audit_fixture() -> MomentumValidationYieldAuditV4 {
        let mut value = MomentumValidationYieldAuditV4 {
            audit_version: VALIDATION_YIELD_AUDIT_VERSION_V4.to_string(),
            source_snapshot_digest: "snapshot".to_string(),
            label_policy_digest: "label-policy".to_string(),
            validation_index_range: IndexRangeV0 { start: 24, end: 48 },
            validation_index_count: 24,
            minimum_required_valid_samples: 24,
            valid_labelled_sample_count: 20,
            neutral_excluded_count: 4,
            horizon_unavailable_count: 0,
            feature_unavailable_count: 0,
            substantive_qualification_possible: false,
            audit_digest: String::new(),
        };
        value.audit_digest = validation_yield_audit_digest_v4(&value);
        value
    }

    fn config_fixture(
        kind: MomentumRawFeatureModelKindV4,
        seed: u64,
    ) -> MomentumRawFeatureParticipantConfigV4 {
        let (participant_id, learned) = expected_config_identity(kind);
        let mut value = MomentumRawFeatureParticipantConfigV4 {
            participant_id: participant_id.to_string(),
            model_kind: kind,
            feature_policy_digest: "feature-policy".to_string(),
            label_policy_digest: "label-policy".to_string(),
            input_feature_schema_digest: schema_digest_v4(kind, 6),
            learning_rate_bits: if learned { 0.05_f32.to_bits() } else { 0 },
            l2_regularization_bits: if learned { 0.001_f32.to_bits() } else { 0 },
            maximum_epochs: if learned { 30 } else { 0 },
            initialization_seed: if learned { seed } else { 0 },
            fresh_initialization: learned,
            training_only_normalizer: learned,
            config_digest: String::new(),
        };
        value.config_digest = config_digest_v4(&value);
        value
    }

    fn registration_fixture() -> MomentumRawFeatureRegistrationV4 {
        let closure = closure_fixture();
        let split = split_fixture();
        let mut value = MomentumRawFeatureRegistrationV4 {
            registration_version: REGISTRATION_VERSION_V4.to_string(),
            agent_id: AGENT_ID_V4.to_string(),
            source_snapshot_digest: "snapshot".to_string(),
            canonical_intent_digest: "intent".to_string(),
            canonical_view_digest: "view".to_string(),
            frozen_mamba_closure_digest: closure.closure_digest,
            split_digest: split.split_digest,
            participants: vec![
                config_fixture(MomentumRawFeatureModelKindV4::RawFeatureLogistic, 1),
                config_fixture(
                    MomentumRawFeatureModelKindV4::RawFeatureInteractionLogistic,
                    2,
                ),
                config_fixture(MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant, 0),
            ],
            maximum_learned_participants: 2,
            interaction_contribution_policy_digest: interaction_policy_digest_v4(),
            fresh_validation_hidden: true,
            final_reserve_forbidden: true,
            historical_test_forbidden: true,
            future_evaluation_forbidden: true,
            winner_selection_forbidden: true,
            active_promotion_forbidden: true,
            reward_application_forbidden: true,
            registration_digest: String::new(),
        };
        value.registration_digest = registration_digest_v4(&value);
        value
    }

    fn participant_fixture(kind: MomentumRawFeatureModelKindV4) -> FrozenCandidateParticipantV4 {
        let config = config_fixture(
            kind,
            match kind {
                MomentumRawFeatureModelKindV4::RawFeatureLogistic => 1,
                MomentumRawFeatureModelKindV4::RawFeatureInteractionLogistic => 2,
                MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant => 0,
            },
        );
        let role = match kind {
            MomentumRawFeatureModelKindV4::RawFeatureLogistic => {
                MomentumRawFeatureRoleV4::LearnedRawLogistic
            }
            MomentumRawFeatureModelKindV4::RawFeatureInteractionLogistic => {
                MomentumRawFeatureRoleV4::LearnedInteractionLogistic
            }
            MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant => {
                MomentumRawFeatureRoleV4::ConstantBenchmark
            }
        };
        let mut value = FrozenCandidateParticipantV4 {
            participant_version: PARTICIPANT_VERSION_V4.to_string(),
            participant_id: config.participant_id.clone(),
            participant_role: role,
            model_kind: kind,
            config_digest: config.config_digest,
            source_snapshot_digest: "snapshot".to_string(),
            training_range_digest: "training-range".to_string(),
            fresh_validation_range_digest: "validation-range".to_string(),
            validation_timestamp_digest: "validation-timestamps".to_string(),
            input_feature_schema_digest: config.input_feature_schema_digest,
            model_artifact_digest: format!("model-{kind:?}"),
            parameter_digest: format!("parameters-{kind:?}"),
            normalizer_digest: format!("normalizer-{kind:?}"),
            training_identity_digest: "training-identities".to_string(),
            fresh_initialization: kind != MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant,
            prior_parameters_reused: false,
            prior_normalizer_reused: false,
            prior_predictions_reused: false,
            validation_parameter_updates: 0,
            deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
            participant_digest: String::new(),
        };
        value.participant_digest = participant_digest_v4(&value);
        value
    }

    fn metric_fixture() -> EvaluationMetricsV0 {
        EvaluationMetricsV0 {
            brier_score: 0.2,
            sample_count: 24,
            accuracy: 0.5,
            positive_label_rate: 0.5,
            mean_predicted_probability: 0.5,
            high_confidence_error_count: 0,
            abstention_count: 0,
            calibration_buckets: vec![],
        }
    }

    fn family_fixture(
        raw_status: MomentumRawFeatureQualificationStatusV4,
        interaction_status: MomentumRawFeatureQualificationStatusV4,
    ) -> MomentumRawFeatureFamilyV4 {
        let raw = participant_fixture(MomentumRawFeatureModelKindV4::RawFeatureLogistic);
        let interaction =
            participant_fixture(MomentumRawFeatureModelKindV4::RawFeatureInteractionLogistic);
        let constant =
            participant_fixture(MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant);
        let mut audit = MomentumInteractionContributionAuditV4 {
            participant_digest: interaction.participant_digest.clone(),
            original_feature_parameter_digest: "original".to_string(),
            squared_feature_parameter_digest: "squared".to_string(),
            pairwise_feature_parameter_digest: "pairwise".to_string(),
            original_block_nonzero: true,
            nonlinear_blocks_nonzero: true,
            full_prediction_digest: "full".to_string(),
            nonlinear_ablated_prediction_digest: "ablated".to_string(),
            contribution_policy_digest: interaction_policy_digest_v4(),
            contribution_status: if interaction_status
                == MomentumRawFeatureQualificationStatusV4::QualifiedLearned
            {
                InteractionContributionStatusV4::MaterialInteractionContribution
            } else {
                InteractionContributionStatusV4::LinearEquivalent
            },
            audit_digest: String::new(),
        };
        audit.audit_digest = contribution_digest_v4(&audit);
        let receipts = vec![
            make_receipt_v4(&raw, raw_status, &metric_fixture(), None),
            make_receipt_v4(
                &interaction,
                interaction_status,
                &metric_fixture(),
                Some(audit.audit_digest.clone()),
            ),
            make_receipt_v4(
                &constant,
                MomentumRawFeatureQualificationStatusV4::BenchmarkQualified,
                &metric_fixture(),
                None,
            ),
        ];
        let mut value = MomentumRawFeatureFamilyV4 {
            family_version: FAMILY_VERSION_V4.to_string(),
            agent_id: AGENT_ID_V4.to_string(),
            source_snapshot_digest: "snapshot".to_string(),
            canonical_view_digest: "view".to_string(),
            frozen_mamba_closure_digest: closure_fixture().closure_digest,
            split_digest: split_fixture().split_digest,
            registration_digest: registration_fixture().registration_digest,
            participants: vec![raw, interaction, constant],
            qualification_receipts: receipts,
            interaction_contribution_audit: Some(audit),
            qualified_learned_count: usize::from(matches!(
                raw_status,
                MomentumRawFeatureQualificationStatusV4::QualifiedLearned
                    | MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent
            )) + usize::from(matches!(
                interaction_status,
                MomentumRawFeatureQualificationStatusV4::QualifiedLearned
                    | MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent
            )),
            qualified_benchmark_count: 1,
            winner_selected: false,
            final_reserve_accessed: false,
            eligible_for_active_committee: false,
            eligible_for_promotion: false,
            eligible_for_reward: false,
            family_digest: String::new(),
        };
        value.family_digest = family_digest_v4(&value);
        value
    }

    fn decision_fixture(
        family: &MomentumRawFeatureFamilyV4,
    ) -> MomentumRawFeaturePathDecisionArtifactV4 {
        derive_decision_v4(family)
    }

    fn evaluation_fixture(
        family: &MomentumRawFeatureFamilyV4,
        roster: &MomentumRawFeatureFutureRosterV4,
    ) -> MomentumRawFeatureEvaluationRegistrationV4 {
        let included = roster
            .learned_participant_digests
            .iter()
            .chain(&roster.benchmark_participant_digests)
            .collect::<BTreeSet<_>>();
        let mut value = MomentumRawFeatureEvaluationRegistrationV4 {
            registration_version: EVALUATION_VERSION_V4.to_string(),
            agent_id: AGENT_ID_V4.to_string(),
            family_digest: family.family_digest.clone(),
            roster_digest: roster.roster_digest.clone(),
            frozen_mamba_closure_digest: family.frozen_mamba_closure_digest.clone(),
            split_digest: family.split_digest.clone(),
            raw_feature_registration_digest: family.registration_digest.clone(),
            qualification_receipt_digests: sorted_unique(
                family
                    .qualification_receipts
                    .iter()
                    .filter(|item| included.contains(&item.participant_digest))
                    .map(|item| item.receipt_digest.clone())
                    .collect(),
            ),
            interaction_contribution_audit_digest: family
                .interaction_contribution_audit
                .as_ref()
                .filter(|item| included.contains(&item.participant_digest))
                .map(|item| item.audit_digest.clone()),
            source_snapshot_digest: family.source_snapshot_digest.clone(),
            source_boundary_timestamp_ms: 100,
            protected_registration_digests: vec!["protected".to_string()],
            protected_timestamp_ms: vec![101, 102, 103, 104],
            provider_finality_boundary_ms: 104,
            prior_validation_identity_digests: vec![
                "v1".to_string(),
                "v2".to_string(),
                "v3".to_string(),
            ],
            v4_final_untouched_reserve_digest: "v4-reserve".to_string(),
            minimum_accepted_timestamp_ms: 105,
            labels_hidden_until_opening: true,
            probabilities_hidden_until_opening: true,
            one_time_opening_required: true,
            winner_selection_forbidden_before_opening: true,
            active_promotion_forbidden: true,
            reward_application_forbidden: true,
            maximum_requests: 1,
            maximum_concurrency: 1,
            maximum_retries: 0,
            registration_digest: String::new(),
        };
        value.registration_digest = evaluation_digest_v4(&value);
        value
    }

    fn journal_fixture(
        family: &MomentumRawFeatureFamilyV4,
        decision: &MomentumRawFeaturePathDecisionArtifactV4,
        roster: Option<&MomentumRawFeatureFutureRosterV4>,
        evaluation: Option<&MomentumRawFeatureEvaluationRegistrationV4>,
    ) -> MomentumRawFeatureJournalV4 {
        let mut value = MomentumRawFeatureJournalV4 {
            journal_version: JOURNAL_VERSION_V4.to_string(),
            agent_id: AGENT_ID_V4.to_string(),
            closure_digest: closure_fixture().closure_digest,
            split_digest: split_fixture().split_digest,
            registration_digest: registration_fixture().registration_digest,
            family_digest: Some(family.family_digest.clone()),
            decision_digest: Some(decision.decision_digest.clone()),
            roster_digest: roster.map(|item| item.roster_digest.clone()),
            evaluation_registration_digest: evaluation.map(|item| item.registration_digest.clone()),
            preregistration_reopened_before_validation: true,
            final_reserve_accessed: false,
            prior_parameters_reused: false,
            active_registry_mutated: false,
            legacy_trainer_capability: "MomentumFrozenMambaLegacy/terminal-current-evidence-policy"
                .to_string(),
            raw_feature_trainer_capability: "MomentumRawFeatureV4/ShadowOnly".to_string(),
            status: MomentumRawFeatureExecutionStatusV4::Executed,
            journal_digest: String::new(),
        };
        value.journal_digest = journal_digest_v4(&value);
        value
    }

    #[test]
    fn pr11_terminal_invariants_remain_bound() {
        let closure = closure_fixture();
        assert_eq!(closure.v3_route_decision_digest, "v3-decision");
        assert_eq!(closure.genuine_mamba_qualified_count, 0);
    }

    #[test]
    fn protected_v1_v2_v3_bytes_ignore_v4_additions() {
        let root = temporary_root();
        fs::create_dir_all(root.join("v3")).unwrap();
        fs::create_dir_all(root.join("v4")).unwrap();
        fs::write(root.join("v3/history.pb"), b"history").unwrap();
        fs::write(root.join("v4/current.pb"), b"first").unwrap();
        let mut before = Vec::new();
        collect_protected_v4(&root, &root, &mut before).unwrap();
        fs::write(root.join("v4/current.pb"), b"second").unwrap();
        let mut after = Vec::new();
        collect_protected_v4(&root, &root, &mut after).unwrap();
        assert_eq!(before, after);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn closure_binds_exact_v1_v2_v3_history() {
        assert!(validate_closure_v4(&closure_fixture()).is_ok());
    }

    #[test]
    fn closure_scope_is_limited_to_new_identities() {
        let closure = closure_fixture();
        assert!(
            closure.reopening_requires_new_encoder_identity
                && closure.reopening_requires_new_evidence_identity
                && closure.reopening_requires_new_preregistration
        );
    }

    #[test]
    fn closed_encoder_cannot_become_v4_parent() {
        assert!(
            registration_fixture()
                .participants
                .iter()
                .all(|item| !item.participant_id.contains("Mamba"))
        );
    }

    #[test]
    fn split_is_derived_inside_v3_final_reserve() {
        let split = split_fixture();
        assert_eq!(
            split.purge_range,
            IndexRangeV0 {
                start: 240,
                end: 264
            }
        );
        assert_eq!(split.fresh_validation_range.start, split.purge_range.end);
    }

    #[test]
    fn validation_is_untouched_before_registration_reopens() {
        let root = temporary_root();
        let counts = persist_preregistration_v4(
            &root,
            &closure_fixture(),
            &split_fixture(),
            &registration_fixture(),
        )
        .unwrap();
        assert_eq!(counts, (3, 0));
        assert!(!root.join("participants").exists());
        assert!(reopen_preregistration_v4(&root).is_ok());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn final_reserve_access_is_rejected() {
        let family = family_fixture(
            MomentumRawFeatureQualificationStatusV4::QualifiedLearned,
            MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent,
        );
        let mut invalid = family.clone();
        invalid.final_reserve_accessed = true;
        invalid.family_digest = family_digest_v4(&invalid);
        assert!(validate_family_v4(&invalid).is_err());
    }

    #[test]
    fn purge_preserves_full_v3_validation_boundary() {
        let split = split_fixture();
        assert_eq!(split.purge_range.end - split.purge_range.start, 24);
    }

    #[test]
    fn exactly_two_learned_participants_are_registered() {
        let registration = registration_fixture();
        assert_eq!(registration.maximum_learned_participants, 2);
        assert_eq!(
            registration
                .participants
                .iter()
                .filter(|item| item.model_kind
                    != MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant)
                .count(),
            2
        );
    }

    #[test]
    fn result_dependent_participant_is_rejected() {
        let mut registration = registration_fixture();
        registration.participants.push(config_fixture(
            MomentumRawFeatureModelKindV4::RawFeatureLogistic,
            9,
        ));
        registration.registration_digest = registration_digest_v4(&registration);
        assert!(validate_registration_v4(&registration).is_err());
    }

    #[test]
    fn raw_logistic_uses_existing_ordered_features() {
        let config = config_fixture(MomentumRawFeatureModelKindV4::RawFeatureLogistic, 1);
        assert_eq!(
            config.input_feature_schema_digest,
            schema_digest_v4(MomentumRawFeatureModelKindV4::RawFeatureLogistic, 6)
        );
    }

    #[test]
    fn raw_logistic_rejects_prior_parameter_reuse() {
        let mut participant =
            participant_fixture(MomentumRawFeatureModelKindV4::RawFeatureLogistic);
        participant.prior_parameters_reused = true;
        participant.participant_digest = participant_digest_v4(&participant);
        assert!(validate_participant_v4(&participant).is_err());
    }

    #[test]
    fn interaction_expansion_order_is_deterministic() {
        let rows = vec![EncodedTrainingExampleV0 {
            representation: vec![2.0, 3.0],
            label: 1.0,
            snapshot_ids: vec![],
        }];
        assert_eq!(
            expand_interactions_v4(&rows).unwrap()[0].representation,
            vec![2.0, 3.0, 4.0, 9.0, 6.0]
        );
    }

    #[test]
    fn interaction_dimensions_match_schema() {
        let rows = vec![EncodedTrainingExampleV0 {
            representation: vec![1.0; 6],
            label: 0.0,
            snapshot_ids: vec![],
        }];
        assert_eq!(
            expand_interactions_v4(&rows).unwrap()[0]
                .representation
                .len(),
            27
        );
        assert_ne!(
            schema_digest_v4(MomentumRawFeatureModelKindV4::RawFeatureLogistic, 6),
            schema_digest_v4(
                MomentumRawFeatureModelKindV4::RawFeatureInteractionLogistic,
                6
            )
        );
    }

    #[test]
    fn interaction_expansion_rejects_nonfinite_values() {
        let rows = vec![EncodedTrainingExampleV0 {
            representation: vec![f32::NAN],
            label: 0.0,
            snapshot_ids: vec![],
        }];
        assert!(expand_interactions_v4(&rows).is_err());
    }

    #[test]
    fn normalizers_fit_training_only() {
        assert!(
            registration_fixture()
                .participants
                .iter()
                .filter(|item| item.model_kind
                    != MomentumRawFeatureModelKindV4::TrainingPrevalenceConstant)
                .all(|item| item.training_only_normalizer)
        );
    }

    #[test]
    fn all_participants_share_validation_timestamps() {
        let family = family_fixture(
            MomentumRawFeatureQualificationStatusV4::QualifiedLearned,
            MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent,
        );
        assert_eq!(
            family
                .participants
                .iter()
                .map(|item| &item.validation_timestamp_digest)
                .collect::<BTreeSet<_>>()
                .len(),
            1
        );
    }

    #[test]
    fn validation_parameter_updates_are_zero() {
        let family = family_fixture(
            MomentumRawFeatureQualificationStatusV4::QualifiedLearned,
            MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent,
        );
        assert!(
            family
                .qualification_receipts
                .iter()
                .all(|item| item.validation_parameter_updates == 0)
        );
    }

    #[test]
    fn constant_probability_uses_training_labels_only() {
        let labels = [0.0_f32, 1.0, 1.0, 0.0];
        let prevalence = labels.iter().sum::<f32>() / labels.len() as f32;
        assert_eq!(prevalence, 0.5);
    }

    #[test]
    fn constant_is_not_counted_as_learned() {
        let family = family_fixture(
            MomentumRawFeatureQualificationStatusV4::QualifiedLearned,
            MomentumRawFeatureQualificationStatusV4::RejectedProbabilityCollapse,
        );
        assert_eq!(family.qualified_learned_count, 1);
        assert_eq!(family.qualified_benchmark_count, 1);
    }

    #[test]
    fn learned_qualification_rejects_probability_collapse() {
        assert_eq!(
            base_qualification_v4(&metric_fixture(), &vec![0.4; 24], 24),
            MomentumRawFeatureQualificationStatusV4::RejectedProbabilityCollapse
        );
    }

    #[test]
    fn interaction_contribution_audit_is_deterministic() {
        let participant =
            participant_fixture(MomentumRawFeatureModelKindV4::RawFeatureInteractionLogistic);
        let head = LogisticPredictionHeadV0 {
            weights: vec![0.2, 0.3, 0.4],
            bias: 0.1,
        };
        let rows = vec![EncodedTrainingExampleV0 {
            representation: vec![1.0, 2.0, 3.0],
            label: 1.0,
            snapshot_ids: vec![],
        }];
        assert_eq!(
            contribution_audit_v4(&participant, &head, &rows, 1).unwrap(),
            contribution_audit_v4(&participant, &head, &rows, 1).unwrap()
        );
    }

    #[test]
    fn linear_equivalent_is_not_claimed_nonlinear() {
        let family = family_fixture(
            MomentumRawFeatureQualificationStatusV4::QualifiedLearned,
            MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent,
        );
        assert_eq!(
            family
                .interaction_contribution_audit
                .unwrap()
                .contribution_status,
            InteractionContributionStatusV4::LinearEquivalent
        );
    }

    #[test]
    fn semantic_duplicates_are_deduplicated_without_metrics() {
        let family = family_fixture(
            MomentumRawFeatureQualificationStatusV4::QualifiedLearned,
            MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent,
        );
        let roster = derive_roster_v4(&family).unwrap().0.unwrap();
        assert_eq!(roster.learned_participant_digests.len(), 1);
        assert_eq!(roster.excluded_semantic_duplicate_digests.len(), 1);
    }

    #[test]
    fn every_qualified_learned_participant_enters_roster() {
        let family = family_fixture(
            MomentumRawFeatureQualificationStatusV4::QualifiedLearned,
            MomentumRawFeatureQualificationStatusV4::QualifiedLearned,
        );
        assert_eq!(
            derive_roster_v4(&family)
                .unwrap()
                .0
                .unwrap()
                .learned_participant_digests
                .len(),
            2
        );
    }

    #[test]
    fn no_qualified_learned_creates_no_registration() {
        let family = family_fixture(
            MomentumRawFeatureQualificationStatusV4::RejectedProbabilityCollapse,
            MomentumRawFeatureQualificationStatusV4::RejectedProbabilityCollapse,
        );
        let (roster, status) = derive_roster_v4(&family).unwrap();
        assert!(roster.is_none());
        assert_eq!(
            status,
            MomentumRawFeatureRosterStatusV4::NoQualifiedLearnedParticipant
        );
    }

    #[test]
    fn all_insufficient_receipts_use_evidence_insufficient_taxonomy() {
        let mut family = family_fixture(
            MomentumRawFeatureQualificationStatusV4::RejectedInsufficientValidation,
            MomentumRawFeatureQualificationStatusV4::RejectedInsufficientValidation,
        );
        let benchmark = family
            .qualification_receipts
            .iter_mut()
            .find(|item| item.participant_role == MomentumRawFeatureRoleV4::ConstantBenchmark)
            .unwrap();
        benchmark.status = MomentumRawFeatureQualificationStatusV4::RejectedInsufficientValidation;
        benchmark.receipt_digest = receipt_digest_v4(benchmark);
        family.qualified_benchmark_count = 0;
        family.family_digest = family_digest_v4(&family);
        assert!(validate_family_v4(&family).is_ok());
        assert_eq!(
            derive_decision_v4(&family).decision,
            MomentumRawFeaturePathDecisionV4::InsufficientFreshValidation
        );
        assert_eq!(
            derive_roster_v4(&family).unwrap().1,
            MomentumRawFeatureRosterStatusV4::QualificationEvidenceInsufficient
        );
    }

    #[test]
    fn substantive_rejection_still_means_no_qualified_learner() {
        let family = family_fixture(
            MomentumRawFeatureQualificationStatusV4::RejectedProbabilityCollapse,
            MomentumRawFeatureQualificationStatusV4::RejectedProbabilityCollapse,
        );
        assert_eq!(
            derive_decision_v4(&family).decision,
            MomentumRawFeaturePathDecisionV4::NoQualifiedRawFeatureLearner
        );
    }

    #[test]
    fn validation_yield_categories_are_exclusive_and_neutral_sensitive() {
        let closes = [100.0_f32, 100.0, 100.0, 102.0, 102.0, 100.0];
        let candles = closes
            .iter()
            .enumerate()
            .map(|(index, close)| MomentumCandleV0 {
                timestamp: index as i64,
                open: *close,
                high: *close,
                low: *close,
                close: *close,
                volume: 1.0,
            })
            .collect::<Vec<_>>();
        let features = (0..candles.len())
            .map(|source_index| MomentumFeatureRowV0 {
                source_index,
                values: vec![source_index as f32],
            })
            .collect::<Vec<_>>();
        let range = IndexRangeV0 { start: 1, end: 6 };
        let config = MomentumSequenceConfigV0 {
            sequence_length: 2,
            prediction_horizon: 1,
            label_dead_zone: 0.01,
            stride: 1,
            include_neutral_labels: false,
        };
        let audit = derive_validation_yield_audit_v4(
            "snapshot",
            "label-policy",
            &range,
            3,
            &candles,
            &features,
            &config,
        )
        .unwrap();
        assert_eq!(audit.validation_index_count, 5);
        assert_eq!(audit.valid_labelled_sample_count, 2);
        assert_eq!(audit.neutral_excluded_count, 2);
        assert_eq!(audit.horizon_unavailable_count, 0);
        assert_eq!(audit.feature_unavailable_count, 1);
        assert!(!audit.substantive_qualification_possible);

        let include_neutral = MomentumSequenceConfigV0 {
            include_neutral_labels: true,
            ..config
        };
        let inclusive = derive_validation_yield_audit_v4(
            "snapshot",
            "label-policy-with-neutral",
            &range,
            3,
            &candles,
            &features,
            &include_neutral,
        )
        .unwrap();
        assert_eq!(inclusive.valid_labelled_sample_count, 4);
        assert_eq!(inclusive.neutral_excluded_count, 0);
        assert_eq!(inclusive.feature_unavailable_count, 1);
        assert!(inclusive.substantive_qualification_possible);
    }

    #[test]
    fn no_winner_is_selected() {
        assert!(
            !family_fixture(
                MomentumRawFeatureQualificationStatusV4::QualifiedLearned,
                MomentumRawFeatureQualificationStatusV4::QualifiedLearned
            )
            .winner_selected
        );
    }

    #[test]
    fn future_registration_preserves_exclusions() {
        let family = family_fixture(
            MomentumRawFeatureQualificationStatusV4::QualifiedLearned,
            MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent,
        );
        let roster = derive_roster_v4(&family).unwrap().0.unwrap();
        let evaluation = evaluation_fixture(&family, &roster);
        assert!(validate_evaluation_v4(&evaluation, &family, &roster).is_ok());
    }

    #[test]
    fn final_reserve_remains_unopened_after_registration() {
        let counters = zero_safety_counters_v4();
        assert_eq!(
            counters.final_reserve_row_reads + counters.final_reserve_label_reads,
            0
        );
    }

    #[test]
    fn prospective_replay_has_no_v4_mutation_authority() {
        let counters = zero_safety_counters_v4();
        assert_eq!(
            counters.prospective_row_reads + counters.prospective_label_openings,
            0
        );
    }

    #[test]
    fn reward_and_penalty_applications_are_zero() {
        let counters = zero_safety_counters_v4();
        assert_eq!(
            counters.reward_applications + counters.penalty_applications,
            0
        );
    }

    #[test]
    fn protobuf_corruption_rejects() {
        assert!(decode_momentum_frozen_mamba_closure_protobuf_v4(&[0xff]).is_err());
        assert!(decode_momentum_raw_feature_split_protobuf_v4(&[0xff]).is_err());
        assert!(decode_momentum_validation_yield_audit_protobuf_v4(&[0xff]).is_err());
        assert!(decode_momentum_raw_feature_registration_protobuf_v4(&[0xff]).is_err());
    }

    #[test]
    fn repeated_execution_is_idempotent() {
        let root = temporary_root();
        let first = persist_preregistration_v4(
            &root,
            &closure_fixture(),
            &split_fixture(),
            &registration_fixture(),
        )
        .unwrap();
        let second = persist_preregistration_v4(
            &root,
            &closure_fixture(),
            &split_fixture(),
            &registration_fixture(),
        )
        .unwrap();
        assert_eq!(first, (3, 0));
        assert_eq!(second, (0, 3));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn all_manual_protobuf_contracts_round_trip() {
        let closure = closure_fixture();
        let split = split_fixture();
        let validation_yield_audit = validation_yield_audit_fixture();
        let registration = registration_fixture();
        let family = family_fixture(
            MomentumRawFeatureQualificationStatusV4::QualifiedLearned,
            MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent,
        );
        let decision = decision_fixture(&family);
        let roster = derive_roster_v4(&family).unwrap().0.unwrap();
        let evaluation = evaluation_fixture(&family, &roster);
        let journal = journal_fixture(&family, &decision, Some(&roster), Some(&evaluation));
        assert_eq!(
            decode_momentum_frozen_mamba_closure_protobuf_v4(
                &encode_momentum_frozen_mamba_closure_protobuf_v4(&closure).unwrap()
            )
            .unwrap(),
            closure
        );
        assert_eq!(
            decode_momentum_raw_feature_split_protobuf_v4(
                &encode_momentum_raw_feature_split_protobuf_v4(&split).unwrap()
            )
            .unwrap(),
            split
        );
        assert_eq!(
            decode_momentum_validation_yield_audit_protobuf_v4(
                &encode_momentum_validation_yield_audit_protobuf_v4(&validation_yield_audit)
                    .unwrap()
            )
            .unwrap(),
            validation_yield_audit
        );
        assert_eq!(
            decode_momentum_raw_feature_registration_protobuf_v4(
                &encode_momentum_raw_feature_registration_protobuf_v4(&registration).unwrap()
            )
            .unwrap(),
            registration
        );
        assert_eq!(
            decode_frozen_candidate_participant_protobuf_v4(
                &encode_frozen_candidate_participant_protobuf_v4(&family.participants[0]).unwrap()
            )
            .unwrap(),
            family.participants[0]
        );
        assert_eq!(
            decode_momentum_raw_feature_qualification_protobuf_v4(
                &encode_momentum_raw_feature_qualification_protobuf_v4(
                    &family.qualification_receipts[0]
                )
                .unwrap()
            )
            .unwrap(),
            family.qualification_receipts[0]
        );
        assert_eq!(
            decode_momentum_interaction_contribution_protobuf_v4(
                &encode_momentum_interaction_contribution_protobuf_v4(
                    family.interaction_contribution_audit.as_ref().unwrap()
                )
                .unwrap()
            )
            .unwrap(),
            *family.interaction_contribution_audit.as_ref().unwrap()
        );
        assert_eq!(
            decode_momentum_raw_feature_family_protobuf_v4(
                &encode_momentum_raw_feature_family_protobuf_v4(&family).unwrap()
            )
            .unwrap(),
            family
        );
        assert_eq!(
            decode_momentum_raw_feature_decision_protobuf_v4(
                &encode_momentum_raw_feature_decision_protobuf_v4(&decision, &family).unwrap(),
                &family
            )
            .unwrap(),
            decision
        );
        assert_eq!(
            decode_momentum_raw_feature_roster_protobuf_v4(
                &encode_momentum_raw_feature_roster_protobuf_v4(&roster, &family).unwrap(),
                &family
            )
            .unwrap(),
            roster
        );
        assert_eq!(
            decode_momentum_raw_feature_evaluation_protobuf_v4(
                &encode_momentum_raw_feature_evaluation_protobuf_v4(&evaluation, &family, &roster)
                    .unwrap(),
                &family,
                &roster
            )
            .unwrap(),
            evaluation
        );
        assert_eq!(
            decode_momentum_raw_feature_journal_protobuf_v4(
                &encode_momentum_raw_feature_journal_protobuf_v4(&journal).unwrap()
            )
            .unwrap(),
            journal
        );
    }

    #[test]
    fn network_and_authority_counters_are_zero() {
        let counters = zero_safety_counters_v4();
        assert_eq!(counters.active_committee_count, 3);
        assert_eq!(
            counters.network_requests
                + counters.transport_constructions
                + counters.credential_reads
                + counters.prospective_row_reads
                + counters.prospective_label_openings
                + counters.historical_test_reads
                + counters.future_evaluation_reads
                + counters.final_reserve_row_reads
                + counters.final_reserve_label_reads
                + counters.active_model_changes
                + counters.chair_decisions
                + counters.votes
                + counters.reward_applications
                + counters.penalty_applications
                + counters.voice_changes
                + counters.cooldowns_started
                + counters.promotions
                + counters.quarantines
                + counters.executions,
            0
        );
    }

    #[test]
    fn trainer_registry_boundary_remains_shadow_only() {
        let family = family_fixture(
            MomentumRawFeatureQualificationStatusV4::QualifiedLearned,
            MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent,
        );
        let decision = decision_fixture(&family);
        let roster = derive_roster_v4(&family).unwrap().0.unwrap();
        let evaluation = evaluation_fixture(&family, &roster);
        let journal = journal_fixture(&family, &decision, Some(&roster), Some(&evaluation));
        assert!(!journal.active_registry_mutated);
        assert_eq!(
            journal.raw_feature_trainer_capability,
            "MomentumRawFeatureV4/ShadowOnly"
        );
    }
}
