//! Offline, agent-private learning sessions backed by the existing shadow trainers.
//!
//! This boundary resolves only explicitly authorized immutable evidence.  It owns
//! private manifests and candidate metadata, but has no active committee, Chair,
//! reward, prospective-evaluation, network, or execution authority.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{
    core::stable_hash_string,
    data::{
        AcquisitionMarketScope, AcquisitionMode, AcquisitionPolicy, AgentCanonicalViewGapReportV1,
        AgentDataIntent, AgentDataPolicy, AgentLearningDataViewV0, AgentLearningIntentV0,
        AgentPrivateLearningStateV0, CanonicalViewGapStatusV1,
        CompositeLearningAcquisitionRegistrationV1, CompositeLearningEpochReceiptV1,
        CompositeLearningEpochStatusV1, ConfiguredUniverse, DataLookback, DataPriority,
        DataSnapshot, DatasetKind, EvidenceDecisionGate, LearningDataArtifactRefV0,
        LearningDataCallerV0, LearningDataPlaneSafetyCountersV0, LearningDataVisibilityV0,
        PERSISTED_LEARNING_INTENT_PROJECTION_VERSION_V1, ReadOnlyProviderRegistry,
        ReadOnlyProviderRequest, SnapshotAdjustmentSemanticsV1, SnapshotSourceType,
        build_agent_learning_data_view_v0, build_learning_acquisition_plan_v0,
        create_agent_learning_intent_v0, decode_agent_canonical_view_gap_report_protobuf_v1,
        decode_agent_learning_data_view_protobuf_v0, default_agent_data_policies,
        derive_active_agent_learning_intents_v0, derive_agent_private_learning_state_v0,
        encode_agent_learning_data_view_protobuf_v0, historical_replay_dataset_digest_v0,
        plan_agent_data_intent, read_and_verify_agent_learning_data_view_v0,
        read_composite_epoch_receipt_v1, read_composite_learning_registration_v1,
        read_local_snapshot_protobuf_v1, validate_agent_learning_data_view_v0,
        validate_agent_learning_intent_v0, write_and_verify_agent_learning_data_view_v0,
    },
    league::{AgentKind, HistoricalOhlcvRow, canonical_current_agent_states},
};

use super::cycle_risk_shadow::{
    CycleRiskValidationOnlyExecutionV1, run_cycle_risk_validation_only_v1,
};
use super::{
    ConstantProbabilityBaselineV0, CycleRiskErrorV0, CycleRiskShadowConfigV0, EvaluationMetricsV0,
    FeatureNormalizerV0, IndexRangeV0, LinearMomentumBaselineV0, LogisticPredictionHeadV0,
    ModelAgentDeploymentStatus, MomentumCandleV0, MomentumLearningCampaignConfigV0,
    MomentumLearningCampaignStatusV0, SequenceExampleV0, build_momentum_features_v0,
    build_momentum_learning_windows_v0, build_momentum_sequence_examples_v0, evaluate_head_v0,
    frozen_mamba3_encoder_from_seed_v0, run_cycle_risk_shadow_v0,
    run_momentum_learning_campaign_v0, train_frozen_mamba_head_v0,
};

const SESSION_VERSION_V0: &str = "agent-private-learning-session-v0";
const SESSION_VERSION_V1: &str = "agent-private-learning-session-v1";
const DATASET_VERSION_V0: &str = "agent-private-dataset-manifest-v0";
const CANDIDATE_VERSION_V0: &str = "agent-sandbox-learning-candidate-v0";
const JOURNAL_VERSION_V0: &str = "agent-private-learning-journal-v0";
const REGISTRY_VERSION_V0: &str = "agent-trainer-capability-registry-v0";
const PROJECTION_VERSION_V0: &str = "agent-trainer-input-projection-v0";
const EVIDENCE_LEDGER_VERSION_V0: &str = "candidate-evidence-usage-ledger-v0";
const IDENTITY_AUDIT_VERSION_V0: &str = "agent-candidate-identity-audit-v0";
const EVALUATION_REGISTRATION_VERSION_V0: &str = "agent-candidate-evaluation-registration-v0";
const EVALUATION_JOURNAL_VERSION_V0: &str = "agent-candidate-evaluation-journal-v0";
const SESSION_VERSION_V1_FAMILY: &str = "agent-private-learning-session-v1-family";
const PROJECTION_VERSION_V1: &str = "agent-trainer-input-projection-v1";
const PARTICIPANT_VERSION_V1: &str = "frozen-candidate-participant-v1";
const FAMILY_VERSION_V1: &str = "agent-candidate-family-v1";
const QUALIFICATION_VERSION_V1: &str = "participant-validation-qualification-v1";
const USAGE_LEDGER_VERSION_V1: &str = "agent-candidate-usage-ledger-v1";
const EXCLUSION_VERSION_V1: &str = "evaluation-evidence-exclusion-v1";
const EVALUATION_REGISTRATION_VERSION_V1: &str = "agent-candidate-evaluation-registration-v1";
const EVALUATION_JOURNAL_VERSION_V1: &str = "agent-candidate-evaluation-journal-v1";
const DAILY_CADENCE_MS_V1: u64 = 86_400_000;
const INTENT_MIGRATION_VERSION_V1: &str = "persisted-learning-intent-migration-v1";
const INTENT_POLICY_PROOF_VERSION_V1: &str = "persisted-intent-policy-compatibility-proof-v1";
const INTENT_MIGRATION_PROOF_VERSION_V1: &str = "persisted-learning-intent-migration-proof-v1";
const INTENT_MIGRATION_JOURNAL_VERSION_V1: &str = "persisted-learning-intent-migration-journal-v1";
const INTENT_FIELD_PROVENANCE_VERSION_V1: &str = "migrated-intent-field-provenance-v1";
const MOMENTUM_AGENT_ID_V1: &str = "momentum_trend_fast";
const ARTIFACT_MAGIC_V0: &[u8] = b"SOMA-AGENT-PRIVATE-LEARNING-PB-V0";
const ARTIFACT_SCHEMA_V0: &str = "soma.agent_private_learning.v0";
const DEFAULT_PRIVATE_LEARNING_ROOT_V0: &str = "state/learning_data";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentTrainerKindV0 {
    MomentumFrozenMambaHead,
    CycleRiskIndependentShadow,
    ValueQualityUnavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentTrainerCapabilityV0 {
    pub agent_id: String,
    pub trainer_kind: AgentTrainerKindV0,
    pub supported_dataset_kinds: Vec<DatasetKind>,
    pub supports_training: bool,
    pub shadow_only: bool,
    pub capability_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentTrainerCapabilityRegistryV0 {
    pub registry_version: String,
    pub capabilities: Vec<AgentTrainerCapabilityV0>,
    pub registry_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentLearningSessionStatusV0 {
    Registered,
    DatasetReady,
    CandidateProduced,
    InsufficientEvidence,
    TrainerUnavailable,
    RejectedUnauthorizedEvidence,
    RejectedCutoffLeakage,
    RejectedLabelLeakage,
    RejectedSafetyInvariant,
    TechnicalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentViewResolutionStatusV0 {
    Complete,
    MissingRequiredEvidence,
    OptionalEvidenceUnavailable,
    AmbiguousEquivalentArtifacts,
    UnauthorizedArtifact,
    CutoffLeakage,
    IntegrityFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPrivateLearningSessionV0 {
    pub session_version: String,
    pub session_id: String,
    pub agent_id: String,
    pub agent_kind: AgentKind,
    pub intent_digest: String,
    pub data_view_digest: String,
    pub trainer_capability_digest: String,
    pub information_cutoff_ms: u64,
    pub required_dataset_kinds: Vec<DatasetKind>,
    pub optional_dataset_kinds: Vec<DatasetKind>,
    pub allowed_markets: Vec<AcquisitionMarketScope>,
    pub symbols: Vec<String>,
    pub cadence: String,
    pub lookback: DataLookback,
    pub maximum_staleness_ms: u64,
    pub source_artifact_digests: Vec<String>,
    pub source_policy_digest: String,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub curriculum_policy_digest: String,
    pub private_namespace_digest: String,
    pub training_ledger_digest: String,
    pub trainer_projection_digest: Option<String>,
    pub parent_model_version: Option<String>,
    pub session_status: AgentLearningSessionStatusV0,
    pub session_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentTrainerInputProjectionV0 {
    pub projection_version: String,
    pub agent_id: String,
    pub trainer_kind: AgentTrainerKindV0,
    pub source_view_digest: String,
    pub consumed_artifact_digests: Vec<String>,
    pub referenced_but_unconsumed_artifact_digests: Vec<String>,
    pub primary_series_digest: Option<String>,
    pub auxiliary_series_digests: Vec<String>,
    pub projection_policy_digest: String,
    pub projection_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateEvidenceUseV0 {
    IntentBinding,
    ViewBinding,
    TrainerProjection,
    FeatureDerivation,
    LabelDerivation,
    NormalizerFit,
    ParameterTraining,
    ValidationInference,
    ValidationMetric,
    CheckpointSelection,
    HistoricalTestInference,
    HistoricalTestMetric,
    CandidateIdentity,
    ReportOnly,
    Unused,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateEvidenceUsageEntryV0 {
    pub artifact_digest: String,
    pub range: Option<IndexRangeV0>,
    pub use_kind: CandidateEvidenceUseV0,
    pub labels_read: bool,
    pub parameters_updated: bool,
    pub checkpoint_selection_influenced: bool,
    pub candidate_identity_influenced: bool,
    pub entry_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateEvidenceUsageLedgerV0 {
    pub ledger_version: String,
    pub agent_id: String,
    pub candidate_digest: String,
    pub session_digest: String,
    pub entries: Vec<CandidateEvidenceUsageEntryV0>,
    pub ledger_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateHistoricalTestStatusV0 {
    FreshAndSealed,
    ReadForInferenceOnly,
    MetricsAlreadyComputed,
    InfluencedCandidateSelection,
    InfluencedCandidateIdentity,
    FullyConsumedRetrospectively,
    LineageAmbiguous,
    NoCandidate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateIdentityAuditV0 {
    pub audit_version: String,
    pub candidate_digest: String,
    pub model_identity_inputs: Vec<String>,
    pub metric_identity_inputs: Vec<String>,
    pub test_evidence_in_identity: bool,
    pub historical_test_status: CandidateHistoricalTestStatusV0,
    pub eligible_for_fresh_historical_test: bool,
    pub eligible_for_future_evaluation_registration: bool,
    pub superseded_by_input_binding_hardening: bool,
    pub audit_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateEvaluationRegistrationStatusV0 {
    Registered,
    CandidateUnavailable,
    CandidateIntegrityInvalid,
    LineageAmbiguousBlocked,
    ComparatorUnavailable,
    PolicyInvalid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateEvaluationRegistrationV0 {
    pub registration_version: String,
    pub agent_id: String,
    pub candidate_digest: String,
    pub session_digest: String,
    pub evidence_usage_ledger_digest: String,
    pub identity_audit_digest: String,
    pub evaluation_cutoff_exclusive_ms: u64,
    pub required_dataset_kinds: Vec<DatasetKind>,
    pub source_policy_digest: String,
    pub finality_policy_digest: String,
    pub label_policy_digest: String,
    pub metric_policy_digest: String,
    pub support_policy_digest: String,
    pub comparator_digests: Vec<String>,
    pub minimum_future_rows: usize,
    pub minimum_mature_events: usize,
    pub maximum_requests: usize,
    pub maximum_concurrency: usize,
    pub maximum_retries: usize,
    pub labels_hidden_until_opening: bool,
    pub probabilities_hidden_until_opening: bool,
    pub one_time_opening_required: bool,
    pub active_promotion_forbidden: bool,
    pub reward_application_forbidden: bool,
    pub status: CandidateEvaluationRegistrationStatusV0,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateEvaluationRegistrationJournalEntryV0 {
    pub registration_digest: String,
    pub status: CandidateEvaluationRegistrationStatusV0,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateEvaluationRegistrationJournalV0 {
    pub journal_version: String,
    pub agent_id: String,
    pub candidate_digest: String,
    pub entries: Vec<AgentCandidateEvaluationRegistrationJournalEntryV0>,
    pub journal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateEvaluationSafetyCountersV0 {
    pub active_committee_count: usize,
    pub network_requests: usize,
    pub credential_reads: usize,
    pub prospective_row_reads: usize,
    pub prospective_label_reads: usize,
    pub prospective_mutations: usize,
    pub active_model_changes: usize,
    pub chair_decisions: usize,
    pub votes: usize,
    pub rewards: usize,
    pub penalties: usize,
    pub voice_changes: usize,
    pub promotions: usize,
    pub executions: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateEvaluationResultV0 {
    pub agent_id: String,
    pub candidate_digest: Option<String>,
    pub session_digest: Option<String>,
    pub view_digest: Option<String>,
    pub trainer_projection: Option<AgentTrainerInputProjectionV0>,
    pub evidence_usage_ledger: Option<CandidateEvidenceUsageLedgerV0>,
    pub identity_audit: Option<AgentCandidateIdentityAuditV0>,
    pub evaluation_registration: Option<AgentCandidateEvaluationRegistrationV0>,
    pub registration_journal: Option<AgentCandidateEvaluationRegistrationJournalV0>,
    pub blocked_status: CandidateEvaluationRegistrationStatusV0,
    pub sanitized_error_code: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateEvaluationReportV0 {
    pub report_version: String,
    pub mode: AgentPrivateLearningRunModeV0,
    pub registration_requested: bool,
    pub results: Vec<AgentCandidateEvaluationResultV0>,
    pub safety_counters: CandidateEvaluationSafetyCountersV0,
    pub active_state_unchanged: bool,
    pub duplicate_artifact_count: usize,
    pub storage_failure_count: usize,
    pub report_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateEvaluationPublicSummaryV0 {
    pub agent_id: String,
    pub candidate_digest: Option<String>,
    pub session_digest: Option<String>,
    pub view_digest: Option<String>,
    pub projection_digest: Option<String>,
    pub historical_test_status: CandidateHistoricalTestStatusV0,
    pub evidence_usage_ledger_digest: Option<String>,
    pub identity_audit_digest: Option<String>,
    pub evaluation_cutoff_exclusive_ms: Option<u64>,
    pub registration_status: CandidateEvaluationRegistrationStatusV0,
    pub comparator_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentLearningSessionStatusV1 {
    Registered,
    PersistedViewVerified,
    ProjectionReady,
    CandidateFamilyFrozen,
    InsufficientEvidence,
    TrainerUnavailable,
    ValidationBlocked,
    RejectedUnauthorizedEvidence,
    RejectedCutoffLeakage,
    RejectedSafetyInvariant,
    TechnicalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentTrainerInputProjectionV1 {
    pub projection_version: String,
    pub agent_id: String,
    pub trainer_kind: AgentTrainerKindV0,
    pub source_view_digest: String,
    pub consumed_artifact_digests: Vec<String>,
    pub referenced_unconsumed_artifact_digests: Vec<String>,
    pub primary_series_digest: Option<String>,
    pub projection_policy_digest: String,
    pub projection_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPrivateLearningSessionV1 {
    pub session_version: String,
    pub session_id: String,
    pub agent_id: String,
    pub agent_kind: AgentKind,
    pub intent_digest: String,
    pub view_digest: String,
    pub projection_digest: String,
    pub capability_digest: String,
    pub source_policy_digest: String,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub curriculum_policy_digest: String,
    pub information_cutoff_ms: u64,
    pub source_artifact_digests: Vec<String>,
    pub consumed_artifact_digests: Vec<String>,
    pub referenced_unconsumed_artifact_digests: Vec<String>,
    pub private_namespace_digest: String,
    pub training_ledger_digest: String,
    pub fresh_initialization: bool,
    pub historical_test_access_forbidden: bool,
    pub status: AgentLearningSessionStatusV1,
    pub session_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateParticipantRoleV1 {
    ModelCandidate,
    LinearComparator,
    ConstantComparator,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenCandidateParticipantV1 {
    pub participant_id: String,
    pub role: CandidateParticipantRoleV1,
    pub model_kind: String,
    pub model_artifact_digest: String,
    pub parameter_digest: String,
    pub normalizer_digest: String,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub training_policy_digest: String,
    pub initialization_digest: String,
    pub deployment_status: ModelAgentDeploymentStatus,
    pub participant_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationQualificationStatusV1 {
    Qualified,
    RejectedInsufficientValidation,
    RejectedProbabilityCollapse,
    RejectedNumericalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParticipantValidationQualificationV1 {
    pub participant_digest: String,
    pub validation_range_digest: String,
    pub metric_policy_digest: String,
    pub private_metric_digest: String,
    pub qualification_status: ValidationQualificationStatusV1,
    pub parameter_updates_during_validation: usize,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateFamilyV1 {
    pub family_version: String,
    pub agent_id: String,
    pub session_digest: String,
    pub view_digest: String,
    pub projection_digest: String,
    pub participants: Vec<FrozenCandidateParticipantV1>,
    pub validation_qualification_receipts: Vec<String>,
    pub winner_selected: bool,
    pub historical_test_accessed: bool,
    pub eligible_for_active_committee: bool,
    pub eligible_for_promotion: bool,
    pub eligible_for_reward: bool,
    pub family_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateEvidenceUseV1 {
    ViewBinding,
    TrainerProjection,
    FeatureDerivation,
    LabelDerivation,
    NormalizerFit,
    ParameterTraining,
    ValidationInference,
    ValidationMetric,
    FamilyInclusion,
    ReferencedButUnconsumed,
    ReservedRetrospectiveUnused,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateEvidenceUsageEntryV1 {
    pub artifact_digest: String,
    pub range: Option<IndexRangeV0>,
    pub use_kind: CandidateEvidenceUseV1,
    pub labels_read: bool,
    pub parameters_updated: bool,
    pub entry_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateUsageLedgerV1 {
    pub ledger_version: String,
    pub agent_id: String,
    pub session_digest: String,
    pub family_digest: String,
    pub entries: Vec<CandidateEvidenceUsageEntryV1>,
    pub historical_test_row_reads: usize,
    pub historical_test_label_reads: usize,
    pub historical_test_inference_count: usize,
    pub historical_test_metric_count: usize,
    pub historical_test_checkpoint_selection_count: usize,
    pub historical_test_identity_influence: bool,
    pub ledger_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationEvidenceExclusionV1 {
    pub protected_registration_digests: Vec<String>,
    pub excluded_timestamp_ms: Vec<u64>,
    pub excluded_range_digests: Vec<String>,
    pub exclusion_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateEvaluationRegistrationStatusV1 {
    Registered,
    CandidateUnavailable,
    SessionInvalid,
    ViewInvalid,
    ProjectionInvalid,
    FamilyInvalid,
    QualificationBlocked,
    HistoricalTestAccessDetected,
    ExclusionInvalid,
    InsufficientParticipants,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateEvaluationRegistrationV1 {
    pub registration_version: String,
    pub agent_id: String,
    pub family_digest: String,
    pub session_digest: String,
    pub usage_ledger_digest: String,
    pub participant_digests: Vec<String>,
    pub qualification_receipt_digests: Vec<String>,
    pub exclusion_digest: String,
    pub minimum_accepted_timestamp_ms: u64,
    pub required_dataset_kinds: Vec<DatasetKind>,
    pub source_policy_digest: String,
    pub finality_policy_digest: String,
    pub label_policy_digest: String,
    pub metric_policy_digest: String,
    pub support_policy_digest: String,
    pub minimum_future_rows: usize,
    pub minimum_mature_events: usize,
    pub maximum_requests: usize,
    pub maximum_concurrency: usize,
    pub maximum_retries: usize,
    pub labels_hidden_until_opening: bool,
    pub probabilities_hidden_until_opening: bool,
    pub one_time_opening_required: bool,
    pub winner_selection_forbidden_before_opening: bool,
    pub active_promotion_forbidden: bool,
    pub reward_application_forbidden: bool,
    pub status: CandidateEvaluationRegistrationStatusV1,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateEvaluationRegistrationJournalEntryV1 {
    pub registration_digest: String,
    pub status: CandidateEvaluationRegistrationStatusV1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateEvaluationRegistrationJournalV1 {
    pub journal_version: String,
    pub agent_id: String,
    pub family_digest: String,
    pub entries: Vec<AgentCandidateEvaluationRegistrationJournalEntryV1>,
    pub journal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentLearningSafetyCountersV1 {
    pub active_committee_count: usize,
    pub network_requests: usize,
    pub credential_reads: usize,
    pub prospective_row_reads: usize,
    pub prospective_label_reads: usize,
    pub prospective_mutations: usize,
    pub historical_test_reads_v1: usize,
    pub active_model_changes: usize,
    pub chair_decisions: usize,
    pub votes: usize,
    pub rewards: usize,
    pub penalties: usize,
    pub voice_changes: usize,
    pub promotions: usize,
    pub executions: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentPrivateLearningInputV1 {
    pub input: AgentPrivateLearningSessionInputV0,
    pub persisted_view_verified: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PersistedIntentMigrationBlockerV1 {
    None,
    LegacySessionNotSelfDescribing,
    LegacyIntentMetadataIncomplete,
    PersistedIntentDigestMismatch,
    PolicyDigestMismatch,
    CanonicalSnapshotBindingMismatch,
    ViewDigestMismatch,
    RequiredEvidenceMismatch,
    OptionalEvidenceOnlyMisclassified,
    CutoffMismatch,
    AmbiguousFieldProvenance,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MigratedIntentFieldSourceV1 {
    LegacySession,
    LegacyIntentProjection,
    VerifiedAgentPolicy,
    CanonicalGapReport,
    CompositeAcquisitionRegistration,
    CanonicalSnapshot,
    ExistingPrivateLearningState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersistedLearningIntentMigrationStatusV1 {
    Migrated,
    AlreadyMigrated,
    SourceArtifactMissing,
    SourceIntegrityMismatch,
    PolicyBindingMismatch,
    CanonicalIntentInvalid,
    CanonicalViewInvalid,
    AmbiguousFieldProvenance,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigratedIntentFieldProvenanceV1 {
    pub provenance_version: String,
    pub field_name: String,
    pub sources: Vec<MigratedIntentFieldSourceV1>,
    pub source_artifact_digests: Vec<String>,
    pub value_digest: String,
    pub provenance_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedIntentPolicyCompatibilityProofV1 {
    pub agent_id: String,
    pub legacy_policy_digest: String,
    pub current_policy_digest: String,
    pub required_datasets_equal: bool,
    pub optional_datasets_equal: bool,
    pub allowed_markets_equal: bool,
    pub cadence_equal: bool,
    pub lookback_equal: bool,
    pub staleness_equal: bool,
    pub semantically_compatible: bool,
    pub proof_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedLearningIntentMigrationProofV1 {
    pub migration_version: String,
    pub agent_id: String,
    pub legacy_session_digest: String,
    pub legacy_intent_digest: String,
    pub gap_report_digest: String,
    pub composite_registration_digest: String,
    pub merged_snapshot_digest: String,
    pub policy_compatibility_proof_digest: String,
    pub field_provenance_digests: Vec<String>,
    pub canonical_intent_digest: String,
    pub canonical_view_digest: String,
    pub information_cutoff_unchanged: bool,
    pub lookback_unchanged: bool,
    pub policy_semantics_unchanged: bool,
    pub evidence_set_unchanged: bool,
    pub exclusions_unchanged: bool,
    pub no_field_invented: bool,
    pub migration_status: PersistedLearningIntentMigrationStatusV1,
    pub proof_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedLearningIntentMigrationJournalV1 {
    pub journal_version: String,
    pub agent_id: String,
    pub migration_proof_digest: String,
    pub canonical_intent_digest: String,
    pub canonical_view_digest: String,
    pub entry_count: usize,
    pub network_requests: usize,
    pub transport_constructions: usize,
    pub credential_reads: usize,
    pub prospective_reads: usize,
    pub active_model_changes: usize,
    pub status: PersistedLearningIntentMigrationStatusV1,
    pub journal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedIntentMigrationSafetyCountersV1 {
    pub active_committee_count: usize,
    pub network_requests: usize,
    pub transport_constructions: usize,
    pub credential_reads: usize,
    pub prospective_artifact_reads: usize,
    pub prospective_label_reads: usize,
    pub future_evaluation_reads: usize,
    pub active_model_changes: usize,
    pub chair_decisions: usize,
    pub votes: usize,
    pub rewards: usize,
    pub penalties: usize,
    pub voice_changes: usize,
    pub promotions: usize,
    pub executions: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedLearningIntentMigrationReportV1 {
    pub report_version: String,
    pub mode: AgentPrivateLearningRunModeV0,
    pub agent_id: String,
    pub blocker: PersistedIntentMigrationBlockerV1,
    pub first_failing_invariant: Option<String>,
    pub status: PersistedLearningIntentMigrationStatusV1,
    pub legacy_session_digest: Option<String>,
    pub legacy_intent_digest: Option<String>,
    pub canonical_gap_digest: Option<String>,
    pub composite_registration_digest: Option<String>,
    pub canonical_snapshot_digest: Option<String>,
    pub canonical_intent_digest: Option<String>,
    pub canonical_view_digest: Option<String>,
    pub policy_compatibility_proof_digest: Option<String>,
    pub migration_proof_digest: Option<String>,
    pub migration_journal_digest: Option<String>,
    pub field_provenance_count: usize,
    pub required_evidence_complete: bool,
    pub optional_evidence_unavailable: bool,
    pub normal_validator_passed: bool,
    pub normal_view_builder_passed: bool,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub storage_failure_count: usize,
    pub protected_artifacts_unchanged: bool,
    pub active_state_unchanged: bool,
    pub safety_counters: PersistedIntentMigrationSafetyCountersV1,
    pub report_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PersistedLearningIntentMigrationExecutionV1 {
    pub report: PersistedLearningIntentMigrationReportV1,
    pub canonical_input: Option<AgentPrivateLearningInputV1>,
}

#[derive(Clone, Debug)]
struct PersistedIntentMigrationSourcesV1 {
    legacy_session: AgentPrivateLearningSessionV0,
    legacy_projection: AgentLearningIntentV0,
    policy: AgentDataPolicy,
    acquisition_gap: crate::data::AgentCanonicalViewGapV1,
    canonical_gap: crate::data::AgentCanonicalViewGapV1,
    composite_registration: CompositeLearningAcquisitionRegistrationV1,
    epoch_receipt: CompositeLearningEpochReceiptV1,
    canonical_snapshot: DataSnapshot,
}

#[derive(Clone, Debug)]
struct DerivedPersistedIntentMigrationV1 {
    blocker: PersistedIntentMigrationBlockerV1,
    first_failing_invariant: Option<String>,
    canonical_intent: AgentLearningIntentV0,
    canonical_view: AgentLearningDataViewV0,
    canonical_input: AgentPrivateLearningInputV1,
    policy_proof: PersistedIntentPolicyCompatibilityProofV1,
    field_provenance: Vec<MigratedIntentFieldProvenanceV1>,
    migration_proof: PersistedLearningIntentMigrationProofV1,
    journal: PersistedLearningIntentMigrationJournalV1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateFamilyResultV1 {
    pub agent_id: String,
    pub session: Option<AgentPrivateLearningSessionV1>,
    pub projection: Option<AgentTrainerInputProjectionV1>,
    pub family: Option<AgentCandidateFamilyV1>,
    pub qualification_receipts: Vec<ParticipantValidationQualificationV1>,
    pub usage_ledger: Option<AgentCandidateUsageLedgerV1>,
    pub status: AgentLearningSessionStatusV1,
    pub sanitized_error_code: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateFamiliesReportV1 {
    pub report_version: String,
    pub mode: AgentPrivateLearningRunModeV0,
    pub results: Vec<AgentCandidateFamilyResultV1>,
    pub safety_counters: AgentLearningSafetyCountersV1,
    pub active_state_unchanged: bool,
    pub duplicate_artifact_count: usize,
    pub storage_failure_count: usize,
    pub report_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtectedEvaluationReservationV1 {
    pub protected_registration_digests: Vec<String>,
    pub reserved_timestamp_ms: Vec<u64>,
    pub cadence_ms: u64,
    pub provider_finality_boundary_ms: u64,
    pub reservation_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateEvaluationResultV1 {
    pub agent_id: String,
    pub family_digest: Option<String>,
    pub session_digest: Option<String>,
    pub exclusion: Option<EvaluationEvidenceExclusionV1>,
    pub registration: Option<AgentCandidateEvaluationRegistrationV1>,
    pub journal: Option<AgentCandidateEvaluationRegistrationJournalV1>,
    pub status: CandidateEvaluationRegistrationStatusV1,
    pub sanitized_error_code: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateEvaluationsReportV1 {
    pub report_version: String,
    pub mode: AgentPrivateLearningRunModeV0,
    pub results: Vec<AgentCandidateEvaluationResultV1>,
    pub safety_counters: AgentLearningSafetyCountersV1,
    pub active_state_unchanged: bool,
    pub duplicate_artifact_count: usize,
    pub storage_failure_count: usize,
    pub report_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateFamilyPublicSummaryV1 {
    pub agent_id: String,
    pub session_digest: Option<String>,
    pub view_digest: Option<String>,
    pub projection_digest: Option<String>,
    pub family_digest: Option<String>,
    pub participant_count: usize,
    pub historical_test_access_count: usize,
    pub status: AgentLearningSessionStatusV1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCandidateEvaluationPublicSummaryV1 {
    pub agent_id: String,
    pub session_digest: Option<String>,
    pub family_digest: Option<String>,
    pub participant_count: usize,
    pub historical_test_access_count: usize,
    pub minimum_accepted_timestamp_ms: Option<u64>,
    pub exclusion_digest: Option<String>,
    pub registration_status: CandidateEvaluationRegistrationStatusV1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPrivateDatasetManifestV0 {
    pub dataset_version: String,
    pub dataset_id: String,
    pub agent_id: String,
    pub session_id: String,
    pub data_view_digest: String,
    pub source_artifact_digests: Vec<String>,
    pub dataset_kinds: Vec<DatasetKind>,
    pub information_cutoff_ms: u64,
    pub row_count: usize,
    pub training_range: IndexRangeV0,
    pub first_purge_range: IndexRangeV0,
    pub validation_range: IndexRangeV0,
    pub second_purge_range: IndexRangeV0,
    pub sealed_test_range: Option<IndexRangeV0>,
    pub normalizer_fit_range: IndexRangeV0,
    pub validation_parameter_update_count: usize,
    pub test_checkpoint_selection_count: usize,
    pub prospective_row_read_count: usize,
    pub prospective_label_read_count: usize,
    pub feature_artifact_digest: String,
    pub label_artifact_digest: String,
    pub normalizer_digest: String,
    pub manifest_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSandboxLearningCandidateV0 {
    pub candidate_version: String,
    pub agent_id: String,
    pub session_digest: String,
    pub data_view_digest: String,
    pub parent_model_version: Option<String>,
    pub model_artifact_digest: String,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub normalizer_digest: String,
    pub training_policy_digest: String,
    pub private_metrics_digest: String,
    pub deployment_status: ModelAgentDeploymentStatus,
    pub retrospective_research_only: bool,
    pub eligible_for_active_committee: bool,
    pub eligible_for_promotion: bool,
    pub eligible_for_reward: bool,
    pub candidate_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentLearningSessionJournalEntryV0 {
    pub session_digest: String,
    pub session_status: AgentLearningSessionStatusV0,
    pub dataset_manifest_digest: Option<String>,
    pub candidate_digest: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentLearningSessionJournalV0 {
    pub journal_version: String,
    pub agent_id: String,
    pub entries: Vec<AgentLearningSessionJournalEntryV0>,
    pub journal_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentPrivateLearningRunModeV0 {
    Status,
    DryRun,
    ExecuteLocal,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentPrivateLearningArtifactV0 {
    pub artifact_ref: LearningDataArtifactRefV0,
    pub snapshot: DataSnapshot,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentPrivateLearningSessionInputV0 {
    pub intent: AgentLearningIntentV0,
    pub policy: AgentDataPolicy,
    pub view: AgentLearningDataViewV0,
    pub artifacts: Vec<AgentPrivateLearningArtifactV0>,
    pub resolution_status: AgentViewResolutionStatusV0,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPrivateLearningSessionResultV0 {
    pub session: AgentPrivateLearningSessionV0,
    pub trainer_kind: AgentTrainerKindV0,
    pub view_resolution_status: AgentViewResolutionStatusV0,
    pub source_count: usize,
    pub trainer_projection: Option<AgentTrainerInputProjectionV0>,
    pub dataset_manifest: Option<AgentPrivateDatasetManifestV0>,
    pub candidate: Option<AgentSandboxLearningCandidateV0>,
    pub journal: AgentLearningSessionJournalV0,
    pub sanitized_error_code: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPrivateLearningSessionsReportV0 {
    pub report_version: String,
    pub mode: AgentPrivateLearningRunModeV0,
    pub capability_registry: AgentTrainerCapabilityRegistryV0,
    pub results: Vec<AgentPrivateLearningSessionResultV0>,
    pub safety_counters: LearningDataPlaneSafetyCountersV0,
    pub active_state_unchanged: bool,
    pub duplicate_artifact_count: usize,
    pub storage_failure_count: usize,
    pub report_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPrivateLearningPublicSummaryV0 {
    pub agent_id: String,
    pub intent_digest: String,
    pub data_view_digest: String,
    pub session_digest: String,
    pub trainer_kind: AgentTrainerKindV0,
    pub view_resolution_status: AgentViewResolutionStatusV0,
    pub trainer_projection_digest: Option<String>,
    pub source_count: usize,
    pub session_status: AgentLearningSessionStatusV0,
    pub candidate_present: bool,
    pub candidate_digest: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EvidenceResolutionErrorV0 {
    SourceDigest,
    UnauthorizedDataset,
    CrossAgentArtifact,
    CutoffLeakage,
    Chronology,
    Duplicate,
    NonFinite,
    UnsafeEvidence,
    Insufficient,
}

#[derive(Clone, Debug)]
struct MaterializedPrivateDatasetV0 {
    snapshot: DataSnapshot,
    manifest: AgentPrivateDatasetManifestV0,
}

#[derive(Clone, Debug)]
struct ValidationParticipantBuildV1 {
    model_kind: String,
    role: CandidateParticipantRoleV1,
    model_artifact_digest: String,
    parameter_digest: String,
    normalizer_digest: String,
    training_policy_digest: String,
    initialization_digest: String,
    private_validation_metric_digest: String,
    qualification_status: ValidationQualificationStatusV1,
}

#[derive(Clone, Debug)]
struct ValidationOnlyExecutionV1 {
    training_range: IndexRangeV0,
    purge_range: IndexRangeV0,
    validation_range: IndexRangeV0,
    reserved_retrospective_unused_range: IndexRangeV0,
    participants: Vec<ValidationParticipantBuildV1>,
    validation_parameter_updates: usize,
    historical_test_row_reads: usize,
    historical_test_label_reads: usize,
    historical_test_inference_count: usize,
    historical_test_metric_count: usize,
    historical_test_checkpoint_selection_count: usize,
}

pub fn agent_trainer_capability_registry_v0() -> AgentTrainerCapabilityRegistryV0 {
    let mut capabilities = vec![
        capability(
            "momentum_trend_fast",
            AgentTrainerKindV0::MomentumFrozenMambaHead,
            vec![
                DatasetKind::DailyOhlcv,
                DatasetKind::AdjustedDailyOhlcv,
                DatasetKind::CryptoDailyOhlcv,
            ],
            true,
        ),
        capability(
            "value_quality_filter",
            AgentTrainerKindV0::ValueQualityUnavailable,
            vec![
                DatasetKind::AdjustedDailyOhlcv,
                DatasetKind::QuarterlyFundamentals,
                DatasetKind::ValuationMetrics,
            ],
            false,
        ),
        capability(
            "cycle_risk_skeptic",
            AgentTrainerKindV0::CycleRiskIndependentShadow,
            vec![DatasetKind::MarketIndexDaily],
            true,
        ),
    ];
    capabilities.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
    let mut registry = AgentTrainerCapabilityRegistryV0 {
        registry_version: REGISTRY_VERSION_V0.to_string(),
        capabilities,
        registry_digest: String::new(),
    };
    registry.registry_digest = registry_digest_v0(&registry);
    registry
}

fn capability(
    agent_id: &str,
    trainer_kind: AgentTrainerKindV0,
    mut supported_dataset_kinds: Vec<DatasetKind>,
    supports_training: bool,
) -> AgentTrainerCapabilityV0 {
    supported_dataset_kinds.sort();
    supported_dataset_kinds.dedup();
    let mut capability = AgentTrainerCapabilityV0 {
        agent_id: agent_id.to_string(),
        trainer_kind,
        supported_dataset_kinds,
        supports_training,
        shadow_only: true,
        capability_digest: String::new(),
    };
    capability.capability_digest = capability_digest_v0(&capability);
    capability
}

pub fn build_agent_private_learning_inputs_v0(
    snapshots: &[DataSnapshot],
    information_cutoff_ms: u64,
) -> Result<Vec<AgentPrivateLearningSessionInputV0>, String> {
    if information_cutoff_ms == 0 {
        return Err("private learning cutoff must be positive".to_string());
    }
    let policies = default_agent_data_policies();
    let configured_universe = configured_universe_from_snapshots_v0(snapshots);
    let intents = derive_active_agent_learning_intents_v0(
        &canonical_current_agent_states(),
        &configured_universe,
        &policies,
        information_cutoff_ms,
    )?;
    let mut inputs = intents
        .iter()
        .map(|intent| {
            let policy = policies
                .iter()
                .find(|policy| policy.agent_kind == intent.agent_kind)
                .ok_or_else(|| "canonical agent learning policy unavailable".to_string())?;
            build_session_input_v0(intent, policy, snapshots)
        })
        .collect::<Result<Vec<_>, String>>()?;
    inputs.sort_by(|left, right| left.intent.agent_id.cmp(&right.intent.agent_id));
    Ok(inputs)
}

fn build_session_input_v0(
    intent: &AgentLearningIntentV0,
    policy: &AgentDataPolicy,
    snapshots: &[DataSnapshot],
) -> Result<AgentPrivateLearningSessionInputV0, String> {
    validate_agent_learning_intent_v0(intent, policy)?;
    let plan = build_learning_acquisition_plan_v0(
        std::slice::from_ref(intent),
        std::slice::from_ref(policy),
        &ReadOnlyProviderRegistry::default(),
        AcquisitionMode::LocalSnapshotReplay,
        &AcquisitionPolicy::default(),
    )?;
    let mut artifacts = Vec::new();
    let mut resolution_status = AgentViewResolutionStatusV0::Complete;
    let mut optional_missing = false;
    for planned in &plan.planned_requests {
        let required = planned.required_by_agents.contains(&intent.agent_id);
        match resolve_snapshot_for_request_v0(&planned.request, snapshots) {
            Ok(Some(snapshot)) => artifacts.push(AgentPrivateLearningArtifactV0 {
                artifact_ref: LearningDataArtifactRefV0 {
                    artifact_digest: snapshot.content_digest.clone(),
                    dataset_kind: snapshot.dataset_kind,
                    visibility: LearningDataVisibilityV0::SharedCanonicalRaw,
                    owner_agent_id: None,
                    maximum_event_timestamp_ms: snapshot
                        .actual_end_timestamp_ms
                        .unwrap_or_default(),
                },
                snapshot,
            }),
            Ok(None) if required => {
                resolution_status = AgentViewResolutionStatusV0::MissingRequiredEvidence;
            }
            Ok(None) => optional_missing = true,
            Err(status) => resolution_status = status,
        }
    }
    if plan.rejected_requests.iter().any(|rejected| {
        rejected.agent_ids.contains(&intent.agent_id)
            && intent.required_datasets.contains(&rejected.dataset_kind)
    }) {
        resolution_status = AgentViewResolutionStatusV0::MissingRequiredEvidence;
    }
    artifacts.sort_by(|left, right| {
        left.artifact_ref
            .dataset_kind
            .cmp(&right.artifact_ref.dataset_kind)
            .then_with(|| {
                left.artifact_ref
                    .artifact_digest
                    .cmp(&right.artifact_ref.artifact_digest)
            })
    });
    let artifact_refs = artifacts
        .iter()
        .map(|artifact| artifact.artifact_ref.clone())
        .collect::<Vec<_>>();
    let view = build_agent_learning_data_view_v0(
        intent,
        policy,
        &artifact_refs,
        &derive_agent_private_learning_state_v0(&intent),
    )?;
    if resolution_status == AgentViewResolutionStatusV0::Complete {
        resolution_status = if view.decision_gate != EvidenceDecisionGate::Ready {
            AgentViewResolutionStatusV0::MissingRequiredEvidence
        } else if optional_missing
            || intent
                .optional_datasets
                .iter()
                .any(|kind| !view.visible_dataset_kinds.contains(kind))
        {
            AgentViewResolutionStatusV0::OptionalEvidenceUnavailable
        } else {
            AgentViewResolutionStatusV0::Complete
        };
    }
    Ok(AgentPrivateLearningSessionInputV0 {
        intent: intent.clone(),
        policy: policy.clone(),
        view,
        artifacts,
        resolution_status,
    })
}

pub fn build_agent_private_learning_input_from_persisted_view_v0(
    intent: &AgentLearningIntentV0,
    policy: &AgentDataPolicy,
    view: &AgentLearningDataViewV0,
    snapshots: &[DataSnapshot],
) -> Result<AgentPrivateLearningSessionInputV0, String> {
    validate_agent_learning_intent_v0(intent, policy)?;
    validate_agent_learning_data_view_v0(view)?;
    if intent.agent_id != view.agent_id
        || intent.information_cutoff_ms != view.information_cutoff_ms
        || intent.feature_policy_digest != view.feature_policy_digest
        || intent.label_policy_digest != view.label_policy_digest
        || intent.curriculum_policy_digest != view.curriculum_policy_digest
    {
        return Err("persisted learning view binding rejected".to_string());
    }
    let mut artifacts = view
        .source_artifact_digests
        .iter()
        .map(|digest| {
            let snapshot = snapshots
                .iter()
                .find(|snapshot| snapshot.content_digest == *digest)
                .ok_or_else(|| "persisted learning view source unavailable".to_string())?;
            Ok(AgentPrivateLearningArtifactV0 {
                artifact_ref: LearningDataArtifactRefV0 {
                    artifact_digest: digest.clone(),
                    dataset_kind: snapshot.dataset_kind,
                    visibility: LearningDataVisibilityV0::SharedCanonicalRaw,
                    owner_agent_id: None,
                    maximum_event_timestamp_ms: snapshot
                        .actual_end_timestamp_ms
                        .unwrap_or_default(),
                },
                snapshot: snapshot.clone(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    artifacts.sort_by(|left, right| {
        left.artifact_ref
            .artifact_digest
            .cmp(&right.artifact_ref.artifact_digest)
    });
    let optional_missing = intent
        .optional_datasets
        .iter()
        .any(|kind| !view.visible_dataset_kinds.contains(kind));
    Ok(AgentPrivateLearningSessionInputV0 {
        intent: intent.clone(),
        policy: policy.clone(),
        view: view.clone(),
        artifacts,
        resolution_status: if view.decision_gate != EvidenceDecisionGate::Ready {
            AgentViewResolutionStatusV0::MissingRequiredEvidence
        } else if optional_missing {
            AgentViewResolutionStatusV0::OptionalEvidenceUnavailable
        } else {
            AgentViewResolutionStatusV0::Complete
        },
    })
}

pub fn build_agent_private_learning_inputs_v1(
    snapshots: &[DataSnapshot],
    information_cutoff_ms: u64,
    root: &Path,
    mode: AgentPrivateLearningRunModeV0,
) -> Vec<AgentPrivateLearningInputV1> {
    if information_cutoff_ms == 0 {
        return Vec::new();
    }
    let policies = default_agent_data_policies();
    let states = canonical_current_agent_states();
    let registry = agent_trainer_capability_registry_v0();
    let migrated_momentum = read_persisted_learning_intent_migration_v1(root, snapshots).ok();
    let projected_intents = load_persisted_agent_learning_intents_v0(root, snapshots).ok();
    let universe = configured_universe_from_snapshots_v0(snapshots);
    let mut inputs = Vec::new();
    for capability in registry
        .capabilities
        .iter()
        .filter(|capability| capability.supports_training)
    {
        let Some(state) = states
            .iter()
            .find(|state| state.agent_id == capability.agent_id)
            .cloned()
        else {
            continue;
        };
        let Some(policy) = policies
            .iter()
            .find(|policy| policy.agent_kind == state.kind)
        else {
            continue;
        };
        if capability.agent_id == MOMENTUM_AGENT_ID_V1 {
            if let Some(migrated) = migrated_momentum.as_ref().filter(|migrated| {
                migrated.input.intent.information_cutoff_ms == information_cutoff_ms
                    && migrated.input.intent.agent_kind == state.kind
                    && validate_agent_learning_intent_v0(&migrated.input.intent, policy).is_ok()
            }) {
                inputs.push(migrated.clone());
                continue;
            }
        }
        let persisted_intent = projected_intents.as_ref().and_then(|intents| {
            intents
                .iter()
                .find(|intent| intent.agent_id == capability.agent_id)
        });
        let intent = if let Some(persisted_intent) = persisted_intent {
            if validate_agent_learning_intent_v0(persisted_intent, policy).is_err() {
                continue;
            }
            persisted_intent.clone()
        } else {
            let data_intent = plan_agent_data_intent(
                state.agent_id.clone(),
                state.kind,
                &universe,
                policy,
                information_cutoff_ms,
            );
            let Ok(intent) = create_agent_learning_intent_v0(
                &LearningDataCallerV0::Agent(state.agent_id.clone()),
                &data_intent,
                policy,
                information_cutoff_ms,
            ) else {
                continue;
            };
            intent
        };
        let Ok(planned) = build_session_input_v0(&intent, policy, snapshots) else {
            continue;
        };
        if planned.view.decision_gate != EvidenceDecisionGate::Ready {
            inputs.push(AgentPrivateLearningInputV1 {
                input: planned,
                persisted_view_verified: false,
            });
            continue;
        }
        let persisted_view = match mode {
            AgentPrivateLearningRunModeV0::ExecuteLocal => {
                let view_root = root.join("v1").join(&capability.agent_id).join("views");
                write_and_verify_agent_learning_data_view_v0(&planned.view, &view_root)
                    .and_then(|path| read_and_verify_agent_learning_data_view_v0(&path))
            }
            AgentPrivateLearningRunModeV0::DryRun => {
                encode_agent_learning_data_view_protobuf_v0(&planned.view).and_then(|bytes| {
                    decode_agent_learning_data_view_protobuf_v0(&bytes).map(|(_, view)| view)
                })
            }
            AgentPrivateLearningRunModeV0::Status => Ok(planned.view.clone()),
        };
        let Ok(persisted_view) = persisted_view else {
            inputs.push(AgentPrivateLearningInputV1 {
                input: planned,
                persisted_view_verified: false,
            });
            continue;
        };
        match build_agent_private_learning_input_from_persisted_view_v0(
            &intent,
            policy,
            &persisted_view,
            snapshots,
        ) {
            Ok(input) => inputs.push(AgentPrivateLearningInputV1 {
                input,
                persisted_view_verified: true,
            }),
            Err(_) => inputs.push(AgentPrivateLearningInputV1 {
                input: planned,
                persisted_view_verified: false,
            }),
        }
    }
    inputs.sort_by(|left, right| left.input.intent.agent_id.cmp(&right.input.intent.agent_id));
    inputs
}

pub fn configured_universe_from_snapshots_v0(snapshots: &[DataSnapshot]) -> ConfiguredUniverse {
    let mut symbols_by_market = BTreeMap::<AcquisitionMarketScope, Vec<String>>::new();
    for snapshot in snapshots {
        symbols_by_market
            .entry(snapshot.market_scope)
            .or_default()
            .extend(snapshot.symbols.clone());
    }
    for symbols in symbols_by_market.values_mut() {
        symbols.sort();
        symbols.dedup();
    }
    ConfiguredUniverse { symbols_by_market }
}

pub fn load_persisted_agent_learning_intents_v0(
    root: &Path,
    snapshots: &[DataSnapshot],
) -> Result<Vec<AgentLearningIntentV0>, String> {
    let states = canonical_current_agent_states();
    let policies = default_agent_data_policies();
    let mut intents = Vec::with_capacity(states.len());
    for state in states {
        let policy = policies
            .iter()
            .find(|policy| policy.agent_kind == state.kind)
            .ok_or_else(|| "persisted learning policy unavailable".to_string())?;
        let session_dir = root.join(&state.agent_id).join("sessions");
        let mut paths = fs::read_dir(&session_dir)
            .map_err(|_| "persisted learning session directory unavailable".to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && path.extension().is_some_and(|value| value == "pb"))
            .collect::<Vec<_>>();
        paths.sort();
        let mut sessions = Vec::new();
        for path in paths {
            let session = decode_session_protobuf_v0(
                &fs::read(path)
                    .map_err(|_| "persisted learning session read failed".to_string())?,
            )?;
            if session.agent_id == state.agent_id && session.agent_kind == state.kind {
                sessions.push(session);
            }
        }
        sessions.sort_by(|left, right| {
            left.information_cutoff_ms
                .cmp(&right.information_cutoff_ms)
                .then_with(|| left.session_digest.cmp(&right.session_digest))
        });
        let latest_cutoff = sessions
            .last()
            .map(|session| session.information_cutoff_ms)
            .ok_or_else(|| {
                format!(
                    "persisted learning intent unavailable for active agent {}",
                    state.agent_id
                )
            })?;
        let mut latest = sessions
            .into_iter()
            .filter(|session| session.information_cutoff_ms == latest_cutoff)
            .collect::<Vec<_>>();
        latest.dedup_by(|left, right| left.session_digest == right.session_digest);
        if latest.len() != 1 {
            return Err("persisted learning intent is ambiguous".to_string());
        }
        let session = latest.pop().unwrap();
        let exact_matches = session
            .allowed_markets
            .iter()
            .filter_map(|market_scope| {
                let data_intent = AgentDataIntent {
                    agent_id: session.agent_id.clone(),
                    agent_kind: session.agent_kind,
                    market_scope: *market_scope,
                    symbols: session.symbols.clone(),
                    required_datasets: session.required_dataset_kinds.clone(),
                    optional_datasets: session.optional_dataset_kinds.clone(),
                    lookback: session.lookback.clone(),
                    target_cadence: session.cadence.clone(),
                    max_staleness_ms: session.maximum_staleness_ms,
                    priority: DataPriority::Required,
                    reason_codes: policy.reason_codes.clone(),
                };
                create_agent_learning_intent_v0(
                    &LearningDataCallerV0::Agent(session.agent_id.clone()),
                    &data_intent,
                    policy,
                    session.information_cutoff_ms,
                )
                .ok()
                .filter(|intent| intent.intent_digest == session.intent_digest)
            })
            .collect::<Vec<_>>();
        if exact_matches.len() == 1 {
            intents.push(exact_matches.into_iter().next().unwrap());
            continue;
        }
        if session.session_version != SESSION_VERSION_V0 {
            return Err("persisted learning intent metadata rejected".to_string());
        }
        let source_snapshots = snapshots
            .iter()
            .filter(|snapshot| {
                session
                    .source_artifact_digests
                    .contains(&snapshot.content_digest)
            })
            .collect::<Vec<_>>();
        let mut market_scopes = source_snapshots
            .iter()
            .map(|snapshot| snapshot.market_scope)
            .filter(|market| policy.allowed_markets.contains(market))
            .collect::<Vec<_>>();
        market_scopes.sort();
        market_scopes.dedup();
        if market_scopes.is_empty() {
            market_scopes = policy.allowed_markets.clone();
        }
        let mut symbols = source_snapshots
            .iter()
            .flat_map(|snapshot| snapshot.symbols.clone())
            .collect::<Vec<_>>();
        symbols.sort();
        symbols.dedup();
        let inferred_exact_matches = market_scopes
            .iter()
            .filter_map(|market_scope| {
                let mut lookback = policy.default_lookback.clone();
                lookback.end_timestamp_ms = Some(session.information_cutoff_ms);
                let data_intent = AgentDataIntent {
                    agent_id: session.agent_id.clone(),
                    agent_kind: session.agent_kind,
                    market_scope: *market_scope,
                    symbols: symbols.clone(),
                    required_datasets: policy.required_dataset_kinds.clone(),
                    optional_datasets: policy.optional_dataset_kinds.clone(),
                    lookback,
                    target_cadence: "1d".to_string(),
                    max_staleness_ms: policy.max_staleness_ms,
                    priority: DataPriority::Required,
                    reason_codes: policy.reason_codes.clone(),
                };
                create_agent_learning_intent_v0(
                    &LearningDataCallerV0::Agent(session.agent_id.clone()),
                    &data_intent,
                    policy,
                    session.information_cutoff_ms,
                )
                .ok()
                .filter(|intent| intent.intent_digest == session.intent_digest)
            })
            .collect::<Vec<_>>();
        if inferred_exact_matches.len() == 1 {
            intents.push(inferred_exact_matches.into_iter().next().unwrap());
            continue;
        }
        let cadence = source_snapshots
            .iter()
            .filter_map(|snapshot| {
                snapshot
                    .compatibility
                    .as_ref()
                    .map(|compatibility| compatibility.cadence.clone())
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .next()
            .unwrap_or_else(|| "1d".to_string());
        let mut lookback = source_snapshots
            .first()
            .map(|snapshot| snapshot.requested_lookback.clone())
            .unwrap_or_else(|| policy.default_lookback.clone());
        lookback.end_timestamp_ms = Some(session.information_cutoff_ms);
        intents.push(AgentLearningIntentV0 {
            intent_version: PERSISTED_LEARNING_INTENT_PROJECTION_VERSION_V1.to_string(),
            agent_id: session.agent_id,
            agent_kind: session.agent_kind,
            market_scopes,
            symbols,
            required_datasets: policy.required_dataset_kinds.clone(),
            optional_datasets: policy.optional_dataset_kinds.clone(),
            cadence,
            lookback,
            information_cutoff_ms: session.information_cutoff_ms,
            maximum_staleness_ms: policy.max_staleness_ms,
            source_policy_digest: session.source_policy_digest,
            feature_policy_digest: session.feature_policy_digest,
            label_policy_digest: session.label_policy_digest,
            curriculum_policy_digest: session.curriculum_policy_digest,
            intent_digest: session.intent_digest,
        });
    }
    intents.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
    Ok(intents)
}

#[derive(Clone, Debug)]
struct PersistedIntentMigrationFailureV1 {
    blocker: PersistedIntentMigrationBlockerV1,
    status: PersistedLearningIntentMigrationStatusV1,
    invariant: &'static str,
}

fn migration_failure_v1(
    blocker: PersistedIntentMigrationBlockerV1,
    status: PersistedLearningIntentMigrationStatusV1,
    invariant: &'static str,
) -> PersistedIntentMigrationFailureV1 {
    PersistedIntentMigrationFailureV1 {
        blocker,
        status,
        invariant,
    }
}

fn stable_migration_values_v1<T: Clone + Ord>(values: &[T]) -> Vec<T> {
    values
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn migration_policy_digest_v1(intent: &AgentLearningIntentV0) -> String {
    stable_hash_string(&format!(
        "persisted-intent-policy-v1:{:?}:{:?}:{:?}:{:?}:{}:{:?}:{}:{}:{}:{}:{}",
        intent.agent_kind,
        intent.market_scopes,
        intent.required_datasets,
        intent.optional_datasets,
        intent.cadence,
        intent.lookback,
        intent.maximum_staleness_ms,
        intent.source_policy_digest,
        intent.feature_policy_digest,
        intent.label_policy_digest,
        intent.curriculum_policy_digest,
    ))
}

fn field_provenance_v1(
    field_name: &str,
    mut sources: Vec<MigratedIntentFieldSourceV1>,
    mut source_artifact_digests: Vec<String>,
    value: &impl std::fmt::Debug,
) -> MigratedIntentFieldProvenanceV1 {
    sources.sort();
    sources.dedup();
    source_artifact_digests.sort();
    source_artifact_digests.dedup();
    let value_digest = stable_hash_string(&format!(
        "persisted-intent-migrated-field-value-v1:{field_name}:{value:?}"
    ));
    let mut provenance = MigratedIntentFieldProvenanceV1 {
        provenance_version: INTENT_FIELD_PROVENANCE_VERSION_V1.to_string(),
        field_name: field_name.to_string(),
        sources,
        source_artifact_digests,
        value_digest,
        provenance_digest: String::new(),
    };
    provenance.provenance_digest = migrated_field_provenance_digest_v1(&provenance);
    provenance
}

fn migrated_field_provenance_digest_v1(value: &MigratedIntentFieldProvenanceV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}:{:?}:{}",
        value.provenance_version,
        value.field_name,
        value.sources,
        value.source_artifact_digests,
        value.value_digest,
    ))
}

fn policy_compatibility_proof_digest_v1(
    value: &PersistedIntentPolicyCompatibilityProofV1,
) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        INTENT_POLICY_PROOF_VERSION_V1,
        value.agent_id,
        value.legacy_policy_digest,
        value.current_policy_digest,
        value.required_datasets_equal,
        value.optional_datasets_equal,
        value.allowed_markets_equal,
        value.cadence_equal,
        value.lookback_equal,
        value.staleness_equal,
        value.semantically_compatible,
    ))
}

fn migration_proof_digest_v1(value: &PersistedLearningIntentMigrationProofV1) -> String {
    stable_hash_string(&format!(
        "{:?}",
        (
            (
                value.migration_version.as_str(),
                value.agent_id.as_str(),
                value.legacy_session_digest.as_str(),
                value.legacy_intent_digest.as_str(),
                value.gap_report_digest.as_str(),
                value.composite_registration_digest.as_str(),
                value.merged_snapshot_digest.as_str(),
            ),
            (
                value.policy_compatibility_proof_digest.as_str(),
                &value.field_provenance_digests,
                value.canonical_intent_digest.as_str(),
                value.canonical_view_digest.as_str(),
            ),
            (
                value.information_cutoff_unchanged,
                value.lookback_unchanged,
                value.policy_semantics_unchanged,
                value.evidence_set_unchanged,
                value.exclusions_unchanged,
                value.no_field_invented,
                value.migration_status,
            ),
        )
    ))
}

fn migration_journal_digest_v1(value: &PersistedLearningIntentMigrationJournalV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}",
        value.journal_version,
        value.agent_id,
        value.migration_proof_digest,
        value.canonical_intent_digest,
        value.canonical_view_digest,
        value.entry_count,
        value.network_requests,
        value.transport_constructions,
        value.credential_reads,
        value.prospective_reads,
        value.active_model_changes,
        value.status,
    ))
}

fn zero_intent_migration_safety_counters_v1() -> PersistedIntentMigrationSafetyCountersV1 {
    PersistedIntentMigrationSafetyCountersV1 {
        active_committee_count: 3,
        network_requests: 0,
        transport_constructions: 0,
        credential_reads: 0,
        prospective_artifact_reads: 0,
        prospective_label_reads: 0,
        future_evaluation_reads: 0,
        active_model_changes: 0,
        chair_decisions: 0,
        votes: 0,
        rewards: 0,
        penalties: 0,
        voice_changes: 0,
        promotions: 0,
        executions: 0,
    }
}

fn derive_persisted_learning_intent_migration_v1(
    sources: &PersistedIntentMigrationSourcesV1,
) -> Result<DerivedPersistedIntentMigrationV1, PersistedIntentMigrationFailureV1> {
    let session = &sources.legacy_session;
    let projection = &sources.legacy_projection;
    let policy = &sources.policy;
    let acquisition_gap = &sources.acquisition_gap;
    let canonical_gap = &sources.canonical_gap;
    let registration = &sources.composite_registration;
    let epoch = &sources.epoch_receipt;
    let snapshot = &sources.canonical_snapshot;
    let integrity = |invariant| {
        migration_failure_v1(
            PersistedIntentMigrationBlockerV1::IntegrityFailure,
            PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch,
            invariant,
        )
    };
    if session.session_digest != session_digest_v0(session) {
        return Err(integrity("legacy_session_digest"));
    }
    if session.agent_id != MOMENTUM_AGENT_ID_V1
        || session.agent_kind != AgentKind::MomentumTrendFast
        || projection.agent_id != session.agent_id
        || projection.agent_kind != session.agent_kind
        || acquisition_gap.agent_id != session.agent_id
        || canonical_gap.agent_id != session.agent_id
        || registration.target_agent_ids.len() != 1
        || registration.target_agent_ids.first() != Some(&session.agent_id)
    {
        return Err(integrity("agent_identity"));
    }
    if projection.intent_version != PERSISTED_LEARNING_INTENT_PROJECTION_VERSION_V1 {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::LegacyIntentMetadataIncomplete,
            PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch,
            "legacy_projection_version",
        ));
    }
    if projection.intent_digest.is_empty()
        || session.intent_digest.is_empty()
        || projection.intent_digest != session.intent_digest
        || acquisition_gap.intent_digest != session.intent_digest
        || canonical_gap.intent_digest != session.intent_digest
        || registration.intent_digest != session.intent_digest
    {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::PersistedIntentDigestMismatch,
            PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch,
            "legacy_intent_digest",
        ));
    }
    let blocker = if validate_agent_learning_intent_v0(projection, policy).is_err() {
        PersistedIntentMigrationBlockerV1::LegacySessionNotSelfDescribing
    } else {
        PersistedIntentMigrationBlockerV1::None
    };
    let first_failing_invariant = (blocker
        == PersistedIntentMigrationBlockerV1::LegacySessionNotSelfDescribing)
        .then(|| "intent_version".to_string());

    let required = stable_migration_values_v1(&projection.required_datasets);
    let optional = stable_migration_values_v1(&projection.optional_datasets);
    let markets = stable_migration_values_v1(&projection.market_scopes);
    let symbols = stable_migration_values_v1(&projection.symbols);
    let policy_required = stable_migration_values_v1(&policy.required_dataset_kinds);
    let policy_optional = stable_migration_values_v1(&policy.optional_dataset_kinds);
    let gap_required = stable_migration_values_v1(&canonical_gap.required_dataset_kinds);
    let gap_optional = stable_migration_values_v1(&canonical_gap.optional_dataset_kinds);
    let resolved_required =
        stable_migration_values_v1(&canonical_gap.resolved_required_dataset_kinds);
    let missing_optional =
        stable_migration_values_v1(&canonical_gap.missing_optional_dataset_kinds);
    let required_datasets_equal = required == policy_required && required == gap_required;
    let optional_datasets_equal = optional == policy_optional && optional == gap_optional;
    let markets_equal = markets.len() == 1
        && markets == stable_migration_values_v1(&canonical_gap.market_scopes)
        && markets == stable_migration_values_v1(&acquisition_gap.market_scopes)
        && markets.as_slice() == [registration.market_scope]
        && markets.as_slice() == [snapshot.market_scope]
        && markets
            .iter()
            .all(|market| policy.allowed_markets.contains(market));
    let cadence_equal = !projection.cadence.trim().is_empty()
        && projection.cadence == canonical_gap.cadence
        && projection.cadence == acquisition_gap.cadence
        && projection.cadence == registration.cadence
        && snapshot
            .compatibility
            .as_ref()
            .is_some_and(|value| value.cadence == projection.cadence);
    let lookback_equal = projection.lookback == canonical_gap.lookback
        && projection.lookback == acquisition_gap.lookback
        && projection.lookback == snapshot.requested_lookback
        && projection.lookback.bars == registration.required_row_count
        && projection.lookback.bars == snapshot.row_count
        && projection.lookback.bars == snapshot.normalized_dataset.rows.len()
        && projection.lookback.start_timestamp_ms == snapshot.actual_start_timestamp_ms
        && projection.lookback.end_timestamp_ms == snapshot.actual_end_timestamp_ms
        && projection.lookback.end_timestamp_ms == Some(projection.information_cutoff_ms);
    let staleness_equal = projection.maximum_staleness_ms == canonical_gap.maximum_staleness_ms
        && projection.maximum_staleness_ms == acquisition_gap.maximum_staleness_ms
        && projection.maximum_staleness_ms == policy.max_staleness_ms
        && snapshot
            .compatibility
            .as_ref()
            .is_some_and(|value| value.maximum_staleness_ms <= projection.maximum_staleness_ms)
        && (session.maximum_staleness_ms == 0
            || session.maximum_staleness_ms == projection.maximum_staleness_ms);
    let cutoff_unchanged = projection.information_cutoff_ms == session.information_cutoff_ms
        && projection.information_cutoff_ms == canonical_gap.information_cutoff_ms
        && projection.information_cutoff_ms == acquisition_gap.information_cutoff_ms
        && projection.information_cutoff_ms == registration.information_cutoff_ms
        && projection.information_cutoff_ms == snapshot.actual_end_timestamp_ms.unwrap_or_default()
        && snapshot.compatibility.as_ref().is_some_and(|value| {
            value.requested_cutoff_timestamp_ms == Some(projection.information_cutoff_ms)
        });
    if !cutoff_unchanged {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::CutoffMismatch,
            PersistedLearningIntentMigrationStatusV1::AmbiguousFieldProvenance,
            "information_cutoff_ms",
        ));
    }
    if !markets_equal {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::AmbiguousFieldProvenance,
            PersistedLearningIntentMigrationStatusV1::AmbiguousFieldProvenance,
            "market_scopes",
        ));
    }
    if symbols != stable_migration_values_v1(&canonical_gap.symbols)
        || symbols != stable_migration_values_v1(&acquisition_gap.symbols)
        || symbols != stable_migration_values_v1(&registration.symbols)
        || symbols != stable_migration_values_v1(&snapshot.symbols)
        || symbols.is_empty()
    {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::AmbiguousFieldProvenance,
            PersistedLearningIntentMigrationStatusV1::AmbiguousFieldProvenance,
            "symbols",
        ));
    }
    if !cadence_equal {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::AmbiguousFieldProvenance,
            PersistedLearningIntentMigrationStatusV1::AmbiguousFieldProvenance,
            "cadence",
        ));
    }
    if !lookback_equal || (session.lookback.bars != 0 && session.lookback != projection.lookback) {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::AmbiguousFieldProvenance,
            PersistedLearningIntentMigrationStatusV1::AmbiguousFieldProvenance,
            "lookback",
        ));
    }
    if !required_datasets_equal
        || resolved_required != required
        || !canonical_gap.missing_required_dataset_kinds.is_empty()
    {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::RequiredEvidenceMismatch,
            PersistedLearningIntentMigrationStatusV1::PolicyBindingMismatch,
            "required_datasets",
        ));
    }
    if !optional_datasets_equal {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::PolicyDigestMismatch,
            PersistedLearningIntentMigrationStatusV1::PolicyBindingMismatch,
            "optional_datasets",
        ));
    }
    if canonical_gap.status != CanonicalViewGapStatusV1::MissingOptionalEvidenceOnly
        || missing_optional != optional
        || !canonical_gap.resolved_optional_dataset_kinds.is_empty()
    {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::OptionalEvidenceOnlyMisclassified,
            PersistedLearningIntentMigrationStatusV1::CanonicalViewInvalid,
            "optional_evidence_classification",
        ));
    }
    if acquisition_gap.gap_digest != registration.gap_report_digest
        || acquisition_gap.status != CanonicalViewGapStatusV1::SegmentedAcquisitionRequired
        || epoch.registration_digest != registration.registration_digest
        || epoch.status != CompositeLearningEpochStatusV1::EvidenceAcquired
        || epoch.request_count != registration.maximum_total_requests
        || epoch.retry_count != 0
        || registration.maximum_concurrency != 1
        || registration.maximum_retries_per_segment != 0
        || epoch.merged_snapshot_digest.as_deref() != Some(snapshot.content_digest.as_str())
        || epoch
            .merged_provenance_digest
            .as_deref()
            .is_none_or(str::is_empty)
        || canonical_gap.usable_artifact_digests.len() != 1
        || canonical_gap.usable_artifact_digests.first() != Some(&snapshot.content_digest)
        || snapshot.dataset_kind != registration.dataset_kind
        || historical_replay_dataset_digest_v0(&snapshot.normalized_dataset)
            != snapshot.content_digest
        || !snapshot.quality_summary.accepted
        || !snapshot.sanitized
        || !snapshot.read_only
        || !snapshot.provenance.credential_free
    {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::CanonicalSnapshotBindingMismatch,
            PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch,
            "canonical_snapshot_binding",
        ));
    }
    if registration.excluded_timestamp_ms.is_empty()
        || registration.protected_registration_digests.is_empty()
        || snapshot.normalized_dataset.rows.iter().any(|row| {
            registration
                .excluded_timestamp_ms
                .contains(&row.timestamp_ms)
        })
    {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::IntegrityFailure,
            PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch,
            "protected_evidence_exclusions",
        ));
    }
    if !staleness_equal {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::PolicyDigestMismatch,
            PersistedLearningIntentMigrationStatusV1::PolicyBindingMismatch,
            "maximum_staleness_ms",
        ));
    }
    let data_intent = AgentDataIntent {
        agent_id: projection.agent_id.clone(),
        agent_kind: projection.agent_kind,
        market_scope: markets[0],
        symbols: symbols.clone(),
        required_datasets: required.clone(),
        optional_datasets: optional.clone(),
        lookback: projection.lookback.clone(),
        target_cadence: projection.cadence.clone(),
        max_staleness_ms: projection.maximum_staleness_ms,
        priority: DataPriority::Required,
        reason_codes: policy.reason_codes.clone(),
    };
    let canonical_intent = create_agent_learning_intent_v0(
        &LearningDataCallerV0::Agent(projection.agent_id.clone()),
        &data_intent,
        policy,
        projection.information_cutoff_ms,
    )
    .map_err(|_| {
        migration_failure_v1(
            PersistedIntentMigrationBlockerV1::IntegrityFailure,
            PersistedLearningIntentMigrationStatusV1::CanonicalIntentInvalid,
            "normal_intent_validator",
        )
    })?;
    if (!projection.source_policy_digest.is_empty()
        && projection.source_policy_digest != canonical_intent.source_policy_digest)
        || (!session.source_policy_digest.is_empty()
            && session.source_policy_digest != canonical_intent.source_policy_digest)
    {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::PolicyDigestMismatch,
            PersistedLearningIntentMigrationStatusV1::PolicyBindingMismatch,
            "source_policy_digest",
        ));
    }
    let legacy_policy_digest = migration_policy_digest_v1(projection);
    let current_policy_digest = migration_policy_digest_v1(&canonical_intent);
    let source_policy_compatible = projection.source_policy_digest.is_empty()
        || projection.source_policy_digest == canonical_intent.source_policy_digest;
    let semantically_compatible = required_datasets_equal
        && optional_datasets_equal
        && markets_equal
        && cadence_equal
        && lookback_equal
        && staleness_equal
        && source_policy_compatible;
    let mut policy_proof = PersistedIntentPolicyCompatibilityProofV1 {
        agent_id: projection.agent_id.clone(),
        legacy_policy_digest,
        current_policy_digest,
        required_datasets_equal,
        optional_datasets_equal,
        allowed_markets_equal: markets_equal,
        cadence_equal,
        lookback_equal,
        staleness_equal,
        semantically_compatible,
        proof_digest: String::new(),
    };
    policy_proof.proof_digest = policy_compatibility_proof_digest_v1(&policy_proof);
    if !policy_proof.semantically_compatible {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::PolicyDigestMismatch,
            PersistedLearningIntentMigrationStatusV1::PolicyBindingMismatch,
            "policy_compatibility",
        ));
    }

    let derived_private_state = derive_agent_private_learning_state_v0(&canonical_intent);
    let training_ledger_digest = if session.training_ledger_digest.is_empty() {
        derived_private_state.training_ledger_digest
    } else if session.training_ledger_digest == derived_private_state.training_ledger_digest {
        session.training_ledger_digest.clone()
    } else {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::PolicyDigestMismatch,
            PersistedLearningIntentMigrationStatusV1::PolicyBindingMismatch,
            "training_ledger_digest",
        ));
    };
    let private_state = AgentPrivateLearningStateV0 {
        agent_id: session.agent_id.clone(),
        private_namespace_digest: session.private_namespace_digest.clone(),
        training_ledger_digest,
    };
    if private_state.private_namespace_digest.is_empty()
        || private_state.training_ledger_digest.is_empty()
    {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::LegacyIntentMetadataIncomplete,
            PersistedLearningIntentMigrationStatusV1::CanonicalViewInvalid,
            "private_learning_state",
        ));
    }
    let artifact_ref = LearningDataArtifactRefV0 {
        artifact_digest: snapshot.content_digest.clone(),
        dataset_kind: snapshot.dataset_kind,
        visibility: LearningDataVisibilityV0::SharedCanonicalRaw,
        owner_agent_id: None,
        maximum_event_timestamp_ms: snapshot.actual_end_timestamp_ms.unwrap_or_default(),
    };
    let canonical_view = build_agent_learning_data_view_v0(
        &canonical_intent,
        policy,
        std::slice::from_ref(&artifact_ref),
        &private_state,
    )
    .map_err(|_| {
        migration_failure_v1(
            PersistedIntentMigrationBlockerV1::IntegrityFailure,
            PersistedLearningIntentMigrationStatusV1::CanonicalViewInvalid,
            "normal_view_builder",
        )
    })?;
    if canonical_view.source_artifact_digests.len() != 1
        || canonical_view.source_artifact_digests.first() != Some(&snapshot.content_digest)
        || canonical_view.visible_dataset_kinds != required
        || !canonical_view.missing_required_datasets.is_empty()
        || canonical_view.decision_gate != EvidenceDecisionGate::Ready
        || canonical_view.private_namespace_digest != session.private_namespace_digest
        || canonical_view.training_ledger_digest != private_state.training_ledger_digest
    {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::ViewDigestMismatch,
            PersistedLearningIntentMigrationStatusV1::CanonicalViewInvalid,
            "canonical_view_binding",
        ));
    }
    let input = build_agent_private_learning_input_from_persisted_view_v0(
        &canonical_intent,
        policy,
        &canonical_view,
        std::slice::from_ref(snapshot),
    )
    .map_err(|_| {
        migration_failure_v1(
            PersistedIntentMigrationBlockerV1::ViewDigestMismatch,
            PersistedLearningIntentMigrationStatusV1::CanonicalViewInvalid,
            "normal_persisted_view_reader",
        )
    })?;
    if input.resolution_status != AgentViewResolutionStatusV0::OptionalEvidenceUnavailable {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::OptionalEvidenceOnlyMisclassified,
            PersistedLearningIntentMigrationStatusV1::CanonicalViewInvalid,
            "normal_view_resolution",
        ));
    }

    let policy_digest = policy_proof.current_policy_digest.clone();
    let gap_digest = canonical_gap.gap_digest.clone();
    let registration_digest = registration.registration_digest.clone();
    let snapshot_digest = snapshot.content_digest.clone();
    let session_digest = session.session_digest.clone();
    let projection_digest = projection.intent_digest.clone();
    let private_digests = vec![session_digest.clone(), policy_digest.clone()];
    let source_digests =
        |sources: &[&str]| sources.iter().map(|value| (*value).to_string()).collect();
    let mut field_provenance = vec![
        field_provenance_v1(
            "agent_id",
            vec![
                MigratedIntentFieldSourceV1::LegacySession,
                MigratedIntentFieldSourceV1::LegacyIntentProjection,
                MigratedIntentFieldSourceV1::CanonicalGapReport,
                MigratedIntentFieldSourceV1::CompositeAcquisitionRegistration,
            ],
            source_digests(&[
                &session_digest,
                &projection_digest,
                &gap_digest,
                &registration_digest,
            ]),
            &canonical_intent.agent_id,
        ),
        field_provenance_v1(
            "agent_kind",
            vec![
                MigratedIntentFieldSourceV1::LegacySession,
                MigratedIntentFieldSourceV1::LegacyIntentProjection,
                MigratedIntentFieldSourceV1::VerifiedAgentPolicy,
            ],
            source_digests(&[&session_digest, &projection_digest, &policy_digest]),
            &canonical_intent.agent_kind,
        ),
        field_provenance_v1(
            "market_scopes",
            vec![
                MigratedIntentFieldSourceV1::LegacyIntentProjection,
                MigratedIntentFieldSourceV1::VerifiedAgentPolicy,
                MigratedIntentFieldSourceV1::CanonicalGapReport,
                MigratedIntentFieldSourceV1::CompositeAcquisitionRegistration,
                MigratedIntentFieldSourceV1::CanonicalSnapshot,
            ],
            source_digests(&[
                &projection_digest,
                &policy_digest,
                &gap_digest,
                &registration_digest,
                &snapshot_digest,
            ]),
            &canonical_intent.market_scopes,
        ),
        field_provenance_v1(
            "symbols",
            vec![
                MigratedIntentFieldSourceV1::LegacyIntentProjection,
                MigratedIntentFieldSourceV1::CanonicalGapReport,
                MigratedIntentFieldSourceV1::CompositeAcquisitionRegistration,
                MigratedIntentFieldSourceV1::CanonicalSnapshot,
            ],
            source_digests(&[
                &projection_digest,
                &gap_digest,
                &registration_digest,
                &snapshot_digest,
            ]),
            &canonical_intent.symbols,
        ),
        field_provenance_v1(
            "required_datasets",
            vec![
                MigratedIntentFieldSourceV1::LegacyIntentProjection,
                MigratedIntentFieldSourceV1::VerifiedAgentPolicy,
                MigratedIntentFieldSourceV1::CanonicalGapReport,
                MigratedIntentFieldSourceV1::CompositeAcquisitionRegistration,
                MigratedIntentFieldSourceV1::CanonicalSnapshot,
            ],
            source_digests(&[
                &projection_digest,
                &policy_digest,
                &gap_digest,
                &registration_digest,
                &snapshot_digest,
            ]),
            &canonical_intent.required_datasets,
        ),
        field_provenance_v1(
            "optional_datasets",
            vec![
                MigratedIntentFieldSourceV1::LegacyIntentProjection,
                MigratedIntentFieldSourceV1::VerifiedAgentPolicy,
                MigratedIntentFieldSourceV1::CanonicalGapReport,
            ],
            source_digests(&[&projection_digest, &policy_digest, &gap_digest]),
            &canonical_intent.optional_datasets,
        ),
        field_provenance_v1(
            "cadence",
            vec![
                MigratedIntentFieldSourceV1::LegacyIntentProjection,
                MigratedIntentFieldSourceV1::CanonicalGapReport,
                MigratedIntentFieldSourceV1::CompositeAcquisitionRegistration,
                MigratedIntentFieldSourceV1::CanonicalSnapshot,
            ],
            source_digests(&[
                &projection_digest,
                &gap_digest,
                &registration_digest,
                &snapshot_digest,
            ]),
            &canonical_intent.cadence,
        ),
        field_provenance_v1(
            "lookback",
            vec![
                MigratedIntentFieldSourceV1::LegacyIntentProjection,
                MigratedIntentFieldSourceV1::CanonicalGapReport,
                MigratedIntentFieldSourceV1::CompositeAcquisitionRegistration,
                MigratedIntentFieldSourceV1::CanonicalSnapshot,
            ],
            source_digests(&[
                &projection_digest,
                &gap_digest,
                &registration_digest,
                &snapshot_digest,
            ]),
            &canonical_intent.lookback,
        ),
        field_provenance_v1(
            "information_cutoff_ms",
            vec![
                MigratedIntentFieldSourceV1::LegacySession,
                MigratedIntentFieldSourceV1::LegacyIntentProjection,
                MigratedIntentFieldSourceV1::CanonicalGapReport,
                MigratedIntentFieldSourceV1::CompositeAcquisitionRegistration,
                MigratedIntentFieldSourceV1::CanonicalSnapshot,
            ],
            source_digests(&[
                &session_digest,
                &projection_digest,
                &gap_digest,
                &registration_digest,
                &snapshot_digest,
            ]),
            &canonical_intent.information_cutoff_ms,
        ),
        field_provenance_v1(
            "maximum_staleness_ms",
            vec![
                MigratedIntentFieldSourceV1::LegacyIntentProjection,
                MigratedIntentFieldSourceV1::VerifiedAgentPolicy,
                MigratedIntentFieldSourceV1::CanonicalGapReport,
                MigratedIntentFieldSourceV1::CanonicalSnapshot,
            ],
            source_digests(&[
                &projection_digest,
                &policy_digest,
                &gap_digest,
                &snapshot_digest,
            ]),
            &canonical_intent.maximum_staleness_ms,
        ),
        field_provenance_v1(
            "source_policy_digest",
            vec![MigratedIntentFieldSourceV1::VerifiedAgentPolicy],
            source_digests(&[&policy_digest]),
            &canonical_intent.source_policy_digest,
        ),
        field_provenance_v1(
            "feature_policy_digest",
            vec![MigratedIntentFieldSourceV1::VerifiedAgentPolicy],
            source_digests(&[&policy_digest]),
            &canonical_intent.feature_policy_digest,
        ),
        field_provenance_v1(
            "label_policy_digest",
            vec![MigratedIntentFieldSourceV1::VerifiedAgentPolicy],
            source_digests(&[&policy_digest]),
            &canonical_intent.label_policy_digest,
        ),
        field_provenance_v1(
            "curriculum_policy_digest",
            vec![MigratedIntentFieldSourceV1::VerifiedAgentPolicy],
            source_digests(&[&policy_digest]),
            &canonical_intent.curriculum_policy_digest,
        ),
        field_provenance_v1(
            "private_namespace_digest",
            vec![
                MigratedIntentFieldSourceV1::LegacySession,
                MigratedIntentFieldSourceV1::ExistingPrivateLearningState,
            ],
            private_digests.clone(),
            &canonical_view.private_namespace_digest,
        ),
        field_provenance_v1(
            "training_ledger_digest",
            vec![
                MigratedIntentFieldSourceV1::LegacySession,
                MigratedIntentFieldSourceV1::VerifiedAgentPolicy,
                MigratedIntentFieldSourceV1::ExistingPrivateLearningState,
            ],
            private_digests,
            &canonical_view.training_ledger_digest,
        ),
    ];
    field_provenance.sort_by(|left, right| left.field_name.cmp(&right.field_name));
    let no_field_invented = field_provenance.len() == 16
        && field_provenance.iter().all(|value| {
            !value.sources.is_empty()
                && !value.source_artifact_digests.is_empty()
                && value.provenance_digest == migrated_field_provenance_digest_v1(value)
        });
    if !no_field_invented {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::AmbiguousFieldProvenance,
            PersistedLearningIntentMigrationStatusV1::AmbiguousFieldProvenance,
            "field_provenance",
        ));
    }
    let mut field_provenance_digests = field_provenance
        .iter()
        .map(|value| value.provenance_digest.clone())
        .collect::<Vec<_>>();
    field_provenance_digests.sort();
    let mut migration_proof = PersistedLearningIntentMigrationProofV1 {
        migration_version: INTENT_MIGRATION_PROOF_VERSION_V1.to_string(),
        agent_id: canonical_intent.agent_id.clone(),
        legacy_session_digest: session.session_digest.clone(),
        legacy_intent_digest: session.intent_digest.clone(),
        gap_report_digest: canonical_gap.gap_digest.clone(),
        composite_registration_digest: registration.registration_digest.clone(),
        merged_snapshot_digest: snapshot.content_digest.clone(),
        policy_compatibility_proof_digest: policy_proof.proof_digest.clone(),
        field_provenance_digests,
        canonical_intent_digest: canonical_intent.intent_digest.clone(),
        canonical_view_digest: canonical_view.view_digest.clone(),
        information_cutoff_unchanged: cutoff_unchanged,
        lookback_unchanged: lookback_equal,
        policy_semantics_unchanged: policy_proof.semantically_compatible,
        evidence_set_unchanged: resolved_required == required && missing_optional == optional,
        exclusions_unchanged: true,
        no_field_invented,
        migration_status: PersistedLearningIntentMigrationStatusV1::Migrated,
        proof_digest: String::new(),
    };
    migration_proof.proof_digest = migration_proof_digest_v1(&migration_proof);
    let mut journal = PersistedLearningIntentMigrationJournalV1 {
        journal_version: INTENT_MIGRATION_JOURNAL_VERSION_V1.to_string(),
        agent_id: canonical_intent.agent_id.clone(),
        migration_proof_digest: migration_proof.proof_digest.clone(),
        canonical_intent_digest: canonical_intent.intent_digest.clone(),
        canonical_view_digest: canonical_view.view_digest.clone(),
        entry_count: 1,
        network_requests: 0,
        transport_constructions: 0,
        credential_reads: 0,
        prospective_reads: 0,
        active_model_changes: 0,
        status: PersistedLearningIntentMigrationStatusV1::Migrated,
        journal_digest: String::new(),
    };
    journal.journal_digest = migration_journal_digest_v1(&journal);
    Ok(DerivedPersistedIntentMigrationV1 {
        blocker,
        first_failing_invariant,
        canonical_intent,
        canonical_view,
        canonical_input: AgentPrivateLearningInputV1 {
            input,
            persisted_view_verified: true,
        },
        policy_proof,
        field_provenance,
        migration_proof,
        journal,
    })
}

fn load_latest_persisted_learning_session_v1(
    root: &Path,
    agent_id: &str,
) -> Result<AgentPrivateLearningSessionV0, PersistedIntentMigrationFailureV1> {
    let session_dir = root.join(agent_id).join("sessions");
    let entries = fs::read_dir(session_dir).map_err(|_| {
        migration_failure_v1(
            PersistedIntentMigrationBlockerV1::LegacyIntentMetadataIncomplete,
            PersistedLearningIntentMigrationStatusV1::SourceArtifactMissing,
            "legacy_session",
        )
    })?;
    let mut sessions = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|value| value == "pb"))
        .map(|path| {
            fs::read(path)
                .map_err(|_| {
                    migration_failure_v1(
                        PersistedIntentMigrationBlockerV1::IntegrityFailure,
                        PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch,
                        "legacy_session_read",
                    )
                })
                .and_then(|bytes| {
                    decode_session_protobuf_v0(&bytes).map_err(|_| {
                        migration_failure_v1(
                            PersistedIntentMigrationBlockerV1::IntegrityFailure,
                            PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch,
                            "legacy_session_protobuf",
                        )
                    })
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    sessions.retain(|session| session.agent_id == agent_id);
    sessions.sort_by(|left, right| {
        left.information_cutoff_ms
            .cmp(&right.information_cutoff_ms)
            .then_with(|| left.session_digest.cmp(&right.session_digest))
    });
    let cutoff = sessions
        .last()
        .map(|value| value.information_cutoff_ms)
        .ok_or_else(|| {
            migration_failure_v1(
                PersistedIntentMigrationBlockerV1::LegacyIntentMetadataIncomplete,
                PersistedLearningIntentMigrationStatusV1::SourceArtifactMissing,
                "legacy_session",
            )
        })?;
    let latest = sessions
        .into_iter()
        .filter(|value| value.information_cutoff_ms == cutoff)
        .collect::<Vec<_>>();
    if latest.len() != 1 {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::AmbiguousFieldProvenance,
            PersistedLearningIntentMigrationStatusV1::AmbiguousFieldProvenance,
            "legacy_session_selection",
        ));
    }
    Ok(latest.into_iter().next().unwrap())
}

fn load_gap_reports_for_migration_v1(
    root: &Path,
) -> Result<Vec<AgentCanonicalViewGapReportV1>, PersistedIntentMigrationFailureV1> {
    let directory = root.join("acquisition_v1/gap_reports");
    let entries = fs::read_dir(directory).map_err(|_| {
        migration_failure_v1(
            PersistedIntentMigrationBlockerV1::LegacyIntentMetadataIncomplete,
            PersistedLearningIntentMigrationStatusV1::SourceArtifactMissing,
            "canonical_gap_reports",
        )
    })?;
    let reports = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|value| value == "pb"))
        .map(|path| {
            fs::read(path)
                .map_err(|_| {
                    migration_failure_v1(
                        PersistedIntentMigrationBlockerV1::IntegrityFailure,
                        PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch,
                        "canonical_gap_report_read",
                    )
                })
                .and_then(|bytes| {
                    decode_agent_canonical_view_gap_report_protobuf_v1(&bytes).map_err(|_| {
                        migration_failure_v1(
                            PersistedIntentMigrationBlockerV1::IntegrityFailure,
                            PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch,
                            "canonical_gap_report_protobuf",
                        )
                    })
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if reports.is_empty() {
        Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::LegacyIntentMetadataIncomplete,
            PersistedLearningIntentMigrationStatusV1::SourceArtifactMissing,
            "canonical_gap_reports",
        ))
    } else {
        Ok(reports)
    }
}

fn latest_agent_canonical_view_gap_statuses_v1(
    gaps: impl IntoIterator<Item = crate::data::AgentCanonicalViewGapV1>,
) -> Result<BTreeMap<String, CanonicalViewGapStatusV1>, String> {
    let mut latest =
        BTreeMap::<String, (u64, usize, usize, usize, CanonicalViewGapStatusV1)>::new();
    for gap in gaps {
        let rank = (
            gap.information_cutoff_ms,
            gap.resolved_required_dataset_kinds.len(),
            gap.usable_artifact_digests.len(),
            gap.resolved_optional_dataset_kinds.len(),
        );
        match latest.get(&gap.agent_id) {
            Some((cutoff, required, usable, optional, _))
                if (*cutoff, *required, *usable, *optional) > rank => {}
            Some((cutoff, required, usable, optional, status))
                if (*cutoff, *required, *usable, *optional) == rank =>
            {
                if *status != gap.status {
                    return Err("canonical gap status is ambiguous".to_string());
                }
            }
            _ => {
                latest.insert(
                    gap.agent_id,
                    (
                        gap.information_cutoff_ms,
                        gap.resolved_required_dataset_kinds.len(),
                        gap.usable_artifact_digests.len(),
                        gap.resolved_optional_dataset_kinds.len(),
                        gap.status,
                    ),
                );
            }
        }
    }
    Ok(latest
        .into_iter()
        .map(|(agent_id, (_, _, _, _, status))| (agent_id, status))
        .collect())
}

pub fn load_latest_agent_canonical_view_gap_statuses_v1(
    root: &Path,
) -> Result<BTreeMap<String, CanonicalViewGapStatusV1>, String> {
    let reports = load_gap_reports_for_migration_v1(root)
        .map_err(|_| "canonical gap status artifacts unavailable".to_string())?;
    latest_agent_canonical_view_gap_statuses_v1(reports.into_iter().flat_map(|report| report.gaps))
}

fn load_persisted_intent_migration_sources_v1(
    root: &Path,
    snapshots: &[DataSnapshot],
) -> Result<PersistedIntentMigrationSourcesV1, PersistedIntentMigrationFailureV1> {
    let legacy_session = load_latest_persisted_learning_session_v1(root, MOMENTUM_AGENT_ID_V1)?;
    let legacy_projection = load_persisted_agent_learning_intents_v0(root, snapshots)
        .map_err(|_| {
            migration_failure_v1(
                PersistedIntentMigrationBlockerV1::LegacyIntentMetadataIncomplete,
                PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch,
                "legacy_intent_projection",
            )
        })?
        .into_iter()
        .find(|intent| intent.agent_id == MOMENTUM_AGENT_ID_V1)
        .ok_or_else(|| {
            migration_failure_v1(
                PersistedIntentMigrationBlockerV1::LegacyIntentMetadataIncomplete,
                PersistedLearningIntentMigrationStatusV1::SourceArtifactMissing,
                "legacy_intent_projection",
            )
        })?;
    let policy = default_agent_data_policies()
        .into_iter()
        .find(|policy| policy.agent_kind == legacy_session.agent_kind)
        .ok_or_else(|| {
            migration_failure_v1(
                PersistedIntentMigrationBlockerV1::PolicyDigestMismatch,
                PersistedLearningIntentMigrationStatusV1::SourceArtifactMissing,
                "verified_agent_policy",
            )
        })?;
    let composite_registration = read_composite_learning_registration_v1(root)
        .map_err(|_| {
            migration_failure_v1(
                PersistedIntentMigrationBlockerV1::IntegrityFailure,
                PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch,
                "composite_registration",
            )
        })?
        .ok_or_else(|| {
            migration_failure_v1(
                PersistedIntentMigrationBlockerV1::LegacyIntentMetadataIncomplete,
                PersistedLearningIntentMigrationStatusV1::SourceArtifactMissing,
                "composite_registration",
            )
        })?;
    let epoch_receipt = read_composite_epoch_receipt_v1(&composite_registration, root)
        .map_err(|_| {
            migration_failure_v1(
                PersistedIntentMigrationBlockerV1::IntegrityFailure,
                PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch,
                "composite_epoch_receipt",
            )
        })?
        .ok_or_else(|| {
            migration_failure_v1(
                PersistedIntentMigrationBlockerV1::LegacyIntentMetadataIncomplete,
                PersistedLearningIntentMigrationStatusV1::SourceArtifactMissing,
                "composite_epoch_receipt",
            )
        })?;
    let snapshot_digest = epoch_receipt
        .merged_snapshot_digest
        .as_ref()
        .ok_or_else(|| {
            migration_failure_v1(
                PersistedIntentMigrationBlockerV1::CanonicalSnapshotBindingMismatch,
                PersistedLearningIntentMigrationStatusV1::SourceArtifactMissing,
                "canonical_snapshot_digest",
            )
        })?;
    let canonical_snapshot = snapshots
        .iter()
        .find(|snapshot| snapshot.content_digest == *snapshot_digest)
        .cloned()
        .ok_or_else(|| {
            migration_failure_v1(
                PersistedIntentMigrationBlockerV1::CanonicalSnapshotBindingMismatch,
                PersistedLearningIntentMigrationStatusV1::SourceArtifactMissing,
                "canonical_snapshot",
            )
        })?;
    let reports = load_gap_reports_for_migration_v1(root)?;
    let mut acquisition_gaps = reports
        .iter()
        .flat_map(|report| report.gaps.iter())
        .filter(|gap| gap.gap_digest == composite_registration.gap_report_digest)
        .cloned()
        .collect::<Vec<_>>();
    acquisition_gaps.sort_by(|left, right| left.gap_digest.cmp(&right.gap_digest));
    acquisition_gaps.dedup();
    if acquisition_gaps.len() != 1 {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::AmbiguousFieldProvenance,
            PersistedLearningIntentMigrationStatusV1::AmbiguousFieldProvenance,
            "acquisition_gap",
        ));
    }
    let mut canonical_gaps = reports
        .iter()
        .flat_map(|report| report.gaps.iter())
        .filter(|gap| {
            gap.agent_id == MOMENTUM_AGENT_ID_V1
                && gap.status == CanonicalViewGapStatusV1::MissingOptionalEvidenceOnly
                && gap
                    .usable_artifact_digests
                    .contains(&canonical_snapshot.content_digest)
        })
        .cloned()
        .collect::<Vec<_>>();
    canonical_gaps.sort_by(|left, right| left.gap_digest.cmp(&right.gap_digest));
    canonical_gaps.dedup();
    if canonical_gaps.len() != 1 {
        return Err(migration_failure_v1(
            PersistedIntentMigrationBlockerV1::AmbiguousFieldProvenance,
            PersistedLearningIntentMigrationStatusV1::AmbiguousFieldProvenance,
            "canonical_gap",
        ));
    }
    Ok(PersistedIntentMigrationSourcesV1 {
        legacy_session,
        legacy_projection,
        policy,
        acquisition_gap: acquisition_gaps.into_iter().next().unwrap(),
        canonical_gap: canonical_gaps.into_iter().next().unwrap(),
        composite_registration,
        epoch_receipt,
        canonical_snapshot,
    })
}

fn collect_migration_protected_bytes_v1(
    path: &Path,
    protected: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), String> {
    if path.ends_with("intent_migration_v1") {
        return Ok(());
    }
    if path.is_file() {
        protected.push((
            path.to_path_buf(),
            fs::read(path).map_err(|_| "protected learning artifact read failed".to_string())?,
        ));
        return Ok(());
    }
    if !path.is_dir() {
        return Ok(());
    }
    let mut children = fs::read_dir(path)
        .map_err(|_| "protected learning directory read failed".to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    children.sort();
    for child in children {
        collect_migration_protected_bytes_v1(&child, protected)?;
    }
    Ok(())
}

fn migration_report_digest_v1(report: &PersistedLearningIntentMigrationReportV1) -> String {
    stable_hash_string(&format!(
        "{:?}",
        (
            (
                report.report_version.as_str(),
                report.mode,
                report.agent_id.as_str(),
                report.blocker,
                &report.first_failing_invariant,
                report.status,
            ),
            (
                &report.legacy_session_digest,
                &report.legacy_intent_digest,
                &report.canonical_gap_digest,
                &report.composite_registration_digest,
                &report.canonical_snapshot_digest,
                &report.canonical_intent_digest,
                &report.canonical_view_digest,
                &report.policy_compatibility_proof_digest,
                &report.migration_proof_digest,
                &report.migration_journal_digest,
            ),
            (
                report.field_provenance_count,
                report.required_evidence_complete,
                report.optional_evidence_unavailable,
                report.normal_validator_passed,
                report.normal_view_builder_passed,
                report.artifacts_written,
                report.duplicate_artifact_count,
                report.storage_failure_count,
                report.protected_artifacts_unchanged,
                report.active_state_unchanged,
                &report.safety_counters,
            ),
        )
    ))
}

fn failed_migration_execution_v1(
    mode: AgentPrivateLearningRunModeV0,
    failure: PersistedIntentMigrationFailureV1,
    active_state_unchanged: bool,
) -> PersistedLearningIntentMigrationExecutionV1 {
    let mut report = PersistedLearningIntentMigrationReportV1 {
        report_version: INTENT_MIGRATION_VERSION_V1.to_string(),
        mode,
        agent_id: MOMENTUM_AGENT_ID_V1.to_string(),
        blocker: failure.blocker,
        first_failing_invariant: Some(failure.invariant.to_string()),
        status: failure.status,
        legacy_session_digest: None,
        legacy_intent_digest: None,
        canonical_gap_digest: None,
        composite_registration_digest: None,
        canonical_snapshot_digest: None,
        canonical_intent_digest: None,
        canonical_view_digest: None,
        policy_compatibility_proof_digest: None,
        migration_proof_digest: None,
        migration_journal_digest: None,
        field_provenance_count: 0,
        required_evidence_complete: false,
        optional_evidence_unavailable: false,
        normal_validator_passed: false,
        normal_view_builder_passed: false,
        artifacts_written: 0,
        duplicate_artifact_count: 0,
        storage_failure_count: usize::from(mode == AgentPrivateLearningRunModeV0::ExecuteLocal),
        protected_artifacts_unchanged: true,
        active_state_unchanged,
        safety_counters: zero_intent_migration_safety_counters_v1(),
        report_digest: String::new(),
    };
    report.report_digest = migration_report_digest_v1(&report);
    PersistedLearningIntentMigrationExecutionV1 {
        report,
        canonical_input: None,
    }
}

pub fn run_persisted_learning_intent_migration_v1(
    root: &Path,
    snapshots: &[DataSnapshot],
    mode: AgentPrivateLearningRunModeV0,
) -> PersistedLearningIntentMigrationExecutionV1 {
    let before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let mut protected_before = Vec::new();
    if mode == AgentPrivateLearningRunModeV0::ExecuteLocal
        && collect_migration_protected_bytes_v1(root, &mut protected_before).is_err()
    {
        return failed_migration_execution_v1(
            mode,
            migration_failure_v1(
                PersistedIntentMigrationBlockerV1::IntegrityFailure,
                PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch,
                "protected_artifact_snapshot",
            ),
            true,
        );
    }
    let sources = match load_persisted_intent_migration_sources_v1(root, snapshots) {
        Ok(sources) => sources,
        Err(failure) => {
            let after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
            return failed_migration_execution_v1(mode, failure, before == after);
        }
    };
    let derived = match derive_persisted_learning_intent_migration_v1(&sources) {
        Ok(derived) => derived,
        Err(failure) => {
            let after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
            return failed_migration_execution_v1(mode, failure, before == after);
        }
    };
    let already_migrated = read_persisted_learning_intent_migration_v1(root, snapshots)
        .is_ok_and(|persisted| persisted == derived.canonical_input);
    let mut artifacts_written = 0;
    let mut duplicate_artifact_count = 0;
    let mut storage_failure_count = 0;
    if mode == AgentPrivateLearningRunModeV0::DryRun {
        if migration_round_trip_v1(&derived).is_err() {
            storage_failure_count = 1;
        }
    } else if mode == AgentPrivateLearningRunModeV0::ExecuteLocal {
        match persist_persisted_learning_intent_migration_v1(&derived, root) {
            Ok((written, duplicates)) => {
                artifacts_written = written;
                duplicate_artifact_count = duplicates;
            }
            Err(_) => storage_failure_count = 1,
        }
    }
    let canonical_input =
        if mode == AgentPrivateLearningRunModeV0::ExecuteLocal && storage_failure_count == 0 {
            read_persisted_learning_intent_migration_v1(root, snapshots).ok()
        } else {
            Some(derived.canonical_input.clone())
        };
    let protected_artifacts_unchanged = if mode == AgentPrivateLearningRunModeV0::ExecuteLocal {
        let mut protected_after = Vec::new();
        collect_migration_protected_bytes_v1(root, &mut protected_after).is_ok()
            && protected_before == protected_after
    } else {
        true
    };
    let after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let status = if already_migrated {
        PersistedLearningIntentMigrationStatusV1::AlreadyMigrated
    } else {
        PersistedLearningIntentMigrationStatusV1::Migrated
    };
    let mut report = PersistedLearningIntentMigrationReportV1 {
        report_version: INTENT_MIGRATION_VERSION_V1.to_string(),
        mode,
        agent_id: MOMENTUM_AGENT_ID_V1.to_string(),
        blocker: derived.blocker,
        first_failing_invariant: derived.first_failing_invariant.clone(),
        status,
        legacy_session_digest: Some(sources.legacy_session.session_digest.clone()),
        legacy_intent_digest: Some(sources.legacy_session.intent_digest.clone()),
        canonical_gap_digest: Some(sources.canonical_gap.gap_digest.clone()),
        composite_registration_digest: Some(
            sources.composite_registration.registration_digest.clone(),
        ),
        canonical_snapshot_digest: Some(sources.canonical_snapshot.content_digest.clone()),
        canonical_intent_digest: Some(derived.canonical_intent.intent_digest.clone()),
        canonical_view_digest: Some(derived.canonical_view.view_digest.clone()),
        policy_compatibility_proof_digest: Some(derived.policy_proof.proof_digest.clone()),
        migration_proof_digest: Some(derived.migration_proof.proof_digest.clone()),
        migration_journal_digest: Some(derived.journal.journal_digest.clone()),
        field_provenance_count: derived.field_provenance.len(),
        required_evidence_complete: true,
        optional_evidence_unavailable: true,
        normal_validator_passed: validate_agent_learning_intent_v0(
            &derived.canonical_intent,
            &sources.policy,
        )
        .is_ok(),
        normal_view_builder_passed: canonical_input.is_some(),
        artifacts_written,
        duplicate_artifact_count,
        storage_failure_count,
        protected_artifacts_unchanged,
        active_state_unchanged: before == after,
        safety_counters: zero_intent_migration_safety_counters_v1(),
        report_digest: String::new(),
    };
    report.report_digest = migration_report_digest_v1(&report);
    PersistedLearningIntentMigrationExecutionV1 {
        report,
        canonical_input,
    }
}

fn resolve_snapshot_for_request_v0(
    request: &ReadOnlyProviderRequest,
    snapshots: &[DataSnapshot],
) -> Result<Option<DataSnapshot>, AgentViewResolutionStatusV0> {
    let mut candidates = Vec::new();
    let mut rejected_status = None;
    for snapshot in snapshots {
        if snapshot.dataset_kind != request.dataset_kind
            || snapshot.market_scope != request.market_scope
            || stabilized_strings_v0(&snapshot.symbols) != request.symbols
        {
            continue;
        }
        match validate_snapshot_for_request_v0(snapshot, request) {
            Ok(true) => candidates.push(snapshot),
            Ok(false) => {}
            Err(status) => rejected_status = Some(status),
        }
    }
    if candidates.is_empty() {
        return rejected_status.map_or(Ok(None), Err);
    }
    candidates.sort_by(|left, right| {
        right
            .fetched_at_ms
            .cmp(&left.fetched_at_ms)
            .then_with(|| right.row_count.cmp(&left.row_count))
            .then_with(|| left.content_digest.cmp(&right.content_digest))
    });
    let selected = candidates[0];
    if candidates.iter().skip(1).any(|candidate| {
        candidate.fetched_at_ms == selected.fetched_at_ms
            && candidate.row_count == selected.row_count
            && candidate.content_digest != selected.content_digest
    }) {
        return Err(AgentViewResolutionStatusV0::AmbiguousEquivalentArtifacts);
    }
    Ok(Some(selected.clone()))
}

fn validate_snapshot_for_request_v0(
    snapshot: &DataSnapshot,
    request: &ReadOnlyProviderRequest,
) -> Result<bool, AgentViewResolutionStatusV0> {
    let exact_request = snapshot.request_key == request.request_key
        && snapshot.requested_lookback == request.lookback;
    let compatible_request = snapshot
        .compatibility
        .as_ref()
        .is_some_and(|compatibility| {
            snapshot.requested_lookback == request.lookback
                && compatibility.cadence == request.cadence
                && compatibility.adjustment_semantics
                    == expected_adjustment_semantics_v0(request.dataset_kind)
                && compatibility.source_schema == "application/x-soma-normalized-dataset"
                && compatibility.requested_cutoff_timestamp_ms == request.lookback.end_timestamp_ms
                && compatibility.maximum_staleness_ms <= request.max_staleness_ms
                && compatibility.all_rows_finalized
        });
    if !exact_request && !compatible_request {
        return Ok(false);
    }
    let digest = historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
    if snapshot.schema_version != 1
        || snapshot.content_digest != digest
        || snapshot.row_count != snapshot.normalized_dataset.rows.len()
        || snapshot.quality_summary.row_count != snapshot.row_count
        || !snapshot.quality_summary.accepted
        || !snapshot.sanitized
        || !snapshot.read_only
        || !snapshot.provenance.sanitized
        || !snapshot.provenance.credential_free
        || snapshot.provenance.provider_id != snapshot.provider_id
        || snapshot.provenance.source_type == SnapshotSourceType::LocalSnapshotReplay
        || snapshot.normalized_dataset.rows.is_empty()
        || snapshot.actual_start_timestamp_ms
            != snapshot
                .normalized_dataset
                .rows
                .first()
                .map(|row| row.timestamp_ms)
        || snapshot.actual_end_timestamp_ms
            != snapshot
                .normalized_dataset
                .rows
                .last()
                .map(|row| row.timestamp_ms)
        || snapshot
            .normalized_dataset
            .rows
            .windows(2)
            .any(|pair| pair[0].timestamp_ms >= pair[1].timestamp_ms)
        || snapshot.normalized_dataset.rows.iter().any(|row| {
            row.symbol != snapshot.normalized_dataset.symbol || !finite_valid_row_v0(row)
        })
    {
        return Err(AgentViewResolutionStatusV0::IntegrityFailure);
    }
    let Some(actual_end) = snapshot.actual_end_timestamp_ms else {
        return Err(AgentViewResolutionStatusV0::IntegrityFailure);
    };
    if actual_end > request.lookback.end_timestamp_ms.unwrap_or_default() {
        return Err(AgentViewResolutionStatusV0::CutoffLeakage);
    }
    if request
        .lookback
        .end_timestamp_ms
        .is_some_and(|cutoff| cutoff.saturating_sub(actual_end) > request.max_staleness_ms)
    {
        return Ok(false);
    }
    Ok(true)
}

fn expected_adjustment_semantics_v0(kind: DatasetKind) -> SnapshotAdjustmentSemanticsV1 {
    match kind {
        DatasetKind::DailyOhlcv | DatasetKind::CryptoDailyOhlcv => {
            SnapshotAdjustmentSemanticsV1::Unadjusted
        }
        DatasetKind::AdjustedDailyOhlcv => SnapshotAdjustmentSemanticsV1::Adjusted,
        _ => SnapshotAdjustmentSemanticsV1::NotApplicable,
    }
}

fn stabilized_strings_v0(values: &[String]) -> Vec<String> {
    let mut values = values.to_vec();
    values.sort();
    values.dedup();
    values
}

pub fn run_agent_private_learning_sessions_v0(
    inputs: &[AgentPrivateLearningSessionInputV0],
    mode: AgentPrivateLearningRunModeV0,
) -> AgentPrivateLearningSessionsReportV0 {
    let before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let registry = agent_trainer_capability_registry_v0();
    let results = inputs
        .iter()
        .map(|input| run_one_session_v0(input, &registry, mode))
        .collect::<Vec<_>>();
    let after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let mut report = AgentPrivateLearningSessionsReportV0 {
        report_version: "agent-private-learning-sessions-report-v0".to_string(),
        mode,
        capability_registry: registry,
        results,
        safety_counters: zero_safety_counters_v0(),
        active_state_unchanged: before == after,
        duplicate_artifact_count: 0,
        storage_failure_count: 0,
        report_digest: String::new(),
    };
    report.report_digest = report_digest_v0(&report);
    report
}

pub fn run_agent_private_learning_candidates_v1(
    inputs: &[AgentPrivateLearningInputV1],
    mode: AgentPrivateLearningRunModeV0,
) -> AgentCandidateFamiliesReportV1 {
    let before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let registry = agent_trainer_capability_registry_v0();
    let mut results = registry
        .capabilities
        .iter()
        .map(|capability| {
            let input = inputs
                .iter()
                .find(|input| input.input.intent.agent_id == capability.agent_id);
            run_one_candidate_family_v1(input, capability, mode)
        })
        .collect::<Vec<_>>();
    results.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
    let after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let mut report = AgentCandidateFamiliesReportV1 {
        report_version: "agent-candidate-families-report-v1".to_string(),
        mode,
        results,
        safety_counters: zero_agent_learning_safety_counters_v1(),
        active_state_unchanged: before == after,
        duplicate_artifact_count: 0,
        storage_failure_count: 0,
        report_digest: String::new(),
    };
    report.report_digest = candidate_families_report_digest_v1(&report);
    report
}

#[derive(Deserialize)]
struct MomentumReservationCapsuleMetadataV1 {
    capsule_digest: String,
    prediction_horizon: usize,
}

#[derive(Deserialize)]
struct MomentumReservationEventMetadataV1 {
    required_label_maturity_timestamp_ms: u64,
}

#[derive(Deserialize)]
struct MomentumReservationJournalMetadataV1 {
    events: Vec<MomentumReservationEventMetadataV1>,
}

#[derive(Deserialize)]
struct MomentumReservationMetadataV1 {
    capsule: MomentumReservationCapsuleMetadataV1,
    journal: MomentumReservationJournalMetadataV1,
}

#[derive(Deserialize)]
struct RiskReservationCapsuleMetadataV1 {
    capsule_digest: String,
    prediction_horizon: usize,
}

#[derive(Deserialize)]
struct RiskReservationJournalMetadataV1 {
    sealed_event_timestamps: Vec<u64>,
}

#[derive(Deserialize)]
struct RiskReservationMetadataV1 {
    capsule: RiskReservationCapsuleMetadataV1,
    journal: RiskReservationJournalMetadataV1,
}

pub fn load_protected_evaluation_reservation_v1(
    local_config_root: &Path,
) -> Result<ProtectedEvaluationReservationV1, String> {
    let opening = super::read_prospective_one_time_opening_registration_v0(
        &local_config_root.join("prospective_one_time_opening_registration_v0.json"),
    )?;
    if opening.registration_digest.is_empty()
        || opening.maximum_future_requests != 1
        || opening.maximum_concurrency != 1
        || opening.maximum_retries != 0
        || !opening.explicit_opening_authorization_required
        || !opening.one_time_opening_required
        || !opening.early_opening_forbidden
        || !opening.duplicate_opening_forbidden
        || !opening.interim_metrics_forbidden
        || opening.network_execution_allowed_this_sprint
        || opening.label_access_allowed_this_sprint
        || opening.reward_application_allowed
    {
        return Err("protected prospective registration rejected".to_string());
    }
    let momentum: MomentumReservationMetadataV1 = serde_json::from_reader(
        File::open(local_config_root.join("prospective_shadow_challenge_v0.json"))
            .map_err(|_| "protected momentum metadata unavailable".to_string())?,
    )
    .map_err(|_| "protected momentum metadata rejected".to_string())?;
    let risk: RiskReservationMetadataV1 = serde_json::from_reader(
        File::open(local_config_root.join("cycle_risk_prospective_local_state_v0.json"))
            .map_err(|_| "protected risk metadata unavailable".to_string())?,
    )
    .map_err(|_| "protected risk metadata rejected".to_string())?;
    if momentum.capsule.capsule_digest.is_empty()
        || momentum.capsule.prediction_horizon == 0
        || risk.capsule.capsule_digest.is_empty()
        || risk.capsule.prediction_horizon == 0
    {
        return Err("protected prospective capsule metadata rejected".to_string());
    }
    let mut reserved = BTreeSet::new();
    for event in momentum.journal.events {
        if event.required_label_maturity_timestamp_ms == 0 {
            return Err("protected momentum maturity timestamp rejected".to_string());
        }
        reserved.insert(event.required_label_maturity_timestamp_ms);
    }
    for event_timestamp in risk.journal.sealed_event_timestamps {
        if event_timestamp == 0 {
            return Err("protected risk event timestamp rejected".to_string());
        }
        for offset in 1..=risk.capsule.prediction_horizon {
            let offset =
                u64::try_from(offset).map_err(|_| "protected risk horizon rejected".to_string())?;
            reserved.insert(
                event_timestamp
                    .checked_add(offset.saturating_mul(DAILY_CADENCE_MS_V1))
                    .ok_or_else(|| "protected risk timestamp overflow".to_string())?,
            );
        }
    }
    if reserved.is_empty() {
        return Err("protected prospective reservation is empty".to_string());
    }
    let mut protected_registration_digests = vec![
        opening.registration_digest,
        momentum.capsule.capsule_digest,
        risk.capsule.capsule_digest,
    ];
    protected_registration_digests.sort();
    protected_registration_digests.dedup();
    let reserved_timestamp_ms = reserved.into_iter().collect::<Vec<_>>();
    let provider_finality_boundary_ms = reserved_timestamp_ms
        .last()
        .copied()
        .and_then(|timestamp| timestamp.checked_add(DAILY_CADENCE_MS_V1))
        .ok_or_else(|| "protected finality boundary unavailable".to_string())?;
    let mut reservation = ProtectedEvaluationReservationV1 {
        protected_registration_digests,
        reserved_timestamp_ms,
        cadence_ms: DAILY_CADENCE_MS_V1,
        provider_finality_boundary_ms,
        reservation_digest: String::new(),
    };
    reservation.reservation_digest = protected_reservation_digest_v1(&reservation);
    validate_protected_reservation_v1(&reservation)?;
    Ok(reservation)
}

pub fn run_agent_candidate_evaluations_v1(
    families: &AgentCandidateFamiliesReportV1,
    inputs: &[AgentPrivateLearningInputV1],
    reservation: &ProtectedEvaluationReservationV1,
    mode: AgentPrivateLearningRunModeV0,
) -> AgentCandidateEvaluationsReportV1 {
    let before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let reservation_valid = validate_protected_reservation_v1(reservation).is_ok();
    let registry = agent_trainer_capability_registry_v0();
    let mut results = registry
        .capabilities
        .iter()
        .map(|capability| {
            let family = families
                .results
                .iter()
                .find(|result| result.agent_id == capability.agent_id);
            let input = inputs
                .iter()
                .find(|input| input.input.intent.agent_id == capability.agent_id);
            register_one_candidate_family_v1(
                family,
                input,
                capability,
                reservation_valid.then_some(reservation),
            )
        })
        .collect::<Vec<_>>();
    results.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
    let after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let mut report = AgentCandidateEvaluationsReportV1 {
        report_version: "agent-candidate-evaluations-report-v1".to_string(),
        mode,
        results,
        safety_counters: zero_agent_learning_safety_counters_v1(),
        active_state_unchanged: before == after,
        duplicate_artifact_count: 0,
        storage_failure_count: 0,
        report_digest: String::new(),
    };
    report.report_digest = candidate_evaluations_report_digest_v1(&report);
    report
}

fn register_one_candidate_family_v1(
    family_result: Option<&AgentCandidateFamilyResultV1>,
    input: Option<&AgentPrivateLearningInputV1>,
    capability: &AgentTrainerCapabilityV0,
    reservation: Option<&ProtectedEvaluationReservationV1>,
) -> AgentCandidateEvaluationResultV1 {
    let unavailable = |status, code: &str| AgentCandidateEvaluationResultV1 {
        agent_id: capability.agent_id.clone(),
        family_digest: None,
        session_digest: None,
        exclusion: None,
        registration: None,
        journal: None,
        status,
        sanitized_error_code: Some(code.to_string()),
    };
    if !capability.supports_training
        || capability.trainer_kind == AgentTrainerKindV0::ValueQualityUnavailable
    {
        return unavailable(
            CandidateEvaluationRegistrationStatusV1::CandidateUnavailable,
            "candidate_family_unavailable",
        );
    }
    let (Some(family_result), Some(input), Some(reservation)) = (family_result, input, reservation)
    else {
        return unavailable(
            CandidateEvaluationRegistrationStatusV1::CandidateUnavailable,
            "registration_inputs_unavailable",
        );
    };
    let (Some(session), Some(projection), Some(family), Some(ledger)) = (
        family_result.session.as_ref(),
        family_result.projection.as_ref(),
        family_result.family.as_ref(),
        family_result.usage_ledger.as_ref(),
    ) else {
        return unavailable(
            CandidateEvaluationRegistrationStatusV1::CandidateUnavailable,
            "candidate_family_unavailable",
        );
    };
    let blocked = |status, code: &str| AgentCandidateEvaluationResultV1 {
        agent_id: capability.agent_id.clone(),
        family_digest: Some(family.family_digest.clone()),
        session_digest: Some(session.session_digest.clone()),
        exclusion: None,
        registration: None,
        journal: None,
        status,
        sanitized_error_code: Some(code.to_string()),
    };
    if validate_session_v1(session).is_err()
        || session.status != AgentLearningSessionStatusV1::CandidateFamilyFrozen
        || session.agent_id != capability.agent_id
    {
        return blocked(
            CandidateEvaluationRegistrationStatusV1::SessionInvalid,
            "v1_session_invalid",
        );
    }
    if !input.persisted_view_verified
        || input.input.view.decision_gate != EvidenceDecisionGate::Ready
        || validate_agent_learning_data_view_v0(&input.input.view).is_err()
        || input.input.view.view_digest != session.view_digest
        || input.input.intent.intent_digest != session.intent_digest
    {
        return blocked(
            CandidateEvaluationRegistrationStatusV1::ViewInvalid,
            "complete_persisted_view_invalid",
        );
    }
    if validate_projection_v1(projection).is_err()
        || projection.projection_digest != session.projection_digest
        || projection.source_view_digest != session.view_digest
    {
        return blocked(
            CandidateEvaluationRegistrationStatusV1::ProjectionInvalid,
            "v1_projection_invalid",
        );
    }
    if validate_candidate_family_v1(family).is_err()
        || family.session_digest != session.session_digest
        || family.view_digest != session.view_digest
        || family.projection_digest != projection.projection_digest
        || family_result.status != AgentLearningSessionStatusV1::CandidateFamilyFrozen
    {
        return blocked(
            CandidateEvaluationRegistrationStatusV1::FamilyInvalid,
            "v1_family_invalid",
        );
    }
    if validate_usage_ledger_v1(ledger).is_err()
        || ledger.session_digest != session.session_digest
        || ledger.family_digest != family.family_digest
    {
        return blocked(
            CandidateEvaluationRegistrationStatusV1::HistoricalTestAccessDetected,
            "v1_usage_ledger_invalid",
        );
    }
    if family.participants.len() < 2 {
        return blocked(
            CandidateEvaluationRegistrationStatusV1::InsufficientParticipants,
            "insufficient_frozen_participants",
        );
    }
    let participant_digests = family
        .participants
        .iter()
        .map(|participant| participant.participant_digest.clone())
        .collect::<BTreeSet<_>>();
    let mut qualified_receipts = family_result
        .qualification_receipts
        .iter()
        .filter(|receipt| participant_digests.contains(&receipt.participant_digest))
        .cloned()
        .collect::<Vec<_>>();
    qualified_receipts.sort_by(|left, right| left.receipt_digest.cmp(&right.receipt_digest));
    if qualified_receipts.len() != family.participants.len()
        || qualified_receipts.iter().any(|receipt| {
            receipt.qualification_status != ValidationQualificationStatusV1::Qualified
                || validate_qualification_receipt_v1(receipt).is_err()
        })
        || qualified_receipts
            .iter()
            .map(|receipt| receipt.receipt_digest.clone())
            .collect::<Vec<_>>()
            != family.validation_qualification_receipts
    {
        return blocked(
            CandidateEvaluationRegistrationStatusV1::QualificationBlocked,
            "validation_qualification_invalid",
        );
    }
    let mut exclusion = EvaluationEvidenceExclusionV1 {
        protected_registration_digests: reservation.protected_registration_digests.clone(),
        excluded_timestamp_ms: reservation.reserved_timestamp_ms.clone(),
        excluded_range_digests: reservation
            .reserved_timestamp_ms
            .iter()
            .map(|timestamp| {
                stable_hash_string(&format!(
                    "protected-prospective-timestamp-range-v1:{timestamp}:{}",
                    reservation.cadence_ms
                ))
            })
            .collect(),
        exclusion_digest: String::new(),
    };
    exclusion.excluded_range_digests.sort();
    exclusion.exclusion_digest = evaluation_exclusion_digest_v1(&exclusion);
    if validate_evaluation_exclusion_v1(&exclusion).is_err() {
        return blocked(
            CandidateEvaluationRegistrationStatusV1::ExclusionInvalid,
            "evaluation_exclusion_invalid",
        );
    }
    let candidate_source_end = input
        .input
        .artifacts
        .iter()
        .filter(|artifact| {
            session
                .source_artifact_digests
                .contains(&artifact.artifact_ref.artifact_digest)
        })
        .filter_map(|artifact| artifact.snapshot.actual_end_timestamp_ms)
        .max();
    let Some(candidate_source_next) =
        candidate_source_end.and_then(|timestamp| timestamp.checked_add(reservation.cadence_ms))
    else {
        return blocked(
            CandidateEvaluationRegistrationStatusV1::ViewInvalid,
            "candidate_source_boundary_unavailable",
        );
    };
    let Some(reserved_next) = reservation
        .reserved_timestamp_ms
        .last()
        .and_then(|timestamp| timestamp.checked_add(reservation.cadence_ms))
    else {
        return blocked(
            CandidateEvaluationRegistrationStatusV1::ExclusionInvalid,
            "protected_boundary_unavailable",
        );
    };
    let minimum_accepted_timestamp_ms = candidate_source_next
        .max(reserved_next)
        .max(reservation.provider_finality_boundary_ms);
    let mut required_dataset_kinds = input.input.intent.required_datasets.clone();
    required_dataset_kinds.sort();
    required_dataset_kinds.dedup();
    let mut participant_digests = participant_digests.into_iter().collect::<Vec<_>>();
    participant_digests.sort();
    let qualification_receipt_digests = qualified_receipts
        .iter()
        .map(|receipt| receipt.receipt_digest.clone())
        .collect::<Vec<_>>();
    let mut registration = AgentCandidateEvaluationRegistrationV1 {
        registration_version: EVALUATION_REGISTRATION_VERSION_V1.to_string(),
        agent_id: session.agent_id.clone(),
        family_digest: family.family_digest.clone(),
        session_digest: session.session_digest.clone(),
        usage_ledger_digest: ledger.ledger_digest.clone(),
        participant_digests,
        qualification_receipt_digests,
        exclusion_digest: exclusion.exclusion_digest.clone(),
        minimum_accepted_timestamp_ms,
        required_dataset_kinds,
        source_policy_digest: session.source_policy_digest.clone(),
        finality_policy_digest: stable_hash_string(&format!(
            "finalized-daily-only-v1:{}:{}",
            reservation.cadence_ms, reservation.provider_finality_boundary_ms
        )),
        label_policy_digest: session.label_policy_digest.clone(),
        metric_policy_digest: stable_hash_string("future-common-timestamp-brier-v1"),
        support_policy_digest: stable_hash_string("future-common-timestamp-support-v1"),
        minimum_future_rows: 1,
        minimum_mature_events: 1,
        maximum_requests: 1,
        maximum_concurrency: 1,
        maximum_retries: 0,
        labels_hidden_until_opening: true,
        probabilities_hidden_until_opening: true,
        one_time_opening_required: true,
        winner_selection_forbidden_before_opening: true,
        active_promotion_forbidden: true,
        reward_application_forbidden: true,
        status: CandidateEvaluationRegistrationStatusV1::Registered,
        registration_digest: String::new(),
    };
    registration.registration_digest = evaluation_registration_digest_v1(&registration);
    if validate_evaluation_registration_v1(&registration).is_err() {
        return blocked(
            CandidateEvaluationRegistrationStatusV1::FamilyInvalid,
            "v1_registration_invalid",
        );
    }
    let mut journal = AgentCandidateEvaluationRegistrationJournalV1 {
        journal_version: EVALUATION_JOURNAL_VERSION_V1.to_string(),
        agent_id: session.agent_id.clone(),
        family_digest: family.family_digest.clone(),
        entries: vec![AgentCandidateEvaluationRegistrationJournalEntryV1 {
            registration_digest: registration.registration_digest.clone(),
            status: CandidateEvaluationRegistrationStatusV1::Registered,
        }],
        journal_digest: String::new(),
    };
    journal.journal_digest = evaluation_journal_digest_v1(&journal);
    AgentCandidateEvaluationResultV1 {
        agent_id: capability.agent_id.clone(),
        family_digest: Some(family.family_digest.clone()),
        session_digest: Some(session.session_digest.clone()),
        exclusion: Some(exclusion),
        registration: Some(registration),
        journal: Some(journal),
        status: CandidateEvaluationRegistrationStatusV1::Registered,
        sanitized_error_code: None,
    }
}

pub fn evaluation_evidence_allowed_v1(
    registration: &AgentCandidateEvaluationRegistrationV1,
    exclusion: &EvaluationEvidenceExclusionV1,
    timestamp_ms: u64,
) -> bool {
    validate_evaluation_registration_v1(registration).is_ok()
        && validate_evaluation_exclusion_v1(exclusion).is_ok()
        && registration.exclusion_digest == exclusion.exclusion_digest
        && timestamp_ms >= registration.minimum_accepted_timestamp_ms
        && !exclusion.excluded_timestamp_ms.contains(&timestamp_ms)
}

pub fn public_candidate_family_summaries_v1(
    report: &AgentCandidateFamiliesReportV1,
) -> Vec<AgentCandidateFamilyPublicSummaryV1> {
    report
        .results
        .iter()
        .map(|result| AgentCandidateFamilyPublicSummaryV1 {
            agent_id: result.agent_id.clone(),
            session_digest: result
                .session
                .as_ref()
                .map(|session| session.session_digest.clone()),
            view_digest: result
                .session
                .as_ref()
                .map(|session| session.view_digest.clone()),
            projection_digest: result
                .projection
                .as_ref()
                .map(|projection| projection.projection_digest.clone()),
            family_digest: result
                .family
                .as_ref()
                .map(|family| family.family_digest.clone()),
            participant_count: result
                .family
                .as_ref()
                .map_or(0, |family| family.participants.len()),
            historical_test_access_count: result.usage_ledger.as_ref().map_or(0, |ledger| {
                ledger.historical_test_row_reads
                    + ledger.historical_test_label_reads
                    + ledger.historical_test_inference_count
                    + ledger.historical_test_metric_count
                    + ledger.historical_test_checkpoint_selection_count
                    + usize::from(ledger.historical_test_identity_influence)
            }),
            status: result.status,
        })
        .collect()
}

pub fn public_candidate_evaluation_summaries_v1(
    report: &AgentCandidateEvaluationsReportV1,
) -> Vec<AgentCandidateEvaluationPublicSummaryV1> {
    report
        .results
        .iter()
        .map(|result| AgentCandidateEvaluationPublicSummaryV1 {
            agent_id: result.agent_id.clone(),
            session_digest: result.session_digest.clone(),
            family_digest: result.family_digest.clone(),
            participant_count: result
                .registration
                .as_ref()
                .map_or(0, |registration| registration.participant_digests.len()),
            historical_test_access_count: 0,
            minimum_accepted_timestamp_ms: result
                .registration
                .as_ref()
                .map(|registration| registration.minimum_accepted_timestamp_ms),
            exclusion_digest: result
                .exclusion
                .as_ref()
                .map(|exclusion| exclusion.exclusion_digest.clone()),
            registration_status: result.status,
        })
        .collect()
}

fn run_one_candidate_family_v1(
    input: Option<&AgentPrivateLearningInputV1>,
    capability: &AgentTrainerCapabilityV0,
    mode: AgentPrivateLearningRunModeV0,
) -> AgentCandidateFamilyResultV1 {
    if !capability.supports_training
        || capability.trainer_kind == AgentTrainerKindV0::ValueQualityUnavailable
    {
        return unavailable_candidate_family_result_v1(
            &capability.agent_id,
            AgentLearningSessionStatusV1::TrainerUnavailable,
            "trainer_unavailable",
        );
    }
    let Some(input) = input else {
        return unavailable_candidate_family_result_v1(
            &capability.agent_id,
            AgentLearningSessionStatusV1::InsufficientEvidence,
            "complete_persisted_view_unavailable",
        );
    };
    if !input.persisted_view_verified {
        return unavailable_candidate_family_result_v1(
            &capability.agent_id,
            AgentLearningSessionStatusV1::InsufficientEvidence,
            "persisted_view_verification_failed",
        );
    }
    if !matches!(
        input.input.resolution_status,
        AgentViewResolutionStatusV0::Complete
            | AgentViewResolutionStatusV0::OptionalEvidenceUnavailable
    ) || input.input.view.decision_gate != EvidenceDecisionGate::Ready
    {
        return unavailable_candidate_family_result_v1(
            &capability.agent_id,
            AgentLearningSessionStatusV1::InsufficientEvidence,
            "complete_view_required",
        );
    }
    let projection = match build_trainer_projection_v1(&input.input, capability.trainer_kind) {
        Ok(projection) => projection,
        Err(status) => {
            return unavailable_candidate_family_result_v1(
                &capability.agent_id,
                status,
                "trainer_projection_rejected",
            );
        }
    };
    let mut session = registered_session_v1(&input.input, capability, &projection);
    if mode == AgentPrivateLearningRunModeV0::Status {
        session.status = AgentLearningSessionStatusV1::ProjectionReady;
        session.session_digest = session_digest_v1(&session);
        return AgentCandidateFamilyResultV1 {
            agent_id: capability.agent_id.clone(),
            session: Some(session),
            projection: Some(projection),
            family: None,
            qualification_receipts: vec![],
            usage_ledger: None,
            status: AgentLearningSessionStatusV1::ProjectionReady,
            sanitized_error_code: None,
        };
    }
    let snapshot = projection
        .primary_series_digest
        .as_ref()
        .and_then(|digest| {
            input
                .input
                .artifacts
                .iter()
                .find(|artifact| artifact.artifact_ref.artifact_digest == *digest)
        })
        .map(|artifact| &artifact.snapshot);
    let Some(snapshot) = snapshot else {
        return unavailable_candidate_family_result_v1(
            &capability.agent_id,
            AgentLearningSessionStatusV1::InsufficientEvidence,
            "projected_source_unavailable",
        );
    };
    let execution = match capability.trainer_kind {
        AgentTrainerKindV0::MomentumFrozenMambaHead => {
            run_momentum_validation_only_v1(&input.input, snapshot)
        }
        AgentTrainerKindV0::CycleRiskIndependentShadow => {
            run_cycle_risk_validation_only_v1(snapshot, &CycleRiskShadowConfigV0::default())
                .map(validation_execution_from_cycle_v1)
                .map_err(|_| AgentLearningSessionStatusV1::ValidationBlocked)
        }
        AgentTrainerKindV0::ValueQualityUnavailable => {
            Err(AgentLearningSessionStatusV1::TrainerUnavailable)
        }
    };
    let execution = match execution {
        Ok(execution) => execution,
        Err(status) => {
            session.status = status;
            session.session_digest = session_digest_v1(&session);
            return AgentCandidateFamilyResultV1 {
                agent_id: capability.agent_id.clone(),
                session: Some(session),
                projection: Some(projection),
                family: None,
                qualification_receipts: vec![],
                usage_ledger: None,
                status,
                sanitized_error_code: Some("validation_only_training_blocked".to_string()),
            };
        }
    };
    if execution.validation_parameter_updates != 0
        || execution.historical_test_row_reads != 0
        || execution.historical_test_label_reads != 0
        || execution.historical_test_inference_count != 0
        || execution.historical_test_metric_count != 0
        || execution.historical_test_checkpoint_selection_count != 0
    {
        session.status = AgentLearningSessionStatusV1::RejectedSafetyInvariant;
        session.session_digest = session_digest_v1(&session);
        return AgentCandidateFamilyResultV1 {
            agent_id: capability.agent_id.clone(),
            session: Some(session),
            projection: Some(projection),
            family: None,
            qualification_receipts: vec![],
            usage_ledger: None,
            status: AgentLearningSessionStatusV1::RejectedSafetyInvariant,
            sanitized_error_code: Some("validation_only_safety_invariant".to_string()),
        };
    }

    session.status = AgentLearningSessionStatusV1::CandidateFamilyFrozen;
    session.session_digest = session_digest_v1(&session);
    let validation_range_digest = stable_hash_string(&format!(
        "validation-only-range-v1:{}:{:?}",
        session.session_digest, execution.validation_range
    ));
    let metric_policy_digest = stable_hash_string(
        "validation-only-qualification-metric-v1:brier:finite:role-aware-collapse",
    );
    let mut participants = Vec::new();
    let mut receipts = Vec::new();
    for built in execution.participants {
        let participant_id = format!(
            "{}-{}",
            session.agent_id,
            stable_hash_string(&format!(
                "participant-v1:{}:{}:{}",
                session.session_digest, built.model_kind, built.parameter_digest
            ))
        );
        let mut participant = FrozenCandidateParticipantV1 {
            participant_id,
            role: built.role,
            model_kind: built.model_kind,
            model_artifact_digest: built.model_artifact_digest,
            parameter_digest: built.parameter_digest,
            normalizer_digest: built.normalizer_digest,
            feature_policy_digest: session.feature_policy_digest.clone(),
            label_policy_digest: session.label_policy_digest.clone(),
            training_policy_digest: built.training_policy_digest,
            initialization_digest: built.initialization_digest,
            deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
            participant_digest: String::new(),
        };
        participant.participant_digest = participant_digest_v1(&participant);
        let mut receipt = ParticipantValidationQualificationV1 {
            participant_digest: participant.participant_digest.clone(),
            validation_range_digest: validation_range_digest.clone(),
            metric_policy_digest: metric_policy_digest.clone(),
            private_metric_digest: built.private_validation_metric_digest,
            qualification_status: built.qualification_status,
            parameter_updates_during_validation: 0,
            receipt_digest: String::new(),
        };
        receipt.receipt_digest = qualification_receipt_digest_v1(&receipt);
        participants.push(participant);
        receipts.push(receipt);
    }
    participants.sort_by(|left, right| left.participant_id.cmp(&right.participant_id));
    receipts.sort_by(|left, right| left.participant_digest.cmp(&right.participant_digest));
    let included_participant_digests = participants
        .iter()
        .map(|participant| participant.participant_digest.as_str())
        .collect::<BTreeSet<_>>();
    let mut qualification_receipt_digests = receipts
        .iter()
        .filter(|receipt| {
            included_participant_digests.contains(receipt.participant_digest.as_str())
        })
        .map(|receipt| receipt.receipt_digest.clone())
        .collect::<Vec<_>>();
    qualification_receipt_digests.sort();
    qualification_receipt_digests.dedup();
    let mut family = AgentCandidateFamilyV1 {
        family_version: FAMILY_VERSION_V1.to_string(),
        agent_id: session.agent_id.clone(),
        session_digest: session.session_digest.clone(),
        view_digest: session.view_digest.clone(),
        projection_digest: session.projection_digest.clone(),
        participants,
        validation_qualification_receipts: qualification_receipt_digests,
        winner_selected: false,
        historical_test_accessed: false,
        eligible_for_active_committee: false,
        eligible_for_promotion: false,
        eligible_for_reward: false,
        family_digest: String::new(),
    };
    family.family_digest = candidate_family_digest_v1(&family);
    let usage_ledger = candidate_usage_ledger_v1(
        &session,
        &projection,
        &family,
        &execution.training_range,
        &execution.purge_range,
        &execution.validation_range,
        &execution.reserved_retrospective_unused_range,
    );
    let status = if family.participants.len() >= 2 {
        AgentLearningSessionStatusV1::CandidateFamilyFrozen
    } else {
        AgentLearningSessionStatusV1::ValidationBlocked
    };
    AgentCandidateFamilyResultV1 {
        agent_id: capability.agent_id.clone(),
        session: Some(session),
        projection: Some(projection),
        family: Some(family),
        qualification_receipts: receipts,
        usage_ledger: Some(usage_ledger),
        status,
        sanitized_error_code: (status != AgentLearningSessionStatusV1::CandidateFamilyFrozen)
            .then(|| "validation_qualification_blocked".to_string()),
    }
}

fn unavailable_candidate_family_result_v1(
    agent_id: &str,
    status: AgentLearningSessionStatusV1,
    error_code: &str,
) -> AgentCandidateFamilyResultV1 {
    AgentCandidateFamilyResultV1 {
        agent_id: agent_id.to_string(),
        session: None,
        projection: None,
        family: None,
        qualification_receipts: vec![],
        usage_ledger: None,
        status,
        sanitized_error_code: Some(error_code.to_string()),
    }
}

fn build_trainer_projection_v1(
    input: &AgentPrivateLearningSessionInputV0,
    trainer_kind: AgentTrainerKindV0,
) -> Result<AgentTrainerInputProjectionV1, AgentLearningSessionStatusV1> {
    let preliminary = build_trainer_projection_v0(input, trainer_kind)
        .map_err(|_| AgentLearningSessionStatusV1::InsufficientEvidence)?;
    let mut projection = AgentTrainerInputProjectionV1 {
        projection_version: PROJECTION_VERSION_V1.to_string(),
        agent_id: preliminary.agent_id,
        trainer_kind: preliminary.trainer_kind,
        source_view_digest: preliminary.source_view_digest,
        consumed_artifact_digests: preliminary.consumed_artifact_digests,
        referenced_unconsumed_artifact_digests: preliminary
            .referenced_but_unconsumed_artifact_digests,
        primary_series_digest: preliminary.primary_series_digest,
        projection_policy_digest: stable_hash_string(&format!(
            "validation-only-projection-v1:{}",
            preliminary.projection_policy_digest
        )),
        projection_digest: String::new(),
    };
    projection.projection_digest = projection_digest_v1(&projection);
    validate_projection_v1(&projection)
        .map_err(|_| AgentLearningSessionStatusV1::RejectedSafetyInvariant)?;
    Ok(projection)
}

fn registered_session_v1(
    input: &AgentPrivateLearningSessionInputV0,
    capability: &AgentTrainerCapabilityV0,
    projection: &AgentTrainerInputProjectionV1,
) -> AgentPrivateLearningSessionV1 {
    let session_id = format!(
        "v1-family-session-{}",
        stable_hash_string(&format!(
            "{}:{}:{}:{}",
            input.intent.intent_digest,
            input.view.view_digest,
            projection.projection_digest,
            capability.capability_digest
        ))
    );
    let mut session = AgentPrivateLearningSessionV1 {
        session_version: SESSION_VERSION_V1_FAMILY.to_string(),
        session_id,
        agent_id: input.intent.agent_id.clone(),
        agent_kind: input.intent.agent_kind,
        intent_digest: input.intent.intent_digest.clone(),
        view_digest: input.view.view_digest.clone(),
        projection_digest: projection.projection_digest.clone(),
        capability_digest: capability.capability_digest.clone(),
        source_policy_digest: input.intent.source_policy_digest.clone(),
        feature_policy_digest: input.view.feature_policy_digest.clone(),
        label_policy_digest: input.view.label_policy_digest.clone(),
        curriculum_policy_digest: input.view.curriculum_policy_digest.clone(),
        information_cutoff_ms: input.view.information_cutoff_ms,
        source_artifact_digests: input.view.source_artifact_digests.clone(),
        consumed_artifact_digests: projection.consumed_artifact_digests.clone(),
        referenced_unconsumed_artifact_digests: projection
            .referenced_unconsumed_artifact_digests
            .clone(),
        private_namespace_digest: input.view.private_namespace_digest.clone(),
        training_ledger_digest: input.view.training_ledger_digest.clone(),
        fresh_initialization: true,
        historical_test_access_forbidden: true,
        status: AgentLearningSessionStatusV1::PersistedViewVerified,
        session_digest: String::new(),
    };
    session.session_digest = session_digest_v1(&session);
    session
}

fn run_momentum_validation_only_v1(
    input: &AgentPrivateLearningSessionInputV0,
    snapshot: &DataSnapshot,
) -> Result<ValidationOnlyExecutionV1, AgentLearningSessionStatusV1> {
    let mut config = MomentumLearningCampaignConfigV0::default();
    config.agent_id = input.intent.agent_id.clone();
    config.campaign_id = format!(
        "validation-only-v1-{}",
        stable_hash_string(&format!(
            "{}:{}",
            input.intent.intent_digest, input.view.view_digest
        ))
    );
    config.initialization_policy = super::HeadInitializationPolicyV0::ColdStartEachWindow;
    config
        .validate()
        .map_err(|_| AgentLearningSessionStatusV1::RejectedSafetyInvariant)?;
    let gap = config
        .required_purge_gap()
        .map_err(|_| AgentLearningSessionStatusV1::RejectedSafetyInvariant)?;
    let validation_start = config
        .train_rows
        .checked_add(gap)
        .ok_or(AgentLearningSessionStatusV1::RejectedSafetyInvariant)?;
    let validation_end = validation_start
        .checked_add(config.validation_rows)
        .ok_or(AgentLearningSessionStatusV1::RejectedSafetyInvariant)?;
    let rows = &snapshot.normalized_dataset.rows;
    if validation_end >= rows.len()
        || rows[..validation_end]
            .windows(2)
            .any(|pair| pair[0].timestamp_ms >= pair[1].timestamp_ms)
        || historical_replay_dataset_digest_v0(&snapshot.normalized_dataset)
            != snapshot.content_digest
    {
        return Err(AgentLearningSessionStatusV1::InsufficientEvidence);
    }
    let candles = rows[..validation_end]
        .iter()
        .map(|row| {
            Ok(MomentumCandleV0 {
                timestamp: i64::try_from(row.timestamp_ms)
                    .map_err(|_| AgentLearningSessionStatusV1::RejectedSafetyInvariant)?,
                open: row.open as f32,
                high: row.high as f32,
                low: row.low as f32,
                close: row.close as f32,
                volume: row.volume as f32,
            })
        })
        .collect::<Result<Vec<_>, AgentLearningSessionStatusV1>>()?;
    let raw_features = build_momentum_features_v0(&candles, &config.feature_config)
        .map_err(|_| AgentLearningSessionStatusV1::ValidationBlocked)?;
    let training_features = raw_features
        .iter()
        .filter(|row| row.source_index < config.train_rows)
        .cloned()
        .collect::<Vec<_>>();
    if training_features.is_empty() {
        return Err(AgentLearningSessionStatusV1::InsufficientEvidence);
    }
    let normalizer = FeatureNormalizerV0::fit(&training_features)
        .map_err(|_| AgentLearningSessionStatusV1::ValidationBlocked)?;
    let normalized = normalizer
        .transform(&raw_features)
        .map_err(|_| AgentLearningSessionStatusV1::ValidationBlocked)?;
    let all_examples = build_momentum_sequence_examples_v0(
        &candles,
        &normalized,
        &config.sequence_config,
        std::slice::from_ref(&snapshot.snapshot_id),
    )
    .map_err(|_| AgentLearningSessionStatusV1::ValidationBlocked)?;
    let training_examples = examples_for_partition_v1(
        &all_examples,
        &IndexRangeV0 {
            start: 0,
            end: config.train_rows,
        },
    );
    let validation_examples = examples_for_partition_v1(
        &all_examples,
        &IndexRangeV0 {
            start: validation_start,
            end: validation_end,
        },
    );
    if training_examples.is_empty()
        || validation_examples.len() < config.validation_signal_gate.minimum_samples
        || training_examples
            .last()
            .is_some_and(|row| row.label_index >= validation_start)
        || validation_examples
            .iter()
            .any(|row| row.label_index >= validation_end)
    {
        return Err(AgentLearningSessionStatusV1::ValidationBlocked);
    }

    let encoder = frozen_mamba3_encoder_from_seed_v0(
        &config.feature_config,
        config.campaign_seed,
        config.backend_preference,
        config.fallback_policy,
    )
    .map_err(|_| AgentLearningSessionStatusV1::TechnicalFailure)?;
    let encoder_digest = encoder.parameter_digest();
    let representation_dimension = encoder
        .encode_sequence(&training_examples[0].input)
        .map_err(|_| AgentLearningSessionStatusV1::TechnicalFailure)?
        .representation
        .len();
    let initial_head = LogisticPredictionHeadV0::seeded(
        representation_dimension,
        config.campaign_seed ^ 0x74A1_0001,
    )
    .map_err(|_| AgentLearningSessionStatusV1::TechnicalFailure)?;
    let initial_head_digest = initial_head.parameter_digest();
    let trained = train_frozen_mamba_head_v0(
        &encoder,
        initial_head,
        &training_examples,
        &validation_examples,
        &config.training_config,
    )
    .map_err(|_| AgentLearningSessionStatusV1::ValidationBlocked)?;
    if trained.encoder_digest_before != encoder_digest
        || trained.encoder_digest_after != encoder_digest
    {
        return Err(AgentLearningSessionStatusV1::RejectedSafetyInvariant);
    }
    let encoded_validation = encoder
        .encode_batch(&validation_examples)
        .map_err(|_| AgentLearningSessionStatusV1::TechnicalFailure)?;
    let mamba_metric = evaluate_head_v0(&trained.final_head, &encoded_validation)
        .map_err(|_| AgentLearningSessionStatusV1::ValidationBlocked)?;
    let mamba_probabilities = encoded_validation
        .iter()
        .map(|example| trained.final_head.probability(&example.representation))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| AgentLearningSessionStatusV1::ValidationBlocked)?;

    let mut linear_config = config.training_config.clone();
    linear_config.seed = config.campaign_seed ^ 0x74A1_0002;
    let linear =
        LinearMomentumBaselineV0::train(&training_examples, &validation_examples, &linear_config)
            .map_err(|_| AgentLearningSessionStatusV1::ValidationBlocked)?;
    let linear_metric = linear
        .evaluate(&validation_examples)
        .map_err(|_| AgentLearningSessionStatusV1::ValidationBlocked)?;
    let linear_probabilities = validation_examples
        .iter()
        .map(|example| {
            example
                .input
                .last()
                .ok_or(AgentLearningSessionStatusV1::ValidationBlocked)
                .and_then(|row| {
                    linear
                        .head
                        .probability(row)
                        .map_err(|_| AgentLearningSessionStatusV1::ValidationBlocked)
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let constant = ConstantProbabilityBaselineV0::fit(&training_examples)
        .map_err(|_| AgentLearningSessionStatusV1::ValidationBlocked)?;
    let constant_metric = constant
        .evaluate(&validation_examples)
        .map_err(|_| AgentLearningSessionStatusV1::ValidationBlocked)?;
    let constant_parameter_digest = stable_hash_string(&format!(
        "momentum-training-prevalence-constant-v1:{:08x}",
        constant.probability.to_bits()
    ));
    let training_policy_digest = config.digest();
    let normalizer_digest = normalizer.digest();
    let mut participants = vec![
        ValidationParticipantBuildV1 {
            model_kind: "FrozenMambaHeadV1".to_string(),
            role: CandidateParticipantRoleV1::ModelCandidate,
            model_artifact_digest: stable_hash_string(&format!(
                "momentum-frozen-mamba-head-v1:{}:{}:{}:{}",
                encoder_digest,
                trained.final_head.parameter_digest(),
                normalizer_digest,
                training_policy_digest
            )),
            parameter_digest: trained.final_head.parameter_digest(),
            normalizer_digest: normalizer_digest.clone(),
            training_policy_digest: training_policy_digest.clone(),
            initialization_digest: stable_hash_string(&format!(
                "momentum-fresh-mamba-initialization-v1:{}:{}",
                config.campaign_seed ^ 0x74A1_0001,
                initial_head_digest
            )),
            private_validation_metric_digest: private_validation_metric_digest_v1(
                "FrozenMambaHeadV1",
                &mamba_metric,
            ),
            qualification_status: momentum_qualification_status_v1(
                &mamba_metric,
                &mamba_probabilities,
                config.validation_signal_gate.minimum_samples,
                config.validation_signal_gate.minimum_probability_stddev,
                false,
            ),
        },
        ValidationParticipantBuildV1 {
            model_kind: "LinearMomentumBaselineV1".to_string(),
            role: CandidateParticipantRoleV1::LinearComparator,
            model_artifact_digest: stable_hash_string(&format!(
                "momentum-linear-baseline-v1:{}:{}:{}",
                linear.head.parameter_digest(),
                normalizer_digest,
                training_policy_digest
            )),
            parameter_digest: linear.head.parameter_digest(),
            normalizer_digest: normalizer_digest.clone(),
            training_policy_digest: training_policy_digest.clone(),
            initialization_digest: stable_hash_string(&format!(
                "momentum-fresh-linear-initialization-v1:{}",
                linear_config.seed
            )),
            private_validation_metric_digest: private_validation_metric_digest_v1(
                "LinearMomentumBaselineV1",
                &linear_metric,
            ),
            qualification_status: momentum_qualification_status_v1(
                &linear_metric,
                &linear_probabilities,
                config.validation_signal_gate.minimum_samples,
                config.validation_signal_gate.minimum_probability_stddev,
                false,
            ),
        },
        ValidationParticipantBuildV1 {
            model_kind: "ConstantProbabilityBaselineV1".to_string(),
            role: CandidateParticipantRoleV1::ConstantComparator,
            model_artifact_digest: stable_hash_string(&format!(
                "momentum-constant-baseline-v1:{}:{}:{}",
                constant_parameter_digest, normalizer_digest, training_policy_digest
            )),
            parameter_digest: constant_parameter_digest,
            normalizer_digest,
            training_policy_digest,
            initialization_digest: stable_hash_string(&format!(
                "momentum-fresh-training-prevalence-v1:{}",
                training_examples.len()
            )),
            private_validation_metric_digest: private_validation_metric_digest_v1(
                "ConstantProbabilityBaselineV1",
                &constant_metric,
            ),
            qualification_status: momentum_qualification_status_v1(
                &constant_metric,
                &vec![constant.probability; validation_examples.len()],
                config.validation_signal_gate.minimum_samples,
                config.validation_signal_gate.minimum_probability_stddev,
                true,
            ),
        },
    ];
    participants.sort_by(|left, right| left.model_kind.cmp(&right.model_kind));
    Ok(ValidationOnlyExecutionV1 {
        training_range: IndexRangeV0 {
            start: 0,
            end: config.train_rows,
        },
        purge_range: IndexRangeV0 {
            start: config.train_rows,
            end: validation_start,
        },
        validation_range: IndexRangeV0 {
            start: validation_start,
            end: validation_end,
        },
        reserved_retrospective_unused_range: IndexRangeV0 {
            start: validation_end,
            end: rows.len(),
        },
        participants,
        validation_parameter_updates: 0,
        historical_test_row_reads: 0,
        historical_test_label_reads: 0,
        historical_test_inference_count: 0,
        historical_test_metric_count: 0,
        historical_test_checkpoint_selection_count: 0,
    })
}

fn examples_for_partition_v1(
    examples: &[SequenceExampleV0],
    range: &IndexRangeV0,
) -> Vec<SequenceExampleV0> {
    examples
        .iter()
        .filter(|example| example.sequence_start >= range.start && example.label_index < range.end)
        .cloned()
        .collect()
}

fn private_validation_metric_digest_v1(model_kind: &str, metric: &EvaluationMetricsV0) -> String {
    stable_hash_string(&format!(
        "private-validation-metric-v1:{model_kind}:{metric:?}"
    ))
}

fn momentum_qualification_status_v1(
    metric: &EvaluationMetricsV0,
    probabilities: &[f32],
    minimum_samples: usize,
    minimum_probability_stddev: f32,
    allow_constant: bool,
) -> ValidationQualificationStatusV1 {
    if metric.sample_count < minimum_samples || probabilities.len() != metric.sample_count {
        return ValidationQualificationStatusV1::RejectedInsufficientValidation;
    }
    if !metric.brier_score.is_finite()
        || !metric.accuracy.is_finite()
        || !metric.mean_predicted_probability.is_finite()
        || probabilities.iter().any(|value| !value.is_finite())
    {
        return ValidationQualificationStatusV1::RejectedNumericalFailure;
    }
    let mean = probabilities.iter().sum::<f32>() / probabilities.len() as f32;
    let stddev = (probabilities
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f32>()
        / probabilities.len() as f32)
        .sqrt();
    if !allow_constant && stddev < minimum_probability_stddev {
        ValidationQualificationStatusV1::RejectedProbabilityCollapse
    } else {
        ValidationQualificationStatusV1::Qualified
    }
}

fn validation_execution_from_cycle_v1(
    execution: CycleRiskValidationOnlyExecutionV1,
) -> ValidationOnlyExecutionV1 {
    let participants = execution
        .participants
        .into_iter()
        .map(|participant| ValidationParticipantBuildV1 {
            role: match participant.model_kind.as_str() {
                "FrozenMambaRiskV1" => CandidateParticipantRoleV1::ModelCandidate,
                "LinearRiskV1" => CandidateParticipantRoleV1::LinearComparator,
                _ => CandidateParticipantRoleV1::ConstantComparator,
            },
            model_kind: participant.model_kind,
            model_artifact_digest: participant.model_artifact_digest,
            parameter_digest: participant.parameter_digest,
            normalizer_digest: participant.normalizer_digest,
            training_policy_digest: participant.training_policy_digest,
            initialization_digest: participant.initialization_digest,
            private_validation_metric_digest: participant.private_validation_metric_digest,
            qualification_status: if participant.qualified {
                ValidationQualificationStatusV1::Qualified
            } else {
                ValidationQualificationStatusV1::RejectedProbabilityCollapse
            },
        })
        .collect();
    ValidationOnlyExecutionV1 {
        training_range: execution.training_range,
        purge_range: execution.purge_range,
        validation_range: execution.validation_range,
        reserved_retrospective_unused_range: execution.reserved_retrospective_unused_range,
        participants,
        validation_parameter_updates: execution.validation_parameter_updates,
        historical_test_row_reads: execution.historical_test_row_reads,
        historical_test_label_reads: execution.historical_test_label_reads,
        historical_test_inference_count: execution.historical_test_inference_count,
        historical_test_metric_count: execution.historical_test_metric_count,
        historical_test_checkpoint_selection_count: execution
            .historical_test_checkpoint_selection_count,
    }
}

fn run_one_session_v0(
    input: &AgentPrivateLearningSessionInputV0,
    registry: &AgentTrainerCapabilityRegistryV0,
    mode: AgentPrivateLearningRunModeV0,
) -> AgentPrivateLearningSessionResultV0 {
    let capability = registry
        .capabilities
        .iter()
        .find(|capability| capability.agent_id == input.intent.agent_id);
    let Some(capability) = capability else {
        return technical_result_v0(input, AgentTrainerKindV0::ValueQualityUnavailable);
    };
    let mut session = registered_session_v0(input, capability);
    let mut dataset_manifest = None;
    let mut candidate = None;
    let mut trainer_projection = None;
    let mut sanitized_error_code = None;

    if !capability.supports_training {
        session.session_status = AgentLearningSessionStatusV0::TrainerUnavailable;
    } else if mode == AgentPrivateLearningRunModeV0::Status {
        session.session_status = AgentLearningSessionStatusV0::Registered;
    } else if !matches!(
        input.resolution_status,
        AgentViewResolutionStatusV0::Complete
            | AgentViewResolutionStatusV0::OptionalEvidenceUnavailable
    ) {
        session.session_status = resolution_session_status_v0(input.resolution_status);
        sanitized_error_code = Some(resolution_error_code_v0(input.resolution_status).to_string());
    } else {
        match build_trainer_projection_v0(input, capability.trainer_kind).and_then(|projection| {
            materialize_private_dataset_v0(
                input,
                capability.trainer_kind,
                &session.session_id,
                &projection,
            )
            .map(|materialized| (projection, materialized))
        }) {
            Ok((projection, materialized)) => {
                session.trainer_projection_digest = Some(projection.projection_digest.clone());
                trainer_projection = Some(projection);
                dataset_manifest = Some(materialized.manifest);
                session.session_status = AgentLearningSessionStatusV0::DatasetReady;
                if mode == AgentPrivateLearningRunModeV0::ExecuteLocal {
                    let training_result = dataset_manifest
                        .as_mut()
                        .ok_or(AgentLearningSessionStatusV0::TechnicalFailure)
                        .and_then(|manifest| match capability.trainer_kind {
                            AgentTrainerKindV0::MomentumFrozenMambaHead => run_momentum_adapter_v0(
                                input,
                                &mut session,
                                manifest,
                                &materialized.snapshot,
                            )
                            .map(|value| candidate = value),
                            AgentTrainerKindV0::CycleRiskIndependentShadow => {
                                run_cycle_risk_adapter_v0(
                                    input,
                                    &mut session,
                                    manifest,
                                    &materialized.snapshot,
                                )
                                .map(|value| candidate = value)
                            }
                            AgentTrainerKindV0::ValueQualityUnavailable => Ok(()),
                        });
                    training_result.unwrap_or_else(|status| {
                        session.session_status = status;
                        sanitized_error_code = Some(session_status_code_v0(status).to_string());
                    });
                }
            }
            Err(error) => {
                session.session_status = evidence_error_status_v0(error);
                sanitized_error_code = Some(evidence_error_code_v0(error).to_string());
            }
        }
    }
    session.session_digest = session_digest_v0(&session);
    if let Some(value) = &mut candidate {
        value.session_digest = session.session_digest.clone();
        value.candidate_digest = candidate_digest_v0(value);
    }
    if candidate
        .as_ref()
        .is_some_and(|value| validate_candidate_v0(value).is_err())
    {
        candidate = None;
        session.session_status = AgentLearningSessionStatusV0::RejectedSafetyInvariant;
        session.session_digest = session_digest_v0(&session);
        sanitized_error_code = Some("candidate_safety_invariant".to_string());
    }
    if let Some(manifest) = &mut dataset_manifest {
        manifest.manifest_digest = dataset_manifest_digest_v0(manifest);
    }
    let journal = journal_v0(&session, dataset_manifest.as_ref(), candidate.as_ref());
    AgentPrivateLearningSessionResultV0 {
        trainer_kind: capability.trainer_kind,
        view_resolution_status: input.resolution_status,
        source_count: session.source_artifact_digests.len(),
        session,
        trainer_projection,
        dataset_manifest,
        candidate,
        journal,
        sanitized_error_code,
    }
}

fn technical_result_v0(
    input: &AgentPrivateLearningSessionInputV0,
    trainer_kind: AgentTrainerKindV0,
) -> AgentPrivateLearningSessionResultV0 {
    let capability = capability(&input.intent.agent_id, trainer_kind, vec![], false);
    let mut session = registered_session_v0(input, &capability);
    session.session_status = AgentLearningSessionStatusV0::TechnicalFailure;
    session.session_digest = session_digest_v0(&session);
    let journal = journal_v0(&session, None, None);
    AgentPrivateLearningSessionResultV0 {
        trainer_kind,
        view_resolution_status: input.resolution_status,
        source_count: input.view.source_artifact_digests.len(),
        session,
        trainer_projection: None,
        dataset_manifest: None,
        candidate: None,
        journal,
        sanitized_error_code: Some("trainer_registry_missing".to_string()),
    }
}

fn registered_session_v0(
    input: &AgentPrivateLearningSessionInputV0,
    capability: &AgentTrainerCapabilityV0,
) -> AgentPrivateLearningSessionV0 {
    let session_id = format!(
        "session-{}",
        stable_hash_string(&format!(
            "{}:{}:{}:{}",
            input.intent.agent_id,
            input.intent.intent_digest,
            input.view.view_digest,
            capability.capability_digest
        ))
    );
    let mut session = AgentPrivateLearningSessionV0 {
        session_version: SESSION_VERSION_V1.to_string(),
        session_id,
        agent_id: input.intent.agent_id.clone(),
        agent_kind: input.intent.agent_kind,
        intent_digest: input.intent.intent_digest.clone(),
        data_view_digest: input.view.view_digest.clone(),
        trainer_capability_digest: capability.capability_digest.clone(),
        information_cutoff_ms: input.view.information_cutoff_ms,
        required_dataset_kinds: input.intent.required_datasets.clone(),
        optional_dataset_kinds: input.intent.optional_datasets.clone(),
        allowed_markets: input.policy.allowed_markets.clone(),
        symbols: input.intent.symbols.clone(),
        cadence: input.intent.cadence.clone(),
        lookback: input.intent.lookback.clone(),
        maximum_staleness_ms: input.intent.maximum_staleness_ms,
        source_artifact_digests: input.view.source_artifact_digests.clone(),
        source_policy_digest: input.intent.source_policy_digest.clone(),
        feature_policy_digest: input.view.feature_policy_digest.clone(),
        label_policy_digest: input.view.label_policy_digest.clone(),
        curriculum_policy_digest: input.view.curriculum_policy_digest.clone(),
        private_namespace_digest: input.view.private_namespace_digest.clone(),
        training_ledger_digest: input.view.training_ledger_digest.clone(),
        trainer_projection_digest: None,
        parent_model_version: None,
        session_status: AgentLearningSessionStatusV0::Registered,
        session_digest: String::new(),
    };
    session.session_digest = session_digest_v0(&session);
    session
}

fn build_trainer_projection_v0(
    input: &AgentPrivateLearningSessionInputV0,
    trainer_kind: AgentTrainerKindV0,
) -> Result<AgentTrainerInputProjectionV0, EvidenceResolutionErrorV0> {
    if input.view.view_digest.is_empty()
        || input.view.source_artifact_digests.len() != input.artifacts.len()
        || input.artifacts.iter().any(|artifact| {
            !input
                .view
                .source_artifact_digests
                .contains(&artifact.artifact_ref.artifact_digest)
        })
    {
        return Err(EvidenceResolutionErrorV0::SourceDigest);
    }
    let primary_kinds: &[DatasetKind] = match trainer_kind {
        AgentTrainerKindV0::MomentumFrozenMambaHead => &[
            DatasetKind::DailyOhlcv,
            DatasetKind::AdjustedDailyOhlcv,
            DatasetKind::CryptoDailyOhlcv,
        ],
        AgentTrainerKindV0::CycleRiskIndependentShadow => &[DatasetKind::MarketIndexDaily],
        AgentTrainerKindV0::ValueQualityUnavailable => {
            return Err(EvidenceResolutionErrorV0::Insufficient);
        }
    };
    let mut eligible = input
        .artifacts
        .iter()
        .filter(|artifact| primary_kinds.contains(&artifact.snapshot.dataset_kind))
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| {
        let left_required = input
            .intent
            .required_datasets
            .contains(&left.snapshot.dataset_kind);
        let right_required = input
            .intent
            .required_datasets
            .contains(&right.snapshot.dataset_kind);
        right_required
            .cmp(&left_required)
            .then_with(|| {
                primary_kinds
                    .iter()
                    .position(|kind| *kind == left.snapshot.dataset_kind)
                    .cmp(
                        &primary_kinds
                            .iter()
                            .position(|kind| *kind == right.snapshot.dataset_kind),
                    )
            })
            .then_with(|| {
                left.artifact_ref
                    .artifact_digest
                    .cmp(&right.artifact_ref.artifact_digest)
            })
    });
    let primary = eligible
        .first()
        .ok_or(EvidenceResolutionErrorV0::Insufficient)?;
    let consumed_artifact_digests = vec![primary.artifact_ref.artifact_digest.clone()];
    let referenced_but_unconsumed_artifact_digests = input
        .view
        .source_artifact_digests
        .iter()
        .filter(|digest| !consumed_artifact_digests.contains(digest))
        .cloned()
        .collect::<Vec<_>>();
    let projection_policy_digest = stable_hash_string(&format!(
        "SOMA-AGENT-TRAINER-PROJECTION-POLICY-V0:{:?}:{:?}:single-primary:no-concatenation",
        trainer_kind, primary_kinds
    ));
    let mut projection = AgentTrainerInputProjectionV0 {
        projection_version: PROJECTION_VERSION_V0.to_string(),
        agent_id: input.intent.agent_id.clone(),
        trainer_kind,
        source_view_digest: input.view.view_digest.clone(),
        consumed_artifact_digests,
        referenced_but_unconsumed_artifact_digests,
        primary_series_digest: Some(primary.snapshot.content_digest.clone()),
        auxiliary_series_digests: vec![],
        projection_policy_digest,
        projection_digest: String::new(),
    };
    projection.projection_digest = projection_digest_v0(&projection);
    Ok(projection)
}

fn materialize_private_dataset_v0(
    input: &AgentPrivateLearningSessionInputV0,
    trainer_kind: AgentTrainerKindV0,
    session_id: &str,
    projection: &AgentTrainerInputProjectionV0,
) -> Result<MaterializedPrivateDatasetV0, EvidenceResolutionErrorV0> {
    validate_agent_learning_data_view_v0(&input.view)
        .map_err(|_| EvidenceResolutionErrorV0::UnsafeEvidence)?;
    validate_agent_learning_intent_v0(&input.intent, &input.policy)
        .map_err(|_| EvidenceResolutionErrorV0::UnsafeEvidence)?;
    if input.intent.agent_id != input.view.agent_id
        || input.intent.agent_kind != input.policy.agent_kind
        || input.intent.information_cutoff_ms != input.view.information_cutoff_ms
        || input.intent.feature_policy_digest != input.view.feature_policy_digest
        || input.intent.label_policy_digest != input.view.label_policy_digest
        || input.intent.curriculum_policy_digest != input.view.curriculum_policy_digest
        || derive_agent_private_learning_state_v0(&input.intent).private_namespace_digest
            != input.view.private_namespace_digest
        || derive_agent_private_learning_state_v0(&input.intent).training_ledger_digest
            != input.view.training_ledger_digest
        || input.view.decision_gate != EvidenceDecisionGate::Ready
        || projection.source_view_digest != input.view.view_digest
        || projection.agent_id != input.intent.agent_id
        || projection.trainer_kind != trainer_kind
        || projection.projection_digest != projection_digest_v0(projection)
    {
        return Err(EvidenceResolutionErrorV0::Insufficient);
    }
    let view_digests = input
        .view
        .source_artifact_digests
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let artifact_digests = input
        .artifacts
        .iter()
        .map(|artifact| artifact.artifact_ref.artifact_digest.clone())
        .collect::<BTreeSet<_>>();
    if view_digests.len() != input.view.source_artifact_digests.len()
        || artifact_digests.len() != input.artifacts.len()
        || view_digests != artifact_digests
    {
        return Err(EvidenceResolutionErrorV0::SourceDigest);
    }
    let mut source_digests = Vec::new();
    let mut dataset_kinds = Vec::new();
    for artifact in &input.artifacts {
        let reference = &artifact.artifact_ref;
        let snapshot = &artifact.snapshot;
        match reference.visibility {
            LearningDataVisibilityV0::SharedCanonicalRaw if reference.owner_agent_id.is_none() => {}
            LearningDataVisibilityV0::AgentAuthorizedRaw
                if reference.owner_agent_id.as_deref() == Some(input.intent.agent_id.as_str()) => {}
            LearningDataVisibilityV0::AgentPrivateDerived => {
                return Err(EvidenceResolutionErrorV0::CrossAgentArtifact);
            }
            _ => return Err(EvidenceResolutionErrorV0::CrossAgentArtifact),
        }
        if reference.dataset_kind != snapshot.dataset_kind
            || !input
                .view
                .visible_dataset_kinds
                .contains(&snapshot.dataset_kind)
            || !input.intent.market_scopes.contains(&snapshot.market_scope)
            || stabilized_strings_v0(&snapshot.symbols) != input.intent.symbols
            || snapshot.requested_lookback != input.intent.lookback
        {
            return Err(EvidenceResolutionErrorV0::UnauthorizedDataset);
        }
        let digest = historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
        if digest != snapshot.content_digest || digest != reference.artifact_digest {
            return Err(EvidenceResolutionErrorV0::SourceDigest);
        }
        if !snapshot.read_only
            || !snapshot.sanitized
            || !snapshot.quality_summary.accepted
            || snapshot.row_count != snapshot.normalized_dataset.rows.len()
            || snapshot.quality_summary.row_count != snapshot.row_count
            || !snapshot.provenance.sanitized
            || !snapshot.provenance.credential_free
            || snapshot.provenance.provider_id != snapshot.provider_id
            || snapshot.provenance.source_type == SnapshotSourceType::LocalSnapshotReplay
        {
            return Err(EvidenceResolutionErrorV0::UnsafeEvidence);
        }
        if reference.maximum_event_timestamp_ms > input.view.information_cutoff_ms
            || snapshot
                .normalized_dataset
                .rows
                .iter()
                .any(|row| row.timestamp_ms > input.view.information_cutoff_ms)
        {
            return Err(EvidenceResolutionErrorV0::CutoffLeakage);
        }
        if snapshot
            .normalized_dataset
            .rows
            .windows(2)
            .any(|pair| pair[0].timestamp_ms == pair[1].timestamp_ms)
        {
            return Err(EvidenceResolutionErrorV0::Duplicate);
        }
        if snapshot
            .normalized_dataset
            .rows
            .windows(2)
            .any(|pair| pair[0].timestamp_ms > pair[1].timestamp_ms)
        {
            return Err(EvidenceResolutionErrorV0::Chronology);
        }
        if snapshot.normalized_dataset.rows.iter().any(|row| {
            row.symbol != snapshot.normalized_dataset.symbol || !finite_valid_row_v0(row)
        }) {
            return Err(EvidenceResolutionErrorV0::NonFinite);
        }
        source_digests.push(digest);
        dataset_kinds.push(snapshot.dataset_kind);
    }
    source_digests.sort();
    dataset_kinds.sort();
    dataset_kinds.dedup();
    let primary_digest = projection
        .primary_series_digest
        .as_ref()
        .ok_or(EvidenceResolutionErrorV0::Insufficient)?;
    if projection.consumed_artifact_digests != [primary_digest.clone()]
        || projection
            .referenced_but_unconsumed_artifact_digests
            .iter()
            .any(|digest| projection.consumed_artifact_digests.contains(digest))
    {
        return Err(EvidenceResolutionErrorV0::UnsafeEvidence);
    }
    let snapshot = input
        .artifacts
        .iter()
        .find(|artifact| artifact.artifact_ref.artifact_digest == *primary_digest)
        .map(|artifact| artifact.snapshot.clone())
        .ok_or(EvidenceResolutionErrorV0::SourceDigest)?;
    let manifest = initial_dataset_manifest_v0(
        input,
        trainer_kind,
        session_id,
        &source_digests,
        &dataset_kinds,
        snapshot.normalized_dataset.rows.len(),
    )?;
    Ok(MaterializedPrivateDatasetV0 { snapshot, manifest })
}

fn initial_dataset_manifest_v0(
    input: &AgentPrivateLearningSessionInputV0,
    trainer_kind: AgentTrainerKindV0,
    session_id: &str,
    source_digests: &[String],
    dataset_kinds: &[DatasetKind],
    row_count: usize,
) -> Result<AgentPrivateDatasetManifestV0, EvidenceResolutionErrorV0> {
    let (training_range, first_purge_range, validation_range, second_purge_range, test_range) =
        match trainer_kind {
            AgentTrainerKindV0::MomentumFrozenMambaHead => {
                let config = MomentumLearningCampaignConfigV0::default();
                let windows = build_momentum_learning_windows_v0(
                    &config,
                    row_count,
                    &source_digests.to_vec(),
                )
                .map_err(|_| EvidenceResolutionErrorV0::UnsafeEvidence)?;
                let window = windows
                    .first()
                    .ok_or(EvidenceResolutionErrorV0::Insufficient)?;
                (
                    window.train_range.clone(),
                    IndexRangeV0 {
                        start: window.train_range.end,
                        end: window.validation_range.start,
                    },
                    window.validation_range.clone(),
                    IndexRangeV0 {
                        start: window.validation_range.end,
                        end: window.test_range.start,
                    },
                    window.test_range.clone(),
                )
            }
            AgentTrainerKindV0::CycleRiskIndependentShadow => {
                let config = CycleRiskShadowConfigV0::default();
                let train_end = (row_count as f32 * config.train_fraction).floor() as usize;
                let validation_end =
                    train_end + (row_count as f32 * config.validation_fraction).floor() as usize;
                let gap = config.label.purge_gap_rows + config.sequence_length;
                if train_end <= gap
                    || train_end + gap >= validation_end
                    || validation_end + gap >= row_count
                {
                    return Err(EvidenceResolutionErrorV0::Insufficient);
                }
                (
                    IndexRangeV0 {
                        start: 0,
                        end: train_end,
                    },
                    IndexRangeV0 {
                        start: train_end,
                        end: train_end + gap,
                    },
                    IndexRangeV0 {
                        start: train_end + gap,
                        end: validation_end,
                    },
                    IndexRangeV0 {
                        start: validation_end,
                        end: validation_end + gap,
                    },
                    IndexRangeV0 {
                        start: validation_end + gap,
                        end: row_count,
                    },
                )
            }
            AgentTrainerKindV0::ValueQualityUnavailable => {
                return Err(EvidenceResolutionErrorV0::Insufficient);
            }
        };
    let mut manifest = AgentPrivateDatasetManifestV0 {
        dataset_version: DATASET_VERSION_V0.to_string(),
        dataset_id: format!(
            "private-dataset-{}",
            stable_hash_string(&format!(
                "{}:{}:{:?}",
                input.intent.agent_id, input.view.view_digest, source_digests
            ))
        ),
        agent_id: input.intent.agent_id.clone(),
        session_id: session_id.to_string(),
        data_view_digest: input.view.view_digest.clone(),
        source_artifact_digests: source_digests.to_vec(),
        dataset_kinds: dataset_kinds.to_vec(),
        information_cutoff_ms: input.view.information_cutoff_ms,
        row_count,
        training_range: training_range.clone(),
        first_purge_range,
        validation_range,
        second_purge_range,
        sealed_test_range: Some(test_range),
        normalizer_fit_range: training_range,
        validation_parameter_update_count: 0,
        test_checkpoint_selection_count: 0,
        prospective_row_read_count: 0,
        prospective_label_read_count: 0,
        feature_artifact_digest: stable_hash_string(&format!(
            "private-feature:{}:{}",
            input.intent.agent_id, input.view.feature_policy_digest
        )),
        label_artifact_digest: stable_hash_string(&format!(
            "private-label:{}:{}",
            input.intent.agent_id, input.view.label_policy_digest
        )),
        normalizer_digest: String::new(),
        manifest_digest: String::new(),
    };
    validate_dataset_manifest_v0(&manifest)
        .map_err(|_| EvidenceResolutionErrorV0::UnsafeEvidence)?;
    manifest.manifest_digest = dataset_manifest_digest_v0(&manifest);
    Ok(manifest)
}

fn run_momentum_adapter_v0(
    input: &AgentPrivateLearningSessionInputV0,
    session: &mut AgentPrivateLearningSessionV0,
    manifest: &mut AgentPrivateDatasetManifestV0,
    snapshot: &DataSnapshot,
) -> Result<Option<AgentSandboxLearningCandidateV0>, AgentLearningSessionStatusV0> {
    let mut config = MomentumLearningCampaignConfigV0::default();
    config.agent_id = input.intent.agent_id.clone();
    config.campaign_id = session.session_id.clone();
    let encoder = frozen_mamba3_encoder_from_seed_v0(
        &config.feature_config,
        config.campaign_seed,
        config.backend_preference,
        config.fallback_policy,
    )
    .map_err(|_| AgentLearningSessionStatusV0::TechnicalFailure)?;
    let result =
        run_momentum_learning_campaign_v0(&config, std::slice::from_ref(snapshot), &encoder)
            .map_err(|_| AgentLearningSessionStatusV0::TechnicalFailure)?;
    if result.generated_versions.is_empty() {
        return Err(match result.status {
            MomentumLearningCampaignStatusV0::LeakageInvariantFailed => {
                AgentLearningSessionStatusV0::RejectedLabelLeakage
            }
            MomentumLearningCampaignStatusV0::RejectedForSafety => {
                AgentLearningSessionStatusV0::RejectedSafetyInvariant
            }
            MomentumLearningCampaignStatusV0::BackendUnavailable => {
                AgentLearningSessionStatusV0::TechnicalFailure
            }
            _ => AgentLearningSessionStatusV0::InsufficientEvidence,
        });
    }
    let version = result
        .generated_versions
        .last()
        .ok_or(AgentLearningSessionStatusV0::TechnicalFailure)?;
    let window = result
        .windows
        .last()
        .ok_or(AgentLearningSessionStatusV0::TechnicalFailure)?;
    manifest.training_range = version.train_range.clone();
    manifest.first_purge_range = IndexRangeV0 {
        start: version.train_range.end,
        end: version.validation_range.start,
    };
    manifest.validation_range = version.validation_range.clone();
    manifest.second_purge_range = IndexRangeV0 {
        start: version.validation_range.end,
        end: version.test_range.start,
    };
    manifest.sealed_test_range = Some(version.test_range.clone());
    manifest.normalizer_fit_range = version.train_range.clone();
    manifest.normalizer_digest = window.normalizer_digest.clone();
    validate_dataset_manifest_v0(manifest)
        .map_err(|_| AgentLearningSessionStatusV0::RejectedLabelLeakage)?;
    session.parent_model_version = version.parent_version_id.clone();
    session.session_status = AgentLearningSessionStatusV0::CandidateProduced;
    let mut candidate = AgentSandboxLearningCandidateV0 {
        candidate_version: CANDIDATE_VERSION_V0.to_string(),
        agent_id: input.intent.agent_id.clone(),
        session_digest: String::new(),
        data_view_digest: input.view.view_digest.clone(),
        parent_model_version: version.parent_version_id.clone(),
        model_artifact_digest: version.head_parameter_digest.clone(),
        feature_policy_digest: input.view.feature_policy_digest.clone(),
        label_policy_digest: input.view.label_policy_digest.clone(),
        normalizer_digest: version.normalizer_digest.clone(),
        training_policy_digest: config.digest(),
        private_metrics_digest: stable_hash_string(&format!("{:?}", version.metrics)),
        deployment_status: version.deployment_status,
        retrospective_research_only: true,
        eligible_for_active_committee: false,
        eligible_for_promotion: false,
        eligible_for_reward: false,
        candidate_digest: String::new(),
    };
    candidate.candidate_digest = candidate_digest_v0(&candidate);
    Ok(Some(candidate))
}

fn run_cycle_risk_adapter_v0(
    input: &AgentPrivateLearningSessionInputV0,
    session: &mut AgentPrivateLearningSessionV0,
    manifest: &mut AgentPrivateDatasetManifestV0,
    snapshot: &DataSnapshot,
) -> Result<Option<AgentSandboxLearningCandidateV0>, AgentLearningSessionStatusV0> {
    let config = CycleRiskShadowConfigV0::default();
    let report = run_cycle_risk_shadow_v0(snapshot, &config).map_err(|error| match error {
        CycleRiskErrorV0::InsufficientHistory => AgentLearningSessionStatusV0::InsufficientEvidence,
        CycleRiskErrorV0::Leakage => AgentLearningSessionStatusV0::RejectedLabelLeakage,
        CycleRiskErrorV0::InvalidEvidence | CycleRiskErrorV0::InvalidConfig => {
            AgentLearningSessionStatusV0::RejectedSafetyInvariant
        }
        CycleRiskErrorV0::Training => AgentLearningSessionStatusV0::TechnicalFailure,
    })?;
    let regime = report
        .regimes
        .last()
        .ok_or(AgentLearningSessionStatusV0::InsufficientEvidence)?;
    let regime_offset = snapshot.row_count / 2;
    let feature_offset = regime_offset + config.feature.drawdown_lookback;
    manifest.training_range = IndexRangeV0 {
        start: feature_offset,
        end: feature_offset + regime.train_feature_end,
    };
    manifest.first_purge_range = IndexRangeV0 {
        start: manifest.training_range.end,
        end: feature_offset + regime.validation_feature_start,
    };
    manifest.validation_range = IndexRangeV0 {
        start: feature_offset + regime.validation_feature_start,
        end: feature_offset + regime.validation_feature_end,
    };
    manifest.second_purge_range = IndexRangeV0 {
        start: manifest.validation_range.end,
        end: feature_offset + regime.test_feature_start,
    };
    manifest.sealed_test_range = Some(IndexRangeV0 {
        start: feature_offset + regime.test_feature_start,
        end: snapshot.row_count,
    });
    manifest.normalizer_fit_range = manifest.training_range.clone();
    manifest.normalizer_digest = stable_hash_string(&format!(
        "{}:{}",
        regime.normalizer_digest, regime.representation_normalizer_digest
    ));
    validate_dataset_manifest_v0(manifest)
        .map_err(|_| AgentLearningSessionStatusV0::RejectedLabelLeakage)?;
    session.session_status = AgentLearningSessionStatusV0::CandidateProduced;
    let model_ids = report
        .regimes
        .iter()
        .map(|regime| regime.candidate_model_version_id.as_str())
        .collect::<Vec<_>>();
    let normalizers = report
        .regimes
        .iter()
        .map(|regime| {
            format!(
                "{}:{}",
                regime.normalizer_digest, regime.representation_normalizer_digest
            )
        })
        .collect::<Vec<_>>();
    let mut candidate = AgentSandboxLearningCandidateV0 {
        candidate_version: CANDIDATE_VERSION_V0.to_string(),
        agent_id: input.intent.agent_id.clone(),
        session_digest: String::new(),
        data_view_digest: input.view.view_digest.clone(),
        parent_model_version: None,
        model_artifact_digest: stable_hash_string(&format!("{:?}", model_ids)),
        feature_policy_digest: input.view.feature_policy_digest.clone(),
        label_policy_digest: input.view.label_policy_digest.clone(),
        normalizer_digest: stable_hash_string(&format!("{:?}", normalizers)),
        training_policy_digest: config.digest(),
        private_metrics_digest: stable_hash_string(&format!("{:?}", report.regimes)),
        deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
        retrospective_research_only: true,
        eligible_for_active_committee: false,
        eligible_for_promotion: false,
        eligible_for_reward: false,
        candidate_digest: String::new(),
    };
    candidate.candidate_digest = candidate_digest_v0(&candidate);
    Ok(Some(candidate))
}

pub fn validate_dataset_manifest_v0(
    manifest: &AgentPrivateDatasetManifestV0,
) -> Result<(), String> {
    let test = manifest
        .sealed_test_range
        .as_ref()
        .ok_or_else(|| "sealed historical test range missing".to_string())?;
    if manifest.dataset_version != DATASET_VERSION_V0
        || manifest.agent_id.is_empty()
        || manifest.session_id.is_empty()
        || manifest.source_artifact_digests.is_empty()
        || manifest.dataset_kinds.is_empty()
        || manifest.row_count == 0
        || manifest.training_range.start >= manifest.training_range.end
        || manifest.training_range.end > manifest.first_purge_range.start
        || manifest.first_purge_range.start >= manifest.first_purge_range.end
        || manifest.first_purge_range.end > manifest.validation_range.start
        || manifest.validation_range.start >= manifest.validation_range.end
        || manifest.validation_range.end > manifest.second_purge_range.start
        || manifest.second_purge_range.start >= manifest.second_purge_range.end
        || manifest.second_purge_range.end > test.start
        || test.start >= test.end
        || test.end > manifest.row_count
        || manifest.normalizer_fit_range != manifest.training_range
        || manifest.validation_parameter_update_count != 0
        || manifest.test_checkpoint_selection_count != 0
        || manifest.prospective_row_read_count != 0
        || manifest.prospective_label_read_count != 0
    {
        return Err("private chronological split invariant rejected".to_string());
    }
    Ok(())
}

fn validate_candidate_v0(candidate: &AgentSandboxLearningCandidateV0) -> Result<(), String> {
    if candidate.candidate_version != CANDIDATE_VERSION_V0
        || candidate.deployment_status != ModelAgentDeploymentStatus::ShadowOnly
        || !candidate.retrospective_research_only
        || candidate.eligible_for_active_committee
        || candidate.eligible_for_promotion
        || candidate.eligible_for_reward
        || candidate.model_artifact_digest.is_empty()
        || candidate.normalizer_digest.is_empty()
    {
        return Err("sandbox candidate safety contract rejected".to_string());
    }
    Ok(())
}

fn finite_valid_row_v0(row: &HistoricalOhlcvRow) -> bool {
    row.open.is_finite()
        && row.high.is_finite()
        && row.low.is_finite()
        && row.close.is_finite()
        && row.volume.is_finite()
        && row.open > 0.0
        && row.high >= row.open.max(row.close)
        && row.low <= row.open.min(row.close)
        && row.low > 0.0
        && row.volume >= 0.0
        && row
            .trade_value
            .is_none_or(|value| value.is_finite() && value >= 0.0)
}

fn evidence_error_status_v0(error: EvidenceResolutionErrorV0) -> AgentLearningSessionStatusV0 {
    match error {
        EvidenceResolutionErrorV0::SourceDigest
        | EvidenceResolutionErrorV0::UnauthorizedDataset
        | EvidenceResolutionErrorV0::CrossAgentArtifact => {
            AgentLearningSessionStatusV0::RejectedUnauthorizedEvidence
        }
        EvidenceResolutionErrorV0::CutoffLeakage => {
            AgentLearningSessionStatusV0::RejectedCutoffLeakage
        }
        EvidenceResolutionErrorV0::Chronology
        | EvidenceResolutionErrorV0::Duplicate
        | EvidenceResolutionErrorV0::NonFinite
        | EvidenceResolutionErrorV0::UnsafeEvidence => {
            AgentLearningSessionStatusV0::RejectedSafetyInvariant
        }
        EvidenceResolutionErrorV0::Insufficient => {
            AgentLearningSessionStatusV0::InsufficientEvidence
        }
    }
}

fn evidence_error_code_v0(error: EvidenceResolutionErrorV0) -> &'static str {
    match error {
        EvidenceResolutionErrorV0::SourceDigest => "source_digest_rejected",
        EvidenceResolutionErrorV0::UnauthorizedDataset => "dataset_unauthorized",
        EvidenceResolutionErrorV0::CrossAgentArtifact => "cross_agent_artifact_rejected",
        EvidenceResolutionErrorV0::CutoffLeakage => "cutoff_leakage_rejected",
        EvidenceResolutionErrorV0::Chronology => "chronology_rejected",
        EvidenceResolutionErrorV0::Duplicate => "duplicate_timestamp_rejected",
        EvidenceResolutionErrorV0::NonFinite => "non_finite_evidence_rejected",
        EvidenceResolutionErrorV0::UnsafeEvidence => "safety_invariant_rejected",
        EvidenceResolutionErrorV0::Insufficient => "insufficient_evidence",
    }
}

fn resolution_session_status_v0(
    status: AgentViewResolutionStatusV0,
) -> AgentLearningSessionStatusV0 {
    match status {
        AgentViewResolutionStatusV0::Complete
        | AgentViewResolutionStatusV0::OptionalEvidenceUnavailable => {
            AgentLearningSessionStatusV0::DatasetReady
        }
        AgentViewResolutionStatusV0::MissingRequiredEvidence => {
            AgentLearningSessionStatusV0::InsufficientEvidence
        }
        AgentViewResolutionStatusV0::UnauthorizedArtifact => {
            AgentLearningSessionStatusV0::RejectedUnauthorizedEvidence
        }
        AgentViewResolutionStatusV0::CutoffLeakage => {
            AgentLearningSessionStatusV0::RejectedCutoffLeakage
        }
        AgentViewResolutionStatusV0::AmbiguousEquivalentArtifacts
        | AgentViewResolutionStatusV0::IntegrityFailure => {
            AgentLearningSessionStatusV0::RejectedSafetyInvariant
        }
    }
}

fn resolution_error_code_v0(status: AgentViewResolutionStatusV0) -> &'static str {
    match status {
        AgentViewResolutionStatusV0::Complete => "complete",
        AgentViewResolutionStatusV0::OptionalEvidenceUnavailable => "optional_evidence_unavailable",
        AgentViewResolutionStatusV0::MissingRequiredEvidence => "missing_required_evidence",
        AgentViewResolutionStatusV0::AmbiguousEquivalentArtifacts => {
            "ambiguous_equivalent_artifacts"
        }
        AgentViewResolutionStatusV0::UnauthorizedArtifact => "unauthorized_artifact",
        AgentViewResolutionStatusV0::CutoffLeakage => "cutoff_leakage",
        AgentViewResolutionStatusV0::IntegrityFailure => "integrity_failure",
    }
}

fn journal_v0(
    session: &AgentPrivateLearningSessionV0,
    manifest: Option<&AgentPrivateDatasetManifestV0>,
    candidate: Option<&AgentSandboxLearningCandidateV0>,
) -> AgentLearningSessionJournalV0 {
    let mut journal = AgentLearningSessionJournalV0 {
        journal_version: JOURNAL_VERSION_V0.to_string(),
        agent_id: session.agent_id.clone(),
        entries: vec![AgentLearningSessionJournalEntryV0 {
            session_digest: session.session_digest.clone(),
            session_status: session.session_status,
            dataset_manifest_digest: manifest.map(|manifest| manifest.manifest_digest.clone()),
            candidate_digest: candidate.map(|candidate| candidate.candidate_digest.clone()),
        }],
        journal_digest: String::new(),
    };
    journal.journal_digest = journal_digest_v0(&journal);
    journal
}

pub fn public_session_summaries_v0(
    report: &AgentPrivateLearningSessionsReportV0,
) -> Vec<AgentPrivateLearningPublicSummaryV0> {
    report
        .results
        .iter()
        .map(|result| AgentPrivateLearningPublicSummaryV0 {
            agent_id: result.session.agent_id.clone(),
            intent_digest: result.session.intent_digest.clone(),
            data_view_digest: result.session.data_view_digest.clone(),
            session_digest: result.session.session_digest.clone(),
            trainer_kind: result.trainer_kind,
            view_resolution_status: result.view_resolution_status,
            trainer_projection_digest: result
                .trainer_projection
                .as_ref()
                .map(|projection| projection.projection_digest.clone()),
            source_count: result.source_count,
            session_status: result.session.session_status,
            candidate_present: result.candidate.is_some(),
            candidate_digest: result
                .candidate
                .as_ref()
                .map(|candidate| candidate.candidate_digest.clone()),
        })
        .collect()
}

#[derive(Clone, Debug)]
struct CandidateLineageArtifactsV0 {
    trainer_kind: AgentTrainerKindV0,
    session: AgentPrivateLearningSessionV0,
    dataset_manifest: AgentPrivateDatasetManifestV0,
    candidate: AgentSandboxLearningCandidateV0,
    projection: AgentTrainerInputProjectionV0,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CandidateLineageLoadErrorV0 {
    CandidateUnavailable,
    IntegrityInvalid,
    Ambiguous,
}

pub fn run_agent_candidate_evaluation_v0(
    root: &Path,
    mode: AgentPrivateLearningRunModeV0,
    registration_requested: bool,
    protected_prospective_boundary_ms: Option<u64>,
) -> AgentCandidateEvaluationReportV0 {
    let before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let registry = agent_trainer_capability_registry_v0();
    let mut results = registry
        .capabilities
        .iter()
        .map(
            |capability| match load_candidate_lineage_v0(root, capability) {
                Ok(lineage) => build_candidate_evaluation_result_v0(
                    &lineage,
                    registration_requested,
                    protected_prospective_boundary_ms,
                ),
                Err(error) => unavailable_candidate_evaluation_result_v0(
                    &capability.agent_id,
                    error,
                    registration_requested,
                ),
            },
        )
        .collect::<Vec<_>>();
    results.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
    let after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let mut report = AgentCandidateEvaluationReportV0 {
        report_version: "agent-candidate-evaluation-report-v0".to_string(),
        mode,
        registration_requested,
        results,
        safety_counters: zero_candidate_evaluation_safety_counters_v0(),
        active_state_unchanged: before == after,
        duplicate_artifact_count: 0,
        storage_failure_count: 0,
        report_digest: String::new(),
    };
    report.report_digest = candidate_evaluation_report_digest_v0(&report);
    if mode == AgentPrivateLearningRunModeV0::ExecuteLocal {
        persist_agent_candidate_evaluation_report_v0(&mut report, root);
    }
    report
}

fn load_candidate_lineage_v0(
    root: &Path,
    capability: &AgentTrainerCapabilityV0,
) -> Result<CandidateLineageArtifactsV0, CandidateLineageLoadErrorV0> {
    if !safe_agent_component_v0(&capability.agent_id) {
        return Err(CandidateLineageLoadErrorV0::IntegrityInvalid);
    }
    let agent_root = root.join(&capability.agent_id);
    let candidates = read_direct_protobuf_files_v0(&agent_root.join("candidates"))?
        .into_iter()
        .map(|(_, bytes)| {
            decode_candidate_protobuf_v0(&bytes)
                .map_err(|_| CandidateLineageLoadErrorV0::IntegrityInvalid)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if candidates.is_empty() {
        return Err(CandidateLineageLoadErrorV0::CandidateUnavailable);
    }
    if candidates.len() != 1 {
        return Err(CandidateLineageLoadErrorV0::Ambiguous);
    }
    let candidate = candidates.into_iter().next().unwrap();
    let sessions = read_direct_protobuf_files_v0(&agent_root.join("sessions"))?
        .into_iter()
        .filter_map(|(_, bytes)| decode_session_protobuf_v0(&bytes).ok())
        .filter(|session| session.session_digest == candidate.session_digest)
        .collect::<Vec<_>>();
    if sessions.len() != 1 {
        return Err(if sessions.is_empty() {
            CandidateLineageLoadErrorV0::IntegrityInvalid
        } else {
            CandidateLineageLoadErrorV0::Ambiguous
        });
    }
    let session = sessions.into_iter().next().unwrap();
    let manifests = read_direct_protobuf_files_v0(&agent_root.join("datasets"))?
        .into_iter()
        .filter_map(|(_, bytes)| decode_dataset_manifest_protobuf_v0(&bytes).ok())
        .filter(|manifest| manifest.session_id == session.session_id)
        .collect::<Vec<_>>();
    if manifests.len() != 1 {
        return Err(if manifests.is_empty() {
            CandidateLineageLoadErrorV0::IntegrityInvalid
        } else {
            CandidateLineageLoadErrorV0::Ambiguous
        });
    }
    let dataset_manifest = manifests.into_iter().next().unwrap();
    if candidate.agent_id != capability.agent_id
        || session.agent_id != capability.agent_id
        || dataset_manifest.agent_id != capability.agent_id
        || candidate.data_view_digest != session.data_view_digest
        || dataset_manifest.data_view_digest != session.data_view_digest
        || dataset_manifest.source_artifact_digests != session.source_artifact_digests
        || candidate.candidate_digest != candidate_digest_v0(&candidate)
        || session.session_digest != session_digest_v0(&session)
        || dataset_manifest.manifest_digest != dataset_manifest_digest_v0(&dataset_manifest)
        || validate_candidate_v0(&candidate).is_err()
        || validate_dataset_manifest_v0(&dataset_manifest).is_err()
    {
        return Err(CandidateLineageLoadErrorV0::IntegrityInvalid);
    }
    let projection = if let Some(expected) = session.trainer_projection_digest.as_ref() {
        let projections = read_direct_protobuf_files_v0(&agent_root.join("projections"))?
            .into_iter()
            .filter_map(|(_, bytes)| decode_trainer_projection_protobuf_v0(&bytes).ok())
            .filter(|projection| projection.projection_digest == *expected)
            .collect::<Vec<_>>();
        if projections.len() != 1 {
            return Err(CandidateLineageLoadErrorV0::IntegrityInvalid);
        }
        projections.into_iter().next().unwrap()
    } else {
        reconstruct_preliminary_projection_v0(capability.trainer_kind, &session, &dataset_manifest)?
    };
    Ok(CandidateLineageArtifactsV0 {
        trainer_kind: capability.trainer_kind,
        session,
        dataset_manifest,
        candidate,
        projection,
    })
}

fn read_direct_protobuf_files_v0(
    directory: &Path,
) -> Result<Vec<(PathBuf, Vec<u8>)>, CandidateLineageLoadErrorV0> {
    if !directory.is_dir() {
        return Ok(vec![]);
    }
    let mut paths = fs::read_dir(directory)
        .map_err(|_| CandidateLineageLoadErrorV0::IntegrityInvalid)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|value| value == "pb"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let bytes =
                fs::read(&path).map_err(|_| CandidateLineageLoadErrorV0::IntegrityInvalid)?;
            Ok((path, bytes))
        })
        .collect()
}

fn reconstruct_preliminary_projection_v0(
    trainer_kind: AgentTrainerKindV0,
    session: &AgentPrivateLearningSessionV0,
    manifest: &AgentPrivateDatasetManifestV0,
) -> Result<AgentTrainerInputProjectionV0, CandidateLineageLoadErrorV0> {
    if manifest.source_artifact_digests.len() != 1
        || manifest.source_artifact_digests != session.source_artifact_digests
    {
        return Err(CandidateLineageLoadErrorV0::Ambiguous);
    }
    let primary = manifest.source_artifact_digests[0].clone();
    let mut projection = AgentTrainerInputProjectionV0 {
        projection_version: PROJECTION_VERSION_V0.to_string(),
        agent_id: session.agent_id.clone(),
        trainer_kind,
        source_view_digest: session.data_view_digest.clone(),
        consumed_artifact_digests: vec![primary.clone()],
        referenced_but_unconsumed_artifact_digests: vec![],
        primary_series_digest: Some(primary),
        auxiliary_series_digests: vec![],
        projection_policy_digest: stable_hash_string(
            "SOMA-PRELIMINARY-SINGLE-ARTIFACT-EFFECTIVE-PROJECTION-V0",
        ),
        projection_digest: String::new(),
    };
    projection.projection_digest = projection_digest_v0(&projection);
    Ok(projection)
}

fn build_candidate_evaluation_result_v0(
    lineage: &CandidateLineageArtifactsV0,
    registration_requested: bool,
    protected_prospective_boundary_ms: Option<u64>,
) -> AgentCandidateEvaluationResultV0 {
    let ledger = candidate_evidence_usage_ledger_v0(lineage);
    let audit = agent_candidate_identity_audit_v0(lineage, &ledger);
    let registration = agent_candidate_evaluation_registration_v0(
        lineage,
        &ledger,
        &audit,
        protected_prospective_boundary_ms,
    );
    let journal =
        registration_requested.then(|| candidate_evaluation_registration_journal_v0(&registration));
    AgentCandidateEvaluationResultV0 {
        agent_id: lineage.session.agent_id.clone(),
        candidate_digest: Some(lineage.candidate.candidate_digest.clone()),
        session_digest: Some(lineage.session.session_digest.clone()),
        view_digest: Some(lineage.session.data_view_digest.clone()),
        trainer_projection: Some(lineage.projection.clone()),
        evidence_usage_ledger: Some(ledger),
        identity_audit: Some(audit),
        evaluation_registration: registration_requested.then_some(registration.clone()),
        registration_journal: journal,
        blocked_status: registration.status,
        sanitized_error_code: (registration.status
            != CandidateEvaluationRegistrationStatusV0::Registered)
            .then(|| registration_status_code_v0(registration.status).to_string()),
    }
}

fn unavailable_candidate_evaluation_result_v0(
    agent_id: &str,
    error: CandidateLineageLoadErrorV0,
    _registration_requested: bool,
) -> AgentCandidateEvaluationResultV0 {
    let (status, code) = match error {
        CandidateLineageLoadErrorV0::CandidateUnavailable => (
            CandidateEvaluationRegistrationStatusV0::CandidateUnavailable,
            "candidate_unavailable",
        ),
        CandidateLineageLoadErrorV0::IntegrityInvalid => (
            CandidateEvaluationRegistrationStatusV0::CandidateIntegrityInvalid,
            "candidate_integrity_invalid",
        ),
        CandidateLineageLoadErrorV0::Ambiguous => (
            CandidateEvaluationRegistrationStatusV0::LineageAmbiguousBlocked,
            "candidate_lineage_ambiguous",
        ),
    };
    AgentCandidateEvaluationResultV0 {
        agent_id: agent_id.to_string(),
        candidate_digest: None,
        session_digest: None,
        view_digest: None,
        trainer_projection: None,
        evidence_usage_ledger: None,
        identity_audit: None,
        evaluation_registration: None,
        registration_journal: None,
        blocked_status: status,
        sanitized_error_code: Some(code.to_string()),
    }
}

fn candidate_evidence_usage_ledger_v0(
    lineage: &CandidateLineageArtifactsV0,
) -> CandidateEvidenceUsageLedgerV0 {
    let manifest = &lineage.dataset_manifest;
    let test_range = manifest.sealed_test_range.clone().unwrap_or(IndexRangeV0 {
        start: manifest.row_count,
        end: manifest.row_count,
    });
    let mut entries = Vec::new();
    for artifact_digest in &manifest.source_artifact_digests {
        entries.push(evidence_usage_entry_v0(
            artifact_digest,
            None,
            CandidateEvidenceUseV0::IntentBinding,
            false,
            false,
            false,
            true,
        ));
        entries.push(evidence_usage_entry_v0(
            artifact_digest,
            None,
            CandidateEvidenceUseV0::ViewBinding,
            false,
            false,
            false,
            true,
        ));
        if lineage
            .projection
            .consumed_artifact_digests
            .contains(artifact_digest)
        {
            entries.push(evidence_usage_entry_v0(
                artifact_digest,
                None,
                CandidateEvidenceUseV0::TrainerProjection,
                false,
                false,
                false,
                true,
            ));
            for (range, use_kind, labels, updates, selection, identity) in [
                (
                    manifest.training_range.clone(),
                    CandidateEvidenceUseV0::FeatureDerivation,
                    false,
                    false,
                    false,
                    true,
                ),
                (
                    manifest.training_range.clone(),
                    CandidateEvidenceUseV0::LabelDerivation,
                    true,
                    false,
                    false,
                    true,
                ),
                (
                    manifest.normalizer_fit_range.clone(),
                    CandidateEvidenceUseV0::NormalizerFit,
                    false,
                    false,
                    false,
                    true,
                ),
                (
                    manifest.training_range.clone(),
                    CandidateEvidenceUseV0::ParameterTraining,
                    true,
                    true,
                    false,
                    true,
                ),
                (
                    manifest.validation_range.clone(),
                    CandidateEvidenceUseV0::FeatureDerivation,
                    false,
                    false,
                    false,
                    true,
                ),
                (
                    manifest.validation_range.clone(),
                    CandidateEvidenceUseV0::LabelDerivation,
                    true,
                    false,
                    false,
                    true,
                ),
                (
                    manifest.validation_range.clone(),
                    CandidateEvidenceUseV0::ValidationInference,
                    false,
                    false,
                    false,
                    true,
                ),
                (
                    manifest.validation_range.clone(),
                    CandidateEvidenceUseV0::ValidationMetric,
                    true,
                    false,
                    false,
                    true,
                ),
                (
                    manifest.validation_range.clone(),
                    CandidateEvidenceUseV0::CheckpointSelection,
                    true,
                    false,
                    true,
                    true,
                ),
                (
                    test_range.clone(),
                    CandidateEvidenceUseV0::FeatureDerivation,
                    false,
                    false,
                    false,
                    true,
                ),
                (
                    test_range.clone(),
                    CandidateEvidenceUseV0::LabelDerivation,
                    true,
                    false,
                    false,
                    true,
                ),
                (
                    test_range.clone(),
                    CandidateEvidenceUseV0::HistoricalTestInference,
                    false,
                    false,
                    false,
                    true,
                ),
                (
                    test_range.clone(),
                    CandidateEvidenceUseV0::HistoricalTestMetric,
                    true,
                    false,
                    false,
                    true,
                ),
                (
                    test_range.clone(),
                    CandidateEvidenceUseV0::CandidateIdentity,
                    true,
                    false,
                    false,
                    true,
                ),
            ] {
                entries.push(evidence_usage_entry_v0(
                    artifact_digest,
                    Some(range),
                    use_kind,
                    labels,
                    updates,
                    selection,
                    identity,
                ));
            }
        } else {
            entries.push(evidence_usage_entry_v0(
                artifact_digest,
                None,
                CandidateEvidenceUseV0::Unused,
                false,
                false,
                false,
                false,
            ));
        }
    }
    let mut ledger = CandidateEvidenceUsageLedgerV0 {
        ledger_version: EVIDENCE_LEDGER_VERSION_V0.to_string(),
        agent_id: lineage.session.agent_id.clone(),
        candidate_digest: lineage.candidate.candidate_digest.clone(),
        session_digest: lineage.session.session_digest.clone(),
        entries,
        ledger_digest: String::new(),
    };
    ledger.ledger_digest = evidence_usage_ledger_digest_v0(&ledger);
    ledger
}

fn evidence_usage_entry_v0(
    artifact_digest: &str,
    range: Option<IndexRangeV0>,
    use_kind: CandidateEvidenceUseV0,
    labels_read: bool,
    parameters_updated: bool,
    checkpoint_selection_influenced: bool,
    candidate_identity_influenced: bool,
) -> CandidateEvidenceUsageEntryV0 {
    let mut entry = CandidateEvidenceUsageEntryV0 {
        artifact_digest: artifact_digest.to_string(),
        range,
        use_kind,
        labels_read,
        parameters_updated,
        checkpoint_selection_influenced,
        candidate_identity_influenced,
        entry_digest: String::new(),
    };
    entry.entry_digest = evidence_usage_entry_digest_v0(&entry);
    entry
}

fn agent_candidate_identity_audit_v0(
    lineage: &CandidateLineageArtifactsV0,
    ledger: &CandidateEvidenceUsageLedgerV0,
) -> AgentCandidateIdentityAuditV0 {
    let mut model_identity_inputs = vec![
        lineage.session.session_digest.clone(),
        lineage.session.data_view_digest.clone(),
        lineage.projection.projection_digest.clone(),
        lineage.candidate.model_artifact_digest.clone(),
        lineage.candidate.normalizer_digest.clone(),
        lineage.candidate.training_policy_digest.clone(),
        lineage.candidate.feature_policy_digest.clone(),
        lineage.candidate.label_policy_digest.clone(),
    ];
    model_identity_inputs.extend(lineage.session.source_artifact_digests.clone());
    model_identity_inputs.sort();
    model_identity_inputs.dedup();
    let metric_identity_inputs = (!lineage.candidate.private_metrics_digest.is_empty())
        .then(|| vec![lineage.candidate.private_metrics_digest.clone()])
        .unwrap_or_default();
    let test_metric_recorded = ledger.entries.iter().any(|entry| {
        entry.use_kind == CandidateEvidenceUseV0::HistoricalTestMetric && entry.labels_read
    });
    let test_identity_recorded = ledger.entries.iter().any(|entry| {
        entry.use_kind == CandidateEvidenceUseV0::CandidateIdentity
            && entry.candidate_identity_influenced
    });
    let historical_test_status = if lineage.dataset_manifest.sealed_test_range.is_none() {
        CandidateHistoricalTestStatusV0::LineageAmbiguous
    } else if test_identity_recorded && !metric_identity_inputs.is_empty() {
        CandidateHistoricalTestStatusV0::InfluencedCandidateIdentity
    } else if test_metric_recorded {
        CandidateHistoricalTestStatusV0::MetricsAlreadyComputed
    } else if ledger
        .entries
        .iter()
        .any(|entry| entry.use_kind == CandidateEvidenceUseV0::HistoricalTestInference)
    {
        CandidateHistoricalTestStatusV0::ReadForInferenceOnly
    } else {
        CandidateHistoricalTestStatusV0::FreshAndSealed
    };
    let full_hardened_policy_binding = lineage.session.session_version == SESSION_VERSION_V1
        && !lineage.session.source_policy_digest.is_empty()
        && !lineage.session.required_dataset_kinds.is_empty()
        && lineage.session.trainer_projection_digest.as_deref()
            == Some(lineage.projection.projection_digest.as_str());
    let mut audit = AgentCandidateIdentityAuditV0 {
        audit_version: IDENTITY_AUDIT_VERSION_V0.to_string(),
        candidate_digest: lineage.candidate.candidate_digest.clone(),
        model_identity_inputs,
        metric_identity_inputs,
        test_evidence_in_identity: test_identity_recorded,
        historical_test_status,
        eligible_for_fresh_historical_test: historical_test_status
            == CandidateHistoricalTestStatusV0::FreshAndSealed,
        eligible_for_future_evaluation_registration: historical_test_status
            != CandidateHistoricalTestStatusV0::LineageAmbiguous
            && full_hardened_policy_binding,
        superseded_by_input_binding_hardening: !full_hardened_policy_binding,
        audit_digest: String::new(),
    };
    audit.audit_digest = identity_audit_digest_v0(&audit);
    audit
}

fn agent_candidate_evaluation_registration_v0(
    lineage: &CandidateLineageArtifactsV0,
    ledger: &CandidateEvidenceUsageLedgerV0,
    audit: &AgentCandidateIdentityAuditV0,
    protected_prospective_boundary_ms: Option<u64>,
) -> AgentCandidateEvaluationRegistrationV0 {
    let comparators = lineage
        .candidate
        .parent_model_version
        .as_ref()
        .map(|parent| {
            vec![stable_hash_string(&format!(
                "SOMA-FROZEN-PARENT-COMPARATOR-V0:{parent}"
            ))]
        })
        .unwrap_or_default();
    let policy_valid = audit.eligible_for_future_evaluation_registration
        && !lineage.session.source_policy_digest.is_empty()
        && !lineage.session.required_dataset_kinds.is_empty()
        && !lineage.session.label_policy_digest.is_empty();
    let status = if validate_candidate_v0(&lineage.candidate).is_err() {
        CandidateEvaluationRegistrationStatusV0::CandidateIntegrityInvalid
    } else if audit.historical_test_status == CandidateHistoricalTestStatusV0::LineageAmbiguous {
        CandidateEvaluationRegistrationStatusV0::LineageAmbiguousBlocked
    } else if !policy_valid {
        CandidateEvaluationRegistrationStatusV0::PolicyInvalid
    } else if comparators.is_empty() {
        CandidateEvaluationRegistrationStatusV0::ComparatorUnavailable
    } else {
        CandidateEvaluationRegistrationStatusV0::Registered
    };
    let evaluation_cutoff_exclusive_ms = protected_prospective_boundary_ms
        .unwrap_or_default()
        .max(lineage.session.information_cutoff_ms)
        .max(lineage.dataset_manifest.information_cutoff_ms);
    let (minimum_future_rows, minimum_mature_events) = match lineage.trainer_kind {
        AgentTrainerKindV0::MomentumFrozenMambaHead => (64, 32),
        AgentTrainerKindV0::CycleRiskIndependentShadow => (96, 48),
        AgentTrainerKindV0::ValueQualityUnavailable => (0, 0),
    };
    let mut registration = AgentCandidateEvaluationRegistrationV0 {
        registration_version: EVALUATION_REGISTRATION_VERSION_V0.to_string(),
        agent_id: lineage.session.agent_id.clone(),
        candidate_digest: lineage.candidate.candidate_digest.clone(),
        session_digest: lineage.session.session_digest.clone(),
        evidence_usage_ledger_digest: ledger.ledger_digest.clone(),
        identity_audit_digest: audit.audit_digest.clone(),
        evaluation_cutoff_exclusive_ms,
        required_dataset_kinds: lineage.session.required_dataset_kinds.clone(),
        source_policy_digest: lineage.session.source_policy_digest.clone(),
        finality_policy_digest: stable_hash_string("SOMA-FUTURE-FINALIZED-EVIDENCE-ONLY-V0"),
        label_policy_digest: lineage.session.label_policy_digest.clone(),
        metric_policy_digest: stable_hash_string(
            "SOMA-FUTURE-CANDIDATE-BRIER-CALIBRATION-METRIC-V0",
        ),
        support_policy_digest: stable_hash_string("SOMA-FUTURE-CANDIDATE-MINIMUM-SUPPORT-V0"),
        comparator_digests: comparators,
        minimum_future_rows,
        minimum_mature_events,
        maximum_requests: 1,
        maximum_concurrency: 1,
        maximum_retries: 0,
        labels_hidden_until_opening: true,
        probabilities_hidden_until_opening: true,
        one_time_opening_required: true,
        active_promotion_forbidden: true,
        reward_application_forbidden: true,
        status,
        registration_digest: String::new(),
    };
    registration.registration_digest = evaluation_registration_digest_v0(&registration);
    registration
}

fn candidate_evaluation_registration_journal_v0(
    registration: &AgentCandidateEvaluationRegistrationV0,
) -> AgentCandidateEvaluationRegistrationJournalV0 {
    let mut journal = AgentCandidateEvaluationRegistrationJournalV0 {
        journal_version: EVALUATION_JOURNAL_VERSION_V0.to_string(),
        agent_id: registration.agent_id.clone(),
        candidate_digest: registration.candidate_digest.clone(),
        entries: vec![AgentCandidateEvaluationRegistrationJournalEntryV0 {
            registration_digest: registration.registration_digest.clone(),
            status: registration.status,
        }],
        journal_digest: String::new(),
    };
    journal.journal_digest = evaluation_journal_digest_v0(&journal);
    journal
}

pub fn candidate_evaluation_accepts_timestamp_v0(
    registration: &AgentCandidateEvaluationRegistrationV0,
    event_timestamp_ms: u64,
) -> bool {
    registration.status == CandidateEvaluationRegistrationStatusV0::Registered
        && event_timestamp_ms > registration.evaluation_cutoff_exclusive_ms
}

pub fn public_candidate_evaluation_summaries_v0(
    report: &AgentCandidateEvaluationReportV0,
) -> Vec<AgentCandidateEvaluationPublicSummaryV0> {
    report
        .results
        .iter()
        .map(|result| AgentCandidateEvaluationPublicSummaryV0 {
            agent_id: result.agent_id.clone(),
            candidate_digest: result.candidate_digest.clone(),
            session_digest: result.session_digest.clone(),
            view_digest: result.view_digest.clone(),
            projection_digest: result
                .trainer_projection
                .as_ref()
                .map(|projection| projection.projection_digest.clone()),
            historical_test_status: result
                .identity_audit
                .as_ref()
                .map(|audit| audit.historical_test_status)
                .unwrap_or(CandidateHistoricalTestStatusV0::NoCandidate),
            evidence_usage_ledger_digest: result
                .evidence_usage_ledger
                .as_ref()
                .map(|ledger| ledger.ledger_digest.clone()),
            identity_audit_digest: result
                .identity_audit
                .as_ref()
                .map(|audit| audit.audit_digest.clone()),
            evaluation_cutoff_exclusive_ms: result
                .evaluation_registration
                .as_ref()
                .map(|registration| registration.evaluation_cutoff_exclusive_ms),
            registration_status: result
                .evaluation_registration
                .as_ref()
                .map(|registration| registration.status)
                .unwrap_or(result.blocked_status),
            comparator_count: result
                .evaluation_registration
                .as_ref()
                .map(|registration| registration.comparator_digests.len())
                .unwrap_or(0),
        })
        .collect()
}

pub fn load_local_learning_snapshots_v0(root: &Path) -> Result<Vec<DataSnapshot>, String> {
    let mut paths = Vec::new();
    collect_protobuf_paths_v0(root, &mut paths)?;
    paths.sort();
    let mut snapshots = Vec::new();
    let mut digests = BTreeSet::new();
    for path in paths {
        let snapshot = read_local_snapshot_protobuf_v1(&path)?;
        if digests.insert(snapshot.content_digest.clone()) {
            snapshots.push(snapshot);
        }
    }
    snapshots.sort_by(|left, right| left.content_digest.cmp(&right.content_digest));
    Ok(snapshots)
}

fn collect_protobuf_paths_v0(root: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    for entry in
        fs::read_dir(root).map_err(|_| "local evidence directory unavailable".to_string())?
    {
        let entry = entry.map_err(|_| "local evidence directory entry unavailable".to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect_protobuf_paths_v0(&path, paths)?;
        } else if path.extension().is_some_and(|extension| extension == "pb")
            && !path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with('.'))
        {
            paths.push(path);
        }
    }
    Ok(())
}

pub fn default_private_learning_root_v0() -> &'static Path {
    Path::new(DEFAULT_PRIVATE_LEARNING_ROOT_V0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentPrivateLearningArtifactWriteStatusV0 {
    Written,
    DuplicateRejected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPrivateLearningStorageReportV0 {
    pub written_artifact_count: usize,
    pub duplicate_artifact_count: usize,
    pub failed_artifact_count: usize,
}

pub fn persist_agent_private_learning_report_v0(
    report: &mut AgentPrivateLearningSessionsReportV0,
    root: &Path,
) -> AgentPrivateLearningStorageReportV0 {
    let mut storage = AgentPrivateLearningStorageReportV0 {
        written_artifact_count: 0,
        duplicate_artifact_count: 0,
        failed_artifact_count: 0,
    };
    record_write_result_v0(
        write_registry_artifact_v0(&report.capability_registry, root),
        &mut storage,
    );
    for result in &report.results {
        if !safe_agent_component_v0(&result.session.agent_id) {
            storage.failed_artifact_count += 1;
            continue;
        }
        let agent_root = root.join(&result.session.agent_id);
        record_write_result_v0(
            write_session_artifact_v0(&result.session, &agent_root.join("sessions")),
            &mut storage,
        );
        if let Some(projection) = &result.trainer_projection {
            record_write_result_v0(
                write_projection_artifact_v0(projection, &agent_root.join("projections")),
                &mut storage,
            );
        }
        if let Some(manifest) = &result.dataset_manifest {
            record_write_result_v0(
                write_dataset_artifact_v0(manifest, &agent_root.join("datasets")),
                &mut storage,
            );
        }
        if let Some(candidate) = &result.candidate {
            record_write_result_v0(
                write_candidate_artifact_v0(candidate, &agent_root.join("candidates")),
                &mut storage,
            );
        }
        record_write_result_v0(
            write_journal_artifact_v0(&result.journal, &agent_root.join("journals")),
            &mut storage,
        );
    }
    report.duplicate_artifact_count = storage.duplicate_artifact_count;
    report.storage_failure_count = storage.failed_artifact_count;
    report.report_digest = report_digest_v0(report);
    storage
}

pub fn persist_agent_candidate_evaluation_report_v0(
    report: &mut AgentCandidateEvaluationReportV0,
    root: &Path,
) -> AgentPrivateLearningStorageReportV0 {
    let mut storage = AgentPrivateLearningStorageReportV0 {
        written_artifact_count: 0,
        duplicate_artifact_count: 0,
        failed_artifact_count: 0,
    };
    for result in &report.results {
        if !safe_agent_component_v0(&result.agent_id) {
            storage.failed_artifact_count += 1;
            continue;
        }
        let evaluation_root = root.join("evaluation").join(&result.agent_id);
        if let Some(projection) = &result.trainer_projection {
            record_write_result_v0(
                write_projection_artifact_v0(projection, &evaluation_root.join("projections")),
                &mut storage,
            );
        }
        if let Some(ledger) = &result.evidence_usage_ledger {
            record_write_result_v0(
                write_evidence_usage_ledger_artifact_v0(
                    ledger,
                    &evaluation_root.join("evidence_usage"),
                ),
                &mut storage,
            );
        }
        if let Some(audit) = &result.identity_audit {
            record_write_result_v0(
                write_identity_audit_artifact_v0(audit, &evaluation_root.join("identity_audits")),
                &mut storage,
            );
        }
        if let Some(registration) = &result.evaluation_registration {
            record_write_result_v0(
                write_evaluation_registration_artifact_v0(
                    registration,
                    &evaluation_root.join("registrations"),
                ),
                &mut storage,
            );
        }
        if let Some(journal) = &result.registration_journal {
            record_write_result_v0(
                write_evaluation_journal_artifact_v0(
                    journal,
                    &evaluation_root.join("registration_journals"),
                ),
                &mut storage,
            );
        }
    }
    report.duplicate_artifact_count = storage.duplicate_artifact_count;
    report.storage_failure_count = storage.failed_artifact_count;
    report.report_digest = candidate_evaluation_report_digest_v0(report);
    storage
}

pub fn persist_agent_candidate_families_report_v1(
    report: &mut AgentCandidateFamiliesReportV1,
    root: &Path,
) -> AgentPrivateLearningStorageReportV0 {
    let mut storage = AgentPrivateLearningStorageReportV0 {
        written_artifact_count: 0,
        duplicate_artifact_count: 0,
        failed_artifact_count: 0,
    };
    for result in &report.results {
        if !safe_agent_component_v0(&result.agent_id) {
            storage.failed_artifact_count += 1;
            continue;
        }
        let agent_root = root.join("v1").join(&result.agent_id);
        if let Some(session) = &result.session {
            record_write_result_v0(
                write_session_artifact_v1(session, &agent_root.join("sessions")),
                &mut storage,
            );
        }
        if let Some(projection) = &result.projection {
            record_write_result_v0(
                write_projection_artifact_v1(projection, &agent_root.join("projections")),
                &mut storage,
            );
        }
        if let Some(family) = &result.family {
            for participant in &family.participants {
                record_write_result_v0(
                    write_participant_artifact_v1(participant, &agent_root.join("participants")),
                    &mut storage,
                );
            }
            record_write_result_v0(
                write_family_artifact_v1(family, &agent_root.join("families")),
                &mut storage,
            );
        }
        for receipt in &result.qualification_receipts {
            record_write_result_v0(
                write_qualification_artifact_v1(
                    receipt,
                    &agent_root.join("qualification_receipts"),
                ),
                &mut storage,
            );
        }
        if let Some(ledger) = &result.usage_ledger {
            record_write_result_v0(
                write_usage_ledger_artifact_v1(ledger, &agent_root.join("usage_ledgers")),
                &mut storage,
            );
        }
    }
    report.duplicate_artifact_count = storage.duplicate_artifact_count;
    report.storage_failure_count = storage.failed_artifact_count;
    report.report_digest = candidate_families_report_digest_v1(report);
    storage
}

pub fn persist_agent_candidate_evaluations_report_v1(
    report: &mut AgentCandidateEvaluationsReportV1,
    root: &Path,
) -> AgentPrivateLearningStorageReportV0 {
    let mut storage = AgentPrivateLearningStorageReportV0 {
        written_artifact_count: 0,
        duplicate_artifact_count: 0,
        failed_artifact_count: 0,
    };
    for result in &report.results {
        if !safe_agent_component_v0(&result.agent_id) {
            storage.failed_artifact_count += 1;
            continue;
        }
        let agent_root = root.join("evaluation_v1").join(&result.agent_id);
        if let Some(exclusion) = &result.exclusion {
            record_write_result_v0(
                write_exclusion_artifact_v1(exclusion, &agent_root.join("exclusions")),
                &mut storage,
            );
        }
        if let Some(registration) = &result.registration {
            record_write_result_v0(
                write_evaluation_registration_artifact_v1(
                    registration,
                    &agent_root.join("registrations"),
                ),
                &mut storage,
            );
        }
        if let Some(journal) = &result.journal {
            record_write_result_v0(
                write_evaluation_journal_artifact_v1(
                    journal,
                    &agent_root.join("registration_journals"),
                ),
                &mut storage,
            );
        }
    }
    report.duplicate_artifact_count = storage.duplicate_artifact_count;
    report.storage_failure_count = storage.failed_artifact_count;
    report.report_digest = candidate_evaluations_report_digest_v1(report);
    storage
}

fn write_session_artifact_v1(
    session: &AgentPrivateLearningSessionV1,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_session_protobuf_v1(session)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", session.session_digest)),
        &bytes,
        &session.session_digest,
        |stored| Ok(decode_session_protobuf_v1(stored)?.session_digest),
    )
}

fn write_projection_artifact_v1(
    projection: &AgentTrainerInputProjectionV1,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_trainer_projection_protobuf_v1(projection)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", projection.projection_digest)),
        &bytes,
        &projection.projection_digest,
        |stored| Ok(decode_trainer_projection_protobuf_v1(stored)?.projection_digest),
    )
}

fn write_participant_artifact_v1(
    participant: &FrozenCandidateParticipantV1,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_participant_protobuf_v1(participant)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", participant.participant_digest)),
        &bytes,
        &participant.participant_digest,
        |stored| Ok(decode_participant_protobuf_v1(stored)?.participant_digest),
    )
}

fn write_qualification_artifact_v1(
    receipt: &ParticipantValidationQualificationV1,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_qualification_receipt_protobuf_v1(receipt)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", receipt.receipt_digest)),
        &bytes,
        &receipt.receipt_digest,
        |stored| Ok(decode_qualification_receipt_protobuf_v1(stored)?.receipt_digest),
    )
}

fn write_family_artifact_v1(
    family: &AgentCandidateFamilyV1,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_candidate_family_protobuf_v1(family)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", family.family_digest)),
        &bytes,
        &family.family_digest,
        |stored| Ok(decode_candidate_family_protobuf_v1(stored)?.family_digest),
    )
}

fn write_usage_ledger_artifact_v1(
    ledger: &AgentCandidateUsageLedgerV1,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_usage_ledger_protobuf_v1(ledger)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", ledger.ledger_digest)),
        &bytes,
        &ledger.ledger_digest,
        |stored| Ok(decode_usage_ledger_protobuf_v1(stored)?.ledger_digest),
    )
}

fn write_exclusion_artifact_v1(
    exclusion: &EvaluationEvidenceExclusionV1,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_evidence_exclusion_protobuf_v1(exclusion)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", exclusion.exclusion_digest)),
        &bytes,
        &exclusion.exclusion_digest,
        |stored| Ok(decode_evidence_exclusion_protobuf_v1(stored)?.exclusion_digest),
    )
}

fn write_evaluation_registration_artifact_v1(
    registration: &AgentCandidateEvaluationRegistrationV1,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_evaluation_registration_protobuf_v1(registration)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", registration.registration_digest)),
        &bytes,
        &registration.registration_digest,
        |stored| Ok(decode_evaluation_registration_protobuf_v1(stored)?.registration_digest),
    )
}

fn write_evaluation_journal_artifact_v1(
    journal: &AgentCandidateEvaluationRegistrationJournalV1,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_evaluation_journal_protobuf_v1(journal)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", journal.journal_digest)),
        &bytes,
        &journal.journal_digest,
        |stored| Ok(decode_evaluation_journal_protobuf_v1(stored)?.journal_digest),
    )
}

fn write_evidence_usage_ledger_artifact_v0(
    ledger: &CandidateEvidenceUsageLedgerV0,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_evidence_usage_ledger_protobuf_v0(ledger)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", ledger.ledger_digest)),
        &bytes,
        &ledger.ledger_digest,
        |stored| Ok(decode_evidence_usage_ledger_protobuf_v0(stored)?.ledger_digest),
    )
}

fn write_identity_audit_artifact_v0(
    audit: &AgentCandidateIdentityAuditV0,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_candidate_identity_audit_protobuf_v0(audit)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", audit.audit_digest)),
        &bytes,
        &audit.audit_digest,
        |stored| Ok(decode_candidate_identity_audit_protobuf_v0(stored)?.audit_digest),
    )
}

fn write_evaluation_registration_artifact_v0(
    registration: &AgentCandidateEvaluationRegistrationV0,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_candidate_evaluation_registration_protobuf_v0(registration)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", registration.registration_digest)),
        &bytes,
        &registration.registration_digest,
        |stored| {
            Ok(decode_candidate_evaluation_registration_protobuf_v0(stored)?.registration_digest)
        },
    )
}

fn write_evaluation_journal_artifact_v0(
    journal: &AgentCandidateEvaluationRegistrationJournalV0,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_candidate_evaluation_journal_protobuf_v0(journal)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", journal.journal_digest)),
        &bytes,
        &journal.journal_digest,
        |stored| Ok(decode_candidate_evaluation_journal_protobuf_v0(stored)?.journal_digest),
    )
}

fn record_write_result_v0(
    result: Result<AgentPrivateLearningArtifactWriteStatusV0, String>,
    storage: &mut AgentPrivateLearningStorageReportV0,
) {
    match result {
        Ok(AgentPrivateLearningArtifactWriteStatusV0::Written) => {
            storage.written_artifact_count += 1;
        }
        Ok(AgentPrivateLearningArtifactWriteStatusV0::DuplicateRejected) => {
            storage.duplicate_artifact_count += 1;
        }
        Err(_) => storage.failed_artifact_count += 1,
    }
}

fn write_registry_artifact_v0(
    registry: &AgentTrainerCapabilityRegistryV0,
    root: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_capability_registry_protobuf_v0(registry)?;
    atomic_write_verified_v0(
        &root
            .join("capability_registries")
            .join(format!("{}.pb", registry.registry_digest)),
        &bytes,
        &registry.registry_digest,
        |stored| Ok(decode_capability_registry_protobuf_v0(stored)?.registry_digest),
    )
}

fn write_projection_artifact_v0(
    projection: &AgentTrainerInputProjectionV0,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_trainer_projection_protobuf_v0(projection)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", projection.projection_digest)),
        &bytes,
        &projection.projection_digest,
        |stored| Ok(decode_trainer_projection_protobuf_v0(stored)?.projection_digest),
    )
}

fn write_session_artifact_v0(
    session: &AgentPrivateLearningSessionV0,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_session_protobuf_v0(session)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", session.session_id)),
        &bytes,
        &session.session_digest,
        |stored| Ok(decode_session_protobuf_v0(stored)?.session_digest),
    )
}

fn write_dataset_artifact_v0(
    manifest: &AgentPrivateDatasetManifestV0,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_dataset_manifest_protobuf_v0(manifest)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", manifest.dataset_id)),
        &bytes,
        &manifest.manifest_digest,
        |stored| Ok(decode_dataset_manifest_protobuf_v0(stored)?.manifest_digest),
    )
}

fn write_candidate_artifact_v0(
    candidate: &AgentSandboxLearningCandidateV0,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_candidate_protobuf_v0(candidate)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", candidate.candidate_digest)),
        &bytes,
        &candidate.candidate_digest,
        |stored| Ok(decode_candidate_protobuf_v0(stored)?.candidate_digest),
    )
}

fn write_journal_artifact_v0(
    journal: &AgentLearningSessionJournalV0,
    directory: &Path,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String> {
    let bytes = encode_journal_protobuf_v0(journal)?;
    atomic_write_verified_v0(
        &directory.join(format!("{}.pb", journal.journal_digest)),
        &bytes,
        &journal.journal_digest,
        |stored| Ok(decode_journal_protobuf_v0(stored)?.journal_digest),
    )
}

fn intent_migration_agent_root_v1(root: &Path) -> PathBuf {
    root.join("intent_migration_v1").join(MOMENTUM_AGENT_ID_V1)
}

fn migration_round_trip_v1(derived: &DerivedPersistedIntentMigrationV1) -> Result<(), String> {
    let intent = decode_canonical_learning_intent_migration_protobuf_v1(
        &encode_canonical_learning_intent_migration_protobuf_v1(&derived.canonical_intent)?,
    )?;
    let (_, view) = decode_agent_learning_data_view_protobuf_v0(
        &encode_agent_learning_data_view_protobuf_v0(&derived.canonical_view)?,
    )?;
    let policy = decode_intent_policy_compatibility_proof_protobuf_v1(
        &encode_intent_policy_compatibility_proof_protobuf_v1(&derived.policy_proof)?,
    )?;
    let proof = decode_learning_intent_migration_proof_protobuf_v1(
        &encode_learning_intent_migration_proof_protobuf_v1(&derived.migration_proof)?,
    )?;
    let journal = decode_learning_intent_migration_journal_protobuf_v1(
        &encode_learning_intent_migration_journal_protobuf_v1(&derived.journal)?,
    )?;
    if intent != derived.canonical_intent
        || view != derived.canonical_view
        || policy != derived.policy_proof
        || proof != derived.migration_proof
        || journal != derived.journal
    {
        return Err("intent migration Protobuf round trip rejected".to_string());
    }
    Ok(())
}

fn record_migration_write_status_v1(
    status: AgentPrivateLearningArtifactWriteStatusV0,
    written: &mut usize,
    duplicates: &mut usize,
) {
    match status {
        AgentPrivateLearningArtifactWriteStatusV0::Written => *written += 1,
        AgentPrivateLearningArtifactWriteStatusV0::DuplicateRejected => *duplicates += 1,
    }
}

fn persist_persisted_learning_intent_migration_v1(
    derived: &DerivedPersistedIntentMigrationV1,
    root: &Path,
) -> Result<(usize, usize), String> {
    migration_round_trip_v1(derived)?;
    let migration_root = intent_migration_agent_root_v1(root);
    let mut written = 0;
    let mut duplicates = 0;
    let intent_bytes =
        encode_canonical_learning_intent_migration_protobuf_v1(&derived.canonical_intent)?;
    let status = atomic_write_verified_v0(
        &migration_root.join("canonical-intent.pb"),
        &intent_bytes,
        &derived.canonical_intent.intent_digest,
        |bytes| Ok(decode_canonical_learning_intent_migration_protobuf_v1(bytes)?.intent_digest),
    )?;
    record_migration_write_status_v1(status, &mut written, &mut duplicates);
    let view_root = migration_root.join("canonical-view");
    let view_path = view_root.join(format!(
        "agent-view-{}.pb",
        derived.canonical_view.view_digest
    ));
    let view_existed = view_path.is_file();
    let stored_view_path =
        write_and_verify_agent_learning_data_view_v0(&derived.canonical_view, &view_root)?;
    if stored_view_path != view_path
        || read_and_verify_agent_learning_data_view_v0(&stored_view_path)? != derived.canonical_view
    {
        return Err("canonical migrated view reopen rejected".to_string());
    }
    if view_existed {
        duplicates += 1;
    } else {
        written += 1;
    }
    let policy_bytes = encode_intent_policy_compatibility_proof_protobuf_v1(&derived.policy_proof)?;
    let status = atomic_write_verified_v0(
        &migration_root.join("policy-compatibility-proof.pb"),
        &policy_bytes,
        &derived.policy_proof.proof_digest,
        |bytes| Ok(decode_intent_policy_compatibility_proof_protobuf_v1(bytes)?.proof_digest),
    )?;
    record_migration_write_status_v1(status, &mut written, &mut duplicates);
    let proof_bytes = encode_learning_intent_migration_proof_protobuf_v1(&derived.migration_proof)?;
    let status = atomic_write_verified_v0(
        &migration_root.join("migration-proof.pb"),
        &proof_bytes,
        &derived.migration_proof.proof_digest,
        |bytes| Ok(decode_learning_intent_migration_proof_protobuf_v1(bytes)?.proof_digest),
    )?;
    record_migration_write_status_v1(status, &mut written, &mut duplicates);
    let journal_bytes = encode_learning_intent_migration_journal_protobuf_v1(&derived.journal)?;
    let status = atomic_write_verified_v0(
        &migration_root.join("journal.pb"),
        &journal_bytes,
        &derived.journal.journal_digest,
        |bytes| Ok(decode_learning_intent_migration_journal_protobuf_v1(bytes)?.journal_digest),
    )?;
    record_migration_write_status_v1(status, &mut written, &mut duplicates);
    Ok((written, duplicates))
}

pub fn read_persisted_learning_intent_migration_v1(
    root: &Path,
    snapshots: &[DataSnapshot],
) -> Result<AgentPrivateLearningInputV1, String> {
    let migration_root = intent_migration_agent_root_v1(root);
    let intent = decode_canonical_learning_intent_migration_protobuf_v1(
        &fs::read(migration_root.join("canonical-intent.pb"))
            .map_err(|_| "canonical migrated intent unavailable".to_string())?,
    )?;
    let policy_proof = decode_intent_policy_compatibility_proof_protobuf_v1(
        &fs::read(migration_root.join("policy-compatibility-proof.pb"))
            .map_err(|_| "intent policy compatibility proof unavailable".to_string())?,
    )?;
    let migration_proof = decode_learning_intent_migration_proof_protobuf_v1(
        &fs::read(migration_root.join("migration-proof.pb"))
            .map_err(|_| "intent migration proof unavailable".to_string())?,
    )?;
    let journal = decode_learning_intent_migration_journal_protobuf_v1(
        &fs::read(migration_root.join("journal.pb"))
            .map_err(|_| "intent migration journal unavailable".to_string())?,
    )?;
    let view_path = migration_root.join("canonical-view").join(format!(
        "agent-view-{}.pb",
        migration_proof.canonical_view_digest
    ));
    let view = read_and_verify_agent_learning_data_view_v0(&view_path)?;
    if intent.agent_id != MOMENTUM_AGENT_ID_V1
        || intent.intent_digest != migration_proof.canonical_intent_digest
        || view.agent_id != intent.agent_id
        || view.view_digest != migration_proof.canonical_view_digest
        || policy_proof.agent_id != intent.agent_id
        || policy_proof.proof_digest != migration_proof.policy_compatibility_proof_digest
        || journal.agent_id != intent.agent_id
        || journal.migration_proof_digest != migration_proof.proof_digest
        || journal.canonical_intent_digest != intent.intent_digest
        || journal.canonical_view_digest != view.view_digest
    {
        return Err("persisted intent migration cross-artifact binding rejected".to_string());
    }
    let policy = default_agent_data_policies()
        .into_iter()
        .find(|policy| policy.agent_kind == intent.agent_kind)
        .ok_or_else(|| "canonical migrated intent policy unavailable".to_string())?;
    validate_agent_learning_intent_v0(&intent, &policy)?;
    validate_agent_learning_data_view_v0(&view)?;
    let snapshot = snapshots
        .iter()
        .find(|snapshot| snapshot.content_digest == migration_proof.merged_snapshot_digest)
        .ok_or_else(|| "canonical migrated view snapshot unavailable".to_string())?;
    if view.source_artifact_digests.len() != 1
        || view.source_artifact_digests.first() != Some(&snapshot.content_digest)
        || historical_replay_dataset_digest_v0(&snapshot.normalized_dataset)
            != snapshot.content_digest
    {
        return Err("canonical migrated view snapshot binding rejected".to_string());
    }
    let input = build_agent_private_learning_input_from_persisted_view_v0(
        &intent,
        &policy,
        &view,
        std::slice::from_ref(snapshot),
    )?;
    if input.resolution_status != AgentViewResolutionStatusV0::OptionalEvidenceUnavailable
        || input.view.decision_gate != EvidenceDecisionGate::Ready
    {
        return Err("canonical migrated view readiness rejected".to_string());
    }
    Ok(AgentPrivateLearningInputV1 {
        input,
        persisted_view_verified: true,
    })
}

fn atomic_write_verified_v0<F>(
    path: &Path,
    bytes: &[u8],
    expected_digest: &str,
    verify: F,
) -> Result<AgentPrivateLearningArtifactWriteStatusV0, String>
where
    F: Fn(&[u8]) -> Result<String, String>,
{
    if path.file_name().is_none() || expected_digest.is_empty() {
        return Err("private learning artifact path rejected".to_string());
    }
    if path.exists() {
        let existing =
            fs::read(path).map_err(|_| "private learning artifact reopen failed".to_string())?;
        if verify(&existing)? == expected_digest {
            return Ok(AgentPrivateLearningArtifactWriteStatusV0::DuplicateRejected);
        }
        return Err("private learning artifact identity collision".to_string());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "private learning artifact parent missing".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|_| "private learning artifact directory unavailable".to_string())?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "private learning artifact filename rejected".to_string())?;
    let temporary = parent.join(format!(".{file_name}.{expected_digest}.tmp"));
    let write_result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| "private learning temporary create failed".to_string())?;
        file.write_all(bytes)
            .map_err(|_| "private learning temporary write failed".to_string())?;
        file.flush()
            .map_err(|_| "private learning temporary flush failed".to_string())?;
        file.sync_all()
            .map_err(|_| "private learning temporary sync failed".to_string())?;
        drop(file);
        let mut reopened = Vec::new();
        File::open(&temporary)
            .map_err(|_| "private learning temporary reopen failed".to_string())?
            .read_to_end(&mut reopened)
            .map_err(|_| "private learning temporary reread failed".to_string())?;
        if verify(&reopened)? != expected_digest {
            return Err("private learning temporary verification failed".to_string());
        }
        fs::rename(&temporary, path)
            .map_err(|error| format!("private learning atomic rename failed: {error}"))?;
        let stored =
            fs::read(path).map_err(|_| "private learning final reopen failed".to_string())?;
        if verify(&stored)? != expected_digest {
            return Err("private learning final verification failed".to_string());
        }
        Ok(())
    })();
    if write_result.is_err() && temporary.is_file() {
        let _ = fs::remove_file(&temporary);
    }
    write_result?;
    Ok(AgentPrivateLearningArtifactWriteStatusV0::Written)
}

fn safe_agent_component_v0(agent_id: &str) -> bool {
    matches!(
        agent_id,
        "momentum_trend_fast" | "value_quality_filter" | "cycle_risk_skeptic"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArtifactKindV0 {
    Session,
    Projection,
    Dataset,
    Candidate,
    Journal,
    Registry,
    EvidenceUsageLedger,
    CandidateIdentityAudit,
    EvaluationRegistration,
    EvaluationJournal,
    SessionV1,
    ProjectionV1,
    CandidateFamilyV1,
    ParticipantV1,
    QualificationReceiptV1,
    UsageLedgerV1,
    EvidenceExclusionV1,
    EvaluationRegistrationV1,
    EvaluationJournalV1,
    CanonicalIntentMigrationV1,
    IntentPolicyCompatibilityProofV1,
    IntentMigrationProofV1,
    IntentMigrationJournalV1,
}

impl ArtifactKindV0 {
    fn wire_name(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Projection => "projection",
            Self::Dataset => "dataset",
            Self::Candidate => "candidate",
            Self::Journal => "journal",
            Self::Registry => "registry",
            Self::EvidenceUsageLedger => "evidence-usage-ledger",
            Self::CandidateIdentityAudit => "candidate-identity-audit",
            Self::EvaluationRegistration => "candidate-evaluation-registration",
            Self::EvaluationJournal => "candidate-evaluation-journal",
            Self::SessionV1 => "session-v1",
            Self::ProjectionV1 => "projection-v1",
            Self::CandidateFamilyV1 => "candidate-family-v1",
            Self::ParticipantV1 => "candidate-participant-v1",
            Self::QualificationReceiptV1 => "validation-qualification-v1",
            Self::UsageLedgerV1 => "candidate-usage-ledger-v1",
            Self::EvidenceExclusionV1 => "evaluation-evidence-exclusion-v1",
            Self::EvaluationRegistrationV1 => "candidate-evaluation-registration-v1",
            Self::EvaluationJournalV1 => "candidate-evaluation-journal-v1",
            Self::CanonicalIntentMigrationV1 => "canonical-learning-intent-migration-v1",
            Self::IntentPolicyCompatibilityProofV1 => "intent-policy-compatibility-proof-v1",
            Self::IntentMigrationProofV1 => "persisted-learning-intent-migration-proof-v1",
            Self::IntentMigrationJournalV1 => "persisted-learning-intent-migration-journal-v1",
        }
    }
}

#[derive(Clone, PartialEq, Message)]
struct ArtifactEnvelopeProtobufV0 {
    #[prost(bytes = "vec", tag = "1")]
    magic: Vec<u8>,
    #[prost(uint32, tag = "2")]
    version: u32,
    #[prost(string, tag = "3")]
    schema: String,
    #[prost(string, tag = "4")]
    artifact_kind: String,
    #[prost(string, tag = "5")]
    semantic_digest: String,
    #[prost(uint64, tag = "6")]
    payload_length: u64,
    #[prost(string, tag = "7")]
    payload_digest: String,
    #[prost(bytes = "vec", tag = "8")]
    payload: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
struct RangeProtobufV0 {
    #[prost(uint64, tag = "1")]
    start: u64,
    #[prost(uint64, tag = "2")]
    end: u64,
}

#[derive(Clone, PartialEq, Message)]
struct SessionProtobufV0 {
    #[prost(string, tag = "1")]
    session_version: String,
    #[prost(string, tag = "2")]
    session_id: String,
    #[prost(string, tag = "3")]
    agent_id: String,
    #[prost(uint32, tag = "4")]
    agent_kind: u32,
    #[prost(string, tag = "5")]
    intent_digest: String,
    #[prost(string, tag = "6")]
    data_view_digest: String,
    #[prost(string, tag = "7")]
    trainer_capability_digest: String,
    #[prost(uint64, tag = "8")]
    information_cutoff_ms: u64,
    #[prost(string, repeated, tag = "9")]
    source_artifact_digests: Vec<String>,
    #[prost(string, tag = "10")]
    feature_policy_digest: String,
    #[prost(string, tag = "11")]
    label_policy_digest: String,
    #[prost(string, tag = "12")]
    curriculum_policy_digest: String,
    #[prost(string, tag = "13")]
    private_namespace_digest: String,
    #[prost(string, optional, tag = "14")]
    parent_model_version: Option<String>,
    #[prost(uint32, tag = "15")]
    session_status: u32,
    #[prost(string, tag = "16")]
    session_digest: String,
    #[prost(uint32, repeated, tag = "17")]
    required_dataset_kinds: Vec<u32>,
    #[prost(uint32, repeated, tag = "18")]
    optional_dataset_kinds: Vec<u32>,
    #[prost(uint32, repeated, tag = "19")]
    allowed_markets: Vec<u32>,
    #[prost(string, repeated, tag = "20")]
    symbols: Vec<String>,
    #[prost(string, tag = "21")]
    cadence: String,
    #[prost(uint64, tag = "22")]
    lookback_bars: u64,
    #[prost(uint64, optional, tag = "23")]
    lookback_start_timestamp_ms: Option<u64>,
    #[prost(uint64, optional, tag = "24")]
    lookback_end_timestamp_ms: Option<u64>,
    #[prost(uint64, tag = "25")]
    maximum_staleness_ms: u64,
    #[prost(string, tag = "26")]
    source_policy_digest: String,
    #[prost(string, tag = "27")]
    training_ledger_digest: String,
    #[prost(string, optional, tag = "28")]
    trainer_projection_digest: Option<String>,
}

#[derive(Clone, PartialEq, Message)]
struct CanonicalIntentMigrationProtobufV1 {
    #[prost(string, tag = "1")]
    intent_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(uint32, tag = "3")]
    agent_kind: u32,
    #[prost(uint32, repeated, tag = "4")]
    market_scopes: Vec<u32>,
    #[prost(string, repeated, tag = "5")]
    symbols: Vec<String>,
    #[prost(uint32, repeated, tag = "6")]
    required_datasets: Vec<u32>,
    #[prost(uint32, repeated, tag = "7")]
    optional_datasets: Vec<u32>,
    #[prost(string, tag = "8")]
    cadence: String,
    #[prost(uint64, tag = "9")]
    lookback_bars: u64,
    #[prost(uint64, optional, tag = "10")]
    lookback_start_timestamp_ms: Option<u64>,
    #[prost(uint64, optional, tag = "11")]
    lookback_end_timestamp_ms: Option<u64>,
    #[prost(uint64, tag = "12")]
    information_cutoff_ms: u64,
    #[prost(uint64, tag = "13")]
    maximum_staleness_ms: u64,
    #[prost(string, tag = "14")]
    source_policy_digest: String,
    #[prost(string, tag = "15")]
    feature_policy_digest: String,
    #[prost(string, tag = "16")]
    label_policy_digest: String,
    #[prost(string, tag = "17")]
    curriculum_policy_digest: String,
    #[prost(string, tag = "18")]
    intent_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct IntentPolicyCompatibilityProofProtobufV1 {
    #[prost(string, tag = "1")]
    proof_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    legacy_policy_digest: String,
    #[prost(string, tag = "4")]
    current_policy_digest: String,
    #[prost(bool, tag = "5")]
    required_datasets_equal: bool,
    #[prost(bool, tag = "6")]
    optional_datasets_equal: bool,
    #[prost(bool, tag = "7")]
    markets_equal: bool,
    #[prost(bool, tag = "8")]
    cadence_equal: bool,
    #[prost(bool, tag = "9")]
    lookback_equal: bool,
    #[prost(bool, tag = "10")]
    staleness_equal: bool,
    #[prost(bool, tag = "11")]
    compatible: bool,
    #[prost(string, tag = "12")]
    proof_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct IntentMigrationProofProtobufV1 {
    #[prost(string, tag = "1")]
    proof_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    legacy_session_digest: String,
    #[prost(string, tag = "4")]
    legacy_intent_digest: String,
    #[prost(string, tag = "5")]
    canonical_gap_digest: String,
    #[prost(string, tag = "6")]
    composite_registration_digest: String,
    #[prost(string, tag = "7")]
    canonical_snapshot_digest: String,
    #[prost(string, tag = "8")]
    policy_compatibility_proof_digest: String,
    #[prost(string, repeated, tag = "9")]
    field_provenance_digests: Vec<String>,
    #[prost(string, tag = "10")]
    canonical_intent_digest: String,
    #[prost(string, tag = "11")]
    canonical_view_digest: String,
    #[prost(bool, tag = "12")]
    cutoff_unchanged: bool,
    #[prost(bool, tag = "13")]
    lookback_unchanged: bool,
    #[prost(bool, tag = "14")]
    policy_unchanged: bool,
    #[prost(bool, tag = "15")]
    required_evidence_unchanged: bool,
    #[prost(bool, tag = "16")]
    optional_evidence_unchanged: bool,
    #[prost(bool, tag = "17")]
    exclusions_unchanged: bool,
    #[prost(bool, tag = "18")]
    no_field_invented: bool,
    #[prost(uint32, tag = "19")]
    status: u32,
    #[prost(string, tag = "20")]
    proof_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct IntentMigrationJournalProtobufV1 {
    #[prost(string, tag = "1")]
    journal_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    migration_proof_digest: String,
    #[prost(string, tag = "4")]
    canonical_intent_digest: String,
    #[prost(string, tag = "5")]
    canonical_view_digest: String,
    #[prost(uint64, tag = "6")]
    entry_count: u64,
    #[prost(uint64, tag = "7")]
    network_requests: u64,
    #[prost(uint64, tag = "8")]
    transport_constructions: u64,
    #[prost(uint64, tag = "9")]
    credential_reads: u64,
    #[prost(uint64, tag = "10")]
    prospective_reads: u64,
    #[prost(uint64, tag = "11")]
    active_model_changes: u64,
    #[prost(uint32, tag = "12")]
    status: u32,
    #[prost(string, tag = "13")]
    journal_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct TrainerProjectionProtobufV0 {
    #[prost(string, tag = "1")]
    projection_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(uint32, tag = "3")]
    trainer_kind: u32,
    #[prost(string, tag = "4")]
    source_view_digest: String,
    #[prost(string, repeated, tag = "5")]
    consumed_artifact_digests: Vec<String>,
    #[prost(string, repeated, tag = "6")]
    referenced_but_unconsumed_artifact_digests: Vec<String>,
    #[prost(string, optional, tag = "7")]
    primary_series_digest: Option<String>,
    #[prost(string, repeated, tag = "8")]
    auxiliary_series_digests: Vec<String>,
    #[prost(string, tag = "9")]
    projection_policy_digest: String,
    #[prost(string, tag = "10")]
    projection_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct EvidenceUsageEntryProtobufV0 {
    #[prost(string, tag = "1")]
    artifact_digest: String,
    #[prost(message, optional, tag = "2")]
    range: Option<RangeProtobufV0>,
    #[prost(uint32, tag = "3")]
    use_kind: u32,
    #[prost(bool, tag = "4")]
    labels_read: bool,
    #[prost(bool, tag = "5")]
    parameters_updated: bool,
    #[prost(bool, tag = "6")]
    checkpoint_selection_influenced: bool,
    #[prost(bool, tag = "7")]
    candidate_identity_influenced: bool,
    #[prost(string, tag = "8")]
    entry_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct EvidenceUsageLedgerProtobufV0 {
    #[prost(string, tag = "1")]
    ledger_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    candidate_digest: String,
    #[prost(string, tag = "4")]
    session_digest: String,
    #[prost(message, repeated, tag = "5")]
    entries: Vec<EvidenceUsageEntryProtobufV0>,
    #[prost(string, tag = "6")]
    ledger_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct CandidateIdentityAuditProtobufV0 {
    #[prost(string, tag = "1")]
    audit_version: String,
    #[prost(string, tag = "2")]
    candidate_digest: String,
    #[prost(string, repeated, tag = "3")]
    model_identity_inputs: Vec<String>,
    #[prost(string, repeated, tag = "4")]
    metric_identity_inputs: Vec<String>,
    #[prost(bool, tag = "5")]
    test_evidence_in_identity: bool,
    #[prost(uint32, tag = "6")]
    historical_test_status: u32,
    #[prost(bool, tag = "7")]
    eligible_for_fresh_historical_test: bool,
    #[prost(bool, tag = "8")]
    eligible_for_future_evaluation_registration: bool,
    #[prost(bool, tag = "9")]
    superseded_by_input_binding_hardening: bool,
    #[prost(string, tag = "10")]
    audit_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct CandidateEvaluationRegistrationProtobufV0 {
    #[prost(string, tag = "1")]
    registration_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    candidate_digest: String,
    #[prost(string, tag = "4")]
    session_digest: String,
    #[prost(string, tag = "5")]
    evidence_usage_ledger_digest: String,
    #[prost(string, tag = "6")]
    identity_audit_digest: String,
    #[prost(uint64, tag = "7")]
    evaluation_cutoff_exclusive_ms: u64,
    #[prost(uint32, repeated, tag = "8")]
    required_dataset_kinds: Vec<u32>,
    #[prost(string, tag = "9")]
    source_policy_digest: String,
    #[prost(string, tag = "10")]
    finality_policy_digest: String,
    #[prost(string, tag = "11")]
    label_policy_digest: String,
    #[prost(string, tag = "12")]
    metric_policy_digest: String,
    #[prost(string, tag = "13")]
    support_policy_digest: String,
    #[prost(string, repeated, tag = "14")]
    comparator_digests: Vec<String>,
    #[prost(uint64, tag = "15")]
    minimum_future_rows: u64,
    #[prost(uint64, tag = "16")]
    minimum_mature_events: u64,
    #[prost(uint64, tag = "17")]
    maximum_requests: u64,
    #[prost(uint64, tag = "18")]
    maximum_concurrency: u64,
    #[prost(uint64, tag = "19")]
    maximum_retries: u64,
    #[prost(bool, tag = "20")]
    labels_hidden_until_opening: bool,
    #[prost(bool, tag = "21")]
    probabilities_hidden_until_opening: bool,
    #[prost(bool, tag = "22")]
    one_time_opening_required: bool,
    #[prost(bool, tag = "23")]
    active_promotion_forbidden: bool,
    #[prost(bool, tag = "24")]
    reward_application_forbidden: bool,
    #[prost(uint32, tag = "25")]
    status: u32,
    #[prost(string, tag = "26")]
    registration_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct CandidateEvaluationJournalEntryProtobufV0 {
    #[prost(string, tag = "1")]
    registration_digest: String,
    #[prost(uint32, tag = "2")]
    status: u32,
}

#[derive(Clone, PartialEq, Message)]
struct CandidateEvaluationJournalProtobufV0 {
    #[prost(string, tag = "1")]
    journal_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    candidate_digest: String,
    #[prost(message, repeated, tag = "4")]
    entries: Vec<CandidateEvaluationJournalEntryProtobufV0>,
    #[prost(string, tag = "5")]
    journal_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct DatasetManifestProtobufV0 {
    #[prost(string, tag = "1")]
    dataset_version: String,
    #[prost(string, tag = "2")]
    dataset_id: String,
    #[prost(string, tag = "3")]
    agent_id: String,
    #[prost(string, tag = "4")]
    session_id: String,
    #[prost(string, tag = "5")]
    data_view_digest: String,
    #[prost(string, repeated, tag = "6")]
    source_artifact_digests: Vec<String>,
    #[prost(uint32, repeated, tag = "7")]
    dataset_kinds: Vec<u32>,
    #[prost(uint64, tag = "8")]
    information_cutoff_ms: u64,
    #[prost(uint64, tag = "9")]
    row_count: u64,
    #[prost(message, optional, tag = "10")]
    training_range: Option<RangeProtobufV0>,
    #[prost(message, optional, tag = "11")]
    first_purge_range: Option<RangeProtobufV0>,
    #[prost(message, optional, tag = "12")]
    validation_range: Option<RangeProtobufV0>,
    #[prost(message, optional, tag = "13")]
    second_purge_range: Option<RangeProtobufV0>,
    #[prost(message, optional, tag = "14")]
    sealed_test_range: Option<RangeProtobufV0>,
    #[prost(message, optional, tag = "15")]
    normalizer_fit_range: Option<RangeProtobufV0>,
    #[prost(uint64, tag = "16")]
    validation_parameter_update_count: u64,
    #[prost(uint64, tag = "17")]
    test_checkpoint_selection_count: u64,
    #[prost(uint64, tag = "18")]
    prospective_row_read_count: u64,
    #[prost(uint64, tag = "19")]
    prospective_label_read_count: u64,
    #[prost(string, tag = "20")]
    feature_artifact_digest: String,
    #[prost(string, tag = "21")]
    label_artifact_digest: String,
    #[prost(string, tag = "22")]
    normalizer_digest: String,
    #[prost(string, tag = "23")]
    manifest_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct CandidateProtobufV0 {
    #[prost(string, tag = "1")]
    candidate_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    session_digest: String,
    #[prost(string, tag = "4")]
    data_view_digest: String,
    #[prost(string, optional, tag = "5")]
    parent_model_version: Option<String>,
    #[prost(string, tag = "6")]
    model_artifact_digest: String,
    #[prost(string, tag = "7")]
    feature_policy_digest: String,
    #[prost(string, tag = "8")]
    label_policy_digest: String,
    #[prost(string, tag = "9")]
    normalizer_digest: String,
    #[prost(string, tag = "10")]
    training_policy_digest: String,
    #[prost(string, tag = "11")]
    private_metrics_digest: String,
    #[prost(uint32, tag = "12")]
    deployment_status: u32,
    #[prost(bool, tag = "13")]
    retrospective_research_only: bool,
    #[prost(bool, tag = "14")]
    eligible_for_active_committee: bool,
    #[prost(bool, tag = "15")]
    eligible_for_promotion: bool,
    #[prost(bool, tag = "16")]
    eligible_for_reward: bool,
    #[prost(string, tag = "17")]
    candidate_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct JournalEntryProtobufV0 {
    #[prost(string, tag = "1")]
    session_digest: String,
    #[prost(uint32, tag = "2")]
    session_status: u32,
    #[prost(string, optional, tag = "3")]
    dataset_manifest_digest: Option<String>,
    #[prost(string, optional, tag = "4")]
    candidate_digest: Option<String>,
}

#[derive(Clone, PartialEq, Message)]
struct JournalProtobufV0 {
    #[prost(string, tag = "1")]
    journal_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(message, repeated, tag = "3")]
    entries: Vec<JournalEntryProtobufV0>,
    #[prost(string, tag = "4")]
    journal_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct CapabilityProtobufV0 {
    #[prost(string, tag = "1")]
    agent_id: String,
    #[prost(uint32, tag = "2")]
    trainer_kind: u32,
    #[prost(uint32, repeated, tag = "3")]
    supported_dataset_kinds: Vec<u32>,
    #[prost(bool, tag = "4")]
    supports_training: bool,
    #[prost(bool, tag = "5")]
    shadow_only: bool,
    #[prost(string, tag = "6")]
    capability_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct CapabilityRegistryProtobufV0 {
    #[prost(string, tag = "1")]
    registry_version: String,
    #[prost(message, repeated, tag = "2")]
    capabilities: Vec<CapabilityProtobufV0>,
    #[prost(string, tag = "3")]
    registry_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct SessionProtobufV1 {
    #[prost(string, tag = "1")]
    session_version: String,
    #[prost(string, tag = "2")]
    session_id: String,
    #[prost(string, tag = "3")]
    agent_id: String,
    #[prost(uint32, tag = "4")]
    agent_kind: u32,
    #[prost(string, tag = "5")]
    intent_digest: String,
    #[prost(string, tag = "6")]
    view_digest: String,
    #[prost(string, tag = "7")]
    projection_digest: String,
    #[prost(string, tag = "8")]
    capability_digest: String,
    #[prost(string, tag = "9")]
    source_policy_digest: String,
    #[prost(string, tag = "10")]
    feature_policy_digest: String,
    #[prost(string, tag = "11")]
    label_policy_digest: String,
    #[prost(string, tag = "12")]
    curriculum_policy_digest: String,
    #[prost(uint64, tag = "13")]
    information_cutoff_ms: u64,
    #[prost(string, repeated, tag = "14")]
    source_artifact_digests: Vec<String>,
    #[prost(string, repeated, tag = "15")]
    consumed_artifact_digests: Vec<String>,
    #[prost(string, repeated, tag = "16")]
    referenced_unconsumed_artifact_digests: Vec<String>,
    #[prost(string, tag = "17")]
    private_namespace_digest: String,
    #[prost(string, tag = "18")]
    training_ledger_digest: String,
    #[prost(bool, tag = "19")]
    fresh_initialization: bool,
    #[prost(bool, tag = "20")]
    historical_test_access_forbidden: bool,
    #[prost(uint32, tag = "21")]
    status: u32,
    #[prost(string, tag = "22")]
    session_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct ProjectionProtobufV1 {
    #[prost(string, tag = "1")]
    projection_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(uint32, tag = "3")]
    trainer_kind: u32,
    #[prost(string, tag = "4")]
    source_view_digest: String,
    #[prost(string, repeated, tag = "5")]
    consumed_artifact_digests: Vec<String>,
    #[prost(string, repeated, tag = "6")]
    referenced_unconsumed_artifact_digests: Vec<String>,
    #[prost(string, optional, tag = "7")]
    primary_series_digest: Option<String>,
    #[prost(string, tag = "8")]
    projection_policy_digest: String,
    #[prost(string, tag = "9")]
    projection_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct ParticipantProtobufV1 {
    #[prost(string, tag = "1")]
    participant_version: String,
    #[prost(string, tag = "2")]
    participant_id: String,
    #[prost(uint32, tag = "3")]
    role: u32,
    #[prost(string, tag = "4")]
    model_kind: String,
    #[prost(string, tag = "5")]
    model_artifact_digest: String,
    #[prost(string, tag = "6")]
    parameter_digest: String,
    #[prost(string, tag = "7")]
    normalizer_digest: String,
    #[prost(string, tag = "8")]
    feature_policy_digest: String,
    #[prost(string, tag = "9")]
    label_policy_digest: String,
    #[prost(string, tag = "10")]
    training_policy_digest: String,
    #[prost(string, tag = "11")]
    initialization_digest: String,
    #[prost(uint32, tag = "12")]
    deployment_status: u32,
    #[prost(string, tag = "13")]
    participant_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct QualificationReceiptProtobufV1 {
    #[prost(string, tag = "1")]
    qualification_version: String,
    #[prost(string, tag = "2")]
    participant_digest: String,
    #[prost(string, tag = "3")]
    validation_range_digest: String,
    #[prost(string, tag = "4")]
    metric_policy_digest: String,
    #[prost(string, tag = "5")]
    private_metric_digest: String,
    #[prost(uint32, tag = "6")]
    qualification_status: u32,
    #[prost(uint64, tag = "7")]
    parameter_updates_during_validation: u64,
    #[prost(string, tag = "8")]
    receipt_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct CandidateFamilyProtobufV1 {
    #[prost(string, tag = "1")]
    family_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    session_digest: String,
    #[prost(string, tag = "4")]
    view_digest: String,
    #[prost(string, tag = "5")]
    projection_digest: String,
    #[prost(message, repeated, tag = "6")]
    participants: Vec<ParticipantProtobufV1>,
    #[prost(string, repeated, tag = "7")]
    validation_qualification_receipts: Vec<String>,
    #[prost(bool, tag = "8")]
    winner_selected: bool,
    #[prost(bool, tag = "9")]
    historical_test_accessed: bool,
    #[prost(bool, tag = "10")]
    eligible_for_active_committee: bool,
    #[prost(bool, tag = "11")]
    eligible_for_promotion: bool,
    #[prost(bool, tag = "12")]
    eligible_for_reward: bool,
    #[prost(string, tag = "13")]
    family_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct UsageEntryProtobufV1 {
    #[prost(string, tag = "1")]
    artifact_digest: String,
    #[prost(message, optional, tag = "2")]
    range: Option<RangeProtobufV0>,
    #[prost(uint32, tag = "3")]
    use_kind: u32,
    #[prost(bool, tag = "4")]
    labels_read: bool,
    #[prost(bool, tag = "5")]
    parameters_updated: bool,
    #[prost(string, tag = "6")]
    entry_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct UsageLedgerProtobufV1 {
    #[prost(string, tag = "1")]
    ledger_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    session_digest: String,
    #[prost(string, tag = "4")]
    family_digest: String,
    #[prost(message, repeated, tag = "5")]
    entries: Vec<UsageEntryProtobufV1>,
    #[prost(uint64, tag = "6")]
    historical_test_row_reads: u64,
    #[prost(uint64, tag = "7")]
    historical_test_label_reads: u64,
    #[prost(uint64, tag = "8")]
    historical_test_inference_count: u64,
    #[prost(uint64, tag = "9")]
    historical_test_metric_count: u64,
    #[prost(uint64, tag = "10")]
    historical_test_checkpoint_selection_count: u64,
    #[prost(bool, tag = "11")]
    historical_test_identity_influence: bool,
    #[prost(string, tag = "12")]
    ledger_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct EvidenceExclusionProtobufV1 {
    #[prost(string, tag = "1")]
    exclusion_version: String,
    #[prost(string, repeated, tag = "2")]
    protected_registration_digests: Vec<String>,
    #[prost(uint64, repeated, tag = "3")]
    excluded_timestamp_ms: Vec<u64>,
    #[prost(string, repeated, tag = "4")]
    excluded_range_digests: Vec<String>,
    #[prost(string, tag = "5")]
    exclusion_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct EvaluationRegistrationProtobufV1 {
    #[prost(string, tag = "1")]
    registration_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    family_digest: String,
    #[prost(string, tag = "4")]
    session_digest: String,
    #[prost(string, tag = "5")]
    usage_ledger_digest: String,
    #[prost(string, repeated, tag = "6")]
    participant_digests: Vec<String>,
    #[prost(string, repeated, tag = "7")]
    qualification_receipt_digests: Vec<String>,
    #[prost(string, tag = "8")]
    exclusion_digest: String,
    #[prost(uint64, tag = "9")]
    minimum_accepted_timestamp_ms: u64,
    #[prost(uint32, repeated, tag = "10")]
    required_dataset_kinds: Vec<u32>,
    #[prost(string, tag = "11")]
    source_policy_digest: String,
    #[prost(string, tag = "12")]
    finality_policy_digest: String,
    #[prost(string, tag = "13")]
    label_policy_digest: String,
    #[prost(string, tag = "14")]
    metric_policy_digest: String,
    #[prost(string, tag = "15")]
    support_policy_digest: String,
    #[prost(uint64, tag = "16")]
    minimum_future_rows: u64,
    #[prost(uint64, tag = "17")]
    minimum_mature_events: u64,
    #[prost(uint64, tag = "18")]
    maximum_requests: u64,
    #[prost(uint64, tag = "19")]
    maximum_concurrency: u64,
    #[prost(uint64, tag = "20")]
    maximum_retries: u64,
    #[prost(bool, tag = "21")]
    labels_hidden_until_opening: bool,
    #[prost(bool, tag = "22")]
    probabilities_hidden_until_opening: bool,
    #[prost(bool, tag = "23")]
    one_time_opening_required: bool,
    #[prost(bool, tag = "24")]
    winner_selection_forbidden_before_opening: bool,
    #[prost(bool, tag = "25")]
    active_promotion_forbidden: bool,
    #[prost(bool, tag = "26")]
    reward_application_forbidden: bool,
    #[prost(uint32, tag = "27")]
    status: u32,
    #[prost(string, tag = "28")]
    registration_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct EvaluationJournalEntryProtobufV1 {
    #[prost(string, tag = "1")]
    registration_digest: String,
    #[prost(uint32, tag = "2")]
    status: u32,
}

#[derive(Clone, PartialEq, Message)]
struct EvaluationJournalProtobufV1 {
    #[prost(string, tag = "1")]
    journal_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, tag = "3")]
    family_digest: String,
    #[prost(message, repeated, tag = "4")]
    entries: Vec<EvaluationJournalEntryProtobufV1>,
    #[prost(string, tag = "5")]
    journal_digest: String,
}

fn encode_envelope_v0<M: Message>(
    kind: ArtifactKindV0,
    semantic_digest: &str,
    payload: &M,
) -> Result<Vec<u8>, String> {
    let payload = payload.encode_to_vec();
    Ok(ArtifactEnvelopeProtobufV0 {
        magic: ARTIFACT_MAGIC_V0.to_vec(),
        version: 0,
        schema: ARTIFACT_SCHEMA_V0.to_string(),
        artifact_kind: kind.wire_name().to_string(),
        semantic_digest: semantic_digest.to_string(),
        payload_length: u64::try_from(payload.len())
            .map_err(|_| "private learning protobuf payload too large".to_string())?,
        payload_digest: crate::data::acquisition::canonical_hash_hex(&payload),
        payload,
    }
    .encode_to_vec())
}

fn decode_envelope_v0(
    bytes: &[u8],
    expected_kind: ArtifactKindV0,
) -> Result<ArtifactEnvelopeProtobufV0, String> {
    let envelope = ArtifactEnvelopeProtobufV0::decode(bytes)
        .map_err(|_| "private learning protobuf envelope decode failed".to_string())?;
    if envelope.magic != ARTIFACT_MAGIC_V0
        || envelope.version != 0
        || envelope.schema != ARTIFACT_SCHEMA_V0
        || envelope.artifact_kind != expected_kind.wire_name()
        || usize::try_from(envelope.payload_length).ok() != Some(envelope.payload.len())
        || envelope.payload_digest
            != crate::data::acquisition::canonical_hash_hex(&envelope.payload)
    {
        return Err("private learning protobuf envelope rejected".to_string());
    }
    Ok(envelope)
}

fn participant_to_protobuf_v1(participant: &FrozenCandidateParticipantV1) -> ParticipantProtobufV1 {
    ParticipantProtobufV1 {
        participant_version: PARTICIPANT_VERSION_V1.to_string(),
        participant_id: participant.participant_id.clone(),
        role: participant_role_tag_v1(participant.role),
        model_kind: participant.model_kind.clone(),
        model_artifact_digest: participant.model_artifact_digest.clone(),
        parameter_digest: participant.parameter_digest.clone(),
        normalizer_digest: participant.normalizer_digest.clone(),
        feature_policy_digest: participant.feature_policy_digest.clone(),
        label_policy_digest: participant.label_policy_digest.clone(),
        training_policy_digest: participant.training_policy_digest.clone(),
        initialization_digest: participant.initialization_digest.clone(),
        deployment_status: 1,
        participant_digest: participant.participant_digest.clone(),
    }
}

fn participant_from_protobuf_v1(
    value: ParticipantProtobufV1,
) -> Result<FrozenCandidateParticipantV1, String> {
    if value.participant_version != PARTICIPANT_VERSION_V1 || value.deployment_status != 1 {
        return Err("V1 participant protobuf rejected".to_string());
    }
    let participant = FrozenCandidateParticipantV1 {
        participant_id: value.participant_id,
        role: participant_role_from_tag_v1(value.role)?,
        model_kind: value.model_kind,
        model_artifact_digest: value.model_artifact_digest,
        parameter_digest: value.parameter_digest,
        normalizer_digest: value.normalizer_digest,
        feature_policy_digest: value.feature_policy_digest,
        label_policy_digest: value.label_policy_digest,
        training_policy_digest: value.training_policy_digest,
        initialization_digest: value.initialization_digest,
        deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
        participant_digest: value.participant_digest,
    };
    validate_participant_v1(&participant)?;
    Ok(participant)
}

pub fn encode_session_protobuf_v1(
    session: &AgentPrivateLearningSessionV1,
) -> Result<Vec<u8>, String> {
    validate_session_v1(session)?;
    encode_envelope_v0(
        ArtifactKindV0::SessionV1,
        &session.session_digest,
        &SessionProtobufV1 {
            session_version: session.session_version.clone(),
            session_id: session.session_id.clone(),
            agent_id: session.agent_id.clone(),
            agent_kind: agent_kind_tag_v0(session.agent_kind)?,
            intent_digest: session.intent_digest.clone(),
            view_digest: session.view_digest.clone(),
            projection_digest: session.projection_digest.clone(),
            capability_digest: session.capability_digest.clone(),
            source_policy_digest: session.source_policy_digest.clone(),
            feature_policy_digest: session.feature_policy_digest.clone(),
            label_policy_digest: session.label_policy_digest.clone(),
            curriculum_policy_digest: session.curriculum_policy_digest.clone(),
            information_cutoff_ms: session.information_cutoff_ms,
            source_artifact_digests: session.source_artifact_digests.clone(),
            consumed_artifact_digests: session.consumed_artifact_digests.clone(),
            referenced_unconsumed_artifact_digests: session
                .referenced_unconsumed_artifact_digests
                .clone(),
            private_namespace_digest: session.private_namespace_digest.clone(),
            training_ledger_digest: session.training_ledger_digest.clone(),
            fresh_initialization: session.fresh_initialization,
            historical_test_access_forbidden: session.historical_test_access_forbidden,
            status: session_status_tag_v1(session.status),
            session_digest: session.session_digest.clone(),
        },
    )
}

pub fn decode_session_protobuf_v1(bytes: &[u8]) -> Result<AgentPrivateLearningSessionV1, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::SessionV1)?;
    let value = SessionProtobufV1::decode(envelope.payload.as_slice())
        .map_err(|_| "V1 session protobuf decode failed".to_string())?;
    let session = AgentPrivateLearningSessionV1 {
        session_version: value.session_version,
        session_id: value.session_id,
        agent_id: value.agent_id,
        agent_kind: agent_kind_from_tag_v0(value.agent_kind)?,
        intent_digest: value.intent_digest,
        view_digest: value.view_digest,
        projection_digest: value.projection_digest,
        capability_digest: value.capability_digest,
        source_policy_digest: value.source_policy_digest,
        feature_policy_digest: value.feature_policy_digest,
        label_policy_digest: value.label_policy_digest,
        curriculum_policy_digest: value.curriculum_policy_digest,
        information_cutoff_ms: value.information_cutoff_ms,
        source_artifact_digests: value.source_artifact_digests,
        consumed_artifact_digests: value.consumed_artifact_digests,
        referenced_unconsumed_artifact_digests: value.referenced_unconsumed_artifact_digests,
        private_namespace_digest: value.private_namespace_digest,
        training_ledger_digest: value.training_ledger_digest,
        fresh_initialization: value.fresh_initialization,
        historical_test_access_forbidden: value.historical_test_access_forbidden,
        status: session_status_from_tag_v1(value.status)?,
        session_digest: value.session_digest,
    };
    validate_session_v1(&session)?;
    if session.session_digest != envelope.semantic_digest {
        return Err("V1 session envelope identity rejected".to_string());
    }
    Ok(session)
}

pub fn encode_trainer_projection_protobuf_v1(
    projection: &AgentTrainerInputProjectionV1,
) -> Result<Vec<u8>, String> {
    validate_projection_v1(projection)?;
    encode_envelope_v0(
        ArtifactKindV0::ProjectionV1,
        &projection.projection_digest,
        &ProjectionProtobufV1 {
            projection_version: projection.projection_version.clone(),
            agent_id: projection.agent_id.clone(),
            trainer_kind: trainer_kind_tag_v0(projection.trainer_kind),
            source_view_digest: projection.source_view_digest.clone(),
            consumed_artifact_digests: projection.consumed_artifact_digests.clone(),
            referenced_unconsumed_artifact_digests: projection
                .referenced_unconsumed_artifact_digests
                .clone(),
            primary_series_digest: projection.primary_series_digest.clone(),
            projection_policy_digest: projection.projection_policy_digest.clone(),
            projection_digest: projection.projection_digest.clone(),
        },
    )
}

pub fn decode_trainer_projection_protobuf_v1(
    bytes: &[u8],
) -> Result<AgentTrainerInputProjectionV1, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::ProjectionV1)?;
    let value = ProjectionProtobufV1::decode(envelope.payload.as_slice())
        .map_err(|_| "V1 projection protobuf decode failed".to_string())?;
    let projection = AgentTrainerInputProjectionV1 {
        projection_version: value.projection_version,
        agent_id: value.agent_id,
        trainer_kind: trainer_kind_from_tag_v0(value.trainer_kind)?,
        source_view_digest: value.source_view_digest,
        consumed_artifact_digests: value.consumed_artifact_digests,
        referenced_unconsumed_artifact_digests: value.referenced_unconsumed_artifact_digests,
        primary_series_digest: value.primary_series_digest,
        projection_policy_digest: value.projection_policy_digest,
        projection_digest: value.projection_digest,
    };
    validate_projection_v1(&projection)?;
    if projection.projection_digest != envelope.semantic_digest {
        return Err("V1 projection envelope identity rejected".to_string());
    }
    Ok(projection)
}

pub fn encode_participant_protobuf_v1(
    participant: &FrozenCandidateParticipantV1,
) -> Result<Vec<u8>, String> {
    validate_participant_v1(participant)?;
    encode_envelope_v0(
        ArtifactKindV0::ParticipantV1,
        &participant.participant_digest,
        &participant_to_protobuf_v1(participant),
    )
}

pub fn decode_participant_protobuf_v1(
    bytes: &[u8],
) -> Result<FrozenCandidateParticipantV1, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::ParticipantV1)?;
    let value = ParticipantProtobufV1::decode(envelope.payload.as_slice())
        .map_err(|_| "V1 participant protobuf decode failed".to_string())?;
    let participant = participant_from_protobuf_v1(value)?;
    if participant.participant_digest != envelope.semantic_digest {
        return Err("V1 participant envelope identity rejected".to_string());
    }
    Ok(participant)
}

pub fn encode_qualification_receipt_protobuf_v1(
    receipt: &ParticipantValidationQualificationV1,
) -> Result<Vec<u8>, String> {
    validate_qualification_receipt_v1(receipt)?;
    encode_envelope_v0(
        ArtifactKindV0::QualificationReceiptV1,
        &receipt.receipt_digest,
        &QualificationReceiptProtobufV1 {
            qualification_version: QUALIFICATION_VERSION_V1.to_string(),
            participant_digest: receipt.participant_digest.clone(),
            validation_range_digest: receipt.validation_range_digest.clone(),
            metric_policy_digest: receipt.metric_policy_digest.clone(),
            private_metric_digest: receipt.private_metric_digest.clone(),
            qualification_status: qualification_status_tag_v1(receipt.qualification_status),
            parameter_updates_during_validation: usize_to_u64_v0(
                receipt.parameter_updates_during_validation,
            )?,
            receipt_digest: receipt.receipt_digest.clone(),
        },
    )
}

pub fn decode_qualification_receipt_protobuf_v1(
    bytes: &[u8],
) -> Result<ParticipantValidationQualificationV1, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::QualificationReceiptV1)?;
    let value = QualificationReceiptProtobufV1::decode(envelope.payload.as_slice())
        .map_err(|_| "V1 qualification receipt protobuf decode failed".to_string())?;
    if value.qualification_version != QUALIFICATION_VERSION_V1 {
        return Err("V1 qualification receipt version rejected".to_string());
    }
    let receipt = ParticipantValidationQualificationV1 {
        participant_digest: value.participant_digest,
        validation_range_digest: value.validation_range_digest,
        metric_policy_digest: value.metric_policy_digest,
        private_metric_digest: value.private_metric_digest,
        qualification_status: qualification_status_from_tag_v1(value.qualification_status)?,
        parameter_updates_during_validation: u64_to_usize_v0(
            value.parameter_updates_during_validation,
        )?,
        receipt_digest: value.receipt_digest,
    };
    validate_qualification_receipt_v1(&receipt)?;
    if receipt.receipt_digest != envelope.semantic_digest {
        return Err("V1 qualification receipt envelope identity rejected".to_string());
    }
    Ok(receipt)
}

pub fn encode_candidate_family_protobuf_v1(
    family: &AgentCandidateFamilyV1,
) -> Result<Vec<u8>, String> {
    validate_candidate_family_v1(family)?;
    encode_envelope_v0(
        ArtifactKindV0::CandidateFamilyV1,
        &family.family_digest,
        &CandidateFamilyProtobufV1 {
            family_version: family.family_version.clone(),
            agent_id: family.agent_id.clone(),
            session_digest: family.session_digest.clone(),
            view_digest: family.view_digest.clone(),
            projection_digest: family.projection_digest.clone(),
            participants: family
                .participants
                .iter()
                .map(participant_to_protobuf_v1)
                .collect(),
            validation_qualification_receipts: family.validation_qualification_receipts.clone(),
            winner_selected: family.winner_selected,
            historical_test_accessed: family.historical_test_accessed,
            eligible_for_active_committee: family.eligible_for_active_committee,
            eligible_for_promotion: family.eligible_for_promotion,
            eligible_for_reward: family.eligible_for_reward,
            family_digest: family.family_digest.clone(),
        },
    )
}

pub fn decode_candidate_family_protobuf_v1(bytes: &[u8]) -> Result<AgentCandidateFamilyV1, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::CandidateFamilyV1)?;
    let value = CandidateFamilyProtobufV1::decode(envelope.payload.as_slice())
        .map_err(|_| "V1 family protobuf decode failed".to_string())?;
    let family = AgentCandidateFamilyV1 {
        family_version: value.family_version,
        agent_id: value.agent_id,
        session_digest: value.session_digest,
        view_digest: value.view_digest,
        projection_digest: value.projection_digest,
        participants: value
            .participants
            .into_iter()
            .map(participant_from_protobuf_v1)
            .collect::<Result<Vec<_>, _>>()?,
        validation_qualification_receipts: value.validation_qualification_receipts,
        winner_selected: value.winner_selected,
        historical_test_accessed: value.historical_test_accessed,
        eligible_for_active_committee: value.eligible_for_active_committee,
        eligible_for_promotion: value.eligible_for_promotion,
        eligible_for_reward: value.eligible_for_reward,
        family_digest: value.family_digest,
    };
    validate_candidate_family_v1(&family)?;
    if family.family_digest != envelope.semantic_digest {
        return Err("V1 family envelope identity rejected".to_string());
    }
    Ok(family)
}

pub fn encode_usage_ledger_protobuf_v1(
    ledger: &AgentCandidateUsageLedgerV1,
) -> Result<Vec<u8>, String> {
    validate_usage_ledger_v1(ledger)?;
    encode_envelope_v0(
        ArtifactKindV0::UsageLedgerV1,
        &ledger.ledger_digest,
        &UsageLedgerProtobufV1 {
            ledger_version: ledger.ledger_version.clone(),
            agent_id: ledger.agent_id.clone(),
            session_digest: ledger.session_digest.clone(),
            family_digest: ledger.family_digest.clone(),
            entries: ledger
                .entries
                .iter()
                .map(|entry| {
                    Ok(UsageEntryProtobufV1 {
                        artifact_digest: entry.artifact_digest.clone(),
                        range: entry
                            .range
                            .as_ref()
                            .map(|range| range_to_protobuf_v0(range))
                            .transpose()?,
                        use_kind: evidence_use_tag_v1(entry.use_kind),
                        labels_read: entry.labels_read,
                        parameters_updated: entry.parameters_updated,
                        entry_digest: entry.entry_digest.clone(),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
            historical_test_row_reads: usize_to_u64_v0(ledger.historical_test_row_reads)?,
            historical_test_label_reads: usize_to_u64_v0(ledger.historical_test_label_reads)?,
            historical_test_inference_count: usize_to_u64_v0(
                ledger.historical_test_inference_count,
            )?,
            historical_test_metric_count: usize_to_u64_v0(ledger.historical_test_metric_count)?,
            historical_test_checkpoint_selection_count: usize_to_u64_v0(
                ledger.historical_test_checkpoint_selection_count,
            )?,
            historical_test_identity_influence: ledger.historical_test_identity_influence,
            ledger_digest: ledger.ledger_digest.clone(),
        },
    )
}

pub fn decode_usage_ledger_protobuf_v1(
    bytes: &[u8],
) -> Result<AgentCandidateUsageLedgerV1, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::UsageLedgerV1)?;
    let value = UsageLedgerProtobufV1::decode(envelope.payload.as_slice())
        .map_err(|_| "V1 usage ledger protobuf decode failed".to_string())?;
    let ledger = AgentCandidateUsageLedgerV1 {
        ledger_version: value.ledger_version,
        agent_id: value.agent_id,
        session_digest: value.session_digest,
        family_digest: value.family_digest,
        entries: value
            .entries
            .into_iter()
            .map(|entry| {
                Ok(CandidateEvidenceUsageEntryV1 {
                    artifact_digest: entry.artifact_digest,
                    range: entry
                        .range
                        .map(|range| range_from_protobuf_v0(Some(range)))
                        .transpose()?,
                    use_kind: evidence_use_from_tag_v1(entry.use_kind)?,
                    labels_read: entry.labels_read,
                    parameters_updated: entry.parameters_updated,
                    entry_digest: entry.entry_digest,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        historical_test_row_reads: u64_to_usize_v0(value.historical_test_row_reads)?,
        historical_test_label_reads: u64_to_usize_v0(value.historical_test_label_reads)?,
        historical_test_inference_count: u64_to_usize_v0(value.historical_test_inference_count)?,
        historical_test_metric_count: u64_to_usize_v0(value.historical_test_metric_count)?,
        historical_test_checkpoint_selection_count: u64_to_usize_v0(
            value.historical_test_checkpoint_selection_count,
        )?,
        historical_test_identity_influence: value.historical_test_identity_influence,
        ledger_digest: value.ledger_digest,
    };
    validate_usage_ledger_v1(&ledger)?;
    if ledger.ledger_digest != envelope.semantic_digest {
        return Err("V1 usage ledger envelope identity rejected".to_string());
    }
    Ok(ledger)
}

pub fn encode_evidence_exclusion_protobuf_v1(
    exclusion: &EvaluationEvidenceExclusionV1,
) -> Result<Vec<u8>, String> {
    validate_evaluation_exclusion_v1(exclusion)?;
    encode_envelope_v0(
        ArtifactKindV0::EvidenceExclusionV1,
        &exclusion.exclusion_digest,
        &EvidenceExclusionProtobufV1 {
            exclusion_version: EXCLUSION_VERSION_V1.to_string(),
            protected_registration_digests: exclusion.protected_registration_digests.clone(),
            excluded_timestamp_ms: exclusion.excluded_timestamp_ms.clone(),
            excluded_range_digests: exclusion.excluded_range_digests.clone(),
            exclusion_digest: exclusion.exclusion_digest.clone(),
        },
    )
}

pub fn decode_evidence_exclusion_protobuf_v1(
    bytes: &[u8],
) -> Result<EvaluationEvidenceExclusionV1, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::EvidenceExclusionV1)?;
    let value = EvidenceExclusionProtobufV1::decode(envelope.payload.as_slice())
        .map_err(|_| "V1 evidence exclusion protobuf decode failed".to_string())?;
    if value.exclusion_version != EXCLUSION_VERSION_V1 {
        return Err("V1 evidence exclusion version rejected".to_string());
    }
    let exclusion = EvaluationEvidenceExclusionV1 {
        protected_registration_digests: value.protected_registration_digests,
        excluded_timestamp_ms: value.excluded_timestamp_ms,
        excluded_range_digests: value.excluded_range_digests,
        exclusion_digest: value.exclusion_digest,
    };
    validate_evaluation_exclusion_v1(&exclusion)?;
    if exclusion.exclusion_digest != envelope.semantic_digest {
        return Err("V1 evidence exclusion envelope identity rejected".to_string());
    }
    Ok(exclusion)
}

pub fn encode_evaluation_registration_protobuf_v1(
    registration: &AgentCandidateEvaluationRegistrationV1,
) -> Result<Vec<u8>, String> {
    validate_evaluation_registration_v1(registration)?;
    encode_envelope_v0(
        ArtifactKindV0::EvaluationRegistrationV1,
        &registration.registration_digest,
        &EvaluationRegistrationProtobufV1 {
            registration_version: registration.registration_version.clone(),
            agent_id: registration.agent_id.clone(),
            family_digest: registration.family_digest.clone(),
            session_digest: registration.session_digest.clone(),
            usage_ledger_digest: registration.usage_ledger_digest.clone(),
            participant_digests: registration.participant_digests.clone(),
            qualification_receipt_digests: registration.qualification_receipt_digests.clone(),
            exclusion_digest: registration.exclusion_digest.clone(),
            minimum_accepted_timestamp_ms: registration.minimum_accepted_timestamp_ms,
            required_dataset_kinds: registration
                .required_dataset_kinds
                .iter()
                .map(|kind| dataset_kind_tag_v0(*kind))
                .collect::<Result<Vec<_>, _>>()?,
            source_policy_digest: registration.source_policy_digest.clone(),
            finality_policy_digest: registration.finality_policy_digest.clone(),
            label_policy_digest: registration.label_policy_digest.clone(),
            metric_policy_digest: registration.metric_policy_digest.clone(),
            support_policy_digest: registration.support_policy_digest.clone(),
            minimum_future_rows: usize_to_u64_v0(registration.minimum_future_rows)?,
            minimum_mature_events: usize_to_u64_v0(registration.minimum_mature_events)?,
            maximum_requests: usize_to_u64_v0(registration.maximum_requests)?,
            maximum_concurrency: usize_to_u64_v0(registration.maximum_concurrency)?,
            maximum_retries: usize_to_u64_v0(registration.maximum_retries)?,
            labels_hidden_until_opening: registration.labels_hidden_until_opening,
            probabilities_hidden_until_opening: registration.probabilities_hidden_until_opening,
            one_time_opening_required: registration.one_time_opening_required,
            winner_selection_forbidden_before_opening: registration
                .winner_selection_forbidden_before_opening,
            active_promotion_forbidden: registration.active_promotion_forbidden,
            reward_application_forbidden: registration.reward_application_forbidden,
            status: registration_status_tag_v1(registration.status),
            registration_digest: registration.registration_digest.clone(),
        },
    )
}

pub fn decode_evaluation_registration_protobuf_v1(
    bytes: &[u8],
) -> Result<AgentCandidateEvaluationRegistrationV1, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::EvaluationRegistrationV1)?;
    let value = EvaluationRegistrationProtobufV1::decode(envelope.payload.as_slice())
        .map_err(|_| "V1 evaluation registration protobuf decode failed".to_string())?;
    let registration = AgentCandidateEvaluationRegistrationV1 {
        registration_version: value.registration_version,
        agent_id: value.agent_id,
        family_digest: value.family_digest,
        session_digest: value.session_digest,
        usage_ledger_digest: value.usage_ledger_digest,
        participant_digests: value.participant_digests,
        qualification_receipt_digests: value.qualification_receipt_digests,
        exclusion_digest: value.exclusion_digest,
        minimum_accepted_timestamp_ms: value.minimum_accepted_timestamp_ms,
        required_dataset_kinds: value
            .required_dataset_kinds
            .into_iter()
            .map(dataset_kind_from_tag_v0)
            .collect::<Result<Vec<_>, _>>()?,
        source_policy_digest: value.source_policy_digest,
        finality_policy_digest: value.finality_policy_digest,
        label_policy_digest: value.label_policy_digest,
        metric_policy_digest: value.metric_policy_digest,
        support_policy_digest: value.support_policy_digest,
        minimum_future_rows: u64_to_usize_v0(value.minimum_future_rows)?,
        minimum_mature_events: u64_to_usize_v0(value.minimum_mature_events)?,
        maximum_requests: u64_to_usize_v0(value.maximum_requests)?,
        maximum_concurrency: u64_to_usize_v0(value.maximum_concurrency)?,
        maximum_retries: u64_to_usize_v0(value.maximum_retries)?,
        labels_hidden_until_opening: value.labels_hidden_until_opening,
        probabilities_hidden_until_opening: value.probabilities_hidden_until_opening,
        one_time_opening_required: value.one_time_opening_required,
        winner_selection_forbidden_before_opening: value.winner_selection_forbidden_before_opening,
        active_promotion_forbidden: value.active_promotion_forbidden,
        reward_application_forbidden: value.reward_application_forbidden,
        status: registration_status_from_tag_v1(value.status)?,
        registration_digest: value.registration_digest,
    };
    validate_evaluation_registration_v1(&registration)?;
    if registration.registration_digest != envelope.semantic_digest {
        return Err("V1 evaluation registration envelope identity rejected".to_string());
    }
    Ok(registration)
}

pub fn encode_evaluation_journal_protobuf_v1(
    journal: &AgentCandidateEvaluationRegistrationJournalV1,
) -> Result<Vec<u8>, String> {
    if journal.journal_version != EVALUATION_JOURNAL_VERSION_V1
        || journal.agent_id.is_empty()
        || journal.family_digest.is_empty()
        || journal.entries.is_empty()
        || journal.entries.iter().any(|entry| {
            entry.registration_digest.is_empty()
                || entry.status != CandidateEvaluationRegistrationStatusV1::Registered
        })
        || journal.journal_digest != evaluation_journal_digest_v1(journal)
    {
        return Err("V1 evaluation journal rejected".to_string());
    }
    encode_envelope_v0(
        ArtifactKindV0::EvaluationJournalV1,
        &journal.journal_digest,
        &EvaluationJournalProtobufV1 {
            journal_version: journal.journal_version.clone(),
            agent_id: journal.agent_id.clone(),
            family_digest: journal.family_digest.clone(),
            entries: journal
                .entries
                .iter()
                .map(|entry| EvaluationJournalEntryProtobufV1 {
                    registration_digest: entry.registration_digest.clone(),
                    status: registration_status_tag_v1(entry.status),
                })
                .collect(),
            journal_digest: journal.journal_digest.clone(),
        },
    )
}

pub fn decode_evaluation_journal_protobuf_v1(
    bytes: &[u8],
) -> Result<AgentCandidateEvaluationRegistrationJournalV1, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::EvaluationJournalV1)?;
    let value = EvaluationJournalProtobufV1::decode(envelope.payload.as_slice())
        .map_err(|_| "V1 evaluation journal protobuf decode failed".to_string())?;
    let journal = AgentCandidateEvaluationRegistrationJournalV1 {
        journal_version: value.journal_version,
        agent_id: value.agent_id,
        family_digest: value.family_digest,
        entries: value
            .entries
            .into_iter()
            .map(|entry| {
                Ok(AgentCandidateEvaluationRegistrationJournalEntryV1 {
                    registration_digest: entry.registration_digest,
                    status: registration_status_from_tag_v1(entry.status)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        journal_digest: value.journal_digest,
    };
    if journal.journal_version != EVALUATION_JOURNAL_VERSION_V1
        || journal.agent_id.is_empty()
        || journal.family_digest.is_empty()
        || journal.entries.is_empty()
        || journal.entries.iter().any(|entry| {
            entry.registration_digest.is_empty()
                || entry.status != CandidateEvaluationRegistrationStatusV1::Registered
        })
        || journal.journal_digest != evaluation_journal_digest_v1(&journal)
        || journal.journal_digest != envelope.semantic_digest
    {
        return Err("V1 evaluation journal identity rejected".to_string());
    }
    Ok(journal)
}

fn migration_status_tag_v1(value: PersistedLearningIntentMigrationStatusV1) -> u32 {
    match value {
        PersistedLearningIntentMigrationStatusV1::Migrated => 1,
        PersistedLearningIntentMigrationStatusV1::AlreadyMigrated => 2,
        PersistedLearningIntentMigrationStatusV1::SourceArtifactMissing => 3,
        PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch => 4,
        PersistedLearningIntentMigrationStatusV1::PolicyBindingMismatch => 5,
        PersistedLearningIntentMigrationStatusV1::CanonicalIntentInvalid => 6,
        PersistedLearningIntentMigrationStatusV1::CanonicalViewInvalid => 7,
        PersistedLearningIntentMigrationStatusV1::AmbiguousFieldProvenance => 8,
    }
}

fn migration_status_from_tag_v1(
    value: u32,
) -> Result<PersistedLearningIntentMigrationStatusV1, String> {
    match value {
        1 => Ok(PersistedLearningIntentMigrationStatusV1::Migrated),
        2 => Ok(PersistedLearningIntentMigrationStatusV1::AlreadyMigrated),
        3 => Ok(PersistedLearningIntentMigrationStatusV1::SourceArtifactMissing),
        4 => Ok(PersistedLearningIntentMigrationStatusV1::SourceIntegrityMismatch),
        5 => Ok(PersistedLearningIntentMigrationStatusV1::PolicyBindingMismatch),
        6 => Ok(PersistedLearningIntentMigrationStatusV1::CanonicalIntentInvalid),
        7 => Ok(PersistedLearningIntentMigrationStatusV1::CanonicalViewInvalid),
        8 => Ok(PersistedLearningIntentMigrationStatusV1::AmbiguousFieldProvenance),
        _ => Err("intent migration status rejected".to_string()),
    }
}

pub fn encode_canonical_learning_intent_migration_protobuf_v1(
    intent: &AgentLearningIntentV0,
) -> Result<Vec<u8>, String> {
    let policy = default_agent_data_policies()
        .into_iter()
        .find(|policy| policy.agent_kind == intent.agent_kind)
        .ok_or_else(|| "canonical migrated intent policy unavailable".to_string())?;
    validate_agent_learning_intent_v0(intent, &policy)?;
    encode_envelope_v0(
        ArtifactKindV0::CanonicalIntentMigrationV1,
        &intent.intent_digest,
        &CanonicalIntentMigrationProtobufV1 {
            intent_version: intent.intent_version.clone(),
            agent_id: intent.agent_id.clone(),
            agent_kind: agent_kind_tag_v0(intent.agent_kind)?,
            market_scopes: intent
                .market_scopes
                .iter()
                .map(|value| market_scope_tag_v0(*value))
                .collect::<Result<Vec<_>, _>>()?,
            symbols: intent.symbols.clone(),
            required_datasets: intent
                .required_datasets
                .iter()
                .map(|value| dataset_kind_tag_v0(*value))
                .collect::<Result<Vec<_>, _>>()?,
            optional_datasets: intent
                .optional_datasets
                .iter()
                .map(|value| dataset_kind_tag_v0(*value))
                .collect::<Result<Vec<_>, _>>()?,
            cadence: intent.cadence.clone(),
            lookback_bars: usize_to_u64_v0(intent.lookback.bars)?,
            lookback_start_timestamp_ms: intent.lookback.start_timestamp_ms,
            lookback_end_timestamp_ms: intent.lookback.end_timestamp_ms,
            information_cutoff_ms: intent.information_cutoff_ms,
            maximum_staleness_ms: intent.maximum_staleness_ms,
            source_policy_digest: intent.source_policy_digest.clone(),
            feature_policy_digest: intent.feature_policy_digest.clone(),
            label_policy_digest: intent.label_policy_digest.clone(),
            curriculum_policy_digest: intent.curriculum_policy_digest.clone(),
            intent_digest: intent.intent_digest.clone(),
        },
    )
}

pub fn decode_canonical_learning_intent_migration_protobuf_v1(
    bytes: &[u8],
) -> Result<AgentLearningIntentV0, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::CanonicalIntentMigrationV1)?;
    let value = CanonicalIntentMigrationProtobufV1::decode(envelope.payload.as_slice())
        .map_err(|_| "canonical migrated intent decode failed".to_string())?;
    let intent = AgentLearningIntentV0 {
        intent_version: value.intent_version,
        agent_id: value.agent_id,
        agent_kind: agent_kind_from_tag_v0(value.agent_kind)?,
        market_scopes: value
            .market_scopes
            .into_iter()
            .map(market_scope_from_tag_v0)
            .collect::<Result<Vec<_>, _>>()?,
        symbols: value.symbols,
        required_datasets: value
            .required_datasets
            .into_iter()
            .map(dataset_kind_from_tag_v0)
            .collect::<Result<Vec<_>, _>>()?,
        optional_datasets: value
            .optional_datasets
            .into_iter()
            .map(dataset_kind_from_tag_v0)
            .collect::<Result<Vec<_>, _>>()?,
        cadence: value.cadence,
        lookback: DataLookback {
            bars: u64_to_usize_v0(value.lookback_bars)?,
            start_timestamp_ms: value.lookback_start_timestamp_ms,
            end_timestamp_ms: value.lookback_end_timestamp_ms,
        },
        information_cutoff_ms: value.information_cutoff_ms,
        maximum_staleness_ms: value.maximum_staleness_ms,
        source_policy_digest: value.source_policy_digest,
        feature_policy_digest: value.feature_policy_digest,
        label_policy_digest: value.label_policy_digest,
        curriculum_policy_digest: value.curriculum_policy_digest,
        intent_digest: value.intent_digest,
    };
    let policy = default_agent_data_policies()
        .into_iter()
        .find(|policy| policy.agent_kind == intent.agent_kind)
        .ok_or_else(|| "canonical migrated intent policy unavailable".to_string())?;
    validate_agent_learning_intent_v0(&intent, &policy)?;
    if intent.intent_digest != envelope.semantic_digest {
        return Err("canonical migrated intent envelope identity rejected".to_string());
    }
    Ok(intent)
}

pub fn encode_intent_policy_compatibility_proof_protobuf_v1(
    proof: &PersistedIntentPolicyCompatibilityProofV1,
) -> Result<Vec<u8>, String> {
    if proof.agent_id != MOMENTUM_AGENT_ID_V1
        || !proof.semantically_compatible
        || !proof.required_datasets_equal
        || !proof.optional_datasets_equal
        || !proof.allowed_markets_equal
        || !proof.cadence_equal
        || !proof.lookback_equal
        || !proof.staleness_equal
        || proof.proof_digest != policy_compatibility_proof_digest_v1(proof)
    {
        return Err("intent policy compatibility proof rejected".to_string());
    }
    encode_envelope_v0(
        ArtifactKindV0::IntentPolicyCompatibilityProofV1,
        &proof.proof_digest,
        &IntentPolicyCompatibilityProofProtobufV1 {
            proof_version: INTENT_POLICY_PROOF_VERSION_V1.to_string(),
            agent_id: proof.agent_id.clone(),
            legacy_policy_digest: proof.legacy_policy_digest.clone(),
            current_policy_digest: proof.current_policy_digest.clone(),
            required_datasets_equal: proof.required_datasets_equal,
            optional_datasets_equal: proof.optional_datasets_equal,
            markets_equal: proof.allowed_markets_equal,
            cadence_equal: proof.cadence_equal,
            lookback_equal: proof.lookback_equal,
            staleness_equal: proof.staleness_equal,
            compatible: proof.semantically_compatible,
            proof_digest: proof.proof_digest.clone(),
        },
    )
}

pub fn decode_intent_policy_compatibility_proof_protobuf_v1(
    bytes: &[u8],
) -> Result<PersistedIntentPolicyCompatibilityProofV1, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::IntentPolicyCompatibilityProofV1)?;
    let value = IntentPolicyCompatibilityProofProtobufV1::decode(envelope.payload.as_slice())
        .map_err(|_| "intent policy compatibility proof decode failed".to_string())?;
    if value.proof_version != INTENT_POLICY_PROOF_VERSION_V1 {
        return Err("intent policy compatibility proof version rejected".to_string());
    }
    let proof = PersistedIntentPolicyCompatibilityProofV1 {
        agent_id: value.agent_id,
        legacy_policy_digest: value.legacy_policy_digest,
        current_policy_digest: value.current_policy_digest,
        required_datasets_equal: value.required_datasets_equal,
        optional_datasets_equal: value.optional_datasets_equal,
        allowed_markets_equal: value.markets_equal,
        cadence_equal: value.cadence_equal,
        lookback_equal: value.lookback_equal,
        staleness_equal: value.staleness_equal,
        semantically_compatible: value.compatible,
        proof_digest: value.proof_digest,
    };
    if encode_intent_policy_compatibility_proof_protobuf_v1(&proof).is_err()
        || proof.proof_digest != envelope.semantic_digest
    {
        return Err("intent policy compatibility proof identity rejected".to_string());
    }
    Ok(proof)
}

pub fn encode_learning_intent_migration_proof_protobuf_v1(
    proof: &PersistedLearningIntentMigrationProofV1,
) -> Result<Vec<u8>, String> {
    if proof.migration_version != INTENT_MIGRATION_PROOF_VERSION_V1
        || proof.agent_id != MOMENTUM_AGENT_ID_V1
        || proof.field_provenance_digests.len() != 16
        || proof.field_provenance_digests
            != stable_migration_values_v1(&proof.field_provenance_digests)
        || !proof.information_cutoff_unchanged
        || !proof.lookback_unchanged
        || !proof.policy_semantics_unchanged
        || !proof.evidence_set_unchanged
        || !proof.exclusions_unchanged
        || !proof.no_field_invented
        || proof.migration_status != PersistedLearningIntentMigrationStatusV1::Migrated
        || proof.proof_digest != migration_proof_digest_v1(proof)
    {
        return Err("persisted learning intent migration proof rejected".to_string());
    }
    encode_envelope_v0(
        ArtifactKindV0::IntentMigrationProofV1,
        &proof.proof_digest,
        &IntentMigrationProofProtobufV1 {
            proof_version: proof.migration_version.clone(),
            agent_id: proof.agent_id.clone(),
            legacy_session_digest: proof.legacy_session_digest.clone(),
            legacy_intent_digest: proof.legacy_intent_digest.clone(),
            canonical_gap_digest: proof.gap_report_digest.clone(),
            composite_registration_digest: proof.composite_registration_digest.clone(),
            canonical_snapshot_digest: proof.merged_snapshot_digest.clone(),
            policy_compatibility_proof_digest: proof.policy_compatibility_proof_digest.clone(),
            field_provenance_digests: proof.field_provenance_digests.clone(),
            canonical_intent_digest: proof.canonical_intent_digest.clone(),
            canonical_view_digest: proof.canonical_view_digest.clone(),
            cutoff_unchanged: proof.information_cutoff_unchanged,
            lookback_unchanged: proof.lookback_unchanged,
            policy_unchanged: proof.policy_semantics_unchanged,
            required_evidence_unchanged: proof.evidence_set_unchanged,
            optional_evidence_unchanged: proof.evidence_set_unchanged,
            exclusions_unchanged: proof.exclusions_unchanged,
            no_field_invented: proof.no_field_invented,
            status: migration_status_tag_v1(proof.migration_status),
            proof_digest: proof.proof_digest.clone(),
        },
    )
}

pub fn decode_learning_intent_migration_proof_protobuf_v1(
    bytes: &[u8],
) -> Result<PersistedLearningIntentMigrationProofV1, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::IntentMigrationProofV1)?;
    let value = IntentMigrationProofProtobufV1::decode(envelope.payload.as_slice())
        .map_err(|_| "persisted learning intent migration proof decode failed".to_string())?;
    let proof = PersistedLearningIntentMigrationProofV1 {
        migration_version: value.proof_version,
        agent_id: value.agent_id,
        legacy_session_digest: value.legacy_session_digest,
        legacy_intent_digest: value.legacy_intent_digest,
        gap_report_digest: value.canonical_gap_digest,
        composite_registration_digest: value.composite_registration_digest,
        merged_snapshot_digest: value.canonical_snapshot_digest,
        policy_compatibility_proof_digest: value.policy_compatibility_proof_digest,
        field_provenance_digests: value.field_provenance_digests,
        canonical_intent_digest: value.canonical_intent_digest,
        canonical_view_digest: value.canonical_view_digest,
        information_cutoff_unchanged: value.cutoff_unchanged,
        lookback_unchanged: value.lookback_unchanged,
        policy_semantics_unchanged: value.policy_unchanged,
        evidence_set_unchanged: value.required_evidence_unchanged
            && value.optional_evidence_unchanged,
        exclusions_unchanged: value.exclusions_unchanged,
        no_field_invented: value.no_field_invented,
        migration_status: migration_status_from_tag_v1(value.status)?,
        proof_digest: value.proof_digest,
    };
    if encode_learning_intent_migration_proof_protobuf_v1(&proof).is_err()
        || proof.proof_digest != envelope.semantic_digest
    {
        return Err("persisted learning intent migration proof identity rejected".to_string());
    }
    Ok(proof)
}

pub fn encode_learning_intent_migration_journal_protobuf_v1(
    journal: &PersistedLearningIntentMigrationJournalV1,
) -> Result<Vec<u8>, String> {
    if journal.journal_version != INTENT_MIGRATION_JOURNAL_VERSION_V1
        || journal.agent_id != MOMENTUM_AGENT_ID_V1
        || journal.entry_count != 1
        || journal.network_requests != 0
        || journal.transport_constructions != 0
        || journal.credential_reads != 0
        || journal.prospective_reads != 0
        || journal.active_model_changes != 0
        || journal.status != PersistedLearningIntentMigrationStatusV1::Migrated
        || journal.journal_digest != migration_journal_digest_v1(journal)
    {
        return Err("persisted learning intent migration journal rejected".to_string());
    }
    encode_envelope_v0(
        ArtifactKindV0::IntentMigrationJournalV1,
        &journal.journal_digest,
        &IntentMigrationJournalProtobufV1 {
            journal_version: journal.journal_version.clone(),
            agent_id: journal.agent_id.clone(),
            migration_proof_digest: journal.migration_proof_digest.clone(),
            canonical_intent_digest: journal.canonical_intent_digest.clone(),
            canonical_view_digest: journal.canonical_view_digest.clone(),
            entry_count: usize_to_u64_v0(journal.entry_count)?,
            network_requests: usize_to_u64_v0(journal.network_requests)?,
            transport_constructions: usize_to_u64_v0(journal.transport_constructions)?,
            credential_reads: usize_to_u64_v0(journal.credential_reads)?,
            prospective_reads: usize_to_u64_v0(journal.prospective_reads)?,
            active_model_changes: usize_to_u64_v0(journal.active_model_changes)?,
            status: migration_status_tag_v1(journal.status),
            journal_digest: journal.journal_digest.clone(),
        },
    )
}

pub fn decode_learning_intent_migration_journal_protobuf_v1(
    bytes: &[u8],
) -> Result<PersistedLearningIntentMigrationJournalV1, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::IntentMigrationJournalV1)?;
    let value = IntentMigrationJournalProtobufV1::decode(envelope.payload.as_slice())
        .map_err(|_| "persisted learning intent migration journal decode failed".to_string())?;
    let journal = PersistedLearningIntentMigrationJournalV1 {
        journal_version: value.journal_version,
        agent_id: value.agent_id,
        migration_proof_digest: value.migration_proof_digest,
        canonical_intent_digest: value.canonical_intent_digest,
        canonical_view_digest: value.canonical_view_digest,
        entry_count: u64_to_usize_v0(value.entry_count)?,
        network_requests: u64_to_usize_v0(value.network_requests)?,
        transport_constructions: u64_to_usize_v0(value.transport_constructions)?,
        credential_reads: u64_to_usize_v0(value.credential_reads)?,
        prospective_reads: u64_to_usize_v0(value.prospective_reads)?,
        active_model_changes: u64_to_usize_v0(value.active_model_changes)?,
        status: migration_status_from_tag_v1(value.status)?,
        journal_digest: value.journal_digest,
    };
    if encode_learning_intent_migration_journal_protobuf_v1(&journal).is_err()
        || journal.journal_digest != envelope.semantic_digest
    {
        return Err("persisted learning intent migration journal identity rejected".to_string());
    }
    Ok(journal)
}

pub fn encode_session_protobuf_v0(
    session: &AgentPrivateLearningSessionV0,
) -> Result<Vec<u8>, String> {
    if session.session_digest != session_digest_v0(session) {
        return Err("private learning session semantic digest rejected".to_string());
    }
    encode_envelope_v0(
        ArtifactKindV0::Session,
        &session.session_digest,
        &SessionProtobufV0 {
            session_version: session.session_version.clone(),
            session_id: session.session_id.clone(),
            agent_id: session.agent_id.clone(),
            agent_kind: agent_kind_tag_v0(session.agent_kind)?,
            intent_digest: session.intent_digest.clone(),
            data_view_digest: session.data_view_digest.clone(),
            trainer_capability_digest: session.trainer_capability_digest.clone(),
            information_cutoff_ms: session.information_cutoff_ms,
            source_artifact_digests: session.source_artifact_digests.clone(),
            feature_policy_digest: session.feature_policy_digest.clone(),
            label_policy_digest: session.label_policy_digest.clone(),
            curriculum_policy_digest: session.curriculum_policy_digest.clone(),
            private_namespace_digest: session.private_namespace_digest.clone(),
            parent_model_version: session.parent_model_version.clone(),
            session_status: session_status_tag_v0(session.session_status),
            session_digest: session.session_digest.clone(),
            required_dataset_kinds: session
                .required_dataset_kinds
                .iter()
                .map(|kind| dataset_kind_tag_v0(*kind))
                .collect::<Result<Vec<_>, _>>()?,
            optional_dataset_kinds: session
                .optional_dataset_kinds
                .iter()
                .map(|kind| dataset_kind_tag_v0(*kind))
                .collect::<Result<Vec<_>, _>>()?,
            allowed_markets: session
                .allowed_markets
                .iter()
                .map(|market| market_scope_tag_v0(*market))
                .collect::<Result<Vec<_>, _>>()?,
            symbols: session.symbols.clone(),
            cadence: session.cadence.clone(),
            lookback_bars: usize_to_u64_v0(session.lookback.bars)?,
            lookback_start_timestamp_ms: session.lookback.start_timestamp_ms,
            lookback_end_timestamp_ms: session.lookback.end_timestamp_ms,
            maximum_staleness_ms: session.maximum_staleness_ms,
            source_policy_digest: session.source_policy_digest.clone(),
            training_ledger_digest: session.training_ledger_digest.clone(),
            trainer_projection_digest: session.trainer_projection_digest.clone(),
        },
    )
}

pub fn decode_session_protobuf_v0(bytes: &[u8]) -> Result<AgentPrivateLearningSessionV0, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::Session)?;
    let value = SessionProtobufV0::decode(envelope.payload.as_slice())
        .map_err(|_| "private learning session decode failed".to_string())?;
    let session = AgentPrivateLearningSessionV0 {
        session_version: value.session_version,
        session_id: value.session_id,
        agent_id: value.agent_id,
        agent_kind: agent_kind_from_tag_v0(value.agent_kind)?,
        intent_digest: value.intent_digest,
        data_view_digest: value.data_view_digest,
        trainer_capability_digest: value.trainer_capability_digest,
        information_cutoff_ms: value.information_cutoff_ms,
        required_dataset_kinds: value
            .required_dataset_kinds
            .into_iter()
            .map(dataset_kind_from_tag_v0)
            .collect::<Result<Vec<_>, _>>()?,
        optional_dataset_kinds: value
            .optional_dataset_kinds
            .into_iter()
            .map(dataset_kind_from_tag_v0)
            .collect::<Result<Vec<_>, _>>()?,
        allowed_markets: value
            .allowed_markets
            .into_iter()
            .map(market_scope_from_tag_v0)
            .collect::<Result<Vec<_>, _>>()?,
        symbols: value.symbols,
        cadence: value.cadence,
        lookback: DataLookback {
            bars: u64_to_usize_v0(value.lookback_bars)?,
            start_timestamp_ms: value.lookback_start_timestamp_ms,
            end_timestamp_ms: value.lookback_end_timestamp_ms,
        },
        maximum_staleness_ms: value.maximum_staleness_ms,
        source_artifact_digests: value.source_artifact_digests,
        source_policy_digest: value.source_policy_digest,
        feature_policy_digest: value.feature_policy_digest,
        label_policy_digest: value.label_policy_digest,
        curriculum_policy_digest: value.curriculum_policy_digest,
        private_namespace_digest: value.private_namespace_digest,
        training_ledger_digest: value.training_ledger_digest,
        trainer_projection_digest: value.trainer_projection_digest,
        parent_model_version: value.parent_model_version,
        session_status: session_status_from_tag_v0(value.session_status)?,
        session_digest: value.session_digest,
    };
    if session.session_digest != envelope.semantic_digest
        || session.session_digest != session_digest_v0(&session)
    {
        return Err("private learning session identity rejected".to_string());
    }
    Ok(session)
}

pub fn encode_trainer_projection_protobuf_v0(
    projection: &AgentTrainerInputProjectionV0,
) -> Result<Vec<u8>, String> {
    if projection.projection_version != PROJECTION_VERSION_V0
        || projection.projection_digest != projection_digest_v0(projection)
        || projection.source_view_digest.is_empty()
        || projection.consumed_artifact_digests.len() != 1
        || projection.primary_series_digest.as_ref() != projection.consumed_artifact_digests.first()
    {
        return Err("trainer input projection semantic digest rejected".to_string());
    }
    encode_envelope_v0(
        ArtifactKindV0::Projection,
        &projection.projection_digest,
        &TrainerProjectionProtobufV0 {
            projection_version: projection.projection_version.clone(),
            agent_id: projection.agent_id.clone(),
            trainer_kind: trainer_kind_tag_v0(projection.trainer_kind),
            source_view_digest: projection.source_view_digest.clone(),
            consumed_artifact_digests: projection.consumed_artifact_digests.clone(),
            referenced_but_unconsumed_artifact_digests: projection
                .referenced_but_unconsumed_artifact_digests
                .clone(),
            primary_series_digest: projection.primary_series_digest.clone(),
            auxiliary_series_digests: projection.auxiliary_series_digests.clone(),
            projection_policy_digest: projection.projection_policy_digest.clone(),
            projection_digest: projection.projection_digest.clone(),
        },
    )
}

pub fn decode_trainer_projection_protobuf_v0(
    bytes: &[u8],
) -> Result<AgentTrainerInputProjectionV0, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::Projection)?;
    let value = TrainerProjectionProtobufV0::decode(envelope.payload.as_slice())
        .map_err(|_| "trainer input projection decode failed".to_string())?;
    let projection = AgentTrainerInputProjectionV0 {
        projection_version: value.projection_version,
        agent_id: value.agent_id,
        trainer_kind: trainer_kind_from_tag_v0(value.trainer_kind)?,
        source_view_digest: value.source_view_digest,
        consumed_artifact_digests: value.consumed_artifact_digests,
        referenced_but_unconsumed_artifact_digests: value
            .referenced_but_unconsumed_artifact_digests,
        primary_series_digest: value.primary_series_digest,
        auxiliary_series_digests: value.auxiliary_series_digests,
        projection_policy_digest: value.projection_policy_digest,
        projection_digest: value.projection_digest,
    };
    if projection.projection_version != PROJECTION_VERSION_V0
        || projection.projection_digest != envelope.semantic_digest
        || projection.projection_digest != projection_digest_v0(&projection)
        || projection.consumed_artifact_digests.len() != 1
        || projection.primary_series_digest.as_ref() != projection.consumed_artifact_digests.first()
    {
        return Err("trainer input projection identity rejected".to_string());
    }
    Ok(projection)
}

pub fn encode_evidence_usage_ledger_protobuf_v0(
    ledger: &CandidateEvidenceUsageLedgerV0,
) -> Result<Vec<u8>, String> {
    if ledger.ledger_version != EVIDENCE_LEDGER_VERSION_V0
        || ledger.entries.is_empty()
        || ledger.ledger_digest != evidence_usage_ledger_digest_v0(ledger)
        || ledger
            .entries
            .iter()
            .any(|entry| entry.entry_digest != evidence_usage_entry_digest_v0(entry))
    {
        return Err("candidate evidence usage ledger semantic digest rejected".to_string());
    }
    encode_envelope_v0(
        ArtifactKindV0::EvidenceUsageLedger,
        &ledger.ledger_digest,
        &EvidenceUsageLedgerProtobufV0 {
            ledger_version: ledger.ledger_version.clone(),
            agent_id: ledger.agent_id.clone(),
            candidate_digest: ledger.candidate_digest.clone(),
            session_digest: ledger.session_digest.clone(),
            entries: ledger
                .entries
                .iter()
                .map(|entry| {
                    Ok(EvidenceUsageEntryProtobufV0 {
                        artifact_digest: entry.artifact_digest.clone(),
                        range: entry.range.as_ref().map(range_to_protobuf_v0).transpose()?,
                        use_kind: evidence_use_kind_tag_v0(entry.use_kind),
                        labels_read: entry.labels_read,
                        parameters_updated: entry.parameters_updated,
                        checkpoint_selection_influenced: entry.checkpoint_selection_influenced,
                        candidate_identity_influenced: entry.candidate_identity_influenced,
                        entry_digest: entry.entry_digest.clone(),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
            ledger_digest: ledger.ledger_digest.clone(),
        },
    )
}

pub fn decode_evidence_usage_ledger_protobuf_v0(
    bytes: &[u8],
) -> Result<CandidateEvidenceUsageLedgerV0, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::EvidenceUsageLedger)?;
    let value = EvidenceUsageLedgerProtobufV0::decode(envelope.payload.as_slice())
        .map_err(|_| "candidate evidence usage ledger decode failed".to_string())?;
    let ledger = CandidateEvidenceUsageLedgerV0 {
        ledger_version: value.ledger_version,
        agent_id: value.agent_id,
        candidate_digest: value.candidate_digest,
        session_digest: value.session_digest,
        entries: value
            .entries
            .into_iter()
            .map(|entry| {
                Ok(CandidateEvidenceUsageEntryV0 {
                    artifact_digest: entry.artifact_digest,
                    range: entry
                        .range
                        .map(|range| range_from_protobuf_v0(Some(range)))
                        .transpose()?,
                    use_kind: evidence_use_kind_from_tag_v0(entry.use_kind)?,
                    labels_read: entry.labels_read,
                    parameters_updated: entry.parameters_updated,
                    checkpoint_selection_influenced: entry.checkpoint_selection_influenced,
                    candidate_identity_influenced: entry.candidate_identity_influenced,
                    entry_digest: entry.entry_digest,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        ledger_digest: value.ledger_digest,
    };
    if ledger.ledger_version != EVIDENCE_LEDGER_VERSION_V0
        || ledger.entries.is_empty()
        || ledger.ledger_digest != envelope.semantic_digest
        || ledger.ledger_digest != evidence_usage_ledger_digest_v0(&ledger)
        || ledger
            .entries
            .iter()
            .any(|entry| entry.entry_digest != evidence_usage_entry_digest_v0(entry))
    {
        return Err("candidate evidence usage ledger identity rejected".to_string());
    }
    Ok(ledger)
}

pub fn encode_candidate_identity_audit_protobuf_v0(
    audit: &AgentCandidateIdentityAuditV0,
) -> Result<Vec<u8>, String> {
    if audit.audit_version != IDENTITY_AUDIT_VERSION_V0
        || audit.audit_digest != identity_audit_digest_v0(audit)
    {
        return Err("candidate identity audit semantic digest rejected".to_string());
    }
    encode_envelope_v0(
        ArtifactKindV0::CandidateIdentityAudit,
        &audit.audit_digest,
        &CandidateIdentityAuditProtobufV0 {
            audit_version: audit.audit_version.clone(),
            candidate_digest: audit.candidate_digest.clone(),
            model_identity_inputs: audit.model_identity_inputs.clone(),
            metric_identity_inputs: audit.metric_identity_inputs.clone(),
            test_evidence_in_identity: audit.test_evidence_in_identity,
            historical_test_status: historical_test_status_tag_v0(audit.historical_test_status),
            eligible_for_fresh_historical_test: audit.eligible_for_fresh_historical_test,
            eligible_for_future_evaluation_registration: audit
                .eligible_for_future_evaluation_registration,
            superseded_by_input_binding_hardening: audit.superseded_by_input_binding_hardening,
            audit_digest: audit.audit_digest.clone(),
        },
    )
}

pub fn decode_candidate_identity_audit_protobuf_v0(
    bytes: &[u8],
) -> Result<AgentCandidateIdentityAuditV0, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::CandidateIdentityAudit)?;
    let value = CandidateIdentityAuditProtobufV0::decode(envelope.payload.as_slice())
        .map_err(|_| "candidate identity audit decode failed".to_string())?;
    let audit = AgentCandidateIdentityAuditV0 {
        audit_version: value.audit_version,
        candidate_digest: value.candidate_digest,
        model_identity_inputs: value.model_identity_inputs,
        metric_identity_inputs: value.metric_identity_inputs,
        test_evidence_in_identity: value.test_evidence_in_identity,
        historical_test_status: historical_test_status_from_tag_v0(value.historical_test_status)?,
        eligible_for_fresh_historical_test: value.eligible_for_fresh_historical_test,
        eligible_for_future_evaluation_registration: value
            .eligible_for_future_evaluation_registration,
        superseded_by_input_binding_hardening: value.superseded_by_input_binding_hardening,
        audit_digest: value.audit_digest,
    };
    if audit.audit_version != IDENTITY_AUDIT_VERSION_V0
        || audit.audit_digest != envelope.semantic_digest
        || audit.audit_digest != identity_audit_digest_v0(&audit)
    {
        return Err("candidate identity audit identity rejected".to_string());
    }
    Ok(audit)
}

pub fn encode_candidate_evaluation_registration_protobuf_v0(
    registration: &AgentCandidateEvaluationRegistrationV0,
) -> Result<Vec<u8>, String> {
    validate_candidate_evaluation_registration_v0(registration)?;
    if registration.registration_digest != evaluation_registration_digest_v0(registration) {
        return Err("candidate evaluation registration semantic digest rejected".to_string());
    }
    encode_envelope_v0(
        ArtifactKindV0::EvaluationRegistration,
        &registration.registration_digest,
        &CandidateEvaluationRegistrationProtobufV0 {
            registration_version: registration.registration_version.clone(),
            agent_id: registration.agent_id.clone(),
            candidate_digest: registration.candidate_digest.clone(),
            session_digest: registration.session_digest.clone(),
            evidence_usage_ledger_digest: registration.evidence_usage_ledger_digest.clone(),
            identity_audit_digest: registration.identity_audit_digest.clone(),
            evaluation_cutoff_exclusive_ms: registration.evaluation_cutoff_exclusive_ms,
            required_dataset_kinds: registration
                .required_dataset_kinds
                .iter()
                .map(|kind| dataset_kind_tag_v0(*kind))
                .collect::<Result<Vec<_>, _>>()?,
            source_policy_digest: registration.source_policy_digest.clone(),
            finality_policy_digest: registration.finality_policy_digest.clone(),
            label_policy_digest: registration.label_policy_digest.clone(),
            metric_policy_digest: registration.metric_policy_digest.clone(),
            support_policy_digest: registration.support_policy_digest.clone(),
            comparator_digests: registration.comparator_digests.clone(),
            minimum_future_rows: usize_to_u64_v0(registration.minimum_future_rows)?,
            minimum_mature_events: usize_to_u64_v0(registration.minimum_mature_events)?,
            maximum_requests: usize_to_u64_v0(registration.maximum_requests)?,
            maximum_concurrency: usize_to_u64_v0(registration.maximum_concurrency)?,
            maximum_retries: usize_to_u64_v0(registration.maximum_retries)?,
            labels_hidden_until_opening: registration.labels_hidden_until_opening,
            probabilities_hidden_until_opening: registration.probabilities_hidden_until_opening,
            one_time_opening_required: registration.one_time_opening_required,
            active_promotion_forbidden: registration.active_promotion_forbidden,
            reward_application_forbidden: registration.reward_application_forbidden,
            status: registration_status_tag_v0(registration.status),
            registration_digest: registration.registration_digest.clone(),
        },
    )
}

pub fn decode_candidate_evaluation_registration_protobuf_v0(
    bytes: &[u8],
) -> Result<AgentCandidateEvaluationRegistrationV0, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::EvaluationRegistration)?;
    let value = CandidateEvaluationRegistrationProtobufV0::decode(envelope.payload.as_slice())
        .map_err(|_| "candidate evaluation registration decode failed".to_string())?;
    let registration = AgentCandidateEvaluationRegistrationV0 {
        registration_version: value.registration_version,
        agent_id: value.agent_id,
        candidate_digest: value.candidate_digest,
        session_digest: value.session_digest,
        evidence_usage_ledger_digest: value.evidence_usage_ledger_digest,
        identity_audit_digest: value.identity_audit_digest,
        evaluation_cutoff_exclusive_ms: value.evaluation_cutoff_exclusive_ms,
        required_dataset_kinds: value
            .required_dataset_kinds
            .into_iter()
            .map(dataset_kind_from_tag_v0)
            .collect::<Result<Vec<_>, _>>()?,
        source_policy_digest: value.source_policy_digest,
        finality_policy_digest: value.finality_policy_digest,
        label_policy_digest: value.label_policy_digest,
        metric_policy_digest: value.metric_policy_digest,
        support_policy_digest: value.support_policy_digest,
        comparator_digests: value.comparator_digests,
        minimum_future_rows: u64_to_usize_v0(value.minimum_future_rows)?,
        minimum_mature_events: u64_to_usize_v0(value.minimum_mature_events)?,
        maximum_requests: u64_to_usize_v0(value.maximum_requests)?,
        maximum_concurrency: u64_to_usize_v0(value.maximum_concurrency)?,
        maximum_retries: u64_to_usize_v0(value.maximum_retries)?,
        labels_hidden_until_opening: value.labels_hidden_until_opening,
        probabilities_hidden_until_opening: value.probabilities_hidden_until_opening,
        one_time_opening_required: value.one_time_opening_required,
        active_promotion_forbidden: value.active_promotion_forbidden,
        reward_application_forbidden: value.reward_application_forbidden,
        status: registration_status_from_tag_v0(value.status)?,
        registration_digest: value.registration_digest,
    };
    validate_candidate_evaluation_registration_v0(&registration)?;
    if registration.registration_digest != envelope.semantic_digest
        || registration.registration_digest != evaluation_registration_digest_v0(&registration)
    {
        return Err("candidate evaluation registration identity rejected".to_string());
    }
    Ok(registration)
}

pub fn encode_candidate_evaluation_journal_protobuf_v0(
    journal: &AgentCandidateEvaluationRegistrationJournalV0,
) -> Result<Vec<u8>, String> {
    if journal.journal_version != EVALUATION_JOURNAL_VERSION_V0
        || journal.entries.is_empty()
        || journal.journal_digest != evaluation_journal_digest_v0(journal)
    {
        return Err("candidate evaluation journal semantic digest rejected".to_string());
    }
    encode_envelope_v0(
        ArtifactKindV0::EvaluationJournal,
        &journal.journal_digest,
        &CandidateEvaluationJournalProtobufV0 {
            journal_version: journal.journal_version.clone(),
            agent_id: journal.agent_id.clone(),
            candidate_digest: journal.candidate_digest.clone(),
            entries: journal
                .entries
                .iter()
                .map(|entry| CandidateEvaluationJournalEntryProtobufV0 {
                    registration_digest: entry.registration_digest.clone(),
                    status: registration_status_tag_v0(entry.status),
                })
                .collect(),
            journal_digest: journal.journal_digest.clone(),
        },
    )
}

pub fn decode_candidate_evaluation_journal_protobuf_v0(
    bytes: &[u8],
) -> Result<AgentCandidateEvaluationRegistrationJournalV0, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::EvaluationJournal)?;
    let value = CandidateEvaluationJournalProtobufV0::decode(envelope.payload.as_slice())
        .map_err(|_| "candidate evaluation journal decode failed".to_string())?;
    let journal = AgentCandidateEvaluationRegistrationJournalV0 {
        journal_version: value.journal_version,
        agent_id: value.agent_id,
        candidate_digest: value.candidate_digest,
        entries: value
            .entries
            .into_iter()
            .map(|entry| {
                Ok(AgentCandidateEvaluationRegistrationJournalEntryV0 {
                    registration_digest: entry.registration_digest,
                    status: registration_status_from_tag_v0(entry.status)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        journal_digest: value.journal_digest,
    };
    if journal.journal_version != EVALUATION_JOURNAL_VERSION_V0
        || journal.entries.is_empty()
        || journal.journal_digest != envelope.semantic_digest
        || journal.journal_digest != evaluation_journal_digest_v0(&journal)
    {
        return Err("candidate evaluation journal identity rejected".to_string());
    }
    Ok(journal)
}

pub fn encode_dataset_manifest_protobuf_v0(
    manifest: &AgentPrivateDatasetManifestV0,
) -> Result<Vec<u8>, String> {
    validate_dataset_manifest_v0(manifest)?;
    if manifest.manifest_digest != dataset_manifest_digest_v0(manifest) {
        return Err("private dataset semantic digest rejected".to_string());
    }
    encode_envelope_v0(
        ArtifactKindV0::Dataset,
        &manifest.manifest_digest,
        &DatasetManifestProtobufV0 {
            dataset_version: manifest.dataset_version.clone(),
            dataset_id: manifest.dataset_id.clone(),
            agent_id: manifest.agent_id.clone(),
            session_id: manifest.session_id.clone(),
            data_view_digest: manifest.data_view_digest.clone(),
            source_artifact_digests: manifest.source_artifact_digests.clone(),
            dataset_kinds: manifest
                .dataset_kinds
                .iter()
                .map(|kind| dataset_kind_tag_v0(*kind))
                .collect::<Result<Vec<_>, _>>()?,
            information_cutoff_ms: manifest.information_cutoff_ms,
            row_count: usize_to_u64_v0(manifest.row_count)?,
            training_range: Some(range_to_protobuf_v0(&manifest.training_range)?),
            first_purge_range: Some(range_to_protobuf_v0(&manifest.first_purge_range)?),
            validation_range: Some(range_to_protobuf_v0(&manifest.validation_range)?),
            second_purge_range: Some(range_to_protobuf_v0(&manifest.second_purge_range)?),
            sealed_test_range: manifest
                .sealed_test_range
                .as_ref()
                .map(range_to_protobuf_v0)
                .transpose()?,
            normalizer_fit_range: Some(range_to_protobuf_v0(&manifest.normalizer_fit_range)?),
            validation_parameter_update_count: usize_to_u64_v0(
                manifest.validation_parameter_update_count,
            )?,
            test_checkpoint_selection_count: usize_to_u64_v0(
                manifest.test_checkpoint_selection_count,
            )?,
            prospective_row_read_count: usize_to_u64_v0(manifest.prospective_row_read_count)?,
            prospective_label_read_count: usize_to_u64_v0(manifest.prospective_label_read_count)?,
            feature_artifact_digest: manifest.feature_artifact_digest.clone(),
            label_artifact_digest: manifest.label_artifact_digest.clone(),
            normalizer_digest: manifest.normalizer_digest.clone(),
            manifest_digest: manifest.manifest_digest.clone(),
        },
    )
}

pub fn decode_dataset_manifest_protobuf_v0(
    bytes: &[u8],
) -> Result<AgentPrivateDatasetManifestV0, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::Dataset)?;
    let value = DatasetManifestProtobufV0::decode(envelope.payload.as_slice())
        .map_err(|_| "private dataset manifest decode failed".to_string())?;
    let manifest = AgentPrivateDatasetManifestV0 {
        dataset_version: value.dataset_version,
        dataset_id: value.dataset_id,
        agent_id: value.agent_id,
        session_id: value.session_id,
        data_view_digest: value.data_view_digest,
        source_artifact_digests: value.source_artifact_digests,
        dataset_kinds: value
            .dataset_kinds
            .into_iter()
            .map(dataset_kind_from_tag_v0)
            .collect::<Result<Vec<_>, _>>()?,
        information_cutoff_ms: value.information_cutoff_ms,
        row_count: u64_to_usize_v0(value.row_count)?,
        training_range: range_from_protobuf_v0(value.training_range)?,
        first_purge_range: range_from_protobuf_v0(value.first_purge_range)?,
        validation_range: range_from_protobuf_v0(value.validation_range)?,
        second_purge_range: range_from_protobuf_v0(value.second_purge_range)?,
        sealed_test_range: value
            .sealed_test_range
            .map(|range| range_from_protobuf_v0(Some(range)))
            .transpose()?,
        normalizer_fit_range: range_from_protobuf_v0(value.normalizer_fit_range)?,
        validation_parameter_update_count: u64_to_usize_v0(
            value.validation_parameter_update_count,
        )?,
        test_checkpoint_selection_count: u64_to_usize_v0(value.test_checkpoint_selection_count)?,
        prospective_row_read_count: u64_to_usize_v0(value.prospective_row_read_count)?,
        prospective_label_read_count: u64_to_usize_v0(value.prospective_label_read_count)?,
        feature_artifact_digest: value.feature_artifact_digest,
        label_artifact_digest: value.label_artifact_digest,
        normalizer_digest: value.normalizer_digest,
        manifest_digest: value.manifest_digest,
    };
    validate_dataset_manifest_v0(&manifest)?;
    if manifest.manifest_digest != envelope.semantic_digest
        || manifest.manifest_digest != dataset_manifest_digest_v0(&manifest)
    {
        return Err("private dataset manifest identity rejected".to_string());
    }
    Ok(manifest)
}

pub fn encode_candidate_protobuf_v0(
    candidate: &AgentSandboxLearningCandidateV0,
) -> Result<Vec<u8>, String> {
    validate_candidate_v0(candidate)?;
    if candidate.candidate_digest != candidate_digest_v0(candidate) {
        return Err("private candidate semantic digest rejected".to_string());
    }
    encode_envelope_v0(
        ArtifactKindV0::Candidate,
        &candidate.candidate_digest,
        &CandidateProtobufV0 {
            candidate_version: candidate.candidate_version.clone(),
            agent_id: candidate.agent_id.clone(),
            session_digest: candidate.session_digest.clone(),
            data_view_digest: candidate.data_view_digest.clone(),
            parent_model_version: candidate.parent_model_version.clone(),
            model_artifact_digest: candidate.model_artifact_digest.clone(),
            feature_policy_digest: candidate.feature_policy_digest.clone(),
            label_policy_digest: candidate.label_policy_digest.clone(),
            normalizer_digest: candidate.normalizer_digest.clone(),
            training_policy_digest: candidate.training_policy_digest.clone(),
            private_metrics_digest: candidate.private_metrics_digest.clone(),
            deployment_status: 1,
            retrospective_research_only: candidate.retrospective_research_only,
            eligible_for_active_committee: candidate.eligible_for_active_committee,
            eligible_for_promotion: candidate.eligible_for_promotion,
            eligible_for_reward: candidate.eligible_for_reward,
            candidate_digest: candidate.candidate_digest.clone(),
        },
    )
}

pub fn decode_candidate_protobuf_v0(
    bytes: &[u8],
) -> Result<AgentSandboxLearningCandidateV0, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::Candidate)?;
    let value = CandidateProtobufV0::decode(envelope.payload.as_slice())
        .map_err(|_| "private candidate decode failed".to_string())?;
    if value.deployment_status != 1 {
        return Err("private candidate deployment status rejected".to_string());
    }
    let candidate = AgentSandboxLearningCandidateV0 {
        candidate_version: value.candidate_version,
        agent_id: value.agent_id,
        session_digest: value.session_digest,
        data_view_digest: value.data_view_digest,
        parent_model_version: value.parent_model_version,
        model_artifact_digest: value.model_artifact_digest,
        feature_policy_digest: value.feature_policy_digest,
        label_policy_digest: value.label_policy_digest,
        normalizer_digest: value.normalizer_digest,
        training_policy_digest: value.training_policy_digest,
        private_metrics_digest: value.private_metrics_digest,
        deployment_status: ModelAgentDeploymentStatus::ShadowOnly,
        retrospective_research_only: value.retrospective_research_only,
        eligible_for_active_committee: value.eligible_for_active_committee,
        eligible_for_promotion: value.eligible_for_promotion,
        eligible_for_reward: value.eligible_for_reward,
        candidate_digest: value.candidate_digest,
    };
    validate_candidate_v0(&candidate)?;
    if candidate.candidate_digest != envelope.semantic_digest
        || candidate.candidate_digest != candidate_digest_v0(&candidate)
    {
        return Err("private candidate identity rejected".to_string());
    }
    Ok(candidate)
}

pub fn encode_journal_protobuf_v0(
    journal: &AgentLearningSessionJournalV0,
) -> Result<Vec<u8>, String> {
    if journal.journal_digest != journal_digest_v0(journal) {
        return Err("private journal semantic digest rejected".to_string());
    }
    encode_envelope_v0(
        ArtifactKindV0::Journal,
        &journal.journal_digest,
        &JournalProtobufV0 {
            journal_version: journal.journal_version.clone(),
            agent_id: journal.agent_id.clone(),
            entries: journal
                .entries
                .iter()
                .map(|entry| JournalEntryProtobufV0 {
                    session_digest: entry.session_digest.clone(),
                    session_status: session_status_tag_v0(entry.session_status),
                    dataset_manifest_digest: entry.dataset_manifest_digest.clone(),
                    candidate_digest: entry.candidate_digest.clone(),
                })
                .collect(),
            journal_digest: journal.journal_digest.clone(),
        },
    )
}

pub fn decode_journal_protobuf_v0(bytes: &[u8]) -> Result<AgentLearningSessionJournalV0, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::Journal)?;
    let value = JournalProtobufV0::decode(envelope.payload.as_slice())
        .map_err(|_| "private journal decode failed".to_string())?;
    let journal = AgentLearningSessionJournalV0 {
        journal_version: value.journal_version,
        agent_id: value.agent_id,
        entries: value
            .entries
            .into_iter()
            .map(|entry| {
                Ok(AgentLearningSessionJournalEntryV0 {
                    session_digest: entry.session_digest,
                    session_status: session_status_from_tag_v0(entry.session_status)?,
                    dataset_manifest_digest: entry.dataset_manifest_digest,
                    candidate_digest: entry.candidate_digest,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        journal_digest: value.journal_digest,
    };
    if journal.journal_digest != envelope.semantic_digest
        || journal.journal_digest != journal_digest_v0(&journal)
    {
        return Err("private journal identity rejected".to_string());
    }
    Ok(journal)
}

pub fn encode_capability_registry_protobuf_v0(
    registry: &AgentTrainerCapabilityRegistryV0,
) -> Result<Vec<u8>, String> {
    if registry.registry_digest != registry_digest_v0(registry)
        || registry
            .capabilities
            .iter()
            .any(|capability| capability.capability_digest != capability_digest_v0(capability))
    {
        return Err("trainer capability registry semantic digest rejected".to_string());
    }
    encode_envelope_v0(
        ArtifactKindV0::Registry,
        &registry.registry_digest,
        &CapabilityRegistryProtobufV0 {
            registry_version: registry.registry_version.clone(),
            capabilities: registry
                .capabilities
                .iter()
                .map(|capability| {
                    Ok(CapabilityProtobufV0 {
                        agent_id: capability.agent_id.clone(),
                        trainer_kind: trainer_kind_tag_v0(capability.trainer_kind),
                        supported_dataset_kinds: capability
                            .supported_dataset_kinds
                            .iter()
                            .map(|kind| dataset_kind_tag_v0(*kind))
                            .collect::<Result<Vec<_>, String>>()?,
                        supports_training: capability.supports_training,
                        shadow_only: capability.shadow_only,
                        capability_digest: capability.capability_digest.clone(),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
            registry_digest: registry.registry_digest.clone(),
        },
    )
}

pub fn decode_capability_registry_protobuf_v0(
    bytes: &[u8],
) -> Result<AgentTrainerCapabilityRegistryV0, String> {
    let envelope = decode_envelope_v0(bytes, ArtifactKindV0::Registry)?;
    let value = CapabilityRegistryProtobufV0::decode(envelope.payload.as_slice())
        .map_err(|_| "trainer capability registry decode failed".to_string())?;
    let registry = AgentTrainerCapabilityRegistryV0 {
        registry_version: value.registry_version,
        capabilities: value
            .capabilities
            .into_iter()
            .map(|capability| {
                Ok(AgentTrainerCapabilityV0 {
                    agent_id: capability.agent_id,
                    trainer_kind: trainer_kind_from_tag_v0(capability.trainer_kind)?,
                    supported_dataset_kinds: capability
                        .supported_dataset_kinds
                        .into_iter()
                        .map(dataset_kind_from_tag_v0)
                        .collect::<Result<Vec<_>, String>>()?,
                    supports_training: capability.supports_training,
                    shadow_only: capability.shadow_only,
                    capability_digest: capability.capability_digest,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        registry_digest: value.registry_digest,
    };
    if registry.registry_digest != envelope.semantic_digest
        || registry.registry_digest != registry_digest_v0(&registry)
        || registry
            .capabilities
            .iter()
            .any(|capability| capability.capability_digest != capability_digest_v0(capability))
    {
        return Err("trainer capability registry identity rejected".to_string());
    }
    Ok(registry)
}

fn range_to_protobuf_v0(range: &IndexRangeV0) -> Result<RangeProtobufV0, String> {
    Ok(RangeProtobufV0 {
        start: usize_to_u64_v0(range.start)?,
        end: usize_to_u64_v0(range.end)?,
    })
}

fn range_from_protobuf_v0(value: Option<RangeProtobufV0>) -> Result<IndexRangeV0, String> {
    let value = value.ok_or_else(|| "private dataset range missing".to_string())?;
    Ok(IndexRangeV0 {
        start: u64_to_usize_v0(value.start)?,
        end: u64_to_usize_v0(value.end)?,
    })
}

fn usize_to_u64_v0(value: usize) -> Result<u64, String> {
    u64::try_from(value).map_err(|_| "private learning count overflow".to_string())
}

fn u64_to_usize_v0(value: u64) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| "private learning count rejected".to_string())
}

fn agent_kind_tag_v0(kind: AgentKind) -> Result<u32, String> {
    match kind {
        AgentKind::MomentumTrendFast => Ok(1),
        AgentKind::ValueQualityFilter => Ok(2),
        AgentKind::CycleRiskSkeptic => Ok(3),
        _ => Err("private learning agent kind rejected".to_string()),
    }
}

fn agent_kind_from_tag_v0(value: u32) -> Result<AgentKind, String> {
    match value {
        1 => Ok(AgentKind::MomentumTrendFast),
        2 => Ok(AgentKind::ValueQualityFilter),
        3 => Ok(AgentKind::CycleRiskSkeptic),
        _ => Err("private learning agent kind rejected".to_string()),
    }
}

fn trainer_kind_tag_v0(kind: AgentTrainerKindV0) -> u32 {
    match kind {
        AgentTrainerKindV0::MomentumFrozenMambaHead => 1,
        AgentTrainerKindV0::CycleRiskIndependentShadow => 2,
        AgentTrainerKindV0::ValueQualityUnavailable => 3,
    }
}

fn trainer_kind_from_tag_v0(value: u32) -> Result<AgentTrainerKindV0, String> {
    match value {
        1 => Ok(AgentTrainerKindV0::MomentumFrozenMambaHead),
        2 => Ok(AgentTrainerKindV0::CycleRiskIndependentShadow),
        3 => Ok(AgentTrainerKindV0::ValueQualityUnavailable),
        _ => Err("private learning trainer kind rejected".to_string()),
    }
}

fn market_scope_tag_v0(market: AcquisitionMarketScope) -> Result<u32, String> {
    match market {
        AcquisitionMarketScope::UsStocks => Ok(1),
        AcquisitionMarketScope::KoreanStocks => Ok(2),
        AcquisitionMarketScope::BtcCrypto => Ok(3),
        AcquisitionMarketScope::Unknown => {
            Err("private learning market scope rejected".to_string())
        }
    }
}

fn market_scope_from_tag_v0(value: u32) -> Result<AcquisitionMarketScope, String> {
    match value {
        1 => Ok(AcquisitionMarketScope::UsStocks),
        2 => Ok(AcquisitionMarketScope::KoreanStocks),
        3 => Ok(AcquisitionMarketScope::BtcCrypto),
        _ => Err("private learning market scope rejected".to_string()),
    }
}

fn session_status_tag_v0(status: AgentLearningSessionStatusV0) -> u32 {
    match status {
        AgentLearningSessionStatusV0::Registered => 1,
        AgentLearningSessionStatusV0::DatasetReady => 2,
        AgentLearningSessionStatusV0::CandidateProduced => 3,
        AgentLearningSessionStatusV0::InsufficientEvidence => 4,
        AgentLearningSessionStatusV0::TrainerUnavailable => 5,
        AgentLearningSessionStatusV0::RejectedUnauthorizedEvidence => 6,
        AgentLearningSessionStatusV0::RejectedCutoffLeakage => 7,
        AgentLearningSessionStatusV0::RejectedLabelLeakage => 8,
        AgentLearningSessionStatusV0::RejectedSafetyInvariant => 9,
        AgentLearningSessionStatusV0::TechnicalFailure => 10,
    }
}

fn session_status_from_tag_v0(value: u32) -> Result<AgentLearningSessionStatusV0, String> {
    match value {
        1 => Ok(AgentLearningSessionStatusV0::Registered),
        2 => Ok(AgentLearningSessionStatusV0::DatasetReady),
        3 => Ok(AgentLearningSessionStatusV0::CandidateProduced),
        4 => Ok(AgentLearningSessionStatusV0::InsufficientEvidence),
        5 => Ok(AgentLearningSessionStatusV0::TrainerUnavailable),
        6 => Ok(AgentLearningSessionStatusV0::RejectedUnauthorizedEvidence),
        7 => Ok(AgentLearningSessionStatusV0::RejectedCutoffLeakage),
        8 => Ok(AgentLearningSessionStatusV0::RejectedLabelLeakage),
        9 => Ok(AgentLearningSessionStatusV0::RejectedSafetyInvariant),
        10 => Ok(AgentLearningSessionStatusV0::TechnicalFailure),
        _ => Err("private learning session status rejected".to_string()),
    }
}

fn session_status_tag_v1(status: AgentLearningSessionStatusV1) -> u32 {
    match status {
        AgentLearningSessionStatusV1::Registered => 1,
        AgentLearningSessionStatusV1::PersistedViewVerified => 2,
        AgentLearningSessionStatusV1::ProjectionReady => 3,
        AgentLearningSessionStatusV1::CandidateFamilyFrozen => 4,
        AgentLearningSessionStatusV1::InsufficientEvidence => 5,
        AgentLearningSessionStatusV1::TrainerUnavailable => 6,
        AgentLearningSessionStatusV1::ValidationBlocked => 7,
        AgentLearningSessionStatusV1::RejectedUnauthorizedEvidence => 8,
        AgentLearningSessionStatusV1::RejectedCutoffLeakage => 9,
        AgentLearningSessionStatusV1::RejectedSafetyInvariant => 10,
        AgentLearningSessionStatusV1::TechnicalFailure => 11,
    }
}

fn session_status_from_tag_v1(value: u32) -> Result<AgentLearningSessionStatusV1, String> {
    match value {
        1 => Ok(AgentLearningSessionStatusV1::Registered),
        2 => Ok(AgentLearningSessionStatusV1::PersistedViewVerified),
        3 => Ok(AgentLearningSessionStatusV1::ProjectionReady),
        4 => Ok(AgentLearningSessionStatusV1::CandidateFamilyFrozen),
        5 => Ok(AgentLearningSessionStatusV1::InsufficientEvidence),
        6 => Ok(AgentLearningSessionStatusV1::TrainerUnavailable),
        7 => Ok(AgentLearningSessionStatusV1::ValidationBlocked),
        8 => Ok(AgentLearningSessionStatusV1::RejectedUnauthorizedEvidence),
        9 => Ok(AgentLearningSessionStatusV1::RejectedCutoffLeakage),
        10 => Ok(AgentLearningSessionStatusV1::RejectedSafetyInvariant),
        11 => Ok(AgentLearningSessionStatusV1::TechnicalFailure),
        _ => Err("V1 session status rejected".to_string()),
    }
}

fn participant_role_tag_v1(role: CandidateParticipantRoleV1) -> u32 {
    match role {
        CandidateParticipantRoleV1::ModelCandidate => 1,
        CandidateParticipantRoleV1::LinearComparator => 2,
        CandidateParticipantRoleV1::ConstantComparator => 3,
    }
}

fn participant_role_from_tag_v1(value: u32) -> Result<CandidateParticipantRoleV1, String> {
    match value {
        1 => Ok(CandidateParticipantRoleV1::ModelCandidate),
        2 => Ok(CandidateParticipantRoleV1::LinearComparator),
        3 => Ok(CandidateParticipantRoleV1::ConstantComparator),
        _ => Err("V1 participant role rejected".to_string()),
    }
}

fn qualification_status_tag_v1(status: ValidationQualificationStatusV1) -> u32 {
    match status {
        ValidationQualificationStatusV1::Qualified => 1,
        ValidationQualificationStatusV1::RejectedInsufficientValidation => 2,
        ValidationQualificationStatusV1::RejectedProbabilityCollapse => 3,
        ValidationQualificationStatusV1::RejectedNumericalFailure => 4,
    }
}

fn qualification_status_from_tag_v1(value: u32) -> Result<ValidationQualificationStatusV1, String> {
    match value {
        1 => Ok(ValidationQualificationStatusV1::Qualified),
        2 => Ok(ValidationQualificationStatusV1::RejectedInsufficientValidation),
        3 => Ok(ValidationQualificationStatusV1::RejectedProbabilityCollapse),
        4 => Ok(ValidationQualificationStatusV1::RejectedNumericalFailure),
        _ => Err("V1 qualification status rejected".to_string()),
    }
}

fn evidence_use_tag_v1(kind: CandidateEvidenceUseV1) -> u32 {
    match kind {
        CandidateEvidenceUseV1::ViewBinding => 1,
        CandidateEvidenceUseV1::TrainerProjection => 2,
        CandidateEvidenceUseV1::FeatureDerivation => 3,
        CandidateEvidenceUseV1::LabelDerivation => 4,
        CandidateEvidenceUseV1::NormalizerFit => 5,
        CandidateEvidenceUseV1::ParameterTraining => 6,
        CandidateEvidenceUseV1::ValidationInference => 7,
        CandidateEvidenceUseV1::ValidationMetric => 8,
        CandidateEvidenceUseV1::FamilyInclusion => 9,
        CandidateEvidenceUseV1::ReferencedButUnconsumed => 10,
        CandidateEvidenceUseV1::ReservedRetrospectiveUnused => 11,
    }
}

fn evidence_use_from_tag_v1(value: u32) -> Result<CandidateEvidenceUseV1, String> {
    match value {
        1 => Ok(CandidateEvidenceUseV1::ViewBinding),
        2 => Ok(CandidateEvidenceUseV1::TrainerProjection),
        3 => Ok(CandidateEvidenceUseV1::FeatureDerivation),
        4 => Ok(CandidateEvidenceUseV1::LabelDerivation),
        5 => Ok(CandidateEvidenceUseV1::NormalizerFit),
        6 => Ok(CandidateEvidenceUseV1::ParameterTraining),
        7 => Ok(CandidateEvidenceUseV1::ValidationInference),
        8 => Ok(CandidateEvidenceUseV1::ValidationMetric),
        9 => Ok(CandidateEvidenceUseV1::FamilyInclusion),
        10 => Ok(CandidateEvidenceUseV1::ReferencedButUnconsumed),
        11 => Ok(CandidateEvidenceUseV1::ReservedRetrospectiveUnused),
        _ => Err("V1 evidence use rejected".to_string()),
    }
}

fn registration_status_tag_v1(status: CandidateEvaluationRegistrationStatusV1) -> u32 {
    match status {
        CandidateEvaluationRegistrationStatusV1::Registered => 1,
        CandidateEvaluationRegistrationStatusV1::CandidateUnavailable => 2,
        CandidateEvaluationRegistrationStatusV1::SessionInvalid => 3,
        CandidateEvaluationRegistrationStatusV1::ViewInvalid => 4,
        CandidateEvaluationRegistrationStatusV1::ProjectionInvalid => 5,
        CandidateEvaluationRegistrationStatusV1::FamilyInvalid => 6,
        CandidateEvaluationRegistrationStatusV1::QualificationBlocked => 7,
        CandidateEvaluationRegistrationStatusV1::HistoricalTestAccessDetected => 8,
        CandidateEvaluationRegistrationStatusV1::ExclusionInvalid => 9,
        CandidateEvaluationRegistrationStatusV1::InsufficientParticipants => 10,
    }
}

fn registration_status_from_tag_v1(
    value: u32,
) -> Result<CandidateEvaluationRegistrationStatusV1, String> {
    match value {
        1 => Ok(CandidateEvaluationRegistrationStatusV1::Registered),
        2 => Ok(CandidateEvaluationRegistrationStatusV1::CandidateUnavailable),
        3 => Ok(CandidateEvaluationRegistrationStatusV1::SessionInvalid),
        4 => Ok(CandidateEvaluationRegistrationStatusV1::ViewInvalid),
        5 => Ok(CandidateEvaluationRegistrationStatusV1::ProjectionInvalid),
        6 => Ok(CandidateEvaluationRegistrationStatusV1::FamilyInvalid),
        7 => Ok(CandidateEvaluationRegistrationStatusV1::QualificationBlocked),
        8 => Ok(CandidateEvaluationRegistrationStatusV1::HistoricalTestAccessDetected),
        9 => Ok(CandidateEvaluationRegistrationStatusV1::ExclusionInvalid),
        10 => Ok(CandidateEvaluationRegistrationStatusV1::InsufficientParticipants),
        _ => Err("V1 registration status rejected".to_string()),
    }
}

fn dataset_kind_tag_v0(kind: DatasetKind) -> Result<u32, String> {
    match kind {
        DatasetKind::DailyOhlcv => Ok(1),
        DatasetKind::AdjustedDailyOhlcv => Ok(2),
        DatasetKind::CorporateActions => Ok(3),
        DatasetKind::QuarterlyFundamentals => Ok(4),
        DatasetKind::ValuationMetrics => Ok(5),
        DatasetKind::MarketIndexDaily => Ok(6),
        DatasetKind::MarketBreadthDaily => Ok(7),
        DatasetKind::VolatilityDaily => Ok(8),
        DatasetKind::LiquidityDaily => Ok(9),
        DatasetKind::CryptoDailyOhlcv => Ok(10),
        DatasetKind::MacroSeries => Ok(11),
        DatasetKind::Unknown => Err("private learning dataset kind rejected".to_string()),
    }
}

fn dataset_kind_from_tag_v0(value: u32) -> Result<DatasetKind, String> {
    match value {
        1 => Ok(DatasetKind::DailyOhlcv),
        2 => Ok(DatasetKind::AdjustedDailyOhlcv),
        3 => Ok(DatasetKind::CorporateActions),
        4 => Ok(DatasetKind::QuarterlyFundamentals),
        5 => Ok(DatasetKind::ValuationMetrics),
        6 => Ok(DatasetKind::MarketIndexDaily),
        7 => Ok(DatasetKind::MarketBreadthDaily),
        8 => Ok(DatasetKind::VolatilityDaily),
        9 => Ok(DatasetKind::LiquidityDaily),
        10 => Ok(DatasetKind::CryptoDailyOhlcv),
        11 => Ok(DatasetKind::MacroSeries),
        _ => Err("private learning dataset kind rejected".to_string()),
    }
}

fn evidence_use_kind_tag_v0(kind: CandidateEvidenceUseV0) -> u32 {
    match kind {
        CandidateEvidenceUseV0::IntentBinding => 1,
        CandidateEvidenceUseV0::ViewBinding => 2,
        CandidateEvidenceUseV0::TrainerProjection => 3,
        CandidateEvidenceUseV0::FeatureDerivation => 4,
        CandidateEvidenceUseV0::LabelDerivation => 5,
        CandidateEvidenceUseV0::NormalizerFit => 6,
        CandidateEvidenceUseV0::ParameterTraining => 7,
        CandidateEvidenceUseV0::ValidationInference => 8,
        CandidateEvidenceUseV0::ValidationMetric => 9,
        CandidateEvidenceUseV0::CheckpointSelection => 10,
        CandidateEvidenceUseV0::HistoricalTestInference => 11,
        CandidateEvidenceUseV0::HistoricalTestMetric => 12,
        CandidateEvidenceUseV0::CandidateIdentity => 13,
        CandidateEvidenceUseV0::ReportOnly => 14,
        CandidateEvidenceUseV0::Unused => 15,
    }
}

fn evidence_use_kind_from_tag_v0(value: u32) -> Result<CandidateEvidenceUseV0, String> {
    match value {
        1 => Ok(CandidateEvidenceUseV0::IntentBinding),
        2 => Ok(CandidateEvidenceUseV0::ViewBinding),
        3 => Ok(CandidateEvidenceUseV0::TrainerProjection),
        4 => Ok(CandidateEvidenceUseV0::FeatureDerivation),
        5 => Ok(CandidateEvidenceUseV0::LabelDerivation),
        6 => Ok(CandidateEvidenceUseV0::NormalizerFit),
        7 => Ok(CandidateEvidenceUseV0::ParameterTraining),
        8 => Ok(CandidateEvidenceUseV0::ValidationInference),
        9 => Ok(CandidateEvidenceUseV0::ValidationMetric),
        10 => Ok(CandidateEvidenceUseV0::CheckpointSelection),
        11 => Ok(CandidateEvidenceUseV0::HistoricalTestInference),
        12 => Ok(CandidateEvidenceUseV0::HistoricalTestMetric),
        13 => Ok(CandidateEvidenceUseV0::CandidateIdentity),
        14 => Ok(CandidateEvidenceUseV0::ReportOnly),
        15 => Ok(CandidateEvidenceUseV0::Unused),
        _ => Err("candidate evidence use kind rejected".to_string()),
    }
}

fn historical_test_status_tag_v0(status: CandidateHistoricalTestStatusV0) -> u32 {
    match status {
        CandidateHistoricalTestStatusV0::FreshAndSealed => 1,
        CandidateHistoricalTestStatusV0::ReadForInferenceOnly => 2,
        CandidateHistoricalTestStatusV0::MetricsAlreadyComputed => 3,
        CandidateHistoricalTestStatusV0::InfluencedCandidateSelection => 4,
        CandidateHistoricalTestStatusV0::InfluencedCandidateIdentity => 5,
        CandidateHistoricalTestStatusV0::FullyConsumedRetrospectively => 6,
        CandidateHistoricalTestStatusV0::LineageAmbiguous => 7,
        CandidateHistoricalTestStatusV0::NoCandidate => 8,
    }
}

fn historical_test_status_from_tag_v0(
    value: u32,
) -> Result<CandidateHistoricalTestStatusV0, String> {
    match value {
        1 => Ok(CandidateHistoricalTestStatusV0::FreshAndSealed),
        2 => Ok(CandidateHistoricalTestStatusV0::ReadForInferenceOnly),
        3 => Ok(CandidateHistoricalTestStatusV0::MetricsAlreadyComputed),
        4 => Ok(CandidateHistoricalTestStatusV0::InfluencedCandidateSelection),
        5 => Ok(CandidateHistoricalTestStatusV0::InfluencedCandidateIdentity),
        6 => Ok(CandidateHistoricalTestStatusV0::FullyConsumedRetrospectively),
        7 => Ok(CandidateHistoricalTestStatusV0::LineageAmbiguous),
        8 => Ok(CandidateHistoricalTestStatusV0::NoCandidate),
        _ => Err("candidate historical test status rejected".to_string()),
    }
}

fn registration_status_tag_v0(status: CandidateEvaluationRegistrationStatusV0) -> u32 {
    match status {
        CandidateEvaluationRegistrationStatusV0::Registered => 1,
        CandidateEvaluationRegistrationStatusV0::CandidateUnavailable => 2,
        CandidateEvaluationRegistrationStatusV0::CandidateIntegrityInvalid => 3,
        CandidateEvaluationRegistrationStatusV0::LineageAmbiguousBlocked => 4,
        CandidateEvaluationRegistrationStatusV0::ComparatorUnavailable => 5,
        CandidateEvaluationRegistrationStatusV0::PolicyInvalid => 6,
    }
}

fn registration_status_from_tag_v0(
    value: u32,
) -> Result<CandidateEvaluationRegistrationStatusV0, String> {
    match value {
        1 => Ok(CandidateEvaluationRegistrationStatusV0::Registered),
        2 => Ok(CandidateEvaluationRegistrationStatusV0::CandidateUnavailable),
        3 => Ok(CandidateEvaluationRegistrationStatusV0::CandidateIntegrityInvalid),
        4 => Ok(CandidateEvaluationRegistrationStatusV0::LineageAmbiguousBlocked),
        5 => Ok(CandidateEvaluationRegistrationStatusV0::ComparatorUnavailable),
        6 => Ok(CandidateEvaluationRegistrationStatusV0::PolicyInvalid),
        _ => Err("candidate evaluation registration status rejected".to_string()),
    }
}

fn registration_status_code_v0(status: CandidateEvaluationRegistrationStatusV0) -> &'static str {
    match status {
        CandidateEvaluationRegistrationStatusV0::Registered => "registered",
        CandidateEvaluationRegistrationStatusV0::CandidateUnavailable => "candidate_unavailable",
        CandidateEvaluationRegistrationStatusV0::CandidateIntegrityInvalid => {
            "candidate_integrity_invalid"
        }
        CandidateEvaluationRegistrationStatusV0::LineageAmbiguousBlocked => {
            "lineage_ambiguous_blocked"
        }
        CandidateEvaluationRegistrationStatusV0::ComparatorUnavailable => "comparator_unavailable",
        CandidateEvaluationRegistrationStatusV0::PolicyInvalid => "policy_invalid",
    }
}

fn validate_candidate_evaluation_registration_v0(
    registration: &AgentCandidateEvaluationRegistrationV0,
) -> Result<(), String> {
    let mut required = registration.required_dataset_kinds.clone();
    required.sort();
    required.dedup();
    let mut comparators = registration.comparator_digests.clone();
    comparators.sort();
    comparators.dedup();
    if registration.registration_version != EVALUATION_REGISTRATION_VERSION_V0
        || registration.agent_id.is_empty()
        || registration.candidate_digest.is_empty()
        || registration.session_digest.is_empty()
        || registration.evidence_usage_ledger_digest.is_empty()
        || registration.identity_audit_digest.is_empty()
        || registration.evaluation_cutoff_exclusive_ms == 0
        || registration.required_dataset_kinds != required
        || registration.comparator_digests != comparators
        || registration.finality_policy_digest.is_empty()
        || registration.label_policy_digest.is_empty()
        || registration.metric_policy_digest.is_empty()
        || registration.support_policy_digest.is_empty()
        || registration.minimum_future_rows == 0
        || registration.minimum_mature_events == 0
        || registration.maximum_requests != 1
        || registration.maximum_concurrency != 1
        || registration.maximum_retries != 0
        || !registration.labels_hidden_until_opening
        || !registration.probabilities_hidden_until_opening
        || !registration.one_time_opening_required
        || !registration.active_promotion_forbidden
        || !registration.reward_application_forbidden
        || (registration.status == CandidateEvaluationRegistrationStatusV0::Registered
            && (registration.required_dataset_kinds.is_empty()
                || registration.source_policy_digest.is_empty()
                || registration.comparator_digests.is_empty()))
    {
        return Err("candidate evaluation registration invariant rejected".to_string());
    }
    Ok(())
}

fn zero_candidate_evaluation_safety_counters_v0() -> CandidateEvaluationSafetyCountersV0 {
    CandidateEvaluationSafetyCountersV0 {
        active_committee_count: 3,
        network_requests: 0,
        credential_reads: 0,
        prospective_row_reads: 0,
        prospective_label_reads: 0,
        prospective_mutations: 0,
        active_model_changes: 0,
        chair_decisions: 0,
        votes: 0,
        rewards: 0,
        penalties: 0,
        voice_changes: 0,
        promotions: 0,
        executions: 0,
    }
}

fn zero_safety_counters_v0() -> LearningDataPlaneSafetyCountersV0 {
    LearningDataPlaneSafetyCountersV0 {
        active_committee_count: 3,
        network_requests: 0,
        credential_reads: 0,
        prospective_artifact_mutations: 0,
        prospective_label_reads: 0,
        chair_decisions: 0,
        votes: 0,
        rewards: 0,
        penalties: 0,
        voice_changes: 0,
        executions: 0,
    }
}

fn capability_digest_v0(capability: &AgentTrainerCapabilityV0) -> String {
    stable_hash_string(&format!(
        "{}:{:?}:{:?}:{}:{}",
        capability.agent_id,
        capability.trainer_kind,
        capability.supported_dataset_kinds,
        capability.supports_training,
        capability.shadow_only
    ))
}

fn registry_digest_v0(registry: &AgentTrainerCapabilityRegistryV0) -> String {
    stable_hash_string(&format!(
        "{}:{:?}",
        registry.registry_version,
        registry
            .capabilities
            .iter()
            .map(|capability| capability.capability_digest.as_str())
            .collect::<Vec<_>>()
    ))
}

fn projection_digest_v1(projection: &AgentTrainerInputProjectionV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}:{}:{:?}:{:?}:{:?}:{}",
        projection.projection_version,
        projection.agent_id,
        projection.trainer_kind,
        projection.source_view_digest,
        projection.consumed_artifact_digests,
        projection.referenced_unconsumed_artifact_digests,
        projection.primary_series_digest,
        projection.projection_policy_digest
    ))
}

fn validate_projection_v1(projection: &AgentTrainerInputProjectionV1) -> Result<(), String> {
    let mut consumed = projection.consumed_artifact_digests.clone();
    consumed.sort();
    consumed.dedup();
    let mut referenced = projection.referenced_unconsumed_artifact_digests.clone();
    referenced.sort();
    referenced.dedup();
    if projection.projection_version != PROJECTION_VERSION_V1
        || projection.agent_id.is_empty()
        || projection.source_view_digest.is_empty()
        || projection.consumed_artifact_digests != consumed
        || projection.referenced_unconsumed_artifact_digests != referenced
        || projection
            .consumed_artifact_digests
            .iter()
            .any(|digest| referenced.contains(digest))
        || projection
            .primary_series_digest
            .as_ref()
            .is_none_or(|digest| !consumed.contains(digest))
        || projection.projection_policy_digest.is_empty()
        || projection.projection_digest != projection_digest_v1(projection)
    {
        return Err("V1 trainer projection rejected".to_string());
    }
    Ok(())
}

fn session_digest_v1(session: &AgentPrivateLearningSessionV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}:{:?}:{:?}:{}:{}:{}:{}:{:?}",
        session.session_version,
        session.session_id,
        session.agent_id,
        session.agent_kind,
        session.intent_digest,
        session.view_digest,
        session.projection_digest,
        session.capability_digest,
        session.source_policy_digest,
        session.feature_policy_digest,
        session.label_policy_digest,
        session.curriculum_policy_digest,
        session.information_cutoff_ms,
        session.source_artifact_digests,
        session.consumed_artifact_digests,
        session.referenced_unconsumed_artifact_digests,
        session.private_namespace_digest,
        session.training_ledger_digest,
        session.fresh_initialization,
        session.historical_test_access_forbidden,
        session.status
    ))
}

fn validate_session_v1(session: &AgentPrivateLearningSessionV1) -> Result<(), String> {
    let sorted_unique = |values: &[String]| {
        let mut expected = values.to_vec();
        expected.sort();
        expected.dedup();
        expected == values
    };
    if session.session_version != SESSION_VERSION_V1_FAMILY
        || session.session_id.is_empty()
        || session.agent_id.is_empty()
        || session.intent_digest.is_empty()
        || session.view_digest.is_empty()
        || session.projection_digest.is_empty()
        || session.capability_digest.is_empty()
        || session.source_policy_digest.is_empty()
        || session.feature_policy_digest.is_empty()
        || session.label_policy_digest.is_empty()
        || session.curriculum_policy_digest.is_empty()
        || session.information_cutoff_ms == 0
        || session.source_artifact_digests.is_empty()
        || !sorted_unique(&session.source_artifact_digests)
        || !sorted_unique(&session.consumed_artifact_digests)
        || !sorted_unique(&session.referenced_unconsumed_artifact_digests)
        || session
            .consumed_artifact_digests
            .iter()
            .any(|digest| !session.source_artifact_digests.contains(digest))
        || session
            .referenced_unconsumed_artifact_digests
            .iter()
            .any(|digest| !session.source_artifact_digests.contains(digest))
        || session.private_namespace_digest.is_empty()
        || session.training_ledger_digest.is_empty()
        || !session.fresh_initialization
        || !session.historical_test_access_forbidden
        || session.session_digest != session_digest_v1(session)
    {
        return Err("V1 learning session rejected".to_string());
    }
    Ok(())
}

fn participant_digest_v1(participant: &FrozenCandidateParticipantV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}",
        PARTICIPANT_VERSION_V1,
        participant.participant_id,
        participant.role,
        participant.model_kind,
        participant.model_artifact_digest,
        participant.parameter_digest,
        participant.normalizer_digest,
        participant.feature_policy_digest,
        participant.label_policy_digest,
        participant.training_policy_digest,
        participant.initialization_digest,
        participant.deployment_status
    ))
}

fn validate_participant_v1(participant: &FrozenCandidateParticipantV1) -> Result<(), String> {
    if participant.participant_id.is_empty()
        || participant.model_kind.is_empty()
        || participant.model_artifact_digest.is_empty()
        || participant.parameter_digest.is_empty()
        || participant.normalizer_digest.is_empty()
        || participant.feature_policy_digest.is_empty()
        || participant.label_policy_digest.is_empty()
        || participant.training_policy_digest.is_empty()
        || participant.initialization_digest.is_empty()
        || participant.deployment_status != ModelAgentDeploymentStatus::ShadowOnly
        || participant.participant_digest != participant_digest_v1(participant)
    {
        return Err("V1 frozen participant rejected".to_string());
    }
    Ok(())
}

fn qualification_receipt_digest_v1(receipt: &ParticipantValidationQualificationV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{:?}:{}",
        QUALIFICATION_VERSION_V1,
        receipt.participant_digest,
        receipt.validation_range_digest,
        receipt.metric_policy_digest,
        receipt.private_metric_digest,
        receipt.qualification_status,
        receipt.parameter_updates_during_validation
    ))
}

fn validate_qualification_receipt_v1(
    receipt: &ParticipantValidationQualificationV1,
) -> Result<(), String> {
    if receipt.participant_digest.is_empty()
        || receipt.validation_range_digest.is_empty()
        || receipt.metric_policy_digest.is_empty()
        || receipt.private_metric_digest.is_empty()
        || receipt.parameter_updates_during_validation != 0
        || receipt.receipt_digest != qualification_receipt_digest_v1(receipt)
    {
        return Err("V1 validation qualification receipt rejected".to_string());
    }
    Ok(())
}

fn candidate_family_digest_v1(family: &AgentCandidateFamilyV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{:?}:{}:{}:{}:{}:{}",
        family.family_version,
        family.agent_id,
        family.session_digest,
        family.view_digest,
        family.projection_digest,
        family
            .participants
            .iter()
            .map(|participant| participant.participant_digest.as_str())
            .collect::<Vec<_>>(),
        family.winner_selected,
        family.historical_test_accessed,
        family.eligible_for_active_committee,
        family.eligible_for_promotion,
        family.eligible_for_reward
    ))
}

fn validate_candidate_family_v1(family: &AgentCandidateFamilyV1) -> Result<(), String> {
    let mut participant_ids = family
        .participants
        .iter()
        .map(|participant| participant.participant_id.as_str())
        .collect::<Vec<_>>();
    let original_ids = participant_ids.clone();
    participant_ids.sort();
    participant_ids.dedup();
    let mut receipt_digests = family.validation_qualification_receipts.clone();
    receipt_digests.sort();
    receipt_digests.dedup();
    if family.family_version != FAMILY_VERSION_V1
        || family.agent_id.is_empty()
        || family.session_digest.is_empty()
        || family.view_digest.is_empty()
        || family.projection_digest.is_empty()
        || original_ids != participant_ids
        || family.validation_qualification_receipts != receipt_digests
        || family.participants.iter().any(|participant| {
            validate_participant_v1(participant).is_err()
                || participant.deployment_status != ModelAgentDeploymentStatus::ShadowOnly
        })
        || family.winner_selected
        || family.historical_test_accessed
        || family.eligible_for_active_committee
        || family.eligible_for_promotion
        || family.eligible_for_reward
        || family.family_digest != candidate_family_digest_v1(family)
    {
        return Err("V1 candidate family rejected".to_string());
    }
    Ok(())
}

fn usage_entry_digest_v1(entry: &CandidateEvidenceUsageEntryV1) -> String {
    stable_hash_string(&format!(
        "{}:{:?}:{:?}:{}:{}",
        entry.artifact_digest,
        entry.range,
        entry.use_kind,
        entry.labels_read,
        entry.parameters_updated
    ))
}

fn usage_ledger_digest_v1(ledger: &AgentCandidateUsageLedgerV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{:?}:{}:{}:{}:{}:{}:{}",
        ledger.ledger_version,
        ledger.agent_id,
        ledger.session_digest,
        ledger.family_digest,
        ledger
            .entries
            .iter()
            .map(|entry| entry.entry_digest.as_str())
            .collect::<Vec<_>>(),
        ledger.historical_test_row_reads,
        ledger.historical_test_label_reads,
        ledger.historical_test_inference_count,
        ledger.historical_test_metric_count,
        ledger.historical_test_checkpoint_selection_count,
        ledger.historical_test_identity_influence
    ))
}

fn validate_usage_ledger_v1(ledger: &AgentCandidateUsageLedgerV1) -> Result<(), String> {
    if ledger.ledger_version != USAGE_LEDGER_VERSION_V1
        || ledger.agent_id.is_empty()
        || ledger.session_digest.is_empty()
        || ledger.family_digest.is_empty()
        || ledger.entries.is_empty()
        || ledger.entries.iter().any(|entry| {
            entry.artifact_digest.is_empty()
                || entry
                    .range
                    .as_ref()
                    .is_some_and(|range| range.start > range.end)
                || entry.entry_digest != usage_entry_digest_v1(entry)
        })
        || ledger.historical_test_row_reads != 0
        || ledger.historical_test_label_reads != 0
        || ledger.historical_test_inference_count != 0
        || ledger.historical_test_metric_count != 0
        || ledger.historical_test_checkpoint_selection_count != 0
        || ledger.historical_test_identity_influence
        || ledger.ledger_digest != usage_ledger_digest_v1(ledger)
    {
        return Err("V1 candidate usage ledger rejected".to_string());
    }
    Ok(())
}

fn candidate_usage_ledger_v1(
    session: &AgentPrivateLearningSessionV1,
    projection: &AgentTrainerInputProjectionV1,
    family: &AgentCandidateFamilyV1,
    training_range: &IndexRangeV0,
    purge_range: &IndexRangeV0,
    validation_range: &IndexRangeV0,
    reserved_range: &IndexRangeV0,
) -> AgentCandidateUsageLedgerV1 {
    let mut entries = Vec::new();
    let mut push = |artifact_digest: String,
                    range: Option<IndexRangeV0>,
                    use_kind: CandidateEvidenceUseV1,
                    labels_read: bool,
                    parameters_updated: bool| {
        let mut entry = CandidateEvidenceUsageEntryV1 {
            artifact_digest,
            range,
            use_kind,
            labels_read,
            parameters_updated,
            entry_digest: String::new(),
        };
        entry.entry_digest = usage_entry_digest_v1(&entry);
        entries.push(entry);
    };
    for digest in &session.source_artifact_digests {
        push(
            digest.clone(),
            None,
            CandidateEvidenceUseV1::ViewBinding,
            false,
            false,
        );
    }
    for digest in &projection.consumed_artifact_digests {
        push(
            digest.clone(),
            None,
            CandidateEvidenceUseV1::TrainerProjection,
            false,
            false,
        );
    }
    if let Some(primary) = &projection.primary_series_digest {
        for (range, use_kind, labels_read, parameters_updated) in [
            (
                training_range,
                CandidateEvidenceUseV1::FeatureDerivation,
                false,
                false,
            ),
            (
                training_range,
                CandidateEvidenceUseV1::LabelDerivation,
                true,
                false,
            ),
            (
                training_range,
                CandidateEvidenceUseV1::NormalizerFit,
                false,
                false,
            ),
            (
                training_range,
                CandidateEvidenceUseV1::ParameterTraining,
                true,
                true,
            ),
            (
                validation_range,
                CandidateEvidenceUseV1::FeatureDerivation,
                false,
                false,
            ),
            (
                validation_range,
                CandidateEvidenceUseV1::LabelDerivation,
                true,
                false,
            ),
            (
                validation_range,
                CandidateEvidenceUseV1::ValidationInference,
                false,
                false,
            ),
            (
                validation_range,
                CandidateEvidenceUseV1::ValidationMetric,
                true,
                false,
            ),
            (
                purge_range,
                CandidateEvidenceUseV1::ReferencedButUnconsumed,
                false,
                false,
            ),
            (
                reserved_range,
                CandidateEvidenceUseV1::ReservedRetrospectiveUnused,
                false,
                false,
            ),
        ] {
            push(
                primary.clone(),
                Some(range.clone()),
                use_kind,
                labels_read,
                parameters_updated,
            );
        }
    }
    for digest in &projection.referenced_unconsumed_artifact_digests {
        push(
            digest.clone(),
            None,
            CandidateEvidenceUseV1::ReferencedButUnconsumed,
            false,
            false,
        );
    }
    for participant in &family.participants {
        push(
            participant.participant_digest.clone(),
            None,
            CandidateEvidenceUseV1::FamilyInclusion,
            false,
            false,
        );
    }
    let mut ledger = AgentCandidateUsageLedgerV1 {
        ledger_version: USAGE_LEDGER_VERSION_V1.to_string(),
        agent_id: session.agent_id.clone(),
        session_digest: session.session_digest.clone(),
        family_digest: family.family_digest.clone(),
        entries,
        historical_test_row_reads: 0,
        historical_test_label_reads: 0,
        historical_test_inference_count: 0,
        historical_test_metric_count: 0,
        historical_test_checkpoint_selection_count: 0,
        historical_test_identity_influence: false,
        ledger_digest: String::new(),
    };
    ledger.ledger_digest = usage_ledger_digest_v1(&ledger);
    ledger
}

fn zero_agent_learning_safety_counters_v1() -> AgentLearningSafetyCountersV1 {
    AgentLearningSafetyCountersV1 {
        active_committee_count: 3,
        network_requests: 0,
        credential_reads: 0,
        prospective_row_reads: 0,
        prospective_label_reads: 0,
        prospective_mutations: 0,
        historical_test_reads_v1: 0,
        active_model_changes: 0,
        chair_decisions: 0,
        votes: 0,
        rewards: 0,
        penalties: 0,
        voice_changes: 0,
        promotions: 0,
        executions: 0,
    }
}

fn candidate_families_report_digest_v1(report: &AgentCandidateFamiliesReportV1) -> String {
    stable_hash_string(&format!(
        "{}:{:?}:{:?}:{:?}:{}:{}:{}",
        report.report_version,
        report.mode,
        report
            .results
            .iter()
            .map(|result| (
                result.agent_id.as_str(),
                result.status,
                result
                    .family
                    .as_ref()
                    .map(|family| family.family_digest.as_str()),
                result
                    .usage_ledger
                    .as_ref()
                    .map(|ledger| ledger.ledger_digest.as_str())
            ))
            .collect::<Vec<_>>(),
        report.safety_counters,
        report.active_state_unchanged,
        report.duplicate_artifact_count,
        report.storage_failure_count
    ))
}

fn protected_reservation_digest_v1(reservation: &ProtectedEvaluationReservationV1) -> String {
    stable_hash_string(&format!(
        "protected-evaluation-reservation-v1:{:?}:{:?}:{}:{}",
        reservation.protected_registration_digests,
        reservation.reserved_timestamp_ms,
        reservation.cadence_ms,
        reservation.provider_finality_boundary_ms
    ))
}

fn validate_protected_reservation_v1(
    reservation: &ProtectedEvaluationReservationV1,
) -> Result<(), String> {
    let mut registrations = reservation.protected_registration_digests.clone();
    registrations.sort();
    registrations.dedup();
    let mut timestamps = reservation.reserved_timestamp_ms.clone();
    timestamps.sort();
    timestamps.dedup();
    if registrations.is_empty()
        || registrations != reservation.protected_registration_digests
        || registrations.iter().any(String::is_empty)
        || timestamps.is_empty()
        || timestamps != reservation.reserved_timestamp_ms
        || timestamps.contains(&0)
        || reservation.cadence_ms == 0
        || timestamps.windows(2).any(|pair| {
            pair[1]
                .checked_sub(pair[0])
                .is_none_or(|delta| delta % reservation.cadence_ms != 0)
        })
        || timestamps
            .last()
            .and_then(|timestamp| timestamp.checked_add(reservation.cadence_ms))
            .is_none_or(|boundary| boundary > reservation.provider_finality_boundary_ms)
        || reservation.reservation_digest != protected_reservation_digest_v1(reservation)
    {
        return Err("protected evaluation reservation rejected".to_string());
    }
    Ok(())
}

fn evaluation_exclusion_digest_v1(exclusion: &EvaluationEvidenceExclusionV1) -> String {
    stable_hash_string(&format!(
        "{}:{:?}:{:?}:{:?}",
        EXCLUSION_VERSION_V1,
        exclusion.protected_registration_digests,
        exclusion.excluded_timestamp_ms,
        exclusion.excluded_range_digests
    ))
}

fn validate_evaluation_exclusion_v1(
    exclusion: &EvaluationEvidenceExclusionV1,
) -> Result<(), String> {
    let sorted_unique_strings = |values: &[String]| {
        let mut expected = values.to_vec();
        expected.sort();
        expected.dedup();
        expected == values && !expected.is_empty() && expected.iter().all(|value| !value.is_empty())
    };
    let mut timestamps = exclusion.excluded_timestamp_ms.clone();
    timestamps.sort();
    timestamps.dedup();
    if !sorted_unique_strings(&exclusion.protected_registration_digests)
        || !sorted_unique_strings(&exclusion.excluded_range_digests)
        || timestamps.is_empty()
        || timestamps != exclusion.excluded_timestamp_ms
        || timestamps.contains(&0)
        || exclusion.exclusion_digest != evaluation_exclusion_digest_v1(exclusion)
    {
        return Err("V1 evaluation evidence exclusion rejected".to_string());
    }
    Ok(())
}

fn evaluation_registration_digest_v1(
    registration: &AgentCandidateEvaluationRegistrationV1,
) -> String {
    stable_hash_string(&format!(
        "{:?}",
        (
            (
                registration.registration_version.as_str(),
                registration.agent_id.as_str(),
                registration.family_digest.as_str(),
                registration.session_digest.as_str(),
                registration.usage_ledger_digest.as_str(),
                &registration.participant_digests,
                &registration.qualification_receipt_digests,
                registration.exclusion_digest.as_str(),
                registration.minimum_accepted_timestamp_ms,
            ),
            (
                &registration.required_dataset_kinds,
                registration.source_policy_digest.as_str(),
                registration.finality_policy_digest.as_str(),
                registration.label_policy_digest.as_str(),
                registration.metric_policy_digest.as_str(),
                registration.support_policy_digest.as_str(),
                registration.minimum_future_rows,
                registration.minimum_mature_events,
                registration.maximum_requests,
                registration.maximum_concurrency,
            ),
            (
                registration.maximum_retries,
                registration.labels_hidden_until_opening,
                registration.probabilities_hidden_until_opening,
                registration.one_time_opening_required,
                registration.winner_selection_forbidden_before_opening,
                registration.active_promotion_forbidden,
                registration.reward_application_forbidden,
                registration.status,
            )
        )
    ))
}

fn validate_evaluation_registration_v1(
    registration: &AgentCandidateEvaluationRegistrationV1,
) -> Result<(), String> {
    let mut participants = registration.participant_digests.clone();
    participants.sort();
    participants.dedup();
    let mut receipts = registration.qualification_receipt_digests.clone();
    receipts.sort();
    receipts.dedup();
    let mut kinds = registration.required_dataset_kinds.clone();
    kinds.sort();
    kinds.dedup();
    if registration.registration_version != EVALUATION_REGISTRATION_VERSION_V1
        || registration.agent_id.is_empty()
        || registration.family_digest.is_empty()
        || registration.session_digest.is_empty()
        || registration.usage_ledger_digest.is_empty()
        || registration.participant_digests != participants
        || participants.len() < 2
        || registration.qualification_receipt_digests != receipts
        || receipts.len() != participants.len()
        || registration.exclusion_digest.is_empty()
        || registration.minimum_accepted_timestamp_ms == 0
        || registration.required_dataset_kinds != kinds
        || kinds.is_empty()
        || registration.source_policy_digest.is_empty()
        || registration.finality_policy_digest.is_empty()
        || registration.label_policy_digest.is_empty()
        || registration.metric_policy_digest.is_empty()
        || registration.support_policy_digest.is_empty()
        || registration.minimum_future_rows == 0
        || registration.minimum_mature_events == 0
        || registration.maximum_requests != 1
        || registration.maximum_concurrency != 1
        || registration.maximum_retries != 0
        || !registration.labels_hidden_until_opening
        || !registration.probabilities_hidden_until_opening
        || !registration.one_time_opening_required
        || !registration.winner_selection_forbidden_before_opening
        || !registration.active_promotion_forbidden
        || !registration.reward_application_forbidden
        || registration.status != CandidateEvaluationRegistrationStatusV1::Registered
        || registration.registration_digest != evaluation_registration_digest_v1(registration)
    {
        return Err("V1 candidate evaluation registration rejected".to_string());
    }
    Ok(())
}

fn evaluation_journal_digest_v1(journal: &AgentCandidateEvaluationRegistrationJournalV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{:?}",
        journal.journal_version, journal.agent_id, journal.family_digest, journal.entries
    ))
}

fn candidate_evaluations_report_digest_v1(report: &AgentCandidateEvaluationsReportV1) -> String {
    stable_hash_string(&format!(
        "{}:{:?}:{:?}:{:?}:{}:{}:{}",
        report.report_version,
        report.mode,
        report
            .results
            .iter()
            .map(|result| (
                result.agent_id.as_str(),
                result.status,
                result
                    .registration
                    .as_ref()
                    .map(|registration| registration.registration_digest.as_str()),
                result
                    .exclusion
                    .as_ref()
                    .map(|exclusion| exclusion.exclusion_digest.as_str())
            ))
            .collect::<Vec<_>>(),
        report.safety_counters,
        report.active_state_unchanged,
        report.duplicate_artifact_count,
        report.storage_failure_count
    ))
}

fn session_digest_v0(session: &AgentPrivateLearningSessionV0) -> String {
    if session.session_version == SESSION_VERSION_V0 {
        return stable_hash_string(&format!(
            "{}:{}:{}:{:?}:{}:{}:{}:{}:{:?}:{}:{}:{}:{}:{:?}:{:?}",
            session.session_version,
            session.session_id,
            session.agent_id,
            session.agent_kind,
            session.intent_digest,
            session.data_view_digest,
            session.trainer_capability_digest,
            session.information_cutoff_ms,
            session.source_artifact_digests,
            session.feature_policy_digest,
            session.label_policy_digest,
            session.curriculum_policy_digest,
            session.private_namespace_digest,
            session.parent_model_version,
            session.session_status
        ));
    }
    stable_hash_string(&format!(
        "{}:{}:{}:{:?}:{}:{}:{}:{}:{:?}:{:?}:{:?}:{:?}:{}:{:?}:{}:{:?}:{}:{}:{}:{}:{}:{}:{:?}:{:?}:{:?}",
        session.session_version,
        session.session_id,
        session.agent_id,
        session.agent_kind,
        session.intent_digest,
        session.data_view_digest,
        session.trainer_capability_digest,
        session.information_cutoff_ms,
        session.required_dataset_kinds,
        session.optional_dataset_kinds,
        session.allowed_markets,
        session.symbols,
        session.cadence,
        session.lookback,
        session.maximum_staleness_ms,
        session.source_artifact_digests,
        session.source_policy_digest,
        session.feature_policy_digest,
        session.label_policy_digest,
        session.curriculum_policy_digest,
        session.private_namespace_digest,
        session.training_ledger_digest,
        session.trainer_projection_digest,
        session.parent_model_version,
        session.session_status
    ))
}

fn projection_digest_v0(projection: &AgentTrainerInputProjectionV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}:{}:{:?}:{:?}:{:?}:{:?}:{}",
        projection.projection_version,
        projection.agent_id,
        projection.trainer_kind,
        projection.source_view_digest,
        projection.consumed_artifact_digests,
        projection.referenced_but_unconsumed_artifact_digests,
        projection.primary_series_digest,
        projection.auxiliary_series_digests,
        projection.projection_policy_digest
    ))
}

fn evidence_usage_entry_digest_v0(entry: &CandidateEvidenceUsageEntryV0) -> String {
    stable_hash_string(&format!(
        "{}:{:?}:{:?}:{}:{}:{}:{}",
        entry.artifact_digest,
        entry.range,
        entry.use_kind,
        entry.labels_read,
        entry.parameters_updated,
        entry.checkpoint_selection_influenced,
        entry.candidate_identity_influenced
    ))
}

fn evidence_usage_ledger_digest_v0(ledger: &CandidateEvidenceUsageLedgerV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{:?}",
        ledger.ledger_version,
        ledger.agent_id,
        ledger.candidate_digest,
        ledger.session_digest,
        ledger
            .entries
            .iter()
            .map(|entry| entry.entry_digest.as_str())
            .collect::<Vec<_>>()
    ))
}

fn identity_audit_digest_v0(audit: &AgentCandidateIdentityAuditV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}:{:?}:{}:{:?}:{}:{}:{}",
        audit.audit_version,
        audit.candidate_digest,
        audit.model_identity_inputs,
        audit.metric_identity_inputs,
        audit.test_evidence_in_identity,
        audit.historical_test_status,
        audit.eligible_for_fresh_historical_test,
        audit.eligible_for_future_evaluation_registration,
        audit.superseded_by_input_binding_hardening
    ))
}

fn evaluation_registration_digest_v0(
    registration: &AgentCandidateEvaluationRegistrationV0,
) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{:?}:{}:{}:{}:{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}",
        registration.registration_version,
        registration.agent_id,
        registration.candidate_digest,
        registration.session_digest,
        registration.evidence_usage_ledger_digest,
        registration.identity_audit_digest,
        registration.evaluation_cutoff_exclusive_ms,
        registration.required_dataset_kinds,
        registration.source_policy_digest,
        registration.finality_policy_digest,
        registration.label_policy_digest,
        registration.metric_policy_digest,
        registration.support_policy_digest,
        registration.comparator_digests,
        registration.minimum_future_rows,
        registration.minimum_mature_events,
        registration.maximum_requests,
        registration.maximum_concurrency,
        registration.maximum_retries,
        registration.labels_hidden_until_opening,
        registration.probabilities_hidden_until_opening,
        registration.one_time_opening_required,
        registration.active_promotion_forbidden,
        registration.reward_application_forbidden,
        registration.status
    ))
}

fn evaluation_journal_digest_v0(journal: &AgentCandidateEvaluationRegistrationJournalV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{:?}",
        journal.journal_version, journal.agent_id, journal.candidate_digest, journal.entries
    ))
}

fn candidate_evaluation_report_digest_v0(report: &AgentCandidateEvaluationReportV0) -> String {
    stable_hash_string(&format!(
        "{}:{:?}:{}:{:?}:{:?}:{}:{}:{}",
        report.report_version,
        report.mode,
        report.registration_requested,
        report
            .results
            .iter()
            .map(|result| (
                result.agent_id.as_str(),
                result.candidate_digest.as_deref(),
                result
                    .evidence_usage_ledger
                    .as_ref()
                    .map(|ledger| ledger.ledger_digest.as_str()),
                result
                    .identity_audit
                    .as_ref()
                    .map(|audit| audit.audit_digest.as_str()),
                result
                    .evaluation_registration
                    .as_ref()
                    .map(|registration| registration.registration_digest.as_str()),
                result.blocked_status,
            ))
            .collect::<Vec<_>>(),
        report.safety_counters,
        report.active_state_unchanged,
        report.duplicate_artifact_count,
        report.storage_failure_count
    ))
}

fn dataset_manifest_digest_v0(manifest: &AgentPrivateDatasetManifestV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{:?}:{:?}:{}:{}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{}:{}:{}:{}:{}:{}:{}:{}",
        manifest.dataset_version,
        manifest.dataset_id,
        manifest.agent_id,
        manifest.session_id,
        manifest.data_view_digest,
        manifest.source_artifact_digests,
        manifest.dataset_kinds,
        manifest.information_cutoff_ms,
        manifest.row_count,
        manifest.training_range,
        manifest.first_purge_range,
        manifest.validation_range,
        manifest.second_purge_range,
        manifest.sealed_test_range,
        manifest.normalizer_fit_range,
        manifest.validation_parameter_update_count,
        manifest.test_checkpoint_selection_count,
        manifest.prospective_row_read_count,
        manifest.prospective_label_read_count,
        manifest.feature_artifact_digest,
        manifest.label_artifact_digest,
        manifest.normalizer_digest,
        manifest.data_view_digest
    ))
}

fn candidate_digest_v0(candidate: &AgentSandboxLearningCandidateV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{:?}:{}:{}:{}:{}",
        candidate.candidate_version,
        candidate.agent_id,
        candidate.session_digest,
        candidate.data_view_digest,
        candidate.parent_model_version,
        candidate.model_artifact_digest,
        candidate.feature_policy_digest,
        candidate.label_policy_digest,
        candidate.normalizer_digest,
        candidate.training_policy_digest,
        candidate.private_metrics_digest,
        candidate.deployment_status,
        candidate.retrospective_research_only,
        candidate.eligible_for_active_committee,
        candidate.eligible_for_promotion,
        candidate.eligible_for_reward
    ))
}

fn journal_digest_v0(journal: &AgentLearningSessionJournalV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}",
        journal.journal_version, journal.agent_id, journal.entries
    ))
}

fn report_digest_v0(report: &AgentPrivateLearningSessionsReportV0) -> String {
    stable_hash_string(&format!(
        "{}:{:?}:{}:{:?}:{:?}:{}:{}:{}",
        report.report_version,
        report.mode,
        report.capability_registry.registry_digest,
        report
            .results
            .iter()
            .map(|result| (
                &result.session.session_digest,
                result
                    .candidate
                    .as_ref()
                    .map(|candidate| &candidate.candidate_digest)
            ))
            .collect::<Vec<_>>(),
        report.safety_counters,
        report.active_state_unchanged,
        report.duplicate_artifact_count,
        report.storage_failure_count
    ))
}

fn session_status_code_v0(status: AgentLearningSessionStatusV0) -> &'static str {
    match status {
        AgentLearningSessionStatusV0::Registered => "registered",
        AgentLearningSessionStatusV0::DatasetReady => "dataset_ready",
        AgentLearningSessionStatusV0::CandidateProduced => "candidate_produced",
        AgentLearningSessionStatusV0::InsufficientEvidence => "insufficient_evidence",
        AgentLearningSessionStatusV0::TrainerUnavailable => "trainer_unavailable",
        AgentLearningSessionStatusV0::RejectedUnauthorizedEvidence => "unauthorized_evidence",
        AgentLearningSessionStatusV0::RejectedCutoffLeakage => "cutoff_leakage",
        AgentLearningSessionStatusV0::RejectedLabelLeakage => "label_leakage",
        AgentLearningSessionStatusV0::RejectedSafetyInvariant => "safety_invariant",
        AgentLearningSessionStatusV0::TechnicalFailure => "technical_failure",
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use super::*;
    use crate::{
        core::ReasonCode,
        data::{
            SnapshotCompatibilityV1, SnapshotProvenance, SnapshotQualitySummary,
            snapshot_id_from_semantic_digest_v1,
        },
        league::HistoricalReplayDataset,
    };

    fn snapshot_for(
        dataset_kind: DatasetKind,
        lookback_bars: usize,
        maximum_staleness_ms: u64,
        salt: usize,
    ) -> DataSnapshot {
        let rows = 360;
        let normalized_dataset = HistoricalReplayDataset {
            symbol: "SPY".to_string(),
            rows: (0..rows)
                .map(|index| {
                    let price = 100.0
                        + salt as f64 * 0.01
                        + index as f64 * 0.07
                        + (index % 11) as f64 * 0.8;
                    HistoricalOhlcvRow {
                        symbol: "SPY".to_string(),
                        timestamp_ms: index as u64 + 1,
                        open: price,
                        high: price * 1.01,
                        low: price * if index % 7 == 0 { 0.965 } else { 0.99 },
                        close: price,
                        volume: 1_000.0 + (index % 17) as f64 * 30.0,
                        trade_value: Some(price * 1_000.0),
                    }
                })
                .collect(),
            source: "approved-sanitized-history".to_string(),
            reason_codes: vec![],
        };
        let digest = historical_replay_dataset_digest_v0(&normalized_dataset);
        DataSnapshot {
            snapshot_id: snapshot_id_from_semantic_digest_v1(&digest),
            request_key: format!("private-learning-test-{dataset_kind:?}-{lookback_bars}-{salt}"),
            provider_id: "approved-provider".to_string(),
            dataset_kind,
            market_scope: AcquisitionMarketScope::UsStocks,
            symbols: vec!["SPY".to_string()],
            requested_lookback: DataLookback {
                bars: lookback_bars,
                start_timestamp_ms: None,
                end_timestamp_ms: Some(rows as u64),
            },
            actual_start_timestamp_ms: Some(1),
            actual_end_timestamp_ms: Some(rows as u64),
            fetched_at_ms: rows as u64,
            normalized_at_ms: rows as u64,
            schema_version: 1,
            row_count: rows,
            quality_summary: SnapshotQualitySummary {
                accepted: true,
                row_count: rows,
                reason_codes: vec![],
            },
            content_digest: digest,
            sanitized: true,
            read_only: true,
            compatibility: Some(SnapshotCompatibilityV1 {
                cadence: "1d".to_string(),
                adjustment_semantics: expected_adjustment_semantics_v0(dataset_kind),
                source_schema: "application/x-soma-normalized-dataset".to_string(),
                requested_cutoff_timestamp_ms: Some(rows as u64),
                maximum_staleness_ms,
                all_rows_finalized: true,
            }),
            normalized_dataset,
            provenance: SnapshotProvenance {
                provider_id: "approved-provider".to_string(),
                acquisition_request_id: "private-learning-request".to_string(),
                fetch_receipt_id: "private-learning-receipt".to_string(),
                source_type: crate::data::SnapshotSourceType::ApprovedReadOnlyProvider,
                sanitized: true,
                credential_free: true,
                reason_codes: vec![],
            },
            reason_codes: vec![ReasonCode::DataSnapshotImmutable],
        }
    }

    fn snapshots() -> Vec<DataSnapshot> {
        vec![
            snapshot_for(DatasetKind::DailyOhlcv, 90, 86_400_000, 1),
            snapshot_for(DatasetKind::AdjustedDailyOhlcv, 90, 86_400_000, 2),
            snapshot_for(DatasetKind::VolatilityDaily, 90, 86_400_000, 3),
            snapshot_for(DatasetKind::LiquidityDaily, 90, 86_400_000, 4),
            snapshot_for(DatasetKind::AdjustedDailyOhlcv, 252, 7 * 86_400_000, 5),
            snapshot_for(DatasetKind::QuarterlyFundamentals, 252, 7 * 86_400_000, 6),
            snapshot_for(DatasetKind::ValuationMetrics, 252, 7 * 86_400_000, 7),
            snapshot_for(DatasetKind::CorporateActions, 252, 7 * 86_400_000, 8),
            snapshot_for(DatasetKind::MarketIndexDaily, 126, 86_400_000, 9),
            snapshot_for(DatasetKind::VolatilityDaily, 126, 86_400_000, 10),
            snapshot_for(DatasetKind::MarketBreadthDaily, 126, 86_400_000, 11),
            snapshot_for(DatasetKind::LiquidityDaily, 126, 86_400_000, 12),
            snapshot_for(DatasetKind::MacroSeries, 126, 86_400_000, 13),
        ]
    }

    fn inputs() -> Vec<AgentPrivateLearningSessionInputV0> {
        build_agent_private_learning_inputs_v0(&snapshots(), 360).unwrap()
    }

    fn result_for<'a>(
        report: &'a AgentPrivateLearningSessionsReportV0,
        agent_id: &str,
    ) -> &'a AgentPrivateLearningSessionResultV0 {
        report
            .results
            .iter()
            .find(|result| result.session.agent_id == agent_id)
            .unwrap()
    }

    fn execution_report() -> AgentPrivateLearningSessionsReportV0 {
        static REPORT: OnceLock<AgentPrivateLearningSessionsReportV0> = OnceLock::new();
        REPORT
            .get_or_init(|| {
                run_agent_private_learning_sessions_v0(
                    &inputs(),
                    AgentPrivateLearningRunModeV0::ExecuteLocal,
                )
            })
            .clone()
    }

    fn unique_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "soma-agent-private-learning-{name}-{}",
            std::process::id()
        ))
    }

    fn candidate_evaluation_fixture() -> (PathBuf, AgentCandidateEvaluationReportV0) {
        static FIXTURE: OnceLock<(PathBuf, AgentCandidateEvaluationReportV0)> = OnceLock::new();
        FIXTURE
            .get_or_init(|| {
                let root = unique_root("candidate-evaluation-fixture");
                let _ = fs::remove_dir_all(&root);
                let mut sessions = execution_report();
                let storage = persist_agent_private_learning_report_v0(&mut sessions, &root);
                assert_eq!(storage.failed_artifact_count, 0);
                let report = run_agent_candidate_evaluation_v0(
                    &root,
                    AgentPrivateLearningRunModeV0::DryRun,
                    true,
                    Some(500),
                );
                (root, report)
            })
            .clone()
    }

    fn evaluation_result_for<'a>(
        report: &'a AgentCandidateEvaluationReportV0,
        agent_id: &str,
    ) -> &'a AgentCandidateEvaluationResultV0 {
        report
            .results
            .iter()
            .find(|result| result.agent_id == agent_id)
            .unwrap()
    }

    #[test]
    fn trainer_registry_is_explicit_and_value_is_unavailable() {
        let registry = agent_trainer_capability_registry_v0();
        assert_eq!(registry.capabilities.len(), 3);
        assert!(registry.capabilities.iter().any(|capability| {
            capability.agent_id == "value_quality_filter"
                && capability.trainer_kind == AgentTrainerKindV0::ValueQualityUnavailable
                && !capability.supports_training
        }));
    }

    #[test]
    fn three_session_manifests_derive_independently() {
        let report = run_agent_private_learning_sessions_v0(
            &inputs(),
            AgentPrivateLearningRunModeV0::Status,
        );
        assert_eq!(report.results.len(), 3);
        assert_eq!(
            report
                .results
                .iter()
                .map(|result| result.session.session_digest.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
    }

    #[test]
    fn three_private_namespaces_are_distinct() {
        let inputs = inputs();
        assert_eq!(
            inputs
                .iter()
                .map(|input| input.view.private_namespace_digest.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
    }

    #[test]
    fn complete_persisted_view_is_consumed_without_replacement() {
        let snapshots = snapshots();
        let planned = inputs()
            .into_iter()
            .find(|input| input.intent.agent_id == "momentum_trend_fast")
            .unwrap();
        let persisted = build_agent_private_learning_input_from_persisted_view_v0(
            &planned.intent,
            &planned.policy,
            &planned.view,
            &snapshots,
        )
        .unwrap();
        assert_eq!(persisted.intent.intent_digest, planned.intent.intent_digest);
        assert_eq!(persisted.view, planned.view);
        let report = run_agent_private_learning_sessions_v0(
            &[persisted],
            AgentPrivateLearningRunModeV0::DryRun,
        );
        assert_eq!(
            report.results[0].session.session_status,
            AgentLearningSessionStatusV0::DatasetReady
        );
        assert_eq!(
            report.results[0]
                .trainer_projection
                .as_ref()
                .unwrap()
                .source_view_digest,
            planned.view.view_digest
        );
    }

    #[test]
    fn incompatible_larger_snapshot_cannot_replace_canonical_request() {
        let mut snapshots = snapshots();
        let expected = snapshots
            .iter()
            .find(|snapshot| snapshot.dataset_kind == DatasetKind::DailyOhlcv)
            .unwrap()
            .content_digest
            .clone();
        let mut incompatible = snapshots
            .iter()
            .find(|snapshot| snapshot.dataset_kind == DatasetKind::DailyOhlcv)
            .unwrap()
            .clone();
        incompatible.requested_lookback.bars = 10_000;
        incompatible.row_count = 10_000;
        snapshots.push(incompatible);
        let momentum = build_agent_private_learning_inputs_v0(&snapshots, 360)
            .unwrap()
            .into_iter()
            .find(|input| input.intent.agent_id == "momentum_trend_fast")
            .unwrap();
        assert!(momentum.view.source_artifact_digests.contains(&expected));
        assert_eq!(momentum.view.source_artifact_digests.len(), 4);
    }

    #[test]
    fn compatible_snapshot_may_be_fresher_than_request_policy() {
        let mut snapshot = snapshots()
            .into_iter()
            .find(|snapshot| snapshot.dataset_kind == DatasetKind::DailyOhlcv)
            .unwrap();
        let compatibility = snapshot.compatibility.as_ref().unwrap().clone();
        let request = ReadOnlyProviderRequest {
            request_id: "fresher-compatible-snapshot".into(),
            request_key: "fresher-compatible-snapshot".into(),
            provider_id: snapshot.provider_id.clone(),
            dataset_kind: snapshot.dataset_kind,
            market_scope: snapshot.market_scope,
            symbols: snapshot.symbols.clone(),
            lookback: snapshot.requested_lookback.clone(),
            cadence: compatibility.cadence,
            max_staleness_ms: compatibility.maximum_staleness_ms,
            reason_codes: vec![],
        };
        snapshot
            .compatibility
            .as_mut()
            .unwrap()
            .maximum_staleness_ms = 0;
        assert_eq!(
            validate_snapshot_for_request_v0(&snapshot, &request),
            Ok(true)
        );

        snapshot
            .compatibility
            .as_mut()
            .unwrap()
            .maximum_staleness_ms = request.max_staleness_ms + 1;
        assert_eq!(
            validate_snapshot_for_request_v0(&snapshot, &request),
            Ok(false)
        );
    }

    #[test]
    fn missing_required_dataset_blocks_only_affected_session() {
        let mut snapshots = snapshots();
        snapshots.retain(|snapshot| snapshot.dataset_kind != DatasetKind::MarketIndexDaily);
        let inputs = build_agent_private_learning_inputs_v0(&snapshots, 360).unwrap();
        let cycle = inputs
            .iter()
            .find(|input| input.intent.agent_id == "cycle_risk_skeptic")
            .unwrap();
        assert_eq!(
            cycle.resolution_status,
            AgentViewResolutionStatusV0::MissingRequiredEvidence
        );
        let report =
            run_agent_private_learning_sessions_v0(&inputs, AgentPrivateLearningRunModeV0::DryRun);
        assert_eq!(
            result_for(&report, "cycle_risk_skeptic")
                .session
                .session_status,
            AgentLearningSessionStatusV0::InsufficientEvidence
        );
        assert_eq!(
            result_for(&report, "momentum_trend_fast")
                .session
                .session_status,
            AgentLearningSessionStatusV0::DatasetReady
        );
    }

    #[test]
    fn optional_missing_evidence_is_not_fabricated() {
        let mut snapshots = snapshots();
        snapshots.retain(|snapshot| {
            snapshot.requested_lookback.bars != 90
                || snapshot.dataset_kind == DatasetKind::DailyOhlcv
        });
        let inputs = build_agent_private_learning_inputs_v0(&snapshots, 360).unwrap();
        let momentum = inputs
            .iter()
            .find(|input| input.intent.agent_id == "momentum_trend_fast")
            .unwrap();
        assert_eq!(
            momentum.resolution_status,
            AgentViewResolutionStatusV0::OptionalEvidenceUnavailable
        );
        assert_eq!(momentum.artifacts.len(), 1);
    }

    #[test]
    fn semantically_equivalent_artifacts_choose_unique_latest_deterministically() {
        let mut snapshots = snapshots();
        let mut latest = snapshots
            .iter()
            .find(|snapshot| snapshot.dataset_kind == DatasetKind::DailyOhlcv)
            .unwrap()
            .clone();
        latest.fetched_at_ms += 1;
        latest.normalized_dataset.rows[0].volume += 1.0;
        latest.content_digest = historical_replay_dataset_digest_v0(&latest.normalized_dataset);
        latest.snapshot_id = snapshot_id_from_semantic_digest_v1(&latest.content_digest);
        let expected = latest.content_digest.clone();
        snapshots.push(latest);
        let first = build_agent_private_learning_inputs_v0(&snapshots, 360).unwrap();
        let second = build_agent_private_learning_inputs_v0(&snapshots, 360).unwrap();
        let primary = |inputs: &[AgentPrivateLearningSessionInputV0]| {
            inputs
                .iter()
                .find(|input| input.intent.agent_id == "momentum_trend_fast")
                .unwrap()
                .artifacts
                .iter()
                .find(|artifact| artifact.snapshot.dataset_kind == DatasetKind::DailyOhlcv)
                .unwrap()
                .artifact_ref
                .artifact_digest
                .clone()
        };
        assert_eq!(primary(&first), expected);
        assert_eq!(primary(&first), primary(&second));
    }

    #[test]
    fn unresolved_equivalent_artifact_tie_fails_closed() {
        let mut snapshots = snapshots();
        let mut tied = snapshots
            .iter()
            .find(|snapshot| snapshot.dataset_kind == DatasetKind::DailyOhlcv)
            .unwrap()
            .clone();
        tied.normalized_dataset.rows[0].volume += 1.0;
        tied.content_digest = historical_replay_dataset_digest_v0(&tied.normalized_dataset);
        tied.snapshot_id = snapshot_id_from_semantic_digest_v1(&tied.content_digest);
        snapshots.push(tied);
        let momentum = build_agent_private_learning_inputs_v0(&snapshots, 360)
            .unwrap()
            .into_iter()
            .find(|input| input.intent.agent_id == "momentum_trend_fast")
            .unwrap();
        assert_eq!(
            momentum.resolution_status,
            AgentViewResolutionStatusV0::AmbiguousEquivalentArtifacts
        );
    }

    #[test]
    fn heterogeneous_evidence_is_projected_without_concatenation() {
        let report = run_agent_private_learning_sessions_v0(
            &inputs(),
            AgentPrivateLearningRunModeV0::DryRun,
        );
        let momentum = result_for(&report, "momentum_trend_fast");
        let projection = momentum.trainer_projection.as_ref().unwrap();
        assert_eq!(projection.consumed_artifact_digests.len(), 1);
        assert_eq!(
            projection.referenced_but_unconsumed_artifact_digests.len(),
            3
        );
        assert_eq!(
            projection.source_view_digest,
            momentum.session.data_view_digest
        );
        assert_eq!(momentum.dataset_manifest.as_ref().unwrap().row_count, 360);
        let cycle = result_for(&report, "cycle_risk_skeptic");
        let cycle_projection = cycle.trainer_projection.as_ref().unwrap();
        let cycle_primary = cycle_projection.primary_series_digest.as_ref().unwrap();
        assert!(
            cycle
                .session
                .source_artifact_digests
                .contains(cycle_primary)
        );
        assert_eq!(cycle_projection.consumed_artifact_digests.len(), 1);
    }

    #[test]
    fn different_symbol_or_cadence_cannot_enter_projection() {
        let mut symbol_inputs = inputs();
        let momentum = symbol_inputs
            .iter_mut()
            .find(|input| input.intent.agent_id == "momentum_trend_fast")
            .unwrap();
        momentum.artifacts[0].snapshot.symbols = vec!["QQQ".to_string()];
        let symbol_report = run_agent_private_learning_sessions_v0(
            &symbol_inputs,
            AgentPrivateLearningRunModeV0::DryRun,
        );
        assert!(
            result_for(&symbol_report, "momentum_trend_fast")
                .trainer_projection
                .is_none()
        );

        let mut cadence_snapshots = snapshots();
        cadence_snapshots
            .iter_mut()
            .find(|snapshot| snapshot.dataset_kind == DatasetKind::DailyOhlcv)
            .unwrap()
            .compatibility
            .as_mut()
            .unwrap()
            .cadence = "1h".to_string();
        let cadence_inputs =
            build_agent_private_learning_inputs_v0(&cadence_snapshots, 360).unwrap();
        let cadence_momentum = cadence_inputs
            .iter()
            .find(|input| input.intent.agent_id == "momentum_trend_fast")
            .unwrap();
        assert_eq!(
            cadence_momentum.resolution_status,
            AgentViewResolutionStatusV0::MissingRequiredEvidence
        );
    }

    #[test]
    fn source_digest_mutation_rejects_only_that_session() {
        let mut inputs = inputs();
        let momentum = inputs
            .iter_mut()
            .find(|input| input.intent.agent_id == "momentum_trend_fast")
            .unwrap();
        momentum.artifacts[0].snapshot.content_digest = "0000000000000000".to_string();
        let report =
            run_agent_private_learning_sessions_v0(&inputs, AgentPrivateLearningRunModeV0::DryRun);
        assert_eq!(
            result_for(&report, "momentum_trend_fast")
                .session
                .session_status,
            AgentLearningSessionStatusV0::RejectedUnauthorizedEvidence
        );
        assert_eq!(
            result_for(&report, "cycle_risk_skeptic")
                .session
                .session_status,
            AgentLearningSessionStatusV0::DatasetReady
        );
    }

    #[test]
    fn unauthorized_dataset_rejects() {
        let mut inputs = inputs();
        let momentum = inputs
            .iter_mut()
            .find(|input| input.intent.agent_id == "momentum_trend_fast")
            .unwrap();
        momentum.artifacts[0].snapshot.dataset_kind = DatasetKind::QuarterlyFundamentals;
        let report =
            run_agent_private_learning_sessions_v0(&inputs, AgentPrivateLearningRunModeV0::DryRun);
        assert_eq!(
            result_for(&report, "momentum_trend_fast")
                .session
                .session_status,
            AgentLearningSessionStatusV0::RejectedUnauthorizedEvidence
        );
    }

    #[test]
    fn actual_row_beyond_cutoff_rejects() {
        let mut snapshots = snapshots();
        let snapshot = snapshots
            .iter_mut()
            .find(|snapshot| {
                snapshot.dataset_kind == DatasetKind::DailyOhlcv
                    && snapshot.requested_lookback.bars == 90
            })
            .unwrap();
        snapshot
            .normalized_dataset
            .rows
            .last_mut()
            .unwrap()
            .timestamp_ms = 361;
        snapshot.actual_end_timestamp_ms = Some(361);
        snapshot.content_digest = historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
        snapshot.snapshot_id = snapshot_id_from_semantic_digest_v1(&snapshot.content_digest);
        let inputs = build_agent_private_learning_inputs_v0(&snapshots, 360).unwrap();
        let report =
            run_agent_private_learning_sessions_v0(&inputs, AgentPrivateLearningRunModeV0::DryRun);
        assert_eq!(
            result_for(&report, "momentum_trend_fast")
                .session
                .session_status,
            AgentLearningSessionStatusV0::RejectedCutoffLeakage
        );
    }

    #[test]
    fn cross_agent_private_artifact_rejects() {
        let mut inputs = inputs();
        let momentum = inputs
            .iter_mut()
            .find(|input| input.intent.agent_id == "momentum_trend_fast")
            .unwrap();
        momentum.artifacts[0].artifact_ref.visibility =
            LearningDataVisibilityV0::AgentPrivateDerived;
        momentum.artifacts[0].artifact_ref.owner_agent_id = Some("cycle_risk_skeptic".to_string());
        let report =
            run_agent_private_learning_sessions_v0(&inputs, AgentPrivateLearningRunModeV0::DryRun);
        assert_eq!(
            result_for(&report, "momentum_trend_fast")
                .session
                .session_status,
            AgentLearningSessionStatusV0::RejectedUnauthorizedEvidence
        );
    }

    #[test]
    fn duplicate_timestamp_rejects() {
        let mut snapshots = snapshots();
        let snapshot = snapshots
            .iter_mut()
            .find(|snapshot| {
                snapshot.dataset_kind == DatasetKind::DailyOhlcv
                    && snapshot.requested_lookback.bars == 90
            })
            .unwrap();
        snapshot.normalized_dataset.rows[1].timestamp_ms = 1;
        snapshot.content_digest = historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
        snapshot.snapshot_id = snapshot_id_from_semantic_digest_v1(&snapshot.content_digest);
        let inputs = build_agent_private_learning_inputs_v0(&snapshots, 360).unwrap();
        let report =
            run_agent_private_learning_sessions_v0(&inputs, AgentPrivateLearningRunModeV0::DryRun);
        assert_eq!(
            result_for(&report, "momentum_trend_fast")
                .sanitized_error_code
                .as_deref(),
            Some("integrity_failure")
        );
    }

    #[test]
    fn non_finite_row_rejects() {
        let mut snapshots = snapshots();
        let snapshot = snapshots
            .iter_mut()
            .find(|snapshot| {
                snapshot.dataset_kind == DatasetKind::MarketIndexDaily
                    && snapshot.requested_lookback.bars == 126
            })
            .unwrap();
        snapshot.normalized_dataset.rows[10].volume = f64::NAN;
        snapshot.content_digest = historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
        snapshot.snapshot_id = snapshot_id_from_semantic_digest_v1(&snapshot.content_digest);
        let inputs = build_agent_private_learning_inputs_v0(&snapshots, 360).unwrap();
        let report =
            run_agent_private_learning_sessions_v0(&inputs, AgentPrivateLearningRunModeV0::DryRun);
        assert_eq!(
            result_for(&report, "cycle_risk_skeptic")
                .session
                .session_status,
            AgentLearningSessionStatusV0::RejectedSafetyInvariant
        );
    }

    #[test]
    fn train_validation_leakage_manifest_rejects() {
        let report = run_agent_private_learning_sessions_v0(
            &inputs(),
            AgentPrivateLearningRunModeV0::DryRun,
        );
        let mut manifest = result_for(&report, "momentum_trend_fast")
            .dataset_manifest
            .clone()
            .unwrap();
        manifest.first_purge_range.end = manifest.first_purge_range.start;
        manifest.validation_range.start = manifest.training_range.end;
        assert!(validate_dataset_manifest_v0(&manifest).is_err());
    }

    #[test]
    fn normalizer_fit_range_is_training_only() {
        let report = run_agent_private_learning_sessions_v0(
            &inputs(),
            AgentPrivateLearningRunModeV0::DryRun,
        );
        for manifest in report
            .results
            .iter()
            .filter_map(|result| result.dataset_manifest.as_ref())
        {
            assert_eq!(manifest.normalizer_fit_range, manifest.training_range);
            assert_eq!(manifest.validation_parameter_update_count, 0);
        }
    }

    #[test]
    fn momentum_uses_registered_frozen_mamba_head_trainer() {
        let report = execution_report();
        let result = result_for(&report, "momentum_trend_fast");
        assert_eq!(
            result.trainer_kind,
            AgentTrainerKindV0::MomentumFrozenMambaHead
        );
        assert_eq!(
            result.session.session_status,
            AgentLearningSessionStatusV0::CandidateProduced
        );
        assert!(result.candidate.is_some());
    }

    #[test]
    fn cycle_risk_uses_registered_independent_trainer() {
        let report = execution_report();
        let result = result_for(&report, "cycle_risk_skeptic");
        assert_eq!(
            result.trainer_kind,
            AgentTrainerKindV0::CycleRiskIndependentShadow
        );
        assert_eq!(
            result.session.session_status,
            AgentLearningSessionStatusV0::CandidateProduced
        );
        assert!(result.candidate.is_some());
    }

    #[test]
    fn value_returns_trainer_unavailable_without_candidate() {
        let report = execution_report();
        let result = result_for(&report, "value_quality_filter");
        assert_eq!(
            result.session.session_status,
            AgentLearningSessionStatusV0::TrainerUnavailable
        );
        assert!(result.candidate.is_none());
    }

    #[test]
    fn one_agent_failure_does_not_stop_other_sessions() {
        let mut inputs = inputs();
        inputs
            .iter_mut()
            .find(|input| input.intent.agent_id == "momentum_trend_fast")
            .unwrap()
            .artifacts[0]
            .snapshot
            .read_only = false;
        let report =
            run_agent_private_learning_sessions_v0(&inputs, AgentPrivateLearningRunModeV0::DryRun);
        assert_eq!(report.results.len(), 3);
        assert_eq!(
            result_for(&report, "cycle_risk_skeptic")
                .session
                .session_status,
            AgentLearningSessionStatusV0::DatasetReady
        );
        assert_eq!(
            result_for(&report, "value_quality_filter")
                .session
                .session_status,
            AgentLearningSessionStatusV0::TrainerUnavailable
        );
    }

    #[test]
    fn candidates_are_retrospective_shadow_only() {
        let report = execution_report();
        assert!(
            report
                .results
                .iter()
                .filter_map(|result| result.candidate.as_ref())
                .all(|candidate| candidate.deployment_status
                    == ModelAgentDeploymentStatus::ShadowOnly
                    && candidate.retrospective_research_only)
        );
    }

    #[test]
    fn candidates_cannot_enter_active_committee() {
        let report = execution_report();
        assert!(
            report
                .results
                .iter()
                .filter_map(|result| result.candidate.as_ref())
                .all(|candidate| !candidate.eligible_for_active_committee
                    && !candidate.eligible_for_promotion)
        );
    }

    #[test]
    fn candidates_cannot_receive_reward() {
        let report = execution_report();
        assert!(
            report
                .results
                .iter()
                .filter_map(|result| result.candidate.as_ref())
                .all(|candidate| !candidate.eligible_for_reward)
        );
    }

    #[test]
    fn protobuf_round_trips_all_machine_artifacts_semantically() {
        let report = execution_report();
        let momentum = result_for(&report, "momentum_trend_fast");
        let session_bytes = encode_session_protobuf_v0(&momentum.session).unwrap();
        assert_eq!(
            decode_session_protobuf_v0(&session_bytes).unwrap(),
            momentum.session
        );
        let projection = momentum.trainer_projection.as_ref().unwrap();
        let projection_bytes = encode_trainer_projection_protobuf_v0(projection).unwrap();
        assert_eq!(
            decode_trainer_projection_protobuf_v0(&projection_bytes).unwrap(),
            *projection
        );
        let manifest = momentum.dataset_manifest.as_ref().unwrap();
        let manifest_bytes = encode_dataset_manifest_protobuf_v0(manifest).unwrap();
        assert_eq!(
            decode_dataset_manifest_protobuf_v0(&manifest_bytes).unwrap(),
            *manifest
        );
        let candidate = momentum.candidate.as_ref().unwrap();
        let candidate_bytes = encode_candidate_protobuf_v0(candidate).unwrap();
        assert_eq!(
            decode_candidate_protobuf_v0(&candidate_bytes).unwrap(),
            *candidate
        );
        let journal_bytes = encode_journal_protobuf_v0(&momentum.journal).unwrap();
        assert_eq!(
            decode_journal_protobuf_v0(&journal_bytes).unwrap(),
            momentum.journal
        );
        let registry_bytes =
            encode_capability_registry_protobuf_v0(&report.capability_registry).unwrap();
        assert_eq!(
            decode_capability_registry_protobuf_v0(&registry_bytes).unwrap(),
            report.capability_registry
        );
    }

    #[test]
    fn corrupt_protobuf_envelope_rejects() {
        let report = execution_report();
        let mut bytes = encode_session_protobuf_v0(&report.results[0].session).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        assert!(decode_session_protobuf_v0(&bytes).is_err());
    }

    #[test]
    fn atomic_write_reopens_and_verifies() {
        let root = unique_root("atomic");
        let _ = fs::remove_dir_all(&root);
        let mut report = execution_report();
        let storage = persist_agent_private_learning_report_v0(&mut report, &root);
        assert_eq!(storage.failed_artifact_count, 0);
        assert!(storage.written_artifact_count >= 9);
        let momentum = result_for(&report, "momentum_trend_fast");
        let stored = fs::read(
            root.join("momentum_trend_fast")
                .join("sessions")
                .join(format!("{}.pb", momentum.session.session_id)),
        )
        .unwrap();
        assert_eq!(
            decode_session_protobuf_v0(&stored).unwrap(),
            momentum.session
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn repeated_local_write_is_explicitly_duplicate_rejected() {
        let root = unique_root("duplicate");
        let _ = fs::remove_dir_all(&root);
        let mut first = execution_report();
        let initial = persist_agent_private_learning_report_v0(&mut first, &root);
        assert_eq!(initial.failed_artifact_count, 0);
        let mut second = execution_report();
        let replay = persist_agent_private_learning_report_v0(&mut second, &root);
        assert_eq!(replay.failed_artifact_count, 0);
        assert_eq!(replay.written_artifact_count, 0);
        assert!(replay.duplicate_artifact_count >= 9);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn preliminary_artifact_paths_remain_byte_identical() {
        let root = unique_root("preliminary-preservation");
        let _ = fs::remove_dir_all(&root);
        let candidate_path = root
            .join("momentum_trend_fast")
            .join("candidates")
            .join("preliminary.pb");
        let journal_path = root.join("momentum_trend_fast").join("journal.pb");
        let registry_path = root.join("capability_registry.pb");
        fs::create_dir_all(candidate_path.parent().unwrap()).unwrap();
        let frozen = b"preliminary-retrospective-artifact";
        fs::write(&candidate_path, frozen).unwrap();
        fs::write(&journal_path, frozen).unwrap();
        fs::write(&registry_path, frozen).unwrap();
        let mut report = execution_report();
        let storage = persist_agent_private_learning_report_v0(&mut report, &root);
        assert_eq!(storage.failed_artifact_count, 0);
        assert_eq!(fs::read(candidate_path).unwrap(), frozen);
        assert_eq!(fs::read(journal_path).unwrap(), frozen);
        assert_eq!(fs::read(registry_path).unwrap(), frozen);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn prospective_artifact_sentinel_remains_byte_identical() {
        let root = unique_root("freeze");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let sentinel = root.join("prospective-sentinel.pb");
        let frozen = b"sealed-prospective-bytes".to_vec();
        fs::write(&sentinel, &frozen).unwrap();
        let mut report = execution_report();
        let storage =
            persist_agent_private_learning_report_v0(&mut report, &root.join("learning_data"));
        assert_eq!(storage.failed_artifact_count, 0);
        assert_eq!(fs::read(&sentinel).unwrap(), frozen);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn network_and_authority_counters_remain_zero() {
        let report = execution_report();
        let counters = report.safety_counters;
        assert_eq!(counters.network_requests, 0);
        assert_eq!(counters.credential_reads, 0);
        assert_eq!(counters.prospective_artifact_mutations, 0);
        assert_eq!(counters.prospective_label_reads, 0);
        assert_eq!(counters.chair_decisions, 0);
        assert_eq!(counters.votes, 0);
        assert_eq!(counters.rewards, 0);
        assert_eq!(counters.penalties, 0);
        assert_eq!(counters.voice_changes, 0);
        assert_eq!(counters.executions, 0);
        assert!(report.active_state_unchanged);
    }

    #[test]
    fn repeated_dry_run_is_deterministic() {
        let inputs = inputs();
        let first =
            run_agent_private_learning_sessions_v0(&inputs, AgentPrivateLearningRunModeV0::DryRun);
        let second =
            run_agent_private_learning_sessions_v0(&inputs, AgentPrivateLearningRunModeV0::DryRun);
        assert_eq!(first, second);
    }

    #[test]
    fn evidence_usage_ledgers_cover_training_normalization_validation_and_test() {
        let (_, report) = candidate_evaluation_fixture();
        for agent_id in ["momentum_trend_fast", "cycle_risk_skeptic"] {
            let ledger = evaluation_result_for(&report, agent_id)
                .evidence_usage_ledger
                .as_ref()
                .unwrap();
            for required in [
                CandidateEvidenceUseV0::ParameterTraining,
                CandidateEvidenceUseV0::NormalizerFit,
                CandidateEvidenceUseV0::ValidationInference,
                CandidateEvidenceUseV0::ValidationMetric,
                CandidateEvidenceUseV0::CheckpointSelection,
                CandidateEvidenceUseV0::HistoricalTestInference,
                CandidateEvidenceUseV0::HistoricalTestMetric,
                CandidateEvidenceUseV0::CandidateIdentity,
            ] {
                assert!(
                    ledger
                        .entries
                        .iter()
                        .any(|entry| entry.use_kind == required)
                );
            }
            assert!(
                ledger
                    .entries
                    .iter()
                    .all(|entry| { entry.entry_digest == evidence_usage_entry_digest_v0(entry) })
            );
        }
    }

    #[test]
    fn test_metrics_and_candidate_identity_usage_are_detected_independently() {
        let (_, report) = candidate_evaluation_fixture();
        for agent_id in ["momentum_trend_fast", "cycle_risk_skeptic"] {
            let result = evaluation_result_for(&report, agent_id);
            let audit = result.identity_audit.as_ref().unwrap();
            assert_eq!(
                audit.historical_test_status,
                CandidateHistoricalTestStatusV0::InfluencedCandidateIdentity
            );
            assert!(audit.test_evidence_in_identity);
            assert!(!audit.eligible_for_fresh_historical_test);
            assert!(
                result
                    .evidence_usage_ledger
                    .as_ref()
                    .unwrap()
                    .entries
                    .iter()
                    .any(
                        |entry| entry.use_kind == CandidateEvidenceUseV0::HistoricalTestMetric
                            && entry.labels_read
                    )
            );
        }
    }

    #[test]
    fn value_has_no_candidate_audit_or_registration() {
        let (_, report) = candidate_evaluation_fixture();
        let value = evaluation_result_for(&report, "value_quality_filter");
        assert_eq!(
            value.blocked_status,
            CandidateEvaluationRegistrationStatusV0::CandidateUnavailable
        );
        assert!(value.candidate_digest.is_none());
        assert!(value.evidence_usage_ledger.is_none());
        assert!(value.evaluation_registration.is_none());
    }

    #[test]
    fn preliminary_v0_lineage_is_superseded_and_policy_invalid() {
        let root = unique_root("legacy-lineage");
        let _ = fs::remove_dir_all(&root);
        let sessions = execution_report();
        for source in sessions
            .results
            .iter()
            .filter(|result| result.candidate.is_some())
        {
            let mut session = source.session.clone();
            let primary_artifact_digest = source
                .trainer_projection
                .as_ref()
                .and_then(|projection| projection.primary_series_digest.clone())
                .unwrap();
            session.session_version = SESSION_VERSION_V0.to_string();
            session.required_dataset_kinds.clear();
            session.optional_dataset_kinds.clear();
            session.allowed_markets.clear();
            session.symbols.clear();
            session.cadence.clear();
            session.lookback = DataLookback {
                bars: 0,
                start_timestamp_ms: None,
                end_timestamp_ms: None,
            };
            session.maximum_staleness_ms = 0;
            session.source_artifact_digests = vec![primary_artifact_digest.clone()];
            session.source_policy_digest.clear();
            session.training_ledger_digest.clear();
            session.trainer_projection_digest = None;
            session.session_digest = session_digest_v0(&session);
            let mut candidate = source.candidate.clone().unwrap();
            candidate.session_digest = session.session_digest.clone();
            candidate.candidate_digest = candidate_digest_v0(&candidate);
            let mut manifest = source.dataset_manifest.clone().unwrap();
            manifest.source_artifact_digests = vec![primary_artifact_digest];
            manifest.manifest_digest = dataset_manifest_digest_v0(&manifest);
            let agent_root = root.join(&session.agent_id);
            assert!(write_session_artifact_v0(&session, &agent_root.join("sessions")).is_ok());
            assert!(write_dataset_artifact_v0(&manifest, &agent_root.join("datasets")).is_ok());
            assert!(
                write_candidate_artifact_v0(&candidate, &agent_root.join("candidates")).is_ok()
            );
        }
        let report = run_agent_candidate_evaluation_v0(
            &root,
            AgentPrivateLearningRunModeV0::DryRun,
            true,
            None,
        );
        for agent_id in ["momentum_trend_fast", "cycle_risk_skeptic"] {
            let result = evaluation_result_for(&report, agent_id);
            let audit = result.identity_audit.as_ref().unwrap();
            assert!(audit.superseded_by_input_binding_hardening);
            assert_eq!(
                result.evaluation_registration.as_ref().unwrap().status,
                CandidateEvaluationRegistrationStatusV0::PolicyInvalid
            );
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ambiguous_candidate_selection_fails_closed() {
        let root = unique_root("ambiguous-candidate-lineage");
        let _ = fs::remove_dir_all(&root);
        let mut sessions = execution_report();
        assert_eq!(
            persist_agent_private_learning_report_v0(&mut sessions, &root).failed_artifact_count,
            0
        );
        let momentum = result_for(&sessions, "momentum_trend_fast");
        let source = root
            .join("momentum_trend_fast")
            .join("candidates")
            .join(format!(
                "{}.pb",
                momentum.candidate.as_ref().unwrap().candidate_digest
            ));
        let duplicate = source.with_file_name("second-candidate.pb");
        fs::copy(source, duplicate).unwrap();
        let report = run_agent_candidate_evaluation_v0(
            &root,
            AgentPrivateLearningRunModeV0::DryRun,
            true,
            None,
        );
        assert_eq!(
            evaluation_result_for(&report, "momentum_trend_fast").blocked_status,
            CandidateEvaluationRegistrationStatusV0::LineageAmbiguousBlocked
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn future_cutoff_is_strict_and_comparator_set_is_frozen() {
        let (root, _) = candidate_evaluation_fixture();
        let capability = agent_trainer_capability_registry_v0()
            .capabilities
            .into_iter()
            .find(|capability| capability.agent_id == "momentum_trend_fast")
            .unwrap();
        let mut lineage = load_candidate_lineage_v0(&root, &capability).unwrap();
        lineage.candidate.parent_model_version = Some("frozen-parent-v0".to_string());
        lineage.candidate.candidate_digest = candidate_digest_v0(&lineage.candidate);
        let ledger = candidate_evidence_usage_ledger_v0(&lineage);
        let audit = agent_candidate_identity_audit_v0(&lineage, &ledger);
        let first =
            agent_candidate_evaluation_registration_v0(&lineage, &ledger, &audit, Some(1_000));
        let second =
            agent_candidate_evaluation_registration_v0(&lineage, &ledger, &audit, Some(2_000));
        assert_eq!(
            first.status,
            CandidateEvaluationRegistrationStatusV0::Registered
        );
        assert_eq!(first.evaluation_cutoff_exclusive_ms, 1_000);
        assert!(!candidate_evaluation_accepts_timestamp_v0(&first, 1_000));
        assert!(candidate_evaluation_accepts_timestamp_v0(&first, 1_001));
        assert_eq!(first.comparator_digests, second.comparator_digests);
    }

    #[test]
    fn registration_hides_labels_probabilities_and_calculates_no_future_metric() {
        let (_, report) = candidate_evaluation_fixture();
        for registration in report
            .results
            .iter()
            .filter_map(|result| result.evaluation_registration.as_ref())
        {
            assert!(registration.labels_hidden_until_opening);
            assert!(registration.probabilities_hidden_until_opening);
            assert!(registration.one_time_opening_required);
            assert!(registration.active_promotion_forbidden);
            assert!(registration.reward_application_forbidden);
            assert_eq!(registration.maximum_concurrency, 1);
            assert_eq!(registration.maximum_retries, 0);
        }
        assert_eq!(report.safety_counters.prospective_row_reads, 0);
        assert_eq!(report.safety_counters.prospective_label_reads, 0);
        assert_eq!(report.safety_counters.prospective_mutations, 0);
    }

    #[test]
    fn candidate_evaluation_protobufs_round_trip_and_corruption_rejects() {
        let (_, report) = candidate_evaluation_fixture();
        let result = evaluation_result_for(&report, "momentum_trend_fast");
        let ledger = result.evidence_usage_ledger.as_ref().unwrap();
        let audit = result.identity_audit.as_ref().unwrap();
        let registration = result.evaluation_registration.as_ref().unwrap();
        let journal = result.registration_journal.as_ref().unwrap();
        assert_eq!(
            decode_evidence_usage_ledger_protobuf_v0(
                &encode_evidence_usage_ledger_protobuf_v0(ledger).unwrap()
            )
            .unwrap(),
            *ledger
        );
        assert_eq!(
            decode_candidate_identity_audit_protobuf_v0(
                &encode_candidate_identity_audit_protobuf_v0(audit).unwrap()
            )
            .unwrap(),
            *audit
        );
        assert_eq!(
            decode_candidate_evaluation_registration_protobuf_v0(
                &encode_candidate_evaluation_registration_protobuf_v0(registration).unwrap()
            )
            .unwrap(),
            *registration
        );
        let mut bytes = encode_candidate_evaluation_journal_protobuf_v0(journal).unwrap();
        assert_eq!(
            decode_candidate_evaluation_journal_protobuf_v0(&bytes).unwrap(),
            *journal
        );
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        assert!(decode_candidate_evaluation_journal_protobuf_v0(&bytes).is_err());
    }

    #[test]
    fn evaluation_persistence_reopens_and_duplicate_rejects() {
        let root = unique_root("evaluation-persistence");
        let _ = fs::remove_dir_all(&root);
        let mut sessions = execution_report();
        assert_eq!(
            persist_agent_private_learning_report_v0(&mut sessions, &root).failed_artifact_count,
            0
        );
        let first = run_agent_candidate_evaluation_v0(
            &root,
            AgentPrivateLearningRunModeV0::ExecuteLocal,
            true,
            None,
        );
        assert_eq!(first.storage_failure_count, 0);
        assert!(first.duplicate_artifact_count == 0);
        let second = run_agent_candidate_evaluation_v0(
            &root,
            AgentPrivateLearningRunModeV0::ExecuteLocal,
            true,
            None,
        );
        assert_eq!(second.storage_failure_count, 0);
        assert!(second.duplicate_artifact_count >= 10);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn evaluation_namespace_preserves_prospective_sentinel() {
        let root = unique_root("evaluation-prospective-freeze");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let sentinel = root.join("sealed-prospective-lane.pb");
        let frozen = b"sealed-prospective-lane".to_vec();
        fs::write(&sentinel, &frozen).unwrap();
        let mut sessions = execution_report();
        assert_eq!(
            persist_agent_private_learning_report_v0(&mut sessions, &root.join("learning_data"))
                .failed_artifact_count,
            0
        );
        let report = run_agent_candidate_evaluation_v0(
            &root.join("learning_data"),
            AgentPrivateLearningRunModeV0::ExecuteLocal,
            true,
            None,
        );
        assert_eq!(report.storage_failure_count, 0);
        assert_eq!(fs::read(&sentinel).unwrap(), frozen);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn candidate_evaluation_network_authority_and_active_state_remain_zero() {
        let (_, report) = candidate_evaluation_fixture();
        let counters = &report.safety_counters;
        assert_eq!(counters.active_committee_count, 3);
        assert_eq!(counters.network_requests, 0);
        assert_eq!(counters.credential_reads, 0);
        assert_eq!(counters.active_model_changes, 0);
        assert_eq!(counters.chair_decisions, 0);
        assert_eq!(counters.votes, 0);
        assert_eq!(counters.rewards, 0);
        assert_eq!(counters.penalties, 0);
        assert_eq!(counters.voice_changes, 0);
        assert_eq!(counters.promotions, 0);
        assert_eq!(counters.executions, 0);
        assert!(report.active_state_unchanged);
    }

    #[test]
    fn public_candidate_summary_excludes_rows_labels_metrics_parameters_and_paths() {
        let (_, report) = candidate_evaluation_fixture();
        let json =
            serde_json::to_string(&public_candidate_evaluation_summaries_v0(&report)).unwrap();
        for forbidden in [
            "rows",
            "labels",
            "private_metrics",
            "normalizer",
            "weights",
            "parameters",
            "prediction",
            "path",
        ] {
            assert!(!json.contains(forbidden));
        }
    }

    #[test]
    fn public_summary_excludes_private_training_material() {
        let report = execution_report();
        let json = serde_json::to_string(&public_session_summaries_v0(&report)).unwrap();
        assert!(!json.contains("private_metrics"));
        assert!(!json.contains("normalizer"));
        assert!(!json.contains("weights"));
        assert!(!json.contains("prediction"));
    }

    fn v1_inputs() -> Vec<AgentPrivateLearningInputV1> {
        let mut source = snapshots();
        for (snapshot_index, snapshot) in source.iter_mut().enumerate() {
            for (index, row) in snapshot.normalized_dataset.rows.iter_mut().enumerate() {
                let phase = index as f64 * 0.31 + snapshot_index as f64 * 0.07;
                let close =
                    110.0 + index as f64 * 0.015 + phase.sin() * 6.0 + (phase * 0.37).cos() * 3.0;
                let open = close + (phase * 1.7).sin() * 0.8;
                row.open = open;
                row.high = open.max(close) * (1.01 + (index % 5) as f64 * 0.001);
                row.low = open.min(close) * if index % 13 == 0 { 0.94 } else { 0.985 };
                row.close = close;
                row.volume = 1_000.0 + (phase * 0.9).sin().abs() * 900.0;
                row.trade_value = Some(row.close * row.volume);
            }
            snapshot.content_digest =
                historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
            snapshot.snapshot_id = snapshot_id_from_semantic_digest_v1(&snapshot.content_digest);
        }
        build_agent_private_learning_inputs_v0(&source, 360)
            .unwrap()
            .into_iter()
            .map(|input| AgentPrivateLearningInputV1 {
                input,
                persisted_view_verified: true,
            })
            .collect()
    }

    fn v1_family_report() -> AgentCandidateFamiliesReportV1 {
        static REPORT: OnceLock<AgentCandidateFamiliesReportV1> = OnceLock::new();
        REPORT
            .get_or_init(|| {
                run_agent_private_learning_candidates_v1(
                    &v1_inputs(),
                    AgentPrivateLearningRunModeV0::DryRun,
                )
            })
            .clone()
    }

    fn v1_family_result<'a>(
        report: &'a AgentCandidateFamiliesReportV1,
        agent_id: &str,
    ) -> &'a AgentCandidateFamilyResultV1 {
        report
            .results
            .iter()
            .find(|result| result.agent_id == agent_id)
            .unwrap()
    }

    #[test]
    fn v1_momentum_freezes_three_participants() {
        let report = v1_family_report();
        let result = v1_family_result(&report, "momentum_trend_fast");
        assert_eq!(
            result.status,
            AgentLearningSessionStatusV1::CandidateFamilyFrozen
        );
        assert_eq!(
            result.family.as_ref().unwrap().participants.len(),
            3,
            "participants={:?};receipts={:?}",
            result
                .family
                .as_ref()
                .unwrap()
                .participants
                .iter()
                .map(|participant| (&participant.model_kind, &participant.participant_digest))
                .collect::<Vec<_>>(),
            result.qualification_receipts,
        );
    }

    #[test]
    fn v1_cycle_risk_freezes_three_participants() {
        let report = v1_family_report();
        let result = v1_family_result(&report, "cycle_risk_skeptic");
        assert_eq!(
            result.status,
            AgentLearningSessionStatusV1::CandidateFamilyFrozen
        );
        assert_eq!(result.family.as_ref().unwrap().participants.len(), 3);
    }

    fn v1_reservation() -> ProtectedEvaluationReservationV1 {
        let mut reservation = ProtectedEvaluationReservationV1 {
            protected_registration_digests: vec![
                "capsule-momentum".to_string(),
                "capsule-risk".to_string(),
                "opening-registration".to_string(),
            ],
            reserved_timestamp_ms: vec![
                1_784_332_800_000,
                1_784_419_200_000,
                1_784_505_600_000,
                1_784_592_000_000,
            ],
            cadence_ms: DAILY_CADENCE_MS_V1,
            provider_finality_boundary_ms: 1_784_678_400_000,
            reservation_digest: String::new(),
        };
        reservation.reservation_digest = protected_reservation_digest_v1(&reservation);
        reservation
    }

    fn fully_qualified_v1_families() -> AgentCandidateFamiliesReportV1 {
        let mut report = v1_family_report();
        for result in &mut report.results {
            let Some(family) = &mut result.family else {
                continue;
            };
            for receipt in &mut result.qualification_receipts {
                receipt.qualification_status = ValidationQualificationStatusV1::Qualified;
                receipt.receipt_digest = qualification_receipt_digest_v1(receipt);
            }
            let participant_digests = family
                .participants
                .iter()
                .map(|participant| participant.participant_digest.as_str())
                .collect::<BTreeSet<_>>();
            family.validation_qualification_receipts = result
                .qualification_receipts
                .iter()
                .filter(|receipt| participant_digests.contains(receipt.participant_digest.as_str()))
                .map(|receipt| receipt.receipt_digest.clone())
                .collect();
            family.validation_qualification_receipts.sort();
            result.status = AgentLearningSessionStatusV1::CandidateFamilyFrozen;
        }
        report.report_digest = candidate_families_report_digest_v1(&report);
        report
    }

    fn v1_evaluation_report() -> AgentCandidateEvaluationsReportV1 {
        static REPORT: OnceLock<AgentCandidateEvaluationsReportV1> = OnceLock::new();
        REPORT
            .get_or_init(|| {
                run_agent_candidate_evaluations_v1(
                    &fully_qualified_v1_families(),
                    &v1_inputs(),
                    &v1_reservation(),
                    AgentPrivateLearningRunModeV0::DryRun,
                )
            })
            .clone()
    }

    #[test]
    fn v1_v0_artifacts_remain_byte_identical() {
        let root = unique_root("v1-v0-byte-freeze");
        let _ = fs::remove_dir_all(&root);
        let mut v0 = execution_report();
        assert_eq!(
            persist_agent_private_learning_report_v0(&mut v0, &root).failed_artifact_count,
            0
        );
        let mut paths = Vec::new();
        collect_protobuf_paths_v0(&root, &mut paths).unwrap();
        let before = paths
            .into_iter()
            .map(|path| {
                let bytes = fs::read(&path).unwrap();
                (path, bytes)
            })
            .collect::<Vec<_>>();
        let mut v1 = v1_family_report();
        assert_eq!(
            persist_agent_candidate_families_report_v1(&mut v1, &root).failed_artifact_count,
            0
        );
        for (path, bytes) in before {
            assert_eq!(fs::read(path).unwrap(), bytes);
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn v1_v0_candidate_is_never_a_parent() {
        for result in v1_family_report().results {
            if let Some(session) = result.session {
                assert!(session.fresh_initialization);
                assert!(!session.session_id.contains(CANDIDATE_VERSION_V0));
                assert!(!session.source_artifact_digests.iter().any(|digest| {
                    execution_report().results.iter().any(|v0| {
                        v0.candidate
                            .as_ref()
                            .is_some_and(|candidate| candidate.candidate_digest == *digest)
                    })
                }));
            }
        }
    }

    #[test]
    fn v1_complete_persisted_view_is_required() {
        let mut inputs = v1_inputs();
        let momentum = inputs
            .iter_mut()
            .find(|input| input.input.intent.agent_id == "momentum_trend_fast")
            .unwrap();
        momentum.persisted_view_verified = false;
        let report = run_agent_private_learning_candidates_v1(
            &inputs,
            AgentPrivateLearningRunModeV0::DryRun,
        );
        let result = v1_family_result(&report, "momentum_trend_fast");
        assert!(result.family.is_none());
        assert_eq!(
            result.status,
            AgentLearningSessionStatusV1::InsufficientEvidence
        );
    }

    #[test]
    fn v1_missing_required_evidence_blocks_only_affected_agent() {
        let mut inputs = v1_inputs();
        inputs
            .iter_mut()
            .find(|input| input.input.intent.agent_id == "cycle_risk_skeptic")
            .unwrap()
            .persisted_view_verified = false;
        let report = run_agent_private_learning_candidates_v1(
            &inputs,
            AgentPrivateLearningRunModeV0::DryRun,
        );
        assert!(
            v1_family_result(&report, "cycle_risk_skeptic")
                .family
                .is_none()
        );
        assert!(
            v1_family_result(&report, "momentum_trend_fast")
                .family
                .is_some()
        );
    }

    #[test]
    fn v1_projection_binds_every_consumed_artifact() {
        for result in v1_family_report().results {
            let (Some(session), Some(projection)) = (result.session, result.projection) else {
                continue;
            };
            assert_eq!(
                session.consumed_artifact_digests,
                projection.consumed_artifact_digests
            );
            assert!(
                projection
                    .consumed_artifact_digests
                    .iter()
                    .all(|digest| { session.source_artifact_digests.contains(digest) })
            );
        }
    }

    #[test]
    fn v1_unconsumed_authorized_artifacts_are_recorded() {
        for result in v1_family_report().results {
            let (Some(projection), Some(ledger)) = (result.projection, result.usage_ledger) else {
                continue;
            };
            for digest in projection.referenced_unconsumed_artifact_digests {
                assert!(ledger.entries.iter().any(|entry| {
                    entry.artifact_digest == digest
                        && entry.use_kind == CandidateEvidenceUseV1::ReferencedButUnconsumed
                        && entry.range.is_none()
                }));
            }
        }
    }

    #[test]
    fn v1_largest_row_shortcut_is_absent() {
        let mut source = snapshots();
        let expected = source
            .iter()
            .find(|snapshot| snapshot.dataset_kind == DatasetKind::DailyOhlcv)
            .unwrap()
            .content_digest
            .clone();
        let mut incompatible = source
            .iter()
            .find(|snapshot| snapshot.dataset_kind == DatasetKind::DailyOhlcv)
            .unwrap()
            .clone();
        incompatible.requested_lookback.bars = 50_000;
        incompatible.row_count = 50_000;
        source.push(incompatible);
        let built = build_agent_private_learning_inputs_v1(
            &source,
            360,
            &unique_root("v1-largest-row"),
            AgentPrivateLearningRunModeV0::DryRun,
        );
        let momentum = built
            .iter()
            .find(|input| input.input.intent.agent_id == "momentum_trend_fast")
            .unwrap();
        assert!(
            momentum
                .input
                .view
                .source_artifact_digests
                .contains(&expected)
        );
    }

    #[test]
    fn v1_fresh_initialization_is_deterministic() {
        let first = v1_family_report();
        let second = run_agent_private_learning_candidates_v1(
            &v1_inputs(),
            AgentPrivateLearningRunModeV0::DryRun,
        );
        assert_eq!(
            first
                .results
                .iter()
                .filter_map(|result| result.family.as_ref().map(|family| &family.family_digest))
                .collect::<Vec<_>>(),
            second
                .results
                .iter()
                .filter_map(|result| result.family.as_ref().map(|family| &family.family_digest))
                .collect::<Vec<_>>()
        );
        assert!(
            first
                .results
                .iter()
                .filter_map(|result| result.session.as_ref())
                .all(|session| session.fresh_initialization)
        );
    }

    fn ledger_range_for(
        ledger: &AgentCandidateUsageLedgerV1,
        use_kind: CandidateEvidenceUseV1,
    ) -> IndexRangeV0 {
        ledger
            .entries
            .iter()
            .find(|entry| entry.use_kind == use_kind && entry.range.is_some())
            .and_then(|entry| entry.range.clone())
            .unwrap()
    }

    #[test]
    fn v1_training_labels_stay_inside_training() {
        for result in v1_family_report().results {
            let Some(ledger) = result.usage_ledger else {
                continue;
            };
            let training = ledger_range_for(&ledger, CandidateEvidenceUseV1::ParameterTraining);
            assert!(ledger.entries.iter().any(|entry| {
                entry.use_kind == CandidateEvidenceUseV1::LabelDerivation
                    && entry.labels_read
                    && entry.range.as_ref() == Some(&training)
            }));
        }
    }

    #[test]
    fn v1_validation_labels_stay_inside_validation() {
        for result in v1_family_report().results {
            let Some(ledger) = result.usage_ledger else {
                continue;
            };
            let validation = ledger_range_for(&ledger, CandidateEvidenceUseV1::ValidationInference);
            assert!(ledger.entries.iter().any(|entry| {
                entry.use_kind == CandidateEvidenceUseV1::LabelDerivation
                    && entry.labels_read
                    && entry.range.as_ref() == Some(&validation)
            }));
        }
    }

    #[test]
    fn v1_normalizer_fits_training_only() {
        for result in v1_family_report().results {
            let Some(ledger) = result.usage_ledger else {
                continue;
            };
            assert_eq!(
                ledger_range_for(&ledger, CandidateEvidenceUseV1::NormalizerFit),
                ledger_range_for(&ledger, CandidateEvidenceUseV1::ParameterTraining)
            );
        }
    }

    #[test]
    fn v1_validation_parameter_updates_remain_zero() {
        for result in v1_family_report().results {
            assert!(
                result
                    .qualification_receipts
                    .iter()
                    .all(|receipt| { receipt.parameter_updates_during_validation == 0 })
            );
            if let Some(ledger) = result.usage_ledger {
                assert!(
                    ledger
                        .entries
                        .iter()
                        .filter(|entry| {
                            matches!(
                                entry.use_kind,
                                CandidateEvidenceUseV1::ValidationInference
                                    | CandidateEvidenceUseV1::ValidationMetric
                            )
                        })
                        .all(|entry| !entry.parameters_updated)
                );
            }
        }
    }

    #[test]
    fn v1_historical_test_row_reads_are_zero() {
        assert!(
            v1_family_report()
                .results
                .iter()
                .filter_map(|result| result.usage_ledger.as_ref())
                .all(|ledger| ledger.historical_test_row_reads == 0)
        );
    }

    #[test]
    fn v1_historical_test_label_reads_are_zero() {
        assert!(
            v1_family_report()
                .results
                .iter()
                .filter_map(|result| result.usage_ledger.as_ref())
                .all(|ledger| ledger.historical_test_label_reads == 0)
        );
    }

    #[test]
    fn v1_historical_test_metrics_are_zero() {
        assert!(
            v1_family_report()
                .results
                .iter()
                .filter_map(|result| result.usage_ledger.as_ref())
                .all(|ledger| {
                    ledger.historical_test_inference_count == 0
                        && ledger.historical_test_metric_count == 0
                        && ledger.historical_test_checkpoint_selection_count == 0
                        && !ledger.historical_test_identity_influence
                })
        );
    }

    #[test]
    fn v1_participant_identity_excludes_metric_results() {
        let report = v1_family_report();
        let result = v1_family_result(&report, "momentum_trend_fast");
        let participant = &result.family.as_ref().unwrap().participants[0];
        let before = participant.participant_digest.clone();
        let mut receipt = result
            .qualification_receipts
            .iter()
            .find(|receipt| receipt.participant_digest == before)
            .unwrap()
            .clone();
        receipt.private_metric_digest = "different-private-validation-metric".to_string();
        receipt.receipt_digest = qualification_receipt_digest_v1(&receipt);
        assert_eq!(participant_digest_v1(participant), before);
        assert_ne!(
            receipt.receipt_digest,
            result
                .qualification_receipts
                .iter()
                .find(|candidate| candidate.participant_digest == before)
                .unwrap()
                .receipt_digest
        );
    }

    #[test]
    fn v1_value_quality_remains_unavailable() {
        let report = v1_family_report();
        let result = v1_family_result(&report, "value_quality_filter");
        assert_eq!(
            result.status,
            AgentLearningSessionStatusV1::TrainerUnavailable
        );
        assert!(result.family.is_none());
    }

    #[test]
    fn v1_no_winner_is_selected() {
        assert!(
            v1_family_report()
                .results
                .iter()
                .filter_map(|result| result.family.as_ref())
                .all(|family| {
                    !family.winner_selected
                        && !family.eligible_for_active_committee
                        && !family.eligible_for_promotion
                        && !family.eligible_for_reward
                })
        );
    }

    #[test]
    fn v1_qualification_receipt_is_separate_from_identity() {
        for result in v1_family_report().results {
            let Some(family) = result.family else {
                continue;
            };
            assert_eq!(
                family.participants.len(),
                result.qualification_receipts.len()
            );
            assert!(result.qualification_receipts.iter().all(|receipt| {
                family
                    .participants
                    .iter()
                    .any(|participant| participant.participant_digest == receipt.participant_digest)
                    && receipt.receipt_digest != receipt.participant_digest
            }));
        }
    }

    #[test]
    fn v1_four_prospective_timestamps_enter_exclusion() {
        let reservation =
            load_protected_evaluation_reservation_v1(Path::new("config/local")).unwrap();
        assert_eq!(reservation.reserved_timestamp_ms.len(), 4);
        let report = v1_evaluation_report();
        assert!(
            report
                .results
                .iter()
                .filter_map(|result| result.exclusion.as_ref())
                .all(|exclusion| exclusion.excluded_timestamp_ms.len() == 4)
        );
    }

    #[test]
    fn v1_excluded_timestamp_admission_rejects() {
        let report = v1_evaluation_report();
        for result in report.results {
            let (Some(registration), Some(exclusion)) = (result.registration, result.exclusion)
            else {
                continue;
            };
            for timestamp in &exclusion.excluded_timestamp_ms {
                assert!(!evaluation_evidence_allowed_v1(
                    &registration,
                    &exclusion,
                    *timestamp
                ));
            }
        }
    }

    #[test]
    fn v1_minimum_timestamp_respects_reserved_boundary() {
        let reservation = v1_reservation();
        let boundary = reservation.reserved_timestamp_ms.last().unwrap() + reservation.cadence_ms;
        for registration in v1_evaluation_report()
            .results
            .iter()
            .filter_map(|result| result.registration.as_ref())
        {
            assert!(registration.minimum_accepted_timestamp_ms >= boundary);
        }
    }

    #[test]
    fn v1_registration_freezes_participant_set() {
        let families = fully_qualified_v1_families();
        let evaluations = v1_evaluation_report();
        for registration in evaluations
            .results
            .iter()
            .filter_map(|result| result.registration.as_ref())
        {
            let family = v1_family_result(&families, &registration.agent_id)
                .family
                .as_ref()
                .unwrap();
            let mut expected = family
                .participants
                .iter()
                .map(|participant| participant.participant_digest.clone())
                .collect::<Vec<_>>();
            expected.sort();
            assert_eq!(registration.participant_digests, expected);
        }
    }

    #[test]
    fn v1_registration_reads_no_future_evidence() {
        let report = v1_evaluation_report();
        assert_eq!(report.safety_counters.prospective_row_reads, 0);
        assert_eq!(report.safety_counters.prospective_label_reads, 0);
        assert_eq!(report.safety_counters.prospective_mutations, 0);
    }

    #[test]
    fn v1_repeated_persistence_is_duplicate_rejected() {
        let root = unique_root("v1-repeat-persistence");
        let _ = fs::remove_dir_all(&root);
        let mut first = v1_family_report();
        let first_storage = persist_agent_candidate_families_report_v1(&mut first, &root);
        assert_eq!(first_storage.failed_artifact_count, 0);
        let mut second = v1_family_report();
        let second_storage = persist_agent_candidate_families_report_v1(&mut second, &root);
        assert_eq!(second_storage.failed_artifact_count, 0);
        assert_eq!(
            second_storage.duplicate_artifact_count,
            first_storage.written_artifact_count
        );
        let mut evaluation_first = v1_evaluation_report();
        let evaluation_storage =
            persist_agent_candidate_evaluations_report_v1(&mut evaluation_first, &root);
        assert_eq!(evaluation_storage.failed_artifact_count, 0);
        let mut evaluation_second = v1_evaluation_report();
        let evaluation_replay =
            persist_agent_candidate_evaluations_report_v1(&mut evaluation_second, &root);
        assert_eq!(evaluation_replay.failed_artifact_count, 0);
        assert_eq!(
            evaluation_replay.duplicate_artifact_count,
            evaluation_storage.written_artifact_count
        );
        fs::remove_dir_all(root).unwrap();
    }

    fn corrupt_last_byte(mut bytes: Vec<u8>) -> Vec<u8> {
        let last = bytes.len() - 1;
        bytes[last] ^= 0xff;
        bytes
    }

    #[test]
    fn v1_all_manual_protobuf_artifacts_reject_corruption() {
        let families = fully_qualified_v1_families();
        let family_result = v1_family_result(&families, "momentum_trend_fast");
        let session = family_result.session.as_ref().unwrap();
        let projection = family_result.projection.as_ref().unwrap();
        let family = family_result.family.as_ref().unwrap();
        let participant = &family.participants[0];
        let receipt = &family_result.qualification_receipts[0];
        let ledger = family_result.usage_ledger.as_ref().unwrap();
        let evaluations = v1_evaluation_report();
        let evaluation = evaluations
            .results
            .iter()
            .find(|result| result.agent_id == "momentum_trend_fast")
            .unwrap();
        let exclusion = evaluation.exclusion.as_ref().unwrap();
        let registration = evaluation.registration.as_ref().unwrap();
        let journal = evaluation.journal.as_ref().unwrap();
        assert!(
            decode_session_protobuf_v1(&corrupt_last_byte(
                encode_session_protobuf_v1(session).unwrap()
            ))
            .is_err()
        );
        assert!(
            decode_trainer_projection_protobuf_v1(&corrupt_last_byte(
                encode_trainer_projection_protobuf_v1(projection).unwrap()
            ))
            .is_err()
        );
        assert!(
            decode_candidate_family_protobuf_v1(&corrupt_last_byte(
                encode_candidate_family_protobuf_v1(family).unwrap()
            ))
            .is_err()
        );
        assert!(
            decode_participant_protobuf_v1(&corrupt_last_byte(
                encode_participant_protobuf_v1(participant).unwrap()
            ))
            .is_err()
        );
        assert!(
            decode_qualification_receipt_protobuf_v1(&corrupt_last_byte(
                encode_qualification_receipt_protobuf_v1(receipt).unwrap()
            ))
            .is_err()
        );
        assert!(
            decode_usage_ledger_protobuf_v1(&corrupt_last_byte(
                encode_usage_ledger_protobuf_v1(ledger).unwrap()
            ))
            .is_err()
        );
        assert!(
            decode_evidence_exclusion_protobuf_v1(&corrupt_last_byte(
                encode_evidence_exclusion_protobuf_v1(exclusion).unwrap()
            ))
            .is_err()
        );
        assert!(
            decode_evaluation_registration_protobuf_v1(&corrupt_last_byte(
                encode_evaluation_registration_protobuf_v1(registration).unwrap()
            ))
            .is_err()
        );
        assert!(
            decode_evaluation_journal_protobuf_v1(&corrupt_last_byte(
                encode_evaluation_journal_protobuf_v1(journal).unwrap()
            ))
            .is_err()
        );
    }

    #[test]
    fn v1_active_and_prospective_artifacts_remain_unchanged() {
        let root = unique_root("v1-protected-sentinel");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let active = root.join("active-model.pb");
        let prospective = root.join("prospective-lane.pb");
        fs::write(&active, b"active-frozen").unwrap();
        fs::write(&prospective, b"prospective-frozen").unwrap();
        let mut families = v1_family_report();
        let mut evaluations = v1_evaluation_report();
        assert_eq!(
            persist_agent_candidate_families_report_v1(&mut families, &root).failed_artifact_count,
            0
        );
        assert_eq!(
            persist_agent_candidate_evaluations_report_v1(&mut evaluations, &root)
                .failed_artifact_count,
            0
        );
        assert_eq!(fs::read(active).unwrap(), b"active-frozen");
        assert_eq!(fs::read(prospective).unwrap(), b"prospective-frozen");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn v1_network_and_authority_counters_remain_zero() {
        for counters in [
            v1_family_report().safety_counters,
            v1_evaluation_report().safety_counters,
        ] {
            assert_eq!(counters.active_committee_count, 3);
            assert_eq!(counters.network_requests, 0);
            assert_eq!(counters.credential_reads, 0);
            assert_eq!(counters.prospective_row_reads, 0);
            assert_eq!(counters.prospective_label_reads, 0);
            assert_eq!(counters.prospective_mutations, 0);
            assert_eq!(counters.historical_test_reads_v1, 0);
            assert_eq!(counters.active_model_changes, 0);
            assert_eq!(counters.chair_decisions, 0);
            assert_eq!(counters.votes, 0);
            assert_eq!(counters.rewards, 0);
            assert_eq!(counters.penalties, 0);
            assert_eq!(counters.voice_changes, 0);
            assert_eq!(counters.promotions, 0);
            assert_eq!(counters.executions, 0);
        }
    }

    fn migration_sources_fixture_v1() -> PersistedIntentMigrationSourcesV1 {
        let mut snapshot = snapshot_for(DatasetKind::DailyOhlcv, 360, 86_400_000, 77);
        snapshot.requested_lookback = DataLookback {
            bars: 360,
            start_timestamp_ms: Some(1),
            end_timestamp_ms: Some(360),
        };
        snapshot.actual_start_timestamp_ms = Some(1);
        snapshot.actual_end_timestamp_ms = Some(360);
        snapshot
            .compatibility
            .as_mut()
            .unwrap()
            .requested_cutoff_timestamp_ms = Some(360);
        snapshot
            .compatibility
            .as_mut()
            .unwrap()
            .maximum_staleness_ms = 0;
        let policy = default_agent_data_policies()
            .into_iter()
            .find(|policy| policy.agent_kind == AgentKind::MomentumTrendFast)
            .unwrap();
        let data_intent = AgentDataIntent {
            agent_id: MOMENTUM_AGENT_ID_V1.to_string(),
            agent_kind: AgentKind::MomentumTrendFast,
            market_scope: AcquisitionMarketScope::UsStocks,
            symbols: vec!["SPY".to_string()],
            required_datasets: policy.required_dataset_kinds.clone(),
            optional_datasets: policy.optional_dataset_kinds.clone(),
            lookback: snapshot.requested_lookback.clone(),
            target_cadence: "1d".to_string(),
            max_staleness_ms: policy.max_staleness_ms,
            priority: DataPriority::Required,
            reason_codes: policy.reason_codes.clone(),
        };
        let canonical = create_agent_learning_intent_v0(
            &LearningDataCallerV0::Agent(MOMENTUM_AGENT_ID_V1.to_string()),
            &data_intent,
            &policy,
            360,
        )
        .unwrap();
        let legacy_intent_digest = stable_hash_string("migration-legacy-intent-v1");
        let mut legacy_projection = canonical.clone();
        legacy_projection.intent_version =
            PERSISTED_LEARNING_INTENT_PROJECTION_VERSION_V1.to_string();
        legacy_projection.intent_digest = legacy_intent_digest.clone();
        legacy_projection.source_policy_digest.clear();
        let legacy_private_state = derive_agent_private_learning_state_v0(&legacy_projection);
        let mut legacy_session = AgentPrivateLearningSessionV0 {
            session_version: SESSION_VERSION_V0.to_string(),
            session_id: "migration-legacy-session-v1".to_string(),
            agent_id: MOMENTUM_AGENT_ID_V1.to_string(),
            agent_kind: AgentKind::MomentumTrendFast,
            intent_digest: legacy_intent_digest.clone(),
            data_view_digest: stable_hash_string("migration-legacy-view-v1"),
            trainer_capability_digest: stable_hash_string("migration-capability-v1"),
            information_cutoff_ms: 360,
            required_dataset_kinds: vec![],
            optional_dataset_kinds: vec![],
            allowed_markets: vec![],
            symbols: vec![],
            cadence: String::new(),
            lookback: DataLookback {
                bars: 0,
                start_timestamp_ms: None,
                end_timestamp_ms: None,
            },
            maximum_staleness_ms: 0,
            source_artifact_digests: vec![snapshot.content_digest.clone()],
            source_policy_digest: String::new(),
            feature_policy_digest: canonical.feature_policy_digest.clone(),
            label_policy_digest: canonical.label_policy_digest.clone(),
            curriculum_policy_digest: canonical.curriculum_policy_digest.clone(),
            private_namespace_digest: legacy_private_state.private_namespace_digest,
            training_ledger_digest: String::new(),
            trainer_projection_digest: None,
            parent_model_version: None,
            session_status: AgentLearningSessionStatusV0::CandidateProduced,
            session_digest: String::new(),
        };
        legacy_session.session_digest = session_digest_v0(&legacy_session);
        let acquisition_gap = crate::data::AgentCanonicalViewGapV1 {
            agent_id: MOMENTUM_AGENT_ID_V1.to_string(),
            intent_digest: legacy_intent_digest.clone(),
            market_scopes: legacy_projection.market_scopes.clone(),
            symbols: legacy_projection.symbols.clone(),
            cadence: legacy_projection.cadence.clone(),
            lookback: legacy_projection.lookback.clone(),
            information_cutoff_ms: legacy_projection.information_cutoff_ms,
            maximum_staleness_ms: legacy_projection.maximum_staleness_ms,
            required_dataset_kinds: legacy_projection.required_datasets.clone(),
            resolved_required_dataset_kinds: vec![],
            missing_required_dataset_kinds: legacy_projection.required_datasets.clone(),
            optional_dataset_kinds: legacy_projection.optional_datasets.clone(),
            resolved_optional_dataset_kinds: vec![],
            missing_optional_dataset_kinds: legacy_projection.optional_datasets.clone(),
            usable_artifact_digests: vec![],
            rejected_artifact_digests: vec![],
            authorized_provider_ids: vec!["approved-provider".to_string()],
            trainer_available: true,
            status: CanonicalViewGapStatusV1::SegmentedAcquisitionRequired,
            gap_digest: stable_hash_string("migration-acquisition-gap-v1"),
        };
        let mut canonical_gap = acquisition_gap.clone();
        canonical_gap.resolved_required_dataset_kinds = legacy_projection.required_datasets.clone();
        canonical_gap.missing_required_dataset_kinds.clear();
        canonical_gap.usable_artifact_digests = vec![snapshot.content_digest.clone()];
        canonical_gap.authorized_provider_ids.clear();
        canonical_gap.status = CanonicalViewGapStatusV1::MissingOptionalEvidenceOnly;
        canonical_gap.gap_digest = stable_hash_string("migration-canonical-gap-v1");
        let segments = vec![
            crate::data::LearningEvidenceSegmentRegistrationV1 {
                segment_index: 0,
                expected_timestamps: (161..=360).collect(),
                expected_row_count: 200,
                request_to_utc: "1970-01-01T00:00:00Z".to_string(),
                maximum_requests: 1,
                maximum_retries: 0,
                segment_digest: stable_hash_string("migration-segment-0-v1"),
            },
            crate::data::LearningEvidenceSegmentRegistrationV1 {
                segment_index: 1,
                expected_timestamps: (1..=160).collect(),
                expected_row_count: 160,
                request_to_utc: "1970-01-01T00:00:00Z".to_string(),
                maximum_requests: 1,
                maximum_retries: 0,
                segment_digest: stable_hash_string("migration-segment-1-v1"),
            },
        ];
        let composite_registration = CompositeLearningAcquisitionRegistrationV1 {
            registration_version: "composite-learning-acquisition-registration-v1".to_string(),
            target_agent_ids: vec![MOMENTUM_AGENT_ID_V1.to_string()],
            intent_digest: legacy_intent_digest,
            gap_report_digest: acquisition_gap.gap_digest.clone(),
            provider_contract_digest: stable_hash_string("migration-provider-contract-v1"),
            dataset_kind: DatasetKind::DailyOhlcv,
            market_scope: AcquisitionMarketScope::UsStocks,
            symbols: vec!["SPY".to_string()],
            cadence: "1d".to_string(),
            information_cutoff_ms: 360,
            required_row_count: 360,
            expected_timestamp_digest: stable_hash_string("migration-timestamps-v1"),
            segments,
            maximum_total_requests: 2,
            maximum_concurrency: 1,
            maximum_retries_per_segment: 0,
            protected_registration_digests: vec![stable_hash_string("migration-protected-v1")],
            excluded_timestamp_ms: vec![1_000],
            read_only_required: true,
            credential_free_required: true,
            prospective_storage_forbidden: true,
            registration_digest: stable_hash_string("migration-registration-v1"),
        };
        let epoch_receipt = CompositeLearningEpochReceiptV1 {
            receipt_version: "composite-learning-epoch-receipt-v1".to_string(),
            registration_digest: composite_registration.registration_digest.clone(),
            segment_receipt_digests: vec![
                stable_hash_string("migration-receipt-0-v1"),
                stable_hash_string("migration-receipt-1-v1"),
            ],
            attempted_segment_count: 2,
            successful_segment_count: 2,
            request_count: 2,
            retry_count: 0,
            merged_snapshot_digest: Some(snapshot.content_digest.clone()),
            merged_provenance_digest: Some(stable_hash_string("migration-provenance-v1")),
            status: CompositeLearningEpochStatusV1::EvidenceAcquired,
            receipt_digest: stable_hash_string("migration-epoch-v1"),
        };
        PersistedIntentMigrationSourcesV1 {
            legacy_session,
            legacy_projection,
            policy,
            acquisition_gap,
            canonical_gap,
            composite_registration,
            epoch_receipt,
            canonical_snapshot: snapshot,
        }
    }

    fn migration_failure_for_v1(
        mutate: impl FnOnce(&mut PersistedIntentMigrationSourcesV1),
    ) -> PersistedIntentMigrationFailureV1 {
        let mut sources = migration_sources_fixture_v1();
        mutate(&mut sources);
        derive_persisted_learning_intent_migration_v1(&sources).unwrap_err()
    }

    #[test]
    fn migration_identifies_legacy_intent_version_as_first_normal_validation_failure() {
        let derived =
            derive_persisted_learning_intent_migration_v1(&migration_sources_fixture_v1()).unwrap();
        assert_eq!(
            derived.blocker,
            PersistedIntentMigrationBlockerV1::LegacySessionNotSelfDescribing
        );
        assert_eq!(
            derived.first_failing_invariant.as_deref(),
            Some("intent_version")
        );
        assert!(
            validate_agent_learning_intent_v0(
                &derived.canonical_intent,
                &migration_sources_fixture_v1().policy
            )
            .is_ok()
        );
    }

    #[test]
    fn migration_provenance_uses_only_declared_authoritative_sources() {
        let derived =
            derive_persisted_learning_intent_migration_v1(&migration_sources_fixture_v1()).unwrap();
        assert_eq!(derived.field_provenance.len(), 16);
        assert!(derived.field_provenance.iter().all(|field| {
            !field.sources.is_empty()
                && !field.source_artifact_digests.is_empty()
                && field.provenance_digest == migrated_field_provenance_digest_v1(field)
        }));
        let source_policy = derived
            .field_provenance
            .iter()
            .find(|field| field.field_name == "source_policy_digest")
            .unwrap();
        assert_eq!(
            source_policy.sources,
            vec![MigratedIntentFieldSourceV1::VerifiedAgentPolicy]
        );
    }

    #[test]
    fn migration_rejects_undocumented_empty_symbol_and_cadence() {
        let symbol = migration_failure_for_v1(|sources| sources.legacy_projection.symbols.clear());
        assert_eq!(symbol.invariant, "symbols");
        let cadence = migration_failure_for_v1(|sources| sources.legacy_projection.cadence.clear());
        assert_eq!(cadence.invariant, "cadence");
    }

    #[test]
    fn migration_rejects_market_symbol_and_cadence_conflicts() {
        let market = migration_failure_for_v1(|sources| {
            sources.canonical_gap.market_scopes = vec![AcquisitionMarketScope::BtcCrypto]
        });
        assert_eq!(market.invariant, "market_scopes");
        let symbol = migration_failure_for_v1(|sources| {
            sources.composite_registration.symbols = vec!["QQQ".to_string()]
        });
        assert_eq!(symbol.invariant, "symbols");
        let cadence = migration_failure_for_v1(|sources| {
            sources.composite_registration.cadence = "1h".to_string()
        });
        assert_eq!(cadence.invariant, "cadence");
    }

    #[test]
    fn migration_rejects_cutoff_conflicts() {
        let failure = migration_failure_for_v1(|sources| {
            sources.composite_registration.information_cutoff_ms -= 1
        });
        assert_eq!(
            failure.blocker,
            PersistedIntentMigrationBlockerV1::CutoffMismatch
        );
        assert_eq!(failure.invariant, "information_cutoff_ms");
    }

    #[test]
    fn migration_rejects_policy_dataset_and_staleness_mismatch() {
        let required = migration_failure_for_v1(|sources| {
            sources.canonical_gap.required_dataset_kinds = vec![DatasetKind::VolatilityDaily]
        });
        assert_eq!(
            required.blocker,
            PersistedIntentMigrationBlockerV1::RequiredEvidenceMismatch
        );
        let optional = migration_failure_for_v1(|sources| {
            sources.policy.optional_dataset_kinds.pop();
        });
        assert_eq!(optional.invariant, "optional_datasets");
        let staleness = migration_failure_for_v1(|sources| {
            sources.policy.max_staleness_ms += 1;
        });
        assert_eq!(staleness.invariant, "maximum_staleness_ms");
    }

    #[test]
    fn migration_rejects_explicit_source_policy_digest_mismatch() {
        let source = migration_failure_for_v1(|sources| {
            sources.legacy_projection.source_policy_digest = stable_hash_string("wrong-source")
        });
        assert_eq!(
            source.blocker,
            PersistedIntentMigrationBlockerV1::PolicyDigestMismatch
        );
        assert_eq!(source.invariant, "source_policy_digest");
    }

    #[test]
    fn migration_never_shortens_lookback() {
        let failure = migration_failure_for_v1(|sources| {
            sources.composite_registration.required_row_count -= 1
        });
        assert_eq!(failure.invariant, "lookback");
    }

    #[test]
    fn migration_requires_complete_required_and_exact_optional_unavailability() {
        let required = migration_failure_for_v1(|sources| {
            sources
                .canonical_gap
                .resolved_required_dataset_kinds
                .clear()
        });
        assert_eq!(
            required.blocker,
            PersistedIntentMigrationBlockerV1::RequiredEvidenceMismatch
        );
        let optional = migration_failure_for_v1(|sources| {
            sources.canonical_gap.missing_optional_dataset_kinds.clear()
        });
        assert_eq!(
            optional.blocker,
            PersistedIntentMigrationBlockerV1::OptionalEvidenceOnlyMisclassified
        );
    }

    #[test]
    fn migration_builds_ready_view_bound_to_exact_snapshot() {
        let sources = migration_sources_fixture_v1();
        let derived = derive_persisted_learning_intent_migration_v1(&sources).unwrap();
        assert_eq!(
            derived.canonical_view.source_artifact_digests,
            vec![sources.canonical_snapshot.content_digest]
        );
        assert_eq!(
            derived.canonical_view.decision_gate,
            EvidenceDecisionGate::Ready
        );
        assert!(derived.canonical_view.missing_required_datasets.is_empty());
        assert_eq!(
            derived.canonical_input.input.resolution_status,
            AgentViewResolutionStatusV0::OptionalEvidenceUnavailable
        );
    }

    #[test]
    fn migration_gap_status_prefers_more_resolved_same_cutoff_evidence() {
        let sources = migration_sources_fixture_v1();
        let statuses = latest_agent_canonical_view_gap_statuses_v1(vec![
            sources.acquisition_gap.clone(),
            sources.canonical_gap.clone(),
        ])
        .unwrap();
        assert_eq!(
            statuses.get(MOMENTUM_AGENT_ID_V1),
            Some(&CanonicalViewGapStatusV1::MissingOptionalEvidenceOnly)
        );

        let mut conflicting = sources.canonical_gap.clone();
        conflicting.status = CanonicalViewGapStatusV1::ProviderContractUnverified;
        assert!(
            latest_agent_canonical_view_gap_statuses_v1(vec![conflicting, sources.canonical_gap])
                .is_err()
        );
    }

    #[test]
    fn migration_rejects_missing_required_view_and_snapshot_mismatch() {
        let missing = migration_failure_for_v1(|sources| {
            sources.canonical_gap.missing_required_dataset_kinds =
                sources.canonical_gap.required_dataset_kinds.clone()
        });
        assert_eq!(
            missing.blocker,
            PersistedIntentMigrationBlockerV1::RequiredEvidenceMismatch
        );
        let snapshot = migration_failure_for_v1(|sources| {
            sources.epoch_receipt.merged_snapshot_digest = Some(stable_hash_string("wrong"))
        });
        assert_eq!(
            snapshot.blocker,
            PersistedIntentMigrationBlockerV1::CanonicalSnapshotBindingMismatch
        );
    }

    #[test]
    fn migration_rejects_protected_timestamps() {
        let failure = migration_failure_for_v1(|sources| {
            sources.composite_registration.excluded_timestamp_ms = vec![1]
        });
        assert_eq!(failure.invariant, "protected_evidence_exclusions");
    }

    #[test]
    fn migration_protobufs_round_trip_and_reject_corruption() {
        let derived =
            derive_persisted_learning_intent_migration_v1(&migration_sources_fixture_v1()).unwrap();
        migration_round_trip_v1(&derived).unwrap();
        let mut intent =
            encode_canonical_learning_intent_migration_protobuf_v1(&derived.canonical_intent)
                .unwrap();
        *intent.last_mut().unwrap() ^= 1;
        assert!(decode_canonical_learning_intent_migration_protobuf_v1(&intent).is_err());
        let mut proof =
            encode_learning_intent_migration_proof_protobuf_v1(&derived.migration_proof).unwrap();
        *proof.last_mut().unwrap() ^= 1;
        assert!(decode_learning_intent_migration_proof_protobuf_v1(&proof).is_err());
    }

    #[test]
    fn migration_sidecars_are_additive_idempotent_and_preserve_legacy_bytes() {
        let root = PathBuf::from(format!(
            "state/learning_data/migration-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let legacy = root.join("legacy-session.pb");
        fs::write(&legacy, b"legacy-frozen").unwrap();
        let sources = migration_sources_fixture_v1();
        let derived = derive_persisted_learning_intent_migration_v1(&sources).unwrap();
        assert_eq!(
            persist_persisted_learning_intent_migration_v1(&derived, &root).unwrap(),
            (5, 0)
        );
        assert_eq!(fs::read(&legacy).unwrap(), b"legacy-frozen");
        assert_eq!(
            persist_persisted_learning_intent_migration_v1(&derived, &root).unwrap(),
            (0, 5)
        );
        assert_eq!(
            read_persisted_learning_intent_migration_v1(
                &root,
                std::slice::from_ref(&sources.canonical_snapshot)
            )
            .unwrap(),
            derived.canonical_input
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn migration_candidate_family_is_fresh_metric_free_and_historical_test_sealed() {
        let derived =
            derive_persisted_learning_intent_migration_v1(&migration_sources_fixture_v1()).unwrap();
        let report = run_agent_private_learning_candidates_v1(
            std::slice::from_ref(&derived.canonical_input),
            AgentPrivateLearningRunModeV0::DryRun,
        );
        let result = v1_family_result(&report, MOMENTUM_AGENT_ID_V1);
        let session = result.session.as_ref().unwrap();
        let family = result.family.as_ref().unwrap();
        assert!(session.fresh_initialization);
        assert_eq!(family.participants.len(), 3);
        assert!(!family.winner_selected);
        assert!(!family.historical_test_accessed);
        assert!(family.participants.iter().all(|participant| {
            !participant.participant_id.contains("metric")
                && !participant.participant_digest.contains("metric")
        }));
        let ledger = result.usage_ledger.as_ref().unwrap();
        assert_eq!(ledger.historical_test_row_reads, 0);
        assert_eq!(ledger.historical_test_label_reads, 0);
        assert_eq!(ledger.historical_test_inference_count, 0);
        assert_eq!(ledger.historical_test_metric_count, 0);
        assert_eq!(ledger.historical_test_checkpoint_selection_count, 0);
    }

    #[test]
    fn migration_registration_keeps_exclusions_and_blocks_unqualified_family() {
        let derived =
            derive_persisted_learning_intent_migration_v1(&migration_sources_fixture_v1()).unwrap();
        let families = run_agent_private_learning_candidates_v1(
            std::slice::from_ref(&derived.canonical_input),
            AgentPrivateLearningRunModeV0::DryRun,
        );
        let evaluations = run_agent_candidate_evaluations_v1(
            &families,
            std::slice::from_ref(&derived.canonical_input),
            &v1_reservation(),
            AgentPrivateLearningRunModeV0::DryRun,
        );
        let result = evaluations
            .results
            .iter()
            .find(|result| result.agent_id == MOMENTUM_AGENT_ID_V1)
            .unwrap();
        assert!(matches!(
            result.status,
            CandidateEvaluationRegistrationStatusV1::Registered
                | CandidateEvaluationRegistrationStatusV1::QualificationBlocked
                | CandidateEvaluationRegistrationStatusV1::InsufficientParticipants
        ));
        if let Some(registration) = result.registration.as_ref() {
            assert_eq!(registration.maximum_concurrency, 1);
            assert_eq!(registration.maximum_retries, 0);
            assert!(registration.labels_hidden_until_opening);
            assert!(registration.probabilities_hidden_until_opening);
            assert!(registration.winner_selection_forbidden_before_opening);
            assert!(registration.active_promotion_forbidden);
            assert!(registration.reward_application_forbidden);
        }
    }

    #[test]
    fn migration_safety_counters_have_only_active_committee_count() {
        let counters = zero_intent_migration_safety_counters_v1();
        assert_eq!(counters.active_committee_count, 3);
        assert_eq!(
            stable_hash_string(&format!("{counters:?}")),
            stable_hash_string(&format!("{:?}", zero_intent_migration_safety_counters_v1()))
        );
        assert_eq!(counters.network_requests, 0);
        assert_eq!(counters.transport_constructions, 0);
        assert_eq!(counters.credential_reads, 0);
        assert_eq!(counters.prospective_artifact_reads, 0);
        assert_eq!(counters.prospective_label_reads, 0);
        assert_eq!(counters.future_evaluation_reads, 0);
        assert_eq!(counters.active_model_changes, 0);
        assert_eq!(counters.chair_decisions, 0);
        assert_eq!(counters.votes, 0);
        assert_eq!(counters.rewards, 0);
        assert_eq!(counters.penalties, 0);
        assert_eq!(counters.voice_changes, 0);
        assert_eq!(counters.promotions, 0);
        assert_eq!(counters.executions, 0);
    }
}
