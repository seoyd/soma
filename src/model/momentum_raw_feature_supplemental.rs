//! Offline one-time supplemental qualification for the frozen Momentum V4 family.
//!
//! This module can consume only the persisted V4 final reserve. It has no
//! network, active-model, reward, voting, Chair, promotion, or execution
//! authority.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{
    core::stable_hash_string,
    data::{AcquisitionMarketScope, DataSnapshot, DatasetKind},
    league::canonical_current_agent_states,
};

use super::agent_learning_session::{
    AgentPrivateLearningArtifactWriteStatusV0, atomic_write_verified_v0,
};
use super::momentum_raw_feature_v4::{
    InteractionContributionStatusV4, MomentumAccumulatedReplayEvaluationV4, MomentumFrozenReplayV4,
    MomentumRawFeatureFamilyV4, MomentumRawFeaturePathDecisionArtifactV4,
    MomentumRawFeatureQualificationStatusV4, MomentumRawFeatureRegistrationV4,
    MomentumRawFeatureRoleV4, MomentumRawFeatureSplitV4, MomentumValidationYieldAuditV4,
    decode_momentum_frozen_mamba_closure_protobuf_v4,
    decode_momentum_raw_feature_decision_protobuf_v4,
    decode_momentum_raw_feature_family_protobuf_v4,
    decode_momentum_raw_feature_registration_protobuf_v4,
    decode_momentum_raw_feature_split_protobuf_v4,
    decode_momentum_validation_yield_audit_protobuf_v4, evaluate_frozen_momentum_v4_accumulated,
    reconstruct_frozen_momentum_v4,
};
use super::{AgentPrivateLearningRunModeV0, IndexRangeV0, ProtectedEvaluationReservationV1};

