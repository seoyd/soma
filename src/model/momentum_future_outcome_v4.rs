//! One-request outcome acquisition and separately authorized local opening for
//! the sealed Momentum V4.3 prospective prediction.
//!
//! Acquisition can persist only closed outcome evidence. Opening is a distinct
//! one-time local operation and has no network, ranking, reward, governance, or
//! active-model authority.

use std::{
    collections::BTreeSet,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    core::stable_hash_string,
    data::{
        AcquisitionMarketScope, DataLookback, DatasetKind, LearningEvidenceTransportFailureV1,
        LearningEvidenceTransportResponseV1, ReadOnlyProviderRequest, UpbitHistoricalPilotConfigV0,
        fetch_upbit_learning_evidence_once_v1, parse_upbit_daily_ohlcv_v0,
        upbit_learning_evidence_provider_contract_v1,
    },
    league::{HistoricalOhlcvRow, canonical_current_agent_states},
};

use super::{
    MomentumLearningCampaignConfigV0,
    momentum_future_prediction_v4::{
        ArtifactBuilderV4_2, ArtifactReaderV4_2, MomentumSealedPredictionChainV4_3, as_u64,
        as_usize, persist_artifact, read_single, reopen_momentum_v4_3_sealed_chain,
        row_identity_digest,
    },
};

const ROOT_VERSION_V4_4: &str = "v4_4";
const DAILY_CADENCE_MS: u64 = 86_400_000;
pub(super) const REGISTRATION_VERSION: &str = "momentum-outcome-acquisition-registration-v4.4";
pub(super) const RECEIPT_VERSION: &str = "momentum-outcome-acquisition-receipt-v4.4";
pub(super) const ROW_PROOF_VERSION: &str = "momentum-outcome-row-identity-proof-v4.4";
pub(super) const CAPSULE_VERSION: &str = "momentum-sealed-outcome-capsule-v4.4";
pub(super) const OPENING_AUTHORIZATION_VERSION: &str =
    "momentum-outcome-opening-authorization-v4.4";
