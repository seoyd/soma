use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use serde::{Deserialize, Serialize};

use crate::{
    data::{AcquisitionMarketScope, DataSnapshot, historical_replay_dataset_digest_v0},
    league::{HistoricalOhlcvRow, canonical_current_agent_states},
    stable_hash_string,
};

use super::{
    EncodedTrainingExampleV0, LogisticPredictionHeadV0, MomentumCandleV0, MomentumFeatureConfigV0,
    MomentumLearningCampaignConfigV0, MomentumProspectiveLabelStatusV4_4, MomentumSequenceConfigV0,
    RepresentationNormalizerV0, build_momentum_features_v0, build_momentum_sequence_examples_v0,
    evaluate_probabilities_v0,
    momentum_future_outcome_v4::classify_label_v4_4,
    momentum_future_prediction_v4::{
        ArtifactBuilderV4_2, ArtifactReaderV4_2, as_u64, as_usize, persist_artifact,
    },
    momentum_raw_feature_v4::{expand_interaction_representation_v4, raw_encoded, train_head_v4},
};

const SNAPSHOT_VERSION: &str = "momentum-historical-dataset-snapshot-v1";
const AUDIT_VERSION: &str = "momentum-historical-contamination-audit-v1";
const REGISTRATION_VERSION: &str = "momentum-historical-replay-registration-v1";
const AGGREGATE_VERSION: &str = "momentum-historical-aggregate-report-v1";
const JOURNAL_VERSION: &str = "momentum-historical-replay-journal-v1";
const BACKFILL_VERSION: &str = "momentum-historical-backfill-plan-v1";
const PUBLIC_REPORT_VERSION: &str = "momentum-historical-public-report-v1";
const DEFAULT_SNAPSHOT_ROOT: &str = "data/local_snapshots";
const DEFAULT_REPLAY_ROOT: &str = "state/historical_replay/momentum_v4/v1";
const LIVE_PROTECTED_ROOT: &str = "state/learning_data";
const DAILY_CADENCE_MS: u64 = 86_400_000;
const RAW_PARTICIPANT: &str = "HistoricalRawFeatureLogisticV1";
const INTERACTION_PARTICIPANT: &str = "HistoricalRawFeatureInteractionLogisticV1";
const CONSTANT_PARTICIPANT: &str = "HistoricalTrainingPrevalenceConstantV1";
const RESEARCH_LABELS: [&str; 3] = [
    "HistoricalResearchOnly",
    "NotIndependentHoldout",
    "NotProspectiveAuthority",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoricalEvidenceUseClassV1 {
    ProtocolReplayOnly,
    PreviouslyConsumedResearchEvidence,
    CausalWalkForwardResearch,
    SealedHistoricalHoldout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumHistoricalReplayModeV1 {
    ProtocolReplay,
    ExpandingWindowWalkForward,
}

impl MomentumHistoricalReplayModeV1 {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "protocol-replay" => Ok(Self::ProtocolReplay),
            "expanding-window-walk-forward" => Ok(Self::ExpandingWindowWalkForward),
            _ => Err("historical replay mode rejected".to_string()),
        }
    }

    pub fn as_cli_value(self) -> &'static str {
        match self {
            Self::ProtocolReplay => "protocol-replay",
            Self::ExpandingWindowWalkForward => "expanding-window-walk-forward",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumHistoricalFoldSelectionPolicyV1 {
    EveryChronologicallyEligibleEvent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumHistoricalRunModeV1 {
    Status,
    DryRun,
    ExecuteLocal,
}

impl MomentumHistoricalRunModeV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::DryRun => "dry-run",
            Self::ExecuteLocal => "execute-local",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoricalLabelStatusV1 {
    ScorableBinaryOutcome,
    NeutralOutcomeExcluded,
    InvalidOutcomeEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumHistoricalComparisonStatusV1 {
    LearnedBetterOnResearchReplay,
    BenchmarkBetterOnResearchReplay,
    MixedResearchEvidence,
    InsufficientScorableFolds,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumHistoricalTradingSimulationStatusV1 {
    BlockedNoFrozenExecutionPolicy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoricalBackfillDirectionV1 {
    OlderThanExistingSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumHistoricalDatasetSnapshotV1 {
    pub snapshot_version: String,
    pub provider_id: String,
    pub market: String,
    pub symbol: String,
    pub cadence: String,
    pub first_timestamp_ms: u64,
    pub last_timestamp_ms: u64,
    pub row_count: usize,
    pub ordered_row_digests: Vec<String>,
    pub source_capsule_digests: Vec<String>,
    pub dataset_aggregate_digest: String,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub evidence_use_class: HistoricalEvidenceUseClassV1,
    pub previously_consumed: bool,
    pub blind_holdout: bool,
    pub authority_eligible: bool,
    pub snapshot_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumHistoricalContaminationAuditV1 {
    pub audit_version: String,
    pub dataset_snapshot_digest: String,
    pub used_for_feature_design: bool,
    pub used_for_model_design: bool,
    pub used_for_hyperparameter_design: bool,
    pub used_for_qualification: bool,
    pub used_for_participant_selection: bool,
    pub used_for_prior_reporting: bool,
    pub conservative_unknown_history_assumed_consumed: bool,
    pub independent_holdout_claim_forbidden: bool,
    pub live_authority_use_forbidden: bool,
    pub audit_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumHistoricalReplayRegistrationV1 {
    pub registration_version: String,
    pub dataset_snapshot_digest: String,
    pub contamination_audit_digest: String,
    pub replay_mode: MomentumHistoricalReplayModeV1,
    pub context_row_count: usize,
    pub prediction_horizon: usize,
    pub minimum_training_examples: usize,
    pub first_eligible_event_index: usize,
    pub final_eligible_event_index: usize,
    pub training_policy_digest: String,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub evaluation_policy_digest: String,
    pub interaction_schema_digest: String,
    pub participant_templates: Vec<String>,
    pub initialization_seeds: Vec<u64>,
    pub fold_selection_policy: MomentumHistoricalFoldSelectionPolicyV1,
    pub result_conditioned_fold_selection_forbidden: bool,
    pub result_conditioned_hyperparameters_forbidden: bool,
    pub live_prospective_write_forbidden: bool,
    pub reward_application_forbidden: bool,
    pub chair_action_forbidden: bool,
    pub trading_authority_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumHistoricalFoldPlanV1 {
    pub registration_digest: String,
    pub fold_number: u64,
    pub prediction_event_index: usize,
    pub target_index: usize,
    pub prediction_event_timestamp_ms: u64,
    pub target_timestamp_ms: u64,
    pub training_event_timestamp_ms: Vec<u64>,
    pub context_timestamp_ms: Vec<u64>,
    pub latest_training_label_timestamp_ms: u64,
    pub fold_plan_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MomentumHistoricalFoldPredictionV1 {
    pub fold_plan_digest: String,
    pub participant_id: String,
    pub participant_role: String,
    pub parameter_digest: String,
    pub normalizer_digest: String,
    pub feature_digest: String,
    pub private_prediction: f64,
    pub prediction_digest: String,
    pub target_accessed: bool,
    pub prediction_digest_identity: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumHistoricalFoldPredictionCapsuleV1 {
    pub fold_plan_digest: String,
    pub prediction_digests: Vec<String>,
    pub prediction_count: usize,
    pub target_accessed: bool,
    pub label_derived: bool,
    pub metrics_computed: bool,
    pub capsule_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumHistoricalFoldEvaluationV1 {
    pub fold_plan_digest: String,
    pub prediction_capsule_digest: String,
    pub label_status: HistoricalLabelStatusV1,
    pub private_label_bits: Option<u64>,
    pub participant_evaluation_digests: Vec<String>,
    pub fold_evaluation_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumHistoricalParticipantAggregateV1 {
    pub participant_id: String,
    pub scorable_fold_count: usize,
    pub mean_brier_score: f64,
    pub binary_correctness_rate: f64,
    pub benchmark_relative_brier_delta: f64,
    pub comparison_status: MomentumHistoricalComparisonStatusV1,
    pub research_only: bool,
    pub aggregate_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumHistoricalSafetyCountersV1 {
    pub network_requests: usize,
    pub transport_constructions: usize,
    pub credential_reads: usize,
    pub live_prospective_event_count_changes: usize,
    pub live_prospective_scorable_count_changes: usize,
    pub live_participant_changes: usize,
    pub live_parameter_updates: usize,
    pub live_normalizer_refits: usize,
    pub winner_selections: usize,
    pub rankings: usize,
    pub reward_applications: usize,
    pub penalty_applications: usize,
    pub chair_decisions: usize,
    pub committee_votes: usize,
    pub voice_changes: usize,
    pub tier_changes: usize,
    pub cooldowns: usize,
    pub promotions: usize,
    pub quarantines: usize,
    pub paper_executions: usize,
    pub live_executions: usize,
    pub active_committee_count: usize,
}

impl MomentumHistoricalSafetyCountersV1 {
    fn zero_authority() -> Self {
        Self {
            network_requests: 0,
            transport_constructions: 0,
            credential_reads: 0,
            live_prospective_event_count_changes: 0,
            live_prospective_scorable_count_changes: 0,
            live_participant_changes: 0,
            live_parameter_updates: 0,
            live_normalizer_refits: 0,
            winner_selections: 0,
            rankings: 0,
            reward_applications: 0,
            penalty_applications: 0,
            chair_decisions: 0,
            committee_votes: 0,
            voice_changes: 0,
            tier_changes: 0,
            cooldowns: 0,
            promotions: 0,
            quarantines: 0,
            paper_executions: 0,
            live_executions: 0,
            active_committee_count: canonical_current_agent_states().len(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumHistoricalAggregateReportV1 {
    pub aggregate_version: String,
    pub dataset_snapshot_digest: String,
    pub contamination_audit_digest: String,
    pub registration_digest: String,
    pub replay_mode: MomentumHistoricalReplayModeV1,
    pub eligible_fold_count: usize,
    pub completed_fold_count: usize,
    pub scorable_fold_count: usize,
    pub neutral_fold_count: usize,
    pub invalid_fold_count: usize,
    pub learned_vs_constant_paired_scorable_count: usize,
    pub participants: Vec<MomentumHistoricalParticipantAggregateV1>,
    pub comparison_status: MomentumHistoricalComparisonStatusV1,
    pub finite_metric_proof: bool,
    pub chronology_audit_passed: bool,
    pub leakage_audit_passed: bool,
    pub prediction_before_reveal_audit_passed: bool,
    pub winner_selected: bool,
    pub evidence_labels: Vec<String>,
    pub trading_simulation_status: MomentumHistoricalTradingSimulationStatusV1,
    pub safety_counters: MomentumHistoricalSafetyCountersV1,
    pub protected_artifacts_unchanged: bool,
    pub active_roster_unchanged: bool,
    pub aggregate_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumHistoricalReplayJournalV1 {
    journal_version: String,
    registration_digest: String,
    aggregate_digest: String,
    fold_plan_digests: Vec<String>,
    prediction_capsule_digests: Vec<String>,
    fold_evaluation_digests: Vec<String>,
    completed: bool,
    journal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumHistoricalBackfillPlanV1 {
    pub plan_version: String,
    pub provider_id: String,
    pub market: String,
    pub symbol: String,
    pub cadence: String,
    pub existing_first_timestamp_ms: u64,
    pub existing_last_timestamp_ms: u64,
    pub desired_direction: HistoricalBackfillDirectionV1,
    pub request_count_upper_bound: usize,
    pub request_limit_known: bool,
    pub maximum_concurrency: usize,
    pub maximum_retries: usize,
    pub explicit_network_authorization_required: bool,
    pub executed: bool,
    pub backfill_plan_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumHistoricalPublicReportV1 {
    pub report_version: String,
    pub run_mode: String,
    pub replay_mode: MomentumHistoricalReplayModeV1,
    pub offline: bool,
    pub evidence_use_class: HistoricalEvidenceUseClassV1,
    pub dataset_snapshot_digest: String,
    pub contamination_audit_digest: String,
    pub registration_digest: String,
    pub row_count: usize,
    pub first_timestamp_ms: u64,
    pub last_timestamp_ms: u64,
    pub eligible_fold_count: usize,
    pub completed_fold_count: usize,
    pub scorable_fold_count: usize,
    pub neutral_fold_count: usize,
    pub invalid_fold_count: usize,
    pub participants: Vec<MomentumHistoricalParticipantAggregateV1>,
    pub comparison_status: MomentumHistoricalComparisonStatusV1,
    pub chronology_audit_passed: bool,
    pub leakage_audit_passed: bool,
    pub prediction_before_reveal_audit_passed: bool,
    pub replay_deterministic: bool,
    pub existing_completed_replay: bool,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub runtime_duration_ms: u64,
    pub evidence_labels: Vec<String>,
    pub trading_simulation_status: MomentumHistoricalTradingSimulationStatusV1,
    pub backfill_plan: MomentumHistoricalBackfillPlanV1,
    pub safety_counters: MomentumHistoricalSafetyCountersV1,
    pub protected_artifacts_unchanged: bool,
    pub active_roster_unchanged: bool,
    pub replay_digest: String,
}

#[derive(Clone, Debug)]
struct FoldPrivateEvaluation {
    participant_id: String,
    probability: f64,
    label: f64,
    brier: f64,
    correct: bool,
    evaluation_digest: String,
}

#[derive(Clone, Debug, Default)]
struct ParticipantAccumulator {
    brier_sum: f64,
    correct_count: usize,
    count: usize,
}

fn row_digest(row: &HistoricalOhlcvRow) -> String {
    stable_hash_string(&format!(
        "momentum-historical-row-v1:{}:{}:{}:{}:{}:{}:{}:{:?}",
        row.symbol,
        row.timestamp_ms,
        row.open.to_bits(),
        row.high.to_bits(),
        row.low.to_bits(),
        row.close.to_bits(),
        row.volume.to_bits(),
        row.trade_value.map(f64::to_bits),
    ))
}

fn label_policy_digest(config: &MomentumSequenceConfigV0) -> String {
    stable_hash_string(&format!(
        "momentum-v4-label-policy:{}:{}:{}:{}",
        config.prediction_horizon,
        config.label_dead_zone.to_bits(),
        config.include_neutral_labels,
        config.stride,
    ))
}

fn interaction_schema_digest(dimension: usize) -> String {
    let identities = (0..dimension)
        .map(|index| format!("original:{index}"))
        .chain((0..dimension).map(|index| format!("square:{index}")))
        .chain(
            (0..dimension)
                .flat_map(|left| ((left + 1)..dimension).map(move |right| (left, right)))
                .map(|(left, right)| format!("pair:{left}:{right}")),
        )
        .collect::<Vec<_>>();
    stable_hash_string(&format!(
        "momentum-historical-interaction-schema-v1:{}",
        identities.join("|")
    ))
}

fn snapshot_digest(value: &MomentumHistoricalDatasetSnapshotV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{:?}:{:?}:{}:{}:{}:{:?}:{}:{}:{}",
        value.snapshot_version,
        value.provider_id,
        value.market,
        value.symbol,
        value.cadence,
        value.first_timestamp_ms,
        value.last_timestamp_ms,
        value.row_count,
        value.ordered_row_digests,
        value.source_capsule_digests,
        value.dataset_aggregate_digest,
        value.feature_policy_digest,
        value.label_policy_digest,
        value.evidence_use_class,
        value.previously_consumed,
        value.blind_holdout,
        value.authority_eligible,
    ))
}

fn validate_dataset_source(snapshot: &DataSnapshot) -> Result<(), String> {
    crate::data::upbit_historical_pilot::validate_snapshot_shape_v1(snapshot)?;
    if snapshot.provider_id.is_empty()
        || snapshot.provenance.provider_id != snapshot.provider_id
        || !snapshot.sanitized
        || !snapshot.read_only
        || !snapshot.provenance.sanitized
        || !snapshot.provenance.credential_free
        || !snapshot.quality_summary.accepted
        || snapshot.quality_summary.row_count != snapshot.row_count
        || snapshot.row_count != snapshot.normalized_dataset.rows.len()
        || snapshot.content_digest
            != historical_replay_dataset_digest_v0(&snapshot.normalized_dataset)
    {
        return Err("historical dataset provenance or digest rejected".to_string());
    }
    let compatibility = snapshot
        .compatibility
        .as_ref()
        .ok_or_else(|| "historical dataset compatibility unavailable".to_string())?;
    if compatibility.cadence != "1d" || !compatibility.all_rows_finalized {
        return Err("historical dataset daily finality contract rejected".to_string());
    }
    let source = snapshot.normalized_dataset.source.to_ascii_lowercase();
    if source.contains("private")
        || source.contains("evaluation")
        || source.contains("prospective")
        || source.contains("future")
    {
        return Err("historical dataset private evaluation source rejected".to_string());
    }
    let rows = &snapshot.normalized_dataset.rows;
    if rows.is_empty()
        || snapshot.normalized_dataset.symbol.is_empty()
        || snapshot.symbols != vec![snapshot.normalized_dataset.symbol.clone()]
    {
        return Err("historical dataset identity rejected".to_string());
    }
    let mut timestamps = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        if !timestamps.insert(row.timestamp_ms)
            || row.open <= 0.0
            || row.high <= 0.0
            || row.low <= 0.0
            || row.close <= 0.0
            || row.volume < 0.0
            || row.high < row.low
            || row.high < row.open.max(row.close)
            || row.low > row.open.min(row.close)
        {
            return Err(format!("historical row {index} rejected"));
        }
    }
    if rows
        .windows(2)
        .any(|pair| pair[1].timestamp_ms <= pair[0].timestamp_ms)
    {
        return Err("historical chronology rejected".to_string());
    }
    if snapshot.market_scope == AcquisitionMarketScope::BtcCrypto
        && rows
            .windows(2)
            .any(|pair| pair[1].timestamp_ms - pair[0].timestamp_ms != DAILY_CADENCE_MS)
    {
        return Err("historical daily cadence gap rejected".to_string());
    }
    if snapshot.actual_start_timestamp_ms != rows.first().map(|row| row.timestamp_ms)
        || snapshot.actual_end_timestamp_ms != rows.last().map(|row| row.timestamp_ms)
    {
        return Err("historical timestamp provenance rejected".to_string());
    }
    Ok(())
}

fn build_dataset_snapshot(
    source: &DataSnapshot,
) -> Result<MomentumHistoricalDatasetSnapshotV1, String> {
    validate_dataset_source(source)?;
    let campaign = MomentumLearningCampaignConfigV0::default();
    let rows = &source.normalized_dataset.rows;
    let mut source_capsule_digests = vec![
        source.content_digest.clone(),
        stable_hash_string(&format!(
            "historical-source-capsule:{}:{}:{}",
            source.snapshot_id,
            source.provenance.acquisition_request_id,
            source.provenance.fetch_receipt_id,
        )),
    ];
    source_capsule_digests.sort();
    source_capsule_digests.dedup();
    let mut value = MomentumHistoricalDatasetSnapshotV1 {
        snapshot_version: SNAPSHOT_VERSION.to_string(),
        provider_id: source.provider_id.clone(),
        market: format!("{:?}", source.market_scope),
        symbol: source.normalized_dataset.symbol.clone(),
        cadence: source
            .compatibility
            .as_ref()
            .map(|value| value.cadence.clone())
            .ok_or_else(|| "historical cadence unavailable".to_string())?,
        first_timestamp_ms: rows
            .first()
            .map(|row| row.timestamp_ms)
            .ok_or_else(|| "historical first row unavailable".to_string())?,
        last_timestamp_ms: rows
            .last()
            .map(|row| row.timestamp_ms)
            .ok_or_else(|| "historical last row unavailable".to_string())?,
        row_count: rows.len(),
        ordered_row_digests: rows.iter().map(row_digest).collect(),
        source_capsule_digests,
        dataset_aggregate_digest: source.content_digest.clone(),
        feature_policy_digest: campaign.feature_config.digest(),
        label_policy_digest: label_policy_digest(&campaign.sequence_config),
        evidence_use_class: HistoricalEvidenceUseClassV1::PreviouslyConsumedResearchEvidence,
        previously_consumed: true,
        blind_holdout: false,
        authority_eligible: false,
        snapshot_digest: String::new(),
    };
    value.snapshot_digest = snapshot_digest(&value);
    validate_dataset_snapshot(&value)?;
    Ok(value)
}

fn validate_dataset_snapshot(value: &MomentumHistoricalDatasetSnapshotV1) -> Result<(), String> {
    if value.snapshot_version != SNAPSHOT_VERSION
        || value.provider_id.is_empty()
        || value.symbol.is_empty()
        || value.cadence != "1d"
        || value.row_count == 0
        || value.ordered_row_digests.len() != value.row_count
        || value.ordered_row_digests.iter().any(String::is_empty)
        || value.source_capsule_digests.is_empty()
        || value.dataset_aggregate_digest.is_empty()
        || value.feature_policy_digest.is_empty()
        || value.label_policy_digest.is_empty()
        || value.evidence_use_class
            != HistoricalEvidenceUseClassV1::PreviouslyConsumedResearchEvidence
        || !value.previously_consumed
        || value.blind_holdout
        || value.authority_eligible
        || value.first_timestamp_ms >= value.last_timestamp_ms
        || value.snapshot_digest != snapshot_digest(value)
    {
        return Err("historical dataset snapshot rejected".to_string());
    }
    Ok(())
}

fn discover_dataset(
    root: &Path,
) -> Result<(DataSnapshot, MomentumHistoricalDatasetSnapshotV1), String> {
    let snapshots = super::load_local_learning_snapshots_v0(root)?;
    if snapshots.is_empty() {
        return Err("canonical historical dataset unavailable".to_string());
    }
    let valid_snapshots = snapshots
        .into_iter()
        .filter(|snapshot| validate_dataset_source(snapshot).is_ok())
        .collect::<Vec<_>>();
    if valid_snapshots.is_empty() {
        return Err("canonical historical dataset validation failed".to_string());
    }
    let maximum_rows = valid_snapshots
        .iter()
        .map(|snapshot| snapshot.row_count)
        .max()
        .ok_or_else(|| "canonical historical dataset unavailable".to_string())?;
    let mut candidates = valid_snapshots
        .into_iter()
        .filter(|snapshot| snapshot.row_count == maximum_rows)
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.content_digest.cmp(&right.content_digest));
    let first_digest = candidates
        .first()
        .map(|snapshot| snapshot.content_digest.clone())
        .ok_or_else(|| "canonical historical dataset unavailable".to_string())?;
    if candidates
        .iter()
        .any(|snapshot| snapshot.content_digest != first_digest)
    {
        return Err("ambiguous canonical historical dataset rejected".to_string());
    }
    let source = candidates.remove(0);
    let snapshot = build_dataset_snapshot(&source)?;
    Ok((source, snapshot))
}

fn contamination_audit_digest(value: &MomentumHistoricalContaminationAuditV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        value.audit_version,
        value.dataset_snapshot_digest,
        value.used_for_feature_design,
        value.used_for_model_design,
        value.used_for_hyperparameter_design,
        value.used_for_qualification,
        value.used_for_participant_selection,
        value.used_for_prior_reporting,
        value.conservative_unknown_history_assumed_consumed,
        value.independent_holdout_claim_forbidden,
        value.live_authority_use_forbidden,
    ))
}

fn build_contamination_audit(
    snapshot: &MomentumHistoricalDatasetSnapshotV1,
) -> MomentumHistoricalContaminationAuditV1 {
    let mut value = MomentumHistoricalContaminationAuditV1 {
        audit_version: AUDIT_VERSION.to_string(),
        dataset_snapshot_digest: snapshot.snapshot_digest.clone(),
        used_for_feature_design: true,
        used_for_model_design: true,
        used_for_hyperparameter_design: true,
        used_for_qualification: true,
        used_for_participant_selection: true,
        used_for_prior_reporting: true,
        conservative_unknown_history_assumed_consumed: true,
        independent_holdout_claim_forbidden: true,
        live_authority_use_forbidden: true,
        audit_digest: String::new(),
    };
    value.audit_digest = contamination_audit_digest(&value);
    value
}

fn validate_contamination_audit(
    value: &MomentumHistoricalContaminationAuditV1,
) -> Result<(), String> {
    if value.audit_version != AUDIT_VERSION
        || value.dataset_snapshot_digest.is_empty()
        || !value.used_for_feature_design
        || !value.used_for_model_design
        || !value.used_for_hyperparameter_design
        || !value.used_for_qualification
        || !value.used_for_participant_selection
        || !value.used_for_prior_reporting
        || !value.conservative_unknown_history_assumed_consumed
        || !value.independent_holdout_claim_forbidden
        || !value.live_authority_use_forbidden
        || value.audit_digest != contamination_audit_digest(value)
    {
        return Err("historical contamination audit rejected".to_string());
    }
    Ok(())
}

fn candles(rows: &[HistoricalOhlcvRow]) -> Result<Vec<MomentumCandleV0>, String> {
    rows.iter()
        .map(|row| {
            let timestamp = i64::try_from(row.timestamp_ms)
                .map_err(|_| "historical candle timestamp overflow".to_string())?;
            let candle = MomentumCandleV0 {
                timestamp,
                open: row.open as f32,
                high: row.high as f32,
                low: row.low as f32,
                close: row.close as f32,
                volume: row.volume as f32,
            };
            if [
                candle.open,
                candle.high,
                candle.low,
                candle.close,
                candle.volume,
            ]
            .iter()
            .any(|value| !value.is_finite())
            {
                return Err("historical candle precision conversion rejected".to_string());
            }
            Ok(candle)
        })
        .collect()
}

fn training_examples(
    rows_through_prediction_event: &[HistoricalOhlcvRow],
    snapshot_digest: &str,
    feature_config: &MomentumFeatureConfigV0,
    sequence_config: &MomentumSequenceConfigV0,
) -> Result<Vec<super::SequenceExampleV0>, String> {
    let candle_rows = candles(rows_through_prediction_event)?;
    let features = build_momentum_features_v0(&candle_rows, feature_config)
        .map_err(|_| "historical feature construction rejected".to_string())?;
    build_momentum_sequence_examples_v0(
        &candle_rows,
        &features,
        sequence_config,
        &[snapshot_digest.to_string()],
    )
    .map_err(|_| "historical training examples unavailable".to_string())
}

fn eligible_event_indices(
    rows: &[HistoricalOhlcvRow],
    snapshot_digest: &str,
    campaign: &MomentumLearningCampaignConfigV0,
    context_row_count: usize,
) -> Result<Vec<usize>, String> {
    let mut eligible = Vec::new();
    if rows.len() <= context_row_count {
        return Ok(eligible);
    }
    for prediction_event_index in (context_row_count - 1)..rows.len().saturating_sub(1) {
        let Ok(examples) = training_examples(
            &rows[..=prediction_event_index],
            snapshot_digest,
            &campaign.feature_config,
            &campaign.sequence_config,
        ) else {
            continue;
        };
        if examples.len() < campaign.train_rows {
            continue;
        }
        if examples.iter().any(|example| {
            example.sequence_end >= prediction_event_index
                || example.label_index > prediction_event_index
        }) {
            return Err("historical eligibility leakage rejected".to_string());
        }
        eligible.push(prediction_event_index);
    }
    Ok(eligible)
}

fn registration_digest(value: &MomentumHistoricalReplayRegistrationV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}:{:?}:{:?}:{}:{}:{}:{}:{}:{}",
        value.registration_version,
        value.dataset_snapshot_digest,
        value.contamination_audit_digest,
        value.replay_mode,
        value.context_row_count,
        value.prediction_horizon,
        value.minimum_training_examples,
        value.first_eligible_event_index,
        value.final_eligible_event_index,
        value.training_policy_digest,
        value.feature_policy_digest,
        value.label_policy_digest,
        value.evaluation_policy_digest,
        value.interaction_schema_digest,
        value.participant_templates,
        value.initialization_seeds,
        value.fold_selection_policy,
        value.result_conditioned_fold_selection_forbidden,
        value.result_conditioned_hyperparameters_forbidden,
        value.live_prospective_write_forbidden,
        value.reward_application_forbidden,
        value.chair_action_forbidden,
        value.trading_authority_forbidden,
    ))
}

fn build_registration(
    rows: &[HistoricalOhlcvRow],
    snapshot: &MomentumHistoricalDatasetSnapshotV1,
    audit: &MomentumHistoricalContaminationAuditV1,
    replay_mode: MomentumHistoricalReplayModeV1,
) -> Result<(MomentumHistoricalReplayRegistrationV1, Vec<usize>), String> {
    let campaign = MomentumLearningCampaignConfigV0::default();
    campaign
        .feature_config
        .validate()
        .map_err(|_| "historical feature policy rejected".to_string())?;
    campaign
        .sequence_config
        .validate()
        .map_err(|_| "historical label policy rejected".to_string())?;
    campaign
        .training_config
        .validate()
        .map_err(|_| "historical training policy rejected".to_string())?;
    if campaign.sequence_config.prediction_horizon != 1
        || campaign.sequence_config.include_neutral_labels
    {
        return Err("historical frozen V4 label policy rejected".to_string());
    }
    let context_row_count = campaign
        .feature_config
        .minimum_history()
        .map_err(|_| "historical context policy rejected".to_string())?
        .checked_add(campaign.sequence_config.sequence_length.saturating_sub(1))
        .ok_or_else(|| "historical context overflow".to_string())?;
    let eligible = eligible_event_indices(
        rows,
        &snapshot.snapshot_digest,
        &campaign,
        context_row_count,
    )?;
    let first_eligible_event_index = *eligible
        .first()
        .ok_or_else(|| "historical eligible folds unavailable".to_string())?;
    let final_eligible_event_index = *eligible
        .last()
        .ok_or_else(|| "historical eligible folds unavailable".to_string())?;
    let seeds = vec![
        campaign.campaign_seed,
        campaign.campaign_seed.saturating_add(101),
    ];
    let mut value = MomentumHistoricalReplayRegistrationV1 {
        registration_version: REGISTRATION_VERSION.to_string(),
        dataset_snapshot_digest: snapshot.snapshot_digest.clone(),
        contamination_audit_digest: audit.audit_digest.clone(),
        replay_mode,
        context_row_count,
        prediction_horizon: campaign.sequence_config.prediction_horizon,
        minimum_training_examples: campaign.train_rows,
        first_eligible_event_index,
        final_eligible_event_index,
        training_policy_digest: stable_hash_string(&format!(
            "historical-training-policy-v1:{}:{}:{}:{}",
            campaign.training_config.digest(),
            campaign.train_rows,
            campaign.campaign_seed,
            "fold-local-fresh-initialization"
        )),
        feature_policy_digest: snapshot.feature_policy_digest.clone(),
        label_policy_digest: snapshot.label_policy_digest.clone(),
        evaluation_policy_digest: stable_hash_string(
            "historical-evaluation-v1:evaluate-probabilities-v0:neutral-excluded:no-winner",
        ),
        interaction_schema_digest: interaction_schema_digest(6),
        participant_templates: vec![
            RAW_PARTICIPANT.to_string(),
            INTERACTION_PARTICIPANT.to_string(),
            CONSTANT_PARTICIPANT.to_string(),
        ],
        initialization_seeds: seeds,
        fold_selection_policy:
            MomentumHistoricalFoldSelectionPolicyV1::EveryChronologicallyEligibleEvent,
        result_conditioned_fold_selection_forbidden: true,
        result_conditioned_hyperparameters_forbidden: true,
        live_prospective_write_forbidden: true,
        reward_application_forbidden: true,
        chair_action_forbidden: true,
        trading_authority_forbidden: true,
        registration_digest: String::new(),
    };
    value.registration_digest = registration_digest(&value);
    validate_registration(&value, rows.len())?;
    Ok((value, eligible))
}

fn validate_registration(
    value: &MomentumHistoricalReplayRegistrationV1,
    row_count: usize,
) -> Result<(), String> {
    if value.registration_version != REGISTRATION_VERSION
        || value.dataset_snapshot_digest.is_empty()
        || value.contamination_audit_digest.is_empty()
        || value.context_row_count == 0
        || value.prediction_horizon != 1
        || value.minimum_training_examples == 0
        || value.first_eligible_event_index > value.final_eligible_event_index
        || value.final_eligible_event_index + value.prediction_horizon >= row_count
        || value.participant_templates
            != [
                RAW_PARTICIPANT.to_string(),
                INTERACTION_PARTICIPANT.to_string(),
                CONSTANT_PARTICIPANT.to_string(),
            ]
        || value.initialization_seeds.len() != 2
        || value.fold_selection_policy
            != MomentumHistoricalFoldSelectionPolicyV1::EveryChronologicallyEligibleEvent
        || !value.result_conditioned_fold_selection_forbidden
        || !value.result_conditioned_hyperparameters_forbidden
        || !value.live_prospective_write_forbidden
        || !value.reward_application_forbidden
        || !value.chair_action_forbidden
        || !value.trading_authority_forbidden
        || value.registration_digest != registration_digest(value)
    {
        return Err("historical replay registration rejected".to_string());
    }
    Ok(())
}

fn fold_plan_digest(value: &MomentumHistoricalFoldPlanV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{:?}:{:?}:{}",
        value.registration_digest,
        value.fold_number,
        value.prediction_event_index,
        value.target_index,
        value.prediction_event_timestamp_ms,
        value.target_timestamp_ms,
        value.training_event_timestamp_ms,
        value.context_timestamp_ms,
        value.latest_training_label_timestamp_ms,
    ))
}

fn build_fold_plan(
    fold_number: usize,
    prediction_event_index: usize,
    rows: &[HistoricalOhlcvRow],
    registration: &MomentumHistoricalReplayRegistrationV1,
    examples: &[super::SequenceExampleV0],
) -> Result<MomentumHistoricalFoldPlanV1, String> {
    let target_index = prediction_event_index
        .checked_add(registration.prediction_horizon)
        .ok_or_else(|| "historical target index overflow".to_string())?;
    let context_start = prediction_event_index
        .checked_add(1)
        .and_then(|value| value.checked_sub(registration.context_row_count))
        .ok_or_else(|| "historical context underflow".to_string())?;
    let latest_label_index = examples
        .iter()
        .map(|example| example.label_index)
        .max()
        .ok_or_else(|| "historical training labels unavailable".to_string())?;
    let mut value = MomentumHistoricalFoldPlanV1 {
        registration_digest: registration.registration_digest.clone(),
        fold_number: u64::try_from(fold_number)
            .map_err(|_| "historical fold number overflow".to_string())?,
        prediction_event_index,
        target_index,
        prediction_event_timestamp_ms: rows[prediction_event_index].timestamp_ms,
        target_timestamp_ms: rows[target_index].timestamp_ms,
        training_event_timestamp_ms: examples
            .iter()
            .map(|example| rows[example.sequence_end].timestamp_ms)
            .collect(),
        context_timestamp_ms: rows[context_start..=prediction_event_index]
            .iter()
            .map(|row| row.timestamp_ms)
            .collect(),
        latest_training_label_timestamp_ms: rows[latest_label_index].timestamp_ms,
        fold_plan_digest: String::new(),
    };
    value.fold_plan_digest = fold_plan_digest(&value);
    validate_fold_plan(&value, registration)?;
    Ok(value)
}

fn validate_fold_plan(
    value: &MomentumHistoricalFoldPlanV1,
    registration: &MomentumHistoricalReplayRegistrationV1,
) -> Result<(), String> {
    if value.registration_digest != registration.registration_digest
        || value.target_index != value.prediction_event_index + registration.prediction_horizon
        || value.target_timestamp_ms <= value.prediction_event_timestamp_ms
        || value.context_timestamp_ms.len() != registration.context_row_count
        || value.context_timestamp_ms.last().copied() != Some(value.prediction_event_timestamp_ms)
        || value
            .context_timestamp_ms
            .windows(2)
            .any(|pair| pair[1] <= pair[0])
        || value.training_event_timestamp_ms.len() < registration.minimum_training_examples
        || value
            .training_event_timestamp_ms
            .iter()
            .any(|timestamp| *timestamp >= value.prediction_event_timestamp_ms)
        || value.latest_training_label_timestamp_ms > value.prediction_event_timestamp_ms
        || value.fold_plan_digest != fold_plan_digest(value)
    {
        return Err("historical fold chronology rejected".to_string());
    }
    Ok(())
}

fn classify_label(
    prediction_row: &HistoricalOhlcvRow,
    target_row: &HistoricalOhlcvRow,
    sequence_config: &MomentumSequenceConfigV0,
) -> Result<(HistoricalLabelStatusV1, Option<f64>), String> {
    let frozen = MomentumSequenceConfigV0::default();
    if prediction_row.timestamp_ms >= target_row.timestamp_ms
        || sequence_config.prediction_horizon != frozen.prediction_horizon
        || sequence_config.label_dead_zone.to_bits() != frozen.label_dead_zone.to_bits()
        || sequence_config.include_neutral_labels != frozen.include_neutral_labels
    {
        return Ok((HistoricalLabelStatusV1::InvalidOutcomeEvidence, None));
    }
    match classify_label_v4_4(prediction_row.close, target_row.close) {
        Ok((MomentumProspectiveLabelStatusV4_4::ScorableBinaryOutcome, Some(label), _)) => Ok((
            HistoricalLabelStatusV1::ScorableBinaryOutcome,
            Some(if label { 1.0 } else { 0.0 }),
        )),
        Ok((MomentumProspectiveLabelStatusV4_4::NeutralOutcomeExcluded, None, _)) => {
            Ok((HistoricalLabelStatusV1::NeutralOutcomeExcluded, None))
        }
        Ok(_) | Err(_) => Ok((HistoricalLabelStatusV1::InvalidOutcomeEvidence, None)),
    }
}

fn historical_evidence_class(value: &str) -> Result<HistoricalEvidenceUseClassV1, String> {
    match value {
        "ProtocolReplayOnly" => Ok(HistoricalEvidenceUseClassV1::ProtocolReplayOnly),
        "PreviouslyConsumedResearchEvidence" => {
            Ok(HistoricalEvidenceUseClassV1::PreviouslyConsumedResearchEvidence)
        }
        "CausalWalkForwardResearch" => Ok(HistoricalEvidenceUseClassV1::CausalWalkForwardResearch),
        "SealedHistoricalHoldout" => Ok(HistoricalEvidenceUseClassV1::SealedHistoricalHoldout),
        _ => Err("historical evidence class rejected".to_string()),
    }
}

fn replay_mode(value: &str) -> Result<MomentumHistoricalReplayModeV1, String> {
    match value {
        "ProtocolReplay" => Ok(MomentumHistoricalReplayModeV1::ProtocolReplay),
        "ExpandingWindowWalkForward" => {
            Ok(MomentumHistoricalReplayModeV1::ExpandingWindowWalkForward)
        }
        _ => Err("historical replay mode artifact rejected".to_string()),
    }
}

fn label_status(value: &str) -> Result<HistoricalLabelStatusV1, String> {
    match value {
        "ScorableBinaryOutcome" => Ok(HistoricalLabelStatusV1::ScorableBinaryOutcome),
        "NeutralOutcomeExcluded" => Ok(HistoricalLabelStatusV1::NeutralOutcomeExcluded),
        "InvalidOutcomeEvidence" => Ok(HistoricalLabelStatusV1::InvalidOutcomeEvidence),
        _ => Err("historical label status rejected".to_string()),
    }
}

fn comparison_status(value: &str) -> Result<MomentumHistoricalComparisonStatusV1, String> {
    match value {
        "LearnedBetterOnResearchReplay" => {
            Ok(MomentumHistoricalComparisonStatusV1::LearnedBetterOnResearchReplay)
        }
        "BenchmarkBetterOnResearchReplay" => {
            Ok(MomentumHistoricalComparisonStatusV1::BenchmarkBetterOnResearchReplay)
        }
        "MixedResearchEvidence" => Ok(MomentumHistoricalComparisonStatusV1::MixedResearchEvidence),
        "InsufficientScorableFolds" => {
            Ok(MomentumHistoricalComparisonStatusV1::InsufficientScorableFolds)
        }
        "IntegrityFailure" => Ok(MomentumHistoricalComparisonStatusV1::IntegrityFailure),
        _ => Err("historical comparison status rejected".to_string()),
    }
}

fn encode_dataset_snapshot(value: &MomentumHistoricalDatasetSnapshotV1) -> Result<Vec<u8>, String> {
    validate_dataset_snapshot(value)?;
    ArtifactBuilderV4_2::new("MomentumHistoricalDatasetSnapshotV1")
        .string("snapshot_version", &value.snapshot_version)
        .string("provider_id", &value.provider_id)
        .string("market", &value.market)
        .string("symbol", &value.symbol)
        .string("cadence", &value.cadence)
        .unsigned("first_timestamp_ms", value.first_timestamp_ms)
        .unsigned("last_timestamp_ms", value.last_timestamp_ms)
        .unsigned("row_count", as_u64(value.row_count)?)
        .strings("ordered_row_digests", &value.ordered_row_digests)
        .strings("source_capsule_digests", &value.source_capsule_digests)
        .string("dataset_aggregate_digest", &value.dataset_aggregate_digest)
        .string("feature_policy_digest", &value.feature_policy_digest)
        .string("label_policy_digest", &value.label_policy_digest)
        .string(
            "evidence_use_class",
            format!("{:?}", value.evidence_use_class),
        )
        .boolean("previously_consumed", value.previously_consumed)
        .boolean("blind_holdout", value.blind_holdout)
        .boolean("authority_eligible", value.authority_eligible)
        .string("snapshot_digest", &value.snapshot_digest)
        .encode()
}

fn decode_dataset_snapshot(bytes: &[u8]) -> Result<MomentumHistoricalDatasetSnapshotV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumHistoricalDatasetSnapshotV1")?;
    let value = MomentumHistoricalDatasetSnapshotV1 {
        snapshot_version: fields.string("snapshot_version")?,
        provider_id: fields.string("provider_id")?,
        market: fields.string("market")?,
        symbol: fields.string("symbol")?,
        cadence: fields.string("cadence")?,
        first_timestamp_ms: fields.unsigned("first_timestamp_ms")?,
        last_timestamp_ms: fields.unsigned("last_timestamp_ms")?,
        row_count: as_usize(fields.unsigned("row_count")?)?,
        ordered_row_digests: fields.strings("ordered_row_digests")?,
        source_capsule_digests: fields.strings("source_capsule_digests")?,
        dataset_aggregate_digest: fields.string("dataset_aggregate_digest")?,
        feature_policy_digest: fields.string("feature_policy_digest")?,
        label_policy_digest: fields.string("label_policy_digest")?,
        evidence_use_class: historical_evidence_class(&fields.string("evidence_use_class")?)?,
        previously_consumed: fields.boolean("previously_consumed")?,
        blind_holdout: fields.boolean("blind_holdout")?,
        authority_eligible: fields.boolean("authority_eligible")?,
        snapshot_digest: fields.string("snapshot_digest")?,
    };
    fields.finish()?;
    validate_dataset_snapshot(&value)?;
    Ok(value)
}

fn encode_contamination_audit(
    value: &MomentumHistoricalContaminationAuditV1,
) -> Result<Vec<u8>, String> {
    validate_contamination_audit(value)?;
    ArtifactBuilderV4_2::new("MomentumHistoricalContaminationAuditV1")
        .string("audit_version", &value.audit_version)
        .string("dataset_snapshot_digest", &value.dataset_snapshot_digest)
        .boolean("used_for_feature_design", value.used_for_feature_design)
        .boolean("used_for_model_design", value.used_for_model_design)
        .boolean(
            "used_for_hyperparameter_design",
            value.used_for_hyperparameter_design,
        )
        .boolean("used_for_qualification", value.used_for_qualification)
        .boolean(
            "used_for_participant_selection",
            value.used_for_participant_selection,
        )
        .boolean("used_for_prior_reporting", value.used_for_prior_reporting)
        .boolean(
            "conservative_unknown_history_assumed_consumed",
            value.conservative_unknown_history_assumed_consumed,
        )
        .boolean(
            "independent_holdout_claim_forbidden",
            value.independent_holdout_claim_forbidden,
        )
        .boolean(
            "live_authority_use_forbidden",
            value.live_authority_use_forbidden,
        )
        .string("audit_digest", &value.audit_digest)
        .encode()
}

fn decode_contamination_audit(
    bytes: &[u8],
) -> Result<MomentumHistoricalContaminationAuditV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumHistoricalContaminationAuditV1")?;
    let value = MomentumHistoricalContaminationAuditV1 {
        audit_version: fields.string("audit_version")?,
        dataset_snapshot_digest: fields.string("dataset_snapshot_digest")?,
        used_for_feature_design: fields.boolean("used_for_feature_design")?,
        used_for_model_design: fields.boolean("used_for_model_design")?,
        used_for_hyperparameter_design: fields.boolean("used_for_hyperparameter_design")?,
        used_for_qualification: fields.boolean("used_for_qualification")?,
        used_for_participant_selection: fields.boolean("used_for_participant_selection")?,
        used_for_prior_reporting: fields.boolean("used_for_prior_reporting")?,
        conservative_unknown_history_assumed_consumed: fields
            .boolean("conservative_unknown_history_assumed_consumed")?,
        independent_holdout_claim_forbidden: fields
            .boolean("independent_holdout_claim_forbidden")?,
        live_authority_use_forbidden: fields.boolean("live_authority_use_forbidden")?,
        audit_digest: fields.string("audit_digest")?,
    };
    fields.finish()?;
    validate_contamination_audit(&value)?;
    Ok(value)
}

fn encode_registration(
    value: &MomentumHistoricalReplayRegistrationV1,
    row_count: usize,
) -> Result<Vec<u8>, String> {
    validate_registration(value, row_count)?;
    ArtifactBuilderV4_2::new("MomentumHistoricalReplayRegistrationV1")
        .string("registration_version", &value.registration_version)
        .string("dataset_snapshot_digest", &value.dataset_snapshot_digest)
        .string(
            "contamination_audit_digest",
            &value.contamination_audit_digest,
        )
        .string("replay_mode", format!("{:?}", value.replay_mode))
        .unsigned("context_row_count", as_u64(value.context_row_count)?)
        .unsigned("prediction_horizon", as_u64(value.prediction_horizon)?)
        .unsigned(
            "minimum_training_examples",
            as_u64(value.minimum_training_examples)?,
        )
        .unsigned(
            "first_eligible_event_index",
            as_u64(value.first_eligible_event_index)?,
        )
        .unsigned(
            "final_eligible_event_index",
            as_u64(value.final_eligible_event_index)?,
        )
        .string("training_policy_digest", &value.training_policy_digest)
        .string("feature_policy_digest", &value.feature_policy_digest)
        .string("label_policy_digest", &value.label_policy_digest)
        .string("evaluation_policy_digest", &value.evaluation_policy_digest)
        .string(
            "interaction_schema_digest",
            &value.interaction_schema_digest,
        )
        .strings("participant_templates", &value.participant_templates)
        .unsigneds("initialization_seeds", &value.initialization_seeds)
        .string(
            "fold_selection_policy",
            format!("{:?}", value.fold_selection_policy),
        )
        .boolean(
            "result_conditioned_fold_selection_forbidden",
            value.result_conditioned_fold_selection_forbidden,
        )
        .boolean(
            "result_conditioned_hyperparameters_forbidden",
            value.result_conditioned_hyperparameters_forbidden,
        )
        .boolean(
            "live_prospective_write_forbidden",
            value.live_prospective_write_forbidden,
        )
        .boolean(
            "reward_application_forbidden",
            value.reward_application_forbidden,
        )
        .boolean("chair_action_forbidden", value.chair_action_forbidden)
        .boolean(
            "trading_authority_forbidden",
            value.trading_authority_forbidden,
        )
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_registration(
    bytes: &[u8],
    row_count: usize,
) -> Result<MomentumHistoricalReplayRegistrationV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumHistoricalReplayRegistrationV1")?;
    let fold_policy = fields.string("fold_selection_policy")?;
    if fold_policy != "EveryChronologicallyEligibleEvent" {
        return Err("historical fold policy rejected".to_string());
    }
    let value = MomentumHistoricalReplayRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        dataset_snapshot_digest: fields.string("dataset_snapshot_digest")?,
        contamination_audit_digest: fields.string("contamination_audit_digest")?,
        replay_mode: replay_mode(&fields.string("replay_mode")?)?,
        context_row_count: as_usize(fields.unsigned("context_row_count")?)?,
        prediction_horizon: as_usize(fields.unsigned("prediction_horizon")?)?,
        minimum_training_examples: as_usize(fields.unsigned("minimum_training_examples")?)?,
        first_eligible_event_index: as_usize(fields.unsigned("first_eligible_event_index")?)?,
        final_eligible_event_index: as_usize(fields.unsigned("final_eligible_event_index")?)?,
        training_policy_digest: fields.string("training_policy_digest")?,
        feature_policy_digest: fields.string("feature_policy_digest")?,
        label_policy_digest: fields.string("label_policy_digest")?,
        evaluation_policy_digest: fields.string("evaluation_policy_digest")?,
        interaction_schema_digest: fields.string("interaction_schema_digest")?,
        participant_templates: fields.strings("participant_templates")?,
        initialization_seeds: fields.unsigneds("initialization_seeds")?,
        fold_selection_policy:
            MomentumHistoricalFoldSelectionPolicyV1::EveryChronologicallyEligibleEvent,
        result_conditioned_fold_selection_forbidden: fields
            .boolean("result_conditioned_fold_selection_forbidden")?,
        result_conditioned_hyperparameters_forbidden: fields
            .boolean("result_conditioned_hyperparameters_forbidden")?,
        live_prospective_write_forbidden: fields.boolean("live_prospective_write_forbidden")?,
        reward_application_forbidden: fields.boolean("reward_application_forbidden")?,
        chair_action_forbidden: fields.boolean("chair_action_forbidden")?,
        trading_authority_forbidden: fields.boolean("trading_authority_forbidden")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_registration(&value, row_count)?;
    Ok(value)
}

fn encode_fold_plan(value: &MomentumHistoricalFoldPlanV1) -> Result<Vec<u8>, String> {
    ArtifactBuilderV4_2::new("MomentumHistoricalFoldPlanV1")
        .string("registration_digest", &value.registration_digest)
        .unsigned("fold_number", value.fold_number)
        .unsigned(
            "prediction_event_index",
            as_u64(value.prediction_event_index)?,
        )
        .unsigned("target_index", as_u64(value.target_index)?)
        .unsigned(
            "prediction_event_timestamp_ms",
            value.prediction_event_timestamp_ms,
        )
        .unsigned("target_timestamp_ms", value.target_timestamp_ms)
        .unsigneds(
            "training_event_timestamp_ms",
            &value.training_event_timestamp_ms,
        )
        .unsigneds("context_timestamp_ms", &value.context_timestamp_ms)
        .unsigned(
            "latest_training_label_timestamp_ms",
            value.latest_training_label_timestamp_ms,
        )
        .string("fold_plan_digest", &value.fold_plan_digest)
        .encode()
}

fn decode_fold_plan(
    bytes: &[u8],
    registration: &MomentumHistoricalReplayRegistrationV1,
) -> Result<MomentumHistoricalFoldPlanV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumHistoricalFoldPlanV1")?;
    let value = MomentumHistoricalFoldPlanV1 {
        registration_digest: fields.string("registration_digest")?,
        fold_number: fields.unsigned("fold_number")?,
        prediction_event_index: as_usize(fields.unsigned("prediction_event_index")?)?,
        target_index: as_usize(fields.unsigned("target_index")?)?,
        prediction_event_timestamp_ms: fields.unsigned("prediction_event_timestamp_ms")?,
        target_timestamp_ms: fields.unsigned("target_timestamp_ms")?,
        training_event_timestamp_ms: fields.unsigneds("training_event_timestamp_ms")?,
        context_timestamp_ms: fields.unsigneds("context_timestamp_ms")?,
        latest_training_label_timestamp_ms: fields
            .unsigned("latest_training_label_timestamp_ms")?,
        fold_plan_digest: fields.string("fold_plan_digest")?,
    };
    fields.finish()?;
    validate_fold_plan(&value, registration)?;
    Ok(value)
}

fn prediction_digest(value: &MomentumHistoricalFoldPredictionV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}",
        value.fold_plan_digest,
        value.participant_id,
        value.participant_role,
        value.parameter_digest,
        value.normalizer_digest,
        value.feature_digest,
        value.private_prediction.to_bits(),
        value.target_accessed,
    ))
}

fn prediction_identity(value: &MomentumHistoricalFoldPredictionV1) -> String {
    stable_hash_string(&format!(
        "historical-prediction-seal-v1:{}:{}:{}",
        value.fold_plan_digest, value.participant_id, value.prediction_digest
    ))
}

fn build_prediction(
    plan: &MomentumHistoricalFoldPlanV1,
    participant_id: &str,
    participant_role: &str,
    parameter_digest: String,
    normalizer_digest: String,
    feature_digest: String,
    probability: f64,
) -> Result<MomentumHistoricalFoldPredictionV1, String> {
    let mut value = MomentumHistoricalFoldPredictionV1 {
        fold_plan_digest: plan.fold_plan_digest.clone(),
        participant_id: participant_id.to_string(),
        participant_role: participant_role.to_string(),
        parameter_digest,
        normalizer_digest,
        feature_digest,
        private_prediction: probability,
        prediction_digest: String::new(),
        target_accessed: false,
        prediction_digest_identity: String::new(),
    };
    value.prediction_digest = prediction_digest(&value);
    value.prediction_digest_identity = prediction_identity(&value);
    validate_prediction(&value)?;
    Ok(value)
}

fn validate_prediction(value: &MomentumHistoricalFoldPredictionV1) -> Result<(), String> {
    if value.fold_plan_digest.is_empty()
        || ![
            RAW_PARTICIPANT,
            INTERACTION_PARTICIPANT,
            CONSTANT_PARTICIPANT,
        ]
        .contains(&value.participant_id.as_str())
        || value.participant_role != "HistoricalResearchReplica"
        || !value.parameter_digest.starts_with("historical-research:")
        || !value.normalizer_digest.starts_with("historical-research:")
        || value.feature_digest.is_empty()
        || !value.private_prediction.is_finite()
        || !(0.0..=1.0).contains(&value.private_prediction)
        || value.target_accessed
        || value.prediction_digest != prediction_digest(value)
        || value.prediction_digest_identity != prediction_identity(value)
    {
        return Err("historical prediction seal rejected".to_string());
    }
    Ok(())
}

fn encode_prediction(value: &MomentumHistoricalFoldPredictionV1) -> Result<Vec<u8>, String> {
    validate_prediction(value)?;
    ArtifactBuilderV4_2::new("MomentumHistoricalFoldPredictionV1")
        .string("fold_plan_digest", &value.fold_plan_digest)
        .string("participant_id", &value.participant_id)
        .string("participant_role", &value.participant_role)
        .string("parameter_digest", &value.parameter_digest)
        .string("normalizer_digest", &value.normalizer_digest)
        .string("feature_digest", &value.feature_digest)
        .unsigned(
            "private_prediction_bits",
            value.private_prediction.to_bits(),
        )
        .string("prediction_digest", &value.prediction_digest)
        .boolean("target_accessed", value.target_accessed)
        .string(
            "prediction_digest_identity",
            &value.prediction_digest_identity,
        )
        .encode()
}

fn decode_prediction(bytes: &[u8]) -> Result<MomentumHistoricalFoldPredictionV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumHistoricalFoldPredictionV1")?;
    let value = MomentumHistoricalFoldPredictionV1 {
        fold_plan_digest: fields.string("fold_plan_digest")?,
        participant_id: fields.string("participant_id")?,
        participant_role: fields.string("participant_role")?,
        parameter_digest: fields.string("parameter_digest")?,
        normalizer_digest: fields.string("normalizer_digest")?,
        feature_digest: fields.string("feature_digest")?,
        private_prediction: f64::from_bits(fields.unsigned("private_prediction_bits")?),
        prediction_digest: fields.string("prediction_digest")?,
        target_accessed: fields.boolean("target_accessed")?,
        prediction_digest_identity: fields.string("prediction_digest_identity")?,
    };
    fields.finish()?;
    validate_prediction(&value)?;
    Ok(value)
}

fn capsule_digest(value: &MomentumHistoricalFoldPredictionCapsuleV1) -> String {
    stable_hash_string(&format!(
        "{}:{:?}:{}:{}:{}:{}",
        value.fold_plan_digest,
        value.prediction_digests,
        value.prediction_count,
        value.target_accessed,
        value.label_derived,
        value.metrics_computed,
    ))
}

fn build_prediction_capsule(
    plan: &MomentumHistoricalFoldPlanV1,
    predictions: &[MomentumHistoricalFoldPredictionV1],
) -> Result<MomentumHistoricalFoldPredictionCapsuleV1, String> {
    let mut prediction_digests = predictions
        .iter()
        .map(|value| value.prediction_digest.clone())
        .collect::<Vec<_>>();
    prediction_digests.sort();
    let mut value = MomentumHistoricalFoldPredictionCapsuleV1 {
        fold_plan_digest: plan.fold_plan_digest.clone(),
        prediction_count: prediction_digests.len(),
        prediction_digests,
        target_accessed: false,
        label_derived: false,
        metrics_computed: false,
        capsule_digest: String::new(),
    };
    value.capsule_digest = capsule_digest(&value);
    validate_prediction_capsule(&value, predictions)?;
    Ok(value)
}

fn validate_prediction_capsule(
    value: &MomentumHistoricalFoldPredictionCapsuleV1,
    predictions: &[MomentumHistoricalFoldPredictionV1],
) -> Result<(), String> {
    validate_prediction_capsule_shape(value)?;
    let participant_ids = predictions
        .iter()
        .map(|prediction| prediction.participant_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut expected_digests = predictions
        .iter()
        .map(|prediction| prediction.prediction_digest.clone())
        .collect::<Vec<_>>();
    expected_digests.sort();
    if predictions.len() != 3
        || participant_ids
            != BTreeSet::from([
                RAW_PARTICIPANT,
                INTERACTION_PARTICIPANT,
                CONSTANT_PARTICIPANT,
            ])
        || predictions.iter().any(|prediction| {
            prediction.fold_plan_digest != value.fold_plan_digest
                || validate_prediction(prediction).is_err()
        })
        || value.prediction_digests != expected_digests
    {
        return Err("historical prediction capsule rejected".to_string());
    }
    Ok(())
}

fn validate_prediction_capsule_shape(
    value: &MomentumHistoricalFoldPredictionCapsuleV1,
) -> Result<(), String> {
    if value.fold_plan_digest.is_empty()
        || value.prediction_count != 3
        || value.prediction_digests.len() != 3
        || value
            .prediction_digests
            .iter()
            .any(|digest| digest.is_empty())
        || value.target_accessed
        || value.label_derived
        || value.metrics_computed
        || value.capsule_digest != capsule_digest(value)
    {
        return Err("historical prediction capsule shape rejected".to_string());
    }
    Ok(())
}

fn encode_prediction_capsule(
    value: &MomentumHistoricalFoldPredictionCapsuleV1,
    predictions: &[MomentumHistoricalFoldPredictionV1],
) -> Result<Vec<u8>, String> {
    validate_prediction_capsule(value, predictions)?;
    ArtifactBuilderV4_2::new("MomentumHistoricalFoldPredictionCapsuleV1")
        .string("fold_plan_digest", &value.fold_plan_digest)
        .strings("prediction_digests", &value.prediction_digests)
        .unsigned("prediction_count", as_u64(value.prediction_count)?)
        .boolean("target_accessed", value.target_accessed)
        .boolean("label_derived", value.label_derived)
        .boolean("metrics_computed", value.metrics_computed)
        .string("capsule_digest", &value.capsule_digest)
        .encode()
}

fn decode_prediction_capsule(
    bytes: &[u8],
    predictions: &[MomentumHistoricalFoldPredictionV1],
) -> Result<MomentumHistoricalFoldPredictionCapsuleV1, String> {
    let value = decode_prediction_capsule_unbound(bytes)?;
    validate_prediction_capsule(&value, predictions)?;
    Ok(value)
}

fn decode_prediction_capsule_unbound(
    bytes: &[u8],
) -> Result<MomentumHistoricalFoldPredictionCapsuleV1, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumHistoricalFoldPredictionCapsuleV1")?;
    let value = MomentumHistoricalFoldPredictionCapsuleV1 {
        fold_plan_digest: fields.string("fold_plan_digest")?,
        prediction_digests: fields.strings("prediction_digests")?,
        prediction_count: as_usize(fields.unsigned("prediction_count")?)?,
        target_accessed: fields.boolean("target_accessed")?,
        label_derived: fields.boolean("label_derived")?,
        metrics_computed: fields.boolean("metrics_computed")?,
        capsule_digest: fields.string("capsule_digest")?,
    };
    fields.finish()?;
    validate_prediction_capsule_shape(&value)?;
    Ok(value)
}

fn seed_for_fold(base_seed: u64, fold_number: u64) -> Result<u64, String> {
    fold_number
        .checked_mul(1_000_003)
        .and_then(|offset| base_seed.checked_add(offset))
        .ok_or_else(|| "historical fold seed overflow".to_string())
}

fn latest_raw_feature(
    rows_through_prediction_event: &[HistoricalOhlcvRow],
    feature_config: &MomentumFeatureConfigV0,
) -> Result<Vec<f32>, String> {
    let candle_rows = candles(rows_through_prediction_event)?;
    build_momentum_features_v0(&candle_rows, feature_config)
        .map_err(|_| "historical prediction feature construction rejected".to_string())?
        .last()
        .map(|row| row.values.clone())
        .ok_or_else(|| "historical prediction feature unavailable".to_string())
}

fn expand_normalized_training(
    values: &[EncodedTrainingExampleV0],
) -> Result<Vec<EncodedTrainingExampleV0>, String> {
    values
        .iter()
        .map(|value| {
            Ok(EncodedTrainingExampleV0 {
                representation: expand_interaction_representation_v4(&value.representation)?,
                label: value.label,
                snapshot_ids: value.snapshot_ids.clone(),
            })
        })
        .collect()
}

fn walk_forward_predictions(
    plan: &MomentumHistoricalFoldPlanV1,
    rows_through_prediction_event: &[HistoricalOhlcvRow],
    examples: &[super::SequenceExampleV0],
    registration: &MomentumHistoricalReplayRegistrationV1,
    campaign: &MomentumLearningCampaignConfigV0,
) -> Result<Vec<MomentumHistoricalFoldPredictionV1>, String> {
    if examples.len() < registration.minimum_training_examples
        || examples.iter().any(|example| {
            example.sequence_end >= plan.prediction_event_index
                || example.label_index > plan.prediction_event_index
        })
    {
        return Err("historical past-only training boundary rejected".to_string());
    }
    let raw_training = raw_encoded(examples)?;
    let normalizer = RepresentationNormalizerV0::fit(&raw_training)
        .map_err(|_| "historical fold-local normalizer rejected".to_string())?;
    let normalized_training = normalizer
        .transform(&raw_training)
        .map_err(|_| "historical fold-local normalization rejected".to_string())?;
    let current_raw = latest_raw_feature(rows_through_prediction_event, &campaign.feature_config)?;
    let normalized_current = normalizer
        .transform_representation(&current_raw)
        .map_err(|_| "historical prediction normalization rejected".to_string())?;
    let raw_head = LogisticPredictionHeadV0::seeded(
        normalized_current.len(),
        seed_for_fold(registration.initialization_seeds[0], plan.fold_number)?,
    )
    .map_err(|_| "historical raw head initialization rejected".to_string())?;
    let raw_head = train_head_v4(raw_head, &normalized_training, &campaign.training_config)?;
    let raw_probability = f64::from(
        raw_head
            .probability(&normalized_current)
            .map_err(|_| "historical raw prediction rejected".to_string())?,
    );

    let interaction_training = expand_normalized_training(&normalized_training)?;
    let interaction_current = expand_interaction_representation_v4(&normalized_current)?;
    if interaction_schema_digest(normalized_current.len()) != registration.interaction_schema_digest
    {
        return Err("historical interaction schema rejected".to_string());
    }
    let interaction_head = LogisticPredictionHeadV0::seeded(
        interaction_current.len(),
        seed_for_fold(registration.initialization_seeds[1], plan.fold_number)?,
    )
    .map_err(|_| "historical interaction head initialization rejected".to_string())?;
    let interaction_head = train_head_v4(
        interaction_head,
        &interaction_training,
        &campaign.training_config,
    )?;
    let interaction_probability = f64::from(
        interaction_head
            .probability(&interaction_current)
            .map_err(|_| "historical interaction prediction rejected".to_string())?,
    );

    let prevalence = raw_training
        .iter()
        .map(|example| f64::from(example.label))
        .sum::<f64>()
        / raw_training.len() as f64;
    if !prevalence.is_finite() || !(0.0..=1.0).contains(&prevalence) {
        return Err("historical prevalence benchmark rejected".to_string());
    }
    let normalizer_digest = format!("historical-research:{}", normalizer.digest());
    Ok(vec![
        build_prediction(
            plan,
            RAW_PARTICIPANT,
            "HistoricalResearchReplica",
            format!(
                "historical-research:{}",
                stable_hash_string(&format!(
                    "{}:{}:{}",
                    plan.fold_plan_digest,
                    RAW_PARTICIPANT,
                    raw_head.parameter_digest()
                ))
            ),
            normalizer_digest.clone(),
            stable_hash_string(&format!(
                "historical-private-feature:{}:{:?}",
                plan.fold_plan_digest,
                normalized_current
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            )),
            raw_probability,
        )?,
        build_prediction(
            plan,
            INTERACTION_PARTICIPANT,
            "HistoricalResearchReplica",
            format!(
                "historical-research:{}",
                stable_hash_string(&format!(
                    "{}:{}:{}",
                    plan.fold_plan_digest,
                    INTERACTION_PARTICIPANT,
                    interaction_head.parameter_digest()
                ))
            ),
            normalizer_digest,
            stable_hash_string(&format!(
                "historical-private-interaction-feature:{}:{:?}",
                plan.fold_plan_digest,
                interaction_current
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            )),
            interaction_probability,
        )?,
        build_prediction(
            plan,
            CONSTANT_PARTICIPANT,
            "HistoricalResearchReplica",
            format!(
                "historical-research:{}",
                stable_hash_string(&format!(
                    "{}:{}:{}",
                    plan.fold_plan_digest,
                    CONSTANT_PARTICIPANT,
                    prevalence.to_bits()
                ))
            ),
            "historical-research:past-labels-only".to_string(),
            stable_hash_string(&format!(
                "historical-private-prevalence:{}:{}",
                plan.fold_plan_digest,
                prevalence.to_bits()
            )),
            prevalence,
        )?,
    ])
}

fn protocol_predictions(
    plan: &MomentumHistoricalFoldPlanV1,
    examples: &[super::SequenceExampleV0],
) -> Result<Vec<MomentumHistoricalFoldPredictionV1>, String> {
    let prevalence = examples
        .iter()
        .map(|example| f64::from(example.label))
        .sum::<f64>()
        / examples.len() as f64;
    let fixtures = [
        (RAW_PARTICIPANT, 0.5),
        (INTERACTION_PARTICIPANT, prevalence),
        (CONSTANT_PARTICIPANT, prevalence),
    ];
    fixtures
        .into_iter()
        .map(|(participant, probability)| {
            build_prediction(
                plan,
                participant,
                "HistoricalResearchReplica",
                format!(
                    "historical-research:{}",
                    stable_hash_string(&format!(
                        "protocol-fixture:{}:{}",
                        plan.fold_plan_digest, participant
                    ))
                ),
                "historical-research:protocol-fixture".to_string(),
                stable_hash_string(&format!(
                    "historical-protocol-feature:{}:{}",
                    plan.fold_plan_digest, participant
                )),
                probability,
            )
        })
        .collect()
}

fn fold_evaluation_digest(value: &MomentumHistoricalFoldEvaluationV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}:{:?}:{:?}",
        value.fold_plan_digest,
        value.prediction_capsule_digest,
        value.label_status,
        value.private_label_bits,
        value.participant_evaluation_digests,
    ))
}

fn build_fold_evaluation(
    plan: &MomentumHistoricalFoldPlanV1,
    capsule: &MomentumHistoricalFoldPredictionCapsuleV1,
    predictions: &[MomentumHistoricalFoldPredictionV1],
    prediction_row: &HistoricalOhlcvRow,
    target_row: &HistoricalOhlcvRow,
    sequence_config: &MomentumSequenceConfigV0,
) -> Result<
    (
        MomentumHistoricalFoldEvaluationV1,
        Vec<FoldPrivateEvaluation>,
    ),
    String,
> {
    validate_prediction_capsule(capsule, predictions)?;
    if target_row.timestamp_ms != plan.target_timestamp_ms {
        return Err("historical registered target mismatch".to_string());
    }
    let (status, label) = classify_label(prediction_row, target_row, sequence_config)?;
    let private = label
        .map(|label| {
            predictions
                .iter()
                .map(|prediction| {
                    let probability = prediction.private_prediction as f32;
                    let binary_label = label as f32;
                    let metrics = evaluate_probabilities_v0(&[probability], &[binary_label])
                        .map_err(|_| "historical fold metric rejected".to_string())?;
                    let brier = f64::from(metrics.brier_score);
                    let correct = metrics.accuracy == 1.0;
                    Ok(FoldPrivateEvaluation {
                        participant_id: prediction.participant_id.clone(),
                        probability: prediction.private_prediction,
                        label,
                        brier,
                        correct,
                        evaluation_digest: stable_hash_string(&format!(
                            "historical-private-evaluation-v1:{}:{}:{}:{}:{}",
                            plan.fold_plan_digest,
                            prediction.participant_id,
                            prediction.private_prediction.to_bits(),
                            label.to_bits(),
                            brier.to_bits(),
                        )),
                    })
                })
                .collect::<Result<Vec<_>, String>>()
        })
        .transpose()?
        .unwrap_or_default();
    let mut participant_evaluation_digests = private
        .iter()
        .map(|value| value.evaluation_digest.clone())
        .collect::<Vec<_>>();
    participant_evaluation_digests.sort();
    let mut value = MomentumHistoricalFoldEvaluationV1 {
        fold_plan_digest: plan.fold_plan_digest.clone(),
        prediction_capsule_digest: capsule.capsule_digest.clone(),
        label_status: status,
        private_label_bits: label.map(f64::to_bits),
        participant_evaluation_digests,
        fold_evaluation_digest: String::new(),
    };
    value.fold_evaluation_digest = fold_evaluation_digest(&value);
    validate_fold_evaluation(&value, predictions)?;
    Ok((value, private))
}

fn validate_fold_evaluation(
    value: &MomentumHistoricalFoldEvaluationV1,
    predictions: &[MomentumHistoricalFoldPredictionV1],
) -> Result<(), String> {
    validate_fold_evaluation_shape(value)?;
    if predictions.len() != 3 {
        return Err("historical fold prediction set rejected".to_string());
    }
    Ok(())
}

fn validate_fold_evaluation_shape(
    value: &MomentumHistoricalFoldEvaluationV1,
) -> Result<(), String> {
    let expected_count = if value.label_status == HistoricalLabelStatusV1::ScorableBinaryOutcome {
        3
    } else {
        0
    };
    let label_valid = match value.label_status {
        HistoricalLabelStatusV1::ScorableBinaryOutcome => value
            .private_label_bits
            .map(f64::from_bits)
            .is_some_and(|label| label == 0.0 || label == 1.0),
        HistoricalLabelStatusV1::NeutralOutcomeExcluded => value.private_label_bits.is_none(),
        HistoricalLabelStatusV1::InvalidOutcomeEvidence => value.private_label_bits.is_none(),
    };
    if value.fold_plan_digest.is_empty()
        || value.prediction_capsule_digest.is_empty()
        || !label_valid
        || value.participant_evaluation_digests.len() != expected_count
        || value.fold_evaluation_digest != fold_evaluation_digest(value)
    {
        return Err("historical fold evaluation rejected".to_string());
    }
    Ok(())
}

fn encode_fold_evaluation(
    value: &MomentumHistoricalFoldEvaluationV1,
    predictions: &[MomentumHistoricalFoldPredictionV1],
) -> Result<Vec<u8>, String> {
    validate_fold_evaluation(value, predictions)?;
    ArtifactBuilderV4_2::new("MomentumHistoricalFoldEvaluationV1")
        .string("fold_plan_digest", &value.fold_plan_digest)
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .string("label_status", format!("{:?}", value.label_status))
        .optional_string(
            "private_label_bits",
            &value.private_label_bits.map(|bits| bits.to_string()),
        )
        .strings(
            "participant_evaluation_digests",
            &value.participant_evaluation_digests,
        )
        .string("fold_evaluation_digest", &value.fold_evaluation_digest)
        .encode()
}

fn decode_fold_evaluation(
    bytes: &[u8],
    predictions: &[MomentumHistoricalFoldPredictionV1],
) -> Result<MomentumHistoricalFoldEvaluationV1, String> {
    let value = decode_fold_evaluation_unbound(bytes)?;
    validate_fold_evaluation(&value, predictions)?;
    Ok(value)
}

fn decode_fold_evaluation_unbound(
    bytes: &[u8],
) -> Result<MomentumHistoricalFoldEvaluationV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumHistoricalFoldEvaluationV1")?;
    let private_label_bits = fields
        .optional_string("private_label_bits")?
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| "historical private label bits rejected".to_string())
        })
        .transpose()?;
    let value = MomentumHistoricalFoldEvaluationV1 {
        fold_plan_digest: fields.string("fold_plan_digest")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        label_status: label_status(&fields.string("label_status")?)?,
        private_label_bits,
        participant_evaluation_digests: fields.strings("participant_evaluation_digests")?,
        fold_evaluation_digest: fields.string("fold_evaluation_digest")?,
    };
    fields.finish()?;
    validate_fold_evaluation_shape(&value)?;
    Ok(value)
}

