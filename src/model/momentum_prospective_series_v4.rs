//! Append-only prospective-series lifecycle for the frozen Momentum V4 roster.
//!
//! The series adopts the already opened first event, preregisters the next
//! cadence-derived legal event before input finality, reuses verified canonical
//! raw evidence, and permits at most one explicitly confirmed public input
//! request. It has no training, ranking, reward, Chair, or execution authority.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    core::stable_hash_string,
    data::{
        AcquisitionMarketScope, DataLookback, DataSnapshot, DatasetKind,
        LearningEvidenceTransportFailureV1, LearningEvidenceTransportResponseV1,
        ReadOnlyProviderRequest, UpbitHistoricalPilotConfigV0,
        fetch_upbit_learning_evidence_once_v1, historical_replay_dataset_digest_v0,
        parse_upbit_daily_ohlcv_v0, upbit_learning_evidence_provider_contract_v1,
    },
    league::{HistoricalOhlcvRow, canonical_current_agent_states},
};

use super::{
    MomentumLearningCampaignConfigV0, ProtectedEvaluationReservationV1,
    momentum_future_outcome_v4::{
        CAPSULE_VERSION, MomentumFutureOutcomeOpeningReportV4_4, MomentumFutureOutcomeReportV4_4,
        MomentumOutcomeAcquisitionReceiptV4_4, MomentumOutcomeAcquisitionRegistrationV4_4,
        MomentumOutcomeAcquisitionStatusV4_4, MomentumOutcomeOpeningAuthorizationV4_4,
        MomentumOutcomeOpeningBundleV4_4, MomentumOutcomeOpeningReceiptV4_4,
        MomentumOutcomeOpeningRunModeV4_4, MomentumOutcomeOpeningStatusV4_4,
        MomentumOutcomeReadinessV4_4, MomentumOutcomeRowIdentityProofV4_4,
        MomentumOutcomeRunModeV4_4, MomentumParticipantProspectiveEvaluationV4_4,
        MomentumProspectiveEvaluationStatusV4_4, MomentumProspectiveLabelStatusV4_4,
        MomentumRewardEligibilityStatusV4_4, MomentumSealedOutcomeCapsuleV4_4,
        OPENING_AUTHORIZATION_VERSION, OPENING_BUNDLE_VERSION, OPENING_RECEIPT_VERSION,
        RECEIPT_VERSION, REGISTRATION_VERSION, ROW_PROOF_VERSION, authorization_digest,
        build_participant_evaluation_v4_4, build_request as build_outcome_request,
        capsule_digest as sealed_outcome_capsule_digest, classify_label_v4_4,
        decode_capsule as decode_series_outcome_capsule,
        decode_evaluation as decode_series_evaluation,
        decode_opening_authorization as decode_series_opening_authorization,
        decode_opening_bundle as decode_series_opening_bundle,
        decode_opening_receipt as decode_series_opening_receipt,
        decode_receipt as decode_series_outcome_receipt,
        decode_registration as decode_series_outcome_registration,
        decode_row_proof as decode_series_outcome_proof,
        encode_capsule as encode_series_outcome_capsule,
        encode_evaluation as encode_series_evaluation,
        encode_opening_authorization as encode_series_opening_authorization,
        encode_opening_bundle as encode_series_opening_bundle,
        encode_opening_receipt as encode_series_opening_receipt,
        encode_receipt as encode_series_outcome_receipt,
        encode_registration as encode_series_outcome_registration,
        encode_row_proof as encode_series_outcome_proof, evaluation_policy_digest,
        frozen_label_policy_digest, opening_bundle_digest, opening_receipt_digest,
        parse_http_status_class, raw_outcome_digest,
        receipt_digest as series_outcome_receipt_digest, registration_digest,
        request_config as outcome_request_config,
        request_fingerprint as outcome_request_fingerprint, row_proof_digest,
        run_momentum_future_outcome_opening_v4_4, run_momentum_future_outcome_v4_4,
        sanitized_raw_response, valid_ohlcv as valid_outcome_ohlcv,
        validate_capsule_shape as validate_series_outcome_capsule, validate_evaluation_shape,
        validate_opening_authorization_shape, validate_opening_bundle_shape,
        validate_opening_receipt_shape, validate_outcome_transport,
        validate_receipt_shape as validate_series_outcome_receipt,
        validate_registration_shape as validate_series_outcome_registration,
        validate_row_proof_shape as validate_series_outcome_proof,
    },
    momentum_future_prediction_v4::{
        ArtifactBuilderV4_2, ArtifactReaderV4_2, MomentumSealedPredictionChainV4_3, as_u64,
        as_usize, persist_artifact, read_single, reopen_momentum_v4_3_sealed_chain,
        row_identity_digest,
    },
    momentum_raw_feature_supplemental::reopen_momentum_v4_1_future_source,
    momentum_raw_feature_v4::{
        MomentumRawFeatureRoleV4, predict_frozen_momentum_v4_event, reconstruct_frozen_momentum_v4,
    },
};

const AGENT_ID: &str = "momentum_trend_fast";
const SERIES_ROOT: &str = "v4_series";
const DAILY_CADENCE_MS: u64 = 86_400_000;
const SERIES_VERSION: &str = "momentum-prospective-series-v4";
const ADOPTION_VERSION: &str = "momentum-prospective-series-adoption-v4";
const GAP_AUDIT_VERSION: &str = "momentum-prospective-candidate-gap-audit-v4";
const DELTA_PLAN_VERSION: &str = "momentum-prospective-context-delta-plan-v4";
const EPOCH_REGISTRATION_VERSION: &str = "momentum-prospective-epoch-registration-v4";
const CONTEXT_USE_PROOF_VERSION: &str = "momentum-prospective-context-use-proof-v4";
const CONTEXT_ASSEMBLY_VERSION: &str = "momentum-prospective-context-assembly-v4";
const STATUS_VERSION: &str = "momentum-prospective-epoch-status-v4";
const INPUT_RECEIPT_VERSION: &str = "momentum-prospective-series-input-receipt-v4";
const INPUT_CAPSULE_VERSION: &str = "momentum-prospective-series-input-capsule-v4";
const PREDICTION_SEAL_VERSION: &str = "momentum-prospective-series-prediction-seal-v4";
const PREDICTION_CAPSULE_VERSION: &str = "momentum-prospective-series-prediction-capsule-v4";
const JOURNAL_VERSION: &str = "momentum-prospective-series-journal-v4";
const OUTCOME_PLAN_VERSION: &str = "momentum-prospective-series-outcome-plan-v4";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumProspectiveContinuationPolicyV4 {
    FixedDailyCadenceNextLegalEvent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumProspectiveCandidateDispositionV4 {
    Eligible,
    SkippedRegistrationAfterInputFinality,
    SkippedPriorOutcomeAlreadyOpened,
    SkippedIntegrityConflict,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumSeriesContextUseV4 {
    ExistingCanonicalHistoricalRaw,
    PriorProtectedRawContext,
    PriorProspectiveEventRawContext,
    PriorOpenedOutcomeRawContext,
    NewIncrementalRawContext,
    CurrentProspectiveEventInput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumProspectiveEpochReadinessV4 {
    RegisteredAwaitingInputFinality,
    ReadyForInputAcquisition,
    ReadyForLocalPredictionRecovery,
    PredictionAlreadySealed,
    PredictionSealWindowExpired,
    PriorInputAttemptTerminal,
    ProspectiveWindowExpired,
    MissingCanonicalContext,
    MissingSetNotContiguous,
    PriorPrivateEvidenceAccessDetected,
    FrozenIdentityMismatch,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumProspectiveSeriesRunModeV4 {
    Status,
    DryRun,
    RegisterNextEpoch,
    ExecuteInput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumProspectiveSeriesInputStatusV4 {
    ReadyNotAttempted,
    EvidenceAcquired,
    TerminalTransportFailure,
    TerminalValidationFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveSeriesV4 {
    pub series_version: String,
    pub agent_id: String,
    pub frozen_roster_digest: String,
    pub participant_digests: Vec<String>,
    pub parameter_digests: Vec<String>,
    pub normalizer_digests: Vec<String>,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub evaluation_policy_digest: String,
    pub minimum_sample_policy_digest: String,
    pub provider_id: String,
    pub market: String,
    pub symbol: String,
    pub cadence_ms: u64,
    pub context_row_count: usize,
    pub prediction_horizon: usize,
    pub first_event_ledger_entry_digest: String,
    pub first_event_opening_bundle_digest: String,
    pub first_event_eligibility_digest: String,
    pub continuation_policy: MomentumProspectiveContinuationPolicyV4,
    pub maximum_open_epochs: usize,
    pub manual_network_confirmation_required: bool,
    pub automatic_network_execution_forbidden: bool,
    pub retraining_forbidden: bool,
    pub participant_selection_forbidden: bool,
    pub result_conditioned_continuation_forbidden: bool,
    pub winner_selection_forbidden: bool,
    pub ranking_forbidden: bool,
    pub reward_application_forbidden: bool,
    pub penalty_application_forbidden: bool,
    pub chair_action_forbidden: bool,
    pub trading_forbidden: bool,
    pub protected_before_artifact_count: usize,
    pub protected_before_aggregate_digest: String,
    pub active_agent_state_digest: String,
    pub series_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveSeriesAdoptionV4 {
    pub adoption_version: String,
    pub series_digest: String,
    pub adopted_epoch_number: u64,
    pub adopted_event_timestamp_ms: u64,
    pub prediction_capsule_digest: String,
    pub outcome_capsule_digest: String,
    pub opening_bundle_digest: String,
    pub evaluation_ledger_entry_digest: String,
    pub reward_eligibility_digest: String,
    pub total_event_count: usize,
    pub scorable_event_count: usize,
    pub winner_selected: bool,
    pub ranking_created: bool,
    pub reward_applied: bool,
    pub penalty_applied: bool,
    pub chair_action_taken: bool,
    pub adoption_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveCandidateGapAuditV4 {
    pub audit_version: String,
    pub series_digest: String,
    pub prior_event_timestamp_ms: u64,
    pub adjacent_candidate_timestamp_ms: u64,
    pub registration_observed_at_ms: u64,
    pub candidate_input_finality_boundary_ms: u64,
    pub registration_after_input_finality: bool,
    pub prior_outcome_already_opened: bool,
    pub applicable_reasons: Vec<String>,
    pub canonical_disposition: MomentumProspectiveCandidateDispositionV4,
    pub counted_as_model_failure: bool,
    pub reward_or_penalty_consequence: bool,
    pub audit_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumSeriesCanonicalRowRefV4 {
    pub timestamp_ms: u64,
    pub raw_row_digest: String,
    pub source_capsule_digest: String,
    pub use_class: MomentumSeriesContextUseV4,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumCanonicalContextDeltaPlanV4 {
    pub plan_version: String,
    pub series_digest: String,
    pub epoch_number: u64,
    pub event_timestamp_ms: u64,
    pub exact_context_timestamp_ms: Vec<u64>,
    pub canonical_rows: Vec<MomentumSeriesCanonicalRowRefV4>,
    pub exact_missing_timestamp_ms: Vec<u64>,
    pub maximum_requests: usize,
    pub maximum_retries: usize,
    pub maximum_concurrency: usize,
    pub full_context_refetch_forbidden: bool,
    pub prior_private_evaluation_accessed: bool,
    pub missing_set_contiguous: bool,
    pub plan_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveEpochRegistrationV4 {
    pub registration_version: String,
    pub series_digest: String,
    pub epoch_number: u64,
    pub previous_epoch_ledger_entry_digest: String,
    pub previous_epoch_opening_bundle_digest: String,
    pub event_timestamp_ms: u64,
    pub registration_created_at_ms: u64,
    pub input_finality_boundary_ms: u64,
    pub outcome_timestamp_ms: u64,
    pub outcome_finality_boundary_ms: u64,
    pub exact_context_timestamp_ms: Vec<u64>,
    pub exact_missing_timestamp_ms: Vec<u64>,
    pub context_delta_plan_digest: String,
    pub provider_id: String,
    pub market: String,
    pub symbol: String,
    pub cadence: String,
    pub maximum_input_requests: usize,
    pub maximum_input_retries: usize,
    pub maximum_input_concurrency: usize,
    pub maximum_response_bytes: usize,
    pub prior_private_evaluation_access_forbidden: bool,
    pub parameter_update_forbidden: bool,
    pub normalizer_refit_forbidden: bool,
    pub outcome_access_forbidden: bool,
    pub winner_selection_forbidden: bool,
    pub ranking_forbidden: bool,
    pub reward_application_forbidden: bool,
    pub penalty_application_forbidden: bool,
    pub chair_action_forbidden: bool,
    pub trading_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumSeriesContextUseEntryV4 {
    pub timestamp_ms: u64,
    pub raw_row_digest: String,
    pub source_capsule_digest: String,
    pub use_class: MomentumSeriesContextUseV4,
    pub feature_construction_allowed: bool,
    pub training_forbidden: bool,
    pub normalizer_fitting_forbidden: bool,
    pub label_use_forbidden: bool,
    pub metric_use_forbidden: bool,
    pub reward_use_forbidden: bool,
    pub participant_selection_forbidden: bool,
    pub entry_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumSeriesContextUseProofV4 {
    pub proof_version: String,
    pub series_digest: String,
    pub epoch_registration_digest: String,
    pub entries: Vec<MomentumSeriesContextUseEntryV4>,
    pub prior_opening_bundle_used_as_raw_source: bool,
    pub prior_private_scores_accessed: bool,
    pub prior_label_used_as_feature: bool,
    pub reward_eligibility_used_as_feature: bool,
    pub proof_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumSeriesContextAssemblyProofV4 {
    pub proof_version: String,
    pub series_digest: String,
    pub epoch_registration_digest: String,
    pub input_capsule_digest: String,
    pub context_use_proof_digest: String,
    pub exact_context_timestamp_ms: Vec<u64>,
    pub exact_row_digests: Vec<String>,
    pub exact_row_count: usize,
    pub strict_chronology_verified: bool,
    pub all_row_digests_verified: bool,
    pub event_timestamp_is_last: bool,
    pub outcome_timestamp_absent: bool,
    pub proof_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveSeriesInputReceiptV4 {
    pub receipt_version: String,
    pub series_digest: String,
    pub epoch_registration_digest: String,
    pub request_attempted: bool,
    pub request_count: usize,
    pub retry_count: usize,
    pub transport_construction_count: usize,
    pub status: MomentumProspectiveSeriesInputStatusV4,
    pub http_status_class: Option<String>,
    pub returned_row_count: usize,
    pub verified_row_count: usize,
    pub raw_response_digest: Option<String>,
    pub input_capsule_digest: Option<String>,
    pub terminal: bool,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveSeriesInputCapsuleV4 {
    pub capsule_version: String,
    pub series_digest: String,
    pub epoch_registration_digest: String,
    pub context_delta_plan_digest: String,
    pub provider_id: String,
    pub request_attempt_count: usize,
    pub event_timestamp_ms: u64,
    pub exact_timestamp_ms: Vec<u64>,
    pub row_identity_digests: Vec<String>,
    pub normalized_dataset_digest: String,
    pub raw_response_digest: String,
    pub outcome_row_present: bool,
    pub labels_accessed: bool,
    pub metrics_computed: bool,
    pub prior_private_evaluation_accessed: bool,
    pub credential_free: bool,
    pub read_only: bool,
    pub sanitized: bool,
    pub capsule_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumSeriesParticipantPredictionSealV4 {
    pub seal_version: String,
    pub series_digest: String,
    pub epoch_number: u64,
    pub epoch_registration_digest: String,
    pub participant_digest: String,
    pub participant_role: String,
    pub event_timestamp_ms: u64,
    pub input_receipt_digest: String,
    pub input_capsule_digest: String,
    pub context_use_proof_digest: String,
    pub context_assembly_proof_digest: String,
    pub feature_identity_digest: String,
    pub prediction_probability_bits: u32,
    pub prediction_digest: String,
    pub participant_identity_verified: bool,
    pub parameter_updates: usize,
    pub normalizer_refits: usize,
    pub prior_score_reads: usize,
    pub outcome_access_count: usize,
    pub seal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveSeriesPredictionCapsuleV4 {
    pub capsule_version: String,
    pub series_digest: String,
    pub epoch_registration_digest: String,
    pub event_timestamp_ms: u64,
    pub input_receipt_digest: String,
    pub input_capsule_digest: String,
    pub context_assembly_proof_digest: String,
    pub participant_seal_digests: Vec<String>,
    pub participant_prediction_digests: Vec<String>,
    pub probabilities_hidden: bool,
    pub labels_hidden: bool,
    pub prior_scores_accessed: bool,
    pub outcome_accessed: bool,
    pub metrics_computed: bool,
    pub winner_selected: bool,
    pub ranking_created: bool,
    pub reward_applied: bool,
    pub penalty_applied: bool,
    pub chair_action_taken: bool,
    pub capsule_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveSeriesJournalEntryV4 {
    pub journal_version: String,
    pub series_digest: String,
    pub epoch_number: u64,
    pub event_one_adoption_digest: String,
    pub previous_epoch_ledger_entry_digest: String,
    pub context_delta_plan_digest: String,
    pub event_timestamp_ms: u64,
    pub registration_created_at_ms: u64,
    pub input_finality_boundary_ms: u64,
    pub input_receipt_digest: String,
    pub input_capsule_digest: String,
    pub context_assembly_proof_digest: String,
    pub prediction_capsule_digest: String,
    pub participant_seal_digests: Vec<String>,
    pub participant_prediction_digests: Vec<String>,
    pub deterministic_fixed_cadence_selection: bool,
    pub prior_event_scores_read: bool,
    pub prior_event_correctness_read: bool,
    pub registration_preceded_input_finality: bool,
    pub input_acquisition_preceded_prediction: bool,
    pub prediction_preceded_outcome_access: bool,
    pub outcome_stage_locked: bool,
    pub winner_selected: bool,
    pub ranking_created: bool,
    pub reward_applied: bool,
    pub penalty_applied: bool,
    pub chair_action_taken: bool,
    pub trading_action_taken: bool,
    pub entry_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveSeriesOutcomePlanV4 {
    pub plan_version: String,
    pub series_digest: String,
    pub epoch_registration_digest: String,
    pub prediction_capsule_digest: String,
    pub event_timestamp_ms: u64,
    pub prediction_horizon: usize,
    pub required_outcome_timestamp_ms: Vec<u64>,
    pub outcome_finality_boundary_ms: u64,
    pub maximum_outcome_requests: usize,
    pub maximum_outcome_retries: usize,
    pub outcome_acquisition_count: usize,
    pub outcome_opening_count: usize,
    pub labels_hidden_until_opening: bool,
    pub one_time_opening_required: bool,
    pub outcome_stage_locked_before_finality: bool,
    pub plan_digest: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveSeriesSafetyCountersV4 {
    pub network_request_attempts: usize,
    pub retries: usize,
    pub maximum_concurrency: usize,
    pub transport_constructions: usize,
    pub canonical_raw_row_reads: usize,
    pub prior_private_evaluation_reads: usize,
    pub participant_reconstructions: usize,
    pub feature_generations: usize,
    pub prediction_computations: usize,
    pub parameter_updates: usize,
    pub normalizer_refits: usize,
    pub training_uses: usize,
    pub qualification_uses: usize,
    pub outcome_requests: usize,
    pub outcome_openings: usize,
    pub metric_computations: usize,
    pub winner_selections: usize,
    pub ranking_creations: usize,
    pub reward_applications: usize,
    pub penalty_applications: usize,
    pub chair_decisions: usize,
    pub votes: usize,
    pub voice_changes: usize,
    pub tier_changes: usize,
    pub cooldowns: usize,
    pub promotions: usize,
    pub quarantines: usize,
    pub paper_executions: usize,
    pub live_executions: usize,
    pub active_committee_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveEpochStatusReceiptV4 {
    pub status_version: String,
    pub series_digest: String,
    pub event_one_adoption_digest: String,
    pub candidate_gap_audit_digest: String,
    pub context_delta_plan_digest: String,
    pub epoch_registration_digest: String,
    pub epoch_number: u64,
    pub event_timestamp_ms: u64,
    pub input_finality_boundary_ms: u64,
    pub outcome_timestamp_ms: u64,
    pub outcome_finality_boundary_ms: u64,
    pub exact_context_timestamp_ms: Vec<u64>,
    pub exact_missing_timestamp_ms: Vec<u64>,
    pub readiness: MomentumProspectiveEpochReadinessV4,
    pub input_receipt_digest: Option<String>,
    pub input_capsule_digest: Option<String>,
    pub context_assembly_proof_digest: Option<String>,
    pub participant_prediction_digests: Vec<String>,
    pub prediction_capsule_digest: Option<String>,
    pub journal_entry_digest: Option<String>,
    pub outcome_plan_digest: Option<String>,
    pub total_event_count: usize,
    pub scorable_event_count: usize,
    pub reward_eligibility_status: MomentumRewardEligibilityStatusV4_4,
    pub protected_artifacts_unchanged: bool,
    pub active_state_unchanged: bool,
    pub safety_counters: MomentumProspectiveSeriesSafetyCountersV4,
    pub status_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumProspectiveSeriesReportV4 {
    pub status: MomentumProspectiveEpochStatusReceiptV4,
    pub series: MomentumProspectiveSeriesV4,
    pub event_one_adoption: MomentumProspectiveSeriesAdoptionV4,
    pub candidate_gap_audit: MomentumProspectiveCandidateGapAuditV4,
    pub context_delta_plan: MomentumCanonicalContextDeltaPlanV4,
    pub epoch_registration: MomentumProspectiveEpochRegistrationV4,
    pub input_receipt: Option<MomentumProspectiveSeriesInputReceiptV4>,
    pub input_capsule: Option<MomentumProspectiveSeriesInputCapsuleV4>,
    pub context_use_proof: Option<MomentumSeriesContextUseProofV4>,
    pub context_assembly_proof: Option<MomentumSeriesContextAssemblyProofV4>,
    pub prediction_capsule: Option<MomentumProspectiveSeriesPredictionCapsuleV4>,
    pub journal_entry: Option<MomentumProspectiveSeriesJournalEntryV4>,
    pub outcome_plan: Option<MomentumProspectiveSeriesOutcomePlanV4>,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
}

#[derive(Clone)]
struct CanonicalRowV4 {
    row: HistoricalOhlcvRow,
    reference: MomentumSeriesCanonicalRowRefV4,
}

struct EventOneStateV4 {
    chain: MomentumSealedPredictionChainV4_3,
    outcome: MomentumFutureOutcomeReportV4_4,
    opening: MomentumFutureOutcomeOpeningReportV4_4,
}

fn canonical_digest<T: Clone + Debug>(value: &T, clear: impl FnOnce(&mut T)) -> String {
    let mut canonical = value.clone();
    clear(&mut canonical);
    stable_hash_string(&format!("{canonical:?}"))
}

fn series_digest(value: &MomentumProspectiveSeriesV4) -> String {
    canonical_digest(value, |item| item.series_digest.clear())
}

fn adoption_digest(value: &MomentumProspectiveSeriesAdoptionV4) -> String {
    canonical_digest(value, |item| item.adoption_digest.clear())
}

fn gap_audit_digest(value: &MomentumProspectiveCandidateGapAuditV4) -> String {
    canonical_digest(value, |item| item.audit_digest.clear())
}

fn delta_plan_digest(value: &MomentumCanonicalContextDeltaPlanV4) -> String {
    canonical_digest(value, |item| item.plan_digest.clear())
}

fn epoch_registration_digest(value: &MomentumProspectiveEpochRegistrationV4) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}

fn context_use_entry_digest(value: &MomentumSeriesContextUseEntryV4) -> String {
    canonical_digest(value, |item| item.entry_digest.clear())
}

fn context_use_proof_digest(value: &MomentumSeriesContextUseProofV4) -> String {
    canonical_digest(value, |item| item.proof_digest.clear())
}

fn context_assembly_digest(value: &MomentumSeriesContextAssemblyProofV4) -> String {
    canonical_digest(value, |item| item.proof_digest.clear())
}

fn input_receipt_digest(value: &MomentumProspectiveSeriesInputReceiptV4) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn input_capsule_digest(value: &MomentumProspectiveSeriesInputCapsuleV4) -> String {
    canonical_digest(value, |item| item.capsule_digest.clear())
}

fn prediction_seal_digest(value: &MomentumSeriesParticipantPredictionSealV4) -> String {
    canonical_digest(value, |item| item.seal_digest.clear())
}

fn prediction_capsule_digest(value: &MomentumProspectiveSeriesPredictionCapsuleV4) -> String {
    canonical_digest(value, |item| item.capsule_digest.clear())
}

fn journal_entry_digest(value: &MomentumProspectiveSeriesJournalEntryV4) -> String {
    canonical_digest(value, |item| item.entry_digest.clear())
}

fn outcome_plan_digest(value: &MomentumProspectiveSeriesOutcomePlanV4) -> String {
    canonical_digest(value, |item| item.plan_digest.clear())
}

fn status_digest(value: &MomentumProspectiveEpochStatusReceiptV4) -> String {
    canonical_digest(value, |item| item.status_digest.clear())
}

fn parse_continuation_policy(
    value: &str,
) -> Result<MomentumProspectiveContinuationPolicyV4, String> {
    match value {
        "FixedDailyCadenceNextLegalEvent" => {
            Ok(MomentumProspectiveContinuationPolicyV4::FixedDailyCadenceNextLegalEvent)
        }
        _ => Err("V4 series continuation policy rejected".to_string()),
    }
}

fn parse_candidate_disposition(
    value: &str,
) -> Result<MomentumProspectiveCandidateDispositionV4, String> {
    match value {
        "Eligible" => Ok(MomentumProspectiveCandidateDispositionV4::Eligible),
        "SkippedRegistrationAfterInputFinality" => {
            Ok(MomentumProspectiveCandidateDispositionV4::SkippedRegistrationAfterInputFinality)
        }
        "SkippedPriorOutcomeAlreadyOpened" => {
            Ok(MomentumProspectiveCandidateDispositionV4::SkippedPriorOutcomeAlreadyOpened)
        }
        "SkippedIntegrityConflict" => {
            Ok(MomentumProspectiveCandidateDispositionV4::SkippedIntegrityConflict)
        }
        _ => Err("V4 series candidate disposition rejected".to_string()),
    }
}

fn parse_context_use(value: &str) -> Result<MomentumSeriesContextUseV4, String> {
    match value {
        "ExistingCanonicalHistoricalRaw" => {
            Ok(MomentumSeriesContextUseV4::ExistingCanonicalHistoricalRaw)
        }
        "PriorProtectedRawContext" => Ok(MomentumSeriesContextUseV4::PriorProtectedRawContext),
        "PriorProspectiveEventRawContext" => {
            Ok(MomentumSeriesContextUseV4::PriorProspectiveEventRawContext)
        }
        "PriorOpenedOutcomeRawContext" => {
            Ok(MomentumSeriesContextUseV4::PriorOpenedOutcomeRawContext)
        }
        "NewIncrementalRawContext" => Ok(MomentumSeriesContextUseV4::NewIncrementalRawContext),
        "CurrentProspectiveEventInput" => {
            Ok(MomentumSeriesContextUseV4::CurrentProspectiveEventInput)
        }
        _ => Err("V4 series context use rejected".to_string()),
    }
}

fn parse_readiness(value: &str) -> Result<MomentumProspectiveEpochReadinessV4, String> {
    match value {
        "RegisteredAwaitingInputFinality" => {
            Ok(MomentumProspectiveEpochReadinessV4::RegisteredAwaitingInputFinality)
        }
        "ReadyForInputAcquisition" => {
            Ok(MomentumProspectiveEpochReadinessV4::ReadyForInputAcquisition)
        }
        "ReadyForLocalPredictionRecovery" => {
            Ok(MomentumProspectiveEpochReadinessV4::ReadyForLocalPredictionRecovery)
        }
        "PredictionAlreadySealed" => {
            Ok(MomentumProspectiveEpochReadinessV4::PredictionAlreadySealed)
        }
        "PredictionSealWindowExpired" => {
            Ok(MomentumProspectiveEpochReadinessV4::PredictionSealWindowExpired)
        }
        "PriorInputAttemptTerminal" => {
            Ok(MomentumProspectiveEpochReadinessV4::PriorInputAttemptTerminal)
        }
        "ProspectiveWindowExpired" => {
            Ok(MomentumProspectiveEpochReadinessV4::ProspectiveWindowExpired)
        }
        "MissingCanonicalContext" => {
            Ok(MomentumProspectiveEpochReadinessV4::MissingCanonicalContext)
        }
        "MissingSetNotContiguous" => {
            Ok(MomentumProspectiveEpochReadinessV4::MissingSetNotContiguous)
        }
        "PriorPrivateEvidenceAccessDetected" => {
            Ok(MomentumProspectiveEpochReadinessV4::PriorPrivateEvidenceAccessDetected)
        }
        "FrozenIdentityMismatch" => Ok(MomentumProspectiveEpochReadinessV4::FrozenIdentityMismatch),
        "IntegrityFailure" => Ok(MomentumProspectiveEpochReadinessV4::IntegrityFailure),
        _ => Err("V4 series readiness rejected".to_string()),
    }
}

fn parse_input_status(value: &str) -> Result<MomentumProspectiveSeriesInputStatusV4, String> {
    match value {
        "ReadyNotAttempted" => Ok(MomentumProspectiveSeriesInputStatusV4::ReadyNotAttempted),
        "EvidenceAcquired" => Ok(MomentumProspectiveSeriesInputStatusV4::EvidenceAcquired),
        "TerminalTransportFailure" => {
            Ok(MomentumProspectiveSeriesInputStatusV4::TerminalTransportFailure)
        }
        "TerminalValidationFailure" => {
            Ok(MomentumProspectiveSeriesInputStatusV4::TerminalValidationFailure)
        }
        _ => Err("V4 series input status rejected".to_string()),
    }
}

fn parse_reward_status(value: &str) -> Result<MomentumRewardEligibilityStatusV4_4, String> {
    match value {
        "IneligibleMinimumSamples" => {
            Ok(MomentumRewardEligibilityStatusV4_4::IneligibleMinimumSamples)
        }
        "IneligibleNeutralOutcome" => {
            Ok(MomentumRewardEligibilityStatusV4_4::IneligibleNeutralOutcome)
        }
        "IneligibleIntegrityFailure" => {
            Ok(MomentumRewardEligibilityStatusV4_4::IneligibleIntegrityFailure)
        }
        "EligibleCandidateComputed" => {
            Ok(MomentumRewardEligibilityStatusV4_4::EligibleCandidateComputed)
        }
        _ => Err("V4 series reward status rejected".to_string()),
    }
}

fn validate_series(value: &MomentumProspectiveSeriesV4) -> Result<(), String> {
    if value.series_version != SERIES_VERSION
        || value.agent_id != AGENT_ID
        || value.frozen_roster_digest.is_empty()
        || value.participant_digests.len() != 3
        || value.parameter_digests.len() != 3
        || value.normalizer_digests.len() != 3
        || value.feature_policy_digest.is_empty()
        || value.label_policy_digest.is_empty()
        || value.evaluation_policy_digest.is_empty()
        || value.minimum_sample_policy_digest.is_empty()
        || value.provider_id.is_empty()
        || value.market.is_empty()
        || value.symbol.is_empty()
        || value.cadence_ms != DAILY_CADENCE_MS
        || value.context_row_count == 0
        || value.prediction_horizon != 1
        || value.first_event_ledger_entry_digest.is_empty()
        || value.first_event_opening_bundle_digest.is_empty()
        || value.first_event_eligibility_digest.is_empty()
        || value.continuation_policy
            != MomentumProspectiveContinuationPolicyV4::FixedDailyCadenceNextLegalEvent
        || value.maximum_open_epochs != 1
        || !value.manual_network_confirmation_required
        || !value.automatic_network_execution_forbidden
        || !value.retraining_forbidden
        || !value.participant_selection_forbidden
        || !value.result_conditioned_continuation_forbidden
        || !value.winner_selection_forbidden
        || !value.ranking_forbidden
        || !value.reward_application_forbidden
        || !value.penalty_application_forbidden
        || !value.chair_action_forbidden
        || !value.trading_forbidden
        || value.protected_before_artifact_count == 0
        || value.protected_before_aggregate_digest.is_empty()
        || value.active_agent_state_digest.is_empty()
        || value.series_digest != series_digest(value)
    {
        return Err("V4 prospective series contract rejected".to_string());
    }
    Ok(())
}

fn encode_series(value: &MomentumProspectiveSeriesV4) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("prospective-series")
        .string("series_version", &value.series_version)
        .string("agent_id", &value.agent_id)
        .string("frozen_roster_digest", &value.frozen_roster_digest)
        .strings("participant_digests", &value.participant_digests)
        .strings("parameter_digests", &value.parameter_digests)
        .strings("normalizer_digests", &value.normalizer_digests)
        .string("feature_policy_digest", &value.feature_policy_digest)
        .string("label_policy_digest", &value.label_policy_digest)
        .string("evaluation_policy_digest", &value.evaluation_policy_digest)
        .string(
            "minimum_sample_policy_digest",
            &value.minimum_sample_policy_digest,
        )
        .string("provider_id", &value.provider_id)
        .string("market", &value.market)
        .string("symbol", &value.symbol)
        .unsigned("cadence_ms", value.cadence_ms)
        .unsigned("context_row_count", as_u64(value.context_row_count)?)
        .unsigned("prediction_horizon", as_u64(value.prediction_horizon)?)
        .string(
            "first_event_ledger_entry_digest",
            &value.first_event_ledger_entry_digest,
        )
        .string(
            "first_event_opening_bundle_digest",
            &value.first_event_opening_bundle_digest,
        )
        .string(
            "first_event_eligibility_digest",
            &value.first_event_eligibility_digest,
        )
        .string(
            "continuation_policy",
            format!("{:?}", value.continuation_policy),
        )
        .unsigned("maximum_open_epochs", as_u64(value.maximum_open_epochs)?)
        .boolean(
            "manual_network_confirmation_required",
            value.manual_network_confirmation_required,
        )
        .boolean(
            "automatic_network_execution_forbidden",
            value.automatic_network_execution_forbidden,
        )
        .boolean("retraining_forbidden", value.retraining_forbidden)
        .boolean(
            "participant_selection_forbidden",
            value.participant_selection_forbidden,
        )
        .boolean(
            "result_conditioned_continuation_forbidden",
            value.result_conditioned_continuation_forbidden,
        )
        .boolean(
            "winner_selection_forbidden",
            value.winner_selection_forbidden,
        )
        .boolean("ranking_forbidden", value.ranking_forbidden)
        .boolean(
            "reward_application_forbidden",
            value.reward_application_forbidden,
        )
        .boolean(
            "penalty_application_forbidden",
            value.penalty_application_forbidden,
        )
        .boolean("chair_action_forbidden", value.chair_action_forbidden)
        .boolean("trading_forbidden", value.trading_forbidden)
        .unsigned(
            "protected_before_artifact_count",
            as_u64(value.protected_before_artifact_count)?,
        )
        .string(
            "protected_before_aggregate_digest",
            &value.protected_before_aggregate_digest,
        )
        .string(
            "active_agent_state_digest",
            &value.active_agent_state_digest,
        )
        .string("series_digest", &value.series_digest)
        .encode()
}

fn decode_series(bytes: &[u8]) -> Result<MomentumProspectiveSeriesV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "prospective-series")?;
    let value = MomentumProspectiveSeriesV4 {
        series_version: fields.string("series_version")?,
        agent_id: fields.string("agent_id")?,
        frozen_roster_digest: fields.string("frozen_roster_digest")?,
        participant_digests: fields.strings("participant_digests")?,
        parameter_digests: fields.strings("parameter_digests")?,
        normalizer_digests: fields.strings("normalizer_digests")?,
        feature_policy_digest: fields.string("feature_policy_digest")?,
        label_policy_digest: fields.string("label_policy_digest")?,
        evaluation_policy_digest: fields.string("evaluation_policy_digest")?,
        minimum_sample_policy_digest: fields.string("minimum_sample_policy_digest")?,
        provider_id: fields.string("provider_id")?,
        market: fields.string("market")?,
        symbol: fields.string("symbol")?,
        cadence_ms: fields.unsigned("cadence_ms")?,
        context_row_count: as_usize(fields.unsigned("context_row_count")?)?,
        prediction_horizon: as_usize(fields.unsigned("prediction_horizon")?)?,
        first_event_ledger_entry_digest: fields.string("first_event_ledger_entry_digest")?,
        first_event_opening_bundle_digest: fields.string("first_event_opening_bundle_digest")?,
        first_event_eligibility_digest: fields.string("first_event_eligibility_digest")?,
        continuation_policy: parse_continuation_policy(&fields.string("continuation_policy")?)?,
        maximum_open_epochs: as_usize(fields.unsigned("maximum_open_epochs")?)?,
        manual_network_confirmation_required: fields
            .boolean("manual_network_confirmation_required")?,
        automatic_network_execution_forbidden: fields
            .boolean("automatic_network_execution_forbidden")?,
        retraining_forbidden: fields.boolean("retraining_forbidden")?,
        participant_selection_forbidden: fields.boolean("participant_selection_forbidden")?,
        result_conditioned_continuation_forbidden: fields
            .boolean("result_conditioned_continuation_forbidden")?,
        winner_selection_forbidden: fields.boolean("winner_selection_forbidden")?,
        ranking_forbidden: fields.boolean("ranking_forbidden")?,
        reward_application_forbidden: fields.boolean("reward_application_forbidden")?,
        penalty_application_forbidden: fields.boolean("penalty_application_forbidden")?,
        chair_action_forbidden: fields.boolean("chair_action_forbidden")?,
        trading_forbidden: fields.boolean("trading_forbidden")?,
        protected_before_artifact_count: as_usize(
            fields.unsigned("protected_before_artifact_count")?,
        )?,
        protected_before_aggregate_digest: fields.string("protected_before_aggregate_digest")?,
        active_agent_state_digest: fields.string("active_agent_state_digest")?,
        series_digest: fields.string("series_digest")?,
    };
    fields.finish()?;
    validate_series(&value)?;
    Ok(value)
}

fn validate_adoption(value: &MomentumProspectiveSeriesAdoptionV4) -> Result<(), String> {
    if value.adoption_version != ADOPTION_VERSION
        || value.series_digest.is_empty()
        || value.adopted_epoch_number != 1
        || value.adopted_event_timestamp_ms == 0
        || value.prediction_capsule_digest.is_empty()
        || value.outcome_capsule_digest.is_empty()
        || value.opening_bundle_digest.is_empty()
        || value.evaluation_ledger_entry_digest.is_empty()
        || value.reward_eligibility_digest.is_empty()
        || value.total_event_count == 0
        || value.scorable_event_count > value.total_event_count
        || value.winner_selected
        || value.ranking_created
        || value.reward_applied
        || value.penalty_applied
        || value.chair_action_taken
        || value.adoption_digest != adoption_digest(value)
    {
        return Err("V4 prospective series adoption rejected".to_string());
    }
    Ok(())
}

fn encode_adoption(value: &MomentumProspectiveSeriesAdoptionV4) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("series-adoption")
        .string("adoption_version", &value.adoption_version)
        .string("series_digest", &value.series_digest)
        .unsigned("adopted_epoch_number", value.adopted_epoch_number)
        .unsigned(
            "adopted_event_timestamp_ms",
            value.adopted_event_timestamp_ms,
        )
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .string("outcome_capsule_digest", &value.outcome_capsule_digest)
        .string("opening_bundle_digest", &value.opening_bundle_digest)
        .string(
            "evaluation_ledger_entry_digest",
            &value.evaluation_ledger_entry_digest,
        )
        .string(
            "reward_eligibility_digest",
            &value.reward_eligibility_digest,
        )
        .unsigned("total_event_count", as_u64(value.total_event_count)?)
        .unsigned("scorable_event_count", as_u64(value.scorable_event_count)?)
        .boolean("winner_selected", value.winner_selected)
        .boolean("ranking_created", value.ranking_created)
        .boolean("reward_applied", value.reward_applied)
        .boolean("penalty_applied", value.penalty_applied)
        .boolean("chair_action_taken", value.chair_action_taken)
        .string("adoption_digest", &value.adoption_digest)
        .encode()
}

fn decode_adoption(bytes: &[u8]) -> Result<MomentumProspectiveSeriesAdoptionV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "series-adoption")?;
    let value = MomentumProspectiveSeriesAdoptionV4 {
        adoption_version: fields.string("adoption_version")?,
        series_digest: fields.string("series_digest")?,
        adopted_epoch_number: fields.unsigned("adopted_epoch_number")?,
        adopted_event_timestamp_ms: fields.unsigned("adopted_event_timestamp_ms")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        outcome_capsule_digest: fields.string("outcome_capsule_digest")?,
        opening_bundle_digest: fields.string("opening_bundle_digest")?,
        evaluation_ledger_entry_digest: fields.string("evaluation_ledger_entry_digest")?,
        reward_eligibility_digest: fields.string("reward_eligibility_digest")?,
        total_event_count: as_usize(fields.unsigned("total_event_count")?)?,
        scorable_event_count: as_usize(fields.unsigned("scorable_event_count")?)?,
        winner_selected: fields.boolean("winner_selected")?,
        ranking_created: fields.boolean("ranking_created")?,
        reward_applied: fields.boolean("reward_applied")?,
        penalty_applied: fields.boolean("penalty_applied")?,
        chair_action_taken: fields.boolean("chair_action_taken")?,
        adoption_digest: fields.string("adoption_digest")?,
    };
    fields.finish()?;
    validate_adoption(&value)?;
    Ok(value)
}

fn validate_gap_audit(value: &MomentumProspectiveCandidateGapAuditV4) -> Result<(), String> {
    let expected = if value.prior_outcome_already_opened {
        MomentumProspectiveCandidateDispositionV4::SkippedPriorOutcomeAlreadyOpened
    } else if value.registration_after_input_finality {
        MomentumProspectiveCandidateDispositionV4::SkippedRegistrationAfterInputFinality
    } else {
        MomentumProspectiveCandidateDispositionV4::Eligible
    };
    let mut expected_reasons = Vec::new();
    if value.registration_after_input_finality {
        expected_reasons.push("RegistrationAfterInputFinality".to_string());
    }
    if value.prior_outcome_already_opened {
        expected_reasons.push("PriorOutcomeAlreadyOpened".to_string());
    }
    if value.audit_version != GAP_AUDIT_VERSION
        || value.series_digest.is_empty()
        || value.prior_event_timestamp_ms == 0
        || value.adjacent_candidate_timestamp_ms
            != value
                .prior_event_timestamp_ms
                .checked_add(DAILY_CADENCE_MS)
                .unwrap_or_default()
        || value.candidate_input_finality_boundary_ms
            != value
                .adjacent_candidate_timestamp_ms
                .checked_add(DAILY_CADENCE_MS)
                .unwrap_or_default()
        || value.canonical_disposition != expected
        || value.applicable_reasons != expected_reasons
        || value.counted_as_model_failure
        || value.reward_or_penalty_consequence
        || value.audit_digest != gap_audit_digest(value)
    {
        return Err("V4 prospective adjacent-candidate audit rejected".to_string());
    }
    Ok(())
}

fn encode_gap_audit(value: &MomentumProspectiveCandidateGapAuditV4) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("candidate-gap-audit")
        .string("audit_version", &value.audit_version)
        .string("series_digest", &value.series_digest)
        .unsigned("prior_event_timestamp_ms", value.prior_event_timestamp_ms)
        .unsigned(
            "adjacent_candidate_timestamp_ms",
            value.adjacent_candidate_timestamp_ms,
        )
        .unsigned(
            "registration_observed_at_ms",
            value.registration_observed_at_ms,
        )
        .unsigned(
            "candidate_input_finality_boundary_ms",
            value.candidate_input_finality_boundary_ms,
        )
        .boolean(
            "registration_after_input_finality",
            value.registration_after_input_finality,
        )
        .boolean(
            "prior_outcome_already_opened",
            value.prior_outcome_already_opened,
        )
        .strings("applicable_reasons", &value.applicable_reasons)
        .string(
            "canonical_disposition",
            format!("{:?}", value.canonical_disposition),
        )
        .boolean("counted_as_model_failure", value.counted_as_model_failure)
        .boolean(
            "reward_or_penalty_consequence",
            value.reward_or_penalty_consequence,
        )
        .string("audit_digest", &value.audit_digest)
        .encode()
}

fn decode_gap_audit(bytes: &[u8]) -> Result<MomentumProspectiveCandidateGapAuditV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "candidate-gap-audit")?;
    let value = MomentumProspectiveCandidateGapAuditV4 {
        audit_version: fields.string("audit_version")?,
        series_digest: fields.string("series_digest")?,
        prior_event_timestamp_ms: fields.unsigned("prior_event_timestamp_ms")?,
        adjacent_candidate_timestamp_ms: fields.unsigned("adjacent_candidate_timestamp_ms")?,
        registration_observed_at_ms: fields.unsigned("registration_observed_at_ms")?,
        candidate_input_finality_boundary_ms: fields
            .unsigned("candidate_input_finality_boundary_ms")?,
        registration_after_input_finality: fields.boolean("registration_after_input_finality")?,
        prior_outcome_already_opened: fields.boolean("prior_outcome_already_opened")?,
        applicable_reasons: fields.strings("applicable_reasons")?,
        canonical_disposition: parse_candidate_disposition(
            &fields.string("canonical_disposition")?,
        )?,
        counted_as_model_failure: fields.boolean("counted_as_model_failure")?,
        reward_or_penalty_consequence: fields.boolean("reward_or_penalty_consequence")?,
        audit_digest: fields.string("audit_digest")?,
    };
    fields.finish()?;
    validate_gap_audit(&value)?;
    Ok(value)
}

fn encode_row_ref(value: &MomentumSeriesCanonicalRowRefV4) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("canonical-row-ref")
        .unsigned("timestamp_ms", value.timestamp_ms)
        .string("raw_row_digest", &value.raw_row_digest)
        .string("source_capsule_digest", &value.source_capsule_digest)
        .string("use_class", format!("{:?}", value.use_class))
        .encode()
}

fn decode_row_ref(bytes: &[u8]) -> Result<MomentumSeriesCanonicalRowRefV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "canonical-row-ref")?;
    let value = MomentumSeriesCanonicalRowRefV4 {
        timestamp_ms: fields.unsigned("timestamp_ms")?,
        raw_row_digest: fields.string("raw_row_digest")?,
        source_capsule_digest: fields.string("source_capsule_digest")?,
        use_class: parse_context_use(&fields.string("use_class")?)?,
    };
    fields.finish()?;
    if value.timestamp_ms == 0
        || value.raw_row_digest.is_empty()
        || value.source_capsule_digest.is_empty()
    {
        return Err("V4 series canonical row reference rejected".to_string());
    }
    Ok(value)
}

fn validate_delta_plan(value: &MomentumCanonicalContextDeltaPlanV4) -> Result<(), String> {
    let expected = value
        .canonical_rows
        .iter()
        .map(|entry| entry.timestamp_ms)
        .chain(value.exact_missing_timestamp_ms.iter().copied())
        .collect::<BTreeSet<_>>();
    let exact = value
        .exact_context_timestamp_ms
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let canonical_timestamp_count = value
        .canonical_rows
        .iter()
        .map(|entry| entry.timestamp_ms)
        .collect::<BTreeSet<_>>()
        .len();
    let missing_timestamp_count = value
        .exact_missing_timestamp_ms
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .len();
    if value.plan_version != DELTA_PLAN_VERSION
        || value.series_digest.is_empty()
        || value.epoch_number < 2
        || value.event_timestamp_ms == 0
        || value.exact_context_timestamp_ms.is_empty()
        || exact.len() != value.exact_context_timestamp_ms.len()
        || canonical_timestamp_count != value.canonical_rows.len()
        || missing_timestamp_count != value.exact_missing_timestamp_ms.len()
        || value
            .exact_context_timestamp_ms
            .windows(2)
            .any(|pair| pair[1] != pair[0].saturating_add(DAILY_CADENCE_MS))
        || value.exact_context_timestamp_ms.last() != Some(&value.event_timestamp_ms)
        || expected != exact
        || value.exact_missing_timestamp_ms.is_empty()
        || !missing_set_is_contiguous(&value.exact_missing_timestamp_ms, DAILY_CADENCE_MS)
        || value.maximum_requests != 1
        || value.maximum_retries != 0
        || value.maximum_concurrency != 1
        || !value.full_context_refetch_forbidden
        || value.prior_private_evaluation_accessed
        || !value.missing_set_contiguous
        || value.plan_digest != delta_plan_digest(value)
    {
        return Err("V4 prospective context delta plan rejected".to_string());
    }
    Ok(())
}

fn encode_delta_plan(value: &MomentumCanonicalContextDeltaPlanV4) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("context-delta-plan")
        .string("plan_version", &value.plan_version)
        .string("series_digest", &value.series_digest)
        .unsigned("epoch_number", value.epoch_number)
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigneds(
            "exact_context_timestamp_ms",
            &value.exact_context_timestamp_ms,
        )
        .messages(
            "canonical_rows",
            value
                .canonical_rows
                .iter()
                .map(encode_row_ref)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .unsigneds(
            "exact_missing_timestamp_ms",
            &value.exact_missing_timestamp_ms,
        )
        .unsigned("maximum_requests", as_u64(value.maximum_requests)?)
        .unsigned("maximum_retries", as_u64(value.maximum_retries)?)
        .unsigned("maximum_concurrency", as_u64(value.maximum_concurrency)?)
        .boolean(
            "full_context_refetch_forbidden",
            value.full_context_refetch_forbidden,
        )
        .boolean(
            "prior_private_evaluation_accessed",
            value.prior_private_evaluation_accessed,
        )
        .boolean("missing_set_contiguous", value.missing_set_contiguous)
        .string("plan_digest", &value.plan_digest)
        .encode()
}

fn decode_delta_plan(bytes: &[u8]) -> Result<MomentumCanonicalContextDeltaPlanV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "context-delta-plan")?;
    let value = MomentumCanonicalContextDeltaPlanV4 {
        plan_version: fields.string("plan_version")?,
        series_digest: fields.string("series_digest")?,
        epoch_number: fields.unsigned("epoch_number")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        exact_context_timestamp_ms: fields.unsigneds("exact_context_timestamp_ms")?,
        canonical_rows: fields
            .messages("canonical_rows")?
            .iter()
            .map(|bytes| decode_row_ref(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        exact_missing_timestamp_ms: fields.unsigneds("exact_missing_timestamp_ms")?,
        maximum_requests: as_usize(fields.unsigned("maximum_requests")?)?,
        maximum_retries: as_usize(fields.unsigned("maximum_retries")?)?,
        maximum_concurrency: as_usize(fields.unsigned("maximum_concurrency")?)?,
        full_context_refetch_forbidden: fields.boolean("full_context_refetch_forbidden")?,
        prior_private_evaluation_accessed: fields.boolean("prior_private_evaluation_accessed")?,
        missing_set_contiguous: fields.boolean("missing_set_contiguous")?,
        plan_digest: fields.string("plan_digest")?,
    };
    fields.finish()?;
    validate_delta_plan(&value)?;
    Ok(value)
}

fn validate_epoch_registration(
    value: &MomentumProspectiveEpochRegistrationV4,
) -> Result<(), String> {
    if value.registration_version != EPOCH_REGISTRATION_VERSION
        || value.series_digest.is_empty()
        || value.epoch_number < 2
        || value.previous_epoch_ledger_entry_digest.is_empty()
        || value.context_delta_plan_digest.is_empty()
        || value.previous_epoch_opening_bundle_digest.is_empty()
        || value.registration_created_at_ms >= value.input_finality_boundary_ms
        || value.input_finality_boundary_ms
            != value
                .event_timestamp_ms
                .checked_add(DAILY_CADENCE_MS)
                .unwrap_or_default()
        || value.outcome_timestamp_ms != value.input_finality_boundary_ms
        || value.outcome_finality_boundary_ms
            != value
                .outcome_timestamp_ms
                .checked_add(DAILY_CADENCE_MS)
                .unwrap_or_default()
        || value.exact_context_timestamp_ms.is_empty()
        || value.exact_context_timestamp_ms.last() != Some(&value.event_timestamp_ms)
        || value.exact_missing_timestamp_ms.is_empty()
        || value.context_delta_plan_digest.is_empty()
        || value.provider_id.is_empty()
        || value.market.is_empty()
        || value.symbol.is_empty()
        || value.cadence != "1d"
        || value.maximum_input_requests != 1
        || value.maximum_input_retries != 0
        || value.maximum_input_concurrency != 1
        || value.maximum_response_bytes == 0
        || !value.prior_private_evaluation_access_forbidden
        || !value.parameter_update_forbidden
        || !value.normalizer_refit_forbidden
        || !value.outcome_access_forbidden
        || !value.winner_selection_forbidden
        || !value.ranking_forbidden
        || !value.reward_application_forbidden
        || !value.penalty_application_forbidden
        || !value.chair_action_forbidden
        || !value.trading_forbidden
        || value.registration_digest != epoch_registration_digest(value)
    {
        return Err("V4 prospective epoch registration rejected".to_string());
    }
    Ok(())
}

fn encode_epoch_registration(
    value: &MomentumProspectiveEpochRegistrationV4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("epoch-registration")
        .string("registration_version", &value.registration_version)
        .string("series_digest", &value.series_digest)
        .unsigned("epoch_number", value.epoch_number)
        .string(
            "previous_epoch_ledger_entry_digest",
            &value.previous_epoch_ledger_entry_digest,
        )
        .string(
            "previous_epoch_opening_bundle_digest",
            &value.previous_epoch_opening_bundle_digest,
        )
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigned(
            "registration_created_at_ms",
            value.registration_created_at_ms,
        )
        .unsigned(
            "input_finality_boundary_ms",
            value.input_finality_boundary_ms,
        )
        .unsigned("outcome_timestamp_ms", value.outcome_timestamp_ms)
        .unsigned(
            "outcome_finality_boundary_ms",
            value.outcome_finality_boundary_ms,
        )
        .unsigneds(
            "exact_context_timestamp_ms",
            &value.exact_context_timestamp_ms,
        )
        .unsigneds(
            "exact_missing_timestamp_ms",
            &value.exact_missing_timestamp_ms,
        )
        .string(
            "context_delta_plan_digest",
            &value.context_delta_plan_digest,
        )
        .string("provider_id", &value.provider_id)
        .string("market", &value.market)
        .string("symbol", &value.symbol)
        .string("cadence", &value.cadence)
        .unsigned(
            "maximum_input_requests",
            as_u64(value.maximum_input_requests)?,
        )
        .unsigned(
            "maximum_input_retries",
            as_u64(value.maximum_input_retries)?,
        )
        .unsigned(
            "maximum_input_concurrency",
            as_u64(value.maximum_input_concurrency)?,
        )
        .unsigned(
            "maximum_response_bytes",
            as_u64(value.maximum_response_bytes)?,
        )
        .boolean(
            "prior_private_evaluation_access_forbidden",
            value.prior_private_evaluation_access_forbidden,
        )
        .boolean(
            "parameter_update_forbidden",
            value.parameter_update_forbidden,
        )
        .boolean(
            "normalizer_refit_forbidden",
            value.normalizer_refit_forbidden,
        )
        .boolean("outcome_access_forbidden", value.outcome_access_forbidden)
        .boolean(
            "winner_selection_forbidden",
            value.winner_selection_forbidden,
        )
        .boolean("ranking_forbidden", value.ranking_forbidden)
        .boolean(
            "reward_application_forbidden",
            value.reward_application_forbidden,
        )
        .boolean(
            "penalty_application_forbidden",
            value.penalty_application_forbidden,
        )
        .boolean("chair_action_forbidden", value.chair_action_forbidden)
        .boolean("trading_forbidden", value.trading_forbidden)
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_epoch_registration(
    bytes: &[u8],
) -> Result<MomentumProspectiveEpochRegistrationV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "epoch-registration")?;
    let value = MomentumProspectiveEpochRegistrationV4 {
        registration_version: fields.string("registration_version")?,
        series_digest: fields.string("series_digest")?,
        epoch_number: fields.unsigned("epoch_number")?,
        previous_epoch_ledger_entry_digest: fields.string("previous_epoch_ledger_entry_digest")?,
        previous_epoch_opening_bundle_digest: fields
            .string("previous_epoch_opening_bundle_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        registration_created_at_ms: fields.unsigned("registration_created_at_ms")?,
        input_finality_boundary_ms: fields.unsigned("input_finality_boundary_ms")?,
        outcome_timestamp_ms: fields.unsigned("outcome_timestamp_ms")?,
        outcome_finality_boundary_ms: fields.unsigned("outcome_finality_boundary_ms")?,
        exact_context_timestamp_ms: fields.unsigneds("exact_context_timestamp_ms")?,
        exact_missing_timestamp_ms: fields.unsigneds("exact_missing_timestamp_ms")?,
        context_delta_plan_digest: fields.string("context_delta_plan_digest")?,
        provider_id: fields.string("provider_id")?,
        market: fields.string("market")?,
        symbol: fields.string("symbol")?,
        cadence: fields.string("cadence")?,
        maximum_input_requests: as_usize(fields.unsigned("maximum_input_requests")?)?,
        maximum_input_retries: as_usize(fields.unsigned("maximum_input_retries")?)?,
        maximum_input_concurrency: as_usize(fields.unsigned("maximum_input_concurrency")?)?,
        maximum_response_bytes: as_usize(fields.unsigned("maximum_response_bytes")?)?,
        prior_private_evaluation_access_forbidden: fields
            .boolean("prior_private_evaluation_access_forbidden")?,
        parameter_update_forbidden: fields.boolean("parameter_update_forbidden")?,
        normalizer_refit_forbidden: fields.boolean("normalizer_refit_forbidden")?,
        outcome_access_forbidden: fields.boolean("outcome_access_forbidden")?,
        winner_selection_forbidden: fields.boolean("winner_selection_forbidden")?,
        ranking_forbidden: fields.boolean("ranking_forbidden")?,
        reward_application_forbidden: fields.boolean("reward_application_forbidden")?,
        penalty_application_forbidden: fields.boolean("penalty_application_forbidden")?,
        chair_action_forbidden: fields.boolean("chair_action_forbidden")?,
        trading_forbidden: fields.boolean("trading_forbidden")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_epoch_registration(&value)?;
    Ok(value)
}

fn validate_context_use_entry(value: &MomentumSeriesContextUseEntryV4) -> Result<(), String> {
    if value.timestamp_ms == 0
        || value.raw_row_digest.is_empty()
        || value.source_capsule_digest.is_empty()
        || !value.feature_construction_allowed
        || !value.training_forbidden
        || !value.normalizer_fitting_forbidden
        || !value.label_use_forbidden
        || !value.metric_use_forbidden
        || !value.reward_use_forbidden
        || !value.participant_selection_forbidden
        || value.entry_digest != context_use_entry_digest(value)
    {
        return Err("V4 series context-use entry rejected".to_string());
    }
    Ok(())
}

fn encode_context_use_entry(value: &MomentumSeriesContextUseEntryV4) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("context-use-entry")
        .unsigned("timestamp_ms", value.timestamp_ms)
        .string("raw_row_digest", &value.raw_row_digest)
        .string("source_capsule_digest", &value.source_capsule_digest)
        .string("use_class", format!("{:?}", value.use_class))
        .boolean(
            "feature_construction_allowed",
            value.feature_construction_allowed,
        )
        .boolean("training_forbidden", value.training_forbidden)
        .boolean(
            "normalizer_fitting_forbidden",
            value.normalizer_fitting_forbidden,
        )
        .boolean("label_use_forbidden", value.label_use_forbidden)
        .boolean("metric_use_forbidden", value.metric_use_forbidden)
        .boolean("reward_use_forbidden", value.reward_use_forbidden)
        .boolean(
            "participant_selection_forbidden",
            value.participant_selection_forbidden,
        )
        .string("entry_digest", &value.entry_digest)
        .encode()
}

fn decode_context_use_entry(bytes: &[u8]) -> Result<MomentumSeriesContextUseEntryV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "context-use-entry")?;
    let value = MomentumSeriesContextUseEntryV4 {
        timestamp_ms: fields.unsigned("timestamp_ms")?,
        raw_row_digest: fields.string("raw_row_digest")?,
        source_capsule_digest: fields.string("source_capsule_digest")?,
        use_class: parse_context_use(&fields.string("use_class")?)?,
        feature_construction_allowed: fields.boolean("feature_construction_allowed")?,
        training_forbidden: fields.boolean("training_forbidden")?,
        normalizer_fitting_forbidden: fields.boolean("normalizer_fitting_forbidden")?,
        label_use_forbidden: fields.boolean("label_use_forbidden")?,
        metric_use_forbidden: fields.boolean("metric_use_forbidden")?,
        reward_use_forbidden: fields.boolean("reward_use_forbidden")?,
        participant_selection_forbidden: fields.boolean("participant_selection_forbidden")?,
        entry_digest: fields.string("entry_digest")?,
    };
    fields.finish()?;
    validate_context_use_entry(&value)?;
    Ok(value)
}

fn validate_context_use_proof(value: &MomentumSeriesContextUseProofV4) -> Result<(), String> {
    if value.proof_version != CONTEXT_USE_PROOF_VERSION
        || value.series_digest.is_empty()
        || value.epoch_registration_digest.is_empty()
        || value.entries.is_empty()
        || value
            .entries
            .iter()
            .any(|entry| validate_context_use_entry(entry).is_err())
        || value.prior_opening_bundle_used_as_raw_source
        || value.prior_private_scores_accessed
        || value.prior_label_used_as_feature
        || value.reward_eligibility_used_as_feature
        || value.proof_digest != context_use_proof_digest(value)
    {
        return Err("V4 series context-use proof rejected".to_string());
    }
    Ok(())
}

fn encode_context_use_proof(value: &MomentumSeriesContextUseProofV4) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("context-use-proof")
        .string("proof_version", &value.proof_version)
        .string("series_digest", &value.series_digest)
        .string(
            "epoch_registration_digest",
            &value.epoch_registration_digest,
        )
        .messages(
            "entries",
            value
                .entries
                .iter()
                .map(encode_context_use_entry)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .boolean(
            "prior_opening_bundle_used_as_raw_source",
            value.prior_opening_bundle_used_as_raw_source,
        )
        .boolean(
            "prior_private_scores_accessed",
            value.prior_private_scores_accessed,
        )
        .boolean(
            "prior_label_used_as_feature",
            value.prior_label_used_as_feature,
        )
        .boolean(
            "reward_eligibility_used_as_feature",
            value.reward_eligibility_used_as_feature,
        )
        .string("proof_digest", &value.proof_digest)
        .encode()
}

fn decode_context_use_proof(bytes: &[u8]) -> Result<MomentumSeriesContextUseProofV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "context-use-proof")?;
    let value = MomentumSeriesContextUseProofV4 {
        proof_version: fields.string("proof_version")?,
        series_digest: fields.string("series_digest")?,
        epoch_registration_digest: fields.string("epoch_registration_digest")?,
        entries: fields
            .messages("entries")?
            .iter()
            .map(|bytes| decode_context_use_entry(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        prior_opening_bundle_used_as_raw_source: fields
            .boolean("prior_opening_bundle_used_as_raw_source")?,
        prior_private_scores_accessed: fields.boolean("prior_private_scores_accessed")?,
        prior_label_used_as_feature: fields.boolean("prior_label_used_as_feature")?,
        reward_eligibility_used_as_feature: fields.boolean("reward_eligibility_used_as_feature")?,
        proof_digest: fields.string("proof_digest")?,
    };
    fields.finish()?;
    validate_context_use_proof(&value)?;
    Ok(value)
}

fn validate_context_assembly(value: &MomentumSeriesContextAssemblyProofV4) -> Result<(), String> {
    if value.proof_version != CONTEXT_ASSEMBLY_VERSION
        || value.series_digest.is_empty()
        || value.epoch_registration_digest.is_empty()
        || value.input_capsule_digest.is_empty()
        || value.context_use_proof_digest.is_empty()
        || value.exact_context_timestamp_ms.len() != value.exact_row_count
        || value.exact_row_digests.len() != value.exact_row_count
        || value.exact_row_count == 0
        || !value.strict_chronology_verified
        || !value.all_row_digests_verified
        || !value.event_timestamp_is_last
        || !value.outcome_timestamp_absent
        || value.proof_digest != context_assembly_digest(value)
    {
        return Err("V4 series context assembly proof rejected".to_string());
    }
    Ok(())
}

fn encode_context_assembly(
    value: &MomentumSeriesContextAssemblyProofV4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("context-assembly-proof")
        .string("proof_version", &value.proof_version)
        .string("series_digest", &value.series_digest)
        .string(
            "epoch_registration_digest",
            &value.epoch_registration_digest,
        )
        .string("input_capsule_digest", &value.input_capsule_digest)
        .string("context_use_proof_digest", &value.context_use_proof_digest)
        .unsigneds(
            "exact_context_timestamp_ms",
            &value.exact_context_timestamp_ms,
        )
        .strings("exact_row_digests", &value.exact_row_digests)
        .unsigned("exact_row_count", as_u64(value.exact_row_count)?)
        .boolean(
            "strict_chronology_verified",
            value.strict_chronology_verified,
        )
        .boolean("all_row_digests_verified", value.all_row_digests_verified)
        .boolean("event_timestamp_is_last", value.event_timestamp_is_last)
        .boolean("outcome_timestamp_absent", value.outcome_timestamp_absent)
        .string("proof_digest", &value.proof_digest)
        .encode()
}

fn decode_context_assembly(bytes: &[u8]) -> Result<MomentumSeriesContextAssemblyProofV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "context-assembly-proof")?;
    let value = MomentumSeriesContextAssemblyProofV4 {
        proof_version: fields.string("proof_version")?,
        series_digest: fields.string("series_digest")?,
        epoch_registration_digest: fields.string("epoch_registration_digest")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        context_use_proof_digest: fields.string("context_use_proof_digest")?,
        exact_context_timestamp_ms: fields.unsigneds("exact_context_timestamp_ms")?,
        exact_row_digests: fields.strings("exact_row_digests")?,
        exact_row_count: as_usize(fields.unsigned("exact_row_count")?)?,
        strict_chronology_verified: fields.boolean("strict_chronology_verified")?,
        all_row_digests_verified: fields.boolean("all_row_digests_verified")?,
        event_timestamp_is_last: fields.boolean("event_timestamp_is_last")?,
        outcome_timestamp_absent: fields.boolean("outcome_timestamp_absent")?,
        proof_digest: fields.string("proof_digest")?,
    };
    fields.finish()?;
    validate_context_assembly(&value)?;
    Ok(value)
}

fn validate_input_receipt(value: &MomentumProspectiveSeriesInputReceiptV4) -> Result<(), String> {
    let attempted = value.status != MomentumProspectiveSeriesInputStatusV4::ReadyNotAttempted;
    let status_fields_valid = match value.status {
        MomentumProspectiveSeriesInputStatusV4::ReadyNotAttempted => {
            value.http_status_class.is_none()
                && value.returned_row_count == 0
                && value.verified_row_count == 0
                && value.raw_response_digest.is_none()
                && value.input_capsule_digest.is_none()
        }
        MomentumProspectiveSeriesInputStatusV4::EvidenceAcquired => {
            value.http_status_class.as_deref() == Some("2xx")
                && value.returned_row_count > 0
                && value.verified_row_count == value.returned_row_count
                && value.raw_response_digest.is_some()
                && value.input_capsule_digest.is_some()
        }
        MomentumProspectiveSeriesInputStatusV4::TerminalTransportFailure => {
            value.http_status_class.as_deref() != Some("2xx")
                && value.returned_row_count == 0
                && value.verified_row_count == 0
                && value.raw_response_digest.is_none()
                && value.input_capsule_digest.is_none()
        }
        MomentumProspectiveSeriesInputStatusV4::TerminalValidationFailure => {
            value.verified_row_count == 0
                && value.raw_response_digest.is_none()
                && value.input_capsule_digest.is_none()
        }
    };
    if value.receipt_version != INPUT_RECEIPT_VERSION
        || value.series_digest.is_empty()
        || value.epoch_registration_digest.is_empty()
        || value.request_attempted != attempted
        || value.request_count != usize::from(attempted)
        || value.retry_count != 0
        || value.transport_construction_count != usize::from(attempted)
        || value.verified_row_count > value.returned_row_count
        || !status_fields_valid
        || value.terminal != attempted
        || value.receipt_digest != input_receipt_digest(value)
    {
        return Err("V4 series input receipt rejected".to_string());
    }
    Ok(())
}

fn encode_input_receipt(
    value: &MomentumProspectiveSeriesInputReceiptV4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("series-input-receipt")
        .string("receipt_version", &value.receipt_version)
        .string("series_digest", &value.series_digest)
        .string(
            "epoch_registration_digest",
            &value.epoch_registration_digest,
        )
        .boolean("request_attempted", value.request_attempted)
        .unsigned("request_count", as_u64(value.request_count)?)
        .unsigned("retry_count", as_u64(value.retry_count)?)
        .unsigned(
            "transport_construction_count",
            as_u64(value.transport_construction_count)?,
        )
        .string("status", format!("{:?}", value.status))
        .optional_string("http_status_class", &value.http_status_class)
        .unsigned("returned_row_count", as_u64(value.returned_row_count)?)
        .unsigned("verified_row_count", as_u64(value.verified_row_count)?)
        .optional_string("raw_response_digest", &value.raw_response_digest)
        .optional_string("input_capsule_digest", &value.input_capsule_digest)
        .boolean("terminal", value.terminal)
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_input_receipt(bytes: &[u8]) -> Result<MomentumProspectiveSeriesInputReceiptV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "series-input-receipt")?;
    let value = MomentumProspectiveSeriesInputReceiptV4 {
        receipt_version: fields.string("receipt_version")?,
        series_digest: fields.string("series_digest")?,
        epoch_registration_digest: fields.string("epoch_registration_digest")?,
        request_attempted: fields.boolean("request_attempted")?,
        request_count: as_usize(fields.unsigned("request_count")?)?,
        retry_count: as_usize(fields.unsigned("retry_count")?)?,
        transport_construction_count: as_usize(fields.unsigned("transport_construction_count")?)?,
        status: parse_input_status(&fields.string("status")?)?,
        http_status_class: fields.optional_string("http_status_class")?,
        returned_row_count: as_usize(fields.unsigned("returned_row_count")?)?,
        verified_row_count: as_usize(fields.unsigned("verified_row_count")?)?,
        raw_response_digest: fields.optional_string("raw_response_digest")?,
        input_capsule_digest: fields.optional_string("input_capsule_digest")?,
        terminal: fields.boolean("terminal")?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_input_receipt(&value)?;
    Ok(value)
}

fn validate_input_capsule(value: &MomentumProspectiveSeriesInputCapsuleV4) -> Result<(), String> {
    if value.capsule_version != INPUT_CAPSULE_VERSION
        || value.series_digest.is_empty()
        || value.epoch_registration_digest.is_empty()
        || value.context_delta_plan_digest.is_empty()
        || value.provider_id.is_empty()
        || value.request_attempt_count != 1
        || value.event_timestamp_ms == 0
        || value.exact_timestamp_ms.is_empty()
        || value.exact_timestamp_ms.len() != value.row_identity_digests.len()
        || value.normalized_dataset_digest.is_empty()
        || value.raw_response_digest.is_empty()
        || value.outcome_row_present
        || value.labels_accessed
        || value.metrics_computed
        || value.prior_private_evaluation_accessed
        || !value.credential_free
        || !value.read_only
        || !value.sanitized
        || value.capsule_digest != input_capsule_digest(value)
    {
        return Err("V4 series input capsule rejected".to_string());
    }
    Ok(())
}

fn encode_input_capsule(
    value: &MomentumProspectiveSeriesInputCapsuleV4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("series-input-capsule")
        .string("capsule_version", &value.capsule_version)
        .string("series_digest", &value.series_digest)
        .string(
            "epoch_registration_digest",
            &value.epoch_registration_digest,
        )
        .string(
            "context_delta_plan_digest",
            &value.context_delta_plan_digest,
        )
        .string("provider_id", &value.provider_id)
        .unsigned(
            "request_attempt_count",
            as_u64(value.request_attempt_count)?,
        )
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigneds("exact_timestamp_ms", &value.exact_timestamp_ms)
        .strings("row_identity_digests", &value.row_identity_digests)
        .string(
            "normalized_dataset_digest",
            &value.normalized_dataset_digest,
        )
        .string("raw_response_digest", &value.raw_response_digest)
        .boolean("outcome_row_present", value.outcome_row_present)
        .boolean("labels_accessed", value.labels_accessed)
        .boolean("metrics_computed", value.metrics_computed)
        .boolean(
            "prior_private_evaluation_accessed",
            value.prior_private_evaluation_accessed,
        )
        .boolean("credential_free", value.credential_free)
        .boolean("read_only", value.read_only)
        .boolean("sanitized", value.sanitized)
        .string("capsule_digest", &value.capsule_digest)
        .encode()
}

fn decode_input_capsule(bytes: &[u8]) -> Result<MomentumProspectiveSeriesInputCapsuleV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "series-input-capsule")?;
    let value = MomentumProspectiveSeriesInputCapsuleV4 {
        capsule_version: fields.string("capsule_version")?,
        series_digest: fields.string("series_digest")?,
        epoch_registration_digest: fields.string("epoch_registration_digest")?,
        context_delta_plan_digest: fields.string("context_delta_plan_digest")?,
        provider_id: fields.string("provider_id")?,
        request_attempt_count: as_usize(fields.unsigned("request_attempt_count")?)?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        exact_timestamp_ms: fields.unsigneds("exact_timestamp_ms")?,
        row_identity_digests: fields.strings("row_identity_digests")?,
        normalized_dataset_digest: fields.string("normalized_dataset_digest")?,
        raw_response_digest: fields.string("raw_response_digest")?,
        outcome_row_present: fields.boolean("outcome_row_present")?,
        labels_accessed: fields.boolean("labels_accessed")?,
        metrics_computed: fields.boolean("metrics_computed")?,
        prior_private_evaluation_accessed: fields.boolean("prior_private_evaluation_accessed")?,
        credential_free: fields.boolean("credential_free")?,
        read_only: fields.boolean("read_only")?,
        sanitized: fields.boolean("sanitized")?,
        capsule_digest: fields.string("capsule_digest")?,
    };
    fields.finish()?;
    validate_input_capsule(&value)?;
    Ok(value)
}

fn validate_prediction_seal(
    value: &MomentumSeriesParticipantPredictionSealV4,
) -> Result<(), String> {
    if value.seal_version != PREDICTION_SEAL_VERSION
        || value.series_digest.is_empty()
        || value.epoch_number < 2
        || value.epoch_registration_digest.is_empty()
        || value.participant_digest.is_empty()
        || value.participant_role.is_empty()
        || value.event_timestamp_ms == 0
        || value.input_receipt_digest.is_empty()
        || value.input_capsule_digest.is_empty()
        || value.context_use_proof_digest.is_empty()
        || value.context_assembly_proof_digest.is_empty()
        || value.feature_identity_digest.is_empty()
        || value.prediction_digest.is_empty()
        || !value.participant_identity_verified
        || value.parameter_updates != 0
        || value.normalizer_refits != 0
        || value.prior_score_reads != 0
        || value.outcome_access_count != 0
        || value.seal_digest != prediction_seal_digest(value)
    {
        return Err("V4 series participant prediction seal rejected".to_string());
    }
    Ok(())
}

fn encode_prediction_seal(
    value: &MomentumSeriesParticipantPredictionSealV4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("series-prediction-seal")
        .string("seal_version", &value.seal_version)
        .string("series_digest", &value.series_digest)
        .unsigned("epoch_number", value.epoch_number)
        .string(
            "epoch_registration_digest",
            &value.epoch_registration_digest,
        )
        .string("participant_digest", &value.participant_digest)
        .string("participant_role", &value.participant_role)
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .string("input_receipt_digest", &value.input_receipt_digest)
        .string("input_capsule_digest", &value.input_capsule_digest)
        .string("context_use_proof_digest", &value.context_use_proof_digest)
        .string(
            "context_assembly_proof_digest",
            &value.context_assembly_proof_digest,
        )
        .string("feature_identity_digest", &value.feature_identity_digest)
        .unsigned(
            "prediction_probability_bits",
            u64::from(value.prediction_probability_bits),
        )
        .string("prediction_digest", &value.prediction_digest)
        .boolean(
            "participant_identity_verified",
            value.participant_identity_verified,
        )
        .unsigned("parameter_updates", as_u64(value.parameter_updates)?)
        .unsigned("normalizer_refits", as_u64(value.normalizer_refits)?)
        .unsigned("prior_score_reads", as_u64(value.prior_score_reads)?)
        .unsigned("outcome_access_count", as_u64(value.outcome_access_count)?)
        .string("seal_digest", &value.seal_digest)
        .encode()
}

fn decode_prediction_seal(
    bytes: &[u8],
) -> Result<MomentumSeriesParticipantPredictionSealV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "series-prediction-seal")?;
    let value = MomentumSeriesParticipantPredictionSealV4 {
        seal_version: fields.string("seal_version")?,
        series_digest: fields.string("series_digest")?,
        epoch_number: fields.unsigned("epoch_number")?,
        epoch_registration_digest: fields.string("epoch_registration_digest")?,
        participant_digest: fields.string("participant_digest")?,
        participant_role: fields.string("participant_role")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        input_receipt_digest: fields.string("input_receipt_digest")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        context_use_proof_digest: fields.string("context_use_proof_digest")?,
        context_assembly_proof_digest: fields.string("context_assembly_proof_digest")?,
        feature_identity_digest: fields.string("feature_identity_digest")?,
        prediction_probability_bits: u32::try_from(fields.unsigned("prediction_probability_bits")?)
            .map_err(|_| "V4 series prediction bits rejected".to_string())?,
        prediction_digest: fields.string("prediction_digest")?,
        participant_identity_verified: fields.boolean("participant_identity_verified")?,
        parameter_updates: as_usize(fields.unsigned("parameter_updates")?)?,
        normalizer_refits: as_usize(fields.unsigned("normalizer_refits")?)?,
        prior_score_reads: as_usize(fields.unsigned("prior_score_reads")?)?,
        outcome_access_count: as_usize(fields.unsigned("outcome_access_count")?)?,
        seal_digest: fields.string("seal_digest")?,
    };
    fields.finish()?;
    validate_prediction_seal(&value)?;
    Ok(value)
}

fn validate_prediction_capsule(
    value: &MomentumProspectiveSeriesPredictionCapsuleV4,
) -> Result<(), String> {
    if value.capsule_version != PREDICTION_CAPSULE_VERSION
        || value.series_digest.is_empty()
        || value.epoch_registration_digest.is_empty()
        || value.event_timestamp_ms == 0
        || value.input_receipt_digest.is_empty()
        || value.input_capsule_digest.is_empty()
        || value.context_assembly_proof_digest.is_empty()
        || value.participant_seal_digests.len() != 3
        || value.participant_prediction_digests.len() != 3
        || value
            .participant_seal_digests
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != 3
        || !value.probabilities_hidden
        || !value.labels_hidden
        || value.prior_scores_accessed
        || value.outcome_accessed
        || value.metrics_computed
        || value.winner_selected
        || value.ranking_created
        || value.reward_applied
        || value.penalty_applied
        || value.chair_action_taken
        || value.capsule_digest != prediction_capsule_digest(value)
    {
        return Err("V4 series prediction capsule rejected".to_string());
    }
    Ok(())
}

fn encode_prediction_capsule(
    value: &MomentumProspectiveSeriesPredictionCapsuleV4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("series-prediction-capsule")
        .string("capsule_version", &value.capsule_version)
        .string("series_digest", &value.series_digest)
        .string(
            "epoch_registration_digest",
            &value.epoch_registration_digest,
        )
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .string("input_receipt_digest", &value.input_receipt_digest)
        .string("input_capsule_digest", &value.input_capsule_digest)
        .string(
            "context_assembly_proof_digest",
            &value.context_assembly_proof_digest,
        )
        .strings("participant_seal_digests", &value.participant_seal_digests)
        .strings(
            "participant_prediction_digests",
            &value.participant_prediction_digests,
        )
        .boolean("probabilities_hidden", value.probabilities_hidden)
        .boolean("labels_hidden", value.labels_hidden)
        .boolean("prior_scores_accessed", value.prior_scores_accessed)
        .boolean("outcome_accessed", value.outcome_accessed)
        .boolean("metrics_computed", value.metrics_computed)
        .boolean("winner_selected", value.winner_selected)
        .boolean("ranking_created", value.ranking_created)
        .boolean("reward_applied", value.reward_applied)
        .boolean("penalty_applied", value.penalty_applied)
        .boolean("chair_action_taken", value.chair_action_taken)
        .string("capsule_digest", &value.capsule_digest)
        .encode()
}

fn decode_prediction_capsule(
    bytes: &[u8],
) -> Result<MomentumProspectiveSeriesPredictionCapsuleV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "series-prediction-capsule")?;
    let value = MomentumProspectiveSeriesPredictionCapsuleV4 {
        capsule_version: fields.string("capsule_version")?,
        series_digest: fields.string("series_digest")?,
        epoch_registration_digest: fields.string("epoch_registration_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        input_receipt_digest: fields.string("input_receipt_digest")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        context_assembly_proof_digest: fields.string("context_assembly_proof_digest")?,
        participant_seal_digests: fields.strings("participant_seal_digests")?,
        participant_prediction_digests: fields.strings("participant_prediction_digests")?,
        probabilities_hidden: fields.boolean("probabilities_hidden")?,
        labels_hidden: fields.boolean("labels_hidden")?,
        prior_scores_accessed: fields.boolean("prior_scores_accessed")?,
        outcome_accessed: fields.boolean("outcome_accessed")?,
        metrics_computed: fields.boolean("metrics_computed")?,
        winner_selected: fields.boolean("winner_selected")?,
        ranking_created: fields.boolean("ranking_created")?,
        reward_applied: fields.boolean("reward_applied")?,
        penalty_applied: fields.boolean("penalty_applied")?,
        chair_action_taken: fields.boolean("chair_action_taken")?,
        capsule_digest: fields.string("capsule_digest")?,
    };
    fields.finish()?;
    validate_prediction_capsule(&value)?;
    Ok(value)
}

fn validate_journal(value: &MomentumProspectiveSeriesJournalEntryV4) -> Result<(), String> {
    if value.journal_version != JOURNAL_VERSION
        || value.series_digest.is_empty()
        || value.epoch_number < 2
        || value.event_one_adoption_digest.is_empty()
        || value.previous_epoch_ledger_entry_digest.is_empty()
        || value.context_delta_plan_digest.is_empty()
        || value.event_timestamp_ms == 0
        || value.registration_created_at_ms >= value.input_finality_boundary_ms
        || value.input_receipt_digest.is_empty()
        || value.input_capsule_digest.is_empty()
        || value.context_assembly_proof_digest.is_empty()
        || value.prediction_capsule_digest.is_empty()
        || value.participant_seal_digests.len() != 3
        || value
            .participant_seal_digests
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != 3
        || value.participant_prediction_digests.len() != 3
        || !value.deterministic_fixed_cadence_selection
        || value.prior_event_scores_read
        || value.prior_event_correctness_read
        || !value.registration_preceded_input_finality
        || !value.input_acquisition_preceded_prediction
        || !value.prediction_preceded_outcome_access
        || !value.outcome_stage_locked
        || value.winner_selected
        || value.ranking_created
        || value.reward_applied
        || value.penalty_applied
        || value.chair_action_taken
        || value.trading_action_taken
        || value.entry_digest != journal_entry_digest(value)
    {
        return Err("V4 prospective series journal rejected".to_string());
    }
    Ok(())
}

fn encode_journal(value: &MomentumProspectiveSeriesJournalEntryV4) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("series-journal-entry")
        .string("journal_version", &value.journal_version)
        .string("series_digest", &value.series_digest)
        .unsigned("epoch_number", value.epoch_number)
        .string(
            "event_one_adoption_digest",
            &value.event_one_adoption_digest,
        )
        .string(
            "previous_epoch_ledger_entry_digest",
            &value.previous_epoch_ledger_entry_digest,
        )
        .string(
            "context_delta_plan_digest",
            &value.context_delta_plan_digest,
        )
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigned(
            "registration_created_at_ms",
            value.registration_created_at_ms,
        )
        .unsigned(
            "input_finality_boundary_ms",
            value.input_finality_boundary_ms,
        )
        .string("input_receipt_digest", &value.input_receipt_digest)
        .string("input_capsule_digest", &value.input_capsule_digest)
        .string(
            "context_assembly_proof_digest",
            &value.context_assembly_proof_digest,
        )
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .strings("participant_seal_digests", &value.participant_seal_digests)
        .strings(
            "participant_prediction_digests",
            &value.participant_prediction_digests,
        )
        .boolean(
            "deterministic_fixed_cadence_selection",
            value.deterministic_fixed_cadence_selection,
        )
        .boolean("prior_event_scores_read", value.prior_event_scores_read)
        .boolean(
            "prior_event_correctness_read",
            value.prior_event_correctness_read,
        )
        .boolean(
            "registration_preceded_input_finality",
            value.registration_preceded_input_finality,
        )
        .boolean(
            "input_acquisition_preceded_prediction",
            value.input_acquisition_preceded_prediction,
        )
        .boolean(
            "prediction_preceded_outcome_access",
            value.prediction_preceded_outcome_access,
        )
        .boolean("outcome_stage_locked", value.outcome_stage_locked)
        .boolean("winner_selected", value.winner_selected)
        .boolean("ranking_created", value.ranking_created)
        .boolean("reward_applied", value.reward_applied)
        .boolean("penalty_applied", value.penalty_applied)
        .boolean("chair_action_taken", value.chair_action_taken)
        .boolean("trading_action_taken", value.trading_action_taken)
        .string("entry_digest", &value.entry_digest)
        .encode()
}

fn decode_journal(bytes: &[u8]) -> Result<MomentumProspectiveSeriesJournalEntryV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "series-journal-entry")?;
    let value = MomentumProspectiveSeriesJournalEntryV4 {
        journal_version: fields.string("journal_version")?,
        series_digest: fields.string("series_digest")?,
        epoch_number: fields.unsigned("epoch_number")?,
        event_one_adoption_digest: fields.string("event_one_adoption_digest")?,
        previous_epoch_ledger_entry_digest: fields.string("previous_epoch_ledger_entry_digest")?,
        context_delta_plan_digest: fields.string("context_delta_plan_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        registration_created_at_ms: fields.unsigned("registration_created_at_ms")?,
        input_finality_boundary_ms: fields.unsigned("input_finality_boundary_ms")?,
        input_receipt_digest: fields.string("input_receipt_digest")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        context_assembly_proof_digest: fields.string("context_assembly_proof_digest")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        participant_seal_digests: fields.strings("participant_seal_digests")?,
        participant_prediction_digests: fields.strings("participant_prediction_digests")?,
        deterministic_fixed_cadence_selection: fields
            .boolean("deterministic_fixed_cadence_selection")?,
        prior_event_scores_read: fields.boolean("prior_event_scores_read")?,
        prior_event_correctness_read: fields.boolean("prior_event_correctness_read")?,
        registration_preceded_input_finality: fields
            .boolean("registration_preceded_input_finality")?,
        input_acquisition_preceded_prediction: fields
            .boolean("input_acquisition_preceded_prediction")?,
        prediction_preceded_outcome_access: fields.boolean("prediction_preceded_outcome_access")?,
        outcome_stage_locked: fields.boolean("outcome_stage_locked")?,
        winner_selected: fields.boolean("winner_selected")?,
        ranking_created: fields.boolean("ranking_created")?,
        reward_applied: fields.boolean("reward_applied")?,
        penalty_applied: fields.boolean("penalty_applied")?,
        chair_action_taken: fields.boolean("chair_action_taken")?,
        trading_action_taken: fields.boolean("trading_action_taken")?,
        entry_digest: fields.string("entry_digest")?,
    };
    fields.finish()?;
    validate_journal(&value)?;
    Ok(value)
}

fn validate_outcome_plan(value: &MomentumProspectiveSeriesOutcomePlanV4) -> Result<(), String> {
    if value.plan_version != OUTCOME_PLAN_VERSION
        || value.series_digest.is_empty()
        || value.epoch_registration_digest.is_empty()
        || value.prediction_capsule_digest.is_empty()
        || value.event_timestamp_ms == 0
        || value.prediction_horizon != 1
        || value.required_outcome_timestamp_ms
            != [value
                .event_timestamp_ms
                .checked_add(DAILY_CADENCE_MS)
                .unwrap_or_default()]
        || value.outcome_finality_boundary_ms
            != value.required_outcome_timestamp_ms[0]
                .checked_add(DAILY_CADENCE_MS)
                .unwrap_or_default()
        || value.maximum_outcome_requests != 1
        || value.maximum_outcome_retries != 0
        || value.outcome_acquisition_count != 0
        || value.outcome_opening_count != 0
        || !value.labels_hidden_until_opening
        || !value.one_time_opening_required
        || !value.outcome_stage_locked_before_finality
        || value.plan_digest != outcome_plan_digest(value)
    {
        return Err("V4 prospective series outcome plan rejected".to_string());
    }
    Ok(())
}

fn encode_outcome_plan(value: &MomentumProspectiveSeriesOutcomePlanV4) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("series-outcome-plan")
        .string("plan_version", &value.plan_version)
        .string("series_digest", &value.series_digest)
        .string(
            "epoch_registration_digest",
            &value.epoch_registration_digest,
        )
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigned("prediction_horizon", as_u64(value.prediction_horizon)?)
        .unsigneds(
            "required_outcome_timestamp_ms",
            &value.required_outcome_timestamp_ms,
        )
        .unsigned(
            "outcome_finality_boundary_ms",
            value.outcome_finality_boundary_ms,
        )
        .unsigned(
            "maximum_outcome_requests",
            as_u64(value.maximum_outcome_requests)?,
        )
        .unsigned(
            "maximum_outcome_retries",
            as_u64(value.maximum_outcome_retries)?,
        )
        .unsigned(
            "outcome_acquisition_count",
            as_u64(value.outcome_acquisition_count)?,
        )
        .unsigned(
            "outcome_opening_count",
            as_u64(value.outcome_opening_count)?,
        )
        .boolean(
            "labels_hidden_until_opening",
            value.labels_hidden_until_opening,
        )
        .boolean("one_time_opening_required", value.one_time_opening_required)
        .boolean(
            "outcome_stage_locked_before_finality",
            value.outcome_stage_locked_before_finality,
        )
        .string("plan_digest", &value.plan_digest)
        .encode()
}

fn decode_outcome_plan(bytes: &[u8]) -> Result<MomentumProspectiveSeriesOutcomePlanV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "series-outcome-plan")?;
    let value = MomentumProspectiveSeriesOutcomePlanV4 {
        plan_version: fields.string("plan_version")?,
        series_digest: fields.string("series_digest")?,
        epoch_registration_digest: fields.string("epoch_registration_digest")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        prediction_horizon: as_usize(fields.unsigned("prediction_horizon")?)?,
        required_outcome_timestamp_ms: fields.unsigneds("required_outcome_timestamp_ms")?,
        outcome_finality_boundary_ms: fields.unsigned("outcome_finality_boundary_ms")?,
        maximum_outcome_requests: as_usize(fields.unsigned("maximum_outcome_requests")?)?,
        maximum_outcome_retries: as_usize(fields.unsigned("maximum_outcome_retries")?)?,
        outcome_acquisition_count: as_usize(fields.unsigned("outcome_acquisition_count")?)?,
        outcome_opening_count: as_usize(fields.unsigned("outcome_opening_count")?)?,
        labels_hidden_until_opening: fields.boolean("labels_hidden_until_opening")?,
        one_time_opening_required: fields.boolean("one_time_opening_required")?,
        outcome_stage_locked_before_finality: fields
            .boolean("outcome_stage_locked_before_finality")?,
        plan_digest: fields.string("plan_digest")?,
    };
    fields.finish()?;
    validate_outcome_plan(&value)?;
    Ok(value)
}

fn encode_safety_counters(
    value: &MomentumProspectiveSeriesSafetyCountersV4,
) -> Result<Vec<u8>, String> {
    let pairs = [
        ("network_request_attempts", value.network_request_attempts),
        ("retries", value.retries),
        ("maximum_concurrency", value.maximum_concurrency),
        ("transport_constructions", value.transport_constructions),
        ("canonical_raw_row_reads", value.canonical_raw_row_reads),
        (
            "prior_private_evaluation_reads",
            value.prior_private_evaluation_reads,
        ),
        (
            "participant_reconstructions",
            value.participant_reconstructions,
        ),
        ("feature_generations", value.feature_generations),
        ("prediction_computations", value.prediction_computations),
        ("parameter_updates", value.parameter_updates),
        ("normalizer_refits", value.normalizer_refits),
        ("training_uses", value.training_uses),
        ("qualification_uses", value.qualification_uses),
        ("outcome_requests", value.outcome_requests),
        ("outcome_openings", value.outcome_openings),
        ("metric_computations", value.metric_computations),
        ("winner_selections", value.winner_selections),
        ("ranking_creations", value.ranking_creations),
        ("reward_applications", value.reward_applications),
        ("penalty_applications", value.penalty_applications),
        ("chair_decisions", value.chair_decisions),
        ("votes", value.votes),
        ("voice_changes", value.voice_changes),
        ("tier_changes", value.tier_changes),
        ("cooldowns", value.cooldowns),
        ("promotions", value.promotions),
        ("quarantines", value.quarantines),
        ("paper_executions", value.paper_executions),
        ("live_executions", value.live_executions),
        ("active_committee_count", value.active_committee_count),
    ];
    let mut builder = ArtifactBuilderV4_2::new("series-safety-counters");
    for (name, count) in pairs {
        builder = builder.unsigned(name, as_u64(count)?);
    }
    builder.encode()
}

fn decode_safety_counters(
    bytes: &[u8],
) -> Result<MomentumProspectiveSeriesSafetyCountersV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "series-safety-counters")?;
    let value = MomentumProspectiveSeriesSafetyCountersV4 {
        network_request_attempts: as_usize(fields.unsigned("network_request_attempts")?)?,
        retries: as_usize(fields.unsigned("retries")?)?,
        maximum_concurrency: as_usize(fields.unsigned("maximum_concurrency")?)?,
        transport_constructions: as_usize(fields.unsigned("transport_constructions")?)?,
        canonical_raw_row_reads: as_usize(fields.unsigned("canonical_raw_row_reads")?)?,
        prior_private_evaluation_reads: as_usize(
            fields.unsigned("prior_private_evaluation_reads")?,
        )?,
        participant_reconstructions: as_usize(fields.unsigned("participant_reconstructions")?)?,
        feature_generations: as_usize(fields.unsigned("feature_generations")?)?,
        prediction_computations: as_usize(fields.unsigned("prediction_computations")?)?,
        parameter_updates: as_usize(fields.unsigned("parameter_updates")?)?,
        normalizer_refits: as_usize(fields.unsigned("normalizer_refits")?)?,
        training_uses: as_usize(fields.unsigned("training_uses")?)?,
        qualification_uses: as_usize(fields.unsigned("qualification_uses")?)?,
        outcome_requests: as_usize(fields.unsigned("outcome_requests")?)?,
        outcome_openings: as_usize(fields.unsigned("outcome_openings")?)?,
        metric_computations: as_usize(fields.unsigned("metric_computations")?)?,
        winner_selections: as_usize(fields.unsigned("winner_selections")?)?,
        ranking_creations: as_usize(fields.unsigned("ranking_creations")?)?,
        reward_applications: as_usize(fields.unsigned("reward_applications")?)?,
        penalty_applications: as_usize(fields.unsigned("penalty_applications")?)?,
        chair_decisions: as_usize(fields.unsigned("chair_decisions")?)?,
        votes: as_usize(fields.unsigned("votes")?)?,
        voice_changes: as_usize(fields.unsigned("voice_changes")?)?,
        tier_changes: as_usize(fields.unsigned("tier_changes")?)?,
        cooldowns: as_usize(fields.unsigned("cooldowns")?)?,
        promotions: as_usize(fields.unsigned("promotions")?)?,
        quarantines: as_usize(fields.unsigned("quarantines")?)?,
        paper_executions: as_usize(fields.unsigned("paper_executions")?)?,
        live_executions: as_usize(fields.unsigned("live_executions")?)?,
        active_committee_count: as_usize(fields.unsigned("active_committee_count")?)?,
    };
    fields.finish()?;
    validate_safety_counters(&value)?;
    Ok(value)
}

fn validate_safety_counters(
    value: &MomentumProspectiveSeriesSafetyCountersV4,
) -> Result<(), String> {
    if value.network_request_attempts > 1
        || value.retries != 0
        || value.maximum_concurrency != 1
        || value.transport_constructions > 1
        || value.prior_private_evaluation_reads != 0
        || value.parameter_updates != 0
        || value.normalizer_refits != 0
        || value.training_uses != 0
        || value.qualification_uses != 0
        || value.outcome_requests != 0
        || value.outcome_openings != 0
        || value.metric_computations != 0
        || value.winner_selections != 0
        || value.ranking_creations != 0
        || value.reward_applications != 0
        || value.penalty_applications != 0
        || value.chair_decisions != 0
        || value.votes != 0
        || value.voice_changes != 0
        || value.tier_changes != 0
        || value.cooldowns != 0
        || value.promotions != 0
        || value.quarantines != 0
        || value.paper_executions != 0
        || value.live_executions != 0
        || value.active_committee_count != 3
    {
        return Err("V4 series safety counters rejected".to_string());
    }
    Ok(())
}

fn validate_status(value: &MomentumProspectiveEpochStatusReceiptV4) -> Result<(), String> {
    let prediction_sealed =
        value.readiness == MomentumProspectiveEpochReadinessV4::PredictionAlreadySealed;
    let complete_prediction_bindings = value.input_receipt_digest.is_some()
        && value.input_capsule_digest.is_some()
        && value.context_assembly_proof_digest.is_some()
        && value.participant_prediction_digests.len() == 3
        && value.prediction_capsule_digest.is_some()
        && value.journal_entry_digest.is_some()
        && value.outcome_plan_digest.is_some();
    if value.status_version != STATUS_VERSION
        || value.series_digest.is_empty()
        || value.event_one_adoption_digest.is_empty()
        || value.candidate_gap_audit_digest.is_empty()
        || value.context_delta_plan_digest.is_empty()
        || value.epoch_registration_digest.is_empty()
        || value.epoch_number < 2
        || value.event_timestamp_ms == 0
        || value.input_finality_boundary_ms <= value.event_timestamp_ms
        || value.outcome_timestamp_ms != value.input_finality_boundary_ms
        || value.outcome_finality_boundary_ms <= value.outcome_timestamp_ms
        || value.exact_context_timestamp_ms.is_empty()
        || value.total_event_count == 0
        || value.scorable_event_count > value.total_event_count
        || prediction_sealed != complete_prediction_bindings
        || value.input_capsule_digest.is_some() && value.input_receipt_digest.is_none()
        || !prediction_sealed
            && (!value.participant_prediction_digests.is_empty()
                || value.context_assembly_proof_digest.is_some()
                || value.prediction_capsule_digest.is_some()
                || value.journal_entry_digest.is_some()
                || value.outcome_plan_digest.is_some())
        || !value.protected_artifacts_unchanged
        || !value.active_state_unchanged
        || validate_safety_counters(&value.safety_counters).is_err()
        || value.status_digest != status_digest(value)
    {
        return Err("V4 prospective epoch status rejected".to_string());
    }
    Ok(())
}

pub(super) fn validate_sealed_epoch_two_report_v4(
    report: &MomentumProspectiveSeriesReportV4,
) -> Result<(), String> {
    validate_status(&report.status)?;
    validate_series(&report.series)?;
    validate_adoption(&report.event_one_adoption)?;
    validate_gap_audit(&report.candidate_gap_audit)?;
    validate_delta_plan(&report.context_delta_plan)?;
    validate_epoch_registration(&report.epoch_registration)?;
    let (
        Some(input_receipt),
        Some(input_capsule),
        Some(context_use_proof),
        Some(context_assembly_proof),
        Some(prediction_capsule),
        Some(journal_entry),
        Some(outcome_plan),
    ) = (
        report.input_receipt.as_ref(),
        report.input_capsule.as_ref(),
        report.context_use_proof.as_ref(),
        report.context_assembly_proof.as_ref(),
        report.prediction_capsule.as_ref(),
        report.journal_entry.as_ref(),
        report.outcome_plan.as_ref(),
    )
    else {
        return Err("V4 sealed epoch-two report lineage rejected".to_string());
    };
    validate_input_receipt(input_receipt)?;
    validate_input_capsule(input_capsule)?;
    validate_context_use_proof(context_use_proof)?;
    validate_context_assembly(context_assembly_proof)?;
    validate_prediction_capsule(prediction_capsule)?;
    validate_journal(journal_entry)?;
    validate_outcome_plan(outcome_plan)?;

    let status = &report.status;
    let series_digest = &report.series.series_digest;
    let registration_digest = &report.epoch_registration.registration_digest;
    if status.readiness != MomentumProspectiveEpochReadinessV4::PredictionAlreadySealed
        || status.epoch_number != 2
        || report.epoch_registration.epoch_number != 2
        || status.series_digest != *series_digest
        || status.event_one_adoption_digest != report.event_one_adoption.adoption_digest
        || status.candidate_gap_audit_digest != report.candidate_gap_audit.audit_digest
        || status.context_delta_plan_digest != report.context_delta_plan.plan_digest
        || status.epoch_registration_digest != *registration_digest
        || status.input_receipt_digest.as_deref() != Some(input_receipt.receipt_digest.as_str())
        || status.input_capsule_digest.as_deref() != Some(input_capsule.capsule_digest.as_str())
        || status.context_assembly_proof_digest.as_deref()
            != Some(context_assembly_proof.proof_digest.as_str())
        || status.prediction_capsule_digest.as_deref()
            != Some(prediction_capsule.capsule_digest.as_str())
        || status.journal_entry_digest.as_deref() != Some(journal_entry.entry_digest.as_str())
        || status.outcome_plan_digest.as_deref() != Some(outcome_plan.plan_digest.as_str())
        || status.participant_prediction_digests
            != prediction_capsule.participant_prediction_digests
        || status.total_event_count != report.event_one_adoption.total_event_count
        || status.scorable_event_count != report.event_one_adoption.scorable_event_count
        || status.exact_context_timestamp_ms != report.epoch_registration.exact_context_timestamp_ms
        || status.exact_missing_timestamp_ms != report.epoch_registration.exact_missing_timestamp_ms
        || report.event_one_adoption.series_digest != *series_digest
        || report.candidate_gap_audit.series_digest != *series_digest
        || report.context_delta_plan.series_digest != *series_digest
        || report.context_delta_plan.epoch_number != 2
        || report.epoch_registration.series_digest != *series_digest
        || report.epoch_registration.context_delta_plan_digest
            != report.context_delta_plan.plan_digest
        || input_receipt.series_digest != *series_digest
        || input_receipt.epoch_registration_digest != *registration_digest
        || input_receipt.status != MomentumProspectiveSeriesInputStatusV4::EvidenceAcquired
        || input_receipt.input_capsule_digest.as_deref()
            != Some(input_capsule.capsule_digest.as_str())
        || input_capsule.series_digest != *series_digest
        || input_capsule.epoch_registration_digest != *registration_digest
        || input_capsule.context_delta_plan_digest != report.context_delta_plan.plan_digest
        || context_use_proof.series_digest != *series_digest
        || context_use_proof.epoch_registration_digest != *registration_digest
        || context_assembly_proof.series_digest != *series_digest
        || context_assembly_proof.epoch_registration_digest != *registration_digest
        || context_assembly_proof.input_capsule_digest != input_capsule.capsule_digest
        || context_assembly_proof.context_use_proof_digest != context_use_proof.proof_digest
        || prediction_capsule.series_digest != *series_digest
        || prediction_capsule.epoch_registration_digest != *registration_digest
        || prediction_capsule.input_receipt_digest != input_receipt.receipt_digest
        || prediction_capsule.input_capsule_digest != input_capsule.capsule_digest
        || prediction_capsule.context_assembly_proof_digest != context_assembly_proof.proof_digest
        || journal_entry.series_digest != *series_digest
        || journal_entry.epoch_number != 2
        || journal_entry.event_one_adoption_digest != report.event_one_adoption.adoption_digest
        || journal_entry.context_delta_plan_digest != report.context_delta_plan.plan_digest
        || journal_entry.prediction_capsule_digest != prediction_capsule.capsule_digest
        || journal_entry.participant_prediction_digests
            != prediction_capsule.participant_prediction_digests
        || outcome_plan.series_digest != *series_digest
        || outcome_plan.epoch_registration_digest != *registration_digest
        || outcome_plan.prediction_capsule_digest != prediction_capsule.capsule_digest
        || outcome_plan.required_outcome_timestamp_ms
            != [report.epoch_registration.outcome_timestamp_ms]
        || outcome_plan.outcome_finality_boundary_ms
            != report.epoch_registration.outcome_finality_boundary_ms
    {
        return Err("V4 sealed epoch-two report lineage rejected".to_string());
    }
    Ok(())
}

fn encode_status(value: &MomentumProspectiveEpochStatusReceiptV4) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("series-epoch-status")
        .string("status_version", &value.status_version)
        .string("series_digest", &value.series_digest)
        .string(
            "event_one_adoption_digest",
            &value.event_one_adoption_digest,
        )
        .string(
            "candidate_gap_audit_digest",
            &value.candidate_gap_audit_digest,
        )
        .string(
            "context_delta_plan_digest",
            &value.context_delta_plan_digest,
        )
        .string(
            "epoch_registration_digest",
            &value.epoch_registration_digest,
        )
        .unsigned("epoch_number", value.epoch_number)
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigned(
            "input_finality_boundary_ms",
            value.input_finality_boundary_ms,
        )
        .unsigned("outcome_timestamp_ms", value.outcome_timestamp_ms)
        .unsigned(
            "outcome_finality_boundary_ms",
            value.outcome_finality_boundary_ms,
        )
        .unsigneds(
            "exact_context_timestamp_ms",
            &value.exact_context_timestamp_ms,
        )
        .unsigneds(
            "exact_missing_timestamp_ms",
            &value.exact_missing_timestamp_ms,
        )
        .string("readiness", format!("{:?}", value.readiness))
        .optional_string("input_receipt_digest", &value.input_receipt_digest)
        .optional_string("input_capsule_digest", &value.input_capsule_digest)
        .optional_string(
            "context_assembly_proof_digest",
            &value.context_assembly_proof_digest,
        )
        .strings(
            "participant_prediction_digests",
            &value.participant_prediction_digests,
        )
        .optional_string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .optional_string("journal_entry_digest", &value.journal_entry_digest)
        .optional_string("outcome_plan_digest", &value.outcome_plan_digest)
        .unsigned("total_event_count", as_u64(value.total_event_count)?)
        .unsigned("scorable_event_count", as_u64(value.scorable_event_count)?)
        .string(
            "reward_eligibility_status",
            format!("{:?}", value.reward_eligibility_status),
        )
        .boolean(
            "protected_artifacts_unchanged",
            value.protected_artifacts_unchanged,
        )
        .boolean("active_state_unchanged", value.active_state_unchanged)
        .messages(
            "safety_counters",
            vec![encode_safety_counters(&value.safety_counters)?],
        )
        .string("status_digest", &value.status_digest)
        .encode()
}

fn decode_status(bytes: &[u8]) -> Result<MomentumProspectiveEpochStatusReceiptV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "series-epoch-status")?;
    let counters = fields.messages("safety_counters")?;
    if counters.len() != 1 {
        return Err("V4 series status safety counters rejected".to_string());
    }
    let value = MomentumProspectiveEpochStatusReceiptV4 {
        status_version: fields.string("status_version")?,
        series_digest: fields.string("series_digest")?,
        event_one_adoption_digest: fields.string("event_one_adoption_digest")?,
        candidate_gap_audit_digest: fields.string("candidate_gap_audit_digest")?,
        context_delta_plan_digest: fields.string("context_delta_plan_digest")?,
        epoch_registration_digest: fields.string("epoch_registration_digest")?,
        epoch_number: fields.unsigned("epoch_number")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        input_finality_boundary_ms: fields.unsigned("input_finality_boundary_ms")?,
        outcome_timestamp_ms: fields.unsigned("outcome_timestamp_ms")?,
        outcome_finality_boundary_ms: fields.unsigned("outcome_finality_boundary_ms")?,
        exact_context_timestamp_ms: fields.unsigneds("exact_context_timestamp_ms")?,
        exact_missing_timestamp_ms: fields.unsigneds("exact_missing_timestamp_ms")?,
        readiness: parse_readiness(&fields.string("readiness")?)?,
        input_receipt_digest: fields.optional_string("input_receipt_digest")?,
        input_capsule_digest: fields.optional_string("input_capsule_digest")?,
        context_assembly_proof_digest: fields.optional_string("context_assembly_proof_digest")?,
        participant_prediction_digests: fields.strings("participant_prediction_digests")?,
        prediction_capsule_digest: fields.optional_string("prediction_capsule_digest")?,
        journal_entry_digest: fields.optional_string("journal_entry_digest")?,
        outcome_plan_digest: fields.optional_string("outcome_plan_digest")?,
        total_event_count: as_usize(fields.unsigned("total_event_count")?)?,
        scorable_event_count: as_usize(fields.unsigned("scorable_event_count")?)?,
        reward_eligibility_status: parse_reward_status(
            &fields.string("reward_eligibility_status")?,
        )?,
        protected_artifacts_unchanged: fields.boolean("protected_artifacts_unchanged")?,
        active_state_unchanged: fields.boolean("active_state_unchanged")?,
        safety_counters: decode_safety_counters(&counters[0])?,
        status_digest: fields.string("status_digest")?,
    };
    fields.finish()?;
    validate_status(&value)?;
    Ok(value)
}

fn series_root(root: &Path) -> PathBuf {
    root.join(SERIES_ROOT).join(AGENT_ID)
}

fn collect_protected_artifacts(
    root: &Path,
    current: &Path,
    values: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), String> {
    if current == series_root(root) || current == root.join(SERIES_ROOT) {
        return Ok(());
    }
    if current.is_dir() {
        let mut paths = fs::read_dir(current)
            .map_err(|_| "V4 series protected directory read failed".to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            collect_protected_artifacts(root, &path, values)?;
        }
    } else if current.is_file() {
        values.push((
            current
                .strip_prefix(root)
                .map_err(|_| "V4 series protected path rejected".to_string())?
                .to_path_buf(),
            fs::read(current)
                .map_err(|_| "V4 series protected artifact read failed".to_string())?,
        ));
    }
    Ok(())
}

fn protected_artifacts(root: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>, String> {
    let mut values = Vec::new();
    collect_protected_artifacts(root, root, &mut values)?;
    values.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(values)
}

fn protected_aggregate_digest(values: &[(PathBuf, Vec<u8>)]) -> String {
    stable_hash_string(&format!("momentum-v4-series-protected:{values:?}"))
}

fn timestamp_range(
    end_timestamp_ms: u64,
    count: usize,
    cadence_ms: u64,
) -> Result<Vec<u64>, String> {
    let span = u64::try_from(count.saturating_sub(1))
        .ok()
        .and_then(|count| count.checked_mul(cadence_ms))
        .ok_or_else(|| "V4 series context span overflow".to_string())?;
    let start = end_timestamp_ms
        .checked_sub(span)
        .ok_or_else(|| "V4 series context start underflow".to_string())?;
    (0..count)
        .map(|index| {
            u64::try_from(index)
                .ok()
                .and_then(|index| index.checked_mul(cadence_ms))
                .and_then(|offset| start.checked_add(offset))
                .ok_or_else(|| "V4 series timestamp range overflow".to_string())
        })
        .collect()
}

fn read_single_json(directory: &Path) -> Result<(String, Vec<u8>), String> {
    let mut paths = if directory.exists() {
        fs::read_dir(directory)
            .map_err(|_| "V4 series raw evidence directory read failed".to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "json")
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    paths.sort();
    if paths.len() != 1 {
        return Err("V4 series canonical raw evidence identity rejected".to_string());
    }
    let digest = paths[0]
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "V4 series raw evidence digest unavailable".to_string())?
        .to_string();
    let bytes =
        fs::read(&paths[0]).map_err(|_| "V4 series raw evidence read failed".to_string())?;
    Ok((digest, bytes))
}

fn reopen_event_one(
    root: &Path,
    provider_config: &UpbitHistoricalPilotConfigV0,
    observed_timestamp_ms: u64,
) -> Result<EventOneStateV4, String> {
    let chain = reopen_momentum_v4_3_sealed_chain(root)?;
    let outcome = run_momentum_future_outcome_v4_4(
        root,
        provider_config,
        observed_timestamp_ms,
        MomentumOutcomeRunModeV4_4::Status,
        false,
        false,
    )?;
    let opening = run_momentum_future_outcome_opening_v4_4(
        root,
        provider_config,
        observed_timestamp_ms,
        MomentumOutcomeOpeningRunModeV4_4::Status,
        false,
        false,
    )?;
    let bundle = opening
        .opening_bundle
        .as_ref()
        .ok_or_else(|| "V4 series event-one opening bundle unavailable".to_string())?;
    let ledger = opening
        .evaluation_ledger
        .as_ref()
        .ok_or_else(|| "V4 series event-one ledger unavailable".to_string())?;
    let reward = opening
        .reward_eligibility
        .as_ref()
        .ok_or_else(|| "V4 series event-one eligibility unavailable".to_string())?;
    let receipt = outcome
        .receipt
        .as_ref()
        .ok_or_else(|| "V4 series event-one outcome receipt unavailable".to_string())?;
    let capsule = outcome
        .outcome_capsule
        .as_ref()
        .ok_or_else(|| "V4 series event-one outcome capsule unavailable".to_string())?;
    if outcome.status.outcome_readiness != MomentumOutcomeReadinessV4_4::OutcomeAlreadyOpened
        || receipt.request_attempt_count != 1
        || receipt.retry_count != 0
        || capsule.labels_opened
        || capsule.probabilities_opened
        || capsule.metrics_computed
        || capsule.winner_selected
        || capsule.reward_applied
        || capsule.penalty_applied
        || bundle.label_status != MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome
        || bundle.participant_evaluations.len() != 3
        || bundle
            .participant_evaluations
            .iter()
            .any(|evaluation| evaluation.status != MomentumProspectiveEvaluationStatusV4_4::Scored)
        || bundle.winner_selected
        || bundle.ranking_created
        || bundle.reward_applied
        || bundle.penalty_applied
        || bundle.chair_action_taken
        || ledger.entries.len() != 1
        || reward.event_count != ledger.entries.len()
        || reward.scorable_event_count != 1
        || reward.status != MomentumRewardEligibilityStatusV4_4::IneligibleMinimumSamples
        || reward.reward_application_count != 0
        || reward.penalty_application_count != 0
        || reward.voice_mutation_count != 0
        || reward.promotion_count != 0
        || chain.prediction_capsule.participant_prediction_seals.len() != 3
    {
        return Err("V4 series event-one immutable chain rejected".to_string());
    }
    Ok(EventOneStateV4 {
        chain,
        outcome,
        opening,
    })
}

fn derive_series(
    event: &EventOneStateV4,
    provider_config: &UpbitHistoricalPilotConfigV0,
    protected: &[(PathBuf, Vec<u8>)],
    active_state_digest: &str,
) -> Result<MomentumProspectiveSeriesV4, String> {
    provider_config.validate()?;
    let authorization = event
        .opening
        .authorization
        .as_ref()
        .ok_or_else(|| "V4 series event-one authorization unavailable".to_string())?;
    let bundle = event
        .opening
        .opening_bundle
        .as_ref()
        .ok_or_else(|| "V4 series event-one opening bundle unavailable".to_string())?;
    let ledger = event
        .opening
        .evaluation_ledger
        .as_ref()
        .ok_or_else(|| "V4 series event-one ledger unavailable".to_string())?;
    let reward = event
        .opening
        .reward_eligibility
        .as_ref()
        .ok_or_else(|| "V4 series event-one eligibility unavailable".to_string())?;
    let entry = ledger
        .entries
        .first()
        .ok_or_else(|| "V4 series event-one ledger entry unavailable".to_string())?;
    if authorization.feature_policy_digest != event.chain.lifecycle.feature_policy_digest
        || authorization.label_policy_digest.is_empty()
        || authorization.evaluation_policy_digest.is_empty()
        || event.chain.lifecycle.participant_digests.len() != 3
        || event.chain.lifecycle.participant_parameter_digests.len() != 3
        || event.chain.lifecycle.participant_normalizer_digests.len() != 3
    {
        return Err("V4 series frozen policy identity rejected".to_string());
    }
    let minimum_sample_policy_digest = stable_hash_string(&format!(
        "momentum-v4-minimum-sample-policy:{}:{}:{}:{:?}",
        reward.minimum_sample_gate,
        reward.learned_participant_count,
        reward.benchmark_participant_count,
        reward.participant_roles
    ));
    let registration = &event.outcome.registration;
    let mut value = MomentumProspectiveSeriesV4 {
        series_version: SERIES_VERSION.to_string(),
        agent_id: event.chain.lifecycle.agent_id.clone(),
        frozen_roster_digest: event.chain.lifecycle.roster_digest.clone(),
        participant_digests: event.chain.lifecycle.participant_digests.clone(),
        parameter_digests: event.chain.lifecycle.participant_parameter_digests.clone(),
        normalizer_digests: event.chain.lifecycle.participant_normalizer_digests.clone(),
        feature_policy_digest: authorization.feature_policy_digest.clone(),
        label_policy_digest: authorization.label_policy_digest.clone(),
        evaluation_policy_digest: authorization.evaluation_policy_digest.clone(),
        minimum_sample_policy_digest,
        provider_id: registration.provider_id.clone(),
        market: registration.market.clone(),
        symbol: registration.symbol.clone(),
        cadence_ms: event.chain.lifecycle.cadence_ms,
        context_row_count: event.chain.context_plan.required_row_count,
        prediction_horizon: event.chain.lifecycle.prediction_horizon,
        first_event_ledger_entry_digest: entry.entry_digest.clone(),
        first_event_opening_bundle_digest: bundle.bundle_digest.clone(),
        first_event_eligibility_digest: reward.receipt_digest.clone(),
        continuation_policy:
            MomentumProspectiveContinuationPolicyV4::FixedDailyCadenceNextLegalEvent,
        maximum_open_epochs: 1,
        manual_network_confirmation_required: true,
        automatic_network_execution_forbidden: true,
        retraining_forbidden: true,
        participant_selection_forbidden: true,
        result_conditioned_continuation_forbidden: true,
        winner_selection_forbidden: true,
        ranking_forbidden: true,
        reward_application_forbidden: true,
        penalty_application_forbidden: true,
        chair_action_forbidden: true,
        trading_forbidden: true,
        protected_before_artifact_count: protected.len(),
        protected_before_aggregate_digest: protected_aggregate_digest(protected),
        active_agent_state_digest: active_state_digest.to_string(),
        series_digest: String::new(),
    };
    value.series_digest = series_digest(&value);
    validate_series(&value)?;
    Ok(value)
}

fn derive_adoption(
    series: &MomentumProspectiveSeriesV4,
    event: &EventOneStateV4,
) -> Result<MomentumProspectiveSeriesAdoptionV4, String> {
    let outcome = event
        .outcome
        .outcome_capsule
        .as_ref()
        .ok_or_else(|| "V4 series event-one outcome unavailable".to_string())?;
    let bundle = event
        .opening
        .opening_bundle
        .as_ref()
        .ok_or_else(|| "V4 series event-one opening unavailable".to_string())?;
    let ledger = event
        .opening
        .evaluation_ledger
        .as_ref()
        .ok_or_else(|| "V4 series event-one ledger unavailable".to_string())?;
    let entry = ledger
        .entries
        .first()
        .ok_or_else(|| "V4 series event-one entry unavailable".to_string())?;
    let reward = event
        .opening
        .reward_eligibility
        .as_ref()
        .ok_or_else(|| "V4 series event-one eligibility unavailable".to_string())?;
    let mut value = MomentumProspectiveSeriesAdoptionV4 {
        adoption_version: ADOPTION_VERSION.to_string(),
        series_digest: series.series_digest.clone(),
        adopted_epoch_number: 1,
        adopted_event_timestamp_ms: entry.event_timestamp_ms,
        prediction_capsule_digest: entry.prediction_capsule_digest.clone(),
        outcome_capsule_digest: outcome.capsule_digest.clone(),
        opening_bundle_digest: bundle.bundle_digest.clone(),
        evaluation_ledger_entry_digest: entry.entry_digest.clone(),
        reward_eligibility_digest: reward.receipt_digest.clone(),
        total_event_count: entry.total_event_count_after,
        scorable_event_count: entry.scorable_event_count_after,
        winner_selected: entry.winner_selected,
        ranking_created: bundle.ranking_created,
        reward_applied: entry.reward_applied,
        penalty_applied: entry.penalty_applied,
        chair_action_taken: bundle.chair_action_taken,
        adoption_digest: String::new(),
    };
    value.adoption_digest = adoption_digest(&value);
    validate_adoption(&value)?;
    Ok(value)
}

fn derive_gap_audit(
    series: &MomentumProspectiveSeriesV4,
    adoption: &MomentumProspectiveSeriesAdoptionV4,
    outcome_timestamp_ms: u64,
    registration_observed_at_ms: u64,
) -> Result<MomentumProspectiveCandidateGapAuditV4, String> {
    let candidate = adoption
        .adopted_event_timestamp_ms
        .checked_add(series.cadence_ms)
        .ok_or_else(|| "V4 series adjacent candidate overflow".to_string())?;
    let input_finality = candidate
        .checked_add(series.cadence_ms)
        .ok_or_else(|| "V4 series adjacent finality overflow".to_string())?;
    let registration_after_input_finality = registration_observed_at_ms >= input_finality;
    let prior_outcome_already_opened = candidate == outcome_timestamp_ms;
    let mut reasons = Vec::new();
    if registration_after_input_finality {
        reasons.push("RegistrationAfterInputFinality".to_string());
    }
    if prior_outcome_already_opened {
        reasons.push("PriorOutcomeAlreadyOpened".to_string());
    }
    let canonical_disposition = if prior_outcome_already_opened {
        MomentumProspectiveCandidateDispositionV4::SkippedPriorOutcomeAlreadyOpened
    } else if registration_after_input_finality {
        MomentumProspectiveCandidateDispositionV4::SkippedRegistrationAfterInputFinality
    } else {
        MomentumProspectiveCandidateDispositionV4::Eligible
    };
    let mut value = MomentumProspectiveCandidateGapAuditV4 {
        audit_version: GAP_AUDIT_VERSION.to_string(),
        series_digest: series.series_digest.clone(),
        prior_event_timestamp_ms: adoption.adopted_event_timestamp_ms,
        adjacent_candidate_timestamp_ms: candidate,
        registration_observed_at_ms,
        candidate_input_finality_boundary_ms: input_finality,
        registration_after_input_finality,
        prior_outcome_already_opened,
        applicable_reasons: reasons,
        canonical_disposition,
        counted_as_model_failure: false,
        reward_or_penalty_consequence: false,
        audit_digest: String::new(),
    };
    value.audit_digest = gap_audit_digest(&value);
    validate_gap_audit(&value)?;
    Ok(value)
}

fn derive_next_legal_event(
    series: &MomentumProspectiveSeriesV4,
    adoption: &MomentumProspectiveSeriesAdoptionV4,
    prior_outcome_timestamp_ms: u64,
    registration_created_at_ms: u64,
) -> Result<u64, String> {
    let mut candidate = adoption
        .adopted_event_timestamp_ms
        .checked_add(series.cadence_ms)
        .ok_or_else(|| "V4 series next candidate overflow".to_string())?;
    loop {
        let finality = candidate
            .checked_add(series.cadence_ms)
            .ok_or_else(|| "V4 series next finality overflow".to_string())?;
        if candidate != prior_outcome_timestamp_ms && registration_created_at_ms < finality {
            return Ok(candidate);
        }
        candidate = candidate
            .checked_add(series.cadence_ms)
            .ok_or_else(|| "V4 series candidate advance overflow".to_string())?;
    }
}

fn load_canonical_rows(
    root: &Path,
    event: &EventOneStateV4,
) -> Result<BTreeMap<u64, CanonicalRowV4>, String> {
    let mut rows = BTreeMap::new();
    let input_root = root.join("v4_3").join(AGENT_ID);
    let (input_raw_digest, input_raw) = read_single_json(&input_root.join("raw_input"))?;
    if input_raw_digest != event.chain.input_capsule.raw_response_digest {
        return Err("V4 series prior input raw response identity rejected".to_string());
    }
    let input_text = std::str::from_utf8(&input_raw)
        .map_err(|_| "V4 series prior input UTF-8 rejected".to_string())?;
    let input_dataset = parse_upbit_daily_ohlcv_v0(input_text, &event.outcome.registration.symbol)?;
    let input_timestamps = input_dataset
        .rows
        .iter()
        .map(|row| row.timestamp_ms)
        .collect::<Vec<_>>();
    let input_digests = input_dataset
        .rows
        .iter()
        .map(row_identity_digest)
        .collect::<Vec<_>>();
    if input_timestamps != event.chain.input_capsule.exact_timestamp_ms
        || input_digests != event.chain.input_capsule.row_identity_digests
    {
        return Err("V4 series prior input canonical rows rejected".to_string());
    }
    for row in input_dataset.rows {
        let use_class = if row.timestamp_ms == event.chain.prediction_capsule.event_timestamp_ms {
            MomentumSeriesContextUseV4::PriorProspectiveEventRawContext
        } else {
            MomentumSeriesContextUseV4::ExistingCanonicalHistoricalRaw
        };
        let reference = MomentumSeriesCanonicalRowRefV4 {
            timestamp_ms: row.timestamp_ms,
            raw_row_digest: row_identity_digest(&row),
            source_capsule_digest: event.chain.input_capsule.capsule_digest.clone(),
            use_class,
        };
        if rows
            .insert(row.timestamp_ms, CanonicalRowV4 { row, reference })
            .is_some()
        {
            return Err("V4 series duplicate prior input row rejected".to_string());
        }
    }
    let outcome_root = root.join("v4_4").join(AGENT_ID);
    let (_, outcome_raw) = read_single_json(&outcome_root.join("raw_outcome"))?;
    let outcome_text = std::str::from_utf8(&outcome_raw)
        .map_err(|_| "V4 series prior outcome UTF-8 rejected".to_string())?;
    let outcome_dataset =
        parse_upbit_daily_ohlcv_v0(outcome_text, &event.outcome.registration.symbol)?;
    let outcome_capsule = event
        .outcome
        .outcome_capsule
        .as_ref()
        .ok_or_else(|| "V4 series prior outcome capsule unavailable".to_string())?;
    if outcome_dataset.rows.len() != 1
        || outcome_dataset.rows[0].timestamp_ms != outcome_capsule.outcome_timestamp_ms
        || row_identity_digest(&outcome_dataset.rows[0]) != outcome_capsule.outcome_row_digest
    {
        return Err("V4 series prior outcome canonical row rejected".to_string());
    }
    let row = outcome_dataset.rows[0].clone();
    let reference = MomentumSeriesCanonicalRowRefV4 {
        timestamp_ms: row.timestamp_ms,
        raw_row_digest: row_identity_digest(&row),
        source_capsule_digest: outcome_capsule.capsule_digest.clone(),
        use_class: MomentumSeriesContextUseV4::PriorOpenedOutcomeRawContext,
    };
    if rows
        .insert(row.timestamp_ms, CanonicalRowV4 { row, reference })
        .is_some()
    {
        return Err("V4 series canonical source overlap rejected".to_string());
    }
    Ok(rows)
}

fn missing_set_is_contiguous(values: &[u64], cadence_ms: u64) -> bool {
    !values.is_empty()
        && values
            .windows(2)
            .all(|pair| pair[1] == pair[0].saturating_add(cadence_ms))
}

fn derive_delta_plan(
    series: &MomentumProspectiveSeriesV4,
    epoch_number: u64,
    event_timestamp_ms: u64,
    canonical: &BTreeMap<u64, CanonicalRowV4>,
) -> Result<MomentumCanonicalContextDeltaPlanV4, String> {
    let exact_context = timestamp_range(
        event_timestamp_ms,
        series.context_row_count,
        series.cadence_ms,
    )?;
    let canonical_rows = exact_context
        .iter()
        .filter_map(|timestamp| canonical.get(timestamp).map(|row| row.reference.clone()))
        .collect::<Vec<_>>();
    let missing = exact_context
        .iter()
        .filter(|timestamp| !canonical.contains_key(timestamp))
        .copied()
        .collect::<Vec<_>>();
    let mut value = MomentumCanonicalContextDeltaPlanV4 {
        plan_version: DELTA_PLAN_VERSION.to_string(),
        series_digest: series.series_digest.clone(),
        epoch_number,
        event_timestamp_ms,
        exact_context_timestamp_ms: exact_context,
        canonical_rows,
        exact_missing_timestamp_ms: missing.clone(),
        maximum_requests: 1,
        maximum_retries: 0,
        maximum_concurrency: 1,
        full_context_refetch_forbidden: true,
        prior_private_evaluation_accessed: false,
        missing_set_contiguous: missing_set_is_contiguous(&missing, series.cadence_ms),
        plan_digest: String::new(),
    };
    value.plan_digest = delta_plan_digest(&value);
    validate_delta_plan(&value)?;
    Ok(value)
}

fn derive_epoch_registration(
    series: &MomentumProspectiveSeriesV4,
    adoption: &MomentumProspectiveSeriesAdoptionV4,
    delta: &MomentumCanonicalContextDeltaPlanV4,
    provider_config: &UpbitHistoricalPilotConfigV0,
    registration_created_at_ms: u64,
) -> Result<MomentumProspectiveEpochRegistrationV4, String> {
    let contract = upbit_learning_evidence_provider_contract_v1(provider_config)?;
    if contract.provider_id != series.provider_id
        || contract.market_scope != AcquisitionMarketScope::BtcCrypto
        || contract.dataset_kind != DatasetKind::DailyOhlcv
        || contract.symbols != [series.symbol.clone()]
        || contract.cadence != "1d"
        || !contract.credential_free
        || !contract.read_only
        || !contract.approved_for_network
        || !contract.all_rows_finalized
        || contract.maximum_lookback_bars < delta.exact_missing_timestamp_ms.len()
    {
        return Err("V4 series provider contract rejected".to_string());
    }
    let input_finality_boundary_ms = delta
        .event_timestamp_ms
        .checked_add(series.cadence_ms)
        .ok_or_else(|| "V4 series input finality overflow".to_string())?;
    let outcome_timestamp_ms = input_finality_boundary_ms;
    let outcome_finality_boundary_ms = outcome_timestamp_ms
        .checked_add(series.cadence_ms)
        .ok_or_else(|| "V4 series outcome finality overflow".to_string())?;
    let mut value = MomentumProspectiveEpochRegistrationV4 {
        registration_version: EPOCH_REGISTRATION_VERSION.to_string(),
        series_digest: series.series_digest.clone(),
        epoch_number: adoption.adopted_epoch_number + 1,
        previous_epoch_ledger_entry_digest: adoption.evaluation_ledger_entry_digest.clone(),
        previous_epoch_opening_bundle_digest: adoption.opening_bundle_digest.clone(),
        event_timestamp_ms: delta.event_timestamp_ms,
        registration_created_at_ms,
        input_finality_boundary_ms,
        outcome_timestamp_ms,
        outcome_finality_boundary_ms,
        exact_context_timestamp_ms: delta.exact_context_timestamp_ms.clone(),
        exact_missing_timestamp_ms: delta.exact_missing_timestamp_ms.clone(),
        context_delta_plan_digest: delta.plan_digest.clone(),
        provider_id: series.provider_id.clone(),
        market: series.market.clone(),
        symbol: series.symbol.clone(),
        cadence: contract.cadence,
        maximum_input_requests: 1,
        maximum_input_retries: 0,
        maximum_input_concurrency: 1,
        maximum_response_bytes: contract.maximum_response_bytes,
        prior_private_evaluation_access_forbidden: true,
        parameter_update_forbidden: true,
        normalizer_refit_forbidden: true,
        outcome_access_forbidden: true,
        winner_selection_forbidden: true,
        ranking_forbidden: true,
        reward_application_forbidden: true,
        penalty_application_forbidden: true,
        chair_action_forbidden: true,
        trading_forbidden: true,
        registration_digest: String::new(),
    };
    value.registration_digest = epoch_registration_digest(&value);
    validate_epoch_registration(&value)?;
    Ok(value)
}

fn persist_pb_if_absent<T: PartialEq>(
    root: &Path,
    directory: &str,
    digest: &str,
    value: &T,
    bytes: &[u8],
    decode: fn(&[u8]) -> Result<T, String>,
    stored_digest: fn(&T) -> &str,
) -> Result<(usize, usize), String> {
    let directory = root.join(directory);
    if let Some(existing) = read_single(&directory, decode)? {
        if &existing != value || stored_digest(&existing) != digest {
            return Err("V4 series persisted artifact identity mismatch".to_string());
        }
        return Ok((0, 0));
    }
    persist_artifact(
        &directory.join(format!("{digest}.pb")),
        bytes,
        digest,
        |stored| Ok(stored_digest(&decode(stored)?).to_string()),
    )
}

fn persist_status_if_absent(
    root: &Path,
    value: &MomentumProspectiveEpochStatusReceiptV4,
) -> Result<(usize, usize), String> {
    let path = root
        .join("epoch_status_receipts")
        .join(format!("{}.pb", value.status_digest));
    if path.exists() {
        let stored = decode_status(
            &fs::read(&path).map_err(|_| "V4 series status reread failed".to_string())?,
        )?;
        if stored != *value {
            return Err("V4 series status replay mismatch".to_string());
        }
        return Ok((0, 0));
    }
    persist_artifact(
        &path,
        &encode_status(value)?,
        &value.status_digest,
        |bytes| Ok(decode_status(bytes)?.status_digest),
    )
}

fn persist_raw_input(root: &Path, digest: &str, bytes: &[u8]) -> Result<(usize, usize), String> {
    let directory = root.join("raw_input");
    let path = directory.join(format!("{digest}.json"));
    if path.exists() {
        let stored =
            fs::read(&path).map_err(|_| "V4 series raw input reread failed".to_string())?;
        if stable_hash_string(&format!("momentum-v4-series-raw-input:{stored:?}")) != digest
            || stored != bytes
        {
            return Err("V4 series raw input replay mismatch".to_string());
        }
        return Ok((0, 0));
    }
    persist_artifact(&path, bytes, digest, |stored| {
        Ok(stable_hash_string(&format!(
            "momentum-v4-series-raw-input:{stored:?}"
        )))
    })
}

fn add_counts(total: &mut (usize, usize), next: (usize, usize)) {
    total.0 += next.0;
    total.1 += next.1;
}

fn persist_preregistration(
    root: &Path,
    series: &MomentumProspectiveSeriesV4,
    adoption: &MomentumProspectiveSeriesAdoptionV4,
    audit: &MomentumProspectiveCandidateGapAuditV4,
    delta: &MomentumCanonicalContextDeltaPlanV4,
    registration: &MomentumProspectiveEpochRegistrationV4,
    status: &MomentumProspectiveEpochStatusReceiptV4,
) -> Result<(usize, usize), String> {
    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_pb_if_absent(
            root,
            "series_contracts",
            &series.series_digest,
            series,
            &encode_series(series)?,
            decode_series,
            |value| &value.series_digest,
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb_if_absent(
            root,
            "event_adoptions",
            &adoption.adoption_digest,
            adoption,
            &encode_adoption(adoption)?,
            decode_adoption,
            |value| &value.adoption_digest,
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb_if_absent(
            root,
            "candidate_gap_audits",
            &audit.audit_digest,
            audit,
            &encode_gap_audit(audit)?,
            decode_gap_audit,
            |value| &value.audit_digest,
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb_if_absent(
            root,
            "context_delta_plans",
            &delta.plan_digest,
            delta,
            &encode_delta_plan(delta)?,
            decode_delta_plan,
            |value| &value.plan_digest,
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb_if_absent(
            root,
            "epoch_registrations",
            &registration.registration_digest,
            registration,
            &encode_epoch_registration(registration)?,
            decode_epoch_registration,
            |value| &value.registration_digest,
        )?,
    );
    add_counts(&mut counts, persist_status_if_absent(root, status)?);
    Ok(counts)
}

pub(crate) fn build_provider_request(
    registration: &MomentumProspectiveEpochRegistrationV4,
) -> Result<ReadOnlyProviderRequest, String> {
    let first = registration
        .exact_missing_timestamp_ms
        .first()
        .copied()
        .ok_or_else(|| "V4 series missing request start unavailable".to_string())?;
    let end = registration
        .exact_missing_timestamp_ms
        .last()
        .copied()
        .and_then(|timestamp| timestamp.checked_add(DAILY_CADENCE_MS))
        .ok_or_else(|| "V4 series missing request end overflow".to_string())?;
    let request_id = stable_hash_string(&format!(
        "momentum-v4-series-input-request:{}",
        registration.registration_digest
    ));
    Ok(ReadOnlyProviderRequest {
        request_id,
        request_key: stable_hash_string(&format!(
            "momentum-v4-series-input-key:{}:{:?}",
            registration.registration_digest, registration.exact_missing_timestamp_ms
        )),
        provider_id: registration.provider_id.clone(),
        dataset_kind: DatasetKind::DailyOhlcv,
        market_scope: AcquisitionMarketScope::BtcCrypto,
        symbols: vec![registration.symbol.clone()],
        lookback: DataLookback {
            bars: registration.exact_missing_timestamp_ms.len(),
            start_timestamp_ms: Some(first),
            end_timestamp_ms: Some(end),
        },
        cadence: registration.cadence.clone(),
        max_staleness_ms: 0,
        reason_codes: vec![],
    })
}

fn request_config(
    provider_config: &UpbitHistoricalPilotConfigV0,
    registration: &MomentumProspectiveEpochRegistrationV4,
) -> Result<UpbitHistoricalPilotConfigV0, String> {
    let mut value = provider_config.clone();
    value.start_timestamp_ms = registration.exact_missing_timestamp_ms[0];
    value.end_timestamp_ms = registration
        .exact_missing_timestamp_ms
        .last()
        .copied()
        .and_then(|timestamp| timestamp.checked_add(DAILY_CADENCE_MS))
        .ok_or_else(|| "V4 series request boundary overflow".to_string())?;
    value.max_retries = 0;
    value.validate()?;
    let contract = upbit_learning_evidence_provider_contract_v1(&value)?;
    if contract.provider_id != registration.provider_id
        || contract.market_scope != AcquisitionMarketScope::BtcCrypto
        || contract.dataset_kind != DatasetKind::DailyOhlcv
        || contract.symbols != [registration.symbol.clone()]
        || contract.cadence != registration.cadence
        || contract.maximum_response_bytes != registration.maximum_response_bytes
        || !contract.credential_free
        || !contract.read_only
        || !contract.approved_for_network
        || !contract.all_rows_finalized
        || !contract.enabled
    {
        return Err("V4 series input provider contract rejected".to_string());
    }
    Ok(value)
}

fn row_is_valid(row: &HistoricalOhlcvRow, symbol: &str) -> bool {
    row.symbol == symbol
        && [
            row.open,
            row.high,
            row.low,
            row.close,
            row.volume,
            row.trade_value.unwrap_or_default(),
        ]
        .iter()
        .all(|value| value.is_finite())
        && row.open > 0.0
        && row.high > 0.0
        && row.low > 0.0
        && row.close > 0.0
        && row.volume >= 0.0
        && row.trade_value.is_none_or(|value| value >= 0.0)
        && row.high >= row.open.max(row.close)
        && row.low <= row.open.min(row.close)
}

fn validate_input_response(
    registration: &MomentumProspectiveEpochRegistrationV4,
    request: &ReadOnlyProviderRequest,
    transport: &LearningEvidenceTransportResponseV1,
) -> Result<
    (
        MomentumProspectiveSeriesInputCapsuleV4,
        Vec<HistoricalOhlcvRow>,
    ),
    String,
> {
    if transport.http_status_class != "2xx"
        || transport.raw_response.is_empty()
        || transport.raw_response.len() > registration.maximum_response_bytes
        || serde_json::from_slice::<serde_json::Value>(&transport.raw_response).is_err()
        || transport.response.request_id != request.request_id
        || transport.response.provider_id != registration.provider_id
        || transport.response.content_type != "application/x-soma-normalized-dataset"
        || !transport.response.all_rows_finalized
        || transport.response.normalized_dataset.symbol != registration.symbol
        || transport.response.reported_content_bytes != transport.raw_response.len()
    {
        return Err("V4 series input response envelope rejected".to_string());
    }
    let raw_text = std::str::from_utf8(&transport.raw_response)
        .map_err(|_| "V4 series input response UTF-8 rejected".to_string())?;
    let raw_dataset = parse_upbit_daily_ohlcv_v0(raw_text, &registration.symbol)?;
    if raw_dataset.symbol != transport.response.normalized_dataset.symbol
        || raw_dataset.rows != transport.response.normalized_dataset.rows
    {
        return Err("V4 series raw and normalized response mismatch".to_string());
    }
    let rows = &transport.response.normalized_dataset.rows;
    let timestamps = rows.iter().map(|row| row.timestamp_ms).collect::<Vec<_>>();
    if timestamps != registration.exact_missing_timestamp_ms
        || rows.len() != registration.exact_missing_timestamp_ms.len()
        || timestamps.iter().copied().collect::<BTreeSet<_>>().len() != rows.len()
        || rows
            .windows(2)
            .any(|pair| pair[1].timestamp_ms != pair[0].timestamp_ms + DAILY_CADENCE_MS)
        || rows
            .iter()
            .any(|row| !row_is_valid(row, &registration.symbol))
        || timestamps.contains(&registration.outcome_timestamp_ms)
        || timestamps
            .iter()
            .any(|timestamp| *timestamp >= registration.input_finality_boundary_ms)
    {
        return Err("V4 series exact missing input evidence rejected".to_string());
    }
    let raw_response_digest = stable_hash_string(&format!(
        "momentum-v4-series-raw-input:{:?}",
        transport.raw_response
    ));
    let mut capsule = MomentumProspectiveSeriesInputCapsuleV4 {
        capsule_version: INPUT_CAPSULE_VERSION.to_string(),
        series_digest: registration.series_digest.clone(),
        epoch_registration_digest: registration.registration_digest.clone(),
        context_delta_plan_digest: registration.context_delta_plan_digest.clone(),
        provider_id: registration.provider_id.clone(),
        request_attempt_count: 1,
        event_timestamp_ms: registration.event_timestamp_ms,
        exact_timestamp_ms: timestamps,
        row_identity_digests: rows.iter().map(row_identity_digest).collect(),
        normalized_dataset_digest: historical_replay_dataset_digest_v0(
            &transport.response.normalized_dataset,
        ),
        raw_response_digest,
        outcome_row_present: false,
        labels_accessed: false,
        metrics_computed: false,
        prior_private_evaluation_accessed: false,
        credential_free: true,
        read_only: true,
        sanitized: true,
        capsule_digest: String::new(),
    };
    capsule.capsule_digest = input_capsule_digest(&capsule);
    validate_input_capsule(&capsule)?;
    decode_input_capsule(&encode_input_capsule(&capsule)?)?;
    Ok((capsule, rows.clone()))
}

fn build_input_receipt(
    registration: &MomentumProspectiveEpochRegistrationV4,
    status: MomentumProspectiveSeriesInputStatusV4,
    http_status_class: Option<String>,
    returned_row_count: usize,
    verified_row_count: usize,
    raw_response_digest: Option<String>,
    input_capsule_digest: Option<String>,
) -> Result<MomentumProspectiveSeriesInputReceiptV4, String> {
    let attempted = status != MomentumProspectiveSeriesInputStatusV4::ReadyNotAttempted;
    let mut value = MomentumProspectiveSeriesInputReceiptV4 {
        receipt_version: INPUT_RECEIPT_VERSION.to_string(),
        series_digest: registration.series_digest.clone(),
        epoch_registration_digest: registration.registration_digest.clone(),
        request_attempted: attempted,
        request_count: usize::from(attempted),
        retry_count: 0,
        transport_construction_count: usize::from(attempted),
        status,
        http_status_class,
        returned_row_count,
        verified_row_count,
        raw_response_digest,
        input_capsule_digest,
        terminal: attempted,
        receipt_digest: String::new(),
    };
    value.receipt_digest = input_receipt_digest(&value);
    validate_input_receipt(&value)?;
    Ok(value)
}

fn assemble_context(
    series: &MomentumProspectiveSeriesV4,
    registration: &MomentumProspectiveEpochRegistrationV4,
    mut canonical: BTreeMap<u64, CanonicalRowV4>,
    new_rows: &[HistoricalOhlcvRow],
    input_capsule: &MomentumProspectiveSeriesInputCapsuleV4,
) -> Result<
    (
        Vec<HistoricalOhlcvRow>,
        MomentumSeriesContextUseProofV4,
        MomentumSeriesContextAssemblyProofV4,
    ),
    String,
> {
    for row in new_rows {
        let use_class = if row.timestamp_ms == registration.event_timestamp_ms {
            MomentumSeriesContextUseV4::CurrentProspectiveEventInput
        } else {
            MomentumSeriesContextUseV4::NewIncrementalRawContext
        };
        let reference = MomentumSeriesCanonicalRowRefV4 {
            timestamp_ms: row.timestamp_ms,
            raw_row_digest: row_identity_digest(row),
            source_capsule_digest: input_capsule.capsule_digest.clone(),
            use_class,
        };
        if canonical
            .insert(
                row.timestamp_ms,
                CanonicalRowV4 {
                    row: row.clone(),
                    reference,
                },
            )
            .is_some()
        {
            return Err("V4 series acquired row duplicates canonical evidence".to_string());
        }
    }
    let context = registration
        .exact_context_timestamp_ms
        .iter()
        .map(|timestamp| {
            canonical
                .get(timestamp)
                .cloned()
                .ok_or_else(|| "V4 series assembled context row unavailable".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if context.len() != series.context_row_count
        || context
            .windows(2)
            .any(|pair| pair[1].row.timestamp_ms != pair[0].row.timestamp_ms + series.cadence_ms)
        || context.last().map(|row| row.row.timestamp_ms) != Some(registration.event_timestamp_ms)
        || context
            .iter()
            .any(|row| row.row.timestamp_ms == registration.outcome_timestamp_ms)
        || context
            .iter()
            .any(|row| row_identity_digest(&row.row) != row.reference.raw_row_digest)
    {
        return Err("V4 series exact context assembly rejected".to_string());
    }
    let entries = context
        .iter()
        .map(|row| {
            let mut value = MomentumSeriesContextUseEntryV4 {
                timestamp_ms: row.row.timestamp_ms,
                raw_row_digest: row.reference.raw_row_digest.clone(),
                source_capsule_digest: row.reference.source_capsule_digest.clone(),
                use_class: row.reference.use_class,
                feature_construction_allowed: true,
                training_forbidden: true,
                normalizer_fitting_forbidden: true,
                label_use_forbidden: true,
                metric_use_forbidden: true,
                reward_use_forbidden: true,
                participant_selection_forbidden: true,
                entry_digest: String::new(),
            };
            value.entry_digest = context_use_entry_digest(&value);
            validate_context_use_entry(&value)?;
            Ok(value)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut use_proof = MomentumSeriesContextUseProofV4 {
        proof_version: CONTEXT_USE_PROOF_VERSION.to_string(),
        series_digest: series.series_digest.clone(),
        epoch_registration_digest: registration.registration_digest.clone(),
        entries,
        prior_opening_bundle_used_as_raw_source: false,
        prior_private_scores_accessed: false,
        prior_label_used_as_feature: false,
        reward_eligibility_used_as_feature: false,
        proof_digest: String::new(),
    };
    use_proof.proof_digest = context_use_proof_digest(&use_proof);
    validate_context_use_proof(&use_proof)?;
    let mut assembly = MomentumSeriesContextAssemblyProofV4 {
        proof_version: CONTEXT_ASSEMBLY_VERSION.to_string(),
        series_digest: series.series_digest.clone(),
        epoch_registration_digest: registration.registration_digest.clone(),
        input_capsule_digest: input_capsule.capsule_digest.clone(),
        context_use_proof_digest: use_proof.proof_digest.clone(),
        exact_context_timestamp_ms: context.iter().map(|row| row.row.timestamp_ms).collect(),
        exact_row_digests: context
            .iter()
            .map(|row| row.reference.raw_row_digest.clone())
            .collect(),
        exact_row_count: context.len(),
        strict_chronology_verified: true,
        all_row_digests_verified: true,
        event_timestamp_is_last: true,
        outcome_timestamp_absent: true,
        proof_digest: String::new(),
    };
    assembly.proof_digest = context_assembly_digest(&assembly);
    validate_context_assembly(&assembly)?;
    Ok((
        context.into_iter().map(|row| row.row).collect(),
        use_proof,
        assembly,
    ))
}

fn participant_role_name(role: MomentumRawFeatureRoleV4) -> &'static str {
    match role {
        MomentumRawFeatureRoleV4::LearnedRawLogistic => "RawFeatureLogisticV4",
        MomentumRawFeatureRoleV4::LearnedInteractionLogistic => "RawFeatureInteractionLogisticV4",
        MomentumRawFeatureRoleV4::ConstantBenchmark => "TrainingPrevalenceConstantV4",
    }
}

fn derive_prediction_artifacts(
    root: &Path,
    snapshots: &[DataSnapshot],
    reservation: &ProtectedEvaluationReservationV1,
    series: &MomentumProspectiveSeriesV4,
    adoption: &MomentumProspectiveSeriesAdoptionV4,
    registration: &MomentumProspectiveEpochRegistrationV4,
    receipt: &MomentumProspectiveSeriesInputReceiptV4,
    capsule: &MomentumProspectiveSeriesInputCapsuleV4,
    use_proof: &MomentumSeriesContextUseProofV4,
    assembly: &MomentumSeriesContextAssemblyProofV4,
    context: &[HistoricalOhlcvRow],
) -> Result<
    (
        Vec<MomentumSeriesParticipantPredictionSealV4>,
        MomentumProspectiveSeriesPredictionCapsuleV4,
        MomentumProspectiveSeriesJournalEntryV4,
        MomentumProspectiveSeriesOutcomePlanV4,
    ),
    String,
> {
    let source = reopen_momentum_v4_1_future_source(root)?;
    if source.roster.roster_digest != series.frozen_roster_digest
        || source.evaluation.registration_digest
            != reopen_momentum_v4_3_sealed_chain(root)?
                .lifecycle
                .evaluation_registration_digest
        || source.source_family.participants.len() != 3
    {
        return Err("V4 series frozen source identity rejected".to_string());
    }
    let replay = reconstruct_frozen_momentum_v4(root, snapshots, reservation)?;
    let prediction =
        predict_frozen_momentum_v4_event(&replay, &series.participant_digests, context)?;
    if prediction.participant_predictions.len() != 3 {
        return Err("V4 series frozen prediction count rejected".to_string());
    }
    let seals = series
        .participant_digests
        .iter()
        .enumerate()
        .map(|(index, participant_digest)| {
            let predicted = prediction
                .participant_predictions
                .iter()
                .find(|value| &value.participant_digest == participant_digest)
                .ok_or_else(|| "V4 series participant prediction unavailable".to_string())?;
            let participant = source
                .source_family
                .participants
                .iter()
                .find(|value| &value.participant_digest == participant_digest)
                .ok_or_else(|| "V4 series frozen participant unavailable".to_string())?;
            if predicted.config_digest != participant.config_digest
                || predicted.parameter_digest != participant.parameter_digest
                || predicted.normalizer_digest != participant.normalizer_digest
                || predicted.model_artifact_digest != participant.model_artifact_digest
                || predicted.feature_schema_digest != participant.input_feature_schema_digest
                || predicted.training_identity_digest != participant.training_identity_digest
                || predicted.parameter_digest != series.parameter_digests[index]
                || predicted.normalizer_digest != series.normalizer_digests[index]
            {
                return Err("V4 series participant continuity rejected".to_string());
            }
            let prediction_digest = stable_hash_string(&format!(
                "momentum-v4-series-prediction:{}:{}:{}:{}:{}",
                participant_digest,
                registration.event_timestamp_ms,
                receipt.receipt_digest,
                assembly.proof_digest,
                predicted.probability_bits
            ));
            let mut value = MomentumSeriesParticipantPredictionSealV4 {
                seal_version: PREDICTION_SEAL_VERSION.to_string(),
                series_digest: series.series_digest.clone(),
                epoch_number: registration.epoch_number,
                epoch_registration_digest: registration.registration_digest.clone(),
                participant_digest: participant_digest.clone(),
                participant_role: participant_role_name(participant.participant_role).to_string(),
                event_timestamp_ms: registration.event_timestamp_ms,
                input_receipt_digest: receipt.receipt_digest.clone(),
                input_capsule_digest: capsule.capsule_digest.clone(),
                context_use_proof_digest: use_proof.proof_digest.clone(),
                context_assembly_proof_digest: assembly.proof_digest.clone(),
                feature_identity_digest: prediction.feature_identity_digest.clone(),
                prediction_probability_bits: predicted.probability_bits,
                prediction_digest,
                participant_identity_verified: true,
                parameter_updates: 0,
                normalizer_refits: 0,
                prior_score_reads: 0,
                outcome_access_count: 0,
                seal_digest: String::new(),
            };
            value.seal_digest = prediction_seal_digest(&value);
            validate_prediction_seal(&value)?;
            Ok(value)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut prediction_capsule = MomentumProspectiveSeriesPredictionCapsuleV4 {
        capsule_version: PREDICTION_CAPSULE_VERSION.to_string(),
        series_digest: series.series_digest.clone(),
        epoch_registration_digest: registration.registration_digest.clone(),
        event_timestamp_ms: registration.event_timestamp_ms,
        input_receipt_digest: receipt.receipt_digest.clone(),
        input_capsule_digest: capsule.capsule_digest.clone(),
        context_assembly_proof_digest: assembly.proof_digest.clone(),
        participant_seal_digests: seals.iter().map(|seal| seal.seal_digest.clone()).collect(),
        participant_prediction_digests: seals
            .iter()
            .map(|seal| seal.prediction_digest.clone())
            .collect(),
        probabilities_hidden: true,
        labels_hidden: true,
        prior_scores_accessed: false,
        outcome_accessed: false,
        metrics_computed: false,
        winner_selected: false,
        ranking_created: false,
        reward_applied: false,
        penalty_applied: false,
        chair_action_taken: false,
        capsule_digest: String::new(),
    };
    prediction_capsule.capsule_digest = prediction_capsule_digest(&prediction_capsule);
    validate_prediction_capsule(&prediction_capsule)?;
    let mut journal = MomentumProspectiveSeriesJournalEntryV4 {
        journal_version: JOURNAL_VERSION.to_string(),
        series_digest: series.series_digest.clone(),
        epoch_number: registration.epoch_number,
        event_one_adoption_digest: adoption.adoption_digest.clone(),
        previous_epoch_ledger_entry_digest: adoption.evaluation_ledger_entry_digest.clone(),
        context_delta_plan_digest: registration.context_delta_plan_digest.clone(),
        event_timestamp_ms: registration.event_timestamp_ms,
        registration_created_at_ms: registration.registration_created_at_ms,
        input_finality_boundary_ms: registration.input_finality_boundary_ms,
        input_receipt_digest: receipt.receipt_digest.clone(),
        input_capsule_digest: capsule.capsule_digest.clone(),
        context_assembly_proof_digest: assembly.proof_digest.clone(),
        prediction_capsule_digest: prediction_capsule.capsule_digest.clone(),
        participant_seal_digests: prediction_capsule.participant_seal_digests.clone(),
        participant_prediction_digests: prediction_capsule.participant_prediction_digests.clone(),
        deterministic_fixed_cadence_selection: true,
        prior_event_scores_read: false,
        prior_event_correctness_read: false,
        registration_preceded_input_finality: true,
        input_acquisition_preceded_prediction: true,
        prediction_preceded_outcome_access: true,
        outcome_stage_locked: true,
        winner_selected: false,
        ranking_created: false,
        reward_applied: false,
        penalty_applied: false,
        chair_action_taken: false,
        trading_action_taken: false,
        entry_digest: String::new(),
    };
    journal.entry_digest = journal_entry_digest(&journal);
    validate_journal(&journal)?;
    let mut outcome_plan = MomentumProspectiveSeriesOutcomePlanV4 {
        plan_version: OUTCOME_PLAN_VERSION.to_string(),
        series_digest: series.series_digest.clone(),
        epoch_registration_digest: registration.registration_digest.clone(),
        prediction_capsule_digest: prediction_capsule.capsule_digest.clone(),
        event_timestamp_ms: registration.event_timestamp_ms,
        prediction_horizon: series.prediction_horizon,
        required_outcome_timestamp_ms: vec![registration.outcome_timestamp_ms],
        outcome_finality_boundary_ms: registration.outcome_finality_boundary_ms,
        maximum_outcome_requests: 1,
        maximum_outcome_retries: 0,
        outcome_acquisition_count: 0,
        outcome_opening_count: 0,
        labels_hidden_until_opening: true,
        one_time_opening_required: true,
        outcome_stage_locked_before_finality: true,
        plan_digest: String::new(),
    };
    outcome_plan.plan_digest = outcome_plan_digest(&outcome_plan);
    validate_outcome_plan(&outcome_plan)?;
    Ok((seals, prediction_capsule, journal, outcome_plan))
}

fn persist_prediction_seals(
    root: &Path,
    values: &[MomentumSeriesParticipantPredictionSealV4],
) -> Result<(usize, usize), String> {
    if values.len() != 3 {
        return Err("V4 series prediction seal persistence count rejected".to_string());
    }
    let directory = root.join("participant_prediction_seals");
    let mut counts = (0, 0);
    for value in values {
        let path = directory.join(format!("{}.pb", value.seal_digest));
        if path.exists() {
            let stored = decode_prediction_seal(
                &fs::read(&path)
                    .map_err(|_| "V4 series prediction seal reread failed".to_string())?,
            )?;
            if stored != *value {
                return Err("V4 series prediction seal replay mismatch".to_string());
            }
            continue;
        }
        add_counts(
            &mut counts,
            persist_artifact(
                &path,
                &encode_prediction_seal(value)?,
                &value.seal_digest,
                |bytes| Ok(decode_prediction_seal(bytes)?.seal_digest),
            )?,
        );
    }
    Ok(counts)
}

fn reopen_prediction_seals(
    root: &Path,
) -> Result<Vec<MomentumSeriesParticipantPredictionSealV4>, String> {
    let directory = root.join("participant_prediction_seals");
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(&directory)
        .map_err(|_| "V4 series prediction seal directory read failed".to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "pb"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .iter()
        .map(|path| {
            fs::read(path)
                .map_err(|_| "V4 series prediction seal read failed".to_string())
                .and_then(|bytes| decode_prediction_seal(&bytes))
        })
        .collect()
}

fn persist_success_artifacts(
    root: &Path,
    raw_response: &[u8],
    receipt: &MomentumProspectiveSeriesInputReceiptV4,
    input_capsule: &MomentumProspectiveSeriesInputCapsuleV4,
    use_proof: &MomentumSeriesContextUseProofV4,
    assembly: &MomentumSeriesContextAssemblyProofV4,
    seals: &[MomentumSeriesParticipantPredictionSealV4],
    prediction_capsule: &MomentumProspectiveSeriesPredictionCapsuleV4,
    journal: &MomentumProspectiveSeriesJournalEntryV4,
    outcome_plan: &MomentumProspectiveSeriesOutcomePlanV4,
    status: &MomentumProspectiveEpochStatusReceiptV4,
) -> Result<(usize, usize), String> {
    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_raw_input(root, &input_capsule.raw_response_digest, raw_response)?,
    );
    add_counts(
        &mut counts,
        persist_pb_if_absent(
            root,
            "input_capsules",
            &input_capsule.capsule_digest,
            input_capsule,
            &encode_input_capsule(input_capsule)?,
            decode_input_capsule,
            |value| &value.capsule_digest,
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb_if_absent(
            root,
            "input_receipts",
            &receipt.receipt_digest,
            receipt,
            &encode_input_receipt(receipt)?,
            decode_input_receipt,
            |value| &value.receipt_digest,
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb_if_absent(
            root,
            "context_use_proofs",
            &use_proof.proof_digest,
            use_proof,
            &encode_context_use_proof(use_proof)?,
            decode_context_use_proof,
            |value| &value.proof_digest,
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb_if_absent(
            root,
            "context_assembly_proofs",
            &assembly.proof_digest,
            assembly,
            &encode_context_assembly(assembly)?,
            decode_context_assembly,
            |value| &value.proof_digest,
        )?,
    );
    add_counts(&mut counts, persist_prediction_seals(root, seals)?);
    add_counts(
        &mut counts,
        persist_pb_if_absent(
            root,
            "prediction_capsules",
            &prediction_capsule.capsule_digest,
            prediction_capsule,
            &encode_prediction_capsule(prediction_capsule)?,
            decode_prediction_capsule,
            |value| &value.capsule_digest,
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb_if_absent(
            root,
            "series_journal",
            &journal.entry_digest,
            journal,
            &encode_journal(journal)?,
            decode_journal,
            |value| &value.entry_digest,
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb_if_absent(
            root,
            "outcome_plans",
            &outcome_plan.plan_digest,
            outcome_plan,
            &encode_outcome_plan(outcome_plan)?,
            decode_outcome_plan,
            |value| &value.plan_digest,
        )?,
    );
    add_counts(&mut counts, persist_status_if_absent(root, status)?);
    Ok(counts)
}

fn readiness(
    observed_timestamp_ms: u64,
    registration: &MomentumProspectiveEpochRegistrationV4,
    receipt: Option<&MomentumProspectiveSeriesInputReceiptV4>,
    prediction_capsule: Option<&MomentumProspectiveSeriesPredictionCapsuleV4>,
) -> MomentumProspectiveEpochReadinessV4 {
    if prediction_capsule.is_some() {
        return MomentumProspectiveEpochReadinessV4::PredictionAlreadySealed;
    }
    if let Some(receipt) = receipt {
        if validate_input_receipt(receipt).is_err()
            || receipt.series_digest != registration.series_digest
            || receipt.epoch_registration_digest != registration.registration_digest
        {
            return MomentumProspectiveEpochReadinessV4::IntegrityFailure;
        }
        match receipt.status {
            MomentumProspectiveSeriesInputStatusV4::EvidenceAcquired => {
                return if observed_timestamp_ms >= registration.outcome_finality_boundary_ms {
                    MomentumProspectiveEpochReadinessV4::PredictionSealWindowExpired
                } else {
                    MomentumProspectiveEpochReadinessV4::ReadyForLocalPredictionRecovery
                };
            }
            MomentumProspectiveSeriesInputStatusV4::TerminalTransportFailure
            | MomentumProspectiveSeriesInputStatusV4::TerminalValidationFailure => {
                return MomentumProspectiveEpochReadinessV4::PriorInputAttemptTerminal;
            }
            MomentumProspectiveSeriesInputStatusV4::ReadyNotAttempted => {}
        }
    }
    if observed_timestamp_ms < registration.input_finality_boundary_ms {
        MomentumProspectiveEpochReadinessV4::RegisteredAwaitingInputFinality
    } else if observed_timestamp_ms >= registration.outcome_finality_boundary_ms {
        MomentumProspectiveEpochReadinessV4::PredictionSealWindowExpired
    } else {
        MomentumProspectiveEpochReadinessV4::ReadyForInputAcquisition
    }
}

#[allow(clippy::too_many_arguments)]
fn build_status(
    adoption: &MomentumProspectiveSeriesAdoptionV4,
    audit: &MomentumProspectiveCandidateGapAuditV4,
    delta: &MomentumCanonicalContextDeltaPlanV4,
    registration: &MomentumProspectiveEpochRegistrationV4,
    current_readiness: MomentumProspectiveEpochReadinessV4,
    receipt: Option<&MomentumProspectiveSeriesInputReceiptV4>,
    input_capsule: Option<&MomentumProspectiveSeriesInputCapsuleV4>,
    assembly: Option<&MomentumSeriesContextAssemblyProofV4>,
    prediction_capsule: Option<&MomentumProspectiveSeriesPredictionCapsuleV4>,
    journal: Option<&MomentumProspectiveSeriesJournalEntryV4>,
    outcome_plan: Option<&MomentumProspectiveSeriesOutcomePlanV4>,
    reward_status: MomentumRewardEligibilityStatusV4_4,
    protected_unchanged: bool,
    active_unchanged: bool,
    safety_counters: MomentumProspectiveSeriesSafetyCountersV4,
) -> Result<MomentumProspectiveEpochStatusReceiptV4, String> {
    let mut value = MomentumProspectiveEpochStatusReceiptV4 {
        status_version: STATUS_VERSION.to_string(),
        series_digest: registration.series_digest.clone(),
        event_one_adoption_digest: adoption.adoption_digest.clone(),
        candidate_gap_audit_digest: audit.audit_digest.clone(),
        context_delta_plan_digest: delta.plan_digest.clone(),
        epoch_registration_digest: registration.registration_digest.clone(),
        epoch_number: registration.epoch_number,
        event_timestamp_ms: registration.event_timestamp_ms,
        input_finality_boundary_ms: registration.input_finality_boundary_ms,
        outcome_timestamp_ms: registration.outcome_timestamp_ms,
        outcome_finality_boundary_ms: registration.outcome_finality_boundary_ms,
        exact_context_timestamp_ms: registration.exact_context_timestamp_ms.clone(),
        exact_missing_timestamp_ms: registration.exact_missing_timestamp_ms.clone(),
        readiness: current_readiness,
        input_receipt_digest: receipt.map(|value| value.receipt_digest.clone()),
        input_capsule_digest: input_capsule.map(|value| value.capsule_digest.clone()),
        context_assembly_proof_digest: assembly.map(|value| value.proof_digest.clone()),
        participant_prediction_digests: prediction_capsule
            .map(|value| value.participant_prediction_digests.clone())
            .unwrap_or_default(),
        prediction_capsule_digest: prediction_capsule.map(|value| value.capsule_digest.clone()),
        journal_entry_digest: journal.map(|value| value.entry_digest.clone()),
        outcome_plan_digest: outcome_plan.map(|value| value.plan_digest.clone()),
        total_event_count: adoption.total_event_count,
        scorable_event_count: adoption.scorable_event_count,
        reward_eligibility_status: reward_status,
        protected_artifacts_unchanged: protected_unchanged,
        active_state_unchanged: active_unchanged,
        safety_counters,
        status_digest: String::new(),
    };
    value.status_digest = status_digest(&value);
    validate_status(&value)?;
    Ok(value)
}

fn idle_safety_counters() -> MomentumProspectiveSeriesSafetyCountersV4 {
    MomentumProspectiveSeriesSafetyCountersV4 {
        maximum_concurrency: 1,
        active_committee_count: 3,
        ..Default::default()
    }
}

fn ensure_same<T: PartialEq>(persisted: Option<&T>, derived: &T, name: &str) -> Result<(), String> {
    if persisted.is_some_and(|value| value != derived) {
        return Err(format!("V4 series persisted {name} changed"));
    }
    Ok(())
}

fn reward_status(event: &EventOneStateV4) -> Result<MomentumRewardEligibilityStatusV4_4, String> {
    event
        .opening
        .reward_eligibility
        .as_ref()
        .map(|value| value.status)
        .ok_or_else(|| "V4 series event-one eligibility unavailable".to_string())
}

fn local_prediction_recovery_allowed(
    mode: MomentumProspectiveSeriesRunModeV4,
    current_readiness: MomentumProspectiveEpochReadinessV4,
) -> bool {
    mode == MomentumProspectiveSeriesRunModeV4::ExecuteInput
        && current_readiness == MomentumProspectiveEpochReadinessV4::ReadyForLocalPredictionRecovery
}

fn input_acquisition_allowed(current_readiness: MomentumProspectiveEpochReadinessV4) -> bool {
    current_readiness == MomentumProspectiveEpochReadinessV4::ReadyForInputAcquisition
}

fn report(
    status: MomentumProspectiveEpochStatusReceiptV4,
    series: MomentumProspectiveSeriesV4,
    event_one_adoption: MomentumProspectiveSeriesAdoptionV4,
    candidate_gap_audit: MomentumProspectiveCandidateGapAuditV4,
    context_delta_plan: MomentumCanonicalContextDeltaPlanV4,
    epoch_registration: MomentumProspectiveEpochRegistrationV4,
    input_receipt: Option<MomentumProspectiveSeriesInputReceiptV4>,
    input_capsule: Option<MomentumProspectiveSeriesInputCapsuleV4>,
    context_use_proof: Option<MomentumSeriesContextUseProofV4>,
    context_assembly_proof: Option<MomentumSeriesContextAssemblyProofV4>,
    prediction_capsule: Option<MomentumProspectiveSeriesPredictionCapsuleV4>,
    journal_entry: Option<MomentumProspectiveSeriesJournalEntryV4>,
    outcome_plan: Option<MomentumProspectiveSeriesOutcomePlanV4>,
    counts: (usize, usize),
) -> MomentumProspectiveSeriesReportV4 {
    MomentumProspectiveSeriesReportV4 {
        status,
        series,
        event_one_adoption,
        candidate_gap_audit,
        context_delta_plan,
        epoch_registration,
        input_receipt,
        input_capsule,
        context_use_proof,
        context_assembly_proof,
        prediction_capsule,
        journal_entry,
        outcome_plan,
        artifacts_written: counts.0,
        duplicate_artifact_count: counts.1,
    }
}

fn validate_persisted_prediction_chain(
    series: &MomentumProspectiveSeriesV4,
    adoption: &MomentumProspectiveSeriesAdoptionV4,
    registration: &MomentumProspectiveEpochRegistrationV4,
    receipt: &MomentumProspectiveSeriesInputReceiptV4,
    input_capsule: &MomentumProspectiveSeriesInputCapsuleV4,
    use_proof: &MomentumSeriesContextUseProofV4,
    assembly: &MomentumSeriesContextAssemblyProofV4,
    seals: &[MomentumSeriesParticipantPredictionSealV4],
    prediction_capsule: &MomentumProspectiveSeriesPredictionCapsuleV4,
    journal: &MomentumProspectiveSeriesJournalEntryV4,
    outcome_plan: &MomentumProspectiveSeriesOutcomePlanV4,
) -> Result<(), String> {
    let seals_by_digest = seals
        .iter()
        .map(|value| (value.seal_digest.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let capsule_seal_bindings_match = seals_by_digest.len() == seals.len()
        && prediction_capsule.participant_seal_digests.len() == seals.len()
        && prediction_capsule.participant_prediction_digests.len() == seals.len()
        && prediction_capsule
            .participant_seal_digests
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            == seals.len()
        && prediction_capsule
            .participant_seal_digests
            .iter()
            .zip(&prediction_capsule.participant_prediction_digests)
            .all(|(seal_digest, prediction_digest)| {
                seals_by_digest
                    .get(seal_digest.as_str())
                    .is_some_and(|seal| seal.prediction_digest == *prediction_digest)
            });
    let participant_roster_matches = series.participant_digests.len() == seals.len()
        && seals
            .iter()
            .map(|value| value.participant_digest.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            == seals.len()
        && series.participant_digests.iter().all(|participant_digest| {
            seals
                .iter()
                .any(|seal| seal.participant_digest == *participant_digest)
        });
    if registration.series_digest != series.series_digest
        || receipt.status != MomentumProspectiveSeriesInputStatusV4::EvidenceAcquired
        || receipt.epoch_registration_digest != registration.registration_digest
        || receipt.input_capsule_digest.as_deref() != Some(input_capsule.capsule_digest.as_str())
        || input_capsule.epoch_registration_digest != registration.registration_digest
        || input_capsule.context_delta_plan_digest != registration.context_delta_plan_digest
        || input_capsule.provider_id != registration.provider_id
        || input_capsule.request_attempt_count != 1
        || input_capsule.exact_timestamp_ms != registration.exact_missing_timestamp_ms
        || use_proof.epoch_registration_digest != registration.registration_digest
        || assembly.epoch_registration_digest != registration.registration_digest
        || assembly.input_capsule_digest != input_capsule.capsule_digest
        || assembly.context_use_proof_digest != use_proof.proof_digest
        || assembly.exact_context_timestamp_ms != registration.exact_context_timestamp_ms
        || seals.len() != 3
        || !participant_roster_matches
        || seals.iter().any(|value| {
            value.series_digest != series.series_digest
                || value.epoch_number != registration.epoch_number
                || value.epoch_registration_digest != registration.registration_digest
                || value.input_receipt_digest != receipt.receipt_digest
                || value.input_capsule_digest != input_capsule.capsule_digest
                || value.context_use_proof_digest != use_proof.proof_digest
                || value.context_assembly_proof_digest != assembly.proof_digest
        })
        || prediction_capsule.epoch_registration_digest != registration.registration_digest
        || prediction_capsule.input_receipt_digest != receipt.receipt_digest
        || prediction_capsule.input_capsule_digest != input_capsule.capsule_digest
        || prediction_capsule.context_assembly_proof_digest != assembly.proof_digest
        || !capsule_seal_bindings_match
        || prediction_capsule.reward_applied
        || prediction_capsule.penalty_applied
        || prediction_capsule.chair_action_taken
        || journal.epoch_number != registration.epoch_number
        || journal.event_one_adoption_digest != adoption.adoption_digest
        || journal.context_delta_plan_digest != registration.context_delta_plan_digest
        || journal.prediction_capsule_digest != prediction_capsule.capsule_digest
        || journal.participant_seal_digests != prediction_capsule.participant_seal_digests
        || journal.participant_prediction_digests
            != prediction_capsule.participant_prediction_digests
        || journal.prior_event_correctness_read
        || outcome_plan.epoch_registration_digest != registration.registration_digest
        || outcome_plan.prediction_capsule_digest != prediction_capsule.capsule_digest
        || outcome_plan.required_outcome_timestamp_ms != [registration.outcome_timestamp_ms]
        || outcome_plan.outcome_finality_boundary_ms != registration.outcome_finality_boundary_ms
    {
        return Err("V4 series persisted prediction chain binding rejected".to_string());
    }
    Ok(())
}

fn reopen_acquired_rows(
    root: &Path,
    registration: &MomentumProspectiveEpochRegistrationV4,
    capsule: &MomentumProspectiveSeriesInputCapsuleV4,
) -> Result<(Vec<u8>, Vec<HistoricalOhlcvRow>), String> {
    let path = root
        .join("raw_input")
        .join(format!("{}.json", capsule.raw_response_digest));
    let raw = fs::read(path).map_err(|_| "V4 series acquired raw input unavailable".to_string())?;
    if stable_hash_string(&format!("momentum-v4-series-raw-input:{raw:?}"))
        != capsule.raw_response_digest
    {
        return Err("V4 series acquired raw input digest rejected".to_string());
    }
    let text = std::str::from_utf8(&raw)
        .map_err(|_| "V4 series acquired raw input UTF-8 rejected".to_string())?;
    let dataset = parse_upbit_daily_ohlcv_v0(text, &registration.symbol)?;
    let timestamps = dataset
        .rows
        .iter()
        .map(|row| row.timestamp_ms)
        .collect::<Vec<_>>();
    let row_digests = dataset
        .rows
        .iter()
        .map(row_identity_digest)
        .collect::<Vec<_>>();
    if timestamps != registration.exact_missing_timestamp_ms
        || timestamps != capsule.exact_timestamp_ms
        || row_digests != capsule.row_identity_digests
        || historical_replay_dataset_digest_v0(&dataset) != capsule.normalized_dataset_digest
        || dataset
            .rows
            .iter()
            .any(|row| !row_is_valid(row, &registration.symbol))
    {
        return Err("V4 series acquired raw input reopening rejected".to_string());
    }
    Ok((raw, dataset.rows))
}

fn validate_run_authority(
    mode: MomentumProspectiveSeriesRunModeV4,
    network_allowed: bool,
    one_time_request_confirmed: bool,
    requested_epoch: Option<u64>,
) -> Result<(), String> {
    if mode != MomentumProspectiveSeriesRunModeV4::ExecuteInput
        && (network_allowed || one_time_request_confirmed || requested_epoch.is_some())
    {
        return Err("V4 series non-input mode rejects network authority".to_string());
    }
    if mode == MomentumProspectiveSeriesRunModeV4::ExecuteInput
        && (!network_allowed || !one_time_request_confirmed || requested_epoch.is_none())
    {
        return Err(
            "V4 series input execution requires epoch, network permission, and exact confirmation"
                .to_string(),
        );
    }
    Ok(())
}

fn run_with_transport<F>(
    root: &Path,
    snapshots: &[DataSnapshot],
    reservation: &ProtectedEvaluationReservationV1,
    provider_config: &UpbitHistoricalPilotConfigV0,
    observed_timestamp_ms: u64,
    mode: MomentumProspectiveSeriesRunModeV4,
    network_allowed: bool,
    one_time_request_confirmed: bool,
    requested_epoch: Option<u64>,
    transport: F,
) -> Result<MomentumProspectiveSeriesReportV4, String>
where
    F: FnOnce(
        &UpbitHistoricalPilotConfigV0,
        &ReadOnlyProviderRequest,
    )
        -> Result<LearningEvidenceTransportResponseV1, LearningEvidenceTransportFailureV1>,
{
    validate_run_authority(
        mode,
        network_allowed,
        one_time_request_confirmed,
        requested_epoch,
    )?;
    provider_config.validate()?;
    let protected_before = protected_artifacts(root)?;
    let active_before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let event = reopen_event_one(root, provider_config, observed_timestamp_ms)?;
    let series = derive_series(&event, provider_config, &protected_before, &active_before)?;
    let adoption = derive_adoption(&series, &event)?;
    let artifact_root = series_root(root);
    let persisted_series = read_single(&artifact_root.join("series_contracts"), decode_series)?;
    let persisted_adoption = read_single(&artifact_root.join("event_adoptions"), decode_adoption)?;
    ensure_same(persisted_series.as_ref(), &series, "contract")?;
    ensure_same(persisted_adoption.as_ref(), &adoption, "adoption")?;

    let persisted_registration = read_single(
        &artifact_root.join("epoch_registrations"),
        decode_epoch_registration,
    )?;
    let registration_created_at_ms = persisted_registration
        .as_ref()
        .map(|value| value.registration_created_at_ms)
        .unwrap_or(observed_timestamp_ms);
    let prior_outcome_timestamp_ms = match event
        .outcome
        .registration
        .required_outcome_timestamp_ms
        .as_slice()
    {
        [timestamp] => *timestamp,
        _ => return Err("V4 series prior outcome timestamp identity rejected".to_string()),
    };
    let audit = derive_gap_audit(
        &series,
        &adoption,
        prior_outcome_timestamp_ms,
        registration_created_at_ms,
    )?;
    let event_timestamp_ms = derive_next_legal_event(
        &series,
        &adoption,
        prior_outcome_timestamp_ms,
        registration_created_at_ms,
    )?;
    let persisted_audit = read_single(
        &artifact_root.join("candidate_gap_audits"),
        decode_gap_audit,
    )?;
    let persisted_delta = read_single(
        &artifact_root.join("context_delta_plans"),
        decode_delta_plan,
    )?;
    ensure_same(persisted_audit.as_ref(), &audit, "candidate audit")?;
    let (delta, mut canonical) = if let Some(value) = persisted_delta.as_ref() {
        if value.series_digest != series.series_digest
            || value.epoch_number != adoption.adopted_epoch_number + 1
            || value.event_timestamp_ms != event_timestamp_ms
            || value.exact_context_timestamp_ms.len() != series.context_row_count
        {
            return Err("V4 series persisted context delta binding rejected".to_string());
        }
        (value.clone(), None)
    } else {
        let rows = load_canonical_rows(root, &event)?;
        let value = derive_delta_plan(
            &series,
            adoption.adopted_epoch_number + 1,
            event_timestamp_ms,
            &rows,
        )?;
        (value, Some(rows))
    };
    let registration = derive_epoch_registration(
        &series,
        &adoption,
        &delta,
        provider_config,
        registration_created_at_ms,
    )?;
    ensure_same(
        persisted_registration.as_ref(),
        &registration,
        "epoch registration",
    )?;
    if requested_epoch.is_some_and(|value| value != registration.epoch_number) {
        return Err("V4 series requested epoch rejected".to_string());
    }
    if mode == MomentumProspectiveSeriesRunModeV4::ExecuteInput && persisted_registration.is_none()
    {
        return Err("V4 series input execution requires prior preregistration".to_string());
    }

    let persisted_receipt =
        read_single(&artifact_root.join("input_receipts"), decode_input_receipt)?;
    let persisted_input_capsule =
        read_single(&artifact_root.join("input_capsules"), decode_input_capsule)?;
    let persisted_use_proof = read_single(
        &artifact_root.join("context_use_proofs"),
        decode_context_use_proof,
    )?;
    let persisted_assembly = read_single(
        &artifact_root.join("context_assembly_proofs"),
        decode_context_assembly,
    )?;
    let persisted_prediction_capsule = read_single(
        &artifact_root.join("prediction_capsules"),
        decode_prediction_capsule,
    )?;
    let persisted_journal = read_single(&artifact_root.join("series_journal"), decode_journal)?;
    let persisted_outcome_plan =
        read_single(&artifact_root.join("outcome_plans"), decode_outcome_plan)?;
    let persisted_seals = reopen_prediction_seals(&artifact_root)?;
    if let (
        Some(receipt),
        Some(input_capsule),
        Some(use_proof),
        Some(assembly),
        Some(prediction_capsule),
        Some(journal),
        Some(outcome_plan),
    ) = (
        persisted_receipt.as_ref(),
        persisted_input_capsule.as_ref(),
        persisted_use_proof.as_ref(),
        persisted_assembly.as_ref(),
        persisted_prediction_capsule.as_ref(),
        persisted_journal.as_ref(),
        persisted_outcome_plan.as_ref(),
    ) {
        validate_persisted_prediction_chain(
            &series,
            &adoption,
            &registration,
            receipt,
            input_capsule,
            use_proof,
            assembly,
            &persisted_seals,
            prediction_capsule,
            journal,
            outcome_plan,
        )?;
        let protected_after = protected_artifacts(root)?;
        let active_after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
        let status = build_status(
            &adoption,
            &audit,
            &delta,
            &registration,
            MomentumProspectiveEpochReadinessV4::PredictionAlreadySealed,
            Some(receipt),
            Some(input_capsule),
            Some(assembly),
            Some(prediction_capsule),
            Some(journal),
            Some(outcome_plan),
            reward_status(&event)?,
            protected_before == protected_after,
            active_before == active_after,
            idle_safety_counters(),
        )?;
        return Ok(report(
            status,
            series,
            adoption,
            audit,
            delta,
            registration,
            persisted_receipt,
            persisted_input_capsule,
            persisted_use_proof,
            persisted_assembly,
            persisted_prediction_capsule,
            persisted_journal,
            persisted_outcome_plan,
            (0, 0),
        ));
    }
    if persisted_prediction_capsule.is_some()
        || persisted_use_proof.is_some()
        || persisted_assembly.is_some()
        || !persisted_seals.is_empty()
        || persisted_journal.is_some()
        || persisted_outcome_plan.is_some()
        || persisted_input_capsule.is_some() && persisted_receipt.is_none()
        || persisted_receipt.as_ref().is_some_and(|value| {
            value.status == MomentumProspectiveSeriesInputStatusV4::EvidenceAcquired
                && persisted_input_capsule.is_none()
        })
    {
        return Err("V4 series incomplete persisted prediction chain rejected".to_string());
    }
    if let (Some(receipt), Some(input_capsule)) =
        (persisted_receipt.as_ref(), persisted_input_capsule.as_ref())
    {
        if receipt.status != MomentumProspectiveSeriesInputStatusV4::EvidenceAcquired
            || receipt.series_digest != registration.series_digest
            || receipt.epoch_registration_digest != registration.registration_digest
            || receipt.input_capsule_digest.as_deref()
                != Some(input_capsule.capsule_digest.as_str())
            || input_capsule.series_digest != registration.series_digest
            || input_capsule.epoch_registration_digest != registration.registration_digest
            || input_capsule.context_delta_plan_digest != registration.context_delta_plan_digest
            || input_capsule.provider_id != registration.provider_id
            || input_capsule.request_attempt_count != 1
            || input_capsule.event_timestamp_ms != registration.event_timestamp_ms
            || input_capsule.exact_timestamp_ms != registration.exact_missing_timestamp_ms
        {
            return Err("V4 series terminal input chain rejected".to_string());
        }
        let recovery_readiness =
            readiness(observed_timestamp_ms, &registration, Some(receipt), None);
        if !local_prediction_recovery_allowed(mode, recovery_readiness) {
            let protected_after = protected_artifacts(root)?;
            let active_after =
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
            let status = build_status(
                &adoption,
                &audit,
                &delta,
                &registration,
                recovery_readiness,
                Some(receipt),
                Some(input_capsule),
                None,
                None,
                None,
                None,
                reward_status(&event)?,
                protected_before == protected_after,
                active_before == active_after,
                idle_safety_counters(),
            )?;
            return Ok(report(
                status,
                series,
                adoption,
                audit,
                delta,
                registration,
                persisted_receipt,
                persisted_input_capsule,
                None,
                None,
                None,
                None,
                None,
                (0, 0),
            ));
        }
        let (raw_response, new_rows) =
            reopen_acquired_rows(&artifact_root, &registration, input_capsule)?;
        let canonical_rows = match canonical.take() {
            Some(value) => value,
            None => load_canonical_rows(root, &event)?,
        };
        let (context, use_proof, assembly) = assemble_context(
            &series,
            &registration,
            canonical_rows,
            &new_rows,
            input_capsule,
        )?;
        let (seals, prediction_capsule, journal, outcome_plan) = derive_prediction_artifacts(
            root,
            snapshots,
            reservation,
            &series,
            &adoption,
            &registration,
            receipt,
            input_capsule,
            &use_proof,
            &assembly,
            &context,
        )?;
        let mut recovery_safety = idle_safety_counters();
        recovery_safety.canonical_raw_row_reads = context.len();
        recovery_safety.participant_reconstructions = 3;
        recovery_safety.feature_generations = 1;
        recovery_safety.prediction_computations = 3;
        let protected_after = protected_artifacts(root)?;
        let active_after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
        let status = build_status(
            &adoption,
            &audit,
            &delta,
            &registration,
            MomentumProspectiveEpochReadinessV4::PredictionAlreadySealed,
            Some(receipt),
            Some(input_capsule),
            Some(&assembly),
            Some(&prediction_capsule),
            Some(&journal),
            Some(&outcome_plan),
            reward_status(&event)?,
            protected_before == protected_after,
            active_before == active_after,
            recovery_safety,
        )?;
        let counts = persist_success_artifacts(
            &artifact_root,
            &raw_response,
            receipt,
            input_capsule,
            &use_proof,
            &assembly,
            &seals,
            &prediction_capsule,
            &journal,
            &outcome_plan,
            &status,
        )?;
        return Ok(report(
            status,
            series,
            adoption,
            audit,
            delta,
            registration,
            persisted_receipt,
            persisted_input_capsule,
            Some(use_proof),
            Some(assembly),
            Some(prediction_capsule),
            Some(journal),
            Some(outcome_plan),
            counts,
        ));
    }

    let current_readiness = readiness(
        observed_timestamp_ms,
        &registration,
        persisted_receipt.as_ref(),
        None,
    );
    let protected_now = protected_artifacts(root)? == protected_before;
    let active_now =
        stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before;
    let mut base_safety = idle_safety_counters();
    if canonical.is_some() {
        base_safety.canonical_raw_row_reads = delta.canonical_rows.len();
    }
    if matches!(
        mode,
        MomentumProspectiveSeriesRunModeV4::Status | MomentumProspectiveSeriesRunModeV4::DryRun
    ) || mode == MomentumProspectiveSeriesRunModeV4::ExecuteInput
        && !input_acquisition_allowed(current_readiness)
        || persisted_receipt.is_some()
    {
        let status = build_status(
            &adoption,
            &audit,
            &delta,
            &registration,
            current_readiness,
            persisted_receipt.as_ref(),
            persisted_input_capsule.as_ref(),
            None,
            None,
            None,
            None,
            reward_status(&event)?,
            protected_now,
            active_now,
            base_safety,
        )?;
        return Ok(report(
            status,
            series,
            adoption,
            audit,
            delta,
            registration,
            persisted_receipt,
            persisted_input_capsule,
            None,
            None,
            None,
            None,
            None,
            (0, 0),
        ));
    }

    if mode == MomentumProspectiveSeriesRunModeV4::RegisterNextEpoch {
        if observed_timestamp_ms >= registration.input_finality_boundary_ms {
            return Err("V4 series late preregistration rejected".to_string());
        }
        let status = build_status(
            &adoption,
            &audit,
            &delta,
            &registration,
            MomentumProspectiveEpochReadinessV4::RegisteredAwaitingInputFinality,
            None,
            None,
            None,
            None,
            None,
            None,
            reward_status(&event)?,
            protected_now,
            active_now,
            base_safety,
        )?;
        if persisted_series.is_some()
            && persisted_adoption.is_some()
            && persisted_audit.is_some()
            && persisted_delta.is_some()
            && persisted_registration.is_some()
        {
            return Ok(report(
                status,
                series,
                adoption,
                audit,
                delta,
                registration,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                (0, 0),
            ));
        }
        let counts = persist_preregistration(
            &artifact_root,
            &series,
            &adoption,
            &audit,
            &delta,
            &registration,
            &status,
        )?;
        return Ok(report(
            status,
            series,
            adoption,
            audit,
            delta,
            registration,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            counts,
        ));
    }

    if !input_acquisition_allowed(current_readiness) {
        return Err("V4 series input acquisition readiness rejected".to_string());
    }
    let canonical = canonical.unwrap_or(load_canonical_rows(root, &event)?);
    let request = build_provider_request(&registration)?;
    let request_config = request_config(provider_config, &registration)?;
    let mut attempt_safety = idle_safety_counters();
    attempt_safety.network_request_attempts = 1;
    attempt_safety.transport_constructions = 1;
    attempt_safety.canonical_raw_row_reads = delta.canonical_rows.len();
    let mut counts = (0, 0);
    let transport = match transport(&request_config, &request) {
        Ok(value) => value,
        Err(failure) => {
            let http_status_class = match failure {
                LearningEvidenceTransportFailureV1::ProviderRejected {
                    http_status_class, ..
                } => http_status_class,
                LearningEvidenceTransportFailureV1::TimedOut
                | LearningEvidenceTransportFailureV1::Technical => None,
            };
            let receipt = build_input_receipt(
                &registration,
                MomentumProspectiveSeriesInputStatusV4::TerminalTransportFailure,
                http_status_class,
                0,
                0,
                None,
                None,
            )?;
            add_counts(
                &mut counts,
                persist_pb_if_absent(
                    &artifact_root,
                    "input_receipts",
                    &receipt.receipt_digest,
                    &receipt,
                    &encode_input_receipt(&receipt)?,
                    decode_input_receipt,
                    |value| &value.receipt_digest,
                )?,
            );
            let protected_after = protected_artifacts(root)?;
            let active_after =
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
            let status = build_status(
                &adoption,
                &audit,
                &delta,
                &registration,
                MomentumProspectiveEpochReadinessV4::PriorInputAttemptTerminal,
                Some(&receipt),
                None,
                None,
                None,
                None,
                None,
                reward_status(&event)?,
                protected_before == protected_after,
                active_before == active_after,
                attempt_safety,
            )?;
            add_counts(
                &mut counts,
                persist_status_if_absent(&artifact_root, &status)?,
            );
            return Ok(report(
                status,
                series,
                adoption,
                audit,
                delta,
                registration,
                Some(receipt),
                None,
                None,
                None,
                None,
                None,
                None,
                counts,
            ));
        }
    };
    let returned_row_count = transport.response.normalized_dataset.rows.len();
    let (input_capsule, new_rows) =
        match validate_input_response(&registration, &request, &transport) {
            Ok(value) => value,
            Err(_) => {
                let receipt = build_input_receipt(
                    &registration,
                    MomentumProspectiveSeriesInputStatusV4::TerminalValidationFailure,
                    Some(transport.http_status_class),
                    returned_row_count,
                    0,
                    None,
                    None,
                )?;
                add_counts(
                    &mut counts,
                    persist_pb_if_absent(
                        &artifact_root,
                        "input_receipts",
                        &receipt.receipt_digest,
                        &receipt,
                        &encode_input_receipt(&receipt)?,
                        decode_input_receipt,
                        |value| &value.receipt_digest,
                    )?,
                );
                let protected_after = protected_artifacts(root)?;
                let active_after =
                    stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
                let status = build_status(
                    &adoption,
                    &audit,
                    &delta,
                    &registration,
                    MomentumProspectiveEpochReadinessV4::PriorInputAttemptTerminal,
                    Some(&receipt),
                    None,
                    None,
                    None,
                    None,
                    None,
                    reward_status(&event)?,
                    protected_before == protected_after,
                    active_before == active_after,
                    attempt_safety,
                )?;
                add_counts(
                    &mut counts,
                    persist_status_if_absent(&artifact_root, &status)?,
                );
                return Ok(report(
                    status,
                    series,
                    adoption,
                    audit,
                    delta,
                    registration,
                    Some(receipt),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    counts,
                ));
            }
        };
    let receipt = build_input_receipt(
        &registration,
        MomentumProspectiveSeriesInputStatusV4::EvidenceAcquired,
        Some(transport.http_status_class.clone()),
        new_rows.len(),
        new_rows.len(),
        Some(input_capsule.raw_response_digest.clone()),
        Some(input_capsule.capsule_digest.clone()),
    )?;
    add_counts(
        &mut counts,
        persist_pb_if_absent(
            &artifact_root,
            "input_receipts",
            &receipt.receipt_digest,
            &receipt,
            &encode_input_receipt(&receipt)?,
            decode_input_receipt,
            |value| &value.receipt_digest,
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb_if_absent(
            &artifact_root,
            "input_capsules",
            &input_capsule.capsule_digest,
            &input_capsule,
            &encode_input_capsule(&input_capsule)?,
            decode_input_capsule,
            |value| &value.capsule_digest,
        )?,
    );
    add_counts(
        &mut counts,
        persist_raw_input(
            &artifact_root,
            &input_capsule.raw_response_digest,
            &transport.raw_response,
        )?,
    );
    let (context, use_proof, assembly) =
        assemble_context(&series, &registration, canonical, &new_rows, &input_capsule)?;
    let (seals, prediction_capsule, journal, outcome_plan) = derive_prediction_artifacts(
        root,
        snapshots,
        reservation,
        &series,
        &adoption,
        &registration,
        &receipt,
        &input_capsule,
        &use_proof,
        &assembly,
        &context,
    )?;
    attempt_safety.canonical_raw_row_reads = context.len();
    attempt_safety.participant_reconstructions = 3;
    attempt_safety.feature_generations = 1;
    attempt_safety.prediction_computations = 3;
    let protected_after = protected_artifacts(root)?;
    let active_after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let status = build_status(
        &adoption,
        &audit,
        &delta,
        &registration,
        MomentumProspectiveEpochReadinessV4::PredictionAlreadySealed,
        Some(&receipt),
        Some(&input_capsule),
        Some(&assembly),
        Some(&prediction_capsule),
        Some(&journal),
        Some(&outcome_plan),
        reward_status(&event)?,
        protected_before == protected_after,
        active_before == active_after,
        attempt_safety,
    )?;
    add_counts(
        &mut counts,
        persist_success_artifacts(
            &artifact_root,
            &transport.raw_response,
            &receipt,
            &input_capsule,
            &use_proof,
            &assembly,
            &seals,
            &prediction_capsule,
            &journal,
            &outcome_plan,
            &status,
        )?,
    );
    Ok(report(
        status,
        series,
        adoption,
        audit,
        delta,
        registration,
        Some(receipt),
        Some(input_capsule),
        Some(use_proof),
        Some(assembly),
        Some(prediction_capsule),
        Some(journal),
        Some(outcome_plan),
        counts,
    ))
}

/// Opens or advances the append-only Momentum V4 prospective series.
///
/// Network acquisition is possible only in `ExecuteInput` mode with an exact
/// epoch, explicit network permission, and one-time confirmation.
#[allow(clippy::too_many_arguments)]
pub fn run_momentum_prospective_series_v4(
    root: &Path,
    snapshots: &[DataSnapshot],
    reservation: &ProtectedEvaluationReservationV1,
    provider_config: &UpbitHistoricalPilotConfigV0,
    observed_timestamp_ms: u64,
    mode: MomentumProspectiveSeriesRunModeV4,
    network_allowed: bool,
    one_time_request_confirmed: bool,
    requested_epoch: Option<u64>,
) -> Result<MomentumProspectiveSeriesReportV4, String> {
    run_with_transport(
        root,
        snapshots,
        reservation,
        provider_config,
        observed_timestamp_ms,
        mode,
        network_allowed,
        one_time_request_confirmed,
        requested_epoch,
        fetch_upbit_learning_evidence_once_v1,
    )
}

const SERIES_OUTCOME_DIRECTORY: &str = "epoch_two_outcome";
const COMPLETED_PAUSE_DIRECTORY: &str = "completed_continuation_pauses";
const SERIES_LEDGER_VERSION: &str = "momentum-prospective-series-ledger-entry-v4.5";
const SERIES_ELIGIBILITY_VERSION: &str = "momentum-prospective-series-eligibility-v4.5";
const COMPLETED_PAUSE_VERSION: &str = "live-prospective-continuation-pause-v2";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumProspectiveOutcomeReadinessV4 {
    AwaitingOutcomeFinality,
    ReadyForOutcomeAcquisition,
    ReadyForOutcomeOpening,
    OutcomeAlreadyOpened,
    PriorOutcomeAttemptTerminal,
    SealedPredictionMismatch,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumProspectiveOutcomeRunModeV4 {
    Status,
    DryRun,
    ExecuteOutcome,
    OpenOutcome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumProspectiveSeriesEligibilityV4 {
    IneligibleMinimumSamples,
    EligibleForShadowRewardAssessment,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiveProspectiveContinuationPolicyV2 {
    PausedAfterCompletedEpochTwo,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveOutcomeSafetyCountersV4 {
    pub network_request_attempts: usize,
    pub transport_constructions: usize,
    pub retries: usize,
    pub maximum_observed_concurrency: usize,
    pub outcome_raw_loads: usize,
    pub prediction_private_value_reads: usize,
    pub label_derivations: usize,
    pub evaluations: usize,
    pub opening_attempts: usize,
    pub ledger_appends: usize,
    pub eligibility_derivations: usize,
    pub winner_selections: usize,
    pub ranking_creations: usize,
    pub reward_applications: usize,
    pub penalty_applications: usize,
    pub chair_model_executions: usize,
    pub chair_learning_actions: usize,
    pub chair_decisions: usize,
    pub committee_votes: usize,
    pub voice_changes: usize,
    pub tier_changes: usize,
    pub cooldowns: usize,
    pub promotions: usize,
    pub quarantines: usize,
    pub paper_executions: usize,
    pub live_executions: usize,
    pub epoch_three_registrations: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveSeriesLedgerEntryV4 {
    pub ledger_version: String,
    pub previous_event_ledger_entry_digest: String,
    pub series_digest: String,
    pub epoch_registration_digest: String,
    pub input_receipt_digest: String,
    pub input_capsule_digest: String,
    pub context_proof_digest: String,
    pub participant_seal_digests: Vec<String>,
    pub prediction_capsule_digest: String,
    pub prediction_journal_digest: String,
    pub outcome_plan_digest: String,
    pub outcome_receipt_digest: String,
    pub outcome_capsule_digest: String,
    pub opening_authorization_digest: String,
    pub opening_bundle_digest: String,
    pub label_status: MomentumProspectiveLabelStatusV4_4,
    pub participant_evaluation_digests: Vec<String>,
    pub total_event_count_after: usize,
    pub scorable_event_count_after: usize,
    pub winner_selected: bool,
    pub ranking_created: bool,
    pub reward_applied: bool,
    pub penalty_applied: bool,
    pub chair_action_taken: bool,
    pub trading_action_taken: bool,
    pub entry_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveSeriesEligibilityReceiptV4 {
    pub receipt_version: String,
    pub event_one_eligibility_digest: String,
    pub event_two_ledger_entry_digest: String,
    pub participant_roles: Vec<String>,
    pub completed_event_count: usize,
    pub scorable_event_count: usize,
    pub minimum_sample_gate: usize,
    pub status: MomentumProspectiveSeriesEligibilityV4,
    pub integrity_verified: bool,
    pub reward_application_count: usize,
    pub penalty_application_count: usize,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveProspectiveContinuationPauseV2 {
    pub pause_version: String,
    pub policy: LiveProspectiveContinuationPolicyV2,
    pub prior_pause_digest: String,
    pub event_two_ledger_entry_digest: String,
    pub completed_event_count: usize,
    pub scorable_event_count: usize,
    pub eligibility_receipt_digest: String,
    pub epoch_three_registered: bool,
    pub historical_challenger_research_prioritized: bool,
    pub scheduler_count: usize,
    pub automatic_registration_count: usize,
    pub network_authority_count: usize,
    pub pause_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveOutcomeStatusV4 {
    pub series_digest: String,
    pub epoch_number: u64,
    pub event_timestamp_ms: u64,
    pub required_outcome_timestamp_ms: u64,
    pub outcome_finality_boundary_ms: u64,
    pub prediction_capsule_digest: String,
    pub prediction_journal_digest: String,
    pub outcome_plan_digest: String,
    pub provider_id: String,
    pub market: String,
    pub symbol: String,
    pub cadence: String,
    pub request_start_timestamp_ms: u64,
    pub request_end_timestamp_ms: u64,
    pub request_fingerprint: String,
    pub maximum_requests: usize,
    pub maximum_retries: usize,
    pub maximum_concurrency: usize,
    pub prior_attempt_count: usize,
    pub outcome_receipt_digest: Option<String>,
    pub outcome_capsule_digest: Option<String>,
    pub opening_authorization_digest: Option<String>,
    pub opening_bundle_digest: Option<String>,
    pub participant_evaluation_digests: Vec<String>,
    pub label_status: Option<MomentumProspectiveLabelStatusV4_4>,
    pub event_two_ledger_entry_digest: Option<String>,
    pub completed_event_count: usize,
    pub scorable_event_count: usize,
    pub eligibility_status: MomentumProspectiveSeriesEligibilityV4,
    pub eligibility_receipt_digest: Option<String>,
    pub completed_pause_digest: Option<String>,
    pub readiness: MomentumProspectiveOutcomeReadinessV4,
    pub protected_live_artifact_count: usize,
    pub protected_live_aggregate_digest: String,
    pub event_one_chain_digest: String,
    pub event_two_sealed_chain_digest: String,
    pub historical_store_digest: String,
    pub qualified_six_replay_digest: String,
    pub diagnostic_store_digest: String,
    pub active_roster_digest: String,
    pub participant_parameter_digests: Vec<String>,
    pub participant_normalizer_digests: Vec<String>,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub prior_pause_digest: String,
    pub epoch_three_registered: bool,
    pub safety_counters: MomentumProspectiveOutcomeSafetyCountersV4,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub protected_artifacts_unchanged: bool,
    pub active_state_unchanged: bool,
    pub status_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumProspectiveOutcomeReportV4 {
    pub status: MomentumProspectiveOutcomeStatusV4,
    pub registration: MomentumOutcomeAcquisitionRegistrationV4_4,
    pub receipt: Option<MomentumOutcomeAcquisitionReceiptV4_4>,
    pub outcome_capsule: Option<MomentumSealedOutcomeCapsuleV4_4>,
    pub opening_authorization: Option<MomentumOutcomeOpeningAuthorizationV4_4>,
    pub opening_receipt: Option<MomentumOutcomeOpeningReceiptV4_4>,
    pub opening_bundle: Option<MomentumOutcomeOpeningBundleV4_4>,
    pub ledger_entry: Option<MomentumProspectiveSeriesLedgerEntryV4>,
    pub eligibility_receipt: Option<MomentumProspectiveSeriesEligibilityReceiptV4>,
    pub completed_pause: Option<LiveProspectiveContinuationPauseV2>,
}

fn series_ledger_entry_digest(value: &MomentumProspectiveSeriesLedgerEntryV4) -> String {
    canonical_digest(value, |item| item.entry_digest.clear())
}

fn series_eligibility_digest(value: &MomentumProspectiveSeriesEligibilityReceiptV4) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn completed_pause_digest(value: &LiveProspectiveContinuationPauseV2) -> String {
    canonical_digest(value, |item| item.pause_digest.clear())
}

fn prospective_outcome_status_digest(value: &MomentumProspectiveOutcomeStatusV4) -> String {
    canonical_digest(value, |item| item.status_digest.clear())
}

fn parse_series_label_status(value: &str) -> Result<MomentumProspectiveLabelStatusV4_4, String> {
    match value {
        "ScorableBinaryOutcome" => Ok(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome),
        "NeutralOutcomeExcluded" => Ok(MomentumProspectiveLabelStatusV4_4::NeutralOutcomeExcluded),
        "InvalidOutcomeEvidence" => Ok(MomentumProspectiveLabelStatusV4_4::InvalidOutcomeEvidence),
        _ => Err("V4 series outcome label status rejected".to_string()),
    }
}

fn validate_series_ledger_entry(
    value: &MomentumProspectiveSeriesLedgerEntryV4,
) -> Result<(), String> {
    let scorable = value.label_status == MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome;
    if value.ledger_version != SERIES_LEDGER_VERSION
        || [
            &value.previous_event_ledger_entry_digest,
            &value.series_digest,
            &value.epoch_registration_digest,
            &value.input_receipt_digest,
            &value.input_capsule_digest,
            &value.context_proof_digest,
            &value.prediction_capsule_digest,
            &value.prediction_journal_digest,
            &value.outcome_plan_digest,
            &value.outcome_receipt_digest,
            &value.outcome_capsule_digest,
            &value.opening_authorization_digest,
            &value.opening_bundle_digest,
        ]
        .iter()
        .any(|value| value.is_empty())
        || value.participant_seal_digests.len() != 3
        || value
            .participant_seal_digests
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != 3
        || value.participant_evaluation_digests.len() != 3
        || value
            .participant_evaluation_digests
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != 3
        || value.total_event_count_after < 2
        || value.scorable_event_count_after > value.total_event_count_after
        || (scorable && value.scorable_event_count_after == 0)
        || value.winner_selected
        || value.ranking_created
        || value.reward_applied
        || value.penalty_applied
        || value.chair_action_taken
        || value.trading_action_taken
        || value.entry_digest != series_ledger_entry_digest(value)
    {
        return Err("V4 series event-two ledger entry rejected".to_string());
    }
    Ok(())
}

fn validate_series_eligibility(
    value: &MomentumProspectiveSeriesEligibilityReceiptV4,
) -> Result<(), String> {
    let expected = if !value.integrity_verified {
        MomentumProspectiveSeriesEligibilityV4::IntegrityFailure
    } else if value.scorable_event_count < value.minimum_sample_gate {
        MomentumProspectiveSeriesEligibilityV4::IneligibleMinimumSamples
    } else {
        MomentumProspectiveSeriesEligibilityV4::EligibleForShadowRewardAssessment
    };
    let expected_roles = [
        "RawFeatureLogisticV4",
        "RawFeatureInteractionLogisticV4",
        "TrainingPrevalenceConstantV4",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    if value.receipt_version != SERIES_ELIGIBILITY_VERSION
        || value.event_one_eligibility_digest.is_empty()
        || value.event_two_ledger_entry_digest.is_empty()
        || value.participant_roles.len() != 3
        || value
            .participant_roles
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != expected_roles
        || value.completed_event_count < 2
        || value.scorable_event_count > value.completed_event_count
        || value.minimum_sample_gate == 0
        || value.status != expected
        || value.reward_application_count != 0
        || value.penalty_application_count != 0
        || value.receipt_digest != series_eligibility_digest(value)
    {
        return Err("V4 series eligibility receipt rejected".to_string());
    }
    Ok(())
}

fn validate_completed_pause(value: &LiveProspectiveContinuationPauseV2) -> Result<(), String> {
    if value.pause_version != COMPLETED_PAUSE_VERSION
        || value.policy != LiveProspectiveContinuationPolicyV2::PausedAfterCompletedEpochTwo
        || value.prior_pause_digest.is_empty()
        || value.event_two_ledger_entry_digest.is_empty()
        || value.completed_event_count < 2
        || value.scorable_event_count > value.completed_event_count
        || value.eligibility_receipt_digest.is_empty()
        || value.epoch_three_registered
        || !value.historical_challenger_research_prioritized
        || value.scheduler_count != 0
        || value.automatic_registration_count != 0
        || value.network_authority_count != 0
        || value.pause_digest != completed_pause_digest(value)
    {
        return Err("V4 completed continuation pause rejected".to_string());
    }
    Ok(())
}

fn encode_series_ledger_entry(
    value: &MomentumProspectiveSeriesLedgerEntryV4,
) -> Result<Vec<u8>, String> {
    validate_series_ledger_entry(value)?;
    ArtifactBuilderV4_2::new("MomentumProspectiveSeriesLedgerEntryV4")
        .string("ledger_version", &value.ledger_version)
        .string(
            "previous_event_ledger_entry_digest",
            &value.previous_event_ledger_entry_digest,
        )
        .string("series_digest", &value.series_digest)
        .string(
            "epoch_registration_digest",
            &value.epoch_registration_digest,
        )
        .string("input_receipt_digest", &value.input_receipt_digest)
        .string("input_capsule_digest", &value.input_capsule_digest)
        .string("context_proof_digest", &value.context_proof_digest)
        .strings("participant_seal_digests", &value.participant_seal_digests)
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .string(
            "prediction_journal_digest",
            &value.prediction_journal_digest,
        )
        .string("outcome_plan_digest", &value.outcome_plan_digest)
        .string("outcome_receipt_digest", &value.outcome_receipt_digest)
        .string("outcome_capsule_digest", &value.outcome_capsule_digest)
        .string(
            "opening_authorization_digest",
            &value.opening_authorization_digest,
        )
        .string("opening_bundle_digest", &value.opening_bundle_digest)
        .string("label_status", format!("{:?}", value.label_status))
        .strings(
            "participant_evaluation_digests",
            &value.participant_evaluation_digests,
        )
        .unsigned(
            "total_event_count_after",
            as_u64(value.total_event_count_after)?,
        )
        .unsigned(
            "scorable_event_count_after",
            as_u64(value.scorable_event_count_after)?,
        )
        .boolean("winner_selected", value.winner_selected)
        .boolean("ranking_created", value.ranking_created)
        .boolean("reward_applied", value.reward_applied)
        .boolean("penalty_applied", value.penalty_applied)
        .boolean("chair_action_taken", value.chair_action_taken)
        .boolean("trading_action_taken", value.trading_action_taken)
        .string("entry_digest", &value.entry_digest)
        .encode()
}

fn decode_series_ledger_entry(
    bytes: &[u8],
) -> Result<MomentumProspectiveSeriesLedgerEntryV4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumProspectiveSeriesLedgerEntryV4")?;
    let value = MomentumProspectiveSeriesLedgerEntryV4 {
        ledger_version: fields.string("ledger_version")?,
        previous_event_ledger_entry_digest: fields.string("previous_event_ledger_entry_digest")?,
        series_digest: fields.string("series_digest")?,
        epoch_registration_digest: fields.string("epoch_registration_digest")?,
        input_receipt_digest: fields.string("input_receipt_digest")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        context_proof_digest: fields.string("context_proof_digest")?,
        participant_seal_digests: fields.strings("participant_seal_digests")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        prediction_journal_digest: fields.string("prediction_journal_digest")?,
        outcome_plan_digest: fields.string("outcome_plan_digest")?,
        outcome_receipt_digest: fields.string("outcome_receipt_digest")?,
        outcome_capsule_digest: fields.string("outcome_capsule_digest")?,
        opening_authorization_digest: fields.string("opening_authorization_digest")?,
        opening_bundle_digest: fields.string("opening_bundle_digest")?,
        label_status: parse_series_label_status(&fields.string("label_status")?)?,
        participant_evaluation_digests: fields.strings("participant_evaluation_digests")?,
        total_event_count_after: as_usize(fields.unsigned("total_event_count_after")?)?,
        scorable_event_count_after: as_usize(fields.unsigned("scorable_event_count_after")?)?,
        winner_selected: fields.boolean("winner_selected")?,
        ranking_created: fields.boolean("ranking_created")?,
        reward_applied: fields.boolean("reward_applied")?,
        penalty_applied: fields.boolean("penalty_applied")?,
        chair_action_taken: fields.boolean("chair_action_taken")?,
        trading_action_taken: fields.boolean("trading_action_taken")?,
        entry_digest: fields.string("entry_digest")?,
    };
    fields.finish()?;
    validate_series_ledger_entry(&value)?;
    Ok(value)
}

fn encode_series_eligibility(
    value: &MomentumProspectiveSeriesEligibilityReceiptV4,
) -> Result<Vec<u8>, String> {
    validate_series_eligibility(value)?;
    ArtifactBuilderV4_2::new("MomentumProspectiveSeriesEligibilityReceiptV4")
        .string("receipt_version", &value.receipt_version)
        .string(
            "event_one_eligibility_digest",
            &value.event_one_eligibility_digest,
        )
        .string(
            "event_two_ledger_entry_digest",
            &value.event_two_ledger_entry_digest,
        )
        .strings("participant_roles", &value.participant_roles)
        .unsigned(
            "completed_event_count",
            as_u64(value.completed_event_count)?,
        )
        .unsigned("scorable_event_count", as_u64(value.scorable_event_count)?)
        .unsigned("minimum_sample_gate", as_u64(value.minimum_sample_gate)?)
        .string("status", format!("{:?}", value.status))
        .boolean("integrity_verified", value.integrity_verified)
        .unsigned(
            "reward_application_count",
            as_u64(value.reward_application_count)?,
        )
        .unsigned(
            "penalty_application_count",
            as_u64(value.penalty_application_count)?,
        )
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn parse_series_eligibility(value: &str) -> Result<MomentumProspectiveSeriesEligibilityV4, String> {
    match value {
        "IneligibleMinimumSamples" => {
            Ok(MomentumProspectiveSeriesEligibilityV4::IneligibleMinimumSamples)
        }
        "EligibleForShadowRewardAssessment" => {
            Ok(MomentumProspectiveSeriesEligibilityV4::EligibleForShadowRewardAssessment)
        }
        "IntegrityFailure" => Ok(MomentumProspectiveSeriesEligibilityV4::IntegrityFailure),
        _ => Err("V4 series eligibility status rejected".to_string()),
    }
}

fn decode_series_eligibility(
    bytes: &[u8],
) -> Result<MomentumProspectiveSeriesEligibilityReceiptV4, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumProspectiveSeriesEligibilityReceiptV4")?;
    let value = MomentumProspectiveSeriesEligibilityReceiptV4 {
        receipt_version: fields.string("receipt_version")?,
        event_one_eligibility_digest: fields.string("event_one_eligibility_digest")?,
        event_two_ledger_entry_digest: fields.string("event_two_ledger_entry_digest")?,
        participant_roles: fields.strings("participant_roles")?,
        completed_event_count: as_usize(fields.unsigned("completed_event_count")?)?,
        scorable_event_count: as_usize(fields.unsigned("scorable_event_count")?)?,
        minimum_sample_gate: as_usize(fields.unsigned("minimum_sample_gate")?)?,
        status: parse_series_eligibility(&fields.string("status")?)?,
        integrity_verified: fields.boolean("integrity_verified")?,
        reward_application_count: as_usize(fields.unsigned("reward_application_count")?)?,
        penalty_application_count: as_usize(fields.unsigned("penalty_application_count")?)?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_series_eligibility(&value)?;
    Ok(value)
}

fn encode_completed_pause(value: &LiveProspectiveContinuationPauseV2) -> Result<Vec<u8>, String> {
    validate_completed_pause(value)?;
    ArtifactBuilderV4_2::new("LiveProspectiveContinuationPauseV2")
        .string("pause_version", &value.pause_version)
        .string("policy", "PausedAfterCompletedEpochTwo")
        .string("prior_pause_digest", &value.prior_pause_digest)
        .string(
            "event_two_ledger_entry_digest",
            &value.event_two_ledger_entry_digest,
        )
        .unsigned(
            "completed_event_count",
            as_u64(value.completed_event_count)?,
        )
        .unsigned("scorable_event_count", as_u64(value.scorable_event_count)?)
        .string(
            "eligibility_receipt_digest",
            &value.eligibility_receipt_digest,
        )
        .boolean("epoch_three_registered", value.epoch_three_registered)
        .boolean(
            "historical_challenger_research_prioritized",
            value.historical_challenger_research_prioritized,
        )
        .unsigned("scheduler_count", as_u64(value.scheduler_count)?)
        .unsigned(
            "automatic_registration_count",
            as_u64(value.automatic_registration_count)?,
        )
        .unsigned(
            "network_authority_count",
            as_u64(value.network_authority_count)?,
        )
        .string("pause_digest", &value.pause_digest)
        .encode()
}

fn decode_completed_pause(bytes: &[u8]) -> Result<LiveProspectiveContinuationPauseV2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "LiveProspectiveContinuationPauseV2")?;
    if fields.string("policy")? != "PausedAfterCompletedEpochTwo" {
        return Err("V4 completed continuation policy rejected".to_string());
    }
    let value = LiveProspectiveContinuationPauseV2 {
        pause_version: fields.string("pause_version")?,
        policy: LiveProspectiveContinuationPolicyV2::PausedAfterCompletedEpochTwo,
        prior_pause_digest: fields.string("prior_pause_digest")?,
        event_two_ledger_entry_digest: fields.string("event_two_ledger_entry_digest")?,
        completed_event_count: as_usize(fields.unsigned("completed_event_count")?)?,
        scorable_event_count: as_usize(fields.unsigned("scorable_event_count")?)?,
        eligibility_receipt_digest: fields.string("eligibility_receipt_digest")?,
        epoch_three_registered: fields.boolean("epoch_three_registered")?,
        historical_challenger_research_prioritized: fields
            .boolean("historical_challenger_research_prioritized")?,
        scheduler_count: as_usize(fields.unsigned("scheduler_count")?)?,
        automatic_registration_count: as_usize(fields.unsigned("automatic_registration_count")?)?,
        network_authority_count: as_usize(fields.unsigned("network_authority_count")?)?,
        pause_digest: fields.string("pause_digest")?,
    };
    fields.finish()?;
    validate_completed_pause(&value)?;
    Ok(value)
}

fn series_outcome_root(root: &Path) -> PathBuf {
    series_root(root).join(SERIES_OUTCOME_DIRECTORY)
}

pub fn prospective_outcome_stage_started_v4(root: &Path) -> bool {
    series_outcome_root(root).exists()
}

fn collect_protected_live_files(
    root: &Path,
    current: &Path,
    values: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), String> {
    let outcome_root = series_outcome_root(root);
    let pause_root = series_root(root).join(COMPLETED_PAUSE_DIRECTORY);
    if current == outcome_root || current == pause_root {
        return Ok(());
    }
    if current.is_dir() {
        let mut paths = fs::read_dir(current)
            .map_err(|_| "V4 protected live directory read failed".to_string())?
            .map(|entry| {
                entry
                    .map(|entry| entry.path())
                    .map_err(|_| "V4 protected live entry read failed".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        paths.sort();
        for path in paths {
            collect_protected_live_files(root, &path, values)?;
        }
    } else if current.is_file() {
        values.push((
            current
                .strip_prefix(root)
                .map_err(|_| "V4 protected live path rejected".to_string())?
                .to_path_buf(),
            fs::read(current).map_err(|_| "V4 protected live artifact read failed".to_string())?,
        ));
    }
    Ok(())
}

fn protected_live_identity(root: &Path) -> Result<(usize, String), String> {
    let mut values = Vec::new();
    collect_protected_live_files(root, root, &mut values)?;
    values.sort_by(|left, right| left.0.cmp(&right.0));
    Ok((
        values.len(),
        stable_hash_string(&format!("momentum-v4-protected-live-v2:{values:?}")),
    ))
}

fn directory_manifest_identity(root: &Path, name: &str) -> Result<String, String> {
    if !root.is_dir() {
        return Err(format!("{name} directory unavailable"));
    }
    let mut stack = vec![root.to_path_buf()];
    let mut entries = Vec::new();
    while let Some(current) = stack.pop() {
        let mut paths = fs::read_dir(&current)
            .map_err(|_| format!("{name} directory read failed"))?
            .map(|entry| {
                entry
                    .map(|entry| entry.path())
                    .map_err(|_| format!("{name} entry read failed"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        paths.sort_by(|left, right| right.cmp(left));
        for path in paths {
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                let metadata =
                    fs::metadata(&path).map_err(|_| format!("{name} metadata read failed"))?;
                let bytes = fs::read(&path).map_err(|_| format!("{name} artifact read failed"))?;
                entries.push((
                    path.strip_prefix(root)
                        .map_err(|_| format!("{name} path rejected"))?
                        .to_path_buf(),
                    metadata.len(),
                    stable_hash_string(&format!("{name}:artifact-bytes:{bytes:?}")),
                ));
            }
        }
    }
    entries.sort();
    Ok(stable_hash_string(&format!("{name}:{entries:?}")))
}

fn historical_identity_roots(root: &Path) -> Result<(String, String, String), String> {
    let state_root = root
        .parent()
        .ok_or_else(|| "V4 historical state root unavailable".to_string())?;
    let historical_root = state_root.join("historical_replay");
    Ok((
        directory_manifest_identity(&historical_root, "historical-store")?,
        directory_manifest_identity(
            &historical_root.join("momentum_qualified_six").join("v1"),
            "qualified-six-replay",
        )?,
        directory_manifest_identity(
            &historical_root
                .join("momentum_qualified_six_diagnostics")
                .join("v1"),
            "qualified-six-diagnostics",
        )?,
    ))
}

fn prior_live_pause_digest(root: &Path) -> Result<String, String> {
    let pause_root = root
        .parent()
        .ok_or_else(|| "V4 historical state root unavailable".to_string())?
        .join("historical_replay")
        .join("momentum_multitimeframe")
        .join("v1")
        .join("live_continuation_pause");
    let mut paths = fs::read_dir(&pause_root)
        .map_err(|_| "V4 prior live pause directory unavailable".to_string())?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|_| "V4 prior live pause entry read failed".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| path.extension().is_some_and(|extension| extension == "pb"));
    paths.sort();
    if paths.len() != 1 {
        return Err("V4 prior live pause identity rejected".to_string());
    }
    let bytes = fs::read(&paths[0]).map_err(|_| "V4 prior live pause read failed".to_string())?;
    let mut fields = ArtifactReaderV4_2::decode(&bytes, "LiveContinuationPauseV1")?;
    if fields.string("pause_version")? != "live-prospective-continuation-pause-v1"
        || fields.string("policy")? != "PausedAfterSealedEpochTwo"
        || fields.string("series_digest")?.is_empty()
        || fields.string("epoch_registration_digest")?.is_empty()
        || fields.string("input_receipt_digest")?.is_empty()
        || fields.string("input_capsule_digest")?.is_empty()
        || fields.string("context_proof_digest")?.is_empty()
        || fields.string("prediction_capsule_digest")?.is_empty()
        || fields.string("prediction_journal_digest")?.is_empty()
        || fields.string("outcome_plan_digest")?.is_empty()
        || fields.unsigned("protected_first_event_input_boundary_ms")? == 0
        || fields.unsigned("completed_event_count")? != 1
        || fields.unsigned("scorable_event_count")? != 1
        || fields.unsigned("prediction_seal_count")? != 3
        || fields.unsigned("input_attempts")? != 1
        || fields.unsigned("input_retries")? != 0
        || fields.unsigned("outcome_requests")? != 0
        || fields.unsigned("outcome_openings")? != 0
        || fields.boolean("epoch_three_registered")?
    {
        return Err("V4 prior live pause semantics rejected".to_string());
    }
    let digest = fields.string("pause_digest")?;
    fields.finish()?;
    let file_digest = paths[0]
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "V4 prior live pause filename rejected".to_string())?;
    if digest.is_empty() || digest != file_digest {
        return Err("V4 prior live pause digest rejected".to_string());
    }
    Ok(digest)
}

fn derive_series_outcome_registration(
    live: &MomentumProspectiveSeriesReportV4,
    config: &UpbitHistoricalPilotConfigV0,
) -> Result<MomentumOutcomeAcquisitionRegistrationV4_4, String> {
    let receipt = live
        .input_receipt
        .as_ref()
        .ok_or_else(|| "V4 event-two input receipt unavailable".to_string())?;
    let capsule = live
        .input_capsule
        .as_ref()
        .ok_or_else(|| "V4 event-two input capsule unavailable".to_string())?;
    let context = live
        .context_use_proof
        .as_ref()
        .ok_or_else(|| "V4 event-two context proof unavailable".to_string())?;
    let prediction = live
        .prediction_capsule
        .as_ref()
        .ok_or_else(|| "V4 event-two prediction capsule unavailable".to_string())?;
    let journal = live
        .journal_entry
        .as_ref()
        .ok_or_else(|| "V4 event-two prediction journal unavailable".to_string())?;
    let plan = live
        .outcome_plan
        .as_ref()
        .ok_or_else(|| "V4 event-two outcome plan unavailable".to_string())?;
    let mut value = MomentumOutcomeAcquisitionRegistrationV4_4 {
        registration_version: REGISTRATION_VERSION.to_string(),
        agent_id: live.series.agent_id.clone(),
        lifecycle_digest: live.series.series_digest.clone(),
        evaluation_registration_digest: live.epoch_registration.registration_digest.clone(),
        roster_digest: live.series.frozen_roster_digest.clone(),
        input_receipt_digest: receipt.receipt_digest.clone(),
        input_capsule_digest: capsule.capsule_digest.clone(),
        context_usage_ledger_digest: context.proof_digest.clone(),
        prediction_capsule_digest: prediction.capsule_digest.clone(),
        prediction_journal_digest: journal.entry_digest.clone(),
        outcome_plan_digest: plan.plan_digest.clone(),
        event_timestamp_ms: plan.event_timestamp_ms,
        required_outcome_timestamp_ms: plan.required_outcome_timestamp_ms.clone(),
        outcome_finality_boundary_ms: plan.outcome_finality_boundary_ms,
        provider_id: live.epoch_registration.provider_id.clone(),
        market: live.epoch_registration.market.clone(),
        symbol: live.epoch_registration.symbol.clone(),
        cadence: live.epoch_registration.cadence.clone(),
        exact_expected_timestamp_ms: plan.required_outcome_timestamp_ms.clone(),
        expected_row_count: plan.required_outcome_timestamp_ms.len(),
        request_to_timestamp_ms: plan.outcome_finality_boundary_ms,
        maximum_requests: plan.maximum_outcome_requests,
        maximum_concurrency: 1,
        maximum_retries: plan.maximum_outcome_retries,
        maximum_response_bytes: config.maximum_response_bytes,
        credential_free_required: true,
        read_only_required: true,
        labels_must_remain_unopened: plan.labels_hidden_until_opening,
        metric_computation_forbidden: true,
        winner_selection_forbidden: true,
        reward_application_forbidden: true,
        registration_digest: String::new(),
    };
    value.registration_digest = registration_digest(&value);
    validate_series_outcome_registration(&value)?;
    validate_series_outcome_registration_binding(&value, live, config)?;
    Ok(value)
}

fn validate_series_outcome_registration_binding(
    value: &MomentumOutcomeAcquisitionRegistrationV4_4,
    live: &MomentumProspectiveSeriesReportV4,
    config: &UpbitHistoricalPilotConfigV0,
) -> Result<(), String> {
    validate_series_outcome_registration(value)?;
    let plan = live
        .outcome_plan
        .as_ref()
        .ok_or_else(|| "V4 event-two outcome plan unavailable".to_string())?;
    let prediction = live
        .prediction_capsule
        .as_ref()
        .ok_or_else(|| "V4 event-two prediction capsule unavailable".to_string())?;
    let journal = live
        .journal_entry
        .as_ref()
        .ok_or_else(|| "V4 event-two prediction journal unavailable".to_string())?;
    let contract = upbit_learning_evidence_provider_contract_v1(config)?;
    if live.status.readiness != MomentumProspectiveEpochReadinessV4::PredictionAlreadySealed
        || live.epoch_registration.epoch_number != 2
        || prediction.participant_seal_digests.len() != 3
        || prediction.participant_prediction_digests.len() != 3
        || prediction.outcome_accessed
        || prediction.metrics_computed
        || !journal.outcome_stage_locked
        || plan.outcome_acquisition_count != 0
        || plan.outcome_opening_count != 0
        || value.lifecycle_digest != live.series.series_digest
        || value.evaluation_registration_digest != live.epoch_registration.registration_digest
        || value.roster_digest != live.series.frozen_roster_digest
        || value.prediction_capsule_digest != prediction.capsule_digest
        || value.prediction_journal_digest != journal.entry_digest
        || value.outcome_plan_digest != plan.plan_digest
        || value.event_timestamp_ms != live.epoch_registration.event_timestamp_ms
        || value.required_outcome_timestamp_ms != [live.epoch_registration.outcome_timestamp_ms]
        || value.outcome_finality_boundary_ms
            != live.epoch_registration.outcome_finality_boundary_ms
        || value.maximum_requests != 1
        || value.maximum_retries != 0
        || value.maximum_concurrency != 1
        || value.provider_id != config.provider_id
        || value.symbol != config.symbol
        || contract.provider_id != value.provider_id
        || contract.market_scope != AcquisitionMarketScope::BtcCrypto
        || contract.dataset_kind != DatasetKind::DailyOhlcv
        || contract.symbols != [value.symbol.clone()]
        || contract.cadence != value.cadence
        || !contract.credential_free
        || !contract.read_only
        || !contract.approved_for_network
        || !contract.all_rows_finalized
        || !contract.enabled
    {
        return Err("V4 event-two outcome registration binding rejected".to_string());
    }
    Ok(())
}

#[derive(Default)]
struct SeriesOutcomeArtifacts {
    registration: Option<MomentumOutcomeAcquisitionRegistrationV4_4>,
    receipt: Option<MomentumOutcomeAcquisitionReceiptV4_4>,
    proof: Option<MomentumOutcomeRowIdentityProofV4_4>,
    capsule: Option<MomentumSealedOutcomeCapsuleV4_4>,
    authorization: Option<MomentumOutcomeOpeningAuthorizationV4_4>,
    opening_receipt: Option<MomentumOutcomeOpeningReceiptV4_4>,
    opening_bundle: Option<MomentumOutcomeOpeningBundleV4_4>,
    evaluations: Vec<MomentumParticipantProspectiveEvaluationV4_4>,
    ledger: Option<MomentumProspectiveSeriesLedgerEntryV4>,
    eligibility: Option<MomentumProspectiveSeriesEligibilityReceiptV4>,
    pause: Option<LiveProspectiveContinuationPauseV2>,
}

fn read_all_series_evaluations(
    root: &Path,
) -> Result<Vec<MomentumParticipantProspectiveEvaluationV4_4>, String> {
    let directory = root.join("participant_evaluations");
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(directory)
        .map_err(|_| "V4 event-two evaluation directory read failed".to_string())?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|_| "V4 event-two evaluation entry read failed".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| path.extension().is_some_and(|extension| extension == "pb"));
    paths.sort();
    paths
        .iter()
        .map(|path| {
            fs::read(path)
                .map_err(|_| "V4 event-two evaluation read failed".to_string())
                .and_then(|bytes| decode_series_evaluation(&bytes))
        })
        .collect()
}

fn reopen_series_outcome_artifacts(root: &Path) -> Result<SeriesOutcomeArtifacts, String> {
    let outcome_root = series_outcome_root(root);
    Ok(SeriesOutcomeArtifacts {
        registration: read_single(
            &outcome_root.join("outcome_registrations"),
            decode_series_outcome_registration,
        )?,
        receipt: read_single(
            &outcome_root.join("outcome_receipts"),
            decode_series_outcome_receipt,
        )?,
        proof: read_single(
            &outcome_root.join("outcome_row_proofs"),
            decode_series_outcome_proof,
        )?,
        capsule: read_single(
            &outcome_root.join("outcome_capsules"),
            decode_series_outcome_capsule,
        )?,
        authorization: read_single(
            &outcome_root.join("opening_authorizations"),
            decode_series_opening_authorization,
        )?,
        opening_receipt: read_single(
            &outcome_root.join("opening_receipts"),
            decode_series_opening_receipt,
        )?,
        opening_bundle: read_single(
            &outcome_root.join("opening_bundles"),
            decode_series_opening_bundle,
        )?,
        evaluations: read_all_series_evaluations(&outcome_root)?,
        ledger: read_single(
            &outcome_root.join("evaluation_ledger"),
            decode_series_ledger_entry,
        )?,
        eligibility: read_single(
            &outcome_root.join("eligibility_receipts"),
            decode_series_eligibility,
        )?,
        pause: read_single(
            &series_root(root).join(COMPLETED_PAUSE_DIRECTORY),
            decode_completed_pause,
        )?,
    })
}

fn validate_completed_series_outcome(
    artifacts: &SeriesOutcomeArtifacts,
    live: &MomentumProspectiveSeriesReportV4,
    derived_registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    prior_pause_digest: &str,
) -> Result<(), String> {
    let registration = artifacts
        .registration
        .as_ref()
        .ok_or_else(|| "V4 completed outcome registration unavailable".to_string())?;
    let receipt = artifacts
        .receipt
        .as_ref()
        .ok_or_else(|| "V4 completed outcome receipt unavailable".to_string())?;
    let proof = artifacts
        .proof
        .as_ref()
        .ok_or_else(|| "V4 completed outcome proof unavailable".to_string())?;
    let capsule = artifacts
        .capsule
        .as_ref()
        .ok_or_else(|| "V4 completed outcome capsule unavailable".to_string())?;
    let authorization = artifacts
        .authorization
        .as_ref()
        .ok_or_else(|| "V4 completed opening authorization unavailable".to_string())?;
    let opening_receipt = artifacts
        .opening_receipt
        .as_ref()
        .ok_or_else(|| "V4 completed opening receipt unavailable".to_string())?;
    let bundle = artifacts
        .opening_bundle
        .as_ref()
        .ok_or_else(|| "V4 completed opening bundle unavailable".to_string())?;
    let ledger = artifacts
        .ledger
        .as_ref()
        .ok_or_else(|| "V4 completed event-two ledger unavailable".to_string())?;
    let eligibility = artifacts
        .eligibility
        .as_ref()
        .ok_or_else(|| "V4 completed eligibility unavailable".to_string())?;
    let pause = artifacts
        .pause
        .as_ref()
        .ok_or_else(|| "V4 completed pause unavailable".to_string())?;
    let prediction = live
        .prediction_capsule
        .as_ref()
        .ok_or_else(|| "V4 completed prediction capsule unavailable".to_string())?;
    let journal = live
        .journal_entry
        .as_ref()
        .ok_or_else(|| "V4 completed prediction journal unavailable".to_string())?;
    let plan = live
        .outcome_plan
        .as_ref()
        .ok_or_else(|| "V4 completed outcome plan unavailable".to_string())?;
    let evaluation_digests = artifacts
        .evaluations
        .iter()
        .map(|value| value.evaluation_digest.clone())
        .collect::<BTreeSet<_>>();
    let bundle_digests = bundle
        .participant_evaluations
        .iter()
        .map(|value| value.evaluation_digest.clone())
        .collect::<BTreeSet<_>>();
    let ledger_digests = ledger
        .participant_evaluation_digests
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if registration != derived_registration
        || receipt.status != MomentumOutcomeAcquisitionStatusV4_4::EvidenceAcquired
        || receipt.registration_digest != registration.registration_digest
        || receipt.outcome_capsule_digest.as_deref() != Some(capsule.capsule_digest.as_str())
        || proof.registration_digest != registration.registration_digest
        || proof.outcome_row_digest != capsule.outcome_row_digest
        || capsule.receipt_digest != receipt.receipt_digest
        || capsule.prediction_capsule_digest != prediction.capsule_digest
        || authorization.outcome_registration_digest != registration.registration_digest
        || authorization.outcome_receipt_digest != receipt.receipt_digest
        || authorization.outcome_capsule_digest != capsule.capsule_digest
        || authorization.prediction_capsule_digest != prediction.capsule_digest
        || authorization.prediction_journal_digest != journal.entry_digest
        || opening_receipt.authorization_digest != authorization.authorization_digest
        || opening_receipt.opening_bundle_digest != bundle.bundle_digest
        || bundle.authorization_digest != authorization.authorization_digest
        || bundle.outcome_capsule_digest != capsule.capsule_digest
        || bundle.prediction_capsule_digest != prediction.capsule_digest
        || artifacts.evaluations.len() != 3
        || evaluation_digests != bundle_digests
        || ledger.series_digest != live.series.series_digest
        || ledger.epoch_registration_digest != live.epoch_registration.registration_digest
        || ledger.prediction_capsule_digest != prediction.capsule_digest
        || ledger.prediction_journal_digest != journal.entry_digest
        || ledger.outcome_plan_digest != plan.plan_digest
        || ledger.outcome_receipt_digest != receipt.receipt_digest
        || ledger.outcome_capsule_digest != capsule.capsule_digest
        || ledger.opening_authorization_digest != authorization.authorization_digest
        || ledger.opening_bundle_digest != bundle.bundle_digest
        || ledger_digests != bundle_digests
        || eligibility.event_two_ledger_entry_digest != ledger.entry_digest
        || eligibility.completed_event_count != ledger.total_event_count_after
        || eligibility.scorable_event_count != ledger.scorable_event_count_after
        || pause.prior_pause_digest != prior_pause_digest
        || pause.event_two_ledger_entry_digest != ledger.entry_digest
        || pause.eligibility_receipt_digest != eligibility.receipt_digest
        || pause.completed_event_count != ledger.total_event_count_after
        || pause.scorable_event_count != ledger.scorable_event_count_after
    {
        return Err("V4 completed event-two outcome chain rejected".to_string());
    }
    Ok(())
}

fn validate_series_acquisition_binding(
    artifacts: &SeriesOutcomeArtifacts,
    live: &MomentumProspectiveSeriesReportV4,
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
) -> Result<(), String> {
    let persisted_registration = artifacts
        .registration
        .as_ref()
        .ok_or_else(|| "V4 event-two persisted registration unavailable".to_string())?;
    let receipt = artifacts
        .receipt
        .as_ref()
        .ok_or_else(|| "V4 event-two successful receipt unavailable".to_string())?;
    let proof = artifacts
        .proof
        .as_ref()
        .ok_or_else(|| "V4 event-two outcome proof unavailable".to_string())?;
    let capsule = artifacts
        .capsule
        .as_ref()
        .ok_or_else(|| "V4 event-two outcome capsule unavailable".to_string())?;
    let prediction = live
        .prediction_capsule
        .as_ref()
        .ok_or_else(|| "V4 event-two prediction capsule unavailable".to_string())?;
    let plan = live
        .outcome_plan
        .as_ref()
        .ok_or_else(|| "V4 event-two outcome plan unavailable".to_string())?;
    if persisted_registration != registration
        || receipt.status != MomentumOutcomeAcquisitionStatusV4_4::EvidenceAcquired
        || receipt.registration_digest != registration.registration_digest
        || receipt.prediction_capsule_digest != prediction.capsule_digest
        || receipt.outcome_plan_digest != plan.plan_digest
        || receipt.outcome_capsule_digest.as_deref() != Some(capsule.capsule_digest.as_str())
        || proof.registration_digest != registration.registration_digest
        || proof.prediction_capsule_digest != prediction.capsule_digest
        || proof.input_capsule_digest != registration.input_capsule_digest
        || proof.event_timestamp_ms != registration.event_timestamp_ms
        || proof.outcome_timestamp_ms != registration.exact_expected_timestamp_ms[0]
        || capsule.registration_digest != registration.registration_digest
        || capsule.receipt_digest != receipt.receipt_digest
        || capsule.prediction_capsule_digest != prediction.capsule_digest
        || capsule.event_timestamp_ms != registration.event_timestamp_ms
        || capsule.outcome_timestamp_ms != registration.exact_expected_timestamp_ms[0]
        || capsule.outcome_row_digest != proof.outcome_row_digest
    {
        return Err("V4 event-two acquisition chain rejected".to_string());
    }
    Ok(())
}

fn validate_series_opening_bundle_binding(
    live: &MomentumProspectiveSeriesReportV4,
    authorization: &MomentumOutcomeOpeningAuthorizationV4_4,
    capsule: &MomentumSealedOutcomeCapsuleV4_4,
    bundle: &MomentumOutcomeOpeningBundleV4_4,
) -> Result<(), String> {
    validate_opening_authorization_shape(authorization)?;
    validate_opening_bundle_shape(bundle)?;
    let prediction = live
        .prediction_capsule
        .as_ref()
        .ok_or_else(|| "V4 event-two prediction capsule unavailable".to_string())?;
    let expected_seals = authorization
        .participant_seal_digests
        .iter()
        .zip(&authorization.participant_prediction_digests)
        .map(|(seal, prediction)| (seal.clone(), prediction.clone()))
        .collect::<BTreeSet<_>>();
    let evaluation_bindings = bundle
        .participant_evaluations
        .iter()
        .map(|evaluation| {
            (
                evaluation.participant_seal_digest.clone(),
                evaluation.prediction_digest.clone(),
            )
        })
        .collect::<BTreeSet<_>>();
    if bundle.authorization_digest != authorization.authorization_digest
        || bundle.outcome_capsule_digest != capsule.capsule_digest
        || bundle.prediction_capsule_digest != prediction.capsule_digest
        || expected_seals != evaluation_bindings
        || bundle
            .participant_evaluations
            .iter()
            .any(|evaluation| evaluation.label_status != bundle.label_status)
    {
        return Err("V4 event-two opening bundle binding rejected".to_string());
    }
    Ok(())
}

fn series_outcome_readiness(
    observed_timestamp_ms: u64,
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    artifacts: &SeriesOutcomeArtifacts,
    live: &MomentumProspectiveSeriesReportV4,
    prior_pause_digest: &str,
) -> MomentumProspectiveOutcomeReadinessV4 {
    if artifacts.ledger.is_some() {
        return if validate_completed_series_outcome(
            artifacts,
            live,
            registration,
            prior_pause_digest,
        )
        .is_ok()
        {
            MomentumProspectiveOutcomeReadinessV4::OutcomeAlreadyOpened
        } else {
            MomentumProspectiveOutcomeReadinessV4::IntegrityFailure
        };
    }
    if let Some(receipt) = artifacts
        .receipt
        .as_ref()
        .filter(|receipt| receipt.status != MomentumOutcomeAcquisitionStatusV4_4::EvidenceAcquired)
    {
        let no_later_artifacts = artifacts.proof.is_none()
            && artifacts.capsule.is_none()
            && artifacts.authorization.is_none()
            && artifacts.opening_receipt.is_none()
            && artifacts.opening_bundle.is_none()
            && artifacts.evaluations.is_empty()
            && artifacts.eligibility.is_none()
            && artifacts.pause.is_none();
        let terminal_binding_valid = artifacts.registration.as_ref() == Some(registration)
            && receipt.registration_digest == registration.registration_digest
            && receipt.prediction_capsule_digest == registration.prediction_capsule_digest
            && receipt.outcome_plan_digest == registration.outcome_plan_digest;
        return if no_later_artifacts && terminal_binding_valid {
            MomentumProspectiveOutcomeReadinessV4::PriorOutcomeAttemptTerminal
        } else {
            MomentumProspectiveOutcomeReadinessV4::IntegrityFailure
        };
    }
    if artifacts.receipt.as_ref().is_some_and(|receipt| {
        receipt.status == MomentumOutcomeAcquisitionStatusV4_4::EvidenceAcquired
    }) && artifacts.capsule.is_some()
    {
        let acquisition_valid =
            validate_series_acquisition_binding(artifacts, live, registration).is_ok();
        let opening_shapes_valid = artifacts
            .authorization
            .as_ref()
            .is_none_or(|value| validate_opening_authorization_shape(value).is_ok())
            && artifacts
                .opening_bundle
                .as_ref()
                .is_none_or(|value| validate_opening_bundle_shape(value).is_ok())
            && artifacts
                .opening_receipt
                .as_ref()
                .is_none_or(|value| validate_opening_receipt_shape(value).is_ok())
            && artifacts
                .evaluations
                .iter()
                .all(|value| validate_evaluation_shape(value).is_ok())
            && artifacts
                .eligibility
                .as_ref()
                .is_none_or(|value| validate_series_eligibility(value).is_ok())
            && artifacts
                .pause
                .as_ref()
                .is_none_or(|value| validate_completed_pause(value).is_ok());
        let prefix_valid = (artifacts.authorization.is_some()
            || (artifacts.opening_receipt.is_none()
                && artifacts.opening_bundle.is_none()
                && artifacts.evaluations.is_empty()
                && artifacts.eligibility.is_none()
                && artifacts.pause.is_none()))
            && (artifacts.opening_bundle.is_some()
                || (artifacts.opening_receipt.is_none()
                    && artifacts.eligibility.is_none()
                    && artifacts.pause.is_none()))
            && (artifacts.opening_receipt.is_some()
                || (artifacts.eligibility.is_none() && artifacts.pause.is_none()))
            && (artifacts.eligibility.is_some() || artifacts.pause.is_none());
        let opening_bindings_valid = artifacts.opening_bundle.as_ref().is_none_or(|bundle| {
            let Some(authorization) = artifacts.authorization.as_ref() else {
                return false;
            };
            let Some(capsule) = artifacts.capsule.as_ref() else {
                return false;
            };
            let bundle_evaluation_digests = bundle
                .participant_evaluations
                .iter()
                .map(|evaluation| evaluation.evaluation_digest.clone())
                .collect::<BTreeSet<_>>();
            validate_series_opening_bundle_binding(live, authorization, capsule, bundle).is_ok()
                && artifacts.evaluations.iter().all(|evaluation| {
                    bundle_evaluation_digests.contains(&evaluation.evaluation_digest)
                })
                && artifacts.opening_receipt.as_ref().is_none_or(|receipt| {
                    receipt.authorization_digest == authorization.authorization_digest
                        && receipt.opening_bundle_digest == bundle.bundle_digest
                })
        });
        let completion_prefix_bindings_valid = artifacts.pause.as_ref().is_none_or(|pause| {
            artifacts.eligibility.as_ref().is_some_and(|eligibility| {
                pause.eligibility_receipt_digest == eligibility.receipt_digest
                    && pause.completed_event_count == eligibility.completed_event_count
                    && pause.scorable_event_count == eligibility.scorable_event_count
            })
        });
        return if acquisition_valid
            && opening_shapes_valid
            && prefix_valid
            && opening_bindings_valid
            && completion_prefix_bindings_valid
        {
            MomentumProspectiveOutcomeReadinessV4::ReadyForOutcomeOpening
        } else {
            MomentumProspectiveOutcomeReadinessV4::IntegrityFailure
        };
    }
    let partial_acquisition = artifacts.receipt.is_some()
        || artifacts.proof.is_some()
        || artifacts.capsule.is_some()
        || artifacts.authorization.is_some()
        || artifacts.opening_receipt.is_some()
        || artifacts.opening_bundle.is_some()
        || !artifacts.evaluations.is_empty()
        || artifacts.eligibility.is_some()
        || artifacts.pause.is_some();
    if partial_acquisition {
        return MomentumProspectiveOutcomeReadinessV4::IntegrityFailure;
    }
    if observed_timestamp_ms < registration.outcome_finality_boundary_ms {
        MomentumProspectiveOutcomeReadinessV4::AwaitingOutcomeFinality
    } else {
        MomentumProspectiveOutcomeReadinessV4::ReadyForOutcomeAcquisition
    }
}

fn persist_series_outcome_pb(
    root: &Path,
    category: &str,
    digest: &str,
    bytes: &[u8],
    decode_digest: impl Fn(&[u8]) -> Result<String, String>,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &series_outcome_root(root)
            .join(category)
            .join(format!("{digest}.pb")),
        bytes,
        digest,
        decode_digest,
    )
}

fn persist_series_outcome_raw(
    root: &Path,
    digest: &str,
    bytes: &[u8],
) -> Result<(usize, usize), String> {
    persist_artifact(
        &series_outcome_root(root)
            .join("raw_outcome")
            .join(format!("{digest}.json")),
        bytes,
        digest,
        |stored| Ok(raw_outcome_digest(stored)),
    )
}

fn read_single_series_outcome_raw(root: &Path) -> Result<Option<(String, Vec<u8>)>, String> {
    let directory = series_outcome_root(root).join("raw_outcome");
    if !directory.exists() {
        return Ok(None);
    }
    let mut paths = fs::read_dir(directory)
        .map_err(|_| "V4 event-two raw outcome directory read failed".to_string())?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|_| "V4 event-two raw outcome entry read failed".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    paths.sort();
    if paths.len() != 1 {
        return Err("V4 event-two raw outcome identity rejected".to_string());
    }
    let digest = paths[0]
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "V4 event-two raw outcome filename rejected".to_string())?
        .to_string();
    let bytes =
        fs::read(&paths[0]).map_err(|_| "V4 event-two raw outcome read failed".to_string())?;
    if raw_outcome_digest(&bytes) != digest {
        return Err("V4 event-two raw outcome digest rejected".to_string());
    }
    Ok(Some((digest, bytes)))
}

fn event_two_input_row(
    root: &Path,
    live: &MomentumProspectiveSeriesReportV4,
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
) -> Result<(String, HistoricalOhlcvRow), String> {
    let capsule = live
        .input_capsule
        .as_ref()
        .ok_or_else(|| "V4 event-two input capsule unavailable".to_string())?;
    let raw = fs::read(
        series_root(root)
            .join("raw_input")
            .join(format!("{}.json", capsule.raw_response_digest)),
    )
    .map_err(|_| "V4 event-two input raw evidence unavailable".to_string())?;
    if stable_hash_string(&format!("momentum-v4-series-raw-input:{raw:?}"))
        != capsule.raw_response_digest
        || !sanitized_raw_response(&raw, registration.maximum_response_bytes)
    {
        return Err("V4 event-two input raw evidence rejected".to_string());
    }
    let dataset = parse_upbit_daily_ohlcv_v0(
        std::str::from_utf8(&raw)
            .map_err(|_| "V4 event-two input raw encoding rejected".to_string())?,
        &registration.symbol,
    )?;
    let row = dataset
        .rows
        .iter()
        .find(|row| row.timestamp_ms == registration.event_timestamp_ms)
        .cloned()
        .ok_or_else(|| "V4 event-two input event row unavailable".to_string())?;
    let row_digests = dataset
        .rows
        .iter()
        .map(row_identity_digest)
        .collect::<Vec<_>>();
    if row_digests != capsule.row_identity_digests || !valid_series_row(&row, &registration.symbol)
    {
        return Err("V4 event-two input row identity rejected".to_string());
    }
    Ok((capsule.raw_response_digest.clone(), row))
}

fn valid_series_row(row: &HistoricalOhlcvRow, symbol: &str) -> bool {
    row.symbol == symbol && valid_outcome_ohlcv(row)
}

fn build_series_outcome_proof(
    root: &Path,
    live: &MomentumProspectiveSeriesReportV4,
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    outcome_row: &HistoricalOhlcvRow,
    raw_outcome_response: &[u8],
) -> Result<MomentumOutcomeRowIdentityProofV4_4, String> {
    let (raw_input_response_digest, input_row) = event_two_input_row(root, live, registration)?;
    if outcome_row.timestamp_ms != registration.exact_expected_timestamp_ms[0]
        || outcome_row.timestamp_ms == input_row.timestamp_ms
        || !valid_series_row(outcome_row, &registration.symbol)
    {
        return Err("V4 event-two outcome row proof input rejected".to_string());
    }
    let mut value = MomentumOutcomeRowIdentityProofV4_4 {
        proof_version: ROW_PROOF_VERSION.to_string(),
        registration_digest: registration.registration_digest.clone(),
        prediction_capsule_digest: registration.prediction_capsule_digest.clone(),
        input_capsule_digest: registration.input_capsule_digest.clone(),
        event_timestamp_ms: registration.event_timestamp_ms,
        outcome_timestamp_ms: outcome_row.timestamp_ms,
        input_event_row_digest: row_identity_digest(&input_row),
        outcome_row_digest: row_identity_digest(outcome_row),
        raw_input_response_digest,
        raw_outcome_response_digest: raw_outcome_digest(raw_outcome_response),
        exact_timestamp_verified: true,
        strict_single_row_verified: true,
        finalized: true,
        sanitized: true,
        credential_free: true,
        read_only: true,
        proof_digest: String::new(),
    };
    value.proof_digest = row_proof_digest(&value);
    validate_series_outcome_proof(&value)?;
    Ok(value)
}

fn build_series_success_receipt_and_capsule(
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    proof: &MomentumOutcomeRowIdentityProofV4_4,
) -> Result<
    (
        MomentumOutcomeAcquisitionReceiptV4_4,
        MomentumSealedOutcomeCapsuleV4_4,
    ),
    String,
> {
    let mut receipt = MomentumOutcomeAcquisitionReceiptV4_4 {
        receipt_version: RECEIPT_VERSION.to_string(),
        registration_digest: registration.registration_digest.clone(),
        prediction_capsule_digest: registration.prediction_capsule_digest.clone(),
        outcome_plan_digest: registration.outcome_plan_digest.clone(),
        request_attempt_count: 1,
        retry_count: 0,
        http_status_class: Some(200),
        returned_row_count: 1,
        verified_row_count: 1,
        outcome_capsule_digest: None,
        status: MomentumOutcomeAcquisitionStatusV4_4::EvidenceAcquired,
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = series_outcome_receipt_digest(&receipt);
    let mut capsule = MomentumSealedOutcomeCapsuleV4_4 {
        capsule_version: CAPSULE_VERSION.to_string(),
        registration_digest: registration.registration_digest.clone(),
        receipt_digest: receipt.receipt_digest.clone(),
        prediction_capsule_digest: registration.prediction_capsule_digest.clone(),
        event_timestamp_ms: registration.event_timestamp_ms,
        outcome_timestamp_ms: proof.outcome_timestamp_ms,
        outcome_row_digest: proof.outcome_row_digest.clone(),
        labels_opened: false,
        probabilities_opened: false,
        metrics_computed: false,
        winner_selected: false,
        reward_applied: false,
        penalty_applied: false,
        capsule_digest: String::new(),
    };
    capsule.capsule_digest = sealed_outcome_capsule_digest(&capsule);
    receipt.outcome_capsule_digest = Some(capsule.capsule_digest.clone());
    validate_series_outcome_receipt(&receipt)?;
    validate_series_outcome_capsule(&capsule)?;
    Ok((receipt, capsule))
}

fn build_series_terminal_receipt(
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    status: MomentumOutcomeAcquisitionStatusV4_4,
    http_status_class: Option<u16>,
    returned_row_count: usize,
) -> Result<MomentumOutcomeAcquisitionReceiptV4_4, String> {
    let mut value = MomentumOutcomeAcquisitionReceiptV4_4 {
        receipt_version: RECEIPT_VERSION.to_string(),
        registration_digest: registration.registration_digest.clone(),
        prediction_capsule_digest: registration.prediction_capsule_digest.clone(),
        outcome_plan_digest: registration.outcome_plan_digest.clone(),
        request_attempt_count: 1,
        retry_count: 0,
        http_status_class,
        returned_row_count,
        verified_row_count: 0,
        outcome_capsule_digest: None,
        status,
        receipt_digest: String::new(),
    };
    value.receipt_digest = series_outcome_receipt_digest(&value);
    validate_series_outcome_receipt(&value)?;
    Ok(value)
}

fn persist_series_success_acquisition(
    root: &Path,
    raw: &[u8],
    proof: &MomentumOutcomeRowIdentityProofV4_4,
    receipt: &MomentumOutcomeAcquisitionReceiptV4_4,
    capsule: &MomentumSealedOutcomeCapsuleV4_4,
) -> Result<(usize, usize), String> {
    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_series_outcome_raw(root, &proof.raw_outcome_response_digest, raw)?,
    );
    add_counts(
        &mut counts,
        persist_series_outcome_pb(
            root,
            "outcome_row_proofs",
            &proof.proof_digest,
            &encode_series_outcome_proof(proof)?,
            |bytes| Ok(decode_series_outcome_proof(bytes)?.proof_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_series_outcome_pb(
            root,
            "outcome_receipts",
            &receipt.receipt_digest,
            &encode_series_outcome_receipt(receipt)?,
            |bytes| Ok(decode_series_outcome_receipt(bytes)?.receipt_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_series_outcome_pb(
            root,
            "outcome_capsules",
            &capsule.capsule_digest,
            &encode_series_outcome_capsule(capsule)?,
            |bytes| Ok(decode_series_outcome_capsule(bytes)?.capsule_digest),
        )?,
    );
    Ok(counts)
}

fn ordered_event_two_seals(
    root: &Path,
    live: &MomentumProspectiveSeriesReportV4,
) -> Result<Vec<MomentumSeriesParticipantPredictionSealV4>, String> {
    let capsule = live
        .prediction_capsule
        .as_ref()
        .ok_or_else(|| "V4 event-two prediction capsule unavailable".to_string())?;
    let stored = reopen_prediction_seals(&series_root(root))?;
    let by_digest = stored
        .into_iter()
        .map(|seal| (seal.seal_digest.clone(), seal))
        .collect::<BTreeMap<_, _>>();
    let mut ordered = Vec::with_capacity(3);
    for (seal_digest, prediction_digest) in capsule
        .participant_seal_digests
        .iter()
        .zip(&capsule.participant_prediction_digests)
    {
        let seal = by_digest
            .get(seal_digest)
            .ok_or_else(|| "V4 event-two participant seal unavailable".to_string())?;
        if seal.prediction_digest != *prediction_digest
            || seal.epoch_number != live.epoch_registration.epoch_number
            || seal.epoch_registration_digest != live.epoch_registration.registration_digest
            || seal.event_timestamp_ms != live.epoch_registration.event_timestamp_ms
        {
            return Err("V4 event-two participant seal binding rejected".to_string());
        }
        ordered.push(seal.clone());
    }
    if ordered.len() != 3
        || ordered
            .iter()
            .map(|seal| &seal.participant_digest)
            .collect::<BTreeSet<_>>()
            .len()
            != 3
        || by_digest.len() != 3
    {
        return Err("V4 event-two participant roster rejected".to_string());
    }
    Ok(ordered)
}

fn derive_series_opening_authorization(
    root: &Path,
    live: &MomentumProspectiveSeriesReportV4,
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    receipt: &MomentumOutcomeAcquisitionReceiptV4_4,
    capsule: &MomentumSealedOutcomeCapsuleV4_4,
) -> Result<MomentumOutcomeOpeningAuthorizationV4_4, String> {
    let prediction = live
        .prediction_capsule
        .as_ref()
        .ok_or_else(|| "V4 event-two prediction capsule unavailable".to_string())?;
    let journal = live
        .journal_entry
        .as_ref()
        .ok_or_else(|| "V4 event-two prediction journal unavailable".to_string())?;
    let seals = ordered_event_two_seals(root, live)?;
    let mut value = MomentumOutcomeOpeningAuthorizationV4_4 {
        authorization_version: OPENING_AUTHORIZATION_VERSION.to_string(),
        outcome_registration_digest: registration.registration_digest.clone(),
        outcome_receipt_digest: receipt.receipt_digest.clone(),
        outcome_capsule_digest: capsule.capsule_digest.clone(),
        prediction_capsule_digest: prediction.capsule_digest.clone(),
        prediction_journal_digest: journal.entry_digest.clone(),
        participant_seal_digests: seals.iter().map(|seal| seal.seal_digest.clone()).collect(),
        participant_prediction_digests: seals
            .iter()
            .map(|seal| seal.prediction_digest.clone())
            .collect(),
        feature_policy_digest: live.series.feature_policy_digest.clone(),
        label_policy_digest: frozen_label_policy_digest(),
        evaluation_policy_digest: evaluation_policy_digest(),
        opening_attempt_count_before: 0,
        opened_event_count_before: 0,
        explicit_owner_authorization: true,
        one_time_only: true,
        winner_selection_forbidden: true,
        ranking_forbidden: true,
        reward_application_forbidden: true,
        penalty_application_forbidden: true,
        chair_action_forbidden: true,
        voice_mutation_forbidden: true,
        promotion_forbidden: true,
        trading_forbidden: true,
        authorization_digest: String::new(),
    };
    value.authorization_digest = authorization_digest(&value);
    validate_opening_authorization_shape(&value)?;
    if receipt.status != MomentumOutcomeAcquisitionStatusV4_4::EvidenceAcquired
        || receipt.outcome_capsule_digest.as_deref() != Some(capsule.capsule_digest.as_str())
        || capsule.receipt_digest != receipt.receipt_digest
        || capsule.prediction_capsule_digest != prediction.capsule_digest
        || value.participant_seal_digests != prediction.participant_seal_digests
        || value.participant_prediction_digests != prediction.participant_prediction_digests
    {
        return Err("V4 event-two opening authorization binding rejected".to_string());
    }
    Ok(value)
}

fn reopen_event_two_opening_closes(
    root: &Path,
    live: &MomentumProspectiveSeriesReportV4,
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    proof: &MomentumOutcomeRowIdentityProofV4_4,
) -> Result<(f64, f64), String> {
    let (_, input_row) = event_two_input_row(root, live, registration)?;
    if row_identity_digest(&input_row) != proof.input_event_row_digest {
        return Err("V4 event-two opening input identity rejected".to_string());
    }
    let (raw_digest, raw) = read_single_series_outcome_raw(root)?
        .ok_or_else(|| "V4 event-two opening raw outcome unavailable".to_string())?;
    if raw_digest != proof.raw_outcome_response_digest
        || !sanitized_raw_response(&raw, registration.maximum_response_bytes)
    {
        return Err("V4 event-two opening raw outcome rejected".to_string());
    }
    let dataset = parse_upbit_daily_ohlcv_v0(
        std::str::from_utf8(&raw)
            .map_err(|_| "V4 event-two opening raw encoding rejected".to_string())?,
        &registration.symbol,
    )?;
    if dataset.rows.len() != 1 {
        return Err("V4 event-two opening row count rejected".to_string());
    }
    let outcome_row = &dataset.rows[0];
    if outcome_row.timestamp_ms != registration.exact_expected_timestamp_ms[0]
        || row_identity_digest(outcome_row) != proof.outcome_row_digest
        || !valid_series_row(outcome_row, &registration.symbol)
    {
        return Err("V4 event-two opening outcome identity rejected".to_string());
    }
    Ok((input_row.close, outcome_row.close))
}

fn build_series_evaluations(
    root: &Path,
    live: &MomentumProspectiveSeriesReportV4,
    proof: &MomentumOutcomeRowIdentityProofV4_4,
    label_status: MomentumProspectiveLabelStatusV4_4,
    label: Option<bool>,
    return_bits: u64,
) -> Result<Vec<MomentumParticipantProspectiveEvaluationV4_4>, String> {
    let seals = ordered_event_two_seals(root, live)?;
    let values = seals
        .iter()
        .map(|seal| {
            build_participant_evaluation_v4_4(
                &seal.participant_digest,
                &seal.participant_role,
                &seal.seal_digest,
                &seal.prediction_digest,
                seal.prediction_probability_bits,
                seal.event_timestamp_ms,
                proof,
                label_status,
                label,
                return_bits,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    if values.len() != 3 {
        return Err("V4 event-two evaluation count rejected".to_string());
    }
    Ok(values)
}

fn build_series_opening_bundle(
    live: &MomentumProspectiveSeriesReportV4,
    authorization: &MomentumOutcomeOpeningAuthorizationV4_4,
    capsule: &MomentumSealedOutcomeCapsuleV4_4,
    label_status: MomentumProspectiveLabelStatusV4_4,
    evaluations: Vec<MomentumParticipantProspectiveEvaluationV4_4>,
) -> Result<MomentumOutcomeOpeningBundleV4_4, String> {
    let prediction = live
        .prediction_capsule
        .as_ref()
        .ok_or_else(|| "V4 event-two prediction capsule unavailable".to_string())?;
    let mut value = MomentumOutcomeOpeningBundleV4_4 {
        bundle_version: OPENING_BUNDLE_VERSION.to_string(),
        authorization_digest: authorization.authorization_digest.clone(),
        outcome_capsule_digest: capsule.capsule_digest.clone(),
        prediction_capsule_digest: prediction.capsule_digest.clone(),
        opening_attempt_count: 1,
        opened_event_count: 1,
        label_status,
        participant_evaluations: evaluations,
        metrics_computed: label_status == MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome,
        winner_selected: false,
        ranking_created: false,
        reward_applied: false,
        penalty_applied: false,
        chair_action_taken: false,
        bundle_digest: String::new(),
    };
    value.bundle_digest = opening_bundle_digest(&value);
    validate_opening_bundle_shape(&value)?;
    Ok(value)
}

fn build_series_opening_receipt(
    authorization: &MomentumOutcomeOpeningAuthorizationV4_4,
    bundle: &MomentumOutcomeOpeningBundleV4_4,
) -> Result<MomentumOutcomeOpeningReceiptV4_4, String> {
    let mut value = MomentumOutcomeOpeningReceiptV4_4 {
        receipt_version: OPENING_RECEIPT_VERSION.to_string(),
        authorization_digest: authorization.authorization_digest.clone(),
        opening_bundle_digest: bundle.bundle_digest.clone(),
        opening_attempt_count: 1,
        opened_event_count: 1,
        status: if bundle.label_status == MomentumProspectiveLabelStatusV4_4::NeutralOutcomeExcluded
        {
            MomentumOutcomeOpeningStatusV4_4::NeutralOutcomeOpened
        } else {
            MomentumOutcomeOpeningStatusV4_4::Opened
        },
        receipt_digest: String::new(),
    };
    value.receipt_digest = opening_receipt_digest(&value);
    validate_opening_receipt_shape(&value)?;
    Ok(value)
}

fn build_series_ledger_and_eligibility(
    root: &Path,
    live: &MomentumProspectiveSeriesReportV4,
    receipt: &MomentumOutcomeAcquisitionReceiptV4_4,
    capsule: &MomentumSealedOutcomeCapsuleV4_4,
    authorization: &MomentumOutcomeOpeningAuthorizationV4_4,
    bundle: &MomentumOutcomeOpeningBundleV4_4,
    prior_pause_digest: &str,
) -> Result<
    (
        MomentumProspectiveSeriesLedgerEntryV4,
        MomentumProspectiveSeriesEligibilityReceiptV4,
        LiveProspectiveContinuationPauseV2,
    ),
    String,
> {
    let input_receipt = live
        .input_receipt
        .as_ref()
        .ok_or_else(|| "V4 event-two input receipt unavailable".to_string())?;
    let input_capsule = live
        .input_capsule
        .as_ref()
        .ok_or_else(|| "V4 event-two input capsule unavailable".to_string())?;
    let context = live
        .context_use_proof
        .as_ref()
        .ok_or_else(|| "V4 event-two context proof unavailable".to_string())?;
    let prediction = live
        .prediction_capsule
        .as_ref()
        .ok_or_else(|| "V4 event-two prediction capsule unavailable".to_string())?;
    let journal = live
        .journal_entry
        .as_ref()
        .ok_or_else(|| "V4 event-two prediction journal unavailable".to_string())?;
    let plan = live
        .outcome_plan
        .as_ref()
        .ok_or_else(|| "V4 event-two outcome plan unavailable".to_string())?;
    let completed_event_count = live.event_one_adoption.total_event_count + 1;
    let scorable_event_count = live.event_one_adoption.scorable_event_count
        + usize::from(
            bundle.label_status == MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome,
        );
    let mut ledger = MomentumProspectiveSeriesLedgerEntryV4 {
        ledger_version: SERIES_LEDGER_VERSION.to_string(),
        previous_event_ledger_entry_digest: live
            .event_one_adoption
            .evaluation_ledger_entry_digest
            .clone(),
        series_digest: live.series.series_digest.clone(),
        epoch_registration_digest: live.epoch_registration.registration_digest.clone(),
        input_receipt_digest: input_receipt.receipt_digest.clone(),
        input_capsule_digest: input_capsule.capsule_digest.clone(),
        context_proof_digest: context.proof_digest.clone(),
        participant_seal_digests: prediction.participant_seal_digests.clone(),
        prediction_capsule_digest: prediction.capsule_digest.clone(),
        prediction_journal_digest: journal.entry_digest.clone(),
        outcome_plan_digest: plan.plan_digest.clone(),
        outcome_receipt_digest: receipt.receipt_digest.clone(),
        outcome_capsule_digest: capsule.capsule_digest.clone(),
        opening_authorization_digest: authorization.authorization_digest.clone(),
        opening_bundle_digest: bundle.bundle_digest.clone(),
        label_status: bundle.label_status,
        participant_evaluation_digests: bundle
            .participant_evaluations
            .iter()
            .map(|value| value.evaluation_digest.clone())
            .collect(),
        total_event_count_after: completed_event_count,
        scorable_event_count_after: scorable_event_count,
        winner_selected: false,
        ranking_created: false,
        reward_applied: false,
        penalty_applied: false,
        chair_action_taken: false,
        trading_action_taken: false,
        entry_digest: String::new(),
    };
    ledger.entry_digest = series_ledger_entry_digest(&ledger);
    validate_series_ledger_entry(&ledger)?;
    let participant_roles = ordered_event_two_seals(root, live)?
        .iter()
        .map(|seal| seal.participant_role.clone())
        .collect::<Vec<_>>();
    let minimum_sample_gate = MomentumLearningCampaignConfigV0::default().minimum_test_samples;
    let integrity_verified = participant_roles.len() == 3
        && participant_roles.iter().cloned().collect::<BTreeSet<_>>()
            == [
                "RawFeatureLogisticV4",
                "RawFeatureInteractionLogisticV4",
                "TrainingPrevalenceConstantV4",
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
    let status = if !integrity_verified {
        MomentumProspectiveSeriesEligibilityV4::IntegrityFailure
    } else if scorable_event_count < minimum_sample_gate {
        MomentumProspectiveSeriesEligibilityV4::IneligibleMinimumSamples
    } else {
        MomentumProspectiveSeriesEligibilityV4::EligibleForShadowRewardAssessment
    };
    let mut eligibility = MomentumProspectiveSeriesEligibilityReceiptV4 {
        receipt_version: SERIES_ELIGIBILITY_VERSION.to_string(),
        event_one_eligibility_digest: live.event_one_adoption.reward_eligibility_digest.clone(),
        event_two_ledger_entry_digest: ledger.entry_digest.clone(),
        participant_roles,
        completed_event_count,
        scorable_event_count,
        minimum_sample_gate,
        status,
        integrity_verified,
        reward_application_count: 0,
        penalty_application_count: 0,
        receipt_digest: String::new(),
    };
    eligibility.receipt_digest = series_eligibility_digest(&eligibility);
    validate_series_eligibility(&eligibility)?;
    let mut pause = LiveProspectiveContinuationPauseV2 {
        pause_version: COMPLETED_PAUSE_VERSION.to_string(),
        policy: LiveProspectiveContinuationPolicyV2::PausedAfterCompletedEpochTwo,
        prior_pause_digest: prior_pause_digest.to_string(),
        event_two_ledger_entry_digest: ledger.entry_digest.clone(),
        completed_event_count,
        scorable_event_count,
        eligibility_receipt_digest: eligibility.receipt_digest.clone(),
        epoch_three_registered: false,
        historical_challenger_research_prioritized: true,
        scheduler_count: 0,
        automatic_registration_count: 0,
        network_authority_count: 0,
        pause_digest: String::new(),
    };
    pause.pause_digest = completed_pause_digest(&pause);
    validate_completed_pause(&pause)?;
    Ok((ledger, eligibility, pause))
}

fn persist_series_opening(
    root: &Path,
    authorization: &MomentumOutcomeOpeningAuthorizationV4_4,
    bundle: &MomentumOutcomeOpeningBundleV4_4,
    opening_receipt: &MomentumOutcomeOpeningReceiptV4_4,
    ledger: &MomentumProspectiveSeriesLedgerEntryV4,
    eligibility: &MomentumProspectiveSeriesEligibilityReceiptV4,
    pause: &LiveProspectiveContinuationPauseV2,
) -> Result<(usize, usize), String> {
    let mut counts = (0, 0);
    validate_opening_authorization_shape(authorization)?;
    if bundle.authorization_digest != authorization.authorization_digest {
        return Err("V4 event-two opening authorization changed".to_string());
    }
    for evaluation in &bundle.participant_evaluations {
        add_counts(
            &mut counts,
            persist_series_outcome_pb(
                root,
                "participant_evaluations",
                &evaluation.evaluation_digest,
                &encode_series_evaluation(evaluation)?,
                |bytes| Ok(decode_series_evaluation(bytes)?.evaluation_digest),
            )?,
        );
    }
    add_counts(
        &mut counts,
        persist_series_outcome_pb(
            root,
            "opening_bundles",
            &bundle.bundle_digest,
            &encode_series_opening_bundle(bundle)?,
            |bytes| Ok(decode_series_opening_bundle(bytes)?.bundle_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_series_outcome_pb(
            root,
            "opening_receipts",
            &opening_receipt.receipt_digest,
            &encode_series_opening_receipt(opening_receipt)?,
            |bytes| Ok(decode_series_opening_receipt(bytes)?.receipt_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_series_outcome_pb(
            root,
            "eligibility_receipts",
            &eligibility.receipt_digest,
            &encode_series_eligibility(eligibility)?,
            |bytes| Ok(decode_series_eligibility(bytes)?.receipt_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_artifact(
            &series_root(root)
                .join(COMPLETED_PAUSE_DIRECTORY)
                .join(format!("{}.pb", pause.pause_digest)),
            &encode_completed_pause(pause)?,
            &pause.pause_digest,
            |bytes| Ok(decode_completed_pause(bytes)?.pause_digest),
        )?,
    );
    // The ledger is the completion marker and is intentionally persisted last.
    add_counts(
        &mut counts,
        persist_series_outcome_pb(
            root,
            "evaluation_ledger",
            &ledger.entry_digest,
            &encode_series_ledger_entry(ledger)?,
            |bytes| Ok(decode_series_ledger_entry(bytes)?.entry_digest),
        )?,
    );
    Ok(counts)
}

fn persist_series_outcome_registration(
    root: &Path,
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
) -> Result<(usize, usize), String> {
    persist_series_outcome_pb(
        root,
        "outcome_registrations",
        &registration.registration_digest,
        &encode_series_outcome_registration(registration)?,
        |bytes| Ok(decode_series_outcome_registration(bytes)?.registration_digest),
    )
}

fn persist_series_terminal_receipt(
    root: &Path,
    receipt: &MomentumOutcomeAcquisitionReceiptV4_4,
) -> Result<(usize, usize), String> {
    persist_series_outcome_pb(
        root,
        "outcome_receipts",
        &receipt.receipt_digest,
        &encode_series_outcome_receipt(receipt)?,
        |bytes| Ok(decode_series_outcome_receipt(bytes)?.receipt_digest),
    )
}

fn validate_prospective_outcome_status(
    value: &MomentumProspectiveOutcomeStatusV4,
) -> Result<(), String> {
    let zero_authority = [
        value.safety_counters.winner_selections,
        value.safety_counters.ranking_creations,
        value.safety_counters.reward_applications,
        value.safety_counters.penalty_applications,
        value.safety_counters.chair_model_executions,
        value.safety_counters.chair_learning_actions,
        value.safety_counters.chair_decisions,
        value.safety_counters.committee_votes,
        value.safety_counters.voice_changes,
        value.safety_counters.tier_changes,
        value.safety_counters.cooldowns,
        value.safety_counters.promotions,
        value.safety_counters.quarantines,
        value.safety_counters.paper_executions,
        value.safety_counters.live_executions,
        value.safety_counters.epoch_three_registrations,
    ]
    .into_iter()
    .all(|value| value == 0);
    if value.series_digest.is_empty()
        || value.epoch_number != 2
        || value.event_timestamp_ms == 0
        || value.required_outcome_timestamp_ms
            != value.event_timestamp_ms.saturating_add(DAILY_CADENCE_MS)
        || value.outcome_finality_boundary_ms
            != value
                .required_outcome_timestamp_ms
                .saturating_add(DAILY_CADENCE_MS)
        || value.prediction_capsule_digest.is_empty()
        || value.prediction_journal_digest.is_empty()
        || value.outcome_plan_digest.is_empty()
        || value.provider_id != "upbit"
        || value.market != "btc_crypto"
        || value.symbol != "KRW-BTC"
        || value.cadence != "1d"
        || value.request_start_timestamp_ms != value.required_outcome_timestamp_ms
        || value.request_end_timestamp_ms != value.outcome_finality_boundary_ms
        || value.request_fingerprint.is_empty()
        || value.maximum_requests != 1
        || value.maximum_retries != 0
        || value.maximum_concurrency != 1
        || value.prior_attempt_count > 1
        || value.completed_event_count == 0
        || value.scorable_event_count > value.completed_event_count
        || value.protected_live_artifact_count == 0
        || value.protected_live_aggregate_digest.is_empty()
        || value.event_one_chain_digest.is_empty()
        || value.event_two_sealed_chain_digest.is_empty()
        || value.historical_store_digest.is_empty()
        || value.qualified_six_replay_digest.is_empty()
        || value.diagnostic_store_digest.is_empty()
        || value.active_roster_digest.is_empty()
        || value.participant_parameter_digests.len() != 3
        || value.participant_normalizer_digests.len() != 3
        || value.feature_policy_digest.is_empty()
        || value.label_policy_digest != frozen_label_policy_digest()
        || value.prior_pause_digest.is_empty()
        || value.epoch_three_registered
        || !zero_authority
        || value.safety_counters.retries != 0
        || value.safety_counters.maximum_observed_concurrency > 1
        || !value.protected_artifacts_unchanged
        || !value.active_state_unchanged
        || value.status_digest != prospective_outcome_status_digest(value)
    {
        return Err("V4 prospective outcome status rejected".to_string());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_prospective_outcome_status(
    live: &MomentumProspectiveSeriesReportV4,
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    artifacts: &SeriesOutcomeArtifacts,
    readiness: MomentumProspectiveOutcomeReadinessV4,
    protected_live: (usize, String),
    historical_identities: (String, String, String),
    prior_pause_digest: String,
    protected_artifacts_unchanged: bool,
    active_state_unchanged: bool,
    safety_counters: MomentumProspectiveOutcomeSafetyCountersV4,
    counts: (usize, usize),
) -> Result<MomentumProspectiveOutcomeStatusV4, String> {
    let prediction = live
        .prediction_capsule
        .as_ref()
        .ok_or_else(|| "V4 event-two prediction capsule unavailable".to_string())?;
    let journal = live
        .journal_entry
        .as_ref()
        .ok_or_else(|| "V4 event-two prediction journal unavailable".to_string())?;
    let plan = live
        .outcome_plan
        .as_ref()
        .ok_or_else(|| "V4 event-two outcome plan unavailable".to_string())?;
    let request = build_outcome_request(registration)?;
    let bundle = artifacts.opening_bundle.as_ref();
    let ledger = artifacts.ledger.as_ref();
    let eligibility = artifacts.eligibility.as_ref();
    let pause = artifacts.pause.as_ref();
    let mut participant_evaluation_digests = bundle.map_or_else(Vec::new, |bundle| {
        bundle
            .participant_evaluations
            .iter()
            .map(|value| value.evaluation_digest.clone())
            .collect()
    });
    participant_evaluation_digests.sort();
    let mut participant_parameter_digests = live.series.parameter_digests.clone();
    participant_parameter_digests.sort();
    let mut participant_normalizer_digests = live.series.normalizer_digests.clone();
    participant_normalizer_digests.sort();
    let mut value = MomentumProspectiveOutcomeStatusV4 {
        series_digest: live.series.series_digest.clone(),
        epoch_number: live.epoch_registration.epoch_number,
        event_timestamp_ms: registration.event_timestamp_ms,
        required_outcome_timestamp_ms: registration.exact_expected_timestamp_ms[0],
        outcome_finality_boundary_ms: registration.outcome_finality_boundary_ms,
        prediction_capsule_digest: prediction.capsule_digest.clone(),
        prediction_journal_digest: journal.entry_digest.clone(),
        outcome_plan_digest: plan.plan_digest.clone(),
        provider_id: registration.provider_id.clone(),
        market: registration.market.clone(),
        symbol: registration.symbol.clone(),
        cadence: registration.cadence.clone(),
        request_start_timestamp_ms: request
            .lookback
            .start_timestamp_ms
            .ok_or_else(|| "V4 event-two request start unavailable".to_string())?,
        request_end_timestamp_ms: request
            .lookback
            .end_timestamp_ms
            .ok_or_else(|| "V4 event-two request end unavailable".to_string())?,
        request_fingerprint: outcome_request_fingerprint(registration),
        maximum_requests: registration.maximum_requests,
        maximum_retries: registration.maximum_retries,
        maximum_concurrency: registration.maximum_concurrency,
        prior_attempt_count: artifacts
            .receipt
            .as_ref()
            .map_or(0, |receipt| receipt.request_attempt_count),
        outcome_receipt_digest: artifacts
            .receipt
            .as_ref()
            .map(|receipt| receipt.receipt_digest.clone()),
        outcome_capsule_digest: artifacts
            .capsule
            .as_ref()
            .map(|capsule| capsule.capsule_digest.clone()),
        opening_authorization_digest: artifacts
            .authorization
            .as_ref()
            .map(|value| value.authorization_digest.clone()),
        opening_bundle_digest: bundle.map(|value| value.bundle_digest.clone()),
        participant_evaluation_digests,
        label_status: bundle.map(|value| value.label_status),
        event_two_ledger_entry_digest: ledger.map(|value| value.entry_digest.clone()),
        completed_event_count: ledger.map_or(live.event_one_adoption.total_event_count, |value| {
            value.total_event_count_after
        }),
        scorable_event_count: ledger
            .map_or(live.event_one_adoption.scorable_event_count, |value| {
                value.scorable_event_count_after
            }),
        eligibility_status: eligibility.map_or(
            MomentumProspectiveSeriesEligibilityV4::IneligibleMinimumSamples,
            |value| value.status,
        ),
        eligibility_receipt_digest: eligibility.map(|value| value.receipt_digest.clone()),
        completed_pause_digest: pause.map(|value| value.pause_digest.clone()),
        readiness,
        protected_live_artifact_count: protected_live.0,
        protected_live_aggregate_digest: protected_live.1,
        event_one_chain_digest: stable_hash_string(&format!(
            "momentum-v4-event-one-chain:{}:{}:{}:{}",
            live.event_one_adoption.prediction_capsule_digest,
            live.event_one_adoption.outcome_capsule_digest,
            live.event_one_adoption.opening_bundle_digest,
            live.event_one_adoption.evaluation_ledger_entry_digest,
        )),
        event_two_sealed_chain_digest: stable_hash_string(&format!(
            "momentum-v4-event-two-sealed-chain:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            live.series.series_digest,
            live.epoch_registration.registration_digest,
            live.input_receipt
                .as_ref()
                .map(|value| value.receipt_digest.as_str())
                .unwrap_or_default(),
            live.input_capsule
                .as_ref()
                .map(|value| value.capsule_digest.as_str())
                .unwrap_or_default(),
            live.context_use_proof
                .as_ref()
                .map(|value| value.proof_digest.as_str())
                .unwrap_or_default(),
            prediction.capsule_digest,
            journal.entry_digest,
            plan.plan_digest,
            prediction.participant_seal_digests.len(),
        )),
        historical_store_digest: historical_identities.0,
        qualified_six_replay_digest: historical_identities.1,
        diagnostic_store_digest: historical_identities.2,
        active_roster_digest: stable_hash_string(&format!(
            "{:?}",
            canonical_current_agent_states()
        )),
        participant_parameter_digests,
        participant_normalizer_digests,
        feature_policy_digest: live.series.feature_policy_digest.clone(),
        label_policy_digest: frozen_label_policy_digest(),
        prior_pause_digest,
        epoch_three_registered: false,
        safety_counters,
        artifacts_written: counts.0,
        duplicate_artifact_count: counts.1,
        protected_artifacts_unchanged,
        active_state_unchanged,
        status_digest: String::new(),
    };
    value.status_digest = prospective_outcome_status_digest(&value);
    validate_prospective_outcome_status(&value)?;
    Ok(value)
}

fn outcome_report(
    status: MomentumProspectiveOutcomeStatusV4,
    registration: MomentumOutcomeAcquisitionRegistrationV4_4,
    artifacts: SeriesOutcomeArtifacts,
) -> MomentumProspectiveOutcomeReportV4 {
    MomentumProspectiveOutcomeReportV4 {
        status,
        registration,
        receipt: artifacts.receipt,
        outcome_capsule: artifacts.capsule,
        opening_authorization: artifacts.authorization,
        opening_receipt: artifacts.opening_receipt,
        opening_bundle: artifacts.opening_bundle,
        ledger_entry: artifacts.ledger,
        eligibility_receipt: artifacts.eligibility,
        completed_pause: artifacts.pause,
    }
}

fn validate_series_outcome_authority(
    mode: MomentumProspectiveOutcomeRunModeV4,
    network_allowed: bool,
    request_confirmed: bool,
    opening_confirmed: bool,
    requested_epoch: Option<u64>,
) -> Result<(), String> {
    match mode {
        MomentumProspectiveOutcomeRunModeV4::Status
        | MomentumProspectiveOutcomeRunModeV4::DryRun => {
            if network_allowed
                || request_confirmed
                || opening_confirmed
                || requested_epoch.is_some()
            {
                return Err("V4 event-two read-only mode rejects authority".to_string());
            }
        }
        MomentumProspectiveOutcomeRunModeV4::ExecuteOutcome => {
            if !network_allowed
                || !request_confirmed
                || opening_confirmed
                || requested_epoch != Some(2)
            {
                return Err(
                    "V4 event-two acquisition requires epoch two and exact network confirmation"
                        .to_string(),
                );
            }
        }
        MomentumProspectiveOutcomeRunModeV4::OpenOutcome => {
            if network_allowed
                || request_confirmed
                || !opening_confirmed
                || requested_epoch != Some(2)
            {
                return Err(
                    "V4 event-two opening requires epoch two and exact local confirmation"
                        .to_string(),
                );
            }
        }
    }
    Ok(())
}

fn recover_series_success_acquisition(
    root: &Path,
    live: &MomentumProspectiveSeriesReportV4,
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    artifacts: &SeriesOutcomeArtifacts,
) -> Result<Option<(usize, usize)>, String> {
    if artifacts.receipt.as_ref().is_some_and(|receipt| {
        receipt.status != MomentumOutcomeAcquisitionStatusV4_4::EvidenceAcquired
    }) || artifacts.capsule.is_some()
    {
        return Ok(None);
    }
    let Some((_, raw)) = read_single_series_outcome_raw(root)? else {
        return Ok(None);
    };
    if !sanitized_raw_response(&raw, registration.maximum_response_bytes) {
        return Err("V4 event-two recovery raw response rejected".to_string());
    }
    let dataset = parse_upbit_daily_ohlcv_v0(
        std::str::from_utf8(&raw)
            .map_err(|_| "V4 event-two recovery raw encoding rejected".to_string())?,
        &registration.symbol,
    )?;
    if dataset.rows.len() != 1 {
        return Err("V4 event-two recovery row count rejected".to_string());
    }
    let row = &dataset.rows[0];
    if row.timestamp_ms != registration.exact_expected_timestamp_ms[0]
        || !valid_series_row(row, &registration.symbol)
    {
        return Err("V4 event-two recovery row rejected".to_string());
    }
    let proof = build_series_outcome_proof(root, live, registration, row, &raw)?;
    if artifacts
        .proof
        .as_ref()
        .is_some_and(|stored| stored != &proof)
    {
        return Err("V4 event-two recovery proof conflict".to_string());
    }
    let (receipt, capsule) = build_series_success_receipt_and_capsule(registration, &proof)?;
    if artifacts
        .receipt
        .as_ref()
        .is_some_and(|stored| stored != &receipt)
    {
        return Err("V4 event-two recovery receipt conflict".to_string());
    }
    Ok(Some(persist_series_success_acquisition(
        root, &raw, &proof, &receipt, &capsule,
    )?))
}

#[allow(clippy::too_many_arguments)]
fn run_prospective_outcome_with_transport<F>(
    root: &Path,
    snapshots: &[DataSnapshot],
    reservation: &ProtectedEvaluationReservationV1,
    provider_config: &UpbitHistoricalPilotConfigV0,
    observed_timestamp_ms: u64,
    mode: MomentumProspectiveOutcomeRunModeV4,
    network_allowed: bool,
    request_confirmed: bool,
    opening_confirmed: bool,
    requested_epoch: Option<u64>,
    transport: F,
) -> Result<MomentumProspectiveOutcomeReportV4, String>
where
    F: FnOnce(
        &UpbitHistoricalPilotConfigV0,
        &ReadOnlyProviderRequest,
    )
        -> Result<LearningEvidenceTransportResponseV1, LearningEvidenceTransportFailureV1>,
{
    validate_series_outcome_authority(
        mode,
        network_allowed,
        request_confirmed,
        opening_confirmed,
        requested_epoch,
    )?;
    provider_config.validate()?;
    let protected_before = protected_live_identity(root)?;
    let active_before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let historical_identities = historical_identity_roots(root)?;
    let prior_pause = prior_live_pause_digest(root)?;
    let live = run_momentum_prospective_series_v4(
        root,
        snapshots,
        reservation,
        provider_config,
        observed_timestamp_ms,
        MomentumProspectiveSeriesRunModeV4::Status,
        false,
        false,
        None,
    )?;
    let registration = derive_series_outcome_registration(&live, provider_config)?;
    let mut artifacts = reopen_series_outcome_artifacts(root)?;
    if artifacts
        .registration
        .as_ref()
        .is_some_and(|stored| stored != &registration)
    {
        return Err("V4 event-two persisted outcome registration changed".to_string());
    }
    let initial_readiness = series_outcome_readiness(
        observed_timestamp_ms,
        &registration,
        &artifacts,
        &live,
        &prior_pause,
    );
    let read_only = matches!(
        mode,
        MomentumProspectiveOutcomeRunModeV4::Status | MomentumProspectiveOutcomeRunModeV4::DryRun
    );
    let completed_or_terminal = matches!(
        initial_readiness,
        MomentumProspectiveOutcomeReadinessV4::OutcomeAlreadyOpened
            | MomentumProspectiveOutcomeReadinessV4::PriorOutcomeAttemptTerminal
    );
    let acquisition_already_complete = mode == MomentumProspectiveOutcomeRunModeV4::ExecuteOutcome
        && initial_readiness == MomentumProspectiveOutcomeReadinessV4::ReadyForOutcomeOpening;
    if read_only || completed_or_terminal || acquisition_already_complete {
        let protected_after = protected_live_identity(root)?;
        let historical_after = historical_identity_roots(root)?;
        let active_after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
        let protected_unchanged =
            protected_before == protected_after && historical_identities == historical_after;
        let status = build_prospective_outcome_status(
            &live,
            &registration,
            &artifacts,
            initial_readiness,
            protected_before.clone(),
            historical_identities,
            prior_pause,
            protected_unchanged,
            active_before == active_after,
            MomentumProspectiveOutcomeSafetyCountersV4::default(),
            (0, 0),
        )?;
        return Ok(outcome_report(status, registration, artifacts));
    }

    let mut counts = (0, 0);
    let mut safety = MomentumProspectiveOutcomeSafetyCountersV4::default();
    match mode {
        MomentumProspectiveOutcomeRunModeV4::ExecuteOutcome => {
            if initial_readiness == MomentumProspectiveOutcomeReadinessV4::AwaitingOutcomeFinality {
                return Err("V4 event-two outcome finality not reached".to_string());
            }
            if !matches!(
                initial_readiness,
                MomentumProspectiveOutcomeReadinessV4::ReadyForOutcomeAcquisition
                    | MomentumProspectiveOutcomeReadinessV4::IntegrityFailure
            ) {
                return Err("V4 event-two acquisition readiness rejected".to_string());
            }
            add_counts(
                &mut counts,
                persist_series_outcome_registration(root, &registration)?,
            );
            artifacts = reopen_series_outcome_artifacts(root)?;
            if let Some(recovered) =
                recover_series_success_acquisition(root, &live, &registration, &artifacts)?
            {
                add_counts(&mut counts, recovered);
            } else {
                if initial_readiness == MomentumProspectiveOutcomeReadinessV4::IntegrityFailure {
                    return Err("V4 event-two partial acquisition cannot recover".to_string());
                }
                let request = build_outcome_request(&registration)?;
                let request_config = outcome_request_config(provider_config, &registration)?;
                safety.network_request_attempts = 1;
                safety.transport_constructions = 1;
                safety.maximum_observed_concurrency = 1;
                match transport(&request_config, &request) {
                    Err(failure) => {
                        let (status, http_status_class, raw) = match failure {
                            LearningEvidenceTransportFailureV1::ProviderRejected {
                                http_status_class,
                                raw_response,
                            } => (
                                MomentumOutcomeAcquisitionStatusV4_4::TerminalHttpFailure,
                                parse_http_status_class(http_status_class.as_deref()),
                                raw_response,
                            ),
                            LearningEvidenceTransportFailureV1::TimedOut
                            | LearningEvidenceTransportFailureV1::Technical => (
                                MomentumOutcomeAcquisitionStatusV4_4::TerminalTransportFailure,
                                None,
                                None,
                            ),
                        };
                        if let Some(raw) = raw
                            && sanitized_raw_response(&raw, registration.maximum_response_bytes)
                        {
                            let digest = raw_outcome_digest(&raw);
                            add_counts(
                                &mut counts,
                                persist_series_outcome_raw(root, &digest, &raw)?,
                            );
                        }
                        let receipt = build_series_terminal_receipt(
                            &registration,
                            status,
                            http_status_class,
                            0,
                        )?;
                        add_counts(
                            &mut counts,
                            persist_series_terminal_receipt(root, &receipt)?,
                        );
                    }
                    Ok(response) => {
                        let returned_row_count = response.response.normalized_dataset.rows.len();
                        match validate_outcome_transport(&registration, &request, &response) {
                            Err(_) => {
                                if sanitized_raw_response(
                                    &response.raw_response,
                                    registration.maximum_response_bytes,
                                ) {
                                    let digest = raw_outcome_digest(&response.raw_response);
                                    add_counts(
                                        &mut counts,
                                        persist_series_outcome_raw(
                                            root,
                                            &digest,
                                            &response.raw_response,
                                        )?,
                                    );
                                }
                                let receipt = build_series_terminal_receipt(
                                    &registration,
                                    MomentumOutcomeAcquisitionStatusV4_4::TerminalValidationFailure,
                                    parse_http_status_class(Some(&response.http_status_class)),
                                    returned_row_count,
                                )?;
                                add_counts(
                                    &mut counts,
                                    persist_series_terminal_receipt(root, &receipt)?,
                                );
                            }
                            Ok(outcome_row) => {
                                let proof = build_series_outcome_proof(
                                    root,
                                    &live,
                                    &registration,
                                    &outcome_row,
                                    &response.raw_response,
                                )?;
                                let (receipt, capsule) = build_series_success_receipt_and_capsule(
                                    &registration,
                                    &proof,
                                )?;
                                add_counts(
                                    &mut counts,
                                    persist_series_success_acquisition(
                                        root,
                                        &response.raw_response,
                                        &proof,
                                        &receipt,
                                        &capsule,
                                    )?,
                                );
                            }
                        }
                    }
                }
            }
        }
        MomentumProspectiveOutcomeRunModeV4::OpenOutcome => {
            if initial_readiness != MomentumProspectiveOutcomeReadinessV4::ReadyForOutcomeOpening {
                return Err("V4 event-two opening readiness rejected".to_string());
            }
            let receipt = artifacts
                .receipt
                .as_ref()
                .ok_or_else(|| "V4 event-two outcome receipt unavailable".to_string())?;
            let proof = artifacts
                .proof
                .as_ref()
                .ok_or_else(|| "V4 event-two outcome proof unavailable".to_string())?;
            let capsule = artifacts
                .capsule
                .as_ref()
                .ok_or_else(|| "V4 event-two outcome capsule unavailable".to_string())?;
            let authorization = if let Some(stored) = artifacts.authorization.as_ref() {
                let derived = derive_series_opening_authorization(
                    root,
                    &live,
                    &registration,
                    receipt,
                    capsule,
                )?;
                if stored != &derived {
                    return Err("V4 event-two opening authorization conflict".to_string());
                }
                stored.clone()
            } else {
                let value = derive_series_opening_authorization(
                    root,
                    &live,
                    &registration,
                    receipt,
                    capsule,
                )?;
                add_counts(
                    &mut counts,
                    persist_series_outcome_pb(
                        root,
                        "opening_authorizations",
                        &value.authorization_digest,
                        &encode_series_opening_authorization(&value)?,
                        |bytes| Ok(decode_series_opening_authorization(bytes)?.authorization_digest),
                    )?,
                );
                let reopened = read_single(
                    &series_outcome_root(root).join("opening_authorizations"),
                    decode_series_opening_authorization,
                )?
                .ok_or_else(|| "V4 event-two opening authorization reopen failed".to_string())?;
                if reopened != value {
                    return Err("V4 event-two opening authorization reopen mismatch".to_string());
                }
                value
            };
            let bundle = if let Some(stored) = artifacts.opening_bundle.as_ref() {
                validate_series_opening_bundle_binding(&live, &authorization, capsule, stored)?;
                stored.clone()
            } else {
                let (event_close, outcome_close) =
                    reopen_event_two_opening_closes(root, &live, &registration, proof)?;
                safety.outcome_raw_loads = 1;
                let (label_status, label, return_bits) =
                    classify_label_v4_4(event_close, outcome_close)?;
                safety.label_derivations = 1;
                let evaluations =
                    build_series_evaluations(root, &live, proof, label_status, label, return_bits)?;
                safety.prediction_private_value_reads = evaluations.len();
                safety.evaluations = evaluations.len();
                safety.opening_attempts = 1;
                build_series_opening_bundle(
                    &live,
                    &authorization,
                    capsule,
                    label_status,
                    evaluations,
                )?
            };
            validate_series_opening_bundle_binding(&live, &authorization, capsule, &bundle)?;
            let bundle_evaluation_digests = bundle
                .participant_evaluations
                .iter()
                .map(|evaluation| evaluation.evaluation_digest.clone())
                .collect::<BTreeSet<_>>();
            if artifacts.evaluations.iter().any(|evaluation| {
                !bundle_evaluation_digests.contains(&evaluation.evaluation_digest)
            }) {
                return Err("V4 event-two partial evaluation conflict".to_string());
            }
            let opening_receipt = if let Some(stored) = artifacts.opening_receipt.as_ref() {
                let derived = build_series_opening_receipt(&authorization, &bundle)?;
                if stored != &derived {
                    return Err("V4 event-two opening receipt conflict".to_string());
                }
                stored.clone()
            } else {
                build_series_opening_receipt(&authorization, &bundle)?
            };
            let (ledger, eligibility, pause) = build_series_ledger_and_eligibility(
                root,
                &live,
                receipt,
                capsule,
                &authorization,
                &bundle,
                &prior_pause,
            )?;
            safety.ledger_appends = usize::from(artifacts.ledger.is_none());
            safety.eligibility_derivations = usize::from(artifacts.eligibility.is_none());
            add_counts(
                &mut counts,
                persist_series_opening(
                    root,
                    &authorization,
                    &bundle,
                    &opening_receipt,
                    &ledger,
                    &eligibility,
                    &pause,
                )?,
            );
        }
        MomentumProspectiveOutcomeRunModeV4::Status
        | MomentumProspectiveOutcomeRunModeV4::DryRun => {
            return Err("V4 read-only outcome mode reached mutation path".to_string());
        }
    }

    artifacts = reopen_series_outcome_artifacts(root)?;
    let final_readiness = series_outcome_readiness(
        observed_timestamp_ms,
        &registration,
        &artifacts,
        &live,
        &prior_pause,
    );
    if mode == MomentumProspectiveOutcomeRunModeV4::OpenOutcome
        && final_readiness != MomentumProspectiveOutcomeReadinessV4::OutcomeAlreadyOpened
    {
        return Err("V4 event-two opening completion rejected".to_string());
    }
    let protected_after = protected_live_identity(root)?;
    let historical_after = historical_identity_roots(root)?;
    let active_after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let protected_unchanged =
        protected_before == protected_after && historical_identities == historical_after;
    let status = build_prospective_outcome_status(
        &live,
        &registration,
        &artifacts,
        final_readiness,
        protected_before.clone(),
        historical_identities,
        prior_pause,
        protected_unchanged,
        active_before == active_after,
        safety,
        counts,
    )?;
    Ok(outcome_report(status, registration, artifacts))
}

#[allow(clippy::too_many_arguments)]
pub fn run_momentum_prospective_outcome_v4(
    root: &Path,
    snapshots: &[DataSnapshot],
    reservation: &ProtectedEvaluationReservationV1,
    provider_config: &UpbitHistoricalPilotConfigV0,
    observed_timestamp_ms: u64,
    mode: MomentumProspectiveOutcomeRunModeV4,
    network_allowed: bool,
    request_confirmed: bool,
    opening_confirmed: bool,
    requested_epoch: Option<u64>,
) -> Result<MomentumProspectiveOutcomeReportV4, String> {
    run_prospective_outcome_with_transport(
        root,
        snapshots,
        reservation,
        provider_config,
        observed_timestamp_ms,
        mode,
        network_allowed,
        request_confirmed,
        opening_confirmed,
        requested_epoch,
        fetch_upbit_learning_evidence_once_v1,
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        data::ReadOnlyProviderResponse,
        league::{HistoricalOhlcvRow, HistoricalReplayDataset},
    };

    const EVENT: u64 = 1_704_067_200_000;

    fn fixture_series() -> MomentumProspectiveSeriesV4 {
        let mut value = MomentumProspectiveSeriesV4 {
            series_version: SERIES_VERSION.into(),
            agent_id: AGENT_ID.into(),
            frozen_roster_digest: "roster".into(),
            participant_digests: vec!["p1".into(), "p2".into(), "p3".into()],
            parameter_digests: vec!["a1".into(), "a2".into(), "a3".into()],
            normalizer_digests: vec!["n1".into(), "n2".into(), "n3".into()],
            feature_policy_digest: "feature".into(),
            label_policy_digest: "label".into(),
            evaluation_policy_digest: "evaluation".into(),
            minimum_sample_policy_digest: "minimum".into(),
            provider_id: "upbit-public-v1".into(),
            market: "btc-crypto".into(),
            symbol: "KRW-BTC".into(),
            cadence_ms: DAILY_CADENCE_MS,
            context_row_count: 16,
            prediction_horizon: 1,
            first_event_ledger_entry_digest: "ledger".into(),
            first_event_opening_bundle_digest: "opening".into(),
            first_event_eligibility_digest: "eligibility".into(),
            continuation_policy:
                MomentumProspectiveContinuationPolicyV4::FixedDailyCadenceNextLegalEvent,
            maximum_open_epochs: 1,
            manual_network_confirmation_required: true,
            automatic_network_execution_forbidden: true,
            retraining_forbidden: true,
            participant_selection_forbidden: true,
            result_conditioned_continuation_forbidden: true,
            winner_selection_forbidden: true,
            ranking_forbidden: true,
            reward_application_forbidden: true,
            penalty_application_forbidden: true,
            chair_action_forbidden: true,
            trading_forbidden: true,
            protected_before_artifact_count: 1,
            protected_before_aggregate_digest: "protected".into(),
            active_agent_state_digest: "active".into(),
            series_digest: String::new(),
        };
        value.series_digest = series_digest(&value);
        value
    }

    fn fixture_adoption(
        series: &MomentumProspectiveSeriesV4,
    ) -> MomentumProspectiveSeriesAdoptionV4 {
        let mut value = MomentumProspectiveSeriesAdoptionV4 {
            adoption_version: ADOPTION_VERSION.into(),
            series_digest: series.series_digest.clone(),
            adopted_epoch_number: 1,
            adopted_event_timestamp_ms: EVENT - 2 * DAILY_CADENCE_MS,
            prediction_capsule_digest: "prediction-one".into(),
            outcome_capsule_digest: "outcome-one".into(),
            opening_bundle_digest: "opening-one".into(),
            evaluation_ledger_entry_digest: "ledger-one".into(),
            reward_eligibility_digest: "eligibility-one".into(),
            total_event_count: 1,
            scorable_event_count: 1,
            winner_selected: false,
            ranking_created: false,
            reward_applied: false,
            penalty_applied: false,
            chair_action_taken: false,
            adoption_digest: String::new(),
        };
        value.adoption_digest = adoption_digest(&value);
        value
    }

    fn fixture_gap(
        series: &MomentumProspectiveSeriesV4,
        adoption: &MomentumProspectiveSeriesAdoptionV4,
    ) -> MomentumProspectiveCandidateGapAuditV4 {
        derive_gap_audit(
            series,
            adoption,
            EVENT - DAILY_CADENCE_MS,
            EVENT + DAILY_CADENCE_MS / 2,
        )
        .expect("gap fixture")
    }

    fn fixture_delta(series: &MomentumProspectiveSeriesV4) -> MomentumCanonicalContextDeltaPlanV4 {
        let exact =
            timestamp_range(EVENT, series.context_row_count, DAILY_CADENCE_MS).expect("timestamps");
        let canonical_rows = exact[..15]
            .iter()
            .map(|timestamp| MomentumSeriesCanonicalRowRefV4 {
                timestamp_ms: *timestamp,
                raw_row_digest: format!("row-{timestamp}"),
                source_capsule_digest: "prior-input".into(),
                use_class: MomentumSeriesContextUseV4::ExistingCanonicalHistoricalRaw,
            })
            .collect::<Vec<_>>();
        let mut value = MomentumCanonicalContextDeltaPlanV4 {
            plan_version: DELTA_PLAN_VERSION.into(),
            series_digest: series.series_digest.clone(),
            epoch_number: 2,
            event_timestamp_ms: EVENT,
            exact_context_timestamp_ms: exact,
            canonical_rows,
            exact_missing_timestamp_ms: vec![EVENT],
            maximum_requests: 1,
            maximum_retries: 0,
            maximum_concurrency: 1,
            full_context_refetch_forbidden: true,
            prior_private_evaluation_accessed: false,
            missing_set_contiguous: true,
            plan_digest: String::new(),
        };
        value.plan_digest = delta_plan_digest(&value);
        value
    }

    fn fixture_registration(
        series: &MomentumProspectiveSeriesV4,
        delta: &MomentumCanonicalContextDeltaPlanV4,
    ) -> MomentumProspectiveEpochRegistrationV4 {
        let mut value = MomentumProspectiveEpochRegistrationV4 {
            registration_version: EPOCH_REGISTRATION_VERSION.into(),
            series_digest: series.series_digest.clone(),
            epoch_number: 2,
            previous_epoch_ledger_entry_digest: "ledger-one".into(),
            previous_epoch_opening_bundle_digest: "opening-one".into(),
            event_timestamp_ms: EVENT,
            registration_created_at_ms: EVENT + DAILY_CADENCE_MS / 2,
            input_finality_boundary_ms: EVENT + DAILY_CADENCE_MS,
            outcome_timestamp_ms: EVENT + DAILY_CADENCE_MS,
            outcome_finality_boundary_ms: EVENT + 2 * DAILY_CADENCE_MS,
            exact_context_timestamp_ms: delta.exact_context_timestamp_ms.clone(),
            exact_missing_timestamp_ms: delta.exact_missing_timestamp_ms.clone(),
            context_delta_plan_digest: delta.plan_digest.clone(),
            provider_id: series.provider_id.clone(),
            market: series.market.clone(),
            symbol: series.symbol.clone(),
            cadence: "1d".into(),
            maximum_input_requests: 1,
            maximum_input_retries: 0,
            maximum_input_concurrency: 1,
            maximum_response_bytes: 10_000,
            prior_private_evaluation_access_forbidden: true,
            parameter_update_forbidden: true,
            normalizer_refit_forbidden: true,
            outcome_access_forbidden: true,
            winner_selection_forbidden: true,
            ranking_forbidden: true,
            reward_application_forbidden: true,
            penalty_application_forbidden: true,
            chair_action_forbidden: true,
            trading_forbidden: true,
            registration_digest: String::new(),
        };
        value.registration_digest = epoch_registration_digest(&value);
        value
    }

    fn fixture_input_capsule(
        series: &MomentumProspectiveSeriesV4,
        registration: &MomentumProspectiveEpochRegistrationV4,
    ) -> MomentumProspectiveSeriesInputCapsuleV4 {
        let mut value = MomentumProspectiveSeriesInputCapsuleV4 {
            capsule_version: INPUT_CAPSULE_VERSION.into(),
            series_digest: series.series_digest.clone(),
            epoch_registration_digest: registration.registration_digest.clone(),
            context_delta_plan_digest: registration.context_delta_plan_digest.clone(),
            provider_id: registration.provider_id.clone(),
            request_attempt_count: 1,
            event_timestamp_ms: EVENT,
            exact_timestamp_ms: vec![EVENT],
            row_identity_digests: vec!["row-event".into()],
            normalized_dataset_digest: "dataset".into(),
            raw_response_digest: "raw".into(),
            outcome_row_present: false,
            labels_accessed: false,
            metrics_computed: false,
            prior_private_evaluation_accessed: false,
            credential_free: true,
            read_only: true,
            sanitized: true,
            capsule_digest: String::new(),
        };
        value.capsule_digest = input_capsule_digest(&value);
        value
    }

    fn fixture_receipt(
        series: &MomentumProspectiveSeriesV4,
        registration: &MomentumProspectiveEpochRegistrationV4,
        capsule: &MomentumProspectiveSeriesInputCapsuleV4,
    ) -> MomentumProspectiveSeriesInputReceiptV4 {
        let mut value = MomentumProspectiveSeriesInputReceiptV4 {
            receipt_version: INPUT_RECEIPT_VERSION.into(),
            series_digest: series.series_digest.clone(),
            epoch_registration_digest: registration.registration_digest.clone(),
            request_attempted: true,
            request_count: 1,
            retry_count: 0,
            transport_construction_count: 1,
            status: MomentumProspectiveSeriesInputStatusV4::EvidenceAcquired,
            http_status_class: Some("2xx".into()),
            returned_row_count: 1,
            verified_row_count: 1,
            raw_response_digest: Some(capsule.raw_response_digest.clone()),
            input_capsule_digest: Some(capsule.capsule_digest.clone()),
            terminal: true,
            receipt_digest: String::new(),
        };
        value.receipt_digest = input_receipt_digest(&value);
        value
    }

    fn fixture_context_proofs(
        series: &MomentumProspectiveSeriesV4,
        registration: &MomentumProspectiveEpochRegistrationV4,
        input_capsule: &MomentumProspectiveSeriesInputCapsuleV4,
    ) -> (
        MomentumSeriesContextUseProofV4,
        MomentumSeriesContextAssemblyProofV4,
    ) {
        let entries = registration
            .exact_context_timestamp_ms
            .iter()
            .map(|timestamp| {
                let mut value = MomentumSeriesContextUseEntryV4 {
                    timestamp_ms: *timestamp,
                    raw_row_digest: format!("row-{timestamp}"),
                    source_capsule_digest: if *timestamp == EVENT {
                        input_capsule.capsule_digest.clone()
                    } else {
                        "prior-input".into()
                    },
                    use_class: if *timestamp == EVENT {
                        MomentumSeriesContextUseV4::CurrentProspectiveEventInput
                    } else {
                        MomentumSeriesContextUseV4::ExistingCanonicalHistoricalRaw
                    },
                    feature_construction_allowed: true,
                    training_forbidden: true,
                    normalizer_fitting_forbidden: true,
                    label_use_forbidden: true,
                    metric_use_forbidden: true,
                    reward_use_forbidden: true,
                    participant_selection_forbidden: true,
                    entry_digest: String::new(),
                };
                value.entry_digest = context_use_entry_digest(&value);
                value
            })
            .collect::<Vec<_>>();
        let mut use_proof = MomentumSeriesContextUseProofV4 {
            proof_version: CONTEXT_USE_PROOF_VERSION.into(),
            series_digest: series.series_digest.clone(),
            epoch_registration_digest: registration.registration_digest.clone(),
            entries,
            prior_opening_bundle_used_as_raw_source: false,
            prior_private_scores_accessed: false,
            prior_label_used_as_feature: false,
            reward_eligibility_used_as_feature: false,
            proof_digest: String::new(),
        };
        use_proof.proof_digest = context_use_proof_digest(&use_proof);
        let mut assembly = MomentumSeriesContextAssemblyProofV4 {
            proof_version: CONTEXT_ASSEMBLY_VERSION.into(),
            series_digest: series.series_digest.clone(),
            epoch_registration_digest: registration.registration_digest.clone(),
            input_capsule_digest: input_capsule.capsule_digest.clone(),
            context_use_proof_digest: use_proof.proof_digest.clone(),
            exact_context_timestamp_ms: registration.exact_context_timestamp_ms.clone(),
            exact_row_digests: use_proof
                .entries
                .iter()
                .map(|entry| entry.raw_row_digest.clone())
                .collect(),
            exact_row_count: 16,
            strict_chronology_verified: true,
            all_row_digests_verified: true,
            event_timestamp_is_last: true,
            outcome_timestamp_absent: true,
            proof_digest: String::new(),
        };
        assembly.proof_digest = context_assembly_digest(&assembly);
        (use_proof, assembly)
    }

    fn fixture_seals(
        series: &MomentumProspectiveSeriesV4,
        registration: &MomentumProspectiveEpochRegistrationV4,
        receipt: &MomentumProspectiveSeriesInputReceiptV4,
        input_capsule: &MomentumProspectiveSeriesInputCapsuleV4,
        use_proof: &MomentumSeriesContextUseProofV4,
        assembly: &MomentumSeriesContextAssemblyProofV4,
    ) -> Vec<MomentumSeriesParticipantPredictionSealV4> {
        series
            .participant_digests
            .iter()
            .enumerate()
            .map(|(index, participant)| {
                let mut value = MomentumSeriesParticipantPredictionSealV4 {
                    seal_version: PREDICTION_SEAL_VERSION.into(),
                    series_digest: series.series_digest.clone(),
                    epoch_number: registration.epoch_number,
                    epoch_registration_digest: registration.registration_digest.clone(),
                    participant_digest: participant.clone(),
                    participant_role: [
                        "RawFeatureLogisticV4",
                        "RawFeatureInteractionLogisticV4",
                        "TrainingPrevalenceConstantV4",
                    ][index]
                        .to_string(),
                    event_timestamp_ms: EVENT,
                    input_receipt_digest: receipt.receipt_digest.clone(),
                    input_capsule_digest: input_capsule.capsule_digest.clone(),
                    context_use_proof_digest: use_proof.proof_digest.clone(),
                    context_assembly_proof_digest: assembly.proof_digest.clone(),
                    feature_identity_digest: "feature-identity".into(),
                    prediction_probability_bits: (index as f32 / 10.0 + 0.4).to_bits(),
                    prediction_digest: format!("prediction-{index}"),
                    participant_identity_verified: true,
                    parameter_updates: 0,
                    normalizer_refits: 0,
                    prior_score_reads: 0,
                    outcome_access_count: 0,
                    seal_digest: String::new(),
                };
                value.seal_digest = prediction_seal_digest(&value);
                value
            })
            .collect()
    }

    fn fixture_prediction_chain() -> (
        MomentumProspectiveSeriesV4,
        MomentumProspectiveSeriesAdoptionV4,
        MomentumCanonicalContextDeltaPlanV4,
        MomentumProspectiveEpochRegistrationV4,
        MomentumProspectiveSeriesInputReceiptV4,
        MomentumProspectiveSeriesInputCapsuleV4,
        MomentumSeriesContextUseProofV4,
        MomentumSeriesContextAssemblyProofV4,
        Vec<MomentumSeriesParticipantPredictionSealV4>,
        MomentumProspectiveSeriesPredictionCapsuleV4,
        MomentumProspectiveSeriesJournalEntryV4,
        MomentumProspectiveSeriesOutcomePlanV4,
    ) {
        let series = fixture_series();
        let adoption = fixture_adoption(&series);
        let delta = fixture_delta(&series);
        let registration = fixture_registration(&series, &delta);
        let input_capsule = fixture_input_capsule(&series, &registration);
        let receipt = fixture_receipt(&series, &registration, &input_capsule);
        let (use_proof, assembly) = fixture_context_proofs(&series, &registration, &input_capsule);
        let seals = fixture_seals(
            &series,
            &registration,
            &receipt,
            &input_capsule,
            &use_proof,
            &assembly,
        );
        let mut prediction_capsule = MomentumProspectiveSeriesPredictionCapsuleV4 {
            capsule_version: PREDICTION_CAPSULE_VERSION.into(),
            series_digest: series.series_digest.clone(),
            epoch_registration_digest: registration.registration_digest.clone(),
            event_timestamp_ms: EVENT,
            input_receipt_digest: receipt.receipt_digest.clone(),
            input_capsule_digest: input_capsule.capsule_digest.clone(),
            context_assembly_proof_digest: assembly.proof_digest.clone(),
            participant_seal_digests: seals
                .iter()
                .map(|value| value.seal_digest.clone())
                .collect(),
            participant_prediction_digests: seals
                .iter()
                .map(|value| value.prediction_digest.clone())
                .collect(),
            probabilities_hidden: true,
            labels_hidden: true,
            prior_scores_accessed: false,
            outcome_accessed: false,
            metrics_computed: false,
            winner_selected: false,
            ranking_created: false,
            reward_applied: false,
            penalty_applied: false,
            chair_action_taken: false,
            capsule_digest: String::new(),
        };
        prediction_capsule.capsule_digest = prediction_capsule_digest(&prediction_capsule);
        let mut journal = MomentumProspectiveSeriesJournalEntryV4 {
            journal_version: JOURNAL_VERSION.into(),
            series_digest: series.series_digest.clone(),
            epoch_number: 2,
            event_one_adoption_digest: adoption.adoption_digest.clone(),
            previous_epoch_ledger_entry_digest: adoption.evaluation_ledger_entry_digest.clone(),
            context_delta_plan_digest: registration.context_delta_plan_digest.clone(),
            event_timestamp_ms: EVENT,
            registration_created_at_ms: registration.registration_created_at_ms,
            input_finality_boundary_ms: registration.input_finality_boundary_ms,
            input_receipt_digest: receipt.receipt_digest.clone(),
            input_capsule_digest: input_capsule.capsule_digest.clone(),
            context_assembly_proof_digest: assembly.proof_digest.clone(),
            prediction_capsule_digest: prediction_capsule.capsule_digest.clone(),
            participant_seal_digests: prediction_capsule.participant_seal_digests.clone(),
            participant_prediction_digests: prediction_capsule
                .participant_prediction_digests
                .clone(),
            deterministic_fixed_cadence_selection: true,
            prior_event_scores_read: false,
            prior_event_correctness_read: false,
            registration_preceded_input_finality: true,
            input_acquisition_preceded_prediction: true,
            prediction_preceded_outcome_access: true,
            outcome_stage_locked: true,
            winner_selected: false,
            ranking_created: false,
            reward_applied: false,
            penalty_applied: false,
            chair_action_taken: false,
            trading_action_taken: false,
            entry_digest: String::new(),
        };
        journal.entry_digest = journal_entry_digest(&journal);
        let mut outcome_plan = MomentumProspectiveSeriesOutcomePlanV4 {
            plan_version: OUTCOME_PLAN_VERSION.into(),
            series_digest: series.series_digest.clone(),
            epoch_registration_digest: registration.registration_digest.clone(),
            prediction_capsule_digest: prediction_capsule.capsule_digest.clone(),
            event_timestamp_ms: EVENT,
            prediction_horizon: 1,
            required_outcome_timestamp_ms: vec![EVENT + DAILY_CADENCE_MS],
            outcome_finality_boundary_ms: EVENT + 2 * DAILY_CADENCE_MS,
            maximum_outcome_requests: 1,
            maximum_outcome_retries: 0,
            outcome_acquisition_count: 0,
            outcome_opening_count: 0,
            labels_hidden_until_opening: true,
            one_time_opening_required: true,
            outcome_stage_locked_before_finality: true,
            plan_digest: String::new(),
        };
        outcome_plan.plan_digest = outcome_plan_digest(&outcome_plan);
        (
            series,
            adoption,
            delta,
            registration,
            receipt,
            input_capsule,
            use_proof,
            assembly,
            seals,
            prediction_capsule,
            journal,
            outcome_plan,
        )
    }

    fn fixture_row() -> HistoricalOhlcvRow {
        HistoricalOhlcvRow {
            symbol: "KRW-BTC".into(),
            timestamp_ms: EVENT,
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
            volume: 2.0,
            trade_value: Some(205.0),
        }
    }

    fn fixture_transport(
        registration: &MomentumProspectiveEpochRegistrationV4,
    ) -> (ReadOnlyProviderRequest, LearningEvidenceTransportResponseV1) {
        let request = build_provider_request(registration).expect("request");
        let raw = br#"[{"market":"KRW-BTC","candle_date_time_utc":"2024-01-01T00:00:00","opening_price":100.0,"high_price":110.0,"low_price":90.0,"trade_price":105.0,"candle_acc_trade_volume":2.0,"candle_acc_trade_price":205.0}]"#.to_vec();
        let response = LearningEvidenceTransportResponseV1 {
            http_status_class: "2xx".into(),
            raw_response: raw.clone(),
            response: ReadOnlyProviderResponse {
                request_id: request.request_id.clone(),
                provider_id: registration.provider_id.clone(),
                fetched_at_ms: EVENT + DAILY_CADENCE_MS,
                content_type: "application/x-soma-normalized-dataset".into(),
                all_rows_finalized: true,
                normalized_dataset: HistoricalReplayDataset {
                    symbol: registration.symbol.clone(),
                    rows: vec![fixture_row()],
                    source: "upbit-approved-readonly-daily".into(),
                    reason_codes: vec![],
                },
                reported_content_bytes: raw.len(),
                reason_codes: vec![],
            },
        };
        (request, response)
    }

    pub(crate) fn deterministic_sealed_epoch_two_report_fixture_v4()
    -> MomentumProspectiveSeriesReportV4 {
        let chain = fixture_prediction_chain();
        let gap = fixture_gap(&chain.0, &chain.1);
        let status = build_status(
            &chain.1,
            &gap,
            &chain.2,
            &chain.3,
            MomentumProspectiveEpochReadinessV4::PredictionAlreadySealed,
            Some(&chain.4),
            Some(&chain.5),
            Some(&chain.7),
            Some(&chain.9),
            Some(&chain.10),
            Some(&chain.11),
            MomentumRewardEligibilityStatusV4_4::IneligibleMinimumSamples,
            true,
            true,
            idle_safety_counters(),
        )
        .expect("live status fixture");
        MomentumProspectiveSeriesReportV4 {
            status,
            series: chain.0,
            event_one_adoption: chain.1,
            candidate_gap_audit: gap,
            context_delta_plan: chain.2,
            epoch_registration: chain.3,
            input_receipt: Some(chain.4),
            input_capsule: Some(chain.5),
            context_use_proof: Some(chain.6),
            context_assembly_proof: Some(chain.7),
            prediction_capsule: Some(chain.9),
            journal_entry: Some(chain.10),
            outcome_plan: Some(chain.11),
            artifacts_written: 0,
            duplicate_artifact_count: 0,
        }
    }

    fn fixture_live_report() -> MomentumProspectiveSeriesReportV4 {
        deterministic_sealed_epoch_two_report_fixture_v4()
    }

    fn fixture_outcome_registration() -> MomentumOutcomeAcquisitionRegistrationV4_4 {
        let live = fixture_live_report();
        let receipt = live.input_receipt.as_ref().expect("input receipt");
        let capsule = live.input_capsule.as_ref().expect("input capsule");
        let context = live.context_use_proof.as_ref().expect("context");
        let prediction = live.prediction_capsule.as_ref().expect("prediction");
        let journal = live.journal_entry.as_ref().expect("journal");
        let plan = live.outcome_plan.as_ref().expect("plan");
        let mut value = MomentumOutcomeAcquisitionRegistrationV4_4 {
            registration_version: REGISTRATION_VERSION.into(),
            agent_id: live.series.agent_id.clone(),
            lifecycle_digest: live.series.series_digest.clone(),
            evaluation_registration_digest: live.epoch_registration.registration_digest.clone(),
            roster_digest: live.series.frozen_roster_digest.clone(),
            input_receipt_digest: receipt.receipt_digest.clone(),
            input_capsule_digest: capsule.capsule_digest.clone(),
            context_usage_ledger_digest: context.proof_digest.clone(),
            prediction_capsule_digest: prediction.capsule_digest.clone(),
            prediction_journal_digest: journal.entry_digest.clone(),
            outcome_plan_digest: plan.plan_digest.clone(),
            event_timestamp_ms: plan.event_timestamp_ms,
            required_outcome_timestamp_ms: plan.required_outcome_timestamp_ms.clone(),
            outcome_finality_boundary_ms: plan.outcome_finality_boundary_ms,
            provider_id: "upbit".into(),
            market: "btc_crypto".into(),
            symbol: live.epoch_registration.symbol.clone(),
            cadence: live.epoch_registration.cadence.clone(),
            exact_expected_timestamp_ms: plan.required_outcome_timestamp_ms.clone(),
            expected_row_count: 1,
            request_to_timestamp_ms: plan.outcome_finality_boundary_ms,
            maximum_requests: 1,
            maximum_concurrency: 1,
            maximum_retries: 0,
            maximum_response_bytes: 10_000,
            credential_free_required: true,
            read_only_required: true,
            labels_must_remain_unopened: true,
            metric_computation_forbidden: true,
            winner_selection_forbidden: true,
            reward_application_forbidden: true,
            registration_digest: String::new(),
        };
        value.registration_digest = registration_digest(&value);
        validate_series_outcome_registration(&value).expect("outcome registration fixture");
        value
    }

    fn fixture_outcome_proof(
        registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    ) -> MomentumOutcomeRowIdentityProofV4_4 {
        let mut value = MomentumOutcomeRowIdentityProofV4_4 {
            proof_version: ROW_PROOF_VERSION.into(),
            registration_digest: registration.registration_digest.clone(),
            prediction_capsule_digest: registration.prediction_capsule_digest.clone(),
            input_capsule_digest: registration.input_capsule_digest.clone(),
            event_timestamp_ms: registration.event_timestamp_ms,
            outcome_timestamp_ms: registration.exact_expected_timestamp_ms[0],
            input_event_row_digest: "input-row".into(),
            outcome_row_digest: "outcome-row".into(),
            raw_input_response_digest: "input-raw".into(),
            raw_outcome_response_digest: "outcome-raw".into(),
            exact_timestamp_verified: true,
            strict_single_row_verified: true,
            finalized: true,
            sanitized: true,
            credential_free: true,
            read_only: true,
            proof_digest: String::new(),
        };
        value.proof_digest = row_proof_digest(&value);
        validate_series_outcome_proof(&value).expect("outcome proof fixture");
        value
    }

    fn fixture_outcome_receipt_capsule() -> (
        MomentumOutcomeAcquisitionRegistrationV4_4,
        MomentumOutcomeRowIdentityProofV4_4,
        MomentumOutcomeAcquisitionReceiptV4_4,
        MomentumSealedOutcomeCapsuleV4_4,
    ) {
        let registration = fixture_outcome_registration();
        let proof = fixture_outcome_proof(&registration);
        let (receipt, capsule) =
            build_series_success_receipt_and_capsule(&registration, &proof).expect("success");
        (registration, proof, receipt, capsule)
    }

    fn fixture_evaluations(
        proof: &MomentumOutcomeRowIdentityProofV4_4,
        label_status: MomentumProspectiveLabelStatusV4_4,
    ) -> Vec<MomentumParticipantProspectiveEvaluationV4_4> {
        [
            ("p1", "RawFeatureLogisticV4", 0.4_f32),
            ("p2", "RawFeatureInteractionLogisticV4", 0.5_f32),
            ("p3", "TrainingPrevalenceConstantV4", 0.6_f32),
        ]
        .into_iter()
        .map(|(participant, role, probability)| {
            build_participant_evaluation_v4_4(
                participant,
                role,
                &format!("seal-{participant}"),
                &format!("prediction-{participant}"),
                probability.to_bits(),
                EVENT,
                proof,
                label_status,
                (label_status == MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome)
                    .then_some(true),
                0.02_f64.to_bits(),
            )
            .expect("evaluation fixture")
        })
        .collect()
    }

    fn fixture_opening_authorization(
        registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
        receipt: &MomentumOutcomeAcquisitionReceiptV4_4,
        capsule: &MomentumSealedOutcomeCapsuleV4_4,
    ) -> MomentumOutcomeOpeningAuthorizationV4_4 {
        let live = fixture_live_report();
        let prediction = live.prediction_capsule.expect("prediction");
        let journal = live.journal_entry.expect("journal");
        let mut value = MomentumOutcomeOpeningAuthorizationV4_4 {
            authorization_version: OPENING_AUTHORIZATION_VERSION.into(),
            outcome_registration_digest: registration.registration_digest.clone(),
            outcome_receipt_digest: receipt.receipt_digest.clone(),
            outcome_capsule_digest: capsule.capsule_digest.clone(),
            prediction_capsule_digest: prediction.capsule_digest,
            prediction_journal_digest: journal.entry_digest,
            participant_seal_digests: prediction.participant_seal_digests,
            participant_prediction_digests: prediction.participant_prediction_digests,
            feature_policy_digest: live.series.feature_policy_digest,
            label_policy_digest: frozen_label_policy_digest(),
            evaluation_policy_digest: evaluation_policy_digest(),
            opening_attempt_count_before: 0,
            opened_event_count_before: 0,
            explicit_owner_authorization: true,
            one_time_only: true,
            winner_selection_forbidden: true,
            ranking_forbidden: true,
            reward_application_forbidden: true,
            penalty_application_forbidden: true,
            chair_action_forbidden: true,
            voice_mutation_forbidden: true,
            promotion_forbidden: true,
            trading_forbidden: true,
            authorization_digest: String::new(),
        };
        value.authorization_digest = authorization_digest(&value);
        validate_opening_authorization_shape(&value).expect("authorization fixture");
        value
    }

    fn fixture_opening_bundle(
        authorization: &MomentumOutcomeOpeningAuthorizationV4_4,
        capsule: &MomentumSealedOutcomeCapsuleV4_4,
        proof: &MomentumOutcomeRowIdentityProofV4_4,
        label_status: MomentumProspectiveLabelStatusV4_4,
    ) -> MomentumOutcomeOpeningBundleV4_4 {
        build_series_opening_bundle(
            &fixture_live_report(),
            authorization,
            capsule,
            label_status,
            fixture_evaluations(proof, label_status),
        )
        .expect("opening bundle fixture")
    }

    fn fixture_completion_artifacts(
        label_status: MomentumProspectiveLabelStatusV4_4,
    ) -> (
        MomentumProspectiveSeriesLedgerEntryV4,
        MomentumProspectiveSeriesEligibilityReceiptV4,
        LiveProspectiveContinuationPauseV2,
    ) {
        let live = fixture_live_report();
        let (registration, proof, receipt, capsule) = fixture_outcome_receipt_capsule();
        let authorization = fixture_opening_authorization(&registration, &receipt, &capsule);
        let bundle = fixture_opening_bundle(&authorization, &capsule, &proof, label_status);
        let mut ledger = MomentumProspectiveSeriesLedgerEntryV4 {
            ledger_version: SERIES_LEDGER_VERSION.into(),
            previous_event_ledger_entry_digest: live
                .event_one_adoption
                .evaluation_ledger_entry_digest,
            series_digest: live.series.series_digest,
            epoch_registration_digest: live.epoch_registration.registration_digest,
            input_receipt_digest: live.input_receipt.expect("input receipt").receipt_digest,
            input_capsule_digest: live.input_capsule.expect("input capsule").capsule_digest,
            context_proof_digest: live.context_use_proof.expect("context").proof_digest,
            participant_seal_digests: live
                .prediction_capsule
                .as_ref()
                .expect("prediction")
                .participant_seal_digests
                .clone(),
            prediction_capsule_digest: live
                .prediction_capsule
                .as_ref()
                .expect("prediction")
                .capsule_digest
                .clone(),
            prediction_journal_digest: live.journal_entry.expect("journal").entry_digest,
            outcome_plan_digest: live.outcome_plan.expect("plan").plan_digest,
            outcome_receipt_digest: receipt.receipt_digest,
            outcome_capsule_digest: capsule.capsule_digest,
            opening_authorization_digest: authorization.authorization_digest,
            opening_bundle_digest: bundle.bundle_digest,
            label_status,
            participant_evaluation_digests: bundle
                .participant_evaluations
                .iter()
                .map(|evaluation| evaluation.evaluation_digest.clone())
                .collect(),
            total_event_count_after: 2,
            scorable_event_count_after: 1 + usize::from(
                label_status == MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome,
            ),
            winner_selected: false,
            ranking_created: false,
            reward_applied: false,
            penalty_applied: false,
            chair_action_taken: false,
            trading_action_taken: false,
            entry_digest: String::new(),
        };
        ledger.entry_digest = series_ledger_entry_digest(&ledger);
        validate_series_ledger_entry(&ledger).expect("ledger fixture");
        let mut eligibility = MomentumProspectiveSeriesEligibilityReceiptV4 {
            receipt_version: SERIES_ELIGIBILITY_VERSION.into(),
            event_one_eligibility_digest: "event-one-eligibility".into(),
            event_two_ledger_entry_digest: ledger.entry_digest.clone(),
            participant_roles: vec![
                "RawFeatureLogisticV4".into(),
                "RawFeatureInteractionLogisticV4".into(),
                "TrainingPrevalenceConstantV4".into(),
            ],
            completed_event_count: ledger.total_event_count_after,
            scorable_event_count: ledger.scorable_event_count_after,
            minimum_sample_gate: 3,
            status: MomentumProspectiveSeriesEligibilityV4::IneligibleMinimumSamples,
            integrity_verified: true,
            reward_application_count: 0,
            penalty_application_count: 0,
            receipt_digest: String::new(),
        };
        eligibility.receipt_digest = series_eligibility_digest(&eligibility);
        validate_series_eligibility(&eligibility).expect("eligibility fixture");
        let mut pause = LiveProspectiveContinuationPauseV2 {
            pause_version: COMPLETED_PAUSE_VERSION.into(),
            policy: LiveProspectiveContinuationPolicyV2::PausedAfterCompletedEpochTwo,
            prior_pause_digest: "prior-pause".into(),
            event_two_ledger_entry_digest: ledger.entry_digest.clone(),
            completed_event_count: ledger.total_event_count_after,
            scorable_event_count: ledger.scorable_event_count_after,
            eligibility_receipt_digest: eligibility.receipt_digest.clone(),
            epoch_three_registered: false,
            historical_challenger_research_prioritized: true,
            scheduler_count: 0,
            automatic_registration_count: 0,
            network_authority_count: 0,
            pause_digest: String::new(),
        };
        pause.pause_digest = completed_pause_digest(&pause);
        validate_completed_pause(&pause).expect("pause fixture");
        (ledger, eligibility, pause)
    }

    fn fixture_outcome_status() -> MomentumProspectiveOutcomeStatusV4 {
        build_prospective_outcome_status(
            &fixture_live_report(),
            &fixture_outcome_registration(),
            &SeriesOutcomeArtifacts::default(),
            MomentumProspectiveOutcomeReadinessV4::ReadyForOutcomeAcquisition,
            (1, "protected".into()),
            (
                "historical".into(),
                "qualified-six".into(),
                "diagnostics".into(),
            ),
            "prior-pause".into(),
            true,
            true,
            MomentumProspectiveOutcomeSafetyCountersV4::default(),
            (0, 0),
        )
        .expect("outcome status fixture")
    }

    #[test]
    fn sprint89_01_prior_contract_invariants_round_trip() {
        let value = fixture_series();
        assert_eq!(
            decode_series(&encode_series(&value).unwrap()).unwrap(),
            value
        );
    }

    #[test]
    fn sprint89_02_protected_artifact_identity_is_stable() {
        let values = vec![(PathBuf::from("a.pb"), vec![1, 2, 3])];
        assert_eq!(
            protected_aggregate_digest(&values),
            protected_aggregate_digest(&values)
        );
    }

    #[test]
    fn sprint89_03_event_one_completed_identity_reopens() {
        let series = fixture_series();
        let value = fixture_adoption(&series);
        assert_eq!(
            decode_adoption(&encode_adoption(&value).unwrap()).unwrap(),
            value
        );
    }

    #[test]
    fn sprint89_04_event_one_adoption_is_additive() {
        let series = fixture_series();
        let value = fixture_adoption(&series);
        assert_eq!(value.total_event_count, 1);
        assert!(!value.reward_applied && !value.penalty_applied);
    }

    #[test]
    fn sprint89_05_series_binds_roster_and_policies() {
        let value = fixture_series();
        assert_eq!(value.participant_digests.len(), 3);
        assert!(validate_series(&value).is_ok());
    }

    #[test]
    fn sprint89_06_private_scores_cannot_affect_continuation() {
        let (mut use_proof, _) = {
            let series = fixture_series();
            let delta = fixture_delta(&series);
            let registration = fixture_registration(&series, &delta);
            let capsule = fixture_input_capsule(&series, &registration);
            fixture_context_proofs(&series, &registration, &capsule)
        };
        use_proof.prior_private_scores_accessed = true;
        use_proof.proof_digest = context_use_proof_digest(&use_proof);
        assert!(validate_context_use_proof(&use_proof).is_err());
    }

    #[test]
    fn sprint89_07_result_cannot_change_membership() {
        let mut value = fixture_series();
        value.participant_digests.pop();
        value.series_digest = series_digest(&value);
        assert!(validate_series(&value).is_err());
    }

    #[test]
    fn sprint89_08_adjacent_finalized_candidate_cannot_be_backdated() {
        let series = fixture_series();
        let adoption = fixture_adoption(&series);
        let audit = fixture_gap(&series, &adoption);
        assert!(audit.registration_after_input_finality);
        assert_eq!(
            audit.canonical_disposition,
            MomentumProspectiveCandidateDispositionV4::SkippedPriorOutcomeAlreadyOpened
        );
    }

    #[test]
    fn sprint89_09_skipped_candidate_is_not_model_failure() {
        let series = fixture_series();
        let audit = fixture_gap(&series, &fixture_adoption(&series));
        assert!(!audit.counted_as_model_failure && !audit.reward_or_penalty_consequence);
    }

    #[test]
    fn sprint89_10_next_event_derives_from_cadence_and_time() {
        let series = fixture_series();
        let adoption = fixture_adoption(&series);
        assert_eq!(
            derive_next_legal_event(
                &series,
                &adoption,
                EVENT - DAILY_CADENCE_MS,
                EVENT + DAILY_CADENCE_MS / 2,
            )
            .unwrap(),
            EVENT
        );
    }

    #[test]
    fn sprint89_11_registration_precedes_input_finality() {
        let series = fixture_series();
        let delta = fixture_delta(&series);
        let mut value = fixture_registration(&series, &delta);
        value.registration_created_at_ms = value.input_finality_boundary_ms;
        value.registration_digest = epoch_registration_digest(&value);
        assert!(validate_epoch_registration(&value).is_err());
    }

    #[test]
    fn sprint89_12_exactly_one_open_epoch_is_allowed() {
        assert_eq!(fixture_series().maximum_open_epochs, 1);
    }

    #[test]
    fn sprint89_13_duplicate_epoch_number_rejects() {
        let series = fixture_series();
        let delta = fixture_delta(&series);
        let first = fixture_registration(&series, &delta);
        let mut duplicate = first.clone();
        duplicate.event_timestamp_ms += DAILY_CADENCE_MS;
        duplicate.input_finality_boundary_ms += DAILY_CADENCE_MS;
        duplicate.outcome_timestamp_ms += DAILY_CADENCE_MS;
        duplicate.outcome_finality_boundary_ms += DAILY_CADENCE_MS;
        duplicate.registration_digest = epoch_registration_digest(&duplicate);
        assert!(ensure_same(Some(&first), &duplicate, "epoch registration").is_err());
    }

    #[test]
    fn sprint89_14_duplicate_event_timestamp_rejects() {
        let mut delta = fixture_delta(&fixture_series());
        delta.exact_context_timestamp_ms[14] = EVENT;
        delta.plan_digest = delta_plan_digest(&delta);
        assert!(validate_delta_plan(&delta).is_err());
    }

    #[test]
    fn sprint89_15_context_has_exactly_sixteen_timestamps() {
        let delta = fixture_delta(&fixture_series());
        assert_eq!(delta.exact_context_timestamp_ms.len(), 16);
        assert!(validate_delta_plan(&delta).is_ok());
    }

    #[test]
    fn sprint89_16_opened_outcome_raw_is_context_only() {
        let mut value = fixture_delta(&fixture_series()).canonical_rows[14].clone();
        value.use_class = MomentumSeriesContextUseV4::PriorOpenedOutcomeRawContext;
        assert_eq!(
            decode_row_ref(&encode_row_ref(&value).unwrap()).unwrap(),
            value
        );
    }

    #[test]
    fn sprint89_17_opening_bundle_cannot_supply_raw_values() {
        let (mut proof, _) = {
            let series = fixture_series();
            let delta = fixture_delta(&series);
            let registration = fixture_registration(&series, &delta);
            let capsule = fixture_input_capsule(&series, &registration);
            fixture_context_proofs(&series, &registration, &capsule)
        };
        proof.prior_opening_bundle_used_as_raw_source = true;
        proof.proof_digest = context_use_proof_digest(&proof);
        assert!(validate_context_use_proof(&proof).is_err());
    }

    #[test]
    fn sprint89_18_prior_scores_cannot_be_read_for_prediction() {
        let chain = fixture_prediction_chain();
        let mut seal = chain.8[0].clone();
        seal.prior_score_reads = 1;
        seal.seal_digest = prediction_seal_digest(&seal);
        assert!(validate_prediction_seal(&seal).is_err());
    }

    #[test]
    fn sprint89_19_missing_set_is_derived_from_canonical_rows() {
        let delta = fixture_delta(&fixture_series());
        assert_eq!(delta.exact_missing_timestamp_ms, [EVENT]);
    }

    #[test]
    fn sprint89_20_existing_rows_are_not_requested_again() {
        let chain = fixture_prediction_chain();
        let request = build_provider_request(&chain.3).unwrap();
        assert_eq!(request.lookback.bars, 1);
        assert_eq!(request.lookback.start_timestamp_ms, Some(EVENT));
    }

    #[test]
    fn sprint89_21_noncontiguous_missing_set_blocks() {
        let mut delta = fixture_delta(&fixture_series());
        delta.exact_missing_timestamp_ms = vec![EVENT - 2 * DAILY_CADENCE_MS, EVENT];
        delta.plan_digest = delta_plan_digest(&delta);
        assert!(validate_delta_plan(&delta).is_err());
    }

    #[test]
    fn sprint89_22_prefinality_execution_has_zero_network() {
        let chain = fixture_prediction_chain();
        assert_eq!(
            readiness(EVENT, &chain.3, None, None),
            MomentumProspectiveEpochReadinessV4::RegisteredAwaitingInputFinality
        );
        assert_eq!(idle_safety_counters().network_request_attempts, 0);
    }

    #[test]
    fn sprint89_23_postfinality_allows_one_request() {
        let chain = fixture_prediction_chain();
        assert_eq!(
            readiness(EVENT + DAILY_CADENCE_MS, &chain.3, None, None),
            MomentumProspectiveEpochReadinessV4::ReadyForInputAcquisition
        );
        assert_eq!(chain.3.maximum_input_requests, 1);
    }

    #[test]
    fn sprint89_24_outcome_finality_expiry_blocks_prediction() {
        let chain = fixture_prediction_chain();
        assert_eq!(
            readiness(EVENT + 2 * DAILY_CADENCE_MS, &chain.3, None, None),
            MomentumProspectiveEpochReadinessV4::PredictionSealWindowExpired
        );
    }

    #[test]
    fn sprint89_25_retries_remain_zero() {
        let chain = fixture_prediction_chain();
        assert_eq!(chain.3.maximum_input_retries, 0);
        assert_eq!(chain.4.retry_count, 0);
    }

    #[test]
    fn sprint89_26_exact_missing_timestamp_is_enforced() {
        let chain = fixture_prediction_chain();
        let (request, mut response) = fixture_transport(&chain.3);
        response.response.normalized_dataset.rows[0].timestamp_ms += DAILY_CADENCE_MS;
        assert!(validate_input_response(&chain.3, &request, &response).is_err());
    }

    #[test]
    fn sprint89_27_wrong_duplicate_missing_extra_unfinished_rows_reject() {
        let chain = fixture_prediction_chain();
        let (request, valid) = fixture_transport(&chain.3);
        assert!(validate_input_response(&chain.3, &request, &valid).is_ok());
        let mut duplicate = valid.clone();
        duplicate
            .response
            .normalized_dataset
            .rows
            .push(fixture_row());
        let mut missing = valid.clone();
        missing.response.normalized_dataset.rows.clear();
        let mut extra = valid.clone();
        let mut row = fixture_row();
        row.timestamp_ms += DAILY_CADENCE_MS;
        extra.response.normalized_dataset.rows.push(row);
        let mut unfinished = valid.clone();
        unfinished.response.all_rows_finalized = false;
        assert!(
            [&duplicate, &missing, &extra, &unfinished]
                .iter()
                .all(|response| validate_input_response(&chain.3, &request, response).is_err())
        );
    }

    #[test]
    fn sprint89_28_context_assembly_verifies_all_digests() {
        let chain = fixture_prediction_chain();
        assert_eq!(chain.7.exact_row_count, 16);
        assert_eq!(chain.7.exact_row_digests.len(), 16);
        assert!(validate_context_assembly(&chain.7).is_ok());
    }

    #[test]
    fn sprint89_29_outcome_timestamp_is_absent_from_context() {
        let chain = fixture_prediction_chain();
        assert!(
            !chain
                .3
                .exact_context_timestamp_ms
                .contains(&chain.3.outcome_timestamp_ms)
        );
        assert!(chain.7.outcome_timestamp_absent);
    }

    #[test]
    fn sprint89_30_parameters_remain_unchanged() {
        let series = fixture_series();
        let before = series.parameter_digests.clone();
        assert_eq!(before, series.parameter_digests);
        assert!(series.retraining_forbidden);
    }

    #[test]
    fn sprint89_31_normalizers_remain_unchanged() {
        let series = fixture_series();
        let before = series.normalizer_digests.clone();
        assert_eq!(before, series.normalizer_digests);
        assert!(series.retraining_forbidden);
    }

    #[test]
    fn sprint89_32_all_participants_share_identical_context() {
        let chain = fixture_prediction_chain();
        assert!(
            chain
                .8
                .iter()
                .all(|seal| seal.context_assembly_proof_digest == chain.7.proof_digest)
        );
    }

    #[test]
    fn sprint89_33_exactly_three_prediction_seals_are_required() {
        let mut capsule = fixture_prediction_chain().9;
        capsule.participant_seal_digests.pop();
        capsule.capsule_digest = prediction_capsule_digest(&capsule);
        assert!(validate_prediction_capsule(&capsule).is_err());
    }

    #[test]
    fn sprint89_34_probabilities_remain_private() {
        let capsule = fixture_prediction_chain().9;
        let public = serde_json::to_string(&capsule).unwrap();
        assert!(capsule.probabilities_hidden);
        assert!(!public.contains("prediction_probability_bits"));
    }

    #[test]
    fn sprint89_35_prediction_precedes_outcome_access() {
        let chain = fixture_prediction_chain();
        assert!(chain.10.prediction_preceded_outcome_access);
        assert!(!chain.9.outcome_accessed);
    }

    #[test]
    fn sprint89_36_outcome_plan_derives_horizon_one() {
        let plan = fixture_prediction_chain().11;
        assert_eq!(plan.prediction_horizon, 1);
        assert_eq!(
            plan.required_outcome_timestamp_ms,
            [EVENT + DAILY_CADENCE_MS]
        );
        assert!(validate_outcome_plan(&plan).is_ok());
    }

    #[test]
    fn sprint89_37_event_two_outcome_acquisition_remains_zero() {
        let plan = fixture_prediction_chain().11;
        assert_eq!(plan.outcome_acquisition_count, 0);
        assert_eq!(plan.outcome_opening_count, 0);
    }

    #[test]
    fn sprint89_38_event_one_ledger_identity_remains_unchanged() {
        let adoption = fixture_adoption(&fixture_series());
        assert_eq!(adoption.evaluation_ledger_entry_digest, "ledger-one");
        assert_eq!(adoption.total_event_count, 1);
    }

    #[test]
    fn sprint89_39_event_one_eligibility_identity_remains_unchanged() {
        let adoption = fixture_adoption(&fixture_series());
        assert_eq!(adoption.reward_eligibility_digest, "eligibility-one");
        assert_eq!(adoption.scorable_event_count, 1);
    }

    #[test]
    fn sprint89_40_all_authority_counters_remain_zero() {
        let counters = idle_safety_counters();
        assert!(validate_safety_counters(&counters).is_ok());
        assert_eq!(
            counters.winner_selections
                + counters.ranking_creations
                + counters.reward_applications
                + counters.penalty_applications
                + counters.chair_decisions
                + counters.paper_executions
                + counters.live_executions,
            0
        );
    }

    #[test]
    fn sprint89_41_malformed_protobuf_rejects() {
        let value = fixture_series();
        let mut bytes = encode_series(&value).unwrap();
        bytes.truncate(bytes.len() / 2);
        assert!(decode_series(&bytes).is_err());
    }

    #[test]
    fn sprint89_42_text_and_json_status_fields_agree() {
        let chain = fixture_prediction_chain();
        let series = chain.0;
        let adoption = chain.1;
        let delta = chain.2;
        let registration = chain.3;
        let gap = fixture_gap(&series, &adoption);
        let status = build_status(
            &adoption,
            &gap,
            &delta,
            &registration,
            MomentumProspectiveEpochReadinessV4::RegisteredAwaitingInputFinality,
            None,
            None,
            None,
            None,
            None,
            None,
            MomentumRewardEligibilityStatusV4_4::IneligibleMinimumSamples,
            true,
            true,
            idle_safety_counters(),
        )
        .unwrap();
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["epoch_number"], status.epoch_number);
        assert_eq!(json["event_timestamp_ms"], status.event_timestamp_ms);
        assert_eq!(
            json["readiness"],
            serde_json::to_value(status.readiness).unwrap()
        );
    }

    #[test]
    fn sprint89_43_every_persistence_replay_is_zero_work() {
        let root =
            std::env::temp_dir().join(format!("soma-v4-series-replay-test-{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove prior isolated fixture");
        }
        let value = fixture_series();
        let bytes = encode_series(&value).unwrap();
        let first = persist_pb_if_absent(
            &root,
            "series_contracts",
            &value.series_digest,
            &value,
            &bytes,
            decode_series,
            |item| &item.series_digest,
        )
        .unwrap();
        let replay = persist_pb_if_absent(
            &root,
            "series_contracts",
            &value.series_digest,
            &value,
            &bytes,
            decode_series,
            |item| &item.series_digest,
        )
        .unwrap();
        assert_eq!(first, (1, 0));
        assert_eq!(replay, (0, 0));
        fs::remove_dir_all(root).expect("remove isolated fixture");
    }

    #[test]
    fn sprint90_44_success_capsule_binds_registered_delta_provider_and_attempt() {
        let chain = fixture_prediction_chain();
        let registration = chain.3;
        let capsule = chain.5;
        assert_eq!(
            capsule.context_delta_plan_digest,
            registration.context_delta_plan_digest
        );
        assert_eq!(capsule.provider_id, registration.provider_id);
        assert_eq!(capsule.request_attempt_count, 1);
        assert_eq!(
            capsule.row_identity_digests.len(),
            registration.exact_missing_timestamp_ms.len()
        );
        assert!(validate_input_capsule(&capsule).is_ok());
    }

    #[test]
    fn sprint90_45_prediction_artifacts_bind_epoch_context_and_zero_authority() {
        let chain = fixture_prediction_chain();
        assert!(chain.8.iter().all(|seal| {
            seal.epoch_number == chain.3.epoch_number
                && seal.context_use_proof_digest == chain.6.proof_digest
                && validate_prediction_seal(seal).is_ok()
        }));
        assert!(!chain.9.reward_applied);
        assert!(!chain.9.penalty_applied);
        assert!(!chain.9.chair_action_taken);
        assert!(validate_prediction_capsule(&chain.9).is_ok());
    }

    #[test]
    fn sprint90_46_journal_binds_adoption_delta_and_exact_seals() {
        let chain = fixture_prediction_chain();
        assert_eq!(chain.10.event_one_adoption_digest, chain.1.adoption_digest);
        assert_eq!(
            chain.10.context_delta_plan_digest,
            chain.3.context_delta_plan_digest
        );
        assert_eq!(
            chain.10.participant_seal_digests,
            chain
                .8
                .iter()
                .map(|seal| seal.seal_digest.clone())
                .collect::<Vec<_>>()
        );
        assert!(!chain.10.prior_event_correctness_read);
        assert!(validate_journal(&chain.10).is_ok());
    }

    #[test]
    fn sprint90_47_recovery_after_outcome_finality_cannot_backdate_seal() {
        let chain = fixture_prediction_chain();
        assert_eq!(
            readiness(
                chain.3.outcome_finality_boundary_ms,
                &chain.3,
                Some(&chain.4),
                None,
            ),
            MomentumProspectiveEpochReadinessV4::PredictionSealWindowExpired
        );
        assert_eq!(
            readiness(
                chain.3.outcome_finality_boundary_ms,
                &chain.3,
                Some(&chain.4),
                Some(&chain.9),
            ),
            MomentumProspectiveEpochReadinessV4::PredictionAlreadySealed
        );
    }

    #[test]
    fn sprint90_48_dry_run_has_no_network_authority() {
        assert!(
            validate_run_authority(
                MomentumProspectiveSeriesRunModeV4::DryRun,
                false,
                false,
                None,
            )
            .is_ok()
        );
        assert!(
            validate_run_authority(
                MomentumProspectiveSeriesRunModeV4::DryRun,
                true,
                false,
                None,
            )
            .is_err()
        );
        let chain = fixture_prediction_chain();
        let request = build_provider_request(&chain.3).expect("exact delta request");
        assert_eq!(request.lookback.bars, 1);
        assert_eq!(request.lookback.start_timestamp_ms, Some(EVENT));
        assert_eq!(
            request.lookback.end_timestamp_ms,
            Some(EVENT + DAILY_CADENCE_MS)
        );
    }

    #[test]
    fn sprint90_49_successful_input_status_and_dry_run_do_not_seal() {
        let chain = fixture_prediction_chain();
        let recovery_readiness = readiness(
            chain.3.input_finality_boundary_ms,
            &chain.3,
            Some(&chain.4),
            None,
        );
        assert_eq!(
            recovery_readiness,
            MomentumProspectiveEpochReadinessV4::ReadyForLocalPredictionRecovery
        );
        assert!(!local_prediction_recovery_allowed(
            MomentumProspectiveSeriesRunModeV4::Status,
            recovery_readiness,
        ));
        assert!(!local_prediction_recovery_allowed(
            MomentumProspectiveSeriesRunModeV4::DryRun,
            recovery_readiness,
        ));
        assert!(local_prediction_recovery_allowed(
            MomentumProspectiveSeriesRunModeV4::ExecuteInput,
            recovery_readiness,
        ));

        let expired_readiness = readiness(
            chain.3.outcome_finality_boundary_ms,
            &chain.3,
            Some(&chain.4),
            None,
        );
        assert_eq!(
            expired_readiness,
            MomentumProspectiveEpochReadinessV4::PredictionSealWindowExpired
        );
        assert!(!local_prediction_recovery_allowed(
            MomentumProspectiveSeriesRunModeV4::ExecuteInput,
            expired_readiness,
        ));
    }

    #[test]
    fn sprint91_50_terminal_input_receipt_cannot_retry() {
        let chain = fixture_prediction_chain();
        for status in [
            MomentumProspectiveSeriesInputStatusV4::TerminalTransportFailure,
            MomentumProspectiveSeriesInputStatusV4::TerminalValidationFailure,
        ] {
            let mut receipt = chain.4.clone();
            receipt.status = status;
            receipt.http_status_class = None;
            receipt.returned_row_count = 0;
            receipt.verified_row_count = 0;
            receipt.raw_response_digest = None;
            receipt.input_capsule_digest = None;
            receipt.receipt_digest = input_receipt_digest(&receipt);
            assert!(validate_input_receipt(&receipt).is_ok());

            let terminal_readiness = readiness(
                chain.3.input_finality_boundary_ms,
                &chain.3,
                Some(&receipt),
                None,
            );
            assert_eq!(
                terminal_readiness,
                MomentumProspectiveEpochReadinessV4::PriorInputAttemptTerminal
            );
            assert!(!input_acquisition_allowed(terminal_readiness));
        }
    }

    #[test]
    fn sprint92_51_corrupt_terminal_receipt_remains_integrity_failure() {
        let chain = fixture_prediction_chain();
        let mut receipt = chain.4.clone();
        receipt.status = MomentumProspectiveSeriesInputStatusV4::TerminalValidationFailure;
        receipt.verified_row_count = 0;
        receipt.input_capsule_digest = None;
        receipt.receipt_digest = input_receipt_digest(&receipt);
        assert!(validate_input_receipt(&receipt).is_err());
        assert_eq!(
            readiness(
                chain.3.input_finality_boundary_ms,
                &chain.3,
                Some(&receipt),
                None,
            ),
            MomentumProspectiveEpochReadinessV4::IntegrityFailure
        );

        receipt.raw_response_digest = None;
        receipt.series_digest = "different-series".into();
        receipt.receipt_digest = input_receipt_digest(&receipt);
        assert!(validate_input_receipt(&receipt).is_ok());
        assert_eq!(
            readiness(
                chain.3.input_finality_boundary_ms,
                &chain.3,
                Some(&receipt),
                None,
            ),
            MomentumProspectiveEpochReadinessV4::IntegrityFailure
        );
    }

    #[test]
    fn sprint92_52_only_ready_acquisition_authorizes_input() {
        for current_readiness in [
            MomentumProspectiveEpochReadinessV4::RegisteredAwaitingInputFinality,
            MomentumProspectiveEpochReadinessV4::ReadyForLocalPredictionRecovery,
            MomentumProspectiveEpochReadinessV4::PredictionAlreadySealed,
            MomentumProspectiveEpochReadinessV4::PredictionSealWindowExpired,
            MomentumProspectiveEpochReadinessV4::PriorInputAttemptTerminal,
            MomentumProspectiveEpochReadinessV4::ProspectiveWindowExpired,
            MomentumProspectiveEpochReadinessV4::MissingCanonicalContext,
            MomentumProspectiveEpochReadinessV4::MissingSetNotContiguous,
            MomentumProspectiveEpochReadinessV4::PriorPrivateEvidenceAccessDetected,
            MomentumProspectiveEpochReadinessV4::FrozenIdentityMismatch,
            MomentumProspectiveEpochReadinessV4::IntegrityFailure,
        ] {
            assert!(!input_acquisition_allowed(current_readiness));
        }
        assert!(input_acquisition_allowed(
            MomentumProspectiveEpochReadinessV4::ReadyForInputAcquisition
        ));
    }

    #[test]
    fn sprint95_53_completed_chain_validation_is_independent_of_directory_order() {
        let chain = fixture_prediction_chain();
        let mut reopened_seals = chain.8.clone();
        reopened_seals.reverse();
        assert!(
            validate_persisted_prediction_chain(
                &chain.0,
                &chain.1,
                &chain.3,
                &chain.4,
                &chain.5,
                &chain.6,
                &chain.7,
                &reopened_seals,
                &chain.9,
                &chain.10,
                &chain.11,
            )
            .is_ok()
        );
    }

    #[test]
    fn sprint96_54_completed_chain_rejects_non_exact_seal_bindings() {
        let chain = fixture_prediction_chain();

        let mut duplicate_capsule = chain.9.clone();
        duplicate_capsule.participant_seal_digests[1] =
            duplicate_capsule.participant_seal_digests[0].clone();
        duplicate_capsule.participant_prediction_digests[1] =
            duplicate_capsule.participant_prediction_digests[0].clone();

        let mut missing_seals = chain.8.clone();
        missing_seals.pop();

        let mut mismatched_capsule = chain.9.clone();
        mismatched_capsule.participant_prediction_digests[0] = "mismatch".into();

        let mut extra_seals = chain.8.clone();
        extra_seals.push(chain.8[0].clone());

        for (seals, capsule) in [
            (&chain.8, &duplicate_capsule),
            (&missing_seals, &chain.9),
            (&chain.8, &mismatched_capsule),
            (&extra_seals, &chain.9),
        ] {
            assert!(
                validate_persisted_prediction_chain(
                    &chain.0, &chain.1, &chain.3, &chain.4, &chain.5, &chain.6, &chain.7, seals,
                    capsule, &chain.10, &chain.11,
                )
                .is_err()
            );
        }

        let mut wrong_roster = chain.8.clone();
        wrong_roster[0].participant_digest = "different-participant".into();
        wrong_roster[0].seal_digest = prediction_seal_digest(&wrong_roster[0]);
        let mut wrong_roster_capsule = chain.9.clone();
        wrong_roster_capsule.participant_seal_digests[0] = wrong_roster[0].seal_digest.clone();
        wrong_roster_capsule.capsule_digest = prediction_capsule_digest(&wrong_roster_capsule);
        let mut wrong_roster_journal = chain.10.clone();
        wrong_roster_journal.participant_seal_digests =
            wrong_roster_capsule.participant_seal_digests.clone();
        wrong_roster_journal.prediction_capsule_digest =
            wrong_roster_capsule.capsule_digest.clone();
        wrong_roster_journal.entry_digest = journal_entry_digest(&wrong_roster_journal);
        let mut wrong_roster_outcome_plan = chain.11.clone();
        wrong_roster_outcome_plan.prediction_capsule_digest =
            wrong_roster_capsule.capsule_digest.clone();
        wrong_roster_outcome_plan.plan_digest = outcome_plan_digest(&wrong_roster_outcome_plan);
        assert!(
            validate_persisted_prediction_chain(
                &chain.0,
                &chain.1,
                &chain.3,
                &chain.4,
                &chain.5,
                &chain.6,
                &chain.7,
                &wrong_roster,
                &wrong_roster_capsule,
                &wrong_roster_journal,
                &wrong_roster_outcome_plan,
            )
            .is_err()
        );
    }

    #[test]
    fn sprint100_01_diagnostic_lanes_have_no_outcome_authority() {
        let counters = MomentumProspectiveOutcomeSafetyCountersV4::default();
        assert_eq!(counters.network_request_attempts, 0);
        assert_eq!(counters.prediction_private_value_reads, 0);
        assert_eq!(counters.label_derivations, 0);
    }

    #[test]
    fn sprint100_02_source_identities_are_deterministic() {
        let first = fixture_outcome_registration();
        let second = fixture_outcome_registration();
        assert_eq!(first.registration_digest, second.registration_digest);
    }

    #[test]
    fn sprint100_03_historical_holdout_counters_remain_outside_live_status() {
        let status = fixture_outcome_status();
        assert_eq!(status.safety_counters.outcome_raw_loads, 0);
        assert_eq!(status.safety_counters.evaluations, 0);
        assert!(!status.historical_store_digest.is_empty());
    }

    #[test]
    fn sprint100_04_event_two_prediction_chain_reopens() {
        let chain = fixture_prediction_chain();
        assert!(
            validate_persisted_prediction_chain(
                &chain.0, &chain.1, &chain.3, &chain.4, &chain.5, &chain.6, &chain.7, &chain.8,
                &chain.9, &chain.10, &chain.11,
            )
            .is_ok()
        );
    }

    #[test]
    fn sprint100_05_prediction_seals_resolve_without_directory_order() {
        let chain = fixture_prediction_chain();
        let mut reversed = chain.8.clone();
        reversed.reverse();
        assert!(
            validate_persisted_prediction_chain(
                &chain.0, &chain.1, &chain.3, &chain.4, &chain.5, &chain.6, &chain.7, &reversed,
                &chain.9, &chain.10, &chain.11,
            )
            .is_ok()
        );
    }

    #[test]
    fn sprint100_06_outcome_plan_binds_epoch_two() {
        let registration = fixture_outcome_registration();
        assert_eq!(registration.event_timestamp_ms, EVENT);
        assert_eq!(
            registration.exact_expected_timestamp_ms,
            [EVENT + DAILY_CADENCE_MS]
        );
        assert_eq!(
            registration.outcome_finality_boundary_ms,
            EVENT + 2 * DAILY_CADENCE_MS
        );
    }

    #[test]
    fn sprint100_07_before_finality_is_not_acquirable() {
        let registration = fixture_outcome_registration();
        assert_eq!(
            series_outcome_readiness(
                registration.outcome_finality_boundary_ms - 1,
                &registration,
                &SeriesOutcomeArtifacts::default(),
                &fixture_live_report(),
                "prior-pause",
            ),
            MomentumProspectiveOutcomeReadinessV4::AwaitingOutcomeFinality
        );
    }

    #[test]
    fn sprint100_08_at_finality_is_ready_for_acquisition() {
        let registration = fixture_outcome_registration();
        assert_eq!(
            series_outcome_readiness(
                registration.outcome_finality_boundary_ms,
                &registration,
                &SeriesOutcomeArtifacts::default(),
                &fixture_live_report(),
                "prior-pause",
            ),
            MomentumProspectiveOutcomeReadinessV4::ReadyForOutcomeAcquisition
        );
    }

    #[test]
    fn sprint100_09_outcome_request_is_exactly_one_row() {
        let request = build_outcome_request(&fixture_outcome_registration()).expect("request");
        assert_eq!(request.lookback.bars, 1);
        assert_eq!(request.symbols, ["KRW-BTC"]);
    }

    #[test]
    fn sprint100_10_outcome_request_contains_only_locked_timestamp() {
        let registration = fixture_outcome_registration();
        let request = build_outcome_request(&registration).expect("request");
        assert_eq!(
            request.lookback.start_timestamp_ms,
            Some(registration.exact_expected_timestamp_ms[0])
        );
        assert_eq!(
            request.lookback.end_timestamp_ms,
            Some(registration.outcome_finality_boundary_ms)
        );
    }

    #[test]
    fn sprint100_11_outcome_retries_remain_zero() {
        let registration = fixture_outcome_registration();
        assert_eq!(registration.maximum_retries, 0);
        assert_eq!(
            MomentumProspectiveOutcomeSafetyCountersV4::default().retries,
            0
        );
    }

    #[test]
    fn sprint100_12_terminal_transport_receipt_round_trips() {
        let registration = fixture_outcome_registration();
        let receipt = build_series_terminal_receipt(
            &registration,
            MomentumOutcomeAcquisitionStatusV4_4::TerminalTransportFailure,
            None,
            0,
        )
        .expect("terminal receipt");
        assert_eq!(
            decode_series_outcome_receipt(&encode_series_outcome_receipt(&receipt).unwrap())
                .unwrap(),
            receipt
        );
    }

    #[test]
    fn sprint100_13_terminal_validation_receipt_round_trips() {
        let registration = fixture_outcome_registration();
        let receipt = build_series_terminal_receipt(
            &registration,
            MomentumOutcomeAcquisitionStatusV4_4::TerminalValidationFailure,
            Some(200),
            2,
        )
        .expect("terminal receipt");
        assert_eq!(receipt.request_attempt_count, 1);
        assert_eq!(receipt.verified_row_count, 0);
    }

    #[test]
    fn sprint100_14_terminal_replay_cannot_retry() {
        let registration = fixture_outcome_registration();
        let receipt = build_series_terminal_receipt(
            &registration,
            MomentumOutcomeAcquisitionStatusV4_4::TerminalTransportFailure,
            None,
            0,
        )
        .expect("terminal receipt");
        let artifacts = SeriesOutcomeArtifacts {
            registration: Some(registration.clone()),
            receipt: Some(receipt),
            ..Default::default()
        };
        assert_eq!(
            series_outcome_readiness(
                registration.outcome_finality_boundary_ms,
                &registration,
                &artifacts,
                &fixture_live_report(),
                "prior-pause",
            ),
            MomentumProspectiveOutcomeReadinessV4::PriorOutcomeAttemptTerminal
        );
    }

    #[test]
    fn sprint100_15_success_receipt_binds_prediction_chain() {
        let (registration, _, receipt, capsule) = fixture_outcome_receipt_capsule();
        assert_eq!(
            receipt.prediction_capsule_digest,
            registration.prediction_capsule_digest
        );
        assert_eq!(
            receipt.outcome_capsule_digest.as_deref(),
            Some(capsule.capsule_digest.as_str())
        );
    }

    #[test]
    fn sprint100_16_success_capsule_is_still_sealed() {
        let (_, _, _, capsule) = fixture_outcome_receipt_capsule();
        assert!(!capsule.labels_opened);
        assert!(!capsule.probabilities_opened);
        assert!(!capsule.metrics_computed);
    }

    #[test]
    fn sprint100_17_acquisition_status_does_not_open() {
        let (registration, proof, receipt, capsule) = fixture_outcome_receipt_capsule();
        let artifacts = SeriesOutcomeArtifacts {
            registration: Some(registration.clone()),
            receipt: Some(receipt),
            proof: Some(proof),
            capsule: Some(capsule),
            ..Default::default()
        };
        assert_eq!(
            series_outcome_readiness(
                registration.outcome_finality_boundary_ms,
                &registration,
                &artifacts,
                &fixture_live_report(),
                "prior-pause",
            ),
            MomentumProspectiveOutcomeReadinessV4::ReadyForOutcomeOpening
        );
        assert!(artifacts.authorization.is_none());
    }

    #[test]
    fn sprint100_18_opening_requires_exact_explicit_authority() {
        assert!(
            validate_series_outcome_authority(
                MomentumProspectiveOutcomeRunModeV4::OpenOutcome,
                false,
                false,
                true,
                Some(2),
            )
            .is_ok()
        );
        assert!(
            validate_series_outcome_authority(
                MomentumProspectiveOutcomeRunModeV4::OpenOutcome,
                false,
                false,
                false,
                Some(2),
            )
            .is_err()
        );
    }

    #[test]
    fn sprint100_19_opening_authorization_is_event_two_specific() {
        let (registration, _, receipt, capsule) = fixture_outcome_receipt_capsule();
        let authorization = fixture_opening_authorization(&registration, &receipt, &capsule);
        assert_eq!(
            authorization.outcome_registration_digest,
            registration.registration_digest
        );
        assert_eq!(authorization.opening_attempt_count_before, 0);
        assert!(authorization.one_time_only);
    }

    #[test]
    fn sprint100_20_frozen_label_policy_is_bound() {
        let (registration, _, receipt, capsule) = fixture_outcome_receipt_capsule();
        let authorization = fixture_opening_authorization(&registration, &receipt, &capsule);
        assert_eq!(
            authorization.label_policy_digest,
            frozen_label_policy_digest()
        );
    }

    #[test]
    fn sprint100_21_neutral_outcome_remains_neutral() {
        let (status, label, _) = classify_label_v4_4(100.0, 100.0).expect("neutral label");
        assert_eq!(
            status,
            MomentumProspectiveLabelStatusV4_4::NeutralOutcomeExcluded
        );
        assert_eq!(label, None);
    }

    #[test]
    fn sprint100_22_invalid_evidence_is_explicit() {
        let proof = fixture_outcome_proof(&fixture_outcome_registration());
        let evaluations = fixture_evaluations(
            &proof,
            MomentumProspectiveLabelStatusV4_4::InvalidOutcomeEvidence,
        );
        assert!(evaluations.iter().all(|evaluation| {
            evaluation.status == MomentumProspectiveEvaluationStatusV4_4::InvalidOutcomeEvidence
                && evaluation.private_score_digest.is_none()
                && evaluation.private_correctness_digest.is_none()
        }));
    }

    #[test]
    fn sprint100_23_exactly_three_predictions_are_authorized() {
        let (registration, _, receipt, capsule) = fixture_outcome_receipt_capsule();
        let authorization = fixture_opening_authorization(&registration, &receipt, &capsule);
        assert_eq!(authorization.participant_seal_digests.len(), 3);
        assert_eq!(authorization.participant_prediction_digests.len(), 3);
    }

    #[test]
    fn sprint100_24_exactly_three_evaluations_are_required() {
        let (registration, proof, receipt, capsule) = fixture_outcome_receipt_capsule();
        let authorization = fixture_opening_authorization(&registration, &receipt, &capsule);
        let mut bundle = fixture_opening_bundle(
            &authorization,
            &capsule,
            &proof,
            MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome,
        );
        bundle.participant_evaluations.pop();
        bundle.bundle_digest = opening_bundle_digest(&bundle);
        assert!(validate_opening_bundle_shape(&bundle).is_err());
    }

    #[test]
    fn sprint100_25_evaluation_binding_is_order_independent() {
        let (registration, proof, receipt, capsule) = fixture_outcome_receipt_capsule();
        let authorization = fixture_opening_authorization(&registration, &receipt, &capsule);
        let mut bundle = fixture_opening_bundle(
            &authorization,
            &capsule,
            &proof,
            MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome,
        );
        bundle.participant_evaluations.reverse();
        bundle.bundle_digest = opening_bundle_digest(&bundle);
        assert!(validate_opening_bundle_shape(&bundle).is_ok());
    }

    #[test]
    fn sprint100_26_non_live_participants_cannot_substitute() {
        let live = fixture_live_report();
        let prediction = live.prediction_capsule.expect("prediction");
        let mut substituted = prediction.participant_seal_digests.clone();
        substituted[0] = "historical-participant-seal".into();
        assert_ne!(substituted, prediction.participant_seal_digests);
    }

    #[test]
    fn sprint100_27_event_two_ledger_is_append_only() {
        let (ledger, _, _) =
            fixture_completion_artifacts(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        let root = std::env::temp_dir().join(format!(
            "soma-v4-event-two-ledger-test-{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove prior fixture");
        }
        let bytes = encode_series_ledger_entry(&ledger).unwrap();
        assert_eq!(
            persist_artifact(
                &root.join(format!("{}.pb", ledger.entry_digest)),
                &bytes,
                &ledger.entry_digest,
                |stored| Ok(decode_series_ledger_entry(stored)?.entry_digest),
            )
            .unwrap(),
            (1, 0)
        );
        assert_eq!(
            persist_artifact(
                &root.join(format!("{}.pb", ledger.entry_digest)),
                &bytes,
                &ledger.entry_digest,
                |stored| Ok(decode_series_ledger_entry(stored)?.entry_digest),
            )
            .unwrap(),
            (0, 1)
        );
        fs::remove_dir_all(root).expect("remove fixture");
    }

    #[test]
    fn sprint100_28_event_one_ledger_identity_is_immutable() {
        let live = fixture_live_report();
        let before = live
            .event_one_adoption
            .evaluation_ledger_entry_digest
            .clone();
        let (ledger, _, _) =
            fixture_completion_artifacts(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(ledger.previous_event_ledger_entry_digest, before);
    }

    #[test]
    fn sprint100_29_completed_count_derives_from_ledger() {
        let (ledger, _, _) =
            fixture_completion_artifacts(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(
            ledger.total_event_count_after,
            fixture_live_report().event_one_adoption.total_event_count + 1
        );
    }

    #[test]
    fn sprint100_30_scorable_count_excludes_neutral() {
        let (binary, _, _) =
            fixture_completion_artifacts(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        let (neutral, _, _) = fixture_completion_artifacts(
            MomentumProspectiveLabelStatusV4_4::NeutralOutcomeExcluded,
        );
        assert_eq!(
            binary.scorable_event_count_after,
            neutral.scorable_event_count_after + 1
        );
    }

    #[test]
    fn sprint100_31_eligibility_uses_frozen_minimum_gate() {
        let (_, eligibility, _) =
            fixture_completion_artifacts(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(
            eligibility.status,
            MomentumProspectiveSeriesEligibilityV4::IneligibleMinimumSamples
        );
        assert!(eligibility.scorable_event_count < eligibility.minimum_sample_gate);
    }

    #[test]
    fn sprint100_32_eligibility_never_applies_reward() {
        let (_, eligibility, _) =
            fixture_completion_artifacts(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(eligibility.reward_application_count, 0);
        assert_eq!(eligibility.penalty_application_count, 0);
    }

    #[test]
    fn sprint100_33_no_winner_is_selected() {
        let (ledger, _, _) =
            fixture_completion_artifacts(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert!(!ledger.winner_selected);
    }

    #[test]
    fn sprint100_34_no_ranking_is_created() {
        let (ledger, _, _) =
            fixture_completion_artifacts(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert!(!ledger.ranking_created);
    }

    #[test]
    fn sprint100_35_no_chair_action_occurs() {
        let (ledger, _, _) =
            fixture_completion_artifacts(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert!(!ledger.chair_action_taken);
        assert_eq!(
            MomentumProspectiveOutcomeSafetyCountersV4::default().chair_decisions,
            0
        );
    }

    #[test]
    fn sprint100_36_completed_pause_is_additive() {
        let (_, _, pause) =
            fixture_completion_artifacts(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(pause.prior_pause_digest, "prior-pause");
        assert_eq!(
            pause.policy,
            LiveProspectiveContinuationPolicyV2::PausedAfterCompletedEpochTwo
        );
    }

    #[test]
    fn sprint100_37_epoch_three_remains_absent() {
        let (_, _, pause) =
            fixture_completion_artifacts(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert!(!pause.epoch_three_registered);
        assert_eq!(pause.automatic_registration_count, 0);
        assert_eq!(pause.scheduler_count, 0);
    }

    #[test]
    fn sprint100_38_completed_acquisition_replay_has_zero_network() {
        let status = fixture_outcome_status();
        assert_eq!(status.safety_counters.network_request_attempts, 0);
        assert_eq!(status.safety_counters.transport_constructions, 0);
    }

    #[test]
    fn sprint100_39_completed_opening_replay_has_zero_opening_work() {
        let counters = MomentumProspectiveOutcomeSafetyCountersV4::default();
        assert_eq!(counters.opening_attempts, 0);
        assert_eq!(counters.evaluations, 0);
        assert_eq!(counters.ledger_appends, 0);
    }

    #[test]
    fn sprint100_40_partial_recovery_does_not_authorize_transport() {
        let (_, _, receipt, _) = fixture_outcome_receipt_capsule();
        assert_eq!(receipt.request_attempt_count, 1);
        assert_eq!(receipt.retry_count, 0);
        assert!(
            validate_series_outcome_authority(
                MomentumProspectiveOutcomeRunModeV4::OpenOutcome,
                false,
                false,
                true,
                Some(2),
            )
            .is_ok()
        );
    }

    #[test]
    fn sprint100_41_historical_diagnostics_do_not_enter_evaluation() {
        let proof = fixture_outcome_proof(&fixture_outcome_registration());
        let evaluation = &fixture_evaluations(
            &proof,
            MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome,
        )[0];
        assert_eq!(evaluation.outcome_timestamp_ms, proof.outcome_timestamp_ms);
        assert_eq!(evaluation.event_timestamp_ms, EVENT);
    }

    #[test]
    fn sprint100_42_historical_holdout_is_not_an_outcome_source() {
        let proof = fixture_outcome_proof(&fixture_outcome_registration());
        assert!(proof.credential_free);
        assert!(proof.read_only);
        assert!(!proof.raw_outcome_response_digest.contains("holdout"));
    }

    #[test]
    fn sprint100_43_live_parameters_remain_unchanged() {
        let live = fixture_live_report();
        let before = live.series.parameter_digests.clone();
        let _ =
            fixture_completion_artifacts(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(live.series.parameter_digests, before);
    }

    #[test]
    fn sprint100_44_live_normalizers_remain_unchanged() {
        let live = fixture_live_report();
        let before = live.series.normalizer_digests.clone();
        let _ =
            fixture_completion_artifacts(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(live.series.normalizer_digests, before);
    }

    #[test]
    fn sprint100_45_reward_and_chair_counters_are_zero() {
        let counters = MomentumProspectiveOutcomeSafetyCountersV4::default();
        assert_eq!(
            counters.reward_applications
                + counters.penalty_applications
                + counters.chair_model_executions
                + counters.chair_learning_actions
                + counters.chair_decisions
                + counters.committee_votes,
            0
        );
    }

    #[test]
    fn sprint100_46_paper_and_live_execution_are_zero() {
        let counters = MomentumProspectiveOutcomeSafetyCountersV4::default();
        assert_eq!(counters.paper_executions, 0);
        assert_eq!(counters.live_executions, 0);
    }

    #[test]
    fn sprint100_47_malformed_event_two_protobuf_rejects() {
        let (ledger, eligibility, pause) =
            fixture_completion_artifacts(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        for mut bytes in [
            encode_series_ledger_entry(&ledger).unwrap(),
            encode_series_eligibility(&eligibility).unwrap(),
            encode_completed_pause(&pause).unwrap(),
        ] {
            bytes.truncate(bytes.len() / 2);
            assert!(
                decode_series_ledger_entry(&bytes).is_err()
                    && decode_series_eligibility(&bytes).is_err()
                    && decode_completed_pause(&bytes).is_err()
            );
        }
    }

    #[test]
    fn sprint100_48_conflicting_artifacts_reject() {
        let (registration, proof, receipt, capsule) = fixture_outcome_receipt_capsule();
        let terminal = build_series_terminal_receipt(
            &registration,
            MomentumOutcomeAcquisitionStatusV4_4::TerminalTransportFailure,
            None,
            0,
        )
        .unwrap();
        let artifacts = SeriesOutcomeArtifacts {
            receipt: Some(terminal),
            proof: Some(proof),
            capsule: Some(capsule),
            ..Default::default()
        };
        assert_eq!(
            series_outcome_readiness(
                registration.outcome_finality_boundary_ms,
                &registration,
                &artifacts,
                &fixture_live_report(),
                "prior-pause",
            ),
            MomentumProspectiveOutcomeReadinessV4::IntegrityFailure
        );
        assert_ne!(
            artifacts.receipt.unwrap().receipt_digest,
            receipt.receipt_digest
        );
    }

    #[test]
    fn sprint100_49_text_and_json_status_agree() {
        let status = fixture_outcome_status();
        let report = MomentumProspectiveOutcomeReportV4 {
            status: status.clone(),
            registration: fixture_outcome_registration(),
            receipt: None,
            outcome_capsule: None,
            opening_authorization: None,
            opening_receipt: None,
            opening_bundle: None,
            ledger_entry: None,
            eligibility_receipt: None,
            completed_pause: None,
        };
        let text = crate::cli::format_momentum_v4_prospective_outcome_text(&report);
        let json = serde_json::to_value(&status).unwrap();
        assert!(text.contains(&format!("series_digest={}", status.series_digest)));
        assert!(text.contains(&format!("epoch_number={}", status.epoch_number)));
        assert_eq!(json["series_digest"], status.series_digest);
        assert_eq!(json["epoch_number"], status.epoch_number);
        assert_eq!(
            json["readiness"],
            serde_json::to_value(status.readiness).unwrap()
        );
    }
}
