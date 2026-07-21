//! Offline, agent-private learning sessions backed by the existing shadow trainers.
//!
//! This boundary resolves only explicitly authorized immutable evidence.  It owns
//! private manifests and candidate metadata, but has no active committee, Chair,
//! reward, prospective-evaluation, network, or execution authority.

use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{
    core::{ReasonCode, stable_hash_string},
    data::{
        AcquisitionMarketScope, AgentDataIntent, AgentDataPolicy, AgentLearningDataViewV0,
        AgentLearningIntentV0, DataLookback, DataPriority, DataSnapshot, DatasetKind,
        EvidenceDecisionGate, LearningDataArtifactRefV0, LearningDataPlaneSafetyCountersV0,
        LearningDataVisibilityV0, build_agent_learning_data_view_v0,
        create_agent_learning_intent_v0, derive_agent_private_learning_state_v0,
        historical_replay_dataset_digest_v0, read_local_snapshot_protobuf_v1,
        snapshot_id_from_semantic_digest_v1, validate_agent_learning_data_view_v0,
    },
    league::{
        AgentKind, HistoricalOhlcvRow, HistoricalReplayDataset, canonical_current_agent_states,
    },
};

use super::{
    CycleRiskErrorV0, CycleRiskShadowConfigV0, IndexRangeV0, ModelAgentDeploymentStatus,
    MomentumLearningCampaignConfigV0, MomentumLearningCampaignStatusV0,
    build_momentum_learning_windows_v0, frozen_mamba3_encoder_from_seed_v0,
    run_cycle_risk_shadow_v0, run_momentum_learning_campaign_v0,
};

const SESSION_VERSION_V0: &str = "agent-private-learning-session-v0";
const DATASET_VERSION_V0: &str = "agent-private-dataset-manifest-v0";
const CANDIDATE_VERSION_V0: &str = "agent-sandbox-learning-candidate-v0";
const JOURNAL_VERSION_V0: &str = "agent-private-learning-journal-v0";
const REGISTRY_VERSION_V0: &str = "agent-trainer-capability-registry-v0";
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
    pub source_artifact_digests: Vec<String>,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub curriculum_policy_digest: String,
    pub private_namespace_digest: String,
    pub parent_model_version: Option<String>,
    pub session_status: AgentLearningSessionStatusV0,
    pub session_digest: String,
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
    pub view: AgentLearningDataViewV0,
    pub artifacts: Vec<AgentPrivateLearningArtifactV0>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPrivateLearningSessionResultV0 {
    pub session: AgentPrivateLearningSessionV0,
    pub trainer_kind: AgentTrainerKindV0,
    pub source_count: usize,
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
            vec![
                DatasetKind::DailyOhlcv,
                DatasetKind::CryptoDailyOhlcv,
                DatasetKind::MarketIndexDaily,
                DatasetKind::VolatilityDaily,
            ],
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
    let registry = agent_trainer_capability_registry_v0();
    let mut inputs = Vec::new();
    for state in canonical_current_agent_states() {
        let capability = registry
            .capabilities
            .iter()
            .find(|capability| capability.agent_id == state.agent_id)
            .ok_or_else(|| "agent trainer capability missing".to_string())?;
        let selected = snapshots
            .iter()
            .filter(|snapshot| {
                capability
                    .supported_dataset_kinds
                    .contains(&snapshot.dataset_kind)
                    && snapshot
                        .actual_end_timestamp_ms
                        .is_some_and(|end| end <= information_cutoff_ms)
            })
            .max_by(|left, right| {
                left.row_count
                    .cmp(&right.row_count)
                    .then_with(|| right.content_digest.cmp(&left.content_digest))
            });
        inputs.push(build_session_input_v0(
            state.agent_id.as_str(),
            state.kind,
            capability,
            selected,
            information_cutoff_ms,
        )?);
    }
    inputs.sort_by(|left, right| left.intent.agent_id.cmp(&right.intent.agent_id));
    Ok(inputs)
}

