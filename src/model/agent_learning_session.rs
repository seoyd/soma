//! Offline, agent-private learning sessions backed by the existing shadow trainers.
//!
//! This boundary resolves only explicitly authorized immutable evidence.  It owns
//! private manifests and candidate metadata, but has no active committee, Chair,
//! reward, prospective-evaluation, network, or execution authority.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{
    core::stable_hash_string,
    data::{
        AcquisitionMarketScope, AcquisitionMode, AcquisitionPolicy, AgentDataPolicy,
        AgentLearningDataViewV0, AgentLearningIntentV0, ConfiguredUniverse, DataLookback,
        DataSnapshot, DatasetKind, EvidenceDecisionGate, LearningDataArtifactRefV0,
        LearningDataPlaneSafetyCountersV0, LearningDataVisibilityV0, ReadOnlyProviderRegistry,
        ReadOnlyProviderRequest, SnapshotAdjustmentSemanticsV1, SnapshotSourceType,
        build_agent_learning_data_view_v0, build_learning_acquisition_plan_v0,
        default_agent_data_policies, derive_active_agent_learning_intents_v0,
        derive_agent_private_learning_state_v0, historical_replay_dataset_digest_v0,
        read_local_snapshot_protobuf_v1, validate_agent_learning_data_view_v0,
        validate_agent_learning_intent_v0,
    },
    league::{AgentKind, HistoricalOhlcvRow, canonical_current_agent_states},
};

use super::{
    CycleRiskErrorV0, CycleRiskShadowConfigV0, IndexRangeV0, ModelAgentDeploymentStatus,
    MomentumLearningCampaignConfigV0, MomentumLearningCampaignStatusV0,
    build_momentum_learning_windows_v0, frozen_mamba3_encoder_from_seed_v0,
    run_cycle_risk_shadow_v0, run_momentum_learning_campaign_v0,
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

fn configured_universe_from_snapshots_v0(snapshots: &[DataSnapshot]) -> ConfiguredUniverse {
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
                && compatibility.maximum_staleness_ms == request.max_staleness_ms
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
    let mut file = File::create(&temporary)
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
        .map_err(|_| "private learning atomic rename failed".to_string())?;
    let stored = fs::read(path).map_err(|_| "private learning final reopen failed".to_string())?;
    if verify(&stored)? != expected_digest {
        return Err("private learning final verification failed".to_string());
    }
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
}
