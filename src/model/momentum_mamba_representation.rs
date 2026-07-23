//! Offline, additive representation-path probes and preregistered V3 routes.
//!
//! This module has no network, active-committee, reward, voting, or execution authority.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{core::stable_hash_string, data::DataSnapshot, league::canonical_current_agent_states};

use super::agent_learning_session::{
    AgentPrivateLearningArtifactWriteStatusV0, atomic_write_verified_v0,
};
use super::momentum_mamba_repair::{
    MomentumCandidateFamilyV2, MomentumMambaCollapseAuditV2, MomentumMambaCollapseRootCauseV2,
    MomentumMambaRepairCapabilityStatusV2, MomentumMambaRepairExecutionStatusV2,
    MomentumMambaRepairSplitV2, ParticipantQualificationRoleV2, V1FrozenStateV2,
    ValidationQualificationStatusV2, candles_from_snapshot_prefix,
    decode_momentum_candidate_family_protobuf_v2, decode_momentum_mamba_collapse_audit_protobuf_v2,
    decode_momentum_mamba_repair_journal_protobuf_v2,
    decode_momentum_mamba_repair_registration_protobuf_v2,
    decode_momentum_mamba_repair_split_protobuf_v2, examples_in_range, load_v1_frozen_state_v2,
};
use super::{
    AgentPrivateLearningRunModeV0, ConstantProbabilityBaselineV0, EncodedTrainingExampleV0,
    EvaluationMetricsV0, FeatureNormalizerV0, HeadTrainingConfigV0, IndexRangeV0,
    LinearMomentumBaselineV0, LogisticPredictionHeadV0, ModelAgentDeploymentStatus,
    MomentumLearningCampaignConfigV0, ProtectedEvaluationReservationV1, RepresentationNormalizerV0,
    SequenceExampleV0, SequencePooling, apply_sgd_v0, brier_loss_and_gradients_v0,
    build_momentum_features_v0, build_momentum_sequence_examples_v0, evaluate_head_v0,
    frozen_mamba3_encoder_from_seed_v0,
};