fn build_session_input_v0(
    agent_id: &str,
    agent_kind: AgentKind,
    capability: &AgentTrainerCapabilityV0,
    snapshot: Option<&DataSnapshot>,
    information_cutoff_ms: u64,
) -> Result<AgentPrivateLearningSessionInputV0, String> {
    let dataset_kind = snapshot
        .map(|snapshot| snapshot.dataset_kind)
        .or_else(|| capability.supported_dataset_kinds.first().copied())
        .ok_or_else(|| "trainer capability has no dataset contract".to_string())?;
    let market_scope = snapshot
        .map(|snapshot| snapshot.market_scope)
        .unwrap_or(AcquisitionMarketScope::UsStocks);
    let symbols = snapshot
        .map(|snapshot| snapshot.symbols.clone())
        .unwrap_or_else(|| vec![format!("{agent_id}-offline")]);
    let bars = snapshot.map_or(1, |snapshot| snapshot.row_count.max(1));
    let policy = AgentDataPolicy {
        agent_kind,
        allowed_markets: vec![
            AcquisitionMarketScope::UsStocks,
            AcquisitionMarketScope::KoreanStocks,
            AcquisitionMarketScope::BtcCrypto,
        ],
        allowed_dataset_kinds: capability.supported_dataset_kinds.clone(),
        required_dataset_kinds: vec![dataset_kind],
        optional_dataset_kinds: vec![],
        default_lookback: DataLookback {
            bars,
            start_timestamp_ms: snapshot.and_then(|snapshot| snapshot.actual_start_timestamp_ms),
            end_timestamp_ms: Some(information_cutoff_ms),
        },
        max_staleness_ms: u64::MAX,
        request_budget: 1,
        abstain_when_required_missing: true,
        reason_codes: vec![ReasonCode::AgentDataPolicyApplied],
    };
    let data_intent = AgentDataIntent {
        agent_id: agent_id.to_string(),
        agent_kind,
        market_scope,
        symbols,
        required_datasets: vec![dataset_kind],
        optional_datasets: vec![],
        lookback: policy.default_lookback.clone(),
        target_cadence: "1d".to_string(),
        max_staleness_ms: policy.max_staleness_ms,
        priority: DataPriority::Required,
        reason_codes: vec![ReasonCode::AgentDataPolicyApplied],
    };
    let intent = create_agent_learning_intent_v0(
        &crate::data::LearningDataCallerV0::Agent(agent_id.to_string()),
        &data_intent,
        &policy,
        information_cutoff_ms,
    )?;
    let artifacts = snapshot
        .map(|snapshot| AgentPrivateLearningArtifactV0 {
            artifact_ref: LearningDataArtifactRefV0 {
                artifact_digest: snapshot.content_digest.clone(),
                dataset_kind: snapshot.dataset_kind,
                visibility: LearningDataVisibilityV0::SharedCanonicalRaw,
                owner_agent_id: None,
                maximum_event_timestamp_ms: snapshot.actual_end_timestamp_ms.unwrap_or_default(),
            },
            snapshot: snapshot.clone(),
        })
        .into_iter()
        .collect::<Vec<_>>();
    let artifact_refs = artifacts
        .iter()
        .map(|artifact| artifact.artifact_ref.clone())
        .collect::<Vec<_>>();
    let view = build_agent_learning_data_view_v0(
        &intent,
        &policy,
        &artifact_refs,
        &derive_agent_private_learning_state_v0(&intent),
    )?;
    Ok(AgentPrivateLearningSessionInputV0 {
        intent,
        view,
        artifacts,
    })
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
    let mut sanitized_error_code = None;

    if !capability.supports_training {
        session.session_status = AgentLearningSessionStatusV0::TrainerUnavailable;
    } else if mode == AgentPrivateLearningRunModeV0::Status {
        session.session_status = AgentLearningSessionStatusV0::Registered;
    } else {
        match materialize_private_dataset_v0(input, capability.trainer_kind, &session.session_id) {
            Ok(materialized) => {
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
        source_count: session.source_artifact_digests.len(),
        session,
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
        source_count: input.view.source_artifact_digests.len(),
        session,
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
        session_version: SESSION_VERSION_V0.to_string(),
        session_id,
        agent_id: input.intent.agent_id.clone(),
        agent_kind: input.intent.agent_kind,
        intent_digest: input.intent.intent_digest.clone(),
        data_view_digest: input.view.view_digest.clone(),
        trainer_capability_digest: capability.capability_digest.clone(),
        information_cutoff_ms: input.view.information_cutoff_ms,
        source_artifact_digests: input.view.source_artifact_digests.clone(),
        feature_policy_digest: input.view.feature_policy_digest.clone(),
        label_policy_digest: input.view.label_policy_digest.clone(),
        curriculum_policy_digest: input.view.curriculum_policy_digest.clone(),
        private_namespace_digest: input.view.private_namespace_digest.clone(),
        parent_model_version: None,
        session_status: AgentLearningSessionStatusV0::Registered,
        session_digest: String::new(),
    };
    session.session_digest = session_digest_v0(&session);
    session
}

fn materialize_private_dataset_v0(
    input: &AgentPrivateLearningSessionInputV0,
    trainer_kind: AgentTrainerKindV0,
    session_id: &str,
) -> Result<MaterializedPrivateDatasetV0, EvidenceResolutionErrorV0> {
    validate_agent_learning_data_view_v0(&input.view)
        .map_err(|_| EvidenceResolutionErrorV0::UnsafeEvidence)?;
    if input.intent.agent_id != input.view.agent_id
        || input.intent.intent_digest.is_empty()
        || input.view.decision_gate != EvidenceDecisionGate::Ready
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
    let mut rows = Vec::<HistoricalOhlcvRow>::new();
    let mut source_digests = Vec::new();
    let mut dataset_kinds = Vec::new();
    let mut symbol = None::<String>;
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
            || !snapshot.provenance.sanitized
            || !snapshot.provenance.credential_free
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
        if symbol
            .as_ref()
            .is_some_and(|expected| expected != &snapshot.normalized_dataset.symbol)
        {
            return Err(EvidenceResolutionErrorV0::UnauthorizedDataset);
        }
        symbol = Some(snapshot.normalized_dataset.symbol.clone());
        rows.extend(snapshot.normalized_dataset.rows.clone());
        source_digests.push(digest);
        dataset_kinds.push(snapshot.dataset_kind);
    }
    if rows.is_empty() {
        return Err(EvidenceResolutionErrorV0::Insufficient);
    }
    rows.sort_by_key(|row| row.timestamp_ms);
    if rows
        .windows(2)
        .any(|pair| pair[0].timestamp_ms == pair[1].timestamp_ms)
    {
        return Err(EvidenceResolutionErrorV0::Duplicate);
    }
    source_digests.sort();
    dataset_kinds.sort();
    dataset_kinds.dedup();
    let mut snapshot = input.artifacts[0].snapshot.clone();
    snapshot.normalized_dataset = HistoricalReplayDataset {
        symbol: symbol.ok_or(EvidenceResolutionErrorV0::Insufficient)?,
        source: "agent-private-immutable-historical-evidence".to_string(),
        rows,
        reason_codes: vec![],
    };
    snapshot.row_count = snapshot.normalized_dataset.rows.len();
    snapshot.quality_summary.row_count = snapshot.row_count;
    snapshot.actual_start_timestamp_ms = snapshot
        .normalized_dataset
        .rows
        .first()
        .map(|row| row.timestamp_ms);
    snapshot.actual_end_timestamp_ms = snapshot
        .normalized_dataset
        .rows
        .last()
        .map(|row| row.timestamp_ms);
    snapshot.requested_lookback = DataLookback {
        bars: snapshot.row_count,
        start_timestamp_ms: snapshot.actual_start_timestamp_ms,
        end_timestamp_ms: Some(input.view.information_cutoff_ms),
    };
    snapshot.compatibility = None;
    snapshot.content_digest = historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
    snapshot.snapshot_id = snapshot_id_from_semantic_digest_v1(&snapshot.content_digest);
    snapshot.request_key = stable_hash_string(&format!(
        "private-dataset:{}:{}",
        input.intent.agent_id, input.view.view_digest
    ));
    let manifest = initial_dataset_manifest_v0(
        input,
        trainer_kind,
        session_id,
        &source_digests,
        &dataset_kinds,
        snapshot.row_count,
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
            write_journal_artifact_v0(&result.journal, &agent_root),
            &mut storage,
        );
    }
    report.duplicate_artifact_count = storage.duplicate_artifact_count;
    report.storage_failure_count = storage.failed_artifact_count;
    report.report_digest = report_digest_v0(report);
    storage
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
        &root.join("capability_registry.pb"),
        &bytes,
        &registry.registry_digest,
        |stored| Ok(decode_capability_registry_protobuf_v0(stored)?.registry_digest),
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
        &directory.join("journal.pb"),
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
    Dataset,
    Candidate,
    Journal,
    Registry,
}