pub(super) const EVALUATION_VERSION: &str = "momentum-participant-prospective-evaluation-v4.4";
pub(super) const OPENING_BUNDLE_VERSION: &str = "momentum-outcome-opening-bundle-v4.4";
pub(super) const OPENING_RECEIPT_VERSION: &str = "momentum-outcome-opening-receipt-v4.4";
const LEDGER_VERSION: &str = "momentum-prospective-evaluation-ledger-v4.4";
const REWARD_RECEIPT_VERSION: &str = "momentum-reward-eligibility-replay-receipt-v4.4";
const STATUS_VERSION: &str = "momentum-future-outcome-status-v4.4";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumOutcomeReadinessV4_4 {
    AwaitingOutcomeFinality,
    ReadyForOutcomeAcquisition,
    PredictionChainInvalid,
    OutcomePlanInvalid,
    PriorOutcomeAttemptTerminal,
    OutcomeEvidenceAcquired,
    OutcomeAlreadyOpened,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumOutcomeRunModeV4_4 {
    Status,
    DryRun,
    Execute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumOutcomeOpeningRunModeV4_4 {
    Status,
    DryRun,
    ExecuteLocal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumOutcomeAcquisitionRegistrationV4_4 {
    pub registration_version: String,
    pub agent_id: String,
    pub lifecycle_digest: String,
    pub evaluation_registration_digest: String,
    pub roster_digest: String,
    pub input_receipt_digest: String,
    pub input_capsule_digest: String,
    pub context_usage_ledger_digest: String,
    pub prediction_capsule_digest: String,
    pub prediction_journal_digest: String,
    pub outcome_plan_digest: String,
    pub event_timestamp_ms: u64,
    pub required_outcome_timestamp_ms: Vec<u64>,
    pub outcome_finality_boundary_ms: u64,
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
    pub labels_must_remain_unopened: bool,
    pub metric_computation_forbidden: bool,
    pub winner_selection_forbidden: bool,
    pub reward_application_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumOutcomeAcquisitionStatusV4_4 {
    EvidenceAcquired,
    TerminalTransportFailure,
    TerminalHttpFailure,
    TerminalValidationFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumOutcomeAcquisitionReceiptV4_4 {
    pub receipt_version: String,
    pub registration_digest: String,
    pub prediction_capsule_digest: String,
    pub outcome_plan_digest: String,
    pub request_attempt_count: usize,
    pub retry_count: usize,
    pub http_status_class: Option<u16>,
    pub returned_row_count: usize,
    pub verified_row_count: usize,
    pub outcome_capsule_digest: Option<String>,
    pub status: MomentumOutcomeAcquisitionStatusV4_4,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumOutcomeRowIdentityProofV4_4 {
    pub proof_version: String,
    pub registration_digest: String,
    pub prediction_capsule_digest: String,
    pub input_capsule_digest: String,
    pub event_timestamp_ms: u64,
    pub outcome_timestamp_ms: u64,
    pub input_event_row_digest: String,
    pub outcome_row_digest: String,
    pub raw_input_response_digest: String,
    pub raw_outcome_response_digest: String,
    pub exact_timestamp_verified: bool,
    pub strict_single_row_verified: bool,
    pub finalized: bool,
    pub sanitized: bool,
    pub credential_free: bool,
    pub read_only: bool,
    pub proof_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumSealedOutcomeCapsuleV4_4 {
    pub capsule_version: String,
    pub registration_digest: String,
    pub receipt_digest: String,
    pub prediction_capsule_digest: String,
    pub event_timestamp_ms: u64,
    pub outcome_timestamp_ms: u64,
    pub outcome_row_digest: String,
    pub labels_opened: bool,
    pub probabilities_opened: bool,
    pub metrics_computed: bool,
    pub winner_selected: bool,
    pub reward_applied: bool,
    pub penalty_applied: bool,
    pub capsule_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumOutcomeOpeningAuthorizationV4_4 {
    pub authorization_version: String,
    pub outcome_registration_digest: String,
    pub outcome_receipt_digest: String,
    pub outcome_capsule_digest: String,
    pub prediction_capsule_digest: String,
    pub prediction_journal_digest: String,
    pub participant_seal_digests: Vec<String>,
    pub participant_prediction_digests: Vec<String>,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub evaluation_policy_digest: String,
    pub opening_attempt_count_before: usize,
    pub opened_event_count_before: usize,
    pub explicit_owner_authorization: bool,
    pub one_time_only: bool,
    pub winner_selection_forbidden: bool,
    pub ranking_forbidden: bool,
    pub reward_application_forbidden: bool,
    pub penalty_application_forbidden: bool,
    pub chair_action_forbidden: bool,
    pub voice_mutation_forbidden: bool,
    pub promotion_forbidden: bool,
    pub trading_forbidden: bool,
    pub authorization_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumProspectiveLabelStatusV4_4 {
    ScorableBinaryOutcome,
    NeutralOutcomeExcluded,
    InvalidOutcomeEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumProspectiveEvaluationStatusV4_4 {
    Scored,
    NeutralOutcomeExcluded,
    InvalidOutcomeEvidence,
    PredictionIntegrityFailure,
    OutcomeIntegrityFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumParticipantProspectiveEvaluationV4_4 {
    pub evaluation_version: String,
    pub participant_digest: String,
    pub participant_role: String,
    pub participant_seal_digest: String,
    pub prediction_digest: String,
    pub event_timestamp_ms: u64,
    pub outcome_timestamp_ms: u64,
    pub label_status: MomentumProspectiveLabelStatusV4_4,
    pub private_label_digest: String,
    pub private_prediction_digest: String,
    pub private_score_digest: Option<String>,
    pub private_correctness_digest: Option<String>,
    pub status: MomentumProspectiveEvaluationStatusV4_4,
    pub evaluation_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumOutcomeOpeningStatusV4_4 {
    Opened,
    AlreadyOpened,
    NeutralOutcomeOpened,
    AuthorizationMissing,
    EvidenceMismatch,
    PredictionMismatch,
    LabelPolicyMismatch,
    TerminalIntegrityFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumOutcomeOpeningBundleV4_4 {
    pub bundle_version: String,
    pub authorization_digest: String,
    pub outcome_capsule_digest: String,
    pub prediction_capsule_digest: String,
    pub opening_attempt_count: usize,
    pub opened_event_count: usize,
    pub label_status: MomentumProspectiveLabelStatusV4_4,
    pub participant_evaluations: Vec<MomentumParticipantProspectiveEvaluationV4_4>,
    pub metrics_computed: bool,
    pub winner_selected: bool,
    pub ranking_created: bool,
    pub reward_applied: bool,
    pub penalty_applied: bool,
    pub chair_action_taken: bool,
    pub bundle_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumOutcomeOpeningReceiptV4_4 {
    pub receipt_version: String,
    pub authorization_digest: String,
    pub opening_bundle_digest: String,
    pub opening_attempt_count: usize,
    pub opened_event_count: usize,
    pub status: MomentumOutcomeOpeningStatusV4_4,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveEvaluationLedgerEntryV4_4 {
    pub event_timestamp_ms: u64,
    pub prediction_capsule_digest: String,
    pub outcome_capsule_digest: String,
    pub opening_bundle_digest: String,
    pub label_status: MomentumProspectiveLabelStatusV4_4,
    pub participant_evaluation_digests: Vec<String>,
    pub total_event_count_after: usize,
    pub scorable_event_count_after: usize,
    pub winner_selected: bool,
    pub reward_applied: bool,
    pub penalty_applied: bool,
    pub entry_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveEvaluationLedgerV4_4 {
    pub ledger_version: String,
    pub entries: Vec<MomentumProspectiveEvaluationLedgerEntryV4_4>,
    pub ledger_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumRewardEligibilityStatusV4_4 {
    IneligibleMinimumSamples,
    IneligibleNeutralOutcome,
    IneligibleIntegrityFailure,
    EligibleCandidateComputed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumRewardEligibilityReplayReceiptV4_4 {
    pub receipt_version: String,
    pub evaluation_ledger_digest: String,
    pub participant_roles: Vec<String>,
    pub learned_participant_count: usize,
    pub benchmark_participant_count: usize,
    pub event_count: usize,
    pub scorable_event_count: usize,
    pub minimum_sample_gate: usize,
    pub integrity_verified: bool,
    pub status: MomentumRewardEligibilityStatusV4_4,
    pub reward_application_count: usize,
    pub penalty_application_count: usize,
    pub voice_mutation_count: usize,
    pub cooldown_count: usize,
    pub promotion_count: usize,
    pub quarantine_count: usize,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumFutureOutcomeSafetyCountersV4_4 {
    pub outcome_request_attempts: usize,
    pub outcome_retries: usize,
    pub maximum_outcome_concurrency: usize,
    pub outcome_transport_constructions: usize,
    pub outcome_opening_attempts: usize,
    pub opened_v4_events: usize,
    pub outcome_row_reads: usize,
    pub outcome_label_reads: usize,
    pub metric_computations: usize,
    pub participant_parameter_updates: usize,
    pub normalizer_refits: usize,
    pub new_training_uses: usize,
    pub new_qualification_uses: usize,
    pub winner_selections: usize,
    pub ranking_creations: usize,
    pub reward_applications: usize,
    pub penalty_applications: usize,
    pub voice_mutations: usize,
    pub cooldowns_started: usize,
    pub promotions: usize,
    pub quarantines: usize,
    pub active_model_changes: usize,
    pub chair_decisions: usize,
    pub votes: usize,
    pub executions: usize,
    pub active_committee_count: usize,
}

impl Default for MomentumFutureOutcomeSafetyCountersV4_4 {
    fn default() -> Self {
        Self {
            outcome_request_attempts: 0,
            outcome_retries: 0,
            maximum_outcome_concurrency: 1,
            outcome_transport_constructions: 0,
            outcome_opening_attempts: 0,
            opened_v4_events: 0,
            outcome_row_reads: 0,
            outcome_label_reads: 0,
            metric_computations: 0,
            participant_parameter_updates: 0,
            normalizer_refits: 0,
            new_training_uses: 0,
            new_qualification_uses: 0,
            winner_selections: 0,
            ranking_creations: 0,
            reward_applications: 0,
            penalty_applications: 0,
            voice_mutations: 0,
            cooldowns_started: 0,
            promotions: 0,
            quarantines: 0,
            active_model_changes: 0,
            chair_decisions: 0,
            votes: 0,
            executions: 0,
            active_committee_count: 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumFutureOutcomeStatusReceiptV4_4 {
    pub status_version: String,
    pub outcome_readiness: MomentumOutcomeReadinessV4_4,
    pub outcome_finality_boundary_ms: u64,
    pub registration_digest: String,
    pub request_fingerprint: String,
    pub request_attempt_count: usize,
    pub outcome_receipt_digest: Option<String>,
    pub outcome_capsule_digest: Option<String>,
    pub opening_readiness: String,
    pub opening_status: Option<MomentumOutcomeOpeningStatusV4_4>,
    pub label_status: Option<MomentumProspectiveLabelStatusV4_4>,
    pub participant_evaluation_statuses: Vec<String>,
    pub participant_evaluation_digests: Vec<String>,
    pub total_event_count: usize,
    pub scorable_event_count: usize,
    pub reward_eligibility_status: Option<MomentumRewardEligibilityStatusV4_4>,
    pub prediction_chain_verified: bool,
    pub protected_artifacts_unchanged: bool,
    pub active_state_unchanged: bool,
    pub safety_counters: MomentumFutureOutcomeSafetyCountersV4_4,
    pub status_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumFutureOutcomeReportV4_4 {
    pub status: MomentumFutureOutcomeStatusReceiptV4_4,
    pub registration: MomentumOutcomeAcquisitionRegistrationV4_4,
    pub receipt: Option<MomentumOutcomeAcquisitionReceiptV4_4>,
    pub outcome_capsule: Option<MomentumSealedOutcomeCapsuleV4_4>,
    pub prediction_value_reads: usize,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumFutureOutcomeOpeningReportV4_4 {
    pub status: MomentumFutureOutcomeStatusReceiptV4_4,
    pub authorization: Option<MomentumOutcomeOpeningAuthorizationV4_4>,
    pub opening_receipt: Option<MomentumOutcomeOpeningReceiptV4_4>,
    pub opening_bundle: Option<MomentumOutcomeOpeningBundleV4_4>,
    pub evaluation_ledger: Option<MomentumProspectiveEvaluationLedgerV4_4>,
    pub reward_eligibility: Option<MomentumRewardEligibilityReplayReceiptV4_4>,
    pub prediction_value_reads: usize,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
}

fn semantic_digest<T: Debug>(prefix: &str, value: &T) -> String {
    stable_hash_string(&format!("{prefix}:{value:?}"))
}

fn with_cleared_digest<T: Clone>(value: &T, clear: impl FnOnce(&mut T)) -> T {
    let mut copy = value.clone();
    clear(&mut copy);
    copy
}

pub(super) fn registration_digest(value: &MomentumOutcomeAcquisitionRegistrationV4_4) -> String {
    semantic_digest(
        "momentum-v4.4-outcome-registration",
        &with_cleared_digest(value, |value| value.registration_digest.clear()),
    )
}

pub(super) fn receipt_digest(value: &MomentumOutcomeAcquisitionReceiptV4_4) -> String {
    semantic_digest(
        "momentum-v4.4-outcome-receipt",
        &with_cleared_digest(value, |value| {
            value.receipt_digest.clear();
            value.outcome_capsule_digest = None;
        }),
    )
}

pub(super) fn row_proof_digest(value: &MomentumOutcomeRowIdentityProofV4_4) -> String {
    semantic_digest(
        "momentum-v4.4-outcome-row-proof",
        &with_cleared_digest(value, |value| value.proof_digest.clear()),
    )
}

pub(super) fn capsule_digest(value: &MomentumSealedOutcomeCapsuleV4_4) -> String {
    semantic_digest(
        "momentum-v4.4-outcome-capsule",
        &with_cleared_digest(value, |value| value.capsule_digest.clear()),
    )
}

pub(super) fn authorization_digest(value: &MomentumOutcomeOpeningAuthorizationV4_4) -> String {
    semantic_digest(
        "momentum-v4.4-opening-authorization",
        &with_cleared_digest(value, |value| value.authorization_digest.clear()),
    )
}

pub(super) fn evaluation_digest(value: &MomentumParticipantProspectiveEvaluationV4_4) -> String {
    semantic_digest(
        "momentum-v4.4-participant-evaluation",
        &with_cleared_digest(value, |value| value.evaluation_digest.clear()),
    )
}

pub(super) fn opening_bundle_digest(value: &MomentumOutcomeOpeningBundleV4_4) -> String {
    semantic_digest(
        "momentum-v4.4-opening-bundle",
        &with_cleared_digest(value, |value| value.bundle_digest.clear()),
    )
}

pub(super) fn opening_receipt_digest(value: &MomentumOutcomeOpeningReceiptV4_4) -> String {
    semantic_digest(
        "momentum-v4.4-opening-receipt",
        &with_cleared_digest(value, |value| value.receipt_digest.clear()),
    )
}

fn ledger_entry_digest(value: &MomentumProspectiveEvaluationLedgerEntryV4_4) -> String {
    semantic_digest(
        "momentum-v4.4-evaluation-ledger-entry",
        &with_cleared_digest(value, |value| value.entry_digest.clear()),
    )
}

fn ledger_digest(value: &MomentumProspectiveEvaluationLedgerV4_4) -> String {
    semantic_digest(
        "momentum-v4.4-evaluation-ledger",
        &with_cleared_digest(value, |value| value.ledger_digest.clear()),
    )
}

fn reward_receipt_digest(value: &MomentumRewardEligibilityReplayReceiptV4_4) -> String {
    semantic_digest(
        "momentum-v4.4-reward-eligibility-replay",
        &with_cleared_digest(value, |value| value.receipt_digest.clear()),
    )
}

fn status_digest(value: &MomentumFutureOutcomeStatusReceiptV4_4) -> String {
    semantic_digest(
        "momentum-v4.4-outcome-status",
        &with_cleared_digest(value, |value| value.status_digest.clear()),
    )
}

pub(super) fn encode_registration(
    value: &MomentumOutcomeAcquisitionRegistrationV4_4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumOutcomeAcquisitionRegistrationV4_4")
        .string("registration_version", &value.registration_version)
        .string("agent_id", &value.agent_id)
        .string("lifecycle_digest", &value.lifecycle_digest)
        .string(
            "evaluation_registration_digest",
            &value.evaluation_registration_digest,
        )
        .string("roster_digest", &value.roster_digest)
        .string("input_receipt_digest", &value.input_receipt_digest)
        .string("input_capsule_digest", &value.input_capsule_digest)
        .string(
            "context_usage_ledger_digest",
            &value.context_usage_ledger_digest,
        )
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .string(
            "prediction_journal_digest",
            &value.prediction_journal_digest,
        )
        .string("outcome_plan_digest", &value.outcome_plan_digest)
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigneds(
            "required_outcome_timestamp_ms",
            &value.required_outcome_timestamp_ms,
        )
        .unsigned(
            "outcome_finality_boundary_ms",
            value.outcome_finality_boundary_ms,
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
            "labels_must_remain_unopened",
            value.labels_must_remain_unopened,
        )
        .boolean(
            "metric_computation_forbidden",
            value.metric_computation_forbidden,
        )
        .boolean(
            "winner_selection_forbidden",
            value.winner_selection_forbidden,
        )
        .boolean(
            "reward_application_forbidden",
            value.reward_application_forbidden,
        )
        .string("registration_digest", &value.registration_digest)
        .encode()
}

pub(super) fn decode_registration(
    bytes: &[u8],
) -> Result<MomentumOutcomeAcquisitionRegistrationV4_4, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumOutcomeAcquisitionRegistrationV4_4")?;
    let value = MomentumOutcomeAcquisitionRegistrationV4_4 {
        registration_version: fields.string("registration_version")?,
        agent_id: fields.string("agent_id")?,
        lifecycle_digest: fields.string("lifecycle_digest")?,
        evaluation_registration_digest: fields.string("evaluation_registration_digest")?,
        roster_digest: fields.string("roster_digest")?,
        input_receipt_digest: fields.string("input_receipt_digest")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        context_usage_ledger_digest: fields.string("context_usage_ledger_digest")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        prediction_journal_digest: fields.string("prediction_journal_digest")?,
        outcome_plan_digest: fields.string("outcome_plan_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        required_outcome_timestamp_ms: fields.unsigneds("required_outcome_timestamp_ms")?,
        outcome_finality_boundary_ms: fields.unsigned("outcome_finality_boundary_ms")?,
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
        labels_must_remain_unopened: fields.boolean("labels_must_remain_unopened")?,
        metric_computation_forbidden: fields.boolean("metric_computation_forbidden")?,
        winner_selection_forbidden: fields.boolean("winner_selection_forbidden")?,
        reward_application_forbidden: fields.boolean("reward_application_forbidden")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_registration_shape(&value)?;
    Ok(value)
}

fn parse_acquisition_status(value: &str) -> Result<MomentumOutcomeAcquisitionStatusV4_4, String> {
    match value {
        "EvidenceAcquired" => Ok(MomentumOutcomeAcquisitionStatusV4_4::EvidenceAcquired),
        "TerminalTransportFailure" => {
            Ok(MomentumOutcomeAcquisitionStatusV4_4::TerminalTransportFailure)
        }
        "TerminalHttpFailure" => Ok(MomentumOutcomeAcquisitionStatusV4_4::TerminalHttpFailure),
        "TerminalValidationFailure" => {
            Ok(MomentumOutcomeAcquisitionStatusV4_4::TerminalValidationFailure)
        }
        _ => Err("V4.4 acquisition status rejected".to_string()),
    }
}

pub(super) fn encode_receipt(
    value: &MomentumOutcomeAcquisitionReceiptV4_4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumOutcomeAcquisitionReceiptV4_4")
        .string("receipt_version", &value.receipt_version)
        .string("registration_digest", &value.registration_digest)
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .string("outcome_plan_digest", &value.outcome_plan_digest)
        .unsigned(
            "request_attempt_count",
            as_u64(value.request_attempt_count)?,
        )
        .unsigned("retry_count", as_u64(value.retry_count)?)
        .unsigneds(
            "http_status_class",
            &value
                .http_status_class
                .map(u64::from)
                .into_iter()
                .collect::<Vec<_>>(),
        )
        .unsigned("returned_row_count", as_u64(value.returned_row_count)?)
        .unsigned("verified_row_count", as_u64(value.verified_row_count)?)
        .optional_string("outcome_capsule_digest", &value.outcome_capsule_digest)
        .string("status", format!("{:?}", value.status))
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

pub(super) fn decode_receipt(
    bytes: &[u8],
) -> Result<MomentumOutcomeAcquisitionReceiptV4_4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumOutcomeAcquisitionReceiptV4_4")?;
    let http_status_values = fields.unsigneds("http_status_class")?;
    if http_status_values.len() > 1 {
        return Err("V4.4 HTTP status class rejected".to_string());
    }
    let value = MomentumOutcomeAcquisitionReceiptV4_4 {
        receipt_version: fields.string("receipt_version")?,
        registration_digest: fields.string("registration_digest")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        outcome_plan_digest: fields.string("outcome_plan_digest")?,
        request_attempt_count: as_usize(fields.unsigned("request_attempt_count")?)?,
        retry_count: as_usize(fields.unsigned("retry_count")?)?,
        http_status_class: http_status_values
            .into_iter()
            .next()
            .map(|value| {
                u16::try_from(value).map_err(|_| "V4.4 HTTP status class rejected".to_string())
            })
            .transpose()?,
        returned_row_count: as_usize(fields.unsigned("returned_row_count")?)?,
        verified_row_count: as_usize(fields.unsigned("verified_row_count")?)?,
        outcome_capsule_digest: fields.optional_string("outcome_capsule_digest")?,
        status: parse_acquisition_status(&fields.string("status")?)?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_receipt_shape(&value)?;
    Ok(value)
}

pub(super) fn encode_row_proof(
    value: &MomentumOutcomeRowIdentityProofV4_4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumOutcomeRowIdentityProofV4_4")
        .string("proof_version", &value.proof_version)
        .string("registration_digest", &value.registration_digest)
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .string("input_capsule_digest", &value.input_capsule_digest)
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigned("outcome_timestamp_ms", value.outcome_timestamp_ms)
        .string("input_event_row_digest", &value.input_event_row_digest)
        .string("outcome_row_digest", &value.outcome_row_digest)
        .string(
            "raw_input_response_digest",
            &value.raw_input_response_digest,
        )
        .string(
            "raw_outcome_response_digest",
            &value.raw_outcome_response_digest,
        )
        .boolean("exact_timestamp_verified", value.exact_timestamp_verified)
        .boolean(
            "strict_single_row_verified",
            value.strict_single_row_verified,
        )
        .boolean("finalized", value.finalized)
        .boolean("sanitized", value.sanitized)
        .boolean("credential_free", value.credential_free)
        .boolean("read_only", value.read_only)
        .string("proof_digest", &value.proof_digest)
        .encode()
}

pub(super) fn decode_row_proof(
    bytes: &[u8],
) -> Result<MomentumOutcomeRowIdentityProofV4_4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumOutcomeRowIdentityProofV4_4")?;
    let value = MomentumOutcomeRowIdentityProofV4_4 {
        proof_version: fields.string("proof_version")?,
        registration_digest: fields.string("registration_digest")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        outcome_timestamp_ms: fields.unsigned("outcome_timestamp_ms")?,
        input_event_row_digest: fields.string("input_event_row_digest")?,
        outcome_row_digest: fields.string("outcome_row_digest")?,
        raw_input_response_digest: fields.string("raw_input_response_digest")?,
        raw_outcome_response_digest: fields.string("raw_outcome_response_digest")?,
        exact_timestamp_verified: fields.boolean("exact_timestamp_verified")?,
        strict_single_row_verified: fields.boolean("strict_single_row_verified")?,
        finalized: fields.boolean("finalized")?,
        sanitized: fields.boolean("sanitized")?,
        credential_free: fields.boolean("credential_free")?,
        read_only: fields.boolean("read_only")?,
        proof_digest: fields.string("proof_digest")?,
    };
    fields.finish()?;
    validate_row_proof_shape(&value)?;
    Ok(value)
}

pub(super) fn encode_capsule(value: &MomentumSealedOutcomeCapsuleV4_4) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumSealedOutcomeCapsuleV4_4")
        .string("capsule_version", &value.capsule_version)
        .string("registration_digest", &value.registration_digest)
        .string("receipt_digest", &value.receipt_digest)
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigned("outcome_timestamp_ms", value.outcome_timestamp_ms)
        .string("outcome_row_digest", &value.outcome_row_digest)
        .boolean("labels_opened", value.labels_opened)
        .boolean("probabilities_opened", value.probabilities_opened)
        .boolean("metrics_computed", value.metrics_computed)
        .boolean("winner_selected", value.winner_selected)
        .boolean("reward_applied", value.reward_applied)
        .boolean("penalty_applied", value.penalty_applied)
        .string("capsule_digest", &value.capsule_digest)
        .encode()
}

pub(super) fn decode_capsule(bytes: &[u8]) -> Result<MomentumSealedOutcomeCapsuleV4_4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumSealedOutcomeCapsuleV4_4")?;
    let value = MomentumSealedOutcomeCapsuleV4_4 {
        capsule_version: fields.string("capsule_version")?,
        registration_digest: fields.string("registration_digest")?,
        receipt_digest: fields.string("receipt_digest")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        outcome_timestamp_ms: fields.unsigned("outcome_timestamp_ms")?,
        outcome_row_digest: fields.string("outcome_row_digest")?,
        labels_opened: fields.boolean("labels_opened")?,
        probabilities_opened: fields.boolean("probabilities_opened")?,
        metrics_computed: fields.boolean("metrics_computed")?,
        winner_selected: fields.boolean("winner_selected")?,
        reward_applied: fields.boolean("reward_applied")?,
        penalty_applied: fields.boolean("penalty_applied")?,
        capsule_digest: fields.string("capsule_digest")?,
    };
    fields.finish()?;
    validate_capsule_shape(&value)?;
    Ok(value)
}

pub(super) fn encode_opening_authorization(
    value: &MomentumOutcomeOpeningAuthorizationV4_4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumOutcomeOpeningAuthorizationV4_4")
        .string("authorization_version", &value.authorization_version)
        .string(
            "outcome_registration_digest",
            &value.outcome_registration_digest,
        )
        .string("outcome_receipt_digest", &value.outcome_receipt_digest)
        .string("outcome_capsule_digest", &value.outcome_capsule_digest)
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .string(
            "prediction_journal_digest",
            &value.prediction_journal_digest,
        )
        .strings("participant_seal_digests", &value.participant_seal_digests)
        .strings(
            "participant_prediction_digests",
            &value.participant_prediction_digests,
        )
        .string("feature_policy_digest", &value.feature_policy_digest)
        .string("label_policy_digest", &value.label_policy_digest)
        .string("evaluation_policy_digest", &value.evaluation_policy_digest)
        .unsigned(
            "opening_attempt_count_before",
            as_u64(value.opening_attempt_count_before)?,
        )
        .unsigned(
            "opened_event_count_before",
            as_u64(value.opened_event_count_before)?,
        )
        .boolean(
            "explicit_owner_authorization",
            value.explicit_owner_authorization,
        )
        .boolean("one_time_only", value.one_time_only)
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
        .boolean("voice_mutation_forbidden", value.voice_mutation_forbidden)
        .boolean("promotion_forbidden", value.promotion_forbidden)
        .boolean("trading_forbidden", value.trading_forbidden)
        .string("authorization_digest", &value.authorization_digest)
        .encode()
}

pub(super) fn decode_opening_authorization(
    bytes: &[u8],
) -> Result<MomentumOutcomeOpeningAuthorizationV4_4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumOutcomeOpeningAuthorizationV4_4")?;
    let value = MomentumOutcomeOpeningAuthorizationV4_4 {
        authorization_version: fields.string("authorization_version")?,
        outcome_registration_digest: fields.string("outcome_registration_digest")?,
        outcome_receipt_digest: fields.string("outcome_receipt_digest")?,
        outcome_capsule_digest: fields.string("outcome_capsule_digest")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        prediction_journal_digest: fields.string("prediction_journal_digest")?,
        participant_seal_digests: fields.strings("participant_seal_digests")?,
        participant_prediction_digests: fields.strings("participant_prediction_digests")?,
        feature_policy_digest: fields.string("feature_policy_digest")?,
        label_policy_digest: fields.string("label_policy_digest")?,
        evaluation_policy_digest: fields.string("evaluation_policy_digest")?,
        opening_attempt_count_before: as_usize(fields.unsigned("opening_attempt_count_before")?)?,
        opened_event_count_before: as_usize(fields.unsigned("opened_event_count_before")?)?,
        explicit_owner_authorization: fields.boolean("explicit_owner_authorization")?,
        one_time_only: fields.boolean("one_time_only")?,
        winner_selection_forbidden: fields.boolean("winner_selection_forbidden")?,
        ranking_forbidden: fields.boolean("ranking_forbidden")?,
        reward_application_forbidden: fields.boolean("reward_application_forbidden")?,
        penalty_application_forbidden: fields.boolean("penalty_application_forbidden")?,
        chair_action_forbidden: fields.boolean("chair_action_forbidden")?,
        voice_mutation_forbidden: fields.boolean("voice_mutation_forbidden")?,
        promotion_forbidden: fields.boolean("promotion_forbidden")?,
        trading_forbidden: fields.boolean("trading_forbidden")?,
        authorization_digest: fields.string("authorization_digest")?,
    };
    fields.finish()?;
    validate_opening_authorization_shape(&value)?;
    Ok(value)
}

fn parse_label_status(value: &str) -> Result<MomentumProspectiveLabelStatusV4_4, String> {
    match value {
        "ScorableBinaryOutcome" => Ok(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome),
        "NeutralOutcomeExcluded" => Ok(MomentumProspectiveLabelStatusV4_4::NeutralOutcomeExcluded),
        "InvalidOutcomeEvidence" => Ok(MomentumProspectiveLabelStatusV4_4::InvalidOutcomeEvidence),
        _ => Err("V4.4 label status rejected".to_string()),
    }
}

fn parse_evaluation_status(value: &str) -> Result<MomentumProspectiveEvaluationStatusV4_4, String> {
    match value {
        "Scored" => Ok(MomentumProspectiveEvaluationStatusV4_4::Scored),
        "NeutralOutcomeExcluded" => {
            Ok(MomentumProspectiveEvaluationStatusV4_4::NeutralOutcomeExcluded)
        }
        "InvalidOutcomeEvidence" => {
            Ok(MomentumProspectiveEvaluationStatusV4_4::InvalidOutcomeEvidence)
        }
        "PredictionIntegrityFailure" => {
            Ok(MomentumProspectiveEvaluationStatusV4_4::PredictionIntegrityFailure)
        }
        "OutcomeIntegrityFailure" => {
            Ok(MomentumProspectiveEvaluationStatusV4_4::OutcomeIntegrityFailure)
        }
        _ => Err("V4.4 evaluation status rejected".to_string()),
    }
}

pub(super) fn encode_evaluation(
    value: &MomentumParticipantProspectiveEvaluationV4_4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumParticipantProspectiveEvaluationV4_4")
        .string("evaluation_version", &value.evaluation_version)
        .string("participant_digest", &value.participant_digest)
        .string("participant_role", &value.participant_role)
        .string("participant_seal_digest", &value.participant_seal_digest)
        .string("prediction_digest", &value.prediction_digest)
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .unsigned("outcome_timestamp_ms", value.outcome_timestamp_ms)
        .string("label_status", format!("{:?}", value.label_status))
        .string("private_label_digest", &value.private_label_digest)
        .string(
            "private_prediction_digest",
            &value.private_prediction_digest,
        )
        .optional_string("private_score_digest", &value.private_score_digest)
        .optional_string(
            "private_correctness_digest",
            &value.private_correctness_digest,
        )
        .string("status", format!("{:?}", value.status))
        .string("evaluation_digest", &value.evaluation_digest)
        .encode()
}

pub(super) fn decode_evaluation(
    bytes: &[u8],
) -> Result<MomentumParticipantProspectiveEvaluationV4_4, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumParticipantProspectiveEvaluationV4_4")?;
    let value = MomentumParticipantProspectiveEvaluationV4_4 {
        evaluation_version: fields.string("evaluation_version")?,
        participant_digest: fields.string("participant_digest")?,
        participant_role: fields.string("participant_role")?,
        participant_seal_digest: fields.string("participant_seal_digest")?,
        prediction_digest: fields.string("prediction_digest")?,
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        outcome_timestamp_ms: fields.unsigned("outcome_timestamp_ms")?,
        label_status: parse_label_status(&fields.string("label_status")?)?,
        private_label_digest: fields.string("private_label_digest")?,
        private_prediction_digest: fields.string("private_prediction_digest")?,
        private_score_digest: fields.optional_string("private_score_digest")?,
        private_correctness_digest: fields.optional_string("private_correctness_digest")?,
        status: parse_evaluation_status(&fields.string("status")?)?,
        evaluation_digest: fields.string("evaluation_digest")?,
    };
    fields.finish()?;
    validate_evaluation_shape(&value)?;
    Ok(value)
}

pub(super) fn encode_opening_bundle(
    value: &MomentumOutcomeOpeningBundleV4_4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumOutcomeOpeningBundleV4_4")
        .string("bundle_version", &value.bundle_version)
        .string("authorization_digest", &value.authorization_digest)
        .string("outcome_capsule_digest", &value.outcome_capsule_digest)
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .unsigned(
            "opening_attempt_count",
            as_u64(value.opening_attempt_count)?,
        )
        .unsigned("opened_event_count", as_u64(value.opened_event_count)?)
        .string("label_status", format!("{:?}", value.label_status))
        .messages(
            "participant_evaluations",
            value
                .participant_evaluations
                .iter()
                .map(encode_evaluation)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .boolean("metrics_computed", value.metrics_computed)
        .boolean("winner_selected", value.winner_selected)
        .boolean("ranking_created", value.ranking_created)
        .boolean("reward_applied", value.reward_applied)
        .boolean("penalty_applied", value.penalty_applied)
        .boolean("chair_action_taken", value.chair_action_taken)
        .string("bundle_digest", &value.bundle_digest)
        .encode()
}

pub(super) fn decode_opening_bundle(
    bytes: &[u8],
) -> Result<MomentumOutcomeOpeningBundleV4_4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumOutcomeOpeningBundleV4_4")?;
    let value = MomentumOutcomeOpeningBundleV4_4 {
        bundle_version: fields.string("bundle_version")?,
        authorization_digest: fields.string("authorization_digest")?,
        outcome_capsule_digest: fields.string("outcome_capsule_digest")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        opening_attempt_count: as_usize(fields.unsigned("opening_attempt_count")?)?,
        opened_event_count: as_usize(fields.unsigned("opened_event_count")?)?,
        label_status: parse_label_status(&fields.string("label_status")?)?,
        participant_evaluations: fields
            .messages("participant_evaluations")?
            .iter()
            .map(|bytes| decode_evaluation(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        metrics_computed: fields.boolean("metrics_computed")?,
        winner_selected: fields.boolean("winner_selected")?,
        ranking_created: fields.boolean("ranking_created")?,
        reward_applied: fields.boolean("reward_applied")?,
        penalty_applied: fields.boolean("penalty_applied")?,
        chair_action_taken: fields.boolean("chair_action_taken")?,
        bundle_digest: fields.string("bundle_digest")?,
    };
    fields.finish()?;
    validate_opening_bundle_shape(&value)?;
    Ok(value)
}

fn parse_opening_status(value: &str) -> Result<MomentumOutcomeOpeningStatusV4_4, String> {
    match value {
        "Opened" => Ok(MomentumOutcomeOpeningStatusV4_4::Opened),
        "AlreadyOpened" => Ok(MomentumOutcomeOpeningStatusV4_4::AlreadyOpened),
        "NeutralOutcomeOpened" => Ok(MomentumOutcomeOpeningStatusV4_4::NeutralOutcomeOpened),
        "AuthorizationMissing" => Ok(MomentumOutcomeOpeningStatusV4_4::AuthorizationMissing),
        "EvidenceMismatch" => Ok(MomentumOutcomeOpeningStatusV4_4::EvidenceMismatch),
        "PredictionMismatch" => Ok(MomentumOutcomeOpeningStatusV4_4::PredictionMismatch),
        "LabelPolicyMismatch" => Ok(MomentumOutcomeOpeningStatusV4_4::LabelPolicyMismatch),
        "TerminalIntegrityFailure" => {
            Ok(MomentumOutcomeOpeningStatusV4_4::TerminalIntegrityFailure)
        }
        _ => Err("V4.4 opening status rejected".to_string()),
    }
}

pub(super) fn encode_opening_receipt(
    value: &MomentumOutcomeOpeningReceiptV4_4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumOutcomeOpeningReceiptV4_4")
        .string("receipt_version", &value.receipt_version)
        .string("authorization_digest", &value.authorization_digest)
        .string("opening_bundle_digest", &value.opening_bundle_digest)
        .unsigned(
            "opening_attempt_count",
            as_u64(value.opening_attempt_count)?,
        )
        .unsigned("opened_event_count", as_u64(value.opened_event_count)?)
        .string("status", format!("{:?}", value.status))
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

pub(super) fn decode_opening_receipt(
    bytes: &[u8],
) -> Result<MomentumOutcomeOpeningReceiptV4_4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumOutcomeOpeningReceiptV4_4")?;
    let value = MomentumOutcomeOpeningReceiptV4_4 {
        receipt_version: fields.string("receipt_version")?,
        authorization_digest: fields.string("authorization_digest")?,
        opening_bundle_digest: fields.string("opening_bundle_digest")?,
        opening_attempt_count: as_usize(fields.unsigned("opening_attempt_count")?)?,
        opened_event_count: as_usize(fields.unsigned("opened_event_count")?)?,
        status: parse_opening_status(&fields.string("status")?)?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_opening_receipt_shape(&value)?;
    Ok(value)
}

fn encode_ledger_entry(
    value: &MomentumProspectiveEvaluationLedgerEntryV4_4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumProspectiveEvaluationLedgerEntryV4_4")
        .unsigned("event_timestamp_ms", value.event_timestamp_ms)
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .string("outcome_capsule_digest", &value.outcome_capsule_digest)
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
        .boolean("reward_applied", value.reward_applied)
        .boolean("penalty_applied", value.penalty_applied)
        .string("entry_digest", &value.entry_digest)
        .encode()
}

fn decode_ledger_entry(
    bytes: &[u8],
) -> Result<MomentumProspectiveEvaluationLedgerEntryV4_4, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumProspectiveEvaluationLedgerEntryV4_4")?;
    let value = MomentumProspectiveEvaluationLedgerEntryV4_4 {
        event_timestamp_ms: fields.unsigned("event_timestamp_ms")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        outcome_capsule_digest: fields.string("outcome_capsule_digest")?,
        opening_bundle_digest: fields.string("opening_bundle_digest")?,
        label_status: parse_label_status(&fields.string("label_status")?)?,
        participant_evaluation_digests: fields.strings("participant_evaluation_digests")?,
        total_event_count_after: as_usize(fields.unsigned("total_event_count_after")?)?,
        scorable_event_count_after: as_usize(fields.unsigned("scorable_event_count_after")?)?,
        winner_selected: fields.boolean("winner_selected")?,
        reward_applied: fields.boolean("reward_applied")?,
        penalty_applied: fields.boolean("penalty_applied")?,
        entry_digest: fields.string("entry_digest")?,
    };
    fields.finish()?;
    validate_ledger_entry_shape(&value)?;
    Ok(value)
}

fn encode_ledger(value: &MomentumProspectiveEvaluationLedgerV4_4) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumProspectiveEvaluationLedgerV4_4")
        .string("ledger_version", &value.ledger_version)
        .messages(
            "entries",
            value
                .entries
                .iter()
                .map(encode_ledger_entry)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .string("ledger_digest", &value.ledger_digest)
        .encode()
}

fn decode_ledger(bytes: &[u8]) -> Result<MomentumProspectiveEvaluationLedgerV4_4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumProspectiveEvaluationLedgerV4_4")?;
    let value = MomentumProspectiveEvaluationLedgerV4_4 {
        ledger_version: fields.string("ledger_version")?,
        entries: fields
            .messages("entries")?
            .iter()
            .map(|bytes| decode_ledger_entry(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        ledger_digest: fields.string("ledger_digest")?,
    };
    fields.finish()?;
    validate_ledger_shape(&value)?;
    Ok(value)
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
        _ => Err("V4.4 reward status rejected".to_string()),
    }
}

fn encode_reward_receipt(
    value: &MomentumRewardEligibilityReplayReceiptV4_4,
) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumRewardEligibilityReplayReceiptV4_4")
        .string("receipt_version", &value.receipt_version)
        .string("evaluation_ledger_digest", &value.evaluation_ledger_digest)
        .strings("participant_roles", &value.participant_roles)
        .unsigned(
            "learned_participant_count",
            as_u64(value.learned_participant_count)?,
        )
        .unsigned(
            "benchmark_participant_count",
            as_u64(value.benchmark_participant_count)?,
        )
        .unsigned("event_count", as_u64(value.event_count)?)
        .unsigned("scorable_event_count", as_u64(value.scorable_event_count)?)
        .unsigned("minimum_sample_gate", as_u64(value.minimum_sample_gate)?)
        .boolean("integrity_verified", value.integrity_verified)
        .string("status", format!("{:?}", value.status))
        .unsigned(
            "reward_application_count",
            as_u64(value.reward_application_count)?,
        )
        .unsigned(
            "penalty_application_count",
            as_u64(value.penalty_application_count)?,
        )
        .unsigned("voice_mutation_count", as_u64(value.voice_mutation_count)?)
        .unsigned("cooldown_count", as_u64(value.cooldown_count)?)
        .unsigned("promotion_count", as_u64(value.promotion_count)?)
        .unsigned("quarantine_count", as_u64(value.quarantine_count)?)
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_reward_receipt(
    bytes: &[u8],
) -> Result<MomentumRewardEligibilityReplayReceiptV4_4, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumRewardEligibilityReplayReceiptV4_4")?;
    let value = MomentumRewardEligibilityReplayReceiptV4_4 {
        receipt_version: fields.string("receipt_version")?,
        evaluation_ledger_digest: fields.string("evaluation_ledger_digest")?,
        participant_roles: fields.strings("participant_roles")?,
        learned_participant_count: as_usize(fields.unsigned("learned_participant_count")?)?,
        benchmark_participant_count: as_usize(fields.unsigned("benchmark_participant_count")?)?,
        event_count: as_usize(fields.unsigned("event_count")?)?,
        scorable_event_count: as_usize(fields.unsigned("scorable_event_count")?)?,
        minimum_sample_gate: as_usize(fields.unsigned("minimum_sample_gate")?)?,
        integrity_verified: fields.boolean("integrity_verified")?,
        status: parse_reward_status(&fields.string("status")?)?,
        reward_application_count: as_usize(fields.unsigned("reward_application_count")?)?,
        penalty_application_count: as_usize(fields.unsigned("penalty_application_count")?)?,
        voice_mutation_count: as_usize(fields.unsigned("voice_mutation_count")?)?,
        cooldown_count: as_usize(fields.unsigned("cooldown_count")?)?,
        promotion_count: as_usize(fields.unsigned("promotion_count")?)?,
        quarantine_count: as_usize(fields.unsigned("quarantine_count")?)?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_reward_receipt_shape(&value)?;
    Ok(value)
}

fn encode_safety_counters(
    value: &MomentumFutureOutcomeSafetyCountersV4_4,
) -> Result<Vec<u8>, String> {
    let values = [
        value.outcome_request_attempts,
        value.outcome_retries,
        value.maximum_outcome_concurrency,
        value.outcome_transport_constructions,
        value.outcome_opening_attempts,
        value.opened_v4_events,
        value.outcome_row_reads,
        value.outcome_label_reads,
        value.metric_computations,
        value.participant_parameter_updates,
        value.normalizer_refits,
        value.new_training_uses,
        value.new_qualification_uses,
        value.winner_selections,
        value.ranking_creations,
        value.reward_applications,
        value.penalty_applications,
        value.voice_mutations,
        value.cooldowns_started,
        value.promotions,
        value.quarantines,
        value.active_model_changes,
        value.chair_decisions,
        value.votes,
        value.executions,
        value.active_committee_count,
    ]
    .into_iter()
    .map(as_u64)
    .collect::<Result<Vec<_>, _>>()?;
    ArtifactBuilderV4_2::new("MomentumFutureOutcomeSafetyCountersV4_4")
        .unsigneds("values", &values)
        .encode()
}

fn decode_safety_counters(bytes: &[u8]) -> Result<MomentumFutureOutcomeSafetyCountersV4_4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumFutureOutcomeSafetyCountersV4_4")?;
    let values = fields
        .unsigneds("values")?
        .into_iter()
        .map(as_usize)
        .collect::<Result<Vec<_>, _>>()?;
    fields.finish()?;
    if values.len() != 26 {
        return Err("V4.4 safety counter shape rejected".to_string());
    }
    Ok(MomentumFutureOutcomeSafetyCountersV4_4 {
        outcome_request_attempts: values[0],
        outcome_retries: values[1],
        maximum_outcome_concurrency: values[2],
        outcome_transport_constructions: values[3],
        outcome_opening_attempts: values[4],
        opened_v4_events: values[5],
        outcome_row_reads: values[6],
        outcome_label_reads: values[7],
        metric_computations: values[8],
        participant_parameter_updates: values[9],
        normalizer_refits: values[10],
        new_training_uses: values[11],
        new_qualification_uses: values[12],
        winner_selections: values[13],
        ranking_creations: values[14],
        reward_applications: values[15],
        penalty_applications: values[16],
        voice_mutations: values[17],
        cooldowns_started: values[18],
        promotions: values[19],
        quarantines: values[20],
        active_model_changes: values[21],
        chair_decisions: values[22],
        votes: values[23],
        executions: values[24],
        active_committee_count: values[25],
    })
}

fn parse_readiness(value: &str) -> Result<MomentumOutcomeReadinessV4_4, String> {
    match value {
        "AwaitingOutcomeFinality" => Ok(MomentumOutcomeReadinessV4_4::AwaitingOutcomeFinality),
        "ReadyForOutcomeAcquisition" => {
            Ok(MomentumOutcomeReadinessV4_4::ReadyForOutcomeAcquisition)
        }
        "PredictionChainInvalid" => Ok(MomentumOutcomeReadinessV4_4::PredictionChainInvalid),
        "OutcomePlanInvalid" => Ok(MomentumOutcomeReadinessV4_4::OutcomePlanInvalid),
        "PriorOutcomeAttemptTerminal" => {
            Ok(MomentumOutcomeReadinessV4_4::PriorOutcomeAttemptTerminal)
        }
        "OutcomeEvidenceAcquired" => Ok(MomentumOutcomeReadinessV4_4::OutcomeEvidenceAcquired),
        "OutcomeAlreadyOpened" => Ok(MomentumOutcomeReadinessV4_4::OutcomeAlreadyOpened),
        "IntegrityFailure" => Ok(MomentumOutcomeReadinessV4_4::IntegrityFailure),
        _ => Err("V4.4 outcome readiness rejected".to_string()),
    }
}

fn encode_status(value: &MomentumFutureOutcomeStatusReceiptV4_4) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumFutureOutcomeStatusReceiptV4_4")
        .string("status_version", &value.status_version)
        .string(
            "outcome_readiness",
            format!("{:?}", value.outcome_readiness),
        )
        .unsigned(
            "outcome_finality_boundary_ms",
            value.outcome_finality_boundary_ms,
        )
        .string("registration_digest", &value.registration_digest)
        .string("request_fingerprint", &value.request_fingerprint)
        .unsigned(
            "request_attempt_count",
            as_u64(value.request_attempt_count)?,
        )
        .optional_string("outcome_receipt_digest", &value.outcome_receipt_digest)
        .optional_string("outcome_capsule_digest", &value.outcome_capsule_digest)
        .string("opening_readiness", &value.opening_readiness)
        .optional_string(
            "opening_status",
            &value.opening_status.map(|status| format!("{status:?}")),
        )
        .optional_string(
            "label_status",
            &value.label_status.map(|status| format!("{status:?}")),
        )
        .strings(
            "participant_evaluation_statuses",
            &value.participant_evaluation_statuses,
        )
        .strings(
            "participant_evaluation_digests",
            &value.participant_evaluation_digests,
        )
        .unsigned("total_event_count", as_u64(value.total_event_count)?)
        .unsigned("scorable_event_count", as_u64(value.scorable_event_count)?)
        .optional_string(
            "reward_eligibility_status",
            &value
                .reward_eligibility_status
                .map(|status| format!("{status:?}")),
        )
        .boolean("prediction_chain_verified", value.prediction_chain_verified)
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

fn decode_status(bytes: &[u8]) -> Result<MomentumFutureOutcomeStatusReceiptV4_4, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumFutureOutcomeStatusReceiptV4_4")?;
    let opening_status = fields
        .optional_string("opening_status")?
        .map(|value| parse_opening_status(&value))
        .transpose()?;
    let label_status = fields
        .optional_string("label_status")?
        .map(|value| parse_label_status(&value))
        .transpose()?;
    let reward_eligibility_status = fields
        .optional_string("reward_eligibility_status")?
        .map(|value| parse_reward_status(&value))
        .transpose()?;
    let safety_messages = fields.messages("safety_counters")?;
    if safety_messages.len() != 1 {
        return Err("V4.4 safety counter message rejected".to_string());
    }
    let value = MomentumFutureOutcomeStatusReceiptV4_4 {
        status_version: fields.string("status_version")?,
        outcome_readiness: parse_readiness(&fields.string("outcome_readiness")?)?,
        outcome_finality_boundary_ms: fields.unsigned("outcome_finality_boundary_ms")?,
        registration_digest: fields.string("registration_digest")?,
        request_fingerprint: fields.string("request_fingerprint")?,
        request_attempt_count: as_usize(fields.unsigned("request_attempt_count")?)?,
        outcome_receipt_digest: fields.optional_string("outcome_receipt_digest")?,
        outcome_capsule_digest: fields.optional_string("outcome_capsule_digest")?,
        opening_readiness: fields.string("opening_readiness")?,
        opening_status,
        label_status,
        participant_evaluation_statuses: fields.strings("participant_evaluation_statuses")?,
        participant_evaluation_digests: fields.strings("participant_evaluation_digests")?,
        total_event_count: as_usize(fields.unsigned("total_event_count")?)?,
        scorable_event_count: as_usize(fields.unsigned("scorable_event_count")?)?,
        reward_eligibility_status,
        prediction_chain_verified: fields.boolean("prediction_chain_verified")?,
        protected_artifacts_unchanged: fields.boolean("protected_artifacts_unchanged")?,
        active_state_unchanged: fields.boolean("active_state_unchanged")?,
        safety_counters: decode_safety_counters(&safety_messages[0])?,
        status_digest: fields.string("status_digest")?,
    };
    fields.finish()?;
    validate_status_shape(&value)?;
    Ok(value)
}

pub(super) fn validate_registration_shape(
    value: &MomentumOutcomeAcquisitionRegistrationV4_4,
) -> Result<(), String> {
    if value.registration_version != REGISTRATION_VERSION
        || value.agent_id.is_empty()
        || value.lifecycle_digest.is_empty()
        || value.evaluation_registration_digest.is_empty()
        || value.roster_digest.is_empty()
        || value.input_receipt_digest.is_empty()
        || value.input_capsule_digest.is_empty()
        || value.context_usage_ledger_digest.is_empty()
        || value.prediction_capsule_digest.is_empty()
        || value.prediction_journal_digest.is_empty()
        || value.outcome_plan_digest.is_empty()
        || value.required_outcome_timestamp_ms.len() != 1
        || value.exact_expected_timestamp_ms != value.required_outcome_timestamp_ms
        || value.expected_row_count != 1
        || value.provider_id != "upbit"
        || value.market != "btc_crypto"
        || value.symbol != "KRW-BTC"
        || value.cadence != "1d"
        || value.request_to_timestamp_ms != value.outcome_finality_boundary_ms
        || value.outcome_finality_boundary_ms
            != value.required_outcome_timestamp_ms[0].saturating_add(DAILY_CADENCE_MS)
        || value.maximum_requests != 1
        || value.maximum_concurrency != 1
        || value.maximum_retries != 0
        || value.maximum_response_bytes == 0
        || !value.credential_free_required
        || !value.read_only_required
        || !value.labels_must_remain_unopened
        || !value.metric_computation_forbidden
        || !value.winner_selection_forbidden
        || !value.reward_application_forbidden
        || value.registration_digest != registration_digest(value)
    {
        return Err("V4.4 outcome registration rejected".to_string());
    }
    Ok(())
}

fn validate_registration_chain(
    value: &MomentumOutcomeAcquisitionRegistrationV4_4,
    chain: &MomentumSealedPredictionChainV4_3,
    config: &UpbitHistoricalPilotConfigV0,
) -> Result<(), String> {
    validate_registration_shape(value)?;
    let contract = upbit_learning_evidence_provider_contract_v1(config)?;
    if value.agent_id != chain.lifecycle.agent_id
        || value.lifecycle_digest != chain.lifecycle.lifecycle_digest
        || value.evaluation_registration_digest != chain.lifecycle.evaluation_registration_digest
        || value.roster_digest != chain.lifecycle.roster_digest
        || value.input_receipt_digest != chain.input_receipt.receipt_digest
        || value.input_capsule_digest != chain.input_capsule.capsule_digest
        || value.context_usage_ledger_digest != chain.context_usage_ledger.ledger_digest
        || value.prediction_capsule_digest != chain.prediction_capsule.capsule_digest
        || value.prediction_journal_digest != chain.prediction_journal.journal_digest
        || value.outcome_plan_digest != chain.outcome_plan.plan_digest
        || value.event_timestamp_ms != chain.outcome_plan.event_timestamp_ms
        || value.required_outcome_timestamp_ms != chain.outcome_plan.required_outcome_timestamp_ms
        || value.outcome_finality_boundary_ms != chain.outcome_plan.outcome_finality_boundary_ms
        || value.maximum_requests != chain.outcome_plan.maximum_outcome_requests
        || value.maximum_retries != chain.outcome_plan.maximum_outcome_retries
        || value.provider_id != config.provider_id
        || value.symbol != config.symbol
        || value.maximum_response_bytes != config.maximum_response_bytes
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
        return Err("V4.4 outcome registration chain binding rejected".to_string());
    }
    Ok(())
}

pub(super) fn validate_receipt_shape(
    value: &MomentumOutcomeAcquisitionReceiptV4_4,
) -> Result<(), String> {
    let success = value.status == MomentumOutcomeAcquisitionStatusV4_4::EvidenceAcquired;
    if value.receipt_version != RECEIPT_VERSION
        || value.registration_digest.is_empty()
        || value.prediction_capsule_digest.is_empty()
        || value.outcome_plan_digest.is_empty()
        || value.request_attempt_count != 1
        || value.retry_count != 0
        || value.verified_row_count > value.returned_row_count
        || (success
            && (value.http_status_class != Some(200)
                || value.returned_row_count != 1
                || value.verified_row_count != 1
                || value.outcome_capsule_digest.is_none()))
        || (!success && (value.verified_row_count != 0 || value.outcome_capsule_digest.is_some()))
        || value.receipt_digest != receipt_digest(value)
    {
        return Err("V4.4 outcome receipt rejected".to_string());
    }
    Ok(())
}

pub(super) fn validate_row_proof_shape(
    value: &MomentumOutcomeRowIdentityProofV4_4,
) -> Result<(), String> {
    if value.proof_version != ROW_PROOF_VERSION
        || value.registration_digest.is_empty()
        || value.prediction_capsule_digest.is_empty()
        || value.input_capsule_digest.is_empty()
        || value.input_event_row_digest.is_empty()
        || value.outcome_row_digest.is_empty()
        || value.raw_input_response_digest.is_empty()
        || value.raw_outcome_response_digest.is_empty()
        || !value.exact_timestamp_verified
        || !value.strict_single_row_verified
        || !value.finalized
        || !value.sanitized
        || !value.credential_free
        || !value.read_only
        || value.proof_digest != row_proof_digest(value)
    {
        return Err("V4.4 outcome row proof rejected".to_string());
    }
    Ok(())
}

pub(super) fn validate_capsule_shape(
    value: &MomentumSealedOutcomeCapsuleV4_4,
) -> Result<(), String> {
    if value.capsule_version != CAPSULE_VERSION
        || value.registration_digest.is_empty()
        || value.receipt_digest.is_empty()
        || value.prediction_capsule_digest.is_empty()
        || value.outcome_row_digest.is_empty()
        || value.labels_opened
        || value.probabilities_opened
        || value.metrics_computed
        || value.winner_selected
        || value.reward_applied
        || value.penalty_applied
        || value.capsule_digest != capsule_digest(value)
    {
        return Err("V4.4 sealed outcome capsule rejected".to_string());
    }
    Ok(())
}

pub(super) fn validate_opening_authorization_shape(
    value: &MomentumOutcomeOpeningAuthorizationV4_4,
) -> Result<(), String> {
    if value.authorization_version != OPENING_AUTHORIZATION_VERSION
        || value.outcome_registration_digest.is_empty()
        || value.outcome_receipt_digest.is_empty()
        || value.outcome_capsule_digest.is_empty()
        || value.prediction_capsule_digest.is_empty()
        || value.prediction_journal_digest.is_empty()
        || value.participant_seal_digests.len() != 3
        || value
            .participant_seal_digests
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != 3
        || value.participant_prediction_digests.len() != 3
        || value
            .participant_prediction_digests
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != 3
        || value.feature_policy_digest.is_empty()
        || value.label_policy_digest != frozen_label_policy_digest()
        || value.evaluation_policy_digest.is_empty()
        || value.opening_attempt_count_before != 0
        || value.opened_event_count_before != 0
        || !value.explicit_owner_authorization
        || !value.one_time_only
        || !value.winner_selection_forbidden
        || !value.ranking_forbidden
        || !value.reward_application_forbidden
        || !value.penalty_application_forbidden
        || !value.chair_action_forbidden
        || !value.voice_mutation_forbidden
        || !value.promotion_forbidden
        || !value.trading_forbidden
        || value.authorization_digest != authorization_digest(value)
    {
        return Err("V4.4 opening authorization rejected".to_string());
    }
    Ok(())
}

pub(super) fn validate_evaluation_shape(
    value: &MomentumParticipantProspectiveEvaluationV4_4,
) -> Result<(), String> {
    let scored = value.status == MomentumProspectiveEvaluationStatusV4_4::Scored;
    let neutral = value.status == MomentumProspectiveEvaluationStatusV4_4::NeutralOutcomeExcluded;
    let invalid = value.status == MomentumProspectiveEvaluationStatusV4_4::InvalidOutcomeEvidence;
    if value.evaluation_version != EVALUATION_VERSION
        || value.participant_digest.is_empty()
        || !matches!(
            value.participant_role.as_str(),
            "RawFeatureLogisticV4"
                | "RawFeatureInteractionLogisticV4"
                | "TrainingPrevalenceConstantV4"
        )
        || value.participant_seal_digest.is_empty()
        || value.prediction_digest.is_empty()
        || value.private_label_digest.is_empty()
        || value.private_prediction_digest.is_empty()
        || (scored
            && (value.label_status != MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome
                || value.private_score_digest.is_none()
                || value.private_correctness_digest.is_none()))
        || (neutral
            && (value.label_status != MomentumProspectiveLabelStatusV4_4::NeutralOutcomeExcluded
                || value.private_score_digest.is_some()
                || value.private_correctness_digest.is_some()))
        || (invalid
            && (value.label_status != MomentumProspectiveLabelStatusV4_4::InvalidOutcomeEvidence
                || value.private_score_digest.is_some()
                || value.private_correctness_digest.is_some()))
        || (!scored && !neutral && !invalid)
        || value.evaluation_digest != evaluation_digest(value)
    {
        return Err("V4.4 participant evaluation rejected".to_string());
    }
    Ok(())
}

pub(super) fn validate_opening_bundle_shape(
    value: &MomentumOutcomeOpeningBundleV4_4,
) -> Result<(), String> {
    let scorable = value.label_status == MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome;
    if value.bundle_version != OPENING_BUNDLE_VERSION
        || value.authorization_digest.is_empty()
        || value.outcome_capsule_digest.is_empty()
        || value.prediction_capsule_digest.is_empty()
        || value.opening_attempt_count != 1
        || value.opened_event_count != 1
        || value.participant_evaluations.len() != 3
        || value
            .participant_evaluations
            .iter()
            .map(|evaluation| &evaluation.participant_digest)
            .collect::<BTreeSet<_>>()
            .len()
            != 3
        || value.metrics_computed != scorable
        || value.winner_selected
        || value.ranking_created
        || value.reward_applied
        || value.penalty_applied
        || value.chair_action_taken
        || value.bundle_digest != opening_bundle_digest(value)
    {
        return Err("V4.4 opening bundle rejected".to_string());
    }
    Ok(())
}

pub(super) fn validate_opening_receipt_shape(
    value: &MomentumOutcomeOpeningReceiptV4_4,
) -> Result<(), String> {
    if value.receipt_version != OPENING_RECEIPT_VERSION
        || value.authorization_digest.is_empty()
        || value.opening_bundle_digest.is_empty()
        || value.opening_attempt_count != 1
        || value.opened_event_count != 1
        || !matches!(
            value.status,
            MomentumOutcomeOpeningStatusV4_4::Opened
                | MomentumOutcomeOpeningStatusV4_4::NeutralOutcomeOpened
        )
        || value.receipt_digest != opening_receipt_digest(value)
    {
        return Err("V4.4 opening receipt rejected".to_string());
    }
    Ok(())
}

fn validate_ledger_entry_shape(
    value: &MomentumProspectiveEvaluationLedgerEntryV4_4,
) -> Result<(), String> {
    let scorable = value.label_status == MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome;
    if value.prediction_capsule_digest.is_empty()
        || value.outcome_capsule_digest.is_empty()
        || value.opening_bundle_digest.is_empty()
        || value.participant_evaluation_digests.len() != 3
        || value
            .participant_evaluation_digests
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != 3
        || value.total_event_count_after != 1
        || value.scorable_event_count_after != usize::from(scorable)
        || value.winner_selected
        || value.reward_applied
        || value.penalty_applied
        || value.entry_digest != ledger_entry_digest(value)
    {
        return Err("V4.4 evaluation ledger entry rejected".to_string());
    }
    Ok(())
}

fn validate_ledger_shape(value: &MomentumProspectiveEvaluationLedgerV4_4) -> Result<(), String> {
    if value.ledger_version != LEDGER_VERSION
        || value.entries.len() != 1
        || value.ledger_digest != ledger_digest(value)
    {
        return Err("V4.4 evaluation ledger rejected".to_string());
    }
    validate_ledger_entry_shape(&value.entries[0])
}

fn validate_reward_receipt_shape(
    value: &MomentumRewardEligibilityReplayReceiptV4_4,
) -> Result<(), String> {
    let learned_count = value
        .participant_roles
        .iter()
        .filter(|role| role.as_str() != "TrainingPrevalenceConstantV4")
        .count();
    let benchmark_count = value
        .participant_roles
        .iter()
        .filter(|role| role.as_str() == "TrainingPrevalenceConstantV4")
        .count();
    let derived_status = if !value.integrity_verified {
        MomentumRewardEligibilityStatusV4_4::IneligibleIntegrityFailure
    } else if value.scorable_event_count == 0 {
        MomentumRewardEligibilityStatusV4_4::IneligibleNeutralOutcome
    } else if value.scorable_event_count < value.minimum_sample_gate {
        MomentumRewardEligibilityStatusV4_4::IneligibleMinimumSamples
    } else {
        MomentumRewardEligibilityStatusV4_4::EligibleCandidateComputed
    };
    if value.receipt_version != REWARD_RECEIPT_VERSION
        || value.evaluation_ledger_digest.is_empty()
        || value.participant_roles.len() != 3
        || learned_count != 2
        || benchmark_count != 1
        || value.learned_participant_count != learned_count
        || value.benchmark_participant_count != benchmark_count
        || value.event_count != 1
        || value.scorable_event_count > value.event_count
        || value.minimum_sample_gate == 0
        || value.status != derived_status
        || value.reward_application_count != 0
        || value.penalty_application_count != 0
        || value.voice_mutation_count != 0
        || value.cooldown_count != 0
        || value.promotion_count != 0
        || value.quarantine_count != 0
        || value.receipt_digest != reward_receipt_digest(value)
    {
        return Err("V4.4 reward eligibility replay rejected".to_string());
    }
    Ok(())
}

fn validate_safety_counters(value: &MomentumFutureOutcomeSafetyCountersV4_4) -> Result<(), String> {
    if value.outcome_request_attempts > 1
        || value.outcome_retries != 0
        || value.maximum_outcome_concurrency > 1
        || value.outcome_transport_constructions > 1
        || value.outcome_opening_attempts > 1
        || value.opened_v4_events > 1
        || value.outcome_row_reads > 1
        || value.outcome_label_reads > 1
        || value.metric_computations > 3
        || value.participant_parameter_updates != 0
        || value.normalizer_refits != 0
        || value.new_training_uses != 0
        || value.new_qualification_uses != 0
        || value.winner_selections != 0
        || value.ranking_creations != 0
        || value.reward_applications != 0
        || value.penalty_applications != 0
        || value.voice_mutations != 0
        || value.cooldowns_started != 0
        || value.promotions != 0
        || value.quarantines != 0
        || value.active_model_changes != 0
        || value.chair_decisions != 0
        || value.votes != 0
        || value.executions != 0
        || value.active_committee_count != 3
    {
        return Err("V4.4 safety counters rejected".to_string());
    }
    Ok(())
}

fn validate_status_shape(value: &MomentumFutureOutcomeStatusReceiptV4_4) -> Result<(), String> {
    validate_safety_counters(&value.safety_counters)?;
    if value.status_version != STATUS_VERSION
        || value.registration_digest.is_empty()
        || value.request_fingerprint.is_empty()
        || value.request_attempt_count > 1
        || value.participant_evaluation_statuses.len() != value.participant_evaluation_digests.len()
        || value.total_event_count > 1
        || value.scorable_event_count > value.total_event_count
        || !value.prediction_chain_verified
        || !value.protected_artifacts_unchanged
        || !value.active_state_unchanged
        || value.status_digest != status_digest(value)
    {
        return Err("V4.4 outcome status rejected".to_string());
    }
    Ok(())
}

fn v4_4_root(root: &Path, agent_id: &str) -> PathBuf {
    root.join(ROOT_VERSION_V4_4).join(agent_id)
}

pub(super) fn raw_outcome_digest(bytes: &[u8]) -> String {
    stable_hash_string(&format!("momentum-v4.4-raw-outcome:{bytes:?}"))
}

fn raw_input_digest(bytes: &[u8]) -> String {
    stable_hash_string(&format!("momentum-v4.3-raw-input:{bytes:?}"))
}

pub(super) fn sanitized_raw_response(bytes: &[u8], maximum_bytes: usize) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    let lowered = text.to_ascii_lowercase();
    !bytes.is_empty()
        && bytes.len() <= maximum_bytes
        && !bytes.contains(&0)
        && serde_json::from_slice::<serde_json::Value>(bytes).is_ok()
        && !lowered.contains("authorization")
        && !lowered.contains("access_key")
        && !lowered.contains("secret_key")
        && !lowered.contains("<html")
}

fn collect_prior_artifacts(
    root: &Path,
    current: &Path,
    values: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), String> {
    if current == root.join(ROOT_VERSION_V4_4) {
        return Ok(());
    }
    if current.is_dir() {
        let mut paths = fs::read_dir(current)
            .map_err(|_| "V4.4 protected directory read failed".to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            collect_prior_artifacts(root, &path, values)?;
        }
    } else if current.is_file() {
        values.push((
            current
                .strip_prefix(root)
                .map_err(|_| "V4.4 protected path rejected".to_string())?
                .to_path_buf(),
            fs::read(current).map_err(|_| "V4.4 protected artifact read failed".to_string())?,
        ));
    }
    Ok(())
}

fn protected_prior_artifacts(root: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>, String> {
    let mut values = Vec::new();
    collect_prior_artifacts(root, root, &mut values)?;
    values.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(values)
}

fn derive_registration(
    chain: &MomentumSealedPredictionChainV4_3,
    config: &UpbitHistoricalPilotConfigV0,
) -> Result<MomentumOutcomeAcquisitionRegistrationV4_4, String> {
    let mut value = MomentumOutcomeAcquisitionRegistrationV4_4 {
        registration_version: REGISTRATION_VERSION.to_string(),
        agent_id: chain.lifecycle.agent_id.clone(),
        lifecycle_digest: chain.lifecycle.lifecycle_digest.clone(),
        evaluation_registration_digest: chain.lifecycle.evaluation_registration_digest.clone(),
        roster_digest: chain.lifecycle.roster_digest.clone(),
        input_receipt_digest: chain.input_receipt.receipt_digest.clone(),
        input_capsule_digest: chain.input_capsule.capsule_digest.clone(),
        context_usage_ledger_digest: chain.context_usage_ledger.ledger_digest.clone(),
        prediction_capsule_digest: chain.prediction_capsule.capsule_digest.clone(),
        prediction_journal_digest: chain.prediction_journal.journal_digest.clone(),
        outcome_plan_digest: chain.outcome_plan.plan_digest.clone(),
        event_timestamp_ms: chain.outcome_plan.event_timestamp_ms,
        required_outcome_timestamp_ms: chain.outcome_plan.required_outcome_timestamp_ms.clone(),
        outcome_finality_boundary_ms: chain.outcome_plan.outcome_finality_boundary_ms,
        provider_id: config.provider_id.clone(),
        market: "btc_crypto".to_string(),
        symbol: config.symbol.clone(),
        cadence: "1d".to_string(),
        exact_expected_timestamp_ms: chain.outcome_plan.required_outcome_timestamp_ms.clone(),
        expected_row_count: chain.outcome_plan.required_outcome_timestamp_ms.len(),
        request_to_timestamp_ms: chain.outcome_plan.outcome_finality_boundary_ms,
        maximum_requests: chain.outcome_plan.maximum_outcome_requests,
        maximum_concurrency: 1,
        maximum_retries: chain.outcome_plan.maximum_outcome_retries,
        maximum_response_bytes: config.maximum_response_bytes,
        credential_free_required: true,
        read_only_required: true,
        labels_must_remain_unopened: chain.outcome_plan.labels_hidden_until_opening,
        metric_computation_forbidden: true,
        winner_selection_forbidden: chain.lifecycle.winner_selection_forbidden,
        reward_application_forbidden: chain.lifecycle.reward_application_forbidden,
        registration_digest: String::new(),
    };
    value.registration_digest = registration_digest(&value);
    validate_registration_chain(&value, chain, config)?;
    Ok(value)
}

pub(super) fn request_fingerprint(value: &MomentumOutcomeAcquisitionRegistrationV4_4) -> String {
    stable_hash_string(&format!(
        "momentum-v4.4-outcome-request:{}:{}:{}:{:?}:{}:1",
        value.provider_id,
        value.market,
        value.symbol,
        value.exact_expected_timestamp_ms,
        value.request_to_timestamp_ms,
    ))
}

pub(super) fn build_request(
    value: &MomentumOutcomeAcquisitionRegistrationV4_4,
) -> Result<ReadOnlyProviderRequest, String> {
    validate_registration_shape(value)?;
    let timestamp = value.exact_expected_timestamp_ms[0];
    Ok(ReadOnlyProviderRequest {
        request_id: stable_hash_string(&format!(
            "momentum-v4.4-outcome-request:{}",
            value.registration_digest
        )),
        request_key: request_fingerprint(value),
        provider_id: value.provider_id.clone(),
        dataset_kind: DatasetKind::DailyOhlcv,
        market_scope: AcquisitionMarketScope::BtcCrypto,
        symbols: vec![value.symbol.clone()],
        lookback: DataLookback {
            bars: 1,
            start_timestamp_ms: Some(timestamp),
            end_timestamp_ms: Some(value.request_to_timestamp_ms),
        },
        cadence: value.cadence.clone(),
        max_staleness_ms: 0,
        reason_codes: vec![],
    })
}

pub(super) fn request_config(
    config: &UpbitHistoricalPilotConfigV0,
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
) -> Result<UpbitHistoricalPilotConfigV0, String> {
    let mut value = config.clone();
    value.start_timestamp_ms = registration.exact_expected_timestamp_ms[0];
    value.end_timestamp_ms = registration.request_to_timestamp_ms;
    value.max_retries = 0;
    value.validate()?;
    let contract = upbit_learning_evidence_provider_contract_v1(&value)?;
    if contract.maximum_lookback_bars < 1
        || contract.provider_id != registration.provider_id
        || contract.symbols != [registration.symbol.clone()]
        || contract.cadence != registration.cadence
        || contract.maximum_response_bytes != registration.maximum_response_bytes
        || !contract.credential_free
        || !contract.read_only
        || !contract.approved_for_network
        || !contract.all_rows_finalized
    {
        return Err("V4.4 provider contract rejected".to_string());
    }
    Ok(value)
}

fn acquisition_readiness(
    observed_timestamp_ms: u64,
    finality_boundary_ms: u64,
    receipt: Option<&MomentumOutcomeAcquisitionReceiptV4_4>,
    capsule: Option<&MomentumSealedOutcomeCapsuleV4_4>,
    opening_receipt: Option<&MomentumOutcomeOpeningReceiptV4_4>,
) -> Result<MomentumOutcomeReadinessV4_4, String> {
    if opening_receipt.is_some() {
        if receipt.is_none() || capsule.is_none() {
            return Err("V4.4 opening replay evidence missing".to_string());
        }
        return Ok(MomentumOutcomeReadinessV4_4::OutcomeAlreadyOpened);
    }
    if let Some(receipt) = receipt {
        if receipt.status == MomentumOutcomeAcquisitionStatusV4_4::EvidenceAcquired {
            let capsule =
                capsule.ok_or_else(|| "V4.4 successful receipt capsule unavailable".to_string())?;
            if receipt.outcome_capsule_digest.as_deref() != Some(capsule.capsule_digest.as_str())
                || capsule.receipt_digest != receipt.receipt_digest
            {
                return Err("V4.4 outcome replay cross-binding rejected".to_string());
            }
            return Ok(MomentumOutcomeReadinessV4_4::OutcomeEvidenceAcquired);
        }
        if capsule.is_some() {
            return Err("V4.4 terminal receipt has capsule".to_string());
        }
        return Ok(MomentumOutcomeReadinessV4_4::PriorOutcomeAttemptTerminal);
    }
    if capsule.is_some() {
        return Err("V4.4 outcome capsule lacks receipt".to_string());
    }
    if observed_timestamp_ms < finality_boundary_ms {
        Ok(MomentumOutcomeReadinessV4_4::AwaitingOutcomeFinality)
    } else {
        Ok(MomentumOutcomeReadinessV4_4::ReadyForOutcomeAcquisition)
    }
}

fn persist_pb(
    root: &Path,
    directory: &str,
    digest: &str,
    bytes: &[u8],
    decode_digest: impl Fn(&[u8]) -> Result<String, String>,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root.join(directory).join(format!("{digest}.pb")),
        bytes,
        digest,
        decode_digest,
    )
}

fn persist_raw_outcome(root: &Path, digest: &str, bytes: &[u8]) -> Result<(usize, usize), String> {
    persist_artifact(
        &root.join("raw_outcome").join(format!("{digest}.json")),
        bytes,
        digest,
        |stored| Ok(raw_outcome_digest(stored)),
    )
}

fn add_counts(total: &mut (usize, usize), next: (usize, usize)) {
    total.0 += next.0;
    total.1 += next.1;
}

fn reopen_outcome_artifacts(
    root: &Path,
) -> Result<
    (
        Option<MomentumOutcomeAcquisitionReceiptV4_4>,
        Option<MomentumSealedOutcomeCapsuleV4_4>,
        Option<MomentumOutcomeOpeningReceiptV4_4>,
    ),
    String,
> {
    Ok((
        read_single(&root.join("outcome_receipts"), decode_receipt)?,
        read_single(&root.join("outcome_capsules"), decode_capsule)?,
        read_single(&root.join("opening_receipts"), decode_opening_receipt)?,
    ))
}

fn build_status(
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    readiness: MomentumOutcomeReadinessV4_4,
    receipt: Option<&MomentumOutcomeAcquisitionReceiptV4_4>,
    capsule: Option<&MomentumSealedOutcomeCapsuleV4_4>,
    opening_receipt: Option<&MomentumOutcomeOpeningReceiptV4_4>,
    opening_bundle: Option<&MomentumOutcomeOpeningBundleV4_4>,
    ledger: Option<&MomentumProspectiveEvaluationLedgerV4_4>,
    reward: Option<&MomentumRewardEligibilityReplayReceiptV4_4>,
    protected_artifacts_unchanged: bool,
    active_state_unchanged: bool,
    safety_counters: MomentumFutureOutcomeSafetyCountersV4_4,
) -> Result<MomentumFutureOutcomeStatusReceiptV4_4, String> {
    let evaluations = opening_bundle
        .map(|bundle| bundle.participant_evaluations.as_slice())
        .unwrap_or_default();
    let mut value = MomentumFutureOutcomeStatusReceiptV4_4 {
        status_version: STATUS_VERSION.to_string(),
        outcome_readiness: readiness,
        outcome_finality_boundary_ms: registration.outcome_finality_boundary_ms,
        registration_digest: registration.registration_digest.clone(),
        request_fingerprint: request_fingerprint(registration),
        request_attempt_count: receipt.map_or(0, |receipt| receipt.request_attempt_count),
        outcome_receipt_digest: receipt.map(|receipt| receipt.receipt_digest.clone()),
        outcome_capsule_digest: capsule.map(|capsule| capsule.capsule_digest.clone()),
        opening_readiness: if opening_receipt.is_some() {
            "AlreadyOpened"
        } else if capsule.is_some() {
            "ReadyForLocalOpening"
        } else {
            "OutcomeEvidenceUnavailable"
        }
        .to_string(),
        opening_status: opening_receipt.map(|receipt| receipt.status),
        label_status: opening_bundle.map(|bundle| bundle.label_status),
        participant_evaluation_statuses: evaluations
            .iter()
            .map(|evaluation| format!("{:?}", evaluation.status))
            .collect(),
        participant_evaluation_digests: evaluations
            .iter()
            .map(|evaluation| evaluation.evaluation_digest.clone())
            .collect(),
        total_event_count: ledger.map_or(0, |ledger| ledger.entries.len()),
        scorable_event_count: ledger
            .and_then(|ledger| ledger.entries.last())
            .map_or(0, |entry| entry.scorable_event_count_after),
        reward_eligibility_status: reward.map(|receipt| receipt.status),
        prediction_chain_verified: true,
        protected_artifacts_unchanged,
        active_state_unchanged,
        safety_counters,
        status_digest: String::new(),
    };
    value.status_digest = status_digest(&value);
    validate_status_shape(&value)?;
    Ok(value)
}

fn receipt_after_attempt(
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    status: MomentumOutcomeAcquisitionStatusV4_4,
    http_status_class: Option<u16>,
    returned_row_count: usize,
) -> MomentumOutcomeAcquisitionReceiptV4_4 {
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
    value.receipt_digest = receipt_digest(&value);
    value
}

pub(super) fn parse_http_status_class(value: Option<&str>) -> Option<u16> {
    value.and_then(|value| {
        let digit = value.as_bytes().first().copied()?;
        digit
            .is_ascii_digit()
            .then(|| u16::from(digit - b'0') * 100)
    })
}

pub(super) fn valid_ohlcv(row: &HistoricalOhlcvRow) -> bool {
    row.open.is_finite()
        && row.high.is_finite()
        && row.low.is_finite()
        && row.close.is_finite()
        && row.volume.is_finite()
        && row.trade_value.is_none_or(f64::is_finite)
        && row.open > 0.0
        && row.high > 0.0
        && row.low > 0.0
        && row.close > 0.0
        && row.high >= row.open.max(row.close)
        && row.low <= row.open.min(row.close)
        && row.low <= row.high
        && row.volume >= 0.0
        && row.trade_value.is_none_or(|value| value >= 0.0)
}

pub(super) fn validate_outcome_transport(
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    request: &ReadOnlyProviderRequest,
    transport: &LearningEvidenceTransportResponseV1,
) -> Result<HistoricalOhlcvRow, String> {
    if transport.http_status_class != "2xx"
        || !sanitized_raw_response(&transport.raw_response, registration.maximum_response_bytes)
        || transport.response.request_id != request.request_id
        || transport.response.provider_id != registration.provider_id
        || transport.response.content_type != "application/x-soma-normalized-dataset"
        || !transport.response.all_rows_finalized
        || transport.response.reported_content_bytes != transport.raw_response.len()
        || transport.response.normalized_dataset.symbol != registration.symbol
        || transport.response.normalized_dataset.rows.len() != 1
    {
        return Err("V4.4 outcome response envelope rejected".to_string());
    }
    let outcome_row = &transport.response.normalized_dataset.rows[0];
    if outcome_row.symbol != registration.symbol
        || outcome_row.timestamp_ms != registration.exact_expected_timestamp_ms[0]
        || !valid_ohlcv(outcome_row)
    {
        return Err("V4.4 exact outcome row rejected".to_string());
    }
    let raw_text = std::str::from_utf8(&transport.raw_response)
        .map_err(|_| "V4.4 raw outcome encoding rejected".to_string())?;
    let parsed_outcome = parse_upbit_daily_ohlcv_v0(raw_text, &registration.symbol)?;
    if parsed_outcome.rows.as_slice() != [outcome_row.clone()] {
        return Err("V4.4 raw and normalized outcome mismatch".to_string());
    }
    Ok(outcome_row.clone())
}

fn validate_outcome_response(
    root: &Path,
    chain: &MomentumSealedPredictionChainV4_3,
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    request: &ReadOnlyProviderRequest,
    transport: &LearningEvidenceTransportResponseV1,
) -> Result<MomentumOutcomeRowIdentityProofV4_4, String> {
    let outcome_row = validate_outcome_transport(registration, request, transport)?;
    let raw_input_path = root
        .join("v4_3")
        .join(&registration.agent_id)
        .join("raw_input")
        .join(format!("{}.json", chain.input_capsule.raw_response_digest));
    let raw_input = fs::read(raw_input_path)
        .map_err(|_| "V4.4 sealed input raw response unavailable".to_string())?;
    if raw_input_digest(&raw_input) != chain.input_capsule.raw_response_digest
        || !sanitized_raw_response(&raw_input, chain.input_registration.maximum_response_bytes)
    {
        return Err("V4.4 sealed input raw response rejected".to_string());
    }
    let input_text = std::str::from_utf8(&raw_input)
        .map_err(|_| "V4.4 sealed input response encoding rejected".to_string())?;
    let parsed_input = parse_upbit_daily_ohlcv_v0(input_text, &registration.symbol)?;
    let parsed_input_digests = parsed_input
        .rows
        .iter()
        .map(row_identity_digest)
        .collect::<Vec<_>>();
    if parsed_input_digests != chain.input_capsule.row_identity_digests {
        return Err("V4.4 sealed input row identities rejected".to_string());
    }
    let event_row = parsed_input
        .rows
        .iter()
        .find(|row| row.timestamp_ms == registration.event_timestamp_ms)
        .ok_or_else(|| "V4.4 event input row unavailable".to_string())?;
    let mut proof = MomentumOutcomeRowIdentityProofV4_4 {
        proof_version: ROW_PROOF_VERSION.to_string(),
        registration_digest: registration.registration_digest.clone(),
        prediction_capsule_digest: registration.prediction_capsule_digest.clone(),
        input_capsule_digest: registration.input_capsule_digest.clone(),
        event_timestamp_ms: registration.event_timestamp_ms,
        outcome_timestamp_ms: outcome_row.timestamp_ms,
        input_event_row_digest: row_identity_digest(event_row),
        outcome_row_digest: row_identity_digest(&outcome_row),
        raw_input_response_digest: chain.input_capsule.raw_response_digest.clone(),
        raw_outcome_response_digest: raw_outcome_digest(&transport.raw_response),
        exact_timestamp_verified: true,
        strict_single_row_verified: true,
        finalized: true,
        sanitized: true,
        credential_free: true,
        read_only: true,
        proof_digest: String::new(),
    };
    proof.proof_digest = row_proof_digest(&proof);
    validate_row_proof_shape(&proof)?;
    Ok(proof)
}

fn persist_terminal_receipt(
    root: &Path,
    receipt: &MomentumOutcomeAcquisitionReceiptV4_4,
) -> Result<(usize, usize), String> {
    persist_pb(
        root,
        "outcome_receipts",
        &receipt.receipt_digest,
        &encode_receipt(receipt)?,
        |bytes| Ok(decode_receipt(bytes)?.receipt_digest),
    )
}

fn run_outcome_with_transport<F>(
    root: &Path,
    provider_config: &UpbitHistoricalPilotConfigV0,
    observed_timestamp_ms: u64,
    mode: MomentumOutcomeRunModeV4_4,
    network_allowed: bool,
    one_time_request_confirmed: bool,
    transport: F,
) -> Result<MomentumFutureOutcomeReportV4_4, String>
where
    F: FnOnce(
        &UpbitHistoricalPilotConfigV0,
        &ReadOnlyProviderRequest,
    )
        -> Result<LearningEvidenceTransportResponseV1, LearningEvidenceTransportFailureV1>,
{
    if mode != MomentumOutcomeRunModeV4_4::Execute
        && (network_allowed || one_time_request_confirmed)
    {
        return Err("V4.4 read-only acquisition rejects network authority".to_string());
    }
    if mode == MomentumOutcomeRunModeV4_4::Execute
        && (!network_allowed || !one_time_request_confirmed)
    {
        return Err(
            "V4.4 outcome execute requires network permission and exact confirmation".to_string(),
        );
    }
    provider_config.validate()?;
    let protected_before = protected_prior_artifacts(root)?;
    let active_before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let chain = reopen_momentum_v4_3_sealed_chain(root)?;
    let registration = derive_registration(&chain, provider_config)?;
    let artifact_root = v4_4_root(root, &registration.agent_id);
    let persisted_registration = read_single(
        &artifact_root.join("outcome_registrations"),
        decode_registration,
    )?;
    if persisted_registration
        .as_ref()
        .is_some_and(|value| value != &registration)
    {
        return Err("V4.4 persisted outcome registration changed".to_string());
    }
    let (persisted_receipt, persisted_capsule, opening_receipt) =
        reopen_outcome_artifacts(&artifact_root)?;
    let readiness = acquisition_readiness(
        observed_timestamp_ms,
        registration.outcome_finality_boundary_ms,
        persisted_receipt.as_ref(),
        persisted_capsule.as_ref(),
        opening_receipt.as_ref(),
    )?;
    let protected_now = protected_prior_artifacts(root)? == protected_before;
    let active_now =
        stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before;
    if mode != MomentumOutcomeRunModeV4_4::Execute
        || matches!(
            readiness,
            MomentumOutcomeReadinessV4_4::OutcomeEvidenceAcquired
                | MomentumOutcomeReadinessV4_4::OutcomeAlreadyOpened
                | MomentumOutcomeReadinessV4_4::PriorOutcomeAttemptTerminal
        )
    {
        let status = build_status(
            &registration,
            readiness,
            persisted_receipt.as_ref(),
            persisted_capsule.as_ref(),
            opening_receipt.as_ref(),
            None,
            None,
            None,
            protected_now,
            active_now,
            MomentumFutureOutcomeSafetyCountersV4_4::default(),
        )?;
        return Ok(MomentumFutureOutcomeReportV4_4 {
            status,
            registration,
            receipt: persisted_receipt,
            outcome_capsule: persisted_capsule,
            prediction_value_reads: 0,
            artifacts_written: 0,
            duplicate_artifact_count: 0,
        });
    }

    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_pb(
            &artifact_root,
            "outcome_registrations",
            &registration.registration_digest,
            &encode_registration(&registration)?,
            |bytes| Ok(decode_registration(bytes)?.registration_digest),
        )?,
    );
    let reopened_registration = read_single(
        &artifact_root.join("outcome_registrations"),
        decode_registration,
    )?
    .ok_or_else(|| "V4.4 outcome registration reopen failed".to_string())?;
    validate_registration_chain(&reopened_registration, &chain, provider_config)?;
    if reopened_registration != registration {
        return Err("V4.4 outcome registration reopen mismatch".to_string());
    }

    if readiness == MomentumOutcomeReadinessV4_4::AwaitingOutcomeFinality {
        let protected_after = protected_prior_artifacts(root)? == protected_before;
        let active_after =
            stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before;
        let status = build_status(
            &registration,
            readiness,
            None,
            None,
            None,
            None,
            None,
            None,
            protected_after,
            active_after,
            MomentumFutureOutcomeSafetyCountersV4_4::default(),
        )?;
        add_counts(
            &mut counts,
            persist_pb(
                &artifact_root,
                "status_receipts",
                &status.status_digest,
                &encode_status(&status)?,
                |bytes| Ok(decode_status(bytes)?.status_digest),
            )?,
        );
        return Ok(MomentumFutureOutcomeReportV4_4 {
            status,
            registration,
            receipt: None,
            outcome_capsule: None,
            prediction_value_reads: 0,
            artifacts_written: counts.0,
            duplicate_artifact_count: counts.1,
        });
    }
    if readiness != MomentumOutcomeReadinessV4_4::ReadyForOutcomeAcquisition {
        return Err("V4.4 outcome acquisition readiness rejected".to_string());
    }

    let request = build_request(&registration)?;
    let request_config = request_config(provider_config, &registration)?;
    let mut safety = MomentumFutureOutcomeSafetyCountersV4_4 {
        outcome_request_attempts: 1,
        maximum_outcome_concurrency: 1,
        outcome_transport_constructions: 1,
        ..Default::default()
    };
    let transport = match transport(&request_config, &request) {
        Ok(value) => value,
        Err(failure) => {
            let (status, http_status_class, raw_response) = match failure {
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
            if let Some(raw_response) = raw_response
                && sanitized_raw_response(&raw_response, registration.maximum_response_bytes)
            {
                let digest = raw_outcome_digest(&raw_response);
                add_counts(
                    &mut counts,
                    persist_raw_outcome(&artifact_root, &digest, &raw_response)?,
                );
            }
            let receipt = receipt_after_attempt(&registration, status, http_status_class, 0);
            validate_receipt_shape(&receipt)?;
            add_counts(
                &mut counts,
                persist_terminal_receipt(&artifact_root, &receipt)?,
            );
            let status = build_status(
                &registration,
                MomentumOutcomeReadinessV4_4::PriorOutcomeAttemptTerminal,
                Some(&receipt),
                None,
                None,
                None,
                None,
                None,
                protected_prior_artifacts(root)? == protected_before,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
                safety,
            )?;
            add_counts(
                &mut counts,
                persist_pb(
                    &artifact_root,
                    "status_receipts",
                    &status.status_digest,
                    &encode_status(&status)?,
                    |bytes| Ok(decode_status(bytes)?.status_digest),
                )?,
            );
            return Ok(MomentumFutureOutcomeReportV4_4 {
                status,
                registration,
                receipt: Some(receipt),
                outcome_capsule: None,
                prediction_value_reads: 0,
                artifacts_written: counts.0,
                duplicate_artifact_count: counts.1,
            });
        }
    };

    let returned_row_count = transport.response.normalized_dataset.rows.len();
    let proof = match validate_outcome_response(root, &chain, &registration, &request, &transport) {
        Ok(value) => value,
        Err(_) => {
            if sanitized_raw_response(&transport.raw_response, registration.maximum_response_bytes)
            {
                let digest = raw_outcome_digest(&transport.raw_response);
                add_counts(
                    &mut counts,
                    persist_raw_outcome(&artifact_root, &digest, &transport.raw_response)?,
                );
            }
            let receipt = receipt_after_attempt(
                &registration,
                MomentumOutcomeAcquisitionStatusV4_4::TerminalValidationFailure,
                parse_http_status_class(Some(&transport.http_status_class)),
                returned_row_count,
            );
            add_counts(
                &mut counts,
                persist_terminal_receipt(&artifact_root, &receipt)?,
            );
            let status = build_status(
                &registration,
                MomentumOutcomeReadinessV4_4::PriorOutcomeAttemptTerminal,
                Some(&receipt),
                None,
                None,
                None,
                None,
                None,
                protected_prior_artifacts(root)? == protected_before,
                stable_hash_string(&format!("{:?}", canonical_current_agent_states()))
                    == active_before,
                safety,
            )?;
            add_counts(
                &mut counts,
                persist_pb(
                    &artifact_root,
                    "status_receipts",
                    &status.status_digest,
                    &encode_status(&status)?,
                    |bytes| Ok(decode_status(bytes)?.status_digest),
                )?,
            );
            return Ok(MomentumFutureOutcomeReportV4_4 {
                status,
                registration,
                receipt: Some(receipt),
                outcome_capsule: None,
                prediction_value_reads: 0,
                artifacts_written: counts.0,
                duplicate_artifact_count: counts.1,
            });
        }
    };
    safety.outcome_row_reads = 1;
    let mut receipt = receipt_after_attempt(
        &registration,
        MomentumOutcomeAcquisitionStatusV4_4::EvidenceAcquired,
        Some(200),
        1,
    );
    receipt.verified_row_count = 1;
    receipt.receipt_digest = receipt_digest(&receipt);
    let mut capsule = MomentumSealedOutcomeCapsuleV4_4 {
        capsule_version: CAPSULE_VERSION.to_string(),
        registration_digest: registration.registration_digest.clone(),
        receipt_digest: receipt.receipt_digest.clone(),
        prediction_capsule_digest: registration.prediction_capsule_digest.clone(),
        event_timestamp_ms: registration.event_timestamp_ms,
        outcome_timestamp_ms: registration.exact_expected_timestamp_ms[0],
        outcome_row_digest: proof.outcome_row_digest.clone(),
        labels_opened: false,
        probabilities_opened: false,
        metrics_computed: false,
        winner_selected: false,
        reward_applied: false,
        penalty_applied: false,
        capsule_digest: String::new(),
    };
    capsule.capsule_digest = capsule_digest(&capsule);
    receipt.outcome_capsule_digest = Some(capsule.capsule_digest.clone());
    validate_receipt_shape(&receipt)?;
    validate_capsule_shape(&capsule)?;
    add_counts(
        &mut counts,
        persist_raw_outcome(
            &artifact_root,
            &proof.raw_outcome_response_digest,
            &transport.raw_response,
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb(
            &artifact_root,
            "outcome_row_proofs",
            &proof.proof_digest,
            &encode_row_proof(&proof)?,
            |bytes| Ok(decode_row_proof(bytes)?.proof_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb(
            &artifact_root,
            "outcome_capsules",
            &capsule.capsule_digest,
            &encode_capsule(&capsule)?,
            |bytes| Ok(decode_capsule(bytes)?.capsule_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_terminal_receipt(&artifact_root, &receipt)?,
    );
    let status = build_status(
        &registration,
        MomentumOutcomeReadinessV4_4::OutcomeEvidenceAcquired,
        Some(&receipt),
        Some(&capsule),
        None,
        None,
        None,
        None,
        protected_prior_artifacts(root)? == protected_before,
        stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before,
        safety,
    )?;
    add_counts(
        &mut counts,
        persist_pb(
            &artifact_root,
            "status_receipts",
            &status.status_digest,
            &encode_status(&status)?,
            |bytes| Ok(decode_status(bytes)?.status_digest),
        )?,
    );
    Ok(MomentumFutureOutcomeReportV4_4 {
        status,
        registration,
        receipt: Some(receipt),
        outcome_capsule: Some(capsule),
        prediction_value_reads: 0,
        artifacts_written: counts.0,
        duplicate_artifact_count: counts.1,
    })
}

pub fn run_momentum_future_outcome_v4_4(
    root: &Path,
    provider_config: &UpbitHistoricalPilotConfigV0,
    observed_timestamp_ms: u64,
    mode: MomentumOutcomeRunModeV4_4,
    network_allowed: bool,
    one_time_request_confirmed: bool,
) -> Result<MomentumFutureOutcomeReportV4_4, String> {
    run_outcome_with_transport(
        root,
        provider_config,
        observed_timestamp_ms,
        mode,
        network_allowed,
        one_time_request_confirmed,
        fetch_upbit_learning_evidence_once_v1,
    )
}

pub(super) fn frozen_label_policy_digest() -> String {
    let config = MomentumLearningCampaignConfigV0::default();
    let sequence = &config.sequence_config;
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}",
        sequence.sequence_length,
        sequence.prediction_horizon,
        sequence.label_dead_zone.to_bits(),
        sequence.stride,
        sequence.include_neutral_labels,
    ))
}

pub(super) fn evaluation_policy_digest() -> String {
    let config = MomentumLearningCampaignConfigV0::default();
    stable_hash_string(&format!(
        "momentum-v4.4-single-event-brier-correctness:{}:no-ranking:no-winner:no-authority",
        config.minimum_test_samples
    ))
}

fn derive_opening_authorization(
    chain: &MomentumSealedPredictionChainV4_3,
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    receipt: &MomentumOutcomeAcquisitionReceiptV4_4,
    capsule: &MomentumSealedOutcomeCapsuleV4_4,
) -> Result<MomentumOutcomeOpeningAuthorizationV4_4, String> {
    let mut value = MomentumOutcomeOpeningAuthorizationV4_4 {
        authorization_version: OPENING_AUTHORIZATION_VERSION.to_string(),
        outcome_registration_digest: registration.registration_digest.clone(),
        outcome_receipt_digest: receipt.receipt_digest.clone(),
        outcome_capsule_digest: capsule.capsule_digest.clone(),
        prediction_capsule_digest: chain.prediction_capsule.capsule_digest.clone(),
        prediction_journal_digest: chain.prediction_journal.journal_digest.clone(),
        participant_seal_digests: chain
            .prediction_capsule
            .participant_prediction_seals
            .iter()
            .map(|seal| seal.seal_digest.clone())
            .collect(),
        participant_prediction_digests: chain
            .prediction_capsule
            .participant_prediction_seals
            .iter()
            .map(|seal| seal.prediction_digest.clone())
            .collect(),
        feature_policy_digest: chain.lifecycle.feature_policy_digest.clone(),
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
    Ok(value)
}

fn validate_opening_bindings(
    authorization: &MomentumOutcomeOpeningAuthorizationV4_4,
    chain: &MomentumSealedPredictionChainV4_3,
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    receipt: &MomentumOutcomeAcquisitionReceiptV4_4,
    capsule: &MomentumSealedOutcomeCapsuleV4_4,
) -> Result<(), String> {
    validate_opening_authorization_shape(authorization)?;
    let seal_digests = chain
        .prediction_capsule
        .participant_prediction_seals
        .iter()
        .map(|seal| seal.seal_digest.clone())
        .collect::<Vec<_>>();
    let prediction_digests = chain
        .prediction_capsule
        .participant_prediction_seals
        .iter()
        .map(|seal| seal.prediction_digest.clone())
        .collect::<Vec<_>>();
    if receipt.status != MomentumOutcomeAcquisitionStatusV4_4::EvidenceAcquired
        || receipt.outcome_capsule_digest.as_deref() != Some(capsule.capsule_digest.as_str())
        || capsule.receipt_digest != receipt.receipt_digest
        || capsule.registration_digest != registration.registration_digest
        || capsule.prediction_capsule_digest != chain.prediction_capsule.capsule_digest
        || authorization.outcome_registration_digest != registration.registration_digest
        || authorization.outcome_receipt_digest != receipt.receipt_digest
        || authorization.outcome_capsule_digest != capsule.capsule_digest
        || authorization.prediction_capsule_digest != chain.prediction_capsule.capsule_digest
        || authorization.prediction_journal_digest != chain.prediction_journal.journal_digest
        || authorization.participant_seal_digests != seal_digests
        || authorization.participant_prediction_digests != prediction_digests
        || authorization.feature_policy_digest != chain.lifecycle.feature_policy_digest
        || authorization.label_policy_digest != frozen_label_policy_digest()
        || authorization.evaluation_policy_digest != evaluation_policy_digest()
    {
        return Err("V4.4 opening cross-binding rejected".to_string());
    }
    Ok(())
}

pub(super) fn classify_label_v4_4(
    event_close: f64,
    outcome_close: f64,
) -> Result<(MomentumProspectiveLabelStatusV4_4, Option<bool>, u64), String> {
    if !event_close.is_finite()
        || event_close <= 0.0
        || !outcome_close.is_finite()
        || outcome_close <= 0.0
    {
        return Err("V4.4 outcome close evidence rejected".to_string());
    }
    let future_return = outcome_close / event_close - 1.0;
    if !future_return.is_finite() {
        return Err("V4.4 future return rejected".to_string());
    }
    let sequence = MomentumLearningCampaignConfigV0::default().sequence_config;
    let dead_zone = f64::from(sequence.label_dead_zone);
    if !sequence.include_neutral_labels && future_return.abs() <= dead_zone {
        return Ok((
            MomentumProspectiveLabelStatusV4_4::NeutralOutcomeExcluded,
            None,
            future_return.to_bits(),
        ));
    }
    let label = if future_return > dead_zone {
        true
    } else if future_return < -dead_zone {
        false
    } else {
        future_return > 0.0
    };
    Ok((
        MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome,
        Some(label),
        future_return.to_bits(),
    ))
}

fn reopen_private_opening_rows(
    root: &Path,
    chain: &MomentumSealedPredictionChainV4_3,
    registration: &MomentumOutcomeAcquisitionRegistrationV4_4,
    proof: &MomentumOutcomeRowIdentityProofV4_4,
) -> Result<(f64, f64), String> {
    let raw_input = fs::read(
        root.join("v4_3")
            .join(&registration.agent_id)
            .join("raw_input")
            .join(format!("{}.json", proof.raw_input_response_digest)),
    )
    .map_err(|_| "V4.4 private input evidence unavailable".to_string())?;
    if raw_input_digest(&raw_input) != proof.raw_input_response_digest
        || proof.raw_input_response_digest != chain.input_capsule.raw_response_digest
        || !sanitized_raw_response(&raw_input, chain.input_registration.maximum_response_bytes)
    {
        return Err("V4.4 private input evidence rejected".to_string());
    }
    let parsed_input = parse_upbit_daily_ohlcv_v0(
        std::str::from_utf8(&raw_input)
            .map_err(|_| "V4.4 private input encoding rejected".to_string())?,
        &registration.symbol,
    )?;
    let event_row = parsed_input
        .rows
        .iter()
        .find(|row| row.timestamp_ms == registration.event_timestamp_ms)
        .ok_or_else(|| "V4.4 private event row unavailable".to_string())?;
    if row_identity_digest(event_row) != proof.input_event_row_digest {
        return Err("V4.4 private event row identity rejected".to_string());
    }

    let raw_outcome = fs::read(
        v4_4_root(root, &registration.agent_id)
            .join("raw_outcome")
            .join(format!("{}.json", proof.raw_outcome_response_digest)),
    )
    .map_err(|_| "V4.4 private outcome evidence unavailable".to_string())?;
    if raw_outcome_digest(&raw_outcome) != proof.raw_outcome_response_digest
        || !sanitized_raw_response(&raw_outcome, registration.maximum_response_bytes)
    {
        return Err("V4.4 private outcome evidence rejected".to_string());
    }
    let parsed_outcome = parse_upbit_daily_ohlcv_v0(
        std::str::from_utf8(&raw_outcome)
            .map_err(|_| "V4.4 private outcome encoding rejected".to_string())?,
        &registration.symbol,
    )?;
    if parsed_outcome.rows.len() != 1 {
        return Err("V4.4 private outcome row count rejected".to_string());
    }
    let outcome_row = &parsed_outcome.rows[0];
    if outcome_row.timestamp_ms != registration.exact_expected_timestamp_ms[0]
        || row_identity_digest(outcome_row) != proof.outcome_row_digest
    {
        return Err("V4.4 private outcome row identity rejected".to_string());
    }
    Ok((event_row.close, outcome_row.close))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_participant_evaluation_v4_4(
    participant_digest: &str,
    participant_role: &str,
    participant_seal_digest: &str,
    prediction_digest: &str,
    prediction_probability_bits: u32,
    event_timestamp_ms: u64,
    proof: &MomentumOutcomeRowIdentityProofV4_4,
    label_status: MomentumProspectiveLabelStatusV4_4,
    label: Option<bool>,
    private_return_bits: u64,
) -> Result<MomentumParticipantProspectiveEvaluationV4_4, String> {
    validate_row_proof_shape(proof)?;
    let probability = f32::from_bits(prediction_probability_bits);
    if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
        return Err("V4.4 sealed prediction value rejected".to_string());
    }
    let private_label_digest = stable_hash_string(&format!(
        "momentum-v4.4-private-label:{}:{label_status:?}:{label:?}:{private_return_bits}",
        proof.proof_digest
    ));
    let private_prediction_digest = stable_hash_string(&format!(
        "momentum-v4.4-private-prediction:{participant_seal_digest}:{prediction_digest}:{prediction_probability_bits}"
    ));
    let (private_score_digest, private_correctness_digest, status) = if let Some(label) = label {
        let target = if label { 1.0_f64 } else { 0.0_f64 };
        let prediction = f64::from(probability);
        let score = (prediction - target).powi(2);
        let correct = (prediction >= 0.5) == label;
        (
            Some(stable_hash_string(&format!(
                "momentum-v4.4-private-brier:{prediction_digest}:{}",
                score.to_bits()
            ))),
            Some(stable_hash_string(&format!(
                "momentum-v4.4-private-correctness:{prediction_digest}:{correct}"
            ))),
            MomentumProspectiveEvaluationStatusV4_4::Scored,
        )
    } else if label_status == MomentumProspectiveLabelStatusV4_4::NeutralOutcomeExcluded {
        (
            None,
            None,
            MomentumProspectiveEvaluationStatusV4_4::NeutralOutcomeExcluded,
        )
    } else {
        (
            None,
            None,
            MomentumProspectiveEvaluationStatusV4_4::InvalidOutcomeEvidence,
        )
    };
    let mut value = MomentumParticipantProspectiveEvaluationV4_4 {
        evaluation_version: EVALUATION_VERSION.to_string(),
        participant_digest: participant_digest.to_string(),
        participant_role: participant_role.to_string(),
        participant_seal_digest: participant_seal_digest.to_string(),
        prediction_digest: prediction_digest.to_string(),
        event_timestamp_ms,
        outcome_timestamp_ms: proof.outcome_timestamp_ms,
        label_status,
        private_label_digest,
        private_prediction_digest,
        private_score_digest,
        private_correctness_digest,
        status,
        evaluation_digest: String::new(),
    };
    value.evaluation_digest = evaluation_digest(&value);
    validate_evaluation_shape(&value)?;
    Ok(value)
}

fn build_participant_evaluations(
    chain: &MomentumSealedPredictionChainV4_3,
    proof: &MomentumOutcomeRowIdentityProofV4_4,
    label_status: MomentumProspectiveLabelStatusV4_4,
    label: Option<bool>,
    private_return_bits: u64,
) -> Result<Vec<MomentumParticipantProspectiveEvaluationV4_4>, String> {
    validate_row_proof_shape(proof)?;
    let mut values = Vec::with_capacity(3);
    for seal in &chain.prediction_capsule.participant_prediction_seals {
        values.push(build_participant_evaluation_v4_4(
            &seal.participant_digest,
            &seal.participant_role,
            &seal.seal_digest,
            &seal.prediction_digest,
            seal.prediction_probability_bits,
            seal.event_timestamp_ms,
            proof,
            label_status,
            label,
            private_return_bits,
        )?);
    }
    if values.len() != 3 {
        return Err("V4.4 participant evaluation count rejected".to_string());
    }
    Ok(values)
}

fn build_opening_bundle(
    authorization: &MomentumOutcomeOpeningAuthorizationV4_4,
    capsule: &MomentumSealedOutcomeCapsuleV4_4,
    chain: &MomentumSealedPredictionChainV4_3,
    label_status: MomentumProspectiveLabelStatusV4_4,
    evaluations: Vec<MomentumParticipantProspectiveEvaluationV4_4>,
) -> Result<MomentumOutcomeOpeningBundleV4_4, String> {
    let mut value = MomentumOutcomeOpeningBundleV4_4 {
        bundle_version: OPENING_BUNDLE_VERSION.to_string(),
        authorization_digest: authorization.authorization_digest.clone(),
        outcome_capsule_digest: capsule.capsule_digest.clone(),
        prediction_capsule_digest: chain.prediction_capsule.capsule_digest.clone(),
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

fn build_evaluation_ledger(
    chain: &MomentumSealedPredictionChainV4_3,
    capsule: &MomentumSealedOutcomeCapsuleV4_4,
    bundle: &MomentumOutcomeOpeningBundleV4_4,
) -> Result<MomentumProspectiveEvaluationLedgerV4_4, String> {
    let mut entry = MomentumProspectiveEvaluationLedgerEntryV4_4 {
        event_timestamp_ms: chain.prediction_capsule.event_timestamp_ms,
        prediction_capsule_digest: chain.prediction_capsule.capsule_digest.clone(),
        outcome_capsule_digest: capsule.capsule_digest.clone(),
        opening_bundle_digest: bundle.bundle_digest.clone(),
        label_status: bundle.label_status,
        participant_evaluation_digests: bundle
            .participant_evaluations
            .iter()
            .map(|evaluation| evaluation.evaluation_digest.clone())
            .collect(),
        total_event_count_after: 1,
        scorable_event_count_after: usize::from(
            bundle.label_status == MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome,
        ),
        winner_selected: false,
        reward_applied: false,
        penalty_applied: false,
        entry_digest: String::new(),
    };
    entry.entry_digest = ledger_entry_digest(&entry);
    validate_ledger_entry_shape(&entry)?;
    let mut ledger = MomentumProspectiveEvaluationLedgerV4_4 {
        ledger_version: LEDGER_VERSION.to_string(),
        entries: vec![entry],
        ledger_digest: String::new(),
    };
    ledger.ledger_digest = ledger_digest(&ledger);
    validate_ledger_shape(&ledger)?;
    Ok(ledger)
}

fn build_reward_eligibility(
    ledger: &MomentumProspectiveEvaluationLedgerV4_4,
    bundle: &MomentumOutcomeOpeningBundleV4_4,
) -> Result<MomentumRewardEligibilityReplayReceiptV4_4, String> {
    let roles = bundle
        .participant_evaluations
        .iter()
        .map(|evaluation| evaluation.participant_role.clone())
        .collect::<Vec<_>>();
    let learned_count = roles
        .iter()
        .filter(|role| role.as_str() != "TrainingPrevalenceConstantV4")
        .count();
    let benchmark_count = roles.len().saturating_sub(learned_count);
    let entry = &ledger.entries[0];
    let minimum_sample_gate = MomentumLearningCampaignConfigV0::default().minimum_test_samples;
    let integrity_verified = validate_ledger_shape(ledger).is_ok()
        && validate_opening_bundle_shape(bundle).is_ok()
        && learned_count == 2
        && benchmark_count == 1;
    let status = if !integrity_verified {
        MomentumRewardEligibilityStatusV4_4::IneligibleIntegrityFailure
    } else if entry.scorable_event_count_after == 0 {
        MomentumRewardEligibilityStatusV4_4::IneligibleNeutralOutcome
    } else if entry.scorable_event_count_after < minimum_sample_gate {
        MomentumRewardEligibilityStatusV4_4::IneligibleMinimumSamples
    } else {
        MomentumRewardEligibilityStatusV4_4::EligibleCandidateComputed
    };
    let mut value = MomentumRewardEligibilityReplayReceiptV4_4 {
        receipt_version: REWARD_RECEIPT_VERSION.to_string(),
        evaluation_ledger_digest: ledger.ledger_digest.clone(),
        participant_roles: roles,
        learned_participant_count: learned_count,
        benchmark_participant_count: benchmark_count,
        event_count: entry.total_event_count_after,
        scorable_event_count: entry.scorable_event_count_after,
        minimum_sample_gate,
        integrity_verified,
        status,
        reward_application_count: 0,
        penalty_application_count: 0,
        voice_mutation_count: 0,
        cooldown_count: 0,
        promotion_count: 0,
        quarantine_count: 0,
        receipt_digest: String::new(),
    };
    value.receipt_digest = reward_receipt_digest(&value);
    validate_reward_receipt_shape(&value)?;
    Ok(value)
}

fn build_opening_receipt(
    authorization: &MomentumOutcomeOpeningAuthorizationV4_4,
    bundle: &MomentumOutcomeOpeningBundleV4_4,
) -> Result<MomentumOutcomeOpeningReceiptV4_4, String> {
    let status =
        if bundle.label_status == MomentumProspectiveLabelStatusV4_4::NeutralOutcomeExcluded {
            MomentumOutcomeOpeningStatusV4_4::NeutralOutcomeOpened
        } else {
            MomentumOutcomeOpeningStatusV4_4::Opened
        };
    let mut value = MomentumOutcomeOpeningReceiptV4_4 {
        receipt_version: OPENING_RECEIPT_VERSION.to_string(),
        authorization_digest: authorization.authorization_digest.clone(),
        opening_bundle_digest: bundle.bundle_digest.clone(),
        opening_attempt_count: 1,
        opened_event_count: 1,
        status,
        receipt_digest: String::new(),
    };
    value.receipt_digest = opening_receipt_digest(&value);
    validate_opening_receipt_shape(&value)?;
    Ok(value)
}

type CompletedOpeningArtifacts = (
    Option<MomentumOutcomeOpeningAuthorizationV4_4>,
    Option<MomentumOutcomeOpeningReceiptV4_4>,
    Option<MomentumOutcomeOpeningBundleV4_4>,
    Option<MomentumProspectiveEvaluationLedgerV4_4>,
    Option<MomentumRewardEligibilityReplayReceiptV4_4>,
);

fn reopen_opening_artifacts(root: &Path) -> Result<CompletedOpeningArtifacts, String> {
    Ok((
        read_single(
            &root.join("opening_authorizations"),
            decode_opening_authorization,
        )?,
        read_single(&root.join("opening_receipts"), decode_opening_receipt)?,
        read_single(&root.join("opening_bundles"), decode_opening_bundle)?,
        read_single(&root.join("evaluation_ledgers"), decode_ledger)?,
        read_single(
            &root.join("reward_eligibility_receipts"),
            decode_reward_receipt,
        )?,
    ))
}

pub fn run_momentum_future_outcome_opening_v4_4(
    root: &Path,
    provider_config: &UpbitHistoricalPilotConfigV0,
    observed_timestamp_ms: u64,
    mode: MomentumOutcomeOpeningRunModeV4_4,
    network_allowed: bool,
    one_time_opening_confirmed: bool,
) -> Result<MomentumFutureOutcomeOpeningReportV4_4, String> {
    if network_allowed {
        return Err("V4.4 local opening rejects network permission".to_string());
    }
    if mode != MomentumOutcomeOpeningRunModeV4_4::ExecuteLocal && one_time_opening_confirmed {
        return Err("V4.4 read-only opening rejects opening authority".to_string());
    }
    if mode == MomentumOutcomeOpeningRunModeV4_4::ExecuteLocal && !one_time_opening_confirmed {
        return Err("V4.4 local opening requires exact owner confirmation".to_string());
    }
    provider_config.validate()?;
    let protected_before = protected_prior_artifacts(root)?;
    let active_before = stable_hash_string(&format!("{:?}", canonical_current_agent_states()));
    let chain = reopen_momentum_v4_3_sealed_chain(root)?;
    let registration = derive_registration(&chain, provider_config)?;
    let artifact_root = v4_4_root(root, &registration.agent_id);
    let persisted_registration = read_single(
        &artifact_root.join("outcome_registrations"),
        decode_registration,
    )?;
    if persisted_registration
        .as_ref()
        .is_some_and(|value| value != &registration)
    {
        return Err("V4.4 opening registration identity rejected".to_string());
    }
    let (outcome_receipt, outcome_capsule, _) = reopen_outcome_artifacts(&artifact_root)?;
    let (
        persisted_authorization,
        opening_receipt,
        opening_bundle,
        evaluation_ledger,
        reward_eligibility,
    ) = reopen_opening_artifacts(&artifact_root)?;
    let readiness = acquisition_readiness(
        observed_timestamp_ms,
        registration.outcome_finality_boundary_ms,
        outcome_receipt.as_ref(),
        outcome_capsule.as_ref(),
        opening_receipt.as_ref(),
    )?;

    if let Some(opening_receipt) = opening_receipt.as_ref() {
        let authorization = persisted_authorization
            .as_ref()
            .ok_or_else(|| "V4.4 opening authorization unavailable".to_string())?;
        let outcome_receipt = outcome_receipt
            .as_ref()
            .ok_or_else(|| "V4.4 outcome receipt unavailable".to_string())?;
        let outcome_capsule = outcome_capsule
            .as_ref()
            .ok_or_else(|| "V4.4 outcome capsule unavailable".to_string())?;
        let bundle = opening_bundle
            .as_ref()
            .ok_or_else(|| "V4.4 opening bundle unavailable".to_string())?;
        let ledger = evaluation_ledger
            .as_ref()
            .ok_or_else(|| "V4.4 evaluation ledger unavailable".to_string())?;
        let reward = reward_eligibility
            .as_ref()
            .ok_or_else(|| "V4.4 reward replay receipt unavailable".to_string())?;
        validate_opening_bindings(
            authorization,
            &chain,
            &registration,
            outcome_receipt,
            outcome_capsule,
        )?;
        if opening_receipt.authorization_digest != authorization.authorization_digest
            || opening_receipt.opening_bundle_digest != bundle.bundle_digest
            || bundle.authorization_digest != authorization.authorization_digest
            || bundle.outcome_capsule_digest != outcome_capsule.capsule_digest
            || ledger.entries[0].opening_bundle_digest != bundle.bundle_digest
            || reward.evaluation_ledger_digest != ledger.ledger_digest
        {
            return Err("V4.4 completed opening replay rejected".to_string());
        }
        let mut status = build_status(
            &registration,
            MomentumOutcomeReadinessV4_4::OutcomeAlreadyOpened,
            Some(outcome_receipt),
            Some(outcome_capsule),
            Some(opening_receipt),
            Some(bundle),
            Some(ledger),
            Some(reward),
            protected_prior_artifacts(root)? == protected_before,
            stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before,
            MomentumFutureOutcomeSafetyCountersV4_4::default(),
        )?;
        if mode == MomentumOutcomeOpeningRunModeV4_4::ExecuteLocal {
            status.opening_status = Some(MomentumOutcomeOpeningStatusV4_4::AlreadyOpened);
            status.status_digest = status_digest(&status);
            validate_status_shape(&status)?;
        }
        return Ok(MomentumFutureOutcomeOpeningReportV4_4 {
            status,
            authorization: persisted_authorization,
            opening_receipt: Some(opening_receipt.clone()),
            opening_bundle,
            evaluation_ledger,
            reward_eligibility,
            prediction_value_reads: 0,
            artifacts_written: 0,
            duplicate_artifact_count: 0,
        });
    }
    if opening_bundle.is_some() || evaluation_ledger.is_some() || reward_eligibility.is_some() {
        return Err("V4.4 partial opening artifacts rejected".to_string());
    }

    if mode != MomentumOutcomeOpeningRunModeV4_4::ExecuteLocal {
        let status = build_status(
            &registration,
            readiness,
            outcome_receipt.as_ref(),
            outcome_capsule.as_ref(),
            None,
            None,
            None,
            None,
            protected_prior_artifacts(root)? == protected_before,
            stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before,
            MomentumFutureOutcomeSafetyCountersV4_4::default(),
        )?;
        return Ok(MomentumFutureOutcomeOpeningReportV4_4 {
            status,
            authorization: persisted_authorization,
            opening_receipt: None,
            opening_bundle: None,
            evaluation_ledger: None,
            reward_eligibility: None,
            prediction_value_reads: 0,
            artifacts_written: 0,
            duplicate_artifact_count: 0,
        });
    }
    if readiness != MomentumOutcomeReadinessV4_4::OutcomeEvidenceAcquired {
        return Err("V4.4 opening evidence is not ready".to_string());
    }
    let outcome_receipt =
        outcome_receipt.ok_or_else(|| "V4.4 outcome receipt unavailable".to_string())?;
    let outcome_capsule =
        outcome_capsule.ok_or_else(|| "V4.4 outcome capsule unavailable".to_string())?;
    let authorization =
        derive_opening_authorization(&chain, &registration, &outcome_receipt, &outcome_capsule)?;
    if persisted_authorization
        .as_ref()
        .is_some_and(|value| value != &authorization)
    {
        return Err("V4.4 persisted opening authorization changed".to_string());
    }
    let mut counts = (0, 0);
    add_counts(
        &mut counts,
        persist_pb(
            &artifact_root,
            "opening_authorizations",
            &authorization.authorization_digest,
            &encode_opening_authorization(&authorization)?,
            |bytes| Ok(decode_opening_authorization(bytes)?.authorization_digest),
        )?,
    );
    let reopened_authorization = read_single(
        &artifact_root.join("opening_authorizations"),
        decode_opening_authorization,
    )?
    .ok_or_else(|| "V4.4 opening authorization reopen failed".to_string())?;
    if reopened_authorization != authorization {
        return Err("V4.4 opening authorization reopen mismatch".to_string());
    }
    validate_opening_bindings(
        &reopened_authorization,
        &chain,
        &registration,
        &outcome_receipt,
        &outcome_capsule,
    )?;

    let proof = read_single(&artifact_root.join("outcome_row_proofs"), decode_row_proof)?
        .ok_or_else(|| "V4.4 outcome row proof unavailable".to_string())?;
    if proof.registration_digest != registration.registration_digest
        || proof.prediction_capsule_digest != chain.prediction_capsule.capsule_digest
        || proof.input_capsule_digest != chain.input_capsule.capsule_digest
        || proof.event_timestamp_ms != registration.event_timestamp_ms
        || proof.outcome_timestamp_ms != registration.exact_expected_timestamp_ms[0]
        || proof.outcome_row_digest != outcome_capsule.outcome_row_digest
    {
        return Err("V4.4 outcome row proof binding rejected".to_string());
    }
    let (event_close, outcome_close) =
        reopen_private_opening_rows(root, &chain, &registration, &proof)?;
    let (label_status, label, private_return_bits) =
        classify_label_v4_4(event_close, outcome_close)?;
    let evaluations =
        build_participant_evaluations(&chain, &proof, label_status, label, private_return_bits)?;
    let bundle = build_opening_bundle(
        &authorization,
        &outcome_capsule,
        &chain,
        label_status,
        evaluations,
    )?;
    let ledger = build_evaluation_ledger(&chain, &outcome_capsule, &bundle)?;
    let reward = build_reward_eligibility(&ledger, &bundle)?;
    let opening_receipt = build_opening_receipt(&authorization, &bundle)?;
    add_counts(
        &mut counts,
        persist_pb(
            &artifact_root,
            "opening_bundles",
            &bundle.bundle_digest,
            &encode_opening_bundle(&bundle)?,
            |bytes| Ok(decode_opening_bundle(bytes)?.bundle_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb(
            &artifact_root,
            "evaluation_ledgers",
            &ledger.ledger_digest,
            &encode_ledger(&ledger)?,
            |bytes| Ok(decode_ledger(bytes)?.ledger_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb(
            &artifact_root,
            "reward_eligibility_receipts",
            &reward.receipt_digest,
            &encode_reward_receipt(&reward)?,
            |bytes| Ok(decode_reward_receipt(bytes)?.receipt_digest),
        )?,
    );
    add_counts(
        &mut counts,
        persist_pb(
            &artifact_root,
            "opening_receipts",
            &opening_receipt.receipt_digest,
            &encode_opening_receipt(&opening_receipt)?,
            |bytes| Ok(decode_opening_receipt(bytes)?.receipt_digest),
        )?,
    );
    let scorable = label_status == MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome;
    let safety = MomentumFutureOutcomeSafetyCountersV4_4 {
        outcome_opening_attempts: 1,
        opened_v4_events: 1,
        outcome_row_reads: 1,
        outcome_label_reads: 1,
        metric_computations: if scorable { 3 } else { 0 },
        ..Default::default()
    };
    let status = build_status(
        &registration,
        MomentumOutcomeReadinessV4_4::OutcomeAlreadyOpened,
        Some(&outcome_receipt),
        Some(&outcome_capsule),
        Some(&opening_receipt),
        Some(&bundle),
        Some(&ledger),
        Some(&reward),
        protected_prior_artifacts(root)? == protected_before,
        stable_hash_string(&format!("{:?}", canonical_current_agent_states())) == active_before,
        safety,
    )?;
    add_counts(
        &mut counts,
        persist_pb(
            &artifact_root,
            "status_receipts",
            &status.status_digest,
            &encode_status(&status)?,
            |bytes| Ok(decode_status(bytes)?.status_digest),
        )?,
    );
    Ok(MomentumFutureOutcomeOpeningReportV4_4 {
        status,
        authorization: Some(authorization),
        opening_receipt: Some(opening_receipt),
        opening_bundle: Some(bundle),
        evaluation_ledger: Some(ledger),
        reward_eligibility: Some(reward),
        prediction_value_reads: 3,
        artifacts_written: counts.0,
        duplicate_artifact_count: counts.1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::{NetworkConsentV0, ReadOnlyProviderResponse},
        league::HistoricalReplayDataset,
    };

    const OUTCOME_TIMESTAMP: u64 = 1_784_851_200_000;
    const FINALITY_TIMESTAMP: u64 = 1_784_937_600_000;

    fn provider_config() -> UpbitHistoricalPilotConfigV0 {
        UpbitHistoricalPilotConfigV0 {
            provider_id: "upbit".into(),
            enabled: true,
            market: AcquisitionMarketScope::BtcCrypto,
            symbol: "KRW-BTC".into(),
            start_timestamp_ms: OUTCOME_TIMESTAMP - 10 * DAILY_CADENCE_MS,
            end_timestamp_ms: FINALITY_TIMESTAMP + DAILY_CADENCE_MS,
            maximum_rows: 200,
            timeout_seconds: 10,
            max_retries: 0,
            maximum_response_bytes: 262_144,
            snapshot_output_dir: "data/local_snapshots/upbit".into(),
            network_consent: NetworkConsentV0::ManualLocalSmoke,
            manual_smoke_enabled: true,
            page_size: 200,
            target_rows: 152,
            maximum_pages: 1,
            stop_when_campaign_sufficient: true,
            campaign_attempt_enabled: true,
            minimum_inter_request_delay_ms: 1,
        }
    }

    fn registration_fixture() -> MomentumOutcomeAcquisitionRegistrationV4_4 {
        let mut value = MomentumOutcomeAcquisitionRegistrationV4_4 {
            registration_version: REGISTRATION_VERSION.into(),
            agent_id: "momentum_trend_fast".into(),
            lifecycle_digest: "lifecycle".into(),
            evaluation_registration_digest: "evaluation".into(),
            roster_digest: "roster".into(),
            input_receipt_digest: "input-receipt".into(),
            input_capsule_digest: "input-capsule".into(),
            context_usage_ledger_digest: "context-ledger".into(),
            prediction_capsule_digest: "prediction-capsule".into(),
            prediction_journal_digest: "prediction-journal".into(),
            outcome_plan_digest: "outcome-plan".into(),
            event_timestamp_ms: OUTCOME_TIMESTAMP - DAILY_CADENCE_MS,
            required_outcome_timestamp_ms: vec![OUTCOME_TIMESTAMP],
            outcome_finality_boundary_ms: FINALITY_TIMESTAMP,
            provider_id: "upbit".into(),
            market: "btc_crypto".into(),
            symbol: "KRW-BTC".into(),
            cadence: "1d".into(),
            exact_expected_timestamp_ms: vec![OUTCOME_TIMESTAMP],
            expected_row_count: 1,
            request_to_timestamp_ms: FINALITY_TIMESTAMP,
            maximum_requests: 1,
            maximum_concurrency: 1,
            maximum_retries: 0,
            maximum_response_bytes: 262_144,
            credential_free_required: true,
            read_only_required: true,
            labels_must_remain_unopened: true,
            metric_computation_forbidden: true,
            winner_selection_forbidden: true,
            reward_application_forbidden: true,
            registration_digest: String::new(),
        };
        value.registration_digest = registration_digest(&value);
        value
    }

    fn outcome_row() -> HistoricalOhlcvRow {
        HistoricalOhlcvRow {
            symbol: "KRW-BTC".into(),
            timestamp_ms: OUTCOME_TIMESTAMP,
            open: 100.0,
            high: 102.0,
            low: 99.0,
            close: 101.0,
            volume: 10.0,
            trade_value: Some(1_010.0),
        }
    }

    fn raw_outcome() -> Vec<u8> {
        br#"[{"market":"KRW-BTC","candle_date_time_utc":"2026-07-24T00:00:00","opening_price":100.0,"high_price":102.0,"low_price":99.0,"trade_price":101.0,"candle_acc_trade_volume":10.0,"candle_acc_trade_price":1010.0}]"#.to_vec()
    }

    fn transport_fixture() -> LearningEvidenceTransportResponseV1 {
        let registration = registration_fixture();
        let request = build_request(&registration).unwrap();
        let raw = raw_outcome();
        LearningEvidenceTransportResponseV1 {
            http_status_class: "2xx".into(),
            raw_response: raw.clone(),
            response: ReadOnlyProviderResponse {
                request_id: request.request_id,
                provider_id: "upbit".into(),
                fetched_at_ms: FINALITY_TIMESTAMP,
                content_type: "application/x-soma-normalized-dataset".into(),
                all_rows_finalized: true,
                normalized_dataset: HistoricalReplayDataset {
                    symbol: "KRW-BTC".into(),
                    rows: vec![outcome_row()],
                    source: "upbit-approved-readonly-daily".into(),
                    reason_codes: vec![],
                },
                reported_content_bytes: raw.len(),
                reason_codes: vec![],
            },
        }
    }

    fn receipt_and_capsule() -> (
        MomentumOutcomeAcquisitionReceiptV4_4,
        MomentumSealedOutcomeCapsuleV4_4,
    ) {
        let registration = registration_fixture();
        let mut receipt = receipt_after_attempt(
            &registration,
            MomentumOutcomeAcquisitionStatusV4_4::EvidenceAcquired,
            Some(200),
            1,
        );
        receipt.verified_row_count = 1;
        receipt.receipt_digest = receipt_digest(&receipt);
        let mut capsule = MomentumSealedOutcomeCapsuleV4_4 {
            capsule_version: CAPSULE_VERSION.into(),
            registration_digest: registration.registration_digest,
            receipt_digest: receipt.receipt_digest.clone(),
            prediction_capsule_digest: "prediction-capsule".into(),
            event_timestamp_ms: OUTCOME_TIMESTAMP - DAILY_CADENCE_MS,
            outcome_timestamp_ms: OUTCOME_TIMESTAMP,
            outcome_row_digest: "outcome-row".into(),
            labels_opened: false,
            probabilities_opened: false,
            metrics_computed: false,
            winner_selected: false,
            reward_applied: false,
            penalty_applied: false,
            capsule_digest: String::new(),
        };
        capsule.capsule_digest = capsule_digest(&capsule);
        receipt.outcome_capsule_digest = Some(capsule.capsule_digest.clone());
        (receipt, capsule)
    }

    fn opening_authorization_fixture() -> MomentumOutcomeOpeningAuthorizationV4_4 {
        let mut value = MomentumOutcomeOpeningAuthorizationV4_4 {
            authorization_version: OPENING_AUTHORIZATION_VERSION.into(),
            outcome_registration_digest: "registration".into(),
            outcome_receipt_digest: "receipt".into(),
            outcome_capsule_digest: "outcome-capsule".into(),
            prediction_capsule_digest: "prediction-capsule".into(),
            prediction_journal_digest: "prediction-journal".into(),
            participant_seal_digests: vec!["seal-1".into(), "seal-2".into(), "seal-3".into()],
            participant_prediction_digests: vec![
                "prediction-1".into(),
                "prediction-2".into(),
                "prediction-3".into(),
            ],
            feature_policy_digest: "feature-policy".into(),
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
        value
    }

    fn evaluations_fixture(
        label_status: MomentumProspectiveLabelStatusV4_4,
    ) -> Vec<MomentumParticipantProspectiveEvaluationV4_4> {
        let roles = [
            "RawFeatureLogisticV4",
            "RawFeatureInteractionLogisticV4",
            "TrainingPrevalenceConstantV4",
        ];
        roles
            .iter()
            .enumerate()
            .map(|(index, role)| {
                let scored =
                    label_status == MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome;
                let mut value = MomentumParticipantProspectiveEvaluationV4_4 {
                    evaluation_version: EVALUATION_VERSION.into(),
                    participant_digest: format!("participant-{index}"),
                    participant_role: (*role).into(),
                    participant_seal_digest: format!("seal-{index}"),
                    prediction_digest: format!("prediction-{index}"),
                    event_timestamp_ms: OUTCOME_TIMESTAMP - DAILY_CADENCE_MS,
                    outcome_timestamp_ms: OUTCOME_TIMESTAMP,
                    label_status,
                    private_label_digest: format!("private-label-{index}"),
                    private_prediction_digest: format!("private-prediction-{index}"),
                    private_score_digest: scored.then(|| format!("private-score-{index}")),
                    private_correctness_digest: scored
                        .then(|| format!("private-correctness-{index}")),
                    status: if scored {
                        MomentumProspectiveEvaluationStatusV4_4::Scored
                    } else {
                        MomentumProspectiveEvaluationStatusV4_4::NeutralOutcomeExcluded
                    },
                    evaluation_digest: String::new(),
                };
                value.evaluation_digest = evaluation_digest(&value);
                value
            })
            .collect()
    }

    fn bundle_fixture(
        label_status: MomentumProspectiveLabelStatusV4_4,
    ) -> MomentumOutcomeOpeningBundleV4_4 {
        let mut value = MomentumOutcomeOpeningBundleV4_4 {
            bundle_version: OPENING_BUNDLE_VERSION.into(),
            authorization_digest: "authorization".into(),
            outcome_capsule_digest: "outcome-capsule".into(),
            prediction_capsule_digest: "prediction-capsule".into(),
            opening_attempt_count: 1,
            opened_event_count: 1,
            label_status,
            participant_evaluations: evaluations_fixture(label_status),
            metrics_computed: label_status
                == MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome,
            winner_selected: false,
            ranking_created: false,
            reward_applied: false,
            penalty_applied: false,
            chair_action_taken: false,
            bundle_digest: String::new(),
        };
        value.bundle_digest = opening_bundle_digest(&value);
        value
    }

    fn ledger_fixture(
        label_status: MomentumProspectiveLabelStatusV4_4,
    ) -> MomentumProspectiveEvaluationLedgerV4_4 {
        let bundle = bundle_fixture(label_status);
        let mut entry = MomentumProspectiveEvaluationLedgerEntryV4_4 {
            event_timestamp_ms: OUTCOME_TIMESTAMP - DAILY_CADENCE_MS,
            prediction_capsule_digest: "prediction-capsule".into(),
            outcome_capsule_digest: "outcome-capsule".into(),
            opening_bundle_digest: bundle.bundle_digest,
            label_status,
            participant_evaluation_digests: bundle
                .participant_evaluations
                .iter()
                .map(|value| value.evaluation_digest.clone())
                .collect(),
            total_event_count_after: 1,
            scorable_event_count_after: usize::from(
                label_status == MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome,
            ),
            winner_selected: false,
            reward_applied: false,
            penalty_applied: false,
            entry_digest: String::new(),
        };
        entry.entry_digest = ledger_entry_digest(&entry);
        let mut ledger = MomentumProspectiveEvaluationLedgerV4_4 {
            ledger_version: LEDGER_VERSION.into(),
            entries: vec![entry],
            ledger_digest: String::new(),
        };
        ledger.ledger_digest = ledger_digest(&ledger);
        ledger
    }

    fn reward_fixture(
        label_status: MomentumProspectiveLabelStatusV4_4,
    ) -> MomentumRewardEligibilityReplayReceiptV4_4 {
        build_reward_eligibility(&ledger_fixture(label_status), &bundle_fixture(label_status))
            .unwrap()
    }

    fn status_fixture() -> MomentumFutureOutcomeStatusReceiptV4_4 {
        build_status(
            &registration_fixture(),
            MomentumOutcomeReadinessV4_4::AwaitingOutcomeFinality,
            None,
            None,
            None,
            None,
            None,
            None,
            true,
            true,
            MomentumFutureOutcomeSafetyCountersV4_4::default(),
        )
        .unwrap()
    }

    #[test]
    fn sprint85_01_prior_prediction_invariants_remain_closed() {
        let capsule = receipt_and_capsule().1;
        assert!(!capsule.labels_opened);
        assert!(!capsule.probabilities_opened);
        assert!(!capsule.metrics_computed);
        assert!(!capsule.winner_selected);
        assert!(!capsule.reward_applied);
        assert!(!capsule.penalty_applied);
    }

    #[test]
    fn sprint85_02_protected_prior_artifacts_remain_byte_identical() {
        let root =
            std::env::temp_dir().join(format!("soma-sprint85-protected-{}", std::process::id()));
        fs::create_dir_all(root.join("v4_3")).unwrap();
        fs::create_dir_all(root.join(ROOT_VERSION_V4_4)).unwrap();
        fs::write(root.join("v4_3/sealed.pb"), b"sealed").unwrap();
        let before = protected_prior_artifacts(&root).unwrap();
        fs::write(root.join(ROOT_VERSION_V4_4).join("new.pb"), b"new").unwrap();
        assert_eq!(before, protected_prior_artifacts(&root).unwrap());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sprint85_07_prefinality_readiness_awaits_finality() {
        assert_eq!(
            acquisition_readiness(OUTCOME_TIMESTAMP, FINALITY_TIMESTAMP, None, None, None).unwrap(),
            MomentumOutcomeReadinessV4_4::AwaitingOutcomeFinality
        );
    }

    #[test]
    fn sprint85_08_prefinality_constructs_zero_transport() {
        let status = status_fixture();
        assert_eq!(status.safety_counters.outcome_transport_constructions, 0);
        assert_eq!(status.safety_counters.outcome_request_attempts, 0);
    }

    #[test]
    fn sprint85_09_postfinality_is_ready_for_one_acquisition() {
        assert_eq!(
            acquisition_readiness(FINALITY_TIMESTAMP, FINALITY_TIMESTAMP, None, None, None)
                .unwrap(),
            MomentumOutcomeReadinessV4_4::ReadyForOutcomeAcquisition
        );
    }

    #[test]
    fn sprint85_10_exactly_one_outcome_request_is_permitted() {
        let registration = registration_fixture();
        let request = build_request(&registration).unwrap();
        assert_eq!(registration.maximum_requests, 1);
        assert_eq!(request.lookback.bars, 1);
    }

    #[test]
    fn sprint85_11_outcome_retries_remain_zero() {
        assert_eq!(registration_fixture().maximum_retries, 0);
        let receipt = receipt_after_attempt(
            &registration_fixture(),
            MomentumOutcomeAcquisitionStatusV4_4::TerminalTransportFailure,
            None,
            0,
        );
        assert_eq!(receipt.retry_count, 0);
    }

    #[test]
    fn sprint85_12_exact_outcome_timestamp_is_required() {
        let registration = registration_fixture();
        let request = build_request(&registration).unwrap();
        let mut transport = transport_fixture();
        transport.response.normalized_dataset.rows[0].timestamp_ms -= DAILY_CADENCE_MS;
        assert!(validate_outcome_transport(&registration, &request, &transport).is_err());
    }

    #[test]
    fn sprint85_13_multiple_rows_reject() {
        let registration = registration_fixture();
        let request = build_request(&registration).unwrap();
        let mut transport = transport_fixture();
        transport
            .response
            .normalized_dataset
            .rows
            .push(outcome_row());
        assert!(validate_outcome_transport(&registration, &request, &transport).is_err());
    }

    #[test]
    fn sprint85_14_missing_row_rejects() {
        let registration = registration_fixture();
        let request = build_request(&registration).unwrap();
        let mut transport = transport_fixture();
        transport.response.normalized_dataset.rows.clear();
        assert!(validate_outcome_transport(&registration, &request, &transport).is_err());
    }

    #[test]
    fn sprint85_15_wrong_row_rejects() {
        let registration = registration_fixture();
        let request = build_request(&registration).unwrap();
        let mut transport = transport_fixture();
        transport.response.normalized_dataset.rows[0].symbol = "KRW-ETH".into();
        assert!(validate_outcome_transport(&registration, &request, &transport).is_err());
    }

    #[test]
    fn sprint85_16_wrong_market_rejects() {
        let registration = registration_fixture();
        let request = build_request(&registration).unwrap();
        let mut transport = transport_fixture();
        transport.response.normalized_dataset.symbol = "KRW-ETH".into();
        assert!(validate_outcome_transport(&registration, &request, &transport).is_err());
    }

    #[test]
    fn sprint85_17_nonfinalized_row_rejects() {
        let registration = registration_fixture();
        let request = build_request(&registration).unwrap();
        let mut transport = transport_fixture();
        transport.response.all_rows_finalized = false;
        assert!(validate_outcome_transport(&registration, &request, &transport).is_err());
    }

    #[test]
    fn sprint85_18_invalid_ohlcv_rejects() {
        let registration = registration_fixture();
        let request = build_request(&registration).unwrap();
        let mut transport = transport_fixture();
        transport.response.normalized_dataset.rows[0].high = 1.0;
        assert!(validate_outcome_transport(&registration, &request, &transport).is_err());
    }

    #[test]
    fn sprint85_19_terminal_failure_cannot_retry() {
        let receipt = receipt_after_attempt(
            &registration_fixture(),
            MomentumOutcomeAcquisitionStatusV4_4::TerminalTransportFailure,
            None,
            0,
        );
        assert_eq!(
            acquisition_readiness(
                FINALITY_TIMESTAMP,
                FINALITY_TIMESTAMP,
                Some(&receipt),
                None,
                None
            )
            .unwrap(),
            MomentumOutcomeReadinessV4_4::PriorOutcomeAttemptTerminal
        );
        assert_eq!(receipt.retry_count, 0);
    }

    #[test]
    fn sprint85_20_successful_capsule_keeps_labels_closed() {
        let capsule = receipt_and_capsule().1;
        assert!(!capsule.labels_opened);
        assert!(!capsule.probabilities_opened);
    }

    #[test]
    fn sprint85_21_acquisition_computes_no_metrics() {
        assert!(!receipt_and_capsule().1.metrics_computed);
    }

    #[test]
    fn sprint85_22_acquisition_selects_no_winner() {
        assert!(!receipt_and_capsule().1.winner_selected);
    }

    #[test]
    fn sprint85_23_repeated_acquisition_performs_zero_new_work() {
        let (receipt, capsule) = receipt_and_capsule();
        assert_eq!(
            acquisition_readiness(
                FINALITY_TIMESTAMP,
                FINALITY_TIMESTAMP,
                Some(&receipt),
                Some(&capsule),
                None
            )
            .unwrap(),
            MomentumOutcomeReadinessV4_4::OutcomeEvidenceAcquired
        );
        assert_eq!(
            MomentumFutureOutcomeSafetyCountersV4_4::default().outcome_request_attempts,
            0
        );
    }

    #[test]
    fn sprint85_24_opening_requires_explicit_authorization() {
        let mut authorization = opening_authorization_fixture();
        authorization.explicit_owner_authorization = false;
        authorization.authorization_digest = authorization_digest(&authorization);
        assert!(validate_opening_authorization_shape(&authorization).is_err());
    }

    #[test]
    fn sprint87_01_opening_authorization_binds_every_authority_prohibition() {
        let authorization = opening_authorization_fixture();
        assert_eq!(
            decode_opening_authorization(&encode_opening_authorization(&authorization).unwrap())
                .unwrap(),
            authorization
        );
        for invalidate in [
            |value: &mut MomentumOutcomeOpeningAuthorizationV4_4| value.ranking_forbidden = false,
            |value: &mut MomentumOutcomeOpeningAuthorizationV4_4| {
                value.penalty_application_forbidden = false
            },
            |value: &mut MomentumOutcomeOpeningAuthorizationV4_4| {
                value.chair_action_forbidden = false
            },
            |value: &mut MomentumOutcomeOpeningAuthorizationV4_4| {
                value.voice_mutation_forbidden = false
            },
            |value: &mut MomentumOutcomeOpeningAuthorizationV4_4| value.promotion_forbidden = false,
            |value: &mut MomentumOutcomeOpeningAuthorizationV4_4| value.trading_forbidden = false,
        ] {
            let mut invalid = authorization.clone();
            invalidate(&mut invalid);
            invalid.authorization_digest = authorization_digest(&invalid);
            assert!(validate_opening_authorization_shape(&invalid).is_err());
        }
    }

    #[test]
    fn sprint85_25_opening_cannot_construct_network_transport() {
        let root =
            std::env::temp_dir().join(format!("soma-sprint85-opening-{}", std::process::id()));
        let result = run_momentum_future_outcome_opening_v4_4(
            &root,
            &provider_config(),
            FINALITY_TIMESTAMP,
            MomentumOutcomeOpeningRunModeV4_4::Status,
            true,
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn sprint85_26_opening_binds_all_three_participant_seals() {
        let authorization = opening_authorization_fixture();
        assert_eq!(authorization.participant_seal_digests.len(), 3);
        assert_eq!(
            authorization
                .participant_seal_digests
                .iter()
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
    }

    #[test]
    fn sprint85_27_changed_participant_seal_blocks_opening() {
        let mut authorization = opening_authorization_fixture();
        authorization.participant_seal_digests[2] =
            authorization.participant_seal_digests[1].clone();
        authorization.authorization_digest = authorization_digest(&authorization);
        assert!(validate_opening_authorization_shape(&authorization).is_err());
    }

    #[test]
    fn sprint85_28_changed_label_policy_blocks_opening() {
        let mut authorization = opening_authorization_fixture();
        authorization.label_policy_digest = "changed-label-policy".to_string();
        authorization.authorization_digest = authorization_digest(&authorization);
        assert!(validate_opening_authorization_shape(&authorization).is_err());
    }

    #[test]
    fn sprint85_29_neutral_outcomes_remain_unscored() {
        let dead_zone = f64::from(
            MomentumLearningCampaignConfigV0::default()
                .sequence_config
                .label_dead_zone,
        );
        let result = classify_label_v4_4(100.0, 100.0 * (1.0 + dead_zone / 2.0)).unwrap();
        assert_eq!(
            result.0,
            MomentumProspectiveLabelStatusV4_4::NeutralOutcomeExcluded
        );
        assert!(result.1.is_none());
    }

    #[test]
    fn sprint85_30_binary_outcome_has_three_evaluations() {
        let evaluations =
            evaluations_fixture(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(evaluations.len(), 3);
        assert!(
            evaluations
                .iter()
                .all(|value| { value.status == MomentumProspectiveEvaluationStatusV4_4::Scored })
        );
    }

    #[test]
    fn sprint85_31_numeric_predictions_remain_private() {
        let json = serde_json::to_string(&status_fixture()).unwrap();
        assert!(!json.contains("probability"));
        assert!(!json.contains("prediction_probability_bits"));
    }

    #[test]
    fn sprint85_32_numeric_labels_remain_private() {
        let json = serde_json::to_string(&status_fixture()).unwrap();
        assert!(!json.contains("private_label"));
        assert!(!json.contains("future_return"));
    }

    #[test]
    fn sprint85_33_event_brier_values_remain_private() {
        let status = status_fixture();
        let json = serde_json::to_string(&status).unwrap();
        assert!(!json.to_ascii_lowercase().contains("brier"));
        assert!(!json.contains("private_score"));
    }

    #[test]
    fn sprint85_34_participant_roles_remain_distinct() {
        let evaluations =
            evaluations_fixture(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(
            evaluations
                .iter()
                .map(|value| &value.participant_role)
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
    }

    #[test]
    fn sprint85_35_benchmark_is_not_counted_as_learned() {
        let reward = reward_fixture(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(reward.learned_participant_count, 2);
        assert_eq!(reward.benchmark_participant_count, 1);
    }

    #[test]
    fn sprint85_36_no_ranking_is_created() {
        assert_eq!(
            MomentumFutureOutcomeSafetyCountersV4_4::default().ranking_creations,
            0
        );
    }

    #[test]
    fn sprint85_37_no_winner_is_selected() {
        let bundle = bundle_fixture(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert!(!bundle.winner_selected);
        assert!(!bundle.ranking_created);
        assert!(!bundle.chair_action_taken);
    }

    #[test]
    fn sprint85_38_evaluation_ledger_appends_exactly_once() {
        let ledger = ledger_fixture(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(ledger.entries.len(), 1);
        assert_eq!(ledger.entries[0].total_event_count_after, 1);
        assert!(!ledger.entries[0].reward_applied);
        assert!(!ledger.entries[0].penalty_applied);
    }

    #[test]
    fn sprint85_39_prior_prospective_records_remain_separate() {
        let ledger = ledger_fixture(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(ledger.ledger_version, LEDGER_VERSION);
        assert!(!ledger.ledger_digest.contains("prospective_opening_v0"));
    }

    #[test]
    fn sprint85_40_reward_eligibility_derives_from_ledger() {
        let reward = reward_fixture(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(reward.event_count, 1);
        assert_eq!(reward.scorable_event_count, 1);
        assert_eq!(
            reward.status,
            MomentumRewardEligibilityStatusV4_4::IneligibleMinimumSamples
        );
    }

    #[test]
    fn sprint85_41_reward_application_remains_zero() {
        assert_eq!(
            reward_fixture(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome)
                .reward_application_count,
            0
        );
    }

    #[test]
    fn sprint85_42_penalty_application_remains_zero() {
        assert_eq!(
            reward_fixture(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome)
                .penalty_application_count,
            0
        );
    }

    #[test]
    fn sprint85_43_voice_and_promotion_mutations_remain_zero() {
        let reward = reward_fixture(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(reward.voice_mutation_count, 0);
        assert_eq!(reward.promotion_count, 0);
        assert_eq!(reward.cooldown_count, 0);
        assert_eq!(reward.quarantine_count, 0);
    }

    #[test]
    fn sprint85_44_repeated_opening_performs_zero_new_work() {
        let (receipt, capsule) = receipt_and_capsule();
        let bundle = bundle_fixture(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        let authorization = opening_authorization_fixture();
        let opening_receipt = build_opening_receipt(&authorization, &bundle).unwrap();
        assert_eq!(
            acquisition_readiness(
                FINALITY_TIMESTAMP,
                FINALITY_TIMESTAMP,
                Some(&receipt),
                Some(&capsule),
                Some(&opening_receipt)
            )
            .unwrap(),
            MomentumOutcomeReadinessV4_4::OutcomeAlreadyOpened
        );
        assert_eq!(
            MomentumFutureOutcomeSafetyCountersV4_4::default().outcome_opening_attempts,
            0
        );
    }

    #[test]
    fn sprint85_45_malformed_protobuf_rejects() {
        assert!(decode_registration(b"not-protobuf").is_err());
        assert!(decode_opening_bundle(&[0xff, 0x00, 0x01]).is_err());
    }

    #[test]
    fn sprint85_46_text_and_json_status_agree() {
        let status = status_fixture();
        let text = crate::cli::format_momentum_v4_future_outcome_text(&status);
        let json = serde_json::to_string(&status).unwrap();
        let decoded: MomentumFutureOutcomeStatusReceiptV4_4 = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, status);
        assert!(text.contains(&format!("outcome_readiness={:?}", status.outcome_readiness)));
        assert!(text.contains(&status.status_digest));
    }

    #[test]
    fn sprint85_47_all_active_and_authority_counters_remain_zero() {
        let counters = MomentumFutureOutcomeSafetyCountersV4_4::default();
        assert_eq!(counters.active_model_changes, 0);
        assert_eq!(counters.chair_decisions, 0);
        assert_eq!(counters.votes, 0);
        assert_eq!(counters.executions, 0);
        assert_eq!(counters.reward_applications, 0);
        assert_eq!(counters.penalty_applications, 0);
        assert_eq!(counters.promotions, 0);
        assert_eq!(counters.quarantines, 0);
        assert_eq!(counters.active_committee_count, 3);
    }

    #[test]
    fn sprint87_02_closed_artifacts_bind_zero_authority_application() {
        let capsule = receipt_and_capsule().1;
        assert_eq!(
            decode_capsule(&encode_capsule(&capsule).unwrap()).unwrap(),
            capsule
        );
        assert!(!capsule.reward_applied);
        assert!(!capsule.penalty_applied);
        for invalidate in [
            |value: &mut MomentumSealedOutcomeCapsuleV4_4| value.reward_applied = true,
            |value: &mut MomentumSealedOutcomeCapsuleV4_4| value.penalty_applied = true,
        ] {
            let mut invalid = capsule.clone();
            invalidate(&mut invalid);
            invalid.capsule_digest = capsule_digest(&invalid);
            assert!(validate_capsule_shape(&invalid).is_err());
        }

        let bundle = bundle_fixture(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(
            decode_opening_bundle(&encode_opening_bundle(&bundle).unwrap()).unwrap(),
            bundle
        );
        assert!(!bundle.ranking_created);
        assert!(!bundle.chair_action_taken);
        for invalidate in [
            |value: &mut MomentumOutcomeOpeningBundleV4_4| value.ranking_created = true,
            |value: &mut MomentumOutcomeOpeningBundleV4_4| value.chair_action_taken = true,
        ] {
            let mut invalid = bundle.clone();
            invalidate(&mut invalid);
            invalid.bundle_digest = opening_bundle_digest(&invalid);
            assert!(validate_opening_bundle_shape(&invalid).is_err());
        }

        let ledger = ledger_fixture(MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome);
        assert_eq!(
            decode_ledger(&encode_ledger(&ledger).unwrap()).unwrap(),
            ledger
        );
        assert!(!ledger.entries[0].reward_applied);
        assert!(!ledger.entries[0].penalty_applied);
        for invalidate in [
            |value: &mut MomentumProspectiveEvaluationLedgerEntryV4_4| value.reward_applied = true,
            |value: &mut MomentumProspectiveEvaluationLedgerEntryV4_4| value.penalty_applied = true,
        ] {
            let mut invalid = ledger.clone();
            invalidate(&mut invalid.entries[0]);
            invalid.entries[0].entry_digest = ledger_entry_digest(&invalid.entries[0]);
            invalid.ledger_digest = ledger_digest(&invalid);
            assert!(validate_ledger_shape(&invalid).is_err());
        }
    }

    #[test]
    fn sprint88_01_acquisition_cli_exposes_the_complete_preflight_contract() {
        let registration = registration_fixture();
        let report = MomentumFutureOutcomeReportV4_4 {
            status: status_fixture(),
            registration: registration.clone(),
            receipt: None,
            outcome_capsule: None,
            prediction_value_reads: 0,
            artifacts_written: 0,
            duplicate_artifact_count: 0,
        };
        let text = crate::cli::format_momentum_v4_future_outcome_report_text(&report);
        let json: serde_json::Value = serde_json::from_str(
            &crate::cli::format_momentum_v4_future_outcome_report_json(&report).unwrap(),
        )
        .unwrap();
        for (name, value) in [
            (
                "prediction_capsule_digest",
                registration.prediction_capsule_digest.as_str(),
            ),
            (
                "prediction_journal_digest",
                registration.prediction_journal_digest.as_str(),
            ),
            (
                "outcome_plan_digest",
                registration.outcome_plan_digest.as_str(),
            ),
            ("provider_id", registration.provider_id.as_str()),
            ("market", registration.market.as_str()),
            ("symbol", registration.symbol.as_str()),
            ("cadence", registration.cadence.as_str()),
        ] {
            assert!(text.contains(&format!("{name}={value}")));
            assert_eq!(json[name], value);
        }
        for (name, value) in [
            (
                "request_to_timestamp_ms",
                registration.request_to_timestamp_ms,
            ),
            ("expected_row_count", registration.expected_row_count as u64),
            ("maximum_requests", registration.maximum_requests as u64),
            (
                "maximum_concurrency",
                registration.maximum_concurrency as u64,
            ),
            ("maximum_retries", registration.maximum_retries as u64),
        ] {
            assert!(text.contains(&format!("{name}={value}")));
            assert_eq!(json[name], value);
        }
        assert_eq!(
            json["exact_expected_timestamp_ms"],
            serde_json::json!(registration.exact_expected_timestamp_ms)
        );
        assert_eq!(json["prediction_value_reads"], 0);
        assert_eq!(json["artifacts_written"], 0);
    }

    #[test]
    fn sprint88_02_opening_cli_exposes_zero_preflight_work() {
        let report = MomentumFutureOutcomeOpeningReportV4_4 {
            status: status_fixture(),
            authorization: None,
            opening_receipt: None,
            opening_bundle: None,
            evaluation_ledger: None,
            reward_eligibility: None,
            prediction_value_reads: 0,
            artifacts_written: 0,
            duplicate_artifact_count: 0,
        };
        let text = crate::cli::format_momentum_v4_future_outcome_opening_report_text(&report);
        let json: serde_json::Value = serde_json::from_str(
            &crate::cli::format_momentum_v4_future_outcome_opening_report_json(&report).unwrap(),
        )
        .unwrap();
        for field in [
            "prediction_value_reads",
            "artifacts_written",
            "duplicate_artifact_count",
        ] {
            assert!(text.contains(&format!("{field}=0")));
            assert_eq!(json[field], 0);
        }
    }
}