fn participant_aggregate_digest(value: &MomentumHistoricalParticipantAggregateV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{:?}:{}",
        value.participant_id,
        value.scorable_fold_count,
        value.mean_brier_score.to_bits(),
        value.binary_correctness_rate.to_bits(),
        value.benchmark_relative_brier_delta.to_bits(),
        value.comparison_status,
        value.research_only,
    ))
}

fn aggregate_digest(value: &MomentumHistoricalAggregateReportV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{:?}:{:?}:{}:{}:{}:{}:{}:{:?}:{:?}:{:?}:{}:{}",
        value.aggregate_version,
        value.dataset_snapshot_digest,
        value.contamination_audit_digest,
        value.registration_digest,
        value.replay_mode,
        value.eligible_fold_count,
        value.completed_fold_count,
        value.scorable_fold_count,
        value.neutral_fold_count,
        value.invalid_fold_count,
        value.learned_vs_constant_paired_scorable_count,
        value.participants,
        value.comparison_status,
        value.finite_metric_proof,
        value.chronology_audit_passed,
        value.leakage_audit_passed,
        value.prediction_before_reveal_audit_passed,
        value.winner_selected,
        value.evidence_labels,
        value.trading_simulation_status,
        value.safety_counters,
        value.protected_artifacts_unchanged,
        value.active_roster_unchanged,
    ))
}