impl ArtifactKindV0 {
    fn wire_name(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Dataset => "dataset",
            Self::Candidate => "candidate",
            Self::Journal => "journal",
            Self::Registry => "registry",
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
        source_artifact_digests: value.source_artifact_digests,
        feature_policy_digest: value.feature_policy_digest,
        label_policy_digest: value.label_policy_digest,
        curriculum_policy_digest: value.curriculum_policy_digest,
        private_namespace_digest: value.private_namespace_digest,
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
    stable_hash_string(&format!(
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
    use crate::data::{SnapshotProvenance, SnapshotQualitySummary};

    fn snapshot(rows: usize) -> DataSnapshot {
        let normalized_dataset = HistoricalReplayDataset {
            symbol: "BTC-KRW".to_string(),
            rows: (0..rows)
                .map(|index| {
                    let price = 100.0 + index as f64 * 0.07 + (index % 11) as f64 * 0.8;
                    HistoricalOhlcvRow {
                        symbol: "BTC-KRW".to_string(),
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
            request_key: "private-learning-test".to_string(),
            provider_id: "approved-provider".to_string(),
            dataset_kind: DatasetKind::DailyOhlcv,
            market_scope: AcquisitionMarketScope::BtcCrypto,
            symbols: vec!["BTC-KRW".to_string()],
            requested_lookback: DataLookback {
                bars: rows,
                start_timestamp_ms: Some(1),
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
            compatibility: None,
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

    fn inputs() -> Vec<AgentPrivateLearningSessionInputV0> {
        let snapshot = snapshot(360);
        build_agent_private_learning_inputs_v0(&[snapshot], 360).unwrap()
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
        let mut snapshot = snapshot(360);
        snapshot
            .normalized_dataset
            .rows
            .last_mut()
            .unwrap()
            .timestamp_ms = 361;
        snapshot.content_digest = historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
        snapshot.snapshot_id = snapshot_id_from_semantic_digest_v1(&snapshot.content_digest);
        let inputs = build_agent_private_learning_inputs_v0(&[snapshot], 360).unwrap();
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
        let mut snapshot = snapshot(360);
        snapshot.normalized_dataset.rows[1].timestamp_ms = 1;
        snapshot.content_digest = historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
        snapshot.snapshot_id = snapshot_id_from_semantic_digest_v1(&snapshot.content_digest);
        let inputs = build_agent_private_learning_inputs_v0(&[snapshot], 360).unwrap();
        let report =
            run_agent_private_learning_sessions_v0(&inputs, AgentPrivateLearningRunModeV0::DryRun);
        assert_eq!(
            result_for(&report, "momentum_trend_fast")
                .sanitized_error_code
                .as_deref(),
            Some("duplicate_timestamp_rejected")
        );
    }

    #[test]
    fn non_finite_row_rejects() {
        let mut snapshot = snapshot(360);
        snapshot.normalized_dataset.rows[10].volume = f64::NAN;
        snapshot.content_digest = historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
        snapshot.snapshot_id = snapshot_id_from_semantic_digest_v1(&snapshot.content_digest);
        let inputs = build_agent_private_learning_inputs_v0(&[snapshot], 360).unwrap();
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
    fn public_summary_excludes_private_training_material() {
        let report = execution_report();
        let json = serde_json::to_string(&public_session_summaries_v0(&report)).unwrap();
        assert!(!json.contains("private_metrics"));
        assert!(!json.contains("normalizer"));
        assert!(!json.contains("weights"));
        assert!(!json.contains("prediction"));
    }
}