const AGENT_ID_V3: &str = "momentum_trend_fast";
const AUDIT_VERSION_V3: &str = "momentum-representation-path-audit-v3";
const SPLIT_VERSION_V3: &str = "momentum-representation-split-v3";
const REGISTRATION_VERSION_V3: &str = "momentum-representation-registration-v3";
const PARTICIPANT_VERSION_V3: &str = "frozen-candidate-participant-v3";
const RECEIPT_VERSION_V3: &str = "momentum-representation-qualification-v3";
const FAMILY_VERSION_V3: &str = "momentum-representation-family-v3";
const DECISION_VERSION_V3: &str = "momentum-representation-route-decision-v3";
const ROSTER_VERSION_V3: &str = "momentum-representation-roster-v3";
const EVALUATION_VERSION_V3: &str = "momentum-representation-evaluation-registration-v3";
const JOURNAL_VERSION_V3: &str = "momentum-representation-journal-v3";
const MAXIMUM_VARIANTS_V3: usize = 4;
const MAMBA_MATERIAL_EFFECT_BITS_V3: u32 = 0.001_f32.to_bits();
const MAMBA_DETECTABLE_EFFECT_BITS_V3: u32 = 0.000001_f32.to_bits();

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumFrozenMambaRepairStageV3 {
    V1OriginalCollapsed,
    V2HeadOnlyRepairExhausted,
    V3RepresentationPathPending,
    V3RepresentationPathViable,
    V3ResidualHybridViable,
    V3MambaContributionAbsent,
    V3FrozenMambaPathRejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MomentumRepresentationProbeKindV3 {
    RawFeatureLinearProbe,
    MambaLastOutputProbe,
    MambaMeanOutputProbe,
    MambaLastMeanConcatProbe,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumRepresentationProbeStatusV3 {
    FiniteUsable,
    LowVariance,
    LowEffectiveRank,
    SingleSidedPrediction,
    NonCollapsedPrediction,
    NumericalFailure,
    InsufficientEvidence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRepresentationProbeV3 {
    pub probe_kind: MomentumRepresentationProbeKindV3,
    pub source_snapshot_digest: String,
    pub consumed_range_digest: String,
    pub feature_policy_digest: String,
    pub encoder_digest: Option<String>,
    pub representation_diagnostic_digest: String,
    pub private_probe_metric_digest: String,
    pub status: MomentumRepresentationProbeStatusV3,
    pub probe_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRepresentationPathAuditV3 {
    pub audit_version: String,
    pub v1_family_digest: String,
    pub v2_family_digest: String,
    pub v2_collapse_audit_digest: String,
    pub probes: Vec<MomentumRepresentationProbeV3>,
    pub head_only_repair_exhausted: bool,
    pub fresh_v3_validation_accessed: bool,
    pub audit_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRepresentationSplitV3 {
    pub split_version: String,
    pub source_snapshot_digest: String,
    pub v1_usage_ledger_digest: String,
    pub v2_split_digest: String,
    pub training_range: IndexRangeV0,
    pub purge_range: IndexRangeV0,
    pub fresh_validation_range: IndexRangeV0,
    pub final_reserved_range: IndexRangeV0,
    pub minimum_validation_samples: usize,
    pub minimum_final_reserved_samples: usize,
    pub prior_validation_overlap_count: usize,
    pub prospective_overlap_count: usize,
    pub future_evaluation_overlap_count: usize,
    pub split_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MomentumRepresentationInputKindV3 {
    MambaLastOutput,
    MambaMeanOutput,
    MambaLastMeanConcat,
    MambaRawFeatureResidual,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRepresentationVariantConfigV3 {
    pub variant_id: String,
    pub input_kind: MomentumRepresentationInputKindV3,
    pub pooling_policy: String,
    pub raw_feature_residual_enabled: bool,
    pub head_kind: String,
    pub learning_rate_bits: u32,
    pub l2_regularization_bits: u32,
    pub maximum_epochs: usize,
    pub initialization_seed: u64,
    pub encoder_frozen: bool,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub training_policy_digest: String,
    pub variant_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRepresentationRegistrationV3 {
    pub registration_version: String,
    pub agent_id: String,
    pub source_snapshot_digest: String,
    pub canonical_intent_digest: String,
    pub canonical_view_digest: String,
    pub representation_audit_digest: String,
    pub split_digest: String,
    pub variants: Vec<MomentumRepresentationVariantConfigV3>,
    pub maximum_variants: usize,
    pub contribution_policy_digest: String,
    pub fresh_validation_hidden: bool,
    pub historical_test_forbidden: bool,
    pub future_evaluation_forbidden: bool,
    pub winner_selection_forbidden: bool,
    pub active_promotion_forbidden: bool,
    pub reward_application_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MambaContributionStatusV3 {
    NotApplicable,
    MaterialContribution,
    DetectableButBelowPolicy,
    NoDetectableContribution,
    RawFeatureDominated,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MambaContributionAuditV3 {
    pub participant_digest: String,
    pub mamba_parameter_block_digest: String,
    pub raw_parameter_block_digest: String,
    pub mamba_block_nonzero: bool,
    pub raw_block_nonzero: bool,
    pub full_prediction_digest: String,
    pub mamba_ablated_prediction_digest: String,
    pub raw_ablated_prediction_digest: String,
    pub mamba_ablation_effect_status: String,
    pub raw_ablation_effect_status: String,
    pub contribution_policy_digest: String,
    pub contribution_status: MambaContributionStatusV3,
    pub audit_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumResidualQualificationV3 {
    QualifiedMambaContributingHybrid,
    QualifiedRawFallbackNotMamba,
    RejectedProbabilityCollapse,
    RejectedNumericalFailure,
    RejectedContributionInvariant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumRepresentationQualificationStatusV3 {
    QualifiedMambaOnly,
    QualifiedMambaContributingHybrid,
    QualifiedRawFallbackNotMamba,
    ComparatorQualified,
    BenchmarkQualified,
    RejectedInsufficientValidation,
    RejectedRepresentationInvariant,
    RejectedProbabilityCollapse,
    RejectedNumericalFailure,
    RejectedContributionInvariant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumRepresentationParticipantRoleV3 {
    MambaOnly,
    MambaResidualHybrid,
    LinearComparator,
    ConstantBenchmark,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenCandidateParticipantV3 {
    pub participant_version: String,
    pub participant_id: String,
    pub participant_role: MomentumRepresentationParticipantRoleV3,
    pub model_kind: String,
    pub input_kind: String,
    pub variant_digest: Option<String>,
    pub source_snapshot_digest: String,
    pub training_range_digest: String,
    pub fresh_validation_range_digest: String,
    pub validation_timestamp_digest: String,
    pub model_artifact_digest: String,
    pub parameter_digest: String,
    pub feature_normalizer_digest: String,
    pub representation_normalizer_digest: String,
    pub encoder_digest: Option<String>,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub training_policy_digest: String,
    pub initialization_digest: String,
    pub warm_start: bool,
    pub v1_head_reused: bool,
    pub v2_head_reused: bool,
    pub fresh_deterministic_initialization: bool,
    pub encoder_frozen: bool,
    pub deployment_status: ModelAgentDeploymentStatus,
    pub participant_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRepresentationQualificationReceiptV3 {
    pub receipt_version: String,
    pub participant_id: String,
    pub participant_digest: String,
    pub input_kind: String,
    pub fresh_validation_range_digest: String,
    pub qualification_policy_digest: String,
    pub private_metric_digest: String,
    pub contribution_audit_digest: Option<String>,
    pub status: MomentumRepresentationQualificationStatusV3,
    pub validation_parameter_updates: usize,
    pub historical_test_reads: usize,
    pub future_evaluation_reads: usize,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRepresentationFamilyV3 {
    pub family_version: String,
    pub agent_id: String,
    pub source_snapshot_digest: String,
    pub canonical_view_digest: String,
    pub representation_audit_digest: String,
    pub split_digest: String,
    pub registration_digest: String,
    pub participants: Vec<FrozenCandidateParticipantV3>,
    pub qualification_receipts: Vec<MomentumRepresentationQualificationReceiptV3>,
    pub contribution_audits: Vec<MambaContributionAuditV3>,
    pub qualified_mamba_only_count: usize,
    pub qualified_mamba_hybrid_count: usize,
    pub qualified_raw_fallback_count: usize,
    pub qualified_comparator_count: usize,
    pub winner_selected: bool,
    pub historical_test_accessed: bool,
    pub eligible_for_active_committee: bool,
    pub eligible_for_promotion: bool,
    pub eligible_for_reward: bool,
    pub family_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumRepresentationRouteDecisionV3 {
    FrozenMambaOnlyViable,
    MambaResidualHybridViable,
    RawFeatureFallbackOnly,
    FrozenMambaAddsNoIncrementalSignal,
    AllRepresentationRoutesCollapsed,
    InsufficientFreshValidation,
    TechnicalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRepresentationRouteDecisionArtifactV3 {
    pub decision_version: String,
    pub v1_family_digest: String,
    pub v2_family_digest: String,
    pub v3_family_digest: String,
    pub qualified_mamba_only_digests: Vec<String>,
    pub qualified_mamba_hybrid_digests: Vec<String>,
    pub raw_fallback_digests: Vec<String>,
    pub rejected_route_digests: Vec<String>,
    pub further_head_only_repair_forbidden: bool,
    pub further_frozen_representation_sweep_forbidden: bool,
    pub decision: MomentumRepresentationRouteDecisionV3,
    pub decision_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRepresentationFutureRosterV3 {
    pub roster_version: String,
    pub family_digest: String,
    pub decision_digest: String,
    pub qualified_genuine_mamba_digests: Vec<String>,
    pub qualified_comparator_digests: Vec<String>,
    pub excluded_participant_digests: Vec<String>,
    pub inclusion_policy_digest: String,
    pub roster_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumRepresentationRosterStatusV3 {
    Registered,
    FrozenMambaRepresentationPathRejected,
    InsufficientComparators,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRepresentationEvaluationRegistrationV3 {
    pub registration_version: String,
    pub agent_id: String,
    pub family_digest: String,
    pub roster_digest: String,
    pub decision_digest: String,
    pub qualification_receipt_digests: Vec<String>,
    pub contribution_audit_digests: Vec<String>,
    pub source_snapshot_digest: String,
    pub source_boundary_timestamp_ms: u64,
    pub protected_registration_digests: Vec<String>,
    pub protected_timestamp_ms: Vec<u64>,
    pub prior_validation_and_reserved_range_digests: Vec<String>,
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
    pub registration_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumRepresentationEvaluationStatusV3 {
    Registered,
    FrozenMambaRepresentationPathRejected,
    InsufficientComparators,
    SafetyContractInvalid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumRepresentationExecutionStatusV3 {
    Planned,
    Executed,
    AlreadyExecuted,
    InsufficientFreshValidation,
    TechnicalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRepresentationJournalV3 {
    pub journal_version: String,
    pub agent_id: String,
    pub repair_stage: MomentumFrozenMambaRepairStageV3,
    pub representation_audit_digest: String,
    pub split_digest: String,
    pub registration_digest: String,
    pub family_digest: Option<String>,
    pub decision_digest: Option<String>,
    pub roster_digest: Option<String>,
    pub evaluation_registration_digest: Option<String>,
    pub prior_validation_used_for_v3_qualification: bool,
    pub final_reserve_accessed: bool,
    pub warm_start: bool,
    pub v1_head_reused: bool,
    pub v2_head_reused: bool,
    pub fresh_deterministic_initialization: bool,
    pub status: MomentumRepresentationExecutionStatusV3,
    pub journal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRepresentationSafetyCountersV3 {
    pub network_requests: usize,
    pub transport_constructions: usize,
    pub credential_reads: usize,
    pub prospective_row_reads: usize,
    pub prospective_label_openings: usize,
    pub historical_test_reads: usize,
    pub future_evaluation_reads: usize,
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
pub struct MomentumRepresentationReportV3 {
    pub report_version: String,
    pub mode: AgentPrivateLearningRunModeV0,
    pub status: MomentumRepresentationExecutionStatusV3,
    pub repair_stage: MomentumFrozenMambaRepairStageV3,
    pub representation_audit: Option<MomentumRepresentationPathAuditV3>,
    pub split: Option<MomentumRepresentationSplitV3>,
    pub registration: Option<MomentumRepresentationRegistrationV3>,
    pub family: Option<MomentumRepresentationFamilyV3>,
    pub decision: Option<MomentumRepresentationRouteDecisionArtifactV3>,
    pub roster: Option<MomentumRepresentationFutureRosterV3>,
    pub roster_status: MomentumRepresentationRosterStatusV3,
    pub evaluation_registration: Option<MomentumRepresentationEvaluationRegistrationV3>,
    pub evaluation_registration_status: MomentumRepresentationEvaluationStatusV3,
    pub journal: Option<MomentumRepresentationJournalV3>,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub storage_failure_count: usize,
    pub protected_artifacts_unchanged: bool,
    pub active_state_unchanged: bool,
    pub safety_counters: MomentumRepresentationSafetyCountersV3,
    pub report_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VarianceClassV3 {
    Adequate,
    Low,
    NearConstant,
    NumericalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RankClassV3 {
    Adequate,
    Low,
    RankOneOrLess,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RepresentationDiagnosticV3 {
    finite: bool,
    variance: VarianceClassV3,
    effective_rank: RankClassV3,
    redundant_dimension_count: usize,
    digest: String,
}

#[derive(Clone, Debug)]
struct FrozenHistoryV3 {
    v1: V1FrozenStateV2,
    v2_audit: MomentumMambaCollapseAuditV2,
    v2_split: MomentumMambaRepairSplitV2,
    v2_family: MomentumCandidateFamilyV2,
}

#[derive(Clone, Debug)]
struct RepresentationExperimentV3 {
    family: MomentumRepresentationFamilyV3,
    decision: MomentumRepresentationRouteDecisionArtifactV3,
    roster: Option<MomentumRepresentationFutureRosterV3>,
    roster_status: MomentumRepresentationRosterStatusV3,
    evaluation_registration: Option<MomentumRepresentationEvaluationRegistrationV3>,
    evaluation_registration_status: MomentumRepresentationEvaluationStatusV3,
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn range_digest_v3(label: &str, range: &IndexRangeV0) -> String {
    stable_hash_string(&format!(
        "momentum-representation-range-v3:{label}:{}:{}",
        range.start, range.end
    ))
}

fn ranges_overlap_v3(left: &IndexRangeV0, right: &IndexRangeV0) -> usize {
    left.end
        .min(right.end)
        .saturating_sub(left.start.max(right.start))
}

fn contribution_policy_digest_v3() -> String {
    stable_hash_string(&format!(
        "mamba-contribution-policy-v3:mean-absolute-probability-effect:{}:{}:deterministic-block-zero-ablation",
        MAMBA_MATERIAL_EFFECT_BITS_V3, MAMBA_DETECTABLE_EFFECT_BITS_V3
    ))
}

fn digest_without_identity<T: std::fmt::Debug>(label: &str, value: &T) -> String {
    stable_hash_string(&format!("{label}:{value:?}"))
}

fn probe_digest_v3(value: &MomentumRepresentationProbeV3) -> String {
    let mut canonical = value.clone();
    canonical.probe_digest.clear();
    digest_without_identity("momentum-representation-probe-v3", &canonical)
}

fn audit_digest_v3(value: &MomentumRepresentationPathAuditV3) -> String {
    let mut canonical = value.clone();
    canonical.audit_digest.clear();
    digest_without_identity("momentum-representation-audit-v3", &canonical)
}

fn split_digest_v3(value: &MomentumRepresentationSplitV3) -> String {
    let mut canonical = value.clone();
    canonical.split_digest.clear();
    digest_without_identity("momentum-representation-split-v3", &canonical)
}

fn variant_digest_v3(value: &MomentumRepresentationVariantConfigV3) -> String {
    let mut canonical = value.clone();
    canonical.variant_digest.clear();
    digest_without_identity("momentum-representation-variant-v3", &canonical)
}

fn registration_digest_v3(value: &MomentumRepresentationRegistrationV3) -> String {
    let mut canonical = value.clone();
    canonical.registration_digest.clear();
    digest_without_identity("momentum-representation-registration-v3", &canonical)
}

fn participant_digest_v3(value: &FrozenCandidateParticipantV3) -> String {
    let mut canonical = value.clone();
    canonical.participant_digest.clear();
    digest_without_identity("momentum-representation-participant-v3", &canonical)
}

fn receipt_digest_v3(value: &MomentumRepresentationQualificationReceiptV3) -> String {
    let mut canonical = value.clone();
    canonical.receipt_digest.clear();
    digest_without_identity("momentum-representation-receipt-v3", &canonical)
}

fn contribution_digest_v3(value: &MambaContributionAuditV3) -> String {
    let mut canonical = value.clone();
    canonical.audit_digest.clear();
    digest_without_identity("mamba-contribution-audit-v3", &canonical)
}

fn family_digest_v3(value: &MomentumRepresentationFamilyV3) -> String {
    let mut canonical = value.clone();
    canonical.family_digest.clear();
    digest_without_identity("momentum-representation-family-v3", &canonical)
}

fn decision_digest_v3(value: &MomentumRepresentationRouteDecisionArtifactV3) -> String {
    let mut canonical = value.clone();
    canonical.decision_digest.clear();
    digest_without_identity("momentum-representation-decision-v3", &canonical)
}

fn roster_digest_v3(value: &MomentumRepresentationFutureRosterV3) -> String {
    let mut canonical = value.clone();
    canonical.roster_digest.clear();
    digest_without_identity("momentum-representation-roster-v3", &canonical)
}

fn evaluation_digest_v3(value: &MomentumRepresentationEvaluationRegistrationV3) -> String {
    let mut canonical = value.clone();
    canonical.registration_digest.clear();
    digest_without_identity("momentum-representation-evaluation-v3", &canonical)
}

fn journal_digest_v3(value: &MomentumRepresentationJournalV3) -> String {
    let mut canonical = value.clone();
    canonical.journal_digest.clear();
    digest_without_identity("momentum-representation-journal-v3", &canonical)
}

fn zero_safety_counters_v3() -> MomentumRepresentationSafetyCountersV3 {
    MomentumRepresentationSafetyCountersV3 {
        network_requests: 0,
        transport_constructions: 0,
        credential_reads: 0,
        prospective_row_reads: 0,
        prospective_label_openings: 0,
        historical_test_reads: 0,
        future_evaluation_reads: 0,
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
        active_committee_count: canonical_current_agent_states().len(),
    }
}

fn validate_probe_v3(value: &MomentumRepresentationProbeV3) -> Result<(), String> {
    if value.source_snapshot_digest.is_empty()
        || value.consumed_range_digest.is_empty()
        || value.feature_policy_digest.is_empty()
        || value.representation_diagnostic_digest.is_empty()
        || value.private_probe_metric_digest.is_empty()
        || value.probe_digest != probe_digest_v3(value)
    {
        return Err("V3 representation probe rejected".to_string());
    }
    match value.probe_kind {
        MomentumRepresentationProbeKindV3::RawFeatureLinearProbe => {
            if value.encoder_digest.is_some() {
                return Err("raw probe encoder binding rejected".to_string());
            }
        }
        _ if value.encoder_digest.is_none() => {
            return Err("Mamba probe encoder binding rejected".to_string());
        }
        _ => {}
    }
    Ok(())
}

fn validate_audit_v3(value: &MomentumRepresentationPathAuditV3) -> Result<(), String> {
    if value.audit_version != AUDIT_VERSION_V3
        || value.v1_family_digest.is_empty()
        || value.v2_family_digest.is_empty()
        || value.v2_collapse_audit_digest.is_empty()
        || value.probes.len() != 4
        || !value.head_only_repair_exhausted
        || value.fresh_v3_validation_accessed
        || value.audit_digest != audit_digest_v3(value)
    {
        return Err("V3 representation audit rejected".to_string());
    }
    let kinds = value
        .probes
        .iter()
        .map(|probe| probe.probe_kind)
        .collect::<BTreeSet<_>>();
    let source_digests = value
        .probes
        .iter()
        .map(|probe| probe.source_snapshot_digest.as_str())
        .collect::<BTreeSet<_>>();
    let consumed_range_digests = value
        .probes
        .iter()
        .map(|probe| probe.consumed_range_digest.as_str())
        .collect::<BTreeSet<_>>();
    let feature_policy_digests = value
        .probes
        .iter()
        .map(|probe| probe.feature_policy_digest.as_str())
        .collect::<BTreeSet<_>>();
    let representation_digests = value
        .probes
        .iter()
        .map(|probe| probe.representation_diagnostic_digest.as_str())
        .collect::<BTreeSet<_>>();
    let encoder_digests = value
        .probes
        .iter()
        .filter_map(|probe| probe.encoder_digest.as_deref())
        .collect::<BTreeSet<_>>();
    if kinds.len() != 4
        || source_digests.len() != 1
        || consumed_range_digests.len() != 1
        || feature_policy_digests.len() != 1
        || representation_digests.len() != 4
        || encoder_digests.len() != 1
    {
        return Err("V3 representation probes incomplete".to_string());
    }
    for probe in &value.probes {
        validate_probe_v3(probe)?;
    }
    Ok(())
}

fn validate_split_v3(value: &MomentumRepresentationSplitV3) -> Result<(), String> {
    let config = MomentumLearningCampaignConfigV0::default();
    let expected_purge_count = config
        .feature_config
        .minimum_history()
        .map_err(|_| "V3 purge validation unavailable".to_string())?
        .checked_sub(1)
        .and_then(|count| count.checked_add(config.sequence_config.sequence_length - 1))
        .and_then(|count| count.checked_add(config.sequence_config.prediction_horizon))
        .ok_or_else(|| "V3 purge validation overflow".to_string())?;
    if value.split_version != SPLIT_VERSION_V3
        || value.source_snapshot_digest.is_empty()
        || value.v1_usage_ledger_digest.is_empty()
        || value.v2_split_digest.is_empty()
        || value.training_range.start != 0
        || value.training_range.end != value.purge_range.start
        || value.purge_range.end != value.fresh_validation_range.start
        || value.fresh_validation_range.end != value.final_reserved_range.start
        || value.fresh_validation_range.end <= value.fresh_validation_range.start
        || value.final_reserved_range.end <= value.final_reserved_range.start
        || value.minimum_validation_samples != config.validation_rows
        || value.purge_range.end - value.purge_range.start != expected_purge_count
        || value.fresh_validation_range.end - value.fresh_validation_range.start
            != value.minimum_validation_samples
        || value.minimum_final_reserved_samples
            != value.minimum_validation_samples.saturating_mul(2)
        || value.final_reserved_range.end - value.final_reserved_range.start
            != value.minimum_final_reserved_samples
        || value.prior_validation_overlap_count != 0
        || value.prospective_overlap_count != 0
        || value.future_evaluation_overlap_count != 0
        || value.split_digest != split_digest_v3(value)
    {
        return Err("V3 representation split rejected".to_string());
    }
    Ok(())
}

fn validate_variant_v3(value: &MomentumRepresentationVariantConfigV3) -> Result<(), String> {
    let learning_rate = f32::from_bits(value.learning_rate_bits);
    let l2 = f32::from_bits(value.l2_regularization_bits);
    if value.variant_id.is_empty()
        || value.pooling_policy.is_empty()
        || value.head_kind != "LogisticPredictionHeadV0"
        || !learning_rate.is_finite()
        || learning_rate <= 0.0
        || !l2.is_finite()
        || l2 < 0.0
        || value.maximum_epochs == 0
        || !value.encoder_frozen
        || value.feature_policy_digest.is_empty()
        || value.label_policy_digest.is_empty()
        || value.training_policy_digest.is_empty()
        || value.raw_feature_residual_enabled
            != (value.input_kind == MomentumRepresentationInputKindV3::MambaRawFeatureResidual)
        || value.variant_digest != variant_digest_v3(value)
    {
        return Err("V3 representation variant rejected".to_string());
    }
    Ok(())
}

fn validate_registration_v3(value: &MomentumRepresentationRegistrationV3) -> Result<(), String> {
    if value.registration_version != REGISTRATION_VERSION_V3
        || value.agent_id != AGENT_ID_V3
        || value.source_snapshot_digest.is_empty()
        || value.canonical_intent_digest.is_empty()
        || value.canonical_view_digest.is_empty()
        || value.representation_audit_digest.is_empty()
        || value.split_digest.is_empty()
        || value.maximum_variants != MAXIMUM_VARIANTS_V3
        || value.variants.len() != MAXIMUM_VARIANTS_V3
        || value.contribution_policy_digest != contribution_policy_digest_v3()
        || !value.fresh_validation_hidden
        || !value.historical_test_forbidden
        || !value.future_evaluation_forbidden
        || !value.winner_selection_forbidden
        || !value.active_promotion_forbidden
        || !value.reward_application_forbidden
        || value.registration_digest != registration_digest_v3(value)
    {
        return Err("V3 representation registration rejected".to_string());
    }
    let kinds = value
        .variants
        .iter()
        .map(|variant| variant.input_kind)
        .collect::<BTreeSet<_>>();
    if kinds.len() != MAXIMUM_VARIANTS_V3 {
        return Err("V3 representation route set rejected".to_string());
    }
    let head_policies = value
        .variants
        .iter()
        .map(|variant| {
            (
                variant.learning_rate_bits,
                variant.l2_regularization_bits,
                variant.maximum_epochs,
            )
        })
        .collect::<BTreeSet<_>>();
    let initialization_seeds = value
        .variants
        .iter()
        .map(|variant| variant.initialization_seed)
        .collect::<BTreeSet<_>>();
    let feature_policies = value
        .variants
        .iter()
        .map(|variant| variant.feature_policy_digest.as_str())
        .collect::<BTreeSet<_>>();
    let label_policies = value
        .variants
        .iter()
        .map(|variant| variant.label_policy_digest.as_str())
        .collect::<BTreeSet<_>>();
    if head_policies.len() != 1
        || initialization_seeds.len() != MAXIMUM_VARIANTS_V3
        || feature_policies.len() != 1
        || label_policies.len() != 1
    {
        return Err("V3 fixed route head policy rejected".to_string());
    }
    for variant in &value.variants {
        validate_variant_v3(variant)?;
        let expected_identity = match variant.input_kind {
            MomentumRepresentationInputKindV3::MambaLastOutput => {
                ("last-output-control", "LastOutput", false)
            }
            MomentumRepresentationInputKindV3::MambaMeanOutput => {
                ("mean-output", "MeanOutput", false)
            }
            MomentumRepresentationInputKindV3::MambaLastMeanConcat => {
                ("last-mean-concat", "LastOutput+MeanOutput", false)
            }
            MomentumRepresentationInputKindV3::MambaRawFeatureResidual => (
                "raw-feature-residual",
                "LastOutput+SequenceEndRawFeatureResidual",
                true,
            ),
        };
        if variant.variant_id != expected_identity.0
            || variant.pooling_policy != expected_identity.1
            || variant.raw_feature_residual_enabled != expected_identity.2
        {
            return Err("V3 fixed representation route identity rejected".to_string());
        }
    }
    Ok(())
}

fn validate_participant_v3(value: &FrozenCandidateParticipantV3) -> Result<(), String> {
    let learned = matches!(
        value.participant_role,
        MomentumRepresentationParticipantRoleV3::MambaOnly
            | MomentumRepresentationParticipantRoleV3::MambaResidualHybrid
    );
    if value.participant_version != PARTICIPANT_VERSION_V3
        || value.participant_id.is_empty()
        || value.model_kind.is_empty()
        || value.input_kind.is_empty()
        || value.source_snapshot_digest.is_empty()
        || value.training_range_digest.is_empty()
        || value.fresh_validation_range_digest.is_empty()
        || value.validation_timestamp_digest.is_empty()
        || value.model_artifact_digest.is_empty()
        || value.parameter_digest.is_empty()
        || value.feature_normalizer_digest.is_empty()
        || value.representation_normalizer_digest.is_empty()
        || value.feature_policy_digest.is_empty()
        || value.label_policy_digest.is_empty()
        || value.training_policy_digest.is_empty()
        || value.initialization_digest.is_empty()
        || value.warm_start
        || value.v1_head_reused
        || value.v2_head_reused
        || !value.fresh_deterministic_initialization
        || value.encoder_frozen != learned
        || learned != value.encoder_digest.is_some()
        || value.deployment_status != ModelAgentDeploymentStatus::ShadowOnly
        || value.participant_digest != participant_digest_v3(value)
    {
        return Err("V3 participant rejected".to_string());
    }
    if learned != value.variant_digest.is_some() {
        return Err("V3 participant route binding rejected".to_string());
    }
    let identity_matches = match value.participant_role {
        MomentumRepresentationParticipantRoleV3::MambaOnly => matches!(
            (value.input_kind.as_str(), value.model_kind.as_str()),
            ("MambaLastOutput", "FrozenMambaLastOutputLogisticV3")
                | ("MambaMeanOutput", "FrozenMambaMeanOutputLogisticV3")
                | ("MambaLastMeanConcat", "FrozenMambaLastMeanConcatLogisticV3")
        ),
        MomentumRepresentationParticipantRoleV3::MambaResidualHybrid => {
            value.input_kind == "MambaRawFeatureResidual"
                && value.model_kind == "FrozenMambaRawResidualLogisticV3"
        }
        MomentumRepresentationParticipantRoleV3::LinearComparator => {
            value.input_kind == "RawFeatureLinearComparator"
                && value.model_kind == "LinearMomentumBaselineV3"
        }
        MomentumRepresentationParticipantRoleV3::ConstantBenchmark => {
            value.input_kind == "TrainingPrevalenceConstant"
                && value.model_kind == "ConstantProbabilityBaselineV3"
        }
    };
    if !identity_matches {
        return Err("V3 participant model identity rejected".to_string());
    }
    Ok(())
}

fn validate_receipt_v3(value: &MomentumRepresentationQualificationReceiptV3) -> Result<(), String> {
    if value.receipt_version != RECEIPT_VERSION_V3
        || value.participant_id.is_empty()
        || value.participant_digest.is_empty()
        || value.input_kind.is_empty()
        || value.fresh_validation_range_digest.is_empty()
        || value.qualification_policy_digest.is_empty()
        || value.private_metric_digest.is_empty()
        || value.validation_parameter_updates != 0
        || value.historical_test_reads != 0
        || value.future_evaluation_reads != 0
        || value.receipt_digest != receipt_digest_v3(value)
    {
        return Err("V3 qualification receipt rejected".to_string());
    }
    let residual_route = value.input_kind
        == format!(
            "{:?}",
            MomentumRepresentationInputKindV3::MambaRawFeatureResidual
        );
    if residual_route != value.contribution_audit_digest.is_some() {
        return Err("V3 contribution receipt binding rejected".to_string());
    }
    Ok(())
}

fn validate_contribution_v3(value: &MambaContributionAuditV3) -> Result<(), String> {
    if value.participant_digest.is_empty()
        || value.mamba_parameter_block_digest.is_empty()
        || value.raw_parameter_block_digest.is_empty()
        || value.full_prediction_digest.is_empty()
        || value.mamba_ablated_prediction_digest.is_empty()
        || value.raw_ablated_prediction_digest.is_empty()
        || value.mamba_ablation_effect_status.is_empty()
        || value.raw_ablation_effect_status.is_empty()
        || value.contribution_policy_digest != contribution_policy_digest_v3()
        || value.audit_digest != contribution_digest_v3(value)
    {
        return Err("V3 contribution audit rejected".to_string());
    }
    let status_matches = match value.contribution_status {
        MambaContributionStatusV3::NotApplicable => {
            !value.raw_block_nonzero
                && value.mamba_ablation_effect_status == "NotApplicable"
                && value.raw_ablation_effect_status == "NotApplicable"
        }
        MambaContributionStatusV3::MaterialContribution => {
            value.mamba_block_nonzero
                && value.raw_block_nonzero
                && value.mamba_ablation_effect_status == "Material"
        }
        MambaContributionStatusV3::DetectableButBelowPolicy => {
            value.mamba_block_nonzero
                && value.raw_block_nonzero
                && value.mamba_ablation_effect_status == "DetectableBelowPolicy"
        }
        MambaContributionStatusV3::NoDetectableContribution => {
            value.mamba_block_nonzero
                && value.raw_block_nonzero
                && value.mamba_ablation_effect_status == "NotDetectable"
                && value.raw_ablation_effect_status != "Material"
        }
        MambaContributionStatusV3::RawFeatureDominated => {
            value.mamba_block_nonzero
                && value.raw_block_nonzero
                && value.mamba_ablation_effect_status != "Material"
                && value.raw_ablation_effect_status == "Material"
        }
        MambaContributionStatusV3::Invalid => {
            !value.mamba_block_nonzero || !value.raw_block_nonzero
        }
    };
    if !status_matches {
        return Err("V3 contribution classification rejected".to_string());
    }
    Ok(())
}

fn validate_family_v3(value: &MomentumRepresentationFamilyV3) -> Result<(), String> {
    if value.family_version != FAMILY_VERSION_V3
        || value.agent_id != AGENT_ID_V3
        || value.source_snapshot_digest.is_empty()
        || value.canonical_view_digest.is_empty()
        || value.representation_audit_digest.is_empty()
        || value.split_digest.is_empty()
        || value.registration_digest.is_empty()
        || value.participants.len() != 6
        || value.qualification_receipts.len() != 6
        || value.contribution_audits.len() != 4
        || value.winner_selected
        || value.historical_test_accessed
        || value.eligible_for_active_committee
        || value.eligible_for_promotion
        || value.eligible_for_reward
        || value.family_digest != family_digest_v3(value)
    {
        return Err("V3 representation family rejected".to_string());
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
        return Err("V3 family receipt coverage rejected".to_string());
    }
    let role_counts = value
        .participants
        .iter()
        .fold([0_usize; 4], |mut counts, participant| {
            let index = match participant.participant_role {
                MomentumRepresentationParticipantRoleV3::MambaOnly => 0,
                MomentumRepresentationParticipantRoleV3::MambaResidualHybrid => 1,
                MomentumRepresentationParticipantRoleV3::LinearComparator => 2,
                MomentumRepresentationParticipantRoleV3::ConstantBenchmark => 3,
            };
            counts[index] += 1;
            counts
        });
    if role_counts != [3, 1, 1, 1] {
        return Err("V3 family participant roles rejected".to_string());
    }
    let learned_digests = value
        .participants
        .iter()
        .filter(|participant| {
            matches!(
                participant.participant_role,
                MomentumRepresentationParticipantRoleV3::MambaOnly
                    | MomentumRepresentationParticipantRoleV3::MambaResidualHybrid
            )
        })
        .map(|participant| participant.participant_digest.as_str())
        .collect::<BTreeSet<_>>();
    let contribution_digests = value
        .contribution_audits
        .iter()
        .map(|audit| audit.participant_digest.as_str())
        .collect::<BTreeSet<_>>();
    if learned_digests != contribution_digests {
        return Err("V3 family contribution coverage rejected".to_string());
    }
    for participant in &value.participants {
        let receipt = value
            .qualification_receipts
            .iter()
            .find(|receipt| receipt.participant_digest == participant.participant_digest)
            .ok_or_else(|| "V3 family participant receipt missing".to_string())?;
        if receipt.participant_id != participant.participant_id
            || receipt.input_kind != participant.input_kind
            || receipt.fresh_validation_range_digest != participant.fresh_validation_range_digest
        {
            return Err("V3 family participant receipt binding rejected".to_string());
        }
        let status_allowed = match participant.participant_role {
            MomentumRepresentationParticipantRoleV3::MambaOnly => matches!(
                receipt.status,
                MomentumRepresentationQualificationStatusV3::QualifiedMambaOnly
                    | MomentumRepresentationQualificationStatusV3::RejectedInsufficientValidation
                    | MomentumRepresentationQualificationStatusV3::RejectedRepresentationInvariant
                    | MomentumRepresentationQualificationStatusV3::RejectedProbabilityCollapse
                    | MomentumRepresentationQualificationStatusV3::RejectedNumericalFailure
                    | MomentumRepresentationQualificationStatusV3::RejectedContributionInvariant
            ),
            MomentumRepresentationParticipantRoleV3::MambaResidualHybrid => matches!(
                receipt.status,
                MomentumRepresentationQualificationStatusV3::QualifiedMambaContributingHybrid
                    | MomentumRepresentationQualificationStatusV3::QualifiedRawFallbackNotMamba
                    | MomentumRepresentationQualificationStatusV3::RejectedInsufficientValidation
                    | MomentumRepresentationQualificationStatusV3::RejectedRepresentationInvariant
                    | MomentumRepresentationQualificationStatusV3::RejectedProbabilityCollapse
                    | MomentumRepresentationQualificationStatusV3::RejectedNumericalFailure
                    | MomentumRepresentationQualificationStatusV3::RejectedContributionInvariant
            ),
            MomentumRepresentationParticipantRoleV3::LinearComparator => matches!(
                receipt.status,
                MomentumRepresentationQualificationStatusV3::ComparatorQualified
                    | MomentumRepresentationQualificationStatusV3::RejectedInsufficientValidation
                    | MomentumRepresentationQualificationStatusV3::RejectedNumericalFailure
            ),
            MomentumRepresentationParticipantRoleV3::ConstantBenchmark => matches!(
                receipt.status,
                MomentumRepresentationQualificationStatusV3::BenchmarkQualified
                    | MomentumRepresentationQualificationStatusV3::RejectedInsufficientValidation
                    | MomentumRepresentationQualificationStatusV3::RejectedNumericalFailure
            ),
        };
        if !status_allowed {
            return Err("V3 participant qualification role rejected".to_string());
        }
        let contribution = value
            .contribution_audits
            .iter()
            .find(|audit| audit.participant_digest == participant.participant_digest);
        match participant.participant_role {
            MomentumRepresentationParticipantRoleV3::MambaResidualHybrid => {
                if receipt.contribution_audit_digest.as_deref()
                    != contribution.map(|audit| audit.audit_digest.as_str())
                {
                    return Err("V3 residual contribution binding rejected".to_string());
                }
                let contribution_status = contribution.map(|audit| audit.contribution_status);
                if (receipt.status
                    == MomentumRepresentationQualificationStatusV3::QualifiedMambaContributingHybrid
                    && contribution_status != Some(MambaContributionStatusV3::MaterialContribution))
                    || (receipt.status
                        == MomentumRepresentationQualificationStatusV3::QualifiedRawFallbackNotMamba
                        && contribution_status
                            != Some(MambaContributionStatusV3::RawFeatureDominated))
                {
                    return Err("V3 residual qualification contribution rejected".to_string());
                }
            }
            MomentumRepresentationParticipantRoleV3::MambaOnly => {
                if receipt.contribution_audit_digest.is_some()
                    || contribution.is_none_or(|audit| {
                        audit.contribution_status != MambaContributionStatusV3::NotApplicable
                    })
                {
                    return Err("V3 Mamba-only contribution binding rejected".to_string());
                }
            }
            _ if contribution.is_some() || receipt.contribution_audit_digest.is_some() => {
                return Err("V3 comparator contribution binding rejected".to_string());
            }
            _ => {}
        }
    }
    let validation_timestamp_digests = value
        .participants
        .iter()
        .map(|participant| participant.validation_timestamp_digest.as_str())
        .collect::<BTreeSet<_>>();
    let validation_range_digests = value
        .participants
        .iter()
        .map(|participant| participant.fresh_validation_range_digest.as_str())
        .collect::<BTreeSet<_>>();
    if validation_timestamp_digests.len() != 1 || validation_range_digests.len() != 1 {
        return Err("V3 family validation identity diverged".to_string());
    }
    let actual_mamba_only = value
        .qualification_receipts
        .iter()
        .filter(|receipt| {
            receipt.status == MomentumRepresentationQualificationStatusV3::QualifiedMambaOnly
        })
        .count();
    let actual_mamba_hybrid = value
        .qualification_receipts
        .iter()
        .filter(|receipt| {
            receipt.status
                == MomentumRepresentationQualificationStatusV3::QualifiedMambaContributingHybrid
        })
        .count();
    let actual_raw = value
        .qualification_receipts
        .iter()
        .filter(|receipt| {
            receipt.status
                == MomentumRepresentationQualificationStatusV3::QualifiedRawFallbackNotMamba
        })
        .count();
    let actual_comparators = value
        .qualification_receipts
        .iter()
        .filter(|receipt| {
            matches!(
                receipt.status,
                MomentumRepresentationQualificationStatusV3::ComparatorQualified
                    | MomentumRepresentationQualificationStatusV3::BenchmarkQualified
            )
        })
        .count();
    if value.qualified_mamba_only_count != actual_mamba_only
        || value.qualified_mamba_hybrid_count != actual_mamba_hybrid
        || value.qualified_raw_fallback_count != actual_raw
        || value.qualified_comparator_count != actual_comparators
    {
        return Err("V3 family qualification counts rejected".to_string());
    }
    for participant in &value.participants {
        validate_participant_v3(participant)?;
    }
    for receipt in &value.qualification_receipts {
        validate_receipt_v3(receipt)?;
    }
    for audit in &value.contribution_audits {
        validate_contribution_v3(audit)?;
    }
    Ok(())
}

fn validate_decision_v3(
    value: &MomentumRepresentationRouteDecisionArtifactV3,
    family: &MomentumRepresentationFamilyV3,
) -> Result<(), String> {
    let no_genuine = family.qualified_mamba_only_count + family.qualified_mamba_hybrid_count == 0;
    if value.decision_version != DECISION_VERSION_V3
        || value.v3_family_digest != family.family_digest
        || !value.further_head_only_repair_forbidden
        || value.further_frozen_representation_sweep_forbidden != no_genuine
        || value.decision_digest != decision_digest_v3(value)
    {
        return Err("V3 route decision rejected".to_string());
    }
    let expected = if family.qualified_mamba_only_count > 0 {
        MomentumRepresentationRouteDecisionV3::FrozenMambaOnlyViable
    } else if family.qualified_mamba_hybrid_count > 0 {
        MomentumRepresentationRouteDecisionV3::MambaResidualHybridViable
    } else if family.qualified_raw_fallback_count > 0 {
        MomentumRepresentationRouteDecisionV3::RawFeatureFallbackOnly
    } else {
        MomentumRepresentationRouteDecisionV3::AllRepresentationRoutesCollapsed
    };
    if value.decision != expected {
        return Err("V3 route decision rules rejected".to_string());
    }
    let status_for = |digest: &str| {
        family
            .qualification_receipts
            .iter()
            .find(|receipt| receipt.participant_digest == digest)
            .map(|receipt| receipt.status)
    };
    let route_digests_for = |status: MomentumRepresentationQualificationStatusV3| {
        sorted_unique(
            family
                .participants
                .iter()
                .filter(|participant| {
                    matches!(
                        participant.participant_role,
                        MomentumRepresentationParticipantRoleV3::MambaOnly
                            | MomentumRepresentationParticipantRoleV3::MambaResidualHybrid
                    ) && status_for(&participant.participant_digest) == Some(status)
                })
                .map(|participant| participant.participant_digest.clone())
                .collect(),
        )
    };
    let expected_mamba_only =
        route_digests_for(MomentumRepresentationQualificationStatusV3::QualifiedMambaOnly);
    let expected_mamba_hybrid = route_digests_for(
        MomentumRepresentationQualificationStatusV3::QualifiedMambaContributingHybrid,
    );
    let expected_raw = route_digests_for(
        MomentumRepresentationQualificationStatusV3::QualifiedRawFallbackNotMamba,
    );
    let included = expected_mamba_only
        .iter()
        .chain(&expected_mamba_hybrid)
        .chain(&expected_raw)
        .collect::<BTreeSet<_>>();
    let expected_rejected = sorted_unique(
        family
            .participants
            .iter()
            .filter(|participant| {
                matches!(
                    participant.participant_role,
                    MomentumRepresentationParticipantRoleV3::MambaOnly
                        | MomentumRepresentationParticipantRoleV3::MambaResidualHybrid
                ) && !included.contains(&participant.participant_digest)
            })
            .map(|participant| participant.participant_digest.clone())
            .collect(),
    );
    if value.qualified_mamba_only_digests != expected_mamba_only
        || value.qualified_mamba_hybrid_digests != expected_mamba_hybrid
        || value.raw_fallback_digests != expected_raw
        || value.rejected_route_digests != expected_rejected
    {
        return Err("V3 route decision participant sets rejected".to_string());
    }
    Ok(())
}

fn validate_roster_v3(
    value: &MomentumRepresentationFutureRosterV3,
    family: &MomentumRepresentationFamilyV3,
    decision: &MomentumRepresentationRouteDecisionArtifactV3,
) -> Result<(), String> {
    if value.roster_version != ROSTER_VERSION_V3
        || value.family_digest != family.family_digest
        || value.decision_digest != decision.decision_digest
        || value.qualified_genuine_mamba_digests.is_empty()
        || value.qualified_comparator_digests.is_empty()
        || value.roster_digest != roster_digest_v3(value)
    {
        return Err("V3 future roster rejected".to_string());
    }
    let expected_genuine = sorted_unique(
        family
            .qualification_receipts
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt.status,
                    MomentumRepresentationQualificationStatusV3::QualifiedMambaOnly
                        | MomentumRepresentationQualificationStatusV3::QualifiedMambaContributingHybrid
                )
            })
            .map(|receipt| receipt.participant_digest.clone())
            .collect(),
    );
    let expected_comparators = sorted_unique(
        family
            .qualification_receipts
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt.status,
                    MomentumRepresentationQualificationStatusV3::ComparatorQualified
                        | MomentumRepresentationQualificationStatusV3::BenchmarkQualified
                )
            })
            .map(|receipt| receipt.participant_digest.clone())
            .collect(),
    );
    let included = expected_genuine
        .iter()
        .chain(&expected_comparators)
        .collect::<BTreeSet<_>>();
    let expected_excluded = sorted_unique(
        family
            .participants
            .iter()
            .filter(|participant| !included.contains(&participant.participant_digest))
            .map(|participant| participant.participant_digest.clone())
            .collect(),
    );
    let raw = family
        .qualification_receipts
        .iter()
        .filter(|receipt| {
            receipt.status
                == MomentumRepresentationQualificationStatusV3::QualifiedRawFallbackNotMamba
        })
        .map(|receipt| receipt.participant_digest.as_str())
        .collect::<BTreeSet<_>>();
    if value.qualified_genuine_mamba_digests != expected_genuine
        || value.qualified_comparator_digests != expected_comparators
        || value.excluded_participant_digests != expected_excluded
        || value
            .qualified_genuine_mamba_digests
            .iter()
            .any(|digest| raw.contains(digest.as_str()))
    {
        return Err("raw fallback entered Mamba roster".to_string());
    }
    Ok(())
}

fn validate_evaluation_v3(
    value: &MomentumRepresentationEvaluationRegistrationV3,
    family: &MomentumRepresentationFamilyV3,
    decision: &MomentumRepresentationRouteDecisionArtifactV3,
    roster: &MomentumRepresentationFutureRosterV3,
) -> Result<(), String> {
    if value.registration_version != EVALUATION_VERSION_V3
        || value.agent_id != AGENT_ID_V3
        || value.family_digest != family.family_digest
        || value.roster_digest != roster.roster_digest
        || value.decision_digest != decision.decision_digest
        || value.qualification_receipt_digests.is_empty()
        || value.contribution_audit_digests.is_empty()
        || value.source_snapshot_digest != family.source_snapshot_digest
        || value.source_boundary_timestamp_ms == 0
        || value.protected_registration_digests.is_empty()
        || value.protected_timestamp_ms.len() != 4
        || value
            .protected_timestamp_ms
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        || value.prior_validation_and_reserved_range_digests.len() != 6
        || value.minimum_accepted_timestamp_ms <= value.source_boundary_timestamp_ms
        || value
            .protected_timestamp_ms
            .last()
            .is_none_or(|timestamp| value.minimum_accepted_timestamp_ms <= *timestamp)
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
        || value.registration_digest != evaluation_digest_v3(value)
    {
        return Err("V3 evaluation registration rejected".to_string());
    }
    let included = roster
        .qualified_genuine_mamba_digests
        .iter()
        .chain(&roster.qualified_comparator_digests)
        .collect::<BTreeSet<_>>();
    let expected_receipts = sorted_unique(
        family
            .qualification_receipts
            .iter()
            .filter(|receipt| included.contains(&receipt.participant_digest))
            .map(|receipt| receipt.receipt_digest.clone())
            .collect(),
    );
    let expected_contributions = sorted_unique(
        family
            .contribution_audits
            .iter()
            .filter(|audit| included.contains(&audit.participant_digest))
            .map(|audit| audit.audit_digest.clone())
            .collect(),
    );
    if value.qualification_receipt_digests != expected_receipts
        || value.contribution_audit_digests != expected_contributions
    {
        return Err("V3 evaluation participant evidence rejected".to_string());
    }
    Ok(())
}

fn validate_journal_v3(value: &MomentumRepresentationJournalV3) -> Result<(), String> {
    if value.journal_version != JOURNAL_VERSION_V3
        || value.agent_id != AGENT_ID_V3
        || value.prior_validation_used_for_v3_qualification
        || value.final_reserve_accessed
        || value.warm_start
        || value.v1_head_reused
        || value.v2_head_reused
        || !value.fresh_deterministic_initialization
        || value.status != MomentumRepresentationExecutionStatusV3::Executed
        || value.journal_digest != journal_digest_v3(value)
    {
        return Err("V3 journal rejected".to_string());
    }
    Ok(())
}

fn protobuf_paths_v3(directory: &Path) -> Result<Vec<PathBuf>, String> {
    if !directory.is_dir() {
        return Err("V3 artifact directory unavailable".to_string());
    }
    let mut paths = fs::read_dir(directory)
        .map_err(|_| "V3 artifact directory read failed".to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|value| value == "pb"))
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn read_single_v3<T>(
    directory: &Path,
    decode: impl Fn(&[u8]) -> Result<T, String>,
) -> Result<T, String> {
    let paths = protobuf_paths_v3(directory)?;
    if paths.len() != 1 {
        return Err("V3 single artifact identity rejected".to_string());
    }
    let bytes = fs::read(&paths[0]).map_err(|_| "V3 artifact read failed".to_string())?;
    decode(&bytes)
}

fn load_frozen_history_v3(
    root: &Path,
    snapshots: &[DataSnapshot],
) -> Result<FrozenHistoryV3, String> {
    let v1 = load_v1_frozen_state_v2(root, snapshots)?;
    let v2_root = root.join("v2").join(AGENT_ID_V3);
    let v2_audit = read_single_v3(
        &v2_root.join("collapse_audits"),
        decode_momentum_mamba_collapse_audit_protobuf_v2,
    )?;
    let v2_split = read_single_v3(
        &v2_root.join("repair_splits"),
        decode_momentum_mamba_repair_split_protobuf_v2,
    )?;
    let v2_registration = read_single_v3(
        &v2_root.join("repair_registrations"),
        decode_momentum_mamba_repair_registration_protobuf_v2,
    )?;
    let v2_family = read_single_v3(
        &v2_root.join("families"),
        decode_momentum_candidate_family_protobuf_v2,
    )?;
    let v2_journal = read_single_v3(
        &v2_root.join("journals"),
        decode_momentum_mamba_repair_journal_protobuf_v2,
    )?;
    let v2_qualifications_match = v2_family.participants.iter().all(|participant| {
        let Some(receipt) = v2_family
            .qualification_receipts
            .iter()
            .find(|receipt| receipt.participant_digest == participant.participant_digest)
        else {
            return false;
        };
        match participant.participant_role {
            ParticipantQualificationRoleV2::LearnedCandidate => {
                receipt.qualification_status
                    == ValidationQualificationStatusV2::RejectedProbabilityCollapse
            }
            ParticipantQualificationRoleV2::LinearComparator => {
                receipt.qualification_status == ValidationQualificationStatusV2::Qualified
            }
            ParticipantQualificationRoleV2::ConstantBenchmark => {
                receipt.qualification_status == ValidationQualificationStatusV2::BenchmarkQualified
            }
        }
    });
    if v2_registration.collapse_audit_digest != v2_audit.audit_digest
        || v2_audit.source_family_digest != v1.family.family_digest
        || v2_audit.failed_participant_digest != v1.failed_participant.participant_digest
        || v2_audit.failed_qualification_receipt_digest != v1.failed_receipt.receipt_digest
        || v2_audit.root_causes != vec![MomentumMambaCollapseRootCauseV2::ProbabilitySingleSided]
        || v2_audit.repair_capability_status
            != MomentumMambaRepairCapabilityStatusV2::RepairableWithBoundedHeadRegularization
        || v2_registration.repair_split_digest != v2_split.split_digest
        || v2_family.collapse_audit_digest != v2_audit.audit_digest
        || v2_family.repair_split_digest != v2_split.split_digest
        || v2_family.repair_registration_digest != v2_registration.registration_digest
        || v2_journal.collapse_audit_digest != v2_audit.audit_digest
        || v2_journal.repair_split_digest != v2_split.split_digest
        || v2_journal.repair_registration_digest != v2_registration.registration_digest
        || v2_journal.family_digest.as_deref() != Some(v2_family.family_digest.as_str())
        || v2_family.qualified_learned_participant_count != 0
        || v2_family.qualified_comparator_count != 2
        || v2_family.learned_participant_count != 3
        || v2_registration.allowed_variant_configs.len() != 3
        || !v2_qualifications_match
        || v2_family.winner_selected
        || v2_family.historical_test_accessed
        || v2_family.eligible_for_active_committee
        || v2_family.eligible_for_promotion
        || v2_family.eligible_for_reward
        || v2_split.remaining_reserved_range.is_none()
        || v2_journal.roster_digest.is_some()
        || v2_journal.evaluation_registration_digest.is_some()
        || v2_journal.status != MomentumMambaRepairExecutionStatusV2::Executed
        || v2_root.join("rosters").exists()
        || v2_root.join("evaluation_registrations").exists()
    {
        return Err("immutable V2 failure history rejected".to_string());
    }
    Ok(FrozenHistoryV3 {
        v1,
        v2_audit,
        v2_split,
        v2_family,
    })
}

fn representation_diagnostic_v3(rows: &[EncodedTrainingExampleV0]) -> RepresentationDiagnosticV3 {
    let dimension = rows.first().map_or(0, |row| row.representation.len());
    let finite = dimension > 0
        && rows.iter().all(|row| {
            row.representation.len() == dimension
                && row.representation.iter().all(|value| value.is_finite())
        });
    let variances = if finite {
        (0..dimension)
            .map(|index| {
                let mean = rows
                    .iter()
                    .map(|row| row.representation[index])
                    .sum::<f32>()
                    / rows.len() as f32;
                rows.iter()
                    .map(|row| (row.representation[index] - mean).powi(2))
                    .sum::<f32>()
                    / rows.len() as f32
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let aggregate = if variances.is_empty() {
        f32::NAN
    } else {
        variances.iter().sum::<f32>() / variances.len() as f32
    };
    let variance = if !aggregate.is_finite() {
        VarianceClassV3::NumericalFailure
    } else if aggregate <= 1e-12 {
        VarianceClassV3::NearConstant
    } else if aggregate <= 1e-5 {
        VarianceClassV3::Low
    } else {
        VarianceClassV3::Adequate
    };
    let sum = variances.iter().sum::<f32>();
    let squared_sum = variances.iter().map(|value| value * value).sum::<f32>();
    let rank = if sum.is_finite() && squared_sum.is_finite() && squared_sum > 0.0 {
        Some(sum * sum / squared_sum)
    } else {
        None
    };
    let effective_rank = match rank {
        None => RankClassV3::Unavailable,
        Some(value) if value <= 1.5 => RankClassV3::RankOneOrLess,
        Some(value) if value < (dimension as f32 * 0.5).max(2.0) => RankClassV3::Low,
        Some(_) => RankClassV3::Adequate,
    };
    let redundant_dimension_count = variances.iter().filter(|value| **value <= 1e-12).count();
    let digest = stable_hash_string(&format!(
        "representation-diagnostic-v3:{finite}:{variance:?}:{effective_rank:?}:{redundant_dimension_count}:{:?}",
        variances
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    ));
    RepresentationDiagnosticV3 {
        finite,
        variance,
        effective_rank,
        redundant_dimension_count,
        digest,
    }
}

fn probabilities_collapsed_v3(probabilities: &[f32]) -> Result<bool, String> {
    if probabilities.is_empty() || probabilities.iter().any(|value| !value.is_finite()) {
        return Err("V3 probability diagnostics unavailable".to_string());
    }
    let mean = probabilities.iter().sum::<f32>() / probabilities.len() as f32;
    let variance = probabilities
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f32>()
        / probabilities.len() as f32;
    let all_low = probabilities.iter().all(|value| *value < 0.5);
    let all_high = probabilities.iter().all(|value| *value >= 0.5);
    Ok(variance <= 1e-6 || all_low || all_high)
}

fn train_encoded_head_v3(
    mut head: LogisticPredictionHeadV0,
    training: &[EncodedTrainingExampleV0],
    config: &HeadTrainingConfigV0,
) -> Result<LogisticPredictionHeadV0, String> {
    config
        .validate()
        .map_err(|_| "V3 head training policy rejected".to_string())?;
    if training.is_empty() {
        return Err("V3 head training evidence unavailable".to_string());
    }
    for _ in 0..config.epochs {
        for batch in training.chunks(config.batch_size) {
            let (_, gradients) = brier_loss_and_gradients_v0(&head, batch)
                .map_err(|_| "V3 head gradient failed".to_string())?;
            apply_sgd_v0(&mut head, &gradients, &config.optimizer)
                .map_err(|_| "V3 head update failed".to_string())?;
        }
    }
    Ok(head)
}

fn raw_end_encoded_v3(
    examples: &[SequenceExampleV0],
) -> Result<Vec<EncodedTrainingExampleV0>, String> {
    examples
        .iter()
        .map(|example| {
            let representation = example
                .input
                .last()
                .cloned()
                .ok_or_else(|| "V3 raw feature representation unavailable".to_string())?;
            Ok(EncodedTrainingExampleV0 {
                representation,
                label: example.label,
                snapshot_ids: example.snapshot_ids.clone(),
            })
        })
        .collect()
}

fn concatenate_encoded_v3(
    left: &[EncodedTrainingExampleV0],
    right: &[EncodedTrainingExampleV0],
) -> Result<Vec<EncodedTrainingExampleV0>, String> {
    if left.len() != right.len() {
        return Err("V3 representation concatenation length rejected".to_string());
    }
    left.iter()
        .zip(right)
        .map(|(left, right)| {
            if left.label != right.label || left.snapshot_ids != right.snapshot_ids {
                return Err("V3 representation concatenation identity rejected".to_string());
            }
            let mut representation = left.representation.clone();
            representation.extend_from_slice(&right.representation);
            Ok(EncodedTrainingExampleV0 {
                representation,
                label: left.label,
                snapshot_ids: left.snapshot_ids.clone(),
            })
        })
        .collect()
}

fn probe_from_encoded_v3(
    kind: MomentumRepresentationProbeKindV3,
    source_snapshot_digest: &str,
    consumed_range_digest: &str,
    feature_policy_digest: &str,
    encoder_digest: Option<String>,
    training: &[EncodedTrainingExampleV0],
    validation: &[EncodedTrainingExampleV0],
    seed: u64,
) -> Result<MomentumRepresentationProbeV3, String> {
    if training.is_empty() || validation.is_empty() {
        return Err("V3 probe evidence unavailable".to_string());
    }
    let diagnostic = representation_diagnostic_v3(validation);
    let mut training_config = HeadTrainingConfigV0::default();
    training_config.seed = seed;
    training_config.early_stopping_patience = None;
    let initial = LogisticPredictionHeadV0::seeded(training[0].representation.len(), seed)
        .map_err(|_| "V3 probe initialization failed".to_string())?;
    let head = train_encoded_head_v3(initial, training, &training_config)?;
    let metrics =
        evaluate_head_v0(&head, validation).map_err(|_| "V3 probe metric failed".to_string())?;
    let probabilities = validation
        .iter()
        .map(|row| head.probability(&row.representation))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "V3 probe probability failed".to_string())?;
    let collapsed = probabilities_collapsed_v3(&probabilities)?;
    let status = if !diagnostic.finite {
        MomentumRepresentationProbeStatusV3::NumericalFailure
    } else if validation.len()
        < MomentumLearningCampaignConfigV0::default()
            .validation_signal_gate
            .minimum_samples
    {
        MomentumRepresentationProbeStatusV3::InsufficientEvidence
    } else if matches!(
        diagnostic.variance,
        VarianceClassV3::NearConstant | VarianceClassV3::Low
    ) {
        MomentumRepresentationProbeStatusV3::LowVariance
    } else if matches!(
        diagnostic.effective_rank,
        RankClassV3::RankOneOrLess | RankClassV3::Low
    ) {
        MomentumRepresentationProbeStatusV3::LowEffectiveRank
    } else if collapsed {
        MomentumRepresentationProbeStatusV3::SingleSidedPrediction
    } else {
        MomentumRepresentationProbeStatusV3::NonCollapsedPrediction
    };
    let mut probe = MomentumRepresentationProbeV3 {
        probe_kind: kind,
        source_snapshot_digest: source_snapshot_digest.to_string(),
        consumed_range_digest: consumed_range_digest.to_string(),
        feature_policy_digest: feature_policy_digest.to_string(),
        encoder_digest,
        representation_diagnostic_digest: diagnostic.digest,
        private_probe_metric_digest: stable_hash_string(&format!(
            "private-representation-probe-metric-v3:{kind:?}:{metrics:?}"
        )),
        status,
        probe_digest: String::new(),
    };
    probe.probe_digest = probe_digest_v3(&probe);
    validate_probe_v3(&probe)?;
    Ok(probe)
}

fn derive_representation_audit_v3(
    history: &FrozenHistoryV3,
) -> Result<MomentumRepresentationPathAuditV3, String> {
    let config = MomentumLearningCampaignConfigV0::default();
    let consumed_end = history.v2_split.fresh_repair_validation_range.end;
    let candles = candles_from_snapshot_prefix(&history.v1.snapshot, consumed_end)?;
    let features = build_momentum_features_v0(&candles, &config.feature_config)
        .map_err(|_| "V3 probe feature derivation failed".to_string())?;
    let training_features = features
        .iter()
        .filter(|row| row.source_index < history.v2_split.repair_training_range.end)
        .cloned()
        .collect::<Vec<_>>();
    let feature_normalizer = FeatureNormalizerV0::fit(&training_features)
        .map_err(|_| "V3 probe feature normalizer failed".to_string())?;
    if feature_normalizer.fitted_on_end >= history.v2_split.repair_training_range.end {
        return Err("V3 probe normalizer crossed consumed training".to_string());
    }
    let normalized = feature_normalizer
        .transform(&features)
        .map_err(|_| "V3 probe feature transform failed".to_string())?;
    let examples = build_momentum_sequence_examples_v0(
        &candles,
        &normalized,
        &config.sequence_config,
        std::slice::from_ref(&history.v1.snapshot.snapshot_id),
    )
    .map_err(|_| "V3 probe sequence derivation failed".to_string())?;
    let training = examples_in_range(&examples, &history.v2_split.repair_training_range);
    let validation = examples_in_range(&examples, &history.v2_split.fresh_repair_validation_range);
    if training.is_empty()
        || validation.len() < config.validation_signal_gate.minimum_samples
        || validation.iter().any(|row| {
            row.label_index < history.v2_split.fresh_repair_validation_range.start
                || row.label_index >= history.v2_split.fresh_repair_validation_range.end
        })
    {
        return Err("V3 consumed probe partition rejected".to_string());
    }
    let raw_training = raw_end_encoded_v3(&training)?;
    let raw_validation = raw_end_encoded_v3(&validation)?;
    let mut last_encoder = frozen_mamba3_encoder_from_seed_v0(
        &config.feature_config,
        config.campaign_seed,
        config.backend_preference,
        config.fallback_policy,
    )
    .map_err(|_| "V3 last-output probe encoder unavailable".to_string())?;
    last_encoder.pooling = SequencePooling::LastOutput;
    let mut mean_encoder = last_encoder.clone();
    mean_encoder.pooling = SequencePooling::MeanOutput;
    let encoder_digest = last_encoder.parameter_digest();
    if mean_encoder.parameter_digest() != encoder_digest {
        return Err("V3 probe encoder identity diverged".to_string());
    }
    let last_training = last_encoder
        .encode_batch(&training)
        .map_err(|_| "V3 last-output probe training representation failed".to_string())?;
    let last_validation = last_encoder
        .encode_batch(&validation)
        .map_err(|_| "V3 last-output probe validation representation failed".to_string())?;
    let mean_training = mean_encoder
        .encode_batch(&training)
        .map_err(|_| "V3 mean-output probe training representation failed".to_string())?;
    let mean_validation = mean_encoder
        .encode_batch(&validation)
        .map_err(|_| "V3 mean-output probe validation representation failed".to_string())?;
    let concat_training = concatenate_encoded_v3(&last_training, &mean_training)?;
    let concat_validation = concatenate_encoded_v3(&last_validation, &mean_validation)?;
    let consumed_range_digest = stable_hash_string(&format!(
        "consumed-probe-ranges-v3:{:?}:{:?}",
        history.v2_split.repair_training_range, history.v2_split.fresh_repair_validation_range
    ));
    let source = history.v1.snapshot.content_digest.as_str();
    let feature_policy = history.v1.session.feature_policy_digest.as_str();
    let base_seed = config.campaign_seed ^ 0x79A0_0000;
    let mut probes = vec![
        probe_from_encoded_v3(
            MomentumRepresentationProbeKindV3::RawFeatureLinearProbe,
            source,
            &consumed_range_digest,
            feature_policy,
            None,
            &raw_training,
            &raw_validation,
            base_seed ^ 1,
        )?,
        probe_from_encoded_v3(
            MomentumRepresentationProbeKindV3::MambaLastOutputProbe,
            source,
            &consumed_range_digest,
            feature_policy,
            Some(encoder_digest.clone()),
            &last_training,
            &last_validation,
            base_seed ^ 2,
        )?,
        probe_from_encoded_v3(
            MomentumRepresentationProbeKindV3::MambaMeanOutputProbe,
            source,
            &consumed_range_digest,
            feature_policy,
            Some(encoder_digest.clone()),
            &mean_training,
            &mean_validation,
            base_seed ^ 3,
        )?,
        probe_from_encoded_v3(
            MomentumRepresentationProbeKindV3::MambaLastMeanConcatProbe,
            source,
            &consumed_range_digest,
            feature_policy,
            Some(encoder_digest),
            &concat_training,
            &concat_validation,
            base_seed ^ 4,
        )?,
    ];
    probes.sort_by_key(|probe| probe.probe_kind);
    let diagnostic_digests = probes
        .iter()
        .map(|probe| probe.representation_diagnostic_digest.as_str())
        .collect::<BTreeSet<_>>();
    if diagnostic_digests.len() != probes.len() {
        return Err("V3 probe representations were not distinct".to_string());
    }
    let mut audit = MomentumRepresentationPathAuditV3 {
        audit_version: AUDIT_VERSION_V3.to_string(),
        v1_family_digest: history.v1.family.family_digest.clone(),
        v2_family_digest: history.v2_family.family_digest.clone(),
        v2_collapse_audit_digest: history.v2_audit.audit_digest.clone(),
        probes,
        head_only_repair_exhausted: true,
        fresh_v3_validation_accessed: false,
        audit_digest: String::new(),
    };
    audit.audit_digest = audit_digest_v3(&audit);
    validate_audit_v3(&audit)?;
    Ok(audit)
}

fn derive_representation_split_v3(
    history: &FrozenHistoryV3,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<MomentumRepresentationSplitV3, String> {
    let config = MomentumLearningCampaignConfigV0::default();
    let available = history
        .v2_split
        .remaining_reserved_range
        .as_ref()
        .ok_or_else(|| "V3 remaining reserve unavailable".to_string())?;
    let feature_history = config
        .feature_config
        .minimum_history()
        .map_err(|_| "V3 feature history unavailable".to_string())?;
    let purge_count = feature_history
        .checked_sub(1)
        .and_then(|value| value.checked_add(config.sequence_config.sequence_length - 1))
        .and_then(|value| value.checked_add(config.sequence_config.prediction_horizon))
        .ok_or_else(|| "V3 purge calculation overflow".to_string())?;
    let minimum_validation_samples = config.validation_rows;
    let minimum_final_reserved_samples = minimum_validation_samples
        .checked_mul(2)
        .ok_or_else(|| "V3 final reserve calculation overflow".to_string())?;
    let fresh_validation_end = available
        .end
        .checked_sub(minimum_final_reserved_samples)
        .ok_or_else(|| "V3 final reserve is insufficient".to_string())?;
    let fresh_validation_start = fresh_validation_end
        .checked_sub(minimum_validation_samples)
        .ok_or_else(|| "V3 validation reserve is insufficient".to_string())?;
    let training_end = fresh_validation_start
        .checked_sub(purge_count)
        .ok_or_else(|| "V3 purge reserve is insufficient".to_string())?;
    if training_end < available.start || fresh_validation_end > available.end {
        return Err("V3 split exceeds remaining reserve".to_string());
    }
    let training_range = IndexRangeV0 {
        start: 0,
        end: training_end,
    };
    let purge_range = IndexRangeV0 {
        start: training_end,
        end: fresh_validation_start,
    };
    let fresh_validation_range = IndexRangeV0 {
        start: fresh_validation_start,
        end: fresh_validation_end,
    };
    let final_reserved_range = IndexRangeV0 {
        start: fresh_validation_end,
        end: available.end,
    };
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
    let prior_validation_overlap_count =
        ranges_overlap_v3(&history.v1.prior_validation_range, &fresh_validation_range)
            + ranges_overlap_v3(
                &history.v2_split.fresh_repair_validation_range,
                &fresh_validation_range,
            );
    let mut split = MomentumRepresentationSplitV3 {
        split_version: SPLIT_VERSION_V3.to_string(),
        source_snapshot_digest: history.v1.snapshot.content_digest.clone(),
        v1_usage_ledger_digest: history.v1.usage_ledger.ledger_digest.clone(),
        v2_split_digest: history.v2_split.split_digest.clone(),
        training_range,
        purge_range,
        fresh_validation_range,
        final_reserved_range,
        minimum_validation_samples,
        minimum_final_reserved_samples,
        prior_validation_overlap_count,
        prospective_overlap_count,
        future_evaluation_overlap_count: 0,
        split_digest: String::new(),
    };
    split.split_digest = split_digest_v3(&split);
    validate_split_v3(&split)?;
    Ok(split)
}

fn representation_variant_v3(
    variant_id: &str,
    input_kind: MomentumRepresentationInputKindV3,
    pooling_policy: &str,
    initialization_seed: u64,
    history: &FrozenHistoryV3,
    config: &MomentumLearningCampaignConfigV0,
    contribution_policy_digest: &str,
) -> Result<MomentumRepresentationVariantConfigV3, String> {
    let raw_feature_residual_enabled =
        input_kind == MomentumRepresentationInputKindV3::MambaRawFeatureResidual;
    let mut value = MomentumRepresentationVariantConfigV3 {
        variant_id: variant_id.to_string(),
        input_kind,
        pooling_policy: pooling_policy.to_string(),
        raw_feature_residual_enabled,
        head_kind: "LogisticPredictionHeadV0".to_string(),
        learning_rate_bits: config.training_config.optimizer.learning_rate.to_bits(),
        l2_regularization_bits: config.training_config.optimizer.weight_decay.to_bits(),
        maximum_epochs: config.training_config.epochs,
        initialization_seed,
        encoder_frozen: true,
        feature_policy_digest: history.v1.session.feature_policy_digest.clone(),
        label_policy_digest: history.v1.session.label_policy_digest.clone(),
        training_policy_digest: stable_hash_string(&format!(
            "fixed-representation-route-head-v3:{}:{}:{}:{}:{}:{}",
            config.training_config.optimizer.learning_rate.to_bits(),
            config.training_config.optimizer.weight_decay.to_bits(),
            config.training_config.epochs,
            initialization_seed,
            input_kind as u8,
            contribution_policy_digest,
        )),
        variant_digest: String::new(),
    };
    value.variant_digest = variant_digest_v3(&value);
    validate_variant_v3(&value)?;
    Ok(value)
}

fn derive_representation_registration_v3(
    history: &FrozenHistoryV3,
    audit: &MomentumRepresentationPathAuditV3,
    split: &MomentumRepresentationSplitV3,
) -> Result<MomentumRepresentationRegistrationV3, String> {
    validate_audit_v3(audit)?;
    validate_split_v3(split)?;
    let config = MomentumLearningCampaignConfigV0::default();
    let contribution_policy_digest = contribution_policy_digest_v3();
    let base_seed = config.campaign_seed ^ 0x79B0_0000;
    let variants = vec![
        representation_variant_v3(
            "last-output-control",
            MomentumRepresentationInputKindV3::MambaLastOutput,
            "LastOutput",
            base_seed ^ 1,
            history,
            &config,
            &contribution_policy_digest,
        )?,
        representation_variant_v3(
            "mean-output",
            MomentumRepresentationInputKindV3::MambaMeanOutput,
            "MeanOutput",
            base_seed ^ 2,
            history,
            &config,
            &contribution_policy_digest,
        )?,
        representation_variant_v3(
            "last-mean-concat",
            MomentumRepresentationInputKindV3::MambaLastMeanConcat,
            "LastOutput+MeanOutput",
            base_seed ^ 3,
            history,
            &config,
            &contribution_policy_digest,
        )?,
        representation_variant_v3(
            "raw-feature-residual",
            MomentumRepresentationInputKindV3::MambaRawFeatureResidual,
            "LastOutput+SequenceEndRawFeatureResidual",
            base_seed ^ 4,
            history,
            &config,
            &contribution_policy_digest,
        )?,
    ];
    let mut registration = MomentumRepresentationRegistrationV3 {
        registration_version: REGISTRATION_VERSION_V3.to_string(),
        agent_id: AGENT_ID_V3.to_string(),
        source_snapshot_digest: history.v1.snapshot.content_digest.clone(),
        canonical_intent_digest: history.v1.input.input.intent.intent_digest.clone(),
        canonical_view_digest: history.v1.input.input.view.view_digest.clone(),
        representation_audit_digest: audit.audit_digest.clone(),
        split_digest: split.split_digest.clone(),
        variants,
        maximum_variants: MAXIMUM_VARIANTS_V3,
        contribution_policy_digest,
        fresh_validation_hidden: true,
        historical_test_forbidden: true,
        future_evaluation_forbidden: true,
        winner_selection_forbidden: true,
        active_promotion_forbidden: true,
        reward_application_forbidden: true,
        registration_digest: String::new(),
    };
    registration.registration_digest = registration_digest_v3(&registration);
    validate_registration_v3(&registration)?;
    Ok(registration)
}

fn metric_is_finite_v3(metric: &EvaluationMetricsV0) -> bool {
    metric.brier_score.is_finite()
        && metric.accuracy.is_finite()
        && metric.positive_label_rate.is_finite()
        && metric.mean_predicted_probability.is_finite()
}

fn private_metric_digest_v3(model_kind: &str, metric: &EvaluationMetricsV0) -> String {
    stable_hash_string(&format!(
        "private-fresh-validation-metric-v3:{model_kind}:{metric:?}"
    ))
}

fn make_participant_v3(
    role: MomentumRepresentationParticipantRoleV3,
    model_kind: String,
    input_kind: String,
    variant_digest: Option<String>,
    parameter_digest: String,
    feature_normalizer_digest: String,
    representation_normalizer_digest: String,
    encoder_digest: Option<String>,
    training_policy_digest: String,
    initialization_digest: String,
    history: &FrozenHistoryV3,
    split: &MomentumRepresentationSplitV3,
    validation_timestamp_digest: &str,
) -> Result<FrozenCandidateParticipantV3, String> {
    let learned = matches!(
        role,
        MomentumRepresentationParticipantRoleV3::MambaOnly
            | MomentumRepresentationParticipantRoleV3::MambaResidualHybrid
    );
    let participant_id = format!(
        "{}-{}",
        AGENT_ID_V3,
        stable_hash_string(&format!(
            "participant-id-v3:{role:?}:{model_kind}:{parameter_digest}:{initialization_digest}"
        ))
    );
    let model_artifact_digest = stable_hash_string(&format!(
        "model-artifact-v3:{model_kind}:{parameter_digest}:{feature_normalizer_digest}:{representation_normalizer_digest}:{encoder_digest:?}:{training_policy_digest}"
    ));
    let mut participant = FrozenCandidateParticipantV3 {
        participant_version: PARTICIPANT_VERSION_V3.to_string(),
        participant_id,
        participant_role: role,
        model_kind,
        input_kind,
        variant_digest,
        source_snapshot_digest: history.v1.snapshot.content_digest.clone(),
        training_range_digest: range_digest_v3("training", &split.training_range),
        fresh_validation_range_digest: range_digest_v3(
            "fresh-validation",
            &split.fresh_validation_range,
        ),
        validation_timestamp_digest: validation_timestamp_digest.to_string(),
        model_artifact_digest,
        parameter_digest,
        feature_normalizer_digest,
        representation_normalizer_digest,
        encoder_digest,
        feature_policy_digest: history.v1.session.feature_policy_digest.clone(),
        label_policy_digest: history.v1.session.label_policy_digest.clone(),
        training_policy_digest,
        initialization_digest,
        warm_start: false,
        v1_head_reused: false,
        v2_head_reused: false,
        fresh_deterministic_initialization: true,
        encoder_frozen: learned,
        deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
        participant_digest: String::new(),
    };
    participant.participant_digest = participant_digest_v3(&participant);
    validate_participant_v3(&participant)?;
    Ok(participant)
}

fn make_receipt_v3(
    participant: &FrozenCandidateParticipantV3,
    split: &MomentumRepresentationSplitV3,
    private_metric_digest: String,
    contribution_audit_digest: Option<String>,
    status: MomentumRepresentationQualificationStatusV3,
) -> Result<MomentumRepresentationQualificationReceiptV3, String> {
    let mut receipt = MomentumRepresentationQualificationReceiptV3 {
        receipt_version: RECEIPT_VERSION_V3.to_string(),
        participant_id: participant.participant_id.clone(),
        participant_digest: participant.participant_digest.clone(),
        input_kind: participant.input_kind.clone(),
        fresh_validation_range_digest: range_digest_v3(
            "fresh-validation",
            &split.fresh_validation_range,
        ),
        qualification_policy_digest: stable_hash_string(&format!(
            "fresh-representation-qualification-v3:{:?}:finite:min-samples:noncollapse:zero-validation-updates:no-ranking",
            participant.participant_role
        )),
        private_metric_digest,
        contribution_audit_digest,
        status,
        validation_parameter_updates: 0,
        historical_test_reads: 0,
        future_evaluation_reads: 0,
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = receipt_digest_v3(&receipt);
    validate_receipt_v3(&receipt)?;
    Ok(receipt)
}

fn learned_base_qualification_v3(
    metric: &EvaluationMetricsV0,
    probabilities: &[f32],
    diagnostic: &RepresentationDiagnosticV3,
    minimum_samples: usize,
    encoder_unchanged: bool,
) -> MomentumRepresentationQualificationStatusV3 {
    if metric.sample_count < minimum_samples || probabilities.len() != metric.sample_count {
        return MomentumRepresentationQualificationStatusV3::RejectedInsufficientValidation;
    }
    if !metric_is_finite_v3(metric)
        || probabilities.iter().any(|value| !value.is_finite())
        || !diagnostic.finite
    {
        return MomentumRepresentationQualificationStatusV3::RejectedNumericalFailure;
    }
    if !encoder_unchanged
        || matches!(
            diagnostic.variance,
            VarianceClassV3::NearConstant | VarianceClassV3::Low
        )
        || matches!(
            diagnostic.effective_rank,
            RankClassV3::RankOneOrLess | RankClassV3::Low
        )
    {
        return MomentumRepresentationQualificationStatusV3::RejectedRepresentationInvariant;
    }
    match probabilities_collapsed_v3(probabilities) {
        Ok(true) => MomentumRepresentationQualificationStatusV3::RejectedProbabilityCollapse,
        Ok(false) => MomentumRepresentationQualificationStatusV3::QualifiedMambaOnly,
        Err(_) => MomentumRepresentationQualificationStatusV3::RejectedNumericalFailure,
    }
}

fn comparator_qualification_v3(
    role: MomentumRepresentationParticipantRoleV3,
    metric: &EvaluationMetricsV0,
    probabilities: &[f32],
    minimum_samples: usize,
) -> MomentumRepresentationQualificationStatusV3 {
    if metric.sample_count < minimum_samples || probabilities.len() != metric.sample_count {
        return MomentumRepresentationQualificationStatusV3::RejectedInsufficientValidation;
    }
    if !metric_is_finite_v3(metric) || probabilities.iter().any(|value| !value.is_finite()) {
        return MomentumRepresentationQualificationStatusV3::RejectedNumericalFailure;
    }
    match role {
        MomentumRepresentationParticipantRoleV3::LinearComparator => {
            MomentumRepresentationQualificationStatusV3::ComparatorQualified
        }
        MomentumRepresentationParticipantRoleV3::ConstantBenchmark => {
            MomentumRepresentationQualificationStatusV3::BenchmarkQualified
        }
        _ => MomentumRepresentationQualificationStatusV3::RejectedContributionInvariant,
    }
}

fn prediction_digest_v3(label: &str, probabilities: &[f32]) -> String {
    stable_hash_string(&format!(
        "prediction-digest-v3:{label}:{:?}",
        probabilities
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    ))
}

fn ablated_probabilities_v3(
    head: &LogisticPredictionHeadV0,
    rows: &[EncodedTrainingExampleV0],
    start: usize,
    end: usize,
) -> Result<Vec<f32>, String> {
    if start >= end || end > head.weights.len() {
        return Err("V3 ablation block rejected".to_string());
    }
    rows.iter()
        .map(|row| {
            let mut representation = row.representation.clone();
            if representation.len() != head.weights.len() {
                return Err("V3 ablation representation rejected".to_string());
            }
            representation[start..end].fill(0.0);
            head.probability(&representation)
                .map_err(|_| "V3 ablation probability failed".to_string())
        })
        .collect()
}

fn mean_absolute_effect_v3(left: &[f32], right: &[f32]) -> Result<f32, String> {
    if left.is_empty() || left.len() != right.len() {
        return Err("V3 ablation effect evidence rejected".to_string());
    }
    let value = left
        .iter()
        .zip(right)
        .map(|(left, right)| (left - right).abs())
        .sum::<f32>()
        / left.len() as f32;
    value
        .is_finite()
        .then_some(value)
        .ok_or_else(|| "V3 ablation effect was non-finite".to_string())
}

fn effect_status_v3(effect: f32) -> &'static str {
    if effect >= f32::from_bits(MAMBA_MATERIAL_EFFECT_BITS_V3) {
        "Material"
    } else if effect >= f32::from_bits(MAMBA_DETECTABLE_EFFECT_BITS_V3) {
        "DetectableBelowPolicy"
    } else {
        "NotDetectable"
    }
}

fn not_applicable_contribution_v3(
    participant: &FrozenCandidateParticipantV3,
    head: &LogisticPredictionHeadV0,
    probabilities: &[f32],
) -> Result<MambaContributionAuditV3, String> {
    let mut audit = MambaContributionAuditV3 {
        participant_digest: participant.participant_digest.clone(),
        mamba_parameter_block_digest: stable_hash_string(&format!(
            "mamba-only-parameter-block-v3:{:?}",
            head.weights
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        )),
        raw_parameter_block_digest: stable_hash_string("raw-parameter-block-not-applicable-v3"),
        mamba_block_nonzero: head.weights.iter().any(|value| value.abs() > 1e-12),
        raw_block_nonzero: false,
        full_prediction_digest: prediction_digest_v3("full", probabilities),
        mamba_ablated_prediction_digest: stable_hash_string("mamba-ablation-not-applicable-v3"),
        raw_ablated_prediction_digest: stable_hash_string("raw-ablation-not-applicable-v3"),
        mamba_ablation_effect_status: "NotApplicable".to_string(),
        raw_ablation_effect_status: "NotApplicable".to_string(),
        contribution_policy_digest: contribution_policy_digest_v3(),
        contribution_status: MambaContributionStatusV3::NotApplicable,
        audit_digest: String::new(),
    };
    audit.audit_digest = contribution_digest_v3(&audit);
    validate_contribution_v3(&audit)?;
    Ok(audit)
}

fn residual_contribution_v3(
    participant: &FrozenCandidateParticipantV3,
    head: &LogisticPredictionHeadV0,
    validation: &[EncodedTrainingExampleV0],
    mamba_dimension: usize,
    full_probabilities: &[f32],
) -> Result<MambaContributionAuditV3, String> {
    if mamba_dimension == 0 || mamba_dimension >= head.weights.len() {
        return Err("V3 residual block dimensions rejected".to_string());
    }
    let mamba_ablated = ablated_probabilities_v3(head, validation, 0, mamba_dimension)?;
    let raw_ablated =
        ablated_probabilities_v3(head, validation, mamba_dimension, head.weights.len())?;
    let mamba_effect = mean_absolute_effect_v3(full_probabilities, &mamba_ablated)?;
    let raw_effect = mean_absolute_effect_v3(full_probabilities, &raw_ablated)?;
    let mamba_block_nonzero = head.weights[..mamba_dimension]
        .iter()
        .any(|value| value.abs() > 1e-12);
    let raw_block_nonzero = head.weights[mamba_dimension..]
        .iter()
        .any(|value| value.abs() > 1e-12);
    let contribution_status = if !mamba_block_nonzero || !raw_block_nonzero {
        MambaContributionStatusV3::Invalid
    } else if mamba_effect >= f32::from_bits(MAMBA_MATERIAL_EFFECT_BITS_V3) {
        MambaContributionStatusV3::MaterialContribution
    } else if raw_effect >= f32::from_bits(MAMBA_MATERIAL_EFFECT_BITS_V3) {
        MambaContributionStatusV3::RawFeatureDominated
    } else if mamba_effect >= f32::from_bits(MAMBA_DETECTABLE_EFFECT_BITS_V3) {
        MambaContributionStatusV3::DetectableButBelowPolicy
    } else {
        MambaContributionStatusV3::NoDetectableContribution
    };
    let mut audit = MambaContributionAuditV3 {
        participant_digest: participant.participant_digest.clone(),
        mamba_parameter_block_digest: stable_hash_string(&format!(
            "mamba-parameter-block-v3:{:?}",
            head.weights[..mamba_dimension]
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        )),
        raw_parameter_block_digest: stable_hash_string(&format!(
            "raw-parameter-block-v3:{:?}",
            head.weights[mamba_dimension..]
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        )),
        mamba_block_nonzero,
        raw_block_nonzero,
        full_prediction_digest: prediction_digest_v3("residual-full", full_probabilities),
        mamba_ablated_prediction_digest: prediction_digest_v3(
            "residual-mamba-ablated",
            &mamba_ablated,
        ),
        raw_ablated_prediction_digest: prediction_digest_v3("residual-raw-ablated", &raw_ablated),
        mamba_ablation_effect_status: effect_status_v3(mamba_effect).to_string(),
        raw_ablation_effect_status: effect_status_v3(raw_effect).to_string(),
        contribution_policy_digest: contribution_policy_digest_v3(),
        contribution_status,
        audit_digest: String::new(),
    };
    audit.audit_digest = contribution_digest_v3(&audit);
    validate_contribution_v3(&audit)?;
    Ok(audit)
}

fn residual_qualification_v3(
    base: MomentumRepresentationQualificationStatusV3,
    contribution: &MambaContributionAuditV3,
) -> (
    MomentumResidualQualificationV3,
    MomentumRepresentationQualificationStatusV3,
) {
    match base {
        MomentumRepresentationQualificationStatusV3::RejectedProbabilityCollapse => (
            MomentumResidualQualificationV3::RejectedProbabilityCollapse,
            MomentumRepresentationQualificationStatusV3::RejectedProbabilityCollapse,
        ),
        MomentumRepresentationQualificationStatusV3::RejectedNumericalFailure => (
            MomentumResidualQualificationV3::RejectedNumericalFailure,
            MomentumRepresentationQualificationStatusV3::RejectedNumericalFailure,
        ),
        MomentumRepresentationQualificationStatusV3::QualifiedMambaOnly => {
            match contribution.contribution_status {
                MambaContributionStatusV3::MaterialContribution => (
                    MomentumResidualQualificationV3::QualifiedMambaContributingHybrid,
                    MomentumRepresentationQualificationStatusV3::QualifiedMambaContributingHybrid,
                ),
                MambaContributionStatusV3::RawFeatureDominated => (
                    MomentumResidualQualificationV3::QualifiedRawFallbackNotMamba,
                    MomentumRepresentationQualificationStatusV3::QualifiedRawFallbackNotMamba,
                ),
                _ => (
                    MomentumResidualQualificationV3::RejectedContributionInvariant,
                    MomentumRepresentationQualificationStatusV3::RejectedContributionInvariant,
                ),
            }
        }
        _ => (
            MomentumResidualQualificationV3::RejectedContributionInvariant,
            base,
        ),
    }
}

fn normalized_route_partitions_v3(
    input_kind: MomentumRepresentationInputKindV3,
    last_training: &[EncodedTrainingExampleV0],
    last_validation: &[EncodedTrainingExampleV0],
    mean_training: &[EncodedTrainingExampleV0],
    mean_validation: &[EncodedTrainingExampleV0],
    raw_training: &[EncodedTrainingExampleV0],
    raw_validation: &[EncodedTrainingExampleV0],
) -> Result<
    (
        Vec<EncodedTrainingExampleV0>,
        Vec<EncodedTrainingExampleV0>,
        String,
        usize,
    ),
    String,
> {
    let (training, validation, residual_raw) = match input_kind {
        MomentumRepresentationInputKindV3::MambaLastOutput => {
            (last_training.to_vec(), last_validation.to_vec(), false)
        }
        MomentumRepresentationInputKindV3::MambaMeanOutput => {
            (mean_training.to_vec(), mean_validation.to_vec(), false)
        }
        MomentumRepresentationInputKindV3::MambaLastMeanConcat => (
            concatenate_encoded_v3(last_training, mean_training)?,
            concatenate_encoded_v3(last_validation, mean_validation)?,
            false,
        ),
        MomentumRepresentationInputKindV3::MambaRawFeatureResidual => {
            (last_training.to_vec(), last_validation.to_vec(), true)
        }
    };
    let normalizer = RepresentationNormalizerV0::fit(&training)
        .map_err(|_| "V3 representation normalizer fit failed".to_string())?;
    let normalized_training = normalizer
        .transform(&training)
        .map_err(|_| "V3 training representation normalization failed".to_string())?;
    let normalized_validation = normalizer
        .transform(&validation)
        .map_err(|_| "V3 validation representation normalization failed".to_string())?;
    let mamba_dimension = normalized_training[0].representation.len();
    if residual_raw {
        Ok((
            concatenate_encoded_v3(&normalized_training, raw_training)?,
            concatenate_encoded_v3(&normalized_validation, raw_validation)?,
            stable_hash_string(&format!(
                "residual-block-normalizers-v3:{}:raw-input-feature-normalized",
                normalizer.digest()
            )),
            mamba_dimension,
        ))
    } else {
        Ok((
            normalized_training,
            normalized_validation,
            normalizer.digest(),
            mamba_dimension,
        ))
    }
}

fn derive_route_decision_v3(
    history: &FrozenHistoryV3,
    family: &MomentumRepresentationFamilyV3,
) -> Result<MomentumRepresentationRouteDecisionArtifactV3, String> {
    validate_family_v3(family)?;
    let status_for = |digest: &str| {
        family
            .qualification_receipts
            .iter()
            .find(|receipt| receipt.participant_digest == digest)
            .map(|receipt| receipt.status)
    };
    let mut qualified_mamba_only_digests = Vec::new();
    let mut qualified_mamba_hybrid_digests = Vec::new();
    let mut raw_fallback_digests = Vec::new();
    let mut rejected_route_digests = Vec::new();
    for participant in &family.participants {
        if !matches!(
            participant.participant_role,
            MomentumRepresentationParticipantRoleV3::MambaOnly
                | MomentumRepresentationParticipantRoleV3::MambaResidualHybrid
        ) {
            continue;
        }
        match status_for(&participant.participant_digest) {
            Some(MomentumRepresentationQualificationStatusV3::QualifiedMambaOnly) => {
                qualified_mamba_only_digests.push(participant.participant_digest.clone())
            }
            Some(MomentumRepresentationQualificationStatusV3::QualifiedMambaContributingHybrid) => {
                qualified_mamba_hybrid_digests.push(participant.participant_digest.clone())
            }
            Some(MomentumRepresentationQualificationStatusV3::QualifiedRawFallbackNotMamba) => {
                raw_fallback_digests.push(participant.participant_digest.clone())
            }
            _ => rejected_route_digests.push(participant.participant_digest.clone()),
        }
    }
    qualified_mamba_only_digests = sorted_unique(qualified_mamba_only_digests);
    qualified_mamba_hybrid_digests = sorted_unique(qualified_mamba_hybrid_digests);
    raw_fallback_digests = sorted_unique(raw_fallback_digests);
    rejected_route_digests = sorted_unique(rejected_route_digests);
    let decision = if !qualified_mamba_only_digests.is_empty() {
        MomentumRepresentationRouteDecisionV3::FrozenMambaOnlyViable
    } else if !qualified_mamba_hybrid_digests.is_empty() {
        MomentumRepresentationRouteDecisionV3::MambaResidualHybridViable
    } else if !raw_fallback_digests.is_empty() {
        MomentumRepresentationRouteDecisionV3::RawFeatureFallbackOnly
    } else {
        MomentumRepresentationRouteDecisionV3::AllRepresentationRoutesCollapsed
    };
    let no_genuine =
        qualified_mamba_only_digests.is_empty() && qualified_mamba_hybrid_digests.is_empty();
    let mut artifact = MomentumRepresentationRouteDecisionArtifactV3 {
        decision_version: DECISION_VERSION_V3.to_string(),
        v1_family_digest: history.v1.family.family_digest.clone(),
        v2_family_digest: history.v2_family.family_digest.clone(),
        v3_family_digest: family.family_digest.clone(),
        qualified_mamba_only_digests,
        qualified_mamba_hybrid_digests,
        raw_fallback_digests,
        rejected_route_digests,
        further_head_only_repair_forbidden: true,
        further_frozen_representation_sweep_forbidden: no_genuine,
        decision,
        decision_digest: String::new(),
    };
    artifact.decision_digest = decision_digest_v3(&artifact);
    validate_decision_v3(&artifact, family)?;
    Ok(artifact)
}

fn derive_roster_v3(
    family: &MomentumRepresentationFamilyV3,
    decision: &MomentumRepresentationRouteDecisionArtifactV3,
) -> Result<
    (
        Option<MomentumRepresentationFutureRosterV3>,
        MomentumRepresentationRosterStatusV3,
    ),
    String,
> {
    validate_decision_v3(decision, family)?;
    let qualified_genuine_mamba_digests = sorted_unique(
        decision
            .qualified_mamba_only_digests
            .iter()
            .chain(&decision.qualified_mamba_hybrid_digests)
            .cloned()
            .collect(),
    );
    let qualified_comparator_digests = sorted_unique(
        family
            .qualification_receipts
            .iter()
            .filter(|receipt| {
                matches!(
                    receipt.status,
                    MomentumRepresentationQualificationStatusV3::ComparatorQualified
                        | MomentumRepresentationQualificationStatusV3::BenchmarkQualified
                )
            })
            .map(|receipt| receipt.participant_digest.clone())
            .collect(),
    );
    if qualified_genuine_mamba_digests.is_empty() {
        return Ok((
            None,
            MomentumRepresentationRosterStatusV3::FrozenMambaRepresentationPathRejected,
        ));
    }
    if qualified_comparator_digests.is_empty() {
        return Ok((
            None,
            MomentumRepresentationRosterStatusV3::InsufficientComparators,
        ));
    }
    let included = qualified_genuine_mamba_digests
        .iter()
        .chain(&qualified_comparator_digests)
        .collect::<BTreeSet<_>>();
    let excluded_participant_digests = sorted_unique(
        family
            .participants
            .iter()
            .filter(|participant| !included.contains(&participant.participant_digest))
            .map(|participant| participant.participant_digest.clone())
            .collect(),
    );
    let mut roster = MomentumRepresentationFutureRosterV3 {
        roster_version: ROSTER_VERSION_V3.to_string(),
        family_digest: family.family_digest.clone(),
        decision_digest: decision.decision_digest.clone(),
        qualified_genuine_mamba_digests,
        qualified_comparator_digests,
        excluded_participant_digests,
        inclusion_policy_digest: stable_hash_string(
            "future-roster-v3:all-qualified-genuine-mamba:all-qualified-comparators:no-raw-fallback:no-ranking",
        ),
        roster_digest: String::new(),
    };
    roster.roster_digest = roster_digest_v3(&roster);
    validate_roster_v3(&roster, family, decision)?;
    Ok((
        Some(roster),
        MomentumRepresentationRosterStatusV3::Registered,
    ))
}

fn derive_evaluation_registration_v3(
    history: &FrozenHistoryV3,
    split: &MomentumRepresentationSplitV3,
    family: &MomentumRepresentationFamilyV3,
    decision: &MomentumRepresentationRouteDecisionArtifactV3,
    roster: &MomentumRepresentationFutureRosterV3,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<MomentumRepresentationEvaluationRegistrationV3, String> {
    validate_roster_v3(roster, family, decision)?;
    let source_boundary_timestamp_ms = history
        .v1
        .snapshot
        .actual_end_timestamp_ms
        .ok_or_else(|| "V3 source boundary unavailable".to_string())?;
    let protected_next = reservation
        .reserved_timestamp_ms
        .last()
        .and_then(|timestamp| timestamp.checked_add(reservation.cadence_ms))
        .ok_or_else(|| "V3 protected future boundary unavailable".to_string())?;
    let source_next = source_boundary_timestamp_ms
        .checked_add(reservation.cadence_ms)
        .ok_or_else(|| "V3 source boundary overflow".to_string())?;
    let included = roster
        .qualified_genuine_mamba_digests
        .iter()
        .chain(&roster.qualified_comparator_digests)
        .collect::<BTreeSet<_>>();
    let qualification_receipt_digests = sorted_unique(
        family
            .qualification_receipts
            .iter()
            .filter(|receipt| included.contains(&receipt.participant_digest))
            .map(|receipt| receipt.receipt_digest.clone())
            .collect(),
    );
    let contribution_audit_digests = sorted_unique(
        family
            .contribution_audits
            .iter()
            .filter(|audit| included.contains(&audit.participant_digest))
            .map(|audit| audit.audit_digest.clone())
            .collect(),
    );
    let prior_validation_and_reserved_range_digests = sorted_unique(vec![
        range_digest_v3("v1-prior-validation", &history.v1.prior_validation_range),
        range_digest_v3("v1-prior-reserved", &history.v1.prior_reserved_range),
        range_digest_v3(
            "v2-prior-validation",
            &history.v2_split.fresh_repair_validation_range,
        ),
        range_digest_v3(
            "v2-remaining-reserved",
            history
                .v2_split
                .remaining_reserved_range
                .as_ref()
                .ok_or_else(|| "V2 remaining reserve unavailable".to_string())?,
        ),
        range_digest_v3("v3-fresh-validation", &split.fresh_validation_range),
        range_digest_v3("v3-final-reserved", &split.final_reserved_range),
    ]);
    let mut registration = MomentumRepresentationEvaluationRegistrationV3 {
        registration_version: EVALUATION_VERSION_V3.to_string(),
        agent_id: AGENT_ID_V3.to_string(),
        family_digest: family.family_digest.clone(),
        roster_digest: roster.roster_digest.clone(),
        decision_digest: decision.decision_digest.clone(),
        qualification_receipt_digests,
        contribution_audit_digests,
        source_snapshot_digest: history.v1.snapshot.content_digest.clone(),
        source_boundary_timestamp_ms,
        protected_registration_digests: sorted_unique(
            reservation.protected_registration_digests.clone(),
        ),
        protected_timestamp_ms: reservation.reserved_timestamp_ms.clone(),
        prior_validation_and_reserved_range_digests,
        provider_finality_boundary_ms: reservation.provider_finality_boundary_ms,
        minimum_accepted_timestamp_ms: source_next
            .max(protected_next)
            .max(reservation.provider_finality_boundary_ms),
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
    registration.registration_digest = evaluation_digest_v3(&registration);
    validate_evaluation_v3(&registration, family, decision, roster)?;
    Ok(registration)
}

fn examples_with_labels_in_range_v3(
    examples: &[SequenceExampleV0],
    range: &IndexRangeV0,
) -> Vec<SequenceExampleV0> {
    examples
        .iter()
        .filter(|example| example.label_index >= range.start && example.label_index < range.end)
        .cloned()
        .collect()
}

fn run_representation_experiment_v3(
    history: &FrozenHistoryV3,
    audit: &MomentumRepresentationPathAuditV3,
    split: &MomentumRepresentationSplitV3,
    registration: &MomentumRepresentationRegistrationV3,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<RepresentationExperimentV3, String> {
    validate_audit_v3(audit)?;
    validate_split_v3(split)?;
    validate_registration_v3(registration)?;
    if registration.representation_audit_digest != audit.audit_digest
        || registration.split_digest != split.split_digest
        || registration.source_snapshot_digest != history.v1.snapshot.content_digest
    {
        return Err("V3 execution preregistration binding rejected".to_string());
    }
    let config = MomentumLearningCampaignConfigV0::default();
    let candles =
        candles_from_snapshot_prefix(&history.v1.snapshot, split.fresh_validation_range.end)?;
    let raw_features = build_momentum_features_v0(&candles, &config.feature_config)
        .map_err(|_| "V3 feature derivation failed".to_string())?;
    let training_features = raw_features
        .iter()
        .filter(|row| row.source_index < split.training_range.end)
        .cloned()
        .collect::<Vec<_>>();
    let feature_normalizer = FeatureNormalizerV0::fit(&training_features)
        .map_err(|_| "V3 feature normalizer fit failed".to_string())?;
    if feature_normalizer.fitted_on_end >= split.training_range.end {
        return Err("V3 feature normalizer crossed training boundary".to_string());
    }
    let normalized_features = feature_normalizer
        .transform(&raw_features)
        .map_err(|_| "V3 feature transform failed".to_string())?;
    let examples = build_momentum_sequence_examples_v0(
        &candles,
        &normalized_features,
        &config.sequence_config,
        std::slice::from_ref(&history.v1.snapshot.snapshot_id),
    )
    .map_err(|_| "V3 sequence derivation failed".to_string())?;
    let training = examples_with_labels_in_range_v3(&examples, &split.training_range);
    let validation = examples_with_labels_in_range_v3(&examples, &split.fresh_validation_range);
    if training.is_empty()
        || validation.len() < split.minimum_validation_samples
        || training
            .iter()
            .any(|example| example.label_index >= split.training_range.end)
        || validation.iter().any(|example| {
            example.sequence_start < split.purge_range.start
                || example.label_index < split.fresh_validation_range.start
                || example.label_index >= split.fresh_validation_range.end
        })
        || examples
            .iter()
            .any(|example| example.label_index >= split.final_reserved_range.start)
    {
        return Err("V3 fresh validation partition rejected".to_string());
    }
    let validation_timestamp_digest = stable_hash_string(&format!(
        "fresh-validation-timestamps-v3:{:?}",
        validation
            .iter()
            .map(
                |example| history.v1.snapshot.normalized_dataset.rows[example.label_index]
                    .timestamp_ms
            )
            .collect::<Vec<_>>()
    ));
    let raw_training = raw_end_encoded_v3(&training)?;
    let raw_validation = raw_end_encoded_v3(&validation)?;
    let mut last_encoder = frozen_mamba3_encoder_from_seed_v0(
        &config.feature_config,
        config.campaign_seed,
        config.backend_preference,
        config.fallback_policy,
    )
    .map_err(|_| "V3 encoder unavailable".to_string())?;
    last_encoder.pooling = SequencePooling::LastOutput;
    let mut mean_encoder = last_encoder.clone();
    mean_encoder.pooling = SequencePooling::MeanOutput;
    let encoder_digest = last_encoder.parameter_digest();
    let last_training = last_encoder
        .encode_batch(&training)
        .map_err(|_| "V3 last training representation failed".to_string())?;
    let last_validation = last_encoder
        .encode_batch(&validation)
        .map_err(|_| "V3 last validation representation failed".to_string())?;
    let mean_training = mean_encoder
        .encode_batch(&training)
        .map_err(|_| "V3 mean training representation failed".to_string())?;
    let mean_validation = mean_encoder
        .encode_batch(&validation)
        .map_err(|_| "V3 mean validation representation failed".to_string())?;
    if mean_encoder.parameter_digest() != encoder_digest {
        return Err("V3 frozen encoder identity diverged".to_string());
    }
    let mut participants = Vec::new();
    let mut receipts = Vec::new();
    let mut contribution_audits = Vec::new();
    for variant in &registration.variants {
        let (route_training, route_validation, representation_normalizer_digest, mamba_dimension) =
            normalized_route_partitions_v3(
                variant.input_kind,
                &last_training,
                &last_validation,
                &mean_training,
                &mean_validation,
                &raw_training,
                &raw_validation,
            )?;
        let initial_head = LogisticPredictionHeadV0::seeded(
            route_training[0].representation.len(),
            variant.initialization_seed,
        )
        .map_err(|_| "V3 fresh head initialization failed".to_string())?;
        let initial_digest = initial_head.parameter_digest();
        let mut training_config = HeadTrainingConfigV0::default();
        training_config.epochs = variant.maximum_epochs;
        training_config.seed = variant.initialization_seed;
        training_config.early_stopping_patience = None;
        training_config.optimizer.learning_rate = f32::from_bits(variant.learning_rate_bits);
        training_config.optimizer.weight_decay = f32::from_bits(variant.l2_regularization_bits);
        let head = train_encoded_head_v3(initial_head, &route_training, &training_config)?;
        let metric = evaluate_head_v0(&head, &route_validation)
            .map_err(|_| "V3 route metric failed".to_string())?;
        let probabilities = route_validation
            .iter()
            .map(|row| head.probability(&row.representation))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "V3 route probability failed".to_string())?;
        let diagnostic = representation_diagnostic_v3(&route_validation);
        let encoder_unchanged = last_encoder.parameter_digest() == encoder_digest
            && mean_encoder.parameter_digest() == encoder_digest;
        let base_status = learned_base_qualification_v3(
            &metric,
            &probabilities,
            &diagnostic,
            split.minimum_validation_samples,
            encoder_unchanged,
        );
        let (role, model_kind) = match variant.input_kind {
            MomentumRepresentationInputKindV3::MambaLastOutput => (
                MomentumRepresentationParticipantRoleV3::MambaOnly,
                "FrozenMambaLastOutputLogisticV3",
            ),
            MomentumRepresentationInputKindV3::MambaMeanOutput => (
                MomentumRepresentationParticipantRoleV3::MambaOnly,
                "FrozenMambaMeanOutputLogisticV3",
            ),
            MomentumRepresentationInputKindV3::MambaLastMeanConcat => (
                MomentumRepresentationParticipantRoleV3::MambaOnly,
                "FrozenMambaLastMeanConcatLogisticV3",
            ),
            MomentumRepresentationInputKindV3::MambaRawFeatureResidual => (
                MomentumRepresentationParticipantRoleV3::MambaResidualHybrid,
                "FrozenMambaRawResidualLogisticV3",
            ),
        };
        let participant = make_participant_v3(
            role,
            model_kind.to_string(),
            format!("{:?}", variant.input_kind),
            Some(variant.variant_digest.clone()),
            head.parameter_digest(),
            feature_normalizer.digest(),
            representation_normalizer_digest,
            Some(encoder_digest.clone()),
            variant.training_policy_digest.clone(),
            stable_hash_string(&format!(
                "fresh-deterministic-head-v3:{}:{}",
                variant.initialization_seed, initial_digest
            )),
            history,
            split,
            &validation_timestamp_digest,
        )?;
        let contribution = if role == MomentumRepresentationParticipantRoleV3::MambaResidualHybrid {
            residual_contribution_v3(
                &participant,
                &head,
                &route_validation,
                mamba_dimension,
                &probabilities,
            )?
        } else {
            not_applicable_contribution_v3(&participant, &head, &probabilities)?
        };
        let status = if role == MomentumRepresentationParticipantRoleV3::MambaResidualHybrid {
            residual_qualification_v3(base_status, &contribution).1
        } else {
            base_status
        };
        let receipt = make_receipt_v3(
            &participant,
            split,
            private_metric_digest_v3(model_kind, &metric),
            (role == MomentumRepresentationParticipantRoleV3::MambaResidualHybrid)
                .then_some(contribution.audit_digest.clone()),
            status,
        )?;
        participants.push(participant);
        receipts.push(receipt);
        contribution_audits.push(contribution);
    }
    let mut linear_config = HeadTrainingConfigV0::default();
    linear_config.seed = config.campaign_seed ^ 0x79C0_0001;
    linear_config.early_stopping_patience = None;
    let linear = LinearMomentumBaselineV0::train(&training, &training, &linear_config)
        .map_err(|_| "V3 linear comparator training failed".to_string())?;
    let linear_metric = linear
        .evaluate(&validation)
        .map_err(|_| "V3 linear comparator metric failed".to_string())?;
    let linear_probabilities = raw_validation
        .iter()
        .map(|row| linear.head.probability(&row.representation))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "V3 linear comparator probability failed".to_string())?;
    let linear_status = comparator_qualification_v3(
        MomentumRepresentationParticipantRoleV3::LinearComparator,
        &linear_metric,
        &linear_probabilities,
        split.minimum_validation_samples,
    );
    let linear_participant = make_participant_v3(
        MomentumRepresentationParticipantRoleV3::LinearComparator,
        "LinearMomentumBaselineV3".to_string(),
        "RawFeatureLinearComparator".to_string(),
        None,
        linear.head.parameter_digest(),
        feature_normalizer.digest(),
        stable_hash_string("linear-representation-normalizer-not-applicable-v3"),
        None,
        stable_hash_string(&format!(
            "linear-training-policy-v3:{}",
            linear_config.digest()
        )),
        stable_hash_string(&format!(
            "fresh-linear-initialization-v3:{}",
            linear_config.seed
        )),
        history,
        split,
        &validation_timestamp_digest,
    )?;
    let linear_receipt = make_receipt_v3(
        &linear_participant,
        split,
        private_metric_digest_v3("LinearMomentumBaselineV3", &linear_metric),
        None,
        linear_status,
    )?;
    participants.push(linear_participant);
    receipts.push(linear_receipt);
    let constant = ConstantProbabilityBaselineV0::fit(&training)
        .map_err(|_| "V3 constant benchmark training failed".to_string())?;
    let constant_metric = constant
        .evaluate(&validation)
        .map_err(|_| "V3 constant benchmark metric failed".to_string())?;
    let constant_probabilities = vec![constant.probability; validation.len()];
    let constant_status = comparator_qualification_v3(
        MomentumRepresentationParticipantRoleV3::ConstantBenchmark,
        &constant_metric,
        &constant_probabilities,
        split.minimum_validation_samples,
    );
    let constant_participant = make_participant_v3(
        MomentumRepresentationParticipantRoleV3::ConstantBenchmark,
        "ConstantProbabilityBaselineV3".to_string(),
        "TrainingPrevalenceConstant".to_string(),
        None,
        stable_hash_string(&format!(
            "training-prevalence-constant-v3:{}:{}",
            constant.probability.to_bits(),
            training.len()
        )),
        feature_normalizer.digest(),
        stable_hash_string("constant-representation-normalizer-not-applicable-v3"),
        None,
        stable_hash_string("constant-training-prevalence-policy-v3"),
        stable_hash_string(&format!("fresh-constant-fit-v3:{}", training.len())),
        history,
        split,
        &validation_timestamp_digest,
    )?;
    let constant_receipt = make_receipt_v3(
        &constant_participant,
        split,
        private_metric_digest_v3("ConstantProbabilityBaselineV3", &constant_metric),
        None,
        constant_status,
    )?;
    participants.push(constant_participant);
    receipts.push(constant_receipt);
    participants.sort_by(|left, right| left.participant_id.cmp(&right.participant_id));
    receipts.sort_by(|left, right| left.participant_digest.cmp(&right.participant_digest));
    contribution_audits
        .sort_by(|left, right| left.participant_digest.cmp(&right.participant_digest));
    let qualified_mamba_only_count = receipts
        .iter()
        .filter(|receipt| {
            receipt.status == MomentumRepresentationQualificationStatusV3::QualifiedMambaOnly
        })
        .count();
    let qualified_mamba_hybrid_count = receipts
        .iter()
        .filter(|receipt| {
            receipt.status
                == MomentumRepresentationQualificationStatusV3::QualifiedMambaContributingHybrid
        })
        .count();
    let qualified_raw_fallback_count = receipts
        .iter()
        .filter(|receipt| {
            receipt.status
                == MomentumRepresentationQualificationStatusV3::QualifiedRawFallbackNotMamba
        })
        .count();
    let qualified_comparator_count = receipts
        .iter()
        .filter(|receipt| {
            matches!(
                receipt.status,
                MomentumRepresentationQualificationStatusV3::ComparatorQualified
                    | MomentumRepresentationQualificationStatusV3::BenchmarkQualified
            )
        })
        .count();
    let mut family = MomentumRepresentationFamilyV3 {
        family_version: FAMILY_VERSION_V3.to_string(),
        agent_id: AGENT_ID_V3.to_string(),
        source_snapshot_digest: history.v1.snapshot.content_digest.clone(),
        canonical_view_digest: history.v1.input.input.view.view_digest.clone(),
        representation_audit_digest: audit.audit_digest.clone(),
        split_digest: split.split_digest.clone(),
        registration_digest: registration.registration_digest.clone(),
        participants,
        qualification_receipts: receipts,
        contribution_audits,
        qualified_mamba_only_count,
        qualified_mamba_hybrid_count,
        qualified_raw_fallback_count,
        qualified_comparator_count,
        winner_selected: false,
        historical_test_accessed: false,
        eligible_for_active_committee: false,
        eligible_for_promotion: false,
        eligible_for_reward: false,
        family_digest: String::new(),
    };
    family.family_digest = family_digest_v3(&family);
    validate_family_v3(&family)?;
    let decision = derive_route_decision_v3(history, &family)?;
    let (roster, roster_status) = derive_roster_v3(&family, &decision)?;
    let (evaluation_registration, evaluation_registration_status) = if let Some(roster) = &roster {
        (
            Some(derive_evaluation_registration_v3(
                history,
                split,
                &family,
                &decision,
                roster,
                reservation,
            )?),
            MomentumRepresentationEvaluationStatusV3::Registered,
        )
    } else {
        let status = match roster_status {
            MomentumRepresentationRosterStatusV3::FrozenMambaRepresentationPathRejected => {
                MomentumRepresentationEvaluationStatusV3::FrozenMambaRepresentationPathRejected
            }
            MomentumRepresentationRosterStatusV3::InsufficientComparators => {
                MomentumRepresentationEvaluationStatusV3::InsufficientComparators
            }
            MomentumRepresentationRosterStatusV3::Registered => {
                MomentumRepresentationEvaluationStatusV3::SafetyContractInvalid
            }
        };
        (None, status)
    };
    Ok(RepresentationExperimentV3 {
        family,
        decision,
        roster,
        roster_status,
        evaluation_registration,
        evaluation_registration_status,
    })
}

#[derive(Clone, PartialEq, Message)]
struct ProbeProtobufV3 {
    #[prost(string, tag = "1")]
    probe_kind: String,
    #[prost(string, tag = "2")]
    source_snapshot_digest: String,
    #[prost(string, tag = "3")]
    consumed_range_digest: String,
    #[prost(string, tag = "4")]
    feature_policy_digest: String,
    #[prost(string, optional, tag = "5")]
    encoder_digest: Option<String>,
    #[prost(string, tag = "6")]
    representation_diagnostic_digest: String,
    #[prost(string, tag = "7")]
    private_probe_metric_digest: String,
    #[prost(string, tag = "8")]
    status: String,
    #[prost(string, tag = "9")]
    probe_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct AuditProtobufV3 {
    #[prost(string, tag = "1")]
    audit_version: String,
    #[prost(string, tag = "2")]
    v1_family_digest: String,
    #[prost(string, tag = "3")]
    v2_family_digest: String,
    #[prost(string, tag = "4")]
    v2_collapse_audit_digest: String,
    #[prost(bytes = "vec", repeated, tag = "5")]
    probes: Vec<Vec<u8>>,
    #[prost(bool, tag = "6")]
    head_only_repair_exhausted: bool,
    #[prost(bool, tag = "7")]
    fresh_v3_validation_accessed: bool,
    #[prost(string, tag = "8")]
    audit_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct RangeProtobufV3 {
    #[prost(uint64, tag = "1")]
    start: u64,
    #[prost(uint64, tag = "2")]
    end: u64,
}

#[derive(Clone, PartialEq, Message)]
struct SplitProtobufV3 {
    #[prost(string, tag = "1")]
    split_version: String,
    #[prost(string, tag = "2")]
    source_snapshot_digest: String,
    #[prost(string, tag = "3")]
    v1_usage_ledger_digest: String,
    #[prost(string, tag = "4")]
    v2_split_digest: String,
    #[prost(message, optional, tag = "5")]
    training_range: Option<RangeProtobufV3>,
    #[prost(message, optional, tag = "6")]
    purge_range: Option<RangeProtobufV3>,
    #[prost(message, optional, tag = "7")]
    fresh_validation_range: Option<RangeProtobufV3>,
    #[prost(message, optional, tag = "8")]
    final_reserved_range: Option<RangeProtobufV3>,
    #[prost(uint64, tag = "9")]
    minimum_validation_samples: u64,
    #[prost(uint64, tag = "10")]
    minimum_final_reserved_samples: u64,
    #[prost(uint64, tag = "11")]
    prior_validation_overlap_count: u64,
    #[prost(uint64, tag = "12")]
    prospective_overlap_count: u64,
    #[prost(uint64, tag = "13")]
    future_evaluation_overlap_count: u64,
    #[prost(string, tag = "14")]
    split_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct VariantProtobufV3 {
    #[prost(string, tag = "1")]
    variant_id: String,
    #[prost(string, tag = "2")]
    input_kind: String,
    #[prost(string, tag = "3")]
    pooling_policy: String,
    #[prost(bool, tag = "4")]
    raw_feature_residual_enabled: bool,
    #[prost(string, tag = "5")]
    head_kind: String,
    #[prost(uint32, tag = "6")]
    learning_rate_bits: u32,
    #[prost(uint32, tag = "7")]
    l2_regularization_bits: u32,
    #[prost(uint64, tag = "8")]
    maximum_epochs: u64,
    #[prost(uint64, tag = "9")]
    initialization_seed: u64,
    #[prost(bool, tag = "10")]
    encoder_frozen: bool,
    #[prost(string, tag = "11")]
    feature_policy_digest: String,
    #[prost(string, tag = "12")]
    label_policy_digest: String,
    #[prost(string, tag = "13")]
    training_policy_digest: String,
    #[prost(string, tag = "14")]
    variant_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct RegistrationProtobufV3 {
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
    representation_audit_digest: String,
    #[prost(string, tag = "7")]
    split_digest: String,
    #[prost(bytes = "vec", repeated, tag = "8")]
    variants: Vec<Vec<u8>>,
    #[prost(uint64, tag = "9")]
    maximum_variants: u64,
    #[prost(string, tag = "10")]
    contribution_policy_digest: String,
    #[prost(bool, tag = "11")]
    fresh_validation_hidden: bool,
    #[prost(bool, tag = "12")]
    historical_test_forbidden: bool,
    #[prost(bool, tag = "13")]
    future_evaluation_forbidden: bool,
    #[prost(bool, tag = "14")]
    winner_selection_forbidden: bool,
    #[prost(bool, tag = "15")]
    active_promotion_forbidden: bool,
    #[prost(bool, tag = "16")]
    reward_application_forbidden: bool,
    #[prost(string, tag = "17")]
    registration_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct ParticipantProtobufV3 {
    #[prost(string, tag = "1")]
    participant_version: String,
    #[prost(string, tag = "2")]
    participant_id: String,
    #[prost(string, tag = "3")]
    participant_role: String,
    #[prost(string, tag = "4")]
    model_kind: String,
    #[prost(string, tag = "5")]
    input_kind: String,
    #[prost(string, optional, tag = "6")]
    variant_digest: Option<String>,
    #[prost(string, tag = "7")]
    source_snapshot_digest: String,
    #[prost(string, tag = "8")]
    training_range_digest: String,
    #[prost(string, tag = "9")]
    fresh_validation_range_digest: String,
    #[prost(string, tag = "10")]
    validation_timestamp_digest: String,
    #[prost(string, tag = "11")]
    model_artifact_digest: String,
    #[prost(string, tag = "12")]
    parameter_digest: String,
    #[prost(string, tag = "13")]
    feature_normalizer_digest: String,
    #[prost(string, tag = "14")]
    representation_normalizer_digest: String,
    #[prost(string, optional, tag = "15")]
    encoder_digest: Option<String>,
    #[prost(string, tag = "16")]
    feature_policy_digest: String,
    #[prost(string, tag = "17")]
    label_policy_digest: String,
    #[prost(string, tag = "18")]
    training_policy_digest: String,
    #[prost(string, tag = "19")]
    initialization_digest: String,
    #[prost(bool, tag = "20")]
    warm_start: bool,
    #[prost(bool, tag = "21")]
    v1_head_reused: bool,
    #[prost(bool, tag = "22")]
    v2_head_reused: bool,
    #[prost(bool, tag = "23")]
    fresh_deterministic_initialization: bool,
    #[prost(bool, tag = "24")]
    encoder_frozen: bool,
    #[prost(string, tag = "25")]
    deployment_status: String,
    #[prost(string, tag = "26")]
    participant_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct ReceiptProtobufV3 {
    #[prost(string, tag = "1")]
    receipt_version: String,
    #[prost(string, tag = "2")]
    participant_id: String,
    #[prost(string, tag = "3")]
    participant_digest: String,
    #[prost(string, tag = "4")]
    input_kind: String,
    #[prost(string, tag = "5")]
    fresh_validation_range_digest: String,
    #[prost(string, tag = "6")]
    qualification_policy_digest: String,
    #[prost(string, tag = "7")]
    private_metric_digest: String,
    #[prost(string, optional, tag = "8")]
    contribution_audit_digest: Option<String>,
    #[prost(string, tag = "9")]
    status: String,
    #[prost(uint64, tag = "10")]
    validation_parameter_updates: u64,
    #[prost(uint64, tag = "11")]
    historical_test_reads: u64,
    #[prost(uint64, tag = "12")]
    future_evaluation_reads: u64,
    #[prost(string, tag = "13")]
    receipt_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct ContributionProtobufV3 {
    #[prost(string, tag = "1")]
    participant_digest: String,
    #[prost(string, tag = "2")]
    mamba_parameter_block_digest: String,
    #[prost(string, tag = "3")]
    raw_parameter_block_digest: String,
    #[prost(bool, tag = "4")]
    mamba_block_nonzero: bool,
    #[prost(bool, tag = "5")]
    raw_block_nonzero: bool,
    #[prost(string, tag = "6")]
    full_prediction_digest: String,
    #[prost(string, tag = "7")]
    mamba_ablated_prediction_digest: String,
    #[prost(string, tag = "8")]
    raw_ablated_prediction_digest: String,
    #[prost(string, tag = "9")]
    mamba_ablation_effect_status: String,
    #[prost(string, tag = "10")]
    raw_ablation_effect_status: String,
    #[prost(string, tag = "11")]
    contribution_policy_digest: String,
    #[prost(string, tag = "12")]
    contribution_status: String,
    #[prost(string, tag = "13")]
    audit_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct FamilyProtobufV3 {
    #[prost(string, tag = "1")]
    family_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    source_snapshot_digest: String,
    #[prost(string, tag = "4")]
    canonical_view_digest: String,
    #[prost(string, tag = "5")]
    representation_audit_digest: String,
    #[prost(string, tag = "6")]
    split_digest: String,
    #[prost(string, tag = "7")]
    registration_digest: String,
    #[prost(bytes = "vec", repeated, tag = "8")]
    participants: Vec<Vec<u8>>,
    #[prost(bytes = "vec", repeated, tag = "9")]
    qualification_receipts: Vec<Vec<u8>>,
    #[prost(bytes = "vec", repeated, tag = "10")]
    contribution_audits: Vec<Vec<u8>>,
    #[prost(uint64, tag = "11")]
    qualified_mamba_only_count: u64,
    #[prost(uint64, tag = "12")]
    qualified_mamba_hybrid_count: u64,
    #[prost(uint64, tag = "13")]
    qualified_raw_fallback_count: u64,
    #[prost(uint64, tag = "14")]
    qualified_comparator_count: u64,
    #[prost(bool, tag = "15")]
    winner_selected: bool,
    #[prost(bool, tag = "16")]
    historical_test_accessed: bool,
    #[prost(bool, tag = "17")]
    eligible_for_active_committee: bool,
    #[prost(bool, tag = "18")]
    eligible_for_promotion: bool,
    #[prost(bool, tag = "19")]
    eligible_for_reward: bool,
    #[prost(string, tag = "20")]
    family_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct DecisionProtobufV3 {
    #[prost(string, tag = "1")]
    decision_version: String,
    #[prost(string, tag = "2")]
    v1_family_digest: String,
    #[prost(string, tag = "3")]
    v2_family_digest: String,
    #[prost(string, tag = "4")]
    v3_family_digest: String,
    #[prost(string, repeated, tag = "5")]
    qualified_mamba_only_digests: Vec<String>,
    #[prost(string, repeated, tag = "6")]
    qualified_mamba_hybrid_digests: Vec<String>,
    #[prost(string, repeated, tag = "7")]
    raw_fallback_digests: Vec<String>,
    #[prost(string, repeated, tag = "8")]
    rejected_route_digests: Vec<String>,
    #[prost(bool, tag = "9")]
    further_head_only_repair_forbidden: bool,
    #[prost(bool, tag = "10")]
    further_frozen_representation_sweep_forbidden: bool,
    #[prost(string, tag = "11")]
    decision: String,
    #[prost(string, tag = "12")]
    decision_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct RosterProtobufV3 {
    #[prost(string, tag = "1")]
    roster_version: String,
    #[prost(string, tag = "2")]
    family_digest: String,
    #[prost(string, tag = "3")]
    decision_digest: String,
    #[prost(string, repeated, tag = "4")]
    qualified_genuine_mamba_digests: Vec<String>,
    #[prost(string, repeated, tag = "5")]
    qualified_comparator_digests: Vec<String>,
    #[prost(string, repeated, tag = "6")]
    excluded_participant_digests: Vec<String>,
    #[prost(string, tag = "7")]
    inclusion_policy_digest: String,
    #[prost(string, tag = "8")]
    roster_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct EvaluationProtobufV3 {
    #[prost(string, tag = "1")]
    registration_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    family_digest: String,
    #[prost(string, tag = "4")]
    roster_digest: String,
    #[prost(string, tag = "5")]
    decision_digest: String,
    #[prost(string, repeated, tag = "6")]
    qualification_receipt_digests: Vec<String>,
    #[prost(string, repeated, tag = "7")]
    contribution_audit_digests: Vec<String>,
    #[prost(string, tag = "8")]
    source_snapshot_digest: String,
    #[prost(uint64, tag = "9")]
    source_boundary_timestamp_ms: u64,
    #[prost(string, repeated, tag = "10")]
    protected_registration_digests: Vec<String>,
    #[prost(uint64, repeated, tag = "11")]
    protected_timestamp_ms: Vec<u64>,
    #[prost(string, repeated, tag = "12")]
    prior_validation_and_reserved_range_digests: Vec<String>,
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
    registration_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct JournalProtobufV3 {
    #[prost(string, tag = "1")]
    journal_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    repair_stage: String,
    #[prost(string, tag = "4")]
    representation_audit_digest: String,
    #[prost(string, tag = "5")]
    split_digest: String,
    #[prost(string, tag = "6")]
    registration_digest: String,
    #[prost(string, optional, tag = "7")]
    family_digest: Option<String>,
    #[prost(string, optional, tag = "8")]
    decision_digest: Option<String>,
    #[prost(string, optional, tag = "9")]
    roster_digest: Option<String>,
    #[prost(string, optional, tag = "10")]
    evaluation_registration_digest: Option<String>,
    #[prost(bool, tag = "11")]
    prior_validation_used_for_v3_qualification: bool,
    #[prost(bool, tag = "12")]
    final_reserve_accessed: bool,
    #[prost(bool, tag = "13")]
    warm_start: bool,
    #[prost(bool, tag = "14")]
    v1_head_reused: bool,
    #[prost(bool, tag = "15")]
    v2_head_reused: bool,
    #[prost(bool, tag = "16")]
    fresh_deterministic_initialization: bool,
    #[prost(string, tag = "17")]
    status: String,
    #[prost(string, tag = "18")]
    journal_digest: String,
}

fn usize_from_u64_v3(value: u64) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| "V3 Protobuf integer overflow".to_string())
}

fn range_to_protobuf_v3(value: &IndexRangeV0) -> Result<RangeProtobufV3, String> {
    Ok(RangeProtobufV3 {
        start: u64::try_from(value.start).map_err(|_| "V3 range overflow".to_string())?,
        end: u64::try_from(value.end).map_err(|_| "V3 range overflow".to_string())?,
    })
}

fn range_from_protobuf_v3(value: Option<RangeProtobufV3>) -> Result<IndexRangeV0, String> {
    let value = value.ok_or_else(|| "V3 Protobuf range missing".to_string())?;
    Ok(IndexRangeV0 {
        start: usize_from_u64_v3(value.start)?,
        end: usize_from_u64_v3(value.end)?,
    })
}

fn parse_probe_kind_v3(value: &str) -> Result<MomentumRepresentationProbeKindV3, String> {
    match value {
        "RawFeatureLinearProbe" => Ok(MomentumRepresentationProbeKindV3::RawFeatureLinearProbe),
        "MambaLastOutputProbe" => Ok(MomentumRepresentationProbeKindV3::MambaLastOutputProbe),
        "MambaMeanOutputProbe" => Ok(MomentumRepresentationProbeKindV3::MambaMeanOutputProbe),
        "MambaLastMeanConcatProbe" => {
            Ok(MomentumRepresentationProbeKindV3::MambaLastMeanConcatProbe)
        }
        _ => Err("V3 Protobuf probe kind rejected".to_string()),
    }
}

fn parse_probe_status_v3(value: &str) -> Result<MomentumRepresentationProbeStatusV3, String> {
    match value {
        "FiniteUsable" => Ok(MomentumRepresentationProbeStatusV3::FiniteUsable),
        "LowVariance" => Ok(MomentumRepresentationProbeStatusV3::LowVariance),
        "LowEffectiveRank" => Ok(MomentumRepresentationProbeStatusV3::LowEffectiveRank),
        "SingleSidedPrediction" => Ok(MomentumRepresentationProbeStatusV3::SingleSidedPrediction),
        "NonCollapsedPrediction" => Ok(MomentumRepresentationProbeStatusV3::NonCollapsedPrediction),
        "NumericalFailure" => Ok(MomentumRepresentationProbeStatusV3::NumericalFailure),
        "InsufficientEvidence" => Ok(MomentumRepresentationProbeStatusV3::InsufficientEvidence),
        _ => Err("V3 Protobuf probe status rejected".to_string()),
    }
}

fn parse_input_kind_v3(value: &str) -> Result<MomentumRepresentationInputKindV3, String> {
    match value {
        "MambaLastOutput" => Ok(MomentumRepresentationInputKindV3::MambaLastOutput),
        "MambaMeanOutput" => Ok(MomentumRepresentationInputKindV3::MambaMeanOutput),
        "MambaLastMeanConcat" => Ok(MomentumRepresentationInputKindV3::MambaLastMeanConcat),
        "MambaRawFeatureResidual" => Ok(MomentumRepresentationInputKindV3::MambaRawFeatureResidual),
        _ => Err("V3 Protobuf input kind rejected".to_string()),
    }
}

fn parse_participant_role_v3(
    value: &str,
) -> Result<MomentumRepresentationParticipantRoleV3, String> {
    match value {
        "MambaOnly" => Ok(MomentumRepresentationParticipantRoleV3::MambaOnly),
        "MambaResidualHybrid" => Ok(MomentumRepresentationParticipantRoleV3::MambaResidualHybrid),
        "LinearComparator" => Ok(MomentumRepresentationParticipantRoleV3::LinearComparator),
        "ConstantBenchmark" => Ok(MomentumRepresentationParticipantRoleV3::ConstantBenchmark),
        _ => Err("V3 Protobuf participant role rejected".to_string()),
    }
}

fn parse_qualification_status_v3(
    value: &str,
) -> Result<MomentumRepresentationQualificationStatusV3, String> {
    match value {
        "QualifiedMambaOnly" => Ok(MomentumRepresentationQualificationStatusV3::QualifiedMambaOnly),
        "QualifiedMambaContributingHybrid" => {
            Ok(MomentumRepresentationQualificationStatusV3::QualifiedMambaContributingHybrid)
        }
        "QualifiedRawFallbackNotMamba" => {
            Ok(MomentumRepresentationQualificationStatusV3::QualifiedRawFallbackNotMamba)
        }
        "ComparatorQualified" => {
            Ok(MomentumRepresentationQualificationStatusV3::ComparatorQualified)
        }
        "BenchmarkQualified" => Ok(MomentumRepresentationQualificationStatusV3::BenchmarkQualified),
        "RejectedInsufficientValidation" => {
            Ok(MomentumRepresentationQualificationStatusV3::RejectedInsufficientValidation)
        }
        "RejectedRepresentationInvariant" => {
            Ok(MomentumRepresentationQualificationStatusV3::RejectedRepresentationInvariant)
        }
        "RejectedProbabilityCollapse" => {
            Ok(MomentumRepresentationQualificationStatusV3::RejectedProbabilityCollapse)
        }
        "RejectedNumericalFailure" => {
            Ok(MomentumRepresentationQualificationStatusV3::RejectedNumericalFailure)
        }
        "RejectedContributionInvariant" => {
            Ok(MomentumRepresentationQualificationStatusV3::RejectedContributionInvariant)
        }
        _ => Err("V3 Protobuf qualification status rejected".to_string()),
    }
}

fn parse_contribution_status_v3(value: &str) -> Result<MambaContributionStatusV3, String> {
    match value {
        "NotApplicable" => Ok(MambaContributionStatusV3::NotApplicable),
        "MaterialContribution" => Ok(MambaContributionStatusV3::MaterialContribution),
        "DetectableButBelowPolicy" => Ok(MambaContributionStatusV3::DetectableButBelowPolicy),
        "NoDetectableContribution" => Ok(MambaContributionStatusV3::NoDetectableContribution),
        "RawFeatureDominated" => Ok(MambaContributionStatusV3::RawFeatureDominated),
        "Invalid" => Ok(MambaContributionStatusV3::Invalid),
        _ => Err("V3 Protobuf contribution status rejected".to_string()),
    }
}

fn parse_decision_v3(value: &str) -> Result<MomentumRepresentationRouteDecisionV3, String> {
    match value {
        "FrozenMambaOnlyViable" => Ok(MomentumRepresentationRouteDecisionV3::FrozenMambaOnlyViable),
        "MambaResidualHybridViable" => {
            Ok(MomentumRepresentationRouteDecisionV3::MambaResidualHybridViable)
        }
        "RawFeatureFallbackOnly" => {
            Ok(MomentumRepresentationRouteDecisionV3::RawFeatureFallbackOnly)
        }
        "FrozenMambaAddsNoIncrementalSignal" => {
            Ok(MomentumRepresentationRouteDecisionV3::FrozenMambaAddsNoIncrementalSignal)
        }
        "AllRepresentationRoutesCollapsed" => {
            Ok(MomentumRepresentationRouteDecisionV3::AllRepresentationRoutesCollapsed)
        }
        "InsufficientFreshValidation" => {
            Ok(MomentumRepresentationRouteDecisionV3::InsufficientFreshValidation)
        }
        "TechnicalFailure" => Ok(MomentumRepresentationRouteDecisionV3::TechnicalFailure),
        _ => Err("V3 Protobuf route decision rejected".to_string()),
    }
}

fn parse_stage_v3(value: &str) -> Result<MomentumFrozenMambaRepairStageV3, String> {
    match value {
        "V1OriginalCollapsed" => Ok(MomentumFrozenMambaRepairStageV3::V1OriginalCollapsed),
        "V2HeadOnlyRepairExhausted" => {
            Ok(MomentumFrozenMambaRepairStageV3::V2HeadOnlyRepairExhausted)
        }
        "V3RepresentationPathPending" => {
            Ok(MomentumFrozenMambaRepairStageV3::V3RepresentationPathPending)
        }
        "V3RepresentationPathViable" => {
            Ok(MomentumFrozenMambaRepairStageV3::V3RepresentationPathViable)
        }
        "V3ResidualHybridViable" => Ok(MomentumFrozenMambaRepairStageV3::V3ResidualHybridViable),
        "V3MambaContributionAbsent" => {
            Ok(MomentumFrozenMambaRepairStageV3::V3MambaContributionAbsent)
        }
        "V3FrozenMambaPathRejected" => {
            Ok(MomentumFrozenMambaRepairStageV3::V3FrozenMambaPathRejected)
        }
        _ => Err("V3 Protobuf repair stage rejected".to_string()),
    }
}

fn parse_execution_status_v3(
    value: &str,
) -> Result<MomentumRepresentationExecutionStatusV3, String> {
    match value {
        "Planned" => Ok(MomentumRepresentationExecutionStatusV3::Planned),
        "Executed" => Ok(MomentumRepresentationExecutionStatusV3::Executed),
        "AlreadyExecuted" => Ok(MomentumRepresentationExecutionStatusV3::AlreadyExecuted),
        "InsufficientFreshValidation" => {
            Ok(MomentumRepresentationExecutionStatusV3::InsufficientFreshValidation)
        }
        "TechnicalFailure" => Ok(MomentumRepresentationExecutionStatusV3::TechnicalFailure),
        _ => Err("V3 Protobuf execution status rejected".to_string()),
    }
}

fn probe_to_protobuf_v3(value: &MomentumRepresentationProbeV3) -> ProbeProtobufV3 {
    ProbeProtobufV3 {
        probe_kind: format!("{:?}", value.probe_kind),
        source_snapshot_digest: value.source_snapshot_digest.clone(),
        consumed_range_digest: value.consumed_range_digest.clone(),
        feature_policy_digest: value.feature_policy_digest.clone(),
        encoder_digest: value.encoder_digest.clone(),
        representation_diagnostic_digest: value.representation_diagnostic_digest.clone(),
        private_probe_metric_digest: value.private_probe_metric_digest.clone(),
        status: format!("{:?}", value.status),
        probe_digest: value.probe_digest.clone(),
    }
}

fn probe_from_protobuf_v3(value: ProbeProtobufV3) -> Result<MomentumRepresentationProbeV3, String> {
    let probe = MomentumRepresentationProbeV3 {
        probe_kind: parse_probe_kind_v3(&value.probe_kind)?,
        source_snapshot_digest: value.source_snapshot_digest,
        consumed_range_digest: value.consumed_range_digest,
        feature_policy_digest: value.feature_policy_digest,
        encoder_digest: value.encoder_digest,
        representation_diagnostic_digest: value.representation_diagnostic_digest,
        private_probe_metric_digest: value.private_probe_metric_digest,
        status: parse_probe_status_v3(&value.status)?,
        probe_digest: value.probe_digest,
    };
    validate_probe_v3(&probe)?;
    Ok(probe)
}

pub fn encode_momentum_representation_probe_protobuf_v3(
    value: &MomentumRepresentationProbeV3,
) -> Result<Vec<u8>, String> {
    validate_probe_v3(value)?;
    Ok(probe_to_protobuf_v3(value).encode_to_vec())
}

pub fn decode_momentum_representation_probe_protobuf_v3(
    bytes: &[u8],
) -> Result<MomentumRepresentationProbeV3, String> {
    probe_from_protobuf_v3(
        ProbeProtobufV3::decode(bytes).map_err(|_| "V3 probe Protobuf rejected".to_string())?,
    )
}

pub fn encode_momentum_representation_audit_protobuf_v3(
    value: &MomentumRepresentationPathAuditV3,
) -> Result<Vec<u8>, String> {
    validate_audit_v3(value)?;
    Ok(AuditProtobufV3 {
        audit_version: value.audit_version.clone(),
        v1_family_digest: value.v1_family_digest.clone(),
        v2_family_digest: value.v2_family_digest.clone(),
        v2_collapse_audit_digest: value.v2_collapse_audit_digest.clone(),
        probes: value
            .probes
            .iter()
            .map(encode_momentum_representation_probe_protobuf_v3)
            .collect::<Result<Vec<_>, _>>()?,
        head_only_repair_exhausted: value.head_only_repair_exhausted,
        fresh_v3_validation_accessed: value.fresh_v3_validation_accessed,
        audit_digest: value.audit_digest.clone(),
    }
    .encode_to_vec())
}

pub fn decode_momentum_representation_audit_protobuf_v3(
    bytes: &[u8],
) -> Result<MomentumRepresentationPathAuditV3, String> {
    let value =
        AuditProtobufV3::decode(bytes).map_err(|_| "V3 audit Protobuf rejected".to_string())?;
    let audit = MomentumRepresentationPathAuditV3 {
        audit_version: value.audit_version,
        v1_family_digest: value.v1_family_digest,
        v2_family_digest: value.v2_family_digest,
        v2_collapse_audit_digest: value.v2_collapse_audit_digest,
        probes: value
            .probes
            .iter()
            .map(|bytes| decode_momentum_representation_probe_protobuf_v3(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        head_only_repair_exhausted: value.head_only_repair_exhausted,
        fresh_v3_validation_accessed: value.fresh_v3_validation_accessed,
        audit_digest: value.audit_digest,
    };
    validate_audit_v3(&audit)?;
    Ok(audit)
}

pub fn encode_momentum_representation_split_protobuf_v3(
    value: &MomentumRepresentationSplitV3,
) -> Result<Vec<u8>, String> {
    validate_split_v3(value)?;
    Ok(SplitProtobufV3 {
        split_version: value.split_version.clone(),
        source_snapshot_digest: value.source_snapshot_digest.clone(),
        v1_usage_ledger_digest: value.v1_usage_ledger_digest.clone(),
        v2_split_digest: value.v2_split_digest.clone(),
        training_range: Some(range_to_protobuf_v3(&value.training_range)?),
        purge_range: Some(range_to_protobuf_v3(&value.purge_range)?),
        fresh_validation_range: Some(range_to_protobuf_v3(&value.fresh_validation_range)?),
        final_reserved_range: Some(range_to_protobuf_v3(&value.final_reserved_range)?),
        minimum_validation_samples: u64::try_from(value.minimum_validation_samples)
            .map_err(|_| "V3 split integer overflow".to_string())?,
        minimum_final_reserved_samples: u64::try_from(value.minimum_final_reserved_samples)
            .map_err(|_| "V3 split integer overflow".to_string())?,
        prior_validation_overlap_count: u64::try_from(value.prior_validation_overlap_count)
            .map_err(|_| "V3 split integer overflow".to_string())?,
        prospective_overlap_count: u64::try_from(value.prospective_overlap_count)
            .map_err(|_| "V3 split integer overflow".to_string())?,
        future_evaluation_overlap_count: u64::try_from(value.future_evaluation_overlap_count)
            .map_err(|_| "V3 split integer overflow".to_string())?,
        split_digest: value.split_digest.clone(),
    }
    .encode_to_vec())
}

pub fn decode_momentum_representation_split_protobuf_v3(
    bytes: &[u8],
) -> Result<MomentumRepresentationSplitV3, String> {
    let value =
        SplitProtobufV3::decode(bytes).map_err(|_| "V3 split Protobuf rejected".to_string())?;
    let split = MomentumRepresentationSplitV3 {
        split_version: value.split_version,
        source_snapshot_digest: value.source_snapshot_digest,
        v1_usage_ledger_digest: value.v1_usage_ledger_digest,
        v2_split_digest: value.v2_split_digest,
        training_range: range_from_protobuf_v3(value.training_range)?,
        purge_range: range_from_protobuf_v3(value.purge_range)?,
        fresh_validation_range: range_from_protobuf_v3(value.fresh_validation_range)?,
        final_reserved_range: range_from_protobuf_v3(value.final_reserved_range)?,
        minimum_validation_samples: usize_from_u64_v3(value.minimum_validation_samples)?,
        minimum_final_reserved_samples: usize_from_u64_v3(value.minimum_final_reserved_samples)?,
        prior_validation_overlap_count: usize_from_u64_v3(value.prior_validation_overlap_count)?,
        prospective_overlap_count: usize_from_u64_v3(value.prospective_overlap_count)?,
        future_evaluation_overlap_count: usize_from_u64_v3(value.future_evaluation_overlap_count)?,
        split_digest: value.split_digest,
    };
    validate_split_v3(&split)?;
    Ok(split)
}

fn variant_to_protobuf_v3(value: &MomentumRepresentationVariantConfigV3) -> VariantProtobufV3 {
    VariantProtobufV3 {
        variant_id: value.variant_id.clone(),
        input_kind: format!("{:?}", value.input_kind),
        pooling_policy: value.pooling_policy.clone(),
        raw_feature_residual_enabled: value.raw_feature_residual_enabled,
        head_kind: value.head_kind.clone(),
        learning_rate_bits: value.learning_rate_bits,
        l2_regularization_bits: value.l2_regularization_bits,
        maximum_epochs: value.maximum_epochs as u64,
        initialization_seed: value.initialization_seed,
        encoder_frozen: value.encoder_frozen,
        feature_policy_digest: value.feature_policy_digest.clone(),
        label_policy_digest: value.label_policy_digest.clone(),
        training_policy_digest: value.training_policy_digest.clone(),
        variant_digest: value.variant_digest.clone(),
    }
}

fn variant_from_protobuf_v3(
    value: VariantProtobufV3,
) -> Result<MomentumRepresentationVariantConfigV3, String> {
    let variant = MomentumRepresentationVariantConfigV3 {
        variant_id: value.variant_id,
        input_kind: parse_input_kind_v3(&value.input_kind)?,
        pooling_policy: value.pooling_policy,
        raw_feature_residual_enabled: value.raw_feature_residual_enabled,
        head_kind: value.head_kind,
        learning_rate_bits: value.learning_rate_bits,
        l2_regularization_bits: value.l2_regularization_bits,
        maximum_epochs: usize_from_u64_v3(value.maximum_epochs)?,
        initialization_seed: value.initialization_seed,
        encoder_frozen: value.encoder_frozen,
        feature_policy_digest: value.feature_policy_digest,
        label_policy_digest: value.label_policy_digest,
        training_policy_digest: value.training_policy_digest,
        variant_digest: value.variant_digest,
    };
    validate_variant_v3(&variant)?;
    Ok(variant)
}

pub fn encode_momentum_representation_registration_protobuf_v3(
    value: &MomentumRepresentationRegistrationV3,
) -> Result<Vec<u8>, String> {
    validate_registration_v3(value)?;
    Ok(RegistrationProtobufV3 {
        registration_version: value.registration_version.clone(),
        agent_id: value.agent_id.clone(),
        source_snapshot_digest: value.source_snapshot_digest.clone(),
        canonical_intent_digest: value.canonical_intent_digest.clone(),
        canonical_view_digest: value.canonical_view_digest.clone(),
        representation_audit_digest: value.representation_audit_digest.clone(),
        split_digest: value.split_digest.clone(),
        variants: value
            .variants
            .iter()
            .map(|variant| variant_to_protobuf_v3(variant).encode_to_vec())
            .collect(),
        maximum_variants: u64::try_from(value.maximum_variants)
            .map_err(|_| "V3 registration integer overflow".to_string())?,
        contribution_policy_digest: value.contribution_policy_digest.clone(),
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

pub fn decode_momentum_representation_registration_protobuf_v3(
    bytes: &[u8],
) -> Result<MomentumRepresentationRegistrationV3, String> {
    let value = RegistrationProtobufV3::decode(bytes)
        .map_err(|_| "V3 registration Protobuf rejected".to_string())?;
    let registration = MomentumRepresentationRegistrationV3 {
        registration_version: value.registration_version,
        agent_id: value.agent_id,
        source_snapshot_digest: value.source_snapshot_digest,
        canonical_intent_digest: value.canonical_intent_digest,
        canonical_view_digest: value.canonical_view_digest,
        representation_audit_digest: value.representation_audit_digest,
        split_digest: value.split_digest,
        variants: value
            .variants
            .iter()
            .map(|bytes| {
                VariantProtobufV3::decode(bytes.as_slice())
                    .map_err(|_| "V3 variant Protobuf rejected".to_string())
                    .and_then(variant_from_protobuf_v3)
            })
            .collect::<Result<Vec<_>, _>>()?,
        maximum_variants: usize_from_u64_v3(value.maximum_variants)?,
        contribution_policy_digest: value.contribution_policy_digest,
        fresh_validation_hidden: value.fresh_validation_hidden,
        historical_test_forbidden: value.historical_test_forbidden,
        future_evaluation_forbidden: value.future_evaluation_forbidden,
        winner_selection_forbidden: value.winner_selection_forbidden,
        active_promotion_forbidden: value.active_promotion_forbidden,
        reward_application_forbidden: value.reward_application_forbidden,
        registration_digest: value.registration_digest,
    };
    validate_registration_v3(&registration)?;
    Ok(registration)
}

fn participant_to_protobuf_v3(value: &FrozenCandidateParticipantV3) -> ParticipantProtobufV3 {
    ParticipantProtobufV3 {
        participant_version: value.participant_version.clone(),
        participant_id: value.participant_id.clone(),
        participant_role: format!("{:?}", value.participant_role),
        model_kind: value.model_kind.clone(),
        input_kind: value.input_kind.clone(),
        variant_digest: value.variant_digest.clone(),
        source_snapshot_digest: value.source_snapshot_digest.clone(),
        training_range_digest: value.training_range_digest.clone(),
        fresh_validation_range_digest: value.fresh_validation_range_digest.clone(),
        validation_timestamp_digest: value.validation_timestamp_digest.clone(),
        model_artifact_digest: value.model_artifact_digest.clone(),
        parameter_digest: value.parameter_digest.clone(),
        feature_normalizer_digest: value.feature_normalizer_digest.clone(),
        representation_normalizer_digest: value.representation_normalizer_digest.clone(),
        encoder_digest: value.encoder_digest.clone(),
        feature_policy_digest: value.feature_policy_digest.clone(),
        label_policy_digest: value.label_policy_digest.clone(),
        training_policy_digest: value.training_policy_digest.clone(),
        initialization_digest: value.initialization_digest.clone(),
        warm_start: value.warm_start,
        v1_head_reused: value.v1_head_reused,
        v2_head_reused: value.v2_head_reused,
        fresh_deterministic_initialization: value.fresh_deterministic_initialization,
        encoder_frozen: value.encoder_frozen,
        deployment_status: format!("{:?}", value.deployment_status),
        participant_digest: value.participant_digest.clone(),
    }
}

fn participant_from_protobuf_v3(
    value: ParticipantProtobufV3,
) -> Result<FrozenCandidateParticipantV3, String> {
    if value.deployment_status != "ShadowOnly" {
        return Err("V3 participant deployment status rejected".to_string());
    }
    let participant = FrozenCandidateParticipantV3 {
        participant_version: value.participant_version,
        participant_id: value.participant_id,
        participant_role: parse_participant_role_v3(&value.participant_role)?,
        model_kind: value.model_kind,
        input_kind: value.input_kind,
        variant_digest: value.variant_digest,
        source_snapshot_digest: value.source_snapshot_digest,
        training_range_digest: value.training_range_digest,
        fresh_validation_range_digest: value.fresh_validation_range_digest,
        validation_timestamp_digest: value.validation_timestamp_digest,
        model_artifact_digest: value.model_artifact_digest,
        parameter_digest: value.parameter_digest,
        feature_normalizer_digest: value.feature_normalizer_digest,
        representation_normalizer_digest: value.representation_normalizer_digest,
        encoder_digest: value.encoder_digest,
        feature_policy_digest: value.feature_policy_digest,
        label_policy_digest: value.label_policy_digest,
        training_policy_digest: value.training_policy_digest,
        initialization_digest: value.initialization_digest,
        warm_start: value.warm_start,
        v1_head_reused: value.v1_head_reused,
        v2_head_reused: value.v2_head_reused,
        fresh_deterministic_initialization: value.fresh_deterministic_initialization,
        encoder_frozen: value.encoder_frozen,
        deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
        participant_digest: value.participant_digest,
    };
    validate_participant_v3(&participant)?;
    Ok(participant)
}

pub fn encode_frozen_candidate_participant_protobuf_v3(
    value: &FrozenCandidateParticipantV3,
) -> Result<Vec<u8>, String> {
    validate_participant_v3(value)?;
    Ok(participant_to_protobuf_v3(value).encode_to_vec())
}

pub fn decode_frozen_candidate_participant_protobuf_v3(
    bytes: &[u8],
) -> Result<FrozenCandidateParticipantV3, String> {
    participant_from_protobuf_v3(
        ParticipantProtobufV3::decode(bytes)
            .map_err(|_| "V3 participant Protobuf rejected".to_string())?,
    )
}

fn receipt_to_protobuf_v3(
    value: &MomentumRepresentationQualificationReceiptV3,
) -> Result<ReceiptProtobufV3, String> {
    Ok(ReceiptProtobufV3 {
        receipt_version: value.receipt_version.clone(),
        participant_id: value.participant_id.clone(),
        participant_digest: value.participant_digest.clone(),
        input_kind: value.input_kind.clone(),
        fresh_validation_range_digest: value.fresh_validation_range_digest.clone(),
        qualification_policy_digest: value.qualification_policy_digest.clone(),
        private_metric_digest: value.private_metric_digest.clone(),
        contribution_audit_digest: value.contribution_audit_digest.clone(),
        status: format!("{:?}", value.status),
        validation_parameter_updates: u64::try_from(value.validation_parameter_updates)
            .map_err(|_| "V3 receipt integer overflow".to_string())?,
        historical_test_reads: u64::try_from(value.historical_test_reads)
            .map_err(|_| "V3 receipt integer overflow".to_string())?,
        future_evaluation_reads: u64::try_from(value.future_evaluation_reads)
            .map_err(|_| "V3 receipt integer overflow".to_string())?,
        receipt_digest: value.receipt_digest.clone(),
    })
}

fn receipt_from_protobuf_v3(
    value: ReceiptProtobufV3,
) -> Result<MomentumRepresentationQualificationReceiptV3, String> {
    let receipt = MomentumRepresentationQualificationReceiptV3 {
        receipt_version: value.receipt_version,
        participant_id: value.participant_id,
        participant_digest: value.participant_digest,
        input_kind: value.input_kind,
        fresh_validation_range_digest: value.fresh_validation_range_digest,
        qualification_policy_digest: value.qualification_policy_digest,
        private_metric_digest: value.private_metric_digest,
        contribution_audit_digest: value.contribution_audit_digest,
        status: parse_qualification_status_v3(&value.status)?,
        validation_parameter_updates: usize_from_u64_v3(value.validation_parameter_updates)?,
        historical_test_reads: usize_from_u64_v3(value.historical_test_reads)?,
        future_evaluation_reads: usize_from_u64_v3(value.future_evaluation_reads)?,
        receipt_digest: value.receipt_digest,
    };
    validate_receipt_v3(&receipt)?;
    Ok(receipt)
}

pub fn encode_momentum_representation_qualification_protobuf_v3(
    value: &MomentumRepresentationQualificationReceiptV3,
) -> Result<Vec<u8>, String> {
    validate_receipt_v3(value)?;
    Ok(receipt_to_protobuf_v3(value)?.encode_to_vec())
}

pub fn decode_momentum_representation_qualification_protobuf_v3(
    bytes: &[u8],
) -> Result<MomentumRepresentationQualificationReceiptV3, String> {
    receipt_from_protobuf_v3(
        ReceiptProtobufV3::decode(bytes).map_err(|_| "V3 receipt Protobuf rejected".to_string())?,
    )
}

fn contribution_to_protobuf_v3(value: &MambaContributionAuditV3) -> ContributionProtobufV3 {
    ContributionProtobufV3 {
        participant_digest: value.participant_digest.clone(),
        mamba_parameter_block_digest: value.mamba_parameter_block_digest.clone(),
        raw_parameter_block_digest: value.raw_parameter_block_digest.clone(),
        mamba_block_nonzero: value.mamba_block_nonzero,
        raw_block_nonzero: value.raw_block_nonzero,
        full_prediction_digest: value.full_prediction_digest.clone(),
        mamba_ablated_prediction_digest: value.mamba_ablated_prediction_digest.clone(),
        raw_ablated_prediction_digest: value.raw_ablated_prediction_digest.clone(),
        mamba_ablation_effect_status: value.mamba_ablation_effect_status.clone(),
        raw_ablation_effect_status: value.raw_ablation_effect_status.clone(),
        contribution_policy_digest: value.contribution_policy_digest.clone(),
        contribution_status: format!("{:?}", value.contribution_status),
        audit_digest: value.audit_digest.clone(),
    }
}

fn contribution_from_protobuf_v3(
    value: ContributionProtobufV3,
) -> Result<MambaContributionAuditV3, String> {
    let audit = MambaContributionAuditV3 {
        participant_digest: value.participant_digest,
        mamba_parameter_block_digest: value.mamba_parameter_block_digest,
        raw_parameter_block_digest: value.raw_parameter_block_digest,
        mamba_block_nonzero: value.mamba_block_nonzero,
        raw_block_nonzero: value.raw_block_nonzero,
        full_prediction_digest: value.full_prediction_digest,
        mamba_ablated_prediction_digest: value.mamba_ablated_prediction_digest,
        raw_ablated_prediction_digest: value.raw_ablated_prediction_digest,
        mamba_ablation_effect_status: value.mamba_ablation_effect_status,
        raw_ablation_effect_status: value.raw_ablation_effect_status,
        contribution_policy_digest: value.contribution_policy_digest,
        contribution_status: parse_contribution_status_v3(&value.contribution_status)?,
        audit_digest: value.audit_digest,
    };
    validate_contribution_v3(&audit)?;
    Ok(audit)
}

pub fn encode_mamba_contribution_audit_protobuf_v3(
    value: &MambaContributionAuditV3,
) -> Result<Vec<u8>, String> {
    validate_contribution_v3(value)?;
    Ok(contribution_to_protobuf_v3(value).encode_to_vec())
}

pub fn decode_mamba_contribution_audit_protobuf_v3(
    bytes: &[u8],
) -> Result<MambaContributionAuditV3, String> {
    contribution_from_protobuf_v3(
        ContributionProtobufV3::decode(bytes)
            .map_err(|_| "V3 contribution Protobuf rejected".to_string())?,
    )
}

pub fn encode_momentum_representation_family_protobuf_v3(
    value: &MomentumRepresentationFamilyV3,
) -> Result<Vec<u8>, String> {
    validate_family_v3(value)?;
    Ok(FamilyProtobufV3 {
        family_version: value.family_version.clone(),
        agent_id: value.agent_id.clone(),
        source_snapshot_digest: value.source_snapshot_digest.clone(),
        canonical_view_digest: value.canonical_view_digest.clone(),
        representation_audit_digest: value.representation_audit_digest.clone(),
        split_digest: value.split_digest.clone(),
        registration_digest: value.registration_digest.clone(),
        participants: value
            .participants
            .iter()
            .map(encode_frozen_candidate_participant_protobuf_v3)
            .collect::<Result<Vec<_>, _>>()?,
        qualification_receipts: value
            .qualification_receipts
            .iter()
            .map(encode_momentum_representation_qualification_protobuf_v3)
            .collect::<Result<Vec<_>, _>>()?,
        contribution_audits: value
            .contribution_audits
            .iter()
            .map(encode_mamba_contribution_audit_protobuf_v3)
            .collect::<Result<Vec<_>, _>>()?,
        qualified_mamba_only_count: value.qualified_mamba_only_count as u64,
        qualified_mamba_hybrid_count: value.qualified_mamba_hybrid_count as u64,
        qualified_raw_fallback_count: value.qualified_raw_fallback_count as u64,
        qualified_comparator_count: value.qualified_comparator_count as u64,
        winner_selected: value.winner_selected,
        historical_test_accessed: value.historical_test_accessed,
        eligible_for_active_committee: value.eligible_for_active_committee,
        eligible_for_promotion: value.eligible_for_promotion,
        eligible_for_reward: value.eligible_for_reward,
        family_digest: value.family_digest.clone(),
    }
    .encode_to_vec())
}

pub fn decode_momentum_representation_family_protobuf_v3(
    bytes: &[u8],
) -> Result<MomentumRepresentationFamilyV3, String> {
    let value =
        FamilyProtobufV3::decode(bytes).map_err(|_| "V3 family Protobuf rejected".to_string())?;
    let family = MomentumRepresentationFamilyV3 {
        family_version: value.family_version,
        agent_id: value.agent_id,
        source_snapshot_digest: value.source_snapshot_digest,
        canonical_view_digest: value.canonical_view_digest,
        representation_audit_digest: value.representation_audit_digest,
        split_digest: value.split_digest,
        registration_digest: value.registration_digest,
        participants: value
            .participants
            .iter()
            .map(|bytes| decode_frozen_candidate_participant_protobuf_v3(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        qualification_receipts: value
            .qualification_receipts
            .iter()
            .map(|bytes| decode_momentum_representation_qualification_protobuf_v3(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        contribution_audits: value
            .contribution_audits
            .iter()
            .map(|bytes| decode_mamba_contribution_audit_protobuf_v3(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        qualified_mamba_only_count: usize_from_u64_v3(value.qualified_mamba_only_count)?,
        qualified_mamba_hybrid_count: usize_from_u64_v3(value.qualified_mamba_hybrid_count)?,
        qualified_raw_fallback_count: usize_from_u64_v3(value.qualified_raw_fallback_count)?,
        qualified_comparator_count: usize_from_u64_v3(value.qualified_comparator_count)?,
        winner_selected: value.winner_selected,
        historical_test_accessed: value.historical_test_accessed,
        eligible_for_active_committee: value.eligible_for_active_committee,
        eligible_for_promotion: value.eligible_for_promotion,
        eligible_for_reward: value.eligible_for_reward,
        family_digest: value.family_digest,
    };
    validate_family_v3(&family)?;
    Ok(family)
}

pub fn encode_momentum_representation_decision_protobuf_v3(
    value: &MomentumRepresentationRouteDecisionArtifactV3,
    family: &MomentumRepresentationFamilyV3,
) -> Result<Vec<u8>, String> {
    validate_decision_v3(value, family)?;
    Ok(DecisionProtobufV3 {
        decision_version: value.decision_version.clone(),
        v1_family_digest: value.v1_family_digest.clone(),
        v2_family_digest: value.v2_family_digest.clone(),
        v3_family_digest: value.v3_family_digest.clone(),
        qualified_mamba_only_digests: value.qualified_mamba_only_digests.clone(),
        qualified_mamba_hybrid_digests: value.qualified_mamba_hybrid_digests.clone(),
        raw_fallback_digests: value.raw_fallback_digests.clone(),
        rejected_route_digests: value.rejected_route_digests.clone(),
        further_head_only_repair_forbidden: value.further_head_only_repair_forbidden,
        further_frozen_representation_sweep_forbidden: value
            .further_frozen_representation_sweep_forbidden,
        decision: format!("{:?}", value.decision),
        decision_digest: value.decision_digest.clone(),
    }
    .encode_to_vec())
}

pub fn decode_momentum_representation_decision_protobuf_v3(
    bytes: &[u8],
    family: &MomentumRepresentationFamilyV3,
) -> Result<MomentumRepresentationRouteDecisionArtifactV3, String> {
    let value = DecisionProtobufV3::decode(bytes)
        .map_err(|_| "V3 decision Protobuf rejected".to_string())?;
    let decision = MomentumRepresentationRouteDecisionArtifactV3 {
        decision_version: value.decision_version,
        v1_family_digest: value.v1_family_digest,
        v2_family_digest: value.v2_family_digest,
        v3_family_digest: value.v3_family_digest,
        qualified_mamba_only_digests: value.qualified_mamba_only_digests,
        qualified_mamba_hybrid_digests: value.qualified_mamba_hybrid_digests,
        raw_fallback_digests: value.raw_fallback_digests,
        rejected_route_digests: value.rejected_route_digests,
        further_head_only_repair_forbidden: value.further_head_only_repair_forbidden,
        further_frozen_representation_sweep_forbidden: value
            .further_frozen_representation_sweep_forbidden,
        decision: parse_decision_v3(&value.decision)?,
        decision_digest: value.decision_digest,
    };
    validate_decision_v3(&decision, family)?;
    Ok(decision)
}

pub fn encode_momentum_representation_roster_protobuf_v3(
    value: &MomentumRepresentationFutureRosterV3,
    family: &MomentumRepresentationFamilyV3,
    decision: &MomentumRepresentationRouteDecisionArtifactV3,
) -> Result<Vec<u8>, String> {
    validate_roster_v3(value, family, decision)?;
    Ok(RosterProtobufV3 {
        roster_version: value.roster_version.clone(),
        family_digest: value.family_digest.clone(),
        decision_digest: value.decision_digest.clone(),
        qualified_genuine_mamba_digests: value.qualified_genuine_mamba_digests.clone(),
        qualified_comparator_digests: value.qualified_comparator_digests.clone(),
        excluded_participant_digests: value.excluded_participant_digests.clone(),
        inclusion_policy_digest: value.inclusion_policy_digest.clone(),
        roster_digest: value.roster_digest.clone(),
    }
    .encode_to_vec())
}

pub fn decode_momentum_representation_roster_protobuf_v3(
    bytes: &[u8],
    family: &MomentumRepresentationFamilyV3,
    decision: &MomentumRepresentationRouteDecisionArtifactV3,
) -> Result<MomentumRepresentationFutureRosterV3, String> {
    let value =
        RosterProtobufV3::decode(bytes).map_err(|_| "V3 roster Protobuf rejected".to_string())?;
    let roster = MomentumRepresentationFutureRosterV3 {
        roster_version: value.roster_version,
        family_digest: value.family_digest,
        decision_digest: value.decision_digest,
        qualified_genuine_mamba_digests: value.qualified_genuine_mamba_digests,
        qualified_comparator_digests: value.qualified_comparator_digests,
        excluded_participant_digests: value.excluded_participant_digests,
        inclusion_policy_digest: value.inclusion_policy_digest,
        roster_digest: value.roster_digest,
    };
    validate_roster_v3(&roster, family, decision)?;
    Ok(roster)
}

pub fn encode_momentum_representation_evaluation_protobuf_v3(
    value: &MomentumRepresentationEvaluationRegistrationV3,
    family: &MomentumRepresentationFamilyV3,
    decision: &MomentumRepresentationRouteDecisionArtifactV3,
    roster: &MomentumRepresentationFutureRosterV3,
) -> Result<Vec<u8>, String> {
    validate_evaluation_v3(value, family, decision, roster)?;
    Ok(EvaluationProtobufV3 {
        registration_version: value.registration_version.clone(),
        agent_id: value.agent_id.clone(),
        family_digest: value.family_digest.clone(),
        roster_digest: value.roster_digest.clone(),
        decision_digest: value.decision_digest.clone(),
        qualification_receipt_digests: value.qualification_receipt_digests.clone(),
        contribution_audit_digests: value.contribution_audit_digests.clone(),
        source_snapshot_digest: value.source_snapshot_digest.clone(),
        source_boundary_timestamp_ms: value.source_boundary_timestamp_ms,
        protected_registration_digests: value.protected_registration_digests.clone(),
        protected_timestamp_ms: value.protected_timestamp_ms.clone(),
        prior_validation_and_reserved_range_digests: value
            .prior_validation_and_reserved_range_digests
            .clone(),
        provider_finality_boundary_ms: value.provider_finality_boundary_ms,
        minimum_accepted_timestamp_ms: value.minimum_accepted_timestamp_ms,
        labels_hidden_until_opening: value.labels_hidden_until_opening,
        probabilities_hidden_until_opening: value.probabilities_hidden_until_opening,
        one_time_opening_required: value.one_time_opening_required,
        winner_selection_forbidden_before_opening: value.winner_selection_forbidden_before_opening,
        active_promotion_forbidden: value.active_promotion_forbidden,
        reward_application_forbidden: value.reward_application_forbidden,
        maximum_requests: value.maximum_requests as u64,
        maximum_concurrency: value.maximum_concurrency as u64,
        maximum_retries: value.maximum_retries as u64,
        registration_digest: value.registration_digest.clone(),
    }
    .encode_to_vec())
}

pub fn decode_momentum_representation_evaluation_protobuf_v3(
    bytes: &[u8],
    family: &MomentumRepresentationFamilyV3,
    decision: &MomentumRepresentationRouteDecisionArtifactV3,
    roster: &MomentumRepresentationFutureRosterV3,
) -> Result<MomentumRepresentationEvaluationRegistrationV3, String> {
    let value = EvaluationProtobufV3::decode(bytes)
        .map_err(|_| "V3 evaluation Protobuf rejected".to_string())?;
    let registration = MomentumRepresentationEvaluationRegistrationV3 {
        registration_version: value.registration_version,
        agent_id: value.agent_id,
        family_digest: value.family_digest,
        roster_digest: value.roster_digest,
        decision_digest: value.decision_digest,
        qualification_receipt_digests: value.qualification_receipt_digests,
        contribution_audit_digests: value.contribution_audit_digests,
        source_snapshot_digest: value.source_snapshot_digest,
        source_boundary_timestamp_ms: value.source_boundary_timestamp_ms,
        protected_registration_digests: value.protected_registration_digests,
        protected_timestamp_ms: value.protected_timestamp_ms,
        prior_validation_and_reserved_range_digests: value
            .prior_validation_and_reserved_range_digests,
        provider_finality_boundary_ms: value.provider_finality_boundary_ms,
        minimum_accepted_timestamp_ms: value.minimum_accepted_timestamp_ms,
        labels_hidden_until_opening: value.labels_hidden_until_opening,
        probabilities_hidden_until_opening: value.probabilities_hidden_until_opening,
        one_time_opening_required: value.one_time_opening_required,
        winner_selection_forbidden_before_opening: value.winner_selection_forbidden_before_opening,
        active_promotion_forbidden: value.active_promotion_forbidden,
        reward_application_forbidden: value.reward_application_forbidden,
        maximum_requests: usize_from_u64_v3(value.maximum_requests)?,
        maximum_concurrency: usize_from_u64_v3(value.maximum_concurrency)?,
        maximum_retries: usize_from_u64_v3(value.maximum_retries)?,
        registration_digest: value.registration_digest,
    };
    validate_evaluation_v3(&registration, family, decision, roster)?;
    Ok(registration)
}

pub fn encode_momentum_representation_journal_protobuf_v3(
    value: &MomentumRepresentationJournalV3,
) -> Result<Vec<u8>, String> {
    validate_journal_v3(value)?;
    Ok(JournalProtobufV3 {
        journal_version: value.journal_version.clone(),
        agent_id: value.agent_id.clone(),
        repair_stage: format!("{:?}", value.repair_stage),
        representation_audit_digest: value.representation_audit_digest.clone(),
        split_digest: value.split_digest.clone(),
        registration_digest: value.registration_digest.clone(),
        family_digest: value.family_digest.clone(),
        decision_digest: value.decision_digest.clone(),
        roster_digest: value.roster_digest.clone(),
        evaluation_registration_digest: value.evaluation_registration_digest.clone(),
        prior_validation_used_for_v3_qualification: value
            .prior_validation_used_for_v3_qualification,
        final_reserve_accessed: value.final_reserve_accessed,
        warm_start: value.warm_start,
        v1_head_reused: value.v1_head_reused,
        v2_head_reused: value.v2_head_reused,
        fresh_deterministic_initialization: value.fresh_deterministic_initialization,
        status: format!("{:?}", value.status),
        journal_digest: value.journal_digest.clone(),
    }
    .encode_to_vec())
}

pub fn decode_momentum_representation_journal_protobuf_v3(
    bytes: &[u8],
) -> Result<MomentumRepresentationJournalV3, String> {
    let value =
        JournalProtobufV3::decode(bytes).map_err(|_| "V3 journal Protobuf rejected".to_string())?;
    let journal = MomentumRepresentationJournalV3 {
        journal_version: value.journal_version,
        agent_id: value.agent_id,
        repair_stage: parse_stage_v3(&value.repair_stage)?,
        representation_audit_digest: value.representation_audit_digest,
        split_digest: value.split_digest,
        registration_digest: value.registration_digest,
        family_digest: value.family_digest,
        decision_digest: value.decision_digest,
        roster_digest: value.roster_digest,
        evaluation_registration_digest: value.evaluation_registration_digest,
        prior_validation_used_for_v3_qualification: value
            .prior_validation_used_for_v3_qualification,
        final_reserve_accessed: value.final_reserve_accessed,
        warm_start: value.warm_start,
        v1_head_reused: value.v1_head_reused,
        v2_head_reused: value.v2_head_reused,
        fresh_deterministic_initialization: value.fresh_deterministic_initialization,
        status: parse_execution_status_v3(&value.status)?,
        journal_digest: value.journal_digest,
    };
    validate_journal_v3(&journal)?;
    Ok(journal)
}

fn persist_artifact_v3<F>(
    path: &Path,
    bytes: &[u8],
    digest: &str,
    decode_digest: F,
) -> Result<(usize, usize), String>
where
    F: Fn(&[u8]) -> Result<String, String>,
{
    match atomic_write_verified_v0(path, bytes, digest, decode_digest)? {
        AgentPrivateLearningArtifactWriteStatusV0::Written => Ok((1, 0)),
        AgentPrivateLearningArtifactWriteStatusV0::DuplicateRejected => Ok((0, 1)),
    }
}

fn add_storage_counts(left: &mut (usize, usize), right: (usize, usize)) {
    left.0 += right.0;
    left.1 += right.1;
}

fn persist_preregistration_v3(
    root: &Path,
    audit: &MomentumRepresentationPathAuditV3,
    split: &MomentumRepresentationSplitV3,
    registration: &MomentumRepresentationRegistrationV3,
) -> Result<(usize, usize), String> {
    validate_audit_v3(audit)?;
    validate_split_v3(split)?;
    validate_registration_v3(registration)?;
    let mut counts = (0, 0);
    for probe in &audit.probes {
        let path = root
            .join("representation_probes")
            .join(format!("{}.pb", probe.probe_digest));
        add_storage_counts(
            &mut counts,
            persist_artifact_v3(
                &path,
                &encode_momentum_representation_probe_protobuf_v3(probe)?,
                &probe.probe_digest,
                |bytes| Ok(decode_momentum_representation_probe_protobuf_v3(bytes)?.probe_digest),
            )?,
        );
    }
    add_storage_counts(
        &mut counts,
        persist_artifact_v3(
            &root
                .join("representation_audits")
                .join(format!("{}.pb", audit.audit_digest)),
            &encode_momentum_representation_audit_protobuf_v3(audit)?,
            &audit.audit_digest,
            |bytes| Ok(decode_momentum_representation_audit_protobuf_v3(bytes)?.audit_digest),
        )?,
    );
    add_storage_counts(
        &mut counts,
        persist_artifact_v3(
            &root
                .join("representation_splits")
                .join(format!("{}.pb", split.split_digest)),
            &encode_momentum_representation_split_protobuf_v3(split)?,
            &split.split_digest,
            |bytes| Ok(decode_momentum_representation_split_protobuf_v3(bytes)?.split_digest),
        )?,
    );
    add_storage_counts(
        &mut counts,
        persist_artifact_v3(
            &root
                .join("representation_registrations")
                .join(format!("{}.pb", registration.registration_digest)),
            &encode_momentum_representation_registration_protobuf_v3(registration)?,
            &registration.registration_digest,
            |bytes| {
                Ok(
                    decode_momentum_representation_registration_protobuf_v3(bytes)?
                        .registration_digest,
                )
            },
        )?,
    );
    Ok(counts)
}

fn reopen_preregistration_v3(
    root: &Path,
) -> Result<
    (
        MomentumRepresentationPathAuditV3,
        MomentumRepresentationSplitV3,
        MomentumRepresentationRegistrationV3,
    ),
    String,
> {
    let audit = read_single_v3(
        &root.join("representation_audits"),
        decode_momentum_representation_audit_protobuf_v3,
    )?;
    let split = read_single_v3(
        &root.join("representation_splits"),
        decode_momentum_representation_split_protobuf_v3,
    )?;
    let registration = read_single_v3(
        &root.join("representation_registrations"),
        decode_momentum_representation_registration_protobuf_v3,
    )?;
    let mut probes = protobuf_paths_v3(&root.join("representation_probes"))?
        .iter()
        .map(|path| fs::read(path).map_err(|_| "V3 probe reopen failed".to_string()))
        .map(|bytes| {
            bytes.and_then(|bytes| decode_momentum_representation_probe_protobuf_v3(&bytes))
        })
        .collect::<Result<Vec<_>, _>>()?;
    probes.sort_by_key(|probe| probe.probe_kind);
    if probes != audit.probes
        || registration.representation_audit_digest != audit.audit_digest
        || registration.split_digest != split.split_digest
    {
        return Err("V3 preregistration reopen binding rejected".to_string());
    }
    Ok((audit, split, registration))
}

fn persist_experiment_v3(
    root: &Path,
    experiment: &RepresentationExperimentV3,
    journal: &MomentumRepresentationJournalV3,
) -> Result<(usize, usize), String> {
    validate_family_v3(&experiment.family)?;
    validate_decision_v3(&experiment.decision, &experiment.family)?;
    validate_journal_v3(journal)?;
    let mut counts = (0, 0);
    for participant in &experiment.family.participants {
        add_storage_counts(
            &mut counts,
            persist_artifact_v3(
                &root
                    .join("participants")
                    .join(format!("{}.pb", participant.participant_digest)),
                &encode_frozen_candidate_participant_protobuf_v3(participant)?,
                &participant.participant_digest,
                |bytes| {
                    Ok(decode_frozen_candidate_participant_protobuf_v3(bytes)?.participant_digest)
                },
            )?,
        );
    }
    for receipt in &experiment.family.qualification_receipts {
        add_storage_counts(
            &mut counts,
            persist_artifact_v3(
                &root
                    .join("qualification_receipts")
                    .join(format!("{}.pb", receipt.receipt_digest)),
                &encode_momentum_representation_qualification_protobuf_v3(receipt)?,
                &receipt.receipt_digest,
                |bytes| {
                    Ok(
                        decode_momentum_representation_qualification_protobuf_v3(bytes)?
                            .receipt_digest,
                    )
                },
            )?,
        );
    }
    for audit in &experiment.family.contribution_audits {
        add_storage_counts(
            &mut counts,
            persist_artifact_v3(
                &root
                    .join("contribution_audits")
                    .join(format!("{}.pb", audit.audit_digest)),
                &encode_mamba_contribution_audit_protobuf_v3(audit)?,
                &audit.audit_digest,
                |bytes| Ok(decode_mamba_contribution_audit_protobuf_v3(bytes)?.audit_digest),
            )?,
        );
    }
    add_storage_counts(
        &mut counts,
        persist_artifact_v3(
            &root
                .join("families")
                .join(format!("{}.pb", experiment.family.family_digest)),
            &encode_momentum_representation_family_protobuf_v3(&experiment.family)?,
            &experiment.family.family_digest,
            |bytes| Ok(decode_momentum_representation_family_protobuf_v3(bytes)?.family_digest),
        )?,
    );
    let family_for_decision = experiment.family.clone();
    add_storage_counts(
        &mut counts,
        persist_artifact_v3(
            &root
                .join("route_decisions")
                .join(format!("{}.pb", experiment.decision.decision_digest)),
            &encode_momentum_representation_decision_protobuf_v3(
                &experiment.decision,
                &experiment.family,
            )?,
            &experiment.decision.decision_digest,
            move |bytes| {
                Ok(decode_momentum_representation_decision_protobuf_v3(
                    bytes,
                    &family_for_decision,
                )?
                .decision_digest)
            },
        )?,
    );
    if let Some(roster) = &experiment.roster {
        let family_for_roster = experiment.family.clone();
        let decision_for_roster = experiment.decision.clone();
        add_storage_counts(
            &mut counts,
            persist_artifact_v3(
                &root
                    .join("rosters")
                    .join(format!("{}.pb", roster.roster_digest)),
                &encode_momentum_representation_roster_protobuf_v3(
                    roster,
                    &experiment.family,
                    &experiment.decision,
                )?,
                &roster.roster_digest,
                move |bytes| {
                    Ok(decode_momentum_representation_roster_protobuf_v3(
                        bytes,
                        &family_for_roster,
                        &decision_for_roster,
                    )?
                    .roster_digest)
                },
            )?,
        );
    }
    if let (Some(evaluation), Some(roster)) =
        (&experiment.evaluation_registration, &experiment.roster)
    {
        let family_for_evaluation = experiment.family.clone();
        let decision_for_evaluation = experiment.decision.clone();
        let roster_for_evaluation = roster.clone();
        add_storage_counts(
            &mut counts,
            persist_artifact_v3(
                &root
                    .join("evaluation_registrations")
                    .join(format!("{}.pb", evaluation.registration_digest)),
                &encode_momentum_representation_evaluation_protobuf_v3(
                    evaluation,
                    &experiment.family,
                    &experiment.decision,
                    roster,
                )?,
                &evaluation.registration_digest,
                move |bytes| {
                    Ok(decode_momentum_representation_evaluation_protobuf_v3(
                        bytes,
                        &family_for_evaluation,
                        &decision_for_evaluation,
                        &roster_for_evaluation,
                    )?
                    .registration_digest)
                },
            )?,
        );
    }
    add_storage_counts(
        &mut counts,
        persist_artifact_v3(
            &root
                .join("journals")
                .join(format!("{}.pb", journal.journal_digest)),
            &encode_momentum_representation_journal_protobuf_v3(journal)?,
            &journal.journal_digest,
            |bytes| Ok(decode_momentum_representation_journal_protobuf_v3(bytes)?.journal_digest),
        )?,
    );
    Ok(counts)
}

fn reopen_experiment_v3(
    root: &Path,
) -> Result<(RepresentationExperimentV3, MomentumRepresentationJournalV3), String> {
    let family = read_single_v3(
        &root.join("families"),
        decode_momentum_representation_family_protobuf_v3,
    )?;
    let decision = read_single_v3(&root.join("route_decisions"), |bytes| {
        decode_momentum_representation_decision_protobuf_v3(bytes, &family)
    })?;
    let roster = if root.join("rosters").is_dir() {
        Some(read_single_v3(&root.join("rosters"), |bytes| {
            decode_momentum_representation_roster_protobuf_v3(bytes, &family, &decision)
        })?)
    } else {
        None
    };
    let evaluation_registration = match (&roster, root.join("evaluation_registrations").is_dir()) {
        (Some(roster), true) => Some(read_single_v3(
            &root.join("evaluation_registrations"),
            |bytes| {
                decode_momentum_representation_evaluation_protobuf_v3(
                    bytes, &family, &decision, roster,
                )
            },
        )?),
        (None, false) => None,
        _ => return Err("V3 evaluation registration presence rejected".to_string()),
    };
    let journal = read_single_v3(
        &root.join("journals"),
        decode_momentum_representation_journal_protobuf_v3,
    )?;
    let participant_digests = protobuf_paths_v3(&root.join("participants"))?
        .iter()
        .map(|path| fs::read(path).map_err(|_| "V3 participant reopen failed".to_string()))
        .map(|bytes| {
            bytes.and_then(|bytes| decode_frozen_candidate_participant_protobuf_v3(&bytes))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let receipt_digests = protobuf_paths_v3(&root.join("qualification_receipts"))?
        .iter()
        .map(|path| fs::read(path).map_err(|_| "V3 receipt reopen failed".to_string()))
        .map(|bytes| {
            bytes.and_then(|bytes| decode_momentum_representation_qualification_protobuf_v3(&bytes))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let contribution_digests = protobuf_paths_v3(&root.join("contribution_audits"))?
        .iter()
        .map(|path| fs::read(path).map_err(|_| "V3 contribution reopen failed".to_string()))
        .map(|bytes| bytes.and_then(|bytes| decode_mamba_contribution_audit_protobuf_v3(&bytes)))
        .collect::<Result<Vec<_>, _>>()?;
    if participant_digests
        .iter()
        .map(|value| value.participant_digest.as_str())
        .collect::<BTreeSet<_>>()
        != family
            .participants
            .iter()
            .map(|value| value.participant_digest.as_str())
            .collect::<BTreeSet<_>>()
        || receipt_digests
            .iter()
            .map(|value| value.receipt_digest.as_str())
            .collect::<BTreeSet<_>>()
            != family
                .qualification_receipts
                .iter()
                .map(|value| value.receipt_digest.as_str())
                .collect::<BTreeSet<_>>()
        || contribution_digests
            .iter()
            .map(|value| value.audit_digest.as_str())
            .collect::<BTreeSet<_>>()
            != family
                .contribution_audits
                .iter()
                .map(|value| value.audit_digest.as_str())
                .collect::<BTreeSet<_>>()
    {
        return Err("V3 family sidecar coverage rejected".to_string());
    }
    let roster_status = if roster.is_some() {
        MomentumRepresentationRosterStatusV3::Registered
    } else if family.qualified_mamba_only_count + family.qualified_mamba_hybrid_count == 0 {
        MomentumRepresentationRosterStatusV3::FrozenMambaRepresentationPathRejected
    } else {
        MomentumRepresentationRosterStatusV3::InsufficientComparators
    };
    let evaluation_registration_status = if evaluation_registration.is_some() {
        MomentumRepresentationEvaluationStatusV3::Registered
    } else {
        match roster_status {
            MomentumRepresentationRosterStatusV3::FrozenMambaRepresentationPathRejected => {
                MomentumRepresentationEvaluationStatusV3::FrozenMambaRepresentationPathRejected
            }
            MomentumRepresentationRosterStatusV3::InsufficientComparators => {
                MomentumRepresentationEvaluationStatusV3::InsufficientComparators
            }
            MomentumRepresentationRosterStatusV3::Registered => {
                MomentumRepresentationEvaluationStatusV3::SafetyContractInvalid
            }
        }
    };
    if journal.family_digest.as_deref() != Some(family.family_digest.as_str())
        || journal.decision_digest.as_deref() != Some(decision.decision_digest.as_str())
        || journal.roster_digest.as_deref()
            != roster.as_ref().map(|value| value.roster_digest.as_str())
        || journal.evaluation_registration_digest.as_deref()
            != evaluation_registration
                .as_ref()
                .map(|value| value.registration_digest.as_str())
    {
        return Err("V3 journal cross-artifact binding rejected".to_string());
    }
    Ok((
        RepresentationExperimentV3 {
            family,
            decision,
            roster,
            roster_status,
            evaluation_registration,
            evaluation_registration_status,
        },
        journal,
    ))
}

fn collect_protected_artifacts_v3(
    root: &Path,
    current: &Path,
    values: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), String> {
    if current == root.join("v3") {
        return Ok(());
    }
    if current.is_file() {
        values.push((
            current
                .strip_prefix(root)
                .map_err(|_| "V3 protected path rejected".to_string())?
                .to_path_buf(),
            fs::read(current).map_err(|_| "V3 protected artifact read failed".to_string())?,
        ));
        return Ok(());
    }
    if !current.is_dir() {
        return Ok(());
    }
    let mut children = fs::read_dir(current)
        .map_err(|_| "V3 protected directory unavailable".to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    children.sort();
    for child in children {
        collect_protected_artifacts_v3(root, &child, values)?;
    }
    Ok(())
}

fn stage_for_decision_v3(
    decision: MomentumRepresentationRouteDecisionV3,
) -> MomentumFrozenMambaRepairStageV3 {
    match decision {
        MomentumRepresentationRouteDecisionV3::FrozenMambaOnlyViable => {
            MomentumFrozenMambaRepairStageV3::V3RepresentationPathViable
        }
        MomentumRepresentationRouteDecisionV3::MambaResidualHybridViable => {
            MomentumFrozenMambaRepairStageV3::V3ResidualHybridViable
        }
        MomentumRepresentationRouteDecisionV3::RawFeatureFallbackOnly
        | MomentumRepresentationRouteDecisionV3::FrozenMambaAddsNoIncrementalSignal => {
            MomentumFrozenMambaRepairStageV3::V3MambaContributionAbsent
        }
        _ => MomentumFrozenMambaRepairStageV3::V3FrozenMambaPathRejected,
    }
}

fn report_digest_v3(value: &MomentumRepresentationReportV3) -> String {
    let mut canonical = value.clone();
    canonical.report_digest.clear();
    digest_without_identity("momentum-representation-report-v3", &canonical)
}

fn base_report_v3(
    mode: AgentPrivateLearningRunModeV0,
    status: MomentumRepresentationExecutionStatusV3,
    protected_artifacts_unchanged: bool,
    active_state_unchanged: bool,
) -> MomentumRepresentationReportV3 {
    let mut report = MomentumRepresentationReportV3 {
        report_version: "momentum-mamba-representation-report-v3".to_string(),
        mode,
        status,
        repair_stage: MomentumFrozenMambaRepairStageV3::V3RepresentationPathPending,
        representation_audit: None,
        split: None,
        registration: None,
        family: None,
        decision: None,
        roster: None,
        roster_status: MomentumRepresentationRosterStatusV3::FrozenMambaRepresentationPathRejected,
        evaluation_registration: None,
        evaluation_registration_status:
            MomentumRepresentationEvaluationStatusV3::FrozenMambaRepresentationPathRejected,
        journal: None,
        artifacts_written: 0,
        duplicate_artifact_count: 0,
        storage_failure_count: usize::from(
            status == MomentumRepresentationExecutionStatusV3::TechnicalFailure,
        ),
        protected_artifacts_unchanged,
        active_state_unchanged,
        safety_counters: zero_safety_counters_v3(),
        report_digest: String::new(),
    };
    report.report_digest = report_digest_v3(&report);
    report
}

pub fn run_momentum_mamba_representation_v3(
    root: &Path,
    snapshots: &[DataSnapshot],
    reservation: &ProtectedEvaluationReservationV1,
    mode: AgentPrivateLearningRunModeV0,
) -> MomentumRepresentationReportV3 {
    let active_before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let mut protected_before = Vec::new();
    if collect_protected_artifacts_v3(root, root, &mut protected_before).is_err() {
        return base_report_v3(
            mode,
            MomentumRepresentationExecutionStatusV3::TechnicalFailure,
            false,
            true,
        );
    }
    let history = match load_frozen_history_v3(root, snapshots) {
        Ok(value) => value,
        Err(_) => {
            return base_report_v3(
                mode,
                MomentumRepresentationExecutionStatusV3::TechnicalFailure,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
        }
    };
    let audit = match derive_representation_audit_v3(&history) {
        Ok(value) => value,
        Err(_) => {
            return base_report_v3(
                mode,
                MomentumRepresentationExecutionStatusV3::TechnicalFailure,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
        }
    };
    let split = match derive_representation_split_v3(&history, reservation) {
        Ok(value) => value,
        Err(_) => {
            let mut report = base_report_v3(
                mode,
                MomentumRepresentationExecutionStatusV3::InsufficientFreshValidation,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
            report.representation_audit = Some(audit);
            report.report_digest = report_digest_v3(&report);
            return report;
        }
    };
    let registration = match derive_representation_registration_v3(&history, &audit, &split) {
        Ok(value) => value,
        Err(_) => {
            let mut report = base_report_v3(
                mode,
                MomentumRepresentationExecutionStatusV3::TechnicalFailure,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
            report.representation_audit = Some(audit);
            report.split = Some(split);
            report.report_digest = report_digest_v3(&report);
            return report;
        }
    };
    let v3_root = root.join("v3").join(AGENT_ID_V3);
    if mode != AgentPrivateLearningRunModeV0::ExecuteLocal {
        let persisted = reopen_preregistration_v3(&v3_root)
            .and_then(|prereg| {
                reopen_experiment_v3(&v3_root).map(|experiment| (prereg, experiment))
            })
            .ok();
        let mut report = base_report_v3(
            mode,
            if persisted.is_some() {
                MomentumRepresentationExecutionStatusV3::AlreadyExecuted
            } else {
                MomentumRepresentationExecutionStatusV3::Planned
            },
            true,
            stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before,
        );
        if let Some(((stored_audit, stored_split, stored_registration), (experiment, journal))) =
            persisted
        {
            report.repair_stage = journal.repair_stage;
            report.representation_audit = Some(stored_audit);
            report.split = Some(stored_split);
            report.registration = Some(stored_registration);
            report.family = Some(experiment.family);
            report.decision = Some(experiment.decision);
            report.roster = experiment.roster;
            report.roster_status = experiment.roster_status;
            report.evaluation_registration = experiment.evaluation_registration;
            report.evaluation_registration_status = experiment.evaluation_registration_status;
            report.journal = Some(journal);
        } else {
            report.representation_audit = Some(audit);
            report.split = Some(split);
            report.registration = Some(registration);
        }
        report.report_digest = report_digest_v3(&report);
        return report;
    }
    let persisted_before = reopen_preregistration_v3(&v3_root)
        .and_then(|prereg| reopen_experiment_v3(&v3_root).map(|experiment| (prereg, experiment)))
        .ok();
    let mut storage_counts = (0, 0);
    let ((stored_audit, stored_split, stored_registration), (experiment, journal)) =
        if let Some(value) = persisted_before {
            value
        } else {
            let prereg_counts =
                match persist_preregistration_v3(&v3_root, &audit, &split, &registration) {
                    Ok(value) => value,
                    Err(_) => {
                        let mut report = base_report_v3(
                            mode,
                            MomentumRepresentationExecutionStatusV3::TechnicalFailure,
                            true,
                            stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                                == active_before,
                        );
                        report.representation_audit = Some(audit);
                        report.split = Some(split);
                        report.registration = Some(registration);
                        report.storage_failure_count = 1;
                        report.report_digest = report_digest_v3(&report);
                        return report;
                    }
                };
            add_storage_counts(&mut storage_counts, prereg_counts);
            let prereg = match reopen_preregistration_v3(&v3_root) {
                Ok(value) if value == (audit.clone(), split.clone(), registration.clone()) => value,
                _ => {
                    let mut report = base_report_v3(
                        mode,
                        MomentumRepresentationExecutionStatusV3::TechnicalFailure,
                        true,
                        stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                            == active_before,
                    );
                    report.storage_failure_count = 1;
                    report.report_digest = report_digest_v3(&report);
                    return report;
                }
            };
            let experiment = match run_representation_experiment_v3(
                &history,
                &prereg.0,
                &prereg.1,
                &prereg.2,
                reservation,
            ) {
                Ok(value) => value,
                Err(_) => {
                    let mut report = base_report_v3(
                        mode,
                        MomentumRepresentationExecutionStatusV3::TechnicalFailure,
                        true,
                        stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                            == active_before,
                    );
                    report.representation_audit = Some(prereg.0);
                    report.split = Some(prereg.1);
                    report.registration = Some(prereg.2);
                    report.report_digest = report_digest_v3(&report);
                    return report;
                }
            };
            let stage = stage_for_decision_v3(experiment.decision.decision);
            let mut journal = MomentumRepresentationJournalV3 {
                journal_version: JOURNAL_VERSION_V3.to_string(),
                agent_id: AGENT_ID_V3.to_string(),
                repair_stage: stage,
                representation_audit_digest: prereg.0.audit_digest.clone(),
                split_digest: prereg.1.split_digest.clone(),
                registration_digest: prereg.2.registration_digest.clone(),
                family_digest: Some(experiment.family.family_digest.clone()),
                decision_digest: Some(experiment.decision.decision_digest.clone()),
                roster_digest: experiment
                    .roster
                    .as_ref()
                    .map(|value| value.roster_digest.clone()),
                evaluation_registration_digest: experiment
                    .evaluation_registration
                    .as_ref()
                    .map(|value| value.registration_digest.clone()),
                prior_validation_used_for_v3_qualification: false,
                final_reserve_accessed: false,
                warm_start: false,
                v1_head_reused: false,
                v2_head_reused: false,
                fresh_deterministic_initialization: true,
                status: MomentumRepresentationExecutionStatusV3::Executed,
                journal_digest: String::new(),
            };
            journal.journal_digest = journal_digest_v3(&journal);
            if validate_journal_v3(&journal).is_err() {
                let mut report = base_report_v3(
                    mode,
                    MomentumRepresentationExecutionStatusV3::TechnicalFailure,
                    true,
                    true,
                );
                report.storage_failure_count = 1;
                report.report_digest = report_digest_v3(&report);
                return report;
            }
            let experiment_counts = match persist_experiment_v3(&v3_root, &experiment, &journal) {
                Ok(value) => value,
                Err(_) => {
                    let mut report = base_report_v3(
                        mode,
                        MomentumRepresentationExecutionStatusV3::TechnicalFailure,
                        true,
                        stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                            == active_before,
                    );
                    report.storage_failure_count = 1;
                    report.report_digest = report_digest_v3(&report);
                    return report;
                }
            };
            add_storage_counts(&mut storage_counts, experiment_counts);
            (prereg, (experiment, journal))
        };
    if storage_counts.0 == 0 {
        match persist_preregistration_v3(
            &v3_root,
            &stored_audit,
            &stored_split,
            &stored_registration,
        ) {
            Ok(value) => add_storage_counts(&mut storage_counts, value),
            Err(_) => {
                let mut protected_after = Vec::new();
                let protected_artifacts_unchanged =
                    collect_protected_artifacts_v3(root, root, &mut protected_after).is_ok()
                        && protected_before == protected_after;
                return base_report_v3(
                    mode,
                    MomentumRepresentationExecutionStatusV3::TechnicalFailure,
                    protected_artifacts_unchanged,
                    stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                        == active_before,
                );
            }
        }
        match persist_experiment_v3(&v3_root, &experiment, &journal) {
            Ok(value) => add_storage_counts(&mut storage_counts, value),
            Err(_) => {
                let mut protected_after = Vec::new();
                let protected_artifacts_unchanged =
                    collect_protected_artifacts_v3(root, root, &mut protected_after).is_ok()
                        && protected_before == protected_after;
                return base_report_v3(
                    mode,
                    MomentumRepresentationExecutionStatusV3::TechnicalFailure,
                    protected_artifacts_unchanged,
                    stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                        == active_before,
                );
            }
        }
    }
    let reopened = reopen_preregistration_v3(&v3_root)
        .and_then(|prereg| reopen_experiment_v3(&v3_root).map(|experiment| (prereg, experiment)));
    let (
        (reopened_audit, reopened_split, reopened_registration),
        (reopened_experiment, reopened_journal),
    ) = match reopened {
        Ok(value) => value,
        Err(_) => {
            let mut report = base_report_v3(
                mode,
                MomentumRepresentationExecutionStatusV3::TechnicalFailure,
                true,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
            );
            report.storage_failure_count = 1;
            report.report_digest = report_digest_v3(&report);
            return report;
        }
    };
    let mut protected_after = Vec::new();
    let protected_artifacts_unchanged =
        collect_protected_artifacts_v3(root, root, &mut protected_after).is_ok()
            && protected_before == protected_after;
    let active_state_unchanged =
        stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before;
    let status = if storage_counts.0 == 0 {
        MomentumRepresentationExecutionStatusV3::AlreadyExecuted
    } else {
        MomentumRepresentationExecutionStatusV3::Executed
    };
    let mut report = base_report_v3(
        mode,
        status,
        protected_artifacts_unchanged,
        active_state_unchanged,
    );
    report.repair_stage = reopened_journal.repair_stage;
    report.representation_audit = Some(reopened_audit);
    report.split = Some(reopened_split);
    report.registration = Some(reopened_registration);
    report.family = Some(reopened_experiment.family);
    report.decision = Some(reopened_experiment.decision);
    report.roster = reopened_experiment.roster;
    report.roster_status = reopened_experiment.roster_status;
    report.evaluation_registration = reopened_experiment.evaluation_registration;
    report.evaluation_registration_status = reopened_experiment.evaluation_registration_status;
    report.journal = Some(reopened_journal);
    report.artifacts_written = storage_counts.0;
    report.duplicate_artifact_count = storage_counts.1;
    report.storage_failure_count = usize::from(
        !protected_artifacts_unchanged
            || !active_state_unchanged
            || (status == MomentumRepresentationExecutionStatusV3::AlreadyExecuted
                && storage_counts.0 > 0),
    );
    report.report_digest = report_digest_v3(&report);
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn probe_fixture(
        kind: MomentumRepresentationProbeKindV3,
        index: usize,
    ) -> MomentumRepresentationProbeV3 {
        let mut probe = MomentumRepresentationProbeV3 {
            probe_kind: kind,
            source_snapshot_digest: "snapshot".to_string(),
            consumed_range_digest: "consumed".to_string(),
            feature_policy_digest: "feature-policy".to_string(),
            encoder_digest: (kind != MomentumRepresentationProbeKindV3::RawFeatureLinearProbe)
                .then(|| "encoder".to_string()),
            representation_diagnostic_digest: format!("representation-{index}"),
            private_probe_metric_digest: format!("private-metric-{index}"),
            status: MomentumRepresentationProbeStatusV3::NonCollapsedPrediction,
            probe_digest: String::new(),
        };
        probe.probe_digest = probe_digest_v3(&probe);
        probe
    }

    fn audit_fixture() -> MomentumRepresentationPathAuditV3 {
        let mut audit = MomentumRepresentationPathAuditV3 {
            audit_version: AUDIT_VERSION_V3.to_string(),
            v1_family_digest: "v1-family".to_string(),
            v2_family_digest: "v2-family".to_string(),
            v2_collapse_audit_digest: "v2-audit".to_string(),
            probes: vec![
                probe_fixture(MomentumRepresentationProbeKindV3::RawFeatureLinearProbe, 0),
                probe_fixture(MomentumRepresentationProbeKindV3::MambaLastOutputProbe, 1),
                probe_fixture(MomentumRepresentationProbeKindV3::MambaMeanOutputProbe, 2),
                probe_fixture(
                    MomentumRepresentationProbeKindV3::MambaLastMeanConcatProbe,
                    3,
                ),
            ],
            head_only_repair_exhausted: true,
            fresh_v3_validation_accessed: false,
            audit_digest: String::new(),
        };
        audit.probes.sort_by_key(|probe| probe.probe_kind);
        audit.audit_digest = audit_digest_v3(&audit);
        audit
    }

    fn split_fixture() -> MomentumRepresentationSplitV3 {
        let mut split = MomentumRepresentationSplitV3 {
            split_version: SPLIT_VERSION_V3.to_string(),
            source_snapshot_digest: "snapshot".to_string(),
            v1_usage_ledger_digest: "v1-ledger".to_string(),
            v2_split_digest: "v2-split".to_string(),
            training_range: IndexRangeV0 { start: 0, end: 224 },
            purge_range: IndexRangeV0 {
                start: 224,
                end: 240,
            },
            fresh_validation_range: IndexRangeV0 {
                start: 240,
                end: 264,
            },
            final_reserved_range: IndexRangeV0 {
                start: 264,
                end: 312,
            },
            minimum_validation_samples: 24,
            minimum_final_reserved_samples: 48,
            prior_validation_overlap_count: 0,
            prospective_overlap_count: 0,
            future_evaluation_overlap_count: 0,
            split_digest: String::new(),
        };
        split.split_digest = split_digest_v3(&split);
        split
    }

    fn variant_fixture(
        kind: MomentumRepresentationInputKindV3,
        index: usize,
    ) -> MomentumRepresentationVariantConfigV3 {
        let (variant_id, pooling_policy) = match kind {
            MomentumRepresentationInputKindV3::MambaLastOutput => {
                ("last-output-control", "LastOutput")
            }
            MomentumRepresentationInputKindV3::MambaMeanOutput => ("mean-output", "MeanOutput"),
            MomentumRepresentationInputKindV3::MambaLastMeanConcat => {
                ("last-mean-concat", "LastOutput+MeanOutput")
            }
            MomentumRepresentationInputKindV3::MambaRawFeatureResidual => (
                "raw-feature-residual",
                "LastOutput+SequenceEndRawFeatureResidual",
            ),
        };
        let mut variant = MomentumRepresentationVariantConfigV3 {
            variant_id: variant_id.to_string(),
            input_kind: kind,
            pooling_policy: pooling_policy.to_string(),
            raw_feature_residual_enabled: kind
                == MomentumRepresentationInputKindV3::MambaRawFeatureResidual,
            head_kind: "LogisticPredictionHeadV0".to_string(),
            learning_rate_bits: 0.02_f32.to_bits(),
            l2_regularization_bits: 0.0_f32.to_bits(),
            maximum_epochs: 30,
            initialization_seed: 79 + index as u64,
            encoder_frozen: true,
            feature_policy_digest: "feature-policy".to_string(),
            label_policy_digest: "label-policy".to_string(),
            training_policy_digest: format!("training-policy-{index}"),
            variant_digest: String::new(),
        };
        variant.variant_digest = variant_digest_v3(&variant);
        variant
    }

    fn registration_fixture() -> MomentumRepresentationRegistrationV3 {
        let mut registration = MomentumRepresentationRegistrationV3 {
            registration_version: REGISTRATION_VERSION_V3.to_string(),
            agent_id: AGENT_ID_V3.to_string(),
            source_snapshot_digest: "snapshot".to_string(),
            canonical_intent_digest: "intent".to_string(),
            canonical_view_digest: "view".to_string(),
            representation_audit_digest: audit_fixture().audit_digest,
            split_digest: split_fixture().split_digest,
            variants: vec![
                variant_fixture(MomentumRepresentationInputKindV3::MambaLastOutput, 0),
                variant_fixture(MomentumRepresentationInputKindV3::MambaMeanOutput, 1),
                variant_fixture(MomentumRepresentationInputKindV3::MambaLastMeanConcat, 2),
                variant_fixture(
                    MomentumRepresentationInputKindV3::MambaRawFeatureResidual,
                    3,
                ),
            ],
            maximum_variants: 4,
            contribution_policy_digest: contribution_policy_digest_v3(),
            fresh_validation_hidden: true,
            historical_test_forbidden: true,
            future_evaluation_forbidden: true,
            winner_selection_forbidden: true,
            active_promotion_forbidden: true,
            reward_application_forbidden: true,
            registration_digest: String::new(),
        };
        registration.registration_digest = registration_digest_v3(&registration);
        registration
    }

    fn participant_fixture(
        index: usize,
        role: MomentumRepresentationParticipantRoleV3,
        input_kind: &str,
    ) -> FrozenCandidateParticipantV3 {
        let learned = matches!(
            role,
            MomentumRepresentationParticipantRoleV3::MambaOnly
                | MomentumRepresentationParticipantRoleV3::MambaResidualHybrid
        );
        let model_kind = match role {
            MomentumRepresentationParticipantRoleV3::MambaOnly => match input_kind {
                "MambaLastOutput" => "FrozenMambaLastOutputLogisticV3",
                "MambaMeanOutput" => "FrozenMambaMeanOutputLogisticV3",
                "MambaLastMeanConcat" => "FrozenMambaLastMeanConcatLogisticV3",
                _ => "invalid",
            },
            MomentumRepresentationParticipantRoleV3::MambaResidualHybrid => {
                "FrozenMambaRawResidualLogisticV3"
            }
            MomentumRepresentationParticipantRoleV3::LinearComparator => "LinearMomentumBaselineV3",
            MomentumRepresentationParticipantRoleV3::ConstantBenchmark => {
                "ConstantProbabilityBaselineV3"
            }
        };
        let mut participant = FrozenCandidateParticipantV3 {
            participant_version: PARTICIPANT_VERSION_V3.to_string(),
            participant_id: format!("participant-{index}"),
            participant_role: role,
            model_kind: model_kind.to_string(),
            input_kind: input_kind.to_string(),
            variant_digest: learned.then(|| format!("variant-{index}")),
            source_snapshot_digest: "snapshot".to_string(),
            training_range_digest: "training-range".to_string(),
            fresh_validation_range_digest: "validation-range".to_string(),
            validation_timestamp_digest: "validation-timestamps".to_string(),
            model_artifact_digest: format!("model-artifact-{index}"),
            parameter_digest: format!("parameters-{index}"),
            feature_normalizer_digest: "feature-normalizer".to_string(),
            representation_normalizer_digest: format!("representation-normalizer-{index}"),
            encoder_digest: learned.then(|| "encoder".to_string()),
            feature_policy_digest: "feature-policy".to_string(),
            label_policy_digest: "label-policy".to_string(),
            training_policy_digest: format!("training-policy-{index}"),
            initialization_digest: format!("initialization-{index}"),
            warm_start: false,
            v1_head_reused: false,
            v2_head_reused: false,
            fresh_deterministic_initialization: true,
            encoder_frozen: learned,
            deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
            participant_digest: String::new(),
        };
        participant.participant_digest = participant_digest_v3(&participant);
        participant
    }

    fn contribution_fixture(
        participant: &FrozenCandidateParticipantV3,
        status: MambaContributionStatusV3,
    ) -> MambaContributionAuditV3 {
        let residual = participant.participant_role
            == MomentumRepresentationParticipantRoleV3::MambaResidualHybrid;
        let (mamba_effect, raw_effect) = match status {
            MambaContributionStatusV3::NotApplicable => ("NotApplicable", "NotApplicable"),
            MambaContributionStatusV3::MaterialContribution => ("Material", "Material"),
            MambaContributionStatusV3::DetectableButBelowPolicy => {
                ("DetectableBelowPolicy", "DetectableBelowPolicy")
            }
            MambaContributionStatusV3::NoDetectableContribution => {
                ("NotDetectable", "NotDetectable")
            }
            MambaContributionStatusV3::RawFeatureDominated => ("NotDetectable", "Material"),
            MambaContributionStatusV3::Invalid => ("NotDetectable", "NotDetectable"),
        };
        let mut audit = MambaContributionAuditV3 {
            participant_digest: participant.participant_digest.clone(),
            mamba_parameter_block_digest: "mamba-block".to_string(),
            raw_parameter_block_digest: "raw-block".to_string(),
            mamba_block_nonzero: status != MambaContributionStatusV3::Invalid,
            raw_block_nonzero: residual && status != MambaContributionStatusV3::Invalid,
            full_prediction_digest: "full-prediction".to_string(),
            mamba_ablated_prediction_digest: "mamba-ablated".to_string(),
            raw_ablated_prediction_digest: "raw-ablated".to_string(),
            mamba_ablation_effect_status: mamba_effect.to_string(),
            raw_ablation_effect_status: raw_effect.to_string(),
            contribution_policy_digest: contribution_policy_digest_v3(),
            contribution_status: status,
            audit_digest: String::new(),
        };
        audit.audit_digest = contribution_digest_v3(&audit);
        audit
    }

    fn receipt_fixture(
        participant: &FrozenCandidateParticipantV3,
        status: MomentumRepresentationQualificationStatusV3,
        contribution: Option<&MambaContributionAuditV3>,
    ) -> MomentumRepresentationQualificationReceiptV3 {
        let mut receipt = MomentumRepresentationQualificationReceiptV3 {
            receipt_version: RECEIPT_VERSION_V3.to_string(),
            participant_id: participant.participant_id.clone(),
            participant_digest: participant.participant_digest.clone(),
            input_kind: participant.input_kind.clone(),
            fresh_validation_range_digest: participant.fresh_validation_range_digest.clone(),
            qualification_policy_digest: "qualification-policy".to_string(),
            private_metric_digest: "private-metric".to_string(),
            contribution_audit_digest: contribution.map(|audit| audit.audit_digest.clone()),
            status,
            validation_parameter_updates: 0,
            historical_test_reads: 0,
            future_evaluation_reads: 0,
            receipt_digest: String::new(),
        };
        receipt.receipt_digest = receipt_digest_v3(&receipt);
        receipt
    }

    fn family_fixture(genuine_mamba: bool, raw_fallback: bool) -> MomentumRepresentationFamilyV3 {
        let participants = vec![
            participant_fixture(
                0,
                MomentumRepresentationParticipantRoleV3::MambaOnly,
                "MambaLastOutput",
            ),
            participant_fixture(
                1,
                MomentumRepresentationParticipantRoleV3::MambaOnly,
                "MambaMeanOutput",
            ),
            participant_fixture(
                2,
                MomentumRepresentationParticipantRoleV3::MambaOnly,
                "MambaLastMeanConcat",
            ),
            participant_fixture(
                3,
                MomentumRepresentationParticipantRoleV3::MambaResidualHybrid,
                "MambaRawFeatureResidual",
            ),
            participant_fixture(
                4,
                MomentumRepresentationParticipantRoleV3::LinearComparator,
                "RawFeatureLinearComparator",
            ),
            participant_fixture(
                5,
                MomentumRepresentationParticipantRoleV3::ConstantBenchmark,
                "TrainingPrevalenceConstant",
            ),
        ];
        let contributions = vec![
            contribution_fixture(&participants[0], MambaContributionStatusV3::NotApplicable),
            contribution_fixture(&participants[1], MambaContributionStatusV3::NotApplicable),
            contribution_fixture(&participants[2], MambaContributionStatusV3::NotApplicable),
            contribution_fixture(
                &participants[3],
                if raw_fallback {
                    MambaContributionStatusV3::RawFeatureDominated
                } else {
                    MambaContributionStatusV3::NoDetectableContribution
                },
            ),
        ];
        let statuses = [
            if genuine_mamba {
                MomentumRepresentationQualificationStatusV3::QualifiedMambaOnly
            } else {
                MomentumRepresentationQualificationStatusV3::RejectedProbabilityCollapse
            },
            MomentumRepresentationQualificationStatusV3::RejectedProbabilityCollapse,
            MomentumRepresentationQualificationStatusV3::RejectedProbabilityCollapse,
            if raw_fallback {
                MomentumRepresentationQualificationStatusV3::QualifiedRawFallbackNotMamba
            } else {
                MomentumRepresentationQualificationStatusV3::RejectedContributionInvariant
            },
            MomentumRepresentationQualificationStatusV3::ComparatorQualified,
            MomentumRepresentationQualificationStatusV3::BenchmarkQualified,
        ];
        let receipts = participants
            .iter()
            .enumerate()
            .map(|(index, participant)| {
                receipt_fixture(
                    participant,
                    statuses[index],
                    (index == 3).then_some(&contributions[3]),
                )
            })
            .collect::<Vec<_>>();
        let mut family = MomentumRepresentationFamilyV3 {
            family_version: FAMILY_VERSION_V3.to_string(),
            agent_id: AGENT_ID_V3.to_string(),
            source_snapshot_digest: "snapshot".to_string(),
            canonical_view_digest: "view".to_string(),
            representation_audit_digest: "audit".to_string(),
            split_digest: "split".to_string(),
            registration_digest: "registration".to_string(),
            participants,
            qualification_receipts: receipts,
            contribution_audits: contributions,
            qualified_mamba_only_count: usize::from(genuine_mamba),
            qualified_mamba_hybrid_count: 0,
            qualified_raw_fallback_count: usize::from(raw_fallback),
            qualified_comparator_count: 2,
            winner_selected: false,
            historical_test_accessed: false,
            eligible_for_active_committee: false,
            eligible_for_promotion: false,
            eligible_for_reward: false,
            family_digest: String::new(),
        };
        family.family_digest = family_digest_v3(&family);
        family
    }

    fn decision_fixture(
        family: &MomentumRepresentationFamilyV3,
    ) -> MomentumRepresentationRouteDecisionArtifactV3 {
        let qualified_mamba_only_digests = sorted_unique(
            family
                .qualification_receipts
                .iter()
                .filter(|receipt| {
                    receipt.status
                        == MomentumRepresentationQualificationStatusV3::QualifiedMambaOnly
                })
                .map(|receipt| receipt.participant_digest.clone())
                .collect(),
        );
        let raw_fallback_digests = sorted_unique(
            family
                .qualification_receipts
                .iter()
                .filter(|receipt| {
                    receipt.status
                        == MomentumRepresentationQualificationStatusV3::QualifiedRawFallbackNotMamba
                })
                .map(|receipt| receipt.participant_digest.clone())
                .collect(),
        );
        let rejected_route_digests = sorted_unique(
            family
                .participants
                .iter()
                .filter(|participant| {
                    matches!(
                        participant.participant_role,
                        MomentumRepresentationParticipantRoleV3::MambaOnly
                            | MomentumRepresentationParticipantRoleV3::MambaResidualHybrid
                    )
                })
                .filter(|participant| {
                    !qualified_mamba_only_digests.contains(&participant.participant_digest)
                        && !raw_fallback_digests.contains(&participant.participant_digest)
                })
                .map(|participant| participant.participant_digest.clone())
                .collect(),
        );
        let decision = if !qualified_mamba_only_digests.is_empty() {
            MomentumRepresentationRouteDecisionV3::FrozenMambaOnlyViable
        } else if !raw_fallback_digests.is_empty() {
            MomentumRepresentationRouteDecisionV3::RawFeatureFallbackOnly
        } else {
            MomentumRepresentationRouteDecisionV3::AllRepresentationRoutesCollapsed
        };
        let mut value = MomentumRepresentationRouteDecisionArtifactV3 {
            decision_version: DECISION_VERSION_V3.to_string(),
            v1_family_digest: "v1-family".to_string(),
            v2_family_digest: "v2-family".to_string(),
            v3_family_digest: family.family_digest.clone(),
            qualified_mamba_only_digests,
            qualified_mamba_hybrid_digests: Vec::new(),
            raw_fallback_digests,
            rejected_route_digests,
            further_head_only_repair_forbidden: true,
            further_frozen_representation_sweep_forbidden: family.qualified_mamba_only_count == 0,
            decision,
            decision_digest: String::new(),
        };
        value.decision_digest = decision_digest_v3(&value);
        value
    }

    fn evaluation_fixture(
        family: &MomentumRepresentationFamilyV3,
        decision: &MomentumRepresentationRouteDecisionArtifactV3,
        roster: &MomentumRepresentationFutureRosterV3,
    ) -> MomentumRepresentationEvaluationRegistrationV3 {
        let included = roster
            .qualified_genuine_mamba_digests
            .iter()
            .chain(&roster.qualified_comparator_digests)
            .collect::<BTreeSet<_>>();
        let qualification_receipt_digests = sorted_unique(
            family
                .qualification_receipts
                .iter()
                .filter(|receipt| included.contains(&receipt.participant_digest))
                .map(|receipt| receipt.receipt_digest.clone())
                .collect(),
        );
        let contribution_audit_digests = sorted_unique(
            family
                .contribution_audits
                .iter()
                .filter(|audit| included.contains(&audit.participant_digest))
                .map(|audit| audit.audit_digest.clone())
                .collect(),
        );
        let mut value = MomentumRepresentationEvaluationRegistrationV3 {
            registration_version: EVALUATION_VERSION_V3.to_string(),
            agent_id: AGENT_ID_V3.to_string(),
            family_digest: family.family_digest.clone(),
            roster_digest: roster.roster_digest.clone(),
            decision_digest: decision.decision_digest.clone(),
            qualification_receipt_digests,
            contribution_audit_digests,
            source_snapshot_digest: "snapshot".to_string(),
            source_boundary_timestamp_ms: 100,
            protected_registration_digests: vec!["protected".to_string()],
            protected_timestamp_ms: vec![101, 102, 103, 104],
            prior_validation_and_reserved_range_digests: (0..6)
                .map(|index| format!("range-{index}"))
                .collect(),
            provider_finality_boundary_ms: 105,
            minimum_accepted_timestamp_ms: 106,
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
        value.registration_digest = evaluation_digest_v3(&value);
        value
    }

    fn journal_fixture(
        family: &MomentumRepresentationFamilyV3,
        decision: &MomentumRepresentationRouteDecisionArtifactV3,
    ) -> MomentumRepresentationJournalV3 {
        let mut journal = MomentumRepresentationJournalV3 {
            journal_version: JOURNAL_VERSION_V3.to_string(),
            agent_id: AGENT_ID_V3.to_string(),
            repair_stage: stage_for_decision_v3(decision.decision),
            representation_audit_digest: audit_fixture().audit_digest,
            split_digest: split_fixture().split_digest,
            registration_digest: registration_fixture().registration_digest,
            family_digest: Some(family.family_digest.clone()),
            decision_digest: Some(decision.decision_digest.clone()),
            roster_digest: None,
            evaluation_registration_digest: None,
            prior_validation_used_for_v3_qualification: false,
            final_reserve_accessed: false,
            warm_start: false,
            v1_head_reused: false,
            v2_head_reused: false,
            fresh_deterministic_initialization: true,
            status: MomentumRepresentationExecutionStatusV3::Executed,
            journal_digest: String::new(),
        };
        journal.journal_digest = journal_digest_v3(&journal);
        journal
    }

    fn temporary_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "soma-mamba-representation-v3-{}-{nonce}",
            std::process::id()
        ))
    }

    #[test]
    fn pr10_invariants_remain_encoded_in_v2_binding() {
        let audit = audit_fixture();
        assert_eq!(audit.v2_family_digest, "v2-family");
        assert_eq!(audit.v2_collapse_audit_digest, "v2-audit");
    }

    #[test]
    fn protected_collector_keeps_v1_and_v2_byte_identical() {
        let root = temporary_root();
        fs::create_dir_all(root.join("v1")).unwrap();
        fs::create_dir_all(root.join("v2")).unwrap();
        fs::create_dir_all(root.join("v3")).unwrap();
        fs::write(root.join("v1/a.pb"), b"v1").unwrap();
        fs::write(root.join("v2/b.pb"), b"v2").unwrap();
        fs::write(root.join("v3/c.pb"), b"v3").unwrap();
        let mut before = Vec::new();
        collect_protected_artifacts_v3(&root, &root, &mut before).unwrap();
        fs::write(root.join("v3/c.pb"), b"changed-v3").unwrap();
        let mut after = Vec::new();
        collect_protected_artifacts_v3(&root, &root, &mut after).unwrap();
        assert_eq!(before, after);
        assert_eq!(before.len(), 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn head_only_repair_is_marked_exhausted() {
        assert!(audit_fixture().head_only_repair_exhausted);
    }

    #[test]
    fn probes_forbid_fresh_v3_validation() {
        let mut audit = audit_fixture();
        audit.fresh_v3_validation_accessed = true;
        audit.audit_digest = audit_digest_v3(&audit);
        assert!(validate_audit_v3(&audit).is_err());
    }

    #[test]
    fn last_mean_and_concat_probe_identities_are_distinct() {
        let audit = audit_fixture();
        assert_eq!(
            audit
                .probes
                .iter()
                .map(|probe| &probe.representation_diagnostic_digest)
                .collect::<BTreeSet<_>>()
                .len(),
            4
        );
    }

    #[test]
    fn residual_input_binds_mamba_and_raw_blocks() {
        let left = vec![EncodedTrainingExampleV0 {
            representation: vec![1.0, 2.0],
            label: 1.0,
            snapshot_ids: vec!["s".to_string()],
        }];
        let right = vec![EncodedTrainingExampleV0 {
            representation: vec![3.0],
            label: 1.0,
            snapshot_ids: vec!["s".to_string()],
        }];
        assert_eq!(
            concatenate_encoded_v3(&left, &right).unwrap()[0].representation,
            vec![1.0, 2.0, 3.0]
        );
    }

    #[test]
    fn split_is_bound_to_v2_remaining_reserve() {
        let split = split_fixture();
        assert_eq!(split.v2_split_digest, "v2-split");
        assert!(validate_split_v3(&split).is_ok());
    }

    #[test]
    fn final_reserve_remains_untouched() {
        let family = family_fixture(false, false);
        let decision = decision_fixture(&family);
        let mut journal = journal_fixture(&family, &decision);
        journal.final_reserve_accessed = true;
        journal.journal_digest = journal_digest_v3(&journal);
        assert!(validate_journal_v3(&journal).is_err());
    }

    #[test]
    fn purge_covers_feature_sequence_and_label_horizons() {
        let split = split_fixture();
        assert_eq!(split.purge_range.end - split.purge_range.start, 16);
    }

    #[test]
    fn preregistration_persists_before_participants() {
        let root = temporary_root();
        let counts = persist_preregistration_v3(
            &root,
            &audit_fixture(),
            &split_fixture(),
            &registration_fixture(),
        )
        .unwrap();
        assert_eq!(counts, (7, 0));
        assert!(!root.join("participants").exists());
        assert!(reopen_preregistration_v3(&root).is_ok());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn exactly_four_learned_routes_are_frozen() {
        let registration = registration_fixture();
        assert_eq!(registration.variants.len(), 4);
        assert_eq!(
            registration
                .variants
                .iter()
                .map(|variant| variant.input_kind)
                .collect::<BTreeSet<_>>()
                .len(),
            4
        );
    }

    #[test]
    fn result_dependent_fifth_route_rejects() {
        let mut registration = registration_fixture();
        registration.variants.push(variant_fixture(
            MomentumRepresentationInputKindV3::MambaLastOutput,
            4,
        ));
        registration.registration_digest = registration_digest_v3(&registration);
        assert!(validate_registration_v3(&registration).is_err());
    }

    #[test]
    fn encoder_mutation_rejects() {
        let mut participant = participant_fixture(
            0,
            MomentumRepresentationParticipantRoleV3::MambaOnly,
            "MambaLastOutput",
        );
        participant.encoder_frozen = false;
        participant.participant_digest = participant_digest_v3(&participant);
        assert!(validate_participant_v3(&participant).is_err());
    }

    #[test]
    fn v1_and_v2_head_reuse_rejects() {
        let mut participant = participant_fixture(
            0,
            MomentumRepresentationParticipantRoleV3::MambaOnly,
            "MambaLastOutput",
        );
        participant.v2_head_reused = true;
        participant.participant_digest = participant_digest_v3(&participant);
        assert!(validate_participant_v3(&participant).is_err());
        participant.v2_head_reused = false;
        participant.v1_head_reused = true;
        participant.participant_digest = participant_digest_v3(&participant);
        assert!(validate_participant_v3(&participant).is_err());
    }

    #[test]
    fn every_route_requires_fresh_deterministic_initialization() {
        let mut participant = participant_fixture(
            0,
            MomentumRepresentationParticipantRoleV3::MambaOnly,
            "MambaLastOutput",
        );
        participant.fresh_deterministic_initialization = false;
        participant.participant_digest = participant_digest_v3(&participant);
        assert!(validate_participant_v3(&participant).is_err());
    }

    #[test]
    fn representation_normalizer_is_fit_on_training_only() {
        let training = vec![
            EncodedTrainingExampleV0 {
                representation: vec![0.0],
                label: 0.0,
                snapshot_ids: vec![],
            },
            EncodedTrainingExampleV0 {
                representation: vec![2.0],
                label: 1.0,
                snapshot_ids: vec![],
            },
        ];
        let validation = vec![EncodedTrainingExampleV0 {
            representation: vec![100.0],
            label: 1.0,
            snapshot_ids: vec![],
        }];
        let normalizer = RepresentationNormalizerV0::fit(&training).unwrap();
        let digest = normalizer.digest();
        normalizer.transform(&validation).unwrap();
        assert_eq!(normalizer.digest(), digest);
    }

    #[test]
    fn every_route_uses_identical_validation_timestamps() {
        let family = family_fixture(true, true);
        assert_eq!(
            family
                .participants
                .iter()
                .map(|participant| &participant.validation_timestamp_digest)
                .collect::<BTreeSet<_>>()
                .len(),
            1
        );
    }

    #[test]
    fn validation_parameter_updates_remain_zero() {
        assert!(
            family_fixture(true, true)
                .qualification_receipts
                .iter()
                .all(|receipt| receipt.validation_parameter_updates == 0)
        );
    }

    #[test]
    fn linear_and_constant_use_same_split() {
        let family = family_fixture(true, true);
        assert_eq!(
            family.participants[4].fresh_validation_range_digest,
            family.participants[5].fresh_validation_range_digest
        );
    }

    #[test]
    fn mamba_only_qualification_requires_noncollapse() {
        let metric = EvaluationMetricsV0 {
            sample_count: 24,
            brier_score: 0.2,
            accuracy: 0.5,
            positive_label_rate: 0.5,
            mean_predicted_probability: 0.4,
            high_confidence_error_count: 0,
            abstention_count: 0,
            calibration_buckets: vec![],
        };
        let diagnostic = RepresentationDiagnosticV3 {
            finite: true,
            variance: VarianceClassV3::Adequate,
            effective_rank: RankClassV3::Adequate,
            redundant_dimension_count: 0,
            digest: "d".to_string(),
        };
        assert_eq!(
            learned_base_qualification_v3(&metric, &vec![0.4; 24], &diagnostic, 24, true),
            MomentumRepresentationQualificationStatusV3::RejectedProbabilityCollapse
        );
    }

    #[test]
    fn residual_qualification_requires_contribution_audit() {
        let participant = participant_fixture(
            3,
            MomentumRepresentationParticipantRoleV3::MambaResidualHybrid,
            "MambaRawFeatureResidual",
        );
        let receipt = receipt_fixture(
            &participant,
            MomentumRepresentationQualificationStatusV3::QualifiedRawFallbackNotMamba,
            None,
        );
        assert!(validate_receipt_v3(&receipt).is_err());
    }

    #[test]
    fn raw_only_residual_cannot_count_as_mamba_success() {
        let family = family_fixture(false, true);
        assert_eq!(
            family.qualified_mamba_only_count + family.qualified_mamba_hybrid_count,
            0
        );
        assert_eq!(family.qualified_raw_fallback_count, 1);
    }

    #[test]
    fn mamba_block_ablation_is_deterministic() {
        let participant = participant_fixture(
            3,
            MomentumRepresentationParticipantRoleV3::MambaResidualHybrid,
            "MambaRawFeatureResidual",
        );
        let head = LogisticPredictionHeadV0 {
            weights: vec![0.2, 0.3],
            bias: 0.1,
        };
        let rows = vec![EncodedTrainingExampleV0 {
            representation: vec![1.0, 2.0],
            label: 1.0,
            snapshot_ids: vec![],
        }];
        let full = vec![head.probability(&rows[0].representation).unwrap()];
        assert_eq!(
            residual_contribution_v3(&participant, &head, &rows, 1, &full).unwrap(),
            residual_contribution_v3(&participant, &head, &rows, 1, &full).unwrap()
        );
    }

    #[test]
    fn rejected_routes_remain_in_family() {
        let family = family_fixture(false, false);
        assert_eq!(family.participants.len(), 6);
        assert_eq!(family.qualification_receipts.len(), 6);
    }

    #[test]
    fn every_qualified_genuine_mamba_route_enters_roster() {
        let family = family_fixture(true, true);
        let decision = decision_fixture(&family);
        let roster = derive_roster_v3(&family, &decision).unwrap().0.unwrap();
        assert_eq!(
            roster.qualified_genuine_mamba_digests,
            decision.qualified_mamba_only_digests
        );
    }

    #[test]
    fn raw_fallback_remains_outside_mamba_roster() {
        let family = family_fixture(true, true);
        let decision = decision_fixture(&family);
        let roster = derive_roster_v3(&family, &decision).unwrap().0.unwrap();
        assert!(
            decision
                .raw_fallback_digests
                .iter()
                .all(|digest| !roster.qualified_genuine_mamba_digests.contains(digest))
        );
    }

    #[test]
    fn baselines_only_registration_rejects() {
        let family = family_fixture(false, false);
        let decision = decision_fixture(&family);
        let (roster, status) = derive_roster_v3(&family, &decision).unwrap();
        assert!(roster.is_none());
        assert_eq!(
            status,
            MomentumRepresentationRosterStatusV3::FrozenMambaRepresentationPathRejected
        );
    }

    #[test]
    fn no_private_metric_ranking_selects_winner() {
        let family = family_fixture(true, true);
        assert!(!family.winner_selected);
        let original_decision = decision_fixture(&family);
        let mut changed = family.clone();
        changed.qualification_receipts[0].private_metric_digest =
            "different-private-metric".to_string();
        changed.qualification_receipts[0].receipt_digest =
            receipt_digest_v3(&changed.qualification_receipts[0]);
        changed.family_digest = family_digest_v3(&changed);
        let changed_decision = decision_fixture(&changed);
        assert_eq!(changed_decision.decision, original_decision.decision);
        assert_eq!(
            changed_decision.qualified_mamba_only_digests,
            original_decision.qualified_mamba_only_digests
        );
        assert!(!changed.winner_selected);
    }

    #[test]
    fn route_decision_follows_declared_rules() {
        let family = family_fixture(false, true);
        let decision = decision_fixture(&family);
        assert_eq!(
            decision.decision,
            MomentumRepresentationRouteDecisionV3::RawFeatureFallbackOnly
        );
        assert!(validate_decision_v3(&decision, &family).is_ok());
    }

    #[test]
    fn total_failure_makes_frozen_path_terminal() {
        let family = family_fixture(false, false);
        let decision = decision_fixture(&family);
        assert_eq!(
            decision.decision,
            MomentumRepresentationRouteDecisionV3::AllRepresentationRoutesCollapsed
        );
        assert!(decision.further_frozen_representation_sweep_forbidden);
    }

    #[test]
    fn future_registration_preserves_all_exclusions() {
        let family = family_fixture(true, true);
        let decision = decision_fixture(&family);
        let roster = derive_roster_v3(&family, &decision).unwrap().0.unwrap();
        let evaluation = evaluation_fixture(&family, &decision, &roster);
        assert!(validate_evaluation_v3(&evaluation, &family, &decision, &roster).is_ok());
    }

    #[test]
    fn historical_test_access_remains_zero() {
        let family = family_fixture(false, false);
        assert!(!family.historical_test_accessed);
        assert!(
            family
                .qualification_receipts
                .iter()
                .all(|receipt| receipt.historical_test_reads == 0)
        );
    }

    #[test]
    fn reward_replay_has_no_application_authority() {
        let counters = zero_safety_counters_v3();
        assert_eq!(
            counters.reward_applications + counters.penalty_applications,
            0
        );
    }

    #[test]
    fn protobuf_corruption_rejects() {
        assert!(decode_momentum_representation_audit_protobuf_v3(&[0xff]).is_err());
        assert!(decode_momentum_representation_split_protobuf_v3(&[0xff]).is_err());
        assert!(decode_momentum_representation_registration_protobuf_v3(&[0xff]).is_err());
    }

    #[test]
    fn repeated_execution_sidecars_are_idempotent() {
        let root = temporary_root();
        let first = persist_preregistration_v3(
            &root,
            &audit_fixture(),
            &split_fixture(),
            &registration_fixture(),
        )
        .unwrap();
        let second = persist_preregistration_v3(
            &root,
            &audit_fixture(),
            &split_fixture(),
            &registration_fixture(),
        )
        .unwrap();
        assert_eq!(first, (7, 0));
        assert_eq!(second, (0, 7));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn all_manual_protobuf_contracts_round_trip() {
        let audit = audit_fixture();
        let split = split_fixture();
        let registration = registration_fixture();
        let family = family_fixture(true, true);
        let decision = decision_fixture(&family);
        let roster = derive_roster_v3(&family, &decision).unwrap().0.unwrap();
        let evaluation = evaluation_fixture(&family, &decision, &roster);
        let mut journal = journal_fixture(&family, &decision);
        journal.roster_digest = Some(roster.roster_digest.clone());
        journal.evaluation_registration_digest = Some(evaluation.registration_digest.clone());
        journal.journal_digest = journal_digest_v3(&journal);
        assert_eq!(
            decode_momentum_representation_audit_protobuf_v3(
                &encode_momentum_representation_audit_protobuf_v3(&audit).unwrap()
            )
            .unwrap(),
            audit
        );
        assert_eq!(
            decode_momentum_representation_split_protobuf_v3(
                &encode_momentum_representation_split_protobuf_v3(&split).unwrap()
            )
            .unwrap(),
            split
        );
        assert_eq!(
            decode_momentum_representation_registration_protobuf_v3(
                &encode_momentum_representation_registration_protobuf_v3(&registration).unwrap()
            )
            .unwrap(),
            registration
        );
        assert_eq!(
            decode_frozen_candidate_participant_protobuf_v3(
                &encode_frozen_candidate_participant_protobuf_v3(&family.participants[0]).unwrap()
            )
            .unwrap(),
            family.participants[0]
        );
        assert_eq!(
            decode_momentum_representation_qualification_protobuf_v3(
                &encode_momentum_representation_qualification_protobuf_v3(
                    &family.qualification_receipts[0]
                )
                .unwrap()
            )
            .unwrap(),
            family.qualification_receipts[0]
        );
        assert_eq!(
            decode_mamba_contribution_audit_protobuf_v3(
                &encode_mamba_contribution_audit_protobuf_v3(&family.contribution_audits[0])
                    .unwrap()
            )
            .unwrap(),
            family.contribution_audits[0]
        );
        assert_eq!(
            decode_momentum_representation_family_protobuf_v3(
                &encode_momentum_representation_family_protobuf_v3(&family).unwrap()
            )
            .unwrap(),
            family
        );
        assert_eq!(
            decode_momentum_representation_decision_protobuf_v3(
                &encode_momentum_representation_decision_protobuf_v3(&decision, &family).unwrap(),
                &family
            )
            .unwrap(),
            decision
        );
        assert_eq!(
            decode_momentum_representation_roster_protobuf_v3(
                &encode_momentum_representation_roster_protobuf_v3(&roster, &family, &decision)
                    .unwrap(),
                &family,
                &decision
            )
            .unwrap(),
            roster
        );
        assert_eq!(
            decode_momentum_representation_evaluation_protobuf_v3(
                &encode_momentum_representation_evaluation_protobuf_v3(
                    &evaluation,
                    &family,
                    &decision,
                    &roster
                )
                .unwrap(),
                &family,
                &decision,
                &roster
            )
            .unwrap(),
            evaluation
        );
        assert_eq!(
            decode_momentum_representation_journal_protobuf_v3(
                &encode_momentum_representation_journal_protobuf_v3(&journal).unwrap()
            )
            .unwrap(),
            journal
        );
        assert_eq!(
            decode_momentum_representation_probe_protobuf_v3(
                &encode_momentum_representation_probe_protobuf_v3(&audit.probes[0]).unwrap()
            )
            .unwrap(),
            audit.probes[0]
        );
    }

    #[test]
    fn network_and_authority_counters_are_zero() {
        let counters = zero_safety_counters_v3();
        assert_eq!(counters.active_committee_count, 3);
        assert_eq!(
            counters.network_requests
                + counters.transport_constructions
                + counters.credential_reads
                + counters.prospective_row_reads
                + counters.prospective_label_openings
                + counters.historical_test_reads
                + counters.future_evaluation_reads
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
    fn fresh_validation_uses_purge_context_and_exact_label_range() {
        let examples = (239..265)
            .map(|label_index| SequenceExampleV0 {
                sequence_start: label_index - 8,
                sequence_end: label_index - 1,
                label_index,
                input: vec![],
                label: 0.0,
                snapshot_ids: vec![],
            })
            .collect::<Vec<_>>();
        let selected = examples_with_labels_in_range_v3(
            &examples,
            &IndexRangeV0 {
                start: 240,
                end: 264,
            },
        );
        assert_eq!(selected.len(), 24);
        assert_eq!(selected.first().unwrap().sequence_start, 232);
        assert_eq!(selected.first().unwrap().label_index, 240);
        assert_eq!(selected.last().unwrap().label_index, 263);
    }
}