fn zero_authority_valid(value: &MomentumHistoricalSafetyCountersV1) -> bool {
    value.network_requests == 0
        && value.transport_constructions == 0
        && value.credential_reads == 0
        && value.live_prospective_event_count_changes == 0
        && value.live_prospective_scorable_count_changes == 0
        && value.live_participant_changes == 0
        && value.live_parameter_updates == 0
        && value.live_normalizer_refits == 0
        && value.winner_selections == 0
        && value.rankings == 0
        && value.reward_applications == 0
        && value.penalty_applications == 0
        && value.chair_decisions == 0
        && value.committee_votes == 0
        && value.voice_changes == 0
        && value.tier_changes == 0
        && value.cooldowns == 0
        && value.promotions == 0
        && value.quarantines == 0
        && value.paper_executions == 0
        && value.live_executions == 0
        && value.active_committee_count == 3
}

fn validate_participant_aggregate(
    value: &MomentumHistoricalParticipantAggregateV1,
) -> Result<(), String> {
    if ![
        RAW_PARTICIPANT,
        INTERACTION_PARTICIPANT,
        CONSTANT_PARTICIPANT,
    ]
    .contains(&value.participant_id.as_str())
        || value.scorable_fold_count == 0
        || !value.mean_brier_score.is_finite()
        || !(0.0..=1.0).contains(&value.mean_brier_score)
        || !value.binary_correctness_rate.is_finite()
        || !(0.0..=1.0).contains(&value.binary_correctness_rate)
        || !value.benchmark_relative_brier_delta.is_finite()
        || !value.research_only
        || value.aggregate_digest != participant_aggregate_digest(value)
    {
        return Err("historical participant aggregate rejected".to_string());
    }
    Ok(())
}