const AGENT_ID_V4_1: &str = "momentum_trend_fast";
const ROOT_VERSION_V4_1: &str = "v4_1";
const REGISTRATION_VERSION_V4_1: &str = "momentum-supplemental-registration-v4.1";
const OPENING_RECEIPT_VERSION_V4_1: &str = "momentum-reserve-opening-receipt-v4.1";
const RECEIPT_VERSION_V4_1: &str = "momentum-accumulated-qualification-receipt-v4.1";
const FAMILY_VERSION_V4_1: &str = "momentum-accumulated-qualification-family-v4.1";
const DECISION_VERSION_V4_1: &str = "momentum-accumulated-path-decision-v4.1";
const ROSTER_VERSION_V4_1: &str = "momentum-accumulated-future-roster-v4.1";
const EVALUATION_VERSION_V4_1: &str = "momentum-accumulated-evaluation-registration-v4.1";
const REQUIREMENT_VERSION_V4_1: &str = "momentum-additional-evidence-requirement-v4.1";
const JOURNAL_VERSION_V4_1: &str = "momentum-supplemental-journal-v4.1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumSupplementalQualificationRegistrationV4_1 {
    pub registration_version: String,
    pub agent_id: String,
    pub source_snapshot_digest: String,
    pub canonical_intent_digest: String,
    pub canonical_view_digest: String,
    pub v4_split_digest: String,
    pub v4_registration_digest: String,
    pub v4_family_digest: String,
    pub validation_yield_audit_digest: String,
    pub participant_digests: Vec<String>,
    pub participant_parameter_digests: Vec<String>,
    pub participant_normalizer_digests: Vec<String>,
    pub original_validation_range: IndexRangeV0,
    pub supplemental_validation_range: IndexRangeV0,
    pub accumulated_validation_range_digest: String,
    pub minimum_required_valid_samples: usize,
    pub model_retraining_forbidden: bool,
    pub parameter_updates_forbidden: bool,
    pub configuration_changes_forbidden: bool,
    pub final_reserve_opening_one_time_only: bool,
    pub historical_test_forbidden: bool,
    pub future_evaluation_forbidden: bool,
    pub winner_selection_forbidden: bool,
    pub promotion_forbidden: bool,
    pub reward_application_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumReserveOpeningStatusV4_1 {
    Ready,
    Opened,
    AlreadyOpened,
    RegistrationMismatch,
    ParticipantIdentityMismatch,
    ReserveIdentityMismatch,
    IntegrityFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumReserveOpeningReceiptV4_1 {
    pub receipt_version: String,
    pub supplemental_registration_digest: String,
    pub source_snapshot_digest: String,
    pub v4_final_reserve_digest: String,
    pub opening_attempt_count: usize,
    pub opened_index_count: usize,
    pub participant_identity_set_digest: String,
    pub status: MomentumReserveOpeningStatusV4_1,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumSupplementalValidationYieldV4_1 {
    pub original_valid_sample_count: usize,
    pub supplemental_valid_sample_count: usize,
    pub accumulated_valid_sample_count: usize,
    pub original_neutral_excluded_count: usize,
    pub supplemental_neutral_excluded_count: usize,
    pub minimum_required_valid_samples: usize,
    pub minimum_reached: bool,
    pub yield_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumAccumulatedQualificationStatusV4_1 {
    QualifiedLearned,
    QualifiedLinearEquivalent,
    BenchmarkQualified,
    StillInsufficientValidation,
    RejectedProbabilityCollapse,
    RejectedNumericalFailure,
    RejectedFeatureIntegrity,
    RejectedPolicyInvariant,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumAccumulatedQualificationReceiptV4_1 {
    pub receipt_version: String,
    pub participant_digest: String,
    pub v4_original_receipt_digest: String,
    pub supplemental_registration_digest: String,
    pub reserve_opening_receipt_digest: String,
    pub accumulated_yield_digest: String,
    pub accumulated_validation_identity_digest: String,
    pub qualification_policy_digest: String,
    pub private_metric_digest: String,
    pub status: MomentumAccumulatedQualificationStatusV4_1,
    pub parameter_updates_after_freeze: usize,
    pub historical_test_reads: usize,
    pub future_evaluation_reads: usize,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumAccumulatedInteractionContributionAuditV4_1 {
    pub participant_digest: String,
    pub original_contribution_audit_digest: String,
    pub accumulated_validation_identity_digest: String,
    pub full_prediction_digest: String,
    pub nonlinear_ablated_prediction_digest: String,
    pub contribution_policy_digest: String,
    pub contribution_status: InteractionContributionStatusV4,
    pub audit_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumAccumulatedQualificationFamilyV4_1 {
    pub family_version: String,
    pub source_v4_family_digest: String,
    pub supplemental_registration_digest: String,
    pub reserve_opening_receipt_digest: String,
    pub accumulated_yield_digest: String,
    pub participant_digests: Vec<String>,
    pub accumulated_receipts: Vec<MomentumAccumulatedQualificationReceiptV4_1>,
    pub accumulated_interaction_audit_digest: Option<String>,
    pub qualified_learned_count: usize,
    pub qualified_benchmark_count: usize,
    pub winner_selected: bool,
    pub parameters_changed: bool,
    pub eligible_for_active_committee: bool,
    pub eligible_for_promotion: bool,
    pub eligible_for_reward: bool,
    pub family_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumAccumulatedPathDecisionV4_1 {
    RawFeatureLearnedPathViable,
    OnlyLinearRawPathViable,
    StillInsufficientValidation,
    NoQualifiedRawFeatureLearner,
    TechnicalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumAccumulatedPathDecisionArtifactV4_1 {
    pub decision_version: String,
    pub accumulated_family_digest: String,
    pub minimum_reached: bool,
    pub qualified_raw_logistic: bool,
    pub qualified_material_interaction: bool,
    pub decision: MomentumAccumulatedPathDecisionV4_1,
    pub decision_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumAccumulatedRosterStatusV4_1 {
    Ready,
    StillInsufficientValidation,
    NoQualifiedLearnedParticipant,
    BenchmarkUnavailable,
    SemanticDuplicateOnly,
    IntegrityFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumAccumulatedFutureRosterV4_1 {
    pub roster_version: String,
    pub accumulated_family_digest: String,
    pub learned_participant_digests: Vec<String>,
    pub benchmark_participant_digests: Vec<String>,
    pub excluded_semantic_duplicate_digests: Vec<String>,
    pub rejected_participant_digests: Vec<String>,
    pub inclusion_policy_digest: String,
    pub status: MomentumAccumulatedRosterStatusV4_1,
    pub roster_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumAccumulatedEvaluationStatusV4_1 {
    Registered,
    StillInsufficientValidation,
    NoQualifiedLearnedParticipant,
    BenchmarkUnavailable,
    SemanticDuplicateOnly,
    SafetyContractInvalid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumAccumulatedEvaluationRegistrationV4_1 {
    pub registration_version: String,
    pub agent_id: String,
    pub source_v4_family_digest: String,
    pub accumulated_family_digest: String,
    pub roster_digest: String,
    pub supplemental_registration_digest: String,
    pub reserve_opening_receipt_digest: String,
    pub accumulated_yield_digest: String,
    pub accumulated_receipt_digests: Vec<String>,
    pub accumulated_interaction_audit_digest: Option<String>,
    pub source_snapshot_digest: String,
    pub source_boundary_timestamp_ms: u64,
    pub consumed_validation_identity_digests: Vec<String>,
    pub protected_registration_digests: Vec<String>,
    pub protected_timestamp_ms: Vec<u64>,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumAdditionalEvidenceRequirementV4_1 {
    pub requirement_version: String,
    pub accumulated_valid_sample_count: usize,
    pub minimum_required_valid_samples: usize,
    pub minimum_additional_valid_samples: usize,
    pub current_source_boundary_timestamp_ms: u64,
    pub required_dataset_kind: DatasetKind,
    pub market_scope: AcquisitionMarketScope,
    pub symbols: Vec<String>,
    pub cadence: String,
    pub existing_evidence_fully_consumed_for_qualification: bool,
    pub new_evidence_identity_required: bool,
    pub separate_acquisition_preregistration_required: bool,
    pub requirement_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumSupplementalExecutionStatusV4_1 {
    Planned,
    Executed,
    AlreadyOpened,
    TechnicalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumSupplementalJournalV4_1 {
    pub journal_version: String,
    pub agent_id: String,
    pub supplemental_registration_digest: String,
    pub ready_opening_receipt_digest: String,
    pub opened_receipt_digest: String,
    pub accumulated_yield_digest: String,
    pub accumulated_family_digest: String,
    pub accumulated_decision_digest: String,
    pub roster_digest: Option<String>,
    pub evaluation_registration_digest: Option<String>,
    pub additional_evidence_requirement_digest: Option<String>,
    pub registration_reopened_before_reserve_access: bool,
    pub frozen_participants_reconstructed: bool,
    pub participant_parameters_unchanged: bool,
    pub normalizers_unchanged: bool,
    pub status: MomentumSupplementalExecutionStatusV4_1,
    pub journal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumSupplementalSafetyCountersV4_1 {
    pub network_requests: usize,
    pub transport_constructions: usize,
    pub credential_reads: usize,
    pub new_prospective_row_reads: usize,
    pub new_prospective_label_openings: usize,
    pub historical_test_reads: usize,
    pub future_evaluation_reads: usize,
    pub reserve_opening_attempts: usize,
    pub reserve_row_reads: usize,
    pub reserve_label_reads: usize,
    pub participant_parameter_changes: usize,
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
pub struct MomentumSupplementalReportV4_1 {
    pub report_version: String,
    pub mode: AgentPrivateLearningRunModeV0,
    pub status: MomentumSupplementalExecutionStatusV4_1,
    pub original_validation_yield_audit: Option<MomentumValidationYieldAuditV4>,
    pub corrected_v4_decision: Option<MomentumRawFeaturePathDecisionArtifactV4>,
    pub supplemental_registration: Option<MomentumSupplementalQualificationRegistrationV4_1>,
    pub reserve_opening_status: MomentumReserveOpeningStatusV4_1,
    pub reserve_opening_receipt: Option<MomentumReserveOpeningReceiptV4_1>,
    pub supplemental_yield: Option<MomentumSupplementalValidationYieldV4_1>,
    pub accumulated_interaction_audit: Option<MomentumAccumulatedInteractionContributionAuditV4_1>,
    pub accumulated_family: Option<MomentumAccumulatedQualificationFamilyV4_1>,
    pub accumulated_decision: Option<MomentumAccumulatedPathDecisionArtifactV4_1>,
    pub roster: Option<MomentumAccumulatedFutureRosterV4_1>,
    pub roster_status: MomentumAccumulatedRosterStatusV4_1,
    pub evaluation_registration: Option<MomentumAccumulatedEvaluationRegistrationV4_1>,
    pub evaluation_registration_status: MomentumAccumulatedEvaluationStatusV4_1,
    pub additional_evidence_requirement: Option<MomentumAdditionalEvidenceRequirementV4_1>,
    pub journal: Option<MomentumSupplementalJournalV4_1>,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub storage_failure_count: usize,
    pub protected_artifacts_unchanged: bool,
    pub active_state_unchanged: bool,
    pub safety_counters: MomentumSupplementalSafetyCountersV4_1,
    pub report_digest: String,
}

#[derive(Clone, Debug)]
struct FrozenV4Artifacts {
    closure: super::momentum_raw_feature_v4::MomentumFrozenMambaPathClosureV4,
    split: MomentumRawFeatureSplitV4,
    registration: MomentumRawFeatureRegistrationV4,
    validation_yield_audit: MomentumValidationYieldAuditV4,
    family: MomentumRawFeatureFamilyV4,
    decision: MomentumRawFeaturePathDecisionArtifactV4,
}

#[derive(Clone, Debug)]
struct SupplementalResultV4_1 {
    registration: MomentumSupplementalQualificationRegistrationV4_1,
    ready_receipt: MomentumReserveOpeningReceiptV4_1,
    opened_receipt: MomentumReserveOpeningReceiptV4_1,
    yield_result: MomentumSupplementalValidationYieldV4_1,
    interaction_audit: MomentumAccumulatedInteractionContributionAuditV4_1,
    family: MomentumAccumulatedQualificationFamilyV4_1,
    decision: MomentumAccumulatedPathDecisionArtifactV4_1,
    roster: Option<MomentumAccumulatedFutureRosterV4_1>,
    roster_status: MomentumAccumulatedRosterStatusV4_1,
    evaluation: Option<MomentumAccumulatedEvaluationRegistrationV4_1>,
    evaluation_status: MomentumAccumulatedEvaluationStatusV4_1,
    additional_requirement: Option<MomentumAdditionalEvidenceRequirementV4_1>,
    journal: MomentumSupplementalJournalV4_1,
}

#[derive(Clone, Debug)]
pub(crate) struct MomentumFutureEvaluationSourceV4_2 {
    pub(crate) closure: super::momentum_raw_feature_v4::MomentumFrozenMambaPathClosureV4,
    pub(crate) split: MomentumRawFeatureSplitV4,
    pub(crate) registration: MomentumRawFeatureRegistrationV4,
    pub(crate) source_family: MomentumRawFeatureFamilyV4,
    pub(crate) supplemental_registration: MomentumSupplementalQualificationRegistrationV4_1,
    pub(crate) accumulated_family: MomentumAccumulatedQualificationFamilyV4_1,
    pub(crate) roster: MomentumAccumulatedFutureRosterV4_1,
    pub(crate) evaluation: MomentumAccumulatedEvaluationRegistrationV4_1,
}

fn canonical_digest<T: Clone + std::fmt::Debug>(value: &T, clear: impl FnOnce(&mut T)) -> String {
    let mut canonical = value.clone();
    clear(&mut canonical);
    stable_hash_string(&format!("{canonical:?}"))
}

fn registration_digest(value: &MomentumSupplementalQualificationRegistrationV4_1) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}
fn opening_digest(value: &MomentumReserveOpeningReceiptV4_1) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}
fn yield_digest(value: &MomentumSupplementalValidationYieldV4_1) -> String {
    canonical_digest(value, |item| item.yield_digest.clear())
}
fn receipt_digest(value: &MomentumAccumulatedQualificationReceiptV4_1) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}
fn contribution_digest(value: &MomentumAccumulatedInteractionContributionAuditV4_1) -> String {
    canonical_digest(value, |item| item.audit_digest.clear())
}
fn family_digest(value: &MomentumAccumulatedQualificationFamilyV4_1) -> String {
    canonical_digest(value, |item| item.family_digest.clear())
}
fn decision_digest(value: &MomentumAccumulatedPathDecisionArtifactV4_1) -> String {
    canonical_digest(value, |item| item.decision_digest.clear())
}
fn roster_digest(value: &MomentumAccumulatedFutureRosterV4_1) -> String {
    canonical_digest(value, |item| item.roster_digest.clear())
}
fn evaluation_digest(value: &MomentumAccumulatedEvaluationRegistrationV4_1) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}
fn requirement_digest(value: &MomentumAdditionalEvidenceRequirementV4_1) -> String {
    canonical_digest(value, |item| item.requirement_digest.clear())
}
fn journal_digest(value: &MomentumSupplementalJournalV4_1) -> String {
    canonical_digest(value, |item| item.journal_digest.clear())
}
fn report_digest(value: &MomentumSupplementalReportV4_1) -> String {
    canonical_digest(value, |item| item.report_digest.clear())
}

fn range_digest(label: &str, value: &IndexRangeV0) -> String {
    stable_hash_string(&format!("{label}:{}:{}", value.start, value.end))
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn participant_set_digest(values: &[String]) -> String {
    stable_hash_string(&format!(
        "v4.1-participant-set:{:?}",
        sorted_unique(values.to_vec())
    ))
}

fn zero_counters() -> MomentumSupplementalSafetyCountersV4_1 {
    MomentumSupplementalSafetyCountersV4_1 {
        network_requests: 0,
        transport_constructions: 0,
        credential_reads: 0,
        new_prospective_row_reads: 0,
        new_prospective_label_openings: 0,
        historical_test_reads: 0,
        future_evaluation_reads: 0,
        reserve_opening_attempts: 0,
        reserve_row_reads: 0,
        reserve_label_reads: 0,
        participant_parameter_changes: 0,
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

fn validate_registration(
    value: &MomentumSupplementalQualificationRegistrationV4_1,
) -> Result<(), String> {
    let original_count = value
        .original_validation_range
        .end
        .checked_sub(value.original_validation_range.start)
        .ok_or_else(|| "V4.1 original validation range rejected".to_string())?;
    let supplemental_count = value
        .supplemental_validation_range
        .end
        .checked_sub(value.supplemental_validation_range.start)
        .ok_or_else(|| "V4.1 supplemental validation range rejected".to_string())?;
    if value.registration_version != REGISTRATION_VERSION_V4_1
        || value.agent_id != AGENT_ID_V4_1
        || value.participant_digests.len() != 3
        || value.participant_parameter_digests.len() != 3
        || value.participant_normalizer_digests.len() != 3
        || value
            .participant_digests
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != 3
        || original_count == 0
        || supplemental_count == 0
        || value.original_validation_range.end != value.supplemental_validation_range.start
        || value.minimum_required_valid_samples == 0
        || !value.model_retraining_forbidden
        || !value.parameter_updates_forbidden
        || !value.configuration_changes_forbidden
        || !value.final_reserve_opening_one_time_only
        || !value.historical_test_forbidden
        || !value.future_evaluation_forbidden
        || !value.winner_selection_forbidden
        || !value.promotion_forbidden
        || !value.reward_application_forbidden
        || value.registration_digest != registration_digest(value)
    {
        return Err("V4.1 supplemental registration rejected".to_string());
    }
    Ok(())
}

fn validate_opening(value: &MomentumReserveOpeningReceiptV4_1) -> Result<(), String> {
    let counts_valid = match value.status {
        MomentumReserveOpeningStatusV4_1::Ready => {
            value.opening_attempt_count == 1 && value.opened_index_count == 0
        }
        MomentumReserveOpeningStatusV4_1::Opened => {
            value.opening_attempt_count == 1 && value.opened_index_count > 0
        }
        _ => false,
    };
    if value.receipt_version != OPENING_RECEIPT_VERSION_V4_1
        || value.supplemental_registration_digest.is_empty()
        || value.source_snapshot_digest.is_empty()
        || value.v4_final_reserve_digest.is_empty()
        || value.participant_identity_set_digest.is_empty()
        || !counts_valid
        || value.receipt_digest != opening_digest(value)
    {
        return Err("V4.1 reserve opening receipt rejected".to_string());
    }
    Ok(())
}

fn validate_yield(value: &MomentumSupplementalValidationYieldV4_1) -> Result<(), String> {
    let accumulated = value
        .original_valid_sample_count
        .checked_add(value.supplemental_valid_sample_count)
        .ok_or_else(|| "V4.1 accumulated yield overflow".to_string())?;
    if value.accumulated_valid_sample_count != accumulated
        || value.minimum_required_valid_samples == 0
        || value.minimum_reached != (accumulated >= value.minimum_required_valid_samples)
        || value.yield_digest != yield_digest(value)
    {
        return Err("V4.1 accumulated yield rejected".to_string());
    }
    Ok(())
}

fn map_status(
    status: MomentumRawFeatureQualificationStatusV4,
) -> MomentumAccumulatedQualificationStatusV4_1 {
    match status {
        MomentumRawFeatureQualificationStatusV4::QualifiedLearned => {
            MomentumAccumulatedQualificationStatusV4_1::QualifiedLearned
        }
        MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent => {
            MomentumAccumulatedQualificationStatusV4_1::QualifiedLinearEquivalent
        }
        MomentumRawFeatureQualificationStatusV4::BenchmarkQualified => {
            MomentumAccumulatedQualificationStatusV4_1::BenchmarkQualified
        }
        MomentumRawFeatureQualificationStatusV4::RejectedInsufficientValidation => {
            MomentumAccumulatedQualificationStatusV4_1::StillInsufficientValidation
        }
        MomentumRawFeatureQualificationStatusV4::RejectedProbabilityCollapse => {
            MomentumAccumulatedQualificationStatusV4_1::RejectedProbabilityCollapse
        }
        MomentumRawFeatureQualificationStatusV4::RejectedNumericalFailure => {
            MomentumAccumulatedQualificationStatusV4_1::RejectedNumericalFailure
        }
        MomentumRawFeatureQualificationStatusV4::RejectedFeatureIntegrity => {
            MomentumAccumulatedQualificationStatusV4_1::RejectedFeatureIntegrity
        }
        MomentumRawFeatureQualificationStatusV4::RejectedPolicyInvariant => {
            MomentumAccumulatedQualificationStatusV4_1::RejectedPolicyInvariant
        }
    }
}

fn derive_registration(
    frozen: &FrozenV4Artifacts,
) -> Result<MomentumSupplementalQualificationRegistrationV4_1, String> {
    if frozen.decision.decision
        != super::momentum_raw_feature_v4::MomentumRawFeaturePathDecisionV4::InsufficientFreshValidation
        || frozen.validation_yield_audit.substantive_qualification_possible
        || frozen.split.fresh_validation_range.end != frozen.split.final_untouched_range.start
    {
        return Err("V4.1 supplemental eligibility rejected".to_string());
    }
    let mut value = MomentumSupplementalQualificationRegistrationV4_1 {
        registration_version: REGISTRATION_VERSION_V4_1.to_string(),
        agent_id: AGENT_ID_V4_1.to_string(),
        source_snapshot_digest: frozen.family.source_snapshot_digest.clone(),
        canonical_intent_digest: frozen.closure.canonical_intent_digest.clone(),
        canonical_view_digest: frozen.closure.canonical_view_digest.clone(),
        v4_split_digest: frozen.split.split_digest.clone(),
        v4_registration_digest: frozen.registration.registration_digest.clone(),
        v4_family_digest: frozen.family.family_digest.clone(),
        validation_yield_audit_digest: frozen.validation_yield_audit.audit_digest.clone(),
        participant_digests: frozen
            .family
            .participants
            .iter()
            .map(|item| item.participant_digest.clone())
            .collect(),
        participant_parameter_digests: frozen
            .family
            .participants
            .iter()
            .map(|item| item.parameter_digest.clone())
            .collect(),
        participant_normalizer_digests: frozen
            .family
            .participants
            .iter()
            .map(|item| item.normalizer_digest.clone())
            .collect(),
        original_validation_range: frozen.split.fresh_validation_range.clone(),
        supplemental_validation_range: frozen.split.final_untouched_range.clone(),
        accumulated_validation_range_digest: stable_hash_string(&format!(
            "v4.1-accumulated-ranges:{}:{}",
            range_digest("original", &frozen.split.fresh_validation_range),
            range_digest("supplemental", &frozen.split.final_untouched_range)
        )),
        minimum_required_valid_samples: frozen.split.minimum_validation_samples,
        model_retraining_forbidden: true,
        parameter_updates_forbidden: true,
        configuration_changes_forbidden: true,
        final_reserve_opening_one_time_only: true,
        historical_test_forbidden: true,
        future_evaluation_forbidden: true,
        winner_selection_forbidden: true,
        promotion_forbidden: true,
        reward_application_forbidden: true,
        registration_digest: String::new(),
    };
    value.registration_digest = registration_digest(&value);
    validate_registration(&value)?;
    Ok(value)
}

fn opening_receipt(
    registration: &MomentumSupplementalQualificationRegistrationV4_1,
    status: MomentumReserveOpeningStatusV4_1,
    opened_index_count: usize,
) -> Result<MomentumReserveOpeningReceiptV4_1, String> {
    let mut value = MomentumReserveOpeningReceiptV4_1 {
        receipt_version: OPENING_RECEIPT_VERSION_V4_1.to_string(),
        supplemental_registration_digest: registration.registration_digest.clone(),
        source_snapshot_digest: registration.source_snapshot_digest.clone(),
        v4_final_reserve_digest: range_digest(
            "v4-final-reserve",
            &registration.supplemental_validation_range,
        ),
        opening_attempt_count: 1,
        opened_index_count,
        participant_identity_set_digest: participant_set_digest(&registration.participant_digests),
        status,
        receipt_digest: String::new(),
    };
    value.receipt_digest = opening_digest(&value);
    validate_opening(&value)?;
    Ok(value)
}

fn validate_receipt(value: &MomentumAccumulatedQualificationReceiptV4_1) -> Result<(), String> {
    if value.receipt_version != RECEIPT_VERSION_V4_1
        || value.participant_digest.is_empty()
        || value.v4_original_receipt_digest.is_empty()
        || value.supplemental_registration_digest.is_empty()
        || value.reserve_opening_receipt_digest.is_empty()
        || value.accumulated_yield_digest.is_empty()
        || value.accumulated_validation_identity_digest.is_empty()
        || value.qualification_policy_digest.is_empty()
        || value.private_metric_digest.is_empty()
        || value.parameter_updates_after_freeze != 0
        || value.historical_test_reads != 0
        || value.future_evaluation_reads != 0
        || value.receipt_digest != receipt_digest(value)
    {
        return Err("V4.1 accumulated receipt rejected".to_string());
    }
    Ok(())
}

fn validate_contribution(
    value: &MomentumAccumulatedInteractionContributionAuditV4_1,
) -> Result<(), String> {
    if value.participant_digest.is_empty()
        || value.original_contribution_audit_digest.is_empty()
        || value.accumulated_validation_identity_digest.is_empty()
        || value.full_prediction_digest.is_empty()
        || value.nonlinear_ablated_prediction_digest.is_empty()
        || value.contribution_policy_digest.is_empty()
        || value.audit_digest != contribution_digest(value)
    {
        return Err("V4.1 accumulated contribution audit rejected".to_string());
    }
    Ok(())
}

fn validate_family(value: &MomentumAccumulatedQualificationFamilyV4_1) -> Result<(), String> {
    let learned = value
        .accumulated_receipts
        .iter()
        .filter(|item| {
            matches!(
                item.status,
                MomentumAccumulatedQualificationStatusV4_1::QualifiedLearned
                    | MomentumAccumulatedQualificationStatusV4_1::QualifiedLinearEquivalent
            )
        })
        .count();
    let benchmark = value
        .accumulated_receipts
        .iter()
        .filter(|item| {
            item.status == MomentumAccumulatedQualificationStatusV4_1::BenchmarkQualified
        })
        .count();
    if value.family_version != FAMILY_VERSION_V4_1
        || value.participant_digests.len() != 3
        || value.accumulated_receipts.len() != 3
        || value
            .participant_digests
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != 3
        || value.qualified_learned_count != learned
        || value.qualified_benchmark_count != benchmark
        || value.winner_selected
        || value.parameters_changed
        || value.eligible_for_active_committee
        || value.eligible_for_promotion
        || value.eligible_for_reward
        || value.family_digest != family_digest(value)
    {
        return Err("V4.1 accumulated family rejected".to_string());
    }
    for receipt in &value.accumulated_receipts {
        validate_receipt(receipt)?;
    }
    Ok(())
}

fn decision_inputs(
    frozen: &FrozenV4Artifacts,
    family: &MomentumAccumulatedQualificationFamilyV4_1,
    yield_result: &MomentumSupplementalValidationYieldV4_1,
    contribution: &MomentumAccumulatedInteractionContributionAuditV4_1,
) -> (bool, bool, MomentumAccumulatedPathDecisionV4_1) {
    let status_for = |role| {
        frozen
            .family
            .participants
            .iter()
            .find(|item| item.participant_role == role)
            .and_then(|participant| {
                family
                    .accumulated_receipts
                    .iter()
                    .find(|item| item.participant_digest == participant.participant_digest)
            })
            .map(|item| item.status)
    };
    let raw = status_for(MomentumRawFeatureRoleV4::LearnedRawLogistic)
        == Some(MomentumAccumulatedQualificationStatusV4_1::QualifiedLearned);
    let interaction = status_for(MomentumRawFeatureRoleV4::LearnedInteractionLogistic)
        == Some(MomentumAccumulatedQualificationStatusV4_1::QualifiedLearned)
        && contribution.contribution_status
            == InteractionContributionStatusV4::MaterialInteractionContribution;
    let decision = if !yield_result.minimum_reached {
        MomentumAccumulatedPathDecisionV4_1::StillInsufficientValidation
    } else if interaction {
        MomentumAccumulatedPathDecisionV4_1::RawFeatureLearnedPathViable
    } else if raw {
        MomentumAccumulatedPathDecisionV4_1::OnlyLinearRawPathViable
    } else {
        MomentumAccumulatedPathDecisionV4_1::NoQualifiedRawFeatureLearner
    };
    (raw, interaction, decision)
}

fn validate_decision(
    value: &MomentumAccumulatedPathDecisionArtifactV4_1,
    frozen: &FrozenV4Artifacts,
    family: &MomentumAccumulatedQualificationFamilyV4_1,
    yield_result: &MomentumSupplementalValidationYieldV4_1,
    contribution: &MomentumAccumulatedInteractionContributionAuditV4_1,
) -> Result<(), String> {
    let (raw, interaction, decision) = decision_inputs(frozen, family, yield_result, contribution);
    if value.decision_version != DECISION_VERSION_V4_1
        || value.accumulated_family_digest != family.family_digest
        || value.minimum_reached != yield_result.minimum_reached
        || value.qualified_raw_logistic != raw
        || value.qualified_material_interaction != interaction
        || value.decision != decision
        || value.decision_digest != decision_digest(value)
    {
        return Err("V4.1 accumulated decision rejected".to_string());
    }
    Ok(())
}

fn derive_result(
    frozen: &FrozenV4Artifacts,
    replay: &MomentumFrozenReplayV4,
    registration: MomentumSupplementalQualificationRegistrationV4_1,
    evaluation: MomentumAccumulatedReplayEvaluationV4,
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<SupplementalResultV4_1, String> {
    if replay.registration != frozen.registration
        || replay.family != frozen.family
        || replay.decision != frozen.decision
        || replay.validation_yield_audit != frozen.validation_yield_audit
        || !frozen_identity_vectors_match(&registration, &replay.family)
    {
        return Err("V4.1 frozen reconstruction identity rejected".to_string());
    }
    let reserve_count = registration
        .supplemental_validation_range
        .end
        .checked_sub(registration.supplemental_validation_range.start)
        .ok_or_else(|| "V4.1 reserve range rejected".to_string())?;
    let ready_receipt = opening_receipt(&registration, MomentumReserveOpeningStatusV4_1::Ready, 0)?;
    let opened_receipt = opening_receipt(
        &registration,
        MomentumReserveOpeningStatusV4_1::Opened,
        reserve_count,
    )?;
    let mut yield_result = MomentumSupplementalValidationYieldV4_1 {
        original_valid_sample_count: evaluation.original_valid_sample_count,
        supplemental_valid_sample_count: evaluation.supplemental_valid_sample_count,
        accumulated_valid_sample_count: evaluation
            .original_valid_sample_count
            .checked_add(evaluation.supplemental_valid_sample_count)
            .ok_or_else(|| "V4.1 accumulated sample overflow".to_string())?,
        original_neutral_excluded_count: evaluation.original_neutral_excluded_count,
        supplemental_neutral_excluded_count: evaluation.supplemental_neutral_excluded_count,
        minimum_required_valid_samples: registration.minimum_required_valid_samples,
        minimum_reached: false,
        yield_digest: String::new(),
    };
    yield_result.minimum_reached =
        yield_result.accumulated_valid_sample_count >= yield_result.minimum_required_valid_samples;
    yield_result.yield_digest = yield_digest(&yield_result);
    validate_yield(&yield_result)?;
    let mut receipts = Vec::new();
    for item in &evaluation.participant_evaluations {
        let original = frozen
            .family
            .qualification_receipts
            .iter()
            .find(|receipt| receipt.receipt_digest == item.original_receipt_digest)
            .ok_or_else(|| "V4.1 original receipt identity rejected".to_string())?;
        let mut receipt = MomentumAccumulatedQualificationReceiptV4_1 {
            receipt_version: RECEIPT_VERSION_V4_1.to_string(),
            participant_digest: item.participant_digest.clone(),
            v4_original_receipt_digest: item.original_receipt_digest.clone(),
            supplemental_registration_digest: registration.registration_digest.clone(),
            reserve_opening_receipt_digest: opened_receipt.receipt_digest.clone(),
            accumulated_yield_digest: yield_result.yield_digest.clone(),
            accumulated_validation_identity_digest: evaluation
                .accumulated_validation_identity_digest
                .clone(),
            qualification_policy_digest: original.qualification_policy_digest.clone(),
            private_metric_digest: item.private_metric_digest.clone(),
            status: map_status(item.status),
            parameter_updates_after_freeze: 0,
            historical_test_reads: 0,
            future_evaluation_reads: 0,
            receipt_digest: String::new(),
        };
        receipt.receipt_digest = receipt_digest(&receipt);
        validate_receipt(&receipt)?;
        receipts.push(receipt);
    }
    let original_contribution = frozen
        .family
        .interaction_contribution_audit
        .as_ref()
        .ok_or_else(|| "V4.1 original contribution audit missing".to_string())?;
    let mut interaction_audit = MomentumAccumulatedInteractionContributionAuditV4_1 {
        participant_digest: evaluation
            .interaction_contribution
            .participant_digest
            .clone(),
        original_contribution_audit_digest: original_contribution.audit_digest.clone(),
        accumulated_validation_identity_digest: evaluation
            .accumulated_validation_identity_digest
            .clone(),
        full_prediction_digest: evaluation
            .interaction_contribution
            .full_prediction_digest
            .clone(),
        nonlinear_ablated_prediction_digest: evaluation
            .interaction_contribution
            .nonlinear_ablated_prediction_digest
            .clone(),
        contribution_policy_digest: evaluation
            .interaction_contribution
            .contribution_policy_digest
            .clone(),
        contribution_status: evaluation.interaction_contribution.contribution_status,
        audit_digest: String::new(),
    };
    interaction_audit.audit_digest = contribution_digest(&interaction_audit);
    validate_contribution(&interaction_audit)?;
    let mut family = MomentumAccumulatedQualificationFamilyV4_1 {
        family_version: FAMILY_VERSION_V4_1.to_string(),
        source_v4_family_digest: frozen.family.family_digest.clone(),
        supplemental_registration_digest: registration.registration_digest.clone(),
        reserve_opening_receipt_digest: opened_receipt.receipt_digest.clone(),
        accumulated_yield_digest: yield_result.yield_digest.clone(),
        participant_digests: registration.participant_digests.clone(),
        qualified_learned_count: receipts
            .iter()
            .filter(|item| {
                matches!(
                    item.status,
                    MomentumAccumulatedQualificationStatusV4_1::QualifiedLearned
                        | MomentumAccumulatedQualificationStatusV4_1::QualifiedLinearEquivalent
                )
            })
            .count(),
        qualified_benchmark_count: receipts
            .iter()
            .filter(|item| {
                item.status == MomentumAccumulatedQualificationStatusV4_1::BenchmarkQualified
            })
            .count(),
        accumulated_receipts: receipts,
        accumulated_interaction_audit_digest: Some(interaction_audit.audit_digest.clone()),
        winner_selected: false,
        parameters_changed: false,
        eligible_for_active_committee: false,
        eligible_for_promotion: false,
        eligible_for_reward: false,
        family_digest: String::new(),
    };
    family.family_digest = family_digest(&family);
    validate_family(&family)?;
    let (raw, interaction, decision_value) =
        decision_inputs(frozen, &family, &yield_result, &interaction_audit);
    let mut decision = MomentumAccumulatedPathDecisionArtifactV4_1 {
        decision_version: DECISION_VERSION_V4_1.to_string(),
        accumulated_family_digest: family.family_digest.clone(),
        minimum_reached: yield_result.minimum_reached,
        qualified_raw_logistic: raw,
        qualified_material_interaction: interaction,
        decision: decision_value,
        decision_digest: String::new(),
    };
    decision.decision_digest = decision_digest(&decision);
    validate_decision(
        &decision,
        frozen,
        &family,
        &yield_result,
        &interaction_audit,
    )?;
    let (roster, roster_status) =
        derive_roster(frozen, &family, &yield_result, &interaction_audit)?;
    let (future_evaluation, evaluation_status) = derive_evaluation(
        frozen,
        &registration,
        &opened_receipt,
        &yield_result,
        &family,
        &interaction_audit,
        roster.as_ref(),
        roster_status,
        reservation,
        evaluation.source_boundary_timestamp_ms,
        &evaluation.accumulated_validation_identity_digest,
    )?;
    let additional_requirement = if yield_result.minimum_reached {
        None
    } else {
        let mut requirement = MomentumAdditionalEvidenceRequirementV4_1 {
            requirement_version: REQUIREMENT_VERSION_V4_1.to_string(),
            accumulated_valid_sample_count: yield_result.accumulated_valid_sample_count,
            minimum_required_valid_samples: yield_result.minimum_required_valid_samples,
            minimum_additional_valid_samples: yield_result
                .minimum_required_valid_samples
                .saturating_sub(yield_result.accumulated_valid_sample_count),
            current_source_boundary_timestamp_ms: evaluation.source_boundary_timestamp_ms,
            required_dataset_kind: DatasetKind::CryptoDailyOhlcv,
            market_scope: AcquisitionMarketScope::BtcCrypto,
            symbols: vec!["BTC".to_string()],
            cadence: "1d".to_string(),
            existing_evidence_fully_consumed_for_qualification: true,
            new_evidence_identity_required: true,
            separate_acquisition_preregistration_required: true,
            requirement_digest: String::new(),
        };
        requirement.requirement_digest = requirement_digest(&requirement);
        Some(requirement)
    };
    let mut journal = MomentumSupplementalJournalV4_1 {
        journal_version: JOURNAL_VERSION_V4_1.to_string(),
        agent_id: AGENT_ID_V4_1.to_string(),
        supplemental_registration_digest: registration.registration_digest.clone(),
        ready_opening_receipt_digest: ready_receipt.receipt_digest.clone(),
        opened_receipt_digest: opened_receipt.receipt_digest.clone(),
        accumulated_yield_digest: yield_result.yield_digest.clone(),
        accumulated_family_digest: family.family_digest.clone(),
        accumulated_decision_digest: decision.decision_digest.clone(),
        roster_digest: roster.as_ref().map(|item| item.roster_digest.clone()),
        evaluation_registration_digest: future_evaluation
            .as_ref()
            .map(|item| item.registration_digest.clone()),
        additional_evidence_requirement_digest: additional_requirement
            .as_ref()
            .map(|item| item.requirement_digest.clone()),
        registration_reopened_before_reserve_access: true,
        frozen_participants_reconstructed: true,
        participant_parameters_unchanged: true,
        normalizers_unchanged: true,
        status: MomentumSupplementalExecutionStatusV4_1::Executed,
        journal_digest: String::new(),
    };
    journal.journal_digest = journal_digest(&journal);
    Ok(SupplementalResultV4_1 {
        registration,
        ready_receipt,
        opened_receipt,
        yield_result,
        interaction_audit,
        family,
        decision,
        roster,
        roster_status,
        evaluation: future_evaluation,
        evaluation_status,
        additional_requirement,
        journal,
    })
}

fn frozen_identity_vectors_match(
    registration: &MomentumSupplementalQualificationRegistrationV4_1,
    family: &MomentumRawFeatureFamilyV4,
) -> bool {
    let parameter_digests = family
        .participants
        .iter()
        .map(|item| item.parameter_digest.clone())
        .collect::<Vec<_>>();
    let normalizer_digests = family
        .participants
        .iter()
        .map(|item| item.normalizer_digest.clone())
        .collect::<Vec<_>>();
    identity_vectors_match(registration, &parameter_digests, &normalizer_digests)
}

fn identity_vectors_match(
    registration: &MomentumSupplementalQualificationRegistrationV4_1,
    parameter_digests: &[String],
    normalizer_digests: &[String],
) -> bool {
    registration.participant_parameter_digests == parameter_digests
        && registration.participant_normalizer_digests == normalizer_digests
}

fn derive_roster(
    frozen: &FrozenV4Artifacts,
    family: &MomentumAccumulatedQualificationFamilyV4_1,
    yield_result: &MomentumSupplementalValidationYieldV4_1,
    contribution: &MomentumAccumulatedInteractionContributionAuditV4_1,
) -> Result<
    (
        Option<MomentumAccumulatedFutureRosterV4_1>,
        MomentumAccumulatedRosterStatusV4_1,
    ),
    String,
> {
    if !yield_result.minimum_reached {
        return Ok((
            None,
            MomentumAccumulatedRosterStatusV4_1::StillInsufficientValidation,
        ));
    }
    let status_for = |digest: &str| {
        family
            .accumulated_receipts
            .iter()
            .find(|item| item.participant_digest == digest)
            .map(|item| item.status)
    };
    let mut learned = Vec::new();
    let mut benchmarks = Vec::new();
    let mut duplicates = Vec::new();
    let mut rejected = Vec::new();
    for participant in &frozen.family.participants {
        match (
            participant.participant_role,
            status_for(&participant.participant_digest),
        ) {
            (
                MomentumRawFeatureRoleV4::ConstantBenchmark,
                Some(MomentumAccumulatedQualificationStatusV4_1::BenchmarkQualified),
            ) => benchmarks.push(participant.participant_digest.clone()),
            (
                MomentumRawFeatureRoleV4::LearnedInteractionLogistic,
                Some(MomentumAccumulatedQualificationStatusV4_1::QualifiedLearned),
            ) if contribution.contribution_status
                == InteractionContributionStatusV4::MaterialInteractionContribution =>
            {
                learned.push(participant.participant_digest.clone())
            }
            (
                MomentumRawFeatureRoleV4::LearnedRawLogistic,
                Some(MomentumAccumulatedQualificationStatusV4_1::QualifiedLearned),
            ) => learned.push(participant.participant_digest.clone()),
            (
                MomentumRawFeatureRoleV4::LearnedInteractionLogistic,
                Some(MomentumAccumulatedQualificationStatusV4_1::QualifiedLinearEquivalent),
            ) => duplicates.push(participant.participant_digest.clone()),
            _ => rejected.push(participant.participant_digest.clone()),
        }
    }
    if learned.is_empty() {
        return Ok((
            None,
            if duplicates.is_empty() {
                MomentumAccumulatedRosterStatusV4_1::NoQualifiedLearnedParticipant
            } else {
                MomentumAccumulatedRosterStatusV4_1::SemanticDuplicateOnly
            },
        ));
    }
    if benchmarks.is_empty() {
        return Ok((
            None,
            MomentumAccumulatedRosterStatusV4_1::BenchmarkUnavailable,
        ));
    }
    let mut roster = MomentumAccumulatedFutureRosterV4_1 {
        roster_version: ROSTER_VERSION_V4_1.to_string(),
        accumulated_family_digest: family.family_digest.clone(),
        learned_participant_digests: sorted_unique(learned),
        benchmark_participant_digests: sorted_unique(benchmarks),
        excluded_semantic_duplicate_digests: sorted_unique(duplicates),
        rejected_participant_digests: sorted_unique(rejected),
        inclusion_policy_digest: stable_hash_string(
            "v4.1-roster:all-qualified-learned+benchmark:no-ranking:deduplicate-linear-equivalent",
        ),
        status: MomentumAccumulatedRosterStatusV4_1::Ready,
        roster_digest: String::new(),
    };
    roster.roster_digest = roster_digest(&roster);
    Ok((Some(roster), MomentumAccumulatedRosterStatusV4_1::Ready))
}

#[allow(clippy::too_many_arguments)]
fn derive_evaluation(
    frozen: &FrozenV4Artifacts,
    registration: &MomentumSupplementalQualificationRegistrationV4_1,
    opening: &MomentumReserveOpeningReceiptV4_1,
    yield_result: &MomentumSupplementalValidationYieldV4_1,
    family: &MomentumAccumulatedQualificationFamilyV4_1,
    contribution: &MomentumAccumulatedInteractionContributionAuditV4_1,
    roster: Option<&MomentumAccumulatedFutureRosterV4_1>,
    roster_status: MomentumAccumulatedRosterStatusV4_1,
    reservation: &ProtectedEvaluationReservationV1,
    source_boundary_timestamp_ms: u64,
    accumulated_validation_identity_digest: &str,
) -> Result<
    (
        Option<MomentumAccumulatedEvaluationRegistrationV4_1>,
        MomentumAccumulatedEvaluationStatusV4_1,
    ),
    String,
> {
    let Some(roster) = roster else {
        let status = match roster_status {
            MomentumAccumulatedRosterStatusV4_1::StillInsufficientValidation => {
                MomentumAccumulatedEvaluationStatusV4_1::StillInsufficientValidation
            }
            MomentumAccumulatedRosterStatusV4_1::BenchmarkUnavailable => {
                MomentumAccumulatedEvaluationStatusV4_1::BenchmarkUnavailable
            }
            MomentumAccumulatedRosterStatusV4_1::SemanticDuplicateOnly => {
                MomentumAccumulatedEvaluationStatusV4_1::SemanticDuplicateOnly
            }
            _ => MomentumAccumulatedEvaluationStatusV4_1::NoQualifiedLearnedParticipant,
        };
        return Ok((None, status));
    };
    let minimum_accepted_timestamp_ms = source_boundary_timestamp_ms
        .max(reservation.provider_finality_boundary_ms)
        .max(
            reservation
                .reserved_timestamp_ms
                .iter()
                .copied()
                .max()
                .unwrap_or(0),
        )
        .checked_add(reservation.cadence_ms)
        .ok_or_else(|| "V4.1 future timestamp overflow".to_string())?;
    let mut value = MomentumAccumulatedEvaluationRegistrationV4_1 {
        registration_version: EVALUATION_VERSION_V4_1.to_string(),
        agent_id: AGENT_ID_V4_1.to_string(),
        source_v4_family_digest: frozen.family.family_digest.clone(),
        accumulated_family_digest: family.family_digest.clone(),
        roster_digest: roster.roster_digest.clone(),
        supplemental_registration_digest: registration.registration_digest.clone(),
        reserve_opening_receipt_digest: opening.receipt_digest.clone(),
        accumulated_yield_digest: yield_result.yield_digest.clone(),
        accumulated_receipt_digests: sorted_unique(
            family
                .accumulated_receipts
                .iter()
                .map(|item| item.receipt_digest.clone())
                .collect(),
        ),
        accumulated_interaction_audit_digest: Some(contribution.audit_digest.clone()),
        source_snapshot_digest: registration.source_snapshot_digest.clone(),
        source_boundary_timestamp_ms,
        consumed_validation_identity_digests: sorted_unique(vec![
            frozen.closure.v1_family_digest.clone(),
            frozen.closure.v2_family_digest.clone(),
            frozen.closure.v3_family_digest.clone(),
            frozen.family.family_digest.clone(),
            frozen.validation_yield_audit.audit_digest.clone(),
            accumulated_validation_identity_digest.to_string(),
        ]),
        protected_registration_digests: sorted_unique(
            reservation.protected_registration_digests.clone(),
        ),
        protected_timestamp_ms: {
            let mut values = reservation.reserved_timestamp_ms.clone();
            values.sort();
            values.dedup();
            values
        },
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
        registration_digest: String::new(),
    };
    value.registration_digest = evaluation_digest(&value);
    Ok((
        Some(value),
        MomentumAccumulatedEvaluationStatusV4_1::Registered,
    ))
}

#[derive(Clone, PartialEq, Message)]
struct FieldProtobufV4_1 {
    #[prost(string, tag = "1")]
    key: String,
    #[prost(string, tag = "2")]
    value: String,
}

macro_rules! fields_message {
    ($name:ident) => {
        #[derive(Clone, PartialEq, Message)]
        struct $name {
            #[prost(message, repeated, tag = "1")]
            fields: Vec<FieldProtobufV4_1>,
        }
    };
}

fields_message!(RegistrationProtobufV4_1);
fields_message!(OpeningProtobufV4_1);
fields_message!(YieldProtobufV4_1);
fields_message!(ReceiptProtobufV4_1);
fields_message!(ContributionProtobufV4_1);
fields_message!(DecisionProtobufV4_1);
fields_message!(EvaluationProtobufV4_1);
fields_message!(RequirementProtobufV4_1);
fields_message!(JournalProtobufV4_1);

#[derive(Clone, PartialEq, Message)]
struct FamilyProtobufV4_1 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4_1>,
    #[prost(message, repeated, tag = "2")]
    receipts: Vec<ReceiptProtobufV4_1>,
}

#[derive(Clone, PartialEq, Message)]
struct RosterProtobufV4_1 {
    #[prost(message, repeated, tag = "1")]
    fields: Vec<FieldProtobufV4_1>,
}

fn field(key: &str, value: impl ToString) -> FieldProtobufV4_1 {
    FieldProtobufV4_1 {
        key: key.to_string(),
        value: value.to_string(),
    }
}

fn fields(values: Vec<FieldProtobufV4_1>) -> Result<BTreeMap<String, String>, String> {
    let mut result = BTreeMap::new();
    for item in values {
        if item.key.is_empty() || result.insert(item.key, item.value).is_some() {
            return Err("V4.1 Protobuf field identity rejected".to_string());
        }
    }
    Ok(result)
}

fn take(values: &mut BTreeMap<String, String>, key: &str) -> Result<String, String> {
    values
        .remove(key)
        .ok_or_else(|| format!("V4.1 Protobuf field missing: {key}"))
}

fn take_usize(values: &mut BTreeMap<String, String>, key: &str) -> Result<usize, String> {
    take(values, key)?
        .parse()
        .map_err(|_| format!("V4.1 Protobuf usize rejected: {key}"))
}

fn take_u64(values: &mut BTreeMap<String, String>, key: &str) -> Result<u64, String> {
    take(values, key)?
        .parse()
        .map_err(|_| format!("V4.1 Protobuf u64 rejected: {key}"))
}

fn take_bool(values: &mut BTreeMap<String, String>, key: &str) -> Result<bool, String> {
    take(values, key)?
        .parse()
        .map_err(|_| format!("V4.1 Protobuf bool rejected: {key}"))
}

fn take_list(values: &mut BTreeMap<String, String>, key: &str) -> Result<Vec<String>, String> {
    let value = take(values, key)?;
    Ok(if value.is_empty() {
        Vec::new()
    } else {
        value.split(',').map(ToString::to_string).collect()
    })
}

fn take_u64_list(values: &mut BTreeMap<String, String>, key: &str) -> Result<Vec<u64>, String> {
    take_list(values, key)?
        .into_iter()
        .map(|item| {
            item.parse()
                .map_err(|_| format!("V4.1 Protobuf u64 list rejected: {key}"))
        })
        .collect()
}

fn optional(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}

fn take_optional(
    values: &mut BTreeMap<String, String>,
    key: &str,
) -> Result<Option<String>, String> {
    let value = take(values, key)?;
    Ok((!value.is_empty()).then_some(value))
}

fn finish(values: BTreeMap<String, String>) -> Result<(), String> {
    if values.is_empty() {
        Ok(())
    } else {
        Err("V4.1 unexpected Protobuf field rejected".to_string())
    }
}

fn encode_message(value: &impl Message) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    value
        .encode(&mut bytes)
        .map_err(|_| "V4.1 Protobuf encoding failed".to_string())?;
    Ok(bytes)
}

fn join(values: &[String]) -> String {
    values.join(",")
}

fn registration_to_pb(
    value: &MomentumSupplementalQualificationRegistrationV4_1,
) -> RegistrationProtobufV4_1 {
    RegistrationProtobufV4_1 {
        fields: vec![
            field("registration_version", &value.registration_version),
            field("agent_id", &value.agent_id),
            field("source_snapshot_digest", &value.source_snapshot_digest),
            field("canonical_intent_digest", &value.canonical_intent_digest),
            field("canonical_view_digest", &value.canonical_view_digest),
            field("v4_split_digest", &value.v4_split_digest),
            field("v4_registration_digest", &value.v4_registration_digest),
            field("v4_family_digest", &value.v4_family_digest),
            field(
                "validation_yield_audit_digest",
                &value.validation_yield_audit_digest,
            ),
            field("participant_digests", join(&value.participant_digests)),
            field(
                "participant_parameter_digests",
                join(&value.participant_parameter_digests),
            ),
            field(
                "participant_normalizer_digests",
                join(&value.participant_normalizer_digests),
            ),
            field("original_start", value.original_validation_range.start),
            field("original_end", value.original_validation_range.end),
            field(
                "supplemental_start",
                value.supplemental_validation_range.start,
            ),
            field("supplemental_end", value.supplemental_validation_range.end),
            field(
                "accumulated_validation_range_digest",
                &value.accumulated_validation_range_digest,
            ),
            field(
                "minimum_required_valid_samples",
                value.minimum_required_valid_samples,
            ),
            field(
                "model_retraining_forbidden",
                value.model_retraining_forbidden,
            ),
            field(
                "parameter_updates_forbidden",
                value.parameter_updates_forbidden,
            ),
            field(
                "configuration_changes_forbidden",
                value.configuration_changes_forbidden,
            ),
            field(
                "final_reserve_opening_one_time_only",
                value.final_reserve_opening_one_time_only,
            ),
            field("historical_test_forbidden", value.historical_test_forbidden),
            field(
                "future_evaluation_forbidden",
                value.future_evaluation_forbidden,
            ),
            field(
                "winner_selection_forbidden",
                value.winner_selection_forbidden,
            ),
            field("promotion_forbidden", value.promotion_forbidden),
            field(
                "reward_application_forbidden",
                value.reward_application_forbidden,
            ),
            field("registration_digest", &value.registration_digest),
        ],
    }
}

fn registration_from_pb(
    value: RegistrationProtobufV4_1,
) -> Result<MomentumSupplementalQualificationRegistrationV4_1, String> {
    let mut f = fields(value.fields)?;
    let result = MomentumSupplementalQualificationRegistrationV4_1 {
        registration_version: take(&mut f, "registration_version")?,
        agent_id: take(&mut f, "agent_id")?,
        source_snapshot_digest: take(&mut f, "source_snapshot_digest")?,
        canonical_intent_digest: take(&mut f, "canonical_intent_digest")?,
        canonical_view_digest: take(&mut f, "canonical_view_digest")?,
        v4_split_digest: take(&mut f, "v4_split_digest")?,
        v4_registration_digest: take(&mut f, "v4_registration_digest")?,
        v4_family_digest: take(&mut f, "v4_family_digest")?,
        validation_yield_audit_digest: take(&mut f, "validation_yield_audit_digest")?,
        participant_digests: take_list(&mut f, "participant_digests")?,
        participant_parameter_digests: take_list(&mut f, "participant_parameter_digests")?,
        participant_normalizer_digests: take_list(&mut f, "participant_normalizer_digests")?,
        original_validation_range: IndexRangeV0 {
            start: take_usize(&mut f, "original_start")?,
            end: take_usize(&mut f, "original_end")?,
        },
        supplemental_validation_range: IndexRangeV0 {
            start: take_usize(&mut f, "supplemental_start")?,
            end: take_usize(&mut f, "supplemental_end")?,
        },
        accumulated_validation_range_digest: take(&mut f, "accumulated_validation_range_digest")?,
        minimum_required_valid_samples: take_usize(&mut f, "minimum_required_valid_samples")?,
        model_retraining_forbidden: take_bool(&mut f, "model_retraining_forbidden")?,
        parameter_updates_forbidden: take_bool(&mut f, "parameter_updates_forbidden")?,
        configuration_changes_forbidden: take_bool(&mut f, "configuration_changes_forbidden")?,
        final_reserve_opening_one_time_only: take_bool(
            &mut f,
            "final_reserve_opening_one_time_only",
        )?,
        historical_test_forbidden: take_bool(&mut f, "historical_test_forbidden")?,
        future_evaluation_forbidden: take_bool(&mut f, "future_evaluation_forbidden")?,
        winner_selection_forbidden: take_bool(&mut f, "winner_selection_forbidden")?,
        promotion_forbidden: take_bool(&mut f, "promotion_forbidden")?,
        reward_application_forbidden: take_bool(&mut f, "reward_application_forbidden")?,
        registration_digest: take(&mut f, "registration_digest")?,
    };
    finish(f)?;
    validate_registration(&result)?;
    Ok(result)
}

fn opening_to_pb(value: &MomentumReserveOpeningReceiptV4_1) -> OpeningProtobufV4_1 {
    OpeningProtobufV4_1 {
        fields: vec![
            field("receipt_version", &value.receipt_version),
            field(
                "supplemental_registration_digest",
                &value.supplemental_registration_digest,
            ),
            field("source_snapshot_digest", &value.source_snapshot_digest),
            field("v4_final_reserve_digest", &value.v4_final_reserve_digest),
            field("opening_attempt_count", value.opening_attempt_count),
            field("opened_index_count", value.opened_index_count),
            field(
                "participant_identity_set_digest",
                &value.participant_identity_set_digest,
            ),
            field("status", format!("{:?}", value.status)),
            field("receipt_digest", &value.receipt_digest),
        ],
    }
}

fn parse_opening(value: &str) -> Result<MomentumReserveOpeningStatusV4_1, String> {
    match value {
        "Ready" => Ok(MomentumReserveOpeningStatusV4_1::Ready),
        "Opened" => Ok(MomentumReserveOpeningStatusV4_1::Opened),
        "AlreadyOpened" => Ok(MomentumReserveOpeningStatusV4_1::AlreadyOpened),
        "RegistrationMismatch" => Ok(MomentumReserveOpeningStatusV4_1::RegistrationMismatch),
        "ParticipantIdentityMismatch" => {
            Ok(MomentumReserveOpeningStatusV4_1::ParticipantIdentityMismatch)
        }
        "ReserveIdentityMismatch" => Ok(MomentumReserveOpeningStatusV4_1::ReserveIdentityMismatch),
        "IntegrityFailure" => Ok(MomentumReserveOpeningStatusV4_1::IntegrityFailure),
        _ => Err("V4.1 opening status rejected".to_string()),
    }
}

fn opening_from_pb(
    value: OpeningProtobufV4_1,
) -> Result<MomentumReserveOpeningReceiptV4_1, String> {
    let mut f = fields(value.fields)?;
    let result = MomentumReserveOpeningReceiptV4_1 {
        receipt_version: take(&mut f, "receipt_version")?,
        supplemental_registration_digest: take(&mut f, "supplemental_registration_digest")?,
        source_snapshot_digest: take(&mut f, "source_snapshot_digest")?,
        v4_final_reserve_digest: take(&mut f, "v4_final_reserve_digest")?,
        opening_attempt_count: take_usize(&mut f, "opening_attempt_count")?,
        opened_index_count: take_usize(&mut f, "opened_index_count")?,
        participant_identity_set_digest: take(&mut f, "participant_identity_set_digest")?,
        status: parse_opening(&take(&mut f, "status")?)?,
        receipt_digest: take(&mut f, "receipt_digest")?,
    };
    finish(f)?;
    validate_opening(&result)?;
    Ok(result)
}

fn yield_to_pb(value: &MomentumSupplementalValidationYieldV4_1) -> YieldProtobufV4_1 {
    YieldProtobufV4_1 {
        fields: vec![
            field(
                "original_valid_sample_count",
                value.original_valid_sample_count,
            ),
            field(
                "supplemental_valid_sample_count",
                value.supplemental_valid_sample_count,
            ),
            field(
                "accumulated_valid_sample_count",
                value.accumulated_valid_sample_count,
            ),
            field(
                "original_neutral_excluded_count",
                value.original_neutral_excluded_count,
            ),
            field(
                "supplemental_neutral_excluded_count",
                value.supplemental_neutral_excluded_count,
            ),
            field(
                "minimum_required_valid_samples",
                value.minimum_required_valid_samples,
            ),
            field("minimum_reached", value.minimum_reached),
            field("yield_digest", &value.yield_digest),
        ],
    }
}

fn yield_from_pb(
    value: YieldProtobufV4_1,
) -> Result<MomentumSupplementalValidationYieldV4_1, String> {
    let mut f = fields(value.fields)?;
    let result = MomentumSupplementalValidationYieldV4_1 {
        original_valid_sample_count: take_usize(&mut f, "original_valid_sample_count")?,
        supplemental_valid_sample_count: take_usize(&mut f, "supplemental_valid_sample_count")?,
        accumulated_valid_sample_count: take_usize(&mut f, "accumulated_valid_sample_count")?,
        original_neutral_excluded_count: take_usize(&mut f, "original_neutral_excluded_count")?,
        supplemental_neutral_excluded_count: take_usize(
            &mut f,
            "supplemental_neutral_excluded_count",
        )?,
        minimum_required_valid_samples: take_usize(&mut f, "minimum_required_valid_samples")?,
        minimum_reached: take_bool(&mut f, "minimum_reached")?,
        yield_digest: take(&mut f, "yield_digest")?,
    };
    finish(f)?;
    validate_yield(&result)?;
    Ok(result)
}

fn receipt_to_pb(value: &MomentumAccumulatedQualificationReceiptV4_1) -> ReceiptProtobufV4_1 {
    ReceiptProtobufV4_1 {
        fields: vec![
            field("receipt_version", &value.receipt_version),
            field("participant_digest", &value.participant_digest),
            field(
                "v4_original_receipt_digest",
                &value.v4_original_receipt_digest,
            ),
            field(
                "supplemental_registration_digest",
                &value.supplemental_registration_digest,
            ),
            field(
                "reserve_opening_receipt_digest",
                &value.reserve_opening_receipt_digest,
            ),
            field("accumulated_yield_digest", &value.accumulated_yield_digest),
            field(
                "accumulated_validation_identity_digest",
                &value.accumulated_validation_identity_digest,
            ),
            field(
                "qualification_policy_digest",
                &value.qualification_policy_digest,
            ),
            field("private_metric_digest", &value.private_metric_digest),
            field("status", format!("{:?}", value.status)),
            field(
                "parameter_updates_after_freeze",
                value.parameter_updates_after_freeze,
            ),
            field("historical_test_reads", value.historical_test_reads),
            field("future_evaluation_reads", value.future_evaluation_reads),
            field("receipt_digest", &value.receipt_digest),
        ],
    }
}

fn parse_qualification(value: &str) -> Result<MomentumAccumulatedQualificationStatusV4_1, String> {
    match value {
        "QualifiedLearned" => Ok(MomentumAccumulatedQualificationStatusV4_1::QualifiedLearned),
        "QualifiedLinearEquivalent" => {
            Ok(MomentumAccumulatedQualificationStatusV4_1::QualifiedLinearEquivalent)
        }
        "BenchmarkQualified" => Ok(MomentumAccumulatedQualificationStatusV4_1::BenchmarkQualified),
        "StillInsufficientValidation" => {
            Ok(MomentumAccumulatedQualificationStatusV4_1::StillInsufficientValidation)
        }
        "RejectedProbabilityCollapse" => {
            Ok(MomentumAccumulatedQualificationStatusV4_1::RejectedProbabilityCollapse)
        }
        "RejectedNumericalFailure" => {
            Ok(MomentumAccumulatedQualificationStatusV4_1::RejectedNumericalFailure)
        }
        "RejectedFeatureIntegrity" => {
            Ok(MomentumAccumulatedQualificationStatusV4_1::RejectedFeatureIntegrity)
        }
        "RejectedPolicyInvariant" => {
            Ok(MomentumAccumulatedQualificationStatusV4_1::RejectedPolicyInvariant)
        }
        _ => Err("V4.1 qualification status rejected".to_string()),
    }
}

fn receipt_from_pb(
    value: ReceiptProtobufV4_1,
) -> Result<MomentumAccumulatedQualificationReceiptV4_1, String> {
    let mut f = fields(value.fields)?;
    let result = MomentumAccumulatedQualificationReceiptV4_1 {
        receipt_version: take(&mut f, "receipt_version")?,
        participant_digest: take(&mut f, "participant_digest")?,
        v4_original_receipt_digest: take(&mut f, "v4_original_receipt_digest")?,
        supplemental_registration_digest: take(&mut f, "supplemental_registration_digest")?,
        reserve_opening_receipt_digest: take(&mut f, "reserve_opening_receipt_digest")?,
        accumulated_yield_digest: take(&mut f, "accumulated_yield_digest")?,
        accumulated_validation_identity_digest: take(
            &mut f,
            "accumulated_validation_identity_digest",
        )?,
        qualification_policy_digest: take(&mut f, "qualification_policy_digest")?,
        private_metric_digest: take(&mut f, "private_metric_digest")?,
        status: parse_qualification(&take(&mut f, "status")?)?,
        parameter_updates_after_freeze: take_usize(&mut f, "parameter_updates_after_freeze")?,
        historical_test_reads: take_usize(&mut f, "historical_test_reads")?,
        future_evaluation_reads: take_usize(&mut f, "future_evaluation_reads")?,
        receipt_digest: take(&mut f, "receipt_digest")?,
    };
    finish(f)?;
    validate_receipt(&result)?;
    Ok(result)
}

fn contribution_to_pb(
    value: &MomentumAccumulatedInteractionContributionAuditV4_1,
) -> ContributionProtobufV4_1 {
    ContributionProtobufV4_1 {
        fields: vec![
            field("participant_digest", &value.participant_digest),
            field(
                "original_contribution_audit_digest",
                &value.original_contribution_audit_digest,
            ),
            field(
                "accumulated_validation_identity_digest",
                &value.accumulated_validation_identity_digest,
            ),
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

fn parse_contribution(value: &str) -> Result<InteractionContributionStatusV4, String> {
    match value {
        "MaterialInteractionContribution" => {
            Ok(InteractionContributionStatusV4::MaterialInteractionContribution)
        }
        "DetectableButBelowPolicy" => Ok(InteractionContributionStatusV4::DetectableButBelowPolicy),
        "LinearEquivalent" => Ok(InteractionContributionStatusV4::LinearEquivalent),
        "Invalid" => Ok(InteractionContributionStatusV4::Invalid),
        _ => Err("V4.1 contribution status rejected".to_string()),
    }
}

fn contribution_from_pb(
    value: ContributionProtobufV4_1,
) -> Result<MomentumAccumulatedInteractionContributionAuditV4_1, String> {
    let mut f = fields(value.fields)?;
    let result = MomentumAccumulatedInteractionContributionAuditV4_1 {
        participant_digest: take(&mut f, "participant_digest")?,
        original_contribution_audit_digest: take(&mut f, "original_contribution_audit_digest")?,
        accumulated_validation_identity_digest: take(
            &mut f,
            "accumulated_validation_identity_digest",
        )?,
        full_prediction_digest: take(&mut f, "full_prediction_digest")?,
        nonlinear_ablated_prediction_digest: take(&mut f, "nonlinear_ablated_prediction_digest")?,
        contribution_policy_digest: take(&mut f, "contribution_policy_digest")?,
        contribution_status: parse_contribution(&take(&mut f, "contribution_status")?)?,
        audit_digest: take(&mut f, "audit_digest")?,
    };
    finish(f)?;
    validate_contribution(&result)?;
    Ok(result)
}

fn family_to_pb(value: &MomentumAccumulatedQualificationFamilyV4_1) -> FamilyProtobufV4_1 {
    FamilyProtobufV4_1 {
        fields: vec![
            field("family_version", &value.family_version),
            field("source_v4_family_digest", &value.source_v4_family_digest),
            field(
                "supplemental_registration_digest",
                &value.supplemental_registration_digest,
            ),
            field(
                "reserve_opening_receipt_digest",
                &value.reserve_opening_receipt_digest,
            ),
            field("accumulated_yield_digest", &value.accumulated_yield_digest),
            field("participant_digests", join(&value.participant_digests)),
            field(
                "accumulated_interaction_audit_digest",
                optional(&value.accumulated_interaction_audit_digest),
            ),
            field("qualified_learned_count", value.qualified_learned_count),
            field("qualified_benchmark_count", value.qualified_benchmark_count),
            field("winner_selected", value.winner_selected),
            field("parameters_changed", value.parameters_changed),
            field(
                "eligible_for_active_committee",
                value.eligible_for_active_committee,
            ),
            field("eligible_for_promotion", value.eligible_for_promotion),
            field("eligible_for_reward", value.eligible_for_reward),
            field("family_digest", &value.family_digest),
        ],
        receipts: value
            .accumulated_receipts
            .iter()
            .map(receipt_to_pb)
            .collect(),
    }
}

fn family_from_pb(
    value: FamilyProtobufV4_1,
) -> Result<MomentumAccumulatedQualificationFamilyV4_1, String> {
    let receipts = value
        .receipts
        .into_iter()
        .map(receipt_from_pb)
        .collect::<Result<Vec<_>, _>>()?;
    let mut f = fields(value.fields)?;
    let result = MomentumAccumulatedQualificationFamilyV4_1 {
        family_version: take(&mut f, "family_version")?,
        source_v4_family_digest: take(&mut f, "source_v4_family_digest")?,
        supplemental_registration_digest: take(&mut f, "supplemental_registration_digest")?,
        reserve_opening_receipt_digest: take(&mut f, "reserve_opening_receipt_digest")?,
        accumulated_yield_digest: take(&mut f, "accumulated_yield_digest")?,
        participant_digests: take_list(&mut f, "participant_digests")?,
        accumulated_receipts: receipts,
        accumulated_interaction_audit_digest: take_optional(
            &mut f,
            "accumulated_interaction_audit_digest",
        )?,
        qualified_learned_count: take_usize(&mut f, "qualified_learned_count")?,
        qualified_benchmark_count: take_usize(&mut f, "qualified_benchmark_count")?,
        winner_selected: take_bool(&mut f, "winner_selected")?,
        parameters_changed: take_bool(&mut f, "parameters_changed")?,
        eligible_for_active_committee: take_bool(&mut f, "eligible_for_active_committee")?,
        eligible_for_promotion: take_bool(&mut f, "eligible_for_promotion")?,
        eligible_for_reward: take_bool(&mut f, "eligible_for_reward")?,
        family_digest: take(&mut f, "family_digest")?,
    };
    finish(f)?;
    validate_family(&result)?;
    Ok(result)
}

fn decision_to_pb(value: &MomentumAccumulatedPathDecisionArtifactV4_1) -> DecisionProtobufV4_1 {
    DecisionProtobufV4_1 {
        fields: vec![
            field("decision_version", &value.decision_version),
            field(
                "accumulated_family_digest",
                &value.accumulated_family_digest,
            ),
            field("minimum_reached", value.minimum_reached),
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

fn parse_decision(value: &str) -> Result<MomentumAccumulatedPathDecisionV4_1, String> {
    match value {
        "RawFeatureLearnedPathViable" => {
            Ok(MomentumAccumulatedPathDecisionV4_1::RawFeatureLearnedPathViable)
        }
        "OnlyLinearRawPathViable" => {
            Ok(MomentumAccumulatedPathDecisionV4_1::OnlyLinearRawPathViable)
        }
        "StillInsufficientValidation" => {
            Ok(MomentumAccumulatedPathDecisionV4_1::StillInsufficientValidation)
        }
        "NoQualifiedRawFeatureLearner" => {
            Ok(MomentumAccumulatedPathDecisionV4_1::NoQualifiedRawFeatureLearner)
        }
        "TechnicalFailure" => Ok(MomentumAccumulatedPathDecisionV4_1::TechnicalFailure),
        _ => Err("V4.1 decision rejected".to_string()),
    }
}

fn decision_from_pb(
    value: DecisionProtobufV4_1,
) -> Result<MomentumAccumulatedPathDecisionArtifactV4_1, String> {
    let mut f = fields(value.fields)?;
    let result = MomentumAccumulatedPathDecisionArtifactV4_1 {
        decision_version: take(&mut f, "decision_version")?,
        accumulated_family_digest: take(&mut f, "accumulated_family_digest")?,
        minimum_reached: take_bool(&mut f, "minimum_reached")?,
        qualified_raw_logistic: take_bool(&mut f, "qualified_raw_logistic")?,
        qualified_material_interaction: take_bool(&mut f, "qualified_material_interaction")?,
        decision: parse_decision(&take(&mut f, "decision")?)?,
        decision_digest: take(&mut f, "decision_digest")?,
    };
    finish(f)?;
    if result.decision_digest != decision_digest(&result) {
        return Err("V4.1 decision digest rejected".to_string());
    }
    Ok(result)
}

fn roster_to_pb(value: &MomentumAccumulatedFutureRosterV4_1) -> RosterProtobufV4_1 {
    RosterProtobufV4_1 {
        fields: vec![
            field("roster_version", &value.roster_version),
            field(
                "accumulated_family_digest",
                &value.accumulated_family_digest,
            ),
            field(
                "learned_participant_digests",
                join(&value.learned_participant_digests),
            ),
            field(
                "benchmark_participant_digests",
                join(&value.benchmark_participant_digests),
            ),
            field(
                "excluded_semantic_duplicate_digests",
                join(&value.excluded_semantic_duplicate_digests),
            ),
            field(
                "rejected_participant_digests",
                join(&value.rejected_participant_digests),
            ),
            field("inclusion_policy_digest", &value.inclusion_policy_digest),
            field("status", format!("{:?}", value.status)),
            field("roster_digest", &value.roster_digest),
        ],
    }
}

fn parse_roster(value: &str) -> Result<MomentumAccumulatedRosterStatusV4_1, String> {
    match value {
        "Ready" => Ok(MomentumAccumulatedRosterStatusV4_1::Ready),
        "StillInsufficientValidation" => {
            Ok(MomentumAccumulatedRosterStatusV4_1::StillInsufficientValidation)
        }
        "NoQualifiedLearnedParticipant" => {
            Ok(MomentumAccumulatedRosterStatusV4_1::NoQualifiedLearnedParticipant)
        }
        "BenchmarkUnavailable" => Ok(MomentumAccumulatedRosterStatusV4_1::BenchmarkUnavailable),
        "SemanticDuplicateOnly" => Ok(MomentumAccumulatedRosterStatusV4_1::SemanticDuplicateOnly),
        "IntegrityFailure" => Ok(MomentumAccumulatedRosterStatusV4_1::IntegrityFailure),
        _ => Err("V4.1 roster status rejected".to_string()),
    }
}

fn roster_from_pb(
    value: RosterProtobufV4_1,
) -> Result<MomentumAccumulatedFutureRosterV4_1, String> {
    let mut f = fields(value.fields)?;
    let result = MomentumAccumulatedFutureRosterV4_1 {
        roster_version: take(&mut f, "roster_version")?,
        accumulated_family_digest: take(&mut f, "accumulated_family_digest")?,
        learned_participant_digests: take_list(&mut f, "learned_participant_digests")?,
        benchmark_participant_digests: take_list(&mut f, "benchmark_participant_digests")?,
        excluded_semantic_duplicate_digests: take_list(
            &mut f,
            "excluded_semantic_duplicate_digests",
        )?,
        rejected_participant_digests: take_list(&mut f, "rejected_participant_digests")?,
        inclusion_policy_digest: take(&mut f, "inclusion_policy_digest")?,
        status: parse_roster(&take(&mut f, "status")?)?,
        roster_digest: take(&mut f, "roster_digest")?,
    };
    finish(f)?;
    if result.roster_version != ROSTER_VERSION_V4_1
        || result.status != MomentumAccumulatedRosterStatusV4_1::Ready
        || result.learned_participant_digests.is_empty()
        || result.benchmark_participant_digests.is_empty()
        || result.roster_digest != roster_digest(&result)
    {
        return Err("V4.1 roster rejected".to_string());
    }
    Ok(result)
}

fn evaluation_to_pb(
    value: &MomentumAccumulatedEvaluationRegistrationV4_1,
) -> EvaluationProtobufV4_1 {
    EvaluationProtobufV4_1 {
        fields: vec![
            field("registration_version", &value.registration_version),
            field("agent_id", &value.agent_id),
            field("source_v4_family_digest", &value.source_v4_family_digest),
            field(
                "accumulated_family_digest",
                &value.accumulated_family_digest,
            ),
            field("roster_digest", &value.roster_digest),
            field(
                "supplemental_registration_digest",
                &value.supplemental_registration_digest,
            ),
            field(
                "reserve_opening_receipt_digest",
                &value.reserve_opening_receipt_digest,
            ),
            field("accumulated_yield_digest", &value.accumulated_yield_digest),
            field(
                "accumulated_receipt_digests",
                join(&value.accumulated_receipt_digests),
            ),
            field(
                "accumulated_interaction_audit_digest",
                optional(&value.accumulated_interaction_audit_digest),
            ),
            field("source_snapshot_digest", &value.source_snapshot_digest),
            field(
                "source_boundary_timestamp_ms",
                value.source_boundary_timestamp_ms,
            ),
            field(
                "consumed_validation_identity_digests",
                join(&value.consumed_validation_identity_digests),
            ),
            field(
                "protected_registration_digests",
                join(&value.protected_registration_digests),
            ),
            field(
                "protected_timestamp_ms",
                value
                    .protected_timestamp_ms
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            field(
                "provider_finality_boundary_ms",
                value.provider_finality_boundary_ms,
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
    }
}

fn evaluation_from_pb(
    value: EvaluationProtobufV4_1,
) -> Result<MomentumAccumulatedEvaluationRegistrationV4_1, String> {
    let mut f = fields(value.fields)?;
    let result = MomentumAccumulatedEvaluationRegistrationV4_1 {
        registration_version: take(&mut f, "registration_version")?,
        agent_id: take(&mut f, "agent_id")?,
        source_v4_family_digest: take(&mut f, "source_v4_family_digest")?,
        accumulated_family_digest: take(&mut f, "accumulated_family_digest")?,
        roster_digest: take(&mut f, "roster_digest")?,
        supplemental_registration_digest: take(&mut f, "supplemental_registration_digest")?,
        reserve_opening_receipt_digest: take(&mut f, "reserve_opening_receipt_digest")?,
        accumulated_yield_digest: take(&mut f, "accumulated_yield_digest")?,
        accumulated_receipt_digests: take_list(&mut f, "accumulated_receipt_digests")?,
        accumulated_interaction_audit_digest: take_optional(
            &mut f,
            "accumulated_interaction_audit_digest",
        )?,
        source_snapshot_digest: take(&mut f, "source_snapshot_digest")?,
        source_boundary_timestamp_ms: take_u64(&mut f, "source_boundary_timestamp_ms")?,
        consumed_validation_identity_digests: take_list(
            &mut f,
            "consumed_validation_identity_digests",
        )?,
        protected_registration_digests: take_list(&mut f, "protected_registration_digests")?,
        protected_timestamp_ms: take_u64_list(&mut f, "protected_timestamp_ms")?,
        provider_finality_boundary_ms: take_u64(&mut f, "provider_finality_boundary_ms")?,
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
    finish(f)?;
    if result.registration_version != EVALUATION_VERSION_V4_1
        || result.agent_id != AGENT_ID_V4_1
        || result.accumulated_receipt_digests.len() != 3
        || !result.labels_hidden_until_opening
        || !result.probabilities_hidden_until_opening
        || !result.one_time_opening_required
        || !result.winner_selection_forbidden_before_opening
        || !result.active_promotion_forbidden
        || !result.reward_application_forbidden
        || result.maximum_requests != 1
        || result.maximum_concurrency != 1
        || result.maximum_retries != 0
        || result.minimum_accepted_timestamp_ms <= result.source_boundary_timestamp_ms
        || result.registration_digest != evaluation_digest(&result)
    {
        return Err("V4.1 future evaluation registration rejected".to_string());
    }
    Ok(result)
}

fn requirement_to_pb(value: &MomentumAdditionalEvidenceRequirementV4_1) -> RequirementProtobufV4_1 {
    RequirementProtobufV4_1 {
        fields: vec![
            field("requirement_version", &value.requirement_version),
            field(
                "accumulated_valid_sample_count",
                value.accumulated_valid_sample_count,
            ),
            field(
                "minimum_required_valid_samples",
                value.minimum_required_valid_samples,
            ),
            field(
                "minimum_additional_valid_samples",
                value.minimum_additional_valid_samples,
            ),
            field(
                "current_source_boundary_timestamp_ms",
                value.current_source_boundary_timestamp_ms,
            ),
            field(
                "required_dataset_kind",
                format!("{:?}", value.required_dataset_kind),
            ),
            field("market_scope", format!("{:?}", value.market_scope)),
            field("symbols", join(&value.symbols)),
            field("cadence", &value.cadence),
            field(
                "existing_evidence_fully_consumed_for_qualification",
                value.existing_evidence_fully_consumed_for_qualification,
            ),
            field(
                "new_evidence_identity_required",
                value.new_evidence_identity_required,
            ),
            field(
                "separate_acquisition_preregistration_required",
                value.separate_acquisition_preregistration_required,
            ),
            field("requirement_digest", &value.requirement_digest),
        ],
    }
}

fn requirement_from_pb(
    value: RequirementProtobufV4_1,
) -> Result<MomentumAdditionalEvidenceRequirementV4_1, String> {
    let mut f = fields(value.fields)?;
    let dataset = match take(&mut f, "required_dataset_kind")?.as_str() {
        "CryptoDailyOhlcv" => DatasetKind::CryptoDailyOhlcv,
        _ => return Err("V4.1 required dataset kind rejected".to_string()),
    };
    let market = match take(&mut f, "market_scope")?.as_str() {
        "BtcCrypto" => AcquisitionMarketScope::BtcCrypto,
        _ => return Err("V4.1 market scope rejected".to_string()),
    };
    let result = MomentumAdditionalEvidenceRequirementV4_1 {
        requirement_version: take(&mut f, "requirement_version")?,
        accumulated_valid_sample_count: take_usize(&mut f, "accumulated_valid_sample_count")?,
        minimum_required_valid_samples: take_usize(&mut f, "minimum_required_valid_samples")?,
        minimum_additional_valid_samples: take_usize(&mut f, "minimum_additional_valid_samples")?,
        current_source_boundary_timestamp_ms: take_u64(
            &mut f,
            "current_source_boundary_timestamp_ms",
        )?,
        required_dataset_kind: dataset,
        market_scope: market,
        symbols: take_list(&mut f, "symbols")?,
        cadence: take(&mut f, "cadence")?,
        existing_evidence_fully_consumed_for_qualification: take_bool(
            &mut f,
            "existing_evidence_fully_consumed_for_qualification",
        )?,
        new_evidence_identity_required: take_bool(&mut f, "new_evidence_identity_required")?,
        separate_acquisition_preregistration_required: take_bool(
            &mut f,
            "separate_acquisition_preregistration_required",
        )?,
        requirement_digest: take(&mut f, "requirement_digest")?,
    };
    finish(f)?;
    if result.requirement_version != REQUIREMENT_VERSION_V4_1
        || result.minimum_additional_valid_samples
            != result
                .minimum_required_valid_samples
                .saturating_sub(result.accumulated_valid_sample_count)
        || !result.existing_evidence_fully_consumed_for_qualification
        || !result.new_evidence_identity_required
        || !result.separate_acquisition_preregistration_required
        || result.requirement_digest != requirement_digest(&result)
    {
        return Err("V4.1 evidence requirement rejected".to_string());
    }
    Ok(result)
}

fn journal_to_pb(value: &MomentumSupplementalJournalV4_1) -> JournalProtobufV4_1 {
    JournalProtobufV4_1 {
        fields: vec![
            field("journal_version", &value.journal_version),
            field("agent_id", &value.agent_id),
            field(
                "supplemental_registration_digest",
                &value.supplemental_registration_digest,
            ),
            field(
                "ready_opening_receipt_digest",
                &value.ready_opening_receipt_digest,
            ),
            field("opened_receipt_digest", &value.opened_receipt_digest),
            field("accumulated_yield_digest", &value.accumulated_yield_digest),
            field(
                "accumulated_family_digest",
                &value.accumulated_family_digest,
            ),
            field(
                "accumulated_decision_digest",
                &value.accumulated_decision_digest,
            ),
            field("roster_digest", optional(&value.roster_digest)),
            field(
                "evaluation_registration_digest",
                optional(&value.evaluation_registration_digest),
            ),
            field(
                "additional_evidence_requirement_digest",
                optional(&value.additional_evidence_requirement_digest),
            ),
            field(
                "registration_reopened_before_reserve_access",
                value.registration_reopened_before_reserve_access,
            ),
            field(
                "frozen_participants_reconstructed",
                value.frozen_participants_reconstructed,
            ),
            field(
                "participant_parameters_unchanged",
                value.participant_parameters_unchanged,
            ),
            field("normalizers_unchanged", value.normalizers_unchanged),
            field("status", format!("{:?}", value.status)),
            field("journal_digest", &value.journal_digest),
        ],
    }
}

fn parse_execution(value: &str) -> Result<MomentumSupplementalExecutionStatusV4_1, String> {
    match value {
        "Planned" => Ok(MomentumSupplementalExecutionStatusV4_1::Planned),
        "Executed" => Ok(MomentumSupplementalExecutionStatusV4_1::Executed),
        "AlreadyOpened" => Ok(MomentumSupplementalExecutionStatusV4_1::AlreadyOpened),
        "TechnicalFailure" => Ok(MomentumSupplementalExecutionStatusV4_1::TechnicalFailure),
        _ => Err("V4.1 execution status rejected".to_string()),
    }
}

fn journal_from_pb(value: JournalProtobufV4_1) -> Result<MomentumSupplementalJournalV4_1, String> {
    let mut f = fields(value.fields)?;
    let result = MomentumSupplementalJournalV4_1 {
        journal_version: take(&mut f, "journal_version")?,
        agent_id: take(&mut f, "agent_id")?,
        supplemental_registration_digest: take(&mut f, "supplemental_registration_digest")?,
        ready_opening_receipt_digest: take(&mut f, "ready_opening_receipt_digest")?,
        opened_receipt_digest: take(&mut f, "opened_receipt_digest")?,
        accumulated_yield_digest: take(&mut f, "accumulated_yield_digest")?,
        accumulated_family_digest: take(&mut f, "accumulated_family_digest")?,
        accumulated_decision_digest: take(&mut f, "accumulated_decision_digest")?,
        roster_digest: take_optional(&mut f, "roster_digest")?,
        evaluation_registration_digest: take_optional(&mut f, "evaluation_registration_digest")?,
        additional_evidence_requirement_digest: take_optional(
            &mut f,
            "additional_evidence_requirement_digest",
        )?,
        registration_reopened_before_reserve_access: take_bool(
            &mut f,
            "registration_reopened_before_reserve_access",
        )?,
        frozen_participants_reconstructed: take_bool(&mut f, "frozen_participants_reconstructed")?,
        participant_parameters_unchanged: take_bool(&mut f, "participant_parameters_unchanged")?,
        normalizers_unchanged: take_bool(&mut f, "normalizers_unchanged")?,
        status: parse_execution(&take(&mut f, "status")?)?,
        journal_digest: take(&mut f, "journal_digest")?,
    };
    finish(f)?;
    if result.journal_version != JOURNAL_VERSION_V4_1
        || result.agent_id != AGENT_ID_V4_1
        || !result.registration_reopened_before_reserve_access
        || !result.frozen_participants_reconstructed
        || !result.participant_parameters_unchanged
        || !result.normalizers_unchanged
        || result.status != MomentumSupplementalExecutionStatusV4_1::Executed
        || result.journal_digest != journal_digest(&result)
    {
        return Err("V4.1 journal rejected".to_string());
    }
    Ok(result)
}

macro_rules! public_codec {
    ($encode:ident, $decode:ident, $ty:ty, $pb:ty, $validate:expr, $to_pb:ident, $from_pb:ident, $error:literal) => {
        pub fn $encode(value: &$ty) -> Result<Vec<u8>, String> {
            ($validate)(value)?;
            encode_message(&$to_pb(value))
        }
        pub fn $decode(bytes: &[u8]) -> Result<$ty, String> {
            $from_pb(<$pb>::decode(bytes).map_err(|_| $error.to_string())?)
        }
    };
}

public_codec!(
    encode_momentum_supplemental_registration_protobuf_v4_1,
    decode_momentum_supplemental_registration_protobuf_v4_1,
    MomentumSupplementalQualificationRegistrationV4_1,
    RegistrationProtobufV4_1,
    validate_registration,
    registration_to_pb,
    registration_from_pb,
    "V4.1 registration Protobuf rejected"
);
public_codec!(
    encode_momentum_reserve_opening_protobuf_v4_1,
    decode_momentum_reserve_opening_protobuf_v4_1,
    MomentumReserveOpeningReceiptV4_1,
    OpeningProtobufV4_1,
    validate_opening,
    opening_to_pb,
    opening_from_pb,
    "V4.1 opening Protobuf rejected"
);
public_codec!(
    encode_momentum_supplemental_yield_protobuf_v4_1,
    decode_momentum_supplemental_yield_protobuf_v4_1,
    MomentumSupplementalValidationYieldV4_1,
    YieldProtobufV4_1,
    validate_yield,
    yield_to_pb,
    yield_from_pb,
    "V4.1 yield Protobuf rejected"
);
public_codec!(
    encode_momentum_accumulated_receipt_protobuf_v4_1,
    decode_momentum_accumulated_receipt_protobuf_v4_1,
    MomentumAccumulatedQualificationReceiptV4_1,
    ReceiptProtobufV4_1,
    validate_receipt,
    receipt_to_pb,
    receipt_from_pb,
    "V4.1 receipt Protobuf rejected"
);
public_codec!(
    encode_momentum_accumulated_contribution_protobuf_v4_1,
    decode_momentum_accumulated_contribution_protobuf_v4_1,
    MomentumAccumulatedInteractionContributionAuditV4_1,
    ContributionProtobufV4_1,
    validate_contribution,
    contribution_to_pb,
    contribution_from_pb,
    "V4.1 contribution Protobuf rejected"
);
public_codec!(
    encode_momentum_accumulated_family_protobuf_v4_1,
    decode_momentum_accumulated_family_protobuf_v4_1,
    MomentumAccumulatedQualificationFamilyV4_1,
    FamilyProtobufV4_1,
    validate_family,
    family_to_pb,
    family_from_pb,
    "V4.1 family Protobuf rejected"
);

pub fn encode_momentum_accumulated_decision_protobuf_v4_1(
    value: &MomentumAccumulatedPathDecisionArtifactV4_1,
) -> Result<Vec<u8>, String> {
    if value.decision_digest != decision_digest(value) {
        return Err("V4.1 decision digest rejected".to_string());
    }
    encode_message(&decision_to_pb(value))
}
pub fn decode_momentum_accumulated_decision_protobuf_v4_1(
    bytes: &[u8],
) -> Result<MomentumAccumulatedPathDecisionArtifactV4_1, String> {
    decision_from_pb(
        DecisionProtobufV4_1::decode(bytes)
            .map_err(|_| "V4.1 decision Protobuf rejected".to_string())?,
    )
}

pub fn encode_momentum_accumulated_roster_protobuf_v4_1(
    value: &MomentumAccumulatedFutureRosterV4_1,
) -> Result<Vec<u8>, String> {
    if value.roster_digest != roster_digest(value) {
        return Err("V4.1 roster digest rejected".to_string());
    }
    encode_message(&roster_to_pb(value))
}
pub fn decode_momentum_accumulated_roster_protobuf_v4_1(
    bytes: &[u8],
) -> Result<MomentumAccumulatedFutureRosterV4_1, String> {
    roster_from_pb(
        RosterProtobufV4_1::decode(bytes)
            .map_err(|_| "V4.1 roster Protobuf rejected".to_string())?,
    )
}

pub fn encode_momentum_accumulated_evaluation_protobuf_v4_1(
    value: &MomentumAccumulatedEvaluationRegistrationV4_1,
) -> Result<Vec<u8>, String> {
    if value.registration_digest != evaluation_digest(value) {
        return Err("V4.1 evaluation digest rejected".to_string());
    }
    encode_message(&evaluation_to_pb(value))
}
pub fn decode_momentum_accumulated_evaluation_protobuf_v4_1(
    bytes: &[u8],
) -> Result<MomentumAccumulatedEvaluationRegistrationV4_1, String> {
    evaluation_from_pb(
        EvaluationProtobufV4_1::decode(bytes)
            .map_err(|_| "V4.1 evaluation Protobuf rejected".to_string())?,
    )
}

pub fn encode_momentum_additional_requirement_protobuf_v4_1(
    value: &MomentumAdditionalEvidenceRequirementV4_1,
) -> Result<Vec<u8>, String> {
    if value.requirement_digest != requirement_digest(value) {
        return Err("V4.1 requirement digest rejected".to_string());
    }
    encode_message(&requirement_to_pb(value))
}
pub fn decode_momentum_additional_requirement_protobuf_v4_1(
    bytes: &[u8],
) -> Result<MomentumAdditionalEvidenceRequirementV4_1, String> {
    requirement_from_pb(
        RequirementProtobufV4_1::decode(bytes)
            .map_err(|_| "V4.1 requirement Protobuf rejected".to_string())?,
    )
}

pub fn encode_momentum_supplemental_journal_protobuf_v4_1(
    value: &MomentumSupplementalJournalV4_1,
) -> Result<Vec<u8>, String> {
    if value.journal_digest != journal_digest(value) {
        return Err("V4.1 journal digest rejected".to_string());
    }
    encode_message(&journal_to_pb(value))
}
pub fn decode_momentum_supplemental_journal_protobuf_v4_1(
    bytes: &[u8],
) -> Result<MomentumSupplementalJournalV4_1, String> {
    journal_from_pb(
        JournalProtobufV4_1::decode(bytes)
            .map_err(|_| "V4.1 journal Protobuf rejected".to_string())?,
    )
}

fn protobuf_paths(directory: &Path) -> Result<Vec<PathBuf>, String> {
    if !directory.is_dir() {
        return Err("V4.1 artifact directory unavailable".to_string());
    }
    let mut paths = fs::read_dir(directory)
        .map_err(|_| "V4.1 artifact directory read failed".to_string())?
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
        return Err("V4.1 single artifact identity rejected".to_string());
    }
    decode(&fs::read(&paths[0]).map_err(|_| "V4.1 artifact read failed".to_string())?)
}

fn load_frozen_v4(root: &Path) -> Result<FrozenV4Artifacts, String> {
    let v4 = root.join("v4").join(AGENT_ID_V4_1);
    let closure = read_single(
        &v4.join("closures"),
        decode_momentum_frozen_mamba_closure_protobuf_v4,
    )?;
    let split = read_single(
        &v4.join("splits"),
        decode_momentum_raw_feature_split_protobuf_v4,
    )?;
    let registration = read_single(
        &v4.join("registrations"),
        decode_momentum_raw_feature_registration_protobuf_v4,
    )?;
    let validation_yield_audit = read_single(
        &v4.join("validation_yield_audits"),
        decode_momentum_validation_yield_audit_protobuf_v4,
    )?;
    let family = read_single(
        &v4.join("families"),
        decode_momentum_raw_feature_family_protobuf_v4,
    )?;
    let mut decisions = Vec::new();
    for path in protobuf_paths(&v4.join("path_decisions"))? {
        let bytes = fs::read(path).map_err(|_| "V4.1 V4 decision read failed".to_string())?;
        if let Ok(value) = decode_momentum_raw_feature_decision_protobuf_v4(&bytes, &family) {
            decisions.push(value);
        }
    }
    if decisions.len() != 1 {
        return Err("V4.1 corrected V4 decision identity rejected".to_string());
    }
    let decision = decisions.remove(0);
    if registration.split_digest != split.split_digest
        || registration.source_snapshot_digest != family.source_snapshot_digest
        || validation_yield_audit.source_snapshot_digest != family.source_snapshot_digest
        || decision.family_digest != family.family_digest
    {
        return Err("V4.1 frozen V4 cross-binding rejected".to_string());
    }
    Ok(FrozenV4Artifacts {
        closure,
        split,
        registration,
        validation_yield_audit,
        family,
        decision,
    })
}

fn persist_artifact(
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

fn persist_registration(
    root: &Path,
    value: &MomentumSupplementalQualificationRegistrationV4_1,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("registrations")
            .join(format!("{}.pb", value.registration_digest)),
        &encode_momentum_supplemental_registration_protobuf_v4_1(value)?,
        &value.registration_digest,
        |bytes| {
            Ok(decode_momentum_supplemental_registration_protobuf_v4_1(bytes)?.registration_digest)
        },
    )
}

fn persist_opening(
    root: &Path,
    value: &MomentumReserveOpeningReceiptV4_1,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("reserve_opening_receipts")
            .join(format!("{}.pb", value.receipt_digest)),
        &encode_momentum_reserve_opening_protobuf_v4_1(value)?,
        &value.receipt_digest,
        |bytes| Ok(decode_momentum_reserve_opening_protobuf_v4_1(bytes)?.receipt_digest),
    )
}

fn persist_result(root: &Path, result: &SupplementalResultV4_1) -> Result<(usize, usize), String> {
    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_registration(root, &result.registration)?,
    );
    add_counts(&mut counts, persist_opening(root, &result.ready_receipt)?);
    add_counts(&mut counts, persist_opening(root, &result.opened_receipt)?);
    add_counts(
        &mut counts,
        persist_artifact(
            &root
                .join("supplemental_yields")
                .join(format!("{}.pb", result.yield_result.yield_digest)),
            &encode_momentum_supplemental_yield_protobuf_v4_1(&result.yield_result)?,
            &result.yield_result.yield_digest,
            |bytes| Ok(decode_momentum_supplemental_yield_protobuf_v4_1(bytes)?.yield_digest),
        )?,
    );
    for receipt in &result.family.accumulated_receipts {
        add_counts(
            &mut counts,
            persist_artifact(
                &root
                    .join("accumulated_receipts")
                    .join(format!("{}.pb", receipt.receipt_digest)),
                &encode_momentum_accumulated_receipt_protobuf_v4_1(receipt)?,
                &receipt.receipt_digest,
                |bytes| Ok(decode_momentum_accumulated_receipt_protobuf_v4_1(bytes)?.receipt_digest),
            )?,
        );
    }
    add_counts(
        &mut counts,
        persist_artifact(
            &root
                .join("accumulated_interaction_audits")
                .join(format!("{}.pb", result.interaction_audit.audit_digest)),
            &encode_momentum_accumulated_contribution_protobuf_v4_1(&result.interaction_audit)?,
            &result.interaction_audit.audit_digest,
            |bytes| Ok(decode_momentum_accumulated_contribution_protobuf_v4_1(bytes)?.audit_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_artifact(
            &root
                .join("accumulated_families")
                .join(format!("{}.pb", result.family.family_digest)),
            &encode_momentum_accumulated_family_protobuf_v4_1(&result.family)?,
            &result.family.family_digest,
            |bytes| Ok(decode_momentum_accumulated_family_protobuf_v4_1(bytes)?.family_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_artifact(
            &root
                .join("accumulated_decisions")
                .join(format!("{}.pb", result.decision.decision_digest)),
            &encode_momentum_accumulated_decision_protobuf_v4_1(&result.decision)?,
            &result.decision.decision_digest,
            |bytes| Ok(decode_momentum_accumulated_decision_protobuf_v4_1(bytes)?.decision_digest),
        )?,
    );
    if let Some(roster) = &result.roster {
        add_counts(
            &mut counts,
            persist_artifact(
                &root
                    .join("rosters")
                    .join(format!("{}.pb", roster.roster_digest)),
                &encode_momentum_accumulated_roster_protobuf_v4_1(roster)?,
                &roster.roster_digest,
                |bytes| Ok(decode_momentum_accumulated_roster_protobuf_v4_1(bytes)?.roster_digest),
            )?,
        );
    }
    if let Some(evaluation) = &result.evaluation {
        add_counts(
            &mut counts,
            persist_artifact(
                &root
                    .join("evaluation_registrations")
                    .join(format!("{}.pb", evaluation.registration_digest)),
                &encode_momentum_accumulated_evaluation_protobuf_v4_1(evaluation)?,
                &evaluation.registration_digest,
                |bytes| {
                    Ok(decode_momentum_accumulated_evaluation_protobuf_v4_1(bytes)?
                        .registration_digest)
                },
            )?,
        );
    }
    if let Some(requirement) = &result.additional_requirement {
        add_counts(
            &mut counts,
            persist_artifact(
                &root
                    .join("additional_evidence_requirements")
                    .join(format!("{}.pb", requirement.requirement_digest)),
                &encode_momentum_additional_requirement_protobuf_v4_1(requirement)?,
                &requirement.requirement_digest,
                |bytes| {
                    Ok(decode_momentum_additional_requirement_protobuf_v4_1(bytes)?
                        .requirement_digest)
                },
            )?,
        );
    }
    add_counts(
        &mut counts,
        persist_artifact(
            &root
                .join("journals")
                .join(format!("{}.pb", result.journal.journal_digest)),
            &encode_momentum_supplemental_journal_protobuf_v4_1(&result.journal)?,
            &result.journal.journal_digest,
            |bytes| Ok(decode_momentum_supplemental_journal_protobuf_v4_1(bytes)?.journal_digest),
        )?,
    );
    Ok(counts)
}

fn reopen_result(
    root: &Path,
    frozen: &FrozenV4Artifacts,
) -> Result<SupplementalResultV4_1, String> {
    let registration = read_single(
        &root.join("registrations"),
        decode_momentum_supplemental_registration_protobuf_v4_1,
    )?;
    let expected_registration = derive_registration(frozen)?;
    if registration != expected_registration {
        return Err("V4.1 reopened registration mismatch".to_string());
    }
    let mut ready_receipt = None;
    let mut opened_receipt = None;
    for path in protobuf_paths(&root.join("reserve_opening_receipts"))? {
        let value = decode_momentum_reserve_opening_protobuf_v4_1(
            &fs::read(path).map_err(|_| "V4.1 opening receipt read failed".to_string())?,
        )?;
        match value.status {
            MomentumReserveOpeningStatusV4_1::Ready => {
                if ready_receipt.replace(value).is_some() {
                    return Err("V4.1 duplicate ready opening receipt rejected".to_string());
                }
            }
            MomentumReserveOpeningStatusV4_1::Opened => {
                if opened_receipt.replace(value).is_some() {
                    return Err("V4.1 duplicate opened receipt rejected".to_string());
                }
            }
            _ => return Err("V4.1 persisted opening status rejected".to_string()),
        }
    }
    let ready_receipt =
        ready_receipt.ok_or_else(|| "V4.1 ready opening receipt unavailable".to_string())?;
    let opened_receipt =
        opened_receipt.ok_or_else(|| "V4.1 opened receipt unavailable".to_string())?;
    let expected_ready =
        opening_receipt(&registration, MomentumReserveOpeningStatusV4_1::Ready, 0)?;
    let expected_opened = opening_receipt(
        &registration,
        MomentumReserveOpeningStatusV4_1::Opened,
        registration
            .supplemental_validation_range
            .end
            .checked_sub(registration.supplemental_validation_range.start)
            .ok_or_else(|| "V4.1 reserve range rejected".to_string())?,
    )?;
    if ready_receipt != expected_ready || opened_receipt != expected_opened {
        return Err("V4.1 opening receipt cross-binding rejected".to_string());
    }
    let yield_result = read_single(
        &root.join("supplemental_yields"),
        decode_momentum_supplemental_yield_protobuf_v4_1,
    )?;
    let family = read_single(
        &root.join("accumulated_families"),
        decode_momentum_accumulated_family_protobuf_v4_1,
    )?;
    let persisted_receipts = protobuf_paths(&root.join("accumulated_receipts"))?
        .into_iter()
        .map(|path| {
            decode_momentum_accumulated_receipt_protobuf_v4_1(
                &fs::read(path).map_err(|_| "V4.1 receipt read failed".to_string())?,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    if persisted_receipts.len() != 3
        || persisted_receipts
            .iter()
            .map(|item| item.receipt_digest.clone())
            .collect::<BTreeSet<_>>()
            != family
                .accumulated_receipts
                .iter()
                .map(|item| item.receipt_digest.clone())
                .collect::<BTreeSet<_>>()
    {
        return Err("V4.1 persisted receipt set rejected".to_string());
    }
    let interaction_audit = read_single(
        &root.join("accumulated_interaction_audits"),
        decode_momentum_accumulated_contribution_protobuf_v4_1,
    )?;
    let decision = read_single(
        &root.join("accumulated_decisions"),
        decode_momentum_accumulated_decision_protobuf_v4_1,
    )?;
    validate_decision(
        &decision,
        frozen,
        &family,
        &yield_result,
        &interaction_audit,
    )?;
    let (expected_roster, expected_roster_status) =
        derive_roster(frozen, &family, &yield_result, &interaction_audit)?;
    let roster = if let Some(expected) = expected_roster {
        let persisted = read_single(
            &root.join("rosters"),
            decode_momentum_accumulated_roster_protobuf_v4_1,
        )?;
        if persisted != expected {
            return Err("V4.1 reopened roster mismatch".to_string());
        }
        Some(persisted)
    } else {
        if root.join("rosters").exists() {
            return Err("V4.1 unexpected roster artifact rejected".to_string());
        }
        None
    };
    let (evaluation, evaluation_status) = if roster.is_some() {
        (
            Some(read_single(
                &root.join("evaluation_registrations"),
                decode_momentum_accumulated_evaluation_protobuf_v4_1,
            )?),
            MomentumAccumulatedEvaluationStatusV4_1::Registered,
        )
    } else {
        if root.join("evaluation_registrations").exists() {
            return Err("V4.1 unexpected evaluation artifact rejected".to_string());
        }
        let status = match expected_roster_status {
            MomentumAccumulatedRosterStatusV4_1::StillInsufficientValidation => {
                MomentumAccumulatedEvaluationStatusV4_1::StillInsufficientValidation
            }
            MomentumAccumulatedRosterStatusV4_1::BenchmarkUnavailable => {
                MomentumAccumulatedEvaluationStatusV4_1::BenchmarkUnavailable
            }
            MomentumAccumulatedRosterStatusV4_1::SemanticDuplicateOnly => {
                MomentumAccumulatedEvaluationStatusV4_1::SemanticDuplicateOnly
            }
            _ => MomentumAccumulatedEvaluationStatusV4_1::NoQualifiedLearnedParticipant,
        };
        (None, status)
    };
    let additional_requirement = if yield_result.minimum_reached {
        if root.join("additional_evidence_requirements").exists() {
            return Err("V4.1 unexpected evidence requirement rejected".to_string());
        }
        None
    } else {
        Some(read_single(
            &root.join("additional_evidence_requirements"),
            decode_momentum_additional_requirement_protobuf_v4_1,
        )?)
    };
    let journal = read_single(
        &root.join("journals"),
        decode_momentum_supplemental_journal_protobuf_v4_1,
    )?;
    let required_validation_identities = [
        &frozen.closure.v1_family_digest,
        &frozen.closure.v2_family_digest,
        &frozen.closure.v3_family_digest,
        &frozen.family.family_digest,
        &frozen.validation_yield_audit.audit_digest,
    ];
    let evaluation_valid = evaluation.as_ref().is_none_or(|value| {
        value.source_v4_family_digest == frozen.family.family_digest
            && value.accumulated_family_digest == family.family_digest
            && roster
                .as_ref()
                .is_some_and(|item| value.roster_digest == item.roster_digest)
            && value.supplemental_registration_digest == registration.registration_digest
            && value.reserve_opening_receipt_digest == opened_receipt.receipt_digest
            && value.accumulated_yield_digest == yield_result.yield_digest
            && value.accumulated_receipt_digests
                == sorted_unique(
                    family
                        .accumulated_receipts
                        .iter()
                        .map(|item| item.receipt_digest.clone())
                        .collect(),
                )
            && value.accumulated_interaction_audit_digest
                == Some(interaction_audit.audit_digest.clone())
            && value.source_snapshot_digest == registration.source_snapshot_digest
            && required_validation_identities
                .iter()
                .all(|digest| value.consumed_validation_identity_digests.contains(digest))
    });
    let requirement_valid = additional_requirement.as_ref().is_none_or(|value| {
        value.accumulated_valid_sample_count == yield_result.accumulated_valid_sample_count
            && value.minimum_required_valid_samples == yield_result.minimum_required_valid_samples
            && value.minimum_additional_valid_samples
                == yield_result
                    .minimum_required_valid_samples
                    .saturating_sub(yield_result.accumulated_valid_sample_count)
    });
    if ready_receipt.supplemental_registration_digest != registration.registration_digest
        || opened_receipt.supplemental_registration_digest != registration.registration_digest
        || family.supplemental_registration_digest != registration.registration_digest
        || family.reserve_opening_receipt_digest != opened_receipt.receipt_digest
        || family.accumulated_yield_digest != yield_result.yield_digest
        || family.source_v4_family_digest != frozen.family.family_digest
        || family.participant_digests != registration.participant_digests
        || family.accumulated_interaction_audit_digest
            != Some(interaction_audit.audit_digest.clone())
        || family.accumulated_receipts.iter().any(|receipt| {
            receipt.supplemental_registration_digest != registration.registration_digest
                || receipt.reserve_opening_receipt_digest != opened_receipt.receipt_digest
                || receipt.accumulated_yield_digest != yield_result.yield_digest
        })
        || decision.accumulated_family_digest != family.family_digest
        || !evaluation_valid
        || !requirement_valid
        || journal.supplemental_registration_digest != registration.registration_digest
        || journal.ready_opening_receipt_digest != ready_receipt.receipt_digest
        || journal.opened_receipt_digest != opened_receipt.receipt_digest
        || journal.accumulated_yield_digest != yield_result.yield_digest
        || journal.accumulated_family_digest != family.family_digest
        || journal.accumulated_decision_digest != decision.decision_digest
        || journal.roster_digest != roster.as_ref().map(|value| value.roster_digest.clone())
        || journal.evaluation_registration_digest
            != evaluation
                .as_ref()
                .map(|value| value.registration_digest.clone())
        || journal.additional_evidence_requirement_digest
            != additional_requirement
                .as_ref()
                .map(|value| value.requirement_digest.clone())
    {
        return Err("V4.1 reopened result cross-binding rejected".to_string());
    }
    Ok(SupplementalResultV4_1 {
        registration,
        ready_receipt,
        opened_receipt,
        yield_result,
        interaction_audit,
        family,
        decision,
        roster,
        roster_status: expected_roster_status,
        evaluation,
        evaluation_status,
        additional_requirement,
        journal,
    })
}

pub(crate) fn reopen_momentum_v4_1_future_source(
    root: &Path,
) -> Result<MomentumFutureEvaluationSourceV4_2, String> {
    let frozen = load_frozen_v4(root)?;
    let result = reopen_result(&root.join(ROOT_VERSION_V4_1).join(AGENT_ID_V4_1), &frozen)?;
    let roster = result
        .roster
        .ok_or_else(|| "V4.2 future roster unavailable".to_string())?;
    let evaluation = result
        .evaluation
        .ok_or_else(|| "V4.2 future evaluation registration unavailable".to_string())?;
    if result.roster_status != MomentumAccumulatedRosterStatusV4_1::Ready
        || result.evaluation_status != MomentumAccumulatedEvaluationStatusV4_1::Registered
        || result.family.qualified_learned_count != 2
        || result.family.qualified_benchmark_count != 1
        || result.family.winner_selected
        || result.family.parameters_changed
        || roster.learned_participant_digests.len() != 2
        || roster.benchmark_participant_digests.len() != 1
        || evaluation.maximum_requests != 1
        || evaluation.maximum_concurrency != 1
        || evaluation.maximum_retries != 0
        || !evaluation.labels_hidden_until_opening
        || !evaluation.probabilities_hidden_until_opening
        || !evaluation.one_time_opening_required
        || !evaluation.winner_selection_forbidden_before_opening
        || !evaluation.active_promotion_forbidden
        || !evaluation.reward_application_forbidden
    {
        return Err("V4.2 frozen future source contract rejected".to_string());
    }
    Ok(MomentumFutureEvaluationSourceV4_2 {
        closure: frozen.closure,
        split: frozen.split,
        registration: frozen.registration,
        source_family: frozen.family,
        supplemental_registration: result.registration,
        accumulated_family: result.family,
        roster,
        evaluation,
    })
}

fn collect_protected(
    root: &Path,
    current: &Path,
    values: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), String> {
    if current == root.join(ROOT_VERSION_V4_1) {
        return Ok(());
    }
    if current.is_dir() {
        let mut entries = fs::read_dir(current)
            .map_err(|_| "V4.1 protected directory read failed".to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        entries.sort();
        for path in entries {
            collect_protected(root, &path, values)?;
        }
    } else if current.is_file() {
        values.push((
            current
                .strip_prefix(root)
                .map_err(|_| "V4.1 protected path rejected".to_string())?
                .to_path_buf(),
            fs::read(current).map_err(|_| "V4.1 protected artifact read failed".to_string())?,
        ));
    }
    Ok(())
}

fn base_report(
    mode: AgentPrivateLearningRunModeV0,
    status: MomentumSupplementalExecutionStatusV4_1,
    frozen: Option<&FrozenV4Artifacts>,
) -> MomentumSupplementalReportV4_1 {
    let mut value = MomentumSupplementalReportV4_1 {
        report_version: "momentum-supplemental-report-v4.1".to_string(),
        mode,
        status,
        original_validation_yield_audit: frozen.map(|item| item.validation_yield_audit.clone()),
        corrected_v4_decision: frozen.map(|item| item.decision.clone()),
        supplemental_registration: None,
        reserve_opening_status: MomentumReserveOpeningStatusV4_1::Ready,
        reserve_opening_receipt: None,
        supplemental_yield: None,
        accumulated_interaction_audit: None,
        accumulated_family: None,
        accumulated_decision: None,
        roster: None,
        roster_status: MomentumAccumulatedRosterStatusV4_1::StillInsufficientValidation,
        evaluation_registration: None,
        evaluation_registration_status:
            MomentumAccumulatedEvaluationStatusV4_1::StillInsufficientValidation,
        additional_evidence_requirement: None,
        journal: None,
        artifacts_written: 0,
        duplicate_artifact_count: 0,
        storage_failure_count: 0,
        protected_artifacts_unchanged: true,
        active_state_unchanged: true,
        safety_counters: zero_counters(),
        report_digest: String::new(),
    };
    value.report_digest = report_digest(&value);
    value
}

fn populate_report(
    report: &mut MomentumSupplementalReportV4_1,
    result: SupplementalResultV4_1,
    already_opened: bool,
) {
    report.supplemental_registration = Some(result.registration);
    report.reserve_opening_status = if already_opened {
        MomentumReserveOpeningStatusV4_1::AlreadyOpened
    } else {
        MomentumReserveOpeningStatusV4_1::Opened
    };
    report.reserve_opening_receipt = Some(result.opened_receipt);
    report.supplemental_yield = Some(result.yield_result);
    report.accumulated_interaction_audit = Some(result.interaction_audit);
    report.accumulated_family = Some(result.family);
    report.accumulated_decision = Some(result.decision);
    report.roster = result.roster;
    report.roster_status = result.roster_status;
    report.evaluation_registration = result.evaluation;
    report.evaluation_registration_status = result.evaluation_status;
    report.additional_evidence_requirement = result.additional_requirement;
    report.journal = Some(result.journal);
}

pub fn run_momentum_v4_supplemental_qualification(
    root: &Path,
    snapshots: &[DataSnapshot],
    reservation: &ProtectedEvaluationReservationV1,
    mode: AgentPrivateLearningRunModeV0,
) -> MomentumSupplementalReportV4_1 {
    let active_before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let mut protected_before = Vec::new();
    if collect_protected(root, root, &mut protected_before).is_err() {
        let mut report = base_report(
            mode,
            MomentumSupplementalExecutionStatusV4_1::TechnicalFailure,
            None,
        );
        report.protected_artifacts_unchanged = false;
        report.report_digest = report_digest(&report);
        return report;
    }
    let frozen = match load_frozen_v4(root) {
        Ok(value) => value,
        Err(_) => {
            let mut report = base_report(
                mode,
                MomentumSupplementalExecutionStatusV4_1::TechnicalFailure,
                None,
            );
            report.report_digest = report_digest(&report);
            return report;
        }
    };
    let expected_registration = match derive_registration(&frozen) {
        Ok(value) => value,
        Err(_) => {
            let mut report = base_report(
                mode,
                MomentumSupplementalExecutionStatusV4_1::TechnicalFailure,
                Some(&frozen),
            );
            report.report_digest = report_digest(&report);
            return report;
        }
    };
    let supplemental_root = root.join(ROOT_VERSION_V4_1).join(AGENT_ID_V4_1);
    if mode != AgentPrivateLearningRunModeV0::ExecuteLocal {
        let persisted = reopen_result(&supplemental_root, &frozen).ok();
        let mut report = base_report(
            mode,
            if persisted.is_some() {
                MomentumSupplementalExecutionStatusV4_1::AlreadyOpened
            } else {
                MomentumSupplementalExecutionStatusV4_1::Planned
            },
            Some(&frozen),
        );
        if let Some(result) = persisted {
            populate_report(&mut report, result, true);
        } else {
            report.supplemental_registration = Some(expected_registration);
        }
        report.active_state_unchanged =
            stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before;
        report.report_digest = report_digest(&report);
        return report;
    }

    if let Ok(result) = reopen_result(&supplemental_root, &frozen) {
        let mut report = base_report(
            mode,
            MomentumSupplementalExecutionStatusV4_1::AlreadyOpened,
            Some(&frozen),
        );
        match persist_result(&supplemental_root, &result) {
            Ok((written, duplicates)) => {
                report.artifacts_written = written;
                report.duplicate_artifact_count = duplicates;
            }
            Err(_) => report.storage_failure_count = 1,
        }
        populate_report(&mut report, result, true);
        let mut protected_after = Vec::new();
        report.protected_artifacts_unchanged = collect_protected(root, root, &mut protected_after)
            .is_ok()
            && protected_after == protected_before;
        report.active_state_unchanged =
            stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before;
        report.report_digest = report_digest(&report);
        return report;
    }

    if supplemental_root.join("reserve_opening_receipts").exists() {
        let mut report = base_report(
            mode,
            MomentumSupplementalExecutionStatusV4_1::TechnicalFailure,
            Some(&frozen),
        );
        report.supplemental_registration = Some(expected_registration);
        report.reserve_opening_status = MomentumReserveOpeningStatusV4_1::IntegrityFailure;
        report.report_digest = report_digest(&report);
        return report;
    }
    let mut counts = (0, 0);
    match persist_registration(&supplemental_root, &expected_registration) {
        Ok(value) => add_counts(&mut counts, value),
        Err(_) => {
            let mut report = base_report(
                mode,
                MomentumSupplementalExecutionStatusV4_1::TechnicalFailure,
                Some(&frozen),
            );
            report.storage_failure_count = 1;
            report.report_digest = report_digest(&report);
            return report;
        }
    }
    let reopened_registration = match read_single(
        &supplemental_root.join("registrations"),
        decode_momentum_supplemental_registration_protobuf_v4_1,
    ) {
        Ok(value) if value == expected_registration => value,
        _ => {
            let mut report = base_report(
                mode,
                MomentumSupplementalExecutionStatusV4_1::TechnicalFailure,
                Some(&frozen),
            );
            report.storage_failure_count = 1;
            report.report_digest = report_digest(&report);
            return report;
        }
    };
    let replay = match reconstruct_frozen_momentum_v4(root, snapshots, reservation) {
        Ok(value) => value,
        Err(_) => {
            let mut report = base_report(
                mode,
                MomentumSupplementalExecutionStatusV4_1::TechnicalFailure,
                Some(&frozen),
            );
            report.supplemental_registration = Some(reopened_registration);
            report.report_digest = report_digest(&report);
            return report;
        }
    };
    let ready_receipt = match opening_receipt(
        &reopened_registration,
        MomentumReserveOpeningStatusV4_1::Ready,
        0,
    ) {
        Ok(value) => value,
        Err(_) => {
            let mut report = base_report(
                mode,
                MomentumSupplementalExecutionStatusV4_1::TechnicalFailure,
                Some(&frozen),
            );
            report.report_digest = report_digest(&report);
            return report;
        }
    };
    match persist_opening(&supplemental_root, &ready_receipt) {
        Ok(value) => add_counts(&mut counts, value),
        Err(_) => {
            let mut report = base_report(
                mode,
                MomentumSupplementalExecutionStatusV4_1::TechnicalFailure,
                Some(&frozen),
            );
            report.storage_failure_count = 1;
            report.report_digest = report_digest(&report);
            return report;
        }
    }
    let accumulated_evaluation = match evaluate_frozen_momentum_v4_accumulated(&replay) {
        Ok(value) => value,
        Err(_) => {
            let mut report = base_report(
                mode,
                MomentumSupplementalExecutionStatusV4_1::TechnicalFailure,
                Some(&frozen),
            );
            report.reserve_opening_status = MomentumReserveOpeningStatusV4_1::IntegrityFailure;
            report.report_digest = report_digest(&report);
            return report;
        }
    };
    let result = match derive_result(
        &frozen,
        &replay,
        reopened_registration,
        accumulated_evaluation,
        reservation,
    ) {
        Ok(value) if value.ready_receipt == ready_receipt => value,
        _ => {
            let mut report = base_report(
                mode,
                MomentumSupplementalExecutionStatusV4_1::TechnicalFailure,
                Some(&frozen),
            );
            report.reserve_opening_status = MomentumReserveOpeningStatusV4_1::IntegrityFailure;
            report.report_digest = report_digest(&report);
            return report;
        }
    };
    match persist_result(&supplemental_root, &result) {
        Ok(value) => add_counts(&mut counts, value),
        Err(_) => {
            let mut report = base_report(
                mode,
                MomentumSupplementalExecutionStatusV4_1::TechnicalFailure,
                Some(&frozen),
            );
            report.storage_failure_count = 1;
            report.report_digest = report_digest(&report);
            return report;
        }
    }
    let reopened = match reopen_result(&supplemental_root, &frozen) {
        Ok(value) if value.journal == result.journal => value,
        _ => {
            let mut report = base_report(
                mode,
                MomentumSupplementalExecutionStatusV4_1::TechnicalFailure,
                Some(&frozen),
            );
            report.storage_failure_count = 1;
            report.report_digest = report_digest(&report);
            return report;
        }
    };
    let mut report = base_report(
        mode,
        MomentumSupplementalExecutionStatusV4_1::Executed,
        Some(&frozen),
    );
    report.artifacts_written = counts.0;
    report.duplicate_artifact_count = counts.1;
    report.safety_counters.reserve_opening_attempts = 1;
    report.safety_counters.reserve_row_reads = reopened.opened_receipt.opened_index_count;
    report.safety_counters.reserve_label_reads = reopened.opened_receipt.opened_index_count;
    populate_report(&mut report, reopened, false);
    let mut protected_after = Vec::new();
    report.protected_artifacts_unchanged = collect_protected(root, root, &mut protected_after)
        .is_ok()
        && protected_after == protected_before;
    report.active_state_unchanged =
        stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before;
    report.report_digest = report_digest(&report);
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registration_fixture() -> MomentumSupplementalQualificationRegistrationV4_1 {
        let mut value = MomentumSupplementalQualificationRegistrationV4_1 {
            registration_version: REGISTRATION_VERSION_V4_1.to_string(),
            agent_id: AGENT_ID_V4_1.to_string(),
            source_snapshot_digest: "snapshot".to_string(),
            canonical_intent_digest: "intent".to_string(),
            canonical_view_digest: "view".to_string(),
            v4_split_digest: "split".to_string(),
            v4_registration_digest: "v4-registration".to_string(),
            v4_family_digest: "v4-family".to_string(),
            validation_yield_audit_digest: "yield-audit".to_string(),
            participant_digests: vec![
                "raw".to_string(),
                "interaction".to_string(),
                "benchmark".to_string(),
            ],
            participant_parameter_digests: vec![
                "raw-parameter".to_string(),
                "interaction-parameter".to_string(),
                "benchmark-parameter".to_string(),
            ],
            participant_normalizer_digests: vec![
                "raw-normalizer".to_string(),
                "interaction-normalizer".to_string(),
                "benchmark-normalizer".to_string(),
            ],
            original_validation_range: IndexRangeV0 {
                start: 264,
                end: 288,
            },
            supplemental_validation_range: IndexRangeV0 {
                start: 288,
                end: 312,
            },
            accumulated_validation_range_digest: "accumulated-ranges".to_string(),
            minimum_required_valid_samples: 24,
            model_retraining_forbidden: true,
            parameter_updates_forbidden: true,
            configuration_changes_forbidden: true,
            final_reserve_opening_one_time_only: true,
            historical_test_forbidden: true,
            future_evaluation_forbidden: true,
            winner_selection_forbidden: true,
            promotion_forbidden: true,
            reward_application_forbidden: true,
            registration_digest: String::new(),
        };
        value.registration_digest = registration_digest(&value);
        value
    }

    fn yield_fixture(
        original: usize,
        supplemental: usize,
    ) -> MomentumSupplementalValidationYieldV4_1 {
        let accumulated = original + supplemental;
        let mut value = MomentumSupplementalValidationYieldV4_1 {
            original_valid_sample_count: original,
            supplemental_valid_sample_count: supplemental,
            accumulated_valid_sample_count: accumulated,
            original_neutral_excluded_count: 24usize.saturating_sub(original),
            supplemental_neutral_excluded_count: 24usize.saturating_sub(supplemental),
            minimum_required_valid_samples: 24,
            minimum_reached: accumulated >= 24,
            yield_digest: String::new(),
        };
        value.yield_digest = yield_digest(&value);
        value
    }

    fn accumulated_receipt(
        participant: &str,
        status: MomentumAccumulatedQualificationStatusV4_1,
    ) -> MomentumAccumulatedQualificationReceiptV4_1 {
        let mut value = MomentumAccumulatedQualificationReceiptV4_1 {
            receipt_version: RECEIPT_VERSION_V4_1.to_string(),
            participant_digest: participant.to_string(),
            v4_original_receipt_digest: format!("{participant}-original"),
            supplemental_registration_digest: "supplemental-registration".to_string(),
            reserve_opening_receipt_digest: "opened".to_string(),
            accumulated_yield_digest: "yield".to_string(),
            accumulated_validation_identity_digest: "accumulated-validation".to_string(),
            qualification_policy_digest: "qualification-policy".to_string(),
            private_metric_digest: format!("{participant}-private"),
            status,
            parameter_updates_after_freeze: 0,
            historical_test_reads: 0,
            future_evaluation_reads: 0,
            receipt_digest: String::new(),
        };
        value.receipt_digest = receipt_digest(&value);
        value
    }

    fn contribution_fixture() -> MomentumAccumulatedInteractionContributionAuditV4_1 {
        let mut value = MomentumAccumulatedInteractionContributionAuditV4_1 {
            participant_digest: "interaction".to_string(),
            original_contribution_audit_digest: "original-audit".to_string(),
            accumulated_validation_identity_digest: "accumulated-validation".to_string(),
            full_prediction_digest: "full".to_string(),
            nonlinear_ablated_prediction_digest: "ablated".to_string(),
            contribution_policy_digest: "policy".to_string(),
            contribution_status: InteractionContributionStatusV4::MaterialInteractionContribution,
            audit_digest: String::new(),
        };
        value.audit_digest = contribution_digest(&value);
        value
    }

    fn family_fixture() -> MomentumAccumulatedQualificationFamilyV4_1 {
        let receipts = vec![
            accumulated_receipt(
                "raw",
                MomentumAccumulatedQualificationStatusV4_1::QualifiedLearned,
            ),
            accumulated_receipt(
                "interaction",
                MomentumAccumulatedQualificationStatusV4_1::QualifiedLearned,
            ),
            accumulated_receipt(
                "benchmark",
                MomentumAccumulatedQualificationStatusV4_1::BenchmarkQualified,
            ),
        ];
        let mut value = MomentumAccumulatedQualificationFamilyV4_1 {
            family_version: FAMILY_VERSION_V4_1.to_string(),
            source_v4_family_digest: "v4-family".to_string(),
            supplemental_registration_digest: "supplemental-registration".to_string(),
            reserve_opening_receipt_digest: "opened".to_string(),
            accumulated_yield_digest: "yield".to_string(),
            participant_digests: vec![
                "raw".to_string(),
                "interaction".to_string(),
                "benchmark".to_string(),
            ],
            accumulated_receipts: receipts,
            accumulated_interaction_audit_digest: Some("interaction-audit".to_string()),
            qualified_learned_count: 2,
            qualified_benchmark_count: 1,
            winner_selected: false,
            parameters_changed: false,
            eligible_for_active_committee: false,
            eligible_for_promotion: false,
            eligible_for_reward: false,
            family_digest: String::new(),
        };
        value.family_digest = family_digest(&value);
        value
    }

    fn decision_fixture() -> MomentumAccumulatedPathDecisionArtifactV4_1 {
        let mut value = MomentumAccumulatedPathDecisionArtifactV4_1 {
            decision_version: DECISION_VERSION_V4_1.to_string(),
            accumulated_family_digest: "family".to_string(),
            minimum_reached: true,
            qualified_raw_logistic: true,
            qualified_material_interaction: true,
            decision: MomentumAccumulatedPathDecisionV4_1::RawFeatureLearnedPathViable,
            decision_digest: String::new(),
        };
        value.decision_digest = decision_digest(&value);
        value
    }

    fn roster_fixture() -> MomentumAccumulatedFutureRosterV4_1 {
        let mut value = MomentumAccumulatedFutureRosterV4_1 {
            roster_version: ROSTER_VERSION_V4_1.to_string(),
            accumulated_family_digest: "family".to_string(),
            learned_participant_digests: vec!["raw".to_string(), "interaction".to_string()],
            benchmark_participant_digests: vec!["benchmark".to_string()],
            excluded_semantic_duplicate_digests: vec![],
            rejected_participant_digests: vec![],
            inclusion_policy_digest: "inclusion".to_string(),
            status: MomentumAccumulatedRosterStatusV4_1::Ready,
            roster_digest: String::new(),
        };
        value.roster_digest = roster_digest(&value);
        value
    }

    fn evaluation_fixture() -> MomentumAccumulatedEvaluationRegistrationV4_1 {
        let mut value = MomentumAccumulatedEvaluationRegistrationV4_1 {
            registration_version: EVALUATION_VERSION_V4_1.to_string(),
            agent_id: AGENT_ID_V4_1.to_string(),
            source_v4_family_digest: "v4-family".to_string(),
            accumulated_family_digest: "family".to_string(),
            roster_digest: "roster".to_string(),
            supplemental_registration_digest: "supplemental".to_string(),
            reserve_opening_receipt_digest: "opening".to_string(),
            accumulated_yield_digest: "yield".to_string(),
            accumulated_receipt_digests: vec![
                "raw-receipt".to_string(),
                "interaction-receipt".to_string(),
                "benchmark-receipt".to_string(),
            ],
            accumulated_interaction_audit_digest: Some("interaction-audit".to_string()),
            source_snapshot_digest: "snapshot".to_string(),
            source_boundary_timestamp_ms: 100,
            consumed_validation_identity_digests: vec!["v1".to_string(), "v4".to_string()],
            protected_registration_digests: vec!["protected".to_string()],
            protected_timestamp_ms: vec![80, 90],
            provider_finality_boundary_ms: 90,
            minimum_accepted_timestamp_ms: 110,
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
        value.registration_digest = evaluation_digest(&value);
        value
    }

    fn requirement_fixture() -> MomentumAdditionalEvidenceRequirementV4_1 {
        let mut value = MomentumAdditionalEvidenceRequirementV4_1 {
            requirement_version: REQUIREMENT_VERSION_V4_1.to_string(),
            accumulated_valid_sample_count: 20,
            minimum_required_valid_samples: 24,
            minimum_additional_valid_samples: 4,
            current_source_boundary_timestamp_ms: 100,
            required_dataset_kind: DatasetKind::CryptoDailyOhlcv,
            market_scope: AcquisitionMarketScope::BtcCrypto,
            symbols: vec!["BTC".to_string()],
            cadence: "1d".to_string(),
            existing_evidence_fully_consumed_for_qualification: true,
            new_evidence_identity_required: true,
            separate_acquisition_preregistration_required: true,
            requirement_digest: String::new(),
        };
        value.requirement_digest = requirement_digest(&value);
        value
    }

    fn journal_fixture() -> MomentumSupplementalJournalV4_1 {
        let mut value = MomentumSupplementalJournalV4_1 {
            journal_version: JOURNAL_VERSION_V4_1.to_string(),
            agent_id: AGENT_ID_V4_1.to_string(),
            supplemental_registration_digest: "registration".to_string(),
            ready_opening_receipt_digest: "ready".to_string(),
            opened_receipt_digest: "opened".to_string(),
            accumulated_yield_digest: "yield".to_string(),
            accumulated_family_digest: "family".to_string(),
            accumulated_decision_digest: "decision".to_string(),
            roster_digest: Some("roster".to_string()),
            evaluation_registration_digest: Some("evaluation".to_string()),
            additional_evidence_requirement_digest: None,
            registration_reopened_before_reserve_access: true,
            frozen_participants_reconstructed: true,
            participant_parameters_unchanged: true,
            normalizers_unchanged: true,
            status: MomentumSupplementalExecutionStatusV4_1::Executed,
            journal_digest: String::new(),
        };
        value.journal_digest = journal_digest(&value);
        value
    }

    fn temporary_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "soma-v4-1-test-{}",
            stable_hash_string(&format!("{:?}", std::time::SystemTime::now()))
        ))
    }

    #[test]
    fn unchanged_minimum_is_twenty_four() {
        let registration = registration_fixture();
        assert_eq!(registration.minimum_required_valid_samples, 24);
        assert!(validate_registration(&registration).is_ok());
    }

    #[test]
    fn supplemental_range_follows_original_without_overlap() {
        let registration = registration_fixture();
        assert_eq!(
            registration.original_validation_range.end,
            registration.supplemental_validation_range.start
        );
    }

    #[test]
    fn exactly_three_frozen_participants_are_registered() {
        assert_eq!(registration_fixture().participant_digests.len(), 3);
    }

    #[test]
    fn duplicate_participant_identity_rejects() {
        let mut value = registration_fixture();
        value.participant_digests[1] = value.participant_digests[0].clone();
        value.registration_digest = registration_digest(&value);
        assert!(validate_registration(&value).is_err());
    }

    #[test]
    fn configuration_mutation_rejects() {
        let mut value = registration_fixture();
        value.configuration_changes_forbidden = false;
        value.registration_digest = registration_digest(&value);
        assert!(validate_registration(&value).is_err());
    }

    #[test]
    fn parameter_update_permission_rejects() {
        let mut value = registration_fixture();
        value.parameter_updates_forbidden = false;
        value.registration_digest = registration_digest(&value);
        assert!(validate_registration(&value).is_err());
    }

    #[test]
    fn frozen_parameter_digest_mismatch_rejects() {
        let registration = registration_fixture();
        let mut parameter_digests = registration.participant_parameter_digests.clone();
        parameter_digests[0].push_str("-changed");
        assert!(!identity_vectors_match(
            &registration,
            &parameter_digests,
            &registration.participant_normalizer_digests
        ));
    }

    #[test]
    fn frozen_normalizer_digest_mismatch_rejects() {
        let registration = registration_fixture();
        let mut normalizer_digests = registration.participant_normalizer_digests.clone();
        normalizer_digests[1].push_str("-changed");
        assert!(!identity_vectors_match(
            &registration,
            &registration.participant_parameter_digests,
            &normalizer_digests
        ));
    }

    #[test]
    fn ready_receipt_precedes_opened_receipt() {
        let registration = registration_fixture();
        let ready =
            opening_receipt(&registration, MomentumReserveOpeningStatusV4_1::Ready, 0).unwrap();
        let opened =
            opening_receipt(&registration, MomentumReserveOpeningStatusV4_1::Opened, 24).unwrap();
        assert_ne!(ready.receipt_digest, opened.receipt_digest);
        assert_eq!(ready.opening_attempt_count, 1);
        assert_eq!(opened.opened_index_count, 24);
    }

    #[test]
    fn already_opened_is_runtime_only_not_persistable() {
        let registration = registration_fixture();
        assert!(
            opening_receipt(
                &registration,
                MomentumReserveOpeningStatusV4_1::AlreadyOpened,
                24
            )
            .is_err()
        );
    }

    #[test]
    fn accumulated_yield_is_exact_union() {
        let value = yield_fixture(23, 22);
        assert_eq!(value.accumulated_valid_sample_count, 45);
        assert!(value.minimum_reached);
    }

    #[test]
    fn insufficient_union_does_not_lower_minimum() {
        let value = yield_fixture(10, 10);
        assert_eq!(value.minimum_required_valid_samples, 24);
        assert!(!value.minimum_reached);
    }

    #[test]
    fn accumulated_receipt_has_zero_updates_and_external_reads() {
        let value = accumulated_receipt(
            "raw",
            MomentumAccumulatedQualificationStatusV4_1::QualifiedLearned,
        );
        assert_eq!(
            value.parameter_updates_after_freeze
                + value.historical_test_reads
                + value.future_evaluation_reads,
            0
        );
    }

    #[test]
    fn constant_benchmark_status_is_distinct() {
        assert_eq!(
            map_status(MomentumRawFeatureQualificationStatusV4::BenchmarkQualified),
            MomentumAccumulatedQualificationStatusV4_1::BenchmarkQualified
        );
    }

    #[test]
    fn linear_equivalent_status_remains_distinct() {
        assert_eq!(
            map_status(MomentumRawFeatureQualificationStatusV4::QualifiedLinearEquivalent),
            MomentumAccumulatedQualificationStatusV4_1::QualifiedLinearEquivalent
        );
    }

    #[test]
    fn accumulated_contribution_is_additive() {
        let value = contribution_fixture();
        assert!(!value.original_contribution_audit_digest.is_empty());
        assert_eq!(
            value.contribution_status,
            InteractionContributionStatusV4::MaterialInteractionContribution
        );
    }

    #[test]
    fn family_never_selects_winner_or_changes_parameters() {
        let value = family_fixture();
        assert!(!value.winner_selected);
        assert!(!value.parameters_changed);
        assert!(!value.eligible_for_active_committee);
    }

    #[test]
    fn roster_contains_all_fixture_qualifiers_without_ranking() {
        let value = roster_fixture();
        assert_eq!(value.learned_participant_digests.len(), 2);
        assert_eq!(value.benchmark_participant_digests.len(), 1);
    }

    #[test]
    fn evaluation_contract_has_one_request_and_zero_retries() {
        let value = evaluation_fixture();
        assert_eq!(value.maximum_requests, 1);
        assert_eq!(value.maximum_concurrency, 1);
        assert_eq!(value.maximum_retries, 0);
        assert!(value.minimum_accepted_timestamp_ms > value.source_boundary_timestamp_ms);
    }

    #[test]
    fn additional_evidence_requirement_derives_gap() {
        let value = requirement_fixture();
        assert_eq!(value.minimum_additional_valid_samples, 4);
        assert!(value.new_evidence_identity_required);
        assert!(value.separate_acquisition_preregistration_required);
    }

    #[test]
    fn all_manual_protobuf_contracts_round_trip() {
        let registration = registration_fixture();
        let ready =
            opening_receipt(&registration, MomentumReserveOpeningStatusV4_1::Ready, 0).unwrap();
        let yield_result = yield_fixture(23, 22);
        let receipt = accumulated_receipt(
            "raw",
            MomentumAccumulatedQualificationStatusV4_1::QualifiedLearned,
        );
        let contribution = contribution_fixture();
        let family = family_fixture();
        let decision = decision_fixture();
        let roster = roster_fixture();
        let evaluation = evaluation_fixture();
        let requirement = requirement_fixture();
        let journal = journal_fixture();
        assert_eq!(
            decode_momentum_supplemental_registration_protobuf_v4_1(
                &encode_momentum_supplemental_registration_protobuf_v4_1(&registration).unwrap()
            )
            .unwrap(),
            registration
        );
        assert_eq!(
            decode_momentum_reserve_opening_protobuf_v4_1(
                &encode_momentum_reserve_opening_protobuf_v4_1(&ready).unwrap()
            )
            .unwrap(),
            ready
        );
        assert_eq!(
            decode_momentum_supplemental_yield_protobuf_v4_1(
                &encode_momentum_supplemental_yield_protobuf_v4_1(&yield_result).unwrap()
            )
            .unwrap(),
            yield_result
        );
        assert_eq!(
            decode_momentum_accumulated_receipt_protobuf_v4_1(
                &encode_momentum_accumulated_receipt_protobuf_v4_1(&receipt).unwrap()
            )
            .unwrap(),
            receipt
        );
        assert_eq!(
            decode_momentum_accumulated_contribution_protobuf_v4_1(
                &encode_momentum_accumulated_contribution_protobuf_v4_1(&contribution).unwrap()
            )
            .unwrap(),
            contribution
        );
        assert_eq!(
            decode_momentum_accumulated_family_protobuf_v4_1(
                &encode_momentum_accumulated_family_protobuf_v4_1(&family).unwrap()
            )
            .unwrap(),
            family
        );
        assert_eq!(
            decode_momentum_accumulated_decision_protobuf_v4_1(
                &encode_momentum_accumulated_decision_protobuf_v4_1(&decision).unwrap()
            )
            .unwrap(),
            decision
        );
        assert_eq!(
            decode_momentum_accumulated_roster_protobuf_v4_1(
                &encode_momentum_accumulated_roster_protobuf_v4_1(&roster).unwrap()
            )
            .unwrap(),
            roster
        );
        assert_eq!(
            decode_momentum_accumulated_evaluation_protobuf_v4_1(
                &encode_momentum_accumulated_evaluation_protobuf_v4_1(&evaluation).unwrap()
            )
            .unwrap(),
            evaluation
        );
        assert_eq!(
            decode_momentum_additional_requirement_protobuf_v4_1(
                &encode_momentum_additional_requirement_protobuf_v4_1(&requirement).unwrap()
            )
            .unwrap(),
            requirement
        );
        assert_eq!(
            decode_momentum_supplemental_journal_protobuf_v4_1(
                &encode_momentum_supplemental_journal_protobuf_v4_1(&journal).unwrap()
            )
            .unwrap(),
            journal
        );
    }

    #[test]
    fn protobuf_corruption_rejects_every_category() {
        assert!(decode_momentum_supplemental_registration_protobuf_v4_1(&[0xff]).is_err());
        assert!(decode_momentum_reserve_opening_protobuf_v4_1(&[0xff]).is_err());
        assert!(decode_momentum_supplemental_yield_protobuf_v4_1(&[0xff]).is_err());
        assert!(decode_momentum_accumulated_receipt_protobuf_v4_1(&[0xff]).is_err());
        assert!(decode_momentum_accumulated_contribution_protobuf_v4_1(&[0xff]).is_err());
        assert!(decode_momentum_accumulated_family_protobuf_v4_1(&[0xff]).is_err());
        assert!(decode_momentum_accumulated_decision_protobuf_v4_1(&[0xff]).is_err());
        assert!(decode_momentum_accumulated_roster_protobuf_v4_1(&[0xff]).is_err());
        assert!(decode_momentum_accumulated_evaluation_protobuf_v4_1(&[0xff]).is_err());
        assert!(decode_momentum_additional_requirement_protobuf_v4_1(&[0xff]).is_err());
        assert!(decode_momentum_supplemental_journal_protobuf_v4_1(&[0xff]).is_err());
    }

    #[test]
    fn registration_persistence_is_idempotent() {
        let root = temporary_root();
        let registration = registration_fixture();
        assert_eq!(persist_registration(&root, &registration).unwrap(), (1, 0));
        assert_eq!(persist_registration(&root, &registration).unwrap(), (0, 1));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn opening_persistence_is_idempotent() {
        let root = temporary_root();
        let registration = registration_fixture();
        let ready =
            opening_receipt(&registration, MomentumReserveOpeningStatusV4_1::Ready, 0).unwrap();
        assert_eq!(persist_opening(&root, &ready).unwrap(), (1, 0));
        assert_eq!(persist_opening(&root, &ready).unwrap(), (0, 1));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn all_network_and_authority_counters_are_zero() {
        let value = zero_counters();
        assert_eq!(value.active_committee_count, 3);
        assert_eq!(
            value.network_requests
                + value.transport_constructions
                + value.credential_reads
                + value.new_prospective_row_reads
                + value.new_prospective_label_openings
                + value.historical_test_reads
                + value.future_evaluation_reads
                + value.reserve_opening_attempts
                + value.reserve_row_reads
                + value.reserve_label_reads
                + value.participant_parameter_changes
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
    fn protected_collector_excludes_only_supplemental_root() {
        let root = temporary_root();
        fs::create_dir_all(root.join(ROOT_VERSION_V4_1)).unwrap();
        fs::create_dir_all(root.join("v4")).unwrap();
        fs::write(root.join(ROOT_VERSION_V4_1).join("new.pb"), b"new").unwrap();
        fs::write(root.join("v4").join("old.pb"), b"old").unwrap();
        let mut values = Vec::new();
        collect_protected(&root, &root, &mut values).unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].0, PathBuf::from("v4/old.pb"));
        fs::remove_dir_all(root).unwrap();
    }
}
