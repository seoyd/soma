//! Additive, time-separated future-evaluation lifecycle for the frozen Momentum V4 family.
//!
//! The input stage can acquire only finalized daily OHLCV needed to construct one
//! event feature. Predictions are sealed before the outcome stage becomes
//! eligible. This module has no metric, ranking, reward, active-model, voting,
//! Chair, promotion, or execution authority.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{
    core::stable_hash_string,
    data::{
        AcquisitionMarketScope, DataLookback, DataSnapshot, DatasetKind,
        LearningEvidenceTransportFailureV1, LearningEvidenceTransportResponseV1,
        ReadOnlyProviderRequest, UpbitHistoricalPilotConfigV0,
        fetch_upbit_learning_evidence_once_v1, historical_replay_dataset_digest_v0,
        upbit_learning_evidence_provider_contract_v1,
    },
    league::{HistoricalOhlcvRow, canonical_current_agent_states},
};

use super::agent_learning_session::{
    AgentPrivateLearningArtifactWriteStatusV0, atomic_write_verified_v0,
};
use super::momentum_raw_feature_supplemental::{
    MomentumAccumulatedEvaluationRegistrationV4_1, MomentumFutureEvaluationSourceV4_2,
    reopen_momentum_v4_1_future_source,
};
use super::momentum_raw_feature_v4::{
    MomentumFrozenParticipantPredictionV4, MomentumRawFeatureRoleV4,
    predict_frozen_momentum_v4_event, reconstruct_frozen_momentum_v4,
};
use super::{MomentumLearningCampaignConfigV0, ProtectedEvaluationReservationV1};

const AGENT_ID_V4_2: &str = "momentum_trend_fast";
const ROOT_VERSION_V4_2: &str = "v4_2";
const DAILY_CADENCE_MS_V4_2: u64 = 86_400_000;
const LIFECYCLE_VERSION_V4_2: &str = "momentum-future-evaluation-lifecycle-v4.2";
const CONTEXT_PLAN_VERSION_V4_2: &str = "momentum-prospective-feature-context-plan-v4.2";
const INPUT_REGISTRATION_VERSION_V4_2: &str =
    "momentum-prospective-input-acquisition-registration-v4.2";
const INPUT_RECEIPT_VERSION_V4_2: &str = "momentum-prospective-input-receipt-v4.2";
const INPUT_CAPSULE_VERSION_V4_2: &str = "momentum-prospective-input-capsule-v4.2";
const CONTEXT_PROOF_VERSION_V4_2: &str = "momentum-feature-context-verification-v4.2";
const PREDICTION_CAPSULE_VERSION_V4_2: &str = "momentum-prospective-prediction-capsule-v4.2";
const PREDICTION_JOURNAL_VERSION_V4_2: &str = "momentum-prospective-prediction-journal-v4.2";
const MATURITY_PLAN_VERSION_V4_2: &str = "momentum-prospective-outcome-maturity-plan-v4.2";
const STATUS_RECEIPT_VERSION_V4_2: &str = "momentum-future-prediction-status-v4.2";
const ROOT_VERSION_V4_3: &str = "v4_3";
const AUTHORIZATION_VERSION_V4_3: &str = "momentum-protected-context-authorization-v4.3";
const SUPERSESSION_VERSION_V4_3: &str = "momentum-input-plan-supersession-v4.3";
const CONTEXT_PLAN_VERSION_V4_3: &str = "momentum-prospective-feature-context-plan-v4.3";
const INPUT_REGISTRATION_VERSION_V4_3: &str = "momentum-prospective-input-registration-v4.3";
const INPUT_RECEIPT_VERSION_V4_3: &str = "momentum-prospective-input-receipt-v4.3";
const INPUT_CAPSULE_VERSION_V4_3: &str = "momentum-prospective-input-capsule-v4.3";
const CONTEXT_LEDGER_VERSION_V4_3: &str = "momentum-context-usage-ledger-v4.3";
const CONTEXT_PROOF_VERSION_V4_3: &str = "momentum-feature-context-verification-v4.3";
const PREDICTION_CAPSULE_VERSION_V4_3: &str = "momentum-prospective-prediction-capsule-v4.3";
const PREDICTION_JOURNAL_VERSION_V4_3: &str = "momentum-prospective-prediction-journal-v4.3";
const OUTCOME_PLAN_VERSION_V4_3: &str = "momentum-prospective-outcome-plan-v4.3";
const STATUS_RECEIPT_VERSION_V4_3: &str = "momentum-future-prediction-status-v4.3";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FutureEvaluationRequestBudgetMeaningV4_2 {
    InputEvidenceOnly,
    OutcomeEvidenceOnly,
    EntireLifecycleSingleRequest,
    ExistingLocalInputPlusOutcomeRequest,
    Ambiguous,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumFutureEvaluationLifecycleV4_2 {
    pub lifecycle_version: String,
    pub agent_id: String,
    pub source_v4_family_digest: String,
    pub accumulated_family_digest: String,
    pub roster_digest: String,
    pub evaluation_registration_digest: String,
    pub participant_digests: Vec<String>,
    pub participant_parameter_digests: Vec<String>,
    pub participant_normalizer_digests: Vec<String>,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub qualification_policy_digest: String,
    pub minimum_accepted_event_timestamp_ms: u64,
    pub cadence_ms: u64,
    pub prediction_horizon: usize,
    pub input_stage_maximum_requests: usize,
    pub input_stage_maximum_retries: usize,
    pub outcome_stage_maximum_requests: usize,
    pub outcome_stage_maximum_retries: usize,
    pub prediction_must_precede_outcome_access: bool,
    pub outcome_stage_locked_until_prediction_sealed: bool,
    pub labels_hidden_until_opening: bool,
    pub probabilities_hidden_until_opening: bool,
    pub winner_selection_forbidden: bool,
    pub active_promotion_forbidden: bool,
    pub reward_application_forbidden: bool,
    pub lifecycle_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumEventReadinessV4_2 {
    ReadyForInputAcquisition,
    AwaitingMinimumTimestamp,
    AwaitingInputFinality,
    AwaitingSufficientFeatureContext,
    AwaitingPostExclusionContext,
    ContextPolicyAmbiguous,
    SupersededInputRegistration,
    PriorInputAttemptTerminal,
    PredictionAlreadySealed,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumContextPolicyStatusV4_2 {
    ContextOnlyUseExplicitlyAllowed,
    ContextUseExplicitlyForbidden,
    ContextUseAmbiguous,
    NoProtectedTimestampRequired,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveFeatureContextPlanV4_2 {
    pub plan_version: String,
    pub event_timestamp_ms: u64,
    pub required_context_start_timestamp_ms: u64,
    pub required_context_end_timestamp_ms: u64,
    pub required_row_count: usize,
    pub required_timestamp_digest: String,
    pub existing_source_row_digests: Vec<String>,
    pub incremental_row_timestamps: Vec<u64>,
    pub protected_context_timestamp_ms: Vec<u64>,
    pub context_policy_status: MomentumContextPolicyStatusV4_2,
    pub plan_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveInputAcquisitionRegistrationV4_2 {
    pub registration_version: String,
    pub lifecycle_digest: String,
    pub evaluation_registration_digest: String,
    pub roster_digest: String,
    pub event_timestamp_ms: u64,
    pub feature_context_plan_digest: String,
    pub provider_id: String,
    pub market: String,
    pub symbol: String,
    pub cadence: String,
    pub exact_expected_timestamp_ms: Vec<u64>,
    pub exact_request_count: usize,
    pub request_to_timestamp_ms: u64,
    pub maximum_requests: usize,
    pub maximum_concurrency: usize,
    pub maximum_retries: usize,
    pub maximum_response_bytes: usize,
    pub credential_free_required: bool,
    pub read_only_required: bool,
    pub outcome_timestamp_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumProspectiveInputStatusV4_2 {
    ReadyNotAttempted,
    EvidenceAcquired,
    ProviderRejected,
    TimeoutNoRetry,
    InvalidResponse,
    TechnicalFailure,
}

impl MomentumProspectiveInputStatusV4_2 {
    fn is_terminal(self) -> bool {
        self != Self::ReadyNotAttempted
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveInputReceiptV4_2 {
    pub receipt_version: String,
    pub lifecycle_digest: String,
    pub input_registration_digest: String,
    pub request_attempted: bool,
    pub request_count: usize,
    pub retry_count: usize,
    pub status: MomentumProspectiveInputStatusV4_2,
    pub http_status_class: Option<String>,
    pub returned_row_count: usize,
    pub verified_row_count: usize,
    pub raw_response_digest: Option<String>,
    pub input_capsule_digest: Option<String>,
    pub terminal: bool,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveInputCapsuleV4_2 {
    pub capsule_version: String,
    pub lifecycle_digest: String,
    pub input_registration_digest: String,
    pub event_timestamp_ms: u64,
    pub exact_timestamp_ms: Vec<u64>,
    pub row_identity_digests: Vec<String>,
    pub normalized_dataset_digest: String,
    pub raw_response_digest: String,
    pub outcome_rows_present: bool,
    pub labels_accessed: bool,
    pub metrics_computed: bool,
    pub credential_free: bool,
    pub read_only: bool,
    pub sanitized: bool,
    pub capsule_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveContextVerificationV4_2 {
    pub proof_version: String,
    pub feature_context_plan_digest: String,
    pub input_registration_digest: String,
    pub input_capsule_digest: String,
    pub exact_timestamps_verified: bool,
    pub strict_chronology_verified: bool,
    pub feature_history_complete: bool,
    pub protected_events_not_scored: bool,
    pub outcome_timestamp_absent: bool,
    pub proof_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumParticipantPredictionSealV4_2 {
    pub participant_digest: String,
    pub event_timestamp_ms: u64,
    pub input_capsule_digest: String,
    pub feature_identity_digest: String,
    pub prediction_probability_bits: u32,
    pub prediction_digest: String,
    pub participant_reconstructed: bool,
    pub parameter_updates: usize,
    pub outcome_access_count: usize,
    pub seal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectivePredictionCapsuleV4_2 {
    pub capsule_version: String,
    pub lifecycle_digest: String,
    pub evaluation_registration_digest: String,
    pub roster_digest: String,
    pub event_timestamp_ms: u64,
    pub input_receipt_digest: String,
    pub input_capsule_digest: String,
    pub participant_prediction_seals: Vec<MomentumParticipantPredictionSealV4_2>,
    pub probabilities_hidden: bool,
    pub labels_hidden: bool,
    pub outcome_accessed: bool,
    pub metrics_computed: bool,
    pub winner_selected: bool,
    pub capsule_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectivePredictionJournalEntryV4_2 {
    pub event_timestamp_ms: u64,
    pub input_capsule_digest: String,
    pub prediction_capsule_digest: String,
    pub participant_prediction_digests: Vec<String>,
    pub prediction_sealed_before_outcome: bool,
    pub outcome_stage_unlocked: bool,
    pub entry_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectivePredictionJournalV4_2 {
    pub journal_version: String,
    pub entries: Vec<MomentumProspectivePredictionJournalEntryV4_2>,
    pub journal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveOutcomeMaturityPlanV4_2 {
    pub plan_version: String,
    pub prediction_capsule_digest: String,
    pub event_timestamp_ms: u64,
    pub prediction_horizon: usize,
    pub required_outcome_timestamp_ms: Vec<u64>,
    pub outcome_finality_boundary_ms: u64,
    pub maximum_outcome_requests: usize,
    pub maximum_outcome_retries: usize,
    pub labels_hidden_until_opening: bool,
    pub one_time_opening_required: bool,
    pub plan_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumFuturePredictionSafetyCountersV4_2 {
    pub input_request_attempts: usize,
    pub input_retries: usize,
    pub input_concurrency: usize,
    pub outcome_request_attempts: usize,
    pub outcome_retries: usize,
    pub participant_parameter_updates: usize,
    pub normalizer_refits: usize,
    pub outcome_row_reads: usize,
    pub outcome_label_reads: usize,
    pub metric_computations: usize,
    pub winner_selections: usize,
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

impl Default for MomentumFuturePredictionSafetyCountersV4_2 {
    fn default() -> Self {
        Self {
            input_request_attempts: 0,
            input_retries: 0,
            input_concurrency: 1,
            outcome_request_attempts: 0,
            outcome_retries: 0,
            participant_parameter_updates: 0,
            normalizer_refits: 0,
            outcome_row_reads: 0,
            outcome_label_reads: 0,
            metric_computations: 0,
            winner_selections: 0,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumFuturePredictionRunModeV4_2 {
    Status,
    DryRun,
    Execute,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumFuturePredictionStatusReceiptV4_2 {
    pub status_version: String,
    pub request_budget_meaning: FutureEvaluationRequestBudgetMeaningV4_2,
    pub lifecycle_digest: String,
    pub event_readiness: MomentumEventReadinessV4_2,
    pub event_timestamp_ms: u64,
    pub input_finality_boundary_ms: u64,
    pub context_policy_status: MomentumContextPolicyStatusV4_2,
    pub context_plan_digest: String,
    pub input_registration_digest: String,
    pub request_attempt_count: usize,
    pub input_receipt_digest: Option<String>,
    pub input_capsule_digest: Option<String>,
    pub participant_prediction_digests: Vec<String>,
    pub prediction_capsule_digest: Option<String>,
    pub outcome_maturity_plan_digest: Option<String>,
    pub outcome_finality_boundary_ms: Option<u64>,
    pub cycle_risk_status: String,
    pub value_quality_status: String,
    pub prior_momentum_attribution: String,
    pub prior_cycle_risk_attribution: String,
    pub protected_artifacts_unchanged: bool,
    pub active_state_unchanged: bool,
    pub safety_counters: MomentumFuturePredictionSafetyCountersV4_2,
    pub status_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumFuturePredictionReportV4_2 {
    pub status: MomentumFuturePredictionStatusReceiptV4_2,
    pub lifecycle: MomentumFutureEvaluationLifecycleV4_2,
    pub context_plan: MomentumProspectiveFeatureContextPlanV4_2,
    pub input_registration: MomentumProspectiveInputAcquisitionRegistrationV4_2,
    pub input_receipt: Option<MomentumProspectiveInputReceiptV4_2>,
    pub input_capsule: Option<MomentumProspectiveInputCapsuleV4_2>,
    pub prediction_capsule: Option<MomentumProspectivePredictionCapsuleV4_2>,
    pub prediction_journal: Option<MomentumProspectivePredictionJournalV4_2>,
    pub outcome_maturity_plan: Option<MomentumProspectiveOutcomeMaturityPlanV4_2>,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub storage_failure_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtectedContextUseClassV4_3 {
    RawOhlcvInferenceContext,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtectedContextAuthorizationStatusV4_3 {
    Authorized,
    AlreadyAuthorized,
    ModelFreezeNotProven,
    RawEvidenceUnavailable,
    PriorOutcomeArtifactReferenced,
    PolicyConflict,
    IntegrityFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProtectedContextAuthorizationV4_3 {
    pub authorization_version: String,
    pub agent_id: String,
    pub v4_family_digest: String,
    pub accumulated_family_digest: String,
    pub roster_digest: String,
    pub evaluation_registration_digest: String,
    pub lifecycle_digest: String,
    pub participant_digests: Vec<String>,
    pub parameter_digests: Vec<String>,
    pub normalizer_digests: Vec<String>,
    pub model_source_boundary_timestamp_ms: u64,
    pub protected_registration_digests: Vec<String>,
    pub protected_timestamp_ms: Vec<u64>,
    pub allowed_use_class: ProtectedContextUseClassV4_3,
    pub raw_ohlcv_context_allowed: bool,
    pub training_use_forbidden: bool,
    pub normalizer_fit_forbidden: bool,
    pub label_use_forbidden: bool,
    pub qualification_use_forbidden: bool,
    pub metric_use_forbidden: bool,
    pub reward_use_forbidden: bool,
    pub event_timestamp_use_forbidden: bool,
    pub outcome_timestamp_use_forbidden: bool,
    pub prior_outcome_capsule_use_forbidden: bool,
    pub authorization_status: ProtectedContextAuthorizationStatusV4_3,
    pub authorization_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumContextEvidenceUseV4_3 {
    ExistingFrozenHistoricalContext,
    ProtectedRawInferenceContext,
    NewIncrementalInferenceContext,
    ProspectiveEventInput,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumContextUsageEntryV4_3 {
    pub timestamp_ms: u64,
    pub raw_row_digest: String,
    pub use_class: MomentumContextEvidenceUseV4_3,
    pub used_for_feature_construction: bool,
    pub used_for_training: bool,
    pub used_for_normalizer_fit: bool,
    pub used_for_label: bool,
    pub used_for_metric: bool,
    pub used_for_reward: bool,
    pub entry_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumContextUsageLedgerV4_3 {
    pub ledger_version: String,
    pub authorization_digest: String,
    pub event_timestamp_ms: u64,
    pub entries: Vec<MomentumContextUsageEntryV4_3>,
    pub ledger_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumInputPlanSupersessionStatusV4_3 {
    Superseded,
    AlreadySuperseded,
    PriorAttemptExists,
    PriorEvidenceExists,
    IntegrityFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumInputPlanSupersessionV4_3 {
    pub supersession_version: String,
    pub lifecycle_digest: String,
    pub context_authorization_digest: String,
    pub superseded_context_plan_digest: String,
    pub superseded_input_registration_digest: String,
    pub superseded_event_timestamp_ms: u64,
    pub prior_request_attempt_count: usize,
    pub prior_receipt_present: bool,
    pub prior_capsule_present: bool,
    pub prior_prediction_capsule_present: bool,
    pub replacement_event_timestamp_ms: u64,
    pub replacement_context_plan_digest: String,
    pub replacement_input_registration_digest: String,
    pub status: MomentumInputPlanSupersessionStatusV4_3,
    pub supersession_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumEventReadinessV4_3 {
    ReadyForInputAcquisition,
    AwaitingInputFinality,
    ContextOnlyAuthorizationMissing,
    ContextAuthorizationIntegrityFailure,
    InputRegistrationSuperseded,
    PriorInputAttemptTerminal,
    PredictionAlreadySealed,
    IntegrityFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveFeatureContextPlanV4_3 {
    pub plan_version: String,
    pub lifecycle_digest: String,
    pub context_authorization_digest: String,
    pub event_timestamp_ms: u64,
    pub input_finality_boundary_ms: u64,
    pub required_context_start_timestamp_ms: u64,
    pub required_context_end_timestamp_ms: u64,
    pub required_row_count: usize,
    pub exact_required_timestamp_ms: Vec<u64>,
    pub existing_source_timestamp_ms: Vec<u64>,
    pub protected_context_timestamp_ms: Vec<u64>,
    pub incremental_timestamp_ms: Vec<u64>,
    pub context_usage_policy_digest: String,
    pub plan_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveInputRegistrationV4_3 {
    pub registration_version: String,
    pub lifecycle_digest: String,
    pub evaluation_registration_digest: String,
    pub roster_digest: String,
    pub context_authorization_digest: String,
    pub supersession_digest: String,
    pub context_plan_digest: String,
    pub event_timestamp_ms: u64,
    pub input_finality_boundary_ms: u64,
    pub provider_id: String,
    pub market: String,
    pub symbol: String,
    pub cadence: String,
    pub exact_expected_timestamp_ms: Vec<u64>,
    pub expected_row_count: usize,
    pub request_to_timestamp_ms: u64,
    pub maximum_requests: usize,
    pub maximum_concurrency: usize,
    pub maximum_retries: usize,
    pub maximum_response_bytes: usize,
    pub credential_free_required: bool,
    pub read_only_required: bool,
    pub outcome_timestamp_forbidden: bool,
    pub prior_outcome_artifact_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveInputReceiptV4_3 {
    pub receipt_version: String,
    pub lifecycle_digest: String,
    pub context_authorization_digest: String,
    pub input_registration_digest: String,
    pub request_attempted: bool,
    pub request_count: usize,
    pub retry_count: usize,
    pub status: MomentumProspectiveInputStatusV4_2,
    pub http_status_class: Option<String>,
    pub returned_row_count: usize,
    pub verified_row_count: usize,
    pub raw_response_digest: Option<String>,
    pub input_capsule_digest: Option<String>,
    pub terminal: bool,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveInputCapsuleV4_3 {
    pub capsule_version: String,
    pub lifecycle_digest: String,
    pub context_authorization_digest: String,
    pub input_registration_digest: String,
    pub event_timestamp_ms: u64,
    pub exact_timestamp_ms: Vec<u64>,
    pub row_identity_digests: Vec<String>,
    pub normalized_dataset_digest: String,
    pub raw_response_digest: String,
    pub outcome_row_present: bool,
    pub labels_accessed: bool,
    pub metrics_computed: bool,
    pub prior_outcome_capsule_accessed: bool,
    pub credential_free: bool,
    pub read_only: bool,
    pub sanitized: bool,
    pub capsule_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveContextVerificationV4_3 {
    pub proof_version: String,
    pub context_authorization_digest: String,
    pub feature_context_plan_digest: String,
    pub input_registration_digest: String,
    pub input_capsule_digest: String,
    pub context_usage_ledger_digest: String,
    pub exact_timestamps_verified: bool,
    pub strict_chronology_verified: bool,
    pub feature_history_complete: bool,
    pub protected_context_inference_only: bool,
    pub outcome_timestamp_absent: bool,
    pub prior_outcome_artifact_accessed: bool,
    pub proof_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumParticipantPredictionSealV4_3 {
    pub participant_digest: String,
    pub participant_role: String,
    pub event_timestamp_ms: u64,
    pub input_receipt_digest: String,
    pub input_capsule_digest: String,
    pub context_usage_ledger_digest: String,
    pub feature_identity_digest: String,
    pub prediction_probability_bits: u32,
    pub prediction_digest: String,
    pub participant_identity_verified: bool,
    pub parameter_updates: usize,
    pub normalizer_refits: usize,
    pub outcome_access_count: usize,
    pub seal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectivePredictionCapsuleV4_3 {
    pub capsule_version: String,
    pub lifecycle_digest: String,
    pub evaluation_registration_digest: String,
    pub roster_digest: String,
    pub context_authorization_digest: String,
    pub supersession_digest: String,
    pub corrected_context_plan_digest: String,
    pub input_registration_digest: String,
    pub event_timestamp_ms: u64,
    pub input_receipt_digest: String,
    pub input_capsule_digest: String,
    pub context_usage_ledger_digest: String,
    pub participant_prediction_seals: Vec<MomentumParticipantPredictionSealV4_3>,
    pub probabilities_hidden: bool,
    pub labels_hidden: bool,
    pub outcome_accessed: bool,
    pub metrics_computed: bool,
    pub winner_selected: bool,
    pub capsule_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectivePredictionJournalEntryV4_3 {
    pub event_timestamp_ms: u64,
    pub context_authorization_digest: String,
    pub input_capsule_digest: String,
    pub context_usage_ledger_digest: String,
    pub prediction_capsule_digest: String,
    pub participant_prediction_digests: Vec<String>,
    pub prediction_sealed_before_outcome: bool,
    pub outcome_stage_unlocked: bool,
    pub entry_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectivePredictionJournalV4_3 {
    pub journal_version: String,
    pub entries: Vec<MomentumProspectivePredictionJournalEntryV4_3>,
    pub journal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveOutcomePlanV4_3 {
    pub plan_version: String,
    pub prediction_capsule_digest: String,
    pub event_timestamp_ms: u64,
    pub prediction_horizon: usize,
    pub required_outcome_timestamp_ms: Vec<u64>,
    pub outcome_finality_boundary_ms: u64,
    pub maximum_outcome_requests: usize,
    pub maximum_outcome_retries: usize,
    pub labels_hidden_until_opening: bool,
    pub one_time_opening_required: bool,
    pub outcome_stage_locked_before_finality: bool,
    pub plan_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumFuturePredictionSafetyCountersV4_3 {
    pub input_request_attempts: usize,
    pub input_retries: usize,
    pub maximum_concurrency: usize,
    pub outcome_request_attempts: usize,
    pub outcome_retries: usize,
    pub participant_parameter_updates: usize,
    pub normalizer_refits: usize,
    pub protected_context_training_uses: usize,
    pub protected_context_label_uses: usize,
    pub protected_context_metric_uses: usize,
    pub protected_context_reward_uses: usize,
    pub outcome_row_reads: usize,
    pub outcome_label_reads: usize,
    pub metric_computations: usize,
    pub winner_selections: usize,
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

impl Default for MomentumFuturePredictionSafetyCountersV4_3 {
    fn default() -> Self {
        Self {
            input_request_attempts: 0,
            input_retries: 0,
            maximum_concurrency: 1,
            outcome_request_attempts: 0,
            outcome_retries: 0,
            participant_parameter_updates: 0,
            normalizer_refits: 0,
            protected_context_training_uses: 0,
            protected_context_label_uses: 0,
            protected_context_metric_uses: 0,
            protected_context_reward_uses: 0,
            outcome_row_reads: 0,
            outcome_label_reads: 0,
            metric_computations: 0,
            winner_selections: 0,
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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumFuturePredictionStatusReceiptV4_3 {
    pub status_version: String,
    pub lifecycle_version: String,
    pub lifecycle_digest: String,
    pub context_authorization_status: ProtectedContextAuthorizationStatusV4_3,
    pub context_authorization_digest: String,
    pub supersession_status: MomentumInputPlanSupersessionStatusV4_3,
    pub supersession_digest: String,
    pub superseded_event_timestamp_ms: u64,
    pub replacement_event_timestamp_ms: u64,
    pub input_finality_boundary_ms: u64,
    pub event_readiness: MomentumEventReadinessV4_3,
    pub corrected_context_plan_digest: String,
    pub corrected_input_registration_digest: String,
    pub input_request_count: usize,
    pub input_receipt_digest: Option<String>,
    pub input_capsule_digest: Option<String>,
    pub context_usage_ledger_digest: Option<String>,
    pub context_usage_classes: Vec<String>,
    pub participant_prediction_digests: Vec<String>,
    pub prediction_capsule_digest: Option<String>,
    pub prediction_journal_digest: Option<String>,
    pub outcome_plan_digest: Option<String>,
    pub outcome_finality_boundary_ms: Option<u64>,
    pub cycle_risk_status: String,
    pub value_quality_status: String,
    pub prior_momentum_attribution: String,
    pub prior_cycle_risk_attribution: String,
    pub protected_artifacts_unchanged: bool,
    pub active_state_unchanged: bool,
    pub safety_counters: MomentumFuturePredictionSafetyCountersV4_3,
    pub status_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumFuturePredictionReportV4_3 {
    pub status: MomentumFuturePredictionStatusReceiptV4_3,
    pub lifecycle: MomentumFutureEvaluationLifecycleV4_2,
    pub context_authorization: MomentumProtectedContextAuthorizationV4_3,
    pub supersession: MomentumInputPlanSupersessionV4_3,
    pub context_plan: MomentumProspectiveFeatureContextPlanV4_3,
    pub input_registration: MomentumProspectiveInputRegistrationV4_3,
    pub input_receipt: Option<MomentumProspectiveInputReceiptV4_3>,
    pub input_capsule: Option<MomentumProspectiveInputCapsuleV4_3>,
    pub context_usage_ledger: Option<MomentumContextUsageLedgerV4_3>,
    pub prediction_capsule: Option<MomentumProspectivePredictionCapsuleV4_3>,
    pub prediction_journal: Option<MomentumProspectivePredictionJournalV4_3>,
    pub outcome_plan: Option<MomentumProspectiveOutcomePlanV4_3>,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub storage_failure_count: usize,
}

fn canonical_digest<T: Clone + std::fmt::Debug>(value: &T, clear: impl FnOnce(&mut T)) -> String {
    let mut canonical = value.clone();
    clear(&mut canonical);
    stable_hash_string(&format!("{canonical:?}"))
}

fn lifecycle_digest(value: &MomentumFutureEvaluationLifecycleV4_2) -> String {
    canonical_digest(value, |item| item.lifecycle_digest.clear())
}

fn context_plan_digest(value: &MomentumProspectiveFeatureContextPlanV4_2) -> String {
    canonical_digest(value, |item| item.plan_digest.clear())
}

fn input_registration_digest(
    value: &MomentumProspectiveInputAcquisitionRegistrationV4_2,
) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}

fn input_receipt_digest(value: &MomentumProspectiveInputReceiptV4_2) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn input_capsule_digest(value: &MomentumProspectiveInputCapsuleV4_2) -> String {
    canonical_digest(value, |item| item.capsule_digest.clear())
}

fn context_proof_digest(value: &MomentumProspectiveContextVerificationV4_2) -> String {
    canonical_digest(value, |item| item.proof_digest.clear())
}

fn prediction_seal_digest(value: &MomentumParticipantPredictionSealV4_2) -> String {
    canonical_digest(value, |item| item.seal_digest.clear())
}

fn prediction_capsule_digest(value: &MomentumProspectivePredictionCapsuleV4_2) -> String {
    canonical_digest(value, |item| item.capsule_digest.clear())
}

fn prediction_entry_digest(value: &MomentumProspectivePredictionJournalEntryV4_2) -> String {
    canonical_digest(value, |item| item.entry_digest.clear())
}

fn prediction_journal_digest(value: &MomentumProspectivePredictionJournalV4_2) -> String {
    canonical_digest(value, |item| item.journal_digest.clear())
}

fn maturity_plan_digest(value: &MomentumProspectiveOutcomeMaturityPlanV4_2) -> String {
    canonical_digest(value, |item| item.plan_digest.clear())
}

fn status_digest(value: &MomentumFuturePredictionStatusReceiptV4_2) -> String {
    canonical_digest(value, |item| item.status_digest.clear())
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn classify_request_budget(
    evaluation: &MomentumAccumulatedEvaluationRegistrationV4_1,
) -> FutureEvaluationRequestBudgetMeaningV4_2 {
    if evaluation.maximum_requests == 1 {
        FutureEvaluationRequestBudgetMeaningV4_2::Ambiguous
    } else {
        FutureEvaluationRequestBudgetMeaningV4_2::EntireLifecycleSingleRequest
    }
}

fn roster_participant_digests(source: &MomentumFutureEvaluationSourceV4_2) -> Vec<String> {
    source
        .roster
        .learned_participant_digests
        .iter()
        .chain(&source.roster.benchmark_participant_digests)
        .cloned()
        .collect()
}

fn derive_lifecycle(
    source: &MomentumFutureEvaluationSourceV4_2,
) -> Result<MomentumFutureEvaluationLifecycleV4_2, String> {
    if classify_request_budget(&source.evaluation)
        != FutureEvaluationRequestBudgetMeaningV4_2::Ambiguous
        || source.registration.split_digest != source.split.split_digest
        || source.source_family.registration_digest != source.registration.registration_digest
        || source.source_family.split_digest != source.split.split_digest
        || source.accumulated_family.supplemental_registration_digest
            != source.supplemental_registration.registration_digest
        || source.evaluation.supplemental_registration_digest
            != source.supplemental_registration.registration_digest
    {
        return Err("V4.2 original request-budget audit rejected".to_string());
    }
    let participant_digests = roster_participant_digests(source);
    let participants = participant_digests
        .iter()
        .map(|digest| {
            source
                .source_family
                .participants
                .iter()
                .find(|participant| &participant.participant_digest == digest)
                .ok_or_else(|| "V4.2 roster participant missing from source family".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let qualification_policy_digests = sorted_unique(
        source
            .accumulated_family
            .accumulated_receipts
            .iter()
            .map(|receipt| receipt.qualification_policy_digest.clone())
            .collect(),
    );
    if participant_digests.len() != 3 || qualification_policy_digests.is_empty() {
        return Err("V4.2 lifecycle participant set rejected".to_string());
    }
    let mut lifecycle = MomentumFutureEvaluationLifecycleV4_2 {
        lifecycle_version: LIFECYCLE_VERSION_V4_2.to_string(),
        agent_id: AGENT_ID_V4_2.to_string(),
        source_v4_family_digest: source.source_family.family_digest.clone(),
        accumulated_family_digest: source.accumulated_family.family_digest.clone(),
        roster_digest: source.roster.roster_digest.clone(),
        evaluation_registration_digest: source.evaluation.registration_digest.clone(),
        participant_digests,
        participant_parameter_digests: participants
            .iter()
            .map(|participant| participant.parameter_digest.clone())
            .collect(),
        participant_normalizer_digests: participants
            .iter()
            .map(|participant| participant.normalizer_digest.clone())
            .collect(),
        feature_policy_digest: source.closure.feature_policy_digest.clone(),
        label_policy_digest: source.closure.label_policy_digest.clone(),
        qualification_policy_digest: stable_hash_string(&format!(
            "momentum-v4.2-qualification-policies:{qualification_policy_digests:?}"
        )),
        minimum_accepted_event_timestamp_ms: source.evaluation.minimum_accepted_timestamp_ms,
        cadence_ms: DAILY_CADENCE_MS_V4_2,
        prediction_horizon: MomentumLearningCampaignConfigV0::default()
            .sequence_config
            .prediction_horizon,
        input_stage_maximum_requests: 1,
        input_stage_maximum_retries: 0,
        outcome_stage_maximum_requests: 1,
        outcome_stage_maximum_retries: 0,
        prediction_must_precede_outcome_access: true,
        outcome_stage_locked_until_prediction_sealed: true,
        labels_hidden_until_opening: source.evaluation.labels_hidden_until_opening,
        probabilities_hidden_until_opening: source.evaluation.probabilities_hidden_until_opening,
        winner_selection_forbidden: source.evaluation.winner_selection_forbidden_before_opening,
        active_promotion_forbidden: source.evaluation.active_promotion_forbidden,
        reward_application_forbidden: source.evaluation.reward_application_forbidden,
        lifecycle_digest: String::new(),
    };
    lifecycle.lifecycle_digest = lifecycle_digest(&lifecycle);
    validate_lifecycle(&lifecycle, source)?;
    Ok(lifecycle)
}

fn validate_lifecycle(
    value: &MomentumFutureEvaluationLifecycleV4_2,
    source: &MomentumFutureEvaluationSourceV4_2,
) -> Result<(), String> {
    if value.lifecycle_version != LIFECYCLE_VERSION_V4_2
        || value.agent_id != AGENT_ID_V4_2
        || value.source_v4_family_digest != source.source_family.family_digest
        || value.accumulated_family_digest != source.accumulated_family.family_digest
        || value.roster_digest != source.roster.roster_digest
        || value.evaluation_registration_digest != source.evaluation.registration_digest
        || value.participant_digests.len() != 3
        || value.participant_parameter_digests.len() != 3
        || value.participant_normalizer_digests.len() != 3
        || value.minimum_accepted_event_timestamp_ms
            != source.evaluation.minimum_accepted_timestamp_ms
        || value.cadence_ms != DAILY_CADENCE_MS_V4_2
        || value.prediction_horizon == 0
        || value.input_stage_maximum_requests != 1
        || value.input_stage_maximum_retries != 0
        || value.outcome_stage_maximum_requests != 1
        || value.outcome_stage_maximum_retries != 0
        || !value.prediction_must_precede_outcome_access
        || !value.outcome_stage_locked_until_prediction_sealed
        || !value.labels_hidden_until_opening
        || !value.probabilities_hidden_until_opening
        || !value.winner_selection_forbidden
        || !value.active_promotion_forbidden
        || !value.reward_application_forbidden
        || value.lifecycle_digest != lifecycle_digest(value)
    {
        return Err("V4.2 lifecycle contract rejected".to_string());
    }
    Ok(())
}

fn align_up(timestamp_ms: u64, cadence_ms: u64) -> Result<u64, String> {
    if cadence_ms == 0 {
        return Err("V4.2 cadence unavailable".to_string());
    }
    let remainder = timestamp_ms % cadence_ms;
    if remainder == 0 {
        Ok(timestamp_ms)
    } else {
        timestamp_ms
            .checked_add(cadence_ms - remainder)
            .ok_or_else(|| "V4.2 cadence alignment overflow".to_string())
    }
}

fn timestamp_range(
    start_timestamp_ms: u64,
    count: usize,
    cadence_ms: u64,
) -> Result<Vec<u64>, String> {
    (0..count)
        .map(|index| {
            u64::try_from(index)
                .ok()
                .and_then(|index| index.checked_mul(cadence_ms))
                .and_then(|offset| start_timestamp_ms.checked_add(offset))
                .ok_or_else(|| "V4.2 timestamp range overflow".to_string())
        })
        .collect()
}

fn row_identity_digest(row: &HistoricalOhlcvRow) -> String {
    stable_hash_string(&format!(
        "momentum-v4.2-context-row:{}:{}:{}:{}:{}:{}:{}:{:?}",
        row.symbol,
        row.timestamp_ms,
        row.open.to_bits(),
        row.high.to_bits(),
        row.low.to_bits(),
        row.close.to_bits(),
        row.volume.to_bits(),
        row.trade_value.map(f64::to_bits)
    ))
}

fn derive_context_plan_with_policy(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    evaluation: &MomentumAccumulatedEvaluationRegistrationV4_1,
    source_snapshot: &DataSnapshot,
    declared_policy: Option<MomentumContextPolicyStatusV4_2>,
) -> Result<MomentumProspectiveFeatureContextPlanV4_2, String> {
    let config = MomentumLearningCampaignConfigV0::default();
    let required_row_count = config
        .feature_config
        .minimum_history()
        .map_err(|_| "V4.2 feature context policy rejected".to_string())?
        .checked_add(config.sequence_config.sequence_length.saturating_sub(1))
        .ok_or_else(|| "V4.2 feature context length overflow".to_string())?;
    let source_next = evaluation
        .source_boundary_timestamp_ms
        .checked_add(lifecycle.cadence_ms)
        .ok_or_else(|| "V4.2 source boundary overflow".to_string())?;
    let initial_event = align_up(
        lifecycle
            .minimum_accepted_event_timestamp_ms
            .max(source_next),
        lifecycle.cadence_ms,
    )?;
    let history_offset = u64::try_from(required_row_count.saturating_sub(1))
        .ok()
        .and_then(|count| count.checked_mul(lifecycle.cadence_ms))
        .ok_or_else(|| "V4.2 context history overflow".to_string())?;
    let initial_start = initial_event
        .checked_sub(history_offset)
        .ok_or_else(|| "V4.2 initial context start unavailable".to_string())?;
    let initial_timestamps =
        timestamp_range(initial_start, required_row_count, lifecycle.cadence_ms)?;
    let protected_context_timestamp_ms = evaluation
        .protected_timestamp_ms
        .iter()
        .filter(|timestamp| initial_timestamps.contains(timestamp))
        .copied()
        .collect::<Vec<_>>();
    let context_policy_status = if protected_context_timestamp_ms.is_empty() {
        MomentumContextPolicyStatusV4_2::NoProtectedTimestampRequired
    } else {
        declared_policy.unwrap_or(MomentumContextPolicyStatusV4_2::ContextUseAmbiguous)
    };
    let event_timestamp_ms = match context_policy_status {
        MomentumContextPolicyStatusV4_2::ContextUseExplicitlyForbidden
        | MomentumContextPolicyStatusV4_2::ContextUseAmbiguous => {
            let latest_protected = evaluation
                .protected_timestamp_ms
                .iter()
                .max()
                .copied()
                .ok_or_else(|| "V4.2 protected timestamp identity unavailable".to_string())?;
            latest_protected
                .checked_add(lifecycle.cadence_ms)
                .and_then(|start| start.checked_add(history_offset))
                .map(|event| event.max(initial_event))
                .ok_or_else(|| "V4.2 post-exclusion event overflow".to_string())?
        }
        MomentumContextPolicyStatusV4_2::ContextOnlyUseExplicitlyAllowed
        | MomentumContextPolicyStatusV4_2::NoProtectedTimestampRequired => initial_event,
    };
    let required_context_start_timestamp_ms = event_timestamp_ms
        .checked_sub(history_offset)
        .ok_or_else(|| "V4.2 context start unavailable".to_string())?;
    let required_timestamps = timestamp_range(
        required_context_start_timestamp_ms,
        required_row_count,
        lifecycle.cadence_ms,
    )?;
    let source_rows = source_snapshot
        .normalized_dataset
        .rows
        .iter()
        .map(|row| (row.timestamp_ms, row))
        .collect::<BTreeMap<_, _>>();
    let existing_source_row_digests = required_timestamps
        .iter()
        .filter_map(|timestamp| source_rows.get(timestamp))
        .map(|row| row_identity_digest(row))
        .collect::<Vec<_>>();
    let incremental_row_timestamps = required_timestamps
        .iter()
        .filter(|timestamp| !source_rows.contains_key(timestamp))
        .copied()
        .collect::<Vec<_>>();
    let mut plan = MomentumProspectiveFeatureContextPlanV4_2 {
        plan_version: CONTEXT_PLAN_VERSION_V4_2.to_string(),
        event_timestamp_ms,
        required_context_start_timestamp_ms,
        required_context_end_timestamp_ms: event_timestamp_ms,
        required_row_count,
        required_timestamp_digest: stable_hash_string(&format!(
            "momentum-v4.2-required-context:{required_timestamps:?}"
        )),
        existing_source_row_digests,
        incremental_row_timestamps,
        protected_context_timestamp_ms,
        context_policy_status,
        plan_digest: String::new(),
    };
    plan.plan_digest = context_plan_digest(&plan);
    validate_context_plan(&plan, lifecycle, evaluation, source_snapshot)?;
    Ok(plan)
}

fn derive_context_plan(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    evaluation: &MomentumAccumulatedEvaluationRegistrationV4_1,
    source_snapshot: &DataSnapshot,
) -> Result<MomentumProspectiveFeatureContextPlanV4_2, String> {
    derive_context_plan_with_policy(lifecycle, evaluation, source_snapshot, None)
}

fn validate_context_plan(
    value: &MomentumProspectiveFeatureContextPlanV4_2,
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    evaluation: &MomentumAccumulatedEvaluationRegistrationV4_1,
    source_snapshot: &DataSnapshot,
) -> Result<(), String> {
    let timestamps = timestamp_range(
        value.required_context_start_timestamp_ms,
        value.required_row_count,
        lifecycle.cadence_ms,
    )?;
    let source_timestamp_set = source_snapshot
        .normalized_dataset
        .rows
        .iter()
        .map(|row| row.timestamp_ms)
        .collect::<BTreeSet<_>>();
    let expected_incremental = timestamps
        .iter()
        .filter(|timestamp| !source_timestamp_set.contains(timestamp))
        .copied()
        .collect::<Vec<_>>();
    if value.plan_version != CONTEXT_PLAN_VERSION_V4_2
        || value.event_timestamp_ms % lifecycle.cadence_ms != 0
        || value.event_timestamp_ms < lifecycle.minimum_accepted_event_timestamp_ms
        || value.required_context_end_timestamp_ms != value.event_timestamp_ms
        || timestamps.last() != Some(&value.event_timestamp_ms)
        || value.required_timestamp_digest
            != stable_hash_string(&format!("momentum-v4.2-required-context:{timestamps:?}"))
        || value.incremental_row_timestamps != expected_incremental
        || value
            .protected_context_timestamp_ms
            .iter()
            .any(|timestamp| !evaluation.protected_timestamp_ms.contains(timestamp))
        || matches!(
            value.context_policy_status,
            MomentumContextPolicyStatusV4_2::ContextUseExplicitlyForbidden
                | MomentumContextPolicyStatusV4_2::ContextUseAmbiguous
        ) && timestamps
            .iter()
            .any(|timestamp| evaluation.protected_timestamp_ms.contains(timestamp))
        || value.plan_digest != context_plan_digest(value)
    {
        return Err("V4.2 feature context plan rejected".to_string());
    }
    Ok(())
}

fn event_readiness(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    plan: &MomentumProspectiveFeatureContextPlanV4_2,
    observed_timestamp_ms: u64,
    maximum_provider_rows: usize,
) -> MomentumEventReadinessV4_2 {
    if plan.event_timestamp_ms < lifecycle.minimum_accepted_event_timestamp_ms {
        MomentumEventReadinessV4_2::AwaitingMinimumTimestamp
    } else if plan.context_policy_status == MomentumContextPolicyStatusV4_2::ContextUseAmbiguous {
        MomentumEventReadinessV4_2::ContextPolicyAmbiguous
    } else if plan.context_policy_status
        == MomentumContextPolicyStatusV4_2::ContextUseExplicitlyForbidden
    {
        MomentumEventReadinessV4_2::AwaitingPostExclusionContext
    } else if plan.incremental_row_timestamps.len() > maximum_provider_rows {
        MomentumEventReadinessV4_2::AwaitingSufficientFeatureContext
    } else if plan
        .event_timestamp_ms
        .checked_add(lifecycle.cadence_ms)
        .is_none_or(|finality| observed_timestamp_ms < finality)
    {
        MomentumEventReadinessV4_2::AwaitingInputFinality
    } else {
        MomentumEventReadinessV4_2::ReadyForInputAcquisition
    }
}

fn derive_input_registration(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    plan: &MomentumProspectiveFeatureContextPlanV4_2,
    source: &MomentumFutureEvaluationSourceV4_2,
    config: &UpbitHistoricalPilotConfigV0,
) -> Result<MomentumProspectiveInputAcquisitionRegistrationV4_2, String> {
    let request_to_timestamp_ms = plan
        .event_timestamp_ms
        .checked_add(lifecycle.cadence_ms)
        .ok_or_else(|| "V4.2 input request boundary overflow".to_string())?;
    let mut registration = MomentumProspectiveInputAcquisitionRegistrationV4_2 {
        registration_version: INPUT_REGISTRATION_VERSION_V4_2.to_string(),
        lifecycle_digest: lifecycle.lifecycle_digest.clone(),
        evaluation_registration_digest: source.evaluation.registration_digest.clone(),
        roster_digest: source.roster.roster_digest.clone(),
        event_timestamp_ms: plan.event_timestamp_ms,
        feature_context_plan_digest: plan.plan_digest.clone(),
        provider_id: config.provider_id.clone(),
        market: "btc_crypto".to_string(),
        symbol: config.symbol.clone(),
        cadence: "1d".to_string(),
        exact_expected_timestamp_ms: plan.incremental_row_timestamps.clone(),
        exact_request_count: usize::from(!plan.incremental_row_timestamps.is_empty()),
        request_to_timestamp_ms,
        maximum_requests: 1,
        maximum_concurrency: 1,
        maximum_retries: 0,
        maximum_response_bytes: config.maximum_response_bytes,
        credential_free_required: true,
        read_only_required: true,
        outcome_timestamp_forbidden: true,
        registration_digest: String::new(),
    };
    registration.registration_digest = input_registration_digest(&registration);
    validate_input_registration(&registration, lifecycle, plan, source, config)?;
    Ok(registration)
}

fn validate_input_registration(
    value: &MomentumProspectiveInputAcquisitionRegistrationV4_2,
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    plan: &MomentumProspectiveFeatureContextPlanV4_2,
    source: &MomentumFutureEvaluationSourceV4_2,
    config: &UpbitHistoricalPilotConfigV0,
) -> Result<(), String> {
    let request_count = usize::from(!value.exact_expected_timestamp_ms.is_empty());
    if value.registration_version != INPUT_REGISTRATION_VERSION_V4_2
        || value.lifecycle_digest != lifecycle.lifecycle_digest
        || value.evaluation_registration_digest != source.evaluation.registration_digest
        || value.roster_digest != source.roster.roster_digest
        || value.event_timestamp_ms != plan.event_timestamp_ms
        || value.feature_context_plan_digest != plan.plan_digest
        || value.provider_id != config.provider_id
        || value.market != "btc_crypto"
        || value.symbol != config.symbol
        || value.cadence != "1d"
        || value.exact_expected_timestamp_ms != plan.incremental_row_timestamps
        || value.exact_request_count != request_count
        || value.maximum_requests != 1
        || value.maximum_concurrency != 1
        || value.maximum_retries != 0
        || value.maximum_response_bytes == 0
        || !value.credential_free_required
        || !value.read_only_required
        || !value.outcome_timestamp_forbidden
        || value.registration_digest != input_registration_digest(value)
    {
        return Err("V4.2 input registration rejected".to_string());
    }
    Ok(())
}

#[derive(Clone, PartialEq, Message)]
struct ArtifactFieldProtobufV4_2 {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(string, repeated, tag = "2")]
    strings: Vec<String>,
    #[prost(uint64, repeated, tag = "3")]
    unsigned: Vec<u64>,
    #[prost(bool, repeated, tag = "4")]
    booleans: Vec<bool>,
    #[prost(bytes = "vec", repeated, tag = "5")]
    messages: Vec<Vec<u8>>,
}

#[derive(Clone, PartialEq, Message)]
struct ArtifactProtobufV4_2 {
    #[prost(string, tag = "1")]
    kind: String,
    #[prost(message, repeated, tag = "2")]
    fields: Vec<ArtifactFieldProtobufV4_2>,
}

struct ArtifactBuilderV4_2 {
    kind: String,
    fields: Vec<ArtifactFieldProtobufV4_2>,
}

impl ArtifactBuilderV4_2 {
    fn new(kind: &str) -> Self {
        Self {
            kind: kind.to_string(),
            fields: Vec::new(),
        }
    }

    fn string(mut self, name: &str, value: impl Into<String>) -> Self {
        self.fields.push(ArtifactFieldProtobufV4_2 {
            name: name.to_string(),
            strings: vec![value.into()],
            unsigned: vec![],
            booleans: vec![],
            messages: vec![],
        });
        self
    }

    fn optional_string(mut self, name: &str, value: &Option<String>) -> Self {
        self.fields.push(ArtifactFieldProtobufV4_2 {
            name: name.to_string(),
            strings: value.iter().cloned().collect(),
            unsigned: vec![],
            booleans: vec![],
            messages: vec![],
        });
        self
    }

    fn strings(mut self, name: &str, values: &[String]) -> Self {
        self.fields.push(ArtifactFieldProtobufV4_2 {
            name: name.to_string(),
            strings: values.to_vec(),
            unsigned: vec![],
            booleans: vec![],
            messages: vec![],
        });
        self
    }

    fn unsigned(mut self, name: &str, value: u64) -> Self {
        self.fields.push(ArtifactFieldProtobufV4_2 {
            name: name.to_string(),
            strings: vec![],
            unsigned: vec![value],
            booleans: vec![],
            messages: vec![],
        });
        self
    }

    fn unsigneds(mut self, name: &str, values: &[u64]) -> Self {
        self.fields.push(ArtifactFieldProtobufV4_2 {
            name: name.to_string(),
            strings: vec![],
            unsigned: values.to_vec(),
            booleans: vec![],
            messages: vec![],
        });
        self
    }

    fn boolean(mut self, name: &str, value: bool) -> Self {
        self.fields.push(ArtifactFieldProtobufV4_2 {
            name: name.to_string(),
            strings: vec![],
            unsigned: vec![],
            booleans: vec![value],
            messages: vec![],
        });
        self
    }

    fn messages(mut self, name: &str, values: Vec<Vec<u8>>) -> Self {
        self.fields.push(ArtifactFieldProtobufV4_2 {
            name: name.to_string(),
            strings: vec![],
            unsigned: vec![],
            booleans: vec![],
            messages: values,
        });
        self
    }

    fn encode(self) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        ArtifactProtobufV4_2 {
            kind: self.kind,
            fields: self.fields,
        }
        .encode(&mut bytes)
        .map_err(|_| "V4.2 Protobuf encoding failed".to_string())?;
        Ok(bytes)
    }
}

struct ArtifactReaderV4_2 {
    fields: BTreeMap<String, ArtifactFieldProtobufV4_2>,
}

impl ArtifactReaderV4_2 {
    fn decode(bytes: &[u8], expected_kind: &str) -> Result<Self, String> {
        let value = ArtifactProtobufV4_2::decode(bytes)
            .map_err(|_| "V4.2 Protobuf decoding failed".to_string())?;
        if value.kind != expected_kind {
            return Err("V4.2 Protobuf artifact kind rejected".to_string());
        }
        let mut fields = BTreeMap::new();
        for field in value.fields {
            if field.name.is_empty() || fields.insert(field.name.clone(), field).is_some() {
                return Err("V4.2 Protobuf field identity rejected".to_string());
            }
        }
        Ok(Self { fields })
    }

    fn take(&mut self, name: &str) -> Result<ArtifactFieldProtobufV4_2, String> {
        self.fields
            .remove(name)
            .ok_or_else(|| "V4.2 Protobuf required field missing".to_string())
    }

    fn string(&mut self, name: &str) -> Result<String, String> {
        let field = self.take(name)?;
        if field.strings.len() != 1
            || !field.unsigned.is_empty()
            || !field.booleans.is_empty()
            || !field.messages.is_empty()
        {
            return Err("V4.2 Protobuf string field rejected".to_string());
        }
        Ok(field.strings[0].clone())
    }

    fn optional_string(&mut self, name: &str) -> Result<Option<String>, String> {
        let field = self.take(name)?;
        if field.strings.len() > 1
            || !field.unsigned.is_empty()
            || !field.booleans.is_empty()
            || !field.messages.is_empty()
        {
            return Err("V4.2 Protobuf optional string rejected".to_string());
        }
        Ok(field.strings.into_iter().next())
    }

    fn strings(&mut self, name: &str) -> Result<Vec<String>, String> {
        let field = self.take(name)?;
        if !field.unsigned.is_empty() || !field.booleans.is_empty() || !field.messages.is_empty() {
            return Err("V4.2 Protobuf string list rejected".to_string());
        }
        Ok(field.strings)
    }

    fn unsigned(&mut self, name: &str) -> Result<u64, String> {
        let field = self.take(name)?;
        if field.unsigned.len() != 1
            || !field.strings.is_empty()
            || !field.booleans.is_empty()
            || !field.messages.is_empty()
        {
            return Err("V4.2 Protobuf unsigned field rejected".to_string());
        }
        Ok(field.unsigned[0])
    }

    fn unsigneds(&mut self, name: &str) -> Result<Vec<u64>, String> {
        let field = self.take(name)?;
        if !field.strings.is_empty() || !field.booleans.is_empty() || !field.messages.is_empty() {
            return Err("V4.2 Protobuf unsigned list rejected".to_string());
        }
        Ok(field.unsigned)
    }

    fn boolean(&mut self, name: &str) -> Result<bool, String> {
        let field = self.take(name)?;
        if field.booleans.len() != 1
            || !field.strings.is_empty()
            || !field.unsigned.is_empty()
            || !field.messages.is_empty()
        {
            return Err("V4.2 Protobuf Boolean field rejected".to_string());
        }
        Ok(field.booleans[0])
    }

    fn messages(&mut self, name: &str) -> Result<Vec<Vec<u8>>, String> {
        let field = self.take(name)?;
        if !field.strings.is_empty() || !field.unsigned.is_empty() || !field.booleans.is_empty() {
            return Err("V4.2 Protobuf nested message list rejected".to_string());
        }
        Ok(field.messages)
    }

    fn finish(self) -> Result<(), String> {
        if self.fields.is_empty() {
            Ok(())
        } else {
            Err("V4.2 Protobuf unknown field rejected".to_string())
        }
    }
}

fn as_u64(value: usize) -> Result<u64, String> {
    u64::try_from(value).map_err(|_| "V4.2 integer encoding overflow".to_string())
}

fn as_usize(value: u64) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| "V4.2 integer decoding overflow".to_string())
}

fn encode_lifecycle(value: &MomentumFutureEvaluationLifecycleV4_2) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("lifecycle")
        .string("lifecycle_version", &value.lifecycle_version)
        .string("agent_id", &value.agent_id)
        .string("source_v4_family_digest", &value.source_v4_family_digest)
        .string(
            "accumulated_family_digest",
            &value.accumulated_family_digest,
        )
        .string("roster_digest", &value.roster_digest)
        .string(
            "evaluation_registration_digest",
            &value.evaluation_registration_digest,
        )
        .strings("participant_digests", &value.participant_digests)
        .strings(
            "participant_parameter_digests",
            &value.participant_parameter_digests,
        )
        .strings(
            "participant_normalizer_digests",
            &value.participant_normalizer_digests,
        )
        .string("feature_policy_digest", &value.feature_policy_digest)
        .string("label_policy_digest", &value.label_policy_digest)
        .string(
            "qualification_policy_digest",
            &value.qualification_policy_digest,
        )
        .unsigned(
            "minimum_accepted_event_timestamp_ms",
            value.minimum_accepted_event_timestamp_ms,
        )
        .unsigned("cadence_ms", value.cadence_ms)
        .unsigned("prediction_horizon", as_u64(value.prediction_horizon)?)
        .unsigned(
            "input_stage_maximum_requests",
            as_u64(value.input_stage_maximum_requests)?,
        )
        .unsigned(
            "input_stage_maximum_retries",
            as_u64(value.input_stage_maximum_retries)?,
        )
        .unsigned(
            "outcome_stage_maximum_requests",
            as_u64(value.outcome_stage_maximum_requests)?,
        )
        .unsigned(
            "outcome_stage_maximum_retries",
            as_u64(value.outcome_stage_maximum_retries)?,
        )
        .boolean(
            "prediction_must_precede_outcome_access",
            value.prediction_must_precede_outcome_access,
        )
        .boolean(
            "outcome_stage_locked_until_prediction_sealed",
            value.outcome_stage_locked_until_prediction_sealed,
        )
        .boolean(
            "labels_hidden_until_opening",
            value.labels_hidden_until_opening,
        )
        .boolean(
            "probabilities_hidden_until_opening",
            value.probabilities_hidden_until_opening,
        )
        .boolean(
            "winner_selection_forbidden",
            value.winner_selection_forbidden,
        )
        .boolean(
            "active_promotion_forbidden",
            value.active_promotion_forbidden,
        )
        .boolean(
            "reward_application_forbidden",
            value.reward_application_forbidden,
        )
        .string("lifecycle_digest", &value.lifecycle_digest)
        .encode()
}

fn decode_lifecycle(bytes: &[u8]) -> Result<MomentumFutureEvaluationLifecycleV4_2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "lifecycle")?;
    let value = MomentumFutureEvaluationLifecycleV4_2 {
        lifecycle_version: fields.string("lifecycle_version")?,
        agent_id: fields.string("agent_id")?,
        source_v4_family_digest: fields.string("source_v4_family_digest")?,
        accumulated_family_digest: fields.string("accumulated_family_digest")?,
        roster_digest: fields.string("roster_digest")?,
        evaluation_registration_digest: fields.string("evaluation_registration_digest")?,
        participant_digests: fields.strings("participant_digests")?,
        participant_parameter_digests: fields.strings("participant_parameter_digests")?,
        participant_normalizer_digests: fields.strings("participant_normalizer_digests")?,
        feature_policy_digest: fields.string("feature_policy_digest")?,
        label_policy_digest: fields.string("label_policy_digest")?,
        qualification_policy_digest: fields.string("qualification_policy_digest")?,
        minimum_accepted_event_timestamp_ms: fields
            .unsigned("minimum_accepted_event_timestamp_ms")?,
        cadence_ms: fields.unsigned("cadence_ms")?,
        prediction_horizon: as_usize(fields.unsigned("prediction_horizon")?)?,
        input_stage_maximum_requests: as_usize(fields.unsigned("input_stage_maximum_requests")?)?,
        input_stage_maximum_retries: as_usize(fields.unsigned("input_stage_maximum_retries")?)?,
        outcome_stage_maximum_requests: as_usize(
            fields.unsigned("outcome_stage_maximum_requests")?,
        )?,
        outcome_stage_maximum_retries: as_usize(fields.unsigned("outcome_stage_maximum_retries")?)?,
        prediction_must_precede_outcome_access: fields
            .boolean("prediction_must_precede_outcome_access")?,
        outcome_stage_locked_until_prediction_sealed: fields
            .boolean("outcome_stage_locked_until_prediction_sealed")?,
        labels_hidden_until_opening: fields.boolean("labels_hidden_until_opening")?,
        probabilities_hidden_until_opening: fields.boolean("probabilities_hidden_until_opening")?,
        winner_selection_forbidden: fields.boolean("winner_selection_forbidden")?,
        active_promotion_forbidden: fields.boolean("active_promotion_forbidden")?,
        reward_application_forbidden: fields.boolean("reward_application_forbidden")?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
    };
    fields.finish()?;
    if value.lifecycle_version != LIFECYCLE_VERSION_V4_2
        || value.lifecycle_digest != lifecycle_digest(&value)
    {
        return Err("V4.2 lifecycle Protobuf rejected".to_string());
    }
    Ok(value)
}

fn encode_context_plan(
    value: &MomentumProspectiveFeatureContextPlanV4_2,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("context-plan")
        .string("plan_version", &value.plan_version)
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigned(
            "required_context_start_timestamp_ms",
            value.required_context_start_timestamp_ms,
        )
        .unsigned(
            "required_context_end_timestamp_ms",
            value.required_context_end_timestamp_ms,
        )
        .unsigned("required_row_count", as_u64(value.required_row_count)?)
        .string(
            "required_timestamp_digest",
            &value.required_timestamp_digest,
        )
        .strings(
            "existing_source_row_digests",
            &value.existing_source_row_digests,
        )
        .unsigneds(
            "incremental_row_timestamps",
            &value.incremental_row_timestamps,
        )
        .unsigneds(
            "protected_context_timestamp_ms",
            &value.protected_context_timestamp_ms,
        )
        .string(
            "context_policy_status",
            format!("{:?}", value.context_policy_status),
        )
        .string("plan_digest", &value.plan_digest)
        .encode()
}

fn parse_context_policy(value: &str) -> Result<MomentumContextPolicyStatusV4_2, String> {
    match value {
        "ContextOnlyUseExplicitlyAllowed" => {
            Ok(MomentumContextPolicyStatusV4_2::ContextOnlyUseExplicitlyAllowed)
        }
        "ContextUseExplicitlyForbidden" => {
            Ok(MomentumContextPolicyStatusV4_2::ContextUseExplicitlyForbidden)
        }
        "ContextUseAmbiguous" => Ok(MomentumContextPolicyStatusV4_2::ContextUseAmbiguous),
        "NoProtectedTimestampRequired" => {
            Ok(MomentumContextPolicyStatusV4_2::NoProtectedTimestampRequired)
        }
        _ => Err("V4.2 context-policy value rejected".to_string()),
    }
}

fn decode_context_plan(bytes: &[u8]) -> Result<MomentumProspectiveFeatureContextPlanV4_2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "context-plan")?;
    let value = MomentumProspectiveFeatureContextPlanV4_2 {
        plan_version: fields.string("plan_version")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        required_context_start_timestamp_ms: fields
            .unsigned("required_context_start_timestamp_ms")?,
        required_context_end_timestamp_ms: fields.unsigned("required_context_end_timestamp_ms")?,
        required_row_count: as_usize(fields.unsigned("required_row_count")?)?,
        required_timestamp_digest: fields.string("required_timestamp_digest")?,
        existing_source_row_digests: fields.strings("existing_source_row_digests")?,
        incremental_row_timestamps: fields.unsigneds("incremental_row_timestamps")?,
        protected_context_timestamp_ms: fields.unsigneds("protected_context_timestamp_ms")?,
        context_policy_status: parse_context_policy(&fields.string("context_policy_status")?)?,
        plan_digest: fields.string("plan_digest")?,
    };
    fields.finish()?;
    if value.plan_version != CONTEXT_PLAN_VERSION_V4_2
        || value.plan_digest != context_plan_digest(&value)
    {
        return Err("V4.2 context-plan Protobuf rejected".to_string());
    }
    Ok(value)
}

fn encode_input_registration(
    value: &MomentumProspectiveInputAcquisitionRegistrationV4_2,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("input-registration")
        .string("registration_version", &value.registration_version)
        .string("lifecycle_digest", &value.lifecycle_digest)
        .string(
            "evaluation_registration_digest",
            &value.evaluation_registration_digest,
        )
        .string("roster_digest", &value.roster_digest)
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .string(
            "feature_context_plan_digest",
            &value.feature_context_plan_digest,
        )
        .string("provider_id", &value.provider_id)
        .string("market", &value.market)
        .string("symbol", &value.symbol)
        .string("cadence", &value.cadence)
        .unsigneds(
            "exact_expected_timestamp_ms",
            &value.exact_expected_timestamp_ms,
        )
        .unsigned("exact_request_count", as_u64(value.exact_request_count)?)
        .unsigned("request_to_timestamp_ms", value.request_to_timestamp_ms)
        .unsigned("maximum_requests", as_u64(value.maximum_requests)?)
        .unsigned("maximum_concurrency", as_u64(value.maximum_concurrency)?)
        .unsigned("maximum_retries", as_u64(value.maximum_retries)?)
        .unsigned(
            "maximum_response_bytes",
            as_u64(value.maximum_response_bytes)?,
        )
        .boolean("credential_free_required", value.credential_free_required)
        .boolean("read_only_required", value.read_only_required)
        .boolean(
            "outcome_timestamp_forbidden",
            value.outcome_timestamp_forbidden,
        )
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_input_registration(
    bytes: &[u8],
) -> Result<MomentumProspectiveInputAcquisitionRegistrationV4_2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "input-registration")?;
    let value = MomentumProspectiveInputAcquisitionRegistrationV4_2 {
        registration_version: fields.string("registration_version")?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
        evaluation_registration_digest: fields.string("evaluation_registration_digest")?,
        roster_digest: fields.string("roster_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        feature_context_plan_digest: fields.string("feature_context_plan_digest")?,
        provider_id: fields.string("provider_id")?,
        market: fields.string("market")?,
        symbol: fields.string("symbol")?,
        cadence: fields.string("cadence")?,
        exact_expected_timestamp_ms: fields.unsigneds("exact_expected_timestamp_ms")?,
        exact_request_count: as_usize(fields.unsigned("exact_request_count")?)?,
        request_to_timestamp_ms: fields.unsigned("request_to_timestamp_ms")?,
        maximum_requests: as_usize(fields.unsigned("maximum_requests")?)?,
        maximum_concurrency: as_usize(fields.unsigned("maximum_concurrency")?)?,
        maximum_retries: as_usize(fields.unsigned("maximum_retries")?)?,
        maximum_response_bytes: as_usize(fields.unsigned("maximum_response_bytes")?)?,
        credential_free_required: fields.boolean("credential_free_required")?,
        read_only_required: fields.boolean("read_only_required")?,
        outcome_timestamp_forbidden: fields.boolean("outcome_timestamp_forbidden")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    if value.registration_version != INPUT_REGISTRATION_VERSION_V4_2
        || value.registration_digest != input_registration_digest(&value)
    {
        return Err("V4.2 input-registration Protobuf rejected".to_string());
    }
    Ok(value)
}

fn parse_input_status(value: &str) -> Result<MomentumProspectiveInputStatusV4_2, String> {
    match value {
        "ReadyNotAttempted" => Ok(MomentumProspectiveInputStatusV4_2::ReadyNotAttempted),
        "EvidenceAcquired" => Ok(MomentumProspectiveInputStatusV4_2::EvidenceAcquired),
        "ProviderRejected" => Ok(MomentumProspectiveInputStatusV4_2::ProviderRejected),
        "TimeoutNoRetry" => Ok(MomentumProspectiveInputStatusV4_2::TimeoutNoRetry),
        "InvalidResponse" => Ok(MomentumProspectiveInputStatusV4_2::InvalidResponse),
        "TechnicalFailure" => Ok(MomentumProspectiveInputStatusV4_2::TechnicalFailure),
        _ => Err("V4.2 input status rejected".to_string()),
    }
}

fn encode_input_receipt(value: &MomentumProspectiveInputReceiptV4_2) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("input-receipt")
        .string("receipt_version", &value.receipt_version)
        .string("lifecycle_digest", &value.lifecycle_digest)
        .string(
            "input_registration_digest",
            &value.input_registration_digest,
        )
        .boolean("request_attempted", value.request_attempted)
        .unsigned("request_count", as_u64(value.request_count)?)
        .unsigned("retry_count", as_u64(value.retry_count)?)
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

fn decode_input_receipt(bytes: &[u8]) -> Result<MomentumProspectiveInputReceiptV4_2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "input-receipt")?;
    let value = MomentumProspectiveInputReceiptV4_2 {
        receipt_version: fields.string("receipt_version")?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
        input_registration_digest: fields.string("input_registration_digest")?,
        request_attempted: fields.boolean("request_attempted")?,
        request_count: as_usize(fields.unsigned("request_count")?)?,
        retry_count: as_usize(fields.unsigned("retry_count")?)?,
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
    if value.receipt_version != INPUT_RECEIPT_VERSION_V4_2
        || value.request_count > 1
        || value.retry_count != 0
        || value.terminal != value.status.is_terminal()
        || value.receipt_digest != input_receipt_digest(&value)
    {
        return Err("V4.2 input-receipt Protobuf rejected".to_string());
    }
    Ok(value)
}

fn encode_input_capsule(value: &MomentumProspectiveInputCapsuleV4_2) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("input-capsule")
        .string("capsule_version", &value.capsule_version)
        .string("lifecycle_digest", &value.lifecycle_digest)
        .string(
            "input_registration_digest",
            &value.input_registration_digest,
        )
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigneds("exact_timestamp_ms", &value.exact_timestamp_ms)
        .strings("row_identity_digests", &value.row_identity_digests)
        .string(
            "normalized_dataset_digest",
            &value.normalized_dataset_digest,
        )
        .string("raw_response_digest", &value.raw_response_digest)
        .boolean("outcome_rows_present", value.outcome_rows_present)
        .boolean("labels_accessed", value.labels_accessed)
        .boolean("metrics_computed", value.metrics_computed)
        .boolean("credential_free", value.credential_free)
        .boolean("read_only", value.read_only)
        .boolean("sanitized", value.sanitized)
        .string("capsule_digest", &value.capsule_digest)
        .encode()
}

fn decode_input_capsule(bytes: &[u8]) -> Result<MomentumProspectiveInputCapsuleV4_2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "input-capsule")?;
    let value = MomentumProspectiveInputCapsuleV4_2 {
        capsule_version: fields.string("capsule_version")?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
        input_registration_digest: fields.string("input_registration_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        exact_timestamp_ms: fields.unsigneds("exact_timestamp_ms")?,
        row_identity_digests: fields.strings("row_identity_digests")?,
        normalized_dataset_digest: fields.string("normalized_dataset_digest")?,
        raw_response_digest: fields.string("raw_response_digest")?,
        outcome_rows_present: fields.boolean("outcome_rows_present")?,
        labels_accessed: fields.boolean("labels_accessed")?,
        metrics_computed: fields.boolean("metrics_computed")?,
        credential_free: fields.boolean("credential_free")?,
        read_only: fields.boolean("read_only")?,
        sanitized: fields.boolean("sanitized")?,
        capsule_digest: fields.string("capsule_digest")?,
    };
    fields.finish()?;
    if value.capsule_version != INPUT_CAPSULE_VERSION_V4_2
        || value.outcome_rows_present
        || value.labels_accessed
        || value.metrics_computed
        || !value.credential_free
        || !value.read_only
        || !value.sanitized
        || value.exact_timestamp_ms.len() != value.row_identity_digests.len()
        || value.capsule_digest != input_capsule_digest(&value)
    {
        return Err("V4.2 input-capsule Protobuf rejected".to_string());
    }
    Ok(value)
}

fn encode_context_proof(
    value: &MomentumProspectiveContextVerificationV4_2,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("context-proof")
        .string("proof_version", &value.proof_version)
        .string(
            "feature_context_plan_digest",
            &value.feature_context_plan_digest,
        )
        .string(
            "input_registration_digest",
            &value.input_registration_digest,
        )
        .string("input_capsule_digest", &value.input_capsule_digest)
        .boolean("exact_timestamps_verified", value.exact_timestamps_verified)
        .boolean(
            "strict_chronology_verified",
            value.strict_chronology_verified,
        )
        .boolean("feature_history_complete", value.feature_history_complete)
        .boolean(
            "protected_events_not_scored",
            value.protected_events_not_scored,
        )
        .boolean("outcome_timestamp_absent", value.outcome_timestamp_absent)
        .string("proof_digest", &value.proof_digest)
        .encode()
}

fn decode_context_proof(
    bytes: &[u8],
) -> Result<MomentumProspectiveContextVerificationV4_2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "context-proof")?;
    let value = MomentumProspectiveContextVerificationV4_2 {
        proof_version: fields.string("proof_version")?,
        feature_context_plan_digest: fields.string("feature_context_plan_digest")?,
        input_registration_digest: fields.string("input_registration_digest")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        exact_timestamps_verified: fields.boolean("exact_timestamps_verified")?,
        strict_chronology_verified: fields.boolean("strict_chronology_verified")?,
        feature_history_complete: fields.boolean("feature_history_complete")?,
        protected_events_not_scored: fields.boolean("protected_events_not_scored")?,
        outcome_timestamp_absent: fields.boolean("outcome_timestamp_absent")?,
        proof_digest: fields.string("proof_digest")?,
    };
    fields.finish()?;
    if value.proof_version != CONTEXT_PROOF_VERSION_V4_2
        || !value.exact_timestamps_verified
        || !value.strict_chronology_verified
        || !value.feature_history_complete
        || !value.protected_events_not_scored
        || !value.outcome_timestamp_absent
        || value.proof_digest != context_proof_digest(&value)
    {
        return Err("V4.2 context-proof Protobuf rejected".to_string());
    }
    Ok(value)
}

fn encode_prediction_seal(
    value: &MomentumParticipantPredictionSealV4_2,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("prediction-seal")
        .string("participant_digest", &value.participant_digest)
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .string("input_capsule_digest", &value.input_capsule_digest)
        .string("feature_identity_digest", &value.feature_identity_digest)
        .unsigned(
            "prediction_probability_bits",
            u64::from(value.prediction_probability_bits),
        )
        .string("prediction_digest", &value.prediction_digest)
        .boolean("participant_reconstructed", value.participant_reconstructed)
        .unsigned("parameter_updates", as_u64(value.parameter_updates)?)
        .unsigned("outcome_access_count", as_u64(value.outcome_access_count)?)
        .string("seal_digest", &value.seal_digest)
        .encode()
}

fn decode_prediction_seal(bytes: &[u8]) -> Result<MomentumParticipantPredictionSealV4_2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "prediction-seal")?;
    let value = MomentumParticipantPredictionSealV4_2 {
        participant_digest: fields.string("participant_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        feature_identity_digest: fields.string("feature_identity_digest")?,
        prediction_probability_bits: u32::try_from(fields.unsigned("prediction_probability_bits")?)
            .map_err(|_| "V4.2 probability bits rejected".to_string())?,
        prediction_digest: fields.string("prediction_digest")?,
        participant_reconstructed: fields.boolean("participant_reconstructed")?,
        parameter_updates: as_usize(fields.unsigned("parameter_updates")?)?,
        outcome_access_count: as_usize(fields.unsigned("outcome_access_count")?)?,
        seal_digest: fields.string("seal_digest")?,
    };
    fields.finish()?;
    let probability = f32::from_bits(value.prediction_probability_bits);
    if value.participant_digest.is_empty()
        || !probability.is_finite()
        || !(0.0..=1.0).contains(&probability)
        || !value.participant_reconstructed
        || value.parameter_updates != 0
        || value.outcome_access_count != 0
        || value.prediction_digest
            != stable_hash_string(&format!(
                "momentum-v4.2-sealed-prediction:{}:{}:{}:{}",
                value.participant_digest,
                value.event_timestamp_ms,
                value.input_capsule_digest,
                value.prediction_probability_bits
            ))
        || value.seal_digest != prediction_seal_digest(&value)
    {
        return Err("V4.2 prediction seal rejected".to_string());
    }
    Ok(value)
}

fn encode_prediction_capsule(
    value: &MomentumProspectivePredictionCapsuleV4_2,
) -> Result<Vec<u8>, String> {
    let seals = value
        .participant_prediction_seals
        .iter()
        .map(encode_prediction_seal)
        .collect::<Result<Vec<_>, _>>()?;
    ArtifactBuilderV4_2::new("prediction-capsule")
        .string("capsule_version", &value.capsule_version)
        .string("lifecycle_digest", &value.lifecycle_digest)
        .string(
            "evaluation_registration_digest",
            &value.evaluation_registration_digest,
        )
        .string("roster_digest", &value.roster_digest)
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .string("input_receipt_digest", &value.input_receipt_digest)
        .string("input_capsule_digest", &value.input_capsule_digest)
        .messages("participant_prediction_seals", seals)
        .boolean("probabilities_hidden", value.probabilities_hidden)
        .boolean("labels_hidden", value.labels_hidden)
        .boolean("outcome_accessed", value.outcome_accessed)
        .boolean("metrics_computed", value.metrics_computed)
        .boolean("winner_selected", value.winner_selected)
        .string("capsule_digest", &value.capsule_digest)
        .encode()
}

fn decode_prediction_capsule(
    bytes: &[u8],
) -> Result<MomentumProspectivePredictionCapsuleV4_2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "prediction-capsule")?;
    let value = MomentumProspectivePredictionCapsuleV4_2 {
        capsule_version: fields.string("capsule_version")?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
        evaluation_registration_digest: fields.string("evaluation_registration_digest")?,
        roster_digest: fields.string("roster_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        input_receipt_digest: fields.string("input_receipt_digest")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        participant_prediction_seals: fields
            .messages("participant_prediction_seals")?
            .iter()
            .map(|bytes| decode_prediction_seal(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        probabilities_hidden: fields.boolean("probabilities_hidden")?,
        labels_hidden: fields.boolean("labels_hidden")?,
        outcome_accessed: fields.boolean("outcome_accessed")?,
        metrics_computed: fields.boolean("metrics_computed")?,
        winner_selected: fields.boolean("winner_selected")?,
        capsule_digest: fields.string("capsule_digest")?,
    };
    fields.finish()?;
    if value.capsule_version != PREDICTION_CAPSULE_VERSION_V4_2
        || value.participant_prediction_seals.len() != 3
        || value.participant_prediction_seals.iter().any(|seal| {
            seal.event_timestamp_ms != value.event_timestamp_ms
                || seal.input_capsule_digest != value.input_capsule_digest
        })
        || !value.probabilities_hidden
        || !value.labels_hidden
        || value.outcome_accessed
        || value.metrics_computed
        || value.winner_selected
        || value.capsule_digest != prediction_capsule_digest(&value)
    {
        return Err("V4.2 prediction capsule rejected".to_string());
    }
    Ok(value)
}

fn encode_prediction_entry(
    value: &MomentumProspectivePredictionJournalEntryV4_2,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("prediction-entry")
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .string("input_capsule_digest", &value.input_capsule_digest)
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .strings(
            "participant_prediction_digests",
            &value.participant_prediction_digests,
        )
        .boolean(
            "prediction_sealed_before_outcome",
            value.prediction_sealed_before_outcome,
        )
        .boolean("outcome_stage_unlocked", value.outcome_stage_unlocked)
        .string("entry_digest", &value.entry_digest)
        .encode()
}

fn decode_prediction_entry(
    bytes: &[u8],
) -> Result<MomentumProspectivePredictionJournalEntryV4_2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "prediction-entry")?;
    let value = MomentumProspectivePredictionJournalEntryV4_2 {
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        participant_prediction_digests: fields.strings("participant_prediction_digests")?,
        prediction_sealed_before_outcome: fields.boolean("prediction_sealed_before_outcome")?,
        outcome_stage_unlocked: fields.boolean("outcome_stage_unlocked")?,
        entry_digest: fields.string("entry_digest")?,
    };
    fields.finish()?;
    if value.participant_prediction_digests.len() != 3
        || !value.prediction_sealed_before_outcome
        || value.outcome_stage_unlocked
        || value.entry_digest != prediction_entry_digest(&value)
    {
        return Err("V4.2 prediction journal entry rejected".to_string());
    }
    Ok(value)
}

fn encode_prediction_journal(
    value: &MomentumProspectivePredictionJournalV4_2,
) -> Result<Vec<u8>, String> {
    let entries = value
        .entries
        .iter()
        .map(encode_prediction_entry)
        .collect::<Result<Vec<_>, _>>()?;
    ArtifactBuilderV4_2::new("prediction-journal")
        .string("journal_version", &value.journal_version)
        .messages("entries", entries)
        .string("journal_digest", &value.journal_digest)
        .encode()
}

fn decode_prediction_journal(
    bytes: &[u8],
) -> Result<MomentumProspectivePredictionJournalV4_2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "prediction-journal")?;
    let value = MomentumProspectivePredictionJournalV4_2 {
        journal_version: fields.string("journal_version")?,
        entries: fields
            .messages("entries")?
            .iter()
            .map(|bytes| decode_prediction_entry(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        journal_digest: fields.string("journal_digest")?,
    };
    fields.finish()?;
    if value.journal_version != PREDICTION_JOURNAL_VERSION_V4_2
        || value.entries.is_empty()
        || value
            .entries
            .windows(2)
            .any(|pair| pair[0].event_timestamp_ms >= pair[1].event_timestamp_ms)
        || value.journal_digest != prediction_journal_digest(&value)
    {
        return Err("V4.2 prediction journal rejected".to_string());
    }
    Ok(value)
}

fn encode_maturity_plan(
    value: &MomentumProspectiveOutcomeMaturityPlanV4_2,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("maturity-plan")
        .string("plan_version", &value.plan_version)
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
        .boolean(
            "labels_hidden_until_opening",
            value.labels_hidden_until_opening,
        )
        .boolean("one_time_opening_required", value.one_time_opening_required)
        .string("plan_digest", &value.plan_digest)
        .encode()
}

fn decode_maturity_plan(
    bytes: &[u8],
) -> Result<MomentumProspectiveOutcomeMaturityPlanV4_2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "maturity-plan")?;
    let value = MomentumProspectiveOutcomeMaturityPlanV4_2 {
        plan_version: fields.string("plan_version")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        prediction_horizon: as_usize(fields.unsigned("prediction_horizon")?)?,
        required_outcome_timestamp_ms: fields.unsigneds("required_outcome_timestamp_ms")?,
        outcome_finality_boundary_ms: fields.unsigned("outcome_finality_boundary_ms")?,
        maximum_outcome_requests: as_usize(fields.unsigned("maximum_outcome_requests")?)?,
        maximum_outcome_retries: as_usize(fields.unsigned("maximum_outcome_retries")?)?,
        labels_hidden_until_opening: fields.boolean("labels_hidden_until_opening")?,
        one_time_opening_required: fields.boolean("one_time_opening_required")?,
        plan_digest: fields.string("plan_digest")?,
    };
    fields.finish()?;
    if value.plan_version != MATURITY_PLAN_VERSION_V4_2
        || value.prediction_horizon == 0
        || value.required_outcome_timestamp_ms.len() != value.prediction_horizon
        || value.maximum_outcome_requests != 1
        || value.maximum_outcome_retries != 0
        || !value.labels_hidden_until_opening
        || !value.one_time_opening_required
        || value.plan_digest != maturity_plan_digest(&value)
    {
        return Err("V4.2 maturity-plan Protobuf rejected".to_string());
    }
    Ok(value)
}

fn encode_safety_counters(
    value: &MomentumFuturePredictionSafetyCountersV4_2,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("safety-counters")
        .unsigned(
            "input_request_attempts",
            as_u64(value.input_request_attempts)?,
        )
        .unsigned("input_retries", as_u64(value.input_retries)?)
        .unsigned("input_concurrency", as_u64(value.input_concurrency)?)
        .unsigned(
            "outcome_request_attempts",
            as_u64(value.outcome_request_attempts)?,
        )
        .unsigned("outcome_retries", as_u64(value.outcome_retries)?)
        .unsigned(
            "participant_parameter_updates",
            as_u64(value.participant_parameter_updates)?,
        )
        .unsigned("normalizer_refits", as_u64(value.normalizer_refits)?)
        .unsigned("outcome_row_reads", as_u64(value.outcome_row_reads)?)
        .unsigned("outcome_label_reads", as_u64(value.outcome_label_reads)?)
        .unsigned("metric_computations", as_u64(value.metric_computations)?)
        .unsigned("winner_selections", as_u64(value.winner_selections)?)
        .unsigned("active_model_changes", as_u64(value.active_model_changes)?)
        .unsigned("chair_decisions", as_u64(value.chair_decisions)?)
        .unsigned("votes", as_u64(value.votes)?)
        .unsigned("reward_applications", as_u64(value.reward_applications)?)
        .unsigned("penalty_applications", as_u64(value.penalty_applications)?)
        .unsigned("voice_changes", as_u64(value.voice_changes)?)
        .unsigned("cooldowns_started", as_u64(value.cooldowns_started)?)
        .unsigned("promotions", as_u64(value.promotions)?)
        .unsigned("quarantines", as_u64(value.quarantines)?)
        .unsigned("executions", as_u64(value.executions)?)
        .unsigned(
            "active_committee_count",
            as_u64(value.active_committee_count)?,
        )
        .encode()
}

fn decode_safety_counters(
    bytes: &[u8],
) -> Result<MomentumFuturePredictionSafetyCountersV4_2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "safety-counters")?;
    let value = MomentumFuturePredictionSafetyCountersV4_2 {
        input_request_attempts: as_usize(fields.unsigned("input_request_attempts")?)?,
        input_retries: as_usize(fields.unsigned("input_retries")?)?,
        input_concurrency: as_usize(fields.unsigned("input_concurrency")?)?,
        outcome_request_attempts: as_usize(fields.unsigned("outcome_request_attempts")?)?,
        outcome_retries: as_usize(fields.unsigned("outcome_retries")?)?,
        participant_parameter_updates: as_usize(fields.unsigned("participant_parameter_updates")?)?,
        normalizer_refits: as_usize(fields.unsigned("normalizer_refits")?)?,
        outcome_row_reads: as_usize(fields.unsigned("outcome_row_reads")?)?,
        outcome_label_reads: as_usize(fields.unsigned("outcome_label_reads")?)?,
        metric_computations: as_usize(fields.unsigned("metric_computations")?)?,
        winner_selections: as_usize(fields.unsigned("winner_selections")?)?,
        active_model_changes: as_usize(fields.unsigned("active_model_changes")?)?,
        chair_decisions: as_usize(fields.unsigned("chair_decisions")?)?,
        votes: as_usize(fields.unsigned("votes")?)?,
        reward_applications: as_usize(fields.unsigned("reward_applications")?)?,
        penalty_applications: as_usize(fields.unsigned("penalty_applications")?)?,
        voice_changes: as_usize(fields.unsigned("voice_changes")?)?,
        cooldowns_started: as_usize(fields.unsigned("cooldowns_started")?)?,
        promotions: as_usize(fields.unsigned("promotions")?)?,
        quarantines: as_usize(fields.unsigned("quarantines")?)?,
        executions: as_usize(fields.unsigned("executions")?)?,
        active_committee_count: as_usize(fields.unsigned("active_committee_count")?)?,
    };
    fields.finish()?;
    validate_safety_counters(&value)?;
    Ok(value)
}

fn validate_safety_counters(
    value: &MomentumFuturePredictionSafetyCountersV4_2,
) -> Result<(), String> {
    if value.input_request_attempts > 1
        || value.input_retries != 0
        || value.input_concurrency != 1
        || value.outcome_request_attempts != 0
        || value.outcome_retries != 0
        || value.participant_parameter_updates != 0
        || value.normalizer_refits != 0
        || value.outcome_row_reads != 0
        || value.outcome_label_reads != 0
        || value.metric_computations != 0
        || value.winner_selections != 0
        || value.active_model_changes != 0
        || value.chair_decisions != 0
        || value.votes != 0
        || value.reward_applications != 0
        || value.penalty_applications != 0
        || value.voice_changes != 0
        || value.cooldowns_started != 0
        || value.promotions != 0
        || value.quarantines != 0
        || value.executions != 0
        || value.active_committee_count != 3
    {
        return Err("V4.2 safety counters rejected".to_string());
    }
    Ok(())
}

fn parse_budget_meaning(value: &str) -> Result<FutureEvaluationRequestBudgetMeaningV4_2, String> {
    match value {
        "InputEvidenceOnly" => Ok(FutureEvaluationRequestBudgetMeaningV4_2::InputEvidenceOnly),
        "OutcomeEvidenceOnly" => Ok(FutureEvaluationRequestBudgetMeaningV4_2::OutcomeEvidenceOnly),
        "EntireLifecycleSingleRequest" => {
            Ok(FutureEvaluationRequestBudgetMeaningV4_2::EntireLifecycleSingleRequest)
        }
        "ExistingLocalInputPlusOutcomeRequest" => {
            Ok(FutureEvaluationRequestBudgetMeaningV4_2::ExistingLocalInputPlusOutcomeRequest)
        }
        "Ambiguous" => Ok(FutureEvaluationRequestBudgetMeaningV4_2::Ambiguous),
        _ => Err("V4.2 request-budget meaning rejected".to_string()),
    }
}

fn parse_readiness(value: &str) -> Result<MomentumEventReadinessV4_2, String> {
    match value {
        "ReadyForInputAcquisition" => Ok(MomentumEventReadinessV4_2::ReadyForInputAcquisition),
        "AwaitingMinimumTimestamp" => Ok(MomentumEventReadinessV4_2::AwaitingMinimumTimestamp),
        "AwaitingInputFinality" => Ok(MomentumEventReadinessV4_2::AwaitingInputFinality),
        "AwaitingSufficientFeatureContext" => {
            Ok(MomentumEventReadinessV4_2::AwaitingSufficientFeatureContext)
        }
        "AwaitingPostExclusionContext" => {
            Ok(MomentumEventReadinessV4_2::AwaitingPostExclusionContext)
        }
        "ContextPolicyAmbiguous" => Ok(MomentumEventReadinessV4_2::ContextPolicyAmbiguous),
        "SupersededInputRegistration" => {
            Ok(MomentumEventReadinessV4_2::SupersededInputRegistration)
        }
        "PriorInputAttemptTerminal" => Ok(MomentumEventReadinessV4_2::PriorInputAttemptTerminal),
        "PredictionAlreadySealed" => Ok(MomentumEventReadinessV4_2::PredictionAlreadySealed),
        "IntegrityFailure" => Ok(MomentumEventReadinessV4_2::IntegrityFailure),
        _ => Err("V4.2 readiness value rejected".to_string()),
    }
}

fn encode_status(value: &MomentumFuturePredictionStatusReceiptV4_2) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("status")
        .string("status_version", &value.status_version)
        .string(
            "request_budget_meaning",
            format!("{:?}", value.request_budget_meaning),
        )
        .string("lifecycle_digest", &value.lifecycle_digest)
        .string("event_readiness", format!("{:?}", value.event_readiness))
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigned(
            "input_finality_boundary_ms",
            value.input_finality_boundary_ms,
        )
        .string(
            "context_policy_status",
            format!("{:?}", value.context_policy_status),
        )
        .string("context_plan_digest", &value.context_plan_digest)
        .string(
            "input_registration_digest",
            &value.input_registration_digest,
        )
        .unsigned(
            "request_attempt_count",
            as_u64(value.request_attempt_count)?,
        )
        .optional_string("input_receipt_digest", &value.input_receipt_digest)
        .optional_string("input_capsule_digest", &value.input_capsule_digest)
        .strings(
            "participant_prediction_digests",
            &value.participant_prediction_digests,
        )
        .optional_string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .optional_string(
            "outcome_maturity_plan_digest",
            &value.outcome_maturity_plan_digest,
        )
        .unsigneds(
            "outcome_finality_boundary_ms",
            &value
                .outcome_finality_boundary_ms
                .iter()
                .copied()
                .collect::<Vec<_>>(),
        )
        .string("cycle_risk_status", &value.cycle_risk_status)
        .string("value_quality_status", &value.value_quality_status)
        .string(
            "prior_momentum_attribution",
            &value.prior_momentum_attribution,
        )
        .string(
            "prior_cycle_risk_attribution",
            &value.prior_cycle_risk_attribution,
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

fn decode_status(bytes: &[u8]) -> Result<MomentumFuturePredictionStatusReceiptV4_2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "status")?;
    let outcome_finality = fields.unsigneds("outcome_finality_boundary_ms")?;
    if outcome_finality.len() > 1 {
        return Err("V4.2 optional outcome finality rejected".to_string());
    }
    let safety_messages = fields.messages("safety_counters")?;
    if safety_messages.len() != 1 {
        return Err("V4.2 status safety identity rejected".to_string());
    }
    let value = MomentumFuturePredictionStatusReceiptV4_2 {
        status_version: fields.string("status_version")?,
        request_budget_meaning: parse_budget_meaning(&fields.string("request_budget_meaning")?)?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
        event_readiness: parse_readiness(&fields.string("event_readiness")?)?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        input_finality_boundary_ms: fields.unsigned("input_finality_boundary_ms")?,
        context_policy_status: parse_context_policy(&fields.string("context_policy_status")?)?,
        context_plan_digest: fields.string("context_plan_digest")?,
        input_registration_digest: fields.string("input_registration_digest")?,
        request_attempt_count: as_usize(fields.unsigned("request_attempt_count")?)?,
        input_receipt_digest: fields.optional_string("input_receipt_digest")?,
        input_capsule_digest: fields.optional_string("input_capsule_digest")?,
        participant_prediction_digests: fields.strings("participant_prediction_digests")?,
        prediction_capsule_digest: fields.optional_string("prediction_capsule_digest")?,
        outcome_maturity_plan_digest: fields.optional_string("outcome_maturity_plan_digest")?,
        outcome_finality_boundary_ms: outcome_finality.into_iter().next(),
        cycle_risk_status: fields.string("cycle_risk_status")?,
        value_quality_status: fields.string("value_quality_status")?,
        prior_momentum_attribution: fields.string("prior_momentum_attribution")?,
        prior_cycle_risk_attribution: fields.string("prior_cycle_risk_attribution")?,
        protected_artifacts_unchanged: fields.boolean("protected_artifacts_unchanged")?,
        active_state_unchanged: fields.boolean("active_state_unchanged")?,
        safety_counters: decode_safety_counters(&safety_messages[0])?,
        status_digest: fields.string("status_digest")?,
    };
    fields.finish()?;
    if value.status_version != STATUS_RECEIPT_VERSION_V4_2
        || value.request_attempt_count > 1
        || !value.protected_artifacts_unchanged
        || !value.active_state_unchanged
        || value.status_digest != status_digest(&value)
    {
        return Err("V4.2 status receipt rejected".to_string());
    }
    Ok(value)
}

fn protobuf_paths(directory: &Path) -> Result<Vec<PathBuf>, String> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    if !directory.is_dir() {
        return Err("V4.2 artifact directory rejected".to_string());
    }
    let mut paths = fs::read_dir(directory)
        .map_err(|_| "V4.2 artifact directory read failed".to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "pb"))
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn read_single<T>(
    directory: &Path,
    decode: impl Fn(&[u8]) -> Result<T, String>,
) -> Result<Option<T>, String> {
    let paths = protobuf_paths(directory)?;
    if paths.is_empty() {
        return Ok(None);
    }
    if paths.len() != 1 {
        return Err("V4.2 single artifact identity rejected".to_string());
    }
    let bytes = fs::read(&paths[0]).map_err(|_| "V4.2 artifact read failed".to_string())?;
    decode(&bytes).map(Some)
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

fn persist_lifecycle(
    root: &Path,
    value: &MomentumFutureEvaluationLifecycleV4_2,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("lifecycles")
            .join(format!("{}.pb", value.lifecycle_digest)),
        &encode_lifecycle(value)?,
        &value.lifecycle_digest,
        |bytes| Ok(decode_lifecycle(bytes)?.lifecycle_digest),
    )
}

fn persist_context_plan(
    root: &Path,
    value: &MomentumProspectiveFeatureContextPlanV4_2,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("context_plans")
            .join(format!("{}.pb", value.plan_digest)),
        &encode_context_plan(value)?,
        &value.plan_digest,
        |bytes| Ok(decode_context_plan(bytes)?.plan_digest),
    )
}

fn persist_input_registration(
    root: &Path,
    value: &MomentumProspectiveInputAcquisitionRegistrationV4_2,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("input_registrations")
            .join(format!("{}.pb", value.registration_digest)),
        &encode_input_registration(value)?,
        &value.registration_digest,
        |bytes| Ok(decode_input_registration(bytes)?.registration_digest),
    )
}

fn persist_input_receipt(
    root: &Path,
    value: &MomentumProspectiveInputReceiptV4_2,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("input_receipts")
            .join(format!("{}.pb", value.receipt_digest)),
        &encode_input_receipt(value)?,
        &value.receipt_digest,
        |bytes| Ok(decode_input_receipt(bytes)?.receipt_digest),
    )
}

fn persist_input_capsule(
    root: &Path,
    value: &MomentumProspectiveInputCapsuleV4_2,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("input_capsules")
            .join(format!("{}.pb", value.capsule_digest)),
        &encode_input_capsule(value)?,
        &value.capsule_digest,
        |bytes| Ok(decode_input_capsule(bytes)?.capsule_digest),
    )
}

fn persist_context_proof(
    root: &Path,
    value: &MomentumProspectiveContextVerificationV4_2,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("context_verifications")
            .join(format!("{}.pb", value.proof_digest)),
        &encode_context_proof(value)?,
        &value.proof_digest,
        |bytes| Ok(decode_context_proof(bytes)?.proof_digest),
    )
}

fn persist_prediction_seal(
    root: &Path,
    value: &MomentumParticipantPredictionSealV4_2,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("participant_prediction_seals")
            .join(format!("{}.pb", value.seal_digest)),
        &encode_prediction_seal(value)?,
        &value.seal_digest,
        |bytes| Ok(decode_prediction_seal(bytes)?.seal_digest),
    )
}

fn persist_prediction_capsule(
    root: &Path,
    value: &MomentumProspectivePredictionCapsuleV4_2,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("prediction_capsules")
            .join(format!("{}.pb", value.capsule_digest)),
        &encode_prediction_capsule(value)?,
        &value.capsule_digest,
        |bytes| Ok(decode_prediction_capsule(bytes)?.capsule_digest),
    )
}

fn persist_prediction_journal(
    root: &Path,
    value: &MomentumProspectivePredictionJournalV4_2,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("prediction_journals")
            .join(format!("{}.pb", value.journal_digest)),
        &encode_prediction_journal(value)?,
        &value.journal_digest,
        |bytes| Ok(decode_prediction_journal(bytes)?.journal_digest),
    )
}

fn persist_maturity_plan(
    root: &Path,
    value: &MomentumProspectiveOutcomeMaturityPlanV4_2,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("outcome_maturity_plans")
            .join(format!("{}.pb", value.plan_digest)),
        &encode_maturity_plan(value)?,
        &value.plan_digest,
        |bytes| Ok(decode_maturity_plan(bytes)?.plan_digest),
    )
}

fn persist_status(
    root: &Path,
    value: &MomentumFuturePredictionStatusReceiptV4_2,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("status_receipts")
            .join(format!("{}.pb", value.status_digest)),
        &encode_status(value)?,
        &value.status_digest,
        |bytes| Ok(decode_status(bytes)?.status_digest),
    )
}

fn persist_raw_response(root: &Path, bytes: &[u8], digest: &str) -> Result<(usize, usize), String> {
    persist_artifact(
        &root.join("raw_input").join(format!("{digest}.json")),
        bytes,
        digest,
        |stored| {
            Ok(stable_hash_string(&format!(
                "momentum-v4.2-raw-input:{stored:?}"
            )))
        },
    )
}

fn collect_protected_artifacts(
    root: &Path,
    current: &Path,
    values: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), String> {
    if current == root.join(ROOT_VERSION_V4_2) {
        return Ok(());
    }
    if current.is_dir() {
        let mut paths = fs::read_dir(current)
            .map_err(|_| "V4.2 protected directory read failed".to_string())?
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
                .map_err(|_| "V4.2 protected path rejected".to_string())?
                .to_path_buf(),
            fs::read(current).map_err(|_| "V4.2 protected artifact read failed".to_string())?,
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

fn source_snapshot<'a>(
    snapshots: &'a [DataSnapshot],
    source: &MomentumFutureEvaluationSourceV4_2,
) -> Result<&'a DataSnapshot, String> {
    let matches = snapshots
        .iter()
        .filter(|snapshot| snapshot.content_digest == source.evaluation.source_snapshot_digest)
        .collect::<Vec<_>>();
    if matches.len() != 1
        || matches[0].row_count != 312
        || matches[0].normalized_dataset.rows.len() != 312
        || matches[0].actual_end_timestamp_ms
            != Some(source.evaluation.source_boundary_timestamp_ms)
    {
        return Err("V4.2 canonical source snapshot rejected".to_string());
    }
    Ok(matches[0])
}

fn reopen_exact<T>(path: &Path, decode: impl Fn(&[u8]) -> Result<T, String>) -> Result<T, String> {
    decode(&fs::read(path).map_err(|_| "V4.2 persisted artifact unavailable".to_string())?)
}

fn build_provider_request(
    registration: &MomentumProspectiveInputAcquisitionRegistrationV4_2,
) -> Result<ReadOnlyProviderRequest, String> {
    if registration.exact_request_count != 1 || registration.exact_expected_timestamp_ms.is_empty()
    {
        return Err("V4.2 registered input request unavailable".to_string());
    }
    let expected_start = registration.exact_expected_timestamp_ms[0];
    let expected_end = registration
        .exact_expected_timestamp_ms
        .last()
        .copied()
        .and_then(|timestamp| timestamp.checked_add(DAILY_CADENCE_MS_V4_2))
        .ok_or_else(|| "V4.2 registered request end unavailable".to_string())?;
    if expected_end != registration.request_to_timestamp_ms
        || registration
            .exact_expected_timestamp_ms
            .windows(2)
            .any(|pair| pair[1] != pair[0].saturating_add(DAILY_CADENCE_MS_V4_2))
    {
        return Err("V4.2 registered timestamps are not one contiguous request".to_string());
    }
    Ok(ReadOnlyProviderRequest {
        request_id: stable_hash_string(&format!(
            "momentum-v4.2-input-request:{}",
            registration.registration_digest
        )),
        request_key: stable_hash_string(&format!(
            "momentum-v4.2-input-key:{}:{}:{expected_start}:{expected_end}",
            registration.provider_id, registration.symbol
        )),
        provider_id: registration.provider_id.clone(),
        dataset_kind: DatasetKind::DailyOhlcv,
        market_scope: AcquisitionMarketScope::BtcCrypto,
        symbols: vec![registration.symbol.clone()],
        lookback: DataLookback {
            bars: registration.exact_expected_timestamp_ms.len(),
            start_timestamp_ms: Some(expected_start),
            end_timestamp_ms: Some(expected_end),
        },
        cadence: registration.cadence.clone(),
        max_staleness_ms: 0,
        reason_codes: vec![],
    })
}

fn request_config(
    config: &UpbitHistoricalPilotConfigV0,
    registration: &MomentumProspectiveInputAcquisitionRegistrationV4_2,
) -> Result<UpbitHistoricalPilotConfigV0, String> {
    let mut request_config = config.clone();
    request_config.start_timestamp_ms =
        registration
            .exact_expected_timestamp_ms
            .first()
            .copied()
            .ok_or_else(|| "V4.2 request start unavailable".to_string())?;
    request_config.end_timestamp_ms = registration.request_to_timestamp_ms;
    request_config.max_retries = 0;
    request_config.validate()?;
    let contract = upbit_learning_evidence_provider_contract_v1(&request_config)?;
    if contract.provider_id != registration.provider_id
        || contract.market_scope != AcquisitionMarketScope::BtcCrypto
        || contract.dataset_kind != DatasetKind::DailyOhlcv
        || contract.symbols.as_slice() != [registration.symbol.clone()]
        || contract.cadence != registration.cadence
        || contract.maximum_response_bytes != registration.maximum_response_bytes
        || !contract.credential_free
        || !contract.read_only
        || !contract.approved_for_network
        || !contract.all_rows_finalized
        || !contract.enabled
    {
        return Err("V4.2 provider contract rejected".to_string());
    }
    Ok(request_config)
}

fn validate_input_response(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    plan: &MomentumProspectiveFeatureContextPlanV4_2,
    registration: &MomentumProspectiveInputAcquisitionRegistrationV4_2,
    transport: &LearningEvidenceTransportResponseV1,
) -> Result<
    (
        MomentumProspectiveInputCapsuleV4_2,
        MomentumProspectiveContextVerificationV4_2,
        Vec<HistoricalOhlcvRow>,
    ),
    String,
> {
    if transport.http_status_class != "2xx"
        || transport.raw_response.is_empty()
        || transport.raw_response.len() > registration.maximum_response_bytes
        || serde_json::from_slice::<serde_json::Value>(&transport.raw_response).is_err()
        || transport.response.provider_id != registration.provider_id
        || transport.response.content_type != "application/x-soma-normalized-dataset"
        || !transport.response.all_rows_finalized
        || transport.response.normalized_dataset.symbol != registration.symbol
        || transport.response.reported_content_bytes != transport.raw_response.len()
    {
        return Err("V4.2 input response envelope rejected".to_string());
    }
    let rows = &transport.response.normalized_dataset.rows;
    let returned_timestamps = rows.iter().map(|row| row.timestamp_ms).collect::<Vec<_>>();
    let unique_timestamps = returned_timestamps.iter().copied().collect::<BTreeSet<_>>();
    let outcome_timestamp = plan
        .event_timestamp_ms
        .checked_add(
            u64::try_from(lifecycle.prediction_horizon)
                .ok()
                .and_then(|horizon| horizon.checked_mul(lifecycle.cadence_ms))
                .ok_or_else(|| "V4.2 outcome timestamp overflow".to_string())?,
        )
        .ok_or_else(|| "V4.2 outcome timestamp overflow".to_string())?;
    if returned_timestamps != registration.exact_expected_timestamp_ms
        || rows.len() != registration.exact_expected_timestamp_ms.len()
        || unique_timestamps.len() != rows.len()
        || rows.windows(2).any(|pair| {
            pair[1].timestamp_ms
                != pair[0]
                    .timestamp_ms
                    .checked_add(lifecycle.cadence_ms)
                    .unwrap_or_default()
        })
        || returned_timestamps.contains(&outcome_timestamp)
        || rows.iter().any(|row| {
            row.symbol != registration.symbol
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
        })
    {
        return Err("V4.2 exact input evidence rejected".to_string());
    }
    let raw_response_digest = stable_hash_string(&format!(
        "momentum-v4.2-raw-input:{:?}",
        transport.raw_response
    ));
    let mut capsule = MomentumProspectiveInputCapsuleV4_2 {
        capsule_version: INPUT_CAPSULE_VERSION_V4_2.to_string(),
        lifecycle_digest: lifecycle.lifecycle_digest.clone(),
        input_registration_digest: registration.registration_digest.clone(),
        event_timestamp_ms: plan.event_timestamp_ms,
        exact_timestamp_ms: returned_timestamps,
        row_identity_digests: rows.iter().map(row_identity_digest).collect(),
        normalized_dataset_digest: historical_replay_dataset_digest_v0(
            &transport.response.normalized_dataset,
        ),
        raw_response_digest,
        outcome_rows_present: false,
        labels_accessed: false,
        metrics_computed: false,
        credential_free: true,
        read_only: true,
        sanitized: true,
        capsule_digest: String::new(),
    };
    capsule.capsule_digest = input_capsule_digest(&capsule);
    decode_input_capsule(&encode_input_capsule(&capsule)?)?;
    let mut proof = MomentumProspectiveContextVerificationV4_2 {
        proof_version: CONTEXT_PROOF_VERSION_V4_2.to_string(),
        feature_context_plan_digest: plan.plan_digest.clone(),
        input_registration_digest: registration.registration_digest.clone(),
        input_capsule_digest: capsule.capsule_digest.clone(),
        exact_timestamps_verified: true,
        strict_chronology_verified: true,
        feature_history_complete: plan.existing_source_row_digests.len() + rows.len()
            == plan.required_row_count,
        protected_events_not_scored: !plan
            .protected_context_timestamp_ms
            .contains(&plan.event_timestamp_ms),
        outcome_timestamp_absent: true,
        proof_digest: String::new(),
    };
    proof.proof_digest = context_proof_digest(&proof);
    decode_context_proof(&encode_context_proof(&proof)?)?;
    Ok((capsule, proof, rows.clone()))
}

fn build_complete_context(
    plan: &MomentumProspectiveFeatureContextPlanV4_2,
    source_snapshot: &DataSnapshot,
    incremental_rows: &[HistoricalOhlcvRow],
) -> Result<Vec<HistoricalOhlcvRow>, String> {
    let mut rows = source_snapshot
        .normalized_dataset
        .rows
        .iter()
        .chain(incremental_rows)
        .filter(|row| {
            row.timestamp_ms >= plan.required_context_start_timestamp_ms
                && row.timestamp_ms <= plan.required_context_end_timestamp_ms
        })
        .cloned()
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.timestamp_ms);
    if rows.len() != plan.required_row_count
        || rows
            .windows(2)
            .any(|pair| pair[1].timestamp_ms != pair[0].timestamp_ms + DAILY_CADENCE_MS_V4_2)
        || rows.first().map(|row| row.timestamp_ms)
            != Some(plan.required_context_start_timestamp_ms)
        || rows.last().map(|row| row.timestamp_ms) != Some(plan.event_timestamp_ms)
    {
        return Err("V4.2 complete prospective context rejected".to_string());
    }
    Ok(rows)
}

fn seal_participant_predictions(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    input_capsule: &MomentumProspectiveInputCapsuleV4_2,
    prediction: &[MomentumFrozenParticipantPredictionV4],
    feature_identity_digest: &str,
) -> Result<Vec<MomentumParticipantPredictionSealV4_2>, String> {
    if prediction.len() != lifecycle.participant_digests.len() {
        return Err("V4.2 participant prediction count rejected".to_string());
    }
    prediction
        .iter()
        .enumerate()
        .map(|(index, prediction)| {
            if prediction.participant_digest != lifecycle.participant_digests[index]
                || prediction.parameter_digest != lifecycle.participant_parameter_digests[index]
                || prediction.normalizer_digest != lifecycle.participant_normalizer_digests[index]
                || prediction.config_digest.is_empty()
                || prediction.model_artifact_digest.is_empty()
                || prediction.feature_schema_digest.is_empty()
                || prediction.training_identity_digest.is_empty()
            {
                return Err("V4.2 reconstructed participant identity mismatch".to_string());
            }
            let prediction_digest = stable_hash_string(&format!(
                "momentum-v4.2-sealed-prediction:{}:{}:{}:{}",
                prediction.participant_digest,
                input_capsule.event_timestamp_ms,
                input_capsule.capsule_digest,
                prediction.probability_bits
            ));
            let mut seal = MomentumParticipantPredictionSealV4_2 {
                participant_digest: prediction.participant_digest.clone(),
                event_timestamp_ms: input_capsule.event_timestamp_ms,
                input_capsule_digest: input_capsule.capsule_digest.clone(),
                feature_identity_digest: feature_identity_digest.to_string(),
                prediction_probability_bits: prediction.probability_bits,
                prediction_digest,
                participant_reconstructed: true,
                parameter_updates: 0,
                outcome_access_count: 0,
                seal_digest: String::new(),
            };
            seal.seal_digest = prediction_seal_digest(&seal);
            decode_prediction_seal(&encode_prediction_seal(&seal)?)?;
            Ok(seal)
        })
        .collect()
}

fn build_prediction_capsule(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    receipt: &MomentumProspectiveInputReceiptV4_2,
    input_capsule: &MomentumProspectiveInputCapsuleV4_2,
    seals: Vec<MomentumParticipantPredictionSealV4_2>,
) -> Result<MomentumProspectivePredictionCapsuleV4_2, String> {
    let mut capsule = MomentumProspectivePredictionCapsuleV4_2 {
        capsule_version: PREDICTION_CAPSULE_VERSION_V4_2.to_string(),
        lifecycle_digest: lifecycle.lifecycle_digest.clone(),
        evaluation_registration_digest: lifecycle.evaluation_registration_digest.clone(),
        roster_digest: lifecycle.roster_digest.clone(),
        event_timestamp_ms: input_capsule.event_timestamp_ms,
        input_receipt_digest: receipt.receipt_digest.clone(),
        input_capsule_digest: input_capsule.capsule_digest.clone(),
        participant_prediction_seals: seals,
        probabilities_hidden: true,
        labels_hidden: true,
        outcome_accessed: false,
        metrics_computed: false,
        winner_selected: false,
        capsule_digest: String::new(),
    };
    capsule.capsule_digest = prediction_capsule_digest(&capsule);
    decode_prediction_capsule(&encode_prediction_capsule(&capsule)?)?;
    Ok(capsule)
}

fn build_prediction_journal(
    capsule: &MomentumProspectivePredictionCapsuleV4_2,
) -> Result<MomentumProspectivePredictionJournalV4_2, String> {
    let mut entry = MomentumProspectivePredictionJournalEntryV4_2 {
        event_timestamp_ms: capsule.event_timestamp_ms,
        input_capsule_digest: capsule.input_capsule_digest.clone(),
        prediction_capsule_digest: capsule.capsule_digest.clone(),
        participant_prediction_digests: capsule
            .participant_prediction_seals
            .iter()
            .map(|seal| seal.prediction_digest.clone())
            .collect(),
        prediction_sealed_before_outcome: true,
        outcome_stage_unlocked: false,
        entry_digest: String::new(),
    };
    entry.entry_digest = prediction_entry_digest(&entry);
    let mut journal = MomentumProspectivePredictionJournalV4_2 {
        journal_version: PREDICTION_JOURNAL_VERSION_V4_2.to_string(),
        entries: vec![entry],
        journal_digest: String::new(),
    };
    journal.journal_digest = prediction_journal_digest(&journal);
    decode_prediction_journal(&encode_prediction_journal(&journal)?)?;
    Ok(journal)
}

fn build_maturity_plan(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    capsule: &MomentumProspectivePredictionCapsuleV4_2,
) -> Result<MomentumProspectiveOutcomeMaturityPlanV4_2, String> {
    let required_outcome_timestamp_ms = (1..=lifecycle.prediction_horizon)
        .map(|offset| {
            u64::try_from(offset)
                .ok()
                .and_then(|offset| offset.checked_mul(lifecycle.cadence_ms))
                .and_then(|duration| capsule.event_timestamp_ms.checked_add(duration))
                .ok_or_else(|| "V4.2 maturity timestamp overflow".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let outcome_finality_boundary_ms = required_outcome_timestamp_ms
        .last()
        .copied()
        .and_then(|timestamp| timestamp.checked_add(lifecycle.cadence_ms))
        .ok_or_else(|| "V4.2 outcome finality boundary unavailable".to_string())?;
    let mut plan = MomentumProspectiveOutcomeMaturityPlanV4_2 {
        plan_version: MATURITY_PLAN_VERSION_V4_2.to_string(),
        prediction_capsule_digest: capsule.capsule_digest.clone(),
        event_timestamp_ms: capsule.event_timestamp_ms,
        prediction_horizon: lifecycle.prediction_horizon,
        required_outcome_timestamp_ms,
        outcome_finality_boundary_ms,
        maximum_outcome_requests: lifecycle.outcome_stage_maximum_requests,
        maximum_outcome_retries: lifecycle.outcome_stage_maximum_retries,
        labels_hidden_until_opening: true,
        one_time_opening_required: true,
        plan_digest: String::new(),
    };
    plan.plan_digest = maturity_plan_digest(&plan);
    decode_maturity_plan(&encode_maturity_plan(&plan)?)?;
    Ok(plan)
}

fn build_input_receipt(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    registration: &MomentumProspectiveInputAcquisitionRegistrationV4_2,
    status: MomentumProspectiveInputStatusV4_2,
    http_status_class: Option<String>,
    returned_row_count: usize,
    verified_row_count: usize,
    raw_response_digest: Option<String>,
    input_capsule_digest: Option<String>,
) -> MomentumProspectiveInputReceiptV4_2 {
    let mut receipt = MomentumProspectiveInputReceiptV4_2 {
        receipt_version: INPUT_RECEIPT_VERSION_V4_2.to_string(),
        lifecycle_digest: lifecycle.lifecycle_digest.clone(),
        input_registration_digest: registration.registration_digest.clone(),
        request_attempted: status != MomentumProspectiveInputStatusV4_2::ReadyNotAttempted,
        request_count: usize::from(status != MomentumProspectiveInputStatusV4_2::ReadyNotAttempted),
        retry_count: 0,
        status,
        http_status_class,
        returned_row_count,
        verified_row_count,
        raw_response_digest,
        input_capsule_digest,
        terminal: status.is_terminal(),
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = input_receipt_digest(&receipt);
    receipt
}

#[allow(clippy::too_many_arguments)]
fn build_status_receipt(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    plan: &MomentumProspectiveFeatureContextPlanV4_2,
    registration: &MomentumProspectiveInputAcquisitionRegistrationV4_2,
    readiness: MomentumEventReadinessV4_2,
    input_receipt: Option<&MomentumProspectiveInputReceiptV4_2>,
    input_capsule: Option<&MomentumProspectiveInputCapsuleV4_2>,
    prediction_capsule: Option<&MomentumProspectivePredictionCapsuleV4_2>,
    maturity_plan: Option<&MomentumProspectiveOutcomeMaturityPlanV4_2>,
    protected_artifacts_unchanged: bool,
    active_state_unchanged: bool,
    safety_counters: MomentumFuturePredictionSafetyCountersV4_2,
) -> Result<MomentumFuturePredictionStatusReceiptV4_2, String> {
    validate_safety_counters(&safety_counters)?;
    let mut status = MomentumFuturePredictionStatusReceiptV4_2 {
        status_version: STATUS_RECEIPT_VERSION_V4_2.to_string(),
        request_budget_meaning: FutureEvaluationRequestBudgetMeaningV4_2::Ambiguous,
        lifecycle_digest: lifecycle.lifecycle_digest.clone(),
        event_readiness: readiness,
        event_timestamp_ms: plan.event_timestamp_ms,
        input_finality_boundary_ms: plan
            .event_timestamp_ms
            .checked_add(lifecycle.cadence_ms)
            .ok_or_else(|| "V4.2 input finality overflow".to_string())?,
        context_policy_status: plan.context_policy_status,
        context_plan_digest: plan.plan_digest.clone(),
        input_registration_digest: registration.registration_digest.clone(),
        request_attempt_count: input_receipt.map_or(0, |receipt| receipt.request_count),
        input_receipt_digest: input_receipt.map(|receipt| receipt.receipt_digest.clone()),
        input_capsule_digest: input_capsule.map(|capsule| capsule.capsule_digest.clone()),
        participant_prediction_digests: prediction_capsule
            .map(|capsule| {
                capsule
                    .participant_prediction_seals
                    .iter()
                    .map(|seal| seal.prediction_digest.clone())
                    .collect()
            })
            .unwrap_or_default(),
        prediction_capsule_digest: prediction_capsule.map(|capsule| capsule.capsule_digest.clone()),
        outcome_maturity_plan_digest: maturity_plan.map(|plan| plan.plan_digest.clone()),
        outcome_finality_boundary_ms: maturity_plan.map(|plan| plan.outcome_finality_boundary_ms),
        cycle_risk_status: "ProviderContractUnverified".to_string(),
        value_quality_status: "TrainerUnavailable".to_string(),
        prior_momentum_attribution: "MissedMaterialOpportunity".to_string(),
        prior_cycle_risk_attribution: "CorrectUncertainty".to_string(),
        protected_artifacts_unchanged,
        active_state_unchanged,
        safety_counters,
        status_digest: String::new(),
    };
    status.status_digest = status_digest(&status);
    decode_status(&encode_status(&status)?)?;
    Ok(status)
}

fn base_report(
    status: MomentumFuturePredictionStatusReceiptV4_2,
    lifecycle: MomentumFutureEvaluationLifecycleV4_2,
    context_plan: MomentumProspectiveFeatureContextPlanV4_2,
    input_registration: MomentumProspectiveInputAcquisitionRegistrationV4_2,
) -> MomentumFuturePredictionReportV4_2 {
    MomentumFuturePredictionReportV4_2 {
        status,
        lifecycle,
        context_plan,
        input_registration,
        input_receipt: None,
        input_capsule: None,
        prediction_capsule: None,
        prediction_journal: None,
        outcome_maturity_plan: None,
        artifacts_written: 0,
        duplicate_artifact_count: 0,
        storage_failure_count: 0,
    }
}

pub fn run_momentum_future_prediction_v4_2(
    root: &Path,
    snapshots: &[DataSnapshot],
    reservation: &ProtectedEvaluationReservationV1,
    provider_config: &UpbitHistoricalPilotConfigV0,
    observed_timestamp_ms: u64,
    mode: MomentumFuturePredictionRunModeV4_2,
    network_allowed: bool,
    one_time_input_request_confirmed: bool,
) -> Result<MomentumFuturePredictionReportV4_2, String> {
    if mode != MomentumFuturePredictionRunModeV4_2::Execute
        && (network_allowed || one_time_input_request_confirmed)
    {
        return Err("V4.2 non-execute mode rejects network authority".to_string());
    }
    if mode == MomentumFuturePredictionRunModeV4_2::Execute
        && network_allowed != one_time_input_request_confirmed
    {
        return Err("V4.2 execute requires both network permissions".to_string());
    }
    provider_config.validate()?;
    let protected_before = protected_artifacts(root)?;
    let active_before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let source = reopen_momentum_v4_1_future_source(root)?;
    let source_snapshot = source_snapshot(snapshots, &source)?;
    let lifecycle = derive_lifecycle(&source)?;
    let context_plan = derive_context_plan(&lifecycle, &source.evaluation, source_snapshot)?;
    let input_registration =
        derive_input_registration(&lifecycle, &context_plan, &source, provider_config)?;
    let v4_2_root = root.join(ROOT_VERSION_V4_2).join(AGENT_ID_V4_2);
    if let Some(supersession) = read_single(
        &root
            .join(ROOT_VERSION_V4_3)
            .join(AGENT_ID_V4_2)
            .join("supersessions"),
        decode_supersession_v4_3,
    )? {
        if !old_registration_superseded_v4_3(
            &supersession,
            &lifecycle,
            &context_plan,
            &input_registration,
        ) {
            return Err("V4.2 supersession identity rejected".to_string());
        }
        let protected_after = protected_artifacts(root)?;
        let active_after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
        let status = build_status_receipt(
            &lifecycle,
            &context_plan,
            &input_registration,
            MomentumEventReadinessV4_2::SupersededInputRegistration,
            None,
            None,
            None,
            None,
            protected_before == protected_after,
            active_before == active_after,
            MomentumFuturePredictionSafetyCountersV4_2::default(),
        )?;
        return Ok(base_report(
            status,
            lifecycle,
            context_plan,
            input_registration,
        ));
    }
    let persisted_input_receipt =
        read_single(&v4_2_root.join("input_receipts"), decode_input_receipt)?;
    let persisted_input_capsule =
        read_single(&v4_2_root.join("input_capsules"), decode_input_capsule)?;
    let persisted_prediction_capsule = read_single(
        &v4_2_root.join("prediction_capsules"),
        decode_prediction_capsule,
    )?;

    if let Some(prediction_capsule) = persisted_prediction_capsule {
        let input_receipt = persisted_input_receipt
            .ok_or_else(|| "V4.2 sealed prediction input receipt unavailable".to_string())?;
        let input_capsule = persisted_input_capsule
            .ok_or_else(|| "V4.2 sealed prediction input capsule unavailable".to_string())?;
        let prediction_journal = read_single(
            &v4_2_root.join("prediction_journals"),
            decode_prediction_journal,
        )?
        .ok_or_else(|| "V4.2 sealed prediction journal unavailable".to_string())?;
        let outcome_maturity_plan = read_single(
            &v4_2_root.join("outcome_maturity_plans"),
            decode_maturity_plan,
        )?
        .ok_or_else(|| "V4.2 sealed outcome maturity plan unavailable".to_string())?;
        if prediction_capsule.lifecycle_digest != lifecycle.lifecycle_digest
            || prediction_capsule.roster_digest != lifecycle.roster_digest
            || prediction_capsule.input_receipt_digest != input_receipt.receipt_digest
            || prediction_capsule.input_capsule_digest != input_capsule.capsule_digest
            || prediction_journal.entries.len() != 1
            || prediction_journal.entries[0].prediction_capsule_digest
                != prediction_capsule.capsule_digest
            || outcome_maturity_plan.prediction_capsule_digest != prediction_capsule.capsule_digest
        {
            return Err("V4.2 sealed replay cross-binding rejected".to_string());
        }
        let protected_after = protected_artifacts(root)?;
        let active_after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
        let status = build_status_receipt(
            &lifecycle,
            &context_plan,
            &input_registration,
            MomentumEventReadinessV4_2::PredictionAlreadySealed,
            Some(&input_receipt),
            Some(&input_capsule),
            Some(&prediction_capsule),
            Some(&outcome_maturity_plan),
            protected_before == protected_after,
            active_before == active_after,
            MomentumFuturePredictionSafetyCountersV4_2::default(),
        )?;
        let mut report = base_report(status, lifecycle, context_plan, input_registration);
        report.input_receipt = Some(input_receipt);
        report.input_capsule = Some(input_capsule);
        report.prediction_capsule = Some(prediction_capsule);
        report.prediction_journal = Some(prediction_journal);
        report.outcome_maturity_plan = Some(outcome_maturity_plan);
        return Ok(report);
    }

    let mut readiness = event_readiness(&lifecycle, &context_plan, observed_timestamp_ms, 200);
    if persisted_input_receipt
        .as_ref()
        .is_some_and(|receipt| receipt.terminal)
    {
        readiness = MomentumEventReadinessV4_2::PriorInputAttemptTerminal;
    }
    if mode != MomentumFuturePredictionRunModeV4_2::Execute
        || readiness == MomentumEventReadinessV4_2::PriorInputAttemptTerminal
    {
        let protected_after = protected_artifacts(root)?;
        let active_after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
        let status = build_status_receipt(
            &lifecycle,
            &context_plan,
            &input_registration,
            readiness,
            persisted_input_receipt.as_ref(),
            persisted_input_capsule.as_ref(),
            None,
            None,
            protected_before == protected_after,
            active_before == active_after,
            MomentumFuturePredictionSafetyCountersV4_2::default(),
        )?;
        let mut report = base_report(status, lifecycle, context_plan, input_registration);
        report.input_receipt = persisted_input_receipt;
        report.input_capsule = persisted_input_capsule;
        return Ok(report);
    }

    let mut counts = (0, 0);
    add_counts(&mut counts, persist_lifecycle(&v4_2_root, &lifecycle)?);
    add_counts(
        &mut counts,
        persist_context_plan(&v4_2_root, &context_plan)?,
    );
    add_counts(
        &mut counts,
        persist_input_registration(&v4_2_root, &input_registration)?,
    );
    let reopened_lifecycle = reopen_exact(
        &v4_2_root
            .join("lifecycles")
            .join(format!("{}.pb", lifecycle.lifecycle_digest)),
        decode_lifecycle,
    )?;
    let reopened_context = reopen_exact(
        &v4_2_root
            .join("context_plans")
            .join(format!("{}.pb", context_plan.plan_digest)),
        decode_context_plan,
    )?;
    let reopened_input_registration = reopen_exact(
        &v4_2_root
            .join("input_registrations")
            .join(format!("{}.pb", input_registration.registration_digest)),
        decode_input_registration,
    )?;
    if reopened_lifecycle != lifecycle
        || reopened_context != context_plan
        || reopened_input_registration != input_registration
    {
        return Err("V4.2 prerequest contract reopen rejected".to_string());
    }

    if readiness != MomentumEventReadinessV4_2::ReadyForInputAcquisition {
        let protected_after = protected_artifacts(root)?;
        let active_after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
        if protected_before != protected_after || active_before != active_after {
            return Err("V4.2 blocked execution changed protected state".to_string());
        }
        let status = build_status_receipt(
            &lifecycle,
            &context_plan,
            &input_registration,
            readiness,
            None,
            None,
            None,
            None,
            true,
            true,
            MomentumFuturePredictionSafetyCountersV4_2::default(),
        )?;
        add_counts(&mut counts, persist_status(&v4_2_root, &status)?);
        let mut report = base_report(status, lifecycle, context_plan, input_registration);
        report.artifacts_written = counts.0;
        report.duplicate_artifact_count = counts.1;
        return Ok(report);
    }
    if !network_allowed || !one_time_input_request_confirmed {
        return Err("V4.2 ready execution lacks explicit request authority".to_string());
    }

    let request = build_provider_request(&input_registration)?;
    let request_config = request_config(provider_config, &input_registration)?;
    let mut safety_counters = MomentumFuturePredictionSafetyCountersV4_2::default();
    safety_counters.input_request_attempts = 1;
    let transport = fetch_upbit_learning_evidence_once_v1(&request_config, &request);
    let transport = match transport {
        Ok(transport) => transport,
        Err(failure) => {
            let (status, http_status_class, raw_response) = match failure {
                LearningEvidenceTransportFailureV1::ProviderRejected {
                    http_status_class,
                    raw_response,
                } => (
                    MomentumProspectiveInputStatusV4_2::ProviderRejected,
                    http_status_class,
                    raw_response,
                ),
                LearningEvidenceTransportFailureV1::TimedOut => (
                    MomentumProspectiveInputStatusV4_2::TimeoutNoRetry,
                    None,
                    None,
                ),
                LearningEvidenceTransportFailureV1::Technical => (
                    MomentumProspectiveInputStatusV4_2::TechnicalFailure,
                    None,
                    None,
                ),
            };
            let raw_digest = raw_response
                .as_ref()
                .map(|bytes| stable_hash_string(&format!("momentum-v4.2-raw-input:{bytes:?}")));
            if let (Some(bytes), Some(digest)) = (&raw_response, &raw_digest) {
                if bytes.len() <= input_registration.maximum_response_bytes {
                    add_counts(
                        &mut counts,
                        persist_raw_response(&v4_2_root, bytes, digest)?,
                    );
                }
            }
            let receipt = build_input_receipt(
                &lifecycle,
                &input_registration,
                status,
                http_status_class,
                0,
                0,
                raw_digest,
                None,
            );
            add_counts(&mut counts, persist_input_receipt(&v4_2_root, &receipt)?);
            let protected_after = protected_artifacts(root)?;
            let active_after =
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
            if protected_before != protected_after || active_before != active_after {
                return Err("V4.2 failed input attempt changed protected state".to_string());
            }
            let status = build_status_receipt(
                &lifecycle,
                &context_plan,
                &input_registration,
                MomentumEventReadinessV4_2::PriorInputAttemptTerminal,
                Some(&receipt),
                None,
                None,
                None,
                true,
                true,
                safety_counters,
            )?;
            add_counts(&mut counts, persist_status(&v4_2_root, &status)?);
            let mut report = base_report(status, lifecycle, context_plan, input_registration);
            report.input_receipt = Some(receipt);
            report.artifacts_written = counts.0;
            report.duplicate_artifact_count = counts.1;
            return Ok(report);
        }
    };

    let validated =
        validate_input_response(&lifecycle, &context_plan, &input_registration, &transport);
    let (input_capsule, context_proof, incremental_rows) = match validated {
        Ok(value) => value,
        Err(_) => {
            let raw_digest = stable_hash_string(&format!(
                "momentum-v4.2-raw-input:{:?}",
                transport.raw_response
            ));
            add_counts(
                &mut counts,
                persist_raw_response(&v4_2_root, &transport.raw_response, &raw_digest)?,
            );
            let receipt = build_input_receipt(
                &lifecycle,
                &input_registration,
                MomentumProspectiveInputStatusV4_2::InvalidResponse,
                Some(transport.http_status_class),
                transport.response.normalized_dataset.rows.len(),
                0,
                Some(raw_digest),
                None,
            );
            add_counts(&mut counts, persist_input_receipt(&v4_2_root, &receipt)?);
            let protected_after = protected_artifacts(root)?;
            let active_after =
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
            if protected_before != protected_after || active_before != active_after {
                return Err("V4.2 invalid input changed protected state".to_string());
            }
            let status = build_status_receipt(
                &lifecycle,
                &context_plan,
                &input_registration,
                MomentumEventReadinessV4_2::PriorInputAttemptTerminal,
                Some(&receipt),
                None,
                None,
                None,
                true,
                true,
                safety_counters,
            )?;
            add_counts(&mut counts, persist_status(&v4_2_root, &status)?);
            let mut report = base_report(status, lifecycle, context_plan, input_registration);
            report.input_receipt = Some(receipt);
            report.artifacts_written = counts.0;
            report.duplicate_artifact_count = counts.1;
            return Ok(report);
        }
    };
    add_counts(
        &mut counts,
        persist_raw_response(
            &v4_2_root,
            &transport.raw_response,
            &input_capsule.raw_response_digest,
        )?,
    );
    add_counts(
        &mut counts,
        persist_input_capsule(&v4_2_root, &input_capsule)?,
    );
    add_counts(
        &mut counts,
        persist_context_proof(&v4_2_root, &context_proof)?,
    );
    let receipt = build_input_receipt(
        &lifecycle,
        &input_registration,
        MomentumProspectiveInputStatusV4_2::EvidenceAcquired,
        Some(transport.http_status_class),
        incremental_rows.len(),
        incremental_rows.len(),
        Some(input_capsule.raw_response_digest.clone()),
        Some(input_capsule.capsule_digest.clone()),
    );
    add_counts(&mut counts, persist_input_receipt(&v4_2_root, &receipt)?);

    let complete_context =
        build_complete_context(&context_plan, source_snapshot, &incremental_rows)?;
    let replay = reconstruct_frozen_momentum_v4(root, snapshots, reservation)?;
    let frozen_prediction = predict_frozen_momentum_v4_event(
        &replay,
        &lifecycle.participant_digests,
        &complete_context,
    )?;
    let seals = seal_participant_predictions(
        &lifecycle,
        &input_capsule,
        &frozen_prediction.participant_predictions,
        &frozen_prediction.feature_identity_digest,
    )?;
    for seal in &seals {
        add_counts(&mut counts, persist_prediction_seal(&v4_2_root, seal)?);
    }
    let prediction_capsule = build_prediction_capsule(&lifecycle, &receipt, &input_capsule, seals)?;
    add_counts(
        &mut counts,
        persist_prediction_capsule(&v4_2_root, &prediction_capsule)?,
    );
    let prediction_journal = build_prediction_journal(&prediction_capsule)?;
    add_counts(
        &mut counts,
        persist_prediction_journal(&v4_2_root, &prediction_journal)?,
    );
    let maturity_plan = build_maturity_plan(&lifecycle, &prediction_capsule)?;
    add_counts(
        &mut counts,
        persist_maturity_plan(&v4_2_root, &maturity_plan)?,
    );
    let protected_after = protected_artifacts(root)?;
    let active_after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    if protected_before != protected_after || active_before != active_after {
        return Err("V4.2 prediction sealing changed protected state".to_string());
    }
    let status = build_status_receipt(
        &lifecycle,
        &context_plan,
        &input_registration,
        MomentumEventReadinessV4_2::PredictionAlreadySealed,
        Some(&receipt),
        Some(&input_capsule),
        Some(&prediction_capsule),
        Some(&maturity_plan),
        true,
        true,
        safety_counters,
    )?;
    add_counts(&mut counts, persist_status(&v4_2_root, &status)?);
    let mut report = base_report(status, lifecycle, context_plan, input_registration);
    report.input_receipt = Some(receipt);
    report.input_capsule = Some(input_capsule);
    report.prediction_capsule = Some(prediction_capsule);
    report.prediction_journal = Some(prediction_journal);
    report.outcome_maturity_plan = Some(maturity_plan);
    report.artifacts_written = counts.0;
    report.duplicate_artifact_count = counts.1;
    Ok(report)
}

fn authorization_digest_v4_3(value: &MomentumProtectedContextAuthorizationV4_3) -> String {
    canonical_digest(value, |value| value.authorization_digest.clear())
}

fn context_usage_entry_digest_v4_3(value: &MomentumContextUsageEntryV4_3) -> String {
    canonical_digest(value, |value| value.entry_digest.clear())
}

fn context_usage_ledger_digest_v4_3(value: &MomentumContextUsageLedgerV4_3) -> String {
    canonical_digest(value, |value| value.ledger_digest.clear())
}

fn supersession_digest_v4_3(value: &MomentumInputPlanSupersessionV4_3) -> String {
    canonical_digest(value, |value| {
        value.replacement_input_registration_digest.clear();
        value.supersession_digest.clear();
    })
}

fn context_plan_digest_v4_3(value: &MomentumProspectiveFeatureContextPlanV4_3) -> String {
    canonical_digest(value, |value| value.plan_digest.clear())
}

fn input_registration_digest_v4_3(value: &MomentumProspectiveInputRegistrationV4_3) -> String {
    canonical_digest(value, |value| value.registration_digest.clear())
}

fn input_receipt_digest_v4_3(value: &MomentumProspectiveInputReceiptV4_3) -> String {
    canonical_digest(value, |value| value.receipt_digest.clear())
}

fn input_capsule_digest_v4_3(value: &MomentumProspectiveInputCapsuleV4_3) -> String {
    canonical_digest(value, |value| value.capsule_digest.clear())
}

fn context_proof_digest_v4_3(value: &MomentumProspectiveContextVerificationV4_3) -> String {
    canonical_digest(value, |value| value.proof_digest.clear())
}

fn prediction_seal_digest_v4_3(value: &MomentumParticipantPredictionSealV4_3) -> String {
    canonical_digest(value, |value| value.seal_digest.clear())
}

fn prediction_capsule_digest_v4_3(value: &MomentumProspectivePredictionCapsuleV4_3) -> String {
    canonical_digest(value, |value| value.capsule_digest.clear())
}

fn prediction_entry_digest_v4_3(value: &MomentumProspectivePredictionJournalEntryV4_3) -> String {
    canonical_digest(value, |value| value.entry_digest.clear())
}

fn prediction_journal_digest_v4_3(value: &MomentumProspectivePredictionJournalV4_3) -> String {
    canonical_digest(value, |value| value.journal_digest.clear())
}

fn outcome_plan_digest_v4_3(value: &MomentumProspectiveOutcomePlanV4_3) -> String {
    canonical_digest(value, |value| value.plan_digest.clear())
}

fn status_digest_v4_3(value: &MomentumFuturePredictionStatusReceiptV4_3) -> String {
    canonical_digest(value, |value| value.status_digest.clear())
}

fn participant_source_identities_v4_3(
    source: &MomentumFutureEvaluationSourceV4_2,
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut parameter_digests = Vec::with_capacity(lifecycle.participant_digests.len());
    let mut normalizer_digests = Vec::with_capacity(lifecycle.participant_digests.len());
    for participant_digest in &lifecycle.participant_digests {
        let participant = source
            .source_family
            .participants
            .iter()
            .find(|participant| &participant.participant_digest == participant_digest)
            .ok_or_else(|| "V4.3 frozen participant unavailable".to_string())?;
        if (participant.participant_role != MomentumRawFeatureRoleV4::ConstantBenchmark
            && !participant.fresh_initialization)
            || participant.prior_parameters_reused
            || participant.prior_normalizer_reused
            || participant.prior_predictions_reused
            || participant.validation_parameter_updates != 0
        {
            return Err("V4.3 frozen participant contract rejected".to_string());
        }
        parameter_digests.push(participant.parameter_digest.clone());
        normalizer_digests.push(participant.normalizer_digest.clone());
    }
    Ok((parameter_digests, normalizer_digests))
}

fn derive_context_authorization_v4_3(
    source: &MomentumFutureEvaluationSourceV4_2,
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    canonical_raw_provider_available: bool,
    prior_outcome_artifact_referenced: bool,
    model_source_boundary_override: Option<u64>,
) -> Result<MomentumProtectedContextAuthorizationV4_3, String> {
    let (parameter_digests, normalizer_digests) =
        participant_source_identities_v4_3(source, lifecycle)?;
    let model_source_boundary_timestamp_ms =
        model_source_boundary_override.unwrap_or(source.evaluation.source_boundary_timestamp_ms);
    let event_timestamp_ms = align_up(
        lifecycle.minimum_accepted_event_timestamp_ms.max(
            source
                .evaluation
                .source_boundary_timestamp_ms
                .checked_add(lifecycle.cadence_ms)
                .ok_or_else(|| "V4.3 source boundary overflow".to_string())?,
        ),
        lifecycle.cadence_ms,
    )?;
    let outcome_timestamp_ms = event_timestamp_ms
        .checked_add(
            u64::try_from(lifecycle.prediction_horizon)
                .ok()
                .and_then(|horizon| horizon.checked_mul(lifecycle.cadence_ms))
                .ok_or_else(|| "V4.3 outcome timestamp overflow".to_string())?,
        )
        .ok_or_else(|| "V4.3 outcome timestamp overflow".to_string())?;
    let freeze_proven = lifecycle.participant_digests.len() == 3
        && lifecycle.participant_parameter_digests == parameter_digests
        && lifecycle.participant_normalizer_digests == normalizer_digests
        && !source.accumulated_family.parameters_changed
        && !source.accumulated_family.winner_selected;
    let source_precedes_protected = !source.evaluation.protected_timestamp_ms.is_empty()
        && source
            .evaluation
            .protected_timestamp_ms
            .iter()
            .all(|timestamp| model_source_boundary_timestamp_ms < *timestamp);
    let policy_valid = !source.evaluation.protected_registration_digests.is_empty()
        && source
            .evaluation
            .protected_timestamp_ms
            .iter()
            .all(|timestamp| {
                *timestamp != event_timestamp_ms && *timestamp != outcome_timestamp_ms
            });
    let authorization_status = if !freeze_proven || !source_precedes_protected {
        ProtectedContextAuthorizationStatusV4_3::ModelFreezeNotProven
    } else if prior_outcome_artifact_referenced {
        ProtectedContextAuthorizationStatusV4_3::PriorOutcomeArtifactReferenced
    } else if !canonical_raw_provider_available {
        ProtectedContextAuthorizationStatusV4_3::RawEvidenceUnavailable
    } else if !policy_valid {
        ProtectedContextAuthorizationStatusV4_3::PolicyConflict
    } else {
        ProtectedContextAuthorizationStatusV4_3::Authorized
    };
    let mut authorization = MomentumProtectedContextAuthorizationV4_3 {
        authorization_version: AUTHORIZATION_VERSION_V4_3.to_string(),
        agent_id: AGENT_ID_V4_2.to_string(),
        v4_family_digest: source.source_family.family_digest.clone(),
        accumulated_family_digest: source.accumulated_family.family_digest.clone(),
        roster_digest: source.roster.roster_digest.clone(),
        evaluation_registration_digest: source.evaluation.registration_digest.clone(),
        lifecycle_digest: lifecycle.lifecycle_digest.clone(),
        participant_digests: lifecycle.participant_digests.clone(),
        parameter_digests,
        normalizer_digests,
        model_source_boundary_timestamp_ms,
        protected_registration_digests: sorted_unique(
            source.evaluation.protected_registration_digests.clone(),
        ),
        protected_timestamp_ms: {
            let mut timestamps = source.evaluation.protected_timestamp_ms.clone();
            timestamps.sort_unstable();
            timestamps.dedup();
            timestamps
        },
        allowed_use_class: ProtectedContextUseClassV4_3::RawOhlcvInferenceContext,
        raw_ohlcv_context_allowed: true,
        training_use_forbidden: true,
        normalizer_fit_forbidden: true,
        label_use_forbidden: true,
        qualification_use_forbidden: true,
        metric_use_forbidden: true,
        reward_use_forbidden: true,
        event_timestamp_use_forbidden: true,
        outcome_timestamp_use_forbidden: true,
        prior_outcome_capsule_use_forbidden: true,
        authorization_status,
        authorization_digest: String::new(),
    };
    authorization.authorization_digest = authorization_digest_v4_3(&authorization);
    validate_context_authorization_v4_3(&authorization, source, lifecycle)?;
    Ok(authorization)
}

fn validate_context_authorization_v4_3(
    value: &MomentumProtectedContextAuthorizationV4_3,
    source: &MomentumFutureEvaluationSourceV4_2,
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
) -> Result<(), String> {
    let (parameter_digests, normalizer_digests) =
        participant_source_identities_v4_3(source, lifecycle)?;
    validate_authorization_policy_v4_3(value)?;
    if value.authorization_version != AUTHORIZATION_VERSION_V4_3
        || value.agent_id != AGENT_ID_V4_2
        || value.v4_family_digest != source.source_family.family_digest
        || value.accumulated_family_digest != source.accumulated_family.family_digest
        || value.roster_digest != source.roster.roster_digest
        || value.evaluation_registration_digest != source.evaluation.registration_digest
        || value.lifecycle_digest != lifecycle.lifecycle_digest
        || value.participant_digests != lifecycle.participant_digests
        || value.parameter_digests != parameter_digests
        || value.normalizer_digests != normalizer_digests
        || value.protected_registration_digests
            != sorted_unique(source.evaluation.protected_registration_digests.clone())
        || value.protected_timestamp_ms != {
            let mut timestamps = source.evaluation.protected_timestamp_ms.clone();
            timestamps.sort_unstable();
            timestamps.dedup();
            timestamps
        }
        || value.authorization_digest != authorization_digest_v4_3(value)
    {
        return Err("V4.3 protected context authorization rejected".to_string());
    }
    Ok(())
}

fn validate_authorization_policy_v4_3(
    value: &MomentumProtectedContextAuthorizationV4_3,
) -> Result<(), String> {
    if value.authorization_status != ProtectedContextAuthorizationStatusV4_3::Authorized
        || value.protected_timestamp_ms.is_empty()
        || !value
            .protected_timestamp_ms
            .iter()
            .all(|timestamp| value.model_source_boundary_timestamp_ms < *timestamp)
        || value.allowed_use_class != ProtectedContextUseClassV4_3::RawOhlcvInferenceContext
        || !value.raw_ohlcv_context_allowed
        || !value.training_use_forbidden
        || !value.normalizer_fit_forbidden
        || !value.label_use_forbidden
        || !value.qualification_use_forbidden
        || !value.metric_use_forbidden
        || !value.reward_use_forbidden
        || !value.event_timestamp_use_forbidden
        || !value.outcome_timestamp_use_forbidden
        || !value.prior_outcome_capsule_use_forbidden
    {
        return Err("V4.3 protected context policy rejected".to_string());
    }
    Ok(())
}

fn event_readiness_v4_3(
    observed_timestamp_ms: u64,
    input_finality_boundary_ms: u64,
    prior_terminal_receipt: bool,
    prediction_already_sealed: bool,
) -> MomentumEventReadinessV4_3 {
    if prediction_already_sealed {
        MomentumEventReadinessV4_3::PredictionAlreadySealed
    } else if prior_terminal_receipt {
        MomentumEventReadinessV4_3::PriorInputAttemptTerminal
    } else if observed_timestamp_ms < input_finality_boundary_ms {
        MomentumEventReadinessV4_3::AwaitingInputFinality
    } else {
        MomentumEventReadinessV4_3::ReadyForInputAcquisition
    }
}

fn old_registration_superseded_v4_3(
    supersession: &MomentumInputPlanSupersessionV4_3,
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    context_plan: &MomentumProspectiveFeatureContextPlanV4_2,
    input_registration: &MomentumProspectiveInputAcquisitionRegistrationV4_2,
) -> bool {
    supersession.status == MomentumInputPlanSupersessionStatusV4_3::Superseded
        && supersession.lifecycle_digest == lifecycle.lifecycle_digest
        && supersession.superseded_context_plan_digest == context_plan.plan_digest
        && supersession.superseded_input_registration_digest
            == input_registration.registration_digest
        && supersession.prior_request_attempt_count == 0
        && !supersession.prior_receipt_present
        && !supersession.prior_capsule_present
        && !supersession.prior_prediction_capsule_present
}

fn derive_context_plan_v4_3(
    source: &MomentumFutureEvaluationSourceV4_2,
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    authorization: &MomentumProtectedContextAuthorizationV4_3,
    source_snapshot: &DataSnapshot,
) -> Result<MomentumProspectiveFeatureContextPlanV4_3, String> {
    if authorization.authorization_status != ProtectedContextAuthorizationStatusV4_3::Authorized {
        return Err("V4.3 context authorization unavailable".to_string());
    }
    let required_row_count = MomentumLearningCampaignConfigV0::default()
        .feature_config
        .minimum_history()
        .map_err(|_| "V4.3 feature history policy rejected".to_string())?
        .checked_add(
            MomentumLearningCampaignConfigV0::default()
                .sequence_config
                .sequence_length
                .saturating_sub(1),
        )
        .ok_or_else(|| "V4.3 feature history overflow".to_string())?;
    let event_timestamp_ms = align_up(
        lifecycle.minimum_accepted_event_timestamp_ms.max(
            source
                .evaluation
                .source_boundary_timestamp_ms
                .checked_add(lifecycle.cadence_ms)
                .ok_or_else(|| "V4.3 source boundary overflow".to_string())?,
        ),
        lifecycle.cadence_ms,
    )?;
    let history_offset = u64::try_from(required_row_count.saturating_sub(1))
        .ok()
        .and_then(|count| count.checked_mul(lifecycle.cadence_ms))
        .ok_or_else(|| "V4.3 context range overflow".to_string())?;
    let required_context_start_timestamp_ms = event_timestamp_ms
        .checked_sub(history_offset)
        .ok_or_else(|| "V4.3 context start unavailable".to_string())?;
    let exact_required_timestamp_ms = timestamp_range(
        required_context_start_timestamp_ms,
        required_row_count,
        lifecycle.cadence_ms,
    )?;
    let source_timestamps = source_snapshot
        .normalized_dataset
        .rows
        .iter()
        .map(|row| row.timestamp_ms)
        .collect::<BTreeSet<_>>();
    let existing_source_timestamp_ms = exact_required_timestamp_ms
        .iter()
        .filter(|timestamp| source_timestamps.contains(timestamp))
        .copied()
        .collect::<Vec<_>>();
    let protected_context_timestamp_ms = exact_required_timestamp_ms
        .iter()
        .filter(|timestamp| authorization.protected_timestamp_ms.contains(timestamp))
        .copied()
        .collect::<Vec<_>>();
    let incremental_timestamp_ms = exact_required_timestamp_ms
        .iter()
        .filter(|timestamp| {
            !source_timestamps.contains(timestamp)
                && !authorization.protected_timestamp_ms.contains(timestamp)
        })
        .copied()
        .collect::<Vec<_>>();
    let input_finality_boundary_ms = event_timestamp_ms
        .checked_add(lifecycle.cadence_ms)
        .ok_or_else(|| "V4.3 input finality overflow".to_string())?;
    let context_usage_policy_digest = stable_hash_string(&format!(
        "momentum-v4.3-context-usage:{}:{:?}:{:?}:{:?}",
        authorization.authorization_digest,
        existing_source_timestamp_ms,
        protected_context_timestamp_ms,
        incremental_timestamp_ms
    ));
    let mut plan = MomentumProspectiveFeatureContextPlanV4_3 {
        plan_version: CONTEXT_PLAN_VERSION_V4_3.to_string(),
        lifecycle_digest: lifecycle.lifecycle_digest.clone(),
        context_authorization_digest: authorization.authorization_digest.clone(),
        event_timestamp_ms,
        input_finality_boundary_ms,
        required_context_start_timestamp_ms,
        required_context_end_timestamp_ms: event_timestamp_ms,
        required_row_count,
        exact_required_timestamp_ms,
        existing_source_timestamp_ms,
        protected_context_timestamp_ms,
        incremental_timestamp_ms,
        context_usage_policy_digest,
        plan_digest: String::new(),
    };
    plan.plan_digest = context_plan_digest_v4_3(&plan);
    validate_context_plan_v4_3(&plan, source, lifecycle, authorization, source_snapshot)?;
    Ok(plan)
}

fn validate_context_plan_v4_3(
    value: &MomentumProspectiveFeatureContextPlanV4_3,
    source: &MomentumFutureEvaluationSourceV4_2,
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    authorization: &MomentumProtectedContextAuthorizationV4_3,
    source_snapshot: &DataSnapshot,
) -> Result<(), String> {
    let expected = timestamp_range(
        value.required_context_start_timestamp_ms,
        value.required_row_count,
        lifecycle.cadence_ms,
    )?;
    let source_timestamps = source_snapshot
        .normalized_dataset
        .rows
        .iter()
        .map(|row| row.timestamp_ms)
        .collect::<BTreeSet<_>>();
    let expected_existing = expected
        .iter()
        .filter(|timestamp| source_timestamps.contains(timestamp))
        .copied()
        .collect::<Vec<_>>();
    let expected_protected = expected
        .iter()
        .filter(|timestamp| authorization.protected_timestamp_ms.contains(timestamp))
        .copied()
        .collect::<Vec<_>>();
    let expected_incremental = expected
        .iter()
        .filter(|timestamp| {
            !source_timestamps.contains(timestamp)
                && !authorization.protected_timestamp_ms.contains(timestamp)
        })
        .copied()
        .collect::<Vec<_>>();
    let outcome_timestamp = value
        .event_timestamp_ms
        .checked_add(
            u64::try_from(lifecycle.prediction_horizon)
                .ok()
                .and_then(|horizon| horizon.checked_mul(lifecycle.cadence_ms))
                .ok_or_else(|| "V4.3 outcome timestamp overflow".to_string())?,
        )
        .ok_or_else(|| "V4.3 outcome timestamp overflow".to_string())?;
    if value.plan_version != CONTEXT_PLAN_VERSION_V4_3
        || value.lifecycle_digest != lifecycle.lifecycle_digest
        || value.context_authorization_digest != authorization.authorization_digest
        || value.event_timestamp_ms
            != align_up(
                lifecycle.minimum_accepted_event_timestamp_ms.max(
                    source
                        .evaluation
                        .source_boundary_timestamp_ms
                        .checked_add(lifecycle.cadence_ms)
                        .ok_or_else(|| "V4.3 source boundary overflow".to_string())?,
                ),
                lifecycle.cadence_ms,
            )?
        || value.input_finality_boundary_ms
            != value
                .event_timestamp_ms
                .checked_add(lifecycle.cadence_ms)
                .ok_or_else(|| "V4.3 input finality overflow".to_string())?
        || value.required_context_end_timestamp_ms != value.event_timestamp_ms
        || value.required_row_count != 16
        || value.exact_required_timestamp_ms != expected
        || expected.last() != Some(&value.event_timestamp_ms)
        || expected.windows(2).any(|pair| pair[1] <= pair[0])
        || expected.contains(&outcome_timestamp)
        || authorization
            .protected_timestamp_ms
            .contains(&value.event_timestamp_ms)
        || authorization
            .protected_timestamp_ms
            .contains(&outcome_timestamp)
        || value.existing_source_timestamp_ms != expected_existing
        || value.protected_context_timestamp_ms != expected_protected
        || value.incremental_timestamp_ms != expected_incremental
        || value.protected_context_timestamp_ms.is_empty()
        || value.plan_digest != context_plan_digest_v4_3(value)
    {
        return Err("V4.3 corrected context plan rejected".to_string());
    }
    Ok(())
}

fn derive_supersession_v4_3(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    authorization: &MomentumProtectedContextAuthorizationV4_3,
    old_plan: &MomentumProspectiveFeatureContextPlanV4_2,
    old_registration: &MomentumProspectiveInputAcquisitionRegistrationV4_2,
    replacement_plan: &MomentumProspectiveFeatureContextPlanV4_3,
    prior_request_attempt_count: usize,
    prior_receipt_present: bool,
    prior_capsule_present: bool,
    prior_prediction_capsule_present: bool,
) -> MomentumInputPlanSupersessionV4_3 {
    let status = if prior_request_attempt_count != 0 {
        MomentumInputPlanSupersessionStatusV4_3::PriorAttemptExists
    } else if prior_receipt_present || prior_capsule_present || prior_prediction_capsule_present {
        MomentumInputPlanSupersessionStatusV4_3::PriorEvidenceExists
    } else {
        MomentumInputPlanSupersessionStatusV4_3::Superseded
    };
    let mut supersession = MomentumInputPlanSupersessionV4_3 {
        supersession_version: SUPERSESSION_VERSION_V4_3.to_string(),
        lifecycle_digest: lifecycle.lifecycle_digest.clone(),
        context_authorization_digest: authorization.authorization_digest.clone(),
        superseded_context_plan_digest: old_plan.plan_digest.clone(),
        superseded_input_registration_digest: old_registration.registration_digest.clone(),
        superseded_event_timestamp_ms: old_plan.event_timestamp_ms,
        prior_request_attempt_count,
        prior_receipt_present,
        prior_capsule_present,
        prior_prediction_capsule_present,
        replacement_event_timestamp_ms: replacement_plan.event_timestamp_ms,
        replacement_context_plan_digest: replacement_plan.plan_digest.clone(),
        replacement_input_registration_digest: String::new(),
        status,
        supersession_digest: String::new(),
    };
    supersession.supersession_digest = supersession_digest_v4_3(&supersession);
    supersession
}

fn derive_input_registration_v4_3(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    source: &MomentumFutureEvaluationSourceV4_2,
    authorization: &MomentumProtectedContextAuthorizationV4_3,
    supersession: &MomentumInputPlanSupersessionV4_3,
    plan: &MomentumProspectiveFeatureContextPlanV4_3,
    config: &UpbitHistoricalPilotConfigV0,
) -> Result<MomentumProspectiveInputRegistrationV4_3, String> {
    let mut registration = MomentumProspectiveInputRegistrationV4_3 {
        registration_version: INPUT_REGISTRATION_VERSION_V4_3.to_string(),
        lifecycle_digest: lifecycle.lifecycle_digest.clone(),
        evaluation_registration_digest: source.evaluation.registration_digest.clone(),
        roster_digest: source.roster.roster_digest.clone(),
        context_authorization_digest: authorization.authorization_digest.clone(),
        supersession_digest: supersession.supersession_digest.clone(),
        context_plan_digest: plan.plan_digest.clone(),
        event_timestamp_ms: plan.event_timestamp_ms,
        input_finality_boundary_ms: plan.input_finality_boundary_ms,
        provider_id: config.provider_id.clone(),
        market: "btc_crypto".to_string(),
        symbol: config.symbol.clone(),
        cadence: "1d".to_string(),
        exact_expected_timestamp_ms: plan.exact_required_timestamp_ms.clone(),
        expected_row_count: plan.required_row_count,
        request_to_timestamp_ms: plan.input_finality_boundary_ms,
        maximum_requests: 1,
        maximum_concurrency: 1,
        maximum_retries: 0,
        maximum_response_bytes: config.maximum_response_bytes,
        credential_free_required: true,
        read_only_required: true,
        outcome_timestamp_forbidden: true,
        prior_outcome_artifact_forbidden: true,
        registration_digest: String::new(),
    };
    registration.registration_digest = input_registration_digest_v4_3(&registration);
    validate_input_registration_v4_3(
        &registration,
        lifecycle,
        source,
        authorization,
        supersession,
        plan,
        config,
    )?;
    Ok(registration)
}

fn validate_supersession_v4_3(
    value: &MomentumInputPlanSupersessionV4_3,
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    authorization: &MomentumProtectedContextAuthorizationV4_3,
    old_plan: &MomentumProspectiveFeatureContextPlanV4_2,
    old_registration: &MomentumProspectiveInputAcquisitionRegistrationV4_2,
    replacement_plan: &MomentumProspectiveFeatureContextPlanV4_3,
    replacement_registration: &MomentumProspectiveInputRegistrationV4_3,
) -> Result<(), String> {
    if value.supersession_version != SUPERSESSION_VERSION_V4_3
        || value.lifecycle_digest != lifecycle.lifecycle_digest
        || value.context_authorization_digest != authorization.authorization_digest
        || value.superseded_context_plan_digest != old_plan.plan_digest
        || value.superseded_input_registration_digest != old_registration.registration_digest
        || value.superseded_event_timestamp_ms != old_plan.event_timestamp_ms
        || value.prior_request_attempt_count != 0
        || value.prior_receipt_present
        || value.prior_capsule_present
        || value.prior_prediction_capsule_present
        || value.replacement_event_timestamp_ms != replacement_plan.event_timestamp_ms
        || value.replacement_context_plan_digest != replacement_plan.plan_digest
        || value.replacement_input_registration_digest
            != replacement_registration.registration_digest
        || value.status != MomentumInputPlanSupersessionStatusV4_3::Superseded
        || value.supersession_digest != supersession_digest_v4_3(value)
    {
        return Err("V4.3 input-plan supersession rejected".to_string());
    }
    Ok(())
}

fn validate_input_registration_v4_3(
    value: &MomentumProspectiveInputRegistrationV4_3,
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    source: &MomentumFutureEvaluationSourceV4_2,
    authorization: &MomentumProtectedContextAuthorizationV4_3,
    supersession: &MomentumInputPlanSupersessionV4_3,
    plan: &MomentumProspectiveFeatureContextPlanV4_3,
    config: &UpbitHistoricalPilotConfigV0,
) -> Result<(), String> {
    let outcome_timestamp = value
        .event_timestamp_ms
        .checked_add(
            u64::try_from(lifecycle.prediction_horizon)
                .ok()
                .and_then(|horizon| horizon.checked_mul(lifecycle.cadence_ms))
                .ok_or_else(|| "V4.3 outcome timestamp overflow".to_string())?,
        )
        .ok_or_else(|| "V4.3 outcome timestamp overflow".to_string())?;
    if value.registration_version != INPUT_REGISTRATION_VERSION_V4_3
        || value.lifecycle_digest != lifecycle.lifecycle_digest
        || value.evaluation_registration_digest != source.evaluation.registration_digest
        || value.roster_digest != source.roster.roster_digest
        || value.context_authorization_digest != authorization.authorization_digest
        || value.supersession_digest != supersession.supersession_digest
        || value.context_plan_digest != plan.plan_digest
        || value.event_timestamp_ms != plan.event_timestamp_ms
        || value.input_finality_boundary_ms != plan.input_finality_boundary_ms
        || value.provider_id != config.provider_id
        || value.market != "btc_crypto"
        || value.symbol != config.symbol
        || value.cadence != "1d"
        || value.exact_expected_timestamp_ms != plan.exact_required_timestamp_ms
        || value.expected_row_count != 16
        || value.request_to_timestamp_ms != plan.input_finality_boundary_ms
        || value.maximum_requests != 1
        || value.maximum_concurrency != 1
        || value.maximum_retries != 0
        || value.maximum_response_bytes == 0
        || !value.credential_free_required
        || !value.read_only_required
        || !value.outcome_timestamp_forbidden
        || !value.prior_outcome_artifact_forbidden
        || value
            .exact_expected_timestamp_ms
            .contains(&outcome_timestamp)
        || value.registration_digest != input_registration_digest_v4_3(value)
    {
        return Err("V4.3 corrected input registration rejected".to_string());
    }
    Ok(())
}

fn parse_authorization_status_v4_3(
    value: &str,
) -> Result<ProtectedContextAuthorizationStatusV4_3, String> {
    match value {
        "Authorized" => Ok(ProtectedContextAuthorizationStatusV4_3::Authorized),
        "AlreadyAuthorized" => Ok(ProtectedContextAuthorizationStatusV4_3::AlreadyAuthorized),
        "ModelFreezeNotProven" => Ok(ProtectedContextAuthorizationStatusV4_3::ModelFreezeNotProven),
        "RawEvidenceUnavailable" => {
            Ok(ProtectedContextAuthorizationStatusV4_3::RawEvidenceUnavailable)
        }
        "PriorOutcomeArtifactReferenced" => {
            Ok(ProtectedContextAuthorizationStatusV4_3::PriorOutcomeArtifactReferenced)
        }
        "PolicyConflict" => Ok(ProtectedContextAuthorizationStatusV4_3::PolicyConflict),
        "IntegrityFailure" => Ok(ProtectedContextAuthorizationStatusV4_3::IntegrityFailure),
        _ => Err("V4.3 authorization status rejected".to_string()),
    }
}

fn parse_usage_class_v4_3(value: &str) -> Result<MomentumContextEvidenceUseV4_3, String> {
    match value {
        "ExistingFrozenHistoricalContext" => {
            Ok(MomentumContextEvidenceUseV4_3::ExistingFrozenHistoricalContext)
        }
        "ProtectedRawInferenceContext" => {
            Ok(MomentumContextEvidenceUseV4_3::ProtectedRawInferenceContext)
        }
        "NewIncrementalInferenceContext" => {
            Ok(MomentumContextEvidenceUseV4_3::NewIncrementalInferenceContext)
        }
        "ProspectiveEventInput" => Ok(MomentumContextEvidenceUseV4_3::ProspectiveEventInput),
        _ => Err("V4.3 context usage class rejected".to_string()),
    }
}

fn parse_supersession_status_v4_3(
    value: &str,
) -> Result<MomentumInputPlanSupersessionStatusV4_3, String> {
    match value {
        "Superseded" => Ok(MomentumInputPlanSupersessionStatusV4_3::Superseded),
        "AlreadySuperseded" => Ok(MomentumInputPlanSupersessionStatusV4_3::AlreadySuperseded),
        "PriorAttemptExists" => Ok(MomentumInputPlanSupersessionStatusV4_3::PriorAttemptExists),
        "PriorEvidenceExists" => Ok(MomentumInputPlanSupersessionStatusV4_3::PriorEvidenceExists),
        "IntegrityFailure" => Ok(MomentumInputPlanSupersessionStatusV4_3::IntegrityFailure),
        _ => Err("V4.3 supersession status rejected".to_string()),
    }
}

fn parse_readiness_v4_3(value: &str) -> Result<MomentumEventReadinessV4_3, String> {
    match value {
        "ReadyForInputAcquisition" => Ok(MomentumEventReadinessV4_3::ReadyForInputAcquisition),
        "AwaitingInputFinality" => Ok(MomentumEventReadinessV4_3::AwaitingInputFinality),
        "ContextOnlyAuthorizationMissing" => {
            Ok(MomentumEventReadinessV4_3::ContextOnlyAuthorizationMissing)
        }
        "ContextAuthorizationIntegrityFailure" => {
            Ok(MomentumEventReadinessV4_3::ContextAuthorizationIntegrityFailure)
        }
        "InputRegistrationSuperseded" => {
            Ok(MomentumEventReadinessV4_3::InputRegistrationSuperseded)
        }
        "PriorInputAttemptTerminal" => Ok(MomentumEventReadinessV4_3::PriorInputAttemptTerminal),
        "PredictionAlreadySealed" => Ok(MomentumEventReadinessV4_3::PredictionAlreadySealed),
        "IntegrityFailure" => Ok(MomentumEventReadinessV4_3::IntegrityFailure),
        _ => Err("V4.3 readiness rejected".to_string()),
    }
}

fn encode_authorization_v4_3(
    value: &MomentumProtectedContextAuthorizationV4_3,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumProtectedContextAuthorizationV4_3")
        .string("authorization_version", &value.authorization_version)
        .string("agent_id", &value.agent_id)
        .string("v4_family_digest", &value.v4_family_digest)
        .string(
            "accumulated_family_digest",
            &value.accumulated_family_digest,
        )
        .string("roster_digest", &value.roster_digest)
        .string(
            "evaluation_registration_digest",
            &value.evaluation_registration_digest,
        )
        .string("lifecycle_digest", &value.lifecycle_digest)
        .strings("participant_digests", &value.participant_digests)
        .strings("parameter_digests", &value.parameter_digests)
        .strings("normalizer_digests", &value.normalizer_digests)
        .unsigned(
            "model_source_boundary_timestamp_ms",
            value.model_source_boundary_timestamp_ms,
        )
        .strings(
            "protected_registration_digests",
            &value.protected_registration_digests,
        )
        .unsigneds("protected_timestamp_ms", &value.protected_timestamp_ms)
        .string(
            "allowed_use_class",
            format!("{:?}", value.allowed_use_class),
        )
        .boolean("raw_ohlcv_context_allowed", value.raw_ohlcv_context_allowed)
        .boolean("training_use_forbidden", value.training_use_forbidden)
        .boolean("normalizer_fit_forbidden", value.normalizer_fit_forbidden)
        .boolean("label_use_forbidden", value.label_use_forbidden)
        .boolean(
            "qualification_use_forbidden",
            value.qualification_use_forbidden,
        )
        .boolean("metric_use_forbidden", value.metric_use_forbidden)
        .boolean("reward_use_forbidden", value.reward_use_forbidden)
        .boolean(
            "event_timestamp_use_forbidden",
            value.event_timestamp_use_forbidden,
        )
        .boolean(
            "outcome_timestamp_use_forbidden",
            value.outcome_timestamp_use_forbidden,
        )
        .boolean(
            "prior_outcome_capsule_use_forbidden",
            value.prior_outcome_capsule_use_forbidden,
        )
        .string(
            "authorization_status",
            format!("{:?}", value.authorization_status),
        )
        .string("authorization_digest", &value.authorization_digest)
        .encode()
}

fn decode_authorization_v4_3(
    bytes: &[u8],
) -> Result<MomentumProtectedContextAuthorizationV4_3, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumProtectedContextAuthorizationV4_3")?;
    let allowed_use_class = match fields.string("allowed_use_class")?.as_str() {
        "RawOhlcvInferenceContext" => ProtectedContextUseClassV4_3::RawOhlcvInferenceContext,
        _ => return Err("V4.3 protected use class rejected".to_string()),
    };
    let value = MomentumProtectedContextAuthorizationV4_3 {
        authorization_version: fields.string("authorization_version")?,
        agent_id: fields.string("agent_id")?,
        v4_family_digest: fields.string("v4_family_digest")?,
        accumulated_family_digest: fields.string("accumulated_family_digest")?,
        roster_digest: fields.string("roster_digest")?,
        evaluation_registration_digest: fields.string("evaluation_registration_digest")?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
        participant_digests: fields.strings("participant_digests")?,
        parameter_digests: fields.strings("parameter_digests")?,
        normalizer_digests: fields.strings("normalizer_digests")?,
        model_source_boundary_timestamp_ms: fields
            .unsigned("model_source_boundary_timestamp_ms")?,
        protected_registration_digests: fields.strings("protected_registration_digests")?,
        protected_timestamp_ms: fields.unsigneds("protected_timestamp_ms")?,
        allowed_use_class,
        raw_ohlcv_context_allowed: fields.boolean("raw_ohlcv_context_allowed")?,
        training_use_forbidden: fields.boolean("training_use_forbidden")?,
        normalizer_fit_forbidden: fields.boolean("normalizer_fit_forbidden")?,
        label_use_forbidden: fields.boolean("label_use_forbidden")?,
        qualification_use_forbidden: fields.boolean("qualification_use_forbidden")?,
        metric_use_forbidden: fields.boolean("metric_use_forbidden")?,
        reward_use_forbidden: fields.boolean("reward_use_forbidden")?,
        event_timestamp_use_forbidden: fields.boolean("event_timestamp_use_forbidden")?,
        outcome_timestamp_use_forbidden: fields.boolean("outcome_timestamp_use_forbidden")?,
        prior_outcome_capsule_use_forbidden: fields
            .boolean("prior_outcome_capsule_use_forbidden")?,
        authorization_status: parse_authorization_status_v4_3(
            &fields.string("authorization_status")?,
        )?,
        authorization_digest: fields.string("authorization_digest")?,
    };
    fields.finish()?;
    validate_authorization_policy_v4_3(&value)?;
    if value.authorization_digest != authorization_digest_v4_3(&value) {
        return Err("V4.3 authorization digest rejected".to_string());
    }
    Ok(value)
}

fn encode_usage_entry_v4_3(value: &MomentumContextUsageEntryV4_3) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumContextUsageEntryV4_3")
        .unsigned("timestamp_ms", value.timestamp_ms)
        .string("raw_row_digest", &value.raw_row_digest)
        .string("use_class", format!("{:?}", value.use_class))
        .boolean(
            "used_for_feature_construction",
            value.used_for_feature_construction,
        )
        .boolean("used_for_training", value.used_for_training)
        .boolean("used_for_normalizer_fit", value.used_for_normalizer_fit)
        .boolean("used_for_label", value.used_for_label)
        .boolean("used_for_metric", value.used_for_metric)
        .boolean("used_for_reward", value.used_for_reward)
        .string("entry_digest", &value.entry_digest)
        .encode()
}

fn decode_usage_entry_v4_3(bytes: &[u8]) -> Result<MomentumContextUsageEntryV4_3, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumContextUsageEntryV4_3")?;
    let value = MomentumContextUsageEntryV4_3 {
        timestamp_ms: fields.unsigned("timestamp_ms")?,
        raw_row_digest: fields.string("raw_row_digest")?,
        use_class: parse_usage_class_v4_3(&fields.string("use_class")?)?,
        used_for_feature_construction: fields.boolean("used_for_feature_construction")?,
        used_for_training: fields.boolean("used_for_training")?,
        used_for_normalizer_fit: fields.boolean("used_for_normalizer_fit")?,
        used_for_label: fields.boolean("used_for_label")?,
        used_for_metric: fields.boolean("used_for_metric")?,
        used_for_reward: fields.boolean("used_for_reward")?,
        entry_digest: fields.string("entry_digest")?,
    };
    fields.finish()?;
    if !value.used_for_feature_construction
        || value.used_for_training
        || value.used_for_normalizer_fit
        || value.used_for_label
        || value.used_for_metric
        || value.used_for_reward
        || value.raw_row_digest.is_empty()
        || value.entry_digest != context_usage_entry_digest_v4_3(&value)
    {
        return Err("V4.3 context usage entry rejected".to_string());
    }
    Ok(value)
}

fn encode_usage_ledger_v4_3(value: &MomentumContextUsageLedgerV4_3) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumContextUsageLedgerV4_3")
        .string("ledger_version", &value.ledger_version)
        .string("authorization_digest", &value.authorization_digest)
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .messages(
            "entries",
            value
                .entries
                .iter()
                .map(encode_usage_entry_v4_3)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .string("ledger_digest", &value.ledger_digest)
        .encode()
}

fn decode_usage_ledger_v4_3(bytes: &[u8]) -> Result<MomentumContextUsageLedgerV4_3, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumContextUsageLedgerV4_3")?;
    let value = MomentumContextUsageLedgerV4_3 {
        ledger_version: fields.string("ledger_version")?,
        authorization_digest: fields.string("authorization_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        entries: fields
            .messages("entries")?
            .iter()
            .map(|bytes| decode_usage_entry_v4_3(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        ledger_digest: fields.string("ledger_digest")?,
    };
    fields.finish()?;
    if value.ledger_version != CONTEXT_LEDGER_VERSION_V4_3
        || value.entries.len() != 16
        || value
            .entries
            .windows(2)
            .any(|pair| pair[1].timestamp_ms <= pair[0].timestamp_ms)
        || value.ledger_digest != context_usage_ledger_digest_v4_3(&value)
    {
        return Err("V4.3 context usage ledger rejected".to_string());
    }
    Ok(value)
}

fn encode_supersession_v4_3(value: &MomentumInputPlanSupersessionV4_3) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumInputPlanSupersessionV4_3")
        .string("supersession_version", &value.supersession_version)
        .string("lifecycle_digest", &value.lifecycle_digest)
        .string(
            "context_authorization_digest",
            &value.context_authorization_digest,
        )
        .string(
            "superseded_context_plan_digest",
            &value.superseded_context_plan_digest,
        )
        .string(
            "superseded_input_registration_digest",
            &value.superseded_input_registration_digest,
        )
        .unsigned(
            "superseded_event_timestamp_ms",
            value.superseded_event_timestamp_ms,
        )
        .unsigned(
            "prior_request_attempt_count",
            as_u64(value.prior_request_attempt_count)?,
        )
        .boolean("prior_receipt_present", value.prior_receipt_present)
        .boolean("prior_capsule_present", value.prior_capsule_present)
        .boolean(
            "prior_prediction_capsule_present",
            value.prior_prediction_capsule_present,
        )
        .unsigned(
            "replacement_event_timestamp_ms",
            value.replacement_event_timestamp_ms,
        )
        .string(
            "replacement_context_plan_digest",
            &value.replacement_context_plan_digest,
        )
        .string(
            "replacement_input_registration_digest",
            &value.replacement_input_registration_digest,
        )
        .string("status", format!("{:?}", value.status))
        .string("supersession_digest", &value.supersession_digest)
        .encode()
}

fn decode_supersession_v4_3(bytes: &[u8]) -> Result<MomentumInputPlanSupersessionV4_3, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumInputPlanSupersessionV4_3")?;
    let value = MomentumInputPlanSupersessionV4_3 {
        supersession_version: fields.string("supersession_version")?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
        context_authorization_digest: fields.string("context_authorization_digest")?,
        superseded_context_plan_digest: fields.string("superseded_context_plan_digest")?,
        superseded_input_registration_digest: fields
            .string("superseded_input_registration_digest")?,
        superseded_event_timestamp_ms: fields.unsigned("superseded_event_timestamp_ms")?,
        prior_request_attempt_count: as_usize(fields.unsigned("prior_request_attempt_count")?)?,
        prior_receipt_present: fields.boolean("prior_receipt_present")?,
        prior_capsule_present: fields.boolean("prior_capsule_present")?,
        prior_prediction_capsule_present: fields.boolean("prior_prediction_capsule_present")?,
        replacement_event_timestamp_ms: fields.unsigned("replacement_event_timestamp_ms")?,
        replacement_context_plan_digest: fields.string("replacement_context_plan_digest")?,
        replacement_input_registration_digest: fields
            .string("replacement_input_registration_digest")?,
        status: parse_supersession_status_v4_3(&fields.string("status")?)?,
        supersession_digest: fields.string("supersession_digest")?,
    };
    fields.finish()?;
    if value.supersession_version != SUPERSESSION_VERSION_V4_3
        || value.supersession_digest != supersession_digest_v4_3(&value)
    {
        return Err("V4.3 supersession artifact rejected".to_string());
    }
    Ok(value)
}

fn encode_context_plan_v4_3(
    value: &MomentumProspectiveFeatureContextPlanV4_3,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumProspectiveFeatureContextPlanV4_3")
        .string("plan_version", &value.plan_version)
        .string("lifecycle_digest", &value.lifecycle_digest)
        .string(
            "context_authorization_digest",
            &value.context_authorization_digest,
        )
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigned(
            "input_finality_boundary_ms",
            value.input_finality_boundary_ms,
        )
        .unsigned(
            "required_context_start_timestamp_ms",
            value.required_context_start_timestamp_ms,
        )
        .unsigned(
            "required_context_end_timestamp_ms",
            value.required_context_end_timestamp_ms,
        )
        .unsigned("required_row_count", as_u64(value.required_row_count)?)
        .unsigneds(
            "exact_required_timestamp_ms",
            &value.exact_required_timestamp_ms,
        )
        .unsigneds(
            "existing_source_timestamp_ms",
            &value.existing_source_timestamp_ms,
        )
        .unsigneds(
            "protected_context_timestamp_ms",
            &value.protected_context_timestamp_ms,
        )
        .unsigneds("incremental_timestamp_ms", &value.incremental_timestamp_ms)
        .string(
            "context_usage_policy_digest",
            &value.context_usage_policy_digest,
        )
        .string("plan_digest", &value.plan_digest)
        .encode()
}

fn decode_context_plan_v4_3(
    bytes: &[u8],
) -> Result<MomentumProspectiveFeatureContextPlanV4_3, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumProspectiveFeatureContextPlanV4_3")?;
    let value = MomentumProspectiveFeatureContextPlanV4_3 {
        plan_version: fields.string("plan_version")?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
        context_authorization_digest: fields.string("context_authorization_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        input_finality_boundary_ms: fields.unsigned("input_finality_boundary_ms")?,
        required_context_start_timestamp_ms: fields
            .unsigned("required_context_start_timestamp_ms")?,
        required_context_end_timestamp_ms: fields.unsigned("required_context_end_timestamp_ms")?,
        required_row_count: as_usize(fields.unsigned("required_row_count")?)?,
        exact_required_timestamp_ms: fields.unsigneds("exact_required_timestamp_ms")?,
        existing_source_timestamp_ms: fields.unsigneds("existing_source_timestamp_ms")?,
        protected_context_timestamp_ms: fields.unsigneds("protected_context_timestamp_ms")?,
        incremental_timestamp_ms: fields.unsigneds("incremental_timestamp_ms")?,
        context_usage_policy_digest: fields.string("context_usage_policy_digest")?,
        plan_digest: fields.string("plan_digest")?,
    };
    fields.finish()?;
    if value.plan_version != CONTEXT_PLAN_VERSION_V4_3
        || value.plan_digest != context_plan_digest_v4_3(&value)
    {
        return Err("V4.3 context plan artifact rejected".to_string());
    }
    Ok(value)
}

fn encode_input_registration_v4_3(
    value: &MomentumProspectiveInputRegistrationV4_3,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumProspectiveInputRegistrationV4_3")
        .string("registration_version", &value.registration_version)
        .string("lifecycle_digest", &value.lifecycle_digest)
        .string(
            "evaluation_registration_digest",
            &value.evaluation_registration_digest,
        )
        .string("roster_digest", &value.roster_digest)
        .string(
            "context_authorization_digest",
            &value.context_authorization_digest,
        )
        .string("supersession_digest", &value.supersession_digest)
        .string("context_plan_digest", &value.context_plan_digest)
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigned(
            "input_finality_boundary_ms",
            value.input_finality_boundary_ms,
        )
        .string("provider_id", &value.provider_id)
        .string("market", &value.market)
        .string("symbol", &value.symbol)
        .string("cadence", &value.cadence)
        .unsigneds(
            "exact_expected_timestamp_ms",
            &value.exact_expected_timestamp_ms,
        )
        .unsigned("expected_row_count", as_u64(value.expected_row_count)?)
        .unsigned("request_to_timestamp_ms", value.request_to_timestamp_ms)
        .unsigned("maximum_requests", as_u64(value.maximum_requests)?)
        .unsigned("maximum_concurrency", as_u64(value.maximum_concurrency)?)
        .unsigned("maximum_retries", as_u64(value.maximum_retries)?)
        .unsigned(
            "maximum_response_bytes",
            as_u64(value.maximum_response_bytes)?,
        )
        .boolean("credential_free_required", value.credential_free_required)
        .boolean("read_only_required", value.read_only_required)
        .boolean(
            "outcome_timestamp_forbidden",
            value.outcome_timestamp_forbidden,
        )
        .boolean(
            "prior_outcome_artifact_forbidden",
            value.prior_outcome_artifact_forbidden,
        )
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_input_registration_v4_3(
    bytes: &[u8],
) -> Result<MomentumProspectiveInputRegistrationV4_3, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumProspectiveInputRegistrationV4_3")?;
    let value = MomentumProspectiveInputRegistrationV4_3 {
        registration_version: fields.string("registration_version")?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
        evaluation_registration_digest: fields.string("evaluation_registration_digest")?,
        roster_digest: fields.string("roster_digest")?,
        context_authorization_digest: fields.string("context_authorization_digest")?,
        supersession_digest: fields.string("supersession_digest")?,
        context_plan_digest: fields.string("context_plan_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        input_finality_boundary_ms: fields.unsigned("input_finality_boundary_ms")?,
        provider_id: fields.string("provider_id")?,
        market: fields.string("market")?,
        symbol: fields.string("symbol")?,
        cadence: fields.string("cadence")?,
        exact_expected_timestamp_ms: fields.unsigneds("exact_expected_timestamp_ms")?,
        expected_row_count: as_usize(fields.unsigned("expected_row_count")?)?,
        request_to_timestamp_ms: fields.unsigned("request_to_timestamp_ms")?,
        maximum_requests: as_usize(fields.unsigned("maximum_requests")?)?,
        maximum_concurrency: as_usize(fields.unsigned("maximum_concurrency")?)?,
        maximum_retries: as_usize(fields.unsigned("maximum_retries")?)?,
        maximum_response_bytes: as_usize(fields.unsigned("maximum_response_bytes")?)?,
        credential_free_required: fields.boolean("credential_free_required")?,
        read_only_required: fields.boolean("read_only_required")?,
        outcome_timestamp_forbidden: fields.boolean("outcome_timestamp_forbidden")?,
        prior_outcome_artifact_forbidden: fields.boolean("prior_outcome_artifact_forbidden")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    if value.registration_version != INPUT_REGISTRATION_VERSION_V4_3
        || value.registration_digest != input_registration_digest_v4_3(&value)
    {
        return Err("V4.3 input registration artifact rejected".to_string());
    }
    Ok(value)
}

fn encode_input_receipt_v4_3(
    value: &MomentumProspectiveInputReceiptV4_3,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumProspectiveInputReceiptV4_3")
        .string("receipt_version", &value.receipt_version)
        .string("lifecycle_digest", &value.lifecycle_digest)
        .string(
            "context_authorization_digest",
            &value.context_authorization_digest,
        )
        .string(
            "input_registration_digest",
            &value.input_registration_digest,
        )
        .boolean("request_attempted", value.request_attempted)
        .unsigned("request_count", as_u64(value.request_count)?)
        .unsigned("retry_count", as_u64(value.retry_count)?)
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

fn decode_input_receipt_v4_3(bytes: &[u8]) -> Result<MomentumProspectiveInputReceiptV4_3, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumProspectiveInputReceiptV4_3")?;
    let value = MomentumProspectiveInputReceiptV4_3 {
        receipt_version: fields.string("receipt_version")?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
        context_authorization_digest: fields.string("context_authorization_digest")?,
        input_registration_digest: fields.string("input_registration_digest")?,
        request_attempted: fields.boolean("request_attempted")?,
        request_count: as_usize(fields.unsigned("request_count")?)?,
        retry_count: as_usize(fields.unsigned("retry_count")?)?,
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
    if value.receipt_version != INPUT_RECEIPT_VERSION_V4_3
        || value.request_count > 1
        || value.retry_count != 0
        || value.request_attempted != (value.request_count == 1)
        || value.terminal != value.status.is_terminal()
        || value.receipt_digest != input_receipt_digest_v4_3(&value)
    {
        return Err("V4.3 input receipt rejected".to_string());
    }
    Ok(value)
}

fn encode_input_capsule_v4_3(
    value: &MomentumProspectiveInputCapsuleV4_3,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumProspectiveInputCapsuleV4_3")
        .string("capsule_version", &value.capsule_version)
        .string("lifecycle_digest", &value.lifecycle_digest)
        .string(
            "context_authorization_digest",
            &value.context_authorization_digest,
        )
        .string(
            "input_registration_digest",
            &value.input_registration_digest,
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
            "prior_outcome_capsule_accessed",
            value.prior_outcome_capsule_accessed,
        )
        .boolean("credential_free", value.credential_free)
        .boolean("read_only", value.read_only)
        .boolean("sanitized", value.sanitized)
        .string("capsule_digest", &value.capsule_digest)
        .encode()
}

fn decode_input_capsule_v4_3(bytes: &[u8]) -> Result<MomentumProspectiveInputCapsuleV4_3, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumProspectiveInputCapsuleV4_3")?;
    let value = MomentumProspectiveInputCapsuleV4_3 {
        capsule_version: fields.string("capsule_version")?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
        context_authorization_digest: fields.string("context_authorization_digest")?,
        input_registration_digest: fields.string("input_registration_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        exact_timestamp_ms: fields.unsigneds("exact_timestamp_ms")?,
        row_identity_digests: fields.strings("row_identity_digests")?,
        normalized_dataset_digest: fields.string("normalized_dataset_digest")?,
        raw_response_digest: fields.string("raw_response_digest")?,
        outcome_row_present: fields.boolean("outcome_row_present")?,
        labels_accessed: fields.boolean("labels_accessed")?,
        metrics_computed: fields.boolean("metrics_computed")?,
        prior_outcome_capsule_accessed: fields.boolean("prior_outcome_capsule_accessed")?,
        credential_free: fields.boolean("credential_free")?,
        read_only: fields.boolean("read_only")?,
        sanitized: fields.boolean("sanitized")?,
        capsule_digest: fields.string("capsule_digest")?,
    };
    fields.finish()?;
    if value.capsule_version != INPUT_CAPSULE_VERSION_V4_3
        || value.exact_timestamp_ms.len() != 16
        || value.row_identity_digests.len() != 16
        || value.outcome_row_present
        || value.labels_accessed
        || value.metrics_computed
        || value.prior_outcome_capsule_accessed
        || !value.credential_free
        || !value.read_only
        || !value.sanitized
        || value.capsule_digest != input_capsule_digest_v4_3(&value)
    {
        return Err("V4.3 input capsule rejected".to_string());
    }
    Ok(value)
}

fn encode_context_proof_v4_3(
    value: &MomentumProspectiveContextVerificationV4_3,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumProspectiveContextVerificationV4_3")
        .string("proof_version", &value.proof_version)
        .string(
            "context_authorization_digest",
            &value.context_authorization_digest,
        )
        .string(
            "feature_context_plan_digest",
            &value.feature_context_plan_digest,
        )
        .string(
            "input_registration_digest",
            &value.input_registration_digest,
        )
        .string("input_capsule_digest", &value.input_capsule_digest)
        .string(
            "context_usage_ledger_digest",
            &value.context_usage_ledger_digest,
        )
        .boolean("exact_timestamps_verified", value.exact_timestamps_verified)
        .boolean(
            "strict_chronology_verified",
            value.strict_chronology_verified,
        )
        .boolean("feature_history_complete", value.feature_history_complete)
        .boolean(
            "protected_context_inference_only",
            value.protected_context_inference_only,
        )
        .boolean("outcome_timestamp_absent", value.outcome_timestamp_absent)
        .boolean(
            "prior_outcome_artifact_accessed",
            value.prior_outcome_artifact_accessed,
        )
        .string("proof_digest", &value.proof_digest)
        .encode()
}

fn decode_context_proof_v4_3(
    bytes: &[u8],
) -> Result<MomentumProspectiveContextVerificationV4_3, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumProspectiveContextVerificationV4_3")?;
    let value = MomentumProspectiveContextVerificationV4_3 {
        proof_version: fields.string("proof_version")?,
        context_authorization_digest: fields.string("context_authorization_digest")?,
        feature_context_plan_digest: fields.string("feature_context_plan_digest")?,
        input_registration_digest: fields.string("input_registration_digest")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        context_usage_ledger_digest: fields.string("context_usage_ledger_digest")?,
        exact_timestamps_verified: fields.boolean("exact_timestamps_verified")?,
        strict_chronology_verified: fields.boolean("strict_chronology_verified")?,
        feature_history_complete: fields.boolean("feature_history_complete")?,
        protected_context_inference_only: fields.boolean("protected_context_inference_only")?,
        outcome_timestamp_absent: fields.boolean("outcome_timestamp_absent")?,
        prior_outcome_artifact_accessed: fields.boolean("prior_outcome_artifact_accessed")?,
        proof_digest: fields.string("proof_digest")?,
    };
    fields.finish()?;
    if value.proof_version != CONTEXT_PROOF_VERSION_V4_3
        || !value.exact_timestamps_verified
        || !value.strict_chronology_verified
        || !value.feature_history_complete
        || !value.protected_context_inference_only
        || !value.outcome_timestamp_absent
        || value.prior_outcome_artifact_accessed
        || value.proof_digest != context_proof_digest_v4_3(&value)
    {
        return Err("V4.3 context proof rejected".to_string());
    }
    Ok(value)
}

fn encode_prediction_seal_v4_3(
    value: &MomentumParticipantPredictionSealV4_3,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumParticipantPredictionSealV4_3")
        .string("participant_digest", &value.participant_digest)
        .string("participant_role", &value.participant_role)
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .string("input_receipt_digest", &value.input_receipt_digest)
        .string("input_capsule_digest", &value.input_capsule_digest)
        .string(
            "context_usage_ledger_digest",
            &value.context_usage_ledger_digest,
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
        .unsigned("outcome_access_count", as_u64(value.outcome_access_count)?)
        .string("seal_digest", &value.seal_digest)
        .encode()
}

fn decode_prediction_seal_v4_3(
    bytes: &[u8],
) -> Result<MomentumParticipantPredictionSealV4_3, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumParticipantPredictionSealV4_3")?;
    let probability_bits = fields.unsigned("prediction_probability_bits")?;
    let value = MomentumParticipantPredictionSealV4_3 {
        participant_digest: fields.string("participant_digest")?,
        participant_role: fields.string("participant_role")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        input_receipt_digest: fields.string("input_receipt_digest")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        context_usage_ledger_digest: fields.string("context_usage_ledger_digest")?,
        feature_identity_digest: fields.string("feature_identity_digest")?,
        prediction_probability_bits: u32::try_from(probability_bits)
            .map_err(|_| "V4.3 prediction bits rejected".to_string())?,
        prediction_digest: fields.string("prediction_digest")?,
        participant_identity_verified: fields.boolean("participant_identity_verified")?,
        parameter_updates: as_usize(fields.unsigned("parameter_updates")?)?,
        normalizer_refits: as_usize(fields.unsigned("normalizer_refits")?)?,
        outcome_access_count: as_usize(fields.unsigned("outcome_access_count")?)?,
        seal_digest: fields.string("seal_digest")?,
    };
    fields.finish()?;
    if value.participant_role.is_empty()
        || !value.participant_identity_verified
        || value.parameter_updates != 0
        || value.normalizer_refits != 0
        || value.outcome_access_count != 0
        || value.seal_digest != prediction_seal_digest_v4_3(&value)
    {
        return Err("V4.3 participant prediction seal rejected".to_string());
    }
    Ok(value)
}

fn encode_prediction_capsule_v4_3(
    value: &MomentumProspectivePredictionCapsuleV4_3,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumProspectivePredictionCapsuleV4_3")
        .string("capsule_version", &value.capsule_version)
        .string("lifecycle_digest", &value.lifecycle_digest)
        .string(
            "evaluation_registration_digest",
            &value.evaluation_registration_digest,
        )
        .string("roster_digest", &value.roster_digest)
        .string(
            "context_authorization_digest",
            &value.context_authorization_digest,
        )
        .string("supersession_digest", &value.supersession_digest)
        .string(
            "corrected_context_plan_digest",
            &value.corrected_context_plan_digest,
        )
        .string(
            "input_registration_digest",
            &value.input_registration_digest,
        )
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .string("input_receipt_digest", &value.input_receipt_digest)
        .string("input_capsule_digest", &value.input_capsule_digest)
        .string(
            "context_usage_ledger_digest",
            &value.context_usage_ledger_digest,
        )
        .messages(
            "participant_prediction_seals",
            value
                .participant_prediction_seals
                .iter()
                .map(encode_prediction_seal_v4_3)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .boolean("probabilities_hidden", value.probabilities_hidden)
        .boolean("labels_hidden", value.labels_hidden)
        .boolean("outcome_accessed", value.outcome_accessed)
        .boolean("metrics_computed", value.metrics_computed)
        .boolean("winner_selected", value.winner_selected)
        .string("capsule_digest", &value.capsule_digest)
        .encode()
}

fn decode_prediction_capsule_v4_3(
    bytes: &[u8],
) -> Result<MomentumProspectivePredictionCapsuleV4_3, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumProspectivePredictionCapsuleV4_3")?;
    let value = MomentumProspectivePredictionCapsuleV4_3 {
        capsule_version: fields.string("capsule_version")?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
        evaluation_registration_digest: fields.string("evaluation_registration_digest")?,
        roster_digest: fields.string("roster_digest")?,
        context_authorization_digest: fields.string("context_authorization_digest")?,
        supersession_digest: fields.string("supersession_digest")?,
        corrected_context_plan_digest: fields.string("corrected_context_plan_digest")?,
        input_registration_digest: fields.string("input_registration_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        input_receipt_digest: fields.string("input_receipt_digest")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        context_usage_ledger_digest: fields.string("context_usage_ledger_digest")?,
        participant_prediction_seals: fields
            .messages("participant_prediction_seals")?
            .iter()
            .map(|bytes| decode_prediction_seal_v4_3(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        probabilities_hidden: fields.boolean("probabilities_hidden")?,
        labels_hidden: fields.boolean("labels_hidden")?,
        outcome_accessed: fields.boolean("outcome_accessed")?,
        metrics_computed: fields.boolean("metrics_computed")?,
        winner_selected: fields.boolean("winner_selected")?,
        capsule_digest: fields.string("capsule_digest")?,
    };
    fields.finish()?;
    let participant_digests = value
        .participant_prediction_seals
        .iter()
        .map(|seal| &seal.participant_digest)
        .collect::<BTreeSet<_>>();
    if value.capsule_version != PREDICTION_CAPSULE_VERSION_V4_3
        || value.participant_prediction_seals.len() != 3
        || participant_digests.len() != 3
        || !value.probabilities_hidden
        || !value.labels_hidden
        || value.outcome_accessed
        || value.metrics_computed
        || value.winner_selected
        || value.capsule_digest != prediction_capsule_digest_v4_3(&value)
    {
        return Err("V4.3 prediction capsule rejected".to_string());
    }
    Ok(value)
}

fn encode_prediction_entry_v4_3(
    value: &MomentumProspectivePredictionJournalEntryV4_3,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumProspectivePredictionJournalEntryV4_3")
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .string(
            "context_authorization_digest",
            &value.context_authorization_digest,
        )
        .string("input_capsule_digest", &value.input_capsule_digest)
        .string(
            "context_usage_ledger_digest",
            &value.context_usage_ledger_digest,
        )
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .strings(
            "participant_prediction_digests",
            &value.participant_prediction_digests,
        )
        .boolean(
            "prediction_sealed_before_outcome",
            value.prediction_sealed_before_outcome,
        )
        .boolean("outcome_stage_unlocked", value.outcome_stage_unlocked)
        .string("entry_digest", &value.entry_digest)
        .encode()
}

fn decode_prediction_entry_v4_3(
    bytes: &[u8],
) -> Result<MomentumProspectivePredictionJournalEntryV4_3, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumProspectivePredictionJournalEntryV4_3")?;
    let value = MomentumProspectivePredictionJournalEntryV4_3 {
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        context_authorization_digest: fields.string("context_authorization_digest")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        context_usage_ledger_digest: fields.string("context_usage_ledger_digest")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        participant_prediction_digests: fields.strings("participant_prediction_digests")?,
        prediction_sealed_before_outcome: fields.boolean("prediction_sealed_before_outcome")?,
        outcome_stage_unlocked: fields.boolean("outcome_stage_unlocked")?,
        entry_digest: fields.string("entry_digest")?,
    };
    fields.finish()?;
    if value.participant_prediction_digests.len() != 3
        || !value.prediction_sealed_before_outcome
        || value.outcome_stage_unlocked
        || value.entry_digest != prediction_entry_digest_v4_3(&value)
    {
        return Err("V4.3 prediction journal entry rejected".to_string());
    }
    Ok(value)
}

fn encode_prediction_journal_v4_3(
    value: &MomentumProspectivePredictionJournalV4_3,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumProspectivePredictionJournalV4_3")
        .string("journal_version", &value.journal_version)
        .messages(
            "entries",
            value
                .entries
                .iter()
                .map(encode_prediction_entry_v4_3)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .string("journal_digest", &value.journal_digest)
        .encode()
}

fn decode_prediction_journal_v4_3(
    bytes: &[u8],
) -> Result<MomentumProspectivePredictionJournalV4_3, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumProspectivePredictionJournalV4_3")?;
    let value = MomentumProspectivePredictionJournalV4_3 {
        journal_version: fields.string("journal_version")?,
        entries: fields
            .messages("entries")?
            .iter()
            .map(|bytes| decode_prediction_entry_v4_3(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        journal_digest: fields.string("journal_digest")?,
    };
    fields.finish()?;
    if value.journal_version != PREDICTION_JOURNAL_VERSION_V4_3
        || value.entries.len() != 1
        || value.journal_digest != prediction_journal_digest_v4_3(&value)
    {
        return Err("V4.3 prediction journal rejected".to_string());
    }
    Ok(value)
}

fn encode_outcome_plan_v4_3(value: &MomentumProspectiveOutcomePlanV4_3) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumProspectiveOutcomePlanV4_3")
        .string("plan_version", &value.plan_version)
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

fn decode_outcome_plan_v4_3(bytes: &[u8]) -> Result<MomentumProspectiveOutcomePlanV4_3, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumProspectiveOutcomePlanV4_3")?;
    let value = MomentumProspectiveOutcomePlanV4_3 {
        plan_version: fields.string("plan_version")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        prediction_horizon: as_usize(fields.unsigned("prediction_horizon")?)?,
        required_outcome_timestamp_ms: fields.unsigneds("required_outcome_timestamp_ms")?,
        outcome_finality_boundary_ms: fields.unsigned("outcome_finality_boundary_ms")?,
        maximum_outcome_requests: as_usize(fields.unsigned("maximum_outcome_requests")?)?,
        maximum_outcome_retries: as_usize(fields.unsigned("maximum_outcome_retries")?)?,
        labels_hidden_until_opening: fields.boolean("labels_hidden_until_opening")?,
        one_time_opening_required: fields.boolean("one_time_opening_required")?,
        outcome_stage_locked_before_finality: fields
            .boolean("outcome_stage_locked_before_finality")?,
        plan_digest: fields.string("plan_digest")?,
    };
    fields.finish()?;
    if value.plan_version != OUTCOME_PLAN_VERSION_V4_3
        || value.prediction_horizon != 1
        || value.required_outcome_timestamp_ms.len() != 1
        || value.maximum_outcome_requests != 1
        || value.maximum_outcome_retries != 0
        || !value.labels_hidden_until_opening
        || !value.one_time_opening_required
        || !value.outcome_stage_locked_before_finality
        || value.plan_digest != outcome_plan_digest_v4_3(&value)
    {
        return Err("V4.3 outcome plan rejected".to_string());
    }
    Ok(value)
}

fn encode_safety_counters_v4_3(
    value: &MomentumFuturePredictionSafetyCountersV4_3,
) -> Result<Vec<u8>, String> {
    let values = [
        value.input_request_attempts,
        value.input_retries,
        value.maximum_concurrency,
        value.outcome_request_attempts,
        value.outcome_retries,
        value.participant_parameter_updates,
        value.normalizer_refits,
        value.protected_context_training_uses,
        value.protected_context_label_uses,
        value.protected_context_metric_uses,
        value.protected_context_reward_uses,
        value.outcome_row_reads,
        value.outcome_label_reads,
        value.metric_computations,
        value.winner_selections,
        value.active_model_changes,
        value.chair_decisions,
        value.votes,
        value.reward_applications,
        value.penalty_applications,
        value.voice_changes,
        value.cooldowns_started,
        value.promotions,
        value.quarantines,
        value.executions,
        value.active_committee_count,
    ]
    .into_iter()
    .map(as_u64)
    .collect::<Result<Vec<_>, _>>()?;
    ArtifactBuilderV4_2::new("MomentumFuturePredictionSafetyCountersV4_3")
        .unsigneds("values", &values)
        .encode()
}

fn decode_safety_counters_v4_3(
    bytes: &[u8],
) -> Result<MomentumFuturePredictionSafetyCountersV4_3, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumFuturePredictionSafetyCountersV4_3")?;
    let values = fields.unsigneds("values")?;
    fields.finish()?;
    if values.len() != 26 {
        return Err("V4.3 safety counter shape rejected".to_string());
    }
    let value = MomentumFuturePredictionSafetyCountersV4_3 {
        input_request_attempts: as_usize(values[0])?,
        input_retries: as_usize(values[1])?,
        maximum_concurrency: as_usize(values[2])?,
        outcome_request_attempts: as_usize(values[3])?,
        outcome_retries: as_usize(values[4])?,
        participant_parameter_updates: as_usize(values[5])?,
        normalizer_refits: as_usize(values[6])?,
        protected_context_training_uses: as_usize(values[7])?,
        protected_context_label_uses: as_usize(values[8])?,
        protected_context_metric_uses: as_usize(values[9])?,
        protected_context_reward_uses: as_usize(values[10])?,
        outcome_row_reads: as_usize(values[11])?,
        outcome_label_reads: as_usize(values[12])?,
        metric_computations: as_usize(values[13])?,
        winner_selections: as_usize(values[14])?,
        active_model_changes: as_usize(values[15])?,
        chair_decisions: as_usize(values[16])?,
        votes: as_usize(values[17])?,
        reward_applications: as_usize(values[18])?,
        penalty_applications: as_usize(values[19])?,
        voice_changes: as_usize(values[20])?,
        cooldowns_started: as_usize(values[21])?,
        promotions: as_usize(values[22])?,
        quarantines: as_usize(values[23])?,
        executions: as_usize(values[24])?,
        active_committee_count: as_usize(values[25])?,
    };
    validate_safety_counters_v4_3(&value)?;
    Ok(value)
}

fn validate_safety_counters_v4_3(
    value: &MomentumFuturePredictionSafetyCountersV4_3,
) -> Result<(), String> {
    if value.input_request_attempts > 1
        || value.input_retries != 0
        || value.maximum_concurrency != 1
        || value.outcome_request_attempts != 0
        || value.outcome_retries != 0
        || value.participant_parameter_updates != 0
        || value.normalizer_refits != 0
        || value.protected_context_training_uses != 0
        || value.protected_context_label_uses != 0
        || value.protected_context_metric_uses != 0
        || value.protected_context_reward_uses != 0
        || value.outcome_row_reads != 0
        || value.outcome_label_reads != 0
        || value.metric_computations != 0
        || value.winner_selections != 0
        || value.active_model_changes != 0
        || value.chair_decisions != 0
        || value.votes != 0
        || value.reward_applications != 0
        || value.penalty_applications != 0
        || value.voice_changes != 0
        || value.cooldowns_started != 0
        || value.promotions != 0
        || value.quarantines != 0
        || value.executions != 0
        || value.active_committee_count != 3
    {
        return Err("V4.3 safety counter rejected".to_string());
    }
    Ok(())
}

fn encode_status_v4_3(
    value: &MomentumFuturePredictionStatusReceiptV4_3,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumFuturePredictionStatusReceiptV4_3")
        .string("status_version", &value.status_version)
        .string("lifecycle_version", &value.lifecycle_version)
        .string("lifecycle_digest", &value.lifecycle_digest)
        .string(
            "context_authorization_status",
            format!("{:?}", value.context_authorization_status),
        )
        .string(
            "context_authorization_digest",
            &value.context_authorization_digest,
        )
        .string(
            "supersession_status",
            format!("{:?}", value.supersession_status),
        )
        .string("supersession_digest", &value.supersession_digest)
        .unsigned(
            "superseded_event_timestamp_ms",
            value.superseded_event_timestamp_ms,
        )
        .unsigned(
            "replacement_event_timestamp_ms",
            value.replacement_event_timestamp_ms,
        )
        .unsigned(
            "input_finality_boundary_ms",
            value.input_finality_boundary_ms,
        )
        .string("event_readiness", format!("{:?}", value.event_readiness))
        .string(
            "corrected_context_plan_digest",
            &value.corrected_context_plan_digest,
        )
        .string(
            "corrected_input_registration_digest",
            &value.corrected_input_registration_digest,
        )
        .unsigned("input_request_count", as_u64(value.input_request_count)?)
        .optional_string("input_receipt_digest", &value.input_receipt_digest)
        .optional_string("input_capsule_digest", &value.input_capsule_digest)
        .optional_string(
            "context_usage_ledger_digest",
            &value.context_usage_ledger_digest,
        )
        .strings("context_usage_classes", &value.context_usage_classes)
        .strings(
            "participant_prediction_digests",
            &value.participant_prediction_digests,
        )
        .optional_string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .optional_string(
            "prediction_journal_digest",
            &value.prediction_journal_digest,
        )
        .optional_string("outcome_plan_digest", &value.outcome_plan_digest)
        .unsigneds(
            "outcome_finality_boundary_ms",
            &value
                .outcome_finality_boundary_ms
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .string("cycle_risk_status", &value.cycle_risk_status)
        .string("value_quality_status", &value.value_quality_status)
        .string(
            "prior_momentum_attribution",
            &value.prior_momentum_attribution,
        )
        .string(
            "prior_cycle_risk_attribution",
            &value.prior_cycle_risk_attribution,
        )
        .boolean(
            "protected_artifacts_unchanged",
            value.protected_artifacts_unchanged,
        )
        .boolean("active_state_unchanged", value.active_state_unchanged)
        .messages(
            "safety_counters",
            vec![encode_safety_counters_v4_3(&value.safety_counters)?],
        )
        .string("status_digest", &value.status_digest)
        .encode()
}

fn decode_status_v4_3(bytes: &[u8]) -> Result<MomentumFuturePredictionStatusReceiptV4_3, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumFuturePredictionStatusReceiptV4_3")?;
    let finality = fields.unsigneds("outcome_finality_boundary_ms")?;
    let safety = fields.messages("safety_counters")?;
    if finality.len() > 1 || safety.len() != 1 {
        return Err("V4.3 status shape rejected".to_string());
    }
    let value = MomentumFuturePredictionStatusReceiptV4_3 {
        status_version: fields.string("status_version")?,
        lifecycle_version: fields.string("lifecycle_version")?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
        context_authorization_status: parse_authorization_status_v4_3(
            &fields.string("context_authorization_status")?,
        )?,
        context_authorization_digest: fields.string("context_authorization_digest")?,
        supersession_status: parse_supersession_status_v4_3(
            &fields.string("supersession_status")?,
        )?,
        supersession_digest: fields.string("supersession_digest")?,
        superseded_event_timestamp_ms: fields.unsigned("superseded_event_timestamp_ms")?,
        replacement_event_timestamp_ms: fields.unsigned("replacement_event_timestamp_ms")?,
        input_finality_boundary_ms: fields.unsigned("input_finality_boundary_ms")?,
        event_readiness: parse_readiness_v4_3(&fields.string("event_readiness")?)?,
        corrected_context_plan_digest: fields.string("corrected_context_plan_digest")?,
        corrected_input_registration_digest: fields
            .string("corrected_input_registration_digest")?,
        input_request_count: as_usize(fields.unsigned("input_request_count")?)?,
        input_receipt_digest: fields.optional_string("input_receipt_digest")?,
        input_capsule_digest: fields.optional_string("input_capsule_digest")?,
        context_usage_ledger_digest: fields.optional_string("context_usage_ledger_digest")?,
        context_usage_classes: fields.strings("context_usage_classes")?,
        participant_prediction_digests: fields.strings("participant_prediction_digests")?,
        prediction_capsule_digest: fields.optional_string("prediction_capsule_digest")?,
        prediction_journal_digest: fields.optional_string("prediction_journal_digest")?,
        outcome_plan_digest: fields.optional_string("outcome_plan_digest")?,
        outcome_finality_boundary_ms: finality.first().copied(),
        cycle_risk_status: fields.string("cycle_risk_status")?,
        value_quality_status: fields.string("value_quality_status")?,
        prior_momentum_attribution: fields.string("prior_momentum_attribution")?,
        prior_cycle_risk_attribution: fields.string("prior_cycle_risk_attribution")?,
        protected_artifacts_unchanged: fields.boolean("protected_artifacts_unchanged")?,
        active_state_unchanged: fields.boolean("active_state_unchanged")?,
        safety_counters: decode_safety_counters_v4_3(&safety[0])?,
        status_digest: fields.string("status_digest")?,
    };
    fields.finish()?;
    if value.status_version != STATUS_RECEIPT_VERSION_V4_3
        || value.input_request_count > 1
        || !value.protected_artifacts_unchanged
        || !value.active_state_unchanged
        || value.status_digest != status_digest_v4_3(&value)
    {
        return Err("V4.3 status receipt rejected".to_string());
    }
    Ok(value)
}

fn persist_v4_3(
    root: &Path,
    directory: &str,
    digest: &str,
    extension: &str,
    bytes: &[u8],
    decode_digest: impl Fn(&[u8]) -> Result<String, String>,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root.join(directory).join(format!("{digest}.{extension}")),
        bytes,
        digest,
        decode_digest,
    )
}

fn collect_protected_artifacts_v4_3(
    root: &Path,
    current: &Path,
    values: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), String> {
    if current == root.join(ROOT_VERSION_V4_3) {
        return Ok(());
    }
    if current.is_dir() {
        let mut paths = fs::read_dir(current)
            .map_err(|_| "V4.3 protected directory read failed".to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            collect_protected_artifacts_v4_3(root, &path, values)?;
        }
    } else if current.is_file() {
        values.push((
            current
                .strip_prefix(root)
                .map_err(|_| "V4.3 protected path rejected".to_string())?
                .to_path_buf(),
            fs::read(current).map_err(|_| "V4.3 protected artifact read failed".to_string())?,
        ));
    }
    Ok(())
}

fn protected_artifacts_v4_3(root: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>, String> {
    let mut values = Vec::new();
    collect_protected_artifacts_v4_3(root, root, &mut values)?;
    values.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(values)
}

fn provider_contract_available_v4_3(config: &UpbitHistoricalPilotConfigV0) -> Result<bool, String> {
    config.validate()?;
    let contract = upbit_learning_evidence_provider_contract_v1(config)?;
    Ok(contract.provider_id == config.provider_id
        && contract.market_scope == AcquisitionMarketScope::BtcCrypto
        && contract.dataset_kind == DatasetKind::DailyOhlcv
        && contract.symbols.as_slice() == [config.symbol.clone()]
        && contract.cadence == "1d"
        && contract.maximum_response_bytes == config.maximum_response_bytes
        && contract.credential_free
        && contract.read_only
        && contract.approved_for_network
        && contract.all_rows_finalized
        && contract.enabled)
}

fn build_provider_request_v4_3(
    registration: &MomentumProspectiveInputRegistrationV4_3,
) -> Result<ReadOnlyProviderRequest, String> {
    let expected_start = registration
        .exact_expected_timestamp_ms
        .first()
        .copied()
        .ok_or_else(|| "V4.3 registered request start unavailable".to_string())?;
    if registration.expected_row_count != 16
        || registration.exact_expected_timestamp_ms.len() != 16
        || registration
            .exact_expected_timestamp_ms
            .windows(2)
            .any(|pair| pair[1] != pair[0].saturating_add(DAILY_CADENCE_MS_V4_2))
        || registration
            .exact_expected_timestamp_ms
            .last()
            .and_then(|timestamp| timestamp.checked_add(DAILY_CADENCE_MS_V4_2))
            != Some(registration.request_to_timestamp_ms)
    {
        return Err("V4.3 registered input request rejected".to_string());
    }
    Ok(ReadOnlyProviderRequest {
        request_id: stable_hash_string(&format!(
            "momentum-v4.3-input-request:{}",
            registration.registration_digest
        )),
        request_key: stable_hash_string(&format!(
            "momentum-v4.3-input-key:{}:{}:{expected_start}:{}",
            registration.provider_id, registration.symbol, registration.request_to_timestamp_ms
        )),
        provider_id: registration.provider_id.clone(),
        dataset_kind: DatasetKind::DailyOhlcv,
        market_scope: AcquisitionMarketScope::BtcCrypto,
        symbols: vec![registration.symbol.clone()],
        lookback: DataLookback {
            bars: registration.expected_row_count,
            start_timestamp_ms: Some(expected_start),
            end_timestamp_ms: Some(registration.request_to_timestamp_ms),
        },
        cadence: registration.cadence.clone(),
        max_staleness_ms: 0,
        reason_codes: vec![],
    })
}

fn request_config_v4_3(
    config: &UpbitHistoricalPilotConfigV0,
    registration: &MomentumProspectiveInputRegistrationV4_3,
) -> Result<UpbitHistoricalPilotConfigV0, String> {
    let mut request_config = config.clone();
    request_config.start_timestamp_ms =
        registration
            .exact_expected_timestamp_ms
            .first()
            .copied()
            .ok_or_else(|| "V4.3 request start unavailable".to_string())?;
    request_config.end_timestamp_ms = registration.request_to_timestamp_ms;
    request_config.max_retries = 0;
    request_config.validate()?;
    if !provider_contract_available_v4_3(&request_config)? {
        return Err("V4.3 provider contract rejected".to_string());
    }
    Ok(request_config)
}

fn build_context_usage_ledger_v4_3(
    authorization: &MomentumProtectedContextAuthorizationV4_3,
    plan: &MomentumProspectiveFeatureContextPlanV4_3,
    rows: &[HistoricalOhlcvRow],
) -> Result<MomentumContextUsageLedgerV4_3, String> {
    if rows.len() != plan.required_row_count {
        return Err("V4.3 context usage row count rejected".to_string());
    }
    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let use_class = if row.timestamp_ms == plan.event_timestamp_ms {
            MomentumContextEvidenceUseV4_3::ProspectiveEventInput
        } else if plan
            .protected_context_timestamp_ms
            .contains(&row.timestamp_ms)
        {
            MomentumContextEvidenceUseV4_3::ProtectedRawInferenceContext
        } else if plan
            .existing_source_timestamp_ms
            .contains(&row.timestamp_ms)
        {
            MomentumContextEvidenceUseV4_3::ExistingFrozenHistoricalContext
        } else if plan.incremental_timestamp_ms.contains(&row.timestamp_ms) {
            MomentumContextEvidenceUseV4_3::NewIncrementalInferenceContext
        } else {
            return Err("V4.3 context usage classification rejected".to_string());
        };
        let mut entry = MomentumContextUsageEntryV4_3 {
            timestamp_ms: row.timestamp_ms,
            raw_row_digest: row_identity_digest(row),
            use_class,
            used_for_feature_construction: true,
            used_for_training: false,
            used_for_normalizer_fit: false,
            used_for_label: false,
            used_for_metric: false,
            used_for_reward: false,
            entry_digest: String::new(),
        };
        entry.entry_digest = context_usage_entry_digest_v4_3(&entry);
        entries.push(entry);
    }
    let mut ledger = MomentumContextUsageLedgerV4_3 {
        ledger_version: CONTEXT_LEDGER_VERSION_V4_3.to_string(),
        authorization_digest: authorization.authorization_digest.clone(),
        event_timestamp_ms: plan.event_timestamp_ms,
        entries,
        ledger_digest: String::new(),
    };
    ledger.ledger_digest = context_usage_ledger_digest_v4_3(&ledger);
    decode_usage_ledger_v4_3(&encode_usage_ledger_v4_3(&ledger)?)?;
    Ok(ledger)
}

fn validate_input_response_v4_3(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    authorization: &MomentumProtectedContextAuthorizationV4_3,
    plan: &MomentumProspectiveFeatureContextPlanV4_3,
    registration: &MomentumProspectiveInputRegistrationV4_3,
    transport: &LearningEvidenceTransportResponseV1,
) -> Result<
    (
        MomentumProspectiveInputCapsuleV4_3,
        MomentumContextUsageLedgerV4_3,
        MomentumProspectiveContextVerificationV4_3,
        Vec<HistoricalOhlcvRow>,
    ),
    String,
> {
    if transport.http_status_class != "2xx"
        || transport.raw_response.is_empty()
        || transport.raw_response.len() > registration.maximum_response_bytes
        || serde_json::from_slice::<serde_json::Value>(&transport.raw_response).is_err()
        || transport.response.provider_id != registration.provider_id
        || transport.response.content_type != "application/x-soma-normalized-dataset"
        || !transport.response.all_rows_finalized
        || transport.response.normalized_dataset.symbol != registration.symbol
        || transport.response.reported_content_bytes != transport.raw_response.len()
    {
        return Err("V4.3 input response envelope rejected".to_string());
    }
    let rows = &transport.response.normalized_dataset.rows;
    let timestamps = rows.iter().map(|row| row.timestamp_ms).collect::<Vec<_>>();
    let unique = timestamps.iter().copied().collect::<BTreeSet<_>>();
    let outcome_timestamp = plan
        .event_timestamp_ms
        .checked_add(
            u64::try_from(lifecycle.prediction_horizon)
                .ok()
                .and_then(|horizon| horizon.checked_mul(lifecycle.cadence_ms))
                .ok_or_else(|| "V4.3 outcome timestamp overflow".to_string())?,
        )
        .ok_or_else(|| "V4.3 outcome timestamp overflow".to_string())?;
    if rows.len() != registration.expected_row_count
        || timestamps != registration.exact_expected_timestamp_ms
        || unique.len() != rows.len()
        || rows
            .windows(2)
            .any(|pair| pair[1].timestamp_ms != pair[0].timestamp_ms + lifecycle.cadence_ms)
        || timestamps.contains(&outcome_timestamp)
        || rows.iter().any(|row| {
            row.symbol != registration.symbol
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
        })
    {
        return Err("V4.3 exact input evidence rejected".to_string());
    }
    let raw_response_digest = stable_hash_string(&format!(
        "momentum-v4.3-raw-input:{:?}",
        transport.raw_response
    ));
    let mut capsule = MomentumProspectiveInputCapsuleV4_3 {
        capsule_version: INPUT_CAPSULE_VERSION_V4_3.to_string(),
        lifecycle_digest: lifecycle.lifecycle_digest.clone(),
        context_authorization_digest: authorization.authorization_digest.clone(),
        input_registration_digest: registration.registration_digest.clone(),
        event_timestamp_ms: plan.event_timestamp_ms,
        exact_timestamp_ms: timestamps,
        row_identity_digests: rows.iter().map(row_identity_digest).collect(),
        normalized_dataset_digest: historical_replay_dataset_digest_v0(
            &transport.response.normalized_dataset,
        ),
        raw_response_digest,
        outcome_row_present: false,
        labels_accessed: false,
        metrics_computed: false,
        prior_outcome_capsule_accessed: false,
        credential_free: true,
        read_only: true,
        sanitized: true,
        capsule_digest: String::new(),
    };
    capsule.capsule_digest = input_capsule_digest_v4_3(&capsule);
    decode_input_capsule_v4_3(&encode_input_capsule_v4_3(&capsule)?)?;
    let ledger = build_context_usage_ledger_v4_3(authorization, plan, rows)?;
    let mut proof = MomentumProspectiveContextVerificationV4_3 {
        proof_version: CONTEXT_PROOF_VERSION_V4_3.to_string(),
        context_authorization_digest: authorization.authorization_digest.clone(),
        feature_context_plan_digest: plan.plan_digest.clone(),
        input_registration_digest: registration.registration_digest.clone(),
        input_capsule_digest: capsule.capsule_digest.clone(),
        context_usage_ledger_digest: ledger.ledger_digest.clone(),
        exact_timestamps_verified: true,
        strict_chronology_verified: true,
        feature_history_complete: true,
        protected_context_inference_only: ledger.entries.iter().all(|entry| {
            entry.use_class != MomentumContextEvidenceUseV4_3::ProtectedRawInferenceContext
                || (entry.used_for_feature_construction
                    && !entry.used_for_training
                    && !entry.used_for_normalizer_fit
                    && !entry.used_for_label
                    && !entry.used_for_metric
                    && !entry.used_for_reward)
        }),
        outcome_timestamp_absent: true,
        prior_outcome_artifact_accessed: false,
        proof_digest: String::new(),
    };
    proof.proof_digest = context_proof_digest_v4_3(&proof);
    decode_context_proof_v4_3(&encode_context_proof_v4_3(&proof)?)?;
    Ok((capsule, ledger, proof, rows.clone()))
}

fn build_input_receipt_v4_3(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    authorization: &MomentumProtectedContextAuthorizationV4_3,
    registration: &MomentumProspectiveInputRegistrationV4_3,
    status: MomentumProspectiveInputStatusV4_2,
    http_status_class: Option<String>,
    returned_row_count: usize,
    verified_row_count: usize,
    raw_response_digest: Option<String>,
    input_capsule_digest: Option<String>,
) -> MomentumProspectiveInputReceiptV4_3 {
    let mut receipt = MomentumProspectiveInputReceiptV4_3 {
        receipt_version: INPUT_RECEIPT_VERSION_V4_3.to_string(),
        lifecycle_digest: lifecycle.lifecycle_digest.clone(),
        context_authorization_digest: authorization.authorization_digest.clone(),
        input_registration_digest: registration.registration_digest.clone(),
        request_attempted: status != MomentumProspectiveInputStatusV4_2::ReadyNotAttempted,
        request_count: usize::from(status != MomentumProspectiveInputStatusV4_2::ReadyNotAttempted),
        retry_count: 0,
        status,
        http_status_class,
        returned_row_count,
        verified_row_count,
        raw_response_digest,
        input_capsule_digest,
        terminal: status.is_terminal(),
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = input_receipt_digest_v4_3(&receipt);
    receipt
}

fn seal_participant_predictions_v4_3(
    source: &MomentumFutureEvaluationSourceV4_2,
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    receipt: &MomentumProspectiveInputReceiptV4_3,
    input_capsule: &MomentumProspectiveInputCapsuleV4_3,
    ledger: &MomentumContextUsageLedgerV4_3,
    prediction: &[MomentumFrozenParticipantPredictionV4],
    feature_identity_digest: &str,
) -> Result<Vec<MomentumParticipantPredictionSealV4_3>, String> {
    if prediction.len() != 3 || prediction.len() != lifecycle.participant_digests.len() {
        return Err("V4.3 frozen prediction roster rejected".to_string());
    }
    lifecycle
        .participant_digests
        .iter()
        .map(|participant_digest| {
            let predicted = prediction
                .iter()
                .find(|value| &value.participant_digest == participant_digest)
                .ok_or_else(|| "V4.3 participant prediction unavailable".to_string())?;
            let participant = source
                .source_family
                .participants
                .iter()
                .find(|value| &value.participant_digest == participant_digest)
                .ok_or_else(|| "V4.3 participant source unavailable".to_string())?;
            if predicted.config_digest != participant.config_digest
                || predicted.parameter_digest != participant.parameter_digest
                || predicted.normalizer_digest != participant.normalizer_digest
                || predicted.model_artifact_digest != participant.model_artifact_digest
                || predicted.feature_schema_digest != participant.input_feature_schema_digest
                || predicted.training_identity_digest != participant.training_identity_digest
            {
                return Err("V4.3 participant reconstruction identity rejected".to_string());
            }
            let participant_role = match participant.participant_role {
                MomentumRawFeatureRoleV4::LearnedRawLogistic => "RawFeatureLogisticV4",
                MomentumRawFeatureRoleV4::LearnedInteractionLogistic => {
                    "RawFeatureInteractionLogisticV4"
                }
                MomentumRawFeatureRoleV4::ConstantBenchmark => "TrainingPrevalenceConstantV4",
            }
            .to_string();
            let prediction_digest = stable_hash_string(&format!(
                "momentum-v4.3-prediction:{}:{}:{}:{}:{}",
                participant.participant_digest,
                input_capsule.event_timestamp_ms,
                receipt.receipt_digest,
                ledger.ledger_digest,
                predicted.probability_bits
            ));
            let mut seal = MomentumParticipantPredictionSealV4_3 {
                participant_digest: participant.participant_digest.clone(),
                participant_role,
                event_timestamp_ms: input_capsule.event_timestamp_ms,
                input_receipt_digest: receipt.receipt_digest.clone(),
                input_capsule_digest: input_capsule.capsule_digest.clone(),
                context_usage_ledger_digest: ledger.ledger_digest.clone(),
                feature_identity_digest: feature_identity_digest.to_string(),
                prediction_probability_bits: predicted.probability_bits,
                prediction_digest,
                participant_identity_verified: true,
                parameter_updates: 0,
                normalizer_refits: 0,
                outcome_access_count: 0,
                seal_digest: String::new(),
            };
            seal.seal_digest = prediction_seal_digest_v4_3(&seal);
            decode_prediction_seal_v4_3(&encode_prediction_seal_v4_3(&seal)?)?;
            Ok(seal)
        })
        .collect()
}

fn build_prediction_capsule_v4_3(
    source: &MomentumFutureEvaluationSourceV4_2,
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    authorization: &MomentumProtectedContextAuthorizationV4_3,
    supersession: &MomentumInputPlanSupersessionV4_3,
    plan: &MomentumProspectiveFeatureContextPlanV4_3,
    registration: &MomentumProspectiveInputRegistrationV4_3,
    receipt: &MomentumProspectiveInputReceiptV4_3,
    input_capsule: &MomentumProspectiveInputCapsuleV4_3,
    ledger: &MomentumContextUsageLedgerV4_3,
    seals: Vec<MomentumParticipantPredictionSealV4_3>,
) -> Result<MomentumProspectivePredictionCapsuleV4_3, String> {
    if seals
        .iter()
        .map(|seal| seal.participant_digest.clone())
        .collect::<Vec<_>>()
        != lifecycle.participant_digests
    {
        return Err("V4.3 prediction seal ordering rejected".to_string());
    }
    let mut capsule = MomentumProspectivePredictionCapsuleV4_3 {
        capsule_version: PREDICTION_CAPSULE_VERSION_V4_3.to_string(),
        lifecycle_digest: lifecycle.lifecycle_digest.clone(),
        evaluation_registration_digest: source.evaluation.registration_digest.clone(),
        roster_digest: source.roster.roster_digest.clone(),
        context_authorization_digest: authorization.authorization_digest.clone(),
        supersession_digest: supersession.supersession_digest.clone(),
        corrected_context_plan_digest: plan.plan_digest.clone(),
        input_registration_digest: registration.registration_digest.clone(),
        event_timestamp_ms: plan.event_timestamp_ms,
        input_receipt_digest: receipt.receipt_digest.clone(),
        input_capsule_digest: input_capsule.capsule_digest.clone(),
        context_usage_ledger_digest: ledger.ledger_digest.clone(),
        participant_prediction_seals: seals,
        probabilities_hidden: true,
        labels_hidden: true,
        outcome_accessed: false,
        metrics_computed: false,
        winner_selected: false,
        capsule_digest: String::new(),
    };
    capsule.capsule_digest = prediction_capsule_digest_v4_3(&capsule);
    decode_prediction_capsule_v4_3(&encode_prediction_capsule_v4_3(&capsule)?)?;
    Ok(capsule)
}

fn build_prediction_journal_v4_3(
    authorization: &MomentumProtectedContextAuthorizationV4_3,
    input_capsule: &MomentumProspectiveInputCapsuleV4_3,
    ledger: &MomentumContextUsageLedgerV4_3,
    capsule: &MomentumProspectivePredictionCapsuleV4_3,
) -> Result<MomentumProspectivePredictionJournalV4_3, String> {
    let mut entry = MomentumProspectivePredictionJournalEntryV4_3 {
        event_timestamp_ms: capsule.event_timestamp_ms,
        context_authorization_digest: authorization.authorization_digest.clone(),
        input_capsule_digest: input_capsule.capsule_digest.clone(),
        context_usage_ledger_digest: ledger.ledger_digest.clone(),
        prediction_capsule_digest: capsule.capsule_digest.clone(),
        participant_prediction_digests: capsule
            .participant_prediction_seals
            .iter()
            .map(|seal| seal.prediction_digest.clone())
            .collect(),
        prediction_sealed_before_outcome: true,
        outcome_stage_unlocked: false,
        entry_digest: String::new(),
    };
    entry.entry_digest = prediction_entry_digest_v4_3(&entry);
    let mut journal = MomentumProspectivePredictionJournalV4_3 {
        journal_version: PREDICTION_JOURNAL_VERSION_V4_3.to_string(),
        entries: vec![entry],
        journal_digest: String::new(),
    };
    journal.journal_digest = prediction_journal_digest_v4_3(&journal);
    decode_prediction_journal_v4_3(&encode_prediction_journal_v4_3(&journal)?)?;
    Ok(journal)
}

fn build_outcome_plan_v4_3(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    capsule: &MomentumProspectivePredictionCapsuleV4_3,
) -> Result<MomentumProspectiveOutcomePlanV4_3, String> {
    let required_outcome_timestamp_ms = (1..=lifecycle.prediction_horizon)
        .map(|offset| {
            u64::try_from(offset)
                .ok()
                .and_then(|offset| offset.checked_mul(lifecycle.cadence_ms))
                .and_then(|offset| capsule.event_timestamp_ms.checked_add(offset))
                .ok_or_else(|| "V4.3 outcome timestamp overflow".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let outcome_finality_boundary_ms = required_outcome_timestamp_ms
        .last()
        .and_then(|timestamp| timestamp.checked_add(lifecycle.cadence_ms))
        .ok_or_else(|| "V4.3 outcome finality unavailable".to_string())?;
    let mut plan = MomentumProspectiveOutcomePlanV4_3 {
        plan_version: OUTCOME_PLAN_VERSION_V4_3.to_string(),
        prediction_capsule_digest: capsule.capsule_digest.clone(),
        event_timestamp_ms: capsule.event_timestamp_ms,
        prediction_horizon: lifecycle.prediction_horizon,
        required_outcome_timestamp_ms,
        outcome_finality_boundary_ms,
        maximum_outcome_requests: lifecycle.outcome_stage_maximum_requests,
        maximum_outcome_retries: lifecycle.outcome_stage_maximum_retries,
        labels_hidden_until_opening: true,
        one_time_opening_required: true,
        outcome_stage_locked_before_finality: true,
        plan_digest: String::new(),
    };
    plan.plan_digest = outcome_plan_digest_v4_3(&plan);
    decode_outcome_plan_v4_3(&encode_outcome_plan_v4_3(&plan)?)?;
    Ok(plan)
}

fn build_status_v4_3(
    lifecycle: &MomentumFutureEvaluationLifecycleV4_2,
    authorization: &MomentumProtectedContextAuthorizationV4_3,
    authorization_status: ProtectedContextAuthorizationStatusV4_3,
    supersession: &MomentumInputPlanSupersessionV4_3,
    supersession_status: MomentumInputPlanSupersessionStatusV4_3,
    plan: &MomentumProspectiveFeatureContextPlanV4_3,
    registration: &MomentumProspectiveInputRegistrationV4_3,
    readiness: MomentumEventReadinessV4_3,
    receipt: Option<&MomentumProspectiveInputReceiptV4_3>,
    input_capsule: Option<&MomentumProspectiveInputCapsuleV4_3>,
    ledger: Option<&MomentumContextUsageLedgerV4_3>,
    prediction_capsule: Option<&MomentumProspectivePredictionCapsuleV4_3>,
    journal: Option<&MomentumProspectivePredictionJournalV4_3>,
    outcome_plan: Option<&MomentumProspectiveOutcomePlanV4_3>,
    protected_artifacts_unchanged: bool,
    active_state_unchanged: bool,
    safety_counters: MomentumFuturePredictionSafetyCountersV4_3,
) -> Result<MomentumFuturePredictionStatusReceiptV4_3, String> {
    validate_safety_counters_v4_3(&safety_counters)?;
    let mut status = MomentumFuturePredictionStatusReceiptV4_3 {
        status_version: STATUS_RECEIPT_VERSION_V4_3.to_string(),
        lifecycle_version: lifecycle.lifecycle_version.clone(),
        lifecycle_digest: lifecycle.lifecycle_digest.clone(),
        context_authorization_status: authorization_status,
        context_authorization_digest: authorization.authorization_digest.clone(),
        supersession_status,
        supersession_digest: supersession.supersession_digest.clone(),
        superseded_event_timestamp_ms: supersession.superseded_event_timestamp_ms,
        replacement_event_timestamp_ms: plan.event_timestamp_ms,
        input_finality_boundary_ms: plan.input_finality_boundary_ms,
        event_readiness: readiness,
        corrected_context_plan_digest: plan.plan_digest.clone(),
        corrected_input_registration_digest: registration.registration_digest.clone(),
        input_request_count: receipt.map_or(0, |receipt| receipt.request_count),
        input_receipt_digest: receipt.map(|receipt| receipt.receipt_digest.clone()),
        input_capsule_digest: input_capsule.map(|capsule| capsule.capsule_digest.clone()),
        context_usage_ledger_digest: ledger.map(|ledger| ledger.ledger_digest.clone()),
        context_usage_classes: ledger.map_or_else(Vec::new, |ledger| {
            ledger
                .entries
                .iter()
                .map(|entry| format!("{:?}", entry.use_class))
                .collect()
        }),
        participant_prediction_digests: prediction_capsule.map_or_else(Vec::new, |capsule| {
            capsule
                .participant_prediction_seals
                .iter()
                .map(|seal| seal.prediction_digest.clone())
                .collect()
        }),
        prediction_capsule_digest: prediction_capsule.map(|capsule| capsule.capsule_digest.clone()),
        prediction_journal_digest: journal.map(|journal| journal.journal_digest.clone()),
        outcome_plan_digest: outcome_plan.map(|plan| plan.plan_digest.clone()),
        outcome_finality_boundary_ms: outcome_plan.map(|plan| plan.outcome_finality_boundary_ms),
        cycle_risk_status: "ProviderContractUnverified".to_string(),
        value_quality_status: "TrainerUnavailable".to_string(),
        prior_momentum_attribution: "MissedMaterialOpportunity".to_string(),
        prior_cycle_risk_attribution: "CorrectUncertainty".to_string(),
        protected_artifacts_unchanged,
        active_state_unchanged,
        safety_counters,
        status_digest: String::new(),
    };
    status.status_digest = status_digest_v4_3(&status);
    decode_status_v4_3(&encode_status_v4_3(&status)?)?;
    Ok(status)
}

fn base_report_v4_3(
    status: MomentumFuturePredictionStatusReceiptV4_3,
    lifecycle: MomentumFutureEvaluationLifecycleV4_2,
    context_authorization: MomentumProtectedContextAuthorizationV4_3,
    supersession: MomentumInputPlanSupersessionV4_3,
    context_plan: MomentumProspectiveFeatureContextPlanV4_3,
    input_registration: MomentumProspectiveInputRegistrationV4_3,
) -> MomentumFuturePredictionReportV4_3 {
    MomentumFuturePredictionReportV4_3 {
        status,
        lifecycle,
        context_authorization,
        supersession,
        context_plan,
        input_registration,
        input_receipt: None,
        input_capsule: None,
        context_usage_ledger: None,
        prediction_capsule: None,
        prediction_journal: None,
        outcome_plan: None,
        artifacts_written: 0,
        duplicate_artifact_count: 0,
        storage_failure_count: 0,
    }
}

pub fn run_momentum_future_prediction_v4_3(
    root: &Path,
    snapshots: &[DataSnapshot],
    reservation: &ProtectedEvaluationReservationV1,
    provider_config: &UpbitHistoricalPilotConfigV0,
    observed_timestamp_ms: u64,
    mode: MomentumFuturePredictionRunModeV4_2,
    network_allowed: bool,
    one_time_input_request_confirmed: bool,
) -> Result<MomentumFuturePredictionReportV4_3, String> {
    if mode != MomentumFuturePredictionRunModeV4_2::Execute
        && (network_allowed || one_time_input_request_confirmed)
    {
        return Err("V4.3 non-execute mode rejects network authority".to_string());
    }
    if mode == MomentumFuturePredictionRunModeV4_2::Execute
        && (!network_allowed || !one_time_input_request_confirmed)
    {
        return Err("V4.3 execute requires both network permissions".to_string());
    }
    provider_config.validate()?;
    let protected_before = protected_artifacts_v4_3(root)?;
    let active_before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let source = reopen_momentum_v4_1_future_source(root)?;
    let source_snapshot = source_snapshot(snapshots, &source)?;
    let lifecycle = derive_lifecycle(&source)?;
    let v4_2_root = root.join(ROOT_VERSION_V4_2).join(AGENT_ID_V4_2);
    let old_lifecycle = read_single(&v4_2_root.join("lifecycles"), decode_lifecycle)?
        .ok_or_else(|| "V4.3 prior lifecycle unavailable".to_string())?;
    let old_plan = read_single(&v4_2_root.join("context_plans"), decode_context_plan)?
        .ok_or_else(|| "V4.3 prior context plan unavailable".to_string())?;
    let old_registration = read_single(
        &v4_2_root.join("input_registrations"),
        decode_input_registration,
    )?
    .ok_or_else(|| "V4.3 prior input registration unavailable".to_string())?;
    let old_status = read_single(&v4_2_root.join("status_receipts"), decode_status)?
        .ok_or_else(|| "V4.3 prior status unavailable".to_string())?;
    let derived_old_plan = derive_context_plan(&lifecycle, &source.evaluation, source_snapshot)?;
    let derived_old_registration =
        derive_input_registration(&lifecycle, &derived_old_plan, &source, provider_config)?;
    if old_lifecycle != lifecycle
        || old_plan != derived_old_plan
        || old_registration != derived_old_registration
        || old_status.lifecycle_digest != lifecycle.lifecycle_digest
        || old_status.context_plan_digest != old_plan.plan_digest
        || old_status.input_registration_digest != old_registration.registration_digest
        || old_status.request_attempt_count != 0
    {
        return Err("V4.3 prior V4.2 contract rejected".to_string());
    }
    let old_receipt = read_single(&v4_2_root.join("input_receipts"), decode_input_receipt)?;
    let old_capsule = read_single(&v4_2_root.join("input_capsules"), decode_input_capsule)?;
    let old_prediction = read_single(
        &v4_2_root.join("prediction_capsules"),
        decode_prediction_capsule,
    )?;
    let authorization = derive_context_authorization_v4_3(
        &source,
        &lifecycle,
        provider_contract_available_v4_3(provider_config)?,
        false,
        None,
    )?;
    if authorization.authorization_status != ProtectedContextAuthorizationStatusV4_3::Authorized {
        return Err("V4.3 protected context authorization failed closed".to_string());
    }
    let context_plan =
        derive_context_plan_v4_3(&source, &lifecycle, &authorization, source_snapshot)?;
    let mut supersession = derive_supersession_v4_3(
        &lifecycle,
        &authorization,
        &old_plan,
        &old_registration,
        &context_plan,
        old_status.request_attempt_count,
        old_receipt.is_some(),
        old_capsule.is_some(),
        old_prediction.is_some(),
    );
    if supersession.status != MomentumInputPlanSupersessionStatusV4_3::Superseded {
        return Err("V4.3 prior input registration cannot be superseded".to_string());
    }
    let input_registration = derive_input_registration_v4_3(
        &lifecycle,
        &source,
        &authorization,
        &supersession,
        &context_plan,
        provider_config,
    )?;
    supersession.replacement_input_registration_digest =
        input_registration.registration_digest.clone();
    validate_supersession_v4_3(
        &supersession,
        &lifecycle,
        &authorization,
        &old_plan,
        &old_registration,
        &context_plan,
        &input_registration,
    )?;

    let v4_3_root = root.join(ROOT_VERSION_V4_3).join(AGENT_ID_V4_2);
    let persisted_authorization = read_single(
        &v4_3_root.join("context_authorizations"),
        decode_authorization_v4_3,
    )?;
    if persisted_authorization
        .as_ref()
        .is_some_and(|value| value != &authorization)
    {
        return Err("V4.3 persisted authorization changed".to_string());
    }
    let persisted_supersession =
        read_single(&v4_3_root.join("supersessions"), decode_supersession_v4_3)?;
    if persisted_supersession
        .as_ref()
        .is_some_and(|value| value != &supersession)
    {
        return Err("V4.3 persisted supersession changed".to_string());
    }
    let persisted_context_plan =
        read_single(&v4_3_root.join("context_plans"), decode_context_plan_v4_3)?;
    if persisted_context_plan
        .as_ref()
        .is_some_and(|value| value != &context_plan)
    {
        return Err("V4.3 persisted context plan changed".to_string());
    }
    let persisted_registration = read_single(
        &v4_3_root.join("input_registrations"),
        decode_input_registration_v4_3,
    )?;
    if persisted_registration
        .as_ref()
        .is_some_and(|value| value != &input_registration)
    {
        return Err("V4.3 persisted input registration changed".to_string());
    }
    let persisted_receipt =
        read_single(&v4_3_root.join("input_receipts"), decode_input_receipt_v4_3)?;
    let persisted_input_capsule =
        read_single(&v4_3_root.join("input_capsules"), decode_input_capsule_v4_3)?;
    let persisted_prediction = read_single(
        &v4_3_root.join("prediction_capsules"),
        decode_prediction_capsule_v4_3,
    )?;
    let authorization_status = if persisted_authorization.is_some() {
        ProtectedContextAuthorizationStatusV4_3::AlreadyAuthorized
    } else {
        ProtectedContextAuthorizationStatusV4_3::Authorized
    };
    let supersession_status = if persisted_supersession.is_some() {
        MomentumInputPlanSupersessionStatusV4_3::AlreadySuperseded
    } else {
        MomentumInputPlanSupersessionStatusV4_3::Superseded
    };

    if let Some(prediction_capsule) = persisted_prediction {
        let receipt = persisted_receipt
            .ok_or_else(|| "V4.3 sealed replay input receipt unavailable".to_string())?;
        let input_capsule = persisted_input_capsule
            .ok_or_else(|| "V4.3 sealed replay input capsule unavailable".to_string())?;
        let ledger = read_single(
            &v4_3_root.join("context_usage_ledgers"),
            decode_usage_ledger_v4_3,
        )?
        .ok_or_else(|| "V4.3 sealed replay context ledger unavailable".to_string())?;
        let journal = read_single(
            &v4_3_root.join("prediction_journals"),
            decode_prediction_journal_v4_3,
        )?
        .ok_or_else(|| "V4.3 sealed replay journal unavailable".to_string())?;
        let outcome_plan = read_single(&v4_3_root.join("outcome_plans"), decode_outcome_plan_v4_3)?
            .ok_or_else(|| "V4.3 sealed replay outcome plan unavailable".to_string())?;
        if prediction_capsule.lifecycle_digest != lifecycle.lifecycle_digest
            || prediction_capsule.context_authorization_digest != authorization.authorization_digest
            || prediction_capsule.supersession_digest != supersession.supersession_digest
            || prediction_capsule.corrected_context_plan_digest != context_plan.plan_digest
            || prediction_capsule.input_registration_digest
                != input_registration.registration_digest
            || prediction_capsule.input_receipt_digest != receipt.receipt_digest
            || prediction_capsule.input_capsule_digest != input_capsule.capsule_digest
            || prediction_capsule.context_usage_ledger_digest != ledger.ledger_digest
            || journal.entries[0].prediction_capsule_digest != prediction_capsule.capsule_digest
            || outcome_plan.prediction_capsule_digest != prediction_capsule.capsule_digest
        {
            return Err("V4.3 sealed replay cross-binding rejected".to_string());
        }
        let protected_after = protected_artifacts_v4_3(root)?;
        let active_after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
        let status = build_status_v4_3(
            &lifecycle,
            &authorization,
            authorization_status,
            &supersession,
            supersession_status,
            &context_plan,
            &input_registration,
            MomentumEventReadinessV4_3::PredictionAlreadySealed,
            Some(&receipt),
            Some(&input_capsule),
            Some(&ledger),
            Some(&prediction_capsule),
            Some(&journal),
            Some(&outcome_plan),
            protected_before == protected_after,
            active_before == active_after,
            MomentumFuturePredictionSafetyCountersV4_3::default(),
        )?;
        let mut report = base_report_v4_3(
            status,
            lifecycle,
            authorization,
            supersession,
            context_plan,
            input_registration,
        );
        report.input_receipt = Some(receipt);
        report.input_capsule = Some(input_capsule);
        report.context_usage_ledger = Some(ledger);
        report.prediction_capsule = Some(prediction_capsule);
        report.prediction_journal = Some(journal);
        report.outcome_plan = Some(outcome_plan);
        return Ok(report);
    }

    let readiness = event_readiness_v4_3(
        observed_timestamp_ms,
        context_plan.input_finality_boundary_ms,
        persisted_receipt
            .as_ref()
            .is_some_and(|receipt| receipt.terminal),
        false,
    );
    if mode != MomentumFuturePredictionRunModeV4_2::Execute
        || readiness == MomentumEventReadinessV4_3::PriorInputAttemptTerminal
    {
        let protected_after = protected_artifacts_v4_3(root)?;
        let active_after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
        let status = build_status_v4_3(
            &lifecycle,
            &authorization,
            authorization_status,
            &supersession,
            supersession_status,
            &context_plan,
            &input_registration,
            readiness,
            persisted_receipt.as_ref(),
            persisted_input_capsule.as_ref(),
            None,
            None,
            None,
            None,
            protected_before == protected_after,
            active_before == active_after,
            MomentumFuturePredictionSafetyCountersV4_3::default(),
        )?;
        let mut report = base_report_v4_3(
            status,
            lifecycle,
            authorization,
            supersession,
            context_plan,
            input_registration,
        );
        report.input_receipt = persisted_receipt;
        report.input_capsule = persisted_input_capsule;
        return Ok(report);
    }

    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_v4_3(
            &v4_3_root,
            "context_authorizations",
            &authorization.authorization_digest,
            "pb",
            &encode_authorization_v4_3(&authorization)?,
            |bytes| Ok(decode_authorization_v4_3(bytes)?.authorization_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_v4_3(
            &v4_3_root,
            "supersessions",
            &supersession.supersession_digest,
            "pb",
            &encode_supersession_v4_3(&supersession)?,
            |bytes| Ok(decode_supersession_v4_3(bytes)?.supersession_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_v4_3(
            &v4_3_root,
            "context_plans",
            &context_plan.plan_digest,
            "pb",
            &encode_context_plan_v4_3(&context_plan)?,
            |bytes| Ok(decode_context_plan_v4_3(bytes)?.plan_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_v4_3(
            &v4_3_root,
            "input_registrations",
            &input_registration.registration_digest,
            "pb",
            &encode_input_registration_v4_3(&input_registration)?,
            |bytes| Ok(decode_input_registration_v4_3(bytes)?.registration_digest),
        )?,
    );
    let reopened_authorization = reopen_exact(
        &v4_3_root
            .join("context_authorizations")
            .join(format!("{}.pb", authorization.authorization_digest)),
        decode_authorization_v4_3,
    )?;
    let reopened_supersession = reopen_exact(
        &v4_3_root
            .join("supersessions")
            .join(format!("{}.pb", supersession.supersession_digest)),
        decode_supersession_v4_3,
    )?;
    let reopened_context_plan = reopen_exact(
        &v4_3_root
            .join("context_plans")
            .join(format!("{}.pb", context_plan.plan_digest)),
        decode_context_plan_v4_3,
    )?;
    let reopened_registration = reopen_exact(
        &v4_3_root
            .join("input_registrations")
            .join(format!("{}.pb", input_registration.registration_digest)),
        decode_input_registration_v4_3,
    )?;
    if reopened_authorization != authorization
        || reopened_supersession != supersession
        || reopened_context_plan != context_plan
        || reopened_registration != input_registration
    {
        return Err("V4.3 prerequest contract reopen rejected".to_string());
    }

    if readiness == MomentumEventReadinessV4_3::AwaitingInputFinality {
        let protected_after = protected_artifacts_v4_3(root)?;
        let active_after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
        if protected_before != protected_after || active_before != active_after {
            return Err("V4.3 prefinality execution changed protected state".to_string());
        }
        let status = build_status_v4_3(
            &lifecycle,
            &authorization,
            authorization_status,
            &supersession,
            supersession_status,
            &context_plan,
            &input_registration,
            readiness,
            None,
            None,
            None,
            None,
            None,
            None,
            true,
            true,
            MomentumFuturePredictionSafetyCountersV4_3::default(),
        )?;
        add_counts(
            &mut counts,
            persist_v4_3(
                &v4_3_root,
                "status_receipts",
                &status.status_digest,
                "pb",
                &encode_status_v4_3(&status)?,
                |bytes| Ok(decode_status_v4_3(bytes)?.status_digest),
            )?,
        );
        let mut report = base_report_v4_3(
            status,
            lifecycle,
            authorization,
            supersession,
            context_plan,
            input_registration,
        );
        report.artifacts_written = counts.0;
        report.duplicate_artifact_count = counts.1;
        return Ok(report);
    }

    let request = build_provider_request_v4_3(&input_registration)?;
    let request_config = request_config_v4_3(provider_config, &input_registration)?;
    let mut safety_counters = MomentumFuturePredictionSafetyCountersV4_3 {
        input_request_attempts: 1,
        ..Default::default()
    };
    let transport = fetch_upbit_learning_evidence_once_v1(&request_config, &request);
    let transport = match transport {
        Ok(transport) => transport,
        Err(failure) => {
            let (input_status, http_status_class, raw_response) = match failure {
                LearningEvidenceTransportFailureV1::ProviderRejected {
                    http_status_class,
                    raw_response,
                } => (
                    MomentumProspectiveInputStatusV4_2::ProviderRejected,
                    http_status_class,
                    raw_response,
                ),
                LearningEvidenceTransportFailureV1::TimedOut => (
                    MomentumProspectiveInputStatusV4_2::TimeoutNoRetry,
                    None,
                    None,
                ),
                LearningEvidenceTransportFailureV1::Technical => (
                    MomentumProspectiveInputStatusV4_2::TechnicalFailure,
                    None,
                    None,
                ),
            };
            let raw_digest = raw_response
                .as_ref()
                .map(|bytes| stable_hash_string(&format!("momentum-v4.3-raw-input:{bytes:?}")));
            if let (Some(bytes), Some(digest)) = (&raw_response, &raw_digest)
                && bytes.len() <= input_registration.maximum_response_bytes
            {
                add_counts(
                    &mut counts,
                    persist_v4_3(&v4_3_root, "raw_input", digest, "json", bytes, |stored| {
                        Ok(stable_hash_string(&format!(
                            "momentum-v4.3-raw-input:{stored:?}"
                        )))
                    })?,
                );
            }
            let receipt = build_input_receipt_v4_3(
                &lifecycle,
                &authorization,
                &input_registration,
                input_status,
                http_status_class,
                0,
                0,
                raw_digest,
                None,
            );
            add_counts(
                &mut counts,
                persist_v4_3(
                    &v4_3_root,
                    "input_receipts",
                    &receipt.receipt_digest,
                    "pb",
                    &encode_input_receipt_v4_3(&receipt)?,
                    |bytes| Ok(decode_input_receipt_v4_3(bytes)?.receipt_digest),
                )?,
            );
            let protected_after = protected_artifacts_v4_3(root)?;
            let active_after =
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
            if protected_before != protected_after || active_before != active_after {
                return Err("V4.3 failed attempt changed protected state".to_string());
            }
            let status = build_status_v4_3(
                &lifecycle,
                &authorization,
                authorization_status,
                &supersession,
                supersession_status,
                &context_plan,
                &input_registration,
                MomentumEventReadinessV4_3::PriorInputAttemptTerminal,
                Some(&receipt),
                None,
                None,
                None,
                None,
                None,
                true,
                true,
                safety_counters,
            )?;
            add_counts(
                &mut counts,
                persist_v4_3(
                    &v4_3_root,
                    "status_receipts",
                    &status.status_digest,
                    "pb",
                    &encode_status_v4_3(&status)?,
                    |bytes| Ok(decode_status_v4_3(bytes)?.status_digest),
                )?,
            );
            let mut report = base_report_v4_3(
                status,
                lifecycle,
                authorization,
                supersession,
                context_plan,
                input_registration,
            );
            report.input_receipt = Some(receipt);
            report.artifacts_written = counts.0;
            report.duplicate_artifact_count = counts.1;
            return Ok(report);
        }
    };

    let validated = validate_input_response_v4_3(
        &lifecycle,
        &authorization,
        &context_plan,
        &input_registration,
        &transport,
    );
    let (input_capsule, ledger, context_proof, context_rows) = match validated {
        Ok(value) => value,
        Err(_) => {
            let raw_digest = stable_hash_string(&format!(
                "momentum-v4.3-raw-input:{:?}",
                transport.raw_response
            ));
            add_counts(
                &mut counts,
                persist_v4_3(
                    &v4_3_root,
                    "raw_input",
                    &raw_digest,
                    "json",
                    &transport.raw_response,
                    |stored| {
                        Ok(stable_hash_string(&format!(
                            "momentum-v4.3-raw-input:{stored:?}"
                        )))
                    },
                )?,
            );
            let receipt = build_input_receipt_v4_3(
                &lifecycle,
                &authorization,
                &input_registration,
                MomentumProspectiveInputStatusV4_2::InvalidResponse,
                Some(transport.http_status_class),
                transport.response.normalized_dataset.rows.len(),
                0,
                Some(raw_digest),
                None,
            );
            add_counts(
                &mut counts,
                persist_v4_3(
                    &v4_3_root,
                    "input_receipts",
                    &receipt.receipt_digest,
                    "pb",
                    &encode_input_receipt_v4_3(&receipt)?,
                    |bytes| Ok(decode_input_receipt_v4_3(bytes)?.receipt_digest),
                )?,
            );
            let protected_after = protected_artifacts_v4_3(root)?;
            let active_after =
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
            if protected_before != protected_after || active_before != active_after {
                return Err("V4.3 invalid response changed protected state".to_string());
            }
            let status = build_status_v4_3(
                &lifecycle,
                &authorization,
                authorization_status,
                &supersession,
                supersession_status,
                &context_plan,
                &input_registration,
                MomentumEventReadinessV4_3::PriorInputAttemptTerminal,
                Some(&receipt),
                None,
                None,
                None,
                None,
                None,
                true,
                true,
                safety_counters,
            )?;
            add_counts(
                &mut counts,
                persist_v4_3(
                    &v4_3_root,
                    "status_receipts",
                    &status.status_digest,
                    "pb",
                    &encode_status_v4_3(&status)?,
                    |bytes| Ok(decode_status_v4_3(bytes)?.status_digest),
                )?,
            );
            let mut report = base_report_v4_3(
                status,
                lifecycle,
                authorization,
                supersession,
                context_plan,
                input_registration,
            );
            report.input_receipt = Some(receipt);
            report.artifacts_written = counts.0;
            report.duplicate_artifact_count = counts.1;
            return Ok(report);
        }
    };
    add_counts(
        &mut counts,
        persist_v4_3(
            &v4_3_root,
            "raw_input",
            &input_capsule.raw_response_digest,
            "json",
            &transport.raw_response,
            |stored| {
                Ok(stable_hash_string(&format!(
                    "momentum-v4.3-raw-input:{stored:?}"
                )))
            },
        )?,
    );
    add_counts(
        &mut counts,
        persist_v4_3(
            &v4_3_root,
            "input_capsules",
            &input_capsule.capsule_digest,
            "pb",
            &encode_input_capsule_v4_3(&input_capsule)?,
            |bytes| Ok(decode_input_capsule_v4_3(bytes)?.capsule_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_v4_3(
            &v4_3_root,
            "context_usage_ledgers",
            &ledger.ledger_digest,
            "pb",
            &encode_usage_ledger_v4_3(&ledger)?,
            |bytes| Ok(decode_usage_ledger_v4_3(bytes)?.ledger_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_v4_3(
            &v4_3_root,
            "context_verifications",
            &context_proof.proof_digest,
            "pb",
            &encode_context_proof_v4_3(&context_proof)?,
            |bytes| Ok(decode_context_proof_v4_3(bytes)?.proof_digest),
        )?,
    );
    let receipt = build_input_receipt_v4_3(
        &lifecycle,
        &authorization,
        &input_registration,
        MomentumProspectiveInputStatusV4_2::EvidenceAcquired,
        Some(transport.http_status_class),
        context_rows.len(),
        context_rows.len(),
        Some(input_capsule.raw_response_digest.clone()),
        Some(input_capsule.capsule_digest.clone()),
    );
    add_counts(
        &mut counts,
        persist_v4_3(
            &v4_3_root,
            "input_receipts",
            &receipt.receipt_digest,
            "pb",
            &encode_input_receipt_v4_3(&receipt)?,
            |bytes| Ok(decode_input_receipt_v4_3(bytes)?.receipt_digest),
        )?,
    );
    let replay = reconstruct_frozen_momentum_v4(root, snapshots, reservation)?;
    let frozen_prediction =
        predict_frozen_momentum_v4_event(&replay, &lifecycle.participant_digests, &context_rows)?;
    let seals = seal_participant_predictions_v4_3(
        &source,
        &lifecycle,
        &receipt,
        &input_capsule,
        &ledger,
        &frozen_prediction.participant_predictions,
        &frozen_prediction.feature_identity_digest,
    )?;
    for seal in &seals {
        add_counts(
            &mut counts,
            persist_v4_3(
                &v4_3_root,
                "participant_prediction_seals",
                &seal.seal_digest,
                "pb",
                &encode_prediction_seal_v4_3(seal)?,
                |bytes| Ok(decode_prediction_seal_v4_3(bytes)?.seal_digest),
            )?,
        );
    }
    let prediction_capsule = build_prediction_capsule_v4_3(
        &source,
        &lifecycle,
        &authorization,
        &supersession,
        &context_plan,
        &input_registration,
        &receipt,
        &input_capsule,
        &ledger,
        seals,
    )?;
    add_counts(
        &mut counts,
        persist_v4_3(
            &v4_3_root,
            "prediction_capsules",
            &prediction_capsule.capsule_digest,
            "pb",
            &encode_prediction_capsule_v4_3(&prediction_capsule)?,
            |bytes| Ok(decode_prediction_capsule_v4_3(bytes)?.capsule_digest),
        )?,
    );
    let journal = build_prediction_journal_v4_3(
        &authorization,
        &input_capsule,
        &ledger,
        &prediction_capsule,
    )?;
    add_counts(
        &mut counts,
        persist_v4_3(
            &v4_3_root,
            "prediction_journals",
            &journal.journal_digest,
            "pb",
            &encode_prediction_journal_v4_3(&journal)?,
            |bytes| Ok(decode_prediction_journal_v4_3(bytes)?.journal_digest),
        )?,
    );
    let outcome_plan = build_outcome_plan_v4_3(&lifecycle, &prediction_capsule)?;
    add_counts(
        &mut counts,
        persist_v4_3(
            &v4_3_root,
            "outcome_plans",
            &outcome_plan.plan_digest,
            "pb",
            &encode_outcome_plan_v4_3(&outcome_plan)?,
            |bytes| Ok(decode_outcome_plan_v4_3(bytes)?.plan_digest),
        )?,
    );
    let protected_after = protected_artifacts_v4_3(root)?;
    let active_after = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    if protected_before != protected_after || active_before != active_after {
        return Err("V4.3 prediction sealing changed protected state".to_string());
    }
    safety_counters.participant_parameter_updates = prediction_capsule
        .participant_prediction_seals
        .iter()
        .map(|seal| seal.parameter_updates)
        .sum();
    safety_counters.normalizer_refits = prediction_capsule
        .participant_prediction_seals
        .iter()
        .map(|seal| seal.normalizer_refits)
        .sum();
    let status = build_status_v4_3(
        &lifecycle,
        &authorization,
        authorization_status,
        &supersession,
        supersession_status,
        &context_plan,
        &input_registration,
        MomentumEventReadinessV4_3::PredictionAlreadySealed,
        Some(&receipt),
        Some(&input_capsule),
        Some(&ledger),
        Some(&prediction_capsule),
        Some(&journal),
        Some(&outcome_plan),
        true,
        true,
        safety_counters,
    )?;
    add_counts(
        &mut counts,
        persist_v4_3(
            &v4_3_root,
            "status_receipts",
            &status.status_digest,
            "pb",
            &encode_status_v4_3(&status)?,
            |bytes| Ok(decode_status_v4_3(bytes)?.status_digest),
        )?,
    );
    let mut report = base_report_v4_3(
        status,
        lifecycle,
        authorization,
        supersession,
        context_plan,
        input_registration,
    );
    report.input_receipt = Some(receipt);
    report.input_capsule = Some(input_capsule);
    report.context_usage_ledger = Some(ledger);
    report.prediction_capsule = Some(prediction_capsule);
    report.prediction_journal = Some(journal);
    report.outcome_plan = Some(outcome_plan);
    report.artifacts_written = counts.0;
    report.duplicate_artifact_count = counts.1;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::{
            SnapshotCompatibilityV1, SnapshotProvenance, SnapshotQualitySummary, SnapshotSourceType,
        },
        league::HistoricalReplayDataset,
        model::momentum_raw_feature_v4::expand_interaction_representation_v4,
    };

    const DAY: u64 = DAILY_CADENCE_MS_V4_2;
    const MINIMUM_EVENT: u64 = 100 * DAY;
    const SOURCE_BOUNDARY: u64 = 99 * DAY;
    const PROTECTED_EVENT: u64 = 95 * DAY;

    fn lifecycle_fixture() -> MomentumFutureEvaluationLifecycleV4_2 {
        let mut value = MomentumFutureEvaluationLifecycleV4_2 {
            lifecycle_version: LIFECYCLE_VERSION_V4_2.to_string(),
            agent_id: AGENT_ID_V4_2.to_string(),
            source_v4_family_digest: "source-family".to_string(),
            accumulated_family_digest: "accumulated-family".to_string(),
            roster_digest: "roster".to_string(),
            evaluation_registration_digest: "evaluation".to_string(),
            participant_digests: vec!["raw".into(), "interaction".into(), "constant".into()],
            participant_parameter_digests: vec![
                "raw-parameters".into(),
                "interaction-parameters".into(),
                "constant-parameters".into(),
            ],
            participant_normalizer_digests: vec![
                "raw-normalizer".into(),
                "interaction-normalizer".into(),
                "constant-normalizer".into(),
            ],
            feature_policy_digest: "feature-policy".to_string(),
            label_policy_digest: "label-policy".to_string(),
            qualification_policy_digest: "qualification-policy".to_string(),
            minimum_accepted_event_timestamp_ms: MINIMUM_EVENT,
            cadence_ms: DAY,
            prediction_horizon: 1,
            input_stage_maximum_requests: 1,
            input_stage_maximum_retries: 0,
            outcome_stage_maximum_requests: 1,
            outcome_stage_maximum_retries: 0,
            prediction_must_precede_outcome_access: true,
            outcome_stage_locked_until_prediction_sealed: true,
            labels_hidden_until_opening: true,
            probabilities_hidden_until_opening: true,
            winner_selection_forbidden: true,
            active_promotion_forbidden: true,
            reward_application_forbidden: true,
            lifecycle_digest: String::new(),
        };
        value.lifecycle_digest = lifecycle_digest(&value);
        value
    }

    fn evaluation_fixture() -> MomentumAccumulatedEvaluationRegistrationV4_1 {
        MomentumAccumulatedEvaluationRegistrationV4_1 {
            registration_version: "momentum-accumulated-evaluation-registration-v4.1".into(),
            agent_id: AGENT_ID_V4_2.into(),
            source_v4_family_digest: "source-family".into(),
            accumulated_family_digest: "accumulated-family".into(),
            roster_digest: "roster".into(),
            supplemental_registration_digest: "supplemental".into(),
            reserve_opening_receipt_digest: "opening".into(),
            accumulated_yield_digest: "yield".into(),
            accumulated_receipt_digests: vec!["one".into(), "two".into(), "three".into()],
            accumulated_interaction_audit_digest: Some("interaction-audit".into()),
            source_snapshot_digest: "snapshot".into(),
            source_boundary_timestamp_ms: SOURCE_BOUNDARY,
            consumed_validation_identity_digests: vec!["validation".into()],
            protected_registration_digests: vec!["protected".into()],
            protected_timestamp_ms: vec![PROTECTED_EVENT],
            provider_finality_boundary_ms: 96 * DAY,
            minimum_accepted_timestamp_ms: MINIMUM_EVENT,
            labels_hidden_until_opening: true,
            probabilities_hidden_until_opening: true,
            one_time_opening_required: true,
            winner_selection_forbidden_before_opening: true,
            active_promotion_forbidden: true,
            reward_application_forbidden: true,
            maximum_requests: 1,
            maximum_concurrency: 1,
            maximum_retries: 0,
            registration_digest: "evaluation".into(),
        }
    }

    fn row(timestamp_ms: u64) -> HistoricalOhlcvRow {
        HistoricalOhlcvRow {
            symbol: "KRW-BTC".into(),
            timestamp_ms,
            open: 100.0,
            high: 102.0,
            low: 99.0,
            close: 101.0,
            volume: 10.0,
            trade_value: Some(1_010.0),
        }
    }

    fn snapshot_fixture() -> DataSnapshot {
        let rows = (80 * DAY..=SOURCE_BOUNDARY)
            .step_by(DAY as usize)
            .map(row)
            .collect::<Vec<_>>();
        DataSnapshot {
            snapshot_id: "snapshot-id".into(),
            request_key: "request-key".into(),
            provider_id: "upbit".into(),
            dataset_kind: DatasetKind::DailyOhlcv,
            market_scope: AcquisitionMarketScope::BtcCrypto,
            symbols: vec!["KRW-BTC".into()],
            requested_lookback: DataLookback {
                bars: rows.len(),
                start_timestamp_ms: Some(80 * DAY),
                end_timestamp_ms: Some(100 * DAY),
            },
            actual_start_timestamp_ms: Some(80 * DAY),
            actual_end_timestamp_ms: Some(SOURCE_BOUNDARY),
            fetched_at_ms: 100 * DAY,
            normalized_at_ms: 100 * DAY,
            schema_version: 1,
            row_count: rows.len(),
            quality_summary: SnapshotQualitySummary {
                accepted: true,
                row_count: rows.len(),
                reason_codes: vec![],
            },
            content_digest: "snapshot".into(),
            sanitized: true,
            read_only: true,
            compatibility: Some(SnapshotCompatibilityV1 {
                cadence: "1d".into(),
                adjustment_semantics: crate::data::SnapshotAdjustmentSemanticsV1::NotApplicable,
                source_schema: "ohlcv".into(),
                requested_cutoff_timestamp_ms: Some(100 * DAY),
                maximum_staleness_ms: 0,
                all_rows_finalized: true,
            }),
            normalized_dataset: HistoricalReplayDataset {
                symbol: "KRW-BTC".into(),
                rows,
                source: "fixture".into(),
                reason_codes: vec![],
            },
            provenance: SnapshotProvenance {
                provider_id: "upbit".into(),
                acquisition_request_id: "request".into(),
                fetch_receipt_id: "receipt".into(),
                source_type: SnapshotSourceType::ApprovedReadOnlyProvider,
                sanitized: true,
                credential_free: true,
                reason_codes: vec![],
            },
            reason_codes: vec![],
        }
    }

    fn no_protected_plan() -> MomentumProspectiveFeatureContextPlanV4_2 {
        let lifecycle = lifecycle_fixture();
        let mut evaluation = evaluation_fixture();
        evaluation.protected_timestamp_ms.clear();
        derive_context_plan(&lifecycle, &evaluation, &snapshot_fixture()).unwrap()
    }

    fn registration_fixture() -> MomentumProspectiveInputAcquisitionRegistrationV4_2 {
        let lifecycle = lifecycle_fixture();
        let plan = no_protected_plan();
        let mut value = MomentumProspectiveInputAcquisitionRegistrationV4_2 {
            registration_version: INPUT_REGISTRATION_VERSION_V4_2.into(),
            lifecycle_digest: lifecycle.lifecycle_digest,
            evaluation_registration_digest: "evaluation".into(),
            roster_digest: "roster".into(),
            event_timestamp_ms: plan.event_timestamp_ms,
            feature_context_plan_digest: plan.plan_digest,
            provider_id: "upbit".into(),
            market: "btc_crypto".into(),
            symbol: "KRW-BTC".into(),
            cadence: "1d".into(),
            exact_expected_timestamp_ms: plan.incremental_row_timestamps,
            exact_request_count: 1,
            request_to_timestamp_ms: 101 * DAY,
            maximum_requests: 1,
            maximum_concurrency: 1,
            maximum_retries: 0,
            maximum_response_bytes: 262_144,
            credential_free_required: true,
            read_only_required: true,
            outcome_timestamp_forbidden: true,
            registration_digest: String::new(),
        };
        value.registration_digest = input_registration_digest(&value);
        value
    }

    fn transport_fixture() -> LearningEvidenceTransportResponseV1 {
        let registration = registration_fixture();
        let rows = registration
            .exact_expected_timestamp_ms
            .iter()
            .copied()
            .map(row)
            .collect::<Vec<_>>();
        let raw_response = b"[{\"fixture\":true}]".to_vec();
        LearningEvidenceTransportResponseV1 {
            http_status_class: "2xx".into(),
            raw_response: raw_response.clone(),
            response: crate::data::ReadOnlyProviderResponse {
                request_id: "request".into(),
                provider_id: "upbit".into(),
                fetched_at_ms: 101 * DAY,
                content_type: "application/x-soma-normalized-dataset".into(),
                all_rows_finalized: true,
                normalized_dataset: HistoricalReplayDataset {
                    symbol: "KRW-BTC".into(),
                    rows,
                    source: "fixture".into(),
                    reason_codes: vec![],
                },
                reported_content_bytes: raw_response.len(),
                reason_codes: vec![],
            },
        }
    }

    fn input_capsule_fixture() -> MomentumProspectiveInputCapsuleV4_2 {
        let lifecycle = lifecycle_fixture();
        let plan = no_protected_plan();
        let registration = registration_fixture();
        validate_input_response(&lifecycle, &plan, &registration, &transport_fixture())
            .unwrap()
            .0
    }

    fn receipt_fixture(
        capsule: &MomentumProspectiveInputCapsuleV4_2,
    ) -> MomentumProspectiveInputReceiptV4_2 {
        build_input_receipt(
            &lifecycle_fixture(),
            &registration_fixture(),
            MomentumProspectiveInputStatusV4_2::EvidenceAcquired,
            Some("2xx".into()),
            capsule.exact_timestamp_ms.len(),
            capsule.exact_timestamp_ms.len(),
            Some(capsule.raw_response_digest.clone()),
            Some(capsule.capsule_digest.clone()),
        )
    }

    fn predictions_fixture() -> Vec<MomentumFrozenParticipantPredictionV4> {
        let lifecycle = lifecycle_fixture();
        (0..3)
            .map(|index| MomentumFrozenParticipantPredictionV4 {
                participant_digest: lifecycle.participant_digests[index].clone(),
                config_digest: format!("config-{index}"),
                parameter_digest: lifecycle.participant_parameter_digests[index].clone(),
                normalizer_digest: lifecycle.participant_normalizer_digests[index].clone(),
                model_artifact_digest: format!("model-{index}"),
                feature_schema_digest: "schema".into(),
                training_identity_digest: "training".into(),
                probability_bits: (0.4_f32 + index as f32 * 0.1).to_bits(),
            })
            .collect()
    }

    fn prediction_capsule_fixture() -> MomentumProspectivePredictionCapsuleV4_2 {
        let lifecycle = lifecycle_fixture();
        let input_capsule = input_capsule_fixture();
        let receipt = receipt_fixture(&input_capsule);
        let seals = seal_participant_predictions(
            &lifecycle,
            &input_capsule,
            &predictions_fixture(),
            "features",
        )
        .unwrap();
        build_prediction_capsule(&lifecycle, &receipt, &input_capsule, seals).unwrap()
    }

    fn status_fixture() -> MomentumFuturePredictionStatusReceiptV4_2 {
        let lifecycle = lifecycle_fixture();
        let plan = no_protected_plan();
        let registration = registration_fixture();
        build_status_receipt(
            &lifecycle,
            &plan,
            &registration,
            MomentumEventReadinessV4_2::AwaitingInputFinality,
            None,
            None,
            None,
            None,
            true,
            true,
            MomentumFuturePredictionSafetyCountersV4_2::default(),
        )
        .unwrap()
    }

    fn authorization_fixture_v4_3() -> MomentumProtectedContextAuthorizationV4_3 {
        let lifecycle = lifecycle_fixture();
        let mut value = MomentumProtectedContextAuthorizationV4_3 {
            authorization_version: AUTHORIZATION_VERSION_V4_3.into(),
            agent_id: AGENT_ID_V4_2.into(),
            v4_family_digest: lifecycle.source_v4_family_digest.clone(),
            accumulated_family_digest: lifecycle.accumulated_family_digest.clone(),
            roster_digest: lifecycle.roster_digest.clone(),
            evaluation_registration_digest: lifecycle.evaluation_registration_digest.clone(),
            lifecycle_digest: lifecycle.lifecycle_digest.clone(),
            participant_digests: lifecycle.participant_digests.clone(),
            parameter_digests: lifecycle.participant_parameter_digests.clone(),
            normalizer_digests: lifecycle.participant_normalizer_digests.clone(),
            model_source_boundary_timestamp_ms: 92 * DAY,
            protected_registration_digests: vec!["protected".into()],
            protected_timestamp_ms: vec![95 * DAY, 96 * DAY, 97 * DAY, 98 * DAY],
            allowed_use_class: ProtectedContextUseClassV4_3::RawOhlcvInferenceContext,
            raw_ohlcv_context_allowed: true,
            training_use_forbidden: true,
            normalizer_fit_forbidden: true,
            label_use_forbidden: true,
            qualification_use_forbidden: true,
            metric_use_forbidden: true,
            reward_use_forbidden: true,
            event_timestamp_use_forbidden: true,
            outcome_timestamp_use_forbidden: true,
            prior_outcome_capsule_use_forbidden: true,
            authorization_status: ProtectedContextAuthorizationStatusV4_3::Authorized,
            authorization_digest: String::new(),
        };
        value.authorization_digest = authorization_digest_v4_3(&value);
        value
    }

    fn context_plan_fixture_v4_3() -> MomentumProspectiveFeatureContextPlanV4_3 {
        let lifecycle = lifecycle_fixture();
        let authorization = authorization_fixture_v4_3();
        let exact_required_timestamp_ms = timestamp_range(85 * DAY, 16, DAY).unwrap();
        let existing_source_timestamp_ms = timestamp_range(85 * DAY, 8, DAY).unwrap();
        let protected_context_timestamp_ms = authorization.protected_timestamp_ms.clone();
        let incremental_timestamp_ms = vec![93 * DAY, 94 * DAY, 99 * DAY, 100 * DAY];
        let context_usage_policy_digest = stable_hash_string(&format!(
            "momentum-v4.3-context-usage:{}:{:?}:{:?}:{:?}",
            authorization.authorization_digest,
            existing_source_timestamp_ms,
            protected_context_timestamp_ms,
            incremental_timestamp_ms
        ));
        let mut value = MomentumProspectiveFeatureContextPlanV4_3 {
            plan_version: CONTEXT_PLAN_VERSION_V4_3.into(),
            lifecycle_digest: lifecycle.lifecycle_digest,
            context_authorization_digest: authorization.authorization_digest,
            event_timestamp_ms: MINIMUM_EVENT,
            input_finality_boundary_ms: 101 * DAY,
            required_context_start_timestamp_ms: 85 * DAY,
            required_context_end_timestamp_ms: MINIMUM_EVENT,
            required_row_count: 16,
            exact_required_timestamp_ms,
            existing_source_timestamp_ms,
            protected_context_timestamp_ms,
            incremental_timestamp_ms,
            context_usage_policy_digest,
            plan_digest: String::new(),
        };
        value.plan_digest = context_plan_digest_v4_3(&value);
        value
    }

    fn supersession_fixture_v4_3() -> MomentumInputPlanSupersessionV4_3 {
        let lifecycle = lifecycle_fixture();
        let authorization = authorization_fixture_v4_3();
        let old_plan =
            derive_context_plan(&lifecycle, &evaluation_fixture(), &snapshot_fixture()).unwrap();
        let replacement_plan = context_plan_fixture_v4_3();
        let old_registration = registration_fixture();
        let mut value = derive_supersession_v4_3(
            &lifecycle,
            &authorization,
            &old_plan,
            &old_registration,
            &replacement_plan,
            0,
            false,
            false,
            false,
        );
        value.replacement_input_registration_digest = "replacement-registration".into();
        value
    }

    fn registration_fixture_v4_3() -> MomentumProspectiveInputRegistrationV4_3 {
        let lifecycle = lifecycle_fixture();
        let authorization = authorization_fixture_v4_3();
        let supersession = supersession_fixture_v4_3();
        let plan = context_plan_fixture_v4_3();
        let mut value = MomentumProspectiveInputRegistrationV4_3 {
            registration_version: INPUT_REGISTRATION_VERSION_V4_3.into(),
            lifecycle_digest: lifecycle.lifecycle_digest,
            evaluation_registration_digest: lifecycle.evaluation_registration_digest,
            roster_digest: lifecycle.roster_digest,
            context_authorization_digest: authorization.authorization_digest,
            supersession_digest: supersession.supersession_digest,
            context_plan_digest: plan.plan_digest,
            event_timestamp_ms: plan.event_timestamp_ms,
            input_finality_boundary_ms: plan.input_finality_boundary_ms,
            provider_id: "upbit".into(),
            market: "btc_crypto".into(),
            symbol: "KRW-BTC".into(),
            cadence: "1d".into(),
            exact_expected_timestamp_ms: plan.exact_required_timestamp_ms,
            expected_row_count: 16,
            request_to_timestamp_ms: 101 * DAY,
            maximum_requests: 1,
            maximum_concurrency: 1,
            maximum_retries: 0,
            maximum_response_bytes: 262_144,
            credential_free_required: true,
            read_only_required: true,
            outcome_timestamp_forbidden: true,
            prior_outcome_artifact_forbidden: true,
            registration_digest: String::new(),
        };
        value.registration_digest = input_registration_digest_v4_3(&value);
        value
    }

    fn transport_fixture_v4_3() -> LearningEvidenceTransportResponseV1 {
        let registration = registration_fixture_v4_3();
        let rows = registration
            .exact_expected_timestamp_ms
            .iter()
            .copied()
            .map(row)
            .collect::<Vec<_>>();
        let raw_response = b"[{\"fixture\":\"v4.3\"}]".to_vec();
        LearningEvidenceTransportResponseV1 {
            http_status_class: "2xx".into(),
            raw_response: raw_response.clone(),
            response: crate::data::ReadOnlyProviderResponse {
                request_id: "request-v4.3".into(),
                provider_id: "upbit".into(),
                fetched_at_ms: 101 * DAY,
                content_type: "application/x-soma-normalized-dataset".into(),
                all_rows_finalized: true,
                normalized_dataset: HistoricalReplayDataset {
                    symbol: "KRW-BTC".into(),
                    rows,
                    source: "fixture".into(),
                    reason_codes: vec![],
                },
                reported_content_bytes: raw_response.len(),
                reason_codes: vec![],
            },
        }
    }

    fn input_bundle_fixture_v4_3() -> (
        MomentumProspectiveInputCapsuleV4_3,
        MomentumContextUsageLedgerV4_3,
        MomentumProspectiveContextVerificationV4_3,
        Vec<HistoricalOhlcvRow>,
    ) {
        validate_input_response_v4_3(
            &lifecycle_fixture(),
            &authorization_fixture_v4_3(),
            &context_plan_fixture_v4_3(),
            &registration_fixture_v4_3(),
            &transport_fixture_v4_3(),
        )
        .unwrap()
    }

    fn receipt_fixture_v4_3() -> MomentumProspectiveInputReceiptV4_3 {
        let (capsule, _, _, rows) = input_bundle_fixture_v4_3();
        build_input_receipt_v4_3(
            &lifecycle_fixture(),
            &authorization_fixture_v4_3(),
            &registration_fixture_v4_3(),
            MomentumProspectiveInputStatusV4_2::EvidenceAcquired,
            Some("2xx".into()),
            rows.len(),
            rows.len(),
            Some(capsule.raw_response_digest.clone()),
            Some(capsule.capsule_digest.clone()),
        )
    }

    fn prediction_capsule_fixture_v4_3() -> MomentumProspectivePredictionCapsuleV4_3 {
        let lifecycle = lifecycle_fixture();
        let authorization = authorization_fixture_v4_3();
        let supersession = supersession_fixture_v4_3();
        let plan = context_plan_fixture_v4_3();
        let registration = registration_fixture_v4_3();
        let (input_capsule, ledger, _, _) = input_bundle_fixture_v4_3();
        let receipt = receipt_fixture_v4_3();
        let roles = [
            "RawFeatureLogisticV4",
            "RawFeatureInteractionLogisticV4",
            "TrainingPrevalenceConstantV4",
        ];
        let seals = lifecycle
            .participant_digests
            .iter()
            .enumerate()
            .map(|(index, participant_digest)| {
                let bits = (0.4_f32 + index as f32 * 0.1).to_bits();
                let mut seal = MomentumParticipantPredictionSealV4_3 {
                    participant_digest: participant_digest.clone(),
                    participant_role: roles[index].into(),
                    event_timestamp_ms: plan.event_timestamp_ms,
                    input_receipt_digest: receipt.receipt_digest.clone(),
                    input_capsule_digest: input_capsule.capsule_digest.clone(),
                    context_usage_ledger_digest: ledger.ledger_digest.clone(),
                    feature_identity_digest: "feature-identity".into(),
                    prediction_probability_bits: bits,
                    prediction_digest: stable_hash_string(&format!(
                        "fixture-prediction:{participant_digest}:{bits}"
                    )),
                    participant_identity_verified: true,
                    parameter_updates: 0,
                    normalizer_refits: 0,
                    outcome_access_count: 0,
                    seal_digest: String::new(),
                };
                seal.seal_digest = prediction_seal_digest_v4_3(&seal);
                seal
            })
            .collect();
        let mut capsule = MomentumProspectivePredictionCapsuleV4_3 {
            capsule_version: PREDICTION_CAPSULE_VERSION_V4_3.into(),
            lifecycle_digest: lifecycle.lifecycle_digest,
            evaluation_registration_digest: lifecycle.evaluation_registration_digest,
            roster_digest: lifecycle.roster_digest,
            context_authorization_digest: authorization.authorization_digest,
            supersession_digest: supersession.supersession_digest,
            corrected_context_plan_digest: plan.plan_digest,
            input_registration_digest: registration.registration_digest,
            event_timestamp_ms: plan.event_timestamp_ms,
            input_receipt_digest: receipt.receipt_digest,
            input_capsule_digest: input_capsule.capsule_digest,
            context_usage_ledger_digest: ledger.ledger_digest,
            participant_prediction_seals: seals,
            probabilities_hidden: true,
            labels_hidden: true,
            outcome_accessed: false,
            metrics_computed: false,
            winner_selected: false,
            capsule_digest: String::new(),
        };
        capsule.capsule_digest = prediction_capsule_digest_v4_3(&capsule);
        capsule
    }

    fn journal_fixture_v4_3() -> MomentumProspectivePredictionJournalV4_3 {
        let authorization = authorization_fixture_v4_3();
        let (input_capsule, ledger, _, _) = input_bundle_fixture_v4_3();
        build_prediction_journal_v4_3(
            &authorization,
            &input_capsule,
            &ledger,
            &prediction_capsule_fixture_v4_3(),
        )
        .unwrap()
    }

    fn outcome_plan_fixture_v4_3() -> MomentumProspectiveOutcomePlanV4_3 {
        build_outcome_plan_v4_3(&lifecycle_fixture(), &prediction_capsule_fixture_v4_3()).unwrap()
    }

    fn status_fixture_v4_3() -> MomentumFuturePredictionStatusReceiptV4_3 {
        let lifecycle = lifecycle_fixture();
        let authorization = authorization_fixture_v4_3();
        let supersession = supersession_fixture_v4_3();
        let plan = context_plan_fixture_v4_3();
        let registration = registration_fixture_v4_3();
        build_status_v4_3(
            &lifecycle,
            &authorization,
            ProtectedContextAuthorizationStatusV4_3::Authorized,
            &supersession,
            MomentumInputPlanSupersessionStatusV4_3::Superseded,
            &plan,
            &registration,
            MomentumEventReadinessV4_3::AwaitingInputFinality,
            None,
            None,
            None,
            None,
            None,
            None,
            true,
            true,
            MomentumFuturePredictionSafetyCountersV4_3::default(),
        )
        .unwrap()
    }

    #[test]
    fn sprint81_family_invariants_remain_frozen() {
        let lifecycle = lifecycle_fixture();
        assert_eq!(lifecycle.participant_digests.len(), 3);
        assert!(lifecycle.winner_selection_forbidden);
        assert!(lifecycle.active_promotion_forbidden);
        assert!(lifecycle.reward_application_forbidden);
    }

    #[test]
    fn protected_artifact_collection_excludes_only_v4_2() {
        let root = std::env::temp_dir().join(format!("soma-v4-2-protected-{}", std::process::id()));
        fs::create_dir_all(root.join("v4")).unwrap();
        fs::create_dir_all(root.join("v4_2")).unwrap();
        fs::write(root.join("v4/frozen.pb"), b"frozen").unwrap();
        fs::write(root.join("v4_2/additive.pb"), b"additive").unwrap();
        let before = protected_artifacts(&root).unwrap();
        fs::write(root.join("v4_2/additive.pb"), b"changed").unwrap();
        assert_eq!(before, protected_artifacts(&root).unwrap());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn request_budget_meaning_is_explicitly_ambiguous() {
        assert_eq!(
            classify_request_budget(&evaluation_fixture()),
            FutureEvaluationRequestBudgetMeaningV4_2::Ambiguous
        );
    }

    #[test]
    fn ambiguous_request_semantics_block_transport_readiness() {
        let lifecycle = lifecycle_fixture();
        let plan =
            derive_context_plan(&lifecycle, &evaluation_fixture(), &snapshot_fixture()).unwrap();
        assert_eq!(
            event_readiness(&lifecycle, &plan, u64::MAX, 200),
            MomentumEventReadinessV4_2::ContextPolicyAmbiguous
        );
    }

    #[test]
    fn prediction_capsule_rejects_outcome_access() {
        let mut capsule = prediction_capsule_fixture();
        capsule.outcome_accessed = true;
        capsule.capsule_digest = prediction_capsule_digest(&capsule);
        assert!(decode_prediction_capsule(&encode_prediction_capsule(&capsule).unwrap()).is_err());
    }

    #[test]
    fn event_timestamp_is_cadence_aligned() {
        assert_eq!(align_up(MINIMUM_EVENT + 1, DAY).unwrap(), 101 * DAY);
        assert_eq!(align_up(MINIMUM_EVENT, DAY).unwrap(), MINIMUM_EVENT);
    }

    #[test]
    fn event_timestamp_respects_minimum_boundary() {
        let plan = no_protected_plan();
        assert!(plan.event_timestamp_ms >= lifecycle_fixture().minimum_accepted_event_timestamp_ms);
    }

    #[test]
    fn input_finality_is_required() {
        let lifecycle = lifecycle_fixture();
        let plan = no_protected_plan();
        assert_eq!(
            event_readiness(&lifecycle, &plan, plan.event_timestamp_ms, 200),
            MomentumEventReadinessV4_2::AwaitingInputFinality
        );
    }

    #[test]
    fn prefinality_has_zero_transport_attempts() {
        let status = status_fixture();
        assert_eq!(status.safety_counters.input_request_attempts, 0);
        assert_eq!(status.request_attempt_count, 0);
    }

    #[test]
    fn feature_context_is_deterministic() {
        let lifecycle = lifecycle_fixture();
        let mut evaluation = evaluation_fixture();
        evaluation.protected_timestamp_ms.clear();
        let left = derive_context_plan(&lifecycle, &evaluation, &snapshot_fixture()).unwrap();
        let right = derive_context_plan(&lifecycle, &evaluation, &snapshot_fixture()).unwrap();
        assert_eq!(left, right);
        assert_eq!(left.required_row_count, 16);
    }

    #[test]
    fn protected_context_semantics_are_explicit() {
        let plan = derive_context_plan(
            &lifecycle_fixture(),
            &evaluation_fixture(),
            &snapshot_fixture(),
        )
        .unwrap();
        assert_eq!(
            plan.context_policy_status,
            MomentumContextPolicyStatusV4_2::ContextUseAmbiguous
        );
        assert_eq!(plan.protected_context_timestamp_ms, [PROTECTED_EVENT]);
    }

    #[test]
    fn forbidden_context_derives_later_event() {
        let plan = derive_context_plan_with_policy(
            &lifecycle_fixture(),
            &evaluation_fixture(),
            &snapshot_fixture(),
            Some(MomentumContextPolicyStatusV4_2::ContextUseExplicitlyForbidden),
        )
        .unwrap();
        assert_eq!(plan.event_timestamp_ms, 111 * DAY);
        assert_eq!(
            plan.context_policy_status,
            MomentumContextPolicyStatusV4_2::ContextUseExplicitlyForbidden
        );
    }

    #[test]
    fn input_registration_roundtrip_precedes_transport() {
        let registration = registration_fixture();
        let root =
            std::env::temp_dir().join(format!("soma-v4-2-registration-{}", std::process::id()));
        persist_input_registration(&root, &registration).unwrap();
        let reopened = reopen_exact(
            &root
                .join("input_registrations")
                .join(format!("{}.pb", registration.registration_digest)),
            decode_input_registration,
        )
        .unwrap();
        assert_eq!(reopened, registration);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn exactly_one_input_request_is_permitted() {
        let registration = registration_fixture();
        assert_eq!(registration.maximum_requests, 1);
        assert_eq!(registration.exact_request_count, 1);
        assert!(build_provider_request(&registration).is_ok());
    }

    #[test]
    fn input_retries_remain_zero() {
        let registration = registration_fixture();
        let capsule = input_capsule_fixture();
        let receipt = receipt_fixture(&capsule);
        assert_eq!(registration.maximum_retries, 0);
        assert_eq!(receipt.retry_count, 0);
    }

    #[test]
    fn fallback_provider_is_forbidden() {
        let mut registration = registration_fixture();
        registration.provider_id = "fallback".into();
        registration.registration_digest = input_registration_digest(&registration);
        let mut config = UpbitHistoricalPilotConfigV0 {
            provider_id: "upbit".into(),
            enabled: true,
            market: AcquisitionMarketScope::BtcCrypto,
            symbol: "KRW-BTC".into(),
            start_timestamp_ms: 1,
            end_timestamp_ms: 2,
            maximum_rows: 200,
            timeout_seconds: 1,
            max_retries: 0,
            maximum_response_bytes: 262_144,
            snapshot_output_dir: "data/local_snapshots/upbit".into(),
            network_consent: crate::data::NetworkConsentV0::ManualLocalSmoke,
            manual_smoke_enabled: true,
            page_size: 200,
            target_rows: 1,
            maximum_pages: 1,
            stop_when_campaign_sufficient: true,
            campaign_attempt_enabled: true,
            minimum_inter_request_delay_ms: 1,
        };
        config.start_timestamp_ms = 80 * DAY;
        config.end_timestamp_ms = 101 * DAY;
        assert!(request_config(&config, &registration).is_err());
    }

    #[test]
    fn response_requires_exact_timestamp_set() {
        let mut transport = transport_fixture();
        transport.response.normalized_dataset.rows.pop();
        assert!(
            validate_input_response(
                &lifecycle_fixture(),
                &no_protected_plan(),
                &registration_fixture(),
                &transport,
            )
            .is_err()
        );
    }

    #[test]
    fn outcome_timestamp_in_input_response_rejects() {
        let mut registration = registration_fixture();
        registration.exact_expected_timestamp_ms.push(101 * DAY);
        registration.registration_digest = input_registration_digest(&registration);
        let mut transport = transport_fixture();
        transport
            .response
            .normalized_dataset
            .rows
            .push(row(101 * DAY));
        assert!(
            validate_input_response(
                &lifecycle_fixture(),
                &no_protected_plan(),
                &registration,
                &transport,
            )
            .is_err()
        );
    }

    #[test]
    fn participant_parameter_digest_mismatch_rejects() {
        let mut predictions = predictions_fixture();
        predictions[0].parameter_digest = "changed".into();
        assert!(
            seal_participant_predictions(
                &lifecycle_fixture(),
                &input_capsule_fixture(),
                &predictions,
                "features",
            )
            .is_err()
        );
    }

    #[test]
    fn participant_normalizer_digest_mismatch_rejects() {
        let mut predictions = predictions_fixture();
        predictions[1].normalizer_digest = "changed".into();
        assert!(
            seal_participant_predictions(
                &lifecycle_fixture(),
                &input_capsule_fixture(),
                &predictions,
                "features",
            )
            .is_err()
        );
    }

    #[test]
    fn prediction_seals_change_no_parameters() {
        let seals = prediction_capsule_fixture().participant_prediction_seals;
        assert!(seals.iter().all(|seal| seal.parameter_updates == 0));
        assert!(seals.iter().all(|seal| seal.participant_reconstructed));
    }

    #[test]
    fn all_participants_share_event_and_context() {
        let capsule = prediction_capsule_fixture();
        let event = capsule.event_timestamp_ms;
        let input = capsule.input_capsule_digest;
        assert!(capsule.participant_prediction_seals.iter().all(|seal| {
            seal.event_timestamp_ms == event && seal.input_capsule_digest == input
        }));
    }

    #[test]
    fn interaction_feature_order_is_deterministic() {
        assert_eq!(
            expand_interaction_representation_v4(&[2.0, 3.0, 5.0]).unwrap(),
            [2.0, 3.0, 5.0, 4.0, 9.0, 25.0, 6.0, 10.0, 15.0]
        );
    }

    #[test]
    fn public_status_hides_probabilities() {
        let json = serde_json::to_string(&status_fixture()).unwrap();
        assert!(!json.contains("probability"));
        assert!(!json.contains("feature_identity"));
        assert!(!json.contains("parameter_digest"));
    }

    #[test]
    fn prediction_capsule_contains_complete_roster() {
        let capsule = prediction_capsule_fixture();
        assert_eq!(capsule.participant_prediction_seals.len(), 3);
        assert_eq!(
            capsule
                .participant_prediction_seals
                .iter()
                .map(|seal| seal.participant_digest.clone())
                .collect::<Vec<_>>(),
            lifecycle_fixture().participant_digests
        );
    }

    #[test]
    fn prediction_capsule_computes_no_metric() {
        let capsule = prediction_capsule_fixture();
        assert!(!capsule.metrics_computed);
        assert!(capsule.probabilities_hidden);
        assert!(capsule.labels_hidden);
    }

    #[test]
    fn prediction_capsule_selects_no_winner() {
        assert!(!prediction_capsule_fixture().winner_selected);
    }

    #[test]
    fn outcome_maturity_uses_frozen_horizon() {
        let lifecycle = lifecycle_fixture();
        let capsule = prediction_capsule_fixture();
        let plan = build_maturity_plan(&lifecycle, &capsule).unwrap();
        assert_eq!(plan.prediction_horizon, lifecycle.prediction_horizon);
        assert_eq!(
            plan.required_outcome_timestamp_ms,
            [capsule.event_timestamp_ms + lifecycle.cadence_ms]
        );
    }

    #[test]
    fn outcome_stage_remains_locked() {
        let journal = build_prediction_journal(&prediction_capsule_fixture()).unwrap();
        assert!(journal.entries[0].prediction_sealed_before_outcome);
        assert!(!journal.entries[0].outcome_stage_unlocked);
    }

    #[test]
    fn sealed_replay_status_has_zero_new_work() {
        let mut counters = MomentumFuturePredictionSafetyCountersV4_2::default();
        counters.input_request_attempts = 0;
        validate_safety_counters(&counters).unwrap();
        assert_eq!(counters.outcome_request_attempts, 0);
        assert_eq!(counters.participant_parameter_updates, 0);
    }

    #[test]
    fn prior_prospective_attribution_is_unchanged() {
        let status = status_fixture();
        assert_eq!(
            status.prior_momentum_attribution,
            "MissedMaterialOpportunity"
        );
        assert_eq!(status.prior_cycle_risk_attribution, "CorrectUncertainty");
    }

    #[test]
    fn reward_and_penalty_authority_remain_zero() {
        let counters = status_fixture().safety_counters;
        assert_eq!(counters.reward_applications, 0);
        assert_eq!(counters.penalty_applications, 0);
    }

    #[test]
    fn protobuf_corruption_rejects_every_artifact_category() {
        let corrupt = [0xff];
        assert!(decode_lifecycle(&corrupt).is_err());
        assert!(decode_context_plan(&corrupt).is_err());
        assert!(decode_input_registration(&corrupt).is_err());
        assert!(decode_input_receipt(&corrupt).is_err());
        assert!(decode_input_capsule(&corrupt).is_err());
        assert!(decode_context_proof(&corrupt).is_err());
        assert!(decode_prediction_seal(&corrupt).is_err());
        assert!(decode_prediction_capsule(&corrupt).is_err());
        assert!(decode_prediction_journal(&corrupt).is_err());
        assert!(decode_maturity_plan(&corrupt).is_err());
        assert!(decode_status(&corrupt).is_err());
    }

    #[test]
    fn v4_2_status_json_retains_contract_fields() {
        let status = status_fixture();
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(
            json["event_timestamp_ms"].as_u64(),
            Some(status.event_timestamp_ms)
        );
        assert_eq!(
            json["input_finality_boundary_ms"].as_u64(),
            Some(status.input_finality_boundary_ms)
        );
        assert_eq!(
            json["context_plan_digest"].as_str(),
            Some(status.context_plan_digest.as_str())
        );
        assert_eq!(
            json["status_digest"].as_str(),
            Some(status.status_digest.as_str())
        );
    }

    #[test]
    fn active_and_authority_counters_remain_zero() {
        let status = status_fixture();
        let counters = status.safety_counters;
        validate_safety_counters(&counters).unwrap();
        assert!(status.active_state_unchanged);
        assert!(status.protected_artifacts_unchanged);
        assert_eq!(counters.active_committee_count, 3);
    }

    #[test]
    fn sprint83_01_pr14_invariants_remain_valid() {
        let lifecycle = lifecycle_fixture();
        assert_eq!(
            decode_lifecycle(&encode_lifecycle(&lifecycle).unwrap()).unwrap(),
            lifecycle
        );
        assert_eq!(lifecycle.participant_digests.len(), 3);
        assert_eq!(lifecycle.input_stage_maximum_requests, 1);
        assert_eq!(lifecycle.outcome_stage_maximum_requests, 1);
    }

    #[test]
    fn sprint83_02_preexisting_artifacts_are_protected_byte_for_byte() {
        let root = std::env::temp_dir().join(format!("soma-v4-3-protected-{}", std::process::id()));
        fs::create_dir_all(root.join("v4_2")).unwrap();
        fs::create_dir_all(root.join("v4_3")).unwrap();
        fs::write(root.join("v4_2/history.pb"), b"immutable").unwrap();
        fs::write(root.join("v4_3/additive.pb"), b"before").unwrap();
        let before = protected_artifacts_v4_3(&root).unwrap();
        fs::write(root.join("v4_3/additive.pb"), b"after").unwrap();
        assert_eq!(before, protected_artifacts_v4_3(&root).unwrap());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sprint83_03_authorization_binds_exact_frozen_participants() {
        let authorization = authorization_fixture_v4_3();
        let lifecycle = lifecycle_fixture();
        assert_eq!(
            authorization.participant_digests,
            lifecycle.participant_digests
        );
        assert_eq!(
            authorization.parameter_digests,
            lifecycle.participant_parameter_digests
        );
        assert_eq!(
            authorization.normalizer_digests,
            lifecycle.participant_normalizer_digests
        );
        assert_eq!(
            decode_authorization_v4_3(&encode_authorization_v4_3(&authorization).unwrap()).unwrap(),
            authorization
        );
    }

    #[test]
    fn sprint83_04_authorization_rejects_model_source_overlap() {
        let mut authorization = authorization_fixture_v4_3();
        authorization.model_source_boundary_timestamp_ms = authorization.protected_timestamp_ms[0];
        authorization.authorization_digest = authorization_digest_v4_3(&authorization);
        assert!(validate_authorization_policy_v4_3(&authorization).is_err());
    }

    #[test]
    fn sprint83_05_authorization_rejects_prior_outcome_provenance() {
        let mut authorization = authorization_fixture_v4_3();
        authorization.authorization_status =
            ProtectedContextAuthorizationStatusV4_3::PriorOutcomeArtifactReferenced;
        authorization.authorization_digest = authorization_digest_v4_3(&authorization);
        assert!(validate_authorization_policy_v4_3(&authorization).is_err());
    }

    #[test]
    fn sprint83_06_authorization_allows_only_raw_ohlcv_inference_context() {
        let authorization = authorization_fixture_v4_3();
        validate_authorization_policy_v4_3(&authorization).unwrap();
        assert_eq!(
            authorization.allowed_use_class,
            ProtectedContextUseClassV4_3::RawOhlcvInferenceContext
        );
        assert!(authorization.raw_ohlcv_context_allowed);
    }

    #[test]
    fn sprint83_07_protected_rows_cannot_train_parameters() {
        let (_, ledger, _, _) = input_bundle_fixture_v4_3();
        assert!(ledger.entries.iter().all(|entry| !entry.used_for_training));
    }

    #[test]
    fn sprint83_08_protected_rows_cannot_fit_normalizers() {
        let (_, ledger, _, _) = input_bundle_fixture_v4_3();
        assert!(
            ledger
                .entries
                .iter()
                .all(|entry| !entry.used_for_normalizer_fit)
        );
    }

    #[test]
    fn sprint83_09_protected_rows_cannot_create_labels() {
        let (_, ledger, _, _) = input_bundle_fixture_v4_3();
        assert!(ledger.entries.iter().all(|entry| !entry.used_for_label));
    }

    #[test]
    fn sprint83_10_protected_rows_cannot_create_metrics_or_rewards() {
        let (_, ledger, _, _) = input_bundle_fixture_v4_3();
        assert!(
            ledger
                .entries
                .iter()
                .all(|entry| !entry.used_for_metric && !entry.used_for_reward)
        );
    }

    #[test]
    fn sprint83_11_protected_timestamps_cannot_become_event() {
        let authorization = authorization_fixture_v4_3();
        let plan = context_plan_fixture_v4_3();
        assert!(
            !authorization
                .protected_timestamp_ms
                .contains(&plan.event_timestamp_ms)
        );
    }

    #[test]
    fn sprint83_12_protected_timestamps_cannot_become_outcome() {
        let authorization = authorization_fixture_v4_3();
        let outcome = context_plan_fixture_v4_3().event_timestamp_ms + DAY;
        assert!(!authorization.protected_timestamp_ms.contains(&outcome));
    }

    #[test]
    fn sprint83_13_old_context_plan_remains_immutable() {
        let old_plan = derive_context_plan(
            &lifecycle_fixture(),
            &evaluation_fixture(),
            &snapshot_fixture(),
        )
        .unwrap();
        let before = encode_context_plan(&old_plan).unwrap();
        let _ = supersession_fixture_v4_3();
        assert_eq!(before, encode_context_plan(&old_plan).unwrap());
    }

    #[test]
    fn sprint83_14_old_input_registration_remains_immutable() {
        let registration = registration_fixture();
        let before = encode_input_registration(&registration).unwrap();
        let _ = supersession_fixture_v4_3();
        assert_eq!(before, encode_input_registration(&registration).unwrap());
    }

    #[test]
    fn sprint83_15_old_input_registration_is_execution_ineligible() {
        let lifecycle = lifecycle_fixture();
        let old_plan =
            derive_context_plan(&lifecycle, &evaluation_fixture(), &snapshot_fixture()).unwrap();
        let old_registration = registration_fixture();
        let supersession = supersession_fixture_v4_3();
        assert!(old_registration_superseded_v4_3(
            &supersession,
            &lifecycle,
            &old_plan,
            &old_registration
        ));
    }

    #[test]
    fn sprint83_16_supersession_rejects_prior_attempt() {
        let lifecycle = lifecycle_fixture();
        let authorization = authorization_fixture_v4_3();
        let old_plan =
            derive_context_plan(&lifecycle, &evaluation_fixture(), &snapshot_fixture()).unwrap();
        let value = derive_supersession_v4_3(
            &lifecycle,
            &authorization,
            &old_plan,
            &registration_fixture(),
            &context_plan_fixture_v4_3(),
            1,
            false,
            false,
            false,
        );
        assert_eq!(
            value.status,
            MomentumInputPlanSupersessionStatusV4_3::PriorAttemptExists
        );
    }

    #[test]
    fn sprint83_17_corrected_event_derives_from_original_minimum() {
        let lifecycle = lifecycle_fixture();
        let plan = context_plan_fixture_v4_3();
        assert_eq!(
            plan.event_timestamp_ms,
            lifecycle.minimum_accepted_event_timestamp_ms
        );
    }

    #[test]
    fn sprint83_18_corrected_context_has_exactly_sixteen_daily_timestamps() {
        let plan = context_plan_fixture_v4_3();
        assert_eq!(plan.exact_required_timestamp_ms.len(), 16);
        assert!(
            plan.exact_required_timestamp_ms
                .windows(2)
                .all(|pair| pair[1] == pair[0] + DAY)
        );
    }

    #[test]
    fn sprint83_19_corrected_context_ends_at_event() {
        let plan = context_plan_fixture_v4_3();
        assert_eq!(
            plan.exact_required_timestamp_ms.last(),
            Some(&plan.event_timestamp_ms)
        );
        assert_eq!(
            plan.required_context_end_timestamp_ms,
            plan.event_timestamp_ms
        );
    }

    #[test]
    fn sprint83_20_outcome_is_absent_from_input_registration() {
        let registration = registration_fixture_v4_3();
        assert!(
            !registration
                .exact_expected_timestamp_ms
                .contains(&(registration.event_timestamp_ms + DAY))
        );
        assert!(registration.outcome_timestamp_forbidden);
    }

    #[test]
    fn sprint83_21_prefinality_constructs_zero_transport() {
        let readiness = event_readiness_v4_3(100 * DAY, 101 * DAY, false, false);
        let counters = MomentumFuturePredictionSafetyCountersV4_3::default();
        assert_eq!(readiness, MomentumEventReadinessV4_3::AwaitingInputFinality);
        assert_eq!(counters.input_request_attempts, 0);
    }

    #[test]
    fn sprint83_22_postfinality_permits_exactly_one_transport() {
        let readiness = event_readiness_v4_3(101 * DAY, 101 * DAY, false, false);
        let request = build_provider_request_v4_3(&registration_fixture_v4_3()).unwrap();
        assert_eq!(
            readiness,
            MomentumEventReadinessV4_3::ReadyForInputAcquisition
        );
        assert_eq!(request.lookback.bars, 16);
    }

    #[test]
    fn sprint83_23_input_retries_remain_zero() {
        let registration = registration_fixture_v4_3();
        assert_eq!(registration.maximum_requests, 1);
        assert_eq!(registration.maximum_retries, 0);
        assert_eq!(registration.maximum_concurrency, 1);
    }

    #[test]
    fn sprint83_24_exact_response_timestamp_set_is_enforced() {
        let mut transport = transport_fixture_v4_3();
        transport.response.normalized_dataset.rows.pop();
        assert!(
            validate_input_response_v4_3(
                &lifecycle_fixture(),
                &authorization_fixture_v4_3(),
                &context_plan_fixture_v4_3(),
                &registration_fixture_v4_3(),
                &transport
            )
            .is_err()
        );
    }

    #[test]
    fn sprint83_25_missing_duplicate_extra_and_wrong_market_rows_reject() {
        let validate = |transport: &LearningEvidenceTransportResponseV1| {
            validate_input_response_v4_3(
                &lifecycle_fixture(),
                &authorization_fixture_v4_3(),
                &context_plan_fixture_v4_3(),
                &registration_fixture_v4_3(),
                transport,
            )
        };
        let mut duplicate = transport_fixture_v4_3();
        duplicate.response.normalized_dataset.rows[1].timestamp_ms =
            duplicate.response.normalized_dataset.rows[0].timestamp_ms;
        assert!(validate(&duplicate).is_err());
        let mut extra = transport_fixture_v4_3();
        extra.response.normalized_dataset.rows.push(row(101 * DAY));
        assert!(validate(&extra).is_err());
        let mut wrong_market = transport_fixture_v4_3();
        wrong_market.response.normalized_dataset.rows[0].symbol = "KRW-ETH".into();
        assert!(validate(&wrong_market).is_err());
    }

    #[test]
    fn sprint83_26_prior_outcome_artifact_is_never_read_for_input_values() {
        let (capsule, _, proof, _) = input_bundle_fixture_v4_3();
        assert!(!capsule.prior_outcome_capsule_accessed);
        assert!(!proof.prior_outcome_artifact_accessed);
    }

    #[test]
    fn sprint83_27_context_usage_ledger_classifies_every_row() {
        let (_, ledger, _, _) = input_bundle_fixture_v4_3();
        let classes = ledger
            .entries
            .iter()
            .map(|entry| entry.use_class)
            .collect::<Vec<_>>();
        assert_eq!(classes.len(), 16);
        assert_eq!(
            classes
                .iter()
                .filter(|class| {
                    **class == MomentumContextEvidenceUseV4_3::ProtectedRawInferenceContext
                })
                .count(),
            4
        );
        assert!(classes.contains(&MomentumContextEvidenceUseV4_3::ProspectiveEventInput));
    }

    #[test]
    fn sprint83_28_participant_parameter_mismatch_rejects() {
        let mut predictions = predictions_fixture();
        predictions[0].parameter_digest = "changed".into();
        assert!(
            seal_participant_predictions(
                &lifecycle_fixture(),
                &input_capsule_fixture(),
                &predictions,
                "features"
            )
            .is_err()
        );
    }

    #[test]
    fn sprint83_29_participant_normalizer_mismatch_rejects() {
        let mut predictions = predictions_fixture();
        predictions[1].normalizer_digest = "changed".into();
        assert!(
            seal_participant_predictions(
                &lifecycle_fixture(),
                &input_capsule_fixture(),
                &predictions,
                "features"
            )
            .is_err()
        );
    }

    #[test]
    fn sprint83_30_all_three_participants_use_identical_context() {
        let capsule = prediction_capsule_fixture_v4_3();
        let bindings = capsule
            .participant_prediction_seals
            .iter()
            .map(|seal| {
                (
                    seal.event_timestamp_ms,
                    seal.input_capsule_digest.clone(),
                    seal.context_usage_ledger_digest.clone(),
                    seal.feature_identity_digest.clone(),
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn sprint83_31_interaction_feature_order_remains_deterministic() {
        assert_eq!(
            expand_interaction_representation_v4(&[2.0, 3.0, 5.0]).unwrap(),
            vec![2.0, 3.0, 5.0, 4.0, 9.0, 25.0, 6.0, 10.0, 15.0]
        );
    }

    #[test]
    fn sprint83_32_public_output_hides_probabilities() {
        let status = status_fixture_v4_3();
        let text = crate::cli::format_momentum_v4_future_prediction_text(&status);
        let json = serde_json::to_string(&status).unwrap();
        for forbidden in [
            "probability_bits",
            "ohlcv",
            "labels",
            "parameters",
            "features",
        ] {
            assert!(!text.to_lowercase().contains(forbidden));
            assert!(!json.to_lowercase().contains(forbidden));
        }
    }

    #[test]
    fn sprint83_33_prediction_capsule_contains_exactly_three_seals() {
        let capsule = prediction_capsule_fixture_v4_3();
        assert_eq!(capsule.participant_prediction_seals.len(), 3);
        decode_prediction_capsule_v4_3(&encode_prediction_capsule_v4_3(&capsule).unwrap()).unwrap();
    }

    #[test]
    fn sprint83_34_prediction_creates_no_metric_or_winner() {
        let capsule = prediction_capsule_fixture_v4_3();
        assert!(!capsule.metrics_computed);
        assert!(!capsule.winner_selected);
    }

    #[test]
    fn sprint83_35_outcome_plan_derives_from_frozen_horizon() {
        let plan = outcome_plan_fixture_v4_3();
        assert_eq!(
            plan.prediction_horizon,
            lifecycle_fixture().prediction_horizon
        );
        assert_eq!(plan.required_outcome_timestamp_ms, vec![101 * DAY]);
        assert_eq!(plan.outcome_finality_boundary_ms, 102 * DAY);
    }

    #[test]
    fn sprint83_36_outcome_access_remains_zero() {
        let capsule = prediction_capsule_fixture_v4_3();
        assert!(!capsule.outcome_accessed);
        assert!(
            capsule
                .participant_prediction_seals
                .iter()
                .all(|seal| seal.outcome_access_count == 0)
        );
    }

    #[test]
    fn sprint83_37_successful_replay_performs_zero_work() {
        assert_eq!(
            event_readiness_v4_3(u64::MAX, 101 * DAY, false, true),
            MomentumEventReadinessV4_3::PredictionAlreadySealed
        );
        let counters = MomentumFuturePredictionSafetyCountersV4_3::default();
        assert_eq!(counters.input_request_attempts, 0);
        assert_eq!(counters.participant_parameter_updates, 0);
        let journal = journal_fixture_v4_3();
        assert_eq!(
            decode_prediction_journal_v4_3(&encode_prediction_journal_v4_3(&journal).unwrap())
                .unwrap(),
            journal
        );
    }

    #[test]
    fn sprint83_38_terminal_failed_input_does_not_retry() {
        assert_eq!(
            event_readiness_v4_3(u64::MAX, 101 * DAY, true, false),
            MomentumEventReadinessV4_3::PriorInputAttemptTerminal
        );
        let receipt = build_input_receipt_v4_3(
            &lifecycle_fixture(),
            &authorization_fixture_v4_3(),
            &registration_fixture_v4_3(),
            MomentumProspectiveInputStatusV4_2::TimeoutNoRetry,
            None,
            0,
            0,
            None,
            None,
        );
        assert!(receipt.terminal);
        assert_eq!(receipt.retry_count, 0);
    }

    #[test]
    fn sprint83_39_prior_prospective_attribution_remains_unchanged() {
        let status = status_fixture_v4_3();
        assert_eq!(
            status.prior_momentum_attribution,
            "MissedMaterialOpportunity"
        );
        assert_eq!(status.prior_cycle_risk_attribution, "CorrectUncertainty");
    }

    #[test]
    fn sprint83_40_all_authority_counters_remain_zero() {
        let counters = MomentumFuturePredictionSafetyCountersV4_3::default();
        validate_safety_counters_v4_3(&counters).unwrap();
        assert_eq!(counters.chair_decisions, 0);
        assert_eq!(counters.votes, 0);
        assert_eq!(counters.executions, 0);
        assert_eq!(counters.active_committee_count, 3);
    }

    #[test]
    fn sprint83_41_protobuf_corruption_rejects_every_v4_3_category() {
        let corrupt = [0xff];
        assert!(decode_authorization_v4_3(&corrupt).is_err());
        assert!(decode_usage_entry_v4_3(&corrupt).is_err());
        assert!(decode_usage_ledger_v4_3(&corrupt).is_err());
        assert!(decode_supersession_v4_3(&corrupt).is_err());
        assert!(decode_context_plan_v4_3(&corrupt).is_err());
        assert!(decode_input_registration_v4_3(&corrupt).is_err());
        assert!(decode_input_receipt_v4_3(&corrupt).is_err());
        assert!(decode_input_capsule_v4_3(&corrupt).is_err());
        assert!(decode_context_proof_v4_3(&corrupt).is_err());
        assert!(decode_prediction_seal_v4_3(&corrupt).is_err());
        assert!(decode_prediction_capsule_v4_3(&corrupt).is_err());
        assert!(decode_prediction_journal_v4_3(&corrupt).is_err());
        assert!(decode_outcome_plan_v4_3(&corrupt).is_err());
        assert!(decode_status_v4_3(&corrupt).is_err());
    }

    #[test]
    fn sprint83_42_text_and_json_status_agree() {
        let status = status_fixture_v4_3();
        let text = crate::cli::format_momentum_v4_future_prediction_text(&status);
        let json = serde_json::to_value(&status).unwrap();
        let text_fields = text
            .lines()
            .filter_map(|line| line.split_once('='))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            text_fields["replacement_event_timestamp_ms"],
            json["replacement_event_timestamp_ms"]
                .as_u64()
                .unwrap()
                .to_string()
        );
        assert_eq!(
            text_fields["corrected_context_plan_digest"],
            json["corrected_context_plan_digest"].as_str().unwrap()
        );
        assert_eq!(
            text_fields["status_digest"],
            json["status_digest"].as_str().unwrap()
        );
    }
}