fn validate_aggregate(value: &MomentumHistoricalAggregateReportV1) -> Result<(), String> {
    let expected_participants = match value.replay_mode {
        MomentumHistoricalReplayModeV1::ProtocolReplay => 0,
        MomentumHistoricalReplayModeV1::ExpandingWindowWalkForward => {
            if value.scorable_fold_count == 0 { 0 } else { 3 }
        }
    };
    if value.aggregate_version != AGGREGATE_VERSION
        || value.dataset_snapshot_digest.is_empty()
        || value.contamination_audit_digest.is_empty()
        || value.registration_digest.is_empty()
        || value.completed_fold_count != value.eligible_fold_count
        || value.scorable_fold_count + value.neutral_fold_count + value.invalid_fold_count
            != value.completed_fold_count
        || value.participants.len() != expected_participants
        || value
            .participants
            .iter()
            .any(|participant| validate_participant_aggregate(participant).is_err())
        || !value.finite_metric_proof
        || !value.chronology_audit_passed
        || !value.leakage_audit_passed
        || !value.prediction_before_reveal_audit_passed
        || value.winner_selected
        || value.evidence_labels != RESEARCH_LABELS.map(str::to_string)
        || value.trading_simulation_status
            != MomentumHistoricalTradingSimulationStatusV1::BlockedNoFrozenExecutionPolicy
        || !zero_authority_valid(&value.safety_counters)
        || !value.protected_artifacts_unchanged
        || !value.active_roster_unchanged
        || value.aggregate_digest != aggregate_digest(value)
    {
        return Err("historical aggregate report rejected".to_string());
    }
    Ok(())
}

fn comparison_for_delta(delta: f64, scorable_count: usize) -> MomentumHistoricalComparisonStatusV1 {
    if scorable_count < 8 {
        MomentumHistoricalComparisonStatusV1::InsufficientScorableFolds
    } else if delta < -f64::EPSILON {
        MomentumHistoricalComparisonStatusV1::LearnedBetterOnResearchReplay
    } else if delta > f64::EPSILON {
        MomentumHistoricalComparisonStatusV1::BenchmarkBetterOnResearchReplay
    } else {
        MomentumHistoricalComparisonStatusV1::MixedResearchEvidence
    }
}

fn build_aggregate(
    snapshot: &MomentumHistoricalDatasetSnapshotV1,
    audit: &MomentumHistoricalContaminationAuditV1,
    registration: &MomentumHistoricalReplayRegistrationV1,
    eligible_fold_count: usize,
    completed_fold_count: usize,
    scorable_fold_count: usize,
    neutral_fold_count: usize,
    invalid_fold_count: usize,
    accumulators: &BTreeMap<String, ParticipantAccumulator>,
    protected_artifacts_unchanged: bool,
    active_roster_unchanged: bool,
) -> Result<MomentumHistoricalAggregateReportV1, String> {
    let mut participants = Vec::new();
    let mut overall_comparison = if invalid_fold_count > 0 {
        MomentumHistoricalComparisonStatusV1::IntegrityFailure
    } else {
        MomentumHistoricalComparisonStatusV1::InsufficientScorableFolds
    };
    if registration.replay_mode == MomentumHistoricalReplayModeV1::ExpandingWindowWalkForward
        && scorable_fold_count > 0
    {
        let constant = accumulators
            .get(CONSTANT_PARTICIPANT)
            .ok_or_else(|| "historical constant aggregate unavailable".to_string())?;
        if constant.count != scorable_fold_count {
            return Err("historical paired aggregate rejected".to_string());
        }
        let constant_mean = constant.brier_sum / constant.count as f64;
        for participant_id in [
            RAW_PARTICIPANT,
            INTERACTION_PARTICIPANT,
            CONSTANT_PARTICIPANT,
        ] {
            let accumulator = accumulators
                .get(participant_id)
                .ok_or_else(|| "historical participant aggregate unavailable".to_string())?;
            if accumulator.count != scorable_fold_count {
                return Err("historical participant pairing rejected".to_string());
            }
            let mean_brier_score = accumulator.brier_sum / accumulator.count as f64;
            let delta = mean_brier_score - constant_mean;
            let mut participant = MomentumHistoricalParticipantAggregateV1 {
                participant_id: participant_id.to_string(),
                scorable_fold_count: accumulator.count,
                mean_brier_score,
                binary_correctness_rate: accumulator.correct_count as f64
                    / accumulator.count as f64,
                benchmark_relative_brier_delta: delta,
                comparison_status: if participant_id == CONSTANT_PARTICIPANT {
                    MomentumHistoricalComparisonStatusV1::MixedResearchEvidence
                } else {
                    comparison_for_delta(delta, accumulator.count)
                },
                research_only: true,
                aggregate_digest: String::new(),
            };
            participant.aggregate_digest = participant_aggregate_digest(&participant);
            participants.push(participant);
        }
        if invalid_fold_count == 0 {
            let learned = participants
                .iter()
                .filter(|participant| participant.participant_id != CONSTANT_PARTICIPANT)
                .map(|participant| participant.comparison_status)
                .collect::<Vec<_>>();
            overall_comparison = if learned.iter().all(|status| {
                *status == MomentumHistoricalComparisonStatusV1::LearnedBetterOnResearchReplay
            }) {
                MomentumHistoricalComparisonStatusV1::LearnedBetterOnResearchReplay
            } else if learned.iter().all(|status| {
                *status == MomentumHistoricalComparisonStatusV1::BenchmarkBetterOnResearchReplay
            }) {
                MomentumHistoricalComparisonStatusV1::BenchmarkBetterOnResearchReplay
            } else if learned.iter().all(|status| {
                *status == MomentumHistoricalComparisonStatusV1::InsufficientScorableFolds
            }) {
                MomentumHistoricalComparisonStatusV1::InsufficientScorableFolds
            } else {
                MomentumHistoricalComparisonStatusV1::MixedResearchEvidence
            };
        }
    }
    let finite_metric_proof = participants.iter().all(|participant| {
        participant.mean_brier_score.is_finite()
            && participant.binary_correctness_rate.is_finite()
            && participant.benchmark_relative_brier_delta.is_finite()
    });
    let mut value = MomentumHistoricalAggregateReportV1 {
        aggregate_version: AGGREGATE_VERSION.to_string(),
        dataset_snapshot_digest: snapshot.snapshot_digest.clone(),
        contamination_audit_digest: audit.audit_digest.clone(),
        registration_digest: registration.registration_digest.clone(),
        replay_mode: registration.replay_mode,
        eligible_fold_count,
        completed_fold_count,
        scorable_fold_count,
        neutral_fold_count,
        invalid_fold_count,
        learned_vs_constant_paired_scorable_count: if registration.replay_mode
            == MomentumHistoricalReplayModeV1::ExpandingWindowWalkForward
        {
            scorable_fold_count
        } else {
            0
        },
        participants,
        comparison_status: overall_comparison,
        finite_metric_proof,
        chronology_audit_passed: true,
        leakage_audit_passed: true,
        prediction_before_reveal_audit_passed: true,
        winner_selected: false,
        evidence_labels: RESEARCH_LABELS.map(str::to_string).to_vec(),
        trading_simulation_status:
            MomentumHistoricalTradingSimulationStatusV1::BlockedNoFrozenExecutionPolicy,
        safety_counters: MomentumHistoricalSafetyCountersV1::zero_authority(),
        protected_artifacts_unchanged,
        active_roster_unchanged,
        aggregate_digest: String::new(),
    };
    value.aggregate_digest = aggregate_digest(&value);
    validate_aggregate(&value)?;
    Ok(value)
}

fn backfill_plan_digest(value: &MomentumHistoricalBackfillPlanV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{:?}:{}:{}:{}:{}:{}:{}",
        value.plan_version,
        value.provider_id,
        value.market,
        value.symbol,
        value.cadence,
        value.existing_first_timestamp_ms,
        value.existing_last_timestamp_ms,
        value.desired_direction,
        value.request_count_upper_bound,
        value.request_limit_known,
        value.maximum_concurrency,
        value.maximum_retries,
        value.explicit_network_authorization_required,
        value.executed,
    ))
}

fn build_backfill_plan(
    snapshot: &MomentumHistoricalDatasetSnapshotV1,
) -> MomentumHistoricalBackfillPlanV1 {
    let mut value = MomentumHistoricalBackfillPlanV1 {
        plan_version: BACKFILL_VERSION.to_string(),
        provider_id: snapshot.provider_id.clone(),
        market: snapshot.market.clone(),
        symbol: snapshot.symbol.clone(),
        cadence: snapshot.cadence.clone(),
        existing_first_timestamp_ms: snapshot.first_timestamp_ms,
        existing_last_timestamp_ms: snapshot.last_timestamp_ms,
        desired_direction: HistoricalBackfillDirectionV1::OlderThanExistingSnapshot,
        request_count_upper_bound: 0,
        request_limit_known: false,
        maximum_concurrency: 1,
        maximum_retries: 0,
        explicit_network_authorization_required: true,
        executed: false,
        backfill_plan_digest: String::new(),
    };
    value.backfill_plan_digest = backfill_plan_digest(&value);
    value
}

fn validate_backfill_plan(value: &MomentumHistoricalBackfillPlanV1) -> Result<(), String> {
    if value.plan_version != BACKFILL_VERSION
        || value.provider_id.is_empty()
        || value.symbol.is_empty()
        || value.cadence != "1d"
        || value.existing_first_timestamp_ms >= value.existing_last_timestamp_ms
        || value.desired_direction != HistoricalBackfillDirectionV1::OlderThanExistingSnapshot
        || value.request_count_upper_bound != 0
        || value.request_limit_known
        || value.maximum_concurrency != 1
        || value.maximum_retries != 0
        || !value.explicit_network_authorization_required
        || value.executed
        || value.backfill_plan_digest != backfill_plan_digest(value)
    {
        return Err("historical backfill plan rejected".to_string());
    }
    Ok(())
}

fn encode_participant_aggregate(
    value: &MomentumHistoricalParticipantAggregateV1,
) -> Result<Vec<u8>, String> {
    validate_participant_aggregate(value)?;
    ArtifactBuilderV4_2::new("MomentumHistoricalParticipantAggregateV1")
        .string("participant_id", &value.participant_id)
        .unsigned("scorable_fold_count", as_u64(value.scorable_fold_count)?)
        .unsigned("mean_brier_score_bits", value.mean_brier_score.to_bits())
        .unsigned(
            "binary_correctness_rate_bits",
            value.binary_correctness_rate.to_bits(),
        )
        .unsigned(
            "benchmark_relative_brier_delta_bits",
            value.benchmark_relative_brier_delta.to_bits(),
        )
        .string(
            "comparison_status",
            format!("{:?}", value.comparison_status),
        )
        .boolean("research_only", value.research_only)
        .string("aggregate_digest", &value.aggregate_digest)
        .encode()
}

fn decode_participant_aggregate(
    bytes: &[u8],
) -> Result<MomentumHistoricalParticipantAggregateV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumHistoricalParticipantAggregateV1")?;
    let value = MomentumHistoricalParticipantAggregateV1 {
        participant_id: fields.string("participant_id")?,
        scorable_fold_count: as_usize(fields.unsigned("scorable_fold_count")?)?,
        mean_brier_score: f64::from_bits(fields.unsigned("mean_brier_score_bits")?),
        binary_correctness_rate: f64::from_bits(fields.unsigned("binary_correctness_rate_bits")?),
        benchmark_relative_brier_delta: f64::from_bits(
            fields.unsigned("benchmark_relative_brier_delta_bits")?,
        ),
        comparison_status: comparison_status(&fields.string("comparison_status")?)?,
        research_only: fields.boolean("research_only")?,
        aggregate_digest: fields.string("aggregate_digest")?,
    };
    fields.finish()?;
    validate_participant_aggregate(&value)?;
    Ok(value)
}

fn safety_values(value: &MomentumHistoricalSafetyCountersV1) -> Result<Vec<u64>, String> {
    [
        value.network_requests,
        value.transport_constructions,
        value.credential_reads,
        value.live_prospective_event_count_changes,
        value.live_prospective_scorable_count_changes,
        value.live_participant_changes,
        value.live_parameter_updates,
        value.live_normalizer_refits,
        value.winner_selections,
        value.rankings,
        value.reward_applications,
        value.penalty_applications,
        value.chair_decisions,
        value.committee_votes,
        value.voice_changes,
        value.tier_changes,
        value.cooldowns,
        value.promotions,
        value.quarantines,
        value.paper_executions,
        value.live_executions,
        value.active_committee_count,
    ]
    .into_iter()
    .map(as_u64)
    .collect()
}

fn safety_from_values(values: Vec<u64>) -> Result<MomentumHistoricalSafetyCountersV1, String> {
    if values.len() != 22 {
        return Err("historical safety counters rejected".to_string());
    }
    let values = values
        .into_iter()
        .map(as_usize)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MomentumHistoricalSafetyCountersV1 {
        network_requests: values[0],
        transport_constructions: values[1],
        credential_reads: values[2],
        live_prospective_event_count_changes: values[3],
        live_prospective_scorable_count_changes: values[4],
        live_participant_changes: values[5],
        live_parameter_updates: values[6],
        live_normalizer_refits: values[7],
        winner_selections: values[8],
        rankings: values[9],
        reward_applications: values[10],
        penalty_applications: values[11],
        chair_decisions: values[12],
        committee_votes: values[13],
        voice_changes: values[14],
        tier_changes: values[15],
        cooldowns: values[16],
        promotions: values[17],
        quarantines: values[18],
        paper_executions: values[19],
        live_executions: values[20],
        active_committee_count: values[21],
    })
}

fn encode_aggregate(value: &MomentumHistoricalAggregateReportV1) -> Result<Vec<u8>, String> {
    validate_aggregate(value)?;
    ArtifactBuilderV4_2::new("MomentumHistoricalAggregateReportV1")
        .string("aggregate_version", &value.aggregate_version)
        .string("dataset_snapshot_digest", &value.dataset_snapshot_digest)
        .string(
            "contamination_audit_digest",
            &value.contamination_audit_digest,
        )
        .string("registration_digest", &value.registration_digest)
        .string("replay_mode", format!("{:?}", value.replay_mode))
        .unsigned("eligible_fold_count", as_u64(value.eligible_fold_count)?)
        .unsigned("completed_fold_count", as_u64(value.completed_fold_count)?)
        .unsigned("scorable_fold_count", as_u64(value.scorable_fold_count)?)
        .unsigned("neutral_fold_count", as_u64(value.neutral_fold_count)?)
        .unsigned("invalid_fold_count", as_u64(value.invalid_fold_count)?)
        .unsigned(
            "learned_vs_constant_paired_scorable_count",
            as_u64(value.learned_vs_constant_paired_scorable_count)?,
        )
        .messages(
            "participants",
            value
                .participants
                .iter()
                .map(encode_participant_aggregate)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .string(
            "comparison_status",
            format!("{:?}", value.comparison_status),
        )
        .boolean("finite_metric_proof", value.finite_metric_proof)
        .boolean("chronology_audit_passed", value.chronology_audit_passed)
        .boolean("leakage_audit_passed", value.leakage_audit_passed)
        .boolean(
            "prediction_before_reveal_audit_passed",
            value.prediction_before_reveal_audit_passed,
        )
        .boolean("winner_selected", value.winner_selected)
        .strings("evidence_labels", &value.evidence_labels)
        .string(
            "trading_simulation_status",
            format!("{:?}", value.trading_simulation_status),
        )
        .unsigneds("safety_counters", &safety_values(&value.safety_counters)?)
        .boolean(
            "protected_artifacts_unchanged",
            value.protected_artifacts_unchanged,
        )
        .boolean("active_roster_unchanged", value.active_roster_unchanged)
        .string("aggregate_digest", &value.aggregate_digest)
        .encode()
}

fn decode_aggregate(bytes: &[u8]) -> Result<MomentumHistoricalAggregateReportV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumHistoricalAggregateReportV1")?;
    let trading = fields.string("trading_simulation_status")?;
    if trading != "BlockedNoFrozenExecutionPolicy" {
        return Err("historical trading status rejected".to_string());
    }
    let value = MomentumHistoricalAggregateReportV1 {
        aggregate_version: fields.string("aggregate_version")?,
        dataset_snapshot_digest: fields.string("dataset_snapshot_digest")?,
        contamination_audit_digest: fields.string("contamination_audit_digest")?,
        registration_digest: fields.string("registration_digest")?,
        replay_mode: replay_mode(&fields.string("replay_mode")?)?,
        eligible_fold_count: as_usize(fields.unsigned("eligible_fold_count")?)?,
        completed_fold_count: as_usize(fields.unsigned("completed_fold_count")?)?,
        scorable_fold_count: as_usize(fields.unsigned("scorable_fold_count")?)?,
        neutral_fold_count: as_usize(fields.unsigned("neutral_fold_count")?)?,
        invalid_fold_count: as_usize(fields.unsigned("invalid_fold_count")?)?,
        learned_vs_constant_paired_scorable_count: as_usize(
            fields.unsigned("learned_vs_constant_paired_scorable_count")?,
        )?,
        participants: fields
            .messages("participants")?
            .iter()
            .map(|bytes| decode_participant_aggregate(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        comparison_status: comparison_status(&fields.string("comparison_status")?)?,
        finite_metric_proof: fields.boolean("finite_metric_proof")?,
        chronology_audit_passed: fields.boolean("chronology_audit_passed")?,
        leakage_audit_passed: fields.boolean("leakage_audit_passed")?,
        prediction_before_reveal_audit_passed: fields
            .boolean("prediction_before_reveal_audit_passed")?,
        winner_selected: fields.boolean("winner_selected")?,
        evidence_labels: fields.strings("evidence_labels")?,
        trading_simulation_status:
            MomentumHistoricalTradingSimulationStatusV1::BlockedNoFrozenExecutionPolicy,
        safety_counters: safety_from_values(fields.unsigneds("safety_counters")?)?,
        protected_artifacts_unchanged: fields.boolean("protected_artifacts_unchanged")?,
        active_roster_unchanged: fields.boolean("active_roster_unchanged")?,
        aggregate_digest: fields.string("aggregate_digest")?,
    };
    fields.finish()?;
    validate_aggregate(&value)?;
    Ok(value)
}

fn journal_digest(value: &MomentumHistoricalReplayJournalV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{:?}:{:?}:{:?}:{}",
        value.journal_version,
        value.registration_digest,
        value.aggregate_digest,
        value.fold_plan_digests,
        value.prediction_capsule_digests,
        value.fold_evaluation_digests,
        value.completed,
    ))
}

fn validate_journal(value: &MomentumHistoricalReplayJournalV1) -> Result<(), String> {
    if value.journal_version != JOURNAL_VERSION
        || value.registration_digest.is_empty()
        || value.aggregate_digest.is_empty()
        || value.fold_plan_digests.is_empty()
        || value.fold_plan_digests.len() != value.prediction_capsule_digests.len()
        || value.fold_plan_digests.len() != value.fold_evaluation_digests.len()
        || !value.completed
        || value.journal_digest != journal_digest(value)
    {
        return Err("historical replay journal rejected".to_string());
    }
    Ok(())
}

fn encode_journal(value: &MomentumHistoricalReplayJournalV1) -> Result<Vec<u8>, String> {
    validate_journal(value)?;
    ArtifactBuilderV4_2::new("MomentumHistoricalReplayJournalV1")
        .string("journal_version", &value.journal_version)
        .string("registration_digest", &value.registration_digest)
        .string("aggregate_digest", &value.aggregate_digest)
        .strings("fold_plan_digests", &value.fold_plan_digests)
        .strings(
            "prediction_capsule_digests",
            &value.prediction_capsule_digests,
        )
        .strings("fold_evaluation_digests", &value.fold_evaluation_digests)
        .boolean("completed", value.completed)
        .string("journal_digest", &value.journal_digest)
        .encode()
}

fn decode_journal(bytes: &[u8]) -> Result<MomentumHistoricalReplayJournalV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumHistoricalReplayJournalV1")?;
    let value = MomentumHistoricalReplayJournalV1 {
        journal_version: fields.string("journal_version")?,
        registration_digest: fields.string("registration_digest")?,
        aggregate_digest: fields.string("aggregate_digest")?,
        fold_plan_digests: fields.strings("fold_plan_digests")?,
        prediction_capsule_digests: fields.strings("prediction_capsule_digests")?,
        fold_evaluation_digests: fields.strings("fold_evaluation_digests")?,
        completed: fields.boolean("completed")?,
        journal_digest: fields.string("journal_digest")?,
    };
    fields.finish()?;
    validate_journal(&value)?;
    Ok(value)
}

fn encode_backfill_plan(value: &MomentumHistoricalBackfillPlanV1) -> Result<Vec<u8>, String> {
    validate_backfill_plan(value)?;
    ArtifactBuilderV4_2::new("MomentumHistoricalBackfillPlanV1")
        .string("plan_version", &value.plan_version)
        .string("provider_id", &value.provider_id)
        .string("market", &value.market)
        .string("symbol", &value.symbol)
        .string("cadence", &value.cadence)
        .unsigned(
            "existing_first_timestamp_ms",
            value.existing_first_timestamp_ms,
        )
        .unsigned(
            "existing_last_timestamp_ms",
            value.existing_last_timestamp_ms,
        )
        .string(
            "desired_direction",
            format!("{:?}", value.desired_direction),
        )
        .unsigned(
            "request_count_upper_bound",
            as_u64(value.request_count_upper_bound)?,
        )
        .boolean("request_limit_known", value.request_limit_known)
        .unsigned("maximum_concurrency", as_u64(value.maximum_concurrency)?)
        .unsigned("maximum_retries", as_u64(value.maximum_retries)?)
        .boolean(
            "explicit_network_authorization_required",
            value.explicit_network_authorization_required,
        )
        .boolean("executed", value.executed)
        .string("backfill_plan_digest", &value.backfill_plan_digest)
        .encode()
}

fn decode_backfill_plan(bytes: &[u8]) -> Result<MomentumHistoricalBackfillPlanV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumHistoricalBackfillPlanV1")?;
    let direction = fields.string("desired_direction")?;
    if direction != "OlderThanExistingSnapshot" {
        return Err("historical backfill direction rejected".to_string());
    }
    let value = MomentumHistoricalBackfillPlanV1 {
        plan_version: fields.string("plan_version")?,
        provider_id: fields.string("provider_id")?,
        market: fields.string("market")?,
        symbol: fields.string("symbol")?,
        cadence: fields.string("cadence")?,
        existing_first_timestamp_ms: fields.unsigned("existing_first_timestamp_ms")?,
        existing_last_timestamp_ms: fields.unsigned("existing_last_timestamp_ms")?,
        desired_direction: HistoricalBackfillDirectionV1::OlderThanExistingSnapshot,
        request_count_upper_bound: as_usize(fields.unsigned("request_count_upper_bound")?)?,
        request_limit_known: fields.boolean("request_limit_known")?,
        maximum_concurrency: as_usize(fields.unsigned("maximum_concurrency")?)?,
        maximum_retries: as_usize(fields.unsigned("maximum_retries")?)?,
        explicit_network_authorization_required: fields
            .boolean("explicit_network_authorization_required")?,
        executed: fields.boolean("executed")?,
        backfill_plan_digest: fields.string("backfill_plan_digest")?,
    };
    fields.finish()?;
    validate_backfill_plan(&value)?;
    Ok(value)
}

fn add_write_counts(total: &mut (usize, usize), next: (usize, usize)) {
    total.0 += next.0;
    total.1 += next.1;
}

fn collect_tree(
    current: &Path,
    base: &Path,
    values: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), String> {
    if !current.exists() {
        return Ok(());
    }
    if current.is_dir() {
        let mut paths = fs::read_dir(current)
            .map_err(|_| "historical protected directory read failed".to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            collect_tree(&path, base, values)?;
        }
    } else if current.is_file() {
        values.push((
            current
                .strip_prefix(base)
                .map_err(|_| "historical protected path rejected".to_string())?
                .to_path_buf(),
            fs::read(current)
                .map_err(|_| "historical protected artifact read failed".to_string())?,
        ));
    }
    Ok(())
}

fn protected_tree_digest(root: &Path) -> Result<String, String> {
    let mut values = Vec::new();
    collect_tree(root, root, &mut values)?;
    values.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(stable_hash_string(&format!(
        "momentum-historical-live-protected-v1:{values:?}"
    )))
}

fn active_roster_digest() -> String {
    stable_hash_string(&format!(
        "momentum-historical-active-roster-v1:{:?}",
        canonical_current_agent_states()
    ))
}

fn public_report(
    run_mode: MomentumHistoricalRunModeV1,
    snapshot: &MomentumHistoricalDatasetSnapshotV1,
    audit: &MomentumHistoricalContaminationAuditV1,
    registration: &MomentumHistoricalReplayRegistrationV1,
    aggregate: Option<&MomentumHistoricalAggregateReportV1>,
    backfill_plan: MomentumHistoricalBackfillPlanV1,
    existing_completed_replay: bool,
    writes: (usize, usize),
    runtime_duration_ms: u64,
    protected_artifacts_unchanged: bool,
    active_roster_unchanged: bool,
    replay_digest: String,
    eligible_fold_count: usize,
) -> MomentumHistoricalPublicReportV1 {
    let safety = aggregate
        .map(|value| value.safety_counters.clone())
        .unwrap_or_else(MomentumHistoricalSafetyCountersV1::zero_authority);
    MomentumHistoricalPublicReportV1 {
        report_version: PUBLIC_REPORT_VERSION.to_string(),
        run_mode: run_mode.as_str().to_string(),
        replay_mode: registration.replay_mode,
        offline: true,
        evidence_use_class: snapshot.evidence_use_class,
        dataset_snapshot_digest: snapshot.snapshot_digest.clone(),
        contamination_audit_digest: audit.audit_digest.clone(),
        registration_digest: registration.registration_digest.clone(),
        row_count: snapshot.row_count,
        first_timestamp_ms: snapshot.first_timestamp_ms,
        last_timestamp_ms: snapshot.last_timestamp_ms,
        eligible_fold_count: aggregate
            .map(|value| value.eligible_fold_count)
            .unwrap_or(eligible_fold_count),
        completed_fold_count: aggregate.map_or(0, |value| value.completed_fold_count),
        scorable_fold_count: aggregate.map_or(0, |value| value.scorable_fold_count),
        neutral_fold_count: aggregate.map_or(0, |value| value.neutral_fold_count),
        invalid_fold_count: aggregate.map_or(0, |value| value.invalid_fold_count),
        participants: aggregate
            .map(|value| value.participants.clone())
            .unwrap_or_default(),
        comparison_status: aggregate.map_or(
            MomentumHistoricalComparisonStatusV1::InsufficientScorableFolds,
            |value| value.comparison_status,
        ),
        chronology_audit_passed: aggregate.is_none_or(|value| value.chronology_audit_passed),
        leakage_audit_passed: aggregate.is_none_or(|value| value.leakage_audit_passed),
        prediction_before_reveal_audit_passed: aggregate
            .is_none_or(|value| value.prediction_before_reveal_audit_passed),
        replay_deterministic: true,
        existing_completed_replay,
        artifacts_written: writes.0,
        duplicate_artifact_count: writes.1,
        runtime_duration_ms,
        evidence_labels: RESEARCH_LABELS.map(str::to_string).to_vec(),
        trading_simulation_status:
            MomentumHistoricalTradingSimulationStatusV1::BlockedNoFrozenExecutionPolicy,
        backfill_plan,
        safety_counters: safety,
        protected_artifacts_unchanged,
        active_roster_unchanged,
        replay_digest,
    }
}

fn persist_dataset_snapshot(
    root: &Path,
    value: &MomentumHistoricalDatasetSnapshotV1,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("dataset_snapshots")
            .join(format!("{}.pb", value.snapshot_digest)),
        &encode_dataset_snapshot(value)?,
        &value.snapshot_digest,
        |bytes| Ok(decode_dataset_snapshot(bytes)?.snapshot_digest),
    )
}

fn persist_contamination_audit(
    root: &Path,
    value: &MomentumHistoricalContaminationAuditV1,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("contamination_audits")
            .join(format!("{}.pb", value.audit_digest)),
        &encode_contamination_audit(value)?,
        &value.audit_digest,
        |bytes| Ok(decode_contamination_audit(bytes)?.audit_digest),
    )
}

fn persist_registration(
    root: &Path,
    value: &MomentumHistoricalReplayRegistrationV1,
    row_count: usize,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("registrations")
            .join(format!("{}.pb", value.registration_digest)),
        &encode_registration(value, row_count)?,
        &value.registration_digest,
        |bytes| Ok(decode_registration(bytes, row_count)?.registration_digest),
    )
}

fn persist_fold_plan(
    root: &Path,
    value: &MomentumHistoricalFoldPlanV1,
    registration: &MomentumHistoricalReplayRegistrationV1,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("fold_plans")
            .join(format!("{}.pb", value.fold_plan_digest)),
        &encode_fold_plan(value)?,
        &value.fold_plan_digest,
        |bytes| Ok(decode_fold_plan(bytes, registration)?.fold_plan_digest),
    )
}

fn persist_prediction(
    root: &Path,
    value: &MomentumHistoricalFoldPredictionV1,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("prediction_seals")
            .join(format!("{}.pb", value.prediction_digest_identity)),
        &encode_prediction(value)?,
        &value.prediction_digest_identity,
        |bytes| Ok(decode_prediction(bytes)?.prediction_digest_identity),
    )
}

fn persist_prediction_capsule(
    root: &Path,
    value: &MomentumHistoricalFoldPredictionCapsuleV1,
    predictions: &[MomentumHistoricalFoldPredictionV1],
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("prediction_capsules")
            .join(format!("{}.pb", value.capsule_digest)),
        &encode_prediction_capsule(value, predictions)?,
        &value.capsule_digest,
        |bytes| Ok(decode_prediction_capsule(bytes, predictions)?.capsule_digest),
    )
}

fn persist_fold_evaluation(
    root: &Path,
    value: &MomentumHistoricalFoldEvaluationV1,
    predictions: &[MomentumHistoricalFoldPredictionV1],
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("fold_evaluations")
            .join(format!("{}.pb", value.fold_evaluation_digest)),
        &encode_fold_evaluation(value, predictions)?,
        &value.fold_evaluation_digest,
        |bytes| Ok(decode_fold_evaluation(bytes, predictions)?.fold_evaluation_digest),
    )
}

fn persist_backfill_plan(
    root: &Path,
    value: &MomentumHistoricalBackfillPlanV1,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root
            .join("backfill_plans")
            .join(format!("{}.pb", value.backfill_plan_digest)),
        &encode_backfill_plan(value)?,
        &value.backfill_plan_digest,
        |bytes| Ok(decode_backfill_plan(bytes)?.backfill_plan_digest),
    )
}

fn completed_paths(root: &Path, registration_digest: &str) -> (PathBuf, PathBuf) {
    let directory = root.join("completed").join(registration_digest);
    (directory.join("aggregate.pb"), directory.join("journal.pb"))
}

fn verify_completed_fold_artifacts(
    root: &Path,
    registration: &MomentumHistoricalReplayRegistrationV1,
    aggregate: &MomentumHistoricalAggregateReportV1,
    journal: &MomentumHistoricalReplayJournalV1,
) -> Result<(), String> {
    if journal.fold_plan_digests.len() != aggregate.completed_fold_count {
        return Err("historical completed fold count conflict rejected".to_string());
    }
    let prediction_directory = root.join("prediction_seals");
    let mut prediction_paths = fs::read_dir(&prediction_directory)
        .map_err(|_| "historical prediction seal directory unavailable".to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "pb"))
        .collect::<Vec<_>>();
    prediction_paths.sort();
    let mut predictions_by_digest = BTreeMap::new();
    for path in prediction_paths {
        let prediction = decode_prediction(
            &fs::read(path).map_err(|_| "historical prediction seal reopen failed".to_string())?,
        )?;
        if predictions_by_digest
            .insert(prediction.prediction_digest.clone(), prediction)
            .is_some()
        {
            return Err("historical prediction digest collision rejected".to_string());
        }
    }
    for index in 0..journal.fold_plan_digests.len() {
        let plan = decode_fold_plan(
            &fs::read(
                root.join("fold_plans")
                    .join(format!("{}.pb", journal.fold_plan_digests[index])),
            )
            .map_err(|_| "historical completed fold plan unavailable".to_string())?,
            registration,
        )?;
        if plan.fold_number != index as u64 {
            return Err("historical completed fold ordering rejected".to_string());
        }
        let capsule = decode_prediction_capsule_unbound(
            &fs::read(
                root.join("prediction_capsules")
                    .join(format!("{}.pb", journal.prediction_capsule_digests[index])),
            )
            .map_err(|_| "historical completed prediction capsule unavailable".to_string())?,
        )?;
        let predictions = capsule
            .prediction_digests
            .iter()
            .map(|digest| {
                predictions_by_digest
                    .get(digest)
                    .cloned()
                    .ok_or_else(|| "historical completed prediction seal unavailable".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        validate_prediction_capsule(&capsule, &predictions)?;
        let evaluation = decode_fold_evaluation_unbound(
            &fs::read(
                root.join("fold_evaluations")
                    .join(format!("{}.pb", journal.fold_evaluation_digests[index])),
            )
            .map_err(|_| "historical completed evaluation unavailable".to_string())?,
        )?;
        validate_fold_evaluation(&evaluation, &predictions)?;
        if capsule.fold_plan_digest != plan.fold_plan_digest
            || evaluation.fold_plan_digest != plan.fold_plan_digest
            || evaluation.prediction_capsule_digest != capsule.capsule_digest
        {
            return Err("historical completed fold linkage rejected".to_string());
        }
    }
    Ok(())
}

fn read_completed_replay(
    root: &Path,
    registration: &MomentumHistoricalReplayRegistrationV1,
) -> Result<
    Option<(
        MomentumHistoricalAggregateReportV1,
        MomentumHistoricalReplayJournalV1,
    )>,
    String,
> {
    let (aggregate_path, journal_path) = completed_paths(root, &registration.registration_digest);
    match (aggregate_path.exists(), journal_path.exists()) {
        (false, false) => Ok(None),
        (true, true) => {
            let aggregate = decode_aggregate(
                &fs::read(aggregate_path)
                    .map_err(|_| "historical completed aggregate unavailable".to_string())?,
            )?;
            let journal = decode_journal(
                &fs::read(journal_path)
                    .map_err(|_| "historical completed journal unavailable".to_string())?,
            )?;
            if aggregate.registration_digest != registration.registration_digest
                || aggregate.replay_mode != registration.replay_mode
                || journal.registration_digest != registration.registration_digest
                || journal.aggregate_digest != aggregate.aggregate_digest
            {
                return Err("historical completed replay conflict rejected".to_string());
            }
            verify_completed_fold_artifacts(root, registration, &aggregate, &journal)?;
            Ok(Some((aggregate, journal)))
        }
        _ => Err("historical incomplete completed replay rejected".to_string()),
    }
}

fn persist_completed_replay(
    root: &Path,
    aggregate: &MomentumHistoricalAggregateReportV1,
    journal: &MomentumHistoricalReplayJournalV1,
) -> Result<(usize, usize), String> {
    let (aggregate_path, journal_path) = completed_paths(root, &aggregate.registration_digest);
    let mut counts = (0, 0);
    add_write_counts(
        &mut counts,
        persist_artifact(
            &aggregate_path,
            &encode_aggregate(aggregate)?,
            &aggregate.aggregate_digest,
            |bytes| Ok(decode_aggregate(bytes)?.aggregate_digest),
        )?,
    );
    add_write_counts(
        &mut counts,
        persist_artifact(
            &journal_path,
            &encode_journal(journal)?,
            &journal.journal_digest,
            |bytes| Ok(decode_journal(bytes)?.journal_digest),
        )?,
    );
    Ok(counts)
}

pub fn historical_backfill_plan_status_v1() -> Result<MomentumHistoricalBackfillPlanV1, String> {
    let (_, snapshot) = discover_dataset(Path::new(DEFAULT_SNAPSHOT_ROOT))?;
    let plan = build_backfill_plan(&snapshot);
    validate_backfill_plan(&plan)?;
    Ok(plan)
}

pub fn run_momentum_historical_replay_v1(
    run_mode: MomentumHistoricalRunModeV1,
    replay_mode: MomentumHistoricalReplayModeV1,
) -> Result<MomentumHistoricalPublicReportV1, String> {
    run_momentum_historical_replay_at_v1(
        Path::new(DEFAULT_SNAPSHOT_ROOT),
        Path::new(DEFAULT_REPLAY_ROOT),
        Path::new(LIVE_PROTECTED_ROOT),
        run_mode,
        replay_mode,
    )
}

fn run_momentum_historical_replay_at_v1(
    snapshot_root: &Path,
    replay_root: &Path,
    live_protected_root: &Path,
    run_mode: MomentumHistoricalRunModeV1,
    replay_mode: MomentumHistoricalReplayModeV1,
) -> Result<MomentumHistoricalPublicReportV1, String> {
    let started = Instant::now();
    if replay_root.starts_with(live_protected_root) || live_protected_root.starts_with(replay_root)
    {
        return Err("historical replay isolation root rejected".to_string());
    }
    let protected_before = protected_tree_digest(live_protected_root)?;
    let active_before = active_roster_digest();
    let (source, snapshot) = discover_dataset(snapshot_root)?;
    let audit = build_contamination_audit(&snapshot);
    validate_contamination_audit(&audit)?;
    let rows = &source.normalized_dataset.rows;
    let (registration, eligible_indices) =
        build_registration(rows, &snapshot, &audit, replay_mode)?;
    let backfill_plan = build_backfill_plan(&snapshot);
    validate_backfill_plan(&backfill_plan)?;

    if run_mode != MomentumHistoricalRunModeV1::ExecuteLocal {
        let protected_unchanged = protected_before == protected_tree_digest(live_protected_root)?;
        let active_unchanged = active_before == active_roster_digest();
        if !protected_unchanged || !active_unchanged {
            return Err("historical read-only mode changed live state".to_string());
        }
        return Ok(public_report(
            run_mode,
            &snapshot,
            &audit,
            &registration,
            None,
            backfill_plan,
            false,
            (0, 0),
            u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            protected_unchanged,
            active_unchanged,
            stable_hash_string(&format!(
                "historical-{}:{}",
                run_mode.as_str(),
                registration.registration_digest
            )),
            eligible_indices.len(),
        ));
    }

    if let Some((aggregate, journal)) = read_completed_replay(replay_root, &registration)? {
        let protected_unchanged = protected_before == protected_tree_digest(live_protected_root)?;
        let active_unchanged = active_before == active_roster_digest();
        if !protected_unchanged || !active_unchanged {
            return Err("historical repeated replay changed live state".to_string());
        }
        return Ok(public_report(
            run_mode,
            &snapshot,
            &audit,
            &registration,
            Some(&aggregate),
            backfill_plan,
            true,
            (0, 0),
            u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            protected_unchanged,
            active_unchanged,
            journal.journal_digest,
            eligible_indices.len(),
        ));
    }

    let mut writes = (0, 0);
    add_write_counts(
        &mut writes,
        persist_dataset_snapshot(replay_root, &snapshot)?,
    );
    add_write_counts(
        &mut writes,
        persist_contamination_audit(replay_root, &audit)?,
    );
    add_write_counts(
        &mut writes,
        persist_registration(replay_root, &registration, rows.len())?,
    );
    add_write_counts(
        &mut writes,
        persist_backfill_plan(replay_root, &backfill_plan)?,
    );
    let registration_path = replay_root
        .join("registrations")
        .join(format!("{}.pb", registration.registration_digest));
    let reopened_registration = decode_registration(
        &fs::read(registration_path)
            .map_err(|_| "historical registration reopen failed".to_string())?,
        rows.len(),
    )?;
    if reopened_registration != registration {
        return Err("historical registration reopen identity rejected".to_string());
    }

    let campaign = MomentumLearningCampaignConfigV0::default();
    let mut fold_plan_digests = Vec::with_capacity(eligible_indices.len());
    let mut prediction_capsule_digests = Vec::with_capacity(eligible_indices.len());
    let mut fold_evaluation_digests = Vec::with_capacity(eligible_indices.len());
    let mut accumulators = BTreeMap::<String, ParticipantAccumulator>::new();
    let mut completed_fold_count = 0usize;
    let mut scorable_fold_count = 0usize;
    let mut neutral_fold_count = 0usize;
    let mut invalid_fold_count = 0usize;

    for (fold_number, prediction_event_index) in eligible_indices.iter().copied().enumerate() {
        let prefix = &rows[..=prediction_event_index];
        let examples = training_examples(
            prefix,
            &snapshot.snapshot_digest,
            &campaign.feature_config,
            &campaign.sequence_config,
        )?;
        let plan = build_fold_plan(
            fold_number,
            prediction_event_index,
            rows,
            &reopened_registration,
            &examples,
        )?;
        add_write_counts(
            &mut writes,
            persist_fold_plan(replay_root, &plan, &reopened_registration)?,
        );
        let predictions = match replay_mode {
            MomentumHistoricalReplayModeV1::ProtocolReplay => {
                protocol_predictions(&plan, &examples)?
            }
            MomentumHistoricalReplayModeV1::ExpandingWindowWalkForward => walk_forward_predictions(
                &plan,
                prefix,
                &examples,
                &reopened_registration,
                &campaign,
            )?,
        };
        for prediction in &predictions {
            add_write_counts(&mut writes, persist_prediction(replay_root, prediction)?);
        }
        let capsule = build_prediction_capsule(&plan, &predictions)?;
        add_write_counts(
            &mut writes,
            persist_prediction_capsule(replay_root, &capsule, &predictions)?,
        );
        let capsule_path = replay_root
            .join("prediction_capsules")
            .join(format!("{}.pb", capsule.capsule_digest));
        let reopened_capsule = decode_prediction_capsule(
            &fs::read(capsule_path)
                .map_err(|_| "historical prediction capsule reopen failed".to_string())?,
            &predictions,
        )?;
        if reopened_capsule != capsule {
            return Err("historical prediction capsule identity rejected".to_string());
        }

        // Phase B begins only after the exact prediction capsule has persisted and reopened.
        let target_row = &rows[plan.target_index];
        let prediction_row = &rows[plan.prediction_event_index];
        let (evaluation, private_evaluations) = build_fold_evaluation(
            &plan,
            &reopened_capsule,
            &predictions,
            prediction_row,
            target_row,
            &campaign.sequence_config,
        )?;
        add_write_counts(
            &mut writes,
            persist_fold_evaluation(replay_root, &evaluation, &predictions)?,
        );
        completed_fold_count += 1;
        match evaluation.label_status {
            HistoricalLabelStatusV1::ScorableBinaryOutcome => {
                scorable_fold_count += 1;
                for item in private_evaluations {
                    if !item.probability.is_finite()
                        || !(0.0..=1.0).contains(&item.probability)
                        || !item.label.is_finite()
                        || !item.brier.is_finite()
                    {
                        return Err("historical private metric rejected".to_string());
                    }
                    let accumulator = accumulators.entry(item.participant_id).or_default();
                    accumulator.brier_sum += item.brier;
                    accumulator.correct_count += usize::from(item.correct);
                    accumulator.count += 1;
                }
            }
            HistoricalLabelStatusV1::NeutralOutcomeExcluded => neutral_fold_count += 1,
            HistoricalLabelStatusV1::InvalidOutcomeEvidence => invalid_fold_count += 1,
        }
        fold_plan_digests.push(plan.fold_plan_digest);
        prediction_capsule_digests.push(capsule.capsule_digest);
        fold_evaluation_digests.push(evaluation.fold_evaluation_digest);
    }

    let protected_unchanged = protected_before == protected_tree_digest(live_protected_root)?;
    let active_unchanged = active_before == active_roster_digest();
    if !protected_unchanged || !active_unchanged {
        return Err("historical replay changed live protected state".to_string());
    }
    let aggregate = build_aggregate(
        &snapshot,
        &audit,
        &registration,
        eligible_indices.len(),
        completed_fold_count,
        scorable_fold_count,
        neutral_fold_count,
        invalid_fold_count,
        &accumulators,
        protected_unchanged,
        active_unchanged,
    )?;
    let mut journal = MomentumHistoricalReplayJournalV1 {
        journal_version: JOURNAL_VERSION.to_string(),
        registration_digest: registration.registration_digest.clone(),
        aggregate_digest: aggregate.aggregate_digest.clone(),
        fold_plan_digests,
        prediction_capsule_digests,
        fold_evaluation_digests,
        completed: true,
        journal_digest: String::new(),
    };
    journal.journal_digest = journal_digest(&journal);
    validate_journal(&journal)?;
    add_write_counts(
        &mut writes,
        persist_completed_replay(replay_root, &aggregate, &journal)?,
    );
    Ok(public_report(
        run_mode,
        &snapshot,
        &audit,
        &registration,
        Some(&aggregate),
        backfill_plan,
        false,
        writes,
        u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        protected_unchanged,
        active_unchanged,
        journal.journal_digest,
        eligible_indices.len(),
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::{
        core::ReasonCode,
        data::{
            AcquisitionMarketScope, DataLookback, DatasetKind, SnapshotAdjustmentSemanticsV1,
            SnapshotCompatibilityV1, SnapshotProvenance, SnapshotQualitySummary,
            SnapshotSourceType,
        },
        league::HistoricalReplayDataset,
    };

    use super::*;

    static TEST_ID: AtomicU64 = AtomicU64::new(0);

    fn fixture_source(row_count: usize) -> DataSnapshot {
        let rows = (0..row_count)
            .map(|index| {
                let close = 50_000.0 + index as f64 * 250.0 + (index % 3) as f64 * 2.0;
                HistoricalOhlcvRow {
                    symbol: "KRW-BTC".to_string(),
                    timestamp_ms: 1_700_000_000_000 + index as u64 * DAILY_CADENCE_MS,
                    open: close - 10.0,
                    high: close + 25.0,
                    low: close - 25.0,
                    close,
                    volume: 100.0 + index as f64,
                    trade_value: Some(close * (100.0 + index as f64)),
                }
            })
            .collect::<Vec<_>>();
        let dataset = HistoricalReplayDataset {
            symbol: "KRW-BTC".to_string(),
            rows,
            source: "approved-read-only-provider".to_string(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        };
        let content_digest = historical_replay_dataset_digest_v0(&dataset);
        DataSnapshot {
            snapshot_id: "historical-fixture".to_string(),
            request_key: "fixture-request".to_string(),
            provider_id: "upbit-public".to_string(),
            dataset_kind: DatasetKind::CryptoDailyOhlcv,
            market_scope: AcquisitionMarketScope::BtcCrypto,
            symbols: vec!["KRW-BTC".to_string()],
            requested_lookback: DataLookback {
                bars: row_count,
                start_timestamp_ms: dataset.rows.first().map(|row| row.timestamp_ms),
                end_timestamp_ms: dataset.rows.last().map(|row| row.timestamp_ms),
            },
            actual_start_timestamp_ms: dataset.rows.first().map(|row| row.timestamp_ms),
            actual_end_timestamp_ms: dataset.rows.last().map(|row| row.timestamp_ms),
            fetched_at_ms: 1_800_000_000_000,
            normalized_at_ms: 1_800_000_000_001,
            schema_version: 1,
            row_count,
            quality_summary: SnapshotQualitySummary {
                accepted: true,
                row_count,
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            content_digest,
            sanitized: true,
            read_only: true,
            compatibility: Some(SnapshotCompatibilityV1 {
                cadence: "1d".to_string(),
                adjustment_semantics: SnapshotAdjustmentSemanticsV1::NotApplicable,
                source_schema: "application/x-soma-normalized-dataset".to_string(),
                requested_cutoff_timestamp_ms: dataset.rows.last().map(|row| row.timestamp_ms),
                maximum_staleness_ms: DAILY_CADENCE_MS,
                all_rows_finalized: true,
            }),
            normalized_dataset: dataset,
            provenance: SnapshotProvenance {
                provider_id: "upbit-public".to_string(),
                acquisition_request_id: "fixture-acquisition".to_string(),
                fetch_receipt_id: "fixture-receipt".to_string(),
                source_type: SnapshotSourceType::ApprovedReadOnlyProvider,
                sanitized: true,
                credential_free: true,
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }

    fn fixture_registration() -> (
        DataSnapshot,
        MomentumHistoricalDatasetSnapshotV1,
        MomentumHistoricalContaminationAuditV1,
        MomentumHistoricalReplayRegistrationV1,
        Vec<usize>,
    ) {
        let source = fixture_source(100);
        let snapshot = build_dataset_snapshot(&source).unwrap();
        let audit = build_contamination_audit(&snapshot);
        let (registration, eligible) = build_registration(
            &source.normalized_dataset.rows,
            &snapshot,
            &audit,
            MomentumHistoricalReplayModeV1::ExpandingWindowWalkForward,
        )
        .unwrap();
        (source, snapshot, audit, registration, eligible)
    }

    fn fixture_fold() -> (
        DataSnapshot,
        MomentumHistoricalDatasetSnapshotV1,
        MomentumHistoricalReplayRegistrationV1,
        Vec<super::super::SequenceExampleV0>,
        MomentumHistoricalFoldPlanV1,
    ) {
        let (source, snapshot, _, registration, eligible) = fixture_registration();
        let index = eligible[0];
        let campaign = MomentumLearningCampaignConfigV0::default();
        let examples = training_examples(
            &source.normalized_dataset.rows[..=index],
            &snapshot.snapshot_digest,
            &campaign.feature_config,
            &campaign.sequence_config,
        )
        .unwrap();
        let plan = build_fold_plan(
            0,
            index,
            &source.normalized_dataset.rows,
            &registration,
            &examples,
        )
        .unwrap();
        (source, snapshot, registration, examples, plan)
    }

    fn fixture_predictions() -> (
        MomentumHistoricalFoldPlanV1,
        Vec<MomentumHistoricalFoldPredictionV1>,
        MomentumHistoricalFoldPredictionCapsuleV1,
    ) {
        let (_, _, _, examples, plan) = fixture_fold();
        let predictions = protocol_predictions(&plan, &examples).unwrap();
        let capsule = build_prediction_capsule(&plan, &predictions).unwrap();
        (plan, predictions, capsule)
    }

    fn fixture_aggregate() -> (
        MomentumHistoricalAggregateReportV1,
        MomentumHistoricalReplayJournalV1,
    ) {
        let (_, snapshot, audit, registration, _) = fixture_registration();
        let mut accumulators = BTreeMap::new();
        accumulators.insert(
            RAW_PARTICIPANT.to_string(),
            ParticipantAccumulator {
                brier_sum: 1.2,
                correct_count: 6,
                count: 8,
            },
        );
        accumulators.insert(
            INTERACTION_PARTICIPANT.to_string(),
            ParticipantAccumulator {
                brier_sum: 1.4,
                correct_count: 5,
                count: 8,
            },
        );
        accumulators.insert(
            CONSTANT_PARTICIPANT.to_string(),
            ParticipantAccumulator {
                brier_sum: 1.6,
                correct_count: 4,
                count: 8,
            },
        );
        let aggregate = build_aggregate(
            &snapshot,
            &audit,
            &registration,
            8,
            8,
            8,
            0,
            0,
            &accumulators,
            true,
            true,
        )
        .unwrap();
        let mut journal = MomentumHistoricalReplayJournalV1 {
            journal_version: JOURNAL_VERSION.to_string(),
            registration_digest: registration.registration_digest,
            aggregate_digest: aggregate.aggregate_digest.clone(),
            fold_plan_digests: vec!["fold".to_string()],
            prediction_capsule_digests: vec!["capsule".to_string()],
            fold_evaluation_digests: vec!["evaluation".to_string()],
            completed: true,
            journal_digest: String::new(),
        };
        journal.journal_digest = journal_digest(&journal);
        (aggregate, journal)
    }

    fn test_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "soma-historical-replay-{label}-{}-{}",
            std::process::id(),
            TEST_ID.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn sprint94_01_dataset_snapshot_derives_ordered_canonical_rows() {
        let source = fixture_source(100);
        let snapshot = build_dataset_snapshot(&source).unwrap();
        assert_eq!(snapshot.row_count, 100);
        assert_eq!(
            snapshot.ordered_row_digests,
            source
                .normalized_dataset
                .rows
                .iter()
                .map(row_digest)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn sprint94_02_duplicate_historical_timestamp_rejects() {
        let mut source = fixture_source(100);
        source.normalized_dataset.rows[10].timestamp_ms =
            source.normalized_dataset.rows[9].timestamp_ms;
        source.content_digest = historical_replay_dataset_digest_v0(&source.normalized_dataset);
        assert!(validate_dataset_source(&source).is_err());
    }

    #[test]
    fn sprint94_03_malformed_historical_row_rejects() {
        let mut source = fixture_source(100);
        source.normalized_dataset.rows[4].high = source.normalized_dataset.rows[4].low - 1.0;
        source.content_digest = historical_replay_dataset_digest_v0(&source.normalized_dataset);
        assert!(validate_dataset_source(&source).is_err());
    }

    #[test]
    fn sprint94_04_contamination_class_is_not_blind_holdout() {
        let snapshot = build_dataset_snapshot(&fixture_source(100)).unwrap();
        assert_eq!(
            snapshot.evidence_use_class,
            HistoricalEvidenceUseClassV1::PreviouslyConsumedResearchEvidence
        );
        assert!(snapshot.previously_consumed);
        assert!(!snapshot.blind_holdout);
    }

    #[test]
    fn sprint94_05_historical_authority_eligibility_is_false() {
        let snapshot = build_dataset_snapshot(&fixture_source(100)).unwrap();
        assert!(!snapshot.authority_eligible);
        assert!(build_contamination_audit(&snapshot).live_authority_use_forbidden);
    }

    #[test]
    fn sprint94_06_registration_persists_before_folds() {
        let (_, _, _, registration, _) = fixture_registration();
        let root = test_root("registration-first");
        let counts = persist_registration(&root, &registration, 100).unwrap();
        assert_eq!(counts, (1, 0));
        assert!(!root.join("fold_plans").exists());
        let bytes = fs::read(
            root.join("registrations")
                .join(format!("{}.pb", registration.registration_digest)),
        )
        .unwrap();
        assert_eq!(decode_registration(&bytes, 100).unwrap(), registration);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn sprint94_07_result_conditioned_fold_selection_is_forbidden() {
        let (_, _, _, registration, _) = fixture_registration();
        assert!(registration.result_conditioned_fold_selection_forbidden);
        assert_eq!(
            registration.fold_selection_policy,
            MomentumHistoricalFoldSelectionPolicyV1::EveryChronologicallyEligibleEvent
        );
    }

    #[test]
    fn sprint94_08_result_conditioned_hyperparameters_are_forbidden() {
        let (_, _, _, registration, _) = fixture_registration();
        assert!(registration.result_conditioned_hyperparameters_forbidden);
    }

    #[test]
    fn sprint94_09_every_fold_is_chronological() {
        let (_, _, registration, _, plan) = fixture_fold();
        validate_fold_plan(&plan, &registration).unwrap();
        assert!(plan.target_timestamp_ms > plan.prediction_event_timestamp_ms);
    }

    #[test]
    fn sprint94_10_context_has_registered_row_count() {
        let (_, _, registration, _, plan) = fixture_fold();
        assert_eq!(
            plan.context_timestamp_ms.len(),
            registration.context_row_count
        );
    }

    #[test]
    fn sprint94_11_target_is_after_prediction_event() {
        let (_, _, _, _, plan) = fixture_fold();
        assert_eq!(plan.target_index, plan.prediction_event_index + 1);
        assert!(plan.target_timestamp_ms > plan.prediction_event_timestamp_ms);
    }

    #[test]
    fn sprint94_12_latest_training_label_is_observable() {
        let (_, _, _, examples, plan) = fixture_fold();
        assert!(examples.iter().all(|example| {
            example.label_index <= plan.prediction_event_index
                && example.sequence_end < plan.prediction_event_index
        }));
        assert!(plan.latest_training_label_timestamp_ms <= plan.prediction_event_timestamp_ms);
    }

    #[test]
    fn sprint94_13_future_row_cannot_enter_training_features() {
        let (_, _, _, examples, plan) = fixture_fold();
        assert!(
            examples
                .iter()
                .all(|example| example.sequence_end < plan.prediction_event_index)
        );
    }

    #[test]
    fn sprint94_14_future_row_cannot_enter_normalizer_fitting() {
        let (mut source, snapshot, registration, examples, plan) = fixture_fold();
        let campaign = MomentumLearningCampaignConfigV0::default();
        let first = walk_forward_predictions(
            &plan,
            &source.normalized_dataset.rows[..=plan.prediction_event_index],
            &examples,
            &registration,
            &campaign,
        )
        .unwrap();
        source.normalized_dataset.rows[plan.target_index].close *= 5.0;
        let second = walk_forward_predictions(
            &plan,
            &source.normalized_dataset.rows[..=plan.prediction_event_index],
            &examples,
            &registration,
            &campaign,
        )
        .unwrap();
        assert_eq!(first, second);
        assert!(!snapshot.snapshot_digest.is_empty());
    }

    #[test]
    fn sprint94_15_random_split_is_unavailable() {
        let (_, _, _, registration, _) = fixture_registration();
        assert_eq!(
            format!("{:?}", registration.fold_selection_policy),
            "EveryChronologicallyEligibleEvent"
        );
    }

    #[test]
    fn sprint94_16_prediction_persists_before_target_reveal() {
        let (_, predictions, capsule) = fixture_predictions();
        assert!(predictions.iter().all(|value| !value.target_accessed));
        assert!(!capsule.target_accessed);
        assert!(!capsule.label_derived);
        assert!(!capsule.metrics_computed);
    }

    #[test]
    fn sprint94_17_target_access_before_seal_rejects() {
        let (_, mut predictions, _) = fixture_predictions();
        predictions[0].target_accessed = true;
        assert!(validate_prediction(&predictions[0]).is_err());
    }

    #[test]
    fn sprint94_18_neutral_labels_remain_excluded() {
        let source = fixture_source(100);
        let row = &source.normalized_dataset.rows[30];
        let mut target = source.normalized_dataset.rows[31].clone();
        target.close = row.close * (1.0 + 0.0005);
        assert_eq!(
            classify_label(row, &target, &MomentumSequenceConfigV0::default()).unwrap(),
            (HistoricalLabelStatusV1::NeutralOutcomeExcluded, None)
        );
    }

    #[test]
    fn sprint94_19_invalid_outcome_evidence_is_preserved() {
        let (source, _, _, _, plan) = fixture_fold();
        let (_, predictions, capsule) = fixture_predictions();
        let mut target = source.normalized_dataset.rows[plan.target_index].clone();
        target.close = f64::NAN;
        let (evaluation, private) = build_fold_evaluation(
            &plan,
            &capsule,
            &predictions,
            &source.normalized_dataset.rows[plan.prediction_event_index],
            &target,
            &MomentumSequenceConfigV0::default(),
        )
        .unwrap();
        assert_eq!(
            evaluation.label_status,
            HistoricalLabelStatusV1::InvalidOutcomeEvidence
        );
        assert!(private.is_empty());
    }

    #[test]
    fn sprint94_20_fold_local_raw_model_trains_on_past_only() {
        let (source, _, registration, examples, plan) = fixture_fold();
        let predictions = walk_forward_predictions(
            &plan,
            &source.normalized_dataset.rows[..=plan.prediction_event_index],
            &examples,
            &registration,
            &MomentumLearningCampaignConfigV0::default(),
        )
        .unwrap();
        assert!(
            predictions
                .iter()
                .any(|value| value.participant_id == RAW_PARTICIPANT)
        );
    }

    #[test]
    fn sprint94_21_fold_local_interaction_model_trains_on_past_only() {
        let (source, _, registration, examples, plan) = fixture_fold();
        let predictions = walk_forward_predictions(
            &plan,
            &source.normalized_dataset.rows[..=plan.prediction_event_index],
            &examples,
            &registration,
            &MomentumLearningCampaignConfigV0::default(),
        )
        .unwrap();
        assert!(
            predictions
                .iter()
                .any(|value| value.participant_id == INTERACTION_PARTICIPANT)
        );
    }

    #[test]
    fn sprint94_22_constant_uses_past_labels_only() {
        let (mut source, _, _, examples, plan) = fixture_fold();
        let first = protocol_predictions(&plan, &examples).unwrap();
        source.normalized_dataset.rows[plan.target_index].close *= 10.0;
        let second = protocol_predictions(&plan, &examples).unwrap();
        let first_constant = first
            .iter()
            .find(|value| value.participant_id == CONSTANT_PARTICIPANT)
            .unwrap();
        let second_constant = second
            .iter()
            .find(|value| value.participant_id == CONSTANT_PARTICIPANT)
            .unwrap();
        assert_eq!(first_constant, second_constant);
    }

    #[test]
    fn sprint94_23_fold_replicas_cannot_reuse_live_parameters() {
        let (plan, _, _) = fixture_predictions();
        assert!(
            build_prediction(
                &plan,
                RAW_PARTICIPANT,
                "HistoricalResearchReplica",
                "live-parameter".to_string(),
                "historical-research:normalizer".to_string(),
                "feature".to_string(),
                0.5,
            )
            .is_err()
        );
    }

    #[test]
    fn sprint94_24_fold_replicas_cannot_reuse_live_normalizers() {
        let (plan, _, _) = fixture_predictions();
        assert!(
            build_prediction(
                &plan,
                RAW_PARTICIPANT,
                "HistoricalResearchReplica",
                "historical-research:parameter".to_string(),
                "live-normalizer".to_string(),
                "feature".to_string(),
                0.5,
            )
            .is_err()
        );
    }

    #[test]
    fn sprint94_25_live_event_private_results_are_inaccessible() {
        let (_, predictions, _) = fixture_predictions();
        assert!(predictions.iter().all(|value| {
            value.parameter_digest.starts_with("historical-research:")
                && value.normalizer_digest.starts_with("historical-research:")
        }));
        assert!(!DEFAULT_REPLAY_ROOT.starts_with(LIVE_PROTECTED_ROOT));
    }

    #[test]
    fn sprint94_26_exactly_three_fold_predictions_are_required() {
        let (_, predictions, capsule) = fixture_predictions();
        assert_eq!(predictions.len(), 3);
        assert_eq!(capsule.prediction_count, 3);
    }

    #[test]
    fn sprint94_27_partial_fold_capsule_rejects() {
        let (plan, predictions, _) = fixture_predictions();
        assert!(build_prediction_capsule(&plan, &predictions[..2]).is_err());
    }

    #[test]
    fn sprint94_28_per_fold_predictions_remain_private() {
        let (aggregate, _) = fixture_aggregate();
        let public = serde_json::to_string(&aggregate).unwrap();
        for forbidden in [
            "private_prediction",
            "private_label",
            "feature_digest",
            "parameter_digest",
            "normalizer_digest",
            "prediction_event_timestamp",
        ] {
            assert!(!public.contains(forbidden));
        }
    }

    #[test]
    fn sprint94_29_aggregate_fold_counts_derive_exactly() {
        let (aggregate, _) = fixture_aggregate();
        assert_eq!(aggregate.eligible_fold_count, 8);
        assert_eq!(aggregate.completed_fold_count, 8);
        assert_eq!(
            aggregate.scorable_fold_count
                + aggregate.neutral_fold_count
                + aggregate.invalid_fold_count,
            8
        );
    }

    #[test]
    fn sprint94_30_aggregate_brier_uses_scorable_folds_only() {
        let (aggregate, _) = fixture_aggregate();
        let raw = aggregate
            .participants
            .iter()
            .find(|value| value.participant_id == RAW_PARTICIPANT)
            .unwrap();
        assert_eq!(raw.mean_brier_score, 1.2 / 8.0);
        assert_eq!(raw.scorable_fold_count, aggregate.scorable_fold_count);
    }

    #[test]
    fn sprint94_31_correctness_excludes_neutral_folds() {
        let (aggregate, _) = fixture_aggregate();
        let raw = aggregate
            .participants
            .iter()
            .find(|value| value.participant_id == RAW_PARTICIPANT)
            .unwrap();
        assert_eq!(raw.binary_correctness_rate, 6.0 / 8.0);
    }

    #[test]
    fn sprint94_32_benchmark_comparison_is_research_only() {
        let (aggregate, _) = fixture_aggregate();
        assert!(
            aggregate
                .participants
                .iter()
                .all(|participant| participant.research_only)
        );
        assert_eq!(
            aggregate.evidence_labels,
            RESEARCH_LABELS.map(str::to_string)
        );
    }

    #[test]
    fn sprint94_33_no_historical_winner_is_selected() {
        let (aggregate, _) = fixture_aggregate();
        assert!(!aggregate.winner_selected);
        assert_eq!(aggregate.safety_counters.winner_selections, 0);
        assert_eq!(aggregate.safety_counters.rankings, 0);
    }

    #[test]
    fn sprint94_34_live_prospective_counts_remain_unchanged() {
        let safety = MomentumHistoricalSafetyCountersV1::zero_authority();
        assert_eq!(safety.live_prospective_event_count_changes, 0);
        assert_eq!(safety.live_prospective_scorable_count_changes, 0);
    }

    #[test]
    fn sprint94_35_active_roster_remains_unchanged() {
        let before = active_roster_digest();
        let _ = fixture_aggregate();
        assert_eq!(before, active_roster_digest());
        assert_eq!(
            MomentumHistoricalSafetyCountersV1::zero_authority().active_committee_count,
            3
        );
    }

    #[test]
    fn sprint94_36_reward_and_chair_counters_remain_zero() {
        let safety = MomentumHistoricalSafetyCountersV1::zero_authority();
        assert_eq!(safety.reward_applications, 0);
        assert_eq!(safety.penalty_applications, 0);
        assert_eq!(safety.chair_decisions, 0);
        assert_eq!(safety.committee_votes, 0);
    }

    #[test]
    fn sprint94_37_trading_simulation_is_blocked() {
        let (aggregate, _) = fixture_aggregate();
        assert_eq!(
            aggregate.trading_simulation_status,
            MomentumHistoricalTradingSimulationStatusV1::BlockedNoFrozenExecutionPolicy
        );
        assert_eq!(aggregate.safety_counters.paper_executions, 0);
        assert_eq!(aggregate.safety_counters.live_executions, 0);
    }

    #[test]
    fn sprint94_38_backfill_plan_performs_zero_network() {
        let snapshot = build_dataset_snapshot(&fixture_source(100)).unwrap();
        let plan = build_backfill_plan(&snapshot);
        validate_backfill_plan(&plan).unwrap();
        assert_eq!(plan.request_count_upper_bound, 0);
        assert_eq!(plan.maximum_concurrency, 1);
        assert_eq!(plan.maximum_retries, 0);
        assert!(plan.explicit_network_authorization_required);
        assert!(!plan.executed);
    }

    #[test]
    fn sprint94_39_repeated_replay_identity_is_deterministic() {
        let first = fixture_registration();
        let second = fixture_registration();
        assert_eq!(first.1, second.1);
        assert_eq!(first.2, second.2);
        assert_eq!(first.3, second.3);
        assert_eq!(first.4, second.4);
        let first_predictions = fixture_predictions();
        let second_predictions = fixture_predictions();
        assert_eq!(first_predictions, second_predictions);
    }

    #[test]
    fn sprint94_40_duplicate_completed_replay_performs_zero_writes() {
        let (aggregate, journal) = fixture_aggregate();
        let root = test_root("duplicate");
        assert_eq!(
            persist_completed_replay(&root, &aggregate, &journal).unwrap(),
            (2, 0)
        );
        assert_eq!(
            persist_completed_replay(&root, &aggregate, &journal).unwrap(),
            (0, 2)
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn sprint94_41_conflicting_replay_artifact_rejects() {
        let (_, _, _, registration, _) = fixture_registration();
        let (aggregate, journal) = fixture_aggregate();
        let completed_root = test_root("completed-conflict");
        persist_completed_replay(&completed_root, &aggregate, &journal).unwrap();
        assert!(read_completed_replay(&completed_root, &registration).is_err());
        let _ = fs::remove_dir_all(completed_root);

        let snapshot = build_dataset_snapshot(&fixture_source(100)).unwrap();
        let root = test_root("conflict");
        let directory = root.join("dataset_snapshots");
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join(format!("{}.pb", snapshot.snapshot_digest)),
            b"conflict",
        )
        .unwrap();
        assert!(persist_dataset_snapshot(&root, &snapshot).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn sprint94_42_malformed_protobuf_rejects() {
        assert!(decode_dataset_snapshot(b"not-protobuf").is_err());
        assert!(decode_contamination_audit(b"not-protobuf").is_err());
        assert!(decode_aggregate(b"not-protobuf").is_err());
        assert!(decode_backfill_plan(b"not-protobuf").is_err());
    }
}
